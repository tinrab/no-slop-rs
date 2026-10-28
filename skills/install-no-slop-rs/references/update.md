# Update a customized scanner

Use this procedure after confirming the application belongs to an existing Cargo workspace as described in [SKILL.md](../SKILL.md).

## Merge runtime and policy changes

Find the active scanner through the dispatcher and compare it with the bundled runtime.
Record its existing rule IDs, effective levels, exclusions, custom diagnostics, and local tests before merging.
An absent rule-level entry currently means `error`; preserve effective behavior as well as explicit configuration.

Merge compatible changes into the installed copy, retaining local-only rules, tests, diagnostics, command wiring, and unrelated edits.
Do not replace the directory wholesale or discard differences you do not understand.
Add bundled helper modules required by adopted changes and retain modules still used by local code.
Follow the user's requested scope and report unresolved conflicts instead of overwriting custom behavior.

For an update that preserves existing policy, configure newly introduced rule IDs as `off` unless their adoption is part of the request or established project policy.
Preserve the effective levels of retained rules and report which new policies were adopted or left disabled.
If an existing rule's patterns or fixes change, compare those changes with local customizations rather than treating a matching ID as proof of compatibility.

Merge current bundled dependency requirements only into the xtask manifest, preserving existing compatible dependencies and features.
Use the [installation guidance](install.md#fit-the-cargo-layout) for workspace inheritance, features, lockfiles, and toolchain compatibility.
Do not copy LICENSE files or authoring resources into the project.

## Migrate older rule IDs when present

Apply these migrations only when the installed scanner contains the old IDs and the replacement preserves its intended policy.
Local implementations can contain custom behavior beyond their original shipped versions.

| Removed ID                   | Clippy equivalent                 |
| ---------------------------- | --------------------------------- |
| `no-placeholder-macros`      | `todo`, `unimplemented`           |
| `no-production-unwrap`       | `unwrap_used`                     |
| `no-boolean-parameter-lists` | `fn_params_excessive_bools`       |
| `require-lint-reason`        | `allow_attributes_without_reason` |
| `no-inline-qualified-paths`  | `absolute_paths`                  |

Rename `no-string-or-unit-errors` to `no-string-errors` when adopting the retained string-contract rule, preserving its level.
Unit errors belong to Clippy's `result_unit_err`.
The retired `no-silent-error-discard` policy has no replacement in this bundle.
Remove its configuration entries and exception directives when adopting its removal.

Remove obsolete configuration keys and suppression directives only when adopting the corresponding rule removal or rename.
If a local implementation is retained, keep its registration and configuration valid.
Unknown IDs fail validation, so do not leave references to deleted rules.
Do not enable or configure Clippy as a side effect of this migration.

## Verify the merged copy

Use the [installation checks](install.md#verify-the-installation) with the active xtask manifest and application workspace.
Run existing tests for local customizations and adopted behavior.
Run the scanner without fix flags unless cleanup was requested.
Report remaining application findings separately from failures and identify adopted rules, preserved customizations, and unresolved conflicts.
