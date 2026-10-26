#!/usr/bin/env bash
# 辩方腿（m4-i315-r3）：跑一片随机历史探针，写 <日志目录>/run-<标签>.log；已有同名日志且末行是完成标记的跳过（续跑）。
# 用法：bash run_slice.sh <仓根> <副本根> <日志目录> <标签> <步数> <段数> <种子偏移> [缺陷] [线程] [权重名]
# 缺陷：none / one_device / extra_leak / leak_every / leak_once。线程取派发提示的线程上限（调用方传 31）。
# 权重名：broad（默认）/ reuse / rollback / compaction / compaction_rollback / broad_compaction / snapshot / writes_by_inode / truncation。
# 内存包装：默认这里自己套 6G（RUN_SLICE_CAP 改；放大档比重 compaction / snapshot 的峰值到过 11.9 GiB，用 16G）；整条已经经 run-with-memory-cap.sh 跑的调用方（rerun.sh）设 RUN_SLICE_NO_CAP=1，这里不再套。
set -euo pipefail
repo="$1"; tree="$2"; logs="$3"; label="$4"; steps="$5"; seeds="$6"; offset="$7"; defect="${8:-none}"; threads="${9:-31}"; weights="${10:-broad}"
log="$logs/run-$label.log"
if [[ -f "$log" ]] && tail -n 1 "$log" | grep -qF '=== slice done'; then echo "skip $label"; exit 0; fi
cap=(bash "$repo/scripts/hosts/run-with-memory-cap.sh" "${RUN_SLICE_CAP:-6G}")
if [[ -n "${RUN_SLICE_NO_CAP:-}" ]]; then cap=(); fi
cd "$tree"
SINGLEFS_RUN_SCOPE='种类=target; 跑什么=cargo test -p singlefs-harness 的测试目标 --test r3_probe; 范围=1 个测试目标; 为什么=辩方腿副本里一片随机历史在九个臂与三种归类器下的归类，只跑自己的探针目标' \
  ATTACK_DEFECT="$defect" DEFENSE_WEIGHTS="$weights" DEFENSE_DUMP=1 DEFENSE_STEPS="$steps" DEFENSE_SEEDS="$seeds" DEFENSE_SEED_OFFSET="$offset" \
  nice -n 19 bash "$repo/scripts/hosts/capped.sh" "$threads" "${cap[@]}" \
  cargo test --release -p singlefs-harness --test r3_probe -- --nocapture > "$log.tmp" 2>&1
echo "=== slice done $label" >> "$log.tmp"
mv "$log.tmp" "$log"
echo "done $label"
