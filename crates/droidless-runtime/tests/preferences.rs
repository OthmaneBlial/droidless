use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};
use std::{
    fs,
    path::PathBuf,
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

struct Temporary(PathBuf);
impl Temporary {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let name = format!(
            "droidless-preferences-{}-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        );
        let path = std::env::temp_dir().join(name);
        fs::create_dir(&path).unwrap();
        Self(path)
    }
}
impl Drop for Temporary {
    fn drop(&mut self) {
        fs::remove_dir_all(&self.0).unwrap();
    }
}
fn apk() -> Apk {
    Apk::parse(include_bytes!(
        "../../../fixtures/generated/preferences.apk"
    ))
    .unwrap()
}
fn vm(dir: &Temporary) -> Runtime {
    let mut vm = Runtime::with_data_dir(apk(), &dir.0).unwrap();
    vm.launch().unwrap();
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Conformance passed"
    );
    vm
}
fn note(vm: &Runtime) -> String {
    vm.snapshot().unwrap().children[1].view.text.clone()
}

#[test]
fn staged_typed_values_persist_across_restarts_and_stay_per_package() {
    let dir = Temporary::new();
    let mut first = vm(&dir);
    assert_eq!(note(&first), "No saved note");
    first.click_text("Edit note").unwrap();
    let root = first.root.unwrap();
    first
        .heap
        .get_mut(root)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .visible = 4;
    assert!(first.input("hidden editor must not receive input").is_err());
    first
        .heap
        .get_mut(root)
        .unwrap()
        .view
        .as_mut()
        .unwrap()
        .visible = 0;
    first.input("Notes survive restart 📓").unwrap();
    first.click_text("Save note").unwrap();
    assert_eq!(first.activity_depth(), 1);
    assert_eq!(note(&first), "Notes survive restart 📓");
    first.collect();
    first.close().unwrap();
    drop(first);
    let mut second = vm(&dir);
    assert_eq!(note(&second), "Notes survive restart 📓");
    let mut other_apk = apk();
    other_apk.manifest.package = "org.droidless.other".into();
    let mut other = Runtime::with_data_dir(other_apk, &dir.0).unwrap();
    other.launch().unwrap();
    assert_eq!(note(&other), "No saved note");
    second.click_text("Clear note").unwrap();
    second.close().unwrap();
    assert_eq!(note(&vm(&dir)), "No saved note");
    let mut ephemeral = Runtime::new(apk()).unwrap();
    ephemeral.launch().unwrap();
    assert_eq!(note(&ephemeral), "No saved note");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        assert_eq!(
            fs::metadata(
                dir.0
                    .join("org.droidless.preferences/shared_prefs/6e6f7465.json")
            )
            .unwrap()
            .permissions()
            .mode()
                & 0o777,
            0o600
        );
    }
}

#[test]
fn failed_commit_keeps_input_and_damaged_files_are_not_reset() {
    let dir = Temporary::new();
    let mut vm = vm(&dir);
    let file = dir
        .0
        .join("org.droidless.preferences/shared_prefs/6e6f7465.json");
    fs::create_dir(&file).unwrap();
    vm.click_text("Edit note").unwrap();
    vm.input("Keep unsaved text").unwrap();
    vm.click_text("Save note").unwrap();
    assert_eq!(vm.title, "Save failed");
    assert_eq!(vm.activity_depth(), 2);
    assert_eq!(
        vm.snapshot().unwrap().children[0].view.text,
        "Keep unsaved text"
    );
    fs::remove_dir(&file).unwrap();
    vm.click_text("Save note").unwrap();
    assert_eq!(note(&vm), "Keep unsaved text");
    vm.click_text("Edit note").unwrap();
    vm.input(&"x".repeat(1_048_576)).unwrap();
    vm.click_text("Save note").unwrap();
    assert_eq!(vm.title, "Save failed");
    vm.input("Keep unsaved text").unwrap();
    vm.click_text("Save note").unwrap();
    vm.close().unwrap();
    fs::write(&file, b"damaged preferences").unwrap();
    let mut broken = Runtime::with_data_dir(apk(), &dir.0).unwrap();
    assert!(format!("{:#}", broken.launch().unwrap_err()).contains("invalid preference file"));
    assert_eq!(fs::read(&file).unwrap(), b"damaged preferences");
    fs::write(&file, vec![b' '; 1_048_577]).unwrap();
    let mut too_big = Runtime::with_data_dir(apk(), &dir.0).unwrap();
    assert!(format!("{:#}", too_big.launch().unwrap_err()).contains("exceeds 1 MiB"));
}

