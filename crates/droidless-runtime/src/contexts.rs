use crate::{heap::Word, vm::Runtime};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

const WRAPPER: &str = "Landroid/content/ContextWrapper;";
const THEMED: &str = "Landroid/view/ContextThemeWrapper;";
const CONTEXT: &str = "Landroid/content/Context;";
const THEME: &str = "Landroid/content/res/Resources$Theme;";
const THEME_ID: &str = "droidless:theme:resource";

impl Runtime {
    fn context_field(&self, context: Word, key: &str) -> Result<Word> {
        Ok(self
            .heap
            .get(context)?
            .fields
            .get(key)
            .and_then(|words| words.first())
            .copied()
            .unwrap_or(Word::ZERO))
    }
    fn initialize_context_theme(&mut self, context: Word) -> Result<Word> {
        let mut theme = self.context_field(context, "droidless:theme")?;
        let first = theme == Word::ZERO;
        if first {
            let resources = self
                .invoke(
                    Method {
                        class: CONTEXT.into(),
                        name: "getResources".into(),
                        parameters: vec![],
                        returns: "Landroid/content/res/Resources;".into(),
                    },
                    vec![context],
                    true,
                )?
                .first()
                .copied()
                .context("Context callback returned no value")?;
            self.native_roots.push(resources);
            theme = self
                .invoke(
                    Method {
                        class: "Landroid/content/res/Resources;".into(),
                        name: "newTheme".into(),
                        parameters: vec![],
                        returns: THEME.into(),
                    },
                    vec![resources],
                    true,
                )?
                .first()
                .copied()
                .context("Context callback returned no value")?;
            ensure!(
                self.is_a(&self.heap.get(theme)?.class, THEME),
                "Resources.newTheme returned invalid Theme"
            );
            self.heap
                .get_mut(context)?
                .fields
                .insert("droidless:theme".into(), vec![theme]);
            let base = self
                .invoke(
                    Method {
                        class: WRAPPER.into(),
                        name: "getBaseContext".into(),
                        parameters: vec![],
                        returns: CONTEXT.into(),
                    },
                    vec![context],
                    true,
                )?
                .first()
                .copied()
                .context("Context callback returned no value")?;
            self.native_roots.push(base);
            let original = self
                .invoke(
                    Method {
                        class: CONTEXT.into(),
                        name: "getTheme".into(),
                        parameters: vec![],
                        returns: THEME.into(),
                    },
                    vec![base],
                    true,
                )?
                .first()
                .copied()
                .context("Context callback returned no value")?;
            if original != Word::ZERO {
                self.native_roots.push(original);
                self.invoke(
                    Method {
                        class: THEME.into(),
                        name: "setTo".into(),
                        parameters: vec![THEME.into()],
                        returns: "V".into(),
                    },
                    vec![theme, original],
                    true,
                )?;
            }
        }
        let style = self.context_field(context, THEME_ID)?;
        self.invoke(
            Method {
                class: THEMED.into(),
                name: "onApplyThemeResource".into(),
                parameters: vec![THEME.into(), "I".into(), "Z".into()],
                returns: "V".into(),
            },
            vec![context, theme, style, Word::from(i32::from(first))],
            true,
        )?;
        Ok(theme)
    }
    pub(crate) fn context_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method.signature();
        let themed = method.class == THEMED
            && matches!(
                signature.as_str(),
                "<init>()V"
                    | "<init>(Landroid/content/Context;I)V"
                    | "setTheme(I)V"
                    | "getThemeResId()I"
                    | "getTheme()Landroid/content/res/Resources$Theme;"
                    | "getResources()Landroid/content/res/Resources;"
                    | "getSystemService(Ljava/lang/String;)Ljava/lang/Object;"
                    | "onApplyThemeResource(Landroid/content/res/Resources$Theme;IZ)V"
            );
        let delegated = method.class == WRAPPER
            && matches!(
                signature.as_str(),
                "getResources()Landroid/content/res/Resources;"
                    | "getAssets()Landroid/content/res/AssetManager;"
                    | "getTheme()Landroid/content/res/Resources$Theme;"
                    | "setTheme(I)V"
                    | "getThemeResId()I"
                    | "getSystemService(Ljava/lang/String;)Ljava/lang/Object;"
                    | "getApplicationContext()Landroid/content/Context;"
                    | "getPackageName()Ljava/lang/String;"
                    | "getApplicationInfo()Landroid/content/pm/ApplicationInfo;"
                    | "getPackageManager()Landroid/content/pm/PackageManager;"
                    | "getClassLoader()Ljava/lang/ClassLoader;"
                    | "getSharedPreferences(Ljava/lang/String;I)Landroid/content/SharedPreferences;"
                    | "getCacheDir()Ljava/io/File;"
                    | "getFilesDir()Ljava/io/File;"
                    | "getDir(Ljava/lang/String;I)Ljava/io/File;"
                    | "getDatabasePath(Ljava/lang/String;)Ljava/io/File;"
                    | "getFileStreamPath(Ljava/lang/String;)Ljava/io/File;"
                    | "startActivity(Landroid/content/Intent;)V"
            );
        if !themed && !delegated {
            return Ok(None);
        }
        let arg = |index| {
            args.get(index)
                .copied()
                .context("Context wrapper argument missing")
        };
        let receiver = arg(0)?;
        ensure!(
            self.is_a(&self.heap.get(receiver)?.class, &method.class),
            "invalid Context wrapper receiver"
        );
        // Native Activity/Application contexts have no attached base in this profile.
        if delegated
            && !self
                .heap
                .get(receiver)?
                .fields
                .contains_key("droidless:context:base")
        {
            return Ok(None);
        }
        if themed {
            self.require_main_thread()?;
        }
        ensure!(self.sync_depth < 32, "Context wrapper nesting limit");
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        let roots = self.native_roots.len();
        self.native_roots.extend(args.iter().copied());
        let result = (|| -> Result<Vec<Word>> {
            if delegated {
                let base = self.context_field(receiver, "droidless:context:base")?;
                let mut forwarded = args.to_vec();
                forwarded[0] = base;
                return self.invoke(
                    Method {
                        class: CONTEXT.into(),
                        ..method.clone()
                    },
                    forwarded,
                    true,
                );
            }
            match signature.as_str() {
                "<init>()V" | "<init>(Landroid/content/Context;I)V" => {
                    let (base, style) = if method.parameters.is_empty() {
                        (Word::ZERO, Word::ZERO)
                    } else {
                        (arg(1)?, arg(2)?)
                    };
                    style.int()?;
                    self.invoke(
                        Method {
                            class: WRAPPER.into(),
                            name: "<init>".into(),
                            parameters: vec![CONTEXT.into()],
                            returns: "V".into(),
                        },
                        vec![receiver, base],
                        false,
                    )?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(THEME_ID.into(), vec![style]);
                    Ok(vec![])
                }
                "getThemeResId()I" => Ok(vec![self.context_field(receiver, THEME_ID)?]),
                "setTheme(I)V" => {
                    arg(1)?.int()?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(THEME_ID.into(), vec![arg(1)?]);
                    self.initialize_context_theme(receiver)?;
                    Ok(vec![])
                }
                "getTheme()Landroid/content/res/Resources$Theme;" => {
                    let cached = self.context_field(receiver, "droidless:theme")?;
                    if cached != Word::ZERO {
                        return Ok(vec![cached]);
                    }
                    if self.context_field(receiver, THEME_ID)?.int()? == 0 {
                        let info = self
                            .invoke(
                                Method {
                                    class: CONTEXT.into(),
                                    name: "getApplicationInfo".into(),
                                    parameters: vec![],
                                    returns: "Landroid/content/pm/ApplicationInfo;".into(),
                                },
                                vec![receiver],
                                true,
                            )?
                            .first()
                            .copied()
                            .context("Context callback returned no value")?;
                        let target = self
                            .context_field(
                                info,
                                "Landroid/content/pm/ApplicationInfo;->targetSdkVersion:I",
                            )?
                            .int()?;
                        let id = match target {
                            i32::MIN..=10 => 16973829,
                            11..=13 => 16973931,
                            14..=9999 => 16974120,
                            _ => 16974143,
                        };
                        self.heap
                            .get_mut(receiver)?
                            .fields
                            .insert(THEME_ID.into(), vec![Word::from(id)]);
                    }
                    Ok(vec![self.initialize_context_theme(receiver)?])
                }
                "getResources()Landroid/content/res/Resources;" => {
                    let cached = self.context_field(receiver, "droidless:resources")?;
                    if cached != Word::ZERO {
                        return Ok(vec![cached]);
                    }
                    let resources = self
                        .invoke(
                            Method {
                                class: WRAPPER.into(),
                                ..method.clone()
                            },
                            args.to_vec(),
                            false,
                        )?
                        .first()
                        .copied()
                        .context("Context callback returned no value")?;
                    ensure!(
                        self.is_a(
                            &self.heap.get(resources)?.class,
                            "Landroid/content/res/Resources;"
                        ),
                        "Context.getResources returned invalid Resources"
                    );
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:resources".into(), vec![resources]);
                    Ok(vec![resources])
                }
                "getSystemService(Ljava/lang/String;)Ljava/lang/Object;" => {
                    let inflater = self.heap.text(arg(1)?)? == "layout_inflater";
                    if inflater {
                        let cached = self.context_field(receiver, "droidless:layout-inflater")?;
                        if cached != Word::ZERO {
                            return Ok(vec![cached]);
                        }
                    }
                    let base = self
                        .invoke(
                            Method {
                                class: WRAPPER.into(),
                                name: "getBaseContext".into(),
                                parameters: vec![],
                                returns: CONTEXT.into(),
                            },
                            vec![receiver],
                            true,
                        )?
                        .first()
                        .copied()
                        .context("Context callback returned no value")?;
                    self.native_roots.push(base);
                    if !inflater {
                        return self.invoke(
                            Method {
                                class: CONTEXT.into(),
                                ..method.clone()
                            },
                            vec![base, arg(1)?],
                            true,
                        );
                    }
                    let original = self.layout_inflater_from(base)?;
                    self.native_roots.push(original);
                    let clone = self
                        .invoke(
                            Method {
                                class: "Landroid/view/LayoutInflater;".into(),
                                name: "cloneInContext".into(),
                                parameters: vec![CONTEXT.into()],
                                returns: "Landroid/view/LayoutInflater;".into(),
                            },
                            vec![original, receiver],
                            true,
                        )?
                        .first()
                        .copied()
                        .context("Context callback returned no value")?;
                    ensure!(
                        self.is_a(
                            &self.heap.get(clone)?.class,
                            "Landroid/view/LayoutInflater;"
                        ),
                        "cloneInContext returned invalid LayoutInflater"
                    );
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:layout-inflater".into(), vec![clone]);
                    Ok(vec![clone])
                }
                "onApplyThemeResource(Landroid/content/res/Resources$Theme;IZ)V" => {
                    arg(3)?.int()?;
                    self.invoke(
                        Method {
                            class: THEME.into(),
                            name: "applyStyle".into(),
                            parameters: vec!["I".into(), "Z".into()],
                            returns: "V".into(),
                        },
                        vec![arg(1)?, arg(2)?, Word::from(1)],
                        true,
                    )
                }
                _ => unreachable!(),
            }
        })();
        self.native_roots.truncate(roots);
        result.map(Some)
    }
}
