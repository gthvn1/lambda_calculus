use core::fmt;

#[derive(Debug)]
enum LexError {
    UnexpectedChar,
}

impl fmt::Display for LexError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            LexError::UnexpectedChar => write!(f, "unexpected character"),
        }
    }
}

#[derive(Debug)]
enum ParseError {
    VarAfterLambdaMissing,
    DotAfterVarInAbstraction,
    RightParenIsMissing,
    LambdaNotExpected,
    DotNotExpected,
    RightParenNotExpected,
    VarIsMissing,
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ParseError::VarAfterLambdaMissing => {
                write!(f, "a variable is expected after lambda")
            }
            ParseError::DotAfterVarInAbstraction => {
                write!(f, "a dot is expected after a variable in an abstration")
            }
            ParseError::RightParenIsMissing => {
                write!(f, "right parenthesis is missing")
            }
            ParseError::LambdaNotExpected => {
                write!(f, "lambda is not expected")
            }
            ParseError::DotNotExpected => {
                write!(f, "dot is not expected")
            }
            ParseError::RightParenNotExpected => {
                write!(f, "right parenthesis is not expected")
            }
            ParseError::VarIsMissing => {
                write!(f, "a variable is missing")
            }
        }
    }
}

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

fn tokenize(input: &str) -> Result<Vec<Token>, LexError> {
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
            _ => return Err(LexError::UnexpectedChar),
        }
    }

    Ok(tokens)
}

// Here is our grammar for lambda calculus
//
// term        := LAMBDA VARIABLE DOT term | application
// application := atom+
// atom        := VARIABLE | LEFTPAREN term RIGHTPAREN

fn parse(tokens: Vec<Token>) -> Result<Term, ParseError> {
    let mut iter = tokens.into_iter().peekable();
    parse_term(&mut iter)
}

fn parse_term(
    iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
) -> Result<Term, ParseError> {
    match iter.peek() {
        Some(Token::Lambda) => {
            // Consume the lamda
            iter.next();
            // We are expecting an alphabetic
            let name = if let Some(Token::Variable(var)) = iter.next() {
                var
            } else {
                return Err(ParseError::VarAfterLambdaMissing);
            };

            if let Some(Token::Dot) = iter.next() {
                // Nothing to do
            } else {
                return Err(ParseError::DotAfterVarInAbstraction);
            };

            let body = parse_term(iter)?;
            Ok(Term::Abstraction(name, Box::new(body)))
        }
        _ => parse_application(iter),
    }
}

fn parse_application(
    iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
) -> Result<Term, ParseError> {
    // we are expecting an atom
    let mut left = parse_atom(iter)?;

    // Check if we have another atom
    while let Some(token) = iter.peek() {
        match token {
            Token::LeftParen | Token::Variable(_) => {
                let right = parse_atom(iter)?;
                left = Term::app(left, right);
            }
            _ => break,
        }
    }

    Ok(left)
}

