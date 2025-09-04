set shell := ["nu", "-c"]

[private]
default:
  just -l

workspace:
  cargo nextest run --workspace

test package:
  cargo nextest run --package {{package}} --lib

run filter="trace":
  with-env { RUST_LOG: "wollok={{filter}}" } { cargo run }

run-release filter="info":
  with-env { RUST_LOG: "wollok={{filter}}" } { cargo run --release }
