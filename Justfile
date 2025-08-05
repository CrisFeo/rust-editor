list:
  just -l -u

check:
  cargo clippy --quiet --color always

fmt:
  cargo fmt

test:
  cargo test --color always

run *ARGS:
  RUST_BACKTRACE=1 cargo run --color always {{ARGS}} 2> logs.txt

logs:
  less -R logs.txt

watch target:
  #!/bin/sh
  set -euf
  last_hash=''
  running_pid=''
  trap 'kill -s SIGINT $running_pid' SIGINT SIGTERM EXIT
  while :; do
    next_hash="$(ls -aR --full-time | grep '\.rs'| sha256sum)"
    if [ "$next_hash" != "$last_hash" ]; then
      last_hash="$next_hash"
      if [ -n "$running_pid" ]; then
        kill -s SIGINT "$running_pid"
      fi
      just {{target}} 2>&1 | less -R -K &
      running_pid="$!"
    fi
    sleep 1
  done
