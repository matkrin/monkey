use anyhow::Result;
use xterm_js_rs::Terminal;


pub struct KeyEvent {
    pub code: KeyCode,
    pub modifiers: KeyModifiers,
}

impl KeyEvent {
    pub const fn new(code: KeyCode, modifiers: KeyModifiers) -> KeyEvent {
        KeyEvent { code, modifiers }
    }
}

pub enum KeyCode {
    Backspace,
    Enter,
    Left,
    Right,
    Up,
    Down,
    Home,
    End,
    PageUp,
    PageDown,
    Tab,
    BackTab,
    Delete,
    Char(char),
    Null,
    Esc,
}

pub enum KeyModifiers {
    Shift,
    Control,
    Alt,
    None,
}

pub fn parse_key_event(buffer: &[u8]) -> Result<KeyEvent> {
    match buffer[0] {
        b'\x1B' => {
            // ESC
            if buffer.len() == 1 {
                Ok(KeyEvent::new(KeyCode::Esc, KeyModifiers::None))
            } else {
                match buffer[1] {
                    b'[' => match buffer[2] {
                        b'A' => Ok(KeyEvent::new(KeyCode::Up, KeyModifiers::None)),
                        b'B' => Ok(KeyEvent::new(KeyCode::Down, KeyModifiers::None)),
                        b'C' => Ok(KeyEvent::new(KeyCode::Right, KeyModifiers::None)),
                        b'D' => Ok(KeyEvent::new(KeyCode::Left, KeyModifiers::None)),
                        b'H' => Ok(KeyEvent::new(KeyCode::Home, KeyModifiers::None)),
                        b'F' => Ok(KeyEvent::new(KeyCode::End, KeyModifiers::None)),
                        // TODO Delete is: "\x1B[3~"
                        b'3' => Ok(KeyEvent::new(KeyCode::Delete, KeyModifiers::None)),
                        _ => unimplemented!(),
                    },
                    b'\x1B' => Ok(KeyEvent::new(KeyCode::Esc, KeyModifiers::None)),
                    b'b' => Ok(KeyEvent::new(KeyCode::Left, KeyModifiers::Alt)),
                    b'f' => Ok(KeyEvent::new(KeyCode::Right, KeyModifiers::Alt)),
                    _ => unimplemented!("or not? buffer = {:?}", buffer),
                }
            }
        }
        b'\r' => Ok(KeyEvent::new(KeyCode::Enter, KeyModifiers::None)),
        // b'\n' => Ok(KeyEvent::new(KeyCode::Enter, KeyModifiers::None)),
        b'\t' => Ok(KeyEvent::new(KeyCode::Tab, KeyModifiers::None)),
        b'\x7F' => Ok(KeyEvent::new(KeyCode::Backspace, KeyModifiers::None)),
        c @ b'\x01'..=b'\x1A' => Ok(KeyEvent::new(
            KeyCode::Char((c - 0x1 + b'a') as char),
            KeyModifiers::Control,
        )),
        c @ b'\x1C'..=b'\x1F' => Ok(KeyEvent::new(
            KeyCode::Char((c - 0x1C + b'4') as char),
            KeyModifiers::Control,
        )),
        b'\0' => Ok(KeyEvent::new(KeyCode::Char(' '), KeyModifiers::Control)),

        c => Ok(KeyEvent {
            code: KeyCode::Char(c as char),
            modifiers: KeyModifiers::None,
        }),
    }
}

pub struct LineEditor {
    term: Terminal,
    prompt: String,
    buffer: String,
    cursor: usize,
}

