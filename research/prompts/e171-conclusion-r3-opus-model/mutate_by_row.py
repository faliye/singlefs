"""e171-conclusion-r3 攻方：按 E171 产物里的行名（空格换成了 _）在副本的 crates/mutations.tsv 里找那一行，打上或还原。
原文在文件里恰好命中一次才改（\\n 是换行）；原文件存成 <文件>.opus-orig。
用法：python3 mutate_by_row.py <副本根> apply <行名>   → 打印改了的文件（相对副本根）
      python3 mutate_by_row.py <副本根> restore <文件>
"""
import os
import sys

root, action, argument = sys.argv[1], sys.argv[2], sys.argv[3]
if action == "restore":
    path = os.path.join(root, argument)
    os.replace(path + ".opus-orig", path)
    sys.exit(0)
hits = []
for line in open(os.path.join(root, "crates/mutations.tsv"), encoding="utf-8"):
    if line.startswith("#") or not line.strip():
        continue
    c = line.rstrip("\n").split("\t")
    if c[0].replace(" ", "_") == argument:
        hits.append(c)
assert len(hits) == 1, f"表里按名字命中 {len(hits)} 行"
file, original, replacement = hits[0][1], hits[0][2].replace("\\n", "\n"), hits[0][3].replace("\\n", "\n")
path = os.path.join(root, file)
text = open(path, encoding="utf-8").read()
assert text.count(original) == 1, f"原文在文件里命中 {text.count(original)} 次"
os.replace(path, path + ".opus-orig")
with open(path, "w", encoding="utf-8") as handle:
    handle.write(text.replace(original, replacement, 1))
print(file)
