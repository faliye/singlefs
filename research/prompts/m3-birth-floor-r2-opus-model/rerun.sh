#!/usr/bin/env bash
# m3-birth-floor-r2 攻方腿的原型复跑：把冻结副本拷到草稿目录，打上 proto.diff，经内存包装（4G）、线程上限 7 编 release，
# 依次跑原型包 bf-proto 的几份用例（每份一条命令、各自落一份日志），末尾只打汇总行。
# 用法：bash rerun.sh <冻结副本根（含 Cargo.toml 与 crates/）> <草稿目录> [段名…]
#   段名不给时跑全部：bad-r1six bad-r2 sweep-A0 sweep-A1 sweep-B0 sweep-B1 sweep-C0 sweep-C1 sweep-C2 sweep-raise-A stale-ow-a2
#   jwrap1-a1 jwrap1-a2 jwrap6-a2 crash-head crash-tail（本机合计约 35 分钟）。
#   另有 crash-full-raiseTo8、crash-full-rb1 两段（层 0 全量，约 2.8 小时、0.9 小时），只在点名时跑。
# 不跑 checker 档包的任何测试：bf-proto 只把 singlefs-checker-tier 当库链进来，调它的 judge_layer0_state_range；测试目标名里没有 layer0。
set -euo pipefail
frozen="$1"; scratch="$2"; shift 2
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
mkdir -p "$scratch/logs"
if [ ! -d "$scratch/tree" ]; then
    rsync -a --exclude target --exclude .git "$frozen"/ "$scratch/tree/"
    ( cd "$scratch/tree" && patch -p2 -s < "$here/proto.diff" )
fi
cap() {
    RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-攻方原型复跑，峰值表里没有这几条}" \
    RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-3600}" \
        nice -n 19 bash "$repo/research/scripts/run-with-memory-cap.sh" 4G bash "$repo/research/scripts/capped.sh" 7 "$@"
}
cd "$scratch/tree"
nice -n 19 bash "$repo/research/scripts/capped.sh" 7 cargo build --offline --release -p bf-proto --tests
t() { cap cargo test --offline --release -p bf-proto --test "$@" -- --nocapture; }
run() {
    local name="$1"; local log="$scratch/logs/$name.log"
    case "$name" in
        bad-r1six) BI2_SLOTS=50186,50277,50278,50280 t bf_bad_images > "$log" 2>&1 ;;
        bad-r2) BJ_STEP=3 cap cargo test --offline --release -p bf-proto --test bf_r2_bad -- --nocapture --test-threads 1 > "$log" 2>&1 ;;
        sweep-A0) R2_PREFIX=A R2_SHARD=0/2 t bf_r2_sweep final_images > "$log" 2>&1 ;;
        sweep-A1) R2_PREFIX=A R2_SHARD=1/2 t bf_r2_sweep final_images > "$log" 2>&1 ;;
        sweep-B0) R2_PREFIX=B R2_SHARD=0/2 t bf_r2_sweep final_images > "$log" 2>&1 ;;
        sweep-B1) R2_PREFIX=B R2_SHARD=1/2 t bf_r2_sweep final_images > "$log" 2>&1 ;;
        sweep-C0) R2_PREFIX=C R2_SHARD=0/3 t bf_r2_sweep final_images > "$log" 2>&1 ;;
        sweep-C1) R2_PREFIX=C R2_SHARD=1/3 t bf_r2_sweep final_images > "$log" 2>&1 ;;
        sweep-C2) R2_PREFIX=C R2_SHARD=2/3 t bf_r2_sweep final_images > "$log" 2>&1 ;;
        sweep-raise-A) R2_ALPHA=raise R2_PREFIX=A t bf_r2_sweep final_images > "$log" 2>&1 ;;
        stale-ow-a2) R2_STALE=1 R2_A=2 t bf_r2_sweep stale > "$log" 2>&1 ;;
        jwrap1-a1) R2_STALE=1 R2_JRING_MIB=1 R2_SW_UNITS=16 R2_A=1 R2_B=16,17,18,19,20,21,22,23,24 t bf_r2_sweep stale > "$log" 2>&1 ;;
        jwrap1-a2) R2_STALE=1 R2_JRING_MIB=1 R2_SW_UNITS=16 R2_A=2 R2_B=16,17,18,19,20,21,22,23,24 t bf_r2_sweep stale > "$log" 2>&1 ;;
        jwrap6-a2) R2_STALE=1 R2_JRING_MIB=6 R2_SW_UNITS=72 R2_A=2 R2_R=3 R2_B=22 t bf_r2_sweep stale > "$log" 2>&1 ;;
        crash-count) R2_COUNT_ONLY=1 R2_JRING_MIB=1 R2_SW_UNITS=16 R2_SUFFIXES=raiseTo8,raiseTo5,raiseTo9,remount,rb1,rb2 t bf_r2_crash > "$log" 2>&1 ;;
        crash-head) R2_JRING_MIB=1 R2_SW_UNITS=16 R2_STATE_LIMIT=700 R2_SUFFIXES=raiseTo8 t bf_r2_crash > "$log" 2>&1 ;;
        crash-tail) R2_JRING_MIB=1 R2_SW_UNITS=16 R2_STATE_START=195965 R2_STATE_LIMIT=700 R2_SUFFIXES=raiseTo8 t bf_r2_crash > "$log" 2>&1 ;;
        crash-full-raiseTo8) R2_JRING_MIB=1 R2_SW_UNITS=16 R2_SUFFIXES=raiseTo8 t bf_r2_crash > "$log" 2>&1 ;;
        crash-full-rb1) R2_JRING_MIB=1 R2_SW_UNITS=16 R2_SUFFIXES=rb1 t bf_r2_crash > "$log" 2>&1 ;;
        *) echo "段名 $name 不认得" >&2; exit 2 ;;
    esac
    echo "== $name"
    grep -E '^(R2_DONE|R2C_DONE|R2C_HISTORY|R2C_SKIPPED)|^R2 .*(CS2:(tf|beta)|pool-checker|refused)|^R2C name=.*(tf-red-state|layer0-red|crash:CS2:(tf-lenient|beta-lenient) )|^BI .*pool_checker|^BJ .*pool_checker|test result' "$log" | cut -c1-260
}
names=("$@")
if [ "${#names[@]}" -eq 0 ]; then
    names=(bad-r1six bad-r2 sweep-A0 sweep-A1 sweep-B0 sweep-B1 sweep-C0 sweep-C1 sweep-C2 sweep-raise-A stale-ow-a2 jwrap1-a1 jwrap1-a2 jwrap6-a2 crash-count crash-head crash-tail)
fi
for name in "${names[@]}"; do run "$name"; done
