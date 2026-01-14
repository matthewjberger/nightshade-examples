# Build the Leptos frontend
build-frontend:
    cd site && trunk build --release

# Build the native app
build: build-frontend
    cargo build --release

# Run the app (builds frontend first)
run: build-frontend
    cargo run

# Run the app in release mode
run-release: build
    cargo run --release

# Check the native app
check:
    cargo check

# Clean all build artifacts
clean:
    cargo clean
    cd site && trunk clean

# Watch and rebuild frontend on changes
watch-frontend:
    cd site && trunk watch

# Format all code
fmt:
    cargo fmt
    cd site && cargo fmt

# Run clippy
lint:
    cargo clippy
    cd site && cargo clippy
