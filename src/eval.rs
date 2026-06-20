use crate::analysis::Term;
use std::collections::HashSet;

// Notions used for the evaluation:
//
// bound/free,
// shadowing,
// capture, capture-avoiding substitution, Alpha-conversion
// redex (reducible expression): An application where left side is an abstraction
// Beta-reduction
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
// --- Redex, Beta-reduction ---
//   A Beta Redex: Application(Abstraction(x, M), N)
//   A Beta-reduction is the result of substituting the argument into the abstraction's body
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

fn fresh_name(base: &str, forbidden: &HashSet<String>) -> String {
    let mut counter: usize = 1;
    loop {
        let freshname = format!("{}{}", base, counter);
        if forbidden.contains(&freshname) {
            counter += 1;
            continue;
        }
        return freshname;
    }
}

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
        Term::Abstraction(v, body) => {
            // (\v. M)[x:=N]):
            // 1. v == x:  \v. M is unchanged, (v shadows x; stop don't recurse)
            // 2. v != x AND v not in FV(N),   \v. (M[x:=N))  (safe, recurse)
            // 3. v != x AND v in FV(N): CAPTURE risk -> alpha-rename v to fresh w, then recurse
            if v == x {
                m.clone()
            } else {
                let mut fv = free_variables(n);
                if fv.contains(v.as_str()) {
                    // Capture risk -> rename
                    // 1. Get free variables in body because we don't want to use a name that
                    //    is free in the body and fall back in the same situation
                    // 2. Also add x in the forbidden name to avoid to fall back into shadow
                    let body_fv = free_variables(body);
                    fv.extend(body_fv);
                    fv.insert(x.to_string());
                    // We are ready to generate a fresh name...
                    let freshname = fresh_name(v.as_str(), &fv);
                    // We need to substitute the freshname in the body
                    let renamed_body = substitute(body, v.as_str(), &Term::var(freshname.as_str()));
                    // Now we can safely recurse
                    Term::abs(&freshname, substitute(&renamed_body, x, n))
                } else {
                    Term::abs(v, substitute(body, x, n))
                }
            }
        }
    }
}

// (λx. y) Ω
// where Ω = (λx. x x) (λx. x x)
// it is App(Abs(x, y), Ω)
//   => An abstraction on the left => it is a Redex
// but Ω has also some redex... So what to do?
// 1. Applicative order: reduce argument first
// 2. Normal order: always reduce left first
// With Ω there is a trap because it cannot be reduced...
fn reduce_once(m: &Term) -> Option<Term> {
    match m {
        Term::Variable(_) => None,
        Term::Application(f, arg) => {
            if let Term::Abstraction(var, body) = f.as_ref() {
                Some(substitute(body, var, arg))
            } else {
                // f is not a redex. We try to reduce left, if it is not
                // reduced we test right.
                match reduce_once(f.as_ref()) {
                    Some(f2) => Some(Term::app(f2, arg.as_ref().clone())),
                    None => {
                        reduce_once(arg.as_ref()).map(|arg2| Term::app(f.as_ref().clone(), arg2))
                    }
                }
            }
        }
        Term::Abstraction(v, body) => {
            // f itself is not a redex, check if there is one in the body
            reduce_once(body).map(|b| Term::abs(v, b))
        }
    }
}

#[derive(Debug)]
pub enum Reduction {
    NormalForm(Term),      // reduced fully: this is the final Term
    MaxStepsReached(Term), // stopped at the step limit; this Term can be resumed.
}

