#!/usr/bin/env bash
export DIFF_THREADS=6 DIFF_N=3000000
D=/tmp/claude-1000/m3-verus-land-code-r2-sonnet
bash $D/run.sh $D/tree $D/log_batch1.txt -- --test-threads=1 --nocapture positive_controls_are_red random_family max_counter_family key_mismatch_family
