mod analysis;

use std::io;
use std::io::Write;

use analysis::{parse_str, tokenize_parse_and_print};

fn main() {
    // debug
    tokenize_parse_and_print("λf.λx.f (f (f x))");

    println!("LambdaCalculus version 0.1");
    println!("Enter /quit to quit\n");

    // And now the REPL
    loop {
        print!("λ> ");
        let _ = io::stdout().flush();
        let mut input = String::new();
        match io::stdin().read_line(&mut input) {
            Ok(0) => break, // ctrl-d
            Ok(_) => {
                let line = input.trim();
                if line.eq_ignore_ascii_case("/quit") {
                    break;
                }
                match parse_str(line) {
                    Ok(t) => println!("{}", t),
                    Err(e) => eprintln!("Parsing failed: {:?}", e),
                }
            }
            Err(error) => println!("Failed to read input: {error}"),
        }
    }
}
