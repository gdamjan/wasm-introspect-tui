use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render_disassembly(f: &mut Frame, area: Rect, app: &App) {
    let inner_height = area.height.saturating_sub(2) as usize;

    let end = (app.wat_scroll + inner_height).min(app.wat_lines.len());
    let visible = if app.wat_scroll < app.wat_lines.len() {
        &app.wat_lines[app.wat_scroll..end]
    } else {
        &[]
    };

    let lines: Vec<Line> = visible.iter().enumerate().map(|(i, line)| {
        let line_num = app.wat_scroll + i + 1;
        let mut spans = vec![
            Span::styled(
                format!("{:>5} │ ", line_num),
                Style::default().fg(Color::DarkGray),
            ),
        ];
        spans.push(syntax_highlight(line));
        Line::from(spans)
    }).collect();

    let title = format!(
        "WAT Disassembly  [{}/{} lines, ↑↓/PgUp/PgDn: scroll]",
        app.wat_scroll + 1,
        app.wat_lines.len(),
    );

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(paragraph, area);
}

fn syntax_highlight(line: &str) -> Span<'static> {
    let trimmed = line.trim_start();
    let color = if trimmed.starts_with("(module") || trimmed.starts_with("(component") {
        Color::Cyan
    } else if trimmed.starts_with("(func") || trimmed.starts_with("(export") || trimmed.starts_with("(import") {
        Color::Yellow
    } else if trimmed.starts_with("(memory") || trimmed.starts_with("(table") || trimmed.starts_with("(global") || trimmed.starts_with("(data") {
        Color::Green
    } else if trimmed.starts_with("(type") || trimmed.starts_with("(param") || trimmed.starts_with("(result") {
        Color::Magenta
    } else if trimmed.starts_with(";;") {
        Color::DarkGray
    } else {
        Color::White
    };
    Span::styled(line.to_string(), Style::default().fg(color))
}
