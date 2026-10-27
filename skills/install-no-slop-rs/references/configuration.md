# Configure policy and interpret scans

Apply configuration changes only to an application that belongs to an existing Cargo workspace.
Leave standalone applications outside a workspace unchanged and explain the workspace requirement.

All shipped rules default to errors on a fresh installation.
Read the installed registry or run `cargo xtask no-slop --list-rules` for the actual rule IDs.
Preserve existing settings on updates and adopt new rules only when their policy is authorized.

## Configure the workspace

Put configuration in the application workspace root's manifest, including when xtask has its own workspace:

```toml
[workspace.metadata.no-slop]
exclude = ["crates/bindings/src/generated/**"]

[workspace.metadata.no-slop.rules]
no-vague-type-names = "warn"
```

Levels are `error`, `warn`, and `off`.
Unknown rule IDs, fields, levels, and invalid globs fail validation.
Exclusion paths are workspace-relative, except that files outside the workspace use absolute paths.
Keep handwritten wrappers and tooling in coverage.
Leading `@generated` comments and bindgen headers skip generated files.
Do not add those headers to handwritten files to silence findings.

## Explain local exceptions

Name one rule and give a concrete reason:

```rust
// no-slop-allow: no-silent-error-discard -- a missing cache entry is normal
let cached = read_cache().ok();
```

The directive covers intervening leading comments and the next statement or item, including its attributes and nested code.
It does not cover preceding comments or sibling declarations.
Stack directives for multiple rules.
Unknown IDs, empty reasons, and directives without a following statement or item fail the scan.
Strings, doc comments, block comments, and opaque macro bodies cannot introduce directives.

## Understand coverage

The default scan includes every application workspace member, including a member xtask.
Use `-p <package>` to select members and `--manifest-path <application-manifest>` to select the workspace.
Generated files, vendor directories, build output, common agent directories, and bundled skill assets are excluded.
The scanner does not follow symlinks.
Source enumeration includes orphan Rust files, so exclude deliberately invalid parser fixtures.

Inactive cfg branches are inspected without macro expansion or type inference.
Checks follow written method and type names without resolving imported aliases.
The string-error contract rule exempts recognized test scopes, while comment rules also check tests.
Comment rules inspect ordinary line comments and leave rustdoc, block comments, strings, and macro payloads alone.
The error-stringification rule recognizes direct untyped synchronous closures, leaving typed closures, named callbacks, and indirect conversions outside its scope.
The `require-grouped-imports` rule merges consecutive imports with shared written roots and nested prefixes in files, inline modules, and blocks.
It puts `std`, `core`, and `alloc` first, other roots next, and `crate`, `self`, and `super` last, with blank lines between categories.
It preserves first-seen roots within each category and accepts compact groups without requiring alphabetical order or vertical formatting.
Attributes, attached or internal comments, local exception directives, and empty use groups are boundaries that it does not rewrite or cross.
It preserves visibility, aliases, existing wildcards, duplicates, and leading `::`, introduces no wildcards, and does not move imports to the top of a scope.
Root categories follow written spelling without resolving local modules or external crates.
Plain terminal imports and explicit `self` imports remain distinct because they bind different namespaces, so some shared prefixes must remain repeated.
Read the installed rule modules for exact patterns and exceptions.

## Error-message and constructor naming policies

`require-lowercase-error-messages` checks the opening of literal messages at conventional error APIs, including in tests.
It recognizes thiserror-style `#[error("...")]` attributes on structs and enum variants; bare or `anyhow::`, `eyre::`, and `color_eyre::eyre::` error macros named `anyhow`, `eyre`, `bail`, `ensure`, or `format_err`; and `.context`, `.with_context`, `.wrap_err`, and `.wrap_err_with` methods.
It also recognizes `Err`, `Result::Err`, `std::result::Result::Err`, `core::result::Result::Err`, `.ok_or`, `.ok_or_else`, and `new` or `other` calls on `Error`, `io::Error`, or `std::io::Error`.
Supported messages are normal or raw string literals, literal `format!` templates including `std::format!` and `alloc::format!`, and argument-free `.to_owned()`, `.to_string()`, or `.into()` wrappers around those expressions.
Lazy methods require a zero-argument synchronous closure whose body reduces to a supported expression through parentheses or single-expression blocks.
The first cased Unicode character in the initial static text must be lowercase; formatted messages stop at the first interpolation field, and escaped braces remain literal text.
Capitalization later in a message and trailing punctuation are unrestricted, while required uppercase openings such as protocol names need a local exception.
Constants, dynamic expressions, imported aliases, UFCS context calls, `#[error(transparent)]`, nested `cfg_attr` attributes, handwritten `Display`, logging, and panic messages are outside its coverage.
Names are matched by spelling without resolving their definitions, so same-named custom APIs can need exceptions.

