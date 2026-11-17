#!/usr/bin/env python3
"""对 extract.py 写出的 pubs.tsv 做校验与读数。
用法：checks.py validate <pubs.tsv> <合并报告.out>   # 重算的 B8 对合并报告 name=q8 行（第一次与峰值）
      checks.py calls <结果目录>                    # 每次发布 between 段释放查找调用数 == 4 × 各轮释放数之和
      checks.py denominators <pubs.tsv>             # 一单元覆盖写第 1 次（同一次挂载）在各点上四种分母的对照
"""
import csv, glob, re, sys, collections


def read_rows(path):
    return list(csv.DictReader(open(path), delimiter='\t'))


def validate(pubs_path, report_path):
    by = collections.defaultdict(list)
    for r in read_rows(pubs_path):
        by[(r['item'], r['state'])].append(r)
    report = {}
    for line in open(report_path):
        m = re.match(r'name=q8 item=(\S+) state=(\S+) first_publish=(\S+) peak=(\S+) ', line)
        if m:
            report[(m[1], m[2])] = (m[3], m[4])
    same = diff = missing = 0
    for key, value in sorted(report.items()):
        if key not in by:
            missing += 1
            print('missing', key, value)
            continue
        mine = (str(by[key][0]['B8']), str(max(int(x['B8']) for x in by[key])))
        if mine == value:
            same += 1
        else:
            diff += 1
            print('DIFF', key, value, mine)
    print('report_q8_lines', len(report), 'same', same, 'diff', diff, 'missing', missing)


def calls(results_dir):
    total = ok = 0
    bad = []
    for path in sorted(glob.glob(results_dir + '/e206-fixed-point-cost-*-2026-11-09.out')):
        if not re.search(r'-(p1|p2a)-2026', path):
            continue
        finds = {}
        released = collections.defaultdict(int)
        settles = []
        for line in open(path):
            if not line.startswith('E206RESULT ') or ' phase=window ' not in line:
                continue
            f = dict(t.split('=', 1) for t in line.split()[1:] if '=' in t)
            name = f['name']
            if name == 'row' and f['segment'] == 'between' and f['row'] == 'allocator_release_linear_find':
                finds[f['publish']] = int(f['calls'])
            elif name == 'round':
                released[(f['state'], f['op'], f['op_index'], f['settle_call'])] += int(f['placements_released'])
            elif name == 'settle':
                settles.append(f)
        for s in settles:
            key = f"{s['state']}/{s['op']}/{s['op_index']}/{s['judgement']}"
            want = 4 * released[(s['state'], s['op'], s['op_index'], s['settle_call'])]
            total += 1
            if finds.get(key) == want:
                ok += 1
            else:
                bad.append((path[-34:], key, finds.get(key), want))
    print('publishes', total, 'calls==4*sum_rel', ok, 'other', len(bad))
    print(bad[:3])


def denominators(pubs_path):
    rows = [r for r in read_rows(pubs_path) if r['state'] == 'same_mount' and r['op'] == 'W1' and r['op_index'] == '1' and r['arm'] in ('p2a', 'p1')]
    order = ['base', 'files_100', 'files_1000', 'files_10000', 'files_30000', 'files_1000_units_10', 'files_1000_units_100', 'data_units_10000', 'unlinks_10000']
    print('arm point rounds T_first(rel1+rw1) rel_last final_roles sum_rel B8 | B8/T_first B8/rel_last B8/sum_rel')
    for arm in ('p2a', 'p1'):
        for point in order:
            for r in rows:
                if r['arm'] == arm and r['point'] == point:
                    first = int(r['rel_first']) + int(r['rw_first'])
                    n = int(r['B8'])
                    print(arm, point, r['rounds'], first, r['rel_last'], r['final_roles'], r['sum_rel'], n, '|',
                          round(n / first, 1), round(n / int(r['rel_last']), 1), round(n / int(r['sum_rel']), 1))


if __name__ == '__main__':
    if sys.argv[1] == 'validate':
        validate(sys.argv[2], sys.argv[3])
    elif sys.argv[1] == 'calls':
        calls(sys.argv[2])
    elif sys.argv[1] == 'denominators':
        denominators(sys.argv[2])
    else:
        sys.exit('用法见文件头')
