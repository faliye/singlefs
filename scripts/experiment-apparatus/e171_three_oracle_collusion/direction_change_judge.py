#!/usr/bin/env python3
"""E171 第 3 次重跑 V6：判定方向变化的判法（research/prompts/e171-r3-prereg.md 第六节 V6 与 C1 至 C5，第九节 M6 至 M8）。

只按产物里的字段判，不按行号判：`expected`（该不该被恢复读出：in / not_in）、`target_is_red`、`impl_consulted_outcome`、红集（逐个不变量编号）。
旧产物与重出产物按键（行种、流、行号或目标……）配对，每一对差异落进五类里的一类：
  C1 放行变红      旧红集空、新红集非空。正常 = 该行 expected=not_in（或已知坏镜像条目，本身是坏的）且新增的每一项在不变量文档里有定义行；
                   有问题 = expected=in 的行出现新红；expected 读不到 = 归不到
  C2 多红一项      旧红集非空、新红集多出。规则同 C1
                   放行表（pinned-allowances.tsv）：expected=in 的行里，表里点名的 (实验行, 流, 表行号, 名字, 不变量) 那几项新红算正常；
                   同一行多出表外的任何一项仍是有问题；表里有一条在产物里找不到对应的新红（行没有、名字不符、该不变量没新增）判有问题（防表腐烂）。
                   不给 --allowances 就没有任何放行
  C3 实现结局变    impl_consulted_outcome 读成功与拒绝互变，或拒绝句变。正常 = expected=not_in、读成功变拒绝、拒绝句在产品源码里找得到；
                   有问题 = expected=in 的行出现拒绝，或 not_in 的行由拒绝变读成功；句子在源码里找不到 = 归不到
  C4 红变不红      旧红集里的某项新红集里没有。先要求目标不变量仍红（target_is_red=true），不红 = 有问题；
                   仍红时只核「这一项自己作目标的已知坏镜像条目今天仍红」，核不出或核过了都不下「正常」：交主 agent 过目，算归不到
  C5 其余漂移      落进已知的几类（字段改名、文件摘要、基底读数、落点水位、只差摘要、已知坏镜像钉值 / 只差序号 / 新增条目 / 补丁数、族汇总、流结构读数）
                   且这一对里没有判定词变化；落不进去 = 归不到
一对差异可以同时满足几类的规则，每一类各出一条 name=v6_finding；记账用的主类按 C1、C3、C4、C2、C5 的次序取第一个满足的，
各类行数（一对算 2 行，只在一边的算 1 行）之和要等于 diff 的行数，差一行就不算过。

退出码：0 = 没有「有问题」、没有「归不到」、记账相等；1 = 有；2 = 用法错。

子命令：
  judge       --old <旧产物> --new <重出产物> [--labels <含 bad_row 的产物> [--labels-mode same-table|seed-independent-prefix]]
              --invariants-directory <目录> --refusal-source <源码文件> [--entries <含 known_bad_entry 的产物>]
              [--allowances <放行表> --experiment-row <E171.n>]
              labels-mode：same-table = 两边是同一张坏状态表（每条流行数相同），按 (流, 行号) 取；
              seed-independent-prefix = 重出产物用占位种子、labels 产物用盲测种子：P 行与 N 行（family 不是 B 的前缀）与种子无关，按行号取；
              前缀之后的盲测行（family B，形状只有 P2、P3、P5，期望都是 not_in）一律记 not_in
  mutations   --scratch <目录>：第九节 M6 至 M8，在本文件的草稿拷贝上各改一处，跑 test_direction_change_judge.py，必须红在点名的用例
"""

from __future__ import annotations

import argparse
import collections
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass, field

PASS_TO_RED, STRICTER, OUTCOME_CHANGED, RED_TO_PASS, OTHER_DRIFT = "C1", "C2", "C3", "C4", "C5"
PRIMARY_ORDER = (PASS_TO_RED, OUTCOME_CHANGED, RED_TO_PASS, STRICTER, OTHER_DRIFT)
NORMAL, PROBLEM, UNPLACED = "normal", "problem", "unplaced"
EXPECTED_IN, EXPECTED_NOT_IN, KNOWN_BAD = "in", "not_in", "known_bad"
LINE_PREFIX = "E7RESULT"
# 字段名按下划线拆开，带这些词的是判定词：变了就不是漂移
JUDGEMENT_WORDS = frozenset({"red", "pass", "verdict", "equal", "equals", "outcome", "consulted", "ignored", "expected", "violation", "violations",
                             "mismatch", "mismatches", "failed", "refused", "ambiguous", "holds"})
