use std::process::exit;
use std::time::{Duration, Instant};
use term::project::{Project, ProjectDescriptor};
use crossterm::execute;
use crossterm::event::{Event};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};
use tui::backend::{Backend, CrosstermBackend};
use tui::{Terminal};
use term::{ui, app::{App}, result::Result};

fn main() -> Result<()> {
  let mut out = std::io::stdout();
  let args = std::env::args().skip(1).collect::<Vec<String>>();

  if args.is_empty() {
    println!("Provide configuration file path");
    exit(0);
  }

  let data = std::fs::read_to_string(args[0].as_str()).unwrap();
  let projects: Vec<ProjectDescriptor> = serde_json::from_str(data.as_str())?;


  enable_raw_mode()?;
  execute!(out, EnterAlternateScreen)?;

  let backend = CrosstermBackend::new(out);
  let mut terminal = Terminal::new(backend)?;
  terminal.hide_cursor()?;
  let mut app = App::default();

  app.projects = projects.into_iter().map(|descriptor| descriptor.into()).collect();

  run(&mut terminal, & mut app)?;

  disable_raw_mode()?;
  execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
  terminal.show_cursor()?;

  Ok(())
}

fn run<T: Backend>(terminal: &mut Terminal<T>, app: &mut App) -> Result<()> {
  let tick_interval = Duration::from_millis(100);
  let mut last_tick = Instant::now();

  loop {
    terminal.draw(|f| ui::render_ui(f, app))?;

    let poll_timeout = tick_interval.checked_sub(last_tick.elapsed())
      .unwrap_or(Duration::from_secs(0));

    if crossterm::event::poll(poll_timeout)? {
      if let Event::Key(evt) = crossterm::event::read()? {
        app.on_key(evt)?;
      }
    }

    if last_tick.elapsed() >= tick_interval {
      last_tick = Instant::now();
    }

    if app.should_exit {
      return Ok(());
    }
  }
}