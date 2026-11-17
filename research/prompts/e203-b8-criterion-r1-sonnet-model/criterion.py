#!/usr/bin/env python3
"""新判据（按这一次碰到的单元算）在 pubs.tsv 上的读数。
用法：criterion.py kinds <pubs.tsv> <臂> <列> [列出最坏的几种]   # 逐种发布（状态/操作/序号）在每根轴上的档
      criterion.py defect <pubs.tsv> <新的tsv>                  # 阳性对照：B8 加上 B1 在调用方段与落点判之前的步数
      criterion.py bigchange <pubs.tsv>                        # data_units 轴上改动大小随轴涨的那一种：每单元读数与局部斜率
      criterion.py offaxis <pubs.tsv>                          # 一单元覆盖写第 1 次的 B8 对活单元总数（16 个点混在一起，登记没有这根轴）

读法（见报告）：分母 T = 第一轮 placements_released + 第一轮 roles_rewritten（下限 1）。
一种发布在轴上 T 的 α >= 0.5 时读 N/T（每单元），否则读原步数 N；每种单独算 α；
每单元那一种另看相邻两点局部斜率，任一 >= 0.5 记「随」。一格的档取各种里最差的。
"""
import csv, math, sys, collections

AXES = {
    'files': (['base', 'files_100', 'files_1000', 'files_10000'], [10, 100, 1000, 10000]),
    'data_units': (['base', 'data_units_100', 'data_units_1000', 'data_units_10000'], [10, 100, 1000, 10000]),
    'unlinks': (['base', 'unlinks_100', 'unlinks_1000', 'unlinks_10000'], [10, 100, 1000, 10000]),
    'U': (['files_1000_unit_area_2_pow_14', 'files_1000', 'files_1000_unit_area_2_pow_20', 'files_1000_unit_area_2_pow_24'], [2 ** 14, 2 ** 17, 2 ** 20, 2 ** 24]),
}


def band(a):
    return 'grows' if a >= 0.5 else ('no' if a <= 0.25 else 'between')


def touched(r):
    return max(1, int(r['rel_first']) + int(r['rw_first']))


def read_rows(path):
    return list(csv.DictReader(open(path), delimiter='\t'))


def kinds(path, arm, col, show):
    by = collections.defaultdict(dict)
    for r in read_rows(path):
        if r['arm'] == arm:
            by[r['point']][(r['state'], r['op'], r['op_index'])] = r
    for axis, (names, pos) in AXES.items():
        common = set.intersection(*[set(by[n]) for n in names])
        counts = collections.Counter()
        worst = []
        per_unit = 0
        for k in sorted(common):
            ts = [touched(by[n][k]) for n in names]
            alpha_t = math.log(ts[-1] / ts[0]) / math.log(pos[-1] / pos[0])
            unit_kind = alpha_t >= 0.5
            per_unit += unit_kind
            ys = [int(by[n][k][col]) / (touched(by[n][k]) if unit_kind else 1) for n in names]
            a = math.log(ys[-1] / ys[0]) / math.log(pos[-1] / pos[0]) if ys[0] > 0 and ys[-1] > 0 else float('nan')
            if unit_kind:
                local = [math.log(ys[i + 1] / ys[i]) / math.log(pos[i + 1] / pos[i]) for i in range(3)]
                if max(local) >= 0.5:
                    a = max(local)
            counts[band(a)] += 1
            worst.append((a, k, ys, ts, unit_kind))
        worst.sort(reverse=True)
        print(f"{arm} {col:6s} {axis:10s} kinds={len(common)} per_unit_kinds={per_unit} {dict(counts)}")
        for a, k, ys, ts, unit_kind in worst[:show]:
            print(f"    a={a:.3f} {k} y={[round(y, 1) for y in ys]} T={ts} per_unit={unit_kind}")


def defect(path, out_path):
    rows = read_rows(path)
    with open(out_path, 'x') as out:
        writer = csv.DictWriter(out, fieldnames=list(rows[0].keys()), delimiter='\t')
        writer.writeheader()
        for r in rows:
            r['B8'] = str(int(r['B8']) + int(r['B1_pre']))
            writer.writerow(r)


def bigchange(path):
    names, pos = AXES['data_units']
    for arm in ('p2a', 'p1'):
        by = {r['point']: r for r in read_rows(path) if r['arm'] == arm and r['state'] == 'same_mount' and r['op'] == 'W2' and r['op_index'] == '1'}
        for col in ('B1', 'B4_pre', 'B4', 'B8'):
            ts = [touched(by[n]) for n in names]
            ys = [int(by[n][col]) / t for n, t in zip(names, ts)]
            local = [math.log(ys[i + 1] / ys[i]) / math.log(pos[i + 1] / pos[i]) for i in range(3)]
            end = math.log(ys[-1] / ys[0]) / math.log(pos[-1] / pos[0])
            print(arm, col, 'T', ts, 'N/T', [round(v, 1) for v in ys], 'end_to_end', round(end, 3), 'local', [round(v, 3) for v in local])


def offaxis(path):
    for arm in ('p2a', 'p1'):
        rows = [r for r in read_rows(path) if r['arm'] == arm and r['state'] == 'same_mount' and r['op'] == 'W1' and r['op_index'] == '1']
        rows = [r for r in rows if r['point'] != 'files_1000_units_100' or arm == 'p2a']
        rows.sort(key=lambda r: int(r['mapped']))
        print(arm)
        for r in rows:
            print('  ', r['point'], 'mapped', r['mapped'], 'B8', r['B8'], 'B4_pre', r['B4_pre'], 'T_last', r['rel_last'])
        for col in ('B8', 'B4_pre'):
            xs = [math.log(int(r['mapped'])) for r in rows]
            ys = [math.log(int(r[col])) for r in rows]
            n = len(xs)
            mx, my = sum(xs) / n, sum(ys) / n
            slope = sum((x - mx) * (y - my) for x, y in zip(xs, ys)) / sum((x - mx) ** 2 for x in xs)
            lo, hi = rows[0], rows[-1]
            end = math.log(int(hi[col]) / int(lo[col])) / math.log(int(hi['mapped']) / int(lo['mapped']))
            print('  ', col, 'OLS slope vs mapped over', n, 'points', round(slope, 3), 'endpoint alpha', round(end, 3))


if __name__ == '__main__':
    cmd = sys.argv[1]
    if cmd == 'kinds':
        kinds(sys.argv[2], sys.argv[3], sys.argv[4], int(sys.argv[5]) if len(sys.argv) > 5 else 0)
    elif cmd == 'defect':
        defect(sys.argv[2], sys.argv[3])
    elif cmd == 'bigchange':
        bigchange(sys.argv[2])
    elif cmd == 'offaxis':
        offaxis(sys.argv[2])
    else:
        sys.exit('用法见文件头')
