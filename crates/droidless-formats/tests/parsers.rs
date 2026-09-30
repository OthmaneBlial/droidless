use droidless_formats::{apk::Apk, dex::Dex, xml};
use sha1::{Digest, Sha1};
use std::io::{Cursor, Write};

const APK: &[u8] = include_bytes!("../../../fixtures/third-party/smallest.apk");

#[test]
fn independently_built_apk() {
    let apk = Apk::parse(APK).unwrap();
    assert_eq!(apk.manifest.package, "c.k");
    assert_eq!(apk.manifest.main_activity.as_deref(), Some("c.k.a"));
    let dex = &apk.dex[0];
    assert!(dex.classes.iter().any(|c| c.name == "Lc/k/a;"));
    assert!(dex.methods.iter().any(|m| m.name == "onCreate"));
    assert!(
        dex.classes
            .iter()
            .flat_map(|c| &c.methods)
            .filter_map(|m| m.code.as_ref())
            .any(|c| !c.instructions.is_empty())
    );
}

#[test]
fn truncations_are_errors() {
    let apk = Apk::parse(APK).unwrap();
    let dex = &apk.files["classes.dex"];
    for n in 0..dex.len() {
        assert!(Dex::parse(&dex[..n]).is_err(), "DEX prefix {n}");
    }
    let xml = &apk.files["AndroidManifest.xml"];
    for n in 0..xml.len() {
        assert!(xml::parse(&xml[..n]).is_err(), "XML prefix {n}");
    }
    for n in 0..APK.len() {
        assert!(Apk::parse(&APK[..n]).is_err(), "APK prefix {n}");
    }
}

#[test]
fn invalid_dex_indexes_are_rejected_after_valid_checksums() {
    let apk = Apk::parse(APK).unwrap();
    let mut dex = apk.files["classes.dex"].clone();
    let off = u32::from_le_bytes(dex[68..72].try_into().unwrap()) as usize;
    dex[off..off + 4].copy_from_slice(&u32::MAX.to_le_bytes());
    let signature = Sha1::digest(&dex[32..]);
    dex[12..32].copy_from_slice(&signature);
    let (mut a, mut b) = (1u32, 0u32);
    for byte in &dex[12..] {
        a = (a + u32::from(*byte)) % 65521;
        b = (b + a) % 65521;
    }
    dex[8..12].copy_from_slice(&((b << 16) | a).to_le_bytes());
    assert!(
        Dex::parse(&dex)
            .unwrap_err()
            .to_string()
            .contains("string index")
    );
}

#[test]
fn zip_traversal_and_duplicates_are_rejected() {
    for names in [vec!["../escape"], vec!["same", "same"]] {
        let mut writer = zip::ZipWriter::new(Cursor::new(vec![]));
        for name in names {
            writer
                .start_file(name, zip::write::FileOptions::default())
                .unwrap();
            writer.write_all(b"x").unwrap();
        }
        let bytes = writer.finish().unwrap().into_inner();
        assert!(Apk::parse(&bytes).is_err());
    }
}
