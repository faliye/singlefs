#!/usr/bin/env bash
# m3-inv-b-r1 攻方腿的原型复跑：把冻结副本（或同一提交 fcc4d340 的仓）拷到草稿目录，打上 proto.diff，逐条跑 harness 档的原型用例。
# 用法：bash rerun.sh <冻结副本根（含 Cargo.toml 与 crates/）> <草稿目录> [线程数，默认 6]
# 每条经内存包装跑；上限给 8G（首次跑峰值表里没有这几条，理由写在 RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON）。不跑 checker 档包的任何测试。
set -euo pipefail
frozen="$1"; scratch="$2"; threads="${3:-6}"
here="$(cd "$(dirname "$0")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
mkdir -p "$scratch"
rsync -a --exclude target --exclude .git "$frozen"/ "$scratch/tree/"
( cd "$scratch/tree" && patch -p1 < "$here/proto.diff" )
cap() { RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="${RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON:-攻方原型复跑，峰值表里没有这几条}" \
        RUN_WITH_MEMORY_CAP_WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-1800}" \
        nice -n 19 bash "$repo/research/scripts/run-with-memory-cap.sh" 8G bash "$repo/research/scripts/capped.sh" "$threads" "$@"; }
cd "$scratch/tree"
cap cargo build -p singlefs-harness --tests
# B7：组合坏镜像与对照格（三种守卫各判一次）。
cap cargo test -p singlefs-harness --test checker_known_bad_images -- --nocapture --test-threads 1 proto_b7 > "$scratch/b7.log" 2>&1
# B3：两段写死的回退历史，再把用户决定的那几步放开扫（84 格）。
cap cargo test -p singlefs-harness --test inv_b_proto_b3_rollback -- --nocapture > "$scratch/b3.log" 2>&1
# 多条记录的发布：C 取 3 条记录（记录段 6 个写全枚举）与 12 条记录（记录段结构化取样）。
PROTO_C_UNITS=3 PROTO_SUBSET_LIMIT=6 cap cargo test -p singlefs-harness --test inv_b_proto_multi_record_crash -- --nocapture > "$scratch/mr3.log" 2>&1
PROTO_DUMP="seg=1" PROTO_C_UNITS=12 PROTO_SUBSET_LIMIT=6 cap cargo test -p singlefs-harness --test inv_b_proto_multi_record_crash -- --nocapture > "$scratch/mr12.log" 2>&1
# 随机历史两批，各 48 段 × 30 步。
PROTO_FIRST_SEED=1000000 PROTO_SEEDS=48 PROTO_WEIGHTS=rollback cap cargo test -p singlefs-harness --test inv_b_proto_histories -- --nocapture > "$scratch/rollback48.log" 2>&1
PROTO_FIRST_SEED=2000000 PROTO_SEEDS=48 PROTO_WEIGHTS=broad cap cargo test -p singlefs-harness --test inv_b_proto_histories -- --nocapture > "$scratch/broad48.log" 2>&1
grep -hE 'PROTO|test result' "$scratch"/b7.log "$scratch"/b3.log "$scratch"/mr3.log "$scratch"/mr12.log "$scratch"/rollback48.log "$scratch"/broad48.log
