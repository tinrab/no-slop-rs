# no-slop-rs

Editable Rust lints for common patterns coding agents leave behind.

The rules live in your repository and run through `cargo xtask no-slop`.

This is not a replacement for [Clippy](https://github.com/rust-lang/rust-clippy).
Every Rust project should run Clippy; no-slop adds checks Clippy does not cover and avoids duplicating its lints.

## Install the skill

From your target Cargo workspace, install the skill:

```sh
npx skills add tinrab/no-slop-rs --skill install-no-slop-rs
```

Then ask your coding agent:

> Use $install-no-slop-rs to add `cargo xtask no-slop` to this workspace, preserving existing xtask commands.

The skill integrates with an existing xtask or creates one inside an existing Cargo workspace.
For a standalone project outside a workspace, it explains the limitation and leaves the project unchanged.
It copies editable runtime source and declares dependencies in the xtask manifest, reusing existing workspace dependencies where available.
You can also point your agent directly at [SKILL.md](skills/install-no-slop-rs/SKILL.md) in this checkout.

## Run and customize

```sh
cargo xtask no-slop
cargo xtask no-slop -p my-crate
cargo xtask no-slop --fix-dry-run
cargo xtask no-slop --fix
cargo xtask no-slop --list-rules
cargo xtask no-slop --format json
```

The scanner reads source without building application crates or expanding macros.
All rules default to errors.
Preview mechanical fixes with `--fix-dry-run`, then apply them with `--fix`.
Fixes cover import grouping, declaration spacing, decorative comments, exact boilerplate comments, and doc comment sentence lines.
Edit the copied rules or [configure levels, exclusions, and local exceptions](skills/install-no-slop-rs/references/configuration.md) to fit your project.
Exit codes are 0 for success, 1 for error findings, and 2 for invalid input or incomplete coverage.

## Rules

Each diff replaces a violation with an accepted alternative.
These are suggested corrections; only the five rules listed above support automatic fixes.
Checks follow written syntax without resolving types, imported aliases, or macro expansions.
See the [configuration reference](skills/install-no-slop-rs/references/configuration.md) for coverage limits and exceptions.

### `no-vague-error-messages`

Replace complete generic messages with an explanation that identifies the operation or failure.

```diff
-result.context("operation failed")
+result.context("reading configuration")
```

### `no-duplicate-error-context`

Avoid adjacent literal contexts that repeat the same message, including literal-only lazy closures.

```diff
 load()
     .context("reading configuration")
-    .with_context(|| "reading configuration")
```

### `require-error-sources`

Declare the source when a thiserror wrapper retains a recognized standard error.

```diff
 #[derive(Debug, thiserror::Error)]
 enum ConfigError {
     #[error("could not read configuration")]
-    Read(io::Error),
+    Read(#[source] io::Error),
 }
```

### `no-vague-test-names`

Replace placeholder or numeric-only test names with descriptions of the behavior being checked.

```diff
 #[test]
-fn test_basic() {
+fn test_rejects_expired_token() {
     assert_eq!(verify(expired_token()), Err(TokenError::Expired));
 }
```

### `require-lowercase-error-messages`

Begin error-message prose with lowercase text while preserving acronyms and identifiers later in the message.

```diff
-bail!("Failed to read HTTP response");
+bail!("failed to read HTTP response");
```

### `require-create-for-fallible-constructors`

Use `create` or `create_*` for associated `new` functions that explicitly return `Result` or `Option`.
Trait implementations are exempt because their names are constrained by the trait.

```diff
 impl Connection {
-    fn new() -> Result<Self, ConnectError> {
+    fn create() -> Result<Self, ConnectError> {
         connect()
     }
 }
```

### `require-grouped-imports`

Merge shared import prefixes and separate standard-library, external, and local roots with blank lines.

```diff
-use std::env;
-use std::path::PathBuf;
+use std::{env, path::PathBuf};
+
 use crate::Config;
```

### `require-test-prefix`

Start test-attributed function names with `test_`.

```diff
 #[test]
-fn parses_valid_input() {
+fn test_parses_valid_input() {
     assert!(parse("42").is_ok());
 }
```

### `require-readable-spacing`

Separate sibling declarations with a blank line, allowing compact groups of imports, external modules, and single-line constants.

```diff
 fn first() {}
+
 fn second() {}
```

### `no-string-errors`

Use meaningful error types instead of `String` or `&str` in `Result` error contracts outside test scopes.

```diff
-fn load() -> Result<Config, String> {
+fn load() -> Result<Config, ConfigError> {
     read_config()
 }
```

### `no-blanket-lint-suppression`

Keep lint exceptions narrow instead of suppressing `warnings` or `clippy::all`.

```diff
-#[allow(warnings)]
+#[allow(dead_code, reason = "called by the host integration")]
 fn host_entry() {}
```

### `no-vague-type-names`

Name a concrete responsibility instead of a generic `Manager`, `Helper`, `Processor`, or `Data` type.

```diff
-struct UserManager;
+struct SessionStore;
```

### `no-decorative-comments`

Replace decorative separators and framed headings with plain explanations.

```diff
-// ==== Authentication ====
+// Reject expired sessions before loading cached permissions.
 validate_session();
```

### `no-boilerplate-comments`

Remove exact boilerplate labels that add no explanation.
Scaffolding phrases such as `Your code here` are also reported, but remain unchanged by `--fix` because they may indicate unfinished behavior.

```diff
-// Main logic
 process_request();
```

### `no-vague-todos`

Describe the specific remaining change instead of leaving an empty or generic TODO/FIXME.

```diff
-// TODO: improve this
+// TODO: reject expired tokens before consulting the session cache.
```

### `no-stringified-error-mapping`

Preserve the original error instead of directly mapping it to its display string.

```diff
-parse_config(input).map_err(|error| error.to_string())
+parse_config(input).map_err(ConfigError::Parse)
```

### `no-restricted-visibility`

Use private, `pub(crate)`, or `pub` visibility instead of `pub(super)`, `pub(self)`, or `pub(in path)`, which tie an item to its module position.

```diff
-pub(super) fn parse_header(input: &str) -> Header {
+pub(crate) fn parse_header(input: &str) -> Header {
```

### `require-mod-rs-layout`

Put a module with out-of-line child modules in `name/mod.rs` instead of a sibling `name.rs` file.
Crate roots such as `lib.rs`, `main.rs`, tests, and examples keep their children beside them.

```diff
-src/parser.rs
+src/parser/mod.rs
 src/parser/lexer.rs
```

### `require-sentence-per-line`

Write each sentence of a `///` or `//!` doc comment on its own line, so prose diffs stay small.

```diff
-/// Parses the header. Fails when the magic bytes
-/// are missing.
+/// Parses the header.
+/// Fails when the magic bytes are missing.
```

## Update an installed copy

Ask your agent:

> Use $install-no-slop-rs to update this workspace's scanner while preserving local rules and configuration.

The [update procedure](skills/install-no-slop-rs/references/update.md) merges changes into your copy instead of overwriting it.
