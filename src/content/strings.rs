// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
use failure::{err_msg, Error};
use std::collections::linked_list::LinkedList;

use crate::content::Provider;
// -------------------------------------------------------------------------------------------------

/// A data provider that reads strings from the command line arguments.
pub struct StringProvider {
    data: LinkedList<String>,
}

impl StringProvider {
    pub fn new<C>(data: C) -> Self
    where
        C: Into<LinkedList<String>>,
    {
        StringProvider { data: data.into() }
    }
}

impl Provider for StringProvider {
    fn provide(&mut self) -> Result<String, Error> {
        self.data.pop_front().ok_or(err_msg("No more data."))
    }

    fn has_more(&mut self) -> Result<bool, Error> {
        Ok(!self.data.is_empty())
    }
}