# 已知坏镜像表搬进 crates 之后产物不再带的四格（换成一行 name=known_bad_entry_count count=…）
TABLE_MOVE_REMOVED_CHECKS = frozenset({"known_bad_test_file_sha256_is_the_copied_one", "known_bad_entry_count_here",
                                       "known_bad_entry_count_in_the_original_file", "known_bad_entry_names_equal_the_original_file_in_order"})
ORACLE_COLUMN_FIELDS = frozenset({"o1", "o2", "o3"})  # 整个字段名就是一列 oracle 的判定（o2=red）；o1_crash_judge_green 这类是态数
DEFINITION_LINE = re.compile(r"^\|\s*(I-\d+\.\d+)\s*\|")
INVARIANT_NAME = re.compile(r"I-\d+\.\d+")


@dataclass(frozen=True)
class Row:
    kind: str
    fields: dict
    text: str


@dataclass
class Finding:
    cls: str
    result: str
    key: tuple
    reason: str
    expected: str = "-"
    old: str = "-"
    new: str = "-"


@dataclass
class Pair:
    key: tuple
    old: Row | None
    new: Row | None
    findings: list = field(default_factory=list)
    primary: str = OTHER_DRIFT

    @property
    def lines(self) -> int:
        if self.old is None and self.new is None:
            return 0  # 放行表自己的审计行，不对应产物里的行
        return 2 if self.old is not None and self.new is not None else 1


def parse_line(line: str) -> Row:
    fields: dict = {}
    names = []
    for token in line.rstrip("\n").split(" ")[1:]:
        if "=" in token:
            name, value = token.split("=", 1)
            if name == "name":
                names.append(value)
            fields.setdefault(name, value)
    if len(names) > 1:
        fields.setdefault("row_name", names[1])  # bad_row 行第二个 name= 是坏状态行的名字（P5-Mk、N1-F……）
    return Row(names[0] if names else "?", fields, line.rstrip("\n"))


def key_of(row: Row) -> tuple:
    kind, fields = row.kind, row.fields
    if kind == "known_bad_entry":
        return (kind, fields["target"])
    if kind in ("e170_bad_row", "bad_row"):
        return (kind, fields["stream"], fields["row"])
    if kind in ("s1", "bad_s1", "s4"):
        return (kind, fields.get("stream", ""), fields.get("check", "").replace("in_place_subsets", "quick_tier"))
    if kind in ("file", "bad_file"):
        return (kind, fields.get("stream", ""), fields["file"].replace("in-place-subsets", "quick-tier"))
    if kind == "call":
        return (kind, fields["stream"], fields["call"])
    if kind == "batch":
        return (kind, fields["stream"], fields["batch"])
    if kind == "bad_base":
        return (kind, fields["stream"], fields["base"])
    if kind == "bad_family":
        return (kind, fields["stream"], fields["family"])
    if kind == "torn_scan":
        return (kind, fields["stream"], fields["segment"])
    return (kind, fields.get("stream", ""))


def load_rows(path: str) -> collections.OrderedDict:
    """键（带同键第几次出现）→ Row，保持产物里的次序。"""
    rows: collections.OrderedDict = collections.OrderedDict()
    seen: collections.Counter = collections.Counter()
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            if not line.startswith(LINE_PREFIX):
                continue
            row = parse_line(line)
            key = key_of(row)
            seen[key] += 1
            rows[key + (seen[key],)] = row
    return rows


def is_judgement_field(name: str) -> bool:
    """expected_landing 是落点水位（读数），不是该不该读出的判定词。"""
    return name in ORACLE_COLUMN_FIELDS or (name != "expected_landing" and any(word in JUDGEMENT_WORDS for word in name.split("_")))


def red_set(value: str | None) -> frozenset:
    return frozenset() if value in (None, "", "-") else frozenset(value.split(","))


def outcome_class(value: str | None) -> str:
    if value is None:
        return "other"
    if value.startswith("file_read"):
        return "read"
    if value.startswith("failed:"):
        return "failed"
    return "other"


def refusal_sentence(value: str) -> str:
    """impl_consulted_outcome 里的拒绝句：有 detail:_"…" 取引号里的，没有取错误种类名（空格在产物里写成下划线）。"""
    detail = re.search(r'detail:_"(.*?)"', value)
    if detail:
        return detail.group(1)
    return re.match(r"failed:([A-Za-z0-9]+)", value).group(1) if re.match(r"failed:([A-Za-z0-9]+)", value) else ""


