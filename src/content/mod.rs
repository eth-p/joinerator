// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
pub mod streams;
pub mod strings;

#[cfg(feature = "clipboard_support")]
pub mod clipboard;

// -------------------------------------------------------------------------------------------------
use failure::Error;
// -------------------------------------------------------------------------------------------------

/// A content provider.
/// This provides one or more strings to be processed by Joinerator.
pub trait Provider {
    fn provide(&mut self) -> Result<String, Error>;
    fn has_more(&mut self) -> Result<bool, Error>;
}

/// A content consumer.
/// This accepts one or more strings that have been processed by Joinerator.
pub trait Consumer {
    fn consume(&mut self, str: String) -> Result<(), Error>;
}
