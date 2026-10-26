#!/usr/bin/env bash
# peer_mutants.sh：在仓副本树根（第二台上 multi-host-run.sh --peer 拷过去的那棵）里逐条施加变异、跑点名的测试、还原。
# 外面已经套了 run-with-memory-cap.sh，这里不再套（mutate_run.py --direct）。输入：model/mutations/*.tsv；输出：打到标准输出，日志在 mutout/。
# 用法：bash model/tools/peer_mutants.sh <变异表名> <行号逗号表>
set -u
root=$(pwd)
rm -rf "$root/.pristine"; mkdir -p "$root/.pristine"; cp -a "$root/crates" "$root/.pristine/"
find "$root/crates" -type f -exec touch {} +
echo "name=peer_mutants tsv=$1 rows=$2 nproc=$(nproc)"
python3 -I "$root/model/tools/mutate_run.py" --repo "$root" --pristine "$root/.pristine" --work "$root" \
  --target "$root/target" --out "$root/mutout" --tsv "$root/model/mutations/$1" --rows "$2" --direct --tag "$1."
echo "name=peer_mutants done"
