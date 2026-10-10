#!/usr/bin/env bash
# repro.sh  把报告里引的关键探针各跑一遍，日志落 logs/final/<名>.log（piece.sh 起，经内存包装；不带 gpu、不跑 checker 档）
W=$HOME/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-sonnet
cd "$W"
o() { printf 'o%.0s' $(seq "$1"); }
run() { name=$1; shift; timeout 200 ./piece.sh "$name" "$@"; cp "runs/$name.log" "logs/final/$name.log"; echo "$name rc=$(cat runs/$name.rc)"; }
run t1-wide-report-384 S2_PREFIXES=$(o 20) 'S2_MID=' S2_RS=b6 'S2_FAULTS=racct,rlin' S2_ACTIONS=WWWW S2_LABEL=t1
run j0-unmount-remount-fault-lifted S2_NOCHECK=1 S2_PREFIXES=$(o 20) 'S2_MID=' 'S2_RS=b6;n;o' 'S2_FAULTS=racct,rlin;rall;rroot;racct' S2_ACTIONS=oXWW S2_LABEL=j0
run j1-unmount-remount-fault-persists S2_NOCHECK=1 S2_PERSIST=1 S2_PREFIXES=$(o 20) 'S2_MID=' 'S2_RS=b6;n;o' 'S2_FAULTS=racct,rlin;rall;rroot;racct' S2_ACTIONS=oXWW S2_LABEL=j1
for n in 0 20 60 90; do run k$n-push-fails-at-write-$n P_FAIL_AT=$n S2_PREFIXES=$(o 20) 'S2_MID=' S2_RS=b6 'S2_FAULTS=racct,rlin' S2_ACTIONS=oPKoooKoo S2_LABEL=k$n; done
run m5-crash-samples-b6 P_CRASH=1 P_CRASH_STRIDE=1 S2_PREFIXES=$(o 20) 'S2_MID=' S2_RS=b6 'S2_FAULTS=racct,rlin;rall' S2_ACTIONS=oPo S2_LABEL=m5
run m6-crash-samples-n P_CRASH=1 P_CRASH_STRIDE=1 S2_PREFIXES="$(o 40);$(o 80)" 'S2_MID=' S2_RS=n 'S2_FAULTS=racct,rlin' S2_ACTIONS=oPo S2_LABEL=m6
run m7-crash-samples-o-s P_CRASH=1 P_CRASH_STRIDE=1 S2_PREFIXES=$(o 12) 'S2_MID=' 'S2_RS=o;s' 'S2_FAULTS=racct;rall' S2_ACTIONS=oPo S2_LABEL=m7
run ff-stuck-then-push-still-refused H_STEP=20 S2_NOCHECK=1 S2_PREFIXES=$(o 12) 'S2_MID=' S2_RS=o 'S2_FAULTS=racct;none' S2_ACTIONS=HHHH S2_LABEL=ff
run fs-push-keeps-floor-and-data S2_PREFIXES=$(o 20) 'S2_MID=' 'S2_RS=b6;n' 'S2_FAULTS=racct,rlin' S2_ACTIONS=oPo S2_LABEL=fs
run i256-small-geometry S2_NOCHECK=1 S2_SLOTS=256 S2_PREFIXES="$(o 12);$(o 20);$(o 30);$(o 40);$(o 60)" 'S2_MID=' 'S2_RS=b6;n;o;b2' 'S2_FAULTS=racct,rlin;rall' S2_ACTIONS=WWW S2_LABEL=i256
run i288-small-geometry S2_NOCHECK=1 S2_SLOTS=288 S2_PREFIXES="$(o 12);$(o 20);$(o 30);$(o 40);$(o 60)" 'S2_MID=' 'S2_RS=b6;n;o;b2' 'S2_FAULTS=racct,rlin;rall' S2_ACTIONS=WWW S2_LABEL=i288
run pu-320-c-unblocked-at-mount S2_NOCHECK=1 S2_SLOTS=320 S2_PREFIXES=$(o 20) 'S2_MID=' S2_RS=b6 'S2_FAULTS=racct,rlin' S2_ACTIONS=oPoo S2_LABEL=pu1
run base-256-crash P_CRASH=1 P_CRASH_STRIDE=1 S2_SLOTS=256 S2_PREFIXES="$(o 20);$(o 30)" 'S2_MID=' S2_RS=b6 'S2_FAULTS=racct,rlin' S2_ACTIONS=oPoo S2_LABEL=base256
: > logs/final/jj-inject-multislot.log
for offs in 3 3,9 5,30,41 1,2,3 47 20,46; do run jj J_OFFSETS=$offs S2_PREFIXES=$(o 12) 'S2_MID=' S2_RS=o 'S2_FAULTS=none' S2_ACTIONS=oJPoo S2_LABEL=jj >/dev/null; echo "## J_OFFSETS=$offs" >> logs/final/jj-inject-multislot.log; grep -h 'DIAG\|S2 label' runs/jj.log >> logs/final/jj-inject-multislot.log; done; rm -f logs/final/jj.log
: > logs/final/jx-inject-with-abandoned-root.log
for k in $(seq 1 23); do run jx S2_PREFIXES=$(o 12) 'S2_MID=' S2_RS=o 'S2_FAULTS=racct' J_OFFSETS=$k S2_ACTIONS=XJPooo S2_LABEL=jx >/dev/null; echo "## J_OFFSETS=$k" >> logs/final/jx-inject-with-abandoned-root.log; grep -h 'DIAG\|S2 label' runs/jx.log >> logs/final/jx-inject-with-abandoned-root.log; done; rm -f logs/final/jx.log
: > logs/final/jo-overlap-with-abandoned.log
for k in 3 14 20; do run jo J_OVERLAP=1 S2_PREFIXES=$(o 12) 'S2_MID=' S2_RS=o 'S2_FAULTS=racct' J_OFFSETS=$k S2_ACTIONS=XJPooo S2_LABEL=jo >/dev/null; echo "## J_OFFSETS=$k" >> logs/final/jo-overlap-with-abandoned.log; grep -h 'DIAG\|S2 label' runs/jo.log >> logs/final/jo-overlap-with-abandoned.log; done; rm -f logs/final/jo.log
run jn-inject-without-isolation J_NOISO=1 S2_PREFIXES=$(o 12) 'S2_MID=' S2_RS=o 'S2_FAULTS=none' J_OFFSETS=9 S2_ACTIONS=oJPoo S2_LABEL=jn
