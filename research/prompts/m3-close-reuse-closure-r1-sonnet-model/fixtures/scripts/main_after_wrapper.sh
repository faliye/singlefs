#!/usr/bin/env bash
start_driver() {
  bash scripts/wrap.sh --peer -- bash -c "$3"
}
start_driver one two "bash scripts/inner.sh"
