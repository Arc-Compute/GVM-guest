fn main() {
    #[cfg(target_os = "linux")]
    cc::Build::new()
        .file("c_src/linux-comms.c")
        .compile("comms");
}
