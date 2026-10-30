# Apple binary assets

Apple files are not committed to Git; local copies under `assets/local/` are ignored. Docker builds automatically download the pinned Android APK and Windows iTunes installer, verify and extract them with the Rust importers, and bundle only the required libraries. Service startup never downloads or updates files. All three loaders enforce exact SHA-256 hashes before parsing.

The image contains Android libraries in `/assets` and Windows CoreADI at `/assets/CoreADI.dll`. The Docker build also downloads the pinned adid to `/assets/CoreADI-26.x86_64` for selection with `--backend macos`. See [Docker usage](../docs/operations.md#docker); the commands below are for local non-Docker preparation.

## Android

The pinned build is Apple Music **4.9.6.1447**, ARM64 (`arm64-v8a`), recorded in [the manifest](manifests/android-4.9.6.json). The official URL is mutable; a different release must fail verification. Download explicitly with a bounded HTTPS-only command, then import on Linux/macOS:

```sh
mkdir -p assets/local
curl --fail --location --proto '=https' --proto-redir '=https' \
  --max-time 180 --max-filesize 150000000 \
  --output assets/local/applemusic.apk.part \
  https://apps.mzstatic.com/content/android-apple-music-apk/applemusic.apk
cargo run -p adi-android --example import_apk -- \
  assets/local/applemusic.apk.part assets/local/android-4.9.6
```

The importer requires a new destination. It verifies the entire APK before ZIP parsing, reads only the two fixed ARM64 entries, bounds and verifies their contents, checks ELF parsing, then publishes the temporary directory atomically. A failed download remains an unusable `.part` file; a failed import does not replace installed libraries. Windows directory publication is not supported by this preparation command.

The imported libraries have passed device provisioning, OTP, repeated snapshot restoration and HTTP service tests on macOS arm64. Their CPU architecture is independent of the host: Unicorn executes ARM64 guest code.

## macOS

The x86_64 adid slice is pinned to macOS 26.7.1 (25G241), SHA-256 `4b6b320f08edcf5d65ed1141399b76a8d9323fee2045c577c5d2f30f35a5dae1`. See [manifest](manifests/macos-26.json). This exact binary has passed real provisioning/OTP and service tests. No other build is accepted.

The [backend author links this reference file](https://github.com/malmeloo/Anisette.py/pull/6) on ShareBin. This is a third-party host, not an Apple download endpoint. The download page requires an empty POST; GET returns HTML. The Docker build performs the download and verifies the pinned SHA-256 before including it. For local preparation:

```sh
mkdir -p assets/local
curl --disable --fail --location --proto '=https' --proto-redir '=https' \
  --connect-timeout 30 --max-time 120 --max-filesize 3677696 --data '' \
  --output assets/local/CoreADI-26.x86_64 https://sharebin.eu/files/CoreADI.x86_64
cargo run -p adi-macos --example inspect -- assets/local/CoreADI-26.x86_64
```

A file extracted from a matching system is also accepted if it has the same hash.

## Windows

The official iTunes 12.13.12.3 installer contains the pinned x86_64 CoreADI 4.17 DLL; see [manifest](manifests/windows-itunes-12.13.12.3.json). Real provisioning, OTP, repeated snapshot restoration and V1/extended-V3 service tests pass on the macOS ARM64 host using Unicorn; Wine is not used. No third-party DLL download site is used. See [sources and provenance](../docs/upstream.md).

Install `7z` as a preparation-only tool. The Rust importer verifies installer size/hash before extraction, copies the verified bytes to a private temporary directory, extracts only the fixed MSI and DLL entries with size/time limits, verifies the MSI and DLL hashes, and atomically creates a new output file. Existing output is never replaced.

```sh
curl --fail --location --proto '=https' --proto-redir '=https' \
  --max-time 180 --max-filesize 210000000 \
  --output assets/local/iTunes64Setup.exe.part \
  https://secure-appldnld.apple.com/itunes12/142-28289-20261026-6acff6c8-72eb-4045-84eb-fd35239ac645/iTunes64Setup.exe
cargo run -p adi-windows --example import_itunes -- \
  assets/local/iTunes64Setup.exe.part assets/local/CoreADI.dll
```

Neither the installer nor 7z is executed by the service. The Windows installer itself is never executed.
