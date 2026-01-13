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
run $example="doom":
    cargo run -r -p {{example}}

# Runs the specified example in OpenXR VR mode
run-openxr $example="horror":
    cargo run -r -p {{example}} --features openxr

# Build an example for WASM
build-wasm $example="doom":
    trunk build --release --config examples/{{example}}/Trunk.toml

# Serve an example in browser
run-wasm $example="doom":
    trunk serve --release --open --config examples/{{example}}/Trunk.toml

# Runs all tests
test:
    cargo test --workspace -- --nocapture

# Displays version information for Rust tools
@versions:
    rustc --version
    cargo fmt -- --version
    cargo clippy -- --version
