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
