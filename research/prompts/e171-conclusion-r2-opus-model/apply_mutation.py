"""在腿的仓副本上打一条 crates/mutations.tsv 的变异（原文恰好命中一次才改；\\n 是换行），或还原。
用法：python3 apply_mutation.py <副本根> <mutations.tsv> <文件> <变异名>    （打上，原文件存成 <文件>.opus-orig）
      python3 apply_mutation.py <副本根> --restore <文件>
"""
import os
import sys

root = sys.argv[1]
if sys.argv[2] == "--restore":
    path = os.path.join(root, sys.argv[3])
    os.replace(path + ".opus-orig", path)
    print(f"restored {sys.argv[3]}")
    sys.exit(0)
table, file, name = sys.argv[2], sys.argv[3], sys.argv[4]
hits = []
for line in open(table, encoding="utf-8"):
    if line.startswith("#") or not line.strip():
        continue
    c = line.rstrip("\n").split("\t")
    if c[0] == name and c[1] == file:
        hits.append(c)
assert len(hits) == 1, f"表里按 (名字, 文件) 命中 {len(hits)} 行"
original, replacement = hits[0][2].replace("\\n", "\n"), hits[0][3].replace("\\n", "\n")
path = os.path.join(root, file)
text = open(path, encoding="utf-8").read()
assert text.count(original) == 1, f"原文在文件里命中 {text.count(original)} 次"
os.replace(path, path + ".opus-orig")
with open(path, "w", encoding="utf-8") as handle:
    handle.write(text.replace(original, replacement, 1))
print(f"mutated {file} :: {name}")
