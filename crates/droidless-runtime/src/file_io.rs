use crate::{
    heap::{Data, Word, bits64, fault, wide},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

const EXTERNAL_ROOT: &str = "/storage/emulated/0";
const CHANNEL: &str = "droidless:file:channel";
const CHANNEL_STREAM: &str = "droidless:file:channel-stream";

impl Runtime {
    pub(crate) fn file_io_native(
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
            ("Ljava/io/FileInputStream;", "getChannel()Ljava/nio/channels/FileChannel;") => {
                ensure!(args.len() == 1, "invalid getChannel arguments");
                let object = self.heap.get(receiver)?;
                ensure!(
                    matches!(object.data, Data::ByteStream { .. }),
                    fault(
                        "Ljava/lang/IllegalStateException;",
                        "FileInputStream is not initialized"
                    )
                );
                if let Some(channel) = object.fields.get(CHANNEL).and_then(|v| v.first()) {
                    return Ok(Some(vec![*channel]));
                }
                let channel = self.heap.instance("Ljava/nio/channels/FileChannel;")?;
                self.heap
                    .get_mut(channel)?
                    .fields
                    .insert(CHANNEL_STREAM.into(), vec![receiver]);
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(CHANNEL.into(), vec![channel]);
                Ok(Some(vec![channel]))
            }
            (
                "Ljava/nio/channels/FileChannel;",
                "size()J" | "position()J" | "position(J)Ljava/nio/channels/FileChannel;",
            )
            | (
                "Ljava/nio/channels/FileChannel;"
                | "Ljava/nio/channels/spi/AbstractInterruptibleChannel;"
                | "Ljava/nio/channels/Channel;",
                "isOpen()Z" | "close()V",
            ) => {
                ensure!(
                    args.len() == if method.parameters.is_empty() { 1 } else { 3 },
                    "invalid file channel arguments"
                );
                let stream = *self
                    .heap
                    .get(receiver)?
                    .fields
                    .get(CHANNEL_STREAM)
                    .and_then(|v| v.first())
                    .context("uninitialized file channel")?;
                let Data::ByteStream {
                    bytes,
                    position,
                    closed,
                } = &self.heap.get(stream)?.data
                else {
                    return Err(fault(
                        "Ljava/lang/IllegalStateException;",
                        "file channel has no input stream",
                    ));
                };
                if method.name == "isOpen" {
                    return Ok(Some(vec![Word::from(i32::from(!closed))]));
                }
                if method.name == "close" {
                    if let Data::ByteStream { closed, .. } = &mut self.heap.get_mut(stream)?.data {
                        *closed = true;
                    }
                    return Ok(Some(vec![]));
                }
                ensure!(
                    !closed,
                    fault(
                        "Ljava/nio/channels/ClosedChannelException;",
                        "Channel closed"
                    )
                );
                if signature == "position(J)Ljava/nio/channels/FileChannel;" {
                    let next = bits64(&args[1..])? as i64;
                    ensure!(
                        next >= 0,
                        fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "negative channel position"
                        )
                    );
                    let next = usize::try_from(next)
                        .context("channel position exceeds host index size")?;
                    if let Data::ByteStream { position, .. } = &mut self.heap.get_mut(stream)?.data
                    {
                        *position = next;
                    }
                    Ok(Some(vec![receiver]))
                } else {
                    Ok(Some(wide(if method.name == "size" {
                        bytes.len()
                    } else {
                        *position
                    } as u64)))
                }
            }
            ("Landroid/os/Environment;", "getExternalStorageDirectory()Ljava/io/File;") => {
                ensure!(
                    args.is_empty(),
                    "invalid external storage directory arguments"
                );
                self.ensure_guest_directory(EXTERNAL_ROOT)?;
                ensure!(
                    self.file_path_kind(&["external".into()])? == Some(true),
                    "external storage root is not a directory"
                );
                Ok(Some(vec![self.file_object(EXTERNAL_ROOT.into())?]))
            }
            ("Landroid/content/Context;", "getCacheDir()Ljava/io/File;") => {
                let path = format!("/data/data/{}/cache", self.apk.manifest.package);
                self.ensure_guest_directory(&path)?;
                Ok(Some(vec![self.file_object(path)?]))
            }
            ("Landroid/content/Context;", "getFilesDir()Ljava/io/File;") => {
                let path = format!("/data/data/{}/files", self.apk.manifest.package);
                self.ensure_guest_directory(&path)?;
                Ok(Some(vec![self.file_object(path)?]))
            }
            ("Landroid/content/Context;", "getDatabasePath(Ljava/lang/String;)Ljava/io/File;")
            | (
                "Landroid/content/Context;",
                "getFileStreamPath(Ljava/lang/String;)Ljava/io/File;",
            ) => {
                let directory = if method.name == "getDatabasePath" {
                    "databases"
                } else {
                    "files"
                };
                let path = format!(
                    "/data/data/{}/{directory}/{}",
                    self.apk.manifest.package,
                    self.heap.text(argument(1)?)?
                );
                if method.name == "getDatabasePath" {
                    self.ensure_guest_directory(&format!(
                        "/data/data/{}/databases",
                        self.apk.manifest.package
                    ))?;
                }
                Ok(Some(vec![self.file_object(path)?]))
            }
            ("Landroid/content/Context;", "getDir(Ljava/lang/String;I)Ljava/io/File;") => {
                let name = self.heap.text(argument(1)?)?;
                let path = format!("/data/data/{}/app_{name}", self.apk.manifest.package);
                self.ensure_guest_directory(&path)?;
                Ok(Some(vec![self.file_object(path)?]))
            }
            ("Ljava/io/File;", "<init>(Ljava/lang/String;)V") => {
                let path = self.heap.text(argument(1)?)?.to_owned();
                self.heap.get_mut(receiver)?.data = Data::File(path);
                Ok(Some(vec![]))
            }
            ("Ljava/io/File;", "<init>(Ljava/lang/String;Ljava/lang/String;)V") => {
                let parent = self.heap.text(argument(1)?)?;
                let child = self.heap.text(argument(2)?)?;
                let path = join_guest_path(parent, child);
                self.heap.get_mut(receiver)?.data = Data::File(path);
                Ok(Some(vec![]))
            }
            ("Ljava/io/File;", "<init>(Ljava/io/File;Ljava/lang/String;)V") => {
                let parent = self.file_path(argument(1)?)?.to_owned();
                let child = self.heap.text(argument(2)?)?;
                let path = join_guest_path(&parent, child);
                self.heap.get_mut(receiver)?.data = Data::File(path);
                Ok(Some(vec![]))
            }
            ("Ljava/io/File;", "getPath()Ljava/lang/String;" | "toString()Ljava/lang/String;") => {
                return Ok(Some(vec![
                    self.heap.string(self.file_path(receiver)?.to_owned())?,
                ]));
            }
            ("Ljava/io/File;", "getAbsolutePath()Ljava/lang/String;") => {
                let path = self.file_path(receiver)?;
                let absolute = absolute_guest_path(&self.apk.manifest.package, path);
                Ok(Some(vec![self.heap.string(absolute)?]))
            }
            ("Ljava/io/File;", "isAbsolute()Z") => {
                return Ok(Some(vec![Word::from(i32::from(
                    self.file_path(receiver)?.starts_with('/'),
                ))]));
            }
            ("Ljava/io/File;", "exists()Z" | "isDirectory()Z" | "isFile()Z") => {
                let path = self.file_path(receiver)?.to_owned();
                let kind = self
                    .guest_file_relative(&path)
                    .map(|path| self.file_path_kind(&path))
                    .transpose()?
                    .flatten();
                let result = match method.name.as_str() {
                    "exists" => kind.is_some(),
                    "isDirectory" => kind == Some(true),
                    _ => kind == Some(false),
                };
                Ok(Some(vec![Word::from(i32::from(result))]))
            }
            ("Ljava/io/File;", "canRead()Z" | "canWrite()Z") => {
                let path = self.file_path(receiver)?.to_owned();
                let exists = self
                    .guest_file_relative(&path)
                    .map(|path| self.file_path_kind(&path))
                    .transpose()?
                    .flatten()
                    .is_some();
                Ok(Some(vec![Word::from(i32::from(exists))]))
            }
            ("Ljava/io/File;", "mkdirs()Z") => {
                let path = self.file_path(receiver)?.to_owned();
                let created = self.ensure_guest_directory(&path)?;
                Ok(Some(vec![Word::from(i32::from(created))]))
            }
            ("Ljava/io/File;", "isHidden()Z") => {
                let hidden = self
                    .file_path(receiver)?
                    .trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .is_some_and(|name| name.starts_with('.'));
                Ok(Some(vec![Word::from(i32::from(hidden))]))
            }
            ("Ljava/io/File;", "getName()Ljava/lang/String;") => {
                let name = self
                    .file_path(receiver)?
                    .trim_end_matches('/')
                    .rsplit('/')
                    .next()
                    .unwrap_or("");
                Ok(Some(vec![self.heap.string(name.to_owned())?]))
            }
            ("Ljava/io/File;", "getParentFile()Ljava/io/File;") => {
                let path = self.file_path(receiver)?.trim_end_matches('/');
                let Some(index) = path.rfind('/') else {
                    return Ok(Some(vec![Word::ZERO]));
                };
                let parent = if index == 0 { "/" } else { &path[..index] };
                Ok(Some(vec![self.file_object(parent.to_owned())?]))
            }
            ("Ljava/io/File;", "list()[Ljava/lang/String;") => {
                let path = self.file_path(receiver)?.to_owned();
                let entries = self
                    .guest_file_relative(&path)
                    .and_then(|relative| {
                        if let Some(storage) = &self.storage {
                            storage.app_entries(&relative).transpose()
                        } else {
                            Some(Ok(self
                                .virtual_directories
                                .iter()
                                .filter(|directory| {
                                    directory.len() > relative.len()
                                        && directory.starts_with(&relative)
                                })
                                .map(|directory| directory[relative.len()].clone())
                                .collect::<std::collections::BTreeSet<_>>()
                                .into_iter()
                                .collect()))
                        }
                    })
                    .transpose()?;
                let Some(entries) = entries else {
                    return Ok(Some(vec![Word::ZERO]));
                };
                let array = self.array("Ljava/lang/String;".into(), entries.len())?;
                let names = entries
                    .into_iter()
                    .map(|name| self.heap.string(name).map(|name| vec![name]))
                    .collect::<Result<Vec<_>>>()?;
                let Data::Array { values, .. } = &mut self.heap.get_mut(array)?.data else {
                    unreachable!();
                };
                *values = names;
                Ok(Some(vec![array]))
            }
            ("Ljava/lang/String;", "<init>([CII)V") => {
                ensure!(
                    self.heap.get(receiver)?.class == "Ljava/lang/String;",
                    "invalid String receiver"
                );
                let array = argument(1)?;
                let object = self.heap.get(array)?;
                let Data::Array { element, values } = &object.data else {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "String source must be a char array",
                    ));
                };
                ensure!(
                    element == "C",
                    fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "String source must be a char array",
                    )
                );
                let offset = usize::try_from(argument(2)?.int()?).map_err(|_| {
                    fault(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        "negative char-array offset",
                    )
                })?;
                let length = usize::try_from(argument(3)?.int()?).map_err(|_| {
                    fault(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        "negative char count",
                    )
                })?;
                ensure!(
                    offset <= values.len() && length <= values.len() - offset,
                    fault(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        "char-array range is out of bounds",
                    )
                );
                let units = values[offset..offset + length]
                    .iter()
                    .map(|value| value[0].int().map(|unit| unit as u16))
                    .collect::<Result<Vec<_>>>()?;
                let text = String::from_utf16_lossy(&units);
                ensure!(text.len() <= 1_048_576, "guest string exceeds 1 MiB");
                self.heap.get_mut(receiver)?.data = Data::String(text);
                Ok(Some(vec![]))
            }
            ("Ljava/lang/String;", "<init>([BII)V") => {
                ensure!(
                    self.heap.get(receiver)?.class == "Ljava/lang/String;",
                    "invalid String receiver"
                );
                let array = argument(1)?;
                let (offset, length) = {
                    let object = self.heap.get(array)?;
                    let Data::Array { element, values } = &object.data else {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "String source must be a byte array",
                        ));
                    };
                    ensure!(
                        element == "B",
                        fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "String source must be a byte array",
                        )
                    );
                    let offset = usize::try_from(argument(2)?.int()?).map_err(|_| {
                        fault(
                            "Ljava/lang/IndexOutOfBoundsException;",
                            "negative byte-array offset",
                        )
                    })?;
                    let length = usize::try_from(argument(3)?.int()?).map_err(|_| {
                        fault(
                            "Ljava/lang/IndexOutOfBoundsException;",
                            "negative byte count",
                        )
                    })?;
                    ensure!(
                        offset <= values.len() && length <= values.len() - offset,
                        fault(
                            "Ljava/lang/IndexOutOfBoundsException;",
                            "byte-array range is out of bounds",
                        )
                    );
                    (offset, length)
                };
                let text = match &self.heap.get(array)?.data {
                    Data::Array { values, .. } => values[offset..offset + length]
                        .iter()
                        .map(|value| value[0].int().map(|byte| byte as i8 as u8))
                        .collect::<Result<Vec<_>>>()?,
                    _ => unreachable!(),
                };
                let text = String::from_utf8_lossy(&text).into_owned();
                ensure!(text.len() <= 1_048_576, "guest string exceeds 1 MiB");
                self.heap.get_mut(receiver)?.data = Data::String(text);
                Ok(Some(vec![]))
            }
            ("Ljava/io/FileInputStream;", "<init>(Ljava/lang/String;)V") => {
                let path = self.heap.text(argument(1)?)?.to_owned();
                self.open_virtual_file_input(receiver, path)?;
                Ok(Some(vec![]))
            }
            ("Ljava/io/FileInputStream;", "<init>(Ljava/io/File;)V") => {
                let path = self.file_path(argument(1)?)?.to_owned();
                self.open_virtual_file_input(receiver, path)?;
                Ok(Some(vec![]))
            }
            (
                "Ljava/io/FileInputStream;" | "Ljava/io/InputStream;",
                "read()I" | "read([B)I" | "read([BII)I" | "skip(J)J" | "available()I" | "close()V",
            ) => {
                let is_file_input = self.heap.get(receiver)?.class == "Ljava/io/FileInputStream;";
                ensure!(
                    is_file_input,
                    "only DROIDLESS FileInputStream instances are supported"
                );
                if method.name == "close" {
                    let Data::ByteStream { closed, .. } = &mut self.heap.get_mut(receiver)?.data
                    else {
                        return Err(fault(
                            "Ljava/lang/IllegalStateException;",
                            "FileInputStream is not initialized",
                        ));
                    };
                    *closed = true;
                    return Ok(Some(vec![]));
                }
                let Data::ByteStream {
                    bytes,
                    position,
                    closed,
                } = &self.heap.get(receiver)?.data
                else {
                    return Err(fault(
                        "Ljava/lang/IllegalStateException;",
                        "FileInputStream is not initialized",
                    ));
                };
                ensure!(!closed, fault("Ljava/io/IOException;", "Stream closed"));
                let position = *position;
                let remaining = bytes.len().saturating_sub(position);
                let result = match signature.as_str() {
                    "read()I" => {
                        let value = bytes.get(position).map_or(-1, |byte| i32::from(*byte));
                        if value >= 0
                            && let Data::ByteStream { position, .. } =
                                &mut self.heap.get_mut(receiver)?.data
                        {
                            *position += 1;
                        }
                        vec![Word::from(value)]
                    }
                    "read([B)I" | "read([BII)I" => {
                        let array = argument(1)?;
                        let (element, length) = match &self.heap.get(array)?.data {
                            Data::Array { element, values } => (element.as_str(), values.len()),
                            _ => {
                                return Err(fault(
                                    "Ljava/lang/IllegalArgumentException;",
                                    "read target must be a byte array",
                                ));
                            }
                        };
                        ensure!(
                            element == "B",
                            fault(
                                "Ljava/lang/IllegalArgumentException;",
                                "read target must be a byte array"
                            )
                        );
                        let (offset, requested) = if signature == "read([B)I" {
                            (0usize, length)
                        } else {
                            let offset = usize::try_from(argument(2)?.int()?).map_err(|_| {
                                fault(
                                    "Ljava/lang/IndexOutOfBoundsException;",
                                    "negative byte-array offset",
                                )
                            })?;
                            let requested = usize::try_from(argument(3)?.int()?).map_err(|_| {
                                fault(
                                    "Ljava/lang/IndexOutOfBoundsException;",
                                    "negative byte count",
                                )
                            })?;
                            ensure!(
                                offset <= length && requested <= length - offset,
                                fault(
                                    "Ljava/lang/IndexOutOfBoundsException;",
                                    "byte-array range is out of bounds"
                                )
                            );
                            (offset, requested)
                        };
                        let count = requested.min(remaining);
                        let count = if count == 0 && requested == 0 {
                            0
                        } else if count == 0 {
                            -1
                        } else {
                            let read = bytes[position..position + count].to_vec();
                            let values = match &mut self.heap.get_mut(array)?.data {
                                Data::Array { values, .. } => values,
                                _ => unreachable!(),
                            };
                            for (target, byte) in
                                values[offset..offset + count].iter_mut().zip(&read)
                            {
                                *target = vec![Word::from(i32::from(*byte as i8))];
                            }
                            if let Data::ByteStream { position, .. } =
                                &mut self.heap.get_mut(receiver)?.data
                            {
                                *position += count;
                            }
                            count as i32
                        };
                        vec![Word::from(count)]
                    }
                    "skip(J)J" => {
                        let requested = bits64(&args[1..])? as i64;
                        let count = if requested > 0 {
                            (requested as u64).min(remaining as u64) as usize
                        } else {
                            0
                        };
                        if let Data::ByteStream { position, .. } =
                            &mut self.heap.get_mut(receiver)?.data
                        {
                            *position += count;
                        }
                        return Ok(Some(wide(count as u64)));
                    }
                    "available()I" => vec![Word::from(remaining as i32)],
                    _ => unreachable!(),
                };
                Ok(Some(result))
            }
            _ => Ok(None),
        }
    }

    fn file_object(&mut self, path: String) -> Result<Word> {
        let object = self.heap.instance("Ljava/io/File;")?;
        self.heap.get_mut(object)?.data = Data::File(path);
        Ok(object)
    }

    fn file_path(&self, object: Word) -> Result<&str> {
        match &self.heap.get(object)?.data {
            Data::File(path) => Ok(path),
            _ => anyhow::bail!("File object is not initialized"),
        }
    }

    pub(crate) fn guest_file_relative(&self, path: &str) -> Option<Vec<String>> {
        let absolute = absolute_guest_path(&self.apk.manifest.package, path);
        let mut parts = Vec::<String>::new();
        for part in absolute.split('/') {
            match part {
                "" | "." => {}
                ".." => {
                    parts.pop()?;
                }
                part if part.contains('\0') => return None,
                part => parts.push(part.to_owned()),
            }
        }
        let package = &self.apk.manifest.package;
        if parts.len() >= 3 && parts[0] == "data" && parts[1] == "data" && parts[2] == *package {
            Some(parts[3..].to_vec())
        } else if parts.len() >= 4
            && parts[0] == "data"
            && parts[1] == "user"
            && parts[2] == "0"
            && parts[3] == *package
        {
            Some(parts[4..].to_vec())
        } else if parts.len() >= 3 && parts[..3] == ["storage", "emulated", "0"] {
            // This virtual external volume remains inside the current package capability.
            Some(
                std::iter::once("external".to_owned())
                    .chain(parts[3..].iter().cloned())
                    .collect(),
            )
        } else {
            None
        }
    }

    fn file_path_kind(&self, path: &[String]) -> Result<Option<bool>> {
        if let Some(storage) = &self.storage {
            storage.app_path_kind(path)
        } else {
            Ok(self.virtual_directories.contains(path).then_some(true))
        }
    }

    fn ensure_guest_directory(&mut self, path: &str) -> Result<bool> {
        let Some(relative) = self.guest_file_relative(path) else {
            return Ok(false);
        };
        if self.file_path_kind(&relative)?.is_some() {
            return Ok(false);
        }
        if let Some(storage) = &self.storage {
            storage.ensure_app_dir(&relative)?;
        } else {
            for length in 0..=relative.len() {
                self.virtual_directories.insert(relative[..length].to_vec());
            }
        }
        Ok(true)
    }

    fn open_virtual_file_input(&mut self, receiver: Word, path: String) -> Result<()> {
        let bytes = if path == "/proc/self/cmdline" {
            let mut bytes = self.apk.manifest.package.as_bytes().to_vec();
            bytes.push(0);
            bytes
        } else {
            let missing = || {
                fault(
                    "Ljava/io/FileNotFoundException;",
                    format!("not present in DROIDLESS virtual files: {path}"),
                )
            };
            let relative = self.guest_file_relative(&path).ok_or_else(missing)?;
            let storage = self.storage.as_ref().ok_or_else(missing)?;
            // ponytail: bounded snapshot stream; retain open descriptors if live file changes are required.
            storage
                .read_app_file(&relative)
                .map_err(|error| fault("Ljava/io/FileNotFoundException;", format!("{error:#}")))?
                .ok_or_else(missing)?
        };
        self.heap.get_mut(receiver)?.data = Data::ByteStream {
            bytes,
            position: 0,
            closed: false,
        };
        Ok(())
    }
}

