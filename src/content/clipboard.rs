// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
use clipboard::{ClipboardContext, ClipboardProvider as SystemClipboard};
use failure::{err_msg, Error};
use std::sync::Mutex;

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

    pub fn set_data(&mut self, data: String) -> Result<(), Error> {
        Self::fix_error(self.clipboard.set_contents(data))?;
        self.ignore = Self::fix_error(self.clipboard.get_contents())?;
        Ok(())
    }

    pub fn get_data(&mut self) -> Result<String, Error> {
        let data = Self::fix_error(self.clipboard.get_contents())?;
        self.ignore = data.clone();
        Ok(data)
    }

    pub fn has_changed(&mut self) -> Result<bool, Error> {
        let data = Self::fix_error(self.clipboard.get_contents())?;
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
