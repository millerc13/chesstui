use ratatui::layout::Rect;
use ratatui::style::{Modifier, Style};
use ratatui::text::Span;
use ratatui::widgets::{Block, BorderType, Borders};
use ratatui::Frame;

use crate::app::{App, MultiplayerState};

pub fn draw_friends_sidebar(frame: &mut Frame, area: Rect, app: &App) {
    if area.width < 8 || area.height < 4 {
        return;
    }

    // Use a proper ratatui Block with rounded borders and a title
    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(app.theme.border_dim))
        .title(Span::styled(
            " FRIENDS ",
            Style::default()
                .fg(app.theme.accent)
                .add_modifier(Modifier::BOLD),
        ));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.width < 4 || inner.height < 2 {
        return;
    }

    let buf = frame.buffer_mut();
    let mut y = inner.y;
    let max_y = inner.y + inner.height;

    let is_logged_in = matches!(app.multiplayer_state, MultiplayerState::LoggedIn { .. });

    if app.friends_list.is_empty() {
        y += 1; // blank line
        if y >= max_y {
            return;
        }

        let msg = if !is_logged_in {
            "Log in to see friends"
        } else {
            "No friends yet"
        };
        buf.set_string(inner.x + 1, y, msg, Style::default().fg(app.theme.text_dim));
        y += 1;

        if is_logged_in && y < max_y {
            buf.set_string(
                inner.x + 1,
                y,
                "Add with :friend <name>",
                Style::default().fg(app.theme.text_dim),
            );
        }
    } else {
        for friend in &app.friends_list {
            if y + 1 >= max_y {
                break;
            }
            y += 1; // gap between entries

            if y >= max_y {
                break;
            }

            // Line 1: status_dot + name + right-aligned ELO
            let (dot, dot_color) = if friend.online && friend.activity != "Offline" {
                ("●", ratatui::style::Color::Green)
            } else {
                ("◌", app.theme.text_dim)
            };

            buf.set_string(inner.x + 1, y, dot, Style::default().fg(dot_color));
            buf.set_string(
                inner.x + 3,
                y,
                &friend.name,
                Style::default()
                    .fg(app.theme.text_bright)
                    .add_modifier(Modifier::BOLD),
            );

            // Right-aligned ELO
            let elo_str = format!("{}", friend.elo);
            let elo_x = (inner.x + inner.width).saturating_sub(elo_str.len() as u16 + 1);
            buf.set_string(elo_x, y, &elo_str, Style::default().fg(app.theme.text_dim));

            y += 1;
            if y >= max_y {
                break;
            }

            // Line 2: activity + right-aligned challenge icon if online
            buf.set_string(
                inner.x + 3,
                y,
                &friend.activity,
                Style::default().fg(app.theme.text_dim),
            );

            if friend.online && friend.activity != "Offline" {
                let icon = "[⚔]";
                let icon_x = (inner.x + inner.width).saturating_sub(icon.len() as u16 + 1);
                buf.set_string(icon_x, y, icon, Style::default().fg(app.theme.accent));
            }

            y += 1;
        }
    }

    // Bottom: "+ Add Friend" in accent color
    let add_y = inner.y + inner.height - 1;
    if add_y > y && add_y >= inner.y {
        // Draw a separator line above the add button
        let sep_y = add_y.saturating_sub(1);
        if sep_y > y && sep_y >= inner.y {
            let divider = "─".repeat(inner.width as usize);
            buf.set_string(
                inner.x,
                sep_y,
                &divider,
                Style::default().fg(app.theme.border_dim),
            );
        }
        buf.set_string(
            inner.x + 1,
            add_y,
            "+ Add Friend",
            Style::default().fg(app.theme.accent),
        );
    }
}
