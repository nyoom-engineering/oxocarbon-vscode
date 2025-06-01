CARGO      := cargo
PROG       := target/release/toml2json
INPUT      := oxocarbon.toml
OUTDIR     := themes
OUTFILE    := $(OUTDIR)/oxocarbon-color-theme.json
ASSETS     := assets
CURSOR_CFG := ~/Library/Application\ Support/Cursor/User

.PHONY: all build clean dotfiles

all: build $(OUTFILE)

build:
	$(CARGO) build --release

$(OUTFILE): build $(INPUT)
	mkdir -p $(OUTDIR)
	$(PROG) $(INPUT) > $(OUTFILE)

clean:
	$(CARGO) clean
	rm -f $(OUTFILE)

dotfiles:
	mkdir -p $(ASSETS)
	cp $(CURSOR_CFG)/settings.json $(ASSETS)/settings.json
	cp $(CURSOR_CFG)/keybindings.json $(ASSETS)/keybindings.json
