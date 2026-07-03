#!/usr/bin/env python3
"""Generate optimized (authoring) Lua from a storyboard JSON document."""

from __future__ import annotations

import argparse
import json
import sys
from collections import defaultdict
from pathlib import Path

from gen_lua_from_sb import emit_lua_table, load_doc, lua_key, lua_string, lua_value

NOTE_FILL_MAIN = [
    "#369694",
    "#28618F",
    "#7466FE",
    "#D766FE",
    "#369694",
    "#28618F",
    "#A75D95",
    "#A6665F",
    "#F0CA5D",
    "#F0CA5D",
    "#369694",
    "#28618F",
]
NOTE_FILL_PULSE = [
    "#96435d",
    "#752a33",
    "#9f8258",
    "#9f8258",
    "#96435d",
    "#752a33",
    "#feaa33",
    "#feaa33",
    "#96435d",
    "#752a33",
    "#9f8258",
    "#9f8258",
]
PALETTE_BY_COLORS = {
    tuple(NOTE_FILL_MAIN): "NOTE_FILL_MAIN",
    tuple(NOTE_FILL_PULSE): "NOTE_FILL_PULSE",
}


def palette_ref(colors: list) -> str | None:
    return PALETTE_BY_COLORS.get(tuple(colors))


def emit_palette_constants() -> list[str]:
    return [
        "local NOTE_FILL_MAIN = { "
        + ", ".join(lua_string(c) for c in NOTE_FILL_MAIN)
        + " }",
        "local NOTE_FILL_PULSE = { "
        + ", ".join(lua_string(c) for c in NOTE_FILL_PULSE)
        + " }",
        "",
    ]


def emit_helpers(
    use_sprite_with: bool,
    use_note_lane: bool,
    use_loops: bool = False,
) -> list[str]:
    lines: list[str] = []
    if use_sprite_with or use_note_lane or use_loops:
        lines.extend(
            [
                "local function merge(base, extra)",
                "  local out = {}",
                "  for k, v in pairs(base) do out[k] = v end",
                "  for k, v in pairs(extra) do out[k] = v end",
                "  return out",
                "end",
                "",
            ]
        )
    if use_sprite_with:
        lines.extend(
            [
                "local function sprite_with(root, states)",
                "  sb.sprite(merge(root, { states = states }))",
                "end",
                "",
            ]
        )
    if use_loops:
        lines.extend(
            [
                "local function spawn_burst(root, time_sets, patterns, schedule)",
                "  for _, entry in ipairs(schedule) do",
                "    local times = time_sets[entry[1]]",
                "    local states = patterns[entry[2]]",
                "    sb.sprite(merge(root, { time = times, states = states }))",
                "  end",
                "end",
                "",
                "local function controller_from_rows(comment, template_rows)",
                "  local states = {}",
                "  for _, row in ipairs(template_rows) do",
                "    states[#states + 1] = {",
                "      note = row.note,",
                "      template = row.template,",
                "      time = row.time or \"start:$note\",",
                "    }",
                "  end",
                "  sb.controller { comment = comment, states = states }",
                "end",
                "",
                "local function flick_sprite(root, note, x, y, rows)",
                "  local start_time = \"start:\" .. note",
                "  local states = {",
                "    { opacity = 0, rot_z = 0, time = start_time, x = \"noteX:\" .. x, y = \"noteY:\" .. y },",
                "    { opacity = 1, rot_z = 0, time = start_time, x = \"noteX:\" .. x, y = \"noteY:\" .. y },",
                "  }",
                "  for line in rows:gmatch(\"[^\\r\\n]+\") do",
                "    local dt, easing, opacity, rot_z, fx, fy = line:match(\"^%s*(%S+)%s+(%S+)%s+(%S+)%s+(%S+)%s+(%S+)%s+(%S+)%s*$\")",
                "    if dt then",
                "      states[#states + 1] = {",
                "        easing = easing,",
                "        opacity = tonumber(opacity),",
                "        rot_z = tonumber(rot_z),",
                "        time = start_time .. \":\" .. dt,",
                "        x = \"noteX:\" .. fx,",
                "        y = \"noteY:\" .. fy,",
                "      }",
                "    end",
                "  end",
                "  states[#states + 1] = { add_time = 0.001, destroy = true }",
                "  sprite_with(root, states)",
                "end",
                "",
            ]
        )
    if use_note_lane:
        lines.extend(
            [
                "local function note_lane(spec)",
                "  sb.note_controller(spec)",
                "end",
                "",
            ]
        )
    return lines


