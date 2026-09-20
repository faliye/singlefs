#!/usr/bin/env bash
D=/tmp/claude-1000/m2-supp3-item2-code-r2-opus
date -u
for m in "$@"; do echo "## $m"; $D/probe/run-mutant.sh "$m"; done
date -u
