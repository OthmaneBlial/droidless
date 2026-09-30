//! Host-approved package directory capabilities; no guest path reaches ambient I/O.
use anyhow::{Context, Result, ensure};
use cap_fs_ext::{DirExt, FollowSymlinks, OpenOptionsFollowExt, OpenOptionsSyncExt};
#[cfg(unix)]
use cap_std::fs::{DirBuilderExt, OpenOptionsExt};
use cap_std::{
    ambient_authority,
    fs::{Dir, DirBuilder, OpenOptions},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
    path::Path,
    time::{SystemTime, UNIX_EPOCH},
};

pub(crate) const MAX_BYTES: usize = 1_048_576;

#[derive(Serialize, Deserialize)]
pub(crate) enum Value {
    String(String),
    Int(i32),
    Long(i64),
    Float(u32), // Preserve all Java float bit patterns, including NaNs.
    Boolean(bool),
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    schema: u32,
    values: BTreeMap<String, Value>,
}
pub(crate) struct Storage {
    dir: Dir,
    next_temp: u128,
}

pub(crate) fn preference_name(name: &str) -> Result<()> {
    ensure!(
        !name.is_empty()
            && name.len() <= 120
            && name != "."
            && name != ".."
            && name
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b"._-$".contains(&c)),
        "preference name must be 1–120 ASCII letters/digits/._-$ without path components"
    );
    Ok(())
}
fn preference_file(name: &str) -> Result<String> {
    preference_name(name)?;
    // Hex preserves Android's case-sensitive names on case-insensitive host filesystems.
    let mut filename = String::with_capacity(name.len() * 2 + 5);
    for byte in name.bytes() {
        use std::fmt::Write;
        write!(filename, "{byte:02x}")?;
    }
    filename.push_str(".json");
    Ok(filename)
}
fn child_dir(parent: &Dir, name: &str) -> Result<Dir> {
    let mut builder = DirBuilder::new();
    #[cfg(unix)]
    builder.mode(0o700);
    match parent.create_dir_with(name, &builder) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(e.into()),
    }
    parent
        .open_dir_nofollow(name)
        .with_context(|| format!("storage directory must not be a symlink: {name}"))
}
fn regular_file(file: &std::fs::File) -> Result<()> {
    let metadata = file.metadata()?;
    ensure!(
        metadata.is_file(),
        "preference storage must be a regular file"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ensure!(
            metadata.nlink() == 1,
            "hard-linked preference storage rejected"
        );
    }
    Ok(())
}
fn package_identity(dir: &Dir, package: &str) -> Result<()> {
    let mut create = OpenOptions::new();
    create
        .write(true)
        .create_new(true)
        .follow(FollowSymlinks::No);
    #[cfg(unix)]
    create.mode(0o600);
    match dir.open_with(".droidless-package", &create) {
        Ok(mut file) => {
            file.write_all(package.as_bytes())?;
            file.sync_all()?;
            #[cfg(unix)]
            dir.try_clone()?.into_std_file().sync_all()?;
            Ok(())
        }
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {
            let mut read = OpenOptions::new();
            read.read(true).follow(FollowSymlinks::No).nonblock(true);
            let file = dir.open_with(".droidless-package", &read)?.into_std();
            regular_file(&file)?;
            let mut identity = vec![];
            file.take(241).read_to_end(&mut identity)?;
            ensure!(
                identity == package.as_bytes(),
                "storage package identity mismatch (possible case collision)"
            );
            Ok(())
        }
        Err(e) => Err(e.into()),
    }
}
impl Storage {
    pub(crate) fn open(apps_dir: &Path, package: &str) -> Result<Self> {
        ensure!(
            package.len() <= 240
                && package.split('.').all(|part| {
                    part.as_bytes()
                        .first()
                        .is_some_and(|c| c.is_ascii_alphabetic() || *c == b'_')
                        && part.bytes().all(|c| c.is_ascii_alphanumeric() || c == b'_')
                }),
            "invalid storage package identifier"
        );
        // This is the only ambient filesystem access, selected by the host, never the APK.
        let mut builder = std::fs::DirBuilder::new();
        builder.recursive(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(apps_dir)?;
        let apps = Dir::open_ambient_dir(apps_dir, ambient_authority())?;
        let app = child_dir(&apps, package)?;
        package_identity(&app, package)?;
        Ok(Self {
            dir: child_dir(&app, "shared_prefs")?,
            next_temp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos(),
        })
    }
    pub(crate) fn load(&self, name: &str) -> Result<BTreeMap<String, Value>> {
        let filename = preference_file(name)?;
        let mut options = OpenOptions::new();
        options.read(true).follow(FollowSymlinks::No).nonblock(true);
        let file = match self.dir.open_with(filename, &options) {
            Ok(file) => file.into_std(),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(BTreeMap::new()),
            Err(e) => return Err(e).context("cannot read isolated preferences"),
        };
        regular_file(&file)?;
        let mut bytes = vec![];
        file.take((MAX_BYTES + 1) as u64).read_to_end(&mut bytes)?;
        ensure!(bytes.len() <= MAX_BYTES, "preference file exceeds 1 MiB");
        let file: File = serde_json::from_slice(&bytes)
            .context("invalid preference file; refusing to overwrite it")?;
        ensure!(
            file.schema == 1 && file.values.len() <= 16_384,
            "unsupported preference schema or entry count"
        );
        Ok(file.values)
    }
    pub(crate) fn save(&mut self, name: &str, values: BTreeMap<String, Value>) -> Result<()> {
        let target = preference_file(name)?;
        ensure!(values.len() <= 16_384, "preference entry limit reached");
        let bytes = serde_json::to_vec(&File { schema: 1, values })?;
        ensure!(bytes.len() <= MAX_BYTES, "preference file exceeds 1 MiB");
        // Reject existing links/special files as well as link escapes at the directory boundary.
        match self.dir.symlink_metadata(&target) {
            Ok(meta) => {
                ensure!(
                    meta.is_file() && !meta.file_type().is_symlink(),
                    "invalid preference destination"
                );
                let mut read = OpenOptions::new();
                read.read(true).follow(FollowSymlinks::No).nonblock(true);
                regular_file(&self.dir.open_with(&target, &read)?.into_std())?;
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
        let mut options = OpenOptions::new();
        options
            .write(true)
            .create_new(true)
            .follow(FollowSymlinks::No);
        #[cfg(unix)]
        options.mode(0o600);
        let mut temporary = None;
        for _ in 0..32 {
            self.next_temp = self
                .next_temp
                .checked_add(1)
                .context("temporary file counter exhausted")?;
            let name = format!(".tmp-{}-{}", std::process::id(), self.next_temp);
            match self.dir.open_with(&name, &options) {
                Ok(file) => {
                    temporary = Some((name, file));
                    break;
                }
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(e) => return Err(e.into()),
            }
        }
        let (name, mut file) = temporary.context("temporary preference file limit reached")?;
        let result = (|| -> Result<()> {
            file.write_all(&bytes)?;
            file.sync_all()?;
            self.dir.rename(&name, &self.dir, &target)?;
            #[cfg(unix)]
            self.dir.try_clone()?.into_std_file().sync_all()?;
            Ok(())
        })();
        if result.is_err() {
            // Only the newly created temporary file is eligible for cleanup.
            let _ = self.dir.remove_file(&name);
        }
        result
    }
}
