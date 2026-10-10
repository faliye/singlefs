#!/usr/bin/env bash
# 把 $WORK/bins.tsv（名字<TAB>可执行文件）里每个 harness 测试二进制各跑一遍（--nocapture，探针行 B33* 进日志），一次调用不超过 BUDGET 秒；
# 没跑完的下次调用接着跑（有 .rc 的跳过；超时被杀的没有 .rc，下次从头再来）。
# 用法：REPO=… WORK=… tools/sweep.sh；并行 P 个，每个二进制的线程上限 CAP（默认 3，3 × 3 = 9 不超过 10）。
source "$(dirname "${BASH_SOURCE[0]}")/lib.sh"
BUDGET=${BUDGET:-225}; P=${P:-3}; CAP=${CAP:-3}
export SWEEP_DEADLINE=$(( $(date +%s) + BUDGET )) SWEEP_DIR="$WORK/sweep" REPO TREE CAP MEM
mkdir -p "$SWEEP_DIR"
job() {
    name="$1"; exe="$2"
    [[ -f "$SWEEP_DIR/$name.rc" ]] && exit 0
    remaining=$(( SWEEP_DEADLINE - $(date +%s) ))
    (( remaining < 25 )) && exit 0
    cd "$TREE/crates/singlefs-harness" || exit 2
    CARGO_MANIFEST_DIR="$TREE/crates/singlefs-harness" nice -n 19 timeout "$remaining" \
        bash "$REPO/research/scripts/run-with-memory-cap.sh" "$MEM" bash "$REPO/research/scripts/capped.sh" "$CAP" "$exe" --nocapture > "$SWEEP_DIR/$name.log" 2>&1
    echo $? > "$SWEEP_DIR/$name.rc"
}
export -f job
awk -F'\t' '{printf "%s%c%s%c", $1, 0, $2, 0}' "$WORK/bins.tsv" | xargs -0 -n2 -P "$P" bash -c 'job "$0" "$1"'
done_count=$(ls "$SWEEP_DIR"/*.rc 2>/dev/null | wc -l); total=$(wc -l < "$WORK/bins.tsv")
echo "SWEEP done=$done_count total=$total nonzero_rc=$(cat "$SWEEP_DIR"/*.rc 2>/dev/null | grep -vc '^0$')"
