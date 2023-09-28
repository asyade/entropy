use crossterm::{
    event::{self, read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyModifiers, KeyEventState, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{
    error::Error,
    io,
    time::{Duration, Instant},
};
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
    Frame, Terminal,
};

use tui_textarea::{TextArea, Key, Input};

struct App {
    scroll: u16,
}

impl App {
    fn new() -> App {
        App { scroll: 0 }
    }

    fn on_tick(&mut self) {
        self.scroll += 1;
        self.scroll %= 10;
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // create app and run it
    let tick_rate = Duration::from_millis(10);
    let app = App::new();
    let res = run_app(&mut terminal, app, tick_rate);

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{:?}", err)
    }

    Ok(())
}

fn run_app<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    let mut textarea = TextArea::default();

    loop {
        terminal.draw(|f| ui(f, &app, &mut textarea))?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or_else(|| Duration::from_secs(0));
        if crossterm::event::poll(timeout)? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Release {
                        continue;
                    } 
                    if key.code == KeyCode::Esc {
                        return Ok(());
                    }
                    let ctrl = key.modifiers.contains(KeyModifiers::CONTROL);
                    let alt = key.modifiers.contains(KeyModifiers::ALT);
                    let key = match key.code {
                        KeyCode::Char(c) => Key::Char(c),
                        KeyCode::Backspace => Key::Backspace,
                        KeyCode::Enter => Key::Enter,
                        KeyCode::Left => Key::Left,
                        KeyCode::Right => Key::Right,
                        KeyCode::Up => Key::Up,
                        KeyCode::Down => Key::Down,
                        KeyCode::Tab => Key::Tab,
                        KeyCode::Delete => Key::Delete,
                        KeyCode::Home => Key::Home,
                        KeyCode::End => Key::End,
                        KeyCode::PageUp => Key::PageUp,
                        KeyCode::PageDown => Key::PageDown,
                        KeyCode::Esc => Key::Esc,
                        KeyCode::F(x) => Key::F(x),
                        _ => Key::Null,
                    };
                    textarea.input(Input { key, ctrl, alt });
                }
                _ => {}
            }
        }
        if last_tick.elapsed() >= tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn ui<B: Backend>(f: &mut Frame<B>, app: &App, textarea: &mut TextArea) {
    let size = f.size();
    let lines = textarea.lines().len() as u16;
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(size.height.max(lines) - lines),
                Constraint::Length(18),
            ]
            .as_ref(),
        )
        .split(size);

    let history_widget = vec![
        Spans::from("This is a line "),
        Spans::from(Span::styled(
            "This is a line   ",
            Style::default().fg(Color::Red),
        )),
    ];

    let history = Paragraph::new(history_widget.clone())
        // .style(Style::default())
        .block(Block::default())
        .alignment(Alignment::Left);
    f.render_widget(history, chunks[0]);

    let widget = textarea.widget();
    f.render_widget(widget, chunks[1]);
}
