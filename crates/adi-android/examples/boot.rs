use adi_android::{Libraries, Runtime};
use anisette::protocol::Identifier;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("provide library directory")?;
    let libraries = Libraries::open(std::path::Path::new(&path))?;
    let mut id = [0; 16];
    getrandom::fill(&mut id).map_err(|_| "random source failed")?;
    let mut runtime = Runtime::new(&libraries, Identifier::from_bytes(id), None)?;
    assert!(!runtime.is_machine_provisioned()?);
    assert!(matches!(
        runtime.routing(),
        Err(adi_android::Error::Adi(-45061))
    ));
    println!("Android ARM64 libraries initialized; independent fresh device is not provisioned");
    Ok(())
}
