The grammar of lambda calculus
1. variable        x
1. application     (M N)      two lambda terms placed side by side
1. abstraction     (λx. M)    λ, variable, dot, a lambda term

- Greedy λ (or Greedy lambda): the body extends as far to the right as possible.
- Left-associative application: f g h = (f g) h.

- You can use [rwlrap](https://github.com/hanslub42/rlwrap) for a better experience:
```rust
❯ rlwrap cargo run
   Compiling lambda_calculus v0.1.0 (/home/gthouvenin/devel/rust/lambda_calculus)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.17s
     Running `target/debug/lambda_calculus`
LambdaCalculus version 0.1
Enter :quit to quit, :help for help

λ> :env
AND -> λp. λq. p q p
FALSE -> λx. λy. y
TRUE -> λx. λy. x
OR -> λp. λq. p p q
λ> AND TRUE FALSE
λ> :current
(λp. λq. p q p) (λx. λy. x) (λx. λy. y)
λ> :help
:current   show the current term
:env       dump all bindings in the current environmement
:help      show this help
:let       bind <NAME> to current into env. NAME must be uppercase.
:quit      quit the REPL
:reduce    reduce the current term to normal form
:step      reduce the current term by one step
λ> :reduce
Normal form: λx. λy. y
λ> AND TRUE TRUE
λ> :current
(λp. λq. p q p) (λx. λy. x) (λx. λy. x)
λ> :reduce
Normal form: λx. λy. x
λ> :quit
```
