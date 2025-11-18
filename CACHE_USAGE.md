# Asset Caching Usage Guide

## Overview
Arnis now supports asset pre-caching, allowing you to separate the data download phase from the processing phase. This is useful for:

- Working offline after downloading data
- Processing the same area multiple times without re-downloading
- Debugging and development without network delays
- Avoiding API rate limits
- Handling intermittent network connections

## Cache Modes

### 1. Standard Mode (Default)
Normal operation - downloads and processes in one run:
```bash
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --path="/path/to/world"
```

### 2. Cache-Enabled Mode
Automatically uses cache if available, otherwise downloads and caches:
```bash
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --path="/path/to/world" --use-cache
```

### 3. Download-Only Mode
Download and cache data without processing:
```bash
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --download-only
```

This mode:
- Downloads OSM data from Overpass API
- Saves data to cache with validation
- Does NOT require `--path` (no Minecraft world needed)
- Prints cache location and size
- Exits after downloading

### 4. Process-Only Mode
Process from cache without downloading:
```bash
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --path="/path/to/world" --process-only
```

This mode:
- Loads data from cache (must exist)
- Does NOT access network
- Requires exact same bounding box as download
- Fails if cache doesn't exist

## Workflow Examples

### Example 1: Download Now, Process Later
```bash
# Step 1: Download data (e.g., on a machine with internet)
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --download-only

# Step 2: Process later (can be offline, or on different machine)
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --path="~/.minecraft/saves/NYC" --process-only
```

### Example 2: Multiple Processing Runs
```bash
# Download once
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --download-only

# Process with different settings
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --path="world1" --process-only --scale=1.0
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --path="world2" --process-only --scale=2.0 --terrain
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --path="world3" --process-only --scale=0.5
```

### Example 3: Development/Testing
```bash
# Download once
arnis --bbox="40.7128,-74.0060,40.7150,-74.0030" --download-only

# Test repeatedly without network delays
arnis --bbox="40.7128,-74.0060,40.7150,-74.0030" --path="test_world" --process-only --debug
# Make code changes...
arnis --bbox="40.7128,-74.0060,40.7150,-74.0030" --path="test_world" --process-only --debug
# Repeat...
```

## Cache Location

The cache is stored in OS-specific directories:

- **Linux**: `~/.cache/arnis/`
- **macOS**: `~/Library/Caches/arnis/`
- **Windows**: `%LOCALAPPDATA%\arnis\cache\`

## Cache Structure

Each cached area is stored in a subdirectory based on its bounding box:
```
~/.cache/arnis/
├── 40.712800_-74.006000_40.758900_-73.935000/
│   ├── osm_data.json          # Raw OSM data
│   └── metadata.json           # Cache metadata
└── 50.000000_-84.000000_51.000000_-83.000000/
    ├── osm_data.json
    └── metadata.json
```

## Cache Metadata

Each cache includes metadata:
```json
{
  "bbox": {
    "min": {"lat": 40.7128, "lng": -74.0060},
    "max": {"lat": 40.7589, "lng": -73.9350}
  },
  "timestamp": 1700000000,
  "osm_data_file": "osm_data.json",
  "elevation_data_file": null,
  "osm_checksum": "a1b2c3d4e5f6...",
  "osm_data_size": 12345678,
  "download_method": "requests"
}
```

## Cache Validation

The cache system includes integrity checking:
- Checksums verify data hasn't been corrupted
- Exact bounding box matching required
- Automatic validation on load
- Clear error messages if validation fails

## Cache Management

### Viewing Cache
Check what's cached:
```bash
# Cache location is printed by download-only mode
ls -lh ~/.cache/arnis/  # Linux/macOS
dir %LOCALAPPDATA%\arnis\cache  # Windows
```

### Clearing Cache
Remove cached data:
```bash
# Remove specific area
rm -rf ~/.cache/arnis/40.712800_-74.006000_40.758900_-73.935000/

# Remove all cache
rm -rf ~/.cache/arnis/
```

## Advantages

### Network Efficiency
- Download once, process many times
- Avoid API rate limits
- Work offline after initial download

### Development Workflow
- Faster iteration during development
- Consistent test data
- No dependency on external APIs

### Reliability
- Checksum validation prevents corruption
- Metadata tracking
- Clear error messages

## Limitations

### Bounding Box Must Match Exactly
The bounding box must be EXACTLY the same for cache hits:
```bash
# These will NOT share cache:
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --download-only
arnis --bbox="40.7128,-74.0060,40.7590,-73.9350" --process-only  # Different max_lat!
```

### Cache Size
Large areas create large cache files:
- Small area (1km²): ~1-5 MB
- Medium area (10km²): ~10-50 MB
- Large area (100km²): ~100-500 MB
- Dense city: 2-3x larger

Monitor disk space when caching many areas.

### No Automatic Expiration
Cached data doesn't expire automatically. OpenStreetMap data changes over time, so:
- Clear cache periodically for updated data
- Re-download if you need current data
- Cache is best for stable areas or development

## Error Messages

### "Cache not found for this bounding box"
**Cause**: No cached data exists for the specified bbox.
**Solution**: Run download-only mode first.

### "Cache integrity check failed - checksum mismatch"
**Cause**: Cached data was corrupted or modified.
**Solution**: Clear cache and re-download.

### "World path is required for processing mode"
**Cause**: Used process-only without --path.
**Solution**: Add --path argument.

## Future Enhancements

Planned features for the cache system:
- Elevation data caching
- Cache size limits
- Automatic expiration
- Cache sharing across networks
- Differential updates

## Examples with Other Features

### With Terrain
```bash
arnis --bbox="..." --download-only
arnis --bbox="..." --path="world" --process-only --terrain
```

### With Debug
```bash
arnis --bbox="..." --download-only --debug
arnis --bbox="..." --path="world" --process-only --debug
```

### With Scale
```bash
arnis --bbox="..." --download-only
arnis --bbox="..." --path="world" --process-only --scale=2.0
```

### With Different Downloader
```bash
# Try different download method
arnis --bbox="..." --download-only --downloader=curl
arnis --bbox="..." --path="world" --process-only
```

## Tips

1. **Start Small**: Test with small areas first
2. **Use Descriptive Worlds**: Name worlds based on area for easy tracking
3. **Monitor Cache Size**: Check ~/.cache/arnis/ periodically
4. **Document Coordinates**: Save bbox strings for areas you frequently use
5. **Version Control**: For development, cache test areas in your workflow

## Integration with Existing Features

The cache system works seamlessly with:
- `--file` / `--save-json-file`: Can still save to custom locations
- `--debug`: Debug output works in all cache modes
- `--downloader`: Choice of download method is cached in metadata
- `--scale`: Can process cached data at any scale
- `--terrain`: Terrain generation works with cached data
- All element processing options work with cached data

## Conclusion

Asset caching provides flexibility in how you use Arnis, separating concerns between data acquisition and processing. This is especially useful for development, testing, offline work, and avoiding unnecessary network usage.
