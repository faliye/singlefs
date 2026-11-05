"""在一棵仓副本里把一份文件里恰好命中一次的原文换成替换文（攻方腿的 T6 变异，同 crates/mutations.tsv 的一处改法）。
用法：python3 mutate-once.py <仓副本根> <相对路径> <原文文件> <替换文文件>；原文命中次数不是 1 就退 1、不改。"""
import pathlib
import sys

tree, relative, original_file, replacement_file = sys.argv[1:5]
target = pathlib.Path(tree) / relative
text = target.read_text(encoding="utf-8")
original = pathlib.Path(original_file).read_text(encoding="utf-8")
replacement = pathlib.Path(replacement_file).read_text(encoding="utf-8")
hits = text.count(original)
if hits != 1:
    print(f"✗ 原文在 {relative} 里命中 {hits} 次，不是 1 次：不改")
    sys.exit(1)
target.write_text(text.replace(original, replacement), encoding="utf-8")
print(f"mutated {relative}")
