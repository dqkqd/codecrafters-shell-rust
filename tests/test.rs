use anyhow::Result;
use predicates::prelude::predicate;
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use assert_cmd::Command;
use tempfile::tempdir;

#[derive(Default)]
struct TestOption {
    env: Option<(String, String)>,
    current_dir: Option<PathBuf>,
    err: bool,
}

impl TestOption {
    fn env(mut self, k: &str, v: &str) -> TestOption {
        self.env = Some((k.into(), v.into()));
        self
    }
    fn current_dir(mut self, dir: PathBuf) -> TestOption {
        self.current_dir = Some(dir);
        self
    }
    fn no_path() -> TestOption {
        TestOption::default().env("PATH", "")
    }
    fn err(mut self) -> TestOption {
        self.err = true;
        self
    }
}

fn check_contains(input: &str, expected: &str, opt: TestOption) {
    let input = input.to_string();
    let expected = expected.trim().to_string() + "\n";

    let mut command = &mut Command::cargo_bin("codecrafters-shell").unwrap();

    if let Some((k, v)) = opt.env {
        command = command.env(k, v);
    }
    if let Some(dir) = &opt.current_dir {
        command = command.current_dir(dir);
    }
    let assert = command.write_stdin(input).assert().success();

    if opt.err {
        assert.stderr(predicate::str::ends_with(expected));
    } else {
        assert.stdout(predicate::str::ends_with(expected));
    }
}

fn check_complete(input: &str, expected: &str) -> Result<()> {
    let command = Command::cargo_bin("codecrafters-shell")?;
    let path = Path::new(command.get_program()).to_str().unwrap();
    let mut p = rexpect::spawn(path, Some(50))?;

    p.send(input)?;
    p.flush()?;
    p.exp_string(expected)?;

    p.send_control('c')?;
    p.exp_eof()?;
    Ok(())
}

fn check_complete_exec(input: &str, expected: &str) -> Result<()> {
    let command = Command::cargo_bin("codecrafters-shell")?;
    let path = Path::new(command.get_program()).to_str().unwrap();
    let mut p = rexpect::spawn(path, Some(50))?;

    p.send_line(input)?;
    p.flush()?;
    p.exp_string(expected)?;

    p.send_control('c')?;
    p.exp_eof()?;
    Ok(())
}

#[test]
fn handle_invalid_commands() {
    check_contains(
        "some_command",
        "some_command: command not found",
        TestOption::default(),
    );
}

#[test]
fn repl() {
    check_contains(
        r#"
command1
command2
"#,
        r#"
command1: command not found
command2: command not found
"#,
        TestOption::default(),
    )
}

#[test]
fn exit() {
    check_contains(
        r#"
command1
exit 0
command2
"#,
        r#"
command1: command not found
"#,
        TestOption::no_path(),
    )
}

#[test]
fn echo_one() {
    check_contains("echo 123", "123", TestOption::no_path())
}

#[test]
fn echo_many() {
    check_contains(
        r#"
echo 1 2 3
echo 4  5   6
"#,
        r#"
1 2 3
4 5 6
"#,
        TestOption::no_path(),
    )
}

#[test]
fn type_one() {
    check_contains(
        r#"
type echo
type exit
type type
type history
type invalid_command
"#,
        r#"
echo is a shell builtin
exit is a shell builtin
type is a shell builtin
history is a shell builtin
invalid_command: not found
"#,
        TestOption::no_path(),
    )
}

#[test]
fn type_many() {
    check_contains(
        r#"
type echo exit type invalid_command
"#,
        r#"
echo is a shell builtin
exit is a shell builtin
type is a shell builtin
invalid_command: not found
"#,
        TestOption::no_path(),
    )
}

#[test]
fn type_path() {
    let tmp_dir = tempdir().unwrap();
    let executable_path = tmp_dir.path().join("my_executable");
    File::create(&executable_path).unwrap();

    check_contains(
        "type my_executable",
        &format!("my_executable is {}", executable_path.display()),
        TestOption::default().env("PATH", tmp_dir.path().to_str().unwrap()),
    )
}

