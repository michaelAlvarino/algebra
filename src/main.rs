#![feature(iter_map_while, iterator_fold_self, exclusive_range_pattern)]
use clap::Clap;
use std::io;
use std::io::prelude::*;


/// Apply a mathmatical operation to a stream of inputs.
/// e.x.
/// ```
/// $ printf '2\n3\n\n' | mathcli mul
/// $ 6
/// $ printf '6\n2\n\n' | mathcli sub
/// $ 4
/// $ printf '7\n3\n\n' | mathcli div
/// $ 2.3333333
/// $ printf '5\n4\n\n' | mathcli add
/// $ 9
/// ```
#[derive(Clap)]
#[clap(version = "0.1", author = "Mike A. <michael.alvarino@gmail.com>")]
struct Opts {
    /// Options are add, sub, mul, div
    #[clap(subcommand)]
    subcmd: SubCommand,
    /// Use the identity for this operation as a starting point
    #[clap(long)]
    identity_starting_point: bool,
    /// Silence errors parsing input. Applies the identity for the operation
    /// if a parse failure does occur
    #[clap(short, long)]
    silent: bool,
    /// Ignore lines at the beginning of input.
    #[clap(short, long, default_value="0")]
    ignore: usize,
    /// Logging verbosity, all logs go to stderr. Number of v's translates to logging level
    #[clap(short, long, parse(from_occurrences))]
    verbose: usize,
}

/// The set of available sub commands. Standard mathematical operations.
#[derive(Clap, Clone, Copy)]
enum SubCommand {
    /// Add all inputs.
    /// Identity: 0.0
    Add,
    /// Subtract all inputs.
    /// Identity: 0.0
    Sub,
    /// Multiply all inputs.
    /// Identity: 1.0
    Mul,
    /// Divide all inputs.
    /// Identity: 1.0
    Div,
}

/// Go!
fn main() {
    let opts: Opts = Opts::parse();
    let identity = match opts.subcmd {
        SubCommand::Mul | SubCommand::Div => 1.,
        SubCommand::Add | SubCommand::Sub => 0.
    };
    // let input_handler = InputHandler::new(&opts, identity);
    let operator = match opts.subcmd {
        SubCommand::Add => std::ops::Add::add,
        SubCommand::Sub => std::ops::Sub::sub,
        SubCommand::Mul => std::ops::Mul::mul,
        SubCommand::Div => std::ops::Div::div
    };
    stderrlog::new()
        .verbosity(opts.verbose)
        .init()
        .unwrap();
    log::info!("Starting...");
    let stdin = io::stdin();
    let input_handler = InputHandler::new(&opts, identity);
    let cleaned_input = input_handler.clean_and_enumerate(stdin.lock());
    let parsed_lines = input_handler.parse_input(cleaned_input);

    log::info!("Folding...");
    let result = match opts.identity_starting_point {
        true => parsed_lines.fold(identity, |acc: f32, x| operator(acc, x)),
        false => parsed_lines.fold_first(|acc, x| operator(acc, x)).unwrap()
    };
    log::info!("Writing result");
    println!("{}", result);
}

/// Responsible for cleaning user input
#[derive(Copy, Clone)]
struct InputHandler {
    identity: f32,
    ignore: usize,
    silent: bool,
}

impl InputHandler {
    /// Use the given identity (0 or 1) and options to configure the handler.
    pub fn new(opts: &Opts, identity: f32) -> Self {
        InputHandler {
            ignore: opts.ignore,
            silent: opts.silent,
            identity
        }
    }

    fn clean_and_enumerate<R: BufRead>(self, reader: R) -> impl Iterator<Item=(usize, String)> {
        reader.lines()
        // If we fail to read a line due to some io issue, just explode, not useful to continue
        .map(|x| x.unwrap())
        // if we don't take ownership, we're referencing data owned by the current function
        // overhead of creating a String is minimal
        .map(|x| x.trim().to_string())
        // gives us (index, value). useful for ignoring lines, logging, etc
        .enumerate()
    }

    /// Reads each value into a float and continues until Err is returned
    fn parse_input(self, it: impl Iterator<Item=(usize, String)>) -> impl Iterator<Item=f32> {
        // ignore lines, check for empties, parse to f32, etc
        it.map(move |(i, val)| self.handle(i, &val))
        // keep unwrapping while there's a value
        .map_while(|val: Result<Option<f32>, String>| match val {
            Ok(v) => v,
            Err(e) => {
                // We only get here if --ignore-parse-error is false (which is the default)
                log::error!("{}", e);
                None
            }
        })
    }

    /// Handles a value and its index according to the flags specified by the user.
    fn handle(self, i: usize, val: &str) -> Result<Option<f32>, String> {
        if i < self.ignore {
            log::debug!("Ignored value {}", val);
            return Ok(Some(self.identity))
        }
        if val.is_empty() {
            log::debug!("Found empty at line number {}, exiting.", i + 1);
            return Ok(None)
        }
        return match val.parse::<f32>() {
            Ok(v) => Ok(Some(v)),
            Err(e) => {
                match self.silent {
                    true => {
                        log::warn!("Ignoring parse error {} for {} at line {}", e, val, i + 1);
                        Ok(Some(self.identity))
                    },
                    false => {
                        log::debug!("{}", e);
                        Err(format!("Failed to parse {} at line {}", val, i + 1))
                    }
                }
            }
        }
    }
}


#[cfg(test)]
mod tests {

    use super::InputHandler;

    fn handler(silent: bool) -> InputHandler {
        InputHandler {
            ignore: 2,
            silent,
            identity: 1.5
        }
    }

    #[test]
    fn test_trim_lines() {
        let handler = handler(false);
        for (k, v) in handler.clean_and_enumerate(b"\t\t1\n2\t\t\n   3   \n" as &[u8]) {
            match k {
                0 => assert_eq!("1", v),
                1 => assert_eq!("2", v),
                2 => assert_eq!("3", v),
                _ => panic!()
            }
        }
    }

    #[test]
    fn test_successful_handle() {
        let handler = handler(false);
        assert_eq!(Ok(Some(3.0)), handler.handle(2, "3.0"));
    }
 
    #[test]
    fn test_ignore_returns_identity() {
        let handler = handler(false);
        assert_eq!(Ok(Some(1.5)), handler.handle(0, "2.0"));
        assert_eq!(Ok(Some(1.5)), handler.handle(1, "2.0"));
        assert_eq!(Ok(Some(3.0)), handler.handle(2, "3.0"));
    }

    #[test]
    fn test_exit_on_empty_line() {
        let handler = handler(false);
        assert_eq!(Ok(None), handler.handle(2, ""));
    }

    #[test]
    fn test_exit_on_parse_error() {
        let handler = handler(false);
        let input_string = "notf32";
        let msg = format!("Failed to parse {} at line {}", &input_string, 3);
        assert_eq!(Err(msg), handler.handle(2, input_string));
    }

    #[test]
    fn test_ignore_parse_error() {
        let handler = handler(true);
        let input_string = "notf32";
        assert_eq!(Ok(Some(1.5)), handler.handle(2, input_string));
    }
}