fn parse_atom(
    iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
) -> Result<Term, ParseError> {
    match iter.next() {
        Some(Token::LeftParen) => {
            let term = parse_term(iter)?;
            // Check if we have the expected closing parenthesis,
            // We can skip whitespace before parenthesis
            if iter.peek() == Some(&Token::RightParen) {
                iter.next();
                Ok(term)
            } else {
                Err(ParseError::RightParenIsMissing)
            }
        }
        Some(Token::Variable(name)) => Ok(Term::Variable(name)),
        Some(Token::Lambda) => Err(ParseError::LambdaNotExpected),
        Some(Token::Dot) => Err(ParseError::DotNotExpected),
        Some(Token::RightParen) => Err(ParseError::RightParenNotExpected),
        None => Err(ParseError::VarIsMissing),
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
        let tokens = tokenize("x y z").unwrap();
        let expected = ["x", "y", "z"].map(|x| Token::Variable(x.to_string()));
        assert_eq!(tokens, expected);
    }

    #[test]
    fn tokenize_application() {
        let tokens = tokenize("((x y) z)").unwrap();
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
        let tokens = tokenize("((\\x. y) z)").unwrap();
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
        let tokens = tokenize(s).unwrap();
        let mut it = tokens.into_iter().peekable();
        let t = parse_atom(&mut it).unwrap();
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
    fn atom_empty_error() {
        // empty input: parse_atom must panic (nothing to read)
        assert!(parse(tokenize("").unwrap()).is_err());
    }

    #[test]
    fn atom_unclosed_right_paren_error() {
        // "(x": opens, reads term x, then expects ')' which is missing -> panic
        // (this exercises parse_term, so it only passes once parse_term exists;
        //  set it aside if parse_term isn't written yet)
        assert!(matches!(
            parse(tokenize("(x").unwrap()),
            Err(ParseError::RightParenIsMissing)
        ));
    }

    #[test]
    fn single_atom_is_just_the_atom() {
        // one atom, zero repetition: result is the atom itself, no App node
        assert_eq!(parse(tokenize("x").unwrap()).unwrap(), Term::var("x"));
    }

    #[test]
    fn two_atoms_apply() {
        // f g  ->  App(f, g)
        assert_eq!(
            parse(tokenize("f g").unwrap()).unwrap(),
            Term::app(Term::var("f"), Term::var("g"))
        );
    }

    #[test]
    fn three_atoms_left_assoc() {
        // f g h  ->  App(App(f, g), h)   -- LEFT associative
        assert_eq!(
            parse(tokenize("f g h").unwrap()).unwrap(),
            Term::app(Term::app(Term::var("f"), Term::var("g")), Term::var("h"))
        );
    }

    #[test]
    fn extra_whitespace_is_ignored() {
        // multiple/odd spacing must not change the result
        assert_eq!(
            parse(tokenize("  f     g  h   ").unwrap()).unwrap(),
            Term::app(Term::app(Term::var("f"), Term::var("g")), Term::var("h"))
        );
    }

    #[test]
    fn parenthesized_atom_in_application() {
        // (f) g  ->  App(f, g)   -- parens around a single atom are transparent
        // this exercises parse_atom's '(' branch, which calls parse_term;
        // it only passes once parse_term exists. set aside if not yet written.
        assert_eq!(
            parse(tokenize("(f) g").unwrap()).unwrap(),
            Term::app(Term::var("f"), Term::var("g"))
        );
    }

    #[test]
    fn identity() {
        // \x. x  ->  Abs("x", Var("x"))
        assert_eq!(
            parse(tokenize("\\x. x").unwrap()).unwrap(),
            Term::abs("x", Term::var("x"))
        );
    }

    #[test]
    fn nested_abstraction_k() {
        // \x. \y. x  ->  Abs("x", Abs("y", Var("x")))
        // this is the GREEDY test: body of \x is itself an abstraction
        assert_eq!(
            parse(tokenize("\\x. \\y. x").unwrap()).unwrap(),
            Term::abs("x", Term::abs("y", Term::var("x")))
        );
    }

    #[test]
    fn lambda_body_is_greedy() {
        // \x. x y  ->  Abs("x", App(x, y))   NOT  App(Abs("x", x), y)
        // the lambda swallows "x y" entirely as its body
        assert_eq!(
            parse(tokenize("\\x. x y").unwrap()).unwrap(),
            Term::abs("x", Term::app(Term::var("x"), Term::var("y")))
        );
    }

    #[test]
    fn application_of_identity() {
        // (\x. x) y  ->  App(Abs("x", x), y)
        // parens STOP the greedy lambda, so it's an application
        assert_eq!(
            parse(tokenize("(\\x. x) y").unwrap()).unwrap(),
            Term::app(Term::abs("x", Term::var("x")), Term::var("y"))
        );
    }

    #[test]
    fn identity_applied_to_identity() {
        // (\x. x) (\y. y)
        assert_eq!(
            parse(tokenize("(\\x. x) (\\y. y)").unwrap()).unwrap(),
            Term::app(
                Term::abs("x", Term::var("x")),
                Term::abs("y", Term::var("y"))
            )
        );
    }
}
