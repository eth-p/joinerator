// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
use failure::Error;
use std::io;
use std::io::{Read, Stdin, Stdout, Write};

use crate::content::{Consumer, Provider};
// -------------------------------------------------------------------------------------------------

/// A data provider that reads from standard input.
pub struct StdinProvider {
    stream: Option<Stdin>,
    buffer: Option<String>,
}

impl StdinProvider {
    pub fn new() -> Self {
        StdinProvider {
            stream: None,
            buffer: None,
        }
    }

    fn reopen(&mut self) -> &mut Stdin {
        if self.stream.is_none() {
            self.stream = Some(io::stdin());
        }

        self.stream.as_mut().unwrap()
    }
}

impl Provider for StdinProvider {
    fn provide(&mut self) -> Result<String, Error> {
        let mut buffer = self.buffer.take().unwrap_or_else(|| String::new());
        match self.reopen().read_to_string(&mut buffer) {
            Err(err) => return Err(err.into()),
            Ok(_v) => Ok(buffer),
        }
    }

    fn has_more(&mut self) -> Result<bool, Error> {
        let mut buffer = self.buffer.take().unwrap_or_else(|| String::new());
        let result = self
            .reopen()
            .read_line(&mut buffer)
            .map(|n| n > 0)
            .map_err(|e| e.into());

        self.buffer.replace(buffer);
        result
    }
}

// -------------------------------------------------------------------------------------------------

/// A data provider that writes to standard output.
pub struct StdoutConsumer {
    stream: Stdout,
}

impl StdoutConsumer {
    pub fn new() -> Self {
        StdoutConsumer {
            stream: io::stdout(),
        }
    }
}

impl Consumer for StdoutConsumer {
    fn consume(&mut self, str: String) -> Result<(), Error> {
        let bytes = str.as_bytes();
        let mut offset = 0;

        while offset < bytes.len() {
            match self.stream.write(str.as_bytes()) {
                Err(e) => return Err(e.into()),
                Ok(n) => offset += n,
            }
        }

        Ok(())
    }
}
