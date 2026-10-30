# anisette-server-rs

A V1/V3 service providing Anisette headers for clients such as ipatool and AltServer. The Rust service executes pinned Apple ADI binaries through Unicorn; it requires no Python and does not handle account login or 2FA.

## Getting started

Build one image containing all three backends, then select one at startup:

```sh
docker build -t anisette-server-rs:local .
docker run --rm -p 127.0.0.1:6969:6969 \
  --mount type=volume,src=anisette-android-data,dst=/data \
  anisette-server-rs:local --backend android
```

Choose `android`, `macos`, or `windows` with `--backend`; no binary mount is needed.
Use a separate state volume for each backend and retain it on subsequent starts.
Unprovisioned devices are initialized automatically through Apple. See [Docker usage](docs/operations.md#docker).

### Running without Docker

Requires Rust 1.95, CMake, a C/C++ compiler, and libclang. Import the Apple binaries as described in [asset preparation](assets/README.md), then run:

```sh
cargo run --release -- --backend android \
  --binary ./assets/local/android-4.9.6 --state-dir ./data
```

The default listener is `127.0.0.1:6969`; clients retrieve V1 headers through `GET /`. The server automatically provisions a new device with Apple and reuses the saved device on subsequent starts. HTTPS requests use reqwest with standard proxy environment variables. Private persistence currently supports macOS/Linux. Apple ADI binaries are not committed to the repository. Docker builds download and bundle all three pinned backends in one Linux image. Select one with `--backend android|macos|windows`. See [Docker usage](docs/operations.md#docker).

**Before integrating:** V3 provisioning of new macOS/Windows devices requires the client to forward Apple routing information; Android standard V3 uses a default routing value. Generating an OTP does not establish that account login works. See [compatibility](docs/compatibility.md).

## Further reading

- Deployment, other backends, and Docker: [operations](docs/operations.md)
- Development, offline checks, and real-asset tests: [development and validation](docs/development.md)
- Message formats, resource limits, and the routing extension: [protocol](docs/protocol.md)
- Supported backends, hosts, and limitations: [compatibility](docs/compatibility.md)
- Provenance and GPL-2.0-only licensing: [third-party notices](THIRD_PARTY_NOTICES.md), [dependency inventory](docs/licenses.md)
