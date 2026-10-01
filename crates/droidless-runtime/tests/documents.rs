use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{
    Runtime,
    heap::{Data, Word},
};
use std::{
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};

fn invoke(
    vm: &mut Runtime,
    class: &str,
    name: &str,
    parameters: &[&str],
    returns: &str,
    args: Vec<Word>,
) -> anyhow::Result<Vec<Word>> {
    vm.invoke(
        Method {
            class: class.into(),
            name: name.into(),
            parameters: parameters.iter().map(|value| (*value).into()).collect(),
            returns: returns.into(),
        },
        args,
        false,
    )
}
fn uri(vm: &mut Runtime, value: &str) -> Word {
    let text = vm.heap.string(value.to_owned()).unwrap();
    invoke(
        vm,
        "Landroid/net/Uri;",
        "parse",
        &["Ljava/lang/String;"],
        "Landroid/net/Uri;",
        vec![text],
    )
    .unwrap()[0]
}
fn open(vm: &mut Runtime, resolver: Word, value: &str) -> anyhow::Result<Vec<Word>> {
    let value = uri(vm, value);
    invoke(
        vm,
        "Landroid/content/ContentResolver;",
        "openInputStream",
        &["Landroid/net/Uri;"],
        "Ljava/io/InputStream;",
        vec![resolver, value],
    )
}
struct Scratch(PathBuf);
impl Drop for Scratch {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.0).unwrap();
    }
}

