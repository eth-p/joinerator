// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
use failure::Fail;
use regex::Regex;
use serde::Deserialize;
use serde_regex;

#[cfg(not(cached))]
use serde_yaml;

#[cfg(not(cached))]
use serde::Serialize;

#[cfg(not(cached))]
use std::path::Path;

#[cfg(not(cached))]
use std::fs;

// -------------------------------------------------------------------------------------------------

#[cfg_attr(not(cached), derive(Serialize))]
#[derive(Debug, Deserialize)]
pub struct Repertoire {
    pub name: String,
    pub description: String,
    pub glyphs: Vec<Glyph>,
}

#[cfg_attr(not(cached), derive(Serialize))]
#[derive(Debug, Deserialize, Clone)]
pub struct Glyph {
    pub codepoint: char,
    pub position: GlyphPosition,

    #[serde(with = "serde_regex")]
    pub combines: Regex,
}

#[cfg_attr(not(cached), derive(Serialize))]
#[derive(Debug, Deserialize, Hash, PartialEq, Eq, Clone, Copy)]
pub enum GlyphPosition {
    ABOVE,
    BELOW,
    THROUGH,
}

#[derive(Debug, Fail)]
#[allow(dead_code)]
pub enum Error {
    #[fail(display = "failed to deserialize: {}", reason)]
    DeserializeError { reason: String },
}

// -------------------------------------------------------------------------------------------------

impl Repertoire {
    /// Creates a repertoire.
    ///
    /// # Arguments
    /// - `name`       - The repertoire name.
    /// - `description - The repertoire description.
    /// - `glyphs`     - A vector of glyphs for the repertoire.
    #[cfg(not(cached))]
    pub fn new(name: String, description: String, glyphs: Vec<Glyph>) -> Self {
        Repertoire {
            name,
            description,
            glyphs,
        }
    }

    /// Creates a repertoire by deserializing YAML data.
    #[cfg(not(cached))]
    pub fn from_yaml(data: &str) -> Result<Self, Error> {
        serde_yaml::from_str::<Repertoire>(data).map_err(|e| Error::DeserializeError {
            reason: e.to_string(),
        })
    }

    #[cfg(not(cached))]
    pub fn from_file<P>(path: P) -> Result<Self, Error>
    where
        P: AsRef<Path>,
    {
        let path_buf = path.as_ref();
        let path_ext = path_buf.extension().map(|s| s.to_str().unwrap());
        let path_name = path_buf.file_name().unwrap().to_str().unwrap();
        let data = fs::read_to_string(path_buf).map_err(|e| Error::DeserializeError {
            reason: format!("io error: {}", e.to_string()),
        })?;

        if path_ext.is_none() {
            return Err(Error::DeserializeError {
                reason: format!("unable to determine the file type of '{}'", path_name),
            });
        }

        match path_ext {
            Some("yml") | Some("yaml") => Ok(Self::from_yaml(&data)?),
            _ => Err(Error::DeserializeError {
                reason: format!("unable to deserialize the file type of '{}'", path_name),
            }),
        }
    }
}

impl Glyph {
    /// Creates a new glyph definition.
    ///
    /// # Arguments
    /// - `codepoint` - The char representing the glyph.
    /// - `position`  - The position of the glyph relative to other glyphs.
    /// - `combines`  - A regular expression for what other glyphs this will combine onto.
    #[cfg(not(cached))]
    pub fn new(codepoint: char, position: GlyphPosition, combines: Regex) -> Self {
        Glyph {
            codepoint,
            position,
            combines,
        }
    }

    /// Checks if the combining glyph can be applied to a specific glyph.
    pub fn is_applicable(&self, c: char) -> bool {
        let mut buffer = [0; 4];
        self.combines.is_match(c.encode_utf8(&mut buffer))
    }
}
