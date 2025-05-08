use anyhow::Result;
use rustyline::{error::ReadlineError, CompletionType, Config, Editor};

use crate::{
    completer::{ShellCompleter, ShellHelper},
    parse::StreamCommandParser,
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

    loop {
        let readline = rl.readline("$ ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                let parser = StreamCommandParser::new(&line);
                if !parser.is_empty() {
                    let mut command = parser.finish()?;
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