#[test]
fn compiled_document_images_and_confined_snapshot_streams() {
    let scratch = Scratch(std::env::temp_dir().join(format!(
            "droidless-documents-{}-{}",
            std::process::id(),
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        )));
    let folder = scratch.0.join("chosen");
    std::fs::create_dir_all(folder.join("nested")).unwrap();
    let assets = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples/images/assets");
    for (source, name) in [
        ("sample.png", "a space % é.png"),
        ("sample.jpg", "b.jpg"),
        ("sample.webp", "c.webp"),
    ] {
        std::fs::copy(assets.join(source), folder.join(name)).unwrap();
    }
    std::fs::write(folder.join("nested/plain.txt"), b"bounded stream").unwrap();
    std::fs::write(scratch.0.join("outside.txt"), b"outside grant").unwrap();
    std::fs::hard_link(scratch.0.join("outside.txt"), folder.join("hard.txt")).unwrap();
    #[cfg(unix)]
    {
        std::os::unix::fs::symlink(scratch.0.join("outside.txt"), folder.join("linked.txt"))
            .unwrap();
        std::os::unix::fs::symlink(&scratch.0, folder.join("escape")).unwrap();
    }
    let apk = include_bytes!("../../../fixtures/generated/documents.apk");
    let mut vm = Runtime::new(Apk::parse(apk).unwrap()).unwrap();
    vm.launch().unwrap();
    let activity = vm.activity.unwrap();
    let resolver = invoke(
        &mut vm,
        "Landroid/content/Context;",
        "getContentResolver",
        &[],
        "Landroid/content/ContentResolver;",
        vec![activity],
    )
    .unwrap()[0];
    assert!(
        open(
            &mut vm,
            resolver,
            "content://droidless.documents/tree/0/document/0%2Fb.jpg"
        )
        .is_err()
    );
    vm.click_text("Pick image folder").unwrap();
    vm.complete_directory_picker(Some(&folder)).unwrap();
    let tree = vm.snapshot().unwrap();
    assert_eq!(tree.children[0].view.text, "3 images · document streams");
    assert_eq!(
        tree.children
            .iter()
            .filter(|node| node.view.image.is_some())
            .count(),
        3
    );
    assert!(
        tree.children
            .iter()
            .any(|node| node.view.content_description.as_deref() == Some("a space % é.png"))
    );
    for value in [
        "file:///etc/passwd",
        "content://other/tree/0/document/0%2Fb.jpg",
        "content://droidless.documents/tree/1/document/1%2Fb.jpg",
        "content://droidless.documents/tree/0/document/1%2Fb.jpg",
        "content://droidless.documents/tree/0/document/0%2F..%2Foutside.txt",
        "content://droidless.documents/tree/0/document/0%2F%2Foutside.txt",
        "content://droidless.documents/tree/0/document/0%2F.%2Fb.jpg",
        "content://droidless.documents/tree/0/document/0%2Fbad%00.txt",
        "content://droidless.documents/tree/0/document/0%2Fbad%GG.txt",
        "content://droidless.documents/tree/0/document/0%2Fbad%2",
        "content://droidless.documents/tree/0/document/0%2Fhard.txt",
        "content://droidless.documents/tree/0/document/0%2Flinked.txt",
        "content://droidless.documents/tree/0/document/0%2Fescape%2Foutside.txt",
    ] {
        assert!(open(&mut vm, resolver, value).is_err(), "accepted {value}");
    }
    let stream = open(
        &mut vm,
        resolver,
        "content://droidless.documents/tree/0/document/0%2Fnested%2Fplain.txt/children",
    )
    .unwrap()[0];
    // API-21 DocumentsProvider uses the document ID when opening, even with a children suffix.
    vm.heap
        .get_mut(activity)
        .unwrap()
        .fields
        .insert("test:stream".into(), vec![stream]);
    std::fs::write(folder.join("nested/plain.txt"), b"changed").unwrap();
    vm.collect();
    let Data::ByteStream { bytes, .. } = &vm.heap.get(stream).unwrap().data else {
        panic!("missing stream");
    };
    assert_eq!(bytes, b"bounded stream");
    invoke(
        &mut vm,
        "Ljava/io/InputStream;",
        "close",
        &[],
        "V",
        vec![stream],
    )
    .unwrap();
    assert!(
        invoke(
            &mut vm,
            "Ljava/io/InputStream;",
            "read",
            &[],
            "I",
            vec![stream]
        )
        .is_err()
    );
    let document = uri(
        &mut vm,
        "content://droidless.documents/tree/0/document/0%2Fnested/children",
    );
    let cursor = invoke(
        &mut vm,
        "Landroid/content/ContentResolver;",
        "query",
        &[
            "Landroid/net/Uri;",
            "[Ljava/lang/String;",
            "Ljava/lang/String;",
            "[Ljava/lang/String;",
            "Ljava/lang/String;",
        ],
        "Landroid/database/Cursor;",
        vec![
            resolver,
            document,
            Word::ZERO,
            Word::ZERO,
            Word::ZERO,
            Word::ZERO,
        ],
    )
    .unwrap()[0];
    vm.heap
        .get_mut(activity)
        .unwrap()
        .fields
        .insert("test:cursor".into(), vec![cursor]);
    vm.collect();
    assert_eq!(
        invoke(
            &mut vm,
            "Landroid/database/Cursor;",
            "getCount",
            &[],
            "I",
            vec![cursor]
        )
        .unwrap(),
        [Word::from(1)]
    );
    assert_eq!(
        invoke(
            &mut vm,
            "Landroid/database/Cursor;",
            "moveToNext",
            &[],
            "Z",
            vec![cursor]
        )
        .unwrap(),
        [Word::from(1)]
    );
    let value = invoke(
        &mut vm,
        "Landroid/database/Cursor;",
        "getString",
        &["I"],
        "Ljava/lang/String;",
        vec![cursor, Word::from(2)],
    )
    .unwrap()[0];
    assert_eq!(vm.heap.text(value).unwrap(), "plain.txt");
    invoke(
        &mut vm,
        "Landroid/database/Cursor;",
        "close",
        &[],
        "V",
        vec![cursor],
    )
    .unwrap();
    assert!(
        invoke(
            &mut vm,
            "Landroid/database/Cursor;",
            "moveToNext",
            &[],
            "Z",
            vec![cursor]
        )
        .is_err()
    );
    let text = vm.heap.string("a😀/é/a😀".into()).unwrap();
    for (needle, from, expected) in [
        (0x1f600, 99, 7),
        (0xd83d, 99, 7),
        (0x1f600, 6, 1),
        (0x2f, -1, -1),
        (0x2f, 99, 5),
    ] {
        assert_eq!(
            invoke(
                &mut vm,
                "Ljava/lang/String;",
                "lastIndexOf",
                &["I", "I"],
                "I",
                vec![text, Word::from(needle), Word::from(from)]
            )
            .unwrap(),
            [Word::from(expected)]
        );
    }
    let large = std::fs::File::create(folder.join("large.bin")).unwrap();
    large.set_len(64 * 1_048_576 + 1).unwrap();
    assert!(
        format!(
            "{:#}",
            open(
                &mut vm,
                resolver,
                "content://droidless.documents/tree/0/document/0%2Flarge.bin"
            )
            .unwrap_err()
        )
        .contains("64 MiB")
    );
    vm.close().unwrap();
    let mut fresh = Runtime::new(Apk::parse(apk).unwrap()).unwrap();
    fresh.launch().unwrap();
    let activity = fresh.activity.unwrap();
    let resolver = invoke(
        &mut fresh,
        "Landroid/content/Context;",
        "getContentResolver",
        &[],
        "Landroid/content/ContentResolver;",
        vec![activity],
    )
    .unwrap()[0];
    assert!(
        open(
            &mut fresh,
            resolver,
            "content://droidless.documents/tree/0/document/0%2Fb.jpg"
        )
        .is_err()
    );
    fresh.close().unwrap();
}
