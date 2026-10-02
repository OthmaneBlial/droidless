use crate::heap::{Heap, Word};
use anyhow::{Context, Result, ensure};
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct View {
    pub kind: String,
    pub id: u32,
    pub text: String,
    pub content_description: Option<String>,
    pub children: Vec<Word>,
    pub listener: Option<Word>,
    pub key_listener: Option<Word>,
    pub xml_click: Option<String>,
    pub orientation: i32,
    pub width: f32,
    pub height: f32,
    pub weight: f32,
    pub padding: [f32; 4],
    pub clip_children: bool,
    pub clip_to_padding: bool,
    pub margins: [f32; 4],
    pub visible: i32,
    pub enabled: bool,
    pub editable: bool,
    pub text_size: f32,
    pub font_family: u32,
    pub font_style: u32,
    pub alpha: f32,
    pub text_color: u32,
    pub background: Option<u32>,
    pub foreground_overlay: Option<u32>,
    pub foreground_padding: [f32; 4],
    pub gravity: u32,
    pub image_scale: i32,
    pub grid: Option<Grid>,
    #[serde(skip)]
    pub image: Option<Vec<u8>>,
}
impl View {
    pub fn for_class(class: &str) -> Option<Self> {
        let name = match class {
            "Landroid/support/v7/widget/FitWindowsFrameLayout;"
            | "Landroid/support/v7/widget/ContentFrameLayout;"
            | "Landroid/support/v4/widget/DrawerLayout;"
            | "Landroid/support/design/widget/CoordinatorLayout;"
            | "Landroid/support/design/widget/AppBarLayout;"
            | "Landroid/support/design/widget/NavigationView;"
            | "Landroid/support/design/internal/NavigationMenuView;"
            | "Landroid/support/v7/widget/RecyclerView;"
            | "Landroid/support/v7/widget/Toolbar;"
            | "Landroid/widget/RelativeLayout;"
            | "Landroid/widget/ScrollView;"
            | "Landroid/widget/HorizontalScrollView;" => "FrameLayout",
            "Landroid/support/design/widget/FloatingActionButton;"
            | "Landroid/widget/ImageButton;" => "Button",
            "Landroid/support/v7/widget/AppCompatTextView;"
            | "Landroid/widget/CheckedTextView;" => "TextView",
            "Landroid/support/v7/widget/AppCompatEditText;" => "EditText",
            "Landroid/widget/ImageView;" => "ImageView",
            "Landroid/widget/Space;" => "View",
            "Landroid/support/v7/widget/ViewStubCompat;" | "Landroid/view/ViewStub;" => "View",
            _ => class
                .strip_prefix("Landroid/widget/")
                .and_then(|c| c.strip_suffix(';'))
                .or_else(|| {
                    if class == "Landroid/view/View;" || class == "Landroid/view/ViewGroup;" {
                        Some("View")
                    } else {
                        None
                    }
                })?,
        };
        if ![
            "View",
            "LinearLayout",
            "FrameLayout",
            "ImageView",
            "TextView",
            "Button",
            "EditText",
            "TableLayout",
            "TableRow",
            "GridView",
        ]
        .contains(&name)
        {
            return None;
        }
        Some(Self {
            kind: name.to_owned(),
            id: 0,
            text: if class == "Landroid/support/design/widget/FloatingActionButton;" {
                "＋".into()
            } else {
                String::new()
            },
            children: vec![],
            content_description: None,
            listener: None,
            key_listener: None,
            xml_click: None,
            orientation: if name == "TableLayout" { 1 } else { 0 },
            width: -2.0,
            height: -2.0,
            weight: 0.0,
            padding: [0.0; 4],
            clip_children: true,
            clip_to_padding: true,
            margins: [0.0; 4],
            visible: 0,
            enabled: true,
            editable: name == "EditText",
            text_size: 18.0,
            font_family: 0,
            font_style: 0,
            alpha: 1.0,
            text_color: 0xff222222,
            background: None,
            foreground_overlay: None,
            foreground_padding: [0.0; 4],
            gravity: 0,
            image_scale: 3,
            grid: (name == "GridView").then(Grid::default),
            image: None,
        })
    }
}

fn content_padding(view: &View) -> [f32; 4] {
    std::array::from_fn(|edge| view.padding[edge].max(view.foreground_padding[edge]))
}

