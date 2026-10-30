# Running and deployment

Start with the [README quick start](../README.md#quick-start), or use a published image below. For client integration and supported hosts, see [compatibility](compatibility.md).

## Docker

One Linux image bundles all three pinned Apple backends. No binary mount is required. See [compatibility](compatibility.md) when choosing a backend.

CI publishes one multi-platform image for `linux/amd64` and `linux/arm64` to GHCR after successful checks on `main` and version tags. Docker automatically selects the matching host architecture.

Run `ghcr.io/fdkevin0/anisette-server-rs:main` directly as shown below; no local build is required. `main` and `latest` track successful main builds. Version tags pin a release; `:0.1.0` was published for Linux AMD64 only.

For example, initialize an independent macOS device:

```sh
docker volume create anisette-macos-data
docker run --rm --name anisette-macos -p 127.0.0.1:6969:6969 \
  --mount type=volume,src=anisette-macos-data,dst=/data \
  ghcr.io/fdkevin0/anisette-server-rs:main --backend macos
```

Use the same command and volume on subsequent starts to restore the saved device.

Use the same image with `--backend android` or `--backend windows` for the other backends. Each instance selects one backend. Run multiple containers with separate host ports and separate state volumes to serve multiple backends concurrently. Never share a state volume between backends or running instances.

## Running locally

Rust 1.95.0 is pinned. Install CMake, a C/C++ compiler, and libclang, then [prepare the Apple binaries](../assets/README.md). Private state storage supports Unix; Windows-host ACL storage is not implemented.

```sh
cargo run --release -p anisette-server -- --backend android \
  --binary ./assets/local/android-4.9.6 --state-dir ./data
```

For macOS ADI, use `--backend macos --binary /path/to/CoreADI-26.x86_64`; for Windows ADI, use `--backend windows --binary /path/to/CoreADI.dll`. Backend names describe guest binaries, not the host operating system.

## Configuration

| Setting | CLI / environment | Local default | Docker default |
| --- | --- | --- | --- |
| Backend | `--backend` | `android` | `android` |
| Listener | `--listen` / `ANISETTE_LISTEN` | `127.0.0.1:6969` | `0.0.0.0:6969` |
| State directory | `--state-dir` / `ANISETTE_STATE_DIR` | `data` | `/data` |
| Binary override | `--binary` | Backend path below | Backend path below |

CLI arguments override environment variables. Binary defaults are `/assets` for Android, `/assets/CoreADI-26.x86_64` for macOS, and `/assets/CoreADI.dll` for Windows.

Keep the host port bound to loopback for local use. Public deployments need a reverse proxy with TLS and access control.

## Device state and restart

Startup and V1 requests automatically provision an unprovisioned device with Apple. Provisioning must succeed and state must be saved before headers are served. Existing valid state is restored; invalid state fails startup and is never reset automatically. Generating OTPs for provisioned devices needs no network access.

Keep a separate state directory or volume for every instance and backend. The Docker service runs as UID/GID 10001. A new named volume inherits private `/data` permissions; an existing bind mount must belong to UID 10001 with mode 0700.

### Storage and shutdown details

The parent directory must exist. State directories use mode 0700 and files use 0600. Incorrect owners, symlinks, and files with multiple hard links are rejected. An exclusive directory lock prevents two services from sharing an identity; do not delete `identity.lock` while the service is running.

The state record requires version, backend, identifier, adi_pb, and complete headers: device_id, local_user_id (64 hexadecimal characters), serial, client_info, and user_agent. Missing or invalid fields fail startup.

Saves use temporary-file fsync, rename, and directory fsync; save failure disables the V1 worker. SIGINT/SIGTERM stops new requests, closes WebSockets, waits for bounded execution and pending saves, and releases the lock.

## Proxies and offline checks

Provisioning honors `HTTPS_PROXY`, `HTTP_PROXY`, `ALL_PROXY`, and `NO_PROXY`, including lowercase forms. For Apple HTTPS requests, use `HTTPS_PROXY` or `ALL_PROXY`. Pass variables into Docker explicitly, for example `docker run -e HTTPS_PROXY -e NO_PROXY ...`. The proxy must be reachable from inside the container; `127.0.0.1` there is the container itself.

To check bundled binaries without contacting Apple or accessing device state:

```sh
docker run --rm --network none ghcr.io/fdkevin0/anisette-server-rs:main --backend android --check
```

Repeat with `macos` or `windows`. This checks binary loading, not provisioning or OTP generation. For those checks, use [real-backend validation](development.md#real-backend-validation).

### TLS and request limits

Provisioning uses reqwest with rustls, platform trust roots, and the bundled official Apple Root CA. Certificate and hostname verification remain enabled. Redirects are disabled and provisioning endpoints must be on `https://gsa.apple.com`.

Requests have a 10-second connect timeout, a 20-second total timeout, and a 1 MiB response limit. Errors identify lookup, start provisioning, or finish provisioning and report HTTP status or transport failure without logging device payloads. Device headers and CPIM are never written to command arguments, temporary files, or diagnostics.

## Local image builds

```sh
docker build -t anisette-server-rs:local .
```

Use `anisette-server-rs:local` in place of the GHCR image. Docker caches download layers; add `--no-cache` to fetch again. Builds download and verify pinned binaries over HTTPS; changed content or failed downloads stop the build. The Apple Music URL is mutable and the macOS binary is third-party hosted. See [asset sources](../assets/README.md) before changing downloads.

Installers, extraction tools, and compilers stay out of the runtime image. Bundled files are root-owned and not writable by the service. Builds create no device identity, and service startup never downloads binaries.

See [development](development.md#ci-and-containers) for image publishing and CI checks.
