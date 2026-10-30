# Compatibility

The service exposes V1 and V3 through Android, macOS, and Windows ADI backends. Backend names identify the Apple binary being executed, not the host operating system. Unicorn executes the guest code on the host.

| Backend | Pinned guest | V1 | V3 provisioning | State |
| --- | --- | --- | --- | --- |
| Android | Apple Music 4.9.6.1447, ARM64 ELF | Server provisions and saves Apple routing | Standard messages supported; routing defaults to 17106176 unless the client supplies Apple's value | Raw adi.pb |
| macOS | macOS 26.7.1 (25G241) x86_64 adid | Server provisions and saves Apple routing | Requires the Apple routing extension | macos-adid JSON envelope |
| Windows | iTunes 12.13.12.3, CoreADI 4.17.0.0 x86_64 DLL | Server provisions and saves Apple routing | Requires the Apple routing extension | windows-coreadi JSON envelope |

All four HTTP/WS routes are available for each backend. State is not interchangeable across backends. Only the exact binaries in the [manifests](../assets/manifests/README.md) are accepted.

## Host support

macOS ARM64 and Linux ARM64 support private state persistence and have real-backend coverage for provisioning, OTP, snapshot restoration, V1 restart, and the V3 behavior listed above. Windows-host private ACL storage is not implemented, so the server cannot run there. CI builds and offline tests on Windows do not establish Windows-host service support. Linux x86_64 and other host/architecture combinations lack real-backend validation.

The Windows backend executes through Unicorn without Wine or installed Windows software. It does not read the host registry or iCloud identity. The macOS and Android backends likewise use bounded emulated filesystems instead of host device state.

## Protocol and identity limitations

Standard V3 finish messages contain ptm/tk but no Apple routing header. The pinned macOS and Windows binaries return zero routing after these messages; a client must forward `X-Apple-I-MD-RINFO` from Apple's finish-provisioning response. An unmodified standard V3 client cannot provision new macOS/Windows devices. Android uses 17106176 when the field is absent; this fallback establishes protocol behavior, not Apple account acceptance. See the [protocol](protocol.md) for validation rules.

Android and Windows use the Windows HTTP client profile; macOS uses its macOS profile. A profile string does not change the ADI binary. V1 includes Client-Info and User-Agent, but clients may discard or override them. Callers are responsible for consistent identity selection across GSA, 2FA, loginDelegates, and CloudKit.

Generating OTPs does not establish successful Apple account login, 2FA, token issuance, or CloudKit access. These account flows are outside this service and are not validated by device-only tests.

## Emulator scope

Each backend implements only the ABI and system-function subset needed by its pinned binary. Unknown imports and unsupported operations fail explicitly. The Win32 character-conversion subset, including partial output on failure, lacks native Windows differential validation; it is not a general Win32 implementation.

Each native call is limited to 5 seconds or 50 million instructions. Failed emulator instances cannot be reused. State validation checks backend identity, format version, applicable binary hash, paths, and size limits. See [development](development.md) for executable checks and [operations](operations.md) for persistence and shutdown behavior.
