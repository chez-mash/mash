//! Links the glibc compatibility entry points on Linux; see `glibc_compat.c`.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=glibc_compat.c");
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let libc = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if os == "linux" && libc == "gnu" {
        // Whole-archive, because ONNX Runtime is linked after this crate and a
        // linker that reads archives in order would otherwise skip these.
        cc::Build::new()
            .file("glibc_compat.c")
            .link_lib_modifier("+whole-archive")
            .compile("mash_glibc_compat");
    }
}
