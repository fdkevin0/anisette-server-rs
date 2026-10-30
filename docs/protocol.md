# V1/V3 contract

Based on the [pinned Dadoum app.d](https://github.com/Dadoum/anisette-v3-server/blob/684895737aaf1a1c4586fcd5b67509ffd500a36c/source/app.d) and [pinned SideStore V3 client](https://github.com/SideStore/apple-private-apis/blob/03beb1aa42991ccdad6214dee77e72282bef461f/omnisette/src/remote_anisette_v3.rs). All four routes are connected to all three backends. Android supports standard V3 messages; provisioning new macOS/Windows devices is subject to the routing extension below.

| Endpoint | Input | Output |
| --- | --- | --- |
| GET / | None | V1 headers object, without result |
| GET /v3/client_info | None | client_info and user_agent strings |
| POST /v3/get_headers | identifier and adi_pb strings | result=Headers + X-Apple-I-MD, X-Apple-I-MD-M, X-Apple-I-MD-RINFO (all strings) |
| WS /v3/provisioning_session | Handshake below | Handshake below |

```text
server {"result":"GiveIdentifier"}
client {"identifier":"<Base64 of 16 bytes>"}
server {"result":"GiveStartProvisioningData"}
client {"spim":"<Base64>"}
server {"result":"GiveEndProvisioningData","cpim":"<Base64>"}
client {"ptm":"<Base64>","tk":"<Base64>"}
server {"result":"ProvisioningSuccess","adi_pb":"<Base64>"}
server closes connection
```

identifier uses standard Base64. UUIDs group bytes in their original order, without Windows GUID little-endian field swapping. The Android ADI identifier is the first 16 characters of the uppercase, hyphenated UUID string. DSID is i64 -2; its bit pattern is 18446744073709551614 when the ABI requires u64.

The SideStore client decodes adi_pb and stores its bytes in state.plist, then Base64-encodes them for the next request without parsing the internal format. This supports opaque state transport but does not prove that state is interchangeable across backends. The client's finish message contains only ptm/tk and does not forward the Apple routing header; a successful get_headers response does not update adi_pb either.

Exact V1 keys: X-Apple-I-Client-Time, X-Apple-I-MD, X-Apple-I-MD-M, X-Apple-I-MD-RINFO, X-Apple-I-MD-LU, X-Apple-I-SRL-NO, X-MMe-Client-Info, X-Apple-I-TimeZone, X-Apple-Locale, X-Mme-Device-Id. User-Agent is an additional field for clients that use the service-provided HTTP profile.

Errors: get_headers application errors use JSON `result=GetHeadersError` + message; the reference implementation does not set a non-200 status. WS results are InvalidIdentifier, Timeout (without message), StartProvisioningError, and EndProvisioningError (with message), followed by connection closure. The reference waits 3000ms for each client message by default; this is not an ADI execution timeout. Loopback WebSocket tests cover this project's close behavior.

## Validation and resource limits

- identifier must be exactly 16 bytes. Dadoum ignores extra bytes; this project rejects them.
- Standard padded Base64 is required; URL-safe encoding, whitespace, and missing padding are rejected. State/binary fields must not be empty.
- HTTP bodies and WS messages/frames are limited to 4 MiB + 4 KiB (reserving room for JSON and protocol fields beyond the Base64 state); decoded state is limited to 3 MiB. Oversized HTTP requests return 413; exhausted capacity returns 503.
- get_headers input/backend errors return HTTP 200 + GetHeadersError; an unavailable V1 worker returns 503.
- All HTTP responses set Cache-Control: no-store and Implementation-Version: anisette-server-rs/version.
- WS waits 3 seconds for the client at each stage; Ping does not extend the stage deadline. Sending has a 3-second deadline. Shutdown, disconnection, and errors release the session; close frames use the default normal closure.
- Limits are 64 TCP connections (10-second header timeout), 32 in-flight HTTP requests, 4 V3 OTP workers, and 4 provisioning sessions, with no unbounded queue. V1 uses one worker with a queue of 4. Emulator instances stay on their creating thread; each native call is limited to 5 seconds/50 million instructions, and failed instances cannot be reused.
- HTTP waits at most 40 seconds and worker replies at most 35 seconds. A timeout does not release capacity while a worker is still executing. Shutdown notifications close WS and wait for execution/state saves to finish; HTTP connections close after at most 45 seconds, while background execution capacity remains held until actually reclaimed.
- V1 uses a separate server identity. V3 restores opaque client state and does not share the V1 identity.

## macOS/Windows routing extension (nonstandard V3)

In addition to ptm/tk, the finish message may include `X-Apple-I-MD-RINFO`, containing the positive u64 integer or decimal string from Apple's finish-provisioning response. Negative numbers, 0, null, floats, overflow, and nondecimal strings are rejected. The field is optional in the protocol type; macOS/Windows finish with EndProvisioningError if it is absent and ADI itself returns 0.

Real tests confirm that the pinned macOS and Windows builds do not obtain nonzero routing from standard ptm/tk. Consequently, unmodified standard V3 clients cannot provision new devices; extension tests must not be described as standard V3 compatibility. get_headers also rejects state without valid routing.

V1 performs the Apple exchange on the server and saves routing, requiring no client changes. Clients may use the additional User-Agent field or select their own profile.

## Android standard V3 and state formats

Android's standard ptm/tk message carries no Apple routing information. Like Dadoum, this project uses the default `17106176` and accepts the same optional routing field to save Apple's actual value. This is a protocol compatibility fallback, not proof of account acceptance. All three V1 backends save actual Apple routing information.

Android adi_pb is the raw file; macOS uses `macos-adid` JSON and Windows uses `windows-coreadi` JSON, all stored opaquely by the client. State from another backend, unknown versions, incorrect identifiers, and invalid virtual paths are rejected.

New V1 identities derive the device UUID and SHA-256 local-user header from identifier. V1 persists the complete header identity together with ADI state and preserves it across restarts. Clients remain responsible for identity selection in GSA/2FA/CloudKit.
