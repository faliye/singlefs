#!/usr/bin/env bash
# 攻方腿（m3-batch10z-pqrs-code-r1，opus）模型复跑：bash rerun.sh [<工作目录>]
# 从冻结副本拷一份仓、把 harness 测试文件与原型拼成测试目标 atk_pqrs_opus（名字不带 layer0），release 编，逐件跑，输出落 <工作目录>/runs/。
# 每件都直接经 run-with-memory-cap.sh 起（3G 一件、一次一件；编译 12G）；线程数经 capped.sh。只跑 harness 档，不起 checker 档、不枚举崩溃状态。
set -euo pipefail
ROOT=/home/fy5090/code/singlefs
MODEL=$ROOT/research/prompts/m3-batch10z-pqrs-code-r1-opus-model
FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1/tree
WORK=${1:-$HOME/.local/share/singlefs/work/m3-batch10z-pqrs-code-r1-opus-rerun}
CAP=$ROOT/scripts/hosts/run-with-memory-cap.sh
THREADS=$ROOT/scripts/hosts/capped.sh
OUT=${WORK:?}/runs
mkdir -p "$WORK" "$OUT"
rsync -a --delete --exclude target --exclude .git "$FROZEN"/ "${WORK:?}"/repo/
bash "$MODEL"/assemble.sh "$WORK"/repo "$MODEL"
cd "$WORK"/repo
bash "$CAP" 12G env CARGO_TARGET_DIR="$WORK"/target nice -n 19 bash "$THREADS" 15 cargo test --offline --release -p singlefs-harness --test atk_pqrs_opus --no-run
BIN=$(ls -t "$WORK"/target/release/deps/atk_pqrs_opus-* | grep -v '\.d$' | head -1)
# K1-1 / K1-2：盘边上崩在三条记录的顺序写半路，C 那一圈只有一槽；长度 g 放开扫（约 35 秒）
bash "$CAP" 3G env ATK_PUSH_ATTEMPTS=3 ATK_PRES=2,4,8 ATK_FILLS=6,11,16,21 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k1_ways_out_sweep > "$OUT"/k1w-run3.log 2>&1
bash "$CAP" 3G env ATK_STEP=5 ATK_FILLS=16 ATK_C_ARMS=1 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k1_fill_sweep > "$OUT"/k1-run3.log 2>&1
bash "$CAP" 3G env ATK_PRE=4 ATK_FILL=16 ATK_GROW=20 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k1_push_trace > "$OUT"/k1-trace.log 2>&1
bash "$CAP" 3G env ATK_SMALL=inode ATK_MAX_SMALL=24 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k1_small_writes_after_push_sweep > "$OUT"/k1-small-inode.log 2>&1
bash "$CAP" 3G env ATK_SMALL=overwrite nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k1_small_writes_after_push_sweep > "$OUT"/k1-small.log 2>&1
bash "$CAP" 3G nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k1_implementer_forms_sweep > "$OUT"/k1-impl.log 2>&1
# K2-1：推环真发半路块设备错（逐个改盘调用）、推环写流逐个前缀崩溃（各约 70–150 秒）
bash "$CAP" 3G env ATK_FORM=no40 ATK_STEP_J=1 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k2_block_device_error_sweep > "$OUT"/k2err-no40.log 2>&1
bash "$CAP" 3G env ATK_FORM=b6o20 ATK_STEP_J=3 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k2_block_device_error_sweep > "$OUT"/k2err-b6o20.log 2>&1
bash "$CAP" 3G env ATK_FORM=no40 ATK_STEP_K=1 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k2_crash_prefix_sweep > "$OUT"/k2crash-no40.log 2>&1
bash "$CAP" 3G env ATK_FORM=b6o20 ATK_STEP_K=2 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k2_crash_prefix_sweep > "$OUT"/k2crash-b6o20.log 2>&1
# K2-2：C 两槽（Q、R）与环里另有被抛弃根
bash "$CAP" 3G env ATK_PRE_WRITES=0,1 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k2_two_c_slots_push > "$OUT"/k2two-a.log 2>&1
bash "$CAP" 3G env ATK_PRE_WRITES=3,6 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k2_two_c_slots_push > "$OUT"/k2two-b.log 2>&1
bash "$CAP" 3G env ATK_PRE_WRITES=10 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k2_two_c_slots_push > "$OUT"/k2two-c.log 2>&1
bash "$CAP" 3G nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k2_abandoned_root_and_c_push > "$OUT"/k2ab.log 2>&1
# K3-1：被抛弃根那一圈的随机动作串（四段种子）
for part in 1 2 3 4; do
  bash "$CAP" 3G env ATK_SEEDS=20 ATK_SEED_BASE=$(( (part - 1) * 20 )) ATK_LEN=16 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact atk_k3_sweep > "$OUT"/k3-$part.log 2>&1
done
# K4：攻方自己的四个变异（mutants.py 原文恰好命中一次才改），对冻结副本那份 harness 测试文件跑；M12 另对原型的两槽用例跑
for mutant in M1-current-not-moved-on-failure M2-devices-not-checked M7-count-all-isolation M12-r-slot-is-the-first; do
  rsync -a --delete --exclude target --exclude .git "$FROZEN"/ "${WORK:?}"/repo-mut/
  python3 "$MODEL"/mutants.py "$WORK"/repo-mut "$mutant"
  bash "$MODEL"/assemble.sh "$WORK"/repo-mut "$MODEL"
  cd "$WORK"/repo-mut
  bash "$CAP" 12G env CARGO_TARGET_DIR="$WORK"/target-mut nice -n 19 bash "$THREADS" 15 cargo test --offline --release -p singlefs-harness --test raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table --test atk_pqrs_opus --no-run
  MBIN=$(ls -t "$WORK"/target-mut/release/deps/raise_floor_takes_abandoned_root_placements_recorded_in_the_root_ring_table-* | grep -v '\.d$' | head -1)
  bash "$CAP" 12G nice -n 19 bash "$THREADS" 15 "$MBIN" --test-threads 12 > "$OUT"/mut-$mutant-run.log 2>&1 || true
  if [ "$mutant" = M12-r-slot-is-the-first ]; then
    ABIN=$(ls -t "$WORK"/target-mut/release/deps/atk_pqrs_opus-* | grep -v '\.d$' | head -1)
    bash "$CAP" 3G env ATK_PRE_WRITES=0 nice -n 19 bash "$THREADS" 3 "$ABIN" --ignored --nocapture --exact atk_k2_two_c_slots_push > "$OUT"/mut-M12-atk-run.log 2>&1 || true
  fi
  cd "$WORK"/repo
done
echo "复跑完：$OUT"
