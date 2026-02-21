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
    $excludes = @("demo_scene", "multiplayer_pong", "steam", "text_adventure", "assets", "claude_chat", "claude_chat_mcp"); Get-ChildItem -Directory "examples-terminal" | ForEach-Object { $excludes += $_.Name }; $flags = ($excludes | ForEach-Object { "--exclude $_" }) -join " "; Invoke-Expression "cargo clippy --workspace --target wasm32-unknown-unknown $flags -- -D warnings"

# Runs linter for wasm32 target (excludes native-only and terminal examples)
[unix]
lint-wasm:
    #!/usr/bin/env bash
    set -euo pipefail
    excludes="--exclude demo_scene --exclude multiplayer_pong --exclude steam --exclude text_adventure --exclude assets --exclude claude_chat --exclude claude_chat_mcp"
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

# Interactively pick and run any example (2D or 3D)
[windows]
pick:
    $descriptions = @{ "alpha_blending" = "Alpha blending and transparency"; "asteroid_belt" = "Asteroid belt simulation"; "audio" = "Spatial audio playback"; "block_breaker" = "Block breaker arcade game"; "block_breaker_scripts" = "Block breaker with WASM scripts"; "bloom" = "Bloom post-processing"; "bunnymark" = "Sprite rendering benchmark"; "camera_in_camera" = "Nested camera rendering"; "castle_siege" = "GOAP AI castle siege"; "chess" = "3D chess game"; "chip8" = "Super CHIP-8 emulator"; "city" = "Procedural city generator"; "custom_multipass" = "Gaussian blur multipass"; "custom_pass" = "Custom render pass"; "dance" = "Skinned mesh benchmark"; "decals" = "Decal projection"; "depth_of_field" = "Depth of field effect"; "doom" = "Doom-style renderer"; "farming" = "Farming simulation"; "fireworks" = "Fireworks particle effects"; "genetic_walkers" = "Genetic algorithm walkers"; "gfx" = "2D shape primitives and beziers"; "gfx_showcase" = "2D shape primitives showcase"; "gizmo" = "Transform gizmo"; "hex_war" = "Hex-based strategy game"; "hiz" = "Hi-Z occlusion culling"; "horror" = "First-person horror demo"; "hud_text" = "HUD text rendering"; "immersive_sim" = "Immersive sim sandbox"; "interior_mapping" = "Interior mapping shader"; "jigsaw" = "Jigsaw puzzle game"; "lattice" = "Lattice deformation"; "lights" = "Clustered forward rendering"; "maps" = "Map rendering"; "menu" = "Menu system demo"; "morph" = "Morph targets animation"; "mosaic" = "Multi-window mosaic"; "multi_world" = "Multiple ECS worlds"; "navmesh" = "NavMesh pathfinding"; "neon_lights" = "Neon light effects"; "physics" = "Physics interaction"; "physics_benchmark" = "Physics stress test"; "picking" = "3D mouse picking"; "platformer" = "Physics-based platformer"; "pong" = "Classic pong game"; "prefabs" = "Prefab loading and instancing"; "psx" = "PS1-style rendering"; "render_layers" = "Render layer filtering"; "roguelike" = "Roguelike dungeon crawler"; "sdf_sculpt" = "SDF sculpting tool"; "sdf_text" = "SDF text rendering"; "shader_studio" = "Live shader editor"; "shadows" = "Shadow mapping"; "shell" = "Shell texturing demo"; "skybox" = "HDR skybox loading"; "sokoban" = "Sokoban puzzle game"; "space_shooter" = "Bullet hell shooter"; "speedreader" = "Speed reading trainer"; "spotlight_shadows" = "Spotlight shadow mapping"; "sprites" = "2D sprite rendering"; "ssao" = "Screen-space ambient occlusion"; "ssr" = "Screen-space reflections"; "survivors" = "Survivors-style action game"; "terrain" = "Infinite procedural terrain"; "text_adventure" = "Text adventure game"; "textures" = "Texture loading and display"; "topdown_shooter" = "Top-down twin-stick shooter"; "tower_defense" = "Tower defense with ECS"; "ui" = "UI widgets and layouts"; "village_survival" = "Village survival simulation"; "voxels" = "Voxel rendering"; "water" = "Water surface rendering"; "winter" = "Third-person character control" }; $example = (@("examples-2d", "examples-3d") | ForEach-Object { Get-ChildItem -Directory $_ } | ForEach-Object { $name = $_.Name; $desc = $descriptions[$name]; if ($desc) { "$name`t$desc" } else { $name } } | Sort-Object | fzf --prompt="Pick a demo> " --delimiter="`t" --with-nth=1.. --nth=1 --tabstop=30 | ForEach-Object { ($_ -split "`t")[0] }); if ($example) { cargo run -r -p $example }

# Interactively pick and run any example (2D or 3D)
[unix]
pick:
    #!/usr/bin/env bash
    set -euo pipefail
    declare -A descriptions=(
        ["alpha_blending"]="Alpha blending and transparency"
        ["asteroid_belt"]="Asteroid belt simulation"
        ["audio"]="Spatial audio playback"
        ["block_breaker"]="Block breaker arcade game"
        ["block_breaker_scripts"]="Block breaker with WASM scripts"
        ["bloom"]="Bloom post-processing"
        ["bunnymark"]="Sprite rendering benchmark"
        ["camera_in_camera"]="Nested camera rendering"
        ["castle_siege"]="GOAP AI castle siege"
        ["chess"]="3D chess game"
        ["chip8"]="Super CHIP-8 emulator"
        ["city"]="Procedural city generator"
        ["custom_multipass"]="Gaussian blur multipass"
        ["custom_pass"]="Custom render pass"
        ["dance"]="Skinned mesh benchmark"
        ["decals"]="Decal projection"
        ["depth_of_field"]="Depth of field effect"
        ["doom"]="Doom-style renderer"
        ["farming"]="Farming simulation"
        ["fireworks"]="Fireworks particle effects"
        ["genetic_walkers"]="Genetic algorithm walkers"
        ["gfx"]="2D shape primitives and beziers"
        ["gfx_showcase"]="2D shape primitives showcase"
        ["gizmo"]="Transform gizmo"
        ["hex_war"]="Hex-based strategy game"
        ["hiz"]="Hi-Z occlusion culling"
        ["horror"]="First-person horror demo"
        ["hud_text"]="HUD text rendering"
        ["immersive_sim"]="Immersive sim sandbox"
        ["interior_mapping"]="Interior mapping shader"
        ["jigsaw"]="Jigsaw puzzle game"
        ["lattice"]="Lattice deformation"
        ["lights"]="Clustered forward rendering"
        ["maps"]="Map rendering"
        ["menu"]="Menu system demo"
        ["morph"]="Morph targets animation"
        ["mosaic"]="Multi-window mosaic"
        ["multi_world"]="Multiple ECS worlds"
        ["navmesh"]="NavMesh pathfinding"
        ["neon_lights"]="Neon light effects"
        ["physics"]="Physics interaction"
        ["physics_benchmark"]="Physics stress test"
        ["picking"]="3D mouse picking"
        ["platformer"]="Physics-based platformer"
        ["pong"]="Classic pong game"
        ["prefabs"]="Prefab loading and instancing"
        ["psx"]="PS1-style rendering"
        ["render_layers"]="Render layer filtering"
        ["roguelike"]="Roguelike dungeon crawler"
        ["sdf_sculpt"]="SDF sculpting tool"
        ["sdf_text"]="SDF text rendering"
        ["shader_studio"]="Live shader editor"
        ["shadows"]="Shadow mapping"
        ["shell"]="Shell texturing demo"
        ["skybox"]="HDR skybox loading"
        ["sokoban"]="Sokoban puzzle game"
        ["space_shooter"]="Bullet hell shooter"
        ["speedreader"]="Speed reading trainer"
        ["spotlight_shadows"]="Spotlight shadow mapping"
        ["sprites"]="2D sprite rendering"
        ["ssao"]="Screen-space ambient occlusion"
        ["ssr"]="Screen-space reflections"
        ["survivors"]="Survivors-style action game"
        ["terrain"]="Infinite procedural terrain"
        ["text_adventure"]="Text adventure game"
        ["textures"]="Texture loading and display"
        ["topdown_shooter"]="Top-down twin-stick shooter"
        ["tower_defense"]="Tower defense with ECS"
        ["ui"]="UI widgets and layouts"
        ["village_survival"]="Village survival simulation"
        ["voxels"]="Voxel rendering"
        ["water"]="Water surface rendering"
        ["winter"]="Third-person character control"
    )
    items=""
    for dir in examples-2d/*/ examples-3d/*/; do
        name=$(basename "$dir")
        desc="${descriptions[$name]:-}"
        if [ -n "$desc" ]; then
            items+="${name}\t${desc}\n"
        else
            items+="${name}\n"
        fi
    done
    example=$(echo -e "$items" | sort | fzf --prompt="Pick a demo> " --delimiter='\t' --with-nth=1.. --nth=1 --tabstop=30 | cut -f1)
    [ -n "$example" ] && cargo run -r -p "$example"

