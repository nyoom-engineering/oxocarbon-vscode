#!/bin/bash

set -euo pipefail

# Patches cursor to remove extension overrides & enable proprietary marketplace
# Necessary for Cursor to properly access and install proprietary extensions
# Use at your own risk!

MARKETPLACE_URL="https://marketplace.visualstudio.com/_apis/public/gallery"
PRODUCT_JSON="/Applications/Cursor.app/Contents/Resources/app/product.json"
PRODUCT_BACKUP="${PRODUCT_JSON}.backup"
SETTINGS_FILE="$HOME/Library/Application Support/Cursor/User/settings.json"

apply_jq() {
  local file=$1
  local expr=$2
  shift 2
  local tmp
  tmp=$(mktemp)
  if jq --indent 2 "$@" "$expr" "$file" >"$tmp"; then
    mv "$tmp" "$file"
  else
    rm -f "$tmp"
    return 1
  fi
}

rewrite_with_jq() {
  local file=$1
  shift
  local tmp
  tmp=$(mktemp)
  trap 'rm -f "$tmp"' RETURN
  jq -r "$@" >"$tmp"
  mv "$tmp" "$file"
  trap - RETURN
}

ensure_settings() {
  mkdir -p "$(dirname "$SETTINGS_FILE")"
  if [[ ! -f "$SETTINGS_FILE" ]]; then
    printf '{}\n' >"$SETTINGS_FILE"
  fi

  rewrite_with_jq "$SETTINGS_FILE" -n \
    --rawfile data "$SETTINGS_FILE" \
    --arg url "$MARKETPLACE_URL" \
    --argjson use true \
    '
    def ensure($key; $value):
      ($value | tojson) as $json
      | ($key | gsub("\\."; "\\\\.")) as $escaped
      | if test("(?m)^\\s*\"" + $escaped + "\"\\s*:") then
          sub("(?m)^\\s*\"" + $escaped + "\"\\s*:\\s*[^,\\n]*(,?)"; "  \"" + $key + "\": " + $json + "\\1")
        elif test("(?s)^\\s*\\{\\s*\\}\\s*$") then
          "{\n  \"" + $key + "\": " + $json + "\n}"
        else
          (
            sub("(?m)(^\\s*\"[^\\n]*\"\\s*:\\s*[^,\\n]*)(?=\\s*\\n\\s*\\}\\s*$)"; "\\1,")
            | sub("\\}\\s*$"; "\n  \"" + $key + "\": " + $json + "\n}")
          )
        end;
    $data
    | ensure("extensions.gallery.serviceUrl"; $url)
    | ensure("extensions.gallery.useUnpkgResourceApi"; $use)
    | if endswith("\n") then . else . + "\n" end
    '
}

echo "Checking Cursor user settings..."
ensure_settings

echo "Patching Cursor marketplace configuration..."
cp "$PRODUCT_JSON" "$PRODUCT_BACKUP"
apply_jq "$PRODUCT_JSON" '
  .extensionsGallery = {
    "galleryId": "cursor",
    "serviceUrl": $url,
    "itemUrl": "https://marketplace.visualstudio.com/items",
    "cacheUrl": "https://vscode.blob.core.windows.net/gallery/index",
    "resourceUrlTemplate": "https://{publisher}.vscode-unpkg.net/{publisher}/{name}/{version}/{path}",
    "extensionUrlTemplate": "https://www.vscode-unpkg.net/_gallery/{publisher}/{name}/latest"
  }
' --arg url "$MARKETPLACE_URL"

printf "Remove extension overrides as well? (Y/n, default: Y) "
while true; do
  IFS= read -rsn1 response
  if [[ -z "$response" ]]; then
    response="Y"
    break
  fi
  case "$response" in
    [Yy]) response="Y"; break ;;
    [Nn]) response="n"; break ;;
    *)    printf "\r\033[KPlease enter Y or n (Enter defaults to Y): " ;;
  esac
done

# drain input
while IFS= read -rs -t 0 -n 1000 _; do :; done

if [[ "$response" == "Y" ]]; then
  printf "\r\033[K"
  echo "Removing extension overrides..."
  apply_jq "$PRODUCT_JSON" '
    del(
      .extensionReplacementMapForImports,
      .nodejsRepository,
      .extensionMaxVersions,
      .getExtensionOverrides,
      .skipPackagingLocalExtensions,
      .cannotImportExtensions
    )
  '
else
  printf "\r\033[K"
  echo "Skipping extension override removal..."
fi

echo "Codesigning Cursor app..."
codesign --force --deep --sign - /Applications/Cursor.app

echo "Done."
