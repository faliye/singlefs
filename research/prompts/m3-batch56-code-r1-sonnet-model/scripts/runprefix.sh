#!/usr/bin/env bash
# usage: runprefix.sh <label> <name prefix>
W=$HOME/.local/share/singlefs/work/m3-batch56-code-r1-sonnet
F=$HOME/.local/share/singlefs/work/m3-batch56-code-r1/repo
python3 -I $W/tsvrow2.py $F/crates/mutations.tsv "$2" $W/mutdefs/$1 || exit 9
echo "== $1 :: $(cut -c1-90 $W/mutdefs/$1.name)"
bash $W/mut.sh $1 "$(cat $W/mutdefs/$1.rel)" $W/mutdefs/$1.old $W/mutdefs/$1.new $(cat $W/mutdefs/$1.args)
