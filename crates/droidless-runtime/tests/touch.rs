use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{
    Runtime,
    heap::{Word, wide},
};
fn call(
    vm: &mut Runtime,
    name: &str,
    parameters: &[&str],
    returns: &str,
    args: Vec<Word>,
) -> anyhow::Result<Vec<Word>> {
    vm.invoke(
        Method {
            class: "Lorg/droidless/touch/MainActivity;".into(),
            name: name.into(),
            parameters: parameters.iter().map(|s| (*s).into()).collect(),
            returns: returns.into(),
        },
        args,
        false,
    )
}
fn runtime() -> Runtime {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/touch.apk")).unwrap())
            .unwrap();
    vm.launch().unwrap();
    vm
}
fn metrics(vm: &mut Runtime) -> Vec<i32> {
    let word = call(vm, "metrics", &[], "Ljava/lang/String;", vec![]).unwrap()[0];
    vm.heap
        .text(word)
        .unwrap()
        .split(':')
        .map(|s| s.parse().unwrap())
        .collect()
}
fn mode(vm: &mut Runtime, value: i32) {
    call(vm, "mode", &["I"], "V", vec![Word::from(value)]).unwrap();
}
fn tap(vm: &mut Runtime, x: f32, y: f32) {
    vm.touch(0, x, y).unwrap();
    vm.advance_time(20).unwrap();
    vm.touch(1, x, y).unwrap();
}
fn float(vm: &mut Runtime, name: &str) -> f32 {
    f32::from_bits(call(vm, name, &[], "F", vec![]).unwrap()[0].int().unwrap() as u32)
}

#[test]
fn guest_layout_metadata_geometry_root_touch_and_fault_cleanup() {
    fn state(vm: &mut Runtime) -> Vec<i32> {
        let word = call(vm, "layoutState", &[], "Ljava/lang/String;", vec![]).unwrap()[0];
        vm.heap
            .text(word)
            .unwrap()
            .split(':')
            .map(|value| value.parse().unwrap())
            .collect()
    }
    let mut vm = runtime();
    assert_eq!(
        call(&mut vm, "metadataContract", &[], "I", vec![]).unwrap(),
        [Word::from(1)]
    );
    assert!(
        format!(
            "{:#}",
            call(&mut vm, "mergeWithoutParent", &[], "V", vec![]).unwrap_err()
        )
        .contains("merge requires an attached parent")
    );
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "installLayout", &[], "V", vec![]).unwrap();
    assert!(vm.touch(0, f32::NAN, 30.0).is_err());
    assert_eq!(state(&mut vm), [2, 0, 0, 0, 0]);
    tap(&mut vm, 40.0, 30.0);
    assert_eq!(state(&mut vm), [2, 1, 1, 1, 0]);
    for _ in 0..3 {
        vm.layout_snapshot().unwrap();
    }
    assert_eq!(state(&mut vm), [2, 1, 1, 1, 0]);
    call(&mut vm, "configureLayout", &["I"], "V", vec![Word::from(1)]).unwrap();
    tap(&mut vm, 40.0, 30.0);
    assert_eq!(state(&mut vm), [2, 2, 2, 1, 1]);
    for _ in 0..3 {
        let tree = vm.layout_snapshot().unwrap();
        assert_eq!(tree.rect.x, 0.5);
        assert_eq!(tree.children[1].rect.x, 10.5);
    }
    assert_eq!(state(&mut vm), [2, 2, 2, 1, 1]);
    call(&mut vm, "configureLayout", &["I"], "V", vec![Word::from(2)]).unwrap();
    assert!(format!("{:#}", vm.layout_snapshot().unwrap_err()).contains("custom layout failed"));
    assert_eq!(vm.stack_depth(), 0);
    vm.collect();
    call(&mut vm, "configureLayout", &["I"], "V", vec![Word::from(0)]).unwrap();
    tap(&mut vm, 40.0, 30.0);
    assert_eq!(state(&mut vm), [2, 4, 4, 2, 1]);
    vm.close().unwrap();
}

