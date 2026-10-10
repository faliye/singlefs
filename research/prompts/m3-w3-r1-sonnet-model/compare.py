#!/usr/bin/env python3
"""比对：把一次跑出来的结果文件（out 目录）和 expected 目录里记的比。不依赖 SINGLEFS_REPO。
用法：compare.py <out 目录> <expected 目录>
做两件事：
  1. 逐份结果文件去掉 E7TIMING 行与 elapsed_ns 字段、行排序之后取 sha256，与 expected/normalized-sha256.txt 逐行比；
  2. 用 aggregate.py 的口径汇总（每族每候选抓到几个发号事件、无冲突前判红的步数），与 expected/aggregate.txt 逐行比。
退出码：0 全对上；1 有对不上的；2 用法错。
"""
import hashlib
import os
import re
import subprocess
import sys

sys.dont_write_bytecode = True
HERE = os.path.dirname(os.path.realpath(__file__))


def normalized_lines(path):
    lines = []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            line = line.rstrip("\n")
            if line.startswith("E7TIMING"):
                continue
            lines.append(re.sub(r" elapsed_ns=\d+", "", line))
    return sorted(lines)


def digest(path):
    return hashlib.sha256(("\n".join(normalized_lines(path)) + "\n").encode("utf-8")).hexdigest()


def main():
    if len(sys.argv) != 3:
        print(__doc__)
        return 2
    out_dir, expected_dir = sys.argv[1], sys.argv[2]
    problems = 0
    wanted = {}
    with open(os.path.join(expected_dir, "normalized-sha256.txt"), encoding="utf-8") as handle:
        for line in handle:
            if line.strip():
                digest_text, name = line.split()
                wanted[name] = digest_text
    names = sorted(wanted)
    for name in names:
        path = os.path.join(out_dir, name)
        if not os.path.exists(path):
            print(f"没有 {name}")
            problems += 1
            continue
        have = digest(path)
        if have == wanted[name]:
            print(f"同 {name} {have[:16]}")
        else:
            print(f"异 {name} 现 {have[:16]} 预期 {wanted[name][:16]}")
            problems += 1
    aggregate = subprocess.run(
        [sys.executable, "-I", os.path.join(HERE, "aggregate.py")] + [os.path.join(out_dir, n) for n in names if os.path.exists(os.path.join(out_dir, n))],
        capture_output=True, text=True, check=True,
    ).stdout
    aggregate = aggregate.replace(out_dir.rstrip("/") + "/", "")
    with open(os.path.join(expected_dir, "aggregate.txt"), encoding="utf-8") as handle:
        want_aggregate = handle.read()
    if aggregate == want_aggregate:
        print(f"汇总同 {hashlib.sha256(aggregate.encode()).hexdigest()[:16]}（{len(aggregate.splitlines())} 行）")
    else:
        print("汇总异：")
        import difflib
        for line in difflib.unified_diff(want_aggregate.splitlines(), aggregate.splitlines(), "预期", "现", lineterm=""):
            print(line)
        problems += 1
    print(f"对不上 {problems} 处")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
