#[test]
#[ignore = "requires explicit ANISETTE_WINDOWS_BINARY and Unicorn JIT; no network or saved identity"]
fn fresh_windows_dll_initializes_and_repeats_unprovisioned_queries() {
    let path = std::env::var_os("ANISETTE_WINDOWS_BINARY").expect("explicit binary path");
    let binary = adi_windows::Binary::open(std::path::Path::new(&path)).unwrap();
    for _ in 0..2 {
        let mut runtime = adi_windows::Runtime::new(&binary).unwrap();
        assert!(runtime.initialize().unwrap() >= 0);
        for _ in 0..20 {
            assert!(!runtime.is_provisioned().unwrap());
        }
    }
}
