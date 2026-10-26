#!/usr/bin/env python3
"""从拼起来的 WGSL 源码里取一个函数及它传递依赖到的顶层定义（fn / struct / const / var / alias），写成一个小模块；够不到的名字由校验程序补常量。
用法：closure.py <源码.wgsl> <函数名> <输出.wgsl>"""
import re
import sys

source = open(sys.argv[1], encoding="utf-8").read()
root_name = sys.argv[2]
# 去注释
source = re.sub(r"//[^\n]*", "", source)
items = {}
position = 0
header = re.compile(r"(?m)^(fn|struct|const|alias|var(?:<[^>]*>)?|override)\s+([A-Za-z_][A-Za-z0-9_]*)")
while True:
    match = header.search(source, position)
    if not match:
        break
    start = match.start()
    depth = 0
    index = match.end()
    while index < len(source):
        char = source[index]
        if char in "({[":
            depth += 1
        elif char in ")}]":
            depth -= 1
            if depth == 0 and char == "}" and match.group(1) in ("fn", "struct"):
                index += 1
                break
        elif char == ";" and depth == 0:
            index += 1
            break
        index += 1
    items[match.group(2)] = source[start:index]
    position = index
wanted = [root_name]
seen = set()
order = []
while wanted:
    name = wanted.pop()
    if name in seen or name not in items:
        continue
    seen.add(name)
    order.append(name)
    for token in set(re.findall(r"[A-Za-z_][A-Za-z0-9_]*", items[name])):
        if token in items and token not in seen:
            wanted.append(token)
# 定义次序：按在源码里出现的先后
order.sort(key=lambda name: source.index(items[name]))
open(sys.argv[3], "w", encoding="utf-8").write("\n".join(items[name] for name in order) + "\n")
print(f"closure of {root_name}: {len(order)} items")
