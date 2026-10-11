#!/usr/bin/env bash
# 攻方腿（m3-batch10z-tu-code-r1，opus）模型复跑：bash rerun.sh [<工作目录>]
# 只跑 harness 档与独立小 crate：攻方测试目标 atk_tu_opus（冻结副本那份 harness 测试文件 + atk_tu_part*.rs，名字不带 layer0）、
# 量法副本（cost-patch.py）里的 atk_tu_cost_opus 与 singlefs-core 库单测 atk_overlay*、十一个攻方变异、K4 的扫源码小 crate。
# 不起 checker 档测试、不枚举崩溃状态。每件经 run-with-memory-cap.sh（每件 3G，并行时一批合计不超过 12G；编译 12G），线程经 capped.sh。
# 整份约 35 分钟（推的：各件这一轮实测相加，变异 11 × 约 60 秒、K1 各段 1–3 分钟、大盘代价约 2 分钟）。
set -euo pipefail
ROOT=/home/fy5090/code/singlefs
MODEL=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-tu-code-r1/tree
WORK=${1:-$HOME/.local/share/singlefs/work/m3-batch10z-tu-code-r1-opus-rerun}
CAP=$ROOT/scripts/hosts/run-with-memory-cap.sh
THREADS=$ROOT/scripts/hosts/capped.sh
OUT=${WORK:?}/runs
mkdir -p "$WORK" "$OUT"
run3() { bash "$CAP" 3G "$@"; }
# 一、攻方测试目标（未改的冻结副本）
rsync -a --delete --exclude target --exclude .git "$FROZEN"/ "$WORK"/repo/
bash "$MODEL"/assemble.sh "$WORK"/repo "$MODEL"
( cd "$WORK"/repo && bash "$CAP" 12G env CARGO_TARGET_DIR="$WORK"/target nice -n 19 bash "$THREADS" 15 cargo test --offline --release -p singlefs-harness --test atk_tu_opus --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table --no-run )
BIN=$(ls -t "$WORK"/target/release/deps/atk_tu_opus-* | grep -v '\.d$' | head -1)
RBIN=$(ls -t "$WORK"/target/release/deps/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table-* | grep -v '\.d$' | head -1)
t() { local env_args=$1 name=$2 log=$3; run3 env $env_args nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact "$name" > "$OUT/$log" 2>&1; }
bash "$CAP" 12G nice -n 19 bash "$THREADS" 12 "$RBIN" --include-ignored --test-threads 12 > "$OUT"/mut-baseline.log 2>&1
# K1-1：预演与照出路走的真结局逐格比（K1 九形、放开小动作、实现员三形、两槽 C、被抛弃根 A、窄几档单元区）
t "ATK_EXTRA=8" atk_k1_rehearsal_vs_real_sweep k1-main.log
i=0; for spec in "2 6,11" "2 16,21" "4 6,11,16" "8 6,11,16"; do set -- $spec; i=$((i+1)); t "ATK_PRES=$1 ATK_FILLS=$2 ATK_EXTRA=8 ATK_SMALL=inode,overwrite ATK_KS=1,2,3,4,6" atk_k1_rehearsal_vs_real_sweep k1small-$i.log; done
t "" atk_k1_implementer_forms_rehearsal_vs_real k1impl.log
t "ATK_PRE_WRITES=0,1,3" atk_k1_two_c_slots_rehearsal_vs_real k1two-a.log
t "ATK_PRE_WRITES=6,10" atk_k1_two_c_slots_rehearsal_vs_real k1two-b.log
t "" atk_k1_abandoned_root_and_c_rehearsal_vs_real k1ab.log
for a in 352 320 288 256; do t "ATK_AREA=$a ATK_EXTRA=8" atk_k1_rehearsal_vs_real_sweep k1-area$a.log; done
# K1-2：预演里读盘出错
t "ATK_STEP_N=1 ATK_CELLS=4:16:13,4:16:19" atk_k1_transient_read_error_while_reporting k1rd-1.log
t "ATK_STEP_N=1 ATK_CELLS=4:16:20,2:16:17,8:11:40" atk_k1_transient_read_error_while_reporting k1rd-2.log
i=0; for cells in "4:16:13,4:16:19" "4:16:20,2:16:17" "8:11:40"; do i=$((i+1)); t "ATK_CELLS=$cells ATK_STEP_N=1" atk_k1_location_turning_unreadable_while_reporting k1pin-$i.log; done
t "" atk_k1_persistent_read_faults_on_the_isolated_c_slots k1pf.log
t "ATK_CELLS=4:16:13 ATK_MAX_SMALL=40" atk_k1_fill_up_with_small_actions_then_report k1fill-probe.log
# K1-3 / K1-2 / K5：量法副本
rsync -a --delete --exclude target --exclude .git "$FROZEN"/ "$WORK"/repo-cost/
python3 "$MODEL"/cost-patch.py "$WORK"/repo-cost "$MODEL"
bash "$MODEL"/assemble-cost.sh "$WORK"/repo-cost "$MODEL"
( cd "$WORK"/repo-cost && bash "$CAP" 12G env CARGO_TARGET_DIR="$WORK"/target-cost nice -n 19 bash "$THREADS" 15 cargo test --offline --release -p singlefs-harness --test atk_tu_cost_opus --no-run && bash "$CAP" 12G env CARGO_TARGET_DIR="$WORK"/target-cost nice -n 19 bash "$THREADS" 15 cargo test --offline --release -p singlefs-core --lib --no-run > "$OUT"/build-core.log 2>&1 )
CBIN=$(ls -t "$WORK"/target-cost/release/deps/atk_tu_cost_opus-* | grep -v '\.d$' | head -1)
LBIN=$(ls -t "$WORK"/target-cost/release/deps/singlefs_core-* | grep -v '\.d$' | head -1)
bash "$CAP" 3G nice -n 19 bash "$THREADS" 3 "$LBIN" atk_overlay --nocapture > "$OUT"/overlay-unit.log 2>&1
bash "$CAP" 6G env ATK_AREAS=384 nice -n 19 bash "$THREADS" 3 "$CBIN" --ignored --nocapture --exact atk_k1_cost_by_geometry > "$OUT"/cost-384.log 2>&1
bash "$CAP" 6G env ATK_AREAS=384,4096,65536 nice -n 19 bash "$THREADS" 3 "$CBIN" --ignored --nocapture --exact atk_k1_rehearsal_cost_by_device_size > "$OUT"/costsize-probe.log 2>&1
bash "$CAP" 10G env ATK_AREAS=1048576,8388608,33554432 nice -n 19 bash "$THREADS" 3 "$CBIN" --ignored --nocapture --exact atk_k1_rehearsal_cost_by_device_size > "$OUT"/costsize-big.log 2>&1
bash "$CAP" 12G env ATK_AREAS=67108864,134217728 nice -n 19 bash "$THREADS" 3 "$CBIN" --ignored --nocapture --exact atk_k1_rehearsal_cost_by_device_size > "$OUT"/costsize-huge.log 2>&1
# K3：十一个攻方变异（每个从冻结副本另拷一份）
for mutant in $(python3 "$MODEL"/mutants.py x --list); do bash "$MODEL"/run-mutant.sh "$mutant" "$WORK"; done
# 那份测试文件照绿的 B、D、I 再在原型的全量几段上跑（看预演的列不列推环有没有翻）
for mutant in I-rehearsal-with-the-shadow-ledger-off B-overlay-reads-never-see-own-writes D-overlay-keeps-each-block-one-block-later; do
  bash "$MODEL"/run-mutant.sh "$mutant" "$WORK" > /dev/null
  ABIN=$(ls -t "$WORK"/target-mut/release/deps/atk_tu_opus-* | grep -v '\.d$' | head -1)
  run3 nice -n 19 bash "$THREADS" 3 "$ABIN" --ignored --nocapture --exact atk_k1_rehearsal_vs_real_sweep > "$OUT"/mut-$mutant-atk-k1full.log 2>&1 || true
  if [ "$mutant" != D-overlay-keeps-each-block-one-block-later ]; then
    run3 nice -n 19 bash "$THREADS" 3 "$ABIN" --ignored --nocapture --exact atk_k1_abandoned_root_and_c_rehearsal_vs_real > "$OUT"/mut-$mutant-atk-ab.log 2>&1 || true
    run3 nice -n 19 bash "$THREADS" 3 "$ABIN" --ignored --nocapture --exact atk_k1_two_c_slots_rehearsal_vs_real > "$OUT"/mut-$mutant-atk-two.log 2>&1 || true
  fi
