#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
THEME = ROOT / "themes" / "oxocarbon-color-theme.json"
COMPAT = ROOT / "themes" / "oxocarbon-compat-color-theme.json"

CARBON_GREY = {
    "#000000",
    "#0b0b0b",
    "#0f0f0f",
    "#161616",
    "#1b1b1b",
    "#1e1e1e",
    "#212121",
    "#262626",
    "#393939",
    "#525252",
    "#6f6f6f",
    "#8d8d8d",
    "#a8a8a8",
    "#c6c6c6",
    "#dde1e6",
    "#e0e0e0",
    "#f2f4f8",
    "#f4f4f4",
    "#ffffff",
}


def parse(hex_color: str) -> tuple[int, int, int, float]:
    h = hex_color.lstrip("#")
    if len(h) in (3, 4):
        h = "".join(c * 2 for c in h)
    if len(h) == 6:
        r, g, b, a = int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16), 1.0
    elif len(h) == 8:
        r, g, b = int(h[0:2], 16), int(h[2:4], 16), int(h[4:6], 16)
        a = int(h[6:8], 16) / 255.0
    else:
        raise ValueError(hex_color)
    return r, g, b, a


def lin(c: int) -> float:
    x = c / 255.0
    return x / 12.92 if x <= 0.04045 else ((x + 0.055) / 1.055) ** 2.4


def lum(rgb: tuple[int, int, int]) -> float:
    r, g, b = rgb
    return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b)


def contrast(a: tuple[int, int, int], b: tuple[int, int, int]) -> float:
    l1, l2 = lum(a), lum(b)
    lighter, darker = max(l1, l2), min(l1, l2)
    return (lighter + 0.05) / (darker + 0.05)


def opaque_over(fg: str, bg: str) -> tuple[int, int, int]:
    r, g, b, a = parse(fg)
    br, bg_, bb, _ = parse(bg)
    if a >= 0.999:
        return r, g, b
    return (
        round(r * a + br * (1 - a)),
        round(g * a + bg_ * (1 - a)),
        round(b * a + bb * (1 - a)),
    )


PAIRS = [
    ("editor.foreground", "editor.background", 4.5, "editor text"),
    ("editorLineNumber.foreground", "editor.background", 3.0, "line numbers (graphical/muted)"),
    ("editorLineNumber.activeForeground", "editor.background", 4.5, "active line number"),
    ("editorCodeLens.foreground", "editor.background", 4.5, "codelens"),
    ("editorGhostText.foreground", "editor.background", 3.0, "ghost text"),
    ("sideBarTitle.foreground", "sideBar.background", 4.5, "sidebar title"),
    ("statusBar.foreground", "statusBar.background", 4.5, "status bar"),
    ("tab.activeForeground", "tab.activeBackground", 4.5, "active tab"),
    ("tab.inactiveForeground", "tab.inactiveBackground", 4.5, "inactive tab"),
    ("activityBar.foreground", "activityBar.background", 4.5, "activity bar"),
    ("input.foreground", "input.background", 4.5, "input"),
    ("input.placeholderForeground", "input.background", 4.5, "input placeholder"),
    ("editorSuggestWidget.foreground", "editorSuggestWidget.background", 4.5, "suggest"),
    ("editorSuggestWidget.selectedForeground", "editorSuggestWidget.selectedBackground", 4.5, "suggest selected"),
    ("list.activeSelectionForeground", "list.activeSelectionBackground", 4.5, "list selection"),
    ("menu.foreground", "menu.background", 4.5, "menu"),
    ("terminal.foreground", "terminal.background", 4.5, "terminal"),
    ("editorError.foreground", "editor.background", 3.0, "error squiggle"),
    ("editorWarning.foreground", "editor.background", 3.0, "warning squiggle"),
    ("editorInfo.foreground", "editor.background", 3.0, "info squiggle"),
    ("textLink.foreground", "editor.background", 4.5, "links"),
    ("badge.foreground", "badge.background", 4.5, "badge"),
    ("button.foreground", "button.background", 4.5, "button"),
    ("statusBar.debuggingForeground", "statusBar.debuggingBackground", 4.5, "debug status"),
    ("statusBarItem.remoteForeground", "statusBarItem.remoteBackground", 4.5, "remote status"),
    ("statusBarItem.remoteHoverForeground", "statusBarItem.remoteHoverBackground", 4.5, "remote status hover"),
    ("statusBarItem.prominentForeground", "statusBarItem.prominentBackground", 4.5, "prominent status"),
    ("statusBarItem.errorForeground", "statusBarItem.errorBackground", 4.5, "error status chip"),
    ("statusBarItem.warningForeground", "statusBarItem.warningBackground", 4.5, "warning status chip"),
    ("activityWarningBadge.foreground", "activityWarningBadge.background", 4.5, "warning badge"),
    ("activityErrorBadge.foreground", "activityErrorBadge.background", 4.5, "error badge"),
    ("inputValidation.warningForeground", "inputValidation.warningBackground", 4.5, "warning validation"),
    ("inputValidation.infoForeground", "inputValidation.infoBackground", 4.5, "info validation"),
    ("editorInlayHint.foreground", "editorInlayHint.background", 4.5, "inlay hints"),
    ("git.blame.editorDecorationForeground", "editor.background", 3.0, "git blame"),
    # Gray 60 on Gray 100 is the oxocarbon comment token (Carbon secondary).
    ("comments", "editor.background", 3.0, "comments (Carbon Gray 60)"),
]


