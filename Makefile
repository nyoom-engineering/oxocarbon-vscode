CARGO      := cargo
PROG       := target/release/oxocarbon-themec
INPUT      := oxocarbon.toml
OUTDIR     := themes

# Oled
OUTFILE    := $(OUTDIR)/oxocarbon-color-theme.json
OLEDFILE   := $(OUTDIR)/oxocarbon-oled-color-theme.json

# Compatibility
COMPATFILE := $(OUTDIR)/oxocarbon-compat-color-theme.json
COMPATOLED := $(OUTDIR)/oxocarbon-oled-compat-color-theme.json

# Monochrome (default family: Gray)
MONOFILE        := $(OUTDIR)/oxocarbon-mono-color-theme.json
MONOOLED        := $(OUTDIR)/oxocarbon-mono-oled-color-theme.json
MONOCOMPAT      := $(OUTDIR)/oxocarbon-mono-compat-color-theme.json
MONOCOMPATOLED  := $(OUTDIR)/oxocarbon-mono-oled-compat-color-theme.json

# Optional monochrome families (Cool Gray, Warm Gray)
MONO_COOL           := $(OUTDIR)/oxocarbon-mono-coolgray-color-theme.json
MONO_COOL_OLED      := $(OUTDIR)/oxocarbon-mono-coolgray-oled-color-theme.json
MONO_COOL_COMPAT    := $(OUTDIR)/oxocarbon-mono-coolgray-compat-color-theme.json
MONO_COOL_COMPATOLED:= $(OUTDIR)/oxocarbon-mono-coolgray-oled-compat-color-theme.json

MONO_WARM           := $(OUTDIR)/oxocarbon-mono-warmgray-color-theme.json
MONO_WARM_OLED      := $(OUTDIR)/oxocarbon-mono-warmgray-oled-color-theme.json
MONO_WARM_COMPAT    := $(OUTDIR)/oxocarbon-mono-warmgray-compat-color-theme.json
MONO_WARM_COMPATOLED:= $(OUTDIR)/oxocarbon-mono-warmgray-oled-compat-color-theme.json
ASSETS     := assets
CURSOR_CFG := ~/Library/Application\ Support/Cursor/User
EXTENSIONS := $(ASSETS)/extensions.txt

.PHONY: all build clean dotfiles install mono-coolgray mono-warmgray

all: build \
    $(OUTFILE) \
    $(OLEDFILE) \
    $(COMPATFILE) \
    $(COMPATOLED) \
    $(MONOFILE) \
    $(MONOOLED) \
    $(MONOCOMPAT) \
    $(MONOCOMPATOLED)

build:
	$(CARGO) build --release

$(OUTFILE): build $(INPUT) | $(OUTDIR)
	$(PROG) $(INPUT) > $(OUTFILE)

$(OLEDFILE): build $(INPUT) | $(OUTDIR)
	$(PROG) --oled $(INPUT) > $(OLEDFILE)

$(COMPATFILE): build $(INPUT) | $(OUTDIR)
	$(PROG) --compat $(INPUT) > $(COMPATFILE)

$(COMPATOLED): build $(INPUT) | $(OUTDIR)
	$(PROG) --compat --oled $(INPUT) > $(COMPATOLED)

$(MONOFILE): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome $(INPUT) > $(MONOFILE)

$(MONOOLED): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --oled $(INPUT) > $(MONOOLED)

$(MONOCOMPAT): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --compat $(INPUT) > $(MONOCOMPAT)

$(MONOCOMPATOLED): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --compat --oled $(INPUT) > $(MONOCOMPATOLED)

# Monochrome (Cool Gray)
mono-coolgray: $(MONO_COOL) $(MONO_COOL_OLED) $(MONO_COOL_COMPAT) $(MONO_COOL_COMPATOLED)

$(MONO_COOL): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --monochrome-family coolgray $(INPUT) > $(MONO_COOL)

$(MONO_COOL_OLED): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --monochrome-family coolgray --oled $(INPUT) > $(MONO_COOL_OLED)

$(MONO_COOL_COMPAT): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --monochrome-family coolgray --compat $(INPUT) > $(MONO_COOL_COMPAT)

$(MONO_COOL_COMPATOLED): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --monochrome-family coolgray --compat --oled $(INPUT) > $(MONO_COOL_COMPATOLED)

# Monochrome (Warm Gray)
mono-warmgray: $(MONO_WARM) $(MONO_WARM_OLED) $(MONO_WARM_COMPAT) $(MONO_WARM_COMPATOLED)

$(MONO_WARM): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --monochrome-family warmgray $(INPUT) > $(MONO_WARM)

$(MONO_WARM_OLED): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --monochrome-family warmgray --oled $(INPUT) > $(MONO_WARM_OLED)

$(MONO_WARM_COMPAT): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --monochrome-family warmgray --compat $(INPUT) > $(MONO_WARM_COMPAT)

$(MONO_WARM_COMPATOLED): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --monochrome-family warmgray --compat --oled $(INPUT) > $(MONO_WARM_COMPATOLED)

$(OUTDIR):
	mkdir -p $(OUTDIR)

clean:
	$(CARGO) clean
	rm -f $(OUTFILE) $(OLEDFILE) $(COMPATFILE) $(COMPATOLED) \
		$(MONOFILE) $(MONOOLED) $(MONOCOMPAT) $(MONOCOMPATOLED) \
		$(MONO_COOL) $(MONO_COOL_OLED) $(MONO_COOL_COMPAT) $(MONO_COOL_COMPATOLED) \
		$(MONO_WARM) $(MONO_WARM_OLED) $(MONO_WARM_COMPAT) $(MONO_WARM_COMPATOLED)

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