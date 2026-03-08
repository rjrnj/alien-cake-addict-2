# Alien Cake Addict (Extended)

A small 3D game built with [Bevy 0.18](https://bevyengine.org/), extended and refactored from the official Bevy `alien_cake_addict` example.

## What was added

The original example is intentionally minimal — a bare game loop with no sound, no effects, and a monolithic `Game` resource holding most of the state. 
This version is refactored toward stricter ECS conventions and extended with additional features:

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
