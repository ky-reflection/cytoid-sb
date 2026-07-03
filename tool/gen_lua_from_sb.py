#!/usr/bin/env python3
"""Generate a Lua storyboard source from a Lab/uncompiled JSON file."""

from __future__ import annotations

import json
import sys
from pathlib import Path


def lua_string(s: str) -> str:
    return json.dumps(s, ensure_ascii=False)


LUA_RESERVED = {
    "and",
    "break",
    "do",
    "else",
    "elseif",
    "end",
    "false",
    "for",
    "function",
    "goto",
    "if",
    "in",
    "local",
    "nil",
    "not",
    "or",
    "repeat",
    "return",
    "then",
    "true",
    "until",
    "while",
}


def lua_key(key: str) -> str:
    if key in LUA_RESERVED:
        return f"[{lua_string(key)}]"
    return key


def lua_value(value):
    if isinstance(value, bool):
        return "true" if value else "false"
    if value is None:
        return "sb.null"
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
        return emit_lua_table(value)
    return lua_string(str(value))


def emit_lua_table(obj: dict) -> str:
    parts = []
    for key, value in obj.items():
        parts.append(f"{lua_key(key)} = {lua_value(value)}")
    return "{ " + ", ".join(parts) + " }"


def load_doc(path: Path) -> dict:
    text = path.read_text(encoding="utf-8")
    try:
        return json.loads(text)
    except json.JSONDecodeError:
        try:
            import json5  # type: ignore
        except ImportError as exc:
            raise SystemExit(
                "input uses JSON5 comments; install json5 or compile to strict JSON first"
            ) from exc
        return json5.loads(text)


def emit_note_controllers(note_controllers: list[dict]) -> list[str]:
    lines = [
        "-- Note lane overrides",
        "local note_controllers = {",
    ]
    for nc in note_controllers:
        items = ", ".join(f"{lua_key(k)} = {lua_value(v)}" for k, v in nc.items())
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


def emit_controller_states(states: list[dict], var_name: str, root: dict | None = None) -> list[str]:
    lines = [f"local {var_name} = {{"]
    for st in states:
        items = []
        for k, v in st.items():
            items.append(f"{lua_key(k)} = {lua_value(v)}")
        lines.append("  { " + ", ".join(items) + " },")
    lines.extend(
        [
            "}",
        ]
    )
    if root:
        lines.append(f"local {var_name}_handle = sb.controller {emit_lua_table(root)}")
    else:
        lines.append(f"local {var_name}_handle = sb.controller {{}}")
    lines.extend(
        [
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
    return [f"sb.controller {emit_lua_table(obj)}", ""]


def emit_controller(ctrl: dict, index: int) -> list[str]:
    states = ctrl.get("states") or ctrl.get("States") or []
    root = {k: v for k, v in ctrl.items() if k not in ("states", "States")}
    if not states:
        return emit_inline_controller(root)
    return [f"sb.controller {emit_lua_table(ctrl)}", ""]


def emit_stage_object(kind: str, obj: dict) -> list[str]:
    return [f"sb.{kind} {emit_lua_table(obj)}", ""]


def emit_templates(templates: dict) -> list[str]:
    lines = ["-- Templates"]
    for name, spec in templates.items():
        lines.append(f"sb.template({lua_string(name)}, {emit_lua_table(spec)})")
    lines.append("")
    return lines


def emit_triggers(triggers: list[dict]) -> list[str]:
    lines = []
    for trigger in triggers:
        lines.append(f"sb.trigger {emit_lua_table(trigger)}")
    if lines:
        lines.append("")
    return lines


def generate(json_path: Path, level_id: str) -> str:
    doc = load_doc(json_path)
    lines = [
        f"-- Recreates `{level_id}` storyboard from Lab cache.",
        f"-- Source: {json_path.name}",
        "-- Run: cargo run -p cytoid-sb-cli -- check examples/{}/storyboard.lua".format(
            level_id.replace(".", "_")
        ),
        "",
    ]

    if doc.get("templates"):
        lines.extend(emit_templates(doc["templates"]))

    for kind in ("text", "video", "line", "sprite"):
        plural = kind + "s"
        for obj in doc.get(plural, []):
            lines.extend(emit_stage_object(kind, obj))

    for index, ctrl in enumerate(doc.get("controllers", [])):
        lines.extend(emit_controller(ctrl, index))

    if doc.get("note_controllers"):
        lines.extend(emit_note_controllers(doc["note_controllers"]))

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
