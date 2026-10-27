//! Actual CLI runs against disposable application workspaces.

use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use serde_json::{self, Value};
use tempfile::{self, TempDir};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned()
}

fn write(directory: &Path, path: &str, source: &str) {
    let path = directory.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

fn cargo(directory: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO"))
        .args(args)
        .current_dir(directory)
        .env("CARGO_NET_OFFLINE", "true")
        .env(
            "CARGO_TARGET_DIR",
            root().join("target/fix-cli-verification"),
        )
        .output()
        .unwrap()
}

fn lock(directory: &Path) {
    let output = cargo(directory, &["generate-lockfile"]);
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn scan(directory: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["no-slop", "--manifest-path"])
        .arg(directory.join("Cargo.toml"))
        .args(["--format", "json"])
        .args(args)
        .output()
        .unwrap()
}

fn json(output: &Output, status: i32) -> Value {
    assert_eq!(
        output.status.code(),
        Some(status),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

fn demo() -> (TempDir, String) {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "Cargo.toml",
        "[package]\nname=\"fix-demo\"\nversion=\"0.1.0\"\nedition=\"2024\"\n\n[workspace]\n",
    );
    let source = fs::read_to_string(root().join("demo-app/src/lib.rs")).unwrap();
    write(directory.path(), "src/lib.rs", &source);
    lock(directory.path());
    (directory, source)
}

#[test]
fn test_demo_preview_apply_remaining_findings_compilation_and_idempotence() {
    let (directory, original) = demo();
    let path = directory.path().join("src/lib.rs");
    let initial = json(&scan(directory.path(), &[]), 1);
    assert_eq!(initial["diagnostics"].as_array().unwrap().len(), 13);
    assert_eq!(
        initial["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|d| d.get("fix").is_some())
            .count(),
        4
    );
    assert_eq!(fs::read_to_string(&path).unwrap(), original);

    let dry = json(&scan(directory.path(), &["--fix-dry-run"]), 1);
    assert_eq!(dry["diagnostics"].as_array().unwrap().len(), 13);
    assert_eq!(dry["summary"]["files_changed"], 0);
    assert_eq!(dry["summary"]["files_would_change"], 1);
    assert_eq!(dry["changes"][0]["applied"], false);
    assert_eq!(fs::read_to_string(&path).unwrap(), original);
    let text = scan(directory.path(), &["--fix-dry-run", "--format", "text"]);
    assert_eq!(text.status.code(), Some(1));
    let text = String::from_utf8(text.stdout).unwrap();
    assert!(
        text.contains("--- a/src/lib.rs") && text.contains("+++ b/src/lib.rs"),
        "{text}"
    );

    let applied = json(&scan(directory.path(), &["--fix"]), 1);
    assert_eq!(applied["diagnostics"].as_array().unwrap().len(), 9);
    assert!(
        applied["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .all(|d| d.get("fix").is_none())
    );
    assert_eq!(applied["summary"]["files_changed"], 1);
    assert_eq!(applied["summary"]["files_would_change"], 0);
    assert_eq!(applied["changes"][0]["applied"], true);
    assert_eq!(applied["changes"][0]["diff"], dry["changes"][0]["diff"]);
    assert!(applied["failures"].as_array().unwrap().is_empty());
    let fixed = fs::read_to_string(&path).unwrap();
    assert_ne!(fixed, original);
    let again = json(&scan(directory.path(), &["--fix"]), 1);
    assert_eq!(again["summary"]["files_changed"], 0);
    assert!(again.get("changes").is_none());
    assert_eq!(fs::read_to_string(&path).unwrap(), fixed);
    let checked = cargo(directory.path(), &["check", "--all-targets", "--locked"]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert_eq!(
        fs::read_to_string(root().join("demo-app/src/lib.rs")).unwrap(),
        original
    );
}

#[test]
fn test_selection_exclusions_warnings_off_suppressions_and_partial_failures() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "Cargo.toml",
        "[workspace]\nmembers=[\"app\",\"other\"]\nresolver=\"2\"\n\n[workspace.metadata.no-slop]\nexclude=[\"app/src/excluded.rs\"]\n\n[workspace.metadata.no-slop.rules]\nno-boilerplate-comments=\"warn\"\nno-decorative-comments=\"off\"\n",
    );
    for name in ["app", "other"] {
        write(
            directory.path(),
            &format!("{name}/Cargo.toml"),
            &format!("[package]\nname=\"{name}\"\nversion=\"0.1.0\"\nedition=\"2021\"\n"),
        );
    }
    let source = "// no-slop-allow: no-boilerplate-comments -- conventional label\n// Main logic\nfn kept() {}\n\n// Main logic\nfn fixed() {}\n\n// ========\nfn disabled() {}\n";
    write(directory.path(), "app/src/lib.rs", source);
    write(directory.path(), "app/src/excluded.rs", "not Rust");
    write(
        directory.path(),
        "app/src/generated.rs",
        "// @generated\nnot Rust",
    );
    write(
        directory.path(),
        "other/src/lib.rs",
        "// Main logic\nfn other() {}\n",
    );
    lock(directory.path());
    let preview = json(&scan(directory.path(), &["--fix-dry-run", "-p", "app"]), 0);
    assert_eq!(preview["summary"]["files_would_change"], 1);
    assert_eq!(preview["diagnostics"][0]["level"], "warn");
    write(directory.path(), "app/src/broken.rs", "fn {");
    let invalid = "// no-slop-allow: unknown -- invalid\n// Main logic\nfn invalid() {}\n";
    write(directory.path(), "app/src/invalid.rs", invalid);
    let fixed = json(&scan(directory.path(), &["--fix", "-p", "app"]), 2);
    assert_eq!(fixed["summary"]["files_changed"], 1);
    assert_eq!(fixed["failures"].as_array().unwrap().len(), 2);
    let actual = fs::read_to_string(directory.path().join("app/src/lib.rs")).unwrap();
    assert_eq!(actual.matches("// Main logic").count(), 1);
    assert!(actual.contains("// ========"));
    assert_eq!(
        fs::read_to_string(directory.path().join("app/src/invalid.rs")).unwrap(),
        invalid
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("other/src/lib.rs")).unwrap(),
        "// Main logic\nfn other() {}\n"
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("app/src/excluded.rs")).unwrap(),
        "not Rust"
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("app/src/generated.rs")).unwrap(),
        "// @generated\nnot Rust"
    );
}

