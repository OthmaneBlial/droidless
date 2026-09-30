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
    vm.invoke(
        Method {
            class: "Lorg/droidless/collections/MainActivity;".into(),
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
    vm.collect();
    let next = call(&mut vm, "nextRetained", "Ljava/lang/String;")[0];
    assert_eq!(vm.heap.text(next).unwrap(), "iterator keeps its owner");
    let value = call(&mut vm, "retainedValue", "Ljava/lang/String;")[0];
    assert_eq!(vm.heap.text(value).unwrap(), "map keeps key and value");
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
    let map = vm.heap.instance("Ljava/util/HashMap;").unwrap();
    vm.heap.get_mut(map).unwrap().data = Data::Map {
        entries: values.into_iter().map(|key| (key, Word::ZERO)).collect(),
        version: 0,
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
    for object in [set, map] {
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
}
