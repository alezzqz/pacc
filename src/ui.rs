use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, stdout, Stdout};
use tui::{
    backend::CrosstermBackend, style::{Color, Style}, text::{Span, Spans, Text}, widgets::{Block, Borders, List, ListItem, ListState}, Terminal
};

use crate::{pulse::PaContext, source::PaOutput};

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

pub fn show_ui(mut list_state: &mut ListState, list_elems: &mut [PaOutput], pulse_ctx: &mut PaContext) -> io::Result<()> {
    let mut ui_context = UiContext::new().unwrap();

    loop {
        ui_context.terminal.draw(|f| {
            let mut items: Vec<ListItem> = Vec::new();
            for e in list_elems.iter() {
                let percent = e.volume_percent();
                let filled = (percent.min(100) as usize + 4) / 5;
                items.push(ListItem::new(Text::from(vec![
                    Spans::from(Span::styled(e.to_list_line(), Style::default().fg(Color::Yellow))),
                    Spans::from(if e.volume_known { vec![
                        Span::styled("■".repeat(filled), Style::default().fg(Color::Green)),
                        Span::styled("■".repeat(20 - filled), Style::default().fg(Color::Red)),
                        Span::raw(format!(" {percent}%")),
                    ] } else { vec![Span::raw("—")] }),
                ])));
            }

            let list = List::new(items)
                .block(Block::default().borders(Borders::ALL).title("↑/↓ output  ←/→ volume  ENTER select  Esc/x exit"))
                .highlight_style(Style::default().bg(Color::Blue))
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, f.size(), &mut list_state);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char('x') | KeyCode::Esc => {
                    list_state.select(None);
                    break;
                }
                KeyCode::Down => {
                    if let Some(selected) = list_state.selected().filter(|_| !list_elems.is_empty()) {
                        let next = (selected + 1) % list_elems.len();
                        list_state.select(Some(next));
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = list_state.selected().filter(|_| !list_elems.is_empty()) {
                        let prev = if selected == 0 {
                            list_elems.len() - 1
                        } else {
                            selected - 1
                        };
                        list_state.select(Some(prev));
                    }
                }
                KeyCode::Left | KeyCode::Right => {
                    if let Some(selected) = list_state.selected().filter(|&index| index < list_elems.len()) {
                        let current = list_elems[selected].volume_percent();
                        let target = if key.code == KeyCode::Right {
                            current.saturating_add(5).min(100)
                        } else {
                            current.saturating_sub(5).min(100)
                        };
                        if target != current {
                            list_elems[selected].set_volume_percent(target)
                                .map_err(|message| io::Error::new(io::ErrorKind::Other, message))?;
                            if list_elems[selected].is_active_port {
                                pulse_ctx.set_sink_volume(&list_elems[selected].sink_name, &list_elems[selected].volume)
                                    .map_err(|message| io::Error::new(io::ErrorKind::Other, message))?;
                            } else {
                                list_elems[selected].pending_volume = true;
                            }
                            list_elems[selected].volume_known = true;
                        }
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
