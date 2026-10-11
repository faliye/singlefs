#!/usr/bin/env bash
# m3-birth-floor-r1 攻方腿的原型复跑：把冻结副本拷到草稿目录，打上 proto.diff，经内存包装（8G）、线程上限 4 跑原型包 bf-proto 的两份用例。
# 用法：bash rerun.sh <冻结副本根（含 Cargo.toml 与 crates/）> <草稿目录> [crash]
#   不带第三个参数：只跑坏镜像那一份（几秒）；带 crash：再跑四段历史的层 0 全量崩溃状态（458 857 个状态，本机约一小时）。
# 不跑 checker 档包的任何测试：bf-proto 只把 singlefs-checker-tier 当库链进来，调它的 judge_layer0_state_range。
set -euo pipefail
frozen="$1"; scratch="$2"; mode="${3:-}"
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
mkdir -p "$scratch"
rsync -a --exclude target --exclude .git "$frozen"/ "$scratch/tree/"
( cd "$scratch/tree" && patch -p2 -s < "$here/proto.diff" )
cap() {
    RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-攻方原型复跑，峰值表里没有这几条}" \
    RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-3600}" \
        nice -n 19 bash "$repo/scripts/hosts/run-with-memory-cap.sh" 8G bash "$repo/scripts/hosts/capped.sh" 4 "$@"
}
cd "$scratch/tree"
nice -n 19 bash "$repo/scripts/hosts/capped.sh" 4 cargo build --offline --release -p bf-proto --tests
BI2_SLOTS=50186,50277,50278,50280 cap cargo test --offline --release -p bf-proto --test bf_bad_images -- --nocapture > "$scratch/bad.log" 2>&1
grep -E '^BI' "$scratch/bad.log"
if [ "$mode" = crash ]; then
    for name in 'ni300-ow-ni1-ow/rb3/[raiseTo5]' 'sw3-ow-sw2-ow/rb3/[raiseTo5]' 'sw3-ni240-ow-ni5/rb3/[raiseTo5]' 'ni300-ow-ni1-ow/rbcrash3-remount-raise5/[rb1]'; do
        log="$scratch/run1-$(echo "$name" | tr '/[],' '____').log"
        BF_SET=run1 BF_ONLY="$name" cap cargo test --offline --release -p bf-proto --test bf_crash_states -- --nocapture > "$log" 2>&1
        grep -E '^BF_(HISTORY|DONE)|^BF name' "$log"
    done
fi
