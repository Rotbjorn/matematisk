# matematisk

Matematisk (or matex?) is a new experimental mathematical programming language
planned to implement the best parts of various [Computer Algebra Systems][1]
and the best features of functional programming languages (while still allowing for
an imperative style of coding)

For documentation regarding the project, take a look into [docs](/docs).

## Installation

To build the CLI application:

```sh
git clone https://github.com/Rotbjorn/matematisk
cd matematisk/cli
cargo build --release
```

which will produce the optimized binary `target/matex-cli`.

[1]: https://en.wikipedia.org/wiki/Computer_algebra_system
