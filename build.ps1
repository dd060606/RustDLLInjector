#!/usr/bin/env pwsh

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

$targets = @("x86_64-pc-windows-msvc", "i686-pc-windows-msvc")

foreach ($target in $targets) {
    Write-Host "==> installing target $target (no-op if already present)"
    & rustup target add $target
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }

    Write-Host "==> building $target"
    $cargoArgs = @("build", "--release", "--target", $target) + $args
    & cargo @cargoArgs
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "==> done, binaries in:"
foreach ($target in $targets) {
    Write-Host "  target/$target/release/"
}
