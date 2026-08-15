use std::str::FromStr;

use crate::dll::DllInfo;
use crate::error::Result;
use crate::methods;
use crate::InjectRequest;

/// Selects which injection technique to use.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[non_exhaustive]
pub enum InjectionMethod {
    #[default]
    CreateRemoteThread,
    QueueUserApc,
    NtCreateThreadEx,
}

impl InjectionMethod {
    pub const ALL: &'static [InjectionMethod] = &[
        Self::CreateRemoteThread,
        Self::QueueUserApc,
        Self::NtCreateThreadEx,
    ];

    pub fn short_name(&self) -> &'static str {
        match self {
            Self::CreateRemoteThread => "create-remote-thread",
            Self::QueueUserApc => "queue-user-apc",
            Self::NtCreateThreadEx => "nt-create-thread-ex",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::CreateRemoteThread => "CreateRemoteThread",
            Self::QueueUserApc => "QueueUserAPC",
            Self::NtCreateThreadEx => "NtCreateThreadEx",
        }
    }
}

impl FromStr for InjectionMethod {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let norm: String = s.chars().map(|c| c.to_ascii_lowercase()).collect();
        match norm.as_str() {
            "create-remote-thread" | "createremotethread" | "crt" | "default" => {
                Ok(Self::CreateRemoteThread)
            }
            "queue-user-apc" | "queueuserapc" | "apc" => Ok(Self::QueueUserApc),
            "nt-create-thread-ex" | "ntcreatethreadex" | "nt" => Ok(Self::NtCreateThreadEx),
            _ => Err(format!("unknown injection method: {s}")),
        }
    }
}

/// Runtime tweaks that apply to some methods.
#[derive(Debug, Clone, Copy, Default)]
pub struct InjectionOptions {
    /// Wipe the remote buffer that held the DLL path once the loader has consumed it.
    pub clear_path_after: bool,
}

pub(crate) fn dispatch(req: &InjectRequest<'_>, _dll: &DllInfo) -> Result<()> {
    match req.method {
        InjectionMethod::CreateRemoteThread => {
            methods::create_remote_thread::inject(req.pid, req.dll_path, &req.options)
        }
        InjectionMethod::QueueUserApc => {
            methods::queue_user_apc::inject(req.pid, req.dll_path, &req.options)
        }
        InjectionMethod::NtCreateThreadEx => {
            methods::nt_create_thread_ex::inject(req.pid, req.dll_path, &req.options)
        }
    }
}
