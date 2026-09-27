#!/usr/bin/env python3
# admission: always 判的是收尾调度表此刻的状态列，几毫秒跑完；用户每问一次进度都现算
# run-condition: none 只读 records/ 里那一份调度表
"""收尾进度一张表：读收尾调度表里带「批」与「状态」两列的表，按状态归成几类汇总，用户问进度时贴它的输出，不现翻调度表。

用法：
    closeout-status.py [调度表路径]      # 默认 records/2026-09-24-里程碑二收尾调度.md
    closeout-status.py --selftest        # CLOSEOUT_STATUS_BREAK=<项> 时必须判红

归类看状态格的第一个分句（去掉 ** 之后、到第一个「；。（(」为止）含哪一类的词，按下面的次序先中先归：
    等着   排队、排在、等、待、没开、未开、挂起、下一轮
    在跑   已派、在跑、在飞、跑着、在写、续做、接续、重派
    完成   已打、已进主工作区、写完、做完、判完、已判、交回、已提交、已还、完成、不做
    其他   都不是（照原样列出，人看）
输出：每类几批；没完成的（在跑、等着、其他）每批一行（批名、类、状态格前 60 个字），完成的只数不列；最后一行「共 N 批」。没有一张带这两列的表就退 2。
"""
import os
import re
import sys
import tempfile

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, os.path.join(REPOSITORY_ROOT, ".claude", "singlefs-ai-sop", "scripts"))
from preflight import preflight  # noqa: E402
DEFAULT_TABLE = os.path.join(REPOSITORY_ROOT, "records", "2026-09-24-里程碑二收尾调度.md")
BROKEN = os.environ.get("CLOSEOUT_STATUS_BREAK", "")
CATEGORIES = [
    ("等着", ("排队", "排在", "等", "待", "没开", "未开", "挂起", "下一轮")),
    ("在跑", ("已派", "在跑", "在飞", "跑着", "在写", "续做", "接续", "重派")),
    ("完成", ("已打", "已进主工作区", "写完", "做完", "判完", "已判", "交回", "已提交", "已还", "完成", "不做")),
]


def cells(line):
    return [cell.strip() for cell in line.strip().strip("|").split("|")]


def batches(text):
    """→ [(批, 状态格)]：表头里有「批」与「状态」两列的每张表的每一行。"""
    found, header = [], None
    for line in text.splitlines():
        if not line.startswith("|"):
            header = None
            continue
        row = cells(line)
        if header is None:
            header = row if ("批" in row and "状态" in row) else []
            continue
        if not header or set(line.replace("|", "").strip()) <= set("-: "):
            continue
        if len(row) >= len(header):
            found.append((row[header.index("批")], row[header.index("状态")]))
    return found


def category_of(status):
    head = re.split(r"[；。（(]", status.replace("*", "").strip(), 1)[0]
    for name, words in CATEGORIES:
        if any(word in head for word in words):
            return name
    return "完成" if BROKEN == "everything-done" else "其他"


def summarize(text):
    rows = batches(text)
    if not rows:
        return 2, ["✗ 调度表里没有一张表头带「批」与「状态」两列的表", "→ 怎么办：给路径指到收尾调度表，或照它「一、批次」那张表的表头写"]
    grouped = {}
    for batch, status in rows:
        grouped.setdefault(category_of(status), []).append((batch, status))
    order = ["在跑", "等着", "其他", "完成"]
    lines = ["| 类 | 批数 |", "|---|---|"] + [f"| {name} | {len(grouped.get(name, []))} |" for name in order]
    lines += ["", "| 批 | 类 | 状态（前 60 字） |", "|---|---|---|"]
    for name in order:
        for batch, status in grouped.get(name, [])[: None if name != "完成" or BROKEN else 0]:
            lines.append(f"| {batch} | {name} | {status.replace('**', '')[:60]} |")
    lines.append(f"共 {len(rows)} 批")
    return 0, lines


def selftest():
    sample = ("# 调度\n\n| 批 | 做什么 | 状态 |\n|---|---|---|\n| 实一 | 甲 | **已打进主工作区**（09-24） |\n| 实二 | 乙 | 在跑（实现员） |\n"
              "| 实三 | 丙 | 等实二交回 |\n| 实四 | 丁 | 看情况 |\n\n| 步 | 做什么 |\n|---|---|\n| 1 | 不是批次表 |\n")
    code, lines = summarize(sample)
    failures = []
    wanted = {"| 完成 | 1 |", "| 在跑 | 1 |", "| 等着 | 1 |", "| 其他 | 1 |", "共 4 批"}
    if code != 0 or not wanted <= set(lines):
        failures.append(f"四批应当各归一类、共 4 批，实际 {code} {lines}")
    code, _ = summarize("# 空\n\n| 步 | 做什么 |\n|---|---|\n| 1 | x |\n")
    if code != 2:
        failures.append(f"没有批次表应当退 2，实际 {code}")
    for failure in failures:
        print(f"  ✗ 自检：{failure}")  # gate-lint:detail
    if failures:
        print("  → 看 batches() / category_of() 的判法；CLOSEOUT_STATUS_BREAK 设着的话这里本来就该红")
        return 1
    print("  ✓ closeout-status 自检通过：完成、在跑、等着、其他各归一类，别的表不算批次，没有批次表退 2（查了 2 种）")
    return 0


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    path = sys.argv[1] if len(sys.argv) > 1 else DEFAULT_TABLE
    code, lines = summarize(open(path, encoding="utf-8").read())
    print("\n".join(lines))
    sys.exit(code)
