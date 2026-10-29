# Configure policy and interpret scans

Use this reference within an existing Cargo workspace, following the eligibility check in [SKILL.md](../SKILL.md).
All shipped rules default to errors on a fresh installation.
For updates, preserve policy as described in the [update procedure](update.md#merge-runtime-and-policy-changes).
The installed registry and rule source are authoritative when the project has customizations.

## Configure the application workspace

Put policy in the application workspace root's manifest, including when xtask has its own workspace:

```toml
[workspace.metadata.no-slop]
exclude = ["crates/bindings/src/generated/**"]

[workspace.metadata.no-slop.rules]
no-vague-type-names = "warn"
```

Levels are `error`, `warn`, and `off`; omitted levels default to `error`.
Unknown IDs, fields, levels, and invalid globs fail validation.
Run `cargo xtask no-slop --list-rules` to inspect the installed rule IDs and fix support.
Keep scanner dependency declarations in xtask; workspace metadata configures policy, not dependencies.

Exclusion patterns match workspace-relative paths, or absolute paths for source outside the workspace.
Exclude generated bindings and deliberately invalid parser fixtures where appropriate, keeping handwritten application and tooling code in coverage.
Leading `@generated` comments and bindgen headers skip generated files.
Do not add generated-file markers to handwritten files to silence findings.

## Explain local exceptions

Name one rule and give a concrete reason:

```rust
// no-slop-allow: require-lowercase-error-messages -- preserve the protocol's exact wording
let error = std::io::Error::other("Connection rejected");
```

The directive covers intervening leading comments and the next statement, item, or enum variant, including its attributes and nested code.
It does not cover preceding comments or sibling declarations.
Stack directives for multiple rules.
Unknown IDs, empty reasons, and directives without a following supported target fail the scan.
Strings, doc comments, block comments, and opaque macro bodies cannot introduce directives.

## Understand coverage and customization

The default scan includes every application workspace member, including a member xtask, regardless of Cargo's `default-members`.
Use repeated `-p <package>` options to select members and the scanner's `--manifest-path <application-manifest>` to select the application workspace.
An independent xtask is a separate workspace and is not automatically an application member.

The scanner uses locked Cargo metadata and enumerates workspace-owned Rust source, including orphan files and inactive cfg branches.
It skips generated files, vendor directories, build output, common agent directories, and its bundled skill assets, and does not follow source symlinks.
It does not compile application targets, expand macros, resolve imported aliases, or infer types.
Selected conventional macro arguments are parsed by error-message rules, while unrelated macro payloads remain opaque.
Same-named custom APIs and types can need local exceptions because matching follows written conventions.

Edit the installed registry to add or remove rules and edit individual rule modules to change dictionaries or recognized syntax.
Keep new checks generic and syntax-based, document their limits, and test accepted and rejected forms using the project's test layout.
Use Clippy for behavior already covered by its lints rather than duplicating those checks.

The shipped policies have these important boundaries:

| Policy                                                        | Coverage limits                                                                                                                                                                          |
| ------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `require-test-prefix`, `no-vague-test-names`                  | Only test-attributed functions, including qualified async test attributes and nested `cfg_attr`; helpers in test modules are exempt                                                      |
| `require-readable-spacing`                                    | Sibling declarations; imports, external modules, and single-line constants can form compact groups                                                                                       |
| `require-grouped-imports`                                     | Consecutive use runs in files, inline modules, and blocks; attributes, comments attached to imports, local exceptions, and empty groups are boundaries                                   |
| `no-string-errors`                                            | Explicit `Result` string-error contracts and type aliases; recognized test-only contracts are exempt                                                                                     |
| `no-blanket-lint-suppression`                                 | `warnings` and `clippy::all` in `allow` or `expect`, including nested `cfg_attr`                                                                                                         |
| `no-vague-type-names`                                         | Written type names using generic `Manager`, `Helper`, `Processor`, or `Data` naming patterns                                                                                             |
| Comment rules                                                 | Ordinary line comments; rustdoc, block comments, strings, and opaque macro payloads are excluded                                                                                         |
| `no-stringified-error-mapping`                                | Direct untyped synchronous `map_err` closures returning the bound error's `.to_string()`                                                                                                 |
| `no-restricted-visibility`                                    | Written `pub(super)`, `pub(self)`, and `pub(in path)` restrictions, including fields and associated items; macro-generated visibility is excluded                                        |
| `require-mod-rs-layout`                                       | Non-root `name.rs` files declaring out-of-line child modules; target roots, `mod.rs` files, inline modules, and `#[path]` children are exempt                                            |
| `require-sentence-per-line`                                   | `///` and `//!` prose paragraphs and list items; code blocks, tables, headings, quotes, HTML, link definitions, hard breaks, inline code, links, and common abbreviations are left alone; lines are aligned to the documented item and trailing empty lines dropped |
| `require-lowercase-error-messages`, `no-vague-error-messages` | Literal messages at recognized thiserror, anyhow/eyre, context, `Err`, `.ok_or`, and I/O error sites; dynamic messages and handwritten `Display` are excluded                            |
| `no-duplicate-error-context`                                  | Identical decoded literals on adjacent context calls, including literal-only lazy closures; formatting macros and interrupted chains are excluded                                        |
| `require-create-for-fallible-constructors`                    | Receiver-free associated `new` functions explicitly returning outer `Result` or `Option`; trait implementations, differently named aliases, and nested wrappers are excluded             |
| `require-error-sources`                                       | Explicit `Error` or `thiserror::Error` derives with direct recognized standard-error fields; aliases, custom types, containers, and conditional `cfg_attr` forms are excluded            |

Import grouping separates standard-library, external, and local roots without requiring alphabetical order, vertical formatting, or moving imports to the top of a scope.
Plain imports and explicit `self` imports remain distinct, so some shared prefixes must stay repeated to preserve namespace behavior.

Lowercase checking examines the initial static text before any formatting placeholder and preserves capitalization later in the message.
Vague-message, vague-test-name, and comment checks use narrow dictionaries rather than judgments about all prose or identifiers.
Read the installed rule modules for exact dictionaries, paths, and supported expression wrappers.

The source-chain rule recognizes `io::Error`, `ParseIntError`, `ParseFloatError`, `Utf8Error`, and `FromUtf8Error` in their supported standard spellings.
It accepts `#[source]`, `#[from]`, a field named `source`, or `#[error(transparent)]`, and reports at most one finding per wrapper.
Place an intentional source-chain exception above the struct or enum variant.

## Read results

JSON contains `diagnostics`, `failures`, and `summary`.
Locations include one-based lines, Unicode columns, and UTF-8 byte offsets.
A fixable finding includes a `fix` with byte offsets `start` and `end` and a `replacement` string.

| Exit | Meaning                                                         |
| ---- | --------------------------------------------------------------- |
| 0    | Complete scan without error findings; warnings can remain       |
| 1    | Complete scan with error findings                               |
| 2    | Invalid input or incomplete discovery, parsing, or I/O coverage |

Application findings alone do not mean installation failed.
Investigate coverage failures separately, and change application code only when cleanup was requested.
For obsolete IDs, follow the [older-rule migration](update.md#migrate-older-rule-ids-when-present).

## Preview and apply fixes

```sh
cargo xtask no-slop --fix-dry-run
cargo xtask no-slop --fix
```

Only run these modes when application cleanup is part of the request.
Both honor package selection, exclusions, configured levels, and local exceptions.
Warnings are eligible for fixes; disabled and suppressed findings are not.

The shipped fixes cover import grouping, declaration spacing, decorative comments, and exact boilerplate labels.
Scaffolding reminders such as `Your code here` stay diagnostic-only because they can indicate unfinished behavior.
Other policies require manual correction, and some findings within a fixable rule can also lack an edit.
Consult the installed registry and diagnostics rather than assuming every finding can be fixed.

Preview runs the same in-memory edit process as application, prints unified diffs without writing source, and reports the original findings and scan status.
Application reports findings remaining after successful edits and can still exit with code 1.
The two modes cannot be combined with each other or `--list-rules`.
When edits exist, JSON includes `changes` with each file's `path`, unified `diff`, and `applied` status, while the summary reports `files_changed` or `files_would_change`.

Candidates must parse and preserve rustdoc comments, BOMs, and shebangs.
Trivia edits preserve nontrivia tokens, and import regrouping must preserve written paths, aliases, wildcards, explicit `self` imports, visibility, and absolute qualifiers within each lexical import run.
Other executable tokens must remain identical.
The fixer preserves line endings and does not run rustfmt or require a clean Git working tree.
Run the project's formatting, build, and test checks after applying edits; syntax validation alone does not establish that the application compiles.

Invalid local exceptions, unsafe candidates, stale source snapshots, and write failures leave the affected file unchanged and produce exit code 2.
Other files can still have been fixed, so inspect the reported changes as well as failures.
