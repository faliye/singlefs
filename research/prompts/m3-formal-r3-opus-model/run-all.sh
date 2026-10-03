#!/usr/bin/env bash
# m3-formal-r3 攻方腿：在打过 patch_attack.py 的副本编出的 E165 上跑各开关组合。
# 用法：run-all.sh <e165 二进制> <输出目录>；整条经 research/scripts/run-with-memory-cap.sh 跑，线程取 E165_THREADS（默认 8）。
set -u
binary="$1"
out="$2"
mkdir -p "$out"
: > "$out/exits.txt"
export E165_THREADS="${E165_THREADS:-8}"
run() {
    local name="$1"
    shift
    local variables=()
    while [[ $# -gt 0 && "$1" == *=* ]]; do
        variables+=("$1")
        shift
    done
    env ${variables[@]+"${variables[@]}"} "$binary" "$@" > "$out/$name.out" 2> "$out/$name.err"
    echo "$name rc=$?" >> "$out/exits.txt"
}
light_legal=(--bounds light --family legal)
run m0 "${light_legal[@]}"
run m0-adv --bounds light --family adversarial
run m0-seg1-A --bounds segment1 --family adversarial --only A
run gen1 ATTACK_GEN=1 ATTACK_SKIP_PRELUDE=1 "${light_legal[@]}"
run gen1-glue3 ATTACK_GEN=1 ATTACK_GLUE=3 ATTACK_SKIP_PRELUDE=1 "${light_legal[@]}"
run gen2 ATTACK_GEN=2 ATTACK_SKIP_PRELUDE=1 "${light_legal[@]}"
run gen2-glue2 ATTACK_GEN=2 ATTACK_GLUE=2 ATTACK_SKIP_PRELUDE=1 "${light_legal[@]}"
run glue1 ATTACK_GLUE=1 ATTACK_SKIP_PRELUDE=1 "${light_legal[@]}"
for switch in 1 2 3 4 5 6 7 8 9; do
    run "core$switch" ATTACK_CORE=$switch ATTACK_SKIP_PRELUDE=1 "${light_legal[@]}"
done
for switch in 1 2 3 5 7; do
    run "core$switch-adv" ATTACK_CORE=$switch ATTACK_SKIP_PRELUDE=1 --bounds light --family adversarial
done
