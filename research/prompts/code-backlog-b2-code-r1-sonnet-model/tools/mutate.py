#!/usr/bin/env python3
"""mutate.py <树> <变异表.tsv> <第几行，从 0 数>：把那一行的原文（\\n 当换行）在树里的文件上换成替换文，原文要恰好命中一次。
mutate.py <树> extra order|guard：正推腿自己加的两个取样点（豁免与判重的次序对调、去掉「现行那一版引用着就不验」这一道）。"""
import sys
tree, which = sys.argv[1], sys.argv[2]
if which == "extra":
    kind = sys.argv[3]
    path = f"{tree}/crates/singlefs-core/src/mount.rs"
    text = open(path, encoding="utf-8").read()
    table = {
        "order": (".filter(|covered| !exempt.contains(covered) && isolated.insert(*covered))",
                  ".filter(|covered| isolated.insert(*covered) && !exempt.contains(covered))"),
        "guard": ("        } else if !is_referenced_by_the_current_version {\n",
                  "        } else if true || !is_referenced_by_the_current_version {\n"),
    }
    original, replacement = table[kind]
    assert text.count(original) == 1, "原文要恰好命中一次"
    open(path, "w", encoding="utf-8").write(text.replace(original, replacement))
    print("applied extra", kind)
else:
    rows = [l for l in open(which, encoding="utf-8").read().split("\n") if l]
    name, path, original, replacement, args, test = rows[int(sys.argv[3])].split("\t")
    path = f"{tree}/{path}"
    text = open(path, encoding="utf-8").read()
    original, replacement = original.replace("\\n", "\n"), replacement.replace("\\n", "\n")
    assert text.count(original) == 1, "原文要恰好命中一次"
    open(path, "w", encoding="utf-8").write(text.replace(original, replacement))
    print("applied", name)
