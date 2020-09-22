# anthem [![GitHub Release](https://img.shields.io/github/release/potassco/anthem.svg?maxAge=3600)](https://github.com/potassco/anthem/releases) [![Build Status](https://img.shields.io/travis/potassco/anthem/master.svg?maxAge=3600&label=build%20%28master%29)](https://travis-ci.org/potassco/anthem?branch=master) [![Build Status](https://img.shields.io/travis/potassco/anthem/develop.svg?maxAge=3600&label=build%20%28develop%29)](https://travis-ci.org/potassco/anthem?branch=develop)

> Translate answer set programs to first-order theorem prover language

## Overview

`anthem` translates ASP programs (in the input language of [`clingo`](https://github.com/potassco/clingo)) to the language of first-order theorem provers such as [Prover9](https://www.cs.unm.edu/~mccune/mace4/).

## Usage

To verify that a program implements a specification, invoke `anthem` using the `verify-program` command:

```sh
$ anthem verify-program <program file> <specification file>...
```

The example for computing the floor of the square root of a number can be reproduced as follows:

```sh
$ anthem verify-program examples/example-2.{lp,spec,lemmas}
```

By default, `anthem` performs Clark’s completion on the translated formulas, detects which variables are integer, and simplifies the output by applying several basic transformation rules.

These processing steps can be turned off with the options `--no-complete`, `--no-simplify`, and `--no-detect-integers`.

## Building

`anthem` is built with Rust’s `cargo` toolchain.
After [installing Rust](https://rustup.rs/), `anthem` can be built as follows:

```sh
$ git clone https://github.com/potassco/anthem.git
$ cd anthem
$ cargo build --release
```

The `anthem` binary will then be available in the `target/release/` directory.
Alternatively, `anthem` can be used with `cargo` as follows:

```sh
$ cargo run -- verify-program 
```

## Contributors

* [Patrick Lühne](https://www.luehne.de)
