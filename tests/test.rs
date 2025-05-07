use std::{
    fs::{self, File},
    path::{Path, PathBuf},
};

use anyhow::Result;
use assert_cmd::Command;
use serde::{Deserialize, Serialize};
use tempfile::tempdir;

procspawn::enable_test_support!();

#[derive(Default, Serialize, Deserialize)]
struct TestOption {
    env: Option<(String, String)>,
    current_dir: Option<PathBuf>,
}

fn run_test(input: &str, expected: &str, opt: TestOption) -> Result<()> {
    let mut input = input.trim_start();
    if input.ends_with('\n') {
        input = &input[..input.len() - 1]
    }
    let input = input.to_string();

    let expected = expected.trim_start().replace("\n", "\r\n").to_string();

    let output = procspawn::spawn((input, expected, opt), |(input, expected, opt)| {
        if let Some((k, v)) = opt.env {
            std::env::set_var(k, v);
        }
        if let Some(dir) = opt.current_dir {
            std::env::set_current_dir(dir).unwrap();
        }
        run_test_internal(&input, &expected).unwrap();
    });
    output.join()?;
    Ok(())
}

fn run_test_internal(input: &str, expected: &str) -> Result<()> {
    let command = Command::cargo_bin("codecrafters-shell")?;
    let path = Path::new(command.get_program()).to_str().unwrap();
    let mut p = rexpect::spawn(path, Some(50))?;

    p.send_line(input)?;
    p.exp_string(expected)?;

    p.send_control('c')?;
    p.exp_eof()?;
    Ok(())
}

#[test]
fn print_a_prompt() -> Result<()> {
    run_test("", "$ ", TestOption::default())
}

#[test]
fn handle_invalid_commands() -> Result<()> {
    run_test(
        "some_command",
        r#"
$ some_command
some_command: command not found
$ "#,
        TestOption::default(),
    )
}

#[test]
fn repl() -> Result<()> {
    run_test(
        r#"
command1
command2
"#,
        r#"
$ command1
command1: command not found
$ command2
command2: command not found
$ "#,
        TestOption::default(),
    )
}

#[test]
fn repl_empty() -> Result<()> {
    run_test(
        r#"
command1

command2
"#,
        r#"
$ command1
command1: command not found
$ 
$ command2
command2: command not found
$ "#,
        TestOption::default(),
    )
}

