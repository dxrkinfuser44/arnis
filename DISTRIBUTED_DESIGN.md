# Distributed Resource Pooling Design

## Overview
This document describes the design for distributed resource pooling in Arnis, enabling multiple computers with different specs and operating systems to work together on generating large Minecraft worlds.

## Goals
1. **Heterogeneous Support**: Work across different OS (Windows, Linux, macOS) and hardware specs
2. **Scalability**: Efficiently distribute work across multiple nodes
3. **Simplicity**: Easy to set up and use
4. **Fault Tolerance**: Handle node failures gracefully
5. **No Dependencies**: Minimal external dependencies beyond what Arnis already uses

## Architecture

### Design Choice: Coordinator-Worker Pattern
We'll use a coordinator-worker pattern rather than P2P for simplicity:
- **Coordinator**: One machine that:
  - Splits work into chunks
  - Distributes work to workers
  - Collects and merges results
  - Tracks progress
- **Workers**: Multiple machines that:
  - Receive work units
  - Process chunks
  - Return results to coordinator

### Communication Protocol: HTTP/JSON
Use HTTP with JSON for cross-platform compatibility:
- Built on `reqwest` (already a dependency)
- Simple REST-style API
- Works through firewalls/NAT with port forwarding
- Human-readable for debugging

## Work Distribution Strategy

### Geographic Chunking
Divide the world by geographic regions:

1. **Split bounding box** into rectangular chunks
2. **Chunk size**: Configurable (default: 0.01° x 0.01° ~1km²)
3. **Overlap**: Small overlap between chunks to handle edge cases
4. **Priority**: Process chunks in order (no dependencies)

Example:
```
Input: bbox 40.0,-74.0,40.1,-73.9 with scale 1.0
Output: 10x10 = 100 chunks, each 0.01° x 0.01°
```

### Work Unit Structure
```json
{
  "chunk_id": "chunk_0_0",
  "bbox": {
    "min_lat": 40.0,
    "min_lng": -74.0,
    "max_lat": 40.01,
    "max_lng": -73.99
  },
  "settings": {
    "scale": 1.0,
    "terrain": true,
    "interior": true,
    "ground_level": -62
  }
}
```

## API Design

### Coordinator API

#### POST /coordinator/register
Register a worker node
```json
Request:
{
  "worker_id": "worker-abc123",
  "capabilities": {
    "os": "linux",
    "cpu_cores": 8,
    "memory_gb": 16
  }
}

Response:
{
  "status": "registered",
  "coordinator_id": "coord-xyz789"
}
```

#### GET /coordinator/work?worker_id=...
Request work from coordinator
```json
Response:
{
  "work_unit": {
    "chunk_id": "chunk_0_0",
    "bbox": {...},
    "settings": {...},
    "osm_data_url": "http://coordinator:8080/data/chunk_0_0.json"
  }
}
```

#### POST /coordinator/result
Submit completed work
```json
Request:
{
  "worker_id": "worker-abc123",
  "chunk_id": "chunk_0_0",
  "status": "completed",
  "result_url": "http://worker:8081/results/chunk_0_0.mca"
}

Response:
{
  "status": "accepted",
  "next_work": {...}
}
```

#### GET /coordinator/status
Get overall progress
```json
Response:
{
  "total_chunks": 100,
  "completed": 45,
  "in_progress": 5,
  "pending": 50,
  "failed": 0,
  "workers": {
    "active": 3,
    "idle": 1
  }
}
```

### Worker API

#### GET /worker/results/:chunk_id
Download result from worker
```
Response: Binary .mca region file
```

## Implementation Phases

### Phase 3.1: Core Infrastructure
- [x] Design document
- [ ] Create `distributed` module
- [ ] Implement work splitting logic
- [ ] Create coordinator HTTP server
- [ ] Create worker HTTP client
- [ ] Basic work queue management

### Phase 3.2: Data Management
- [ ] Extend asset cache for chunk-level caching
- [ ] OSM data splitting per chunk
- [ ] Result file transfer mechanism
- [ ] Region file merging logic

### Phase 3.3: Worker Implementation  
- [ ] Worker CLI mode
- [ ] Work processing loop
- [ ] Result upload
- [ ] Error handling and retry

### Phase 3.4: Coordinator Implementation
- [ ] Coordinator CLI mode
- [ ] Progress tracking UI
- [ ] Result collection and merging
- [ ] Final world assembly

### Phase 3.5: Polish
- [ ] Fault tolerance (worker disconnection)
- [ ] Load balancing (capability-based)
- [ ] Documentation and examples
- [ ] Testing with multiple nodes

## File Structure

```
crates/arnis-core/src/distributed/
├── mod.rs              # Module exports
├── coordinator.rs      # Coordinator logic
├── worker.rs           # Worker logic
├── work_unit.rs        # Work unit structures
├── chunking.rs         # Geographic chunking logic
├── merge.rs            # Result merging
└── protocol.rs         # HTTP API types
```

