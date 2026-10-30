#!/bin/sh
set -eu
: "${ANISETTE_ANDROID_LIBS:?set the pinned Android library directory}"
: "${ANISETTE_MACOS_BINARY:?set the pinned macOS binary path}"
: "${ANISETTE_WINDOWS_BINARY:?set the pinned Windows binary path}"
test -f "$ANISETTE_ANDROID_LIBS/libCoreADI.so"
test -f "$ANISETTE_ANDROID_LIBS/libstoreservicescore.so"
test -f "$ANISETTE_MACOS_BINARY"
test -f "$ANISETTE_WINDOWS_BINARY"
exec cargo test --manifest-path "$(dirname "$0")/../Cargo.toml" --locked --workspace --all-targets -- --ignored --test-threads=1
