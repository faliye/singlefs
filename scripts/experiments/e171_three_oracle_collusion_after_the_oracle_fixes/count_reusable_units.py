#!/usr/bin/env python3
"""E171 第 1 次重跑·修订 11 第 11.7 节（R6）的计数：两棵树（或一棵树加一处假想改动）上逐单元算键，数能复用的。
键与单元表都取同目录 reuse_library.py（与三层编排算键的是同一份函数）；照设计员草稿 count_reusable.py 写进仓（不从仓里 import 草稿）。

用法（在仓根）：
  count_reusable_units.py --repo R --tree-a A --dep-a-device D1 --dep-a-e170 D2 --toolchain <文本文件>
                          (--tree-b B --dep-b-device D3 --dep-b-e170 D4 | --hypothetical-change <相对树根的路径>) [--per-unit]
开头核：单元表里每条装置与 E170 导出装置的参数串（--seed-hex 之前那一截）、执行器那一项的末一个词、第 4 步每项的「empty_control <段> <对照> <流>」
  与变异驱动的变异表路径，在 segment_one_unit_revision_8.sh 与 segment_one_unit_revision_11.sh 里都逐字出现；缺了退 2、不出计数。
不读任何 E171 产物与导出：[upstream] 与 [limit] 两节按单元名接成上游那一单元的键（上游重跑之后读数字节相同才可复用，所以这两节不同记 upstream 一类）。
第二段跨机对照那一行与 RC3 抽样按那一趟现算，不在计数里；单元总数 330 = 第 0–10 步 33 + 第一段 16 行 + 第二段 281 行。
判定四类：same（键相同，可复用）；upstream（只差 [upstream] 或 [limit]，条件式）；differs（别的节不同，写第一处不同的节）；
  fresh-always（第 11.1、11.3 节永远现跑：step-9、step-10-run-2、第一段 row-0，与要补位的行：按名字取不到、锚点不是恰 1 次、改坏那一处碰到模块集合）。
输出：count group=<组> verdict=<类> units=<数>，first_difference group=<组> section=<节> units=<数>，末行 total units=<数>；--per-unit 另逐单元一行。
退出码：0 出了计数；2 用法错或开头核不过；78 dep-info 里有树外的路径（S18 ②，闭包认不全）。
"""
from __future__ import annotations

import argparse
import collections
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import reuse_library as reuse  # noqa: E402

UNIT_SCRIPTS = ("segment_one_unit_revision_8.sh", "segment_one_unit_revision_11.sh")
WHOLE_TABLE_ROWS = tuple(range(1, 282))
GROUP_STEPS, GROUP_SEGMENT_ONE, GROUP_SEGMENT_TWO = "第 0–10 步单元", "第一段行", "第二段行"


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


def keys_of(inputs: reuse.TreeInputs, selected: dict[int, str]) -> dict[str, tuple[str, dict, str]]:
    """单元 → (键, 各节, normal 或 fresh-always)。上游与限时按单元名接上游那一单元的键。"""
    keys: dict[str, tuple[str, dict, str]] = {}

    def upstream_of(unit) -> list[str]:
        return [f"{role} {name} key={keys[name][0]}" for role, name in unit.upstream]

    for unit in reuse.SEGMENT_ONE_READING_UNITS:
        if unit.kind == "mutation-driver":
            continue
        sections = reuse.reading_unit_sections(inputs, unit, upstream_of(unit))
        keys[unit.name] = (reuse.key_of(sections), sections, "fresh-always" if unit.fresh_always else "normal")

    def add_row(row: reuse.RowUnit, limit_source: str, rerun: bool = False) -> None:
        limit = [f"max(1800,ceil(3*T)){'*3' if rerun else ''} T-of {limit_source} key={keys[limit_source][0]}"]
        sections, reason = reuse.row_sections(inputs, row, selected, limit, upstream_of(row))
        keys[row.name] = (reuse.key_of(sections), sections, "fresh-always" if reason else "normal")

    for row in reuse.step_4_row_units():
        add_row(row, "step-1")
    for row in reuse.segment_one_row_units(("s11", "s12")):
        add_row(row, "step-7")
    for row in reuse.segment_two_row_units(WHOLE_TABLE_ROWS, None):
        add_row(row, "step-1")
    mutations = [unit for unit in reuse.SEGMENT_ONE_READING_UNITS if unit.kind == "mutation-driver"][0]
    sections = reuse.reading_unit_sections(inputs, mutations, [])
    keys[mutations.name] = (reuse.key_of(sections), sections, "normal")
    return keys


