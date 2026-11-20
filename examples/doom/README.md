# Doom : Solix Edition — 3D Retro FPS Arcade Showcase

A complete, modern 3D retro first-person shooter written in **Solix**, powered by `raylib` 3D hardware-accelerated bindings and `solixlib`.

---

## Architecture & Modular File Organization

`doom` demonstrates production Solix modular architecture with clean separation of concerns across 9 specialized compilation units:

| Source File | Subsystem & Role |
| :--- | :--- |
| [`src/types.slx`](src/types.slx) | Game state enum (`TITLE`, `PLAYING`, `PAUSED`, `INTERMISSION`, `GAME_OVER`), weapon kinds, enemy species, AI states, and screen layout constants. |
| [`src/math.slx`](src/math.slx) | 3D Euclidean distance, 2D planar distance, yaw angle trigonometry, and bounding box overlap collision tests. |
| [`src/audio.slx`](src/audio.slx) | Spatial sound manager wrapping Raylib `raudio`, volume balancing, procedural file resolution, and `M` key mute toggling. |
| [`src/camera.slx`](src/camera.slx) | First-person 3D camera controller with mouse look, pitch/yaw angle clamping (-85° to +85°), view bobbing, and directional vectors. |
| [`src/map.slx`](src/map.slx) | Multi-level 3D map engine with sectors, walls, toxic slime damage pits, keycard blast doors (Blue/Red), and exit switches. |
| [`src/weapon.slx`](src/weapon.slx) | 5-weapon arsenal (Fist, Pistol, Shotgun, Chaingun, Rocket Launcher) with firing animations, muzzle blasts, and recoil. |
| [`src/projectile.slx`](src/projectile.slx) | 3D rockets, Imp fireballs, Cacodemon ball-lightning, particle spark pool, and area-of-effect splash radius explosions. |
| [`src/enemy.slx`](src/enemy.slx) | 4 demon classes (Zombieman, Imp, Pinky Demon, Cacodemon) with 6-stage AI state machine (`IDLE`, `CHASE`, `ATTACK`, `PAIN`, `DEATH`, `DEAD`). |
| [`src/player.slx`](src/player.slx) | Player state, smooth friction movement, health, armor absorption, keycard inventory, damage flashes, and mugshot expressions. |
| [`src/ui.slx`](src/ui.slx) | Classic status bar, ammo counters, animated Doomguy mugshot (eyes darting, evil grin, pain grimace, injury stages), and crosshair. |
| [`src/main.slx`](src/main.slx) | Game coordinator, 60 FPS loop, Raylib `BeginMode3D` pipeline, collision detection, and level progression entrypoint (`DoomGame::main`). |

---

## Controls

| Key | Action |
| :--- | :--- |
| **`W` / `A` / `S` / `D`** | Move Forward / Strafe Left / Move Backward / Strafe Right |
| **`MOUSE` / `ARROWS`** | Aim & Look Around (Mouse captured in FPS mode) |
| **`LEFT CLICK` / `CTRL` / `SPACE`** | Fire Active Weapon |
| **`1`, `2`, `3`, `4`, `5`** | Switch Weapons (Fist, Pistol, Shotgun, Chaingun, Rocket Launcher) |
| **`E`** | Interact / Open Keycard Doors & Flip Exit Switches |
| **`SHIFT`** | Sprint / Fast Run |
| **`P` / `ESCAPE`** | Pause & Release Mouse Cursor / Resume Mission |
| **`M`** | Toggle Audio (Mute / Unmute) |
| **`R`** | Quick Restart after Death |
| **`Q`** | Return to Title Menu |

---

## Building and Running

### 1. Build Package
```bash
# Debug profile (local development)
solix build examples/doom --profile debug

# Release profile (cross-platform optimized bundle)
solix build examples/doom --profile release
```

### 2. Execute Package
```bash
solix run package examples/doom/build/release/doom.slxp
```

### 3. Run Automated Unit Tests
```bash
solix build examples/doom --profile test
solix run package --profile test examples/doom/build/test/doom.slxp
```
