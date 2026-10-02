use crate::{heap::Word, vm::Runtime};
use anyhow::{Context, Result, ensure};
use droidless_formats::dex::Method;

const SELECTOR: &str = "Landroid/graphics/drawable/StateListDrawable;";
const DRAWABLE: &str = "Landroid/graphics/drawable/Drawable;";
const ITEMS: &str = "droidless:drawable:selector-items";
const CURRENT: &str = "droidless:drawable:selector-current";
const STATE: &str = "droidless:drawable:state";

pub(crate) fn state_matches(required: &[i32], states: Option<&[i32]>) -> bool {
    let Some(states) = states else {
        return required.is_empty() || required[0] == 0;
    };
    required
        .iter()
        .take_while(|state| **state != 0)
        .all(|state| {
            let wanted = if *state > 0 {
                *state
            } else {
                state.wrapping_neg()
            };
            let present = states
                .iter()
                .take_while(|value| **value != 0)
                .any(|value| *value == wanted);
            present == (*state > 0)
        })
}

impl Runtime {
    fn selector_contains(&self, child: Word, target: Word) -> Result<bool> {
        let mut work = vec![(child, 0)];
        let mut seen = std::collections::HashSet::new();
        let mut edges = 0;
        while let Some((child, depth)) = work.pop() {
            ensure!(depth < 16, "selector nesting limit");
            if child == target {
                return Ok(true);
            }
            if !seen.insert(child.reference()?) {
                continue;
            }
            if let Some(items) = self.heap.get(child)?.fields.get(ITEMS) {
                edges += items.len() / 2;
                ensure!(edges <= 8192, "selector graph limit");
                work.extend(items.chunks_exact(2).map(|item| (item[1], depth + 1)));
            }
        }
        Ok(false)
    }
    fn selector_select(&mut self, selector: Word, state: Word) -> Result<bool> {
        let states = if state == Word::ZERO {
            None
        } else {
            Some(self.color_int_array(state)?)
        };
        let items = self
            .heap
            .get(selector)?
            .fields
            .get(ITEMS)
            .cloned()
            .unwrap_or_default();
        let mut current = Word::ZERO;
        for item in items.chunks_exact(2) {
            let required = if item[0] == Word::ZERO {
                vec![]
            } else {
                self.color_int_array(item[0])?
            };
            if state_matches(&required, states.as_deref()) {
                current = item[1];
                break;
            }
        }
        let previous = self.window_word(selector, CURRENT)?;
        self.heap
            .get_mut(selector)?
            .fields
            .insert(CURRENT.into(), vec![current]);
        let changed = if current == Word::ZERO {
            false
        } else {
            self.invoke(
                Method {
                    class: DRAWABLE.into(),
                    name: "setState".into(),
                    parameters: vec!["[I".into()],
                    returns: "Z".into(),
                },
                vec![current, state],
                true,
            )?[0]
                .truth()
        };
        Ok(previous != current || changed)
    }
    pub(crate) fn background_drawable_color(
        &self,
        drawable: Word,
        depth: usize,
    ) -> Result<Option<u32>> {
        ensure!(depth < 16, "selector nesting limit");
        if drawable == Word::ZERO {
            return Ok(None);
        }
        let object = self.heap.get(drawable)?;
        if self.is_a(&object.class, SELECTOR) {
            return self.background_drawable_color(self.window_word(drawable, CURRENT)?, depth + 1);
        }
        if let Some(id) = object
            .fields
            .get("resourceId")
            .and_then(|values| values.first())
        {
            let color = self
                .apk
                .resources
                .resolve(id.int()? as u32)
                .ok()
                .and_then(|value| self.drawable_color(value, 0).ok().flatten());
            ensure!(
                depth == 0 || color.is_some(),
                "selector background resource rendering unsupported"
            );
            return Ok(color);
        }
        if self.is_a(&object.class, "Landroid/graphics/drawable/ColorDrawable;") {
            let color = object
                .fields
                .get("color")
                .and_then(|values| values.first())
                .map(|value| value.int().map(|color| color as u32))
                .transpose()?;
            ensure!(
                depth == 0 || color.is_some(),
                "uninitialized selector ColorDrawable"
            );
            return Ok(color);
        }
        ensure!(
            depth == 0,
            "selector background child rendering unsupported: {}",
            object.class
        );
        Ok(None)
    }
    pub(crate) fn selector_drawable_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if !((method.class == SELECTOR
            && matches!(
                method.signature().as_str(),
                "<init>()V"
                    | "addState([ILandroid/graphics/drawable/Drawable;)V"
                    | "isStateful()Z"
                    | "onStateChange([I)Z"
            ))
            || (matches!(
                method.class.as_str(),
                SELECTOR | "Landroid/graphics/drawable/DrawableContainer;" | DRAWABLE
            ) && matches!(
                method.signature().as_str(),
                "getCurrent()Landroid/graphics/drawable/Drawable;"
                    | "setState([I)Z"
                    | "getIntrinsicWidth()I"
                    | "getIntrinsicHeight()I"
            ) && args.first().is_some_and(|receiver| {
                self.heap
                    .get(*receiver)
                    .is_ok_and(|object| self.is_a(&object.class, SELECTOR))
            })))
        {
            return Ok(None);
        }
        self.main_thread()?;
        ensure!(self.sync_depth < 32, "selector callback nesting limit");
        let receiver = *args.first().context("missing selector receiver")?;
        ensure!(
            self.is_a(&self.heap.get(receiver)?.class, SELECTOR),
            "invalid selector receiver"
        );
        let roots = self.native_roots.len();
        self.native_roots.extend_from_slice(args);
        let result = (|| -> Result<Vec<Word>> {
            let arg = |index: usize| {
                args.get(index)
                    .copied()
                    .context("missing selector argument")
            };
            match method.signature().as_str() {
                "<init>()V" => {
                    let state = self.array("I".into(), 0)?;
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(STATE.into(), vec![state]);
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(ITEMS.into(), vec![]);
                }
                "addState([ILandroid/graphics/drawable/Drawable;)V" => {
                    let child = arg(2)?;
                    if child == Word::ZERO {
                        return Ok(vec![]);
                    }
                    let spec = arg(1)?;
                    if spec != Word::ZERO {
                        self.color_int_array(spec)?;
                    }
                    ensure!(
                        self.is_a(&self.heap.get(child)?.class, DRAWABLE),
                        "selector child requires Drawable"
                    );
                    ensure!(
                        !self.selector_contains(child, receiver)?,
                        "cyclic selector child"
                    );
                    let items = self
                        .heap
                        .get_mut(receiver)?
                        .fields
                        .entry(ITEMS.into())
                        .or_default();
                    ensure!(items.len() < 8192, "selector item limit");
                    items.extend([spec, child]);
                    let state = self.window_word(receiver, STATE)?;
                    self.invoke(
                        Method {
                            class: SELECTOR.into(),
                            name: "onStateChange".into(),
                            parameters: vec!["[I".into()],
                            returns: "Z".into(),
                        },
                        vec![receiver, state],
                        true,
                    )?;
                }
                "isStateful()Z" => return Ok(vec![Word::from(1)]),
                "getCurrent()Landroid/graphics/drawable/Drawable;" => {
                    return Ok(vec![self.window_word(receiver, CURRENT)?]);
                }
                "getIntrinsicWidth()I" | "getIntrinsicHeight()I" => {
                    let current = self.window_word(receiver, CURRENT)?;
                    if current == Word::ZERO {
                        return Ok(vec![Word::from(-1)]);
                    }
                    return self.invoke(
                        Method {
                            class: DRAWABLE.into(),
                            ..method.clone()
                        },
                        vec![current],
                        true,
                    );
                }
                "onStateChange([I)Z" => {
                    return Ok(vec![Word::from(i32::from(
                        self.selector_select(receiver, arg(1)?)?,
                    ))]);
                }
                "setState([I)Z" => {
                    let state = arg(1)?;
                    let next = self.color_int_array(state)?;
                    let previous = self.window_word(receiver, STATE)?;
                    if previous != Word::ZERO && next == self.color_int_array(previous)? {
                        return Ok(vec![Word::ZERO]);
                    }
                    self.heap
                        .get_mut(receiver)?
                        .fields
                        .insert(STATE.into(), vec![state]);
                    return self.invoke(
                        Method {
                            class: SELECTOR.into(),
                            name: "onStateChange".into(),
                            parameters: vec!["[I".into()],
                            returns: "Z".into(),
                        },
                        vec![receiver, state],
                        true,
                    );
                }
                _ => unreachable!(),
            }
            Ok(vec![])
        })();
        self.native_roots.truncate(roots);
        result.map(Some)
    }
}
