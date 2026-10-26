#!/usr/bin/env python3
"""J3：「只差搬迁加语言切换行」的实验页数，不经门禁格的判法重数一遍（比格严：整页逐字比，不排除历史节与影响的决策一节）。
用法：recount_pages.py --repo <仓根> --base <基准>   读主工作区（会随工作区变），打 name= 行。"""
import argparse
import re
import subprocess

OLD, NEW = "scripts/experiments", "scripts/experiment-apparatus"
BOUND = re.compile(r"(?<![A-Za-z0-9_.\-])" + re.escape(OLD) + r"(?![A-Za-z0-9_\-]|\.[A-Za-z0-9_\-])")
SWITCH = re.compile(r"(\*\*中文\*\*|\[中文\]\([^)]*\)) · (\*\*English\*\*|\[English\]\([^)]*\)) · (\*\*日本語\*\*|\[日本語\]\([^)]*\))")


def strip_switch(lines):
    if len(lines) > 3 and SWITCH.fullmatch(lines[2].rstrip()):
        return lines[:2] + lines[4 if not lines[3].strip() else 3:]
    return lines


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--base", required=True)
    arguments = parser.parse_args()
    git = lambda *a: subprocess.run(["git", "-C", arguments.repo, "-c", "core.quotepath=false", *a], capture_output=True, text=True)
    names = set(git("diff", "--name-only", arguments.base, "--", ".claude/kb/experiments").stdout.split("\n"))
    names |= set(git("ls-files", "--others", "--exclude-standard", "--", ".claude/kb/experiments").stdout.split("\n"))
    names = sorted(n for n in names if n.endswith(".md") and "/" in n and not n.endswith((".en.md", ".ja.md")))
    only_move = switch_and_move = other = new_pages = 0
    for name in names:
        shown = git("show", f"{arguments.base}:{name}")
        if shown.returncode != 0:
            new_pages += 1
            continue
        base = shown.stdout
        try:
            current = open(f"{arguments.repo}/{name}", encoding="utf-8").read()
        except OSError:
            other += 1
            continue
        moved = BOUND.sub(NEW, base)
        if current == moved:
            only_move += 1
        elif strip_switch(current.split("\n")) == strip_switch(moved.split("\n")):
            switch_and_move += 1
        else:
            other += 1
    print(f"name=recount_pages base={arguments.base[:9]} changed_experiment_pages={len(names)} new_pages={new_pages} only_path_move={only_move} "
          f"path_move_plus_switch_line={switch_and_move} other_changes={other}")


main()
