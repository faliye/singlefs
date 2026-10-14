#!/usr/bin/env bash
# 攻方腿（m3-batch10z-vw-code-r1，opus）模型复跑：bash rerun.sh [<工作目录>]
# 只跑 harness 档与独立小 crate：攻方测试目标 atk_vw_opus（冻结副本那份 harness 测试文件 + atk_vw_part*.rs，名字不带 layer0）、
# 十五个攻方变异、变异表第 3575/3577/3583/3584 行与 M10 对 core 库单测、K4 扫源码小 crate。不起 checker 档测试、不枚举崩溃状态。
# 每件经 run-with-memory-cap.sh（3G；编译 12G；并行一批合计不超过 12G），线程经 capped.sh 15。整份约 30 分钟（推的：各件这一轮实测相加）。
set -euo pipefail
ROOT=/home/fy5090/code/singlefs
MODEL=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
FROZEN=$HOME/.local/share/singlefs/work/m3-batch10z-vw-code-r1/tree
WORK=${1:-$HOME/.local/share/singlefs/work/m3-batch10z-vw-code-r1-opus-rerun}
CAP=$ROOT/scripts/hosts/run-with-memory-cap.sh
THREADS=$ROOT/scripts/hosts/capped.sh
OUT=$WORK/runs; mkdir -p "$WORK" "$OUT"
A=none,same,bigger,inode,overwrite,push-swapped,push-reversed,unmount-remount,frozen-overwrite,push-fail-1,push-fail-150,ask-then-same,table-reversed,table-swapped,table-one,table-stale,inode-then-same-1,inode-then-same-3,overwrite-then-same-1,overwrite-then-same-3,overwrite-then-same-6
B=none,same,table-reversed,table-stale,frozen-overwrite,inode-then-same-1,overwrite-then-same-1,ask-then-same,push-fail-1
# 一、攻方测试目标
rsync -a --delete --exclude target --exclude .git "$FROZEN"/ "$WORK"/repo/
bash "$MODEL"/assemble.sh "$WORK"/repo "$MODEL"
( cd "$WORK"/repo && bash "$CAP" 12G env CARGO_TARGET_DIR="$WORK"/target nice -n 19 bash "$THREADS" 15 cargo test --offline --release -p singlefs-harness --test atk_vw_opus --no-run )
BIN=$(ls -t "$WORK"/target/release/deps/atk_vw_opus-* | grep -v '\.d$' | head -1)
t() { bash "$CAP" 3G env $1 nice -n 19 bash "$THREADS" 3 "$BIN" --ignored --nocapture --exact "$2" > "$OUT/$3" 2>&1; }
# K1-1：主形九形（三批）、实现员三形、两槽 C、被抛弃根 A 与 C、窄四档单元区
for pre in 2 4 8; do for fill in 6 11 16 21; do t "ATK_PRES=$pre ATK_FILLS=$fill ATK_ACTIONS=$A" atk_k1_ways_out_vs_real_sweep "k1-pre$pre-fill$fill.log"; done; done
t "ATK_ACTIONS=$B" atk_k1_implementer_forms_ways_out_vs_real fam-impl.log
t "ATK_PRE_WRITES=0,1,3,6,10 ATK_ACTIONS=$B" atk_k1_two_c_slots_ways_out_vs_real fam-two.log
t "ATK_KS=0,1,2,4,6 ATK_ACTIONS=$B" atk_k1_abandoned_root_and_c_ways_out_vs_real fam-ab.log
for a in 352 320 288 256; do for p in 2 4 8; do t "ATK_AREA=$a ATK_PRES=$p ATK_ACTIONS=$B" atk_k1_ways_out_vs_real_sweep "area$a-$p.log"; done; done
# K1-2：报里记的「还剩什么」四种改动
t "" atk_k1_remainder_of_each_kind_of_change kinds.log
# K1-3：只比实例代号与 txg（别的池同一对数）
t "" atk_k1_ways_out_of_another_pools_report other.log
# K1-3 反面：有树而文件集合为空的一版上被宽护卡住
t "" atk_k1_stuck_on_a_version_with_trees_and_no_file k1nofile.log
# K1-4：要出路那一次读盘出错
for c in 4:16:13 4:16:20 2:16:17 8:11:40; do t "ATK_STEP_N=1 ATK_CELLS=$c" atk_k1_read_faults_while_asking_for_the_ways_out "askrd-${c//:/-}.log"; done  # clock-times:allow 槽:代:行三元组，不是钟点
# K3：造 F、H 推完之后那一段与 walkSkipped 分开
t "ATK_RESTICK=1 ATK_OWS=$(seq -s, 0 22) ATK_LEAVES=$(seq -s, 0 90)" atk_k3_walk_stuck_with_a_push_that_goes_through k3walk-restick.log
# K3：十五个攻方变异（每个从冻结副本另拷一份）
SPECS=(
  "atk_k3_walk_stuck_with_a_push_that_goes_through@ATK_RESTICK=1+ATK_OWS=10,11+ATK_LEAVES=27,28,29+ATK_KEEP_ISO=4"
  "atk_k3_walk_stuck_with_a_push_that_goes_through@ATK_RESTICK=1+ATK_OWS=10,11+ATK_LEAVES=27,28,29"
  "atk_k1_remainder_of_each_kind_of_change@ATK_X=1"
  "atk_k1_ways_out_vs_real_sweep@ATK_PRES=4+ATK_FILLS=16+ATK_ACTIONS=none,unmount-remount,same"
  "atk_k1_ways_out_of_another_pools_report@ATK_X=1"
)
for mutant in $(python3 "$MODEL"/mutants.py x --list); do bash "$MODEL"/run-mutant.sh "$mutant" "$WORK" "${SPECS[@]}"; done
# K1-2 / K3：变异表里第 12、14、20、21 行与 M10 对 core 库单测
for row in 3575 3577 3583 3584; do
  rsync -a --delete --exclude target --exclude .git "$FROZEN"/ "$WORK"/repo-row/
  read -r args test < <(python3 "$MODEL"/apply-row.py "$WORK"/repo-row "$row" | tr '\n' ' ')
  ( cd "$WORK"/repo-row && bash "$CAP" 12G env CARGO_TARGET_DIR="$WORK"/target-row nice -n 19 bash "$THREADS" 15 cargo test --offline --release $args ) > "$OUT"/row-$row.log 2>&1 || true
done
# K4：扫源码小 crate（抄 checker 档两份扫源码函数与用例），对攻法输入各扫一次
bash "$CAP" 4G python3 "$MODEL"/build-scan-proto.py "$FROZEN" "$WORK"/scan-proto > "$OUT"/scan-build.log 2>&1 || true
( cd "$WORK"/scan-proto && bash "$CAP" 4G nice -n 19 bash "$THREADS" 4 cargo build --offline --release ) >> "$OUT"/scan-build.log 2>&1
python3 "$MODEL"/gen-scan-inputs.py "$FROZEN" "$WORK"/scan-inputs > "$OUT"/scan-inputs.log
for f in "$WORK"/scan-inputs/c*.rs; do bash "$CAP" 2G nice -n 19 "$WORK"/scan-proto/target/release/atk-stop-scan "$f"; done > "$OUT"/k4-stop.log 2>&1
for f in "$WORK"/scan-inputs/g*.rs; do bash "$CAP" 2G nice -n 19 "$WORK"/scan-proto/target/release/atk-gpu-scan "$f"; done > "$OUT"/k4-gpu.log 2>&1
echo "复跑完：$OUT"
