//! View property frames use the host's existing clock; listeners run in guest DEX.
use crate::{
    heap::{Data, Word, bits64, fault, wide},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

const VPA: &str = "Landroid/view/ViewPropertyAnimator;";
const VALUE: &str = "Landroid/animation/ValueAnimator;";
const VIEW: &str = "Landroid/view/View;";
const OWNER: &str = "droidless:property:owner";
const TARGET: &str = "droidless:property:view";
const SCALAR: &str = "droidless:value:scalar";
const PROPERTIES: [&str; 3] = ["alpha", "translationX", "translationY"];
const LIMIT: usize = 16_384;

#[derive(Clone)]
struct Animation {
    owner: Word,
    animator: Word,
    deadline: u64,
    duration: u64,
    interpolator: Word,
    started: bool,
    finishing: bool,
    properties: Vec<(usize, f32, f32)>,
}
#[derive(Default)]
pub(crate) struct PropertyAnimations {
    pending: Vec<Word>,
    running: Vec<Animation>,
}
impl PropertyAnimations {
    pub(crate) fn roots(&self) -> impl Iterator<Item = Word> + '_ {
        self.pending.iter().copied().chain(
            self.running
                .iter()
                .flat_map(|a| [a.owner, a.animator, a.interpolator]),
        )
    }
}

