#!/usr/bin/env python3
# admission: always 每一次调都比此刻点名的两份产物，上一次的结论不替这一次作保
# run-condition: none 只读点名的两份产物文本，除了 python3 之外没有环境要求，不起 cargo、不编不跑、不碰装置
"""E184 重跑 r2（research/prompts/e184-r2-prereg.md 5.3 节）的逐格比：改前、改后两份长跑产物上，判定词有没有换。

只读产物文本的算术，独立手写，不 import crates/ 任何东西，不起装置。

用法：
  e184_r2_verdict_compare.py <改前产物> <改后产物>

读法（登记 1.1 节写死）：
  一格 = 一类行里的一把键：stop_check 键 item= 词 result=；anchor 键 identifier= 词 match=；selfcheck_done 一格词 clause_anchor_matches=；
  overlap 键 scope= 词 verdict=；share_group 键 scope= 词 verdict=。数值字段（detail= expected= actual=）、挂钟字段、name=segment 行不比。
  窄读 = stop_check 加 anchor 加 selfcheck_done（40 格）；宽读 = 窄读加 overlap 加 share_group（62 格），另加四条推出的条款
  F1、F2、F3、F6_second（式子在 derived_clauses）。宽读的 differing 是换词的格数加换边的条款数。
  行首有没有 E7RESULT 都认；去掉前缀之后第一个字段不是 name= 的行不是结果行。
  一边缺的键写 absent，同一份产物里同一把键出现两次写 duplicate；这两种记 keys_differ（登记停机 S4），不判翻不翻。

输出（登记 5.3 节写死的六种行）：name=cell、name=derived、name=anchor_recompute、name=void_check、name=verdict_comparison（窄读一行、宽读一行）、末行 name=done。
退出码：0 比完（flipped 与 not_flipped 都是 0；anchor_recompute 的 same=false 只在行里报，不改退出码，查法照登记停机 S3）；
  2 用法错或产物读不了、没写完（登记 XR4：末行不是 name=done emitted=行数，或 18 件里有一件的 name=segment 行不合）；5 任一读法 keys_differ（停机 S4）。
单测在同目录的 e184_r2_verdict_compare_test.py（登记 5.2 的对照，一条对照一个用例）；变异在同目录的 e184_r2_verdict_compare.mutations.tsv。
"""
import re
import sys
from decimal import Decimal, InvalidOperation
from math import ceil

import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

PREFIX = "E7RESULT "
TOKEN = re.compile(r'(?<!\S)(\w+)=("[^"]*"|\S*)')

# (行名, 键字段, 判定词字段)；键字段为 None 的那一类整份产物只有一格，键写成行名
FAMILY_FIELDS = (
    ("stop_check", "item", "result"),
    ("anchor", "identifier", "match"),
    ("selfcheck_done", None, "clause_anchor_matches"),
    ("overlap", "scope", "verdict"),
    ("share_group", "scope", "verdict"),
)
NARROW_FAMILIES = ("stop_check", "anchor", "selfcheck_done")
READINGS = (("narrow", NARROW_FAMILIES), ("wide", tuple(name for name, _, _ in FAMILY_FIELDS)))
KEY_PROBLEM_WORDS = ("absent", "duplicate")
RANDOM_SCOPE = "family:random_history_main"
OVERWRITE_SCOPE = "family:overwrite_chain_main"
HALF = Decimal("0.5")
# 登记 XR4：长跑 18 件，library 一件允许是 state=not_run reason=no_family_share_verdict_partial
SEGMENT_COUNT = 18
LIBRARY_NOT_RUN_REASON = "no_family_share_verdict_partial"


class Unreadable(Exception):
    pass


def parse_line(line):
    """一行的字段字典；行首有没有 E7RESULT 都认，去掉前缀之后第一个字段不是 name= 的行返回 None。同名字段取第一次出现的。"""
    body = line.strip()
    if body.startswith(PREFIX):
        body = body[len(PREFIX):].lstrip()
    if not body.startswith("name="):
        return None
    fields = {}
    for key, value in TOKEN.findall(body):
        fields.setdefault(key, value.strip('"'))
    return fields