# Interactively pick and run a 2D example
[windows]
pick-2d:
    $descriptions = @{ "bunnymark" = "Sprite rendering benchmark"; "chip8" = "Super CHIP-8 emulator"; "fireworks" = "Fireworks particle effects"; "gfx" = "2D shape primitives and beziers"; "gfx_showcase" = "2D shape primitives showcase"; "hud_text" = "HUD text rendering"; "roguelike" = "Roguelike dungeon crawler"; "sokoban" = "Sokoban puzzle game"; "space_shooter" = "Bullet hell shooter"; "sprites" = "2D sprite rendering"; "text_adventure" = "Text adventure game"; "topdown_shooter" = "Top-down twin-stick shooter"; "ui" = "UI widgets and layouts" }; $example = (Get-ChildItem -Directory "examples-2d" | ForEach-Object { $name = $_.Name; $desc = $descriptions[$name]; if ($desc) { "$name`t$desc" } else { $name } } | Sort-Object | fzf --prompt="Pick a 2D demo> " --delimiter="`t" --with-nth=1.. --nth=1 --tabstop=30 | ForEach-Object { ($_ -split "`t")[0] }); if ($example) { cargo run -r -p $example }

# Interactively pick and run a 2D example
[unix]
pick-2d:
    #!/usr/bin/env bash
    set -euo pipefail
    declare -A descriptions=(
        ["bunnymark"]="Sprite rendering benchmark"
        ["chip8"]="Super CHIP-8 emulator"
        ["fireworks"]="Fireworks particle effects"
        ["gfx"]="2D shape primitives and beziers"
        ["gfx_showcase"]="2D shape primitives showcase"
        ["hud_text"]="HUD text rendering"
        ["roguelike"]="Roguelike dungeon crawler"
        ["sokoban"]="Sokoban puzzle game"
        ["space_shooter"]="Bullet hell shooter"
        ["sprites"]="2D sprite rendering"
        ["text_adventure"]="Text adventure game"
        ["topdown_shooter"]="Top-down twin-stick shooter"
        ["ui"]="UI widgets and layouts"
    )
    items=""
    for dir in examples-2d/*/; do
        name=$(basename "$dir")
        desc="${descriptions[$name]:-}"
        if [ -n "$desc" ]; then
            items+="${name}\t${desc}\n"
        else
            items+="${name}\n"
        fi
    done
    example=$(echo -e "$items" | sort | fzf --prompt="Pick a 2D demo> " --delimiter='\t' --with-nth=1.. --nth=1 --tabstop=30 | cut -f1)
    [ -n "$example" ] && cargo run -r -p "$example"

# Interactively pick and run a 3D example
[windows]
pick-3d:
    $descriptions = @{ "alpha_blending" = "Alpha blending and transparency"; "asteroid_belt" = "Asteroid belt simulation"; "audio" = "Spatial audio playback"; "block_breaker" = "Block breaker arcade game"; "block_breaker_scripts" = "Block breaker with WASM scripts"; "bloom" = "Bloom post-processing"; "camera_in_camera" = "Nested camera rendering"; "castle_siege" = "GOAP AI castle siege"; "chess" = "3D chess game"; "city" = "Procedural city generator"; "custom_multipass" = "Gaussian blur multipass"; "custom_pass" = "Custom render pass"; "dance" = "Skinned mesh benchmark"; "decals" = "Decal projection"; "depth_of_field" = "Depth of field effect"; "doom" = "Doom-style renderer"; "farming" = "Farming simulation"; "genetic_walkers" = "Genetic algorithm walkers"; "gizmo" = "Transform gizmo"; "hex_war" = "Hex-based strategy game"; "hiz" = "Hi-Z occlusion culling"; "horror" = "First-person horror demo"; "immersive_sim" = "Immersive sim sandbox"; "interior_mapping" = "Interior mapping shader"; "jigsaw" = "Jigsaw puzzle game"; "lattice" = "Lattice deformation"; "lights" = "Clustered forward rendering"; "maps" = "Map rendering"; "menu" = "Menu system demo"; "morph" = "Morph targets animation"; "mosaic" = "Multi-window mosaic"; "multi_world" = "Multiple ECS worlds"; "navmesh" = "NavMesh pathfinding"; "neon_lights" = "Neon light effects"; "physics" = "Physics interaction"; "physics_benchmark" = "Physics stress test"; "picking" = "3D mouse picking"; "platformer" = "Physics-based platformer"; "pong" = "Classic pong game"; "prefabs" = "Prefab loading and instancing"; "psx" = "PS1-style rendering"; "render_layers" = "Render layer filtering"; "sdf_sculpt" = "SDF sculpting tool"; "sdf_text" = "SDF text rendering"; "shader_studio" = "Live shader editor"; "shadows" = "Shadow mapping"; "shell" = "Shell texturing demo"; "skybox" = "HDR skybox loading"; "speedreader" = "Speed reading trainer"; "spotlight_shadows" = "Spotlight shadow mapping"; "ssao" = "Screen-space ambient occlusion"; "ssr" = "Screen-space reflections"; "survivors" = "Survivors-style action game"; "terrain" = "Infinite procedural terrain"; "textures" = "Texture loading and display"; "tower_defense" = "Tower defense with ECS"; "village_survival" = "Village survival simulation"; "voxels" = "Voxel rendering"; "water" = "Water surface rendering"; "winter" = "Third-person character control" }; $example = (Get-ChildItem -Directory "examples-3d" | ForEach-Object { $name = $_.Name; $desc = $descriptions[$name]; if ($desc) { "$name`t$desc" } else { $name } } | Sort-Object | fzf --prompt="Pick a 3D demo> " --delimiter="`t" --with-nth=1.. --nth=1 --tabstop=30 | ForEach-Object { ($_ -split "`t")[0] }); if ($example) { cargo run -r -p $example }

# Interactively pick and run a 3D example
[unix]
pick-3d:
    #!/usr/bin/env bash
    set -euo pipefail
    declare -A descriptions=(
        ["alpha_blending"]="Alpha blending and transparency"
        ["asteroid_belt"]="Asteroid belt simulation"
        ["audio"]="Spatial audio playback"
        ["block_breaker"]="Block breaker arcade game"
        ["block_breaker_scripts"]="Block breaker with WASM scripts"
        ["bloom"]="Bloom post-processing"
        ["camera_in_camera"]="Nested camera rendering"
        ["castle_siege"]="GOAP AI castle siege"
        ["chess"]="3D chess game"
        ["city"]="Procedural city generator"
        ["custom_multipass"]="Gaussian blur multipass"
        ["custom_pass"]="Custom render pass"
        ["dance"]="Skinned mesh benchmark"
        ["decals"]="Decal projection"
        ["depth_of_field"]="Depth of field effect"
        ["doom"]="Doom-style renderer"
        ["farming"]="Farming simulation"
        ["genetic_walkers"]="Genetic algorithm walkers"
        ["gizmo"]="Transform gizmo"
        ["hex_war"]="Hex-based strategy game"
        ["hiz"]="Hi-Z occlusion culling"
        ["horror"]="First-person horror demo"
        ["immersive_sim"]="Immersive sim sandbox"
        ["interior_mapping"]="Interior mapping shader"
        ["jigsaw"]="Jigsaw puzzle game"
        ["lattice"]="Lattice deformation"
        ["lights"]="Clustered forward rendering"
        ["maps"]="Map rendering"
        ["menu"]="Menu system demo"
        ["morph"]="Morph targets animation"
        ["mosaic"]="Multi-window mosaic"
        ["multi_world"]="Multiple ECS worlds"
        ["navmesh"]="NavMesh pathfinding"
        ["neon_lights"]="Neon light effects"
        ["physics"]="Physics interaction"
        ["physics_benchmark"]="Physics stress test"
        ["picking"]="3D mouse picking"
        ["platformer"]="Physics-based platformer"
        ["pong"]="Classic pong game"
        ["prefabs"]="Prefab loading and instancing"
        ["psx"]="PS1-style rendering"
        ["render_layers"]="Render layer filtering"
        ["sdf_sculpt"]="SDF sculpting tool"
        ["sdf_text"]="SDF text rendering"
        ["shader_studio"]="Live shader editor"
        ["shadows"]="Shadow mapping"
        ["shell"]="Shell texturing demo"
        ["skybox"]="HDR skybox loading"
        ["speedreader"]="Speed reading trainer"
        ["spotlight_shadows"]="Spotlight shadow mapping"
        ["ssao"]="Screen-space ambient occlusion"
        ["ssr"]="Screen-space reflections"
        ["survivors"]="Survivors-style action game"
        ["terrain"]="Infinite procedural terrain"
        ["textures"]="Texture loading and display"
        ["tower_defense"]="Tower defense with ECS"
        ["village_survival"]="Village survival simulation"
        ["voxels"]="Voxel rendering"
        ["water"]="Water surface rendering"
        ["winter"]="Third-person character control"
    )
    items=""
    for dir in examples-3d/*/; do
        name=$(basename "$dir")
        desc="${descriptions[$name]:-}"
        if [ -n "$desc" ]; then
            items+="${name}\t${desc}\n"
        else
            items+="${name}\n"
        fi
    done
    example=$(echo -e "$items" | sort | fzf --prompt="Pick a 3D demo> " --delimiter='\t' --with-nth=1.. --nth=1 --tabstop=30 | cut -f1)
    [ -n "$example" ] && cargo run -r -p "$example"

