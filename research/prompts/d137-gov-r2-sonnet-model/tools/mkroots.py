#!/usr/bin/env python3
"""建两棵喂钩子的树：<work>/now（此刻主工作区的 .claude/hooks 整份拷）与 <work>/base（同，但四份被修补的文件换成修补前的备份）。
仓根与 .claude 下别的项符号链接回 --repo；主工作区一个文件都不写。用法：mkroots.py --repo 仓根 --model 模型目录 --work 草稿目录"""
import argparse, os, shutil
ap = argparse.ArgumentParser()
ap.add_argument("--repo", required=True); ap.add_argument("--model", required=True); ap.add_argument("--work", required=True)
a = ap.parse_args()
for name in ("now", "base"):
    tree = os.path.join(a.work, name)
    if os.path.lexists(tree):
        shutil.rmtree(tree)
    os.makedirs(os.path.join(tree, ".claude"))
    for entry in sorted(os.listdir(a.repo)):
        if entry not in (".claude", ".git"):
            os.symlink(os.path.join(a.repo, entry), os.path.join(tree, entry))
    for entry in sorted(os.listdir(os.path.join(a.repo, ".claude"))):
        if entry != "hooks":
            os.symlink(os.path.join(a.repo, ".claude", entry), os.path.join(tree, ".claude", entry))
    shutil.copytree(os.path.join(a.repo, ".claude", "hooks"), os.path.join(tree, ".claude", "hooks"), symlinks=True, ignore=shutil.ignore_patterns("__pycache__"))
    if name == "base":
        for f in ("lib_shell_words.py", "heavy-test-guard.sh", "bash-command-detector.sh", "lib_heavy_tests.py", "runner-dispatch-guard.sh"):
            shutil.copy2(os.path.join(a.model, "baseline", f), os.path.join(tree, ".claude", "hooks", f))
    print("tree", tree)
