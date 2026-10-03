#!/usr/bin/env bash
# e171-conclusion-r3 正推腿（Sonnet）：在「今天的树」（本轮冻结副本 /tmp/claude-1000/e171-conclusion-r3/tree/，与仓里 crates/ 逐字节同）上
# 重做第二轮攻方「实现一侧」的数：61 个见证拒（第 242 行 9、第 269 行 52）与 RefFS 27 / 27、36 态。
# 状态：没跑（要编译 harness 档探针，agent-common.md「不做」一节不让子 agent 编 Rust）；交主 agent 跑，bash -n 过。
# 改自 research/prompts/e171-conclusion-r2-opus-model/tools/rerun.sh 的第 1、3、5 步，只编今天的树那一份探针，不编 s4 树、不打变异。
# 用法：bash research/prompts/e171-conclusion-r3-sonnet-model/rerun-on-r3-tree.sh <工作目录（不存在）> [线程数，默认 15]
# 要的东西：/tmp/claude-1000/e171-conclusion-r3/tree/（crates/）、research/target/e171-dumps/、仓里的 O3 执行器目录。
# 跑完逐文件比（见末尾）：期望 judge-summary 与第二轮 judge_today 那一份逐字同、refusal-judge-summary 的 core=today 行与第二轮同；
# 不同的行就是今天的 core 与 s4 / 第一轮的 core 在这些态上给出不同的挂载结局，要在判决里单列。
set -euo pipefail
R2MODEL=/home/fy5090/code/singlefs/research/prompts/e171-conclusion-r2-opus-model
R1MODEL=/home/fy5090/code/singlefs/research/prompts/e171-conclusion-r1-opus-model
REPO=/home/fy5090/code/singlefs
R3=/tmp/claude-1000/e171-conclusion-r3/tree
WORK=$1
THREADS=${2:-15}
D=$REPO/research/target/e171-dumps
CAP="bash $REPO/research/scripts/run-with-memory-cap.sh"
CAPPED="bash $REPO/research/scripts/capped.sh $THREADS"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='e171-conclusion-r3 正推复跑：自写探针换输入或换二进制'
export PYTHONDONTWRITEBYTECODE=1
mkdir "$WORK"
cd "$WORK"
mkdir out bin jb
cp -r "$REPO/research/e7-index-bench/src/bin/e170_second_run_o3_spec_executor" o3copy
F=fixed_script_through_e_then_unmount

echo "== 1 今天的树的副本，编两个探针"
mkdir ws
rsync -a "$R3/crates" ws/
cp "$R1MODEL/workspace-Cargo.toml" ws/Cargo.toml
cp "$R1MODEL/workspace-Cargo.lock" ws/Cargo.lock
rsync -a "$REPO/.cargo" ws/
cp "$R2MODEL"/opus_r2_drive_probe.rs "$R2MODEL"/opus_r2_judge_probe.rs ws/crates/singlefs-harness/tests/
(cd ws && nice -n 19 $CAP 6G $CAPPED cargo test --offline --release -p singlefs-harness --test opus_r2_judge_probe --no-run 2>&1 | tail -n 1)
cp "$(ls -t ws/target/release/deps/opus_r2_judge_probe-* | grep -v '\.d$' | head -n 1)" bin/judge_r3

echo "== 3 两种「已返回的调用丢了」写流 × 今天的 core：O3、实现恢复、O2、O1-崩溃判官、RefFS、两次可写挂载"
python3 "$R2MODEL/o3_ack_shapes.py" o3copy warmup $D/baseline-run1/new_pool_file_creation shape-warmup > out/shapes.out
python3 "$R2MODEL/o3_ack_shapes.py" o3copy content $D/baseline-run1/$F shape-content >> out/shapes.out
n=0
for shape in warmup content; do n=$((n + 1))
  case $shape in
    warmup) C='0,0:none;1,1:none;1,2:none'; R='create:first,publish';;
    content) C='0,0:none;1,1:none;1,2:none;1,3:first'; R='create:first,publish,overwrite:second,publish';;
  esac
  OPUS_JUDGE_COMMITTED="$C" OPUS_JUDGE_REFFS="$R" nice -n 19 $CAP 8G bash "$R2MODEL/run_judge_batch.sh" "$PWD/bin/judge_r3" $n "$THREADS" "$PWD/jb" "" "$shape=$PWD/shape-$shape" | tail -n 1
  cat jb/j$n-*.tsv | sort -t$'\t' -k2,2n > "out/judge-judge_r3-$shape.tsv"
  python3 "$R2MODEL/summarize_judge.py" "$shape-judge_r3" "shape-$shape/o3-verdicts.tsv" "out/judge-judge_r3-$shape.tsv" >> out/judge-summary-r3.out
done

echo "== 5 第 242、269 行与基线、第 270 行的态上两次可写挂载 + 崩溃注入第二截判拒绝"
R242="r242new=$D/rows-s4-f4/row-242/dump/new_pool_file_creation r242fix=$D/rows-s4-f4/row-242/dump/$F r269new=$D/rows-s4-f4/row-269/dump/new_pool_file_creation r269fix=$D/rows-s4-f4/row-269/dump/$F"
RBASE="base1=$D/baseline-run1/new_pool_file_creation base2=$D/baseline-run1/$F r270new=$D/rows-s4-f4/row-270/dump/new_pool_file_creation r270fix=$D/rows-s4-f4/row-270/dump/$F"
refuse() { OPUS_JUDGE_COMMITTED='0,0:none' OPUS_JUDGE_REFFS='create:first,publish' nice -n 19 $CAP 8G bash "$R2MODEL/run_judge_batch.sh" "$PWD/bin/$1" $2 "$THREADS" "$PWD/jb" 1,2 $3 | tail -n 1; cat jb/j$2-*.tsv | sort -t$'\t' -k1,1 -k2,2n > "out/$4"; }
refuse judge_r3 7 "$R242" refusal-judge-r3-242-269.tsv
refuse judge_r3 8 "$RBASE" refusal-judge-r3-base-270.tsv
python3 "$R2MODEL/summarize_refusals.py" r3 out/refusal-judge-r3-242-269.tsv out/refusal-judge-r3-base-270.tsv > out/refusal-judge-summary-r3.out

echo "== 7 与第二轮攻方的输出比（有输出就是今天的 core 与它们不同）"
grep 'judge_v2' "$R2MODEL/out/judge-summary.out" | sed 's/judge_v2/judge_r3/' > out/expected-judge-summary-r3.out || true
diff out/judge-summary-r3.out out/expected-judge-summary-r3.out || echo "DIFFERS judge-summary（今天的 core 与第二轮的 core 给出不同的逐态结局）"
grep -E 'label=r(242|269)' out/refusal-judge-summary-r3.out | sed 's/^name=refusal_judge core=[a-z0-9]* /name=refusal_judge core=X /' | sort > out/r3-refusals.norm
grep -h 'core=today' "$R2MODEL/out/refusal-judge-summary.out" | sed 's/^name=refusal_judge core=[a-z0-9]* /name=refusal_judge core=X /' | sort > out/r2-today-refusals.norm
diff out/r3-refusals.norm out/r2-today-refusals.norm || echo "DIFFERS refusals（只比第 242、269 行；第 270 行与基线的行第二轮只在 s4 那一份里，今天的树这次多出来的那几行没有对照）"
