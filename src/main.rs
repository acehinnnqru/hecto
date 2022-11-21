use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState},
    terminal,
};
use std::io::{self, Read};

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not turn off raw mode.");
    }
}

fn main() {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode().expect("Could not turn on raw mode.");
    loop {
        if let Event::Key(event) = event::read().expect("Failed to read line") {
            match event {
                KeyEvent {
                    kind: KeyEventKind::Press,
                    code: KeyCode::Char('q'),
                    modifiers: event::KeyModifiers::NONE,
                    state: KeyEventState::NONE,
                } => break,
                _ => {}
            }
            println!("{:?}\r", event);
        };
    }
}
