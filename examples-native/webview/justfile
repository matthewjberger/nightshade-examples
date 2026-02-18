set windows-shell := ["powershell.exe"]

# Displays the list of available commands
@just:
    just --list

# Run desktop app
[working-directory: 'site']
run: setup
  npx tailwindcss -i public/input.css -o public/output.css
  trunk build --release
  cargo run --manifest-path ../Cargo.toml

# Setup site (install npm dependencies)
[working-directory: 'site']
setup:
  npm install

# Format all code
format:
  cargo fmt --all

# Lint all code
lint:
  cargo clippy --all -- -D warnings

# Check all code
check:
  cargo check --all
