use crate::{BINARY_SHA256, Binary, Error, Runtime, filesystem::FileSystem};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
const MAX_STATE: usize = 3 * 1024 * 1024;
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct State {
    provider: String,
    version: u32,
    binary_sha256: String,
    identifier: [u8; 16],
    host_uuid: [u8; 16],
    files: BTreeMap<String, String>,
}
impl State {
    fn decode(bytes: &[u8], identifier: [u8; 16]) -> Result<Self, Error> {
        if bytes.len() > MAX_STATE {
            return Err(Error::Execution("state too large"));
        }
        let state: Self =
            serde_json::from_slice(bytes).map_err(|_| Error::Execution("invalid macOS state"))?;
        if state.provider != "macos-adid"
            || state.version != 1
            || state.binary_sha256 != BINARY_SHA256
            || state.identifier != identifier
        {
            return Err(Error::Execution(
                "state backend, version, binary or identity mismatch",
            ));
        }
        Ok(state)
    }
    fn filesystem(&self) -> Result<FileSystem, Error> {
        if self.files.len() > 64 {
            return Err(Error::Execution("too many state files"));
        }
        let mut fs = FileSystem::default();
        for (path, encoded) in &self.files {
            if !FileSystem::valid_path(path) || !path.starts_with("/var/db/fpsd/adi/") {
                return Err(Error::Execution("invalid state path"));
            }
            let bytes = STANDARD
                .decode(encoded)
                .map_err(|_| Error::Execution("invalid state encoding"))?;
            let fd = fs
                .open(path.clone(), 0x202)
                .map_err(|_| Error::Execution("invalid state file"))?;
            fs.write(fd, &bytes)
                .map_err(|_| Error::Execution("state file exceeds limit"))?;
            fs.close(fd)
                .map_err(|_| Error::Execution("state close failed"))?;
        }
        Ok(fs)
    }
}
impl Runtime {
    /// Exports the device state for later restoration.
    ///
    /// # Errors
    /// Returns an error if the runtime is poisoned, serialization fails, or the snapshot exceeds the state limit.
    pub fn export_state(&self, identifier: [u8; 16]) -> Result<Vec<u8>, Error> {
        self.ensure_usable()?;
        let state = State {
            provider: "macos-adid".into(),
            version: 1,
            binary_sha256: BINARY_SHA256.into(),
            identifier,
            host_uuid: self.cpu.get_data().host_uuid,
            files: self
                .cpu
                .get_data()
                .fs
                .files
                .iter()
                .filter(|(p, _)| p.starts_with("/var/db/fpsd/adi/"))
                .map(|(p, b)| (p.clone(), STANDARD.encode(b)))
                .collect(),
        };
        let bytes = serde_json::to_vec(&state)
            .map_err(|_| Error::Execution("state serialization failed"))?;
        if bytes.len() > MAX_STATE {
            return Err(Error::Execution("state too large"));
        }
        Ok(bytes)
    }
    /// Restores a device from a backend-specific state snapshot.
    ///
    /// # Errors
    /// Returns an error for invalid state size, identity, version, binary hash, or files, or if backend initialization fails.
    pub fn restore(binary: &Binary, identifier: [u8; 16], bytes: &[u8]) -> Result<Self, Error> {
        let state = State::decode(bytes, identifier)?;
        let fs = state.filesystem()?;
        let mut runtime = Self::new(binary)?;
        runtime.cpu.get_data_mut().host_uuid = state.host_uuid;
        runtime.cpu.get_data_mut().fs = fs;
        runtime.boot()?;
        Ok(runtime)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    fn sample() -> State {
        State {
            provider: "macos-adid".into(),
            version: 1,
            binary_sha256: BINARY_SHA256.into(),
            identifier: [1; 16],
            host_uuid: [2; 16],
            files: BTreeMap::new(),
        }
    }
    #[test]
    fn rejects_wrong_identity_backend_version_and_paths() {
        let mut state = sample();
        let bytes = serde_json::to_vec(&state).unwrap();
        assert!(State::decode(&bytes, [0; 16]).is_err());
        state.provider = "android".into();
        assert!(State::decode(&serde_json::to_vec(&state).unwrap(), [1; 16]).is_err());
        state = sample();
        state.version = 2;
        assert!(State::decode(&serde_json::to_vec(&state).unwrap(), [1; 16]).is_err());
        for path in [
            "/etc/passwd",
            "/var/db/fpsd/adi/../x",
            "/var/db/fpsd/adi//x",
        ] {
            let mut state = sample();
            state.files.insert(path.into(), "AA==".into());
            assert!(state.filesystem().is_err());
        }
    }
    #[test]
    fn opaque_bytes_roundtrip_and_file_limits() {
        let mut state = sample();
        state.files.insert(
            "/var/db/fpsd/adi/test".into(),
            STANDARD.encode([0, 255, 0, 128]),
        );
        let encoded = serde_json::to_vec(&state).unwrap();
        let parsed = State::decode(&encoded, [1; 16]).unwrap();
        assert_eq!(
            parsed.filesystem().unwrap().files["/var/db/fpsd/adi/test"],
            [0, 255, 0, 128]
        );
        state.files.insert(
            "/var/db/fpsd/adi/large".into(),
            STANDARD.encode(vec![0; 1024 * 1024 + 1]),
        );
        assert!(state.filesystem().is_err());
    }
}
