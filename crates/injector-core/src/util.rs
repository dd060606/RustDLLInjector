use std::io;

use windows_sys::Win32::Foundation::{CloseHandle, HANDLE, INVALID_HANDLE_VALUE};

pub(crate) struct OwnedHandle(HANDLE);

impl OwnedHandle {
    /// # Safety
    /// `handle` must be a valid Win32 handle owned by the caller.
    pub(crate) unsafe fn new(handle: HANDLE) -> Option<Self> {
        if handle.is_null() || handle == INVALID_HANDLE_VALUE {
            None
        } else {
            Some(Self(handle))
        }
    }

    pub(crate) fn as_raw(&self) -> HANDLE {
        self.0
    }
}

impl Drop for OwnedHandle {
    fn drop(&mut self) {
        // SAFETY: constructor guarantees a valid handle owned by this wrapper.
        unsafe {
            CloseHandle(self.0);
        }
    }
}

pub(crate) fn last_os_error() -> io::Error {
    io::Error::last_os_error()
}
