use crate::{dex::Dex, resources::Resources, xml::Manifest};
use anyhow::{Context, Result, ensure};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
    path::Path,
};

pub struct Apk {
    pub manifest: Manifest,
    pub dex: Vec<Dex>,
    pub resources: Resources,
    pub files: BTreeMap<String, Vec<u8>>,
}
impl Apk {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let metadata = std::fs::metadata(path)?;
        ensure!(
            metadata.len() <= 256 * 1024 * 1024,
            "APK exceeds 256 MiB limit"
        );
        Self::parse(&std::fs::read(path)?)
            .with_context(|| format!("loading APK {}", path.display()))
    }
    pub fn parse(data: &[u8]) -> Result<Self> {
        let mut archive = zip::ZipArchive::new(Cursor::new(data))?;
        ensure!(archive.len() <= 50_000, "too many APK entries");
        let mut files = BTreeMap::new();
        let mut total = 0;
        for n in 0..archive.len() {
            let mut file = archive.by_index(n)?;
            ensure!(
                file.enclosed_name().is_some() && !file.name().contains('\\'),
                "unsafe APK entry name"
            );
            ensure!(
                file.unix_mode().is_none_or(|m| m & 0o170000 != 0o120000),
                "APK symlinks rejected"
            );
            if file.is_dir() {
                continue;
            }
            ensure!(file.size() <= 64 * 1024 * 1024, "APK entry exceeds 64 MiB");
            total += file.size();
            ensure!(total <= 256 * 1024 * 1024, "expanded APK exceeds 256 MiB");
            let name = file.name().to_owned();
            let mut bytes = vec![];
            file.read_to_end(&mut bytes)?;
            ensure!(files.insert(name, bytes).is_none(), "duplicate APK entry");
        }
        let manifest = Manifest::parse(
            files
                .get("AndroidManifest.xml")
                .context("APK has no AndroidManifest.xml")?,
        )?;
        let mut names = files
            .keys()
            .filter(|n| {
                n.as_str() == "classes.dex"
                    || n.strip_prefix("classes")
                        .and_then(|s| s.strip_suffix(".dex"))
                        .is_some_and(|s| s.parse::<u32>().is_ok_and(|n| n >= 2))
            })
            .cloned()
            .collect::<Vec<_>>();
        names.sort_by_key(|n| {
            if n == "classes.dex" {
                1
            } else {
                n[7..n.len() - 4].parse::<u32>().unwrap_or(u32::MAX)
            }
        });
        let dex = names
            .iter()
            .map(|n| Dex::parse(&files[n]).with_context(|| n.clone()))
            .collect::<Result<Vec<_>>>()?;
        ensure!(
            !dex.is_empty(),
            "APK has no DEX modules (split APKs are unsupported)"
        );
        let resources = files
            .get("resources.arsc")
            .map(|b| Resources::parse(b))
            .transpose()?
            .unwrap_or_default();
        Ok(Self {
            manifest,
            dex,
            resources,
            files,
        })
    }
    pub fn native_libraries(&self) -> Vec<&str> {
        self.files
            .keys()
            .filter(|n| n.starts_with("lib/") && n.ends_with(".so"))
            .map(String::as_str)
            .collect()
    }
}
