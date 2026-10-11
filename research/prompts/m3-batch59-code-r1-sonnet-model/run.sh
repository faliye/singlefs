#!/usr/bin/env bash
# usage: run.sh <repo dir> <logfile> <cargo args...>
set -u
R=/home/fy5090/code/singlefs/scripts
repo="$1"; log="$2"; shift 2
cd "$repo" || exit 99
export CARGO_TARGET_DIR=/home/fy5090/.local/share/singlefs/work/m3-batch59-code-r1-sonnet/target
nice -n 19 bash $R/capped.sh 15 bash $R/run-with-memory-cap.sh 10G cargo "$@" > "$log" 2>&1
echo "exit=$?" >> "$log"
