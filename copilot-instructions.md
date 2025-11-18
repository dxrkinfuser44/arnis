# Copilot Instructions for Arnis Project

## Agent Workflow

### 1. Session Initialization
**ALWAYS start by reading `context.md`** to understand:
- Current project state and structure
- Recent changes and active development areas
- Known issues and performance considerations
- Build and test procedures

### 2. Task Execution
Follow these principles when working on Arnis:

#### Code Changes
- **Minimal modifications**: Make the smallest possible changes to achieve the goal
- **Never delete working code** unless absolutely necessary or fixing a security vulnerability
- **Preserve existing functionality**: Don't break existing behavior
- **Follow Rust conventions**: Use idiomatic Rust patterns
- **Document changes**: Add clear comments for complex logic
- **Test thoroughly**: Run `cargo test` before finalizing

#### Module-Specific Guidelines

**Element Processing** (`element_processing/`)
- Each module handles specific OSM element types (buildings, highways, water, etc.)
- Use parallel processing with `rayon` where appropriate
- Maintain consistent coordinate transformation patterns
- Follow existing priority system for element rendering order

**World Editor** (`world_editor.rs`)
- Use `fastanvil` for Minecraft world file I/O
- Handle chunk boundaries carefully
- Optimize for batch operations when possible
- Validate block coordinates before writing

**Data Processing** (`data_processing.rs`)
- Coordinate main generation pipeline
- Manage memory efficiently for large datasets
- Use progress reporting for long operations
- Handle errors gracefully with helpful messages

**Coordinate System** (`coordinate_system/`)
- Maintain precision in geographic transformations
- Use appropriate projections for different scales
- Cache transformation results when beneficial

#### Performance Optimization
- Profile with `--features metrics` before optimizing
- Use `rayon` for CPU-bound parallel work
- Consider SIMD for compute-heavy operations (with `simd-native` feature)
- Minimize allocations in hot paths
- Use iterators instead of collecting when possible

### 3. Testing
```bash
# Run all tests
cargo test

# Test specific module
cargo test osm_parser

# Test with features
cargo test --features metrics

# CLI test run (requires valid Minecraft world path)
cargo run --no-default-features -- --terrain --path="/path/to/world" --bbox="lat1,lng1,lat2,lng2"
```

### 4. Building
```bash
# Debug build with GUI
cargo build

# Release build CLI only
cargo build --release --no-default-features

# With SIMD optimizations
RUSTFLAGS="-C target-cpu=native" cargo build --release --features simd-native
```

### 5. Linting and Formatting
```bash
# Format code
cargo fmt

# Run clippy
cargo clippy --all-targets --all-features
```

### 6. Session Completion
**ALWAYS update `context.md`** at the end of your session with:
- Summary of changes made
- New issues discovered
- Performance impacts (if measured)
- Next steps or recommendations
- Update timestamp in "Last Updated" field

## Common Tasks

### Adding New OSM Element Support
1. Create new module in `element_processing/`
2. Implement element parsing and rendering logic
3. Add to priority system in `osm_parser.rs`
4. Update `element_processing/mod.rs`
5. Add tests for new element type
6. Document in code and update README if user-facing

### Performance Investigation
1. Enable metrics: `--features metrics`
2. Run with `--metrics-out metrics.json`
3. Analyze memory usage and timing
4. Profile with `cargo flamegraph` if needed
5. Make targeted optimizations
6. Verify improvements with metrics

### Bug Fixing
1. Reproduce the issue reliably
2. Write failing test case
3. Fix with minimal changes
4. Verify test passes
5. Check for similar issues elsewhere
6. Update context.md with the fix

### Adding Features
1. Review existing architecture
2. Design minimal API surface
3. Implement feature-gated if optional
4. Add comprehensive tests
5. Update documentation
6. Consider cross-platform implications

## Security Considerations
- Validate all external data (OSM, elevation)
- Check bounds before array access
- Handle file I/O errors gracefully
- Avoid unsafe code unless absolutely necessary
- Review dependencies for known vulnerabilities

## Cross-Platform Notes

### Windows
- Uses Windows API for console handling
- Path separators handled by `PathBuf`
- Test on Windows if making filesystem changes

### macOS
- Apple Silicon support via `simd-native` feature
- Unified memory architecture considerations
- Test both x86_64 and aarch64 if possible

### Linux
- Primary development platform
- Standard POSIX paths and APIs
- Test with various distributions if doing system calls

