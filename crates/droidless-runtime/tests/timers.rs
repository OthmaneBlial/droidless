use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{
    Runtime,
    heap::{Word, bits64},
};

fn runtime() -> Runtime {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/scheduling.apk")).unwrap(),
    )
    .unwrap();
    vm.launch().unwrap();
    vm
}
fn call(vm: &mut Runtime, name: &str, returns: &str) -> Vec<Word> {
    vm.invoke(
        Method {
            class: "Lorg/droidless/scheduling/TimerContract;".into(),
            name: name.into(),
            parameters: vec![],
            returns: returns.into(),
        },
        vec![],
        false,
    )
    .unwrap()
}
fn prepare(vm: &mut Runtime, mode: i32) {
    vm.invoke(
        Method {
            class: "Lorg/droidless/scheduling/TimerContract;".into(),
            name: "prepare".into(),
            parameters: vec!["I".into()],
            returns: "V".into(),
        },
        vec![Word::from(mode)],
        false,
    )
    .unwrap();
}
fn number(vm: &mut Runtime, name: &str) -> i32 {
    call(vm, name, "I")[0].int().unwrap()
}
fn long(vm: &mut Runtime, name: &str) -> i64 {
    bits64(&call(vm, name, "J")).unwrap() as i64
}
fn events(vm: &mut Runtime) -> String {
    let word = call(vm, "readEvents", "Ljava/lang/String;")[0];
    vm.heap.text(word).unwrap().into()
}

