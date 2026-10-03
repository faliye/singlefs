#!/usr/bin/env bash
# 逐行把变异套到核的副本上，照门禁的命令行起 Verus（--no-cheating --num-threads 2 --output-json --rlimit <R>），四行一批并行，
# 每行一份判定行（verus-judge-row.py）追加进 <出口目录>/summary.tsv。
# 用法：verus-rows.sh <核 crate 的干净副本（目录名 singlefs-replay-prefix）> <变异表（五段：名、文件、原文、替换文、期望；攻方行期望写 ->> <rlimit> <出口目录> <行号…>
set -u
clean_copy=$1; table=$2; rlimit=$3; out=$4; shift 4
repo=/home/fy5090/code/singlefs
verus_binary=$HOME/.local/share/singlefs/formal-tools/verus-0.2026.09.20/verus-x86-linux/verus
here=$(cd "$(dirname "$0")" && pwd)
mkdir -p "$out"
one_row() {
    local row=$1 work=$out/work-r$rlimit-$1
    rm -rf "$work"; mkdir -p "$work/crates"
    cp -a "$clean_copy" "$work/crates/singlefs-replay-prefix"
    if ! python3 "$here/apply-row.py" "$table" "$row" "$work" > "$out/apply-r$rlimit-$row.txt"; then
        printf '%s\trlimit=%s\tANCHOR\t%s\n' "$row" "$rlimit" "$(cat "$out/apply-r$rlimit-$row.txt")" > "$out/line-r$rlimit-$row.tsv"
        rm -rf "$work"; return
    fi
    ( cd "$work/crates/singlefs-replay-prefix" && nice -n 19 bash "$repo/research/scripts/run-with-memory-cap.sh" "${VERUS_MEMORY_CAP:-1G}" \
        "$verus_binary" --no-cheating --crate-type=lib --edition=2021 --crate-name singlefs_replay_prefix --num-threads 2 --output-json --rlimit "$rlimit" src/lib.rs ) \
        > "$out/stdout-r$rlimit-$row.json" 2> "$out/stderr-r$rlimit-$row.txt"
    local status=$?
    local expected
    expected=$(awk -v wanted="$row" -F '\t' 'NR==wanted {print $5}' "$table")
    printf 'rc=%s\t%s\n' "$status" "$(python3 "$here/verus-judge-row.py" "$row" "$rlimit" "$out/stdout-r$rlimit-$row.json" "$out/stderr-r$rlimit-$row.txt" "${expected:--}")" > "$out/line-r$rlimit-$row.tsv"
    rm -rf "$work"
}
batch=0
while (( $# > 0 )); do
    batch=$((batch + 1))
    rows=()
    for _ in 1 2 3 4; do (( $# > 0 )) && { rows+=("$1"); shift; }; done
    rm -f "$out"/b"$batch"-*.rc
    piece=0
    for row in "${rows[@]}"; do
        piece=$((piece + 1))
        { one_row "$row"; echo "$?" > "$out/b$batch-$piece.rc"; } &
    done
    wait
    count=$(ls "$out"/b"$batch"-*.rc 2>/dev/null | wc -l)
    if (( count != ${#rows[@]} )); then echo "batch $batch: rc files $count != ${#rows[@]}, void" >> "$out/summary.tsv"; continue; fi
    for row in "${rows[@]}"; do cat "$out/line-r$rlimit-$row.tsv" >> "$out/summary.tsv"; done
done
