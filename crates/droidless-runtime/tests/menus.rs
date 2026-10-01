use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

fn call(
    vm: &mut Runtime,
    class: &str,
    name: &str,
    parameters: &[&str],
    returns: &str,
    args: Vec<Word>,
) -> Vec<Word> {
    vm.invoke(
        Method {
            class: class.into(),
            name: name.into(),
            parameters: parameters.iter().map(|s| (*s).into()).collect(),
            returns: returns.into(),
        },
        args,
        false,
    )
    .unwrap()
}

#[test]
fn compiled_menu_xml_ordering_groups_and_tint_gc_contract() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
    )
    .unwrap();
    vm.launch().unwrap();
    let activity = vm.activity.unwrap();
    let menu = vm.create_menu(activity).unwrap();
    assert_eq!(
        call(
            &mut vm,
            "Lorg/droidless/counter/MenuContract;",
            "verify",
            &["Landroid/view/Menu;", "Landroid/app/Activity;"],
            "I",
            vec![menu, activity]
        ),
        [Word::from(1)]
    );
    vm.collect();
    let items = match &vm.heap.get(menu).unwrap().data {
        droidless_runtime::heap::Data::Menu(items) => items.clone(),
        _ => panic!("missing menu state"),
    };
    assert_eq!(items.len(), 3);
    let tail = vm.heap.get(items[2]).unwrap();
    let listener = tail.fields["listener"][0];
    let icon = tail.fields["icon"][0];
    assert!(vm.heap.get(listener).is_ok());
    let colors = vm.heap.get(icon).unwrap().fields["droidless:drawable:tint-list"][0];
    assert_eq!(
        call(
            &mut vm,
            "Landroid/content/res/ColorStateList;",
            "getDefaultColor",
            &[],
            "I",
            vec![colors]
        ),
        [Word::from(0xff123456u32 as i32)]
    );
    let failed = call(
        &mut vm,
        "Lorg/droidless/counter/MenuContract;",
        "failedTint",
        &[],
        "Landroid/graphics/drawable/ColorDrawable;",
        vec![],
    )[0];
    vm.collect();
    assert!(
        vm.heap.get(failed).is_err(),
        "failed virtual tint leaked native roots"
    );
    let inflater = call(
        &mut vm,
        "Landroid/app/Activity;",
        "getMenuInflater",
        &[],
        "Landroid/view/MenuInflater;",
        vec![activity],
    )[0];
    let id = vm
        .apk
        .resources
        .entries
        .iter()
        .find(|(_, entry)| entry.name == "menu/unsupported")
        .unwrap()
        .0;
    let error = vm
        .invoke(
            Method {
                class: "Landroid/view/MenuInflater;".into(),
                name: "inflate".into(),
                parameters: vec!["I".into(), "Landroid/view/Menu;".into()],
                returns: "V".into(),
            },
            vec![inflater, Word::from(*id as i32), menu],
            false,
        )
        .unwrap_err();
    assert!(format!("{error:#}").contains("onClick remains unsupported"));
    assert_eq!(
        call(&mut vm, "Landroid/view/Menu;", "size", &[], "I", vec![menu]),
        [Word::from(3)],
        "unsupported XML partially mutated menu"
    );
    vm.collect();
    assert!(
        vm.heap.get(inflater).is_err(),
        "XML failure leaked inflater roots"
    );
    call(
        &mut vm,
        "Lorg/droidless/counter/MenuContract;",
        "clear",
        &[],
        "V",
        vec![],
    );
    vm.collect();
    for object in [menu, listener, icon, colors].into_iter().chain(items) {
        assert!(
            vm.heap.get(object).is_err(),
            "menu retained after its last root disappeared"
        );
    }
    assert_eq!(vm.stack_depth(), 0);
    let bounded = vm.create_menu(activity).unwrap();
    let title = vm.heap.string("Bounded".into()).unwrap();
    let add = Method {
        class: "Landroid/view/Menu;".into(),
        name: "add".into(),
        parameters: vec!["Ljava/lang/CharSequence;".into()],
        returns: "Landroid/view/MenuItem;".into(),
    };
    for _ in 0..1024 {
        vm.invoke(add.clone(), vec![bounded, title], false).unwrap();
    }
    assert!(
        vm.invoke(add, vec![bounded, title], false)
            .unwrap_err()
            .to_string()
            .contains("limit reached")
    );
    assert_eq!(
        call(
            &mut vm,
            "Landroid/view/Menu;",
            "size",
            &[],
            "I",
            vec![bounded]
        ),
        [Word::from(1024)]
    );
    vm.collect();
    assert!(vm.heap.get(bounded).is_err());
    vm.close().unwrap();
}

