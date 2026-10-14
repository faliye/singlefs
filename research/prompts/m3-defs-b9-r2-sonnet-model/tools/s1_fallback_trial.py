#!/usr/bin/env python3
"""S1 试改（辩方腿提的，只在本腿模型上量过）：库载不进时的退路从『一行都不遮』改回逐行认法（派发闸里留着的 without_code_blocks_by_lines）。
在 <树> 的拷贝上改，用 s1_library_missing.sh（PATH 前置垫片让 import markdown_it 真的失败）判三份提示。
用法：s1_fallback_trial.py <b10 树> <垫片目录> <工作目录> <s1_library_missing.sh>"""
import os, shutil, subprocess, sys
tree, shim, work, script = (os.path.abspath(p) for p in sys.argv[1:5])
patched = os.path.join(work, "patched"); shutil.rmtree(patched, ignore_errors=True); os.makedirs(os.path.join(patched, "research/prompts")); os.makedirs(os.path.join(patched, "records"))
for name in (".claude", "scripts"): shutil.copytree(os.path.join(tree, name), os.path.join(patched, name))
path = os.path.join(patched, ".claude/hooks/runner-dispatch-guard.sh")
text = open(path, encoding="utf-8").read()
old = '''        return prompt
    lines = prompt.split("\\n")
    parsed ='''
assert text.count(old) == 1, text.count(old)
new = '''        return without_code_blocks_by_lines(prompt, loose=False)   # 试改：退路改回逐行认法
    lines = prompt.split("\\n")
    parsed ='''
open(path, "w", encoding="utf-8").write(text.replace(old, new))
for label, root in (("现状（一行都不遮）", tree), ("试改（退回逐行认法）", patched)):
    print(f"## {label}")
    done = subprocess.run(["bash", script, root, shim, os.path.join(work, "run-" + ("a" if root == tree else "b"))], capture_output=True, text=True)
    print("\n".join(l for l in done.stdout.split("\n") if "\tPATH=shim\t" in l))