#[test]
fn timer_deadlines_serial_workers_cancellation_faults_gc_capacity_and_main_delivery() {
    let mut vm = runtime();
    assert_eq!(number(&mut vm, "validation"), 11);
    for mode in [0, 7] {
        prepare(&mut vm, mode);
        vm.collect();
        assert_eq!(events(&mut vm), "", "Timer task executed inline");
        let before = vm.instructions;
        vm.advance_time(9).unwrap();
        assert_eq!(
            vm.instructions, before,
            "Timer executed before its deadline"
        );
        assert_eq!(events(&mut vm), "");
        vm.advance_time(1).unwrap();
        assert_eq!(events(&mut vm), "A");
        assert_eq!(long(&mut vm, "firstOffset"), 10);
        assert_eq!(long(&mut vm, "wallElapsed"), 10);
        assert_eq!(number(&mut vm, "rejectReuse"), 1);
        assert_eq!(
            number(&mut vm, "cancelTask"),
            0,
            "completed one-shot cancellation"
        );
        assert_eq!(
            number(&mut vm, "alive"),
            1,
            "Timer worker ended after one task"
        );
        call(&mut vm, "cancelTimer", "V");
        vm.poll_messages().unwrap();
        assert_eq!(number(&mut vm, "alive"), 0);
    }
    for (mode, expected, first, last) in [(1, "A", 10, 10), (2, "AAA", 10, 30), (3, "AAA", -25, -5)]
    {
        prepare(&mut vm, mode);
        vm.collect();
        vm.advance_time(if mode == 3 { 0 } else { 35 }).unwrap();
        assert_eq!(events(&mut vm), expected);
        assert_eq!(long(&mut vm, "firstOffset"), first);
        assert_eq!(long(&mut vm, "lastOffset"), last);
        assert_eq!(number(&mut vm, "cancelTask"), 1);
        assert_eq!(number(&mut vm, "cancelTask"), 0);
        assert_eq!(number(&mut vm, "purge"), 1);
        assert_eq!(number(&mut vm, "purge"), 0);
        assert_eq!(number(&mut vm, "rejectReuse"), 1);
        vm.advance_time(100).unwrap();
        assert_eq!(events(&mut vm), expected);
    }
    for mode in [8, 9] {
        prepare(&mut vm, mode);
        vm.poll_messages().unwrap();
        assert_eq!(events(&mut vm), "A");
        assert_eq!(
            long(&mut vm, "firstOffset"),
            0,
            "past Date fixed-delay clamp"
        );
        call(&mut vm, "cancelTimer", "V");
    }
    prepare(&mut vm, 0);
    assert_eq!(number(&mut vm, "cancelTask"), 1);
    assert_eq!(number(&mut vm, "purge"), 1);
    vm.advance_time(20).unwrap();
    assert_eq!(events(&mut vm), "");
    prepare(&mut vm, 0);
    call(&mut vm, "cancelTimer", "V");
    assert_eq!(number(&mut vm, "rejectSchedule"), 1);
    vm.advance_time(20).unwrap();
    assert_eq!(events(&mut vm), "");

    for cancel_active in [false, true] {
        prepare(&mut vm, 4);
        vm.poll_messages().unwrap();
        assert_eq!(number(&mut vm, "state"), 1);
        assert_eq!(
            events(&mut vm),
            "",
            "second task bypassed a blocked Timer task"
        );
        vm.collect();
        if cancel_active {
            call(&mut vm, "cancelTimer", "V");
        }
        let before = vm.instructions;
        vm.poll_messages().unwrap();
        assert_eq!(
            vm.instructions, before,
            "blocked task spun or cancel interrupted it"
        );
        call(&mut vm, "feed", "V");
        vm.poll_messages().unwrap();
        assert_eq!(number(&mut vm, "state"), 2);
        assert_eq!(events(&mut vm), if cancel_active { "A" } else { "AB" });
        assert_eq!(number(&mut vm, "alive"), i32::from(!cancel_active));
    }
    prepare(&mut vm, 5);
    vm.collect();
    let error = vm.poll_messages().unwrap_err();
    assert!(format!("{error:#}").contains("Timer task failure"));
    assert_eq!(vm.stack_depth(), 0);
    assert_eq!(number(&mut vm, "rejectSchedule"), 1);
    vm.advance_time(100).unwrap();
    assert_eq!(events(&mut vm), "");
    for mode in [6, 10] {
        prepare(&mut vm, mode);
        vm.advance_time(100).unwrap();
        assert_eq!(events(&mut vm), "A");
        if mode == 6 {
            assert_eq!(number(&mut vm, "cancelResult"), 1);
        } else {
            assert_eq!(number(&mut vm, "alive"), 0);
        }
    }

    vm.click_text("Start background timer").unwrap();
    vm.collect();
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Background timer queued"
    );
    assert_eq!(vm.advance_time(1499).unwrap(), 0);
    assert_eq!(vm.advance_time(1).unwrap(), 1);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Background tick 1"
    );
    vm.click_text("Cancel background timer").unwrap();
    assert_eq!(vm.advance_time(5000).unwrap(), 0);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Background timer cancelled"
    );
    vm.click_text("Start background timer").unwrap();
    for _ in 0..3 {
        assert_eq!(vm.advance_time(1500).unwrap(), 1);
    }
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Background timer done: 3"
    );
    vm.click_text("Start background timer").unwrap();
    prepare(&mut vm, 4);
    vm.poll_messages().unwrap();
    vm.collect();
    vm.close().unwrap();
    assert_eq!(number(&mut vm, "rejectSchedule"), 1);
    assert_eq!(vm.advance_time(5000).unwrap(), 0);
    assert_eq!(events(&mut vm), "");

    let mut vm = runtime();
    let error = vm
        .invoke(
            Method {
                class: "Lorg/droidless/scheduling/TimerContract;".into(),
                name: "prepareCapacity".into(),
                parameters: vec![],
                returns: "V".into(),
            },
            vec![],
            false,
        )
        .unwrap_err();
    assert!(format!("{error:#}").contains("Timer task limit reached"));
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    call(&mut vm, "refillCapacity", "V");
    call(&mut vm, "cancelTimer", "V");
    vm.poll_messages().unwrap();
    vm.collect();
    vm.close().unwrap();
}
