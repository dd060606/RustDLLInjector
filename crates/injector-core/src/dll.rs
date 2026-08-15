use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};

use crate::error::{InjectError, Result};
use crate::process::Architecture;

const IMAGE_DOS_SIGNATURE: u16 = 0x5A4D;
const IMAGE_NT_SIGNATURE: u32 = 0x0000_4550;
const IMAGE_FILE_MACHINE_I386: u16 = 0x014C;
const IMAGE_FILE_MACHINE_AMD64: u16 = 0x8664;
const IMAGE_FILE_DLL: u16 = 0x2000;

const HEADER_SNIFF_LEN: usize = 0x400;

/// Static description of a payload DLL loaded from disk.
#[derive(Debug, Clone)]
pub struct DllInfo {
    pub path: PathBuf,
    pub architecture: Architecture,
}

pub fn inspect_dll(path: &Path) -> Result<DllInfo> {
    if !path.exists() {
        return Err(InjectError::DllNotFound(path.display().to_string()));
    }
    let mut file = File::open(path)?;
    let mut head = vec![0u8; HEADER_SNIFF_LEN];
    let n = file.read(&mut head)?;
    head.truncate(n);
    let architecture = dll_architecture(&head)?;
    Ok(DllInfo {
        path: path.to_path_buf(),
        architecture,
    })
}

pub fn dll_architecture(bytes: &[u8]) -> Result<Architecture> {
    if bytes.len() < 0x40 {
        return Err(InjectError::InvalidPe);
    }
    let dos_sig = u16::from_le_bytes(bytes[0..2].try_into().unwrap());
    if dos_sig != IMAGE_DOS_SIGNATURE {
        return Err(InjectError::InvalidPe);
    }
    let nt_offset = u32::from_le_bytes(bytes[0x3C..0x40].try_into().unwrap()) as usize;
    if bytes.len() < nt_offset + 0x18 {
        return Err(InjectError::InvalidPe);
    }
    let nt_sig = u32::from_le_bytes(bytes[nt_offset..nt_offset + 4].try_into().unwrap());
    if nt_sig != IMAGE_NT_SIGNATURE {
        return Err(InjectError::InvalidPe);
    }
    let file_hdr = nt_offset + 4;
    let machine = u16::from_le_bytes(bytes[file_hdr..file_hdr + 2].try_into().unwrap());
    let characteristics =
        u16::from_le_bytes(bytes[file_hdr + 18..file_hdr + 20].try_into().unwrap());
    if characteristics & IMAGE_FILE_DLL == 0 {
        return Err(InjectError::InvalidPe);
    }
    match machine {
        IMAGE_FILE_MACHINE_I386 => Ok(Architecture::X86),
        IMAGE_FILE_MACHINE_AMD64 => Ok(Architecture::X64),
        _ => Err(InjectError::InvalidPe),
    }
}
