//! Immediate FIFO operations and resumable worker take/put.
use crate::{
    heap::{Data, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;

const QUEUE: &str = "Ljava/util/concurrent/LinkedBlockingQueue;";

impl Runtime {
    pub(crate) fn queue_capacity(&self, owner: Word) -> Result<i32> {
        self.heap
            .get(owner)?
            .fields
            .get("capacity")
            .and_then(|v| v.first())
            .context("uninitialized queue capacity")?
            .int()
    }
    fn queue_find(&mut self, owner: Word, key: Word) -> Result<Option<usize>> {
        if key.reference()? == 0 {
            return Ok(None);
        }
        let (values, version) = self.collection(owner)?;
        let len = values.len();
        for index in 0..len {
            let value = self.collection(owner)?.0[index];
            let equal = self.object_equal(key, value, false)?;
            // LinkedBlockingQueue is not fail-fast: do not invent a ConcurrentModificationException.
            ensure!(
                self.collection(owner)?.1 == version,
                "unsupported queue mutation during guest equals"
            );
            if equal {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    pub(crate) fn queue_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class != QUEUE {
            return Ok(None);
        }
        let owner = *args.first().context("queue receiver missing")?;
        let arg = |n| args.get(n).copied().context("queue argument missing");
        ensure!(
            self.is_a(&self.heap.get(owner)?.class, QUEUE),
            "invalid queue receiver"
        );
        let sig = method.signature();
        let mut result = vec![];
        match sig.as_str() {
            "<init>()V" | "<init>(I)V" => {
                let capacity = if method.parameters.is_empty() {
                    i32::MAX
                } else {
                    arg(1)?.int()?
                };
                if capacity <= 0 {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "nonpositive queue capacity",
                    ));
                }
                let object = self.heap.get_mut(owner)?;
                object.data = Data::Collection {
                    values: vec![],
                    version: 0,
                };
                object
                    .fields
                    .insert("capacity".into(), vec![Word::from(capacity)]);
            }
            "size()I" => result.push(Word::from(self.collection(owner)?.0.len() as i32)),
            "remainingCapacity()I" => result.push(Word::from(
                self.queue_capacity(owner)? - self.collection(owner)?.0.len() as i32,
            )),
            "take()Ljava/lang/Object;" => {
                self.check_interrupt()?;
                let mut values = self.collection(owner)?.0.to_vec();
                if values.is_empty() {
                    self.wait_worker(crate::workers::Waiting::Take(owner))?;
                }
                result.push(values.remove(0));
                self.change_collection(owner, values)?;
            }
            "put(Ljava/lang/Object;)V" => {
                let value = arg(1)?;
                if value.reference()? == 0 {
                    return Err(fault(
                        "Ljava/lang/NullPointerException;",
                        "null queue element",
                    ));
                }
                self.check_interrupt()?;
                let mut values = self.collection(owner)?.0.to_vec();
                if values.len() == self.queue_capacity(owner)? as usize {
                    self.wait_worker(crate::workers::Waiting::Put(owner))?;
                }
                values.push(value);
                self.change_collection(owner, values)?;
            }
            "offer(Ljava/lang/Object;)Z" => {
                let value = arg(1)?;
                if value.reference()? == 0 {
                    return Err(fault(
                        "Ljava/lang/NullPointerException;",
                        "null queue element",
                    ));
                }
                let (values, _) = self.collection(owner)?;
                if values.len() == self.queue_capacity(owner)? as usize {
                    result.push(Word::ZERO);
                } else {
                    // ponytail: reuse bounded collection storage; use VecDeque if FIFO copies become measurable.
                    let mut values = values.to_vec();
                    values.push(value);
                    self.change_collection(owner, values)?;
                    result.push(Word::from(1));
                }
            }
            "add(Ljava/lang/Object;)Z" => {
                result = self.invoke(
                    Method {
                        class: QUEUE.into(),
                        name: "offer".into(),
                        parameters: vec!["Ljava/lang/Object;".into()],
                        returns: "Z".into(),
                    },
                    args.to_vec(),
                    true,
                )?;
                if result.first().context("offer returned no value")?.int()? == 0 {
                    return Err(fault("Ljava/lang/IllegalStateException;", "Queue full"));
                }
            }
            "poll()Ljava/lang/Object;" => {
                let (values, _) = self.collection(owner)?;
                if values.is_empty() {
                    result.push(Word::ZERO);
                } else {
                    let mut values = values.to_vec();
                    result.push(values.remove(0));
                    self.change_collection(owner, values)?;
                }
            }
            "peek()Ljava/lang/Object;" => result.push(
                self.collection(owner)?
                    .0
                    .first()
                    .copied()
                    .unwrap_or(Word::ZERO),
            ),
            "element()Ljava/lang/Object;" | "remove()Ljava/lang/Object;" | "isEmpty()Z" => {
                let (name, returns) = match method.name.as_str() {
                    "element" => ("peek", "Ljava/lang/Object;"),
                    "remove" => ("poll", "Ljava/lang/Object;"),
                    _ => ("size", "I"),
                };
                let words = self.invoke(
                    Method {
                        class: QUEUE.into(),
                        name: name.into(),
                        parameters: vec![],
                        returns: returns.into(),
                    },
                    vec![owner],
                    true,
                )?;
                let value = *words.first().context("queue operation returned no value")?;
                if method.name == "isEmpty" {
                    result.push(Word::from(i32::from(value.int()? == 0)));
                } else {
                    if value.reference()? == 0 {
                        return Err(fault("Ljava/util/NoSuchElementException;", "empty queue"));
                    }
                    result.push(value);
                }
            }
            "contains(Ljava/lang/Object;)Z" | "remove(Ljava/lang/Object;)Z" => {
                let index = self.queue_find(owner, arg(1)?)?;
                if method.name == "remove"
                    && let Some(index) = index
                {
                    let mut values = self.collection(owner)?.0.to_vec();
                    values.remove(index);
                    self.change_collection(owner, values)?;
                }
                result.push(Word::from(i32::from(index.is_some())));
            }
            "clear()V" => self.change_collection(owner, vec![])?,
            "toString()Ljava/lang/String;" => bail!("unsupported queue method {}", method.key()),
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use droidless_formats::apk::Apk;

    #[test]
    fn capacity_mutation_and_unsupported_operations_preserve_queue_state() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!(
                "../../../fixtures/generated/collections.apk"
            ))
            .unwrap(),
        )
        .unwrap();
        vm.launch().unwrap();
        let mutation = Method {
            class: "Lorg/droidless/collections/QueueContract;".into(),
            name: "mutation".into(),
            parameters: vec![],
            returns: "V".into(),
        };
        let error = format!("{:#}", vm.invoke(mutation, vec![], false).unwrap_err());
        assert!(error.contains("unsupported queue mutation during guest equals"));
        assert!(!error.contains("ConcurrentModificationException"));
        assert_eq!(vm.stack_depth(), 0);
        let queue = vm.heap.instance(QUEUE).unwrap();
        let constructor = Method {
            class: QUEUE.into(),
            name: "<init>".into(),
            parameters: vec![],
            returns: "V".into(),
        };
        vm.invoke(constructor, vec![queue], false).unwrap();
        let item = vm.heap.string("existing FIFO values".into()).unwrap();
        let before = vec![item; 16_384];
        vm.heap.get_mut(queue).unwrap().data = Data::Collection {
            values: before.clone(),
            version: 0,
        };
        let offer = Method {
            class: "Ljava/util/concurrent/BlockingQueue;".into(),
            name: "offer".into(),
            parameters: vec!["Ljava/lang/Object;".into()],
            returns: "Z".into(),
        };
        assert!(
            format!(
                "{:#}",
                vm.invoke(offer, vec![queue, item], true).unwrap_err()
            )
            .contains("entry limit")
        );
        for (name, args, parameters, returns) in [
            ("iterator", vec![queue], vec![], "Ljava/util/Iterator;"),
            ("toString", vec![queue], vec![], "Ljava/lang/String;"),
        ] {
            let method = Method {
                class: QUEUE.into(),
                name: name.into(),
                parameters,
                returns: returns.into(),
            };
            assert!(
                format!("{:#}", vm.invoke(method, args, true).unwrap_err()).contains("unsupported")
            );
            let (values, version) = vm.collection(queue).unwrap();
            assert_eq!(values, &before);
            assert_eq!(version, 0);
        }
        let tiny = vm.heap.instance(QUEUE).unwrap();
        let method = |name: &str, parameters: &[&str], returns: &str| Method {
            class: QUEUE.into(),
            name: name.into(),
            parameters: parameters.iter().map(|s| (*s).into()).collect(),
            returns: returns.into(),
        };
        vm.invoke(
            method("<init>", &["I"], "V"),
            vec![tiny, Word::from(1)],
            false,
        )
        .unwrap();
        vm.invoke(
            method("put", &["Ljava/lang/Object;"], "V"),
            vec![tiny, item],
            true,
        )
        .unwrap();
        let error = vm
            .invoke(
                method("put", &["Ljava/lang/Object;"], "V"),
                vec![tiny, item],
                true,
            )
            .unwrap_err();
        assert!(format!("{error:#}").contains("unsupported blocking wait on the main thread"));
        assert_eq!(vm.collection(tiny).unwrap().0, &[item]);
        assert_eq!(
            vm.invoke(method("take", &[], "Ljava/lang/Object;"), vec![tiny], true)
                .unwrap(),
            vec![item]
        );
        let error = vm
            .invoke(method("take", &[], "Ljava/lang/Object;"), vec![tiny], true)
            .unwrap_err();
        assert!(format!("{error:#}").contains("unsupported blocking wait on the main thread"));
        assert!(vm.collection(tiny).unwrap().0.is_empty());
    }
}
