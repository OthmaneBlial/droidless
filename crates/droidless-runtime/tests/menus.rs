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