done
# K4：扫源码的小 crate
bash "$MODEL"/build-scan-proto.sh "$FROZEN" "$WORK"/scan-proto > "$OUT"/build-scan.log 2>&1
python3 "$MODEL"/gen-scan-inputs.py "$FROZEN" "$WORK"/scan-proto/inputs > "$OUT"/k4-inputs.log
S=$WORK/scan-proto/inputs
bash "$CAP" 2G nice -n 19 "$WORK"/scan-proto/target/release/atk-stop-scan "$S"/v0-frozen.rs "$S"/v1-new-layer-plain.rs "$S"/v2-new-layer-behind-a-commented-opening.rs "$S"/v3-multiline-new-layer-behind-a-commented-opening.rs > "$OUT"/k4-stop-scan.log 2>&1
bash "$CAP" 2G nice -n 19 "$WORK"/scan-proto/target/release/atk-stop-scan "$S"/v4-trailing-comment-with-fn-in-a-signature.rs "$S"/v5-attribute-line-before-a-layer.rs > "$OUT"/k4-stop-scan-fp.log 2>&1
bash "$CAP" 2G nice -n 19 "$WORK"/scan-proto/target/release/atk-gpu-scan "$S"/g0-frozen.rs "$S"/g1-use-wgpu-itself.rs "$S"/g2-block-scoped-rename-in-a-helper.rs "$S"/g3-commented-tests-opening-inside-the-pool.rs > "$OUT"/k4-gpu-scan.log 2>&1
echo "复跑完：$OUT"
