mod analysis;
mod eval;

use std::io::{self, Write};

use analysis::{Term, parse_str};
use eval::{normalize, reduce_once};

#[derive(Clone, Copy)]
enum Command {
    Current,
    Help,
    Quit,
    Reduce,
    Step,
    Unknown,
}

struct CommandInfo {
    name: &'static str,
    command: Command,
    description: &'static str,
}

const COMMANDS: &[CommandInfo] = &[
    CommandInfo {
        name: ":quit",
        command: Command::Quit,
        description: "quit the REPL",
    },
    CommandInfo {
        name: ":current",
        command: Command::Current,
        description: "show the current term",
    },
    CommandInfo {
        name: ":step",
        command: Command::Step,
        description: "reduce the current term by one step",
    },
    CommandInfo {
        name: ":reduce",
        command: Command::Reduce,
        description: "reduce the current term to normal form",
    },
    CommandInfo {
        name: ":help",
        command: Command::Help,
        description: "show this help",
    },
];

impl Command {
    fn from_str(name: &str) -> Command {
        let lower = name.to_lowercase();
        COMMANDS
            .iter()
            .find(|c| c.name == lower)
            .map(|c| c.command)
            .unwrap_or(Command::Unknown)
    }
}

fn main() {
    // debug
    //tokenize_parse_and_print("λf.λx.f (f (f x))");

    println!("LambdaCalculus version 0.1");
    println!("Enter :quit to quit, :help for help\n");

    let mut current: Option<Term> = None;

    // And now the REPL
    loop {
        print!("λ> ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // ctrl-d
            Ok(_) => {
                let line = input.trim();
                if line.starts_with(":") {
                    // This is a command
                    let cmd = Command::from_str(line);
                    match cmd {
                        Command::Quit => break,
                        Command::Current => {
                            if let Some(term) = &current {
                                println!("{}", term)
                            } else {
                                println!("no lambda expression")
                            }
                        }
                        Command::Step => {
                            let result = current.as_ref().map(reduce_once);

                            match result {
                                Some(Some(term)) => {
                                    println!("step: {}", term);
                                    current = Some(term)
                                }
                                Some(None) => {
                                    println!("it is already a normal form");
                                }
                                None => println!("please enter a lambda expression"),
                            }
                        }
                        Command::Reduce => {
                            let result = current.as_ref().map(|term| normalize(term, 100));
                            match result {
                                Some(eval::Reduction::NormalForm(t)) => {
                                    println!("Normal form: {}", t);
                                    current = Some(t);
                                }
                                Some(eval::Reduction::MaxStepsReached(t)) => {
                                    println!("Max steps reached: {}", t);
                                    current = Some(t);
                                }
                                None => println!("please enter a lambda expression"),
                            }
                        }
                        Command::Help => {
                            for c in COMMANDS {
                                println!("{:10} {}", c.name, c.description);
                            }
                        }
                        Command::Unknown => {
                            println!("Unknown command, :help to get the list of commands")
                        }
                    }
                } else {
                    match parse_str(line) {
                        Err(e) => eprintln!("Parsing failed: {:?}", e),
                        Ok(term) => current = Some(term),
                    }
                }
            }
            Err(error) => println!("Failed to read input: {error}"),
        }
    }
}
