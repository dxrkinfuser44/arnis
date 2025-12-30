# Arnis AI Agent Skills Reference

## Table of Contents

1. [Skill Categories](#skill-categories)
2. [Core Skills](#core-skills)
3. [Advanced Skills](#advanced-skills)
4. [Specialized Skills](#specialized-skills)
5. [Skill Development Path](#skill-development-path)
6. [Practical Examples](#practical-examples)

---

## Skill Categories

### 1. Data Acquisition & Processing
- OSM data fetching and parsing
- Elevation data retrieval
- JSON/XML data manipulation
- API interaction and error handling

### 2. Coordinate Mathematics
- Geographic to Cartesian conversion
- Bounding box calculations
- Scaling and transformation
- Clipping algorithms

### 3. Algorithm Implementation
- Pathfinding and connectivity
- Flood-fill algorithms
- Line drawing (Bresenham's)
- Polygon operations

### 4. Block Generation
- Material selection
- Height calculation
- Block property management
- Multi-block structures

### 5. Optimization
- Performance profiling
- Memory management
- Parallel processing
- Caching strategies

---

## Core Skills

### CS-001: OSM Data Understanding

**Description**: Ability to understand and work with OpenStreetMap data structures.

**Prerequisites**: None

**Knowledge Areas**:
- OSM element types (nodes, ways, relations)
- Tag schema and conventions
- Role assignments in relations
- Multipolygon structures

**Practical Tasks**:
```rust
// Parse a building with courtyard (multipolygon)
// Identify outer and inner ways
// Extract relevant tags (height, material, levels)
// Handle missing or invalid data
```

**Validation**:
- Can explain the difference between a way and a relation
- Can identify when to use relations vs ways
- Understands common tag patterns (building=*, amenity=*, etc.)

---

### CS-002: Coordinate Transformation

**Description**: Transform between geographic and Minecraft coordinate systems.

**Prerequisites**: Basic geometry

**Knowledge Areas**:
- WGS84 coordinate system
- Mercator projection considerations
- Scale factor application
- Bounding box operations

**Practical Tasks**:
```rust
// Convert lat/lng to Minecraft x/z
let llpoint = LLPoint::new(51.5074, -0.1278)?; // London
let xzpoint = transformer.transform_point(llpoint);

// Handle edge cases (poles, date line)
// Apply custom scaling
// Validate coordinate bounds
```

**Validation**:
- Can transform coordinates accurately
- Handles edge cases without panic
- Understands scale factor impact

---

### CS-003: WorldEditor Operations

**Description**: Use the WorldEditor API to place blocks in Minecraft worlds.

**Prerequisites**: CS-002

**Knowledge Areas**:
- Relative vs absolute coordinates
- Ground level reference
- Block replacement rules
- Region file management

**Practical Tasks**:
```rust
// Place blocks at correct heights
editor.set_block(STONE, x, y, z, None, None);

// Fill volumes efficiently
editor.fill_blocks(DIRT, x1, y1, z1, x2, y2, z2, None, None);

// Check existing blocks
if editor.check_for_block(x, y, z, Some(&[WATER])) {
    // Avoid placing in water
}
```

**Validation**:
- Can place blocks at correct positions
- Understands when to use relative vs absolute coordinates
- Uses efficient bulk operations

---

### CS-004: Element Processing Pipeline

**Description**: Process OSM elements and generate Minecraft structures.

**Prerequisites**: CS-001, CS-003

**Knowledge Areas**:
- Priority-based processing
- Tag-based routing
- Processor module structure
- Error propagation

**Practical Tasks**:
```rust
// Route elements to correct processors
match element {
    ProcessedElement::Way(way) => {
        if way.tags.contains_key("building") {
            buildings::generate_buildings(&mut editor, way, args, None);
        } else if way.tags.contains_key("highway") {
            highways::generate_highways(&mut editor, element, args, &connectivity);
        }
    }
    // ... handle other cases
}
```

**Validation**:
- Understands processing order and priority
- Can route elements correctly
- Handles missing or invalid tags gracefully

---

### CS-005: Block Material Selection

**Description**: Choose appropriate Minecraft blocks based on OSM tags.

**Prerequisites**: CS-001

**Knowledge Areas**:
- Block definitions and IDs
- Material mapping conventions
- Color matching
- Bedrock vs Java differences

**Practical Tasks**:
```rust
// Map building material
let wall_block = match building_material.as_deref() {
    Some("brick") => BRICK,
    Some("concrete") => CONCRETE,
    Some("wood") => OAK_PLANKS,
    _ => STONE, // default
};

// Consider surface type for roads
let road_surface = match surface.as_deref() {
    Some("asphalt") => GRAY_CONCRETE,
    Some("concrete") => WHITE_CONCRETE,
    Some("gravel") => GRAVEL,
    _ => STONE,
};
```

**Validation**:
- Makes sensible material choices
- Considers real-world context
- Handles missing material tags

---

## Advanced Skills

### AS-001: Building Generation with Roofs

**Description**: Generate complete buildings with walls, floors, and roofs.

**Prerequisites**: CS-003, CS-004, CS-005

**Knowledge Areas**:
- Roof type algorithms (gabled, hipped, flat, etc.)
- Height estimation from tags
- Interior generation
- Courtyard handling

**Implementation Details**:
```rust
// Detect roof type from tags
let roof_type = match roof_shape.as_str() {
    "gabled" => RoofType::Gabled,
    "hipped" => RoofType::Hipped,
    "pyramidal" => RoofType::Pyramidal,
    _ => RoofType::Flat,
};

// Calculate roof slope and height
let roof_height = calculate_roof_height(building_size, roof_type);

// Generate roof geometry
generate_roof(&mut editor, roof_type, min_x, max_x, min_z, max_z, 
               base_height, roof_height, roof_block);
```

**Advanced Techniques**:
- Asymmetric roof handling
- Dormer window placement
- Ridge line calculation
- Overhang generation

**Validation**:
- Generates visually correct roofs
- Handles complex building shapes
- Maintains structural integrity

---

### AS-002: Highway Connectivity Analysis

**Description**: Build and use connectivity maps for realistic road intersections.

**Prerequisites**: CS-004

**Knowledge Areas**:
- Graph construction
- Node degree calculation
- Intersection detection
- Path smoothing

**Implementation Details**:
```rust
// Build connectivity map
let mut connectivity: HashMap<u64, Vec<u64>> = HashMap::new();
for element in elements {
    if let ProcessedElement::Way(way) = element {
        if way.tags.contains_key("highway") {
            for window in way.nodes.windows(2) {
                connectivity.entry(window[0].id)
                    .or_default()
                    .push(window[1].id);
                connectivity.entry(window[1].id)
                    .or_default()
                    .push(window[0].id);
            }
        }
    }
}

// Use for intersection handling
let connections = connectivity.get(&node.id).map(|v| v.len()).unwrap_or(0);
if connections >= 3 {
    // This is an intersection
    create_intersection(&mut editor, node, connections);
}
```

**Validation**:
- Correctly identifies intersections
- Creates smooth road connections
- Handles T-junctions and roundabouts

---

### AS-003: Water Area Generation with Flood-Fill

**Description**: Generate realistic water bodies using flood-fill algorithms.

**Prerequisites**: CS-003, CS-004

**Knowledge Areas**:
- Flood-fill with timeout
- Multipolygon hole handling
- Water depth calculation
- Performance optimization

**Implementation Details**:
```rust
// Flood-fill with boundary checking
fn flood_fill(
    editor: &mut WorldEditor,
    start_x: i32,
    start_z: i32,
    boundary: &HashSet<(i32, i32)>,
    timeout: Option<Duration>,
) {
    let start_time = Instant::now();
    let mut queue = VecDeque::new();
    let mut visited = HashSet::new();
    
    queue.push_back((start_x, start_z));
    
    while let Some((x, z)) = queue.pop_front() {
        if timeout.map(|t| start_time.elapsed() > t).unwrap_or(false) {
            break; // Timeout reached
        }
        
        if visited.contains(&(x, z)) || boundary.contains(&(x, z)) {
            continue;
        }
        
        visited.insert((x, z));
        editor.set_block(WATER, x, 0, z, None, None);
        
        // Add neighbors
        for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1)] {
            queue.push_back((x + dx, z + dz));
        }
    }
}
```

**Advanced Techniques**:
- Island detection within water
- Water depth variation
- Shore smoothing
- Large area optimization

---

### AS-004: Elevation Data Integration

**Description**: Fetch and apply real-world elevation data to terrain.

**Prerequisites**: CS-002, CS-003

**Knowledge Areas**:
- Tile-based elevation systems
- Terrarium format decoding
- Gaussian smoothing
- Outlier filtering

**Implementation Details**:
```rust
// Fetch elevation for area
let elevation = fetch_elevation_data(bbox, scale)?;

// Apply to blocks
for x in min_x..=max_x {
    for z in min_z..=max_z {
        let height = elevation.get_height(x, z);
        let adjusted_y = ground_level + height;
        
        // Place terrain blocks
        editor.set_block_absolute(GRASS_BLOCK, x, adjusted_y, z, None, None);
        for y in (min_y..adjusted_y).rev() {
            editor.set_block_absolute(STONE, x, y, z, None, None);
        }
    }
}
```

**Validation**:
- Accurate height mapping
- Smooth terrain transitions
- Handles data gaps

---

### AS-005: Custom Map Transformations

**Description**: Create and apply custom transformations to OSM data.

**Prerequisites**: CS-001, CS-004

**Knowledge Areas**:
- Transformation operators
- JSON configuration
- Element filtering
- Coordinate translation

**Implementation Details**:
```rust
// Define transformation in JSON
{
    "operations": [
        {
            "type": "translate",
            "offset_x": 100,
            "offset_z": -50
        },
        {
            "type": "filter",
            "tags": {
                "building": "*"
            }
        }
    ]
}

// Apply transformation
pub fn transform_map(
    elements: &mut Vec<ProcessedElement>,
    xzbbox: &mut XZBBox,
    ground: &mut Ground,
) {
    let config = load_transformation_config()?;
    for operation in config.operations {
        apply_operation(elements, operation);
    }
}
```

---

## Specialized Skills

### SP-001: Parallel Processing Optimization

**Description**: Use rayon to parallelize expensive operations.

**Prerequisites**: AS-001, AS-002

**Knowledge Areas**:
- Rayon parallel iterators
- Thread-safe data structures
- Work distribution
- Performance measurement

**Example**:
```rust
use rayon::prelude::*;

// Parallel element processing
elements.par_iter()
    .for_each(|element| {
        // Process each element in parallel
        process_element(element);
    });

// Parallel ground generation
(min_x..=max_x).into_par_iter()
    .for_each(|x| {
        for z in min_z..=max_z {
            generate_ground_block(x, z);
        }
    });
```

---

### SP-002: Bedrock Edition Support

**Description**: Generate worlds compatible with Minecraft Bedrock Edition.

**Prerequisites**: CS-003

**Knowledge Areas**:
- LevelDB storage
- Bedrock block IDs
- MCWorld package format
- NBT differences

**Implementation**:
```rust
// Convert Java block to Bedrock
fn to_bedrock_block(java_block: &str) -> BedrockBlock {
    match java_block {
        "stone" => BedrockBlock::new("stone"),
        "oak_planks" => BedrockBlock::new("planks")
            .with_state("wood_type", "oak"),
        _ => BedrockBlock::new("stone"),
    }
}

// Package as .mcworld
fn create_mcworld(world_path: &Path, output: &Path) -> Result<()> {
    let mut zip = ZipWriter::new(File::create(output)?);
    // Add world files to zip
    // ...
}
```

---

### SP-003: GUI Event Handling

**Description**: Implement frontend-backend communication in Tauri.

**Prerequisites**: None (separate from core generation)

**Knowledge Areas**:
- Tauri commands
- Event emission
- State management
- Error serialization

**Example**:
```rust
#[tauri::command]
async fn start_generation(
    bbox: String,
    path: String,
    settings: GenerationSettings,
) -> Result<String, String> {
    // Parse inputs
    let llbbox = LLBBox::from_str(&bbox)?;
    
    // Start generation
    std::thread::spawn(move || {
        generate_world(llbbox, path, settings);
    });
    
    Ok("Generation started".to_string())
}

// Emit progress updates
emit_gui_progress_update(50.0, "Processing buildings...");
```

---

### SP-004: Memory-Efficient Large Area Processing

**Description**: Handle very large geographic areas without running out of memory.

**Prerequisites**: AS-001, SP-001

**Techniques**:
- Chunk-based processing
- Streaming data access
- Lazy evaluation
- Memory profiling

**Implementation**:
```rust
// Process in chunks
const CHUNK_SIZE: i32 = 512;

for chunk_x in (min_x..max_x).step_by(CHUNK_SIZE as usize) {
    for chunk_z in (min_z..max_z).step_by(CHUNK_SIZE as usize) {
        let chunk_bbox = XZBBox::new(
            chunk_x, chunk_z,
            chunk_x + CHUNK_SIZE, chunk_z + CHUNK_SIZE
        );
        
        // Process only elements in this chunk
        let chunk_elements = clip_elements_to_bbox(&elements, &chunk_bbox);
        process_chunk(&mut editor, chunk_elements);
        
        // Free memory
        drop(chunk_elements);
    }
}
```

---

## Skill Development Path

### Level 1: Novice Agent
**Focus**: Understanding the codebase structure
- Read through AGENT.md
- Understand data flow
- Trace simple element (tree) through pipeline
- Read existing tests

**Projects**:
- Document a single processor module
- Add a simple test case
- Fix typos or improve comments

### Level 2: Intermediate Agent
**Focus**: Making meaningful contributions
- Add support for new OSM tags
- Implement simple element processors
- Optimize existing algorithms
- Write comprehensive tests

**Projects**:
- Add support for `amenity=bench`
- Implement `barrier=hedge`
- Optimize building material selection
- Add unit tests for coordinate transformation

### Level 3: Advanced Agent
**Focus**: Complex features and refactoring
- Design new element processors
- Implement complex algorithms
- Refactor for performance
- Add cross-platform support

**Projects**:
- Implement roundabout detection
- Add support for bridges with elevation
- Refactor highway connectivity
- Add macOS-specific GUI features

### Level 4: Expert Agent
**Focus**: Architecture and major features
- Design system-wide improvements
- Add new world formats
- Implement advanced algorithms
- Performance tuning at scale

**Projects**:
- Add support for Minecraft 1.20+ blocks
- Implement LOD (level of detail) system
- Add real-time generation preview
- Design plugin architecture

---

## Practical Examples

### Example 1: Adding Support for `amenity=fountain`

```rust
// In src/element_processing/amenities.rs

pub fn generate_amenities(
    editor: &mut WorldEditor,
    element: &ProcessedElement,
    args: &Args,
) {
    let tags = element.tags();
    
    // Add fountain handling
    if tags.get("amenity") == Some(&"fountain".to_string()) {
        generate_fountain(editor, element);
        return;
    }
    
    // ... existing amenity handlers
}

fn generate_fountain(editor: &mut WorldEditor, element: &ProcessedElement) {
    match element {
        ProcessedElement::Node(node) => {
            let x = node.x;
            let z = node.z;
            
            // Create fountain structure
            // Base pool
            for dx in -2..=2 {
                for dz in -2..=2 {
                    if dx.abs() + dz.abs() <= 3 {
                        editor.set_block(WATER, x + dx, 0, z + dz, None, None);
                        editor.set_block(STONE_BRICKS, x + dx, -1, z + dz, None, None);
                    }
                }
            }
            
            // Center pillar
            editor.fill_blocks(STONE_BRICKS, x, 0, z, x, 2, z, None, None);
        }
        _ => {} // Fountains are typically nodes
    }
}
```

### Example 2: Optimizing Building Processing

```rust
// Before: Process buildings sequentially
for element in elements {
    if is_building(element) {
        generate_building(editor, element);
    }
}

// After: Use rayon for parallel processing
use rayon::prelude::*;

// Group buildings by region to avoid conflicts
let building_groups = group_by_region(&buildings);

building_groups.par_iter().for_each(|group| {
    let mut local_editor = editor.clone_for_region(group.region);
    for building in group.buildings {
        generate_building(&mut local_editor, building);
    }
});
```

### Example 3: Adding Debug Visualization

```rust
// Add debug output for coordinate transformation
pub fn transform_point(&self, llpoint: LLPoint) -> XZPoint {
    let x = self.transform_x(llpoint.lng());
    let z = self.transform_z(llpoint.lat());
    
    #[cfg(debug_assertions)]
    if std::env::var("DEBUG_COORDS").is_ok() {
        println!("Transform: ({}, {}) -> ({}, {})", 
                 llpoint.lat(), llpoint.lng(), x, z);
    }
    
    XZPoint { x, z }
}
```

---

## Conclusion

These skills build upon each other to create a comprehensive understanding of the Arnis codebase. Start with core skills, master them through practice, then progressively tackle more advanced topics. Each skill should be validated through working code and tests before moving to the next level.

Remember: The best way to learn is by doing. Pick a small feature, implement it completely, and learn from the experience.