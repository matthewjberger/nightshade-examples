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
    just lint-wasm

# Runs linter for wasm32 target (excludes native-only and terminal examples)
[windows]
lint-wasm:
    $excludes = @("demo_scene", "multiplayer_pong", "steam", "benchmark", "text_adventure", "terminal_breakout", "assets"); Get-ChildItem -Directory "examples-terminal" | ForEach-Object { $excludes += $_.Name }; $flags = ($excludes | ForEach-Object { "--exclude $_" }) -join " "; Invoke-Expression "cargo clippy --workspace --target wasm32-unknown-unknown $flags -- -D warnings"

# Runs linter for wasm32 target (excludes native-only and terminal examples)
[unix]
lint-wasm:
    #!/usr/bin/env bash
    set -euo pipefail
    excludes="--exclude demo_scene --exclude multiplayer_pong --exclude steam --exclude benchmark --exclude text_adventure --exclude terminal_breakout --exclude assets"
    for dir in examples-terminal/*/; do
        name=$(basename "$dir")
        excludes="$excludes --exclude $name"
    done
    cargo clippy --workspace --target wasm32-unknown-unknown $excludes -- -D warnings

# Runs a terminal example in TUI mode (windowed with SDF text)
run-tui $example:
    cargo run -r -p {{example}} --no-default-features --features tui

# Build a terminal example for WASM
build-wasm-tui $example:
    trunk build --release --config examples-terminal/{{example}}/Trunk.toml

# Serve a terminal example in browser
run-tui-wasm $example:
    trunk serve --release --open --config examples-terminal/{{example}}/Trunk.toml

# Interactively pick and run an example
[windows]
pick:
    $example = (Get-ChildItem -Directory "examples" | ForEach-Object { $_.Name } | Sort-Object | fzf --prompt="Pick a demo> "); if ($example) { cargo run -r -p $example }

# Interactively pick and run an example
[unix]
pick:
    #!/usr/bin/env bash
    set -euo pipefail
    example=$(ls -d examples/*/ 2>/dev/null | xargs -I{} basename {} | sort | fzf --prompt="Pick a demo> ")
    [ -n "$example" ] && cargo run -r -p "$example"

# Interactively pick and serve an example in browser
[windows]
pick-wasm:
    $example = (Get-ChildItem -Directory "examples" | ForEach-Object { $_.Name } | Sort-Object | fzf --prompt="Pick a WASM demo> "); if ($example) { trunk serve --release --open --config "examples/$example/Trunk.toml" }

# Interactively pick and serve an example in browser
[unix]
pick-wasm:
    #!/usr/bin/env bash
    set -euo pipefail
    example=$(ls -d examples/*/ 2>/dev/null | xargs -I{} basename {} | sort | fzf --prompt="Pick a WASM demo> ")
    [ -n "$example" ] && trunk serve --release --open --config "examples/$example/Trunk.toml"

# Interactively pick and run a native-only example
[windows]
pick-native:
    $example = (Get-ChildItem -Directory "examples-native" | ForEach-Object { $_.Name } | Sort-Object | fzf --prompt="Pick a native demo> "); if ($example) { cargo run -r -p $example }

# Interactively pick and run a native-only example
[unix]
pick-native:
    #!/usr/bin/env bash
    set -euo pipefail
    example=$(ls -d examples-native/*/ 2>/dev/null | xargs -I{} basename {} | sort | fzf --prompt="Pick a native demo> ")
    [ -n "$example" ] && cargo run -r -p "$example"

# Interactively pick and run a terminal example (in terminal)
[windows]
pick-terminal:
    $example = (Get-ChildItem -Directory "examples-terminal" | ForEach-Object { $_.Name } | Sort-Object | fzf --prompt="Pick a terminal demo> "); if ($example) { cargo run -r -p $example }

# Interactively pick and run a terminal example (in terminal)
[unix]
pick-terminal:
    #!/usr/bin/env bash
    set -euo pipefail
    example=$(ls -d examples-terminal/*/ 2>/dev/null | xargs -I{} basename {} | sort | fzf --prompt="Pick a terminal demo> ")
    [ -n "$example" ] && cargo run -r -p "$example"

# Interactively pick and run a terminal example in TUI mode (windowed)
[windows]
pick-tui:
    $example = (Get-ChildItem -Directory "examples-terminal" | ForEach-Object { $_.Name } | Sort-Object | fzf --prompt="Pick a TUI demo> "); if ($example) { cargo run -r -p $example --no-default-features --features tui }

# Interactively pick and run a terminal example in TUI mode (windowed)
[unix]
pick-tui:
    #!/usr/bin/env bash
    set -euo pipefail
    example=$(ls -d examples-terminal/*/ 2>/dev/null | xargs -I{} basename {} | sort | fzf --prompt="Pick a TUI demo> ")
    [ -n "$example" ] && cargo run -r -p "$example" --no-default-features --features tui

