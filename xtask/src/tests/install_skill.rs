//! Exercise direct runtime copies in disposable workspaces.

use std::{
    env, fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

use cargo_metadata::{DependencyKind, MetadataCommand};
use serde_json::{self, Value};
use syn::{
    Attribute, Path as SynPath, parse_file,
    visit::{self, Visit},
};
use tempfile;
use walkdir::WalkDir;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .to_owned()
}

fn write(root: &Path, path: &str, text: &str) {
    let path = root.join(path);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

fn success(output: Output) -> Output {
    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn cargo(directory: &Path, args: &[&str]) -> Output {
    Command::new(env::var_os("CARGO").unwrap_or_else(|| "cargo".into()))
        .args(args)
        .current_dir(directory)
        .env(
            "CARGO_TARGET_DIR",
            root().join("target/vendored-verification"),
        )
        .env("CARGO_NET_OFFLINE", "true")
        .output()
        .unwrap()
}

// Fixture setup mirrors the skill's direct-copy instructions, not an installer.
fn copy_runtime(destination: &Path) {
    assert!(!destination.exists());
    let assets = root().join("skills/install-no-slop-rs/assets");
    let source = assets.join("no_slop");
    for entry in WalkDir::new(&source) {
        let entry = entry.unwrap();
        let target = destination.join(entry.path().strip_prefix(&source).unwrap());
        if entry.file_type().is_dir() {
            fs::create_dir_all(target).unwrap();
        } else {
            fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn fixture(directory: &Path, existing_workspace: bool) {
    let app = if existing_workspace { "app/" } else { "" };
    let package = "[package]\nname = \"app\"\nversion = \"0.1.0\"\nedition = \"2021\"\n";
    let policy = "\n[workspace.metadata.no-slop.rules]\nno-string-errors = \"off\"\n";
    if existing_workspace {
        write(
            directory,
            "Cargo.toml",
            &format!(
                "[workspace]\nmembers = [\"app\", \"xtask\"]\ndefault-members = [\"app\"]\nresolver = \"2\"\n{policy}"
            ),
        );
        write(directory, "app/Cargo.toml", package);
    } else {
        write(
            directory,
            "Cargo.toml",
            &format!("{package}\n[workspace]\nmembers = [\"xtask\"]\nresolver = \"2\"\n{policy}"),
        );
    }
    write(
        directory,
        &format!("{app}src/main.rs"),
        "struct ApplicationData;\n\nfn ignored_contract() -> Result<(), String> { Ok(()) }\n\nfn main() {}\n",
    );
    let requirements =
        fs::read_to_string(root().join("skills/install-no-slop-rs/assets/dependencies.toml"))
            .unwrap();
    write(
        directory,
        "xtask/Cargo.toml",
        &format!(
            "[package]\nname = \"xtask\"\nversion = \"0.1.0\"\nedition = \"2021\"\npublish = false\n{requirements}"
        ),
    );
    write(
        directory,
        ".cargo/config.toml",
        "[alias]\nxtask = \"run --package xtask --\"\n",
    );
    write(
        directory,
        "xtask/src/main.rs",
        r#"
        mod no_slop;

        use std::{env, process::ExitCode};

        fn main() -> ExitCode {
            let mut args = env::args_os().skip(1);
            match args.next().as_deref().and_then(|arg| arg.to_str()) {
                Some("no-slop") => no_slop::run(&args.collect::<Vec<_>>()),
                Some("existing") => { println!("existing command kept"); ExitCode::SUCCESS }
                _ => ExitCode::from(2),
            }
        }
    "#,
    );
}

#[test]
fn test_bundle_contains_only_runtime_sources_and_dependencies() {
    assert_eq!(
        fs::read(root().join("skills/install-no-slop-rs/assets/no_slop/LICENSE")).unwrap(),
        fs::read(root().join("LICENSE")).unwrap(),
        "vendored source must retain its MIT notice"
    );
    #[derive(Default)]
    struct TestCode {
        found: bool,
    }

    impl<'ast> Visit<'ast> for TestCode {
        fn visit_attribute(&mut self, attr: &'ast Attribute) {
            self.found |= attr
                .path()
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "test")
                || (attr.path().is_ident("cfg")
                    && attr
                        .parse_args::<SynPath>()
                        .is_ok_and(|path| path.is_ident("test")));
            visit::visit_attribute(self, attr);
        }
    }

    let assets = root().join("skills/install-no-slop-rs/assets/no_slop");
    for entry in WalkDir::new(&assets) {
        let entry = entry.unwrap();
        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "rs")
        {
            let source = fs::read_to_string(entry.path()).unwrap();
            let syntax = parse_file(&source).unwrap();
            let mut test_code = TestCode::default();
            test_code.visit_file(&syntax);
            assert!(
                !test_code.found,
                "test-only code shipped in {}",
                entry.path().display()
            );
        }
    }

    let directory = tempfile::tempdir().unwrap();
    fixture(directory.path(), false);
    let metadata = MetadataCommand::new()
        .manifest_path(directory.path().join("Cargo.toml"))
        .no_deps()
        .exec()
        .unwrap();
    let xtask = metadata
        .packages
        .iter()
        .find(|package| package.name.as_str() == "xtask")
        .unwrap();
    assert!(!xtask.dependencies.is_empty());
    for dependency in &xtask.dependencies {
        assert_eq!(
            dependency.kind,
            DependencyKind::Normal,
            "{} is not a runtime dependency",
            dependency.name
        );
        assert!(
            dependency.name != "tempfile",
            "author-only dependency shipped: {}",
            dependency.name
        );
    }
}

#[test]
fn test_copied_runtime_runs_and_preserves_local_changes() {
    for existing_workspace in [false, true] {
        let directory = tempfile::tempdir().unwrap();
        fixture(directory.path(), existing_workspace);
        let destination = directory.path().join("xtask/src/no_slop");
        copy_runtime(&destination);
        assert!(!destination.join("tests.rs").exists());
        assert!(!destination.join("tests").exists());
        assert!(destination.join("rules/readable_spacing.rs").is_file());
        assert!(destination.join("rules/comments.rs").is_file());
        assert_eq!(
            fs::read(destination.join("LICENSE")).unwrap(),
            fs::read(root().join("LICENSE")).unwrap()
        );
        // A runtime-only copy must build both normal and test targets without
        // any of this repository's test files or dev dependencies.
        success(cargo(
            directory.path(),
            &["check", "-p", "xtask", "--all-targets", "--quiet"],
        ));
        success(cargo(directory.path(), &["test", "-p", "xtask", "--quiet"]));
        let existing = success(cargo(directory.path(), &["xtask", "existing"]));
        assert_eq!(
            String::from_utf8(existing.stdout).unwrap().trim(),
            "existing command kept"
        );
        let output = cargo(directory.path(), &["xtask", "no-slop", "--format", "json"]);
        assert_eq!(
            output.status.code(),
            Some(1),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["failures"].as_array().unwrap().len(), 0);
        assert_eq!(
            report["diagnostics"].as_array().unwrap().len(),
            1,
            "{report}"
        );
        assert_eq!(report["diagnostics"][0]["rule"], "no-vague-type-names");

        // Keep a local edit while adopting a diagnostic change.
        let rule_path = destination.join("rules/vague_names.rs");
        let base = fs::read_to_string(&rule_path).unwrap();
        let local = format!("{base}\n// local customization retained\n");
        fs::write(&rule_path, &local).unwrap();

        let merged = local.replace(
            "type name `{name}` does not identify a responsibility",
            "type name `{name}` must identify its responsibility",
        );
        fs::write(&rule_path, &merged).unwrap();
        assert!(merged.contains("local customization retained"));
        assert!(merged.contains("must identify its responsibility"));
        success(cargo(directory.path(), &["test", "-p", "xtask", "--quiet"]));
        let output = cargo(directory.path(), &["xtask", "no-slop", "--format", "json"]);
        assert_eq!(output.status.code(), Some(1));
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["diagnostics"].as_array().unwrap().len(), 1);
        assert_eq!(
            report["diagnostics"][0]["message"],
            "type name `ApplicationData` must identify its responsibility"
        );

        // The alias still resolves from a member's source directory.
        let member = if existing_workspace { "app/src" } else { "src" };
        assert_eq!(
            cargo(
                &directory.path().join(member),
                &["xtask", "no-slop", "-p", "app"]
            )
            .status
            .code(),
            Some(1)
        );
        write(
            directory.path(),
            &format!("{member}/spacing.rs"),
            "fn first() {}\n/// Documents second.\nfn second() {}\n",
        );
        let output = cargo(directory.path(), &["xtask", "no-slop", "--format", "json"]);
        assert_eq!(output.status.code(), Some(1));
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        let spacing: Vec<_> = report["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .filter(|diagnostic| diagnostic["rule"] == "require-readable-spacing")
            .collect();
        assert_eq!(spacing.len(), 1, "{report}");
        assert_eq!(spacing[0]["start"]["line"], 2);
        write(
            directory.path(),
            &format!("{member}/spacing.rs"),
            "fn first() {}\n\n/// Documents second.\nfn second() {}\n",
        );
        let output = cargo(directory.path(), &["xtask", "no-slop", "--format", "json"]);
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report["diagnostics"].as_array().unwrap().len(),
            1,
            "{report}"
        );
        write(
            directory.path(),
            &format!("{member}/comments.rs"),
            "// Main logic\nfn explained() {}\n",
        );
        let output = cargo(directory.path(), &["xtask", "no-slop", "--format", "json"]);
        assert_eq!(output.status.code(), Some(1));
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report["diagnostics"].as_array().unwrap().len(),
            2,
            "{report}"
        );
        assert!(
            report["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .any(|diagnostic| diagnostic["rule"] == "no-boilerplate-comments"
                    && diagnostic["start"]["line"] == 1)
        );
        write(
            directory.path(),
            &format!("{member}/comments.rs"),
            "// The FFI entry point must retain its symbol name.\nfn explained() {}\n",
        );
        let output = cargo(directory.path(), &["xtask", "no-slop", "--format", "json"]);
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            report["diagnostics"].as_array().unwrap().len(),
            1,
            "{report}"
        );
        // The copied runtime supports preview and writes without author tooling.
        let fix_path = format!("{member}/fixable.rs");
        let source = "// Main logic\nfn fixed() {}\n\nuse std::env;\nuse std::path::PathBuf;\n";
        write(directory.path(), &fix_path, source);
        let output = cargo(
            directory.path(),
            &["xtask", "no-slop", "--fix-dry-run", "--format", "json"],
        );
        assert_eq!(output.status.code(), Some(1));
        let preview: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(preview["summary"]["files_would_change"], 1);
        assert_eq!(
            fs::read_to_string(directory.path().join(&fix_path)).unwrap(),
            source
        );
        let output = cargo(
            directory.path(),
            &["xtask", "no-slop", "--fix", "--format", "json"],
        );
        assert_eq!(output.status.code(), Some(1));
        let fixed: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(fixed["summary"]["files_changed"], 1);
        assert_eq!(fixed["changes"][0]["diff"], preview["changes"][0]["diff"]);
        assert_eq!(
            fs::read_to_string(directory.path().join(&fix_path)).unwrap(),
            "fn fixed() {}\n\nuse std::{env, path::PathBuf};\n"
        );
        // New policies remain available without author tests or extra dependencies.
        let policy_path = format!("{member}/policies.rs");
        let policy_source = "pub struct Session;\n\nimpl Session {\n    pub fn new() -> Option<Self> { None }\n}\n\npub fn failure() -> std::io::Error {\n    std::io::Error::other(\"Failed to read configuration\")\n}\n";
        write(directory.path(), &policy_path, policy_source);
        let main_path = directory.path().join(format!("{member}/main.rs"));
        let mut main = fs::read_to_string(&main_path).unwrap();
        main.push_str("\nmod policies;\n");
        fs::write(main_path, main).unwrap();
        let output = cargo(
            directory.path(),
            &["xtask", "no-slop", "--fix", "--format", "json"],
        );
        assert_eq!(output.status.code(), Some(1));
        let policies: Value = serde_json::from_slice(&output.stdout).unwrap();
        for rule in [
            "require-lowercase-error-messages",
            "require-create-for-fallible-constructors",
        ] {
            let diagnostics: Vec<_> = policies["diagnostics"]
                .as_array()
                .unwrap()
                .iter()
                .filter(|diagnostic| diagnostic["rule"] == rule)
                .collect();
            assert_eq!(diagnostics.len(), 1, "{policies}");
            assert!(diagnostics[0].get("fix").is_none());
        }
        assert_eq!(policies["summary"]["files_changed"], 0);
        assert_eq!(
            fs::read_to_string(directory.path().join(&policy_path)).unwrap(),
            policy_source
        );
        success(cargo(
            directory.path(),
            &["check", "--workspace", "--all-targets", "--locked"],
        ));
        write(
            directory.path(),
            &format!(
                "{}/broken.rs",
                if existing_workspace { "app/src" } else { "src" }
            ),
            "fn {",
        );
        let output = cargo(directory.path(), &["xtask", "no-slop", "--format", "json"]);
        assert_eq!(output.status.code(), Some(2));
        let report: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(report["failures"].as_array().unwrap().len(), 1);
    }
}
