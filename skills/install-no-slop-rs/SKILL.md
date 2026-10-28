---
name: install-no-slop-rs
description: >-
    Install, configure, or update editable Rust rules behind cargo xtask no-slop in an existing Cargo workspace, integrating with an existing xtask or creating a missing one while preserving project customizations.
license: MIT
---

# Install or update Rust no-slop

The project owns its scanner source and policy.
The scanner checks written Rust syntax without building application crates, expanding macros, or inferring types.
It covers policies beyond Clippy and does not invoke or configure Clippy.
Cargo and a stable Rust toolchain compatible with the bundled dependencies are required.

## Check the application workspace

Read the project's agent instructions and working-tree state.
Run `cargo locate-project --workspace --message-format plain` from the application directory and inspect the returned manifest for `[workspace]` before editing files.
A member is supported even when its own manifest has no `[workspace]` table.
Cargo also reports a workspace root for standalone packages, so metadata alone does not establish eligibility.
An independent xtask's workspace does not make a standalone application eligible.

If the application has no declared Cargo workspace, explain that this skill cannot create or update its xtask and leave the project unchanged.
Do not create a workspace or an independent xtask to bypass this requirement.

## Choose the workflow

Locate the application workspace, Cargo alias, xtask package, dispatcher, and any installed scanner.
Follow the dispatcher's imports to find the active scanner, including renamed modules.

- For a fresh installation, read [installation and command wiring](references/install.md).
- For an existing scanner, read [updating a customized copy](references/update.md).
- For policy changes or interpreting findings, read [configuration and results](references/configuration.md).

## Preserve project intent

Keep existing commands, aliases, default package selection, toolchain, formatting settings, and lint policy.
Copy only runtime source from `assets/no_slop/` and merge the normal dependency requirements from `assets/dependencies.toml`.
Do not copy LICENSE files, the whole skill, this repository's xtask crate, tests, or authoring tools into the project.

Declare scanner dependencies only in the target xtask's `Cargo.toml`.
Inherit compatible entries already present in its containing workspace with `.workspace = true`.
Declare missing dependencies and scanner-specific features directly in xtask; do not add or change root workspace dependency declarations.
For an independent xtask, its containing workspace is separate from the application's workspace.

Apply compatible changes within the request and preserve local customizations when merging updates.
Report dependency or toolchain blockers instead of silently upgrading the project or changing unrelated configuration.
Do not disable existing rules, exclude handwritten code, or add suppressions merely to make verification pass.
Change application code only when cleanup is part of the request.

## Verify and report

Build and test the xtask through its actual manifest, check formatting, and exercise an existing command when one is available.
Run the scanner without fix flags during installation or update verification unless application cleanup was requested.
Inspect paths, findings, and failures to confirm that it scanned the application workspace.
Application findings can be expected on a successful installation; distinguish them from coverage or execution failures.

Report the command, active module path, adopted policy changes, checks, remaining findings, and unresolved issues.