impl LineEditor {
    pub fn new(terminal: Terminal, prompt: &str) -> LineEditor {
        LineEditor {
            term: terminal,
            prompt: prompt.to_string(),
            buffer: String::from(""),
            cursor: 0,
        }
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn prompt(&self) {
        self.term.write(&self.prompt);
    }

    /// Inserts a character at cursor position
    pub fn insert_char(&mut self, insertion: char) {
        self.buffer.insert(self.cursor, insertion);
        self.csi_hide_cursor();
        self.term.write(&self.buffer[self.cursor..]);
        self.cursor += 1;
        self.csi_left(self.buffer.len() - self.cursor);
        self.csi_show_cursor();
    }

    pub fn insert_str(&mut self, insertion: &str) {
        self.buffer.insert_str(self.cursor, insertion);
        self.cursor += insertion.len();
    }

    /// Moves cursor n-times to the left
    fn csi_left(&self, n: usize) {
        if n > 0 {
            self.term.write(&format!("\x1b[{}D", n));
        }
    }

    /// moves cursor n-times to the right
    fn csi_right(&self, n: usize) {
        if n > 0 {
            self.term.write(&format!("\x1b[{}C", n));
        }
    }

    fn csi_hide_cursor(&self) {
        self.term.write("\x1b[?25l");
    }

    fn csi_show_cursor(&self) {
        self.term.write("\x1b[?25h");
    }

    fn csi_new_line(&self) {
        self.term.write("\r\n");
    }

    fn flush(&mut self) {
        self.buffer.clear();
        self.cursor = 0;
    }

    pub fn move_left(&mut self, n: usize) {
        if self.cursor > 0 {
            self.cursor -= n;
            self.csi_left(n);
        }
    }

    pub fn move_right(&mut self, n: usize) {
        if self.cursor < self.buffer.len() {
            self.cursor += n;
            self.csi_right(n);
        }
    }


    pub fn delete_left(&mut self) {
        if self.cursor > 0 {
            self.buffer = self
                .buffer
                .chars()
                .take(self.cursor - 1)
                .chain(self.buffer.chars().skip(self.cursor))
                .collect::<String>();
            self.term.write("\u{0008} \u{0008}");
            self.term.write("\r\x1B[K");
            self.prompt();
            self.term.write(&self.buffer);
            self.cursor -= 1;
            self.csi_left(self.buffer.len() - self.cursor);
        }
    }

    pub fn delete_right(&mut self) {
        if self.cursor < self.buffer.len() {
            self.buffer = self
                .buffer
                .chars()
                .take(self.cursor)
                .chain(self.buffer.chars().skip(self.cursor + 1))
                .collect::<String>();
            self.csi_hide_cursor();
            self.term.write("\r\x1B[K");
            self.prompt();
            self.csi_show_cursor();
            self.term.write(&self.buffer);
            self.csi_left(self.buffer.len() - self.cursor);
        }
    }

    pub fn enter(&mut self, msg: &str) {
        self.csi_new_line();
        self.term.write(msg);
        self.flush();
        self.csi_new_line();
        self.prompt();
    }

    pub fn clear_screen(&self) {
        self.term.clear();
    }

    pub fn word_left(&mut self) {
        if self.cursor > 0 {
            let idx = match self.buffer[..self.cursor - 1].rfind(' ') {
                Some(i) => i as isize,
                None => -1,
            };
            self.move_left(self.buffer[..self.cursor].len() - (idx as usize + 1));
            self.cursor = idx as usize + 1;
        }
    }

    pub fn word_right(&mut self) {
        if self.cursor < self.buffer.len() {
            let idx = match self.buffer[self.cursor..].find(' ') {
                Some(i) => i,
                None => self.buffer[self.cursor..].len() - 1,
            };
            self.move_right(1 + idx);
        }
    }

    pub fn move_start(&mut self) {
        self.move_left(self.cursor);
    }

    pub fn move_end(&mut self) {
        self.move_right(self.buffer.len() - self.cursor);
    }

    pub fn delete_line(&mut self) {
        self.csi_hide_cursor();
        self.term.write("\r\x1B[K");
        self.flush();
        self.move_start();
        self.prompt();
        self.csi_show_cursor();
    }

    pub fn delete_from_cursor(&mut self) {
        self.csi_hide_cursor();
        self.buffer = self.buffer.chars().take(self.cursor).collect();
        self.term.write("\r\x1B[K");
        self.prompt();
        self.term.write(&self.buffer);
        self.csi_show_cursor();
    }
}