#[test]
fn exit() -> Result<()> {
    run_test(
        r#"
command1
exit 0
command2
"#,
        r#"
$ command1
command1: command not found
$ "#,
        // remove `exit` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn echo_one() -> Result<()> {
    run_test(
        r#"
echo 123
"#,
        r#"
$ echo 123
123
$ "#,
        // remove `echo` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn echo_many() -> Result<()> {
    run_test(
        r#"
echo 1 2 3
echo 4  5   6
"#,
        r#"
$ echo 1 2 3
1 2 3
$ echo 4  5   6
4 5 6
$ "#,
        // remove `echo` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn type_one() -> Result<()> {
    run_test(
        r#"
type echo
type exit
type type
type invalid_command
"#,
        r#"
$ type echo
echo is a shell builtin
$ type exit
exit is a shell builtin
$ type type
type is a shell builtin
$ type invalid_command
invalid_command: not found
$ "#,
        // remove `type` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn type_many() -> Result<()> {
    run_test(
        r#"
type echo exit type invalid_command
"#,
        r#"
$ type echo exit type invalid_command
echo is a shell builtin
exit is a shell builtin
type is a shell builtin
invalid_command: not found
$ "#,
        // remove `type` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn type_path() -> Result<()> {
    let tmp_dir = tempdir().unwrap();
    let executable_path = tmp_dir.path().join("my_executable");
    File::create(&executable_path).unwrap();

    run_test(
        r#"
type my_executable
"#,
        &format!(
            r#"
$ type my_executable
my_executable is {}
$ "#,
            executable_path.display()
        ),
        // remove `type` builtin from PATH
        // add executable path to PATH
        TestOption {
            env: Some(("PATH".into(), tmp_dir.path().to_str().unwrap().into())),
            current_dir: None,
        },
    )
}

#[test]
fn path_exec() -> Result<()> {
    let tmp_dir = tempdir().unwrap();
    File::create(tmp_dir.path().join("file1")).unwrap();
    File::create(tmp_dir.path().join("file2")).unwrap();
    File::create(tmp_dir.path().join("file3")).unwrap();

    run_test(
        &format!("ls {}", tmp_dir.path().display()),
        &format!(
            r#"
$ ls {}
file1
file2
file3
$ "#,
            tmp_dir.path().display()
        ),
        TestOption::default(),
    )
}

#[test]
fn pwd() -> Result<()> {
    let tmp_dir = tempdir().unwrap();

    run_test(
        "pwd",
        &format!(
            r#"
$ pwd
{}
$ "#,
            tmp_dir.path().display()
        ),
        // remove `pwd` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: Some(tmp_dir.path().to_path_buf()),
        },
    )
}

#[test]
fn cd() -> Result<()> {
    let tmp_dir = tempdir().unwrap();
    let level2 = tmp_dir.path().join("level1").join("level2");
    fs::create_dir_all(&level2).unwrap();

    run_test(
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
$ cd {}
$ pwd
{}
$ cd {}
$ pwd
{}
$ "#,
            tmp_dir.path().display(),
            tmp_dir.path().display(),
            level2.display(),
            level2.display(),
        ),
        // remove `cd` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn cd_invalid_folder() -> Result<()> {
    let tmp_dir = tempdir().unwrap();
    let level2_non_existed = tmp_dir.path().join("level1").join("level2");

    run_test(
        &format!(
            r#"
cd {}
"#,
            level2_non_existed.display(),
        ),
        &format!(
            r#"
$ cd {}
cd: {}: No such file or directory
$ "#,
            level2_non_existed.display(),
            level2_non_existed.display(),
        ),
        // remove `cd` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn cd_relative() -> Result<()> {
    let tmp_dir = tempdir().unwrap();
    let level1 = tmp_dir.path().join("level1");
    let level2 = level1.join("level2");
    fs::create_dir(&level1).unwrap();
    fs::create_dir(&level2).unwrap();

    run_test(
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
$ cd {}
$ cd level1
$ pwd
{}
$ cd ./level2
$ pwd
{}
$ cd ../..
$ pwd
{}
$ "#,
            tmp_dir.path().display(),
            level1.display(),
            level2.display(),
            tmp_dir.path().display()
        ),
        // remove `cd` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn cd_tilde() -> Result<()> {
    run_test(
        r#"
cd ~
pwd
"#,
        r#"
$ cd ~
$ pwd
/home/"#,
        // remove `cd` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn single_quote() -> Result<()> {
    run_test(
        r#"
echo 'shell hello'
echo 'world     test'
echo 'world     example' 'test''script'
"#,
        r#"
$ echo 'shell hello'
shell hello
$ echo 'world     test'
world     test
$ echo 'world     example' 'test''script'
world     example testscript
$ "#,
        // remove `echo` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn double_quote() -> Result<()> {
    run_test(
        r#"
echo "shell hello"
echo "world\$     test"
echo "hello\" world"
"#,
        r#"
$ echo "shell hello"
shell hello
$ echo "world\$     test"
world$     test
$ echo "hello\" world"
hello" world
$ "#,
        // remove `echo` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn backslash_outside_quotes() -> Result<()> {
    run_test(
        r#"
echo "before\   after"
echo world\ \ \ \ \ \ script
"#,
        r#"
$ echo "before\   after"
before\   after
$ echo world\ \ \ \ \ \ script
world      script
$ "#,
        // remove `echo` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn backslash_within_single_quotes() -> Result<()> {
    run_test(
        r#"
echo 'shell\\\nscript'
echo 'example\"testhello\"shell'
"#,
        r#"
$ echo 'shell\\\nscript'
shell\\\nscript
$ echo 'example\"testhello\"shell'
example\"testhello\"shell
$ "#,
        // remove `echo` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn backslash_within_double_quotes() -> Result<()> {
    run_test(
        r#"
echo "hello'script'\\n'world"
echo "hello\"insidequotes"script\"
"#,
        r#"
$ echo "hello'script'\\n'world"
hello'script'\n'world
$ echo "hello\"insidequotes"script\"
hello"insidequotesscript"
$ "#,
        // remove `echo` builtin from PATH
        TestOption {
            env: Some(("PATH".into(), "".into())),
            current_dir: None,
        },
    )
}

#[test]
fn redirect_output_stdout() -> Result<()> {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");

    run_test(
        &format!(
            r#"
echo > {} "hello world"
cat {}
"#,
            output.display(),
            output.display(),
        ),
        &format!(
            r#"
$ echo > {} "hello world"
$ cat {}
hello world
$ "#,
            output.display(),
            output.display(),
        ),
        TestOption::default(),
    )
}

#[test]
fn redirect_output_stdout_many() -> Result<()> {
    let tmp_dir = tempdir().unwrap();
    let output1 = tmp_dir.path().join("output1");
    let output2 = tmp_dir.path().join("output2");

    run_test(
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
        &format!(
            r#"
$ cat {}
hello world
$ cat {}
hello world
$ "#,
            output1.display(),
            output2.display(),
        ),
        TestOption::default(),
    )
}

#[test]
fn redirect_output_stderr() -> Result<()> {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");

    run_test(
        &format!(
            r#"
cd 2> {} invalid_path
cat {}
"#,
            output.display(),
            output.display(),
        ),
        &format!(
            r#"
$ cd 2> {} invalid_path
$ cat {}
cd: invalid_path: No such file or directory
$ "#,
            output.display(),
            output.display(),
        ),
        TestOption::default(),
    )
}

#[test]
fn redirect_append_output_stdout() -> Result<()> {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");

    run_test(
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
        &format!(
            r#"
$ echo > {} "hello"
$ echo >> {} "world"
$ cat {}
hello
world
$ "#,
            output.display(),
            output.display(),
            output.display(),
        ),
        TestOption::default(),
    )
}

#[test]
fn redirect_append_output_stdout_many() -> Result<()> {
    let tmp_dir = tempdir().unwrap();
    let output1 = tmp_dir.path().join("output1");
    let output2 = tmp_dir.path().join("output2");

    run_test(
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
        &format!(
            r#"
$ echo >> {} >> {} "hello world"
$ cat {}
hello world
$ cat {}
hello world
$ "#,
            output1.display(),
            output2.display(),
            output1.display(),
            output2.display(),
        ),
        TestOption::default(),
    )
}

#[test]
fn redirect_append_output_stderr() -> Result<()> {
    let tmp_dir = tempdir().unwrap();
    let output = tmp_dir.path().join("output");

    run_test(
        &format!(
            r#"
cd 1>> {} invalid_path
cd 2>> {} invalid_path
cat {}
"#,
            output.display(),
            output.display(),
            output.display(),
        ),
        &format!(
            r#"
$ cd 1>> {} invalid_path
cd: invalid_path: No such file or directory
$ cd 2>> {} invalid_path
$ cat {}
cd: invalid_path: No such file or directory
$ "#,
            output.display(),
            output.display(),
            output.display(),
        ),
        TestOption::default(),
    )
}
