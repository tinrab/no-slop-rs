//! Workspace maintenance commands.

use std::env;
use std::process::ExitCode;

use xtask::{no_slop, skill_assets};

fn main() -> ExitCode {
    let mut args = env::args_os().skip(1);
    match args.next().as_deref().and_then(|arg| arg.to_str()) {
        Some("no-slop") => no_slop::run(&args.collect::<Vec<_>>()),
        Some("sync-no-slop-skill-assets") => match skill_assets::run(&args.collect::<Vec<_>>()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error:#}");
                ExitCode::from(2)
            }
        },
        Some("help" | "--help" | "-h") | None => {
            println!("cargo xtask no-slop [options]");
            println!("cargo xtask sync-no-slop-skill-assets [--check]");
            ExitCode::SUCCESS
        }
        _ => {
            eprintln!("unknown xtask command; use cargo xtask --help");
            ExitCode::from(2)
        }
    }
}
