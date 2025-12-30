# Pre-Caching Feature Documentation

## Overview

The pre-caching feature allows users to download and cache OpenStreetMap (OSM) data and elevation data for a region without immediately generating the Minecraft world. This is especially useful for:

- **Lower-end systems** that may crash during generation but can still download data
- **Large areas** where generation might fail, allowing you to cache first and generate later
- **Offline generation** by pre-downloading data when you have internet access
- **Batch processing** multiple regions by caching them all first, then generating worlds later

## How It Works

The pre-caching system separates the data acquisition phase from the world generation phase:

1. **Download Phase (Pre-Cache)**: Fetches OSM data and optionally elevation data from external APIs
2. **Storage Phase**: Saves the downloaded data to a local cache directory
3. **Generation Phase**: Loads cached data and generates the Minecraft world

This separation allows you to recover from crashes during generation without re-downloading data.

## Cache Storage

Cached regions are stored in a platform-specific directory:

- **Windows**: `%LOCALAPPDATA%\arnis\cache\`
- **macOS**: `~/Library/Application Support/arnis/cache/`
- **Linux**: `~/.local/share/arnis/cache/`

Each cache entry is stored in its own directory with the following structure:

```
cache_40123_-74456_40234_-74123_1234567890/
├── metadata.json          # Cache metadata (name, bbox, scale, etc.)
├── osm_data.json         # Raw OSM data from Overpass API
└── elevation_data.bin    # Binary elevation data (if terrain enabled)
```

## CLI Usage

### Pre-Cache Data Only

Download and cache data without generating a world:

```bash
arnis --cache-only --bbox="40.7,-74.0,40.8,-73.9" --scale=1.0 --terrain
```

### List Cached Regions

View all available cached regions:

```bash
arnis --list-caches
```

Output example:
```
Available cached regions:

  ID: cache_40700_-74000_40800_-73900_1234567890
  Name: New York City
  Bbox: 40.7,-74.0,40.8,-73.9
  Scale: 1.00
  Terrain: Yes
  Elements: 15234
  Size: 24.5 MB
  Created: 2025-01-15 14:30:00 UTC

  Total cache size: 124.3 MB
```

### Generate World from Cache

Generate a Minecraft world using cached data:

```bash
arnis --from-cache cache_40700_-74000_40800_-73900_1234567890 \
      --path="C:/Users/YourName/.minecraft/saves/NewWorld" \
      --interior --roof --fillground
```

### Delete a Cache

Remove a specific cached region:

```bash
arnis --delete-cache cache_40700_-74000_40800_-73900_1234567890
```

### Clear All Caches

Remove all cached regions:

```bash
arnis --clear-caches
```

### Custom Cache Directory

Use a custom location for caches:

```bash
arnis --cache-only --bbox="40.7,-74.0,40.8,-73.9" --cache-dir="/path/to/custom/cache"
```

## GUI Usage

### Pre-Cache Mode

1. Open Arnis GUI
2. Select your area on the map using the rectangle tool
3. Click **Settings** (gear icon)
4. Enable the **"Pre-Cache Only"** checkbox
5. Configure terrain and scale settings
6. Click **"Start Generation"**
7. Data will be cached without generating a world

### View Cached Regions

1. Click on the **"Cached Regions"** tab (if implemented)
2. View all available caches with details:
   - Region name and location
   - Creation date
   - Size and element count
   - Whether terrain data is included

### Generate from Cache

1. Select a cached region from the list
2. Choose your Minecraft world (or create new)
3. Configure generation settings (interior, roof, etc.)
4. Click **"Generate from Cache"**
5. World will be generated using the cached data

### Delete Cached Regions

1. Navigate to the **"Cached Regions"** tab
2. Select a cache entry
3. Click **"Delete"** to remove it
4. Or click **"Clear All"** to remove all caches

## Cache Metadata

Each cache entry includes metadata stored in `metadata.json`:

```json
{
  "id": "cache_40700_-74000_40800_-73900_1234567890",
  "name": "New York City",
  "bbox": "40.7,-74.0,40.8,-73.9",
  "scale": 1.0,
  "has_terrain": true,
  "created_at": "2025-01-15T14:30:00.000Z",
  "size_bytes": 25690112,
  "element_count": 15234
}
```

## Performance Considerations

### When to Use Pre-Caching

**✅ Use pre-caching when:**
- Generating very large areas (>1km²)
- Using a lower-end PC with limited RAM
- You've experienced crashes during generation
- You want to generate multiple worlds from the same data
- You want to work offline after downloading data

**❌ Don't need pre-caching when:**
- Generating small areas (<0.5km²)
- You have plenty of RAM and a powerful PC
- You want the fastest single generation

### Cache Size Estimates

Approximate cache sizes for different area sizes:

| Area Size | OSM Elements | Cache Size (No Terrain) | Cache Size (With Terrain) |
|-----------|--------------|-------------------------|---------------------------|
| 0.5 km²   | ~5,000      | ~5 MB                   | ~8 MB                     |
| 1 km²     | ~15,000     | ~15 MB                  | ~25 MB                    |
| 2 km²     | ~30,000     | ~30 MB                  | ~50 MB                    |
| 5 km²     | ~75,000     | ~75 MB                  | ~125 MB                   |
| 10 km²    | ~150,000    | ~150 MB                 | ~250 MB                   |

*Note: Actual sizes vary based on data density (urban vs rural areas)*

## Troubleshooting

### Cache Not Found

**Error**: `Cache entry 'cache_id' not found`

**Solutions**:
- Use `--list-caches` to see available cache IDs
- Check that you're using the correct cache directory
- Verify the cache wasn't accidentally deleted

### Cache Corrupted

**Error**: `Failed to parse metadata` or `Failed to deserialize elevation data`

**Solutions**:
- Delete the corrupted cache: `--delete-cache <cache_id>`
- Re-cache the region: `--cache-only --bbox=...`

### Out of Disk Space

**Error**: `Failed to write OSM data: No space left on device`

**Solutions**:
- Clear old caches: `--clear-caches`
- Use a custom cache directory on a different drive: `--cache-dir=/path/to/drive`
- Free up disk space before caching

### Cache Location Unknown

To find your cache directory:

```bash
# List caches - the output shows the cache location
arnis --list-caches
```

Or manually navigate to the platform-specific directory listed above.

## Advanced Usage

### Automating Batch Pre-Caching

Create a script to pre-cache multiple regions:

```bash
#!/bin/bash

