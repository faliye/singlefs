#!/usr/bin/env python3
"""C1 读法甲（正则去注释、空白折叠）在今天的 crates 上会看错多少处：被变异的两个包（core、checker）的 src 里，
函数体内的串字面量含 // 或 /* 或 */ 的、含两个以上连续空白或换行的。用 run.py 的词法切法与函数切法。
用法：c1_literals_in_crates.py <模型目录> <仓根>
"""
import glob
import os
import re
import sys

sys.argv, model, root = sys.argv[:1] + ["-", "-"], sys.argv[1], sys.argv[2]
sys.path.insert(0, model)
import run  # noqa: E402

counts = {"files": 0, "functions": 0, "string-tokens-in-functions": 0, "comment-markers-in-string": 0,
          "whitespace-run-or-newline-in-string": 0}
examples = []
for path in sorted(glob.glob(os.path.join(root, "crates/singlefs-core/src/**/*.rs"), recursive=True)
                   + glob.glob(os.path.join(root, "crates/singlefs-checker/src/**/*.rs"), recursive=True)):
    source = open(path).read()
    counts["files"] += 1
    relative = os.path.relpath(path, root)
    for key, item, body_start, body_end in run.functions(source):
        counts["functions"] += 1
        text = source[item:body_end]
        for token, start, _ in run.tokens(text):
            if not (token.startswith('"') or token.startswith('r"') or token.startswith("r#") or token.startswith('b"')):
                continue
            counts["string-tokens-in-functions"] += 1
            if "//" in token or "/*" in token or "*/" in token:
                counts["comment-markers-in-string"] += 1
                if len(examples) < 8:
                    line = source[: item + start].count("\n") + 1
                    examples.append(f"{relative}:{line} {token[:60]}")
            if re.search(r"\s\s|\n", token):
                counts["whitespace-run-or-newline-in-string"] += 1
                if len(examples) < 12:
                    line = source[: item + start].count("\n") + 1
                    examples.append(f"{relative}:{line} {token[:50]!r}")
print("C1-LITERALS " + " ".join(f"{k}={v}" for k, v in counts.items()))
for example in examples:
    print("C1-LITERALS example", example)
