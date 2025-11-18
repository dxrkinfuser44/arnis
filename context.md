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

### Recent Changes (This Session)
This session implemented distributed resource pooling and asset management:

**Phase 1 - Documentation (COMPLETED):**
- Created `ARCHITECTURE.md` - System architecture and data flow (10KB)
- Created `ERROR_HANDLING.md` - Error handling and troubleshooting guide (11KB)
- Created `CACHE_USAGE.md` - Asset caching usage guide (7.5KB)
- Created `DISTRIBUTED_DESIGN.md` - Distributed system design (9KB)
- Created `IMPLEMENTATION_SUMMARY.md` - Complete implementation summary (10KB)
- Updated `README.md` with caching features and documentation links
- Total documentation: ~47.5KB across 5 new files

**Phase 2 - Asset Caching (COMPLETED):**
- Created `asset_cache.rs` - Full caching system (12.5KB)
- Implemented download-only mode (`--download-only`)
- Implemented process-only mode (`--process-only`)
- Implemented cache-aware mode (`--use-cache`)
- OS-specific cache directories (Linux, macOS, Windows)
- Cache validation with checksums
- 5 comprehensive tests, all passing ✅

**Phase 3.1 - Distributed Core (COMPLETED):**
- Created `distributed/` module with 4 files (14.7KB)
- `work_unit.rs` - Work unit and result structures
- `chunking.rs` - Geographic bbox splitting logic
- `protocol.rs` - HTTP API protocol types
- 9 comprehensive tests, all passing ✅

**Supporting Changes:**
- Fixed `Cargo.toml` workspace structure
- Added Serialize/Deserialize to LLBBox and LLPoint
- Made `dirs` dependency non-optional
- Updated CLI argument parsing
- Fixed all clippy warnings
- Improved code safety

**Build Status:**
- ✅ All builds successful (debug and release)
- ✅ 14 new tests, all passing
- ✅ No clippy warnings
- ✅ Zero unsafe code
- ⚠️ 3 pre-existing test failures (unrelated)

### Active Development Areas
- Distributed resource pooling (Phase 3.1 COMPLETED, 3.2-3.5 PENDING)
  - Core infrastructure complete
  - HTTP server implementation pending
  - Worker/coordinator CLIs pending
- Asset caching and offline processing (COMPLETED)
- Memory efficiency improvements
- Cross-platform optimization

### Known Issues
- Pre-existing test failures (unrelated to new code):
  - map_transformation tests: missing test JSON files
  - One network-dependent test (expected)
  
### Remaining Work for Distributed System
**Phase 3.2 - Data Management:**
- Chunk-level asset caching
- OSM data splitting per chunk
- Result file transfer
- Region file merging

**Phase 3.3 - Worker Implementation:**
- Worker CLI mode
- Work processing loop
- Result upload
- Error handling

**Phase 3.4 - Coordinator Implementation:**
- Coordinator CLI with HTTP server
- Work queue management
- Progress tracking
- Result collection

**Phase 3.5 - Polish:**
- Fault tolerance
- Load balancing
- End-to-end testing
- Documentation finalization

**Estimated remaining: 12-15 hours**

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
