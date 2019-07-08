// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
extern crate ansi_term;
extern crate clap;
extern crate failure;
extern crate rand;
extern crate regex;
extern crate serde;
extern crate serde_cbor;
extern crate serde_regex;

#[macro_use]
extern crate lazy_static;

#[cfg(not(target = "windows"))]
extern crate atty;

#[cfg(feature = "clipboard_support")]
extern crate clipboard;

// -------------------------------------------------------------------------------------------------
mod content;
mod joinerator;
mod repertoire;
mod transform;

// -------------------------------------------------------------------------------------------------
use std::collections::linked_list::LinkedList;
use std::collections::HashMap;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use ansi_term::{Color, Style};
use clap::{App, Arg, ArgMatches};
use failure::{Error, Fail};

#[cfg(not(target = "windows"))]
use atty::Stream;

use crate::content::{Consumer, Provider};
use crate::joinerator::{GeneratorFrequency, GeneratorOptions, Joinerator, Options};
use crate::repertoire::{GlyphPosition, Repertoire};
use crate::transform::Transformer;
// -------------------------------------------------------------------------------------------------

struct Colors {
    pub heading: Style,
    pub argument: Style,
    pub argument_value: Style,
    pub description: Style,
    pub error_heading: Style,
    pub error: Style,
    pub verbose_input: Style,
    pub verbose_output: Style,
}

impl Colors {
    pub fn default() -> Self {
        Colors {
            heading: Color::Purple.into(),
            argument: Color::Yellow.into(),
            argument_value: Color::Yellow.into(),
            description: Style::new(),
            error_heading: Style::from(Color::Red).bold(),
            error: Color::Red.into(),
            verbose_input: Color::Cyan.into(),
            verbose_output: Color::Green.into(),
        }
    }

    pub fn none() -> Self {
        let plain = Style::new();
        Colors {
            heading: plain,
            argument: plain,
            argument_value: plain,
            description: plain,
            error_heading: plain,
            error: plain,
            verbose_input: plain,
            verbose_output: plain,
        }
    }
}

// -------------------------------------------------------------------------------------------------

#[cfg(target = "windows")]
const EOL: &str = "\r\n";

#[cfg(not(target = "windows"))]
const EOL: &str = "\n";

lazy_static! {
    static ref REPERTOIRES: HashMap<String, Repertoire> = {
        #[cfg(cached)]
        {
            serde_cbor::from_slice(include_bytes!("repertoire.cache")).unwrap()
        }

        #[cfg(not(cached))]
        {
            panic!("Project must be built with cached repertoires.")
        }
    };
    static ref COLORS: Colors = {
        #[cfg(target_os = "windows")]
        let enabled = ansi_term::enable_ansi_support().is_ok();

        #[cfg(not(target_os = "windows"))]
        let enabled = atty::is(Stream::Stdout);

        if enabled {
            Colors::default()
        } else {
            Colors::none()
        }
    };
}

// -------------------------------------------------------------------------------------------------

fn main() {
    let matches = handle_cli();

    // Handle action flags.
    if matches.is_present("list-repertoires") {
        list_repertoires();
        return;
    }

    // Handle verbosity.
    let verbose = if matches.is_present("verbose") {
        true
    } else if matches.is_present("quiet") {
        false
    } else {
        match matches.value_of("output") {
            Some("stdout") => false,
            _ => true,
        }
    };

    // Initialize program.
    let mut provider = get_provider(&matches);
    let mut consumer = get_consumer(&matches);
    let mut transformers = get_transformers(&matches);
    let repertoire = get_repertoire(matches.value_of("repertoire").unwrap()).unwrap();

    let mut joinerator = Joinerator::new(Options {
        allow_unreadable: matches.is_present("unreadable"),
        limit: matches
            .value_of("length")
            .map(|v| v.parse::<usize>().unwrap()),
        repertoire,
        generator: vec![
            GeneratorOptions {
                category: GlyphPosition::ABOVE,
                stacking: parse_stacking(matches.value_of("above:stacking").unwrap()).unwrap(),
                frequency: parse_frequency(matches.value_of("above:frequency").unwrap()).unwrap(),
            },
            GeneratorOptions {
                category: GlyphPosition::BELOW,
                stacking: parse_stacking(matches.value_of("below:stacking").unwrap()).unwrap(),
                frequency: parse_frequency(matches.value_of("below:frequency").unwrap()).unwrap(),
            },
            GeneratorOptions {
                category: GlyphPosition::THROUGH,
                stacking: parse_stacking(matches.value_of("through:stacking").unwrap()).unwrap(),
                frequency: parse_frequency(matches.value_of("through:frequency").unwrap()).unwrap(),
            },
        ],
    });

    // Run program.
    let result = main_loop(
        &mut joinerator,
        &mut provider,
        &mut consumer,
        &mut transformers,
        verbose,
        matches.is_present("watch"),
    );

    if result.is_err() {
        main_errors(result.unwrap_err());
        exit(1);
    }
}

