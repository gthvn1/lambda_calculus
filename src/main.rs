#[derive(Debug)]
enum Term {
    Variable(String),
    Application(Box<Term>, Box<Term>),
    Abstraction(String, Box<Term>),
}

fn main() {
    println!("Hello, world!");
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

#[cfg(test)]
mod tests {
    use super::*;

    // I = λx. x
    #[test]
    fn construit_identite() {
        let i = Term::abs("x", Term::var("x"));
        // si ça construit, c'est déjà gagné — on vérifie juste que le Debug tourne
        let _ = format!("{:?}", i);
    }

    // K = λx. λy. x
    #[test]
    fn construit_k() {
        let k = Term::abs("x", Term::abs("y", Term::var("x")));
        let _ = format!("{:?}", k);
    }

    // (λx. x) y
    #[test]
    fn construit_application() {
        let app = Term::app(Term::abs("x", Term::var("x")), Term::var("y"));
        let _ = format!("{:?}", app);
    }
}
