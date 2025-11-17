# Context System Quick Reference

## 🚀 Quick Start

### Before Starting Work
```bash
# 1. Read the context
cat context.md

# 2. Review guidelines
cat copilot-instructions.md
```

### After Finishing Work
```bash
# 1. Update context.md (see template)
# Edit: Recent Changes, Known Issues, etc.

# 2. Validate your changes
./.github/scripts/validate-context.sh

# 3. Commit
git add context.md
git commit -m "Update context with session changes"
```

## 📋 Common Commands

### Validation
```bash
# Validate context.md
./.github/scripts/validate-context.sh

# Check what changed
git diff context.md
```

### Reading Context
```bash
# View recent changes
sed -n '/### Recent Changes/,/### [A-Z]/p' context.md | head -n -1

# View known issues
sed -n '/### Known Issues/,/### [A-Z]/p' context.md | head -n -1

# Check last update date
grep -A 1 "### Last Updated" context.md | tail -1
```

### Updating Context
```bash
# Use the template
cat .github/CONTEXT_UPDATE_TEMPLATE.md

# Edit context.md
$EDITOR context.md

# Validate before committing
./.github/scripts/validate-context.sh
```

## 📝 Update Template (Copy & Paste)

```markdown
### Last Updated
2025-11-17

### Recent Changes
- [Your change 1]
- [Your change 2]
- [Your change 3]
```

## 🎯 Key Sections to Update

1. **Last Updated** - Current date (YYYY-MM-DD)
2. **Recent Changes** - What you did this session
3. **Known Issues** - New problems found or resolved
4. **Performance Considerations** - Any performance impacts
5. **Active Development Areas** - Update status

## ⚡ Quick Tips

- **Be concise** - Short, clear bullet points
- **Be specific** - Mention file names and modules
- **Use past tense** - "Added", "Fixed", "Refactored"
- **Update date** - Always update "Last Updated"
- **Validate** - Run validation before committing

## 🔍 Finding Information

### Project Structure
```bash
# See all modules
ls -la crates/arnis-core/src/

# View dependencies
cat crates/arnis-core/Cargo.toml
```

### Build & Test
```bash
# Build CLI
cd crates/arnis-cli && cargo build

# Build core library
cd crates/arnis-core && cargo build --no-default-features

# Run tests
cargo test
```

### Agent Configuration
```bash
# View agent setup
cat .github/agents/README.md

# Check Beast Mode agent
cat .github/agents/Beast\ Mode.agent.md
```

## 📖 Full Documentation

- **CONTEXT_SYSTEM.md** - Complete system guide
- **context.md** - Current project state
- **copilot-instructions.md** - Detailed coding guidelines
- **.github/agents/README.md** - Agent system docs

## ⚠️ Common Mistakes

❌ Forgetting to update "Last Updated" date  
❌ Not running validation before committing  
❌ Adding too much detail (keep it concise)  
❌ Leaving TODO markers in context.md  
❌ Not removing resolved issues  

## ✅ Best Practices

✓ Read context before starting  
✓ Update context when done  
✓ Run validation script  
✓ Keep recent changes chronological  
✓ Remove old/resolved issues  
✓ Use template for consistency  

---

**Need help?** See CONTEXT_SYSTEM.md for troubleshooting and detailed guidance.
