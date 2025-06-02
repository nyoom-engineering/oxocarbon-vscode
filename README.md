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
  - Inlay hints
  - Peek view
  - Diff editor
  - Jupyter Notebook support
  - Quick input
  - Menu styling
  - Gauge indicators
  - Minimap customization
  - Banner styling

### Language Support

Any language with a textmate parser and/or language server is supported (i.e. almost all of them)

Oxocarbon also provides handrolled syntax highlighting for:

- C/C++
- Rust
- OCaml
- Lisp
- TOML

## Installation

Install using your package manager of choice: [Oxocarbon Theme - Visual Studio Marketplace](https://marketplace.visualstudio.com/items?itemName=NyoomEngineering.oxocarbon-vscode)

Alternatively, a VSIX package can be found under [releases](https://github.com/nyoom-engineering/oxocarbon-vscode/releases)

### Manual

- Download repository source as ZIP
- Unpack in `~/.vscode/extensions` (VSCode) or `~/.cursor/extensions` (Cursor)
- Reload editor then set theme to either `oxocarbon` or `oxocarbon OLED`

### Recommended VSCode Settings

Personal Opinionated `settings.json`, `keybindings.json`, and list of extensions located under `assets/` in the GitHub Repository. On UNIX systems, you may clone this repository, install Cursor, and run `make install` to intall the configuration.

(optional) install the `Liga SFMono Nerd Font` font for the best experience

## Development

The following requires `Cargo`/`Rust`. Changes should be made in `oxocarbon.toml`

To generate the JSON file, run `make` in the root directory. To test the colorscheme, press `F5`

Reference https://code.visualstudio.com/api/references/theme-color#editor-widget-colors and https://code.visualstudio.com/api/language-extensions/semantic-highlight-guide for highlight groups

Additionally, add the following in your `keybindings.json` to use `cmd+shift+i` to inspect the highlight at cursor

```json
{
    "key": "cmd+shift+i",
    "command": "editor.action.inspectTMScopes"
}
```

## Contributing

Before contributing, its recommended to read through the [style guide](https://github.com/nyoom-engineering/oxocarbon/blob/main/docs/style-guide.md). Discussion primarily takes place on the Nyoom Engineering discord server: https://discord.gg/M528tDKXRG

## License

The project is vendored under the MIT license
