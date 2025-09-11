set shell := ["nu", "-c"]

[private]
default:
  just -l

workspace:
  cargo nextest run --workspace

test package:
  cargo nextest run --package {{package}} --lib

run args filter="trace":
  with-env { RUST_LOG: "wollok={{filter}}" } { cargo run -- {{args}} }

run-release filter="info":
  with-env { RUST_LOG: "wollok={{filter}}" } { cargo run --release }
