# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Nexus Terminal Desktop (星枢终端桌面版) — a cross-platform SSH client built with **Rust + Tauri 2 + Vue 3**. Desktop rewrite of the original Node.js web version, targeting lower resource usage and higher performance. License: GPL-3.0.

Core capabilities: multi-tab SSH with split panes, SFTP file browser with Monaco Editor, AI assistant (OpenAI/Anthropic/Gemini), database management (6 engines), Telnet, local terminal, VNC/RDP, 17 built-in developer tools, TOTP 2FA + WebAuthn passkey, server status monitoring, SSH session suspend/resume, port forwarding, proxy support, audit logging, encrypted backup/restore.

## Architecture

### Monorepo Layout

```
apps/desktop/
  src-tauri/          # Tauri 2 Rust backend (entry: main.rs → lib.rs → state.rs)
    src/commands/     # 24 IPC command modules (~170 Tauri invoke handlers)
    src/local_terminal/  # PTY management
    src/status_monitor/  # Server metrics with platform-specific collectors
  frontend/           # Vue 3 + TypeScript SPA (Vite 6, Pinia, Vue Router hash mode)
    src/lib/          # Tauri IPC API layer (20 api-*.ts + invoke.ts wrapper)
    src/stores/       # 23 Pinia stores
    src/composables/  # 16 Vue composables
    src/components/   # 70+ components organized by feature
    src/views/        # Page-level components (Connections, Workspace, Databases, Tools, Settings...)
    src/utils/        # Command autocomplete system (120+ command definitions)

crates/               # 18 focused Rust library crates
  api-contract/       # Shared DTOs and AppError enum
  auth-core/          # Argon2/bcrypt hashing, TOTP 2FA, WebAuthn passkey
  connection-core/    # Connection model + repository trait
  session-core/       # AuthState (NeedsSetup/Locked/Unlocked), SessionRegistry
  ssh-core/           # russh-based SSH client, host key verification
  sftp-core/          # SFTP service layer
  ssh-suspend-core/   # SSH session suspend/resume
  proxy-core/         # SOCKS5/HTTP proxy
  storage-sqlite/     # sqlx SQLite persistence (pool, migrations, 6 repos)
  shared-utils/       # AES-GCM crypto, UUID, path utilities
  tag-core/           # Connection tagging/grouping
  quick-command-core/ # Quick command snippets
  notifications-core/ # Notification system
  audit-core/         # Audit logging
  settings-core/      # App settings model/repository
  history-core/       # Command/path history
  transfer-core/      # File transfer progress tracking
  ws-gateway/         # WebSocket gateway
```

`Netcatty/` is a separate Electron+React reference app — NOT part of the build.

### Backend Pattern

Crates follow **model/repository/service** separation. `AppState` (state.rs) is the central managed state holding all repositories, session managers, crypto service, and an SFTP upload semaphore. Tauri IPC handlers in `commands/` delegate to these services.

Key dependency chain: `lib.rs` registers ~170 IPC handlers → `state.rs` wires AppState → `commands/*.rs` handle IPC → `crates/*` contain business logic → `storage-sqlite` persists to SQLite.

### Frontend Pattern

Vue 3 SPA with hash-mode routing. Auth guard in `main.ts` enforces NeedsSetup → Setup → Login → Authenticated flow. Routes: `/connections`, `/workspace`, `/databases`, `/tools`, `/settings`, etc.

IPC calls go through `src/lib/invoke.ts` (typed wrapper around `@tauri-apps/api/core` invoke with `TauriError` handling) → `src/lib/api-*.ts` domain modules → barrel-exported from `api.ts`.

## Build Commands

```bash
# Frontend
pnpm --dir apps/desktop/frontend install       # install deps
pnpm --dir apps/desktop/frontend build         # type-check + Vite build
pnpm --dir apps/desktop/frontend test          # run Vitest

# Full desktop app (interactive dev)
cd apps/desktop && pnpm tauri dev

# Production build
cd apps/desktop && pnpm tauri build
# Output: apps/desktop/src-tauri/target/release/bundle/

# Rust
cargo test --workspace                         # all Rust tests
cargo fmt --all                                # format
cargo clippy --workspace --all-targets -- -D warnings  # lint (no warnings policy)
```

## Testing

- **Frontend**: Vitest. Test files in `src/**/__tests__/**/*.test.ts`. Currently 2 test files in `src/lib/__tests__/`.
- **Rust**: Inline module tests + crate-level integration tests (e.g. `crates/proxy-core/tests/socks5_test.rs`, `crates/storage-sqlite/tests/command_history_merge_test.rs`).
- Coverage target: >= 80% for core Rust domains, full coverage for P0/P1 contract paths.
- Add tests near the changed module.

## Coding Conventions

- **Rust**: Default `rustfmt` formatting. Keep crates small and single-purpose. Return full `Result` error chains — do not hide failures.
- **TypeScript**: Strict mode, `@/` path alias, single quotes. PascalCase for Vue components. Semantic names for stores/composables. 2-space indentation in template/script.
- **Commits**: Conventional Commits — `feat`, `fix`, `refactor`, `chore` with scopes like `feat(desktop):` or `fix(ui):`.
- **Version**: Managed in `apps/desktop/src-tauri/tauri.conf.json` (currently 1.1.9).

## Key Configuration Files

| File | Purpose |
|------|---------|
| `Cargo.toml` (root) | Workspace definition (19 members), shared dep versions |
| `apps/desktop/src-tauri/Cargo.toml` | Tauri app crate, depends on all 18 crates |
| `apps/desktop/src-tauri/tauri.conf.json` | Tauri config (window, dev URL, bundle settings) |
| `apps/desktop/src-tauri/capabilities/default.json` | Tauri permission declarations |
| `apps/desktop/frontend/vite.config.ts` | Vite config (Vue plugin, `@/` alias, port 5173) |
| `apps/desktop/frontend/vitest.config.ts` | Vitest config (node env, test pattern) |
| `apps/desktop/frontend/tsconfig.json` | TypeScript strict config |
| `docs/DEVELOPMENT_GUIDE.md` | Detailed development guide (18KB) |

## CI/CD

Single workflow: `.github/workflows/release-single-exe.yml` — produces a standalone Windows `.exe` (no installer). Triggered by manual dispatch or version tag push (`*.*.*`). Runs on `windows-latest`, uses `tauri-cli ^2`, uploads to GitHub Release on tag push.

## Important Notes

- `docs/` is the source of truth for architecture, delivery, and release rules. Update docs when APIs, data models, or workflow expectations change.
- Platform-specific code: `odbc-api` (Oracle DB) is Windows-only. Status monitor has per-platform collectors (`linux.rs`, `macos.rs`, `windows.rs`).
- Monaco Editor and xterm.js are pre-bundled via Vite config.
- Window uses custom titlebar (no OS decorations).