impl Runtime {
    fn property_word(&self, object: Word, name: &str) -> Result<Word> {
        Ok(self
            .heap
            .get(object)?
            .fields
            .get(name)
            .and_then(|v| v.first())
            .copied()
            .unwrap_or(Word::ZERO))
    }
    fn property_time(&self, object: Word, name: &str) -> Result<u64> {
        bits64(
            self.heap
                .get(object)?
                .fields
                .get(name)
                .context("uninitialized animation time")?,
        )
    }
    fn property_view_call(
        &mut self,
        view: Word,
        name: &str,
        parameters: Vec<String>,
        returns: &str,
        values: Vec<Word>,
    ) -> Result<Vec<Word>> {
        // Property animation changes the base render state, independently of APK setter overrides.
        self.invoke(
            Method {
                class: VIEW.into(),
                name: name.into(),
                parameters,
                returns: returns.into(),
            },
            std::iter::once(view).chain(values).collect(),
            false,
        )
    }
    pub(crate) fn property_animation_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method.signature();
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        if matches!(
            method.class.as_str(),
            VALUE | "Landroid/animation/Animator;"
        ) && receiver != Word::ZERO
            && (signature == "<init>()V" || self.property_word(receiver, SCALAR)?.truth())
        {
            self.require_main_thread()?;
            ensure!(
                self.sync_depth < 32,
                "value animation callback nesting limit"
            );
            let roots = self.native_roots.len();
            self.native_roots.extend(args.iter().copied());
            let result = self.scalar_animation_native(method, args);
            self.native_roots.truncate(roots);
            return result;
        }
        if method.class == VIEW && signature == "animate()Landroid/view/ViewPropertyAnimator;" {
            self.require_main_thread()?;
            self.heap
                .get(receiver)?
                .view
                .as_ref()
                .context("animation requires View")?;
            let cached = self.property_word(receiver, "droidless:view:property-animator")?;
            if cached != Word::ZERO {
                return Ok(Some(vec![cached]));
            }
            let animator = self.heap.instance(VPA)?;
            let interpolator = self
                .heap
                .instance("Landroid/view/animation/AccelerateDecelerateInterpolator;")?;
            let fields = &mut self.heap.get_mut(animator)?.fields;
            fields.insert(TARGET.into(), vec![receiver]);
            fields.insert("duration".into(), wide(300));
            fields.insert("delay".into(), wide(0));
            fields.insert("interpolator".into(), vec![interpolator]);
            self.heap
                .get_mut(receiver)?
                .fields
                .insert("droidless:view:property-animator".into(), vec![animator]);
            return Ok(Some(vec![animator]));
        }
        if matches!(
            method.class.as_str(),
            VALUE | "Landroid/animation/Animator;"
        ) && receiver != Word::ZERO
            && receiver.reference().is_ok()
            && self.property_word(receiver, OWNER)? != Word::ZERO
        {
            self.require_main_thread()?;
            let running = self
                .property_animations
                .running
                .iter()
                .find(|a| a.animator == receiver);
            let result = match signature.as_str() {
                "getAnimatedFraction()F" => vec![self.property_word(receiver, "fraction")?],
                "isStarted()Z" => vec![Word::from(i32::from(running.is_some()))],
                "isRunning()Z" => {
                    vec![Word::from(i32::from(running.is_some_and(|a| {
                        a.started && self.uptime_ms() >= a.deadline
                    })))]
                }
                "getDuration()J" => wide(self.property_time(receiver, "duration")?),
                "getStartDelay()J" => wide(self.property_time(receiver, "delay")?),
                "getInterpolator()Landroid/animation/TimeInterpolator;" => {
                    vec![self.property_word(receiver, "interpolator")?]
                }
                "setInterpolator(Landroid/animation/TimeInterpolator;)V" => {
                    let value = args
                        .get(1)
                        .copied()
                        .context("missing animation interpolator")?;
                    if value != Word::ZERO {
                        ensure!(
                            self.is_a(
                                &self.heap.get(value)?.class,
                                "Landroid/animation/TimeInterpolator;"
                            ),
                            "invalid animation interpolator"
                        );
                    }
                    let retained = if value == Word::ZERO {
                        self.heap
                            .instance("Landroid/view/animation/LinearInterpolator;")?
                    } else {
                        value
                    };
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("interpolator".into(), vec![retained]);
                    if let Some(animation) = self
                        .property_animations
                        .running
                        .iter_mut()
                        .find(|a| a.animator == receiver)
                    {
                        animation.interpolator = value;
                    }
                    vec![]
                }
                "cancel()V" => {
                    self.finish_property_animation(receiver, true)?;
                    vec![]
                }
                // This owned ValueAnimator is the callback's progress token, not a general factory.
                _ => return Ok(None),
            };
            return Ok(Some(result));
        }
        if method.class != VPA {
            return Ok(None);
        }
        self.require_main_thread()?;
        let target = self.property_word(receiver, TARGET)?;
        self.heap
            .get(target)?
            .view
            .as_ref()
            .context("uninitialized ViewPropertyAnimator")?;
        let arg = |i| {
            args.get(i)
                .copied()
                .context("missing property animation argument")
        };
        let result = match signature.as_str() {
            "alpha(F)Landroid/view/ViewPropertyAnimator;"
            | "translationX(F)Landroid/view/ViewPropertyAnimator;"
            | "translationY(F)Landroid/view/ViewPropertyAnimator;" => {
                ensure!(
                    !self.queue.closed,
                    "property animation after runtime shutdown"
                );
                let property = PROPERTIES
                    .iter()
                    .position(|name| *name == method.name)
                    .context("unknown animated property")?;
                let value = f32::from_bits(arg(1)?.int()? as u32);
                ensure!(
                    value.is_finite() && (property == 0 || value.abs() <= 1_000_000.0),
                    "invalid animated property value"
                );
                let getter = format!(
                    "get{}{}",
                    PROPERTIES[property][..1].to_ascii_uppercase(),
                    &PROPERTIES[property][1..]
                );
                let from = self.property_view_call(target, &getter, vec![], "F", vec![])?[0];
                let affected: Vec<_> = self
                    .property_animations
                    .running
                    .iter()
                    .filter(|a| a.owner == receiver && a.properties.iter().any(|p| p.0 == property))
                    .map(|a| a.animator)
                    .collect();
                for animator in affected {
                    if let Some(animation) = self
                        .property_animations
                        .running
                        .iter_mut()
                        .find(|a| a.animator == animator)
                    {
                        animation.properties.retain(|p| p.0 != property);
                        if animation.properties.is_empty() {
                            self.finish_property_animation(animator, true)?;
                        }
                    }
                }
                if !self.property_animations.pending.contains(&receiver) {
                    ensure!(
                        self.property_animations.pending.len()
                            + self.property_animations.running.len()
                            < LIMIT,
                        "property animation capacity reached"
                    );
                    self.property_animations.pending.push(receiver);
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(format!("pending:{property}"), vec![from, arg(1)?]);
                vec![receiver]
            }
            "setDuration(J)Landroid/view/ViewPropertyAnimator;"
            | "setStartDelay(J)Landroid/view/ViewPropertyAnimator;" => {
                let time = bits64(args.get(1..3).context("missing animation interval")?)?;
                if time > i64::MAX as u64 {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "animation interval cannot be negative",
                    ));
                }
                let key = if method.name == "setDuration" {
                    "duration"
                } else {
                    "delay"
                };
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(key.into(), wide(time));
                vec![receiver]
            }
            "getDuration()J" => wide(self.property_time(receiver, "duration")?),
            "getStartDelay()J" => wide(self.property_time(receiver, "delay")?),
            "getInterpolator()Landroid/animation/TimeInterpolator;" => {
                vec![self.property_word(receiver, "interpolator")?]
            }
            "setInterpolator(Landroid/animation/TimeInterpolator;)Landroid/view/ViewPropertyAnimator;"
            | "setListener(Landroid/animation/Animator$AnimatorListener;)Landroid/view/ViewPropertyAnimator;"
            | "setUpdateListener(Landroid/animation/ValueAnimator$AnimatorUpdateListener;)Landroid/view/ViewPropertyAnimator;" =>
            {
                let value = arg(1)?;
                if value != Word::ZERO {
                    ensure!(
                        self.is_a(&self.heap.get(value)?.class, &method.parameters[0]),
                        "invalid property animation callback"
                    );
                }
                let key = match method.name.as_str() {
                    "setInterpolator" => "interpolator",
                    "setListener" => "listener",
                    _ => "update",
                };
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(key.into(), vec![value]);
                vec![receiver]
            }
            "start()V" => {
                self.start_property_animation(receiver)?;
                vec![]
            }
            "cancel()V" => {
                let running: Vec<_> = self
                    .property_animations
                    .running
                    .iter()
                    .filter(|a| a.owner == receiver)
                    .map(|a| a.animator)
                    .collect();
                for animator in running {
                    self.finish_property_animation(animator, true)?;
                }
                self.property_animations
                    .pending
                    .retain(|owner| *owner != receiver);
                for property in 0..PROPERTIES.len() {
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .remove(&format!("pending:{property}"));
                }
                vec![]
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    fn property_event(&mut self, owner: Word, animator: Word, name: &str) -> Result<()> {
        let update = name == "onAnimationUpdate";
        if self.property_word(owner, SCALAR)?.truth() {
            let key = if update { "updates" } else { "listeners" };
            let listeners = self
                .heap
                .get(owner)?
                .fields
                .get(key)
                .cloned()
                .unwrap_or_default();
            let roots = self.native_roots.len();
            if !update {
                self.native_roots.extend(listeners.iter().copied());
            }
            let result = (|| -> Result<()> {
                for (index, snapshot) in listeners.into_iter().enumerate() {
                    let listener = if update {
                        self.heap
                            .get(owner)?
                            .fields
                            .get(key)
                            .and_then(|v| v.get(index))
                            .copied()
                            .ok_or_else(|| {
                                fault(
                                    "Ljava/lang/IndexOutOfBoundsException;",
                                    "animation listeners changed during update",
                                )
                            })?
                    } else {
                        snapshot
                    };
                    if listener == Word::ZERO {
                        return Err(fault(
                            "Ljava/lang/NullPointerException;",
                            "null animation listener",
                        ));
                    }
                    self.invoke(
                        Method {
                            class: if update {
                                "Landroid/animation/ValueAnimator$AnimatorUpdateListener;"
                            } else {
                                "Landroid/animation/Animator$AnimatorListener;"
                            }
                            .into(),
                            name: name.into(),
                            parameters: vec![
                                if update {
                                    VALUE
                                } else {
                                    "Landroid/animation/Animator;"
                                }
                                .into(),
                            ],
                            returns: "V".into(),
                        },
                        vec![listener, animator],
                        true,
                    )?;
                }
                Ok(())
            })();
            self.native_roots.truncate(roots);
            return result;
        }
        let listener = self.property_word(owner, if update { "update" } else { "listener" })?;
        if listener == Word::ZERO {
            return Ok(());
        }
        self.invoke(
            Method {
                class: if update {
                    "Landroid/animation/ValueAnimator$AnimatorUpdateListener;"
                } else {
                    "Landroid/animation/Animator$AnimatorListener;"
                }
                .into(),
                name: name.into(),
                parameters: vec![
                    if update {
                        VALUE
                    } else {
                        "Landroid/animation/Animator;"
                    }
                    .into(),
                ],
                returns: "V".into(),
            },
            vec![listener, animator],
            true,
        )?;
        Ok(())
    }
    fn start_property_animation(&mut self, owner: Word) -> Result<()> {
        ensure!(
            !self.queue.closed,
            "property animation after runtime shutdown"
        );
        ensure!(
            self.property_animations.running.len() < LIMIT,
            "property animation capacity reached"
        );
        let roots = self.native_roots.len();
        self.native_roots.push(owner);
        let result = (|| -> Result<()> {
            let duration = self.property_time(owner, "duration")?;
            let delay = self.property_time(owner, "delay")?;
            let deadline = self
                .uptime_ms()
                .checked_add(delay)
                .filter(|time| *time <= i64::MAX as u64)
                .context("animation deadline overflow")?;
            let mut properties = vec![];
            for property in 0..PROPERTIES.len() {
                if let Some(values) = self
                    .heap
                    .get_mut(owner)?
                    .fields
                    .remove(&format!("pending:{property}"))
                {
                    ensure!(values.len() == 2, "invalid pending property");
                    properties.push((
                        property,
                        f32::from_bits(values[0].int()? as u32),
                        f32::from_bits(values[1].int()? as u32),
                    ));
                }
            }
            self.property_animations
                .pending
                .retain(|pending| *pending != owner);
            // An explicit start also creates a real, empty progress animation, as on API 21.
            let animator = self.heap.instance(VALUE)?;
            self.native_roots.push(animator);
            let fields = &mut self.heap.get_mut(animator)?.fields;
            fields.insert(OWNER.into(), vec![owner]);
            fields.insert("duration".into(), wide(duration));
            fields.insert("delay".into(), wide(delay));
            let interpolator = self.property_word(owner, "interpolator")?;
            let retained = if interpolator == Word::ZERO {
                self.heap
                    .instance("Landroid/view/animation/LinearInterpolator;")?
            } else {
                interpolator
            };
            self.heap
                .get_mut(animator)?
                .fields
                .insert("interpolator".into(), vec![retained]);
            let target = self.property_word(owner, TARGET)?;
            self.property_view_call(
                target,
                "setHasTransientState",
                vec!["Z".into()],
                "V",
                vec![Word::from(1)],
            )?;
            self.property_animations.running.push(Animation {
                owner,
                animator,
                deadline,
                duration,
                interpolator,
                started: false,
                finishing: false,
                properties,
            });
            if delay == 0 {
                self.start_property_listener(animator)?;
                self.update_property_animation(animator)?;
            }
            Ok(())
        })();
        self.native_roots.truncate(roots);
        result
    }
    fn start_property_listener(&mut self, animator: Word) -> Result<()> {
        let Some(animation) = self
            .property_animations
            .running
            .iter_mut()
            .find(|a| a.animator == animator)
        else {
            return Ok(());
        };
        if animation.started {
            return Ok(());
        }
        animation.started = true;
        let owner = animation.owner;
        if let Err(error) = self.property_event(owner, animator, "onAnimationStart") {
            self.discard_property_animation(animator)?;
            return Err(error);
        }
        Ok(())
    }
    fn update_property_animation(&mut self, animator: Word) -> Result<bool> {
        let Some(animation) = self
            .property_animations
            .running
            .iter()
            .find(|a| a.animator == animator)
            .cloned()
        else {
            return Ok(false);
        };
        if self.uptime_ms() < animation.deadline {
            return Ok(false);
        }
        let roots = self.native_roots.len();
        self.native_roots
            .extend([animation.owner, animator, animation.interpolator]);
        let result = (|| -> Result<bool> {
            self.start_property_listener(animator)?;
            if !self
                .property_animations
                .running
                .iter()
                .any(|a| a.animator == animator)
            {
                return Ok(true);
            }
            let elapsed = self.uptime_ms().saturating_sub(animation.deadline);
            let fraction = if animation.duration == 0 {
                1.0
            } else {
                (elapsed as f64 / animation.duration as f64).min(1.0) as f32
            };
            let progress = if animation.interpolator == Word::ZERO {
                fraction
            } else {
                let result = self.invoke(
                    Method {
                        class: "Landroid/animation/TimeInterpolator;".into(),
                        name: "getInterpolation".into(),
                        parameters: vec!["F".into()],
                        returns: "F".into(),
                    },
                    vec![animation.interpolator, Word::Bits(fraction.to_bits())],
                    true,
                )?;
                f32::from_bits(
                    result
                        .first()
                        .context("missing property interpolation result")?
                        .int()? as u32,
                )
            };
            ensure!(
                progress.is_finite(),
                "invalid property interpolation result"
            );
            let Some(current) = self
                .property_animations
                .running
                .iter()
                .find(|a| a.animator == animator)
                .cloned()
            else {
                return Ok(true);
            };
            self.heap
                .get_mut(animator)?
                .fields
                .insert("fraction".into(), vec![Word::Bits(progress.to_bits())]);
            let target = self.property_word(current.owner, TARGET)?;
            if self.property_word(current.owner, SCALAR)?.truth() {
                self.scalar_animation_value(animator, progress)?;
            }
            for (property, from, to) in current.properties {
                let value = (f64::from(from)
                    + (f64::from(to) - f64::from(from)) * f64::from(progress))
                    as f32;
                let setter = format!(
                    "set{}{}",
                    PROPERTIES[property][..1].to_ascii_uppercase(),
                    &PROPERTIES[property][1..]
                );
                self.property_view_call(
                    target,
                    &setter,
                    vec!["F".into()],
                    "V",
                    vec![Word::Bits(value.to_bits())],
                )?;
            }
            self.property_event(current.owner, animator, "onAnimationUpdate")?;
            if fraction == 1.0 {
                self.finish_property_animation(animator, false)?;
            }
            Ok(true)
        })();
        let result = result.map_err(|error| match self.discard_property_animation(animator) {
            Ok(_) => error,
            Err(cleanup) => error.context(format!(
                "property animation cleanup also failed: {cleanup:#}"
            )),
        });
        self.native_roots.truncate(roots);
        result
    }
    fn discard_property_animation(&mut self, animator: Word) -> Result<Option<Animation>> {
        let Some(index) = self
            .property_animations
            .running
            .iter()
            .position(|a| a.animator == animator)
        else {
            return Ok(None);
        };
        let animation = self.property_animations.running.remove(index);
        let target = self.property_word(animation.owner, TARGET)?;
        if self.property_word(animation.owner, SCALAR)?.truth() {
            return Ok(Some(animation));
        }
        self.property_view_call(
            target,
            "setHasTransientState",
            vec!["Z".into()],
            "V",
            vec![Word::ZERO],
        )?;
        Ok(Some(animation))
    }
    fn finish_property_animation(&mut self, animator: Word, cancelled: bool) -> Result<()> {
        let Some(animation) = self
            .property_animations
            .running
            .iter()
            .find(|a| a.animator == animator)
            .cloned()
        else {
            return Ok(());
        };
        if animation.finishing {
            return Ok(());
        }
        let roots = self.native_roots.len();
        self.native_roots
            .extend([animation.owner, animator, animation.interpolator]);
        let result = (|| -> Result<()> {
            if cancelled {
                self.start_property_listener(animator)?;
                // Start listeners can cancel this animation reentrantly.
                if !self
                    .property_animations
                    .running
                    .iter()
                    .any(|a| a.animator == animator)
                {
                    return Ok(());
                }
                if let Some(current) = self
                    .property_animations
                    .running
                    .iter_mut()
                    .find(|a| a.animator == animator)
                {
                    current.finishing = true;
                }
                self.property_event(animation.owner, animator, "onAnimationCancel")?;
            }
            if self.discard_property_animation(animator)?.is_some() {
                self.property_event(animation.owner, animator, "onAnimationEnd")?;
            }
            Ok(())
        })();
        let result = result.map_err(|error| match self.discard_property_animation(animator) {
            Ok(_) => error,
            Err(cleanup) => error.context(format!(
                "property animation cleanup also failed: {cleanup:#}"
            )),
        });
        self.native_roots.truncate(roots);
        result
    }
    pub(crate) fn poll_property_animations(&mut self) -> Result<usize> {
        self.require_main_thread()?;
        let pending = self.property_animations.pending.clone();
        for owner in pending {
            if self.property_animations.pending.contains(&owner) {
                self.start_property_animation(owner)?;
            }
        }
        // ponytail: bounded vectors; index by animator handle if many concurrent Views make scans costly.
        let running: Vec<_> = self
            .property_animations
            .running
            .iter()
            .map(|a| a.animator)
            .collect();
        let mut frames = 0;
        for animator in running {
            frames += usize::from(self.update_property_animation(animator)?);
        }
        Ok(frames)
    }
    pub(crate) fn stop_property_animations(&mut self) -> Result<()> {
        while let Some(animation) = self.property_animations.running.last() {
            self.discard_property_animation(animation.animator)?;
        }
        for owner in std::mem::take(&mut self.property_animations.pending) {
            for property in 0..PROPERTIES.len() {
                self.heap
                    .get_mut(owner)?
                    .fields
                    .remove(&format!("pending:{property}"));
            }
        }
        Ok(())
    }

    fn scalar_animation_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let animator = args[0];
        let arg = |index| {
            args.get(index)
                .copied()
                .context("missing ValueAnimator argument")
        };
        let result = match method.signature().as_str() {
            "<init>()V" => {
                let interpolator = self
                    .heap
                    .instance("Landroid/view/animation/AccelerateDecelerateInterpolator;")?;
                let fields = &mut self.heap.get_mut(animator)?.fields;
                fields.insert(SCALAR.into(), vec![Word::from(1)]);
                fields.insert("duration".into(), wide(300));
                fields.insert("delay".into(), wide(0));
                fields.insert("interpolator".into(), vec![interpolator]);
                vec![]
            }
            "setFloatValues([F)V" | "setIntValues([I)V" => {
                let array = arg(1)?;
                if array != Word::ZERO {
                    let Data::Array { element, values } = &self.heap.get(array)?.data else {
                        anyhow::bail!("animation values require array");
                    };
                    ensure!(
                        element
                            == if method.name == "setIntValues" {
                                "I"
                            } else {
                                "F"
                            },
                        "incorrect animation array type"
                    );
                    let values = values
                        .iter()
                        .map(|value| {
                            ensure!(value.len() == 1, "scalar keyframe requires one word");
                            Ok(value[0])
                        })
                        .collect::<Result<Vec<_>>>()?;
                    ensure!(values.len() <= 4096, "animation keyframe limit");
                    if !values.is_empty() {
                        let fields = &mut self.heap.get_mut(animator)?.fields;
                        fields.insert(
                            "integer".into(),
                            vec![Word::from(i32::from(method.name == "setIntValues"))],
                        );
                        fields.insert(
                            "values".into(),
                            if values.len() == 1 {
                                vec![Word::ZERO, values[0]]
                            } else {
                                values
                            },
                        );
                    }
                }
                vec![]
            }
            "setDuration(J)Landroid/animation/ValueAnimator;" | "setStartDelay(J)V" => {
                let value = bits64(args.get(1..3).context("missing animation time")?)?;
                if value > i64::MAX as u64 {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "animation time cannot be negative",
                    ));
                }
                self.heap.get_mut(animator)?.fields.insert(
                    if method.name == "setDuration" {
                        "duration"
                    } else {
                        "delay"
                    }
                    .into(),
                    wide(value),
                );
                if method.name == "setDuration" {
                    vec![animator]
                } else {
                    vec![]
                }
            }
            "setInterpolator(Landroid/animation/TimeInterpolator;)V" => {
                let value = arg(1)?;
                if value != Word::ZERO {
                    ensure!(
                        self.is_a(
                            &self.heap.get(value)?.class,
                            "Landroid/animation/TimeInterpolator;"
                        ),
                        "invalid animation interpolator"
                    );
                }
                self.heap
                    .get_mut(animator)?
                    .fields
                    .insert("interpolator".into(), vec![value]);
                vec![]
            }
            "addUpdateListener(Landroid/animation/ValueAnimator$AnimatorUpdateListener;)V"
            | "addListener(Landroid/animation/Animator$AnimatorListener;)V" => {
                let listener = arg(1)?;
                if listener != Word::ZERO {
                    ensure!(
                        self.is_a(&self.heap.get(listener)?.class, &method.parameters[0]),
                        "invalid animation listener"
                    );
                }
                let list = self
                    .heap
                    .get_mut(animator)?
                    .fields
                    .entry(
                        if method.name == "addListener" {
                            "listeners"
                        } else {
                            "updates"
                        }
                        .into(),
                    )
                    .or_default();
                ensure!(list.len() < 1024, "animation listener limit");
                list.push(listener);
                vec![]
            }
            "getDuration()J" => wide(self.property_time(animator, "duration")?),
            "getStartDelay()J" => wide(self.property_time(animator, "delay")?),
            "getInterpolator()Landroid/animation/TimeInterpolator;" => {
                vec![self.property_word(animator, "interpolator")?]
            }
            "getAnimatedFraction()F" => vec![self.property_word(animator, "fraction")?],
            "getAnimatedValue()Ljava/lang/Object;" => vec![self.property_word(animator, "value")?],
            "isStarted()Z" => vec![Word::from(i32::from(
                self.property_animations
                    .running
                    .iter()
                    .any(|a| a.animator == animator),
            ))],
            "isRunning()Z" => vec![Word::from(i32::from(
                self.property_animations
                    .running
                    .iter()
                    .any(|a| a.animator == animator && a.started && self.uptime_ms() >= a.deadline),
            ))],
            "start()V" => {
                self.start_scalar_animation(animator)?;
                vec![]
            }
            "cancel()V" => {
                self.finish_property_animation(animator, true)?;
                vec![]
            }
            "end()V" => {
                if !self
                    .property_animations
                    .running
                    .iter()
                    .any(|a| a.animator == animator)
                {
                    self.start_scalar_animation(animator)?;
                }
                if let Some(a) = self
                    .property_animations
                    .running
                    .iter_mut()
                    .find(|a| a.animator == animator)
                {
                    a.deadline = 0;
                    a.duration = 0;
                }
                self.update_property_animation(animator)?;
                vec![]
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }

