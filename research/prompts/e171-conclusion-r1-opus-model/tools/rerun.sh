#!/usr/bin/env bash
# e171-conclusion-r1 攻方腿（Opus）报告里引的数，在副本里复跑。只读仓与冻结副本，只写 <工作目录>。
# 用法：bash tools/rerun.sh <工作目录（不存在）> [线程数，默认 15]
# 要的东西：冻结副本 /tmp/claude-1000/e171-conclusion-r1/tree/、E171 导出 research/target/e171-dumps/、比对器产物。
# 跑的都是 harness 档的自写探针（测试目标名不带 layer0）与 Python；不跑 checker 档测试、不起 E171 装置。
set -euo pipefail
MODEL=$(cd "$(dirname "$0")/.." && pwd)
WORK=$1
THREADS=${2:-15}
REPO=/home/fy5090/code/singlefs
FROZEN=/tmp/claude-1000/e171-conclusion-r1/tree
DUMPS=$REPO/research/target/e171-dumps
CAP="bash $REPO/research/scripts/run-with-memory-cap.sh"
CAPPED="bash $REPO/research/scripts/capped.sh $THREADS"
mkdir "$WORK"
cd "$WORK"
export PYTHONDONTWRITEBYTECODE=1
cp -r "$FROZEN/research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor" o3copy

echo "== 1 第 270 / 242 行：执行员式与 C577 式（见证值）两种数法（G1）"
nice -n 19 python3 "$MODEL/witness_after_return.py" o3copy \
  base1=$DUMPS/baseline-run1/new_pool_file_creation base2=$DUMPS/baseline-run1/fixed_script_through_e_then_unmount \
  r270s1=$DUMPS/rows-s4-f4/row-270/dump/new_pool_file_creation r270s2=$DUMPS/rows-s4-f4/row-270/dump/fixed_script_through_e_then_unmount

echo "== 2 只让 O3 红的坏实现（写流模拟，G3）"
nice -n 19 python3 "$MODEL/o3_ack_probe.py" o3copy "$DUMPS/baseline-run1/new_pool_file_creation" ackdump

echo "== 3 计入分母的 37 行各自的抓手测试（G4）"
python3 "$MODEL/killer_tests_of_counted_rows.py" "$FROZEN" "$REPO/research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out" | tail -n 38 | head -n 37

echo "== 4 挑态（第 269 行与基线同号、第 270 行、第 242 行）"
mkdir sel
for s in new_pool_file_creation fixed_script_through_e_then_unmount; do
  python3 "$MODEL/select_states.py" o3copy "$DUMPS/rows-s4-f4/row-269/dump/$s" "sel/r269-$s" differs-from-baseline "$DUMPS/baseline-run1/$s"
  python3 "$MODEL/select_states.py" o3copy "$DUMPS/baseline-run1/$s" "sel/b269-$s" same-as-baseline-numbers "sel/r269-$s"
  python3 "$MODEL/select_states.py" o3copy "$DUMPS/rows-s4-f4/row-270/dump/$s" "sel/r270-$s" rotation-not-durable-after-return
  python3 "$MODEL/select_states.py" o3copy "$DUMPS/rows-s4-f4/row-242/dump/$s" "sel/r242-$s" rotation-landed-root-not
