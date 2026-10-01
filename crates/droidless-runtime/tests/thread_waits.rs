use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

fn runtime() -> Runtime {
    Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/scheduling.apk")).unwrap())
        .unwrap()
}
fn invoke(
    vm: &mut Runtime,
    name: &str,
    parameters: &[&str],
    returns: &str,
    args: Vec<Word>,
) -> anyhow::Result<Vec<Word>> {
    vm.invoke(
        Method {
            class: "Lorg/droidless/scheduling/ThreadWaitContract;".into(),
            name: name.into(),
            parameters: parameters.iter().map(|s| (*s).into()).collect(),
            returns: returns.into(),
        },
        args,
        false,
    )
}
fn number(vm: &mut Runtime, name: &str) -> i32 {
    invoke(vm, name, &[], "I", vec![]).unwrap()[0]
        .int()
        .unwrap()
}
fn action(vm: &mut Runtime, name: &str) {
    invoke(vm, name, &[], "V", vec![]).unwrap();
}
fn prepare(vm: &mut Runtime, mode: i32, kind: i32) {
    invoke(
        vm,
        "prepare",
        &["I", "I", "Ljava/lang/Runnable;"],
        "V",
        vec![Word::from(mode), Word::from(kind), Word::ZERO],
    )
    .unwrap();
    assert_eq!(number(vm, "state"), 0);
    vm.collect();
    vm.poll_messages().unwrap();
}

#[test]
fn sleep_and_join_deadlines_interrupts_monitors_gc_and_shutdown() {
    let mut vm = runtime();
    assert_eq!(number(&mut vm, "validate"), 255);
    prepare(&mut vm, 0, 0);
    assert_eq!(number(&mut vm, "state"), 1);
    assert_eq!(number(&mut vm, "joined"), 1);
    assert_eq!(
        number(&mut vm, "contended"),
        0,
        "sleep released its monitor"
    );
    vm.collect();
    let before = vm.instructions;
    vm.advance_time(9).unwrap();
    assert_eq!(vm.instructions, before, "sleep/join/monitor wait spun");
    vm.advance_time(1).unwrap();
    assert_eq!(number(&mut vm, "state"), 2);
    assert_eq!(number(&mut vm, "joined"), 2);
    assert_eq!(number(&mut vm, "contended"), 1);
    assert_eq!(number(&mut vm, "finished"), 1);
    assert_eq!(number(&mut vm, "joinDead"), 1);
    let word = invoke(&mut vm, "output", &[], "Ljava/lang/String;", vec![]).unwrap()[0];
    assert_eq!(vm.heap.text(word).unwrap(), "kept-sleep");
    assert_eq!(vm.stack_depth(), 0);

    for (kind, deadline) in [(1, 5), (2, 1), (5, 5)] {
        prepare(&mut vm, 2, kind);
        assert_eq!(number(&mut vm, "joined"), 1);
        vm.collect();
        vm.advance_time(deadline - 1).unwrap();
        assert_eq!(number(&mut vm, "joined"), 1);
        vm.advance_time(1).unwrap();
        assert_eq!(number(&mut vm, "joined"), 2);
        assert_eq!(number(&mut vm, "state"), 1);
        action(&mut vm, "feed");
        vm.poll_messages().unwrap();
        assert_eq!(number(&mut vm, "alive"), 0);
    }
    prepare(&mut vm, 1, 0);
    assert_eq!(number(&mut vm, "state"), 1);
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 1);
    vm.advance_time(1).unwrap();
    assert_eq!(number(&mut vm, "state"), 2);
    assert_eq!(number(&mut vm, "joined"), 2);

    prepare(&mut vm, 3, 0);
    assert_eq!(number(&mut vm, "state"), 7);
    assert_eq!(number(&mut vm, "joined"), 2);
    prepare(&mut vm, 0, 0);
    action(&mut vm, "interruptWorker");
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 7);
    assert_eq!(number(&mut vm, "joined"), 2);
    prepare(&mut vm, 2, 0);
    action(&mut vm, "interruptJoiner");
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "joined"), 7);
    action(&mut vm, "feed");
    vm.poll_messages().unwrap();
    prepare(&mut vm, 2, 4);
    assert_eq!(number(&mut vm, "joined"), 7);
    action(&mut vm, "feed");
    vm.poll_messages().unwrap();

    prepare(&mut vm, 2, 3); // API-21 overflow converts this join into an indefinite wait.
    vm.advance_time(100).unwrap();
    assert_eq!(number(&mut vm, "joined"), 1);
    let error = invoke(&mut vm, "joinOnMain", &[], "V", vec![]).unwrap_err();
    assert!(format!("{error:#}").contains("unsupported blocking wait on the main thread"));
    assert_eq!(vm.stack_depth(), 0);
    action(&mut vm, "feed");
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "joined"), 2);
    assert_eq!(number(&mut vm, "joinDead"), 1);

    prepare(&mut vm, 0, 0);
    vm.collect();
    vm.close().unwrap();
    assert_eq!(number(&mut vm, "alive"), 0);
    assert_eq!(vm.poll_messages().unwrap(), 0);
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();

    let mut vm = runtime();
    vm.launch().unwrap();
    vm.click_text("Sleep, join and finish").unwrap();
    vm.poll_messages().unwrap();
    vm.collect();
    assert_eq!(vm.advance_time(9).unwrap(), 0);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Waiting for sleep and join"
    );
    assert_eq!(vm.advance_time(1).unwrap(), 1);
    assert!(vm.activity.is_none());
    assert_eq!(number(&mut vm, "alive"), 0);
    assert_eq!(number(&mut vm, "finished"), 1);
    assert_eq!(vm.stack_depth(), 0);
}