def parse_rows(text):
    return [fields for fields in (parse_line(line) for line in text.splitlines()) if fields is not None]


def completeness_problems(rows):
    """登记 XR4 的两条：末行 name=done 且 emitted 等于结果行数；18 件的 name=segment 行都是 state=done（library 可以是约定的 not_run）。"""
    problems = []
    if not rows or rows[-1].get("name") != "done" or rows[-1].get("emitted") != str(len(rows)):
        problems.append(f"末行不是 name=done emitted={len(rows)}")
    segments = [row for row in rows if row.get("name") == "segment"]
    names = [row.get("segment") for row in segments]
    if len(segments) != SEGMENT_COUNT or len(set(names)) != SEGMENT_COUNT:
        problems.append(f"name=segment 行 {len(segments)} 条、不同的件 {len(set(names))} 个，应为 {SEGMENT_COUNT}")
    for row in segments:
        library_not_run = row.get("segment") == "library" and row.get("state") == "not_run" and row.get("reason") == LIBRARY_NOT_RUN_REASON
        if row.get("state") != "done" and not library_not_run:
            problems.append(f"件 {row.get('segment')} 的 state={row.get('state')}")
    return problems


def load_product(path):
    try:
        with open(path, "rb") as handle:
            text = handle.read().decode("utf-8", errors="replace")
    except OSError as error:
        raise Unreadable(f"{path} 读不了：{error}")
    rows = parse_rows(text)
    problems = completeness_problems(rows)
    if problems:
        raise Unreadable(f"{path} 没写完或件不齐：" + "；".join(problems))
    return rows


# ── 格：一类行里的一把键 ─────────────────────────────────────────────────────

def tables_of(rows):
    """{行名: ({键: 字段字典}, [出现两次的键])}；同一份产物里同一把键出现两次，记进第二项，格里留先出现的那一行。"""
    tables = {name: ({}, []) for name, _, _ in FAMILY_FIELDS}
    for fields in rows:
        for name, key_field, _ in FAMILY_FIELDS:
            if fields.get("name") != name:
                continue
            cells, duplicates = tables[name]
            key = name if key_field is None else fields.get(key_field, "missing")
            if key in cells:
                duplicates.append(key)
            else:
                cells[key] = fields
    return tables


def word_at(table, key, word_field):
    cells, duplicates = table
    if key in duplicates:
        return "duplicate"
    if key not in cells:
        return "absent"
    return cells[key].get(word_field, "missing")


def cell_rows_of(before_tables, after_tables, families):
    """每个范围、每把键一行 {family, key, before, after, same}；键的次序：改前的次序，改后多出来的接在后面。"""
    result = []
    for name, _, word_field in FAMILY_FIELDS:
        if name not in families:
            continue
        before_table, after_table = before_tables[name], after_tables[name]
        keys = list(before_table[0]) + [key for key in after_table[0] if key not in before_table[0]]
        keys += [key for key in before_table[1] + after_table[1] if key not in keys]
        for key in keys:
            before = word_at(before_table, key, word_field)
            after = word_at(after_table, key, word_field)
            result.append({"family": name, "key": key, "before": before, "after": after, "same": before == after})
    return result


# ── 推出的条款（登记第六节「负载判定词」） ──────────────────────────────────────

def fraction_of(row, field):
    try:
        return Decimal(row.get(field, ""))
    except InvalidOperation:
        return None