    fn start_scalar_animation(&mut self, animator: Word) -> Result<()> {
        ensure!(!self.queue.closed, "animation after runtime shutdown");
        ensure!(
            self.property_animations.running.len() < LIMIT,
            "animation capacity reached"
        );
        self.discard_property_animation(animator)?;
        let duration = self.property_time(animator, "duration")?;
        let delay = self.property_time(animator, "delay")?;
        let deadline = self
            .uptime_ms()
            .checked_add(delay)
            .filter(|time| *time <= i64::MAX as u64)
            .context("animation deadline overflow")?;
        let interpolator = self.property_word(animator, "interpolator")?;
        self.property_animations.running.push(Animation {
            owner: animator,
            animator,
            deadline,
            duration,
            interpolator,
            started: false,
            finishing: false,
            properties: vec![],
        });
        if delay == 0 {
            // API 21 delivers the initial value update before the start listener.
            let result = (|| -> Result<()> {
                self.scalar_animation_value(animator, 0.0)?;
                self.property_event(animator, animator, "onAnimationUpdate")?;
                self.start_property_listener(animator)
            })();
            if let Err(error) = result {
                self.discard_property_animation(animator)?;
                return Err(error);
            }
        }
        Ok(())
    }

    fn scalar_animation_value(&mut self, animator: Word, progress: f32) -> Result<()> {
        // ponytail: scalar float/int keyframes reuse the existing clock; object evaluators and repeats remain explicit gaps.
        let values = self
            .heap
            .get(animator)?
            .fields
            .get("values")
            .context("ValueAnimator has no keyframes")?;
        ensure!(
            values.len() >= 2 && progress.is_finite(),
            "invalid animation values"
        );
        let integer = self.property_word(animator, "integer")?.truth();
        let position = progress * (values.len() - 1) as f32;
        let index = (position.floor().max(0.0) as usize).min(values.len() - 2);
        let fraction = position - index as f32;
        let number = |word: Word| -> Result<f64> {
            Ok(if integer {
                f64::from(word.int()?)
            } else {
                f64::from(f32::from_bits(word.int()? as u32))
            })
        };
        let from = number(values[index])?;
        let to = number(values[index + 1])?;
        let value = if integer {
            Word::from((from + (to - from) * f64::from(fraction)) as i32)
        } else {
            Word::Bits(((from + (to - from) * f64::from(fraction)) as f32).to_bits())
        };
        let boxed = self.heap.instance(if integer {
            "Ljava/lang/Integer;"
        } else {
            "Ljava/lang/Float;"
        })?;
        self.heap
            .get_mut(boxed)?
            .fields
            .insert("value".into(), vec![value]);
        let fields = &mut self.heap.get_mut(animator)?.fields;
        fields.insert("fraction".into(), vec![Word::Bits(progress.to_bits())]);
        fields.insert("value".into(), vec![boxed]);
        Ok(())
    }
}
