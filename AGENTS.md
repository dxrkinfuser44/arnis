# Arnis - Agent Development Guidelines

Arnis generates Minecraft worlds from OpenStreetMap data. This file provides essential information for agentic coding agents working in this repository.

## Build/Test/Lint Commands

```bash
# Build (CLI only, no GUI)
cargo build --no-default-features --release

# Build (all features including GUI)
cargo build --all-targets --all-features --release

# Run all tests
cargo test --all-targets --all-features

# Run a single test (example: test_flags in args.rs)
cargo test test_flags -- --nocapture

# Run tests in a specific module
cargo test args::tests -- --nocapture

# Check formatting
cargo fmt -- --check

# Auto-format code
cargo fmt

# Run clippy lints
cargo clippy --all-targets --all-features -- -D warnings

# Clean build artifacts
cargo clean
```

## Project Structure

- **Language**: Rust (Edition 2021)
- **Binary**: `arnis` - Minecraft world generator
- **Features**: 
  - `gui` (default) - Tauri-based GUI
  - `bedrock` - Bedrock Edition support
- **Entry point**: `src/main.rs`
- **Key modules**: `osm_parser`, `data_processing`, `element_processing`, `world_editor`, `coordinate_system`, `block_definitions`

## Code Style Guidelines

### Formatting
- Run `cargo fmt` before committing (enforced in CI)
- Standard Rust formatting (4-space indentation, 100 char line limit)

### Naming Conventions
- **Functions/variables**: `snake_case`
- **Types/structs/enums**: `PascalCase`
- **Constants**: `SCREAMING_SNAKE_CASE` (e.g., `STONE_BRICKS`)
- **Modules**: `snake_case`

### Imports Order
Group imports in this order with blank lines between groups:
1. External crate imports (`serde::`, `colored::`, etc.)
2. Standard library (`std::`)
3. Crate imports (`crate::...`)

Example:
```rust
use fastnbt::Value;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

use crate::colors::RGBTuple;
```

### Type Definitions
Use type aliases for complex tuple types to improve readability:
```rust
type ColorTuple = (u8, u8, u8);
type BlockOptions = &'static [Block];
```

### Performance
- Use `#[inline(always)]` for small, frequently-called functions
- Use `rayon` for parallel processing
- Cache expensive computations when appropriate

### Conditional Compilation
Use feature gates appropriately:
```rust
#[cfg(feature = "gui")]
mod gui;

#[cfg(feature = "bedrock")]
mod bedrock_block_map;

#[cfg(test)]
mod test_utilities;
```

### Error Handling
- Use `.expect("descriptive message")` for unrecoverable errors in production code
- Use `.unwrap()` only in tests or when error is truly impossible
- Return `Result` types for fallible operations
- Use `eprintln!` for error messages with `.red().bold()` styling

### Testing
- Inline tests in the same file using `#[cfg(test)] mod tests`
- Use `tempfile::TempDir` for temporary directories in tests
- Tests for `Args` require a tempdir with a `region` subdirectory:
```rust
fn minecraft_tmpdir() -> tempfile::TempDir {
    let tmpdir = tempfile::tempdir().unwrap();
    let region_path = tmpdir.path().join("region");
    std::fs::create_dir(&region_path).unwrap();
    tmpdir
}
```

### Comments
- Use `//` for inline comments explaining logic
- Use `///` for doc comments on public items
- Keep comments concise and focused on "why", not "what"

### Structs and Enums
- Always derive common traits: `#[derive(Debug, Clone, PartialEq)]`
- Use `Copy` for small, immutable types
- Document public fields with `///` comments

### Safety
- Mark unsafe blocks with `// SAFETY:` comments explaining why it's safe
- Minimize unsafe code; prefer safe Rust alternatives

## CI/CD Notes

- CI runs on PRs modifying `src/`, `Cargo.toml`, `Cargo.lock`, or `.github/`
- All PRs must pass:
  1. Format check (`cargo fmt -- --check`)
  2. Clippy lint check (`cargo clippy --all-targets --all-features -- -D warnings`)
  3. All tests (`cargo test --all-targets --all-features`)
  4. Release build (`cargo build --all-targets --all-features --release`)
- Benchmark runs automatically on PRs to check performance regression

## Project-Specific Patterns

### Block Definitions
Blocks are defined as const values with numeric IDs in `block_definitions.rs`:
```rust
pub struct Block { id: u8 }
impl Block {
    #[inline(always)]
    const fn new(id: u8) -> Self {
        Self { id }
    }
    pub const STONE_BRICKS: Block = Block::new(83);
}
```

### Coordinate Systems
- Geographic: `LLPoint`, `LLBBox` (lat/lon)
- Cartesian: `XZPoint`, `XZBBox` (Minecraft coordinates)
- Use `CoordTransformer` for conversions

### Progress Updates
When running with GUI, emit progress:
```rust
progress::emit_gui_progress_update(0.5, "Processing data...");
```

### OSM Data Flow
1. Fetch from Overpass API or file (`retrieve_data`)
2. Parse to normalized elements (`osm_parser`)
3. Transform map data (`map_transformation`)
4. Process elements to blocks (`element_processing`)
5. Write to Minecraft world (`world_editor`)

## External Dependencies

Key crates used:
- `tauri` - GUI framework
- `geo`, `serde` - Geospatial data and JSON
- `fastnbt`, `fastanvil` - Minecraft world formats
- `reqwest` - HTTP client for data fetching
- `rayon` - Parallel processing
- `colored` - Terminal colors
- `clap` - CLI argument parsing
- `image` - Image processing

## Resources

- Repository: https://github.com/louis-e/arnis
- Wiki: https://github.com/louis-e/arnis/wiki/
- License: Apache-2.0
