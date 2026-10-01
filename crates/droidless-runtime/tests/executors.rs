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
fn invoke(
    vm: &mut Runtime,
    name: &str,
    returns: &str,
    args: Vec<Word>,
) -> anyhow::Result<Vec<Word>> {
    vm.invoke(
        Method {
            class: "Lorg/droidless/scheduling/FutureContract;".into(),
            name: name.into(),
            parameters: args.iter().map(|_| "I".into()).collect(),
            returns: returns.into(),
        },
        args,
        false,
    )
}
fn call(vm: &mut Runtime, name: &str) {
    invoke(vm, name, "V", vec![]).unwrap();
}
fn number(vm: &mut Runtime, name: &str) -> i32 {
    invoke(vm, name, "I", vec![]).unwrap()[0].int().unwrap()
}
fn text(vm: &mut Runtime, name: &str) -> String {
    let word = invoke(vm, name, "Ljava/lang/String;", vec![]).unwrap()[0];
    vm.heap.text(word).unwrap().into()
}
fn parameter(vm: &mut Runtime, name: &str, value: i32, returns: &str) -> Vec<Word> {
    invoke(vm, name, returns, vec![Word::from(value)]).unwrap()
}
fn prepare(vm: &mut Runtime, mode: i32) {
    parameter(vm, "prepare", mode, "V");
}

