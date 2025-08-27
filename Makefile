CARGO      := cargo
PROG       := target/release/toml2json
INPUT      := oxocarbon.toml
OUTDIR     := themes
OUTFILE    := $(OUTDIR)/oxocarbon-color-theme.json
OLEDFILE   := $(OUTDIR)/oxocarbon-oled-color-theme.json
COMPATFILE := $(OUTDIR)/oxocarbon-compat-color-theme.json
COMPATOLED := $(OUTDIR)/oxocarbon-oled-compat-color-theme.json
ASSETS     := assets
CURSOR_CFG := ~/Library/Application\ Support/Cursor/User
EXTENSIONS := $(ASSETS)/extensions.txt

.PHONY: all build clean dotfiles install

all: build $(OUTFILE) $(OLEDFILE) $(COMPATFILE) $(COMPATOLED)

build:
	$(CARGO) build --release

$(OUTFILE): build $(INPUT)
	mkdir -p $(OUTDIR)
	$(PROG) $(INPUT) > $(OUTFILE)

$(OLEDFILE): build $(INPUT)
	mkdir -p $(OUTDIR)
	$(PROG) --oled $(INPUT) > $(OLEDFILE)

$(COMPATFILE): build $(INPUT)
	mkdir -p $(OUTDIR)
	$(PROG) --compat $(INPUT) > $(COMPATFILE)

$(COMPATOLED): build $(INPUT)
	mkdir -p $(OUTDIR)
	$(PROG) --compat --oled $(INPUT) > $(COMPATOLED)

clean:
	$(CARGO) clean
	rm -f $(OUTFILE) $(OLEDFILE) $(COMPATFILE) $(COMPATOLED)

dotfiles:
	mkdir -p $(ASSETS)
	cursor --list-extensions > $(EXTENSIONS)
	cp $(CURSOR_CFG)/settings.json $(ASSETS)/settings.json
	cp $(CURSOR_CFG)/keybindings.json $(ASSETS)/keybindings.json

install: dotfiles
	cat $(EXTENSIONS) | xargs -I {} cursor --install-extension {}
	mkdir -p $(CURSOR_CFG)
	cp $(ASSETS)/settings.json $(CURSOR_CFG)/
	cp $(ASSETS)/keybindings.json $(CURSOR_CFG)/