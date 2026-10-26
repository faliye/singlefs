#!/usr/bin/env bash
run_step() {
  "$@"
}
run_step helper_unrelated bash scripts/inner.sh
