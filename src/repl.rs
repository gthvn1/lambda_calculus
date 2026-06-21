use crate::analysis::{Term, parse_str};
use crate::eval::{Reduction, normalize, reduce_once};
use std::io::{self, Write};

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

pub struct Repl {
    current: Option<Term>,
}

impl Repl {
    pub fn new() -> Self {
        Repl { current: None }
    }

    pub fn run(&mut self) {
        println!("LambdaCalculus version 0.1");
        println!("Enter :quit to quit, :help for help\n");

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
                        if !self.process_line(line) {
                            break;
                        }
                    } else {
                        match parse_str(line) {
                            Err(e) => eprintln!("Parsing failed: {:?}", e),
                            Ok(term) => self.current = Some(term),
                        }
                    }
                }
                Err(error) => println!("Failed to read input: {error}"),
            }
        }
    }

    // Return false to stop the loop
    fn process_line(&mut self, line: &str) -> bool {
        let cmd = Command::from_str(line);
        match cmd {
            Command::Quit => return false,
            Command::Current => {
                if let Some(term) = &self.current {
                    println!("{}", term);
                } else {
                    println!("no lambda expression");
                }
            }
            Command::Step => {
                let result = self.current.as_ref().map(reduce_once);

                match result {
                    Some(Some(term)) => {
                        println!("step: {}", term);
                        self.current = Some(term);
                    }
                    Some(None) => {
                        println!("it is already a normal form");
                    }
                    None => println!("please enter a lambda expression"),
                }
            }
            Command::Reduce => {
                let result = self.current.as_ref().map(|term| normalize(term, 100));
                match result {
                    Some(Reduction::NormalForm(t)) => {
                        println!("Normal form: {}", t);
                        self.current = Some(t);
                    }
                    Some(Reduction::MaxStepsReached(t)) => {
                        println!("Max steps reached: {}", t);
                        self.current = Some(t);
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

        true
    }
}