def source_line_of(sentence: str, source_lines: list[str]) -> int | None:
    if not sentence:
        return None
    for number, text in enumerate(source_lines, start=1):
        if sentence in text.replace(" ", "_"):
            return number
    return None


def defined_invariants(directory: str) -> frozenset:
    names = set()
    for name in sorted(os.listdir(directory)):
        if name.endswith(".md"):
            with open(os.path.join(directory, name), encoding="utf-8") as handle:
                names.update(match.group(1) for match in map(DEFINITION_LINE.match, handle) if match)
    return frozenset(names)


ALLOWANCE_COLUMNS = ("experiment_row", "stream", "row", "name", "invariant", "reason", "evidence")
EXPERIMENT_ROW_NAME = re.compile(r"E\d+\.\d+")


class AllowanceTableError(ValueError):
    """放行表写得不对（列数、编号、重复）：用法错，退 2。"""


@dataclass(frozen=True)
class Allowance:
    experiment_row: str
    stream: str
    row: str
    name: str
    invariant: str
    reason: str
    evidence: str

    @property
    def slot(self) -> tuple:
        return (self.stream, self.row)


def load_allowances(path: str, defined: frozenset) -> list[Allowance]:
    """放行表：制表符分隔七列（ALLOWANCE_COLUMNS），空行与 # 开头的行跳过。多一列、少一列、不变量没有定义行、同一格重复都拒。"""
    entries: list[Allowance] = []
    with open(path, encoding="utf-8") as handle:
        for number, line in enumerate(handle, start=1):
            text = line.rstrip("\n")
            if not text.strip() or text.startswith("#"):
                continue
            cells = text.split("\t")
            if len(cells) != len(ALLOWANCE_COLUMNS):
                raise AllowanceTableError(f"{path}:{number}: 要 {len(ALLOWANCE_COLUMNS)} 列，读到 {len(cells)} 列")
            entry = Allowance(*cells)
            if not EXPERIMENT_ROW_NAME.fullmatch(entry.experiment_row) or not entry.row.isdigit() or not INVARIANT_NAME.fullmatch(entry.invariant):
                raise AllowanceTableError(f"{path}:{number}: 实验行、表行号或不变量编号的写法不对：{entry.experiment_row} {entry.row} {entry.invariant}")
            if entry.invariant not in defined:
                raise AllowanceTableError(f"{path}:{number}: {entry.invariant} 在不变量文档里没有定义行")
            if not all(cell.strip() for cell in cells):
                raise AllowanceTableError(f"{path}:{number}: 有空格子")
            if any(old.experiment_row == entry.experiment_row and old.slot == entry.slot and old.invariant == entry.invariant for old in entries):
                raise AllowanceTableError(f"{path}:{number}: 同一格重复")
            entries.append(entry)
    return entries


@dataclass
class Context:
    labels: dict  # (流, 行号) → expected
    defined: frozenset
    source_lines: list
    entry_target_is_red: dict  # 已知坏镜像条目的目标不变量 → target_is_red（取新产物）
    allowances: list = field(default_factory=list)  # 只放这一次判的那个实验行的条目
    names: dict = field(default_factory=dict)  # (流, 行号) → 坏状态行的名字（来自 labels 产物；bad_row 行自带）
    allowance_hits: set = field(default_factory=set)  # 真用上了的 (流, 行号, 不变量)


def expected_of(row: Row, context: Context) -> str | None:
    if row.kind == "known_bad_entry":
        return KNOWN_BAD
    if "expected" in row.fields:
        return row.fields["expected"]
    return context.labels.get((row.fields.get("stream"), row.fields.get("row")))


def red_field_of(row: Row) -> str:
    return {"e170_bad_row": "o2_red", "bad_row": "impl_pool_checker_red", "known_bad_entry": "patched_red"}[row.kind]


def pair_slot(row: Row) -> tuple:
    return (row.fields.get("stream"), row.fields.get("row"))


def row_name_of(row: Row, context: Context) -> str | None:
    """bad_row 行自带名字；e170_bad_row 行没有，取 labels 产物里同一格的名字。"""
    return row.fields.get("row_name") or context.names.get(pair_slot(row))


