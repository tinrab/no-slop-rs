# Upstream provenance

This scanner is maintained in [no-slop-rs](https://github.com/tinrab/no-slop-rs).

## Design

- Rules are copied into the target repository, which owns them from then on, and local edits are policy to preserve.
- One agent skill covers fresh installs and reviewed updates, with separate procedures.
- Canonical source is synchronized into the skill's assets, and a check mode reports stale assets.
- Provenance is recorded in this file beside the code it describes.
- Agent tooling directories are excluded from scans by default.
- Rules run from `cargo xtask no-slop` on `syn` and `ra-ap-rustc_lexer`.
- Agents copy the runtime directly; there is no installer script.

## Updating and verification

This section applies to the `no-slop-rs` repository; installed copies follow the skill's update procedure instead.

After changing the scanner, run `cargo xtask sync-no-slop-skill-assets` and `cargo test --workspace --locked`.
