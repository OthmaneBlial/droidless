//! Stateful Android components: typed Bundle extras and explicit Activity intents.
use crate::{
    activities::Navigation,
    heap::{Data, Word},
    vm::{Runtime, descriptor},
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;
use std::collections::BTreeMap;

impl Runtime {
    pub(crate) fn component_name_object(&mut self, package: String, class: String) -> Result<Word> {
        let object = self.heap.instance("Landroid/content/ComponentName;")?;
        let package = self.heap.string(package)?;
        let class = self.heap.string(class)?;
        let fields = &mut self.heap.get_mut(object)?.fields;
        fields.insert("packageName".into(), vec![package]);
        fields.insert("className".into(), vec![class]);
        Ok(object)
    }
    fn component_name_parts(&self, object: Word) -> Result<(String, String)> {
        let fields = &self.heap.get(object)?.fields;
        Ok((
            self.heap
                .text(
                    *fields
                        .get("packageName")
                        .and_then(|values| values.first())
                        .context("ComponentName package missing")?,
                )?
                .to_owned(),
            self.heap
                .text(
                    *fields
                        .get("className")
                        .and_then(|values| values.first())
                        .context("ComponentName class missing")?,
                )?
                .to_owned(),
        ))
    }
    fn manifest_activity(&self, class: &str) -> Option<&droidless_formats::xml::Element> {
        self.apk
            .manifest
            .document
            .children
            .iter()
            .find(|element| element.name == "application")?
            .children
            .iter()
            .filter(|element| element.name == "activity" || element.name == "activity-alias")
            .find(|element| {
                element.text("name").is_some_and(|name| {
                    let name = full_component_class(&self.apk.manifest.package, &name);
                    name == class
                })
            })
    }
    fn new_bundle(&mut self) -> Result<Word> {
        let object = self.heap.instance("Landroid/os/Bundle;")?;
        self.heap.get_mut(object)?.data = Data::Bundle(BTreeMap::new());
        Ok(object)
    }
    pub(crate) fn bundle_put(
        &mut self,
        object: Word,
        key: String,
        ty: String,
        words: Vec<Word>,
    ) -> Result<()> {
        ensure!(
            words.len() == crate::heap::default_value(&ty).len(),
            "invalid Bundle value width"
        );
        if ty.starts_with(['L', '[']) {
            if words[0] != Word::ZERO {
                ensure!(
                    self.is_a(&self.heap.get(words[0])?.class, &ty),
                    "Bundle value has the wrong type"
                );
            }
        } else {
            for word in &words {
                word.int()?;
            }
        }
        let Data::Bundle(values) = &mut self.heap.get_mut(object)?.data else {
            bail!("uninitialized Bundle");
        };
        ensure!(
            values.contains_key(&key) || values.len() < 16_384,
            "Bundle entry limit reached"
        );
        values.insert(key, (ty, words));
        Ok(())
    }
    fn bundle_get(
        &self,
        object: Word,
        key: &str,
        ty: &str,
        default: Vec<Word>,
    ) -> Result<Vec<Word>> {
        if object == Word::ZERO {
            return Ok(default);
        }
        let Data::Bundle(values) = &self.heap.get(object)?.data else {
            bail!("uninitialized Bundle");
        };
        Ok(values
            .get(key)
            .filter(|(kind, _)| kind == ty)
            .map(|(_, words)| words.clone())
            .unwrap_or(default))
    }
    fn intent_extras(&mut self, intent: Word, create: bool) -> Result<Word> {
        let extras = self
            .heap
            .get(intent)?
            .fields
            .get("extras")
            .and_then(|v| v.first())
            .copied()
            .unwrap_or(Word::ZERO);
        if extras != Word::ZERO || !create {
            return Ok(extras);
        }
        let extras = self.new_bundle()?;
        self.heap
            .get_mut(intent)?
            .fields
            .insert("extras".into(), vec![extras]);
        Ok(extras)
    }
    fn clone_bundle(&mut self, source: Word) -> Result<Word> {
        let object = self.new_bundle()?;
        let Data::Bundle(values) = &self.heap.get(source)?.data else {
            bail!("expected Bundle");
        };
        let data = Data::Bundle(values.clone());
        self.heap.get_mut(object)?.data = data;
        Ok(object)
    }
    pub(crate) fn component_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let arg = |n: usize| args.get(n).copied().context("component argument missing");
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let sig = method.signature();
        let mut result = vec![];
        match (method.class.as_str(), sig.as_str()) {
            (
                "Landroid/app/Application;",
                "registerActivityLifecycleCallbacks(Landroid/app/Application$ActivityLifecycleCallbacks;)V",
            )
            | (
                "Landroid/app/Application;",
                "unregisterActivityLifecycleCallbacks(Landroid/app/Application$ActivityLifecycleCallbacks;)V",
            ) => {
                let list = self.lifecycle_callbacks(receiver)?;
                self.invoke(
                    Method {
                        class: "Ljava/util/ArrayList;".into(),
                        name: if method.name.starts_with("unregister") {
                            "remove"
                        } else {
                            "add"
                        }
                        .into(),
                        parameters: vec!["Ljava/lang/Object;".into()],
                        returns: "Z".into(),
                    },
                    vec![list, arg(1)?],
                    true,
                )?;
            }
            ("Landroid/app/Activity;", "getApplication()Landroid/app/Application;") => {
                self.screen(receiver)?;
                result.push(self.application_context(receiver)?);
            }
            ("Landroid/app/Activity;", "getComponentName()Landroid/content/ComponentName;") => {
                let class = self
                    .heap
                    .get(receiver)?
                    .class
                    .trim_start_matches('L')
                    .trim_end_matches(';')
                    .replace('/', ".");
                result.push(self.component_name_object(self.apk.manifest.package.clone(), class)?);
            }
            (
                "Landroid/content/ComponentName;",
                "<init>(Ljava/lang/String;Ljava/lang/String;)V",
            ) => {
                let package = self.heap.text(arg(1)?)?.to_owned();
                let class = full_component_class(&package, self.heap.text(arg(2)?)?);
                let package = self.heap.string(package)?;
                let class = self.heap.string(class)?;
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert("packageName".into(), vec![package]);
                fields.insert("className".into(), vec![class]);
            }
            (
                "Landroid/content/ComponentName;",
                "<init>(Landroid/content/Context;Ljava/lang/String;)V",
            ) => {
                let package = self.apk.manifest.package.clone();
                let class = full_component_class(&package, self.heap.text(arg(2)?)?);
                let package = self.heap.string(package)?;
                let class = self.heap.string(class)?;
                let fields = &mut self.heap.get_mut(receiver)?.fields;
                fields.insert("packageName".into(), vec![package]);
                fields.insert("className".into(), vec![class]);
            }
            ("Landroid/content/ComponentName;", "getPackageName()Ljava/lang/String;") => {
                result.push(self.heap.get(receiver)?.fields["packageName"][0]);
            }
            ("Landroid/content/ComponentName;", "getClassName()Ljava/lang/String;") => {
                result.push(self.heap.get(receiver)?.fields["className"][0]);
            }
            (
                "Landroid/content/ComponentName;",
                "flattenToString()Ljava/lang/String;" | "flattenToShortString()Ljava/lang/String;",
            ) => {
                let (package, class) = self.component_name_parts(receiver)?;
                let class = if sig == "flattenToShortString()Ljava/lang/String;" {
                    class
                        .strip_prefix(&package)
                        .filter(|suffix| suffix.starts_with('.'))
                        .unwrap_or(&class)
                } else {
                    &class
                };
                result.push(self.heap.string(format!("{package}/{class}"))?);
            }
            ("Landroid/content/ComponentName;", "equals(Ljava/lang/Object;)Z") => {
                let other = arg(1)?;
                result.push(Word::from(i32::from(
                    other != Word::ZERO
                        && self
                            .heap
                            .get(other)
                            .is_ok_and(|object| object.class == "Landroid/content/ComponentName;")
                        && self.component_name_parts(receiver)?
                            == self.component_name_parts(other)?,
                )));
            }
            ("Landroid/content/ComponentName;", "hashCode()I") => {
                let (package, class) = self.component_name_parts(receiver)?;
                result.push(Word::from(
                    java_string_hash(&package)
                        .wrapping_mul(31)
                        .wrapping_add(java_string_hash(&class)),
                ));
            }
            (
                "Landroid/content/ComponentName;",
                "unflattenFromString(Ljava/lang/String;)Landroid/content/ComponentName;",
            ) => {
                let flattened = self.heap.text(arg(0)?)?;
                let Some((package, class)) = flattened.split_once('/') else {
                    return Ok(Some(vec![Word::ZERO]));
                };
                if package.is_empty() || class.is_empty() {
                    result.push(Word::ZERO);
                } else {
                    result.push(self.component_name_object(
                        package.to_owned(),
                        full_component_class(package, class),
                    )?);
                }
            }
            ("Landroid/os/Bundle;", "<init>()V") => {
                self.heap.get_mut(receiver)?.data = Data::Bundle(BTreeMap::new());
            }
            ("Landroid/os/Bundle;", "<init>(Landroid/os/Bundle;)V") => {
                let data = self.heap.get(arg(1)?)?.data.clone();
                ensure!(matches!(data, Data::Bundle(_)), "expected Bundle");
                self.heap.get_mut(receiver)?.data = data;
            }
            ("Landroid/os/Bundle;", sig)
                if [
                    "putString(Ljava/lang/String;Ljava/lang/String;)V",
                    "putInt(Ljava/lang/String;I)V",
                    "putLong(Ljava/lang/String;J)V",
                    "putFloat(Ljava/lang/String;F)V",
                    "putDouble(Ljava/lang/String;D)V",
                    "putBoolean(Ljava/lang/String;Z)V",
                    "putSerializable(Ljava/lang/String;Ljava/io/Serializable;)V",
                ]
                .contains(&sig) =>
            {
                let key = self.heap.text(arg(1)?)?.to_owned();
                self.bundle_put(
                    receiver,
                    key,
                    method.parameters[1].clone(),
                    args[2..].to_vec(),
                )?;
            }
            ("Landroid/os/Bundle;", sig)
                if [
                    "getString(Ljava/lang/String;)Ljava/lang/String;",
                    "getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
                    "getInt(Ljava/lang/String;)I",
                    "getInt(Ljava/lang/String;I)I",
                    "getLong(Ljava/lang/String;)J",
                    "getLong(Ljava/lang/String;J)J",
                    "getFloat(Ljava/lang/String;)F",
                    "getFloat(Ljava/lang/String;F)F",
                    "getDouble(Ljava/lang/String;)D",
                    "getDouble(Ljava/lang/String;D)D",
                    "getBoolean(Ljava/lang/String;)Z",
                    "getBoolean(Ljava/lang/String;Z)Z",
                    "getSerializable(Ljava/lang/String;)Ljava/io/Serializable;",
                ]
                .contains(&sig) =>
            {
                let default = if method.parameters.len() > 1 {
                    args[2..].to_vec()
                } else {
                    crate::heap::default_value(&method.returns)
                };
                result =
                    self.bundle_get(receiver, self.heap.text(arg(1)?)?, &method.returns, default)?;
            }
            ("Landroid/os/Bundle;", "containsKey(Ljava/lang/String;)Z") => {
                let Data::Bundle(values) = &self.heap.get(receiver)?.data else {
                    bail!("expected Bundle");
                };
                result.push(Word::from(i32::from(
                    values.contains_key(self.heap.text(arg(1)?)?),
                )));
            }
            ("Landroid/os/Bundle;", "get(Ljava/lang/String;)Ljava/lang/Object;") => {
                let Data::Bundle(values) = &self.heap.get(receiver)?.data else {
                    bail!("expected Bundle")
                };
                result.push(
                    values
                        .get(self.heap.text(arg(1)?)?)
                        .and_then(|(_, words)| words.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/os/Bundle;", "remove(Ljava/lang/String;)V") => {
                let key = self.heap.text(arg(1)?)?.to_owned();
                let Data::Bundle(values) = &mut self.heap.get_mut(receiver)?.data else {
                    bail!("expected Bundle");
                };
                values.remove(&key);
            }
            ("Landroid/os/Bundle;", "clear()V") => {
                let Data::Bundle(values) = &mut self.heap.get_mut(receiver)?.data else {
                    bail!("expected Bundle");
                };
                values.clear();
            }
            ("Landroid/os/Bundle;", "size()I") | ("Landroid/os/Bundle;", "isEmpty()Z") => {
                let Data::Bundle(values) = &self.heap.get(receiver)?.data else {
                    bail!("expected Bundle");
                };
                result.push(Word::from(if method.name == "size" {
                    values.len() as i32
                } else {
                    i32::from(values.is_empty())
                }));
            }
            ("Landroid/content/Intent;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/content/Intent;", "getData()Landroid/net/Uri;") => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("data")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/content/Intent;", "setData(Landroid/net/Uri;)Landroid/content/Intent;") => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("data".into(), vec![arg(1)?]);
                result.push(receiver);
            }
            ("Landroid/content/Intent;", "getComponent()Landroid/content/ComponentName;") => {
                let component = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("component")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                if component == Word::ZERO {
                    result.push(Word::ZERO);
                } else {
                    let class = self
                        .heap
                        .text(component)?
                        .trim_start_matches('L')
                        .trim_end_matches(';')
                        .replace('/', ".");
                    result.push(
                        self.component_name_object(self.apk.manifest.package.clone(), class)?,
                    );
                }
            }
            (
                "Landroid/content/Intent;",
                "setComponent(Landroid/content/ComponentName;)Landroid/content/Intent;",
            ) => {
                let (package, class) = self.component_name_parts(arg(1)?)?;
                let descriptor =
                    self.intern(descriptor(&full_component_class(&package, &class)))?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("component".into(), vec![descriptor]);
                result.push(receiver);
            }
            (
                "Landroid/content/Intent;",
                "makeMainActivity(Landroid/content/ComponentName;)Landroid/content/Intent;",
            ) => {
                let (package, class) = self.component_name_parts(arg(0)?)?;
                let intent = self.heap.instance("Landroid/content/Intent;")?;
                let descriptor =
                    self.intern(descriptor(&full_component_class(&package, &class)))?;
                self.heap
                    .get_mut(intent)?
                    .fields
                    .insert("component".into(), vec![descriptor]);
                result.push(intent);
            }
            (
                "Landroid/content/Intent;",
                "resolveActivity(Landroid/content/pm/PackageManager;)Landroid/content/ComponentName;",
            ) => {
                let component = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("component")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                if component == Word::ZERO {
                    result.push(Word::ZERO);
                } else {
                    let class = self
                        .heap
                        .text(component)?
                        .trim_start_matches('L')
                        .trim_end_matches(';')
                        .replace('/', ".");
                    result.push(
                        self.component_name_object(self.apk.manifest.package.clone(), class)?,
                    );
                }
            }
            ("Landroid/content/Intent;", "<init>(Landroid/content/Context;Ljava/lang/Class;)V")
            | (
                "Landroid/content/Intent;",
                "setClass(Landroid/content/Context;Ljava/lang/Class;)Landroid/content/Intent;",
            ) => {
                self.heap.get(arg(1)?)?;
                let target = *self
                    .heap
                    .get(arg(2)?)?
                    .fields
                    .get("name")
                    .and_then(|v| v.first())
                    .context("Intent expects Class literal")?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("component".into(), vec![target]);
                if method.name != "<init>" {
                    result.push(receiver);
                }
            }
            (
                "Landroid/content/Intent;",
                "setClassName(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
            ) => {
                let package = self.heap.text(arg(1)?)?;
                ensure!(
                    package == self.apk.manifest.package,
                    "cross-package Intent unsupported"
                );
                let name = self.heap.text(arg(2)?)?;
                let name = if name.starts_with('.') {
                    format!("{package}{name}")
                } else if !name.contains('.') {
                    format!("{package}.{name}")
                } else {
                    name.into()
                };
                let target = self.intern(descriptor(&name))?;
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("component".into(), vec![target]);
                result.push(receiver);
            }
            ("Landroid/content/Intent;", sig)
                if [
                    "putExtra(Ljava/lang/String;Ljava/lang/String;)Landroid/content/Intent;",
                    "putExtra(Ljava/lang/String;I)Landroid/content/Intent;",
                    "putExtra(Ljava/lang/String;J)Landroid/content/Intent;",
                    "putExtra(Ljava/lang/String;F)Landroid/content/Intent;",
                    "putExtra(Ljava/lang/String;D)Landroid/content/Intent;",
                    "putExtra(Ljava/lang/String;Z)Landroid/content/Intent;",
                    "putExtra(Ljava/lang/String;Ljava/io/Serializable;)Landroid/content/Intent;",
                ]
                .contains(&sig) =>
            {
                let key = self.heap.text(arg(1)?)?.to_owned();
                let extras = self.intent_extras(receiver, true)?;
                self.bundle_put(
                    extras,
                    key,
                    method.parameters[1].clone(),
                    args[2..].to_vec(),
                )?;
                result.push(receiver);
            }
            ("Landroid/content/Intent;", sig)
                if [
                    "getStringExtra(Ljava/lang/String;)Ljava/lang/String;",
                    "getIntExtra(Ljava/lang/String;I)I",
                    "getLongExtra(Ljava/lang/String;J)J",
                    "getFloatExtra(Ljava/lang/String;F)F",
                    "getDoubleExtra(Ljava/lang/String;D)D",
                    "getBooleanExtra(Ljava/lang/String;Z)Z",
                    "getSerializableExtra(Ljava/lang/String;)Ljava/io/Serializable;",
                ]
                .contains(&sig) =>
            {
                let extras = self.intent_extras(receiver, false)?;
                let default = if method.parameters.len() > 1 {
                    args[2..].to_vec()
                } else {
                    vec![Word::ZERO]
                };
                result =
                    self.bundle_get(extras, self.heap.text(arg(1)?)?, &method.returns, default)?;
            }
            (
                "Landroid/content/Intent;",
                "putExtras(Landroid/os/Bundle;)Landroid/content/Intent;",
            ) => {
                let Data::Bundle(source) = &self.heap.get(arg(1)?)?.data else {
                    bail!("putExtras expects Bundle");
                };
                let source = source.clone();
                let extras = self.intent_extras(receiver, true)?;
                for (key, (ty, words)) in source {
                    self.bundle_put(extras, key, ty, words)?;
                }
                result.push(receiver);
            }
            ("Landroid/content/Intent;", "getExtras()Landroid/os/Bundle;") => {
                let extras = self.intent_extras(receiver, false)?;
                result.push(if extras == Word::ZERO {
                    extras
                } else {
                    self.clone_bundle(extras)?
                });
            }
            ("Landroid/content/Context;", "startActivity(Landroid/content/Intent;)V") => {
                ensure!(
                    self.is_a(&self.heap.get(receiver)?.class, "Landroid/app/Activity;"),
                    "startActivity outside Activity requires unsupported NEW_TASK behavior"
                );
                let source = arg(1)?;
                let fields = self.heap.get(source)?.fields.clone();
                ensure!(
                    self.heap.get(source)?.class == "Landroid/content/Intent;",
                    "startActivity expects Intent"
                );
                let target = *fields
                    .get("component")
                    .and_then(|v| v.first())
                    .context("implicit/external Intent unsupported")?;
                let class = self.heap.text(target)?;
                ensure!(
                    self.apk
                        .manifest
                        .activities
                        .iter()
                        .any(|name| descriptor(name) == class),
                    "Intent target is not a declared APK Activity: {class}"
                );
                let copy = self.heap.instance("Landroid/content/Intent;")?;
                self.heap.get_mut(copy)?.fields = fields;
                let extras = self.intent_extras(source, false)?;
                if extras != Word::ZERO {
                    let extras = self.clone_bundle(extras)?;
                    self.heap
                        .get_mut(copy)?
                        .fields
                        .insert("extras".into(), vec![extras]);
                }
                self.queue_navigation(Navigation::Start(copy))?;
            }
            ("Landroid/app/Activity;", "startActivityForResult(Landroid/content/Intent;I)V")
            | (
                "Landroid/app/Activity;",
                "startActivityForResult(Landroid/content/Intent;ILandroid/os/Bundle;)V",
            ) => {
                self.invoke(
                    Method {
                        class: "Landroid/content/Context;".into(),
                        name: "startActivity".into(),
                        parameters: vec!["Landroid/content/Intent;".into()],
                        returns: "V".into(),
                    },
                    vec![receiver, arg(1)?],
                    false,
                )?;
            }
            ("Landroid/app/Activity;", "getIntent()Landroid/content/Intent;") => {
                result.push(self.screen(receiver)?.intent);
            }
            ("Landroid/app/Activity;", "finish()V")
            | ("Landroid/app/Activity;", "onBackPressed()V") => {
                self.queue_navigation(Navigation::Finish(receiver))?;
            }
            ("Landroid/app/Activity;", "isFinishing()Z") => {
                result.push(Word::from(i32::from(self.screen(receiver)?.finishing)));
            }
            ("Landroid/content/Context;", "getPackageName()Ljava/lang/String;") => {
                result.push(self.intern(self.apk.manifest.package.clone())?);
            }
            (
                "Landroid/content/Context;",
                "getPackageManager()Landroid/content/pm/PackageManager;",
            ) => {
                let manager = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:package-manager")
                    .and_then(|values| values.first())
                    .copied();
                result.push(if let Some(manager) = manager {
                    manager
                } else {
                    let manager = self.heap.instance("Landroid/content/pm/PackageManager;")?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:package-manager".into(), vec![manager]);
                    manager
                });
            }
            (
                "Landroid/content/pm/PackageManager;",
                "getActivityInfo(Landroid/content/ComponentName;I)Landroid/content/pm/ActivityInfo;",
            ) => {
                let (package, class) = self.component_name_parts(arg(1)?)?;
                let class = full_component_class(&package, &class);
                let Some(element) = self.manifest_activity(&class).cloned() else {
                    return Err(crate::heap::fault(
                        "Landroid/content/pm/PackageManager$NameNotFoundException;",
                        format!("activity {class} not found"),
                    ));
                };
                let info = self.heap.instance("Landroid/content/pm/ActivityInfo;")?;
                let app_info = self.heap.instance("Landroid/content/pm/ApplicationInfo;")?;
                let app_package = self.heap.string(package.clone())?;
                self.heap.get_mut(app_info)?.fields.insert(
                    "Landroid/content/pm/ApplicationInfo;->packageName:Ljava/lang/String;".into(),
                    vec![app_package],
                );
                let meta_data = self.new_bundle()?;
                for metadata in element
                    .children
                    .iter()
                    .filter(|child| child.name == "meta-data")
                {
                    let Some(name) = metadata.text("name") else {
                        continue;
                    };
                    let value = metadata.attr("value").and_then(|value| {
                        value.text.clone().or_else(|| {
                            (value.kind == 1)
                                .then(|| self.apk.resources.text(value.data).ok())
                                .flatten()
                        })
                    });
                    if let Some(value) = value {
                        let value = self.heap.string(value)?;
                        self.bundle_put(meta_data, name, "Ljava/lang/String;".into(), vec![value])?;
                    }
                }
                let name = self.heap.string(class.clone())?;
                let package_word = self.heap.string(package)?;
                let label_res = Word::from(element.number("label").unwrap_or(0) as i32);
                let icon = Word::from(element.number("icon").unwrap_or(0) as i32);
                let theme = Word::from(element.number("theme").unwrap_or(0) as i32);
                let parent = element
                    .text("parentActivityName")
                    .map(|value| full_component_class(&self.apk.manifest.package, &value));
                let parent = if let Some(parent) = parent {
                    self.heap.string(parent)?
                } else {
                    Word::ZERO
                };
                let fields = &mut self.heap.get_mut(info)?.fields;
                fields.insert(
                    "Landroid/content/pm/ActivityInfo;->name:Ljava/lang/String;".into(),
                    vec![name],
                );
                fields.insert(
                    "Landroid/content/pm/ActivityInfo;->packageName:Ljava/lang/String;".into(),
                    vec![package_word],
                );
                fields.insert(
                    "Landroid/content/pm/ActivityInfo;->metaData:Landroid/os/Bundle;".into(),
                    vec![meta_data],
                );
                fields.insert(
                    "Landroid/content/pm/ActivityInfo;->labelRes:I".into(),
                    vec![label_res],
                );
                fields.insert(
                    "Landroid/content/pm/ActivityInfo;->icon:I".into(),
                    vec![icon],
                );
                fields.insert("Landroid/content/pm/ActivityInfo;->applicationInfo:Landroid/content/pm/ApplicationInfo;".into(), vec![app_info]);
                fields.insert(
                    "Landroid/content/pm/ActivityInfo;->parentActivityName:Ljava/lang/String;"
                        .into(),
                    vec![parent],
                );
                fields.insert(
                    "Landroid/content/pm/ActivityInfo;->targetActivity:Ljava/lang/String;".into(),
                    vec![Word::ZERO],
                );
                fields.insert(
                    "Landroid/content/pm/ActivityInfo;->exported:Z".into(),
                    vec![Word::from(i32::from(
                        element.text("exported").as_deref() == Some("true"),
                    ))],
                );
                fields.insert(
                    "Landroid/content/pm/ActivityInfo;->permission:Ljava/lang/String;".into(),
                    vec![Word::ZERO],
                );
                fields.insert(
                    "Landroid/content/pm/ActivityInfo;->theme:I".into(),
                    vec![theme],
                );
                result.push(info);
            }
            ("Landroid/content/pm/ActivityInfo;", "getIconResource()I") => {
                result.push(
                    self.heap
                        .get(receiver)?
                        .fields
                        .get("Landroid/content/pm/ActivityInfo;->icon:I")
                        .and_then(|values| values.first())
                        .copied()
                        .unwrap_or(Word::ZERO),
                );
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
}

fn full_component_class(package: &str, class: &str) -> String {
    if class.starts_with('.') {
        format!("{package}{class}")
    } else if !class.contains('.') {
        format!("{package}.{class}")
    } else {
        class.to_owned()
    }
}

fn java_string_hash(value: &str) -> i32 {
    value.encode_utf16().fold(0i32, |hash, unit| {
        hash.wrapping_mul(31).wrapping_add(i32::from(unit))
    })
}
