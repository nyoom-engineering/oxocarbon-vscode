# Change Log

All notable changes to the "oxocarbon-vscode" extension will be documented in this file.

Check [Keep a Changelog](http://keepachangelog.com/) for recommendations on how to structure this file.

## [Unreleased]

- `semanticTokenColors` match TextMate roles; comments stay on TM so compat can lift Gray 50
- Drop inherit-duplicate 1.136 keys (agents/surface/quickInput/sticky scroll) so compat remaps propagate
- Command palette stays on Gray 100 (`quickInput` inherits `editorWidget`); unread agents badge is not error magenta
- IBM Blue `#0f62fe` + white FG on remote/debug/noFolder/prominent, including remote hover
- Diagnostics follow Carbon g100 support: warning Yellow 30 `#f1c21b`, info Blue 50 `#4589ff`; error stays Magenta 50. Warning badges use Gray 100 on yellow (not white).
- Chat describe/edit uses canvas (`input.background` Gray 100); the tip notice is a Gray 90 menu card (`agentsChatInput`)
- Status error is Magenta 50 wash with status-bar FG; warning stays Yellow 30 text, no fill
- AI lightbulb inherits the regular bulb (no special purple)
- Intelligence and AI lightbulbs share `#dde1e6`; chrome icons Gray 30
- Hover widgets (`editorHoverWidget`) match menus: Gray 90 fill, Gray 80 border. Settings option rows inherit (labels dim on hover)
- Context attachment chips use Gray 80 `chat.requestBorder`
- Error chip is Magenta 40 on Magenta 50 wash, white on hover
- PRINT picker name `PRINT`; mono ramp remaps `#0f62fe`; trim trailing space on OLED monochrom label

## [1.3.0]

- Cover VS Code 1.136 / 2026 Dark: author clash keys (agents, command center, chat, inline edit); inherit breadcrumbs, quick input, and semantic tokens from parents / TextMate
- Kill stock current-line box, off-token indent guides, IBM-blue prominent status hover, and top activity-bar accent drift
- Debug/merge/settings/comment-unresolved clash keys; types/tags/invalid follow Carbon syntax roles
- Nix flake packages and checks; CI `nix flake check` + theme coverage
- Compiler: stdin, `--help`; packed-RGB OLED, binary-search ramps; rebuild on `make dev`

## [1.2.0]