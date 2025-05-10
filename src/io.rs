use std::{
    fs::File,
    io::{self, Read, Write},
    sync::mpsc::{Receiver, Sender},
};

use anyhow::Result;

pub(crate) fn write_stderr(stderr: &mut [PErr], data: &[u8]) -> Result<()> {
    for s in stderr {
        s.consume(data)?;
    }
    Ok(())
}

pub(crate) fn write_stdout(stdout: &mut [POut], data: &[u8]) -> Result<()> {
    for s in stdout {
        s.consume(data)?;
    }
    Ok(())
}

#[derive(Debug)]
pub(crate) enum PIn {
    File(File),
    Pipe(Receiver<Vec<u8>>),
    Empty,
}

#[derive(Debug)]
pub(crate) enum POut {
    File(File),
    Std(io::Stdout),
    Pipe(Sender<Vec<u8>>),
}

#[derive(Debug)]
pub(crate) enum PErr {
    File(File),
    Std(io::Stderr),
    #[allow(dead_code)]
    Pipe(Sender<Vec<u8>>),
}

#[derive(Debug)]
pub(crate) enum PType {
    #[allow(dead_code)]
    In(PIn),
    Out(POut),
    Err(PErr),
}

impl PIn {
    /// Write all the remaining data into writer
    pub(crate) fn send_to_writer<W: Write>(&mut self, mut writer: W) -> Result<()> {
        match self {
            PIn::File(file) => {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                write_all_and_flush(&mut writer, &data)?;
                Ok(())
            }
            PIn::Pipe(receiver) => {
                while let Ok(data) = receiver.recv() {
                    write_all_and_flush(&mut writer, &data)?;
                }
                Ok(())
            }
            PIn::Empty => Ok(()),
        }
    }
}

impl POut {
    /// Get all data and send into `POut`
    fn consume(&mut self, data: &[u8]) -> Result<()> {
        match self {
            POut::Std(stdout) => write_all_and_flush(stdout, data)?,
            POut::File(file) => write_all_and_flush(file, data)?,
            POut::Pipe(sender) => sender.send(data.to_vec())?,
        }
        Ok(())
    }
}

impl PErr {
    /// Get all data and send into `PErr`
    fn consume(&mut self, data: &[u8]) -> Result<()> {
        match self {
            PErr::Std(stderr) => write_all_and_flush(stderr, data)?,
            PErr::File(file) => write_all_and_flush(file, data)?,
            PErr::Pipe(sender) => sender.send(data.to_vec())?,
        }
        Ok(())
    }
}

fn write_all_and_flush<W: Write>(w: &mut W, data: &[u8]) -> Result<()> {
    w.write_all(data)?;
    w.flush()?;
    Ok(())
}
