use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

fn runtime() -> Runtime {
    Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/intents.apk")).unwrap())
        .unwrap()
}
fn call(vm: &mut Runtime, name: &str, returns: &str) -> Vec<Word> {
    vm.invoke(
        Method {
            class: "Lorg/droidless/intents/MainActivity;".into(),
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
