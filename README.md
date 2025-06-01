# <img src="./assets/output-3840x1330-shadow.png"> 

WIP WIP WIP

Oxocarbon is a High contrast accessible colorscheme inspired by IBM Carbon. It delivers class-leading readability without strain by adhering to WCAG 2.1 guidelines

## Features (optional)

- Special features and UI tweaks

### Plugin Support (optional)

- What plugins does it support

## Installation

- Installation Instructions
- clone to `~/.vscode/extensions`
- TODO https://code.visualstudio.com/api/working-with-extensions/publishing-extension

### Recommended VSCode Settings

Personal Opinionated Config. Recommended to install the `icons-carbon` extension and `Liga SF Nerd Font` font beforehand. 

```
"editor.fontFamily": "Liga SF Nerd Font, monospace",
"editor.fontLigatures": true,
"editor.fontSize": 14,

"editor.wordWrap": "off",
"editor.lineNumbers": "off",
"editor.bracketPairColorization.enabled": true,
"editor.guides.indentation": false,

"workbench.layoutControl.enabled": false,
"workbench.productIconTheme": "icons-carbon",
"workbench.iconTheme": null,
```

### Development

The following requires `Cargo`/`Rust`. Changes should be made in `oxocarbon.toml`

```
make
```

Open in VSCode, press `F5`

## Contributing

Before contributing, its recommended to read through the [style guide](https://github.com/nyoom-engineering/oxocarbon/blob/main/docs/style-guide.md). Discussion primarily takes place on the Nyoom Engineering discord server: https://discord.gg/M528tDKXRG

## License

The project is vendored under the MIT license