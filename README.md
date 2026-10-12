# Alien Cake Addict (Extended)

A small 3D game built with [Bevy 0.20](https://bevyengine.org/), extended and refactored from the official Bevy `alien_cake_addict` example.

## What was added

The original example is an intentionally minimal game with no sound, no effects, and a monolithic `Game` resource holding most of the state.

- **Missiles** — projectiles that fly across the board and trigger game over on contact
- **Sound** — background music and a game over sound
- **Cake VFX** — a floating `+2` animation in screen space when the player picks up a cake
- **Visual improvements** — scoreboard UI, bonus lighting, cake scale animation tied to score

## Controls

| Key | Action |
|-----|--------|
| Arrow keys | Move |
| Space | Restart (game over screen) |

## Running

```bash
cargo run --release
```

## Development

```bash
cargo test
cargo clippy --all-targets -- -D warnings
cargo fmt --check
```

CI runs the same checks on every push to `main` and on pull requests.

## License

The code is licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option. See [CREDITS.md](CREDITS.md) for asset credits and licenses.