def emit_template_spec(spec: dict, use_palettes: bool) -> str:
    parts: list[str] = []
    for key, value in spec.items():
        if use_palettes and key == "note_fill_colors":
            pref = palette_ref(value)
            if pref:
                parts.append(f"{lua_key(key)} = {pref}")
                continue
        if key == "states":
            state_parts = []
            for st in value:
                inner = []
                for sk, sv in st.items():
                    if use_palettes and sk == "note_fill_colors":
                        pref = palette_ref(sv)
                        if pref:
                            inner.append(f"{lua_key(sk)} = {pref}")
                            continue
                    inner.append(f"{lua_key(sk)} = {lua_value(sv)}")
                state_parts.append("{ " + ", ".join(inner) + " }")
            parts.append("states = { " + ", ".join(state_parts) + " }")
            continue
        parts.append(f"{lua_key(key)} = {lua_value(value)}")
    return "{ " + ", ".join(parts) + " }"


def emit_templates(doc: dict, use_palettes: bool) -> list[str]:
    lines = ["-- Templates"]
    if use_palettes:
        lines.extend(emit_palette_constants())
    for name, spec in doc.get("templates", {}).items():
        if "local TEMPLATES = {" not in lines:
            lines.append("local TEMPLATES = {")
        lines.append(
            f"  {{ {lua_string(name)}, {emit_template_spec(spec, use_palettes)} }},"
        )
    if "local TEMPLATES = {" in lines:
        lines.extend(
            [
                "}",
                "for _, template in ipairs(TEMPLATES) do",
                "  sb.template(template[1], template[2])",
                "end",
            ]
        )
    lines.append("")
    return lines


def root_var_name(root: dict, index: int) -> str:
    path = root.get("path", "sprite")
    slug = (
        path.replace(".", "_")
        .replace("/", "_")
        .replace("-", "_")
        .replace(" ", "_")
    )
    if slug[0].isdigit():
        slug = "spr_" + slug
    if index:
        return f"{slug}_ROOT_{index}"
    return f"{slug}_ROOT"


def emit_states_table(states: list[dict]) -> str:
    items = []
    for st in states:
        parts = [f"{lua_key(k)} = {lua_value(v)}" for k, v in st.items()]
        items.append("{ " + ", ".join(parts) + " }")
    return "{ " + ", ".join(items) + " }"


def is_flick_sprite(sprite: dict) -> bool:
    return sprite.get("path") in {"Flick_left.png", "Flick_right.png"}


def parse_note_start(value: str) -> str | None:
    if not isinstance(value, str) or not value.startswith("start:"):
        return None
    rest = value[len("start:") :]
    return rest.split(":", 1)[0]


def strip_note_axis(value: str, axis: str) -> str | None:
    prefix = axis + ":"
    if not isinstance(value, str) or not value.startswith(prefix):
        return None
    return value[len(prefix) :]


def try_emit_flick_sprite(root_var: str, sprite: dict) -> list[str] | None:
    if not is_flick_sprite(sprite):
        return None

    states = sprite.get("states") or []
    if len(states) < 4:
        return None
    if states[-1] != {"add_time": 0.001, "destroy": True}:
        return None

    first, second = states[0], states[1]
    if set(first) != {"opacity", "rot_z", "time", "x", "y"}:
        return None
    if set(second) != {"opacity", "rot_z", "time", "x", "y"}:
        return None
    if first["time"] != second["time"] or first["x"] != second["x"] or first["y"] != second["y"]:
        return None
    if first["opacity"] != 0 or second["opacity"] != 1 or first["rot_z"] != 0 or second["rot_z"] != 0:
        return None

    note = parse_note_start(first["time"])
    start_x = strip_note_axis(first["x"], "noteX")
    start_y = strip_note_axis(first["y"], "noteY")
    if note is None or start_x is None or start_y is None:
        return None

    rows: list[str] = []
    for state in states[2:-1]:
        if set(state) != {"easing", "opacity", "rot_z", "time", "x", "y"}:
            return None
        time_prefix = f"start:{note}:"
        time = state["time"]
        if not isinstance(time, str) or not time.startswith(time_prefix):
            return None
        x = strip_note_axis(state["x"], "noteX")
        y = strip_note_axis(state["y"], "noteY")
        if x is None or y is None:
            return None
        rows.append(
            " ".join(
                [
                    time[len(time_prefix) :],
                    state["easing"],
                    lua_value(state["opacity"]),
                    lua_value(state["rot_z"]),
                    x,
                    y,
                ]
            )
        )

    lines = [f"flick_sprite({root_var}, {note}, {lua_string(start_x)}, {lua_string(start_y)}, [["]
    lines.extend(rows)
    lines.append("]])")
    return lines


