use crate::heap::{Heap, Word};
use anyhow::{Context, Result, ensure};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct View {
    pub kind: String,
    pub id: u32,
    pub text: String,
    pub children: Vec<Word>,
    pub listener: Option<Word>,
    pub key_listener: Option<Word>,
    pub xml_click: Option<String>,
    pub orientation: i32,
    pub width: f32,
    pub height: f32,
    pub weight: f32,
    pub padding: f32,
    pub margins: [f32; 4],
    pub visible: i32,
    pub enabled: bool,
    pub editable: bool,
    pub text_size: f32,
    pub text_color: u32,
    pub background: Option<u32>,
    pub gravity: u32,
}
impl View {
    pub fn for_class(class: &str) -> Option<Self> {
        let name = class
            .strip_prefix("Landroid/widget/")
            .and_then(|c| c.strip_suffix(';'))
            .or_else(|| {
                if class == "Landroid/view/View;" || class == "Landroid/view/ViewGroup;" {
                    Some("View")
                } else {
                    None
                }
            })?;
        if ![
            "View",
            "LinearLayout",
            "FrameLayout",
            "TextView",
            "Button",
            "EditText",
            "TableLayout",
            "TableRow",
        ]
        .contains(&name)
        {
            return None;
        }
        Some(Self {
            kind: name.to_owned(),
            id: 0,
            text: String::new(),
            children: vec![],
            listener: None,
            key_listener: None,
            xml_click: None,
            orientation: if name == "TableLayout" { 1 } else { 0 },
            width: -2.0,
            height: -2.0,
            weight: 0.0,
            padding: 0.0,
            margins: [0.0; 4],
            visible: 0,
            enabled: true,
            editable: name == "EditText",
            text_size: 18.0,
            text_color: 0xff222222,
            background: None,
            gravity: 0,
        })
    }
}

#[derive(Clone, Copy, Debug, Serialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
#[derive(Debug, Serialize)]
pub struct Node {
    pub handle: usize,
    pub view: View,
    pub rect: Rect,
    pub children: Vec<Node>,
}

pub fn layout(heap: &Heap, root: Word, width: f32, height: f32) -> Result<Node> {
    build(
        heap,
        root,
        Rect {
            x: 0.0,
            y: 0.0,
            width,
            height,
        },
        &mut vec![],
    )
}
fn build(heap: &Heap, word: Word, rect: Rect, path: &mut Vec<usize>) -> Result<Node> {
    let handle = word.reference()?;
    ensure!(
        path.len() < 128 && !path.contains(&handle),
        "cyclic or too deep View hierarchy"
    );
    path.push(handle);
    let view = heap
        .get(word)?
        .view
        .as_ref()
        .context("expected a View")?
        .clone();
    let mut children = vec![];
    let available = Rect {
        x: rect.x + view.padding,
        y: rect.y + view.padding,
        width: (rect.width - 2.0 * view.padding).max(0.0),
        height: (rect.height - 2.0 * view.padding).max(0.0),
    };
    if view.kind == "FrameLayout" || view.kind == "View" {
        for child in &view.children {
            let c = heap.get(*child)?.view.as_ref().context("non-View child")?;
            if c.visible == 8 {
                continue;
            }
            children.push(build(
                heap,
                *child,
                Rect {
                    x: available.x + c.margins[0],
                    y: available.y + c.margins[1],
                    width: dimension(heap, *child, true, available.width)?,
                    height: dimension(heap, *child, false, available.height)?,
                },
                path,
            )?);
        }
    } else {
        let vertical = view.orientation == 1;
        let size = if vertical {
            available.height
        } else {
            available.width
        };
        let mut weights = 0.0;
        let mut fixed = 0.0;
        for child in &view.children {
            let c = heap.get(*child)?.view.as_ref().context("non-View child")?;
            if c.visible == 8 {
                continue;
            }
            weights += c.weight;
            fixed += if c.weight > 0.0 {
                0.0
            } else {
                dimension(heap, *child, !vertical, size)?
            };
            fixed += if vertical {
                c.margins[1] + c.margins[3]
            } else {
                c.margins[0] + c.margins[2]
            };
        }
        let mut cursor = if vertical { available.y } else { available.x };
        for child in &view.children {
            let c = heap.get(*child)?.view.as_ref().context("non-View child")?;
            if c.visible == 8 {
                continue;
            }
            let length = if c.weight > 0.0 {
                (size - fixed).max(0.0) * c.weight / weights
            } else {
                dimension(heap, *child, !vertical, size)?
            };
            let (x, y, w, h) = if vertical {
                cursor += c.margins[1];
                let r = (
                    available.x + c.margins[0],
                    cursor,
                    dimension(heap, *child, true, available.width)?,
                    length,
                );
                cursor += length + c.margins[3];
                r
            } else {
                cursor += c.margins[0];
                let r = (
                    cursor,
                    available.y + c.margins[1],
                    length,
                    dimension(heap, *child, false, available.height)?,
                );
                cursor += length + c.margins[2];
                r
            };
            children.push(build(
                heap,
                *child,
                Rect {
                    x,
                    y,
                    width: w,
                    height: h,
                },
                path,
            )?);
        }
    }
    path.pop();
    Ok(Node {
        handle,
        view,
        rect,
        children,
    })
}
fn dimension(heap: &Heap, word: Word, horizontal: bool, parent: f32) -> Result<f32> {
    fn measure(
        heap: &Heap,
        word: Word,
        horizontal: bool,
        parent: f32,
        depth: usize,
    ) -> Result<f32> {
        ensure!(depth < 128, "View measurement nesting limit");
        let v = heap.get(word)?.view.as_ref().context("expected View")?;
        let value = if horizontal { v.width } else { v.height };
        if value == -1.0 {
            return Ok(parent);
        }
        if value >= 0.0 {
            return Ok(value.min(parent));
        }
        if v.children.is_empty() {
            return Ok(if horizontal {
                (v.text.chars().count() as f32 * v.text_size * 0.6 + 24.0)
                    .max(48.0)
                    .min(parent)
            } else {
                (v.text_size + 24.0).max(44.0).min(parent)
            });
        }
        let sizes = v
            .children
            .iter()
            .map(|c| measure(heap, *c, horizontal, parent, depth + 1))
            .collect::<Result<Vec<_>>>()?;
        let total = if horizontal == (v.orientation == 0) {
            sizes.iter().sum()
        } else {
            sizes.iter().copied().fold(0.0, f32::max)
        };
        Ok((total + 2.0 * v.padding).min(parent))
    }
    measure(heap, word, horizontal, parent, 0)
}
