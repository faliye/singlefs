#!/usr/bin/env python3
"""erased_lines.py <文件>…：每个文件里 verus! { … } 去掉规格与证明之后的纯 Rust（strip_verus.py 的原型，rustfmt 排版）有多少行；对比文件里 verus! 块内去掉空行与 // 注释行的行数。"""
import re, subprocess, sys, importlib.util, os
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('strip_verus', os.path.join(os.path.dirname(os.path.abspath(__file__)), 'strip_verus.py'))
sv = importlib.util.module_from_spec(spec); spec.loader.exec_module(sv)
tot_a = tot_b = 0
for path in sys.argv[1:]:
    text = open(path, encoding='utf-8').read()
    inner_all = []
    for m in re.finditer(r'verus!\s*\{', text):
        depth = 0; end = None
        for tm in sv.TOKEN.finditer(text, m.end() - 1):
            if tm.group(0) == '{': depth += 1
            elif tm.group(0) == '}':
                depth -= 1
                if depth == 0: end = tm.start(); break
        inner_all.append(text[m.end():end])
    inner = '\n'.join(inner_all)
    lines_in = sum(1 for l in inner.split('\n') if l.strip() and not l.strip().startswith('//'))
    plain = ' '.join(sv.strip_items(sv.tokens(inner)))
    r = subprocess.run(['rustfmt', '--edition', '2021', '--emit', 'stdout'], input=plain, capture_output=True, text=True)
    erased = sum(1 for l in r.stdout.split('\n') if l.strip() and not l.strip().startswith('//')) if r.returncode == 0 else -1
    print(f'file={os.path.basename(path)} verus_block_code_lines={lines_in} erased_rust_lines_rustfmt={erased} ratio={lines_in/max(erased,1):.2f} rustfmt_rc={r.returncode}')
    if erased > 0: tot_a += lines_in; tot_b += erased
print(f'total verus_block_code_lines={tot_a} erased={tot_b} ratio={tot_a/max(tot_b,1):.2f}')
