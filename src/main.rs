use std::iter::Peekable;
use std::str::Chars;

#[derive(Debug, PartialEq, Eq)]
enum Term {
    Variable(String),
    Application(Box<Term>, Box<Term>),
    Abstraction(String, Box<Term>),
}

impl Term {
    fn var(x: &str) -> Term {
        Term::Variable(x.to_string())
    }

    fn app(m: Term, n: Term) -> Term {
        Term::Application(Box::new(m), Box::new(n))
    }

    fn abs(x: &str, body: Term) -> Term {
        Term::Abstraction(x.to_string(), Box::new(body))
    }
}

#[derive(Debug, PartialEq, Eq)]
enum Token {
    Variable(String),
    Lambda,
    Dot,
    LeftParen,
    RightParen,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut iter = input.chars().peekable();

    while let Some(c) = iter.next() {
        if c.is_whitespace() {
            // Just skip whitespace
            continue;
        }

        match c {
            '\\' => tokens.push(Token::Lambda),
            '.' => tokens.push(Token::Dot),
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            c if c.is_alphabetic() => {
                // We build the string
                let mut buffer = c.to_string();
                while let Some(&next_c) = iter.peek() {
                    if next_c.is_alphabetic() {
                        buffer.push(iter.next().unwrap());
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Variable(buffer));
            }
            _ => panic!("unexpected"),
        }
    }

    tokens
}

// Here is our grammar for lambda calculus
//
// term        := '\' char '.' term | application
// application := atom+
// atom        := char | '(' term ')'

fn parse_term(iter: &mut Peekable<Chars>) -> Term {
    skip_whitespace(iter);
    match iter.peek() {
        Some(&'\\') => {
            // We are expecting an alphabetic
            iter.next();
            if let Some(c) = iter.next() {
                if c.is_alphabetic() {
                    let input_var = c;
                    if let Some('.') = iter.next() {
                        let body = parse_term(iter);
                        Term::abs(&input_var.to_string(), body)
                    } else {
                        panic!("We are expecting a dot to separate variables from body")
                    }
                } else {
                    panic!("We are expecting an alphabetic character")
                }
            } else {
                panic!("we are expecting a character")
            }
        }
        _ => parse_application(iter),
    }
}

fn parse_application(iter: &mut Peekable<Chars>) -> Term {
    // we are expecting an atom
    let mut left = parse_atom(iter);
    // Check if we have another atom
    skip_whitespace(iter);
    while let Some(&c) = iter.peek() {
        if c == '(' || c.is_alphabetic() {
            let right = parse_atom(iter);
            left = Term::app(left, right);
        } else {
            break;
        }
        skip_whitespace(iter);
    }

    left
}

fn parse_atom(iter: &mut Peekable<Chars>) -> Term {
    skip_whitespace(iter);
    match iter.peek() {
        Some(&'(') => {
            iter.next(); // consume open parenthesis
            let term = parse_term(iter);
            // Check if we have the expected closing parenthesis,
            // just panic for now if it is not the case.
            // We can skip whitespace before parenthesis
            skip_whitespace(iter);
            if iter.peek() == Some(&')') {
                iter.next();
                term
            } else {
                panic!("')' is missing");
            }
        }
        Some(&c) if c.is_alphabetic() => {
            if let Some(c) = iter.next() {
                Term::var(&c.to_string())
            } else {
                panic!("failed to read char");
            }
        }
        Some(c) => panic!("char {} is not handled", c),
        None => panic!("a variable is missing"),
    }
}

fn skip_whitespace(iter: &mut Peekable<Chars>) {
    while let Some(c) = iter.peek() {
        if c.is_whitespace() {
            iter.next();
        } else {
            break;
        }
    }
}

fn main() {
    println!("Hello, world!");
}

#[cfg(test)]
mod tests {
    use super::*;

    // --------------------------- TOKEN

    #[test]
    fn tokenize_variables() {
        let tokens = tokenize("x y z");
        let expected = ["x", "y", "z"].map(|x| Token::Variable(x.to_string()));
        assert_eq!(tokens, expected);
    }

    #[test]
    fn tokenize_application() {
        let tokens = tokenize("((x y) z)");
        let expected = [
            Token::LeftParen,
            Token::LeftParen,
            Token::Variable("x".to_string()),
            Token::Variable("y".to_string()),
            Token::RightParen,
            Token::Variable("z".to_string()),
            Token::RightParen,
        ];

        assert_eq!(tokens, expected);
    }

    #[test]
    fn tokenize_abstraction() {
        let tokens = tokenize("((\\x. y) z)");
        let expected = [
            Token::LeftParen,
            Token::LeftParen,
            Token::Lambda,
            Token::Variable("x".to_string()),
            Token::Dot,
            Token::Variable("y".to_string()),
            Token::RightParen,
            Token::Variable("z".to_string()),
            Token::RightParen,
        ];

        assert_eq!(tokens, expected);
    }

