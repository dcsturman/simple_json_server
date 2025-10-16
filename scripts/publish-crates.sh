#!/bin/bash

# Script to publish workspace crates in correct dependency order
# Usage: ./scripts/publish-crates.sh [--dry-run]

set -e

DRY_RUN=""
if [[ "$1" == "--dry-run" ]]; then
    DRY_RUN="--dry-run"
    echo "🧪 Running in dry-run mode"
else
    echo "🚀 Publishing crates to crates.io"
fi

echo ""
echo "📦 Step 1: Publishing actor_attribute_macro..."
cd actor_attribute_macro
if [[ -n "$DRY_RUN" ]]; then
    cargo publish $DRY_RUN
    echo "✅ actor_attribute_macro dry-run successful"
else
    cargo publish
    echo "✅ actor_attribute_macro published"
    echo "⏳ Waiting 60 seconds for crates.io propagation..."
    sleep 60
fi

echo ""
echo "📦 Step 2: Publishing simple_json_server..."
cd ../simple_json_server
cargo publish $DRY_RUN
if [[ -n "$DRY_RUN" ]]; then
    echo "✅ simple_json_server dry-run successful"
else
    echo "✅ simple_json_server published"
fi

echo ""
if [[ -n "$DRY_RUN" ]]; then
    echo "🎉 All dry-runs completed successfully!"
    echo "💡 Note: simple_json_server dry-run will only work after actor_attribute_macro is published"
else
    echo "🎉 All crates published successfully!"
fi