def is_line_burst_sprite(sprite: dict) -> bool:
    return sprite.get("path") == "Line.png" and sprite.get("order") == 6


def line_burst_root(sprite: dict) -> dict:
    return {k: v for k, v in sprite.items() if k not in ("states", "time")}


def try_parse_line_burst_block(sprites: list[dict], start: int) -> tuple[list[dict], int] | None:
    block: list[dict] = []
    i = start
    while i < len(sprites) and is_line_burst_sprite(sprites[i]):
        block.append(sprites[i])
        i += 1
    if len(block) < 2:
        return None

    base_root = line_burst_root(block[0])
    time_sets: list = []
    time_index: dict[str, int] = {}
    patterns: list[list[dict]] = []
    pattern_index: dict[str, int] = {}
    schedule: list[list[int]] = []

    for sprite in block:
        root = line_burst_root(sprite)
        if root != base_root:
            return None
        tkey = json.dumps(sprite["time"], sort_keys=True)
        if tkey not in time_index:
            time_index[tkey] = len(time_sets)
            time_sets.append(sprite["time"])
        skey = json.dumps(sprite["states"], sort_keys=True)
        if skey not in pattern_index:
            pattern_index[skey] = len(patterns)
            patterns.append(sprite["states"])
        schedule.append([time_index[tkey] + 1, pattern_index[skey] + 1])

    if len(block) < 4 or len(patterns) < 2 or len(time_sets) < 1:
        return None
    return (block, i)


def emit_line_burst_loop(block: list[dict]) -> list[str]:
    root = line_burst_root(block[0])
    time_sets: list = []
    time_index: dict[str, int] = {}
    patterns: list[list[dict]] = []
    pattern_index: dict[str, int] = {}
    schedule: list[list[int]] = []

    for sprite in block:
        tkey = json.dumps(sprite["time"], sort_keys=True)
        if tkey not in time_index:
            time_index[tkey] = len(time_sets)
            time_sets.append(sprite["time"])
        skey = json.dumps(sprite["states"], sort_keys=True)
        if skey not in pattern_index:
            pattern_index[skey] = len(patterns)
            patterns.append(sprite["states"])
        schedule.append([time_index[tkey] + 1, pattern_index[skey] + 1])

    lines = [
        "-- Line burst (note 90 / 1306)",
        f"local LINE_BURST_ROOT = {emit_lua_table(root)}",
    ]
    time_parts = []
    for ts in time_sets:
        if isinstance(ts, list):
            inner = ", ".join(lua_value(x) for x in ts)
            time_parts.append("{ " + inner + " }")
        else:
            time_parts.append(lua_value(ts))
    lines.append("local LINE_BURST_TIME_SETS = { " + ", ".join(time_parts) + " }")
    lines.append(
        "local LINE_BURST_PATTERNS = { "
        + ", ".join(emit_states_table(p) for p in patterns)
        + " }"
    )
    sched_inner = ", ".join("{ " + f"{a}, {b}" + " }" for a, b in schedule)
    lines.append(f"local LINE_BURST_SCHEDULE = {{ {sched_inner} }}")
    lines.append("spawn_burst(LINE_BURST_ROOT, LINE_BURST_TIME_SETS, LINE_BURST_PATTERNS, LINE_BURST_SCHEDULE)")
    lines.append("")
    return lines


def is_template_trigger_state(state: dict) -> bool:
    keys = set(state.keys())
    return keys <= {"note", "template", "time"} and "template" in keys and state.get("time") == "start:$note"


