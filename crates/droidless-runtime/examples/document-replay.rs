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
        args.len() == 2 || (args.len() == 3 && args[2] == "--click-first-image"),
        "usage: document-replay APK DIRECTORY [--click-first-image]"
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
    if args.len() == 3 {
        vm.click(first_image(&tree).context("no decoded ImageView to click")?)?;
        tree = vm.layout_snapshot().context("image-click View tree")?;
    }
    eprintln!("Decoded image views: {}", decoded_images(&tree));
    println!("{}", serde_json::to_string_pretty(&tree)?);
    vm.close()?;
    Ok(())
}