#[test]
fn queued_pool_identity_results_waits_cancellation_shutdown_and_failure_contracts() {
    let mut vm = runtime();
    prepare(&mut vm, 0);
    assert_eq!(number(&mut vm, "state"), 0, "submit executed inline");
    assert_eq!(number(&mut vm, "validation"), 5);
    let error = invoke(&mut vm, "firstValue", "Ljava/lang/String;", vec![]).unwrap_err();
    assert!(format!("{error:#}").contains("unsupported blocking wait on the main thread"));
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 1);
    assert_eq!(
        text(&mut vm, "events"),
        "",
        "single pool bypassed blocked task"
    );
    let before = vm.instructions;
    vm.poll_messages().unwrap();
    assert_eq!(vm.instructions, before, "blocked executor spun");
    call(&mut vm, "shutdown");
    assert_eq!(number(&mut vm, "shutdownState"), 1);
    assert_eq!(number(&mut vm, "reject"), 1);
    assert_eq!(number(&mut vm, "zeroAwait"), 0);
    call(&mut vm, "feed");
    vm.poll_messages().unwrap();
    vm.collect();
    assert_eq!(text(&mut vm, "firstValue"), "payload");
    assert_eq!(text(&mut vm, "secondValue"), "second");
    assert_eq!(number(&mut vm, "sameThread"), 1);
    assert_eq!(number(&mut vm, "terminated"), 1);
    assert_eq!(number(&mut vm, "zeroAwait"), 1);
    assert_eq!(number(&mut vm, "completedVariants"), 1);
    call(&mut vm, "prepareRunnableVariants");
    vm.collect();
    vm.poll_messages().unwrap();
    vm.collect();
    assert_eq!(number(&mut vm, "runnableVariants"), 1);
    call(&mut vm, "shutdown");
    vm.poll_messages().unwrap();

    prepare(&mut vm, 1);
    vm.collect();
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "done"), 0);
    assert_eq!(text(&mut vm, "secondValue"), "second");
    assert_eq!(number(&mut vm, "sameThread"), 0);
    parameter(&mut vm, "waitFor", 0, "V");
    vm.poll_messages().unwrap();
    vm.collect();
    assert_eq!(number(&mut vm, "waitState"), 1);
    call(&mut vm, "feed");
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "waitState"), 2);
    call(&mut vm, "shutdown");
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "terminated"), 1);

    for kind in [0, 1, 2, 3] {
        prepare(&mut vm, 0);
        vm.poll_messages().unwrap();
        if kind == 2 {
            call(&mut vm, "shutdown");
        }
        parameter(&mut vm, "waitFor", kind, "V");
        vm.poll_messages().unwrap();
        vm.collect();
        assert_eq!(number(&mut vm, "waitState"), 1);
        if kind == 0 {
            call(&mut vm, "interruptWaiter");
            vm.poll_messages().unwrap();
            assert_eq!(number(&mut vm, "waitState"), 4);
        } else {
            if kind != 3 {
                vm.advance_time(9).unwrap();
                assert_eq!(number(&mut vm, "waitState"), 1);
            }
            vm.advance_time(1).unwrap();
            assert_eq!(number(&mut vm, "waitState"), 3);
        }
        call(&mut vm, "feed");
        vm.poll_messages().unwrap();
        call(&mut vm, "shutdown");
        vm.poll_messages().unwrap();
    }
    prepare(&mut vm, 0);
    vm.poll_messages().unwrap();
    call(&mut vm, "shutdown");
    parameter(&mut vm, "waitFor", 2, "V");
    vm.poll_messages().unwrap();
    call(&mut vm, "feed");
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "waitState"), 2);

    for interrupt in [0, 1] {
        prepare(&mut vm, 8);
        vm.poll_messages().unwrap();
        vm.collect();
        assert_eq!(
            parameter(&mut vm, "cancel", interrupt, "I")[0]
                .int()
                .unwrap(),
            1
        );
        assert_eq!(
            parameter(&mut vm, "cancel", interrupt, "I")[0]
                .int()
                .unwrap(),
            0
        );
        assert_eq!(number(&mut vm, "cancelled"), 1);
        assert_eq!(number(&mut vm, "callbacks"), 1);
        assert_eq!(
            number(&mut vm, "callbackWorker"),
            0,
            "cancel done ran on worker"
        );
        vm.poll_messages().unwrap();
        assert_eq!(number(&mut vm, "finished"), interrupt);
        if interrupt == 0 {
            call(&mut vm, "feed");
            vm.poll_messages().unwrap();
        }
        assert_eq!(number(&mut vm, "finished"), 1);
        assert_eq!(number(&mut vm, "callbacks"), 1);
        assert_eq!(number(&mut vm, "cancelled"), 1);
        call(&mut vm, "shutdown");
        vm.poll_messages().unwrap();
    }
    prepare(&mut vm, 8);
    parameter(&mut vm, "cancel", 1, "I");
    vm.collect();
    vm.poll_messages().unwrap();
    assert_eq!(
        number(&mut vm, "state"),
        0,
        "cancelled queued body executed"
    );
    assert_eq!(number(&mut vm, "callbacks"), 1);
    call(&mut vm, "shutdown");
    vm.poll_messages().unwrap();

    prepare(&mut vm, 4);
    vm.collect();
    vm.poll_messages().unwrap();
    vm.collect();
    assert_eq!(
        number(&mut vm, "failed"),
        1,
        "ExecutionException lost original cause"
    );
    assert_eq!(text(&mut vm, "secondValue"), "second");
    assert_eq!(number(&mut vm, "sameThread"), 1);
    call(&mut vm, "shutdown");
    vm.poll_messages().unwrap();

    prepare(&mut vm, 6);
    vm.poll_messages().unwrap();
    assert_eq!(
        number(&mut vm, "stopNow"),
        1,
        "shutdownNow lost queued Runnable identities or auto-cancelled them"
    );
    assert_eq!(number(&mut vm, "terminated"), 0);
    vm.collect();
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "terminated"), 1);
    assert_eq!(number(&mut vm, "finished"), 1);
    assert_eq!(text(&mut vm, "events"), "");
    assert_eq!(number(&mut vm, "failed"), 1);

    prepare(&mut vm, 7);
    vm.collect();
    vm.poll_messages().unwrap();
    call(&mut vm, "feed");
    vm.poll_messages().unwrap();
    vm.collect();
    assert_eq!(text(&mut vm, "firstValue"), "payload");
    assert_eq!(number(&mut vm, "firstAlive"), 0);
    call(&mut vm, "shutdown");

    prepare(&mut vm, 2);
    vm.poll_messages().unwrap();
    call(&mut vm, "feed");
    vm.poll_messages().unwrap();
    call(&mut vm, "runNext");
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "sameThread"), 1);
    let first_name = text(&mut vm, "threadName");
    vm.advance_time(59_999).unwrap();
    assert_eq!(number(&mut vm, "firstAlive"), 1);
    vm.advance_time(1).unwrap();
    assert_eq!(number(&mut vm, "firstAlive"), 0);
    call(&mut vm, "runNext");
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "sameThread"), 0);
    assert_ne!(text(&mut vm, "threadName"), first_name);
    call(&mut vm, "shutdown");
    vm.poll_messages().unwrap();

    prepare(&mut vm, 9);
    let error = vm.poll_messages().unwrap_err();
    assert!(format!("{error:#}").contains("execute failure"));
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    vm.poll_messages().unwrap();
    assert_eq!(text(&mut vm, "secondValue"), "second");
    assert_eq!(number(&mut vm, "sameThread"), 0);
    call(&mut vm, "shutdown");
    vm.poll_messages().unwrap();

    prepare(&mut vm, 10);
    let error = vm.poll_messages().unwrap_err();
    assert!(format!("{error:#}").contains("getStackTrace"));
    assert_eq!(vm.stack_depth(), 0);
    let error = invoke(&mut vm, "firstValue", "Ljava/lang/String;", vec![]).unwrap_err();
    assert!(format!("{error:#}").contains("Future task aborted"));
    call(&mut vm, "shutdown");
    vm.poll_messages().unwrap();

    prepare(&mut vm, 11);
    vm.poll_messages().unwrap();
    call(&mut vm, "feed");
    vm.collect();
    vm.advance_time(9).unwrap();
    assert_eq!(number(&mut vm, "done"), 0);
    vm.advance_time(1).unwrap();
    assert_eq!(number(&mut vm, "done"), 1);
    assert_eq!(text(&mut vm, "firstValue"), "payload");
    call(&mut vm, "shutdown");
    vm.poll_messages().unwrap();

    prepare(&mut vm, 0);
    vm.poll_messages().unwrap();
    vm.collect();
    vm.close().unwrap();
    assert_eq!(number(&mut vm, "cancelled"), 1);
    assert_eq!(number(&mut vm, "firstAlive"), 0);
    assert_eq!(number(&mut vm, "terminated"), 1);
    assert_eq!(vm.poll_messages().unwrap(), 0);
    assert_eq!(vm.stack_depth(), 0);

    let mut vm = runtime();
    let error = invoke(&mut vm, "prepareCapacity", "V", vec![]).unwrap_err();
    assert!(format!("{error:#}").contains("executor queue limit reached"));
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    call(&mut vm, "shutdown");
    for _ in 0..258 {
        if number(&mut vm, "terminated") == 1 {
            break;
        }
        vm.poll_messages().unwrap();
    }
    assert_eq!(number(&mut vm, "terminated"), 1);
    vm.close().unwrap();
    let mut vm = runtime();
    prepare(&mut vm, 7);
    vm.collect();
    vm.close().unwrap();
    assert_eq!(number(&mut vm, "state"), 0);
    assert_eq!(
        number(&mut vm, "cancelled"),
        1,
        "unstarted Thread Future survived shutdown"
    );
}

