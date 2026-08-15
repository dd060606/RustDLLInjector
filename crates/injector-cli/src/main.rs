#![cfg(windows)]

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use injector_core::{inject, list_processes, InjectRequest, InjectionMethod, InjectionOptions};

#[derive(Parser, Debug)]
#[command(
    name = "injector",
    about = "Windows DLL injector",
    version,
    disable_help_subcommand = true
)]
struct Cli {
    /// Target process id.
    #[arg(long, conflicts_with = "process")]
    pid: Option<u32>,

    /// Target process name (case-insensitive, first match wins).
    #[arg(long)]
    process: Option<String>,

    /// Path to the DLL to inject.
    #[arg(long, required_unless_present = "list")]
    dll: Option<PathBuf>,

    /// Injection method.
    #[arg(long, value_enum, default_value_t = MethodArg::CreateRemoteThread)]
    method: MethodArg,

    /// Wipe the remote path buffer after injection where applicable.
    #[arg(long)]
    clear_path: bool,

    /// List running processes and exit.
    #[arg(long, exclusive = true)]
    list: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum MethodArg {
    CreateRemoteThread,
    QueueUserApc,
    NtCreateThreadEx,
}

impl From<MethodArg> for InjectionMethod {
    fn from(v: MethodArg) -> Self {
        match v {
            MethodArg::CreateRemoteThread => Self::CreateRemoteThread,
            MethodArg::QueueUserApc => Self::QueueUserApc,
            MethodArg::NtCreateThreadEx => Self::NtCreateThreadEx,
        }
    }
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    match run(cli) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn run(cli: Cli) -> Result<(), String> {
    if cli.list {
        return do_list();
    }
    let pid = resolve_pid(&cli)?;
    let dll = cli.dll.clone().ok_or_else(|| "missing --dll".to_string())?;
    let req = InjectRequest {
        pid,
        dll_path: &dll,
        method: cli.method.into(),
        options: InjectionOptions {
            clear_path_after: cli.clear_path,
        },
    };
    inject(&req).map_err(|e| e.to_string())?;
    println!("injected {} into pid {}", dll.display(), pid);
    Ok(())
}

fn do_list() -> Result<(), String> {
    let procs = list_processes().map_err(|e| e.to_string())?;
    for p in procs {
        let arch = p
            .architecture
            .map(|a| format!("{a:?}"))
            .unwrap_or_else(|| "?".into());
        println!("{:>6}  {:>4}  {}", p.pid, arch, p.name);
    }
    Ok(())
}

fn resolve_pid(cli: &Cli) -> Result<u32, String> {
    if let Some(pid) = cli.pid {
        return Ok(pid);
    }
    let name = cli
        .process
        .as_ref()
        .ok_or_else(|| "missing --pid or --process".to_string())?;
    let procs = list_processes().map_err(|e| e.to_string())?;
    let needle = name.to_ascii_lowercase();
    procs
        .into_iter()
        .find(|p| p.name.to_ascii_lowercase() == needle)
        .map(|p| p.pid)
        .ok_or_else(|| format!("process not found: {name}"))
}
