# gate_showcase

Lua-authored complex storyboard sample.

It is not copied from a single level. It intentionally combines patterns seen in
real Cytoid storyboards:

- repeated gate sprite parts with shared timing;
- scanline/camera controller tracks;
- note lane overrides;
- text, line, video declarations;
- a note-clear trigger.

Run:

```bash
cargo run -p cytoid-sb-cli -- check examples/gate_showcase/storyboard.lua
cargo run -p cytoid-sb-cli -- compile examples/gate_showcase/storyboard.lua -o target/gate_showcase.storyboard.json
```
