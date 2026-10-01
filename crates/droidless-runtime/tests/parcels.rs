use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{
    Runtime,
    heap::{Data, Word},
};

fn invoke(
    vm: &mut Runtime,
    class: &str,
    name: &str,
    parameters: &[&str],
    returns: &str,
    args: Vec<Word>,
) -> anyhow::Result<Vec<Word>> {
    vm.invoke(
        Method {
            class: class.into(),
            name: name.into(),
            parameters: parameters.iter().map(|v| (*v).into()).collect(),
            returns: returns.into(),
        },
        args,
        false,
    )
}
fn fixture(vm: &mut Runtime, name: &str, returns: &str) -> Vec<Word> {
    invoke(
        vm,
        "Lorg/droidless/parcels/MainActivity;",
        name,
        &[],
        returns,
        vec![],
    )
    .unwrap()
}
fn parcel(vm: &mut Runtime, bytes: Vec<u8>) -> Word {
    let value = vm.heap.instance("Landroid/os/Parcel;").unwrap();
    vm.heap.get_mut(value).unwrap().data = Data::Parcel {
        bytes,
        position: 0,
        depth: 0,
        read_limit: None,
        recycled: false,
    };
    value
}

#[test]
fn boxed_bundle_and_list_values_round_trip_with_aliases_types_and_gc() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/parcels.apk")).unwrap(),
    )
    .unwrap();
    let class = "Lorg/droidless/parcels/BoxedExtras;";
    assert_eq!(
        invoke(&mut vm, class, "contract", &[], "I", vec![]).unwrap(),
        [Word::from(1)]
    );
    vm.collect();
    let error = invoke(&mut vm, class, "unsupported", &[], "V", vec![]).unwrap_err();
    assert!(format!("{error:#}").contains("Java serialization is unavailable"));
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert_eq!(
        invoke(&mut vm, class, "contract", &[], "I", vec![]).unwrap(),
        [Word::from(1)]
    );
    vm.close().unwrap();
}

#[test]
fn compiled_parcel_callbacks_activity_isolation_gc_bounds_and_cleanup() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/parcels.apk")).unwrap(),
    )
    .unwrap();
    assert_eq!(fixture(&mut vm, "contract", "I"), [Word::from(1)]);
    assert_eq!(fixture(&mut vm, "failedCreator", "I"), [Word::from(1)]);
    vm.launch().unwrap();
    let transient = fixture(&mut vm, "failedWriter", "Ljava/lang/Object;")[0];
    assert_eq!(vm.activity_depth(), 1);
    vm.collect();
    assert!(
        vm.heap.get(transient).is_err(),
        "failed writer retained a native root"
    );
    vm.click_text("Open parcel detail").unwrap();
    assert_eq!(vm.activity_depth(), 2);
    vm.collect();
    assert_eq!(
        vm.snapshot().unwrap().view.text,
        "4 entries · isolated state 70"
    );
    vm.back().unwrap();
    vm.collect();
    assert_eq!(vm.activity_depth(), 1);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Parcel result 70"
    );

    // Reject hostile lengths, magic, value tags, and truncated Bundle payloads.
    for ints in [
        vec![-2],
        vec![4 * 1024 * 1024 + 1],
        vec![4, 0],
        vec![4, 0x4c444e42, 16_385],
        vec![4, 0x4c444e42, 1],
        vec![8, 0x4c444e42, 0, 0],
    ] {
        let p = parcel(
            &mut vm,
            ints.iter().flat_map(|v: &i32| v.to_le_bytes()).collect(),
        );
        assert!(
            invoke(
                &mut vm,
                "Landroid/os/Parcel;",
                "readBundle",
                &[],
                "Landroid/os/Bundle;",
                vec![p]
            )
            .is_err()
        );
        let Data::Parcel {
            depth, read_limit, ..
        } = vm.heap.get(p).unwrap().data
        else {
            panic!();
        };
        assert_eq!(depth, 0);
        assert_eq!(read_limit, None);
    }
    let p = parcel(&mut vm, vec![0; 4 * 1024 * 1024]);
    invoke(
        &mut vm,
        "Landroid/os/Parcel;",
        "setDataPosition",
        &["I"],
        "V",
        vec![p, Word::from(4 * 1024 * 1024)],
    )
    .unwrap();
    assert!(
        invoke(
            &mut vm,
            "Landroid/os/Parcel;",
            "writeInt",
            &["I"],
            "V",
            vec![p, Word::from(1)]
        )
        .is_err()
    );
    assert!(
        invoke(
            &mut vm,
            "Landroid/os/Parcel;",
            "setDataPosition",
            &["I"],
            "V",
            vec![p, Word::from(-1)]
        )
        .is_err()
    );
    invoke(&mut vm, "Landroid/os/Parcel;", "recycle", &[], "V", vec![p]).unwrap();
    assert!(invoke(&mut vm, "Landroid/os/Parcel;", "readInt", &[], "I", vec![p]).is_err());

    let bundle = vm.heap.instance("Landroid/os/Bundle;").unwrap();
    invoke(
        &mut vm,
        "Landroid/os/Bundle;",
        "<init>",
        &[],
        "V",
        vec![bundle],
    )
    .unwrap();
    let key = vm.heap.string("cycle".into()).unwrap();
    invoke(
        &mut vm,
        "Landroid/os/Bundle;",
        "putBundle",
        &["Ljava/lang/String;", "Landroid/os/Bundle;"],
        "V",
        vec![bundle, key, bundle],
    )
    .unwrap();
    let p = parcel(&mut vm, vec![]);
    let error = invoke(
        &mut vm,
        "Landroid/os/Parcel;",
        "writeBundle",
        &["Landroid/os/Bundle;"],
        "V",
        vec![p, bundle],
    )
    .unwrap_err();
    assert!(format!("{error:#}").contains("nesting limit"));
    let Data::Parcel { depth, .. } = vm.heap.get(p).unwrap().data else {
        panic!();
    };
    assert_eq!(depth, 0);
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(bundle).is_err(),
        "failed cyclic write retained its snapshot"
    );
    vm.close().unwrap();
}
