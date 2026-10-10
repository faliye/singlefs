#!/usr/bin/env python3
"""vw_extract.py <源文件> <文件名> <输出 main.rs> [parent=名字…] ：从源文件的 `mod tests {` 里机械抽出「读自己源文件的扫源码用例」连同它们（传递地）引用的条目，
拼成独立小程序：include_str!("<文件名>") 换成读环境变量 SRC 指的文件，#[test] 去掉，main 逐条在 catch_unwind 里跑、打「名字: GREEN/RED」。
起点 = 含 include_str!("<文件名>") 的用例 + 命令行 extra=名字 点名的；闭包 = 条目里出现过的别的条目名。
parent=… 点名的父模块条目（tests 模块用 use super::* 取的）原样从父模块抄一份（常量或函数）。"""
import re, sys
src_path, file_name, out_path = sys.argv[1:4]
extra = [a[6:] for a in sys.argv[4:] if a.startswith('extra=')]
parent_names = [a[7:] for a in sys.argv[4:] if a.startswith('parent=')]
skips = [a[5:] for a in sys.argv[4:] if a.startswith('skip=')]
lines = open(src_path, encoding='utf-8').read().split('\n')
start = next(i for i, l in enumerate(lines) if l == 'mod tests {')
end = max(i for i, l in enumerate(lines) if l == '}')
body = lines[start + 1:end]
item_re = re.compile(r'^    (?:pub(?:\([a-z]+\))? )?(fn|const|static|struct|enum|impl|type|use)\b\s*(?:<[^>]*>)?\s*([A-Za-z_][A-Za-z_0-9]*)?')
items = []  # (name, kind, text_lines, is_test)
i = 0
while i < len(body):
    m = item_re.match(body[i])
    if not m:
        i += 1; continue
    kind, name = m.group(1), m.group(2)
    # 往上收属性与文档
    j = i
    while j > 0 and (body[j - 1].startswith('    #[') or body[j - 1].startswith('    ///') or body[j - 1].startswith('    //')):
        j -= 1
    # 往下找收尾
    k = i
    if kind in ('const', 'static', 'type', 'use'):
        bal = 0
        while True:
            bal += sum(body[k].count(c) for c in '([{') - sum(body[k].count(c) for c in ')]}')
            if bal == 0 and body[k].rstrip().endswith(';'):
                break
            k += 1
    else:
        if body[i].rstrip().endswith('}') and body[i].count('{') == body[i].count('}') and '{' in body[i]:
            k = i
        else:
            while body[k] != '    }':
                k += 1
    text = body[j:k + 1]
    is_test = any(l.strip() == '#[test]' for l in text)
    if kind == 'impl':
        name = 'impl:' + body[i].strip()
    items.append((name, kind, text, is_test))
    i = k + 1
by_name = {n: it for it in items for n in [it[0]] if it[1] != 'use'}
fname_re = re.compile(r'include_str!\(\s*"%s"\s*\)' % re.escape(file_name))
only = any(a=='only=1' for a in sys.argv[4:])
want = ([] if only else [it[0] for it in items if it[3] and fname_re.search('\n'.join(it[2])) and it[0] not in skips]) + extra
chosen, queue = [], list(want)
while queue:
    n = queue.pop(0)
    if n in chosen or n not in by_name or n in skips: continue
    chosen.append(n)
    text = '\n'.join(by_name[n][2])
    for other in by_name:
        if other not in chosen and not other.startswith('impl:') and re.search(r'\b' + re.escape(other) + r'\b', text):
            queue.append(other)
# impl 块：目标类型在已选条目里就带上
for n, it in by_name.items():
    if n.startswith('impl:'):
        m2 = re.match(r'impl(?:<[^>]*>)?\s+(?:[A-Za-z_:<>]+\s+for\s+)?([A-Za-z_][A-Za-z_0-9]*)', n[5:])
        if m2 and m2.group(1) in chosen and n not in chosen: chosen.append(n)
order = [it[0] for it in items if it[0] in chosen]
uses = [it for it in items if it[1] == 'use' and not re.search(r'super|crate::|singlefs_|wgpu', ' '.join(it[2]))]
text = '\n'.join(l for n in order for l in by_name[n][2] + [''] if l.strip() != '#[test]')
text = '\n'.join(l for it in uses for l in it[2]) + '\n' + text
text = fname_re.sub('source_text()', text)
parent = []
for pn in parent_names:
    # 父模块里的 const/fn：从 mod tests 之前找
    pat = re.compile(r'^(?:pub(?:\([a-z]+\))? )?(?:fn|const|static)\s+' + re.escape(pn) + r'\b')
    idx = [i for i in range(start) if pat.match(lines[i])]
    assert len(idx) == 1, (pn, idx)
    a = idx[0]; b = a
    bal = 0
    while True:
        bal += sum(lines[b].count(c) for c in '([{') - sum(lines[b].count(c) for c in ')]}')
        if bal == 0 and (lines[b].rstrip().endswith(';') or lines[b].rstrip().endswith('}')): break
        b += 1
    parent += lines[a:b + 1] + ['']
tests = [n for n in order if by_name[n][3]]
main = ['#![allow(dead_code, unused, warnings)]',
        'fn source_text() -> &\'static str { Box::leak(std::fs::read_to_string(std::env::var("SRC").expect("SRC")).expect("读得到 SRC").into_boxed_str()) }',
        '\n'.join(parent), text, 'fn main() {',
        '    std::panic::set_hook(Box::new(|info| { if std::env::var("VW_TRACE").is_ok() { eprintln!("{info}"); } }));',
        '    let wanted: Vec<String> = std::env::args().skip(1).collect();',
        '    let tests: Vec<(&str, fn())> = vec![' + ', '.join('("%s", %s)' % (t, t) for t in tests) + '];',
        '    for (name, test) in tests {',
        '        if !wanted.is_empty() && !wanted.iter().any(|w| w == name) { continue; }',
        '        let verdict = if std::panic::catch_unwind(test).is_err() { "RED" } else { "GREEN" };',
        '        println!("{name}: {verdict}");',
        '    }', '}']
nomain = any(a=='nomain=1' for a in sys.argv[4:])
if nomain: main = main[:main.index('fn main() {')]
open(out_path, 'w', encoding='utf-8').write('\n'.join(main) + '\n')
print(f'抽出 {len(order)} 个条目（用例 {len(tests)} 条）：', ' '.join(tests))
