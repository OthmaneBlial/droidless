use crate::{heap::Word, ui, vm::Runtime};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

pub(crate) const LAYOUT: &str = "droidless:text:layout";
const TEXT: &str = "droidless:layout:text";
const SOURCE: &str = "droidless:layout:source";
const WIDTH: &str = "droidless:layout:width";
pub(crate) const LINES: &str = "droidless:layout:lines";
pub(crate) const DISPLAY: &str = "droidless:layout:display";
const ELLIPSIS: &str = "droidless:layout:ellipsis";
const ELLIPSIZE: &str = "droidless:text:ellipsize";

struct LayoutText {
    lines: i32,
    text: String,
    display: Option<String>,
    ellipsis: Option<[i32; 3]>,
}

fn layout_text(
    text: &str,
    size: f32,
    width: i32,
    single: bool,
    max: i32,
    mode: Option<i32>,
) -> LayoutText {
    let mut lines = ui::text_lines(text, size, width as f32, single);
    let max = if single { 1 } else { max.max(1) as usize };
    let hidden = mode.is_some() && lines.len() > max;
    if mode.is_some() {
        lines.truncate(max);
    }
    let mut result = LayoutText {
        lines: lines.len() as i32,
        text: text.to_owned(),
        display: None,
        ellipsis: None,
    };
    let line = lines.len() - 1;
    let range = &lines[line];
    let tail = &text[range.clone()];
    let characters: Vec<_> = tail.char_indices().collect();
    let advance = ui::text_advance(size);
    if let Some(mode) = mode
        && mode != 3 // MARQUEE is not a static ellipsis.
        && (max == 1 || mode == 2)
        && (hidden || tail.trim_end_matches('\n').chars().count() as f32 * advance > width as f32)
        && !characters.is_empty()
    {
        let keep = if advance == 0.0 {
            characters.len()
        } else {
            ((width as f32 - advance) / advance).floor().max(0.0) as usize
        }
        .min(characters.len().saturating_sub(1));
        let (left, right) = match mode {
            0 => (0, characters.len() - keep),
            1 => (keep - keep / 2, characters.len() - keep / 2),
            _ => (keep, characters.len()),
        };
        let byte_start = characters[left].0;
        let byte_end = characters.get(right).map_or(tail.len(), |c| c.0);
        let start = tail[..byte_start].encode_utf16().count();
        let count = tail[byte_start..byte_end].encode_utf16().count();
        let marker = if mode == 4 { '\u{2025}' } else { '\u{2026}' };
        let replacement = format!("{marker}{}", "\u{feff}".repeat(count - 1));
        result.text.replace_range(
            range.start + byte_start..range.start + byte_end,
            &replacement,
        );
        result.ellipsis = Some([line as i32, start as i32, count as i32]);
    }
    if hidden || result.ellipsis.is_some() {
        let mut displayed = Vec::with_capacity(lines.len());
        for (index, range) in lines.iter().enumerate() {
            let mut value = text[range.clone()].to_owned();
            if let Some([ellipsis_line, start, count]) = result.ellipsis
                && index == ellipsis_line as usize
            {
                let mut utf16 = 0;
                value = value
                    .chars()
                    .filter_map(|character| {
                        let position = utf16;
                        utf16 += character.len_utf16();
                        if position == start as usize {
                            Some(if mode == Some(4) {
                                '\u{2025}'
                            } else {
                                '\u{2026}'
                            })
                        } else if position > start as usize && position < (start + count) as usize {
                            None
                        } else {
                            Some(character)
                        }
                    })
                    .collect();
            }
            displayed.push(value.trim_end_matches('\n').to_owned());
        }
        result.display = Some(displayed.join("\n"));
    }
    result
}

