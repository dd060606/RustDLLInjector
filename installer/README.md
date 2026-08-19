# Windows installer

An Inno Setup script that packages both the GUI (`RustDLLInjector.exe`) and
CLI (`RustDLLInjector-CLI.exe`), **for both x64 and x86**, into a single
installer.
Installed layout:

```
{app}\x64\RustDLLInjector.exe
{app}\x64\RustDLLInjector-CLI.exe
{app}\x86\RustDLLInjector.exe
{app}\x86\RustDLLInjector-CLI.exe
```

## Prerequisites

- [Inno Setup 6](https://jrsoftware.org/isdl.php) installed. The compiler
  binary is `iscc.exe` (typically at
  `C:\Program Files (x86)\Inno Setup 6\iscc.exe`).

## Build

1. Build both architectures from the repo root (this produces
   `target/x86_64-pc-windows-msvc/release/` and
   `target/i686-pc-windows-msvc/release/`, each with the GUI and CLI exes):

    ```powershell
    ./build.ps1
    ```

2. Compile the installer:

    ```powershell
    iscc installer\RustDLLInjector.iss
    ```

    The setup executable lands in `installer\dist\` as
    `RustDLLInjector-setup-<version>.exe`.