def allowed_new_reds(pair: Pair, added: list, context: Context) -> list:
    """放行表点名这一格（流、表行号、名字）的那几项新红；用上的记进 allowance_hits。"""
    slot, name = pair_slot(pair.new), row_name_of(pair.new, context)
    covered = []
    for allowance in context.allowances:
        if allowance.slot == slot and allowance.name == name and allowance.invariant in added:
            context.allowance_hits.add(slot + (allowance.invariant,))
            covered.append(allowance.invariant)
    return covered


def audit_allowances(new_rows: collections.OrderedDict, context: Context) -> list[Pair]:
    """放行表里每一条都要在产物里真用上；没用上的（行没有、名字不符、该项没新增）判有问题，不让表悄悄腐烂。"""
    audits = []
    for allowance in context.allowances:
        if allowance.slot + (allowance.invariant,) in context.allowance_hits:
            continue
        row = next((candidate for candidate in new_rows.values()
                    if candidate.kind in ("bad_row", "e170_bad_row") and pair_slot(candidate) == allowance.slot), None)
        if row is None:
            why = "产物里没有这一行"
        elif row_name_of(row, context) != allowance.name:
            why = f"名字不符：表里 {allowance.name}，产物里 {row_name_of(row, context)}"
        else:
            why = f"这一行没有新增 {allowance.invariant}，或它的 expected 不是 in"
        audit = Pair(("allowance",) + allowance.slot + (allowance.invariant,), None, None)
        audit.findings.append(Finding(STRICTER, PROBLEM, audit.key, f"放行条目在产物里找不到对应的新红：{why}（防表腐烂）"))
        audit.primary = STRICTER
        audits.append(audit)
    return audits


def judge_red_set_change(pair: Pair, context: Context) -> None:
    """C1、C2、C4：红集的增减。"""
    old, new = pair.old, pair.new
    field_name = red_field_of(new)
    old_red, new_red = red_set(old.fields.get(field_name)), red_set(new.fields.get(field_name))
    added, removed = sorted(new_red - old_red), sorted(old_red - new_red)
    expected = expected_of(new, context)
    shown = dict(expected=expected or "unknown", old=",".join(sorted(old_red)) or "-", new=",".join(sorted(new_red)) or "-")
    if added:
        cls = PASS_TO_RED if not old_red else STRICTER
        undefined = [name for name in added if name not in context.defined]
        allowed = allowed_new_reds(pair, added, context) if expected == EXPECTED_IN else []
        outside = [name for name in added if name not in allowed]
        if expected == EXPECTED_IN and allowed and not outside:
            reasons = "；".join(sorted({entry.reason for entry in context.allowances if entry.slot == pair_slot(pair.new) and entry.invariant in allowed}))
            pair.findings.append(Finding(cls, NORMAL, pair.key, f"expected=in 的行新增 {','.join(allowed)} 在放行表里：{reasons}", **shown))
        elif expected == EXPECTED_IN:
            kept = f"（放行表放行的 {','.join(allowed)} 不计）" if allowed else ""
            pair.findings.append(Finding(cls, PROBLEM, pair.key, f"expected=in 的行出现新红 {','.join(outside)}{kept}", **shown))
        elif expected is None:
            pair.findings.append(Finding(cls, UNPLACED, pair.key, "这一行读不到 expected", **shown))
        elif undefined:
            pair.findings.append(Finding(cls, UNPLACED, pair.key, f"新增的 {','.join(undefined)} 在不变量文档里没有定义行", **shown))
        else:
            pair.findings.append(Finding(cls, NORMAL, pair.key, f"expected={expected}，新增 {','.join(added)} 都有定义行", **shown))
    if removed:
        target_is_red = new.fields.get("target_is_red")
        if new.kind == "known_bad_entry" and target_is_red != "true":
            pair.findings.append(Finding(RED_TO_PASS, PROBLEM, pair.key, f"目标不变量 {new.fields['target']} 不红（target_is_red={target_is_red}），少红 {','.join(removed)}", **shown))
            return
        for name in removed:
            own_entry = context.entry_target_is_red.get(name)
            if own_entry == "false":
                pair.findings.append(Finding(RED_TO_PASS, PROBLEM, pair.key, f"{name} 自己作目标的已知坏镜像条目今天不红", **shown))
            elif own_entry is None:
                pair.findings.append(Finding(RED_TO_PASS, UNPLACED, pair.key, f"{name} 没有自己作目标的已知坏镜像条目，核对 (i) 无从谈起", **shown))
            else:
                pair.findings.append(Finding(RED_TO_PASS, UNPLACED, pair.key, f"{name} 自己作目标的条目今天仍红；定义差与逐步推导要主 agent 过目，这里不下正常", **shown))


