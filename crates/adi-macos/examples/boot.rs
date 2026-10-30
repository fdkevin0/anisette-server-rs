fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("provide explicit binary path")?;
    let binary = adi_macos::Binary::open(std::path::Path::new(&path))?;
    let mut runtime = adi_macos::Runtime::new(&binary)?;
    runtime.boot()?;
    println!(
        "daemon reached run loop; provisioned: {}",
        runtime.is_machine_provisioned(-2)?
    );
    Ok(())
}
