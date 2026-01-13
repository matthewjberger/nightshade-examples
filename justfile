set windows-shell := ["powershell.exe"]
export RUST_LOG := "info,wgpu_core=off"
export RUST_BACKTRACE := "1"

# Displays the list of available commands
@just:
    just --list

# Builds all examples in release mode
build:
    cargo build -r --workspace

# Runs cargo check on all examples
check:
    cargo check --workspace
    cargo fmt --all -- --check

# Generates and opens documentation
docs:
    cargo doc --open

# Fixes linting issues automatically
fix:
    cargo clippy --workspace --fix

# Formats the code using cargo fmt
format:
    cargo fmt --all

# Install wasm tooling
init-wasm:
    rustup target add wasm32-unknown-unknown
    cargo install --locked trunk

# Runs linter and displays warnings
lint:
    cargo clippy --workspace -- -D warnings

# Runs linter for wasm32 target
lint-wasm:
    cargo clippy --workspace --target wasm32-unknown-unknown -- -D warnings

# Runs the specified example
run $example="alpha_blending":
    cargo run -r -p {{example}}

# Runs the specified example in OpenXR VR mode
run-openxr $example="prefabs":
    cargo run -r -p {{example}} --features openxr

# Build an example for WASM
build-wasm $example="alpha_blending":
    trunk build --release --config examples/{{example}}/Trunk.toml

# Serve an example in browser
run-wasm $example="alpha_blending":
    trunk serve --release --open --config examples/{{example}}/Trunk.toml

# Runs all tests
test:
    cargo test --workspace -- --nocapture

# Displays version information for Rust tools
@versions:
    rustc --version
    cargo fmt -- --version
    cargo clippy -- --version

# Creates a redistributable bundle for an example (Windows)
[windows]
bundle $example:
    cargo build -r -p {{example}}
    $ErrorActionPreference = "Stop"; $bundleDir = "dist/{{example}}"; $zipFile = "dist/{{example}}.zip"; Write-Host "Creating bundle directory: $bundleDir" -ForegroundColor Cyan; if (Test-Path $bundleDir) { Remove-Item -Recurse -Force $bundleDir }; if (Test-Path $zipFile) { Remove-Item -Force $zipFile }; New-Item -ItemType Directory -Force -Path $bundleDir | Out-Null; Write-Host "Copying executable..." -ForegroundColor Cyan; Copy-Item "target/release/{{example}}.exe" "$bundleDir/"; Write-Host "Creating zip archive: $zipFile" -ForegroundColor Cyan; Compress-Archive -Path "$bundleDir/*" -DestinationPath $zipFile -Force; Write-Host ""; Write-Host "Bundle created successfully!" -ForegroundColor Green; Write-Host "  Directory: $bundleDir" -ForegroundColor Green; Write-Host "  Archive:   $zipFile" -ForegroundColor Green

# Creates a redistributable bundle for an example (Unix)
[unix]
bundle $example:
    #!/usr/bin/env bash
    set -euo pipefail
    cargo build -r -p {{example}}
    bundle_dir="dist/{{example}}"
    zip_file="dist/{{example}}.zip"
    echo -e "\033[36mCreating bundle directory: $bundle_dir\033[0m"
    rm -rf "$bundle_dir" "$zip_file"
    mkdir -p "$bundle_dir"
    echo -e "\033[36mCopying executable...\033[0m"
    cp "target/release/{{example}}" "$bundle_dir/"
    echo -e "\033[36mCreating zip archive: $zip_file\033[0m"
    cd dist && zip -r "{{example}}.zip" "{{example}}" && cd ..
    echo ""
    echo -e "\033[32mBundle created successfully!\033[0m"
    echo -e "\033[32m  Directory: $bundle_dir\033[0m"
    echo -e "\033[32m  Archive:   $zip_file\033[0m"
