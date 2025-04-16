#!/bin/bash

# Find all Rust files in the protorune crate
find crates/protorune/src -name "*.rs" -type f | while read -r file; do
    echo "Updating imports in $file"
    # Replace metashrew_core with metashrew
    sed -i '' 's/metashrew_core/metashrew/g' "$file"
done

echo "Import updates completed!"
