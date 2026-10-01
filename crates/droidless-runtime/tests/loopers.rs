use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

fn runtime(mode: i32) -> Runtime {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/scheduling.apk")).unwrap(),
    )
    .unwrap();
    call(
        &mut vm,
        "prepare",
        &["I", "Ljava/lang/Runnable;"],
        "V",
        vec![Word::from(mode), Word::ZERO],
    );
    assert_eq!(number(&mut vm, "state"), 0, "worker ran inline");
    vm.collect();
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 1);
    assert_eq!(number(&mut vm, "validations"), 7);
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
            class: "Lorg/droidless/scheduling/WorkerLooperContract;".into(),
            name: name.into(),
            parameters: parameters.iter().map(|s| (*s).into()).collect(),
            returns: returns.into(),
        },
        args,
        false,
    )
    .unwrap()
}
fn action(vm: &mut Runtime, name: &str) {
    call(vm, name, &[], "V", vec![]);
}
fn number(vm: &mut Runtime, name: &str) -> i32 {
    call(vm, name, &[], "I", vec![])[0].int().unwrap()
}
fn text(vm: &mut Runtime, name: &str) -> String {
    let word = call(vm, name, &[], "Ljava/lang/String;", vec![])[0];
    vm.heap.text(word).unwrap().into()
}
fn quit(vm: &mut Runtime, safely: i32) {
    call(vm, "quit", &["I"], "V", vec![Word::from(safely)]);
}

#[test]
fn prepared_worker_loops_route_suspend_resume_quit_and_unwind() {
    let mut vm = runtime(0);
    action(&mut vm, "preparePeer");
    vm.poll_messages().unwrap();
    action(&mut vm, "enqueuePeer");
    action(&mut vm, "enqueueOrdered");
    vm.collect();
    assert_eq!(vm.advance_time(9).unwrap(), 1);
    assert_eq!(text(&mut vm, "readEvents"), "");
    assert_eq!(text(&mut vm, "readMainEvents"), "M");
    assert_eq!(text(&mut vm, "readPeerEvents"), "P");
    vm.advance_time(1).unwrap();
    assert_eq!(text(&mut vm, "readEvents"), "AC9C7H");
    vm.advance_time(1).unwrap();
    assert_eq!(text(&mut vm, "readEvents"), "AC9C7HZ");
    vm.collect();
    let before = vm.instructions;
    vm.poll_messages().unwrap();
    assert_eq!(vm.instructions, before, "empty Looper spun");
    action(&mut vm, "enqueueBlocking");
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 2);
    vm.collect();
    let before = vm.instructions;
    vm.poll_messages().unwrap();
    assert_eq!(vm.instructions, before, "waiting callback spun");
    action(&mut vm, "feed");
    assert_eq!(vm.poll_messages().unwrap(), 1);
    assert_eq!(number(&mut vm, "state"), 3);
    assert_eq!(text(&mut vm, "readOutput"), "kept-payload:input");
    assert_eq!(text(&mut vm, "readEvents"), "AC9C7HZC10BF");
    assert_eq!(text(&mut vm, "readMainEvents"), "MR");
    action(&mut vm, "enqueueChangedTarget");
    vm.poll_messages().unwrap();
    assert!(text(&mut vm, "readEvents").ends_with("C9"));
    action(&mut vm, "interrupt");
    let before = vm.instructions;
    vm.poll_messages().unwrap();
    assert_eq!(vm.instructions, before, "interrupt woke an idle Looper");
    action(&mut vm, "enqueueQuit");
    quit(&mut vm, 1);
    quit(&mut vm, 0); // The first quitSafely keeps messages already due.
    assert_eq!(number(&mut vm, "postRejected"), 1);
    vm.collect();
    vm.poll_messages().unwrap();
    assert!(text(&mut vm, "readEvents").ends_with("C9Q"));
    assert_eq!(number(&mut vm, "finished"), 1);
    assert_eq!(number(&mut vm, "alive"), 0);
    action(&mut vm, "enqueueNativeNoops");
    assert_eq!(vm.poll_messages().unwrap(), 1);
    assert_eq!(text(&mut vm, "readMainEvents"), "MRN");
    assert_eq!(text(&mut vm, "readPeerEvents"), "P");
    vm.poll_messages().unwrap();
    vm.poll_messages().unwrap();
    assert_eq!(text(&mut vm, "readPeerEvents"), "PP");
    action(&mut vm, "quitPeer");
    vm.advance_time(100).unwrap();
    assert!(!text(&mut vm, "readEvents").contains("BAD"));
    assert_eq!(vm.stack_depth(), 0);

    let mut vm = runtime(1);
    action(&mut vm, "enqueueOrdered");
    vm.advance_time(10).unwrap();
    assert_eq!(text(&mut vm, "readEvents"), "DADC9DC7H");
    action(&mut vm, "enqueueQuit");
    quit(&mut vm, 0);
    vm.advance_time(100).unwrap();
    assert_eq!(text(&mut vm, "readEvents"), "DADC9DC7H");
    assert_eq!(number(&mut vm, "alive"), 0);

    let mut vm = runtime(2);
    action(&mut vm, "enqueueFault");
    vm.collect();
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "caught"), 1);
    assert_eq!(text(&mut vm, "readEvents"), "RE");
    assert_eq!(number(&mut vm, "finished"), 1);
    assert_eq!(number(&mut vm, "alive"), 0);
    assert_eq!(vm.stack_depth(), 0);

    let mut vm = runtime(0);
    action(&mut vm, "enqueueBlocking");
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 2);
    vm.collect();
    vm.close().unwrap();
    assert_eq!(number(&mut vm, "alive"), 0);
    assert_eq!(vm.poll_messages().unwrap(), 0);
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();

    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/scheduling.apk")).unwrap(),
    )
    .unwrap();
    vm.launch().unwrap();
    vm.click_text("Start Looper worker").unwrap();
    vm.poll_messages().unwrap();
    assert_eq!(number(&mut vm, "state"), 2);
    vm.collect();
    vm.click_text("Deliver Looper input").unwrap();
    assert_eq!(vm.poll_messages().unwrap(), 1);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Looper result: kept-payload:input"
    );
    assert_eq!(number(&mut vm, "alive"), 0);
    vm.click_text("Start Looper worker").unwrap();
    vm.poll_messages().unwrap();
    vm.click_text("Cancel Looper worker").unwrap();
    vm.poll_messages().unwrap();
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Looper worker cancelled"
    );
    assert_eq!(number(&mut vm, "alive"), 0);
    vm.click_text("Start Looper worker").unwrap();
    vm.poll_messages().unwrap();
    vm.close().unwrap();
    assert_eq!(number(&mut vm, "alive"), 0);
    assert_eq!(vm.stack_depth(), 0);
}