#[test]
fn path_exec() {
    let tmp_dir = tempdir().unwrap();
    File::create(tmp_dir.path().join("file1")).unwrap();
    File::create(tmp_dir.path().join("file2")).unwrap();
    File::create(tmp_dir.path().join("file3")).unwrap();

    check_contains(
        &format!("ls {}", tmp_dir.path().display()),
        r#"
file1
file2
file3
"#,
        TestOption::default(),
    )
}

#[test]
fn pwd() {
    let tmp_dir = tempdir().unwrap();

    check_contains(
        "pwd",
        &format!("{}", tmp_dir.path().display()),
        TestOption::no_path().current_dir(tmp_dir.into_path()),
    )
}

#[test]
fn cd() {
    let tmp_dir = tempdir().unwrap();
    let level2 = tmp_dir.path().join("level1").join("level2");
    fs::create_dir_all(&level2).unwrap();

    check_contains(
        &format!(
            r#"
cd {}
pwd
cd {}
pwd
"#,
            tmp_dir.path().display(),
            level2.display(),
        ),
        &format!(
            r#"
{}
{}
"#,
            tmp_dir.path().display(),
            level2.display(),
        ),
        TestOption::no_path(),
    )
}

#[test]
fn cd_invalid_folder() {
    let tmp_dir = tempdir().unwrap();
    let level2_non_existed = tmp_dir.path().join("level1").join("level2");

    check_contains(
        &format!(
            r#"
cd {}
"#,
            level2_non_existed.display(),
        ),
        &format!(
            r#"
cd: {}: No such file or directory
"#,
            level2_non_existed.display(),
        ),
        TestOption::no_path().err(),
    )
}

#[test]
fn cd_relative() {
    let tmp_dir = tempdir().unwrap();
    let level1 = tmp_dir.path().join("level1");
    let level2 = level1.join("level2");
    fs::create_dir(&level1).unwrap();
    fs::create_dir(&level2).unwrap();

    check_contains(
        &format!(
            r#"
cd {}
cd level1
pwd
cd ./level2
pwd
cd ../..
pwd
"#,
            tmp_dir.path().display(),
        ),
        &format!(
            r#"
{}
{}
{}
"#,
            level1.display(),
            level2.display(),
            tmp_dir.path().display()
        ),
        TestOption::no_path(),
    )
}

#[test]
fn cd_tilde() {
    check_contains(
        r#"
cd ~
pwd
"#,
        &format!(
            "
/home/{}",
            std::env::var("USER").unwrap()
        ),
        TestOption::no_path(),
    )
}

#[test]
fn single_quote() {
    check_contains(
        r#"
echo 'shell hello'
echo 'world     test'
echo 'world     example' 'test''script'
"#,
        r#"
shell hello
world     test
world     example testscript
"#,
        TestOption::no_path(),
    )
}

#[test]
fn double_quote() {
    check_contains(
        r#"
echo "shell hello"
echo "world\$     test"
echo "hello\" world"
"#,
        r#"
shell hello
world$     test
hello" world
"#,
        TestOption::no_path(),
    )
}

#[test]
fn backslash_outside_quotes() {
    check_contains(
        r#"
echo "before\   after"
echo world\ \ \ \ \ \ script
"#,
        r#"
before\   after
world      script
"#,
        TestOption::no_path(),
    )
}

#[test]
fn backslash_within_single_quotes() {
    check_contains(
        r#"
echo 'shell\\\nscript'
echo 'example\"testhello\"shell'
"#,
        r#"
shell\\\nscript
example\"testhello\"shell
"#,
        TestOption::no_path(),
    )
}

#[test]
fn backslash_within_double_quotes() {
    check_contains(
        r#"
echo "hello'script'\\n'world"
echo "hello\"insidequotes"script\"
"#,
        r#"
hello'script'\n'world
hello"insidequotesscript"
"#,
        TestOption::no_path(),
    )
}

#[test]
fn redirect_output_stdout() {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");

    check_contains(
        &format!(
            r#"
echo > {} "hello world"
cat {}
"#,
            output.display(),
            output.display(),
        ),
        "hello world",
        TestOption::default(),
    )
}

