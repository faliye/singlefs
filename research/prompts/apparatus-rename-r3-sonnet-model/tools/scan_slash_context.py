#!/usr/bin/env python3
"""J1：基准那一版里旧前缀 scripts/experiments 前面接着 / 的写法有哪几种上下文（换算会换，别的目录下的同名段也会被换）。
用法：scan_slash_context.py --repo <仓根> --base <基准>   打 name= 行：总命中数、前一个字符是 / 的命中数、这些命中前面那一段的分布（前 12 种）"""
import argparse
import collections
import re
import subprocess

OLD = "scripts/experiments"
# 收严 Z 会放行的前文：./ 与 ../（可连写）、$变量/ 与 ${变量}/、<仓根>/、本仓的绝对路径；别的目录名后面的同名段不换
ANCHORED = re.compile(r"(?:(?:^|[^A-Za-z0-9_.\-/])(?:\.{1,2}/)+|\$\{?[A-Za-z_][A-Za-z0-9_]*\}?/|<仓根>/|/home/fy5090/code/singlefs/)$")
BOUND = re.compile(r"(?<![A-Za-z0-9_.\-])" + re.escape(OLD) + r"(?![A-Za-z0-9_\-]|\.[A-Za-z0-9_\-])")


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", required=True)
    parser.add_argument("--base", required=True)
    arguments = parser.parse_args()
    # 只看含旧前缀字串的文件：git grep -l 列出来（基准里的树，不读工作区），其余文件一次 git show 都不起
    listed = [name.split(b":", 1)[1] for name in subprocess.run(["git", "-C", arguments.repo, "grep", "-l", "-z", "-F", "-I", OLD, arguments.base], capture_output=True).stdout.split(b"\0") if b":" in name]
    total = slash = unanchored = 0
    unanchored_files = set()
    context = collections.Counter()
    files = set()
    for name in listed:
        if not name:
            continue
        shown = subprocess.run(["git", "-C", arguments.repo, "show", f"{arguments.base}:{name.decode()}"], capture_output=True).stdout
        if b"\0" in shown[:4096]:
            continue
        text = shown.decode("utf-8", "replace")
        for found in BOUND.finditer(text):
            total += 1
            before = text[max(0, found.start() - 40):found.start()]
            if before.endswith("/"):
                slash += 1
                files.add(name.decode())
                if not ANCHORED.search(before):
                    unanchored += 1
                    unanchored_files.add(name.decode())
                tail = re.search(r"([^\s\"'`(=:,;]*/)$", before)
                context[tail.group(1) if tail else before[-12:]] += 1
    print(f"name=scan_slash_context base={arguments.base[:9]} occurrences={total} preceded_by_slash={slash} files_with_slash_preceded={len(files)} not_after_dot_slash_var_root={unanchored} in_files={len(unanchored_files)}")
    for text, count in context.most_common(12):
        print(f"    {count:5d}  {text!r}")


main()
