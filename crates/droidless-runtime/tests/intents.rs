use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

fn runtime() -> Runtime {
    Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap())
        .unwrap()
}
fn call(vm: &mut Runtime, name: &str, returns: &str) -> Vec<Word> {
    call_class(vm, "MainActivity", name, returns)
}
fn call_class(vm: &mut Runtime, class: &str, name: &str, returns: &str) -> Vec<Word> {
    vm.invoke(
        Method {
            class: format!("Lorg/droidless/intents/{class};"),
            name: name.into(),
            parameters: vec![],
            returns: returns.into(),
        },
        vec![],
        false,
    )
    .unwrap()
}

#[test]
fn platform_fragments_queue_lifecycle_navigation_gc_and_guest_errors() {
    let mut vm = runtime();
    vm.launch().unwrap();
    let fragment = call_class(
        &mut vm,
        "FragmentProbe",
        "initial",
        "Landroid/app/Fragment;",
    )[0];
    let initial =
        "initial:attach;initial:create;initial:view;initial:activity;initial:start;initial:resume;";
    let log = call_class(&mut vm, "FragmentProbe", "eventLog", "Ljava/lang/String;")[0];
    assert_eq!(vm.heap.text(log).unwrap(), initial);
    vm.click_text("Open detail").unwrap();
    vm.collect();
    assert!(vm.heap.get(fragment).is_ok());
    vm.back().unwrap();
    vm.close().unwrap();
    let log = call_class(&mut vm, "FragmentProbe", "eventLog", "Ljava/lang/String;")[0];
    assert_eq!(
        vm.heap.text(log).unwrap(),
        format!(
            "{initial}initial:pause;initial:stop;initial:start;initial:resume;initial:pause;initial:stop;initial:destroyView;initial:destroy;initial:detach;"
        )
    );
    vm.collect();
    assert!(
        vm.heap.get(fragment).is_err(),
        "destroyed manager retained fragment"
    );

    let mut vm = runtime();
    vm.launch().unwrap();
    call_class(&mut vm, "FragmentProbe", "addLater", "V");
    vm.collect();
    let log = call_class(&mut vm, "FragmentProbe", "eventLog", "Ljava/lang/String;")[0];
    assert_eq!(
        vm.heap.text(log).unwrap(),
        initial,
        "commit executed synchronously"
    );
    vm.poll_messages().unwrap();
    let log = call_class(&mut vm, "FragmentProbe", "eventLog", "Ljava/lang/String;")[0];
    assert_eq!(
        vm.heap.text(log).unwrap(),
        format!("{initial}late:attach;late:create;late:view;late:activity;late:start;late:resume;")
    );
    assert_eq!(
        call_class(&mut vm, "FragmentProbe", "errors", "I"),
        [Word::from(6)]
    );
    assert_eq!(vm.stack_depth(), 0);
    vm.close().unwrap();
}

#[test]
fn application_observers_snapshot_navigation_gc_and_fault_cleanup() {
    let mut vm = runtime();
    vm.launch().unwrap();
    let mut expected =
        String::from("mutator:home:create;removed:home:create;permanent:home:create;");
    let transitions = [
        ("home", "start"),
        ("home", "resume"),
        ("home", "pause"),
        ("detail", "create"),
        ("detail", "start"),
        ("detail", "resume"),
        ("home", "stop"),
        ("detail", "pause"),
        ("home", "start"),
        ("home", "resume"),
        ("detail", "stop"),
        ("detail", "destroy"),
        ("home", "pause"),
        ("home", "stop"),
        ("home", "destroy"),
    ];
    vm.click_text("Open detail").unwrap();
    vm.collect();
    vm.back().unwrap();
    let observer = call_class(
        &mut vm,
        "ProbeApplication",
        "registerTransient",
        "Ljava/lang/Object;",
    )[0];
    vm.collect();
    assert!(
        vm.heap.get(observer).is_ok(),
        "registered observer was collected"
    );
    let app = call_class(
        &mut vm,
        "ProbeApplication",
        "application",
        "Landroid/app/Application;",
    )[0];
    let unregister = Method {
        class: "Landroid/app/Application;".into(),
        name: "unregisterActivityLifecycleCallbacks".into(),
        parameters: vec!["Landroid/app/Application$ActivityLifecycleCallbacks;".into()],
        returns: "V".into(),
    };
    vm.invoke(unregister, vec![app, observer], true).unwrap();
    vm.collect();
    assert!(
        vm.heap.get(observer).is_err(),
        "unregistered observer retained a root"
    );
    vm.close().unwrap();
    for (screen, event) in transitions {
        for observer in ["permanent", "late"] {
            expected.push_str(&format!("{observer}:{screen}:{event};"));
        }
    }
    let log = call_class(
        &mut vm,
        "ProbeApplication",
        "eventLog",
        "Ljava/lang/String;",
    )[0];
    assert_eq!(vm.heap.text(log).unwrap(), expected);

    let mut vm = runtime();
    vm.launch().unwrap();
    let observer = call_class(
        &mut vm,
        "ProbeApplication",
        "registerFault",
        "Ljava/lang/Object;",
    )[0];
    let activity = vm.activity.unwrap();
    let error = vm
        .invoke(
            Method {
                class: "Landroid/app/Activity;".into(),
                name: "onStart".into(),
                parameters: vec![],
                returns: "V".into(),
            },
            vec![activity],
            false,
        )
        .unwrap_err();
    assert!(format!("{error:#}").contains("observer failed"));
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    assert!(
        vm.heap.get(observer).is_err(),
        "failed dispatch retained its snapshot"
    );
    vm.close().unwrap();
}

