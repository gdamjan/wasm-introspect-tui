use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render_imports(f: &mut Frame, area: Rect, app: &App) {
    if app.wasm_info.is_component {
        render_component_imports(f, area, app);
    } else {
        render_core_imports(f, area, app);
    }
}

fn render_core_imports(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<Row> = app.wasm_info.imports.iter().enumerate().map(|(i, imp)| {
        let sig_str = imp.signature.as_ref()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let style = if i == app.import_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
        Row::new(vec![
            Cell::from(imp.module.clone()),
            Cell::from(imp.name.clone()),
            Cell::from(imp.kind.to_string()),
            Cell::from(sig_str),
        ]).style(style)
    }).collect();

    let widths = [
        Constraint::Percentage(20),
        Constraint::Percentage(30),
        Constraint::Percentage(10),
        Constraint::Percentage(40),
    ];

    let table = Table::new(items, widths)
        .header(Row::new(vec!["Module", "Name", "Kind", "Signature"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .bottom_margin(1))
        .block(Block::default().borders(Borders::ALL).title("Imports"));

    f.render_widget(table, area);
}

fn render_component_imports(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<Row> = app.wasm_info.component_imports.iter().enumerate().map(|(i, imp)| {
        let style = if i == app.import_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
        Row::new(vec![
            Cell::from(imp.name.clone()),
            Cell::from(imp.kind.clone()),
        ]).style(style)
    }).collect();

    let widths = [
        Constraint::Percentage(70),
        Constraint::Percentage(30),
    ];

    let table = Table::new(items, widths)
        .header(Row::new(vec!["Name", "Kind"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .bottom_margin(1))
        .block(Block::default().borders(Borders::ALL).title("Component Imports"));

    f.render_widget(table, area);
}
