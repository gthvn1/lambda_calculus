use crate::analysis::Term;
use std::collections::HashSet;

// Notions used for the evaluation:
//
// bound/free,
// shadowing,
// capture,
// capture-avoiding substitution,
// α-conversion
//
// --- Substitution rules: M[x:=N] ---
//
// Variable:
//   x[x:=N] = N   (same var -> replaced)
//   y[x:=N] = y   (other var -> unchanged)
//
//  Application (recuse both sides):
//    (A B)[x:=N] = (A[x:=N] B[x:=N])
//
//  Abstraction (three cases on (\v. M)[x:=N]):
//    1. v == x:  \v. M is unchanged, (v shadows x; stop don't recurse)
//    2. v != x AND v not in FV(N),   \v. (M[x:=N))  (safe, recurse)
//    3. v != x AND v in FV(N): CAPTURE risk -> alpha-rename v to fresh w, then recurse
//
// --- FV(t): free variables of t ---
//
//   FV(Variable x) = {x}
//   FV(Application A B) = FV[A] union FV[B]
//   FV(Abstraction v M) = FV[M] \ {v}
//
// --- Examples ---
//
// (λx. x) y      -> VAR  | x[x:=y] -> y
// (λx. x x) y    -> APP  | (x x)[x:=y] -> (x[x:=y] x[x:=y]) -> (y y)
// (λx. λy. x) a  -> ABS.2| (\y. x)[x:=a] -> \y. (x[x:=a]) -> \y. a
// (λx. λx. x) a  -> ABS.1| (\x. x)[x:=a] -> \x. x
// (λx. λy. x) y  -> ABS.3| (\y. x)[x:=y] -> (\w. x)[x:=y] -> \w. y
//
// (λx. λy. x) (g y)
//   -> (λx. (λy. x)) (g y) ; FV(g y) = {g, y}
//   -> (\y. x)[x:=(g y)]   ; it is the case 3 of the Abstration
//   -> (\w. x)[x:=(g y)]   ; alpha conversion
//   -> (\w. (g y))
//
// (λf. (λx. f x)) (λy. x)
//   -> (\x. f x) [f:= \y.x] ; FV({x}) \ y == FV[{x}]
//   -> capture risk -> (\w. f w) [f:= \y. x]
//   -> \w. (f w)[f:= \y. x]
//   -> \w. (f[f:= \y. x] w[f:=\y.x])
//   -> \w. (\y. x) w

fn substitute(m: &Term, x: &str, n: &Term) -> Term {
    match m {
        Term::Variable(v) => {
            // x[x:=N] = N   (same var -> replaced)
            // y[x:=N] = y   (other var -> unchanged)
            if v == x { n.clone() } else { Term::var(v) }
        }
        Term::Application(t1, t2) => {
            // (A B)[x:=N] = (A[x:=N] B[x:=N])
            Term::app(substitute(t1, x, n), substitute(t2, x, n))
        }
        Term::Abstraction(_, _) => {
            // 1. v == x:  \v. M is unchanged, (v shadows x; stop don't recurse)
            // 2. v != x AND v not in FV(N),   \v. (M[x:=N))  (safe, recurse)
            // 3. v != x AND v in FV(N): CAPTURE risk -> alpha-rename v to fresh w, then recurse
            todo!()
        }
    }
}

