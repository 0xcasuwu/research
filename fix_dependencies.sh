#!/bin/bash

echo "Starting dependency conflict resolution..."

# 1. Update imports in all Rust files across the codebase
echo "Updating imports in all Rust files..."

# Replace metashrew_core with metashrew and fix other import issues
find . -name "*.rs" -type f | while read -r file; do
    echo "Processing $file"
    
    # Basic replacements
    sed -i '' 's/metashrew_core/metashrew/g' "$file"
    
    # Fix KeyValuePointer trait imports - context-aware replacement
    if grep -q "metashrew_support::index_pointer::KeyValuePointer" "$file"; then
        if grep -q "use crate::index_pointer" "$file"; then
            # If file already has crate::index_pointer imports, use that
            sed -i '' 's/use metashrew_support::index_pointer::KeyValuePointer;/use crate::index_pointer::KeyValuePointer;/g' "$file"
        else
            # Otherwise use the canonical metashrew-support path
            sed -i '' 's/use metashrew_support::index_pointer::KeyValuePointer;/use metashrew_support::index_pointer::KeyValuePointer;/g' "$file"
        fi
    fi
    
    # Fix RuneTransfer and other type conflicts
    sed -i '' 's/protorune_support::rune_transfer::RuneTransfer/crate::rune_transfer::RuneTransfer/g' "$file"
    sed -i '' 's/protorune_support::balance_sheet::BalanceSheet/crate::balance_sheet::BalanceSheet/g' "$file"
    
    # Fix method calls on AtomicPointer
    sed -i '' 's/atomic\.keyword/atomic.derive/g' "$file"
    
    # Fix imports for specific files in alkanes-rs
    if [[ "$file" == *"alkanes-rs"* ]] || [[ "$file" == *"alkanes/"* ]]; then
        sed -i '' 's/use metashrew::index_pointer::/use metashrew_support::index_pointer::/g' "$file"
        sed -i '' 's/metashrew::index_pointer::AtomicPointer/metashrew_support::index_pointer::AtomicPointer/g' "$file"
        sed -i '' 's/metashrew::index_pointer::IndexPointer/metashrew_support::index_pointer::IndexPointer/g' "$file"
    fi
done

# 2. Update the patch section in the root Cargo.toml
echo "Updating patch section in root Cargo.toml..."

# Check if we need to add more patches
if ! grep -q "metashrew-6d93e41f08ea6ecf" Cargo.toml; then
    # Add patch for metashrew
    sed -i '' '/\[patch."https:\/\/github.com\/kungfuflex\/alkanes-rs"\]/a\
metashrew = { path = "crates/metashrew" }\
metashrew-support = { path = "crates/metashrew-support" }\
protorune-support = { path = "crates/protorune-support" }' Cargo.toml
fi

# 3. Handle the alkanes-rs checkout
ALKANES_PATH="/Users/erickdelgado/.cargo/git/checkouts/alkanes-rs-6aa9d88f67afd990/1bd1475"
if [ -d "$ALKANES_PATH" ]; then
    echo "Handling alkanes-rs checkout..."
    
    # Backup the original directory
    if [ ! -d "${ALKANES_PATH}.bak" ]; then
        echo "Creating backup of alkanes-rs checkout..."
        cp -r "$ALKANES_PATH" "${ALKANES_PATH}.bak"
    fi
    
    # Update Cargo.toml in the alkanes-rs checkout to use our local crates
    if [ -f "${ALKANES_PATH}/Cargo.toml" ]; then
        echo "Updating alkanes-rs Cargo.toml..."
        
        # Add patch section if it doesn't exist
        if ! grep -q "\[patch.crates-io\]" "${ALKANES_PATH}/Cargo.toml"; then
            echo -e "\n[patch.crates-io]\nmetashrew = { path = \"$(pwd)/crates/metashrew\" }\nmetashrew-support = { path = \"$(pwd)/crates/metashrew-support\" }\nprotorune-support = { path = \"$(pwd)/crates/protorune-support\" }\n" >> "${ALKANES_PATH}/Cargo.toml"
        fi
    fi
    
    # Fix specific files with the most errors
    for file in "${ALKANES_PATH}/src/view.rs" "${ALKANES_PATH}/src/vm/host_functions.rs" "${ALKANES_PATH}/src/vm/utils.rs" "${ALKANES_PATH}/src/lib.rs"; do
        if [ -f "$file" ]; then
            echo "Fixing imports in $file..."
            
            # Replace problematic imports
            sed -i '' 's/use metashrew_support::index_pointer::KeyValuePointer;/use metashrew_support::index_pointer::KeyValuePointer;/g' "$file"
            sed -i '' 's/metashrew::index_pointer::AtomicPointer/metashrew_support::index_pointer::AtomicPointer/g' "$file"
            sed -i '' 's/metashrew::index_pointer::IndexPointer/metashrew_support::index_pointer::IndexPointer/g' "$file"
            
            # Fix method calls
            sed -i '' 's/\.keyword(/\.derive(/g' "$file"
            
            # Fix RuneTransfer and BalanceSheet
            sed -i '' 's/protorune_support::rune_transfer::RuneTransfer/crate::rune_transfer::RuneTransfer/g' "$file"
            sed -i '' 's/protorune_support::balance_sheet::BalanceSheet/crate::balance_sheet::BalanceSheet/g' "$file"
        fi
    done
