# Context Update Template

Use this template when updating `context.md` at the end of your session.

## Instructions

1. Copy the relevant sections below
2. Fill in the details of your session
3. Update the corresponding sections in `context.md`
4. Ensure the "Last Updated" date is current

---

## Last Updated
YYYY-MM-DD

## Recent Changes
- [Brief description of change 1]
- [Brief description of change 2]
- [Brief description of change 3]

## Known Issues
### [Issue Title]
- **Description**: [What is the issue?]
- **Impact**: [How does it affect the project?]
- **Workaround**: [Is there a temporary solution?]
- **Status**: [Open/In Progress/Resolved]

## Performance Considerations
- [Any performance impacts or improvements noted]
- [Memory usage changes]
- [Build time changes]

## Recommendations for Future Work
- [ ] [Recommendation 1]
- [ ] [Recommendation 2]
- [ ] [Recommendation 3]

---

## Tips for Good Context Updates

### Recent Changes
- Be concise but specific
- Focus on user-visible or architectural changes
- Mention which files/modules were affected
- Use past tense ("Added", "Fixed", "Refactored")

### Known Issues
- Only document issues that future agents should know about
- Include workarounds if available
- Remove resolved issues from the main list (can archive in a separate section)
- Link to GitHub issues if applicable

### Performance Considerations
- Note any performance testing you did
- Mention if you enabled/disabled features
- Document before/after metrics if available
- Highlight critical paths that were optimized

### Recommendations
- Prioritize recommendations by importance
- Be specific about what needs to be done
- Link to related documentation or code
- Mark as done when completed by another session

---

## Example Update

```markdown
### Last Updated
2025-11-17

### Recent Changes
- Refactored `element_processing/buildings.rs` to reduce memory usage by 15%
- Added support for multi-story building interior generation
- Fixed coordinate transformation bug affecting large-scale maps
- Updated dependencies: geo 0.30.0, rayon 1.10.0

### Known Issues
#### Memory Spike During Large Area Processing
- **Description**: Processing areas >10km² causes memory usage to spike above 8GB
- **Impact**: May cause OOM on systems with <16GB RAM
- **Workaround**: Split large areas into smaller chunks using multiple bbox calls
- **Status**: In Progress - investigating streaming approach

### Performance Considerations
- Building interior generation now 25% faster due to reduced allocations
- Memory usage reduced by ~15% for typical city generation
- Build time unchanged at ~2min for release builds

### Recommendations for Future Work
- [ ] Implement streaming OSM data parser to reduce peak memory usage
- [ ] Add progress checkpointing for resumable long-running generations
- [ ] Consider implementing GPU acceleration for height map generation
```
