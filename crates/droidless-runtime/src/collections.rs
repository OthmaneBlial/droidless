//! Bounded Java collections; elements remain guest references and use guest equals.
use crate::{
    heap::{Data, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;

const ITERATOR: &str = "Ldroidless/runtime/CollectionIterator;";
const SNAPSHOT_ITERATOR: &str = "Ldroidless/runtime/SnapshotIterator;";
const COPY_ON_WRITE_LIST: &str = "Ljava/util/concurrent/CopyOnWriteArrayList;";
const READ_ONLY_ITERATOR: &str = "Ldroidless/runtime/UnmodifiableIterator;";
const READ_ONLY_SET: &str = "Ldroidless/runtime/UnmodifiableSet;";
const READ_ONLY_LIST: &str = "Ldroidless/runtime/UnmodifiableList;";
const READ_ONLY_RANDOM_LIST: &str = "Ldroidless/runtime/UnmodifiableRandomAccessList;";
const LIMIT: usize = 16_384;

impl Runtime {
    pub(crate) fn object_equal(&mut self, key: Word, value: Word, identity: bool) -> Result<bool> {
        key.reference()?;
        value.reference()?;
        if key == value && (identity || key == Word::ZERO) {
            return Ok(true);
        }
        if key == Word::ZERO {
            return Ok(false);
        }
        Ok(self
            .invoke(
                Method {
                    class: "Ljava/lang/Object;".into(),
                    name: "equals".into(),
                    parameters: vec!["Ljava/lang/Object;".into()],
                    returns: "Z".into(),
                },
                vec![key, value],
                true,
            )?
            .first()
            .context("equals returned no value")?
            .int()?
            != 0)
    }
    pub(crate) fn collection(&self, owner: Word) -> Result<(&[Word], u32)> {
        let Data::Collection { values, version } = &self.heap.get(owner)?.data else {
            bail!("uninitialized collection");
        };
        Ok((values, *version))
    }
    pub(crate) fn change_collection(&mut self, owner: Word, values: Vec<Word>) -> Result<()> {
        ensure!(
            values.len() <= LIMIT,
            "collection entry limit reached ({LIMIT})"
        );
        let version = self
            .collection(owner)?
            .1
            .checked_add(1)
            .context("collection version exhausted")?;
        self.heap.get_mut(owner)?.data = Data::Collection { values, version };
        Ok(())
    }
    fn collection_find(&mut self, owner: Word, key: Word, last: bool) -> Result<Option<usize>> {
        key.reference()?;
        let (values, version) = self.collection(owner)?;
        if self.is_a(&self.heap.get(owner)?.class, COPY_ON_WRITE_LIST) {
            let values = values.to_vec();
            let roots = self.native_roots.len();
            self.native_roots.extend([owner, key]);
            self.native_roots.extend(values.iter().copied());
            let found = (|| -> Result<Option<usize>> {
                for offset in 0..values.len() {
                    let index = if last {
                        values.len() - 1 - offset
                    } else {
                        offset
                    };
                    if self.object_equal(key, values[index], false)? {
                        return Ok(Some(index));
                    }
                }
                Ok(None)
            })();
            self.native_roots.truncate(roots);
            return found;
        }
        // ponytail: linear membership with guest equals; use hash buckets if profiling justifies them.
        let len = values.len();
        let identity = !self.is_a(&self.heap.get(owner)?.class, "Ljava/util/ArrayList;");
        for offset in 0..len {
            let index = if last { len - 1 - offset } else { offset };
            // A guest equals callback may set a later List element without changing its version.
            let value = self.collection(owner)?.0[index];
            let equal = self.object_equal(key, value, identity)?;
            if self.collection(owner)?.1 != version {
                return Err(fault(
                    "Ljava/util/ConcurrentModificationException;",
                    "collection changed during equals",
                ));
            }
            if equal {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    fn list_index(&self, owner: Word, index: Word, insertion: bool) -> Result<usize> {
        let index = index.int()?;
        let len = self.collection(owner)?.0.len();
        if index < 0 || index as usize >= len + usize::from(insertion) {
            return Err(fault(
                "Ljava/lang/IndexOutOfBoundsException;",
                format!("index {index}, size {len}"),
            ));
        }
        Ok(index as usize)
    }
    pub(crate) fn collection_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method.signature();
        if method.class == READ_ONLY_ITERATOR {
            if signature == "remove()V" {
                return Err(fault(
                    "Ljava/lang/UnsupportedOperationException;",
                    "unmodifiable iterator",
                ));
            }
            if !["hasNext()Z", "next()Ljava/lang/Object;"].contains(&signature.as_str()) {
                return Ok(None);
            }
            let wrapper = *args.first().context("iterator receiver missing")?;
            let owner = self
                .heap
                .get(wrapper)?
                .fields
                .get("owner")
                .and_then(|v| v.first())
                .copied()
                .context("uninitialized iterator view")?;
            return Ok(Some(self.invoke(
                Method {
                    class: "Ljava/util/Iterator;".into(),
                    ..method.clone()
                },
                vec![owner],
                true,
            )?));
        }
        if method.class == "Ljava/util/Collections;" {
            let (interface, wrapper_class) = match signature.as_str() {
                "unmodifiableSet(Ljava/util/Set;)Ljava/util/Set;" => {
                    ("Ljava/util/Set;", READ_ONLY_SET)
                }
                "unmodifiableList(Ljava/util/List;)Ljava/util/List;" => {
                    ("Ljava/util/List;", READ_ONLY_LIST)
                }
                _ => return Ok(None),
            };
            let owner = *args.first().context("collection argument missing")?;
            let class = &self.heap.get(owner)?.class;
            ensure!(
                self.is_a(class, interface),
                "unmodifiable view requires {interface}"
            );
            let wrapper_class = if wrapper_class == READ_ONLY_LIST
                && self.is_a(class, "Ljava/util/RandomAccess;")
            {
                READ_ONLY_RANDOM_LIST
            } else {
                wrapper_class
            };
            let wrapper = self.heap.instance(wrapper_class)?;
            self.heap
                .get_mut(wrapper)?
                .fields
                .insert("owner".into(), vec![owner]);
            return Ok(Some(vec![wrapper]));
        }
        let read_only_list =
            method.class == READ_ONLY_LIST || method.class == READ_ONLY_RANDOM_LIST;
        if method.class == READ_ONLY_SET || read_only_list {
            let wrapper = *args.first().context("collection receiver missing")?;
            let sig = signature;
            if [
                "add(Ljava/lang/Object;)Z",
                "remove(Ljava/lang/Object;)Z",
                "clear()V",
                "addAll(Ljava/util/Collection;)Z",
                "removeAll(Ljava/util/Collection;)Z",
                "retainAll(Ljava/util/Collection;)Z",
            ]
            .contains(&sig.as_str())
                || (read_only_list
                    && [
                        "set(ILjava/lang/Object;)Ljava/lang/Object;",
                        "add(ILjava/lang/Object;)V",
                        "remove(I)Ljava/lang/Object;",
                        "addAll(ILjava/util/Collection;)Z",
                    ]
                    .contains(&sig.as_str()))
            {
                return Err(fault(
                    "Ljava/lang/UnsupportedOperationException;",
                    "unmodifiable collection",
                ));
            }
            let read = [
                "size()I",
                "isEmpty()Z",
                "contains(Ljava/lang/Object;)Z",
                "containsAll(Ljava/util/Collection;)Z",
                "iterator()Ljava/util/Iterator;",
                "toArray()[Ljava/lang/Object;",
                "toArray([Ljava/lang/Object;)[Ljava/lang/Object;",
                "equals(Ljava/lang/Object;)Z",
                "hashCode()I",
                "toString()Ljava/lang/String;",
            ]
            .contains(&sig.as_str())
                || (read_only_list
                    && [
                        "get(I)Ljava/lang/Object;",
                        "indexOf(Ljava/lang/Object;)I",
                        "lastIndexOf(Ljava/lang/Object;)I",
                    ]
                    .contains(&sig.as_str()));
            if !read {
                return Ok(None);
            }
            let owner = self
                .heap
                .get(wrapper)?
                .fields
                .get("owner")
                .and_then(|v| v.first())
                .copied()
                .context("uninitialized collection view")?;
            let mut forwarded = args.to_vec();
            forwarded[0] = owner;
            let result = self.invoke(
                Method {
                    class: if read_only_list {
                        "Ljava/util/List;"
                    } else {
                        "Ljava/util/Set;"
                    }
                    .into(),
                    ..method.clone()
                },
                forwarded,
                true,
            )?;
            if sig == "iterator()Ljava/util/Iterator;" {
                let iterator = *result.first().context("iterator returned no value")?;
                ensure!(
                    [ITERATOR, SNAPSHOT_ITERATOR, READ_ONLY_ITERATOR]
                        .contains(&self.heap.get(iterator)?.class.as_str()),
                    "unmodifiable iterator for this collection is unsupported"
                );
                let view = self.heap.instance(READ_ONLY_ITERATOR)?;
                self.heap
                    .get_mut(view)?
                    .fields
                    .insert("owner".into(), vec![iterator]);
                return Ok(Some(vec![view]));
            }
            return Ok(Some(result));
        }
        if method.class == "Ljava/util/HashMap;" || method.class == "Ljava/util/LinkedHashMap;" {
            return self.map_native(method, args);
        }
        let cow = method.class == COPY_ON_WRITE_LIST;
        let list = method.class == "Ljava/util/ArrayList;" || cow;
        let snapshot_iterator = method.class == SNAPSHOT_ITERATOR;
        if !list
            && method.class != "Ljava/util/HashSet;"
            && method.class != ITERATOR
            && !snapshot_iterator
        {
            return Ok(None);
        }
        let receiver = *args.first().context("collection receiver missing")?;
        let arg = |n| args.get(n).copied().context("collection argument missing");
        let sig = method.signature();
        let mut result = vec![];
        if method.class == ITERATOR || snapshot_iterator {
            if snapshot_iterator && sig == "remove()V" {
                return Err(fault(
                    "Ljava/lang/UnsupportedOperationException;",
                    "snapshot iterator",
                ));
            }
            let object = self.heap.get(receiver)?;
            let field = |name: &str| {
                object
                    .fields
                    .get(name)
                    .and_then(|v| v.first())
                    .copied()
                    .context("uninitialized iterator")
            };
            let owner = if snapshot_iterator {
                receiver
            } else {
                field("owner")?
            };
            let position = field("position")?.int()? as usize;
            let (values, version) = self.collection(owner)?;
            if !snapshot_iterator
                && sig != "hasNext()Z"
                && field("version")?.int()? as u32 != version
            {
                return Err(fault(
                    "Ljava/util/ConcurrentModificationException;",
                    "collection changed during iteration",
                ));
            }
            match sig.as_str() {
                "hasNext()Z" => result.push(Word::from(i32::from(position < values.len()))),
                "next()Ljava/lang/Object;" => {
                    let value = values.get(position).copied().ok_or_else(|| {
                        fault("Ljava/util/NoSuchElementException;", "iterator exhausted")
                    })?;
                    let fields = &mut self.heap.get_mut(receiver)?.fields;
                    fields.insert("position".into(), vec![Word::from((position + 1) as i32)]);
                    fields.insert("last".into(), vec![Word::from(position as i32)]);
                    result.push(value);
                }
                "remove()V" => {
                    let last = field("last")?.int()?;
                    if last < 0 {
                        return Err(fault(
                            "Ljava/lang/IllegalStateException;",
                            "iterator remove requires next",
                        ));
                    }
                    ensure!((last as usize) < values.len(), "invalid iterator position");
                    if self.is_a(&self.heap.get(owner)?.class, "Ljava/util/ArrayList;") {
                        self.invoke(
                            Method {
                                class: "Ljava/util/ArrayList;".into(),
                                name: "remove".into(),
                                parameters: vec!["I".into()],
                                returns: "Ljava/lang/Object;".into(),
                            },
                            vec![owner, Word::from(last)],
                            true,
                        )?;
                    } else {
                        let mut values = values.to_vec();
                        values.remove(last as usize);
                        self.change_collection(owner, values)?;
                    }
                    let version = self.collection(owner)?.1;
                    let fields = &mut self.heap.get_mut(receiver)?.fields;
                    fields.insert("position".into(), vec![Word::from(last)]);
                    fields.insert("last".into(), vec![Word::from(-1)]);
                    fields.insert("version".into(), vec![Word::Bits(version)]);
                }
                _ => return Ok(None),
            }
            return Ok(Some(result));
        }
        ensure!(
            self.is_a(&self.heap.get(receiver)?.class, &method.class),
            "invalid collection receiver"
        );
        match sig.as_str() {
            "<init>()V" | "<init>(I)V" if !cow || sig == "<init>()V" => {
                if sig == "<init>(I)V" && arg(1)?.int()? < 0 {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "negative collection capacity",
                    ));
                }
                self.heap.get_mut(receiver)?.data = Data::Collection {
                    values: vec![],
                    version: 0,
                }
            }
            "size()I" => result.push(Word::from(self.collection(receiver)?.0.len() as i32)),
            "isEmpty()Z" => result.push(Word::from(i32::from(
                self.collection(receiver)?.0.is_empty(),
            ))),
            "contains(Ljava/lang/Object;)Z" => result.push(Word::from(i32::from(
                self.collection_find(receiver, arg(1)?, false)?.is_some(),
            ))),
            "add(Ljava/lang/Object;)Z" => {
                let value = arg(1)?;
                value.reference()?;
                let exists = !list && self.collection_find(receiver, value, false)?.is_some();
                if !exists {
                    let mut values = self.collection(receiver)?.0.to_vec();
                    values.push(value);
                    self.change_collection(receiver, values)?;
                }
                result.push(Word::from(i32::from(!exists)));
            }
            "remove(Ljava/lang/Object;)Z" => {
                let version = self.collection(receiver)?.1;
                let index = self.collection_find(receiver, arg(1)?, false)?;
                if cow {
                    ensure!(
                        self.collection(receiver)?.1 == version,
                        "unsupported CopyOnWriteArrayList mutation during remove equality"
                    );
                }
                if let Some(index) = index {
                    let mut values = self.collection(receiver)?.0.to_vec();
                    values.remove(index);
                    self.change_collection(receiver, values)?;
                }
                result.push(Word::from(i32::from(index.is_some())));
            }
            "indexOf(Ljava/lang/Object;)I" | "lastIndexOf(Ljava/lang/Object;)I" if list => {
                let index = self.collection_find(receiver, arg(1)?, sig.starts_with("last"))?;
                result.push(Word::from(index.map_or(-1, |i| i as i32)));
            }
            "get(I)Ljava/lang/Object;" if list => {
                let index = self.list_index(receiver, arg(1)?, false)?;
                result.push(self.collection(receiver)?.0[index]);
            }
            "set(ILjava/lang/Object;)Ljava/lang/Object;" if list => {
                let index = self.list_index(receiver, arg(1)?, false)?;
                let value = arg(2)?;
                value.reference()?;
                if cow {
                    let mut values = self.collection(receiver)?.0.to_vec();
                    result.push(std::mem::replace(&mut values[index], value));
                    self.change_collection(receiver, values)?;
                } else {
                    let Data::Collection { values, .. } = &mut self.heap.get_mut(receiver)?.data
                    else {
                        bail!("uninitialized collection");
                    };
                    result.push(std::mem::replace(&mut values[index], value));
                }
            }
            "add(ILjava/lang/Object;)V" if list => {
                let index = self.list_index(receiver, arg(1)?, true)?;
                let value = arg(2)?;
                value.reference()?;
                let mut values = self.collection(receiver)?.0.to_vec();
                values.insert(index, value);
                self.change_collection(receiver, values)?;
            }
            "remove(I)Ljava/lang/Object;" if list => {
                let index = self.list_index(receiver, arg(1)?, false)?;
                let mut values = self.collection(receiver)?.0.to_vec();
                result.push(values.remove(index));
                self.change_collection(receiver, values)?;
            }
            "clear()V" => self.change_collection(receiver, vec![])?,
            "iterator()Ljava/util/Iterator;" => {
                if cow {
                    // ponytail: bounded snapshot copy; share immutable arrays if iterator allocation is profiled.
                    let values = self.collection(receiver)?.0.to_vec();
                    let iterator = self.heap.instance(SNAPSHOT_ITERATOR)?;
                    let object = self.heap.get_mut(iterator)?;
                    object.data = Data::Collection { values, version: 0 };
                    object.fields.insert("position".into(), vec![Word::ZERO]);
                    result.push(iterator);
                    return Ok(Some(result));
                }
                let version = self.collection(receiver)?.1;
                let iterator = self.heap.instance(ITERATOR)?;
                let fields = &mut self.heap.get_mut(iterator)?.fields;
                fields.insert("owner".into(), vec![receiver]);
                fields.insert("position".into(), vec![Word::ZERO]);
                fields.insert("last".into(), vec![Word::from(-1)]);
                fields.insert("version".into(), vec![Word::Bits(version)]);
                result.push(iterator);
            }
            // Collections override these; do not fall back to Object's behavior.
            "equals(Ljava/lang/Object;)Z" | "hashCode()I" | "toString()Ljava/lang/String;" => {
                bail!("unsupported collection method {}", method.key())
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
    fn map(&self, owner: Word) -> Result<(&[(Word, Word)], u32)> {
        let Data::Map { entries, version } = &self.heap.get(owner)?.data else {
            bail!("uninitialized map");
        };
        Ok((entries, *version))
    }
    fn map_find(&mut self, owner: Word, key: Word) -> Result<Option<usize>> {
        key.reference()?;
        let (entries, version) = self.map(owner)?;
        // ponytail: linear key lookup; use guest hash buckets when large maps are profiled.
        let entries = entries.to_vec();
        for (index, (candidate, _)) in entries.into_iter().enumerate() {
            let equal = self.object_equal(key, candidate, true)?;
            if self.map(owner)?.1 != version {
                return Err(fault(
                    "Ljava/util/ConcurrentModificationException;",
                    "map changed during equals",
                ));
            }
            if equal {
                return Ok(Some(index));
            }
        }
        Ok(None)
    }
    fn map_native(&mut self, method: &Method, args: &[Word]) -> Result<Option<Vec<Word>>> {
        let owner = *args.first().context("map receiver missing")?;
        let class = &self.heap.get(owner)?.class;
        // LinkedHashMap.put must dispatch removeEldestEntry; don't silently skip subclass hooks.
        ensure!(
            class == "Ljava/util/LinkedHashMap;" || !self.is_a(class, "Ljava/util/LinkedHashMap;"),
            "LinkedHashMap subclasses and eviction hooks are unsupported"
        );
        ensure!(
            self.is_a(&self.heap.get(owner)?.class, "Ljava/util/HashMap;"),
            "invalid HashMap receiver"
        );
        let arg = |n| args.get(n).copied().context("map argument missing");
        let sig = method.signature();
        let mut result = vec![];
        match sig.as_str() {
            "<init>()V" | "<init>(I)V" => {
                if sig == "<init>(I)V" && arg(1)?.int()? < 0 {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "negative map capacity",
                    ));
                }
                self.heap.get_mut(owner)?.data = Data::Map {
                    entries: vec![],
                    version: 0,
                };
            }
            "size()I" => result.push(Word::from(self.map(owner)?.0.len() as i32)),
            "isEmpty()Z" => result.push(Word::from(i32::from(self.map(owner)?.0.is_empty()))),
            "containsKey(Ljava/lang/Object;)Z" => result.push(Word::from(i32::from(
                self.map_find(owner, arg(1)?)?.is_some(),
            ))),
            "containsValue(Ljava/lang/Object;)Z" => {
                let values = self
                    .map(owner)?
                    .0
                    .iter()
                    .map(|(_, v)| *v)
                    .collect::<Vec<_>>();
                let mut found = false;
                for value in values {
                    if self.object_equal(arg(1)?, value, true)? {
                        found = true;
                        break;
                    }
                }
                result.push(Word::from(i32::from(found)));
            }
            "get(Ljava/lang/Object;)Ljava/lang/Object;" => {
                let index = self.map_find(owner, arg(1)?)?;
                result.push(match index {
                    Some(i) => self.map(owner)?.0[i].1,
                    None => Word::ZERO,
                });
            }
            "putAll(Ljava/util/Map;)V" => {
                let source = arg(1)?;
                for map in [owner, source] {
                    ensure!(
                        matches!(
                            self.heap.get(map)?.class.as_str(),
                            "Ljava/util/HashMap;" | "Ljava/util/LinkedHashMap;"
                        ),
                        "unsupported putAll with custom Map implementations or subclass hooks"
                    );
                }
                let (entries, version) = self.map(source)?;
                let entries = entries.to_vec();
                // A key's guest equals may mutate a source and collect its old values.
                // Keep the native snapshot alive until every copy/error path has returned.
                let roots = self.native_roots.len();
                self.native_roots.extend([owner, source]);
                self.native_roots
                    .extend(entries.iter().flat_map(|(k, v)| [*k, *v]));
                let put = Method {
                    class: method.class.clone(),
                    name: "put".into(),
                    parameters: vec!["Ljava/lang/Object;".into(); 2],
                    returns: "Ljava/lang/Object;".into(),
                };
                let copied = (|| -> Result<()> {
                    for (key, value) in &entries {
                        self.map_native(&put, &[owner, *key, *value])?;
                        if owner != source {
                            let (current, current_version) = self.map(source)?;
                            ensure!(
                                current_version == version && current == entries.as_slice(),
                                "unsupported source Map mutation during putAll"
                            );
                        }
                    }
                    Ok(())
                })();
                self.native_roots.truncate(roots);
                copied?;
            }
            "put(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;" => {
                arg(2)?.reference()?;
                let index = self.map_find(owner, arg(1)?)?;
                let (entries, mut version) = self.map(owner)?;
                let mut entries = entries.to_vec();
                let old = if let Some(index) = index {
                    let old = entries[index].1;
                    entries[index].1 = arg(2)?;
                    old
                } else {
                    ensure!(entries.len() < LIMIT, "map entry limit reached ({LIMIT})");
                    entries.push((arg(1)?, arg(2)?));
                    version = version.checked_add(1).context("map version exhausted")?;
                    Word::ZERO
                };
                self.heap.get_mut(owner)?.data = Data::Map { entries, version };
                result.push(old);
            }
            "remove(Ljava/lang/Object;)Ljava/lang/Object;" => {
                let index = self.map_find(owner, arg(1)?)?;
                let old = if let Some(index) = index {
                    let (entries, version) = self.map(owner)?;
                    let mut entries = entries.to_vec();
                    let old = entries.remove(index).1;
                    let version = version.checked_add(1).context("map version exhausted")?;
                    self.heap.get_mut(owner)?.data = Data::Map { entries, version };
                    old
                } else {
                    Word::ZERO
                };
                result.push(old);
            }
            "clear()V" => {
                let version = self
                    .map(owner)?
                    .1
                    .checked_add(1)
                    .context("map version exhausted")?;
                self.heap.get_mut(owner)?.data = Data::Map {
                    entries: vec![],
                    version,
                };
            }
            "equals(Ljava/lang/Object;)Z" | "hashCode()I" | "toString()Ljava/lang/String;" => {
                bail!("unsupported collection method {}", method.key())
            }
            _ => return Ok(None),
        }
        Ok(Some(result))
    }
}
