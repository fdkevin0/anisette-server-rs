#[test]
#[ignore = "requires explicitly supplied Apple binary; validates parsing only"]
fn pinned_binary_layout() {
    let path = std::env::var_os("ANISETTE_MACOS_BINARY").expect("set ANISETTE_MACOS_BINARY");
    let binary = adi_macos::Binary::open(std::path::Path::new(&path)).unwrap();
    assert_eq!(binary.bytes().len(), 3677696);
    assert_eq!(binary.image().segments.len(), 5);
    assert_eq!(binary.image().bindings.len(), 131);
    assert!(
        binary
            .image()
            .segments
            .iter()
            .any(|s| s.name == "__PAGEZERO" && s.file_size == 0)
    );
    assert!(
        binary
            .image()
            .bindings
            .iter()
            .any(|b| b.symbol == "_CFRunLoopRun")
    );
}

#[test]
#[ignore = "requires explicit Apple binary and JIT permission; no network or saved identity"]
fn fresh_daemon_answers_repeated_xpc_queries() {
    let path = std::env::var_os("ANISETTE_MACOS_BINARY").expect("set ANISETTE_MACOS_BINARY");
    let binary = adi_macos::Binary::open(std::path::Path::new(&path)).unwrap();
    for _ in 0..2 {
        let mut runtime = adi_macos::Runtime::new(&binary).unwrap();
        runtime.boot().unwrap();
        for _ in 0..20 {
            assert!(!runtime.is_machine_provisioned(-2).unwrap());
        }
        assert!(matches!(
            runtime.request_otp(-2),
            Err(adi_macos::Error::Adi(-45061))
        ));
        let state = runtime.export_state([7; 16]).unwrap();
        assert!(adi_macos::Runtime::restore(&binary, [8; 16], &state).is_err());
        let mut restored = adi_macos::Runtime::restore(&binary, [7; 16], &state).unwrap();
        assert!(!restored.is_machine_provisioned(-2).unwrap());
    }
}
