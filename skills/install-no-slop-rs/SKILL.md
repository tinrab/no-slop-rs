---
name: install-no-slop-rs
description: >-
    Install, configure, or update editable Rust no-slop rules behind cargo xtask no-slop in an existing Cargo workspace.
    Use when adding workspace hygiene checks, integrating an existing xtask, creating a missing workspace xtask, or updating a vendored scanner while preserving local customizations.
license: MIT
compatibility: >-
    Requires an existing Cargo workspace, Cargo, and stable Rust.
---

# Install or update Rust no-slop

The project owns its scanner source and policy.
The scanner inspects written Rust syntax without building application crates, expanding macros, or inferring types.
It covers policy gaps in Clippy and does not invoke or configure Clippy.

## Choose the workflow

Read the target's agent instructions and working-tree state.
Before changing files, confirm that the application belongs to an existing Cargo workspace whose root manifest declares `[workspace]`.
Run `cargo locate-project --workspace --message-format plain` from the application directory and inspect the returned manifest.
A workspace member is supported even when that member's own manifest has no `[workspace]` table.
Cargo metadata also reports a workspace root for standalone packages, so that field alone does not establish eligibility.
An independent xtask's workspace does not make a standalone application eligible.

If the application is not part of an existing workspace, explain that this skill cannot create or update its xtask and stop without changing files.
Do not convert the application into a workspace or create a separate xtask workspace to bypass this requirement.

Locate the application workspace, Cargo alias, xtask package, dispatcher, and any existing scanner.
Follow the command's imports to find the active copy, including renamed modules.

- For a fresh installation, follow [installation and command wiring](references/install.md).
- For an existing scanner, follow [updating a customized copy](references/update.md).
- For configuration changes, follow [policy and scan results](references/configuration.md).

## Preserve project intent

Keep existing commands, aliases, workspace defaults, toolchain, formatting settings, and lint policy.
Copy runtime source and normal dependencies, leaving this repository's tests and authoring tools out of the target.
Use the generated dependency requirements instead of copying this repository's xtask manifest.
Declare scanner dependencies only in the xtask's `Cargo.toml`.
Use `.workspace = true` for compatible dependencies already declared in that workspace's `[workspace.dependencies]` table.
Declare other dependencies directly in xtask rather than adding them to the root manifest.
Keep scanner-specific features on the xtask dependency entries instead of changing shared workspace features.

Report application findings and change application code only when cleanup was requested.
Do not weaken rules to make verification pass.
Apply compatible updates already authorized by the request and defer unresolved policy conflicts.

## Verify and report

Build and test through the xtask's actual manifest, check formatting, and exercise an existing command.
Run the scanner against the application workspace and inspect both its paths and exit status.
Keep installation verification read-only unless fixing application code is requested.
Installation examples and exit-code definitions are in the references.

Report the installed command, module path, scanned workspace, checks, findings, and unresolved issues.
Separate application findings from failures that leave coverage incomplete.
