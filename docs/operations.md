# Building, running, and validation

## Running locally

Rust 1.95.0 is pinned. CMake, a C/C++ compiler, and libclang are required to build Unicorn. The runtime uses Rust and Unicorn without Python; Apple device provisioning uses reqwest with rustls. Generating OTPs for provisioned devices requires no network requests. See [assets](../assets/README.md) for Apple file preparation. Private state storage currently supports Unix; Windows ACL support is not implemented.

```sh
cargo run --release -p anisette-server -- --backend android \
  --binary ./assets/local/android-4.9.6 --state-dir ./data
```

For macOS, use `--backend macos --binary /path/to/CoreADI-26.x86_64`; for Windows ADI, use `--backend windows --binary /path/to/CoreADI.dll`. Startup and V1 requests check the native provisioning status and automatically provision an unprovisioned device with Apple. Provisioning succeeds and is saved before serving headers. Existing state is always restored and is not overwritten if invalid. The state record requires version, backend, identifier, adi_pb, and a complete headers object containing device_id, local_user_id (64 hexadecimal characters), serial, client_info, and user_agent. Missing or invalid fields fail startup; they never trigger an automatic state reset. The parent directory must exist; the state directory is created with mode 0700 and files with 0600. Incorrect owners, symlinks, and files with multiple hard links are rejected. An exclusive directory lock prevents two services from sharing an identity.

The default listener is loopback. External listening requires an explicit `--listen`; public deployments use a reverse proxy for TLS and access control. The service accepts no account passwords or verification codes. SIGINT/SIGTERM stops acceptance, closes WS, waits for bounded execution and atomic saves already in progress, then releases the lock. Do not delete `identity.lock` while the service is running. Atomic saves use temporary-file fsync, rename, and directory fsync; save failure disables the V1 worker.

## Docker

One Linux image bundles all three pinned Apple backends. Android and Windows packages come from Apple; macOS adid comes from the [backend author's reference download](https://github.com/malmeloo/Anisette.py/pull/6) on ShareBin. Downloads use HTTPS and fixed SHA-256 verification. A failed download or changed file stops the build. Installers, extraction tools, and compilers stay out of the runtime image.

CI publishes one multi-platform image for `linux/amd64` and `linux/arm64` to GHCR after successful checks on `main` and version tags. Docker automatically selects the matching host architecture.

Run `ghcr.io/fdkevin0/anisette-server-rs:main` directly as shown below; no local build is required. `main` and `latest` track successful main builds. Version tags pin a release; `:0.1.0` was published for Linux AMD64 only.

Select the backend at runtime; no binary mount or `--binary` argument is required:

| Backend argument | Default binary path |
| --- | --- |
| `--backend android` (default) | `/assets` |
| `--backend macos` | `/assets/CoreADI-26.x86_64` |
| `--backend windows` | `/assets/CoreADI.dll` |

For example, initialize an independent macOS device:

```sh
docker volume create anisette-macos-data
docker run --rm --name anisette-macos -p 127.0.0.1:6969:6969 \
  --mount type=volume,src=anisette-macos-data,dst=/data \
  ghcr.io/fdkevin0/anisette-server-rs:main --backend macos
```

Use the same command and volume on subsequent starts to restore the saved device.

Use the same image with `--backend android` or `--backend windows` for the other backends. Each instance selects one backend. Run multiple containers with separate host ports and separate state volumes to serve multiple backends concurrently. Never share a state volume between backends or running instances.

The image sets `ANISETTE_LISTEN=0.0.0.0:6969` and `ANISETTE_STATE_DIR=/data`, so these settings remain in effect when container arguments are supplied. Explicit `--listen` and `--state-dir` override them. Outside Docker, defaults are loopback and `data` unless these environment variables are set. `--binary` overrides the backend's bundled path for locally supplied files.

The service runs as UID/GID 10001. Bundled files are root-owned and not writable by the service. A new named volume inherits private `/data` permissions; an existing bind mount must belong to UID 10001 with mode 0700. No device identity is created during the build. A new device is provisioned automatically at startup; subsequent starts restore saved state.

### Proxies and offline checks

Provisioning honors `HTTPS_PROXY`, `HTTP_PROXY`, `ALL_PROXY`, and `NO_PROXY` (including lowercase forms). For Apple HTTPS requests, use `HTTPS_PROXY` or `ALL_PROXY`. Pass variables into Docker explicitly, for example `docker run -e HTTPS_PROXY -e NO_PROXY ...`. The proxy address must be reachable from inside the container; `127.0.0.1` there is the container itself.

TLS certificates are verified using the platform trust store plus the bundled official Apple Root CA required by the provisioning service. Requests have a 10-second connect timeout, a 20-second total timeout, and a 1 MiB response limit. Redirects are disabled and provisioning endpoints must be on `https://gsa.apple.com`. Errors identify lookup, start provisioning, or finish provisioning and report HTTP status or the transport failure category without logging device payloads.

To verify bundled binaries without contacting Apple or accessing device state:

```sh
docker run --rm --network none ghcr.io/fdkevin0/anisette-server-rs:main --backend android --check
```

Repeat with `macos` or `windows` for the other backends. Normal startup of an unprovisioned device requires Apple network access.

### Local image builds

To build a local image instead:

```sh
docker build -t anisette-server-rs:local .
```

Use `anisette-server-rs:local` in place of the GHCR image in the run commands. Docker reuses cached download layers. Use `docker build --no-cache -t anisette-server-rs:local .` to fetch again. The Apple Music URL is mutable, and the macOS download is third-party hosted; changed content is rejected by its fixed hash. Backend names describe the guest binaries, not the Linux host. Host architecture coverage and the macOS/Windows V3 routing requirement remain as documented in [compatibility](compatibility.md).

See [development](development.md) for checks and [protocol](protocol.md) for message formats.