# Interactively pick and serve an example in browser
[windows]
pick-wasm:
    $descriptions = @{ "alpha_blending" = "Alpha blending and transparency"; "asteroid_belt" = "Asteroid belt simulation"; "audio" = "Spatial audio playback"; "block_breaker" = "Block breaker arcade game"; "block_breaker_scripts" = "Block breaker with WASM scripts"; "bloom" = "Bloom post-processing"; "bunnymark" = "Sprite rendering benchmark"; "camera_in_camera" = "Nested camera rendering"; "castle_siege" = "GOAP AI castle siege"; "chess" = "3D chess game"; "chip8" = "Super CHIP-8 emulator"; "city" = "Procedural city generator"; "custom_multipass" = "Gaussian blur multipass"; "custom_pass" = "Custom render pass"; "dance" = "Skinned mesh benchmark"; "decals" = "Decal projection"; "depth_of_field" = "Depth of field effect"; "doom" = "Doom-style renderer"; "farming" = "Farming simulation"; "fireworks" = "Fireworks particle effects"; "genetic_walkers" = "Genetic algorithm walkers"; "gfx" = "2D shape primitives and beziers"; "gfx_showcase" = "2D shape primitives showcase"; "gizmo" = "Transform gizmo"; "hex_war" = "Hex-based strategy game"; "hiz" = "Hi-Z occlusion culling"; "horror" = "First-person horror demo"; "hud_text" = "HUD text rendering"; "immersive_sim" = "Immersive sim sandbox"; "interior_mapping" = "Interior mapping shader"; "jigsaw" = "Jigsaw puzzle game"; "lattice" = "Lattice deformation"; "lights" = "Clustered forward rendering"; "maps" = "Map rendering"; "menu" = "Menu system demo"; "morph" = "Morph targets animation"; "mosaic" = "Multi-window mosaic"; "multi_world" = "Multiple ECS worlds"; "navmesh" = "NavMesh pathfinding"; "neon_lights" = "Neon light effects"; "physics" = "Physics interaction"; "physics_benchmark" = "Physics stress test"; "picking" = "3D mouse picking"; "platformer" = "Physics-based platformer"; "pong" = "Classic pong game"; "prefabs" = "Prefab loading and instancing"; "psx" = "PS1-style rendering"; "render_layers" = "Render layer filtering"; "roguelike" = "Roguelike dungeon crawler"; "sdf_sculpt" = "SDF sculpting tool"; "sdf_text" = "SDF text rendering"; "shader_studio" = "Live shader editor"; "shadows" = "Shadow mapping"; "shell" = "Shell texturing demo"; "skybox" = "HDR skybox loading"; "sokoban" = "Sokoban puzzle game"; "space_shooter" = "Bullet hell shooter"; "speedreader" = "Speed reading trainer"; "spotlight_shadows" = "Spotlight shadow mapping"; "sprites" = "2D sprite rendering"; "ssao" = "Screen-space ambient occlusion"; "ssr" = "Screen-space reflections"; "survivors" = "Survivors-style action game"; "terrain" = "Infinite procedural terrain"; "text_adventure" = "Text adventure game"; "textures" = "Texture loading and display"; "topdown_shooter" = "Top-down twin-stick shooter"; "tower_defense" = "Tower defense with ECS"; "ui" = "UI widgets and layouts"; "village_survival" = "Village survival simulation"; "voxels" = "Voxel rendering"; "water" = "Water surface rendering"; "winter" = "Third-person character control" }; $example = (@("examples-2d", "examples-3d") | ForEach-Object { Get-ChildItem -Directory $_ } | ForEach-Object { $name = $_.Name; $desc = $descriptions[$name]; if ($desc) { "$name`t$desc" } else { $name } } | Sort-Object | fzf --prompt="Pick a WASM demo> " --delimiter="`t" --with-nth=1.. --nth=1 --tabstop=30 | ForEach-Object { ($_ -split "`t")[0] }); if ($example) { $config = @("examples-2d", "examples-3d") | ForEach-Object { "$_/$example/Trunk.toml" } | Where-Object { Test-Path $_ } | Select-Object -First 1; if ($config) { trunk serve --release --open --config $config } }

