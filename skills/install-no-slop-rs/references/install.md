# Install the runtime and wire the command

Confirm that the application belongs to an existing Cargo workspace before copying files or changing Cargo configuration.
For a standalone application outside a workspace, explain that the skill cannot create or update an xtask and leave the project unchanged.

## Copy into a new destination

Use the project's established xtask location.
Before changing the project, confirm that the bundle contains `assets/no_slop/mod.rs`, `assets/no_slop/LICENSE`, `assets/dependencies.toml`, and `assets/upstream.json`.
If the destination already exists as a file, directory, or symlink, follow the [update procedure](update.md) instead of copying over it.

1. Copy the contents of `assets/no_slop/` to `<xtask-directory>/src/no_slop/`, preserving the directory structure and MIT license.
2. Copy `assets/upstream.json` to `<xtask-directory>/src/no_slop/UPSTREAM.json` without changing its contents.
3. Create `UPSTREAM.md` beside it to record the bundle source, installed paths, local changes, and verification.
   Leave the source revision unknown unless you can verify it.
4. Compare the copied runtime files with the bundle before applying project formatting or local edits.

Copy only the runtime and provenance files, not the whole skill, upstream xtask crate, or authoring tests.
Merge manifests, command wiring, and aliases as described below.

## Fit the Cargo layout

| Project layout                                              | Integration                                                          |
| ----------------------------------------------------------- | -------------------------------------------------------------------- |
| Workspace with a member xtask                               | Keep its package name and membership                                 |
| Workspace without an xtask                                  | Add a non-published xtask binary member and preserve default-members |
| Application package that is also an existing workspace root | Keep the package layout and add xtask to its workspace members       |
| Independent xtask for an existing application workspace     | Keep its workspace boundary and manifest-path alias                  |

Use the project's edition or another edition supported by its toolchain.
Merge `assets/dependencies.toml` into the xtask manifest, retaining compatible dependencies and existing features.
Add dependency declarations only to the xtask's `Cargo.toml`, usually `xtask/Cargo.toml`.
If a compatible dependency already exists in the containing workspace's `[workspace.dependencies]`, reference it with `.workspace = true`.
Otherwise, declare its version and features directly in xtask.
Do not add scanner dependencies to the root manifest or change shared dependency versions and features to satisfy the scanner.
For an independent xtask workspace, inherit only dependencies already declared in its own workspace, not the application's workspace.

For example, if `walkdir` is already shared but `globset` is not, use:

```toml
[dependencies]
walkdir.workspace = true
globset = "^0.4.20"
```

Keep scanner-specific features on xtask entries, such as `syn = { workspace = true, features = ["full", "visit"] }`.
Add required features such as syn's `visit` without removing features the project already uses.
The generated requirements contain normal runtime dependencies only.
Resolve the target's lockfile through its usual Cargo workflow.
Check older toolchains with a build instead of silently upgrading them.

## Preserve command behavior

For a new binary dispatcher, use:

```rust
mod no_slop;

use std::env;
use std::process::ExitCode;

fn main() -> ExitCode {
	let mut args = env::args_os().skip(1);
	match args.next().as_deref().and_then(|arg| arg.to_str()) {
		Some("no-slop") => no_slop::run(&args.collect::<Vec<_>>()),
		_ => ExitCode::from(2),
	}
}
```

For an existing dispatcher, add the command while preserving other commands and their behavior.
The scanner's `run(&[std::ffi::OsString]) -> std::process::ExitCode` prints diagnostics and returns the scan status.
If the dispatcher returns `Result<()>`, carry that status through the top-level entry point.
Calling `run` and returning `Ok(())` would hide lint failures.
A library-based xtask can expose `pub mod no_slop`.

Keep an existing suitable Cargo alias.
Otherwise, add `xtask = "run --package <xtask-package> --"` under `[alias]` in `.cargo/config.toml`.
An independent xtask usually uses `run --manifest-path <xtask-directory>/Cargo.toml --`.

The alias selects the executable, while the scanner discovers the application workspace from its working directory.
Use the scanner's `--manifest-path <application-manifest>` when an explicit workspace is needed.
If the dispatcher supplies a default application manifest, preserve explicit overrides.

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

Use `--locked` after resolving the lockfile when consistent with the project's workflow.
Format new wiring and copied runtime with the project's edition and rustfmt settings.
Import ordering can differ between editions, so record formatting changes instead of changing the target's edition to match upstream.
Verify normal and test builds without upstream tests or authoring files.

Exercise an existing command, then run:

```sh
cargo xtask no-slop --list-rules
cargo xtask no-slop --format json
```

Inspect paths and failures to confirm the application workspace was scanned.
Verify application coverage from the workspace root and a member directory.
A relative manifest-path alias can fail from a member directory before xtask starts.
Preserve that alias unless changing it is authorized, and verify coverage with an explicit invocation:

```sh
cargo run --manifest-path <actual-xtask-manifest> -- no-slop --manifest-path <application-manifest> --format json
```

Report any existing alias limitation separately from scanner behavior.
See [configuration and results](configuration.md) for policy and exit codes.

Record the source identity, installed paths, manifest changes, policy, local changes, and checks in `UPSTREAM.md`.
Keep `UPSTREAM.json` unchanged as the incoming content manifest.
Leave unverifiable source revisions unknown.
