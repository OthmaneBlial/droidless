use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{
    Runtime,
    heap::{Data, Word},
};

fn runtime() -> Runtime {
    Runtime::new(
        Apk::parse(include_bytes!(
            "../../../fixtures/generated/collections.apk"
        ))
        .unwrap(),
    )
    .unwrap()
}
fn call(vm: &mut Runtime, name: &str, returns: &str) -> Vec<Word> {
    call_class(vm, "MainActivity", name, returns)
}
fn call_class(vm: &mut Runtime, class: &str, name: &str, returns: &str) -> Vec<Word> {
    vm.invoke(
        Method {
            class: format!("Lorg/droidless/collections/{class};"),
            name: name.into(),
            parameters: vec![],
            returns: returns.into(),
        },
        vec![],
        false,
    )
    .unwrap()
}

#[test]
fn guest_equality_nulls_iteration_live_readonly_views_and_gc() {
    let mut vm = runtime();
    vm.launch().unwrap();
    assert_eq!(vm.snapshot().unwrap().view.text, "Collections passed");
    call(&mut vm, "retain", "V");
    call_class(&mut vm, "ListContract", "retain", "V");
    call_class(&mut vm, "QueueContract", "retain", "V");
    vm.collect();
    let next = call(&mut vm, "nextRetained", "Ljava/lang/String;")[0];
    assert_eq!(vm.heap.text(next).unwrap(), "iterator keeps its owner");
    let value = call(&mut vm, "retainedValue", "Ljava/lang/String;")[0];
    assert_eq!(vm.heap.text(value).unwrap(), "map keeps key and value");
    let next = call_class(
        &mut vm,
        "ListContract",
        "nextRetained",
        "Ljava/lang/String;",
    )[0];
    assert_eq!(vm.heap.text(next).unwrap(), "list iterator keeps its owner");
    let value = call_class(
        &mut vm,
        "ListContract",
        "retainedViewValue",
        "Ljava/lang/String;",
    )[0];
    assert_eq!(
        vm.heap.text(value).unwrap(),
        "list iterator keeps its owner"
    );
    let next = call_class(
        &mut vm,
        "QueueContract",
        "pollRetained",
        "Ljava/lang/String;",
    )[0];
    assert_eq!(vm.heap.text(next).unwrap(), "queue retains its element");
}

#[test]
fn bulk_copy_roots_snapshots_and_releases_them_after_guest_mutation() {
    let mut vm = runtime();
    let key = call_class(
        &mut vm,
        "MapCopyContract",
        "prepareMutation",
        "Ljava/lang/Object;",
    )[0];
    let copy = Method {
        class: "Lorg/droidless/collections/MapCopyContract;".into(),
        name: "copyMutation".into(),
        parameters: vec![],
        returns: "V".into(),
    };
    let error = vm.invoke(copy, vec![], false).unwrap_err();
    assert!(format!("{error:#}").contains("unsupported source Map mutation during putAll"));
    assert_eq!(vm.stack_depth(), 0);
    let value = call_class(
        &mut vm,
        "MapCopyContract",
        "afterMutation",
        "Ljava/lang/String;",
    )[0];
    assert_eq!(vm.heap.text(value).unwrap(), "snapshot stays alive");
    vm.collect();
    assert!(
        vm.heap.get(key).is_err(),
        "temporary native snapshot leaked a root"
    );
}

