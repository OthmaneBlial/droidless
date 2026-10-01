use crate::{
    heap::{Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;

const FOCUSED: &str = "droidless:view:focused";
const CHILD: &str = "droidless:view:focused-child";
const PARENT: &str = "droidless:view:parent";
const MODE: &str = "droidless:descendant-focusability";
const BEFORE: i32 = 0x20000;
const AFTER: i32 = 0x40000;
const BLOCK: i32 = 0x60000;

impl Runtime {
    pub(crate) fn focus_field(&self, view: Word, key: &str) -> Result<Word> {
        let object = self.heap.get(view)?;
        ensure!(object.view.is_some(), "focus requires a View");
        if let Some(value) = object.fields.get(key).and_then(|values| values.first()) {
            return Ok(*value);
        }
        let default = match key {
            MODE => BEFORE,
            "droidless:setFocusable" => i32::from(
                [
                    "Landroid/widget/Button;",
                    "Landroid/widget/ImageButton;",
                    "Landroid/widget/EditText;",
                ]
                .iter()
                .any(|class| self.is_a(&object.class, class)),
            ),
            "droidless:setFocusableInTouchMode" => {
                i32::from(self.is_a(&object.class, "Landroid/widget/EditText;"))
            }
            _ => 0,
        };
        Ok(Word::from(default))
    }
    fn focus_set(&mut self, view: Word, key: &str, value: Word) -> Result<()> {
        self.heap
            .get_mut(view)?
            .fields
            .insert(key.into(), vec![value]);
        Ok(())
    }
    pub(crate) fn focus_root(&self, mut view: Word) -> Result<Word> {
        for _ in 0..128 {
            let parent = self.focus_field(view, PARENT)?;
            if parent == Word::ZERO {
                return Ok(view);
            }
            view = parent;
        }
        bail!("cyclic or too deep View focus parent hierarchy")
    }
    pub(crate) fn find_focus(&self, mut view: Word) -> Result<Word> {
        for _ in 0..128 {
            if self.focus_field(view, FOCUSED)?.truth() {
                return Ok(view);
            }
            view = self.focus_field(view, CHILD)?;
            if view == Word::ZERO {
                return Ok(Word::ZERO);
            }
        }
        bail!("cyclic or too deep View focus child hierarchy")
    }
    fn focus_changed(&mut self, view: Word, gain: bool, direction: i32, rect: Word) -> Result<()> {
        self.invoke(
            Method {
                class: "Landroid/view/View;".into(),
                name: "onFocusChanged".into(),
                parameters: vec!["Z".into(), "I".into(), "Landroid/graphics/Rect;".into()],
                returns: "V".into(),
            },
            vec![
                view,
                Word::from(i32::from(gain)),
                Word::from(direction),
                rect,
            ],
            true,
        )?;
        self.invoke(
            Method {
                class: "Landroid/view/View;".into(),
                name: "refreshDrawableState".into(),
                parameters: vec![],
                returns: "V".into(),
            },
            vec![view],
            true,
        )?;
        Ok(())
    }
    fn unfocus_view(&mut self, view: Word, depth: usize) -> Result<()> {
        ensure!(depth < 128, "cyclic or too deep View focus child hierarchy");
        let child = self.focus_field(view, CHILD)?;
        let roots = self.native_roots.len();
        self.native_roots.extend([view, child]);
        let result = (|| {
            self.focus_set(view, CHILD, Word::ZERO)?;
            if child != Word::ZERO {
                self.unfocus_view(child, depth + 1)?;
            }
            if self.focus_field(view, FOCUSED)?.truth() {
                self.focus_set(view, FOCUSED, Word::ZERO)?;
                self.focus_changed(view, false, 0, Word::ZERO)?;
            }
            Ok(())
        })();
        self.native_roots.truncate(roots);
        result
    }
    fn request_view_focus(&mut self, view: Word, direction: i32, rect: Word) -> Result<bool> {
        self.focus_root(view)?;
        // ponytail: explicit focus in the desktop non-touch profile. Add touch-mode
        // transitions with the host input lifecycle, not a fabricated isInTouchMode result.
        if !self.focus_field(view, "droidless:setFocusable")?.truth()
            || self
                .heap
                .get(view)?
                .view
                .as_ref()
                .context("focus requires a View")?
                .visible
                != 0
        {
            return Ok(false);
        }
        let mut parent = self.focus_field(view, PARENT)?;
        while parent != Word::ZERO {
            if self.focus_field(parent, MODE)?.int()? == BLOCK {
                return Ok(false);
            }
            parent = self.focus_field(parent, PARENT)?;
        }
        if !self.focus_field(view, FOCUSED)?.truth() {
            let child = self.focus_field(view, CHILD)?;
            if child != Word::ZERO {
                self.unfocus_view(child, 0)?;
                self.focus_set(view, CHILD, Word::ZERO)?;
            }
            self.focus_set(view, FOCUSED, Word::from(1))?;
            let parent = self.focus_field(view, PARENT)?;
            if parent != Word::ZERO {
                self.invoke(
                    Method {
                        class: "Landroid/view/ViewParent;".into(),
                        name: "requestChildFocus".into(),
                        parameters: vec!["Landroid/view/View;".into(); 2],
                        returns: "V".into(),
                    },
                    vec![parent, view, view],
                    true,
                )?;
            }
            self.focus_changed(view, true, direction, rect)?;
        }
        Ok(true)
    }
    fn request_descendant_focus(&mut self, view: Word, direction: i32, rect: Word) -> Result<bool> {
        let mut children = self
            .heap
            .get(view)?
            .view
            .as_ref()
            .context("focus requires ViewGroup")?
            .children
            .clone();
        if direction & 2 == 0 {
            children.reverse();
        }
        let roots = self.native_roots.len();
        self.native_roots.extend(children.iter().copied());
        let result = (|| {
            for child in children {
                if self.focus_field(child, PARENT)? != view
                    || self
                        .heap
                        .get(child)?
                        .view
                        .as_ref()
                        .context("focus requires a View")?
                        .visible
                        != 0
                {
                    continue;
                }
                let result = self.invoke(
                    Method {
                        class: "Landroid/view/View;".into(),
                        name: "requestFocus".into(),
                        parameters: vec!["I".into(), "Landroid/graphics/Rect;".into()],
                        returns: "Z".into(),
                    },
                    vec![child, Word::from(direction), rect],
                    true,
                )?;
                if result
                    .first()
                    .context("requestFocus returned no value")?
                    .truth()
                {
                    return Ok(true);
                }
            }
            Ok(false)
        })();
        self.native_roots.truncate(roots);
        result
    }
    pub(crate) fn focus_before_remove(&mut self, parent: Word, child: Word) -> Result<()> {
        if self.focus_field(parent, CHILD)? == child {
            self.unfocus_view(child, 0)?;
        }
        Ok(())
    }
    pub(crate) fn focus_hierarchy_change(
        &mut self,
        parent: Word,
        child: Word,
        added: bool,
    ) -> Result<()> {
        if added && self.find_focus(child)? != Word::ZERO {
            let focused = self.find_focus(child)?;
            self.invoke(
                Method {
                    class: "Landroid/view/ViewParent;".into(),
                    name: "requestChildFocus".into(),
                    parameters: vec!["Landroid/view/View;".into(); 2],
                    returns: "V".into(),
                },
                vec![parent, child, focused],
                true,
            )?;
        } else if !added && self.focus_field(parent, CHILD)? == child {
            self.focus_set(parent, CHILD, Word::ZERO)?;
            self.unfocus_view(child, 0)?;
            self.invoke(
                Method {
                    class: "Landroid/view/ViewParent;".into(),
                    name: "clearChildFocus".into(),
                    parameters: vec!["Landroid/view/View;".into()],
                    returns: "V".into(),
                },
                vec![parent, child],
                true,
            )?;
            let root = self.focus_root(parent)?;
            self.invoke(
                Method {
                    class: "Landroid/view/View;".into(),
                    name: "requestFocus".into(),
                    parameters: vec![],
                    returns: "Z".into(),
                },
                vec![root],
                true,
            )?;
        }
        Ok(())
    }
    pub(crate) fn focus_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if ![
            "Landroid/view/View;",
            "Landroid/view/ViewGroup;",
            "Landroid/view/ViewParent;",
        ]
        .contains(&method.class.as_str())
        {
            return Ok(None);
        }
        if !matches!(
            method.name.as_str(),
            "requestFocus"
                | "onRequestFocusInDescendants"
                | "requestChildFocus"
                | "clearChildFocus"
                | "clearFocus"
                | "onFocusChanged"
                | "isFocused"
                | "hasFocus"
                | "findFocus"
                | "getFocusedChild"
                | "getRootView"
                | "isFocusable"
                | "isFocusableInTouchMode"
                | "getDescendantFocusability"
                | "setDescendantFocusability"
        ) {
            return Ok(None);
        }
        self.require_main_thread()?;
        if self.trace.framework {
            eprintln!("framework: {} {args:?}", method.key());
        }
        let arg = |index| args.get(index).copied().context("missing focus argument");
        let view = args.first().copied().unwrap_or(Word::ZERO);
        let group = method.class == "Landroid/view/ViewGroup;";
        let roots = self.native_roots.len();
        self.native_roots.extend_from_slice(args);
        let result = (|| -> Result<Option<Vec<Word>>> {
            let mut words = vec![];
            match method.signature().as_str() {
                "requestFocus()Z" | "requestFocus(I)Z" => {
                    let direction = if method.parameters.is_empty() {
                        Word::from(130)
                    } else {
                        arg(1)?
                    };
                    words = self.invoke(
                        Method {
                            class: "Landroid/view/View;".into(),
                            name: "requestFocus".into(),
                            parameters: vec!["I".into(), "Landroid/graphics/Rect;".into()],
                            returns: "Z".into(),
                        },
                        vec![view, direction, Word::ZERO],
                        true,
                    )?;
                }
                "requestFocus(ILandroid/graphics/Rect;)Z" => {
                    let direction = arg(1)?.int()?;
                    let rect = arg(2)?;
                    if rect != Word::ZERO {
                        ensure!(
                            self.is_a(&self.heap.get(rect)?.class, "Landroid/graphics/Rect;"),
                            "focus hint requires Rect"
                        );
                    }
                    let mode = self.focus_field(view, MODE)?.int()?;
                    let mut took = false;
                    if !group || mode != AFTER {
                        took = self.request_view_focus(view, direction, rect)?;
                    }
                    if group && !took && mode != BLOCK {
                        let result = self.invoke(
                            Method {
                                class: "Landroid/view/ViewGroup;".into(),
                                name: "onRequestFocusInDescendants".into(),
                                parameters: vec!["I".into(), "Landroid/graphics/Rect;".into()],
                                returns: "Z".into(),
                            },
                            args.to_vec(),
                            true,
                        )?;
                        took = result
                            .first()
                            .context("descendant focus callback returned no value")?
                            .truth();
                    }
                    if group && !took && mode == AFTER {
                        took = self.request_view_focus(view, direction, rect)?;
                    }
                    words.push(Word::from(i32::from(took)));
                }
                "onRequestFocusInDescendants(ILandroid/graphics/Rect;)Z" if group => {
                    words.push(Word::from(i32::from(self.request_descendant_focus(
                        view,
                        arg(1)?.int()?,
                        arg(2)?,
                    )?)));
                }
                "requestChildFocus(Landroid/view/View;Landroid/view/View;)V" if group => {
                    self.focus_root(view)?;
                    let child = arg(1)?;
                    let focused = arg(2)?;
                    self.focus_field(child, FOCUSED)?;
                    self.focus_field(focused, FOCUSED)?;
                    if self.focus_field(view, MODE)?.int()? != BLOCK {
                        if self.focus_field(view, FOCUSED)?.truth() {
                            self.focus_set(view, FOCUSED, Word::ZERO)?;
                            self.focus_changed(view, false, 0, Word::ZERO)?;
                        }
                        let old = self.focus_field(view, CHILD)?;
                        if old != child {
                            if old != Word::ZERO {
                                self.unfocus_view(old, 0)?;
                            }
                            self.focus_set(view, CHILD, child)?;
                        }
                        let parent = self.focus_field(view, PARENT)?;
                        if parent != Word::ZERO {
                            self.invoke(method.clone(), vec![parent, view, focused], true)?;
                        }
                    }
                }
                "clearChildFocus(Landroid/view/View;)V" if group => {
                    self.focus_root(view)?;
                    self.focus_field(arg(1)?, FOCUSED)?;
                    self.focus_set(view, CHILD, Word::ZERO)?;
                    let parent = self.focus_field(view, PARENT)?;
                    if parent != Word::ZERO {
                        self.invoke(method.clone(), vec![parent, view], true)?;
                    }
                }
                "clearFocus()V" => {
                    let root = self.focus_root(view)?;
                    let child = self.focus_field(view, CHILD)?;
                    if group && child != Word::ZERO {
                        self.focus_set(view, CHILD, Word::ZERO)?;
                        self.invoke(method.clone(), vec![child], true)?;
                    } else if self.focus_field(view, FOCUSED)?.truth() {
                        self.focus_set(view, FOCUSED, Word::ZERO)?;
                        let parent = self.focus_field(view, PARENT)?;
                        if parent != Word::ZERO {
                            self.invoke(
                                Method {
                                    class: "Landroid/view/ViewParent;".into(),
                                    name: "clearChildFocus".into(),
                                    parameters: vec!["Landroid/view/View;".into()],
                                    returns: "V".into(),
                                },
                                vec![parent, view],
                                true,
                            )?;
                        }
                        self.focus_changed(view, false, 0, Word::ZERO)?;
                        self.invoke(
                            Method {
                                class: "Landroid/view/View;".into(),
                                name: "requestFocus".into(),
                                parameters: vec![],
                                returns: "Z".into(),
                            },
                            vec![root],
                            true,
                        )?;
                    }
                }
                "onFocusChanged(ZILandroid/graphics/Rect;)V" if !group => {
                    self.focus_field(view, FOCUSED)?;
                    if !arg(1)?.truth()
                        && self.focus_field(view, "droidless:touch:pressed")?.truth()
                    {
                        self.invoke(
                            Method {
                                class: "Landroid/view/View;".into(),
                                name: "setPressed".into(),
                                parameters: vec!["Z".into()],
                                returns: "V".into(),
                            },
                            vec![view, Word::ZERO],
                            true,
                        )?;
                    }
                    let listener =
                        self.focus_field(view, "droidless:view:focus-change-listener")?;
                    if listener != Word::ZERO {
                        self.invoke(
                            Method {
                                class: "Landroid/view/View$OnFocusChangeListener;".into(),
                                name: "onFocusChange".into(),
                                parameters: vec!["Landroid/view/View;".into(), "Z".into()],
                                returns: "V".into(),
                            },
                            vec![listener, view, arg(1)?],
                            true,
                        )?;
                    }
                }
                "isFocused()Z" => words.push(self.focus_field(view, FOCUSED)?),
                "hasFocus()Z" => words.push(Word::from(i32::from(if group {
                    self.focus_field(view, FOCUSED)?.truth()
                        || self.focus_field(view, CHILD)? != Word::ZERO
                } else {
                    self.focus_field(view, FOCUSED)?.truth()
                }))),
                "findFocus()Landroid/view/View;" => words.push(if group {
                    self.find_focus(view)?
                } else if self.focus_field(view, FOCUSED)?.truth() {
                    view
                } else {
                    Word::ZERO
                }),
                "getFocusedChild()Landroid/view/View;" if group => {
                    words.push(self.focus_field(view, CHILD)?)
                }
                "getRootView()Landroid/view/View;" => words.push(self.focus_root(view)?),
                "isFocusable()Z" => words.push(self.focus_field(view, "droidless:setFocusable")?),
                "isFocusableInTouchMode()Z" => {
                    words.push(self.focus_field(view, "droidless:setFocusableInTouchMode")?)
                }
                "getDescendantFocusability()I" if group => {
                    words.push(self.focus_field(view, MODE)?)
                }
                "setDescendantFocusability(I)V" => {
                    ensure!(
                        self.is_a(&self.heap.get(view)?.class, "Landroid/view/ViewGroup;"),
                        "descendant focusability requires ViewGroup"
                    );
                    if ![BEFORE, AFTER, BLOCK].contains(&arg(1)?.int()?) {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "invalid descendant focusability",
                        ));
                    }
                    self.focus_set(view, MODE, arg(1)?)?;
                }
                _ => return Ok(None),
            }
            Ok(Some(words))
        })();
        self.native_roots.truncate(roots);
        result
    }
}
