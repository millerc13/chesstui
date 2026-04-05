use cozy_chess::{Color as ChessColor, Piece};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::Frame;

use crate::app::{App, GameMode};
use super::board::ChessBoardWidget;
use super::board_image;
use super::command_bar::CommandBarWidget;
use super::debug_panel::DebugPanel;
use super::move_list::MoveListWidget;
use super::widgets::{render_section_header, PlayerBar};

pub fn draw_game(frame: &mut Frame, app: &mut App) {
    let area = frame.area();

    let rows = Layout::vertical([
        Constraint::Length(1),   // opponent player bar
        Constraint::Min(10),     // main area (move list + board + game info)
        Constraint::Length(1),   // your player bar
        Constraint::Length(2),   // command bar
    ])
    .split(area);

    // --- Determine player info ---
    let (top_name, top_rating, top_icon, top_captured, bottom_name, bottom_rating, bottom_icon, bottom_captured) = {
        // "bottom" = the player sitting at the bottom of the board (White unless flipped)
        // "top" = opponent at the top
        let (bottom_color, top_color) = if app.board_flipped {
            (ChessColor::Black, ChessColor::White)
        } else {
            (ChessColor::White, ChessColor::Black)
        };

        let (bottom_n, bottom_r, top_n, top_r) = match &app.game_mode {
            GameMode::Local => {
                let bn = if bottom_color == ChessColor::White { "White" } else { "Black" };
                let tn = if top_color == ChessColor::White { "White" } else { "Black" };
                (bn.to_string(), 0u32, tn.to_string(), 0u32)
            }
            GameMode::VsAi(ai_color) => {
                if bottom_color == *ai_color {
                    ("Computer".to_string(), 0, "You".to_string(), 0)
                } else {
                    ("You".to_string(), 0, "Computer".to_string(), 0)
                }
            }
            GameMode::Online { opponent_name, my_color, .. } => {
                if bottom_color == *my_color {
                    ("You".to_string(), 0, opponent_name.clone(), 0)
                } else {
                    (opponent_name.clone(), 0, "You".to_string(), 0)
                }
            }
        };

        let bottom_icon = if bottom_color == ChessColor::White { '\u{2659}' } else { '\u{265f}' };
        let top_icon = if top_color == ChessColor::White { '\u{2659}' } else { '\u{265f}' };

        // Captured pieces: pieces captured BY a color (= opponent pieces taken)
        let white_caps = app.captured_by_white();
        let black_caps = app.captured_by_black();

        let bottom_caps: Vec<(Piece, ChessColor)> = if bottom_color == ChessColor::White {
            white_caps.iter().map(|c| (c.piece, c.color)).collect()
        } else {
            black_caps.iter().map(|c| (c.piece, c.color)).collect()
        };

        let top_caps: Vec<(Piece, ChessColor)> = if top_color == ChessColor::White {
            white_caps.iter().map(|c| (c.piece, c.color)).collect()
        } else {
            black_caps.iter().map(|c| (c.piece, c.color)).collect()
        };

        (top_n, top_r, top_icon, top_caps, bottom_n, bottom_r, bottom_icon, bottom_caps)
    };

    // Opponent player bar (top)
    let top_bar = PlayerBar::new(&top_name, top_rating, &top_captured, "--:--", &app.theme)
        .icon(top_icon);
    frame.render_widget(top_bar, rows[0]);

    // Main area: 3-column layout
    let main_area = rows[1];

    let cols = Layout::horizontal([
        Constraint::Percentage(17),
        Constraint::Percentage(66),
        Constraint::Percentage(17),
    ])
    .split(main_area);

    // Board — try image protocol first, fall back to character-based
    draw_board(frame, app, cols[1]);

    // Left panel: debug panel or move list
    if app.show_debug {
        let debug = DebugPanel::new(&app.theme);
        frame.render_widget(debug, cols[0]);
    } else {
        let move_list = MoveListWidget::new(&app.game, &app.theme, app.move_list_scroll);
        frame.render_widget(move_list, cols[0]);
    }

    // Right panel: game info
    render_game_info(frame, app, cols[2]);

    // Your player bar (bottom)
    let bottom_bar = PlayerBar::new(&bottom_name, bottom_rating, &bottom_captured, "--:--", &app.theme)
        .icon(bottom_icon);
    frame.render_widget(bottom_bar, rows[2]);

    // Command bar
    let cmd_bar = CommandBarWidget::new(app);
    frame.render_widget(cmd_bar, rows[3]);

    // Promotion overlay
    if let Some(promo_moves) = &app.pending_promotion {
        draw_promotion_popup(frame, app, promo_moves.as_slice(), area);
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

fn render_game_info(frame: &mut Frame, app: &App, area: Rect) {
    let buf = frame.buffer_mut();

    if area.width < 4 || area.height < 3 {
        return;
    }

    // Section header
    render_section_header(buf, area, area.y, "GAME INFO", &app.theme);

    let content_y = area.y + 2;
    let theme = &app.theme;

    // Opening label
    if content_y < area.y + area.height {
        buf.set_string(
            area.x + 1,
            content_y,
            "Opening:",
            Style::default().fg(theme.text_dim),
        );
    }
    if content_y + 1 < area.y + area.height {
        buf.set_string(
            area.x + 1,
            content_y + 1,
            "Standard",
            Style::default().fg(theme.text_bright).add_modifier(Modifier::BOLD),
        );
    }

    // Material label
    if content_y + 3 < area.y + area.height {
        buf.set_string(
            area.x + 1,
            content_y + 3,
            "Material:",
            Style::default().fg(theme.text_dim),
        );
    }
    if content_y + 4 < area.y + area.height {
        // Calculate material difference from captured pieces
        let white_caps = app.captured_by_white();
        let black_caps = app.captured_by_black();
        let white_material: i32 = white_caps.iter().map(|c| piece_value(c.piece)).sum();
        let black_material: i32 = black_caps.iter().map(|c| piece_value(c.piece)).sum();
        let diff = white_material - black_material;

        let material_text = if diff > 0 {
            format!("White +{}", diff)
        } else if diff < 0 {
            format!("Black +{}", -diff)
        } else {
            "Even".to_string()
        };

        buf.set_string(
            area.x + 1,
            content_y + 4,
            &material_text,
            Style::default().fg(theme.text_bright).add_modifier(Modifier::BOLD),
        );
    }
}

fn draw_board(frame: &mut Frame, app: &mut App, area: Rect) {
    if app.board_picker.is_some() && app.use_kitty && !app.show_help {
        // Image-based rendering via Kitty/Sixel protocol
        draw_board_image(frame, app, area);
    } else {
        // Fallback to character-based rendering
        let board_widget = ChessBoardWidget::new(app.game.board(), &app.theme)
            .flipped(app.board_flipped)
            .cursor(app.cursor_file, app.cursor_rank)
            .selected(app.selected_square)
            .legal_moves(&app.legal_moves_for_selected)
            .last_move(app.last_move);

        // Compute board layout for mouse hit-testing (mirrors ChessBoardWidget::render)
        let label_col_w: u16 = 2;
        let label_row_h: u16 = 1;
        let available_w = area.width.saturating_sub(label_col_w);
        let available_h = area.height.saturating_sub(label_row_h);
        let ratio = super::board::cell_aspect_ratio();
        let mut ch: u16 = (available_h / 8).max(1);
        let mut cw: u16;
        loop {
            cw = (ratio * ch as f32).round() as u16;
            if cw * 8 <= available_w { break; }
            if ch <= 1 { cw = available_w / 8; break; }
            ch -= 1;
        }
        ch = ch.max(1);
        cw = cw.max(2);
        let board_w = 8 * cw;
        let board_h = 8 * ch;
        let ox = area.x + (area.width.saturating_sub(board_w + label_col_w)) / 2;
        let oy = area.y + (area.height.saturating_sub(board_h + label_row_h)) / 2;
        let bx = ox + label_col_w;
        app.board_layout = crate::app::BoardLayout {
            board_x: bx, board_y: oy,
            sq_w: cw as f32, sq_h: ch as f32,
        };

        frame.render_widget(board_widget, area);
    }
}

fn draw_board_image(frame: &mut Frame, app: &mut App, area: Rect) {
    let picker = app.board_picker.as_ref().unwrap();
    let cell_size = picker.font_size();
    if cell_size.0 == 0 || cell_size.1 == 0 {
        let board_widget = ChessBoardWidget::new(app.game.board(), &app.theme)
            .flipped(app.board_flipped)
            .cursor(app.cursor_file, app.cursor_rank)
            .selected(app.selected_square)
            .legal_moves(&app.legal_moves_for_selected)
            .last_move(app.last_move);
        frame.render_widget(board_widget, area);
        return;
    }

    let label_cols: u16 = 2;
    let label_rows: u16 = 1;
    let board_cols = area.width.saturating_sub(label_cols);
    let board_rows = area.height.saturating_sub(label_rows);

    let px_w = board_cols as u32 * cell_size.0 as u32;
    let px_h = board_rows as u32 * cell_size.1 as u32;
    let board_px_side = px_w.min(px_h);
    let sq_px = board_px_side / 8;
    if sq_px < 4 { return; }

    // Rebuild piece cache only when size or style changes
    let need_piece_cache = match &app.cached_piece_cache {
        Some(cache) => cache.sq_px != sq_px || cache.style != app.piece_style,
        None => true,
    };
    if need_piece_cache {
        crate::perf_timer!("piece_cache_rebuild");
        let cache = board_image::PieceCache::new(sq_px, &app.theme, app.piece_style);
        app.cached_piece_cache = Some(cache);
    }

    // Composite board image with ALL visual features
    let piece_cache = app.cached_piece_cache.as_ref().unwrap();
    let board_img = board_image::render_board_image(
        app.game.board(),
        &app.theme,
        sq_px,
        piece_cache,
        app.board_flipped,
        Some((app.cursor_file, app.cursor_rank)),
        app.selected_square,
        &app.legal_moves_for_selected,
        app.last_move,
    );

    // Hash to detect actual visual changes — skip retransmit if unchanged
    let new_hash = super::kitty_transmit::image_hash(&board_img);

    // Compute board area in terminal cells
    let board_px = sq_px * 8;
    let board_char_w = (board_px + cell_size.0 as u32 - 1) / cell_size.0 as u32;
    let board_char_h = (board_px + cell_size.1 as u32 - 1) / cell_size.1 as u32;
    let ox = area.x + label_cols + (board_cols.saturating_sub(board_char_w as u16)) / 2;
    let oy = area.y + (board_rows.saturating_sub(board_char_h as u16)) / 2;
    let board_area = Rect::new(ox, oy, board_char_w as u16, board_char_h as u16);

    // Save layout for mouse hit-testing
    app.board_layout = crate::app::BoardLayout {
        board_x: ox, board_y: oy,
        sq_w: board_char_w as f32 / 8.0,
        sq_h: board_char_h as f32 / 8.0,
    };

    // Only re-encode PNG if image actually changed
    let needs_transmit = if new_hash != app.kitty_image_hash || app.kitty_cache.is_none()
        || app.cached_board_sq_px != sq_px
    {
        crate::perf_timer!("kitty_png_encode");
        let png_bytes = super::kitty_transmit::encode_png(&board_img);
        let transmit_str = super::kitty_transmit::build_transmit_sequence(&png_bytes, app.kitty_image_id);
        app.kitty_cache = Some(super::kitty_transmit::KittyBoardCache {
            transmit_str,
            image_hash: new_hash,
            image_id: app.kitty_image_id,
            area: board_area,
        });
        app.kitty_image_hash = new_hash;
        app.cached_board_sq_px = sq_px;
        true
    } else {
        if let Some(cache) = app.kitty_cache.as_mut() {
            cache.area = board_area;
        }
        false
    };

    // Render into ratatui buffer
    let cache = app.kitty_cache.as_ref().unwrap();
    super::kitty_transmit::render_to_buffer(cache, board_area, frame.buffer_mut(), needs_transmit);

    // Rank labels
    let ch_per_sq = board_char_h as f32 / 8.0;
    let label_style = ratatui::style::Style::default()
        .fg(app.theme.accent_secondary)
        .add_modifier(ratatui::style::Modifier::DIM);

    for dr in 0..8u8 {
        let (_, rank) = if app.board_flipped { (7 - 0, dr) } else { (0, 7 - dr) };
        let y = oy + (dr as f32 * ch_per_sq + ch_per_sq / 2.0) as u16;
        if y < area.y + area.height {
            if let Some(cell) = frame.buffer_mut().cell_mut(ratatui::layout::Position::new(ox.saturating_sub(1), y)) {
                cell.set_char((b'1' + rank) as char);
                cell.set_style(label_style);
            }
        }
    }

    // File labels
    let cw_per_sq = board_char_w as f32 / 8.0;
    let fy = oy + board_char_h as u16;
    for dc in 0..8u8 {
        let (file, _) = if app.board_flipped { (7 - dc, 0) } else { (dc, 0) };
        let x = ox + (dc as f32 * cw_per_sq + cw_per_sq / 2.0) as u16;
        if x < area.x + area.width && fy < area.y + area.height {
            if let Some(cell) = frame.buffer_mut().cell_mut(ratatui::layout::Position::new(x, fy)) {
                cell.set_char((b'a' + file) as char);
                cell.set_style(label_style);
            }
        }
    }
}

fn draw_promotion_popup(frame: &mut Frame, app: &App, promo_moves: &[cozy_chess::Move], area: Rect) {
    let popup_width = (promo_moves.len() as u16) * 4 + 2;
    let popup_height = 3;
    let x = area.x + (area.width.saturating_sub(popup_width)) / 2;
    let y = area.y + (area.height.saturating_sub(popup_height)) / 2;
    let popup_area = Rect::new(x, y, popup_width, popup_height);

    for py in popup_area.y..popup_area.y + popup_area.height {
        for px in popup_area.x..popup_area.x + popup_area.width {
            if let Some(cell) = frame.buffer_mut().cell_mut(ratatui::layout::Position::new(px, py)) {
                cell.set_char(' ');
                cell.set_style(Style::default().bg(app.theme.border_dim));
            }
        }
    }

    frame.buffer_mut().set_string(
        popup_area.x + 1, popup_area.y,
        "Promote to:",
        Style::default().fg(app.theme.text_bright).bg(app.theme.border_dim),
    );

    for (i, mv) in promo_moves.iter().enumerate() {
        let sym = App::promotion_piece_symbol(mv);
        let style = if i == app.promotion_choice {
            Style::default().fg(app.theme.text_bright).bg(app.theme.accent)
        } else {
            Style::default().fg(app.theme.text_primary).bg(app.theme.border_dim)
        };
        frame.buffer_mut().set_string(
            popup_area.x + 1 + (i as u16) * 4, popup_area.y + 1,
            format!(" {} ", sym), style,
        );
    }

    let hint = "h/l choose  Enter confirm  Esc cancel";
    if popup_area.y + 2 < area.y + area.height {
        frame.buffer_mut().set_string(
            popup_area.x + 1, popup_area.y + 2,
            hint, Style::default().fg(app.theme.text_dim).bg(app.theme.border_dim),
        );
    }
}
