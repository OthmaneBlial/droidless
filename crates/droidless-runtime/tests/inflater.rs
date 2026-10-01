use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

#[test]
fn compiled_inflater_factories_clones_callbacks_gc_and_fault_recovery() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    vm.launch().unwrap();
    let activity = vm.activity.unwrap();
    let call = |vm: &mut Runtime, name: &str, parameters: Vec<String>, returns: &str, args| {
        vm.invoke(
            Method {
                class: "Lorg/droidless/images/InflaterContract;".into(),
                name: name.into(),
                parameters,
                returns: returns.into(),
            },
            args,
            false,
        )
    };
    assert_eq!(
        call(
            &mut vm,
            "run",
            vec!["Landroid/app/Activity;".into()],
            "I",
            vec![activity]
        )
        .unwrap(),
        [Word::from(1)]
    );
    let root = call(&mut vm, "root", vec![], "Landroid/view/View;", vec![]).unwrap()[0];
    assert_eq!(
        call(&mut vm, "verifyDeepMerge", vec![], "I", vec![]).unwrap(),
        [Word::from(1)]
    );
    let deep = call(
        &mut vm,
        "deepRoot",
        vec![],
        "Landroid/view/LayoutInflater;",
        vec![],
    )
    .unwrap()[0];
    let error = call(&mut vm, "mergeLimit", vec![], "V", vec![]).unwrap_err();
    assert!(format!("{error:#}").contains("factory merge limit reached (64)"));
    assert_eq!(vm.stack_depth(), 0);
    let error = call(&mut vm, "callbackFault", vec![], "V", vec![]).unwrap_err();
    assert!(format!("{error:#}").contains("inflater callback failed"));
    assert_eq!(vm.stack_depth(), 0);
    assert_eq!(
        call(&mut vm, "recover", vec![], "I", vec![]).unwrap(),
        [Word::from(1)]
    );
    call(&mut vm, "startWorker", vec![], "V", vec![]).unwrap();
    let error = vm.poll_messages().unwrap_err();
    assert!(format!("{error:#}").contains("unsupported UI access from a guest worker"));
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "release", vec![], "V", vec![]).unwrap();
    vm.collect();
    assert!(
        vm.heap.get(root).is_err(),
        "inflater callback leaked temporary roots"
    );
    assert!(
        vm.heap.get(deep).is_err(),
        "merged factory callback leaked temporary roots"
    );
}
