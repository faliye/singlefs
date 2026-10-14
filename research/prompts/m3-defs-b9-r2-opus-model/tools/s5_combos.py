#!/usr/bin/env python3
"""S5 合成小树：叫停登记那一刻〔〕原话在出处记录里恰好命中一次（第 N 行），之后照日常改出处记录，用各版 lib_abandoned_rounds.py 判收不收。
三种日常改动：drift（第 N 行之前插一行，前面的小节补一句）、dup（末尾追加一行补记，逐字抄了原话）、reword（第 N 行里「第一条流」照术语改名换成「新池新建文件流」，
sweep-term --apply 对 records/ 照换、登记表在 research/prompts/ 下整个豁免不换）；单做与两两合做。
另一组 open-line：第 N-1 行是开轮那一行（带这一轮名字与用户「」原话），drift 加 reword 之后行号提示落到开轮那一行。
用法：s5_combos.py --lib <名>=<lib_abandoned_rounds.py> ... --work <新目录>"""
import argparse, os, subprocess

parser = argparse.ArgumentParser()
parser.add_argument("--lib", action="append", required=True)
parser.add_argument("--work", required=True)
arguments = parser.parse_args()
libs = [entry.split("=", 1) for entry in arguments.lib]
ROUND = "hx-term-r1"
EXCERPT = "去向：hx-term-r1 两条腿停掉（第一条流的崩溃枚举没跑完）"

def tree(name, open_line_quote):
    root = os.path.join(arguments.work, name)
    os.makedirs(os.path.join(root, "records"), exist_ok=True)
    os.makedirs(os.path.join(root, "research/prompts", ROUND + "-snapshot"), exist_ok=True)
    open(os.path.join(root, "research/prompts", "_" + ROUND + "-body.md"), "w").close()
    opening = (f"- 开轮 {ROUND}：用户说「这一题照三轮走，先派攻方」；冻结副本在工作根下。" if open_line_quote
               else f"- 开轮 {ROUND}：冻结副本在工作根下，快照 research/prompts/{ROUND}-snapshot/。")
    lines = ["# 调度记录", "", "## 前一批", "", "- 前一批的一行。", "", "## 这一批", "", opening,
             f"- 用户「第一条流那一轮先停掉，不写判决」。{EXCERPT}", "- 别的活照常。"]
    number = lines.index(f"- 用户「第一条流那一轮先停掉，不写判决」。{EXCERPT}") + 1
    open(os.path.join(root, "records/log.md"), "w", encoding="utf-8").write("\n".join(lines) + "\n")
    open(os.path.join(root, "research/prompts/abandoned-rounds.tsv"), "w", encoding="utf-8").write(
        f"{ROUND}\t2026-10-12\t用户叫停（records/log.md 第 {number} 行〔{EXCERPT}〕）\t不写判决\n")
    return root

def change(root, kinds):
    path = os.path.join(root, "records/log.md")
    lines = open(path, encoding="utf-8").read().split("\n")
    if "reword" in kinds:
        lines = [line.replace("第一条流", "新池新建文件流") for line in lines]
    if "dup" in kinds:
        quoted = EXCERPT.replace("第一条流", "新池新建文件流") if "reword" in kinds else EXCERPT
        lines.insert(len(lines) - 1, f"- 补记：{ROUND} 的叫停登记已收，出处抄的是「{quoted}」。")
    if "drift" in kinds:
        lines.insert(lines.index("- 前一批的一行。") + 1, "- 前一批补一句：复跑数对上了。")
    open(path, "w", encoding="utf-8").write("\n".join(lines))

def judge(root):
    cells = []
    for label, lib in libs:
        result = subprocess.run(["python3", "-B", lib, "."], cwd=root, capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        verdict = next((line for line in result.stdout.split("\n") if line.startswith(ROUND)), result.stdout.strip()[:80])
        cells.append(f"{label}：{verdict.split(chr(9), 1)[1][:110] if chr(9) in verdict else verdict}")
    return "\t".join(cells)

for open_quote in (False, True):
    for kinds in ([], ["drift"], ["dup"], ["reword"], ["drift", "dup"], ["drift", "reword"], ["dup", "reword"]):
        if open_quote and kinds != ["drift", "reword"]:
            continue
        name = ("open-line-" if open_quote else "") + ("+".join(kinds) or "none")
        root = tree(name, open_quote)
        change(root, kinds)
        print(f"{name}\t{judge(root)}")
