# cytoid-sb

External compiler / validator for Cytoid Storyboard authoring.

Phase 1 goal: scripts and sugar JSON in → **`storyboard.generated.json`** out — loadable by Unity `Storyboard.Parse()` without runtime changes.

Schema reference: `engines/unity/Assets/Scripts/Storyboard/` in [cytoid-core-unity](https://github.com/Cytoid/cytoid-core-unity).

## Status

Working `check` / `compile` / `watch` for:

- **JSON** — Unity authoring (`sprites`, `controllers`, …) and compiled (`compiled: true`) formats; JSON5 `//` comments supported (Lab exports)
- **Lua** — `sb.sprite` / `sb.text` / `sb.video` / `sb.line` + `:key()` / `:rel()`, `sb.controller`, `sb.note_controller`, and `sb.trigger` (emits Unity authoring JSON for `Storyboard.Parse()`)
- **Level directories** — resolves `storyboard.lua`, `storyboard.json`, `Storyboard.json`, or largest `*storyboard*.json`

Local Lab fixtures (~200 MB) live under `examples/levels/` (gitignored). Run `tool/sync-lab-levels.ps1` then `tool/check-levels.ps1`.

## Workspace

| Crate | Role |
|-------|------|
| `cytoid-sb-model` | Document model, JSON5 parse, field helpers, summary |
| `cytoid-sb-builder` | Authoring API (sprites, controllers, keyframe normalization) |
| `cytoid-sb-lower` | Syntax sugar lowering (identity stub for now) |
| `cytoid-sb-validate` | Semantic validation aligned with Unity rules |
| `cytoid-sb-emit` | JSON emitter; preserves authoring vs compiled root shape |
| `cytoid-sb-diag` | Error types |
| `cytoid-sb-lua` | Lua 5.4 host (`mlua`) |
| `cytoid-sb-cli` | `cytoid-sb` binary |

## Build

```bash
cargo build --release
cargo test
```

## CLI

```bash
# Sprite example (Lua)
cargo run -p cytoid-sb-cli -- check examples/hello/storyboard.lua
cargo run -p cytoid-sb-cli -- compile examples/hello/storyboard.lua

# Controller example (matches offset_guide)
cargo run -p cytoid-sb-cli -- check examples/hello/storyboard_controller.lua

# Compiled JSON
cargo run -p cytoid-sb-cli -- check examples/hello/storyboard.json

# Lab level directory (auto-resolves storyboard file)
cargo run -p cytoid-sb-cli -- check examples/levels/kou.ppap
```

### Complex examples (real charts)

| Example | Level | Objects |
|---------|-------|---------|
| [`examples/kou_ppap/`](examples/kou_ppap/) | [kou.ppap](examples/levels/kou.ppap/) | 1 sprite, 2 controllers — **100% restore** |
| [`examples/kyr_bite_your_nails/`](examples/kyr_bite_your_nails/) | [kyr.bite_your_nails CHAOS](examples/levels/kyr.bite_your_nails/) | 26 note_controllers, 4 controllers — **100% restore** |
| [`examples/gate_showcase/`](examples/gate_showcase/) | synthetic complex showcase | 6 sprites, 1 controller, 1 text, 1 video, 1 line, 12 note_controllers, 1 trigger |

```bash
cargo run -p cytoid-sb-cli -- check examples/kyr_bite_your_nails/storyboard.lua
cargo run -p cytoid-sb-cli -- compile examples/kou_ppap/storyboard.lua
```

Generate Lua from any Lab `storyboard.json`:

```bash
python tool/gen_lua_from_sb.py examples/levels/kou.ppap/storyboard.json examples/kou_ppap/storyboard.lua
```

### Lua API (Phase 1)

```lua
local glow = sb.sprite { id = "hello_glow", path = "hello_glow.png" }
glow:key(0.0, { opacity = 0, scale = 0.8, ease = "out_quint" })

local cam = sb.controller { id = "camera" }
cam:key(0.0, { background_dim = 1, ui_opacity = 0 })
cam:rel(0.1, { note_ring_color = "#FFFFFF" })  -- add_time keyframe

local title = sb.text { id = "title", text = "READY", x = 0, y = 120, opacity = 0 }
title:key(1.0, { opacity = 1, text = "GO" })

sb.note_controller { note = 422, override_y = true, y = 0.75, time = 0 }

sb.trigger { type = "NoteClear", notes = { 422 }, spawn = { "title" } }
```

Lua source currently emits the same snake_case authoring fields that Unity's
`Storyboard.Parse()` already understands. A true compiled/PascalCase lowerer is
planned separately because it must also emit inherited state snapshots,
`UnitFloat` objects, parsed colors, and all required compiled root arrays.

## Related repos

- [Cytoid/cytoid-core-unity](https://github.com/Cytoid/cytoid-core-unity) — Unity runtime
- Storyboard design docs in that repo under `docs/local/` (local-only)

## Roadmap

1. `lower` — template expansion, note selectors, time expressions (needs chart context)
2. Source maps and diagnostics that point back to Lua call sites
3. True compiled/PascalCase lowerer once inherited state snapshots, `UnitFloat`, colors, and required root arrays are modeled
4. Optional `StoryboardProgram` runtime IR (Unity side)
