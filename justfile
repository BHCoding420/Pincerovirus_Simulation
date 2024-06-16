default:
    @just --list

lint:
    cargo clippy --no-deps
    cargo fmt --check

build *ARGS:
    cargo build {{ARGS}}

test *ARGS:
    cargo test --release -- --test-threads=1 {{ARGS}}

doc *ARGS:
    cargo doc --document-private-items {{ARGS}}

fmt:
    cargo fmt

run *ARGS:
    cd {{invocation_directory()}} && cargo run --release --bin spread-sim -- {{ARGS}}