use crate::{heap::Word, vm::Runtime};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

pub(crate) const WINDOW: &str = "Landroid/view/Window;";
pub(crate) const PARAMS: &str = "Landroid/view/WindowManager$LayoutParams;";
const CONTEXT: &str = "Landroid/content/Context;";
const CALLBACK: &str = "Landroid/view/Window$Callback;";
const FRAME: &str = "Landroid/widget/FrameLayout;";

impl Runtime {
    pub(crate) fn window_word(&self, object: Word, key: &str) -> Result<Word> {
        Ok(self
            .heap
            .get(object)?
            .fields
            .get(key)
            .and_then(|words| words.first())
            .copied()
            .unwrap_or(Word::ZERO))
    }
    pub(crate) fn create_window(&mut self, owner: Word, context: Word) -> Result<Word> {
        let window = self.heap.instance(WINDOW)?;
        let attributes = self.heap.instance(PARAMS)?;
        self.native_roots.extend([window, attributes]);
        for (key, value) in [
            ("owner", owner),
            ("context", context),
            ("callback", owner),
            ("attributes", attributes),
        ] {
            self.heap
                .get_mut(window)?
                .fields
                .insert(format!("droidless:window:{key}"), vec![value]);
        }
        for (name, value) in [("width", -1), ("height", -1)] {
            self.heap.get_mut(attributes)?.fields.insert(
                format!("Landroid/view/ViewGroup$LayoutParams;->{name}:I"),
                vec![Word::from(value)],
            );
        }
        self.heap
            .get_mut(attributes)?
            .fields
            .insert(format!("{PARAMS}->type:I"), vec![Word::from(2)]);
        self.heap
            .get_mut(attributes)?
            .fields
            .insert(format!("{PARAMS}->gravity:I"), vec![Word::ZERO]);
        self.heap
            .get_mut(window)?
            .fields
            .insert("droidless:window:features".into(), vec![Word::from(0x41)]);
        let name = self.heap.string("window".into())?;
        self.native_roots.push(name);
        let manager = self
            .invoke(
                Method {
                    class: CONTEXT.into(),
                    name: "getSystemService".into(),
                    parameters: vec!["Ljava/lang/String;".into()],
                    returns: "Ljava/lang/Object;".into(),
                },
                vec![context, name],
                true,
            )?
            .first()
            .copied()
            .context("window service returned no value")?;
        ensure!(
            self.is_a(
                &self.heap.get(manager)?.class,
                "Landroid/view/WindowManager;"
            ),
            "window service is not a WindowManager"
        );
        self.heap
            .get_mut(window)?
            .fields
            .insert("droidless:window:manager".into(), vec![manager]);
        Ok(window)
    }
    pub(crate) fn window_root(&mut self, window: Word, create: bool) -> Result<Word> {
        let owner = self.window_word(window, "droidless:window:owner")?;
        if self.is_a(&self.heap.get(owner)?.class, "Landroid/app/Activity;") {
            if let Some(root) = self.screen(owner)?.root {
                return Ok(root);
            }
            if !create {
                return Ok(Word::ZERO);
            }
            let context = self.window_word(window, "droidless:window:context")?;
            let root = self.window_frame(context)?;
            self.set_content(owner, root)?;
            return Ok(root);
        }
        ensure!(
            self.is_a(&self.heap.get(owner)?.class, "Landroid/app/Dialog;"),
            "unsupported Window owner"
        );
        let root = self.window_word(window, "droidless:window:decor")?;
        if root != Word::ZERO || !create {
            return Ok(root);
        }
        let context = self.window_word(window, "droidless:window:context")?;
        let root = self.window_frame(context)?;
        self.native_roots.push(root);
        let content = self.window_frame(context)?;
        self.native_roots.push(content);
        self.view_mut(root)?.height = -2.0;
        self.view_mut(content)?.height = -2.0;
        self.view_mut(content)?.id = 0x01020002;
        self.invoke(
            Method {
                class: "Landroid/view/ViewGroup;".into(),
                name: "addView".into(),
                parameters: vec!["Landroid/view/View;".into()],
                returns: "V".into(),
            },
            vec![root, content],
            true,
        )?;
        self.heap
            .get_mut(window)?
            .fields
            .insert("droidless:window:decor".into(), vec![root]);
        self.heap
            .get_mut(window)?
            .fields
            .insert("droidless:window:content".into(), vec![content]);
        Ok(root)
    }
    fn window_frame(&mut self, context: Word) -> Result<Word> {
        let frame = self.heap.instance(FRAME)?;
        self.invoke(
            Method {
                class: FRAME.into(),
                name: "<init>".into(),
                parameters: vec![CONTEXT.into()],
                returns: "V".into(),
            },
            vec![frame, context],
            false,
        )?;
        Ok(frame)
    }
    fn window_callback(
        &mut self,
        window: Word,
        name: &str,
        parameters: Vec<String>,
        args: Vec<Word>,
    ) -> Result<()> {
        let callback = self.window_word(window, "droidless:window:callback")?;
        if callback != Word::ZERO {
            self.invoke(
                Method {
                    class: CALLBACK.into(),
                    name: name.into(),
                    parameters,
                    returns: "V".into(),
                },
                std::iter::once(callback).chain(args).collect(),
                true,
            )?;
        }
        Ok(())
    }
    pub(crate) fn window_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method.signature();
        let activity = method.class == "Landroid/app/Activity;"
            && matches!(
                signature.as_str(),
                "getWindow()Landroid/view/Window;"
                    | "onContentChanged()V"
                    | "onWindowAttributesChanged(Landroid/view/WindowManager$LayoutParams;)V"
            );
        let window = method.class == WINDOW
            && matches!(
                signature.as_str(),
                "getContext()Landroid/content/Context;"
                    | "getWindowManager()Landroid/view/WindowManager;"
                    | "getLayoutInflater()Landroid/view/LayoutInflater;"
                    | "getDecorView()Landroid/view/View;"
                    | "peekDecorView()Landroid/view/View;"
                    | "findViewById(I)Landroid/view/View;"
                    | "getCurrentFocus()Landroid/view/View;"
                    | "getCallback()Landroid/view/Window$Callback;"
                    | "setCallback(Landroid/view/Window$Callback;)V"
                    | "setContentView(I)V"
                    | "setContentView(Landroid/view/View;)V"
                    | "setContentView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V"
                    | "addContentView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V"
                    | "setTitle(Ljava/lang/CharSequence;)V"
                    | "requestFeature(I)Z"
                    | "hasFeature(I)Z"
                    | "getAttributes()Landroid/view/WindowManager$LayoutParams;"
                    | "setAttributes(Landroid/view/WindowManager$LayoutParams;)V"
                    | "setLayout(II)V"
                    | "setFlags(II)V"
                    | "addFlags(I)V"
                    | "clearFlags(I)V"
                    | "setGravity(I)V"
                    | "isDestroyed()Z"
                    | "destroy()V"
                    | "setCloseOnTouchOutside(Z)V"
                    | "setCloseOnTouchOutsideIfNotSet(Z)V"
            );
        let title = method.class == PARAMS
            && matches!(
                signature.as_str(),
                "getTitle()Ljava/lang/CharSequence;" | "setTitle(Ljava/lang/CharSequence;)V"
            );
        if !activity && !window && !title {
            return Ok(None);
        }
        self.require_main_thread()?;
        ensure!(self.sync_depth < 32, "Window callback nesting limit");
        let arg = |n| args.get(n).copied().context("Window argument missing");
        let receiver = arg(0)?;
        ensure!(
            self.is_a(&self.heap.get(receiver)?.class, &method.class),
            "invalid Window bridge receiver"
        );
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        let roots = self.native_roots.len();
        self.native_roots.extend(args.iter().copied());
        let result = (|| -> Result<Vec<Word>> {
            if activity {
                if method.name == "onContentChanged" {
                    return Ok(vec![]);
                }
                if method.name == "onWindowAttributesChanged" {
                    ensure!(
                        self.is_a(&self.heap.get(arg(1)?)?.class, PARAMS),
                        "Activity attributes must be WindowManager.LayoutParams"
                    );
                    return Ok(vec![]);
                }
                self.screen(receiver)?;
                let cached = self.window_word(receiver, "droidless:window")?;
                let value = if cached != Word::ZERO {
                    cached
                } else {
                    let window = self.create_window(receiver, receiver)?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:window".into(), vec![window]);
                    window
                };
                return Ok(vec![value]);
            }
            if title {
                if method.name == "setTitle" {
                    let text = if arg(1)? == Word::ZERO {
                        self.heap.string(String::new())?
                    } else {
                        self.heap.string(self.heap.text(arg(1)?)?.to_owned())?
                    };
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:window:title".into(), vec![text]);
                    return Ok(vec![]);
                }
                let text = self.window_word(receiver, "droidless:window:title")?;
                return Ok(vec![if text == Word::ZERO {
                    self.heap.string(String::new())?
                } else {
                    text
                }]);
            }
            match signature.as_str() {
                "setCloseOnTouchOutside(Z)V" | "setCloseOnTouchOutsideIfNotSet(Z)V" => {
                    let value = Word::from(i32::from(arg(1)?.int()? != 0));
                    if method.name == "setCloseOnTouchOutside"
                        || !self
                            .window_word(receiver, "droidless:window:outside-set")?
                            .truth()
                    {
                        let fields = &mut self.heap.get_mut(receiver)?.fields;
                        fields.insert("droidless:window:close-outside".into(), vec![value]);
                        fields.insert("droidless:window:outside-set".into(), vec![Word::from(1)]);
                    }
                    Ok(vec![])
                }
                "getContext()Landroid/content/Context;" => Ok(vec![
                    self.window_word(receiver, "droidless:window:context")?,
                ]),
                "getWindowManager()Landroid/view/WindowManager;" => Ok(vec![
                    self.window_word(receiver, "droidless:window:manager")?,
                ]),
                "getLayoutInflater()Landroid/view/LayoutInflater;" => {
                    let context = self.window_word(receiver, "droidless:window:context")?;
                    Ok(vec![self.layout_inflater_from(context)?])
                }
                "getDecorView()Landroid/view/View;" | "peekDecorView()Landroid/view/View;" => {
                    Ok(vec![
                        self.window_root(receiver, method.name == "getDecorView")?,
                    ])
                }
                "findViewById(I)Landroid/view/View;" => {
                    let root = self.window_root(receiver, false)?;
                    Ok(vec![if root == Word::ZERO {
                        Word::ZERO
                    } else {
                        self.find_view(root, arg(1)?.int()? as u32, 0)?
                            .unwrap_or(Word::ZERO)
                    }])
                }
                "getCurrentFocus()Landroid/view/View;" => {
                    let root = self.window_root(receiver, false)?;
                    Ok(vec![if root == Word::ZERO {
                        Word::ZERO
                    } else {
                        self.find_focus(root)?
                    }])
                }
                "getCallback()Landroid/view/Window$Callback;" => Ok(vec![
                    self.window_word(receiver, "droidless:window:callback")?,
                ]),
                "setCallback(Landroid/view/Window$Callback;)V" => {
                    let value = arg(1)?;
                    ensure!(
                        value == Word::ZERO || self.is_a(&self.heap.get(value)?.class, CALLBACK),
                        "Window callback must implement Window.Callback"
                    );
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:window:callback".into(), vec![value]);
                    Ok(vec![])
                }
                "setContentView(I)V"
                | "setContentView(Landroid/view/View;)V"
                | "setContentView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V"
                | "addContentView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V" => {
                    let owner = self.window_word(receiver, "droidless:window:owner")?;
                    let dialog = self.is_a(&self.heap.get(owner)?.class, "Landroid/app/Dialog;");
                    let container = if dialog {
                        self.window_root(receiver, true)?;
                        self.window_word(receiver, "droidless:window:content")?
                    } else {
                        Word::ZERO
                    };
                    if dialog && method.name == "setContentView" {
                        self.invoke(
                            Method {
                                class: "Landroid/view/ViewGroup;".into(),
                                name: "removeAllViews".into(),
                                parameters: vec![],
                                returns: "V".into(),
                            },
                            vec![container],
                            true,
                        )?;
                    }
                    let view = if method.parameters == ["I"] {
                        let context = self.window_word(receiver, "droidless:window:context")?;
                        let inflater = self.layout_inflater_from(context)?;
                        self.native_roots.push(inflater);
                        self.inflate_id(
                            arg(1)?.int()? as u32,
                            0,
                            context,
                            inflater,
                            container,
                            dialog,
                        )?
                    } else {
                        arg(1)?
                    };
                    ensure!(
                        self.heap.get(view)?.view.is_some(),
                        "Window content must be a View"
                    );
                    self.native_roots.push(view);
                    if dialog {
                        if view != container {
                            let mut parameters = vec!["Landroid/view/View;".into()];
                            let mut call_args = vec![container, view];
                            if method.parameters.len() == 2 {
                                parameters.push(method.parameters[1].clone());
                                call_args.push(arg(2)?);
                            }
                            self.invoke(
                                Method {
                                    class: "Landroid/view/ViewGroup;".into(),
                                    name: "addView".into(),
                                    parameters,
                                    returns: "V".into(),
                                },
                                call_args,
                                true,
                            )?;
                        }
                    } else {
                        ensure!(
                            method.name != "addContentView",
                            "Activity addContentView requires a content container"
                        );
                        if method.parameters.len() == 2 {
                            self.invoke(
                                Method {
                                    class: "Landroid/view/View;".into(),
                                    name: "setLayoutParams".into(),
                                    parameters: vec![method.parameters[1].clone()],
                                    returns: "V".into(),
                                },
                                vec![view, arg(2)?],
                                true,
                            )?;
                        }
                        self.install_android_content_id(view, 0)?;
                        self.set_content(owner, view)?;
                    }
                    self.window_callback(receiver, "onContentChanged", vec![], vec![])?;
                    Ok(vec![])
                }
                "requestFeature(I)Z" | "hasFeature(I)Z" => {
                    let feature = arg(1)?.int()?;
                    ensure!(
                        (0..=13).contains(&feature),
                        "unsupported API-21 Window feature {feature}"
                    );
                    let mut features = self
                        .window_word(receiver, "droidless:window:features")?
                        .int()?;
                    if method.name == "requestFeature" {
                        features |= 1 << feature;
                        self.heap.get_mut(receiver)?.fields.insert(
                            "droidless:window:features".into(),
                            vec![Word::from(features)],
                        );
                    }
                    Ok(vec![Word::from(i32::from(features & (1 << feature) != 0))])
                }
                "getAttributes()Landroid/view/WindowManager$LayoutParams;" => Ok(vec![
                    self.window_word(receiver, "droidless:window:attributes")?,
                ]),
                "setAttributes(Landroid/view/WindowManager$LayoutParams;)V" => {
                    ensure!(
                        self.is_a(&self.heap.get(arg(1)?)?.class, PARAMS),
                        "Window attributes must be WindowManager.LayoutParams"
                    );
                    let target = self.window_word(receiver, "droidless:window:attributes")?;
                    self.heap.get_mut(target)?.fields = self.heap.get(arg(1)?)?.fields.clone();
                    self.window_callback(
                        receiver,
                        "onWindowAttributesChanged",
                        vec![PARAMS.into()],
                        vec![target],
                    )?;
                    Ok(vec![])
                }
                "setLayout(II)V"
                | "setGravity(I)V"
                | "setFlags(II)V"
                | "addFlags(I)V"
                | "clearFlags(I)V"
                | "setTitle(Ljava/lang/CharSequence;)V" => {
                    let attributes = self.window_word(receiver, "droidless:window:attributes")?;
                    if method.name == "setLayout" {
                        for (name, word) in [("width", arg(1)?), ("height", arg(2)?)] {
                            word.int()?;
                            self.heap.get_mut(attributes)?.fields.insert(
                                format!("Landroid/view/ViewGroup$LayoutParams;->{name}:I"),
                                vec![word],
                            );
                        }
                    } else if method.name == "setGravity" {
                        arg(1)?.int()?;
                        self.heap
                            .get_mut(attributes)?
                            .fields
                            .insert(format!("{PARAMS}->gravity:I"), vec![arg(1)?]);
                    } else if method.name == "setTitle" {
                        self.invoke(
                            Method {
                                class: PARAMS.into(),
                                ..method.clone()
                            },
                            vec![attributes, arg(1)?],
                            true,
                        )?;
                        let owner = self.window_word(receiver, "droidless:window:owner")?;
                        if self.is_a(&self.heap.get(owner)?.class, "Landroid/app/Activity;") {
                            let text = self.window_word(attributes, "droidless:window:title")?;
                            self.set_title(owner, self.heap.text(text)?.to_owned())?;
                        }
                    } else {
                        let old = self
                            .window_word(attributes, &format!("{PARAMS}->flags:I"))?
                            .int()?;
                        let value = arg(1)?.int()?;
                        let flags = match method.name.as_str() {
                            "addFlags" => old | value,
                            "clearFlags" => old & !value,
                            _ => {
                                let mask = arg(2)?.int()?;
                                (old & !mask) | (value & mask)
                            }
                        };
                        self.heap
                            .get_mut(attributes)?
                            .fields
                            .insert(format!("{PARAMS}->flags:I"), vec![Word::from(flags)]);
                    }
                    self.window_callback(
                        receiver,
                        "onWindowAttributesChanged",
                        vec![PARAMS.into()],
                        vec![attributes],
                    )?;
                    Ok(vec![])
                }
                "isDestroyed()Z" => Ok(vec![
                    self.window_word(receiver, "droidless:window:destroyed")?,
                ]),
                "destroy()V" => {
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:window:destroyed".into(), vec![Word::from(1)]);
                    Ok(vec![])
                }
                _ => unreachable!(),
            }
        })();
        self.native_roots.truncate(roots);
        result.map(Some)
    }
}
