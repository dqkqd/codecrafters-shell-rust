use std::io::{self, Write};

#[allow(unused)]
pub(crate) enum PIn {
    Std(io::Stdin),
}

pub(crate) enum POut {
    Std(io::Stdout),
}

#[allow(unused)]
pub(crate) enum PErr {
    Std(io::Stderr),
}

#[allow(unused)]
impl PErr {
    pub fn write_all_and_flush(&mut self, data: &[u8]) -> anyhow::Result<()> {
        match self {
            PErr::Std(stderr) => write_all_and_flush(stderr, data)?,
        }
        Ok(())
    }
}

impl POut {
    pub fn write_all_and_flush(&mut self, data: &[u8]) -> anyhow::Result<()> {
        match self {
            POut::Std(stdout) => write_all_and_flush(stdout, data)?,
        }
        Ok(())
    }
}

fn write_all_and_flush<W: Write>(w: &mut W, data: &[u8]) -> anyhow::Result<()> {
    w.write_all(data)?;
    w.flush()?;
    Ok(())
}
