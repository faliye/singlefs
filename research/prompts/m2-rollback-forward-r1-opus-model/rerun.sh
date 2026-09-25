#!/usr/bin/env bash
# m2-rollback-forward-r1 云端攻方腿原型的复跑（副本上的数，不是入库装置上的数）。
#   bash rerun.sh <冻结副本的 tree 目录> <草稿目录> [full]
# 冻结副本：/tmp/claude-1000/m2-rollback-forward-r1/tree（sha256 清单在 research/prompts/m2-rollback-forward-r1-snapshot/crates-sha256.txt）。
# libtest 在 --nocapture 下把「test 名 ...」印在同一行的开头，所以按「RBF 」切行首。
# 不带 full：全部用例（崩溃枚举只展开 10 写以下的段）；带 full：另跑两条全量崩溃枚举（卸载那一串 327690 个状态、回退那条流 524301 个状态，各约 3–8 分钟）。
set -euo pipefail
TREE=$1; WORK=$2; MODE=${3:-fast}
HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$HERE/../../.." && pwd)"
S="$REPO_ROOT/research/scripts"
mkdir -p "$WORK"
rsync -a --delete --exclude target --exclude .git "$TREE/" "$WORK/repo/"
cp "$REPO_ROOT/Cargo.toml" "$REPO_ROOT/Cargo.lock" "$WORK/repo/"
(cd "$WORK/repo" && patch -p0 < "$HERE/prototype.patch")
cp "$HERE/rbf_attack.rs" "$WORK/repo/crates/singlefs-harness/tests/rbf_attack.rs"
cd "$WORK/repo"
run() { nice -n 19 bash "$S/run-with-memory-cap.sh" 20G bash "$S/capped.sh" 12 cargo test --release -p singlefs-harness --test rbf_attack "$@"; }
run -- --nocapture --test-threads 1 2>&1 | grep -E 'RBF |test result' | sed -E 's/^.*(RBF )/\1/' > "$WORK/rerun-fast.out"
if [ "$MODE" = full ]; then
  RBF_FULL=1 RBF_NOPROBE=1 run -- --nocapture --exact t5a_crash_points_of_the_unmount_chain 2>&1 | grep -E 'RBF |test result' | sed -E 's/^.*(RBF )/\1/' > "$WORK/rerun-full-unmount.out"
  RBF_FULL=1 RBF_PROBE_EVERY=200 run -- --nocapture --exact t4_crash_points_of_the_forward_rollback_stream 2>&1 | grep -E 'RBF |test result' | sed -E 's/^.*(RBF )/\1/' > "$WORK/rerun-full-rollback.out"
fi
echo "outputs in $WORK"
