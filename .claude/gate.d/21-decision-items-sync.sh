#!/usr/bin/env bash
# gate-stage: 决策分项清单与正文同步
#
# `decisions.md` 里的「分项清单」是各决策正文的**投影**，不是第二处权威记录。
# 手抄一份就会漂，而漂了没有任何东西会发现——本阶段拿生成器的输出与索引页逐字比对。
#
# 为什么判红而不是提醒：`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条
# 「矛盾比空白更糟」——检索不会把两条都端出来，它会挑一条，而且不告诉你它挑了。
#
#   bash .claude/gate.d/21-decision-items-sync.sh          只比对
#   bash .claude/gate.d/21-decision-items-sync.sh --write  重新生成并写回
set -uo pipefail
cd "${1:-$(dirname "$0")/../..}" 2>/dev/null || true
[[ "${1:-}" == "--write" ]] && { cd "$(dirname "$0")/../.." || exit 2; WRITE=1; } || WRITE=0
IDX=.claude/kb/decisions.md
GEN=.claude/scripts/gen-decision-items.py
S='<!-- gen:decision-items:start -->'
E='<!-- gen:decision-items:end -->'

[[ -f "$IDX" && -f "$GEN" ]] || { echo "  ! 找不到 $IDX 或 $GEN，本阶段跳过"; exit 0; }
grep -qF "$S" "$IDX" || { echo "  ✗ $IDX 里没有生成块标记 $S"; echo "     → 加回标记，或跑 --write 重建"; exit 1; }

want="$(python3 "$GEN")" || { echo "  ✗ 生成器跑不起来"; exit 1; }
got="$(awk -v s="$S" -v e="$E" 'index($0,s){f=1;next} index($0,e){f=0} f' "$IDX")"

if [[ "$WRITE" == "1" ]]; then
  python3 - "$IDX" "$S" "$E" <<'PY'
import sys, subprocess
idx, s, e = sys.argv[1:4]
body = open(idx, encoding='utf-8').read()
new = subprocess.run(['python3', '.claude/scripts/gen-decision-items.py'],
                     capture_output=True, text=True).stdout.rstrip()
a = body.index(s) + len(s)
b = body.index(e)
open(idx, 'w', encoding='utf-8').write(body[:a] + "\n" + new + "\n" + body[b:])
PY
  echo "  ✓ 已重新生成并写回 $IDX"
  exit 0
fi

if [[ "$want" == "$got" ]]; then
  n=$(printf '%s\n' "$want" | grep -c '^  - ' || true)
  echo "  ✓ 决策分项清单与正文同步（$n 个分项）"
  exit 0
fi
echo "  ✗ 决策分项清单与正文不同步"
diff <(printf '%s\n' "$got") <(printf '%s\n' "$want") | head -20 | sed 's/^/     /'
echo "     → 权威记录是各决策正文。改完正文跑： bash .claude/gate.d/21-decision-items-sync.sh --write"
exit 1
