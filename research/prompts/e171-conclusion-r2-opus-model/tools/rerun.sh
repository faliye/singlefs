#!/usr/bin/env bash
# e171-conclusion-r2 攻方腿（Opus）报告里引的数，在副本里从头复跑。只读仓、E171 导出目录与两棵冻住的树，只写 <工作目录>。
# 用法：bash tools/rerun.sh <工作目录（不存在）> [线程数，默认 15]
# 要的东西：E171 跑的那棵树 /tmp/claude-1000/e171-s4/frozen-tree/（这一轮叫「s4 树」）、这一轮的冻结副本 /tmp/claude-1000/e171-conclusion-r1/tree/
# （「今天的树」，它没有 Cargo.toml / Cargo.lock，取 e171-conclusion-r1 攻方模型目录里那两份）、scripts/experiments/target/e171-dumps/。
# 跑的都是 harness 档的自写探针（测试目标名不带 layer0）与 Python；不跑 checker 档测试、不起 E171 装置。每一步编译与跑都经内存包装。
# 跑完逐个与 <模型目录>/out/ 下同名文件比（比法在末尾），judge-*.tsv 原始逐态文件不比（这一版探针多一列第二截判定），比它们的汇总 judge-summary.out。
set -euo pipefail
MODEL=$(cd "$(dirname "$0")/.." && pwd)
WORK=$1
THREADS=${2:-15}
REPO=/home/fy5090/code/singlefs
S4=/tmp/claude-1000/e171-s4/frozen-tree
TODAY=/tmp/claude-1000/e171-conclusion-r1/tree
R1MODEL=$REPO/research/prompts/e171-conclusion-r1-opus-model
D=$REPO/scripts/experiments/target/e171-dumps
CAP="bash $REPO/scripts/hosts/run-with-memory-cap.sh"
CAPPED="bash $REPO/scripts/hosts/capped.sh $THREADS"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='e171-conclusion-r2 攻方复跑：自写探针换输入或换二进制'
export PYTHONDONTWRITEBYTECODE=1
mkdir "$WORK"
cd "$WORK"
mkdir out bin jb
cp -r "$TODAY/scripts/experiments/e170_second_run_o3_spec_executor" o3copy

echo "== 1 两棵树的副本"
mkdir ws ws_s4
rsync -a --exclude target "$S4/" ws_s4/
rsync -a "$TODAY/crates" ws/
cp "$R1MODEL/workspace-Cargo.toml" ws/Cargo.toml
cp "$R1MODEL/workspace-Cargo.lock" ws/Cargo.lock
rsync -a "$REPO/.cargo" ws/
for w in ws ws_s4; do cp "$MODEL"/opus_r2_drive_probe.rs "$MODEL"/opus_r2_judge_probe.rs $w/crates/singlefs-harness/tests/; done
cp "$MODEL/opus_r2_history_probe_s4.rs" ws_s4/crates/singlefs-harness/tests/
build() { (cd "$1" && nice -n 19 $CAP 6G $CAPPED cargo test --offline --release -p singlefs-harness --test "$2" --no-run 2>&1 | tail -n 1); }
take() { cp "$(ls -t "$1"/target/release/deps/"$2"-* | grep -v '\.d$' | head -n 1)" "bin/$3"; }
mutate() { python3 "$MODEL/apply_mutation.py" ws_s4 ws_s4/crates/mutations.tsv "$1" "$2"; }
restore() { python3 "$MODEL/apply_mutation.py" ws_s4 --restore "$1"; touch "ws_s4/$1"; }
for w in ws ws_s4; do build $w opus_r2_drive_probe; build $w opus_r2_judge_probe; done
take ws opus_r2_drive_probe drive_today; take ws opus_r2_judge_probe judge_today
take ws_s4 opus_r2_drive_probe drive_s4; take ws_s4 opus_r2_judge_probe judge_s4
build ws_s4 opus_r2_history_probe_s4; take ws_s4 opus_r2_history_probe_s4 hist_s4
mutate crates/singlefs-core/src/allocator.rs '步 3：重建分配器时不把已释放的放进 defer 队列'
build ws_s4 opus_r2_drive_probe; take ws_s4 opus_r2_drive_probe drive_s4_m125; restore crates/singlefs-core/src/allocator.rs
mutate crates/singlefs-core/src/mount.rs '步 5：抬 F 生效之后不放开扣住的槽'
build ws_s4 opus_r2_drive_probe; take ws_s4 opus_r2_drive_probe drive_s4_m34; restore crates/singlefs-core/src/mount.rs
mutate crates/singlefs-core/src/inode_tree.rs '并行线三 C116：分裂出来的右半容器号不是触发分裂那条新记录的 inode 号'
build ws_s4 opus_r2_history_probe_s4; take ws_s4 opus_r2_history_probe_s4 hist_s4_m23; restore crates/singlefs-core/src/inode_tree.rs

