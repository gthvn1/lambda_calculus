The grammar of lambda calculus
1. variable        x
1. application     (M N)      two lambda terms placed side by side
1. abstraction     (λx. M)    λ, variable, dot, a lambda term

- Greedy λ (or Greedy lambda): the body extends as far to the right as possible.
- Left-associative application: f g h = (f g) h.

- You can use [rwlrap](https://github.com/hanslub42/rlwrap) for a better experience:
```rust
❯ rlwrap cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/lambda_calculus`
LambdaCalculus version 0.1
Enter :quit to quit, :help for help

λ> :help
:quit      quit the REPL
:current   show the current term
:step      reduce the current term by one step
:reduce    reduce the current term to normal form
:help      show this help
λ> (\x. x) y
λ> :step
step: y
λ> :step
it is already a normal form
λ> :current
y
λ> :quit
```
