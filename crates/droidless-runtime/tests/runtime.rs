use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{
    Runtime,
    heap::{Word, bits64, wide},
    ui::Node,
};

fn runtime() -> Runtime {
    Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk")).unwrap())
        .unwrap()
}
fn eval(
    vm: &mut Runtime,
    name: &str,
    parameters: &[&str],
    returns: &str,
    args: Vec<Word>,
) -> Vec<Word> {
    vm.invoke(
        Method {
            class: "Lorg/droidless/counter/MainActivity;".into(),
            name: name.into(),
            parameters: parameters.iter().map(|s| (*s).into()).collect(),
            returns: returns.into(),
        },
        args,
        false,
    )
    .unwrap()
}
fn texts(node: &Node) -> Vec<String> {
    std::iter::once(node.view.text.clone())
        .chain(node.children.iter().flat_map(texts))
        .collect()
}

#[test]
fn real_d8_code_arithmetic_objects_arrays_dispatch_exceptions() {
    let mut vm = runtime();
    assert_eq!(
        eval(&mut vm, "sum", &["I"], "I", vec![Word::from(10)]),
        vec![Word::from(48)]
    );
    let mut args = wide(0xf000000000000001);
    args.push(Word::from(4));
    assert_eq!(
        bits64(&eval(&mut vm, "wideMath", &["J", "I"], "J", args)).unwrap(),
        31
    );
    let args = wide(3.0f64.to_bits())
        .into_iter()
        .chain(wide(4.0f64.to_bits()))
        .collect();
    assert_eq!(
        f64::from_bits(bits64(&eval(&mut vm, "squares", &["D", "D"], "D", args)).unwrap()),
        25.0
    );
    for (name, expected) in [("arrays", 20), ("dispatch", 18), ("caught", 7)] {
        assert_eq!(
            eval(&mut vm, name, &[], "I", vec![]),
            vec![Word::from(expected)]
        );
    }
    assert_eq!(vm.stack_depth(), 0);
}

#[test]
fn lifecycle_xml_resources_and_dex_callbacks() {
    let mut vm = runtime();
    vm.launch().unwrap();
    assert_eq!(vm.title, "DROIDLESS Counter fixture");
    assert_eq!(vm.lifecycle, ["onCreate", "onStart", "onResume"]);
    assert!(vm.click_text("Increment").unwrap());
    vm.click_text("Increment").unwrap();
    vm.click_text("Decrement").unwrap();
    assert!(texts(&vm.snapshot().unwrap()).contains(&"1".into()));
    let tree = vm.snapshot().unwrap();
    let input = tree
        .children
        .iter()
        .find(|n| n.view.kind == "EditText")
        .unwrap();
    vm.edit(input.handle, "Bonjour").unwrap();
    vm.click_text("Copy input").unwrap();
    assert_eq!(
        texts(&vm.snapshot().unwrap())
            .iter()
            .filter(|s| *s == "Bonjour")
            .count(),
        2
    );
    assert!(vm.key(input.handle, 0, 16).unwrap());
    assert!(texts(&vm.snapshot().unwrap()).contains(&"key 16".into()));
    assert!(!vm.key(input.handle, 1, 16).unwrap());
    assert!(
        tree.children
            .iter()
            .all(|n| n.rect.width > 0.0 && n.rect.height > 0.0)
    );
    vm.close().unwrap();
    assert_eq!(&vm.lifecycle[3..], ["onPause", "onStop", "onDestroy"]);
}

#[test]
fn instruction_budget_stops_a_nonterminating_apk_method() {
    let mut vm = runtime();
    let dex = &mut vm.apk.dex[0];
    let idx = dex.methods.iter().position(|m| m.name == "sum").unwrap();
    let method = dex
        .classes
        .iter_mut()
        .flat_map(|c| &mut c.methods)
        .find(|m| m.index == idx)
        .unwrap();
    method.code.as_mut().unwrap().instructions = vec![0x0028]; // goto +0
    let error = vm
        .invoke(
            Method {
                class: "Lorg/droidless/counter/MainActivity;".into(),
                name: "sum".into(),
                parameters: vec!["I".into()],
                returns: "I".into(),
            },
            vec![Word::from(10)],
            false,
        )
        .unwrap_err();
    assert!(format!("{error:#}").contains("instruction budget"));
    assert_eq!(vm.stack_depth(), 0);
}

#[test]
fn unmodified_third_party_activity_executes() {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/third-party/smallest.apk")).unwrap(),
    )
    .unwrap();
    vm.launch().unwrap();
    assert_eq!(vm.snapshot().unwrap().view.text, "OK");
    assert!(vm.instructions > 0);
    vm.close().unwrap();
}

#[test]
fn unsupported_apis_fail_without_succeeding_silently() {
    let mut vm = runtime();
    let method = Method {
        class: "Ljava/lang/System;".into(),
        name: "exit".into(),
        parameters: vec!["I".into()],
        returns: "V".into(),
    };
    assert!(
        vm.invoke(method, vec![Word::ZERO], false)
            .unwrap_err()
            .to_string()
            .contains("unsupported method")
    );
}
