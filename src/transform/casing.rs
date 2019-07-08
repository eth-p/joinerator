// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
use failure::Error;
use rand::rngs::ThreadRng;
use rand::{thread_rng, Rng};

use crate::transform::Transformer;
// -------------------------------------------------------------------------------------------------

/// A transformer that converts everything to uppercase.
pub struct TransformUpperCase {}

impl TransformUpperCase {
    pub fn new() -> Self {
        TransformUpperCase {}
    }
}

impl Transformer for TransformUpperCase {
    fn transform(&mut self, input: String) -> Result<String, Error> {
        Ok(input.to_uppercase())
    }
}

// -------------------------------------------------------------------------------------------------

/// A transformer that converts everything to lowercase.
pub struct TransformLowerCase {}

impl TransformLowerCase {
    pub fn new() -> Self {
        TransformLowerCase {}
    }
}

impl Transformer for TransformLowerCase {
    fn transform(&mut self, input: String) -> Result<String, Error> {
        Ok(input.to_lowercase())
    }
}

// -------------------------------------------------------------------------------------------------

/// A transformer that converts everything to a random case.
pub struct TransformRandomCase {
    random: ThreadRng,
}

impl TransformRandomCase {
    pub fn new() -> Self {
        TransformRandomCase {
            random: thread_rng(),
        }
    }
}

impl Transformer for TransformRandomCase {
    fn transform(&mut self, input: String) -> Result<String, Error> {
        let mut buffer = String::new();
        for char in input.chars() {
            let str: String = if self.random.gen_bool(1.0 / 2.0) {
                char.to_uppercase().collect()
            } else {
                char.to_lowercase().collect()
            };

            buffer.push_str(&str[..]);
        }

        Ok(buffer)
    }
}

// -------------------------------------------------------------------------------------------------

/// A transformer that makes only vowels uppercase.
pub struct TransformVowelCase {}

impl TransformVowelCase {
    pub fn new() -> Self {
        TransformVowelCase {}
    }
}

impl Transformer for TransformVowelCase {
    fn transform(&mut self, input: String) -> Result<String, Error> {
        Ok(input
            .to_lowercase()
            .replace("a", "A")
            .replace("e", "E")
            .replace("i", "I")
            .replace("o", "O")
            .replace("u", "U")
            .replace("y", "Y"))
    }
}