# Interactively pick and serve an example in browser
[unix]
pick-wasm:
    #!/usr/bin/env bash
    set -euo pipefail
    declare -A descriptions=(
        ["alpha_blending"]="Alpha blending and transparency"
        ["asteroid_belt"]="Asteroid belt simulation"
        ["audio"]="Spatial audio playback"
        ["block_breaker"]="Block breaker arcade game"
        ["block_breaker_scripts"]="Block breaker with WASM scripts"
        ["bloom"]="Bloom post-processing"
        ["bunnymark"]="Sprite rendering benchmark"
        ["camera_in_camera"]="Nested camera rendering"
        ["castle_siege"]="GOAP AI castle siege"
        ["chess"]="3D chess game"
        ["chip8"]="Super CHIP-8 emulator"
        ["city"]="Procedural city generator"
        ["custom_multipass"]="Gaussian blur multipass"
        ["custom_pass"]="Custom render pass"
        ["dance"]="Skinned mesh benchmark"
        ["decals"]="Decal projection"
        ["depth_of_field"]="Depth of field effect"
        ["doom"]="Doom-style renderer"
        ["farming"]="Farming simulation"
        ["fireworks"]="Fireworks particle effects"
        ["genetic_walkers"]="Genetic algorithm walkers"
        ["gfx"]="2D shape primitives and beziers"
        ["gfx_showcase"]="2D shape primitives showcase"
        ["gizmo"]="Transform gizmo"
        ["hex_war"]="Hex-based strategy game"
        ["hiz"]="Hi-Z occlusion culling"
        ["horror"]="First-person horror demo"
        ["hud_text"]="HUD text rendering"
        ["immersive_sim"]="Immersive sim sandbox"
        ["interior_mapping"]="Interior mapping shader"
        ["jigsaw"]="Jigsaw puzzle game"
        ["lattice"]="Lattice deformation"
        ["lights"]="Clustered forward rendering"
        ["maps"]="Map rendering"
        ["menu"]="Menu system demo"
        ["morph"]="Morph targets animation"
        ["mosaic"]="Multi-window mosaic"
        ["multi_world"]="Multiple ECS worlds"
        ["navmesh"]="NavMesh pathfinding"
        ["neon_lights"]="Neon light effects"
        ["physics"]="Physics interaction"
        ["physics_benchmark"]="Physics stress test"
        ["picking"]="3D mouse picking"
        ["platformer"]="Physics-based platformer"
        ["pong"]="Classic pong game"
        ["prefabs"]="Prefab loading and instancing"
        ["psx"]="PS1-style rendering"
        ["render_layers"]="Render layer filtering"
        ["roguelike"]="Roguelike dungeon crawler"
        ["sdf_sculpt"]="SDF sculpting tool"
        ["sdf_text"]="SDF text rendering"
        ["shader_studio"]="Live shader editor"
        ["shadows"]="Shadow mapping"
        ["shell"]="Shell texturing demo"
        ["skybox"]="HDR skybox loading"
        ["sokoban"]="Sokoban puzzle game"
        ["space_shooter"]="Bullet hell shooter"
        ["speedreader"]="Speed reading trainer"
        ["spotlight_shadows"]="Spotlight shadow mapping"
        ["sprites"]="2D sprite rendering"
        ["ssao"]="Screen-space ambient occlusion"
        ["ssr"]="Screen-space reflections"
        ["survivors"]="Survivors-style action game"
        ["terrain"]="Infinite procedural terrain"
        ["text_adventure"]="Text adventure game"
        ["textures"]="Texture loading and display"
        ["topdown_shooter"]="Top-down twin-stick shooter"
        ["tower_defense"]="Tower defense with ECS"
        ["ui"]="UI widgets and layouts"
        ["village_survival"]="Village survival simulation"
        ["voxels"]="Voxel rendering"
        ["water"]="Water surface rendering"
        ["winter"]="Third-person character control"
    )
    items=""
    for dir in examples-2d/*/ examples-3d/*/; do
        name=$(basename "$dir")
        desc="${descriptions[$name]:-}"
        if [ -n "$desc" ]; then
            items+="${name}\t${desc}\n"
        else
            items+="${name}\n"
        fi
    done
    example=$(echo -e "$items" | sort | fzf --prompt="Pick a WASM demo> " --delimiter='\t' --with-nth=1.. --nth=1 --tabstop=30 | cut -f1)
    if [ -n "$example" ]; then
        config=$(find examples-2d examples-3d -maxdepth 2 -path "*/$example/Trunk.toml" 2>/dev/null | head -1)
        [ -n "$config" ] && trunk serve --release --open --config "$config"
    fi

# Interactively pick and run a native-only example
[windows]
pick-native:
    $descriptions = @{ "assets" = "3D model and asset viewer"; "claude_chat" = "Claude chat with egui UI"; "claude_chat_leptos" = "Claude chat with Leptos webview"; "claude_chat_leptos_mcp" = "Claude chat with Leptos + MCP tools"; "claude_chat_mcp" = "Claude chat with MCP scene tools"; "demo_scene" = "Visual effects and lighting demo"; "mcp" = "MCP model viewer"; "multiplayer_pong" = "Networked multiplayer pong"; "plugins" = "WASM plugin system demo"; "steam" = "Steam integration demo"; "webview" = "Embedded webview demo" }; $example = (Get-ChildItem -Directory "examples-native" | ForEach-Object { $name = $_.Name; $desc = $descriptions[$name]; if ($desc) { "$name`t$desc" } else { $name } } | Sort-Object | fzf --prompt="Pick a native demo> " --delimiter="`t" --with-nth=1.. --nth=1 --tabstop=30 | ForEach-Object { ($_ -split "`t")[0] }); if ($example) { if (Test-Path "examples-native/$example/justfile") { Push-Location "examples-native/$example"; just run; Pop-Location } else { cargo run -r -p $example } }

