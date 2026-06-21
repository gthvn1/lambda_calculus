mod analysis;
mod eval;

use std::io::{self, Write};

use analysis::{Term, parse_str};
use eval::{normalize, reduce_once};

fn main() {
    // debug
    //tokenize_parse_and_print("λf.λx.f (f (f x))");

    println!("LambdaCalculus version 0.1");
    println!("Enter :quit to quit\n");

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
                    match line.to_lowercase().as_str() {
                        ":quit" => break,
                        ":current" => {
                            if let Some(term) = &current {
                                println!("{}", term)
                            } else {
                                println!("no lambda expression")
                            }
                        }
                        ":step" => {
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
                        ":reduce" => {
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
                        _ => println!("Unkown command"),
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
