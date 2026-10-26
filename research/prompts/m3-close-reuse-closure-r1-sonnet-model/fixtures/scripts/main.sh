#!/usr/bin/env bash
run_step() {
  "$@"
}
run_text() {
  bash -c "$2"
}
run_step bash scripts/inner.sh
