use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

fn call(vm: &mut Runtime, name: &str, returns: &str) -> Vec<Word> {
    vm.invoke(
        Method {
            class: "Lorg/droidless/counter/PropertyAnimationContract;".into(),
            name: name.into(),
            parameters: vec![],
            returns: returns.into(),
        },
        vec![],
        false,
    )
    .unwrap()
}
fn value(vm: &mut Runtime, name: &str) -> f32 {
    f32::from_bits(call(vm, name, "F")[0].int().unwrap() as u32)
}
fn events(vm: &mut Runtime) -> String {
    let word = call(vm, "events", "Ljava/lang/String;")[0];
    vm.heap.text(word).unwrap().to_owned()
}
fn transient(vm: &mut Runtime) -> bool {
    call(vm, "transientState", "Z")[0].truth()
}

#[test]
fn compiled_property_frames_replacement_cancellation_gc_and_fault_cleanup() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
    )
    .unwrap();
    let activity = vm.heap.instance("Landroid/app/Activity;").unwrap();
    let root = vm
        .invoke(
            Method {
                class: "Lorg/droidless/counter/PropertyAnimationContract;".into(),
                name: "setup".into(),
                parameters: vec!["Landroid/app/Activity;".into()],
                returns: "Landroid/view/View;".into(),
            },
            vec![activity],
            false,
        )
        .unwrap()[0];
    vm.root = Some(root);
    call(&mut vm, "reset", "V");
    call(&mut vm, "basic", "V");
    assert_eq!(events(&mut vm), "S");
    assert!(transient(&mut vm));
    assert_eq!(value(&mut vm, "x"), 0.0);
    vm.collect();
    vm.advance_time(50).unwrap();
    assert_eq!(value(&mut vm, "x"), 50.0);
    assert_eq!(value(&mut vm, "alpha"), 0.5);
    let tree = vm.layout_snapshot().unwrap();
    assert_eq!(
        (tree.children[0].rect.x, tree.children[0].view.alpha),
        (50.0, 0.5)
    );
    vm.advance_time(50).unwrap();
    assert_eq!(events(&mut vm), "SE");
    assert_eq!(value(&mut vm, "x"), 100.0);
    assert_eq!(value(&mut vm, "alpha"), 0.0);
    assert!(!transient(&mut vm));
    assert!(call(&mut vm, "ended", "Z")[0].truth());

    call(&mut vm, "reset", "V");
    call(&mut vm, "automatic", "V");
    assert_eq!(events(&mut vm), "");
    vm.poll_messages().unwrap();
    assert_eq!(events(&mut vm), "S");
    vm.advance_time(60).unwrap();
    assert_eq!(value(&mut vm, "y"), 30.0);
    vm.advance_time(60).unwrap();
    assert_eq!(value(&mut vm, "y"), 60.0);
    assert_eq!(events(&mut vm), "SE");

    call(&mut vm, "reset", "V");
    call(&mut vm, "delayed", "V");
    vm.advance_time(49).unwrap();
    assert_eq!(events(&mut vm), "");
    vm.advance_time(1).unwrap();
    assert_eq!(events(&mut vm), "S");
    vm.advance_time(50).unwrap();
    assert_eq!(value(&mut vm, "y"), 50.0);
    call(&mut vm, "cancel", "V");
    assert_eq!(events(&mut vm), "SCE");
    assert_eq!(value(&mut vm, "y"), 50.0);
    assert!(!transient(&mut vm));
    call(&mut vm, "reset", "V");
    call(&mut vm, "delayed", "V");
    call(&mut vm, "cancel", "V");
    assert_eq!(events(&mut vm), "SCE");
    call(&mut vm, "reset", "V");
    call(&mut vm, "cancelPending", "V");
    vm.poll_messages().unwrap();
    assert_eq!(events(&mut vm), "");
    assert!(!transient(&mut vm));

    call(&mut vm, "basic", "V");
    vm.advance_time(50).unwrap();
    call(&mut vm, "replaceX", "V");
    assert_eq!(events(&mut vm), "SS");
    vm.advance_time(50).unwrap();
    assert_eq!(value(&mut vm, "x"), 125.0);
    assert_eq!(value(&mut vm, "alpha"), 0.0);
    assert!(transient(&mut vm));
    vm.advance_time(50).unwrap();
    assert_eq!(value(&mut vm, "x"), 200.0);
    assert_eq!(events(&mut vm), "SSEE");
    assert!(!transient(&mut vm));
    call(&mut vm, "reset", "V");
    call(&mut vm, "cancelReentrantly", "V");
    assert_eq!(events(&mut vm), "SCE");
    assert!(!transient(&mut vm));
    call(&mut vm, "reset", "V");
    call(&mut vm, "restartOnEnd", "V");
    vm.advance_time(50).unwrap();
    assert_eq!(events(&mut vm), "SES");
    vm.advance_time(50).unwrap();
    assert_eq!(events(&mut vm), "SESE");
    assert_eq!(value(&mut vm, "y"), 40.0);

    call(&mut vm, "reset", "V");
    call(&mut vm, "basic", "V");
    vm.advance_time(25).unwrap();
    assert_eq!(value(&mut vm, "x"), 25.0);
    call(&mut vm, "recurve", "V");
    vm.collect();
    vm.advance_time(25).unwrap();
    assert_eq!(value(&mut vm, "x"), 25.0);
    assert_eq!(value(&mut vm, "alpha"), 0.75);
    vm.advance_time(50).unwrap();
    assert_eq!(events(&mut vm), "SE");

    for (kind, text) in [(1, "start"), (2, "update"), (5, "curve")] {
        call(&mut vm, "reset", "V");
        let error = vm
            .invoke(
                Method {
                    class: "Lorg/droidless/counter/PropertyAnimationContract;".into(),
                    name: "fail".into(),
                    parameters: vec!["I".into()],
                    returns: "V".into(),
                },
                vec![Word::from(kind)],
                false,
            )
            .unwrap_err();
        assert!(format!("{error:#}").contains(&format!("property {text} failure")));
        assert_eq!(vm.stack_depth(), 0);
        assert!(!transient(&mut vm));
    }
    call(&mut vm, "reset", "V");
    call(&mut vm, "basic", "V");
    let error = vm
        .invoke(
            Method {
                class: "Lorg/droidless/counter/PropertyAnimationContract;".into(),
                name: "failAtCancel".into(),
                parameters: vec![],
                returns: "V".into(),
            },
            vec![],
            false,
        )
        .unwrap_err();
    assert!(format!("{error:#}").contains("property cancel failure"));
    assert_eq!(vm.stack_depth(), 0);
    assert!(!transient(&mut vm));
    call(&mut vm, "reset", "V");
    vm.invoke(
        Method {
            class: "Lorg/droidless/counter/PropertyAnimationContract;".into(),
            name: "fail".into(),
            parameters: vec!["I".into()],
            returns: "V".into(),
        },
        vec![Word::from(3)],
        false,
    )
    .unwrap();
    let error = vm.advance_time(100).unwrap_err();
    assert!(format!("{error:#}").contains("property end failure"));
    assert_eq!(vm.stack_depth(), 0);
    assert!(!transient(&mut vm));
    call(&mut vm, "reset", "V");
    call(&mut vm, "basic", "V");
    vm.close().unwrap();
    assert!(!transient(&mut vm));
    call(&mut vm, "release", "V");
    vm.collect();
    assert!(vm.heap.get(root).is_err());
    assert!(
        vm.heap.get(activity).is_err(),
        "property callbacks retained temporary roots"
    );
}
