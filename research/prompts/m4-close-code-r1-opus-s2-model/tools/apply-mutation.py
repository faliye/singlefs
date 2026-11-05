"""攻方腿 s2：把 mutations.tsv 里点名的一行改进改后那棵树（先把表里点到的每份文件从冻结副本拷回原样，再改点名的那一行），或只还原。
参数：草稿目录 冻结副本根 变异名|--restore。"""
import shutil
import sys

work, frozen, name = sys.argv[1], sys.argv[2], sys.argv[3]
rows = [line.rstrip("\n").split("\t") for line in open(f"{work}/model/tools/mutations.tsv", encoding="utf-8") if not line.startswith("#")]
for file_name in sorted({row[1] for row in rows}):
    shutil.copyfile(f"{frozen}/crates/singlefs-core/src/{file_name}", f"{work}/new/crates/singlefs-core/src/{file_name}")
applied = 0
for row_name, file_name, original, replacement in rows:
    if row_name == name:
        target = f"{work}/new/crates/singlefs-core/src/{file_name}"
        text = open(target, encoding="utf-8").read()
        assert text.count(original) == 1, (row_name, text.count(original))
        open(target, "w", encoding="utf-8").write(text.replace(original, replacement))
        applied += 1
        print("applied", row_name, "to", file_name)
assert applied == 1 or name == "--restore", ("no such mutation", name)
