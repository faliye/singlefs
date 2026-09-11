#!/usr/bin/env bash
# gate-stage: 欠账表两张登记表的行形状（已还清那张不许混进六列的欠账行）
#
# 判据：`.claude/kb/checks-owed.md` 有两张登记表——欠着的那张六列
# （编号 / 简称 / 要拦什么 / 怎么拦 / 前置 / 出处），「### 已还清」那张四列
# （编号 / 简称 / 怎么还的 / 还清日期）。一行的格数与它所在那张表的表头对不上就判红；
# 同一个编号在两张表里各登记一次也判红。只看紧挨着 `doc-lint:registry` 标记的表，
# 历史版本里别的表不管。
#
# ⚠️ 实测（2026-09-11）：C248–C273 这 26 行全是六列的欠账行，却一行接一行追加在
# 「已还清」那张四列表的末尾——几个会话往文件末尾追加时都没看它落在哪张表下面。
# 于是一条没还的欠账（C266、C272）按小节检索时挂在「已还清」标题下，
# 渲染时多出来的两格被截掉，而此前没有任何一道检查看这件事。
set -uo pipefail
ROOT="${1:-$(cd "$(dirname "$0")/../.." && pwd)}"
F="$ROOT/.claude/kb/checks-owed.md"
if [[ ! -f "$F" ]]; then
  echo "  ✗ 找不到 $F"
  echo "  → 怎么办：欠账表挪了位置就同步改这个阶段里的路径。"
  exit 1
fi
python3 - "$F" <<'PY'
import re, sys
lines = open(sys.argv[1], encoding='utf-8').read().split('\n')
tables, cur, section, marked = [], None, '（开头）', False
for i, l in enumerate(lines, 1):
    s = l.strip()
    if s.startswith('<!-- doc-lint:registry'):
        marked = True; cur = None; continue
    if l.startswith('#'):
        section = l.lstrip('#').strip(); cur = None; marked = False; continue
    if not s.startswith('|'):
        cur = None
        if s:
            marked = False
        continue
    cells = re.sub(r'\\\|', '', s).strip('|').split('|')
    if cur is None:
        cur = {'sec': section, 'width': len(cells), 'head': i, 'rows': [], 'reg': marked}
        tables.append(cur); marked = False; continue
    if re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', s):
        continue
    m = re.match(r'\|\s*(C\d+)\s*\|', s)
    if m:
        cur['rows'].append((i, m.group(1), len(cells)))
reg = [t for t in tables if t['reg'] and t['rows']]
if len(reg) < 2:
    print(f'  ✗ 只找到 {len(reg)} 张带编号行的登记表，欠着的与已还清的两张应当都在')
    print('  → 怎么办：表头或它上面的 doc-lint:registry 标记被改坏了；按文件开头的约定把两张表恢复出来。')
    sys.exit(1)
bad, seen, checked = [], {}, 0
for t in reg:
    for ln, num, n in t['rows']:
        checked += 1
        if n != t['width']:
            bad.append(f"第 {ln} 行 {num}：{n} 格，而它所在那张表（「{t['sec']}」下、表头在第 {t['head']} 行）是 {t['width']} 格")
        seen.setdefault(num, []).append(ln)
dup = {k: v for k, v in seen.items() if len(v) > 1}
if bad or dup:
    print(f'  ✗ 欠账表有 {len(bad)} 行的格数与所在表的表头对不上、{len(dup)} 个编号登记了两处：')  # gate-lint:summary
    for b in bad:
        print('      ' + b)  # gate-lint:detail
    for k, v in dup.items():
        print(f"      {k} 在第 {'、'.join(map(str, v))} 行各登记一次")  # gate-lint:detail
    print('  → 怎么办：六列的欠账行挪回欠着的那张表，接在它最后一行后面，不接在文件末尾；')
    print('            真还清了的，按「已还清」那张表的四列改写（编号 / 简称 / 怎么还的 / 还清日期）再挪过去。')
    sys.exit(1)
print(f'  ✓ 欠账表两张登记表的行形状都对（检查了 {checked} 行，{len(reg)} 张表）')
PY
