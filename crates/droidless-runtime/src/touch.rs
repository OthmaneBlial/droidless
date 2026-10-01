//! Single-pointer input and API-21 gesture callbacks, using the existing main queue.
use crate::{
    heap::{Data, Word, bits64, wide},
    ui::Node,
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;

const MOTION: &str = "Landroid/view/MotionEvent;";
const VELOCITY: &str = "Landroid/view/VelocityTracker;";
const DETECTOR: &str = "Landroid/view/GestureDetector;";
const SIMPLE: &str = "Landroid/view/GestureDetector$SimpleOnGestureListener;";
const LISTENER: &str = "Landroid/view/GestureDetector$OnGestureListener;";
const DOUBLE: &str = "Landroid/view/GestureDetector$OnDoubleTapListener;";
const HANDLER: &str = "Landroid/os/Handler;";
const TIMER: &str = "Ldroidless/runtime/GestureTimer;";
const VIEW: &str = "Landroid/view/View;";
const ACTIVITY: &str = "Landroid/app/Activity;";

#[derive(Clone, Copy, Debug)]
pub struct Motion {
    down: u64,
    time: u64,
    action: i32,
    x: f32,
    y: f32,
    raw_x: f32,
    raw_y: f32,
    meta: i32,
    recycled: bool,
}
impl Motion {
    fn validate(&self) -> Result<()> {
        ensure!(!self.recycled, "recycled MotionEvent");
        ensure!(
            (0..=3).contains(&self.action),
            "only single-pointer DOWN/UP/MOVE/CANCEL supported"
        );
        ensure!(
            self.down <= self.time && self.time <= i64::MAX as u64,
            "invalid MotionEvent times"
        );
        ensure!(
            [self.x, self.y, self.raw_x, self.raw_y]
                .iter()
                .all(|x| x.is_finite() && x.abs() <= 1_000_000.0),
            "invalid MotionEvent coordinates"
        );
        Ok(())
    }
}

#[derive(Clone, Debug, Default)]
pub struct Velocity {
    samples: Vec<Motion>,
    value: [f32; 2],
    recycled: bool,
}

fn add_sample(samples: &mut Vec<Motion>, motion: Motion) {
    samples.retain(|sample| motion.time.saturating_sub(sample.time) <= 100);
    if samples
        .last()
        .is_some_and(|sample| sample.time == motion.time)
    {
        samples.pop();
    }
    if samples.len() == 20 {
        samples.remove(0);
    }
    samples.push(motion);
}
#[derive(Clone, Debug)]
pub(crate) struct TouchStream {
    pub owner: Word,
    pub root: Word,
    down: u64,
    last: u64,
}

#[derive(Clone, Debug)]
pub struct Gesture {
    listener: Word,
    double: Word,
    handler: Word,
    timers: [Word; 3],
    down: Word,
    up: Word,
    still_down: bool,
    tap: bool,
    bigger_tap: bool,
    double_tap: bool,
    long_press: bool,
    defer: bool,
    long_enabled: bool,
    busy: bool,
    focus: [f32; 2],
    samples: Vec<Motion>,
}
impl Gesture {
    pub(crate) fn roots(&self) -> impl Iterator<Item = Word> {
        [self.listener, self.double, self.handler, self.down, self.up]
            .into_iter()
            .chain(self.timers)
    }
}
fn method(class: &str, name: &str, parameters: &[&str], returns: &str) -> Method {
    Method {
        class: class.into(),
        name: name.into(),
        parameters: parameters.iter().map(|s| (*s).into()).collect(),
        returns: returns.into(),
    }
}
fn field(vm: &Runtime, object: Word, key: &str) -> Result<Word> {
    Ok(vm
        .heap
        .get(object)?
        .fields
        .get(key)
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(Word::ZERO))
}
fn find(node: &Node, handle: usize) -> Option<&Node> {
    if node.handle == handle {
        Some(node)
    } else {
        node.children.iter().find_map(|n| find(n, handle))
    }
}

impl Runtime {
    fn motion(&self, word: Word) -> Result<Motion> {
        let Data::Motion(m) = self.heap.get(word)?.data else {
            bail!("expected initialized MotionEvent");
        };
        m.validate()?;
        Ok(m)
    }
    fn new_motion(&mut self, m: Motion) -> Result<Word> {
        m.validate()?;
        let event = self.heap.instance(MOTION)?;
        self.heap.get_mut(event)?.data = Data::Motion(m);
        Ok(event)
    }
    fn gesture(&self, word: Word) -> Result<Gesture> {
        let Data::Gesture(g) = &self.heap.get(word)?.data else {
            bail!("expected initialized GestureDetector");
        };
        Ok(g.clone())
    }
    fn save_gesture(&mut self, word: Word, g: &Gesture) -> Result<()> {
        self.heap.get_mut(word)?.data = Data::Gesture(g.clone());
        Ok(())
    }
    fn gesture_callback(
        &mut self,
        detector: Word,
        g: &mut Gesture,
        double: bool,
        name: &str,
        words: &[Word],
        floats: &[f32],
    ) -> Result<bool> {
        let returns = if matches!(name, "onShowPress" | "onLongPress") {
            "V"
        } else {
            "Z"
        };
        self.save_gesture(detector, g)?;
        let listener = if double { g.double } else { g.listener };
        if listener == Word::ZERO {
            return Ok(false);
        }
        let mut args = vec![listener];
        args.extend_from_slice(words);
        args.extend(floats.iter().map(|f| Word::Bits(f.to_bits())));
        let mut params = vec![MOTION; words.len()];
        params.extend(vec!["F"; floats.len()]);
        let result = self.invoke(
            method(
                if double { DOUBLE } else { LISTENER },
                name,
                &params,
                returns,
            ),
            args,
            true,
        );
        *g = self.gesture(detector)?;
        Ok(result?.first().is_some_and(|w| w.truth()))
    }
    fn gesture_timer(&mut self, g: &Gesture, kind: usize, when: Option<u64>) -> Result<bool> {
        let (name, params, returns) = if when.is_some() {
            ("postAtTime", vec!["Ljava/lang/Runnable;", "J"], "Z")
        } else {
            ("removeCallbacks", vec!["Ljava/lang/Runnable;"], "V")
        };
        let mut args = vec![g.handler, g.timers[kind]];
        if let Some(time) = when {
            args.extend(wide(time));
        }
        Ok(self
            .invoke(method(HANDLER, name, &params, returns), args, false)?
            .first()
            .is_some_and(|w| w.truth()))
    }
    fn cancel_gesture(&mut self, g: &mut Gesture) -> Result<()> {
        for kind in 0..3 {
            self.gesture_timer(g, kind, None)?;
        }
        g.still_down = false;
        g.tap = false;
        g.bigger_tap = false;
        g.double_tap = false;
        g.long_press = false;
        g.defer = false;
        g.samples.clear();
        Ok(())
    }
    fn detector_event(&mut self, detector: Word, event: Word) -> Result<bool> {
        let m = self.motion(event)?;
        let mut g = self.gesture(detector)?;
        ensure!(!g.busy, "reentrant GestureDetector input");
        if m.action != 0 && m.action != 3 {
            ensure!(g.still_down, "GestureDetector event without DOWN");
            let down = self.motion(g.down)?;
            ensure!(
                m.down == down.down
                    && m.time >= down.time
                    && g.samples.last().is_none_or(|last| m.time >= last.time),
                "out-of-order GestureDetector input"
            );
        }
        let roots = self.native_roots.len();
        self.native_roots.extend([detector, event]);
        g.busy = true;
        let result = (|| -> Result<bool> {
            if m.action == 0 {
                let pending = self.invoke(
                    method(HANDLER, "hasCallbacks", &["Ljava/lang/Runnable;"], "Z"),
                    vec![g.handler, g.timers[2]],
                    false,
                )?[0]
                    .truth();
                self.gesture_timer(&g, 2, None)?;
                let double = if g.down != Word::ZERO
                    && g.up != Word::ZERO
                    && pending
                    && g.bigger_tap
                    && g.double != Word::ZERO
                {
                    let down = self.motion(g.down)?;
                    let up = self.motion(g.up)?;
                    let dt = m.time.saturating_sub(up.time);
                    (40..=300).contains(&dt)
                        && (down.x - m.x).powi(2) + (down.y - m.y).powi(2) < 10000.0
                } else {
                    false
                };
                if g.still_down {
                    self.cancel_gesture(&mut g)?;
                }
                g.double_tap = double;
                let mut handled = false;
                if double {
                    let old = g.down;
                    handled |=
                        self.gesture_callback(detector, &mut g, true, "onDoubleTap", &[old], &[])?;
                    handled |= self.gesture_callback(
                        detector,
                        &mut g,
                        true,
                        "onDoubleTapEvent",
                        &[event],
                        &[],
                    )?;
                } else if g.double != Word::ZERO {
                    self.gesture_timer(&g, 2, Some(self.uptime_ms().saturating_add(300)))?;
                }
                g.down = self.new_motion(m)?;
                g.focus = [m.x, m.y];
                g.samples.clear();
                g.still_down = true;
                g.tap = true;
                g.bigger_tap = true;
                g.long_press = false;
                g.defer = false;
                if g.long_enabled {
                    self.gesture_timer(&g, 1, Some(m.down.saturating_add(600)))?;
                }
                self.gesture_timer(&g, 0, Some(m.down.saturating_add(100)))?;
                handled |=
                    self.gesture_callback(detector, &mut g, false, "onDown", &[event], &[])?;
                g.samples.push(m);
                return Ok(handled);
            }
            if m.action == 3 {
                self.cancel_gesture(&mut g)?;
                return Ok(false);
            }
            add_sample(&mut g.samples, m);
            let down = g.down;
            if m.action == 2 {
                if g.long_press {
                    return Ok(false);
                }
                if g.double_tap {
                    return self.gesture_callback(
                        detector,
                        &mut g,
                        true,
                        "onDoubleTapEvent",
                        &[event],
                        &[],
                    );
                }
                let start = self.motion(down)?;
                let distance = (m.x - start.x).powi(2) + (m.y - start.y).powi(2);
                let delta = [g.focus[0] - m.x, g.focus[1] - m.y];
                if g.tap && distance > 64.0 {
                    g.tap = false;
                    g.bigger_tap = false;
                    g.focus = [m.x, m.y];
                    for kind in 0..3 {
                        self.gesture_timer(&g, kind, None)?;
                    }
                    return self.gesture_callback(
                        detector,
                        &mut g,
                        false,
                        "onScroll",
                        &[down, event],
                        &delta,
                    );
                }
                if !g.tap && (delta[0].abs() >= 1.0 || delta[1].abs() >= 1.0) {
                    g.focus = [m.x, m.y];
                    return self.gesture_callback(
                        detector,
                        &mut g,
                        false,
                        "onScroll",
                        &[down, event],
                        &delta,
                    );
                }
                return Ok(false);
            }
            g.still_down = false;
            let up = self.new_motion(m)?;
            self.native_roots.push(up);
            let handled = if g.double_tap {
                self.gesture_callback(detector, &mut g, true, "onDoubleTapEvent", &[event], &[])?
            } else if g.long_press {
                self.gesture_timer(&g, 2, None)?;
                g.long_press = false;
                false
            } else if g.tap {
                let handled =
                    self.gesture_callback(detector, &mut g, false, "onSingleTapUp", &[event], &[])?;
                if g.defer {
                    self.gesture_callback(
                        detector,
                        &mut g,
                        true,
                        "onSingleTapConfirmed",
                        &[event],
                        &[],
                    )?;
                }
                handled
            } else {
                let velocity = velocity(&g.samples, 1000, 8000.0);
                if velocity.iter().any(|v| v.abs() > 50.0) {
                    self.gesture_callback(
                        detector,
                        &mut g,
                        false,
                        "onFling",
                        &[down, event],
                        &velocity,
                    )?
                } else {
                    false
                }
            };
            g.up = up;
            g.double_tap = false;
            g.defer = false;
            g.samples.clear();
            self.gesture_timer(&g, 0, None)?;
            self.gesture_timer(&g, 1, None)?;
            Ok(handled)
        })();
        if result.is_err() {
            let _ = self.cancel_gesture(&mut g);
        }
        g.busy = false;
        let saved = self.save_gesture(detector, &g);
        self.native_roots.truncate(roots);
        saved?;
        result
    }
    /// Native hosts send logical window coordinates and process-relative monotonic milliseconds.
    pub fn touch_at(&mut self, action: i32, x: f32, y: f32, time: u64) -> Result<bool> {
        ensure!(
            self.frames.is_empty() && self.queue.active.is_none(),
            "touch input during guest execution"
        );
        ensure!((0..=3).contains(&action), "unsupported touch action");
        if action == 0 {
            Motion {
                down: time,
                time,
                action,
                x,
                y,
                raw_x: x,
                raw_y: y,
                meta: 0,
                recycled: false,
            }
            .validate()?;
            ensure!(
                !self
                    .touch
                    .as_ref()
                    .is_some_and(|stream| Some(stream.owner) == self.activity
                        && Some(stream.root) == self.root),
                "touch DOWN during active stream"
            );
            self.layout_snapshot()?;
        }
        let owner = self.activity.context("touch without Activity")?;
        let root = self.root.context("touch without content View")?;
        if self
            .touch
            .as_ref()
            .is_some_and(|s| s.owner != owner || s.root != root)
        {
            self.touch = None;
        }
        if action == 0 {
            ensure!(self.touch.is_none(), "touch DOWN during active stream");
            self.touch = Some(TouchStream {
                owner,
                root,
                down: time,
                last: time,
            });
        }
        let stream = self
            .touch
            .as_ref()
            .context("touch event without DOWN")?
            .clone();
        ensure!(
            stream.owner == owner && stream.root == root,
            "touch target changed"
        );
        let m = Motion {
            down: stream.down,
            time,
            action,
            x,
            y,
            raw_x: x,
            raw_y: y,
            meta: 0,
            recycled: false,
        };
        let result = (|| {
            ensure!(time >= stream.last, "out-of-order touch event");
            let event = self.new_motion(m)?;
            self.reset_budget();
            let roots = self.native_roots.len();
            self.native_roots.push(event);
            let result = self.invoke(
                method(ACTIVITY, "dispatchTouchEvent", &[MOTION], "Z"),
                vec![owner, event],
                true,
            );
            self.native_roots.truncate(roots);
            result
        })();
        if action == 1 || action == 3 || result.is_err() {
            self.touch = None;
        } else if let Some(stream) = &mut self.touch {
            stream.last = time;
        }
        let handled = result?.first().is_some_and(|w| w.truth());
        self.drain_navigation()?;
        if self.activity != Some(owner) || self.root != Some(root) {
            self.touch = None;
        }
        self.collect();
        Ok(handled)
    }
    pub fn touch(&mut self, action: i32, x: f32, y: f32) -> Result<bool> {
        self.touch_at(action, x, y, self.uptime_ms())
    }
    pub fn touch_active(&self) -> bool {
        self.touch
            .as_ref()
            .is_some_and(|s| self.activity == Some(s.owner) && self.root == Some(s.root))
    }
    pub fn touch_input_enabled(&self) -> bool {
        fn guest(vm: &Runtime, mut class: String) -> bool {
            for _ in 0..128 {
                let Some((d, c)) = vm.class_location(&class) else {
                    return false;
                };
                let definition = &vm.apk.dex[d].classes[c];
                if definition.methods.iter().any(|m| {
                    matches!(
                        vm.apk.dex[d].methods[m.index].signature().as_str(),
                        "onTouchEvent(Landroid/view/MotionEvent;)Z"
                            | "dispatchTouchEvent(Landroid/view/MotionEvent;)Z"
                    )
                }) {
                    return true;
                }
                let Some(parent) = &definition.super_class else {
                    return false;
                };
                class = parent.clone();
            }
            false
        }
        fn views(vm: &Runtime, word: Word, depth: usize) -> bool {
            if depth >= 128 {
                return false;
            }
            let Ok(object) = vm.heap.get(word) else {
                return false;
            };
            if field(vm, word, "droidless:view:touch-listener").is_ok_and(|w| w != Word::ZERO)
                || guest(vm, object.class.clone())
            {
                return true;
            }
            object
                .view
                .as_ref()
                .is_some_and(|v| v.children.iter().any(|w| views(vm, *w, depth + 1)))
        }
        self.activity
            .is_some_and(|a| self.heap.get(a).is_ok_and(|o| guest(self, o.class.clone())))
            || self.root.is_some_and(|r| views(self, r, 0))
    }
    fn perform_view_click(&mut self, view: Word) -> Result<bool> {
        let data = self
            .heap
            .get(view)?
            .view
            .as_ref()
            .context("performClick expects View")?
            .clone();
        let roots = self.native_roots.len();
        self.native_roots.push(view);
        let result = (|| {
            if let Some(listener) = data.listener {
                self.native_roots.push(listener);
                self.invoke(
                    method(
                        "Landroid/view/View$OnClickListener;",
                        "onClick",
                        &[VIEW],
                        "V",
                    ),
                    vec![listener, view],
                    true,
                )?;
                Ok(true)
            } else if let Some(name) = data.xml_click {
                let activity = self.activity.context("XML click without Activity")?;
                let class = self.heap.get(activity)?.class.clone();
                self.invoke(
                    method(&class, &name, &[VIEW], "V"),
                    vec![activity, view],
                    true,
                )?;
                Ok(true)
            } else {
                Ok(false)
            }
        })();
        self.native_roots.truncate(roots);
        result
    }
    fn dispatch_view_touch(&mut self, view: Word, event: Word) -> Result<bool> {
        let m = self.motion(event)?;
        let data = self
            .heap
            .get(view)?
            .view
            .as_ref()
            .context("touch expects View")?
            .clone();
        if data.visible != 0 {
            return Ok(false);
        }
        if !data.children.is_empty() {
            let tree = self.snapshot()?;
            let node = find(&tree, view.reference()?).context("touch ViewGroup not attached")?;
            let captured = field(self, view, "droidless:touch:child")?;
            let candidates = if m.action == 0 {
                node.children
                    .iter()
                    .rev()
                    .filter(|child| {
                        child.view.visible == 0
                            && m.x >= child.rect.x - node.rect.x
                            && m.x < child.rect.x - node.rect.x + child.rect.width
                            && m.y >= child.rect.y - node.rect.y
                            && m.y < child.rect.y - node.rect.y + child.rect.height
                    })
                    .map(|child| {
                        (
                            Word::Ref(child.handle),
                            child.rect.x - node.rect.x,
                            child.rect.y - node.rect.y,
                        )
                    })
                    .collect::<Vec<_>>()
            } else {
                node.children
                    .iter()
                    .filter(|child| Word::Ref(child.handle) == captured)
                    .map(|child| {
                        (
                            captured,
                            child.rect.x - node.rect.x,
                            child.rect.y - node.rect.y,
                        )
                    })
                    .collect()
            };
            // Keep the snapshot alive if an earlier child mutates the tree and collects.
            self.native_roots
                .extend(candidates.iter().map(|(child, _, _)| *child));
            for (child, x, y) in candidates {
                let translated = self.new_motion(Motion {
                    x: m.x - x,
                    y: m.y - y,
                    ..m
                })?;
                let roots = self.native_roots.len();
                self.native_roots.extend([child, translated]);
                let handled = self.invoke(
                    method(VIEW, "dispatchTouchEvent", &[MOTION], "Z"),
                    vec![child, translated],
                    true,
                );
                self.native_roots.truncate(roots);
                if handled?.first().is_some_and(|w| w.truth())
                    || (m.action != 0 && captured != Word::ZERO)
                {
                    if m.action == 0 {
                        self.heap
                            .get_mut(view)?
                            .fields
                            .insert("droidless:touch:child".into(), vec![child]);
                    }
                    if m.action == 1 || m.action == 3 {
                        self.heap
                            .get_mut(view)?
                            .fields
                            .remove("droidless:touch:child");
                    }
                    return Ok(true);
                }
            }
            if m.action == 1 || m.action == 3 {
                self.heap
                    .get_mut(view)?
                    .fields
                    .remove("droidless:touch:child");
            }
        }
        let listener = field(self, view, "droidless:view:touch-listener")?;
        if data.enabled
            && listener != Word::ZERO
            && self
                .invoke(
                    method(
                        "Landroid/view/View$OnTouchListener;",
                        "onTouch",
                        &[VIEW, MOTION],
                        "Z",
                    ),
                    vec![listener, view, event],
                    true,
                )?
                .first()
                .is_some_and(|w| w.truth())
        {
            return Ok(true);
        }
        Ok(self
            .invoke(
                method(VIEW, "onTouchEvent", &[MOTION], "Z"),
                vec![view, event],
                true,
            )?
            .first()
            .is_some_and(|w| w.truth()))
    }
    pub(crate) fn touch_native(
        &mut self,
        method_: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method_.signature();
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let arg = |i: usize| args.get(i).copied().context("missing touch argument");
        let mut result = vec![];
        if method_.class == VELOCITY {
            if signature == "obtain()Landroid/view/VelocityTracker;" {
                ensure!(args.is_empty(), "invalid VelocityTracker.obtain arguments");
                let tracker = self.heap.instance(VELOCITY)?;
                self.heap.get_mut(tracker)?.data = Data::Velocity(Velocity::default());
                return Ok(Some(vec![tracker]));
            }
            ensure!(
                args.len() == method_.parameters.len() + 1,
                "invalid VelocityTracker arguments"
            );
            let movement = if signature == "addMovement(Landroid/view/MotionEvent;)V" {
                Some(self.motion(arg(1)?)?)
            } else {
                None
            };
            let Data::Velocity(tracker) = &mut self.heap.get_mut(receiver)?.data else {
                bail!("uninitialized VelocityTracker");
            };
            ensure!(!tracker.recycled, "recycled VelocityTracker");
            match signature.as_str() {
                "addMovement(Landroid/view/MotionEvent;)V" => {
                    let movement = movement.context("missing velocity movement")?;
                    if movement.action == 0 {
                        tracker.samples.clear();
                    }
                    ensure!(
                        tracker
                            .samples
                            .last()
                            .is_none_or(|last| movement.time >= last.time),
                        "out-of-order VelocityTracker movement"
                    );
                    add_sample(&mut tracker.samples, movement);
                }
                "computeCurrentVelocity(I)V" | "computeCurrentVelocity(IF)V" => {
                    let units = arg(1)?.int()?;
                    let maximum = if args.len() == 3 {
                        f32::from_bits(arg(2)?.int()? as u32)
                    } else {
                        f32::MAX
                    };
                    ensure!(
                        units > 0 && maximum.is_finite() && maximum >= 0.0,
                        "invalid velocity units or maximum"
                    );
                    tracker.value = velocity(&tracker.samples, units, maximum);
                }
                "getXVelocity()F" | "getYVelocity()F" | "getXVelocity(I)F" | "getYVelocity(I)F" => {
                    let pointer = if args.len() == 2 { arg(1)?.int()? } else { -1 };
                    let value = if matches!(pointer, -1 | 0) {
                        tracker.value[usize::from(method_.name == "getYVelocity")]
                    } else {
                        0.0
                    };
                    result.push(Word::Bits(value.to_bits()));
                }
                "clear()V" | "recycle()V" => {
                    tracker.samples.clear();
                    tracker.value = [0.0; 2];
                    tracker.recycled = signature == "recycle()V";
                }
                _ => return Ok(None),
            }
            return Ok(Some(result));
        }
        match (method_.class.as_str(), signature.as_str()) {
            (MOTION, "obtain(JJIFFI)Landroid/view/MotionEvent;") => {
                let x = f32::from_bits(arg(5)?.int()? as u32);
                let y = f32::from_bits(arg(6)?.int()? as u32);
                result.push(self.new_motion(Motion {
                    down: bits64(&args[0..2])?,
                    time: bits64(&args[2..4])?,
                    action: arg(4)?.int()?,
                    x,
                    y,
                    raw_x: x,
                    raw_y: y,
                    meta: arg(7)?.int()?,
                    recycled: false,
                })?);
            }
            (MOTION, "obtain(Landroid/view/MotionEvent;)Landroid/view/MotionEvent;") => {
                let m = self.motion(receiver)?;
                result.push(self.new_motion(m)?);
            }
            (MOTION, "recycle()V") => {
                self.motion(receiver)?;
                if let Data::Motion(m) = &mut self.heap.get_mut(receiver)?.data {
                    m.recycled = true;
                }
            }
            (
                MOTION,
                "getAction()I"
                | "getActionMasked()I"
                | "getActionIndex()I"
                | "getPointerCount()I"
                | "getPointerId(I)I"
                | "findPointerIndex(I)I"
                | "getMetaState()I"
                | "getX()F"
                | "getY()F"
                | "getX(I)F"
                | "getY(I)F"
                | "getRawX()F"
                | "getRawY()F"
                | "getDownTime()J"
                | "getEventTime()J",
            ) => {
                let m = self.motion(receiver)?;
                if signature == "getPointerId(I)I"
                    || signature == "getX(I)F"
                    || signature == "getY(I)F"
                {
                    ensure!(
                        arg(1)?.int()? == 0,
                        "MotionEvent pointer index outside single-pointer profile"
                    );
                }
                result = match method_.name.as_str() {
                    "getDownTime" => wide(m.down),
                    "getEventTime" => wide(m.time),
                    "getX" => vec![Word::Bits(m.x.to_bits())],
                    "getY" => vec![Word::Bits(m.y.to_bits())],
                    "getRawX" => vec![Word::Bits(m.raw_x.to_bits())],
                    "getRawY" => vec![Word::Bits(m.raw_y.to_bits())],
                    "getAction" | "getActionMasked" => vec![Word::from(m.action)],
                    "getMetaState" => vec![Word::from(m.meta)],
                    "getPointerCount" => vec![Word::from(1)],
                    "findPointerIndex" => {
                        vec![Word::from(if arg(1)?.int()? == 0 { 0 } else { -1 })]
                    }
                    _ => vec![Word::ZERO],
                };
            }
            (MOTION, "offsetLocation(FF)V" | "setLocation(FF)V") => {
                let mut m = self.motion(receiver)?;
                let x = f32::from_bits(arg(1)?.int()? as u32);
                let y = f32::from_bits(arg(2)?.int()? as u32);
                if method_.name == "offsetLocation" {
                    m.x += x;
                    m.y += y;
                } else {
                    m.x = x;
                    m.y = y;
                }
                m.validate()?;
                self.heap.get_mut(receiver)?.data = Data::Motion(m);
            }
            (
                DETECTOR,
                "<init>(Landroid/content/Context;Landroid/view/GestureDetector$OnGestureListener;)V"
                | "<init>(Landroid/content/Context;Landroid/view/GestureDetector$OnGestureListener;Landroid/os/Handler;)V",
            ) => {
                self.heap.get(arg(1)?)?;
                let listener = arg(2)?;
                ensure!(
                    self.is_a(&self.heap.get(listener)?.class, LISTENER),
                    "expected OnGestureListener"
                );
                let handler = if args.len() == 4 && arg(3)? != Word::ZERO {
                    arg(3)?
                } else {
                    let h = self.heap.instance(HANDLER)?;
                    self.invoke(method(HANDLER, "<init>", &[], "V"), vec![h], false)?;
                    h
                };
                ensure!(
                    self.is_a(&self.heap.get(handler)?.class, HANDLER),
                    "expected Handler"
                );
                let double = if self.is_a(&self.heap.get(listener)?.class, DOUBLE) {
                    listener
                } else {
                    Word::ZERO
                };
                let mut timers = [Word::ZERO; 3];
                for (kind, timer) in timers.iter_mut().enumerate() {
                    *timer = self.heap.instance(TIMER)?;
                    let fields = &mut self.heap.get_mut(*timer)?.fields;
                    fields.insert("detector".into(), vec![receiver]);
                    fields.insert("kind".into(), vec![Word::from(kind as i32)]);
                }
                self.heap.get_mut(receiver)?.data = Data::Gesture(Gesture {
                    listener,
                    double,
                    handler,
                    timers,
                    down: Word::ZERO,
                    up: Word::ZERO,
                    still_down: false,
                    tap: false,
                    bigger_tap: false,
                    double_tap: false,
                    long_press: false,
                    defer: false,
                    long_enabled: true,
                    busy: false,
                    focus: [0.0; 2],
                    samples: vec![],
                });
            }
            (
                DETECTOR,
                "setOnDoubleTapListener(Landroid/view/GestureDetector$OnDoubleTapListener;)V",
            ) => {
                let listener = arg(1)?;
                ensure!(
                    listener == Word::ZERO || self.is_a(&self.heap.get(listener)?.class, DOUBLE),
                    "expected OnDoubleTapListener"
                );
                let mut g = self.gesture(receiver)?;
                g.double = listener;
                self.save_gesture(receiver, &g)?;
            }
            (DETECTOR, "setIsLongpressEnabled(Z)V" | "isLongpressEnabled()Z") => {
                let mut g = self.gesture(receiver)?;
                if method_.name == "setIsLongpressEnabled" {
                    g.long_enabled = arg(1)?.truth();
                    self.save_gesture(receiver, &g)?;
                } else {
                    result.push(Word::from(i32::from(g.long_enabled)));
                }
            }
            (DETECTOR, "onTouchEvent(Landroid/view/MotionEvent;)Z") => {
                result.push(Word::from(i32::from(
                    self.detector_event(receiver, arg(1)?)?,
                )));
            }
            (
                SIMPLE,
                "<init>()V"
                | "onDown(Landroid/view/MotionEvent;)Z"
                | "onSingleTapUp(Landroid/view/MotionEvent;)Z"
                | "onSingleTapConfirmed(Landroid/view/MotionEvent;)Z"
                | "onDoubleTap(Landroid/view/MotionEvent;)Z"
                | "onDoubleTapEvent(Landroid/view/MotionEvent;)Z"
                | "onScroll(Landroid/view/MotionEvent;Landroid/view/MotionEvent;FF)Z"
                | "onFling(Landroid/view/MotionEvent;Landroid/view/MotionEvent;FF)Z"
                | "onShowPress(Landroid/view/MotionEvent;)V"
                | "onLongPress(Landroid/view/MotionEvent;)V",
            ) => {
                self.heap.get(receiver)?;
                if method_.returns == "Z" {
                    result.push(Word::ZERO);
                }
            }
            (TIMER, "run()V") => {
                let detector = field(self, receiver, "detector")?;
                let kind = field(self, receiver, "kind")?.int()?;
                let mut g = self.gesture(detector)?;
                ensure!(!g.busy, "reentrant gesture timer");
                g.busy = true;
                let result = (|| {
                    let down = g.down;
                    if down == Word::ZERO {
                        return Ok(());
                    }
                    match kind {
                        0 if g.still_down => {
                            self.gesture_callback(
                                detector,
                                &mut g,
                                false,
                                "onShowPress",
                                &[down],
                                &[],
                            )?;
                        }
                        1 if g.still_down => {
                            self.gesture_timer(&g, 2, None)?;
                            g.defer = false;
                            g.long_press = true;
                            self.gesture_callback(
                                detector,
                                &mut g,
                                false,
                                "onLongPress",
                                &[down],
                                &[],
                            )?;
                        }
                        2 if g.double != Word::ZERO => {
                            if g.still_down {
                                g.defer = true;
                            } else {
                                self.gesture_callback(
                                    detector,
                                    &mut g,
                                    true,
                                    "onSingleTapConfirmed",
                                    &[down],
                                    &[],
                                )?;
                            }
                        }
                        0..=2 => {}
                        _ => bail!("invalid gesture timer"),
                    }
                    Ok(())
                })();
                if result.is_err() {
                    let _ = self.cancel_gesture(&mut g);
                }
                g.busy = false;
                self.save_gesture(detector, &g)?;
                result?;
            }
            (ACTIVITY, "dispatchTouchEvent(Landroid/view/MotionEvent;)Z") => {
                let root = self.root.context("Activity touch without content")?;
                let event = arg(1)?;
                let mut handled = self.invoke(
                    method(VIEW, "dispatchTouchEvent", &[MOTION], "Z"),
                    vec![root, event],
                    true,
                )?[0]
                    .truth();
                if !handled {
                    handled = self.invoke(
                        method(ACTIVITY, "onTouchEvent", &[MOTION], "Z"),
                        vec![receiver, event],
                        true,
                    )?[0]
                        .truth();
                }
                result.push(Word::from(i32::from(handled)));
            }
            (ACTIVITY, "onTouchEvent(Landroid/view/MotionEvent;)Z") => {
                self.motion(arg(1)?)?;
                result.push(Word::ZERO);
            }
            (
                VIEW | "Landroid/view/ViewGroup;",
                "dispatchTouchEvent(Landroid/view/MotionEvent;)Z",
            ) => {
                ensure!(self.touch_depth < 128, "touch dispatch depth limit reached");
                self.touch_depth += 1;
                let roots = self.native_roots.len();
                self.native_roots.extend_from_slice(args);
                let handled = self.dispatch_view_touch(receiver, arg(1)?);
                if handled.is_err() {
                    let fields = &mut self.heap.get_mut(receiver)?.fields;
                    fields.remove("droidless:touch:child");
                    fields.remove("droidless:touch:pressed");
                }
                self.native_roots.truncate(roots);
                self.touch_depth -= 1;
                result.push(Word::from(i32::from(handled?)));
            }
            (VIEW, "performClick()Z") => {
                result.push(Word::from(i32::from(self.perform_view_click(receiver)?)));
            }
            (
                VIEW,
                "setClickable(Z)V"
                | "isClickable()Z"
                | "setPressed(Z)V"
                | "isPressed()Z"
                | "setSystemUiVisibility(I)V"
                | "getSystemUiVisibility()I",
            ) => {
                let key = match method_.name.as_str() {
                    "setClickable" | "isClickable" => "droidless:touch:clickable",
                    "setPressed" | "isPressed" => "droidless:touch:pressed",
                    _ => "droidless:view:system-ui",
                };
                if method_.name.starts_with("set") {
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(key.into(), vec![arg(1)?]);
                } else if method_.name == "isClickable" {
                    result.push(Word::from(i32::from(self.view_clickable(receiver)?)));
                } else {
                    result.push(field(self, receiver, key)?);
                }
            }
            (VIEW, "onTouchEvent(Landroid/view/MotionEvent;)Z") => {
                let m = self.motion(arg(1)?)?;
                let clickable = self.view_clickable(receiver)?;
                let enabled = self
                    .heap
                    .get(receiver)?
                    .view
                    .as_ref()
                    .context("touch expects View")?
                    .enabled;
                if clickable && enabled {
                    match m.action {
                        0 => {
                            self.heap
                                .get_mut(receiver)?
                                .fields
                                .insert("droidless:touch:pressed".into(), vec![Word::from(1)]);
                        }
                        2 => {
                            if let Ok(tree) = self.snapshot()
                                && let Some(node) = find(&tree, receiver.reference()?)
                                && (m.x < -8.0
                                    || m.y < -8.0
                                    || m.x >= node.rect.width + 8.0
                                    || m.y >= node.rect.height + 8.0)
                            {
                                self.heap
                                    .get_mut(receiver)?
                                    .fields
                                    .remove("droidless:touch:pressed");
                            }
                        }
                        1 => {
                            let pressed = field(self, receiver, "droidless:touch:pressed")?.truth();
                            self.heap
                                .get_mut(receiver)?
                                .fields
                                .remove("droidless:touch:pressed");
                            if pressed {
                                self.invoke(
                                    method(VIEW, "performClick", &[], "Z"),
                                    vec![receiver],
                                    true,
                                )?;
                            }
                        }
                        3 => {
                            self.heap
                                .get_mut(receiver)?
                                .fields
                                .remove("droidless:touch:pressed");
                        }
                        _ => unreachable!(),
                    }
                }
                if m.action == 1 || m.action == 3 {
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .remove("droidless:touch:pressed");
                }
                result.push(Word::from(i32::from(clickable)));
            }
            (
                "Landroid/view/ViewConfiguration;",
                "getTapTimeout()I"
                | "getLongPressTimeout()I"
                | "getDoubleTapTimeout()I"
                | "getScaledDoubleTapSlop()I",
            ) => {
                result.push(Word::from(match method_.name.as_str() {
                    "getLongPressTimeout" => 500,
                    "getDoubleTapTimeout" => 300,
                    _ => 100,
                }));
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
    fn view_clickable(&self, word: Word) -> Result<bool> {
        let object = self.heap.get(word)?;
        let view = object.view.as_ref().context("expected View")?;
        Ok(object.fields.get("droidless:touch:clickable").map_or(
            view.kind == "Button" || view.listener.is_some() || view.xml_click.is_some(),
            |v| v.first().is_some_and(|w| w.truth()),
        ))
    }
}

fn velocity(samples: &[Motion], units: i32, maximum: f32) -> [f32; 2] {
    // ponytail: bounded 100-ms linear regression; upgrade to Android's LSQ2 when curved-gesture parity requires it.
    let Some(last) = samples.last() else {
        return [0.0; 2];
    };
    let n = samples.len() as f64;
    let mean_t = samples
        .iter()
        .map(|m| -((last.time - m.time) as f64) / 1000.0)
        .sum::<f64>()
        / n;
    let denominator = samples
        .iter()
        .map(|m| (-((last.time - m.time) as f64) / 1000.0 - mean_t).powi(2))
        .sum::<f64>();
    if denominator < 1e-12 {
        return [0.0; 2];
    }
    [false, true].map(|y| {
        let mean = samples
            .iter()
            .map(|m| f64::from(if y { m.y } else { m.x }))
            .sum::<f64>()
            / n;
        (samples
            .iter()
            .map(|m| {
                (-((last.time - m.time) as f64) / 1000.0 - mean_t)
                    * (f64::from(if y { m.y } else { m.x }) - mean)
            })
            .sum::<f64>()
            / denominator
            * f64::from(units)
            / 1000.0)
            .clamp(-f64::from(maximum), f64::from(maximum)) as f32
    })
}
