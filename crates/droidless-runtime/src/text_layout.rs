use crate::{heap::Word, ui, vm::Runtime};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

pub(crate) const LAYOUT: &str = "droidless:text:layout";
const TEXT: &str = "droidless:layout:text";
const WIDTH: &str = "droidless:layout:width";
pub(crate) const LINES: &str = "droidless:layout:lines";

impl Runtime {
    pub(crate) fn text_layout_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        match (method.class.as_str(), method.signature().as_str()) {
            ("Landroid/widget/TextView;", "getLayout()Landroid/text/Layout;") => {
                ensure!(args.len() == 1, "invalid getLayout arguments");
                let object = self.heap.get(args[0])?;
                object.view.as_ref().context("TextView has no View state")?;
                Ok(Some(vec![
                    object
                        .fields
                        .get(LAYOUT)
                        .and_then(|v| v.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                ]))
            }
            ("Landroid/widget/TextView;", "onMeasure(II)V") => {
                ensure!(args.len() == 3, "invalid TextView measure arguments");
                self.measure_view(args[0], [args[1], args[2]], true)?;
                Ok(Some(vec![]))
            }
            (
                "Landroid/text/Layout;",
                "getLineCount()I" | "getWidth()I" | "getText()Ljava/lang/CharSequence;",
            ) => {
                ensure!(args.len() == 1, "invalid Layout getter arguments");
                let key = match method.name.as_str() {
                    "getLineCount" => LINES,
                    "getWidth" => WIDTH,
                    _ => TEXT,
                };
                Ok(Some(vec![
                    *self
                        .heap
                        .get(args[0])?
                        .fields
                        .get(key)
                        .and_then(|v| v.first())
                        .context("uninitialized text Layout")?,
                ]))
            }
            _ => Ok(None),
        }
    }

    pub(crate) fn invalidate_text_layout(&mut self, view: Word) -> Result<()> {
        let object = self.heap.get_mut(view)?;
        object.view.as_ref().context("expected View")?;
        object.fields.remove(LAYOUT);
        object.fields.insert(
            "droidless:view:layout-requested".into(),
            vec![Word::from(1)],
        );
        Ok(())
    }

    pub(crate) fn measure_view(
        &mut self,
        receiver: Word,
        specs: [Word; 2],
        text: bool,
    ) -> Result<()> {
        self.heap
            .get(receiver)?
            .view
            .as_ref()
            .context("expected View")?;
        for (spec, horizontal, edge) in [(specs[0], true, "width"), (specs[1], false, "height")] {
            let spec = spec.int()? as u32;
            let mode = spec & 0xc000_0000;
            let size = (spec & 0x3fff_ffff) as i32;
            let minimum = self
                .heap
                .get(receiver)?
                .fields
                .get(&format!("droidless:view:minimum-{edge}"))
                .and_then(|v| v.first())
                .copied()
                .unwrap_or(Word::ZERO)
                .int()?;
            let desired = (ui::dimension(
                &self.heap,
                receiver,
                horizontal,
                if mode == 0 {
                    f32::INFINITY
                } else {
                    size as f32
                },
            )? as i32)
                .max(minimum);
            let measured = match mode {
                0x4000_0000 => size,
                0x8000_0000 => desired.min(size),
                _ => desired,
            }
            .max(0);
            self.heap.get_mut(receiver)?.fields.insert(
                format!("droidless:view:measured-{edge}"),
                vec![Word::from(measured)],
            );
            if text && horizontal {
                let object = self.heap.get(receiver)?;
                let view = object.view.as_ref().context("expected TextView")?;
                let width = (measured as f32 - view.padding[0] - view.padding[2]).max(0.0) as i32;
                let single = object
                    .fields
                    .get("droidless:text:single-line")
                    .and_then(|v| v.first())
                    .is_some_and(|v| v.truth());
                let lines = ui::text_line_count(&view.text, view.text_size, width as f32, single);
                let previous = object.fields.get(LAYOUT).and_then(|v| v.first()).copied();
                if let Some(previous) = previous {
                    let fields = &self.heap.get(previous)?.fields;
                    if fields.get(WIDTH).and_then(|v| v.first()) == Some(&Word::from(width))
                        && fields.get(LINES).and_then(|v| v.first()) == Some(&Word::from(lines))
                        && self.heap.text(
                            *fields
                                .get(TEXT)
                                .and_then(|v| v.first())
                                .context("Layout text missing")?,
                        )? == view.text
                    {
                        continue;
                    }
                }
                let owned_text = self.heap.string(view.text.clone())?;
                let layout = self.heap.instance("Landroid/text/Layout;")?;
                for (key, value) in [
                    (TEXT, owned_text),
                    (WIDTH, Word::from(width)),
                    (LINES, Word::from(lines)),
                ] {
                    self.heap
                        .get_mut(layout)?
                        .fields
                        .insert(key.into(), vec![value]);
                }
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert(LAYOUT.into(), vec![layout]);
            }
        }
        Ok(())
    }
}
