use std::mem::size_of;
use std::path::Path;

use windows_sys::Win32::Foundation::FALSE;
use windows_sys::Win32::System::Diagnostics::ToolHelp::{
    CreateToolhelp32Snapshot, Thread32First, Thread32Next, TH32CS_SNAPTHREAD, THREADENTRY32,
};
use windows_sys::Win32::System::Threading::{
    OpenThread, QueueUserAPC, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ,
    PROCESS_VM_WRITE, THREAD_SET_CONTEXT,
};

use crate::error::{InjectError, Result};
use crate::methods::loader::{load_library_w_addr, wide_path_bytes, RemoteBuffer, RW};
use crate::process::open_for_injection;
use crate::util::{last_os_error, OwnedHandle};
use crate::InjectionOptions;

type PapcFunc = unsafe extern "system" fn(usize);

const ACCESS: u32 =
    PROCESS_QUERY_INFORMATION | PROCESS_VM_OPERATION | PROCESS_VM_WRITE | PROCESS_VM_READ;

pub(crate) fn inject(pid: u32, dll_path: &Path, _opts: &InjectionOptions) -> Result<()> {
    let process = open_for_injection(pid, ACCESS)?;
    let load_lib = load_library_w_addr()?;
    let path_bytes = wide_path_bytes(dll_path);

    let buffer = RemoteBuffer::alloc(process.as_raw(), path_bytes.len(), RW)?;
    buffer.write(&path_bytes)?;

    // SAFETY: LoadLibraryW matches PAPCFUNC ABI (single pointer-sized arg, no return).
    let apc: Option<PapcFunc> =
        Some(unsafe { std::mem::transmute::<*const std::ffi::c_void, PapcFunc>(load_lib) });

    let threads = enumerate_threads(pid)?;
    if threads.is_empty() {
        return Err(InjectError::ApcQueueFailed);
    }

    let mut queued = 0usize;
    for tid in threads {
        // SAFETY: OpenThread returns null on failure.
        let raw = unsafe { OpenThread(THREAD_SET_CONTEXT, FALSE, tid) };
        let Some(thread) = (unsafe { OwnedHandle::new(raw) }) else {
            continue;
        };
        // SAFETY: valid thread handle, valid APC ptr, argument is a remote address.
        let ok = unsafe { QueueUserAPC(apc, thread.as_raw(), buffer.as_ptr() as usize) };
        if ok != 0 {
            queued += 1;
        }
    }

    if queued == 0 {
        return Err(InjectError::ApcQueueFailed);
    }
    std::mem::forget(buffer);
    Ok(())
}

fn enumerate_threads(pid: u32) -> Result<Vec<u32>> {
    // SAFETY: snapshot handle checked below.
    let raw = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD, 0) };
    let snap =
        unsafe { OwnedHandle::new(raw) }.ok_or_else(|| InjectError::ThreadEnum(last_os_error()))?;

    let mut entry: THREADENTRY32 = unsafe { std::mem::zeroed() };
    entry.dwSize = size_of::<THREADENTRY32>() as u32;
    let mut out = Vec::new();
    // SAFETY: iteration follows Toolhelp contract.
    if unsafe { Thread32First(snap.as_raw(), &mut entry) } == 0 {
        return Ok(out);
    }
    loop {
        if entry.th32OwnerProcessID == pid {
            out.push(entry.th32ThreadID);
        }
        // SAFETY: same as above.
        if unsafe { Thread32Next(snap.as_raw(), &mut entry) } == 0 {
            break;
        }
    }
    Ok(out)
}
