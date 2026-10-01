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

fn main() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    ensure!(
        (2..=4).contains(&args.len())
            && args.get(2).is_none_or(|v| v == "--click-first-image")
            && args.get(3).is_none_or(|v| v == "--back"),
        "usage: document-replay APK DIRECTORY [--click-first-image [--back]]"
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
        if args.len() == 4 {
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