## Git Workflow
```bash
# Check status
git status

# Create feature branch
git checkout -b feature/description

# Commit changes
git add .
git commit -m "Clear description of changes"

# Push to fork
git push origin feature/description
```

## Getting Help
- **Documentation**: Check GitHub Wiki first
- **Code examples**: Look at similar existing implementations
- **Community**: Join Discord for discussions
- **Issues**: Search existing issues before creating new ones

## Anti-Patterns to Avoid
❌ Removing tests without understanding them  
❌ Breaking existing public APIs  
❌ Adding unnecessary dependencies  
❌ Ignoring clippy warnings without good reason  
❌ Copy-pasting code instead of refactoring  
❌ Premature optimization without profiling  
❌ Platform-specific code without feature gates  
❌ Hardcoding values that should be configurable  

## Best Practices
✅ Write self-documenting code with clear names  
✅ Add tests for new functionality  
✅ Use type system to prevent errors  
✅ Handle errors explicitly, avoid unwrap() in library code  
✅ Use feature flags for optional functionality  
✅ Benchmark before and after optimizations  
✅ Document why, not just what  
✅ Keep functions focused and small  

---

**Remember**: Always read `context.md` at the start and update it at the end of your session. This ensures continuity and effective collaboration across agent sessions.

## Appendix: Detailed Examples

### Example 1: Adding a New OSM Element Type

**Scenario**: Adding support for OSM `leisure=park` elements.

**Step-by-step**:
```bash
# 1. Create new module
touch crates/arnis-core/src/element_processing/parks.rs

# 2. Add basic structure
# (See existing modules like water_areas.rs for template)

# 3. Register in mod.rs
echo "pub mod parks;" >> crates/arnis-core/src/element_processing/mod.rs

# 4. Add to OSM parser priority system
# Edit osm_parser.rs to include park handling

# 5. Add tests
# Add test functions in parks.rs or tests/ directory

# 6. Test
cargo test element_processing::parks
cargo run --no-default-features -- --bbox="..." --path="..."

# 7. Update context.md
# Add to Recent Changes: "Added support for leisure=park OSM elements"
```

### Example 2: Performance Optimization

**Scenario**: Reduce memory usage in building generation.

**Investigation**:
```bash
# 1. Enable metrics
cargo run --no-default-features --features metrics -- \
  --bbox="..." --path="..." --metrics-out before.json

# 2. Identify bottleneck
# Review metrics, use profiler if needed

# 3. Make targeted change
# Example: Use iterators instead of collecting to Vec

# 4. Measure improvement
cargo run --no-default-features --features metrics -- \
  --bbox="..." --path="..." --metrics-out after.json

# 5. Compare results
diff before.json after.json

# 6. Update context.md
# Performance Considerations: "Reduced building gen memory by 20%"
```

### Example 3: Cross-Platform Testing

**Scenario**: Ensure a file path change works on all platforms.

```rust
// ❌ Bad - hardcoded separator
let path = format!("{}/region/r.0.0.mca", world_path);

// ✅ Good - using PathBuf
let path = PathBuf::from(world_path)
    .join("region")
    .join("r.0.0.mca");
```

**Testing**:
```bash
# Test on current platform
cargo test

# Simulate path handling
cargo test --test path_tests -- --nocapture

# Update context.md if issues found on specific platforms
```

### Example 4: Context Update After Bug Fix

**Before work - Read context.md**:
```markdown
### Known Issues
- Coordinate transformation fails for southern hemisphere (#123)
```

**After fix - Update context.md**:
```markdown
### Recent Changes
- Fixed coordinate transformation bug for southern hemisphere (issue #123)
- Added test cases for both hemispheres in coordinate_system tests

### Known Issues
[Remove the fixed issue from this section]
```

### Example 5: Using Feature Flags

**Scenario**: Adding optional feature that depends on new dependency.

**In Cargo.toml**:
```toml
[features]
my-feature = ["some-dependency"]

[dependencies]
some-dependency = { version = "1.0", optional = true }
```

**In code**:
```rust
#[cfg(feature = "my-feature")]
use some_dependency::SomeType;

#[cfg(feature = "my-feature")]
pub fn feature_function() {
    // Implementation
}

#[cfg(not(feature = "my-feature"))]
pub fn feature_function() {
    panic!("my-feature not enabled");
}
```

**Testing**:
```bash
# Test without feature
cargo test

# Test with feature
cargo test --features my-feature

# Update context.md
# Document the new feature in Build System section
```
