//! Host-selected directory grants. Guest URI parsing never grants filesystem access.
use crate::{activities::ActivityResult, heap::Word, vm::Runtime};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;
use std::path::Path;

impl Runtime {
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
        if method.class != "Landroid/net/Uri;" {
            return Ok(None);
        }
        let arg = |index| args.get(index).copied().context("missing URI argument");
        let result = match method.signature().as_str() {
            "parse(Ljava/lang/String;)Landroid/net/Uri;" => vec![self.new_uri(arg(0)?)?],
            "toString()Ljava/lang/String;" => vec![
                *self
                    .heap
                    .get(arg(0)?)?
                    .fields
                    .get("droidless:uri:text")
                    .and_then(|v| v.first())
                    .context("uninitialized Uri")?,
            ],
            _ => return Ok(None),
        };
        Ok(Some(result))
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
                .string(format!("content://droidless.documents/tree/{id}"))?;
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