fn free_variables(t: &Term) -> HashSet<String> {
    match t {
        Term::Variable(v) => {
            // FV(Variable x) = {x}
            let mut s = HashSet::new();
            s.insert(v.clone());
            s
        }
        Term::Application(m, n) => {
            // FV(Application A B) = FV[A] union FV[B]
            let mut fv_m = free_variables(m);
            let fv_n = free_variables(n);

            fv_m.extend(fv_n);
            fv_m
        }
        Term::Abstraction(v, m) => {
            // FV(Abstraction v M) = FV[M] \ {v}
            let mut fv_m = free_variables(m);
            fv_m.remove(v.as_str());
            fv_m
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fv_variable() {
        let t = Term::var("x");
        let fv = free_variables(&t);
        assert!(fv.contains("x"));
        assert_eq!(fv.len(), 1);
    }

    #[test]
    fn fv_application() {
        // FV(x y) = {x, y}
        let t = Term::app(Term::var("x"), Term::var("y"));
        let fv = free_variables(&t);
        assert_eq!(fv.len(), 2);
        assert!(fv.contains("x") && fv.contains("y"));
    }

    #[test]
    fn fv_abstraction_binds() {
        // FV(λx. x) = {}  (x is bound)
        let t = Term::abs("x", Term::var("x"));
        assert!(free_variables(&t).is_empty());
    }

    #[test]
    fn fv_abstraction_free_body() {
        // FV(λx. y) = {y}  (x removed, y stays free)
        let t = Term::abs("x", Term::var("y"));
        let fv = free_variables(&t);
        assert_eq!(fv.len(), 1);
        assert!(fv.contains("y"));
    }

    #[test]
    fn fv_shadowing() {
        // FV(λx. λx. x) = {}  (inner λx binds; remove handles it)
        let t = Term::abs("x", Term::abs("x", Term::var("x")));
        assert!(free_variables(&t).is_empty());
    }

    #[test]
    fn fv_mixed() {
        // FV(λx. x y) = {y}   body is (x y), x removed, y stays
        let t = Term::abs("x", Term::app(Term::var("x"), Term::var("y")));
        let fv = free_variables(&t);
        assert_eq!(fv.len(), 1);
        assert!(fv.contains("y"));
    }

    // --------------------------- SUBSTITUTION: variable
    #[test]
    fn subst_var_same() {
        // x[x := y]  ->  y   (same variable, replaced)
        let m = Term::var("x");
        let n = Term::var("y");
        assert_eq!(substitute(&m, "x", &n), Term::var("y"));
    }

    #[test]
    fn subst_var_other() {
        // z[x := y]  ->  z   (different variable, unchanged)
        let m = Term::var("z");
        let n = Term::var("y");
        assert_eq!(substitute(&m, "x", &n), Term::var("z"));
    }

    #[test]
    fn subst_var_replaced_by_compound() {
        // x[x := (g h)]  ->  g h   (N is not just a variable)
        let m = Term::var("x");
        let n = Term::app(Term::var("g"), Term::var("h"));
        assert_eq!(
            substitute(&m, "x", &n),
            Term::app(Term::var("g"), Term::var("h"))
        );
    }

    // --------------------------- SUBSTITUTION: application

    #[test]
    fn subst_app_both_sides() {
        // (x x)[x := y]  ->  y y   (substitute in both children)
        let m = Term::app(Term::var("x"), Term::var("x"));
        let n = Term::var("y");
        assert_eq!(
            substitute(&m, "x", &n),
            Term::app(Term::var("y"), Term::var("y"))
        );
    }

    #[test]
    fn subst_app_mixed() {
        // (x z)[x := y]  ->  y z   (only x replaced, z untouched)
        let m = Term::app(Term::var("x"), Term::var("z"));
        let n = Term::var("y");
        assert_eq!(
            substitute(&m, "x", &n),
            Term::app(Term::var("y"), Term::var("z"))
        );
    }

    #[test]
    fn subst_app_nested() {
        // ((x x) x)[x := y]  ->  (y y) y   (recursion goes deep)
        let m = Term::app(Term::app(Term::var("x"), Term::var("x")), Term::var("x"));
        let n = Term::var("y");
        let expected = Term::app(Term::app(Term::var("y"), Term::var("y")), Term::var("y"));
        assert_eq!(substitute(&m, "x", &n), expected);
    }

    #[test]
    fn subst_app_compound_n() {
        // (x x)[x := (\a. a)]  ->  (\a. a) (\a. a)   (N is an abstraction, cloned twice)
        let m = Term::app(Term::var("x"), Term::var("x"));
        let n = Term::abs("a", Term::var("a"));
        let expected = Term::app(
            Term::abs("a", Term::var("a")),
            Term::abs("a", Term::var("a")),
        );
        assert_eq!(substitute(&m, "x", &n), expected);
    }
}
