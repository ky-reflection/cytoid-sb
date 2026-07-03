# kyr.bite_your_nails — CHAOS storyboard restore

Lua recreation of **`kyr.bite_your_nails/CHAOS_storyboard.json`** (Lab cache).

| Original | Lua output |
|----------|------------|
| 26 note_controllers | 26 note_controllers |
| 4 controllers (79 keyframes total) | 4 controllers |

This is the **complex example**: data-driven loops instead of hand-written JSON.

Patterns used:

1. **Note lane overrides** — table of `{ note, y, override_y, time }` + `for` loop → `sb.note_controller`
2. **Scanline color timeline** — 53-keyframe controller as a Lua array + `:key(time, patch)` loop
3. **UI / scanline position** controllers — same loop pattern

```bash
cargo run -p cytoid-sb-cli -- check examples/kyr_bite_your_nails/storyboard.lua
# ok: … (4 controllers, 26 note_controllers)
```

Regenerate after syncing Lab levels:

```bash
python tool/gen_lua_from_sb.py \
  examples/levels/kyr.bite_your_nails/CHAOS_storyboard.json \
  examples/kyr_bite_your_nails/storyboard.lua \
  kyr.bite_your_nails
```

Source chart: *Bite Your Nails* (CHAOS difficulty). The Lua file is ~165 lines vs ~12k lines of JSON.
