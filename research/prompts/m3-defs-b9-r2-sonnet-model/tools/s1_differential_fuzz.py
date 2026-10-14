#!/usr/bin/env python3
"""S1：b9（逐行认围栏）与 b10（markdown-it）的 without_code_blocks 差分模糊。随机拼 markdown 碎片，找两边遮的行不同的提示，
按方向分两类并各缩到最短：『b10 比 b9 少遮』（b10 更严，会误拒）、『b10 比 b9 多遮』（b10 更宽，会漏判）。
再按 markdown-it 的令牌给每个例子标出遮/不遮的那些行落在哪种块里。用法：s1_differential_fuzz.py <b9 树> <b10 树> [种子] [份数]"""
import os, random, sys
def load(root):
    hook = os.path.join(os.path.abspath(root), ".claude/hooks/runner-dispatch-guard.sh")
    text = open(hook, encoding="utf-8").read().split("\n")
    start = next(i for i, l in enumerate(text) if l.startswith("python3 /dev/fd/3"))
    end = next(i for i in range(start + 1, len(text)) if text[i] == "PY")
    source = "\n".join(text[start + 1:end]).replace("\nsys.exit(main())", "\n")
    saved = sys.argv; sys.argv = ["hook", os.path.dirname(hook)]
    namespace = {"__name__": "hook_under_test"}
    exec(compile(source, "embedded", "exec"), namespace)
    sys.argv = saved
    return namespace["without_code_blocks"]
w9, w10 = load(sys.argv[1]), load(sys.argv[2])
seed = int(sys.argv[3]) if len(sys.argv) > 3 else 1; count = int(sys.argv[4]) if len(sys.argv) > 4 else 20000
random.seed(seed)
pieces = ["```", "~~~", "````", "    ```", "\t```", "- ```", "  ```", "> ```", "1. ```", "X", "Y", "Z", "", "<div>", "</div>", "<!--", "-->", "- 项", "  - 嵌套", "> 引", "```a`b", "~~~ x", "    X", "```x```"]
def hidden(fn, lines):
    out = fn("\n".join(lines)).split("\n")
    return {i for i, (a, b) in enumerate(zip(lines, out)) if a != b and b == ""}
def classify(lines):
    h9, h10 = hidden(w9, lines), hidden(w10, lines)
    return h9 - h10, h10 - h9
def shrink(lines, kind):
    changed = True
    while changed:
        changed = False
        for i in range(len(lines)):
            trial = lines[:i] + lines[i + 1:]
            if not trial: continue
            less, more = classify(trial)
            if (less if kind == "less" else more):
                lines, changed = trial, True; break
    return lines
found = {"less": {}, "more": {}}
for _ in range(count):
    lines = [random.choice(pieces) for _ in range(random.randint(2, 9))]
    less, more = classify(lines)
    for kind, s in (("less", less), ("more", more)):
        if s:
            small = shrink(list(lines), kind)
            found[kind].setdefault(tuple(small), None)
def show(kind, title):
    items = sorted(found[kind], key=lambda t: (len(t), t))
    print(f"## {title}：缩到最短后不同的例子 {len(items)} 个，列最短的 12 个")
    for t in items[:12]:
        print("   " + repr("\n".join(t)))
show("less", "b10 比 b9 少遮（b10 更严）")
show("more", "b10 比 b9 多遮（b10 更宽）")
