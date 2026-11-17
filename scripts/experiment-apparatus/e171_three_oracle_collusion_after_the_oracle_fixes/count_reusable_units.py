#!/usr/bin/env python3
"""E171 第 1 次重跑·修订 11 第 11.7 节（R6）的计数：两棵冻结树上逐单元算键，数能复用的。
请求与单元表都取同目录 reuse_library.py（与三层编排的是同一份函数），键照 long_run_item_verdicts 算（reuse_library.keyed_items），不查库、不写库。

用法（在仓根；两棵树都要在仓根底下，键的入口闭包从树里算）：
  count_reusable_units.py --repo R --tree-a A --tree-b B [--per-unit]
开头核：单元表里每条装置与 E170 导出装置的参数串（--seed-hex 之前那一截）、执行器那一项的末一个词、第 4 步每项的「empty_control <段> <对照> <流>」
  与变异驱动的变异表路径，在 segment_one_unit_revision_8.sh 与 segment_one_unit_revision_11.sh 里都逐字出现；缺了退 2、不出计数。
不读任何 E171 产物与导出：上游读数与限时按单元名接成上游那一单元的键（line:upstream:<角色>=key-of:<那一把键>、line:limit-from=<计时那一单元>:<那一把键>），
  上游重跑之后读数字节相同才可复用，所以只差这两样的记 upstream 一类；先算没有上游的，再一批批算上游的键都有了的。
第二段跨机对照那一行与 RC3 抽样按那一趟现算，不在计数里；单元总数 = 第 0–10 步 33 + 第一段 16 行 + 第二段整表行数（改后两张名单第一列之并，
  list_check.whole_table_row_numbers 现算；E171 第 2 次重跑撤项之前是 281）。
判定五类：same（键相同，可复用）；upstream（只差上游或限时，条件式）；differs（别的不同）；fresh-always（这一层自己拒算的几类：second-run、补位那几类、
  树那一侧的，reuse_library 文件头「按项判定」）；refused（admission 算不出键，long_run_item_verdicts 照拒算）。
输出：count group=<组> verdict=<类> units=<数>，末行 total units=<数>；--per-unit 另逐单元一行。
退出码：0 出了计数；2 用法错或开头核不过。
"""
from __future__ import annotations

import argparse
import collections
import dataclasses
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import list_check  # noqa: E402
import reuse_library as reuse  # noqa: E402

UNIT_SCRIPTS = ("segment_one_unit_revision_8.sh", "segment_one_unit_revision_11.sh")
WHOLE_TABLE_ROWS = list_check.whole_table_row_numbers()
GROUP_STEPS, GROUP_SEGMENT_ONE, GROUP_SEGMENT_TWO = "第 0–10 步单元", "第一段行", "第二段行"
# 各类行的限时取哪一个单元的计时：第 4 步与第二段取 step-1，第一段第 11、12 步取 step-7（基线那一趟）
LIMIT_SOURCES = {"step-4": "step-1", "seg1": "step-7", "seg2": "step-1"}


def unit_script_problems(script_text: str) -> list[str]:
    """单元表的参数在单元脚本里找不到的那几样。"""
    wanted = []
    for unit in reuse.SEGMENT_ONE_READING_UNITS:
        if unit.kind in ("device", "e170"):
            wanted.append(unit.arguments.split(" --seed-hex")[0])
        elif unit.kind == "o3-executor":
            wanted.append(unit.arguments.split()[-1])
        else:
            wanted.append(unit.arguments)
    wanted += [f"empty_control {slug} {control} {streams}" for slug, control, streams in reuse.STEP_4_ITEMS]
    return [text for text in wanted if text not in script_text]


def counted_units() -> list:
    """计数的单元：第 0–10 步的读数单元、第 4 步十行、第一段第 11、12 步各行、第二段整表各行。"""
    return (list(reuse.SEGMENT_ONE_READING_UNITS) + reuse.step_4_row_units() + reuse.segment_one_row_units(("s11", "s12"))
            + reuse.segment_two_row_units(WHOLE_TABLE_ROWS, None))