fi

# 4. Update the patch section in the root Cargo.toml to be more comprehensive
echo "Updating patch section in root Cargo.toml..."

# Create a new Cargo.toml with the correct structure
cat > Cargo.toml.new << EOF
[workspace]
resolver = "2"
members = [
    "contracts/bonding-contract",
    "crates/alkanes-build",
    "crates/alkanes-common",
    "crates/alkanes-macros",
    "crates/alkanes-runtime",
    "crates/alkanes-std-auth-token",
    "crates/alkanes-std-collection",
    "crates/alkanes-std-factory-support",
    "crates/alkanes-std-genesis-alkane",
    "crates/alkanes-std-genesis-protorune",
    "crates/alkanes-std-merkle-distributor",
    "crates/alkanes-std-mintable",
    "crates/alkanes-std-orbital",
    "crates/alkanes-std-owned-token",
    "crates/alkanes-std-proxy",
    "crates/alkanes-std-test",
    "crates/alkanes-std-upgradeable",
    "crates/alkanes-support",
    "crates/metashrew",
    "crates/metashrew-support",
    "crates/ordinals",
    "crates/protorune",
    "crates/protorune-support"
]

[workspace.dependencies]
alkanes-runtime = { path = "crates/alkanes-runtime" }
alkanes-support = { path = "crates/alkanes-support" }
alkanes-macros = { path = "crates/alkanes-macros" }
alkanes-common = { path = "crates/alkanes-common" }
metashrew = { path = "crates/metashrew", features = ["test-utils"] }
metashrew-support = { path = "crates/metashrew-support" }
protorune-support = { path = "crates/protorune-support" }
anyhow = { version = "1.0", features = ["backtrace"] }
byteorder = "1.5"
hex = "0.4.3"
ruint = "1.11.1"
wasm-bindgen = "0.2.89"
wasm-bindgen-test = "0.3.39"
serde_json = "1.0.114"
protobuf = "3.3.0"

[patch."https://github.com/kungfuflex/alkanes-rs"]
protorune = { path = "crates/protorune", features = ["test-utils"] }
metashrew = { path = "crates/metashrew" }
metashrew-support = { path = "crates/metashrew-support" }
protorune-support = { path = "crates/protorune-support" }

[patch."https://github.com/kungfuflex/metashrew"]
metashrew = { path = "crates/metashrew" }
metashrew-support = { path = "crates/metashrew-support" }

[patch."https://github.com/kungfuflex/protorune-rs"]
protorune = { path = "crates/protorune" }
protorune-support = { path = "crates/protorune-support" }
EOF

# Replace the old Cargo.toml with the new one
mv Cargo.toml.new Cargo.toml

# 5. Clean and rebuild
echo "Cleaning cargo cache and rebuilding..."
rm -rf target/
cargo clean
cargo update

echo "Dependency conflict resolution completed!"
echo "You may need to run 'cargo build' to verify the changes."