fn normalize(m: &Term, max_steps: usize) -> Reduction {
    let mut current = m.clone();

    for _ in 0..max_steps {
        match reduce_once(&current) {
            None => return Reduction::NormalForm(current),
            Some(t) => current = t,
        }
    }
    Reduction::MaxStepsReached(current)
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
    #[test]
    fn subst_abs_shadowing() {
        // (λx. x)[x := y]  ->  λx. x   (case 1: bound var x shadows the substituted x)
        let m = Term::abs("x", Term::var("x"));
        let n = Term::var("y");
        assert_eq!(substitute(&m, "x", &n), Term::abs("x", Term::var("x")));
    }

    #[test]
    fn subst_abs_shadowing_keeps_lambda() {
        // (λx. x x)[x := y]  ->  λx. x x   (unchanged, lambda preserved)
        let m = Term::abs("x", Term::app(Term::var("x"), Term::var("x")));
        let n = Term::var("y");
        assert_eq!(
            substitute(&m, "x", &n),
            Term::abs("x", Term::app(Term::var("x"), Term::var("x")))
        );
    }

    #[test]
    fn subst_abs_safe_descent() {
        // (λy. x)[x := a]  ->  λy. a   (case 2: y != x, y not in FV(a), descend)
        let m = Term::abs("y", Term::var("x"));
        let n = Term::var("a");
        assert_eq!(substitute(&m, "x", &n), Term::abs("y", Term::var("a")));
    }

    #[test]
    fn subst_abs_descent_compound() {
        // (λy. x y)[x := a]  ->  λy. a y   (descend into body, only x replaced)
        let m = Term::abs("y", Term::app(Term::var("x"), Term::var("y")));
        let n = Term::var("a");
        assert_eq!(
            substitute(&m, "x", &n),
            Term::abs("y", Term::app(Term::var("a"), Term::var("y")))
        );
    }

    #[test]
    fn subst_abs_no_capture_safe_name() {
        // (λz. x)[x := y]  ->  λz. y   (z != y, so no capture; safe descent)
        let m = Term::abs("z", Term::var("x"));
        let n = Term::var("y");
        assert_eq!(substitute(&m, "x", &n), Term::abs("z", Term::var("y")));
    }

    #[test]
    fn fresh_name_first_free() {
        // base "x", nothing forbidden -> "x1"
        let forbidden = HashSet::new();
        assert_eq!(fresh_name("x", &forbidden), "x1");
    }

    #[test]
    fn fresh_name_skips_taken() {
        // base "x", x1 taken -> "x2"
        let forbidden: HashSet<String> = ["x1".to_string()].into_iter().collect();
        assert_eq!(fresh_name("x", &forbidden), "x2");
    }

    #[test]
    fn fresh_name_skips_several() {
        // base "x", x1 and x2 taken -> "x3"
        let forbidden: HashSet<String> = ["x1".to_string(), "x2".to_string()].into_iter().collect();
        assert_eq!(fresh_name("x", &forbidden), "x3");
    }

    #[test]
    fn fresh_name_ignores_unrelated() {
        // forbidden names that don't match the pattern don't matter
        let forbidden: HashSet<String> = ["y".to_string(), "z1".to_string()].into_iter().collect();
        assert_eq!(fresh_name("x", &forbidden), "x1");
    }

    #[test]
    fn subst_capture_simple() {
        // (λy. x)[x := y]  ->  λy1. y   (NOT λy. y, which would be capture!)
        // the bound y is renamed to avoid capturing the free y from N
        let m = Term::abs("y", Term::var("x"));
        let n = Term::var("y");
        let result = substitute(&m, "x", &n);
        // expected: λy1. y
        assert_eq!(result, Term::abs("y1", Term::var("y")));
    }

    #[test]
    fn subst_capture_in_body() {
        // (λy. x y)[x := y]  ->  λy1. y y1
        // body x y : x becomes the free y (from N), the bound y becomes y1
        let m = Term::abs("y", Term::app(Term::var("x"), Term::var("y")));
        let n = Term::var("y");
        let result = substitute(&m, "x", &n);
        assert_eq!(
            result,
            Term::abs("y1", Term::app(Term::var("y"), Term::var("y1")))
        );
    }

    #[test]
    fn subst_no_capture_when_safe() {
        // (λy. x)[x := z]  ->  λy. z   (z != y, no capture, no rename)
        let m = Term::abs("y", Term::var("x"));
        let n = Term::var("z");
        assert_eq!(substitute(&m, "x", &n), Term::abs("y", Term::var("z")));
    }

    #[test]
    fn reduce_identity_app() {
        // (λx. x) y  ->  y   (one redex, reduced)
        let m = Term::app(Term::abs("x", Term::var("x")), Term::var("y"));
        assert_eq!(reduce_once(&m), Some(Term::var("y")));
    }

    #[test]
    fn reduce_no_redex() {
        // x y  ->  None  (no redex, normal form)
        let m = Term::app(Term::var("x"), Term::var("y"));
        assert_eq!(reduce_once(&m), None);
    }

    #[test]
    fn reduce_variable() {
        // x  ->  None
        assert_eq!(reduce_once(&Term::var("x")), None);
    }

    #[test]
    fn reduce_inside_abstraction() {
        // λz. ((λx. x) y)  ->  λz. y   (redex in the body)
        let inner = Term::app(Term::abs("x", Term::var("x")), Term::var("y"));
        let m = Term::abs("z", inner);
        assert_eq!(reduce_once(&m), Some(Term::abs("z", Term::var("y"))));
    }

    #[test]
    fn reduce_left_first() {
        // ((λx. x) y) z  ->  y z   (reduce the left redex, keep z)
        let left = Term::app(Term::abs("x", Term::var("x")), Term::var("y"));
        let m = Term::app(left, Term::var("z"));
        assert_eq!(
            reduce_once(&m),
            Some(Term::app(Term::var("y"), Term::var("z")))
        );
    }

    #[test]
    fn reduce_outer_before_arg() {
        // (λx. y) ((λz. z) w)  ->  y   (outer redex first; argument NOT reduced)
        // normal order: the outer redex is reduced, x unused, so the whole arg is dropped
        let arg = Term::app(Term::abs("z", Term::var("z")), Term::var("w"));
        let m = Term::app(Term::abs("x", Term::var("y")), arg);
        assert_eq!(reduce_once(&m), Some(Term::var("y")));
    }

    #[test]
    fn normalize_reaches_normal_form() {
        // (λx. x) y  ->  NormalForm(y)
        let m = Term::app(Term::abs("x", Term::var("x")), Term::var("y"));
        match normalize(&m, 100) {
            Reduction::NormalForm(t) => assert_eq!(t, Term::var("y")),
            _ => panic!("expected normal form"),
        }
    }

    #[test]
    fn normalize_multi_step() {
        // (λx. x) ((λy. y) z)  ->  NormalForm(z)   (two reductions)
        let inner = Term::app(Term::abs("y", Term::var("y")), Term::var("z"));
        let m = Term::app(Term::abs("x", Term::var("x")), inner);
        match normalize(&m, 100) {
            Reduction::NormalForm(t) => assert_eq!(t, Term::var("z")),
            _ => panic!("expected normal form"),
        }
    }

    #[test]
    fn normalize_already_normal() {
        // x y  is already in normal form -> NormalForm(x y), zero steps
        let m = Term::app(Term::var("x"), Term::var("y"));
        match normalize(&m, 100) {
            Reduction::NormalForm(t) => assert_eq!(t, Term::app(Term::var("x"), Term::var("y"))),
            _ => panic!("expected normal form"),
        }
    }

    #[test]
    fn normalize_drops_unused_diverging_arg() {
        // (λx. y) Ω  ->  NormalForm(y)   even though Ω loops, normal order drops it
        let omega_half = Term::abs("a", Term::app(Term::var("a"), Term::var("a")));
        let omega = Term::app(omega_half.clone(), omega_half);
        let m = Term::app(Term::abs("x", Term::var("y")), omega);
        match normalize(&m, 100) {
            Reduction::NormalForm(t) => assert_eq!(t, Term::var("y")),
            _ => panic!("expected normal form"),
        }
    }

    #[test]
    fn normalize_hits_limit_on_omega() {
        // Ω alone loops forever -> MaxStepsReached
        let omega_half = Term::abs("a", Term::app(Term::var("a"), Term::var("a")));
        let omega = Term::app(omega_half.clone(), omega_half);
        match normalize(&omega, 50) {
            Reduction::MaxStepsReached(_) => {} // good, it stopped
            _ => panic!("expected max steps reached"),
        }
    }
}
