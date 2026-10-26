#!/usr/bin/env bash
# 门禁 doc-experiments 的 repro-command 格那个循环照抄（只读、不起门禁阶段）：此刻的 REPRO_RE 与把其中 experiments/replay 换成 experiment-apparatus/replay 的一版，各有几页判「点了产物却没写怎么跑」。
# 用法：bash repro_command_count.sh <仓根>
set -uo pipefail
cd "$1"
P='research/results/[A-Za-z0-9._-]+\.out(\.gz)?'
now="$(grep -oF "REPRO_RE='cargo run --release --bin |scripts/(experiments/replay|devices/vm-bench)\.sh'" .claude/gate.d/doc-experiments.sh | wc -l)"
R_now='cargo run --release --bin |scripts/(experiments/replay|devices/vm-bench)\.sh'
R_new='cargo run --release --bin |scripts/(experiment-apparatus/replay|devices/vm-bench)\.sh'
pointing=0 bad_now=0 bad_new=0
for f in .claude/kb/experiments/*.md; do
  grep -qE "$P" "$f" || continue
  pointing=$((pointing + 1))
  grep -qF '原始输出未留存' "$f" && continue
  grep -qE "$R_now" "$f" || bad_now=$((bad_now + 1))
  grep -qE "$R_new" "$f" || bad_new=$((bad_new + 1))
done
echo "name=repro_command regex_line_present=$now pages_pointing_products=$pointing bad_with_current_regex=$bad_now bad_with_new_prefix_regex=$bad_new"
