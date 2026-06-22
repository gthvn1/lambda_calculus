use crate::analysis::{Term, parse_str};
use crate::eval::{Reduction, normalize, reduce_once};
use std::collections::HashMap;
use std::io::{self, Write};

#[derive(Clone, Copy)]
enum Command {
    Current,
    Env,
    Help,
    Let,
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
        name: ":current",
        command: Command::Current,
        description: "show the current term",
    },
    CommandInfo {
        name: ":env",
        command: Command::Env,
        description: "dump all bindings in the current environmement",
    },
    CommandInfo {
        name: ":help",
        command: Command::Help,
        description: "show this help",
    },
    CommandInfo {
        name: ":let",
        command: Command::Let,
        description: "bind <NAME> to current into env. NAME must be uppercase.",
    },
    CommandInfo {
        name: ":quit",
        command: Command::Quit,
        description: "quit the REPL",
    },
    CommandInfo {
        name: ":reduce",
        command: Command::Reduce,
        description: "reduce the current term to normal form",
    },
    CommandInfo {
        name: ":step",
        command: Command::Step,
        description: "reduce the current term by one step",
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
    env: HashMap<String, Term>,
}

impl Repl {
    pub fn new() -> Self {
        let mut env = HashMap::new();
        // Add some "libraries"
        env.insert("TRUE".to_string(), parse_str("λx.λy.x").unwrap());
        env.insert("FALSE".to_string(), parse_str("λx.λy.y").unwrap());
        env.insert("AND".to_string(), parse_str("λp.λq.p q p").unwrap());
        env.insert("OR".to_string(), parse_str("λp.λq.p p q").unwrap());

        Repl { current: None, env }
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
        if line.is_empty() {
            // Nothing to do just loop into the repl
            return true;
        }

        let cmd: Vec<&str> = line.split_whitespace().collect();
        let cmd_name = Command::from_str(cmd[0]);
        match cmd_name {
            Command::Quit => return false,
            Command::Current => {
                if let Some(term) = &self.current {
                    println!("{}", term);
                } else {
                    println!("no lambda expression");
                }
            }
            Command::Env => {
                // Cannot be empty: we have at least TRUE and few others
                for (k, v) in self.env.iter() {
                    println!("{} -> {}", k, v);
                }
            }
            Command::Let => {
                // We are expecting one argument.
                match cmd.get(1) {
                    None => println!("a name is expected"),
                    Some(s) => {
                        if s.chars().all(|c| c.is_alphabetic() && c.is_uppercase()) {
                            match &self.current {
                                None => println!("current is empty, nothing to bind"),
                                Some(t) => match self.env.insert(s.to_string(), t.clone()) {
                                    None => println!("{} added into env", *s),
                                    Some(_) => println!("{} has been updated", *s),
                                },
                            }
                        } else {
                            println!("{} must be uppercased", s);
                        }
                    }
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
