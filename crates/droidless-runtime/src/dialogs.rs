use crate::{
    heap::Word,
    vm::Runtime,
    windows::{PARAMS, WINDOW},
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

const DIALOG: &str = "Landroid/app/Dialog;";
const CONTEXT: &str = "Landroid/content/Context;";
const LISTENERS: &str = "Landroid/app/Dialog$ListenersHandler;";
const MESSAGE: &str = "Landroid/os/Message;";

/// One managed Dialog surface. The Activity's root is never replaced by this tree.
pub struct DialogWindow {
    pub handle: usize,
    pub title: String,
    pub width: f32,
    pub height: f32,
    pub tree: crate::ui::Node,
}

impl Runtime {
    pub(crate) fn active_dialog(&self) -> Result<Option<Word>> {
        for dialog in self.dialogs.iter().rev() {
            if !self
                .window_word(*dialog, "droidless:dialog:hidden")?
                .truth()
            {
                return Ok(Some(*dialog));
            }
        }
        Ok(None)
    }
    pub(crate) fn dialog_geometry(&self, dialog: Word) -> Result<(Word, f32, f32)> {
        let window = self.window_word(dialog, "droidless:window")?;
        let root = self.window_word(window, "droidless:window:decor")?;
        let attributes = self.window_word(window, "droidless:window:attributes")?;
        let dimension = |name: &str, horizontal: bool, maximum: f32| -> Result<f32> {
            let size = self
                .window_word(
                    attributes,
                    &format!("Landroid/view/ViewGroup$LayoutParams;->{name}:I"),
                )?
                .int()?;
            ensure!(
                maximum.is_finite() && maximum > 0.0 && maximum <= 16384.0,
                "invalid Dialog viewport"
            );
            let value = match size {
                -1 => maximum,
                -2 => {
                    crate::ui::intrinsic_dimension(&self.heap, root, horizontal, maximum)?.max(1.0)
                }
                1..=16384 => size as f32,
                _ => anyhow::bail!("unsupported Dialog {name} {size}"),
            };
            Ok(value.min(maximum))
        };
        Ok((
            root,
            dimension("width", true, self.width)?,
            dimension("height", false, self.height)?,
        ))
    }
    pub(crate) fn input_surface(&self) -> Result<(Word, Word, f32, f32)> {
        if let Some(dialog) = self.active_dialog()? {
            let (root, width, height) = self.dialog_geometry(dialog)?;
            return Ok((dialog, root, width, height));
        }
        Ok((
            self.activity.unwrap_or(Word::ZERO),
            self.root.context("no content View")?,
            self.width,
            self.height,
        ))
    }
    pub(crate) fn input_view_allowed(&self, view: Word) -> Result<bool> {
        if let Some(dialog) = self.active_dialog()? {
            let (root, _, _) = self.dialog_geometry(dialog)?;
            return Ok(self.focus_root(view)? == root);
        }
        Ok(true)
    }
    pub fn dialog_layout_snapshots(&mut self) -> Result<Vec<DialogWindow>> {
        let mut surfaces = vec![];
        for dialog in self.dialogs.clone() {
            if !self.dialogs.contains(&dialog) {
                continue;
            }
            if self.window_word(dialog, "droidless:dialog:hidden")?.truth() {
                continue;
            }
            let (root, width, height) = self.dialog_geometry(dialog)?;
            let tree = self.layout_root(root, width, height)?;
            // Layout callbacks can dismiss their own Dialog. Do not recreate its native window.
            if !self.dialogs.contains(&dialog) {
                continue;
            }
            let window = self.window_word(dialog, "droidless:window")?;
            let attributes = self.window_word(window, "droidless:window:attributes")?;
            let title = self.window_word(attributes, "droidless:window:title")?;
            surfaces.push(DialogWindow {
                handle: dialog.reference()?,
                title: if title == Word::ZERO {
                    self.title.clone()
                } else {
                    self.heap.text(title)?.into()
                },
                width,
                height,
                tree,
            });
        }
        Ok(surfaces)
    }
    /// Closing a native panel is Back; outside clicks use the Window's separate policy.
    pub fn cancel_dialog_window(&mut self, handle: usize, outside: bool) -> Result<()> {
        if self.active_dialog()? != Some(Word::Ref(handle)) {
            return Ok(());
        }
        if outside {
            let dialog = Word::Ref(handle);
            let window = self.window_word(dialog, "droidless:window")?;
            if !self
                .window_word(window, "droidless:window:close-outside")?
                .truth()
            {
                return Ok(());
            }
        }
        self.back()
    }
    pub(crate) fn close_dialogs(&mut self) -> Result<()> {
        let mut count = 0;
        while let Some(dialog) = self.dialogs.last().copied() {
            ensure!(count < 32, "Dialog dismissal loop during close");
            count += 1;
            self.invoke(
                Method {
                    class: DIALOG.into(),
                    name: "dismiss".into(),
                    parameters: vec![],
                    returns: "V".into(),
                },
                vec![dialog],
                false,
            )?;
        }
        Ok(())
    }
    fn dialog_message(&mut self, dialog: Word, kind: &str) -> Result<()> {
        let message = self.window_word(dialog, &format!("droidless:dialog:{kind}-message"))?;
        if message == Word::ZERO {
            return Ok(());
        }
        let copy = self.invoke(
            Method {
                class: MESSAGE.into(),
                name: "obtain".into(),
                parameters: vec![MESSAGE.into()],
                returns: MESSAGE.into(),
            },
            vec![message],
            false,
        )?[0];
        self.native_roots.push(copy);
        self.invoke(
            Method {
                class: MESSAGE.into(),
                name: "sendToTarget".into(),
                parameters: vec![],
                returns: "V".into(),
            },
            vec![copy],
            true,
        )?;
        Ok(())
    }
    fn dialog_listeners_handler(&mut self, dialog: Word) -> Result<Word> {
        let cached = self.window_word(dialog, "droidless:dialog:listeners")?;
        if cached != Word::ZERO {
            return Ok(cached);
        }
        let weak = self.heap.instance("Ljava/lang/ref/WeakReference;")?;
        self.native_roots.push(weak);
        self.invoke(
            Method {
                class: "Ljava/lang/ref/WeakReference;".into(),
                name: "<init>".into(),
                parameters: vec!["Ljava/lang/Object;".into()],
                returns: "V".into(),
            },
            vec![weak, dialog],
            false,
        )?;
        let handler = self.heap.instance(LISTENERS)?;
        self.native_roots.push(handler);
        self.invoke(
            Method {
                class: "Landroid/os/Handler;".into(),
                name: "<init>".into(),
                parameters: vec![],
                returns: "V".into(),
            },
            vec![handler],
            false,
        )?;
        self.heap
            .get_mut(handler)?
            .fields
            .insert("droidless:dialog:weak".into(), vec![weak]);
        self.heap
            .get_mut(dialog)?
            .fields
            .insert("droidless:dialog:listeners".into(), vec![handler]);
        Ok(handler)
    }
    pub(crate) fn dialog_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class == LISTENERS && method.signature() == "handleMessage(Landroid/os/Message;)V"
        {
            self.require_main_thread()?;
            ensure!(
                self.sync_depth < 32,
                "Dialog listener callback nesting limit"
            );
            let handler = *args.first().context("Dialog listener handler missing")?;
            ensure!(
                self.heap.get(handler)?.class == LISTENERS,
                "invalid Dialog listener handler"
            );
            let message = *args.get(1).context("Dialog listener message missing")?;
            ensure!(
                self.is_a(&self.heap.get(message)?.class, MESSAGE),
                "Dialog listener requires Message"
            );
            let what = self
                .window_word(message, &format!("{MESSAGE}->what:I"))?
                .int()?;
            let (interface, name) = match what {
                0x43 => ("OnDismissListener", "onDismiss"),
                0x44 => ("OnCancelListener", "onCancel"),
                0x45 => ("OnShowListener", "onShow"),
                _ => return Ok(Some(vec![])),
            };
            let listener = self.message_word(message, "obj")?;
            let class = format!("Landroid/content/DialogInterface${interface};");
            ensure!(
                self.is_a(&self.heap.get(listener)?.class, &class),
                "invalid Dialog listener payload"
            );
            let roots = self.native_roots.len();
            self.native_roots.extend_from_slice(args);
            let delivered = (|| -> Result<Vec<Word>> {
                let weak = self.window_word(handler, "droidless:dialog:weak")?;
                let dialog = self.invoke(
                    Method {
                        class: "Ljava/lang/ref/Reference;".into(),
                        name: "get".into(),
                        parameters: vec![],
                        returns: "Ljava/lang/Object;".into(),
                    },
                    vec![weak],
                    true,
                )?[0];
                self.native_roots.extend([listener, dialog]);
                self.invoke(
                    Method {
                        class,
                        name: name.into(),
                        parameters: vec!["Landroid/content/DialogInterface;".into()],
                        returns: "V".into(),
                    },
                    vec![listener, dialog],
                    true,
                )
            })();
            self.native_roots.truncate(roots);
            return delivered.map(Some);
        }
        if method.class != DIALOG {
            return Ok(None);
        }
        let signature = method.signature();
        if !matches!(
            signature.as_str(),
            "<init>(Landroid/content/Context;)V"
                | "<init>(Landroid/content/Context;I)V"
                | "getContext()Landroid/content/Context;"
                | "getWindow()Landroid/view/Window;"
                | "getLayoutInflater()Landroid/view/LayoutInflater;"
                | "getCurrentFocus()Landroid/view/View;"
                | "findViewById(I)Landroid/view/View;"
                | "setContentView(I)V"
                | "setContentView(Landroid/view/View;)V"
                | "setContentView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V"
                | "addContentView(Landroid/view/View;Landroid/view/ViewGroup$LayoutParams;)V"
                | "setTitle(Ljava/lang/CharSequence;)V"
                | "requestWindowFeature(I)Z"
                | "create()V"
                | "isShowing()Z"
                | "show()V"
                | "hide()V"
                | "onStart()V"
                | "onStop()V"
                | "onAttachedToWindow()V"
                | "onDetachedFromWindow()V"
                | "onWindowFocusChanged(Z)V"
                | "setCancelable(Z)V"
                | "setCanceledOnTouchOutside(Z)V"
                | "setOnCancelListener(Landroid/content/DialogInterface$OnCancelListener;)V"
                | "setOnDismissListener(Landroid/content/DialogInterface$OnDismissListener;)V"
                | "setOnShowListener(Landroid/content/DialogInterface$OnShowListener;)V"
                | "setOnKeyListener(Landroid/content/DialogInterface$OnKeyListener;)V"
                | "setCancelMessage(Landroid/os/Message;)V"
                | "setDismissMessage(Landroid/os/Message;)V"
                | "cancel()V"
                | "dismiss()V"
                | "onBackPressed()V"
                | "getOwnerActivity()Landroid/app/Activity;"
                | "setOwnerActivity(Landroid/app/Activity;)V"
                | "onCreate(Landroid/os/Bundle;)V"
                | "onContentChanged()V"
                | "onWindowAttributesChanged(Landroid/view/WindowManager$LayoutParams;)V"
        ) {
            return Ok(None);
        }
        self.require_main_thread()?;
        ensure!(self.sync_depth < 32, "Dialog callback nesting limit");
        let arg = |index| args.get(index).copied().context("Dialog argument missing");
        let receiver = arg(0)?;
        ensure!(
            self.is_a(&self.heap.get(receiver)?.class, DIALOG),
            "invalid Dialog receiver"
        );
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        let roots = self.native_roots.len();
        self.native_roots.extend(args.iter().copied());
        let result = (|| -> Result<Vec<Word>> {
            match signature.as_str() {
                "<init>(Landroid/content/Context;)V" | "<init>(Landroid/content/Context;I)V" => {
                    let base = arg(1)?;
                    ensure!(
                        self.is_a(&self.heap.get(base)?.class, CONTEXT),
                        "Dialog expects Context"
                    );
                    let mut theme = if method.parameters.len() == 1 {
                        Word::ZERO
                    } else {
                        arg(2)?
                    };
                    theme.int()?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:dialog:cancelable".into(), vec![Word::from(1)]);
                    if theme == Word::ZERO {
                        let typed = self.heap.instance("Landroid/util/TypedValue;")?;
                        self.native_roots.push(typed);
                        let value = self
                            .invoke(
                                Method {
                                    class: CONTEXT.into(),
                                    name: "getTheme".into(),
                                    parameters: vec![],
                                    returns: "Landroid/content/res/Resources$Theme;".into(),
                                },
                                vec![base],
                                true,
                            )?
                            .first()
                            .copied()
                            .context("Context.getTheme returned no value")?;
                        self.native_roots.push(value);
                        self.invoke(
                            Method {
                                class: "Landroid/content/res/Resources$Theme;".into(),
                                name: "resolveAttribute".into(),
                                parameters: vec![
                                    "I".into(),
                                    "Landroid/util/TypedValue;".into(),
                                    "Z".into(),
                                ],
                                returns: "Z".into(),
                            },
                            vec![value, Word::from(0x01010308), typed, Word::from(1)],
                            true,
                        )?;
                        theme =
                            self.window_word(typed, "Landroid/util/TypedValue;->resourceId:I")?;
                    }
                    let context = self.heap.instance("Landroid/view/ContextThemeWrapper;")?;
                    self.native_roots.push(context);
                    self.invoke(
                        Method {
                            class: "Landroid/view/ContextThemeWrapper;".into(),
                            name: "<init>".into(),
                            parameters: vec![CONTEXT.into(), "I".into()],
                            returns: "V".into(),
                        },
                        vec![context, base, theme],
                        false,
                    )?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:dialog:context".into(), vec![context]);
                    let window = self.create_window(receiver, context)?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:window".into(), vec![window]);
                    let attributes = self.window_word(window, "droidless:window:attributes")?;
                    for name in ["width", "height"] {
                        self.heap.get_mut(attributes)?.fields.insert(
                            format!("Landroid/view/ViewGroup$LayoutParams;->{name}:I"),
                            vec![Word::from(-2)],
                        );
                    }
                    self.invoke(
                        Method {
                            class: WINDOW.into(),
                            name: "setGravity".into(),
                            parameters: vec!["I".into()],
                            returns: "V".into(),
                        },
                        vec![window, Word::from(17)],
                        true,
                    )?;
                    Ok(vec![])
                }
                "getContext()Landroid/content/Context;" => Ok(vec![
                    self.window_word(receiver, "droidless:dialog:context")?,
                ]),
                "getWindow()Landroid/view/Window;" => {
                    Ok(vec![self.window_word(receiver, "droidless:window")?])
                }
                "getOwnerActivity()Landroid/app/Activity;" => {
                    Ok(vec![self.window_word(
                        receiver,
                        "droidless:dialog:owner-activity",
                    )?])
                }
                "setOwnerActivity(Landroid/app/Activity;)V" => {
                    ensure!(
                        self.is_a(&self.heap.get(arg(1)?)?.class, "Landroid/app/Activity;"),
                        "Dialog owner must be Activity"
                    );
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:dialog:owner-activity".into(), vec![arg(1)?]);
                    Ok(vec![])
                }
                "create()V" => {
                    if !self
                        .window_word(receiver, "droidless:dialog:created")?
                        .truth()
                    {
                        self.invoke(
                            Method {
                                class: DIALOG.into(),
                                name: "onCreate".into(),
                                parameters: vec!["Landroid/os/Bundle;".into()],
                                returns: "V".into(),
                            },
                            vec![receiver, Word::ZERO],
                            true,
                        )?;
                        self.heap
                            .get_mut(receiver)?
                            .fields
                            .insert("droidless:dialog:created".into(), vec![Word::from(1)]);
                    }
                    Ok(vec![])
                }
                "isShowing()Z" => Ok(vec![
                    self.window_word(receiver, "droidless:dialog:showing")?,
                ]),
                "show()V" => {
                    if self
                        .window_word(receiver, "droidless:dialog:showing")?
                        .truth()
                    {
                        self.heap
                            .get_mut(receiver)?
                            .fields
                            .insert("droidless:dialog:hidden".into(), vec![Word::ZERO]);
                        return Ok(vec![]);
                    }
                    ensure!(
                        !self
                            .window_word(receiver, "droidless:dialog:transition")?
                            .truth(),
                        "recursive Dialog show"
                    );
                    ensure!(self.dialogs.len() < 32, "Dialog window limit (32)");
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:dialog:transition".into(), vec![Word::from(1)]);
                    let started = (|| -> Result<()> {
                        self.heap
                            .get_mut(receiver)?
                            .fields
                            .insert("droidless:dialog:canceled".into(), vec![Word::ZERO]);
                        self.invoke(
                            Method {
                                class: DIALOG.into(),
                                name: "create".into(),
                                parameters: vec![],
                                returns: "V".into(),
                            },
                            vec![receiver],
                            false,
                        )?;
                        self.invoke(
                            Method {
                                class: DIALOG.into(),
                                name: "onStart".into(),
                                parameters: vec![],
                                returns: "V".into(),
                            },
                            vec![receiver],
                            true,
                        )?;
                        let window = self.window_word(receiver, "droidless:window")?;
                        ensure!(
                            !self
                                .window_word(window, "droidless:window:destroyed")?
                                .truth(),
                            "cannot show destroyed Dialog Window"
                        );
                        self.window_root(window, true)?;
                        self.dialog_geometry(receiver)?;
                        let token = self.heap.instance("Landroid/os/Binder;")?;
                        let fields = &mut self.heap.get_mut(receiver)?.fields;
                        fields.insert("droidless:dialog:token".into(), vec![token]);
                        fields.insert("droidless:dialog:showing".into(), vec![Word::from(1)]);
                        fields.insert("droidless:dialog:hidden".into(), vec![Word::ZERO]);
                        self.dialogs.push(receiver);
                        self.touch = None;
                        self.host_window_focused = false;
                        self.invoke(
                            Method {
                                class: DIALOG.into(),
                                name: "onAttachedToWindow".into(),
                                parameters: vec![],
                                returns: "V".into(),
                            },
                            vec![receiver],
                            true,
                        )?;
                        self.dialog_message(receiver, "show")
                    })();
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:dialog:transition".into(), vec![Word::ZERO]);
                    started?;
                    Ok(vec![])
                }
                "hide()V" => {
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:dialog:hidden".into(), vec![Word::from(1)]);
                    self.touch = None;
                    self.host_window_focused = false;
                    Ok(vec![])
                }
                "setCancelable(Z)V" | "setCanceledOnTouchOutside(Z)V" => {
                    let value = Word::from(i32::from(arg(1)?.int()? != 0));
                    if method.name == "setCancelable" || value.truth() {
                        self.heap
                            .get_mut(receiver)?
                            .fields
                            .insert("droidless:dialog:cancelable".into(), vec![value]);
                    }
                    if method.name == "setCanceledOnTouchOutside" {
                        let window = self.window_word(receiver, "droidless:window")?;
                        self.invoke(
                            Method {
                                class: WINDOW.into(),
                                name: "setCloseOnTouchOutside".into(),
                                parameters: vec!["Z".into()],
                                returns: "V".into(),
                            },
                            vec![window, value],
                            true,
                        )?;
                    }
                    Ok(vec![])
                }
                "setOnCancelListener(Landroid/content/DialogInterface$OnCancelListener;)V"
                | "setOnDismissListener(Landroid/content/DialogInterface$OnDismissListener;)V"
                | "setOnShowListener(Landroid/content/DialogInterface$OnShowListener;)V"
                | "setOnKeyListener(Landroid/content/DialogInterface$OnKeyListener;)V" => {
                    let listener = arg(1)?;
                    ensure!(
                        listener == Word::ZERO
                            || self.is_a(&self.heap.get(listener)?.class, &method.parameters[0]),
                        "invalid Dialog listener"
                    );
                    let (kind, what) = match method.name.as_str() {
                        "setOnCancelListener" => ("cancel", 0x44),
                        "setOnDismissListener" => ("dismiss", 0x43),
                        "setOnShowListener" => ("show", 0x45),
                        _ => ("key", 0),
                    };
                    let message = if what == 0 || listener == Word::ZERO {
                        listener
                    } else {
                        let handler = self.dialog_listeners_handler(receiver)?;
                        self.invoke(
                            Method {
                                class: "Landroid/os/Handler;".into(),
                                name: "obtainMessage".into(),
                                parameters: vec!["I".into(), "Ljava/lang/Object;".into()],
                                returns: MESSAGE.into(),
                            },
                            vec![handler, Word::from(what), listener],
                            true,
                        )?[0]
                    };
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(format!("droidless:dialog:{kind}-message"), vec![message]);
                    Ok(vec![])
                }
                "setCancelMessage(Landroid/os/Message;)V"
                | "setDismissMessage(Landroid/os/Message;)V" => {
                    let message = arg(1)?;
                    ensure!(
                        message == Word::ZERO || self.is_a(&self.heap.get(message)?.class, MESSAGE),
                        "Dialog message requires Message"
                    );
                    let kind = if method.name == "setCancelMessage" {
                        "cancel"
                    } else {
                        "dismiss"
                    };
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(format!("droidless:dialog:{kind}-message"), vec![message]);
                    Ok(vec![])
                }
                "onBackPressed()V" => {
                    if self
                        .window_word(receiver, "droidless:dialog:cancelable")?
                        .truth()
                    {
                        self.invoke(
                            Method {
                                class: DIALOG.into(),
                                name: "cancel".into(),
                                parameters: vec![],
                                returns: "V".into(),
                            },
                            vec![receiver],
                            true,
                        )?;
                    }
                    Ok(vec![])
                }
                "cancel()V" => {
                    let message = self.window_word(receiver, "droidless:dialog:cancel-message")?;
                    if message != Word::ZERO
                        && !self
                            .window_word(receiver, "droidless:dialog:canceled")?
                            .truth()
                    {
                        self.heap
                            .get_mut(receiver)?
                            .fields
                            .insert("droidless:dialog:canceled".into(), vec![Word::from(1)]);
                        self.dialog_message(receiver, "cancel")?;
                    }
                    self.invoke(
                        Method {
                            class: DIALOG.into(),
                            name: "dismiss".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        true,
                    )
                }
                "dismiss()V" => {
                    if !self
                        .window_word(receiver, "droidless:dialog:showing")?
                        .truth()
                    {
                        return Ok(vec![]);
                    }
                    ensure!(
                        !self
                            .window_word(receiver, "droidless:dialog:transition")?
                            .truth(),
                        "recursive Dialog dismissal"
                    );
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:dialog:transition".into(), vec![Word::from(1)]);
                    let detached = self.invoke(
                        Method {
                            class: DIALOG.into(),
                            name: "onDetachedFromWindow".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        true,
                    );
                    let stopped = self.invoke(
                        Method {
                            class: DIALOG.into(),
                            name: "onStop".into(),
                            parameters: vec![],
                            returns: "V".into(),
                        },
                        vec![receiver],
                        true,
                    );
                    self.dialogs.retain(|dialog| *dialog != receiver);
                    self.touch = None;
                    self.host_window_focused = false;
                    let fields = &mut self.heap.get_mut(receiver)?.fields;
                    fields.insert("droidless:dialog:showing".into(), vec![Word::ZERO]);
                    fields.insert("droidless:dialog:token".into(), vec![Word::ZERO]);
                    fields.insert("droidless:dialog:transition".into(), vec![Word::ZERO]);
                    detached?;
                    stopped?;
                    self.dialog_message(receiver, "dismiss")?;
                    Ok(vec![])
                }
                "onCreate(Landroid/os/Bundle;)V"
                | "onContentChanged()V"
                | "onStart()V"
                | "onStop()V"
                | "onAttachedToWindow()V"
                | "onDetachedFromWindow()V" => Ok(vec![]),
                "onWindowFocusChanged(Z)V" => {
                    arg(1)?.int()?;
                    Ok(vec![])
                }
                "onWindowAttributesChanged(Landroid/view/WindowManager$LayoutParams;)V" => {
                    ensure!(
                        self.is_a(&self.heap.get(arg(1)?)?.class, PARAMS),
                        "Dialog attributes must be WindowManager.LayoutParams"
                    );
                    Ok(vec![])
                }
                _ => {
                    let window = self.window_word(receiver, "droidless:window")?;
                    let name = if method.name == "requestWindowFeature" {
                        "requestFeature"
                    } else {
                        &method.name
                    };
                    let mut forwarded = args.to_vec();
                    forwarded[0] = window;
                    self.invoke(
                        Method {
                            class: WINDOW.into(),
                            name: name.into(),
                            ..method.clone()
                        },
                        forwarded,
                        true,
                    )
                }
            }
        })();
        self.native_roots.truncate(roots);
        result.map(Some)
    }
}