#[derive(Clone, Debug, Serialize)]
pub struct Grid {
    pub columns: i32,
    pub column_width: i32,
    pub horizontal_spacing: i32,
    pub vertical_spacing: i32,
    pub stretch: i32,
}
impl Default for Grid {
    fn default() -> Self {
        Self {
            columns: 1,
            column_width: 0,
            horizontal_spacing: 0,
            vertical_spacing: 0,
            stretch: 2,
        }
    }
}
pub(crate) struct GridMetrics {
    pub columns: usize,
    pub width: f32,
    pub spacing: f32,
    pub inset: f32,
}
pub(crate) fn grid_metrics(view: &View, available: f32) -> Result<GridMetrics> {
    let grid = view.grid.as_ref().context("expected GridView")?;
    let available = available.max(0.0) as i64;
    let (width, spacing) = (
        i64::from(grid.column_width),
        i64::from(grid.horizontal_spacing),
    );
    let columns = if grid.columns == -1 {
        if width > 0 {
            (available + spacing)
                .checked_div(width + spacing)
                .context("invalid grid column spacing")?
        } else {
            2
        }
    } else {
        i64::from(grid.columns)
    }
    .max(1);
    ensure!(columns <= 1024, "GridView column limit exceeded");
    let extra = available - columns * width - (columns - 1) * spacing;
    let (width, spacing) = match grid.stretch {
        0 => (width, spacing),
        1 => (width, spacing + extra / (columns - 1).max(1)),
        2 => (width + extra / columns, spacing),
        3 => (
            width,
            spacing + extra / if columns > 1 { columns + 1 } else { 1 },
        ),
        _ => anyhow::bail!("unsupported GridView stretch mode"),
    };
    ensure!(width >= 0, "negative GridView column width is unsupported");
    Ok(GridMetrics {
        columns: columns as usize,
        width: width as f32,
        spacing: spacing as f32,
        inset: if grid.stretch == 3 {
            spacing as f32
        } else {
            0.0
        },
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}
impl Rect {
    pub fn intersection(self, other: Self) -> Self {
        let x = self.x.max(other.x);
        let y = self.y.max(other.y);
        Self {
            x,
            y,
            width: ((self.x + self.width).min(other.x + other.width) - x).max(0.0),
            height: ((self.y + self.height).min(other.y + other.height) - y).max(0.0),
        }
    }
}
#[derive(Debug, Serialize)]
pub struct Node {
    pub handle: usize,
    pub view: View,
    pub rect: Rect,
    pub paint_clip: Rect,
    pub input_clip: Rect,
    pub children: Vec<Node>,
}

pub(crate) fn params_field(heap: &Heap, word: Word, field: &str) -> Result<Option<Word>> {
    let params = heap
        .get(word)?
        .fields
        .get("droidless:view:layout-params")
        .and_then(|v| v.first())
        .copied()
        .unwrap_or(Word::ZERO);
    if params == Word::ZERO {
        return Ok(None);
    }
    Ok(Some(
        heap.get(params)?
            .fields
            .get(field)
            .and_then(|v| v.first())
            .copied()
            .unwrap_or(Word::ZERO),
    ))
}

pub(crate) fn weight(heap: &Heap, word: Word) -> Result<f32> {
    let value = if let Some(value) = params_field(
        heap,
        word,
        "Landroid/widget/LinearLayout$LayoutParams;->weight:F",
    )? {
        f32::from_bits(value.int()? as u32)
    } else {
        heap.get(word)?
            .view
            .as_ref()
            .context("expected View")?
            .weight
    };
    ensure!(value.is_finite() && value >= 0.0, "invalid layout weight");
    Ok(value)
}

pub(crate) fn margins(heap: &Heap, word: Word) -> Result<[f32; 4]> {
    let object = heap.get(word)?;
    let mut margins = object.view.as_ref().context("expected View")?.margins;
    if let Some(params) = object
        .fields
        .get("droidless:view:layout-params")
        .and_then(|v| v.first())
        .filter(|v| **v != Word::ZERO)
    {
        let fields = &heap.get(*params)?.fields;
        for (edge, name) in ["leftMargin", "topMargin", "rightMargin", "bottomMargin"]
            .into_iter()
            .enumerate()
        {
            if let Some(value) = fields
                .get(&format!(
                    "Landroid/view/ViewGroup$MarginLayoutParams;->{name}:I"
                ))
                .and_then(|v| v.first())
            {
                margins[edge] = value.int()? as f32;
            }
        }
    }
    Ok(margins)
}

pub fn layout(heap: &Heap, root: Word, width: f32, height: f32) -> Result<Node> {
    let viewport = Rect {
        x: 0.0,
        y: 0.0,
        width,
        height,
    };
    let mut tree = build(heap, root, viewport, &mut vec![])?;
    apply_clips(&mut tree, viewport, viewport, true);
    Ok(tree)
}
fn apply_clips(node: &mut Node, paint: Rect, input: Rect, clip_child: bool) {
    node.paint_clip = if clip_child {
        paint.intersection(node.rect)
    } else {
        paint
    };
    // Android touch targeting follows ancestor bounds, independently of drawing flags/padding.
    node.input_clip = input.intersection(node.rect);
    let [left, top, right, bottom] = node.view.padding;
    let children_paint = if node.view.clip_to_padding && node.view.padding.iter().any(|p| *p != 0.0)
    {
        node.paint_clip.intersection(Rect {
            x: node.rect.x + left,
            y: node.rect.y + top,
            width: (node.rect.width - left - right).max(0.0),
            height: (node.rect.height - top - bottom).max(0.0),
        })
    } else {
        node.paint_clip
    };
    for child in &mut node.children {
        apply_clips(
            child,
            children_paint,
            node.input_clip,
            node.view.clip_children,
        );
    }
}
fn laid_out_rect(heap: &Heap, word: Word, parent: Rect) -> Result<Option<Rect>> {
    let fields = &heap.get(word)?.fields;
    if !fields
        .get("droidless:view:laid-out")
        .and_then(|values| values.first())
        .is_some_and(|value| value.truth())
    {
        return Ok(None);
    }
    let edge = |name: &str| -> Result<f32> {
        Ok(fields
            .get(&format!("droidless:view:{name}"))
            .and_then(|values| values.first())
            .copied()
            .unwrap_or(Word::ZERO)
            .int()? as f32)
    };
    let (left, top, right, bottom) = (edge("left")?, edge("top")?, edge("right")?, edge("bottom")?);
    Ok(Some(Rect {
        x: parent.x + left,
        y: parent.y + top,
        width: (right - left).max(0.0),
        height: (bottom - top).max(0.0),
    }))
}
fn build(heap: &Heap, word: Word, mut rect: Rect, path: &mut Vec<usize>) -> Result<Node> {
    let handle = word.reference()?;
    ensure!(
        path.len() < 128 && !path.contains(&handle),
        "cyclic or too deep View hierarchy"
    );
    path.push(handle);
    for (key, coordinate) in [
        ("droidless:view:translation-x", &mut rect.x),
        ("droidless:view:translation-y", &mut rect.y),
    ] {
        if let Some(value) = heap.get(word)?.fields.get(key).and_then(|v| v.first()) {
            let offset = f32::from_bits(value.int()? as u32);
            ensure!(
                offset.is_finite() && offset.abs() <= 1_000_000.0,
                "invalid View translation"
            );
            *coordinate += offset;
        }
    }
    let mut view = heap
        .get(word)?
        .view
        .as_ref()
        .context("expected a View")?
        .clone();
    for (field, size) in [("width", &mut view.width), ("height", &mut view.height)] {
        if let Some(value) = params_field(
            heap,
            word,
            &format!("Landroid/view/ViewGroup$LayoutParams;->{field}:I"),
        )? {
            *size = value.int()? as f32;
        }
    }
    view.weight = weight(heap, word)?;
    view.margins = margins(heap, word)?;
    view.padding = content_padding(&view);
    let mut children = vec![];
    let available = Rect {
        x: rect.x + view.padding[0],
        y: rect.y + view.padding[1],
        width: (rect.width - view.padding[0] - view.padding[2]).max(0.0),
        height: (rect.height - view.padding[1] - view.padding[3]).max(0.0),
    };
    if let Some(grid) = &view.grid {
        let metrics = grid_metrics(&view, available.width)?;
        let mut top = available.y;
        for row in view.children.chunks(metrics.columns) {
            let mut row_height: f32 = 0.0;
            for (column, child) in row.iter().enumerate() {
                let width = dimension(heap, *child, true, metrics.width)?;
                let height = dimension(heap, *child, false, available.height)?;
                row_height = row_height.max(height);
                let alignment = match view.gravity & 7 {
                    1 => (metrics.width - width) / 2.0,
                    5 => metrics.width - width,
                    _ => 0.0,
                };
                children.push(build(
                    heap,
                    *child,
                    Rect {
                        x: available.x
                            + metrics.inset
                            + column as f32 * (metrics.width + metrics.spacing)
                            + alignment,
                        y: top,
                        width,
                        height,
                    },
                    path,
                )?);
            }
            top += row_height + grid.vertical_spacing as f32;
        }
    } else if view.kind == "FrameLayout" || view.kind == "View" {
        for child in &view.children {
            let object = heap.get(*child)?;
            let c = object.view.as_ref().context("non-View child")?;
            let margins = margins(heap, *child)?;
            if c.visible == 8 {
                continue;
            }
            let gravity = params_field(
                heap,
                *child,
                "Landroid/widget/FrameLayout$LayoutParams;->gravity:I",
            )?
            .or_else(|| {
                object
                    .fields
                    .get("droidless:view:layout-gravity")?
                    .first()
                    .copied()
            })
            .unwrap_or(Word::from(-1))
            .int()?;
            let gravity = if gravity == -1 { 0x33 } else { gravity };
            let width = dimension(heap, *child, true, available.width)?;
            let height = dimension(heap, *child, false, available.height)?;
            // Relative START/END follow this profile's default left-to-right direction.
            let x = match gravity & 7 {
                1 => available.x + (available.width - width) / 2.0 + margins[0] - margins[2],
                5 => available.x + available.width - width - margins[2],
                _ => available.x + margins[0],
            };
            let y = match gravity & 0x70 {
                0x10 => available.y + (available.height - height) / 2.0 + margins[1] - margins[3],
                0x50 => available.y + available.height - height - margins[3],
                _ => available.y + margins[1],
            };
            let child_rect = laid_out_rect(heap, *child, rect)?.unwrap_or(Rect {
                x,
                y,
                width,
                height,
            });
            children.push(build(heap, *child, child_rect, path)?);
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
            let margins = margins(heap, *child)?;
            if c.visible == 8 {
                continue;
            }
            let child_weight = weight(heap, *child)?;
            let measured = measured_dimension(heap, *child, !vertical)?.is_some();
            weights += if measured { 0.0 } else { child_weight };
            fixed += if child_weight > 0.0 && !measured {
                0.0
            } else {
                dimension(heap, *child, !vertical, size)?
            };
            fixed += if vertical {
                margins[1] + margins[3]
            } else {
                margins[0] + margins[2]
            };
        }
        let mut cursor = if vertical { available.y } else { available.x };
        for child in &view.children {
            let c = heap.get(*child)?.view.as_ref().context("non-View child")?;
            let margins = margins(heap, *child)?;
            if c.visible == 8 {
                continue;
            }
            let child_weight = weight(heap, *child)?;
            let length =
                if child_weight > 0.0 && measured_dimension(heap, *child, !vertical)?.is_none() {
                    (size - fixed).max(0.0) * child_weight / weights
                } else {
                    dimension(heap, *child, !vertical, size)?
                };
            let (x, y, w, h) = if vertical {
                cursor += margins[1];
                let r = (
                    available.x + margins[0],
                    cursor,
                    dimension(heap, *child, true, available.width)?,
                    length,
                );
                cursor += length + margins[3];
                r
            } else {
                cursor += margins[0];
                let r = (
                    cursor,
                    available.y + margins[1],
                    length,
                    dimension(heap, *child, false, available.height)?,
                );
                cursor += length + margins[2];
                r
            };
            let child_rect = laid_out_rect(heap, *child, rect)?.unwrap_or(Rect {
                x,
                y,
                width: w,
                height: h,
            });
            children.push(build(heap, *child, child_rect, path)?);
        }
    }
    path.pop();
    Ok(Node {
        handle,
        view,
        rect,
        paint_clip: rect,
        input_clip: rect,
        children,
    })
}
fn text_advance(size: f32) -> f32 {
    // ponytail: retain the existing scalar-character font approximation;
    // replace with shared shaping/font metrics when native text fidelity is required.
    size * 0.6
}

pub(crate) fn text_line_count(text: &str, size: f32, width: f32, single: bool) -> i32 {
    if single {
        return 1;
    }
    let capacity = if size == 0.0 {
        usize::MAX
    } else {
        (width / text_advance(size)).floor().max(1.0) as usize
    };
    let mut lines = 0;
    for paragraph in text.split('\n') {
        lines += 1;
        let (mut start, mut last_break) = (0, 0);
        for (index, character) in paragraph.chars().enumerate() {
            if index - start >= capacity {
                start = if last_break > start {
                    last_break
                } else {
                    index
                };
                lines += 1;
            }
            if matches!(character, ' ' | '\t' | '-') {
                last_break = index + 1;
            }
        }
    }
    lines
}

pub fn dimension(heap: &Heap, word: Word, horizontal: bool, parent: f32) -> Result<f32> {
    dimension_inner(heap, word, horizontal, parent, true)
}
fn measured_dimension(heap: &Heap, word: Word, horizontal: bool) -> Result<Option<f32>> {
    let fields = &heap.get(word)?.fields;
    if fields
        .get("droidless:view:layout-requested")
        .and_then(|v| v.first())
        .is_some_and(|v| v.truth())
    {
        return Ok(None);
    }
    fields
        .get(if horizontal {
            "droidless:view:measured-width"
        } else {
            "droidless:view:measured-height"
        })
        .and_then(|v| v.first())
        .map(|value| value.int().map(|value| value.max(0) as f32))
        .transpose()
}
pub(crate) fn intrinsic_dimension(
    heap: &Heap,
    word: Word,
    horizontal: bool,
    parent: f32,
) -> Result<f32> {
    dimension_inner(heap, word, horizontal, parent, false)
}
fn dimension_inner(
    heap: &Heap,
    word: Word,
    horizontal: bool,
    parent: f32,
    measured: bool,
) -> Result<f32> {
    fn measure(
        heap: &Heap,
        word: Word,
        horizontal: bool,
        parent: f32,
        depth: usize,
        measured: bool,
    ) -> Result<f32> {
        ensure!(depth < 128, "View measurement nesting limit");
        let object = heap.get(word)?;
        let v = object.view.as_ref().context("expected View")?;
        let padding = content_padding(v);
        let field = if horizontal { "width" } else { "height" };
        if measured && let Some(value) = measured_dimension(heap, word, horizontal)? {
            return Ok(value.min(parent));
        }
        let value = if let Some(value) = params_field(
            heap,
            word,
            &format!("Landroid/view/ViewGroup$LayoutParams;->{field}:I"),
        )? {
            value.int()? as f32
        } else if horizontal {
            v.width
        } else {
            v.height
        };
        // UNSPECIFIED measurement has no parent limit; match-parent contributes intrinsic size.
        if (measured || depth > 0) && value == -1.0 && parent.is_finite() {
            return Ok(parent);
        }
        if (measured || depth > 0) && value >= 0.0 {
            return Ok(value.min(parent));
        }
        if let Some(grid) = &v.grid {
            if horizontal {
                return Ok(parent);
            }
            let columns = heap
                .get(word)?
                .fields
                .get("droidless:grid:columns")
                .and_then(|values| values.first())
                .copied()
                .map(|word| word.int())
                .transpose()?
                .unwrap_or(grid.columns.max(1))
                .max(1) as usize;
            let mut height = padding[1] + padding[3];
            for (index, row) in v.children.chunks(columns).enumerate() {
                if index > 0 {
                    height += grid.vertical_spacing as f32;
                }
                let mut row_height: f32 = 0.0;
                for child in row {
                    row_height =
                        row_height.max(measure(heap, *child, false, parent, depth + 1, true)?);
                }
                height += row_height;
            }
            return Ok(height.min(parent));
        }
        if v.children.is_empty() {
            let object = heap.get(word)?;
            let single = object
                .fields
                .get("droidless:text:single-line")
                .and_then(|v| v.first())
                .is_some_and(|v| v.truth());
            return Ok(if horizontal {
                let characters = if single {
                    v.text.chars().count()
                } else {
                    v.text
                        .split('\n')
                        .map(|line| line.chars().count())
                        .max()
                        .unwrap_or(0)
                };
                (characters as f32 * text_advance(v.text_size) + 24.0 + padding[0] + padding[2])
                    .max(48.0)
                    .min(parent)
            } else {
                let layout = object
                    .fields
                    .get(crate::text_layout::LAYOUT)
                    .and_then(|v| v.first())
                    .copied();
                let lines = if let Some(layout) = layout {
                    heap.get(layout)?
                        .fields
                        .get(crate::text_layout::LINES)
                        .and_then(|v| v.first())
                        .copied()
                        .context("Layout line count missing")?
                        .int()?
                } else {
                    text_line_count(&v.text, v.text_size, f32::INFINITY, single)
                };
                let limit = |key: &str, default| -> Result<i32> {
                    object
                        .fields
                        .get(key)
                        .and_then(|v| v.first())
                        .copied()
                        .unwrap_or(Word::from(default))
                        .int()
                };
                let lines = lines
                    .min(if single {
                        1
                    } else {
                        limit("droidless:text:setMaxLines", i32::MAX)?
                    })
                    .max(limit("droidless:text:setMinLines", 0)?)
                    .max(0);
                (v.text_size * lines as f32 + 24.0 + padding[1] + padding[3])
                    .max(44.0)
                    .min(parent)
            });
        }
        let sizes = v
            .children
            .iter()
            .map(|c| {
                if heap
                    .get(*c)?
                    .view
                    .as_ref()
                    .context("expected View child")?
                    .visible
                    == 8
                {
                    return Ok(0.0);
                }
                let margins = margins(heap, *c)?;
                Ok(measure(heap, *c, horizontal, parent, depth + 1, true)?
                    + if horizontal {
                        margins[0] + margins[2]
                    } else {
                        margins[1] + margins[3]
                    })
            })
            .collect::<Result<Vec<_>>>()?;
        let total = if !matches!(v.kind.as_str(), "FrameLayout" | "View")
            && horizontal == (v.orientation == 0)
        {
            sizes.iter().sum()
        } else {
            sizes.iter().copied().fold(0.0, f32::max)
        };
        Ok((total
            + if horizontal {
                padding[0] + padding[2]
            } else {
                padding[1] + padding[3]
            })
        .min(parent))
    }
    measure(heap, word, horizontal, parent, 0, measured)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn frame_child_gravity_padding_margins_and_layout_params_precedence() {
        let mut heap = Heap::default();
        let root = heap.instance("Landroid/widget/FrameLayout;").unwrap();
        let child = heap.instance("Landroid/widget/Button;").unwrap();
        let frame = heap.get_mut(root).unwrap().view.as_mut().unwrap();
        frame.padding = [20.0; 4];
        frame.children.push(child);
        let button = heap.get_mut(child).unwrap().view.as_mut().unwrap();
        button.width = 40.0;
        button.height = 30.0;
        button.margins = [2.0, 3.0, 4.0, 5.0];
        for (gravity, expected) in [
            (-1, (22.0, 23.0)),
            (0x11, (78.0, 133.0)),
            (0x55, (136.0, 245.0)),
            (0x800053, (22.0, 245.0)),
            (0x800055, (136.0, 245.0)),
        ] {
            heap.get_mut(child).unwrap().fields.insert(
                "droidless:view:layout-gravity".into(),
                vec![Word::from(gravity)],
            );
            let rect = layout(&heap, root, 200.0, 300.0).unwrap().children[0].rect;
            assert_eq!((rect.x, rect.y), expected);
            assert_eq!((rect.width, rect.height), (40.0, 30.0));
        }
        let params = heap
            .instance("Landroid/widget/FrameLayout$LayoutParams;")
            .unwrap();
        for (key, value) in [
            ("Landroid/view/ViewGroup$LayoutParams;->width:I", 40),
            ("Landroid/view/ViewGroup$LayoutParams;->height:I", 30),
            ("Landroid/widget/FrameLayout$LayoutParams;->gravity:I", 0x11),
        ] {
            heap.get_mut(params)
                .unwrap()
                .fields
                .insert(key.into(), vec![Word::from(value)]);
        }
        heap.get_mut(child)
            .unwrap()
            .fields
            .insert("droidless:view:layout-params".into(), vec![params]);
        let rect = layout(&heap, root, 200.0, 300.0).unwrap().children[0].rect;
        assert_eq!((rect.x, rect.y), (78.0, 133.0));
    }
}