def derived_clauses(tables):
    """{条款: "true" / "false" / "absent"}（用到的行不在就是 absent）。
    F1 = 随机历史与覆盖写链两个主范围的 overlap 判定都是 overlap，且 above_fraction 都小于 0.5；
    F2 = 随机历史主范围的 share_group 判定是 renamed；F3 = 覆盖写链主范围的 share_group 判定是 renamed；
    F6_second = 随机历史主范围的 overlap 里 at_most_smoke_interval_maximum_fraction 大于等于 0.5。"""
    overlap, share = tables["overlap"][0], tables["share_group"][0]
    random_overlap, chain_overlap = overlap.get(RANDOM_SCOPE), overlap.get(OVERWRITE_SCOPE)
    random_share, chain_share = share.get(RANDOM_SCOPE), share.get(OVERWRITE_SCOPE)

    def text(value):
        return "absent" if value is None else str(value).lower()

    f1 = None
    if random_overlap is not None and chain_overlap is not None:
        pair = (random_overlap, chain_overlap)
        both_overlap = all(row.get("verdict") == "overlap" for row in pair)
        both_below = all(fraction_of(row, "above_fraction") is not None and fraction_of(row, "above_fraction") < HALF for row in pair)
        f1 = both_overlap and both_below
    f6_second = None
    if random_overlap is not None:
        fraction = fraction_of(random_overlap, "at_most_smoke_interval_maximum_fraction")
        f6_second = fraction is not None and fraction >= HALF
    return {"F1": text(f1),
            "F2": text(None if random_share is None else random_share.get("verdict") == "renamed"),
            "F3": text(None if chain_share is None else chain_share.get("verdict") == "renamed"),
            "F6_second": text(f6_second)}


def derived_rows_of(before_tables, after_tables):
    before, after = derived_clauses(before_tables), derived_clauses(after_tables)
    return [{"clause": clause, "before": before[clause], "after": after[clause], "same": before[clause] == after[clause]} for clause in before]


# ── 每个读法一行判定 ─────────────────────────────────────────────────────────

def reading_summary(reading, cell_rows, derived_rows):
    """name=verdict_comparison 一行的内容。宽读把推出的条款换边的个数也记进 differing；cells 只数格。"""
    differing = sum(1 for row in cell_rows if not row["same"])
    if reading == "wide":
        differing += sum(1 for row in derived_rows if not row["same"])
    key_problem = any(row["before"] in KEY_PROBLEM_WORDS or row["after"] in KEY_PROBLEM_WORDS for row in cell_rows)
    if key_problem:
        verdict = "keys_differ"
    else:
        verdict = "flipped" if differing else "not_flipped"
    return {"reading": reading, "cells": len(cell_rows), "differing": differing, "verdict": verdict}


# ── 十个锚的现算（登记第七节 7.2；整式路照段的角色，不读装置） ──────────────────────
# 段的角色抄自四份冒烟测试文件的注释：系统配置槽写 2 次（原地覆写，取三态）、journal 记录段、根槽写 1 次、单元段（写时复制，取两态）。

CONFIG_SLOT = ("system_configuration_slot", 2)
ROOT_SLOT = ("root_slot", 1)
RECORD_OF_TWO = ("journal_record", 2)


def unit_segment(writes):
    return ("unit", writes)


def record_segment(writes):
    return ("journal_record", writes)


def segment_two_state(segment):
    return 2 ** segment[1] - 1


def segment_full(segment):
    kind, writes = segment
    return 3 ** writes - 1 if kind == "system_configuration_slot" else 2 ** writes - 1


def segment_quick(segment):
    return 1 if segment[0] == "unit" else segment_full(segment)


def states_of(segments, per_segment):
    return 1 + sum(per_segment(segment) for segment in segments)


def writes_of(segments):
    return sum(segment[1] for segment in segments)


# 建快照流：取号写两轮，暖机两次各「记录 2、根槽 1、系统配置槽 2」，单元 24 与 16 各带「记录、根槽、系统配置槽」
SNAPSHOT_SEGMENTS = [CONFIG_SLOT, CONFIG_SLOT, RECORD_OF_TWO, ROOT_SLOT, CONFIG_SLOT, RECORD_OF_TWO, ROOT_SLOT, CONFIG_SLOT,
                     unit_segment(24), RECORD_OF_TWO, ROOT_SLOT, CONFIG_SLOT, unit_segment(16), RECORD_OF_TWO, ROOT_SLOT, CONFIG_SLOT]