def judge_outcome_change(pair: Pair, context: Context) -> None:
    """C3：impl_consulted_outcome 变了。"""
    old_value, new_value = pair.old.fields.get("impl_consulted_outcome"), pair.new.fields.get("impl_consulted_outcome")
    old_class, new_class = outcome_class(old_value), outcome_class(new_value)
    expected = expected_of(pair.new, context)
    shown = dict(expected=expected or "unknown", old=old_class, new=new_class)
    if "other" in (old_class, new_class):
        pair.findings.append(Finding(OUTCOME_CHANGED, UNPLACED, pair.key, f"结局词不认得：{old_value} → {new_value}", **shown))
        return
    if new_class == "read":
        verdict = PROBLEM if expected == EXPECTED_NOT_IN else UNPLACED
        reason = "not_in 的行由拒绝变读成功（产品放宽了）" if verdict == PROBLEM else "拒绝变读成功，expected 不是 not_in：交主 agent"
        pair.findings.append(Finding(OUTCOME_CHANGED, verdict, pair.key, reason, **shown))
        return
    sentence = refusal_sentence(new_value)
    line = source_line_of(sentence, context.source_lines)
    if expected == EXPECTED_IN:
        pair.findings.append(Finding(OUTCOME_CHANGED, PROBLEM, pair.key, f"expected=in 的行被产品拒绝：{sentence}", **shown))
    elif expected is None:
        pair.findings.append(Finding(OUTCOME_CHANGED, UNPLACED, pair.key, "这一行读不到 expected", **shown))
    elif line is None:
        pair.findings.append(Finding(OUTCOME_CHANGED, UNPLACED, pair.key, f"拒绝句在产品源码里找不到：{sentence}", **shown))
    else:
        pair.findings.append(Finding(OUTCOME_CHANGED, NORMAL, pair.key, f"expected=not_in，拒绝句 {sentence} 在源码第 {line} 行", **shown))


def family_summary_matches(pair: Pair, new_rows: collections.OrderedDict) -> bool:
    fields = pair.new.fields
    recount = sum(1 for row in new_rows.values()
                  if row.kind == "bad_row" and row.fields.get("stream") == fields["stream"] and row.fields.get("family") == fields["family"]
                  and row.fields.get("impl_both_pass") == "true")
    return str(recount) == fields.get("impl_both_pass")


def judge_other_drift(pair: Pair, new_rows: collections.OrderedDict) -> None:
    """C5：落不进红集、结局两类的差异。"""
    old, new = pair.old, pair.new
    if old is None or new is None:
        row = new or old
        if row.kind == "torn_scan":
            pair.findings.append(Finding(OTHER_DRIFT, NORMAL, pair.key, "撕裂扫描段号随段数移动"))
        elif row.kind == "known_bad_entry" and new is not None:
            verdict = NORMAL if new.fields.get("target_is_red") == "true" else PROBLEM
            pair.findings.append(Finding(OTHER_DRIFT, verdict, pair.key, "已知坏镜像新增条目" if verdict == NORMAL
                                         else f"新增的已知坏镜像条目目标不红（target_is_red={new.fields.get('target_is_red')}）"))
        elif row.kind == "s4" and new is None and row.fields.get("check") in TABLE_MOVE_REMOVED_CHECKS:
            pair.findings.append(Finding(OTHER_DRIFT, NORMAL, pair.key, "已知坏镜像表搬家：四格删了"))
        elif row.kind == "known_bad_entry_count" and old is None:
            entry_rows = sum(1 for candidate in new_rows.values() if candidate.kind == "known_bad_entry")
            counted = row.fields.get("count") == str(entry_rows)
            pair.findings.append(Finding(OTHER_DRIFT, NORMAL if counted else UNPLACED, pair.key,
                                         "已知坏镜像表搬家：条数一行，与条目行数相符" if counted else f"条数 {row.fields.get('count')} 与条目行数 {entry_rows} 不符"))
        else:
            pair.findings.append(Finding(OTHER_DRIFT, UNPLACED, pair.key, f"只在{'新' if new else '旧'}产物里的 {row.kind} 行"))
        return
    changed = sorted(name for name in set(old.fields) | set(new.fields) if old.fields.get(name) != new.fields.get(name))
    judgement = [name for name in changed if is_judgement_field(name)]
    if new.kind == "bad_family":
        verdict = NORMAL if family_summary_matches(pair, new_rows) else UNPLACED
        pair.findings.append(Finding(OTHER_DRIFT, verdict, pair.key, "族汇总随逐行放行数变，重数相符" if verdict == NORMAL else "族汇总与逐行重数对不上"))
        return
    if judgement:
        pair.findings.append(Finding(OTHER_DRIFT, UNPLACED, pair.key, f"判定词变了：{','.join(judgement)}"))
        return
    if set(old.fields) != set(new.fields):
        category = "字段改名"
    elif new.kind in ("file", "bad_file"):
        category = "文件摘要"
    elif new.kind == "bad_base":
        category = "基底读数"
    elif new.kind == "bad_row":
        category = "落点水位" if "expected_landing" in changed else "只差摘要"
    elif new.kind == "known_bad_entry":
        category = "补丁数" if "patches" in changed else "只差序号"
    elif new.kind == "s4":
        category = "已知坏镜像钉值"
    else:
        category = "流结构读数"
    pair.findings.append(Finding(OTHER_DRIFT, NORMAL, pair.key, category))


