#!/usr/bin/env python3
"""T1-b 的真仓对子：决策正文（历史版本之前）里，实验号不按 `E<号>（` 写、而 doc-lint G 认的写法
（粗体 / 反引号包住编号、编号与括号之间有空格、半角括号），且同一份决策里这个实验号一处标准写法都没有的（只读）。
这种对子上 75 号 ⑤ 后一半看不见「这条决策引了它」。
用法：python3 t1-mention-forms.py <仓根>
"""
import glob, os, re, sys
os.chdir(sys.argv[1])
STANDARD = re.compile(r'(?<![A-Za-z0-9])E(\d+)（')
LOOSE = re.compile(r'(?<![A-Za-z0-9._-])[*`]*E(\d+)[*`]*\s*[（(]')
files = sorted(glob.glob('.claude/kb/decisions/*.md'))
hits, loose_total = [], 0
for path in files:
    text = open(path, encoding='utf-8').read()
    cut = text.find('\n## 历史版本')
    body = text if cut < 0 else text[:cut]
    standard = {int(n) for n in STANDARD.findall(body)}
    for match in LOOSE.finditer(body):
        if body[match.start():match.end()].endswith('（') and body[match.end(1):match.end(1) + 1] == '（':
            continue   # 就是标准写法
        loose_total += 1
        if int(match.group(1)) not in standard:
            hits.append((path, match.group(0)))
print(f'决策 {len(files)} 份；非标准写法（doc-lint 认、75 号 :68 不认）的实验号引用 {loose_total} 处，'
      f'其中同一份决策里没有标准写法的 {len(hits)} 处')
for path, form in hits:
    print(f'  {path}：{form}')
