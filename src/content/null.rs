// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
use failure::Error;

use crate::content::Consumer;

// -------------------------------------------------------------------------------------------------

pub struct NullConsumer {}

impl NullConsumer {
    pub fn new() -> Self {
        NullConsumer {}
    }
}

impl Consumer for NullConsumer {
    fn consume(&mut self, str: String) -> Result<(), Error> {
        Ok(())
    }
}