def classify(old_rows: collections.OrderedDict, new_rows: collections.OrderedDict, context: Context) -> list[Pair]:
    pairs = []
    for key in list(old_rows) + [key for key in new_rows if key not in old_rows]:
        old, new = old_rows.get(key), new_rows.get(key)
        if old is not None and new is not None and old.text == new.text:
            continue
        pair = Pair(key, old, new)
        if old is not None and new is not None and new.kind in ("e170_bad_row", "bad_row", "known_bad_entry"):
            field_name = red_field_of(new)
            if old.fields.get(field_name) != new.fields.get(field_name):
                judge_red_set_change(pair, context)
            if new.kind == "bad_row" and old.fields.get("impl_consulted_outcome") != new.fields.get("impl_consulted_outcome"):
                judge_outcome_change(pair, context)
            if not pair.findings:
                judge_other_drift(pair, new_rows)
        else:
            judge_other_drift(pair, new_rows)
        classes = {finding.cls for finding in pair.findings}
        pair.primary = next(cls for cls in PRIMARY_ORDER if cls in classes)
        pairs.append(pair)
    pairs.extend(audit_allowances(new_rows, context))
    return pairs


def count_diff_lines(old_path: str, new_path: str) -> int:
    completed = subprocess.run(["diff", old_path, new_path], capture_output=True, text=True, check=False)
    return sum(1 for line in completed.stdout.splitlines() if line[:2] in ("< ", "> "))


def report(pairs: list[Pair], diff_lines: int, emit) -> bool:
    lines_by_class: collections.Counter = collections.Counter()
    results: collections.Counter = collections.Counter()
    for pair in pairs:
        lines_by_class[pair.primary] += pair.lines
        for finding in pair.findings:
            results[finding.result] += 1
            if finding.cls != OTHER_DRIFT or finding.result != NORMAL:
                emit(f"name=v6_finding class={finding.cls} result={finding.result} key={'/'.join(map(str, finding.key))} "
                     f"expected={finding.expected} old={finding.old} new={finding.new} reason={finding.reason.replace(' ', '_')}")
    classified = sum(lines_by_class.values())
    emit("name=v6_class_lines " + " ".join(f"{cls}={lines_by_class[cls]}" for cls in PRIMARY_ORDER))
    emit("name=v6_other_drift_categories " + " ".join(
        f"{name}={count}" for name, count in sorted(collections.Counter(
            finding.reason for pair in pairs for finding in pair.findings if finding.cls == OTHER_DRIFT and finding.result == NORMAL).items())))
    accounting_equal = classified == diff_lines
    emit(f"name=v6_accounting diff_lines={diff_lines} classified_lines={classified} equal={str(accounting_equal).lower()}")
    emit(f"name=v6_result_counts normal={results[NORMAL]} problem={results[PROBLEM]} unplaced={results[UNPLACED]}")
    ok = results[PROBLEM] == 0 and results[UNPLACED] == 0 and accounting_equal
    emit(f"name=v6_verdict ok={str(ok).lower()} problems={results[PROBLEM]} unplaced={results[UNPLACED]} accounting_equal={str(accounting_equal).lower()}")
    return ok


