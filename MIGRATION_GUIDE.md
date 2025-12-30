# Migration Guide: Pre-Caching Feature

## Overview

This guide helps existing Arnis users understand and adopt the new pre-caching feature introduced in version 2.4.0+.

## What's New?

The pre-caching feature allows you to download and store OpenStreetMap data locally before generating Minecraft worlds. This is especially useful for:

- **Large area generation** - Avoid re-downloading if generation fails
- **Lower-end systems** - Split the workload into manageable phases
- **Offline work** - Download data once, generate multiple times without internet
- **Experimentation** - Try different settings without re-fetching data

## For Existing Users

### Nothing Changes (Unless You Want It To)

**Important**: The pre-caching feature is **completely optional**. Your existing workflows continue to work exactly as before:

```bash
# This still works exactly the same
arnis --bbox="40.7,-74.0,40.8,-73.9" \
      --path="$HOME/.minecraft/saves/MyWorld" \
      --terrain --interior --roof
```

### When to Consider Pre-Caching

Consider using pre-caching if you've experienced:

- ❌ **Generation crashes** on large areas after downloading data
- ❌ **Long wait times** for re-downloads after failed generations
- ❌ **Memory issues** during world generation
- ❌ **Network interruptions** during data download

## Quick Start

### 1. Pre-Cache Your First Region

Instead of generating immediately, cache the data first:

```bash
# Before (old way - still works):
arnis --bbox="40.7,-74.0,40.8,-73.9" --path="./MyWorld" --terrain

# Now (with pre-caching):
arnis --cache-only --bbox="40.7,-74.0,40.8,-73.9" --terrain
```

This downloads and saves the data without generating the world.

### 2. View Your Cached Regions

```bash
arnis --list-caches
```

Output:
```
Available cached regions:

  ID: cache_40700_-74000_40800_-73900_1234567890
  Name: Manhattan
  Bbox: 40.7,-74.0,40.8,-73.9
  Scale: 1.00
  Terrain: Yes
  Elements: 15234
  Size: 24.5 MB
  Created: 2025-01-15 14:30:00 UTC
```

### 3. Generate from Cache

```bash
arnis --from-cache cache_40700_-74000_40800_-73900_1234567890 \
      --path="$HOME/.minecraft/saves/MyWorld" \
      --interior --roof
```

## Migration Scenarios

### Scenario 1: First-Time Large Area

**Before (risk of data loss on crash)**:
```bash
arnis --bbox="48.8,2.2,48.9,2.4" --path="./Paris" --terrain
# If this crashes, you have to re-download everything
```

**After (safer approach)**:
```bash
# Step 1: Cache data
arnis --cache-only --bbox="48.8,2.2,48.9,2.4" --terrain

# Step 2: Generate from cache (can retry if it fails)
arnis --from-cache <cache_id> --path="./Paris"
```

### Scenario 2: Multiple Variations

**Before (re-download for each variation)**:
```bash
# Download data 3 times for 3 variations
arnis --bbox="..." --path="./WithInterior" --interior --roof
arnis --bbox="..." --path="./NoInterior" --roof
arnis --bbox="..." --path="./Minimal"
```

**After (download once, generate multiple)**:
```bash
# Cache once
arnis --cache-only --bbox="..." --terrain

# Generate variations from same cache
arnis --from-cache <id> --path="./WithInterior" --interior --roof
arnis --from-cache <id> --path="./NoInterior" --roof
arnis --from-cache <id> --path="./Minimal"
```

### Scenario 3: Batch Processing

**Before (sequential downloads and generations)**:
```bash
# Each region downloads and generates sequentially
for bbox in $BBOXES; do
    arnis --bbox="$bbox" --path="./World_$i" --terrain
done
```

**After (cache all first, then generate)**:
```bash
# Cache all regions first (more resilient)
for bbox in $BBOXES; do
    arnis --cache-only --bbox="$bbox" --terrain
done

# Generate at your convenience
arnis --list-caches
for cache_id in $CACHE_IDS; do
    arnis --from-cache "$cache_id" --path="./World_$i"
done
```

## Common Questions

### Q: Do I have to use pre-caching?

**A**: No! Pre-caching is optional. Your existing commands work exactly as before.

### Q: Where are caches stored?

**A**: Platform-specific locations:
- **Windows**: `%LOCALAPPDATA%\arnis\cache\`
- **macOS**: `~/Library/Application Support/arnis/cache/`
- **Linux**: `~/.local/share/arnis/cache/`

### Q: How much disk space do caches use?

**A**: It varies by area size:
- Small area (0.5 km²): ~5-8 MB
- Medium area (2 km²): ~30-50 MB
- Large area (10 km²): ~150-250 MB

Use `arnis --list-caches` to see total cache size.

### Q: Do caches expire?

**A**: No, caches persist until you delete them manually:
```bash
arnis --delete-cache <cache_id>    # Delete specific
arnis --clear-caches               # Delete all
```

### Q: Can I share caches with others?

**A**: Yes! Copy the cache directory and share it. Caches are platform-independent.

### Q: What if I update Arnis?

**A**: Caches remain compatible across versions (unless noted in release notes).

## Best Practices

### 1. Cache Before Large Generations

For any area >2 km², consider caching first:

```bash
arnis --cache-only --bbox="..." --terrain
# Wait for success
arnis --from-cache <id> --path="..."
```

### 2. Clean Up Old Caches

Periodically review and remove old caches:

```bash
arnis --list-caches
arnis --delete-cache <old_cache_id>
```

### 3. Use Descriptive Names

While cache names are auto-generated, you can identify them by location and date in the listing.

### 4. Monitor Disk Space

Check total cache size regularly:

```bash
arnis --list-caches  # Shows total cache size at bottom
```

## Troubleshooting

### Problem: Can't Find Cache ID

**Solution**: List all caches to see available IDs
```bash
arnis --list-caches
```

### Problem: Cache Corrupted

**Solution**: Delete and re-cache
```bash
arnis --delete-cache <corrupted_id>
arnis --cache-only --bbox="..." --terrain
```

### Problem: Out of Disk Space

**Solution**: Clear old caches
```bash
arnis --clear-caches
# Or delete specific ones
arnis --delete-cache <old_id>
```

## Examples

See the `examples/` directory for:
- `batch_precache.sh` - Linux/macOS batch caching script
- `batch_precache.bat` - Windows batch caching script
- `README.md` - Detailed examples and use cases

## Additional Resources

- **Feature Documentation**: [PRE_CACHING.md](PRE_CACHING.md)
- **Implementation Details**: [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)
- **Main Documentation**: [README.md](README.md)
- **GitHub Issues**: https://github.com/louis-e/arnis/issues
- **Discord Community**: https://discord.gg/mA2g69Fhxq

## Feedback

Found an issue or have suggestions? Please open an issue on GitHub:
https://github.com/louis-e/arnis/issues/new

---

**Welcome to the pre-caching era of Arnis!** 🎉

This feature makes large area generation more reliable and flexible. Try it out and let us know what you think!