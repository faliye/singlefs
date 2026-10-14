#!/usr/bin/env bash
# V8 E171 下一次修订的入口：把修订 11 的多机入口拷成修订 12，只把 # long-run-hosts: 一行改成登记值的说法（正文照旧调同一份现算的编排），
# 再拷一份原样不改（老实的修订 12）。判：lib_long_run.py script（detached-run.sh start / long-run-check 用的同一判法）两版各判两份，
# exempt-check 与 heavy-thread-defaults 格（run_htd_cell.py）在零轮那一版上判。用法：v8_rev12.sh <WORK>（WORK 里要有 b6/repo，可有 b3/repo）
set -u
work=$1; here=$(cd "$(dirname "$0")" && pwd)
R="$work/rev12/repo"; rm -rf "$work/rev12"; mkdir -p "$work/rev12"
rsync -a --exclude research/prompts --exclude research/results "$work/b6/repo/" "$R/"
D="$R/scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes"
cp "$here/../feeds/v8/run_segment_one_revision_12_multi_host.sh" "$D/"
cp "$D/run_segment_one_revision_11_multi_host.sh" "$D/run_segment_one_revision_12_honest_multi_host.sh"
for v in b6 b3; do
  [[ -d "$work/$v/repo" ]] || continue
  for s in run_segment_one_revision_12_multi_host.sh run_segment_one_revision_12_honest_multi_host.sh; do
    (cd "$R" && python3 "$work/$v/repo/.claude/hooks/lib_long_run.py" script "$R" -- bash "$D/$s" >/dev/null 2>&1); echo "$v lib_long_run script $s rc=$?"
  done
done
(cd "$R" && python3 .claude/hooks/lib_long_run.py exempt-check "$R" --base-table "$R/.claude/long-run-live-hosts-exempt"); echo "exempt-check rc=$?"
python3 "$here/run_htd_cell.py" "$work/b6/repo" "$R" | tail -n 1; echo "heavy-thread-defaults rc=${PIPESTATUS[0]}"
# detached-run.sh 的 long-run-check（主 agent 起长活前、子 agent 交回前跑的那一道）在零轮那一版的拷贝里各判两份
for s in run_segment_one_revision_12_multi_host.sh run_segment_one_revision_12_honest_multi_host.sh; do
  (cd "$R" && bash scripts/hosts/detached-run.sh long-run-check -- bash "scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/$s" >/dev/null 2>&1); echo "b6 detached-run long-run-check $s rc=$?"
done
