#!/usr/bin/env python3
"""把 mutations.tsv 里变异名以给定前缀开头的行逐条施加到副本、跑点名的 harness 档测试、还原。
用法 mutate_one.py <副本根> <变异名前缀> <cargo 的 -p 与 --test 参数之后的过滤串由表里给>
只跑 harness 档（-p singlefs-harness）的行；每条输出 `MUT <名前 40 字> exit=<码> <test result 行（去耗时）>`。
命令包装由环境变量 MUTATE_CMD_PREFIX 给（编译与跑都经 run-with-memory-cap.sh）。"""
import os
import re
import subprocess
import sys

root, prefix = sys.argv[1], sys.argv[2]
cmd_prefix = os.environ["MUTATE_CMD_PREFIX"]
table = open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8").read().split("\n")
rows = [line for line in table if line.split("\t")[0].startswith(prefix) and len(line.split("\t")) == 6]
print(f"MUT_ROWS {len(rows)} prefix={prefix[:30]}")
for row in rows:
    name, source, find, replacement, arguments, test_name = row.split("\t")
    assert "-p singlefs-harness" in arguments, arguments
    path = os.path.join(root, source)
    original = open(path, encoding="utf-8").read()
    find_text, replacement_text = find.replace("\\n", "\n"), replacement.replace("\\n", "\n")
    assert original.count(find_text) == 1, name
    open(path, "w", encoding="utf-8").write(original.replace(find_text, replacement_text))
    try:
        completed = subprocess.run(
            f"{cmd_prefix} cargo test --offline --release {arguments}",
            shell=True, cwd=root, capture_output=True, text=True,
        )
    finally:
        open(path, "w", encoding="utf-8").write(original)
    lines = (completed.stdout + completed.stderr).split("\n")
    result = [re.sub(r"; finished in .*", "", line) for line in lines if line.startswith("test result")]
    print(f"MUT {name[:40]} exit={completed.returncode} {' '.join(result)}")
