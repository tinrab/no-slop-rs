use super::findings;

#[test]
fn test_rejects_restricted_paths_and_points_to_the_visibility() {
    for visibility in [
        "pub(super)",
        "pub(self)",
        "pub(in crate::a)",
        "pub(in super)",
    ] {
        let source = format!("{visibility} fn f() {{}}");
        let result = findings("no-restricted-visibility", &source);
        assert_eq!(result.len(), 1, "{source}");
        assert_eq!(
            &source[result[0].start.byte..result[0].end.byte],
            visibility
        );
    }
}

#[test]
fn test_checks_fields_associated_items_and_nested_modules() {
    let source = r"
        struct S { pub(super) field: u8 }
        impl S { pub(super) fn f() {} }
        mod nested { pub(super) use super::S; }
    ";
    assert_eq!(findings("no-restricted-visibility", source).len(), 3);
}

#[test]
fn test_accepts_private_crate_and_public_visibility() {
    let source = "fn a() {}\npub(crate) fn b() {}\npub fn c() {}\nstruct S(pub(crate) u8);";
    assert!(findings("no-restricted-visibility", source).is_empty());
}