impl Runtime {
    pub(crate) fn text_layout_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        match (method.class.as_str(), method.signature().as_str()) {
            ("Landroid/widget/TextView;", "getEllipsize()Landroid/text/TextUtils$TruncateAt;") => {
                ensure!(args.len() == 1, "invalid getEllipsize arguments");
                let object = self.heap.get(args[0])?;
                object.view.as_ref().context("expected TextView")?;
                Ok(Some(vec![
                    object
                        .fields
                        .get(ELLIPSIZE)
                        .and_then(|v| v.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                ]))
            }
            ("Landroid/widget/TextView;", "setEllipsize(Landroid/text/TextUtils$TruncateAt;)V") => {
                ensure!(args.len() == 2, "invalid setEllipsize arguments");
                self.set_text_ellipsize(args[0], args[1])?;
                Ok(Some(vec![]))
            }
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
                "Landroid/widget/LinearLayout;" | "Landroid/widget/FrameLayout;",
                "onMeasure(II)V",
            ) => {
                ensure!(args.len() == 3, "invalid container measure arguments");
                self.measure_container(
                    args[0],
                    [args[1], args[2]],
                    method.class == "Landroid/widget/LinearLayout;",
                )?;
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
            ("Landroid/text/Layout;", "getEllipsisCount(I)I" | "getEllipsisStart(I)I") => {
                ensure!(args.len() == 2, "invalid Layout ellipsis arguments");
                let fields = &self.heap.get(args[0])?.fields;
                let lines = fields
                    .get(LINES)
                    .and_then(|v| v.first())
                    .context("uninitialized text Layout")?
                    .int()?;
                let line = args[1].int()?;
                ensure!(
                    (0..lines).contains(&line),
                    crate::heap::fault(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        "Layout line out of bounds"
                    )
                );
                let ellipsis = fields.get(ELLIPSIS);
                let value = if let Some(ellipsis) = ellipsis
                    && ellipsis[0].int()? == line
                {
                    ellipsis[if method.name == "getEllipsisStart" {
                        1
                    } else {
                        2
                    }]
                } else {
                    Word::ZERO
                };
                Ok(Some(vec![value]))
            }
            _ => Ok(None),
        }
    }

    fn text_ellipsis_mode(&self, value: Word) -> Result<Option<i32>> {
        if value == Word::ZERO {
            return Ok(None);
        }
        let object = self.heap.get(value)?;
        ensure!(
            object.class == "Landroid/text/TextUtils$TruncateAt;",
            "expected TextUtils.TruncateAt"
        );
        let ordinal = object
            .fields
            .get("droidless:enum:ordinal")
            .and_then(|v| v.first())
            .context("uninitialized TextUtils.TruncateAt")?
            .int()?;
        ensure!(
            (0..=4).contains(&ordinal),
            "invalid TextUtils.TruncateAt ordinal"
        );
        Ok(Some(ordinal))
    }

    pub(crate) fn set_text_ellipsize(&mut self, receiver: Word, value: Word) -> Result<()> {
        self.text_ellipsis_mode(value)?;
        let object = self.heap.get_mut(receiver)?;
        object.view.as_ref().context("expected TextView")?;
        if object
            .fields
            .get(ELLIPSIZE)
            .and_then(|v| v.first())
            .copied()
            .unwrap_or(Word::ZERO)
            != value
        {
            object.fields.insert(ELLIPSIZE.into(), vec![value]);
            self.invalidate_text_layout(receiver)?;
        }
        Ok(())
    }

    pub(crate) fn invalidate_text_layout(&mut self, view: Word) -> Result<()> {
        let object = self.heap.get_mut(view)?;
        object.view.as_ref().context("expected View")?;
        object.fields.remove(LAYOUT);
        self.request_view_layout(view)
    }

    pub(crate) fn request_view_layout(&mut self, view: Word) -> Result<()> {
        let mut path = vec![];
        let mut current = view;
        while current != Word::ZERO {
            ensure!(
                path.len() < 128 && !path.contains(&current),
                "cyclic or too deep View hierarchy"
            );
            let object = self.heap.get(current)?;
            object.view.as_ref().context("expected View")?;
            path.push(current);
            current = object
                .fields
                .get("droidless:view:parent")
                .and_then(|v| v.first())
                .copied()
                .unwrap_or(Word::ZERO);
        }
        for view in path {
            self.heap.get_mut(view)?.fields.insert(
                "droidless:view:layout-requested".into(),
                vec![Word::from(1)],
            );
        }
        Ok(())
    }

    fn measured_edge(&self, view: Word, horizontal: bool) -> Result<i32> {
        self.heap
            .get(view)?
            .fields
            .get(if horizontal {
                "droidless:view:measured-width"
            } else {
                "droidless:view:measured-height"
            })
            .and_then(|v| v.first())
            .copied()
            .context("View measurement missing")?
            .int()
    }

