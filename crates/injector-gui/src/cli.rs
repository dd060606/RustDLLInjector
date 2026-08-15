use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, ValueEnum};
use injector_core::{inject, list_processes, InjectRequest, InjectionMethod, InjectionOptions};

#[derive(Parser, Debug)]
#[command(name = "injector-gui", about = "Windows DLL injector (GUI)", version)]
struct HeadlessArgs {
    #[arg(long)]
    pid: Option<u32>,
    #[arg(long)]
    process: Option<String>,
    #[arg(long)]
    dll: Option<PathBuf>,
    #[arg(long, value_enum)]
    method: Option<Method>,
    #[arg(long)]
    clear_path: bool,
    #[arg(long)]
    list: bool,
}

#[derive(Copy, Clone, Debug, ValueEnum)]
enum Method {
    CreateRemoteThread,
    QueueUserApc,
    NtCreateThreadEx,
}

impl From<Method> for InjectionMethod {
    fn from(m: Method) -> Self {
        match m {
            Method::CreateRemoteThread => Self::CreateRemoteThread,
            Method::QueueUserApc => Self::QueueUserApc,
            Method::NtCreateThreadEx => Self::NtCreateThreadEx,
        }
    }
}

pub fn try_headless() -> Option<ExitCode> {
    let raw: Vec<String> = std::env::args().collect();
    if raw.len() <= 1 {
        return None;
    }
    let args = match HeadlessArgs::try_parse() {
        Ok(a) => a,
        Err(e) => {
            let _ = e.print();
            return Some(ExitCode::from(2));
        }
    };
    if !args.list && args.dll.is_none() && args.pid.is_none() && args.process.is_none() {
        return None;
    }
    Some(run(args))
}

fn run(a: HeadlessArgs) -> ExitCode {
    if a.list {
        return list();
    }
    let pid = match resolve_pid(a.pid, a.process.as_deref()) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    let Some(dll) = a.dll else {
        eprintln!("error: missing --dll");
        return ExitCode::from(1);
    };
    let req = InjectRequest {
        pid,
        dll_path: &dll,
        method: a.method.map(Into::into).unwrap_or_default(),
        options: InjectionOptions {
            clear_path_after: a.clear_path,
        },
    };
    match inject(&req) {
        Ok(()) => {
            println!("injected {} into pid {}", dll.display(), pid);
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn list() -> ExitCode {
    match list_processes() {
        Ok(procs) => {
            for p in procs {
                let arch = p
                    .architecture
                    .map(|a| format!("{a:?}"))
                    .unwrap_or_else(|| "?".into());
                println!("{:>6}  {:>4}  {}", p.pid, arch, p.name);
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(1)
        }
    }
}

fn resolve_pid(pid: Option<u32>, name: Option<&str>) -> Result<u32, String> {
    if let Some(p) = pid {
        return Ok(p);
    }
    let name = name.ok_or_else(|| "missing --pid or --process".to_string())?;
    let procs = list_processes().map_err(|e| e.to_string())?;
    let needle = name.to_ascii_lowercase();
    procs
        .into_iter()
        .find(|p| p.name.to_ascii_lowercase() == needle)
        .map(|p| p.pid)
        .ok_or_else(|| format!("process not found: {name}"))
}
