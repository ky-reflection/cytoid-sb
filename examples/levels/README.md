# Local test levels (from Cytoid Lab)

Copied from the Lab install cache on Windows:

```text
%USERPROFILE%\AppData\LocalLow\TigerHix\Cytoid Lab\<level-id>\
```

These are **local-only** fixtures for `cytoid-sb check` / `compile` — not committed to git (~200 MB).

## Refresh

```powershell
.\tool\sync-lab-levels.ps1
```

## Batch check

```powershell
.\tool\check-levels.ps1
cargo run -p cytoid-sb-cli -- check examples/levels/he.macaron
```

Last batch run (2026-07-03): **12 passed**, 0 failed, 2 dirs skipped (no storyboard JSON in cache).

`check` prints a per-type summary, e.g. `16 sprites, 10 controllers, 409 note_controllers`.

## Levels (2026-07-03)

| Directory | Storyboard | Notes |
|-----------|------------|-------|
| `kou.ppap` | `storyboard.json` | Smallest (~3 MB) |
| `he.macaron` | `storyboard.json` | Medium SB |
| `cht.ffr` | `CHAOS_storyboard.json` | Template-heavy |
| `cht.desive` | `GLITCH_storyboard.json` | |
| `timitini.bpm_rt_` | `storyboard.json` | |
| `timitini.ccc4th.energy_synergy_matrix_` | `storyboard.json` | Larger SB |
| `timitini.you_are_the_miserable_` | `storyboard_ex.json`, `storyboard_hd.json` | Dual SB |
| `cht.once_forgotten` | `Storyboard.json` | Largest (~85 MB) |

Each folder is an extracted level package (chart JSON, assets, storyboard JSON).
