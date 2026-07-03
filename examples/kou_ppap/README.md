# kou.ppap — full storyboard restore

Lua recreation of the Lab level **`kou.ppap`** (`examples/levels/kou.ppap/storyboard.json`).

| Original | Lua output |
|----------|------------|
| 2 controllers | 2 controllers |
| 1 sprite | 1 sprite |

Demonstrates:

- Inline controller: `sb.controller { scanline_opacity = 0, time = 0 }`
- Controller keyframes + **relative** timing (`:rel(0.1, { add_time … })`)
- Sprite root fields (`width`, `height`, `easing`, …) + destroy keyframe

```bash
cargo run -p cytoid-sb-cli -- check examples/kou_ppap/storyboard.lua
cargo run -p cytoid-sb-cli -- compile examples/kou_ppap/storyboard.lua
```

Regenerate from Lab JSON:

```bash
python tool/gen_lua_from_sb.py examples/levels/kou.ppap/storyboard.json examples/kou_ppap/storyboard.lua kou.ppap
```
