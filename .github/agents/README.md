# Arnis Coding Agents

This directory contains configuration files for GitHub Copilot coding agents used in the Arnis project.

## Available Agents

### Beast Mode Agent
**File**: `Beast Mode.agent.md`  
**Model**: GPT-4.1  
**Purpose**: Autonomous problem-solving agent with extensive research capabilities

This agent is designed to:
- Solve problems autonomously from start to finish
- Perform extensive internet research when needed
- Read and update project context automatically
- Follow comprehensive testing and validation workflows

## Context Management System

All coding agents in this project follow a context management workflow:

### Session Start
1. **Read `context.md`** (in repository root) to understand:
   - Current project state
   - Recent changes
   - Known issues
   - Active development areas

2. **Review `copilot-instructions.md`** for:
   - Coding guidelines
   - Module-specific patterns
   - Best practices
   - Common tasks

### During Session
- Follow project guidelines
- Make minimal, surgical changes
- Test frequently
- Document decisions

### Session End
**CRITICAL**: Update `context.md` with:
- Summary of changes made
- New issues discovered
- Performance impacts
- Next steps or recommendations
- Updated "Last Updated" timestamp

## Why Context Management?

The context management system provides several benefits:

1. **Continuity**: Each agent session builds on previous work
2. **Knowledge Sharing**: Discoveries and lessons are preserved
3. **Efficiency**: Agents don't repeat mistakes or investigations
4. **Transparency**: Clear record of what was done and why
5. **Coordination**: Multiple agents can work effectively on the same codebase

## Creating New Agents

When creating new agents for this project:

1. Create a new `.agent.md` file in this directory
2. Include the context management workflow in the agent instructions
3. Ensure the agent reads `context.md` at start
4. Ensure the agent updates `context.md` at end
5. Reference `copilot-instructions.md` for project-specific guidelines
6. Update this README to list the new agent

## Template for New Agent Context Management Section

```markdown
## Context Management
**CRITICAL**: Before starting any work, ALWAYS read `context.md` in the repository root to understand:
- Current project state and structure
- Recent changes and active development areas
- Known issues and performance considerations
- Build and test procedures

After completing your work and before ending your session, ALWAYS update `context.md` with:
- Summary of changes made
- New issues discovered
- Performance impacts (if measured)
- Next steps or recommendations
- Updated timestamp in "Last Updated" field

Also review `copilot-instructions.md` for detailed coding guidelines specific to this project.
```

## Best Practices

### For Agent Creators
- Keep agent instructions focused and specific
- Avoid overlapping responsibilities between agents
- Document the agent's purpose and use cases
- Test agents with sample tasks before deployment

### For Agent Users
- Choose the right agent for the task
- Provide clear, specific instructions
- Review agent outputs before merging
- Report issues or improvements to agent configurations

## Troubleshooting

### Agent Not Reading Context
- Verify `context.md` exists in repository root
- Check agent configuration includes context management section
- Review agent logs for file access issues

### Agent Not Updating Context
- Check if agent completed session properly
- Verify agent has write permissions
- Review agent configuration for update instructions

### Conflicting Context Updates
- Use git to resolve merge conflicts in `context.md`
- Prefer more recent information when conflicts occur
- Maintain chronological order in "Recent Changes" section

## Additional Resources

- **Project Context**: `../../context.md`
- **Coding Instructions**: `../../copilot-instructions.md`
- **GitHub Wiki**: https://github.com/louis-e/arnis/wiki/
- **Contributing Guide**: `../../README.md` (Open Source section)

---

*For questions or suggestions about the agent system, please open an issue or discussion in the repository.*
