use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{
    Runtime,
    heap::{Word, bits64},
    ui::Node,
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
            class: "Lorg/droidless/grids/MainActivity;".into(),
            name: name.into(),
            parameters: parameters.iter().map(|p| (*p).into()).collect(),
            returns: returns.into(),
        },
        args,
        false,
    )
}
fn grid(tree: &Node) -> &Node {
    &tree.children[1]
}
fn photo(vm: &mut Runtime, position: usize) -> usize {
    grid(&vm.layout_snapshot().unwrap()).children[position].handle
}
#[test]
fn compiled_grid_cells_notifications_geometry_clicks_gc_and_failures() {
    let mut vm =
        Runtime::new(Apk::parse(include_bytes!("../../../fixtures/generated/grids.apk")).unwrap())
            .unwrap();
    vm.launch().unwrap();
    let tree = vm.layout_snapshot().unwrap();
    let cells = &grid(&tree).children;
    assert_eq!(cells.len(), 7);
    assert_eq!(
        cells[2].view.content_description.as_deref(),
        Some("Photo 3")
    );
    assert!(cells.iter().all(|cell| cell.view.image.is_some()));
    assert_eq!(cells[1].rect.x - cells[0].rect.x, 132.0);
    assert_eq!(cells[3].rect.y - cells[0].rect.y, 104.0);
    assert_eq!(cells[0].rect.width, 124.0);
    vm.click(cells[1].handle).unwrap();
    assert_eq!(
        call(&mut vm, "clickCount", &[], "I", vec![]).unwrap(),
        [Word::ZERO]
    );
    vm.click(cells[2].handle).unwrap();
    assert_eq!(
        call(&mut vm, "clickCount", &[], "I", vec![]).unwrap(),
        [Word::from(1)]
    );
    assert_eq!(
        bits64(&call(&mut vm, "selectedId", &[], "J", vec![]).unwrap()).unwrap(),
        3_000_000_002
    );

    assert!(vm.click_text("Refresh photos").unwrap());
    assert_eq!(grid(&vm.layout_snapshot().unwrap()).children.len(), 4);
    for (mode, step, metric) in [
        (0, 92.0, "2:80:12"),
        (1, 308.0, "2:80:228"),
        (2, 200.0, "2:188:12"),
        (3, 164.0, "2:80:84"),
    ] {
        call(&mut vm, "geometry", &["I"], "V", vec![Word::from(mode)]).unwrap();
        let tree = vm.layout_snapshot().unwrap();
        let cells = &grid(&tree).children;
        assert_eq!(cells[1].rect.x - cells[0].rect.x, step);
        let value = call(&mut vm, "metrics", &[], "Ljava/lang/String;", vec![]).unwrap()[0];
        assert_eq!(vm.heap.text(value).unwrap(), metric);
    }
    for (mode, expected) in [
        (1, "cell creation failed"),
        (2, "mutation during binding"),
        (3, "duplicate cell"),
    ] {
        call(&mut vm, "failBind", &["I"], "V", vec![Word::from(mode)]).unwrap();
        let error = vm.layout_snapshot().unwrap_err();
        assert!(format!("{error:#}").contains(expected), "{error:#}");
        assert_eq!(vm.stack_depth(), 0);
        assert_eq!(
            grid(&vm.snapshot().unwrap()).children.len(),
            4,
            "failed bind replaced the old cells"
        );
        let failed = call(
            &mut vm,
            "takeFailedCell",
            &[],
            "Landroid/view/View;",
            vec![],
        )
        .unwrap()[0];
        vm.collect();
        if failed != Word::ZERO {
            assert!(
                vm.heap.get(failed).is_err(),
                "failed bind retained temporary cells"
            );
        }
        call(&mut vm, "recover", &[], "V", vec![]).unwrap();
        assert_eq!(grid(&vm.layout_snapshot().unwrap()).children.len(), 7);
        call(&mut vm, "resize", &["I"], "V", vec![Word::from(4)]).unwrap();
        vm.layout_snapshot().unwrap();
    }
    call(&mut vm, "throwClick", &["Z"], "V", vec![Word::from(1)]).unwrap();
    let target = photo(&mut vm, 0);
    assert!(format!("{:#}", vm.click(target).unwrap_err()).contains("item callback failed"));
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "throwClick", &["Z"], "V", vec![Word::ZERO]).unwrap();
    vm.click(target).unwrap();

    call(&mut vm, "invalidate", &[], "V", vec![]).unwrap();
    assert!(grid(&vm.layout_snapshot().unwrap()).children.is_empty());
    call(&mut vm, "recover", &[], "V", vec![]).unwrap();
    assert_eq!(grid(&vm.layout_snapshot().unwrap()).children.len(), 7);
    call(&mut vm, "replace", &[], "V", vec![]).unwrap();
    assert!(
        grid(&vm.snapshot().unwrap()).children.is_empty(),
        "setAdapter kept old cells"
    );
    assert_eq!(grid(&vm.layout_snapshot().unwrap()).children.len(), 3);
    call(&mut vm, "notifyOld", &[], "V", vec![]).unwrap();
    assert_eq!(
        grid(&vm.layout_snapshot().unwrap()).children.len(),
        3,
        "old adapter observer remained attached"
    );
    assert!(
        format!(
            "{:#}",
            call(&mut vm, "resize", &["I"], "V", vec![Word::from(1025)]).unwrap_err()
        )
        .contains("0..1024")
    );
    assert_eq!(vm.stack_depth(), 0);
    call(&mut vm, "resize", &["I"], "V", vec![Word::from(3)]).unwrap();
    vm.layout_snapshot().unwrap();
    call(&mut vm, "detach", &[], "V", vec![]).unwrap();
    assert!(grid(&vm.layout_snapshot().unwrap()).children.is_empty());
    call(&mut vm, "reattach", &[], "V", vec![]).unwrap();
    assert_eq!(grid(&vm.layout_snapshot().unwrap()).children.len(), 3);
    vm.close().unwrap();
}
