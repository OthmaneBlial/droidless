use crate::{
    heap::{Data, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::{
    dex::Method,
    xml::{self, Value},
};
use std::collections::BTreeMap;

const MENU: &str = "Landroid/view/Menu;";
const ITEM: &str = "Landroid/view/MenuItem;";
const INFLATER: &str = "Landroid/view/MenuInflater;";
const CONTEXT: &str = "droidless:menu:context";
const OWNER: &str = "droidless:menu:owner";
const MAX_ITEMS: usize = 1024;

#[derive(Clone, Copy)]
struct Group {
    id: i32,
    category: i32,
    order: i32,
    checkable: i32,
    visible: bool,
    enabled: bool,
}
impl Default for Group {
    fn default() -> Self {
        Self {
            id: 0,
            category: 0,
            order: 0,
            checkable: 0,
            visible: true,
            enabled: true,
        }
    }
}
struct XmlItem {
    group: i32,
    id: i32,
    order: i32,
    title: Option<String>,
    condensed: Option<String>,
    icon: i32,
    checkable: i32,
    checked: bool,
    visible: bool,
    enabled: bool,
    action: Option<i32>,
}

fn ordering(order: i32) -> Result<u32> {
    // API-21 category precedence; getOrder still exposes the caller's original value.
    let raw = order as u32;
    let category = [1, 4, 5, 3, 2, 0]
        .get((raw >> 16) as usize)
        .ok_or_else(|| {
            fault(
                "Ljava/lang/IllegalArgumentException;",
                "invalid menu category",
            )
        })?;
    Ok((category << 16) | (raw & 0xffff))
}

impl Runtime {
    pub fn create_menu(&mut self, context: Word) -> Result<Word> {
        self.require_main_thread()?;
        ensure!(
            self.is_a(&self.heap.get(context)?.class, "Landroid/content/Context;"),
            "Menu requires Context"
        );
        let menu = self.heap.instance(MENU)?;
        let object = self.heap.get_mut(menu)?;
        object.data = Data::Menu(vec![]);
        object.fields.insert(CONTEXT.into(), vec![context]);
        Ok(menu)
    }
    pub(crate) fn menu_items(&self, menu: Word) -> Result<&[Word]> {
        match &self.heap.get(menu)?.data {
            Data::Menu(items) => Ok(items),
            _ => bail!("uninitialized or unsupported Menu implementation"),
        }
    }
    fn menu_items_mut(&mut self, menu: Word) -> Result<&mut Vec<Word>> {
        match &mut self.heap.get_mut(menu)?.data {
            Data::Menu(items) => Ok(items),
            _ => bail!("uninitialized or unsupported Menu implementation"),
        }
    }
    fn menu_field(&self, object: Word, field: &str) -> Result<Word> {
        Ok(self
            .heap
            .get(object)?
            .fields
            .get(field)
            .and_then(|v| v.first())
            .copied()
            .unwrap_or(Word::ZERO))
    }
    fn menu_set(&mut self, object: Word, field: &str, value: Word) -> Result<()> {
        self.heap
            .get_mut(object)?
            .fields
            .insert(field.into(), vec![value]);
        Ok(())
    }
    fn menu_add(
        &mut self,
        menu: Word,
        group: i32,
        id: i32,
        order: i32,
        title: Word,
    ) -> Result<Word> {
        let sorted = ordering(order)?;
        ensure!(
            self.menu_items(menu)?.len() < MAX_ITEMS,
            "Menu item limit reached (1024)"
        );
        if title != Word::ZERO {
            self.heap.text(title)?;
        }
        let item = self.heap.instance(ITEM)?;
        for (key, value) in [
            (OWNER, menu),
            ("group", Word::from(group)),
            ("id", Word::from(id)),
            ("order", Word::from(order)),
            ("title", title),
            ("enabled", Word::from(1)),
            ("visible", Word::from(1)),
        ] {
            self.menu_set(item, key, value)?;
        }
        let mut index = 0;
        for prior in self.menu_items(menu)? {
            if ordering(self.menu_field(*prior, "order")?.int()?)? > sorted {
                break;
            }
            index += 1;
        }
        self.menu_items_mut(menu)?.insert(index, item);
        Ok(item)
    }
    fn menu_icon(&mut self, item: Word, id: i32) -> Result<Word> {
        if id == 0 {
            return Ok(Word::ZERO);
        }
        let menu = self.menu_field(item, OWNER)?;
        let context = self.menu_field(menu, CONTEXT)?;
        let roots = self.native_roots.len();
        self.native_roots.extend([item, menu, context]);
        let result = self.invoke(
            Method {
                class: "Landroid/content/Context;".into(),
                name: "getDrawable".into(),
                parameters: vec!["I".into()],
                returns: "Landroid/graphics/drawable/Drawable;".into(),
            },
            vec![context, Word::from(id)],
            true,
        );
        self.native_roots.truncate(roots);
        let icon = result?
            .first()
            .copied()
            .context("missing menu icon return")?;
        ensure!(
            icon == Word::ZERO
                || self.is_a(
                    &self.heap.get(icon)?.class,
                    "Landroid/graphics/drawable/Drawable;"
                ),
            "invalid menu icon return"
        );
        Ok(icon)
    }
    fn menu_xml_int(
        &self,
        attrs: &BTreeMap<&str, &Value>,
        name: &str,
        default: i32,
    ) -> Result<i32> {
        let Some(value) = attrs.get(name) else {
            return Ok(default);
        };
        let value = if value.kind == 1 && !matches!(name, "id" | "icon") {
            self.apk.resources.resolve(value.data)?
        } else {
            value
        };
        ensure!(
            value.kind != 2,
            "theme attribute references in menu XML remain unsupported"
        );
        Ok(value.data as i32)
    }
    fn menu_xml_text(&self, attrs: &BTreeMap<&str, &Value>, name: &str) -> Result<Option<String>> {
        attrs
            .get(name)
            .map(|value| {
                ensure!(
                    value.kind != 2,
                    "theme attribute references in menu XML remain unsupported"
                );
                if value.kind == 1 {
                    self.resource_text(value.data)
                } else {
                    Ok(value.display())
                }
            })
            .transpose()
    }
    fn inflate_menu(&mut self, inflater: Word, id: u32, menu: Word) -> Result<()> {
        self.menu_items(menu)?;
        let path = self.apk.resources.resolve(id)?.display();
        let bytes = self
            .apk
            .files
            .get(&path)
            .context("menu XML resource missing")?;
        ensure!(bytes.len() <= 1_048_576, "menu XML exceeds 1 MiB");
        let events = xml::parse_events(bytes)?;
        ensure!(
            events
                .iter()
                .find(|event| event.kind == 2)
                .is_some_and(|event| event.name.as_deref() == Some("menu")),
            "expected menu XML resource"
        );
        let mut group = Group::default();
        let mut grouped = false;
        let mut pending = vec![];
        for event in events {
            let attrs: BTreeMap<_, _> = event
                .attributes
                .iter()
                .filter(|attr| {
                    attr.namespace.as_deref() == Some("http://schemas.android.com/apk/res/android")
                })
                .map(|attr| (attr.name.as_str(), &attr.value))
                .collect();
            match (event.kind, event.name.as_deref()) {
                (2, Some("menu")) => {
                    ensure!(event.depth == 1, "submenu inflation remains unsupported")
                }
                (2, Some("group")) => {
                    ensure!(
                        event.depth == 2 && !grouped,
                        "nested menu groups remain unsupported"
                    );
                    grouped = true;
                    group = Group {
                        id: self.menu_xml_int(&attrs, "id", 0)?,
                        category: self.menu_xml_int(&attrs, "menuCategory", 0)?,
                        order: self.menu_xml_int(&attrs, "orderInCategory", 0)?,
                        checkable: self.menu_xml_int(&attrs, "checkableBehavior", 0)?,
                        visible: self.menu_xml_int(&attrs, "visible", 1)? != 0,
                        enabled: self.menu_xml_int(&attrs, "enabled", 1)? != 0,
                    };
                }
                (3, Some("group")) => {
                    group = Group::default();
                    grouped = false;
                }
                (2, Some("item")) => {
                    ensure!(
                        event.depth == if grouped { 3 } else { 2 },
                        "invalid menu item nesting"
                    );
                    for name in [
                        "onClick",
                        "actionLayout",
                        "actionViewClass",
                        "actionProviderClass",
                        "alphabeticShortcut",
                        "numericShortcut",
                    ] {
                        ensure!(
                            !attrs.contains_key(name),
                            "menu XML {name} remains unsupported"
                        );
                    }
                    let category = self.menu_xml_int(&attrs, "menuCategory", group.category)?;
                    let order = (category & !0xffff)
                        | (self.menu_xml_int(&attrs, "orderInCategory", group.order)? & 0xffff);
                    ordering(order)?;
                    let action = attrs
                        .contains_key("showAsAction")
                        .then(|| self.menu_xml_int(&attrs, "showAsAction", 0))
                        .transpose()?;
                    if let Some(action) = action {
                        ensure!(
                            action & 3 != 3,
                            fault(
                                "Ljava/lang/IllegalArgumentException;",
                                "conflicting showAsAction flags"
                            )
                        );
                    }
                    ensure!(
                        pending.len() + self.menu_items(menu)?.len() < MAX_ITEMS,
                        "Menu item limit reached (1024)"
                    );
                    pending.push(XmlItem {
                        group: group.id,
                        id: self.menu_xml_int(&attrs, "id", 0)?,
                        order,
                        title: self.menu_xml_text(&attrs, "title")?,
                        condensed: self.menu_xml_text(&attrs, "titleCondensed")?,
                        icon: self.menu_xml_int(&attrs, "icon", 0)?,
                        checkable: if attrs.contains_key("checkable") {
                            i32::from(self.menu_xml_int(&attrs, "checkable", 0)? != 0)
                        } else {
                            group.checkable
                        },
                        checked: self.menu_xml_int(&attrs, "checked", 0)? != 0,
                        visible: self.menu_xml_int(&attrs, "visible", i32::from(group.visible))?
                            != 0,
                        enabled: self.menu_xml_int(&attrs, "enabled", i32::from(group.enabled))?
                            != 0,
                        action,
                    });
                }
                (2, Some(name)) => bail!("unsupported menu XML element {name}"),
                _ => {}
            }
        }
        let roots = self.native_roots.len();
        self.native_roots.extend([inflater, menu]);
        let result = (|| {
            for pending in pending {
                let title = pending
                    .title
                    .map(|text| self.heap.string(text))
                    .transpose()?
                    .unwrap_or(Word::ZERO);
                let item = self.menu_add(menu, pending.group, pending.id, pending.order, title)?;
                let condensed = pending
                    .condensed
                    .map(|text| self.heap.string(text))
                    .transpose()?
                    .unwrap_or(Word::ZERO);
                self.menu_set(item, "condensed", condensed)?;
                for (key, value) in [
                    ("visible", pending.visible),
                    ("enabled", pending.enabled),
                    ("checked", pending.checked),
                    ("checkable", pending.checkable > 0),
                    ("exclusive", pending.checkable >= 2),
                ] {
                    self.menu_set(item, key, Word::from(i32::from(value)))?;
                }
                if let Some(action) = pending.action {
                    self.menu_set(item, "showAsAction", Word::from(action))?;
                }
                let icon = self.menu_icon(item, pending.icon)?;
                self.menu_set(item, "icon", icon)?;
            }
            Ok(())
        })();
        self.native_roots.truncate(roots);
        result
    }
    pub(crate) fn menu_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let arg = |index| args.get(index).copied().context("menu argument missing");
        let signature = method.signature();
        let receiver = args.first().copied().unwrap_or(Word::ZERO);
        let result = match (method.class.as_str(), signature.as_str()) {
            (INFLATER, "<init>(Landroid/content/Context;)V") => {
                let context = arg(1)?;
                ensure!(
                    self.is_a(&self.heap.get(context)?.class, "Landroid/content/Context;"),
                    "MenuInflater requires Context"
                );
                self.menu_set(receiver, CONTEXT, context)?;
                vec![]
            }
            (INFLATER, "inflate(ILandroid/view/Menu;)V") => {
                self.inflate_menu(receiver, arg(1)?.int()? as u32, arg(2)?)?;
                vec![]
            }
            ("Landroid/app/Activity;", "getMenuInflater()Landroid/view/MenuInflater;") => {
                let inflater = self.heap.instance(INFLATER)?;
                self.menu_set(inflater, CONTEXT, receiver)?;
                vec![inflater]
            }
            (
                "Landroid/app/Activity;",
                "onCreateOptionsMenu(Landroid/view/Menu;)Z"
                | "onPrepareOptionsMenu(Landroid/view/Menu;)Z",
            ) => vec![Word::from(1)],
            ("Landroid/app/Activity;", "onOptionsItemSelected(Landroid/view/MenuItem;)Z") => {
                vec![Word::ZERO]
            }
            (
                MENU,
                "add(Ljava/lang/CharSequence;)Landroid/view/MenuItem;"
                | "add(I)Landroid/view/MenuItem;"
                | "add(IIILjava/lang/CharSequence;)Landroid/view/MenuItem;"
                | "add(IIII)Landroid/view/MenuItem;",
            ) => {
                let full = method.parameters.len() == 4;
                let value = arg(if full { 4 } else { 1 })?;
                let title = if method.parameters.last().is_some_and(|p| p == "I") {
                    let title = self.resource_text(value.int()? as u32)?;
                    self.heap.string(title)?
                } else {
                    value
                };
                vec![self.menu_add(
                    receiver,
                    if full { arg(1)?.int()? } else { 0 },
                    if full { arg(2)?.int()? } else { 0 },
                    if full { arg(3)?.int()? } else { 0 },
                    title,
                )?]
            }
            (MENU, "size()I") => vec![Word::from(self.menu_items(receiver)?.len() as i32)],
            (MENU, "getItem(I)Landroid/view/MenuItem;") => {
                let index = arg(1)?.int()?;
                let items = self.menu_items(receiver)?;
                let item = usize::try_from(index)
                    .ok()
                    .and_then(|index| items.get(index))
                    .copied()
                    .ok_or_else(|| {
                        fault("Ljava/lang/IndexOutOfBoundsException;", "menu item index")
                    })?;
                vec![item]
            }
            (MENU, "findItem(I)Landroid/view/MenuItem;") => {
                let id = arg(1)?;
                let mut found = Word::ZERO;
                for item in self.menu_items(receiver)? {
                    if self.menu_field(*item, "id")? == id {
                        found = *item;
                        break;
                    }
                }
                vec![found]
            }
            (MENU, "clear()V") => {
                self.menu_items_mut(receiver)?.clear();
                vec![]
            }
            (MENU, "removeItem(I)V" | "removeGroup(I)V") => {
                let value = arg(1)?;
                let group = signature == "removeGroup(I)V";
                let key = if group { "group" } else { "id" };
                let mut kept = vec![];
                let mut removed = false;
                for item in self.menu_items(receiver)? {
                    if self.menu_field(*item, key)? == value && (group || !removed) {
                        removed = true;
                    } else {
                        kept.push(*item);
                    }
                }
                *self.menu_items_mut(receiver)? = kept;
                vec![]
            }
            (MENU, "setGroupEnabled(IZ)V" | "setGroupVisible(IZ)V" | "setGroupCheckable(IZZ)V") => {
                let group = arg(1)?;
                let key = match signature.as_str() {
                    "setGroupEnabled(IZ)V" => "enabled",
                    "setGroupVisible(IZ)V" => "visible",
                    _ => "checkable",
                };
                for item in self.menu_items(receiver)?.to_vec() {
                    if self.menu_field(item, "group")? == group {
                        self.menu_set(item, key, Word::from(i32::from(arg(2)?.truth())))?;
                        if key == "checkable" {
                            self.menu_set(
                                item,
                                "exclusive",
                                Word::from(i32::from(arg(3)?.truth())),
                            )?;
                        }
                    }
                }
                vec![]
            }
            (MENU, "hasVisibleItems()Z") => {
                let mut visible = false;
                for item in self.menu_items(receiver)? {
                    visible |= self.menu_field(*item, "visible")?.truth();
                }
                vec![Word::from(i32::from(visible))]
            }
            (
                ITEM,
                "getItemId()I"
                | "getGroupId()I"
                | "getOrder()I"
                | "getTitle()Ljava/lang/CharSequence;"
                | "getIcon()Landroid/graphics/drawable/Drawable;"
                | "isEnabled()Z"
                | "isVisible()Z"
                | "isCheckable()Z"
                | "isChecked()Z",
            ) => {
                let key = match method.name.as_str() {
                    "getItemId" => "id",
                    "getGroupId" => "group",
                    "getOrder" => "order",
                    "getTitle" => "title",
                    "getIcon" => "icon",
                    "isEnabled" => "enabled",
                    "isVisible" => "visible",
                    "isCheckable" => "checkable",
                    _ => "checked",
                };
                vec![self.menu_field(receiver, key)?]
            }
            (ITEM, "getTitleCondensed()Ljava/lang/CharSequence;") => {
                let value = self.menu_field(receiver, "condensed")?;
                vec![if value == Word::ZERO {
                    self.menu_field(receiver, "title")?
                } else {
                    value
                }]
            }
            (ITEM, "hasSubMenu()Z") => vec![Word::ZERO],
            (ITEM, "getSubMenu()Landroid/view/SubMenu;") => vec![Word::ZERO],
            (
                ITEM,
                "setTitle(Ljava/lang/CharSequence;)Landroid/view/MenuItem;"
                | "setTitle(I)Landroid/view/MenuItem;"
                | "setTitleCondensed(Ljava/lang/CharSequence;)Landroid/view/MenuItem;"
                | "setIcon(I)Landroid/view/MenuItem;"
                | "setIcon(Landroid/graphics/drawable/Drawable;)Landroid/view/MenuItem;"
                | "setEnabled(Z)Landroid/view/MenuItem;"
                | "setVisible(Z)Landroid/view/MenuItem;"
                | "setCheckable(Z)Landroid/view/MenuItem;"
                | "setOnMenuItemClickListener(Landroid/view/MenuItem$OnMenuItemClickListener;)Landroid/view/MenuItem;",
            ) => {
                let key = match method.name.as_str() {
                    "setTitle" => "title",
                    "setTitleCondensed" => "condensed",
                    "setIcon" => "icon",
                    "setEnabled" => "enabled",
                    "setVisible" => "visible",
                    "setCheckable" => "checkable",
                    _ => "listener",
                };
                let value = if signature == "setTitle(I)Landroid/view/MenuItem;" {
                    let title = self.resource_text(arg(1)?.int()? as u32)?;
                    self.heap.string(title)?
                } else if signature == "setIcon(I)Landroid/view/MenuItem;" {
                    self.menu_icon(receiver, arg(1)?.int()?)?
                } else if method.parameters[0] == "Z" {
                    Word::from(i32::from(arg(1)?.truth()))
                } else {
                    arg(1)?
                };
                if value != Word::ZERO {
                    match key {
                        "title" | "condensed" => {
                            self.heap.text(value)?;
                        }
                        "icon" | "listener" => {
                            let ty = if key == "icon" {
                                "Landroid/graphics/drawable/Drawable;"
                            } else {
                                "Landroid/view/MenuItem$OnMenuItemClickListener;"
                            };
                            ensure!(
                                self.is_a(&self.heap.get(value)?.class, ty),
                                "invalid menu {key}"
                            );
                        }
                        _ => {}
                    }
                }
                self.menu_set(receiver, key, value)?;
                vec![receiver]
            }
            (ITEM, "setChecked(Z)Landroid/view/MenuItem;") => {
                let checked = Word::from(i32::from(arg(1)?.truth()));
                if self.menu_field(receiver, "exclusive")?.truth() {
                    let menu = self.menu_field(receiver, OWNER)?;
                    let group = self.menu_field(receiver, "group")?;
                    for item in self.menu_items(menu)?.to_vec() {
                        if self.menu_field(item, "group")? == group
                            && self.menu_field(item, "checkable")?.truth()
                            && self.menu_field(item, "exclusive")?.truth()
                        {
                            self.menu_set(
                                item,
                                "checked",
                                Word::from(i32::from(item == receiver)),
                            )?;
                        }
                    }
                } else {
                    self.menu_set(receiver, "checked", checked)?;
                }
                vec![receiver]
            }
            (ITEM, "setShowAsAction(I)V" | "setShowAsActionFlags(I)Landroid/view/MenuItem;") => {
                let flags = arg(1)?.int()?;
                ensure!(
                    flags & 3 != 3,
                    fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "conflicting showAsAction flags"
                    )
                );
                self.menu_set(receiver, "showAsAction", Word::from(flags))?;
                if method.returns == "V" {
                    vec![]
                } else {
                    vec![receiver]
                }
            }
            _ => return Ok(None),
        };
        Ok(Some(result))
    }
}
