# Project

Matematisk is at it's core a Rust library; the crate [`matex-compiler`](../compiler). All the other programs depend on this crate and use it in their own way:

- [`matex-cli`](../cli) is the most barebones version, a Rust crate providing a CLI to interface with the compiler, allowing you to execute files or enter a REPL-mode.
- [`matex-gui`](../gui) is a Rust crate providing a very experimental GUI using [egui][1]. The purpose of this GUI is to debug `matex-compiler` rather than to provide a user friendly application.
- [`web`](../web) is a SvelteKit project using [wasm-pack][2] to build the `matex-compiler` into a WASM-module for use in the browser. The end goal is to provide a good looking user application, similar to that of GeoGebra CAS,
WolframAlpha, GNU Octave etc.

## Goals

The goals of matematisk as of now is to implement a version
of matematisk that supports most of the basic needs of both a
programming language and a CAS-system, albeit non-performant.

### Core Features

These are the features needed before considering a 1.0 release:

- [ ] Automatic simplification of algebraic expressions
- [ ] Reactive variables, relationship between variables
- [ ] Functional programming constructs, `if`-expressions, `match`-expressions etc.
- [ ] Solve equations, polynomials, differentials, derivatives, integrals etc.
- [ ] Various common mathematical functions: `sin`, `cos`, `log` etc.
- [ ] Statistical functions: `mean` etc.
- [ ] Vectors and matrices
- [ ] Type-declarations
  - [ ] Define domain and range of functions
  - [ ] Union of types
- [ ] Graphing and plotting

### Planned Features

These features are not prioritized but are 100% planned for the future of matematisk, and
should thus be thought about when designing the language and the internal systems.

- [ ] Sets
- [ ] Logic

## Design

One of the core principles of the language is that the syntax
should be concise and easy to read, aiming to be similar to standard mathematical notation in most cases.

[1]: https://github.com/emilk/egui
[2]: https://github.com/rustwasm/wasm-pack
