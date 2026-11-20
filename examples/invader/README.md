# Solix Invaders — Cyber Neon Space Invaders Arcade Showcase

A complete, feature-rich, modular modern remake of the classic arcade space shooter written in **Solix**, powered by `raylib` FFI bindings and `solixlib`.

---

## Architecture & Modular File Organization

Unlike single-file demos, `invader` demonstrates production Solix modular architecture with clear separation of concerns across multiple `.slx` compilation units:

| Source File | Subsystem & Role |
| :--- | :--- |
| [`src/types.slx`](src/types.slx) | Game state enum (`TITLE`, `PLAYING`, `PAUSED`, `VICTORY`, `GAME_OVER`), alien species, projectile kinds, and screen layout constants. |
| [`src/audio.slx`](src/audio.slx) | Sound manager integrating Raylib audio (`raudio`), procedural laser synthesis, marching cadence, explosions, and mute toggling. |
| [`src/starfield.slx`](src/starfield.slx) | Three-layer parallax twinkling neon starfield background with variable speeds and brightness tiers. |
| [`src/particle.slx`](src/particle.slx) | High-performance particle burst system with decaying velocity, alpha fade-out, and color-matched alien explosions. |
| [`src/projectile.slx`](src/projectile.slx) | Dual-mode laser projectile system (player vertical laser beam with outer glow and alien plasma bombs). |
| [`src/bunker.slx`](src/bunker.slx) | Four destructible defensive bunkers composed of 18 micro-blocks each with progressive 3-stage visual erosion. |
| [`src/enemy.slx`](src/enemy.slx) | 55-alien armada (Squid, Crab, Octopus) with synchronized marching, edge descent, speed acceleration, and Mystery UFO mothership flybys. |
| [`src/player.slx`](src/player.slx) | Player defender cannon ship with smooth horizontal movement, weapon cooldown, lives tracking, and respawn invulnerability. |
| [`src/ui.slx`](src/ui.slx) | Retro arcade HUD, score advance table, CRT scanline overlay, pause screen, and game over / victory modals. |
| [`src/main.slx`](src/main.slx) | Main entrypoint (`InvaderGame::main`), 60 FPS game loop, collision matrix, and state coordinator. |

---

## Controls

| Key | Action |
| :--- | :--- |
| `LEFT` / `A` | Move Cannon Left |
| `RIGHT` / `D` | Move Cannon Right |
| `SPACE` | Fire Laser Cannon / Insert Coin |
| `P` / `ESCAPE` | Pause / Resume Mission |
| `M` | Toggle Audio Mute |
| `C` | Toggle CRT Scanline Shader Overlay |
| `R` | Restart Match |
| `Q` | Return to Title Menu |

---

## Building and Running

### 1. Build Package
```bash
# Debug profile (local development)
solix build examples/invader --profile debug

# Release profile (cross-platform optimized bundle)
solix build examples/invader --profile release
```

### 2. Execute Package
```bash
solix run package examples/invader/build/debug/invader.slxp
```

### 3. Run Automated Unit Tests
```bash
solix build examples/invader --profile test
solix run package --profile test examples/invader/build/test/invader.slxp
```
