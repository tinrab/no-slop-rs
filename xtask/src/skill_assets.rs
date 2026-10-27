//! Ship the tested scanner runtime and its dependency requirements.

use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context as _, Result, ensure};
use cargo_metadata::{DependencyKind, MetadataCommand};
use serde_json;
use walkdir::WalkDir;

pub fn run(args: &[OsString]) -> Result<()> {
    ensure!(
        args.is_empty() || args == [OsString::from("--check")],
        "usage: sync-no-slop-skill-assets [--check]"
    );
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .context("xtask needs a repository parent")?;
    sync(root, !args.is_empty())
}

fn sync(root: &Path, check: bool) -> Result<()> {
    let source = root.join("xtask/src/no_slop");
    let destination = root.join("skills/install-no-slop-rs/assets");
    let mut expected = BTreeMap::new();
    for (path, bytes) in read_tree(&source)? {
        expected.insert(PathBuf::from("no_slop").join(path), bytes);
    }
    expected.insert("no_slop/LICENSE".into(), fs::read(root.join("LICENSE"))?);
    expected.insert("dependencies.toml".into(), dependencies(root)?.into_bytes());
    if check {
        ensure!(
            read_tree(&destination)? == expected,
            "skill assets are stale; run cargo xtask sync-no-slop-skill-assets"
        );
        println!("Skill assets match canonical source and dependency requirements.");
    } else {
        if destination.exists() {
            fs::remove_dir_all(&destination)?;
        }
        for (path, bytes) in &expected {
            let path = destination.join(path);
            fs::create_dir_all(path.parent().context("asset has no parent")?)?;
            fs::write(path, bytes)?;
        }
        println!("Synced {} skill assets.", expected.len());
    }
    Ok(())
}

fn read_tree(root: &Path) -> Result<BTreeMap<PathBuf, Vec<u8>>> {
    let mut files = BTreeMap::new();
    if !root.exists() {
        return Ok(files);
    }
    for entry in WalkDir::new(root).follow_links(false) {
        let entry = entry?;
        ensure!(
            !entry.file_type().is_symlink(),
            "asset cannot be a symlink: {}",
            entry.path().display()
        );
        if entry.file_type().is_file() {
            files.insert(
                entry.path().strip_prefix(root)?.to_owned(),
                fs::read(entry.path())?,
            );
        }
    }
    Ok(files)
}

fn dependencies(root: &Path) -> Result<String> {
    let metadata = MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .no_deps()
        .exec()?;
    let package = metadata
        .packages
        .iter()
        .find(|p| p.name.as_str() == "xtask")
        .context("missing xtask package")?;
    let mut dependencies = BTreeMap::new();
    for dependency in &package.dependencies {
        // Authoring and test dependencies stay in this repository.
        if dependency.kind != DependencyKind::Normal {
            continue;
        }
        let features = serde_json::to_string(&dependency.features)?;
        let declaration = format!(
            "{{ version = \"{}\", features = {features}, default-features = {} }}",
            dependency.req, dependency.uses_default_features
        );
        dependencies.insert(dependency.name.clone(), declaration);
    }
    let mut text = String::from(
        "# Runtime requirements from the tested xtask manifest. Merge into the target xtask manifest.\n\n[dependencies]\n",
    );
    for (name, declaration) in dependencies {
        text.push_str(&format!("{name} = {declaration}\n"));
    }
    Ok(text)
}
