use super::*;
use crate::{
    BINARY_SHA256,
    filesystem::{FileSystem, valid_path},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
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
            return Err("state too large".into());
        }
        let state: Self = serde_json::from_slice(bytes).map_err(|_| "invalid Windows state")?;
        if state.provider != "windows-coreadi"
            || state.version != 1
            || state.binary_sha256 != BINARY_SHA256
            || state.identifier != identifier
        {
            return Err("state backend, binary, version or identity mismatch".into());
        }
        Ok(state)
    }
    fn filesystem(&self) -> Result<FileSystem, Error> {
        let mut fs = FileSystem::default();
        for (path, encoded) in &self.files {
            if !valid_path(path) {
                return Err("invalid state path".into());
            }
            let bytes = STANDARD
                .decode(encoded)
                .map_err(|_| "invalid state encoding")?;
            let fd = fs
                .open(path.clone(), 0xc0000000, 0, 1)
                .map_err(|_| "invalid state file")?;
            fs.write(fd, &bytes)?;
            fs.close(fd);
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
        if self.poisoned {
            return Err("failed runtime cannot export state".into());
        }
        let env = self.cpu.get_data();
        let state = State {
            provider: "windows-coreadi".into(),
            version: 1,
            binary_sha256: BINARY_SHA256.into(),
            identifier,
            host_uuid: env.host_uuid,
            files: env
                .fs
                .files
                .iter()
                .map(|(p, b)| (p.clone(), STANDARD.encode(b)))
                .collect(),
        };
        let bytes = serde_json::to_vec(&state).map_err(|_| "state serialization failed")?;
        if bytes.len() > MAX_STATE {
            return Err("state too large".into());
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
        for path in state.files.keys() {
            let mut parent = path.as_str();
            while let Some((p, _)) = parent.rsplit_once('\\') {
                runtime.cpu.get_data_mut().directories.insert(p.into());
                parent = p;
            }
        }
        if runtime.initialize()? < 0 {
            return Err("restored Windows initialization failed".into());
        }
        Ok(runtime)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn state_rejects_identity_version_paths_and_size() {
        let mut state = State {
            provider: "windows-coreadi".into(),
            version: 1,
            binary_sha256: BINARY_SHA256.into(),
            identifier: [1; 16],
            host_uuid: [2; 16],
            files: BTreeMap::new(),
        };
        assert!(State::decode(&serde_json::to_vec(&state).unwrap(), [0; 16]).is_err());
        state.version = 2;
        assert!(State::decode(&serde_json::to_vec(&state).unwrap(), [1; 16]).is_err());
        for path in [
            "C:\\Windows\\x.pb",
            "c:\\programdata\\apple computer\\itunes\\adi\\..\\x.pb",
        ] {
            state.files = BTreeMap::from([(path.into(), "AA==".into())]);
            assert!(state.filesystem().is_err());
        }
        state.files = BTreeMap::from([(
            format!("{}test.pb", crate::filesystem::ROOT),
            STANDARD.encode(vec![0; 1024 * 1024 + 1]),
        )]);
        assert!(state.filesystem().is_err());
    }
}
