#!/usr/bin/env python3
"""lib_shell_words.py 文件头的 LIB_SHELL_WORDS_BREAK 开关，有几条出现在复跑登记表里某一份登记脚本的文件头开关表里（读法同 count-switches.py，抽 code-tooling.sh 的那一份）。
用法：lsw-coverage.py <根>；打 name=lsw_coverage 一行与没被任何登记脚本列出的那几条。只读。"""
import os
import re
import sys

root = sys.argv[1]
with open(os.path.join(root, ".claude/gate.d/code-tooling.sh"), encoding="utf-8") as handle:
    stage = handle.read()
library = re.search(r"BREAK_SWITCH_TABLE_LIBRARY=\"\$\(cat <<'PY_BREAK_SWITCH_TABLE'\n(.*?)\nPY_BREAK_SWITCH_TABLE\n", stage, re.S).group(1)
namespace = {}
exec(library, namespace)


def values(relative, variable):
    with open(os.path.join(root, relative), encoding="utf-8") as handle:
        return {entry["value"] for entry in namespace["switch_entries"](handle.read()) if entry["name"] == variable}


registered = []
with open(os.path.join(root, ".claude/gate.d/break-switch-replay-files.tsv"), encoding="utf-8") as handle:
    for line in handle:
        if line.strip() and not line.startswith("#"):
            registered.append(line.split("\t")[0])
own = values(".claude/hooks/lib_shell_words.py", "LIB_SHELL_WORDS_BREAK")
covered = {}
for relative in registered:
    if os.path.isfile(os.path.join(root, relative)):
        for value in values(relative, "LIB_SHELL_WORDS_BREAK"):
            covered.setdefault(value, []).append(relative)
missing = sorted(own - set(covered))
print(f"name=lsw_coverage registered_rows={len(registered)} lib_shell_words_switches={len(own)} listed_in_some_registered_header={len(own & set(covered))} not_listed={len(missing)}")
for value in missing:
    print(f"  not_listed LIB_SHELL_WORDS_BREAK={value}")
