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

impl Runtime {
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
                    ensure!(
                        !self
                            .window_word(receiver, "droidless:dialog:showing")?
                            .truth(),
                        "visible Dialog dismissal unsupported"
                    );
                    Ok(vec![])
                }
                "onCreate(Landroid/os/Bundle;)V" | "onContentChanged()V" => Ok(vec![]),
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
