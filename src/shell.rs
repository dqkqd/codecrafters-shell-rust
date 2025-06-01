use std::{fs::OpenOptions, io::Write};

use anyhow::Result;
use rustyline::{error::ReadlineError, CompletionType, Config, Editor};

use crate::{
    complete::{ShellCompleter, ShellHelper},
    parse::StreamCommandParser,
    HIST_FILE,
};

pub fn run_shell() -> Result<()> {
    let config = Config::builder()
        .completion_type(CompletionType::List)
        .build();
    let mut rl = Editor::with_config(config)?;
    let h = ShellHelper {
        completer: ShellCompleter {},
    };
    rl.set_helper(Some(h));

    let mut hist_file = OpenOptions::new()
        .truncate(true)
        .create(true)
        .write(true)
        .open(HIST_FILE)
        .unwrap();

    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let parser = StreamCommandParser::new(&line);
                if !parser.is_empty() {
                    hist_file.write_all(format!("{line}\r\n").as_bytes())?;
                    let command = parser.finish()?;
                    command.execute()?;
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("CTRL-C");
                break;
            }
            Err(ReadlineError::Eof) => {
                break;
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    Ok(())
}
