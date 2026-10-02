use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{
    GuestExit, Runtime,
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
fn textutils_utf16_search_and_disjoint_plain_text_replacement() {
    let mut vm = runtime();
    let method = Method {
        class: "Landroid/text/TextUtils;".into(),
        name: "indexOf".into(),
        parameters: vec!["Ljava/lang/CharSequence;".into(); 2],
        returns: "I".into(),
    };
    for spanned in [false, true] {
        for (text, needle, expected) in [
            ("a🦀b🦀", "b", 3),
            ("a🦀b🦀", "🦀", 1),
            ("a🦀b🦀", "🦀x", -1),
            ("", "", 0),
            ("x", "", 0),
            ("x", "xx", -1),
        ] {
            let source = vm.heap.string(text.into()).unwrap();
            let needle = vm.heap.string(needle.into()).unwrap();
            let source = if spanned {
                let target = vm.heap.instance("Landroid/text/SpannedString;").unwrap();
                vm.invoke(
                    Method {
                        class: "Landroid/text/SpannedString;".into(),
                        name: "<init>".into(),
                        parameters: vec!["Ljava/lang/CharSequence;".into()],
                        returns: "V".into(),
                    },
                    vec![target, source],
                    false,
                )
                .unwrap();
                target
            } else {
                source
            };
            assert_eq!(
                vm.invoke(method.clone(), vec![source, needle], false)
                    .unwrap(),
                [Word::from(expected)]
            );
        }
    }
    let text = vm.heap.string("x".into()).unwrap();
    for args in [vec![Word::ZERO, text], vec![text, Word::ZERO]] {
        assert!(
            format!("{:#}", vm.invoke(method.clone(), args, false).unwrap_err())
                .contains("NullPointerException")
        );
    }
    let replace = Method {
        class: "Landroid/text/TextUtils;".into(),
        name: "replace".into(),
        parameters: vec![
            "Ljava/lang/CharSequence;".into(),
            "[Ljava/lang/String;".into(),
            "[Ljava/lang/CharSequence;".into(),
        ],
        returns: "Ljava/lang/CharSequence;".into(),
    };
    let array = |vm: &mut Runtime, element: &str, texts: &[&str]| {
        let values = texts
            .iter()
            .map(|text| vec![vm.heap.string((*text).into()).unwrap()])
            .collect();
        let array = vm.heap.instance(&format!("[{element}")).unwrap();
        vm.heap.get_mut(array).unwrap().data = droidless_runtime::heap::Data::Array {
            element: element.into(),
            values,
        };
        array
    };
    let source = vm.heap.string("🦀 <a> & <a>".into()).unwrap();
    let sources = array(&mut vm, "Ljava/lang/String;", &["&", "<a>"]);
    let destinations = array(&mut vm, "Ljava/lang/CharSequence;", &["<a>", "&amp;"]);
    let result = vm
        .invoke(replace.clone(), vec![source, sources, destinations], false)
        .unwrap()[0];
    assert_eq!(vm.heap.text(result).unwrap(), "🦀 &amp; <a> <a>");
    assert_eq!(vm.heap.text(source).unwrap(), "🦀 <a> & <a>");
    let sources = array(&mut vm, "Ljava/lang/String;", &["<a>", "a>"]);
    let error = vm
        .invoke(replace, vec![source, sources, destinations], false)
        .unwrap_err();
    assert!(format!("{error:#}").contains("overlapping text replacement"));
    assert_eq!(vm.stack_depth(), 0);
    vm.close().unwrap();
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
fn implicit_java_faults_catch_finally_and_inherited_array_types() {
    let mut vm = runtime();
    for (which, expected) in [1, 1, 3, 4, 4, 2, 2, 2, 5, 6, 6, 7, 7, 8, 2, 6, 6]
        .into_iter()
        .enumerate()
    {
        assert_eq!(
            eval(
                &mut vm,
                "faults",
                &["I"],
                "I",
                vec![Word::from(which as i32)]
            ),
            vec![Word::from(expected)],
            "fault case {which}"
        );
        assert_eq!(vm.stack_depth(), 0);
    }
    assert_eq!(
        eval(&mut vm, "finallyFault", &[], "I", vec![]),
        vec![Word::from(1)]
    );
    assert_eq!(
        eval(&mut vm, "inheritedTypes", &[], "I", vec![]),
        vec![Word::from(18)]
    );
}

#[test]
fn failed_class_initialization_is_sticky_and_retains_its_exception() {
    let mut vm = runtime();
    assert_eq!(
        eval(&mut vm, "failedInit", &[], "I", vec![]),
        vec![Word::from(1)]
    );
    vm.collect();
    assert_eq!(
        eval(&mut vm, "failedInit", &[], "I", vec![]),
        vec![Word::from(2)]
    );
    assert_eq!(
        eval(&mut vm, "failedInitError", &[], "I", vec![]),
        vec![Word::from(1)]
    );
    vm.collect();
    assert_eq!(
        eval(&mut vm, "failedInitError", &[], "I", vec![]),
        vec![Word::from(2)]
    );
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
fn text_selection_uses_the_nearest_click_owner_without_bubbling_guest_clicks() {
    let mut vm = runtime();
    vm.launch().unwrap();
    let original = vm.snapshot().unwrap();
    let display = Word::Ref(original.children[0].handle);
    let increment = original.children[1].view.listener;
    let decrement = original.children[2].view.listener;
    let row = vm.heap.instance("Landroid/widget/FrameLayout;").unwrap();
    let nested = vm.heap.instance("Landroid/widget/LinearLayout;").unwrap();
    let label = vm.heap.instance("Landroid/widget/TextView;").unwrap();
    vm.heap
        .get_mut(row)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .listener = increment;
    vm.heap
        .get_mut(row)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .children = vec![nested];
    vm.heap
        .get_mut(nested)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .children = vec![label];
    vm.heap.get_mut(label).unwrap().view.as_mut().unwrap().text = "Saved item".into();
    vm.root = Some(row);
    assert!(
        !vm.click(label.reference().unwrap()).unwrap(),
        "guest click bubbled"
    );
    assert!(vm.click_text("Saved item").unwrap());
    assert_eq!(
        vm.heap.get(display).unwrap().view.as_ref().unwrap().text,
        "1"
    );
    vm.heap
        .get_mut(nested)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .listener = decrement;
    assert!(vm.click_text("Saved item").unwrap());
    assert_eq!(
        vm.heap.get(display).unwrap().view.as_ref().unwrap().text,
        "0"
    );
    vm.heap
        .get_mut(nested)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .enabled = false;
    assert!(
        !vm.click_text("Saved item").unwrap(),
        "disabled owner was bypassed"
    );
    vm.heap
        .get_mut(nested)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .enabled = true;
    vm.heap
        .get_mut(nested)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .visible = 8;
    assert!(
        vm.click_text("Saved item").is_err(),
        "hidden label was selected"
    );
    vm.heap
        .get_mut(nested)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .visible = 0;
    let data = vm.heap.get_mut(label).unwrap().view.as_mut().unwrap();
    data.text = String::new();
    data.content_description = Some("Saved item".into());
    assert!(vm.click_text("Saved item").unwrap());
    assert_eq!(
        vm.heap.get(display).unwrap().view.as_ref().unwrap().text,
        "-1"
    );
    let mut fields = vec![];
    for _ in 0..4 {
        fields.push(vm.heap.instance("Landroid/widget/EditText;").unwrap());
    }
    vm.heap
        .get_mut(fields[1])
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .enabled = false;
    vm.heap
        .get_mut(nested)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .visible = 8;
    vm.heap
        .get_mut(nested)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .children = vec![fields[2]];
    vm.heap
        .get_mut(row)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .children = vec![fields[0], fields[1], nested, fields[3]];
    vm.input("first").unwrap();
    vm.input_at(1, "second").unwrap();
    assert!(vm.input_at(2, "wrong").is_err());
    for (field, expected) in fields.iter().zip(["first", "", "", "second"]) {
        assert_eq!(
            vm.heap.get(*field).unwrap().view.as_ref().unwrap().text,
            expected
        );
    }
    assert_eq!(vm.stack_depth(), 0);
    vm.close().unwrap();
}

#[test]
fn system_exit_returns_guest_status_without_a_host_exit() {
    let mut vm = runtime();
    for code in [0, 23] {
        let error = vm
            .invoke(
                Method {
                    class: "Ljava/lang/System;".into(),
                    name: "exit".into(),
                    parameters: vec!["I".into()],
                    returns: "V".into(),
                },
                vec![Word::from(code)],
                false,
            )
            .unwrap_err();
        assert_eq!(error.downcast_ref::<GuestExit>().unwrap().code(), code);
        assert_eq!(vm.stack_depth(), 0);
    }
}
