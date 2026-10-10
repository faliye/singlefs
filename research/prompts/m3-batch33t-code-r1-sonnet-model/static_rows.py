#!/usr/bin/env python3
# 用法：python3 -I static_rows.py <改动原样 diff 文件> <冻结副本根> —— 静态核：diff 里追加的 18 行与替换的 20 行，与副本的 crates/mutations.tsv 逐行相同、原文在文件里恰好命中一次、必须红的测试名在点名的测试文件里是 fn。
import os, re, sys
diff = open(sys.argv[1], encoding='utf-8').read().split('\n')
root = sys.argv[2]
def block(marker):
    i = [k for k, l in enumerate(diff) if l.startswith(marker)][0]
    j = i + 1
    while not diff[j].startswith('````tsv'):
        j += 1
    j += 1
    out = []
    while not diff[j].startswith('````'):
        out.append(diff[j]); j += 1
    return [x for x in out if x.strip()]
appended, replaced = block('## 追加的变异行'), block('## 按名字整行替换')
print('appended', len(appended), 'replaced', len(replaced))
table = {}
for l in open(os.path.join(root, 'crates/mutations.tsv'), encoding='utf-8'):
    l = l.rstrip('\n')
    if l.strip() and not l.startswith('#'):
        table.setdefault(l.split('\t')[0], []).append(l)
problems = 0
for tag, rows in (('appended', appended), ('replaced', replaced)):
    for l in rows:
        f = l.split('\t'); name = f[0]
        t = table.get(name)
        if len(f) != 6: print(tag, 'MALFORMED', name[:60]); problems += 1; continue
        if not t or len(t) != 1: print(tag, 'NOT-ONCE-IN-TABLE', name[:70]); problems += 1
        elif t[0] != l: print(tag, 'ROW-DIFFERS', name[:70]); problems += 1
        src = open(os.path.join(root, f[1]), encoding='utf-8').read()
        c = src.count(f[2].replace('\\n', '\n'))
        if c != 1: print(tag, 'ANCHOR-COUNT', c, name[:70]); problems += 1
        m = re.search(r'--test (\S+)', f[4]); pkg = re.search(r'-p (\S+)', f[4]).group(1)
        if m:
            tp = os.path.join(root, 'crates', pkg, 'tests', m.group(1) + '.rs')
            if not os.path.exists(tp): print(tag, 'NO-TEST-FILE', m.group(1)[:60]); problems += 1; continue
            s = open(tp, encoding='utf-8').read()
            for nm in f[5].split():
                if not re.search(r'fn ' + re.escape(nm) + r'\b', s): print(tag, 'NO-FN', nm[:60]); problems += 1
print('problems', problems)
