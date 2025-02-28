use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout, Stdout};
use tui::{
    backend::CrosstermBackend, style::{Color, Style}, text::{Span, Spans}, widgets::{Block, Borders, List, ListItem, ListState}, Terminal
};

use crate::source::PaOutput;

pub struct UiContext {
    terminal: Terminal<CrosstermBackend<Stdout>>
}

impl UiContext {
    fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let terminal = Terminal::new(CrosstermBackend::new(stdout))?;
        Ok(UiContext{ terminal })
    }
}

impl Drop for UiContext {
    fn drop(&mut self) {
        match disable_raw_mode() {
            Err(e) => { eprintln!("disable raw mode failure {e}"); }
            Ok(_) => {}
        }
        match execute!(self.terminal.backend_mut(), LeaveAlternateScreen) {
            Err(e) => { eprintln!("LeaveAlternateScreen failure {e}"); }
            Ok(_) => {}
        }
        let _ = self.terminal.show_cursor();
    }
}

pub fn show_ui(mut list_state: &mut ListState, list_elems: &Vec<PaOutput>) -> io::Result<()> {
    let mut ui_context = UiContext::new().unwrap();

    loop {
        ui_context.terminal.draw(|f| {
            let mut items: Vec<ListItem> = Vec::new();
            for e in list_elems {
                items.push(ListItem::new(
                    Spans::from(vec![
                        Span::styled(e.to_list_line(), Style::default().fg(Color::Yellow))
                    ]))
                );
            }

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("Choose output and press ENTER or 'x' to exit"))
                .highlight_style(Style::default().bg(Color::Blue))
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, f.size(), &mut list_state);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('x') => {
                    list_state.select(None);
                    break;
                }
                KeyCode::Down => {
                    if let Some(selected) = list_state.selected() {
                        let next = (selected + 1) % list_elems.len();
                        list_state.select(Some(next));
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = list_state.selected() {
                        let prev = if selected == 0 {
                            list_elems.len() - 1
                        } else {
                            selected - 1
                        };
                        list_state.select(Some(prev));
                    }
                }
                KeyCode::Enter => {
                    if let Some(_) = list_state.selected() {
                        break;
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}