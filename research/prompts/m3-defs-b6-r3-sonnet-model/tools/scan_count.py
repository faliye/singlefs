#!/usr/bin/env python3
"""复刻 code-tooling.sh heavy-thread-defaults 格的 scanned_files()（SCAN_DIRECTORIES、SCAN_FILES、SKIPPED_NAMES、SKIPPED_PREFIXES、LANGUAGE_OF_SUFFIX 取自冻结副本 code-tooling.sh 第 1987–1992 行），
只数文件，不判写法。用法：scan_count.py <冻结副本根>。输出：扫了 N 份（目录 n、…）与排除表数据行数。"""
import os, sys, collections
root = sys.argv[1]
os.chdir(root)
SCAN_DIRECTORIES = ("scripts", ".claude/gate.d", ".claude/hooks", "crates")
SCAN_FILES = (".claude/cargo-command-prefix",)
SKIPPED_NAMES = {"target", "__pycache__", ".git"}
SKIPPED_PREFIXES = (".claude/gate.d/fixtures/",)
LANGUAGE_OF_SUFFIX = {".sh": "shell", ".py": "python", ".rs": "rust"}
by = collections.Counter(); n = 0
for directory in SCAN_DIRECTORIES:
    if not os.path.isdir(directory): continue
    for current, directories, files in os.walk(directory):
        directories[:] = sorted(name for name in directories if name not in SKIPPED_NAMES
                                and not (os.path.join(current, name) + "/").startswith(SKIPPED_PREFIXES))
        for name in sorted(files):
            if os.path.splitext(name)[1] in LANGUAGE_OF_SUFFIX:
                by[directory] += 1; n += 1
for path in SCAN_FILES:
    if os.path.isfile(path): by[path] += 1; n += 1
rows = [l for l in open(".claude/heavy-thread-exempt", encoding="utf-8") if l.strip() and not l.lstrip().startswith("#")]
print(f"扫了 {n} 份（" + "、".join(f"{k} {by[k]}" for k in (*SCAN_DIRECTORIES, *SCAN_FILES) if by[k]) + f"）；排除表数据行 {len(rows)}")