#[test]
fn future_main_handler_delivery_cancellation_and_worker_ui_guard() {
    let mut vm = runtime();
    vm.click_text("Start future worker").unwrap();
    vm.collect();
    vm.poll_messages().unwrap();
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Future waiting for input"
    );
    vm.click_text("Deliver future input").unwrap();
    vm.collect();
    assert_eq!(vm.poll_messages().unwrap(), 1);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Future result: payload"
    );
    vm.click_text("Start future worker").unwrap();
    vm.poll_messages().unwrap();
    vm.collect();
    vm.click_text("Cancel future worker").unwrap();
    assert_eq!(vm.poll_messages().unwrap(), 1);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Future worker cancelled"
    );
    vm.click_text("Deliver future input").unwrap();
    vm.poll_messages().unwrap();
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Future worker cancelled"
    );
    let activity = vm.activity.unwrap();
    vm.invoke(
        Method {
            class: "Lorg/droidless/scheduling/MainActivity;".into(),
            name: "startUnsafeFuture".into(),
            parameters: vec![],
            returns: "V".into(),
        },
        vec![activity],
        true,
    )
    .unwrap();
    let error = vm.poll_messages().unwrap_err();
    assert!(format!("{error:#}").contains("unsupported UI access from a guest worker"));
    assert_eq!(vm.stack_depth(), 0);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Future worker cancelled"
    );
    vm.click_text("Start future worker").unwrap();
    vm.poll_messages().unwrap();
    vm.collect();
    vm.close().unwrap();
    assert_eq!(vm.poll_messages().unwrap(), 0);
    assert_eq!(vm.stack_depth(), 0);
}
