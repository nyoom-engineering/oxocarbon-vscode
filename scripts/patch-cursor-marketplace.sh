#!/bin/bash

set -e

# Patches cursor to remove extension overrides & enable proprietary marketplace
# Necessary for Cursor to properly access and install proprietary extensions
# Use at your own risk!

# add to settings.json:
# "extensions.gallery.serviceUrl": "https://marketplace.visualstudio.com/_apis/public/gallery",
# "extensions.gallery.useUnpkgResourceApi": true,

echo "Patching Cursor marketplace configuration..."

cp /Applications/Cursor.app/Contents/Resources/app/product.json /Applications/Cursor.app/Contents/Resources/app/product.json.backup
jq '
.extensionsGallery = {
    "galleryId": "cursor",
    "serviceUrl": "https://marketplace.visualstudio.com/_apis/public/gallery",
    "itemUrl": "https://marketplace.visualstudio.com/items",
    "cacheUrl": "https://vscode.blob.core.windows.net/gallery/index",
    "resourceUrlTemplate": "https://{publisher}.vscode-unpkg.net/{publisher}/{name}/{version}/{path}",
    "extensionUrlTemplate": "https://www.vscode-unpkg.net/_gallery/{publisher}/{name}/latest"
}' /Applications/Cursor.app/Contents/Resources/app/product.json.backup > /Applications/Cursor.app/Contents/Resources/app/product.json

echo "Remove extension overrides as well? (Y/n)"
read -r response
response=${response:-Y}

if [[ "$response" =~ ^[Yy]$ ]]; then
    echo "Removing extension overrides..."
    jq "
        del(.extensionReplacementMapForImports) |
        del(.nodejsRepository) |
        del(.extensionMaxVersions) |
        del(.getExtensionOverrides) |
        del(.skipPackagingLocalExtensions) |
        del(.cannotImportExtensions)
    " /Applications/Cursor.app/Contents/Resources/app/product.json.backup > /Applications/Cursor.app/Contents/Resources/app/product.json
else
    echo "Skipping extension override removal..."
fi

echo "Codesigning Cursor app..."
codesign --force --deep --sign - /Applications/Cursor.app

echo "Cursor marketplace patch complete!"
