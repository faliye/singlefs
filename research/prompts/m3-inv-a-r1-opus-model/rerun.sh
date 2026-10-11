#!/usr/bin/env bash
# m3-inv-a-r1 攻方腿的模型复跑：把冻结副本（或同一提交的仓）拷到草稿目录，打上这里的三份 diff、放进两份原型用例，编、跑。
# 用法：bash rerun.sh <草稿目录> [<源树，默认仓根>]        # 源树要是 HEAD fcc4d340 的 crates/ 与 Cargo.toml、Cargo.lock
# 经内存包装与线程上限（默认 6，环境变量 INV_A_THREADS 改）；checker 档包自己的测试一个都不跑，只跑 singlefs-harness 的一个目标与原型包 inv-a-proto 的一个目标。
# 输出：<草稿目录>/logs/ 下 harness.log（约 4 分钟）、e3.log、e4.log（观察者每个状态都做，各约 30 分钟）、e2.log（观察者取样 1/160，约 16 分钟）、e5.log。
set -euo pipefail
here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository="$(cd "$here/../../.." && pwd)"
scratch="$1"
source_tree="${2:-$repository}"
threads="${INV_A_THREADS:-6}"
mkdir -p "$scratch/logs"
rsync -a --exclude target --exclude .git "$source_tree/crates" "$source_tree/Cargo.toml" "$source_tree/Cargo.lock" "$scratch/tree/"
cd "$scratch/tree"
patch -p1 < "$here/checker-walk.diff"
patch -p1 < "$here/core-recovery.diff"
patch -p1 < "$here/workspace-cargo.diff"
cp "$here/inv_a_watermark_replay_and_system_configuration.rs" crates/singlefs-harness/tests/
mkdir -p crates/inv-a-proto
cp -r "$here/inv-a-proto/." crates/inv-a-proto/
export CARGO_TARGET_DIR="$scratch/target"
export RUN_WITH_MEMORY_CAP_PARALLEL_REASON="${RUN_WITH_MEMORY_CAP_PARALLEL_REASON:-复跑 m3-inv-a-r1 攻方腿的模型}"
wrap=(nice -n 19 bash "$repository/scripts/hosts/run-with-memory-cap.sh" 8G bash "$repository/scripts/hosts/capped.sh" "$threads")
"${wrap[@]}" cargo test -p singlefs-harness --test inv_a_watermark_replay_and_system_configuration -- --nocapture --test-threads=1 > "$scratch/logs/harness.log" 2>&1
INV_A_CORE_WATERMARK_ROOTS_ONLY=1 "${wrap[@]}" cargo test -p singlefs-harness --test inv_a_watermark_replay_and_system_configuration -- --nocapture --test-threads=1 a5_unapplied > "$scratch/logs/harness-core-roots-only.log" 2>&1
proto=(cargo test --release -p inv-a-proto --test inv_a_crash_states -- --nocapture --test-threads=1)
INV_A_RANDOM_STATES=400 INV_A_RANDOM_SEED=1 "${wrap[@]}" "${proto[@]}" e5_ > "$scratch/logs/e5.log" 2>&1
"${wrap[@]}" "${proto[@]}" e3_ > "$scratch/logs/e3.log" 2>&1
"${wrap[@]}" "${proto[@]}" e4_ > "$scratch/logs/e4.log" 2>&1
INV_A_OBSERVER_ONE_IN=160 "${wrap[@]}" "${proto[@]}" e2_ > "$scratch/logs/e2.log" 2>&1
grep -hE '^INV-A|INV-A-E|test result' "$scratch"/logs/*.log | cut -c1-300
