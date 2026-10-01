use crate::{
    heap::{Data, Word, fault, wide},
    vm::Runtime,
};
use anyhow::{Context, Result};
use droidless_formats::dex::Method;

impl Runtime {
    pub(crate) fn system_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class == "Landroid/os/Process;" && method.signature() == "setThreadPriority(I)V" {
            args.first()
                .context("Process.setThreadPriority priority missing")?
                .int()?;
            // ponytail: guest priorities do not affect host scheduling; model them if timing fidelity matters.
            return Ok(Some(vec![]));
        }
        if method.class.starts_with('[') && method.signature() == "clone()Ljava/lang/Object;" {
            let receiver = args
                .first()
                .copied()
                .context("array clone receiver missing")?;
            let clone = {
                let object = self.heap.get(receiver)?;
                let Data::Array { element, values } = &object.data else {
                    anyhow::bail!("array clone receiver is not an array");
                };
                crate::heap::Object {
                    class: object.class.clone(),
                    fields: object.fields.clone(),
                    data: Data::Array {
                        element: element.clone(),
                        values: values.clone(),
                    },
                    view: None,
                }
            };
            return Ok(Some(vec![self.heap.alloc(clone)?]));
        }
        if method.class == "Ljava/util/Arrays;" && method.name == "fill" {
            return self.arrays_fill(method, args);
        }
        if method.class != "Ljava/lang/System;" {
            return Ok(None);
        }
        match method.signature().as_str() {
            "currentTimeMillis()J" => {
                let now = std::time::SystemTime::now();
                let millis = match now.duration_since(std::time::UNIX_EPOCH) {
                    Ok(elapsed) => elapsed.as_millis().min(i64::MAX as u128) as i64,
                    Err(before) => -(before.duration().as_millis().min(i64::MAX as u128) as i64),
                };
                Ok(Some(wide(millis as u64)))
            }
            "nanoTime()J" => Ok(Some(wide(
                self.started.elapsed().as_nanos().min(i64::MAX as u128) as u64,
            ))),
            "identityHashCode(Ljava/lang/Object;)I" => {
                let object = args
                    .first()
                    .copied()
                    .context("System.identityHashCode argument missing")?;
                Ok(Some(vec![Word::from(object.reference()? as u32 as i32)]))
            }
            "arraycopy(Ljava/lang/Object;ILjava/lang/Object;II)V" => {
                let source = args
                    .first()
                    .copied()
                    .context("System.arraycopy source missing")?;
                let source_position = array_index(args.get(1).copied(), "source position")?;
                let target = args
                    .get(2)
                    .copied()
                    .context("System.arraycopy destination missing")?;
                let target_position = array_index(args.get(3).copied(), "destination position")?;
                let length = array_index(args.get(4).copied(), "length")?;
                let (source_type, source_values) = match &self.heap.get(source)?.data {
                    Data::Array { element, values } => (element.clone(), values.clone()),
                    _ => {
                        return Err(fault(
                            "Ljava/lang/ArrayStoreException;",
                            "source must be an array",
                        ));
                    }
                };
                let (target_type, target_length) = match &self.heap.get(target)?.data {
                    Data::Array { element, values } => (element.clone(), values.len()),
                    _ => {
                        return Err(fault(
                            "Ljava/lang/ArrayStoreException;",
                            "destination must be an array",
                        ));
                    }
                };
                if source_type != target_type
                    && (is_primitive(&source_type) || is_primitive(&target_type))
                {
                    return Err(fault(
                        "Ljava/lang/ArrayStoreException;",
                        "primitive array types must match",
                    ));
                }
                if source_position > source_values.len()
                    || length > source_values.len().saturating_sub(source_position)
                    || target_position > target_length
                    || length > target_length.saturating_sub(target_position)
                {
                    return Err(fault(
                        "Ljava/lang/ArrayIndexOutOfBoundsException;",
                        "arraycopy range is out of bounds",
                    ));
                }
                let values = &source_values[source_position..source_position + length];
                for (offset, value) in values.iter().enumerate() {
                    if !is_primitive(&target_type) && value.first().copied() != Some(Word::ZERO) {
                        let compatible = value
                            .first()
                            .copied()
                            .and_then(|value| value.reference().ok().map(Word::Ref))
                            .is_some_and(|value| {
                                self.heap
                                    .get(value)
                                    .is_ok_and(|object| self.is_a(&object.class, &target_type))
                            });
                        if !compatible {
                            return Err(fault(
                                "Ljava/lang/ArrayStoreException;",
                                format!("element cannot be stored in {target_type}"),
                            ));
                        }
                    }
                    let Data::Array {
                        values: target_values,
                        ..
                    } = &mut self.heap.get_mut(target)?.data
                    else {
                        unreachable!();
                    };
                    target_values[target_position + offset] = value.clone();
                }
                Ok(Some(vec![]))
            }
            _ => Ok(None),
        }
    }

    fn arrays_fill(&mut self, method: &Method, args: &[Word]) -> Result<Option<Vec<Word>>> {
        match method.parameters.as_slice() {
            [array_type, value_type] => {
                self.fill_array(method, args, array_type, value_type, false)
            }
            [array_type, from, to, value_type] if from == "I" && to == "I" => {
                self.fill_array(method, args, array_type, value_type, true)
            }
            _ => Ok(None),
        }
    }

    fn fill_array(
        &mut self,
        method: &Method,
        args: &[Word],
        array_type: &str,
        value_type: &str,
        ranged: bool,
    ) -> Result<Option<Vec<Word>>> {
        if !array_type.starts_with('[') {
            return Ok(None);
        }
        let Some(value_width) = wide_value_width(value_type) else {
            return Ok(None);
        };
        let prefix = if ranged { 3 } else { 1 };
        if args.len() != prefix + value_width {
            anyhow::bail!("invalid argument word count for {}", method.key());
        }
        let array = args[0];
        let (element, length) = match &self.heap.get(array)?.data {
            Data::Array { element, values } => (element.clone(), values.len()),
            _ => {
                return Err(fault(
                    "Ljava/lang/ArrayStoreException;",
                    "Arrays.fill requires an array",
                ));
            }
        };
        if !self.is_a(&format!("[{element}"), array_type) {
            return Err(fault(
                "Ljava/lang/ArrayStoreException;",
                format!("array is incompatible with {array_type}"),
            ));
        }
        if is_primitive(&element) {
            if element != value_type {
                return Err(fault(
                    "Ljava/lang/ArrayStoreException;",
                    format!("{value_type} cannot be stored in [{element}"),
                ));
            }
        } else {
            let value = args[prefix];
            if value.reference()? != 0 {
                let class = self.heap.get(value)?.class.clone();
                if !self.is_a(&class, &element) {
                    return Err(fault(
                        "Ljava/lang/ArrayStoreException;",
                        format!("{class} cannot be stored in [{element}"),
                    ));
                }
            }
        }
        let (start, end) = if ranged {
            let start = array_index(args.get(1).copied(), "from index")?;
            let end = array_index(args.get(2).copied(), "to index")?;
            if start > end {
                return Err(fault(
                    "Ljava/lang/IllegalArgumentException;",
                    "from index exceeds to index",
                ));
            }
            if end > length {
                return Err(fault(
                    "Ljava/lang/ArrayIndexOutOfBoundsException;",
                    "Arrays.fill range is out of bounds",
                ));
            }
            (start, end)
        } else {
            (0, length)
        };
        let value = args[prefix..].to_vec();
        let Data::Array { values, .. } = &mut self.heap.get_mut(array)?.data else {
            unreachable!();
        };
        values[start..end].fill(value);
        Ok(Some(vec![]))
    }
}

