#!/usr/bin/env bash
# 带「读盘日志只差」分栏的差分；T 是树目录名，L 是日志前缀
D=/tmp/claude-1000/m3-verus-land-code-r3-sonnet
T=$1; L=$2
export DIFF_THREADS=10 DIFF_N=3000000
bash $D/run.sh $D/$T $D/logs/${L}-a.txt -- --test-threads=1 --nocapture positive_controls_are_red random_family max_counter_family key_mismatch_family a_parsed_counter_is_below_two_to_the_48 exhaustive_structural exhaustive_chain_and_named
DIFF_SEED=2 DIFF_N=2000000 bash $D/run.sh $D/$T $D/logs/${L}-b.txt -- --test-threads=1 --nocapture random_family
