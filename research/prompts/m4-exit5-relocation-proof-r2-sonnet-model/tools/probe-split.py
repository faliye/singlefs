#!/usr/bin/env python3
"""把 y2 探针文件里 verus! 块中的每个 pub fn 拆成单独的探针文件：probe-split.py <源> <输出目录>"""
import re, sys, os
src, out = sys.argv[1], sys.argv[2]
text = open(src, encoding='utf-8').read()
body = text[text.index('verus! {') + len('verus! {'): text.rindex('}\nfn main')]
chunks = re.split(r'\n(?=pub fn )', '\n' + body)
os.makedirs(out, exist_ok=True)
n = 0
for c in chunks:
    c = c.strip()
    m = re.match(r'pub fn (\w+)', c)
    if not m:
        continue
    # 把紧挨在 fn 前面的注释行也并进去无所谓；这里只取函数
    end = c.find('\n}\n')
    fn = c if end < 0 else c[:end + 3]
    with open(os.path.join(out, m.group(1) + '.rs'), 'w', encoding='utf-8') as f:
        f.write('use vstd::prelude::*;\nverus! {\n' + fn + '\n}\nfn main() {}\n')
    n += 1
print(n)
