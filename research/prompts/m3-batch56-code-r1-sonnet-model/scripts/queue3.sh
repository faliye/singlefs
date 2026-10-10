#!/usr/bin/env bash
W=$HOME/.local/share/singlefs/work/m3-batch56-code-r1-sonnet
cd $W
echo "== fuzz new"; bash fuzzrun.sh fuzz-new new
echo "== fuzz old"; bash fuzzrun.sh fuzz-old old
echo "== t789 tombstone product-mode-only weakenings"
bash mutm.sh t789 crates/singlefs-checker/src/walk.rs t7,t8,t9 -p singlefs-harness --test checker_known_bad_images --test container_trees_grow_a_level_when_the_root_is_full 2>&1 | cut -c1-200
for i in 12 13 14 21; do
  echo "== slow row $i"
  python3 -I $W/tsvrow.py $HOME/.local/share/singlefs/work/m3-batch56-code-r1/repo/crates/mutations.tsv $i $W/mutdefs/row$i >/dev/null
  echo "args: $(cat $W/mutdefs/row$i.args)"
  bash mut.sh row$i "$(cat $W/mutdefs/row$i.rel)" $W/mutdefs/row$i.old $W/mutdefs/row$i.new $(cat $W/mutdefs/row$i.args) 2>&1 | cut -c1-220
done
