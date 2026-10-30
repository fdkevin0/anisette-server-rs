# Binary verification manifests

Each JSON file records one pinned Apple build, its source and exact hashes. Android ARM64, macOS x86_64 adid and Windows x86_64 CoreADI have all completed real device provisioning, OTP, snapshot restoration and V1/restart on macOS and Linux ARM64 hosts. No Apple binary is committed here.

Android standard V3 was tested with the documented default routing value; macOS/Windows require the explicit routing extension. Account acceptance and other host platforms are unverified. See [compatibility](../../docs/compatibility.md) for the complete scope and [asset preparation](../README.md) for import commands.
