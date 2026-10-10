#!/usr/bin/env bash
# 与 sweep.sh 同法，只是跑慢档（#[ignore]）：$WORK/jobs-ignored.tsv 每行 二进制名<TAB>可执行文件<TAB>测试名，一个测试一个任务（--ignored --exact）。
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
BUDGET=${BUDGET:-225}; P=${P:-3}; CAP=${CAP:-3}
export SWEEP_DEADLINE=$(( $(date +%s) + BUDGET )) SWEEP_DIR="$WORK/sweep-ignored" REPO TREE CAP MEM
mkdir -p "$SWEEP_DIR"
job() {
    bin="$1"; exe="$2"; test="$3"
    key="$bin.$test"
    [[ -f "$SWEEP_DIR/$key.rc" ]] && exit 0
    remaining=$(( SWEEP_DEADLINE - $(date +%s) ))
    (( remaining < 25 )) && exit 0
    cd "$TREE/crates/singlefs-harness" || exit 2
    CARGO_MANIFEST_DIR="$TREE/crates/singlefs-harness" nice -n 19 timeout "$remaining" \
        bash "$REPO/research/scripts/run-with-memory-cap.sh" "$MEM" bash "$REPO/research/scripts/capped.sh" "$CAP" "$exe" --ignored --exact "$test" --nocapture > "$SWEEP_DIR/$key.log" 2>&1
    echo $? > "$SWEEP_DIR/$key.rc"
}
export -f job
awk -F'\t' '{printf "%s%c%s%c%s%c", $1, 0, $2, 0, $3, 0}' "$WORK/jobs-ignored.tsv" | xargs -0 -n3 -P "$P" bash -c 'job "$0" "$1" "$2"'
echo "SWEEP-IGNORED done=$(ls "$SWEEP_DIR"/*.rc 2>/dev/null | wc -l) total=$(wc -l < "$WORK/jobs-ignored.tsv") rc_counts=$(cat "$SWEEP_DIR"/*.rc 2>/dev/null | sort | uniq -c | tr '\n' ' ')"
