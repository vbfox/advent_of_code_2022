set windows-shell := ["powershell.exe", "-NoLogo", "-Command"]

build *ARGS:
    cargo build {{ARGS}}

run *ARGS:
    cargo run -- {{ARGS}}

test *ARGS:
    cargo nextest run {{ARGS}}

clippy *ARGS:
    cargo clippy --all-targets --all-features --tests --benches {{ARGS}}