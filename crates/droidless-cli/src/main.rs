use anyhow::{Result, bail};
use droidless_formats::apk::Apk;
use droidless_runtime::{Runtime, Trace};
#[cfg(target_os = "macos")]
mod native;

enum Action {
    Click(String),
    Key(String),
    Input(String),
    Back,
    Advance(u64),
}

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
            "DROIDLESS — Run Android apps without Android.\n\nUsage: droidless <command> <app.apk>\n\nCommands: run, inspect, inspect-ui, manifest, dex, classes, methods, resources\n\nRun options: --headless --click TEXT --key CHAR --input TEXT --back --stats\n             --advance-ms MILLISECONDS (deterministic timer replay)\n             --data-dir APPS_ROOT | --ephemeral\n             --size WIDTHxHEIGHT (128..4096; default 420x720)\n             --trace-bytecode --trace-methods --trace-framework --trace-lifecycle\n\n--input edits the first enabled visible EditText. Native Back: Escape.\nStorage defaults to a per-package host application-data directory.\nExperimental runtime; unsupported features fail explicitly."
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
        let mut data_dir = None;
        let mut ephemeral = false;
        let mut size = None;
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
                    actions.push(Action::Click(
                        args.get(i)
                            .ok_or_else(|| anyhow::anyhow!("--click requires text"))?
                            .clone(),
                    ));
                }
                "--key" => {
                    i += 1;
                    actions.push(Action::Key(
                        args.get(i)
                            .ok_or_else(|| anyhow::anyhow!("--key requires one character"))?
                            .clone(),
                    ));
                }
                "--back" => actions.push(Action::Back),
                "--advance-ms" => {
                    i += 1;
                    let milliseconds = args
                        .get(i)
                        .ok_or_else(|| anyhow::anyhow!("--advance-ms requires milliseconds"))?
                        .parse::<u64>()?;
                    anyhow::ensure!(
                        milliseconds <= i64::MAX as u64,
                        "time advance exceeds the monotonic clock limit"
                    );
                    actions.push(Action::Advance(milliseconds));
                }
                "--input" => {
                    i += 1;
                    actions.push(Action::Input(
                        args.get(i)
                            .ok_or_else(|| anyhow::anyhow!("--input requires text"))?
                            .clone(),
                    ));
                }
                "--data-dir" => {
                    i += 1;
                    let value = args
                        .get(i)
                        .ok_or_else(|| anyhow::anyhow!("--data-dir requires an apps root"))?;
                    if value.is_empty()
                        || data_dir.replace(std::path::PathBuf::from(value)).is_some()
                    {
                        bail!("expected one nonempty --data-dir");
                    }
                }
                "--ephemeral" => ephemeral = true,
                "--size" => {
                    i += 1;
                    let value = args
                        .get(i)
                        .ok_or_else(|| anyhow::anyhow!("--size requires WIDTHxHEIGHT"))?;
                    if size.replace(parse_size(value)?).is_some() {
                        bail!("expected one --size");
                    }
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
        anyhow::ensure!(
            !(ephemeral && data_dir.is_some()),
            "--data-dir and --ephemeral are mutually exclusive"
        );
        let mut runtime = if ephemeral {
            Runtime::new(apk)?
        } else {
            Runtime::with_data_dir(apk, data_dir.map(Ok).unwrap_or_else(default_data_dir)?)?
        };
        runtime.trace = trace;
        if let Some((width, height)) = size {
            runtime.width = width;
            runtime.height = height;
        }
        runtime.launch()?;
        runtime.poll_messages()?;
        for action in actions {
            match action {
                Action::Key(text) => {
                    runtime.layout_snapshot()?;
                    let target = runtime
                        .focused_key_target()?
                        .ok_or_else(|| anyhow::anyhow!("no View key listener"))?;
                    runtime.key_text(target, 0, &text)?;
                    if runtime.activity.is_some() && runtime.focused_key_target()? == Some(target) {
                        runtime.key_text(target, 1, &text)?;
                    }
                }
                Action::Click(text) => {
                    runtime.click_text(&text)?;
                }
                Action::Back => runtime.back()?,
                Action::Input(text) => runtime.input(&text)?,
                Action::Advance(milliseconds) => {
                    runtime.advance_time(milliseconds)?;
                }
            }
            runtime.poll_messages()?;
        }
        if headless {
            anyhow::ensure!(
                !runtime.directory_picker_pending(),
                "APK requested a native directory picker; run without --headless"
            );
            let tree = runtime
                .activity
                .map(|_| runtime.layout_snapshot())
                .transpose()?;
            println!("{}", serde_json::to_string_pretty(&tree)?);
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

fn parse_size(value: &str) -> Result<(f32, f32)> {
    let (width, height) = value
        .split_once('x')
        .ok_or_else(|| anyhow::anyhow!("expected WIDTHxHEIGHT"))?;
    let (width, height) = (width.parse::<u16>()?, height.parse::<u16>()?);
    anyhow::ensure!(
        (128..=4096).contains(&width) && (128..=4096).contains(&height),
        "viewport dimensions must be 128..4096"
    );
    Ok((f32::from(width), f32::from(height)))
}

#[test]
fn viewport_size_is_bounded() {
    assert_eq!(parse_size("192x400").unwrap(), (192.0, 400.0));
    for value in [
        "0x400",
        "192x0",
        "NaNx400",
        "192x400x5",
        "-1x400",
        "4097x400",
        "65536x400",
    ] {
        assert!(parse_size(value).is_err(), "{value}");
    }
}

fn default_data_dir() -> Result<std::path::PathBuf> {
    #[cfg(target_os = "macos")]
    {
        let home = std::env::var_os("HOME")
            .filter(|v| !v.is_empty())
            .ok_or_else(|| anyhow::anyhow!("HOME is unavailable; use --data-dir or --ephemeral"))?;
        Ok(std::path::PathBuf::from(home).join("Library/Application Support/DROIDLESS/apps"))
    }
    #[cfg(not(target_os = "macos"))]
    {
        let base = std::env::var_os("XDG_DATA_HOME")
            .filter(|v| !v.is_empty())
            .map(std::path::PathBuf::from)
            .or_else(|| {
                std::env::var_os("HOME")
                    .filter(|v| !v.is_empty())
                    .map(|v| std::path::PathBuf::from(v).join(".local/share"))
            })
            .ok_or_else(|| {
                anyhow::anyhow!("no application-data directory; use --data-dir or --ephemeral")
            })?;
        Ok(base.join("droidless/apps"))
    }
}
