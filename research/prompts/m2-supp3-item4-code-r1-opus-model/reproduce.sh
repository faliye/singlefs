#!/usr/bin/env bash
# 攻方腿（Opus）K1 / K2 的复跑脚本。全部在副本上跑，不动工作区。
# 副本上量出来的数不进 kb：主 agent 要在入库装置上重做一次才能引
# （.claude/rules/three-way-inference.md「判决由主 agent 做，不由投票做」）。
set -euo pipefail
REPO=${1:?用法: reproduce.sh <singlefs 工作区路径> <副本目录>}
COPY=${2:?用法: reproduce.sh <singlefs 工作区路径> <副本目录>}
HERE=$(cd "$(dirname "$0")" && pwd)

rsync -a --exclude target --exclude .git "$REPO/" "$COPY/"
cd "$COPY"

# 0 基线：工作区原样的快档，应当全绿（新发现 0）。
nice -n 19 cargo test --release -p singlefs-harness \
  --test second_transaction_supplement_three_fault_injection -- --nocapture \
  > "$COPY/baseline-fast.txt" 2>&1

# 1 打上三处探针（两处在 history.rs、一处在 fault_injection.rs），都由环境变量开，默认不改行为。
patch -p0 < "$HERE/probes.patch"
cp "$HERE/opus_probe_one_device_system_configuration.rs" \
   crates/singlefs-harness/tests/opus_probe_one_device_system_configuration.rs

# 2 对照：不注入故障时探针开关是死的（随机历史快档照样全绿）。
SINGLEFS_FAULT_PROBE_CONTINUE_PAST_DEVICE_ERRORS=1 nice -n 19 cargo test --release \
  -p singlefs-harness --test second_transaction_supplement_three_random_history -- --nocapture \
  > "$COPY/control-random-history-with-probe.txt" 2>&1

# 3 K2：注入的块设备错之后照常往下跑（模型对拍照旧）。
SINGLEFS_FAULT_PROBE_CONTINUE_PAST_DEVICE_ERRORS=1 nice -n 19 cargo test --release \
  -p singlefs-harness --test second_transaction_supplement_three_fault_injection \
  fault_injection_fast_tier -- --nocapture \
  > "$COPY/probe-k2-continue.txt" 2>&1 || true

# 4 K2：往下跑，并且从注入那一步起把模型对拍整个挂起，只留 checker 与 panic 两条不依赖模型的判据。
SINGLEFS_FAULT_PROBE_CONTINUE_PAST_DEVICE_ERRORS=1 \
SINGLEFS_FAULT_PROBE_SUSPEND_MODEL_AFTER_DEVICE_ERRORS=1 \
SINGLEFS_FAULT_INJECTION_WORKER_THREADS=1 nice -n 19 cargo test --release \
  -p singlefs-harness --test second_transaction_supplement_three_fault_injection \
  fault_injection_fast_tier -- --nocapture \
  > "$COPY/probe-k2-suspend-model.txt" 2>&1 || true

# 5 K2：三个种子各单独跑一遍（大档那条用例，规模压成 1 段）。
for s in 7463871032432355133 7463871032432355116 7463871032432355122; do
  echo "##### seed $s"
  SINGLEFS_FAULT_PROBE_CONTINUE_PAST_DEVICE_ERRORS=1 \
  SINGLEFS_FAULT_INJECTION_FIRST_SEED=$s SINGLEFS_FAULT_INJECTION_SEEDS=1 \
  SINGLEFS_FAULT_INJECTION_OPERATIONS=20 SINGLEFS_FAULT_INJECTION_FAULTS=4 \
  SINGLEFS_FAULT_INJECTION_WORKER_THREADS=1 nice -n 19 cargo test --release \
    -p singlefs-harness --test second_transaction_supplement_three_fault_injection \
    fault_injection_large_tier -- --ignored --nocapture 2>&1 \
    | grep -v FAULT_INJECTION_PROGRESS | sed -n '/故障注入大档/,/用时/p'
done > "$COPY/probe-k2-single-seeds.txt" 2>&1

# 6 K1：把「第 n 次起每一次」「只一块盘」这两种今天一次都没抽过的排期换进去。
for mode in persistent persistent-one-device; do
  echo "##### schedule=$mode"
  SINGLEFS_FAULT_PROBE_SCHEDULE=$mode SINGLEFS_FAULT_INJECTION_WORKER_THREADS=1 \
  nice -n 19 cargo test --release -p singlefs-harness \
    --test second_transaction_supplement_three_fault_injection \
    fault_injection_fast_tier -- --nocapture 2>&1 \
    | grep -v FAULT_INJECTION_PROGRESS | sed -n '/故障注入快档/,/用时/p'
done > "$COPY/probe-k1-schedule.txt" 2>&1 || true
# probe-k1-dead-device.txt 是第 6 步里 persistent-one-device 那一段单独跑的一份（起点段之后才让盘 1 死）。

# 7 K1 的可跑构造：同一块盘接连两次读失败 ⇒ 整个池挂不上；只坏一次 ⇒ 照常恢复。
nice -n 19 cargo test --release -p singlefs-harness \
  --test opus_probe_one_device_system_configuration -- --nocapture
