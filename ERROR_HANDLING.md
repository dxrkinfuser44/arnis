# Error Handling Guide for Arnis

## Overview
This document describes the error handling patterns used in Arnis, common errors users may encounter, and troubleshooting steps.

## Error Categories

### 1. Network Errors

#### Error: Request Timeout
**Message**: `Error! Request timed out. Try selecting a smaller area.`

**Cause**: The Overpass API query took longer than 360 seconds.

**Solutions**:
1. Reduce the bounding box size
2. Try a less densely populated area
3. Use `--save-json-file` to cache successful downloads
4. Check your internet connection
5. Try a different Overpass API endpoint

**Prevention**:
```bash
# Save data for reuse
arnis --bbox="..." --path="..." --save-json-file data.json

# Reuse saved data
arnis --file data.json --path="..."
```

#### Error: Invalid Server Response
**Message**: `Error! Received response code: [status]`

**Cause**: The Overpass API returned a non-200 status code.

**Solutions**:
1. Check if the API is down: https://overpass-api.de/
2. Verify your bounding box coordinates are valid
3. Try the curl or wget download method:
   ```bash
   arnis --bbox="..." --path="..." --downloader curl
   ```
4. Wait and retry (API might be rate-limited)

#### Error: Empty Data
**Message**: `Error! Received invalid from server`

**Cause**: The server returned an empty response.

**Solutions**:
1. Verify the bounding box contains geographic data
2. Check OSM data coverage for your area
3. Try a different area or larger bounding box

### 2. File System Errors

#### Error: Path Does Not Exist
**Message**: `Path does not exist: [path]`

**Cause**: The specified Minecraft world path doesn't exist.

**Solutions**:
1. Verify the path is correct
2. Ensure the Minecraft world exists
3. Create a new world in Minecraft first
4. Use absolute paths to avoid confusion

**Example**:
```bash
# Windows
arnis --bbox="..." --path="C:\Users\YourName\AppData\Roaming\.minecraft\saves\YourWorld"

# macOS
arnis --bbox="..." --path="/Users/YourName/Library/Application Support/minecraft/saves/YourWorld"

# Linux
arnis --bbox="..." --path="/home/yourname/.minecraft/saves/YourWorld"
```

#### Error: Path Is Not a Directory
**Message**: `Path is not a directory: [path]`

**Cause**: The path points to a file instead of a directory.

**Solutions**:
1. Ensure you're pointing to the world folder, not a file
2. Remove any filename from the path
3. Check for typos in the path

#### Error: Region Folder Does Not Exist
**Message**: `region/ folder does not exist in the specified world path`

**Cause**: The Minecraft world is missing the region folder.

**Solutions**:
1. Ensure you're using a valid Minecraft Java Edition world
2. Create a new world in Minecraft if needed
3. Load the world in Minecraft at least once to initialize it

#### Error: Permission Denied
**Cause**: Insufficient permissions to read/write world files.

**Solutions**:
1. Close Minecraft (files may be locked)
2. Run with appropriate permissions
3. Check file/folder permissions
4. On Windows, run as administrator if needed
5. On Linux/macOS, check ownership with `ls -la`

### 3. Data Validation Errors

#### Error: Invalid Bounding Box
**Message**: Various coordinate-related errors

**Cause**: Bounding box coordinates are invalid or in wrong format.

**Format**: `--bbox="min_lat,min_lng,max_lat,max_lng"`

**Requirements**:
- min_lat < max_lat
- min_lng < max_lng
- Latitude: -90 to 90
- Longitude: -180 to 180

**Example**:
```bash
# Correct
arnis --bbox="40.7128,-74.0060,40.7589,-73.9350" --path="..."

# Incorrect (reversed coordinates)
arnis --bbox="40.7589,-73.9350,40.7128,-74.0060" --path="..."
```

#### Error: Failed to Parse JSON
**Cause**: Invalid or corrupted JSON data file.

**Solutions**:
1. Re-download the data using `--save-json-file`
2. Validate JSON syntax using an online validator
3. Check file encoding (should be UTF-8)
4. Ensure the file isn't truncated

### 4. Memory Errors

#### Error: Out of Memory
**Symptoms**:
- Process crashes without error message
- System becomes unresponsive
- "killed" message on Linux

**Solutions**:
1. Reduce bounding box size
2. Close other applications
3. Use a machine with more RAM
4. Process in smaller chunks
5. Disable debug mode (reduces memory usage)

**Memory Estimation**:
- Small area (1km²): ~500MB RAM
- Medium area (10km²): ~2-4GB RAM
- Large area (100km²): ~8-16GB RAM
- Dense city: 2-3x more RAM

**Optimization**:
```bash
# Process without debug output
arnis --bbox="..." --path="..."

# Future: Split into smaller regions
# (distributed processing feature)
```

### 5. Build and Compilation Errors

#### Error: No Targets Specified
**Message**: `no targets specified in the manifest`

**Cause**: Building from root Cargo.toml which is a workspace.

**Solutions**:
```bash
# Build CLI from workspace member
cd crates/arnis-cli
cargo build --release

# Or use workspace commands
cargo build --release -p arnis

# Build GUI
cargo build --release --features gui
```

#### Error: Missing Dependencies
**Cause**: Platform-specific dependencies not installed.

**Solutions**:

**Linux**:
```bash
# Ubuntu/Debian
sudo apt-get install libwebkit2gtk-4.0-dev libgtk-3-dev libayatana-appindicator3-dev

# Fedora
sudo dnf install webkit2gtk3-devel gtk3-devel libappindicator-gtk3-devel
```

**macOS**:
```bash
# Usually no extra dependencies needed
# Ensure Xcode command line tools are installed
xcode-select --install
```