# Interactively pick and run a native-only example
[unix]
pick-native:
    #!/usr/bin/env bash
    set -euo pipefail
    declare -A descriptions=(
        ["assets"]="3D model and asset viewer"
        ["claude_chat"]="Claude chat with egui UI"
        ["claude_chat_leptos"]="Claude chat with Leptos webview"
        ["claude_chat_leptos_mcp"]="Claude chat with Leptos + MCP tools"
        ["claude_chat_mcp"]="Claude chat with MCP scene tools"
        ["demo_scene"]="Visual effects and lighting demo"
        ["mcp"]="MCP model viewer"
        ["multiplayer_pong"]="Networked multiplayer pong"
        ["plugins"]="WASM plugin system demo"
        ["steam"]="Steam integration demo"
        ["webview"]="Embedded webview demo"
    )
    items=""
    for dir in examples-native/*/; do
        name=$(basename "$dir")
        desc="${descriptions[$name]:-}"
        if [ -n "$desc" ]; then
            items+="${name}\t${desc}\n"
        else
            items+="${name}\n"
        fi
    done
    example=$(echo -e "$items" | sort | fzf --prompt="Pick a native demo> " --delimiter='\t' --with-nth=1.. --nth=1 --tabstop=30 | cut -f1)
    if [ -n "$example" ]; then
        if [ -f "examples-native/$example/justfile" ]; then
            cd "examples-native/$example" && just run
        else
            cargo run -r -p "$example"
        fi
    fi

# Interactively pick and run a terminal example (in terminal)
[windows]
pick-terminal:
    $descriptions = @{ "asteroids" = "Classic asteroids arcade"; "bomberman" = "Bomberman grid game"; "bouncing_balls" = "Physics bouncing simulation"; "breakout" = "Breakout paddle game"; "bullet_hell" = "Bullet hell shooter"; "colony" = "Colony management sim"; "deck_builder" = "Deck-building card game"; "falling_sand" = "Falling sand simulation"; "flappy_bird" = "Flappy bird clone"; "frogger" = "Frogger road crossing"; "game_of_life" = "Conway's Game of Life"; "hex_strategy" = "Hex-based strategy"; "lemonade" = "Lemonade stand business sim"; "match3" = "Match-3 puzzle game"; "minesweeper" = "Minesweeper"; "pacman" = "Pac-Man maze game"; "pathfinding_demo" = "Pathfinding visualization"; "puzzle_2048" = "2048 number puzzle"; "rhythm" = "Rhythm music game"; "snake" = "Classic snake game"; "solitaire" = "Klondike solitaire"; "space_invaders" = "Space invaders arcade"; "terminal_chess" = "Chess with AI"; "terminal_doom" = "Doom-style FPS"; "terminal_platformer" = "Side-scrolling platformer"; "terminal_pong" = "Classic pong"; "terminal_roguelike" = "Roguelike dungeon crawler"; "terminal_sokoban" = "Sokoban box puzzles"; "terminal_text_adventure" = "Text adventure RPG"; "terminal_tower_defense" = "Tower defense strategy"; "tetris" = "Classic tetris"; "typing_game" = "Typing speed game"; "wasteland" = "Post-apocalyptic survival"; "wordle" = "Wordle word guessing" }; $example = (Get-ChildItem -Directory "examples-terminal" | ForEach-Object { $name = $_.Name; $desc = $descriptions[$name]; if ($desc) { "$name`t$desc" } else { $name } } | Sort-Object | fzf --prompt="Pick a terminal demo> " --delimiter="`t" --with-nth=1.. --nth=1 --tabstop=30 | ForEach-Object { ($_ -split "`t")[0] }); if ($example) { cargo run -r -p $example }

# Interactively pick and run a terminal example (in terminal)
[unix]
pick-terminal:
    #!/usr/bin/env bash
    set -euo pipefail
    declare -A descriptions=(
        ["asteroids"]="Classic asteroids arcade"
        ["bomberman"]="Bomberman grid game"
        ["bouncing_balls"]="Physics bouncing simulation"
        ["breakout"]="Breakout paddle game"
        ["bullet_hell"]="Bullet hell shooter"
        ["colony"]="Colony management sim"
        ["deck_builder"]="Deck-building card game"
        ["falling_sand"]="Falling sand simulation"
        ["flappy_bird"]="Flappy bird clone"
        ["frogger"]="Frogger road crossing"
        ["game_of_life"]="Conway's Game of Life"
        ["hex_strategy"]="Hex-based strategy"
        ["lemonade"]="Lemonade stand business sim"
        ["match3"]="Match-3 puzzle game"
        ["minesweeper"]="Minesweeper"
        ["pacman"]="Pac-Man maze game"
        ["pathfinding_demo"]="Pathfinding visualization"
        ["puzzle_2048"]="2048 number puzzle"
        ["rhythm"]="Rhythm music game"
        ["snake"]="Classic snake game"
        ["solitaire"]="Klondike solitaire"
        ["space_invaders"]="Space invaders arcade"
        ["terminal_chess"]="Chess with AI"
        ["terminal_doom"]="Doom-style FPS"
        ["terminal_platformer"]="Side-scrolling platformer"
        ["terminal_pong"]="Classic pong"
        ["terminal_roguelike"]="Roguelike dungeon crawler"
        ["terminal_sokoban"]="Sokoban box puzzles"
        ["terminal_text_adventure"]="Text adventure RPG"
        ["terminal_tower_defense"]="Tower defense strategy"
        ["tetris"]="Classic tetris"
        ["typing_game"]="Typing speed game"
        ["wasteland"]="Post-apocalyptic survival"
        ["wordle"]="Wordle word guessing"
    )
    items=""
    for dir in examples-terminal/*/; do
        name=$(basename "$dir")
        desc="${descriptions[$name]:-}"
        if [ -n "$desc" ]; then
            items+="${name}\t${desc}\n"
        else
            items+="${name}\n"
        fi
    done
    example=$(echo -e "$items" | sort | fzf --prompt="Pick a terminal demo> " --delimiter='\t' --with-nth=1.. --nth=1 --tabstop=30 | cut -f1)
    [ -n "$example" ] && cargo run -r -p "$example"

# Interactively pick and run a terminal example in TUI mode (windowed)
[windows]
pick-tui:
    $descriptions = @{ "asteroids" = "Classic asteroids arcade"; "bomberman" = "Bomberman grid game"; "bouncing_balls" = "Physics bouncing simulation"; "breakout" = "Breakout paddle game"; "bullet_hell" = "Bullet hell shooter"; "colony" = "Colony management sim"; "deck_builder" = "Deck-building card game"; "falling_sand" = "Falling sand simulation"; "flappy_bird" = "Flappy bird clone"; "frogger" = "Frogger road crossing"; "game_of_life" = "Conway's Game of Life"; "hex_strategy" = "Hex-based strategy"; "lemonade" = "Lemonade stand business sim"; "match3" = "Match-3 puzzle game"; "minesweeper" = "Minesweeper"; "pacman" = "Pac-Man maze game"; "pathfinding_demo" = "Pathfinding visualization"; "puzzle_2048" = "2048 number puzzle"; "rhythm" = "Rhythm music game"; "snake" = "Classic snake game"; "solitaire" = "Klondike solitaire"; "space_invaders" = "Space invaders arcade"; "terminal_chess" = "Chess with AI"; "terminal_doom" = "Doom-style FPS"; "terminal_platformer" = "Side-scrolling platformer"; "terminal_pong" = "Classic pong"; "terminal_roguelike" = "Roguelike dungeon crawler"; "terminal_sokoban" = "Sokoban box puzzles"; "terminal_text_adventure" = "Text adventure RPG"; "terminal_tower_defense" = "Tower defense strategy"; "tetris" = "Classic tetris"; "typing_game" = "Typing speed game"; "wasteland" = "Post-apocalyptic survival"; "wordle" = "Wordle word guessing" }; $example = (Get-ChildItem -Directory "examples-terminal" | ForEach-Object { $name = $_.Name; $desc = $descriptions[$name]; if ($desc) { "$name`t$desc" } else { $name } } | Sort-Object | fzf --prompt="Pick a TUI demo> " --delimiter="`t" --with-nth=1.. --nth=1 --tabstop=30 | ForEach-Object { ($_ -split "`t")[0] }); if ($example) { cargo run -r -p $example --no-default-features --features tui }

