use std::fs::{self, File};

use assert_cmd::Command;
use predicates::prelude::predicate;
use tempfile::tempdir;

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
fn echo_multi() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin(
            r#"echo raspberry orange apple
echo grape      strawberry"#,
        )
        .assert()
        .success()
        .stdout(
            r#"$ raspberry orange apple
$ grape strawberry
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

#[test]
fn ty_multi() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin("type echo exit type invalid_command")
        .assert()
        .success()
        .stdout(
            r#"$ echo is a shell builtin
exit is a shell builtin
type is a shell builtin
invalid_command: not found
$ "#,
        );
}

#[test]
fn ty_path_command() {
    let tmp_dir = tempdir().unwrap();
    let executable_path = tmp_dir.path().join("my_executable");
    File::create(&executable_path).unwrap();

    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .env("PATH", tmp_dir.path().as_os_str())
        .write_stdin("type my_executable")
        .assert()
        .success()
        .stdout(format!(
            r#"$ my_executable is {}
$ "#,
            executable_path.as_path().display()
        ));
}

#[test]
fn ty_path_exec() {
    let tmp_dir = tempdir().unwrap();
    File::create(tmp_dir.path().join("file1")).unwrap();
    File::create(tmp_dir.path().join("file2")).unwrap();
    File::create(tmp_dir.path().join("file3")).unwrap();

    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin(format!("ls {}", tmp_dir.path().display()))
        .assert()
        .success()
        .stdout(
            r#"$ file1
file2
file3
$ "#,
        );
}

#[test]
fn pwd() {
    let tmp_dir = tempdir().unwrap();

    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        // remove path to avoid using pwd from path
        .env("PATH", "")
        .write_stdin("pwd")
        .current_dir(tmp_dir.path())
        .assert()
        .success()
        .stdout(format!(
            r#"$ {}
$ "#,
            tmp_dir.path().display()
        ));
}

#[test]
fn cd() {
    let tmp_dir = tempdir().unwrap();
    let level2 = tmp_dir.path().join("level1").join("level2");
    fs::create_dir_all(&level2).unwrap();

    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        // remove path to avoid using cd from path
        .env("PATH", "")
        .current_dir(tmp_dir.path()) // inside tmp path
        .write_stdin(format!(
            r#"pwd
cd {}
pwd"#,
            level2.display()
        ))
        .assert()
        .success()
        .stdout(format!(
            r#"$ {}
$ $ {}
$ "#,
            tmp_dir.path().display(),
            level2.display()
        ));
}

#[test]
fn cd_invalid_folder() {
    let tmp_dir = tempdir().unwrap();
    let level2_non_existed = tmp_dir.path().join("level1").join("level2");

    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        // remove path to avoid using cd from path
        .env("PATH", "")
        .current_dir(tmp_dir.path()) // inside tmp path
        .write_stdin(format!(
            r#"pwd
cd {}
pwd"#,
            level2_non_existed.display()
        ))
        .assert()
        .success()
        .stdout(format!(
            r#"$ {}
$ $ {}
$ "#,
            tmp_dir.path().display(),
            tmp_dir.path().display(),
        ))
        .stderr(format!(
            "cd: {}: No such file or directory\n",
            level2_non_existed.display()
        ));
}

#[test]
fn cd_relative() {
    let tmp_dir = tempdir().unwrap();
    let level1 = tmp_dir.path().join("level1");
    let level2 = level1.join("level2");
    fs::create_dir(&level1).unwrap();
    fs::create_dir(&level2).unwrap();

    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        // remove path to avoid using cd from path
        .env("PATH", "")
        .current_dir(tmp_dir.path()) // inside tmp path
        .write_stdin(
            r#"pwd
cd level1
pwd
cd ./level2
pwd
cd ../..
pwd
"#,
        )
        .assert()
        .success()
        .stdout(format!(
            r#"$ {}
$ $ {}
$ $ {}
$ $ {}
$ "#,
            tmp_dir.path().display(),
            level1.display(),
            level2.display(),
            tmp_dir.path().display(),
        ));
}

#[test]
fn cd_tilde() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        // remove path to avoid using cd from path
        .env("PATH", "")
        .write_stdin(
            r#"cd ~
pwd"#,
        )
        .assert()
        .success()
        .stdout(predicate::str::contains("/home/"));
}

#[test]
fn single_quote() {
    Command::cargo_bin("codecrafters-shell")
        .unwrap()
        .write_stdin(
            r#"echo 'shell hello'
echo 'world     test'
echo 'world     example' 'test''script'
"#,
        )
        .assert()
        .success()
        .stdout(
            r#"$ shell hello
$ world     test
$ world     example testscript
$ "#,
        );
}
