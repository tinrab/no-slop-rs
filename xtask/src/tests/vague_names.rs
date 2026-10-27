use super::findings;

#[test]
fn test_reports_owned_types_and_traits() {
    assert_eq!(
        findings(
            "no-vague-type-names",
            "struct UserManager; enum FileData {} trait ParseHelper {} type RowProcessor = (); "
        )
        .len(),
        4
    );
}

#[test]
fn test_leaves_imports_values_and_specific_names_alone() {
    assert!(
        findings(
            "no-vague-type-names",
            "use foreign::UserManager; struct Metadata; struct Database; fn f() { let data = 0; }"
        )
        .is_empty()
    );
}
