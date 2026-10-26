#!/usr/bin/env python3
"""J3：「395 份里 360 份换算之后相同」不经库重数。基准里旧前缀下的每份文件，与此刻新前缀下同相对路径的那份比：
整串替换（str.replace）与按库文件头写的边界（自己写的正则，不调库）各数一遍，再列出两种数法差在哪几份、差在哪一行。
用法：recount_files.py --repo <仓根> --base <基准> [--old 旧前缀 --new 新前缀]   读主工作区，随工作区变"""
import argparse
import re
import subprocess


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--base", required=True)
    parser.add_argument("--old", default="scripts/experiments")
    parser.add_argument("--new", default="scripts/experiment-apparatus")
    arguments = parser.parse_args()
    old, new = arguments.old, arguments.new
    git = lambda *a: subprocess.run(["git", "-C", arguments.repo, "-c", "core.quotepath=false", *a], capture_output=True)
    files = [name.decode() for name in git("ls-tree", "-r", "--name-only", "-z", arguments.base, "--", old).stdout.split(b"\0") if name]
    pattern = re.compile(r"(?<![A-Za-z0-9_.\-])" + re.escape(old) + r"(?![A-Za-z0-9_\-]|\.[A-Za-z0-9_\-])")
    plain = bounded = missing = 0
    differing = []
    for name in files:
        try:
            current = open(f"{arguments.repo}/{new}{name[len(old):]}", "rb").read()
        except OSError:
            missing += 1
            continue
        before = git("show", f"{arguments.base}:{name}").stdout.decode("utf-8", "surrogateescape")
        by_plain = current == before.replace(old, new).encode("utf-8", "surrogateescape")
        by_bound = current == pattern.sub(new, before).encode("utf-8", "surrogateescape")
        plain += by_plain
        bounded += by_bound
        if by_plain != by_bound:
            for text in before.split("\n"):
                if old in text and pattern.sub(new, text) != text.replace(old, new):
                    differing.append((name, text.strip()[:100]))
                    break
    print(f"name=recount_files base={arguments.base[:9]} files={len(files)} landing_missing={missing} same_by_plain_replace={plain} same_by_boundary_regex={bounded}")
    for name, text in differing:
        print(f"    plain!=boundary: {name} 第一处差在：{text}")


main()
