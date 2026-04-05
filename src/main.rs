use clap::Parser;
use crossterm::{
    event::{EnableMouseCapture, DisableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::{Duration, Instant};

#[derive(Parser)]
#[command(name = "chesstui", version, about = "Multiplayer chess in the terminal")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(clap::Subcommand)]
enum Command {
    /// Run the multiplayer server
    Server {
        /// Address to bind to
        #[arg(long, default_value = "0.0.0.0:7600")]
        bind: String,
        /// PostgreSQL connection URL
        #[arg(long, env = "DATABASE_URL")]
        database_url: String,
        /// Resend API key for email authentication
        #[arg(long, env = "RESEND_API_KEY")]
        resend_api_key: String,
    },
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Some(Command::Server { bind, database_url, resend_api_key }) => {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(chesstui::server::run(bind, database_url, resend_api_key))
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e.to_string()))
        }
        None => run_tui(),
    }
}

fn run_tui() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    chesstui::perf::init();
    let mut app = chesstui::app::App::new();
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    if let Err(err) = result {
        eprintln!("Error: {}", err);
    }
    Ok(())
}

fn run_app(
    terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    app: &mut chesstui::app::App,
) -> io::Result<()> {
    let mut needs_redraw = true;

    loop {
        let frame_start = Instant::now();

        // Only draw when something actually changed
        let draw_elapsed = if needs_redraw {
            chesstui::perf_timer!("terminal.draw");
            terminal.draw(|frame| chesstui::ui::draw(frame, app))?;
            needs_redraw = false;
            frame_start.elapsed().as_micros() as u64
        } else {
            0
        };

        app.tick = app.tick.wrapping_add(1);
        if app.should_quit {
            return Ok(());
        }

        // Screens with animation need continuous redraws
        if matches!(app.screen, chesstui::app::Screen::Launch | chesstui::app::Screen::ColorPicker) {
            needs_redraw = true;
        }

        if crossterm::event::poll(Duration::from_millis(100))? {
            match crossterm::event::read()? {
                crossterm::event::Event::Key(key) => {
                    let input_start = Instant::now();
                    chesstui::input::handle_key(app, key);
                    let input_us = input_start.elapsed().as_micros() as u64;
                    chesstui::perf::set_input_lag(draw_elapsed + input_us);
                    chesstui::perf::log("input.handle_key", input_us as u128);
                    needs_redraw = true;
                }
                crossterm::event::Event::Mouse(mouse) => {
                    if chesstui::input::handle_mouse(app, mouse) {
                        needs_redraw = true;
                    }
                }
                crossterm::event::Event::Resize(_, _) => {
                    needs_redraw = true;
                }
                _ => {}
            }
        }

        // Poll for delayed AI move
        if app.poll_ai_move() {
            needs_redraw = true;
        }

        // Poll for server messages
        let had_network = poll_network(app);
        if had_network {
            needs_redraw = true;
        }

        let total_us = frame_start.elapsed().as_micros();
        chesstui::perf::set_frame_time(total_us as u64);
        chesstui::perf::frame_boundary(total_us);
    }
}

fn poll_network(app: &mut chesstui::app::App) -> bool {
    use chesstui::app::MultiplayerState;
    use chesstui::protocol::ServerMessage;

    let msg = match app.network {
        Some(ref mut net) => net.try_recv(),
        None => return false,
    };

    if let Some(msg) = msg {
        match msg {
            ServerMessage::AuthCodeSent => {
                app.multiplayer_state = MultiplayerState::WaitingForOtp;
                // Also transition to OTP input
                app.multiplayer_state = MultiplayerState::EnteringOtp;
            }
            ServerMessage::Authenticated { token, user } => {
                app.password_input.clear();
                // Save session
                chesstui::network::session::save_session(
                    &chesstui::network::session::StoredSession {
                        token,
                        email: user.email,
                        display_name: user.display_name.clone(),
                        server_url: app.server_url.clone(),
                    },
                );
                if user.display_name.is_none() {
                    app.multiplayer_state = MultiplayerState::EnteringDisplayName;
                } else {
                    app.multiplayer_state = MultiplayerState::LoggedIn {
                        display_name: user.display_name.unwrap_or_default(),
                        elo: user.elo,
                    };
                }
            }
            ServerMessage::AuthError { reason } => {
                app.status_message = reason;
                app.multiplayer_state = MultiplayerState::EnteringEmail;
            }
            ServerMessage::NeedDisplayName => {
                app.multiplayer_state = MultiplayerState::EnteringDisplayName;
            }
            ServerMessage::Searching => {
                app.multiplayer_state = MultiplayerState::Searching;
            }
            ServerMessage::GameStart { game_id, opponent, my_color } => {
                let color = if my_color == "white" {
                    cozy_chess::Color::White
                } else {
                    cozy_chess::Color::Black
                };
                app.start_online_game(game_id, color, opponent);
                app.multiplayer_state = MultiplayerState::InGame;
            }
            ServerMessage::MoveMade { mv, .. } => {
                app.apply_server_move(&mv);
            }
            ServerMessage::MoveRejected { reason } => {
                app.status_message = reason;
            }
            ServerMessage::GameOver { result, detail, .. } => {
                app.status_message = format!("{} — {}", result, detail);
                app.screen = chesstui::app::Screen::PostGame;
                app.postgame_selection = 0;
            }
            ServerMessage::DrawOffered { .. } => {
                app.status_message = "Opponent offers a draw. :accept or :decline".to_string();
            }
            ServerMessage::NeedPassword => {
                app.multiplayer_state = MultiplayerState::EnteringPassword;
            }
            ServerMessage::PasswordSet => {
                app.password_input.clear();
                app.multiplayer_state = MultiplayerState::EnteringDisplayName;
            }
            ServerMessage::PreferencesLoaded { preferences } => {
                app.apply_server_preferences(&preferences);
            }
            ServerMessage::PreferencesUpdated => {
                // Confirmation received, nothing to do
            }
            ServerMessage::FriendsList { .. } => {
                // TODO: store for friends UI
            }
            ServerMessage::FriendAdded { .. } => {
                // TODO: update friends list
            }
            ServerMessage::FriendRemoved { .. } => {
                // TODO: update friends list
            }
            ServerMessage::Error { message } => {
                app.status_message = message;
            }
            ServerMessage::Pong => {}
        }
        true
    } else {
        false
    }
}
