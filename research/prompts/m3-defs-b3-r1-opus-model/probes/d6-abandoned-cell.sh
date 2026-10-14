#!/usr/bin/env bash
# D6 探针二：门禁 doc-process-records 的 abandoned-rounds 格那段判法（冻结副本 .claude/gate.d/doc-process-records.sh 第 686–865 行的 python）
# 抽出来，在一份不是 git 仓的样本根上跑：一轮有正文、写了合格的 halted、没登记 abandoned-rounds.tsv，看它判红还是绿。
# 用法：bash d6-abandoned-cell.sh <冻结副本> <新的工作目录>
set -euo pipefail
FROZEN="$1"; WORK="$2"
awk 'NR>=687 && NR<=865' "$FROZEN/.claude/gate.d/doc-process-records.sh" > "$WORK/abandoned-cell.py"
ROOT="$WORK/d6-cell-root"
mkdir -p "$ROOT/research/prompts" "$ROOT/records" "$ROOT/.claude/kb"
printf '# r-stop-r1 正文\n' > "$ROOT/research/prompts/_r-stop-r1-body.md"
printf '用户说：这一轮先不判了，停掉。\n' > "$ROOT/records/legit.md"
printf '出处：records/legit.md「这一轮先不判了，停掉」\n' > "$ROOT/research/prompts/r-stop-r1-halted.md"
printf '# 表头\n' > "$ROOT/research/prompts/abandoned-rounds.tsv"
run_cell() {
  local rc
  if (cd "$ROOT" && python3 "$WORK/abandoned-cell.py") > "$WORK/d6-cell-$1.out" 2>&1; then rc=0; else rc=$?; fi
  echo "name=d6-cell case=$1 rc=$rc"
  grep -m1 -F 'r-stop-r1' "$WORK/d6-cell-$1.out" || true
}
run_cell halted-without-tsv-row
printf 'r-stop-r1\t2026-10-12\t用户叫停\t不写判决\n' >> "$ROOT/research/prompts/abandoned-rounds.tsv"
run_cell halted-and-tsv-row
