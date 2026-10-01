use crate::{heap::Word, vm::Runtime};
use anyhow::{Context, Result};
use droidless_formats::dex::Method;
use std::f32::consts::PI;

const ACCELERATE_DECELERATE: &str = "Landroid/view/animation/AccelerateDecelerateInterpolator;";
const ACCELERATE: &str = "Landroid/view/animation/AccelerateInterpolator;";
const DECELERATE: &str = "Landroid/view/animation/DecelerateInterpolator;";
const LINEAR: &str = "Landroid/view/animation/LinearInterpolator;";

impl Runtime {
    pub(crate) fn animation_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if ![ACCELERATE_DECELERATE, ACCELERATE, DECELERATE, LINEAR].contains(&method.class.as_str())
        {
            return Ok(None);
        }
        let receiver = *args.first().context("interpolator receiver missing")?;
        let argument = |index: usize| {
            args.get(index)
                .copied()
                .with_context(|| format!("{} argument missing", method.key()))
        };
        let class = method.class.as_str();
        match (class, method.signature().as_str()) {
            (ACCELERATE_DECELERATE | LINEAR, "<init>()V") => Ok(Some(vec![])),
            (ACCELERATE, "<init>()V") => {
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
