//! SharedPreferences uses the heap's existing typed map and GC traversal.
use crate::{
    heap::{Data, Word, bits64, fault, wide},
    storage::{Value, preference_name},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;
use std::collections::BTreeMap;

impl Runtime {
    fn open_preferences(&mut self, name: String, mode: i32) -> Result<Word> {
        ensure!(mode == 0, "only MODE_PRIVATE preferences are supported");
        preference_name(&name)?;
        if let Some(object) = self.preferences.get(&name) {
            return Ok(*object);
        }
        ensure!(
            self.preferences.len() < 128,
            "preference store limit reached (128)"
        );
        let disk = self
            .storage
            .as_ref()
            .map(|s| s.load(&name))
            .transpose()?
            .unwrap_or_default();
        let mut values = BTreeMap::new();
        for (key, value) in disk {
            let (ty, words) = match value {
                Value::String(s) => ("Ljava/lang/String;", vec![self.heap.string(s)?]),
                Value::Int(v) => ("I", vec![Word::from(v)]),
                Value::Long(v) => ("J", wide(v as u64)),
                Value::Float(v) => ("F", vec![Word::Bits(v)]),
                Value::Boolean(v) => ("Z", vec![Word::from(i32::from(v))]),
            };
            values.insert(key, (ty.into(), words));
        }
        let object = self.heap.instance("Landroid/content/SharedPreferences;")?;
        self.heap.get_mut(object)?.data = Data::Bundle(values);
        let name_word = self.heap.string(name.clone())?;
        self.heap
            .get_mut(object)?
            .fields
            .insert("name".into(), vec![name_word]);
        self.preferences.insert(name, object);
        Ok(object)
    }
    fn commit_preferences(&mut self, editor: Word) -> Result<bool> {
        let object = self.heap.get(editor)?;
        let owner = *object
            .fields
            .get("owner")
            .and_then(|v| v.first())
            .context("uninitialized preference editor")?;
        let clear = object
            .fields
            .get("clear")
            .and_then(|v| v.first())
            .is_some_and(|w| w.truth());
        let Data::Bundle(changes) = &object.data else {
            bail!("invalid preference editor");
        };
        let changes = changes.clone();
        let Data::Bundle(values) = &self.heap.get(owner)?.data else {
            bail!("invalid preferences");
        };
        let mut values = if clear {
            BTreeMap::new()
        } else {
            values.clone()
        };
        for (key, (ty, words)) in changes {
            if ty == "remove" {
                values.remove(&key);
            } else {
                values.insert(key, (ty, words));
            }
        }
        ensure!(values.len() <= 16_384, "preference entry limit reached");
        self.heap.get_mut(owner)?.data = Data::Bundle(values.clone());
        let object = self.heap.get_mut(editor)?;
        object.data = Data::Bundle(BTreeMap::new());
        object.fields.remove("clear");
        if self.storage.is_none() {
            return Ok(true);
        }
        let name = *self
            .heap
            .get(owner)?
            .fields
            .get("name")
            .and_then(|v| v.first())
            .context("preference name missing")?;
        let name = self.heap.text(name)?.to_owned();
        let mut disk = BTreeMap::new();
        for (key, (ty, words)) in values {
            let value = match ty.as_str() {
                "Ljava/lang/String;" => Value::String(self.heap.text(words[0])?.to_owned()),
                "I" => Value::Int(words[0].int()?),
                "J" => Value::Long(bits64(&words)? as i64),
                "F" => Value::Float(words[0].int()? as u32),
                "Z" => Value::Boolean(words[0].int()? != 0),
                _ => bail!("invalid preference value type {ty}"),
            };
            disk.insert(key, value);
        }
        match self
            .storage
            .as_mut()
            .context("storage lost")?
            .save(&name, disk)
        {
            Ok(()) => Ok(true),
            Err(error) => {
                eprintln!("DROIDLESS: preference write failed: {error:#}");
                Ok(false)
            }
        }
    }
    pub(crate) fn preference_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let sig = method.signature();
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let arg = |n: usize| args.get(n).copied().context("preference argument missing");
        if [
            "Landroid/content/SharedPreferences;",
            "Landroid/content/SharedPreferences$Editor;",
        ]
        .contains(&method.class.as_str())
        {
            ensure!(
                self.heap.get(receiver)?.class == method.class,
                "invalid preference receiver"
            );
        }
        let mut result = vec![];
        match (method.class.as_str(), sig.as_str()) {
            ("Landroid/content/Context;", "getSharedPreferences(Ljava/lang/String;I)Landroid/content/SharedPreferences;") => {
                let name = self.heap.text(arg(1)?)?.to_owned();
                result.push(self.open_preferences(name, arg(2)?.int()?)?);
            }
            ("Landroid/app/Activity;", "getPreferences(I)Landroid/content/SharedPreferences;") => {
                let class = self.heap.get(receiver)?.class.trim_start_matches('L').trim_end_matches(';').replace('/', ".");
                let prefix = format!("{}.", self.apk.manifest.package);
                let name = class.strip_prefix(&prefix).unwrap_or(&class).to_owned();
                result.push(self.open_preferences(name, arg(1)?.int()?)?);
            }
            ("Landroid/content/Context;", "getApplicationContext()Landroid/content/Context;") => {
                let object = if let Some(object) = self.statics.get("droidless:application").and_then(|v| v.first()).copied() {
                    object
                } else if self.is_a(&self.heap.get(receiver)?.class, "Landroid/app/Application;") {
                    receiver
                } else {
                    self.new_instance("Landroid/app/Application;")?
                };
                self.statics.insert("droidless:application".into(), vec![object]);
                result.push(object);
            }
            ("Landroid/content/SharedPreferences;", "edit()Landroid/content/SharedPreferences$Editor;") => {
                let editor = self.heap.instance("Landroid/content/SharedPreferences$Editor;")?;
                let object = self.heap.get_mut(editor)?;
                object.data = Data::Bundle(BTreeMap::new());
                object.fields.insert("owner".into(), vec![receiver]);
                result.push(editor);
            }
            ("Landroid/content/SharedPreferences;", "contains(Ljava/lang/String;)Z") => {
                let Data::Bundle(values) = &self.heap.get(receiver)?.data else { bail!("invalid preferences"); };
                result.push(Word::from(i32::from(values.contains_key(self.heap.text(arg(1)?)?))));
            }
            ("Landroid/content/SharedPreferences;", sig) if [
                "getString(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;",
                "getInt(Ljava/lang/String;I)I", "getLong(Ljava/lang/String;J)J",
                "getFloat(Ljava/lang/String;F)F", "getBoolean(Ljava/lang/String;Z)Z",
            ].contains(&sig) => {
                let key = self.heap.text(arg(1)?)?;
                let Data::Bundle(values) = &self.heap.get(receiver)?.data else { bail!("invalid preferences"); };
                if let Some((ty, words)) = values.get(key) {
                    if *ty != method.returns { return Err(fault("Ljava/lang/ClassCastException;", format!("preference {key:?} has type {ty}, requested {}", method.returns))); }
                    result = words.clone();
                } else { result = args[2..].to_vec(); }
            }
            ("Landroid/content/SharedPreferences$Editor;", sig) if [
                "putString(Ljava/lang/String;Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;",
                "putInt(Ljava/lang/String;I)Landroid/content/SharedPreferences$Editor;",
                "putLong(Ljava/lang/String;J)Landroid/content/SharedPreferences$Editor;",
                "putFloat(Ljava/lang/String;F)Landroid/content/SharedPreferences$Editor;",
                "putBoolean(Ljava/lang/String;Z)Landroid/content/SharedPreferences$Editor;",
            ].contains(&sig) => {
                let key = self.heap.text(arg(1)?)?.to_owned();
                if method.parameters[1] == "Ljava/lang/String;" && arg(2)? == Word::ZERO {
                    self.remove_preference(receiver, key)?;
                } else { self.bundle_put(receiver, key, method.parameters[1].clone(), args[2..].to_vec())?; }
                result.push(receiver);
            }
            ("Landroid/content/SharedPreferences$Editor;", "remove(Ljava/lang/String;)Landroid/content/SharedPreferences$Editor;") => {
                let key = self.heap.text(arg(1)?)?.to_owned();
                self.remove_preference(receiver, key)?;
                result.push(receiver);
            }
            ("Landroid/content/SharedPreferences$Editor;", "clear()Landroid/content/SharedPreferences$Editor;") => {
                self.heap.get_mut(receiver)?.fields.insert("clear".into(), vec![Word::from(1)]);
                result.push(receiver);
            }
            ("Landroid/content/SharedPreferences$Editor;", "commit()Z") | ("Landroid/content/SharedPreferences$Editor;", "apply()V") => {
                // ponytail: synchronous disk apply on the guest thread; queue I/O when a Looper exists.
                let saved = self.commit_preferences(receiver)?;
                if method.name == "commit" { result.push(Word::from(i32::from(saved))); }
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
    fn remove_preference(&mut self, editor: Word, key: String) -> Result<()> {
        let Data::Bundle(changes) = &mut self.heap.get_mut(editor)?.data else {
            bail!("invalid preference editor");
        };
        ensure!(
            changes.contains_key(&key) || changes.len() < 16_384,
            "preference editor entry limit reached"
        );
        changes.insert(key, ("remove".into(), vec![]));
        Ok(())
    }
}
