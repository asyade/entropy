use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap, Tabs},
    Frame, Terminal,
};

use crate::{prelude::*, state::AppState};
use tui_textarea::{Input, Key, TextArea};

pub fn ui<B: Backend>(f: &mut Frame<B>, app: &AppState, state: &mut AppState) {
    let size = f.size();
    let lines = state.input_textarea.lines().len() as u16;
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

    // let history = Paragraph::new(history_widget.clone())
    // .style(Style::default())
    // .block(Block::default())
    // .alignment(Alignment::Left);
    // f.render_widget(history, chunks[0]);
    let titles = app
        .tabs
        .iter()
        .map(|t| {
            let (first, rest) = t.title.split_at(1);
            Spans::from(vec![
                Span::styled(first, Style::default().fg(Color::Yellow)),
                Span::styled(rest, Style::default().fg(Color::Green)),
            ])
        })
        .collect();
    let tabs = Tabs::new(titles)
        .block(Block::default())
        .select(app.tab_index)
        .style(Style::default().fg(Color::Cyan))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Black),
        );
    f.render_widget(tabs, chunks[0]);

    let widget = state.input_textarea.widget();
    f.render_widget(widget, chunks[1]);
}
