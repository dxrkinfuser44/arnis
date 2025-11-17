# Context Management System

This repository uses a context management system to maintain continuity across coding agent sessions.

## Quick Start for Agents

### Starting a Session
1. **Read `context.md`** - Understand current project state
2. **Review `copilot-instructions.md`** - Follow project guidelines
3. Do your work following best practices

### Ending a Session
1. **Update `context.md`** - Document your changes
2. Use `.github/CONTEXT_UPDATE_TEMPLATE.md` as a guide
3. Run validation: `.github/scripts/validate-context.sh`

## Files Overview

### Core Files
- **`context.md`** - Project context and session state (repository root)
- **`copilot-instructions.md`** - Coding guidelines (repository root)

### Supporting Files
- **`.github/agents/`** - Agent configurations
- **`.github/agents/README.md`** - Agent system documentation
- **`.github/CONTEXT_UPDATE_TEMPLATE.md`** - Template for context updates
- **`.github/scripts/validate-context.sh`** - Validation script
- **`.github/workflows/validate-context.yml`** - CI validation

## Benefits

✅ **Continuity** - Each session builds on previous work  
✅ **Knowledge Sharing** - Discoveries are preserved  
✅ **Efficiency** - No repeated investigations  
✅ **Transparency** - Clear record of changes  
✅ **Coordination** - Multiple agents work effectively  

## Validation

Validate context.md before committing:

```bash
./.github/scripts/validate-context.sh
```

The validation checks:
- File exists and is not empty
- Required sections are present
- Date format is valid
- File size is reasonable
- No incomplete TODO markers

## CI/CD Integration

The `validate-context.yml` workflow runs on every PR to ensure:
- `context.md` is properly formatted
- Required sections are present
- Context was updated when code changed (warning only)

## For Maintainers

### Archiving Old Context

When `context.md` grows too large (>50KB), consider archiving old session data:

1. Create `docs/context-archive/`
2. Move old "Recent Changes" to dated files
3. Keep only last 30 days in main `context.md`

### Updating the System

To update the context management system:
1. Update relevant files (context.md, copilot-instructions.md, etc.)
2. Update `.github/agents/README.md` with any new processes
3. Test with `.github/scripts/validate-context.sh`
4. Document changes in this README

## Troubleshooting

### Validation Fails
- Check error messages from validation script
- Ensure all required sections exist
- Verify date format is YYYY-MM-DD
- Fix any TODO/FIXME markers

### Merge Conflicts in context.md
- Prefer more recent information
- Combine "Recent Changes" chronologically
- Update "Last Updated" to current date
- Re-run validation after resolving

### Agent Not Following Context
- Verify agent configuration includes context management
- Check `.github/agents/README.md` for setup guide
- Ensure agent has read access to context.md

## Examples

See `copilot-instructions.md` Appendix for detailed examples of:
- Adding new features
- Performance optimization
- Cross-platform testing
- Context updates

## Questions?

- Review `.github/agents/README.md`
- Check `copilot-instructions.md`
- Open an issue for context system improvements

---

*This system helps maintain project knowledge across agent sessions. Please keep it up to date!*
