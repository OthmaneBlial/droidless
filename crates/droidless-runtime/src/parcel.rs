//! Bounded API-21 Parcel subset. No Binder objects, descriptors, or Java serialization.
use crate::{
    heap::{Data, Word, bits64, wide},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::{Field, Method};

const LIMIT: usize = 4 * 1024 * 1024;
const BUNDLE_MAGIC: i32 = 0x4c444e42;

impl Runtime {
    pub(crate) fn apk_class_loader(&mut self) -> Result<Word> {
        let key = "droidless:apk-class-loader";
        if let Some(value) = self.statics.get(key) {
            return Ok(value[0]);
        }
        let value = self.heap.instance("Ljava/lang/ClassLoader;")?;
        self.statics.insert(key.into(), vec![value]);
        Ok(value)
    }
    fn parcel_loader(&self, loader: Word) -> Result<()> {
        ensure!(
            loader == Word::ZERO
                || self
                    .statics
                    .get("droidless:apk-class-loader")
                    .is_some_and(|value| value.first() == Some(&loader)),
            "custom Parcel ClassLoader unsupported"
        );
        Ok(())
    }
    fn new_parcel(&mut self) -> Result<Word> {
        let parcel = self.heap.instance("Landroid/os/Parcel;")?;
        self.heap.get_mut(parcel)?.data = Data::Parcel {
            bytes: vec![],
            position: 0,
            depth: 0,
            read_limit: None,
            recycled: false,
        };
        Ok(parcel)
    }
    fn parcel_position(&self, parcel: Word) -> Result<usize> {
        let Data::Parcel {
            position,
            recycled: false,
            ..
        } = self.heap.get(parcel)?.data
        else {
            bail!("uninitialized or recycled Parcel");
        };
        Ok(position)
    }
    fn parcel_seek(&mut self, parcel: Word, target: usize) -> Result<()> {
        let Data::Parcel {
            bytes,
            position,
            read_limit,
            recycled: false,
            ..
        } = &mut self.heap.get_mut(parcel)?.data
        else {
            bail!("uninitialized or recycled Parcel");
        };
        ensure!(
            target <= read_limit.unwrap_or(bytes.len()),
            "Parcel position outside data"
        );
        *position = target;
        Ok(())
    }
    fn parcel_write(&mut self, parcel: Word, value: &[u8]) -> Result<()> {
        let Data::Parcel {
            bytes,
            position,
            read_limit,
            recycled: false,
            ..
        } = &mut self.heap.get_mut(parcel)?.data
        else {
            bail!("uninitialized or recycled Parcel");
        };
        ensure!(
            read_limit.is_none(),
            "writing while reconstructing a Bundle is unsupported"
        );
        let end = position
            .checked_add(value.len())
            .context("Parcel size overflow")?;
        ensure!(end <= LIMIT, "Parcel exceeds 4 MiB limit");
        bytes.resize(bytes.len().max(end), 0);
        bytes[*position..end].copy_from_slice(value);
        *position = end;
        Ok(())
    }
    fn parcel_read(&mut self, parcel: Word, size: usize) -> Result<Vec<u8>> {
        let Data::Parcel {
            bytes,
            position,
            read_limit,
            recycled: false,
            ..
        } = &mut self.heap.get_mut(parcel)?.data
        else {
            bail!("uninitialized or recycled Parcel");
        };
        let end = position.checked_add(size).context("Parcel size overflow")?;
        ensure!(
            end <= read_limit.unwrap_or(bytes.len()) && end <= bytes.len(),
            "truncated Parcel"
        );
        let value = bytes[*position..end].to_vec();
        *position = end;
        Ok(value)
    }
    fn parcel_int(&mut self, parcel: Word, value: i32) -> Result<()> {
        self.parcel_write(parcel, &value.to_le_bytes())
    }
    fn parcel_read_int(&mut self, parcel: Word) -> Result<i32> {
        Ok(i32::from_le_bytes(
            self.parcel_read(parcel, 4)?.try_into().unwrap(),
        ))
    }
    fn parcel_string(&mut self, parcel: Word, value: Option<&str>) -> Result<()> {
        let Some(value) = value else {
            return self.parcel_int(parcel, -1);
        };
        let length = value.encode_utf16().count();
        ensure!(length <= (LIMIT - 8) / 2, "Parcel string exceeds limit");
        let mut bytes = Vec::with_capacity((length * 2 + 5) & !3);
        bytes.extend_from_slice(&(length as i32).to_le_bytes());
        for unit in value.encode_utf16() {
            bytes.extend_from_slice(&unit.to_le_bytes());
        }
        bytes.extend_from_slice(&[0, 0]);
        bytes.resize((bytes.len() + 3) & !3, 0);
        self.parcel_write(parcel, &bytes)
    }
    fn parcel_read_string(&mut self, parcel: Word) -> Result<Option<String>> {
        let length = self.parcel_read_int(parcel)?;
        if length == -1 {
            return Ok(None);
        }
        ensure!(
            length >= 0 && length as usize <= (LIMIT - 8) / 2,
            "invalid Parcel string length"
        );
        let length = length as usize;
        let bytes = self.parcel_read(parcel, ((length + 1) * 2 + 3) & !3)?;
        ensure!(
            bytes[length * 2..length * 2 + 2] == [0, 0],
            "Parcel string missing terminator"
        );
        let units = bytes[..length * 2]
            .chunks_exact(2)
            .map(|unit| u16::from_le_bytes([unit[0], unit[1]]))
            .collect::<Vec<_>>();
        Ok(Some(
            String::from_utf16(&units).context("unpaired UTF-16 in Parcel is unsupported")?,
        ))
    }
    // Root snapshots across guest writers/CREATOR callbacks, including mutation and System.gc().
    fn parcel_scope<T>(
        &mut self,
        parcel: Word,
        roots: &[Word],
        f: impl FnOnce(&mut Self) -> Result<T>,
    ) -> Result<T> {
        let Data::Parcel {
            depth,
            recycled: false,
            ..
        } = &mut self.heap.get_mut(parcel)?.data
        else {
            bail!("uninitialized or recycled Parcel");
        };
        ensure!(*depth < 32, "Parcel nesting limit reached");
        *depth += 1;
        let base = self.native_roots.len();
        self.native_roots.push(parcel);
        self.native_roots.extend_from_slice(roots);
        let result = f(self);
        self.native_roots.truncate(base);
        if let Data::Parcel { depth, .. } = &mut self.heap.get_mut(parcel)?.data {
            *depth -= 1;
        }
        result
    }
    fn parcel_write_object(&mut self, parcel: Word, value: Word, flags: i32) -> Result<()> {
        if value == Word::ZERO {
            return self.parcel_string(parcel, None);
        }
        let class = self.heap.get(value)?.class.clone();
        ensure!(
            self.is_a(&class, "Landroid/os/Parcelable;"),
            "Parcel value is not Parcelable"
        );
        self.parcel_scope(parcel, &[value], |vm| {
            let name = class
                .trim_start_matches('L')
                .trim_end_matches(';')
                .replace('/', ".");
            vm.parcel_string(parcel, Some(&name))?;
            vm.invoke(
                Method {
                    class: "Landroid/os/Parcelable;".into(),
                    name: "writeToParcel".into(),
                    parameters: vec!["Landroid/os/Parcel;".into(), "I".into()],
                    returns: "V".into(),
                },
                vec![value, parcel, Word::from(flags)],
                true,
            )?;
            Ok(())
        })
    }
    fn parcel_read_object(&mut self, parcel: Word, loader: Word) -> Result<Word> {
        self.parcel_loader(loader)?;
        let Some(name) = self.parcel_read_string(parcel)? else {
            return Ok(Word::ZERO);
        };
        ensure!(
            !name.is_empty() && !name.contains([';', '[', '/']),
            "invalid Parcelable class name"
        );
        let class = format!("L{};", name.replace('.', "/"));
        ensure!(
            self.is_a(&class, "Landroid/os/Parcelable;"),
            "unknown/non-Parcelable class {name}"
        );
        self.parcel_scope(parcel, &[loader], |vm| {
            if class == "Landroid/net/Uri;" {
                ensure!(
                    vm.parcel_read_int(parcel)? == 1,
                    "unsupported parcelled Uri representation"
                );
                let value = vm
                    .parcel_read_string(parcel)?
                    .context("null Uri representation")?;
                let text = vm.heap.string(value)?;
                return Ok(vm.invoke(
                    Method {
                        class,
                        name: "parse".into(),
                        parameters: vec!["Ljava/lang/String;".into()],
                        returns: "Landroid/net/Uri;".into(),
                    },
                    vec![text],
                    false,
                )?[0]);
            }
            if class == "Landroid/os/Bundle;" {
                return vm.parcel_read_bundle(parcel, loader);
            }
            ensure!(
                vm.class_location(&class).is_some(),
                "framework Parcelable unsupported: {name}"
            );
            let field = vm.resolve_field(
                &Field {
                    class,
                    name: "CREATOR".into(),
                    ty: "Landroid/os/Parcelable$Creator;".into(),
                },
                true,
            )?;
            vm.initialize(&field.class)?;
            let creator = *vm
                .statics
                .get(&field.key())
                .and_then(|words| words.first())
                .context("Parcelable CREATOR missing")?;
            ensure!(
                creator != Word::ZERO
                    && vm.is_a(
                        &vm.heap.get(creator)?.class,
                        "Landroid/os/Parcelable$Creator;"
                    ),
                "invalid Parcelable CREATOR"
            );
            let with_loader = vm.is_a(
                &vm.heap.get(creator)?.class,
                "Landroid/os/Parcelable$ClassLoaderCreator;",
            );
            let mut args = vec![creator, parcel];
            let mut parameters = vec!["Landroid/os/Parcel;".into()];
            if with_loader {
                args.push(loader);
                parameters.push("Ljava/lang/ClassLoader;".into());
            }
            let value = vm.invoke(
                Method {
                    class: if with_loader {
                        "Landroid/os/Parcelable$ClassLoaderCreator;"
                    } else {
                        "Landroid/os/Parcelable$Creator;"
                    }
                    .into(),
                    name: "createFromParcel".into(),
                    parameters,
                    returns: "Ljava/lang/Object;".into(),
                },
                args,
                true,
            )?[0];
            ensure!(
                value == Word::ZERO
                    || vm.is_a(&vm.heap.get(value)?.class, "Landroid/os/Parcelable;"),
                "CREATOR returned non-Parcelable"
            );
            Ok(value)
        })
    }
    fn parcel_write_reference(&mut self, parcel: Word, value: Word) -> Result<()> {
        if value == Word::ZERO {
            return self.parcel_int(parcel, -1);
        }
        let class = self.heap.get(value)?.class.clone();
        match class.as_str() {
            "Ljava/lang/String;" => {
                let text = self.heap.text(value)?.to_owned();
                self.parcel_int(parcel, 0)?;
                self.parcel_string(parcel, Some(&text))
            }
            "Landroid/os/Bundle;" => {
                self.parcel_int(parcel, 3)?;
                self.parcel_write_bundle(parcel, value, 0)
            }
            "Ljava/util/ArrayList;" => {
                let Data::Collection { values, .. } = &self.heap.get(value)?.data else {
                    bail!("uninitialized parcelled ArrayList");
                };
                let values = values.clone();
                ensure!(values.len() <= 16_384, "Parcel list limit reached");
                self.parcel_scope(parcel, &values, |vm| {
                    vm.parcel_int(parcel, 11)?;
                    vm.parcel_int(parcel, values.len() as i32)?;
                    for value in &values {
                        vm.parcel_write_reference(parcel, *value)?;
                    }
                    Ok(())
                })
            }
            _ if self.is_a(&class, "Landroid/os/Parcelable;") => {
                self.parcel_int(parcel, 4)?;
                self.parcel_write_object(parcel, value, 0)
            }
            _ => {
                bail!("unsupported parcelled reference {class}; Java serialization is unavailable")
            }
        }
    }
    fn parcel_write_bundle(&mut self, parcel: Word, bundle: Word, flags: i32) -> Result<()> {
        if bundle == Word::ZERO {
            return self.parcel_int(parcel, -1);
        }
        let Data::Bundle(values) = &self.heap.get(bundle)?.data else {
            bail!("expected Bundle");
        };
        let values = values.clone();
        let roots = values
            .values()
            .flat_map(|(_, words)| words)
            .copied()
            .chain([bundle])
            .collect::<Vec<_>>();
        self.parcel_scope(parcel, &roots, |vm| {
            ensure!(flags == 0, "Bundle Parcel flags unsupported");
            if values.is_empty() {
                return vm.parcel_int(parcel, 0);
            }
            let length_pos = vm.parcel_position(parcel)?;
            vm.parcel_int(parcel, 0)?;
            vm.parcel_int(parcel, BUNDLE_MAGIC)?;
            let start = vm.parcel_position(parcel)?;
            vm.parcel_int(parcel, values.len() as i32)?;
            for (key, (ty, words)) in values {
                vm.parcel_string(parcel, Some(&key))?;
                let tag = match ty.as_str() {
                    "I" => 1,
                    "J" => 6,
                    "F" => 7,
                    "D" => 8,
                    "Z" => 9,
                    _ => -1,
                };
                if tag == -1 {
                    ensure!(
                        ty.starts_with(['L', '[']) && words.len() == 1,
                        "unsupported Bundle Parcel type {ty}"
                    );
                    vm.parcel_write_reference(parcel, words[0])?;
                } else {
                    vm.parcel_int(parcel, tag)?;
                    for word in words {
                        vm.parcel_int(parcel, word.int()?)?;
                    }
                }
            }
            let end = vm.parcel_position(parcel)?;
            ensure!(end >= start, "Bundle writer moved before its payload");
            vm.parcel_seek(parcel, length_pos)?;
            vm.parcel_int(parcel, (end - start) as i32)?;
            vm.parcel_seek(parcel, end)
        })
    }
    fn parcel_read_value(&mut self, parcel: Word, loader: Word) -> Result<(String, Vec<Word>)> {
        let tag = self.parcel_read_int(parcel)?;
        let (ty, value) = match tag {
            -1 => ("Ljava/lang/Object;", Word::ZERO),
            0 => {
                let value = self.parcel_read_string(parcel)?;
                (
                    "Ljava/lang/String;",
                    if let Some(value) = value {
                        self.heap.string(value)?
                    } else {
                        Word::ZERO
                    },
                )
            }
            1 | 7 | 9 => {
                return Ok((
                    match tag {
                        1 => "I",
                        7 => "F",
                        _ => "Z",
                    }
                    .into(),
                    vec![Word::from(self.parcel_read_int(parcel)?)],
                ));
            }
            6 | 8 => {
                return Ok((
                    if tag == 6 { "J" } else { "D" }.into(),
                    vec![
                        Word::from(self.parcel_read_int(parcel)?),
                        Word::from(self.parcel_read_int(parcel)?),
                    ],
                ));
            }
            3 => (
                "Landroid/os/Bundle;",
                self.parcel_read_bundle(parcel, loader)?,
            ),
            4 => (
                "Landroid/os/Parcelable;",
                self.parcel_read_object(parcel, loader)?,
            ),
            11 => {
                let count = self.parcel_read_int(parcel)?;
                ensure!((0..=16_384).contains(&count), "invalid Parcel list length");
                let list = self.heap.instance("Ljava/util/ArrayList;")?;
                self.heap.get_mut(list)?.data = Data::Collection {
                    values: vec![],
                    version: 0,
                };
                self.parcel_scope(parcel, &[list], |vm| {
                    for _ in 0..count {
                        let (ty, words) = vm.parcel_read_value(parcel, loader)?;
                        ensure!(
                            ty.starts_with(['L', '[']),
                            "primitive Parcel list entries unsupported"
                        );
                        let Data::Collection { values, .. } = &mut vm.heap.get_mut(list)?.data
                        else {
                            bail!("invalid Parcel list");
                        };
                        values.push(words[0]);
                    }
                    Ok(())
                })?;
                ("Ljava/util/ArrayList;", list)
            }
            _ => bail!("unsupported Parcel value tag {tag}"),
        };
        Ok((ty.into(), vec![value]))
    }
    fn parcel_read_bundle(&mut self, parcel: Word, loader: Word) -> Result<Word> {
        self.parcel_loader(loader)?;
        let length = self.parcel_read_int(parcel)?;
        if length == -1 {
            return Ok(Word::ZERO);
        }
        ensure!(
            length >= 0 && length as usize <= LIMIT,
            "invalid Bundle Parcel length"
        );
        let bundle = self.new_bundle()?;
        if length == 0 {
            return Ok(bundle);
        }
        ensure!(
            self.parcel_read_int(parcel)? == BUNDLE_MAGIC,
            "invalid Bundle Parcel magic"
        );
        let start = self.parcel_position(parcel)?;
        let end = start
            .checked_add(length as usize)
            .context("Bundle Parcel length overflow")?;
        let Data::Parcel {
            bytes, read_limit, ..
        } = &mut self.heap.get_mut(parcel)?.data
        else {
            bail!("expected Parcel");
        };
        ensure!(
            end <= read_limit.unwrap_or(bytes.len()),
            "truncated Bundle Parcel"
        );
        let previous = read_limit.replace(end);
        let result = self.parcel_scope(parcel, &[bundle, loader], |vm| {
            let count = vm.parcel_read_int(parcel)?;
            ensure!(
                (0..=16_384).contains(&count),
                "invalid Bundle Parcel entry count"
            );
            for _ in 0..count {
                let key = vm
                    .parcel_read_string(parcel)?
                    .context("null Bundle Parcel keys unsupported")?;
                let (ty, words) = vm.parcel_read_value(parcel, loader)?;
                vm.bundle_put(bundle, key, ty, words)?;
            }
            ensure!(
                vm.parcel_position(parcel)? == end,
                "Bundle Parcel length mismatch"
            );
            Ok(bundle)
        });
        if let Data::Parcel { read_limit, .. } = &mut self.heap.get_mut(parcel)?.data {
            *read_limit = previous;
        }
        result
    }
    pub(crate) fn snapshot_intent(&mut self, source: Word) -> Result<Word> {
        let copy = self.copy_intent(source)?;
        let extras = self
            .heap
            .get(copy)?
            .fields
            .get("extras")
            .and_then(|v| v.first())
            .copied()
            .unwrap_or(Word::ZERO);
        if extras == Word::ZERO {
            return Ok(copy);
        }
        let parcel = self.new_parcel()?;
        let result = self.parcel_scope(parcel, &[source, copy], |vm| {
            vm.parcel_write_bundle(parcel, extras, 0)?;
            vm.parcel_seek(parcel, 0)?;
            let loader = vm.apk_class_loader()?;
            let extras = vm.parcel_read_bundle(parcel, loader)?;
            vm.heap
                .get_mut(copy)?
                .fields
                .insert("extras".into(), vec![extras]);
            Ok(copy)
        });
        self.heap.get_mut(parcel)?.data = Data::Instance;
        result
    }
    pub(crate) fn parcel_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let arg = |n| args.get(n).copied().context("Parcel argument missing");
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let mut result = vec![];
        match (method.class.as_str(), method.signature().as_str()) {
            ("Landroid/os/Parcel;", "obtain()Landroid/os/Parcel;") => {
                result.push(self.new_parcel()?)
            }
            ("Landroid/os/Parcel;", "recycle()V") => {
                let Data::Parcel {
                    depth,
                    recycled,
                    bytes,
                    position,
                    ..
                } = &mut self.heap.get_mut(receiver)?.data
                else {
                    bail!("expected Parcel");
                };
                ensure!(
                    *depth == 0 && !*recycled,
                    "Parcel cannot be recycled during a callback or twice"
                );
                *recycled = true;
                bytes.clear();
                *position = 0;
            }
            ("Landroid/os/Parcel;", "dataPosition()I") => {
                result.push(Word::from(self.parcel_position(receiver)? as i32))
            }
            ("Landroid/os/Parcel;", "dataSize()I" | "dataAvail()I") => {
                self.parcel_position(receiver)?;
                let Data::Parcel {
                    bytes,
                    position,
                    read_limit,
                    ..
                } = &self.heap.get(receiver)?.data
                else {
                    bail!("expected Parcel");
                };
                result.push(Word::from(if method.name == "dataSize" {
                    bytes.len()
                } else {
                    read_limit.unwrap_or(bytes.len()).saturating_sub(*position)
                } as i32));
            }
            ("Landroid/os/Parcel;", "setDataPosition(I)V") => {
                let position = arg(1)?.int()?;
                ensure!(position >= 0, "negative Parcel position");
                self.parcel_seek(receiver, position as usize)?;
            }
            ("Landroid/os/Parcel;", "writeInt(I)V" | "writeFloat(F)V") => {
                self.parcel_int(receiver, arg(1)?.int()?)?
            }
            ("Landroid/os/Parcel;", "readInt()I" | "readFloat()F") => {
                result.push(Word::from(self.parcel_read_int(receiver)?))
            }
            ("Landroid/os/Parcel;", "writeLong(J)V" | "writeDouble(D)V") => {
                self.parcel_write(receiver, &bits64(&args[1..])?.to_le_bytes())?
            }
            ("Landroid/os/Parcel;", "readLong()J" | "readDouble()D") => {
                result = wide(u64::from_le_bytes(
                    self.parcel_read(receiver, 8)?.try_into().unwrap(),
                ))
            }
            ("Landroid/os/Parcel;", "writeString(Ljava/lang/String;)V") => {
                let text = if arg(1)? == Word::ZERO {
                    None
                } else {
                    Some(self.heap.text(arg(1)?)?.to_owned())
                };
                self.parcel_string(receiver, text.as_deref())?;
            }
            ("Landroid/os/Parcel;", "readString()Ljava/lang/String;") => {
                let text = self.parcel_read_string(receiver)?;
                result.push(if let Some(text) = text {
                    self.heap.string(text)?
                } else {
                    Word::ZERO
                });
            }
            ("Landroid/os/Parcel;", "writeParcelable(Landroid/os/Parcelable;I)V") => {
                self.parcel_write_object(receiver, arg(1)?, arg(2)?.int()?)?
            }
            (
                "Landroid/os/Parcel;",
                "readParcelable(Ljava/lang/ClassLoader;)Landroid/os/Parcelable;",
            ) => result.push(self.parcel_read_object(receiver, arg(1)?)?),
            ("Landroid/os/Parcel;", "writeBundle(Landroid/os/Bundle;)V") => {
                self.parcel_write_bundle(receiver, arg(1)?, 0)?
            }
            (
                "Landroid/os/Parcel;",
                "readBundle()Landroid/os/Bundle;"
                | "readBundle(Ljava/lang/ClassLoader;)Landroid/os/Bundle;",
            ) => {
                result.push(self.parcel_read_bundle(
                    receiver,
                    if args.len() > 1 { arg(1)? } else { Word::ZERO },
                )?)
            }
            ("Landroid/os/Bundle;", "writeToParcel(Landroid/os/Parcel;I)V") => {
                self.parcel_write_bundle(arg(1)?, receiver, arg(2)?.int()?)?
            }
            ("Landroid/os/Bundle;" | "Landroid/net/Uri;", "describeContents()I") => {
                result.push(Word::ZERO)
            }
            ("Landroid/net/Uri;", "writeToParcel(Landroid/os/Parcel;I)V") => {
                let text = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:uri:text")
                    .and_then(|v| v.first())
                    .copied()
                    .context("Uri representation missing")?;
                let text = self.heap.text(text)?.to_owned();
                self.parcel_int(arg(1)?, 1)?;
                self.parcel_string(arg(1)?, Some(&text))?;
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
}
