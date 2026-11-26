//! Stamps the commit into the `mash` and `mash-server` programs, and on
//! Windows has them load `DirectML` on first use; see `cli/delay_load.c`.
//!
//! This script belongs to the package that builds the programs, the last one
//! compiled, so a new commit recompiles that package and nothing under it.
//! Keep the package to the programs' entry points: Cargo compiles all of it
//! again whenever the commit changes. The command lines are in `mash-cli`,
//! and the library crates read the commit at run time
//! (`mash_app::build_revision`).
use std::process::Command;

fn main() {
    delay_load_directml();
    let override_revision = std::env::var("MASH_BUILD_REVISION").ok();
    let revision = override_revision.clone().unwrap_or_else(|| {
        Command::new("git")
            .args(["rev-parse", "HEAD"])
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .unwrap_or_else(|| "unknown".into())
    });
    println!("cargo:rerun-if-env-changed=MASH_BUILD_REVISION");
    let revision = revision.trim();
    if revision != "unknown"
        && (revision.len() != 40
            || !revision
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
    {
        eprintln!("MASH_BUILD_REVISION must be a full lowercase Git commit ID");
        std::process::exit(1);
    }
    println!("cargo:rustc-env=MASH_BUILD_REVISION={}", revision.trim());
    println!("cargo:rerun-if-changed=build.rs");
    if override_revision.is_none() {
        for name in ["HEAD", "refs/heads", "packed-refs"] {
            if let Ok(output) = Command::new("git")
                .args(["rev-parse", "--git-path", name])
                .output()
                && output.status.success()
                && let Ok(path) = String::from_utf8(output.stdout)
                && std::path::Path::new(path.trim()).exists()
            {
                println!("cargo:rerun-if-changed={}", path.trim());
            }
        }
    }
}

/// On Windows, `DirectML` and Direct3D 12 are loaded when the GPU is first
/// asked for, so the programs start on a Windows without them. Only the
/// programs get the linker options. The hook that answers a failed load is
/// linked into every target of this package, which has no library.
fn delay_load_directml() {
    println!("cargo:rerun-if-changed=cli/delay_load.c");
    let os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let toolchain = std::env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    if os != "windows" || toolchain != "msvc" {
        return;
    }
    // Whole-archive, because nothing in Rust refers to the hook: the linker's
    // delay-load helper does, and would otherwise take its own empty one.
    cc::Build::new()
        .file("cli/delay_load.c")
        .link_lib_modifier("+whole-archive")
        .compile("mash_delay_load");
    for library in ["directml.dll", "d3d12.dll"] {
        println!("cargo:rustc-link-arg-bins=/DELAYLOAD:{library}");
    }
    println!("cargo:rustc-link-arg-bins=delayimp.lib");
}
