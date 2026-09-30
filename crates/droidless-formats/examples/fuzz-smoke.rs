//! Deterministic mutation smoke, not a substitute for coverage-guided fuzzing.
use droidless_formats::{apk::Apk, binary::Bytes, dex::Dex, resources::Resources, xml};
use sha1::{Digest, Sha1};

fn reseal_dex(data: &mut [u8]) {
    let signature = Sha1::digest(&data[32..]);
    data[12..32].copy_from_slice(&signature);
    let (mut a, mut b) = (1u32, 0u32);
    for byte in &data[12..] {
        a = (a + u32::from(*byte)) % 65521;
        b = (b + a) % 65521;
    }
    data[8..12].copy_from_slice(&((b << 16) | a).to_le_bytes());
}
fn main() {
    let apk = include_bytes!("../../../fixtures/generated/counter.apk");
    let loaded = Apk::parse(apk).unwrap();
    let mut state = 0xdeadbeefu64;
    for round in 0..1024 {
        for name in ["AndroidManifest.xml", "classes.dex", "resources.arsc"] {
            let mut data = loaded.files[name].clone();
            for _ in 0..3 {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                let at = state as usize % data.len();
                data[at] ^= (state >> 32) as u8;
            }
            if name == "classes.dex" {
                reseal_dex(&mut data);
                let _ = Dex::parse(&data);
            } else if name == "resources.arsc" {
                let _ = Resources::parse(&data);
            } else {
                let _ = xml::parse(&data);
            }
        }
        let mut data = apk.to_vec();
        data[round % apk.len()] ^= 0xff;
        let _ = Apk::parse(&data);
        let arbitrary = state.to_le_bytes();
        let _ = Bytes(&arbitrary).chunk(0);
    }
    println!("4096 seeded APK/DEX/XML/resource mutations completed without panics");
}
