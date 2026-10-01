use crate::{
    heap::{Data, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

impl Runtime {
    pub(crate) fn enum_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class != "Ljava/lang/Enum;" {
            return Ok(None);
        }
        let argument = |index: usize| {
            args.get(index)
                .copied()
                .with_context(|| format!("{} argument missing", method.key()))
        };
        match method.signature().as_str() {
            "<init>(Ljava/lang/String;I)V" => {
                let object = argument(0)?;
                self.heap
                    .get_mut(object)?
                    .fields
                    .insert("droidless:enum:name".into(), vec![argument(1)?]);
                self.heap
                    .get_mut(object)?
                    .fields
                    .insert("droidless:enum:ordinal".into(), vec![argument(2)?]);
                Ok(Some(vec![]))
            }
            "name()Ljava/lang/String;" | "toString()Ljava/lang/String;" => {
                let object = self.heap.get(argument(0)?)?;
                let name = object
                    .fields
                    .get("droidless:enum:name")
                    .and_then(|value| value.first())
                    .copied()
                    .context("uninitialized Enum name")?;
                Ok(Some(vec![name]))
            }
            "ordinal()I" => {
                let object = self.heap.get(argument(0)?)?;
                let ordinal = object
                    .fields
                    .get("droidless:enum:ordinal")
                    .and_then(|value| value.first())
                    .copied()
                    .context("uninitialized Enum ordinal")?;
                Ok(Some(vec![ordinal]))
            }
            "equals(Ljava/lang/Object;)Z" => Ok(Some(vec![Word::from(i32::from(
                argument(0)? == argument(1)?,
            ))])),
            "hashCode()I" => Ok(Some(vec![Word::from(
                argument(0)?.reference()? as u32 as i32
            )])),
            "compareTo(Ljava/lang/Enum;)I" => {
                let left = argument(0)?;
                let right = argument(1)?;
                ensure!(
                    self.heap.get(left)?.class == self.heap.get(right)?.class,
                    fault(
                        "Ljava/lang/ClassCastException;",
                        "cannot compare values from different enum classes",
                    )
                );
                let ordinal = |runtime: &Runtime, value: Word| -> Result<i32> {
                    runtime
                        .heap
                        .get(value)?
                        .fields
                        .get("droidless:enum:ordinal")
                        .and_then(|value| value.first())
                        .copied()
                        .context("uninitialized Enum ordinal")?
                        .int()
                };
                Ok(Some(vec![Word::from(
                    ordinal(self, left)?.cmp(&ordinal(self, right)?) as i32,
                )]))
            }
            "valueOf(Ljava/lang/Class;Ljava/lang/String;)Ljava/lang/Enum;" => {
                let class = argument(0)?;
                let name = argument(1)?;
                let class_name = self
                    .heap
                    .get(class)?
                    .fields
                    .get("name")
                    .and_then(|value| value.first())
                    .copied()
                    .context("invalid enum Class")?;
                let class_name = self.heap.text(class_name)?.to_owned();
                let (dex, definition) = self
                    .class_location(&class_name)
                    .context("Enum.valueOf class is not defined by the APK")?;
                ensure!(
                    self.apk.dex[dex].classes[definition].access & 0x4000 != 0,
                    fault(
                        "Ljava/lang/IllegalArgumentException;",
                        format!("{class_name} is not an enum class"),
                    )
                );
                self.initialize(&class_name)?;
                let values = self.invoke(
                    Method {
                        class: class_name.clone(),
                        name: "values".into(),
                        parameters: vec![],
                        returns: format!("[{class_name}"),
                    },
                    vec![],
                    false,
                )?;
                let array = *values.first().context("enum values returned no array")?;
                let Data::Array { values, .. } = &self.heap.get(array)?.data else {
                    anyhow::bail!("enum values() did not return an array")
                };
                for words in values {
                    let Some(value) = words.first().copied() else {
                        continue;
                    };
                    let enum_name = self
                        .heap
                        .get(value)?
                        .fields
                        .get("droidless:enum:name")
                        .and_then(|value| value.first())
                        .copied();
                    if enum_name.is_some_and(|enum_name| {
                        self.heap.text(enum_name).is_ok_and(|value| {
                            self.heap.text(name).is_ok_and(|wanted| value == wanted)
                        })
                    }) {
                        return Ok(Some(vec![value]));
                    }
                }
                Err(fault(
                    "Ljava/lang/IllegalArgumentException;",
                    format!("No enum constant {}.{}", class_name, self.heap.text(name)?),
                ))
            }
            _ => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Runtime;
    use droidless_formats::{apk::Apk, dex::Method};

    #[test]
    fn graphics_enum_constants_are_canonical_and_dispatch_through_enum() {
        use droidless_formats::dex::Field;
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap(),
        )
        .unwrap();
        for (class, name, expected) in [
            ("Landroid/graphics/Paint$Cap;", "BUTT", 0),
            ("Landroid/graphics/Paint$Join;", "BEVEL", 2),
            ("Landroid/graphics/Paint$Style;", "STROKE", 1),
            ("Landroid/graphics/Path$FillType;", "EVEN_ODD", 1),
            ("Landroid/graphics/PorterDuff$Mode;", "SRC_IN", 5),
            ("Landroid/widget/ImageView$ScaleType;", "CENTER_CROP", 6),
        ] {
            let field = Field {
                class: class.into(),
                name: name.into(),
                ty: class.into(),
            };
            assert_eq!(vm.resolve_field(&field, true).unwrap().key(), field.key());
            assert!(vm.resolve_field(&field, false).is_err());
            let value = vm.graphics_enum_object(&field).unwrap();
            assert_eq!(value, vm.graphics_enum_object(&field).unwrap());
            assert!(vm.is_a(class, "Ljava/lang/Enum;"));
            let ordinal = vm
                .invoke(
                    Method {
                        class: "Ljava/lang/Enum;".into(),
                        name: "ordinal".into(),
                        parameters: vec![],
                        returns: "I".into(),
                    },
                    vec![value],
                    true,
                )
                .unwrap();
            assert_eq!(ordinal, vec![Word::from(expected)]);
            let invalid = Field {
                name: "INVALID".into(),
                ..field
            };
            assert!(
                format!("{:#}", vm.resolve_field(&invalid, true).unwrap_err())
                    .contains("NoSuchFieldError")
            );
        }
    }

    #[test]
    fn enum_construction_retains_name_ordinal_and_identity() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap(),
        )
        .unwrap();
        let value = vm.heap.instance("Lexample/State;").unwrap();
        let name = vm.heap.string("READY".into()).unwrap();
        vm.invoke(
            Method {
                class: "Ljava/lang/Enum;".into(),
                name: "<init>".into(),
                parameters: vec!["Ljava/lang/String;".into(), "I".into()],
                returns: "V".into(),
            },
            vec![value, name, Word::from(2)],
            false,
        )
        .unwrap();
        for (method, returns) in [
            ("name", "Ljava/lang/String;"),
            ("toString", "Ljava/lang/String;"),
        ] {
            let result = vm
                .invoke(
                    Method {
                        class: "Ljava/lang/Enum;".into(),
                        name: method.into(),
                        parameters: vec![],
                        returns: returns.into(),
                    },
                    vec![value],
                    false,
                )
                .unwrap();
            assert_eq!(vm.heap.text(result[0]).unwrap(), "READY");
        }
        assert_eq!(
            vm.invoke(
                Method {
                    class: "Ljava/lang/Enum;".into(),
                    name: "ordinal".into(),
                    parameters: vec![],
                    returns: "I".into(),
                },
                vec![value],
                false,
            )
            .unwrap()[0]
                .int()
                .unwrap(),
            2
        );
    }
}
