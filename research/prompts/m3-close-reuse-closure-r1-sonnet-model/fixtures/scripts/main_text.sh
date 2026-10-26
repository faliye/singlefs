#!/usr/bin/env bash
run_text() {
  bash -c "$2"
}
run_text / "bash scripts/inner.sh"
