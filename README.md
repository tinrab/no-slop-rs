# no-slop-rs

Editable Rust lints for common patterns coding agents leave behind.

The rules live in your repository and run through `cargo xtask no-slop`.
They cover policies beyond Clippy, including test names, declaration spacing, comment noise, vague type names, and discarded error information.

## Install the skill

From your target Cargo workspace, install the skill from a local checkout of this repository:

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
cargo xtask no-slop --list-rules
cargo xtask no-slop --format json
```

The scanner reads source without building application crates or expanding macros.
All rules default to errors.
Edit the copied rules or [configure levels, exclusions, and local exceptions](skills/install-no-slop-rs/references/configuration.md) to fit your project.
Exit codes are 0 for success, 1 for error findings, and 2 for invalid input or incomplete coverage.

## Update an installed copy

Ask your agent:

> Use $install-no-slop-rs to update this workspace's scanner while preserving local rules and configuration.

The [update procedure](skills/install-no-slop-rs/references/update.md) merges changes into your copy instead of overwriting it.
