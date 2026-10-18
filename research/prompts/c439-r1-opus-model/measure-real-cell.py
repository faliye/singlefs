#!/usr/bin/env python3
"""接 measure-real.sh：照冻结副本 .claude/gate.d/doc-experiments.sh 第 1694–1773 行的比对法（重写，不是跑那一格），
数一个窗口里的整行抄产物行有几行找不到。
  窗口判（scope 不是全量）：只比 scripts/experiments/replay.sh 登记着的产物（树里的；登记着、已出了树的取归档前最后一版）
  全量判：比树里全部产物，再加全部归档过的产物
产物行与引文都去首尾空白、去 \\r（第 1718、1747 行）。历史节「只判新增的」那一条没照抄：窗口里的行一律算判。
用法（在克隆的工作区里跑）：measure-real-cell.py <模式 window|full> <名字>…（读 ../<克隆目录名>.added.<名字>）
"""
import glob, os, re, subprocess, sys
mode, labels = sys.argv[1], sys.argv[2:]
clone = os.getcwd()
def scoped(path):
    out = []
    for record in open(path, encoding='utf-8', errors='replace').read().split('\n'):
        file, sep, text = record.partition('\t')
        if not sep or not file.endswith('.md') or not (file.startswith('.claude/kb/') or file.startswith('research/')):
            continue
        if file.endswith('-history.md') or '/decisions-history/' in file or file.startswith(('research/prompts/', 'research/results/')):
            continue
        if text.strip().startswith('E7RESULT '):
            out.append((file, text.strip()))
    return out
ROW_RE = re.compile(r'^(E[0-9]+)\|([^|]*)\|([^|]*)\|([^|]*)\|(exact|timing)$')
registered = set()
for line in open('scripts/experiments/replay.sh', encoding='utf-8'):
    row = ROW_RE.match(line.rstrip('\n'))
    if row:
        registered.add(row.group(4))
products = sorted(glob.glob('research/results/*.out'))
tree_lines, registered_lines = set(), set()
for product in products:
    with open(product, encoding='utf-8', errors='ignore') as handle:
        for line in handle:
            text = line.strip().replace('\r', '')
            tree_lines.add(text)
            if os.path.basename(product) in registered:
                registered_lines.add(text)
def archived(only_names=None):
    lines = set()
    log = subprocess.run(['git', '-c', 'core.quotepath=false', 'log', '--all', '--diff-filter=D', '--format=%H', '--name-only', '--', 'research/results'],
                         capture_output=True, text=True).stdout
    commit = None
    for entry in log.split('\n'):
        entry = entry.strip()
        if not entry:
            continue
        if len(entry) == 40 and all(c in '0123456789abcdef' for c in entry):
            commit = entry
            continue
        if commit and entry.endswith('.out') and (only_names is None or os.path.basename(entry) in only_names):
            blob = subprocess.run(['git', 'show', f'{commit}^:{entry}'], capture_output=True, text=True)
            if blob.returncode == 0:
                lines.update(one.strip().replace('\r', '') for one in blob.stdout.split('\n'))
    return lines
if mode == 'window':
    compared = registered_lines | archived(only_names=registered - {os.path.basename(p) for p in products})
else:
    compared = tree_lines | archived()
print(f'name=real_cell_compare mode={mode} registered={len(registered)} products_in_tree={len(products)} compared_lines={len(compared)}')
for label in labels:
    lines = scoped(f'{clone}.added.{label}')
    missing = [entry for entry in lines if entry[1] not in compared]
    files = sorted({f for f, _ in missing})
    print(f'name=real_cell_window mode={mode} label={label} quoted_in_scope={len(lines)} not_found={len(missing)} files_with_not_found={len(files)}')
    for f in files:
        print(f'name=real_cell_file mode={mode} label={label} file={f} not_found={sum(1 for g, _ in missing if g == f)}')