#[test]
fn compiled_touch_dispatch_gestures_timing_gc_and_failure_cleanup() {
    let mut vm = runtime();
    assert!(vm.touch_input_enabled());
    assert_eq!(
        call(&mut vm, "measureContract", &[], "I", vec![]).unwrap(),
        [Word::from(1)]
    );
    assert_eq!(
        call(&mut vm, "velocityContract", &[], "I", vec![]).unwrap(),
        [Word::from(1)]
    );
    for (name, diagnostic) in [
        ("recycledVelocity", "recycled VelocityTracker"),
        ("invalidVelocity", "invalid velocity units or maximum"),
    ] {
        assert!(
            format!("{:#}", call(&mut vm, name, &[], "V", vec![]).unwrap_err())
                .contains(diagnostic)
        );
        assert_eq!(vm.stack_depth(), 0);
        vm.collect();
    }
    assert_eq!(
        call(&mut vm, "motionContract", &[], "I", vec![]).unwrap(),
        [Word::from(1)]
    );
    tap(&mut vm, 40.0, 120.0);
    assert_eq!(metrics(&mut vm), [0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 2]);
    assert_eq!(float(&mut vm, "localX"), 30.0);
    assert_eq!(float(&mut vm, "rawX"), 40.0);
    mode(&mut vm, 1);
    tap(&mut vm, 40.0, 120.0);
    assert_eq!(metrics(&mut vm)[9..], [1, 4]);
    mode(&mut vm, 2);
    tap(&mut vm, 40.0, 120.0);
    assert_eq!(metrics(&mut vm)[9..], [1, 4]);
    mode(&mut vm, 0);
    vm.touch(0, 40.0, 120.0).unwrap();
    vm.touch(2, 350.0, 300.0).unwrap();
    vm.touch(1, 350.0, 300.0).unwrap();
    assert_eq!(metrics(&mut vm)[9], 1);
    vm.touch(0, 40.0, 120.0).unwrap();
    mode(&mut vm, 2);
    vm.touch(1, 40.0, 120.0).unwrap();
    assert_eq!(
        call(&mut vm, "pressed", &[], "Z", vec![]).unwrap(),
        [Word::ZERO]
    );
    mode(&mut vm, 4);
    let tree = vm.snapshot().unwrap();
    let button = &tree.children[1];
    assert_eq!(
        (button.rect.x, button.rect.y, button.view.alpha),
        (30.0, 120.0, 0.5)
    );
    tap(&mut vm, 40.0, 135.0);
    assert_eq!(metrics(&mut vm)[9], 2);
    assert_eq!(float(&mut vm, "localX"), 10.0);
    mode(&mut vm, 0);
    tap(&mut vm, 250.0, 350.0);
    assert_eq!(metrics(&mut vm)[0], 1);
    assert_eq!(metrics(&mut vm)[3..5], [1, 0]);
    vm.advance_time(279).unwrap();
    assert_eq!(metrics(&mut vm)[4], 0);
    vm.advance_time(1).unwrap();
    assert_eq!(metrics(&mut vm)[4], 1);
    vm.touch(0, 250.0, 350.0).unwrap();
    vm.advance_time(99).unwrap();
    assert_eq!(metrics(&mut vm)[1], 0);
    vm.advance_time(1).unwrap();
    assert_eq!(metrics(&mut vm)[1], 1);
    vm.advance_time(499).unwrap();
    assert_eq!(metrics(&mut vm)[2], 0);
    vm.advance_time(1).unwrap();
    assert_eq!(metrics(&mut vm)[2], 1);
    vm.touch(1, 250.0, 350.0).unwrap();
    assert_eq!(metrics(&mut vm)[3..5], [1, 1]);
    tap(&mut vm, 250.0, 350.0);
    vm.advance_time(40).unwrap();
    tap(&mut vm, 250.0, 350.0);
    vm.advance_time(300).unwrap();
    assert_eq!(metrics(&mut vm)[4..7], [1, 1, 2]);
    vm.touch(0, 340.0, 350.0).unwrap();
    vm.advance_time(20).unwrap();
    vm.touch(2, 260.0, 350.0).unwrap();
    assert_eq!(float(&mut vm, "distance"), 80.0);
    vm.advance_time(20).unwrap();
    vm.touch(1, 180.0, 350.0).unwrap();
    assert_eq!(metrics(&mut vm)[7..9], [1, 1]);
    assert!((float(&mut vm, "velocity") + 4000.0).abs() < 0.01);
    vm.touch(0, 340.0, 350.0).unwrap();
    vm.advance_time(20).unwrap();
    vm.touch(2, 180.0, 350.0).unwrap();
    vm.advance_time(200).unwrap();
    vm.touch(1, 180.0, 350.0).unwrap();
    assert_eq!(metrics(&mut vm)[8], 1, "paused drag incorrectly flung");
    vm.touch(0, 250.0, 350.0).unwrap();
    vm.touch(3, 250.0, 350.0).unwrap();
    let cancelled = metrics(&mut vm);
    vm.advance_time(1000).unwrap();
    assert_eq!(metrics(&mut vm), cancelled);
    mode(&mut vm, 3);
    assert!(vm.touch(0, 250.0, 350.0).is_err());
    assert!(!vm.touch_active());
    let failed = metrics(&mut vm);
    vm.advance_time(1000).unwrap();
    assert_eq!(metrics(&mut vm), failed, "failed callback retained timers");
    assert_eq!(vm.stack_depth(), 0);
    mode(&mut vm, 0);
    tap(&mut vm, 250.0, 350.0);
    vm.advance_time(300).unwrap();
    assert_eq!(metrics(&mut vm)[4], 2);
    assert!(vm.touch_at(0, f32::NAN, 0.0, vm.uptime_ms()).is_err());
    assert!(!vm.touch_active());
    assert!(vm.touch_at(2, 0.0, 0.0, vm.uptime_ms()).is_err());
    let mut args = wide(20);
    args.extend(wide(10));
    args.extend([Word::ZERO, Word::ZERO, Word::ZERO, Word::ZERO]);
    assert!(
        vm.invoke(
            Method {
                class: "Landroid/view/MotionEvent;".into(),
                name: "obtain".into(),
                parameters: vec![
                    "J".into(),
                    "J".into(),
                    "I".into(),
                    "F".into(),
                    "F".into(),
                    "I".into()
                ],
                returns: "Landroid/view/MotionEvent;".into()
            },
            args,
            false
        )
        .is_err()
    );
    vm.close().unwrap();
    vm.collect();
    assert!(!vm.touch_active());
}
