use std::path::Path;
use std::ptr;

use windows_sys::Win32::Foundation::WAIT_OBJECT_0;
use windows_sys::Win32::System::Threading::{
    CreateRemoteThread, WaitForSingleObject, INFINITE, LPTHREAD_START_ROUTINE,
    PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION, PROCESS_VM_OPERATION, PROCESS_VM_READ,
    PROCESS_VM_WRITE,
};

use crate::error::{InjectError, Result};
use crate::methods::loader::{load_library_w_addr, wide_path_bytes, RemoteBuffer, RW};
use crate::process::open_for_injection;
use crate::util::{last_os_error, OwnedHandle};
use crate::InjectionOptions;

const ACCESS: u32 = PROCESS_CREATE_THREAD
    | PROCESS_QUERY_INFORMATION
    | PROCESS_VM_OPERATION
    | PROCESS_VM_WRITE
    | PROCESS_VM_READ;

pub(crate) fn inject(pid: u32, dll_path: &Path, opts: &InjectionOptions) -> Result<()> {
    let process = open_for_injection(pid, ACCESS)?;
    let load_lib = load_library_w_addr()?;
    let path_bytes = wide_path_bytes(dll_path);

    let buffer = RemoteBuffer::alloc(process.as_raw(), path_bytes.len(), RW)?;
    buffer.write(&path_bytes)?;

    // SAFETY: LoadLibraryW matches the LPTHREAD_START_ROUTINE ABI (single pointer arg, DWORD return).
    let start: LPTHREAD_START_ROUTINE = Some(unsafe {
        std::mem::transmute::<
            *const std::ffi::c_void,
            unsafe extern "system" fn(*mut std::ffi::c_void) -> u32,
        >(load_lib)
    });

    // SAFETY: process has CREATE_THREAD; start is a valid remote address.
    let thread_raw = unsafe {
        CreateRemoteThread(
            process.as_raw(),
            ptr::null(),
            0,
            start,
            buffer.as_ptr(),
            0,
            ptr::null_mut(),
        )
    };
    let thread = unsafe { OwnedHandle::new(thread_raw) }
        .ok_or_else(|| InjectError::RemoteThread(last_os_error()))?;

    // SAFETY: valid thread handle.
    let wait = unsafe { WaitForSingleObject(thread.as_raw(), INFINITE) };
    if wait != WAIT_OBJECT_0 {
        return Err(InjectError::RemoteTimeout);
    }

    if opts.clear_path_after {
        let zeros = vec![0u8; path_bytes.len()];
        let _ = buffer.write(&zeros);
    }
    Ok(())
}
