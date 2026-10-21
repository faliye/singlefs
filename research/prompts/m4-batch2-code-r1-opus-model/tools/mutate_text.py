"""在攻方副本的一份文件上做一处定点替换（原文必须恰好命中一次），或从备份还原。用法：mutate_text.py <文件> apply <原文> <替换文> | restore"""
import shutil
import sys

path, action = sys.argv[1], sys.argv[2]
backup = path + ".text-orig"
if action == "restore":
    shutil.copyfile(backup, path)
    print("restored", path)
    sys.exit(0)
original = sys.argv[3].replace("\\n", "\n")
replacement = sys.argv[4].replace("\\n", "\n")
with open(path, encoding="utf-8") as source:
    text = source.read()
count = text.count(original)
assert count == 1, f"原文命中 {count} 次"
shutil.copyfile(path, backup)
with open(path, "w", encoding="utf-8") as out:
    out.write(text.replace(original, replacement))
print("applied")
