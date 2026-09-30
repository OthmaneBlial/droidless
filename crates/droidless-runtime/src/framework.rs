use crate::{
    heap::{Data, Word, bits64, exception_parent, fault, wide},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::{
    dex::Method,
    xml::{Element, Value},
};

pub(crate) fn known_class(class: &str) -> bool {
    crate::ui::View::for_class(class).is_some()
        || exception_parent(class).is_some()
        || [
            "Ljava/lang/Object;",
            "Ljava/lang/StringBuilder;",
            "Ljava/lang/String;",
            "Ljava/lang/Class;",
            "Ljava/lang/Double;",
            "Ljava/lang/Number;",
            "Ljava/lang/Thread;",
            "Ljava/lang/System;",
            "Landroid/os/Handler;",
            "Landroid/os/Message;",
            "Landroid/os/Looper;",
            "Landroid/os/SystemClock;",
            "Ljava/util/HashSet;",
            "Ljava/util/HashMap;",
            "Ljava/util/ArrayList;",
            "Ljava/util/LinkedHashMap;",
            "Ljava/util/concurrent/LinkedBlockingQueue;",
            "Ljava/lang/Throwable;",
            "Ljava/lang/Exception;",
            "Ljava/lang/RuntimeException;",
            "Landroid/app/Activity;",
            "Landroid/app/Application;",
            "Landroid/content/res/Resources;",
            "Landroid/util/DisplayMetrics;",
            "Landroid/view/WindowManager;",
            "Landroid/view/Display;",
            "Landroid/os/Bundle;",
            "Landroid/content/Intent;",
            "Landroid/view/KeyEvent;",
        ]
        .contains(&class)
}
impl Runtime {
    pub(crate) fn native(&mut self, method: &Method, args: &[Word]) -> Result<Option<Vec<Word>>> {
        if method.class.starts_with("Landroid/view/")
            || method.class.starts_with("Landroid/widget/")
            || method.class == "Landroid/app/Activity;"
        {
            self.require_main_thread()?;
        }
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        if let Some(result) = self.component_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.preference_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.collection_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.queue_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.reflection_native(method, args)? {
            return Ok(Some(result));
        }
        if let Some(result) = self.scheduling_native(method, args)? {
            return Ok(Some(result));
        }
        let signature = method.signature();
        let arg =
            |n| -> Result<Word> { args.get(n).copied().context("framework argument missing") };
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let mut result = vec![];
        match (method.class.as_str(), signature.as_str()) {
            ("Landroid/view/KeyEvent;", "getAction()I") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("action")
                    .context("uninitialized KeyEvent")?
                    .clone();
            }
            ("Landroid/view/KeyEvent;", "getKeyCode()I") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("keycode")
                    .context("uninitialized KeyEvent")?
                    .clone();
            }
            (class, "<init>()V") if exception_parent(class).is_some() => {
                self.heap.get(receiver)?;
            }
            (class, "<init>(Ljava/lang/String;)V") if exception_parent(class).is_some() => {
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("message".into(), vec![arg(1)?]);
            }
            ("Ljava/lang/Throwable;", "getMessage()Ljava/lang/String;") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("message")
                    .cloned()
                    .unwrap_or_else(|| vec![Word::ZERO]);
            }
            ("Ljava/lang/Throwable;", "getCause()Ljava/lang/Throwable;")
            | ("Ljava/lang/ExceptionInInitializerError;", "getException()Ljava/lang/Throwable;") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("cause")
                    .cloned()
                    .unwrap_or_else(|| vec![Word::ZERO]);
            }
            ("Ljava/lang/Throwable;", "toString()Ljava/lang/String;") => {
                let object = self.heap.get(receiver)?;
                let class = object
                    .class
                    .trim_start_matches('L')
                    .trim_end_matches(';')
                    .replace('/', ".");
                let message = object
                    .fields
                    .get("message")
                    .and_then(|v| v.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                let text = if message == Word::ZERO {
                    class
                } else {
                    format!("{class}: {}", self.heap.text(message)?)
                };
                result.push(self.heap.string(text)?);
            }
            ("Ljava/lang/String;", "valueOf(Ljava/lang/Object;)Ljava/lang/String;") => {
                if arg(0)? == Word::ZERO {
                    result.push(self.intern("null".into())?);
                } else {
                    result = self.invoke(
                        Method {
                            class: "Ljava/lang/Object;".into(),
                            name: "toString".into(),
                            parameters: vec![],
                            returns: "Ljava/lang/String;".into(),
                        },
                        vec![arg(0)?],
                        true,
                    )?;
                }
            }
            ("Ljava/lang/Object;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Ljava/lang/Object;", "toString()Ljava/lang/String;") => {
                let o = self.heap.get(receiver)?;
                let s = format!("{}@{:x}", o.class, receiver.reference()?);
                result.push(self.heap.string(s)?);
            }
            ("Ljava/lang/Object;", "equals(Ljava/lang/Object;)Z") => {
                result.push(Word::from(i32::from(receiver == arg(1)?)))
            }
            ("Ljava/lang/String;", "toString()Ljava/lang/String;") => {
                self.heap.text(receiver)?;
                result.push(receiver);
            }
            ("Ljava/lang/String;", "length()I") => result.push(Word::from(
                self.heap.text(receiver)?.encode_utf16().count() as i32,
            )),
            ("Ljava/lang/String;", "isEmpty()Z") => {
                result.push(Word::from(i32::from(self.heap.text(receiver)?.is_empty())))
            }
            ("Ljava/lang/String;", "equals(Ljava/lang/Object;)Z") => {
                let rhs = arg(1)?;
                result.push(Word::from(i32::from(
                    rhs != Word::ZERO
                        && self
                            .heap
                            .text(rhs)
                            .is_ok_and(|s| s == self.heap.text(receiver).unwrap_or("")),
                )));
            }
            ("Ljava/lang/String;", "startsWith(Ljava/lang/String;)Z") => {
                result.push(Word::from(i32::from(
                    self.heap
                        .text(receiver)?
                        .starts_with(self.heap.text(arg(1)?)?),
                )))
            }
            ("Ljava/lang/String;", "contains(Ljava/lang/CharSequence;)Z") => {
                result.push(Word::from(i32::from(
                    self.heap.text(receiver)?.contains(self.heap.text(arg(1)?)?),
                )))
            }
            ("Ljava/lang/String;", "substring(I)Ljava/lang/String;")
            | ("Ljava/lang/String;", "substring(II)Ljava/lang/String;") => {
                let units = self.heap.text(receiver)?.encode_utf16().collect::<Vec<_>>();
                let start = usize::try_from(arg(1)?.int()?).map_err(|_| {
                    fault(
                        "Ljava/lang/StringIndexOutOfBoundsException;",
                        "negative substring start",
                    )
                })?;
                let end = if args.len() == 3 {
                    usize::try_from(arg(2)?.int()?).map_err(|_| {
                        fault(
                            "Ljava/lang/StringIndexOutOfBoundsException;",
                            "negative substring end",
                        )
                    })?
                } else {
                    units.len()
                };
                let text = String::from_utf16(units.get(start..end).ok_or_else(|| {
                    fault(
                        "Ljava/lang/StringIndexOutOfBoundsException;",
                        "substring indexes out of bounds",
                    )
                })?)?;
                result.push(self.heap.string(text)?);
            }
            ("Ljava/lang/String;", "concat(Ljava/lang/String;)Ljava/lang/String;") => {
                let text = format!("{}{}", self.heap.text(receiver)?, self.heap.text(arg(1)?)?);
                result.push(self.heap.string(text)?);
            }
            ("Ljava/lang/String;", "charAt(I)C") => {
                let index = usize::try_from(arg(1)?.int()?).map_err(|_| {
                    fault(
                        "Ljava/lang/StringIndexOutOfBoundsException;",
                        "negative string index",
                    )
                })?;
                let c = self
                    .heap
                    .text(receiver)?
                    .encode_utf16()
                    .nth(index)
                    .ok_or_else(|| {
                        fault(
                            "Ljava/lang/StringIndexOutOfBoundsException;",
                            "charAt index out of bounds",
                        )
                    })?;
                result.push(Word::Bits(u32::from(c)));
            }
            ("Ljava/lang/String;", "valueOf(I)Ljava/lang/String;")
            | ("Ljava/lang/Integer;", "toString(I)Ljava/lang/String;") => {
                let s = arg(0)?.int()?.to_string();
                result.push(self.heap.string(s)?);
            }
            ("Ljava/lang/Double;", "toString(D)Ljava/lang/String;")
            | ("Ljava/lang/String;", "valueOf(D)Ljava/lang/String;") => result.push(
                self.heap
                    .string(java_double(f64::from_bits(bits64(args)?)))?,
            ),
            ("Ljava/lang/Double;", "valueOf(D)Ljava/lang/Double;") => {
                let value = wide(bits64(args)?);
                let object = self.heap.instance("Ljava/lang/Double;")?;
                self.heap
                    .get_mut(object)?
                    .fields
                    .insert("value".into(), value);
                result.push(object);
            }
            ("Ljava/lang/Double;", "doubleValue()D") => {
                result = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("value")
                    .context("uninitialized Double")?
                    .clone();
            }
            ("Ljava/lang/Double;", "toString()Ljava/lang/String;") => {
                let words = self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("value")
                    .context("uninitialized Double")?;
                result.push(
                    self.heap
                        .string(java_double(f64::from_bits(bits64(words)?)))?,
                );
            }
            ("Ljava/lang/Double;", "isNaN(D)Z") => {
                result.push(Word::from(i32::from(
                    f64::from_bits(bits64(args)?).is_nan(),
                )));
            }
            ("Ljava/lang/Double;", "equals(Ljava/lang/Object;)Z")
            | ("Ljava/lang/Double;", "hashCode()I") => {
                bail!("unsupported Double method {}", method.key());
            }
            ("Ljava/lang/Long;", "toString(J)Ljava/lang/String;") => {
                result.push(self.heap.string((bits64(args)? as i64).to_string())?);
            }
            ("Ljava/lang/Double;", "parseDouble(Ljava/lang/String;)D") => {
                let value = self
                    .heap
                    .text(arg(0)?)?
                    .trim()
                    .parse::<f64>()
                    .map_err(|_| {
                        fault(
                            "Ljava/lang/NumberFormatException;",
                            "invalid floating-point string",
                        )
                    })?;
                result = wide(value.to_bits());
            }
            ("Ljava/lang/Integer;", "parseInt(Ljava/lang/String;)I") => {
                let input = arg(0)?;
                if input == Word::ZERO {
                    return Err(fault(
                        "Ljava/lang/NumberFormatException;",
                        "null integer string",
                    ));
                }
                let value = self.heap.text(input)?.parse::<i32>().map_err(|_| {
                    fault(
                        "Ljava/lang/NumberFormatException;",
                        "invalid integer string",
                    )
                })?;
                result.push(Word::from(value));
            }
            ("Ljava/lang/StringBuilder;", "<init>()V") => {
                self.heap.get_mut(receiver)?.data = Data::Builder(String::new())
            }
            ("Ljava/lang/StringBuilder;", "<init>(Ljava/lang/String;)V") => {
                let s = self.heap.text(arg(1)?)?.to_owned();
                self.heap.get_mut(receiver)?.data = Data::Builder(s);
            }
            (
                "Ljava/lang/StringBuilder;",
                "append(Ljava/lang/String;)Ljava/lang/StringBuilder;",
            )
            | (
                "Ljava/lang/StringBuilder;",
                "append(Ljava/lang/CharSequence;)Ljava/lang/StringBuilder;",
            )
            | ("Ljava/lang/StringBuilder;", "append(I)Ljava/lang/StringBuilder;")
            | ("Ljava/lang/StringBuilder;", "append(D)Ljava/lang/StringBuilder;")
            | ("Ljava/lang/StringBuilder;", "append(Z)Ljava/lang/StringBuilder;") => {
                let text = match method.parameters[0].as_str() {
                    "I" => arg(1)?.int()?.to_string(),
                    "Z" => (arg(1)?.int()? != 0).to_string(),
                    "D" => java_double(f64::from_bits(bits64(&args[1..])?)),
                    _ => {
                        if arg(1)? == Word::ZERO {
                            "null".into()
                        } else {
                            self.heap.text(arg(1)?)?.to_owned()
                        }
                    }
                };
                let Data::Builder(s) = &mut self.heap.get_mut(receiver)?.data else {
                    bail!("StringBuilder not initialized");
                };
                ensure!(
                    s.len() + text.len() <= 1_048_576,
                    "StringBuilder limit reached"
                );
                s.push_str(&text);
                result.push(receiver);
            }
            ("Ljava/lang/StringBuilder;", "toString()Ljava/lang/String;") => {
                let text = self.heap.text(receiver)?.to_owned();
                result.push(self.heap.string(text)?);
            }
            ("Ljava/lang/Math;", sig)
                if [
                    "sqrt(D)D", "cbrt(D)D", "sin(D)D", "cos(D)D", "tan(D)D", "log(D)D", "exp(D)D",
                    "abs(D)D", "pow(DD)D",
                ]
                .contains(&sig) =>
            {
                let x = f64::from_bits(bits64(args)?);
                let value = match method.name.as_str() {
                    "sqrt" => x.sqrt(),
                    "cbrt" => x.cbrt(),
                    "sin" => x.sin(),
                    "cos" => x.cos(),
                    "tan" => x.tan(),
                    "log" => x.ln(),
                    "exp" => x.exp(),
                    "abs" => x.abs(),
                    _ => x.powf(f64::from_bits(bits64(&args[2..])?)),
                };
                result = wide(value.to_bits());
            }
            ("Landroid/app/Activity;", "<init>()V")
            | ("Landroid/app/Application;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/app/Activity;", "onCreate(Landroid/os/Bundle;)V")
            | ("Landroid/app/Activity;", "onStart()V")
            | ("Landroid/app/Activity;", "onRestart()V")
            | ("Landroid/app/Activity;", "onResume()V")
            | ("Landroid/app/Activity;", "onPause()V")
            | ("Landroid/app/Activity;", "onStop()V")
            | ("Landroid/app/Activity;", "onDestroy()V")
            | ("Landroid/app/Application;", "onCreate()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/app/Activity;", "setContentView(Landroid/view/View;)V") => {
                let view = arg(1)?;
                ensure!(
                    self.heap.get(view)?.view.is_some(),
                    "setContentView expects View"
                );
                self.set_content(receiver, view)?;
            }
            ("Landroid/app/Activity;", "setContentView(I)V") => {
                let root = self.inflate_id(arg(1)?.int()? as u32, 0)?;
                self.set_content(receiver, root)?;
            }
            ("Landroid/app/Activity;", "findViewById(I)Landroid/view/View;")
            | ("Landroid/view/View;", "findViewById(I)Landroid/view/View;") => {
                let root = if method.class == "Landroid/app/Activity;" {
                    self.screen(receiver)?
                        .root
                        .context("findViewById before setContentView")?
                } else {
                    receiver
                };
                result.push(
                    self.find_view(root, arg(1)?.int()? as u32, 0)?
                        .unwrap_or(Word::ZERO),
                );
            }
            ("Landroid/app/Activity;", "setTitle(Ljava/lang/CharSequence;)V") => {
                self.set_title(receiver, self.heap.text(arg(1)?)?.to_owned())?;
            }
            ("Landroid/content/Context;", "getString(I)Ljava/lang/String;")
            | ("Landroid/content/res/Resources;", "getString(I)Ljava/lang/String;") => {
                let text = self.resource_text(arg(1)?.int()? as u32)?;
                result.push(self.heap.string(text)?);
            }
            ("Landroid/content/Context;", "getResources()Landroid/content/res/Resources;") => {
                result.push(self.heap.instance("Landroid/content/res/Resources;")?)
            }
            ("Landroid/app/Activity;", "getWindowManager()Landroid/view/WindowManager;") => {
                result.push(self.heap.instance("Landroid/view/WindowManager;")?)
            }
            ("Landroid/view/WindowManager;", "getDefaultDisplay()Landroid/view/Display;") => {
                result.push(self.heap.instance("Landroid/view/Display;")?)
            }
            ("Landroid/util/DisplayMetrics;", "<init>()V") => {
                self.heap.get(receiver)?;
            }
            ("Landroid/view/Display;", "getMetrics(Landroid/util/DisplayMetrics;)V") => {
                let (width, height) = (self.width as i32, self.height as i32);
                let fields = &mut self.heap.get_mut(arg(1)?)?.fields;
                fields.insert(
                    "Landroid/util/DisplayMetrics;->widthPixels:I".into(),
                    vec![Word::from(width)],
                );
                fields.insert(
                    "Landroid/util/DisplayMetrics;->heightPixels:I".into(),
                    vec![Word::from(height)],
                );
                fields.insert(
                    "Landroid/util/DisplayMetrics;->density:F".into(),
                    vec![Word::Bits(1.0f32.to_bits())],
                );
            }
            ("Landroid/widget/LinearLayout;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/FrameLayout;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/TextView;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/Button;", "<init>(Landroid/content/Context;)V")
            | ("Landroid/widget/EditText;", "<init>(Landroid/content/Context;)V") => {
                self.view_mut(receiver)?;
                self.heap.get(arg(1)?)?;
            }
            ("Landroid/widget/LinearLayout;", "setOrientation(I)V") => {
                self.view_mut(receiver)?.orientation = arg(1)?.int()?
            }
            ("Landroid/view/ViewGroup;", "addView(Landroid/view/View;)V") => {
                let child = arg(1)?;
                self.view_mut(child)?;
                self.view_mut(receiver)?.children.push(child);
            }
            ("Landroid/widget/TextView;", "setText(Ljava/lang/CharSequence;)V") => {
                let text = if arg(1)? == Word::ZERO {
                    String::new()
                } else {
                    self.heap.text(arg(1)?)?.to_owned()
                };
                self.view_mut(receiver)?.text = text;
            }
            ("Landroid/widget/TextView;", "setText(I)V") => {
                let text = self.resource_text(arg(1)?.int()? as u32)?;
                self.view_mut(receiver)?.text = text;
            }
            ("Landroid/widget/TextView;", "append(Ljava/lang/CharSequence;)V") => {
                let text = self.heap.text(arg(1)?)?.to_owned();
                let view = self.view_mut(receiver)?;
                ensure!(
                    view.text.len() + text.len() <= 1_048_576,
                    "TextView text limit reached"
                );
                view.text.push_str(&text);
            }
            ("Landroid/widget/TextView;", "getText()Ljava/lang/CharSequence;")
            | ("Landroid/widget/EditText;", "getText()Landroid/text/Editable;") => {
                let text = self.view_mut(receiver)?.text.clone();
                result.push(self.heap.string(text)?);
            }
            ("Landroid/widget/TextView;", "setTextSize(F)V") => {
                let size = f32::from_bits(arg(1)?.int()? as u32);
                ensure!(size.is_finite() && size >= 0.0, "invalid text size");
                self.view_mut(receiver)?.text_size = size;
            }
            ("Landroid/widget/TextView;", "setTextColor(I)V") => {
                self.view_mut(receiver)?.text_color = arg(1)?.int()? as u32
            }
            ("Landroid/widget/TextView;", "setGravity(I)V") => {
                self.view_mut(receiver)?.gravity = arg(1)?.int()? as u32
            }
            ("Landroid/widget/TextView;", "setKeyListener(Landroid/text/method/KeyListener;)V") => {
                ensure!(arg(1)? == Word::ZERO, "non-null KeyListener unsupported");
                self.view_mut(receiver)?.editable = false;
            }
            ("Landroid/widget/EditText;", "setSelection(I)V") => {
                let n = arg(1)?.int()?;
                ensure!(
                    n >= 0 && n as usize <= self.view_mut(receiver)?.text.encode_utf16().count(),
                    "selection index out of range"
                );
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("selection".into(), vec![Word::from(n)]);
            }
            ("Landroid/view/View;", "setOnClickListener(Landroid/view/View$OnClickListener;)V") => {
                let listener = arg(1)?;
                self.view_mut(receiver)?.listener = if listener == Word::ZERO {
                    None
                } else {
                    self.heap.get(listener)?;
                    Some(listener)
                };
            }
            ("Landroid/view/View;", "setOnKeyListener(Landroid/view/View$OnKeyListener;)V") => {
                let listener = arg(1)?;
                self.view_mut(receiver)?.key_listener = if listener == Word::ZERO {
                    None
                } else {
                    self.heap.get(listener)?;
                    Some(listener)
                };
            }
            ("Landroid/view/View;", "setId(I)V") => {
                self.view_mut(receiver)?.id = arg(1)?.int()? as u32
            }
            ("Landroid/view/View;", "getId()I") => {
                result.push(Word::Bits(self.view_mut(receiver)?.id))
            }
            ("Landroid/view/View;", "setVisibility(I)V") => {
                let v = arg(1)?.int()?;
                ensure!([0, 4, 8].contains(&v), "invalid View visibility");
                self.view_mut(receiver)?.visible = v;
            }
            ("Landroid/view/View;", "setEnabled(Z)V") => {
                self.view_mut(receiver)?.enabled = arg(1)?.int()? != 0
            }
            ("Landroid/view/View;", "setBackgroundColor(I)V") => {
                self.view_mut(receiver)?.background = Some(arg(1)?.int()? as u32)
            }
            ("Landroid/view/View;", "setPadding(IIII)V") => {
                let values = args[1..]
                    .iter()
                    .map(|w| w.int())
                    .collect::<Result<Vec<_>>>()?;
                ensure!(
                    values.len() == 4
                        && values.iter().all(|n| *n >= 0)
                        && values.iter().all(|n| *n == values[0]),
                    "only uniform non-negative padding supported"
                );
                self.view_mut(receiver)?.padding = values[0] as f32;
            }
            ("Landroid/util/Log;", sig)
                if [
                    "d(Ljava/lang/String;Ljava/lang/String;)I",
                    "i(Ljava/lang/String;Ljava/lang/String;)I",
                    "w(Ljava/lang/String;Ljava/lang/String;)I",
                    "e(Ljava/lang/String;Ljava/lang/String;)I",
                ]
                .contains(&sig) =>
            {
                eprintln!(
                    "{}/{}: {}",
                    method.name.to_uppercase(),
                    self.heap.text(arg(0)?)?,
                    self.heap.text(arg(1)?)?
                );
                result.push(Word::ZERO);
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
    fn view_mut(&mut self, word: Word) -> Result<&mut crate::ui::View> {
        self.heap
            .get_mut(word)?
            .view
            .as_mut()
            .context("framework method expects a View")
    }
    pub(crate) fn resource_text(&self, id: u32) -> Result<String> {
        // Android's stable public framework string IDs, not application-specific output.
        match id {
            0x0104000a => Ok("OK".into()),
            0x01040000 => Ok("Cancel".into()),
            _ => self.apk.resources.text(id),
        }
    }
    fn find_view(&self, word: Word, id: u32, depth: usize) -> Result<Option<Word>> {
        ensure!(depth < 128, "View search nesting limit");
        let view = self
            .heap
            .get(word)?
            .view
            .as_ref()
            .context("expected View")?;
        if view.id == id {
            return Ok(Some(word));
        }
        for child in &view.children {
            if let Some(v) = self.find_view(*child, id, depth + 1)? {
                return Ok(Some(v));
            }
        }
        Ok(None)
    }
    fn inflate_id(&mut self, id: u32, depth: usize) -> Result<Word> {
        ensure!(depth < 64, "layout inflation nesting limit");
        let name = self.apk.resources.text(id)?;
        let data = self
            .apk
            .files
            .get(&name)
            .with_context(|| format!("layout file {name} missing"))?;
        let element = droidless_formats::xml::parse(data)?;
        self.inflate(&element, depth + 1)
    }
    fn attribute(&self, value: &Value) -> Result<Value> {
        if value.kind == 1 {
            Ok(self.apk.resources.resolve(value.data)?.clone())
        } else {
            Ok(value.clone())
        }
    }
    fn inflate(&mut self, element: &Element, depth: usize) -> Result<Word> {
        ensure!(depth < 64, "layout XML nesting limit");
        if element.name == "include" {
            return self.inflate_id(
                element.number("layout").context("include missing layout")?,
                depth + 1,
            );
        }
        let class = if element.name.contains('.') {
            crate::vm::descriptor(&element.name)
        } else {
            format!("Landroid/widget/{};", element.name)
        };
        let word = self.new_instance(&class)?;
        let mut view = self
            .heap
            .get(word)?
            .view
            .clone()
            .with_context(|| format!("unsupported layout View {}", element.name))?;
        view.id = element.number("id").unwrap_or(0);
        for (name, raw) in &element.attributes {
            match name.as_str() {
                "text" => {
                    view.text = if raw.kind == 1 {
                        self.resource_text(raw.data)?
                    } else {
                        raw.display()
                    }
                }
                "layout_width" => view.width = dimension(&self.attribute(raw)?)?,
                "layout_height" => view.height = dimension(&self.attribute(raw)?)?,
                "layout_weight" => {
                    view.weight = if raw.kind == 4 {
                        f32::from_bits(raw.data)
                    } else {
                        raw.data as f32
                    }
                }
                "orientation" => view.orientation = raw.data as i32,
                "textSize" => view.text_size = dimension(&self.attribute(raw)?)?,
                "textColor" => {
                    let v = self.attribute(raw)?;
                    if (0x1c..=0x1f).contains(&v.kind) {
                        view.text_color = v.data;
                    } else {
                        bail!("unsupported text color state list");
                    }
                }
                "background" => view.background = self.drawable_color(raw, 0)?,
                "padding" => view.padding = dimension(&self.attribute(raw)?)?,
                "layout_margin" => view.margins = [dimension(&self.attribute(raw)?)?; 4],
                "layout_marginLeft"
                | "layout_marginTop"
                | "layout_marginRight"
                | "layout_marginBottom" => {
                    let i = match name.as_str() {
                        "layout_marginLeft" => 0,
                        "layout_marginTop" => 1,
                        "layout_marginRight" => 2,
                        _ => 3,
                    };
                    view.margins[i] = dimension(&self.attribute(raw)?)?;
                }
                "gravity" => view.gravity = raw.data,
                "onClick" => view.xml_click = Some(raw.display()),
                "visibility" => view.visible = raw.data as i32,
                "enabled" => view.enabled = raw.data != 0,
                "inputType" if raw.data == 0 => {
                    view.editable = false;
                }
                _ => {} // Styling outside this subset is documented; execution APIs still fail explicitly.
            }
        }
        ensure!(
            view.weight.is_finite()
                && view.weight >= 0.0
                && view.text_size.is_finite()
                && view.text_size >= 0.0,
            "invalid layout numeric attribute"
        );
        for child in &element.children {
            view.children.push(self.inflate(child, depth + 1)?);
        }
        self.heap.get_mut(word)?.view = Some(view);
        Ok(word)
    }
    fn drawable_color(&self, raw: &Value, depth: usize) -> Result<Option<u32>> {
        ensure!(depth < 32, "drawable reference nesting limit");
        let value = self.attribute(raw)?;
        if (0x1c..=0x1f).contains(&value.kind) {
            return Ok(Some(value.data));
        }
        if value.kind == 3 {
            let name = value.display();
            if name.ends_with(".xml") {
                let data = self.apk.files.get(&name).context("drawable XML missing")?;
                let element = droidless_formats::xml::parse(data)?;
                fn color(node: &Element) -> Option<&Value> {
                    if node.name == "solid" {
                        node.attr("color")
                    } else if node.name == "item"
                        && !node.attributes.contains_key("state_pressed")
                        && !node.attributes.contains_key("state_focused")
                    {
                        node.attr("drawable")
                            .or_else(|| node.children.iter().find_map(color))
                    } else {
                        node.children.iter().find_map(color)
                    }
                }
                if let Some(c) = color(&element) {
                    return self.drawable_color(c, depth + 1);
                }
            }
        }
        Ok(None)
    }
}
fn dimension(v: &Value) -> Result<f32> {
    let n = match v.kind {
        0x10 | 0x11 => v.data as i32 as f32,
        4 => f32::from_bits(v.data),
        5 => {
            let radix = (v.data >> 4) & 3;
            let unit = v.data & 15;
            ensure!(unit <= 2, "only px/dp/sp dimensions supported");
            (v.data & 0xffffff00) as i32 as f32
                * [
                    1.0 / 256.0,
                    1.0 / 32768.0,
                    1.0 / 8388608.0,
                    1.0 / 2147483648.0,
                ][radix as usize]
        }
        _ => bail!("unsupported layout dimension type {}", v.kind),
    };
    ensure!(n.is_finite() && n >= -2.0, "invalid layout dimension");
    Ok(n)
}
fn java_double(v: f64) -> String {
    if v.is_nan() {
        "NaN".into()
    } else if v == f64::INFINITY {
        "Infinity".into()
    } else if v == f64::NEG_INFINITY {
        "-Infinity".into()
    } else {
        let s = v.to_string();
        if s.contains('.') || s.contains('e') {
            s
        } else {
            format!("{s}.0")
        }
    }
}
