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
        if ty == "Ljava/lang/String;" {
            if words[0] != Word::ZERO {
                ensure!(
                    self.heap.get(words[0])?.class == ty,
                    "Bundle expects String"
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
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
}
