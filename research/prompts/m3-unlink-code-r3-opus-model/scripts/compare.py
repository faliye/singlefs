#!/usr/bin/env python3
"""对比丙1（Immediately）与改之前那一版（Held）在同一批起点、同一个前缀刀、同一种丢盘上的判定。
用法：compare.py <imm 日志 glob> <held 日志 glob>"""
import glob, re, sys
from collections import Counter

def parse(pattern):
    shapes = {}
    cur = None
    for path in sorted(glob.glob(pattern)):
        for line in open(path, encoding='utf-8'):
            if line.startswith('START '):
                m = re.search(r'U=(\d+) k=(\d+) u=(\d+) b=(\d+)', line)
                key = m.groups()
                w = int(re.search(r' writes=(\d+)', line).group(1))
                cur = shapes.setdefault(key, {'writes': w, 'raised': 'unmount=Raised' in line, 'cuts': {}, 'current': int(re.search(r'current_txg=(\d+)', line).group(1))})
            elif line.startswith('CUT ') and cur is not None:
                f = dict(re.findall(r'(\w+)=(\S*)', line))
                cur['cuts'][(int(f['n']), f['lost'])] = f
    return shapes

imm, held = parse(sys.argv[1]), parse(sys.argv[2])
c = Counter()
examples = []
for key, a in imm.items():
    b = held.get(key)
    c['shapes'] += 1
    if b is None:
        c['held_missing'] += 1; continue
    if not b['raised']:
        c['held_unmount_refused'] += 1
    elif b['writes'] != a['writes']:
        c['held_writes_differ'] += 1
    for (n, lost), fa in a['cuts'].items():
        if lost == 'none':
            if fa['checker'] not in ('', '-'): c['imm_both_disks_checker_red_cuts'] += 1
            if not fa.get('remount', '').startswith('ok') and n > 0: c['imm_both_disks_remount_not_ok_cuts'] += 1
            if 'after_checker=' in fa.get('remount', '') and not fa['remount'].endswith('after_checker='): c['imm_both_disks_after_remount_checker_red'] += 1
        hit = int(fa['inter_records']) > 0
        ro_fail = fa['whole_read'].startswith('differs[ro') or fa['whole_read'].startswith('differs[open') or fa['whole_read'].startswith('differs[read')
        old_root = fa['chosen'].startswith('Some(') and int(fa['chosen'][5:-1]) <= a['current']
        if hit: c['imm_cuts_with_intersection'] += 1
        if ro_fail and old_root: c['imm_cuts_old_root_read_failure'] += 1
        if not (b['raised'] and b['writes'] == a['writes']):
            if ro_fail and old_root: c['imm_old_root_read_failure_where_held_has_no_same_cut'] += 1
            continue
        fb = b['cuts'].get((n, lost))
        if fb is None: continue
        hb = int(fb['inter_records']) > 0
        if hb: c['held_cuts_with_intersection'] += 1
        rb = fb['whole_read'].startswith('differs[ro') or fb['whole_read'].startswith('differs[open') or fb['whole_read'].startswith('differs[read')
        if hit:
            c['comparable_cuts_with_imm_intersection'] += 1
            if fa['checker'] == fb['checker']: c['  ..same_checker_names_as_held'] += 1
            else:
                c['  ..checker_names_differ_from_held'] += 1
                if len(examples) < 6: examples.append((key, n, lost, fa['checker'], fb['checker']))
        if ro_fail and old_root:
            c['comparable_cuts_imm_old_root_read_failure'] += 1
            if not rb: c['  ..held_read_ok_at_same_cut'] += 1
        if rb: c['held_cuts_read_failure'] += 1
for k in sorted(c): print(f'{k}={c[k]}')
for e in examples: print('EXAMPLE', e)
