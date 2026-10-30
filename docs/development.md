# Development and validation

## Workspace

| Crate | Responsibility |
| --- | --- |
| anisette | Protocol types, provisioning state machine, Apple HTTPS exchange, shared emulator memory |
| adi-android | Pinned Android ELF loading, ARM64 execution, ADI operations, APK import |
| adi-macos | Pinned Mach-O loading, x86_64 execution, CF/XPC support, ADI state |
| adi-windows | Pinned PE loading, x86_64 execution, bounded Win32 support, ADI state, installer extraction |
| anisette-server | CLI, HTTP/WebSocket routes, bounded workers, private V1 persistence, shutdown |

The toolchain is pinned to Rust 1.95.0. Building Unicorn requires CMake, a C/C++ compiler, and libclang. Use `Cargo.lock` for reproducible dependency resolution.

## Offline checks

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps --all-features --locked
```

Lints are defined in the root `Cargo.toml` under `[workspace.lints]`; each crate inherits them with `[lints] workspace = true`. Checks cover unsafe justifications, unsafe function bodies, unused return values, public fallible API `# Errors` documentation, locks/borrows across await, and rustdoc links. CI treats warnings as errors. Do not add `deny(warnings)` to source; use a narrowly scoped `#[expect(..., reason = "...")]` only for a genuine false positive.

The default gate does not enable all of `pedantic`, `nursery`, or `restriction`. Optional review can use `cargo clippy --workspace --all-targets --all-features -- -W clippy::pedantic`. Emulator ABI bit patterns and fixed addresses must retain their meaning when addressing lints.

Offline tests require no Apple files or external network access. They require a host that permits Unicorn JIT and loopback sockets. A sandbox that denies JIT can cause SIGILL; run emulator tests in a JIT-capable environment. Ignored tests do not count as successful real-backend validation.

Tests cover protocol encoding and limits, session ordering and cancellation, WebSocket success/timeouts/disconnection/shutdown, worker capacity retention after request timeouts, private state permissions/locking/atomic saves, invalid-state rejection, binary parser bounds, and emulator memory/file/execution limits. Fixtures must contain only synthetic data, never real device secrets or Apple binaries.

## Real-backend validation

Supply all three pinned binary paths explicitly:

```sh
ANISETTE_ANDROID_LIBS=/path/to/android-libraries \
ANISETTE_MACOS_BINARY=/path/to/CoreADI-26.x86_64 \
ANISETTE_WINDOWS_BINARY=/path/to/CoreADI.dll \
  sh scripts/verify-backends.sh
```

The script rejects missing variables or files and runs the ignored tests. These tests contact Apple to register independent temporary devices, require Unicorn JIT, and do not log into accounts or print secrets. Temporary service identities are deleted automatically.

The service tests cover V1 requests and restart, stable device identity/MID, repeated V3 snapshot restoration, WebSocket provisioning, request/session limits, and lock release on shutdown. Android uses standard V3 messages; macOS/Windows use the routing extension. Backend tests also check cancellation followed by another provisioning attempt and rejection of missing routing after native completion.

To run one service test:

```sh
ANISETTE_ANDROID_LIBS=/path/to/android-libraries \
  cargo test -p anisette-server live_android_http_restart_and_standard_v3 -- --ignored
ANISETTE_MACOS_BINARY=/path/to/CoreADI-26.x86_64 \
  cargo test -p anisette-server live_http_restart_v3_restore_and_extended_provisioning -- --ignored
ANISETTE_WINDOWS_BINARY=/path/to/CoreADI.dll \
  cargo test -p anisette-server live_windows_http_restart_and_extended_v3 -- --ignored
```

For local binary checks without contacting Apple:

```sh
cargo run -p adi-macos --example inspect -- /path/to/CoreADI-26.x86_64
ANISETTE_MACOS_BINARY=/path/to/CoreADI-26.x86_64 \
  cargo test -p adi-macos --test binary -- --ignored
ANISETTE_WINDOWS_BINARY=/path/to/CoreADI.dll \
  cargo test -p adi-windows --test binary -- --ignored
cargo run -p adi-android --example boot -- /path/to/android-libraries
```

Each backend also provides a `provision` example for an independent device-only check. These examples contact Apple and verify OTP/state restoration without persisting or printing secret material.

Apple provisioning uses system curl with certificate verification, ignores .curlrc, disables proxies and redirects, and permits only fixed Apple HTTPS hosts. Requests are limited to 1 MiB, 20 seconds for transfer, and 25 seconds of host waiting; exceeding a limit terminates and reaps the child process. Headers and CPIM pass through stdin, not command arguments or temporary files. Configure custom trust roots through curl-supported CA settings without disabling verification.

## CI and containers

CI runs fmt, Clippy, offline tests, and rustdoc on Linux using `rust-toolchain.toml`. A dependent image job uses Docker Buildx with GitHub Actions layer caching, uses QEMU and the containerd image store to build and load both architectures, checks that all three bundled backends load on each architecture without network access, and publishes to GHCR only after those checks pass. Pull requests build and test but never log in or publish. macOS and Windows host checks remain available locally.

Pushes to `main` publish `ghcr.io/fdkevin0/anisette-server-rs:main` and `:latest`; version tags such as `v0.1.0` publish `:0.1.0`. `latest` tracks the latest successful main build; version tags do not move it. Each published build also receives a `sha-<short commit>` tag. Images target `linux/amd64` and `linux/arm64`; the existing `0.1.0` release remains AMD64-only. Publishing uses the repository's `GITHUB_TOKEN` with `packages: write`; no personal access token is required. The first package is private by default; change its visibility in GitHub package settings if anonymous pulls are needed.

The Dockerfile and binary loaders enforce the pinned hashes. Container smoke checks do not register devices or verify Apple account authentication. See [operations](operations.md#docker) for runtime configuration.
