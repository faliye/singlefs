#!/usr/bin/env python3
"""K4：修补二批补进 implementation-workflow.md 的那一句「实验复跑入口 scripts/experiment-apparatus/replay.sh（不带参数、点名实验号、--admission-only；
--list-items 与登记为 light 的单独 --selftest 不算）」逐项对 .claude/hooks/lib_heavy_tests.py 的 classify。
classify 与 classify_process 都问；--hooks 指到要问的那份 .claude/hooks（默认本脚本往上找不到时必须给）。只读，不起任何被问的命令。
用法：k4_classify.py --repo <仓根>"""
import argparse
import importlib.util
import os
import sys

parser = argparse.ArgumentParser()
parser.add_argument("--repo", required=True)
arguments = parser.parse_args()
repo = os.path.realpath(arguments.repo)
path = os.path.join(repo, ".claude", "hooks", "lib_heavy_tests.py")
sys.path.insert(0, os.path.dirname(path))
specification = importlib.util.spec_from_file_location("lib_heavy_tests_under_probe", path)
module = importlib.util.module_from_spec(specification)
sys.modules[specification.name] = module
specification.loader.exec_module(module)
replay = "scripts/experiment-apparatus/replay.sh"
cases = [
    ("不带参数", [replay]),
    ("点名实验号 E142", [replay, "E142"]),
    ("点名两个实验号", [replay, "E142", "E143"]),
    ("--admission-only", [replay, "--admission-only"]),
    ("--admission-only 加实验号", [replay, "--admission-only", "E142"]),
    ("--full", [replay, "--full"]),
    ("--list-items", [replay, "--list-items"]),
    ("--force --list-items", [replay, "--force", "--list-items"]),
    ("--list-items 后面带实验号", [replay, "--list-items", "E142"]),
    ("--list-items 后面带 --full", [replay, "--list-items", "--full"]),
    ("--list-items 两次", [replay, "--list-items", "--list-items"]),
    ("--selftest", [replay, "--selftest"]),
    ("--selftest 后面带实验号", [replay, "--selftest", "E142"]),
    ("--list-items 前面带实验号", [replay, "E142", "--list-items"]),
    ("--help", [replay, "--help"]),
]
print(f"name=k4_classify repo={repo}")
for label, tail in cases:
    direct = module.classify(tail, repo, {})
    process = module.classify_process(["bash"] + tail, repo)
    def show(result):
        return "None（放行）" if not result else "、".join(f"{item.kind}（{item.category}）" for item in (result if isinstance(result, list) else [result]))
    print(f"case={label} words={' '.join(tail)} classify={show(direct)} classify_process={show(process)}")
