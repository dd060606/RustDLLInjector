use std::ffi::c_void;
use std::path::Path;
use std::ptr;

use windows_sys::Win32::Foundation::HANDLE;
use windows_sys::Win32::System::Diagnostics::Debug::WriteProcessMemory;
use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};
use windows_sys::Win32::System::Memory::{
    VirtualAllocEx, VirtualFreeEx, MEM_COMMIT, MEM_RELEASE, MEM_RESERVE, PAGE_READWRITE,
};

use crate::error::{InjectError, Result};
use crate::util::last_os_error;

pub(crate) const RW: u32 = PAGE_READWRITE;

pub(crate) fn load_library_w_addr() -> Result<*const c_void> {
    resolve_export("kernel32.dll", "LoadLibraryW")
}

pub(crate) fn nt_export(symbol: &'static str) -> Result<*const c_void> {
    resolve_export("ntdll.dll", symbol)
}

fn resolve_export(module: &'static str, symbol: &'static str) -> Result<*const c_void> {
    let mut mod_z = Vec::with_capacity(module.len() + 1);
    mod_z.extend_from_slice(module.as_bytes());
    mod_z.push(0);
    let mut sym_z = Vec::with_capacity(symbol.len() + 1);
    sym_z.extend_from_slice(symbol.as_bytes());
    sym_z.push(0);

    // SAFETY: both buffers are null-terminated ANSI.
    let handle = unsafe { GetModuleHandleA(mod_z.as_ptr()) };
    if handle.is_null() {
        return Err(InjectError::SymbolNotFound {
            module,
            symbol: module,
        });
    }
    // SAFETY: handle is valid, symbol is null-terminated.
    let proc = unsafe { GetProcAddress(handle, sym_z.as_ptr()) };
    proc.map(|f| f as *const c_void)
        .ok_or(InjectError::SymbolNotFound { module, symbol })
}

pub(crate) struct RemoteBuffer {
    process: HANDLE,
    ptr: *mut c_void,
    size: usize,
}

impl RemoteBuffer {
    pub(crate) fn alloc(process: HANDLE, size: usize, protect: u32) -> Result<Self> {
        // SAFETY: caller supplies a valid process handle with VM rights.
        let ptr = unsafe {
            VirtualAllocEx(
                process,
                ptr::null(),
                size,
                MEM_COMMIT | MEM_RESERVE,
                protect,
            )
        };
        if ptr.is_null() {
            return Err(InjectError::RemoteAlloc(last_os_error()));
        }
        Ok(Self { process, ptr, size })
    }

    pub(crate) fn write(&self, data: &[u8]) -> Result<()> {
        if data.len() > self.size {
            return Err(InjectError::RemoteWrite(std::io::Error::from(
                std::io::ErrorKind::InvalidInput,
            )));
        }
        let mut written = 0usize;
        // SAFETY: buffer is remote memory owned by process; sized correctly.
        let ok = unsafe {
            WriteProcessMemory(
                self.process,
                self.ptr,
                data.as_ptr().cast(),
                data.len(),
                &mut written,
            )
        };
        if ok == 0 || written != data.len() {
            return Err(InjectError::RemoteWrite(last_os_error()));
        }
        Ok(())
    }

    pub(crate) fn as_ptr(&self) -> *mut c_void {
        self.ptr
    }
}

impl Drop for RemoteBuffer {
    fn drop(&mut self) {
        // SAFETY: pairs with VirtualAllocEx above.
        unsafe {
            VirtualFreeEx(self.process, self.ptr, 0, MEM_RELEASE);
        }
    }
}

pub(crate) fn wide_path_bytes(path: &Path) -> Vec<u8> {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path
        .as_os_str()
        .encode_wide()
        .chain(std::iter::once(0))
        .collect();
    let mut out = Vec::with_capacity(wide.len() * 2);
    for w in wide {
        out.extend_from_slice(&w.to_le_bytes());
    }
    out
}
