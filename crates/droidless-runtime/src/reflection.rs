//! APK-local class lookup and reflective construction, without host class loading.
use crate::{
    heap::{Data, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::{Annotation, EncodedValue, Field, Method};

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
    fn annotation_value(&mut self, value: &EncodedValue) -> Result<Word> {
        Ok(match value {
            EncodedValue::Bits(value) => Word::Bits(*value as u32),
            EncodedValue::String(value) => self.heap.string(value.clone())?,
            EncodedValue::Type(value) => self.class_object(value)?,
            EncodedValue::Null => Word::ZERO,
            EncodedValue::Field(_) | EncodedValue::Method(_) => {
                bail!("field/method references are not valid runtime annotation values")
            }
            EncodedValue::Enum { class, name } => {
                let class = self.class_object(class)?;
                let name = self.heap.string(name.clone())?;
                *self
                    .invoke(
                        Method {
                            class: "Ljava/lang/Enum;".into(),
                            name: "valueOf".into(),
                            parameters: vec![
                                "Ljava/lang/Class;".into(),
                                "Ljava/lang/String;".into(),
                            ],
                            returns: "Ljava/lang/Enum;".into(),
                        },
                        vec![class, name],
                        false,
                    )?
                    .first()
                    .context("enum valueOf returned no value")?
            }
            EncodedValue::Annotation(annotation) => self.reflected_annotation(annotation)?,
            EncodedValue::Array(values) => {
                let array = self.array("Ljava/lang/Object;".into(), values.len())?;
                for (index, value) in values.iter().enumerate() {
                    let word = self.annotation_value(value)?;
                    if let Data::Array { values, .. } = &mut self.heap.get_mut(array)?.data {
                        values[index] = vec![word];
                    }
                }
                array
            }
        })
    }

    fn reflected_annotation(&mut self, annotation: &Annotation) -> Result<Word> {
        let proxy_class = format!(
            "Ldroidless/runtime/annotation/{};",
            annotation
                .class
                .trim_start_matches('L')
                .trim_end_matches(';')
        );
        let object = self.heap.instance(&proxy_class)?;
        let class = self.heap.string(annotation.class.clone())?;
        self.heap
            .get_mut(object)?
            .fields
            .insert("droidless:annotation:class".into(), vec![class]);
        for (name, value) in &annotation.values {
            let value = self.annotation_value(value)?;
            self.heap
                .get_mut(object)?
                .fields
                .insert(format!("droidless:annotation:value:{name}"), vec![value]);
        }
        Ok(object)
    }

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
    fn reflected_parameters(&self, array: Word) -> Result<Vec<String>> {
        if array == Word::ZERO {
            return Ok(vec![]);
        }
        let Data::Array { values, .. } = &self.heap.get(array)?.data else {
            bail!("reflection parameter list must be Class[]");
        };
        values
            .iter()
            .map(|value| self.reflected_class(*value.first().context("empty Class[] element")?))
            .collect()
    }
    fn reflected_method(
        &self,
        class: &str,
        name: &str,
        parameters: &[String],
        inherited: bool,
    ) -> Option<Method> {
        let mut class = class.to_owned();
        for _ in 0..64 {
            if let Some((dex, index)) = self.class_location(&class) {
                let definition = &self.apk.dex[dex].classes[index];
                if let Some(method) = definition
                    .methods
                    .iter()
                    .map(|encoded| &self.apk.dex[dex].methods[encoded.index])
                    .find(|method| method.name == name && method.parameters == parameters)
                {
                    return Some(method.clone());
                }
                if !inherited {
                    return None;
                }
                let Some(parent) = &definition.super_class else {
                    return None;
                };
                class = parent.clone();
            } else {
                if class == "Landroid/view/View;" {
                    let platform_method = (name == "computeFitSystemWindows"
                        && parameters == ["Landroid/graphics/Rect;", "Landroid/graphics/Rect;"])
                        || (name == "makeOptionalFitsSystemWindows" && parameters.is_empty());
                    if platform_method {
                        return Some(Method {
                            class,
                            name: name.into(),
                            parameters: parameters.to_vec(),
                            returns: "V".into(),
                        });
                    }
                }
                return None;
            }
        }
        None
    }
    fn reflected_method_object(
        &mut self,
        method: Method,
        access: u32,
        annotations: &[Annotation],
    ) -> Result<Word> {
        let object = self.heap.instance("Ljava/lang/reflect/Method;")?;
        self.heap.get_mut(object)?.data = Data::ReflectedMethod(method.clone());
        self.heap.get_mut(object)?.fields.insert(
            "droidless:reflect:modifiers".into(),
            vec![Word::from(access as i32)],
        );
        let parameters = self.array("Ljava/lang/Class;".into(), method.parameters.len())?;
        for (index, parameter) in method.parameters.iter().enumerate() {
            let class = self.class_object(parameter)?;
            if let Data::Array { values, .. } = &mut self.heap.get_mut(parameters)?.data {
                values[index] = vec![class];
            }
        }
        self.heap
            .get_mut(object)?
            .fields
            .insert("droidless:reflect:parameters".into(), vec![parameters]);
        let annotations = annotations
            .iter()
            .filter(|annotation| annotation.visibility == 1)
            .map(|annotation| self.reflected_annotation(annotation))
            .collect::<Result<Vec<_>>>()?;
        self.heap
            .get_mut(object)?
            .fields
            .insert("droidless:reflect:annotations".into(), annotations);
        Ok(object)
    }

    fn reflected_method_access(&self, method: &Method) -> Option<(u32, Vec<Annotation>)> {
        let (dex, class) = self.class_location(&method.class)?;
        let definition = &self.apk.dex[dex].classes[class];
        definition.methods.iter().find_map(|encoded| {
            (self.apk.dex[dex].methods[encoded.index].signature() == method.signature())
                .then(|| (encoded.access, encoded.annotations.clone()))
        })
    }

    fn annotation_proxy_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let receiver = *args.first().context("annotation receiver missing")?;
        let argument = |index| {
            args.get(index)
                .copied()
                .context("annotation argument missing")
        };
        let signature = method.signature();
        if signature == "annotationType()Ljava/lang/Class;" {
            let descriptor = format!(
                "L{}",
                method
                    .class
                    .trim_start_matches("Ldroidless/runtime/annotation/")
            );
            return Ok(Some(vec![self.class_object(&descriptor)?]));
        }
        if signature == "equals(Ljava/lang/Object;)Z" {
            return Ok(Some(vec![Word::from(i32::from(receiver == argument(1)?))]));
        }
        if signature == "hashCode()I" {
            return Ok(Some(vec![Word::from(receiver.reference()? as i32)]));
        }
        if signature == "toString()Ljava/lang/String;" {
            let descriptor = format!(
                "L{}",
                method
                    .class
                    .trim_start_matches("Ldroidless/runtime/annotation/")
            );
            return Ok(Some(vec![
                self.heap.string(format!("@{}", class_name(&descriptor)))?,
            ]));
        }
        let key = format!("droidless:annotation:value:{}", method.name);
        if let Some(value) = self
            .heap
            .get(receiver)?
            .fields
            .get(&key)
            .and_then(|values| values.first())
            .copied()
        {
            return Ok(Some(vec![value]));
        }
        if method.returns == "Lorg/greenrobot/eventbus/ThreadMode;" && method.name == "a" {
            let class = self.class_object("Lorg/greenrobot/eventbus/ThreadMode;")?;
            let name = self.heap.string("POSTING".into())?;
            let value = *self
                .invoke(
                    Method {
                        class: "Ljava/lang/Enum;".into(),
                        name: "valueOf".into(),
                        parameters: vec!["Ljava/lang/Class;".into(), "Ljava/lang/String;".into()],
                        returns: "Ljava/lang/Enum;".into(),
                    },
                    vec![class, name],
                    false,
                )?
                .first()
                .context("enum valueOf returned no value")?;
            self.heap.get_mut(receiver)?.fields.insert(key, vec![value]);
            return Ok(Some(vec![value]));
        }
        Ok(Some(if method.returns == "V" {
            vec![]
        } else {
            vec![Word::ZERO]
        }))
    }
    pub(crate) fn reflection_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class.starts_with("Ldroidless/runtime/annotation/") {
            return self.annotation_proxy_native(method, args);
        }
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
            ("Ljava/lang/Class;", "desiredAssertionStatus()Z") => result.push(Word::ZERO),
            ("Ljava/lang/Class;", "getSimpleName()Ljava/lang/String;") => {
                let class = self.reflected_class(arg(0)?)?;
                let simple = if class.starts_with('[') {
                    let rank = class
                        .chars()
                        .take_while(|character| *character == '[')
                        .count();
                    format!(
                        "{}{}",
                        class_name(&class[rank..]).rsplit('.').next().unwrap_or(""),
                        "[]".repeat(rank)
                    )
                } else {
                    class_name(&class)
                        .rsplit(['.', '$'])
                        .next()
                        .unwrap_or("")
                        .to_owned()
                };
                result.push(self.heap.string(simple)?);
            }
            ("Ljava/lang/Class;", "getSuperclass()Ljava/lang/Class;") => {
                let class = self.reflected_class(arg(0)?)?;
                let interface = self.class_location(&class).is_some_and(|(dex, index)| {
                    self.apk.dex[dex].classes[index].access & 0x200 != 0
                });
                result.push(if interface {
                    Word::ZERO
                } else if let Some(parent) = self.parent(&class) {
                    self.class_object(&parent)?
                } else {
                    Word::ZERO
                });
            }
            ("Ljava/lang/Class;", "getInterfaces()[Ljava/lang/Class;") => {
                let class = self.reflected_class(arg(0)?)?;
                let interfaces = self
                    .class_location(&class)
                    .map(|(dex, index)| self.apk.dex[dex].classes[index].interfaces.clone())
                    .unwrap_or_default();
                ensure!(
                    interfaces.len() <= 65_536,
                    "class interface reflection limit reached"
                );
                let array = self.array("Ljava/lang/Class;".into(), interfaces.len())?;
                for (index, interface) in interfaces.iter().enumerate() {
                    let interface = self.class_object(interface)?;
                    if let Data::Array { values, .. } = &mut self.heap.get_mut(array)?.data {
                        values[index] = vec![interface];
                    }
                }
                result.push(array);
            }
            ("Ljava/lang/Class;", "getDeclaredMethods()[Ljava/lang/reflect/Method;") => {
                let class = self.reflected_class(arg(0)?)?;
                let methods = self
                    .class_location(&class)
                    .map(|(dex, index)| {
                        self.apk.dex[dex].classes[index]
                            .methods
                            .iter()
                            .filter_map(|encoded| {
                                let method = self.apk.dex[dex].methods[encoded.index].clone();
                                (!method.name.starts_with('<')).then_some((
                                    method,
                                    encoded.access,
                                    encoded.annotations.clone(),
                                ))
                            })
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                ensure!(
                    methods.len() <= 65_536,
                    "declared method reflection limit reached"
                );
                let array = self.array("Ljava/lang/reflect/Method;".into(), methods.len())?;
                for (index, (method, access, annotations)) in methods.into_iter().enumerate() {
                    let reflected = self.reflected_method_object(method, access, &annotations)?;
                    if let Data::Array { values, .. } = &mut self.heap.get_mut(array)?.data {
                        values[index] = vec![reflected];
                    }
                }
                result.push(array);
            }
            ("Ljava/lang/Class;", "cast(Ljava/lang/Object;)Ljava/lang/Object;") => {
                let class = self.reflected_class(arg(0)?)?;
                let object = arg(1)?;
                if object != Word::ZERO && !self.is_a(&self.heap.get(object)?.class, &class) {
                    return Err(fault(
                        "Ljava/lang/ClassCastException;",
                        format!("cannot cast to {}", class_name(&class)),
                    ));
                }
                result.push(object);
            }
            ("Ljava/lang/Class;", "isAssignableFrom(Ljava/lang/Class;)Z") => {
                let target = self.reflected_class(arg(0)?)?;
                let source = self.reflected_class(arg(1)?)?;
                result.push(Word::from(i32::from(
                    source == target || self.is_a(&source, &target),
                )));
            }
            (
                "Ljava/lang/Class;",
                "getDeclaredMethod(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;",
            )
            | (
                "Ljava/lang/Class;",
                "getMethod(Ljava/lang/String;[Ljava/lang/Class;)Ljava/lang/reflect/Method;",
            ) => {
                let class = self.reflected_class(arg(0)?)?;
                let name = self.heap.text(arg(1)?)?.to_owned();
                let parameters = self.reflected_parameters(arg(2)?)?;
                let inherited = method.name == "getMethod";
                let reflected = self
                    .reflected_method(&class, &name, &parameters, inherited)
                    .ok_or_else(|| {
                        fault(
                            "Ljava/lang/NoSuchMethodException;",
                            format!(
                                "{}({})",
                                class_name(&class),
                                parameters
                                    .iter()
                                    .map(|parameter| class_name(parameter))
                                    .collect::<Vec<_>>()
                                    .join(",")
                            ),
                        )
                    })?;
                let (access, annotations) =
                    self.reflected_method_access(&reflected).unwrap_or_default();
                let object = self.reflected_method_object(reflected, access, &annotations)?;
                result.push(object);
            }
            ("Ljava/lang/reflect/Method;", "getName()Ljava/lang/String;") => {
                let Data::ReflectedMethod(reflected) = &self.heap.get(arg(0)?)?.data else {
                    bail!("uninitialized reflected Method")
                };
                result.push(self.heap.string(reflected.name.clone())?);
            }
            ("Ljava/lang/reflect/Method;", "getModifiers()I") => {
                let modifiers = self
                    .heap
                    .get(arg(0)?)?
                    .fields
                    .get("droidless:reflect:modifiers")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                result.push(modifiers);
            }
            ("Ljava/lang/reflect/Method;", "getParameterTypes()[Ljava/lang/Class;") => {
                let parameters = self
                    .heap
                    .get(arg(0)?)?
                    .fields
                    .get("droidless:reflect:parameters")
                    .and_then(|values| values.first())
                    .copied()
                    .context("reflected Method has no parameter list")?;
                result.push(parameters);
            }
            ("Ljava/lang/reflect/Method;", "getReturnType()Ljava/lang/Class;") => {
                let Data::ReflectedMethod(reflected) = &self.heap.get(arg(0)?)?.data else {
                    bail!("uninitialized reflected Method")
                };
                let returns = reflected.returns.clone();
                result.push(self.class_object(&returns)?);
            }
            (
                "Ljava/lang/reflect/Method;",
                "getAnnotation(Ljava/lang/Class;)Ljava/lang/annotation/Annotation;",
            ) => {
                let requested = self.reflected_class(arg(1)?)?;
                let annotations = self
                    .heap
                    .get(arg(0)?)?
                    .fields
                    .get("droidless:reflect:annotations")
                    .cloned()
                    .unwrap_or_default();
                let annotation = annotations
                    .into_iter()
                    .find(|annotation| {
                        self.heap
                            .get(*annotation)
                            .ok()
                            .and_then(|object| object.fields.get("droidless:annotation:class"))
                            .and_then(|values| values.first())
                            .and_then(|value| self.heap.text(*value).ok())
                            == Some(requested.as_str())
                    })
                    .unwrap_or(Word::ZERO);
                result.push(annotation);
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
            ("Ljava/lang/reflect/AccessibleObject;", "setAccessible(Z)V")
            | ("Ljava/lang/reflect/Method;", "setAccessible(Z)V") => {
                self.heap
                    .get_mut(arg(0)?)?
                    .fields
                    .insert("droidless:accessible".into(), vec![arg(1)?]);
            }
            ("Ljava/lang/reflect/AccessibleObject;", "isAccessible()Z")
            | ("Ljava/lang/reflect/Method;", "isAccessible()Z") => {
                let accessible = self
                    .heap
                    .get(arg(0)?)?
                    .fields
                    .get("droidless:accessible")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                result.push(accessible);
            }
            (
                "Ljava/lang/reflect/Method;",
                "invoke(Ljava/lang/Object;[Ljava/lang/Object;)Ljava/lang/Object;",
            ) => {
                let reflected = match &self.heap.get(arg(0)?)?.data {
                    Data::ReflectedMethod(method) => method.clone(),
                    _ => bail!("uninitialized reflected Method"),
                };
                let target = arg(1)?;
                let values = match arg(2)? {
                    Word::Bits(0) => vec![],
                    array => match &self.heap.get(array)?.data {
                        Data::Array { values, .. } => values
                            .iter()
                            .map(|value| {
                                value.first().copied().context("empty reflection argument")
                            })
                            .collect::<Result<Vec<_>>>()?,
                        _ => bail!("Method.invoke requires Object[]"),
                    },
                };
                let mut call_args = Vec::with_capacity(values.len() + 1);
                call_args.push(target);
                call_args.extend(values);
                let returned = self.invoke(reflected, call_args, true)?;
                result.push(returned.first().copied().unwrap_or(Word::ZERO));
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
}
