use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

#[test]
fn compiled_text_layout_measurement_invalidation_and_callback_gc() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
    )
    .unwrap();
    let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
    let result = vm
        .invoke(
            Method {
                class: "Lorg/droidless/counter/TextLayoutContract;".into(),
                name: "run".into(),
                parameters: vec!["Landroid/app/Activity;".into()],
                returns: "I".into(),
            },
            vec![activity],
            false,
        )
        .unwrap();
    assert_eq!(result, [Word::from(1)]);
    assert_eq!(vm.stack_depth(), 0);
    let call = |vm: &mut Runtime, name: &str, returns: &str| {
        vm.invoke(
            Method {
                class: "Lorg/droidless/counter/TextLayoutContract;".into(),
                name: name.into(),
                parameters: vec![],
                returns: returns.into(),
            },
            vec![],
            false,
        )
        .unwrap()
    };
    let root = call(&mut vm, "root", "Landroid/view/View;")[0];
    let failure = vm
        .invoke(
            Method {
                class: "Landroid/view/View;".into(),
                name: "setPaddingRelative".into(),
                parameters: vec!["I".into(); 4],
                returns: "V".into(),
            },
            vec![root, Word::from(-1), Word::ZERO, Word::ZERO, Word::ZERO],
            true,
        )
        .unwrap_err();
    assert!(format!("{failure:#}").contains("padding must have four non-negative values"));
    assert_eq!(
        vm.heap.get(root).unwrap().view.as_ref().unwrap().padding,
        [6.0, 3.0, 6.0, 3.0]
    );
    assert!(!vm.heap.get(root).unwrap().fields["droidless:view:padding-relative"][0].truth());
    assert_eq!(vm.stack_depth(), 0);
    let tree = droidless_runtime::ui::layout(&vm.heap, root, 162.0, 1000.0).unwrap();
    let row = &tree.children[0];
    assert_eq!(
        (row.rect.x, row.rect.y, row.rect.width, row.rect.height),
        (6.0, 3.0, 150.0, 48.0)
    );
    assert_eq!(row.children.len(), 2);
    assert_eq!(
        (row.children[0].rect.x, row.children[0].rect.width),
        (14.0, 90.0)
    );
    assert_eq!(
        (row.children[1].rect.x, row.children[1].rect.width),
        (108.0, 42.0)
    );
    vm.root = Some(root);
    vm.width = 162.0;
    vm.height = 1000.0;
    vm.layout_snapshot().unwrap();
    assert_eq!(call(&mut vm, "layoutCount", "I"), [Word::from(1)]);
    call(&mut vm, "remeasure", "V");
    vm.layout_snapshot().unwrap();
    assert_eq!(call(&mut vm, "layoutCount", "I"), [Word::from(2)]);
    vm.layout_snapshot().unwrap();
    assert_eq!(call(&mut vm, "layoutCount", "I"), [Word::from(2)]);
    call(&mut vm, "failLayout", "V");
    assert!(format!("{:#}", vm.layout_snapshot().unwrap_err()).contains("layout failure"));
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "recoverLayout", "V");
    vm.layout_snapshot().unwrap();
    assert_eq!(call(&mut vm, "layoutCount", "I"), [Word::from(4)]);
    vm.root = None;
    call(&mut vm, "release", "V");
    let failure = vm
        .invoke(
            Method {
                class: "Lorg/droidless/counter/TextLayoutContract;".into(),
                name: "mutate".into(),
                parameters: vec!["Landroid/app/Activity;".into()],
                returns: "V".into(),
            },
            vec![activity],
            false,
        )
        .unwrap_err();
    assert!(format!("{failure:#}").contains("hierarchy mutation during container measurement"));
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(activity).is_err(),
        "measurement retained temporary roots"
    );
}

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
    call(&mut vm, "invalidateFrame", "V");
    assert_eq!(vm.poll_messages().unwrap(), 1);
    assert_eq!(vm.poll_messages().unwrap(), 0);
    call(&mut vm, "invalidateDetachedFrame", "V");
    assert_eq!(vm.poll_messages().unwrap(), 0);
    let removed = call(&mut vm, "startFrame", "Landroid/view/View;")[0];
    let root = call(&mut vm, "frameRoot", "Landroid/view/View;")[0];
    let frame_state = |vm: &mut Runtime| {
        let word = call(vm, "frameState", "Ljava/lang/String;")[0];
        vm.heap.text(word).unwrap().to_owned()
    };
    vm.layout_snapshot().unwrap();
    assert_eq!(frame_state(&mut vm), "1:0:0");
    vm.collect();
    assert!(
        vm.heap.get(removed).is_err(),
        "detached child retained after frame"
    );
    assert_eq!(vm.advance_time(500).unwrap(), 1);
    let half = vm.layout_snapshot().unwrap();
    assert_eq!(half.children.len(), 2);
    assert_eq!(half.children[0].rect.x, 50.0);
    assert_eq!(frame_state(&mut vm), "2:50:0");
    assert_eq!(vm.advance_time(500).unwrap(), 1);
    let done = vm.layout_snapshot().unwrap();
    assert_eq!(done.children[0].rect.x, 100.0);
    assert_eq!(frame_state(&mut vm), "3:100:0");
    assert_eq!(vm.poll_messages().unwrap(), 0);
    call(&mut vm, "failFrame", "V");
    assert!(format!("{:#}", vm.layout_snapshot().unwrap_err()).contains("scroll frame failure"));
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "recoverFrame", "V");
    vm.layout_snapshot().unwrap();
    assert_eq!(frame_state(&mut vm), "4:100:0");
    call(&mut vm, "releaseFrame", "V");
    vm.collect();
    assert!(
        vm.heap.get(root).is_err(),
        "scroll failure retained temporary roots"
    );
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
