set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

build:
    cargo build

run:
    cargo run

test:
    cargo nextest run

clippy *ARGS:
    cargo clippy --all-targets --all-features --tests --benches {{ARGS}}