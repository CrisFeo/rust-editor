list:
  just --list

run *ARGS:
  cargo run {{ARGS}}

watch:
  #!/usr/bin/env bash
  set -euxo pipefail
  cleanup() {
    reset
  }
  trap 'cleanup' EXIT
  watchexec     \
    --restart   \
    --watch src \
    --exts rs   \
    'cargo check --color always 2>&1 | less -K -R'
