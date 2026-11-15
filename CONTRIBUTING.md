# Contributing

Bug reports, documentation improvements, and focused pull requests are welcome.
For larger protocol, runtime, or architectural changes, open an issue first to
agree on the problem and scope.

All participation in this project is governed by the
[code of conduct](CODE_OF_CONDUCT.md). Report security vulnerabilities through
the [security policy](SECURITY.md) instead of public issues or pull requests.

## Development

Follow the [development guide](docs/development.md) for workspace layout and
prerequisites, and [asset preparation](assets/README.md) when working with local
Apple binaries. Before opening a pull request, run:

```sh
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo test --workspace --all-targets --all-features --locked
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --locked
```

Offline tests run without Apple binaries or network access. When modifying
ELF/Mach-O/PE loaders, Unicorn runtime hooks, or provisioning and routing
behavior, also run live backend validation (`sh scripts/verify-backends.sh`)
if you have the pinned Apple binaries prepared locally, and note any skipped
live checks in the pull request description.

## Hygiene and licensing

- Keep code, comments, commit messages, documentation, issues, and pull
  requests in English.
- Add regression tests for behavior or protocol changes, and update relevant
  files in [docs/](docs/development.md) when behavior, CLI flags, or client
  compatibility changes.
- **Never** commit or paste proprietary Apple binaries (`libstoreservicescore.so`,
  `CoreADI`, `CoreADI.dll`, APKs, or installers), live `identity.json` or
  `adi.pb` / `adi_pb` state blobs, live `X-Apple-I-MD*` OTP headers, or Apple ID
  credentials.
- By submitting a contribution, you agree that it is licensed under
  `GPL-2.0-only` as stated in [LICENSE](LICENSE).

## Pull requests

Create a branch from `main` and keep each pull request focused on one logical
change. Explain the problem, resulting behavior, and validation performed. All
changes to `main` go through a pull request with passing `check` CI and
resolved review discussions. A second person's approval is not required for this
single-maintainer project, and pull requests are squash merged.
