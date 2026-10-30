fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("usage: boot CORE_ADI_DLL")?;
    let binary = adi_windows::Binary::open(std::path::Path::new(&path))?;
    println!("Verified Windows PE; imports={}", binary.imports().count());
    let mut runtime = adi_windows::Runtime::new(&binary)?;
    println!(
        "DLL attached; initialization status={}",
        runtime.initialize()?
    );
    println!("Fresh device provisioned={}", runtime.is_provisioned()?);
    Ok(())
}