fn main_loop(
    joinerator: &mut Joinerator,
    provider: &mut Box<Provider>,
    consumer: &mut Box<Consumer>,
    transformers: &mut Vec<Box<Transformer>>,
    verbose: bool,
    watch: bool,
) -> Result<(), Error> {
    let mut more = true;
    while more {
        // Input processing.
        let input = (*provider).provide()?;
        let transformed = transformers
            .iter_mut()
            .fold(Ok(input.clone()), |o, t| o.and_then(|v| t.transform(v)))?;

        let processed = joinerator.process(&transformed);

        // Output to logs.
        if verbose {
            eprintln!(
                "{} {}{}{} {}",
                COLORS.heading.paint("Input: "),
                COLORS.verbose_input.paint(input),
                EOL,
                COLORS.heading.paint("Output:"),
                COLORS.verbose_output.paint(&processed)
            );
        }

        // Output to the consumer.
        (*consumer).consume(processed)?;
        more = (*provider).has_more()?;

        // If in watch mode, we'll poll every 10 milliseconds until a change happens.
        // This isn't very efficient, but it works well enough.
        if !more && watch {
            while !more {
                sleep(Duration::from_millis(10));
                more = (*provider).has_more()?;
            }
        }
    }

    Ok(())
}

fn main_errors(error: Error) -> () {
    eprintln!(
        "{}{}{}",
        COLORS
            .error_heading
            .paint("An unexpected error has occurred."),
        EOL,
        COLORS.error.paint(error.to_string())
    );

    // Trace.
    eprintln!("{}{}", EOL, COLORS.error_heading.paint("Trace:"));

    let mut cause: Option<&Fail> = Some(error.as_fail());
    while cause.is_some() {
        let unwrapped = cause.unwrap();
        eprintln!(
            "{}{}",
            COLORS
                .error
                .paint(format!(" - {} ", unwrapped.name().unwrap_or("Error"))),
            unwrapped.to_string()
        );

        cause = unwrapped.cause();
    }
}

fn get_repertoire<'a>(name: &str) -> Option<&'a Repertoire> {
    REPERTOIRES.get(name)
}

fn get_transformers<'a>(matches: &'a ArgMatches<'a>) -> Vec<Box<Transformer>> {
    if !matches.is_present("transform") {
        return vec![];
    }

    matches
        .values_of("transform")
        .unwrap()
        .into_iter()
        .map(get_transformer)
        .collect()
}

// -------------------------------------------------------------------------------------------------
// Helper functions to convert command line arguments into objects.
// -------------------------------------------------------------------------------------------------

fn get_provider<'a>(matches: &'a ArgMatches<'a>) -> Box<Provider> {
    match matches.value_of("input") {
        Some("stdin") => Box::new(content::streams::StdinProvider::new()),
        Some("clipboard") => Box::new(content::clipboard::ClipboardProvider::new()),
        Some("args") | Some("arguments") => {
            let args: LinkedList<String> = matches
                .values_of("values")
                .unwrap()
                .map(|s| s.to_owned())
                .collect();

            Box::new(content::strings::StringProvider::new(args))
        }

        _ => panic!("Unsupported --input argument passed validation."),
    }
}

