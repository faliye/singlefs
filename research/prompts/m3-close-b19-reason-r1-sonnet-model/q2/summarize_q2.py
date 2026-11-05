"""q2_scale{1,4}.txt 与 q2_defense_port.txt 的汇总：把计数换成百分比，按 (变体, 写者, S, J, 种类) 一行一个模式。只读，不跑模型。"""
import re, sys, collections

LINE = re.compile(r"steps_scale=(\d+) genesis=(\d) writer=(\w+) S=(\d) J=(\d+) (legal|illegal)_raise_roots=(\d+) mode=\((\d), (\d), (\d)\) \[(.*?)\] holds=(\d+) partial=(\d+) red=(\d+) none=(\d+)")


def main():
    paths = sys.argv[1:]
    for path in paths:
        if 'defense' in path:
            continue
        for line in open(path):
            m = LINE.match(line)
            if not m:
                continue
            scale, genesis, writer, S, J, kind, n, a, b, c, name, holds, partial, red, none = m.groups()
            if J != '128':
                continue
            n, holds, partial, red, none = map(int, (n, holds, partial, red, none))
            print(f"scale={scale} genesis={genesis} writer={writer} S={S} J={J} {kind} n={n} mode=({a},{b},{c}) [{name}] partial={100 * partial / n:.1f}% red={100 * red / n:.1f}% holds={100 * holds / n:.1f}%")


if __name__ == '__main__':
    main()
