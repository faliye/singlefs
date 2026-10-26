#!/usr/bin/env bash
ssh -o BatchMode=yes "$PEER_SSH_HOST" "cd $(printf '%q' "$PEER_REPOSITORY_DIRECTORY") && python3 scripts/hosts/probe.py 0.2"