fn array_index(value: Option<Word>, label: &str) -> Result<usize> {
    let value = value
        .with_context(|| format!("System.arraycopy {label} missing"))?
        .int()?;
    usize::try_from(value).map_err(|_| {
        fault(
            "Ljava/lang/ArrayIndexOutOfBoundsException;",
            format!("negative arraycopy {label}"),
        )
    })
}
fn is_primitive(element: &str) -> bool {
    matches!(element, "Z" | "B" | "C" | "S" | "I" | "J" | "F" | "D")
}

fn wide_value_width(value_type: &str) -> Option<usize> {
    match value_type {
        "J" | "D" => Some(2),
        "Z" | "B" | "C" | "S" | "I" | "F" | "Ljava/lang/Object;" => Some(1),
        value_type if value_type.starts_with(['L', '[']) => Some(1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        Runtime,
        heap::{Object, bits64},
    };
    use droidless_formats::{apk::Apk, dex::Method};
    use std::collections::BTreeMap;

    fn runtime() -> Runtime {
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap())
            .unwrap()
    }
    fn call(vm: &mut Runtime, name: &str, parameters: &[&str], args: &[Word]) -> Vec<Word> {
        vm.invoke(
            Method {
                class: "Ljava/lang/System;".into(),
                name: name.into(),
                parameters: parameters.iter().map(|value| (*value).into()).collect(),
                returns: match name {
                    "arraycopy" => "V",
                    "identityHashCode" => "I",
                    _ => "J",
                }
                .into(),
            },
            args.to_vec(),
            false,
        )
        .unwrap()
    }
    fn array(vm: &mut Runtime, element: &str, values: Vec<i32>) -> Word {
        vm.heap
            .alloc(Object {
                class: format!("[{element}"),
                fields: BTreeMap::new(),
                data: Data::Array {
                    element: element.into(),
                    values: values
                        .into_iter()
                        .map(|value| vec![Word::from(value)])
                        .collect(),
                },
                view: None,
            })
            .unwrap()
    }

    #[test]
    fn virtual_clock_identity_hash_and_overlapping_arraycopy() {
        let mut vm = runtime();
        let now = call(&mut vm, "currentTimeMillis", &[], &[]);
        assert!(bits64(&now).unwrap() as i64 > 1_700_000_000_000);
        let first = bits64(&call(&mut vm, "nanoTime", &[], &[])).unwrap();
        let second = bits64(&call(&mut vm, "nanoTime", &[], &[])).unwrap();
        assert!(second >= first);
        let object = vm.heap.instance("Ljava/lang/Object;").unwrap();
        let hash = call(
            &mut vm,
            "identityHashCode",
            &["Ljava/lang/Object;"],
            &[object],
        )[0];
        assert_eq!(
            call(
                &mut vm,
                "identityHashCode",
                &["Ljava/lang/Object;"],
                &[object],
            )[0],
            hash
        );
        let values = array(&mut vm, "I", vec![1, 2, 3, 4, 5]);
        call(
            &mut vm,
            "arraycopy",
            &["Ljava/lang/Object;", "I", "Ljava/lang/Object;", "I", "I"],
            &[values, Word::ZERO, values, Word::from(1), Word::from(4)],
        );
        let Data::Array { values: copied, .. } = &vm.heap.get(values).unwrap().data else {
            unreachable!();
        };
        assert_eq!(
            copied
                .iter()
                .map(|entry| entry[0].int().unwrap())
                .collect::<Vec<_>>(),
            [1, 1, 2, 3, 4]
        );
    }

    #[test]
    fn array_clone_copies_storage_and_preserves_element_references() {
        let mut vm = runtime();
        let first = vm.heap.instance("Ljava/lang/Object;").unwrap();
        let second = vm.heap.instance("Ljava/lang/Object;").unwrap();
        let source = vm.array("Ljava/lang/Object;".into(), 2).unwrap();
        let Data::Array { values, .. } = &mut vm.heap.get_mut(source).unwrap().data else {
            unreachable!();
        };
        values[0] = vec![first];
        values[1] = vec![second];

        let clone = vm
            .invoke(
                Method {
                    class: "[Ljava/lang/Object;".into(),
                    name: "clone".into(),
                    parameters: vec![],
                    returns: "Ljava/lang/Object;".into(),
                },
                vec![source],
                true,
            )
            .unwrap()[0];
        assert_eq!(vm.heap.get(clone).unwrap().class, "[Ljava/lang/Object;");
        let Data::Array { values, .. } = &vm.heap.get(clone).unwrap().data else {
            unreachable!();
        };
        assert_eq!(values, &[vec![first], vec![second]]);

        let Data::Array { values, .. } = &mut vm.heap.get_mut(source).unwrap().data else {
            unreachable!();
        };
        values[0] = vec![second];
        let Data::Array { values, .. } = &vm.heap.get(clone).unwrap().data else {
            unreachable!();
        };
        assert_eq!(values[0], vec![first]);
        assert_eq!(values[1], vec![second]);
    }

    #[test]
    fn arrays_fill_handles_whole_primitive_and_ranged_reference_arrays() {
        let mut vm = runtime();
        let numbers = array(&mut vm, "I", vec![1, 2, 3]);
        vm.invoke(
            Method {
                class: "Ljava/util/Arrays;".into(),
                name: "fill".into(),
                parameters: vec!["[I".into(), "I".into()],
                returns: "V".into(),
            },
            vec![numbers, Word::from(7)],
            false,
        )
        .unwrap();
        let Data::Array { values, .. } = &vm.heap.get(numbers).unwrap().data else {
            unreachable!();
        };
        assert_eq!(values, &vec![vec![Word::from(7)]; 3]);

        let strings = vm.array("Ljava/lang/String;".into(), 3).unwrap();
        let word = vm.heap.string("same".into()).unwrap();
        vm.invoke(
            Method {
                class: "Ljava/util/Arrays;".into(),
                name: "fill".into(),
                parameters: vec![
                    "[Ljava/lang/Object;".into(),
                    "I".into(),
                    "I".into(),
                    "Ljava/lang/Object;".into(),
                ],
                returns: "V".into(),
            },
            vec![strings, Word::from(1), Word::from(3), word],
            false,
        )
        .unwrap();
        let Data::Array { values, .. } = &vm.heap.get(strings).unwrap().data else {
            unreachable!();
        };
        assert_eq!(values, &[vec![Word::ZERO], vec![word], vec![word]]);
    }
}
