#!/bin/bash

# Port themes from Qt6 to Qt5 (Master Version)
echo "🔧 Creating legacy themes in 'themes-qt5'..."

# Reset/Create the target directory
rm -rf themes-qt5
cp -r themes themes-qt5

find themes-qt5 -name "*.qml" -type f | while read -r file; do
    echo "Processing $file..."

    # 1. Add missing Screen import if needed (Qt5 requirement)
    if grep -q "Screen\." "$file" && ! grep -q "import QtQuick.Window" "$file"; then
        # Insert after the first import line
        sed -i '1aimport QtQuick.Window 2.12' "$file"
    fi

    # 2. Safer Versioned Imports
    sed -i 's/import QtQuick[[:space:]]*$/import QtQuick 2.12/g' "$file"
    sed -i 's/import QtQuick.Window[[:space:]]*$/import QtQuick.Window 2.12/g' "$file"
    sed -i 's/import QtQuick.Shapes[[:space:]]*$/import QtQuick.Shapes 1.12/g' "$file"
    sed -i 's/import QtMultimedia[[:space:]]*$/import QtMultimedia 5.12/g' "$file"
    sed -i 's/import QtQuick.Controls[[:space:]]*$/import QtQuick.Controls 2.12/g' "$file"
    sed -i 's/import Qt.labs.folderlistmodel[[:space:]]*$/import Qt.labs.folderlistmodel 2.1/g' "$file"
    sed -i 's/import QtQuick.Layouts[[:space:]]*$/import QtQuick.Layouts 1.12/g' "$file"
    
    # Graphical Effects (Qt6 uses Qt5Compat.GraphicalEffects)
    sed -i 's/import Qt5Compat.GraphicalEffects/import QtGraphicalEffects 1.12/g' "$file"

    # 3. Fix Signal Handler Syntax (The "function onSignal" -> "onSignal:" fix)
    perl -i -pe 's/function\s+on([A-Z][a-zA-Z0-9_]*)\s*\((.*?)\)\s*\{/on$1: {/g' "$file"

    # 4. Multimedia API Fix & Semicolon Cleanup
    # This removes 'videoOutput: id' from MediaPlayer and adds 'source: mp_id' to VideoOutput
    perl -0777 -i -pe '
        if (m/MediaPlayer\s*\{.*?id:\s*(\w+).*?videoOutput:\s*(\w+)/s) {
            my ($mp_id, $vo_id) = ($1, $2);
            s/videoOutput:\s*$vo_id\s*;?//g; 
            s/(VideoOutput\s*\{.*?id:\s*$vo_id)/$1\n        source: $mp_id/sg;
        }
        s/;\s*;/;/g; 
    ' "$file"

done

# 5. Update metadata.desktop to use Qt5
echo "Updating metadata.desktop files..."
find themes-qt5 -name "metadata.desktop" -type f | while read -r file; do
    sed -i 's/QtVersion=6/QtVersion=5/g' "$file"
done

echo "✅ Done! All themes in 'themes-qt5' updated."
echo "🚀 You can now run ./sddm.sh to set your theme!"
