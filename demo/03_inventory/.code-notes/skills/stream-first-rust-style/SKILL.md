---
name: stream-first-rust-style
description: Prefer iterator-chain, combinator, and expression-oriented Rust style over explicit for loops and branch-heavy imperative code. Use when editing Rust code that traverses collections, validates data, transforms values, or handles Option/Result flows in this repository.
---

# Stream First Rust Style

## Rules

- Prefer iterator adapters such as `map`, `filter`, `find`, `flat_map`, `fold`, `collect`, `all`, `any`, and `for_each` over explicit `for` loops.
- Prefer `Option` and `Result` combinators such as `map`, `and_then`, `map_or`, `ok_or_else`, `then`, and `then_some` over branch-heavy `if` code when readability stays good.
- Prefer expression-oriented flow. Return expressions directly instead of storing intermediate mutable state for simple transforms.
- Use `match` or a small helper function when a chain would otherwise become deeply nested or harder to scan than straightforward expression code.
- In tests, replace manual iteration with `for_each`, `all`, collected comparisons, or chained assertions when behavior stays obvious.
