use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    terminal,
};
use std::time::Duration;

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not turn off raw mode.");
    }
}

struct Reader;

impl Reader {
    fn read_key(&self) -> crossterm::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(event) = event::read()? {
                    println!("{:?}\r", event);
                    return Ok(event);
                }
            } else {
                println!("no input yet\r");
            }
        }
    }
}

struct Editor {
    reader: Reader,
}

impl Editor {
    fn new() -> Self {
        Self { reader: Reader }
    }

    fn keypress_process(&self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => return Ok(false),
            _ => {}
        }

        Ok(true)
    }

    fn run(&self) -> crossterm::Result<bool> {
        self.keypress_process()
    }
}

fn main() -> crossterm::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode().expect("Could not turn on raw mode.");
    let editor = Editor::new();
    while editor.run()? {}
    Ok(())
}
