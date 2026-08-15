#![cfg(windows)]
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod cli;

use std::process::ExitCode;

fn main() -> ExitCode {
    match cli::try_headless() {
        Some(code) => code,
        None => {
            if let Err(e) = app::run() {
                eprintln!("gui error: {e}");
                return ExitCode::from(1);
            }
            ExitCode::SUCCESS
        }
    }
}
