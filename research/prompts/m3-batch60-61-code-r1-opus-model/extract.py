#!/usr/bin/env python3
"""一次 cargo test 输出的摘录，次序与线程先后无关：按测试二进制（`Running` 行）分段，每段依次给
二进制名、排好序的 OPUS 行、每条失败用例的断言消息（按用例名排，去掉线程号与源码位置）、失败名（排好）、test result 行（去掉用时）。
cargo 自己的编译错误行（`error[`、`error:`）原样留。用法：extract.py < 日志。"""
import re
import sys

lines = sys.stdin.read().splitlines()
segments = []
current = None
for line in lines:
    running = re.match(r"\s+Running (.*)$", line)
    if running:
        path = re.search(r"\(([^)]*)\)", running.group(1))
        name = re.sub(r"-[0-9a-f]{16}$", "", (path.group(1) if path else running.group(1)).split("/")[-1])
        current = {"name": name, "lines": []}
        segments.append(current)
    elif current is not None:
        current["lines"].append(line)
    elif line.startswith("error"):
        print(line)


def blocks_of(body):
    blocks, name, kept = {}, None, []
    for line in body:
        header = re.match(r"---- (\S+) stdout ----", line)
        if header:
            if name:
                blocks[name] = kept
            name, kept = header.group(1), []
            continue
        if name is None:
            continue
        if line == "failures:" or line.startswith("test result:"):
            blocks[name] = kept
            name = None
            continue
        if line.startswith("note: run with") or not line.strip():
            continue
        line = re.sub(r"thread '([^']*)' \(\d+\) panicked at [^ ]+:\d+:\d+:", r"thread '\1' panicked:", line)
        kept.append(line)
    if name:
        blocks[name] = kept
    return blocks


for segment in segments:
    body = segment["lines"]
    print(f"binary {segment['name']}")
    # libtest 先打「test 名字 ... 」不换行、跑完才补结果，--nocapture 下替身的第一行接在它后面：从 OPUS 起截
    for line in sorted(line[line.index("OPUS"):] for line in body if "OPUS_" in line):
        print(line)
    for name, kept in sorted(blocks_of(body).items()):
        print(f"  failed {name}")
        for line in kept:
            print(f"    {line}")
    names = sorted({line.strip() for line in body if re.fullmatch(r"    [a-z_0-9:]+", line)})
    for name in names:
        print(f"  failure-name {name}")
    for line in body:
        if line.startswith("test result:"):
            print(re.sub(r"; finished in [0-9.]+s", "", line))
