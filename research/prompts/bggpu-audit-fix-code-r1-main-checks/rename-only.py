#!/usr/bin/env python3
"""判一份 .rs 相对 HEAD 的改动是不是只有标识符改名：去注释、去空白、去掉收尾逗号之后，
两份记号流等长，非标识符记号逐个相同，标识符之间是一张前后一致的一一映射。"""
import re, subprocess, sys
TOKEN = re.compile(r'''
  (?P<linecomment>//[^\n]*)
 |(?P<blockcomment>/\*.*?\*/)
 |(?P<rawstring>b?r(?P<h>\#*)".*?"(?P=h))
 |(?P<string>b?"(?:\\.|[^"\\])*")
 |(?P<char>b?'(?:\\.[^']*|[^'\\])')
 |(?P<lifetime>'[A-Za-z_][A-Za-z0-9_]*)
 |(?P<ident>[A-Za-z_][A-Za-z0-9_]*)
 |(?P<number>[0-9][A-Za-z0-9_.]*)
 |(?P<space>\s+)
 |(?P<punct>.)
''', re.S | re.X)
def tokens(text):
    out = []
    for m in TOKEN.finditer(text):
        kind = m.lastgroup
        if kind == 'h': kind = 'rawstring'
        if kind in ('linecomment', 'blockcomment', 'space'): continue
        out.append((kind, m.group(0)))
    cleaned = []
    for i, (kind, value) in enumerate(out):
        if value == ',' and i + 1 < len(out) and out[i + 1][1] in ')]}>':
            continue
        cleaned.append((kind, value))
    return cleaned
def judge(path):
    old = subprocess.run(['git', 'show', 'HEAD:' + path], capture_output=True, text=True)
    if old.returncode != 0: return 'HEAD 里没有', {}
    a, b = tokens(old.stdout), tokens(open(path, encoding='utf-8').read())
    if len(a) != len(b): return f'记号数不同 {len(a)} / {len(b)}', {}
    forward, backward = {}, {}
    for (ka, va), (kb, vb) in zip(a, b):
        if ka != kb: return f'记号种类不同 {va!r} / {vb!r}', {}
        if ka in ('ident', 'lifetime'):
            if forward.setdefault(va, vb) != vb: return f'{va} 映到两个名字 {forward[va]} / {vb}', {}
            if backward.setdefault(vb, va) != va: return f'{vb} 来自两个名字 {backward[vb]} / {va}', {}
        elif va != vb: return f'记号不同 {va!r} / {vb!r}', {}
    return '只改名', {k: v for k, v in forward.items() if k != v}
for path in sys.argv[1:]:
    verdict, renames = judge(path)
    print(f'{path}\t{verdict}\t{len(renames)} 个名字' + (''.join(f'\n    {k} -> {v}' for k, v in sorted(renames.items())) if '--names' in sys.argv else ''))
