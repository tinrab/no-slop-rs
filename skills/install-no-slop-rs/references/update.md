# Update a customized scanner

Confirm that the application belongs to an existing Cargo workspace before changing an installed scanner.
If it does not, explain that this skill cannot update its xtask and leave the project unchanged.

Find the active scanner through the xtask dispatcher and inspect local changes before editing it.
Compare the installed files with the bundled runtime, using a temporary copy when helpful.
Apply the requested changes while preserving customized rules, tests, diagnostics, configuration, and command wiring.
Do not replace the installed directory wholesale or discard differences you do not understand.
Defer unresolved policy conflicts and explain them to the user.

Keep dependency changes in the xtask manifest, reusing existing compatible workspace dependencies without changing root declarations.
Declare missing dependencies and scanner-specific features directly in xtask.
Preserve disabled rules, levels, and exclusions, and enable new rules only when authorized.

Run tests for adopted behavior and local customizations, then verify the installed command and application coverage.
Report changes, checks, application findings, and unresolved issues.
Do not clean up application code merely to make the update pass.

## Rule ownership migration

This bundle removes checks already provided by Clippy.
Review each local implementation before adopting its deletion.

| Removed ID                   | Clippy equivalent                 |
| ---------------------------- | --------------------------------- |
| `no-placeholder-macros`      | `todo`, `unimplemented`           |
| `no-production-unwrap`       | `unwrap_used`                     |
| `no-boolean-parameter-lists` | `fn_params_excessive_bools`       |
| `require-lint-reason`        | `allow_attributes_without_reason` |
| `no-inline-qualified-paths`  | `absolute_paths`                  |

Rename `no-string-or-unit-errors` to `no-string-errors` for retained string-contract policy, preserving its level.
Unit errors now belong to Clippy's `result_unit_err`.
Remove obsolete keys and suppression directives only when adopting the corresponding removal.
Unknown IDs continue to fail validation.
Do not discard customized behavior or enable Clippy during migration.

`no-silent-error-discard` leaves bare unused `.ok();` statements to Clippy's `unused_result_ok`.
It retains the policy for consumed conversions and filters that discard errors.
Apply the existing new-rule policy to `no-decorative-comments`, `no-boilerplate-comments`, `no-vague-todos`, and `no-stringified-error-mapping`.
