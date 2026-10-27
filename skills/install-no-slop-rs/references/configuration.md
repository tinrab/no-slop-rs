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
Read the installed rule modules for exact patterns and exceptions.

## Read scan results

JSON contains `diagnostics`, `failures`, and `summary`.
Locations include one-based lines, Unicode columns, and UTF-8 byte offsets.

| Exit | Meaning                                                         |
| ---- | --------------------------------------------------------------- |
| 0    | Complete scan with no error findings, though warnings may exist |
| 1    | Complete scan with error findings                               |
| 2    | Invalid input or incomplete discovery, parsing, or I/O coverage |

Application findings alone do not mean installation failed.
Report coverage failures separately and fix application code only when cleanup was requested.
For obsolete IDs, follow the [update migration](update.md#rule-ownership-migration) before changing local configuration.
