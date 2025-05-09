use std::{
    cell::RefCell,
    fs::File,
    io::{self, Write},
    rc::Rc,
};

pub(crate) fn write_stderr(stderr: &mut [PErr], data: &[u8]) -> anyhow::Result<()> {
    for s in stderr {
        s.write_all_and_flush(data)?;
    }
    Ok(())
}

pub(crate) fn write_stdout(stdout: &mut [POut], data: &[u8]) -> anyhow::Result<()> {
    for s in stdout {
        s.write_all_and_flush(data)?;
    }
    Ok(())
}

pub(crate) type SharedData = Rc<RefCell<Vec<u8>>>;

#[derive(Debug)]
pub(crate) enum PIn {
    File(File),
    Shared(SharedData),
    Empty,
}

#[derive(Debug)]
pub(crate) enum POut {
    File(File),
    Std(io::Stdout),
    Shared(SharedData),
}

#[derive(Debug)]
pub(crate) enum PErr {
    File(File),
    Std(io::Stderr),
    #[allow(dead_code)]
    Shared(SharedData),
}

#[derive(Debug)]
pub(crate) enum PType {
    #[allow(dead_code)]
    In(PIn),
    Out(POut),
    Err(PErr),
}

impl POut {
    fn write_all_and_flush(&mut self, data: &[u8]) -> anyhow::Result<()> {
        match self {
            POut::Std(stdout) => write_all_and_flush(stdout, data)?,
            POut::File(file) => write_all_and_flush(file, data)?,
            POut::Shared(s) => *s.borrow_mut() = data.to_vec(),
        }
        Ok(())
    }
}

impl PErr {
    fn write_all_and_flush(&mut self, data: &[u8]) -> anyhow::Result<()> {
        match self {
            PErr::Std(stderr) => write_all_and_flush(stderr, data)?,
            PErr::File(file) => write_all_and_flush(file, data)?,
            PErr::Shared(s) => *s.borrow_mut() = data.to_vec(),
        }
        Ok(())
    }
}

fn write_all_and_flush<W: Write>(w: &mut W, data: &[u8]) -> anyhow::Result<()> {
    w.write_all(data)?;
    w.flush()?;
    Ok(())
}
