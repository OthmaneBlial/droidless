//! Host-selected directory grants. Guest URI parsing never grants filesystem access.
use crate::{
    activities::ActivityResult,
    heap::{Data, SqlValue, Word, fault},
    storage::{MAX_APP_FILE_BYTES, regular_file, validate_segment},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
use cap_std::fs::{Dir, Metadata, OpenOptions};
use droidless_formats::dex::Method;
use std::{io::Read, path::Path};

const AUTHORITY: &str = "droidless.documents";
const COLUMNS: &[&str] = &[
    "document_id",
    "mime_type",
    "_display_name",
    "_size",
    "flags",
];

fn encode_segment(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || b"_-!.~'()*".contains(&byte) {
            out.push(char::from(byte));
        } else {
            use std::fmt::Write;
            write!(out, "%{byte:02X}").unwrap();
        }
    }
    out
}
fn decode_segment(value: &str) -> Result<String> {
    let mut out = Vec::new();
    let mut bytes = value.bytes();
    while let Some(byte) = bytes.next() {
        out.push(if byte == b'%' {
            let high = bytes.next().context("truncated URI escape")?;
            let low = bytes.next().context("truncated URI escape")?;
            let digit = |byte| char::from(byte).to_digit(16).context("invalid URI escape");
            (digit(high)? * 16 + digit(low)?) as u8
        } else {
            byte
        });
    }
    String::from_utf8(out).context("URI path is not UTF-8")
}
fn uri_parts(text: &str) -> (Option<&str>, &str) {
    let text = text.split(['?', '#']).next().unwrap_or("");
    let rest = if let Some((_, rest)) = text.split_once(':') {
        if !text.split(':').next().unwrap_or("").contains('/') {
            rest
        } else {
            text
        }
    } else {
        text
    };
    if let Some(rest) = rest.strip_prefix("//") {
        let end = rest.find('/').unwrap_or(rest.len());
        (Some(&rest[..end]), &rest[end..])
    } else if text != rest && !rest.starts_with('/') {
        (None, "") // Opaque URI has no hierarchical path.
    } else {
        (None, rest)
    }
}
fn uri_segments(text: &str) -> Result<Vec<String>> {
    uri_parts(text)
        .1
        .split('/')
        .filter(|part| !part.is_empty())
        .map(decode_segment)
        .collect()
}
fn tree_id(parts: &[String]) -> Result<&str> {
    ensure!(
        parts.len() >= 2 && parts[0] == "tree",
        fault("Ljava/lang/IllegalArgumentException;", "not a tree URI")
    );
    Ok(&parts[1])
}
fn document_id(parts: &[String]) -> Result<&str> {
    if parts.len() >= 2 && parts[0] == "document" {
        return Ok(&parts[1]);
    }
    ensure!(
        parts.len() >= 4 && parts[0] == "tree" && parts[2] == "document",
        fault("Ljava/lang/IllegalArgumentException;", "not a document URI")
    );
    Ok(&parts[3])
}
fn mime(name: &str, directory: bool) -> &'static str {
    if directory {
        return "vnd.android.document/directory";
    }
    match name
        .rsplit('.')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "png" => "image/png",
        "jpg" | "jpeg" => "image/jpeg",
        "webp" => "image/webp",
        "gif" => "image/gif",
        "bmp" => "image/bmp",
        "txt" => "text/plain",
        "pdf" => "application/pdf",
        _ => "application/octet-stream",
    }
}

