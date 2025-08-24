set shell := ["nu", "-c"]

[private]
default:
  just -l

workspace:
  cargo test --workspace --lib

test package:
  cargo test --package {{package}} --lib

run filter="trace":
  with-env { RUST_LOG: "wollok={{filter}}" } { cargo run }
