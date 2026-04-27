mod app;
mod wasm;
mod ui;

use std::time::Duration;
use anyhow::Result;
use clap::Parser;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::DefaultTerminal;

use app::{App, InputMode, Tab};
use wasm::inspector;
use wasm::runtime::WasmRuntime;

#[derive(Parser)]
#[command(name = "wasm-introspect-tui")]
#[command(about = "TUI tool for introspecting WebAssembly binaries")]
struct Cli {
    /// Path to the .wasm file
    file: String,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let wasm_bytes = std::fs::read(&cli.file)?;
    let wasm_info = inspector::inspect(&wasm_bytes)?;

    let runtime = if !wasm_info.is_component {
        match WasmRuntime::new(&wasm_bytes) {
            Ok(rt) => Some(rt),
            Err(e) => {
                eprintln!("Warning: could not instantiate runtime: {:#}", e);
                None
            }
        }
    } else {
        None
    };

    let wat_text = wasmprinter::print_bytes(&wasm_bytes).unwrap_or_else(|e| format!(";; Error: {}", e));
    let wat_lines: Vec<String> = wat_text.lines().map(String::from).collect();

    let mut app = App::new(wasm_info, runtime, cli.file, wat_lines);

    let mut terminal = ratatui::init();
    let res = run_app(&mut terminal, &mut app);
    ratatui::restore();

    if let Err(err) = res {
        eprintln!("Error: {:#}", err);
    }

    Ok(())
}

fn run_app(terminal: &mut DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        terminal.draw(|f| ui::layout::render(f, app))?;

        if event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match app.input_mode {
                        InputMode::Normal => handle_normal_input(app, key.code, key.modifiers),
                        InputMode::Invoke => handle_invoke_input(app, key.code),
                    }
                }
            }
        }

        if app.should_quit {
            return Ok(());
        }
    }
}

fn handle_normal_input(app: &mut App, key: KeyCode, modifiers: KeyModifiers) {
    app.status_message = None;
    match key {
        KeyCode::Char('q') => app.should_quit = true,
        KeyCode::Char('c') if modifiers.contains(KeyModifiers::CONTROL) => app.should_quit = true,
        KeyCode::Tab => app.active_tab = app.active_tab.next(),
        KeyCode::BackTab => app.active_tab = app.active_tab.prev(),

        KeyCode::Up => match app.active_tab {
            Tab::Imports => {
                if app.import_selected > 0 {
                    app.import_selected -= 1;
                }
            }
            Tab::Exports => {
                if app.export_selected > 0 {
                    app.export_selected -= 1;
                }
            }
            Tab::Memory => {
                if app.memory_offset >= 16 {
                    app.memory_offset -= 16;
                }
            }
            Tab::Disassembly => {
                if app.wat_scroll > 0 {
                    app.wat_scroll -= 1;
                }
            }
        },
        KeyCode::Down => match app.active_tab {
            Tab::Imports => {
                let max = app.max_imports();
                if max > 0 && app.import_selected < max - 1 {
                    app.import_selected += 1;
                }
            }
            Tab::Exports => {
                let max = app.max_exports();
                if max > 0 && app.export_selected < max - 1 {
                    app.export_selected += 1;
                }
            }
            Tab::Memory => {
                app.memory_offset += 16;
            }
            Tab::Disassembly => {
                if app.wat_scroll + 1 < app.wat_lines.len() {
                    app.wat_scroll += 1;
                }
            }
        },
        KeyCode::PageUp => {
            match app.active_tab {
                Tab::Memory if app.memory_offset >= app.memory_page_size => {
                    app.memory_offset -= app.memory_page_size;
                }
                Tab::Memory => {
                    app.memory_offset = 0;
                }
                Tab::Disassembly => {
                    app.wat_scroll = app.wat_scroll.saturating_sub(30);
                }
                _ => {}
            }
        }
        KeyCode::PageDown => {
            match app.active_tab {
                Tab::Memory => {
                    app.memory_offset += app.memory_page_size;
                }
                Tab::Disassembly => {
                    app.wat_scroll = (app.wat_scroll + 30).min(app.wat_lines.len().saturating_sub(1));
                }
                _ => {}
            }
        }
        KeyCode::Home => {
            match app.active_tab {
                Tab::Memory => app.memory_offset = 0,
                Tab::Disassembly => app.wat_scroll = 0,
                _ => {}
            }
        }
        KeyCode::End => {
            if app.active_tab == Tab::Disassembly {
                app.wat_scroll = app.wat_lines.len().saturating_sub(1);
            }
        }

        KeyCode::Enter => {
            if app.active_tab == Tab::Exports {
                app.open_invoke_dialog();
            }
        }

        _ => {}
    }
}

fn handle_invoke_input(app: &mut App, key: KeyCode) {
    match key {
        KeyCode::Esc => app.close_invoke_dialog(),
        KeyCode::Tab => {
            if !app.invoke_args.is_empty() {
                app.invoke_cursor = (app.invoke_cursor + 1) % app.invoke_args.len();
            }
        }
        KeyCode::Enter => app.execute_invoke(),
        KeyCode::Char(c) => {
            if let Some(arg) = app.invoke_args.get_mut(app.invoke_cursor) {
                arg.push(c);
            }
        }
        KeyCode::Backspace => {
            if let Some(arg) = app.invoke_args.get_mut(app.invoke_cursor) {
                arg.pop();
            }
        }
        _ => {}
    }
}