LABEL_MODES = ("same-table", "seed-independent-prefix")
BLIND_FAMILY = "B"


def label_table(path: str | None, new_rows: collections.OrderedDict, mode: str = "same-table") -> tuple[dict, dict, str]:
    """bad_row 的 (流, 行号) → expected，与 (流, 行号) → 坏状态行的名字，加一句说明。只在两边每条流的行数相同时用，不同就不给标签。
    same-table：两边是同一张坏状态表，按 (流, 行号) 取。
    seed-independent-prefix：重出产物用占位种子、labels 产物用盲测种子。种子只进盲测行（family B）的造法，P 行与 N 行在盲测行之前、与种子无关，
    按行号取；盲测行（前缀之后）的形状只有 P2、P3、P5，期望都是 not_in，一律记 not_in，名字不给（种子不同，名字对不上）。
    前提读不出来（行号不连续、盲测行不在末尾、盲测行有非 not_in、各流前缀长度不同）就不给标签。"""
    if path is None:
        return {}, {}, "no-labels-given"
    labelled = [row for row in load_rows(path).values() if row.kind == "bad_row"]
    per_stream = collections.Counter(row.fields["stream"] for row in labelled)
    judged = collections.Counter(row.fields["stream"] for row in new_rows.values() if row.kind == "e170_bad_row")
    if not judged or per_stream != judged:
        return {}, {}, "rows-per-stream-differ"
    if mode == "same-table":
        return ({pair_slot(row): row.fields["expected"] for row in labelled},
                {pair_slot(row): row.fields["row_name"] for row in labelled if "row_name" in row.fields}, "ok")
    prefix_lengths = set()
    for stream in per_stream:
        ordered = sorted((row for row in labelled if row.fields["stream"] == stream), key=lambda row: int(row.fields["row"]))
        if [int(row.fields["row"]) for row in ordered] != list(range(len(ordered))):
            return {}, {}, "row-numbers-not-contiguous"
        prefix = next((index for index, row in enumerate(ordered) if row.fields.get("family") == BLIND_FAMILY), len(ordered))
        if any(row.fields.get("family") != BLIND_FAMILY or row.fields["expected"] != EXPECTED_NOT_IN for row in ordered[prefix:]):
            return {}, {}, "blind-rows-not-a-not_in-suffix"
        prefix_lengths.add(prefix)
    if len(prefix_lengths) != 1 or 0 in prefix_lengths:
        return {}, {}, "prefix-length-differs-between-streams"
    prefix_length = prefix_lengths.pop()
    labels = {pair_slot(row): (row.fields["expected"] if int(row.fields["row"]) < prefix_length else EXPECTED_NOT_IN) for row in labelled}
    names = {pair_slot(row): row.fields["row_name"] for row in labelled if int(row.fields["row"]) < prefix_length and "row_name" in row.fields}
    return labels, names, f"ok-prefix-{prefix_length}"


def judge_files(old_path: str, new_path: str, labels_path: str | None, invariants_directory: str, source_path: str,
                entries_path: str | None, allowances_path: str | None = None, experiment_row: str | None = None,
                labels_mode: str = "same-table", emit=print) -> bool:
    old_rows, new_rows = load_rows(old_path), load_rows(new_path)
    with open(source_path, encoding="utf-8") as handle:
        source_lines = handle.read().splitlines()
    entries = load_rows(entries_path or new_path)
    defined = defined_invariants(invariants_directory)
    allowances: list = []
    if allowances_path is not None:
        if experiment_row is None:
            raise AllowanceTableError("给了 --allowances 就要给 --experiment-row")
        allowances = [entry for entry in load_allowances(allowances_path, defined) if entry.experiment_row == experiment_row]
    labels, names, label_note = label_table(labels_path, new_rows, labels_mode)
    context = Context(labels, defined, source_lines,
                      {row.fields["target"]: row.fields.get("target_is_red") for row in entries.values() if row.kind == "known_bad_entry"},
                      allowances, names)
    emit(f"name=v6_config old={os.path.basename(old_path)} new={os.path.basename(new_path)} labels={'given' if context.labels else 'none'} "
         f"labels_mode={labels_mode} labels_note={label_note} allowances={len(allowances)} experiment_row={experiment_row or '-'} "
         f"defined_invariants={len(context.defined)} refusal_source_lines={len(source_lines)}")
    return report(classify(old_rows, new_rows, context), count_diff_lines(old_path, new_path), emit)


