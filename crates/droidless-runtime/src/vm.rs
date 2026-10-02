use crate::{
    heap::{Data, Heap, Word, default_value, exception_parent, fault, wide},
    ui::{self, Node},
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::{
    apk::Apk,
    dex::{Code, EncodedValue, Field, Method},
};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default, Clone, Copy)]
pub struct Trace {
    pub bytecode: bool,
    pub methods: bool,
    pub framework: bool,
    pub lifecycle: bool,
}
pub(crate) struct Frame {
    pub dex: usize,
    pub method: Method,
    pub code: Code,
    pub registers: Vec<Word>,
    pub pc: usize,
    pub result: Vec<Word>,
    pub exception: Option<Word>,
    pub return_pc: Option<usize>,
    pub looper_return: Option<Word>,
    pub monitors: Vec<Word>,
}
impl Frame {
    pub(crate) fn roots(&self) -> impl Iterator<Item = Word> + '_ {
        self.registers
            .iter()
            .chain(&self.result)
            .chain(&self.monitors)
            .copied()
            .chain(self.exception)
            .chain(self.looper_return)
    }
    pub(crate) fn location(&self) -> String {
        format!(
            "at {} [classes{}.dex, PC 0x{:04x}]",
            self.method.key(),
            if self.dex == 0 {
                String::new()
            } else {
                (self.dex + 1).to_string()
            },
            self.pc
        )
    }
}
pub struct Runtime {
    pub apk: Apk,
    pub heap: Heap,
    pub activity: Option<Word>,
    pub root: Option<Word>,
    pub title: String,
    pub(crate) default_title: String,
    pub trace: Trace,
    pub instructions: u64,
    pub method_calls: u64,
    pub lifecycle: Vec<String>,
    pub width: f32,
    pub height: f32,
    pub(crate) host_window_focused: bool,
    pub(crate) frames: Vec<Frame>,
    pub(crate) statics: BTreeMap<String, Vec<Word>>,
    initialized: BTreeSet<String>,
    failed_classes: BTreeMap<String, Option<Word>>,
    initializing_depth: usize,
    pub(crate) interned: BTreeMap<String, Word>,
    budget: u64,
    pub(crate) screens: BTreeMap<usize, crate::activities::Screen>,
    pub(crate) back_stack: Vec<Word>,
    pub(crate) navigation: std::collections::VecDeque<crate::activities::Navigation>,
    pub(crate) directory_request: Option<(Word, i32)>,
    pub(crate) document_trees: BTreeMap<String, cap_std::fs::Dir>,
    pub(crate) storage: Option<crate::storage::Storage>,
    pub(crate) virtual_directories: BTreeSet<Vec<String>>,
    pub(crate) databases: BTreeMap<String, std::sync::Arc<std::sync::Mutex<rusqlite::Connection>>>,
    pub(crate) database_transactions: BTreeMap<String, Vec<bool>>,
    pub(crate) preferences: BTreeMap<String, Word>,
    pub(crate) queue: crate::scheduling::MainQueue,
    pub(crate) property_animations: crate::property_animations::PropertyAnimations,
    pub(crate) workers: crate::workers::Workers,
    pub(crate) sync_depth: usize,
    pub(crate) native_roots: Vec<Word>,
    pub(crate) drawable_state_path: Vec<Word>,
    pub(crate) touch: Option<crate::touch::TouchStream>,
    pub(crate) touch_depth: usize,
    pub(crate) started: std::time::Instant,
}
impl Runtime {
    pub(crate) fn reset_budget(&mut self) {
        self.budget = 0;
    }
    pub fn stack_depth(&self) -> usize {
        self.frames.len()
    }
    pub fn new(apk: Apk) -> Result<Self> {
        let mut names = BTreeSet::new();
        for dex in &apk.dex {
            for c in &dex.classes {
                ensure!(
                    c.name != "Landroid/os/Build$VERSION;",
                    "APK redefinition of native Build.VERSION is unsupported"
                );
                ensure!(
                    names.insert(c.name.clone()),
                    "duplicate class across DEX modules: {}",
                    c.name
                );
            }
        }
        let title = apk
            .manifest
            .label
            .as_ref()
            .map(|v| {
                if v.kind == 1 {
                    apk.resources.text(v.data)
                } else {
                    Ok(v.display())
                }
            })
            .transpose()?
            .unwrap_or_else(|| apk.manifest.package.clone());
        Ok(Self {
            apk,
            heap: Heap::default(),
            activity: None,
            root: None,
            default_title: title.clone(),
            title,
            trace: Trace::default(),
            instructions: 0,
            method_calls: 0,
            lifecycle: vec![],
            width: 420.0,
            height: 720.0,
            host_window_focused: false,
            frames: vec![],
            statics: BTreeMap::new(),
            initialized: BTreeSet::new(),
            failed_classes: BTreeMap::new(),
            initializing_depth: 0,
            interned: BTreeMap::new(),
            budget: 0,
            screens: BTreeMap::new(),
            back_stack: vec![],
            navigation: std::collections::VecDeque::new(),
            directory_request: None,
            document_trees: BTreeMap::new(),
            storage: None,
            virtual_directories: [
                vec![],
                vec!["files".into()],
                vec!["cache".into()],
                vec!["databases".into()],
                vec!["shared_prefs".into()],
            ]
            .into(),
            databases: BTreeMap::new(),
            database_transactions: BTreeMap::new(),
            preferences: BTreeMap::new(),
            queue: crate::scheduling::MainQueue::default(),
            property_animations: crate::property_animations::PropertyAnimations::default(),
            workers: crate::workers::Workers::default(),
            sync_depth: 0,
            native_roots: vec![],
            drawable_state_path: vec![],
            touch: None,
            touch_depth: 0,
            started: std::time::Instant::now(),
        })
    }
    /// Enable disk storage below a host-approved apps root; `new` is ephemeral.
    pub fn with_data_dir(apk: Apk, apps_dir: impl AsRef<std::path::Path>) -> Result<Self> {
        let mut vm = Self::new(apk)?;
        vm.storage = Some(crate::storage::Storage::open(
            apps_dir.as_ref(),
            &vm.apk.manifest.package,
        )?);
        Ok(vm)
    }
    pub fn launch(&mut self) -> Result<()> {
        self.budget = 0;
        if let Some(application) = self.apk.manifest.application.clone() {
            let class = descriptor(&application);
            let object = self.new_instance(&class)?;
            self.invoke(
                Method {
                    class: class.clone(),
                    name: "<init>".into(),
                    parameters: vec![],
                    returns: "V".into(),
                },
                vec![object],
                false,
            )?;
            self.statics
                .insert("droidless:application".into(), vec![object]);
            self.lifecycle_call(object, &class, "onCreate", vec![])?;
        }
        let name = self
            .apk
            .manifest
            .main_activity
            .clone()
            .context("APK has no MAIN/LAUNCHER Activity")?;
        let intent = self.heap.instance("Landroid/content/Intent;")?;
        let class = descriptor(&name);
        let action = self.intern("android.intent.action.MAIN".into())?;
        let component = self.intern(class.clone())?;
        self.heap
            .get_mut(intent)?
            .fields
            .insert("action".into(), vec![action]);
        self.heap
            .get_mut(intent)?
            .fields
            .insert("component".into(), vec![component]);
        self.create_screen(&class, intent, None)?;
        self.drain_navigation()?;
        ensure!(
            self.activity.is_none() || self.root.is_some(),
            "Activity ran but did not create a View hierarchy"
        );
        Ok(())
    }
    pub(crate) fn lifecycle_call(
        &mut self,
        object: Word,
        class: &str,
        name: &str,
        args: Vec<Word>,
    ) -> Result<()> {
        if self.trace.lifecycle {
            eprintln!("lifecycle: {class}->{name}");
        }
        self.lifecycle.push(name.to_owned());
        let activity = self.is_a(class, "Landroid/app/Activity;");
        if activity {
            self.fragments_before_activity(object, name)?;
        }
        let parameters = if name == "onCreate" && activity {
            vec!["Landroid/os/Bundle;".into()]
        } else {
            vec![]
        };
        self.invoke(
            Method {
                class: class.into(),
                name: name.into(),
                parameters,
                returns: "V".into(),
            },
            std::iter::once(object).chain(args).collect(),
            true,
        )?;
        if activity {
            self.fragments_after_activity(object, name)?;
            if name == "onResume" || name == "onPause" {
                self.screens
                    .get_mut(&object.reference()?)
                    .context("Activity missing")?
                    .resumed = name == "onResume";
            }
        }
        Ok(())
    }
    pub fn close(&mut self) -> Result<()> {
        self.budget = 0;
        self.touch = None;
        self.stop_messages()?;
        for screen in self.screens.values_mut() {
            screen.finishing = true;
        }
        if let Some(activity) = self.activity {
            let class = self.heap.get(activity)?.class.clone();
            if self.screen(activity)?.resumed {
                self.lifecycle_call(activity, &class, "onPause", vec![])?;
            }
            self.lifecycle_call(activity, &class, "onStop", vec![])?;
        }
        while let Some(activity) = self.back_stack.pop() {
            let class = self.heap.get(activity)?.class.clone();
            self.lifecycle_call(activity, &class, "onDestroy", vec![])?;
        }
        self.screens.clear();
        self.activity = None;
        self.root = None;
        self.navigation.clear();
        self.directory_request = None;
        self.document_trees.clear();
        Ok(())
    }
    pub fn snapshot(&self) -> Result<Node> {
        ui::layout(
            &self.heap,
            self.root.context("no content View")?,
            self.width,
            self.height,
        )
    }
    pub fn layout_snapshot(&mut self) -> Result<Node> {
        self.bind_grids()?;
        fn collect(
            heap: &Heap,
            node: &Node,
            parent: (f32, f32),
            out: &mut Vec<(Word, [i32; 4])>,
        ) -> Result<()> {
            let translation = |axis| -> Result<f32> {
                let value = heap
                    .get(Word::Ref(node.handle))?
                    .fields
                    .get(&format!("droidless:view:translation-{axis}"))
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO)
                    .int()?;
                Ok(f32::from_bits(value as u32))
            };
            let left = node.rect.x - parent.0 - translation("x")?;
            let top = node.rect.y - parent.1 - translation("y")?;
            out.push((
                Word::Ref(node.handle),
                [
                    left as i32,
                    top as i32,
                    (left + node.rect.width) as i32,
                    (top + node.rect.height) as i32,
                ],
            ));
            for child in &node.children {
                collect(heap, child, (node.rect.x, node.rect.y), out)?;
            }
            Ok(())
        }
        let root = self.root.context("no content View")?;
        let before = ui::layout(&self.heap, root, self.width, self.height)?;
        let mut views = vec![];
        collect(&self.heap, &before, (0.0, 0.0), &mut views)?;
        let mut layout_views = vec![];
        for (view, bounds) in views {
            let mut class = self.heap.get(view)?.class.clone();
            for _ in 0..128 {
                let Some((dex, index)) = self.class_location(&class) else {
                    break;
                };
                let definition = &self.apk.dex[dex].classes[index];
                if definition.methods.iter().any(|encoded| {
                    encoded.code.is_some()
                        && matches!(
                            self.apk.dex[dex].methods[encoded.index]
                                .signature()
                                .as_str(),
                            "onMeasure(II)V" | "onLayout(ZIIII)V"
                        )
                }) {
                    layout_views.push((view, bounds));
                    break;
                }
                let Some(parent) = &definition.super_class else {
                    break;
                };
                class = parent.clone();
            }
        }
        let roots = self.native_roots.len();
        self.native_roots
            .extend(layout_views.iter().map(|(view, _)| *view));
        let result = (|| -> Result<()> {
            for (view, _) in layout_views {
                // ponytail: rebuild per callback so parent layout/mutation is respected;
                // use a dirty recursive traversal if this bounded snapshot path becomes costly.
                let mut current = vec![];
                collect(
                    &self.heap,
                    &ui::layout(&self.heap, root, self.width, self.height)?,
                    (0.0, 0.0),
                    &mut current,
                )?;
                let Some((_, bounds @ [left, top, right, bottom])) =
                    current.into_iter().find(|(word, _)| *word == view)
                else {
                    continue;
                };
                let fields = &self.heap.get(view)?.fields;
                let requested = fields
                    .get("droidless:view:layout-requested")
                    .and_then(|values| values.first())
                    .is_some_and(|word| word.truth());
                let laid_out = fields
                    .get("droidless:view:laid-out")
                    .and_then(|values| values.first())
                    .is_some_and(|word| word.truth());
                let layout_required = fields
                    .get("droidless:view:layout-required")
                    .and_then(|values| values.first())
                    .is_some_and(|word| word.truth());
                let changed = ["left", "top", "right", "bottom"]
                    .into_iter()
                    .zip(bounds)
                    .any(|(edge, value)| {
                        fields
                            .get(&format!("droidless:view:{edge}"))
                            .and_then(|values| values.first())
                            .copied()
                            != Some(Word::from(value))
                    });
                if laid_out && !requested && !layout_required && !changed {
                    continue;
                }
                let width = right.saturating_sub(left).max(0);
                let height = bottom.saturating_sub(top).max(0);
                let width_spec = Word::from((0x4000_0000u32 | width as u32) as i32);
                let height_spec = Word::from((0x4000_0000u32 | height as u32) as i32);
                self.invoke(
                    Method {
                        class: "Landroid/view/View;".into(),
                        name: "measure".into(),
                        parameters: vec!["I".into(), "I".into()],
                        returns: "V".into(),
                    },
                    vec![view, width_spec, height_spec],
                    true,
                )?;
                self.invoke(
                    Method {
                        class: "Landroid/view/View;".into(),
                        name: "layout".into(),
                        parameters: vec!["I".into(), "I".into(), "I".into(), "I".into()],
                        returns: "V".into(),
                    },
                    vec![
                        view,
                        Word::from(left),
                        Word::from(top),
                        Word::from(right),
                        Word::from(bottom),
                    ],
                    true,
                )?;
            }
            Ok(())
        })();
        self.native_roots.truncate(roots);
        result?;
        self.compute_scroll_frame(root, &mut vec![])?;
        ui::layout(&self.heap, root, self.width, self.height)
    }
    fn compute_scroll_frame(&mut self, view: Word, path: &mut Vec<Word>) -> Result<()> {
        ensure!(
            path.len() < 128 && !path.contains(&view),
            "cyclic or too deep View hierarchy"
        );
        if self
            .heap
            .get(view)?
            .view
            .as_ref()
            .context("expected View")?
            .visible
            != 0
        {
            return Ok(());
        }
        path.push(view);
        let roots = self.native_roots.len();
        self.native_roots.push(view);
        let result = (|| -> Result<()> {
            self.invoke(
                Method {
                    class: "Landroid/view/View;".into(),
                    name: "computeScroll".into(),
                    parameters: vec![],
                    returns: "V".into(),
                },
                vec![view],
                true,
            )?;
            let children = self
                .heap
                .get(view)?
                .view
                .as_ref()
                .context("expected View")?
                .children
                .clone();
            self.native_roots.extend(children.iter().copied());
            for child in children {
                let parent = self
                    .heap
                    .get(child)?
                    .fields
                    .get("droidless:view:parent")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                if parent == view {
                    self.compute_scroll_frame(child, path)?;
                }
            }
            Ok(())
        })();
        self.native_roots.truncate(roots);
        path.pop();
        result
    }
    /// Request guest focus when the host begins editing or traverses to a View.
    pub fn focus(&mut self, handle: usize) -> Result<bool> {
        self.require_main_thread()?;
        self.budget = 0;
        let word = Word::Ref(handle);
        let view = self
            .heap
            .get(word)?
            .view
            .as_ref()
            .context("focus target is not a View")?;
        if !view.enabled || view.visible != 0 {
            return Ok(false);
        }
        let took = self.invoke(
            Method {
                class: "Landroid/view/View;".into(),
                name: "requestFocus".into(),
                parameters: vec![],
                returns: "Z".into(),
            },
            vec![word],
            true,
        )?;
        let took = took.first().context("missing host focus result")?.int()? != 0;
        self.drain_navigation()?;
        self.collect();
        Ok(took)
    }
    pub fn click(&mut self, handle: usize) -> Result<bool> {
        self.budget = 0;
        let word = Word::Ref(handle);
        let view = self
            .heap
            .get(word)?
            .view
            .as_ref()
            .context("clicked object is not a View")?
            .clone();
        if !view.enabled || view.visible != 0 {
            return Ok(false);
        }
        let result = self.invoke(
            Method {
                class: "Landroid/view/View;".into(),
                name: "performClick".into(),
                parameters: vec![],
                returns: "Z".into(),
            },
            vec![word],
            true,
        )?[0]
            .int()?
            != 0;
        self.drain_navigation()?;
        self.collect();
        Ok(result)
    }
    pub fn click_text(&mut self, text: &str) -> Result<bool> {
        fn find(node: &Node, text: &str, parent: Option<usize>) -> Option<usize> {
            if node.view.visible != 0 {
                return None;
            }
            let target = if node.view.listener.is_some() || node.view.xml_click.is_some() {
                Some(node.handle)
            } else {
                parent
            };
            if (node.view.text == text || node.view.content_description.as_deref() == Some(text))
                && target.is_some()
            {
                return target;
            }
            node.children.iter().find_map(|c| find(c, text, target))
        }
        // Select the owning row/card for a non-clickable label; guest performClick never bubbles.
        let handle = find(&self.layout_snapshot()?, text, None)
            .with_context(|| format!("no clickable View with text or description {text:?}"))?;
        self.click(handle)
    }
    pub fn edit(&mut self, handle: usize, text: &str) -> Result<()> {
        ensure!(text.len() <= 1_048_576, "text exceeds limit");
        let word = Word::Ref(handle);
        {
            let view = self
                .heap
                .get_mut(word)?
                .view
                .as_mut()
                .context("not a View")?;
            ensure!(view.editable && view.enabled, "View is not editable");
        }
        self.set_view_text(word, text.to_owned(), vec![])
    }
    /// Edit the first enabled visible EditText in the foreground screen.
    pub fn input(&mut self, text: &str) -> Result<()> {
        self.input_at(0, text)
    }
    /// Edit an enabled visible EditText by its zero-based position in the View tree.
    pub fn input_at(&mut self, index: usize, text: &str) -> Result<()> {
        let handle = self.editable_handle_at(index)?;
        self.edit(handle, text)
    }
    /// Request guest focus for an enabled visible EditText in the foreground tree.
    pub fn focus_at(&mut self, index: usize) -> Result<bool> {
        let handle = self.editable_handle_at(index)?;
        self.focus(handle)
    }
    fn editable_handle_at(&mut self, index: usize) -> Result<usize> {
        fn find(node: &Node, index: &mut usize) -> Option<usize> {
            if node.view.visible != 0 {
                return None;
            }
            if node.view.editable && node.view.enabled {
                if *index == 0 {
                    return Some(node.handle);
                }
                *index -= 1;
            }
            node.children.iter().find_map(|child| find(child, index))
        }
        let mut remaining = index;
        find(&self.layout_snapshot()?, &mut remaining)
            .with_context(|| format!("no editable View at index {index}"))
    }
    pub fn key(&mut self, handle: usize, action: i32, keycode: i32) -> Result<bool> {
        ensure!([0, 1].contains(&action), "invalid KeyEvent action");
        self.budget = 0;
        let word = Word::Ref(handle);
        let view = self
            .heap
            .get(word)?
            .view
            .as_ref()
            .context("key target is not a View")?;
        if !view.enabled || view.visible != 0 {
            return Ok(false);
        }
        let Some(listener) = view.key_listener else {
            return Ok(false);
        };
        let event = self.heap.instance("Landroid/view/KeyEvent;")?;
        self.heap
            .get_mut(event)?
            .fields
            .insert("action".into(), vec![Word::from(action)]);
        self.heap
            .get_mut(event)?
            .fields
            .insert("keycode".into(), vec![Word::from(keycode)]);
        let class = self.heap.get(listener)?.class.clone();
        let result = self.invoke(
            Method {
                class,
                name: "onKey".into(),
                parameters: vec![
                    "Landroid/view/View;".into(),
                    "I".into(),
                    "Landroid/view/KeyEvent;".into(),
                ],
                returns: "Z".into(),
            },
            vec![listener, word, Word::from(keycode), event],
            true,
        )?;
        let consumed = result.first().context("onKey returned no result")?.int()? != 0;
        self.drain_navigation()?;
        self.collect();
        Ok(consumed)
    }
    pub fn key_text(&mut self, handle: usize, action: i32, text: &str) -> Result<bool> {
        let Some(keycode) = android_keycode(text) else {
            return Ok(false);
        };
        self.key(handle, action, keycode)
    }
    pub fn focused_key_target(&self) -> Result<Option<usize>> {
        fn find(node: &Node) -> Option<usize> {
            if node.view.key_listener.is_some() && node.view.visible == 0 && node.view.enabled {
                Some(node.handle)
            } else {
                node.children.iter().find_map(find)
            }
        }
        Ok(find(&self.snapshot()?))
    }
    pub fn collect(&mut self) -> usize {
        let roots = self
            .activity
            .into_iter()
            .chain(self.root)
            .chain(self.back_stack.iter().copied())
            .chain(
                self.screens
                    .iter()
                    .flat_map(|(handle, s)| std::iter::once(Word::Ref(*handle)).chain(s.roots())),
            )
            .chain(
                self.navigation
                    .iter()
                    .flat_map(crate::activities::Navigation::roots),
            )
            .chain(self.directory_request.map(|(caller, _)| caller))
            .chain(self.statics.values().flatten().copied())
            .chain(self.preferences.values().copied())
            .chain(self.queue.pending.values().copied())
            .chain(self.queue.active)
            .chain(self.property_animations.roots())
            .chain(self.interned.values().copied())
            .chain(self.failed_classes.values().flatten().copied())
            .chain(self.native_roots.iter().copied())
            .chain(
                self.touch
                    .iter()
                    .flat_map(|stream| [stream.owner, stream.root]),
            )
            .chain(self.frames.iter().flat_map(Frame::roots))
            .chain(self.workers.roots());
        self.heap.collect(roots)
    }
    pub(crate) fn intern(&mut self, text: String) -> Result<Word> {
        if let Some(w) = self.interned.get(&text) {
            return Ok(*w);
        }
        let word = self.heap.string(text.clone())?;
        self.interned.insert(text, word);
        Ok(word)
    }
    pub(crate) fn class_object(&mut self, class: &str) -> Result<Word> {
        let key = format!("droidless:class:{class}");
        if let Some(word) = self.statics.get(&key).and_then(|v| v.first()) {
            return Ok(*word);
        }
        let object = self.heap.instance("Ljava/lang/Class;")?;
        let name = self.intern(class.to_owned())?;
        self.heap
            .get_mut(object)?
            .fields
            .insert("name".into(), vec![name]);
        self.statics.insert(key, vec![object]);
        Ok(object)
    }
    pub(crate) fn class_location(&self, class: &str) -> Option<(usize, usize)> {
        self.apk.dex.iter().enumerate().find_map(|(d, dex)| {
            dex.classes
                .iter()
                .position(|c| c.name == class)
                .map(|c| (d, c))
        })
    }
    pub(crate) fn resolve_field(&self, field: &Field, static_field: bool) -> Result<Field> {
        if !static_field {
            let owner = match (field.class.as_str(), field.name.as_str(), field.ty.as_str()) {
                (
                    "Landroid/widget/LinearLayout$LayoutParams;"
                    | "Landroid/widget/FrameLayout$LayoutParams;",
                    "width" | "height",
                    "I",
                ) => Some("Landroid/view/ViewGroup$LayoutParams;"),
                (
                    "Landroid/widget/LinearLayout$LayoutParams;"
                    | "Landroid/widget/FrameLayout$LayoutParams;",
                    "leftMargin" | "topMargin" | "rightMargin" | "bottomMargin",
                    "I",
                ) => Some("Landroid/view/ViewGroup$MarginLayoutParams;"),
                (class, "width" | "height", "I")
                    if self.is_a(class, "Landroid/view/ViewGroup$MarginLayoutParams;") =>
                {
                    Some("Landroid/view/ViewGroup$LayoutParams;")
                }
                (class, "leftMargin" | "topMargin" | "rightMargin" | "bottomMargin", "I")
                    if self.is_a(class, "Landroid/view/ViewGroup$MarginLayoutParams;") =>
                {
                    Some("Landroid/view/ViewGroup$MarginLayoutParams;")
                }
                _ => None,
            };
            if let Some(owner) = owner {
                return Ok(Field {
                    class: owner.into(),
                    ..field.clone()
                });
            }
        }
        let native_type = if field.name == "TYPE"
            && self.class_location(&field.class).is_none()
            && crate::reflection::primitive_wrapper(&field.class).is_some()
        {
            Some("Ljava/lang/Class;")
        } else if self.sdk_field(field) {
            Some("I")
        } else if self.collections_empty_list_field(field) {
            Some("Ljava/util/List;")
        } else if self.view_outline_provider_field(field) {
            Some("Landroid/view/ViewOutlineProvider;")
        } else if self.text_truncate_at_field(field) {
            Some("Landroid/text/TextUtils$TruncateAt;")
        } else if self.typeface_field(field) {
            Some("Landroid/graphics/Typeface;")
        } else if self.time_unit_field(field).is_some() {
            Some("Ljava/util/concurrent/TimeUnit;")
        } else {
            None
        };
        if let Some(ty) = native_type {
            if field.ty != ty {
                return Err(fault("Ljava/lang/NoSuchFieldError;", field.key()));
            }
            if !static_field {
                return Err(fault(
                    "Ljava/lang/IncompatibleClassChangeError;",
                    field.key(),
                ));
            }
            return Ok(field.clone());
        }
        if let Some(names) = crate::framework::graphics_enum_names(&field.class) {
            if !static_field {
                return Err(fault(
                    "Ljava/lang/IncompatibleClassChangeError;",
                    field.key(),
                ));
            }
            if field.ty != field.class || !names.contains(&field.name.as_str()) {
                return Err(fault("Ljava/lang/NoSuchFieldError;", field.key()));
            }
            return Ok(field.clone());
        }
        if !static_field
            && field.class == "Landroid/graphics/BitmapFactory$Options;"
            && [
                ("inJustDecodeBounds", "Z"),
                ("inSampleSize", "I"),
                ("outWidth", "I"),
                ("outHeight", "I"),
                ("outMimeType", "Ljava/lang/String;"),
            ]
            .contains(&(field.name.as_str(), field.ty.as_str()))
        {
            return Ok(field.clone());
        }
        ensure!(
            self.class_location(&field.class).is_some()
                || (!static_field
                    && [
                        "Landroid/util/DisplayMetrics;",
                        "Landroid/util/TypedValue;",
                        "Landroid/content/res/Configuration;",
                        "Landroid/graphics/Rect;",
                        "Landroid/graphics/RectF;",
                        "Landroid/text/TextPaint;",
                        "Landroid/view/ViewGroup$LayoutParams;",
                        "Landroid/view/ViewGroup$MarginLayoutParams;",
                        "Landroid/widget/LinearLayout$LayoutParams;",
                        "Landroid/widget/FrameLayout$LayoutParams;",
                        "Landroid/widget/TableLayout$LayoutParams;",
                        "Landroid/widget/TableRow$LayoutParams;",
                        "Landroid/os/Message;",
                        "Landroid/database/Observable;",
                        "Landroid/content/pm/ActivityInfo;",
                        "Landroid/content/pm/ApplicationInfo;",
                        "Landroid/content/pm/PackageInfo;",
                        "Landroid/content/pm/ResolveInfo;",
                    ]
                    .contains(&field.class.as_str())),
            "unsupported framework field {}",
            field.key()
        );
        let mut work = vec![field.class.clone()];
        let mut visited = BTreeSet::new();
        while let Some(class) = work.pop() {
            if !visited.insert(class.clone()) {
                continue;
            }
            ensure!(
                visited.len() <= 128,
                "field resolution hierarchy limit reached"
            );
            if let Some((d, c)) = self.class_location(&class) {
                let def = &self.apk.dex[d].classes[c];
                for (is_static, indexes) in
                    [(true, &def.static_fields), (false, &def.instance_fields)]
                {
                    for index in indexes {
                        let declared = &self.apk.dex[d].fields[*index];
                        if declared.name == field.name && declared.ty == field.ty {
                            if is_static != static_field {
                                return Err(fault(
                                    "Ljava/lang/IncompatibleClassChangeError;",
                                    field.key(),
                                ));
                            }
                            return Ok(declared.clone());
                        }
                    }
                }
                if let Some(parent) = &def.super_class {
                    work.push(parent.clone());
                }
                work.extend(def.interfaces.iter().rev().cloned());
            } else if class == "Landroid/os/Build$VERSION;" && field.name == "SDK_INT" {
                return self.resolve_field(
                    &Field {
                        class,
                        ..field.clone()
                    },
                    static_field,
                );
            } else if !static_field
                && ((class == "Landroid/util/DisplayMetrics;"
                    && [
                        ("widthPixels", "I"),
                        ("heightPixels", "I"),
                        ("density", "F"),
                    ]
                    .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/util/TypedValue;"
                        && [
                            ("type", "I"),
                            ("data", "I"),
                            ("assetCookie", "I"),
                            ("resourceId", "I"),
                            ("changingConfigurations", "I"),
                            ("string", "Ljava/lang/CharSequence;"),
                            ("float", "F"),
                            ("density", "I"),
                        ]
                        .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/content/res/Configuration;"
                        && [
                            ("orientation", "I"),
                            ("keyboard", "I"),
                            ("screenWidthDp", "I"),
                            ("screenHeightDp", "I"),
                            ("smallestScreenWidthDp", "I"),
                            ("densityDpi", "I"),
                            ("fontScale", "F"),
                        ]
                        .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/text/TextPaint;"
                        && [
                            ("bgColor", "I"),
                            ("baselineShift", "I"),
                            ("linkColor", "I"),
                            ("drawableState", "[I"),
                            ("density", "F"),
                            ("underlineColor", "I"),
                            ("underlineThickness", "F"),
                        ]
                        .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/graphics/Rect;"
                        && [("left", "I"), ("top", "I"), ("right", "I"), ("bottom", "I")]
                            .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/graphics/RectF;"
                        && [("left", "F"), ("top", "F"), ("right", "F"), ("bottom", "F")]
                            .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (([
                        "Landroid/view/ViewGroup$LayoutParams;",
                        "Landroid/widget/LinearLayout$LayoutParams;",
                        "Landroid/widget/FrameLayout$LayoutParams;",
                    ]
                    .contains(&class.as_str())
                        && [("width", "I"), ("height", "I")]
                            .contains(&(field.name.as_str(), field.ty.as_str())))
                        || ([
                            "Landroid/view/ViewGroup$MarginLayoutParams;",
                            "Landroid/widget/LinearLayout$LayoutParams;",
                            "Landroid/widget/FrameLayout$LayoutParams;",
                        ]
                        .contains(&class.as_str())
                            && [
                                ("leftMargin", "I"),
                                ("topMargin", "I"),
                                ("rightMargin", "I"),
                                ("bottomMargin", "I"),
                            ]
                            .contains(&(field.name.as_str(), field.ty.as_str()))))
                    || (class == "Landroid/widget/LinearLayout$LayoutParams;"
                        && [("weight", "F"), ("gravity", "I")]
                            .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/widget/TableRow$LayoutParams;"
                        && [("column", "I"), ("span", "I")]
                            .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/widget/FrameLayout$LayoutParams;"
                        && [("gravity", "I")].contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/os/Message;"
                        && [
                            ("what", "I"),
                            ("arg1", "I"),
                            ("arg2", "I"),
                            ("obj", "Ljava/lang/Object;"),
                        ]
                        .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/database/Observable;"
                        && [("mObservers", "Ljava/util/ArrayList;")]
                            .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/content/pm/ActivityInfo;"
                        && [
                            ("name", "Ljava/lang/String;"),
                            ("packageName", "Ljava/lang/String;"),
                            ("metaData", "Landroid/os/Bundle;"),
                            ("labelRes", "I"),
                            ("icon", "I"),
                            ("applicationInfo", "Landroid/content/pm/ApplicationInfo;"),
                            ("parentActivityName", "Ljava/lang/String;"),
                            ("targetActivity", "Ljava/lang/String;"),
                            ("exported", "Z"),
                            ("permission", "Ljava/lang/String;"),
                            ("theme", "I"),
                        ]
                        .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/content/pm/ApplicationInfo;"
                        && [
                            ("name", "Ljava/lang/String;"),
                            ("packageName", "Ljava/lang/String;"),
                            ("metaData", "Landroid/os/Bundle;"),
                            ("labelRes", "I"),
                            ("icon", "I"),
                            ("targetSdkVersion", "I"),
                            ("flags", "I"),
                            ("uid", "I"),
                        ]
                        .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/content/pm/PackageInfo;"
                        && [
                            ("packageName", "Ljava/lang/String;"),
                            ("versionName", "Ljava/lang/String;"),
                            ("versionCode", "I"),
                            ("applicationInfo", "Landroid/content/pm/ApplicationInfo;"),
                        ]
                        .contains(&(field.name.as_str(), field.ty.as_str())))
                    || (class == "Landroid/content/pm/ResolveInfo;"
                        && [("activityInfo", "Landroid/content/pm/ActivityInfo;")]
                            .contains(&(field.name.as_str(), field.ty.as_str()))))
            {
                return Ok(Field {
                    class,
                    ..field.clone()
                });
            }
        }
        Err(fault("Ljava/lang/NoSuchFieldError;", field.key()))
    }
    pub(crate) fn parent(&self, class: &str) -> Option<String> {
        if crate::framework::graphics_enum_names(class).is_some() {
            return Some("Ljava/lang/Enum;".into());
        }
        if let Some(interface) = class.strip_prefix("Ldroidless/runtime/annotation/") {
            return Some(format!("L{interface}"));
        }
        if class == "Ldroidless/runtime/map/Entry;" {
            return Some("Ljava/util/Map$Entry;".into());
        }
        if let Some((d, c)) = self.class_location(class) {
            return self.apk.dex[d].classes[c].super_class.clone();
        }
        if let Some(parent) = exception_parent(class) {
            return Some(parent.into());
        }
        let parent = match class {
            "Landroid/view/MotionEvent;" => "Landroid/view/InputEvent;",
            "Landroid/text/TextPaint;" => "Landroid/graphics/Paint;",
            "Landroid/text/SpannableStringBuilder;" => "Landroid/text/Editable;",
            "Landroid/text/Editable;" => "Landroid/text/Spannable;",
            "Landroid/text/SpannableString;" => "Landroid/text/Spannable;",
            "Landroid/text/Spannable;" => "Landroid/text/Spanned;",
            "Landroid/text/SpannedString;" => "Landroid/text/Spanned;",
            "Landroid/text/Spanned;" => "Ljava/lang/CharSequence;",
            "Ljava/lang/CharSequence;" => "Ljava/lang/Object;",
            "Ljava/lang/Enum;" => "Ljava/lang/Object;",
            "Ljava/util/ListResourceBundle;" => "Ljava/util/ResourceBundle;",
            "Ljava/util/ResourceBundle;" => "Ljava/lang/Object;",
            "Landroid/text/TextUtils$TruncateAt;" => "Ljava/lang/Enum;",
            "Ljava/lang/ref/WeakReference;" => "Ljava/lang/ref/Reference;",
            "Ljava/io/FileInputStream;" => "Ljava/io/InputStream;",
            "Ljava/io/InputStream;" => "Ljava/lang/Object;",
            "Ljava/io/File;" => "Ljava/lang/Object;",
            "Ljava/lang/Double;" => "Ljava/lang/Number;",
            "Ljava/lang/Integer;" | "Ljava/lang/Long;" => "Ljava/lang/Number;",
            "Landroid/graphics/drawable/ColorDrawable;" => "Landroid/graphics/drawable/Drawable;",
            "Landroid/graphics/drawable/BitmapDrawable;" => "Landroid/graphics/drawable/Drawable;",
            "Landroid/graphics/drawable/GradientDrawable;" => {
                "Landroid/graphics/drawable/Drawable;"
            }
            "Landroid/graphics/drawable/LayerDrawable;"
            | "Landroid/graphics/drawable/RippleDrawable;"
            | "Landroid/graphics/drawable/InsetDrawable;" => "Landroid/graphics/drawable/Drawable;",
            "Landroid/animation/ObjectAnimator;" => "Landroid/animation/ValueAnimator;",
            "Landroid/animation/ValueAnimator;" => "Landroid/animation/Animator;",
            "Ljava/lang/reflect/Constructor;" => "Ljava/lang/reflect/AccessibleObject;",
            "Landroid/view/ViewGroup$MarginLayoutParams;" => {
                "Landroid/view/ViewGroup$LayoutParams;"
            }
            "Landroid/widget/TableLayout$LayoutParams;"
            | "Landroid/widget/TableRow$LayoutParams;" => {
                "Landroid/widget/LinearLayout$LayoutParams;"
            }
            "Landroid/widget/LinearLayout$LayoutParams;" => {
                "Landroid/view/ViewGroup$MarginLayoutParams;"
            }
            "Landroid/widget/FrameLayout$LayoutParams;" => {
                "Landroid/view/ViewGroup$MarginLayoutParams;"
            }
            "Landroid/widget/AbsListView$LayoutParams;" => "Landroid/view/ViewGroup$LayoutParams;",
            "Ljava/util/concurrent/ThreadPoolExecutor;" => "Ljava/util/concurrent/ExecutorService;",
            "Ljava/util/concurrent/ExecutorService;" => "Ljava/util/concurrent/Executor;",
            "Ljava/util/concurrent/FutureTask;" => "Ljava/util/concurrent/RunnableFuture;",
            "Ljava/util/concurrent/RunnableFuture;" => "Ljava/util/concurrent/Future;",
            "Ljava/util/HashSet;" => "Ljava/util/AbstractSet;",
            "Ljava/util/TreeSet;" => "Ljava/util/AbstractSet;",
            "Ljava/util/HashMap;" => "Ljava/util/AbstractMap;",
            "Ljava/util/Hashtable;" => "Ljava/util/Dictionary;",
            "Ljava/util/WeakHashMap;" => "Ljava/util/AbstractMap;",
            "Ljava/util/LinkedHashMap;" => "Ljava/util/HashMap;",
            "Ljava/util/concurrent/ConcurrentHashMap;" => "Ljava/util/HashMap;",
            "Ljava/util/ArrayList;" => "Ljava/util/AbstractList;",
            "Ljava/util/Stack;" => "Ljava/util/Vector;",
            "Ljava/util/Vector;" => "Ljava/util/AbstractList;",
            "Ldroidless/runtime/UnmodifiableRandomAccessList;" => {
                "Ldroidless/runtime/UnmodifiableList;"
            }
            "Ljava/util/concurrent/LinkedBlockingQueue;" => "Ljava/util/AbstractQueue;",
            "Ljava/util/AbstractSet;"
            | "Ljava/util/AbstractList;"
            | "Ljava/util/AbstractQueue;" => "Ljava/util/AbstractCollection;",
            "Landroid/widget/Button;"
            | "Landroid/widget/EditText;"
            | "Landroid/widget/CheckedTextView;" => "Landroid/widget/TextView;",
            "Landroid/widget/ImageButton;" => "Landroid/widget/ImageView;",
            "Landroid/view/ViewStub;" => "Landroid/view/View;",
            "Landroid/widget/GridView;" => "Landroid/widget/AbsListView;",
            "Landroid/widget/AbsListView;" => "Landroid/widget/AdapterView;",
            "Landroid/widget/AdapterView;" => "Landroid/view/ViewGroup;",
            "Landroid/widget/ListAdapter;" | "Landroid/widget/SpinnerAdapter;" => {
                "Landroid/widget/Adapter;"
            }
            "Landroid/database/DataSetObservable;" => "Landroid/database/Observable;",
            "Ldroidless/runtime/GridObserver;" => "Landroid/database/DataSetObserver;",
            "Landroid/widget/RelativeLayout;"
            | "Landroid/widget/ScrollView;"
            | "Landroid/widget/HorizontalScrollView;" => "Landroid/view/ViewGroup;",
            "Landroid/widget/ImageView;" | "Landroid/widget/Space;" => "Landroid/view/View;",
            "Landroid/widget/TextView;" | "Landroid/view/ViewGroup;" => "Landroid/view/View;",
            "Landroid/widget/LinearLayout;" | "Landroid/widget/FrameLayout;" => {
                "Landroid/view/ViewGroup;"
            }
            "Landroid/widget/TableRow;" | "Landroid/widget/TableLayout;" => {
                "Landroid/widget/LinearLayout;"
            }
            "Landroid/app/Activity;" | "Landroid/app/Application;" => {
                "Landroid/content/ContextWrapper;"
            }
            "Landroid/content/ContextWrapper;" => "Landroid/content/Context;",
            _ if class.starts_with('[') => "Ljava/lang/Object;",
            _ if class != "Ljava/lang/Object;" && !class.starts_with('[') => "Ljava/lang/Object;",
            _ => return None,
        };
        Some(parent.into())
    }
    pub(crate) fn is_a(&self, class: &str, target: &str) -> bool {
        let (mut class, mut target) = (class, target);
        while let Some(element) = class.strip_prefix('[') {
            if [
                "Ljava/lang/Object;",
                "Ljava/lang/Cloneable;",
                "Ljava/io/Serializable;",
            ]
            .contains(&target)
            {
                return true;
            }
            let Some(rhs) = target.strip_prefix('[') else {
                return false;
            };
            if !element.starts_with(['L', '[']) || !rhs.starts_with(['L', '[']) {
                return element == rhs;
            }
            (class, target) = (element, rhs);
        }
        let mut work = vec![class.to_owned()];
        let mut visited = BTreeSet::new();
        while let Some(current) = work.pop() {
            if current == target {
                return true;
            }
            if !visited.insert(current.clone()) {
                continue;
            }
            if visited.len() > 128 {
                return false;
            }
            if let Some((d, c)) = self.class_location(&current) {
                work.extend(self.apk.dex[d].classes[c].interfaces.iter().cloned());
            }
            if current == "Ljava/lang/String;" || current == "Ljava/lang/StringBuilder;" {
                work.push("Ljava/lang/CharSequence;".into());
                work.push("Ljava/io/Serializable;".into());
            }
            if current == "Ljava/lang/String;" {
                work.push("Ljava/lang/Comparable;".into());
            }
            if current == "Ljava/lang/Number;" || current == "Ljava/lang/Boolean;" {
                work.push("Ljava/io/Serializable;".into());
            }
            if current == "Landroid/view/ViewGroup;" {
                work.push("Landroid/view/ViewParent;".into());
            }
            if current == "Landroid/view/LayoutInflater$Factory2;"
                || current == crate::inflater::MERGER
            {
                work.push("Landroid/view/LayoutInflater$Factory;".into());
            }
            if current == crate::inflater::MERGER {
                work.push("Landroid/view/LayoutInflater$Factory2;".into());
            }
            if ["Landroid/net/Uri;", "Landroid/os/Bundle;"].contains(&current.as_str()) {
                work.push("Landroid/os/Parcelable;".into());
            }
            if current == "Landroid/os/Parcelable$ClassLoaderCreator;" {
                work.push("Landroid/os/Parcelable$Creator;".into());
            }
            if current.starts_with("Landroid/view/animation/") && current.ends_with("Interpolator;")
            {
                work.extend(
                    [
                        "Landroid/view/animation/Interpolator;",
                        "Landroid/animation/TimeInterpolator;",
                    ]
                    .map(String::from),
                );
            }
            if current == "Ljava/lang/Thread;" {
                work.push("Ljava/lang/Runnable;".into());
            }
            if current == "Landroid/animation/AnimatorListenerAdapter;" {
                work.extend(
                    [
                        "Landroid/animation/Animator$AnimatorListener;",
                        "Landroid/animation/Animator$AnimatorPauseListener;",
                    ]
                    .map(String::from),
                );
            }
            if current == "Landroid/widget/BaseAdapter;" {
                work.extend(
                    [
                        "Landroid/widget/ListAdapter;",
                        "Landroid/widget/SpinnerAdapter;",
                    ]
                    .map(String::from),
                );
            }
            if current == "Ldroidless/runtime/GridClick;" {
                work.push("Landroid/view/View$OnClickListener;".into());
            }
            if current == "Ldroidless/runtime/GestureTimer;"
                || current == "Ljava/util/TimerTask;"
                || current == "Ljava/util/concurrent/RunnableFuture;"
            {
                work.push("Ljava/lang/Runnable;".into());
            }
            if current == "Landroid/view/GestureDetector$SimpleOnGestureListener;" {
                work.extend(
                    [
                        "Landroid/view/GestureDetector$OnGestureListener;",
                        "Landroid/view/GestureDetector$OnDoubleTapListener;",
                    ]
                    .map(String::from),
                );
            }
            if current == "Landroid/os/Binder;" {
                work.push("Landroid/os/IBinder;".into());
            }
            if current == "Landroid/database/Cursor;" {
                work.extend(["Ljava/io/Closeable;", "Ljava/lang/AutoCloseable;"].map(String::from));
            }
            if current == "Landroid/app/Activity;" {
                work.push("Landroid/view/Window$Callback;".into());
            }
            if current == "Ljava/util/concurrent/LinkedBlockingQueue;" {
                work.extend(
                    [
                        "Ljava/util/concurrent/BlockingQueue;",
                        "Ljava/util/Queue;",
                        "Ljava/util/Collection;",
                        "Ljava/lang/Iterable;",
                        "Ljava/io/Serializable;",
                    ]
                    .map(String::from),
                );
            }
            if current == "Ljava/util/HashSet;" {
                work.extend(
                    [
                        "Ljava/util/Set;",
                        "Ljava/util/Collection;",
                        "Ljava/lang/Iterable;",
                        "Ljava/lang/Cloneable;",
                        "Ljava/io/Serializable;",
                    ]
                    .map(String::from),
                );
            }
            if current == "Ljava/util/TreeSet;" {
                work.extend(
                    [
                        "Ljava/util/SortedSet;",
                        "Ljava/util/NavigableSet;",
                        "Ljava/util/Set;",
                        "Ljava/util/Collection;",
                        "Ljava/lang/Iterable;",
                        "Ljava/lang/Cloneable;",
                        "Ljava/io/Serializable;",
                    ]
                    .map(String::from),
                );
            }
            if current == "Ljava/util/ArrayList;"
                || current == "Ljava/util/concurrent/CopyOnWriteArrayList;"
                || current == "Ljava/util/Vector;"
                || current == "Ljava/util/Stack;"
            {
                work.extend(
                    [
                        "Ljava/util/List;",
                        "Ljava/util/Collection;",
                        "Ljava/lang/Iterable;",
                        "Ljava/util/RandomAccess;",
                        "Ljava/lang/Cloneable;",
                        "Ljava/io/Serializable;",
                    ]
                    .map(String::from),
                );
            }
            if current == "Ldroidless/runtime/CollectionIterator;"
                || current == "Ldroidless/runtime/UnmodifiableIterator;"
                || current == "Ldroidless/runtime/SnapshotIterator;"
            {
                work.push("Ljava/util/Iterator;".into());
            }
            if current == "Ldroidless/runtime/UnmodifiableSet;" {
                work.extend(
                    [
                        "Ljava/util/Set;",
                        "Ljava/util/Collection;",
                        "Ljava/lang/Iterable;",
                        "Ljava/io/Serializable;",
                    ]
                    .map(String::from),
                );
            }
            if current == "Ldroidless/runtime/UnmodifiableList;" {
                work.extend(
                    [
                        "Ljava/util/List;",
                        "Ljava/util/Collection;",
                        "Ljava/lang/Iterable;",
                        "Ljava/io/Serializable;",
                    ]
                    .map(String::from),
                );
            }
            if current == "Ldroidless/runtime/UnmodifiableRandomAccessList;" {
                work.push("Ljava/util/RandomAccess;".into());
            }
            if [
                "Ljava/util/HashMap;",
                "Ljava/util/WeakHashMap;",
                "Ljava/util/Hashtable;",
            ]
            .contains(&current.as_str())
            {
                work.extend(
                    [
                        "Ljava/util/Map;",
                        "Ljava/lang/Cloneable;",
                        "Ljava/io/Serializable;",
                    ]
                    .map(String::from),
                );
            }
            if let Some(parent) = self.parent(&current) {
                work.push(parent);
            }
        }
        false
    }
    pub(crate) fn initialize(&mut self, class: &str) -> Result<()> {
        if let Some(cause) = self.failed_classes.get(class).copied() {
            return Err(self.guest_exception(
                "Ljava/lang/NoClassDefFoundError;",
                format!("could not initialize {class}"),
                cause,
            )?);
        }
        if self.initialized.contains(class) {
            return Ok(());
        }
        ensure!(
            self.initializing_depth < 128,
            "class initialization depth limit reached"
        );
        self.initializing_depth += 1;
        self.initialized.insert(class.to_owned());
        let result = (|| -> Result<()> {
            if let Some((d, c)) = self.class_location(class) {
                let def = self.apk.dex[d].classes[c].clone();
                if let Some(parent) = def.super_class {
                    self.initialize(&parent)?;
                }
                for (i, idx) in def.static_fields.iter().enumerate() {
                    let field = self.apk.dex[d].fields[*idx].clone();
                    let value = match def.static_values.get(i) {
                        Some(EncodedValue::Bits(v)) if field.ty == "J" || field.ty == "D" => {
                            wide(*v)
                        }
                        Some(EncodedValue::Bits(v)) => vec![Word::Bits(*v as u32)],
                        Some(EncodedValue::String(s)) => vec![self.intern(s.clone())?],
                        Some(EncodedValue::Null) | None => default_value(&field.ty),
                        Some(v) => bail!("unsupported static initializer value {v:?}"),
                    };
                    self.statics.insert(field.key(), value);
                }
                if def
                    .methods
                    .iter()
                    .any(|m| self.apk.dex[d].methods[m.index].name == "<clinit>")
                {
                    self.invoke(
                        Method {
                            class: class.into(),
                            name: "<clinit>".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![],
                        false,
                    )?;
                }
            }
            Ok(())
        })();
        self.initializing_depth -= 1;
        if let Err(error) = result {
            let cause = error
                .downcast_ref::<crate::interpreter::Thrown>()
                .map(|e| e.0);
            self.failed_classes.insert(class.into(), cause);
            if let Some(cause) = cause
                && !self.is_a(&self.heap.get(cause)?.class, "Ljava/lang/Error;")
            {
                let wrapped = self.guest_exception(
                    "Ljava/lang/ExceptionInInitializerError;",
                    format!("initialization failed for {class}"),
                    Some(cause),
                )?;
                let object = wrapped
                    .downcast_ref::<crate::interpreter::Thrown>()
                    .context("missing initialization exception")?
                    .0;
                self.failed_classes.insert(class.into(), Some(object));
                return Err(wrapped);
            }
            return Err(error);
        }
        Ok(())
    }
    pub(crate) fn guest_exception(
        &mut self,
        class: &str,
        message: String,
        cause: Option<Word>,
    ) -> Result<anyhow::Error> {
        let object = self.heap.instance(class)?;
        let message = self.heap.string(message)?;
        let fields = &mut self.heap.get_mut(object)?.fields;
        fields.insert("message".into(), vec![message]);
        if let Some(cause) = cause {
            fields.insert("cause".into(), vec![cause]);
        }
        self.capture_throwable_trace(object)?;
        self.throw_reference(object)
    }
    pub(crate) fn new_instance(&mut self, class: &str) -> Result<Word> {
        ensure!(
            self.class_location(class).is_some() || crate::framework::known_class(class),
            "unsupported class {class}"
        );
        self.initialize(class)?;
        let object = self.heap.instance(class)?;
        if self.heap.get(object)?.view.is_none() && self.is_a(class, "Landroid/view/View;") {
            let mut ancestor = class.to_owned();
            for _ in 0..64 {
                let Some(parent) = self.parent(&ancestor) else {
                    break;
                };
                if let Some(view) = ui::View::for_class(&parent) {
                    self.heap.get_mut(object)?.view = Some(view);
                    break;
                }
                ancestor = parent;
            }
        }
        Ok(object)
    }
    pub fn invoke(
        &mut self,
        method: Method,
        args: Vec<Word>,
        virtual_call: bool,
    ) -> Result<Vec<Word>> {
        let base = self.frames.len();
        self.sync_depth += 1;
        let result = (|| match self.begin_invoke(method, args, virtual_call)? {
            Some(words) => Ok(words),
            None => self.execute(base),
        })();
        self.sync_depth -= 1;
        result
    }
    /// Resolve a call: native words return immediately; a DEX call pushes one managed frame.
    pub(crate) fn begin_invoke(
        &mut self,
        method: Method,
        args: Vec<Word>,
        virtual_call: bool,
    ) -> Result<Option<Vec<Word>>> {
        ensure!(self.frames.len() < 128, "guest call stack limit reached");
        self.method_calls += 1;
        if self.trace.methods {
            eprintln!("invoke {}", method.key());
        }
        let mut class = if virtual_call {
            self.heap
                .get(*args.first().context("virtual call without receiver")?)?
                .class
                .clone()
        } else {
            method.class.clone()
        };
        for _ in 0..128 {
            if let Some((d, c)) = self.class_location(&class) {
                let signature = method.signature();
                if let Some(encoded) = self.apk.dex[d].classes[c]
                    .methods
                    .iter()
                    .find(|m| self.apk.dex[d].methods[m.index].signature() == signature)
                    .cloned()
                {
                    if encoded.access & 8 != 0 && method.name != "<clinit>" {
                        self.initialize(&class)?;
                    }
                    let target = self.apk.dex[d].methods[encoded.index].clone();
                    let code = encoded.code.with_context(|| {
                        format!("unsupported native/abstract method {}", target.key())
                    })?;
                    ensure!(
                        args.len() == usize::from(code.ins),
                        "argument word count mismatch for {}: expected {}, got {}",
                        target.key(),
                        code.ins,
                        args.len()
                    );
                    let mut registers = vec![Word::ZERO; usize::from(code.registers)];
                    let start = registers.len() - args.len();
                    registers[start..].copy_from_slice(&args);
                    self.frames.push(Frame {
                        dex: d,
                        method: target,
                        code,
                        registers,
                        pc: 0,
                        result: vec![],
                        exception: None,
                        return_pc: None,
                        looper_return: None,
                        monitors: vec![],
                    });
                    return Ok(None);
                }
            } else {
                let native = Method {
                    class: class.clone(),
                    ..method.clone()
                };
                if native.class == "Landroid/os/Looper;" && native.signature() == "loop()V" {
                    ensure!(args.is_empty(), "Looper.loop takes no arguments");
                    return self.begin_worker_looper();
                }
                if let Some(result) = self.native(&native, &args)? {
                    return Ok(Some(result));
                }
            }
            if method.name == "<init>" {
                break;
            }
            let Some(parent) = self.parent(&class) else {
                break;
            };
            class = parent;
        }
        bail!(
            "unsupported method {}\nRun with --trace-framework or --trace-bytecode to locate the compatibility blocker.",
            method.key()
        )
    }
    pub(crate) fn tick(&mut self) -> Result<()> {
        self.instructions += 1;
        self.budget += 1;
        ensure!(
            self.budget <= 5_000_000,
            "guest instruction budget exceeded (5 million per lifecycle/input transaction)"
        );
        Ok(())
    }
    pub(crate) fn array(&mut self, element: String, length: usize) -> Result<Word> {
        ensure!(length <= 1_000_000, "array length limit reached");
        self.heap.alloc(crate::heap::Object {
            class: format!("[{element}"),
            fields: BTreeMap::new(),
            data: Data::Array {
                values: vec![default_value(&element); length],
                element,
            },
            view: None,
        })
    }
}
pub fn descriptor(name: &str) -> String {
    format!("L{};", name.replace('.', "/"))
}
pub fn android_keycode(text: &str) -> Option<i32> {
    if text.chars().count() != 1 {
        return None;
    }
    let c = text.chars().next()?.to_ascii_lowercase();
    Some(match c {
        '0'..='9' => 7 + (c as i32 - '0' as i32),
        'a'..='z' => 29 + (c as i32 - 'a' as i32),
        '.' => 56,
        '-' => 69,
        '=' => 70,
        '+' => 81,
        '/' => 76,
        '*' => 17,
        '\r' | '\n' => 66,
        '\u{7f}' | '\u{8}' => 67,
        _ => return None,
    })
}
