# Makefile to build toml2json and generate Oxocarbon-color-theme.json

CARGO      := cargo
PROG       := target/release/toml2json
INPUT      := oxocarbon.toml
OUTDIR     := themes
OUTFILE    := $(OUTDIR)/oxocarbon-color-theme.json

.PHONY: all build clean

all: build $(OUTFILE)

build:
	$(CARGO) build --release

$(OUTFILE): build $(INPUT)
	mkdir -p $(OUTDIR)
	$(PROG) $(INPUT) > $(OUTFILE)

clean:
	$(CARGO) clean
	rm -f $(OUTFILE)