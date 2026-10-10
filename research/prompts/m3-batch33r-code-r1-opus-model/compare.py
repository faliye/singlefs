# 用法：python3 compare.py <输出目录甲> <输出目录乙>   比两次 rerun.sh 的 *.out：只比 name= 开头的行，去掉耗时与路径字段后排序逐行比；不依赖 SINGLEFS_REPO
"""m3-batch33r-code-r1 攻方腿（Opus）原型两次复跑的比对。

去掉的字段：seconds=、ledger=（账本的绝对路径随输出目录变）、first_violation= 不去（两次都应是 None）。
build.out 不比（编译输出随机器变）。退出码：0 全部相同；1 有不同（逐个打出来）；2 用法错或甲里一份 .out 都没有。
"""
import pathlib
import re
import sys

DROPPED_FIELDS = re.compile(r" (seconds|ledger)=\S+")


def name_lines(path: pathlib.Path) -> list[str]:
    lines = []
    for line in path.read_text(encoding="utf-8", errors="replace").splitlines():
        if line.startswith("name="):
            lines.append(DROPPED_FIELDS.sub("", line))
    return sorted(lines)


def main() -> int:
    if len(sys.argv) != 3:
        print(__doc__.splitlines()[0], file=sys.stderr)
        return 2
    first, second = (pathlib.Path(argument) for argument in sys.argv[1:])
    outputs = sorted(path.name for path in first.glob("*.out") if path.name != "build.out")
    if not outputs:
        print(f"{first} 里没有 .out", file=sys.stderr)
        return 2
    differing = 0
    for name in outputs:
        other = second / name
        if not other.exists():
            print(f"✗ {name}：乙里没有")
            differing += 1
            continue
        mine, theirs = name_lines(first / name), name_lines(other)
        if mine != theirs:
            differing += 1
            print(f"✗ {name}：不同")
            for line in sorted(set(mine) ^ set(theirs))[:10]:
                print(f"    {'甲' if line in mine else '乙'}: {line}")
        else:
            print(f"✓ {name}：{len(mine)} 行相同")
    print(f"compared={len(outputs)} differing={differing}")
    return 1 if differing else 0


if __name__ == "__main__":
    sys.exit(main())