**Windows**:
```bash
# Usually no extra dependencies needed
# Ensure Visual Studio Build Tools are installed
```

### 6. Runtime Errors

#### Error: Chunk Write Failed
**Cause**: Failed to write Minecraft region file.

**Solutions**:
1. Check available disk space
2. Ensure Minecraft is not running
3. Verify write permissions
4. Check if antivirus is blocking writes
5. Try a different world/location

#### Error: Coordinate Out of Bounds
**Cause**: Generated coordinates exceed Minecraft limits.

**Solutions**:
1. Reduce world scale (`--scale` parameter)
2. Use smaller bounding box
3. Adjust ground level if needed

**Minecraft Limits**:
- X/Z: -30,000,000 to 30,000,000
- Y: -64 to 319

### 7. Version and Update Errors

#### Error: Version Check Failed
**Message**: `Error checking for version updates: [error]`

**Cause**: Unable to reach GitHub to check for updates.

**Impact**: Non-critical, program continues normally.

**Solutions**:
1. Check internet connection
2. Verify GitHub is accessible
3. Check firewall settings
4. Ignore if offline (error is non-fatal)

## Error Recovery Strategies

### Automatic Recovery
The program implements several automatic recovery strategies:

1. **Download Method Fallback**:
   - Tries `reqwest` first
   - Falls back to `curl` if available
   - Falls back to `wget` if available

2. **Graceful Element Skipping**:
   - Invalid elements are logged and skipped
   - Processing continues with remaining elements

3. **Partial Progress Saving**:
   - Chunks are saved incrementally
   - Progress is not lost if program terminates

### Manual Recovery

#### Resuming After Failure
Currently, Arnis doesn't support resume from checkpoint. If generation fails:

1. **For Network Failures**:
   ```bash
   # Save data first
   arnis --bbox="..." --save-json-file data.json --path="..."
   
   # Then process (can retry without re-downloading)
   arnis --file data.json --path="..."
   ```

2. **For Processing Failures**:
   - Use a new/empty world
   - Or manually delete corrupted region files
   - Re-run the generation

#### Debugging Failures
Enable debug mode for detailed output:

```bash
arnis --bbox="..." --path="..." --debug
```

This creates `parsed_osm_data.txt` with all elements processed.

## Best Practices

### 1. Data Caching
Always save OSM data for reuse:
```bash
arnis --bbox="..." --save-json-file area.json --path="..."
```

### 2. Test with Small Areas
Start with a small bounding box to verify:
```bash
# Small test area
arnis --bbox="40.7128,-74.0060,40.7150,-74.0030" --path="..."
```

### 3. Monitor Resources
- Watch memory usage during generation
- Close unnecessary applications
- Use Task Manager/Activity Monitor to track

### 4. Backup Worlds
Always backup your Minecraft world before generation:
```bash
# Copy world folder before running Arnis
cp -r "path/to/world" "path/to/world_backup"
```

### 5. Incremental Testing
Test features incrementally:
```bash
# Test without terrain first
arnis --bbox="..." --path="..."

# Then add terrain
arnis --bbox="..." --path="..." --terrain
```

## Troubleshooting Checklist

### Before Running
- [ ] Minecraft world exists and is accessible
- [ ] Sufficient disk space (estimate: 100MB per km²)
- [ ] Sufficient RAM (see memory estimation above)
- [ ] Internet connection active (if downloading data)
- [ ] Minecraft is closed (unlock world files)
- [ ] Valid bounding box coordinates
- [ ] Latest version of Arnis installed

### During Execution
- [ ] Progress bar is updating
- [ ] No error messages in console
- [ ] System resources not maxed out
- [ ] No disk space warnings

### After Generation
- [ ] World files were created/updated
- [ ] World loads in Minecraft
- [ ] Structures appear as expected
- [ ] No corrupted chunks

## Getting Help

### Information to Provide
When reporting issues, include:

1. **System Information**:
   - OS and version
   - Available RAM
   - Rust version: `rustc --version`
   - Arnis version

2. **Command Used**:
   ```bash
   # Copy the exact command you ran
   arnis --bbox="..." --path="..." [other options]
   ```

3. **Error Output**:
   - Full error message
   - Stack trace if available
   - Debug output if applicable

4. **Context**:
   - Bounding box size/location
   - Size of area in km²
   - World scale used
   - Any special configuration

### Where to Get Help
- **GitHub Issues**: https://github.com/louis-e/arnis/issues
- **Discord**: https://discord.gg/mA2g69Fhxq
- **Documentation**: https://github.com/louis-e/arnis/wiki

### Common Solutions Summary

| Error Type | Quick Fix |
|------------|-----------|
| Timeout | Smaller area or save data with `--save-json-file` |
| Path not found | Verify Minecraft world path exists |
| Out of memory | Reduce bounding box size |
| Permission denied | Close Minecraft, check permissions |
| Invalid bbox | Check coordinate format and order |
| Build errors | Check you're building the correct crate |
| Chunk write failed | Ensure disk space and Minecraft is closed |

## Metrics and Diagnostics

### Enable Metrics (if compiled with feature)
```bash
arnis --bbox="..." --path="..." --metrics-out metrics.json
```

This generates a JSON file with:
- Memory usage (RSS, virtual)
- Processing times
- System information

### Performance Monitoring
For detailed performance analysis:
```bash
# Install perf tools
cargo install flamegraph

# Run with profiling
cargo flamegraph --bin arnis -- --bbox="..." --path="..."
```

## Conclusion

Most errors in Arnis are recoverable with appropriate error handling. The key strategies are:
1. Save downloaded data for reuse
2. Start with small test areas
3. Monitor system resources
4. Use debug mode when troubleshooting
5. Keep backups of important worlds

For issues not covered here, consult the GitHub issues or Discord community.
