# Arnis Documentation

Welcome to the comprehensive documentation for the Arnis Minecraft world generator project!

## 📚 Documentation Overview

This documentation suite provides in-depth technical information about Arnis's architecture, algorithms, and implementation. It's designed for developers, contributors, and AI agents working with the codebase.

## 🌐 How to View

### Option 1: Web Browser (Recommended)
Open `index.html` in your web browser to access the full documentation with interactive navigation:

```bash
# From the project root
cd docs
open index.html  # macOS
xdg-open index.html  # Linux
start index.html  # Windows
```

### Option 2: Direct File Access
Browse individual HTML files directly:
- `index.html` - Documentation homepage with navigation
- `architecture.html` - System architecture and design
- `coordinate-system.html` - Coordinate transformation details
- `element-processing.html` - Element processing and algorithms

## 📖 Documentation Contents

### 1. **Architecture Documentation**
- System design overview
- Component structure
- Data flow pipeline
- Technology stack
- Design principles

**Best for:** Understanding the big picture and how components interact

### 2. **Coordinate System Documentation**
- Geographic (WGS84) coordinates
- Minecraft coordinate system
- Transformation algorithms
- Bounding box operations
- Practical examples

**Best for:** Working with coordinate transformations and spatial calculations

### 3. **Element Processing Documentation**
- All processor modules (buildings, highways, water, etc.)
- Core algorithms (roof generation, flood-fill, etc.)
- Priority system
- Material selection logic
- Step-by-step guides for adding processors

**Best for:** Adding features or modifying element processing logic

## 🎯 Quick Start Guides

### For New Contributors
1. Read the [main README](../README.md) for project overview
2. Open `docs/index.html` in your browser
3. Start with Architecture documentation
4. Review Element Processing for feature details
5. Check [AGENT.md](../AGENT.md) for development guidelines

### For AI Agents
1. Parse [AGENT.md](../AGENT.md) for system context
2. Read [AGENT_SKILLS.md](../AGENT_SKILLS.md) for skills taxonomy
3. Process HTML documentation for technical details
4. Use code examples as templates
5. Follow skill development progression

### For Developers
1. Open `index.html` for navigation
2. Browse relevant technical sections
3. Study code examples
4. Follow best practices outlined
5. Refer back when implementing features

## 🏗️ Documentation Structure

```
docs/
├── README.md                    # This file
├── index.html                   # Homepage with navigation
├── architecture.html            # System architecture (797 lines)
├── coordinate-system.html       # Coordinates & transformations (931 lines)
└── element-processing.html      # Processing details (1,124 lines)
```

## 🎨 Features

- **Modern Design:** Clean, professional styling with gradient themes
- **Responsive Layout:** Works on desktop and mobile devices
- **Interactive Navigation:** Easy access to all documentation sections
- **Code Examples:** Syntax-highlighted Rust code throughout
- **Visual Diagrams:** Component cards, flow diagrams, and tables
- **Search-Friendly:** Well-structured HTML with semantic markup

## 📝 Related Documentation

- **[README.md](../README.md)** - Project overview and quick start
- **[AGENT.md](../AGENT.md)** - AI agent overview and guidelines
- **[AGENT_SKILLS.md](../AGENT_SKILLS.md)** - Detailed skills reference
- **[ONBOARDING_SUMMARY.md](../ONBOARDING_SUMMARY.md)** - Project status and analysis
- **[GitHub Wiki](https://github.com/louis-e/arnis/wiki)** - User-focused guides

## 🔧 Technical Details

### Technology Used
- **HTML5** - Semantic markup
- **CSS3** - Modern styling with gradients and animations
- **Responsive Design** - Mobile-friendly layouts
- **No JavaScript Required** - Pure HTML/CSS documentation

### Browser Compatibility
- Chrome/Edge (latest)
- Firefox (latest)
- Safari (latest)
- Opera (latest)

## 🤝 Contributing to Documentation

Documentation contributions are welcome! When updating:

1. **Maintain Style Consistency:** Follow the existing HTML/CSS patterns
2. **Update All References:** If changing structure, update navigation
3. **Test in Multiple Browsers:** Ensure compatibility
4. **Keep Examples Current:** Update code examples when codebase changes
5. **Add to This README:** Document new sections added

### Documentation Standards
- Use semantic HTML5 elements
- Include code examples with syntax highlighting
- Add diagrams where helpful
- Write clear, concise explanations
- Link between related sections
- Keep navigation up to date

## 📊 Documentation Statistics

- **Total Lines:** ~3,500+ lines of documentation
- **HTML Pages:** 4 main pages
- **Code Examples:** 50+ Rust code snippets
- **Diagrams:** Multiple flow and component diagrams
- **Coverage:** All major system components documented

## 🎓 Learning Path

Recommended reading order for newcomers:

1. **Start Here:** `index.html` - Get oriented
2. **Big Picture:** `architecture.html` - Understand system design
3. **Deep Dive:** `coordinate-system.html` - Learn transformations
4. **Implementation:** `element-processing.html` - See algorithms
5. **Practice:** Use code examples in your own contributions

## 💡 Tips for Using This Documentation

- **Use Browser Search:** Press Ctrl/Cmd+F to find specific topics
- **Bookmark Sections:** Save frequently referenced pages
- **Print if Needed:** HTML prints well for offline reference
- **Cross-Reference:** Use links between documentation files
- **Code Examples:** Copy and adapt for your implementations

## 🐛 Reporting Documentation Issues

Found an error or unclear section? Please:
1. Open an issue on GitHub
2. Specify the file and section
3. Suggest improvements
4. Submit a pull request if possible

## 📜 License

This documentation is part of the Arnis project and is licensed under the Apache-2.0 license.

Copyright © 2022-2025 Louis Erbkamm

## 🔗 External Resources

- **OpenStreetMap Wiki:** https://wiki.openstreetmap.org/
- **Minecraft Wiki:** https://minecraft.wiki/
- **Rust Documentation:** https://doc.rust-lang.org/
- **Tauri Documentation:** https://tauri.app/
- **Anvil Format:** https://minecraft.wiki/w/Anvil_file_format

## ✨ Acknowledgments

- **Documentation Created:** January 2025
- **Author:** AI Agent in collaboration with project maintainers
- **Purpose:** Enable contributors and AI agents to understand and extend Arnis
- **Status:** Production-ready, actively maintained

---

**Happy coding!** 🚀

For questions or feedback, visit the [GitHub repository](https://github.com/louis-e/arnis) or join the [Discord community](https://discord.gg/mA2g69Fhxq).