use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{
    Runtime,
    heap::{Data, Word},
};

#[test]
fn reflected_reference_constructors_preserve_access_arguments_gc_and_causes() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/reflection.apk")).unwrap(),
    )
    .unwrap();
    for (name, returns) in [("contract", "I"), ("unsupportedPrimitive", "V")] {
        let result = vm.invoke(
            Method {
                class: "Lorg/droidless/reflection/ConstructorContract;".into(),
                name: name.into(),
                parameters: vec![],
                returns: returns.into(),
            },
            vec![],
            false,
        );
        if name == "contract" {
            assert_eq!(result.unwrap(), [Word::from(1)]);
        } else {
            assert!(
                format!("{:#}", result.unwrap_err())
                    .contains("primitive reflective constructor arguments unsupported")
            );
        }
        assert_eq!(vm.stack_depth(), 0);
        vm.collect();
    }
}

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
    let name = vm
        .heap
        .string("org.droidless.reflection.ReflectionContract$SubConstants".into())
        .unwrap();
    let sub_constants = vm.invoke(for_name.clone(), vec![name], false).unwrap()[0];
    let get_interfaces = Method {
        class: "Ljava/lang/Class;".into(),
        name: "getInterfaces".into(),
        parameters: vec![],
        returns: "[Ljava/lang/Class;".into(),
    };
    let interfaces = vm
        .invoke(get_interfaces.clone(), vec![sub_constants], false)
        .unwrap()[0];
    let Data::Array { values, .. } = &vm.heap.get(interfaces).unwrap().data else {
        panic!("getInterfaces must return a Class[]");
    };
    assert_eq!(values.len(), 1);
    let parent_interface = values[0][0];
    let parent_name = vm
        .invoke(
            Method {
                class: "Ljava/lang/Class;".into(),
                name: "getName".into(),
                parameters: vec![],
                returns: "Ljava/lang/String;".into(),
            },
            vec![parent_interface],
            false,
        )
        .unwrap()[0];
    assert_eq!(
        vm.heap.text(parent_name).unwrap(),
        "org.droidless.reflection.ReflectionContract$Constants"
    );
    assert!(vm.invoke(get_interfaces, vec![first], false).is_ok());
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
fn native_metadata_fields_keep_values_and_reject_wrong_kind_type_and_writes() {
    for (class_name, method_name, field_class, field_name, original_type, sget, sput, iget) in [
        (
            "PrimitiveContract",
            "readInt",
            "Ljava/lang/Integer;",
            "TYPE",
            "Ljava/lang/Class;",
            0x62,
            0x69,
            0x54,
        ),
        (
            "MainActivity",
            "readSdk",
            "Landroid/os/Build$VERSION;",
            "SDK_INT",
            "I",
            0x60,
            0x67,
            0x52,
        ),
    ] {
        let read = Method {
            class: format!("Lorg/droidless/reflection/{class_name};"),
            name: method_name.into(),
            parameters: vec![],
            returns: original_type.into(),
        };
        for (opcode, ty, diagnostic) in [
            (sput, original_type, "IllegalAccessError"),
            (iget, original_type, "IncompatibleClassChangeError"),
            (
                sget,
                if original_type == "I" {
                    "Ljava/lang/Class;"
                } else {
                    "I"
                },
                "NoSuchFieldError",
            ),
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
                .position(|f| f.class == field_class && f.name == field_name)
                .unwrap();
            let class = vm.apk.dex[0]
                .classes
                .iter()
                .position(|c| c.name == read.class)
                .unwrap();
            let method = vm.apk.dex[0].classes[class]
                .methods
                .iter()
                .position(|m| vm.apk.dex[0].methods[m.index].name == method_name)
                .unwrap();
            let original_code = vm.apk.dex[0].classes[class].methods[method]
                .code
                .clone()
                .unwrap();
            let mut code = original_code.clone();
            let access = code
                .instructions
                .windows(2)
                .position(|words| words[0] & 0xff == sget && usize::from(words[1]) == field)
                .expect("compiled native metadata field read");
            code.instructions[access] = (code.instructions[access] & 0xff00) | opcode;
            vm.apk.dex[0].classes[class].methods[method].code = Some(code);
            vm.apk.dex[0].fields[field].ty = ty.into();
            let error = vm.invoke(read.clone(), vec![], false).unwrap_err();
            assert!(format!("{error:#}").contains(diagnostic));
            assert_eq!(vm.stack_depth(), 0);
            vm.collect();
            vm.apk.dex[0].classes[class].methods[method].code = Some(original_code);
            vm.apk.dex[0].fields[field].ty = original_type.into();
            assert_eq!(
                vm.invoke(read.clone(), vec![], false).unwrap(),
                vec![first],
                "native final field changed after rejected access"
            );
        }
    }
    // Exercise a subclass-owned DEX reference even if D8 canonicalizes source aliases.
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/reflection.apk")).unwrap(),
    )
    .unwrap();
    vm.apk.dex[0]
        .fields
        .iter_mut()
        .find(|f| f.class == "Landroid/os/Build$VERSION;" && f.name == "SDK_INT")
        .unwrap()
        .class = "Lorg/droidless/reflection/MainActivity$ApiAlias;".into();
    vm.launch().unwrap();
    assert_eq!(vm.snapshot().unwrap().view.text, "Reflection passed");
    let mut apk = Apk::parse(include_bytes!("../../../fixtures/generated/reflection.apk")).unwrap();
    apk.dex[0]
        .classes
        .iter_mut()
        .find(|c| c.name == "Lorg/droidless/reflection/MainActivity$ApiAlias;")
        .unwrap()
        .name = "Landroid/os/Build$VERSION;".into();
    assert!(
        Runtime::new(apk)
            .err()
            .unwrap()
            .to_string()
            .contains("redefinition of native Build.VERSION")
    );
}
