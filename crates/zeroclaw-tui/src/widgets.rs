use ratatui::{
    buffer::Buffer,
    layout::Rect,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Paragraph, Widget, Wrap},
};

use super::theme;

/// Bordered info panel — title bar + wrapped body content.
pub struct InfoPanel<'a> {
    pub title: &'a str,
    pub lines: Vec<Line<'a>>,
}

impl Widget for InfoPanel<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_style(theme::border_style())
            .title(Span::styled(
                format!(" {} ", self.title),
                theme::heading_style(),
            ));

        let inner = block.inner(area);
        block.render(area, buf);

        Paragraph::new(Text::from(self.lines))
            .wrap(Wrap { trim: false })
            .style(theme::body_style())
            .render(inner, buf);
    }
}

/// Single-line prompt with a label and the current input buffer. Masks the
/// input when `masked` is true (for secrets).
pub struct InputPrompt<'a> {
    pub label: &'a str,
    pub input: &'a str,
    pub masked: bool,
}

impl Widget for InputPrompt<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let display = if self.masked {
            "\u{2022}".repeat(self.input.len())
        } else {
            self.input.to_string()
        };

        let line = Line::from(vec![
            Span::styled("\u{25c6}  ", theme::accent_style()),
            Span::styled(self.label, theme::heading_style()),
            Span::raw("  "),
            Span::styled(display, theme::input_style()),
            Span::styled("\u{2588}", theme::accent_style()),
        ]);
        Paragraph::new(line).render(area, buf);
    }
}
