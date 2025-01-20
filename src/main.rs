use std::{
    error, fmt,
    io::{self, Write},
    num::ParseIntError,
};

// TODO: lifetime
enum Command {
    Exit(i32),
    Echo(String),
    Invalid(String),
}

#[derive(Debug)]
enum CommandError {
    ParseIntError(ParseIntError),
}

impl fmt::Display for CommandError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            CommandError::ParseIntError(ref err) => write!(f, "ParseIntError: {}", err),
        }
    }
}

impl error::Error for CommandError {}

impl From<ParseIntError> for CommandError {
    fn from(err: ParseIntError) -> CommandError {
        CommandError::ParseIntError(err)
    }
}

impl TryFrom<String> for Command {
    type Error = CommandError;

    fn try_from(command: String) -> Result<Command, CommandError> {
        let command = command.trim();
        if let Some(code) = command.strip_prefix("exit ") {
            let code = code.parse()?;
            Ok(Command::Exit(code))
        } else if let Some(echo) = command.strip_prefix("echo ") {
            Ok(Command::Echo(echo.into()))
        } else {
            Ok(Command::Invalid(command.into()))
        }
    }
}

impl Command {
    fn execute(self) {
        match self {
            Command::Exit(code) => std::process::exit(code),
            Command::Echo(s) => println!("{}", s),
            Command::Invalid(s) => println!("{}: command not found", s),
        }
    }
}

fn main() {
    // Uncomment this block to pass the first stage
    loop {
        print!("$ ");
        io::stdout().flush().unwrap();

        // Wait for user input
        let stdin = io::stdin();
        let mut input = String::new();
        stdin.read_line(&mut input).unwrap();

        let cmd: Command = input.try_into().unwrap();
        cmd.execute();
    }
}
