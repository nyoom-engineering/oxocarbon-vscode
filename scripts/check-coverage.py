#!/usr/bin/env python3
"""Assert generated theme JSON contains clash keys and omits inherit-only ones."""
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
THEME = ROOT / "themes" / "oxocarbon-color-theme.json"
REQUIRED = ROOT / "dev/fixtures/vscode-2026-dark.required.keys"

# Registry inherit is Carbon-correct, or the user rejected the override.
FORBIDDEN = (
    "quickInput.background",
    "quickInput.foreground",
    "quickInputList.focusBackground",
    "quickInputList.focusForeground",
    "quickInputList.focusHighlightForeground",
    "breadcrumbPicker.background",
    "agentsUnreadBadge.background",
    "agentsUnreadBadge.foreground",
    # Settings option rows: inherit list.hover + 70% header dim. Do not author.
    "settings.rowHoverBackground",
    "settings.focusedRowBackground",
    "settings.focusedRowBorder",
    "settings.settingsHeaderHoverForeground",
)


def load_required(path: Path) -> list[str]:
    keys: list[str] = []
    for raw in path.read_text().splitlines():
        line = raw.split("#", 1)[0].strip()
        if line:
            keys.append(line)
    return keys


def main() -> int:
    if not THEME.exists():
        print(f"missing {THEME}")
        return 1
    colors = json.loads(THEME.read_text())["colors"]
    required = load_required(REQUIRED)
    missing = [k for k in required if k not in colors]
    forbidden_hit = [k for k in FORBIDDEN if k in colors]
    status = 0
    if missing:
        print("MISSING required keys:")
        for k in missing:
            print(f"  {k}")
        status = 1
    if forbidden_hit:
        print("FORBIDDEN keys present (should inherit):")
        for k in forbidden_hit:
            print(f"  {k}")
        status = 1
    print(f"checked {len(required)} required, {len(colors)} authored")
    if status == 0:
        print("OK")
    return status


if __name__ == "__main__":
    sys.exit(main())
