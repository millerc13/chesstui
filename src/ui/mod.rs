pub mod ascii3d;
pub mod board;
pub mod board_image;
pub mod friends;
pub mod kitty_transmit;
pub mod captured;
pub mod debug_panel;
#[allow(dead_code)]
pub mod pieces;
pub mod color_picker;
pub mod launch;
pub mod command_bar;
pub mod game;
pub mod help;
pub mod menu;
pub mod move_list;
pub mod multiplayer;
pub mod sprites;
pub mod widgets;
pub mod postgame;
pub mod replay_viewer;

use ratatui::Frame;
use crate::app::{App, Screen};

pub fn draw(frame: &mut Frame, app: &mut App) {
    match app.screen {
        Screen::Launch => launch::draw_launch(frame, app),
        Screen::ColorPicker => color_picker::draw_color_picker(frame, app),
        Screen::MainMenu => menu::draw_menu(frame, app),
        Screen::InGame => game::draw_game(frame, app),
        Screen::PostGame => postgame::draw_postgame(frame, app),
        Screen::ReplayViewer => replay_viewer::draw_replay_viewer(frame, app),
    }
    if app.show_help {
        help::draw_help_modal(frame, app);
    }
}
