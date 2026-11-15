## Problem and change

<!-- Explain the problem and resulting behavior. Link an issue if relevant. -->

## Validation

<!-- List checks performed and their results, including any checks not run. -->

- [ ] `cargo fmt --all -- --check`
- [ ] `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- [ ] `cargo test --workspace --all-targets --all-features --locked`
- [ ] `RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps --all-features --locked`
- [ ] Live backend verification (`scripts/verify-backends.sh` or targeted `--ignored` tests, when changing ADI runtimes, loaders, or provisioning)
- [ ] Documentation updated (`README.md` / `docs/`, if behavior, flags, or compatibility changed)
