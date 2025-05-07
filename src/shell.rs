use anyhow::Result;
use crossterm::{
    cursor::MoveLeft,
    event::{read, Event, KeyCode, KeyModifiers},
    execute,
    style::Print,
    terminal::{disable_raw_mode, enable_raw_mode},
};
use std::io;

use crate::parse::StreamCommandParser;

fn execute_command(parser: StreamCommandParser) -> Result<()> {
    if !parser.is_empty() {
        let mut command = parser.finish()?;
        command.execute()?;
    }
    Ok(())
}

pub fn run_shell() -> Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();

    'session: loop {
        // first write dollar sign
        execute!(stdout, Print("$ "))?;

        let mut parser = StreamCommandParser::new();

        'command: loop {
            match read()? {
                Event::Key(key) => {
                    match (key.modifiers, key.code) {
                        // stop
                        (KeyModifiers::CONTROL, KeyCode::Char('c')) => break 'session,
                        (KeyModifiers::CONTROL, KeyCode::Char('d')) => break 'session,
                        (KeyModifiers::CONTROL, KeyCode::Char('z')) => break 'session,
                        // execute
                        (KeyModifiers::CONTROL, KeyCode::Char('j')) | (_, KeyCode::Enter) => {
                            execute!(stdout, Print("\r\n"),)?;
                            execute_command(parser)?;
                            break 'command;
                        }
                        _ => {}
                    }

                    match key.code {
                        KeyCode::Backspace => {
                            parser.pop();
                            execute!(stdout, MoveLeft(1), Print(" "), MoveLeft(1))?;
                        }
                        KeyCode::Char(c) => {
                            parser.push(c);
                            execute!(stdout, Print(c))?;
                        }
                        KeyCode::Tab => todo!(),
                        k => {
                            eprintln!("unimplemented keycode={:?}", k);
                            break 'session;
                        }
                    }
                }
                e => {
                    eprintln!("unimplemented event={:?}", e);
                    break 'session;
                }
            }
        }
    }

    disable_raw_mode()?;

    Ok(())
}
