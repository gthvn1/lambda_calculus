// A lambda expression is either:
// - A variable: x
// - An application: M N   => M applied to N
// - An abstraction: \x. y => Lambda is the binder, binds variable to a lamda term
// In lambda calculus we have parenthesis because by default application are left
// associative and abstraction are greedy.

// =================================================================
//   The entry point of this file is parse_str
pub fn parse_str(input: &str) -> Result<Term, ParseError> {
    parse(tokenize(input))
}

// This entry point is used for debugging puprpose
pub fn tokenize_parse_and_print(input: &str) {
    println!("\nINPUT     : {}", input);

    let ast = parse_str(input).unwrap();
    println!("PARSED    : {:?}", ast);
    println!("PRINTED   : {}\n", ast);
}

// =================================================================
//   TOKENIZER
//
#[derive(Debug, PartialEq)]
enum Token {
    Variable(String),
    Lambda,
    Dot,
    LeftParen,
    RightParen,
}

fn tokenize(input: &str) -> Vec<Token> {
    let mut iter = input.chars().peekable();
    let mut tokens: Vec<Token> = Vec::new();

    while let Some(ch) = iter.next() {
        match ch {
            '\\' | 'λ' => tokens.push(Token::Lambda),
            '.' => tokens.push(Token::Dot),
            '(' => tokens.push(Token::LeftParen),
            ')' => tokens.push(Token::RightParen),
            c if c.is_whitespace() => {}
            c if c.is_alphabetic() => {
                let mut s = String::from(c);
                // Read while it is alphabetic
                while iter.peek().is_some_and(|c| c.is_alphabetic()) {
                    s.push(iter.next().unwrap());
                }
                tokens.push(Token::Variable(s))
            }
            _ => todo!("How handle {}?", ch),
        }
    }

    tokens
}

// =================================================================
//   PARSER
//
// ----- Grammar -----
// term := LAMBDA VARIABLE DOT term | app
// app  := atom+
// atom := VARIABLE | LEFTPAREN term RIGHTPAREN
//
// ----- Example -----
// x   :  term -> app -> atom -> VARIABLE("x")
// x y :  term -> app -> atom -> ...
//

#[derive(Debug, PartialEq)]
pub enum Term {
    Variable(String),
    Application(Box<Term>, Box<Term>),
    Abstraction(String, Box<Term>),
}

// Helpers to build Term
#[allow(dead_code)]
impl Term {
    pub(crate) fn var(name: &str) -> Term {
        Term::Variable(name.to_string())
    }

    pub(crate) fn app(t1: Term, t2: Term) -> Term {
        Term::Application(Box::new(t1), Box::new(t2))
    }

    pub(crate) fn abs(name: &str, t: Term) -> Term {
        Term::Abstraction(name.to_string(), Box::new(t))
    }
}

#[derive(Debug)]
pub enum ParseError {
    Empty,
    VarIsExpected,
    VarOrLeftParenAreExpected,
    DotIsExpected,
    RightParenIsMissing,
    UnexpectedTrailingTokens,
}

fn parse(tokens: Vec<Token>) -> Result<Term, ParseError> {
    // We iter into because we want to take ownership of string in
    // tokens.
    let mut iter = tokens.into_iter().peekable();
    let term = parse_term(&mut iter)?;
    if iter.next().is_none() {
        Ok(term)
    } else {
        Err(ParseError::UnexpectedTrailingTokens)
    }
}

fn parse_term(
    iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
) -> Result<Term, ParseError> {
    match iter.peek() {
        Some(Token::Lambda) => {
            // Consume the lambda
            iter.next();

            // Now we are expecting a variable
            let var = if let Some(Token::Variable(name)) = iter.next() {
                name
            } else {
                return Err(ParseError::VarIsExpected);
            };

            // check the DOT
            if Some(Token::Dot) != iter.next() {
                return Err(ParseError::DotIsExpected);
            }

            let body = parse_term(iter)?;
            Ok(Term::Abstraction(var, Box::new(body)))
        }
        Some(_) => parse_app(iter),
        None => Err(ParseError::Empty),
    }
}

