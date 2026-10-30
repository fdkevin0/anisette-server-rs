# anisette-server-rs

A V1/V3 service providing Anisette headers for clients such as ipatool and AltServer. It executes Apple ADI binaries through Unicorn and does not handle Apple account login or 2FA.

## Quick start

Build and run the Android backend with Docker:

```sh
docker build -t anisette-server-rs:local .
docker run --rm -p 127.0.0.1:6969:6969 \
  --mount type=volume,src=anisette-android-data,dst=/data \
  anisette-server-rs:local --backend android
```

The service is available at `http://127.0.0.1:6969`; `GET /` returns V1 headers. First startup registers a device with Apple and requires network access. Keep the volume to reuse that identity on restart.

The image includes Android, macOS, and Windows backends. Each instance uses one backend and its own state volume. For V3 clients, check [backend compatibility](docs/compatibility.md#protocol-and-identity-limitations) before choosing macOS or Windows.

## Next steps

| Task | Guide |
| --- | --- |
| Use a published image, change backend, configure proxies or persistence | [Running and deployment](docs/operations.md) |
| Run without Docker | [Local setup](docs/operations.md#running-locally) |
| Choose a backend or check host support | [Compatibility](docs/compatibility.md) |
| Integrate a V1/V3 client | [Protocol reference](docs/protocol.md) |
| Change the code or validate a backend | [Development and validation](docs/development.md) |
| Check sources and licensing | [Provenance](docs/upstream.md), [third-party notices](THIRD_PARTY_NOTICES.md), [dependency licenses](docs/licenses.md) |
