#!/usr/bin/env bash
# gate-stage: 决策分项清单与正文同步
#
# `decisions.md` 里的「分项清单」和索引表的「状态」列，都是各决策正文的**投影**，
# 不是第二处权威记录。手抄一份就会漂，而漂了没有任何东西会发现——
# 本阶段拿生成器的输出与索引页逐字比对，两处都比。
#
# 「状态」列写的是**分项计数**（`已定 6 项 / 未定 0 项`）：一条决策有几个分项、
# 其中几个还没定，看一眼就有数。三态词（已定 / 半定 / 待定）的权威记录是各正文首行
# `## D<n> 简称 —— 状态`，索引列不再抄它一遍；没有分项的决策那一格写「无分项 · 整条已定」。
#
# 为什么判红而不是提醒：`.claude/singlefs-ai-sop/rules/kb-discipline.md` 第 4 条
# 「矛盾比空白更糟」——检索不会把两条都端出来，它会挑一条，而且不告诉你它挑了。
#
# ⚠️ **计数对不对，另有一条不经过生成器的检查**：20-kb-shape 第 7 段
# （`.claude/gate.d/lib-index-vs-body.py`）直接去数正文两节里的分项。
# 只有本阶段的话，生成器自己数错时索引页会跟着一起错，而两边仍然逐字相同。
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

gen_err="$(mktemp)"
cells_f="$(mktemp)"
# 生成器自己会说清哪一节取不到编号项、下一步怎么办 —— 吞掉 stderr 等于把那条 howto 扔了
want="$(python3 "$GEN" 2>"$gen_err")" || {
  echo "  ✗ 生成器跑不起来"
  sed 's/^/   /' "$gen_err"
  echo "     → 怎么办：按上面这段报错改 $GEN 或它读的那几份 kb；"
  echo "               生成器跑不起来时这一阶段什么都没查，不是通过。"
  rm -f "$gen_err" "$cells_f"; exit 1; }
python3 "$GEN" --status-cells >"$cells_f" 2>"$gen_err" || {
  echo "  ✗ 生成器的 --status-cells 跑不起来"
  sed 's/^/   /' "$gen_err"
  echo "     → 怎么办：同上——状态列没算出来时，这一半什么都没查，不是通过。"
  rm -f "$gen_err" "$cells_f"; exit 1; }
rm -f "$gen_err"
got="$(awk -v s="$S" -v e="$E" 'index($0,s){f=1;next} index($0,e){f=0} f' "$IDX")"

fail=0

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
fi

# ── 索引表「状态」列 ────────────────────────────────────────────
# 表格的形状是 `| D<n>（简称） | 状态 | 结论（简报） | 正文 |`，本段只碰第 2 格。
# 整行重写会把别的会话刚写进结论列的东西悄悄抹掉（并发会话共写一个仓）。
python3 - "$IDX" "$([[ $WRITE == 1 ]] && echo write || echo check)" "$cells_f" <<'PY'
import re, sys

idx_path, mode, cells_path = sys.argv[1:4]
text = open(idx_path, encoding='utf-8').read()
want = dict(l.split('\t', 1) for l in
            open(cells_path, encoding='utf-8').read().splitlines() if l.strip())

bad, n, wrote = [], 0, 0
for num, cell in want.items():
    pat = re.compile(r'^(\|\s*%s（[^|]*）\s*\|)([^|]*)\|' % re.escape(num), re.M)
    m = pat.search(text)
    if not m:
        bad.append(f"{num} 有正文，而决策索引表里没有它的行")
        continue
    n += 1
    if m.group(2).strip() == cell:
        continue
    if mode == 'write':
        text = text[:m.start(2)] + f" {cell} " + text[m.end(2):]
        wrote += 1
    else:
        bad.append(f"{num} 状态列写着「{m.group(2).strip()}」，正文投影是「{cell}」")

# 反过来也要查：索引表里多出一行（决策正文已经删了/改名了），上面的循环够不着它。
for row in re.findall(r'^\|\s*(D\d+)（', text, re.M):
    if row not in want:
        bad.append(f"{row} 在决策索引表里有行，而 decisions/ 下没有它的正文")

if mode == 'write':
    open(idx_path, 'w', encoding='utf-8').write(text)

if bad:
    print(f"  ✗ 决策索引表的状态列与正文不同步（{len(bad)} 处）")   # gate-lint:summary
    for b in bad:
        print("     " + b)                                          # gate-lint:detail
    print("     → 怎么办：权威记录是各决策正文的「### 已定项」/「### 未定项」两节。")
    print("       改完正文跑： bash .claude/gate.d/21-decision-items-sync.sh --write")
    sys.exit(1)

print(f"  ✓ 决策索引表状态列与正文同步（{n} 条决策"
      + (f"，写回 {wrote} 行）" if mode == 'write' else "）"))
PY
[[ $? -ne 0 ]] && fail=1
rm -f "$cells_f"

if [[ "$WRITE" == "1" ]]; then
  echo "  ✓ 已重新生成并写回 $IDX"
  exit $fail
fi

if [[ "$want" == "$got" ]]; then
  n=$(printf '%s\n' "$want" | grep -c '^  - ' || true)
  echo "  ✓ 决策分项清单与正文同步（$n 个分项）"
  exit $fail
fi
echo "  ✗ 决策分项清单与正文不同步"
diff <(printf '%s\n' "$got") <(printf '%s\n' "$want") | head -20 | sed 's/^/     /'
echo "     → 权威记录是各决策正文。改完正文跑： bash .claude/gate.d/21-decision-items-sync.sh --write"
exit 1