## Usage Examples

### Start Coordinator
```bash
# On the coordinator machine
arnis-coordinator \
  --bbox="40.0,-74.0,40.1,-73.9" \
  --output="output_world" \
  --chunk-size=0.01 \
  --port=8080
```

### Start Worker
```bash
# On worker machines
arnis-worker \
  --coordinator="http://192.168.1.100:8080" \
  --worker-id="worker1" \
  --port=8081
```

### Alternative: Single Command
```bash
# Coordinator automatically starts workers on remote machines via SSH
arnis-distributed \
  --bbox="40.0,-74.0,40.1,-73.9" \
  --output="output_world" \
  --workers="user@192.168.1.101,user@192.168.1.102" \
  --chunk-size=0.01
```

## Technical Considerations

### Chunk Size Selection
- **Too small**: Overhead from coordination, many API calls
- **Too large**: Poor load balancing, workers idle
- **Sweet spot**: 1-5 minutes processing time per chunk
- **Default**: 0.01° x 0.01° (~1km² at mid-latitudes)

### Memory Management
- Workers process one chunk at a time
- Coordinator holds all chunks in memory (metadata only)
- Results streamed to disk immediately

### Error Handling
- Worker failures: Reassign chunk to different worker
- Network errors: Retry with exponential backoff
- Coordinator failure: Workers save partial results locally
- Invalid results: Validation and rejection

### Security Considerations
- **Local network only**: Not designed for internet use
- **Optional auth**: Simple token-based authentication
- **No encryption**: Use VPN for secure communication
- **Trust model**: Workers trusted (can modify world data)

### Performance Optimization
- **Parallel downloads**: Workers download OSM data in parallel
- **Async I/O**: Non-blocking HTTP operations
- **Chunk prefetch**: Download next chunk while processing current
- **Result streaming**: Upload results while processing next chunk

## Future Enhancements

### Version 2 Features
- **Dynamic chunking**: Adaptive chunk size based on complexity
- **Checkpoint/resume**: Save coordinator state for restart
- **Worker pools**: Multiple workers per machine
- **GPU acceleration**: Delegate terrain generation to GPU
- **Cloud workers**: Support for cloud VMs (AWS, GCP, Azure)
- **Web UI**: Browser-based progress monitoring
- **Docker support**: Containerized workers

### Advanced Features
- **Elevation data distribution**: Cache and distribute elevation tiles
- **Differential updates**: Re-generate only changed areas
- **Multi-level merging**: Hierarchical merge for very large worlds
- **Quality validation**: Automated checks for chunk quality
- **Benchmark mode**: Performance testing across workers

## Alternatives Considered

### P2P Architecture
**Rejected** because:
- More complex implementation
- Harder to debug
- No clear benefits for this use case
- NAT traversal issues

### Message Queue (RabbitMQ, Redis)
**Rejected** because:
- Additional dependencies
- Overkill for this use case
- HTTP is simpler and sufficient

### gRPC
**Rejected** because:
- More complex than HTTP/JSON
- Binary protocol harder to debug
- HTTP is more accessible

### File-based Coordination
**Rejected** because:
- Requires shared filesystem (NFS, SMB)
- Not cross-platform friendly
- Locking issues

## Dependencies

New dependencies needed:
- `warp` or `axum`: HTTP server framework (lightweight)
- `tokio`: Async runtime (may already be in tauri deps)
- No new dependencies for client (reqwest exists)

## Timeline Estimate

- Phase 3.1: 2-3 hours
- Phase 3.2: 3-4 hours
- Phase 3.3: 2-3 hours
- Phase 3.4: 3-4 hours
- Phase 3.5: 2-3 hours
- **Total**: 12-17 hours

## Success Criteria

1. ✅ Coordinator splits work into chunks
2. ✅ Worker receives and processes chunks
3. ✅ Results are merged into single world
4. ✅ System handles 3+ workers
5. ✅ Cross-platform (Linux + Windows or macOS)
6. ✅ Performance scales with worker count
7. ✅ Documentation and examples
8. ✅ Graceful error handling

## Open Questions

1. **Chunk overlap**: How much overlap between chunks? (Proposed: 10 blocks)
2. **Retry policy**: How many retries before giving up? (Proposed: 3)
3. **Timeout**: How long before declaring worker dead? (Proposed: 5 minutes)
4. **Port range**: What ports to use? (Proposed: 8080-8089)
5. **Authentication**: Required or optional? (Proposed: Optional with --auth-token)

## Conclusion

This design provides a solid foundation for distributed processing while maintaining simplicity and cross-platform compatibility. The coordinator-worker pattern is well-understood and proven, and HTTP/JSON is universally supported.

The implementation will be done incrementally, with each phase building on the previous one, allowing for testing and refinement at each stage.
