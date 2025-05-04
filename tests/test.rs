use assert_cmd::Command;

#[test]
fn print_a_prompt() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .assert()
        .success()
        .stdout("$ ");
}

#[test]
fn handle_invalid_commands() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin("some_command")
        .assert()
        .success()
        .stdout(
            r#"$ some_command: command not found
$ "#,
        );
}

#[test]
fn repl() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin("command1\ncommand2")
        .assert()
        .success()
        .stdout(
            r#"$ command1: command not found
$ command2: command not found
$ "#,
        );
}

#[test]
fn repl_empty() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin("command1\n   \n   \n")
        .assert()
        .success()
        .stdout(
            r#"$ command1: command not found
$ $ $ "#,
        );
}

#[test]
fn exit() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin("command1\nexit 0\ncommand2")
        .assert()
        .success()
        .code(0)
        .stdout(
            r#"$ command1: command not found
$ "#,
        );
}

#[test]
fn exit_1() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin("exit 101")
        .assert()
        .failure()
        .code(101)
        .stdout("$ ");
}

#[test]
fn echo() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin("echo 123")
        .assert()
        .success()
        .stdout(
            r#"$ 123
$ "#,
        );
}

#[test]
fn echo_space() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin(
            r#"echo raspberry orange apple
echo grape strawberry
echo banana mango"#,
        )
        .assert()
        .success()
        .stdout(
            r#"$ raspberry orange apple
$ grape strawberry
$ banana mango
$ "#,
        );
}

#[test]
fn ty() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin(
            r#"type echo
type exit
type type
type invalid_command
"#,
        )
        .assert()
        .success()
        .stdout(
            r#"$ echo is a shell builtin
$ exit is a shell builtin
$ type is a shell builtin
$ invalid_command: not found
$ "#,
        );
}
