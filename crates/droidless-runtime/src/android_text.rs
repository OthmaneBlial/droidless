use crate::{
    heap::{Data, TextSpan, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;

const EDITABLE_FIELD: &str = "droidless:text:editable";
const TEXT_LIMIT: usize = 1_048_576;

fn utf16_len(text: &str) -> usize {
    text.encode_utf16().count()
}

fn find_utf16(source: &[u16], needle: &[u16]) -> Option<usize> {
    (0..=source.len())
        .find(|index| source.get(*index..index.saturating_add(needle.len())) == Some(needle))
}

fn checked_range(start: i32, end: i32, length: usize) -> Result<(usize, usize)> {
    if start < 0 || end < start || end as usize > length {
        return Err(fault(
            "Ljava/lang/IndexOutOfBoundsException;",
            format!("range {start}..{end} for length {length}"),
        ));
    }
    Ok((start as usize, end as usize))
}

fn slice_utf16(text: &str, start: i32, end: i32) -> Result<String> {
    let units: Vec<u16> = text.encode_utf16().collect();
    let (start, end) = checked_range(start, end, units.len())?;
    Ok(String::from_utf16_lossy(&units[start..end]))
}

fn slice_spans(spans: &[TextSpan], start: usize, end: usize) -> Vec<TextSpan> {
    spans
        .iter()
        .filter_map(|span| {
            let overlaps = if span.start == span.end {
                start <= span.start && span.start <= end
            } else {
                span.start < end && span.end > start
            };
            overlaps.then(|| TextSpan {
                object: span.object,
                start: span.start.max(start).min(end) - start,
                end: span.end.max(start).min(end) - start,
                flags: span.flags,
            })
        })
        .collect()
}

impl Runtime {
    pub(crate) fn text_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class == "Landroid/text/TextUtils;"
            && method.signature() == "indexOf(Ljava/lang/CharSequence;Ljava/lang/CharSequence;)I"
        {
            ensure!(args.len() == 2, "invalid TextUtils.indexOf arguments");
            let source = self.heap.text(args[0])?.encode_utf16().collect::<Vec<_>>();
            let needle = self.heap.text(args[1])?.encode_utf16().collect::<Vec<_>>();
            let found = find_utf16(&source, &needle);
            return Ok(Some(vec![Word::from(
                found.map_or(-1, |index| index as i32),
            )]));
        }
        if method.class == "Landroid/text/TextUtils;"
            && method.signature()
                == "replace(Ljava/lang/CharSequence;[Ljava/lang/String;[Ljava/lang/CharSequence;)Ljava/lang/CharSequence;"
        {
            ensure!(args.len() == 3, "invalid TextUtils.replace arguments");
            let (template, spans) = self.sequence_data(args[0])?;
            ensure!(spans.is_empty(), "TextUtils.replace with spans unsupported");
            let references = |array| -> Result<Vec<Word>> {
                let Data::Array { values, .. } = &self.heap.get(array)?.data else {
                    bail!("TextUtils.replace requires arrays");
                };
                ensure!(
                    values.len() <= 16_384,
                    "text replacement array limit reached"
                );
                values
                    .iter()
                    .map(|words| {
                        ensure!(words.len() == 1, "invalid text replacement element width");
                        words[0].reference()?;
                        Ok(words[0])
                    })
                    .collect()
            };
            let sources = references(args[1])?;
            let destinations = references(args[2])?;
            ensure!(
                sources.len() == destinations.len(),
                "text replacement array lengths differ"
            );
            let units = template.encode_utf16().collect::<Vec<_>>();
            let mut replacements = vec![];
            for (source, destination) in sources.into_iter().zip(destinations) {
                let source = self.heap.text(source)?.encode_utf16().collect::<Vec<_>>();
                // ponytail: disjoint plain-text replacements; span/overlap parity needs the full Editable replace engine.
                ensure!(
                    !source.is_empty(),
                    "empty text replacement source unsupported"
                );
                if let Some(start) = find_utf16(&units, &source) {
                    let (text, spans) = self.sequence_data(destination)?;
                    ensure!(
                        spans.is_empty(),
                        "TextUtils.replace destination spans unsupported"
                    );
                    replacements.push((start, start + source.len(), text));
                }
            }
            replacements.sort_by_key(|replacement| replacement.0);
            let mut text = String::new();
            let mut previous = 0;
            for (start, end, replacement) in replacements {
                ensure!(
                    start >= previous,
                    "overlapping text replacement sources unsupported"
                );
                text.push_str(&String::from_utf16_lossy(&units[previous..start]));
                text.push_str(&replacement);
                ensure!(text.len() <= TEXT_LIMIT, "text replacement exceeds 1 MiB");
                previous = end;
            }
            text.push_str(&String::from_utf16_lossy(&units[previous..]));
            ensure!(text.len() <= TEXT_LIMIT, "text replacement exceeds 1 MiB");
            let result = self
                .heap
                .instance("Landroid/text/SpannableStringBuilder;")?;
            self.heap.get_mut(result)?.data = Data::Spanned {
                text,
                spans: vec![],
            };
            return Ok(Some(vec![result]));
        }
        if ![
            "Ljava/lang/CharSequence;",
            "Landroid/text/Spanned;",
            "Landroid/text/Spannable;",
            "Landroid/text/Editable;",
            "Landroid/text/SpannableStringBuilder;",
            "Landroid/text/SpannableString;",
            "Landroid/text/SpannedString;",
        ]
        .contains(&method.class.as_str())
        {
            return Ok(None);
        }
        if method.name == "<init>"
            && [
                "Landroid/text/SpannableStringBuilder;",
                "Landroid/text/SpannableString;",
                "Landroid/text/SpannedString;",
            ]
            .contains(&method.class.as_str())
        {
            let receiver = *args.first().context("text constructor receiver missing")?;
            let (text, spans) = match args.len() {
                1 => (String::new(), vec![]),
                2 => self.sequence_data(args[1])?,
                4 => {
                    let (source, spans) = self.sequence_data(args[1])?;
                    let (start, end) =
                        checked_range(args[2].int()?, args[3].int()?, utf16_len(&source))?;
                    (
                        slice_utf16(&source, start as i32, end as i32)?,
                        slice_spans(&spans, start, end),
                    )
                }
                _ => bail!("unsupported Android text constructor {}", method.key()),
            };
            ensure!(text.len() <= TEXT_LIMIT, "Android text exceeds 1 MiB");
            self.heap.get_mut(receiver)?.data = Data::Spanned { text, spans };
            return Ok(Some(vec![]));
        }

        let Some(receiver) = args.first().copied() else {
            return Ok(None);
        };
        if !matches!(self.heap.get(receiver)?.data, Data::Spanned { .. }) {
            return Ok(None);
        }

        let signature = method.signature();
        let mut result = vec![];
        match signature.as_str() {
            "toString()Ljava/lang/String;" => {
                result.push(self.heap.string(self.heap.text(receiver)?.to_owned())?);
            }
            "length()I" => {
                result.push(Word::from(utf16_len(self.heap.text(receiver)?) as i32));
            }
            "charAt(I)C" => {
                let units: Vec<u16> = self.heap.text(receiver)?.encode_utf16().collect();
                let index = args.get(1).context("charAt index missing")?.int()?;
                let value = units.get(index as usize).ok_or_else(|| {
                    fault(
                        "Ljava/lang/IndexOutOfBoundsException;",
                        format!("index {index} for length {}", units.len()),
                    )
                })?;
                result.push(Word::from(i32::from(*value)));
            }
            "subSequence(II)Ljava/lang/CharSequence;" => {
                let (text, spans) = self.sequence_data(receiver)?;
                let (start, end) = checked_range(
                    args.get(1).context("subSequence start missing")?.int()?,
                    args.get(2).context("subSequence end missing")?.int()?,
                    utf16_len(&text),
                )?;
                let sub = self.heap.instance("Landroid/text/SpannedString;")?;
                self.heap.get_mut(sub)?.data = Data::Spanned {
                    text: slice_utf16(&text, start as i32, end as i32)?,
                    spans: slice_spans(&spans, start, end),
                };
                result.push(sub);
            }
            "append(Ljava/lang/CharSequence;)Landroid/text/SpannableStringBuilder;"
            | "append(Ljava/lang/CharSequence;)Landroid/text/Editable;" => {
                let (source, source_spans) = self.sequence_data_or_null(args[1])?;
                self.append_spanned(receiver, source, source_spans)?;
                result.push(receiver);
            }
            "append(Ljava/lang/CharSequence;II)Landroid/text/SpannableStringBuilder;" => {
                let (source, spans) = self.sequence_data_or_null(args[1])?;
                let (start, end) =
                    checked_range(args[2].int()?, args[3].int()?, utf16_len(&source))?;
                self.append_spanned(
                    receiver,
                    slice_utf16(&source, start as i32, end as i32)?,
                    slice_spans(&spans, start, end),
                )?;
                result.push(receiver);
            }
            "append(C)Landroid/text/SpannableStringBuilder;" => {
                let unit = args[1].int()? as u16;
                let source = String::from_utf16_lossy(&[unit]);
                self.append_spanned(receiver, source, vec![])?;
                result.push(receiver);
            }
            "setSpan(Ljava/lang/Object;III)V" => {
                let object = args[1];
                if object != Word::ZERO {
                    self.heap.get(object)?;
                }
                let length = utf16_len(self.heap.text(receiver)?);
                let (start, end) = checked_range(args[2].int()?, args[3].int()?, length)?;
                let spans = match &mut self.heap.get_mut(receiver)?.data {
                    Data::Spanned { spans, .. } => spans,
                    _ => unreachable!(),
                };
                spans.retain(|span| span.object != object);
                spans.push(TextSpan {
                    object,
                    start,
                    end,
                    flags: args[4].int()?,
                });
            }
            "removeSpan(Ljava/lang/Object;)V" => {
                let object = args[1];
                if let Data::Spanned { spans, .. } = &mut self.heap.get_mut(receiver)?.data {
                    spans.retain(|span| span.object != object);
                }
            }
            "getSpanStart(Ljava/lang/Object;)I"
            | "getSpanEnd(Ljava/lang/Object;)I"
            | "getSpanFlags(Ljava/lang/Object;)I" => {
                let object = args[1];
                let span = self
                    .sequence_data(receiver)?
                    .1
                    .into_iter()
                    .find(|span| span.object == object);
                let value = span.map_or(-1, |span| match method.name.as_str() {
                    "getSpanStart" => span.start as i32,
                    "getSpanEnd" => span.end as i32,
                    _ => span.flags,
                });
                result.push(Word::from(value));
            }
            "getSpans(IILjava/lang/Class;)[Ljava/lang/Object;" => {
                let (text, spans) = self.sequence_data(receiver)?;
                let (start, end) = checked_range(args[1].int()?, args[2].int()?, utf16_len(&text))?;
                let filter = self.class_filter(args[3])?;
                let matches: Vec<Word> = spans
                    .into_iter()
                    .filter(|span| {
                        let overlaps = if start == end {
                            span.start <= start && start <= span.end
                        } else if span.start == span.end {
                            start <= span.start && span.start <= end
                        } else {
                            span.start < end && span.end > start
                        };
                        overlaps
                            && filter.as_deref().is_none_or(|target| {
                                self.heap
                                    .get(span.object)
                                    .is_ok_and(|object| self.is_a(&object.class, target))
                            })
                    })
                    .map(|span| span.object)
                    .collect();
                let element = filter.unwrap_or_else(|| "Ljava/lang/Object;".into());
                let array = self.array(element, matches.len())?;
                if let Data::Array { values, .. } = &mut self.heap.get_mut(array)?.data {
                    for (slot, value) in values.iter_mut().zip(matches) {
                        *slot = vec![value];
                    }
                }
                result.push(array);
            }
            "nextSpanTransition(IILjava/lang/Class;)I" => {
                let (text, spans) = self.sequence_data(receiver)?;
                let start = args[1].int()?;
                let limit = args[2].int()?;
                let (start, limit) = checked_range(start, limit, utf16_len(&text))?;
                let filter = self.class_filter(args[3])?;
                let next = spans
                    .into_iter()
                    .filter(|span| {
                        filter.as_deref().is_none_or(|target| {
                            self.heap
                                .get(span.object)
                                .is_ok_and(|object| self.is_a(&object.class, target))
                        })
                    })
                    .flat_map(|span| [span.start, span.end])
                    .filter(|position| *position > start && *position < limit)
                    .min()
                    .unwrap_or(limit);
                result.push(Word::from(next as i32));
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }

    fn sequence_data(&self, word: Word) -> Result<(String, Vec<TextSpan>)> {
        match &self.heap.get(word)?.data {
            Data::String(text) | Data::Builder(text) => Ok((text.clone(), vec![])),
            Data::Spanned { text, spans } => Ok((text.clone(), spans.clone())),
            _ => bail!("expected CharSequence, got {}", self.heap.get(word)?.class),
        }
    }

    fn sequence_data_or_null(&self, word: Word) -> Result<(String, Vec<TextSpan>)> {
        if word == Word::ZERO {
            Ok(("null".into(), vec![]))
        } else {
            self.sequence_data(word)
        }
    }

    fn class_filter(&self, class: Word) -> Result<Option<String>> {
        if class == Word::ZERO {
            return Ok(None);
        }
        let name = *self
            .heap
            .get(class)?
            .fields
            .get("name")
            .and_then(|words| words.first())
            .context("Class object has no descriptor")?;
        Ok(Some(self.heap.text(name)?.to_owned()))
    }

    fn append_spanned(
        &mut self,
        receiver: Word,
        suffix: String,
        mut suffix_spans: Vec<TextSpan>,
    ) -> Result<()> {
        let (mut text, mut spans) = self.sequence_data(receiver)?;
        ensure!(
            text.len() + suffix.len() <= TEXT_LIMIT,
            "Editable text exceeds 1 MiB"
        );
        let offset = utf16_len(&text);
        for span in &mut suffix_spans {
            span.start += offset;
            span.end += offset;
        }
        text.push_str(&suffix);
        spans.extend(suffix_spans);
        if let Some(owner) = self
            .heap
            .get(receiver)?
            .fields
            .get("droidless:text:owner")
            .and_then(|v| v.first())
            .copied()
        {
            self.set_view_text(owner, text, spans)?;
        } else {
            self.heap.get_mut(receiver)?.data = Data::Spanned { text, spans };
        }
        Ok(())
    }

    pub(crate) fn editable_text(&mut self, view: Word) -> Result<Word> {
        let class = self.heap.get(view)?.class.clone();
        ensure!(
            self.is_a(&class, "Landroid/widget/EditText;"),
            "View is not an EditText"
        );
        if let Some(editable) = self
            .heap
            .get(view)?
            .fields
            .get(EDITABLE_FIELD)
            .and_then(|words| words.first())
        {
            return Ok(*editable);
        }
        let text = self
            .heap
            .get(view)?
            .view
            .as_ref()
            .context("EditText has no View state")?
            .text
            .clone();
        let editable = self
            .heap
            .instance("Landroid/text/SpannableStringBuilder;")?;
        self.heap.get_mut(editable)?.data = Data::Spanned {
            text,
            spans: vec![],
        };
        self.heap
            .get_mut(editable)?
            .fields
            .insert("droidless:text:owner".into(), vec![view]);
        self.heap
            .get_mut(view)?
            .fields
            .insert(EDITABLE_FIELD.into(), vec![editable]);
        Ok(editable)
    }

    pub(crate) fn set_text_view(&mut self, view: Word, value: Word) -> Result<()> {
        if value == Word::ZERO {
            self.set_view_text(view, String::new(), vec![])
        } else {
            let (text, spans) = self.sequence_data(value)?;
            self.set_view_text(view, text, spans)
        }
    }

    pub(crate) fn append_text_view(&mut self, view: Word, value: Word) -> Result<()> {
        let (mut text, mut spans) = {
            let object = self.heap.get(view)?;
            let current = object
                .fields
                .get(EDITABLE_FIELD)
                .and_then(|words| words.first())
                .copied();
            if let Some(current) = current {
                self.sequence_data(current)?
            } else {
                (
                    object
                        .view
                        .as_ref()
                        .context("TextView has no View state")?
                        .text
                        .clone(),
                    vec![],
                )
            }
        };
        let (suffix, mut suffix_spans) = self.sequence_data_or_null(value)?;
        ensure!(
            text.len() + suffix.len() <= TEXT_LIMIT,
            "TextView text exceeds 1 MiB"
        );
        let offset = utf16_len(&text);
        for span in &mut suffix_spans {
            span.start += offset;
            span.end += offset;
        }
        text.push_str(&suffix);
        spans.extend(suffix_spans);
        self.set_view_text(view, text, spans)
    }

    pub(crate) fn set_view_text(
        &mut self,
        view: Word,
        text: String,
        spans: Vec<TextSpan>,
    ) -> Result<()> {
        ensure!(text.len() <= TEXT_LIMIT, "TextView text exceeds 1 MiB");
        let text_length = utf16_len(&text);
        ensure!(
            spans
                .iter()
                .all(|span| span.start <= span.end && span.end <= text_length),
            "TextView span range exceeds text"
        );
        let class = self.heap.get(view)?.class.clone();
        let is_edit_text = self.is_a(&class, "Landroid/widget/EditText;");
        let editable = self
            .heap
            .get(view)?
            .fields
            .get(EDITABLE_FIELD)
            .and_then(|words| words.first())
            .copied();
        self.heap
            .get_mut(view)?
            .view
            .as_mut()
            .context("not a View")?
            .text = text.clone();
        self.invalidate_text_layout(view)?;
        if is_edit_text || editable.is_some() {
            let editable = match editable {
                Some(editable) => editable,
                None => self
                    .heap
                    .instance("Landroid/text/SpannableStringBuilder;")?,
            };
            self.heap.get_mut(editable)?.data = Data::Spanned { text, spans };
            self.heap
                .get_mut(editable)?
                .fields
                .insert("droidless:text:owner".into(), vec![view]);
            self.heap
                .get_mut(view)?
                .fields
                .insert(EDITABLE_FIELD.into(), vec![editable]);
        }
        Ok(())
    }
}
