# Arnis Project Context

## Project Overview
**Name:** Arnis  
**Version:** 2.3.1  
**Language:** Rust  
**License:** Apache-2.0  
**Repository:** https://github.com/louis-e/arnis

Arnis is an open-source tool that creates complex and accurate Minecraft Java Edition worlds reflecting real-world geography, topography, and architecture. It processes geospatial data from OpenStreetMap and elevation data to generate detailed Minecraft representations of terrain and architecture.

## Project Structure

### Workspace Layout
```
arnis/
├── crates/
│   ├── arnis-cli/          # CLI application entry point
│   └── arnis-core/         # Core library with main logic
├── .github/
│   ├── agents/             # GitHub Copilot agents configuration
│   └── workflows/          # CI/CD workflows
├── assets/                 # Images and resources
├── capabilities/           # Capability definitions
├── tests/                  # Integration tests
└── README.md
```

### Core Modules (arnis-core)
- **args.rs** - Command-line argument parsing
- **block_definitions.rs** - Minecraft block type definitions
- **bresenham.rs** - Line drawing algorithm
- **colors.rs** - Color utilities
- **coordinate_system/** - Geographic coordinate transformations
- **cpu_info.rs** - CPU information detection
- **data_processing.rs** - Main world generation orchestration
- **element_processing/** - OSM element processors (buildings, highways, water, etc.)
- **elevation_data.rs** - Terrain elevation handling
- **floodfill.rs** - Flood fill algorithm for area processing
- **ground.rs** - Ground/terrain generation
- **gui.rs** - GUI integration (feature-gated)
- **map_transformation/** - Coordinate transformation logic
- **metrics.rs** - Performance metrics (feature-gated)
- **osm_parser.rs** - OpenStreetMap data parsing
- **perf_config.rs** - Performance configuration
- **progress.rs** - Progress reporting
- **retrieve_data.rs** - Data fetching from Overpass API
- **version_check.rs** - Version update checking
- **world_editor.rs** - Minecraft world file manipulation

## Key Objectives
1. **Modularity** - Clean separation of components (data fetching, processing, generation)
2. **Performance Optimization** - Efficient world generation with low memory usage
3. **Comprehensive Documentation** - Clear in-code documentation
4. **User-Friendly Experience** - Easy to use for end users
5. **Cross-Platform Support** - Windows, macOS, and Linux compatibility

## Build System

### Features
- **default**: Includes GUI (`gui` feature)
- **gui**: Tauri-based GUI interface
- **simd-native**: SIMD optimizations for Apple Silicon and native CPUs
- **metrics**: Runtime metrics collection

### Build Commands
```bash
# CLI build (no GUI)
cargo run --no-default-features -- --terrain --path="path/to/world" --bbox="min_lat,min_lng,max_lat,max_lng"

# GUI build
cargo run

# With metrics
cargo run --no-default-features --features metrics -- --metrics-out metrics.json

# Apple Silicon optimized
RUSTFLAGS="-C target-cpu=native" cargo build --release --features simd-native
```

## Dependencies
Key dependencies include:
- **fastanvil/fastnbt** - Minecraft world file I/O
- **geo** - Geospatial operations
- **rayon** - Parallel processing
- **reqwest** - HTTP requests for data fetching
- **tauri** - GUI framework (optional)
- **clap** - CLI argument parsing
- **serde/serde_json** - Serialization

## Testing
- Unit tests embedded in source files
- Integration tests in `tests/` directory
- Use `cargo test` to run all tests

## CI/CD
- **ci-build.yml** - Main CI build pipeline
- **release.yml** - Release automation
- Disabled workflows for benchmarking and platform-specific builds

## Current Session Context

### Last Updated
2025-11-17

### Recent Changes
- Implemented asset caching system for OSM data
  - Created `asset_cache.rs` module with full caching functionality
  - Added `--download-only` mode to download and cache without processing
  - Added `--process-only` mode to process from cache without downloading
  - Added `--use-cache` flag for automatic cache usage
  - Implemented cache validation with checksums
  - OS-specific cache directories (Linux: ~/.cache/arnis, etc.)
- Created comprehensive documentation
  - `ARCHITECTURE.md` - Detailed architecture and data flow documentation
  - `ERROR_HANDLING.md` - Error handling patterns and troubleshooting guide
  - `CACHE_USAGE.md` - Complete guide for using the new caching features
- Updated core modules for caching support
  - Modified `retrieve_data.rs` with cache-aware functions
  - Updated `args.rs` with new cache-related CLI arguments
  - Made `--path` optional for download-only mode
  - Added Serialize/Deserialize to LLBBox and LLPoint
- Fixed root Cargo.toml workspace structure
  - Converted to workspace format with members
  - Properly configured arnis-cli and arnis-core as workspace members
- Previous context management work (COMPLETED):
  - Created `context.md` file with comprehensive project information
  - Created `copilot-instructions.md` file with detailed coding guidelines
  - Added context management workflow for coding agents

### Active Development Areas
- Asset caching and offline processing (COMPLETED Phase 2)
- Distributed resource pooling (Phase 3 - PENDING)
- Memory efficiency improvements
- Modular architecture refactoring
- Cross-platform optimization

### Known Issues
- Root `Cargo.toml` workspace structure (FIXED in this session)
  - Now properly configured as a workspace
  - Members: crates/arnis-cli and crates/arnis-core
- Some existing tests fail due to missing test files or network issues
  - map_transformation tests: missing test JSON files
  - One test requires network access (expected for integration tests)

### Performance Considerations
- Use rayon for parallel processing where applicable
- Minimize memory allocations in hot paths
- Consider SIMD optimizations for compute-heavy operations
- Profile with the `metrics` feature when optimizing

## Development Guidelines

### Code Style
- Follow Rust standard conventions
- Use `clippy` for linting
- Format with `rustfmt`
- Add comprehensive documentation for public APIs

### Commit Practices
- Write clear, descriptive commit messages
- Keep commits atomic and focused
- Reference issues in commit messages when applicable

### Pull Request Process
1. Fork the repository
2. Create a feature branch
3. Make changes following key objectives
4. Submit pull request with clear description
5. Maintainer will create releases including merged changes

## Resources
- **Documentation**: GitHub Wiki (https://github.com/louis-e/arnis/wiki/)
- **Official Website**: https://arnismc.com
- **Discord**: https://discord.gg/mA2g69Fhxq
- **OpenStreetMap**: https://en.wikipedia.org/wiki/OpenStreetMap

---
*This context file should be read by coding agents at the start of each session and updated at the end with relevant changes.*
