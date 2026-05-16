# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

**scx-music** — 本地音乐播放器桌面应用。目标是构建一个功能完整的本地音乐播放 app。

**Tech stack:**
- Frontend: Vue 3 + TypeScript + Vite
- UI library: Vuetify
- Utility hooks: VueUse
- Desktop shell: Tauri v2 (Rust backend)
- Package manager: pnpm

## Commands

```bash
pnpm dev          # Vite dev server only (port 1420)
pnpm build        # Type-check (vue-tsc) then Vite build
pnpm preview      # Preview production build

pnpm app:dev      # Tauri dev mode (starts Vite + Rust backend, hot reload)
pnpm app:build    # Production build (Tauri bundles native app)

# Rust backend only (from src-tauri/)
cd src-tauri && cargo build    # Build Rust backend
cd src-tauri && cargo test     # Run Rust tests
```

## Architecture

This is a Tauri v2 app with a dual-process architecture:

- **`src/`** — Vue 3 frontend (SPA rendered in Tauri's webview)
- **`src-tauri/`** — Rust backend (native window, file system access, system APIs)
- Frontend and backend communicate via Tauri's `invoke()` IPC (see `src-tauri/src/lib.rs` for registered commands)

Tauri config lives at `src-tauri/tauri.conf.json` — controls window size, CSP, build commands, and bundling.

Rust dependencies are in `src-tauri/Cargo.toml`, frontend dependencies in root `package.json`.

## Tauri IPC Convention

Rust commands are defined with `#[tauri::command]` in `src-tauri/src/lib.rs` and registered in `invoke_handler`. Frontend calls them via `invoke("command_name", { args })` from `@tauri-apps/api/core`.

## IDE Setup

VS Code with: Vue - Official, Tauri, rust-analyzer extensions.

## Wiki-Driven Development

This project maintains a `.wiki/` directory as the knowledge base. Two rules apply to every feature task:

1. **Before developing a feature**: read the relevant `.wiki/` pages to understand current architecture and design decisions.
2. **After completing a feature**: update the affected `.wiki/` pages to keep documentation in sync with code.

The wiki index is at `.wiki/Home.md`. Key pages: Architecture.md, Frontend.md, Backend.md, DataModel.md, TechStack.md, Development.md..
