#!/usr/bin/env bash
# e171-conclusion-r3 攻方腿（Opus）报告里引的数，在副本里从头复跑。只读仓、E171 导出目录与两棵冻住的树，只写 <工作目录>。
# 用法：bash tools/rerun.sh <工作目录（不存在）> [线程数，默认 15] [hidden|all，默认 hidden]
#   hidden：返回值探针只跑 untouched-subset.tsv 的 13 行（7 行交回值不同的、6 行对照），约 4 分钟；
#   all：跑 untouched-s4.tsv 里除 make_filesystem.rs 之外的 232 行，约 45 分钟（一行一次重编，每行约 11–12 秒），主 agent 放后台跑。
# 要的东西：E171 跑的树 /tmp/claude-1000/e171-s4/frozen-tree/（s4 树）、这一轮的冻结副本 /tmp/claude-1000/e171-conclusion-r3/tree/、
#   scripts/experiment-apparatus/target/e171-dumps/、仓里的 E170 冻结执行器与 E171 研究一侧目录。
# 跑的是 harness 档的自写探针（测试目标名不带 layer0）与 Python；不跑 checker 档测试、不起 E171 装置。编译与跑探针都经内存包装。
# 末尾逐个与 <模型目录>/out/ 下同名文件 cmp，没有输出就是逐字节相同。
set -euo pipefail
MODEL=$(cd "$(dirname "$0")/.." && pwd)
WORK=$1
THREADS=${2:-15}
MODE=${3:-hidden}
REPO=/home/fy5090/code/singlefs
S4=/tmp/claude-1000/e171-s4/frozen-tree
TODAY=/tmp/claude-1000/e171-conclusion-r3/tree
E=$REPO/scripts/experiment-apparatus/target/e171-dumps
P=$REPO/research/results/e171-three-oracle-collusion-2026-10-03
CAP="bash $REPO/scripts/hosts/run-with-memory-cap.sh"
CAPPED="bash $REPO/scripts/hosts/capped.sh $THREADS"
export RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON='e171-conclusion-r3 攻方复跑：自写探针换输入或换变异 core'
export PYTHONDONTWRITEBYTECODE=1
mkdir "$WORK"
cd "$WORK"
mkdir out

echo "== 1 281 行的变异在 s4 树与这一轮冻结副本上还打不打得上"
python3 "$MODEL/anchor_survival.py" "$P-s4-f4-substitutes-comparator.out" "$S4" "$TODAY" > out/anchor-survival.out

echo "== 2 第 15 行新池新建文件那次发布的落点"
python3 "$MODEL/row15_offsets.py" "$E" > out/row15-offsets.out

