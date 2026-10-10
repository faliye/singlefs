#!/usr/bin/env bash
# 正推腿 m3-batch26-code-r1-sonnet 的复跑：在冻结副本的拷贝里编 harness、跑第 26 批 6 条坏镜像、逐行施加 mutations.tsv 里第 24 批 K 那两行与第 26 批 CPU 那六行、再施加自己加的额外变异。
# 环境变量：SINGLEFS_REPO（仓根，要用它的 research/scripts）、FROZEN_TREE（冻结副本，缺省 /tmp/claude-1000/m3-batch26-code-r1/tree）、
#   WORK（工作目录，缺省 /tmp/claude-1000/m3-batch26-code-r1-sonnet-rerun；编译目录在里面）、STAGES（缺省「build base paper extra」，可拆开分次跑，共用同一个 WORK）。
# 编译与跑都经 run-with-memory-cap.sh（编译 6G、跑 4G）与 capped.sh 10。GPU 不跑。
# 输出：$WORK/summary.txt（行号 TAB 命中次数 TAB 退出码 TAB 红的测试数，按行号排序）、$WORK/base.txt（基线 6 条测试的结果行，排序）。
set -euo pipefail
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
HERE="$(cd "$(dirname "$0")" && pwd)"
FROZEN_TREE="${FROZEN_TREE:-/tmp/claude-1000/m3-batch26-code-r1/tree}"
WORK="${WORK:-/tmp/claude-1000/m3-batch26-code-r1-sonnet-rerun}"
STAGES="${STAGES:-build base paper extra}"
export CARGO_TARGET_DIR="$WORK/target" RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="派发提示要求跑上限显式写 4G"
mkdir -p "$WORK/logs"
[ -d "$WORK/tree" ] || cp -r "$FROZEN_TREE" "$WORK/tree"
M="$WORK/tree/Cargo.toml"
cap() { bash "$SINGLEFS_REPO/research/scripts/run-with-memory-cap.sh" "$1" bash "$SINGLEFS_REPO/research/scripts/capped.sh" 10 "${@:2}"; }
for stage in $STAGES; do
  case "$stage" in
    build) cap 6G cargo test --offline --manifest-path "$M" -p singlefs-harness --test checker_known_bad_images --no-run > "$WORK/logs/build.log" 2>&1 ;;
    base)  cap 4G cargo test --offline --manifest-path "$M" -p singlefs-harness --test checker_known_bad_images -- stale_copy_of_the_child child_that_does_not_read_back > "$WORK/logs/base.log" 2>&1
           grep -E '^test |^test result' "$WORK/logs/base.log" | sed 's/finished in .*//' | sort > "$WORK/base.txt" ;;
    paper) LOGDIR="$WORK/logs" python3 "$HERE/mutate_rows.py" "$WORK/tree" 2883 2884 2930 2931 2932 2933 2934 2935 > "$WORK/summary-paper.txt" ;;
    extra) LOGDIR="$WORK/logs" MUTATIONS_TSV="$HERE/extra-rows.tsv" python3 "$HERE/mutate_rows.py" "$WORK/tree" 1 2 3 4 5 6 7 8 9 | sed 's/^/extra-/' > "$WORK/summary-extra.txt" ;;
    *) echo "不认得的阶段 $stage" >&2; exit 2 ;;
  esac
done
cat "$WORK"/summary-paper.txt "$WORK"/summary-extra.txt 2>/dev/null | sort > "$WORK/summary.txt" || true
echo "done: $WORK/summary.txt"
