# Nightshade Examples

Example applications built with the [Nightshade](https://github.com/matthewjberger/nightshade) game engine.

## Running Examples

```bash
cargo run -p doom
cargo run -p chess
cargo run -p horror
# etc.
```

## Building for Web

```bash
trunk build --release --config examples/doom/Trunk.toml
```

## Examples

| Example | Description |
|---------|-------------|
| doom | DOOM WAD renderer |
| chess | 3D chess game with physics |
| horror | First-person horror game |
| platformer | 2D platformer with physics |
| ... | And many more |

## Notes

- `steam` and `multiplayer_pong` require the Steam SDK
- `plugins` is a standalone workspace demonstrating the plugin system
