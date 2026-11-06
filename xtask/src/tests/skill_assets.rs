//! Keep the skill bundle synchronized and limited to runtime files.

use std::{ffi::OsString, fs, path::Path};

use tempfile::TempDir;

use crate::skill_assets::{read_source, run};

fn source_tree(files: &[&str]) -> TempDir {
    let directory = tempfile::tempdir().unwrap();
    for file in files {
        let path = directory.path().join(file);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, "").unwrap();
    }
    directory
}

#[test]
fn test_committed_assets_match_generated_assets() {
    run(&[OsString::from("--check")]).unwrap();
}

#[test]
fn test_source_accepts_rust_files_and_root_provenance() {
    let directory = source_tree(&["UPSTREAM.md", "mod.rs", "rules/comments.rs"]);

    let files = read_source(directory.path()).unwrap();

    assert_eq!(files.len(), 3);
    assert!(files.contains_key(Path::new("rules/comments.rs")));
}

#[test]
fn test_source_rejects_files_outside_the_runtime() {
    for stray in ["mod.rs.orig", "notes.md", "LICENSE", "rules/UPSTREAM.md"] {
        let directory = source_tree(&["UPSTREAM.md", "mod.rs", stray]);

        let error = read_source(directory.path()).unwrap_err().to_string();

        assert!(error.contains(stray), "{stray}: {error}");
    }
}
