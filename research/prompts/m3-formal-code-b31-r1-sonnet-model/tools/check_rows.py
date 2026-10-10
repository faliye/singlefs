#!/usr/bin/env python3
"""N5：第 31 批新加与替换的变异行（crates/mutations.tsv 里 C551 起头的 8 行、E171R1 装置 V32–V36 与 U12 那 7 行、
替换了点名测试的 m4b 那一行），在冻结树上静态核：
  ① 原文（\\n 还原成换行）在文件里恰好命中一次；
  ② 点名的测试是不是一个 fn、上面有 #[test]、有没有 #[ignore]（有的话参数里要带 --include-ignored）；
  ③ `--` 之后的筛选词是不是点名测试名的子串；
  ④ 同名 fn 在那个包里有几处；
  ⑤ 与表里已有的任一行（文件, 原文, 替换文, 点名的测试）逐字相同的；只看原文与替换文相同的行对（点名的测试不同）也列出。
只读，不跑任何测试。用法：check_rows.py <树根>
"""
import os
import re
import sys

root = sys.argv[1]
table = os.path.join(root, "crates", "mutations.tsv")
rows = []
for number, line in enumerate(open(table, encoding="utf-8").read().split("\n"), 1):
    if not line or line.startswith("#"):
        continue
    fields = line.split("\t")
    rows.append((number, fields))

def is_new(fields):
    name = fields[0]
    return (
        name.startswith("C551")
        or name.startswith("E171R1 装置 V3")
        or name.startswith("E171R1 装置 U12")
        or "攻方变异 m4b 同一处" in name
    )

def source_files_of(arguments):
    package = re.search(r"-p (\S+)", arguments).group(1)
    base = os.path.join(root, "crates", package)
    test = re.search(r"--test (\S+)", arguments)
    binary = re.search(r"--bin (\S+)", arguments)
    if test:
        candidates = [os.path.join(base, "tests", test.group(1) + ".rs")]
        # tests/ 下的 common 模块被各测试文件 #[path] 或 mod 带进来：只看那个测试文件自己
        return candidates
    if binary:
        return [os.path.join(base, "src", "bin", binary.group(1) + ".rs")]
    found = []
    for directory, _, files in os.walk(os.path.join(base, "src")):
        for file in files:
            if file.endswith(".rs"):
                found.append(os.path.join(directory, file))
    return found

seen = {}
problems = 0
checked = 0
for number, fields in rows:
    key = (fields[1], fields[2], fields[3], fields[5])
    if key in seen and is_new(fields):
        print(f"DUP row {number} = row {seen[key]} (文件, 原文, 替换文, 点名的测试)")
        problems += 1
    seen.setdefault(key, number)
same_change = {}
for number, fields in rows:
    same_change.setdefault((fields[1], fields[2], fields[3]), []).append((number, fields[5]))

for number, fields in rows:
    if not is_new(fields):
        continue
    checked += 1
    name, path, original, replacement, arguments, test = fields[:6]
    source = open(os.path.join(root, path), encoding="utf-8").read()
    hits = source.count(original.replace("\\n", "\n"))
    stray = [m.group(0) for text in (original, replacement) for m in re.finditer(r"\\(.)", text) if m.group(1) != "n"]
    files = source_files_of(arguments)
    definitions = []
    for file in files:
        text = open(file, encoding="utf-8").read()
        for m in re.finditer(r"((?:[ \t]*#\[[^\n]*\n|[ \t]*//[^\n]*\n)*)[ \t]*(?:pub\s+)?fn\s+" + re.escape(test) + r"\s*\(", text):
            definitions.append((os.path.relpath(file, root), m.group(1)))
    after = arguments.split(" -- ", 1)[1] if " -- " in arguments else ""
    words = [w for w in after.split() if not w.startswith("--")]
    ignored = any("#[ignore" in attrs for _, attrs in definitions)
    has_test_attribute = all("#[test]" in attrs for _, attrs in definitions)
    filter_ok = all(w in test for w in words)
    status = []
    if hits != 1:
        status.append(f"ANCHOR 命中 {hits} 次")
    if stray:
        status.append(f"STRAY {stray}")
    if len(definitions) != 1:
        status.append(f"fn {test} 在 {len(definitions)} 处")
    if not has_test_attribute:
        status.append("没有 #[test]")
    if ignored and "--include-ignored" not in arguments:
        status.append("#[ignore] 而没带 --include-ignored")
    if not filter_ok:
        status.append(f"筛选词 {words} 不是 {test} 的子串")
    twins = [n for n, t in same_change[(path, original, replacement)] if n != number]
    print(
        f"row {number}: {'OK' if not status else 'PROBLEM ' + '; '.join(status)} | anchor_hits={hits} defs={[d[0] for d in definitions]} "
        f"ignore={ignored} twins_same_change_other_test={twins} | {name[:48]}"
    )
    problems += bool(status)
print(f"CHECKED {checked} rows, problems={problems}")
