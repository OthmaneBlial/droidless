//! Bounded adapter-backed grids. Guest DEX owns cells and item callbacks.
use crate::{
    framework::OBSERVERS,
    heap::{Data, Word, fault},
    ui,
    vm::Runtime,
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

const GRID: &str = "Landroid/widget/GridView;";
const ADAPTER_VIEW: &str = "Landroid/widget/AdapterView;";
const ADAPTER: &str = "Landroid/widget/ListAdapter;";
const BASE: &str = "Landroid/widget/BaseAdapter;";
const OBSERVABLE: &str = "Landroid/database/DataSetObservable;";
const OBSERVER: &str = "Landroid/database/DataSetObserver;";
const GRID_OBSERVER: &str = "Ldroidless/runtime/GridObserver;";
const GRID_CLICK: &str = "Ldroidless/runtime/GridClick;";
const ADAPTER_FIELD: &str = "droidless:grid:adapter";
const DIRTY: &str = "droidless:grid:dirty";
const BINDING: &str = "droidless:grid:binding";
const COUNT: &str = "droidless:grid:count";
const OBSERVABLE_FIELD: &str = "droidless:adapter:observable";

impl Runtime {
    fn grid_word(&self, receiver: Word, key: &str) -> Result<Word> {
        Ok(self
            .heap
            .get(receiver)?
            .fields
            .get(key)
            .and_then(|v| v.first())
            .copied()
            .unwrap_or(Word::ZERO))
    }
    fn grid_put(&mut self, receiver: Word, key: &str, value: Word) -> Result<()> {
        self.heap
            .get_mut(receiver)?
            .fields
            .insert(key.into(), vec![value]);
        Ok(())
    }
    fn grid_call(
        &mut self,
        receiver: Word,
        class: &str,
        name: &str,
        parameters: &[&str],
        returns: &str,
        mut args: Vec<Word>,
    ) -> Result<Vec<Word>> {
        args.insert(0, receiver);
        self.invoke(
            Method {
                class: class.into(),
                name: name.into(),
                parameters: parameters.iter().map(|p| (*p).into()).collect(),
                returns: returns.into(),
            },
            args,
            name != "<init>",
        )
    }
    fn adapter_count(&mut self, adapter: Word) -> Result<i32> {
        let count = self
            .grid_call(adapter, ADAPTER, "getCount", &[], "I", vec![])?
            .first()
            .context("adapter count missing")?
            .int()?;
        if !(0..=1024).contains(&count) {
            return Err(fault(
                "Ljava/lang/IllegalStateException;",
                "GridView adapter count must be within 0..1024",
            ));
        }
        Ok(count)
    }
    fn grid_idle(&self, grid: Word) -> Result<()> {
        if self.grid_word(grid, BINDING)?.truth() {
            return Err(fault(
                "Ljava/lang/IllegalStateException;",
                "GridView adapter mutation during binding is unsupported",
            ));
        }
        Ok(())
    }
    fn set_grid_adapter(&mut self, grid: Word, adapter: Word) -> Result<()> {
        self.grid_idle(grid)?;
        if adapter != Word::ZERO {
            ensure!(
                self.is_a(&self.heap.get(adapter)?.class, ADAPTER),
                "GridView requires ListAdapter"
            );
        }
        self.grid_put(grid, BINDING, Word::from(1))?;
        let result = (|| {
            let count = if adapter == Word::ZERO {
                0
            } else {
                self.adapter_count(adapter)?
            };
            let old = self.grid_word(grid, ADAPTER_FIELD)?;
            let observer = self.grid_word(grid, "droidless:grid:observer")?;
            if old != Word::ZERO && observer != Word::ZERO {
                self.grid_call(
                    old,
                    ADAPTER,
                    "unregisterDataSetObserver",
                    &[OBSERVER],
                    "V",
                    vec![observer],
                )?;
            }
            self.grid_put(grid, "droidless:grid:observer", Word::ZERO)?;
            let children = std::mem::take(
                &mut self
                    .heap
                    .get_mut(grid)?
                    .view
                    .as_mut()
                    .context("expected GridView")?
                    .children,
            );
            self.native_roots.extend(children.iter().copied());
            for child in children {
                self.focus_before_remove(grid, child)?;
                self.grid_put(child, "droidless:view:parent", Word::ZERO)?;
                self.focus_hierarchy_change(grid, child, false)?;
            }
            self.grid_put(grid, ADAPTER_FIELD, adapter)?;
            if adapter != Word::ZERO {
                let types = self
                    .grid_call(adapter, ADAPTER, "getViewTypeCount", &[], "I", vec![])?
                    .first()
                    .context("adapter view-type count missing")?
                    .int()?;
                ensure!(
                    (1..=256).contains(&types),
                    "adapter view-type count exceeds limit"
                );
                self.grid_put(grid, "droidless:grid:types", Word::from(types))?;
                let observer = self.heap.instance(GRID_OBSERVER)?;
                self.native_roots.push(observer);
                self.grid_put(observer, "grid", grid)?;
                self.grid_put(observer, "adapter", adapter)?;
                self.grid_put(grid, "droidless:grid:observer", observer)?;
                self.grid_call(
                    adapter,
                    ADAPTER,
                    "registerDataSetObserver",
                    &[OBSERVER],
                    "V",
                    vec![observer],
                )?;
            }
            self.grid_put(grid, COUNT, Word::from(count))?;
            self.grid_put(grid, "droidless:grid:invalid", Word::ZERO)?;
            self.grid_put(grid, DIRTY, Word::from(1))
        })();
        self.grid_put(grid, BINDING, Word::ZERO)?;
        result
    }
    pub(crate) fn grid_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let class = method.class.as_str();
        let signature = method.signature();
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        if ["Landroid/view/View;", "Landroid/view/ViewGroup;"].contains(&class)
            && matches!(receiver, Word::Ref(_))
            && self.is_a(&self.heap.get(receiver)?.class, GRID)
        {
            if method.name == "addView"
                || ["removeView", "removeViewAt", "removeAllViews"].contains(&method.name.as_str())
            {
                return Err(fault(
                    "Ljava/lang/UnsupportedOperationException;",
                    "adapter owns GridView children",
                ));
            }
            if signature == "setOnClickListener(Landroid/view/View$OnClickListener;)V" {
                return Err(fault(
                    "Ljava/lang/RuntimeException;",
                    "use GridView item-click listener",
                ));
            }
        }
        if ![
            GRID,
            ADAPTER_VIEW,
            "Landroid/widget/AbsListView;",
            BASE,
            OBSERVABLE,
            OBSERVER,
            GRID_OBSERVER,
            GRID_CLICK,
        ]
        .contains(&class)
        {
            return Ok(None);
        }
        let roots = self.native_roots.len();
        self.native_roots.extend_from_slice(args);
        let result = (|| {
            let arg = |i| {
                args.get(i)
                    .copied()
                    .context("missing grid/adapter argument")
            };
            self.heap.get(receiver)?;
            let values = match (class, signature.as_str()) {
                (BASE, "<init>()V") => {
                    let observable = self.heap.instance(OBSERVABLE)?;
                    self.grid_put(receiver, OBSERVABLE_FIELD, observable)?;
                    self.grid_call(observable, OBSERVABLE, "<init>", &[], "V", vec![])?
                }
                (OBSERVABLE, "<init>()V") => self.grid_call(
                    receiver,
                    "Landroid/database/Observable;",
                    "<init>",
                    &[],
                    "V",
                    vec![],
                )?,
                (OBSERVER, "<init>()V" | "onChanged()V" | "onInvalidated()V") => vec![],
                (
                    OBSERVABLE,
                    "registerObserver(Landroid/database/DataSetObserver;)V"
                    | "unregisterObserver(Landroid/database/DataSetObserver;)V",
                ) => self.grid_call(
                    receiver,
                    "Landroid/database/Observable;",
                    &method.name,
                    &["Ljava/lang/Object;"],
                    "V",
                    vec![arg(1)?],
                )?,
                (OBSERVABLE, "notifyChanged()V" | "notifyInvalidated()V") => {
                    let observers = self.grid_word(receiver, OBSERVERS)?;
                    self.native_roots.push(observers);
                    let Data::Collection { values, .. } = &self.heap.get(observers)?.data else {
                        anyhow::bail!("uninitialized observer list");
                    };
                    let count = values.len();
                    // Match the platform's reverse, live-list dispatch, including self-removal.
                    for index in (0..count).rev() {
                        let observer = self.grid_call(
                            observers,
                            "Ljava/util/ArrayList;",
                            "get",
                            &["I"],
                            "Ljava/lang/Object;",
                            vec![Word::from(index as i32)],
                        )?[0];
                        self.grid_call(
                            observer,
                            OBSERVER,
                            if method.name == "notifyChanged" {
                                "onChanged"
                            } else {
                                "onInvalidated"
                            },
                            &[],
                            "V",
                            vec![],
                        )?;
                    }
                    vec![]
                }
                (
                    BASE,
                    "registerDataSetObserver(Landroid/database/DataSetObserver;)V"
                    | "unregisterDataSetObserver(Landroid/database/DataSetObserver;)V",
                ) => {
                    let observable = self.grid_word(receiver, OBSERVABLE_FIELD)?;
                    self.grid_call(
                        observable,
                        OBSERVABLE,
                        if method.name == "registerDataSetObserver" {
                            "registerObserver"
                        } else {
                            "unregisterObserver"
                        },
                        &[OBSERVER],
                        "V",
                        vec![arg(1)?],
                    )?
                }
                (BASE, "notifyDataSetChanged()V" | "notifyDataSetInvalidated()V") => {
                    let observable = self.grid_word(receiver, OBSERVABLE_FIELD)?;
                    self.grid_call(
                        observable,
                        OBSERVABLE,
                        if method.name == "notifyDataSetChanged" {
                            "notifyChanged"
                        } else {
                            "notifyInvalidated"
                        },
                        &[],
                        "V",
                        vec![],
                    )?
                }
                (BASE, "areAllItemsEnabled()Z" | "isEnabled(I)Z" | "getViewTypeCount()I") => {
                    vec![Word::from(1)]
                }
                (BASE, "hasStableIds()Z" | "getItemViewType(I)I") => vec![Word::ZERO],
                (BASE, "isEmpty()Z") => {
                    let count = self
                        .grid_call(receiver, ADAPTER, "getCount", &[], "I", vec![])?
                        .first()
                        .context("adapter count missing")?
                        .int()?;
                    vec![Word::from(i32::from(count == 0))]
                }
                (
                    BASE,
                    "getDropDownView(ILandroid/view/View;Landroid/view/ViewGroup;)Landroid/view/View;",
                ) => self.grid_call(
                    receiver,
                    ADAPTER,
                    "getView",
                    &["I", "Landroid/view/View;", "Landroid/view/ViewGroup;"],
                    "Landroid/view/View;",
                    args[1..].to_vec(),
                )?,
                (GRID_OBSERVER, "onChanged()V" | "onInvalidated()V") => {
                    self.require_main_thread()?;
                    let grid = self.grid_word(receiver, "grid")?;
                    let adapter = self.grid_word(receiver, "adapter")?;
                    if self.grid_word(grid, ADAPTER_FIELD)? == adapter {
                        self.grid_idle(grid)?;
                        let invalid = method.name == "onInvalidated";
                        let count = if invalid {
                            0
                        } else {
                            self.adapter_count(adapter)?
                        };
                        ensure!(
                            self.grid_word(grid, ADAPTER_FIELD)? == adapter,
                            "adapter changed while counting items"
                        );
                        self.grid_put(grid, COUNT, Word::from(count))?;
                        self.grid_put(
                            grid,
                            "droidless:grid:invalid",
                            Word::from(i32::from(invalid)),
                        )?;
                        self.grid_put(grid, DIRTY, Word::from(1))?;
                    }
                    vec![]
                }
                (GRID_CLICK, "onClick(Landroid/view/View;)V") => {
                    self.require_main_thread()?;
                    let grid = self.grid_word(receiver, "grid")?;
                    let position = self.grid_word(receiver, "position")?;
                    let adapter = self.grid_word(grid, ADAPTER_FIELD)?;
                    if adapter != Word::ZERO
                        && position.int()? < self.grid_word(grid, COUNT)?.int()?
                    {
                        let enabled = self
                            .grid_call(adapter, ADAPTER, "isEnabled", &["I"], "Z", vec![position])?
                            .first()
                            .context("adapter enabled state missing")?
                            .truth();
                        if enabled {
                            let id = self.grid_call(
                                adapter,
                                ADAPTER,
                                "getItemId",
                                &["I"],
                                "J",
                                vec![position],
                            )?;
                            ensure!(id.len() == 2, "item id must be a long");
                            self.grid_call(
                                grid,
                                ADAPTER_VIEW,
                                "performItemClick",
                                &["Landroid/view/View;", "I", "J"],
                                "Z",
                                vec![arg(1)?, position, id[0], id[1]],
                            )?;
                        }
                    }
                    vec![]
                }
                _ if [GRID, ADAPTER_VIEW, "Landroid/widget/AbsListView;"].contains(&class)
                    && self.is_a(&self.heap.get(receiver)?.class, GRID) =>
                {
                    self.require_main_thread()?;
                    match signature.as_str() {
                        "<init>(Landroid/content/Context;)V"
                        | "<init>(Landroid/content/Context;Landroid/util/AttributeSet;)V"
                        | "<init>(Landroid/content/Context;Landroid/util/AttributeSet;I)V"
                        | "<init>(Landroid/content/Context;Landroid/util/AttributeSet;II)V" => {
                            self.grid_call(
                                receiver,
                                "Landroid/view/View;",
                                "<init>",
                                &["Landroid/content/Context;"],
                                "V",
                                vec![arg(1)?],
                            )?;
                            self.grid_put(receiver, "droidless:grid:columns", Word::from(-1))?;
                            vec![]
                        }
                        "setAdapter(Landroid/widget/ListAdapter;)V"
                        | "setAdapter(Landroid/widget/Adapter;)V" => {
                            self.set_grid_adapter(receiver, arg(1)?)?;
                            vec![]
                        }
                        "getAdapter()Landroid/widget/ListAdapter;"
                        | "getAdapter()Landroid/widget/Adapter;" => {
                            vec![self.grid_word(receiver, ADAPTER_FIELD)?]
                        }
                        "getCount()I" => vec![self.grid_word(receiver, COUNT)?],
                        "setOnItemClickListener(Landroid/widget/AdapterView$OnItemClickListener;)V" =>
                        {
                            self.grid_put(receiver, "droidless:grid:click", arg(1)?)?;
                            vec![]
                        }
                        "getOnItemClickListener()Landroid/widget/AdapterView$OnItemClickListener;" =>
                        {
                            vec![self.grid_word(receiver, "droidless:grid:click")?]
                        }
                        "performItemClick(Landroid/view/View;IJ)Z" => {
                            let listener = self.grid_word(receiver, "droidless:grid:click")?;
                            if listener == Word::ZERO {
                                vec![Word::ZERO]
                            } else {
                                self.grid_call(
                                    listener,
                                    "Landroid/widget/AdapterView$OnItemClickListener;",
                                    "onItemClick",
                                    &[ADAPTER_VIEW, "Landroid/view/View;", "I", "J"],
                                    "V",
                                    vec![receiver, arg(1)?, arg(2)?, arg(3)?, arg(4)?],
                                )?;
                                vec![Word::from(1)]
                            }
                        }
                        "setNumColumns(I)V"
                        | "setColumnWidth(I)V"
                        | "setHorizontalSpacing(I)V"
                        | "setVerticalSpacing(I)V"
                        | "setStretchMode(I)V"
                        | "setGravity(I)V" => {
                            let value = arg(1)?.int()?;
                            let view = self
                                .heap
                                .get_mut(receiver)?
                                .view
                                .as_mut()
                                .context("expected GridView")?;
                            let grid = view.grid.as_mut().context("expected grid geometry")?;
                            match method.name.as_str() {
                                "setNumColumns" => grid.columns = value,
                                "setColumnWidth" => grid.column_width = value,
                                "setHorizontalSpacing" => grid.horizontal_spacing = value,
                                "setVerticalSpacing" => grid.vertical_spacing = value,
                                "setStretchMode" => grid.stretch = value,
                                _ => view.gravity = value as u32,
                            }
                            self.grid_put(
                                receiver,
                                "droidless:view:layout-requested",
                                Word::from(1),
                            )?;
                            vec![]
                        }
                        "getNumColumns()I" | "getColumnWidth()I" | "getHorizontalSpacing()I" => {
                            vec![self.grid_word(
                                receiver,
                                match method.name.as_str() {
                                    "getNumColumns" => "droidless:grid:columns",
                                    "getColumnWidth" => "droidless:grid:column-width",
                                    _ => "droidless:grid:horizontal-spacing",
                                },
                            )?]
                        }
                        "getRequestedColumnWidth()I"
                        | "getRequestedHorizontalSpacing()I"
                        | "getVerticalSpacing()I"
                        | "getStretchMode()I"
                        | "getGravity()I" => {
                            let view = self
                                .heap
                                .get(receiver)?
                                .view
                                .as_ref()
                                .context("expected GridView")?;
                            let grid = view.grid.as_ref().context("expected grid geometry")?;
                            vec![Word::from(match method.name.as_str() {
                                "getRequestedColumnWidth" => grid.column_width,
                                "getRequestedHorizontalSpacing" => grid.horizontal_spacing,
                                "getVerticalSpacing" => grid.vertical_spacing,
                                "getStretchMode" => grid.stretch,
                                _ => view.gravity as i32,
                            })]
                        }
                        _ => return Ok(None),
                    }
                }
                _ => return Ok(None),
            };
            Ok(Some(values))
        })();
        self.native_roots.truncate(roots);
        result
    }

    fn bind_grid(&mut self, grid: Word) -> Result<()> {
        self.grid_idle(grid)?;
        self.grid_put(grid, BINDING, Word::from(1))?;
        let roots = self.native_roots.len();
        self.native_roots.push(grid);
        let result = (|| {
            let adapter = self.grid_word(grid, ADAPTER_FIELD)?;
            let count = self.grid_word(grid, COUNT)?.int()?;
            let old = self
                .heap
                .get(grid)?
                .view
                .as_ref()
                .context("expected GridView")?
                .children
                .clone();
            self.native_roots.extend(old.iter().copied());
            let mut cells = vec![];
            if adapter != Word::ZERO && !self.grid_word(grid, "droidless:grid:invalid")?.truth() {
                self.native_roots.push(adapter);
                ensure!(
                    count == self.adapter_count(adapter)?,
                    "adapter count changed without notification"
                );
                let types = self.grid_word(grid, "droidless:grid:types")?.int()?;
                for position in 0..count {
                    let position = Word::from(position);
                    let kind = self.grid_call(
                        adapter,
                        ADAPTER,
                        "getItemViewType",
                        &["I"],
                        "I",
                        vec![position],
                    )?[0]
                        .int()?;
                    ensure!(
                        kind == -1 || (0..types).contains(&kind),
                        "invalid adapter view type"
                    );
                    // ponytail: materialize at most 1024 cells; viewport recycling and scrolling come next.
                    let cell = self.grid_call(
                        adapter,
                        ADAPTER,
                        "getView",
                        &["I", "Landroid/view/View;", "Landroid/view/ViewGroup;"],
                        "Landroid/view/View;",
                        vec![position, Word::ZERO, grid],
                    )?[0];
                    self.native_roots.push(cell);
                    let parent = self.grid_word(cell, "droidless:view:parent")?;
                    ensure!(
                        cell != grid
                            && !cells.contains(&cell)
                            && (parent == Word::ZERO || parent == grid),
                        "adapter returned an attached or duplicate cell"
                    );
                    let view = self
                        .heap
                        .get(cell)?
                        .view
                        .as_ref()
                        .context("adapter returned a non-View")?;
                    let replace = view.xml_click.is_none()
                        && match view.listener {
                            None => true,
                            Some(listener) => self.heap.get(listener)?.class == GRID_CLICK,
                        };
                    if replace {
                        let click = self.heap.instance(GRID_CLICK)?;
                        self.grid_put(click, "grid", grid)?;
                        self.grid_put(click, "position", position)?;
                        self.heap.get_mut(cell)?.view.as_mut().unwrap().listener = Some(click);
                    }
                    cells.push(cell);
                }
            }
            ensure!(
                adapter == self.grid_word(grid, ADAPTER_FIELD)?,
                "adapter changed during binding"
            );
            for cell in old.iter().filter(|cell| !cells.contains(cell)) {
                self.focus_before_remove(grid, *cell)?;
                self.grid_put(*cell, "droidless:view:parent", Word::ZERO)?;
            }
            for cell in &cells {
                self.grid_put(*cell, "droidless:view:parent", grid)?;
            }
            self.heap
                .get_mut(grid)?
                .view
                .as_mut()
                .context("expected GridView")?
                .children = cells.clone();
            self.grid_put(grid, DIRTY, Word::ZERO)?;
            for cell in old.iter().filter(|cell| !cells.contains(cell)) {
                self.focus_hierarchy_change(grid, *cell, false)?;
            }
            for cell in cells {
                self.focus_hierarchy_change(grid, cell, true)?;
            }
            Ok(())
        })();
        self.grid_put(grid, BINDING, Word::ZERO)?;
        self.native_roots.truncate(roots);
        result
    }
    pub(crate) fn bind_grids(&mut self, root: Word, width: f32, height: f32) -> Result<()> {
        fn collect(node: &ui::Node, out: &mut Vec<(Word, ui::GridMetrics)>) -> Result<()> {
            if node.view.grid.is_some() {
                out.push((
                    Word::Ref(node.handle),
                    ui::grid_metrics(
                        &node.view,
                        node.rect.width - node.view.padding[0] - node.view.padding[2],
                    )?,
                ));
            }
            for child in &node.children {
                collect(child, out)?;
            }
            Ok(())
        }
        for _ in 0..32 {
            let mut grids = vec![];
            collect(&ui::layout(&self.heap, root, width, height)?, &mut grids)?;
            let mut bound = false;
            let roots = self.native_roots.len();
            self.native_roots
                .extend(grids.iter().map(|(grid, _)| *grid));
            let result = (|| -> Result<()> {
                for (grid, metrics) in grids {
                    self.grid_put(
                        grid,
                        "droidless:grid:columns",
                        Word::from(metrics.columns as i32),
                    )?;
                    self.grid_put(
                        grid,
                        "droidless:grid:column-width",
                        Word::from(metrics.width as i32),
                    )?;
                    self.grid_put(
                        grid,
                        "droidless:grid:horizontal-spacing",
                        Word::from(metrics.spacing as i32),
                    )?;
                    if self.grid_word(grid, DIRTY)?.truth() {
                        self.bind_grid(grid)?;
                        bound = true;
                    }
                }
                Ok(())
            })();
            self.native_roots.truncate(roots);
            result?;
            if !bound {
                return Ok(());
            }
        }
        anyhow::bail!("nested GridView binding limit exceeded")
    }
}
