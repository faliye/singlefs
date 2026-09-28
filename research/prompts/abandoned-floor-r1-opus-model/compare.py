#!/usr/bin/env python3
"""按格比各候选：每格 = (实验, 键, F)；签名 = 抬 F 结局 + 抬后违例 + 各段后缀的违例。
输出：每个候选「抬前绿而之后出现红」的格数与红的种类，和与 today 签名不同的格。"""
import re, sys, collections, glob

cell_re = re.compile(r'AF-CELL cand=(\S+) abF=(\S+) exp=(\S+) (.*)$')
verdict_re = re.compile(r'=>(\[[^\]]*\])( NA\[[^\]]*\])?')

def signature(rest):
    key_part = rest.split(' before=')[0] if ' before=' in rest else rest.split(' raise=')[0].split(' result=')[0]
    key_part = re.sub(r' txg_before=\d+', '', key_part)
    before = re.search(r' before=(\[[^\]]*\])', rest)
    before = before.group(1) if before else '-'
    raise_part = rest.split(' raise=', 1)[1].split(' post=', 1)[0] if ' raise=' in rest else ''
    if ' result=' in rest:
        raise_part = rest.split(' result=', 1)[1]
    status = 'ok' if raise_part.startswith('ok') or raise_part.startswith('unmount') else ('at' if raise_part.startswith('at-ceiling') else ('refused:' + raise_part.split('[')[0].split('=>')[0] if raise_part else 'none'))
    landed = re.search(r'ok\(F=(\d+)', raise_part)
    reclaimed = re.search(r'reclaimed (\d+)', raise_part)
    after = verdict_re.search(raise_part)
    after = (after.group(1) + (after.group(2) or '')) if after else '-'
    posts = []
    post_part = rest.split(' post=[', 1)[1] if ' post=[' in rest else ''
    for seg in post_part.rstrip(']').split(' | '):
        if ':' not in seg:
            continue
        name, body = seg.split(':', 1)
        sets = [a + (b or '') for a, b in verdict_re.findall(body)]
        tag = 'refused' if not sets else '>'.join(sets)
        posts.append(f"{name}:{tag}")
    return key_part, before, status, (landed.group(1) if landed else '-'), (reclaimed.group(1) if reclaimed else '-'), after, tuple(posts)

def load(path):
    cells = {}
    for line in open(path, errors='replace'):
        m = cell_re.search(line)
        if not m or m.group(3).endswith('-setup') or m.group(3) == 'E5':
            continue
        sig = signature(m.group(4))
        cells[(m.group(3), sig[0])] = sig
    return cells

def reds(sig):
    out = set()
    if sig[5] not in ('-', '[]'):
        out.add('after:' + sig[5])
    for post in sig[6]:
        name, tags = post.split(':', 1)
        for t in tags.split('>'):
            if t not in ('[]', 'refused'):
                out.add(re.sub(r'\d+', 'N', name) + ':' + t)
    return out

paths = sys.argv[1:]
base = load(paths[0])
for path in paths:
    cells = load(path)
    label = path.split('/')[-1]
    green_before = [k for k, s in cells.items() if s[1] == '[]']
    red_after_green = collections.Counter()
    red_cells = []
    for k in green_before:
        r = reds(cells[k])
        if r:
            for x in r:
                red_after_green[x.split(':', 1)[1]] += 1
            red_cells.append(k)
    diff = [k for k in cells if k in base and cells[k][2:] != base[k][2:]]
    print(f"== {label}: cells={len(cells)} green_before={len(green_before)} red_after_green_before={len(red_cells)} differs_from_first={len(diff)}")
    for name, n in red_after_green.most_common():
        print(f"   red-kind {name}: {n} occurrences")
    exps = collections.Counter(k[0] for k in red_cells)
    print(f"   red cells by exp: {dict(exps)}")
