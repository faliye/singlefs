#!/usr/bin/env bash
# gate-stage: 欠账表两张登记表的行形状（已还清那张不许混进六列的欠账行，欠着那张不许留写着已还的行）
#
# 判据：`.claude/kb/checks-owed.md` 有两张登记表——欠着的那张六列
# （编号 / 简称 / 要拦什么 / 怎么拦 / 前置 / 出处），「### 已还清」那张四列
# （编号 / 简称 / 怎么还的 / 还清日期）。一行的格数与它所在那张表的表头对不上就判红；
# 同一个编号在两张表里各登记一次也判红；欠着的那张里，一行有哪一格（第 3 列起）开头写着
# 「已还」而后面不是「一半 / 这一半 / 大半」、或开头写着「销账」「作废」（前面带 ⚠️ 与日期也算），也判红——还清了就挪走，
# 只还了一部分就把开头写成还欠什么（「已还一半」「决策已定、检查仍欠」「前置已还」）。
# 只看紧挨着 `doc-lint:registry` 标记的表，历史版本里别的表不管。
#
# ⚠️ 实测（2026-09-11）：C248–C273 这 26 行全是六列的欠账行，却一行接一行追加在
# 「已还清」那张四列表的末尾——几个会话往文件末尾追加时都没看它落在哪张表下面。
# 于是一条没还的欠账（C266、C272）按小节检索时挂在「已还清」标题下，
# 渲染时多出来的两格被截掉，而此前没有任何一道检查看这件事。
#
# ⚠️ 实测（2026-09-19）：反过来的形态。C273（小节清单不认一级标题） 2026-09-11 就还清了
# （自检接进门禁 47 号），行首写着「已还（2026-09-11……）」却一直留在欠着的那张表里，
# 门禁 10 号照样把它算进「欠检查 N 条」；同一张表里另有九行开头写「已还（…）」，
# 核下来只还了决策、前置或一半，写法与已还清表同一个词，按行检索分不开。
# 2026-09-19 又查出三行（C137 / C150 / C151）第一格写「⚠️ 2026-09-07 作废……此编号不再使用」却留在欠着的那张表里：
# 同一族的 C135 / C138 当天就按同一理由挪进了已还清，这三行被算进「欠检查」的数里一个多月。
# 同一天还查出欠着的那张表中间夹着 6 个空行：Markdown 在空行处结束表格，第 87 行起约 270 行
# 渲染不成表，本阶段也只把第一段当登记表判，后面几段一行都没判过。所以编号行前面没有表头也判红。
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
headless, in_history = [], False
for i, l in enumerate(lines, 1):
    s = l.strip()
    if s.startswith('<!-- doc-lint:registry'):
        marked = True; cur = None; continue
    if l.startswith('#'):
        section = l.lstrip('#').strip(); cur = None; marked = False
        in_history = in_history or section.startswith('历史版本')
        continue
    if not s.startswith('|'):
        cur = None
        if s:
            marked = False
        continue
    cells = re.sub(r'\\\|', '', s).strip('|').split('|')
    if cur is None:
        # 一张表的第一行就是编号行，说明它前面没有表头：多半是被一个空行从上一张表断开的。
        # Markdown 在空行处结束表格，断开之后的行渲染不成表，也不在 registry 标记管的那张表里。
        if re.match(r'\|\s*C\d+\s*\|', s) and not in_history:
            headless.append(f'第 {i} 行 {s.split("|")[1].strip()}：前面没有表头（上一行是空行或别的正文）')
        cur = {'sec': section, 'width': len(cells), 'head': i, 'rows': [], 'reg': marked}
        tables.append(cur); marked = False; continue
    if re.fullmatch(r'\|(\s*:?-+:?\s*\|)+', s):
        continue
    m = re.match(r'\|\s*(C\d+)\s*\|', s)
    if m:
        cur['rows'].append((i, m.group(1), cells))
reg = [t for t in tables if t['reg'] and t['rows']]
if len(reg) < 2:
    print(f'  ✗ 只找到 {len(reg)} 张带编号行的登记表，欠着的与已还清的两张应当都在')
    print('  → 怎么办：表头或它上面的 doc-lint:registry 标记被改坏了；按文件开头的约定把两张表恢复出来。')
    sys.exit(1)
# 欠着的那张里，一格开头写「已还」（后面不是一半 / 这一半 / 大半）或「销账」，就是在说这一行已经不欠了。
paid_claim = re.compile(r'(已还(?!一半|这一半|大半)|销账|(?:20\d\d-\d\d-\d\d\s*)?作废)')
bad, seen, claims, checked = [], {}, [], 0
for t in reg:
    for ln, num, cells in t['rows']:
        checked += 1
        if len(cells) != t['width']:
            bad.append(f"第 {ln} 行 {num}：{len(cells)} 格，而它所在那张表（「{t['sec']}」下、表头在第 {t['head']} 行）是 {t['width']} 格")
        seen.setdefault(num, []).append(ln)
        if t['sec'] == '已还清':
            continue
        for column_index, cell in enumerate(cells[2:], start=3):
            lead = re.sub(r'^[\s*⚠️]+', '', cell)
            if paid_claim.match(lead):
                claims.append(f"第 {ln} 行 {num}：第 {column_index} 列开头写着「{lead.replace('*', '')[:16]}…」")
dup = {k: v for k, v in seen.items() if len(v) > 1}
if bad or dup or claims or headless:
    print(f'  ✗ 欠账表有 {len(bad)} 行的格数与所在表的表头对不上、{len(dup)} 个编号登记了两处、{len(claims)} 处还在欠着的表里却写着已还、{len(headless)} 处被空行断开：')  # gate-lint:summary
    for b in bad:
        print('      ' + b)  # gate-lint:detail
    for k, v in dup.items():
        print(f"      {k} 在第 {'、'.join(map(str, v))} 行各登记一次")  # gate-lint:detail
    for claim in claims:
        print('      ' + claim)  # gate-lint:detail
    for fragment in headless:
        print('      ' + fragment)  # gate-lint:detail
    print('  → 怎么办：六列的欠账行挪回欠着的那张表，接在它最后一行后面，不接在文件末尾；表中间的空行删掉，一张表从表头到最后一行不许断；')
    print('            真还清了的，按「已还清」那张表的四列改写（编号 / 简称 / 怎么还的 / 还清日期）再挪过去，欠着的那张里删掉；')
    print('            只还了一部分的，把那一格的开头改成写明还欠什么的说法（「已还一半」「决策已定、检查仍欠」「前置已还」）。')
    sys.exit(1)
print(f'  ✓ 欠账表两张登记表的行形状都对，欠着的那张里没有写着已还的行（检查了 {checked} 行，{len(reg)} 张表）')
PY
