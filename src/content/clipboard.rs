// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
use clipboard::{ClipboardContext, ClipboardProvider as SystemClipboard};
use failure::{err_msg, Error};

use std::sync::Mutex;
use std::thread::sleep;
use std::time::Duration;

use crate::content::{Consumer, Provider};
// -------------------------------------------------------------------------------------------------

lazy_static! {
    static ref SINGLETON: Mutex<ClipboardSingleton> = Mutex::new(ClipboardSingleton::new());
}

struct ClipboardSingleton {
    clipboard: ClipboardContext,
    ignore: String,
}

impl ClipboardSingleton {
    pub fn new() -> Self {
        ClipboardSingleton {
            clipboard: SystemClipboard::new().unwrap(),
            ignore: String::new(),
        }
    }

    /// Repeatedly attempts to get the contents of the clipboard, retrying every 10 seconds.
    /// This exists because the Windows clipboard appears to fail randomly.
    ///
    /// ## Arguments:
    /// - `tries` - The number of tries before giving up with an error.
    ///
    /// ## Returns:
    /// The result containing the clipboard contents, or the last error that occurred.
    fn get_contents_retry(&mut self, tries: usize) -> Result<String, Box<std::error::Error>> {
        let mut remaining = tries + 1;
        let mut last_err: Option<Box<std::error::Error>> = None;

        while remaining > 0 {
            remaining -= 1;

            match self.clipboard.get_contents() {
                Ok(data) => return Ok(data),
                Err(err) => last_err = Some(err),
            };

            sleep(Duration::from_millis(10));
        }

        Err(last_err.unwrap())
    }

    pub fn set_data(&mut self, data: String) -> Result<(), Error> {
        Self::fix_error(self.clipboard.set_contents(data))?;
        self.ignore = Self::fix_error(self.get_contents_retry(10))?;
        Ok(())
    }

    pub fn get_data(&mut self) -> Result<String, Error> {
        let data = Self::fix_error(self.get_contents_retry(10))?;
        self.ignore = data.clone();
        Ok(data)
    }

    pub fn has_changed(&mut self) -> Result<bool, Error> {
        let data = Self::fix_error(self.get_contents_retry(10))?;
        if data == self.ignore {
            Ok(false)
        } else {
            Ok(true)
        }
    }

    /// A helper function to convert the errors returned by the clipboard library into
    /// failure-compatible error types.
    fn fix_error<T>(data: Result<T, Box<std::error::Error>>) -> Result<T, Error>
    where
        T: std::fmt::Debug,
    {
        if data.is_err() {
            Err(err_msg(data.unwrap_err().to_string()))
        } else {
            Ok(data.unwrap())
        }
    }
}

// -------------------------------------------------------------------------------------------------

pub struct ClipboardProvider {}

impl ClipboardProvider {
    pub fn new() -> Self {
        ClipboardProvider {}
    }
}

impl Provider for ClipboardProvider {
    fn provide(&mut self) -> Result<String, Error> {
        let mut lock = SINGLETON.lock();
        let clipboard = lock.as_mut().unwrap();
        clipboard.get_data()
    }

    fn has_more(&mut self) -> Result<bool, Error> {
        let mut lock = SINGLETON.lock();
        let clipboard = lock.as_mut().unwrap();
        clipboard.has_changed()
    }
}

pub struct ClipboardConsumer {}

impl ClipboardConsumer {
    pub fn new() -> Self {
        ClipboardConsumer {}
    }
}

impl Consumer for ClipboardConsumer {
    fn consume(&mut self, str: String) -> Result<(), Error> {
        let mut lock = SINGLETON.lock();
        let clipboard = lock.as_mut().unwrap();
        clipboard.set_data(str)
    }
}
