use super::app::App;
use anyhow::Result;
use crossterm::event::{self, Event, KeyCode};
use std::time::{Duration, Instant};

#[derive(PartialEq)]
pub enum AppSignal {
    Quit,
    Continue,
}

pub fn handle_event(app: &mut App) -> Result<Option<AppSignal>> {
    let tick_rate = Duration::from_millis(250);

    if event::poll(Duration::from_millis(50))? {
        match event::read()? {
            Event::Key(key) => match key.code {
                KeyCode::Char('q') | KeyCode::Esc => return Ok(Some(AppSignal::Quit)),
                KeyCode::Down | KeyCode::Char('j') => app.next(),
                KeyCode::Up | KeyCode::Char('k') => app.previous(),
                KeyCode::Enter => {
                    if app.execute_selected()? {
                        return Ok(Some(AppSignal::Quit));
                    }
                }
                _ => {}
            },
            Event::Resize(_, _) => {}
            _ => {}
        }
    }

    // Tick logic
    if app.last_tick.elapsed() >= tick_rate {
        app.check_process();
        app.last_tick = Instant::now();
    }

    Ok(Some(AppSignal::Continue))
}
