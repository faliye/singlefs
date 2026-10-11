#!/usr/bin/env bash
# 攻方腿（m3-batch10z-xy-code-r1，opus）模型复跑：bash rerun.sh [<工作目录>]。整份跑完不中途退出（不用 set -e；每步失败照记、接着下一步）。
# 只跑 harness 档与独立小 crate：攻方测试目标 atk_xy_opus（冻结副本那份 harness 测试文件 + atk_xy_*.rs，名字不带 layer0）、
# 攻方变异与变异表第 3603–3623 行（core、harness）、第 3624–3640 行（施加在抄出来的扫源码小程序上）、K1-2 的另一条臂。
# 不起 checker 档测试、门禁阶段、崩溃枚举。每件经 run-with-memory-cap.sh（并行件 3G×4；编译 12G；扫源码 4G / 2G），线程经 capped.sh（合计不超过 7）。
# 挂钟：这一轮实测相加约 40 分钟（推的；机器上另有 cargo 在跑时更长）。
set -uo pipefail
ROOT=/home/fy5090/code/singlefs
MODEL=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-xy-code-r1/tree
WORK=${1:-$HOME/.local/share/singlefs/work/m3-batch10z-xy-code-r1-opus-rerun}
CAP=$ROOT/scripts/run-with-memory-cap.sh
THREADS=$ROOT/scripts/capped.sh
OUT=$WORK/runs; mkdir -p "$OUT"
step() { echo "== $(TZ=Asia/Tokyo date +%F) $*" | tee -a "$OUT/rerun-steps.log"; }
batches() { # batches <批前缀> <件表>：每 4 件一批
  local prefix=$1 table=$2 i=0
  rm -f "$WORK/$prefix"-part-*
  split -l 4 -d -a 2 "$table" "$WORK/$prefix-part-"
  for part in "$WORK/$prefix"-part-*; do i=$((i+1)); bash "$MODEL"/batch.sh "$WORK" "$prefix$i" "$part" | tr '\n' ' ' | tee -a "$OUT/rerun-steps.log"; echo | tee -a "$OUT/rerun-steps.log"; done
}
step "一、攻方测试目标"
rsync -a --delete --exclude target --exclude .git "$FROZEN"/ "$WORK"/repo/
bash "$MODEL"/assemble.sh "$WORK"/repo "$MODEL"
( cd "$WORK"/repo && bash "$CAP" 12G env CARGO_TARGET_DIR="$WORK"/target nice -n 19 bash "$THREADS" 7 cargo test --offline --release -p singlefs-harness --test atk_xy_opus --no-run ) > "$OUT"/build.log 2>&1
export ATK_BIN=$(ls -t "$WORK"/target/release/deps/atk_xy_opus-* | grep -v '\.d$' | head -1)
step "二、K1-1：K1 九形（12 格）× 三组动作；实现员三形、两槽 C、被抛弃根 A 与 C、窄四档"
G1=none,same,bigger,inode,overwrite,push-swapped,push-reversed
G2=unmount-remount,frozen-overwrite,push-fail-1,push-fail-150,ask-then-same,table-reversed,table-swapped
G3=table-one,table-stale,inode-then-same-1,inode-then-same-3,overwrite-then-same-1,overwrite-then-same-3,overwrite-then-same-6
: > "$WORK"/jobs-k1.txt
for pre in 2 4 8; do for fill in 6 11 16 21; do i=1; for G in $G1 $G2 $G3; do echo "k1-pre$pre-fill$fill-g$i.log atk_k1_ways_out_vs_real_sweep ATK_PRES=$pre ATK_FILLS=$fill ATK_ACTIONS=$G" >> "$WORK"/jobs-k1.txt; i=$((i+1)); done; done; done
batches k1 "$WORK"/jobs-k1.txt
BA=none,same,table-reversed,table-swapped,table-one,table-stale
BB=frozen-overwrite,inode-then-same-1,overwrite-then-same-1,ask-then-same,push-fail-1
{ echo "fam-impl.log atk_k1_implementer_forms_ways_out_vs_real ATK_ACTIONS=$BA,$BB"
  for pw in 0 1 3 6 10; do echo "fam-two-pw$pw-A.log atk_k1_two_c_slots_ways_out_vs_real ATK_PRE_WRITES=$pw ATK_ACTIONS=$BA"; echo "fam-two-pw$pw-B.log atk_k1_two_c_slots_ways_out_vs_real ATK_PRE_WRITES=$pw ATK_ACTIONS=$BB"; done
  for ks in 0 1 2 4 6; do echo "fam-ab-k$ks-A.log atk_k1_abandoned_root_and_c_ways_out_vs_real ATK_KS=$ks ATK_ACTIONS=$BA"; echo "fam-ab-k$ks-B.log atk_k1_abandoned_root_and_c_ways_out_vs_real ATK_KS=$ks ATK_ACTIONS=$BB"; done
  for a in 352 320 288 256; do for p in 2 4 8; do echo "area$a-$p-A.log atk_k1_ways_out_vs_real_sweep ATK_AREA=$a ATK_PRES=$p ATK_ACTIONS=$BA"; echo "area$a-$p-B.log atk_k1_ways_out_vs_real_sweep ATK_AREA=$a ATK_PRES=$p ATK_ACTIONS=$BB"; done; done
} > "$WORK"/jobs-fam.txt
batches fam "$WORK"/jobs-fam.txt
step "三、K1-1 新形（H 推完放得下）、K1-2 推环入口照收次序对调的盘表、两道判的先后、K1-3 别的会话同一对数"
printf '%s\n' "k1h.log atk_k1_h_walk_after_the_intent_admitted_after_the_push" \
  "k1h-natural-c.log atk_k1_h_walk_after_the_intent_admitted_after_the_push ATK_KS=0 ATK_ISOS=0 ATK_LEFTS=$(seq -s, 0 2 80)" \
  "k13.log atk_k13_ways_out_on_another_session_with_the_same_pair" \
  "k2order-baseline.log atk_k2_moved_session_and_reversed_table_which_refusal" \
  "k12-pre2.log atk_k12_push_with_the_device_table_in_reversed_order ATK_PRES=2" \
  "k12-pre4.log atk_k12_push_with_the_device_table_in_reversed_order ATK_PRES=4" \
  "k12-pre8.log atk_k12_push_with_the_device_table_in_reversed_order ATK_PRES=8" > "$WORK"/jobs-new.txt
