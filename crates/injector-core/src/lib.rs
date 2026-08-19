//! Windows DLL injection primitives.
//!
//! Exposes a small API to enumerate processes, validate a payload DLL, and
//! inject it using one of several supported methods.

#![cfg(windows)]
#![deny(unsafe_op_in_unsafe_fn)]

mod dll;
mod error;
mod method;
mod process;
mod util;

pub mod methods;

pub use dll::{dll_architecture, inspect_dll, DllInfo};
pub use error::{InjectError, Result};
pub use method::{InjectionMethod, InjectionOptions};
pub use process::{
    injector_architecture, list_processes, process_architecture, Architecture, ProcessInfo,
};

use std::path::Path;

/// A single injection request.
#[derive(Debug, Clone)]
pub struct InjectRequest<'a> {
    pub pid: u32,
    pub dll_path: &'a Path,
    pub method: InjectionMethod,
    pub options: InjectionOptions,
}

/// Inject `dll_path` into the process identified by `pid` using `method`.
pub fn inject(req: &InjectRequest<'_>) -> Result<()> {
    let dll = inspect_dll(req.dll_path)?;
    let process_arch = process_architecture(req.pid)?;

    let injector_arch = injector_architecture();
    if injector_arch != process_arch {
        return Err(InjectError::InjectorArchitectureMismatch {
            pid: req.pid,
            injector: injector_arch,
            process: process_arch,
        });
    }

    if dll.architecture != process_arch {
        return Err(InjectError::ArchitectureMismatch {
            process: process_arch,
            dll: dll.architecture,
        });
    }
    method::dispatch(req, &dll)
}