    fn measure_container(&mut self, receiver: Word, specs: [Word; 2], linear: bool) -> Result<()> {
        ensure!(self.sync_depth < 128, "View measurement nesting limit");
        let view = self
            .heap
            .get(receiver)?
            .view
            .as_ref()
            .context("expected ViewGroup")?
            .clone();
        let horizontal = view.orientation == 0;
        let main = usize::from(!horizontal);
        let mut total = 0i64;
        let mut weights = 0.0;
        let roots = self.native_roots.len();
        self.native_roots.push(receiver);
        self.native_roots.extend_from_slice(&view.children);
        let measured = (|| -> Result<()> {
            for child in &view.children {
                let child_view = self
                    .heap
                    .get(*child)?
                    .view
                    .as_ref()
                    .context("expected View child")?;
                if child_view.visible == 8 {
                    continue;
                }
                let margins = ui::margins(&self.heap, *child)?;
                let margin = if horizontal {
                    margins[0] + margins[2]
                } else {
                    margins[1] + margins[3]
                };
                let weight = if linear {
                    ui::weight(&self.heap, *child)?
                } else {
                    0.0
                };
                weights += f64::from(weight);
                ensure!(weights.is_finite(), "invalid total layout weight");
                let mut used = [0; 2];
                if linear && weights == 0.0 {
                    used[main] = total.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32;
                }
                let mut dimensions = [None; 2];
                if weight > 0.0 && specs[main].int()? as u32 & 0xc000_0000 != 0x4000_0000 {
                    let name = if horizontal { "width" } else { "height" };
                    let dimension = ui::params_field(
                        &self.heap,
                        *child,
                        &format!("Landroid/view/ViewGroup$LayoutParams;->{name}:I"),
                    )?
                    .map(|v| v.int())
                    .transpose()?
                    .unwrap_or(if horizontal {
                        child_view.width
                    } else {
                        child_view.height
                    } as i32);
                    if dimension == 0 {
                        dimensions[main] = Some(-2);
                    }
                }
                self.measure_child(receiver, *child, specs, used, dimensions)?;
                total += i64::from(self.measured_edge(*child, horizontal)?) + margin as i64;
                ensure!(
                    self.heap
                        .get(receiver)?
                        .view
                        .as_ref()
                        .context("expected ViewGroup")?
                        .children
                        == view.children,
                    "View hierarchy mutation during container measurement unsupported"
                );
            }
            self.measure_view(receiver, specs, false)?;
            let padding = if horizontal {
                view.padding[0] + view.padding[2]
            } else {
                view.padding[1] + view.padding[3]
            };
            let mut remaining =
                i64::from(self.measured_edge(receiver, horizontal)?) - padding as i64 - total;
            for child in &view.children {
                let child_view = self
                    .heap
                    .get(*child)?
                    .view
                    .as_ref()
                    .context("expected View child")?;
                if child_view.visible == 8 {
                    continue;
                }
                let margins = ui::margins(&self.heap, *child)?;
                let weight = if linear {
                    ui::weight(&self.heap, *child)?
                } else {
                    0.0
                };
                let mut exact = [None; 2];
                if weight > 0.0 {
                    let share = if remaining == 0 {
                        0
                    } else {
                        ensure!(weights > 0.0, "invalid remaining layout weight");
                        (remaining as f64 * f64::from(weight) / weights) as i64
                    };
                    let size = (i64::from(self.measured_edge(*child, horizontal)?) + share)
                        .clamp(0, 0x3fff_ffff) as i32;
                    exact[main] = Some(size);
                    remaining -= share;
                    weights -= f64::from(weight);
                }
                for (axis, name) in ["width", "height"].into_iter().enumerate() {
                    let dimension = ui::params_field(
                        &self.heap,
                        *child,
                        &format!("Landroid/view/ViewGroup$LayoutParams;->{name}:I"),
                    )?
                    .map(|v| v.int())
                    .transpose()?
                    .unwrap_or(if axis == 0 {
                        child_view.width
                    } else {
                        child_view.height
                    } as i32);
                    if dimension == -1 && (!linear || axis != main) {
                        let padding = view.padding[axis]
                            + view.padding[axis + 2]
                            + margins[axis]
                            + margins[axis + 2];
                        exact[axis] = Some(
                            self.measured_edge(receiver, axis == 0)?
                                .saturating_sub(padding as i32)
                                .max(0),
                        );
                    }
                }
                if exact.iter().any(Option::is_some) {
                    self.measure_child(receiver, *child, specs, [0; 2], exact)?;
                    ensure!(
                        self.heap
                            .get(receiver)?
                            .view
                            .as_ref()
                            .context("expected ViewGroup")?
                            .children
                            == view.children,
                        "View hierarchy mutation during container measurement unsupported"
                    );
                }
            }
            self.measure_view(receiver, specs, false)
        })();
        self.native_roots.truncate(roots);
        measured
    }

