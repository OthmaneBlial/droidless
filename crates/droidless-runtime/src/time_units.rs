use crate::{
    heap::{Data, TimeUnit, Word, bits64, wide},
    vm::Runtime,
};
use anyhow::{Context, Result};
use droidless_formats::dex::Method;

impl Runtime {
    pub(crate) fn time_unit_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class != "Ljava/util/concurrent/TimeUnit;" {
            return Ok(None);
        }
        let signature = method.signature();
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let target = match &self.heap.get(receiver)?.data {
            Data::TimeUnit(unit) => *unit,
            _ => return Ok(None),
        };
        let target = match signature.as_str() {
            "toNanos(J)J" => TimeUnit::Nanoseconds,
            "toMillis(J)J" => TimeUnit::Milliseconds,
            "toSeconds(J)J" => TimeUnit::Seconds,
            "toMinutes(J)J" => TimeUnit::Minutes,
            "toHours(J)J" => TimeUnit::Hours,
            "toDays(J)J" => TimeUnit::Days,
            "convert(Ljava/util/concurrent/TimeUnit;J)J" => target,
            _ => return Ok(None),
        };
        let (source, value_index) = if signature == "convert(Ljava/util/concurrent/TimeUnit;J)J" {
            let source = args
                .get(1)
                .copied()
                .context("TimeUnit.convert source unit missing")?;
            let unit = match &self.heap.get(source)?.data {
                Data::TimeUnit(unit) => *unit,
                _ => anyhow::bail!("TimeUnit.convert requires a TimeUnit source"),
            };
            (unit, 2)
        } else {
            (
                match &self.heap.get(receiver)?.data {
                    Data::TimeUnit(unit) => *unit,
                    _ => unreachable!(),
                },
                1,
            )
        };
        let value = bits64(&args[value_index..])? as i64;
        Ok(Some(wide(source.convert(target, value) as u64)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;
    use droidless_formats::{apk::Apk, dex::Field};

    #[test]
    fn time_unit_constants_resolve_and_conversions_saturate() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap(),
        )
        .unwrap();
        let class = "Ljava/util/concurrent/TimeUnit;";
        let field = |name: &str| Field {
            class: class.into(),
            name: name.into(),
            ty: class.into(),
        };
        let milliseconds = vm.resolve_field(&field("MILLISECONDS"), true).unwrap();
        let millis = vm
            .time_unit_object(vm.time_unit_field(&milliseconds).unwrap())
            .unwrap();
        let result = vm
            .invoke(
                Method {
                    class: class.into(),
                    name: "toNanos".into(),
                    parameters: vec!["J".into()],
                    returns: "J".into(),
                },
                vec![millis, Word::from(5_000), Word::ZERO],
                false,
            )
            .unwrap();
        assert_eq!(bits64(&result).unwrap(), 5_000_000_000);

        let seconds = vm
            .time_unit_object(TimeUnit::named("SECONDS").unwrap())
            .unwrap();
        let result = vm
            .invoke(
                Method {
                    class: class.into(),
                    name: "toNanos".into(),
                    parameters: vec!["J".into()],
                    returns: "J".into(),
                },
                std::iter::once(seconds)
                    .chain(wide(i64::MAX as u64))
                    .collect(),
                false,
            )
            .unwrap();
        assert_eq!(bits64(&result).unwrap() as i64, i64::MAX);
        assert!(
            vm.resolve_field(&field("UNITS_DOES_NOT_EXIST"), true)
                .is_err()
        );
    }
}
