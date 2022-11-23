use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyEventState, KeyModifiers},
    queue,
    terminal::{self, ClearType},
};
use std::{
    io::{self, stdout, Write},
    time::Duration,
};

const VERSION: &str = "0.0.1";

const KEYCODE_LEFT: KeyCode = KeyCode::Char('h');
const KEYCODE_DOWN: KeyCode = KeyCode::Char('j');
const KEYCODE_UP: KeyCode = KeyCode::Char('k');
const KEYCODE_RIGHT: KeyCode = KeyCode::Char('l');

struct CleanUp;

impl Drop for CleanUp {
    fn drop(&mut self) {
        terminal::disable_raw_mode().expect("Could not turn off raw mode.");
        Output::clear_screen().expect("Fail to clear screen.")
    }
}

struct Reader;

impl Reader {
    fn read_key(&self) -> crossterm::Result<KeyEvent> {
        loop {
            if event::poll(Duration::from_millis(500))? {
                if let Event::Key(event) = event::read()? {
                    return Ok(event);
                }
            }
        }
    }
}

struct CursorController {
    x: usize,
    y: usize,
    window: (usize, usize),
}

enum Direction {
    Left,
    Up,
    Down,
    Right,
    Home,
    End,
}

impl CursorController {
    fn new(window: (usize, usize)) -> CursorController {
        Self { x: 0, y: 0, window }
    }

    fn move_cursor(&mut self, direction: Direction) {
        match direction {
            Direction::Left => {
                self.x = self.x.saturating_sub(1);
            }
            Direction::Down => {
                if self.y < self.window.1 {
                    self.y += 1;
                };
            }
            Direction::Up => {
                self.y = self.y.saturating_sub(1);
            }
            Direction::Right => {
                if self.x < self.window.0 {
                    self.x += 1;
                }
            }
            Direction::Home => {
                self.x = 0;
            }
            Direction::End => {
                self.x = self.window.0;
            }
        }
    }
}

struct Output {
    window: (usize, usize),
    editor_contents: EditorContents,
    cursor_controller: CursorController,
}

impl Output {
    fn new() -> Output {
        let window = terminal::size()
            .map(|(x, y)| (x as usize, y as usize))
            .unwrap();
        Self {
            window,
            editor_contents: EditorContents::new(),
            cursor_controller: CursorController::new(window),
        }
    }

    fn welcome(&self) -> String {
        let columns = self.window.0;
        let mut welcome = format!("{: ^1$}", format!("Version {}", VERSION), columns - 1);
        if welcome.len() > columns {
            welcome.truncate(columns);
        }
        welcome
    }

    fn draw_rows(&mut self) {
        let height = self.window.1;
        for i in 0..height {
            self.editor_contents.push('~');
            queue!(
                self.editor_contents,
                terminal::Clear(ClearType::UntilNewLine)
            )
            .unwrap();
            if i == height / 3 {
                self.editor_contents.push_str(self.welcome().as_str());
            }
            if i < height - 1 {
                self.editor_contents.push_str("\r\n");
            }
        }
        stdout().flush().unwrap();
    }

    fn move_cursor(&mut self, code: KeyCode) {
        let direction: Direction = match code {
            KEYCODE_LEFT => Direction::Left,
            KEYCODE_UP => Direction::Up,
            KEYCODE_DOWN => Direction::Down,
            KEYCODE_RIGHT => Direction::Right,
            KeyCode::Home => Direction::Home,
            KeyCode::End => Direction::End,
            _ => unimplemented!(),
        };
        self.cursor_controller.move_cursor(direction);
    }

    fn clear_screen() -> crossterm::Result<()> {
        queue!(
            stdout(),
            terminal::Clear(ClearType::All),
            cursor::MoveTo(0, 0)
        )
    }

    fn refresh_screen(&mut self) -> crossterm::Result<()> {
        queue!(self.editor_contents, cursor::Hide, cursor::MoveTo(0, 0))?;
        self.draw_rows();
        let x = self.cursor_controller.x;
        let y = self.cursor_controller.y;
        queue!(
            self.editor_contents,
            cursor::MoveTo(x as u16, y as u16),
            cursor::Show
        )?;
        self.editor_contents.flush()
    }
}

struct Editor {
    reader: Reader,
    output: Output,
}

impl Editor {
    fn new() -> Self {
        Self {
            reader: Reader,
            output: Output::new(),
        }
    }

    fn keypress_process(&mut self) -> crossterm::Result<bool> {
        match self.reader.read_key()? {
            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => return Ok(false),
            KeyEvent {
                code:
                    val @ (KEYCODE_LEFT
                    | KEYCODE_RIGHT
                    | KEYCODE_DOWN
                    | KEYCODE_UP
                    | KeyCode::Home
                    | KeyCode::End),
                modifiers: KeyModifiers::NONE,
                kind: KeyEventKind::Press,
                state: KeyEventState::NONE,
            } => self.output.move_cursor(val),
            _ => {}
        }

        Ok(true)
    }

    fn run(&mut self) -> crossterm::Result<bool> {
        self.output.refresh_screen()?;
        self.keypress_process()
    }
}

struct EditorContents {
    content: String,
}

impl EditorContents {
    fn new() -> Self {
        Self {
            content: String::new(),
        }
    }

    fn push(&mut self, ch: char) {
        self.content.push(ch)
    }

    fn push_str(&mut self, string: &str) {
        self.content.push_str(string)
    }
}

impl io::Write for EditorContents {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match std::str::from_utf8(buf) {
            Ok(s) => {
                self.push_str(s);
                Ok(s.len())
            }
            Err(_) => Err(io::ErrorKind::WriteZero.into()),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        let out = write!(stdout(), "{}", self.content);
        stdout().flush()?;
        self.content.clear();
        out
    }
}

fn main() -> crossterm::Result<()> {
    let _clean_up = CleanUp;
    terminal::enable_raw_mode()?;
    let mut editor = Editor::new();
    while editor.run()? {}
    Ok(())
}
