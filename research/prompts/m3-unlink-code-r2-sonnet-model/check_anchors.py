#!/usr/bin/env python3
"""核冻结副本里 crates/mutations.tsv 每一行：文件在、原文恰好命中一次、替换文与原文不同、点名的测试函数在对应目标文件里有定义。
用法：check_anchors.py <冻结副本根> [--only-names <变异名子串文件>]
只读，不编译、不跑测试。"""
import re, sys, os

root = sys.argv[1]
rows = []
for ln, line in enumerate(open(os.path.join(root, 'crates/mutations.tsv'), encoding='utf-8'), 1):
    if line.startswith('#') or not line.strip():
        continue
    f = line.rstrip('\n').split('\t')
    rows.append((ln, f))

def unesc(s):
    return s.replace('\\n', '\n')

bad = 0
checked = 0
src_cache = {}
def src(path):
    if path not in src_cache:
        src_cache[path] = open(os.path.join(root, path), encoding='utf-8').read()
    return src_cache[path]

def target_files(cargo_args):
    # -p <crate> --test <name> | --lib | --bin <name>
    m_p = re.search(r'-p (\S+)', cargo_args)
    crate = m_p.group(1) if m_p else None
    m_t = re.search(r'--test (\S+)', cargo_args)
    if m_t and crate:
        return [f'crates/{crate}/tests/{m_t.group(1)}.rs', f'crates/{crate}/tests/{m_t.group(1)}/main.rs']
    if '--lib' in cargo_args and crate:
        d = os.path.join(root, 'crates', crate, 'src')
        out = []
        for dp, _, fs in os.walk(d):
            for fn in fs:
                if fn.endswith('.rs'):
                    out.append(os.path.relpath(os.path.join(dp, fn), root))
        return out
    m_b = re.search(r'--bin (\S+)', cargo_args)
    if m_b and crate:
        return [f'crates/{crate}/src/bin/{m_b.group(1)}.rs']
    return []

report = []
for ln, f in rows:
    checked += 1
    if len(f) < 6:
        report.append((ln, f[0][:60], f'列数 {len(f)} 不是 6'))
        bad += 1
        continue
    name, path, orig, repl, args, tests = f[:6]
    problems = []
    if not os.path.exists(os.path.join(root, path)):
        problems.append(f'文件不存在 {path}')
    else:
        n = src(path).count(unesc(orig))
        if n != 1:
            problems.append(f'原文命中 {n} 次')
        if unesc(orig) == unesc(repl):
            problems.append('替换文与原文相同')
    for t in tests.split(','):
        t = t.strip()
        if not t:
            problems.append('没点名测试')
            continue
        short = t.split('::')[-1]
        files = target_files(args)
        found = False
        for tf in files:
            p = os.path.join(root, tf)
            if os.path.exists(p) and re.search(r'fn ' + re.escape(short) + r'\b', open(p, encoding='utf-8').read()):
                found = True
                break
        if not found:
            problems.append(f'点名的测试 {short} 在 {args.split(" -- ")[0]} 目标里没找到定义')
    if problems:
        bad += 1
        report.append((ln, name[:70], '; '.join(problems)))
print(f'检了 {checked} 行，有问题 {bad} 行')
for r in report:
    print(r)
