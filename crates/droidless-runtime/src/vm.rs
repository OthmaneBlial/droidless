use crate::{
    heap::{Data, Heap, Word, default_value, exception_parent, wide},
    ui::{self, Node},
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::{
    apk::Apk,
    dex::{Code, EncodedValue, Method},
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
        })
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
            self.lifecycle_call(object, &class, "onCreate", vec![])?;
            self.statics
                .insert("droidless:application".into(), vec![object]);
        }
        let name = self
            .apk
            .manifest
            .main_activity
            .clone()
            .context("APK has no MAIN/LAUNCHER Activity")?;
        let intent = self.heap.instance("Landroid/content/Intent;")?;
        let class = descriptor(&name);
        self.create_screen(&class, intent)?;
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
        let parameters = if name == "onCreate" && self.is_a(class, "Landroid/app/Activity;") {
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
        Ok(())
    }
    pub fn close(&mut self) -> Result<()> {
        self.budget = 0;
        for screen in self.screens.values_mut() {
            screen.finishing = true;
        }
        if let Some(activity) = self.activity {
            let class = self.heap.get(activity)?.class.clone();
            for name in ["onPause", "onStop"] {
                self.lifecycle_call(activity, &class, name, vec![])?;
            }
        }
        while let Some(activity) = self.back_stack.pop() {
            let class = self.heap.get(activity)?.class.clone();
            self.lifecycle_call(activity, &class, "onDestroy", vec![])?;
        }
        self.screens.clear();
        self.activity = None;
        self.root = None;
        self.navigation.clear();
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
        if let Some(listener) = view.listener {
            let class = self.heap.get(listener)?.class.clone();
            self.invoke(
                Method {
                    class,
                    name: "onClick".into(),
                    parameters: vec!["Landroid/view/View;".into()],
                    returns: "V".into(),
                },
                vec![listener, word],
                true,
            )?;
            self.drain_navigation()?;
            self.collect();
            return Ok(true);
        }
        if let Some(name) = view.xml_click {
            let activity = self.activity.context("no Activity for XML click")?;
            let class = self.heap.get(activity)?.class.clone();
            self.invoke(
                Method {
                    class,
                    name,
                    parameters: vec!["Landroid/view/View;".into()],
                    returns: "V".into(),
                },
                vec![activity, word],
                true,
            )?;
            self.drain_navigation()?;
            self.collect();
            return Ok(true);
        }
        Ok(false)
    }
    pub fn click_text(&mut self, text: &str) -> Result<bool> {
        fn find(node: &Node, text: &str) -> Option<usize> {
            if node.view.text == text
                && (node.view.listener.is_some() || node.view.xml_click.is_some())
            {
                Some(node.handle)
            } else {
                node.children.iter().find_map(|c| find(c, text))
            }
        }
        let handle = find(&self.snapshot()?, text)
            .with_context(|| format!("no clickable View with text {text:?}"))?;
        self.click(handle)
    }
    pub fn edit(&mut self, handle: usize, text: &str) -> Result<()> {
        ensure!(text.len() <= 1_048_576, "text exceeds limit");
        let view = self
            .heap
            .get_mut(Word::Ref(handle))?
            .view
            .as_mut()
            .context("not a View")?;
        ensure!(view.editable && view.enabled, "View is not editable");
        view.text = text.to_owned();
        Ok(())
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
                    .values()
                    .flat_map(|s| s.root.into_iter().chain([s.intent])),
            )
            .chain(self.navigation.iter().map(|n| match n {
                crate::activities::Navigation::Start(intent)
                | crate::activities::Navigation::Finish(intent) => *intent,
            }))
            .chain(self.statics.values().flatten().copied())
            .chain(self.interned.values().copied())
            .chain(self.failed_classes.values().flatten().copied())
            .chain(self.frames.iter().flat_map(|f| {
                f.registers
                    .iter()
                    .chain(&f.result)
                    .copied()
                    .chain(f.exception)
            }));
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
    pub(crate) fn class_location(&self, class: &str) -> Option<(usize, usize)> {
        self.apk.dex.iter().enumerate().find_map(|(d, dex)| {
            dex.classes
                .iter()
                .position(|c| c.name == class)
                .map(|c| (d, c))
        })
    }
    pub(crate) fn parent(&self, class: &str) -> Option<String> {
        if let Some((d, c)) = self.class_location(class) {
            return self.apk.dex[d].classes[c].super_class.clone();
        }
        if let Some(parent) = exception_parent(class) {
            return Some(parent.into());
        }
        let parent = match class {
            "Landroid/widget/Button;" | "Landroid/widget/EditText;" => "Landroid/widget/TextView;",
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
        self.throw_reference(object)
    }
    pub(crate) fn new_instance(&mut self, class: &str) -> Result<Word> {
        ensure!(
            self.class_location(class).is_some() || crate::framework::known_class(class),
            "unsupported class {class}"
        );
        self.initialize(class)?;
        self.heap.instance(class)
    }
    pub fn invoke(
        &mut self,
        method: Method,
        args: Vec<Word>,
        virtual_call: bool,
    ) -> Result<Vec<Word>> {
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
                    let frame = self.frames.len();
                    self.frames.push(Frame {
                        dex: d,
                        method: target,
                        code,
                        registers,
                        pc: 0,
                        result: vec![],
                        exception: None,
                    });
                    let result = self.execute(frame);
                    let finished = self.frames.pop().context("frame stack underflow")?;
                    return result.with_context(|| {
                        format!(
                            "at {} [classes{}.dex, PC 0x{:04x}]",
                            finished.method.key(),
                            if d == 0 {
                                String::new()
                            } else {
                                (d + 1).to_string()
                            },
                            finished.pc
                        )
                    });
                }
            } else {
                let native = Method {
                    class: class.clone(),
                    ..method.clone()
                };
                if let Some(result) = self.native(&native, &args)? {
                    return Ok(result);
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
