#!/usr/bin/env python3
"""defs-m2-closeout-r3 云端攻方 K2 / G5：照执行员第 4c 步改后的字面，把 research/results/ 里今天的全部判决行（grep 'name=verdict'）的每个字段分边。
  点名：甲 取值 false；乙 取值 not_run；丙 取值是大于 0 的整数、名字带 violation / mismatch / ambigu / fail（名字表示违例、不匹配、歧义、失败）。
  不点名：丁 取值 true；戊 取值 zero；己 取值是大于 0 的整数、名字不带那几个词；庚 取值是 0。
  字面没说的：辛 其余取值（字符串、小数、比值……），打出来交人判。
每个字段名打一行：边、取值、出现几处、例（文件:行）。只读 research/results/。
用法（仓根下）：python3 research/prompts/defs-m2-closeout-r3-opus-model/verdict-fields.py"""
import glob, os, re

ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
FAILURE_NAME = re.compile(r"violation|mismatch|ambigu|fail")
seen = {}
lines = 0
for path in sorted(glob.glob(os.path.join(ROOT, "research/results/*"))):
    if not os.path.isfile(path):
        continue
    with open(path, encoding="utf-8", errors="replace") as handle:
        for number, line in enumerate(handle, 1):
            if "name=verdict" not in line:
                continue
            lines += 1
            for field, value in re.findall(r"(?<![^\s])([A-Za-z_][A-Za-z0-9_]*)=(\S+)", line):
                if field == "name":
                    continue
                if value == "false": side = "甲 点名"
                elif value == "not_run": side = "乙 点名"
                elif value == "true": side = "丁 不点名"
                elif value == "zero": side = "戊 不点名"
                elif re.fullmatch(r"-?\d+", value):
                    if int(value) == 0: side = "庚 不点名"
                    elif int(value) > 0 and FAILURE_NAME.search(field): side = "丙 点名"
                    elif int(value) > 0: side = "己 不点名"
                    else: side = "辛 字面没说"
                else: side = "辛 字面没说"
                key = (side, field, value if side[0] in "甲乙丁戊庚" else ("<整数>" if side[0] in "丙己" else value))
                entry = seen.setdefault(key, [0, f"{os.path.relpath(path, ROOT)}:{number}"])
                entry[0] += 1
print(f"判决行 {lines} 条")
for (side, field, value), (count, example) in sorted(seen.items()):
    print(f"{side}\t{field}={value}\t{count} 处\t例 {example}")
