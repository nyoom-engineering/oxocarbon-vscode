# <img src="./assets/output-3840x1330-shadow.png"> 

WIP WIP WIP

Oxocarbon is a High contrast accessible colorscheme inspired by IBM Carbon. It delivers class-leading readability without strain by adhering to WCAG 2.1 guidelines

## Features

- TODO

### Plugin Support

- TODO 

## Installation

### Automatic

- TODO https://code.visualstudio.com/api/working-with-extensions/publishing-extension

### Manual

- Download repository source as ZIP
- Unpack in `~/.vscode/extensions` (VSCode) or `~/.cursor/extensions` (Cursor)
- Installation Instructions
- clone to `~/.vscode/extensions`

### Recommended VSCode Settings

Personal Opinionated `settings.json`. Recommended to install the `icons-carbon` extension and `Liga SF Nerd Font` font beforehand. 

```
    "editor.fontFamily": "Liga SF Nerd Font, monospace",
    "editor.fontLigatures": false,
    "editor.fontSize": 14,

    "editor.wordWrap": "off",
    "editor.lineNumbers": "off",
    "editor.bracketPairColorization.enabled": false,
    "editor.guides.indentation": false,

    "workbench.layoutControl.enabled": false,
    "workbench.productIconTheme": "icons-carbon",
    "workbench.iconTheme": null,
    "workbench.colorTheme": "oxocarbon",

    "window.customTitleBarVisibility": "windowed",
    "zenMode.showTabs": "none",
```

### Development

The following requires `Cargo`/`Rust`. Changes should be made in `oxocarbon.toml`

To generate the JSON file, run `make` in the root directory. To test the colorscheme, press `F5`

Reference https://code.visualstudio.com/api/references/theme-color#editor-widget-colors and https://code.visualstudio.com/api/language-extensions/semantic-highlight-guide for highlight groups

Additionally, add the following in your `keybindings.json` to use `cmd+shift+i` to inspect the highlight at cursor

```
    {
        "key": "cmd+shift+i",
        "command": "editor.action.inspectTMScopes"
    }
```

## Contributing

Before contributing, its recommended to read through the [style guide](https://github.com/nyoom-engineering/oxocarbon/blob/main/docs/style-guide.md). Discussion primarily takes place on the Nyoom Engineering discord server: https://discord.gg/M528tDKXRG

## License

The project is vendored under the MIT license