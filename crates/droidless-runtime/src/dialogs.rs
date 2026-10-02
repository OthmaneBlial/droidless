use crate::{
    heap::Word,
    vm::Runtime,
    windows::{PARAMS, WINDOW},
};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

const DIALOG: &str = "Landroid/app/Dialog;";
const CONTEXT: &str = "Landroid/content/Context;";

impl Runtime {
    pub(crate) fn dialog_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
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
