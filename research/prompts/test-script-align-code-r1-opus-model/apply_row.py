# 把 crates/mutations.tsv 里点名的一行（按变异名）改坏进草稿副本：原文恰好一次、\n 是换行（照 mutate.sh 的写法）；写临时文件再换上
import sys, os
tree, name = sys.argv[1], sys.argv[2]
rows = [l.rstrip("\n").split("\t") for l in open(os.path.join(tree, "crates/mutations.tsv"), encoding="utf-8") if l.strip() and not l.startswith("#")]
hit = [r for r in rows if r[0] == name]
assert len(hit) == 1, f"变异名命中 {len(hit)} 行"
_, path, original, replacement = hit[0][:4]
target = os.path.join(tree, path)
source = open(target, encoding="utf-8").read()
original = original.replace("\\n", "\n"); replacement = replacement.replace("\\n", "\n")
assert source.count(original) == 1, f"原文命中 {source.count(original)} 次"
open(target + ".tmp", "w", encoding="utf-8").write(source.replace(original, replacement))
os.replace(target + ".tmp", target)
print(f"applied {name} -> {path}")
