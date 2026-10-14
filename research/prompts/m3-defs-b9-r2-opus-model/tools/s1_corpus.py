#!/usr/bin/env python3
"""S1 专问「库载不进时误伤多少现有派发提示」：工作根下各草稿目录里主 agent 存的派发提示（<目录>/prompt.md，首行「# 派发：<类型>」），
逐份经 <副本根> 的 runner-dispatch-guard.sh --check-prompt 判两次：照常（本机 markdown-it-py 载得进）与 RUNNER_DISPATCH_GUARD_BREAK=code-block-library-missing
（与载不进同一条路：代码块一行都不遮）。打每份两次的退出码与拒的条目名，末行合计。跳过 --skip 点名的目录。
用法：s1_corpus.py --root <副本根> --work-root <工作根> --skip <目录名>..."""
import argparse, glob, os, re, subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--root", required=True)
parser.add_argument("--work-root", required=True)
parser.add_argument("--skip", action="append", default=[])
arguments = parser.parse_args()
paths = sorted(p for p in glob.glob(os.path.join(arguments.work_root, "*", "prompt.md"))
               if os.path.basename(os.path.dirname(p)) not in arguments.skip)

def judge(path, kind, extra):
    environment = dict(os.environ, CLAUDE_PROJECT_DIR=arguments.root, PYTHONDONTWRITEBYTECODE="1", **extra)
    result = subprocess.run(["bash", os.path.join(arguments.root, ".claude/hooks/runner-dispatch-guard.sh"), "--check-prompt", path, "--type", kind, "--root", arguments.root],
                            capture_output=True, text=True, cwd=arguments.root, env=environment)
    names = re.search(r"会拒 \d+ 处（(.*?)）", result.stdout)
    return result.returncode, (names.group(1) if names else "-")

total = changed = typed = 0
for path in paths:
    first = open(path, encoding="utf-8").readline()
    kind = re.match(r"#\s*派发[：:]\s*(\S+)", first)
    if not kind:
        continue
    typed += 1
    normal = judge(path, kind.group(1), {})
    missing = judge(path, kind.group(1), {"RUNNER_DISPATCH_GUARD_BREAK": "code-block-library-missing"})
    total += 1
    mark = "同" if normal == missing else "变"
    changed += normal != missing
    print(f"{mark}\t{os.path.basename(os.path.dirname(path))}\t{kind.group(1)}\t照常 退出={normal[0]} 拒={normal[1]}\t载不进 退出={missing[0]} 拒={missing[1]}")
print(f"合计：{len(paths)} 份提示，认得出类型 {typed} 份，判了 {total} 份，载不进时判定变了的 {changed} 份")
