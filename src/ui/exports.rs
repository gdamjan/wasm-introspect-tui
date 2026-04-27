use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render_exports(f: &mut Frame, area: Rect, app: &App) {
    if app.wasm_info.is_component {
        render_component_exports(f, area, app);
    } else {
        render_core_exports(f, area, app);
    }
}

fn render_core_exports(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<Row> = app.wasm_info.exports.iter().enumerate().map(|(i, exp)| {
        let sig_str = exp.signature.as_ref()
            .map(|s| s.to_string())
            .unwrap_or_default();
        let style = if i == app.export_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
        Row::new(vec![
            Cell::from(exp.name.clone()),
            Cell::from(exp.kind.to_string()),
            Cell::from(format!("{}", exp.index)),
            Cell::from(sig_str),
        ]).style(style)
    }).collect();

    let widths = [
        Constraint::Percentage(30),
        Constraint::Percentage(10),
        Constraint::Percentage(10),
        Constraint::Percentage(50),
    ];

    let table = Table::new(items, widths)
        .header(Row::new(vec!["Name", "Kind", "Index", "Signature"])
            .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
            .bottom_margin(1))
        .block(Block::default().borders(Borders::ALL).title("Exports  [Enter: invoke function]"));

    f.render_widget(table, area);
}

fn render_component_exports(f: &mut Frame, area: Rect, app: &App) {
    let items: Vec<Row> = app.wasm_info.component_exports.iter().enumerate().map(|(i, exp)| {
        let style = if i == app.export_selected {
            Style::default().fg(Color::Black).bg(Color::Cyan)
        } else {
            Style::default()
        };
        Row::new(vec![
            Cell::from(exp.name.clone()),
            Cell::from(exp.kind.clone()),
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
        .block(Block::default().borders(Borders::ALL).title("Component Exports"));

    f.render_widget(table, area);
}
