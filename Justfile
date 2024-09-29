list:
  just -l -u

watch:
  bacon clippy

format:
  cargo fmt

run ARGS:
  RUST_BACKTRACE=1 cargo run --color always {{ARGS}} 2> logs.txt

logs:
  less -R logs.txt
