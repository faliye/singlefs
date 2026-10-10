#!/usr/bin/env bash
# runjobs.sh <jobs file> <parallel>
W=$HOME/.local/share/singlefs/work/m3-batch10z-jk-code-r1-sonnet
xargs -P "$2" -L 1 bash -c 'cd '"$W"'; ./run.sh "$@"' _ < "$1"