#[test]
fn foreground_menu_callbacks_invalidation_navigation_and_error_cleanup() {
    const ACT: &str = "Lorg/droidless/counter/MenuActivity;";
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap(),
    )
    .unwrap();
    vm.launch().unwrap();
    let caller = vm.activity.unwrap();
    call(
        &mut vm,
        ACT,
        "open",
        &["Landroid/app/Activity;"],
        "V",
        vec![caller],
    );
    vm.poll_messages().unwrap();
    let activity = vm.activity.unwrap();
    let counts = |vm: &mut Runtime| {
        let value = call(vm, ACT, "counts", &[], "Ljava/lang/String;", vec![])[0];
        vm.heap.text(value).unwrap().to_owned()
    };
    let configure = |vm: &mut Runtime, value: i32| {
        call(vm, ACT, "configure", &["I"], "V", vec![Word::from(value)]);
    };
    let invalidate = |vm: &mut Runtime| {
        call(
            vm,
            "Landroid/app/Activity;",
            "invalidateOptionsMenu",
            &[],
            "V",
            vec![activity],
        );
    };
    let handle = |entries: &[droidless_runtime::MenuEntry], id: i32| {
        entries.iter().find(|e| e.item_id == id).unwrap().handle
    };
    let entries = vm.options_menu().unwrap();
    assert_eq!(
        entries.iter().map(|e| e.item_id).collect::<Vec<_>>(),
        [1, 2, 3, 5, 6, 7, 8, 9]
    );
    assert_eq!(entries[0].title, "Handled 1");
    assert!(!entries[2].enabled && entries[3].checked);
    assert_eq!(counts(&mut vm), "1:1:0:0");
    vm.collect();
    assert!(vm.heap.get(Word::Ref(entries[0].handle)).is_ok());
    let next = vm.options_menu().unwrap();
    assert_eq!(next[0].handle, entries[0].handle);
    assert_eq!(next[0].title, "Handled 2");
    assert_eq!(counts(&mut vm), "1:2:0:0");
    assert!(vm.select_menu_item(handle(&entries, 1)).unwrap());
    assert_eq!(vm.snapshot().unwrap().view.text, "Listener 1");
    assert_eq!(counts(&mut vm), "1:2:1:0");
    assert!(!vm.select_menu_item(handle(&entries, 2)).unwrap());
    assert_eq!(vm.snapshot().unwrap().view.text, "Activity 2");
    assert_eq!(counts(&mut vm), "1:2:2:1");
    let menu = vm.heap.get(Word::Ref(entries[0].handle)).unwrap().fields["droidless:menu:owner"][0];
    let hidden = call(
        &mut vm,
        "Landroid/view/Menu;",
        "findItem",
        &["I"],
        "Landroid/view/MenuItem;",
        vec![menu, Word::from(4)],
    )[0];
    assert!(!vm.select_menu_item(hidden.reference().unwrap()).unwrap());
    assert!(!vm.select_menu_item(handle(&entries, 3)).unwrap());
    assert_eq!(counts(&mut vm), "1:2:2:1");
    assert!(!vm.select_menu_item(handle(&entries, 5)).unwrap());
    assert!(
        vm.options_menu()
            .unwrap()
            .iter()
            .find(|e| e.item_id == 5)
            .unwrap()
            .checked,
        "host selection toggled guest state"
    );
    assert!(!vm.select_menu_item(handle(&entries, 8)).unwrap());
    assert_eq!(
        vm.snapshot().unwrap().view.text,
        "Activity 8",
        "invalidation skipped listener fallback"
    );
    assert!(vm.heap.get(menu).is_err(), "invalidated menu retained");
    assert!(!vm.select_menu_item(entries[0].handle).unwrap());
    let fresh = vm.options_menu().unwrap();
    configure(&mut vm, 2);
    assert!(vm.options_menu().unwrap().is_empty());
    let before = counts(&mut vm);
    assert!(
        !vm.select_menu_item(fresh[0].handle).unwrap(),
        "rejected preparation allowed stale input"
    );
    assert_eq!(counts(&mut vm), before);
    assert_eq!(
        vm.options_menu().unwrap()[0].handle,
        fresh[0].handle,
        "prepare rejection recreated cached menu"
    );
    let error = vm.select_menu_item(handle(&fresh, 7)).unwrap_err();
    assert!(format!("{error:#}").contains("menu listener failed"));
    vm.collect();
    assert_eq!(vm.stack_depth(), 0);
    configure(&mut vm, 6);
    assert!(format!("{:#}", vm.options_menu().unwrap_err()).contains("prepare menu failed"));
    vm.collect();
    assert!(
        vm.heap.get(Word::Ref(fresh[0].handle)).is_err(),
        "failed preparation leaked menu roots"
    );
    assert!(!vm.options_menu().unwrap().is_empty());
    for mode in [1, 3, 4] {
        invalidate(&mut vm);
        configure(&mut vm, mode);
        let result = vm.options_menu();
        if mode == 4 {
            assert!(format!("{:#}", result.err().unwrap()).contains("create menu failed"));
        } else {
            assert!(result.unwrap().is_empty());
        }
        vm.collect();
        assert_eq!(vm.stack_depth(), 0);
        assert!(
            !vm.options_menu().unwrap().is_empty(),
            "menu did not recover after callback invalidation/rejection/failure"
        );
    }
    let original = vm.options_menu().unwrap();
    assert!(!vm.select_menu_item(handle(&original, 9)).unwrap());
    assert_eq!(vm.activity_depth(), 3);
    assert!(
        !vm.select_menu_item(original[0].handle).unwrap(),
        "background Activity accepted stale input"
    );
    let child = vm.options_menu().unwrap();
    vm.back().unwrap();
    assert_eq!(vm.activity, Some(activity));
    assert!(
        vm.heap.get(Word::Ref(child[0].handle)).is_err(),
        "destroyed Activity retained menu"
    );
    assert!(!vm.select_menu_item(handle(&original, 6)).unwrap());
    assert_eq!(vm.activity, Some(caller));
    assert!(vm.heap.get(Word::Ref(original[0].handle)).is_err());
    assert!(!vm.select_menu_item(original[0].handle).unwrap());
    call(
        &mut vm,
        ACT,
        "open",
        &["Landroid/app/Activity;"],
        "V",
        vec![caller],
    );
    vm.poll_messages().unwrap();
    configure(&mut vm, 5);
    assert!(
        vm.options_menu().unwrap().is_empty(),
        "finishing creation exposed a menu"
    );
    assert_eq!(vm.activity, Some(caller));
    assert_eq!(vm.stack_depth(), 0);
    vm.close().unwrap();
}
