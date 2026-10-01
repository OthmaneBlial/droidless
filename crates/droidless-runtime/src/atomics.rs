use crate::{
    heap::{Data, Word, bits64, wide},
    vm::Runtime,
};
use anyhow::{Context, Result};
use droidless_formats::dex::Method;
use std::sync::{
    Arc,
    atomic::{AtomicBool, AtomicI32, AtomicI64, Ordering},
};

impl Runtime {
    pub(crate) fn atomic_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let signature = method.signature();
        let argument = |index: usize| {
            args.get(index)
                .copied()
                .with_context(|| format!("{} argument missing", method.key()))
        };
        match (method.class.as_str(), signature.as_str()) {
            ("Ljava/util/concurrent/atomic/AtomicInteger;", "<init>()V") => {
                self.heap.get_mut(receiver)?.data =
                    Data::AtomicInteger(Arc::new(AtomicI32::new(0)));
                Ok(Some(vec![]))
            }
            ("Ljava/util/concurrent/atomic/AtomicInteger;", "<init>(I)V") => {
                self.heap.get_mut(receiver)?.data =
                    Data::AtomicInteger(Arc::new(AtomicI32::new(argument(1)?.int()?)));
                Ok(Some(vec![]))
            }
            ("Ljava/util/concurrent/atomic/AtomicInteger;", "get()I") => {
                let Data::AtomicInteger(value) = &self.heap.get(receiver)?.data else {
                    anyhow::bail!("uninitialized AtomicInteger")
                };
                Ok(Some(vec![Word::from(value.load(Ordering::SeqCst))]))
            }
            ("Ljava/util/concurrent/atomic/AtomicInteger;", "incrementAndGet()I") => {
                let Data::AtomicInteger(value) = &self.heap.get(receiver)?.data else {
                    anyhow::bail!("uninitialized AtomicInteger")
                };
                Ok(Some(vec![Word::from(
                    value.fetch_add(1, Ordering::SeqCst).wrapping_add(1),
                )]))
            }
            ("Ljava/util/concurrent/atomic/AtomicLong;", "<init>(J)V") => {
                self.heap.get_mut(receiver)?.data =
                    Data::AtomicLong(Arc::new(AtomicI64::new(bits64(&args[1..])? as i64)));
                Ok(Some(vec![]))
            }
            ("Ljava/util/concurrent/atomic/AtomicLong;", "incrementAndGet()J") => {
                let Data::AtomicLong(value) = &self.heap.get(receiver)?.data else {
                    anyhow::bail!("uninitialized AtomicLong")
                };
                Ok(Some(wide(
                    value.fetch_add(1, Ordering::SeqCst).wrapping_add(1) as u64,
                )))
            }
            ("Ljava/util/concurrent/atomic/AtomicBoolean;", "<init>()V") => {
                self.heap.get_mut(receiver)?.data =
                    Data::AtomicBoolean(Arc::new(AtomicBool::new(false)));
                Ok(Some(vec![]))
            }
            ("Ljava/util/concurrent/atomic/AtomicBoolean;", "<init>(Z)V") => {
                self.heap.get_mut(receiver)?.data =
                    Data::AtomicBoolean(Arc::new(AtomicBool::new(argument(1)?.int()? != 0)));
                Ok(Some(vec![]))
            }
            ("Ljava/util/concurrent/atomic/AtomicBoolean;", "get()Z") => {
                let Data::AtomicBoolean(value) = &self.heap.get(receiver)?.data else {
                    anyhow::bail!("uninitialized AtomicBoolean")
                };
                Ok(Some(vec![Word::from(i32::from(
                    value.load(Ordering::SeqCst),
                ))]))
            }
            ("Ljava/util/concurrent/atomic/AtomicBoolean;", "getAndSet(Z)Z") => {
                let Data::AtomicBoolean(value) = &self.heap.get(receiver)?.data else {
                    anyhow::bail!("uninitialized AtomicBoolean")
                };
                Ok(Some(vec![Word::from(i32::from(
                    value.swap(argument(1)?.int()? != 0, Ordering::SeqCst),
                ))]))
            }
            ("Ljava/util/concurrent/atomic/AtomicBoolean;", "set(Z)V") => {
                let Data::AtomicBoolean(value) = &self.heap.get(receiver)?.data else {
                    anyhow::bail!("uninitialized AtomicBoolean")
                };
                value.store(argument(1)?.int()? != 0, Ordering::SeqCst);
                Ok(Some(vec![]))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;
    use droidless_formats::apk::Apk;

    fn runtime() -> Runtime {
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap())
            .unwrap()
    }
    fn invoke(
        vm: &mut Runtime,
        class: &str,
        receiver: Word,
        name: &str,
        parameters: &[&str],
        extra: &[Word],
    ) -> Vec<Word> {
        vm.invoke(
            Method {
                class: class.into(),
                name: name.into(),
                parameters: parameters.iter().map(|p| (*p).into()).collect(),
                returns: match (class, name) {
                    ("Ljava/util/concurrent/atomic/AtomicInteger;", "get" | "incrementAndGet") => {
                        "I"
                    }
                    ("Ljava/util/concurrent/atomic/AtomicLong;", "incrementAndGet") => "J",
                    ("Ljava/util/concurrent/atomic/AtomicBoolean;", "get" | "getAndSet") => "Z",
                    _ => "V",
                }
                .into(),
            },
            std::iter::once(receiver)
                .chain(extra.iter().copied())
                .collect(),
            false,
        )
        .unwrap()
    }

    #[test]
    fn atomic_integer_and_boolean_methods_share_ordered_values() {
        let mut vm = runtime();
        let zero = vm
            .heap
            .instance("Ljava/util/concurrent/atomic/AtomicInteger;")
            .unwrap();
        invoke(
            &mut vm,
            "Ljava/util/concurrent/atomic/AtomicInteger;",
            zero,
            "<init>",
            &[],
            &[],
        );
        assert_eq!(
            invoke(
                &mut vm,
                "Ljava/util/concurrent/atomic/AtomicInteger;",
                zero,
                "get",
                &[],
                &[],
            )[0]
            .int()
            .unwrap(),
            0
        );
        let integer = vm
            .heap
            .instance("Ljava/util/concurrent/atomic/AtomicInteger;")
            .unwrap();
        invoke(
            &mut vm,
            "Ljava/util/concurrent/atomic/AtomicInteger;",
            integer,
            "<init>",
            &["I"],
            &[Word::from(4)],
        );
        assert_eq!(
            invoke(
                &mut vm,
                "Ljava/util/concurrent/atomic/AtomicInteger;",
                integer,
                "get",
                &[],
                &[],
            )[0]
            .int()
            .unwrap(),
            4
        );
        assert_eq!(
            invoke(
                &mut vm,
                "Ljava/util/concurrent/atomic/AtomicInteger;",
                integer,
                "incrementAndGet",
                &[],
                &[],
            )[0]
            .int()
            .unwrap(),
            5
        );

        let boolean = vm
            .heap
            .instance("Ljava/util/concurrent/atomic/AtomicBoolean;")
            .unwrap();
        invoke(
            &mut vm,
            "Ljava/util/concurrent/atomic/AtomicBoolean;",
            boolean,
            "<init>",
            &["Z"],
            &[Word::ZERO],
        );
        assert_eq!(
            invoke(
                &mut vm,
                "Ljava/util/concurrent/atomic/AtomicBoolean;",
                boolean,
                "getAndSet",
                &["Z"],
                &[Word::from(1)],
            )[0]
            .int()
            .unwrap(),
            0
        );
        assert_eq!(
            invoke(
                &mut vm,
                "Ljava/util/concurrent/atomic/AtomicBoolean;",
                boolean,
                "get",
                &[],
                &[],
            )[0]
            .int()
            .unwrap(),
            1
        );

        let long = vm
            .heap
            .instance("Ljava/util/concurrent/atomic/AtomicLong;")
            .unwrap();
        invoke(
            &mut vm,
            "Ljava/util/concurrent/atomic/AtomicLong;",
            long,
            "<init>",
            &["J"],
            &[Word::Bits(40), Word::ZERO],
        );
        assert_eq!(
            bits64(&invoke(
                &mut vm,
                "Ljava/util/concurrent/atomic/AtomicLong;",
                long,
                "incrementAndGet",
                &[],
                &[],
            ))
            .unwrap(),
            41
        );
    }
}
