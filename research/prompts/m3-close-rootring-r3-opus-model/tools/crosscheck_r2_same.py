"""第二轮攻方 `out/same.out` 与这一轮 `out/narrow.out` 逐格比：同一格（形、族、k、崩溃点、故障）上 core 的 F2 交回哪一条、第二轮
「同实例一律收」交回哪一条、两种规则下恢复的结局，两份是否一字不差。用法：crosscheck_r2_same.py <narrow.out> <第二轮 same.out>。
打一行 `name=r2_same_out_crosscheck lines=<第二轮的格数> agree=<对上的格数>`；有对不上的先逐行打 `DIFF`，退 1。"""
import re
import sys


def main():
    mine = {}
    with open(sys.argv[1], encoding='utf-8') as narrow:
        for line in narrow:
            if not line.startswith('R3NARROW shape=') or 'shape=sweep' in line:
                continue
            head = line.split(' chosen=')[0].replace('R3NARROW ', '')
            mine[head] = (
                re.search(r'exact: f2=(\S+)', line).group(1),
                re.search(r'same: f2=(\S+)', line).group(1),
                re.search(r'exact: f2=\S+ \S+ (rec=\S+)', line).group(1),
                re.search(r'same: f2=\S+ \S+ (rec=\S+)', line).group(1),
            )
    lines = agree = 0
    with open(sys.argv[2], encoding='utf-8') as same:
        for line in same:
            if not line.startswith('R2SAME shape='):
                continue
            head = line.split(' chosen=')[0].replace('R2SAME ', '')
            theirs = (
                re.search(r'f2_core=(\S+)', line).group(1),
                re.search(r'f2_same_instance=(\S+)', line).group(1),
                re.search(r'own/exact: (rec=\S+)', line).group(1),
                re.search(r'own/same: (rec=\S+)', line).group(1),
            )
            lines += 1
            if mine.get(head) == theirs:
                agree += 1
            else:
                print('DIFF', head, theirs, mine.get(head))
    print(f'name=r2_same_out_crosscheck lines={lines} agree={agree}')
    return 0 if lines == agree and lines > 0 else 1


if __name__ == '__main__':
    sys.exit(main())
