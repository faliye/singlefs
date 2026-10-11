#!/usr/bin/env bash
# c673-clauses-r1 云端攻方的模型复跑。整条经内存包装跑（工作目录随意）：
#   bash scripts/run-with-memory-cap.sh 8G bash research/prompts/c673-clauses-r1-opus-model/tools/rerun.sh <新的草稿目录>
# 做的事：把冻结副本（SRC_TREE，默认 /tmp/claude-1000/c673-clauses-r1/tree；按快照清单核 sha256）拷进草稿目录，
# 加仓根的 Cargo.toml / Cargo.lock / .cargo/config.toml 与原型用例，跑原型（out/probe.out 的来源），再逐条跑变异表（out/mutations-*.out 的来源）。
# 只跑 singlefs-harness 的测试目标；不跑 checker 档。cargo 线程上限 8。
set -euo pipefail
MODEL="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="${SINGLEFS_REPO:-$(cd "$MODEL/../../.." && pwd)}"
SRC_TREE="${SRC_TREE:-/tmp/claude-1000/c673-clauses-r1/tree}"
WORK="${1:?要给一个新的草稿目录}"
mkdir -p "$WORK/repo/.cargo"
rsync -a --exclude target --exclude .git "$SRC_TREE/crates" "$WORK/repo/"
(cd "$WORK/repo" && sha256sum --quiet -c "$REPO_ROOT/research/prompts/c673-clauses-r1-snapshot/crates-sha256.txt")
cp "$REPO_ROOT/Cargo.toml" "$REPO_ROOT/Cargo.lock" "$WORK/repo/"
cp "$REPO_ROOT/.cargo/config.toml" "$WORK/repo/.cargo/"
cp "$MODEL/probe/c673_opus_probe.rs" "$WORK/repo/crates/singlefs-harness/tests/"
export CARGO_TARGET_DIR="$WORK/target"
(cd "$WORK/repo" && nice -n 19 bash "$REPO_ROOT/scripts/capped.sh" 8 \
  cargo test -p singlefs-harness --test c673_opus_probe -- --nocapture --test-threads=1) > "$WORK/probe.log" 2>&1
grep -o "name=.*" "$WORK/probe.log" > "$WORK/probe.out"
SINGLEFS_REPO="$REPO_ROOT" nice -n 19 python3 "$MODEL/tools/mutate_and_run.py" "$WORK/repo" "$MODEL/mutations.tsv" "$WORK/mut" > "$WORK/mutations.out" 2>&1
echo "probe: $WORK/probe.out（与 $MODEL/out/probe.out 逐行比）"
echo "mutations: $WORK/mutations.out（与 $MODEL/out/mutations-*.out 里的 name=mutation_result 行比）"
