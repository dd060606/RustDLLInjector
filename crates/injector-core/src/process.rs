use std::mem::size_of;

use windows_sys::Win32::Foundation::{FALSE, MAX_PATH};
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS,
};
use windows_sys::Win32::System::Threading::{
    IsWow64Process, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
};

use crate::error::{InjectError, Result};
use crate::util::{last_os_error, OwnedHandle};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86,
    X64,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub architecture: Option<Architecture>,
}

pub fn list_processes() -> Result<Vec<ProcessInfo>> {
    // SAFETY: CreateToolhelp32Snapshot returns INVALID_HANDLE_VALUE on failure, which OwnedHandle rejects.
    let snapshot = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
    let snap =
        unsafe { OwnedHandle::new(snapshot) }.ok_or_else(|| InjectError::Io(last_os_error()))?;

    let mut entry: PROCESSENTRY32W = unsafe { std::mem::zeroed() };
    entry.dwSize = size_of::<PROCESSENTRY32W>() as u32;

    let mut out = Vec::new();
    // SAFETY: snap.as_raw() is a valid snapshot handle; entry sized correctly.
    if unsafe { Process32FirstW(snap.as_raw(), &mut entry) } == 0 {
        return Ok(out);
    }

    loop {
        let name = read_wide(&entry.szExeFile);
        let pid = entry.th32ProcessID;
        let arch = process_architecture(pid).ok();
        out.push(ProcessInfo {
            pid,
            name,
            architecture: arch,
        });

        // SAFETY: iteration matches the Toolhelp API contract.
        if unsafe { Process32NextW(snap.as_raw(), &mut entry) } == 0 {
            break;
        }
    }
    Ok(out)
}

pub fn process_architecture(pid: u32) -> Result<Architecture> {
    let handle = open_for_query(pid)?;
    let mut is_wow64: i32 = 0;
    // SAFETY: handle is valid; out param sized correctly.
    let ok = unsafe { IsWow64Process(handle.as_raw(), &mut is_wow64) };
    if ok == 0 {
        return Err(InjectError::Io(last_os_error()));
    }
    Ok(if is_wow64 != 0 {
        Architecture::X86
    } else {
        host_native_arch()
    })
}

pub(crate) fn open_for_query(pid: u32) -> Result<OwnedHandle> {
    // SAFETY: OpenProcess returns null on failure; OwnedHandle rejects null.
    let raw = unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid) };
    unsafe { OwnedHandle::new(raw) }.ok_or_else(|| InjectError::OpenProcess {
        pid,
        source: last_os_error(),
    })
}

pub(crate) fn open_for_injection(pid: u32, access: u32) -> Result<OwnedHandle> {
    // SAFETY: See open_for_query.
    let raw = unsafe { OpenProcess(access, FALSE, pid) };
    unsafe { OwnedHandle::new(raw) }.ok_or_else(|| InjectError::OpenProcess {
        pid,
        source: last_os_error(),
    })
}

fn host_native_arch() -> Architecture {
    if cfg!(target_pointer_width = "64") {
        Architecture::X64
    } else {
        Architecture::X86
    }
}

fn read_wide(buf: &[u16]) -> String {
    let len = buf
        .iter()
        .position(|&c| c == 0)
        .unwrap_or(buf.len())
        .min(MAX_PATH as usize);
    String::from_utf16_lossy(&buf[..len])
}
