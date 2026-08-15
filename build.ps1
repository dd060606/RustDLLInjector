#!/usr/bin/env pwsh
# Release build wrapper. Sets --remap-path-prefix for the current user home,
# CARGO_HOME and RUSTUP_HOME so no local filesystem paths (including the
# user name) end up embedded in panic strings inside the produced binaries.

$ErrorActionPreference = "Stop"

function Resolve-Or-Empty($path) {
    if ([string]::IsNullOrEmpty($path)) { return "" }
    try { return (Resolve-Path -LiteralPath $path -ErrorAction Stop).Path } catch { return $path }
}

$homeDir    = Resolve-Or-Empty $env:USERPROFILE
$cargoHome  = Resolve-Or-Empty ($env:CARGO_HOME  ? $env:CARGO_HOME  : (Join-Path $env:USERPROFILE ".cargo"))
$rustupHome = Resolve-Or-Empty ($env:RUSTUP_HOME ? $env:RUSTUP_HOME : (Join-Path $env:USERPROFILE ".rustup"))
$repoDir    = Resolve-Or-Empty (Split-Path -Parent $PSCommandPath)

$flags = @(
    "-Clink-arg=/DEBUG:NONE",
    "-Clink-arg=/PDBALTPATH:none"
)
if ($homeDir)    { $flags += "--remap-path-prefix=$homeDir=[home]" }
if ($cargoHome)  { $flags += "--remap-path-prefix=$cargoHome=[cargo]" }
if ($rustupHome) { $flags += "--remap-path-prefix=$rustupHome=[rustup]" }
if ($repoDir)    { $flags += "--remap-path-prefix=$repoDir=[src]" }

$env:CARGO_ENCODED_RUSTFLAGS = ($flags -join [char]0x1f)

$cargoArgs = @("build", "--release") + $args
& cargo @cargoArgs
exit $LASTEXITCODE
