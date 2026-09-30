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

#[test]
fn primitive_type_fields_keep_identity_and_reject_wrong_kind_type_and_writes() {
    let read = Method {
        class: "Lorg/droidless/reflection/PrimitiveContract;".into(),
        name: "readInt".into(),
        parameters: vec![],
        returns: "Ljava/lang/Class;".into(),
    };
    for (opcode, ty, diagnostic) in [
        (0x69, "Ljava/lang/Class;", "IllegalAccessError"),
        (0x54, "Ljava/lang/Class;", "IncompatibleClassChangeError"),
        (0x62, "I", "NoSuchFieldError"),
    ] {
        let mut vm = Runtime::new(
            Apk::parse(include_bytes!("../../../fixtures/generated/reflection.apk")).unwrap(),
        )
        .unwrap();
        let first = vm.invoke(read.clone(), vec![], false).unwrap()[0];
        vm.collect();
        assert_eq!(vm.invoke(read.clone(), vec![], false).unwrap(), vec![first]);
        let field = vm.apk.dex[0]
            .fields
            .iter()
            .position(|f| f.class == "Ljava/lang/Integer;" && f.name == "TYPE")
            .unwrap();
        let class = vm.apk.dex[0]
            .classes
            .iter()
            .position(|c| c.name == read.class)
            .unwrap();
        let method = vm.apk.dex[0].classes[class]
            .methods
            .iter()
            .position(|m| vm.apk.dex[0].methods[m.index].name == "readInt")
            .unwrap();
        let original_code = vm.apk.dex[0].classes[class].methods[method]
            .code
            .clone()
            .unwrap();
        let mut code = original_code.clone();
        let access = code
            .instructions
            .windows(2)
            .position(|words| words[0] & 0xff == 0x62 && usize::from(words[1]) == field)
            .expect("compiled TYPE sget-object");
        code.instructions[access] = (code.instructions[access] & 0xff00) | opcode;
        vm.apk.dex[0].classes[class].methods[method].code = Some(code);
        vm.apk.dex[0].fields[field].ty = ty.into();
        let error = vm.invoke(read.clone(), vec![], false).unwrap_err();
        assert!(format!("{error:#}").contains(diagnostic));
        assert_eq!(vm.stack_depth(), 0);
        vm.collect();
        vm.apk.dex[0].classes[class].methods[method].code = Some(original_code);
        vm.apk.dex[0].fields[field].ty = "Ljava/lang/Class;".into();
        assert_eq!(
            vm.invoke(read.clone(), vec![], false).unwrap(),
            vec![first],
            "native final TYPE changed after rejected access"
        );
    }
}
