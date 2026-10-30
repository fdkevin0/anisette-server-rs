//! Explicit Apple device provisioning. Never accepts Apple account credentials.
use base64::{Engine, engine::general_purpose::STANDARD};
use plist::{Dictionary, Value};
use sha2::{Digest, Sha256};
use std::{
    io::{Cursor, Read, Write},
    process::{Child, Command, Stdio},
    sync::mpsc,
    time::{Duration, Instant},
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;
fn field<'a>(v: &'a Value, key: &str) -> Result<&'a Value> {
    v.as_dictionary()
        .and_then(|d| d.get(key))
        .ok_or("missing plist field".into())
}
fn string<'a>(v: &'a Value, key: &str) -> Result<&'a str> {
    field(v, key)?
        .as_string()
        .ok_or("expected plist string".into())
}
const MAX_RESPONSE: usize = 1024 * 1024;
fn quoted(value: &str) -> Result<String> {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => return Err("unsupported request character".into()),
            c => out.push(c),
        }
    }
    out.push('"');
    Ok(out)
}
fn collect_transfer(mut child: Child, config: String, timeout: Duration) -> Result<Vec<u8>> {
    let deadline = Instant::now() + timeout;
    let mut input = child.stdin.take().ok_or("missing curl input")?;
    let output = child.stdout.take().ok_or("missing curl output")?;
    let (send, receive) = mpsc::sync_channel(1);
    let io = std::thread::spawn(move || {
        let result = (|| -> std::io::Result<Vec<u8>> {
            input.write_all(config.as_bytes())?;
            drop(input);
            let mut bytes = Vec::new();
            output
                .take(MAX_RESPONSE as u64 + 5)
                .read_to_end(&mut bytes)?;
            Ok(bytes)
        })();
        let _ = send.send(result);
    });
    let result = (|| -> Result<Vec<u8>> {
        let bytes = receive
            .recv_timeout(timeout)
            .map_err(|_| "Apple transfer timed out")?
            .map_err(|_| "Apple transfer IO failed")?;
        if bytes.len() > MAX_RESPONSE + 4 {
            return Err("Apple response too large".into());
        }
        loop {
            if let Some(status) = child.try_wait()? {
                if !status.success() {
                    return Err("curl Apple transfer failed".into());
                }
                return Ok(bytes);
            }
            if Instant::now() >= deadline {
                return Err("Apple transfer timed out".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    })();
    if result.is_err() {
        let _ = child.kill();
        let _ = child.wait();
    }
    io.join().map_err(|_| "Apple transfer worker failed")?;
    result
}
fn response_bytes(mut bytes: Vec<u8>) -> Result<Vec<u8>> {
    if bytes.len() < 4 || bytes.len() > MAX_RESPONSE + 4 || bytes[bytes.len() - 4] != b'\n' {
        return Err("invalid curl response".into());
    }
    let status = std::str::from_utf8(&bytes[bytes.len() - 3..])?.parse::<u16>()?;
    if !(200..300).contains(&status) {
        return Err("Apple HTTP request rejected".into());
    }
    bytes.truncate(bytes.len() - 4);
    Ok(bytes)
}
fn response(url: &url::Url, headers: &str, body: Option<&str>) -> Result<Value> {
    let mut config = format!("url = {}\n{headers}", quoted(url.as_str())?);
    if let Some(body) = body {
        config.push_str(&format!("data-binary = {}\n", quoted(body)?));
    }
    let child = Command::new("curl")
        .args([
            "--disable",
            "--silent",
            "--fail",
            "--globoff",
            "--noproxy",
            "*",
            "--proto",
            "=https",
            "--tlsv1.2",
            "--http1.1",
            "--connect-timeout",
            "10",
            "--max-time",
            "20",
            "--max-filesize",
            "1048576",
            "--write-out",
            "\n%{http_code}",
            "--config",
            "-",
        ])
        .env_remove("SSLKEYLOGFILE")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .map_err(|_| "curl is required for Apple device provisioning")?;
    // Headers and CPIM stay in the pipe, never in argv, temporary files, or diagnostics.
    let bytes = response_bytes(collect_transfer(child, config, Duration::from_secs(25))?)?;
    Ok(Value::from_reader(Cursor::new(bytes))?)
}
fn endpoint(value: &Value, key: &str) -> Result<url::Url> {
    let url = url::Url::parse(string(value, key)?)?;
    if url.scheme() != "https"
        || url.host_str() != Some("gsa.apple.com")
        || url.port_or_known_default() != Some(443)
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err("unexpected Apple provisioning endpoint".into());
    }
    Ok(url)
}
fn post(headers: &str, url: url::Url, request: Dictionary) -> Result<Value> {
    let root = Value::Dictionary(Dictionary::from_iter([
        ("Header", Value::Dictionary(Dictionary::new())),
        ("Request", Value::Dictionary(request)),
    ]));
    let mut bytes = vec![];
    root.to_writer_xml(&mut bytes)?;
    let headers = format!(
        "{headers}header = {}\n",
        quoted(&format!("X-Apple-I-Client-Time: {}", client_time()?))?
    );
    let body = std::str::from_utf8(&bytes)?;
    let value = response(&url, &headers, Some(body))?;
    let response = field(&value, "Response")?;
    let status = field(response, "Status")?;
    if field(status, "ec")?.as_signed_integer() != Some(0) {
        return Err("Apple device provisioning rejected".into());
    }
    Ok(response.clone())
}
pub fn local_user_id(identifier: [u8; 16]) -> String {
    Sha256::digest(identifier)
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect()
}
/// Formats the current UTC time for Apple request headers.
///
/// # Errors
/// Returns an error if the timestamp cannot be normalized or formatted as RFC 3339.
pub fn client_time() -> Result<String> {
    Ok(time::OffsetDateTime::now_utc()
        .replace_nanosecond(0)?
        .format(&time::format_description::well_known::Rfc3339)?)
}
/// One Apple device exchange, also usable by a V3 client. No account credentials.
pub struct Exchange {
    headers: String,
    finish_url: url::Url,
}
pub struct FinishData {
    pub ptm: Vec<u8>,
    pub tk: Vec<u8>,
    pub routing: u64,
}
impl Exchange {
    /// Contacts Apple to obtain start data for one device provisioning exchange.
    ///
    /// # Errors
    /// Returns an error for invalid profile headers, unavailable curl, transport failures, untrusted endpoints, or invalid Apple responses.
    pub fn start(
        identifier: [u8; 16],
        profile: crate::protocol::ClientInfo<'_>,
    ) -> Result<(Self, Vec<u8>)> {
        let device = crate::protocol::Identifier::from_bytes(identifier).uuid();
        let lu = local_user_id(identifier);
        let mut headers = String::new();
        for (name, value) in [
            ("X-Mme-Device-Id", device.as_str()),
            ("X-Apple-I-MD-LU", lu.as_str()),
            ("X-Apple-I-SRL-NO", "0"),
            ("X-MMe-Client-Info", profile.client_info),
            ("User-Agent", profile.user_agent),
            ("X-Apple-Client-App-Name", "Setup"),
            ("Content-Type", "application/x-www-form-urlencoded"),
        ] {
            if value.len() > 512 || !value.bytes().all(|b| (32..127).contains(&b)) {
                return Err("invalid provisioning header".into());
            }
            headers.push_str(&format!(
                "header = {}\n",
                quoted(&format!("{name}: {value}"))?
            ));
        }
        let bag = response(
            &url::Url::parse("https://gsa.apple.com/grandslam/GsService2/lookup")?,
            &headers,
            None,
        )?;
        let urls = field(&bag, "urls")?;
        let start = post(
            &headers,
            endpoint(urls, "midStartProvisioning")?,
            Dictionary::new(),
        )?;
        let spim = STANDARD.decode(string(&start, "spim")?)?;
        Ok((
            Self {
                headers,
                finish_url: endpoint(urls, "midFinishProvisioning")?,
            },
            spim,
        ))
    }
    /// Submits CPIM to Apple and returns the completion data and routing value.
    ///
    /// # Errors
    /// Returns an error for empty or oversized CPIM, transfer failures, rejected provisioning, or invalid completion fields.
    pub fn finish(self, cpim: &[u8]) -> Result<FinishData> {
        if cpim.is_empty() || cpim.len() > 65536 {
            return Err("invalid CPIM size".into());
        }
        let finish = post(
            &self.headers,
            self.finish_url,
            Dictionary::from_iter([("cpim", Value::String(STANDARD.encode(cpim)))]),
        )?;
        let value = field(&finish, "X-Apple-I-MD-RINFO")?;
        let routing = value
            .as_unsigned_integer()
            .or_else(|| value.as_string()?.parse().ok())
            .filter(|n| *n != 0)
            .ok_or("invalid routing")?;
        Ok(FinishData {
            ptm: STANDARD.decode(string(&finish, "ptm")?)?,
            tk: STANDARD.decode(string(&finish, "tk")?)?,
            routing,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn errors_can_cross_worker_boundaries() {
        fn assert_send_sync<T: Send + Sync + 'static>() {}
        assert_send_sync::<Result<()>>();
    }
    #[test]
    fn transfer_config_and_response_boundaries() {
        assert_eq!(quoted("a\"\\\n\r\tb").unwrap(), "\"a\\\"\\\\\\n\\r\\tb\"");
        assert!(quoted("a\0b").is_err());
        assert_eq!(response_bytes(b"body\n200".to_vec()).unwrap(), b"body");
        for bytes in [b"body\n302".as_slice(), b"body\n500", b"body200", b"\nabc"] {
            assert!(response_bytes(bytes.to_vec()).is_err());
        }
        assert!(response_bytes(vec![0; MAX_RESPONSE + 5]).is_err());
    }
    #[cfg(unix)]
    #[test]
    fn transfer_reaps_failed_oversized_and_timed_out_children() {
        fn child(script: &str) -> Child {
            Command::new("sh")
                .args(["-c", script])
                .stdin(Stdio::piped())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
                .unwrap()
        }
        assert_eq!(
            collect_transfer(child("cat"), "input".into(), Duration::from_secs(2)).unwrap(),
            b"input"
        );
        assert!(collect_transfer(child("exit 1"), String::new(), Duration::from_secs(2)).is_err());
        assert!(
            collect_transfer(
                child("exec cat /dev/zero"),
                String::new(),
                Duration::from_secs(2)
            )
            .is_err()
        );
        let start = Instant::now();
        assert!(
            collect_transfer(
                child("exec sleep 10"),
                String::new(),
                Duration::from_millis(50)
            )
            .is_err()
        );
        assert!(start.elapsed() < Duration::from_secs(2));
    }
    #[test]
    fn lookup_cannot_redirect_provisioning_to_untrusted_endpoints() {
        for url in [
            "http://gsa.apple.com/x",
            "https://example.com/x",
            "https://gsa.apple.com.evil.test/x",
            "https://user@gsa.apple.com/x",
            "https://gsa.apple.com:444/x",
            "file:///etc/passwd",
        ] {
            let value =
                Value::Dictionary(Dictionary::from_iter([("url", Value::String(url.into()))]));
            assert!(endpoint(&value, "url").is_err());
        }
        let value = Value::Dictionary(Dictionary::from_iter([(
            "url",
            Value::String("https://gsa.apple.com/x".into()),
        )]));
        assert!(endpoint(&value, "url").is_ok());
    }
}
