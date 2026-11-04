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
Edit the copied rules or [configure levels, exclusions, and local exceptions](skills/install-no-slop-rs/references/configuration.md) to fit your project.
Exit codes are 0 for success, 1 for error findings, and 2 for invalid input or incomplete coverage.

## Rules

Each diff replaces a violation with an accepted alternative.
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

Begin error-message prose with lowercase text while preserving acronyms and identifiers later in the message, and end it without punctuation.
This covers `.expect()` messages too, and `--fix` removes trailing punctuation.

```diff
-bail!("Failed to read HTTP response.");
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

Use meaningful error types instead of `String`, `&str`, or `Box<dyn Error>` in `Result` error contracts outside test scopes.
`Box<dyn Error>` stays allowed on `main` and in trait implementations, whose signatures are fixed.

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
The fix also aligns stray doc lines with the documented item and drops trailing empty doc lines, which rustfmt leaves alone inside macro bodies.

```diff
-/// Parses the header. Fails when the magic bytes
-/// are missing.
+/// Parses the header.
+/// Fails when the magic bytes are missing.
```

### `no-source-in-error-display`

Keep a thiserror source out of its message, because reporters that walk the error chain would print the cause twice.
Message wording is not judged, and details taken from the source, such as `source.kind()`, are accepted.
If your project only prints `{error}`, removing the placeholder hides the cause, so report the whole chain first or explain the message with a local exception.

```diff
 #[derive(Debug, thiserror::Error)]
 enum ConfigError {
-    #[error("could not read configuration: {0}")]
+    #[error("could not read configuration")]
     Read(#[from] io::Error),
 }
```

### `no-log-and-return`

Log a failure where it is handled, instead of logging it and then returning it for the caller to report again.

```diff
 if let Err(error) = result {
-    tracing::error!("could not read configuration: {error}");
     return Err(error);
 }
```

### `no-eager-error-context`

Build a context message only on failure instead of formatting it on every call.
Error values such as `error.context(format!(...))` are exempt because they have no lazy form.

```diff
-read_config(path).context(format!("reading {}", path.display()))
+read_config(path).with_context(|| format!("reading {}", path.display()))
```

### `no-vague-module-names`

Name a module for a responsibility instead of `utils`, `util`, `helpers`, `helper`, `common`, or `misc`.
Test scopes and `tests/`, `examples/`, and `benches/` directories are exempt, because a shared `common` module is how those separate crates reuse code.

```diff
-mod utils;
+mod retry;
```

### `no-dense-statement-runs`

Break a run of six or more sibling statements with blank lines where its purpose changes.
A change of purpose is not visible in syntax, so this only reports the run: one blank line anywhere satisfies it, and choosing the right place is up to you.
Test scopes are exempt.

```diff
 let id = generate_id();
+
 let user = fetch_user(&id)?;
 let profile = load_profile(&user)?;
 let settings = load_settings(&user)?;
 let theme = settings.theme();
+
 update_cache(id, &profile);
 notify(&user);
 record_login(&user);
```

### `require-blank-after-block`

Separate a multi-line `if`, `match`, loop, or block from the next statement, so the end of a nested section is easy to see.
A final return value may follow directly, and values such as `let x = if ...` are not checked.
Test scopes are exempt.

```diff
 for item in items {
     subtotal += item.price;
 }
+
 let discount = discount_for(subtotal);
 subtotal - discount
```

## Update an installed copy

Ask your agent:

> Use $install-no-slop-rs to update this workspace's scanner while preserving local rules and configuration.

The [update procedure](skills/install-no-slop-rs/references/update.md) merges changes into your copy instead of overwriting it, and records provenance in `UPSTREAM.md` for future updates.
