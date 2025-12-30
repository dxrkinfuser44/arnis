#!/bin/bash

# Batch Pre-Caching Script for Arnis
# This script demonstrates how to pre-cache multiple regions for offline generation

set -e  # Exit on error

echo "==================================="
echo "Arnis Batch Pre-Caching Script"
echo "==================================="
echo ""

# Configuration
ARNIS_CMD="arnis"  # Change to ./arnis or full path if needed
SCALE=1.0
TERRAIN_FLAG="--terrain"

# Array of regions to cache (format: "name|bbox")
# bbox format: min_lat,min_lng,max_lat,max_lng
declare -a REGIONS=(
    "Downtown Manhattan|40.7000,-74.0200,40.7500,-73.9700"
    "Central Park|40.7644,-73.9831,40.8006,-73.9489"
    "Times Square Area|40.7540,-73.9900,40.7600,-73.9800"
)

# Function to cache a single region
cache_region() {
    local name="$1"
    local bbox="$2"

    echo "----------------------------------------"
    echo "Caching: $name"
    echo "Bounding Box: $bbox"
    echo "----------------------------------------"

    if $ARNIS_CMD --cache-only --bbox="$bbox" --scale=$SCALE $TERRAIN_FLAG; then
        echo "✓ Successfully cached: $name"
    else
        echo "✗ Failed to cache: $name"
        return 1
    fi

    echo ""
}

# Main execution
echo "Starting batch pre-caching for ${#REGIONS[@]} regions..."
echo ""

CACHED_COUNT=0
FAILED_COUNT=0

for region in "${REGIONS[@]}"; do
    # Split region string into name and bbox
    IFS='|' read -r name bbox <<< "$region"

    if cache_region "$name" "$bbox"; then
        ((CACHED_COUNT++))
    else
        ((FAILED_COUNT++))
    fi

    # Small delay to avoid overwhelming the Overpass API
    echo "Waiting 5 seconds before next request..."
    sleep 5
done

echo "==================================="
echo "Batch Pre-Caching Complete!"
echo "==================================="
echo "Successfully cached: $CACHED_COUNT regions"
echo "Failed: $FAILED_COUNT regions"
echo ""

# List all cached regions
echo "Listing all cached regions:"
echo ""
$ARNIS_CMD --list-caches

echo ""
echo "To generate a world from cache, use:"
echo "  arnis --from-cache <cache_id> --path=/path/to/minecraft/world"
echo ""