# Interactively pick and run a terminal example in TUI mode (windowed)
[unix]
pick-tui:
    #!/usr/bin/env bash
    set -euo pipefail
    declare -A descriptions=(
        ["asteroids"]="Classic asteroids arcade"
        ["bomberman"]="Bomberman grid game"
        ["bouncing_balls"]="Physics bouncing simulation"
        ["breakout"]="Breakout paddle game"
        ["bullet_hell"]="Bullet hell shooter"
        ["colony"]="Colony management sim"
        ["deck_builder"]="Deck-building card game"
        ["falling_sand"]="Falling sand simulation"
        ["flappy_bird"]="Flappy bird clone"
        ["frogger"]="Frogger road crossing"
        ["game_of_life"]="Conway's Game of Life"
        ["hex_strategy"]="Hex-based strategy"
        ["lemonade"]="Lemonade stand business sim"
        ["match3"]="Match-3 puzzle game"
        ["minesweeper"]="Minesweeper"
        ["pacman"]="Pac-Man maze game"
        ["pathfinding_demo"]="Pathfinding visualization"
        ["puzzle_2048"]="2048 number puzzle"
        ["rhythm"]="Rhythm music game"
        ["snake"]="Classic snake game"
        ["solitaire"]="Klondike solitaire"
        ["space_invaders"]="Space invaders arcade"
        ["terminal_chess"]="Chess with AI"
        ["terminal_doom"]="Doom-style FPS"
        ["terminal_platformer"]="Side-scrolling platformer"
        ["terminal_pong"]="Classic pong"
        ["terminal_roguelike"]="Roguelike dungeon crawler"
        ["terminal_sokoban"]="Sokoban box puzzles"
        ["terminal_text_adventure"]="Text adventure RPG"
        ["terminal_tower_defense"]="Tower defense strategy"
        ["tetris"]="Classic tetris"
        ["typing_game"]="Typing speed game"
        ["wasteland"]="Post-apocalyptic survival"
        ["wordle"]="Wordle word guessing"
    )
    items=""
    for dir in examples-terminal/*/; do
        name=$(basename "$dir")
        desc="${descriptions[$name]:-}"
        if [ -n "$desc" ]; then
            items+="${name}\t${desc}\n"
        else
            items+="${name}\n"
        fi
    done
    example=$(echo -e "$items" | sort | fzf --prompt="Pick a TUI demo> " --delimiter='\t' --with-nth=1.. --nth=1 --tabstop=30 | cut -f1)
    [ -n "$example" ] && cargo run -r -p "$example" --no-default-features --features tui

# Interactively pick and serve a terminal example in browser
[windows]
pick-wasm-tui:
    $descriptions = @{ "asteroids" = "Classic asteroids arcade"; "bomberman" = "Bomberman grid game"; "bouncing_balls" = "Physics bouncing simulation"; "breakout" = "Breakout paddle game"; "bullet_hell" = "Bullet hell shooter"; "colony" = "Colony management sim"; "deck_builder" = "Deck-building card game"; "falling_sand" = "Falling sand simulation"; "flappy_bird" = "Flappy bird clone"; "frogger" = "Frogger road crossing"; "game_of_life" = "Conway's Game of Life"; "hex_strategy" = "Hex-based strategy"; "lemonade" = "Lemonade stand business sim"; "match3" = "Match-3 puzzle game"; "minesweeper" = "Minesweeper"; "pacman" = "Pac-Man maze game"; "pathfinding_demo" = "Pathfinding visualization"; "puzzle_2048" = "2048 number puzzle"; "rhythm" = "Rhythm music game"; "snake" = "Classic snake game"; "solitaire" = "Klondike solitaire"; "space_invaders" = "Space invaders arcade"; "terminal_chess" = "Chess with AI"; "terminal_doom" = "Doom-style FPS"; "terminal_platformer" = "Side-scrolling platformer"; "terminal_pong" = "Classic pong"; "terminal_roguelike" = "Roguelike dungeon crawler"; "terminal_sokoban" = "Sokoban box puzzles"; "terminal_text_adventure" = "Text adventure RPG"; "terminal_tower_defense" = "Tower defense strategy"; "tetris" = "Classic tetris"; "typing_game" = "Typing speed game"; "wasteland" = "Post-apocalyptic survival"; "wordle" = "Wordle word guessing" }; $example = (Get-ChildItem -Directory "examples-terminal" | ForEach-Object { $name = $_.Name; $desc = $descriptions[$name]; if ($desc) { "$name`t$desc" } else { $name } } | Sort-Object | fzf --prompt="Pick a WASM TUI demo> " --delimiter="`t" --with-nth=1.. --nth=1 --tabstop=30 | ForEach-Object { ($_ -split "`t")[0] }); if ($example) { trunk serve --release --open --config "examples-terminal/$example/Trunk.toml" }

# Interactively pick and serve a terminal example in browser
[unix]
pick-wasm-tui:
    #!/usr/bin/env bash
    set -euo pipefail
    declare -A descriptions=(
        ["asteroids"]="Classic asteroids arcade"
        ["bomberman"]="Bomberman grid game"
        ["bouncing_balls"]="Physics bouncing simulation"
        ["breakout"]="Breakout paddle game"
        ["bullet_hell"]="Bullet hell shooter"
        ["colony"]="Colony management sim"
        ["deck_builder"]="Deck-building card game"
        ["falling_sand"]="Falling sand simulation"
        ["flappy_bird"]="Flappy bird clone"
        ["frogger"]="Frogger road crossing"
        ["game_of_life"]="Conway's Game of Life"
        ["hex_strategy"]="Hex-based strategy"
        ["lemonade"]="Lemonade stand business sim"
        ["match3"]="Match-3 puzzle game"
        ["minesweeper"]="Minesweeper"
        ["pacman"]="Pac-Man maze game"
        ["pathfinding_demo"]="Pathfinding visualization"
        ["puzzle_2048"]="2048 number puzzle"
        ["rhythm"]="Rhythm music game"
        ["snake"]="Classic snake game"
        ["solitaire"]="Klondike solitaire"
        ["space_invaders"]="Space invaders arcade"
        ["terminal_chess"]="Chess with AI"
        ["terminal_doom"]="Doom-style FPS"
        ["terminal_platformer"]="Side-scrolling platformer"
        ["terminal_pong"]="Classic pong"
        ["terminal_roguelike"]="Roguelike dungeon crawler"
        ["terminal_sokoban"]="Sokoban box puzzles"
        ["terminal_text_adventure"]="Text adventure RPG"
        ["terminal_tower_defense"]="Tower defense strategy"
        ["tetris"]="Classic tetris"
        ["typing_game"]="Typing speed game"
        ["wasteland"]="Post-apocalyptic survival"
        ["wordle"]="Wordle word guessing"
    )
    items=""
    for dir in examples-terminal/*/; do
        name=$(basename "$dir")
        desc="${descriptions[$name]:-}"
        if [ -n "$desc" ]; then
            items+="${name}\t${desc}\n"
        else
            items+="${name}\n"
        fi
    done
    example=$(echo -e "$items" | sort | fzf --prompt="Pick a WASM TUI demo> " --delimiter='\t' --with-nth=1.. --nth=1 --tabstop=30 | cut -f1)
    [ -n "$example" ] && trunk serve --release --open --config "examples-terminal/$example/Trunk.toml"

