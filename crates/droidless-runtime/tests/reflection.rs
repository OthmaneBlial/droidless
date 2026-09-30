use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

#[test]
fn apk_class_lookup_construction_initialization_and_inherited_fields() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/reflection.apk")).unwrap(),
    )
    .unwrap();
    vm.launch().unwrap();
    assert_eq!(vm.snapshot().unwrap().view.text, "Reflection passed");
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    let name = vm
        .heap
        .string("org.droidless.reflection.ReflectionContract$Thing".into())
        .unwrap();
    let for_name = Method {
        class: "Ljava/lang/Class;".into(),
        name: "forName".into(),
        parameters: vec!["Ljava/lang/String;".into()],
        returns: "Ljava/lang/Class;".into(),
    };
    let first = vm.invoke(for_name.clone(), vec![name], false).unwrap()[0];
    vm.collect();
    let name = vm
        .heap
        .string("org.droidless.reflection.ReflectionContract$Thing".into())
        .unwrap();
    assert_eq!(
        vm.invoke(for_name.clone(), vec![name], false).unwrap()[0],
        first
    );
    // Malformed nested array names must never recurse with host stack depth.
    for text in [
        format!("{}I", "[".repeat(256)),
        format!(
            "{}java.lang.String{}",
            "[L".repeat(10_000),
            ";".repeat(10_000)
        ),
    ] {
        let name = vm.heap.string(text).unwrap();
        assert!(
            format!(
                "{:#}",
                vm.invoke(for_name.clone(), vec![name], false).unwrap_err()
            )
            .contains("ClassNotFoundException")
        );
    }
    let null = vm.invoke(for_name, vec![Word::ZERO], false).unwrap_err();
    assert!(format!("{null:#}").contains("NullPointerException"));
    let text = Method {
        class: "Ljava/lang/Object;".into(),
        name: "toString".into(),
        parameters: vec![],
        returns: "Ljava/lang/String;".into(),
    };
    assert!(
        format!("{:#}", vm.invoke(text, vec![first], true).unwrap_err())
            .contains("unsupported Class method")
    );
}

#[test]
fn inherited_field_resolution_rejects_missing_and_wrong_kind_members() {
    for (name, error) in [
        ("missing", "NoSuchFieldError"),
        ("shared", "IncompatibleClassChangeError"),
    ] {
        let apk = Apk::parse(include_bytes!("../../../fixtures/generated/reflection.apk")).unwrap();
        let mut vm = Runtime::new(apk).unwrap();
        let child = "Lorg/droidless/reflection/ReflectionContract$Child;";
        let alias = vm.apk.dex[0]
            .fields
            .iter_mut()
            .find(|field| field.class == child && field.name == "number")
            .expect("compiled subclass field reference");
        alias.name = name.into();
        let object = vm.heap.instance(child).unwrap();
        let read = Method {
            class: "Lorg/droidless/reflection/ReflectionContract;".into(),
            name: "readAlias".into(),
            parameters: vec![child.into()],
            returns: "I".into(),
        };
        assert!(format!("{:#}", vm.invoke(read, vec![object], false).unwrap_err()).contains(error));
        assert_eq!(vm.stack_depth(), 0);
    }
}