#[test]
fn redirect_output_stdout_many() {
    let tmp_dir = tempdir().unwrap();
    let output1 = tmp_dir.path().join("output1");
    let output2 = tmp_dir.path().join("output2");

    check_contains(
        &format!(
            r#"
echo > {} > {} "hello world"
cat {}
cat {}
"#,
            output1.display(),
            output2.display(),
            output1.display(),
            output2.display(),
        ),
        r#"
hello world
hello world
 "#,
        TestOption::default(),
    )
}

#[test]
fn redirect_output_stderr() {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");

    check_contains(
        &format!(
            r#"
cd 2> {} invalid_path
cat {}
"#,
            output.display(),
            output.display(),
        ),
        r#"
cd: invalid_path: No such file or directory
"#,
        TestOption::default(),
    )
}

#[test]
fn redirect_append_output_stdout() {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");

    check_contains(
        &format!(
            r#"
echo > {} "hello"
echo >> {} "world"
cat {}
"#,
            output.display(),
            output.display(),
            output.display(),
        ),
        r#"
hello
world
"#,
        TestOption::default(),
    )
}

#[test]
fn redirect_append_output_stdout_many() {
    let tmp_dir = tempdir().unwrap();
    let output1 = tmp_dir.path().join("output1");
    let output2 = tmp_dir.path().join("output2");

    check_contains(
        &format!(
            r#"
echo >> {} >> {} "hello world"
cat {}
cat {}
"#,
            output1.display(),
            output2.display(),
            output1.display(),
            output2.display(),
        ),
        r#"
hello world
hello world
"#,
        TestOption::default(),
    )
}

#[test]
fn redirect_append_output_stderr() {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");

    check_contains(
        &format!(
            r#"
echo >{} first
cd 2>> {} invalid_path
cat {}
"#,
            output.display(),
            output.display(),
            output.display(),
        ),
        r#"
first
cd: invalid_path: No such file or directory
"#,
        TestOption::default(),
    )
}

#[test]
fn complete_builtin() -> Result<()> {
    check_complete("ec\t", "echo ")?;
    check_complete("ech\t", "echo ")?;
    check_complete("exi\t", "exit ")?;
    Ok(())
}

#[test]
fn complete_builtin_args() -> Result<()> {
    check_complete_exec("typ\techo", "echo is a shell builtin")?;
    Ok(())
}

#[test]
fn complete_builtin_missing() -> Result<()> {
    check_complete("haha\t", "\x07")?;
    Ok(())
}

#[test]
fn complete_path() -> Result<()> {
    check_complete("cargo-fm\t", "cargo-fmt ")?;
    Ok(())
}

#[test]
fn complete_many() -> Result<()> {
    check_complete("exp\t\t", "\x07")?;
    check_complete("exp\t\t", "expand  expiry  expr")?;
    Ok(())
}

#[test]
fn complete_partial() -> Result<()> {
    check_complete("car\t", "cargo")?;
    Ok(())
}

#[test]
fn redirect_input() {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");

    check_contains(
        &format!(
            r#"
echo > {} "hello world"
tail < {}
"#,
            output.display(),
            output.display(),
        ),
        "hello world",
        TestOption::default(),
    )
}

#[test]
fn pipe_exec() {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");
    check_contains(
        &format!(
            r#"
echo > {} hello
cat {} | wc
"#,
            output.display(),
            output.display(),
        ),
        "      1       1       6",
        TestOption::default(),
    )
}

#[test]
fn pipe_exec_wait() {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");
    check_contains(
        &format!(
            r#"
echo > {} hello
tail -f {} | head -n 1 | wc
"#,
            output.display(),
            output.display(),
        ),
        "      1       1       6",
        TestOption::default(),
    )
}

#[test]
fn pipe_exec_non_endline() {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");
    fs::write(&output, "hello").unwrap();

    check_contains(
        &format!(
            r#"
cat {} | wc
"#,
            output.display(),
        ),
        "      0       1       5",
        TestOption::default(),
    )
}
