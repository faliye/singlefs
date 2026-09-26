awk -F'\t' '$1=="59-crates-mutation-replay.sh"{print $2}' .claude/gate.d/stage-inputs.tsv
