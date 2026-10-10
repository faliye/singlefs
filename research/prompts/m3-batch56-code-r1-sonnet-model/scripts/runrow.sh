#!/usr/bin/env bash
# usage: runrow.sh <idx>
W=$HOME/.local/share/singlefs/work/m3-batch56-code-r1-sonnet
F=$HOME/.local/share/singlefs/work/m3-batch56-code-r1/repo
i=$1
python3 -I $W/tsvrow.py $F/crates/mutations.tsv $i $W/mutdefs/row$i || exit 9
rel=$(cat $W/mutdefs/row$i.rel)
args=$(cat $W/mutdefs/row$i.args)
echo "== row $i :: $(cat $W/mutdefs/row$i.name | cut -c1-100)"
echo "   args: $args"
bash $W/mut.sh row$i "$rel" $W/mutdefs/row$i.old $W/mutdefs/row$i.new $args
