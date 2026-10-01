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
    pub padding: f32,
    pub margins: [f32; 4],
    pub visible: i32,
    pub enabled: bool,
    pub editable: bool,
    pub text_size: f32,
    pub text_color: u32,
    pub background: Option<u32>,
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
            "Landroid/support/v7/widget/AppCompatTextView;" => "TextView",
            "Landroid/support/v7/widget/AppCompatEditText;" => "EditText",
            "Landroid/widget/ImageView;" => "ImageView",
            "Landroid/widget/Space;" => "View",
            "Landroid/support/v7/widget/ViewStubCompat;" => "View",
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
            padding: 0.0,
            margins: [0.0; 4],
            visible: 0,
            enabled: true,
            editable: name == "EditText",
            text_size: 18.0,
            text_color: 0xff222222,
            background: None,
            gravity: 0,
            image_scale: 3,
            grid: (name == "GridView").then(Grid::default),
            image: None,
        })
    }
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

fn params_field(heap: &Heap, word: Word, field: &str) -> Result<Option<Word>> {
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
fn weight(heap: &Heap, word: Word) -> Result<f32> {
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
fn build(heap: &Heap, word: Word, rect: Rect, path: &mut Vec<usize>) -> Result<Node> {
    let handle = word.reference()?;
    ensure!(
        path.len() < 128 && !path.contains(&handle),
        "cyclic or too deep View hierarchy"
    );
    path.push(handle);
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
    let mut children = vec![];
    let available = Rect {
        x: rect.x + view.padding,
        y: rect.y + view.padding,
        width: (rect.width - 2.0 * view.padding).max(0.0),
        height: (rect.height - 2.0 * view.padding).max(0.0),
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
            let c = heap.get(*child)?.view.as_ref().context("non-View child")?;
            if c.visible == 8 {
                continue;
            }
            let child_rect = laid_out_rect(heap, *child, rect)?.unwrap_or(Rect {
                x: available.x + c.margins[0],
                y: available.y + c.margins[1],
                width: dimension(heap, *child, true, available.width)?,
                height: dimension(heap, *child, false, available.height)?,
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
            if c.visible == 8 {
                continue;
            }
            let child_weight = weight(heap, *child)?;
            weights += child_weight;
            fixed += if child_weight > 0.0 {
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
            let child_weight = weight(heap, *child)?;
            let length = if child_weight > 0.0 {
                (size - fixed).max(0.0) * child_weight / weights
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
        children,
    })
}
pub fn dimension(heap: &Heap, word: Word, horizontal: bool, parent: f32) -> Result<f32> {
    fn measure(
        heap: &Heap,
        word: Word,
        horizontal: bool,
        parent: f32,
        depth: usize,
    ) -> Result<f32> {
        ensure!(depth < 128, "View measurement nesting limit");
        let v = heap.get(word)?.view.as_ref().context("expected View")?;
        let field = if horizontal { "width" } else { "height" };
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
        if value == -1.0 {
            return Ok(parent);
        }
        if value >= 0.0 {
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
            let mut height = 2.0 * v.padding;
            for (index, row) in v.children.chunks(columns).enumerate() {
                if index > 0 {
                    height += grid.vertical_spacing as f32;
                }
                let mut row_height: f32 = 0.0;
                for child in row {
                    row_height = row_height.max(measure(heap, *child, false, parent, depth + 1)?);
                }
                height += row_height;
            }
            return Ok(height.min(parent));
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