batches new "$WORK"/jobs-new.txt
step "四、K2：攻方变异（core 与 harness），每个从冻结副本另拷一份"
for m in baseline $(python3 "$MODEL"/mutants.py x --list); do
  bash "$MODEL"/run-mutant.sh build "$m" "$WORK" | tee -a "$OUT/rerun-steps.log"
  bash "$MODEL"/run-mutant.sh run "$m" "$WORK" | tee -a "$OUT/rerun-steps.log"
  if [ "$m" = N1-table-check-before-the-version-check ]; then
    BIN=$(ls -t "$WORK"/target-mut/release/deps/atk_xy_opus-* | grep -v '\.d$' | head -1)
    bash "$CAP" 3G nice -n 19 bash "$THREADS" 1 "$BIN" --ignored --nocapture --exact atk_k2_moved_session_and_reversed_table_which_refusal > "$OUT"/k2order-N1.log 2>&1
  fi
done
step "五、K2：变异表第二十四块 21 行（第 3603–3623 行）"
for r in $(seq 3603 3623); do
  bash "$MODEL"/run-mutant.sh build "row:$r" "$WORK" | tee -a "$OUT/rerun-steps.log"
  bash "$MODEL"/run-mutant.sh run "row:$r" "$WORK" | tee -a "$OUT/rerun-steps.log"
done
step "六、K1-2 另一条臂：推环入口也先比次序"
bash "$MODEL"/run-mutant.sh build ARM-push-checks-the-table-order "$WORK" | tee -a "$OUT/rerun-steps.log"
printf '%s\n' "k12-arm-pre2.log atk_k12_push_with_the_device_table_in_reversed_order ATK_PRES=2" \
  "k12-arm-pre4.log atk_k12_push_with_the_device_table_in_reversed_order ATK_PRES=4" \
  "k12-arm-pre8.log atk_k12_push_with_the_device_table_in_reversed_order ATK_PRES=8" > "$WORK"/jobs-arm.txt