#[test]
fn snapshots_retain_values_across_gc_and_workers_and_preserve_limits() {
    let mut vm = runtime();
    let original = call_class(&mut vm, "SnapshotContract", "retain", "Ljava/lang/Object;")[0];
    vm.collect();
    assert!(
        vm.heap.get(original).is_err(),
        "snapshot retained the live list"
    );
    let value = call_class(
        &mut vm,
        "SnapshotContract",
        "nextRetained",
        "Ljava/lang/String;",
    )[0];
    assert_eq!(vm.heap.text(value).unwrap(), "snapshot keeps old value");
    call_class(&mut vm, "SnapshotContract", "release", "V");
    vm.collect();
    assert!(
        vm.heap.get(value).is_err(),
        "released snapshot retained an element"
    );
    let old_value = call_class(
        &mut vm,
        "SnapshotContract",
        "prepareWriteMutation",
        "Ljava/lang/Object;",
    )[0];
    let remove = Method {
        class: "Lorg/droidless/collections/SnapshotContract;".into(),
        name: "removeMutation".into(),
        parameters: vec![],
        returns: "V".into(),
    };
    let error = vm.invoke(remove, vec![], false).unwrap_err();
    assert!(
        format!("{error:#}")
            .contains("unsupported CopyOnWriteArrayList mutation during remove equality")
    );
    assert_eq!(vm.stack_depth(), 0);
    assert_eq!(
        call_class(&mut vm, "SnapshotContract", "mutationSize", "I")[0],
        Word::ZERO
    );
    vm.collect();
    assert!(
        vm.heap.get(old_value).is_err(),
        "failed search retained its snapshot"
    );
    call_class(&mut vm, "SnapshotContract", "prepareWorker", "V");
    vm.collect();
    vm.poll_messages().unwrap();
    vm.collect();
    assert_eq!(
        call_class(&mut vm, "SnapshotContract", "verifyWorker", "I")[0],
        Word::from(1)
    );
    let list = vm
        .heap
        .instance("Ljava/util/concurrent/CopyOnWriteArrayList;")
        .unwrap();
    vm.heap.get_mut(list).unwrap().data = Data::Collection {
        values: vec![Word::ZERO; 16_384],
        version: 0,
    };
    for (parameters, args, returns) in [
        (
            vec!["Ljava/lang/Object;".into()],
            vec![list, Word::ZERO],
            "Z",
        ),
        (
            vec!["I".into(), "Ljava/lang/Object;".into()],
            vec![list, Word::from(16_384), Word::ZERO],
            "V",
        ),
    ] {
        let add = Method {
            class: "Ljava/util/List;".into(),
            name: "add".into(),
            parameters,
            returns: returns.into(),
        };
        assert!(format!("{:#}", vm.invoke(add, args, true).unwrap_err()).contains("entry limit"));
        let Data::Collection { values, version } = &vm.heap.get(list).unwrap().data else {
            panic!("list lost")
        };
        assert_eq!(values, &vec![Word::ZERO; 16_384]);
        assert_eq!(*version, 0);
    }
    for (name, parameters, returns, args) in [
        ("<init>", vec!["I".into()], "V", vec![list, Word::from(1)]),
        (
            "<init>",
            vec!["[Ljava/lang/Object;".into()],
            "V",
            vec![list, Word::ZERO],
        ),
        (
            "addIfAbsent",
            vec!["Ljava/lang/Object;".into()],
            "Z",
            vec![list, Word::ZERO],
        ),
    ] {
        let method = Method {
            class: "Ljava/util/concurrent/CopyOnWriteArrayList;".into(),
            name: name.into(),
            parameters,
            returns: returns.into(),
        };
        assert!(
            format!("{:#}", vm.invoke(method, args, false).unwrap_err())
                .contains("unsupported method")
        );
        let Data::Collection { values, version } = &vm.heap.get(list).unwrap().data else {
            panic!("list lost")
        };
        assert_eq!(values.len(), 16_384);
        assert_eq!(*version, 0);
    }
}

#[test]
fn collection_limits_and_unsupported_overrides_fail_without_mutation() {
    let mut vm = runtime();
    let mut values = vec![];
    for _ in 0..16_384 {
        values.push(vm.heap.instance("Ljava/lang/Object;").unwrap());
    }
    let set = vm.heap.instance("Ljava/util/HashSet;").unwrap();
    vm.heap.get_mut(set).unwrap().data = Data::Collection {
        values: values.clone(),
        version: 0,
    };
    let new_key = vm.heap.instance("Ljava/lang/Object;").unwrap();
    let add = Method {
        class: "Ljava/util/HashSet;".into(),
        name: "add".into(),
        parameters: vec!["Ljava/lang/Object;".into()],
        returns: "Z".into(),
    };
    assert!(
        format!(
            "{:#}",
            vm.invoke(add, vec![set, new_key], true).unwrap_err()
        )
        .contains("entry limit")
    );
    let Data::Collection { values: stored, .. } = &vm.heap.get(set).unwrap().data else {
        panic!("collection lost");
    };
    assert_eq!(stored, &values);
    let list = vm.heap.instance("Ljava/util/ArrayList;").unwrap();
    vm.heap.get_mut(list).unwrap().data = Data::Collection {
        values: values.clone(),
        version: 0,
    };
    for (parameters, args, returns) in [
        (vec!["Ljava/lang/Object;".into()], vec![list, new_key], "Z"),
        (
            vec!["I".into(), "Ljava/lang/Object;".into()],
            vec![list, Word::from(16_384), new_key],
            "V",
        ),
    ] {
        let add = Method {
            class: "Ljava/util/List;".into(),
            name: "add".into(),
            parameters,
            returns: returns.into(),
        };
        assert!(format!("{:#}", vm.invoke(add, args, true).unwrap_err()).contains("entry limit"));
        let Data::Collection {
            values: stored,
            version,
        } = &vm.heap.get(list).unwrap().data
        else {
            panic!("list lost");
        };
        assert_eq!(stored, &values);
        assert_eq!(*version, 0);
    }
    let map = vm.heap.instance("Ljava/util/HashMap;").unwrap();
    vm.heap.get_mut(map).unwrap().data = Data::Map {
        entries: values.into_iter().map(|key| (key, Word::ZERO)).collect(),
        version: 0,
        access_order: false,
    };
    let put = Method {
        class: "Ljava/util/HashMap;".into(),
        name: "put".into(),
        parameters: vec!["Ljava/lang/Object;".into(), "Ljava/lang/Object;".into()],
        returns: "Ljava/lang/Object;".into(),
    };
    assert!(
        format!(
            "{:#}",
            vm.invoke(put, vec![map, new_key, Word::ZERO], true)
                .unwrap_err()
        )
        .contains("entry limit")
    );
    let Data::Map { entries, .. } = &vm.heap.get(map).unwrap().data else {
        panic!("map lost");
    };
    assert_eq!(entries.len(), 16_384);
    assert!(
        entries
            .iter()
            .all(|(key, value)| *key != new_key && *value == Word::ZERO)
    );
    // Bulk copies retain earlier writes if a later entry hits the native map limit.
    let existing = entries[0].0;
    let source = vm.heap.instance("Ljava/util/LinkedHashMap;").unwrap();
    vm.heap.get_mut(source).unwrap().data = Data::Map {
        entries: vec![(existing, new_key), (new_key, Word::ZERO)],
        version: 0,
        access_order: false,
    };
    let copy = Method {
        class: "Ljava/util/Map;".into(),
        name: "putAll".into(),
        parameters: vec!["Ljava/util/Map;".into()],
        returns: "V".into(),
    };
    assert!(
        format!(
            "{:#}",
            vm.invoke(copy, vec![map, source], true).unwrap_err()
        )
        .contains("entry limit")
    );
    let Data::Map {
        entries, version, ..
    } = &vm.heap.get(map).unwrap().data
    else {
        panic!("map lost");
    };
    assert_eq!(entries.len(), 16_384);
    assert_eq!(entries[0], (existing, new_key));
    assert_eq!(*version, 0);
    assert!(entries.iter().all(|(key, _)| *key != new_key));
    for object in [set, list, map] {
        let to_string = Method {
            class: "Ljava/lang/Object;".into(),
            name: "toString".into(),
            parameters: vec![],
            returns: "Ljava/lang/String;".into(),
        };
        assert!(
            format!(
                "{:#}",
                vm.invoke(to_string, vec![object], true).unwrap_err()
            )
            .contains("unsupported collection method")
        );
    }
    let class = "Lorg/droidless/collections/ListContract$EvictingMap;";
    let object = vm.heap.instance(class).unwrap();
    let constructor = Method {
        class: class.into(),
        name: "<init>".into(),
        parameters: vec![],
        returns: "V".into(),
    };
    assert!(
        format!(
            "{:#}",
            vm.invoke(constructor, vec![object], false).unwrap_err()
        )
        .contains("eviction hooks are unsupported")
    );
    vm.collect();
    assert!(
        vm.heap.get(source).is_err(),
        "failed bulk copy retained its source"
    );
    assert!(
        vm.heap.get(map).is_err(),
        "failed bulk copy retained its target"
    );
}

