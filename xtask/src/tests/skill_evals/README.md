# Maintainer evaluation cases

These prompts test agent behavior during installation and updates, alongside the Rust integration tests.
Keep them outside the shipped skill and run them in temporary projects.
Installation itself requires only Rust and Cargo.

## Prepare fixtures

Create identical initial copies for the old and revised skill.
Use these project layouts:

1. **Unsupported standalone crate:** create an edition-2021 package named `release-app` with no workspace declaration or xtask.
   Put `pub struct UserManager;` in `src/lib.rs` and add a `check-app = "check --all-targets"` Cargo alias.
   Expect an explanation of the workspace requirement and no file changes.
2. **Independent xtask:** create a workspace with `members = ["app"]`, `default-members = ["app"]`, and `exclude = ["tools/xtask"]`.
   Put `pub struct UserManager;` in the app.
   Give `tools/xtask` its own workspace, edition 2021, and a `Result<()>` dispatcher whose `ping` command prints `pong`.
   Set the root alias to `run --manifest-path tools/xtask/Cargo.toml --`.
3. **Customized update:** create a workspace with app and xtask members and an installed scanner.
   Change the vague-name diagnostic to include `needs a project-specific name`.
   Set `no-vague-type-names = "off"` and `no-boilerplate-comments = "warn"`.
   Retain a `ping` command and put `pub const LOCAL_SETTING: u8 = 37;` in the app.
   Stage the incoming bundle separately.
4. **Workspace without xtask:** create a virtual workspace with `members = ["app"]` and no `default-members`, containing an edition-2021 package named `release-app`.
   Put `pub struct UserManager;` in the application and add a `check-app` alias without creating an xtask.
   Expect an xtask member to be added while preserving application code, aliases, and the previous default package selection.
5. **Unsupported standalone update:** create an application without a workspace declaration and an existing independent `tools/xtask` workspace.
   Add a customized scanner to that xtask and retain its existing alias.
   Expect the agent to check the application workspace and leave all files unchanged.

## Run and inspect

Use the prompts and expectations in `evals.json`.
For supported workspaces, include an existing compatible shared dependency and leave another scanner dependency absent from `[workspace.dependencies]`.
Verify that the agent inherits the existing entry, declares the missing dependency directly in xtask, and leaves root dependency definitions unchanged.
For an independent xtask, check inheritance against its own workspace rather than the application's workspace.
Save command output, exit codes, application and configuration snapshots, and unresolved problems.
Check artifacts and behavior rather than relying on the agent's report.
For unsupported projects, compare the entire project before and after to verify that refusal did not create a workspace, xtask, lockfile, or alias.
For updates, enable the customized rule in a disposable fixture to verify its diagnostic without changing the project's policy.
Include a rule added by the incoming bundle and confirm that a policy-preserving update leaves it off unless adoption was requested.

Run the executable installation tests from the repository root:

```sh
cargo test -p xtask --test install_skill --locked
```

These tests copy the runtime directly into temporary workspaces and check packaging, builds, command wiring, local changes, and runtime-only copies.
