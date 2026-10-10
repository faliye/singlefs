#!/usr/bin/env python3
"""expected/ 与 out/ 逐份比：每份排序、去耗时字段（elapsed_seconds=…、finished in …）之后逐行相同记「一致」。缺的、多的、不同的都记「不一致」，退出 1。"""
import re
import sys
from pathlib import Path


def normalized(path: Path) -> list[str]:
    lines = []
    for line in path.read_text().splitlines():
        line = re.sub(r" elapsed_seconds=[0-9.]+", "", line)
        line = re.sub(r"; finished in [0-9.]+s", "", line)
        lines.append(line)
    return sorted(lines)


def main() -> int:
    expected, out = Path(sys.argv[1]), Path(sys.argv[2])
    bad = 0
    for path in sorted(expected.glob("*.txt")):
        mine = out / path.name
        if not mine.exists():
            print(f"不一致 {path.name}：out 里没有")
            bad += 1
        elif normalized(path) == normalized(mine):
            print(f"一致 {path.name}")
        else:
            print(f"不一致 {path.name}")
            bad += 1
    extra = sorted(p.name for p in out.glob("*.txt") if not (expected / p.name).exists())
    for name in extra:
        print(f"不一致 {name}：expected 里没有")
        bad += 1
    print(f"compare：{len(list(expected.glob('*.txt')))} 份，不一致 {bad} 份")
    return 1 if bad else 0


if __name__ == "__main__":
    sys.exit(main())
