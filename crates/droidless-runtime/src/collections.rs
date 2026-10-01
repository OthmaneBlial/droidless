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
const TREE_SET: &str = "Ljava/util/TreeSet;";
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
        let identity = !self.is_a(&self.heap.get(owner)?.class, "Ljava/util/List;");
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
    fn tree_compare(&mut self, owner: Word, left: Word, right: Word) -> Result<std::cmp::Ordering> {
        let comparator = self
            .heap
            .get(owner)?
            .fields
            .get("droidless:tree-set:comparator")
            .and_then(|values| values.first())
            .copied()
            .unwrap_or(Word::ZERO);
        let roots = self.native_roots.len();
        self.native_roots.push(owner);
        let result = self.compare_values(comparator, left, right);
        self.native_roots.truncate(roots);
        result
    }
    fn compare_values(
        &mut self,
        comparator: Word,
        left: Word,
        right: Word,
    ) -> Result<std::cmp::Ordering> {
        let roots = self.native_roots.len();
        self.native_roots.extend([comparator, left, right]);
        let comparison = (|| -> Result<i32> {
            let result = if comparator == Word::ZERO {
                if !self.is_a(&self.heap.get(left)?.class, "Ljava/lang/Comparable;") {
                    return Err(fault(
                        "Ljava/lang/ClassCastException;",
                        "element does not implement Comparable",
                    ));
                }
                self.invoke(
                    Method {
                        class: "Ljava/lang/Comparable;".into(),
                        name: "compareTo".into(),
                        parameters: vec!["Ljava/lang/Object;".into()],
                        returns: "I".into(),
                    },
                    vec![left, right],
                    true,
                )?
            } else {
                self.invoke(
                    Method {
                        class: "Ljava/util/Comparator;".into(),
                        name: "compare".into(),
                        parameters: vec!["Ljava/lang/Object;".into(); 2],
                        returns: "I".into(),
                    },
                    vec![comparator, left, right],
                    true,
                )?
            };
            ensure!(
                result.len() == 1,
                "comparison returned invalid result width"
            );
            result
                .first()
                .context("comparison returned no value")?
                .int()
        })();
        self.native_roots.truncate(roots);
        Ok(match comparison? {
            value if value < 0 => std::cmp::Ordering::Less,
            0 => std::cmp::Ordering::Equal,
            _ => std::cmp::Ordering::Greater,
        })
    }
    fn sort_values(
        &mut self,
        source: &mut [Word],
        comparator: Word,
        after_compare: impl Fn(&Self) -> Result<()>,
    ) -> Result<()> {
        if comparator != Word::ZERO {
            ensure!(
                self.is_a(&self.heap.get(comparator)?.class, "Ljava/util/Comparator;"),
                "sort requires Comparator"
            );
        }
        let mut target = vec![Word::ZERO; source.len()];
        let mut width = 1;
        while width < source.len() {
            for start in (0..source.len()).step_by(width * 2) {
                let middle = (start + width).min(source.len());
                let end = (start + width * 2).min(source.len());
                let (mut left, mut right, mut output) = (start, middle, start);
                while left < middle && right < end {
                    let order = self.compare_values(comparator, source[left], source[right])?;
                    after_compare(self)?;
                    if order.is_le() {
                        target[output] = source[left];
                        left += 1;
                    } else {
                        target[output] = source[right];
                        right += 1;
                    }
                    output += 1;
                }
                while left < middle {
                    target[output] = source[left];
                    left += 1;
                    output += 1;
                }
                while right < end {
                    target[output] = source[right];
                    right += 1;
                    output += 1;
                }
            }
            source.copy_from_slice(&target);
            width *= 2;
        }
        Ok(())
    }
    fn tree_find(&mut self, owner: Word, value: Word) -> Result<Option<usize>> {
        for (index, existing) in self.collection(owner)?.0.to_vec().into_iter().enumerate() {
            match self.tree_compare(owner, value, existing)? {
                std::cmp::Ordering::Less => return Ok(None),
                std::cmp::Ordering::Equal => return Ok(Some(index)),
                std::cmp::Ordering::Greater => {}
            }
        }
        Ok(None)
    }
    pub(crate) fn collection_native(
        &mut self,
        method: &Method,
        args: &[Word],
    ) -> Result<Option<Vec<Word>>> {
        let signature = method.signature();
        if method.class == "Ldroidless/runtime/map/Entry;" {
            let receiver = *args.first().context("Map.Entry receiver missing")?;
            let value = match signature.as_str() {
                "getKey()Ljava/lang/Object;" => self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:map-entry:key")
                    .and_then(|values| values.first())
                    .copied(),
                "getValue()Ljava/lang/Object;" => self
                    .heap
                    .get(receiver)?
                    .fields
                    .get("droidless:map-entry:value")
                    .and_then(|values| values.first())
                    .copied(),
                _ => return Ok(None),
            }
            .context("Map.Entry has no value")?;
            return Ok(Some(vec![value]));
        }
        if [
            "Landroid/util/SparseArray;",
            "Landroid/util/SparseIntArray;",
        ]
        .contains(&method.class.as_str())
        {
            let receiver = *args.first().context("SparseArray receiver missing")?;
            let arg = |index| {
                args.get(index)
                    .copied()
                    .context("SparseArray argument missing")
            };
            let result = match signature.as_str() {
                "<init>()V" | "<init>(I)V" => {
                    if signature == "<init>(I)V" && arg(1)?.int()? < 0 {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "negative SparseArray capacity",
                        ));
                    }
                    self.heap.get_mut(receiver)?.data = Data::SparseArray(Default::default());
                    vec![]
                }
                "put(ILjava/lang/Object;)V"
                | "append(ILjava/lang/Object;)V"
                | "put(II)V"
                | "append(II)V" => {
                    let key = arg(1)?.int()?;
                    let value = arg(2)?;
                    let Data::SparseArray(values) = &mut self.heap.get_mut(receiver)?.data else {
                        bail!("uninitialized SparseArray");
                    };
                    values.insert(key, value);
                    vec![]
                }
                "get(I)Ljava/lang/Object;"
                | "get(ILjava/lang/Object;)Ljava/lang/Object;"
                | "get(I)I"
                | "get(II)I" => {
                    let key = arg(1)?.int()?;
                    let Data::SparseArray(values) = &self.heap.get(receiver)?.data else {
                        bail!("uninitialized SparseArray");
                    };
                    vec![values.get(&key).copied().unwrap_or(if args.len() == 3 {
                        arg(2)?
                    } else {
                        Word::ZERO
                    })]
                }
                "indexOfKey(I)I" => {
                    let key = arg(1)?.int()?;
                    let Data::SparseArray(values) = &self.heap.get(receiver)?.data else {
                        bail!("uninitialized SparseArray");
                    };
                    // ponytail: O(n) rank in the existing ordered map; use indexed sparse storage if large-array lookups become costly.
                    let index = values.range(..key).count() as i32;
                    vec![Word::from(if values.contains_key(&key) {
                        index
                    } else {
                        !index
                    })]
                }
                "size()I" => {
                    let Data::SparseArray(values) = &self.heap.get(receiver)?.data else {
                        bail!("uninitialized SparseArray");
                    };
                    vec![Word::from(values.len() as i32)]
                }
                "keyAt(I)I" | "valueAt(I)Ljava/lang/Object;" => {
                    let index = usize::try_from(arg(1)?.int()?).map_err(|_| {
                        fault(
                            "Ljava/lang/ArrayIndexOutOfBoundsException;",
                            "SparseArray index out of bounds",
                        )
                    })?;
                    let Data::SparseArray(values) = &self.heap.get(receiver)?.data else {
                        bail!("uninitialized SparseArray");
                    };
                    let (key, value) = values.iter().nth(index).ok_or_else(|| {
                        fault(
                            "Ljava/lang/ArrayIndexOutOfBoundsException;",
                            "SparseArray index out of bounds",
                        )
                    })?;
                    if signature == "keyAt(I)I" {
                        vec![Word::from(*key)]
                    } else {
                        vec![*value]
                    }
                }
                "clear()V" => {
                    let Data::SparseArray(values) = &mut self.heap.get_mut(receiver)?.data else {
                        bail!("uninitialized SparseArray");
                    };
                    values.clear();
                    vec![]
                }
                "remove(I)V" | "delete(I)V" => {
                    let key = arg(1)?.int()?;
                    let Data::SparseArray(values) = &mut self.heap.get_mut(receiver)?.data else {
                        bail!("uninitialized SparseArray");
                    };
                    values.remove(&key);
                    vec![]
                }
                _ => return Ok(None),
            };
            return Ok(Some(result));
        }
        if method.class == "Ljava/util/Arrays;"
            && matches!(
                signature.as_str(),
                "binarySearch([Ljava/lang/Object;Ljava/lang/Object;)I"
                    | "binarySearch([Ljava/lang/Object;IILjava/lang/Object;)I"
            )
        {
            let array = *args.first().context("Arrays.binarySearch array missing")?;
            if array == Word::ZERO {
                return Err(fault("Ljava/lang/NullPointerException;", "array is null"));
            }
            let values = match &self.heap.get(array)?.data {
                Data::Array { values, element } if element.starts_with(['L', '[']) => values
                    .iter()
                    .map(|value| value.first().copied().unwrap_or(Word::ZERO))
                    .collect::<Vec<_>>(),
                _ => bail!("Arrays.binarySearch requires an object array"),
            };
            let (from, to, key) = if signature
                == "binarySearch([Ljava/lang/Object;Ljava/lang/Object;)I"
            {
                (
                    0,
                    values.len(),
                    args.get(1).copied().context("search key missing")?,
                )
            } else {
                let from =
                    usize::try_from(args.get(1).copied().context("range start missing")?.int()?)
                        .map_err(|_| {
                            fault(
                                "Ljava/lang/ArrayIndexOutOfBoundsException;",
                                "negative binary search range start",
                            )
                        })?;
                let to = usize::try_from(args.get(2).copied().context("range end missing")?.int()?)
                    .map_err(|_| {
                        fault(
                            "Ljava/lang/ArrayIndexOutOfBoundsException;",
                            "negative binary search range end",
                        )
                    })?;
                (
                    from,
                    to,
                    args.get(3).copied().context("search key missing")?,
                )
            };
            if from > to {
                return Err(fault(
                    "Ljava/lang/IllegalArgumentException;",
                    "binary search range start exceeds end",
                ));
            }
            if to > values.len() {
                return Err(fault(
                    "Ljava/lang/ArrayIndexOutOfBoundsException;",
                    "binary search range exceeds array length",
                ));
            }
            let mut low = from as isize;
            let mut high = to as isize - 1;
            while low <= high {
                let middle = ((low + high) >> 1) as usize;
                let value = values[middle];
                if value == Word::ZERO || key == Word::ZERO {
                    return Err(fault(
                        "Ljava/lang/NullPointerException;",
                        "binary search compared a null value",
                    ));
                }
                let class = self.heap.get(value)?.class.clone();
                let comparison = if class == "Ljava/lang/String;" {
                    let left = self.heap.text(value)?.encode_utf16().collect::<Vec<_>>();
                    let right = self.heap.text(key)?.encode_utf16().collect::<Vec<_>>();
                    left.iter()
                        .zip(&right)
                        .find_map(|(left, right)| {
                            (left != right).then_some(i32::from(*left) - i32::from(*right))
                        })
                        .unwrap_or_else(|| left.len() as i32 - right.len() as i32)
                } else {
                    if !self.is_a(&class, "Ljava/lang/Comparable;") {
                        return Err(fault(
                            "Ljava/lang/ClassCastException;",
                            format!("{} does not implement Comparable", class),
                        ));
                    }
                    self.invoke(
                        Method {
                            class: "Ljava/lang/Comparable;".into(),
                            name: "compareTo".into(),
                            parameters: vec!["Ljava/lang/Object;".into()],
                            returns: "I".into(),
                        },
                        vec![value, key],
                        true,
                    )?
                    .first()
                    .context("Comparable.compareTo returned no value")?
                    .int()?
                };
                if comparison < 0 {
                    low = middle as isize + 1;
                } else if comparison > 0 {
                    high = middle as isize - 1;
                } else {
                    return Ok(Some(vec![Word::from(middle as i32)]));
                }
            }
            return Ok(Some(vec![Word::from(-(low as i32) - 1)]));
        }
        if method.class == "Ljava/util/Arrays;"
            && signature == "asList([Ljava/lang/Object;)Ljava/util/List;"
        {
            let array = *args.first().context("Arrays.asList array missing")?;
            let values = {
                let Data::Array { values, .. } = &self.heap.get(array)?.data else {
                    bail!("Arrays.asList requires an object array");
                };
                ensure!(
                    values.len() <= LIMIT,
                    "collection entry limit reached ({LIMIT})"
                );
                values
                    .iter()
                    .map(|value| value.first().copied().unwrap_or(Word::ZERO))
                    .collect()
            };
            let list = self.heap.instance("Ljava/util/ArrayList;")?;
            self.heap.get_mut(list)?.data = Data::Collection { values, version: 0 };
            return Ok(Some(vec![list]));
        }
        if [
            "Ljava/util/Collection;",
            "Ljava/util/List;",
            "Ljava/util/Set;",
        ]
        .contains(&method.class.as_str())
        {
            let owner = *args.first().context("collection receiver missing")?;
            let class = self.heap.get(owner)?.class.clone();
            if class != method.class && self.is_a(&class, &method.class) {
                return self.collection_native(
                    &Method {
                        class,
                        ..method.clone()
                    },
                    args,
                );
            }
        }
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
        if method.class == "Ljava/util/Arrays;"
            && [
                "sort([Ljava/lang/Object;)V",
                "sort([Ljava/lang/Object;Ljava/util/Comparator;)V",
                "sort([Ljava/lang/Object;II)V",
                "sort([Ljava/lang/Object;IILjava/util/Comparator;)V",
            ]
            .contains(&signature.as_str())
        {
            let array = *args.first().context("sort array missing")?;
            let object = self.heap.get(array)?;
            ensure!(
                object.class.starts_with("[L") || object.class.starts_with("[["),
                "sort requires reference array"
            );
            ensure!(
                args.len() == method.parameters.len(),
                "invalid array sort arguments"
            );
            let Data::Array { values, .. } = &object.data else {
                bail!("sort requires array data");
            };
            let original = values.clone();
            ensure!(original.len() <= 16_384, "array sort limit reached (16384)");
            for value in &original {
                ensure!(value.len() == 1, "invalid reference array element width");
                value[0].reference()?;
            }
            let ranged = method.parameters.len() >= 3;
            let (start, end) = if ranged {
                (args[1].int()?, args[2].int()?)
            } else {
                (0, original.len() as i32)
            };
            if start > end {
                return Err(fault(
                    "Ljava/lang/IllegalArgumentException;",
                    "sort range is reversed",
                ));
            }
            if start < 0 || end as usize > original.len() {
                return Err(fault(
                    "Ljava/lang/ArrayIndexOutOfBoundsException;",
                    "sort range exceeds array",
                ));
            }
            let comparator = if method
                .parameters
                .last()
                .is_some_and(|ty| ty == "Ljava/util/Comparator;")
            {
                *args.last().context("sort comparator missing")?
            } else {
                Word::ZERO
            };
            let roots = self.native_roots.len();
            self.native_roots.extend([array, comparator]);
            self.native_roots.extend(original.iter().flatten().copied());
            let sorted = (|| -> Result<()> {
                let mut values = original[start as usize..end as usize]
                    .iter()
                    .map(|words| words[0])
                    .collect::<Vec<_>>();
                self.sort_values(&mut values, comparator, |_| Ok(()))?;
                // Guest mutation during comparison is unsupported; preserve it rather than overwrite it.
                let Data::Array { values: target, .. } = &mut self.heap.get_mut(array)?.data else {
                    bail!("array changed type while sorting");
                };
                ensure!(*target == original, "array changed while sorting");
                for (slot, value) in target[start as usize..end as usize].iter_mut().zip(values) {
                    slot[0] = value;
                }
                Ok(())
            })();
            self.native_roots.truncate(roots);
            sorted?;
            return Ok(Some(vec![]));
        }
        if method.class == "Ljava/util/Collections;" {
            if signature == "reverse(Ljava/util/List;)V" {
                let list = *args.first().context("reverse list missing")?;
                ensure!(
                    self.is_a(&self.heap.get(list)?.class, "Ljava/util/List;"),
                    "Collections.reverse requires a List"
                );
                let size = self.invoke(
                    Method {
                        class: "Ljava/util/List;".into(),
                        name: "size".into(),
                        parameters: vec![],
                        returns: "I".into(),
                    },
                    vec![list],
                    true,
                )?;
                let size = size.first().context("List.size returned no value")?.int()?;
                ensure!(
                    (0..=LIMIT as i32).contains(&size),
                    "reverse list size exceeds collection limit"
                );
                let roots = self.native_roots.len();
                self.native_roots.push(list);
                let reversed = (|| -> Result<()> {
                    for first in 0..size / 2 {
                        let value = self.invoke(
                            Method {
                                class: "Ljava/util/List;".into(),
                                name: "get".into(),
                                parameters: vec!["I".into()],
                                returns: "Ljava/lang/Object;".into(),
                            },
                            vec![list, Word::from(first)],
                            true,
                        )?;
                        let value = *value.first().context("List.get returned no value")?;
                        let set = Method {
                            class: "Ljava/util/List;".into(),
                            name: "set".into(),
                            parameters: vec!["I".into(), "Ljava/lang/Object;".into()],
                            returns: "Ljava/lang/Object;".into(),
                        };
                        let previous = self.invoke(
                            set.clone(),
                            vec![list, Word::from(size - 1 - first), value],
                            true,
                        )?;
                        let previous = *previous.first().context("List.set returned no value")?;
                        self.invoke(set, vec![list, Word::from(first), previous], true)?;
                    }
                    Ok(())
                })();
                self.native_roots.truncate(roots);
                reversed?;
                return Ok(Some(vec![]));
            }
            if signature == "sort(Ljava/util/List;Ljava/util/Comparator;)V"
                || signature == "sort(Ljava/util/List;)V"
            {
                let list = *args.first().context("sort list missing")?;
                let comparator = args.get(1).copied().unwrap_or(Word::ZERO);
                ensure!(
                    self.is_a(&self.heap.get(list)?.class, "Ljava/util/List;"),
                    "Collections.sort requires a List"
                );
                let (values, version) = self.collection(list)?;
                let mut source = values.to_vec();
                let roots = self.native_roots.len();
                self.native_roots.extend([list, comparator]);
                self.native_roots.extend(source.iter().copied());
                let sorted = self
                    .sort_values(&mut source, comparator, |vm| {
                        ensure!(
                            vm.collection(list)?.1 == version,
                            fault(
                                "Ljava/util/ConcurrentModificationException;",
                                "list changed while sorting"
                            )
                        );
                        Ok(())
                    })
                    .and_then(|()| self.change_collection(list, source));
                self.native_roots.truncate(roots);
                sorted?;
                return Ok(Some(vec![]));
            }
            if signature == "addAll(Ljava/util/Collection;[Ljava/lang/Object;)Z" {
                let collection = *args.first().context("addAll collection missing")?;
                ensure!(
                    self.is_a(&self.heap.get(collection)?.class, "Ljava/util/Collection;"),
                    "addAll requires a Collection"
                );
                let array = *args.get(1).context("addAll array missing")?;
                let Data::Array { values, .. } = &self.heap.get(array)?.data else {
                    bail!("addAll requires an Object array");
                };
                let values = values
                    .iter()
                    .map(|value| value.first().copied().unwrap_or(Word::ZERO))
                    .collect::<Vec<_>>();
                let mut modified = false;
                for value in values {
                    modified |= self
                        .invoke(
                            Method {
                                class: "Ljava/util/Collection;".into(),
                                name: "add".into(),
                                parameters: vec!["Ljava/lang/Object;".into()],
                                returns: "Z".into(),
                            },
                            vec![collection, value],
                            true,
                        )?
                        .first()
                        .context("Collection.add returned no result")?
                        .truth();
                }
                return Ok(Some(vec![Word::from(i32::from(modified))]));
            }
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
        if [
            "Ljava/util/HashMap;",
            "Ljava/util/LinkedHashMap;",
            "Ljava/util/WeakHashMap;",
            "Ljava/util/concurrent/ConcurrentHashMap;",
            "Ljava/util/Hashtable;",
        ]
        .contains(&method.class.as_str())
        {
            // ponytail: WeakHashMap keys remain strong until guest garbage collection is modeled.
            let receiver = *args.first().context("map receiver missing")?;
            if self.is_a(&self.heap.get(receiver)?.class, "Ljava/util/Hashtable;") {
                self.enter_monitor(receiver)?;
                let result = self.map_native(method, args);
                self.exit_monitor(receiver)?;
                return result;
            }
            return self.map_native(method, args);
        }
        let cow = method.class == COPY_ON_WRITE_LIST;
        let list = [
            "Ljava/util/ArrayList;",
            "Ljava/util/Vector;",
            "Ljava/util/Stack;",
        ]
        .contains(&method.class.as_str())
            || cow;
        let tree_set = method.class == TREE_SET;
        let snapshot_iterator = method.class == SNAPSHOT_ITERATOR;
        if !list
            && method.class != "Ljava/util/HashSet;"
            && !tree_set
            && method.class != ITERATOR
            && !snapshot_iterator
        {
            return Ok(None);
        }
        let receiver = *args.first().context("collection receiver missing")?;
        let arg = |n| args.get(n).copied().context("collection argument missing");
        let sig = method.signature();
        let mut result = vec![];
        if sig == "addAll(Ljava/util/Collection;)Z" || sig == "addAll(ILjava/util/Collection;)Z" {
            let indexed = sig.starts_with("addAll(I");
            let source = arg(if indexed { 2 } else { 1 })?;
            ensure!(
                self.is_a(&self.heap.get(source)?.class, "Ljava/util/Collection;"),
                "addAll source must be a Collection"
            );
            let values = self.collection(source)?.0.to_vec();
            let mut insertion = if indexed {
                Some(self.list_index(receiver, arg(1)?, true)?)
            } else {
                None
            };
            let mut modified = false;
            for value in values {
                let (name, parameters, returns, forwarded) = if let Some(index) = &mut insertion {
                    let position = *index;
                    *index += 1;
                    (
                        "add",
                        vec!["I".into(), "Ljava/lang/Object;".into()],
                        "V",
                        vec![receiver, Word::from(position as i32), value],
                    )
                } else {
                    (
                        "add",
                        vec!["Ljava/lang/Object;".into()],
                        "Z",
                        vec![receiver, value],
                    )
                };
                let added = self.invoke(
                    Method {
                        class: if indexed {
                            "Ljava/util/List;"
                        } else {
                            "Ljava/util/Collection;"
                        }
                        .into(),
                        name: name.into(),
                        parameters,
                        returns: returns.into(),
                    },
                    forwarded,
                    true,
                )?;
                modified |= indexed
                    || added
                        .first()
                        .context("Collection.add returned no value")?
                        .truth();
            }
            return Ok(Some(vec![Word::from(i32::from(modified))]));
        }
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
            "empty()Z" if method.class == "Ljava/util/Stack;" => {
                result.push(Word::from(i32::from(
                    self.collection(receiver)?.0.is_empty(),
                )));
            }
            "peek()Ljava/lang/Object;" | "pop()Ljava/lang/Object;"
                if method.class == "Ljava/util/Stack;" =>
            {
                let mut values = self.collection(receiver)?.0.to_vec();
                let value = values
                    .last()
                    .copied()
                    .ok_or_else(|| fault("Ljava/util/EmptyStackException;", "Stack is empty"))?;
                if sig == "pop()Ljava/lang/Object;" {
                    values.pop();
                    self.change_collection(receiver, values)?;
                }
                result.push(value);
            }
            "push(Ljava/lang/Object;)Ljava/lang/Object;" if method.class == "Ljava/util/Stack;" => {
                let value = arg(1)?;
                value.reference()?;
                let mut values = self.collection(receiver)?.0.to_vec();
                values.push(value);
                self.change_collection(receiver, values)?;
                result.push(value);
            }
            "search(Ljava/lang/Object;)I" if method.class == "Ljava/util/Stack;" => {
                let index = self.collection_find(receiver, arg(1)?, true)?;
                let depth = match index {
                    Some(index) => (self.collection(receiver)?.0.len() - index) as i32,
                    None => -1,
                };
                result.push(Word::from(depth));
            }
            "<init>(Ljava/util/Collection;)V" if list => {
                let source = arg(1)?;
                ensure!(
                    self.is_a(&self.heap.get(source)?.class, "Ljava/util/Collection;"),
                    "ArrayList source must be a Collection"
                );
                let values = self.collection(source)?.0.to_vec();
                self.heap.get_mut(receiver)?.data = Data::Collection { values, version: 0 };
            }
            "<init>()V" | "<init>(I)V" if !tree_set && (!cow || sig == "<init>()V") => {
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
            "<init>(II)V" if method.class == "Ljava/util/Vector;" => {
                if arg(1)?.int()? < 0 || arg(2)?.int()? < 0 {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "negative Vector capacity or increment",
                    ));
                }
                self.heap.get_mut(receiver)?.data = Data::Collection {
                    values: vec![],
                    version: 0,
                };
            }
            "<init>()V" | "<init>(Ljava/util/Comparator;)V" if tree_set => {
                let comparator = if sig == "<init>()V" {
                    Word::ZERO
                } else {
                    arg(1)?
                };
                self.heap.get_mut(receiver)?.data = Data::Collection {
                    values: vec![],
                    version: 0,
                };
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:tree-set:comparator".into(), vec![comparator]);
            }
            "<init>(Ljava/util/SortedSet;)V" if tree_set => {
                let source = arg(1)?;
                ensure!(
                    self.is_a(&self.heap.get(source)?.class, "Ljava/util/SortedSet;"),
                    "TreeSet source must be a SortedSet"
                );
                let values = self.collection(source)?.0.to_vec();
                let comparator = self
                    .heap
                    .get(source)?
                    .fields
                    .get("droidless:tree-set:comparator")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO);
                self.heap.get_mut(receiver)?.data = Data::Collection { values, version: 0 };
                self.heap
                    .get_mut(receiver)?
                    .fields
                    .insert("droidless:tree-set:comparator".into(), vec![comparator]);
            }
            "comparator()Ljava/util/Comparator;" if tree_set => result.push(
                self.heap
                    .get(receiver)?
                    .fields
                    .get("droidless:tree-set:comparator")
                    .and_then(|values| values.first())
                    .copied()
                    .unwrap_or(Word::ZERO),
            ),
            "first()Ljava/lang/Object;" | "last()Ljava/lang/Object;" if tree_set => {
                let values = self.collection(receiver)?.0;
                let value = if sig == "first()Ljava/lang/Object;" {
                    values.first()
                } else {
                    values.last()
                }
                .copied()
                .ok_or_else(|| fault("Ljava/util/NoSuchElementException;", "TreeSet is empty"))?;
                result.push(value);
            }
            "size()I" => result.push(Word::from(self.collection(receiver)?.0.len() as i32)),
            "isEmpty()Z" => result.push(Word::from(i32::from(
                self.collection(receiver)?.0.is_empty(),
            ))),
            "toArray()[Ljava/lang/Object;" => {
                let values = self.collection(receiver)?.0.to_vec();
                let array = self.array("Ljava/lang/Object;".into(), values.len())?;
                let Data::Array { values: slots, .. } = &mut self.heap.get_mut(array)?.data else {
                    bail!("new object array has invalid storage");
                };
                for (slot, value) in slots.iter_mut().zip(values) {
                    *slot = vec![value];
                }
                result.push(array);
            }
            "toArray([Ljava/lang/Object;)[Ljava/lang/Object;" => {
                let destination = arg(1)?;
                ensure!(
                    destination != Word::ZERO,
                    fault("Ljava/lang/NullPointerException;", "array is null")
                );
                let (element, length) = match &self.heap.get(destination)?.data {
                    Data::Array { element, values }
                        if element.starts_with('L') || element.starts_with('[') =>
                    {
                        (element.clone(), values.len())
                    }
                    _ => {
                        return Err(fault(
                            "Ljava/lang/ArrayStoreException;",
                            "toArray requires a reference array",
                        ));
                    }
                };
                let values = self.collection(receiver)?.0.to_vec();
                let roots = self.native_roots.len();
                self.native_roots.extend([receiver, destination]);
                self.native_roots.extend(values.iter().copied());
                let converted = (|| -> Result<Word> {
                    for value in values.iter().copied().filter(|value| *value != Word::ZERO) {
                        let class = self.heap.get(value)?.class.clone();
                        ensure!(
                            class == element || self.is_a(&class, &element),
                            fault(
                                "Ljava/lang/ArrayStoreException;",
                                "collection element is incompatible with destination array",
                            )
                        );
                    }
                    let array = if length < values.len() {
                        self.array(element.clone(), values.len())?
                    } else {
                        destination
                    };
                    let Data::Array {
                        element: actual_element,
                        values: slots,
                    } = &mut self.heap.get_mut(array)?.data
                    else {
                        bail!("toArray destination lost array storage");
                    };
                    ensure!(
                        *actual_element == element,
                        "toArray destination component type changed"
                    );
                    for (slot, value) in slots.iter_mut().zip(values.iter().copied()) {
                        *slot = vec![value];
                    }
                    if length > values.len() {
                        slots[values.len()] = vec![Word::ZERO];
                    }
                    Ok(array)
                })();
                self.native_roots.truncate(roots);
                result.push(converted?);
            }
            "contains(Ljava/lang/Object;)Z" => {
                let found = if tree_set {
                    self.tree_find(receiver, arg(1)?)?.is_some()
                } else {
                    self.collection_find(receiver, arg(1)?, false)?.is_some()
                };
                result.push(Word::from(i32::from(found)));
            }
            "add(Ljava/lang/Object;)Z" => {
                let value = arg(1)?;
                value.reference()?;
                if tree_set {
                    let mut values = self.collection(receiver)?.0.to_vec();
                    let mut insertion = values.len();
                    for (index, existing) in values.iter().copied().enumerate() {
                        match self.tree_compare(receiver, value, existing)? {
                            std::cmp::Ordering::Less => {
                                insertion = index;
                                break;
                            }
                            std::cmp::Ordering::Equal => {
                                result.push(Word::ZERO);
                                return Ok(Some(result));
                            }
                            std::cmp::Ordering::Greater => {}
                        }
                    }
                    values.insert(insertion, value);
                    self.change_collection(receiver, values)?;
                    result.push(Word::from(1));
                    return Ok(Some(result));
                }
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
                let index = if tree_set {
                    self.tree_find(receiver, arg(1)?)?
                } else {
                    self.collection_find(receiver, arg(1)?, false)?
                };
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
        let Data::Map {
            entries, version, ..
        } = &self.heap.get(owner)?.data
        else {
            bail!("uninitialized map");
        };
        Ok((entries, *version))
    }
    fn map_access(&mut self, owner: Word, index: usize) -> Result<()> {
        let (entries, version, access_order) = match &self.heap.get(owner)?.data {
            Data::Map {
                entries,
                version,
                access_order,
            } => (entries.clone(), *version, *access_order),
            _ => bail!("uninitialized map"),
        };
        if !access_order || index + 1 == entries.len() {
            return Ok(());
        }
        let mut entries = entries;
        let entry = entries.remove(index);
        entries.push(entry);
        let version = version.checked_add(1).context("map version exhausted")?;
        self.heap.get_mut(owner)?.data = Data::Map {
            entries,
            version,
            access_order,
        };
        Ok(())
    }
    fn map_find(&mut self, owner: Word, key: Word) -> Result<Option<usize>> {
        key.reference()?;
        let (entries, version) = self.map(owner)?;
        // ponytail: linear key lookup; use guest hash buckets when large maps are profiled.
        let entries = entries.to_vec();
        for (index, (candidate, _)) in entries.into_iter().enumerate() {
            let equal = if self.is_a(&self.heap.get(owner)?.class, "Ljava/util/Hashtable;") {
                self.object_equal(candidate, key, true)?
            } else {
                self.object_equal(key, candidate, true)?
            };
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
            class == "Ljava/util/WeakHashMap;"
                || self.is_a(class, "Ljava/util/Hashtable;")
                || self.is_a(&self.heap.get(owner)?.class, "Ljava/util/HashMap;"),
            "invalid native map receiver"
        );
        let arg = |n| args.get(n).copied().context("map argument missing");
        let sig = method.signature();
        if self.is_a(class, "Ljava/util/Hashtable;") {
            if sig == "<init>(IFZ)V" {
                return Ok(None);
            }
            let checked = match sig.as_str() {
                "containsKey(Ljava/lang/Object;)Z"
                | "get(Ljava/lang/Object;)Ljava/lang/Object;"
                | "remove(Ljava/lang/Object;)Ljava/lang/Object;"
                | "containsValue(Ljava/lang/Object;)Z" => 1,
                "put(Ljava/lang/Object;Ljava/lang/Object;)Ljava/lang/Object;" => 2,
                _ => 0,
            };
            for index in 1..=checked {
                ensure!(
                    arg(index)? != Word::ZERO,
                    fault(
                        "Ljava/lang/NullPointerException;",
                        "Hashtable rejects null keys and values"
                    )
                );
            }
        }
        let mut result = vec![];
        match sig.as_str() {
            "<init>()V" | "<init>(I)V" | "<init>(IF)V" | "<init>(IFZ)V" => {
                if sig != "<init>()V" && arg(1)?.int()? < 0 {
                    return Err(fault(
                        "Ljava/lang/IllegalArgumentException;",
                        "negative map capacity",
                    ));
                }
                let has_load_factor = sig == "<init>(IF)V" || sig == "<init>(IFZ)V";
                if has_load_factor {
                    let load_factor = f32::from_bits(arg(2)?.int()? as u32);
                    if !load_factor.is_finite() || load_factor <= 0.0 {
                        return Err(fault(
                            "Ljava/lang/IllegalArgumentException;",
                            "map load factor must be positive and finite",
                        ));
                    }
                }
                let access_order = if sig == "<init>(IFZ)V" {
                    arg(3)?.int()? != 0
                } else {
                    false
                };
                self.heap.get_mut(owner)?.data = Data::Map {
                    entries: vec![],
                    version: 0,
                    access_order,
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
                    let equal = if self.is_a(&self.heap.get(owner)?.class, "Ljava/util/Hashtable;")
                    {
                        self.object_equal(value, arg(1)?, true)?
                    } else {
                        self.object_equal(arg(1)?, value, true)?
                    };
                    if equal {
                        found = true;
                        break;
                    }
                }
                result.push(Word::from(i32::from(found)));
            }
            "get(Ljava/lang/Object;)Ljava/lang/Object;" => {
                let index = self.map_find(owner, arg(1)?)?;
                result.push(match index {
                    Some(i) => {
                        let value = self.map(owner)?.0[i].1;
                        self.map_access(owner, i)?;
                        value
                    }
                    None => Word::ZERO,
                });
            }
            "values()Ljava/util/Collection;" => {
                // ponytail: Map.values is a snapshot; add a live view if callers mutate during iteration.
                let values = self.map(owner)?.0.iter().map(|(_, value)| *value).collect();
                let view = self.heap.instance("Ljava/util/ArrayList;")?;
                self.heap.get_mut(view)?.data = Data::Collection { values, version: 0 };
                result.push(view);
            }
            "keySet()Ljava/util/Set;" => {
                // ponytail: Map.keySet is a snapshot; add a live view if callers mutate during iteration.
                let values = self.map(owner)?.0.iter().map(|(key, _)| *key).collect();
                let view = self.heap.instance("Ljava/util/HashSet;")?;
                self.heap.get_mut(view)?.data = Data::Collection { values, version: 0 };
                result.push(view);
            }
            "entrySet()Ljava/util/Set;" => {
                let entries = self.map(owner)?.0.to_vec();
                let set = self.heap.instance("Ljava/util/HashSet;")?;
                let mut values = Vec::with_capacity(entries.len());
                for (key, value) in entries {
                    let entry = self.heap.instance("Ldroidless/runtime/map/Entry;")?;
                    let fields = &mut self.heap.get_mut(entry)?.fields;
                    fields.insert("droidless:map-entry:key".into(), vec![key]);
                    fields.insert("droidless:map-entry:value".into(), vec![value]);
                    values.push(entry);
                }
                self.heap.get_mut(set)?.data = Data::Collection { values, version: 0 };
                result.push(set);
            }
            "putAll(Ljava/util/Map;)V" => {
                let source = arg(1)?;
                for map in [owner, source] {
                    ensure!(
                        matches!(
                            self.heap.get(map)?.class.as_str(),
                            "Ljava/util/HashMap;"
                                | "Ljava/util/LinkedHashMap;"
                                | "Ljava/util/WeakHashMap;"
                                | "Ljava/util/concurrent/ConcurrentHashMap;"
                                | "Ljava/util/Hashtable;"
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
                let access_order = match &self.heap.get(owner)?.data {
                    Data::Map { access_order, .. } => *access_order,
                    _ => bail!("uninitialized map"),
                };
                let old = if let Some(index) = index {
                    let old = entries[index].1;
                    entries[index].1 = arg(2)?;
                    if access_order && index + 1 != entries.len() {
                        let entry = entries.remove(index);
                        entries.push(entry);
                        version = version.checked_add(1).context("map version exhausted")?;
                    }
                    old
                } else {
                    ensure!(entries.len() < LIMIT, "map entry limit reached ({LIMIT})");
                    entries.push((arg(1)?, arg(2)?));
                    version = version.checked_add(1).context("map version exhausted")?;
                    Word::ZERO
                };
                self.heap.get_mut(owner)?.data = Data::Map {
                    entries,
                    version,
                    access_order,
                };
                result.push(old);
            }
            "remove(Ljava/lang/Object;)Ljava/lang/Object;" => {
                let index = self.map_find(owner, arg(1)?)?;
                let old = if let Some(index) = index {
                    let (entries, version) = self.map(owner)?;
                    let mut entries = entries.to_vec();
                    let old = entries.remove(index).1;
                    let version = version.checked_add(1).context("map version exhausted")?;
                    let access_order = match &self.heap.get(owner)?.data {
                        Data::Map { access_order, .. } => *access_order,
                        _ => bail!("uninitialized map"),
                    };
                    self.heap.get_mut(owner)?.data = Data::Map {
                        entries,
                        version,
                        access_order,
                    };
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
                let access_order = match &self.heap.get(owner)?.data {
                    Data::Map { access_order, .. } => *access_order,
                    _ => bail!("uninitialized map"),
                };
                self.heap.get_mut(owner)?.data = Data::Map {
                    entries: vec![],
                    version,
                    access_order,
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

#[cfg(test)]
mod tests {
    use super::*;
    use droidless_formats::{apk::Apk, dex::Method};

    fn call(
        vm: &mut Runtime,
        class: &str,
        name: &str,
        parameters: &[&str],
        returns: &str,
        args: &[Word],
    ) -> Vec<Word> {
        vm.invoke(
            Method {
                class: class.into(),
                name: name.into(),
                parameters: parameters
                    .iter()
                    .map(|parameter| (*parameter).into())
                    .collect(),
                returns: returns.into(),
            },
            args.to_vec(),
            false,
        )
        .unwrap()
    }

    #[test]
    fn tree_set_sorts_compares_and_iterates_strings() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap(),
        )
        .unwrap();
        let set = vm.new_instance(TREE_SET).unwrap();
        call(&mut vm, TREE_SET, "<init>", &[], "V", &[set]);
        for item in ["z", "a", "m", "a"] {
            let value = vm.heap.string(item.into()).unwrap();
            call(
                &mut vm,
                TREE_SET,
                "add",
                &["Ljava/lang/Object;"],
                "Z",
                &[set, value],
            );
        }
        assert_eq!(
            call(&mut vm, TREE_SET, "size", &[], "I", &[set])[0]
                .int()
                .unwrap(),
            3
        );
        let iterator = call(
            &mut vm,
            TREE_SET,
            "iterator",
            &[],
            "Ljava/util/Iterator;",
            &[set],
        )[0];
        let actual = (0..3)
            .map(|_| {
                let value = call(
                    &mut vm,
                    ITERATOR,
                    "next",
                    &[],
                    "Ljava/lang/Object;",
                    &[iterator],
                )[0];
                vm.heap.text(value).unwrap().to_owned()
            })
            .collect::<Vec<_>>();
        assert_eq!(actual, ["a", "m", "z"]);
        let first = call(
            &mut vm,
            TREE_SET,
            "first",
            &[],
            "Ljava/lang/Object;",
            &[set],
        )[0];
        let last = call(&mut vm, TREE_SET, "last", &[], "Ljava/lang/Object;", &[set])[0];
        assert_eq!(vm.heap.text(first).unwrap(), "a");
        assert_eq!(vm.heap.text(last).unwrap(), "z");
    }

    #[test]
    fn arrays_binary_search_returns_java_index_and_insertion_point() {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap(),
        )
        .unwrap();
        let array = vm.array("Ljava/lang/Object;".into(), 2).unwrap();
        let values = [
            vm.heap.string("br".into()).unwrap(),
            vm.heap.string("div".into()).unwrap(),
        ];
        if let Data::Array { values: slots, .. } = &mut vm.heap.get_mut(array).unwrap().data {
            for (slot, value) in slots.iter_mut().zip(values) {
                *slot = vec![value];
            }
        }
        let found = vm.heap.string("div".into()).unwrap();
        let missing = vm.heap.string("p".into()).unwrap();
        assert_eq!(
            call(
                &mut vm,
                "Ljava/util/Arrays;",
                "binarySearch",
                &["[Ljava/lang/Object;", "Ljava/lang/Object;"],
                "I",
                &[array, found],
            )[0]
            .int()
            .unwrap(),
            1
        );
        assert_eq!(
            call(
                &mut vm,
                "Ljava/util/Arrays;",
                "binarySearch",
                &["[Ljava/lang/Object;", "Ljava/lang/Object;"],
                "I",
                &[array, missing],
            )[0]
            .int()
            .unwrap(),
            -3
        );
    }
}
