"""三方腿类型表：哪几种子 agent 类型是三方腿，只登记在各份定义的 frontmatter 里（`leg: three-way` 一行），这一份只管读它。

四处用这一份，不各抄一份类型名：
  .claude/hooks/runner-dispatch-guard.sh   ⑰ 三方腿的报告形态只对这几种判
  .claude/hooks/handback-guard.sh          ② 交回里点名的 research/prompts/…-output.md 交 cite-check.py，只对这几种判
  scripts/agents/dispatch-prompt.py        骨架的「报告：」一行，这几种填 research/prompts/<填 轮>-<填 模型>-output.md 占位
  scripts/agents/agent-cost.py             outcome 的两个三方腿桶，只收这几种

读法（three_way_leg_types）：<仓根>/.claude/agents/ 下每份 .md，开头一行是 `---`，到下一行 `---` 为止是 frontmatter；
其中 `leg:` 那一行的值（去掉行尾 # 注释与首尾空白）等于 `three-way` 的，文件名去掉 .md 就是一种三方腿类型。
正文里写的 `leg: three-way` 不算；目录不在、一份都没登记，交回空集（调用方各自决定空集怎么办）。
每处的自证各有一格，拿样本仓与本仓两份登记核它们读到的集合与 frontmatter 逐个相同；「frontmatter 里写着什么」取 three_way_leg_types_scanned
（不拆键值，在 frontmatter 那一段里按整行找），四处的自证共用它，不各写一份。
"""
import glob
import os
import re

LEG_LINE = re.compile(r"(?m)^leg:[ \t]*three-way[ \t]*(?:#.*)?$")

LEG_KEY = "leg"
THREE_WAY_LEG = "three-way"


def frontmatter_fields(path):
    """一份定义的 frontmatter → {键: 值}（值去掉行尾 # 注释与首尾空白）；读不了或开头不是 `---` 交回 None。"""
    try:
        with open(path, encoding="utf-8") as handle:
            lines = handle.read().split("\n")
    except OSError:
        return None
    if not lines or lines[0].strip() != "---":
        return None
    fields = {}
    for line in lines[1:]:
        if line.strip() == "---":
            break
        if ":" in line:
            key, value = line.split(":", 1)
            fields[key.strip()] = value.split("#", 1)[0].strip()
    return fields


def three_way_leg_types(project_root):
    """<project_root>/.claude/agents/ 里 frontmatter 写了 `leg: three-way` 的类型名 → frozenset。"""
    found = set()
    for path in glob.glob(os.path.join(project_root, ".claude", "agents", "*.md")):
        fields = frontmatter_fields(path)
        if fields is not None and fields.get(LEG_KEY) == THREE_WAY_LEG:
            found.add(os.path.basename(path)[: -len(".md")])
    return frozenset(found)


def three_way_leg_types_scanned(project_root):
    """自证用的另一种读法，当「frontmatter 里写着什么」：每份定义开头是 `---` 的，取到下一处换行接 `---` 为止的那一段，
    整行是 `leg: three-way`（可带行尾 # 注释）的算登记了。不拆键值，与 three_way_leg_types 走的不是同一条路 → 排好序的列表。"""
    found = set()
    for path in glob.glob(os.path.join(project_root, ".claude", "agents", "*.md")):
        try:
            with open(path, encoding="utf-8") as handle:
                head = handle.read().split("\n---", 1)[0]
        except OSError:
            continue
        if head.startswith("---") and LEG_LINE.search(head):
            found.add(os.path.basename(path)[: -len(".md")])
    return sorted(found)
