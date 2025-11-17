# Arnis Architecture Documentation

## Overview
Arnis is a Rust-based application that converts real-world geographic data into Minecraft worlds. This document describes the architecture, data flow, and key components of the system.

## High-Level Architecture

```
┌─────────────────┐
│   User Input    │
│  (CLI or GUI)   │
└────────┬────────┘
         │
         v
┌─────────────────┐
│  Data Fetching  │  ← retrieve_data.rs
│   (Overpass)    │
└────────┬────────┘
         │
         v
┌─────────────────┐
│  OSM Parsing    │  ← osm_parser.rs
│  & Processing   │
└────────┬────────┘
         │
         v
┌─────────────────┐
│   Coordinate    │  ← map_transformation.rs
│ Transformation  │     coordinate_system/
└────────┬────────┘
         │
         v
┌─────────────────┐
│    Element      │  ← element_processing/
│   Processing    │     (buildings, highways, etc.)
└────────┬────────┘
         │
         v
┌─────────────────┐
│  World Editor   │  ← world_editor.rs
│ (Minecraft I/O) │
└─────────────────┘
```

## Core Modules

### 1. Data Retrieval (`retrieve_data.rs`)
**Purpose**: Fetch OpenStreetMap data from Overpass API

**Key Functions**:
- `fetch_data_from_overpass()` - Downloads OSM data for a bounding box
- `fetch_data_from_file()` - Loads previously saved OSM JSON data
- `download_with_reqwest()` - Uses reqwest HTTP client
- `download_with_curl()` - Fallback using curl command
- `download_with_wget()` - Fallback using wget command

**Data Sources**:
- Overpass API (OpenStreetMap data)
- Elevation data (various sources)
- Local JSON files (cached data)

**Error Handling**:
- Network timeouts (360 seconds)
- Invalid server responses
- Empty data validation
- Fallback to alternative download methods

### 2. OSM Parser (`osm_parser.rs`)
**Purpose**: Parse OpenStreetMap JSON data into internal representations

**Key Structures**:
- `ProcessedElement` - Enum for nodes, ways, and relations
- `ProcessedNode` - Geographic points with tags
- `ProcessedWay` - Lines/polygons with nodes and tags
- `ProcessedRelation` - Complex features with multiple members

**Processing Steps**:
1. Parse JSON into OSM structures
2. Convert geographic coordinates to Minecraft coordinates
3. Apply world scale transformations
4. Sort elements by rendering priority

**Priority System**:
Elements are rendered in order to prevent z-fighting:
1. Ground/terrain (lowest)
2. Water areas
3. Landuse areas
4. Roads/highways
5. Buildings
6. Details (doors, trees, etc.)

### 3. Coordinate System (`coordinate_system/`)
**Purpose**: Convert between geographic (lat/lon) and Minecraft (x/z) coordinates

**Key Components**:
- `LLBBox` - Latitude/Longitude bounding box
- `XZBBox` - Minecraft X/Z bounding box
- Mercator projection for coordinate transformation
- Scale factor application

**Transformations**:
- WGS84 geographic coordinates → Mercator projection → Minecraft coordinates
- Handles world scale (blocks per meter)
- Maintains precision for large areas

### 4. Element Processing (`element_processing/`)
**Purpose**: Convert OSM elements into Minecraft blocks

**Modules**:
- `buildings.rs` - Building generation with walls, roofs, interiors
- `highways.rs` - Roads, paths, sidewalks
- `water_areas.rs` - Lakes, rivers, reservoirs
- `waterways.rs` - Streams, canals
- `natural.rs` - Trees, forests, beaches
- `landuse.rs` - Parks, farmland, residential areas
- `amenities.rs` - Parking lots, playgrounds
- `railways.rs` - Train tracks, roller coasters
- `bridges.rs` - Bridge structures
- `barriers.rs` - Fences, walls
- `man_made.rs` - Man-made structures
- `tourisms.rs` - Tourist attractions

**Processing Pattern**:
1. Check element tags (OSM metadata)
2. Extract geometric data (coordinates, nodes)
3. Apply appropriate Minecraft block types
4. Handle elevation and terrain interaction
5. Write blocks to world editor

### 5. World Editor (`world_editor.rs`)
**Purpose**: Read and write Minecraft world files

**Key Features**:
- Region file management (.mca files)
- Chunk-based operations
- Block placement with coordinate validation
- Elevation-aware placement
- Batch operations for performance

**Minecraft File Format**:
- Region files: 32×32 chunks
- Chunks: 16×16×384 blocks
- Block coordinates: (x, y, z)
- Y-range: -64 to 319

**Optimization**:
- Caches loaded chunks
- Batch writes to reduce I/O
- Parallel chunk processing where possible

### 6. Ground/Terrain (`ground.rs`)
**Purpose**: Generate terrain elevation data

**Sources**:
- Elevation APIs (when enabled)
- Flat ground (default fallback)
- Interpolated elevation between points

**Features**:
- Configurable ground level
- Terrain smoothing
- Integration with element placement

### 7. Data Processing (`data_processing.rs`)
**Purpose**: Orchestrate the entire world generation pipeline

