// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
use failure::Error;
use regex::Regex;

use crate::transform::Transformer;
// -------------------------------------------------------------------------------------------------

lazy_static! {
    static ref REGEX_SUFFIXIZER: Regex = Regex::new("(?i)([aeuio])cky\\b").unwrap();
    static ref REGEX_ERRY: Regex = Regex::new("(?i)e([r]+)y").unwrap();
    static ref REGEX_BWU: Regex = Regex::new("(?i)\\bbu([t])").unwrap();
    static ref REGEX_R: Regex = Regex::new("(?i)r\\B").unwrap();
    static ref REGEX_LOO: Regex = Regex::new("(?i)loo").unwrap();
    static ref REGEX_WHA: Regex = Regex::new("(?i)\\bwha").unwrap();
    static ref REGEX_OH: Regex = Regex::new("(?i)\\boh\\b").unwrap();
    static ref REGEX_FU: Regex = Regex::new("(?i)\\b(f)(u)").unwrap();
    static ref REGEX_LI: Regex = Regex::new("(?i)\\bl(i)").unwrap();
    static ref REGEX_TT: Regex = Regex::new("(?i)tt").unwrap();
    static ref REGEX_VY: Regex = Regex::new("(?i)\\Bvy\\b").unwrap();
    static ref REGEX_GOD: Regex = Regex::new("(?i)\\bgod").unwrap();
}

/// A transformer that UwU-izes text.
pub struct TransformUwuize {}

impl TransformUwuize {
    pub fn new() -> Self {
        TransformUwuize {}
    }
}

impl Transformer for TransformUwuize {
    fn transform(&mut self, input: String) -> Result<String, Error> {
        let replaced = REGEX_SUFFIXIZER.replace_all(&input, "${1}cky-w${1}cky");
        let replaced = REGEX_FU.replace_all(&replaced, "${1}w${2}");
        let replaced = REGEX_LI.replace_all(&replaced, "w${1}");
        let replaced = REGEX_TT.replace_all(&replaced, "dd");
        let replaced = REGEX_ERRY.replace_all(&replaced, "e${1}${1}y");
        let replaced = REGEX_BWU.replace_all(&replaced, "bwu${1}");
        let replaced = REGEX_R.replace_all(&replaced, "w${1}");
        let replaced = REGEX_LOO.replace_all(&replaced, "woo");
        let replaced = REGEX_WHA.replace_all(&replaced, "wu");
        let replaced = REGEX_OH.replace_all(&replaced, "owh");
        let replaced = REGEX_VY.replace_all(&replaced, "vwy");
        let replaced = REGEX_GOD.replace_all(&replaced, "gawd");
        Ok(replaced.to_string())
    }
}
