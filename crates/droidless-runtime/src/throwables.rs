use crate::{
    heap::{Word, exception_parent},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;
use std::collections::BTreeSet;

const THROWABLE: &str = "Ljava/lang/Throwable;";
const TRACE: &str = "droidless:throwable:trace";

impl Runtime {
    pub(crate) fn capture_throwable_trace(&mut self, object: Word) -> Result<()> {
        // ponytail: retain actual DEX locations; source lines need debug-info decoding.
        let locations: Vec<_> = self
            .frames
            .iter()
            .rev()
            .filter(|frame| {
                !matches!(frame.method.name.as_str(), "<init>" | "fillInStackTrace")
                    || !self.is_a(&frame.method.class, THROWABLE)
            })
            .map(|frame| frame.location())
            .collect();
        let mut trace = Vec::with_capacity(locations.len());
        for location in locations {
            trace.push(self.heap.string(location)?);
        }
        self.heap
            .get_mut(object)?
            .fields
            .insert(TRACE.into(), trace);
        Ok(())
    }

    fn throwable_call(&mut self, object: Word, name: &str, returns: &str) -> Result<Word> {
        let roots = self.native_roots.len();
        self.native_roots.push(object);
        let result = self.invoke(
            Method {
                class: THROWABLE.into(),
                name: name.into(),
                parameters: vec![],
                returns: returns.into(),
            },
            vec![object],
            true,
        );
        self.native_roots.truncate(roots);
        result?.first().copied().context("missing Throwable return")
    }

    pub(crate) fn throwable_trace(&mut self, mut object: Word) -> Result<String> {
        let roots = self.native_roots.len();
        let result = (|| {
            let mut output = String::new();
            let mut seen = BTreeSet::new();
            while object != Word::ZERO {
                ensure!(
                    self.is_a(&self.heap.get(object)?.class, THROWABLE),
                    "invalid Throwable cause"
                );
                self.native_roots.push(object);
                let description = self.throwable_call(object, "toString", "Ljava/lang/String;")?;
                let description = if description == Word::ZERO {
                    "null"
                } else {
                    self.heap.text(description)?
                };
                ensure!(
                    output.len() + description.len() + 64 <= 1_048_576,
                    "Throwable trace exceeds 1 MiB"
                );
                if !seen.insert(object.reference()?) {
                    output.push_str(&format!("[CIRCULAR REFERENCE: {description}]\n"));
                    break;
                }
                ensure!(seen.len() <= 128, "Throwable cause chain limit reached");
                output.push_str(description);
                output.push('\n');
                if let Some(trace) = self.heap.get(object)?.fields.get(TRACE) {
                    for location in trace {
                        let location = self.heap.text(*location)?;
                        ensure!(
                            output.len() + location.len() + 2 <= 1_048_576,
                            "Throwable trace exceeds 1 MiB"
                        );
                        output.push('\t');
                        output.push_str(location);
                        output.push('\n');
                    }
                }
                object = self.throwable_call(object, "getCause", THROWABLE)?;
                if object != Word::ZERO {
                    output.push_str("Caused by: ");
                }
            }
            Ok(output)
        })();
        self.native_roots.truncate(roots);
        result
    }

    pub(crate) fn throwable_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let arg = |index| {
            args.get(index)
                .copied()
                .context("Throwable argument missing")
        };
        let signature = method.signature();
        let result = match (method.class.as_str(), signature.as_str()) {
            (
                class,
                "<init>()V"
                | "<init>(Ljava/lang/String;)V"
                | "<init>(Ljava/lang/String;Ljava/lang/Throwable;)V",
            ) if exception_parent(class).is_some()
                && (method.parameters.len() != 2
                    || [
                        THROWABLE,
                        "Ljava/lang/Exception;",
                        "Ljava/lang/RuntimeException;",
                        "Ljava/lang/IllegalStateException;",
                    ]
                    .contains(&class)) =>
            {
                let object = arg(0)?;
                let cause = if method.parameters.len() == 2 {
                    Some(arg(2)?)
                } else {
                    None
                };
                if let Some(cause) = cause {
                    ensure!(
                        cause == Word::ZERO || self.is_a(&self.heap.get(cause)?.class, THROWABLE),
                        "invalid Throwable cause"
                    );
                }
                // API-21 libcore assigns message/cause before virtual fillInStackTrace.
                if signature != "<init>()V" {
                    self.heap
                        .get_mut(object)?
                        .fields
                        .insert("message".into(), vec![arg(1)?]);
                }
                if let Some(cause) = cause {
                    self.heap
                        .get_mut(object)?
                        .fields
                        .insert("cause".into(), vec![cause]);
                }
                self.throwable_call(object, "fillInStackTrace", THROWABLE)?;
                vec![]
            }
            (THROWABLE, "fillInStackTrace()Ljava/lang/Throwable;") => {
                self.capture_throwable_trace(arg(0)?)?;
                vec![arg(0)?]
            }
            (THROWABLE, "getMessage()Ljava/lang/String;") => self
                .heap
                .get(arg(0)?)?
                .fields
                .get("message")
                .cloned()
                .unwrap_or_else(|| vec![Word::ZERO]),
            (THROWABLE, "getLocalizedMessage()Ljava/lang/String;") => {
                vec![self.throwable_call(arg(0)?, "getMessage", "Ljava/lang/String;")?]
            }
            (THROWABLE, "getCause()Ljava/lang/Throwable;")
            | ("Ljava/lang/ExceptionInInitializerError;", "getException()Ljava/lang/Throwable;") => {
                self.heap
                    .get(arg(0)?)?
                    .fields
                    .get("cause")
                    .cloned()
                    .unwrap_or_else(|| vec![Word::ZERO])
            }
            (THROWABLE, "toString()Ljava/lang/String;") => {
                let class = self
                    .heap
                    .get(arg(0)?)?
                    .class
                    .trim_start_matches('L')
                    .trim_end_matches(';')
                    .replace('/', ".");
                let message =
                    self.throwable_call(arg(0)?, "getLocalizedMessage", "Ljava/lang/String;")?;
                let text = if message == Word::ZERO {
                    class
                } else {
                    format!("{class}: {}", self.heap.text(message)?)
                };
                vec![self.heap.string(text)?]
            }
            (THROWABLE, "printStackTrace()V") => {
                eprint!("{}", self.throwable_trace(arg(0)?)?);
                vec![]
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use droidless_formats::apk::Apk;

    fn call(
        vm: &mut Runtime,
        name: &str,
        parameters: &[&str],
        returns: &str,
        args: Vec<Word>,
    ) -> Result<Vec<Word>> {
        vm.invoke(
            Method {
                class: "Lorg/droidless/counter/ThrowableContract;".into(),
                name: name.into(),
                parameters: parameters.iter().map(|value| (*value).into()).collect(),
                returns: returns.into(),
            },
            args,
            false,
        )
    }

    #[test]
    fn compiled_log_throwables_callbacks_gc_nulls_and_fault_recovery() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
        )
        .unwrap();
        for mode in [0, 1, 2, 4, 5] {
            call(
                &mut vm,
                "capture",
                &["I"],
                THROWABLE,
                vec![Word::from(mode)],
            )
            .unwrap();
            for level in 0..4 {
                assert_eq!(
                    call(&mut vm, "logSaved", &["I"], "I", vec![Word::from(level)]).unwrap(),
                    [Word::ZERO]
                );
                assert_eq!(vm.stack_depth(), 0);
                assert!(vm.native_roots.is_empty());
            }
        }
        assert_eq!(
            call(&mut vm, "logNulls", &[], "I", vec![]).unwrap(),
            [Word::from(1)]
        );
        call(&mut vm, "capture", &["I"], THROWABLE, vec![Word::from(3)]).unwrap();
        let error = call(&mut vm, "logSaved", &["I"], "I", vec![Word::from(3)]).unwrap_err();
        assert!(format!("{error:#}").contains("description failed"));
        assert_eq!(vm.stack_depth(), 0);
        assert!(vm.native_roots.is_empty());
        call(&mut vm, "capture", &["I"], THROWABLE, vec![Word::ZERO]).unwrap();
        call(&mut vm, "logSaved", &["I"], "I", vec![Word::from(3)]).unwrap();
        assert!(vm.native_roots.is_empty());
    }

    #[test]
    fn retained_dex_traces_callbacks_gc_causes_and_faults() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
        )
        .unwrap();
        assert_eq!(
            call(&mut vm, "verifyCauseConstructors", &[], "I", vec![]).unwrap(),
            [Word::from(1)]
        );
        assert_eq!(
            call(&mut vm, "api21CauseConstructorState", &[], "I", vec![]).unwrap(),
            [Word::from(1)]
        );
        vm.collect();
        let wrapper =
            vm.statics["Lorg/droidless/counter/ThrowableContract;->saved:Ljava/lang/Throwable;"][0];
        let trace = vm.throwable_trace(wrapper).unwrap();
        assert!(
            trace.contains("CauseTrace: outer cause")
                && trace.contains("Caused by: java.lang.IllegalArgumentException: inner cause"),
            "{trace}"
        );
        let error = call(&mut vm, "causeConstructorFault", &[], "V", vec![]).unwrap_err();
        assert!(format!("{error:#}").contains("cause trace failed"));
        assert_eq!(vm.stack_depth(), 0);
        assert!(vm.native_roots.is_empty());
        for mode in [0, 1, 2, 4] {
            let exception = call(
                &mut vm,
                "capture",
                &["I"],
                THROWABLE,
                vec![Word::from(mode)],
            )
            .unwrap()[0];
            vm.collect();
            let trace = vm.throwable_trace(exception).unwrap();
            assert!(
                trace.contains("->leaf(I)Ljava/lang/Throwable; [classes.dex, PC 0x"),
                "{trace}"
            );
            assert!(trace.contains("->outer(I)Ljava/lang/Throwable;"), "{trace}");
            assert!(
                trace.contains("->capture(I)Ljava/lang/Throwable;"),
                "{trace}"
            );
            assert!(
                !trace.contains("->printSaved") && !trace.contains("-><init>"),
                "{trace}"
            );
            assert!(
                trace.starts_with(match mode {
                    0 => "java.lang.IllegalStateException: saved message\n",
                    1 => "java.lang.ArithmeticException:",
                    2 => "org.droidless.counter.ThrowableContract$Localized: localized message\n",
                    4 => "org.droidless.counter.ThrowableContract$Chain: outer\n",
                    _ => unreachable!(),
                }),
                "{trace}"
            );
            if mode == 4 {
                assert!(
                    trace.contains(
                        "Caused by: org.droidless.counter.ThrowableContract$Chain: inner"
                    )
                );
                assert!(trace.contains(
                    "[CIRCULAR REFERENCE: org.droidless.counter.ThrowableContract$Chain: outer]"
                ));
            }
            assert_eq!(
                call(&mut vm, "printSaved", &[], "I", vec![]).unwrap(),
                [Word::from(1)]
            );
            assert_eq!(vm.stack_depth(), 0);
            assert!(vm.native_roots.is_empty());
        }
        let exception = call(&mut vm, "capture", &["I"], THROWABLE, vec![Word::ZERO]).unwrap()[0];
        assert_eq!(
            call(&mut vm, "refresh", &[], THROWABLE, vec![]).unwrap(),
            [exception]
        );
        let trace = vm.throwable_trace(exception).unwrap();
        assert!(trace.contains("->refresh()Ljava/lang/Throwable;") && !trace.contains("->leaf"));
        call(&mut vm, "capture", &["I"], THROWABLE, vec![Word::from(3)]).unwrap();
        let error = call(&mut vm, "printSaved", &[], "I", vec![]).unwrap_err();
        assert!(format!("{error:#}").contains("description failed"));
        assert_eq!(vm.stack_depth(), 0);
        assert!(vm.native_roots.is_empty());

        let exception =
            call(&mut vm, "capture", &["I"], THROWABLE, vec![Word::from(5)]).unwrap()[0];
        assert_eq!(
            vm.throwable_trace(exception).unwrap(),
            "org.droidless.counter.ThrowableContract$NoTrace\n"
        );
        let localized = vm
            .heap
            .instance("Lorg/droidless/counter/ThrowableContract$Localized;")
            .unwrap();
        assert!(
            vm.throwable_trace(localized)
                .unwrap()
                .contains("localized message")
        );
        assert!(vm.heap.get(localized).is_ok());
        assert!(vm.native_roots.is_empty());

        let parent = vm.heap.instance(THROWABLE).unwrap();
        let cause = vm
            .heap
            .instance("Ljava/lang/IllegalStateException;")
            .unwrap();
        vm.heap
            .get_mut(parent)
            .unwrap()
            .fields
            .insert("cause".into(), vec![cause]);
        assert!(
            vm.throwable_trace(parent)
                .unwrap()
                .contains("Caused by: java.lang.IllegalStateException")
        );
        vm.heap
            .get_mut(cause)
            .unwrap()
            .fields
            .insert("cause".into(), vec![parent]);
        assert!(
            vm.throwable_trace(parent)
                .unwrap()
                .contains("[CIRCULAR REFERENCE: java.lang.Throwable]")
        );
        let string = vm.heap.string("bad cause".into()).unwrap();
        vm.heap
            .get_mut(parent)
            .unwrap()
            .fields
            .insert("cause".into(), vec![string]);
        assert!(
            vm.throwable_trace(parent)
                .unwrap_err()
                .to_string()
                .contains("invalid Throwable cause")
        );
        assert!(vm.native_roots.is_empty());

        let mut chain = vm.heap.instance(THROWABLE).unwrap();
        for _ in 0..128 {
            let wrapper = vm.heap.instance(THROWABLE).unwrap();
            vm.heap
                .get_mut(wrapper)
                .unwrap()
                .fields
                .insert("cause".into(), vec![chain]);
            chain = wrapper;
        }
        assert!(
            vm.throwable_trace(chain)
                .unwrap_err()
                .to_string()
                .contains("cause chain limit")
        );
        let message = vm.heap.string("x".repeat(600_000)).unwrap();
        vm.heap
            .get_mut(chain)
            .unwrap()
            .fields
            .insert("message".into(), vec![message]);
        let next = vm.heap.get(chain).unwrap().fields["cause"][0];
        vm.heap
            .get_mut(next)
            .unwrap()
            .fields
            .insert("message".into(), vec![message]);
        assert!(
            vm.throwable_trace(chain)
                .unwrap_err()
                .to_string()
                .contains("trace exceeds")
        );
        assert!(vm.native_roots.is_empty());
    }
}
