"""把 crates/mutations.tsv 的第 N 行（文件行号）施加到攻方副本上，或还原。用法：apply_row.py <副本根> <行号> apply|restore"""
import shutil
import sys

root, line_number, action = sys.argv[1], int(sys.argv[2]), sys.argv[3]
with open(f"{root}/crates/mutations.tsv", encoding="utf-8") as table:
    row = table.read().split("\n")[line_number - 1]
name, path, original, replacement, arguments, test_name = row.split("\t")
target = f"{root}/{path}"
backup = target + ".row-orig"
if action == "restore":
    shutil.copyfile(backup, target)
    print("restored", path)
    sys.exit(0)
original = original.replace("\\n", "\n")
replacement = replacement.replace("\\n", "\n")
with open(target, encoding="utf-8") as source:
    text = source.read()
count = text.count(original)
assert count == 1, f"原文命中 {count} 次"
shutil.copyfile(target, backup)
with open(target, "w", encoding="utf-8") as out:
    out.write(text.replace(original, replacement))
print("applied", name)
print("args", arguments)
print("must_red", test_name)
