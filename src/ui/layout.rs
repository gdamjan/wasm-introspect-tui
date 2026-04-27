use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::{App, Tab, InputMode};
use crate::ui::imports::render_imports;
use crate::ui::exports::render_exports;
use crate::ui::memory::render_memory;
use crate::ui::invoke::render_invoke_dialog;
use crate::ui::disassembly::render_disassembly;

pub fn render(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tab bar
            Constraint::Min(0),   // Content
            Constraint::Length(1), // Status bar
        ])
        .split(f.area());

    render_tabs(f, chunks[0], app);
    render_content(f, chunks[1], app);
    render_status(f, chunks[2], app);

    if app.input_mode == InputMode::Invoke {
        render_invoke_dialog(f, chunks[1], app);
    }
}

fn render_tabs(f: &mut Frame, area: Rect, app: &App) {
    let titles: Vec<Line> = Tab::ALL.iter().map(|t| {
        let style = if *t == app.active_tab {
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        Line::styled(t.title(), style)
    }).collect();

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(format!(" {} ", app.file_path)))
        .highlight_style(Style::default().fg(Color::Cyan))
        .select(Tab::ALL.iter().position(|t| *t == app.active_tab).unwrap_or(0));

    f.render_widget(tabs, area);
}

fn render_content(f: &mut Frame, area: Rect, app: &mut App) {
    match app.active_tab {
        Tab::Imports => render_imports(f, area, app),
        Tab::Exports => render_exports(f, area, app),
        Tab::Memory => render_memory(f, area, app),
        Tab::Disassembly => render_disassembly(f, area, app),
    }
}

fn render_status(f: &mut Frame, area: Rect, app: &App) {
    if let Some(msg) = &app.status_message {
        let paragraph = Paragraph::new(format!(" ⚠ {}", msg))
            .style(Style::default().fg(Color::Yellow).bg(Color::DarkGray));
        f.render_widget(paragraph, area);
        return;
    }

    let mode = if app.wasm_info.is_component { "Component" } else { "Core Module" };
    let imports_count = if app.wasm_info.is_component {
        app.wasm_info.component_imports.len()
    } else {
        app.wasm_info.imports.len()
    };
    let exports_count = if app.wasm_info.is_component {
        app.wasm_info.component_exports.len()
    } else {
        app.wasm_info.exports.len()
    };

    let status = format!(
        " [{}] Imports: {} | Exports: {} | Memories: {} | [q]uit [Tab]switch [↑↓]navigate [Enter]invoke",
        mode, imports_count, exports_count, app.wasm_info.memories.len()
    );

    let paragraph = Paragraph::new(status)
        .style(Style::default().fg(Color::White).bg(Color::DarkGray));

    f.render_widget(paragraph, area);
}
