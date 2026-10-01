use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};
use std::path::Path;

fn runtime() -> Runtime {
    let mut vm = Runtime::new(
        Apk::parse(include_bytes!("../../../fixtures/generated/results.apk")).unwrap(),
    )
    .unwrap();
    vm.launch().unwrap();
    vm
}
fn call(vm: &mut Runtime, name: &str, returns: &str) -> Word {
    vm.invoke(
        Method {
            class: "Lorg/droidless/results/MainActivity;".into(),
            name: name.into(),
            parameters: vec![],
            returns: returns.into(),
        },
        vec![],
        false,
    )
    .unwrap()
    .first()
    .copied()
    .unwrap_or(Word::ZERO)
}
fn label(vm: &Runtime) -> String {
    vm.snapshot().unwrap().children[0].view.text.clone()
}

#[test]
fn compiled_results_copy_finish_state_gc_cancellation_stopped_callers_and_faults() {
    let mut vm = runtime();
    call(&mut vm, "checkAttachment", "V");
    let token = call(&mut vm, "windowToken", "Landroid/os/IBinder;");
    assert_ne!(token, Word::ZERO);
    vm.click_text("Open child").unwrap();
    let child = call(
        &mut vm,
        "childObject",
        "Lorg/droidless/results/MainActivity$Child;",
    );
    vm.click_text("Return result").unwrap();
    assert_eq!(call(&mut vm, "windowToken", "Landroid/os/IBinder;"), token);
    assert_eq!(label(&vm), "Result 7:-1:at finish:image/png");
    assert_eq!(call(&mut vm, "resultCount", "I"), Word::from(1));
    let log = call(&mut vm, "eventLog", "Ljava/lang/String;");
    assert!(vm.heap.text(log).unwrap().ends_with("start;result;resume;"));
    vm.collect();
    assert!(
        vm.heap.get(child).is_err(),
        "finished child remained rooted"
    );
    let data = call(&mut vm, "resultData", "Landroid/content/Intent;");
    call(&mut vm, "dropResult", "V");
    vm.collect();
    assert!(
        vm.heap.get(data).is_err(),
        "delivered result remained rooted"
    );
    vm.click_text("Open child").unwrap();
    vm.back().unwrap();
    assert_eq!(label(&vm), "Result 7:0:none");
    vm.click_text("Open without result").unwrap();
    vm.click_text("Return result").unwrap();
    assert_eq!(call(&mut vm, "resultCount", "I"), Word::from(2));

    vm.click_text("Open child").unwrap();
    vm.click_text("Open overlay").unwrap();
    vm.click_text("Finish stopped child").unwrap();
    assert_eq!(vm.activity_depth(), 2);
    assert_eq!(vm.title, "Overlay");
    assert_eq!(call(&mut vm, "resultCount", "I"), Word::from(2));
    vm.collect();
    vm.back().unwrap();
    assert_eq!(label(&vm), "Result 7:-1:at finish:image/png");
    assert_eq!(call(&mut vm, "resultCount", "I"), Word::from(3));

    vm.click_text("Open child").unwrap();
    call(&mut vm, "failResult", "V");
    assert!(
        format!("{:#}", vm.click_text("Return result").unwrap_err())
            .contains("result callback failed")
    );
    assert_eq!(vm.stack_depth(), 0);
    assert_eq!(vm.activity_depth(), 1);
    assert_eq!(
        call(
            &mut vm,
            "childObject",
            "Lorg/droidless/results/MainActivity$Child;"
        ),
        Word::ZERO
    );
    let failed = call(&mut vm, "takeFailedResult", "Landroid/content/Intent;");
    vm.collect();
    assert!(
        vm.heap.get(failed).is_err(),
        "failed callback retained temporary roots"
    );
    vm.close().unwrap();

    let mut vm = runtime();
    vm.click_text("Open child").unwrap();
    vm.click_text("Finish caller").unwrap();
    vm.click_text("Return result").unwrap();
    assert_eq!(call(&mut vm, "resultCount", "I"), Word::ZERO);
    assert!(vm.activity.is_none());

    let mut vm = runtime();
    vm.click_text("Open child").unwrap();
    call(&mut vm, "finishAfterResult", "V");
    vm.click_text("Return result").unwrap();
    assert!(vm.activity.is_none());
    let log = call(&mut vm, "eventLog", "Ljava/lang/String;");
    assert!(
        vm.heap
            .text(log)
            .unwrap()
            .ends_with("start;result;resume;pause;")
    );
}

#[test]
fn directory_choices_cancel_validate_grants_and_return_to_guest() {
    let mut vm = runtime();
    assert!(vm.complete_directory_picker(None).is_err());
    vm.click_text("Pick folder").unwrap();
    assert!(vm.directory_picker_pending());
    vm.collect();
    vm.complete_directory_picker(None).unwrap();
    assert!(!vm.directory_picker_pending());
    assert_eq!(label(&vm), "Result 404:0:none");
    vm.click_text("Pick folder").unwrap();
    let file =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/results/MainActivity.java");
    assert!(
        format!(
            "{:#}",
            vm.complete_directory_picker(Some(&file)).unwrap_err()
        )
        .contains("not a directory")
    );
    assert!(
        vm.directory_picker_pending(),
        "invalid host choice consumed request"
    );
    let directory = file.parent().unwrap();
    vm.complete_directory_picker(Some(directory)).unwrap();
    assert_eq!(
        label(&vm),
        "Result 404:-1:content://droidless.documents/tree/0"
    );
    for _ in 1..64 {
        vm.click_text("Pick folder").unwrap();
        vm.complete_directory_picker(Some(directory)).unwrap();
    }
    vm.click_text("Pick folder").unwrap();
    assert!(
        format!(
            "{:#}",
            vm.complete_directory_picker(Some(directory)).unwrap_err()
        )
        .contains("grant limit")
    );
    assert!(vm.directory_picker_pending());
    vm.complete_directory_picker(None).unwrap();
    assert_eq!(label(&vm), "Result 404:0:none");
    assert_eq!(vm.stack_depth(), 0);
    vm.close().unwrap();
}
