use std::ffi::c_void;
use std::path::Path;
use std::ptr;

use windows_sys::Win32::Foundation::{HANDLE, WAIT_OBJECT_0};
use windows_sys::Win32::System::Threading::{
    WaitForSingleObject, INFINITE, PROCESS_CREATE_THREAD, PROCESS_QUERY_INFORMATION,
    PROCESS_VM_OPERATION, PROCESS_VM_READ, PROCESS_VM_WRITE,
};

use crate::error::{InjectError, Result};
use crate::methods::loader::{load_library_w_addr, nt_export, wide_path_bytes, RemoteBuffer, RW};
use crate::process::open_for_injection;
use crate::util::OwnedHandle;
use crate::InjectionOptions;

const ACCESS: u32 = PROCESS_CREATE_THREAD
    | PROCESS_QUERY_INFORMATION
    | PROCESS_VM_OPERATION
    | PROCESS_VM_WRITE
    | PROCESS_VM_READ;

type NtCreateThreadEx = unsafe extern "system" fn(
    thread: *mut HANDLE,
    desired_access: u32,
    object_attributes: *mut c_void,
    process: HANDLE,
    start_routine: *mut c_void,
    argument: *mut c_void,
    create_flags: u32,
    zero_bits: usize,
    stack_size: usize,
    max_stack_size: usize,
    attribute_list: *mut c_void,
) -> i32;

const THREAD_ALL_ACCESS: u32 = 0x001F_FFFF;

pub(crate) fn inject(pid: u32, dll_path: &Path, _opts: &InjectionOptions) -> Result<()> {
    let process = open_for_injection(pid, ACCESS)?;
    let load_lib = load_library_w_addr()?;
    let path_bytes = wide_path_bytes(dll_path);

    let buffer = RemoteBuffer::alloc(process.as_raw(), path_bytes.len(), RW)?;
    buffer.write(&path_bytes)?;

    let raw_fn = nt_export("NtCreateThreadEx")?;
    // SAFETY: signature matches the documented NtCreateThreadEx prototype.
    let nt_create: NtCreateThreadEx =
        unsafe { std::mem::transmute::<*const c_void, NtCreateThreadEx>(raw_fn) };

    let mut thread_handle: HANDLE = ptr::null_mut();
    // SAFETY: all pointers valid; process has CREATE_THREAD; start is a remote code address.
    let status = unsafe {
        nt_create(
            &mut thread_handle,
            THREAD_ALL_ACCESS,
            ptr::null_mut(),
            process.as_raw(),
            load_lib as *mut c_void,
            buffer.as_ptr(),
            0,
            0,
            0,
            0,
            ptr::null_mut(),
        )
    };
    if status < 0 {
        return Err(InjectError::NtStatus(status as u32));
    }

    let thread =
        unsafe { OwnedHandle::new(thread_handle) }.ok_or(InjectError::NtStatus(status as u32))?;
    // SAFETY: valid thread handle.
    let wait = unsafe { WaitForSingleObject(thread.as_raw(), INFINITE) };
    if wait != WAIT_OBJECT_0 {
        return Err(InjectError::RemoteTimeout);
    }
    Ok(())
}
