use std::io::{BufReader, BufWriter};
use std::process::{Child, Command, Stdio};

use crate::codec;
use crate::error::{Error, Result};
use crate::message::{IncomingMessage, OutgoingMessage};

/// Manages the pkl server child process and provides message I/O.
pub struct PklProcess {
    child: Child,
    writer: BufWriter<std::process::ChildStdin>,
    reader: BufReader<std::process::ChildStdout>,
}

impl PklProcess {
    /// Spawn a new pkl server process.
    pub fn start() -> Result<Self> {
        Self::start_with_command("pkl")
    }

    /// Spawn a pkl server process with a custom command name/path.
    pub fn start_with_command(pkl_command: &str) -> Result<Self> {
        let mut child = Command::new(pkl_command)
            .arg("server")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| Error::Process(format!("failed to start pkl server: {e}")))?;

        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| Error::Process("failed to capture stdin".into()))?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| Error::Process("failed to capture stdout".into()))?;

        Ok(Self {
            child,
            writer: BufWriter::new(stdin),
            reader: BufReader::new(stdout),
        })
    }

    /// Send a message to the pkl server.
    pub fn send(&mut self, msg: &OutgoingMessage) -> Result<()> {
        codec::encode_message(&mut self.writer, msg)
    }

    /// Receive a message from the pkl server (blocking).
    pub fn recv(&mut self) -> Result<IncomingMessage> {
        codec::decode_message(&mut self.reader)
    }

    /// Kill the pkl server process.
    pub fn kill(&mut self) -> Result<()> {
        self.child.kill().map_err(Error::Io)
    }
}

impl Drop for PklProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}
