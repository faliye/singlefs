#!/usr/bin/env python3
"""攻方腿探针：写入一侧的相抵（.claude/gate.d/lib-forbidden-notations.py 的 new_violation_hits）按「判据行名字加命中字面、整份计数」抵，
几种挪号、换号、换描述的写法各交几处新违例。只读被判那一份库，不写任何仓内文件。
用法：python3 netting_probe.py <冻结树根>   每一种写法打一行 name=netting_<写法> new_hits=<n> expected_by_intent=<n> verdict=<same|differs>"""
import importlib.util
import os
import sys

root = sys.argv[1]
specification = importlib.util.spec_from_file_location("lib_forbidden_notations_probe", os.path.join(root, ".claude", "gate.d", "lib-forbidden-notations.py"))
library = importlib.util.module_from_spec(specification)
specification.loader.exec_module(library)

FORM = "task-and-session-numbers"
PATH = "records/probe-notes.md"
# (写法名, 写前, 写后, 按判据本意该报的新违例数：拿描述指别的会话那一类整句换成指另一个会话算一次新写)
# 规则字面（.claude/agent-common.md「写」一节）按判据与命中字面整份计数；本意与字面不一致的那几种，verdict 打 differs
CASES = (
    ("same-line-path-edit", "甲句改了 a/b.rs（任务 12345）。\n", "甲句改了 a/c/main.rs（任务 12345）。\n", 0),
    ("number-moved-to-another-sentence", "甲句（任务 12345）。\n乙句说另一件事。\n", "甲句。\n乙句说另一件事（任务 12345）。\n", 0),
    ("old-number-deleted-new-number-added", "甲句（任务 12345）。\n", "甲句（任务 67890）。\n", 1),
    ("same-number-duplicated", "甲句（任务 12345）。\n", "甲句（任务 12345）。\n乙句（任务 12345）。\n", 1),
    ("session-description-rewritten-to-another-session", "改闭包的那一路会话回信说库换好了。\n", "管任务服务的那一路会话回信说服务重启了。\n", 1),
    ("session-description-moved-and-reworded", "甲：改闭包的那一路会话回信说库换好了。\n乙：无关。\n", "甲：无关。\n乙：管变异键的那一路会话回信说表改了。\n", 1),
)

for name, before, after, by_intent in CASES:
    hits = library.new_violation_hits(FORM, before, after, PATH)
    literal = "；".join(f"{hit[2]}:{hit[3]}" for hit in hits) or "无"
    print(f"name=netting_{name} new_hits={len(hits)} expected_by_intent={by_intent} verdict={'same' if len(hits) == by_intent else 'differs'} hits={literal}")
