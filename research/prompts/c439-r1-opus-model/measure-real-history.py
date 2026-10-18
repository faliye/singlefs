#!/usr/bin/env python3
"""接 measure-real.sh：窗口里那几行（射程过滤后、以 E7RESULT 开头），拿 HEAD 历史上出现过的每一版 research/results/**/*.out 比，
数「哪一版产物里都找不到」的行数。格对新增的行只比 replay.sh 登记的产物（树里的，或已归档出树的最后一版），
那个集合是这里「全部产物的全部历史版本」的子集，所以这里数出的行数是格在这个窗口上判红行数的下界。
用法：measure-real-history.py <克隆目录> <名字>…（读 <克隆目录>.added.<名字>）
"""
import subprocess, sys
clone, labels = sys.argv[1], sys.argv[2:]
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
windows = {label: scoped(f'{clone}.added.{label}') for label in labels}
pending = {text for lines in windows.values() for _, text in lines}
listing = subprocess.run(['git', '-C', clone, 'rev-list', '--objects', 'HEAD', '--', 'research/results'], capture_output=True, text=True, check=True).stdout
blobs = sorted({line.split(' ', 1)[0] for line in listing.splitlines() if line.endswith('.out') and ' ' in line})
batch = subprocess.Popen(['git', '-C', clone, 'cat-file', '--batch'], stdin=subprocess.PIPE, stdout=subprocess.PIPE)
batch.stdin.write(''.join(b + '\n' for b in blobs).encode()); batch.stdin.close()
for _ in blobs:
    header = batch.stdout.readline().split()
    size = int(header[2])
    data = batch.stdout.read(size); batch.stdout.read(1)
    if pending:
        for line in data.decode('utf-8', errors='replace').split('\n'):
            if line in pending:
                pending.discard(line)
batch.wait()
print(f'name=real_products_history blobs={len(blobs)}')
for label, lines in windows.items():
    never = [entry for entry in lines if entry[1] in pending]
    print(f'name=real_window_history label={label} quoted_in_scope={len(lines)} not_found_in_any_version={len(never)} files_with_such={len({f for f, _ in never})}')
