use std::env;
use std::path::{Path, PathBuf};
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=platform/macos/auth-macos-touchid.swift");

    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "macos" {
        println!("cargo:rustc-env=AUTH_BUILT_MACOS_HELPER=");
        return;
    }

    let source = Path::new("platform/macos/auth-macos-touchid.swift");
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR is set by Cargo"));
    let helper = out_dir.join("auth-macos-touchid");

    let swiftc = env::var("SWIFTC").unwrap_or_else(|_| "swiftc".to_string());
    let status = Command::new(&swiftc)
        .arg("-O")
        .arg("-framework")
        .arg("Foundation")
        .arg("-framework")
        .arg("LocalAuthentication")
        .arg(source)
        .arg("-o")
        .arg(&helper)
        .status();

    match status {
        Ok(status) if status.success() => {
            println!(
                "cargo:rustc-env=AUTH_BUILT_MACOS_HELPER={}",
                helper.display()
            );
        }
        Ok(status) => {
            panic!(
                "swiftc failed while building {} with status {}. Install Xcode Command Line Tools or set SWIFTC.",
                source.display(),
                status
            );
        }
        Err(error) => {
            panic!(
                "could not invoke swiftc while building {}: {}. Install Xcode Command Line Tools or set SWIFTC.",
                source.display(),
                error
            );
        }
    }
}
