#!/usr/bin/env python3
# E173 第一次重跑第二段的 Z2（重跑登记 research/prompts/e173-r1-prereg.md 第六节 Z2）：只读产物，不读装置。
# 对每个 (Y1 的行, Q1 臂, S)：原两份产物（2026-10-05 的第一段、第二段，已归档，从 git 里读）里 reserve=code 的判定，
# 与新模式产物里 reserve=ckpt-code 的判定；两边各报逐取样点（原：c_max 5 / 10；新：每档池大小）的打中，再报「任一取样点打中」的或，
# 一真一假的记「变了」。条款读法同样比（reserve=clause 对 reserve=ckpt-clause）。
# Q1 那几行取 name=judgment 的 hit=；Q2 那几行取 name=q2_comparison 的四个 hit_* 字段（行名带范围 @z_h1_to_h4 / @h5_holes）。
# 用法：python3 <这份脚本> <新模式产物>
import collections
import subprocess
import sys

ARCHIVED_AT = '59ce5a865^'
OLD_PRODUCTS = [
    'research/results/e173-admission-ceiling-bad-ring-slot-dispositions-segment-one-2026-10-05.out',
    'research/results/e173-admission-ceiling-bad-ring-slot-dispositions-segment-two-2026-10-05.out',
]
Q2_FIELDS = {
    'hit_not_merged_has_more': 'Q2-1-not-merged-has-more',
    'hit_merged_has_more': 'Q2-1-merged-has-more',
    'hit_merged_all_enospc': 'Q2-2-merged-all-enospc',
    'hit_merged_all_enospc_beyond_not_merged': 'Q2-2-merged-all-enospc-beyond-not-merged',
}
READING_PAIRS = [('code', 'ckpt-code'), ('clause', 'ckpt-clause')]


def parse(line):
    return dict(token.split('=', 1) for token in line.strip().split(' ')[1:] if '=' in token)


def hits_of(lines, sampling_point_field):
    # (读法, 行, Q1 臂, S) → {取样点: 打中}
    hits = collections.defaultdict(dict)
    for line in lines:
        if not line.startswith('E7RESULT '):
            continue
        row = parse(line)
        name = row.get('name')
        if name == 'judgment':
            point = row.get(sampling_point_field)
            if point is None:
                continue
            hits[(row['reserve'], row['id'], row['arm'], row['S'])][point] = row['hit'] == 'yes'
        elif name == 'q2_comparison':
            point = row.get(sampling_point_field)
            if point is None:
                continue
            for field, label in Q2_FIELDS.items():
                hits[(row['reserve'], f"{label}@{row['scope']}", row['arm'], row['S'])][point] = row[field] == 'yes'
    return hits


def main():
    if len(sys.argv) != 2:
        print('用法：python3 <这份脚本> <新模式产物>', file=sys.stderr)
        return 2
    old_lines = []
    for path in OLD_PRODUCTS:
        text = subprocess.run(['git', 'show', f'{ARCHIVED_AT}:{path}'], check=True, capture_output=True, text=True).stdout
        old_lines.extend(text.splitlines())
    with open(sys.argv[1], encoding='utf-8') as product:
        new_lines = [line for line in product.read().splitlines() if ' segment=ckpt-first ' in line or ' segment=ckpt-second ' in line]
    old_hits = hits_of(old_lines, 'c_max')
    new_hits = hits_of(new_lines, 'pool_size')
    changed = collections.Counter()
    compared = collections.Counter()
    missing = []
    for old_reading, new_reading in READING_PAIRS:
        keys = sorted({key[1:] for key in old_hits if key[0] == old_reading} | {key[1:] for key in new_hits if key[0] == new_reading})
        for row_label, arm, slots_per_region in keys:
            old = old_hits.get((old_reading, row_label, arm, slots_per_region))
            new = new_hits.get((new_reading, row_label, arm, slots_per_region))
            if old is None or new is None:
                missing.append(f'{old_reading}/{new_reading} {row_label} {arm} S={slots_per_region} old={old is not None} new={new is not None}')
                continue
            old_any, new_any = any(old.values()), any(new.values())
            compared[old_reading] += 1
            changed[old_reading] += int(old_any != new_any)
            old_points = ','.join(f'{point}:{"yes" if hit else "no"}' for point, hit in sorted(old.items(), key=lambda item: int(item[0])))
            new_points = ','.join(f'{point}:{"yes" if hit else "no"}' for point, hit in sorted(new.items()))
            print(f'Z2 old={old_reading} new={new_reading} id={row_label} arm={arm} S={slots_per_region} '
                  f'old_points={old_points} new_points={new_points} old_any={"yes" if old_any else "no"} new_any={"yes" if new_any else "no"} '
                  f'changed={"yes" if old_any != new_any else "no"}')
    for line in missing:
        print(f'Z2_unpaired {line}')
    for old_reading, new_reading in READING_PAIRS:
        print(f'Z2_summary old={old_reading} new={new_reading} compared={compared[old_reading]} changed={changed[old_reading]} '
              f'verdict={"changed" if changed[old_reading] else "unchanged"}')
    print(f'Z2_done unpaired={len(missing)}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
