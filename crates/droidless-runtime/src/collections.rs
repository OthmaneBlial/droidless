//! Bounded Java collections; elements remain guest references and use guest equals.
use crate::{
    heap::{Data, Word, fault},
    vm::Runtime,
};
use anyhow::{Context, Result, bail, ensure};
use droidless_formats::dex::Method;

const ITERATOR: &str = "Ldroidless/runtime/SetIterator;";
const READ_ONLY_SET: &str = "Ldroidless/runtime/UnmodifiableSet;";
const LIMIT: usize = 16_384;

impl Runtime {
    fn object_equal(&mut self, key: Word, value: Word) -> Result<bool> {
        key.reference()?;
        value.reference()?;
        if key == value {
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
    fn collection(&self, owner: Word) -> Result<(&[Word], u32)> {
        let Data::Collection { values, version } = &self.heap.get(owner)?.data else {
            bail!("uninitialized collection");
        };
        Ok((values, *version))
    }
    fn change_collection(&mut self, owner: Word, values: Vec<Word>) -> Result<()> {
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
    fn collection_find(&mut self, owner: Word, key: Word) -> Result<Option<usize>> {
        key.reference()?;
        let (values, version) = self.collection(owner)?;
        // ponytail: linear membership with guest equals; use hash buckets if profiling justifies them.
        let values = values.to_vec();
        for (index, value) in values.into_iter().enumerate() {
            let equal = self.object_equal(key, value)?;
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
    pub(crate) fn collection_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        if method.class == "Ljava/util/Collections;"
            && method.signature() == "unmodifiableSet(Ljava/util/Set;)Ljava/util/Set;"
        {
            let owner = *args.first().context("set argument missing")?;
            ensure!(
                self.is_a(&self.heap.get(owner)?.class, "Ljava/util/Set;"),
                "unmodifiableSet requires a Set"
            );
            let wrapper = self.heap.instance(READ_ONLY_SET)?;
            self.heap
                .get_mut(wrapper)?
                .fields
                .insert("owner".into(), vec![owner]);
            return Ok(Some(vec![wrapper]));
        }
        if method.class == READ_ONLY_SET {
            let wrapper = *args.first().context("set receiver missing")?;
            let sig = method.signature();
            if [
                "add(Ljava/lang/Object;)Z",
                "remove(Ljava/lang/Object;)Z",
                "clear()V",
                "addAll(Ljava/util/Collection;)Z",
                "removeAll(Ljava/util/Collection;)Z",
                "retainAll(Ljava/util/Collection;)Z",
            ]
            .contains(&sig.as_str())
            {
                return Err(fault(
                    "Ljava/lang/UnsupportedOperationException;",
                    "unmodifiable set",
                ));
            }
            if ![
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
            {
                return Ok(None);
            }
            let owner = self
                .heap
                .get(wrapper)?
                .fields
                .get("owner")
                .and_then(|v| v.first())
                .copied()
                .context("uninitialized set view")?;
            let mut forwarded = args.to_vec();
            forwarded[0] = owner;
            let result = self.invoke(
                Method {
                    class: "Ljava/util/Set;".into(),
                    ..method.clone()
                },
                forwarded,
                true,
            )?;
            if sig == "iterator()Ljava/util/Iterator;" {
                let iterator = *result.first().context("iterator returned no value")?;
                ensure!(
                    self.heap.get(iterator)?.class == ITERATOR,
                    "unmodifiable iterator for this Set is unsupported"
                );
                self.heap
                    .get_mut(iterator)?
                    .fields
                    .insert("readOnly".into(), vec![Word::from(1)]);
            }
            return Ok(Some(result));
        }
        if method.class == "Ljava/util/HashMap;" {
            return self.map_native(method, args);
        }
        if method.class != "Ljava/util/HashSet;" && method.class != ITERATOR {
            return Ok(None);
        }
        let receiver = *args.first().context("collection receiver missing")?;
        let arg = |n| args.get(n).copied().context("collection argument missing");
        let sig = method.signature();
        let mut result = vec![];
        if method.class == ITERATOR {
            let object = self.heap.get(receiver)?;
            if sig == "remove()V"
                && object
                    .fields
                    .get("readOnly")
                    .and_then(|v| v.first())
                    .is_some_and(|w| w.truth())
            {
                return Err(fault(
                    "Ljava/lang/UnsupportedOperationException;",
                    "unmodifiable iterator",
                ));
            }
            let field = |name: &str| {
                object
                    .fields
                    .get(name)
                    .and_then(|v| v.first())
                    .copied()
                    .context("uninitialized iterator")
            };
            let owner = field("owner")?;
            let position = field("position")?.int()? as usize;
            let expected = field("version")?.int()? as u32;
            let (values, version) = self.collection(owner)?;
            if sig != "hasNext()Z" && expected != version {
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
                    let mut values = values.to_vec();
                    ensure!((last as usize) < values.len(), "invalid iterator position");
                    values.remove(last as usize);
                    self.change_collection(owner, values)?;
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
            self.is_a(&self.heap.get(receiver)?.class, "Ljava/util/HashSet;"),
            "invalid HashSet receiver"
        );
        match sig.as_str() {
            "<init>()V" | "<init>(I)V" => {
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
                self.collection_find(receiver, arg(1)?)?.is_some(),
            ))),
            "add(Ljava/lang/Object;)Z" => {
                let value = arg(1)?;
                let exists = self.collection_find(receiver, value)?.is_some();
                if !exists {
                    let mut values = self.collection(receiver)?.0.to_vec();
                    values.push(value);
                    self.change_collection(receiver, values)?;
                }
                result.push(Word::from(i32::from(!exists)));
            }
            "remove(Ljava/lang/Object;)Z" => {
                let index = self.collection_find(receiver, arg(1)?)?;
                if let Some(index) = index {
                    let mut values = self.collection(receiver)?.0.to_vec();
                    values.remove(index);
                    self.change_collection(receiver, values)?;
                }
                result.push(Word::from(i32::from(index.is_some())));
            }
            "clear()V" => self.change_collection(receiver, vec![])?,
            "iterator()Ljava/util/Iterator;" => {
                let version = self.collection(receiver)?.1;
                let iterator = self.heap.instance(ITERATOR)?;
                let fields = &mut self.heap.get_mut(iterator)?.fields;
                fields.insert("owner".into(), vec![receiver]);
                fields.insert("position".into(), vec![Word::ZERO]);
                fields.insert("last".into(), vec![Word::from(-1)]);
                fields.insert("version".into(), vec![Word::Bits(version)]);
                result.push(iterator);
            }
            // AbstractSet/AbstractCollection override these; do not fall back to Object's behavior.
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
            let equal = self.object_equal(key, candidate)?;
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
                    if self.object_equal(arg(1)?, value)? {
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
