#!/usr/bin/env python3
"""戊（全量判 + 只缩不涨的存量豁免表）的判法：在当前目录（被判的树）里打出要判的行「路径<TAB>行文」。
用法：wu_judge.py <set|count|fixed|cf> <豁免表比对的基准提交>
  set / fixed：豁免按（路径、行文）集合认；count / cf：按（路径、行文）计数认，多出来的那几行照判。
  豁免表 .claude/kb/quoted-exempt.tsv 比基准那一版多出来的行，打成「路径<TAB>GROWN …」（判红）。
"""
import collections, glob, subprocess, sys
mode, table_base = sys.argv[1], sys.argv[2]
TABLE = '.claude/kb/quoted-exempt.tsv'
def rows(text):
    return [tuple(line.split('\t', 1)) for line in text.splitlines() if '\t' in line]
current = rows(open(TABLE, encoding='utf-8').read())
shown = subprocess.run(['git', 'show', f'{table_base}:{TABLE}'], capture_output=True, text=True)
base_rows = rows(shown.stdout) if shown.returncode == 0 else []
for row in sorted(set(current) - set(base_rows)):
    print(f'{row[0]}\tGROWN {row[1]}')
exempt = collections.Counter(current) if mode in ('count', 'cf') else None
exempt_set = set(current)
for path in sorted(glob.glob('.claude/kb/**/*.md', recursive=True) + glob.glob('research/**/*.md', recursive=True)):
    for line in open(path, encoding='utf-8').read().split('\n'):
        text = line.strip()
        if not text.startswith('E7RESULT '):
            continue
        key = (path, text)
        if exempt is not None:
            if exempt[key] > 0:
                exempt[key] -= 1
                continue
        elif key in exempt_set:
            continue
        print(f'{path}\t{text}')
