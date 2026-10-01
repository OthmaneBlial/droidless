use crate::{
    heap::{Word, bits64, wide},
    vm::Runtime,
};
use anyhow::{Context, Result};
use droidless_formats::dex::Method;
use std::f32::consts::PI;

const ACCELERATE_DECELERATE: &str = "Landroid/view/animation/AccelerateDecelerateInterpolator;";
const ACCELERATE: &str = "Landroid/view/animation/AccelerateInterpolator;";
const DECELERATE: &str = "Landroid/view/animation/DecelerateInterpolator;";
const LINEAR: &str = "Landroid/view/animation/LinearInterpolator;";
const ANIMATION: &str = "Landroid/view/animation/Animation;";
const ANIMATION_UTILS: &str = "Landroid/view/animation/AnimationUtils;";

impl Runtime {
    pub(crate) fn animation_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let argument = |index: usize| {
            args.get(index)
                .copied()
                .with_context(|| format!("{} argument missing", method.key()))
        };
        let class = method.class.as_str();
        let signature = method.signature();
        if class == ANIMATION_UTILS
            && signature
                == "loadAnimation(Landroid/content/Context;I)Landroid/view/animation/Animation;"
        {
            self.heap.get(argument(0)?)?;
            let animation = self.heap.instance(ANIMATION)?;
            self.heap
                .get_mut(animation)?
                .fields
                .insert("droidless:animation:resource".into(), vec![argument(1)?]);
            return Ok(Some(vec![animation]));
        }
        if class == ANIMATION_UTILS
            && signature
                == "loadInterpolator(Landroid/content/Context;I)Landroid/view/animation/Interpolator;"
        {
            self.heap.get(argument(0)?)?;
            let interpolator = self.heap.instance(LINEAR)?;
            self.heap
                .get_mut(interpolator)?
                .fields
                .insert("droidless:interpolator:resource".into(), vec![argument(1)?]);
            return Ok(Some(vec![interpolator]));
        }
        if class == ANIMATION {
            let result = match signature.as_str() {
                "<init>()V" | "cancel()V" | "start()V" | "startNow()V" | "reset()V" => {
                    self.heap.get(receiver)?;
                    vec![]
                }
                "setDuration(J)V" | "setDuration(J)Landroid/view/animation/Animation;" => {
                    let duration = bits64(&args[1..3])?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:animation:duration".into(), wide(duration));
                    if signature.ends_with("Animation;") {
                        vec![receiver]
                    } else {
                        vec![]
                    }
                }
                "getDuration()J" => self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:animation:duration")
                    .cloned()
                    .unwrap_or_else(|| wide(0)),
                "setStartOffset(J)V" => {
                    let offset = bits64(&args[1..3])?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:animation:start-offset".into(), wide(offset));
                    vec![]
                }
                "setInterpolator(Landroid/view/animation/Interpolator;)V"
                | "setAnimationListener(Landroid/view/animation/Animation$AnimationListener;)V" => {
                    let value = argument(1)?;
                    if value != Word::ZERO {
                        self.heap.get(value)?;
                    }
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(format!("droidless:animation:{}", method.name), vec![value]);
                    vec![]
                }
                _ => return Ok(None),
            };
            // ponytail: native-window animations stay nonvisual; add timed listener delivery when a real app workflow depends on it.
            return Ok(Some(result));
        }
        if ![ACCELERATE_DECELERATE, ACCELERATE, DECELERATE, LINEAR].contains(&class) {
            return Ok(None);
        }
        match (class, method.signature().as_str()) {
            (ACCELERATE_DECELERATE | LINEAR, "<init>()V") => Ok(Some(vec![])),
            (ACCELERATE, "<init>()V") => {
                self.store_factor(receiver, 1.0)?;
                Ok(Some(vec![]))
            }
            (DECELERATE, "<init>()V") => {
                self.store_factor(receiver, 1.0)?;
                Ok(Some(vec![]))
            }
            (ACCELERATE | DECELERATE, "<init>(F)V") => {
                let factor = f32::from_bits(argument(1)?.int()? as u32);
                self.store_factor(receiver, factor)?;
                Ok(Some(vec![]))
            }
            (ACCELERATE_DECELERATE | ACCELERATE | DECELERATE | LINEAR, "getInterpolation(F)F") => {
                let input = f32::from_bits(argument(1)?.int()? as u32);
                let value = match class {
                    ACCELERATE_DECELERATE => ((input + 1.0) * PI).cos() / 2.0 + 0.5,
                    ACCELERATE => input.powf(2.0 * self.factor(receiver)?),
                    DECELERATE => 1.0 - (1.0 - input).powf(2.0 * self.factor(receiver)?),
                    _ => input,
                };
                Ok(Some(vec![Word::from(value.to_bits() as i32)]))
            }
            _ => Ok(None),
        }
    }

    fn store_factor(&mut self, receiver: Word, factor: f32) -> Result<()> {
        self.heap.get_mut(receiver)?.fields.insert(
            "droidless:interpolator:factor".into(),
            vec![Word::from(factor.to_bits() as i32)],
        );
        Ok(())
    }

    fn factor(&self, receiver: Word) -> Result<f32> {
        Ok(f32::from_bits(
            self.heap
                .get(receiver)?
                .fields
                .get("droidless:interpolator:factor")
                .and_then(|value| value.first())
                .copied()
                .context("uninitialized interpolator")?
                .int()? as u32,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use droidless_formats::apk::Apk;

    #[test]
    fn decelerate_interpolator_applies_its_factor() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap(),
        )
        .unwrap();
        let object = vm.heap.instance(DECELERATE).unwrap();
        vm.invoke(
            Method {
                class: DECELERATE.into(),
                name: "<init>".into(),
                parameters: vec!["F".into()],
                returns: "V".into(),
            },
            vec![object, Word::from(2.0f32.to_bits() as i32)],
            false,
        )
        .unwrap();
        let result = vm
            .invoke(
                Method {
                    class: DECELERATE.into(),
                    name: "getInterpolation".into(),
                    parameters: vec!["F".into()],
                    returns: "F".into(),
                },
                vec![object, Word::from(0.5f32.to_bits() as i32)],
                false,
            )
            .unwrap()[0]
            .int()
            .unwrap() as u32;
        assert!((f32::from_bits(result) - 0.9375).abs() < 0.00001);
    }
}
