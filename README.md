# CP Project: Concurrent Pandemic Simulation – Rust Template 🦀

Reference implementation and template for the concurrent programming project 2023.

## Structure

This project is structured as follows:

- `crates/`: Rust crates of the project.
  - `spread-sim/`: Command line interface and main binary.
  - `spread-sim-core/`: Core data structures and algorithms.
  - `spread-sim-rocket/`: Your concurrent implementation goes here.
  - `spread-sim-slug/`: Sequential reference implementation.
  - `spread-sim-tests/`: Public tests and testing infrastructure.
- `scenarios`: Some sample scenarios.

Note that our testing infrastructure copies the `spread-sim-rocket` crate into our testing framework. This means: (a) You must not modify the `spread-sim-core` crate or add any dependencies to local crates or files and (b) you may add third-party libraries to the `spread-sim-rocket` crate. If you add third-party libraries, be prepared to argue why they are correct (especially if they contain `unsafe` code).


## Install Rust

Use `rustup` to install Rust and Cargo: https://rustup.rs.

## Building and Testing

This project uses Rust `nightly-2023-06-23` because our testing infrastructure uses unstable features. Note that you should not use any unstable features yourself as they may contain soundness holes, i.e., may render Rust's safety guarantees void. As long as you do not explicitly enable any unstable features yourself, you may argue for the correctness of your code based on Rust stable 1.70. The file `rust-toolchain.toml` takes care of configuring your development environment (e.g., `cargo` should automatically use the correct version).

As usual, you can use `cargo build`, to build the project, `cargo test`, to run the tests, and `cargo doc`, to build the documentation. Note, however, that `cargo` does run tests concurrently which is likely to interfere with the performance of the simulation. Hence, we highly recommend running the tests with:

```
cargo test --release -- --test-threads=1
```

For your convenience, we provide a [`justfile`](https://just.systems/man/en/). To install `just` via Cargo, run `cargo install just`. You can then run tests simply with `just test`. Note that our `.gitlab-ci.yml` also uses `just`. In addition to `just test`, the command `just lint` performs linting (style checking and running [Clippy](https://doc.rust-lang.org/clippy/)) and the command `just doc` builds the documentation (including private items). You can also use `just run` to run the main binary (compiled in release mode).

To build and open the documentation, run `just doc --open`.


## Integrated Development Environment

We recommend you use a proper _Integrated Development Environment_ (IDE) for this project. A good open source IDE is [VS Code](https://code.visualstudio.com/). Which IDE or editor you use is up to you. However, we only provide help for VS Code. In case you use something else, do not expect help.

In case you decide to use VS Code, open the `vscode.code-workspace` workspace. After opening the workspace, VS Code should ask you whether you want to install the *recommended extensions*. For maximal convenience, please do so. In particular, the *Rust Analyzer* extension is highly recommended.
