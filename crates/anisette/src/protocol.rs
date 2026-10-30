use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use std::fmt;

pub const DSID: i64 = -2;
pub const MAX_STATE_BYTES: usize = 3 * 1024 * 1024;
/// Allows a maximum Base64 state plus 4 KiB for JSON and protocol fields.
pub const MAX_MESSAGE_BYTES: usize = MAX_STATE_BYTES.div_ceil(3) * 4 + 4096;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputError {
    TooLarge,
    InvalidJson,
    InvalidBase64,
    InvalidIdentifier,
    EmptyData,
}

impl fmt::Display for InputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::TooLarge => "input exceeds size limit",
            Self::InvalidJson => "invalid JSON message",
            Self::InvalidBase64 => "invalid standard Base64",
            Self::InvalidIdentifier => "identifier must contain exactly 16 bytes",
            Self::EmptyData => "binary field must not be empty",
        })
    }
}
impl std::error::Error for InputError {}

/// Error messages deliberately exclude request contents and decoder diagnostics.
///
/// # Errors
/// Returns an error if the message exceeds the wire limit or cannot deserialize into `T`.
pub fn parse_message<'a, T: Deserialize<'a>>(bytes: &'a [u8]) -> Result<T, InputError> {
    if bytes.len() > MAX_MESSAGE_BYTES {
        return Err(InputError::TooLarge);
    }
    serde_json::from_slice(bytes).map_err(|_| InputError::InvalidJson)
}

/// Decodes a nonempty standard Base64 field within its binary size limit.
///
/// # Errors
/// Returns an error for invalid Base64, empty data, or encoded or decoded data exceeding the limit.
pub fn decode_binary(encoded: &str, max_bytes: usize) -> Result<Vec<u8>, InputError> {
    if encoded.len() > max_bytes.div_ceil(3).saturating_mul(4) {
        return Err(InputError::TooLarge);
    }
    let bytes = STANDARD
        .decode(encoded)
        .map_err(|_| InputError::InvalidBase64)?;
    if bytes.len() > max_bytes {
        return Err(InputError::TooLarge);
    }
    if bytes.is_empty() {
        return Err(InputError::EmptyData);
    }
    Ok(bytes)
}

// No Debug: avoid accidentally logging device identities.
#[derive(Clone, PartialEq, Eq)]
pub struct Identifier([u8; 16]);

impl Identifier {
    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    /// Parses the canonical hyphenated UUID representation without changing byte order.
    ///
    /// # Errors
    /// Returns an error unless the input contains 32 ASCII hex digits in the canonical UUID layout.
    pub fn from_uuid(value: &str) -> Result<Self, InputError> {
        if value.len() != 36 || !value.is_ascii() {
            return Err(InputError::InvalidIdentifier);
        }
        let mut bytes = [0; 16];
        let mut source = value.bytes();
        for (i, byte) in bytes.iter_mut().enumerate() {
            if [4, 6, 8, 10].contains(&i) && source.next() != Some(b'-') {
                return Err(InputError::InvalidIdentifier);
            }
            let hi = source
                .next()
                .and_then(|c| (c as char).to_digit(16))
                .ok_or(InputError::InvalidIdentifier)?;
            let lo = source
                .next()
                .and_then(|c| (c as char).to_digit(16))
                .ok_or(InputError::InvalidIdentifier)?;
            *byte = ((hi << 4) | lo) as u8;
        }
        Ok(Self(bytes))
    }

    pub fn bytes(&self) -> [u8; 16] {
        self.0
    }
    /// Decodes an identifier from its standard Base64 representation.
    ///
    /// # Errors
    /// Returns an error unless standard Base64 decodes to exactly 16 bytes.
    pub fn decode(encoded: &str) -> Result<Self, InputError> {
        let bytes = decode_binary(encoded, 16).map_err(|_| InputError::InvalidIdentifier)?;
        Ok(Self(
            bytes
                .try_into()
                .map_err(|_| InputError::InvalidIdentifier)?,
        ))
    }

    pub fn uuid(&self) -> String {
        let b = self.0;
        format!(
            "{:02X}{:02X}{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}-{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}",
            b[0],
            b[1],
            b[2],
            b[3],
            b[4],
            b[5],
            b[6],
            b[7],
            b[8],
            b[9],
            b[10],
            b[11],
            b[12],
            b[13],
            b[14],
            b[15]
        )
    }

