mod analysis;
mod eval;
mod repl;

use repl::Repl;

fn main() {
    // debug
    //tokenize_parse_and_print("λf.λx.f (f (f x))");

    Repl::new().run();
}
