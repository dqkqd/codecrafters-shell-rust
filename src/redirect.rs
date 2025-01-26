use std::{
    fs::OpenOptions,
    io::{self, Write},
};

use anyhow::Result;

use crate::token::RedirectToken;

#[derive(Default)]
pub struct Redirector {
    stdout: Vec<RedirectToken>,
    stderr: Vec<RedirectToken>,
}

impl Redirector {
    pub fn new(redirects: Vec<RedirectToken>) -> Redirector {
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        for r in redirects {
            match r {
                r @ RedirectToken::Stdout(_) => stdout.push(r),
                r @ RedirectToken::Stderr(_) => stderr.push(r),
            }
        }

        Redirector { stdout, stderr }
    }

    pub fn write_stdout<T: AsRef<[u8]>>(&self, msg: T) -> Result<()> {
        let msg = msg.as_ref().trim_ascii_end();
        if self.stdout.is_empty() && !msg.is_empty() {
            let mut stdout = io::stdout().lock();
            stdout.write_all(msg.as_ref())?;
            stdout.write_all(b"\n")?;
        } else {
            write_to_redirected_files(msg, &self.stdout);
        }
        Ok(())
    }

    pub fn write_stderr<T: AsRef<[u8]>>(&self, msg: T) -> Result<()> {
        let msg = msg.as_ref().trim_ascii_end();
        if self.stderr.is_empty() && !msg.is_empty() {
            let mut stderr = io::stderr().lock();
            stderr.write_all(msg.as_ref())?;
            stderr.write_all(b"\n")?;
        } else {
            write_to_redirected_files(msg, &self.stderr);
        }
        Ok(())
    }
}

fn write_to_redirected_files<T: AsRef<[u8]>>(msg: T, redirects: &[RedirectToken]) {
    for r in redirects {
        let path = r.path();
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(path)
        {
            if !msg.as_ref().is_empty() {
                let _ = file.write_all(msg.as_ref());
                let _ = file.write_all(b"\n");
            }
        }
    }
}