# Interactively pick an example and generate a snapshot
[windows]
pick-snapshot:
    $descriptions = @{ "alpha_blending" = "Alpha blending and transparency"; "asteroid_belt" = "Asteroid belt simulation"; "audio" = "Spatial audio playback"; "block_breaker" = "Block breaker arcade game"; "block_breaker_scripts" = "Block breaker with WASM scripts"; "bloom" = "Bloom post-processing"; "bunnymark" = "Sprite rendering benchmark"; "camera_in_camera" = "Nested camera rendering"; "castle_siege" = "GOAP AI castle siege"; "chess" = "3D chess game"; "chip8" = "Super CHIP-8 emulator"; "city" = "Procedural city generator"; "custom_multipass" = "Gaussian blur multipass"; "custom_pass" = "Custom render pass"; "dance" = "Skinned mesh benchmark"; "decals" = "Decal projection"; "depth_of_field" = "Depth of field effect"; "doom" = "Doom-style renderer"; "farming" = "Farming simulation"; "fireworks" = "Fireworks particle effects"; "genetic_walkers" = "Genetic algorithm walkers"; "gfx" = "2D shape primitives and beziers"; "gfx_showcase" = "2D shape primitives showcase"; "gizmo" = "Transform gizmo"; "hex_war" = "Hex-based strategy game"; "hiz" = "Hi-Z occlusion culling"; "horror" = "First-person horror demo"; "hud_text" = "HUD text rendering"; "immersive_sim" = "Immersive sim sandbox"; "interior_mapping" = "Interior mapping shader"; "jigsaw" = "Jigsaw puzzle game"; "lattice" = "Lattice deformation"; "lights" = "Clustered forward rendering"; "maps" = "Map rendering"; "menu" = "Menu system demo"; "morph" = "Morph targets animation"; "mosaic" = "Multi-window mosaic"; "multi_world" = "Multiple ECS worlds"; "navmesh" = "NavMesh pathfinding"; "neon_lights" = "Neon light effects"; "physics" = "Physics interaction"; "physics_benchmark" = "Physics stress test"; "picking" = "3D mouse picking"; "platformer" = "Physics-based platformer"; "pong" = "Classic pong game"; "prefabs" = "Prefab loading and instancing"; "psx" = "PS1-style rendering"; "render_layers" = "Render layer filtering"; "roguelike" = "Roguelike dungeon crawler"; "sdf_sculpt" = "SDF sculpting tool"; "sdf_text" = "SDF text rendering"; "shader_studio" = "Live shader editor"; "shadows" = "Shadow mapping"; "shell" = "Shell texturing demo"; "skybox" = "HDR skybox loading"; "sokoban" = "Sokoban puzzle game"; "space_shooter" = "Bullet hell shooter"; "speedreader" = "Speed reading trainer"; "spotlight_shadows" = "Spotlight shadow mapping"; "sprites" = "2D sprite rendering"; "ssao" = "Screen-space ambient occlusion"; "ssr" = "Screen-space reflections"; "survivors" = "Survivors-style action game"; "terrain" = "Infinite procedural terrain"; "text_adventure" = "Text adventure game"; "textures" = "Texture loading and display"; "topdown_shooter" = "Top-down twin-stick shooter"; "tower_defense" = "Tower defense with ECS"; "ui" = "UI widgets and layouts"; "village_survival" = "Village survival simulation"; "voxels" = "Voxel rendering"; "water" = "Water surface rendering"; "winter" = "Third-person character control" }; $example = (@("examples-2d", "examples-3d") | ForEach-Object { Get-ChildItem -Directory $_ } | ForEach-Object { $name = $_.Name; $desc = $descriptions[$name]; if ($desc) { "$name`t$desc" } else { $name } } | Sort-Object | fzf --prompt="Pick a demo to snapshot> " --delimiter="`t" --with-nth=1.. --nth=1 --tabstop=30 | ForEach-Object { ($_ -split "`t")[0] }); if ($example) { New-Item -ItemType Directory -Force -Path "snapshots" | Out-Null; $env:NIGHTSHADE_SNAPSHOT_PATH = "snapshots/$example.png"; cargo run -r -p $example }

# Interactively pick an example and generate a snapshot
[unix]
pick-snapshot:
    #!/usr/bin/env bash
    set -euo pipefail
    declare -A descriptions=(
        ["alpha_blending"]="Alpha blending and transparency"
        ["asteroid_belt"]="Asteroid belt simulation"
        ["audio"]="Spatial audio playback"
        ["block_breaker"]="Block breaker arcade game"
        ["block_breaker_scripts"]="Block breaker with WASM scripts"
        ["bloom"]="Bloom post-processing"
        ["bunnymark"]="Sprite rendering benchmark"
        ["camera_in_camera"]="Nested camera rendering"
        ["castle_siege"]="GOAP AI castle siege"
        ["chess"]="3D chess game"
        ["chip8"]="Super CHIP-8 emulator"
        ["city"]="Procedural city generator"
        ["custom_multipass"]="Gaussian blur multipass"
        ["custom_pass"]="Custom render pass"
        ["dance"]="Skinned mesh benchmark"
        ["decals"]="Decal projection"
        ["depth_of_field"]="Depth of field effect"
        ["doom"]="Doom-style renderer"
        ["farming"]="Farming simulation"
        ["fireworks"]="Fireworks particle effects"
        ["genetic_walkers"]="Genetic algorithm walkers"
        ["gfx"]="2D shape primitives and beziers"
        ["gfx_showcase"]="2D shape primitives showcase"
        ["gizmo"]="Transform gizmo"
        ["hex_war"]="Hex-based strategy game"
        ["hiz"]="Hi-Z occlusion culling"
        ["horror"]="First-person horror demo"
        ["hud_text"]="HUD text rendering"
        ["immersive_sim"]="Immersive sim sandbox"
        ["interior_mapping"]="Interior mapping shader"
        ["jigsaw"]="Jigsaw puzzle game"
        ["lattice"]="Lattice deformation"
        ["lights"]="Clustered forward rendering"
        ["maps"]="Map rendering"
        ["menu"]="Menu system demo"
        ["morph"]="Morph targets animation"
        ["mosaic"]="Multi-window mosaic"
        ["multi_world"]="Multiple ECS worlds"
        ["navmesh"]="NavMesh pathfinding"
        ["neon_lights"]="Neon light effects"
        ["physics"]="Physics interaction"
        ["physics_benchmark"]="Physics stress test"
        ["picking"]="3D mouse picking"
        ["platformer"]="Physics-based platformer"
        ["pong"]="Classic pong game"
        ["prefabs"]="Prefab loading and instancing"
        ["psx"]="PS1-style rendering"
        ["render_layers"]="Render layer filtering"
        ["roguelike"]="Roguelike dungeon crawler"
        ["sdf_sculpt"]="SDF sculpting tool"
        ["sdf_text"]="SDF text rendering"
        ["shader_studio"]="Live shader editor"
        ["shadows"]="Shadow mapping"
        ["shell"]="Shell texturing demo"
        ["skybox"]="HDR skybox loading"
        ["sokoban"]="Sokoban puzzle game"
        ["space_shooter"]="Bullet hell shooter"
        ["speedreader"]="Speed reading trainer"
        ["spotlight_shadows"]="Spotlight shadow mapping"
        ["sprites"]="2D sprite rendering"
        ["ssao"]="Screen-space ambient occlusion"
        ["ssr"]="Screen-space reflections"
        ["survivors"]="Survivors-style action game"
        ["terrain"]="Infinite procedural terrain"
        ["text_adventure"]="Text adventure game"
        ["textures"]="Texture loading and display"
        ["topdown_shooter"]="Top-down twin-stick shooter"
        ["tower_defense"]="Tower defense with ECS"
        ["ui"]="UI widgets and layouts"
        ["village_survival"]="Village survival simulation"
        ["voxels"]="Voxel rendering"
        ["water"]="Water surface rendering"
        ["winter"]="Third-person character control"
    )
    items=""
    for dir in examples-2d/*/ examples-3d/*/; do
        name=$(basename "$dir")
        desc="${descriptions[$name]:-}"
        if [ -n "$desc" ]; then
            items+="${name}\t${desc}\n"
        else
            items+="${name}\n"
        fi
    done
    example=$(echo -e "$items" | sort | fzf --prompt="Pick a demo to snapshot> " --delimiter='\t' --with-nth=1.. --nth=1 --tabstop=30 | cut -f1)
    if [ -n "$example" ]; then
        mkdir -p snapshots
        NIGHTSHADE_SNAPSHOT_PATH="snapshots/$example.png" cargo run -r -p "$example"
    fi

