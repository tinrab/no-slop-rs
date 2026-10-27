//! Keep the checked-in demo aligned with the actual scanner rule registry.

use std::{collections::BTreeSet, path::Path, process::Command};

use serde_json::{self, Value};

#[test]
fn test_demo_app_triggers_every_registered_rule_exactly_once() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).parent().unwrap();
    let binary = env!("CARGO_BIN_EXE_xtask");
    let listed = Command::new(binary)
        .args(["no-slop", "--list-rules", "--format", "json"])
        .current_dir(root)
        .output()
        .unwrap();
    assert!(
        listed.status.success(),
        "{}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let rules: Value = serde_json::from_slice(&listed.stdout).unwrap();
    let expected: BTreeSet<_> = rules
        .as_array()
        .unwrap()
        .iter()
        .map(|rule| rule["id"].as_str().unwrap())
        .collect();

    let scanned = Command::new(binary)
        .args(["no-slop", "-p", "demo-app", "--format", "json"])
        .current_dir(root)
        .output()
        .unwrap();
    assert_eq!(
        scanned.status.code(),
        Some(1),
        "{}",
        String::from_utf8_lossy(&scanned.stderr)
    );
    let report: Value = serde_json::from_slice(&scanned.stdout).unwrap();
    assert!(
        report["failures"].as_array().unwrap().is_empty(),
        "{report}"
    );
    let diagnostics = report["diagnostics"].as_array().unwrap();
    let actual: BTreeSet<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic["rule"].as_str().unwrap())
        .collect();
    assert_eq!(
        actual, expected,
        "demo coverage differs from the rule registry"
    );
    assert_eq!(
        diagnostics.len(),
        expected.len(),
        "expected one example per rule: {report}"
    );
}