#[test]
fn path_components_and_nonprivate_modes_fail() {
    let dir = Temporary::new();
    for package in ["../outside", "", "/tmp/outside", "a..b", "a/b", "a\\b"] {
        let mut bad = apk();
        bad.manifest.package = package.into();
        assert!(Runtime::with_data_dir(bad, &dir.0).is_err());
    }
    assert_eq!(fs::read_dir(&dir.0).unwrap().count(), 0);
    let mut vm = vm(&dir);
    let method = Method {
        class: "Landroid/content/Context;".into(),
        name: "getSharedPreferences".into(),
        parameters: vec!["Ljava/lang/String;".into(), "I".into()],
        returns: "Landroid/content/SharedPreferences;".into(),
    };
    for name in [
        "../outside",
        "/tmp/outside",
        "a/b",
        "a\\b",
        "..",
        ".",
        "",
        "a\0b",
    ] {
        let name = vm.heap.string(name.into()).unwrap();
        assert!(
            vm.invoke(
                method.clone(),
                vec![vm.activity.unwrap(), name, Word::ZERO],
                false
            )
            .is_err()
        );
    }
    let name = vm.heap.string("public".into()).unwrap();
    assert!(
        vm.invoke(
            method,
            vec![vm.activity.unwrap(), name, Word::from(1)],
            false
        )
        .is_err()
    );
}

#[test]
fn package_identity_rejects_case_aliases_and_tampered_markers() {
    let dir = Temporary::new();
    let mut first = vm(&dir);
    first.click_text("Edit note").unwrap();
    first.input("belongs to the original package").unwrap();
    first.click_text("Save note").unwrap();
    first.close().unwrap();
    let package = dir.0.join("org.droidless.preferences");
    let alias = dir.0.join("ORG.droidless.preferences");
    let mut other = apk();
    other.manifest.package = "ORG.droidless.preferences".into();
    if alias.exists() {
        assert!(
            format!("{:#}", Runtime::with_data_dir(other, &dir.0).err().unwrap())
                .contains("identity mismatch")
        );
    } else {
        let mut other = Runtime::with_data_dir(other, &dir.0).unwrap();
        other.launch().unwrap();
        assert_eq!(note(&other), "No saved note");
    }
    assert!(package.join("shared_prefs/63617365.json").is_file());
    assert!(package.join("shared_prefs/43415345.json").is_file());
    fs::write(package.join(".droidless-package"), "another.package").unwrap();
    assert!(
        format!("{:#}", Runtime::with_data_dir(apk(), &dir.0).err().unwrap())
            .contains("identity mismatch")
    );
}

#[cfg(unix)]
#[test]
fn symlinks_hard_links_and_replaced_directories_cannot_escape() {
    use std::os::unix::fs::symlink;
    let dir = Temporary::new();
    let outside = dir.0.join("outside");
    fs::create_dir(&outside).unwrap();
    symlink(&outside, dir.0.join("org.droidless.preferences")).unwrap();
    assert!(Runtime::with_data_dir(apk(), &dir.0).is_err());
    fs::remove_file(dir.0.join("org.droidless.preferences")).unwrap();
    let mut first = vm(&dir);
    let prefs = dir.0.join("org.droidless.preferences/shared_prefs");
    let outside_file = outside.join("private.json");
    let secret = br#"{"schema":1,"values":{"text":{"String":"outside data"}}}"#;
    fs::write(&outside_file, secret).unwrap();
    symlink(&outside_file, prefs.join("6e6f7465.json")).unwrap();
    let mut reader = Runtime::with_data_dir(apk(), &dir.0).unwrap();
    assert!(reader.launch().is_err());
    first.click_text("Edit note").unwrap();
    first.input("must not overwrite outside").unwrap();
    first.click_text("Save note").unwrap();
    assert_eq!(first.title, "Save failed");
    assert_eq!(fs::read(&outside_file).unwrap(), secret);
    fs::remove_file(prefs.join("6e6f7465.json")).unwrap();
    fs::hard_link(&outside_file, prefs.join("6e6f7465.json")).unwrap();
    let mut reader = Runtime::with_data_dir(apk(), &dir.0).unwrap();
    assert!(format!("{:#}", reader.launch().unwrap_err()).contains("hard-linked"));
    first.click_text("Save note").unwrap();
    assert_eq!(first.title, "Save failed");
    assert_eq!(fs::read(&outside_file).unwrap(), secret);
    fs::remove_file(prefs.join("6e6f7465.json")).unwrap();
    // An opened capability stays bound when the host replaces its directory name.
    let moved = dir.0.join("original-prefs");
    fs::rename(&prefs, &moved).unwrap();
    symlink(&outside, &prefs).unwrap();
    assert!(Runtime::with_data_dir(apk(), &dir.0).is_err());
    first.click_text("Save note").unwrap();
    assert_eq!(first.activity_depth(), 1);
    assert!(moved.join("6e6f7465.json").is_file());
    assert!(!outside.join("6e6f7465.json").exists());
    assert_eq!(fs::read(&outside_file).unwrap(), secret);
    let marker = dir.0.join("org.droidless.preferences/.droidless-package");
    fs::remove_file(&marker).unwrap();
    symlink(&outside_file, &marker).unwrap();
    assert!(Runtime::with_data_dir(apk(), &dir.0).is_err());
    fs::remove_file(&marker).unwrap();
    fs::hard_link(&outside_file, &marker).unwrap();
    assert!(
        format!("{:#}", Runtime::with_data_dir(apk(), &dir.0).err().unwrap())
            .contains("hard-linked")
    );
    assert_eq!(fs::read(&outside_file).unwrap(), secret);
}
