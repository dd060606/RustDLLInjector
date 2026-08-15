#![cfg(windows)]

use std::ffi::c_void;

use windows_sys::Win32::Foundation::{BOOL, HMODULE, TRUE};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    MB_ICONINFORMATION, MB_OK, MB_TOPMOST, MessageBoxW,
};

const DLL_PROCESS_ATTACH: u32 = 1;

#[no_mangle]
pub extern "system" fn DllMain(_module: HMODULE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == DLL_PROCESS_ATTACH {
        show_popup();
    }
    TRUE
}

fn show_popup() {
    let title: Vec<u16> = "Injector test".encode_utf16().chain(std::iter::once(0)).collect();
    let text: Vec<u16> =
        "Hello from the injected DLL.".encode_utf16().chain(std::iter::once(0)).collect();
    // SAFETY: both strings are null-terminated UTF-16; null hWnd is valid for MessageBox.
    unsafe {
        MessageBoxW(
            std::ptr::null_mut(),
            text.as_ptr(),
            title.as_ptr(),
            MB_OK | MB_ICONINFORMATION | MB_TOPMOST,
        );
    }
}