impl Runtime {
    pub(crate) fn uri_bitmap(&mut self, context: Word, uri: Word) -> Result<Word> {
        let roots = self.native_roots.len();
        self.native_roots.extend([context, uri]);
        let result = (|| -> Result<Word> {
            let resolver = self.invoke(
                Method {
                    class: "Landroid/content/Context;".into(),
                    name: "getContentResolver".into(),
                    parameters: vec![],
                    returns: "Landroid/content/ContentResolver;".into(),
                },
                vec![context],
                true,
            )?[0];
            self.native_roots.push(resolver);
            let stream = self.invoke(
                Method {
                    class: "Landroid/content/ContentResolver;".into(),
                    name: "openInputStream".into(),
                    parameters: vec!["Landroid/net/Uri;".into()],
                    returns: "Ljava/io/InputStream;".into(),
                },
                vec![resolver, uri],
                true,
            )?[0];
            self.native_roots.push(stream);
            let bitmap = self.invoke(
                Method {
                    class: "Landroid/graphics/BitmapFactory;".into(),
                    name: "decodeStream".into(),
                    parameters: vec!["Ljava/io/InputStream;".into()],
                    returns: "Landroid/graphics/Bitmap;".into(),
                },
                vec![stream],
                false,
            );
            if let Ok(words) = &bitmap {
                self.native_roots.extend_from_slice(words);
            }
            let closed = self.invoke(
                Method {
                    class: "Ljava/io/InputStream;".into(),
                    name: "close".into(),
                    parameters: vec![],
                    returns: "V".into(),
                },
                vec![stream],
                true,
            );
            let bitmap = bitmap?[0];
            closed?;
            Ok(bitmap)
        })();
        self.native_roots.truncate(roots);
        result
    }
    fn uri_text(&self, uri: Word) -> Result<&str> {
        let text = self
            .heap
            .get(uri)?
            .fields
            .get("droidless:uri:text")
            .and_then(|v| v.first())
            .context("uninitialized Uri")?;
        self.heap.text(*text)
    }
    fn uri_string(&mut self, text: String) -> Result<Word> {
        let text = self.heap.string(text)?;
        self.new_uri(text)
    }
    fn new_uri(&mut self, text: Word) -> Result<Word> {
        ensure!(
            self.heap.text(text)?.len() <= 16_384,
            "URI length limit exceeded"
        );
        let uri = self.heap.instance("Landroid/net/Uri;")?;
        self.heap
            .get_mut(uri)?
            .fields
            .insert("droidless:uri:text".into(), vec![text]);
        Ok(uri)
    }
    pub(crate) fn document_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let arg = |index| args.get(index).copied().context("missing URI argument");
        let result = match (method.class.as_str(), method.signature().as_str()) {
            ("Landroid/net/Uri;", "parse(Ljava/lang/String;)Landroid/net/Uri;") => {
                vec![self.new_uri(arg(0)?)?]
            }
            ("Landroid/net/Uri;", "toString()Ljava/lang/String;") => vec![
                *self
                    .heap
                    .get(arg(0)?)?
                    .fields
                    .get("droidless:uri:text")
                    .and_then(|v| v.first())
                    .context("uninitialized Uri")?,
            ],
            ("Landroid/net/Uri;", "getAuthority()Ljava/lang/String;") => {
                let authority = uri_parts(self.uri_text(arg(0)?)?).0.map(str::to_owned);
                vec![if let Some(authority) = authority {
                    self.heap.string(authority)?
                } else {
                    Word::ZERO
                }]
            }
            ("Landroid/net/Uri;", "getPathSegments()Ljava/util/List;") => {
                let parts = uri_segments(self.uri_text(arg(0)?)?)?;
                let values = parts
                    .into_iter()
                    .map(|part| self.heap.string(part))
                    .collect::<Result<Vec<_>>>()?;
                let list = self.heap.instance("Ljava/util/ArrayList;")?;
                self.heap.get_mut(list)?.data = Data::Collection { values, version: 0 };
                self.collection_native(
                    &Method {
                        class: "Ljava/util/Collections;".into(),
                        name: "unmodifiableList".into(),
                        parameters: vec!["Ljava/util/List;".into()],
                        returns: "Ljava/util/List;".into(),
                    },
                    &[list],
                )?
                .context("missing read-only List implementation")?
            }
            (
                "Landroid/provider/DocumentsContract;",
                "getTreeDocumentId(Landroid/net/Uri;)Ljava/lang/String;"
                | "getDocumentId(Landroid/net/Uri;)Ljava/lang/String;",
            ) => {
                let parts = uri_segments(self.uri_text(arg(0)?)?)?;
                let id = if method.name == "getTreeDocumentId" {
                    tree_id(&parts)?
                } else {
                    document_id(&parts)?
                };
                vec![self.heap.string(id.to_owned())?]
            }
            (
                "Landroid/provider/DocumentsContract;",
                "buildDocumentUriUsingTree(Landroid/net/Uri;Ljava/lang/String;)Landroid/net/Uri;"
                | "buildChildDocumentsUriUsingTree(Landroid/net/Uri;Ljava/lang/String;)Landroid/net/Uri;",
            ) => {
                let text = self.uri_text(arg(0)?)?;
                let authority = uri_parts(text)
                    .0
                    .context("tree URI has no authority")?
                    .to_owned();
                let parts = uri_segments(text)?;
                let tree = encode_segment(tree_id(&parts)?);
                let document = encode_segment(self.heap.text(arg(1)?)?);
                let suffix = if method.name == "buildChildDocumentsUriUsingTree" {
                    "/children"
                } else {
                    ""
                };
                vec![self.uri_string(format!(
                    "content://{authority}/tree/{tree}/document/{document}{suffix}"
                ))?]
            }
            (
                "Landroid/content/Context;",
                "getContentResolver()Landroid/content/ContentResolver;",
            ) => {
                self.heap.get(arg(0)?)?;
                let resolver = if let Some(value) = self
                    .statics
                    .get("droidless:resolver")
                    .and_then(|v| v.first())
                {
                    *value
                } else {
                    let value = self.heap.instance("Landroid/content/ContentResolver;")?;
                    self.statics
                        .insert("droidless:resolver".into(), vec![value]);
                    value
                };
                vec![resolver]
            }
            (
                "Landroid/content/ContentResolver;",
                "query(Landroid/net/Uri;[Ljava/lang/String;Ljava/lang/String;[Ljava/lang/String;Ljava/lang/String;)Landroid/database/Cursor;",
            ) => {
                self.heap.get(arg(0)?)?;
                ensure!(
                    args[3..].iter().all(|arg| *arg == Word::ZERO),
                    "document selection and sort options are unsupported"
                );
                vec![self.query_documents(arg(1)?, arg(2)?)?]
            }
            (
                "Landroid/content/ContentResolver;",
                "openInputStream(Landroid/net/Uri;)Ljava/io/InputStream;",
            ) => {
                self.heap.get(arg(0)?)?;
                let (root, path, _) = self.document_target(arg(1)?)?;
                let (parent, name) = self.document_parent(&root, &path)?;
                let name = name.context("cannot open a directory as a stream")?;
                let mut options = OpenOptions::new();
                options.read(true).follow(FollowSymlinks::No).nonblock(true);
                let file = parent
                    .open_with(name, &options)
                    .map_err(|error| fault("Ljava/io/FileNotFoundException;", error.to_string()))?
                    .into_std();
                regular_file(&file)?;
                ensure!(
                    file.metadata()?.len() <= MAX_APP_FILE_BYTES as u64,
                    "document stream exceeds 64 MiB"
                );
                let mut bytes = Vec::new();
                file.take((MAX_APP_FILE_BYTES + 1) as u64)
                    .read_to_end(&mut bytes)?;
                ensure!(
                    bytes.len() <= MAX_APP_FILE_BYTES,
                    "document stream exceeds 64 MiB"
                );
                let stream = self.heap.instance("Ljava/io/FileInputStream;")?;
                self.heap.get_mut(stream)?.data = Data::ByteStream {
                    bytes,
                    position: 0,
                    closed: false,
                };
                vec![stream]
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }
    fn document_target(&self, uri: Word) -> Result<(String, Vec<String>, bool)> {
        let text = self.uri_text(uri)?;
        ensure!(
            text.starts_with("content://droidless.documents/") && !text.contains(['?', '#']),
            fault(
                "Ljava/lang/SecurityException;",
                "document provider is not granted"
            )
        );
        let parts = uri_segments(text)?;
        let root = tree_id(&parts)?.to_owned();
        ensure!(
            self.document_trees.contains_key(&root),
            fault(
                "Ljava/lang/SecurityException;",
                "document tree is not granted"
            )
        );
        let document = document_id(&parts)?;
        let relative = if document == root {
            ""
        } else {
            document
                .strip_prefix(&format!("{root}/"))
                .context("document is outside granted tree")?
        };
        let path = if relative.is_empty() {
            vec![]
        } else {
            relative.split('/').map(str::to_owned).collect::<Vec<_>>()
        };
        ensure!(path.len() <= 64, "document path depth exceeds 64");
        for part in &path {
            validate_segment(part)?;
        }
        let children = parts.len() == 5 && parts[4] == "children";
        ensure!(
            parts.len() == 4 || children,
            "unsupported document URI shape"
        );
        Ok((root, path, children))
    }
    fn document_parent(&self, root: &str, path: &[String]) -> Result<(Dir, Option<String>)> {
        let mut directory = self
            .document_trees
            .get(root)
            .context("document tree is not granted")?
            .try_clone()?;
        for part in path.iter().take(path.len().saturating_sub(1)) {
            directory = directory
                .open_dir_nofollow(part)
                .context("document parent must be a real directory")?;
        }
        Ok((directory, path.last().cloned()))
    }
    fn query_documents(&mut self, uri: Word, projection: Word) -> Result<Word> {
        let (root, path, children) = self.document_target(uri)?;
        let columns = if projection == Word::ZERO {
            COLUMNS.iter().map(|column| (*column).to_owned()).collect()
        } else {
            let Data::Array { values, element } = &self.heap.get(projection)?.data else {
                bail!("document projection requires String[]");
            };
            ensure!(
                element == "Ljava/lang/String;" && values.len() <= 64,
                "invalid document projection"
            );
            values
                .iter()
                .map(|value| {
                    self.heap
                        .text(*value.first().context("missing projection value")?)
                        .map(str::to_owned)
                })
                .collect::<Result<Vec<_>>>()?
        };
        ensure!(
            columns
                .iter()
                .all(|column| COLUMNS.contains(&column.as_str())),
            "unsupported document column"
        );
        let (parent, name) = self.document_parent(&root, &path)?;
        let id = std::iter::once(root.as_str())
            .chain(path.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join("/");
        let mut records = Vec::new();
        if children {
            let directory = if let Some(name) = name {
                parent.open_dir_nofollow(name)?
            } else {
                parent
            };
            // ponytail: snapshot up to 4096 entries; paged cursors are needed for larger folders.
            for (index, entry) in directory.entries()?.enumerate() {
                ensure!(
                    index < 4096,
                    "document directory entry limit reached (4096)"
                );
                let entry = entry?;
                let name = entry
                    .file_name()
                    .into_string()
                    .map_err(|_| anyhow::anyhow!("document name is not UTF-8"))?;
                let metadata = directory.symlink_metadata(&name)?;
                if metadata.file_type().is_symlink() || (!metadata.is_file() && !metadata.is_dir())
                {
                    continue;
                }
                #[cfg(unix)]
                {
                    use cap_std::fs::MetadataExt;
                    if metadata.is_file() && metadata.nlink() != 1 {
                        continue;
                    }
                }
                records.push((format!("{id}/{name}"), name, metadata));
            }
            records.sort_by(|left, right| left.1.encode_utf16().cmp(right.1.encode_utf16()));
        } else {
            let metadata = if let Some(name) = &name {
                parent.symlink_metadata(name)?
            } else {
                parent.dir_metadata()?
            };
            ensure!(
                !metadata.file_type().is_symlink() && (metadata.is_file() || metadata.is_dir()),
                "document is not a regular file or directory"
            );
            #[cfg(unix)]
            {
                use cap_std::fs::MetadataExt;
                ensure!(
                    !metadata.is_file() || metadata.nlink() == 1,
                    "hard-linked document rejected"
                );
            }
            records.push((
                id,
                name.unwrap_or_else(|| "Selected folder".into()),
                metadata,
            ));
        }
        let rows = records
            .into_iter()
            .map(|(id, name, metadata)| document_row(&columns, id, name, metadata))
            .collect();
        let cursor = self.heap.instance("Landroid/database/Cursor;")?;
        self.heap.get_mut(cursor)?.data = Data::Cursor {
            columns,
            rows,
            position: -1,
            closed: false,
        };
        Ok(cursor)
    }
    pub fn directory_picker_pending(&self) -> bool {
        self.directory_request.is_some()
    }

    /// Complete a real host choice; None is cancellation. Grants are read-only and session-local.
    pub fn complete_directory_picker(&mut self, path: Option<&Path>) -> Result<()> {
        self.require_main_thread()?;
        self.reset_budget();
        let (caller, request) = self
            .directory_request
            .context("no directory picker pending")?;
        ensure!(
            self.activity == Some(caller),
            "directory picker caller is no longer foreground"
        );
        let data = if let Some(path) = path {
            let path = path.canonicalize().context("resolve selected directory")?;
            ensure!(path.is_dir(), "selected path is not a directory");
            // ponytail: 64 session grants; persistent/revocable grants require a separate permission model.
            ensure!(
                self.document_trees.len() < 64,
                "document tree grant limit reached (64)"
            );
            let directory = cap_std::fs::Dir::open_ambient_dir(&path, cap_std::ambient_authority())
                .context("open host-selected document tree")?;
            let id = self.document_trees.len().to_string();
            let text = self
                .heap
                .string(format!("content://{AUTHORITY}/tree/{id}"))?;
            let uri = self.new_uri(text)?;
            let intent = self.heap.instance("Landroid/content/Intent;")?;
            self.heap
                .get_mut(intent)?
                .fields
                .insert("data".into(), vec![uri]);
            self.document_trees.insert(id, directory);
            intent
        } else {
            Word::ZERO
        };
        self.enqueue_result(
            caller,
            ActivityResult {
                request,
                code: if data == Word::ZERO { 0 } else { -1 },
                data,
            },
        )?;
        self.directory_request = None;
        self.flush_active_results()?;
        self.drain_navigation()?;
        self.collect();
        Ok(())
    }
}

fn document_row(columns: &[String], id: String, name: String, metadata: Metadata) -> Vec<SqlValue> {
    columns
        .iter()
        .map(|column| match column.as_str() {
            "document_id" => SqlValue::Text(id.clone()),
            "mime_type" => SqlValue::Text(mime(&name, metadata.is_dir()).into()),
            "_display_name" => SqlValue::Text(name.clone()),
            "_size" if metadata.is_file() => {
                SqlValue::Integer(metadata.len().min(i64::MAX as u64) as i64)
            }
            "flags" => SqlValue::Integer(0),
            _ => SqlValue::Null,
        })
        .collect()
}
