# Third-party notices

No Apple binaries are committed to the source repository. Docker builds download and bundle the verified Android and Windows libraries from Apple packages; the same image also bundles the backend author-provided adid. These files retain their own terms. The macOS parser includes adapted upstream code as documented below. Protocol behavior was checked against the commits in docs/upstream.md.

Repository source is GPL-2.0-only (see LICENSE); workspace crates are publish=false. The dependency inventory and distribution requirements are recorded in [docs/licenses.md](docs/licenses.md). The macOS emulator source explicitly specifies GPL-2.0-only; its adapted code retains that license.

Current Rust dependencies are recorded exactly in Cargo.lock. base64: MIT OR Apache-2.0; serde/serde_json: MIT OR Apache-2.0; clap: MIT OR Apache-2.0 (check transitive package notices before distribution). Apple files remain subject to their own terms and are not covered by any repository license.

## macOS backend

`crates/adi-macos/src/macho.rs` is adapted from sacca97/os-findmy-e at `d4d2a832baf47d1cf43d989879950ac2aec45f47`, native/adi-core/src/macho.rs, with executable parsing and bounds validation changes. Copyright (c) 2026 sacca. The adi-macos crate is GPL-2.0-only; the full license is at crates/adi-macos/LICENSE. sha2 is MIT OR Apache-2.0; choose its MIT alternative for GPL-2.0-only distribution. Any binary distribution must include the corresponding source and applicable notices.

The memory allocator/runtime layout and daemon startup import models in `anisette/src/memory.rs` and `adi-macos/src/runtime.rs` are adapted from the same fixed os-findmy-e macOS-ADI-emulator `harness.py` and `adi.py`, GPL-2.0-only, copyright (c) 2026 sacca. Execution limits, fail-closed import handling and allocator checks are local implementations. `unicorn-engine`/`unicorn-engine-sys` 2.1.5 are GPL-2.0; getrandom 0.4 uses its MIT alternative. The Docker build downloads the executable from the backend author's reference link and verifies its pinned hash. Apple executable bytes are not covered by the emulator's GPL license.

`adi-macos/src/{filesystem,stubs,xpc}.rs` additionally adapt the same reference's `stubs.py` and `adi.py`, retaining GPL-2.0-only. Bounds, filesystem limits, input validation, error propagation and strict unsupported-operation handling are local changes.

`adi-macos/src/adi.rs` adapts the same GPL-2.0-only ADI wire-operation reference. The service performs Apple provisioning through a separate system curl executable, using plist/time in Rust. The production HTTP service uses Hyper/Tokio/Tungstenite. The reqwest integration-test client and its TLS dependencies (including AWS-LC in reqwest 0.13) remain development-only; those test executables are not release artifacts. Build-only clang-sys is also recorded in docs/licenses.md. The importer enables only the required ZIP decoder.

## Android and shared code

`adi-android/src/runtime.rs` implements the fixed ELF runtime/import behavior using malmeloo/Anisette.py at `627c9c663062bd872182b653113759c9d66b7e13`, especially `src/anisette/vm/_vm.py` and `_hooks.py`. Its MIT copyright and permission notice is retained in `crates/adi-android/LICENSE-MIT-Anisette.py`. The dispatcher frame also follows the GPL-2.0-only os-findmy-e native ADI reference; the combined crate retains GPL-2.0-only at `crates/adi-android/LICENSE`.

`anisette/src/memory.rs` retains the GPL-2.0-only attribution documented above; the shared crate has a full LICENSE copy. `goblin` and `zip` are MIT.

The lockfile includes target-specific, optional, build and example dependencies as well as linked runtime dependencies. Their presence alone does not establish what a specific binary distributes. No release compatibility conclusion is asserted by the SPDX summaries above.

## Windows backend

`adi-windows` uses locally authored PE loading and a bounded Win64 API subset, plus the GPL-2.0-only os-findmy-e dispatcher framing and shared allocator. Its full GPL-2.0-only license is retained. Apple DLL bytes are extracted from the official iTunes installer and bundled by the Docker build. No Wine, Perun, Blackwood or unlicensed apple-crates implementation code is included.
