//! Explicit host folder choice for reproducible APK document workflows, without GUI automation.
use anyhow::{Context, Result, ensure};
use droidless_formats::apk::Apk;
use droidless_runtime::Runtime;
use std::path::Path;

fn decoded_images(node: &droidless_runtime::ui::Node) -> usize {
    usize::from(node.view.image.is_some()) + node.children.iter().map(decoded_images).sum::<usize>()
}
fn first_image(node: &droidless_runtime::ui::Node) -> Option<usize> {
    if node.view.kind == "ImageView" && node.view.image.is_some() {
        return Some(node.handle);
    }
    node.children.iter().find_map(first_image)
}
fn image_bytes(node: &droidless_runtime::ui::Node) -> Option<&[u8]> {
    node.view
        .image
        .as_deref()
        .or_else(|| node.children.iter().find_map(image_bytes))
}
fn buttons(node: &droidless_runtime::ui::Node) -> usize {
    usize::from(node.view.kind == "Button") + node.children.iter().map(buttons).sum::<usize>()
}
fn swipe(vm: &mut Runtime, left: bool) -> Result<()> {
    let (start, end) = if left { (340.0, 80.0) } else { (80.0, 340.0) };
    vm.touch(0, start, 360.0)?;
    for step in 1..=4 {
        vm.advance_time(20)?;
        vm.touch(
            if step == 4 { 1 } else { 2 },
            start + (end - start) * step as f32 / 4.0,
            360.0,
        )?;
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        (2..=4).contains(&args.len())
            && args.get(2).is_none_or(|v| v == "--click-first-image")
            && args
                .get(3)
                .is_none_or(|v| v == "--back" || v == "--gestures"),
        "usage: document-replay APK DIRECTORY [--click-first-image [--back|--gestures]]"
    );
    let mut vm = Runtime::new(Apk::parse(&std::fs::read(&args[0])?)?)?;
    vm.launch()?;
    if !vm.directory_picker_pending() {
        ensure!(
            vm.click_text("Pick image folder")?,
            "APK has no directory choice pending"
        );
    }
    ensure!(
        vm.directory_picker_pending(),
        "APK did not request a directory picker"
    );
    vm.complete_directory_picker(Some(Path::new(&args[1])))?;
    let mut tree = vm.layout_snapshot().context("selected-folder View tree")?;
    if args.len() >= 3 {
        vm.click(first_image(&tree).context("no decoded ImageView to click")?)?;
        tree = vm.layout_snapshot().context("image-click View tree")?;
        ensure!(
            vm.activity_depth() == 2,
            "image click did not open another Activity"
        );
        ensure!(
            decoded_images(&tree) == 1,
            "viewer did not decode one image"
        );
        if args.get(3).is_some_and(|s| s == "--gestures") {
            let original_buttons = buttons(&tree);
            ensure!(original_buttons > 0, "viewer controls missing before tap");
            let expected = ["sample.jpg", "sample.png", "sample.webp"]
                .map(|name| std::fs::read(Path::new(&args[1]).join(name)))
                .into_iter()
                .collect::<Result<Vec<_>, _>>()?;
            ensure!(
                image_bytes(&tree) == Some(expected[0].as_slice()),
                "initial viewer image differs"
            );
            for (left, index) in [
                (false, 0),
                (true, 1),
                (true, 2),
                (true, 2),
                (false, 1),
                (false, 0),
            ] {
                swipe(&mut vm, left)?;
                vm.collect();
                tree = vm.layout_snapshot()?;
                ensure!(
                    image_bytes(&tree) == Some(expected[index].as_slice()),
                    "swipe did not select image {index}"
                );
            }
            vm.touch(0, 200.0, 360.0)?;
            vm.advance_time(20)?;
            vm.touch(1, 200.0, 360.0)?;
            vm.advance_time(300)?;
            tree = vm.layout_snapshot()?;
            ensure!(buttons(&tree) == 0, "confirmed tap did not hide controls");
            vm.advance_time(300)?;
            vm.touch(0, 200.0, 360.0)?;
            vm.advance_time(20)?;
            vm.touch(1, 200.0, 360.0)?;
            vm.advance_time(300)?;
            vm.advance_time(300)?;
            tree = vm.layout_snapshot()?;
            ensure!(
                buttons(&tree) == original_buttons,
                "second confirmed tap did not restore controls"
            );
            ensure!(
                image_bytes(&tree) == Some(expected[0].as_slice()),
                "taps changed the image"
            );
            eprintln!(
                "PASS public swipe navigation: first/last bounds, next/previous JPEG/PNG/WebP; confirmed taps hide and restore controls"
            );
        } else if args.len() == 4 {
            vm.back()?;
            vm.collect();
            tree = vm.layout_snapshot().context("viewer Back View tree")?;
            ensure!(
                vm.activity_depth() == 1,
                "Back did not return to the image list"
            );
        }
    }
    eprintln!("Decoded image views: {}", decoded_images(&tree));
    println!("{}", serde_json::to_string_pretty(&tree)?);
    vm.close()?;
    Ok(())
}
