# Windows installer

An Inno Setup script that packages the GUI (`RustDLLInjector.exe`) into a
single-file installer for Windows.

## Prerequisites

- [Inno Setup 6](https://jrsoftware.org/isdl.php) installed. The compiler
  binary is `iscc.exe` (typically at
  `C:\Program Files (x86)\Inno Setup 6\iscc.exe`).

## Build

1. Build the release binary from the repo root (this produces
   `target/release/RustDLLInjector.exe` and embeds the app icon):

    ```powershell
    ./build.ps1
    ```

2. Compile the installer:

    ```powershell
    iscc installer\RustDLLInjector.iss
    ```

    The setup executable lands in `installer\dist\` as
    `RustDLLInjector-setup-<version>.exe`.