SNAPSHOT_SEGMENTS_OLD_SHAPE = SNAPSHOT_SEGMENTS[1:]
NEW_POOL_SEGMENTS = SNAPSHOT_SEGMENTS[:12]
NEW_POOL_SEGMENTS_OLD_SHAPE = SNAPSHOT_SEGMENTS_OLD_SHAPE[:11]
# 多记录流的角色是推的：取号两轮，暖机两次与三次发布各「记录、根槽、系统配置槽」，后两次的记录段 4、6 写
MULTI_RECORD_SEGMENTS = [CONFIG_SLOT, CONFIG_SLOT, RECORD_OF_TWO, ROOT_SLOT, CONFIG_SLOT, RECORD_OF_TWO, ROOT_SLOT, CONFIG_SLOT,
                         unit_segment(24), RECORD_OF_TWO, ROOT_SLOT, CONFIG_SLOT, unit_segment(28), record_segment(4), ROOT_SLOT, CONFIG_SLOT,
                         unit_segment(30), record_segment(6), ROOT_SLOT, CONFIG_SLOT]
MULTI_RECORD_SEGMENTS_OLD_SHAPE = MULTI_RECORD_SEGMENTS[1:]
# 固定脚本到卸载：段形没钉，只有增量路：原登记写死的旧数加两个多出来的系统配置槽段（两态每段加 3、全量与甲二每段加 8）
FIXED_SCRIPT_OLD = {"two_state": 1678377089, "full": 1678377204, "quick": 278, "segments": 78}
FIXED_SCRIPT_ADDED_CONFIG_SLOT_SEGMENTS = 2
FIXED_SCRIPT_CONFIG_SLOT_SEGMENTS_NEW = 25
FIXED_SCRIPT_CONFIG_SLOT_SEGMENTS_OLD = 23
CONFIG_SLOT_FULL_MINUS_TWO_STATE = segment_full(CONFIG_SLOT) - segment_two_state(CONFIG_SLOT)
FIXED_SCRIPT_NEW = {"two_state": FIXED_SCRIPT_OLD["two_state"] + FIXED_SCRIPT_ADDED_CONFIG_SLOT_SEGMENTS * segment_two_state(CONFIG_SLOT),
                    "full": FIXED_SCRIPT_OLD["full"] + FIXED_SCRIPT_ADDED_CONFIG_SLOT_SEGMENTS * segment_full(CONFIG_SLOT),
                    "quick": FIXED_SCRIPT_OLD["quick"] + FIXED_SCRIPT_ADDED_CONFIG_SLOT_SEGMENTS * segment_full(CONFIG_SLOT),
                    "segments": FIXED_SCRIPT_OLD["segments"] + FIXED_SCRIPT_ADDED_CONFIG_SLOT_SEGMENTS}
STATES_PER_BLOCK = 65536
FIRST_HISTORY_SEED = 7463871032432355113  # 原登记 A4：随机历史种子基，不随段形变
CLAUSE_ANCHOR = "multi_record_full_named_by_the_clause"


def blocks_needed(states):
    return ceil(states / STATES_PER_BLOCK)


RECOMPUTED = {
    "snapshot_creation_two_state": states_of(SNAPSHOT_SEGMENTS, segment_two_state),
    "new_pool_two_state": states_of(NEW_POOL_SEGMENTS, segment_two_state),
    "new_pool_writes": writes_of(NEW_POOL_SEGMENTS),
    "blocks_needed_at_and_one_state_beyond_the_states_per_block": f"{blocks_needed(STATES_PER_BLOCK)},{blocks_needed(STATES_PER_BLOCK + 1)}",
    "first_history_seed": FIRST_HISTORY_SEED,
    "full_states_smoke-new_pool_file_creation_stream": states_of(NEW_POOL_SEGMENTS, segment_full),
    "full_states_smoke-overwrite_fixed_script_stream": FIXED_SCRIPT_NEW["full"],
    "full_states_smoke-multi_record_publish_stream": states_of(MULTI_RECORD_SEGMENTS, segment_full),
    "full_states_smoke-snapshot_creation_stream": states_of(SNAPSHOT_SEGMENTS, segment_full),
    CLAUSE_ANCHOR: states_of(MULTI_RECORD_SEGMENTS, segment_full),
}