# Interactively pick and serve a terminal example in browser
[windows]
pick-wasm-tui:
    $example = (Get-ChildItem -Directory "examples-terminal" | ForEach-Object { $_.Name } | Sort-Object | fzf --prompt="Pick a WASM TUI demo> "); if ($example) { trunk serve --release --open --config "examples-terminal/$example/Trunk.toml" }

# Interactively pick and serve a terminal example in browser
[unix]
pick-wasm-tui:
    #!/usr/bin/env bash
    set -euo pipefail
    example=$(ls -d examples-terminal/*/ 2>/dev/null | xargs -I{} basename {} | sort | fzf --prompt="Pick a WASM TUI demo> ")
    [ -n "$example" ] && trunk serve --release --open --config "examples-terminal/$example/Trunk.toml"

# Interactively pick an example and generate a snapshot
[windows]
pick-snapshot:
    $example = (Get-ChildItem -Directory "examples" | ForEach-Object { $_.Name } | Sort-Object | fzf --prompt="Pick a demo to snapshot> "); if ($example) { New-Item -ItemType Directory -Force -Path "snapshots" | Out-Null; $env:NIGHTSHADE_SNAPSHOT_PATH = "snapshots/$example.png"; cargo run -r -p $example }

# Interactively pick an example and generate a snapshot
[unix]
pick-snapshot:
    #!/usr/bin/env bash
    set -euo pipefail
    example=$(ls -d examples/*/ 2>/dev/null | xargs -I{} basename {} | sort | fzf --prompt="Pick a demo to snapshot> ")
    if [ -n "$example" ]; then
        mkdir -p snapshots
        NIGHTSHADE_SNAPSHOT_PATH="snapshots/$example.png" cargo run -r -p "$example"
    fi

# Generate snapshots for all examples
[windows]
generate-snapshots:
    $ErrorActionPreference = "Stop"; New-Item -ItemType Directory -Force -Path "snapshots" | Out-Null; $examples = Get-ChildItem -Directory "examples" | ForEach-Object { $_.Name } | Sort-Object; $total = $examples.Count; $current = 0; foreach ($example in $examples) { $current++; Write-Host "[$current/$total] Snapshotting $example..." -ForegroundColor Cyan; $env:NIGHTSHADE_SNAPSHOT_PATH = "snapshots/$example.png"; cargo run -r -p $example; }; Write-Host "All snapshots complete! Output in snapshots/" -ForegroundColor Green

# Generate snapshots for all examples
[unix]
generate-snapshots:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p snapshots
    examples=($(ls -d examples/*/ 2>/dev/null | xargs -I{} basename {} | sort))
    total=${#examples[@]}
    current=0
    for example in "${examples[@]}"; do
        current=$((current + 1))
        echo -e "\033[36m[$current/$total] Snapshotting $example...\033[0m"
        NIGHTSHADE_SNAPSHOT_PATH="snapshots/$example.png" cargo run -r -p "$example"
    done
    echo -e "\033[32mAll snapshots complete! Output in snapshots/\033[0m"

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

# Build all examples for WASM (Windows)
[windows]
build-all-wasm:
    $ErrorActionPreference = "Stop"; $prefix = $env:PUBLIC_URL_PREFIX; $root = (Get-Location).Path; Get-ChildItem -Path "examples/*/Trunk.toml" | ForEach-Object { $example = $_.Directory.Name; Write-Host "Building $example for WASM..." -ForegroundColor Cyan; if ($prefix) { trunk build --release --config $_.FullName --dist "$root/dist/wasm/$example" --public-url "$prefix$example/" } else { trunk build --release --config $_.FullName --dist "$root/dist/wasm/$example" } }; Get-ChildItem -Path "examples-terminal/*/Trunk.toml" | ForEach-Object { $example = $_.Directory.Name; Write-Host "Building $example (terminal) for WASM..." -ForegroundColor Cyan; if ($prefix) { trunk build --release --config $_.FullName --dist "$root/dist/wasm/$example" --public-url "$prefix$example/" } else { trunk build --release --config $_.FullName --dist "$root/dist/wasm/$example" } }; Write-Host "All WASM builds complete! Output in dist/wasm/" -ForegroundColor Green

# Build all examples for WASM (Unix)
[unix]
build-all-wasm:
    #!/usr/bin/env bash
    set -euo pipefail
    prefix="${PUBLIC_URL_PREFIX:-}"
    root="$(pwd)"
    for trunk_file in examples/*/Trunk.toml examples-terminal/*/Trunk.toml; do
        [ -f "$trunk_file" ] || continue
        example=$(basename "$(dirname "$trunk_file")")
        echo "Building $example for WASM..."
        if [ -n "$prefix" ]; then
            trunk build --release --config "$trunk_file" --dist "$root/dist/wasm/$example" --public-url "${prefix}$example/"
        else
            trunk build --release --config "$trunk_file" --dist "$root/dist/wasm/$example"
        fi
    done
    echo "All WASM builds complete! Output in dist/wasm/"

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