echo "== 3 C407 改写的判官：四个 O3（冻结 / 三个变坏的）在 Q2 坏镜像与两种「已确认的丢了」写流上；判官在入库产物与造出来的产物上"
mkdir k3
for v in v0 v1 v2 v3; do
  mkdir -p k3/$v/scripts/experiment-apparatus k3/$v/research/results
  cp -r "$REPO/scripts/experiment-apparatus/e170_second_run_o3_spec_executor" "$REPO/scripts/experiment-apparatus/e171_three_oracle_collusion" k3/$v/scripts/experiment-apparatus/
  rm -rf k3/$v/scripts/experiment-apparatus/*/__pycache__
  cp "$REPO/research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out" k3/$v/research/results/
done
python3 "$MODEL/make_o3_variants.py" "$PWD/k3/v1" "$PWD/k3/v2" "$PWD/k3/v3" > /dev/null
for v in v0 v1 v2 v3; do
  nice -n 19 python3 k3/$v/scripts/experiment-apparatus/e171_three_oracle_collusion/o3_bad_image_judge.py judge --repo-root "$PWD/k3/$v" \
    --e170-dumps "$E/e170-bad-states-copy-s2" --known-bad-dir "$E/known-bad-s2" --output "$PWD/k3/images-$v.out" > /dev/null
done
diff <(grep 'name=image_judgement' k3/images-v0.out) <(grep 'name=image_judgement' "$P-s2-images-o3.out") > out/images-v0-vs-product.diff || true
(cd k3 && python3 "$MODEL/q2_o3_only.py" v0/scripts/experiment-apparatus/e171_three_oracle_collusion "$E/known-bad-s2" "$E/e170-bad-states-copy-s2" \
   v0=images-v0.out v1=images-v1.out v2=images-v2.out v3=images-v3.out) > out/q2-o3-only.out
for v in v0 v1 v2 v3; do
  echo "name=o3_controls variant=$v p1_f_new_pool_strict=$(grep 'source=e170 stream=new_pool_file_creation row=0 ' k3/images-$v.out | grep -o 'strict=[a-z_]*' | cut -d= -f2) base_images_strict_not_in=$(grep -c 'source=base .*strict=not_in' k3/images-$v.out) freeze_check_differences=$(grep 'name=freeze_check before=true' k3/images-$v.out | grep -o 'differences=[0-9]*' | cut -d= -f2)"
done > out/o3-controls.out
for v in v0 v1 v2 v3; do
  X=k3/$v/scripts/experiment-apparatus/e170_second_run_o3_spec_executor
  nice -n 19 python3 "$MODEL/o3_ack_shapes.py" $X warmup "$E/baseline-run1/new_pool_file_creation" k3/shape-warmup-$v | sed "s/^/variant=$v /"
  nice -n 19 python3 "$MODEL/o3_ack_shapes.py" $X content "$E/baseline-run1/fixed_script_through_e_then_unmount" k3/shape-content-$v | sed "s/^/variant=$v /"
done > out/shapes-variants.out
mkdir k3/products
python3 "$MODEL/make_products.py" "$P-s4-f4-substitutes-comparator.out" k3/products > /dev/null
REG=15,23,27,34,42,92,112,242,269,270,271,277
for p in "$P-s4-f4-substitutes-comparator.out" "$P-s4-f4-comparator.out" "$P-s4-comparator.out" k3/products/short.out k3/products/o3red.out k3/products/o3in.out; do
  echo "$(basename "$p") $(python3 "$MODEL/c407_judge.py" "$p" $REG)"
done > out/judge-runs.out

python3 "$MODEL/e165_pin_overlap.py" "$P-s4-f4-substitutes-comparator.out" "$REPO/.claude/gate.d/e165-pinned-red-examples.tsv" > out/e165-pin-overlap.out

echo "== 4 E171 判「未触达」的行：成功调用的交回值（s4 树副本，一行一次重编）"
mkdir ws_s4 bin dbg
rsync -a --exclude target "$S4/" ws_s4/
cp "$MODEL/opus_r3_return_probe.rs" ws_s4/crates/singlefs-harness/tests/
(cd ws_s4 && nice -n 19 $CAP 3G $CAPPED cargo test --offline --release -p singlefs-harness --test opus_r3_return_probe --no-run 2>&1 | tail -n 1)
cp "$(ls -t ws_s4/target/release/deps/opus_r3_return_probe-* | grep -v '\.d$' | head -n 1)" bin/ret_base
OPUS_DRIVE_OUT=$PWD/out/ret-base-1.tsv OPUS_DRIVE_LABEL=base nice -n 19 $CAP 256M bin/ret_base --test-threads 1 > /dev/null 2>&1
python3 "$MODEL/compare_ops.py" out/ret-base-1.tsv base "$E/baseline-run1/fixed_script_through_e_then_unmount" > out/compare-ops-base.out
if [ "$MODE" = all ]; then
  awk -F'\t' '$2 != "crates/singlefs-core/src/make_filesystem.rs"' "$MODEL/untouched-s4.tsv" > table.tsv
  bash "$MODEL/run_return_batch.sh" "$PWD/ws_s4" table.tsv 1 281 "$PWD/rb" "$THREADS" > rb.log
  python3 "$MODEL/compare_return.py" out/ret-base-1.tsv rb "$MODEL/untouched-s4.tsv" > out/return-rows.out
else
  bash "$MODEL/run_return_batch.sh" "$PWD/ws_s4" "$MODEL/untouched-subset.tsv" 1 281 "$PWD/rb" "$THREADS" > rb.log
  python3 "$MODEL/compare_return.py" out/ret-base-1.tsv rb "$MODEL/untouched-subset.tsv" > out/return-rows-subset.out
fi
echo "== 5 交回值不同的 6 行：调试输出里改了哪个字段（调试输出约 1.2 GB，汇总之后删掉）"
OPUS_DRIVE_DEBUG_DIR=$PWD/dbg OPUS_DRIVE_OUT=$PWD/dbg-base.tsv OPUS_DRIVE_LABEL=base nice -n 19 $CAP 256M bin/ret_base --test-threads 1 > /dev/null 2>&1
grep -P '^(49|110|155|162|174|252)\t' "$MODEL/untouched-s4.tsv" > hidden.tsv
OPUS_DRIVE_DEBUG_DIR=$PWD/dbg bash "$MODEL/run_return_batch.sh" "$PWD/ws_s4" hidden.tsv 1 281 "$PWD/rb-debug" "$THREADS" > rb-debug.log
python3 "$MODEL/hidden_diffs.py" dbg 49 110 155 161 162 174 252 > out/hidden-diffs.out
rm -rf dbg

echo "== 6 与模型目录 out/ 比（没有输出就是逐字节相同；images-v0-vs-product.diff 应为空）"
test ! -s out/images-v0-vs-product.diff || echo "DIFFERS images-v0 与入库产物 s2-images-o3.out"
for f in out/*; do
  b=$(basename "$f")
  [ "$b" = images-v0-vs-product.diff ] && continue
  cmp "$f" "$MODEL/out/$b" || echo "DIFFERS $b"
done
