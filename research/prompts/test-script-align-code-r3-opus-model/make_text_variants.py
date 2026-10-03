"""按变异表的行号造只改原文的仓副本（判法代码按原文收、源码扫描按原文扫，用不着编译）：
python3 make_text_variants.py <冻结副本> <输出目录> <行号>...；每行一个 <输出目录>/row<行号>/，原文必须恰好命中一次。
替换文与原文里的 \\n 照 mutate.sh 的写法换成换行。"""
import os, subprocess, sys
frozen, out, numbers = sys.argv[1], sys.argv[2], [int(n) for n in sys.argv[3:]]
lines = open(os.path.join(frozen, "crates/mutations.tsv"), encoding="utf-8").read().split("\n")
for number in numbers:
    name, path, original, replacement = lines[number - 1].split("\t")[:4]
    original, replacement = original.replace("\\n", "\n"), replacement.replace("\\n", "\n")
    target = os.path.join(out, f"row{number}")
    subprocess.run(["rsync", "-a", "--delete", "--exclude", "target", frozen.rstrip("/") + "/", target + "/"], check=True)
    file = os.path.join(target, path)
    text = open(file, encoding="utf-8").read()
    assert text.count(original) == 1, (number, text.count(original))
    open(file, "w", encoding="utf-8").write(text.replace(original, replacement, 1))
    print(f"row {number} {name} -> {file}")
