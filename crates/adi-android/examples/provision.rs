//! Device-only provisioning and state restoration check.
use adi_android::{Libraries, Runtime};
use anisette::{
    protocol::{ClientInfo, Identifier},
    provisioning::Exchange,
};
fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("provide explicit library directory; contacts Apple")?;
    let libraries = Libraries::open(std::path::Path::new(&path))?;
    let mut id = [0; 16];
    getrandom::fill(&mut id).map_err(|_| "random source failed")?;
    let identifier = Identifier::from_bytes(id);
    let mut runtime = Runtime::new(&libraries, identifier.clone(), None)?;
    // Android ADI uses the Windows HTTP profile.
    let (exchange, spim) = Exchange::start(
        id,
        ClientInfo {
            client_info: "<PC> <Windows;6.2(0,0);9200> <com.apple.AuthKit/1 (com.apple.akd/1.0)>",
            user_agent: "akd/1.0 CFNetwork/978.0.7 Darwin/18.7.0",
        },
    )?;
    let (cpim, session) = runtime.start_provisioning(&spim)?;
    let result = (|| -> Result<_, Box<dyn std::error::Error + Send + Sync>> {
        let finish = exchange.finish(&cpim)?;
        runtime.finish_provisioning(session, &finish.ptm, &finish.tk)?;
        Ok(finish.routing)
    })();
    let routing = match result {
        Ok(r) => r,
        Err(e) => {
            let _ = runtime.cancel_provisioning(session);
            return Err(e);
        }
    };
    assert!(runtime.is_machine_provisioned()?);
    println!(
        "Android routing from standard ptm/tk matches Apple: {}",
        runtime.routing()? == routing
    );
    runtime.set_routing(routing)?;
    assert!(runtime.routing()? == routing);
    let (mid, otp) = runtime.request_otp()?;
    assert!(!mid.is_empty() && !otp.is_empty());
    let state = runtime.export_state()?;
    for _ in 0..2 {
        let mut restored = Runtime::new(&libraries, identifier.clone(), Some(&state))?;
        assert!(restored.is_machine_provisioned()?);
        assert!(restored.routing()? == routing);
        for _ in 0..5 {
            let (next_mid, next_otp) = restored.request_otp()?;
            assert!(next_mid == mid && !next_otp.is_empty());
        }
    }
    println!(
        "Android provisioning/OTP/two repeated restores passed; routing matches Dadoum: {}; no secrets persisted; no account login tested",
        routing == 17106176
    );
    Ok(())
}