# Generate snapshots for all examples
[windows]
generate-snapshots:
    $ErrorActionPreference = "Stop"; New-Item -ItemType Directory -Force -Path "snapshots" | Out-Null; $examples = @("examples-2d", "examples-3d") | ForEach-Object { Get-ChildItem -Directory $_ } | ForEach-Object { $_.Name } | Sort-Object; $total = $examples.Count; $current = 0; foreach ($example in $examples) { $current++; Write-Host "[$current/$total] Snapshotting $example..." -ForegroundColor Cyan; $env:NIGHTSHADE_SNAPSHOT_PATH = "snapshots/$example.png"; cargo run -r -p $example; }; Write-Host "All snapshots complete! Output in snapshots/" -ForegroundColor Green

# Generate snapshots for all examples
[unix]
generate-snapshots:
    #!/usr/bin/env bash
    set -euo pipefail
    mkdir -p snapshots
    examples=($(ls -d examples-2d/*/ examples-3d/*/ 2>/dev/null | xargs -I{} basename {} | sort))
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
[windows]
build-wasm $example="alpha_blending":
    $config = @("examples-2d", "examples-3d") | ForEach-Object { "$_/$env:example/Trunk.toml" } | Where-Object { Test-Path $_ } | Select-Object -First 1; if ($config) { trunk build --release --config $config } else { Write-Error "Trunk.toml not found for $env:example" }

# Build an example for WASM
[unix]
build-wasm $example="alpha_blending":
    #!/usr/bin/env bash
    set -euo pipefail
    config=$(find examples-2d examples-3d -maxdepth 2 -path "*/$example/Trunk.toml" 2>/dev/null | head -1)
    [ -n "$config" ] && trunk build --release --config "$config" || echo "Trunk.toml not found for $example" >&2

# Serve an example in browser
[windows]
run-wasm $example="alpha_blending":
    $config = @("examples-2d", "examples-3d") | ForEach-Object { "$_/$env:example/Trunk.toml" } | Where-Object { Test-Path $_ } | Select-Object -First 1; if ($config) { trunk serve --release --open --config $config } else { Write-Error "Trunk.toml not found for $env:example" }

# Serve an example in browser
[unix]
run-wasm $example="alpha_blending":
    #!/usr/bin/env bash
    set -euo pipefail
    config=$(find examples-2d examples-3d -maxdepth 2 -path "*/$example/Trunk.toml" 2>/dev/null | head -1)
    [ -n "$config" ] && trunk serve --release --open --config "$config" || echo "Trunk.toml not found for $example" >&2

# Build only examples-2d/ and examples-3d/ for WASM (Windows)
[windows]
build-examples-wasm:
    $ErrorActionPreference = "Stop"; $prefix = $env:PUBLIC_URL_PREFIX; $root = (Get-Location).Path; @("examples-2d/*/Trunk.toml", "examples-3d/*/Trunk.toml") | ForEach-Object { Get-ChildItem -Path $_ -ErrorAction SilentlyContinue } | ForEach-Object { $example = $_.Directory.Name; Write-Host "Building $example for WASM..." -ForegroundColor Cyan; if ($prefix) { trunk build --release --config $_.FullName --dist "$root/dist/wasm/$example" --public-url "$prefix$example/" } else { trunk build --release --config $_.FullName --dist "$root/dist/wasm/$example" } }; Write-Host "All WASM builds complete! Output in dist/wasm/" -ForegroundColor Green

# Build only examples-2d/ and examples-3d/ for WASM (Unix)
[unix]
build-examples-wasm:
    #!/usr/bin/env bash
    set -euo pipefail
    prefix="${PUBLIC_URL_PREFIX:-}"
    root="$(pwd)"
    for trunk_file in examples-2d/*/Trunk.toml examples-3d/*/Trunk.toml; do
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

# Build all examples for WASM (includes examples-2d/, examples-3d/, and examples-terminal/) (Windows)
[windows]
build-all-wasm:
    $ErrorActionPreference = "Stop"; $prefix = $env:PUBLIC_URL_PREFIX; $root = (Get-Location).Path; @("examples-2d/*/Trunk.toml", "examples-3d/*/Trunk.toml") | ForEach-Object { Get-ChildItem -Path $_ -ErrorAction SilentlyContinue } | ForEach-Object { $example = $_.Directory.Name; Write-Host "Building $example for WASM..." -ForegroundColor Cyan; if ($prefix) { trunk build --release --config $_.FullName --dist "$root/dist/wasm/$example" --public-url "$prefix$example/" } else { trunk build --release --config $_.FullName --dist "$root/dist/wasm/$example" } }; Get-ChildItem -Path "examples-terminal/*/Trunk.toml" | ForEach-Object { $example = $_.Directory.Name; Write-Host "Building $example (terminal) for WASM..." -ForegroundColor Cyan; if ($prefix) { trunk build --release --config $_.FullName --dist "$root/dist/wasm/$example" --public-url "$prefix$example/" } else { trunk build --release --config $_.FullName --dist "$root/dist/wasm/$example" } }; Write-Host "All WASM builds complete! Output in dist/wasm/" -ForegroundColor Green

# Build all examples for WASM (includes examples-2d/, examples-3d/, and examples-terminal/) (Unix)
[unix]
build-all-wasm:
    #!/usr/bin/env bash
    set -euo pipefail
    prefix="${PUBLIC_URL_PREFIX:-}"
    root="$(pwd)"
    for trunk_file in examples-2d/*/Trunk.toml examples-3d/*/Trunk.toml examples-terminal/*/Trunk.toml; do
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
