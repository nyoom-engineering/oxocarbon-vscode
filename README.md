# <img src="./assets/output-3840x1330-shadow.png"> 

<figure>
  <img alt="" src="https://github.com/user-attachments/assets/0cb2aa5e-81ed-4b60-bfac-4bdba8249592" />
  <figcaption>Oxocarbon Dark</figcaption>
</figure>

<br>
<br>

<figure>
  <img alt="" src="https://github.com/user-attachments/assets/5b31e536-810e-44c1-814b-b3a99ae62bbe" />
  <figcaption>Oxocarbon OLED</figcaption>
</figure>

<br>
<br>

Oxocarbon is a High contrast accessible colorscheme inspired by IBM Carbon. It delivers class-leading readability without strain by adhering to WCAG 2.1 guidelines. Now with an OLED variant!

## Features

- Comprehensive semantic highlighting
- Carefully crafted color palette for maximum contrast and readability
- Support for various editor features:
  - Semantic tokens
  - Git decorations
  - Debug & Testing
  - Terminal colors
  - Status bar indicators
  - Editor widgets and overlays
  - Bracket pair colorization
  - Remote Development
  - Inlay hints
  - Peek view
  - Diff editor
  - Jupyter Notebook support
  - Quick input
  - Menu styling
  - Gauge indicators
  - Minimap customization
  - Banner styling
  - Cursor Chat

### Language Support

Any language with a textmate parser and/or semantic highlghting support is supported (i.e. almost all of them)

Oxocarbon also provides handrolled syntax highlighting for:

- C
- Rust
- Haskell
- OCaml
- Lisp
- TOML
- Markdown

## Installation

Install using your package manager of choice: [Oxocarbon Theme - Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=NyoomEngineering.oxocarbon-vscode)

Alternatively, a VSIX package can be found under [releases](https://github.com/nyoom-engineering/oxocarbon-vscode/releases)

### Manual

- Download repository source as ZIP
- Unpack in `~/.vscode/extensions` (VSCode) or `~/.cursor/extensions` (Cursor)
- Reload editor then set theme to either `oxocarbon` or `oxocarbon OLED`

### Additional Configuration

It is recommended to enable Semantic Parsing by default

```json
{
  "editor.semanticHighlighting.enabled": true,
}
```

Rust semantic parsing is buggy so it is recommended to default back to TextMate Parsing. In your `settings.json`

```json
{
  "[rust]": {
    "editor.semanticHighlighting.enabled": false,
  },
}
```

It is recommended to disable Bracket Pair Colorization by default and enable it on a case-by-case basis

```json
{
  "editor.bracketPairColorization.enabled": false,
  "[commonlisp]": {
    "editor.bracketPairColorization.enabled": true,
  },
}
```

(optional) Install the `Liga SFMono Nerd Font` font for the best experience

```json
{
  "editor.fontFamily": "Liga SFMono Nerd Font, monospace",
  "editor.fontLigatures": true,
  "editor.fontSize": 14,
}
```

On HiDPI/Retina displays, you may find text rendering improved by adjusting font ant-aliasing

```json
{
  "workbench.fontAliasing": "auto",
}
```

(optional) enable smooth scrolling and cursor effects

```json
{
  "editor.smoothScrolling": true,
  "editor.cursorBlinking": "smooth",
  "editor.cursorSmoothCaretAnimation": "on",

  "terminal.integrated.smoothScrolling": true,
  "terminal.integrated.cursorBlinking": true,
  "terminal.integrated.enableVisualBell": true,
}
```

An opinionated `settings.json`, `keybindings.json`, and list of extensions is also provided under `assets/` in the GitHub Repository. On UNIX systems, you may clone this repository, install Cursor, and run `make install` to intall the configuration.

## Development

The following requires `Cargo`/`Rust`. Changes should be made in `oxocarbon.toml`

To generate the JSON file, run `make` in the root directory. To test the colorscheme, press `F5`

Reference the [Theme Color Reference](https://code.visualstudio.com/api/references/theme-color#editor-widget-colors) & [Semantic Highlight Guide]*https://code.visualstudio.com/api/language-extensions/semantic-highlight-guide) for highlight groups

Additionally, add the following in your `keybindings.json` to use `cmd+shift+i` to inspect the highlight at cursor

```json
{
    "key": "cmd+shift+i",
    "command": "editor.action.inspectTMScopes"
}
```

## Contributing

Before contributing, its recommended to read through the [style guide](https://github.com/nyoom-engineering/oxocarbon/blob/main/docs/style-guide.md). Discussion primarily takes place on the [Nyoom Engineering Discord Server](https://discord.gg/M528tDKXRG)

## License

The project is vendored under the MIT license