echo "== 2 流二驱动（s4 树基线、第 34 行、第 125 行；今天的树基线），与 E171 导出的 ops.tsv 逐条比"
drive() { OPUS_DRIVE_OUT=$PWD/out/$2.tsv OPUS_DRIVE_LABEL=$3 nice -n 19 $CAP 68M "bin/$1" --test-threads 1 > /dev/null 2>&1; }
drive drive_s4 drive-s4-base s4base; drive drive_s4_m34 drive-s4-m34 s4m34; drive drive_s4_m125 drive-s4-m125 s4m125; drive drive_today drive-base base
F=fixed_script_through_e_then_unmount
{ python3 "$MODEL/compare_ops.py" out/drive-s4-base.tsv s4base $D/baseline-run1/$F
  python3 "$MODEL/compare_ops.py" out/drive-s4-m34.tsv s4m34 $D/rows-s4/row-34/dump/$F
  python3 "$MODEL/compare_ops.py" out/drive-s4-m125.tsv s4m125 $D/baseline-run1/$F
  python3 "$MODEL/compare_ops.py" out/drive-s4-m125.tsv s4m125 $D/rows-s4/row-34/dump/$F
  python3 "$MODEL/compare_ops.py" out/drive-base.tsv base $D/baseline-run1/$F; } > out/compare-ops.out

echo "== 3 两种「已返回的调用丢了」写流：冻结 O3、实现恢复、O2、O1-崩溃判官、RefFS、两次可写挂载"
python3 "$MODEL/o3_ack_shapes.py" o3copy warmup $D/baseline-run1/new_pool_file_creation shape-warmup > out/shapes.out
python3 "$MODEL/o3_ack_shapes.py" o3copy content $D/baseline-run1/$F shape-content >> out/shapes.out
cp shape-warmup/o3-verdicts.tsv out/shape-warmup-o3-verdicts.tsv; cp shape-content/o3-verdicts.tsv out/shape-content-o3-verdicts.tsv
n=0
for bin in judge_s4 judge_today; do for shape in warmup content; do n=$((n + 1))
  case $shape in
    warmup) C='0,0:none;1,1:none;1,2:none'; R='create:first,publish';;
    content) C='0,0:none;1,1:none;1,2:none;1,3:first'; R='create:first,publish,overwrite:second,publish';;
  esac
  OPUS_JUDGE_COMMITTED="$C" OPUS_JUDGE_REFFS="$R" nice -n 19 $CAP 8G bash "$MODEL/run_judge_batch.sh" "$PWD/bin/$bin" $n "$THREADS" "$PWD/jb" "" "$shape=$PWD/shape-$shape" | tail -n 1
  cat jb/j$n-*.tsv | sort -t$'\t' -k2,2n > "out/judge-$bin-$shape.tsv"
  if [ "$bin" = judge_s4 ]; then label=judge_s4_v2; else label=judge_v2; fi
  python3 "$MODEL/summarize_judge.py" "$shape-$label" "shape-$shape/o3-verdicts.tsv" "out/judge-$bin-$shape.tsv" >> out/judge-summary.out
done; done

echo "== 4 单镜像谓词 W：基线与计入分母的 37 行（批 1、批 2）"
P=$REPO/research/results/e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out
args="base1=$D/baseline-run1/new_pool_file_creation base2=$D/baseline-run1/$F"
for r in $(grep -o 'name=row row=[0-9]* .*category=counted' $P | grep -o 'row=[0-9]*' | cut -d= -f2); do
  for t in rows-s4 rows-s4-f4; do if [ -d $D/$t/row-$r/dump ]; then dd=$D/$t/row-$r/dump; fi; done
  for s in new_pool_file_creation $F; do if [ -f $dd/$s/states.tsv ]; then args="$args r${r}${s:0:3}=$dd/$s"; fi; done
