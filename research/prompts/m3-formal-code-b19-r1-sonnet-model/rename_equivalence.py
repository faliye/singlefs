#!/usr/bin/env python3
"""正推腿：19-2 的改名有没有动行为。输入：补丁反向打回去的「改前」文件与冻结副本里的「改后」文件。
做法：去掉 // 与 /// 注释后切成记号；两边记号数必须相等；记号不同的位置必须都是标识符；
再对每个标识符记号按「最近的前一个 let 绑定」解析（let 到第一个 = 之间的标识符都算绑定），
要求每个位置在改前、改后解析到同一个绑定位置——改名没有把某处使用从一个绑定挪到另一个绑定上。
用法：rename_equivalence.py 改前.rs 改后.rs"""
import re, sys
TOK = re.compile(r'"(?:\\.|[^"\\])*"|\'(?:\\.|[^\'\\])\'|[A-Za-z_][A-Za-z0-9_]*|[0-9][0-9A-Za-z_.]*|\S')
def tokens(path):
    out = []
    for line in open(path, encoding="utf-8"):
        code = re.sub(r'//.*$', '', line) if '"' not in line.split('//')[0][-1:] else line
        # 字符串里带 // 的行不多；先按最朴素的去注释，再核记号数
        out += TOK.findall(code)
    return out
def resolve(toks):
    last_let = {}
    res = []
    i = 0
    n = len(toks)
    bind_positions = set()
    while i < n:
        t = toks[i]
        if t == 'let':
            j = i + 1
            names = []
            depth = 0
            while j < n and not (toks[j] in ('=', ';') and depth == 0):
                if toks[j] in '([{': depth += 1
                if toks[j] in ')]}': depth -= 1
                if re.fullmatch(r'[a-z_][A-Za-z0-9_]*', toks[j]) and toks[j] not in ('mut', 'ref', 'let'):
                    names.append((toks[j], j))
                j += 1
            for name, pos in names:
                bind_positions.add(pos)
            res += [None] * (j - i)
            # 绑定在 = 右边的表达式之后才生效：先解析右边，再登记
            k = j
            # 右边到分号为止（粗略：到下一个顶层 ;）
            depth = 0
            while k < n and not (toks[k] == ';' and depth == 0):
                if toks[k] in '([{': depth += 1
                if toks[k] in ')]}': depth -= 1
                k += 1
            for m in range(j, k + 1):
                if m < n and re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', toks[m]):
                    res.append(last_let.get(toks[m]))
                else:
                    res.append(None)
            for name, pos in names:
                last_let[name] = pos
            i = k + 1
            continue
        res.append(last_let.get(t) if re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', t) else None)
        i += 1
    return res
pre, post = tokens(sys.argv[1]), tokens(sys.argv[2])
print(f"记号数 改前 {len(pre)} 改后 {len(post)}")
if len(pre) != len(post):
    sys.exit("记号数不等")
diff = [(i, a, b) for i, (a, b) in enumerate(zip(pre, post)) if a != b]
non_ident = [d for d in diff if not (re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', d[1]) and re.fullmatch(r'[A-Za-z_][A-Za-z0-9_]*', d[2]))]
print(f"不同的记号 {len(diff)} 个，其中不是标识符对标识符的 {len(non_ident)} 个")
for d in non_ident[:10]: print("  非标识符差：", d)
pairs = {}
for _, a, b in diff: pairs.setdefault(a, set()).add(b)
for a, bs in sorted(pairs.items()): print(f"  {a} -> {sorted(bs)}")
rp, rq = resolve(pre), resolve(post)
bad = [(i, pre[i], post[i], rp[i], rq[i]) for i in range(len(pre)) if rp[i] != rq[i]]
print(f"解析到的绑定位置前后不同的使用 {len(bad)} 处")
for b in bad[:15]: print("  ", b)
