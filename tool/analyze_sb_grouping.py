#!/usr/bin/env python3
"""Analyze compiled storyboard JSON for sprite / note_controller grouping."""

from __future__ import annotations

import json
import re
import sys
from collections import defaultdict
from pathlib import Path


def note_from_sprite(s):
    for st in s.get("states", []):
        t = st.get("time", "")
        if isinstance(t, str) and t.startswith("start:"):
            m = re.match(r"start:(\d+)", t)
            if m:
                return int(m.group(1))
    return None


def normalize_states(states, note):
    out = []
    for st in states:
        nst = {}
        for k, v in st.items():
            if k == "time" and isinstance(v, str):
                nst[k] = v.replace(f"start:{note}", "start:NOTE").replace(f"end:{note}", "end:NOTE")
            else:
                nst[k] = v
        out.append(json.dumps(nst, sort_keys=True))
    return tuple(out)


def main() -> int:
    if len(sys.argv) != 2:
        print("usage: analyze_sb_grouping.py <compiled.json>", file=sys.stderr)
        return 2
    doc = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))

    groups = defaultdict(list)
    for i, s in enumerate(doc["sprites"]):
        root = {k: v for k, v in s.items() if k != "states"}
        key = json.dumps(root, sort_keys=True)
        groups[key].append(i)

    for root_json, idxs in sorted(groups.items(), key=lambda x: -len(x[1])):
        root = json.loads(root_json)
        notes = [note_from_sprite(doc["sprites"][i]) for i in idxs]
        sigs = {
            normalize_states(doc["sprites"][i]["states"], notes[j])
            for j, i in enumerate(idxs)
        }
        print(
            f"path={root.get('path')} n={len(idxs)} states={len(doc['sprites'][idxs[0]]['states'])} "
            f"unique_motion={len(sigs)} notes={notes[:5]}..."
        )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
