#!/bin/bash

# Create a temporary directory
mkdir -p temp_alkanes_support

# Clone the alkanes-rs repository
git clone https://github.com/kungfuflex/alkanes-rs.git temp_alkanes_support

# Copy the necessary files from the cloned repository to our local alkanes-support
cp -r temp_alkanes_support/crates/alkanes-support/* crates/alkanes-support/

# Clean up
rm -rf temp_alkanes_support

# Update the Cargo.toml file to use our local dependencies
sed -i '' 's/protorune-support = { git = ".*" }/protorune-support = { path = "..\/protorune-support" }/g' crates/alkanes-support/Cargo.toml
sed -i '' 's/metashrew = { git = ".*" }/metashrew = { path = "..\/metashrew" }/g' crates/alkanes-support/Cargo.toml
sed -i '' 's/metashrew-support = { git = ".*" }/metashrew-support = { path = "..\/metashrew-support" }/g' crates/alkanes-support/Cargo.toml

echo "Fixed alkanes-support dependencies"
