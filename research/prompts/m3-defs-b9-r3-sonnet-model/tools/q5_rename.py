#!/usr/bin/env python3
"""Q5：叫停登记的出处记录前面补一句（漂移）、再按改名表真换一个词（改字），各版 lib_abandoned_rounds.py 收不收。
改名对用 .claude/kb/term-renames.md 表里真有的一行（第一个事务 → 新池新建文件）；有表 / 没表两组。
用法：q5_rename.py <名>=<冻结副本根> ... --kb <含 .claude/kb/term-renames.md 的冻结副本根> --work <新目录>"""
import argparse, os, shutil, subprocess
parser = argparse.ArgumentParser(); parser.add_argument("roots", nargs="+"); parser.add_argument("--kb", required=True); parser.add_argument("--work", required=True)
arguments = parser.parse_args()
libs = [e.split("=", 1) for e in arguments.roots]
ROUND = "hx-rn-r1"; EXCERPT = "去向：hx-rn-r1 两条腿停掉（第一个事务的崩溃枚举没跑完）"
def tree(name, with_kb):
    root = os.path.join(arguments.work, name)
    os.makedirs(root + "/records", exist_ok=True); os.makedirs(root + "/research/prompts/" + ROUND + "-snapshot", exist_ok=True)
    open(root + "/research/prompts/_" + ROUND + "-body.md", "w").close()
    lines = ["# 调度记录", "", "## 前一批", "", "- 前一批的一行。", "", "## 这一批", "", f"- 开轮 {ROUND}：冻结副本在工作根下，快照 research/prompts/{ROUND}-snapshot/。",
             f"- 用户「第一个事务那一轮先停掉，不写判决」。{EXCERPT}", "- 别的活照常。"]
    number = [i for i, l in enumerate(lines) if EXCERPT in l][0] + 1
    open(root + "/records/log.md", "w", encoding="utf-8").write("\n".join(lines) + "\n")
    open(root + "/research/prompts/abandoned-rounds.tsv", "w", encoding="utf-8").write(f"{ROUND}\t2026-10-12\t用户叫停（records/log.md 第 {number} 行〔{EXCERPT}〕）\t不写判决\n")
    if with_kb:
        os.makedirs(root + "/.claude/kb", exist_ok=True); shutil.copy(arguments.kb + "/.claude/kb/term-renames.md", root + "/.claude/kb/term-renames.md")
    return root
def edit(root, kinds):
    path = root + "/records/log.md"; lines = open(path, encoding="utf-8").read().split("\n")
    if "reword" in kinds: lines = [l.replace("第一个事务", "新池新建文件") for l in lines]
    if "dup" in kinds:
        quoted = EXCERPT.replace("第一个事务", "新池新建文件") if "reword" in kinds else EXCERPT
        lines.insert(len(lines) - 1, f"- 补记：{ROUND} 的叫停登记已收，出处抄的是「{quoted}」。")
    if "drift" in kinds: lines.insert(lines.index("- 前一批的一行。") + 1, "- 前一批补一句：复跑数对上了。")
    open(path, "w", encoding="utf-8").write("\n".join(lines))
def judge(root, base):
    done = subprocess.run(["python3", "-B", base + "/.claude/hooks/lib_abandoned_rounds.py", "."], cwd=root, capture_output=True, text=True, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
    line = next((l for l in done.stdout.split("\n") if l.startswith(ROUND)), done.stdout.strip()[:80])
    return (line.split("\t", 1)[1][:34] if "\t" in line else line)
for with_kb in (False, True):
    for kinds in ([], ["reword"], ["drift"], ["drift", "reword"], ["dup", "reword"], ["drift", "dup"]):
        name = ("有改名表-" if with_kb else "无改名表-") + ("+".join(kinds) or "不改"); root = tree(name, with_kb); edit(root, kinds)
        print(name + "\t" + "\t".join(f"{label}：{judge(root, base)}" for label, base in libs))