# ---- 第九节 M6 至 M8：在本文件的草稿拷贝上各改一处，跑单测，必须红在点名的用例 ----
MUTATIONS = (
    ("M6-expected-in-new-red-accepted", 'elif expected == EXPECTED_IN:\n            kept = ',
     'elif expected == EXPECTED_IN and False:\n            kept = ', "test_a_new_red_on_an_expected_in_row_is_a_problem"),
    ("M12-allowance-ignores-other-new-reds", "\n        if expected == EXPECTED_IN and allowed and not outside:", "\n        if expected == EXPECTED_IN and allowed:",
     "test_another_new_red_on_the_pinned_row_is_still_a_problem"),
    ("M13-allowance-ignores-the-row-name", "\n        if allowance.slot == slot and allowance.name == name and allowance.invariant in added:",
     "\n        if allowance.slot == slot and allowance.invariant in added:", "test_an_allowance_naming_a_different_row_than_the_product_has_is_not_applied"),
    ("M14-allowance-audit-off", "\n    pairs.extend(audit_allowances(new_rows, context))", "\n    pairs.extend([])", "test_a_stale_allowance_is_a_problem"),
    ("M7-target-not-required-red", 'target_is_red = new.fields.get("target_is_red")\n        if new.kind == "known_bad_entry" and target_is_red != "true":',
     'target_is_red = new.fields.get("target_is_red")\n        if False:',
     "test_a_target_that_stops_being_red_is_a_problem"),
    ("M8-relaxed-refusal-not-seen", '    if new_class == "read":\n        verdict = PROBLEM if expected == EXPECTED_NOT_IN else UNPLACED',
     '    if new_class == "read":\n        verdict = NORMAL', "test_a_refusal_turned_into_a_read_on_a_not_in_row_is_a_problem"),
)


def run_mutations(scratch: str) -> bool:
    here = os.path.dirname(os.path.abspath(__file__))
    all_red = True
    for name, old, new, test_name in MUTATIONS:
        work = os.path.join(scratch, f"mutation-{name}")
        shutil.rmtree(work, ignore_errors=True)
        os.makedirs(work)
        for file_name in ("direction_change_judge.py", "test_direction_change_judge.py", "pinned-allowances.tsv"):
            shutil.copy(os.path.join(here, file_name), os.path.join(work, file_name))
        path = os.path.join(work, "direction_change_judge.py")
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        if text.count(old) != 1:
            print(f"name=v6_mutation mutation={name} applied=false matches={text.count(old)}")
            all_red = False
            continue
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text.replace(old, new))
        completed = subprocess.run([sys.executable, "-B", "-m", "unittest", "-q", "test_direction_change_judge"], cwd=work,
                                   env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"), capture_output=True, text=True, check=False)
        red = sorted(set(re.findall(r"^(?:FAIL|ERROR): (\S+)", completed.stderr, flags=re.MULTILINE)))
        caught = completed.returncode != 0 and any(test_name in test for test in red)
        all_red = all_red and caught
        print(f"name=v6_mutation mutation={name} applied=true exit_code={completed.returncode} must_be_red={test_name} caught={str(caught).lower()} "
              f"red_tests={','.join(red) or '-'}")
    print(f"name=v6_mutation_verdict ok={str(all_red).lower()}")
    return all_red


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    commands = parser.add_subparsers(dest="command", required=True)
    judge = commands.add_parser("judge")
    judge.add_argument("--old", required=True)
    judge.add_argument("--new", required=True)
    judge.add_argument("--labels")
    judge.add_argument("--entries")
    judge.add_argument("--labels-mode", choices=LABEL_MODES, default="same-table")
    judge.add_argument("--allowances")
    judge.add_argument("--experiment-row")
    judge.add_argument("--invariants-directory", required=True)
    judge.add_argument("--refusal-source", required=True)
    mutations = commands.add_parser("mutations")
    mutations.add_argument("--scratch", required=True)
    arguments = parser.parse_args(argument_list)
    if arguments.command == "mutations":
        os.makedirs(arguments.scratch, exist_ok=True)
        return 0 if run_mutations(arguments.scratch) else 1
    try:
        ok = judge_files(arguments.old, arguments.new, arguments.labels, arguments.invariants_directory, arguments.refusal_source, arguments.entries,
                         arguments.allowances, arguments.experiment_row, arguments.labels_mode)
    except AllowanceTableError as error:
        print(f"放行表有误：{error}", file=sys.stderr)
        return 2
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
