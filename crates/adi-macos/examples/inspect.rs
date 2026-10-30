use adi_macos::Binary;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = std::env::args_os()
        .nth(1)
        .ok_or("provide an explicit binary path")?;
    let binary = Binary::open(std::path::Path::new(&path))?;
    println!(
        "verified macOS 26 x86_64 adid: {} bytes, {} segments, {} bindings",
        binary.bytes().len(),
        binary.image().segments.len(),
        binary.image().bindings.len()
    );
    Ok(())
}
