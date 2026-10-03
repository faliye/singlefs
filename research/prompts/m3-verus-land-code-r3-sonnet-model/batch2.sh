#!/usr/bin/env bash
D=/tmp/claude-1000/m3-verus-land-code-r3-sonnet
export DIFF_THREADS=10
bash $D/run.sh $D/difftree $D/logs/batch2a.txt -- --test-threads=1 --nocapture a_parsed_counter_is_below_two_to_the_48 exhaustive_structural exhaustive_chain_and_named
DIFF_SEED=2 DIFF_N=2000000 bash $D/run.sh $D/difftree $D/logs/batch2b.txt -- --test-threads=1 --nocapture random_family