    pub(crate) fn measure_child(
        &mut self,
        parent: Word,
        child: Word,
        specs: [Word; 2],
        used: [i32; 2],
        dimensions: [Option<i32>; 2],
    ) -> Result<()> {
        let padding = self
            .heap
            .get(parent)?
            .view
            .as_ref()
            .context("expected parent View")?
            .padding;
        let (width, height, margins) = {
            let view = self
                .heap
                .get(child)?
                .view
                .as_ref()
                .context("expected child View")?;
            (view.width, view.height, view.margins)
        };
        let mut arguments = vec![child];
        for (axis, name, start, end) in [
            (0, "width", "leftMargin", "rightMargin"),
            (1, "height", "topMargin", "bottomMargin"),
        ] {
            let dimension = if let Some(dimension) = dimensions[axis] {
                dimension
            } else {
                ui::params_field(
                    &self.heap,
                    child,
                    &format!("Landroid/view/ViewGroup$LayoutParams;->{name}:I"),
                )?
                .map(|v| v.int())
                .transpose()?
                .unwrap_or(if axis == 0 { width } else { height } as i32)
            };
            let mut consumed = (padding[axis] + padding[axis + 2]) as i32;
            for (name, edge) in [(start, axis), (end, axis + 2)] {
                consumed = consumed.saturating_add(
                    ui::params_field(
                        &self.heap,
                        child,
                        &format!("Landroid/view/ViewGroup$MarginLayoutParams;->{name}:I"),
                    )?
                    .map(|v| v.int())
                    .transpose()?
                    .unwrap_or(margins[edge] as i32),
                );
            }
            consumed = consumed.saturating_add(used[axis]);
            let spec = self.invoke(
                Method {
                    class: "Landroid/view/ViewGroup;".into(),
                    name: "getChildMeasureSpec".into(),
                    parameters: vec!["I".into(); 3],
                    returns: "I".into(),
                },
                vec![specs[axis], Word::from(consumed), Word::from(dimension)],
                false,
            )?;
            arguments.push(*spec.first().context("child measure spec missing")?);
        }
        let roots = self.native_roots.len();
        self.native_roots.extend([parent, child]);
        let measured = self.invoke(
            Method {
                class: "Landroid/view/View;".into(),
                name: "measure".into(),
                parameters: vec!["I".into(); 2],
                returns: "V".into(),
            },
            arguments,
            true,
        );
        self.native_roots.truncate(roots);
        measured.map(|_| ())
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
            let desired = (ui::intrinsic_dimension(
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
                let max = object
                    .fields
                    .get("droidless:text:setMaxLines")
                    .and_then(|v| v.first())
                    .copied()
                    .unwrap_or(Word::from(i32::MAX))
                    .int()?;
                let mode = if view.editable {
                    None
                } else {
                    self.text_ellipsis_mode(
                        object
                            .fields
                            .get(ELLIPSIZE)
                            .and_then(|v| v.first())
                            .copied()
                            .unwrap_or(Word::ZERO),
                    )?
                };
                ensure!(
                    mode.is_none() || single || max > 0,
                    "ellipsized layouts require positive maxLines"
                );
                let computed = layout_text(&view.text, view.text_size, width, single, max, mode);
                let previous = object.fields.get(LAYOUT).and_then(|v| v.first()).copied();
                if let Some(previous) = previous {
                    let fields = &self.heap.get(previous)?.fields;
                    if fields.get(WIDTH).and_then(|v| v.first()) == Some(&Word::from(width))
                        && fields.get(LINES).and_then(|v| v.first())
                            == Some(&Word::from(computed.lines))
                        && self.heap.text(
                            *fields
                                .get(SOURCE)
                                .and_then(|v| v.first())
                                .context("Layout text missing")?,
                        )? == view.text
                    {
                        continue;
                    }
                }
                let unchanged = computed.text == view.text;
                let source = self.heap.string(view.text.clone())?;
                let owned_text = if unchanged {
                    source
                } else {
                    self.heap.string(computed.text)?
                };
                let display = computed
                    .display
                    .map(|text| self.heap.string(text))
                    .transpose()?;
                let layout = self.heap.instance("Landroid/text/Layout;")?;
                for (key, value) in [
                    (TEXT, owned_text),
                    (SOURCE, source),
                    (WIDTH, Word::from(width)),
                    (LINES, Word::from(computed.lines)),
                ] {
                    self.heap
                        .get_mut(layout)?
                        .fields
                        .insert(key.into(), vec![value]);
                }
                if let Some(display) = display {
                    self.heap
                        .get_mut(layout)?
                        .fields
                        .insert(DISPLAY.into(), vec![display]);
                }
                if let Some(ellipsis) = computed.ellipsis {
                    self.heap
                        .get_mut(layout)?
                        .fields
                        .insert(ELLIPSIS.into(), ellipsis.map(Word::from).to_vec());
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