    // --------------------------- PARSE

    // I = λx. x
    #[test]
    fn construct_identity() {
        let i = Term::abs("x", Term::var("x"));
        let _ = format!("{:?}", i);
    }

    // K = λx. λy. x
    #[test]
    fn construct_k() {
        let k = Term::abs("x", Term::abs("y", Term::var("x")));
        let _ = format!("{:?}", k);
    }

    // (λx. x) y
    #[test]
    fn construct_application() {
        let app = Term::app(Term::abs("x", Term::var("x")), Term::var("y"));
        let _ = format!("{:?}", app);
    }

    // helper: build the iterator, call parse_atom, and check
    // that the WHOLE input was consumed (nothing left over).
    fn atom_of(s: &str) -> Term {
        let mut it = s.chars().peekable();
        let t = parse_atom(&mut it);
        assert!(it.next().is_none(), "leftover characters remain");
        t
    }

    #[test]
    fn atom_simple_variable() {
        assert_eq!(atom_of("x"), Term::var("x"));
    }

    #[test]
    fn atom_other_variable() {
        assert_eq!(atom_of("y"), Term::var("y"));
    }

    #[test]
    #[should_panic]
    fn atom_empty_panics() {
        // empty input: parse_atom must panic (nothing to read)
        let mut it = "".chars().peekable();
        let _ = parse_atom(&mut it);
    }

    #[test]
    #[should_panic]
    fn atom_unclosed_paren_panics() {
        // "(x": opens, reads term x, then expects ')' which is missing -> panic
        // (this exercises parse_term, so it only passes once parse_term exists;
        //  set it aside if parse_term isn't written yet)
        let mut it = "(x".chars().peekable();
        let _ = parse_atom(&mut it);
    }

    fn app_of(s: &str) -> Term {
        let mut it = s.chars().peekable();
        let t = parse_application(&mut it);
        // not asserting full consumption here: trailing spaces may remain,
        // and parse_application stops at the first non-atom char
        t
    }

    #[test]
    fn single_atom_is_just_the_atom() {
        // one atom, zero repetition: result is the atom itself, no App node
        assert_eq!(app_of("x"), Term::var("x"));
    }

    #[test]
    fn two_atoms_apply() {
        // f g  ->  App(f, g)
        assert_eq!(app_of("f g"), Term::app(Term::var("f"), Term::var("g")));
    }

    #[test]
    fn three_atoms_left_assoc() {
        // f g h  ->  App(App(f, g), h)   -- LEFT associative
        assert_eq!(
            app_of("f g h"),
            Term::app(Term::app(Term::var("f"), Term::var("g")), Term::var("h"))
        );
    }

    #[test]
    fn extra_whitespace_is_ignored() {
        // multiple/odd spacing must not change the result
        assert_eq!(
            app_of("  f   g  h "),
            Term::app(Term::app(Term::var("f"), Term::var("g")), Term::var("h"))
        );
    }

    #[test]
    fn parenthesized_atom_in_application() {
        // (f) g  ->  App(f, g)   -- parens around a single atom are transparent
        // this exercises parse_atom's '(' branch, which calls parse_term;
        // it only passes once parse_term exists. set aside if not yet written.
        assert_eq!(app_of("(f) g"), Term::app(Term::var("f"), Term::var("g")));
    }

    fn parse(s: &str) -> Term {
        let mut it = s.chars().peekable();
        parse_term(&mut it)
    }

    #[test]
    fn identity() {
        // \x. x  ->  Abs("x", Var("x"))
        assert_eq!(parse("\\x. x"), Term::abs("x", Term::var("x")));
    }

    #[test]
    fn nested_abstraction_k() {
        // \x. \y. x  ->  Abs("x", Abs("y", Var("x")))
        // this is the GREEDY test: body of \x is itself an abstraction
        assert_eq!(
            parse("\\x. \\y. x"),
            Term::abs("x", Term::abs("y", Term::var("x")))
        );
    }

    #[test]
    fn lambda_body_is_greedy() {
        // \x. x y  ->  Abs("x", App(x, y))   NOT  App(Abs("x", x), y)
        // the lambda swallows "x y" entirely as its body
        assert_eq!(
            parse("\\x. x y"),
            Term::abs("x", Term::app(Term::var("x"), Term::var("y")))
        );
    }

    #[test]
    fn application_of_identity() {
        // (\x. x) y  ->  App(Abs("x", x), y)
        // parens STOP the greedy lambda, so it's an application
        assert_eq!(
            parse("(\\x. x) y"),
            Term::app(Term::abs("x", Term::var("x")), Term::var("y"))
        );
    }

    #[test]
    fn identity_applied_to_identity() {
        // (\x. x) (\y. y)
        assert_eq!(
            parse("(\\x. x) (\\y. y)"),
            Term::app(
                Term::abs("x", Term::var("x")),
                Term::abs("y", Term::var("y"))
            )
        );
    }
}