def anchor_recompute_rows(after_tables):
    """十格：改后产物 selfcheck 件的 name=anchor 行，actual= 与现算值逐格比；expected=（装置钉值）也要等于现算值，
    只有条款锚（expected= 是条款写的 12230590578，登记预计它与现算不等，F6 第一条照旧触发）不比 expected=。"""
    result = []
    for identifier, value in RECOMPUTED.items():
        row = after_tables["anchor"][0].get(identifier)
        expected = "absent" if row is None else row.get("expected", "missing")
        actual = "absent" if row is None else row.get("actual", "missing")
        same = actual == str(value) and (identifier == CLAUSE_ANCHOR or expected == str(value))
        result.append({"identifier": identifier, "recomputed": str(value), "expected": expected, "actual": actual, "same": same})
    return result


# ── 作废类行的计数（登记 XR1、XR2、XR3 读它） ────────────────────────────────────

# (行名, 字段, 过关的取值)
VOID_FAMILIES = (("control", "passed", "true"), ("determinism", "equal", "true"), ("dump_done", "base_mismatches", "0"), ("pinned_smoke", "match", "true"))


def void_check_rows(rows):
    result = []
    for name, field, good_value in VOID_FAMILIES:
        selected = [row for row in rows if row.get("name") == name]
        result.append({"family": name, "cells": len(selected), "failing": sum(1 for row in selected if row.get(field) != good_value)})
    return result


# ── 一次比较 ─────────────────────────────────────────────────────────────────

def emit(lines, name, **fields):
    lines.append("E7RESULT name=" + name + "".join(f" {key}={value}" for key, value in fields.items()))


def flag(value):
    return str(value).lower()


def compare_rows(before_rows, after_rows):
    """两份产物的结果行（字段字典的列表）比一遍；返回 (输出行, 退出码)。"""
    before_tables, after_tables = tables_of(before_rows), tables_of(after_rows)
    lines = []
    summaries = []
    derived_rows = derived_rows_of(before_tables, after_tables)
    wide_cells = []
    for reading, families in READINGS:
        cell_rows = cell_rows_of(before_tables, after_tables, families)
        if reading == "wide":
            wide_cells = cell_rows
        summaries.append(reading_summary(reading, cell_rows, derived_rows))
    for row in wide_cells:
        emit(lines, "cell", family=row["family"], key=row["key"], before=row["before"], after=row["after"], same=flag(row["same"]))
    for row in derived_rows:
        emit(lines, "derived", clause=row["clause"], before=row["before"], after=row["after"], same=flag(row["same"]))
    for row in anchor_recompute_rows(after_tables):
        emit(lines, "anchor_recompute", identifier=row["identifier"], recomputed=row["recomputed"], expected=row["expected"],
             actual=row["actual"], same=flag(row["same"]))
    for row in void_check_rows(after_rows):
        emit(lines, "void_check", family=row["family"], cells=row["cells"], failing=row["failing"])
    for summary in summaries:
        emit(lines, "verdict_comparison", reading=summary["reading"], cells=summary["cells"], differing=summary["differing"],
             verdict=summary["verdict"])
    emit(lines, "done", emitted=len(lines) + 1)
    return lines, 5 if any(summary["verdict"] == "keys_differ" for summary in summaries) else 0


def main(argv):
    if len(argv) != 2:
        print(__doc__)
        return 2
    try:
        before_rows, after_rows = load_product(argv[0]), load_product(argv[1])
    except Unreadable as unreadable:
        print(f"  ✗ {unreadable}", file=sys.stderr)
        print("     → 怎么办：核命令行给的两个产物路径在、读得了、长跑已做完（登记 XR4），再同一条命令重跑", file=sys.stderr)
        return 2
    lines, code = compare_rows(before_rows, after_rows)
    print("\n".join(lines))
    if code == 5:
        print("  ✗ 两份产物的格对不齐（登记停机 S4）：见 name=verdict_comparison 里 verdict=keys_differ 的读法与 name=cell 里 absent、duplicate 的格", file=sys.stderr)
        print("     → 怎么办：两边查是装置还是长跑脚本改了行形，不判翻不翻", file=sys.stderr)
    return code


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
