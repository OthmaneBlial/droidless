use anyhow::{Result, bail};
use droidless_formats::apk::Apk;

fn main() {
    if let Err(error) = run() {
        eprintln!("DROIDLESS: {error:#}");
        std::process::exit(1);
    }
}
fn run() -> Result<()> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        println!(
            "DROIDLESS — Run Android apps without Android.\n\nUsage: droidless <command> <app.apk>\n\nCommands: inspect, manifest, dex, classes, methods, resources\n\nExperimental runtime; unsupported features fail explicitly."
        );
        return Ok(());
    }
    if args[0] == "--version" {
        println!("droidless {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }
    if args.len() != 2 {
        bail!("expected a command and APK path; see --help");
    }
    let apk = Apk::open(&args[1])?;
    match args[0].as_str() {
        "inspect" => {
            println!(
                "Package:       {}\nVersion:       {}\nMin SDK:       {:?}\nTarget SDK:    {:?}\nMain Activity: {}\nDEX files:     {}\nActivities:    {}\nServices:      {}\nPermissions:   {}\nNative libs:   {}\nResources:     {}",
                apk.manifest.package,
                apk.manifest.version_name.as_deref().unwrap_or("unknown"),
                apk.manifest.min_sdk,
                apk.manifest.target_sdk,
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
