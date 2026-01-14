use std::env;
use std::path::Path;
use std::process::Command;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let leptos_site_dir = Path::new(&manifest_dir).join("leptos-site");

    println!("cargo:rerun-if-changed=leptos-site/src");
    println!("cargo:rerun-if-changed=leptos-site/index.html");
    println!("cargo:rerun-if-changed=leptos-site/Cargo.toml");
    println!("cargo:rerun-if-changed=leptos-site/public");

    if !leptos_site_dir.exists() {
        panic!("leptos-site directory not found at {:?}", leptos_site_dir);
    }

    let status = Command::new("trunk")
        .args(["build", "--release"])
        .current_dir(&leptos_site_dir)
        .status()
        .expect("Failed to run trunk build");

    if !status.success() {
        panic!("trunk build failed with status: {}", status);
    }
}
