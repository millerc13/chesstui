use cozy_chess::{Color as ChessColor, Piece};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Widget};

use crate::app::App;
use crate::theme::Theme;

pub struct CapturedWidget<'a> {
    app: &'a App,
    theme: &'a Theme,
}

impl<'a> CapturedWidget<'a> {
    pub fn new(app: &'a App, theme: &'a Theme) -> Self {
        Self { app, theme }
    }
}

fn piece_unicode(piece: Piece, color: ChessColor) -> &'static str {
    match (color, piece) {
        (ChessColor::White, Piece::Pawn)   => "\u{2659}",
        (ChessColor::White, Piece::Knight) => "\u{2658}",
        (ChessColor::White, Piece::Bishop) => "\u{2657}",
        (ChessColor::White, Piece::Rook)   => "\u{2656}",
        (ChessColor::White, Piece::Queen)  => "\u{2655}",
        (ChessColor::White, Piece::King)   => "\u{2654}",
        (ChessColor::Black, Piece::Pawn)   => "\u{265f}",
        (ChessColor::Black, Piece::Knight) => "\u{265e}",
        (ChessColor::Black, Piece::Bishop) => "\u{265d}",
        (ChessColor::Black, Piece::Rook)   => "\u{265c}",
        (ChessColor::Black, Piece::Queen)  => "\u{265b}",
        (ChessColor::Black, Piece::King)   => "\u{265a}",
    }
}

fn piece_value(piece: Piece) -> i32 {
    match piece {
        Piece::Pawn => 1,
        Piece::Knight | Piece::Bishop => 3,
        Piece::Rook => 5,
        Piece::Queen => 9,
        Piece::King => 0,
    }
}

impl Widget for CapturedWidget<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let block = Block::default()
            .title(" Captured ")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(self.theme.border_dim));

        let inner = block.inner(area);
        block.render(area, buf);

        if inner.width == 0 || inner.height == 0 {
            return;
        }

        // White's captures (black pieces taken)
        let white_caps = self.app.captured_by_white();
        let black_caps = self.app.captured_by_black();

        let white_material: i32 = white_caps.iter().map(|c| piece_value(c.piece)).sum();
        let black_material: i32 = black_caps.iter().map(|c| piece_value(c.piece)).sum();
        let diff = white_material - black_material;

        let mut lines: Vec<Line> = Vec::new();

        // White's captures line
        let mut white_spans: Vec<Span> = vec![Span::styled(
            "W: ",
            Style::default().fg(self.theme.text_dim),
        )];
        for cap in white_caps {
            white_spans.push(Span::styled(
                piece_unicode(cap.piece, cap.color),
                Style::default().fg(self.theme.text_primary),
            ));
        }
        if diff > 0 {
            white_spans.push(Span::styled(
                format!(" +{}", diff),
                Style::default().fg(self.theme.accent),
            ));
        }
        lines.push(Line::from(white_spans));

        // Black's captures line
        let mut black_spans: Vec<Span> = vec![Span::styled(
            "B: ",
            Style::default().fg(self.theme.text_dim),
        )];
        for cap in black_caps {
            black_spans.push(Span::styled(
                piece_unicode(cap.piece, cap.color),
                Style::default().fg(self.theme.text_primary),
            ));
        }
        if diff < 0 {
            black_spans.push(Span::styled(
                format!(" +{}", -diff),
                Style::default().fg(self.theme.accent),
            ));
        }
        lines.push(Line::from(black_spans));

        let paragraph = Paragraph::new(lines);
        paragraph.render(inner, buf);
    }
}
