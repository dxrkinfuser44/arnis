# Arnis AI Agent Documentation

## Overview

This document provides comprehensive guidance for AI agents working with the Arnis codebase. Arnis is a sophisticated Minecraft world generator that transforms real-world geographic data from OpenStreetMap (OSM) into accurate Minecraft representations.

## Project Architecture

### Core Components

1. **Data Acquisition Layer** (`src/retrieve_data.rs`)
   - Fetches OSM data via Overpass API
   - Supports multiple download methods (reqwest, curl, wget)
   - Handles elevation data from AWS Terrarium tiles
   - Implements timeout and error handling

2. **Parsing & Transformation** (`src/osm_parser.rs`)
   - Deserializes raw OSM JSON data
   - Converts geographic coordinates (lat/lng) to Minecraft coordinates (x/z)
   - Processes three OSM element types: nodes, ways, and relations
   - Implements bbox clipping for performance optimization

3. **Coordinate Systems** (`src/coordinate_system/`)
   - Geographic coordinates (latitude/longitude)
   - Cartesian Minecraft coordinates (x/z)
   - Coordinate transformation with configurable scaling
   - Bounding box calculations and validation

4. **Element Processing** (`src/element_processing/`)
   - Modular processors for different OSM feature types:
     - `buildings.rs` - Generates structures with roofs, interiors, materials
     - `highways.rs` - Roads, paths, intersections with connectivity
     - `water_areas.rs` - Lakes, rivers, oceans with flood-fill
     - `natural.rs` - Trees, forests, terrain features
     - `amenities.rs` - Points of interest (benches, fountains, etc.)
     - `railways.rs` - Train tracks and roller coasters
     - `barriers.rs` - Walls, fences, gates
     - `landuse.rs` - Parks, farmland, industrial areas
   - Priority-based processing ensures correct layering

5. **World Generation** (`src/world_editor/`)
   - Supports both Java Edition (Anvil format) and Bedrock Edition
   - Region-based file management
   - Elevation-aware block placement
   - NBT serialization for world data

6. **Map Transformation** (`src/map_transformation/`)
   - Applies custom transformations defined in JSON
   - Extensible operator system
   - Allows preprocessing of OSM data before world generation

7. **GUI Layer** (`src/gui.rs`)
   - Tauri-based desktop application
   - Interactive map selection
   - Real-time progress updates
   - Cross-platform support (Windows, macOS, Linux)

### Data Flow

```
1. User Input (bbox, settings)
   ↓
2. Fetch OSM Data (retrieve_data)
   ↓
3. Parse Elements (osm_parser)
   ↓
4. Transform Coordinates (coordinate_system)
   ↓
5. Apply Transformations (map_transformation)
   ↓
6. Process Elements by Priority (element_processing)
   ↓
7. Generate Terrain (ground, elevation_data)
   ↓
8. Write World Files (world_editor)
   ↓
9. Render Preview Map (map_renderer)
```

## Agent Capabilities & Skills

### 1. Code Analysis & Understanding

**Skills:**
- Parse Rust module structure and dependencies
- Understand OSM data schema and tagging conventions
- Trace data flow through processing pipeline
- Identify performance bottlenecks
- Analyze coordinate transformations

**Example Tasks:**
- Explain how a building is generated from OSM data
- Document the roof generation algorithm
- Trace how elevation data affects block placement

### 2. Feature Development

**Skills:**
- Add new OSM element processors
- Implement new building materials or styles
- Create custom terrain generation algorithms
- Add support for new OSM tags
- Extend GUI functionality

**Example Tasks:**
- Add support for OSM `aerialway=*` (cable cars)
- Implement roundabout detection for highways
- Create custom building height estimation
- Add new roof shapes (butterfly, sawtooth, etc.)

### 3. Bug Fixes & Optimization

**Skills:**
- Debug coordinate transformation issues
- Fix memory leaks in large world generation
- Optimize flood-fill algorithms
- Resolve NBT serialization errors
- Fix GUI rendering issues

**Example Tasks:**
- Fix buildings clipping through terrain
- Optimize water area generation for large lakes
- Resolve session.lock file conflicts
- Fix Bedrock edition compatibility issues

### 4. Testing & Validation

**Skills:**
- Write unit tests for coordinate transformations
- Create integration tests for element processors
- Validate generated world structure
- Test edge cases (crossing date line, poles)
- Benchmark performance improvements

**Example Tasks:**
- Test building generation with various roof types
- Validate water area flood-fill accuracy
- Benchmark highway connectivity algorithms
- Test multi-platform GUI compatibility

### 5. Documentation

**Skills:**
- Write technical documentation
- Create architecture diagrams
- Document OSM tag mappings
- Write user guides
- Create API documentation

**Example Tasks:**
- Document how to add a new element processor
- Create flowchart for building generation
- Write guide for custom transformations
- Document block material selection logic

## Key Concepts

### OSM Element Types

1. **Nodes**: Points with lat/lng coordinates
   - Examples: trees, benches, traffic lights
   - May have tags describing their purpose

2. **Ways**: Ordered lists of nodes forming lines or polygons
   - Examples: roads, building outlines, rivers
   - Can be open (lines) or closed (polygons)

3. **Relations**: Groups of nodes/ways with roles
   - Examples: multipolygon buildings with courtyards
   - Members have roles like "outer" or "inner"

### Priority System

Elements are processed in priority order to ensure correct layering:
1. Entrances/doors (highest priority)
2. Buildings
3. Highways
4. Waterways
5. Water areas
6. Barriers

