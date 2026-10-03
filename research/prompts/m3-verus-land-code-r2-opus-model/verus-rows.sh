#!/usr/bin/env bash
# 逐行把变异套到核的副本上跑 Verus（--no-cheating --rlimit 10），每行一份判定行写进 <出口目录>/summary.tsv。
# 用法：verus-rows.sh <核 crate 的干净副本目录（含 src/lib.rs）> <变异表> <出口目录> <行号…>
# 变异表第 2 段的路径若以 crates/singlefs-replay-prefix/ 开头，套在 <副本的上一级>/ 这棵假树上（副本目录名须是 singlefs-replay-prefix）。
set -u
clean_copy=$1; table=$2; out=$3; shift 3
verus_binary=$HOME/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus
script_directory=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"
for row in "$@"; do
    work=$out/work-$row
    rm -rf "$work"; mkdir -p "$work/crates"
    cp -a "$clean_copy" "$work/crates/singlefs-replay-prefix"
    if ! python3 "$script_directory/apply-row.py" "$table" "$row" "$work" > "$out/apply-$row.txt"; then
        printf '%s\t%s\t%s\n' "$row" "ANCHOR" "$(cat "$out/apply-$row.txt")" >> "$out/summary.tsv"
        rm -rf "$work"; continue
    fi
    ( cd "$work/crates/singlefs-replay-prefix" && "$verus_binary" --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --no-cheating --rlimit 10 src/lib.rs ) > "$out/verus-$row.log" 2>&1
    status=$?
    verdict=$(grep -o 'verification results:: [0-9]* verified, [0-9]* errors' "$out/verus-$row.log" | tail -n 1)
    rlimit=$(grep -c 'rlimit' "$out/verus-$row.log")
    printf '%s\t%s\t%s\trlimit_lines=%s\n' "$row" "rc=$status" "${verdict:-NO-VERDICT}" "$rlimit" >> "$out/summary.tsv"
    rm -rf "$work"
done
