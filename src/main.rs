// -------------------------------------------------------------------------------------------------
// joinerator | Copyright (C) 2019 eth-p
// -------------------------------------------------------------------------------------------------
extern crate ansi_term;
extern crate atty;
extern crate clap;
extern crate failure;
extern crate rand;
extern crate regex;
extern crate serde;
extern crate serde_cbor;
extern crate serde_regex;

#[macro_use]
extern crate lazy_static;

#[cfg(feature = "clipboard_support")]
extern crate clipboard;

// -------------------------------------------------------------------------------------------------
mod content;
mod joinerator;
mod repertoire;

// -------------------------------------------------------------------------------------------------
use std::collections::linked_list::LinkedList;
use std::collections::HashMap;
use std::process::exit;
use std::thread::sleep;
use std::time::Duration;

use ansi_term::{Color, Style};
use atty::Stream;
use clap::{App, Arg, ArgMatches};
use failure::{Error, Fail};

use crate::content::{Consumer, Provider};
use crate::joinerator::{GeneratorFrequency, GeneratorOptions, Joinerator, Options};
use crate::repertoire::{GlyphPosition, Repertoire};
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
        let enabled = ansi_term::enable_ansi_support();

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

    // Initialize program.
    let mut provider = get_provider(&matches);
    let mut consumer = get_consumer(&matches);
    let repertoire = get_repertoire(matches.value_of("repertoire").unwrap()).unwrap();

    let mut joinerator = Joinerator::new(Options {
        allow_unreadable: false,
        limit: matches
            .value_of("length")
            .map(|v| v.parse::<usize>().unwrap()),
        repertoire,
        generator: vec![
            GeneratorOptions {
                category: GlyphPosition::ABOVE,
                stacking: 1,
                frequency: GeneratorFrequency::Percentage(0.6),
            },
            GeneratorOptions {
                category: GlyphPosition::BELOW,
                frequency: GeneratorFrequency::Percentage(0.6),
                stacking: 0,
            },
        ],
    });

    // Run program.
    let result = main_loop(
        &mut joinerator,
        &mut provider,
        &mut consumer,
        !matches.is_present("quiet"),
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
    verbose: bool,
    watch: bool,
) -> Result<(), Error> {
    let mut more = true;
    while more {
        // Input from the source.
        let input = (*provider).provide()?;
        let processed = joinerator.process(&input);

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
        Some("clipboard") => Box::new(content::clipboard::ClipboardConsumer::new()),
        _ => panic!("Unsupported --output argument passed validation."),
    }
}

// -------------------------------------------------------------------------------------------------

fn handle_cli() -> ArgMatches<'static> {
    let valid_reps: Vec<&'static str> = REPERTOIRES.keys().into_iter().map(|s| &s[..]).collect();
    let mut valid_input: Vec<&'static str> = vec!["stdin", "args", "arguments"];
    let mut valid_output: Vec<&'static str> = vec!["stdout"];

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
                .value_name("LENGTH")
                .takes_value(true)
                .validator(|v| {
                    v.parse::<usize>()
                        .or(Err("Length provided is not an integer.".to_owned()))
                        .and_then(|v| {
                            if v > 0 {
                                Ok(())
                            } else {
                                Err("Length provided is not a positive integer.".to_owned())
                            }
                        })
                }),
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
                .help("Suppresses non-essential messages."),
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
