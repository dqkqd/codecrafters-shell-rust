use std::io::{StdoutLock, Write};

use anyhow::Result;
use crossterm::event::{self, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub(super) enum Key {
    Char(char),
    Newline,
    Backspace,
    Tab,
}

impl Key {
    pub fn read(stdout: &mut StdoutLock<'static>) -> Result<Key> {
        let key = match event::read()? {
            event::Event::Key(KeyEvent {
                code, modifiers, ..
            }) => match code {
                event::KeyCode::Backspace => Key::Backspace,
                event::KeyCode::Enter => Key::Newline,
                event::KeyCode::Char('j') if modifiers == KeyModifiers::CONTROL => Key::Newline,
                event::KeyCode::Char(ch) => Key::Char(ch),
                event::KeyCode::Tab => Key::Tab,
                code => unimplemented!("{:?}", code),
            },
            e => unimplemented!("{:?}", e),
        };

        match key {
            Key::Char(ch) => {
                let mut buf = [0; 4];
                let res = ch.encode_utf8(&mut buf);
                stdout.write_all(res.as_bytes())?;
                stdout.flush()?;
            }
            Key::Newline => {
                stdout.write_all(b"\r\n")?;
                stdout.flush()?;
            }
            Key::Tab => {}
            Key::Backspace => todo!("handle backspace"),
        };

        Ok(key)
    }
}
