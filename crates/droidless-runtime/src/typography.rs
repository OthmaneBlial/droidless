use crate::{
    heap::{Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::{Field, Method};

const TYPEFACE: &str = "Landroid/graphics/Typeface;";
const FACE: &str = "droidless:font:face";
const FAMILY: &str = "droidless:font:family";
const STYLE: &str = "droidless:font:style";

impl Runtime {
    pub(crate) fn typeface_field(&self, field: &Field) -> bool {
        field.class == TYPEFACE
            && self.class_location(TYPEFACE).is_none()
            && matches!(
                field.name.as_str(),
                "DEFAULT" | "DEFAULT_BOLD" | "SANS_SERIF" | "SERIF" | "MONOSPACE"
            )
    }
    pub(crate) fn typeface_field_object(&mut self, field: &Field) -> Result<Word> {
        let family = match field.name.as_str() {
            "SERIF" => 1,
            "MONOSPACE" => 2,
            _ => 0,
        };
        self.typeface_object(family, i32::from(field.name == "DEFAULT_BOLD"))
    }
    fn typeface_object(&mut self, family: i32, style: i32) -> Result<Word> {
        ensure!(
            (0..=2).contains(&family) && (0..=3).contains(&style),
            "invalid native typeface profile"
        );
        let key = format!("droidless:typeface:{family}:{style}");
        if let Some(face) = self.statics.get(&key).and_then(|words| words.first()) {
            return Ok(*face);
        }
        let face = self.heap.instance(TYPEFACE)?;
        let fields = &mut self.heap.get_mut(face)?.fields;
        fields.insert(FAMILY.into(), vec![Word::from(family)]);
        fields.insert(STYLE.into(), vec![Word::from(style)]);
        self.statics.insert(key, vec![face]);
        Ok(face)
    }
    fn typeface_values(&self, face: Word) -> Result<(i32, i32)> {
        if face == Word::ZERO {
            return Ok((0, 0));
        }
        let object = self.heap.get(face)?;
        ensure!(self.is_a(&object.class, TYPEFACE), "expected Typeface");
        let value = |key| {
            object
                .fields
                .get(key)
                .and_then(|words| words.first())
                .context("uninitialized Typeface")?
                .int()
        };
        let (family, style) = (value(FAMILY)?, value(STYLE)?);
        ensure!(
            (0..=2).contains(&family) && (0..=3).contains(&style),
            "invalid native typeface profile"
        );
        Ok((family, style))
    }
    pub(crate) fn typography_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method.signature();
        let arg = |index| {
            args.get(index)
                .copied()
                .context("missing typography argument")
        };
        let result = match (method.class.as_str(), signature.as_str()) {
            (
                TYPEFACE,
                "create(Ljava/lang/String;I)Landroid/graphics/Typeface;"
                | "create(Landroid/graphics/Typeface;I)Landroid/graphics/Typeface;",
            ) => {
                ensure!(args.len() == 2, "invalid Typeface.create arguments");
                let requested = arg(1)?.int()?;
                let style = if (0..=3).contains(&requested) {
                    requested
                } else {
                    0
                };
                let family = if method.parameters[0] == "Ljava/lang/String;" {
                    if arg(0)? == Word::ZERO {
                        0
                    } else {
                        match self.heap.text(arg(0)?)? {
                            "serif" => 1,
                            "monospace" => 2,
                            _ => 0,
                        }
                    }
                } else {
                    let (family, old_style) = self.typeface_values(arg(0)?)?;
                    if arg(0)? != Word::ZERO && old_style == style {
                        return Ok(Some(vec![arg(0)?]));
                    }
                    family
                };
                // ponytail: three host-backed families; asset fonts and Android font-map parity remain ahead.
                vec![self.typeface_object(family, style)?]
            }
            (TYPEFACE, "defaultFromStyle(I)Landroid/graphics/Typeface;") => {
                ensure!(args.len() == 1, "invalid defaultFromStyle arguments");
                let style = arg(0)?.int()?;
                if !(0..=3).contains(&style) {
                    return Err(fault(
                        "Ljava/lang/ArrayIndexOutOfBoundsException;",
                        "typeface style",
                    ));
                }
                vec![self.typeface_object(0, style)?]
            }
            (TYPEFACE, "getStyle()I" | "isBold()Z" | "isItalic()Z") => {
                ensure!(args.len() == 1, "invalid Typeface getter arguments");
                if arg(0)? == Word::ZERO {
                    return Err(fault("Ljava/lang/NullPointerException;", "null Typeface"));
                }
                let (_, style) = self.typeface_values(arg(0)?)?;
                vec![Word::from(match method.name.as_str() {
                    "isBold" => i32::from(style & 1 != 0),
                    "isItalic" => i32::from(style & 2 != 0),
                    _ => style,
                })]
            }
            (
                "Landroid/graphics/Paint;" | "Landroid/widget/TextView;",
                "getTypeface()Landroid/graphics/Typeface;",
            ) => {
                ensure!(args.len() == 1, "invalid getTypeface arguments");
                if method.class == "Landroid/widget/TextView;" {
                    self.require_main_thread()?;
                    self.view_mut(arg(0)?)?;
                }
                vec![
                    self.heap
                        .get(arg(0)?)?
                        .fields
                        .get(FACE)
                        .and_then(|words| words.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                ]
            }
            (
                "Landroid/graphics/Paint;",
                "setTypeface(Landroid/graphics/Typeface;)Landroid/graphics/Typeface;",
            )
            | ("Landroid/widget/TextView;", "setTypeface(Landroid/graphics/Typeface;)V") => {
                ensure!(args.len() == 2, "invalid setTypeface arguments");
                let (family, style) = self.typeface_values(arg(1)?)?;
                let text = method.class == "Landroid/widget/TextView;";
                if text {
                    self.require_main_thread()?;
                    self.view_mut(arg(0)?)?;
                }
                let old = self
                    .heap
                    .get(arg(0)?)?
                    .fields
                    .get(FACE)
                    .and_then(|words| words.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                self.heap
                    .get_mut(arg(0)?)?
                    .fields
                    .insert(FACE.into(), vec![arg(1)?]);
                if text && old != arg(1)? {
                    let view = self.view_mut(arg(0)?)?;
                    view.font_family = family as u32;
                    view.font_style = style as u32;
                    self.invalidate_text_layout(arg(0)?)?;
                }
                if text { vec![] } else { vec![arg(1)?] }
            }
            _ => return Ok(None),
        };
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        Ok(Some(result))
    }
}
