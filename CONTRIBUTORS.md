# Contributors

## Core Team
- nexus-terminal-rust contributors - Project maintainers and primary developers

## Original Work Attribution

### nexus-terminal
- **Original Author**: Heavrnl
- **Repository**: https://github.com/Heavrnl/nexus-terminal
- **License**: GPL-3.0
- **Contribution**: Base architecture, terminal functionality, SFTP implementation, session management

### Design Inspiration
- **Team**: Mshell Team
- **Repository**: https://github.com/inspoaibox/Mshell
- **License**: MIT
- **Contribution**: AI assistant feature concept, workflow design patterns (no source code used)

## Third-Party Libraries

This project uses numerous open-source libraries. For a complete list:

### Rust Dependencies
```bash
cargo license --json
```

### Frontend Dependencies
```bash
cd apps/desktop/frontend && pnpm licenses list
```

### Key Dependencies

**Backend (Rust)**:
- Tauri 2 - Desktop application framework
- russh 0.49 - SSH protocol implementation
- russh-sftp 2.1 - SFTP protocol implementation
- SQLx 0.8 - Database toolkit
- Tokio - Async runtime

**Frontend (JavaScript/TypeScript)**:
- Vue 3.5 - Progressive JavaScript framework
- xterm.js 6.0 - Terminal emulator
- Monaco Editor - Code editor
- Pinia - State management
- Vite 6 - Build tool

## Contributing

Contributions are welcome! Please ensure:
1. All contributions comply with GPL-3.0 license
2. Code follows project conventions
3. Tests pass before submitting PR
4. Commit messages follow conventional commits format

## License

All contributions to this project are licensed under GPL-3.0.
See [LICENSE](LICENSE) for full text.