#[test]
fn extras_lifecycle_back_and_preserved_screen() {
    let mut vm = runtime();
    assert_eq!(call(&mut vm, "bundleValues", "I"), [Word::from(1)]);
    vm.launch().unwrap();
    let root = vm.root;
    let tree = vm.snapshot().unwrap();
    let input = tree
        .children
        .iter()
        .find(|n| n.view.editable)
        .unwrap()
        .handle;
    vm.edit(input, "Keep me").unwrap();
    vm.click_text("Open detail").unwrap();
    assert_eq!(vm.activity_depth(), 2);
    assert_eq!(vm.title, "Detail");
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Original extras"
    );
    vm.collect();
    assert_eq!(
        vm.heap
            .get(Word::Ref(input))
            .unwrap()
            .view
            .as_ref()
            .unwrap()
            .text,
        "Keep me"
    );
    vm.back().unwrap();
    assert_eq!(vm.activity_depth(), 1);
    assert_eq!(vm.title, "Home");
    assert_eq!(vm.root, root);
    let tree = vm.snapshot().unwrap();
    assert_eq!(tree.children[0].view.text, "Home resume 2");
    assert_eq!(tree.children[1].view.text, "Keep me");
    let log = call(&mut vm, "eventLog", "Ljava/lang/String;")[0];
    assert_eq!(
        vm.heap.text(log).unwrap(),
        concat!(
            "home:create;home:start;home:resume;home:callback-return;home:pause;",
            "detail:create;detail:start;detail:resume;home:stop;detail:pause;",
            "home:restart;home:start;home:resume;detail:stop;detail:destroy;"
        )
    );
    assert_eq!(call(&mut vm, "finishCount", "I"), [Word::from(1)]);
    vm.back().unwrap();
    assert!(vm.activity.is_none() && vm.root.is_none());
    assert_eq!(vm.activity_depth(), 0);
    assert_eq!(call(&mut vm, "finishCount", "I"), [Word::from(2)]);
    let lifecycle = vm.lifecycle.clone();
    vm.close().unwrap();
    assert_eq!(vm.lifecycle, lifecycle);
}

#[test]
fn overridden_back_and_finish_an_inactive_activity() {
    let mut vm = runtime();
    vm.launch().unwrap();
    vm.click_text("Open guarded").unwrap();
    vm.back().unwrap();
    assert_eq!(vm.activity_depth(), 2);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Back intercepted"
    );
    vm.back().unwrap();
    assert_eq!(vm.title, "Home");
    vm.click_text("Open detail").unwrap();
    let detail = vm.activity;
    vm.click_text("Finish home").unwrap();
    assert_eq!(vm.activity, detail);
    assert_eq!(vm.activity_depth(), 1);
    assert_eq!(call(&mut vm, "finishCount", "I"), [Word::from(1)]);
    vm.click_text("Finish").unwrap();
    assert_eq!(vm.activity_depth(), 0);
    assert_eq!(call(&mut vm, "finishCount", "I"), [Word::from(2)]);
}

#[test]
fn close_destroys_both_screens_and_invalid_intents_fail() {
    let mut vm = runtime();
    vm.launch().unwrap();
    let activity = vm.activity.unwrap();
    let intent = vm.heap.instance("Landroid/content/Intent;").unwrap();
    let start = Method {
        class: "Landroid/content/Context;".into(),
        name: "startActivity".into(),
        parameters: vec!["Landroid/content/Intent;".into()],
        returns: "V".into(),
    };
    assert!(
        format!(
            "{:#}",
            vm.invoke(start.clone(), vec![activity, intent], false)
                .unwrap_err()
        )
        .contains("implicit/external")
    );
    let target = vm
        .heap
        .string("Lorg/droidless/intents/Undeclared;".into())
        .unwrap();
    vm.heap
        .get_mut(intent)
        .unwrap()
        .fields
        .insert("component".into(), vec![target]);
    assert!(
        format!(
            "{:#}",
            vm.invoke(start, vec![activity, intent], false).unwrap_err()
        )
        .contains("not a declared")
    );
    vm.click_text("Open detail").unwrap();
    vm.close().unwrap();
    assert_eq!(call(&mut vm, "finishCount", "I"), [Word::from(2)]);
    assert!(vm.activity.is_none() && vm.root.is_none());
    assert_eq!(
        &vm.lifecycle[vm.lifecycle.len() - 4..],
        ["onPause", "onStop", "onDestroy", "onDestroy"]
    );
}
