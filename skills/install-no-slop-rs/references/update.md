# Update a customized scanner

Confirm that the application belongs to an existing Cargo workspace before staging or changing an installed scanner.
If it does not, explain that this skill cannot update its xtask and leave the project unchanged, even when an xtask already exists.

Preserve local rules, tests, configuration, diagnostics, and command wiring when adopting upstream changes.

## Compare before merging

1. Find the active scanner through the xtask dispatcher and read its provenance.
   Inspect working-tree changes and keep a recoverable backup outside the installation.
   Leave unrelated work intact.
2. Copy `assets/no_slop/`, `assets/dependencies.toml`, and `assets/upstream.json` into a separate temporary directory for comparison.
   Preserve the installed scanner until you have reviewed the differences.
   For a requested revision or latest version, retrieve that source and verify its identity.
3. Recover the original unmodified source from verified history when available.
   A hash manifest identifies bytes but cannot reconstruct a missing baseline.
   Keep the baseline, local source, and incoming source separate.
4. With a known baseline, merge upstream changes while preserving local edits and deletions.
   Reconcile renamed rules by behavior and registration, not filenames alone.
   Defer conflicting policy until its intent is clear.
5. Without a baseline, port independently understood changes and retain unexplained differences.
   Do not invent an ancestor or replace the directory with upstream defaults.

## Preserve policy and verify

Keep disabled rules, levels, exclusions, and local-only behavior.
New rules need a policy choice unless enabling them is already authorized.
Change dependencies only when adopted code needs them, preserving the target's toolchain and commands.
Keep dependency changes in the xtask manifest, reusing existing compatible workspace entries without adding or changing root dependency declarations.
Declare dependencies absent from `[workspace.dependencies]` directly in xtask and add scanner-specific features there.
If an older copy contains tests in the runtime module, move customized tests into the project's test area and preserve their behavior.
Keep upstream authoring files and dev dependencies out of fresh installations.

Run tests for adopted behavior and local customizations, then verify the installed command and application coverage.
Report application findings separately from scanner failures.
Do not suppress findings or clean up application code merely to make an update pass.

Record incoming identity, baseline identity or its absence, adopted changes, local deviations, deferred changes, and checks in provenance.
Advance the baseline only after reconciling the complete incoming snapshot.
For a partial update, retain the prior baseline and record adopted changes separately.
Report remaining conflicts and the backup location, keeping recovery material until it is no longer needed.

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