#[test]
fn test_fix_metadata_and_argument_conflicts() {
    let listed = Command::new(env!("CARGO_BIN_EXE_xtask"))
        .args(["no-slop", "--list-rules", "--format", "json"])
        .output()
        .unwrap();
    let rules = json(&listed, 0);
    assert_eq!(
        rules
            .as_array()
            .unwrap()
            .iter()
            .filter(|r| r["fixable"] == true)
            .count(),
        4
    );
    for args in [
        vec!["--fix", "--fix-dry-run"],
        vec!["--fix", "--list-rules"],
        vec!["--fix-dry-run", "--list-rules"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_xtask"))
            .arg("no-slop")
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
    }
}

#[test]
fn test_grouping_preserves_value_and_type_namespace_bindings() {
    let directory = tempfile::tempdir().unwrap();
    write(
        directory.path(),
        "Cargo.toml",
        "[package]\nname=\"import-demo\"\nversion=\"0.1.0\"\nedition=\"2024\"\n[workspace]\n",
    );
    let source = r#"
mod domain {
    pub mod entry { pub const VALUE: u8 = 7; }

    pub fn entry() -> u8 { 3 }
}

use domain::entry;
use domain::entry::VALUE;

pub fn result() -> u8 { entry() + VALUE }
"#;
    write(directory.path(), "src/lib.rs", source);
    lock(directory.path());
    let preview = json(&scan(directory.path(), &["--fix-dry-run"]), 1);
    let applied = json(&scan(directory.path(), &["--fix"]), 0);
    assert_eq!(preview["changes"][0]["diff"], applied["changes"][0]["diff"]);
    let checked = cargo(directory.path(), &["check", "--all-targets", "--locked"]);
    assert!(
        checked.status.success(),
        "{}",
        String::from_utf8_lossy(&checked.stderr)
    );
    assert_eq!(
        json(&scan(directory.path(), &["--fix"]), 0)["summary"]["files_changed"],
        0
    );
}
