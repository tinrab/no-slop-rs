# Install the runtime and wire the command

Use this procedure after confirming the application belongs to an existing Cargo workspace as described in [SKILL.md](../SKILL.md).

## Copy the runtime

Use the project's established xtask location.
Confirm that the bundle contains `assets/no_slop/mod.rs` and `assets/dependencies.toml`, resolving these paths relative to the skill directory.
If the destination already exists, inspect the dispatcher's imports and use the [update procedure](update.md) rather than overwriting it.

Copy `assets/no_slop/` into `<xtask-directory>/src/no_slop/`, preserving its directory structure.
Use any available copying method, including file tools, shell commands, or Python; no particular copying tool is required.
Compare the copied runtime with the bundle before applying project formatting or local edits.
Do not copy LICENSE files or authoring resources.

## Fit the Cargo layout

| Existing layout                                   | Integration                                         |
| ------------------------------------------------- | --------------------------------------------------- |
| Workspace with a member xtask                     | Keep its package name and membership                |
| Workspace without an xtask                        | Add a non-published binary xtask member             |
| Application package that is also a workspace root | Keep the package layout and add xtask as a member   |
| Independent xtask for a workspace application     | Keep its workspace boundary and manifest-path alias |

Keep explicit `default-members` entries unchanged.
For a virtual workspace without `default-members`, record its existing default members before adding xtask and set `default-members` to those package paths so ordinary Cargo commands do not start selecting xtask.
A workspace root that is also an application package already defaults to that root package, so adding xtask does not require changing its defaults.

Use an edition supported by the project's toolchain and format the runtime with that edition and the project's rustfmt settings.
Merge the requirements in `assets/dependencies.toml` into the xtask manifest instead of copying this repository's manifest.
Preserve existing dependencies and features.
Inherit a dependency only when a compatible entry already exists in the xtask's containing `[workspace.dependencies]` table.
Otherwise, declare its version and required features directly in xtask.
Do not add scanner dependencies to the application root or change shared versions and features.

For example, when `walkdir` is already shared and `syn` is absent, the xtask entries can be:

```toml
[dependencies]
walkdir.workspace = true
syn = { version = "^3.0.9", features = ["full", "visit"] }
```

Use the actual bundled requirements rather than treating example versions as a separate source of truth.
Keep scanner-specific features on xtask entries, including when inheriting a dependency.
Account for required features and default-feature settings when merging, and verify that xtask builds without relying on application crates to enable required features through unification.
Resolve the relevant lockfile through the project's normal Cargo workflow before running the scanner, which discovers workspaces using locked Cargo metadata.
An independent xtask can have a separate lockfile from the application workspace.
If existing dependencies or the toolchain are incompatible, report the concrete blocker rather than silently upgrading them.

## Preserve command behavior

For a new binary dispatcher, use:

```rust
mod no_slop;

use std::{env, process::ExitCode};

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    match args.next().as_deref().and_then(|arg| arg.to_str()) {
        Some("no-slop") => no_slop::run(&args.collect::<Vec<_>>()),
        _ => ExitCode::from(2),
    }
}
```

For an existing dispatcher, add the command without changing other commands or their arguments and exit behavior.
The scanner exposes `run(&[std::ffi::OsString]) -> std::process::ExitCode` and prints its own diagnostics.
Pass all arguments after `no-slop` through as `OsString` values so non-UTF-8 paths remain usable.
Propagate its exit status through the top-level entry point; invoking it and returning `Ok(())` would hide failures in a `Result<()>` dispatcher.
A library-based xtask can expose `pub mod no_slop` and call it from the binary.

Keep an existing suitable Cargo alias.
Otherwise, add `xtask = "run --package <xtask-package> --"` under `[alias]` in `.cargo/config.toml`.
An independent xtask usually uses `run --manifest-path <xtask-directory>/Cargo.toml --`.

The Cargo alias selects the executable; the scanner discovers the application workspace from its working directory.
The scanner's `--manifest-path <application-manifest>` selects the application workspace, while Cargo's `--manifest-path` selects the executable crate.
Preserve explicit application-manifest overrides if the dispatcher injects a default.

## Verify the installation

For a member xtask, run:

```sh
cargo check -p <xtask-package> --all-targets
cargo test -p <xtask-package>
```

For an independent xtask, run:

```sh
cargo check --manifest-path <xtask-directory>/Cargo.toml --all-targets
cargo test --manifest-path <xtask-directory>/Cargo.toml
```

Use `--locked` after resolving lockfiles when consistent with the project's workflow.
Check formatting for the new wiring and copied runtime without reformatting unrelated application files.
Exercise an existing command when available, then run these read-only checks:

```sh
cargo xtask no-slop --list-rules
cargo xtask no-slop --format json
```

Inspect the report's paths and failures to verify application coverage and status propagation.
Check invocation from the workspace root and a member directory.
If an existing relative manifest-path alias fails from a member directory, report the alias limitation and verify scanner behavior with absolute manifest paths:

```sh
cargo run --manifest-path <absolute-xtask-manifest> -- no-slop --manifest-path <absolute-application-manifest> --format json
```

See [configuration and results](configuration.md) for exit codes and expected findings.