# Array of bounding boxes to cache
declare -a bboxes=(
  "40.7,-74.0,40.8,-73.9"
  "51.5,-0.2,51.6,-0.1"
  "48.8,2.2,48.9,2.4"
)

# Cache each region
for bbox in "${bboxes[@]}"
do
  echo "Caching: $bbox"
  arnis --cache-only --bbox="$bbox" --scale=1.0 --terrain
  sleep 5  # Avoid overwhelming the API
done

echo "All regions cached!"
```

### Generating Multiple Worlds from One Cache

Use the same cache to generate different variations:

```bash
# Cache once
arnis --cache-only --bbox="40.7,-74.0,40.8,-73.9" --scale=1.0 --terrain

# Get cache ID
CACHE_ID=$(arnis --list-caches | grep "ID:" | head -n1 | awk '{print $2}')

# Generate world with interiors
arnis --from-cache $CACHE_ID --path="./world_with_interiors" --interior --roof

# Generate world without interiors
arnis --from-cache $CACHE_ID --path="./world_no_interiors" --roof

# Generate terrain-only world
arnis --from-cache $CACHE_ID --path="./world_terrain_only" --terrain
```

### Sharing Caches

You can share cache directories with others:

1. Locate your cache directory (see above)
2. Compress the specific cache folder: `cache_<id>/`
3. Share the compressed file
4. Recipients extract to their cache directory
5. Recipients run: `arnis --from-cache <cache_id> --path=...`

**Note**: Caches are platform-independent and can be shared across Windows, macOS, and Linux.

## API Usage

For developers integrating the cache system:

```rust
use arnis::cache_manager::CacheManager;

// Create cache manager
let cache_manager = CacheManager::new()?;

// Save cache
let cache_id = cache_manager.save_cache(
    &bbox,
    scale,
    &osm_data,
    elevation_data.as_ref(),
    Some("Custom Name".to_string())
)?;

// List caches
let caches = cache_manager.list_caches()?;

// Load cache
let cache_entry = cache_manager.load_cache(&cache_id)?;

// Delete cache
cache_manager.delete_cache(&cache_id)?;
```

## Future Enhancements

Planned improvements for the pre-caching system:

- [ ] Automatic cache expiration (remove old caches)
- [ ] Cache compression to reduce disk usage
- [ ] Network cache sharing (download caches from community)
- [ ] Incremental caching (update existing cache with new data)
- [ ] Cache validation and repair tools
- [ ] Preview image generation during caching
- [ ] Progress indicators for large cache operations

## Feedback and Support

If you encounter issues with the pre-caching feature:

1. Check this documentation first
2. Search existing GitHub issues: https://github.com/louis-e/arnis/issues
3. Create a new issue with:
   - Your OS and Arnis version
   - Cache operation you were attempting
   - Error messages (if any)
   - Cache directory contents (if relevant)

## License

This feature is part of Arnis and is licensed under Apache-2.0.
See [LICENSE](LICENSE) for full details.

---

**Last Updated**: January 2025  
**Feature Version**: 1.0  
**Arnis Version**: 2.4.0+