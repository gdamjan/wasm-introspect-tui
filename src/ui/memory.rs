use ratatui::prelude::*;
use ratatui::widgets::*;

use crate::app::App;

pub fn render_memory(f: &mut Frame, area: Rect, app: &mut App) {
    let inner_height = area.height.saturating_sub(3) as usize; // borders + header
    let bytes_per_row = 16;
    let display_bytes = inner_height * bytes_per_row;

    let (data, mem_size) = if let Some(runtime) = &mut app.runtime {
        let size = runtime.memory_size().unwrap_or(0);
        let data = runtime.read_memory(app.memory_offset, display_bytes).unwrap_or_default();
        (data, size)
    } else {
        (Vec::new(), 0)
    };

    let mut lines: Vec<Line> = Vec::new();

    // Header line
    let mut header_spans = vec![
        Span::styled(
            format!("{:>8}  ", "Offset"),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ),
    ];
    for i in 0..bytes_per_row {
        header_spans.push(Span::styled(
            format!("{:02X} ", i),
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        ));
    }
    header_spans.push(Span::styled(
        " ASCII",
        Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
    ));
    lines.push(Line::from(header_spans));

    for row in 0..inner_height {
        let row_offset = app.memory_offset + row * bytes_per_row;
        if row_offset >= mem_size {
            break;
        }

        let mut spans = Vec::new();

        // Address
        spans.push(Span::styled(
            format!("{:08X}  ", row_offset),
            Style::default().fg(Color::DarkGray),
        ));

        let row_start = row * bytes_per_row;
        let row_end = (row_start + bytes_per_row).min(data.len());
        let row_data = if row_start < data.len() { &data[row_start..row_end] } else { &[] };

        // Hex bytes
        for i in 0..bytes_per_row {
            if i < row_data.len() {
                let byte = row_data[i];
                let color = if byte == 0 { Color::DarkGray } else { Color::White };
                spans.push(Span::styled(
                    format!("{:02X} ", byte),
                    Style::default().fg(color),
                ));
            } else {
                spans.push(Span::raw("   "));
            }
        }

        // ASCII
        spans.push(Span::raw(" "));
        let ascii: String = row_data.iter().map(|&b| {
            if b.is_ascii_graphic() || b == b' ' { b as char } else { '.' }
        }).collect();
        spans.push(Span::styled(ascii, Style::default().fg(Color::Green)));

        lines.push(Line::from(spans));
    }

    let title = format!(
        "Memory  [offset: 0x{:08X}, size: {} bytes, ↑/↓: scroll]",
        app.memory_offset, mem_size
    );

    let paragraph = Paragraph::new(lines)
        .block(Block::default().borders(Borders::ALL).title(title));

    f.render_widget(paragraph, area);
}
