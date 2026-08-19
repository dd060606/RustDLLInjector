use std::io;

use thiserror::Error;

use crate::process::Architecture;

pub type Result<T> = std::result::Result<T, InjectError>;

#[derive(Debug, Error)]
pub enum InjectError {
    #[error("process {0} not found")]
    ProcessNotFound(u32),

    #[error("failed to open process {pid}: {source}")]
    OpenProcess {
        pid: u32,
        #[source]
        source: io::Error,
    },

    #[error("dll not found: {0}")]
    DllNotFound(String),

    #[error("invalid PE file")]
    InvalidPe,

    #[error("architecture mismatch: process is {process:?}, dll is {dll:?}")]
    ArchitectureMismatch {
        process: Architecture,
        dll: Architecture,
    },

    #[error(
        "this injector binary is {injector:?} but process {pid} is {process:?} — \
         run the {process:?} build of the injector against this target instead"
    )]
    InjectorArchitectureMismatch {
        pid: u32,
        injector: Architecture,
        process: Architecture,
    },

    #[error("failed to resolve symbol {symbol} in {module}")]
    SymbolNotFound {
        module: &'static str,
        symbol: &'static str,
    },

    #[error("remote allocation failed")]
    RemoteAlloc(#[source] io::Error),

    #[error("remote write failed")]
    RemoteWrite(#[source] io::Error),

    #[error("remote thread creation failed")]
    RemoteThread(#[source] io::Error),

    #[error("thread enumeration failed")]
    ThreadEnum(#[source] io::Error),

    #[error("APC queue failed on all target threads")]
    ApcQueueFailed,

    #[error("nt call failed with status 0x{0:08X}")]
    NtStatus(u32),

    #[error("injected thread did not signal completion")]
    RemoteTimeout,

    #[error(transparent)]
    Io(#[from] io::Error),
}