ATK_BIN=$(ls -t "$WORK"/target-mut/release/deps/atk_xy_opus-* | grep -v '\.d$' | head -1) bash "$MODEL"/batch.sh "$WORK" arm "$WORK"/jobs-arm.txt | tee -a "$OUT/rerun-steps.log"
step "七、K3：扫源码小程序（机械抽两份 mod tests 里读自己源文件的用例），喂造的写法"
SRC_DIR=$FROZEN/crates/singlefs-checker-tier/src
mkdir -p "$WORK"/scan/src/bin
printf '[package]\nname = "atk-scan"\nversion = "0.1.0"\nedition = "2021"\n\n[workspace]\n' > "$WORK"/scan/Cargo.toml; : > "$WORK"/scan/src/lib.rs
python3 "$MODEL"/build-scan-proto.py "$SRC_DIR"/gpu_unit_checks.rs "$WORK"/scan/src/bin/gpu.rs probe=code_before_the_tests_module skip=a_poll_before_booking_takes_a_device_as_idle_only_when_its_queue_is_empty > "$OUT"/scan-build.log
python3 "$MODEL"/build-scan-proto.py "$SRC_DIR"/crash_amplification.rs "$WORK"/scan/src/bin/crash.rs probe=text_outside_the_tests_module skip=every_environment_variable_this_file_reads_by_name_is_in_the_list_a_run_reads skip=the_stop_request_remembered_in_this_run_reaches_the_rounds_the_next_card_the_card_threads_and_the_run skip=the_stop_request_is_asked_once_before_each_block_and_a_stop_leaves_the_later_blocks_untaken >> "$OUT"/scan-build.log
( cd "$WORK"/scan && bash "$CAP" 4G nice -n 19 bash "$THREADS" 4 cargo build --offline --release ) >> "$OUT"/scan-build.log 2>&1
rm -rf "$WORK"/scan-inputs; python3 "$MODEL"/gen-scan-inputs.py "$FROZEN" "$WORK"/scan-inputs > "$OUT"/k3-inputs.log
SRC=$SRC_DIR/gpu_unit_checks.rs bash "$CAP" 2G nice -n 19 "$WORK"/scan/target/release/gpu > "$OUT"/k3-gpu-frozen.log 2>&1
SRC=$SRC_DIR/crash_amplification.rs bash "$CAP" 2G nice -n 19 "$WORK"/scan/target/release/crash > "$OUT"/k3-crash-frozen.log 2>&1
for f in "$WORK"/scan-inputs/c*.rs; do echo "== $(basename "$f" .rs)"; SRC=$f bash "$CAP" 2G nice -n 19 "$WORK"/scan/target/release/crash every_layer_handed_the_stop_request_hands_it_on_or_asks_it 2>&1 | cut -c1-230; done > "$OUT"/k3-crash.log
T="the_entries_outside_the_card_pool_start_their_devices_as_outside_the_card_pool the_card_lease_hands_its_return_to_the_ledger_and_every_booking_entry_polls_first the_card_pool_hands_its_kept_devices_to_the_ledger_before_booking the_card_pool_module_shadows_neither_std_nor_wgpu_nor_the_functions_its_pinned_bodies_call a_card_hands_out_its_own_device_and_its_own_device_condition"
for f in "$WORK"/scan-inputs/g*.rs; do echo "== $(basename "$f" .rs)"; SRC=$f bash "$CAP" 2G nice -n 19 "$WORK"/scan/target/release/gpu $T 2>&1 | cut -c1-260; done > "$OUT"/k3-gpu.log
step "八、K2：第二十五块 17 行（第 3624–3640 行）与 B1–B4，施加在抄出来的那份源文件上"
for r in $(seq 3624 3640); do bash "$MODEL"/scan-mutant.sh "row:$r" "$WORK" | tee -a "$OUT/rerun-steps.log"; done
ATK_SCAN_INPUTS="c09-red-S14b-in-square-brackets" bash "$MODEL"/scan-mutant.sh B1-crash-depth-counts-only-parentheses "$WORK" | tee -a "$OUT/rerun-steps.log"
ATK_SCAN_INPUTS="c09-red-S14b-in-square-brackets c03-red-S14b-wrapped-after-an-item-ending-in-a-semicolon" bash "$MODEL"/scan-mutant.sh B2-crash-depth-counts-only-parentheses-both-sides "$WORK" | tee -a "$OUT/rerun-steps.log"
ATK_SCAN_INPUTS="g09-red-S14b-in-square-brackets g02-red-S14b-wrapped-after-an-item-ending-in-a-semicolon" bash "$MODEL"/scan-mutant.sh B3-gpu-depth-counts-only-parentheses "$WORK" | tee -a "$OUT/rerun-steps.log"
bash "$MODEL"/scan-mutant.sh B4-crash-bool-prefix-of-an-identifier-counts "$WORK" | tee -a "$OUT/rerun-steps.log"
step "九、K4：上一轮冻结副本与这一轮的 crates 差在哪"
VW=$HOME/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree
if [ -d "$VW" ]; then
  { diff -rq "$VW"/crates "$FROZEN"/crates; cmp "$VW"/crates/singlefs-core/src/mount.rs "$FROZEN"/crates/singlefs-core/src/mount.rs && echo "mount.rs 相同"
    diff <(sed -n '/^pub fn refusal_is_short_of_space/,/^}/p' "$VW"/crates/singlefs-core/src/mounted_session.rs) <(sed -n '/^pub fn refusal_is_short_of_space/,/^}/p' "$FROZEN"/crates/singlefs-core/src/mounted_session.rs) && echo "refusal_is_short_of_space 函数体相同"; } > "$OUT"/k4.log 2>&1
else echo "上一轮冻结副本不在：$VW" > "$OUT"/k4.log; fi
step "复跑完：$OUT"