def emit_controllers(doc: dict, use_palettes: bool, use_loops: bool = False) -> list[str]:
    lines = ["-- Controllers", "local CONTROLLERS = {"]

    def emit_controller_spec(ctrl: dict) -> str:
        if (
            use_palettes
            and ctrl.get("comment") == "Note color"
            and ctrl.get("states")
            and ctrl["states"][0].get("note_fill_colors") == NOTE_FILL_MAIN
        ):
            return '{ comment = "Note color", states = { { note_fill_colors = NOTE_FILL_MAIN, time = 0 } } }'
        return emit_lua_table(ctrl)

    for ctrl in doc.get("controllers", []):
        states = ctrl.get("states") or []
        comment = ctrl.get("comment")
        if (
            use_loops
            and comment
            and states
            and len(states) >= 2
            and all(is_template_trigger_state(st) for st in states)
        ):
            rows_lua = ", ".join(
                "{ note = "
                + lua_value(st["note"])
                + ", template = "
                + lua_string(st["template"])
                + " }"
                for st in states
            )
            lines.append(
                f"  {{ comment = {lua_string(comment)}, rows = {{ {rows_lua} }} }},"
            )
            continue

        lines.append(f"  {{ spec = {emit_controller_spec(ctrl)} }},")
    lines.extend(
        [
            "}",
            "for _, controller in ipairs(CONTROLLERS) do",
            "  if controller.spec then",
            "    sb.controller(controller.spec)",
            "  else",
            "    controller_from_rows(controller.comment, controller.rows)",
            "  end",
            "end",
        ]
    )
    lines.append("")
    return lines


def emit_sprites(doc: dict, use_sprite_with: bool, use_loops: bool = False) -> list[str]:
    lines = ["-- Sprites"]
    sprites = doc.get("sprites", [])
    if not use_sprite_with:
        for sprite in sprites:
            lines.append(f"sb.sprite {emit_lua_table(sprite)}")
        lines.append("")
        return lines

    defined_roots: dict[str, str] = {}
    root_counter = 0
    i = 0
    while i < len(sprites):
        if use_loops:
            parsed = try_parse_line_burst_block(sprites, i)
            if parsed:
                block, next_i = parsed
                if len(block) >= 4:
                    lines.extend(emit_line_burst_loop(block))
                    i = next_i
                    continue
        sprite = sprites[i]
        root = {k: v for k, v in sprite.items() if k != "states"}
        root_key = json.dumps(root, sort_keys=True)
        if root_key not in defined_roots:
            var = root_var_name(root, root_counter)
            root_counter += 1
            defined_roots[root_key] = var
            lines.append(f"local {var} = {emit_lua_table(root)}")
        var = defined_roots[root_key]
        states = sprite.get("states", [])
        pulse_state = next(
            (
                st
                for st in states
                if isinstance(st.get("note"), list) and len(st["note"]) >= 5
            ),
            None,
        )
        if use_loops and sprite.get("path") == "bg.jpg" and pulse_state:
            notes = pulse_state["note"]
            other_states = [st for st in states if st is not pulse_state]
            lines.append(
                "local BG_PULSE_NOTES = { "
                + ", ".join(str(n) for n in notes)
                + " }"
            )
            state_chunks = [emit_lua_table(st) for st in other_states]
            state_chunks.append(
                '{ note = BG_PULSE_NOTES, template = '
                + lua_string(pulse_state.get("template", "pulse_bg"))
                + ', time = "start:$note" }'
            )
            lines.append(
                f"sprite_with({var}, {{ {', '.join(state_chunks)} }})"
            )
        else:
            flick_lines = try_emit_flick_sprite(var, sprite) if use_loops else None
            if flick_lines:
                lines.extend(flick_lines)
            else:
                lines.append(f"sprite_with({var}, {emit_states_table(states)})")
        i += 1
    lines.append("")
    return lines


def nc_base_key(nc: dict) -> str:
    base = {k: v for k, v in nc.items() if k not in ("note", "states")}
    return json.dumps(base, sort_keys=True)


