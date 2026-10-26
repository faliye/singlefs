#!/usr/bin/env python3
"""apply_mutation.py <变异表> <名> <树根>：把表里那一行的原文在树里那份文件中换成变异文，原文要恰好命中一次。
表的列：名、说明、文件、原文、变异文；原文与变异文里 \\n 是换行、\\t 是制表符、\\\\ 是反斜杠，别的「\\x」取 x。
名为 PROBE 的那一行另在 transaction.rs 末尾加一个全局计数（探针读它）。"""
import sys
from pathlib import Path

def unescape(text):
    out, i = [], 0
    while i < len(text):
        if text[i] == "\\" and i + 1 < len(text):
            out.append({"n": "\n", "t": "\t"}.get(text[i + 1], text[i + 1])); i += 2
        else:
            out.append(text[i]); i += 1
    return "".join(out)

table, name, root = sys.argv[1], sys.argv[2], Path(sys.argv[3])
for line in Path(table).read_text(encoding="utf-8").splitlines():
    fields = line.split("\t")
    if fields[0] != name:
        continue
    path = root / fields[2]
    old, new = unescape(fields[3]), unescape(fields[4])
    text = path.read_text(encoding="utf-8")
    hits = text.count(old)
    if hits != 1:
        sys.exit(f"{name}: 原文在 {path} 里命中 {hits} 次，不是 1 次")
    text = text.replace(old, new)
    if name == "PROBE":
        text += "\n/// 攻方探针（m4-publish-lookup-followup-code-r2）：第一个文件照抄单元时在 rewritten 上比较的次数。\npub static ATTACK_R2_FIRST_FILE_CONTAINS_COMPARISONS: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);\n"
    path.write_text(text, encoding="utf-8")
    print(f"applied {name} {fields[2]}")
    sys.exit(0)
sys.exit(f"{name}: 表里没有这一行")