fn get_consumer<'a>(matches: &'a ArgMatches<'a>) -> Box<Consumer> {
    match matches.value_of("output") {
        Some("stdout") => Box::new(content::streams::StdoutConsumer::new()),
        Some("null") => Box::new(content::null::NullConsumer::new()),
        Some("clipboard") => Box::new(content::clipboard::ClipboardConsumer::new()),
        _ => panic!("Unsupported --output argument passed validation."),
    }
}

fn get_transformer(name: &str) -> Box<Transformer> {
    match name {
        "upper" | "uppercase" => Box::new(transform::casing::TransformUpperCase::new()),
        "lower" | "lowercase" => Box::new(transform::casing::TransformLowerCase::new()),
        "randomcase" => Box::new(transform::casing::TransformRandomCase::new()),
        "uwuize" => Box::new(transform::uwu::TransformUwuize::new()),
        _ => panic!("Unsupported --transform argument passed validation."),
    }
}

// -------------------------------------------------------------------------------------------------
// Helper functions to parse specific command line argument values.
// -------------------------------------------------------------------------------------------------

fn parse_frequency(str: &str) -> Option<GeneratorFrequency> {
    if str.ends_with("%") {
        match (&str[0..(str.len() - 1)]).parse::<f32>() {
            Err(_) => None,
            Ok(v) => {
                if v > 0.0 && v <= 100.0 {
                    Some(GeneratorFrequency::Percentage(v / 100.0))
                } else {
                    None
                }
            }
        }
    } else {
        match str.parse::<usize>() {
            Err(_) => None,
            Ok(v) => {
                if v > 0 {
                    Some(GeneratorFrequency::Fixed(v))
                } else {
                    None
                }
            }
        }
    }
}

fn parse_stacking(str: &str) -> Option<usize> {
    match str.parse::<usize>() {
        Err(_) => None,
        Ok(v) => {
            if v >= 0 {
                Some(v)
            } else {
                None
            }
        }
    }
}

// -------------------------------------------------------------------------------------------------
// The CLAP command line application.
// -------------------------------------------------------------------------------------------------