This prevents roads from overwriting buildings or water from covering doors.

### Coordinate Transformation

- Geographic coordinates use WGS84 (EPSG:4326)
- Minecraft coordinates are relative to bbox minimum
- Scale factor converts meters to blocks (default: 1.0)
- Elevation data adjusts Y-coordinate based on terrain

### Block Placement

- Ground level: default -62 (configurable)
- Relative Y: offset from ground level
- Absolute Y: fixed world coordinate
- Elevation-aware: adjusts for terrain height

## Common Patterns

### Adding a New Element Processor

1. Create file in `src/element_processing/`
2. Define processing function(s)
3. Register in `src/element_processing/mod.rs`
4. Call from `src/data_processing.rs` match statement
5. Map OSM blocks to Minecraft blocks
6. Handle both ways and relations if applicable

### Working with WorldEditor

```rust
// Set block relative to ground
editor.set_block(STONE, x, y_offset, z, None, None);

// Set block at absolute world coordinate
editor.set_block_absolute(GRASS, x, abs_y, z, None, None);

// Fill area with blocks
editor.fill_blocks(STONE, x1, y1, z1, x2, y2, z2, None, None);

// Check if block exists
if editor.check_for_block(x, y, z, Some(&[STONE, DIRT])) {
    // Block is stone or dirt
}
```

### Error Handling

- Use `Result<T, E>` for recoverable errors
- Propagate errors with `?` operator
- Emit GUI errors with `emit_gui_error()`
- Log warnings without stopping generation
- Use descriptive error messages

## Development Guidelines

### Code Style

- Follow Rust conventions and idioms
- Use meaningful variable names
- Document public APIs with doc comments
- Keep functions focused and small
- Prefer iterators over explicit loops

### Performance Considerations

- Minimize memory allocations in hot loops
- Use rayon for parallel processing where applicable
- Clip ways to bbox early to reduce node count
- Cache expensive calculations (e.g., highway connectivity)
- Profile before optimizing

### Testing

- Write tests in `#[cfg(test)]` modules
- Use `test_utilities.rs` for common test helpers
- Test edge cases (empty data, single point, etc.)
- Use `tempfile` for filesystem tests
- Mock external API calls

### Debugging

- Use `--debug` flag for verbose output
- Check `parsed_osm_data.txt` for OSM elements
- Enable debug prints in specific modules
- Use Rust's `dbg!()` macro for quick inspection
- Verify coordinate transformations first

## Common Issues & Solutions

### Issue: Buildings appear underground
**Solution**: Check ground level setting and elevation data. Ensure `get_absolute_y()` is called correctly.

### Issue: Roads disconnected at intersections
**Solution**: Verify highway connectivity map is being used. Check node merging logic.

### Issue: Water doesn't fill enclosed areas
**Solution**: Ensure relation outer/inner roles are correct. Check flood-fill timeout setting.

### Issue: Memory usage too high
**Solution**: Clip ways earlier in pipeline. Reduce bbox size. Process in chunks.

### Issue: GUI not updating progress
**Solution**: Call `emit_gui_progress_update()` regularly. Check feature flags.

## Agent Skills Matrix

| Skill Level | Capabilities |
|------------|--------------|
| **Beginner** | Read documentation, understand data flow, make simple fixes |
| **Intermediate** | Add new element processors, optimize algorithms, write tests |
| **Advanced** | Refactor architecture, add platform support, implement new formats |
| **Expert** | Design new features, coordinate system changes, performance tuning |

## Resources

- **OSM Wiki**: https://wiki.openstreetmap.org/
- **OSM Tag Info**: https://taginfo.openstreetmap.org/
- **Minecraft Wiki**: https://minecraft.wiki/
- **Anvil Format**: https://minecraft.wiki/w/Anvil_file_format
- **Bedrock Level Format**: https://wiki.bedrock.dev/world-generation/level-format.html
- **Rust Book**: https://doc.rust-lang.org/book/

## Getting Started as an Agent

1. **Understand the Domain**: Learn OSM data structure and Minecraft world format
2. **Trace Code Flow**: Follow a simple element (like a tree) through the entire pipeline
3. **Run Examples**: Generate small test worlds and inspect the output
4. **Read Tests**: Examine existing tests to understand expected behavior
5. **Make Small Changes**: Start with bug fixes or documentation improvements
6. **Build Features**: Once comfortable, add new capabilities

## Advanced Topics

### Custom Map Transformations

Transformations are defined in JSON files (e.g., `capabilities/default.json`) and allow preprocessing OSM data:
- Translate coordinates
- Filter elements by tags
- Merge nearby elements
- Apply custom logic

### Bedrock Edition Support

Bedrock uses a different world format:
- LevelDB for chunk storage
- Different block naming/IDs
- MCWorld package format
- Different NBT structure

### Elevation Integration

Elevation data from AWS Terrarium tiles:
- Fetched at appropriate zoom level
- Gaussian blur for smoothing
- Outlier filtering
- NaN value interpolation

### Parallel Processing

Uses `rayon` for parallelization:
- Element processing
- Ground generation
- Coordinate transformation
- Independent chunks

## Conclusion

Arnis is a complex but well-structured project. As an AI agent, you have access to powerful tools for understanding, modifying, and extending this codebase. Focus on understanding the data flow first, then dive into specific areas of interest. Always test changes with small datasets before processing large areas.

Remember: The goal is to accurately represent real-world geography in Minecraft while maintaining good performance and user experience.