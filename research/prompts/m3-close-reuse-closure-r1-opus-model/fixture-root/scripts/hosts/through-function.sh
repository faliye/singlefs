#!/usr/bin/env bash
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
run_there "$PEER_REPOSITORY_DIRECTORY" "python3 scripts/hosts/probe.py 0.2"
