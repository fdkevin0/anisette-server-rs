//! Explicit live device-only smoke test. No account login, state files, or secret output.
use adi_macos::{Binary, Runtime, provisioning::provision};
use anisette::provisioning::Result;
fn main() -> Result<()> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("provide explicit binary path; this example contacts Apple")?;
    let binary = Binary::open(std::path::Path::new(&path))?;
    let mut identifier = [0; 16];
    getrandom::fill(&mut identifier).map_err(|_| "random source failed")?;
    let mut runtime = provision(&binary, identifier)?;
    let routing = runtime.routing(-2)?;
    assert!(runtime.is_machine_provisioned(-2)?);
    let (mid, otp) = runtime.request_otp(-2)?;
    assert!(!mid.is_empty() && !otp.is_empty());
    let state = runtime.export_state(identifier)?;
    for _ in 0..2 {
        let mut restored = Runtime::restore(&binary, identifier, &state)?;
        assert!(restored.is_machine_provisioned(-2)?);
        let (next_mid, next_otp) = restored.request_otp(-2)?;
        assert_eq!(mid, next_mid);
        assert!(!next_otp.is_empty());
        assert_eq!(restored.routing(-2)?, routing);
    }
    println!(
        "OTP and two independent restores passed; state/OTP not printed or persisted; Apple account acceptance not tested"
    );
    Ok(())
}
