use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

#[test]
fn compiled_widget_metadata_adapter_and_timed_scroll_contracts() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap())
            .unwrap();
    vm.launch().unwrap();
    let tree = vm.snapshot().unwrap();
    assert_eq!(
        tree.children[1].view.content_description.as_deref(),
        Some("Packaged PNG source")
    );
    assert_eq!(
        tree.children[2].view.content_description.as_deref(),
        Some("Decoded PNG image")
    );
    let call = |vm: &mut Runtime, name: &str, returns: &str| {
        vm.invoke(
            Method {
                class: "Lorg/droidless/images/WidgetProbe;".into(),
                name: name.into(),
                parameters: vec![],
                returns: returns.into(),
            },
            vec![],
            false,
        )
        .unwrap()
    };
    let state = |vm: &mut Runtime| {
        let word = call(vm, "scrollState", "Ljava/lang/String;")[0];
        vm.heap.text(word).unwrap().to_owned()
    };
    call(&mut vm, "startScroll", "V");
    assert_eq!(state(&mut vm), "true:false:10:20");
    vm.advance_time(500).unwrap();
    assert_eq!(state(&mut vm), "true:false:21:0");
    call(&mut vm, "stopScroll", "V");
    assert_eq!(state(&mut vm), "false:true:21:0");
    call(&mut vm, "abortScroll", "V");
    assert_eq!(state(&mut vm), "false:true:31:-21");
    call(&mut vm, "startScroll", "V");
    vm.advance_time(1000).unwrap();
    assert_eq!(state(&mut vm), "true:true:31:-21");
    assert_eq!(state(&mut vm), "false:true:31:-21");
    call(&mut vm, "startDefaultScroll", "V");
    assert_eq!(state(&mut vm), "true:false:0:0");
    vm.advance_time(125).unwrap();
    assert_eq!(state(&mut vm), "true:false:969:-969");
    vm.advance_time(125).unwrap();
    assert_eq!(state(&mut vm), "true:true:1000:-1000");
    call(&mut vm, "startLargeScroll", "V");
    assert_eq!(state(&mut vm), "true:false:16777215:0");
    call(&mut vm, "startInstantScroll", "V");
    assert_eq!(state(&mut vm), "true:true:4:6");
    let failed = call(&mut vm, "failedScroll", "Landroid/widget/OverScroller;")[0];
    call(&mut vm, "clearScroll", "V");
    vm.collect();
    assert!(
        vm.heap.get(failed).is_err(),
        "interpolator failure retained temporary roots"
    );
    assert_eq!(vm.stack_depth(), 0);
    vm.close().unwrap();

    for (target, minimum, expected) in [
        (Some(28), Some(21), 28),
        (None, Some(7), 7),
        (None, None, 1),
    ] {
        let mut apk = Apk::parse(include_bytes!("../../../fixtures/generated/images.apk")).unwrap();
        apk.manifest.target_sdk = target;
        apk.manifest.min_sdk = minimum;
        let mut vm = Runtime::new(apk).unwrap();
        let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
        assert_eq!(
            vm.invoke(
                Method {
                    class: "Lorg/droidless/images/WidgetProbe;".into(),
                    name: "metadataTarget".into(),
                    parameters: vec!["Landroid/app/Activity;".into()],
                    returns: "I".into(),
                },
                vec![activity],
                false
            )
            .unwrap(),
            [Word::from(expected)]
        );
    }
}
