CARGO      := cargo
PROG       := target/release/oxocarbon-themec
INPUT      := oxocarbon.toml
OUTDIR     := themes

# Zed export
ZED_REPO_URL := https://github.com/zed-industries/zed.git
ZED_SRC_DIR  := target/zed
ZED_OUTDIR   := out

# Oled
OUTFILE    := $(OUTDIR)/oxocarbon-color-theme.json
OLEDFILE   := $(OUTDIR)/oxocarbon-oled-color-theme.json

# Compatibility
COMPATFILE := $(OUTDIR)/oxocarbon-compat-color-theme.json
COMPATOLED := $(OUTDIR)/oxocarbon-oled-compat-color-theme.json

# Monochrome (default family: Gray)
MONOFILE        := $(OUTDIR)/oxocarbon-mono-color-theme.json
MONOOLED        := $(OUTDIR)/oxocarbon-oled-mono-color-theme.json
MONOCOMPAT      := $(OUTDIR)/oxocarbon-mono-compat-color-theme.json
MONOCOMPATOLED  := $(OUTDIR)/oxocarbon-oled-mono-compat-color-theme.json
PRINTFILE       := $(OUTDIR)/PRINT.json

# Optional monochrome families (Cool Gray, Warm Gray)
MONO_COOL           := $(OUTDIR)/oxocarbon-mono-coolgray-color-theme.json
MONO_COOL_OLED      := $(OUTDIR)/oxocarbon-oled-mono-coolgray-color-theme.json
MONO_COOL_COMPAT    := $(OUTDIR)/oxocarbon-mono-coolgray-compat-color-theme.json
MONO_COOL_COMPATOLED:= $(OUTDIR)/oxocarbon-oled-mono-coolgray-compat-color-theme.json

MONO_WARM           := $(OUTDIR)/oxocarbon-mono-warmgray-color-theme.json
MONO_WARM_OLED      := $(OUTDIR)/oxocarbon-oled-mono-warmgray-color-theme.json
MONO_WARM_COMPAT    := $(OUTDIR)/oxocarbon-mono-warmgray-compat-color-theme.json
MONO_WARM_COMPATOLED:= $(OUTDIR)/oxocarbon-oled-mono-warmgray-compat-color-theme.json
ASSETS     := assets
CURSOR_CFG := ~/Library/Application\ Support/Cursor/User
EXTENSIONS := $(ASSETS)/extensions.txt

.PHONY: all build clean dotfiles install mono-coolgray mono-warmgray \
    zed zed-setup zed-import zed-bundle check-xcode check-jq

all: build \
    $(OUTFILE) \
    $(OLEDFILE) \
    $(COMPATFILE) \
    $(COMPATOLED) \
    $(MONOFILE) \
    $(MONOOLED) \
    $(MONOCOMPAT) \
    $(MONOCOMPATOLED) \
    $(PRINTFILE)

zed: all zed-bundle

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

$(PRINTFILE): build $(INPUT) | $(OUTDIR)
	$(PROG) --monochrome --oled --print $(INPUT) > $(PRINTFILE)

PRINT: $(PRINTFILE)

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

$(ZED_OUTDIR):
	mkdir -p $(ZED_OUTDIR)

check-xcode:
	@xcodebuild -version >/dev/null 2>&1 || { echo "Xcode is required. Please install Xcode from the App Store."; exit 1; }
	@xcrun -f metal >/dev/null 2>&1 || { echo "Metal toolchain not found. Run 'xcode-select --install' or install Xcode command line tools."; exit 1; }
	@xcrun -f metallib >/dev/null 2>&1 || { echo "Metal toolchain (metallib) not found."; exit 1; }
check-jq:
	@command -v jq >/dev/null 2>&1 || { echo "jq is required to bundle Zed themes. Install with: brew install jq"; exit 1; }

$(ZED_SRC_DIR): | check-xcode
	@if [ ! -d "$(ZED_SRC_DIR)/.git" ]; then \
		git clone --depth 1 $(ZED_REPO_URL) $(ZED_SRC_DIR); \
	else \
		git -C $(ZED_SRC_DIR) fetch --depth 1 origin main >/dev/null 2>&1 && \
		git -C $(ZED_SRC_DIR) reset --hard FETCH_HEAD >/dev/null 2>&1; \
	fi

# map vsc -> zed
THEMES_ALL      := $(wildcard $(OUTDIR)/*.json)
THEMES_INPUT    := $(filter-out $(PRINTFILE) $(OUTDIR)/oxocarbon-zed.json,$(THEMES_ALL))
ZED_THEMES_JSON := $(patsubst $(OUTDIR)/%,$(ZED_OUTDIR)/%,$(THEMES_INPUT))
ZED_BUNDLE      := $(OUTDIR)/oxocarbon-zed.json

zed-setup: check-xcode $(ZED_SRC_DIR) $(ZED_OUTDIR)

$(ZED_OUTDIR)/%.json: $(OUTDIR)/%.json | zed-setup
	cd $(ZED_SRC_DIR) && $(CARGO) run -p theme_importer -- $(abspath $<) --output $(abspath $@)

zed-import: $(ZED_THEMES_JSON)

$(ZED_BUNDLE): check-jq $(ZED_THEMES_JSON) | $(OUTDIR)
	jq -s '{\
	  "$schema": "https://zed.dev/schema/themes/v0.2.0.json",\
	  "name": "Oxocarbon",\
	  "author": "Nyoom Engineering",\
	  "themes": [ .[] | (.themes // .) | (if type=="array" then .[] else . end) ]\
	}' $(ZED_THEMES_JSON) > $(ZED_BUNDLE)

zed-bundle: $(ZED_BUNDLE)

clean:
	$(CARGO) clean
	rm -f $(OUTFILE) $(OLEDFILE) $(COMPATFILE) $(COMPATOLED) \
		$(MONOFILE) $(MONOOLED) $(MONOCOMPAT) $(MONOCOMPATOLED) $(PRINTFILE) \
		$(MONO_COOL) $(MONO_COOL_OLED) $(MONO_COOL_COMPAT) $(MONO_COOL_COMPATOLED) \
		$(MONO_WARM) $(MONO_WARM_OLED) $(MONO_WARM_COMPAT) $(MONO_WARM_COMPATOLED) \
		$(ZED_BUNDLE)
	rm -rf $(ZED_OUTDIR)

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