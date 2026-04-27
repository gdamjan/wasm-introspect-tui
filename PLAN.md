# wasm-introspect-tui — Implementation Plan

## Problem
Build a Rust TUI application that introspects WebAssembly binary files (with Component Model/WIT support), lists WIT imports and exports, allows calling exported functions interactively, and displays memory contents.

## Tech Stack
- **Language:** Rust
- **Wasm Runtime:** wasmtime 44 (with component-model feature)
- **TUI Framework:** ratatui 0.30 + crossterm 0.29
- **CLI Args:** clap
- **Wasm Parsing:** wasmparser 0.247

## Architecture
```
src/
  main.rs          — CLI entry point (clap), loads wasm file, launches TUI
  app.rs           — App state, event loop, key handling
  wasm/
    mod.rs         — Module re-exports
    inspector.rs   — Parse wasm binary, extract imports/exports/memory info
    runtime.rs     — Instantiate module, call functions, read memory
  ui/
    mod.rs         — Module re-exports
    layout.rs      — Main layout (panels/tabs)
    imports.rs     — Render imports list
    exports.rs     — Render exports list
    memory.rs      — Hex viewer for memory
    invoke.rs      — Function call dialog (arg input + result display)
```

## Completed

1. ✅ **project-setup** — Cargo project with dependencies (wasmtime, ratatui, crossterm, clap, anyhow, wasmparser)
2. ✅ **wasm-inspector** — Parse wasm binary to extract imports, exports, and memory sections. Supports both core modules and component model.
3. ✅ **wasm-runtime** — Instantiate wasm module with wasmtime, call exported functions with parsed args, read linear memory. Stub imports are auto-provided.
4. ✅ **tui-layout** — Tabbed panels (Imports/Exports/Memory) with tab bar, content area, and status bar. Navigation with Tab/Shift-Tab/arrow keys.
5. ✅ **imports-view** — Table view for imports (module, name, kind, signature). Supports both core and component model.
6. ✅ **exports-view** — Selectable table of exports. Press Enter on a function to invoke it.
7. ✅ **invoke-dialog** — Modal dialog showing function signature, input fields for each parameter (Tab to switch), Enter to call, displays result or error.
8. ✅ **memory-view** — Hex dump viewer with address column, hex bytes, ASCII sidebar, scrolling (↑↓/PageUp/PageDown/Home).
9. ✅ **integration-test** — Smoke tested with a .wasm file containing imports, exports, memory, and data sections.

## Future Ideas
- Component Model function invocation support
- Search/filter within imports and exports lists
- Disassembly view (WAT output)
- Export memory contents to file
- Multiple memory support
- Custom import implementations (beyond stubs)

## Notes
- For Component Model WIT introspection, `wasmparser` decodes component import/export sections directly.
- For core modules, `wasmparser` TypeSection + ImportSection + ExportSection give full metadata with signatures.
- The runtime provides stub implementations for all imports so modules can be instantiated without a host environment.
- Memory viewer reads raw bytes from the linear memory and renders as hex + ASCII.
- Function invocation: string inputs are parsed to the appropriate wasm types (i32/i64/f32/f64) based on the function signature.
