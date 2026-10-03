#!/usr/bin/env bash
# 逐个变体跑 Verus（--no-cheating），每个变体的输出写 <目录>/<名>.log，判定行另收进 <目录>/verdicts.txt。
# 用法：run-verus.sh <verus 可执行文件> <变体目录> [多给 verus 的参数…]；整条经 research/scripts/run-with-memory-cap.sh 跑。
set -u
verus="$1"
dir="$2"
shift 2
: > "$dir/verdicts.txt"
for variant in "$dir"/v*/; do
    name="$(basename "$variant")"
    (cd "$variant" && "$verus" --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating "$@" src/lib.rs) > "$dir/$name.log" 2>&1
    rc=$?
    echo "$name rc=$rc $(grep -o 'verification results:: [0-9]* verified, [0-9]* errors' "$dir/$name.log" | head -1)" >> "$dir/verdicts.txt"
done
