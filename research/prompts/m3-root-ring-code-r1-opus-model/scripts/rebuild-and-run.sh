#!/bin/bash
# m3-root-ring-code-r1 攻方腿模型复跑：把冻结副本拷进 <草稿目录>/tree、打 core-and-checker.diff（patch -p1，只加只读访问器、一个测试入口、
# 一个改法开关 SINGLEFS_PROTO_RAISE_KEEPS_UNREADABLE，开关不设时行为与冻结副本相同）、放进原型 crate rr-attack-proto、编译，再按批跑。
# 用法：rebuild-and-run.sh <草稿目录> <批…>
#   批：build | k2e | k2e-enum | k2b-1 | k2b-2 | k2b-more-1 | k2b-more-2 | k2b-fix-1 | k2b-fix-2 | k1k3k4-1 | k4-enum | b04 | b13 | b15
#   每批的件数与线程数合计不超过 15（批里每件经 run.sh：run-with-memory-cap.sh + capped.sh）。
#   b04、b13、b15 用 1 MiB journal 环、256 MiB 盘（与草稿里跑的相同，由这里的环境变量给）。
set -euo pipefail
scratch=$1; shift
tree=${TREE:-/tmp/claude-1000/m3-root-ring-code-r1/tree}
model=$(cd "$(dirname "$0")/.." && pwd)
mkdir -p "$scratch/out" "$scratch/specs"
if [ ! -d "$scratch/tree" ]; then
  rsync -a --exclude target --exclude .git "$tree/" "$scratch/tree/"
  (cd "$scratch/tree" && patch -p1 -s < "$model/core-and-checker.diff")
  mkdir -p "$scratch/tree/crates/rr-attack-proto"
  cp -r "$model/rr-attack-proto/." "$scratch/tree/crates/rr-attack-proto/"
fi
for f in build.sh run.sh par.sh fz-batch.sh fz-summary.py; do cp "$model/scripts/$f" "$scratch/$f.tmp" && mv "$scratch/$f.tmp" "$scratch/$f"; done
cp "$model"/scripts/specs/*.txt "$scratch/specs/"
cd "$scratch"
./build.sh
for what in "$@"; do
  case "$what" in
    build) ;;
    b04|b13|b15) PROTO_JOURNAL_BYTES=1048576 PROTO_IMAGE_BYTES=268435456 ./fz-batch.sh "${what#b}" "specs/$what.txt" ;;
    k2e) ./par.sh 21 specs/k2e.txt ;;
    k2e-enum) ./par.sh 22 specs/k2e-enum.txt ;;
    k2b-1) ./par.sh 23 specs/k2b-1.txt ;;
    k2b-2) ./par.sh 24 specs/k2b-2.txt ;;
    k2b-more-1) ./par.sh 25 specs/k2b-more-1.txt ;;
    k2b-more-2) ./par.sh 26 specs/k2b-more-2.txt ;;
    k2b-fix-1) ./par.sh 27 specs/k2b-fix-1.txt ;;
    k2b-fix-2) ./par.sh 28 specs/k2b-fix-2.txt ;;
    k1k3k4-1) ./par.sh 29 specs/k1k3k4-1.txt ;;
    k4-enum) ./par.sh 30 specs/k4-enum.txt ;;
    *) echo "不认识的批：$what" >&2; exit 2 ;;
  esac
done
