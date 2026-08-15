#!/usr/bin/env bash
# Release build wrapper. Same purpose as build.ps1 for POSIX shells / git-bash on Windows.
set -euo pipefail

home_dir="${USERPROFILE:-${HOME:-}}"
cargo_home="${CARGO_HOME:-${home_dir}/.cargo}"
rustup_home="${RUSTUP_HOME:-${home_dir}/.rustup}"
repo_dir="$(cd "$(dirname "$0")" && pwd)"

flags=(
    "-Clink-arg=/DEBUG:NONE"
    "-Clink-arg=/PDBALTPATH:none"
)
[[ -n "$home_dir"    ]] && flags+=("--remap-path-prefix=${home_dir}=[home]")
[[ -n "$cargo_home"  ]] && flags+=("--remap-path-prefix=${cargo_home}=[cargo]")
[[ -n "$rustup_home" ]] && flags+=("--remap-path-prefix=${rustup_home}=[rustup]")
[[ -n "$repo_dir"    ]] && flags+=("--remap-path-prefix=${repo_dir}=[src]")

sep=$'\x1f'
joined="${flags[0]}"
for f in "${flags[@]:1}"; do joined+="${sep}${f}"; done

export CARGO_ENCODED_RUSTFLAGS="$joined"
exec cargo build --release "$@"
