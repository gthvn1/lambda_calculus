The grammar of lambda calculus
1. variable        x
1. application     (M N)      two lambda terms placed side by side
1. abstraction     (λx. M)    λ, variable, dot, a lambda term

- Greedy λ (or Greedy lambda): the body extends as far to the right as possible.
- Left-associative application: f g h = (f g) h.

```haskell
❯ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.00s
     Running `target/debug/lambda_calculus`
LambdaCalculus version 0.1
Enter /quit to quit

λ> (\f. \x. f (f x)) g z
(λf. λx. f (f x)) g z
Normal form: g (g z)
λ> (\x. x x)(\x. x x)
(λx. x x) (λx. x x)
Max steps reached: (λx. x x) (λx. x x)
λ> (\x. x) y
(λx. x) y
Normal form: y
λ> /quit
```
