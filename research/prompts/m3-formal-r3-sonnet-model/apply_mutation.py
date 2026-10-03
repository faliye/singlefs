#!/usr/bin/env python3
"""把 crates/mutations.tsv 的一行（行号从 1 起）套到工作副本上：原文必须恰好命中一次。
用法：apply_mutation.py <副本根> <表行号>   套上（先把原文件存成 <文件>.orig，没有才存）
      apply_mutation.py <副本根> restore    把所有 .orig 还原
"""
import sys, pathlib, shutil
root = pathlib.Path(sys.argv[1])
if sys.argv[2] == "restore":
    for orig in root.rglob("*.orig"):
        shutil.copyfile(orig, orig.with_suffix(""))
        orig.unlink()
        print("restored", orig.with_suffix(""))
    sys.exit(0)
row = int(sys.argv[2])
fields = (root / "crates/mutations.tsv").read_text().split("\n")[row - 1].split("\t")
name, file, old, new = fields[0], fields[1], fields[2].replace("\\n", "\n"), fields[3].replace("\\n", "\n")
target = root / file
text = target.read_text()
assert text.count(old) == 1, (row, text.count(old))
backup = target.with_name(target.name + ".orig")
if not backup.exists():
    shutil.copyfile(target, backup)
target.write_text(text.replace(old, new))
print("applied row", row, name, "->", file)
