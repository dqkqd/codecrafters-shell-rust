use std::{
    fs::File,
    io::{self, Read, Write},
    sync::mpsc::{Receiver, Sender},
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
    pub(crate) fn write<W: Write>(&mut self, mut writer: W) -> anyhow::Result<()> {
        match self {
            PIn::File(file) => {
                let mut data = Vec::new();
                file.read_to_end(&mut data)?;
                writer.write_all(&data)?;
                Ok(())
            }
            PIn::Pipe(receiver) => {
                while let Ok(data) = receiver.recv() {
                    writer.write_all(&data)?;
                }
                Ok(())
            }
            PIn::Empty => Ok(()),
        }
    }
}

impl POut {
    // TODO: rename
    fn write_all_and_flush(&mut self, data: &[u8]) -> anyhow::Result<()> {
        match self {
            POut::Std(stdout) => write_all_and_flush(stdout, data)?,
            POut::File(file) => write_all_and_flush(file, data)?,
            POut::Pipe(sender) => sender.send(data.to_vec())?,
        }
        Ok(())
    }
}

impl PErr {
    fn write_all_and_flush(&mut self, data: &[u8]) -> anyhow::Result<()> {
        match self {
            PErr::Std(stderr) => write_all_and_flush(stderr, data)?,
            PErr::File(file) => write_all_and_flush(file, data)?,
            PErr::Pipe(sender) => sender.send(data.to_vec())?,
        }
        Ok(())
    }
}

fn write_all_and_flush<W: Write>(w: &mut W, data: &[u8]) -> anyhow::Result<()> {
    w.write_all(data)?;
    w.flush()?;
    Ok(())
}
