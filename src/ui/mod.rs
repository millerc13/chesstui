pub mod board;
pub mod captured;
pub mod command_bar;
pub mod game;
pub mod help;
pub mod menu;
pub mod move_list;
pub mod postgame;

use ratatui::Frame;
use crate::app::{App, Screen};

pub fn draw(frame: &mut Frame, app: &App) {
    match app.screen {
        Screen::MainMenu => menu::draw_menu(frame, app),
        Screen::InGame => game::draw_game(frame, app),
        Screen::PostGame => postgame::draw_postgame(frame, app),
    }
    if app.show_help {
        help::HelpOverlay::new(&app.theme).render_overlay(frame);
    }
}
