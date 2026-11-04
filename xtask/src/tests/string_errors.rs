use super::findings;

#[test]
fn test_reports_signatures_aliases_and_nested_results() {
    assert_eq!(
        findings(
            "no-string-errors",
            r#"
        type R<T> = std::result::Result<T, String>;
        fn f() -> Result<u8, &'static str> { loop {} }
        async fn g() -> Option<core::result::Result<u8, alloc::string::String>> { loop {} }
    "#
        )
        .len(),
        3
    );
}

#[test]
fn test_allows_typed_errors_anyhow_and_test_contracts() {
    assert!(
        findings(
            "no-string-errors",
            r#"
        fn f() -> Result<(), DecodeError> { loop {} }
        fn g() -> anyhow::Result<()> { loop {} }
        fn unit_error() -> Result<(), ()> { loop {} }
        type R<T> = Result<T, std::io::Error>;
        #[test] fn test_f() -> Result<(), String> { loop {} }
    "#
        )
        .is_empty()
    );
}

#[test]
fn test_reports_boxed_standard_errors_in_every_spelling() {
    for contract in [
        "Result<(), Box<dyn Error>>",
        "Result<u8, Box<dyn std::error::Error>>",
        "Result<u8, Box<dyn core::error::Error>>",
        "Result<u8, Box<dyn Error + Send + Sync>>",
        "Result<u8, Box<dyn Error + Send + Sync + 'static>>",
        "std::result::Result<u8, std::boxed::Box<dyn Error>>",
        "Option<Result<u8, alloc::boxed::Box<dyn Error>>>",
    ] {
        let source = format!("pub fn load() -> {contract} {{ loop {{}} }}");
        assert_eq!(findings("no-string-errors", &source).len(), 1, "{source}");
    }
    assert_eq!(
        findings(
            "no-string-errors",
            "type Loaded<T> = Result<T, Box<dyn Error>>;"
        )
        .len(),
        1
    );
}

#[test]
fn test_allows_boxed_errors_where_the_signature_is_fixed_or_typed() {
    for source in [
        "fn main() -> Result<(), Box<dyn Error>> { loop {} }",
        "async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> { loop {} }",
        "impl FromSql for Id { fn from_sql() -> Result<Self, Box<dyn Error + Send + Sync>> { loop {} } }",
        "#[test] fn test_load() -> Result<(), Box<dyn Error>> { loop {} }",
        "fn load() -> Result<(), Box<dyn Other>> { loop {} }",
        "fn load() -> Result<(), Box<LoadError>> { loop {} }",
        "fn load() -> Box<dyn Error> { loop {} }",
        "fn load() -> Result<Box<dyn Error>, LoadError> { loop {} }",
    ] {
        assert!(findings("no-string-errors", source).is_empty(), "{source}");
    }
}

#[test]
fn test_trait_impl_exemption_covers_only_boxed_errors_and_ends_with_the_impl() {
    let source = r#"
        impl Decode for Id {
            fn decode() -> Result<Self, String> { loop {} }
            fn other() -> Result<Self, Box<dyn Error>> { loop {} }
        }

        impl Id {
            fn load() -> Result<Self, Box<dyn Error>> { loop {} }
        }
    "#;
    assert_eq!(findings("no-string-errors", source).len(), 2);
}
