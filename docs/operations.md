# Building, running, and validation

## Running locally

Rust 1.95.0 is pinned. CMake, a C/C++ compiler, and libclang are required to build Unicorn. The runtime uses Rust and Unicorn without Python; Apple device provisioning requires curl in PATH (with HTTPS/TLS 1.2 support, installed in Docker). OTP generation for existing devices does not invoke curl. See [assets](../assets/README.md) for Apple file preparation. Private state storage currently supports Unix; Windows ACL support is not implemented.

```sh
cargo run --release -p anisette-server -- --backend android \
  --binary ./assets/local/android-4.9.6 --state-dir ./data --initialize
```

For macOS, use `--backend macos --binary /path/to/CoreADI-26.x86_64`; for Windows ADI, use `--backend windows --binary /path/to/CoreADI.dll`. The initial `--initialize` explicitly contacts Apple to register a new V1 device and can be omitted afterward. Existing state is always restored and is not overwritten if invalid. The state record requires version, backend, identifier, adi_pb, and a complete headers object containing device_id, local_user_id (64 hexadecimal characters), serial, client_info, and user_agent. Missing or invalid fields fail startup, including with `--initialize`. The parent directory must exist; the state directory is created with mode 0700 and files with 0600. Incorrect owners, symlinks, and files with multiple hard links are rejected. An exclusive directory lock prevents two services from sharing an identity.

The default listener is loopback. External listening requires an explicit `--listen`; public deployments use a reverse proxy for TLS and access control. The service accepts no account passwords or verification codes. SIGINT/SIGTERM stops acceptance, closes WS, waits for bounded execution and atomic saves already in progress, then releases the lock. Do not delete `identity.lock` while the service is running. Atomic saves use temporary-file fsync, rename, and directory fsync; save failure disables the V1 worker.

## Docker

One Linux image bundles all three pinned Apple backends. Android and Windows packages come from Apple; macOS adid comes from the [backend author's reference download](https://github.com/malmeloo/Anisette.py/pull/6) on ShareBin. Downloads use HTTPS and fixed SHA-256 verification. A failed download or changed file stops the build. Installers, extraction tools, and compilers stay out of the runtime image.

CI publishes the Linux AMD64 image to GHCR after successful checks on `main` and version tags:

```sh
docker pull ghcr.io/fdkevin0/anisette-server-rs:latest
```

`latest` and `main` track successful main builds. Use a version such as `:0.1.0` to pin a release.

Use that image name in the commands below, or build locally:

```sh
docker build -t anisette-server-rs:local .
```

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
  anisette-server-rs:local --backend macos --initialize
```

On subsequent starts, omit `--initialize` and retain the same volume:

```sh
docker run --rm --name anisette-macos -p 127.0.0.1:6969:6969 \
  --mount type=volume,src=anisette-macos-data,dst=/data \
  anisette-server-rs:local --backend macos
```

Use the same image with `--backend android` or `--backend windows` for the other backends. Each instance selects one backend. Run multiple containers with separate host ports and separate state volumes to serve multiple backends concurrently. Never share a state volume between backends or running instances.

The image sets `ANISETTE_LISTEN=0.0.0.0:6969` and `ANISETTE_STATE_DIR=/data`, so these settings remain in effect when container arguments are supplied. Explicit `--listen` and `--state-dir` override them. Outside Docker, defaults are loopback and `data` unless these environment variables are set. `--binary` overrides the backend's bundled path for locally supplied files.

The service runs as UID/GID 10001. Bundled files are root-owned and not writable by the service. A new named volume inherits private `/data` permissions; an existing bind mount must belong to UID 10001 with mode 0700. No device identity is created during the build. Initialization explicitly contacts Apple; subsequent starts restore saved state.

Docker reuses cached download layers. Use `docker build --no-cache -t anisette-server-rs:local .` to fetch again. The Apple Music URL is mutable, and the macOS download is third-party hosted; changed content is rejected by its fixed hash. Backend names describe the guest binaries, not the Linux host. Host architecture coverage and the macOS/Windows V3 routing requirement remain as documented in [compatibility](compatibility.md).

See [development](development.md) for checks and [protocol](protocol.md) for message formats.
