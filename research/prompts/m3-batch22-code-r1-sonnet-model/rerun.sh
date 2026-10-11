#!/usr/bin/env bash
# 正推腿 m3-batch22-code-r1 的探针重跑。
# 用法：rerun.sh <空目录> [baseline|0..8|summary …]   不给后面的参数 = baseline、0..8 逐个跑完再 summary
#   环境：SINGLEFS_REPO（仓路径，取脚本、cargo 配置与快照 sha256 清单）；FROZEN_TREE（冻结副本，默认 /tmp/claude-1000/m3-batch22-code-r1/tree）
#   编译 6G、跑 4G、线程取环境变量 THREADS（默认 10），都经 scripts/hosts/run-with-memory-cap.sh；第一次调用把冻结副本拷进 <目录>/work 并核 12 份 sha256。
#   一项探针一次调用在整机负载高时要两分多钟，前台一次跑不完几项时分几次给序号，最后单独给 summary。
# 比对：summary.txt 与本目录的 expected-summary.txt 逐字相同（run-one.sh 已把并行测试的行去了耗时、失败测试名排了序）。
set -uo pipefail
here="$(cd "$(dirname "$0")" && pwd)"
: "${SINGLEFS_REPO:?要设 SINGLEFS_REPO}"
dir="$1"; shift
frozen="${FROZEN_TREE:-/tmp/claude-1000/m3-batch22-code-r1/tree}"
mkdir -p "$dir/out"
if [ ! -d "$dir/work" ]; then
  mkdir "$dir/work" && cp -r "$frozen/." "$dir/work/" && cp -r "$SINGLEFS_REPO/.cargo" "$dir/work/.cargo" || exit 3
  (cd "$dir/work" && sha256sum -c "$SINGLEFS_REPO/research/prompts/m3-batch22-code-r1-snapshot/crates-sha256.txt" | sed 's/^/SHA256 /' | awk '{c[$NF]++} END {for (k in c) print "SHA256_RESULT", k, c[k]}')
fi
export CARGO_TARGET_DIR="$dir/target"
steps=("$@")
if [ ${#steps[@]} -eq 0 ]; then steps=(baseline 0 1 2 3 4 5 6 7 8 summary); fi
for step in "${steps[@]}"; do
  if [ "$step" = "summary" ]; then
    {
      echo "walk.rs sha256 $(sha256sum "$dir/work/crates/singlefs-checker/src/walk.rs.pristine" 2>/dev/null | cut -d' ' -f1)"
      for f in baseline 0 1 2 3 4 5 6 7 8; do
        if [ -f "$dir/out/$f.txt" ]; then echo "== $f"; cat "$dir/out/$f.txt"; else echo "== $f MISSING"; fi
      done
    } > "$dir/summary.txt"
    if cmp -s "$dir/summary.txt" "$here/expected-summary.txt"; then echo "SUMMARY_MATCHES lines=$(wc -l < "$dir/summary.txt")"; else echo "SUMMARY_DIFFERS"; diff "$dir/summary.txt" "$here/expected-summary.txt" | head -20; fi
  else
    bash "$here/run-one.sh" "$dir/work" "$step" "$dir/out/$step.txt"
    echo "step $step exit=$?"; cat "$dir/out/$step.txt"
  fi
done