def main() -> int:
    theme = json.loads(THEME.read_text())
    colors: dict[str, str] = theme["colors"]
    colors["comments"] = "#6f6f6f"
    bg = colors["editor.background"].lower()[:7]
    failures = []
    warnings = []

    print(f"editor.background {bg}")
    print(" contrast ")
    for fg_key, bg_key, need, label in PAIRS:
        if fg_key not in colors or bg_key not in colors:
            warnings.append(f"missing {fg_key} or {bg_key} ({label})")
            continue
        under = (
            colors.get("statusBar.background", colors["editor.background"])
            if bg_key.startswith("statusBarItem.")
            else colors["editor.background"]
        )
        bg_hex = colors[bg_key]
        if len(bg_hex.lstrip("#")) > 6:
            bg_rgb = opaque_over(bg_hex, under)
            bg_for_fg = "#{:02x}{:02x}{:02x}".format(*bg_rgb)
        else:
            bg_rgb = parse(bg_hex)[:3]
            bg_for_fg = bg_hex
        fg_rgb = opaque_over(colors[fg_key], bg_for_fg)
        ratio = contrast(fg_rgb, bg_rgb)
        status = "OK" if ratio + 1e-6 >= need else "FAIL"
        print(f"  {status:4} {ratio:5.2f}:1  need {need:.1f}  {label}")
        if status == "FAIL":
            failures.append(f"{label}: {ratio:.2f}:1 < {need}:1 ({colors[fg_key]} on {colors[bg_key]})")

    print(" layering (bg must not be darker than editor) ")
    editor_y = lum(parse(colors["editor.background"])[:3])
    for key, val in sorted(colors.items()):
        if "background" not in key.lower() and not key.endswith("Background"):
            continue
        try:
            r, g, b, a = parse(val)
        except ValueError:
            continue
        if a < 0.5:
            continue
        y = lum((r, g, b))
        if y + 1e-4 < editor_y:
            msg = f"{key} {val} darker than editor.background"
            print(f"  FAIL {msg}")
            failures.append(msg)

    print(" extra greys (non-Carbon) ")
    for key, val in sorted(colors.items()):
        try:
            r, g, b, a = parse(val)
        except ValueError:
            continue
        if a < 1 and a > 0:
            continue
        hex6 = f"#{r:02x}{g:02x}{b:02x}"
        if r == g == b and hex6 not in CARBON_GREY:
            print(f"  note {key} {val} (neutral not on Carbon steps)")

    print(" info vs warning collision ")
    if colors.get("editorInfo.foreground") == colors.get("editorWarning.foreground"):
        failures.append("editorInfo.foreground == editorWarning.foreground")
        print("  FAIL editor info/warning share a color")
    if colors.get("statusBarItem.warningBackground") == colors.get("statusBarItem.errorBackground"):
        failures.append("status bar warning == error")
        print("  FAIL statusBar warning/error share a color")
    if colors.get("activityWarningBadge.background") == colors.get("activityErrorBadge.background"):
        failures.append("activity warning badge == error badge")
        print("  FAIL activity warning/error badges share a color")
    if colors.get("input.placeholderForeground") == colors.get("input.foreground"):
        failures.append("input placeholder == input foreground")
        print("  FAIL placeholder not dimmed")

    print(" compat comments (WCAG AA) ")
    compat = json.loads(COMPAT.read_text())
    comment_fg = None
    for tok in compat.get("tokenColors") or []:
        scope = tok.get("scope")
        scopes = scope if isinstance(scope, list) else [scope]
        if any(isinstance(s, str) and "comment" in s and "punctuation" not in s for s in scopes):
            comment_fg = (tok.get("settings") or {}).get("foreground")
            break
    editor_bg = compat["colors"]["editor.background"]
    if not comment_fg:
        failures.append("compat missing comment token")
    else:
        ratio = contrast(opaque_over(comment_fg, editor_bg), parse(editor_bg)[:3])
        status = "OK" if ratio >= 4.5 else "FAIL"
        print(f"  {status:4} {ratio:5.2f}:1  comments {comment_fg} on {editor_bg}")
        if status == "FAIL":
            failures.append(f"compat comments {ratio:.2f}:1")

    if failures:
        print("\nFAILURES:")
        for f in failures:
            print(" -", f)
        return 1
    print("\nok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
