# RustDLLInjector

A lightweight Windows DLL injector written in Rust. Structured as a Cargo
workspace with a shared core library and two binaries: a small CLI and a
minimal GUI (which also accepts CLI arguments for headless use).

Any DLL that exports a valid `DllMain` will work.

## Supported Injection Methods

| Method                           | Description                                                                                                                       |
| -------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `create-remote-thread` (default) | Allocates memory in the target, writes the DLL path, spawns a remote thread on `LoadLibraryW`. Classic and universally supported. |
| `queue-user-apc`                 | Queues an APC pointing at `LoadLibraryW` to every thread of the target; the first alertable thread runs it.                       |
| `nt-create-thread-ex`            | Same effect as `CreateRemoteThread` but calls the `ntdll` layer directly via `NtCreateThreadEx`.                                  |

## Build

Prerequisites: a stable Rust toolchain (MSRV 1.81) on Windows with the MSVC
target.

Release build (recommended — sanitizes local filesystem paths so the user
name, `.cargo`, `.rustup` and repo paths do not end up in the binaries):

```powershell
./build.ps1
```

```bash
./build.sh
```

The wrapper is a thin shim around `cargo build --release` that sets
`CARGO_ENCODED_RUSTFLAGS` with per-host `--remap-path-prefix` entries. Any
extra arguments are forwarded, e.g. `./build.ps1 -p injector-cli`.

Both binaries land in `target/release/`:

- `injector.exe` — command-line interface
- `RustDLLInjector.exe` — graphical interface (also usable headlessly)

### Windows installer

The `installer/` directory contains an Inno Setup script that packages the
GUI into a single-file `.exe` installer. See
[`installer/README.md`](installer/README.md) for details — in short: build
the release binary, then run `iscc installer\RustDLLInjector.iss`.

The DLL you inject must match the target process architecture. To inject
into a 32-bit process, build with `--target i686-pc-windows-msvc`.

## Usage

### CLI

```
injector [OPTIONS] --dll <PATH>

Options:
  --pid <PID>         Target process id
  --process <NAME>    Target process name (case-insensitive)
  --dll <PATH>        Path to the DLL to inject
  --method <METHOD>   create-remote-thread | queue-user-apc | nt-create-thread-ex
                      [default: create-remote-thread]
  --clear-path        Wipe the remote path buffer after injection
  --list              List running processes and exit
  -h, --help          Show help
  -V, --version       Show version
```

Examples:

```bash
injector --process app.exe --dll plugin.dll
injector --pid 4321 --dll .\payload.dll --method queue-user-apc
injector --list
```

### GUI

Launch with no arguments to open the window:

```bash
RustDLLInjector.exe
```

- Pick a process (filter box narrows the list)
- Enter or browse to a DLL path
- Click **Inject**

Advanced controls (method selection, options) live in the **settings** panel
on the right; the default method needs no interaction with it.

Passing CLI arguments to the GUI binary runs it headlessly with the same
flags as `injector.exe` — useful for scripting the GUI build without a
second binary.

## Architecture

```
injector-core    (lib)  All injection logic and PE inspection.
injector-cli     (bin)  Thin clap wrapper over the core.
injector-gui     (bin)  eframe/egui front-end, headless-capable.
```

`injector-core` exposes `inject(&InjectRequest)`. Each method lives in its
own module under `methods/` and is dispatched from an `InjectionMethod`
enum. The binaries contain no injection logic — only argument parsing,
process selection, and status reporting.

The release profile is tuned for size (`opt-level = "z"`, fat LTO, single
codegen unit, symbols stripped, `panic = "abort"`).

## Notes

- The injector must run at the same or higher integrity level as the
  target.
- Always match architecture: an x64 injector cannot load an x86 DLL, and
  vice versa. The core validates this before touching the target.
