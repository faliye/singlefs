#!/usr/bin/env python3
"""L1 C1/C2 的另一半：python -c 的代码改了 sys.argv 再 runpy.run_path，被起的脚本看到的是代码里写的参数，不是代码之后的词。
被起的是草稿目录里同名的桩（只打 sys.argv[1:]），不是判法脚本。用法：python3 l1-argv-demo.py <草稿目录>"""
import os, subprocess, sys

stub_root = os.path.join(sys.argv[1], "stub")
os.makedirs(os.path.join(stub_root, "research", "scripts"), exist_ok=True)
for name in ("verus-proof-check.py", "e165-pin-red-examples.py"):
    with open(os.path.join(stub_root, "research", "scripts", name), "w", encoding="utf-8") as handle:
        handle.write("import sys\nprint('桩看到的参数：', ' '.join(sys.argv[1:]))\n")
vpc, pin = "research/scripts/" + "verus-proof-check.py", "research/scripts/" + "e165-pin-red-examples.py"
for label, code, after in [
    ("C1", f"import runpy,sys; sys.argv=['v','proof','.']; runpy.run_path('{vpc}', run_name='__main__')", ["tool-versions"]),
    ("C3", f"import runpy,sys; sys.argv=['p','--replay']; runpy.run_path('{pin}', run_name='__main__')", ["--check"]),
]:
    completed = subprocess.run([sys.executable, "-c", code, *after], cwd=stub_root, capture_output=True, text=True)
    print(f"{label} 代码之后的词：{' '.join(after)}；{completed.stdout.strip()}")
