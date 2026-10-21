import sys
# 用法：apply_mutation.py <副本根> <crates/mutations.tsv 里的行名片段>   （原文/替换文里的 \n 表示换行；原文必须恰好命中一次）
root, fragment = sys.argv[1], sys.argv[2]
rows = [l.rstrip("\n").split("\t") for l in open(root + "/crates/mutations.tsv", encoding="utf-8") if fragment in l.split("\t")[0]]
assert len(rows) == 1, len(rows)
name, path, old, new = rows[0][:4]
old = old.replace("\\n", "\n"); new = new.replace("\\n", "\n")
text = open(root + "/" + path, encoding="utf-8").read()
assert text.count(old) == 1, text.count(old)
open(root + "/" + path, "w", encoding="utf-8").write(text.replace(old, new))
print("applied:", name)
