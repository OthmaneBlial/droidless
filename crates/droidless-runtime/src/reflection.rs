//! APK-local class lookup and reflective construction, without host class loading.
use crate::{
    heap::{Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::{Field, Method};

pub(crate) fn primitive_wrapper(class: &str) -> Option<&'static str> {
    Some(match class {
        "Ljava/lang/Void;" => "V",
        "Ljava/lang/Boolean;" => "Z",
        "Ljava/lang/Byte;" => "B",
        "Ljava/lang/Character;" => "C",
        "Ljava/lang/Short;" => "S",
        "Ljava/lang/Integer;" => "I",
        "Ljava/lang/Long;" => "J",
        "Ljava/lang/Float;" => "F",
        "Ljava/lang/Double;" => "D",
        _ => return None,
    })
}

fn binary_descriptor(name: &str) -> Option<String> {
    if name.is_empty() || name.contains(['/', '\0']) {
        return None;
    }
    if name.starts_with('[') {
        let element = name.trim_start_matches('[');
        if name.len() - element.len() > 255 {
            return None;
        }
        if element.len() == 1 && "BCDFIJSZ".contains(element) {
            return Some(name.to_owned());
        }
        let component = element.strip_prefix('L')?.strip_suffix(';')?;
        if component.starts_with('[') {
            return None;
        }
        binary_descriptor(component)?;
        return Some(name.replace('.', "/"));
    }
    if name.contains([';', '[']) || name.split('.').any(str::is_empty) {
        return None;
    }
    Some(crate::vm::descriptor(name))
}

fn class_name(descriptor: &str) -> String {
    match descriptor {
        "V" => "void".into(),
        "Z" => "boolean".into(),
        "B" => "byte".into(),
        "C" => "char".into(),
        "S" => "short".into(),
        "I" => "int".into(),
        "J" => "long".into(),
        "F" => "float".into(),
        "D" => "double".into(),
        name => name
            .strip_prefix('L')
            .and_then(|n| n.strip_suffix(';'))
            .unwrap_or(name)
            .replace('/', "."),
    }
}

impl Runtime {
    pub(crate) fn primitive_field(&self, field: &Field) -> Option<&'static str> {
        if field.name == "TYPE"
            && field.ty == "Ljava/lang/Class;"
            && self.class_location(&field.class).is_none()
        {
            primitive_wrapper(&field.class)
        } else {
            None
        }
    }
    fn reflected_class(&self, object: Word) -> Result<String> {
        let object = self.heap.get(object)?;
        ensure!(
            object.class == "Ljava/lang/Class;",
            "invalid Class receiver"
        );
        let name = *object
            .fields
            .get("name")
            .and_then(|v| v.first())
            .context("Class has no descriptor")?;
        Ok(self.heap.text(name)?.to_owned())
    }
    pub(crate) fn reflection_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let arg = |n| args.get(n).copied().context("reflection argument missing");
        let mut result = vec![];
        match (method.class.as_str(), method.signature().as_str()) {
            ("Ljava/lang/Class;", "forName(Ljava/lang/String;)Ljava/lang/Class;") => {
                let name = self.heap.text(arg(0)?)?.to_owned();
                let descriptor = binary_descriptor(&name);
                let exists = descriptor.as_ref().is_some_and(|class| {
                    let element = class.trim_start_matches('[');
                    (class.starts_with('[') && element.len() == 1)
                        || self.class_location(element).is_some()
                        || crate::framework::known_class(element)
                        || primitive_wrapper(element).is_some()
                });
                if !exists {
                    return Err(fault("Ljava/lang/ClassNotFoundException;", name));
                }
                let class = descriptor.context("missing class descriptor")?;
                // Array class loading does not initialize its component class.
                if !class.starts_with('[') {
                    self.initialize(&class)?;
                }
                result.push(self.class_object(&class)?);
            }
            ("Ljava/lang/Object;", "getClass()Ljava/lang/Class;") => {
                let class = self.heap.get(arg(0)?)?.class.clone();
                result.push(self.class_object(&class)?);
            }
            ("Ljava/lang/Class;", "getName()Ljava/lang/String;") => {
                let class = self.reflected_class(arg(0)?)?;
                result.push(self.heap.string(class_name(&class))?);
            }
            // Class overrides Object.toString; falling through would expose a wrong handle string.
            ("Ljava/lang/Class;", "toString()Ljava/lang/String;") => {
                bail!("unsupported Class method {}", method.key());
            }
            ("Ljava/lang/Class;", "newInstance()Ljava/lang/Object;") => {
                let class = self.reflected_class(arg(0)?)?;
                if !class.starts_with('L') {
                    return Err(fault(
                        "Ljava/lang/InstantiationException;",
                        class_name(&class),
                    ));
                }
                if self.class_location(&class).is_none() && primitive_wrapper(&class).is_some() {
                    return Err(fault(
                        if class == "Ljava/lang/Void;" {
                            "Ljava/lang/IllegalAccessException;"
                        } else {
                            "Ljava/lang/InstantiationException;"
                        },
                        class_name(&class),
                    ));
                }
                if let Some((d, c)) = self.class_location(&class) {
                    let def = &self.apk.dex[d].classes[c];
                    if def.access & 0x600 != 0 {
                        return Err(fault(
                            "Ljava/lang/InstantiationException;",
                            class_name(&class),
                        ));
                    }
                    let ctor = def
                        .methods
                        .iter()
                        .find(|m| self.apk.dex[d].methods[m.index].signature() == "<init>()V");
                    let Some(ctor) = ctor else {
                        return Err(fault(
                            "Ljava/lang/InstantiationException;",
                            "no nullary constructor",
                        ));
                    };
                    let caller = self.frames.last().map(|f| f.method.class.as_str());
                    let same_package = caller.is_some_and(|caller| {
                        caller.rsplit_once('/').map(|p| p.0) == class.rsplit_once('/').map(|p| p.0)
                    });
                    let class_accessible = def.access & 1 != 0 || same_package;
                    let ctor_accessible = if ctor.access & 1 != 0 {
                        true
                    } else if ctor.access & 2 != 0 {
                        caller == Some(class.as_str())
                    } else {
                        same_package
                    };
                    if !class_accessible || !ctor_accessible {
                        return Err(fault(
                            "Ljava/lang/IllegalAccessException;",
                            class_name(&class),
                        ));
                    }
                }
                let object = self.new_instance(&class)?;
                self.invoke(
                    Method {
                        class,
                        name: "<init>".into(),
                        parameters: vec![],
                        returns: "V".into(),
                    },
                    vec![object],
                    false,
                )?;
                result.push(object);
            }
            ("Ljava/lang/Class;", "getPackage()Ljava/lang/Package;") => {
                let descriptor = self.reflected_class(arg(0)?)?;
                let package = descriptor
                    .strip_prefix('L')
                    .and_then(|s| s.strip_suffix(';'))
                    .and_then(|s| s.rsplit_once('/'))
                    .map(|(p, _)| p.replace('/', "."));
                let object = if let Some(package) = package {
                    let key = format!("droidless:package:{package}");
                    if let Some(object) = self.statics.get(&key).and_then(|v| v.first()) {
                        *object
                    } else {
                        let object = self.heap.instance("Ljava/lang/Package;")?;
                        let name = self.heap.string(package)?;
                        self.heap
                            .get_mut(object)?
                            .fields
                            .insert("name".into(), vec![name]);
                        self.statics.insert(key, vec![object]);
                        object
                    }
                } else {
                    Word::ZERO
                };
                result.push(object);
            }
            ("Ljava/lang/Package;", "getName()Ljava/lang/String;") => result.push(
                *self
                    .heap
                    .get(arg(0)?)?
                    .fields
                    .get("name")
                    .and_then(|v| v.first())
                    .context("uninitialized Package")?,
            ),
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
}
