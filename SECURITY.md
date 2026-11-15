# Security policy

Security fixes target the latest published release and `main`
(`ghcr.io/fdkevin0/anisette-server-rs:latest`). Upgrade before checking whether
an issue still reproduces.

## Reporting a vulnerability

Use
[GitHub private vulnerability reporting](https://github.com/fdkevin0/anisette-server-rs/security/advisories/new).
Do not disclose vulnerabilities in public issues or pull requests.

Relevant areas include Unicorn guest-to-host memory and sandbox safety,
ELF/Mach-O/PE loader bounds, HTTP and WebSocket resource exhaustion, and private
state storage permissions. Include the version or commit SHA, affected backend
(`android`, `macos`, or `windows`), reproduction steps, and potential impact.

Scrub any live `identity.json`, `adi.pb` / `adi_pb` state payloads,
`X-Apple-I-MD*` OTP headers, and Apple ID credentials from logs or proof of
concepts. Never share proprietary Apple binaries.

This project is maintained by one person; response times are best effort.
