# Performance Issues Analysis

## Executive Summary

Analysis of the Arnis codebase reveals several potential causes for the reported large-scale terrain generation issues:

1. **Stack Overflow in Water Areas**: Recursive flood fill algorithm can cause stack overflow
2. **Excessive Memory Allocation**: Multiple cloning operations in water area processing
3. **Unbounded Recursion**: `merge_loopy_loops` recursion without depth limit
4. **Potential Infinite Loops**: Water area processing can hang on complex geometries

## Issue 1: Stack Overflow in Water Areas ⚠️ CRITICAL

### Location
`crates/arnis-core/src/element_processing/water_areas.rs:236-306`

### Problem
The `inverse_floodfill_recursive` function uses recursion to subdivide large water areas into quadrants. For very large water bodies, this can exceed the stack size.

```rust
fn inverse_floodfill_recursive(
    min: (i32, i32),
    max: (i32, i32),
    outers: &[Polygon],
    inners: &[Polygon],
    editor: &mut WorldEditor,
    start_time: Instant,
) {
    // ... processing ...
    
    for (min_x, max_x, min_z, max_z) in quadrants {
        // ...
        if !outers_intersects.is_empty() {
            inverse_floodfill_recursive(  // <-- RECURSIVE CALL
                (min_x, min_z),
                (max_x, max_z),
                &outers_intersects,
                &inners_intersects,
                editor,
                start_time,
            );
        }
    }
}
```

### Impact
- **StackBufferOverflow errors** for large selections
- **Crashes** when processing large water bodies
- **Blank worlds** if the crash happens mid-processing

### Root Cause
The recursion depth is bounded only by area size. A 100km² area with 1km chunks could require ~10,000 subdivisions, each consuming stack space.

### Solution Needed
Convert to iterative approach using a queue/stack data structure instead of call stack.

## Issue 2: Excessive Cloning in Water Area Merging ⚠️ HIGH

### Location
`crates/arnis-core/src/element_processing/water_areas.rs:93-170`

### Problem
The `merge_loopy_loops` function clones vectors multiple times per merge operation:

```rust
fn merge_loopy_loops(loops: &mut Vec<Vec<ProcessedNode>>) {
    // ...
    if x[0].id == y[0].id {
        removed.push(i);
        removed.push(j);
        
        let mut x: Vec<ProcessedNode> = x.clone();  // <-- CLONE
        x.reverse();
        x.extend(y.iter().skip(1).cloned());  // <-- MORE CLONING
        merged.push(x);
    }
    // ... 4 more similar cases with cloning
    
    if merged_len > 0 {
        merge_loopy_loops(loops);  // <-- RECURSIVE
    }
}
```

### Impact
- **High memory usage** for complex water relations
- **Performance degradation** on large datasets
- **Out of memory errors** possible

### Measurements
- 7 clones found in `water_areas.rs`
- Recursion without depth limit
- O(n²) complexity for n loops

## Issue 3: Unbounded Flood Fill Recursion ⚠️ MEDIUM

### Location
`crates/arnis-core/src/element_processing/water_areas.rs:168`

### Problem
```rust
if merged_len > 0 {
    merge_loopy_loops(loops);  // Recursive call with no depth limit
}
```

For complex geometries with many disconnected segments, this could recurse hundreds of times.

### Impact
- **Stack overflow** on complex water relations
- **Infinite waiting** if the recursion is very deep but doesn't overflow

## Issue 4: Timeout but No Abort ⚠️ MEDIUM

### Location
`crates/arnis-core/src/element_processing/water_areas.rs:244-247`

### Problem
```rust
// Check if we've exceeded 25 seconds
if start_time.elapsed().as_secs() > 25 {
    println!("Water area generation exceeded 25 seconds, continuing anyway");
}
```

The timeout check only prints a warning but continues processing. This can lead to:
- **Infinite waiting** scenarios
- **No user feedback** about stuck processes
- **Resource exhaustion**

## Issue 5: Potential for Floating Water ⚠️ LOW

### Location
`crates/arnis-core/src/element_processing/water_areas.rs:324,340`

### Problem
Water is always placed at `ground_level` (0), which may not match the actual terrain:

