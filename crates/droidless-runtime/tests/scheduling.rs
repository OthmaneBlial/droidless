use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};

fn runtime() -> Runtime {
    Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/scheduling.apk")).unwrap())
        .unwrap()
}
fn call(vm: &mut Runtime, name: &str, returns: &str) -> Vec<Word> {
    vm.invoke(
        Method {
            class: "Lorg/droidless/scheduling/MainActivity;".into(),
            name: name.into(),
            parameters: vec![],
            returns: returns.into(),
        },
        vec![],
        false,
    )
    .unwrap()
}
fn event_text(vm: &mut Runtime) -> String {
    let method = Method {
        class: "Lorg/droidless/scheduling/MainActivity;".into(),
        name: "readEvents".into(),
        parameters: vec![],
        returns: "Ljava/lang/String;".into(),
    };
    let word = vm.invoke(method, vec![], false).unwrap()[0];
    vm.heap.text(word).unwrap().into()
}

#[test]
fn ordered_callbacks_messages_tokens_gc_and_manual_time() {
    let mut vm = runtime();
    vm.launch().unwrap();
    call(&mut vm, "prepare", "V");
    vm.collect();
    assert_eq!(event_text(&mut vm), "");
    assert_eq!(vm.advance_time(0).unwrap(), 3);
    assert_eq!(event_text(&mut vm), "INR");
    assert_eq!(vm.advance_time(9).unwrap(), 0);
    vm.collect();
    assert_eq!(vm.advance_time(1).unwrap(), 5);
    assert_eq!(call(&mut vm, "result", "I")[0].int().unwrap(), 1);
    assert_eq!(vm.stack_depth(), 0);
    assert!(vm.advance_time(u64::MAX).is_err());
    assert_eq!(vm.uptime_ms(), 10);
    call(&mut vm, "prepareLastGc", "V");
    vm.collect();
    assert_eq!(vm.advance_time(0).unwrap(), 1);
    assert!(event_text(&mut vm).ends_with("last"));
}
#[test]
fn timer_clicks_cancellation_shutdown_and_clock_modes() {
    let mut vm = runtime();
    vm.launch().unwrap();
    vm.click_text("Start timer").unwrap();
    vm.collect();
    assert_eq!(vm.advance_time(1499).unwrap(), 0);
    assert_eq!(vm.advance_time(1).unwrap(), 1);
    assert_eq!(vm.snapshot().unwrap().children[0].view.text, "Tick 1");
    vm.click_text("Cancel timer").unwrap();
    assert_eq!(vm.advance_time(5000).unwrap(), 0);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Timer cancelled"
    );
    vm.click_text("Start timer").unwrap();
    for _ in 0..3 {
        assert_eq!(vm.advance_time(1500).unwrap(), 1);
    }
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Timer done: 3"
    );
    vm.click_text("Start timer").unwrap();
    vm.close().unwrap();
    assert_eq!(vm.advance_time(5000).unwrap(), 0);
    assert_eq!(call(&mut vm, "postAfterClose", "I")[0].int().unwrap(), 1);
    vm.use_realtime_clock().unwrap();
    assert!(vm.advance_time(0).is_err());
    let mut vm = runtime();
    vm.launch().unwrap();
    vm.click_text("Start timer").unwrap();
    vm.click_text("Finish later").unwrap();
    assert_eq!(vm.advance_time(20).unwrap(), 1);
    assert!(vm.activity.is_none());
    assert_eq!(vm.advance_time(5000).unwrap(), 0);
    assert_eq!(call(&mut vm, "postAfterClose", "I")[0].int().unwrap(), 1);
}
#[test]
fn uncaught_callbacks_and_self_posting_fail_with_clean_frames() {
    let mut vm = runtime();
    vm.launch().unwrap();
    call(&mut vm, "prepareFault", "V");
    let error = vm.advance_time(0).unwrap_err();
    assert!(format!("{error:#}").contains("timer failure"));
    assert_eq!(vm.stack_depth(), 0);
    assert_eq!(vm.advance_time(1).unwrap(), 1);
    let mut vm = runtime();
    vm.launch().unwrap();
    call(&mut vm, "prepareSpin", "V");
    assert!(format!("{:#}", vm.advance_time(0).unwrap_err()).contains("message dispatch limit"));
    assert_eq!(vm.stack_depth(), 0);
    vm.close().unwrap();
    assert_eq!(vm.advance_time(0).unwrap(), 0);
}
