use crate::{heap::Word, ui::Node, vm::Runtime};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

const FRAME: &str = "Landroid/widget/FrameLayout;";
const DRAWABLE: &str = "Landroid/graphics/drawable/Drawable;";
const CALLBACK: &str = "Landroid/graphics/drawable/Drawable$Callback;";
const FOREGROUND: &str = "droidless:view:foreground";
const WEAK: &str = "droidless:drawable:callback-weak";
const RECT: &str = "Landroid/graphics/Rect;";
const GRAVITY: &str = "droidless:view:foreground-gravity";

fn method(class: &str, name: &str, parameters: &[&str], returns: &str) -> Method {
    Method {
        class: class.into(),
        name: name.into(),
        parameters: parameters.iter().map(|p| (*p).into()).collect(),
        returns: returns.into(),
    }
}

impl Runtime {
    pub(crate) fn refresh_foreground(&mut self, view: Word, state: bool) -> Result<()> {
        let drawable = self.window_word(view, FOREGROUND)?;
        if drawable == Word::ZERO {
            return Ok(());
        }
        let roots = self.native_roots.len();
        self.native_roots.extend([view, drawable]);
        let result = (|| -> Result<()> {
            if state
                && self.invoke(
                    method(DRAWABLE, "isStateful", &[], "Z"),
                    vec![drawable],
                    true,
                )?[0]
                    .truth()
            {
                let states = self.invoke(
                    method("Landroid/view/View;", "getDrawableState", &[], "[I"),
                    vec![view],
                    true,
                )?[0];
                self.native_roots.push(states);
                self.invoke(
                    method(DRAWABLE, "setState", &["[I"], "Z"),
                    vec![drawable, states],
                    true,
                )?;
            }
            // Foreground painting accepts the existing color/selector leaf profile; unknown paint is an error.
            let color = if self.invoke(
                method(DRAWABLE, "isVisible", &[], "Z"),
                vec![drawable],
                true,
            )?[0]
                .truth()
            {
                self.background_drawable_color(drawable, 1)?
            } else {
                None
            };
            let rect = self.heap.instance(RECT)?;
            self.native_roots.push(rect);
            let padded = self.invoke(
                method(DRAWABLE, "getPadding", &[RECT], "Z"),
                vec![drawable, rect],
                true,
            )?[0]
                .truth();
            let mut padding = [0.0; 4];
            if padded {
                for (index, name) in ["left", "top", "right", "bottom"].into_iter().enumerate() {
                    let value = self
                        .window_word(rect, &format!("{RECT}->{name}:I"))?
                        .int()?;
                    ensure!(
                        (0..=1_000_000).contains(&value),
                        "invalid foreground padding"
                    );
                    padding[index] = value as f32;
                }
            }
            let model = self.view_mut(view)?;
            model.foreground_overlay = color;
            model.foreground_padding = padding;
            Ok(())
        })();
        self.native_roots.truncate(roots);
        result
    }
    pub(crate) fn layout_foregrounds(&mut self, node: &Node) -> Result<()> {
        let view = Word::Ref(node.handle);
        let drawable = self.window_word(view, FOREGROUND)?;
        if drawable != Word::ZERO {
            self.invoke(
                method(DRAWABLE, "setBounds", &["I", "I", "I", "I"], "V"),
                vec![
                    drawable,
                    Word::ZERO,
                    Word::ZERO,
                    Word::from(node.rect.width as i32),
                    Word::from(node.rect.height as i32),
                ],
                true,
            )?;
        }
        for child in &node.children {
            self.layout_foregrounds(child)?;
        }
        Ok(())
    }
    pub(crate) fn foreground_native(
        &mut self,
        m: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = m.signature();
        let frame = m.class == FRAME
            && matches!(
                signature.as_str(),
                "setForeground(Landroid/graphics/drawable/Drawable;)V"
                    | "getForeground()Landroid/graphics/drawable/Drawable;"
                    | "setForegroundGravity(I)V"
                    | "getForegroundGravity()I"
            );
        let drawable = m.class == DRAWABLE
            && matches!(
                signature.as_str(),
                "setCallback(Landroid/graphics/drawable/Drawable$Callback;)V"
                    | "getCallback()Landroid/graphics/drawable/Drawable$Callback;"
                    | "getPadding(Landroid/graphics/Rect;)Z"
                    | "setLayoutDirection(I)V"
                    | "getLayoutDirection()I"
                    | "invalidateSelf()V"
            );
        let invalidate = m.class == "Landroid/view/View;"
            && signature == "invalidateDrawable(Landroid/graphics/drawable/Drawable;)V";
        if !frame && !drawable && !invalidate {
            return Ok(None);
        }
        self.require_main_thread()?;
        ensure!(self.sync_depth < 32, "foreground callback nesting limit");
        let arg = |index| {
            args.get(index)
                .copied()
                .context("foreground argument missing")
        };
        let receiver = arg(0)?;
        ensure!(
            self.is_a(&self.heap.get(receiver)?.class, &m.class),
            "invalid foreground receiver"
        );
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", m.key());
        }
        let roots = self.native_roots.len();
        self.native_roots.extend_from_slice(args);
        let result = (|| -> Result<Vec<Word>> {
            match signature.as_str() {
                "getForeground()Landroid/graphics/drawable/Drawable;" => {
                    Ok(vec![self.window_word(receiver, FOREGROUND)?])
                }
                "getForegroundGravity()I" => Ok(vec![
                    self.heap
                        .get(receiver)?
                        .fields
                        .get(GRAVITY)
                        .and_then(|v| v.first())
                        .copied()
                        .unwrap_or(Word::from(119)),
                ]),
                "setForegroundGravity(I)V" => {
                    let mut gravity = arg(1)?.int()?;
                    if gravity & 7 == 0 {
                        gravity |= 0x00800003;
                    }
                    if gravity & 0x70 == 0 {
                        gravity |= 0x30;
                    }
                    ensure!(
                        gravity == 119 || self.window_word(receiver, FOREGROUND)? == Word::ZERO,
                        "painting foreground gravity other than FILL unsupported"
                    );
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(GRAVITY.into(), vec![Word::from(gravity)]);
                    Ok(vec![])
                }
                "setForeground(Landroid/graphics/drawable/Drawable;)V" => {
                    let next = arg(1)?;
                    ensure!(
                        next == Word::ZERO || self.is_a(&self.heap.get(next)?.class, DRAWABLE),
                        "foreground requires Drawable"
                    );
                    let previous = self.window_word(receiver, FOREGROUND)?;
                    if previous == next {
                        return Ok(vec![]);
                    }
                    // Validate supported paint before detaching the previous drawable.
                    if next != Word::ZERO {
                        let gravity = self
                            .heap
                            .get(receiver)?
                            .fields
                            .get(GRAVITY)
                            .and_then(|v| v.first())
                            .copied()
                            .unwrap_or(Word::from(119))
                            .int()?;
                        ensure!(
                            gravity == 119,
                            "painting foreground gravity other than FILL unsupported"
                        );
                        self.background_drawable_color(next, 1)?;
                    }
                    self.native_roots.push(previous);
                    if previous != Word::ZERO {
                        self.invoke(
                            method(DRAWABLE, "setCallback", &[CALLBACK], "V"),
                            vec![previous, Word::ZERO],
                            true,
                        )?;
                    }
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(FOREGROUND.into(), vec![next]);
                    let model = self.view_mut(receiver)?;
                    model.foreground_overlay = None;
                    model.foreground_padding = [0.0; 4];
                    if next != Word::ZERO {
                        self.invoke(
                            method(DRAWABLE, "setCallback", &[CALLBACK], "V"),
                            vec![next, receiver],
                            true,
                        )?;
                        let direction = self.invoke(
                            method("Landroid/view/View;", "getLayoutDirection", &[], "I"),
                            vec![receiver],
                            true,
                        )?[0];
                        self.invoke(
                            method(DRAWABLE, "setLayoutDirection", &["I"], "V"),
                            vec![next, direction],
                            true,
                        )?;
                        self.refresh_foreground(receiver, true)?;
                    }
                    self.request_view_layout(receiver)?;
                    Ok(vec![])
                }
                "setCallback(Landroid/graphics/drawable/Drawable$Callback;)V" => {
                    let callback = arg(1)?;
                    ensure!(
                        callback == Word::ZERO
                            || self.is_a(&self.heap.get(callback)?.class, CALLBACK),
                        "invalid Drawable callback"
                    );
                    let weak = if callback == Word::ZERO {
                        Word::ZERO
                    } else {
                        let weak = self.heap.instance("Ljava/lang/ref/WeakReference;")?;
                        self.native_roots.push(weak);
                        self.invoke(
                            method(
                                "Ljava/lang/ref/WeakReference;",
                                "<init>",
                                &["Ljava/lang/Object;"],
                                "V",
                            ),
                            vec![weak, callback],
                            false,
                        )?;
                        weak
                    };
                    let fields = &mut self.heap.get_mut(receiver)?.fields;
                    fields.remove("droidless:drawable:callback");
                    fields.insert(WEAK.into(), vec![weak]);
                    Ok(vec![])
                }
                "getCallback()Landroid/graphics/drawable/Drawable$Callback;" => {
                    let weak = self.window_word(receiver, WEAK)?;
                    if weak == Word::ZERO {
                        Ok(vec![Word::ZERO])
                    } else {
                        self.invoke(
                            method(
                                "Ljava/lang/ref/Reference;",
                                "get",
                                &[],
                                "Ljava/lang/Object;",
                            ),
                            vec![weak],
                            true,
                        )
                    }
                }
                "getPadding(Landroid/graphics/Rect;)Z" => {
                    let rect = arg(1)?;
                    ensure!(
                        self.is_a(&self.heap.get(rect)?.class, RECT),
                        "Drawable padding requires Rect"
                    );
                    if self.is_a(
                        &self.heap.get(receiver)?.class,
                        "Landroid/graphics/drawable/StateListDrawable;",
                    ) {
                        let items = self
                            .heap
                            .get(receiver)?
                            .fields
                            .get("droidless:drawable:selector-items")
                            .cloned()
                            .unwrap_or_default();
                        let mut padding = [0; 4];
                        for item in items.chunks_exact(2) {
                            if self.invoke(
                                method(DRAWABLE, "getPadding", &[RECT], "Z"),
                                vec![item[1], rect],
                                true,
                            )?[0]
                                .truth()
                            {
                                for (edge, name) in
                                    ["left", "top", "right", "bottom"].into_iter().enumerate()
                                {
                                    let value = self
                                        .window_word(rect, &format!("{RECT}->{name}:I"))?
                                        .int()?;
                                    ensure!(
                                        (0..=1_000_000).contains(&value),
                                        "invalid selector padding"
                                    );
                                    padding[edge] = padding[edge].max(value);
                                }
                            }
                        }
                        for (name, value) in
                            ["left", "top", "right", "bottom"].into_iter().zip(padding)
                        {
                            self.heap
                                .get_mut(rect)?
                                .fields
                                .insert(format!("{RECT}->{name}:I"), vec![Word::from(value)]);
                        }
                        return Ok(vec![Word::from(i32::from(padding.iter().any(|v| *v != 0)))]);
                    }
                    for name in ["left", "top", "right", "bottom"] {
                        self.heap
                            .get_mut(rect)?
                            .fields
                            .insert(format!("{RECT}->{name}:I"), vec![Word::ZERO]);
                    }
                    Ok(vec![Word::ZERO])
                }
                "setLayoutDirection(I)V" => {
                    let direction = arg(1)?;
                    ensure!(
                        (0..=1).contains(&direction.int()?),
                        "invalid Drawable layout direction"
                    );
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert("droidless:drawable:direction".into(), vec![direction]);
                    Ok(vec![])
                }
                "getLayoutDirection()I" => Ok(vec![
                    self.window_word(receiver, "droidless:drawable:direction")?,
                ]),
                "invalidateSelf()V" => {
                    let callback = self.invoke(
                        method(DRAWABLE, "getCallback", &[], CALLBACK),
                        vec![receiver],
                        true,
                    )?[0];
                    self.native_roots.push(callback);
                    if callback != Word::ZERO {
                        self.invoke(
                            method(CALLBACK, "invalidateDrawable", &[DRAWABLE], "V"),
                            vec![callback, receiver],
                            true,
                        )?;
                    }
                    Ok(vec![])
                }
                "invalidateDrawable(Landroid/graphics/drawable/Drawable;)V" => {
                    let source = arg(1)?;
                    ensure!(
                        self.is_a(&self.heap.get(source)?.class, DRAWABLE),
                        "invalid Drawable invalidation"
                    );
                    if self.window_word(receiver, FOREGROUND)? == source {
                        self.refresh_foreground(receiver, false)?;
                        self.request_view_layout(receiver)?;
                    }
                    Ok(vec![])
                }
                _ => unreachable!(),
            }
        })();
        self.native_roots.truncate(roots);
        result.map(Some)
    }
}
