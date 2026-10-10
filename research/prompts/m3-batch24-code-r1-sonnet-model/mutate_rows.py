#!/usr/bin/env python3
"""把 mutations.tsv 里「第24批 K …」且点名 -p singlefs-harness 的行逐条施加到副本、跑点名的测试、还原，记红在哪条断言。
用法 mutate_rows.py <副本根> <从第几条> <到第几条（不含）>   （条号是「第24批 K」行里 harness 档那些的 0 起序号）
命令包装由环境变量 MUTATE_CMD_PREFIX 给（编译与跑都经 run-with-memory-cap.sh）。
每条输出：MUT <序号> <名前 36 字> exit=<码> <test result 行（去耗时）> | FIRST_PANIC <断言消息头 160 字>。
恢复：原文件内容在 try/finally 里写回，写回后再核 sha256 与施加前相同（不同输出 RESTORE_MISMATCH）。"""
import hashlib
import os
import re
import subprocess
import sys

root, lo, hi = sys.argv[1], int(sys.argv[2]), int(sys.argv[3])
prefix = os.environ["MUTATE_CMD_PREFIX"]
table = open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8").read().split("\n")
rows = [l for l in table if l.startswith("第24批 K") and len(l.split("\t")) == 6 and "-p singlefs-harness" in l.split("\t")[4]]
print(f"MUT_ROWS_HARNESS {len(rows)}")
for index, row in enumerate(rows):
    if not lo <= index < hi:
        continue
    name, source, find, replacement, arguments, test_name = row.split("\t")
    path = os.path.join(root, source)
    original = open(path, encoding="utf-8").read()
    digest = hashlib.sha256(original.encode()).hexdigest()
    find_text, replacement_text = find.replace("\\n", "\n"), replacement.replace("\\n", "\n")
    assert original.count(find_text) == 1, name
    open(path, "w", encoding="utf-8").write(original.replace(find_text, replacement_text))
    try:
        completed = subprocess.run(f"{prefix} cargo test --offline {arguments}", shell=True, cwd=root, capture_output=True, text=True)
    finally:
        open(path, "w", encoding="utf-8").write(original)
    if hashlib.sha256(open(path, "rb").read()).hexdigest() != digest:
        print("RESTORE_MISMATCH", name)
    text = completed.stdout + completed.stderr
    result = [re.sub(r"; finished in .*", "", l) for l in text.split("\n") if l.startswith("test result")]
    panic = ""
    lines = text.split("\n")
    for i, l in enumerate(lines):
        if "panicked at" in l:
            # 断言消息取开头 70 字；left / right 两行（违例集合）各取 150 字：红在哪条断言、红成什么样的证据
            head = lines[i + 1].strip()[:70]
            sides = [x.strip()[:150] for x in lines[i + 1 : i + 12] if x.strip().startswith(("left:", "right:"))]
            panic = " ".join([head] + sides)
            break
    print(f"MUT {index} {name[:36]} exit={completed.returncode} {' '.join(result)} | FIRST_PANIC {panic}")