**Pipeline Steps**:
1. Initialize world editor
2. Set ground reference
3. Process terrain
4. Iterate through all elements
5. Dispatch to appropriate element processor
6. Track progress
7. Save world files

**Progress Tracking**:
- Console progress bar (CLI)
- GUI progress updates
- Percentage completion
- ETA calculation

## Data Flow

### Input Phase
```
User Input (bbox, path, options)
    ↓
Fetch OSM Data (retrieve_data)
    ↓
Parse JSON (osm_parser)
    ↓
Create ProcessedElements
```

### Transformation Phase
```
ProcessedElements
    ↓
Apply Scale (map_transformation)
    ↓
Convert Coordinates (coordinate_system)
    ↓
Sort by Priority
```

### Generation Phase
```
For Each Element:
    ↓
Determine Type (building, highway, etc.)
    ↓
Call Appropriate Processor
    ↓
Calculate Minecraft Blocks
    ↓
Write to World Editor
```

### Output Phase
```
World Editor
    ↓
Write Chunks to Region Files
    ↓
Save to Disk (.mca files)
```

## Configuration

### Command Line Arguments (CLI)
- `--bbox` - Geographic bounding box (required)
- `--path` - Minecraft world path (required)
- `--scale` - Blocks per meter (default: 1.0)
- `--ground-level` - Base elevation (default: -62)
- `--terrain` - Enable terrain generation
- `--interior` - Generate building interiors (default: true)
- `--roof` - Generate building roofs (default: true)
- `--debug` - Enable debug output
- `--downloader` - Method: requests/curl/wget
- `--file` - Load from JSON file instead of API
- `--save-json-file` - Save downloaded data to file
- `--metrics-out` - Export metrics (requires feature)

### Features
- `gui` - Tauri-based graphical interface
- `metrics` - Runtime performance metrics
- `simd-native` - CPU-specific optimizations

## Performance Considerations

### Parallel Processing
- Uses `rayon` for CPU-bound parallel operations
- Thread pool sized based on CPU cores
- Element processing can be parallelized
- Chunk operations are parallelizable

### Memory Management
- Streaming JSON parsing for large datasets
- Chunk-based loading/unloading
- Iterator patterns to avoid collecting large vectors
- Careful allocation in hot paths

### Optimization Opportunities
- SIMD for coordinate transformations
- GPU acceleration for terrain generation
- Distributed processing across multiple machines
- Asset pre-caching to separate download from processing

## Error Handling Patterns

### Network Errors
- Timeouts with configurable duration
- Retry logic for transient failures
- Fallback download methods (reqwest → curl → wget)
- User-friendly error messages

### File I/O Errors
- Validate paths before operations
- Check disk space before writing
- Handle permission errors
- Atomic operations where possible

### Data Validation
- Verify bounding box validity
- Check coordinate ranges
- Validate OSM data structure
- Handle missing or malformed tags

### Recovery Strategies
- Graceful degradation (skip invalid elements)
- Continue processing on non-fatal errors
- Report errors without crashing
- Save partial progress when possible

## Extension Points

### Adding New OSM Element Types
1. Create module in `element_processing/`
2. Implement element detection and processing
3. Add to priority system in `osm_parser.rs`
4. Register in `element_processing/mod.rs`
5. Write tests

### Custom Block Types
1. Add to `block_definitions.rs`
2. Define block properties
3. Use in element processors
4. Document in block mappings

### Alternative Data Sources
1. Extend `retrieve_data.rs`
2. Implement data fetching
3. Convert to ProcessedElement format
4. Integrate with pipeline

### Distributed Computing
1. Split work by geographic region
2. Serialize/deserialize work units
3. Process on multiple machines
4. Merge results into single world

## Testing Strategy

### Unit Tests
- Coordinate transformations
- Element parsing
- Block placement logic
- Bounding box operations

### Integration Tests
- End-to-end data flow
- File I/O operations
- API interactions
- Map transformation examples

### Performance Testing
- Memory usage profiling
- CPU utilization
- I/O throughput
- Large dataset handling

## Dependencies

### Core Dependencies
- `reqwest` - HTTP client for API calls
- `serde`/`serde_json` - JSON parsing
- `fastanvil`/`fastnbt` - Minecraft world I/O
- `rayon` - Parallel processing
- `geo` - Geospatial operations
- `clap` - CLI argument parsing
- `indicatif` - Progress bars

### Optional Dependencies
- `tauri` - GUI framework
- `tokio` - Async runtime (for GUI)
- Platform-specific: `windows` crate

## Future Enhancements

### Planned Features
1. **Asset Pre-caching**
   - Download all data before processing
   - Cache management and validation
   - Offline processing mode

2. **Distributed Computing**
   - Work distribution across nodes
   - Heterogeneous system support
   - Result aggregation

3. **Enhanced Error Recovery**
   - Checkpoint/resume functionality
   - Automatic retry with backoff
   - Better error diagnostics

4. **Performance Improvements**
   - GPU acceleration
   - Advanced caching strategies
   - Incremental world updates

## References
- OpenStreetMap documentation: https://wiki.openstreetmap.org/
- Minecraft world format: https://minecraft.wiki/w/Region_file_format
- Overpass API: https://wiki.openstreetmap.org/wiki/Overpass_API