fn parse_app(
    iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
) -> Result<Term, ParseError> {
    let mut left_atom = parse_atom(iter)?;

    while let Some(token) = iter.peek() {
        match token {
            Token::Variable(_) | Token::LeftParen => {
                let right_atom = parse_atom(iter)?;
                left_atom = Term::Application(Box::new(left_atom), Box::new(right_atom));
            }
            _ => break,
        }
    }

    Ok(left_atom)
}

fn parse_atom(
    iter: &mut std::iter::Peekable<std::vec::IntoIter<Token>>,
) -> Result<Term, ParseError> {
    match iter.next() {
        Some(Token::Variable(s)) => Ok(Term::Variable(s)),
        Some(Token::LeftParen) => {
            let term = parse_term(iter)?;
            if let Some(Token::RightParen) = iter.next() {
                Ok(term)
            } else {
                Err(ParseError::RightParenIsMissing)
            }
        }
        _ => Err(ParseError::VarOrLeftParenAreExpected),
    }
}

// =================================================================
//   PRETTY PRINTER
//
#[derive(PartialEq)]
enum Pos {
    Top,
    AppLeft,
    AppRight,
    AbsBody,
}

impl std::fmt::Display for Term {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        fmt_term(self, Pos::Top, f)
    }
}

fn fmt_term(term: &Term, pos: Pos, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    match term {
        Term::Variable(s) => write!(f, "{}", s),
        Term::Application(m, n) => {
            if pos == Pos::AppRight {
                write!(f, "(")?;
            }
            fmt_term(m, Pos::AppLeft, f)?;
            write!(f, " ")?;
            fmt_term(n, Pos::AppRight, f)?;

            if pos == Pos::AppRight {
                write!(f, ")")?;
            }

            Ok(())
        }
        Term::Abstraction(var, body) => {
            if pos == Pos::AppLeft || pos == Pos::AppRight {
                write!(f, "(")?;
            }

            write!(f, "λ{}. ", var)?;
            fmt_term(body, Pos::AbsBody, f)?;

            if pos == Pos::AppLeft || pos == Pos::AppRight {
                write!(f, ")")?;
            }

            Ok(())
        }
    }
}

