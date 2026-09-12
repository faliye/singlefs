cd "$1/research" && PROBE_TRACE=1 ./target/release/probe_a anchor 2>&1 >/dev/null | head -8