done
mkdir sel/r242b2-fixed_script_through_e_then_unmount
for f in sel/r242-fixed_script_through_e_then_unmount/*; do n=$(basename "$f"); [ "$n" = states.tsv ] || ln -s "$(readlink -f "$f")" "sel/r242b2-fixed_script_through_e_then_unmount/$n"; done
awk -F'\t' '$4==2' sel/r242-fixed_script_through_e_then_unmount/states.tsv > sel/r242b2-fixed_script_through_e_then_unmount/states.tsv
mkdir sel/r242b1-fixed_script_through_e_then_unmount
for f in sel/r242-fixed_script_through_e_then_unmount/*; do n=$(basename "$f"); [ "$n" = states.tsv ] || ln -s "$(readlink -f "$f")" "sel/r242b1-fixed_script_through_e_then_unmount/$n"; done
awk -F'\t' '$4==1' sel/r242-fixed_script_through_e_then_unmount/states.tsv > sel/r242b1-fixed_script_through_e_then_unmount/states.tsv

echo "== 5 仓副本与两个探针（基线 core）"
mkdir ws
rsync -a "$FROZEN/crates" ws/
cp "$MODEL/workspace-Cargo.toml" ws/Cargo.toml
cp "$MODEL/workspace-Cargo.lock" ws/Cargo.lock
rsync -a "$REPO/.cargo" ws/
cp "$MODEL/opus_e171_crash_then_mount_probe.rs" "$MODEL/opus_e171_history_probe.rs" ws/crates/singlefs-harness/tests/
build() { (cd ws && RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='副本整包首编，峰值表里没有这条命令' nice -n 19 $CAP 6G $CAPPED cargo test --offline --release -p singlefs-harness --test "$1" --no-run 2>&1 | tail -n 1); }
binary_of() { ls -t ws/target/release/deps/"$1"-* | grep -v '\.d$' | head -n 1; }
build opus_e171_crash_then_mount_probe
build opus_e171_history_probe
mkdir bin out
cp "$(binary_of opus_e171_crash_then_mount_probe)" bin/crash_probe
cp "$(binary_of opus_e171_history_probe)" bin/history_probe

echo "== 6 崩溃之后可写挂载（G1 的 269 / 270 / 242，G3 的写流模拟）"
nice -n 19 $CAP 8G bash "$MODEL/run_crash_probe_batch.sh" "$PWD/bin/crash_probe" 1 "$THREADS" "$PWD/out" 1 ack=$PWD/ackdump
nice -n 19 $CAP 8G bash "$MODEL/run_crash_probe_batch.sh" "$PWD/bin/crash_probe" 3 "$THREADS" "$PWD/out" 1,2 \
  r269s1=$PWD/sel/r269-new_pool_file_creation b269s1=$PWD/sel/b269-new_pool_file_creation \
  r269s2=$PWD/sel/r269-fixed_script_through_e_then_unmount b269s2=$PWD/sel/b269-fixed_script_through_e_then_unmount \
  r270s1=$PWD/sel/r270-new_pool_file_creation r270s2=$PWD/sel/r270-fixed_script_through_e_then_unmount \
  r242s1=$PWD/sel/r242-new_pool_file_creation r242s2b2=$PWD/sel/r242b2-fixed_script_through_e_then_unmount
nice -n 19 $CAP 8G bash "$MODEL/run_crash_probe_batch.sh" "$PWD/bin/crash_probe" 4 "$THREADS" "$PWD/out" 1 \
  r242s2b1=$PWD/sel/r242b1-fixed_script_through_e_then_unmount
cat out/b1-*.tsv | sort -t$'\t' -k2,2n > out/ack.tsv
cat out/b3-*.tsv out/b4-*.tsv > out/sel.tsv
python3 "$MODEL/summarize_probe.py" ackdump/o3-verdicts.tsv out/ack.tsv out/sel.tsv

echo "== 7 短历史（第 42 行卸载之后再挂与回退扫描；基线 core 上建 233 个 inode、unlink 当参照）"
hist() { OPUS_HIST_DUMP=$1 OPUS_HIST_CASE=$2 OPUS_HIST_LABEL=$3 OPUS_HIST_OUT=$PWD/out/hist.tsv nice -n 19 $CAP 3G "$4" --test-threads 1 > /dev/null 2>&1; }
hist "$DUMPS/baseline-run1/fixed_script_through_e_then_unmount" remount_rollback base_s2 bin/history_probe
hist "$DUMPS/rows-s4/row-42/dump/fixed_script_through_e_then_unmount" remount_rollback row42_s2 bin/history_probe
hist "$DUMPS/baseline-run1/new_pool_file_creation" inodes base_s1 bin/history_probe
hist "$DUMPS/baseline-run1/new_pool_file_creation" unlink base_s1 bin/history_probe

echo "== 8 第 23 行的变异打进 core 之后建 233 个 inode（G1）"
python3 "$MODEL/apply_mutation.py" ws ws/crates/mutations.tsv crates/singlefs-core/src/inode_tree.rs '并行线三 C116：分裂出来的右半容器号不是触发分裂那条新记录的 inode 号'
build opus_e171_history_probe
cp "$(binary_of opus_e171_history_probe)" bin/history_probe_m23
python3 "$MODEL/apply_mutation.py" ws --restore crates/singlefs-core/src/inode_tree.rs
hist "$DUMPS/rows-s4/row-23/dump/new_pool_file_creation" inodes row23_s1_m23core bin/history_probe_m23
cut -c1-400 out/hist.tsv
