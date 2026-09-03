#!/usr/bin/env python3
"""决策分项引用的状态与正文是否一致。

每一处「D<n>（简称） 已定项 k / 未定项 k」的**前缀**都断言了那一条分项的状态。
写错了没有任何东西会发现——检索会把「已定项 5」当成已经定了的东西端出去，
而它可能还开着。本检查拿各决策正文里的分项表当权威，逐处比对。

判据：引用处写「已定项 k」而正文那条标着未定（或反过来）⇒ 判红。
归属靠同一行里最近的 D 记号；行内没有 D 记号时，用文件自身的决策号，
再退到该行之前最近出现过的 D 记号——三级都判不出的也判红，因为
一个归属判不出的编号引用，读的人同样判不出。
"""
import re, glob, sys, os

def load_map():
    m, name = {}, {}
    for f in sorted(glob.glob('.claude/kb/decisions/*.md')):
        s = open(f, encoding='utf-8').read()
        t = s.split('\n', 1)[0]
        mm = re.match(r'## (D\d+) (.+?)\s*——', t)
        if not mm: continue
        d = mm.group(1); name[d] = mm.group(2).strip(); m[d] = {}
        for head, st in (('已定项', '已'), ('未定项', '未')):
            sec = re.search(r'^### %s\s*$(.*?)(?=^#{1,3} |\Z)' % head, s, re.M | re.S)
            if not sec: continue
            seg = sec.group(1)
            cut = re.search(r'^#{4}\s', seg, re.M)
            top = seg[:cut.start()] if cut else seg
            # ⚠️ 正则要与 `.claude/scripts/gen-decision-items.py` 逐字同口径：
            # 那边要求表格行有**闭合的第二根竖线**。两边不一致时，一行畸形表格
            # 会「一边看得见、一边看不见」，而两边都不报错（2026-08-30 对抗验证指出）。
            for n in re.findall(r'^\|\s*(\d+)\s*\|[^|]*\|', top, re.M) or re.findall(r'^(\d+)\.\s', top, re.M):
                m[d][int(n)] = st
    return m, name

MAP, NAME = load_map()
bad = []
files = sorted(set(sum([glob.glob(p, recursive=True) for p in
    ('.claude/kb/**/*.md', 'records/**/*.md', 'research/**/*.md',
     'research/**/*.rs', '.claude/rules/*.md')], [])))
# research/prompts/ 显式排除，理由与 26 号门禁相同：那是原样发给模型的提示与模型的原样输出，
# 与 research/results/ 里的产物一一对应，事后改它等于让产物对不上输入。
# 实测（2026-09-03）：反推腿的输出里有一条复现命令 `grep -n "已定项 8" …`，
# 没有决策号可归属，按正文规则判红，而那一行按证据链不许改。
files = [f for f in files if '/prompts/' not in f]
SELF = {f: re.match(r'## (D\d+)', open(f, encoding='utf-8').read()).group(1)
        for f in glob.glob('.claude/kb/decisions/*.md')}
for path in files:
    self_d = SELF.get(path); last = None; hist = None
    for ln, line in enumerate(open(path, encoding='utf-8').read().split('\n'), 1):
        if path.endswith('decisions-history.md') and line.startswith('### '):
            mm = re.search(r'D(\d+)', line); hist = 'D' + mm.group(1) if mm else None
        for m in re.finditer(r'(已定项|未定项)\s*(\d+)', line):
            k = int(m.group(2)); want = '已' if m.group(1) == '已定项' else '未'
            pre = line[:m.start()]; ds = re.findall(r'D(\d+)', pre)
            owner = next((c for c in ('D' + ds[-1] if ds else None, self_d, hist, last)
                          if c in MAP and k in MAP[c]), None)
            if owner is None:
                bad.append(f"{path}:{ln} 「{m.group(0)}」归属判不出：{line.strip()[:70]}")
            elif MAP[owner][k] != want:
                bad.append(f"{path}:{ln} 「{m.group(0)}」写的是{want}定，而 {owner}"
                           f"（{NAME[owner]}） 正文里第 {k} 条是{MAP[owner][k]}定：{line.strip()[:60]}")
        for d in re.findall(r'D(\d+)', line):
            if 'D' + d in MAP: last = 'D' + d
if bad:
    print(f"  ✗ 分项引用与正文状态不一致 {len(bad)} 处")
    for b in bad[:40]: print("    ", b)
    print("     → 权威是各决策正文的「### 已定项 / ### 未定项」两张表；改引用处，或先改正文再改引用。")
    sys.exit(1)
n = sum(len(v) for v in MAP.values())
print(f"  ✓ 分项引用与正文状态一致（{len(MAP)} 条决策、{n} 个分项）")
