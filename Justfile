list:
  just --list --unsorted

run *ARGS:
  cargo run {{ARGS}}

watch:
  #!/usr/bin/env bash
  set -Eeuo pipefail
  cleanup() {
    reset
  }
  trap 'cleanup' EXIT
  watchexec     \
    --clear     \
    --restart   \
    --watch src \
    --exts rs   \
    'cargo check --color always 2>&1 | less -K -R'
