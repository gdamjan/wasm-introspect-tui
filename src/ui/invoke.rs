use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render_invoke_dialog(f: &mut Frame, area: Rect, app: &App) {
    let idx = match app.invoke_export_idx {
        Some(i) => i,
        None => return,
    };

    let export = &app.wasm_info.exports[idx];
    let sig = export.signature.as_ref();

    let dialog_width = 60u16.min(area.width.saturating_sub(4));
    let param_count = sig.map(|s| s.params.len()).unwrap_or(0);
    let dialog_height = (8 + param_count as u16 + 2).min(area.height.saturating_sub(2));

    let x = area.x + (area.width.saturating_sub(dialog_width)) / 2;
    let y = area.y + (area.height.saturating_sub(dialog_height)) / 2;
    let dialog_area = Rect::new(x, y, dialog_width, dialog_height);

    // Clear background
    f.render_widget(Clear, dialog_area);

    let mut lines: Vec<Line> = Vec::new();

    // Function name and signature
    lines.push(Line::from(vec![
        Span::styled("Function: ", Style::default().fg(Color::Yellow)),
        Span::raw(&export.name),
    ]));

    if let Some(sig) = sig {
        lines.push(Line::from(vec![
            Span::styled("Signature: ", Style::default().fg(Color::Yellow)),
            Span::raw(sig.to_string()),
        ]));
        lines.push(Line::raw(""));

        if sig.params.is_empty() {
            lines.push(Line::styled("(no parameters)", Style::default().fg(Color::DarkGray)));
        } else {
            lines.push(Line::styled("Parameters:", Style::default().fg(Color::Cyan)));
            for (i, param) in sig.params.iter().enumerate() {
                let is_active = i == app.invoke_cursor;
                let prefix = if is_active { "▸ " } else { "  " };
                let value = app.invoke_args.get(i).map(|s| s.as_str()).unwrap_or("");
                let style = if is_active {
                    Style::default().fg(Color::White).bg(Color::DarkGray)
                } else {
                    Style::default()
                };
                lines.push(Line::from(vec![
                    Span::raw(prefix),
                    Span::styled(format!("{}: ", param), Style::default().fg(Color::Yellow)),
                    Span::styled(
                        if value.is_empty() { "<empty>" } else { value },
                        style,
                    ),
                ]));
            }
        }
    } else {
        lines.push(Line::styled("(unknown signature)", Style::default().fg(Color::DarkGray)));
    }

    lines.push(Line::raw(""));

    if let Some(result) = &app.invoke_result {
        lines.push(Line::from(vec![
            Span::styled("Result: ", Style::default().fg(Color::Green)),
            Span::raw(result),
        ]));
    }

    if let Some(err) = &app.invoke_error {
        lines.push(Line::from(vec![
            Span::styled("Error: ", Style::default().fg(Color::Red)),
            Span::raw(err),
        ]));
    }

    lines.push(Line::raw(""));
    lines.push(Line::styled(
        "[Tab: next field] [Enter: call] [Esc: close]",
        Style::default().fg(Color::DarkGray),
    ));

    let paragraph = Paragraph::new(lines)
        .block(Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .title("Invoke Function"));

    f.render_widget(paragraph, dialog_area);
}
