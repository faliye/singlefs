#!/usr/bin/env python3
# E173 第一次重跑第二段的 F7 / 第八节「几何敏感性」（重跑登记 research/prompts/e173-r1-prereg.md 第八节、第十节 F7，修订 16）：只读新模式产物，不读装置。
# 对每个 (读法, Y1 的行, Q1 臂)：判定在 (S, 池大小) 上的打中（判定行的 hit= 与 Q2 比较行的四个 hit_* 字段，取法同 Z2 脚本），
# 比两组：同一个 S 上 G-min 对 G-4；同一档池大小上 S = 4 对 S = 16。一真一假的记「不稳定」（登记第八节「判决不稳定的记法」），两点都报。
# 只比 reserve=ckpt-code 与 reserve=ckpt-clause 两组（改回 10 + 7c 的对照臂不进判定行）。
# 用法：python3 <这份脚本> <新模式产物>
import collections
import sys

Q2_FIELDS = {
    'hit_not_merged_has_more': 'Q2-1-not-merged-has-more',
    'hit_merged_has_more': 'Q2-1-merged-has-more',
    'hit_merged_all_enospc': 'Q2-2-merged-all-enospc',
    'hit_merged_all_enospc_beyond_not_merged': 'Q2-2-merged-all-enospc-beyond-not-merged',
}
READINGS = ('ckpt-code', 'ckpt-clause')


def parse(line):
    return dict(token.split('=', 1) for token in line.strip().split(' ')[1:] if '=' in token)


def pool_kind(pool_size):
    # g_min_64mib / g_min_128mib 都记 g_min：G-min 在不同 S 上取到的盘大小不同，比的是「最小那一档」这个位置
    return 'g_min' if pool_size.startswith('g_min_') else pool_size


def hits_of(lines):
    # (读法, 行, Q1 臂) → {(S, 池档): 打中}
    hits = collections.defaultdict(dict)
    for line in lines:
        if not line.startswith('E7RESULT '):
            continue
        row = parse(line)
        name = row.get('name')
        if 'pool_size' not in row or row.get('reserve') not in READINGS:
            continue
        point = (int(row['S']), pool_kind(row['pool_size']))
        if name == 'judgment':
            hits[(row['reserve'], row['id'], row['arm'])][point] = row['hit'] == 'yes'
        elif name == 'q2_comparison':
            for field, label in Q2_FIELDS.items():
                hits[(row['reserve'], f"{label}@{row['scope']}", row['arm'])][point] = row[field] == 'yes'
    return hits


def pairs():
    for slots in (4, 8, 16):
        yield ('pool_size_g_min_against_g_4', f'S={slots}', (slots, 'g_min'), (slots, 'g_4'))
    for pool in ('g_min', 'g_1', 'g_4'):
        yield ('slots_4_against_16', f'pool={pool}', (4, pool), (16, pool))


def main():
    if len(sys.argv) != 2:
        print('用法：python3 <这份脚本> <新模式产物>', file=sys.stderr)
        return 2
    with open(sys.argv[1], encoding='utf-8') as product:
        lines = product.read().splitlines()
    hits = hits_of(lines)
    compared = collections.Counter()
    unstable = collections.Counter()
    missing = []
    for (reading, row_label, arm), by_point in sorted(hits.items()):
        for axis, where, left, right in pairs():
            if left not in by_point or right not in by_point:
                missing.append(f'{reading} {row_label} {arm} {axis} {where} left={left in by_point} right={right in by_point}')
                continue
            differs = by_point[left] != by_point[right]
            compared[(reading, axis)] += 1
            unstable[(reading, axis)] += int(differs)
            print(f'F7 reading={reading} id={row_label} arm={arm} axis={axis} {where} left={left[0]}/{left[1]}:{"yes" if by_point[left] else "no"} '
                  f'right={right[0]}/{right[1]}:{"yes" if by_point[right] else "no"} unstable={"yes" if differs else "no"}')
    for line in missing:
        print(f'F7_unpaired {line}')
    for reading in READINGS:
        for axis in ('pool_size_g_min_against_g_4', 'slots_4_against_16'):
            print(f'F7_summary reading={reading} axis={axis} compared={compared[(reading, axis)]} unstable={unstable[(reading, axis)]}')
    print(f'F7_done rows={len(hits)} unpaired={len(missing)}')
    return 0


if __name__ == '__main__':
    sys.exit(main())
