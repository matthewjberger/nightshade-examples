use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let leptos_site_dir = Path::new(&manifest_dir).join("site");
    let dist_dir = leptos_site_dir.join("dist");

    println!("cargo:rerun-if-changed=site/src");
    println!("cargo:rerun-if-changed=site/index.html");
    println!("cargo:rerun-if-changed=site/Cargo.toml");
    println!("cargo:rerun-if-changed=site/public");
    println!("cargo:rerun-if-changed=site/dist");

    if !leptos_site_dir.exists() {
        panic!("site directory not found at {:?}", leptos_site_dir);
    }

    if dist_dir.join("index.html").exists() {
        return;
    }

    let status = Command::new("trunk")
        .args(["build", "--release"])
        .current_dir(&leptos_site_dir)
        .status()
        .expect("Failed to run trunk build. Install trunk: cargo install trunk");

    if !status.success() {
        panic!("trunk build failed with status: {}", status);
    }
}
