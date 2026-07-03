#!/usr/bin/env python3
"""Generate a Lua storyboard source from a Lab/uncompiled JSON file."""

from __future__ import annotations

import json
import sys
from pathlib import Path


def lua_string(s: str) -> str:
    return json.dumps(s, ensure_ascii=False)


def fmt_inline_table(obj: dict) -> str:
    parts = []
    for key, value in obj.items():
        if key in ("states", "States"):
            continue
        parts.append(f"{key} = {lua_value(value)}")
    return "{ " + ", ".join(parts) + " }"


def lua_value(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if value is None:
        return "nil"
    if isinstance(value, (int, float)):
        return repr(value)
    if isinstance(value, str):
        return lua_string(value)
    if isinstance(value, list):
        if all(isinstance(x, str) for x in value):
            inner = ", ".join(lua_string(x) for x in value)
            return "{ " + inner + " }"
        inner = ", ".join(lua_value(x) for x in value)
        return "{ " + inner + " }"
    if isinstance(value, dict):
        return fmt_inline_table(value)
    return lua_string(str(value))


def emit_note_controllers(note_controllers: list[dict]) -> list[str]:
    lines = [
        "-- Note lane overrides (data from original storyboard.json)",
        "local note_controllers = {",
    ]
    for nc in note_controllers:
        items = ", ".join(f"{k} = {lua_value(v)}" for k, v in nc.items())
        lines.append("  { " + items + " },")
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


def emit_controller_states(states: list[dict], var_name: str) -> list[str]:
    lines = [f"local {var_name} = {{"]
    for st in states:
        items = []
        for k, v in st.items():
            items.append(f"{k} = {lua_value(v)}")
        lines.append("  { " + ", ".join(items) + " },")
    lines.extend(
        [
            "}",
            f"local {var_name}_handle = sb.controller {{}}",
            f"for _, kf in ipairs({var_name}) do",
            "  local patch = {}",
            "  for k, v in pairs(kf) do",
            '    if k ~= "time" and k ~= "add_time" then patch[k] = v end',
            "  end",
            "  if kf.add_time then",
            f"    {var_name}_handle:rel(kf.add_time, patch)",
            "  else",
            "    local t = kf.time or 0",
            f"    {var_name}_handle:key(t, patch)",
            "  end",
            "end",
            "",
        ]
    )
    return lines


def emit_inline_controller(obj: dict) -> list[str]:
    return [f"sb.controller {fmt_inline_table(obj)}", ""]


def hoist_required_stage_field(kind: str, root: dict, states: list[dict]) -> dict:
    required_keys = {
        "sprite": ("path", "Path"),
        "video": ("path", "Path"),
        "text": ("text", "Text"),
        "line": ("pos", "Pos"),
    }.get(kind, ())
    if not required_keys or any(key in root for key in required_keys):
        return root

    out = dict(root)
    for state in states:
        for key in required_keys:
            if key in state:
                out[key] = state[key]
                return out
    return out


def emit_timeline_object(kind: str, obj: dict, index: int) -> list[str]:
    states = obj.get("states") or obj.get("States") or []
    root = {k: v for k, v in obj.items() if k not in ("states", "States")}
    root = hoist_required_stage_field(kind, root, states)
    var_name = f"{kind}_{index}"
    lines = [f"local {var_name} = sb.{kind} {fmt_inline_table(root)}"]
    for st in states:
        if "add_time" in st or "AddTime" in st:
            add = st.get("add_time", st.get("AddTime"))
            patch = {
                k: v
                for k, v in st.items()
                if k not in ("time", "Time", "add_time", "AddTime")
            }
            patch_s = fmt_inline_table(patch) if patch else "{}"
            lines.append(f"{var_name}:rel({add}, {patch_s})")
        else:
            t = st.get("time", st.get("Time", 0))
            patch = {k: v for k, v in st.items() if k not in ("time", "Time")}
            patch_s = fmt_inline_table(patch) if patch else "{}"
            lines.append(f"{var_name}:key({t}, {patch_s})")
    lines.append("")
    return lines


def emit_triggers(triggers: list[dict]) -> list[str]:
    lines = []
    for trigger in triggers:
        lines.append(f"sb.trigger {fmt_inline_table(trigger)}")
    if lines:
        lines.append("")
    return lines


def generate(json_path: Path, level_id: str) -> str:
    doc = json.loads(json_path.read_text(encoding="utf-8"))
    lines = [
        f"-- Recreates `{level_id}` storyboard from Lab cache.",
        f"-- Source: {json_path.name}",
        "-- Run: cargo run -p cytoid-sb-cli -- check examples/{}/storyboard.lua".format(
            level_id.replace(".", "_")
        ),
        "",
    ]

    if doc.get("note_controllers"):
        lines.extend(emit_note_controllers(doc["note_controllers"]))

    for ctrl in doc.get("controllers", []):
        states = ctrl.get("states") or ctrl.get("States") or []
        root = {k: v for k, v in ctrl.items() if k not in ("states", "States")}
        if root and not states:
            lines.extend(emit_inline_controller(root))
        elif states:
            name = f"ctrl_{len(lines)}"
            lines.extend(emit_controller_states(states, name))
        elif root:
            lines.extend(emit_inline_controller(root))

    for kind in ("text", "video", "line", "sprite"):
        plural = kind + "s"
        for index, obj in enumerate(doc.get(plural, [])):
            lines.extend(emit_timeline_object(kind, obj, index))

    if doc.get("triggers"):
        lines.extend(emit_triggers(doc["triggers"]))

    return "\n".join(lines).rstrip() + "\n"


def main() -> int:
    if len(sys.argv) < 3:
        print("usage: gen_lua_from_sb.py <storyboard.json> <output.lua> [level-id]", file=sys.stderr)
        return 2
    src = Path(sys.argv[1])
    dst = Path(sys.argv[2])
    level_id = sys.argv[3] if len(sys.argv) > 3 else src.parent.name
    dst.parent.mkdir(parents=True, exist_ok=True)
    dst.write_text(generate(src, level_id), encoding="utf-8")
    print(f"wrote {dst}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
