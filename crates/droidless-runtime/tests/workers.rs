use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

fn runtime() -> Runtime {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/scheduling.apk")).unwrap(),
    )
    .unwrap();
    vm.launch().unwrap();
    vm
}
fn call(
    vm: &mut Runtime,
    name: &str,
    parameters: &[&str],
    returns: &str,
    args: Vec<Word>,
) -> Vec<Word> {
    vm.invoke(
        Method {
            class: "Lorg/droidless/scheduling/WorkerContract;".into(),
            name: name.into(),
            parameters: parameters.iter().map(|s| (*s).into()).collect(),
            returns: returns.into(),
        },
        args,
        false,
    )
    .unwrap()
}
fn prepare(vm: &mut Runtime, mode: i32) {
    call(
        vm,
        "prepare",
        &["I", "Ljava/lang/Runnable;"],
        "V",
        vec![Word::from(mode), Word::ZERO],
    );
}
fn number(vm: &mut Runtime, name: &str) -> i32 {
    call(vm, name, &[], "I", vec![])[0].int().unwrap()
}
fn output(vm: &mut Runtime) -> String {
    let word = call(vm, "pollOutput", &[], "Ljava/lang/String;", vec![])[0];
    vm.heap.text(word).unwrap().into()
}

#[test]
fn deferred_identity_blocking_queues_gc_interrupt_and_main_delivery() {
    let mut vm = runtime();
    prepare(&mut vm, 0);
    assert_eq!(number(&mut vm, "state"), 0, "Thread.start ran inline");
    assert_eq!(number(&mut vm, "alive"), 1);
    assert_eq!(number(&mut vm, "restartRejected"), 1);
    vm.collect();
    assert_eq!(vm.poll_messages().unwrap(), 0);
    assert_eq!(number(&mut vm, "state"), 1);
    vm.collect();
    let before = vm.instructions;
    vm.poll_messages().unwrap();
    assert_eq!(
        vm.instructions, before,
        "waiting worker executed while its queue was empty"
    );
    call(&mut vm, "feed", &[], "V", vec![]);
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 2);
    assert_eq!(output(&mut vm), "kept-consumer:payload");
    vm.collect();
    call(&mut vm, "interrupt", &[], "V", vec![]);
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 7);
    assert_eq!(number(&mut vm, "finished"), 1);
    assert_eq!(number(&mut vm, "alive"), 0);
    assert_eq!(number(&mut vm, "restartRejected"), 1);
    assert_eq!(vm.stack_depth(), 0);

    prepare(&mut vm, 1);
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 1, "put did not wait for capacity");
    vm.collect();
    assert_eq!(output(&mut vm), "first");
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 2);
    assert_eq!(output(&mut vm), "second");

    vm.click_text("Start worker").unwrap();
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Worker queued"
    );
    vm.collect();
    assert_eq!(vm.poll_messages().unwrap(), 1);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Worker result: kept-consumer:payload"
    );
    assert_eq!(number(&mut vm, "state"), 7);
    assert_eq!(vm.stack_depth(), 0);
}

#[test]
fn monitor_contention_faults_bounded_spinning_and_shutdown() {
    let mut vm = runtime();
    prepare(&mut vm, 2);
    vm.poll_messages().unwrap();
    assert_eq!(
        number(&mut vm, "state"),
        0,
        "contender entered a held monitor"
    );
    vm.collect();
    assert_eq!(output(&mut vm), "locked");
    call(&mut vm, "feed", &[], "V", vec![]);
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 2);
    assert_eq!(number(&mut vm, "alive"), 0);

    prepare(&mut vm, 3);
    let error = vm.poll_messages().unwrap_err();
    assert!(format!("{error:#}").contains("worker failure"));
    assert_eq!(number(&mut vm, "alive"), 0);
    assert_eq!(number(&mut vm, "finished"), 1);
    assert_eq!(vm.stack_depth(), 0);
    assert_eq!(number(&mut vm, "canAcquire"), 1);
    prepare(&mut vm, 5);
    let error = vm.poll_messages().unwrap_err();
    assert!(
        format!("{error:#}")
            .contains("unsupported worker suspension across a synchronous native bridge")
    );
    assert_eq!(vm.stack_depth(), 0);
    assert_eq!(number(&mut vm, "alive"), 0);
    assert_eq!(number(&mut vm, "canAcquire"), 1);

    let before = vm.snapshot().unwrap().children[0].view.text.clone();
    vm.invoke(
        Method {
            class: "Lorg/droidless/scheduling/MainActivity;".into(),
            name: "startUnsafeWorker".into(),
            parameters: vec![],
            returns: "V".into(),
        },
        vec![vm.activity.unwrap()],
        true,
    )
    .unwrap();
    let error = vm.poll_messages().unwrap_err();
    assert!(format!("{error:#}").contains("unsupported UI access from a guest worker"));
    assert_eq!(vm.snapshot().unwrap().children[0].view.text, before);
    assert_eq!(vm.stack_depth(), 0);
    assert_eq!(number(&mut vm, "alive"), 0);

    prepare(&mut vm, 6);
    let before = vm.instructions;
    vm.poll_messages().unwrap();
    assert!(
        vm.instructions - before <= 65_536,
        "worker exceeded its poll slices"
    );
    assert_eq!(number(&mut vm, "alive"), 1);
    vm.close().unwrap();
    assert_eq!(number(&mut vm, "alive"), 0);
    assert_eq!(vm.poll_messages().unwrap(), 0);
    vm.collect();
    let mut waiting = runtime();
    prepare(&mut waiting, 0);
    waiting.poll_messages().unwrap();
    assert_eq!(number(&mut waiting, "state"), 1);
    waiting.collect();
    waiting.close().unwrap();
    assert_eq!(number(&mut waiting, "alive"), 0);
    assert_eq!(waiting.poll_messages().unwrap(), 0);
}