fn handle_cli() -> ArgMatches<'static> {
    let valid_reps: Vec<&'static str> = REPERTOIRES.keys().into_iter().map(|s| &s[..]).collect();
    let mut valid_input: Vec<&'static str> = vec!["stdin", "args", "arguments"];
    let mut valid_output: Vec<&'static str> = vec!["stdout", "null"];

    #[cfg(feature = "clipboard_support")]
    {
        valid_input.push("clipboard");
        valid_output.push("clipboard");
    }

    App::new("joinerator")
        .version(env!("CARGO_PKG_VERSION"))
        .author("Ethan P. <eth-p@hidden.email>")
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            Arg::with_name("repertoire")
                .short("z")
                .long("repertoire")
                .help("Specifies which character repertoire to use.")
                .value_name("NAME")
                .takes_value(true)
                .default_value("default")
                .possible_values(&valid_reps[..]),
        )
        .arg(
            Arg::with_name("length")
                .short("l")
                .long("length")
                .help("Enforces a maximum string length.")
                .long_help("Enforces a maximum string length. This will not truncate the text.")
                .alias("limit")
                .value_name("LENGTH")
                .takes_value(true)
                .validator(|v| {
                    v.parse::<usize>()
                        .or(Err("Length provided is not an integer.".to_owned()))
                        .and_then(|v| {
                            if v >= 0 {
                                Ok(())
                            } else {
                                Err("Length provided is not a positive integer.".to_owned())
                            }
                        })
                }),
        )
        .arg(
            Arg::with_name("above:stacking")
                .short("a")
                .long("above:stacking")
                .help("Specifies the maximum number symbols that can go above a glyph.")
                .alias("above-stacking")
                .value_name("COUNT")
                .takes_value(true)
                .default_value("1")
                .validator(|v| {
                    parse_stacking(&v)
                        .and(Some(()))
                        .ok_or("Invalid stacking size.".to_owned())
                }),
        )
        .arg(
            Arg::with_name("above:frequency")
                .short("A")
                .long("above:frequency")
                .help("Specifies the frequency of the symbols that can go above a glyph.")
                .alias("above-frequency")
                .value_name("FREQUENCY")
                .takes_value(true)
                .default_value("60%")
                .validator(|v| {
                    parse_frequency(&v)
                        .and(Some(()))
                        .ok_or("Invalid frequency.".to_owned())
                }),
        )
        .arg(
            Arg::with_name("below:stacking")
                .short("b")
                .long("below:stacking")
                .help("Specifies the maximum number symbols that can go below a glyph.")
                .alias("below-stacking")
                .value_name("COUNT")
                .takes_value(true)
                .default_value("1")
                .validator(|v| {
                    parse_stacking(&v)
                        .and(Some(()))
                        .ok_or("Invalid stacking size.".to_owned())
                }),
        )
        .arg(
            Arg::with_name("below:frequency")
                .short("B")
                .long("below:frequency")
                .help("Specifies the frequency of the symbols that can go below a glyph.")
                .alias("above-frequency")
                .value_name("FREQUENCY")
                .takes_value(true)
                .default_value("60%")
                .validator(|v| {
                    parse_frequency(&v)
                        .and(Some(()))
                        .ok_or("Invalid frequency.".to_owned())
                }),
        )
        .arg(
            Arg::with_name("through:stacking")
                .short("c")
                .long("through:stacking")
                .help("Specifies the maximum number symbols that can go through a glyph.")
                .alias("through-stacking")
                .value_name("COUNT")
                .takes_value(true)
                .default_value("0")
                .validator(|v| {
                    parse_stacking(&v)
                        .and(Some(()))
                        .ok_or("Invalid stacking size.".to_owned())
                }),
        )
        .arg(
            Arg::with_name("through:frequency")
                .short("C")
                .long("through:frequency")
                .help("Specifies the frequency of the symbols that can go through a glyph.")
                .alias("through-frequency")
                .value_name("FREQUENCY")
                .takes_value(true)
                .default_value("10%")
                .validator(|v| {
                    parse_frequency(&v)
                        .and(Some(()))
                        .ok_or("Invalid frequency.".to_owned())
                }),
        )
        .arg(
            Arg::with_name("transform")
                .short("t")
                .long("transform")
                .help("Applies a transformation to the supplied text.")
                .value_name("TRANSFORMER")
                .takes_value(true)
                .number_of_values(1)
                .multiple(true)
                .possible_values(&[
                    "uppercase",
                    "lowercase",
                    "randomcase",
                    "upper",
                    "lower",
                    "uwuize",
                ]),
        )
        .arg(
            Arg::with_name("list-repertoires")
                .long("list-repertoires")
                .help("Lists all the available character repertoires.")
                .conflicts_with_all(&["repertoire"]),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("Suppresses non-essential messages.")
                .conflicts_with("verbose"),
        )
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .long("verbose")
                .help("Display non-essential messages.")
                .conflicts_with("quiet"),
        )
        .arg(
            Arg::with_name("unreadable")
                .short("u")
                .long("unreadable")
                .help("Allows unreadable character combinations."),
        )
        .arg(
            Arg::with_name("watch")
                .short("W")
                .long("watch")
                .help("Watches for changes over time (when using mutable input sources)."),
        )
        .arg(
            Arg::with_name("input")
                .short("i")
                .long("input")
                .help("Specifies the input source.")
                .takes_value(true)
                .value_name("TYPE")
                .possible_values(&valid_input[..])
                .default_value("stdin"),
        )
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .help("Specifies the output destination.")
                .value_name("TYPE")
                .takes_value(true)
                .possible_values(&valid_output[..])
                .default_value("stdout"),
        )
        .arg(Arg::with_name("values").value_name("INPUT").multiple(true))
        .get_matches()
}

fn list_repertoires() {
    println!("{}", COLORS.heading.paint("Repertoires:"));
    for rep in REPERTOIRES.values() {
        println!(
            "{}{:width$} -- {}",
            COLORS.argument_value.paint(&rep.name),
            "",
            &rep.description,
            width = 16 - rep.name.len()
        );
    }
}