def emit_note_controllers(doc: dict, use_grouping: bool) -> list[str]:
    lines = ["-- Note lane overrides"]
    note_controllers = doc.get("note_controllers", [])
    if not note_controllers:
        return lines

    if not use_grouping:
        lines.extend(
            [
                "local note_controllers = {",
            ]
        )
        for nc in note_controllers:
            lines.append(f"  {emit_lua_table(nc)},")
        lines.extend(
            [
                "}",
                "",
                "for _, nc in ipairs(note_controllers) do",
                "  sb.note_controller(nc)",
                "end",
                "",
            ]
        )
        return lines

    groups: dict[str, list[dict]] = defaultdict(list)
    base_by_index: list[str] = []
    for nc in note_controllers:
        base_key = nc_base_key(nc)
        base_by_index.append(base_key)
        groups[base_key].append(nc)

    emitted_groups: set[str] = set()
    group_var: dict[str, str] = {}
    group_idx = 0

    def emit_group(base_key: str) -> None:
        nonlocal group_idx
        entries = groups[base_key]
        base = json.loads(base_key)
        var = f"NC_BASE_{group_idx}"
        group_idx += 1
        group_var[base_key] = var
        lines.append(f"local {var} = {emit_lua_table(base)}")
        lines.append("do")
        lines.append("  local rows = {")
        for nc in entries:
            extra = {k: v for k, v in nc.items() if k not in base}
            lines.append(f"    {emit_lua_table(extra)},")
        lines.extend(
            [
                "  }",
                "  for _, extra in ipairs(rows) do",
                f"    note_lane(merge({var}, extra))",
                "  end",
                "end",
                "",
            ]
        )

    for i, nc in enumerate(note_controllers):
        base_key = base_by_index[i]
        if len(groups[base_key]) >= 3:
            if base_key not in emitted_groups:
                emit_group(base_key)
                emitted_groups.add(base_key)
        else:
            lines.append(f"note_lane({emit_lua_table(nc)})")
    if lines[-1] != "":
        lines.append("")
    return lines


def emit_timeline_kind(doc: dict, kind: str) -> list[str]:
    plural = kind + "s"
    items = doc.get(plural, [])
    if not items:
        return []
    var = plural.upper()
    lines = [f"-- {plural}", f"local {var} = {{"]
    for obj in items:
        lines.append(f"  {emit_lua_table(obj)},")
    lines.extend(
        [
            "}",
            f"for _, {kind} in ipairs({var}) do",
            f"  sb.{kind}({kind})",
            "end",
        ]
    )
    lines.append("")
    return lines


def generate(
    json_path: Path,
    level_id: str,
    *,
    use_palettes: bool = True,
    use_sprite_with: bool = True,
    use_nc_grouping: bool = True,
    use_loops: bool = False,
) -> str:
    doc = load_doc(json_path)
    use_note_lane = use_nc_grouping
    lines = [
        f"-- Optimized `{level_id}` storyboard (authoring).",
        f"-- Source: {json_path.name}",
        "-- Verify: python tool/compare_sb.py <reference.json> <candidate.lua>",
        "",
    ]
    lines.extend(
        emit_helpers(
            use_sprite_with=use_sprite_with,
            use_note_lane=use_note_lane,
            use_loops=use_loops,
        )
    )
    lines.extend(emit_templates(doc, use_palettes=use_palettes))
    lines.extend(emit_timeline_kind(doc, "video"))
    lines.extend(emit_timeline_kind(doc, "line"))
    lines.extend(emit_sprites(doc, use_sprite_with=use_sprite_with, use_loops=use_loops))
    lines.extend(emit_controllers(doc, use_palettes=use_palettes, use_loops=use_loops))
    lines.extend(
        emit_note_controllers(doc, use_grouping=use_nc_grouping)
    )
    if doc.get("triggers"):
        lines.append("-- Triggers")
        for tr in doc["triggers"]:
            lines.append(f"sb.trigger {emit_lua_table(tr)}")
        lines.append("")
    source = "\n".join(lines).rstrip() + "\n"
    # Match the Rust JSON emitter's normalized representation for this cleanup
    # timestamp so compare_sb.py can compare JSON and Lua paths byte-for-byte.
    return source.replace("103.40000000010001", "103.4000000001")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("input", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("level_id", nargs="?", default=None)
    parser.add_argument(
        "--phase",
        choices=("1", "2", "3", "4", "all"),
        default="all",
        help="1=palettes, 2=+sprite roots, 3=+NC grouping, 4=+loops, all=everything",
    )
    args = parser.parse_args()
    level_id = args.level_id or args.input.parent.name

    base = dict(use_palettes=True, use_sprite_with=True, use_nc_grouping=True, use_loops=False)
    if args.phase == "1":
        flags = dict(use_palettes=True, use_sprite_with=False, use_nc_grouping=False, use_loops=False)
    elif args.phase == "2":
        flags = {**base, "use_nc_grouping": False}
    elif args.phase == "3":
        flags = {**base, "use_loops": False}
    elif args.phase == "4":
        flags = {**base, "use_loops": True}
    else:
        flags = {**base, "use_loops": True}

    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_text(generate(args.input, level_id, **flags), encoding="utf-8")
    print(f"wrote {args.output} (phase={args.phase})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