done
nice -n 19 python3 "$MODEL/witness_consistency.py" o3copy $args > out/witness-consistency-all.out
WITNESS_PER_STATE=$PWD/out/witness-per-state.tsv nice -n 19 python3 "$MODEL/witness_consistency.py" o3copy \
  base1=$D/baseline-run1/new_pool_file_creation base2=$D/baseline-run1/$F \
  r242new=$D/rows-s4-f4/row-242/dump/new_pool_file_creation r242fix=$D/rows-s4-f4/row-242/dump/$F \
  r269new=$D/rows-s4-f4/row-269/dump/new_pool_file_creation r269fix=$D/rows-s4-f4/row-269/dump/$F \
  r270new=$D/rows-s4-f4/row-270/dump/new_pool_file_creation r270fix=$D/rows-s4-f4/row-270/dump/$F > /dev/null

echo "== 5 第 242、269、270 行与基线的态上两次可写挂载、崩溃注入第二截判拒绝合不合法（s4 树与今天的树）"
R242="r242new=$D/rows-s4-f4/row-242/dump/new_pool_file_creation r242fix=$D/rows-s4-f4/row-242/dump/$F r269new=$D/rows-s4-f4/row-269/dump/new_pool_file_creation r269fix=$D/rows-s4-f4/row-269/dump/$F"
RBASE="base1=$D/baseline-run1/new_pool_file_creation base2=$D/baseline-run1/$F r270new=$D/rows-s4-f4/row-270/dump/new_pool_file_creation r270fix=$D/rows-s4-f4/row-270/dump/$F"
refuse() { OPUS_JUDGE_COMMITTED='0,0:none' OPUS_JUDGE_REFFS='create:first,publish' nice -n 19 $CAP 8G bash "$MODEL/run_judge_batch.sh" "$PWD/bin/$1" $2 "$THREADS" "$PWD/jb" 1,2 $3 | tail -n 1; cat jb/j$2-*.tsv | sort -t$'\t' -k1,1 -k2,2n > "out/$4"; }
refuse judge_s4 7 "$R242" refusal-judge-s4-242-269.tsv
refuse judge_s4 8 "$RBASE" refusal-judge-s4-base-270.tsv
refuse judge_today 9 "$R242" refusal-judge-today-242-269.tsv
{ python3 "$MODEL/summarize_refusals.py" s4 out/refusal-judge-s4-242-269.tsv out/refusal-judge-s4-base-270.tsv
  python3 "$MODEL/summarize_refusals.py" today out/refusal-judge-today-242-269.tsv; } > out/refusal-judge-summary.out
python3 "$MODEL/join_witness_and_mount.py" out/witness-per-state.tsv out/refusal-judge-s4-242-269.tsv out/refusal-judge-s4-base-270.tsv > out/witness-vs-mount.out

echo "== 6 第 23 行在 s4 树上：基线与打了变异的 core 各建 233 个 inode"
hist() { OPUS_HIST_DUMP=$1 OPUS_HIST_CASE=inodes OPUS_HIST_LABEL=$2 OPUS_HIST_OUT=$PWD/out/hist-s4.tsv nice -n 19 $CAP 3G "bin/$3" --test-threads 1 > /dev/null 2>&1; }
hist $D/baseline-run1/new_pool_file_creation base_s1_s4 hist_s4
hist $D/rows-s4/row-23/dump/new_pool_file_creation row23_s1_m23core_s4 hist_s4_m23

echo "== 7 与模型目录 out/ 比（没有输出就是逐字节相同）"
for f in compare-ops.out drive-s4-base.tsv drive-s4-m34.tsv drive-s4-m125.tsv drive-base.tsv shapes.out shape-warmup-o3-verdicts.tsv \
         shape-content-o3-verdicts.tsv judge-summary.out witness-consistency-all.out witness-per-state.tsv witness-vs-mount.out \
         refusal-judge-summary.out hist-s4.tsv; do
  cmp "out/$f" "$MODEL/out/$f" || echo "DIFFERS $f"
done
