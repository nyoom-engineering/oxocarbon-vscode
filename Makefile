PROG := target/release/oxocarbon-themec
INPUT := oxocarbon.toml
OUTDIR := out
THEMESDIR := themes
ASSETS := assets

CURSOR_CFG := ~/Library/Application\ Support/Cursor/User
EXTENSIONS := $(ASSETS)/extensions.txt

ZED_REPO_URL := https://github.com/zed-industries/zed.git
ZED_SRC_DIR := target/zed
ZED_BUNDLE := $(THEMESDIR)/oxocarbon-zed.json
ZED_IMPORTER := $(ZED_SRC_DIR)/target/release/theme_importer

DEFAULT_THEMES := \
	$(THEMESDIR)/oxocarbon-color-theme.json \
	$(THEMESDIR)/oxocarbon-oled-color-theme.json \
	$(THEMESDIR)/oxocarbon-compat-color-theme.json \
	$(THEMESDIR)/oxocarbon-oled-compat-color-theme.json \
	$(THEMESDIR)/oxocarbon-mono-color-theme.json \
	$(THEMESDIR)/oxocarbon-oled-mono-color-theme.json \
	$(THEMESDIR)/oxocarbon-mono-compat-color-theme.json \
	$(THEMESDIR)/oxocarbon-oled-mono-compat-color-theme.json \
	$(THEMESDIR)/PRINT.json

.PHONY: all build clean dotfiles help install mono-coolgray mono-warmgray \
	PRINT zed zed-setup

.SECONDARY:

all: $(DEFAULT_THEMES)

build:
	cargo build --release

THEME_FLAGS = $(strip \
	$(if $(findstring oled,$@),--oled,) \
	$(if $(findstring compat,$@),--compat,) \
	$(if $(findstring -mono-,$@),--monochrome,) \
	$(if $(findstring -coolgray-,$@),--monochrome-family coolgray,) \
	$(if $(findstring -warmgray-,$@),--monochrome-family warmgray,))

$(THEMESDIR)/%.json: build $(INPUT) | $(THEMESDIR)
	$(PROG) $(THEME_FLAGS) $(INPUT) > $@

$(THEMESDIR)/PRINT.json: build $(INPUT) | $(THEMESDIR)
	$(PROG) --monochrome --oled --print $(INPUT) > $@

PRINT: $(THEMESDIR)/PRINT.json

mono-%: \
	$(THEMESDIR)/oxocarbon-mono-%-color-theme.json \
	$(THEMESDIR)/oxocarbon-oled-mono-%-color-theme.json \
	$(THEMESDIR)/oxocarbon-mono-%-compat-color-theme.json \
	$(THEMESDIR)/oxocarbon-oled-mono-%-compat-color-theme.json
	@:

mono-coolgray: \
	$(THEMESDIR)/oxocarbon-mono-coolgray-color-theme.json \
	$(THEMESDIR)/oxocarbon-oled-mono-coolgray-color-theme.json \
	$(THEMESDIR)/oxocarbon-mono-coolgray-compat-color-theme.json \
	$(THEMESDIR)/oxocarbon-oled-mono-coolgray-compat-color-theme.json

mono-warmgray: \
	$(THEMESDIR)/oxocarbon-mono-warmgray-color-theme.json \
	$(THEMESDIR)/oxocarbon-oled-mono-warmgray-color-theme.json \
	$(THEMESDIR)/oxocarbon-mono-warmgray-compat-color-theme.json \
	$(THEMESDIR)/oxocarbon-oled-mono-warmgray-compat-color-theme.json

$(OUTDIR) $(THEMESDIR):
	mkdir -p $@

check-xcode:
	@xcodebuild -version >/dev/null 2>&1 && \
		xcrun -f metal >/dev/null 2>&1 && \
		xcrun -f metallib >/dev/null 2>&1 || { \
		echo "Error: Xcode/Metal toolchain required"; \
		exit 1; \
	}

check-jq: ## Verify jq is available
	@command -v jq >/dev/null 2>&1 || { \
		echo "Error: jq required"; \
		exit 1; \
	}

zed: $(ZED_BUNDLE)

zed-setup: check-xcode $(ZED_SRC_DIR)

$(ZED_SRC_DIR): | check-xcode
	@[ -d "$(ZED_SRC_DIR)/.git" ] || \
		git clone --depth 1 $(ZED_REPO_URL) $(ZED_SRC_DIR); \
		git -C $(ZED_SRC_DIR) fetch --depth 1 origin main >/dev/null 2>&1 && \
		git -C $(ZED_SRC_DIR) reset --hard FETCH_HEAD >/dev/null 2>&1

$(ZED_IMPORTER): | zed-setup
	cargo build --release -p theme_importer \
		--manifest-path $(ZED_SRC_DIR)/Cargo.toml

$(ZED_BUNDLE): check-jq zed-setup $(ZED_IMPORTER) all | $(THEMESDIR) $(OUTDIR)
	@echo "Converting themes for Zed..."
	@for f in $(filter-out $(THEMESDIR)/PRINT.json $(ZED_BUNDLE),$(wildcard $(THEMESDIR)/*.json)); do \
		$(ZED_IMPORTER) $$f --output $(OUTDIR)/zed-$$(basename $$f); \
	done; \
	jq -s '{ "$$schema":"https://zed.dev/schema/themes/v0.2.0.json", \
		"name":"Oxocarbon", \
		"author":"Nyoom Engineering", \
		"themes":[ .[] | (.themes // .) | (if type=="array" then .[] else . end) ] }' \
		$(OUTDIR)/zed-*.json > $(ZED_BUNDLE)
	@echo "Zed theme bundle created: $(ZED_BUNDLE)"

clean:
	cargo clean; rm -f $(OUTDIR)/*.json $(THEMESDIR)/*.json

dotfiles:
	mkdir -p $(ASSETS)
	cursor --list-extensions > $(EXTENSIONS)
	cp $(CURSOR_CFG)/settings.json $(ASSETS)/settings.json
	cp $(CURSOR_CFG)/keybindings.json $(ASSETS)/keybindings.json

install: dotfiles
	xargs -I {} cursor --install-extension {} < $(EXTENSIONS)
	mkdir -p $(CURSOR_CFG)
	cp $(ASSETS)/settings.json $(CURSOR_CFG)/
	cp $(ASSETS)/keybindings.json $(CURSOR_CFG)/