`require-create-for-fallible-constructors` requires receiver-free associated functions named exactly `new` to use `create` or a descriptive `create_*` name when their explicit outer return type ends in `Result` or `Option`.
It checks inherent implementations and trait declarations, including private, asynchronous, and test code, and accepts qualified paths and parenthesized return types.
Trait implementations are exempt because their names are constrained by the trait contract.
Free functions, receiver methods, infallible constructors, `new_*`, and `try_new` names are unaffected.
Differently named aliases and fallibility hidden inside containers or explicit future types are not resolved; same-named custom `Result` and `Option` types follow the written-name convention.

Both policies are diagnostic-only because changing message text can affect output contracts and renaming constructors requires updating references.

## Read scan results

JSON contains `diagnostics`, `failures`, and `summary`.
Locations include one-based lines, Unicode columns, and UTF-8 byte offsets.
Fixable diagnostics also include a `fix` with UTF-8 byte offsets `start` and `end` and a `replacement` string.

| Exit | Meaning                                                         |
| ---- | --------------------------------------------------------------- |
| 0    | Complete scan with no error findings, though warnings may exist |
| 1    | Complete scan with error findings                               |
| 2    | Invalid input or incomplete discovery, parsing, or I/O coverage |

Application findings alone do not mean installation failed.
Report coverage failures separately and fix application code only when cleanup was requested.
For obsolete IDs, follow the [update migration](update.md#rule-ownership-migration) before changing local configuration.

## Preview and apply mechanical fixes

```sh
cargo xtask no-slop --fix-dry-run
cargo xtask no-slop --fix
```

Fixes are available for `require-grouped-imports`, `require-readable-spacing`, `no-decorative-comments`, and `no-boilerplate-comments`.
They group shared import prefixes, insert blank lines, remove decorative separators and exact boilerplate, and preserve the text inside framed headings.
Other rules remain diagnostic-only because their corrections require judgment or reference resolution.
Some findings within a fixable rule can also remain without an automatic fix.
Use `--list-rules` to see which rules support fixes.

Preview runs the same in-memory edit process as application and prints unified diffs without writing files.
It reports the original findings and uses the normal scan exit status.
Application reports findings remaining after successful edits, so it can still exit with code 1.
The two modes cannot be combined with each other or with `--list-rules`.

Both modes honor package selection, exclusions, levels, and local exceptions.
Warnings are eligible for fixes, while disabled and suppressed findings are not.
Ordinary scans remain read-only, and installation verification must not fix application code unless cleanup was requested.

Candidates must parse and preserve rustdoc comments, BOMs, and shebangs.
Import regrouping must preserve the multiset of written paths, aliases, wildcards, explicit `self` imports, visibility, and absolute qualifiers in each lexical import run.
All tokens outside those runs must remain identical.
Edits preserve line endings and do not run rustfmt or require a clean Git working tree.
Syntax validation does not establish that the application compiles or that its tests pass.
Run the project's normal checks after applying fixes.

Files with invalid local exceptions, unsafe edit candidates, or write failures remain unchanged while other files can still be fixed.
Files changed since scanning are rejected before replacement.
Such failures produce exit code 2, and the report identifies successful changes separately.

Fix-mode JSON adds `changes` containing per-file `path`, unified `diff`, and `applied` status when changes exist.
The summary includes `files_changed` for application and `files_would_change` for preview.