def classify(first: tuple[str, dict, str], second: tuple[str, dict, str]) -> str:
    if "fresh-always" in (first[2], second[2]):
        return "fresh-always"
    if first[0] == second[0]:
        return "same"
    return "upstream" if reuse.first_differing_section(first[1], second[1]) in ("upstream", "limit") else "differs"


def group_of(unit: str) -> str:
    if unit.startswith("seg2-"):
        return GROUP_SEGMENT_TWO
    if unit.startswith("seg1-"):
        return GROUP_SEGMENT_ONE
    return GROUP_STEPS


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 修订 11 第 11.7 节：两棵树（或一处假想改动）上能复用的单元数")
    for name in ("--repo", "--tree-a", "--dep-a-device", "--dep-a-e170", "--toolchain"):
        parser.add_argument(name, required=True)
    for name in ("--tree-b", "--dep-b-device", "--dep-b-e170", "--hypothetical-change"):
        parser.add_argument(name)
    parser.add_argument("--per-unit", action="store_true")
    arguments = parser.parse_args(argument_list)
    if bool(arguments.hypothetical_change) == bool(arguments.tree_b and arguments.dep_b_device and arguments.dep_b_e170):
        parser.error("--tree-b（连同 --dep-b-device、--dep-b-e170）与 --hypothetical-change 恰给一样")
    repo = os.path.abspath(arguments.repo)
    for name in UNIT_SCRIPTS:
        problems = unit_script_problems(open(os.path.join(HERE, name), encoding="utf-8").read())
        if problems:
            print(f"单元表的参数在 {name} 里找不到：{problems}", file=sys.stderr)
            return 2
    toolchain = open(arguments.toolchain, encoding="utf-8").read()
    python = reuse.python_text()
    selected = reuse.selected_row_lines(repo)

    def inputs_of(tree: str, device: str, e170: str, change: str | None) -> reuse.TreeInputs:
        tree = os.path.abspath(tree)
        return reuse.tree_inputs(repo, tree, {"device": device, "e170": e170}, toolchain, python, {}, reuse.build_environment_lines(repo, tree), change)

    try:
        first = keys_of(inputs_of(arguments.tree_a, arguments.dep_a_device, arguments.dep_a_e170, None), selected)
        second = keys_of(inputs_of(arguments.tree_a, arguments.dep_a_device, arguments.dep_a_e170, arguments.hypothetical_change) if arguments.hypothetical_change
                         else inputs_of(arguments.tree_b, arguments.dep_b_device, arguments.dep_b_e170, None), selected)
    except reuse.ClosureOutsideTheTree as error:
        print(f"✗ {error}（S18 ②：闭包认不全，退 78）", file=sys.stderr)
        return 78
    counts: collections.Counter = collections.Counter()
    first_sections: collections.Counter = collections.Counter()
    for unit in first:
        verdict = classify(first[unit], second[unit])
        difference = reuse.first_differing_section(first[unit][1], second[unit][1])
        counts[(group_of(unit), verdict)] += 1
        if verdict == "differs":
            first_sections[(group_of(unit), difference)] += 1
        if arguments.per_unit:
            print(f"unit={unit} verdict={verdict} first_difference={difference} key_a={first[unit][0][:16]} key_b={second[unit][0][:16]}")
    for (group, verdict), number in sorted(counts.items()):
        print(f"count group={group} verdict={verdict} units={number}")
    for (group, section), number in sorted(first_sections.items()):
        print(f"first_difference group={group} section={section} units={number}")
    print(f"total units={len(first)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
