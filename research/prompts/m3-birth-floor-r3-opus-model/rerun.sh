#!/usr/bin/env bash
# m3-birth-floor-r3 攻方腿的原型复跑：把冻结副本拷到草稿目录、patch -p2 打上 proto.diff（含第二轮攻方原型与这一轮的改动），
# 经内存包装（2G）、线程上限 7 编 release，按段名跑原型包 bf-proto 的用例，每段一份日志、末尾打汇总行。
# 用法：bash rerun.sh <冻结副本根（含 Cargo.toml 与 crates/）> <草稿目录> [段名…]
#   段名不给时跑全部（本机这一轮各段单跑合计约 40 分钟，按 logs/ 里各段 seconds= 加的，推的）。
# 不跑 checker 档包的任何测试：bf-proto 只把 singlefs-checker-tier 当库链进来（层 0 的枚举函数）；测试目标名里没有 layer0。
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
    RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-攻方原型复跑与别的会话各跑各的}" \
    RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-3600}" \
        nice -n 19 bash "$repo/scripts/hosts/run-with-memory-cap.sh" 2G bash "$repo/scripts/hosts/capped.sh" 7 "$@"
}
cd "$scratch/tree"
nice -n 19 bash "$repo/scripts/hosts/capped.sh" 7 cargo build --offline --release -p bf-proto --tests
t() { local target="$1"; shift; cap cargo test --offline --release -p bf-proto --test "$target" "$@" -- --nocapture --test-threads 1; }
enum() { BF_WIDTH=384 BF_NO_CS1=1 R3_PREFIX=A R3_OPS="$1" R3_AFTER="$2" R3_STATE_START="$3" R3_STATE_LIMIT="$4" t bf_r3_gap gap_frequency_over_crash_states_and_remount; }
run() {
    local name="$1"; local log="$scratch/logs/$name.log"
    case "$name" in
        count-A) R3_COUNT_ONLY=1 R3_PREFIX=A R3_OPS=ow,ni1,sw2,raise1,raise3,rb1,rb3,remount t bf_r3_gap gap_frequency_over_crash_states_and_remount > "$log" 2>&1 ;;
        count-A-384) BF_WIDTH=384 R3_COUNT_ONLY=1 R3_PREFIX=A R3_OPS=rb1,raise1,remount,ow,ni1 t bf_r3_gap gap_frequency_over_crash_states_and_remount > "$log" 2>&1 ;;
        torn-sw2-384) BF_WIDTH=384 BF_NO_CS1=1 t bf_r3_bad torn_records_then_remount > "$log" 2>&1 ;;
        bad-gap2) BF_NO_CS1=1 R3B_LEFT=5 t bf_r3_bad bad_unit_right_of_a_gap > "$log" 2>&1 ;;
        bad-gap-extra2) BF_NO_CS1=1 R3B_LEFT=5 R3B_EXTRA_OW=2 t bf_r3_bad bad_unit_right_of_a_gap > "$log" 2>&1 ;;
        r3s-384) R3S=1 BF_WIDTH=384 BF_NO_CS1=1 t bf_r3_gap crafted_crash_histories_sweep > "$log" 2>&1 ;;
        r3s-verbose) R3_VERBOSE=1 R3S=1 R3S_ONLY="A/mcrash0,remount;A/mcrash1,remount;A/torn01,remount;A/rbcrash1,remount;A/mcrash0,remount,mcrash0,remount" BF_WIDTH=384 BF_NO_CS1=1 t bf_r3_gap crafted_crash_histories_sweep > "$log" 2>&1 ;;
        r3d-len2) R3D=1 R3D_LEN=2 BF_NO_CS1=1 t bf_r3_gap device_and_region_faults > "$log" 2>&1 ;;
        r3d-len3-0|r3d-len3-1|r3d-len3-2) R3D=1 R3D_LEN=3 R3D_SHARD="${name##*-}/3" BF_NO_CS1=1 t bf_r3_gap device_and_region_faults > "$log" 2>&1 ;;
        enum-A-rb1) enum rb1 "ow;rb3" 0 4108 > "$log" 2>&1 ;;
        enum-A-remount-0) enum remount "ow;rb1" 0 4000 > "$log" 2>&1 ;;
        enum-A-remount-4000) enum remount "ow;rb1" 4000 7000 > "$log" 2>&1 ;;
        enum-A-remount-11000) enum remount "ow;rb1" 11000 7000 > "$log" 2>&1 ;;
        enum-A-remount-18000) enum remount "ow;rb1" 18000 7000 > "$log" 2>&1 ;;
        enum-A-raise1-0) enum raise1 "ow;rb1" 0 7000 > "$log" 2>&1 ;;
        enum-A-raise1-7000) enum raise1 "ow;rb1" 7000 7000 > "$log" 2>&1 ;;
        jwrap1-a1) R2_STALE=1 R2_JRING_MIB=1 R2_SW_UNITS=16 R2_A=1 R2_B=16,17,18,19,20,21,22,23,24 t bf_r2_sweep stale > "$log" 2>&1 ;;
        jwrap1-a2) R2_STALE=1 R2_JRING_MIB=1 R2_SW_UNITS=16 R2_A=2 R2_B=16,17,18,19,20,21,22,23,24 t bf_r2_sweep stale > "$log" 2>&1 ;;
        jwrap6-a2) R2_STALE=1 R2_JRING_MIB=6 R2_SW_UNITS=72 R2_A=2 R2_R=3 R2_B=22 t bf_r2_sweep stale > "$log" 2>&1 ;;
        crash-head) R2_JRING_MIB=1 R2_SW_UNITS=16 R2_STATE_LIMIT=700 R2_SUFFIXES=raiseTo8 t bf_r2_crash > "$log" 2>&1 ;;
        bad-r2-rerun) BJ_STEP=3 t bf_r2_bad > "$log" 2>&1 ;;
        sweep-A-nofault|sweep-B-nofault|sweep-C-nofault) R2_NO_FAULTS=1 R2_PREFIX="${name:6:1}" t bf_r2_sweep final_images > "$log" 2>&1 ;;
        sweep-raise-A-nofault) R2_NO_FAULTS=1 R2_ALPHA=raise R2_PREFIX=A t bf_r2_sweep final_images > "$log" 2>&1 ;;
        crash-full-raiseTo8) R2_JRING_MIB=1 R2_SW_UNITS=16 R2_SUFFIXES=raiseTo8 t bf_r2_crash > "$log" 2>&1 ;;
        *) echo "段名 $name 不认得" >&2; exit 2 ;;
    esac
    echo "== $name"
    grep -E '^(test \S+ \.\.\. )?(R3G_HISTORY|R3G_DONE|R3S_DONE|R3D_DONE|R2_DONE|R2C_DONE)|R3G .*(-red|gap-states |refused)|^(test \S+ \.\.\. )?R3[SD] :: .*(-red|gap-states |images|refused)|^(test \S+ \.\.\. )?R3B .*pool_red|^R2 (stale|none):(CS2:(tf|beta)(gap|gaprow)?-lenient |pool-checker)|^R2C name=.*(tf-red-state|layer0-red)|^BJ .*pool_checker|test result' "$log" | cut -c1-260
}
names=("$@")
if [ "${#names[@]}" -eq 0 ]; then
    names=(count-A count-A-384 torn-sw2-384 bad-gap2 bad-gap-extra2 r3s-384 r3s-verbose r3d-len2 r3d-len3-0 r3d-len3-1 r3d-len3-2 enum-A-rb1 enum-A-remount-0 enum-A-remount-4000 enum-A-remount-11000 enum-A-remount-18000 enum-A-raise1-0 enum-A-raise1-7000 jwrap1-a1 jwrap1-a2 jwrap6-a2 crash-head bad-r2-rerun sweep-A-nofault sweep-B-nofault sweep-C-nofault sweep-raise-A-nofault)
fi
for name in "${names[@]}"; do run "$name"; done
