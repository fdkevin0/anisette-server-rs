# Sources and provenance

These pinned sources define protocol behavior and the code derivations used by the project. Full copyright and license notices are recorded in [THIRD_PARTY_NOTICES.md](../THIRD_PARTY_NOTICES.md).

| Source | Pinned commit | Use and license |
| --- | --- | --- |
| [Dadoum/anisette-v3-server](https://github.com/Dadoum/anisette-v3-server/tree/684895737aaf1a1c4586fcd5b67509ffd500a36c) | `684895737aaf1a1c4586fcd5b67509ffd500a36c` | V1/V3 protocol reference in source/app.d; no LICENSE/COPYING found in this tree; no implementation imported |
| [SideStore/apple-private-apis](https://github.com/SideStore/apple-private-apis/tree/03beb1aa42991ccdad6214dee77e72282bef461f) | `03beb1aa42991ccdad6214dee77e72282bef461f` | MPL-2.0; reference for V3 encoding and opaque client state storage |
| [malmeloo/Anisette.py](https://github.com/malmeloo/Anisette.py/tree/627c9c663062bd872182b653113759c9d66b7e13) | `627c9c663062bd872182b653113759c9d66b7e13` | MIT; Android VM/import behavior and RemoteADI client reference |
| [sacca97/os-findmy-e](https://github.com/sacca97/os-findmy-e/tree/d4d2a832baf47d1cf43d989879950ac2aec45f47) | `d4d2a832baf47d1cf43d989879950ac2aec45f47` | GPL-2.0-only; Mach-O parser, shared allocator, dispatcher framing, macOS daemon/CF/XPC/import models |

The service runs Rust and Unicorn without a Python runtime dependency. Android executes the pinned ARM64 Apple Music libraries; macOS executes the pinned x86_64 adid; Windows executes the pinned x86_64 CoreADI DLL. Distribution sources, versions, sizes, and hashes are in the [binary manifests](../assets/manifests/README.md). Apple binaries are not committed to the repository. Docker builds fetch the official Android and Windows packages and bundle their verified libraries. The Docker build also downloads adid from the backend author's [reference link](https://github.com/malmeloo/Anisette.py/pull/6) on ShareBin and enforces the pinned hash. ShareBin is not an Apple-operated host.

## Protocol references

The [Dadoum route implementation](https://github.com/Dadoum/anisette-v3-server/blob/684895737aaf1a1c4586fcd5b67509ffd500a36c/source/app.d) defines the V1/V3 message exchange. The [SideStore V3 client](https://github.com/SideStore/apple-private-apis/blob/03beb1aa42991ccdad6214dee77e72282bef461f/omnisette/src/remote_anisette_v3.rs) stores decoded adi_pb as opaque bytes and sends standard ptm/tk finish messages. Neither opaque storage nor a successful OTP establishes cross-backend state interchangeability or Apple account acceptance. The implemented contract is in [protocol](protocol.md).

The Windows HTTP profile used by Android and Windows follows the client pattern in [passwords-app's pinned client implementation](https://github.com/alex-mextner/passwords-app/commit/53c74a89a3b64d8e02eab5cc8b297dd22f5634cd). That client consumes external V1 fields and selects its own account-flow descriptions; it is not a Windows ADI executor. Caller-selected descriptions do not establish binary provenance.

## Platform and transport references

The Windows PE loader and bounded Win32 subset are local implementations. Argument passing follows the [Microsoft x64 ABI](https://learn.microsoft.com/en-us/cpp/build/x64-calling-convention?view=msvc-170), and character conversion follows [MultiByteToWideChar](https://learn.microsoft.com/en-us/windows/win32/api/stringapiset/nf-stringapiset-multibytetowidechar) within the pinned guest's supported subset. Native Windows differential coverage is limited as described in [compatibility](compatibility.md).

The service uses Hyper/Tokio/Tungstenite and the separate system curl program for Apple provisioning. Configuration escaping follows the [curl manual](https://curl.se/docs/manpage.html); HTTP connection budgets use the [Hyper Builder API](https://docs.rs/hyper/1.12.0/hyper/server/conn/http1/struct.Builder.html). Production, build, and development dependency scopes are documented in the [license inventory](licenses.md).
