#!/usr/bin/env bash
# m3-w3-r3 正推腿（Sonnet）的复跑。用法：SINGLEFS_REPO=<仓根> bash rerun.sh <工作目录> <段>
# 路径只从环境变量与参数取：仓根 SINGLEFS_REPO；冻结树 FROZEN_TREE（默认 ~/.local/share/singlefs/work/m3-w3-r1/tree，只有 cost-check 段读它）。
# 每段一条前台命令（负载低时各在 4 分钟以内）；编译 6G、跑 4G，都经 research/scripts/run-with-memory-cap.sh；只跑 harness 档，不碰 crates/ 的编译。
# 段：
#   build-model       编抽象时间线模型（不依赖 crates/）
#   model-main        main 组 0..5（P、V1、V2 三臂 × 清扫阈值从不 / 0；环 4、长 7）
#   model-defense     defense 组 0..5（V1 环 4 / 6 / 8 / 12、V3、P 环 8）
#   model-mutation    mutation 组 0..5（P、V1 × 老三种变异 A / B / C）与 ring6 组 0..1
#   model-mutation2-p / model-mutation2-v1   mutation2 组（P 臂 0..4 / V1 臂 5..9：新五种变异 H / Z / R / D / Mh）
#   model-k2          k2 组 0..2（Y1 取法二，P 的同一组历史）
#   model-blind       blind 组 0..2（P、取法二、控制臂只取环；字母表含 rollback-blind，长 7）
#   model-tolring     P 臂、环 8、字母表 4 个、长 11（历史长过环）：「容许跳升」的几种读法在被抛弃的根被挤出环之后还误不误报
#   model-deep        深一层（长 8、字母表 8 个）：main 组 0（P）与 2（V1）
#   probe-build       拷冻结树进 <工作目录>/probe/tree、核清单、把 probe/*.rs 两段探针接在 code_two_tree.rs 与 container_tree.rs 末尾（只在副本里）、编 singlefs-core 的 lib 测试（不跑 checker 档）
#   probe-run         跑探针（真的 plan_the_tree_after_this_publish 与 after_a_container_is_opened_at_the_right_end）：out/probe-small-real.out、out/probe-shape.out
#   probe-compare     cost/probe_compare.py：Python 复刻的规划器与探针里真规划器逐行比；--red 先弄坏复刻一处，必须红
#   cost              算术与计数小模型（python3）：记账树脏节点、容器树脏节点、崩溃状态闭式、字节；吃 out/probe-shape.out 对容器树形状
#   cost-check        把算术里用到的格式常量与冻结树里的源码行逐个对一遍（读 FROZEN_TREE）
#   compare-r2        老候选的列与 r2 辩方腿的 expected 逐数比（expected 在仓里 research/prompts/m3-w3-r2-sonnet-model/expected/）
#   compare-r2opus    r2 攻方腿的模型输出与这一轮同一组逐格比（五个候选字段 + 头部几个总数）
#   compare           把 <工作目录>/out 与本目录 expected/ 比（模型输出去掉 W3TRACE 行排序后比；cost 输出逐字节比）
#   expect            （只在生成 expected 时用）把 <工作目录>/out 整理成 expected/
set -euo pipefail
export PYTHONDONTWRITEBYTECODE=1
W=${1:?工作目录}; PART=${2:?段}
R=${SINGLEFS_REPO:?设 SINGLEFS_REPO 为仓根}
M=$(cd "$(dirname "$0")" && pwd)
FROZEN=${FROZEN_TREE:-$HOME/.local/share/singlefs/work/m3-w3-r1/tree}
CAP="bash $R/research/scripts/run-with-memory-cap.sh"
THREADS="bash $R/research/scripts/capped.sh 6"
mkdir -p "$W/out"
export SINGLEFS_REPO
BIN=$W/target-model/release/w3r2-timeline-model
model() { nice -n 19 $CAP 4G "$BIN" "$@"; }
# 并行跑一组模型：model_group <组> <起> <止> <文件前缀>；退出码写 out/<前缀>-<i>.exit，等齐后有非 0 退 1
model_group() {
  local group=$1 lo=$2 hi=$3 prefix=$4 i bad=0
  for i in $(seq "$lo" "$hi"); do
    rm -f "$W/out/$prefix-$i.exit"
    { model "$group" "$i" > "$W/out/$prefix-$i.out" 2> "$W/out/$prefix-$i.err"; echo "$?" > "$W/out/$prefix-$i.exit"; } &
  done
  wait
  for i in $(seq "$lo" "$hi"); do [ "$(cat "$W/out/$prefix-$i.exit" 2>/dev/null)" = 0 ] || bad=1; done
  echo "组 $group $lo..$hi 有非 0=$bad"; return $bad
}
case "$PART" in
  build-model)
    rsync -a --exclude target "$M/timeline-model/" "$W/timeline-model/"
    (cd "$W/timeline-model" && CARGO_TARGET_DIR=$W/target-model nice -n 19 $THREADS $CAP 6G cargo build --release --offline 2>>"$W/build.log")
    sha256sum "$BIN" ;;
  model-main) model_group main 0 5 model-main ;;
  model-defense) model_group defense 0 5 model-defense ;;
  model-mutation) { model_group mutation 0 5 model-mutation & model_group ring6 0 1 model-ring6 & wait; } ;;
  model-mutation2-p) model_group mutation2 0 4 model-mutation2 ;;
  model-mutation2-v1) model_group mutation2 5 9 model-mutation2 ;;
  model-k2) model_group k2 0 2 model-k2 ;;
  model-blind)
    export W3_OPS=create,delete,rollback1,rollback2,crash-hide,mount,empty,rollback-blind W3_LONGEST=7
    model_group blind 0 2 model-blind ;;
  model-tolring)
    export W3_OPS=create,rollback1,crash-hide,empty W3_RING=8 W3_LONGEST=11
    model_group main 0 0 model-tolring ;;
  model-deep)
    export W3_OPS=create,delete,rollback1,rollback2,raiseF,mount,crash-hide,switch-redo W3_LONGEST=8
    model_group main 0 0 model-deep & model_group main 2 2 model-deep & wait ;;
  probe-build)
    [ ! -e "$W/probe" ] || { echo "工作目录里已有 probe：$W/probe" >&2; exit 2; }
    mkdir -p "$W/probe"
    rsync -a --exclude target --exclude .git "$FROZEN/" "$W/probe/tree/"
    (cd "$W/probe/tree" && sha256sum -c --quiet "$R/research/prompts/m3-w3-r1-snapshot/tree-sha256.txt" && echo "冻结树副本对上清单")
    cat "$M/probe/code_two_tree_probe.rs" >> "$W/probe/tree/crates/singlefs-core/src/code_two_tree.rs"
    cat "$M/probe/container_tree_probe.rs" >> "$W/probe/tree/crates/singlefs-core/src/container_tree.rs"
    (cd "$W/probe/tree" && CARGO_TARGET_DIR=$W/probe/target nice -n 19 $THREADS $CAP 6G cargo test --release --offline -p singlefs-core --lib w3_ --no-run --message-format=json 2>>"$W/build.log" \
      | python3 -c 'import sys,json
for l in sys.stdin:
    try: m=json.loads(l)
    except Exception: continue
    if m.get("reason")=="compiler-artifact" and m.get("target",{}).get("name")=="singlefs_core" and m.get("profile",{}).get("test") and m.get("executable"): print(m["executable"])' > "$W/probe/exe.txt")
    cat "$W/probe/exe.txt" ;;
  probe-run)
    EXE=$(tail -n 1 "$W/probe/exe.txt")
    W3_PROBE=small,real nice -n 19 $CAP 4G "$EXE" w3_cost_probe --nocapture 2> "$W/out/probe-small-real.err" | grep "^W3COST" > "$W/out/probe-small-real.out"
    W3_PROBE=bulk nice -n 19 $CAP 4G "$EXE" w3_cost_probe --nocapture 2> "$W/out/probe-bulk.err" | grep "^W3COST" > "$W/out/probe-bulk.out"
    W3_PROBE=shape nice -n 19 $CAP 4G "$EXE" w3_shape_probe --nocapture 2> "$W/out/probe-shape.err" | grep "^W3SHAPE" > "$W/out/probe-shape.out"
    grep -c '^W3COST' "$W/out/probe-small-real.out"; grep -c '^W3SHAPE' "$W/out/probe-shape.out" ;;
  probe-compare)
    python3 "$M/cost/probe_compare.py" "$W/out/probe-small-real.out" "$W/out/probe-bulk.out" | tee "$W/out/probe-compare.out"
    python3 "$M/cost/probe_compare.py" --red "$W/out/probe-small-real.out" "$W/out/probe-bulk.out" | tee "$W/out/probe-compare-red.out" || true
    grep -q '不同 [1-9]' "$W/out/probe-compare-red.out" && echo "自证：弄坏复刻之后探针对账判红" ;;
  cost) python3 "$M/cost/cost_model.py" "$W/out/probe-shape.out" > "$W/out/cost-model.out" ; tail -n 3 "$W/out/cost-model.out" ;;
  cost-check) python3 "$M/cost/check_constants.py" "$FROZEN" | tee "$W/out/cost-check.out" ;;
  compare-r2)
    O=$R/research/prompts/m3-w3-r2-sonnet-model/expected; status=0
    for p in "main-0 model-main-0" "main-1 model-main-1" "main-2 model-main-2" "main-3 model-main-3" "main-4 model-main-4" "main-5 model-main-5" \
             "defense-0 model-defense-0" "defense-1 model-defense-1" "defense-2 model-defense-2" "defense-3 model-defense-3" "defense-4 model-defense-4" "defense-5 model-defense-5" \
             "mutation-0 model-mutation-0" "mutation-1 model-mutation-1" "mutation-2 model-mutation-2" "mutation-3 model-mutation-3" "mutation-4 model-mutation-4" "mutation-5 model-mutation-5" \
             "ring6-0 model-ring6-0" "ring6-1 model-ring6-1"; do
      set -- $p
      python3 "$M/tools/compare_r2.py" "$O/model-$1.out" "$W/out/$2.out" > "$W/out/compare-r2-$2.txt" || status=1
      tail -n 1 "$W/out/compare-r2-$2.txt"
    done
    for i in 0 2; do
      python3 "$M/tools/compare_r2.py" "$O/model-deep-$i.out" "$W/out/model-deep-$i.out" > "$W/out/compare-r2-deep-$i.txt" || status=1
      tail -n 1 "$W/out/compare-r2-deep-$i.txt"
    done
    echo "compare-r2 status=$status"
    exit $status ;;
  tables) python3 "$M/tools/report_tables.py" "$W/out" > "$W/out/report-tables.out"; wc -l "$W/out/report-tables.out" ;;
  compare-r2opus) python3 "$M/tools/compare_r2opus.py" "$R/research/prompts/m3-w3-r2-opus-model/expected" "$W/out" ;;
  compare) bash "$M/compare.sh" "$M/expected" "$W/out" ;;
  expect) bash "$M/compare.sh" --make "$M/expected" "$W/out" ;;
  *) echo "不认的段：$PART" >&2; exit 2 ;;
esac
echo "段 $PART 跑完"