    /// Dadoum's Android mapping; not a mapping for macOS or Windows identities.
    pub fn android_adi_identifier(&self) -> String {
        self.uuid()[..16].to_owned()
    }
}

#[derive(Deserialize)]
pub struct IdentifierMessage {
    pub identifier: String,
}
#[derive(Deserialize)]
pub struct StartMessage {
    pub spim: String,
}
#[derive(Deserialize)]
pub struct EndMessage {
    pub ptm: String,
    pub tk: String,
    /// Explicit macOS/Windows extension; ordinary V3 clients do not send this field.
    #[serde(default, rename = "X-Apple-I-MD-RINFO", deserialize_with = "routing")]
    pub routing: Option<u64>,
}

fn routing<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Option<u64>, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Number {
        Integer(u64),
        Decimal(String),
    }
    let value = match Number::deserialize(d)? {
        Number::Integer(n) => n,
        Number::Decimal(s) if !s.is_empty() && s.bytes().all(|c| c.is_ascii_digit()) => {
            s.parse().map_err(serde::de::Error::custom)?
        }
        _ => return Err(serde::de::Error::custom("invalid routing")),
    };
    if value == 0 {
        return Err(serde::de::Error::custom("zero routing"));
    }
    Ok(Some(value))
}
#[derive(Deserialize)]
pub struct HeadersRequest {
    pub identifier: String,
    pub adi_pb: String,
}

impl HeadersRequest {
    /// Decodes the identifier and opaque state from a header request.
    ///
    /// # Errors
    /// Returns an error for malformed Base64, an identifier other than 16 bytes, or empty or oversized state.
    pub fn decode(&self) -> Result<(Identifier, Vec<u8>), InputError> {
        Ok((
            Identifier::decode(&self.identifier)?,
            decode_binary(&self.adi_pb, MAX_STATE_BYTES)?,
        ))
    }
}

#[derive(Serialize)]
pub struct ClientInfo<'a> {
    pub client_info: &'a str,
    pub user_agent: &'a str,
}

#[derive(Serialize)]
#[serde(tag = "result")]
pub enum ServerMessage<'a> {
    GiveIdentifier,
    GiveStartProvisioningData,
    GiveEndProvisioningData {
        cpim: &'a str,
    },
    ProvisioningSuccess {
        adi_pb: &'a str,
    },
    InvalidIdentifier,
    Timeout,
    StartProvisioningError {
        message: &'a str,
    },
    EndProvisioningError {
        message: &'a str,
    },
    GetHeadersError {
        message: &'a str,
    },
    Headers {
        #[serde(rename = "X-Apple-I-MD")]
        otp: &'a str,
        #[serde(rename = "X-Apple-I-MD-M")]
        machine_id: &'a str,
        #[serde(rename = "X-Apple-I-MD-RINFO")]
        routing: &'a str,
    },
}

/// V1 names preserve the exact JSON casing consumed by existing clients.
/// Identity/profile values must come from the actual backend, never the host OS.
#[derive(Serialize)]
pub struct V1Headers<'a> {
    #[serde(rename = "X-Apple-I-MD")]
    pub otp: &'a str,
    #[serde(rename = "X-Apple-I-MD-M")]
    pub machine_id: &'a str,
    #[serde(rename = "X-Apple-I-MD-RINFO")]
    pub routing: &'a str,
    #[serde(rename = "X-Apple-I-MD-LU")]
    pub local_user_id: &'a str,
    #[serde(rename = "X-Apple-I-SRL-NO")]
    pub serial: &'a str,
    #[serde(rename = "X-Mme-Device-Id")]
    pub device_id: &'a str,
    #[serde(rename = "X-Apple-I-Client-Time")]
    pub client_time: &'a str,
    #[serde(rename = "X-Apple-I-TimeZone")]
    pub timezone: &'a str,
    #[serde(rename = "X-Apple-Locale")]
    pub locale: &'a str,
    #[serde(rename = "X-MMe-Client-Info")]
    pub client_info: &'a str,
    #[serde(rename = "User-Agent")]
    pub user_agent: &'a str,
}
