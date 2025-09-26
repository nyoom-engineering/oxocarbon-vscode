#!/bin/bash

# Usage: ./vsc.sh <input.json>
# E.X. ./scripts/vsc.sh ./assets/Community-Material-Theme-Darker-High-Contrast.json

set -euo pipefail

# Check if input file is provided
if [ $# -ne 1 ]; then
    echo "Usage: $0 <input.json>"
    exit 1
fi

INPUT_FILE="$1"

# Check if input file exists and is a JSON file
if [ ! -f "$INPUT_FILE" ]; then
    echo "Error: Input file '$INPUT_FILE' not found"
    exit 1
fi

if [[ "$INPUT_FILE" != *.json ]]; then
    echo "Error: Input file must be a .json file"
    exit 1
fi

# Extract base name without extension
BASENAME=$(basename "$INPUT_FILE" .json)
OUTPUT_DIR="$BASENAME"

# Build the converters
echo "Building converters..."
cargo build --release -p json2tm
cargo build --release -p json2st
cargo build --release -p json2xccolor

# Create output directory structure
echo "Creating directory structure..."
mkdir -p "$OUTPUT_DIR/sublime"
mkdir -p "$OUTPUT_DIR/xcode"

# Convert to TextMate theme
echo "Converting to TextMate theme..."
target/release/json2tm "$INPUT_FILE" "$OUTPUT_DIR/sublime/$BASENAME.tmTheme"

# Convert to Sublime theme
echo "Converting to Sublime theme..."
cat "$INPUT_FILE" | target/release/json2st > "$OUTPUT_DIR/sublime/$BASENAME.sublime-theme"

# Convert to Xcode theme
echo "Converting to Xcode theme..."
target/release/json2xccolor "$INPUT_FILE" "$OUTPUT_DIR/xcode/$BASENAME.xccolortheme"

# Copy original JSON file
echo "Copying original JSON file..."
cp "$INPUT_FILE" "$OUTPUT_DIR/"

# Create zip archive
echo "Creating zip archive..."
zip -r "${OUTPUT_DIR}.zip" "$OUTPUT_DIR"

# Remove the original folder
echo "Removing temporary directory..."
rm -rf "$OUTPUT_DIR"

echo "Done! Created:"
echo "  - ${OUTPUT_DIR}.zip (archive)"
