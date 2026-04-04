use clap::Parser;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io;
use std::time::Duration;

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
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = chesstui::app::App::new();
    let result = run_app(&mut terminal, &mut app);

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
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
    loop {
        terminal.draw(|frame| chesstui::ui::draw(frame, app))?;
        if app.should_quit {
            return Ok(());
        }
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let crossterm::event::Event::Key(key) = crossterm::event::read()? {
                chesstui::input::handle_key(app, key);
            }
        }
    }
}