#[test]
fn linked_hash_map_access_order_supports_lru_lookup() {
    let mut vm = runtime();
    let map = vm.heap.instance("Ljava/util/LinkedHashMap;").unwrap();
    let constructor = Method {
        class: "Ljava/util/LinkedHashMap;".into(),
        name: "<init>".into(),
        parameters: vec!["I".into(), "F".into(), "Z".into()],
        returns: "V".into(),
    };
    vm.invoke(
        constructor.clone(),
        vec![
            map,
            Word::from(4),
            Word::Bits(0.75f32.to_bits()),
            Word::from(1),
        ],
        false,
    )
    .unwrap();
    let keys = [
        vm.heap.instance("Ljava/lang/Object;").unwrap(),
        vm.heap.instance("Ljava/lang/Object;").unwrap(),
    ];
    let values = [
        vm.heap.instance("Ljava/lang/Object;").unwrap(),
        vm.heap.instance("Ljava/lang/Object;").unwrap(),
    ];
    let put = Method {
        class: "Ljava/util/Map;".into(),
        name: "put".into(),
        parameters: vec!["Ljava/lang/Object;".into(), "Ljava/lang/Object;".into()],
        returns: "Ljava/lang/Object;".into(),
    };
    for index in 0..2 {
        vm.invoke(put.clone(), vec![map, keys[index], values[index]], true)
            .unwrap();
    }
    let get = Method {
        class: "Ljava/util/Map;".into(),
        name: "get".into(),
        parameters: vec!["Ljava/lang/Object;".into()],
        returns: "Ljava/lang/Object;".into(),
    };
    assert_eq!(
        vm.invoke(get, vec![map, keys[0]], true).unwrap(),
        vec![values[0]]
    );
    let Data::Map {
        entries,
        access_order,
        ..
    } = &vm.heap.get(map).unwrap().data
    else {
        panic!("map lost");
    };
    assert!(*access_order);
    assert_eq!(entries, &[(keys[1], values[1]), (keys[0], values[0])]);

    let invalid = vm.heap.instance("Ljava/util/LinkedHashMap;").unwrap();
    let error = vm
        .invoke(
            constructor,
            vec![
                invalid,
                Word::from(4),
                Word::Bits(0.0f32.to_bits()),
                Word::from(1),
            ],
            false,
        )
        .unwrap_err();
    assert!(format!("{error:#}").contains("map load factor"));
}
