use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

#[test]
fn string_hash_uses_utf16_values_through_virtual_object_dispatch() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
    )
    .unwrap();
    for (text, expected) in [
        ("", 0),
        ("vector", -820_387_517),
        ("Aa", 2112),
        ("BB", 2112),
        ("😀", 1_772_899),
        ("hash overflow", -1_740_889_388),
    ] {
        let first = vm.heap.string(text.into()).unwrap();
        let second = vm.heap.string(text.into()).unwrap();
        assert_ne!(first, second);
        for (class, receiver) in [
            ("Ljava/lang/String;", first),
            ("Ljava/lang/Object;", second),
        ] {
            let hash = vm
                .invoke(
                    Method {
                        class: class.into(),
                        name: "hashCode".into(),
                        parameters: vec![],
                        returns: "I".into(),
                    },
                    vec![receiver],
                    true,
                )
                .unwrap();
            assert_eq!(hash, vec![Word::from(expected)]);
        }
        let identity = vm
            .invoke(
                Method {
                    class: "Ljava/lang/System;".into(),
                    name: "identityHashCode".into(),
                    parameters: vec!["Ljava/lang/Object;".into()],
                    returns: "I".into(),
                },
                vec![first],
                false,
            )
            .unwrap();
        assert_eq!(
            identity,
            vec![Word::from(first.reference().unwrap() as i32)]
        );
    }
}