```rust
editor.set_block(WATER, x, ground_level, z, None, None);
```

If terrain elevation varies, this creates:
- **Floating water layers** above ground
- **Overlapping terrain** where water should be

### Impact Correlation
Matches user report: "floating layers of water above the ground or the terrain overlapping with the river area"

## Issue 6: Missing Chunk Writes ⚠️ MEDIUM

### Observation
The world editor writes chunks incrementally. If processing is interrupted (crash, timeout), partial chunks may not be flushed.

### Impact
- **Missing chunks** when placed on server
- **Incomplete generation** (only ground, no buildings)

## Recommended Fixes

### Priority 1: Convert Recursive Water Fill to Iterative
Replace `inverse_floodfill_recursive` with an iterative version using a queue:

```rust
fn inverse_floodfill_iterative(
    min: (i32, i32),
    max: (i32, i32),
    outers: Vec<Vec<XZPoint>>,
    inners: Vec<Vec<XZPoint>>,
    editor: &mut WorldEditor,
    start_time: Instant,
) {
    let mut work_queue = VecDeque::new();
    work_queue.push_back((min, max, outers, inners));
    
    while let Some((min, max, outers, inners)) = work_queue.pop_front() {
        // Check timeout and abort if needed
        if start_time.elapsed().as_secs() > 25 {
            eprintln!("Water area generation timed out, aborting");
            return;
        }
        
        // ... subdivision logic ...
        // Instead of recursive call, push to queue:
        work_queue.push_back((min_x, min_z), (max_x, max_z), outers_intersects, inners_intersects));
    }
}
```

### Priority 2: Optimize Loop Merging
Remove cloning and use move semantics or indices:

```rust
fn merge_loopy_loops(loops: &mut Vec<Vec<ProcessedNode>>) {
    let mut changed = true;
    let mut iteration = 0;
    const MAX_ITERATIONS: usize = 100;  // Safety limit
    
    while changed && iteration < MAX_ITERATIONS {
        changed = false;
        iteration += 1;
        
        // Use indices instead of cloning
        // Merge in-place where possible
        // ...
    }
}
```

### Priority 3: Fix Water Elevation
Use actual terrain elevation instead of hardcoded ground_level:

```rust
let water_level = editor.get_ground_level_at(x, z);
editor.set_block(WATER, x, water_level, z, None, None);
```

### Priority 4: Add Proper Timeout Handling
Actually abort processing when timeout is reached:

```rust
if start_time.elapsed().as_secs() > 25 {
    eprintln!("ERROR: Water area generation timed out, aborting to prevent hang");
    return;  // Actually exit instead of continuing
}
```

### Priority 5: Ensure Chunk Flush on Error
Add proper error handling with chunk flushing in `world_editor.rs`.

## Performance Impact Estimates

| Issue | Impact on Large Areas | Severity |
|-------|----------------------|----------|
| Recursive water fill | 100x stack usage | CRITICAL |
| Loop merging clones | 10x memory usage | HIGH |
| Unbounded recursion | Stack overflow | MEDIUM |
| No timeout abort | Infinite hang | MEDIUM |
| Floating water | Visual glitches | LOW |
| Missing chunks | Data loss | MEDIUM |

## Validation

These issues align with reported problems:

✅ **StackBufferOverflow** - Caused by recursive water fill
✅ **Crashes** - Stack overflow or out of memory
✅ **Infinite waiting** - Deep recursion or no timeout abort  
✅ **Blank worlds** - Crash during processing
✅ **Floating water** - Ground level mismatch
✅ **Missing chunks** - Incomplete flush on error

## Next Steps

1. Implement iterative water fill algorithm
2. Optimize loop merging to avoid cloning
3. Add proper timeout with abort
4. Fix water elevation to match terrain
5. Add comprehensive error handling with chunk flush
6. Add integration tests for large areas

## Testing Recommendations

1. Test with progressively larger areas: 1km² → 10km² → 100km²
2. Test with complex water geometries (U-shapes, nested islands)
3. Monitor stack usage and memory consumption
4. Test timeout behavior
5. Verify chunk completeness after generation
