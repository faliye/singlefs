#!/usr/bin/env bash
# run-mut.sh <变异名> : 套变异、编 tree-mut 的 s2_probe、跑两批、写 runs/M/<名>.sum
set -u
W=$HOME/.local/share/singlefs/work/m3-batch10z-no-code-r1-sonnet
R=/home/fy5090/code/singlefs/scripts
name=$1
mkdir -p $W/runs/M
python3 $W/scripts/mutate.py $name || exit 9
cd $W/tree-mut
export CARGO_TARGET_DIR=$W/target-mut
nice -n 19 bash $R/run-with-memory-cap.sh 12G bash $R/capped.sh 8 cargo test --release --offline -p singlefs-harness --test s2_probe --no-run > $W/runs/M/$name.build 2>&1 || { echo BUILDFAIL; tail -20 $W/runs/M/$name.build; exit 8; }
B=$(ls -t $W/target-mut/release/deps/s2_probe-* | grep -v '\.d$' | head -1)
mk(){ python3 -c "import sys;print(sys.argv[1]*int(sys.argv[2]))" "$1" "$2"; }
run1(){ # $1 标签 其余 env
  lab=$1; shift
  ( cd $W/tree-mut; env "$@" S2_NOCHECK=1 S2_LABEL=$name-$lab nice -n 19 bash $R/run-with-memory-cap.sh 1700M bash $R/capped.sh 1 $B --nocapture s2_sweep > $W/runs/M/$name.$lab.log 2>&1; echo $? > $W/runs/M/$name.$lab.rc )
}
run1 s1 S2_SLOTS=1536 S2_PREFIXES=$(mk o 40) S2_RS="b2;b4" S2_FAULTS="rall;racct,rlin;racct" S2_ALPHABET=onsFb S2_COUNT=10 S2_SHORTEST=25 S2_LONGEST=45 S2_SEED=31 &
run1 s2 S2_SLOTS=1536 S2_PREFIXES=$(mk s 22) S2_RS="b8;b6" S2_FAULTS="racct;rall" S2_ALPHABET=onsF S2_COUNT=10 S2_SHORTEST=25 S2_LONGEST=45 S2_SEED=32 &
run1 s3 S2_SLOTS=768 S2_PREFIXES=$(mk o 20) S2_RS="b2;b4" S2_FAULTS="rall;racct,rlin" S2_ALPHABET=onsFb S2_COUNT=10 S2_SHORTEST=25 S2_LONGEST=45 S2_SEED=33 &
run1 s4 S2_SLOTS=3072 S2_PREFIXES=$(mk on 20) S2_RS="b3;b5" S2_FAULTS="rall;racct" S2_ALPHABET=onsFb S2_COUNT=10 S2_SHORTEST=25 S2_LONGEST=45 S2_SEED=34 &
run1 s5 S2_PERSIST=1 S2_SLOTS=1536 S2_PREFIXES=$(mk o 10) S2_RS="b6;b4" S2_FAULTS="rroot,racct,rlin;rroot,rall;rroot" S2_ALPHABET=onsF S2_COUNT=10 S2_SHORTEST=25 S2_LONGEST=45 S2_SEED=35 &
run1 s6 S2_PERSIST=1 S2_SLOTS=3072 S2_PREFIXES=$(mk o 20) S2_RS="b6;b3" S2_FAULTS="rroot,racct,rlin;rroot,rall" S2_ALPHABET=onsF S2_COUNT=10 S2_SHORTEST=25 S2_LONGEST=45 S2_SEED=36 &
run1 s7 S2_PERSIST=1 S2_SLOTS=1536 S2_PREFIXES=$(mk o 10) S2_RS="b6;b4" S2_FAULTS="rroot,racct,rlin;rroot,rall;rroot" S2_ALPHABET=onsF S2_COUNT=8 S2_SHORTEST=3 S2_LONGEST=12 S2_SEED=37 S2_ACTIONS=U,oU,ooU,FU,oFU,nU,sU,nnnU,ooooU,ssU,FFU,oooooooU &
wait
{
 for lab in s1 s2 s3 s4 s5 s6 s7; do
  f=$W/runs/M/$name.$lab.log
  tot=$(grep -c 'S2 label' $f)
  ho=$(grep -o 'S2 label.*' $f | grep -vc 'c_handed_out_max=0 ')
  both=$(grep -o 'S2 label.*' $f | grep -vc 'both_max=0 ')
  cu=$(grep -o 'S2 label.*' $f | sed -E 's/.*c_unblocked_mount=([0-9]+) c_unblocked_max=([0-9]+).*/\1 \2/' | awk '$2>$1' | wc -l)
  echo "$name $lab histories=$tot handed_out>0:$ho both>0:$both unblocked_grew:$cu rc=$(cat $W/runs/M/$name.$lab.rc)"
 done
} | tee $W/runs/M/$name.sum
