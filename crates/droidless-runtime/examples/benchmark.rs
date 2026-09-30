//! Measures this implementation on this machine; no emulator comparisons.
use anyhow::{Context, Result};
use droidless_formats::{apk::Apk, dex::Method};
use droidless_runtime::{Runtime, heap::Word};
use std::time::Instant;
fn median(values: &mut [f64]) -> f64 {
    values.sort_by(f64::total_cmp);
    values[values.len() / 2]
}
fn main() -> Result<()> {
    let path = std::env::args()
        .nth(1)
        .context("usage: benchmark app.apk")?;
    let mut load = vec![];
    let mut startup = vec![];
    let mut ips = vec![];
    for _ in 0..11 {
        let t = Instant::now();
        let apk = Apk::open(&path)?;
        load.push(t.elapsed().as_secs_f64() * 1000.0);
        let t = Instant::now();
        let mut vm = Runtime::new(apk)?;
        vm.launch()?;
        vm.snapshot()?;
        startup.push(t.elapsed().as_secs_f64() * 1000.0);
        vm.close()?;
        let apk = Apk::parse(include_bytes!("../../../fixtures/generated/counter.apk"))?;
        let mut vm = Runtime::new(apk)?;
        let t = Instant::now();
        vm.invoke(
            Method {
                class: "Lorg/droidless/counter/MainActivity;".into(),
                name: "sum".into(),
                parameters: vec!["I".into()],
                returns: "I".into(),
            },
            vec![Word::from(100_000)],
            false,
        )?;
        ips.push(vm.instructions as f64 / t.elapsed().as_secs_f64());
    }
    println!(
        "{}",
        serde_json::to_string_pretty(&serde_json::json!({
            "version":env!("CARGO_PKG_VERSION"),"os":std::env::consts::OS,"arch":std::env::consts::ARCH,"samples":11,
            "apk":path,"apk_decode_median_ms":median(&mut load),"headless_launch_and_layout_median_ms":median(&mut startup),
            "dex_int_loop_instructions_per_second_median":median(&mut ips),
            "scope":"Warm process samples, release build; no native first-frame measurement, no emulator comparison"
        }))?
    );
    Ok(())
}
