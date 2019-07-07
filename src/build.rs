// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// This is a script that generates the cbor serde cache used for pre-parsing the YAML repertoires.
// -------------------------------------------------------------------------------------------------
extern crate failure;
extern crate regex;
extern crate serde_cbor;
extern crate serde_regex;
extern crate serde_yaml;
// -------------------------------------------------------------------------------------------------
pub mod repertoire;
// -------------------------------------------------------------------------------------------------
use failure::Error;
use repertoire::Repertoire;
use std::collections::HashMap;
use std::fs;
// -------------------------------------------------------------------------------------------------

fn main() {
    println!("cargo:rustc-cfg=cached");
    println!("cargo:rerun-if-changed={}", "src/repertoire.rs");

    print_errors(build_cache());
}

fn print_errors(result: Result<(), Error>) -> () {
    if result.is_ok() {
        return ();
    }

    for cause in Error::iter_chain(&result.unwrap_err()) {
        eprintln!("{}: {}", cause.name().unwrap_or("Error"), cause);
    }

    panic!();
}

fn build_cache() -> Result<(), Error> {
    let mut repertoires: HashMap<String, Repertoire> = HashMap::new();

    // Read all the repertoires.
    for entry in fs::read_dir("res")? {
        let entry = entry?;
        let entry_name = entry.file_name().into_string().unwrap();
        if entry_name.starts_with("rep_") {
            println!("cargo:rerun-if-changed=res/{}", entry_name);

            let rep = Repertoire::from_file(entry.path())?;
            repertoires.insert(rep.name.clone(), rep);
        }
    }

    // Create a cbor cache.
    let cache = serde_cbor::to_vec(&repertoires)?;
    Ok(fs::write("src/repertoire.cache", cache)?)
}