def limit_source(unit) -> str | None:
    if not isinstance(unit, reuse.RowUnit):
        return None
    return LIMIT_SOURCES["step-4" if unit.name.startswith("step-4-") else unit.name.split("-", 1)[0]]


def dependencies(unit) -> list[str]:
    return [name for _, name in unit.upstream] + ([limit_source(unit)] if limit_source(unit) else [])


def with_tokens(request: reuse.UnitRequest, tokens: list[str]) -> reuse.UnitRequest:
    """请求的第一行接上几项（上游与限时按上游那一单元的键）。"""
    return dataclasses.replace(request, lines=["\t".join([request.lines[0], *tokens])] + request.lines[1:]) if tokens else request


def keys_of(repo: str, tree: str, linked: bool) -> dict[str, tuple[str | None, str]]:
    """一棵树上每个单元的 (键或 None, 拒算缘由)。linked：上游与限时按上游那一单元的键进请求；不 linked：这两样不进（判 upstream 一类用）。"""
    facts, selected = reuse.tree_facts(repo, tree, {}), reuse.selected_row_lines(repo)
    pending, keys = counted_units(), {}
    while pending:
        ready = [unit for unit in pending if all(name in keys for name in dependencies(unit))]
        if not ready:
            raise RuntimeError(f"上游排不出次序：{', '.join(unit.name for unit in pending[:4])}")
        requests = []
        for unit in ready:
            request = reuse.unit_request(facts, unit, selected)
            tokens = []
            if linked:
                tokens += [f"line:upstream:{role}=key-of:{keys[name][0] or '拒算'}" for role, name in unit.upstream]
                if limit_source(unit):
                    tokens.append(f"line:limit-from={limit_source(unit)}:{keys[limit_source(unit)][0] or '拒算'}")
            requests.append(with_tokens(request, tokens))
        computed = reuse.keyed_items(repo, requests)
        for request in requests:
            keys[request.unit] = (computed[request.unit][0], request.reason)
        pending = [unit for unit in pending if unit.name not in keys]
    return keys


def classify(first: tuple[str | None, str], second: tuple[str | None, str], bare_first: str | None, bare_second: str | None) -> str:
    if first[1] or second[1]:
        return "fresh-always"
    if first[0] is None or second[0] is None:
        return "refused"
    if first[0] == second[0]:
        return "same"
    return "upstream" if bare_first is not None and bare_first == bare_second else "differs"


def group_of(unit: str) -> str:
    if unit.startswith("seg2-"):
        return GROUP_SEGMENT_TWO
    if unit.startswith("seg1-"):
        return GROUP_SEGMENT_ONE
    return GROUP_STEPS


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 修订 11 第 11.7 节：两棵冻结树上能复用的单元数")
    for name in ("--repo", "--tree-a", "--tree-b"):
        parser.add_argument(name, required=True)
    parser.add_argument("--per-unit", action="store_true")
    arguments = parser.parse_args(argument_list)
    repo = os.path.abspath(arguments.repo)
    for name in UNIT_SCRIPTS:
        problems = unit_script_problems(open(os.path.join(HERE, name), encoding="utf-8").read())
        if problems:
            print(f"单元表的参数在 {name} 里找不到：{problems}", file=sys.stderr)
            return 2
    first, second = keys_of(repo, arguments.tree_a, True), keys_of(repo, arguments.tree_b, True)
    bare_first, bare_second = keys_of(repo, arguments.tree_a, False), keys_of(repo, arguments.tree_b, False)
    counts: collections.Counter = collections.Counter()
    for unit in first:
        verdict = classify(first[unit], second[unit], bare_first[unit][0], bare_second[unit][0])
        counts[(group_of(unit), verdict)] += 1
        if arguments.per_unit:
            print(f"unit={unit} verdict={verdict} key_a={str(first[unit][0])[:16]} key_b={str(second[unit][0])[:16]} reason={first[unit][1] or second[unit][1] or '-'}")
    for (group, verdict), number in sorted(counts.items()):
        print(f"count group={group} verdict={verdict} units={number}")
    print(f"total units={len(first)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