fn join_guest_path(parent: &str, child: &str) -> String {
    if child.starts_with('/') || parent.is_empty() {
        child.to_owned()
    } else {
        format!("{}/{child}", parent.trim_end_matches('/'))
    }
}

fn absolute_guest_path(package: &str, path: &str) -> String {
    if path.starts_with('/') {
        path.to_owned()
    } else {
        format!("/data/data/{package}/files/{path}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Runtime, heap::Object};
    use droidless_formats::apk::Apk;
    use std::collections::BTreeMap;

    fn runtime() -> Runtime {
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap())
            .unwrap()
    }
    fn stream(vm: &mut Runtime, path: &str) -> Word {
        let input = vm.heap.instance("Ljava/io/FileInputStream;").unwrap();
        let path = vm.heap.string(path.to_owned()).unwrap();
        vm.invoke(
            Method {
                class: "Ljava/io/FileInputStream;".into(),
                name: "<init>".into(),
                parameters: vec!["Ljava/lang/String;".into()],
                returns: "V".into(),
            },
            vec![input, path],
            false,
        )
        .unwrap();
        input
    }
    fn bytes(vm: &mut Runtime, length: usize) -> Word {
        vm.heap
            .alloc(Object {
                class: "[B".into(),
                fields: BTreeMap::new(),
                data: Data::Array {
                    element: "B".into(),
                    values: vec![vec![Word::ZERO]; length],
                },
                view: None,
            })
            .unwrap()
    }
    fn call(
        vm: &mut Runtime,
        input: Word,
        name: &str,
        parameters: Vec<&str>,
        extra: &[Word],
    ) -> Vec<Word> {
        vm.invoke(
            Method {
                class: "Ljava/io/FileInputStream;".into(),
                name: name.into(),
                parameters: parameters.into_iter().map(str::to_owned).collect(),
                returns: match name {
                    "read" | "available" => "I",
                    "skip" => "J",
                    _ => "V",
                }
                .into(),
            },
            std::iter::once(input)
                .chain(extra.iter().copied())
                .collect(),
            false,
        )
        .unwrap()
    }

    #[test]
    fn file_input_stream_reads_virtual_process_cmdline_and_rejects_host_paths() {
        let mut vm = runtime();
        let input = stream(&mut vm, "/proc/self/cmdline");
        assert!(vm.is_a("Ljava/io/FileNotFoundException;", "Ljava/io/IOException;"));
        assert_eq!(
            call(&mut vm, input, "available", vec![], &[])[0]
                .int()
                .unwrap(),
            vm.apk.manifest.package.len() as i32 + 1
        );
        let target = bytes(&mut vm, 64);
        let count = call(&mut vm, input, "read", vec!["[B"], &[target])[0]
            .int()
            .unwrap() as usize;
        let read = match &vm.heap.get(target).unwrap().data {
            Data::Array { values, .. } => values[..count]
                .iter()
                .map(|value| value[0].int().unwrap() as i8 as u8)
                .collect::<Vec<_>>(),
            _ => unreachable!(),
        };
        assert_eq!(read, [vm.apk.manifest.package.as_bytes(), &[0]].concat());
        let text = vm.heap.instance("Ljava/lang/String;").unwrap();
        vm.invoke(
            Method {
                class: "Ljava/lang/String;".into(),
                name: "<init>".into(),
                parameters: vec!["[B".into(), "I".into(), "I".into()],
                returns: "V".into(),
            },
            vec![text, target, Word::ZERO, Word::from(count as i32 - 1)],
            false,
        )
        .unwrap();
        assert_eq!(vm.heap.text(text).unwrap(), vm.apk.manifest.package);
        assert_eq!(
            call(&mut vm, input, "read", vec![], &[])[0].int().unwrap(),
            -1
        );
        call(&mut vm, input, "close", vec![], &[]);
        assert!(
            vm.invoke(
                Method {
                    class: "Ljava/io/FileInputStream;".into(),
                    name: "read".into(),
                    parameters: vec![],
                    returns: "I".into(),
                },
                vec![input],
                false,
            )
            .unwrap_err()
            .downcast_ref::<crate::heap::GuestFault>()
            .is_some_and(|error| error.0 == "Ljava/io/IOException;")
        );
        let path = vm.heap.string("/etc/passwd".into()).unwrap();
        let blocked = vm.heap.instance("Ljava/io/FileInputStream;").unwrap();
        assert!(
            vm.invoke(
                Method {
                    class: "Ljava/io/FileInputStream;".into(),
                    name: "<init>".into(),
                    parameters: vec!["Ljava/lang/String;".into()],
                    returns: "V".into(),
                },
                vec![blocked, path],
                false,
            )
            .unwrap_err()
            .downcast_ref::<crate::heap::GuestFault>()
            .is_some_and(|error| error.0 == "Ljava/io/FileNotFoundException;")
        );
    }

    #[test]
    fn file_paths_are_virtual_and_resolve_android_app_directories() {
        let mut vm = runtime();
        let parent = vm.heap.string("/data/data/example/files".into()).unwrap();
        let child = vm.heap.string("notes.db".into()).unwrap();
        let file = vm.heap.instance("Ljava/io/File;").unwrap();
        vm.invoke(
            Method {
                class: "Ljava/io/File;".into(),
                name: "<init>".into(),
                parameters: vec!["Ljava/lang/String;".into(), "Ljava/lang/String;".into()],
                returns: "V".into(),
            },
            vec![file, parent, child],
            false,
        )
        .unwrap();
        assert!(matches!(
            &vm.heap.get(file).unwrap().data,
            Data::File(path) if path == "/data/data/example/files/notes.db"
        ));
        let name = vm
            .invoke(
                Method {
                    class: "Ljava/io/File;".into(),
                    name: "getName".into(),
                    parameters: vec![],
                    returns: "Ljava/lang/String;".into(),
                },
                vec![file],
                false,
            )
            .unwrap()[0];
        assert_eq!(vm.heap.text(name).unwrap(), "notes.db");

        let context = vm.heap.instance("Landroid/content/Context;").unwrap();
        let cache = vm
            .invoke(
                Method {
                    class: "Landroid/content/Context;".into(),
                    name: "getCacheDir".into(),
                    parameters: vec![],
                    returns: "Ljava/io/File;".into(),
                },
                vec![context],
                false,
            )
            .unwrap()[0];
        let expected = format!("/data/data/{}/cache", vm.apk.manifest.package);
        assert!(matches!(
            &vm.heap.get(cache).unwrap().data,
            Data::File(path) if path == &expected
        ));
    }

    #[test]
    fn external_directory_is_virtual_persistent_and_package_isolated() {
        let directory = std::env::temp_dir().join(format!(
            "droidless-external-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir(&directory).unwrap();
        let apk = || Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap();
        let method = Method {
            class: "Landroid/os/Environment;".into(),
            name: "getExternalStorageDirectory".into(),
            parameters: vec![],
            returns: "Ljava/io/File;".into(),
        };
        let mut vm = Runtime::with_data_dir(apk(), &directory).unwrap();
        let package = vm.apk.manifest.package.clone();
        let root = vm.invoke(method.clone(), vec![], false).unwrap()[0];
        assert_eq!(vm.file_path(root).unwrap(), EXTERNAL_ROOT);
        assert_eq!(
            vm.guest_file_relative(EXTERNAL_ROOT),
            Some(vec!["external".into()])
        );
        assert!(
            vm.ensure_guest_directory(&format!("{EXTERNAL_ROOT}/notes/subdir"))
                .unwrap()
        );
        assert!(
            !vm.ensure_guest_directory(&format!("{EXTERNAL_ROOT}/notes/subdir"))
                .unwrap()
        );
        assert!(
            directory
                .join(&package)
                .join("external/notes/subdir")
                .is_dir()
        );
        let file_path = ["external".into(), "notes".into(), "body.txt".into()];
        vm.storage
            .as_mut()
            .unwrap()
            .write_app_file(&file_path, b"saved bytes")
            .unwrap();
        let input = stream(&mut vm, &format!("{EXTERNAL_ROOT}/notes/body.txt"));
        vm.storage
            .as_mut()
            .unwrap()
            .write_app_file(&file_path, b"replacement")
            .unwrap();
        assert_eq!(
            call(&mut vm, input, "available", vec![], &[]),
            vec![Word::from(11)]
        );
        let target = bytes(&mut vm, 11);
        assert_eq!(
            call(&mut vm, input, "read", vec!["[B"], &[target]),
            vec![Word::from(11)]
        );
        let Data::Array { values, .. } = &vm.heap.get(target).unwrap().data else {
            panic!("expected bytes")
        };
        assert_eq!(
            values
                .iter()
                .map(|v| v[0].int().unwrap() as u8)
                .collect::<Vec<_>>(),
            b"saved bytes"
        );
        call(&mut vm, input, "close", vec![], &[]);
        let file = vm
            .file_object(format!("/data/data/{package}/external/notes/body.txt"))
            .unwrap();
        let input = vm.heap.instance("Ljava/io/FileInputStream;").unwrap();
        vm.invoke(
            Method {
                class: "Ljava/io/FileInputStream;".into(),
                name: "<init>".into(),
                parameters: vec!["Ljava/io/File;".into()],
                returns: "V".into(),
            },
            vec![input, file],
            false,
        )
        .unwrap();
        assert!(
            matches!(&vm.heap.get(input).unwrap().data, Data::ByteStream { bytes, .. } if bytes==b"replacement")
        );
        let oversized =
            std::fs::File::create(directory.join(&package).join("external/oversized.bin")).unwrap();
        oversized
            .set_len(crate::storage::MAX_APP_FILE_BYTES as u64 + 1)
            .unwrap();
        let unopened = vm.heap.instance("Ljava/io/FileInputStream;").unwrap();
        let error = vm
            .open_virtual_file_input(unopened, format!("{EXTERNAL_ROOT}/oversized.bin"))
            .unwrap_err();
        assert!(format!("{error:#}").contains("app file exceeds 64 MiB"));
        assert!(matches!(
            vm.heap.get(unopened).unwrap().data,
            Data::Instance
        ));
        for path in [
            "/storage/emulated/1/notes",
            "/storage/emulated/0/../escape",
            "/storage/emulated/0/../../../etc/passwd",
            "/storage/emulated/0/notes/\0bad",
            "/data/data/org.droidless.other_external/external",
        ] {
            assert!(vm.guest_file_relative(path).is_none(), "accepted {path:?}");
        }
        let mut reopened = Runtime::with_data_dir(apk(), &directory).unwrap();
        reopened.invoke(method.clone(), vec![], false).unwrap();
        assert_eq!(
            reopened
                .file_path_kind(&["external".into(), "notes".into(), "subdir".into()])
                .unwrap(),
            Some(true)
        );
        let mut other_apk = apk();
        other_apk.manifest.package = "org.droidless.other_external".into();
        let mut other = Runtime::with_data_dir(other_apk, &directory).unwrap();
        other.invoke(method.clone(), vec![], false).unwrap();
        assert_eq!(
            other
                .file_path_kind(&["external".into(), "notes".into()])
                .unwrap(),
            None
        );
        let mut ephemeral = runtime();
        ephemeral.invoke(method.clone(), vec![], false).unwrap();
        assert!(
            ephemeral
                .ensure_guest_directory(&format!("{EXTERNAL_ROOT}/memory-only"))
                .unwrap()
        );
        assert!(
            !directory
                .join(&package)
                .join("external/memory-only")
                .exists()
        );
        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(
                directory.join(&package).join("external/notes"),
                directory.join(&package).join("external/link"),
            )
            .unwrap();
            assert!(
                reopened
                    .ensure_guest_directory(&format!("{EXTERNAL_ROOT}/link/escape"))
                    .is_err()
            );
            assert!(
                !directory
                    .join(&package)
                    .join("external/notes/escape")
                    .exists()
            );
            let source = directory.join(&package).join("external/notes/body.txt");
            std::os::unix::fs::symlink(
                &source,
                directory.join(&package).join("external/linked.txt"),
            )
            .unwrap();
            std::fs::hard_link(&source, directory.join(&package).join("external/hard.txt"))
                .unwrap();
            for name in ["linked.txt", "hard.txt"] {
                let input = reopened.heap.instance("Ljava/io/FileInputStream;").unwrap();
                assert!(
                    reopened
                        .open_virtual_file_input(input, format!("{EXTERNAL_ROOT}/{name}"))
                        .unwrap_err()
                        .downcast_ref::<crate::heap::GuestFault>()
                        .is_some_and(|error| error.0 == "Ljava/io/FileNotFoundException;")
                );
            }
        }
        assert!(vm.invoke(method, vec![Word::ZERO], false).is_err());
        drop((vm, reopened, other, ephemeral));
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn file_list_returns_only_app_private_entries() {
        let mut vm = runtime();
        let nested = vm
            .heap
            .string(format!(
                "/data/data/{}/files/cache",
                vm.apk.manifest.package
            ))
            .unwrap();
        let directory = vm.heap.instance("Ljava/io/File;").unwrap();
        vm.invoke(
            Method {
                class: "Ljava/io/File;".into(),
                name: "<init>".into(),
                parameters: vec!["Ljava/lang/String;".into()],
                returns: "V".into(),
            },
            vec![directory, nested],
            false,
        )
        .unwrap();
        vm.invoke(
            Method {
                class: "Ljava/io/File;".into(),
                name: "mkdirs".into(),
                parameters: vec![],
                returns: "Z".into(),
            },
            vec![directory],
            false,
        )
        .unwrap();
        let files = vm.heap.instance("Ljava/io/File;").unwrap();
        vm.heap.get_mut(files).unwrap().data =
            Data::File(format!("/data/data/{}/files", vm.apk.manifest.package));
        let names = vm
            .invoke(
                Method {
                    class: "Ljava/io/File;".into(),
                    name: "list".into(),
                    parameters: vec![],
                    returns: "[Ljava/lang/String;".into(),
                },
                vec![files],
                false,
            )
            .unwrap()[0];
        let Data::Array { values, .. } = &vm.heap.get(names).unwrap().data else {
            panic!("File.list did not return a String array");
        };
        assert_eq!(values.len(), 1);
        assert_eq!(vm.heap.text(values[0][0]).unwrap(), "cache");

        let outside = vm.heap.instance("Ljava/io/File;").unwrap();
        vm.heap.get_mut(outside).unwrap().data = Data::File("/etc".into());
        assert_eq!(
            vm.invoke(
                Method {
                    class: "Ljava/io/File;".into(),
                    name: "list".into(),
                    parameters: vec![],
                    returns: "[Ljava/lang/String;".into(),
                },
                vec![outside],
                false,
            )
            .unwrap()[0],
            Word::ZERO
        );
    }
}