// =================================================================
//   TESTS
//
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
        let tokens = tokenize(s);
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
        assert!(parse(tokenize("")).is_err());
    }

    #[test]
    fn atom_unclosed_right_paren_error() {
        // "(x": opens, reads term x, then expects ')' which is missing -> panic
        // (this exercises parse_term, so it only passes once parse_term exists;
        //  set it aside if parse_term isn't written yet)
        assert!(matches!(
            parse(tokenize("(x")),
            Err(ParseError::RightParenIsMissing)
        ));
    }

    #[test]
    fn trailing_tokens_error() {
        // "x ) y": parses x, then ") y" remains -> error
        assert!(parse(tokenize("x ) y")).is_err());
    }

    #[test]
    fn single_atom_is_just_the_atom() {
        // one atom, zero repetition: result is the atom itself, no App node
        assert_eq!(parse(tokenize("x")).unwrap(), Term::var("x"));
    }

    #[test]
    fn two_atoms_apply() {
        // f g  ->  App(f, g)
        assert_eq!(
            parse(tokenize("f g")).unwrap(),
            Term::app(Term::var("f"), Term::var("g"))
        );
    }

    #[test]
    fn three_atoms_left_assoc() {
        // f g h  ->  App(App(f, g), h)   -- LEFT associative
        assert_eq!(
            parse(tokenize("f g h")).unwrap(),
            Term::app(Term::app(Term::var("f"), Term::var("g")), Term::var("h"))
        );
    }

    #[test]
    fn extra_whitespace_is_ignored() {
        // multiple/odd spacing must not change the result
        assert_eq!(
            parse(tokenize("  f     g  h   ")).unwrap(),
            Term::app(Term::app(Term::var("f"), Term::var("g")), Term::var("h"))
        );
    }

    #[test]
    fn parenthesized_atom_in_application() {
        // (f) g  ->  App(f, g)   -- parens around a single atom are transparent
        // this exercises parse_atom's '(' branch, which calls parse_term;
        // it only passes once parse_term exists. set aside if not yet written.
        assert_eq!(
            parse(tokenize("(f) g")).unwrap(),
            Term::app(Term::var("f"), Term::var("g"))
        );
    }

    #[test]
    fn identity() {
        // \x. x  ->  Abs("x", Var("x"))
        assert_eq!(
            parse(tokenize("\\x. x")).unwrap(),
            Term::abs("x", Term::var("x"))
        );
    }

    #[test]
    fn nested_abstraction_k() {
        // \x. \y. x  ->  Abs("x", Abs("y", Var("x")))
        // this is the GREEDY test: body of \x is itself an abstraction
        assert_eq!(
            parse(tokenize("\\x. \\y. x")).unwrap(),
            Term::abs("x", Term::abs("y", Term::var("x")))
        );
    }

    #[test]
    fn lambda_body_is_greedy() {
        // \x. x y  ->  Abs("x", App(x, y))   NOT  App(Abs("x", x), y)
        // the lambda swallows "x y" entirely as its body
        assert_eq!(
            parse(tokenize("\\x. x y")).unwrap(),
            Term::abs("x", Term::app(Term::var("x"), Term::var("y")))
        );
    }

    #[test]
    fn application_of_identity() {
        // (\x. x) y  ->  App(Abs("x", x), y)
        // parens STOP the greedy lambda, so it's an application
        assert_eq!(
            parse(tokenize("(\\x. x) y")).unwrap(),
            Term::app(Term::abs("x", Term::var("x")), Term::var("y"))
        );
    }

    #[test]
    fn identity_applied_to_identity() {
        // (\x. x) (\y. y)
        assert_eq!(
            parse(tokenize("(\\x. x) (\\y. y)")).unwrap(),
            Term::app(
                Term::abs("x", Term::var("x")),
                Term::abs("y", Term::var("y"))
            )
        );
    }

    // Pretty printer
    #[test]
    fn pretty_print_exact_output() {
        // exact string output, checking MINIMAL parenthesization
        let cases = [
            ("x", "x"),
            ("\\x. x", "λx. x"),
            ("f g", "f g"),
            ("f g h", "f g h"),                       // left-assoc: no parens
            ("f (g h)", "f (g h)"),                   // right child is app: parens needed
            ("\\x. \\y. x", "λx. λy. x"),             // nested abs in body: no parens
            ("\\x. x y", "λx. x y"),                  // greedy body: no parens
            ("(\\x. x) y", "(λx. x) y"),              // abs as function (AppLeft): parens
            ("f (\\x. x) y", "f (λx. x) y"),          // abs as argument mid (AppRight): parens
            ("(\\x. x) (\\y. y)", "(λx. x) (λy. y)"), // both: left parenthesized, right too
        ];
        for (input, expected) in cases {
            let t = parse(tokenize(input)).unwrap();
            assert_eq!(format!("{}", t), expected, "input was {:?}", input);
        }
    }

    #[test]
    fn print_then_parse_roundtrips() {
        // the printed form must re-parse to the same tree
        let cases = [
            "x",
            "\\x. x",
            "f g",
            "f g h",
            "f (g h)",
            "\\x. \\y. x",
            "(\\x. x) y",
            "f (\\x. x) y",
            "(\\x. x) (\\y. y)",
        ];
        for s in cases {
            let t1 = parse(tokenize(s)).unwrap();
            let printed = format!("{}", t1);
            let t2 = parse(tokenize(&printed))
                .unwrap_or_else(|e| panic!("could not re-parse {:?}: {:?}", printed, e));
            assert_eq!(t1, t2, "roundtrip failed: {:?} -> {:?}", s, printed);
        }
    }
}
