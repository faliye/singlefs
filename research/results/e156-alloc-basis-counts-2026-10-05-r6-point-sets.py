#!/usr/bin/env python3
# R1c ③ 与 R1d ①②③④：只读产物与命令四、五的输出（不 import 装置）。
import re, sys
main_out, pc_out, pins_txt, lowered_txt = sys.argv[1:5]
def kv(line):
    return dict(m.split('=', 1) for m in re.findall(r'(\w+=\S+)', line))
def rows(path, name):
    for line in open(path, encoding='utf-8'):
        if f' name={name} ' in line or line.rstrip().endswith(f' name={name}'):
            yield kv(line)
# L6：排得下的格里门槛比旧读法低的观测点 × 两个 ρ
l6 = set()
for line in open(lowered_txt):
    if line.startswith('LOWERED ') and 'fits=True' in line:
        f = kv(line)
        for rho in ('1', '1/4'):
            l6.add((f['S'], rho, f['k'], f['pos'], f['txg']))
print(f"L6_points_x_rho={len(l6)}")
def point_sets(path):
    k10, f24 = set(), set()
    for f in rows(path, 'q1_observation'):
        key = (f['s'], f['rho'], f['k'], f['position'], f['txg'])
        if f['family'] != 'Hh':
            continue
        if int(f['k10_violations']) > 0:
            k10.add(key)
        if f['q1a_real'] != f['q1a_every'] or f['q1b_real'] != f['q1b_every']:
            f24.add(key)
    return k10, f24
for label, path in (('main', main_out), ('pc', pc_out)):
    k10, f24 = point_sets(path)
    print(f"{label}: k10_points={len(k10)} f24_points={len(f24)} k10_eq_L6={k10 == l6} f24_eq_L6={f24 == l6} k10_minus_L6={sorted(k10 - l6)} L6_minus_k10={sorted(l6 - k10)} f24_minus_L6={sorted(f24 - l6)} L6_minus_f24={sorted(l6 - f24)}")
# R1d ①：每次挂载钉的 txg 集合 = 命令四 --pins 那一格挂载做完那一行（第一行观测）的 pinned
pins = {}
for line in open(pins_txt):
    if line.startswith('PIN ') and 'fits=True' in line:
        f = kv(line)
        txt = line.split('pinned=', 1)[1].strip()
        pins[(f['S'], f['k'], f['pos'], f['txg'])] = set(re.findall(r'\d+', txt))
recovery_end = {}
for f in rows(main_out, 'hole_recovery'):
    if f['family'] == 'Hh':
        recovery_end[(f['s'], f['rho'], f['k'], f['position'], f['hole_txg'])] = f['last_txg_of_mount']
mismatch, checked, pinned_rows = [], 0, 0
for line in open(main_out, encoding='utf-8'):
    if ' name=all_zero_pins_at_mount ' not in line:
        continue
    f = kv(line)
    if f['family'] != 'Hh':
        continue
    pinned_rows += 1
    got = set(t for _, _, t in re.findall(r'\((\d+),(\d+),(\d+)\)', line.split('pinned=', 1)[1]))
    end = recovery_end[(f['s'], f['rho'], f['k'], f['position'], f['hole_txg'])]
    want = pins.get((f['s'], f['k'], f['position'], end))
    checked += 1
    if want is None or got != want:
        mismatch.append((f['s'], f['rho'], f['k'], f['position'], f['hole_txg'], sorted(got), None if want is None else sorted(want)))
    if f['position'] != 'front' and got:
        mismatch.append(('back-or-k0-pinned', f['s'], f['rho'], f['k'], f['position'], f['hole_txg']))
print(f"R1d_1: hh_mount_rows={pinned_rows} checked={checked} mismatches={mismatch}")
# R1d ②③④
l6_per_cell = {}
for s, rho, k, pos, txg in l6:
    l6_per_cell[(s, rho, k, pos)] = l6_per_cell.get((s, rho, k, pos), 0) + 1
bad2, cells, r3, r4 = [], 0, [], []
for f in rows(main_out, 'all_zero_pin_cell'):
    cells += 1
    key = (f['s'], f['rho'], f['k'], f['position'])
    if int(f['threshold_lowered_points']) != l6_per_cell.get(key, 0):
        bad2.append((key, f['threshold_lowered_points'], l6_per_cell.get(key, 0)))
    if f['position'] != 'front' and int(f['pins_installed']) != 0:
        bad2.append(('pins_on_non_front', key, f['pins_installed']))
    if f['released_by_reread_only'] != '0':
        r3.append(key)
    if f['bad_slots_at_mount'] != '0' or f['bad_slots_still_pinned'] != '0':
        r4.append(key)
print(f"R1d_2: cells={cells} mismatches={bad2}")
print(f"R1d_3: cells_with_reread_release={r3}")
print(f"R1d_4: cells_with_bad_slots={r4}")
