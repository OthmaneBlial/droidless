use anyhow::{Result, bail};
use droidless_formats::apk::Apk;
use droidless_runtime::{Runtime, Trace};
#[cfg(target_os = "macos")]
mod native;

fn main() {
    if let Err(error) = run() {
        eprintln!("DROIDLESS: {error:#}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let mut args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        println!(
            "DROIDLESS — Run Android apps without Android.\n\nUsage: droidless <command> <app.apk>\n\nCommands: run, inspect, inspect-ui, manifest, dex, classes, methods, resources\n\nRun options: --headless --click TEXT --key CHAR --stats\n             --trace-bytecode --trace-methods --trace-framework --trace-lifecycle\n\nExperimental runtime; unsupported features fail explicitly."
        );
        return Ok(());
    }
    if args[0] == "--version" {
        println!("droidless {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args[0].ends_with(".apk") {
        args.insert(0, "run".into());
    }
    if args[0] == "run" || args[0] == "inspect-ui" {
        let mut trace = Trace::default();
        let mut actions = vec![];
        let mut path = None;
        let mut stats = false;
        let mut headless = args[0] == "inspect-ui";
        let mut i = 1;
        while i < args.len() {
            match args[i].as_str() {
                "--trace-bytecode" => trace.bytecode = true,
                "--trace-methods" => trace.methods = true,
                "--trace-framework" => trace.framework = true,
                "--trace-lifecycle" => trace.lifecycle = true,
                "--stats" | "--heap-stats" => stats = true,
                "--headless" => {
                    headless = true;
                }
                "--click" => {
                    i += 1;
                    actions.push((
                        false,
                        args.get(i)
                            .ok_or_else(|| anyhow::anyhow!("--click requires text"))?
                            .clone(),
                    ));
                }
                "--key" => {
                    i += 1;
                    actions.push((
                        true,
                        args.get(i)
                            .ok_or_else(|| anyhow::anyhow!("--key requires one character"))?
                            .clone(),
                    ));
                }
                s if s.starts_with('-') => bail!("unknown run option {s}"),
                _ => {
                    if path.replace(args[i].clone()).is_some() {
                        bail!("expected one APK path");
                    }
                }
            }
            i += 1;
        }
        let start = std::time::Instant::now();
        let apk = Apk::open(path.ok_or_else(|| anyhow::anyhow!("missing APK path"))?)?;
        let loaded = start.elapsed();
        let mut runtime = Runtime::new(apk)?;
        runtime.trace = trace;
        runtime.launch()?;
        for (key, text) in actions {
            if key {
                let target = runtime
                    .focused_key_target()?
                    .ok_or_else(|| anyhow::anyhow!("no View key listener"))?;
                runtime.key_text(target, 0, &text)?;
                runtime.key_text(target, 1, &text)?;
            } else {
                runtime.click_text(&text)?;
            }
        }
        if headless {
            println!("{}", serde_json::to_string_pretty(&runtime.snapshot()?)?);
        } else {
            #[cfg(target_os = "macos")]
            native::run(&mut runtime)?;
            #[cfg(not(target_os = "macos"))]
            bail!("native UI currently requires macOS; use --headless on this platform");
        }
        runtime.close()?;
        if stats {
            eprintln!(
                "APK parse: {:.3} ms; total: {:.3} ms; instructions: {}; calls: {}; live objects: {}; collections: {}",
                loaded.as_secs_f64() * 1000.0,
                start.elapsed().as_secs_f64() * 1000.0,
                runtime.instructions,
                runtime.method_calls,
                runtime.heap.live(),
                runtime.heap.collections
            );
        }
        return Ok(());
    }
    if args.len() != 2 {
        bail!("expected a command and APK path; see --help");
    }
    let apk = Apk::open(&args[1])?;
    match args[0].as_str() {
        "inspect" => {
            println!(
                "Package:       {}\nVersion:       {}\nMin SDK:       {}\nTarget SDK:    {}\nMain Activity: {}\nDEX files:     {}\nActivities:    {}\nServices:      {}\nPermissions:   {}\nNative libs:   {}\nResources:     {}",
                apk.manifest.package,
                apk.manifest.version_name.as_deref().unwrap_or("unknown"),
                apk.manifest
                    .min_sdk
                    .map_or_else(|| "unspecified".into(), |v| v.to_string()),
                apk.manifest
                    .target_sdk
                    .map_or_else(|| "unspecified".into(), |v| v.to_string()),
                apk.manifest.main_activity.as_deref().unwrap_or("none"),
                apk.dex.len(),
                apk.manifest.activities.len(),
                apk.manifest.services.len(),
                apk.manifest.permissions.len(),
                apk.native_libraries().len(),
                apk.resources.entries.len()
            );
        }
        "manifest" => println!("{}", serde_json::to_string_pretty(&apk.manifest)?),
        "dex" => {
            for (i, dex) in apk.dex.iter().enumerate() {
                println!(
                    "DEX {}: version {}, {} classes, {} methods, {} fields, {} strings, {} code units",
                    i + 1,
                    dex.version,
                    dex.classes.len(),
                    dex.methods.len(),
                    dex.fields.len(),
                    dex.strings.len(),
                    dex.classes
                        .iter()
                        .flat_map(|c| &c.methods)
                        .filter_map(|m| m.code.as_ref())
                        .map(|c| c.instructions.len())
                        .sum::<usize>()
                );
            }
        }
        "classes" => {
            for d in &apk.dex {
                for c in &d.classes {
                    println!("{}", c.name);
                }
            }
        }
        "methods" => {
            for d in &apk.dex {
                for m in &d.methods {
                    println!("{}", m.key());
                }
            }
        }
        "resources" => println!("{}", serde_json::to_string_pretty(&apk.resources)?),
        command => bail!("unknown command {command}; see --help"),
    }
    Ok(())
}
