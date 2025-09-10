PROG := target/release/oxocarbon-themec
INPUT := oxocarbon.toml
OUTDIR := out
THEMESDIR := themes
ASSETS := assets

CURSOR_CFG := ~/Library/Application\ Support/Cursor/User
ZED_CFG := ~/.config/zed
EXTENSIONS := $(ASSETS)/extensions.txt

ZED_REPO_URL := https://github.com/zed-industries/zed.git
ZED_SRC_DIR := target/zed
ZEDDIR := zed
ZED_BUNDLE := $(ZEDDIR)/oxocarbon.json
ZED_IMPORTER := $(ZED_SRC_DIR)/target/release/theme_importer

TMDIR := textmate
TM_CONVERTER := json2tm/target/release/json2tm
TM_USER := ~/Library/Application\ Support/TextMate/Themes
SUBLIME_USER := ~/Library/Application\ Support/Sublime\ Text/Packages/User

INTELLIJDIR := intellij
JB_REPO_URL := https://github.com/JetBrains/colorSchemeTool
JB_SRC_DIR := target/colorSchemeTool

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

TEXTMATE_THEMES := $(patsubst $(THEMESDIR)/%.json,$(TMDIR)/%.tmTheme,$(DEFAULT_THEMES))

.PHONY: all build clean dotfiles help install mono-coolgray mono-warmgray \
	PRINT zed setup-zed dotfiles-zed dotfiles-sublime install-zed install-sublime install-textmate textmate intellij

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

setup-zed: check-xcode $(ZED_SRC_DIR)

$(ZED_SRC_DIR): | check-xcode
	@[ -d "$(ZED_SRC_DIR)/.git" ] || \
		git clone --depth 1 $(ZED_REPO_URL) $(ZED_SRC_DIR); \
		git -C $(ZED_SRC_DIR) fetch --depth 1 origin main >/dev/null 2>&1 && \
		git -C $(ZED_SRC_DIR) reset --hard FETCH_HEAD >/dev/null 2>&1

$(ZED_IMPORTER): | setup-zed
	cargo build --release -p theme_importer \
		--manifest-path $(ZED_SRC_DIR)/Cargo.toml

$(ZED_BUNDLE): check-jq setup-zed $(ZED_IMPORTER) all | $(THEMESDIR) $(OUTDIR)
	@mkdir -p $(dir $(ZED_BUNDLE))
	@echo "Converting themes for Zed..."
	@for f in $(filter-out $(THEMESDIR)/PRINT.json $(ZED_BUNDLE),$(wildcard $(THEMESDIR)/*.json)); do \
		$(ZED_IMPORTER) $$f --output $(OUTDIR)/zed-$$(basename $$f); \
	done; \
	jq -s 'def set_accent_and_players: \
		(.name | ascii_downcase | contains("monochrom")) as $$mono \
		| (.name | ascii_downcase | contains("compatibility")) as $$compat \
		| .style["text.accent"] = (if $$mono then "#ffffff" else "#ff7eb6" end) \
		| .style["text.muted"] = (if $$compat then "#8d8d8d" else "#f2f4f8" end) \
		| .style["panel.focused_border"] = "#6f6f6f" \
		| .style["editor.document_highlight.bracket_background"] = "#393939" \
		| .style["panel.focused_border"] = "#6f6f6f" \
		| .style.syntax.function.font_weight = 700 \
		| .style.syntax.constructor.font_weight = 600 \
		| .style.syntax.emphasis.font_weight = 500 \
		| .style.syntax["emphasis.strong"].font_weight = 700 \
		| .style.syntax.link_text = { "color": "#be95ff", "font_style": null, "font_weight": null } \
		| .style.syntax.selector = { "color": "#f2f4f8", "font_style": null, "font_weight": null } \
		| .style.syntax["selector.pseudo"] = { "color": "#dde1e6", "font_style": null, "font_weight": null } \
		| .style.syntax.namespace = { "color": "#ffffff", "font_style": null, "font_weight": null } \
		| .style.syntax["function.builtin"] = { "color": (if $$mono then "#ffffff" else "#ff7eb6" end), "font_style": null, "font_weight": 500 } \
		| .style.players = [ { "cursor":"#ffffffff", "background":"#ffffffff", "selection":"#52525290" } ]; \
		{ "$$schema":"https://zed.dev/schema/themes/v0.2.0.json", \
		  "name":"Oxocarbon", \
		  "author":"Nyoom Engineering", \
		  "themes":[ .[] | (.themes // .) | (if type=="array" then .[] else . end) | set_accent_and_players ] }' \
		$(OUTDIR)/zed-*.json > $(ZED_BUNDLE)
	@echo "Zed theme bundle created: $(ZED_BUNDLE)"

textmate: $(TEXTMATE_THEMES)

$(TM_CONVERTER):
	cargo build --release --manifest-path json2tm/Cargo.toml

$(TMDIR)/%.tmTheme: $(THEMESDIR)/%.json $(TM_CONVERTER)
	@mkdir -p $(dir $@)
	$(TM_CONVERTER) $< $@

intellij: textmate
	@echo "Converting TextMate themes to IntelliJ..."
	@set -e; \
	PY=$$(command -v python2.7 >/dev/null 2>&1 && echo python2.7 || (command -v python3 >/dev/null 2>&1 && echo python3 || (command -v python >/dev/null 2>&1 && echo python || echo ""))); \
	if [ -z "$$PY" ]; then echo "Error: python3 or python required"; exit 1; fi; \
	@[ -d "$(JB_SRC_DIR)/.git" ] || git clone --depth 1 $(JB_REPO_URL) $(JB_SRC_DIR); \
	git -C $(JB_SRC_DIR) fetch --depth 1 origin master >/dev/null 2>&1 || true; \
	git -C $(JB_SRC_DIR) reset --hard FETCH_HEAD >/dev/null 2>&1 || true; \
	mkdir -p $(INTELLIJDIR); \
	for f in $(wildcard $(TMDIR)/*.tmTheme); do \
		name=$$(basename $$f .tmTheme); \
		( cd $(JB_SRC_DIR) && $$PY colorSchemeTool.py ../..//$$f ../..//$(INTELLIJDIR)/$$name.icls ); \
	done; \
	echo "IntelliJ schemes written to $(INTELLIJDIR)"

clean:
	cargo clean; rm -f $(OUTDIR)/*.json $(THEMESDIR)/*.json $(ZEDDIR)/*.json $(TMDIR)/*.tmTheme $(INTELLIJDIR)/*.icls

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

dotfiles-zed:
	mkdir -p $(ASSETS)
	cp $(ZED_CFG)/settings.json $(ASSETS)/settings-zed.json

dotfiles-sublime:
	mkdir -p $(ASSETS)
	cp $(SUBLIME_USER)/Preferences.sublime-settings $(ASSETS)/Preferences.sublime-settings

install-zed:
	mkdir -p $(ZED_CFG)/themes
	if [ -f $(ZED_BUNDLE) ]; then cp -f $(ZED_BUNDLE) $(ZED_CFG)/themes/oxocarbon.json; fi
	if [ -f $(ASSETS)/settings-zed.json ]; then mv -f $(ASSETS)/settings-zed.json $(ZED_CFG)/settings.json; fi

install-sublime:
	mkdir -p $(SUBLIME_USER)
	cp $(TMDIR)/*.tmTheme $(SUBLIME_USER)/
	cp $(ASSETS)/Preferences.sublime-settings $(SUBLIME_USER)/Preferences.sublime-settings

install-textmate:
	mkdir -p $(TM_USER)
	cp $(TMDIR)/*.tmTheme $(TM_USER)/