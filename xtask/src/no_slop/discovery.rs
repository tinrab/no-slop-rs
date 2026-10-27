//! Enumerate workspace-owned Rust source without compiling application targets.

use std::{
    collections::{BTreeMap, BTreeSet},
    path::PathBuf,
};

use anyhow::{Context as _, Result, ensure};
use cargo_metadata::{MetadataCommand, TargetKind};
use walkdir::WalkDir;

use super::{
    cli::Options,
    config::{self, Config},
    diagnostic::{Failure, Report},
};

pub(crate) struct SourceFile {
    pub path: PathBuf,
    pub is_test: bool,
    pub is_crate_root: bool,
}

pub(crate) struct Workspace {
    pub root: PathBuf,
    pub config: Config,
    pub files: Vec<SourceFile>,
    pub report: Report,
}

pub(crate) fn discover(options: &Options) -> Result<Workspace> {
    let mut command = MetadataCommand::new();
    command.no_deps().other_options(vec!["--locked".into()]);
    if let Some(path) = &options.manifest_path {
        let path = path
            .canonicalize()
            .with_context(|| format!("locating manifest {}", path.display()))?;
        command
            .current_dir(path.parent().context("manifest has no parent")?)
            .manifest_path(path);
    }
    let metadata = command.exec().context("discovering Cargo workspace")?;
    let root = metadata.workspace_root.into_std_path_buf();
    let target = metadata.target_directory.into_std_path_buf();
    let config = Config::parse(&metadata.workspace_metadata)?;
    let exclusions = config.exclusions()?;
    let members: Vec<_> = metadata
        .packages
        .iter()
        .filter(|package| metadata.workspace_members.contains(&package.id))
        .collect();
    for name in &options.packages {
        ensure!(
            members.iter().any(|package| package.name.as_str() == name),
            "unknown workspace package: {name}"
        );
    }
    let mut files = BTreeMap::<PathBuf, (bool, bool)>::new();
    let mut report = Report::default();
    let mut visited = BTreeSet::new();
    let mut skipped_directories = BTreeSet::new();
    for package in members.iter().filter(|package| {
        options.packages.is_empty()
            || options
                .packages
                .iter()
                .any(|name| name == package.name.as_str())
    }) {
        let package_root = package
            .manifest_path
            .parent()
            .context("package manifest has no parent")?
            .as_std_path();
        // Cargo can retain `..` in custom target paths. Resolve them before
        // checking ownership, otherwise an external target looks package-local.
        let mut targets = Vec::new();
        for target in &package.targets {
            match target.src_path.as_std_path().canonicalize() {
                Ok(path) => targets.push((path, target.kind.contains(&TargetKind::Test))),
                Err(error) => report.failures.push(Failure {
                    path: target
                        .src_path
                        .as_std_path()
                        .strip_prefix(&root)
                        .unwrap_or(target.src_path.as_std_path())
                        .to_owned(),
                    message: format!("locating target source: {error}"),
                    position: None,
                }),
            }
        }
        let test_targets: BTreeSet<_> = targets
            .iter()
            .filter(|(_, test)| *test)
            .map(|(path, _)| path.as_path())
            .collect();
        let production_targets: BTreeSet<_> = targets
            .iter()
            .filter(|(_, test)| !*test)
            .map(|(path, _)| path.as_path())
            .collect();
        let mut roots = BTreeSet::from([package_root.to_owned()]);
        for (target, _) in &targets {
            if !target.starts_with(package_root) {
                roots.insert(
                    target
                        .parent()
                        .context("target source has no parent")?
                        .to_owned(),
                );
            }
        }
        for scan_root in roots {
            for entry in WalkDir::new(&scan_root)
                .follow_links(false)
                .into_iter()
                .filter_entry(|entry| {
                    let path = entry.path();
                    if entry.depth() == 0 || !entry.file_type().is_dir() {
                        return true;
                    }
                    // A nested package is visited through its own workspace membership.
                    if path.join("Cargo.toml").is_file() {
                        return false;
                    }
                    let relative = path.strip_prefix(&root).unwrap_or(path);
                    let excluded = path.starts_with(&target)
                        || config::default_excluded(relative)
                        || exclusions.is_match(relative);
                    if excluded {
                        skipped_directories.insert(path.to_owned());
                    }
                    !excluded
                })
            {
                let entry = match entry {
                    Ok(entry) => entry,
                    Err(error) => {
                        report.failures.push(Failure {
                            path: error
                                .path()
                                .unwrap_or(&scan_root)
                                .strip_prefix(&root)
                                .unwrap_or(error.path().unwrap_or(&scan_root))
                                .to_owned(),
                            message: error.to_string(),
                            position: None,
                        });
                        continue;
                    }
                };
                let path = entry.path();
                if entry.file_type().is_dir() {
                    continue;
                }
                if entry.file_type().is_symlink() {
                    if visited.insert(path.to_owned()) {
                        report.summary.skipped += 1;
                    }
                    continue;
                }
                if path.extension().is_none_or(|extension| extension != "rs") {
                    continue;
                }
                let first_visit = visited.insert(path.to_owned());
                let relative = path.strip_prefix(&root).unwrap_or(path);
                if path.starts_with(&target)
                    || config::default_excluded(relative)
                    || exclusions.is_match(relative)
                {
                    if first_visit {
                        report.summary.skipped += 1;
                    }
                    continue;
                }
                let is_test = (path
                    .strip_prefix(package_root)
                    .is_ok_and(|relative| relative.starts_with("tests"))
                    || test_targets.contains(path))
                    && !production_targets.contains(path);
                files
                    .entry(path.to_owned())
                    .and_modify(|(test, crate_root)| {
                        *test &= is_test;
                        *crate_root |=
                            test_targets.contains(path) || production_targets.contains(path);
                    })
                    .or_insert((
                        is_test,
                        test_targets.contains(path) || production_targets.contains(path),
                    ));
            }
        }
    }
    report.summary.skipped += skipped_directories.len();
    Ok(Workspace {
        root,
        config,
        files: files
            .into_iter()
            .map(|(path, (is_test, is_crate_root))| SourceFile {
                path,
                is_test,
                is_crate_root,
            })
            .collect(),
        report,
    })
}
