"""E170 第 2 次重跑第二段：Q1 比对器（`research/prompts/e170-r2-prereg.md` 第一节读法 5、6，第六节 Q1-a…Q1-g、Q1-pc，第七节 B4–B7）。

只读两版的判定行、`bad-expected.tsv` 与已知坏镜像表的行名，不重算判定（5.4）。逐态逐字段比（`time_ns`、`worker` 除外）：v1 的 `strict=`、
`provisional=` 对 v2 的 `strict_before_witness=`、`provisional_before_witness=`；v2 的 `gaps=` 去掉 G23 起的新行再比（射程（b）单列）。
分类照读法 6：合法态 = I1、I2 的批态与 I3、I4 不打补丁的底态；补丁态 = I3、I4 的补丁行与已知坏镜像；I5 单独一类（单测，另由 --unit-tests 读）。

跑法：python3 comparator_two_versions.py --pair <输入名>:<states|images>:<v1 产物>:<v2 产物> [...] --batches 1,2
      --names <输入名>:<含 bad-expected.tsv 的导出目录>[:<已知坏镜像目录>] [...] --unit-tests <I5 产物>
      --anchors <v2 executor_main 产物> [...] --positive-control-input <输入名> --scratch <草稿目录> --output <新文件>
"""

from __future__ import annotations

import argparse
import os
import re
import sys
from dataclasses import dataclass

sys.dont_write_bytecode = True

FIELD_PATTERN = re.compile(r"(\w+)=(\S*)")
NOT_COMPARED = {"name", "time_ns", "worker", "input", "source", "label", "stream", "state"}
VERSION_ONE_TO_VERSION_TWO_FIELD = {"strict": "strict_before_witness", "provisional": "provisional_before_witness"}
FIRST_NEW_GAP_NUMBER = 23  # 上一次登记读法 5 的欠一句表：v2 新记的从 G23 起（登记 5.2 改四）
IN, NOT_IN, SPEC_GAP = "in", "not_in", "spec_gap"


def fields_of(line: str) -> dict[str, str]:
    return dict(FIELD_PATTERN.findall(line.rstrip("\n").removeprefix("E7RESULT ")))


def keyed_lines(path: str, kind: str, batches: set[str]) -> dict[tuple, dict[str, str]]:
    with open(path, encoding="utf-8") as handle:
        parsed = [fields_of(line) for line in handle]
    if kind == "states":
        return {(fields["stream"], fields["state"]): fields for fields in parsed
                if fields.get("name") == "state" and fields.get("batch") in batches}
    return {(fields["source"], fields["stream"], fields["state"]): fields for fields in parsed if fields.get("name") == "image_judgement"}


def is_new_gap(identifier: str) -> bool:
    match = re.fullmatch(r"G(\d+)", identifier)
    return match is not None and int(match.group(1)) >= FIRST_NEW_GAP_NUMBER


def gaps_without_new_rows(text: str) -> str:
    kept = [identifier for identifier in text.split(",") if identifier and identifier != "-" and not is_new_gap(identifier)]
    return ",".join(kept) or "-"


@dataclass(frozen=True)
class Classified:
    key: tuple
    category: str  # same / fields_differ / legal_flip / patched_flip_without_predicate / not_in_to_in / other_verdict_change / in_range_a / in_range_b
    detail: str


def classify(key: tuple, kind: str, version_one: dict[str, str], version_two: dict[str, str]) -> Classified:
    is_legal = kind == "states" or key[0] == "base"
    differing = []
    for field, value_one in version_one.items():
        if field in NOT_COMPARED:
            continue
        field_two = VERSION_ONE_TO_VERSION_TWO_FIELD.get(field, field)
        value_two = version_two.get(field_two)
        if field == "gaps" and value_two is not None:
            value_two = gaps_without_new_rows(value_two)
        if value_two != value_one:
            differing.append(f"{field}:{value_one}->{value_two}")
    strict_one, strict_two = version_one.get("strict"), version_two.get("strict")
    failed = version_two.get("failed_predicates", "-")
    detail = f"v1_strict={strict_one} v2_strict={strict_two} failed_predicates={failed}"
    if differing:
        return Classified(key, "fields_differ", detail + " fields=" + ";".join(differing))
    if strict_one == strict_two:
        return Classified(key, "same", detail)
    if strict_two == SPEC_GAP and strict_one != SPEC_GAP and any(is_new_gap(item) for item in version_two.get("gaps", "-").split(",")):
        return Classified(key, "in_range_b", detail + f" gaps={version_two.get('gaps')}")
    if strict_one == IN and strict_two == NOT_IN:
        if is_legal:
            return Classified(key, "legal_flip", detail)
        return Classified(key, "in_range_a" if failed != "-" else "patched_flip_without_predicate", detail)
    if strict_one == NOT_IN and strict_two == IN:
        return Classified(key, "not_in_to_in", detail)
    return Classified(key, "other_verdict_change", detail)


OUT_OF_RANGE = ("fields_differ", "legal_flip", "patched_flip_without_predicate", "not_in_to_in", "other_verdict_change")


def compare_input(label: str, kind: str, version_one_lines, version_two_lines, names: dict[tuple, str], emit, quiet: bool = False) -> dict[str, int]:
    common = sorted(set(version_one_lines) & set(version_two_lines))
    results = [classify(key, kind, version_one_lines[key], version_two_lines[key]) for key in common]
    counts = {category: sum(result.category == category for result in results) for category in OUT_OF_RANGE + ("in_range_a", "in_range_b", "same")}
    counts["compared"] = len(common)
    counts["missing"] = len(set(version_one_lines) ^ set(version_two_lines))
    counts["out_of_range_total"] = sum(counts[category] for category in OUT_OF_RANGE)
    legal_flips = [result for result in results if result.category == "legal_flip"]
    counts["legal_flip_with_W"] = sum("failed_predicates=W" in result.detail for result in legal_flips)
    counts["legal_flip_with_T"] = sum(result.detail.split("failed_predicates=")[1].split()[0] in ("T", "W,T") for result in legal_flips)
    before_not_in = [version_two_lines[key] for key in common if version_two_lines[key].get("strict_before_witness") == NOT_IN]
    counts["q1g_before_not_in_with_W_true"] = sum(fields.get("witness_true") == "true" for fields in before_not_in)
    counts["q1g_before_not_in_with_T_false"] = sum(fields.get("tail_check") == "false" for fields in before_not_in)
    if not quiet:
        for result in results:
            if result.category != "same":
                group = "q1_out_of_range" if result.category in OUT_OF_RANGE else "q1_in_range"
                emit(f"name={group} input={label} category={result.category} key={':'.join(result.key)} "
                     f"row_name={names.get(result.key, '-')} {result.detail}")
        emit(f"name=q1_counts input={label} kind={kind} " + " ".join(f"{key}={value}" for key, value in counts.items()))
    return counts


def read_rows(path: str) -> list[list[str]]:
    with open(path, encoding="utf-8") as handle:
        return [line.rstrip("\n").split("\t") for line in handle if line.strip() and line.split("\t")[0].isdigit()]


def row_names(dumps: str, known_bad_dir: str | None) -> dict[tuple, str]:
    """判完之后才读的行名：P、N、盲测各行取 `bad-expected.tsv` 第 2 列（族在第 3 列，底态在第 4 列）；底态取它的底态名；已知坏镜像取 `known-bad-o2.tsv` 第 2 列。"""
    names: dict[tuple, str] = {}
    for stream in ("new_pool_file_creation", "fixed_script_through_e_then_unmount"):
        for columns in read_rows(os.path.join(dumps, stream, "bad-expected.tsv")):
            names[("e170", stream, columns[0])] = f"{columns[1]}|family={columns[2]}|base={columns[3]}"
    if known_bad_dir:
        for columns in read_rows(os.path.join(known_bad_dir, "known-bad-o2.tsv")):
            names[("known_bad", "new_pool_file_creation", columns[0])] = f"known_bad_{columns[0]}|{columns[1]}"
    return names


def anchor_checks(label: str, version_one_lines, version_two_lines, names: dict[tuple, str], emit) -> None:
    """第七节 B5（P8 八格：v1 在、v2 不在、failed_predicates=W；流一 P8-F 的 W 字段）与 B6（N1–N5：W 假、T 真或不判、终判定同 v1）。"""
    for key, name in sorted(names.items()):
        if key not in version_two_lines or key not in version_one_lines:
            continue
        row_name = name.split("|")[0]
        one, two = version_one_lines[key], version_two_lines[key]
        if row_name in ("P8-F", "P8-Mk"):
            holds = one.get("strict") == IN and two.get("strict") == NOT_IN and two.get("failed_predicates") == "W"
            if key[1] == "new_pool_file_creation" and row_name == "P8-F":
                holds = holds and (two.get("witness_counter"), two.get("witness_branch"), two.get("witness_compared")) == ("3", "last_record", "2")
            emit(f"name=anchor_check id=B5 input={label} key={':'.join(key)} row_name={row_name} holds={str(holds).lower()} "
                 f"v1_strict={one.get('strict')} v2_strict={two.get('strict')} failed_predicates={two.get('failed_predicates')} "
                 f"witness_counter={two.get('witness_counter')} witness_branch={two.get('witness_branch')} witness_compared={two.get('witness_compared')}")
        if "|family=N|" in name:
            holds = (two.get("witness_true") == "false" and two.get("tail_check") in ("true", "not_evaluated")
                     and two.get("strict") == one.get("strict"))
            emit(f"name=anchor_check id=B6 input={label} key={':'.join(key)} row_name={row_name} holds={str(holds).lower()} "
                 f"v1_strict={one.get('strict')} v2_strict={two.get('strict')} witness_true={two.get('witness_true')} tail_check={two.get('tail_check')}")


def anchor_assertion_lines(path: str, emit) -> None:
    """第七节 B7：v2 跑 executor_main 时打的上一次登记 A1–A13 与本登记 B1–B3（`anchor_assertion`、`witness_anchor_assertion`）原样计数。"""
    with open(path, encoding="utf-8") as handle:
        parsed = [fields_of(line) for line in handle if line.startswith(("name=anchor_assertion ", "name=witness_anchor_assertion "))]
    outcomes = [f"{fields['id']}@{fields['stream']}={fields['holds']}" for fields in parsed]
    emit(f"name=anchor_assertions_of_v2 file={os.path.basename(path)} lines={len(parsed)} "
         f"holds_true={sum(fields['holds'] == 'true' for fields in parsed)} holds_false={sum(fields['holds'] == 'false' for fields in parsed)} "
         f"not_run={sum(fields['holds'] == 'not_run' for fields in parsed)} detail={','.join(outcomes)}")


def unit_test_lines(path: str, emit) -> None:
    """I5（Q1-e）：v1 冻结的单测文件原样对 v2 的模块跑，`python3 -m unittest -v` 的输出。逐条列不过的。"""
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    ran = re.search(r"^Ran (\d+) tests?", text, re.MULTILINE)
    not_passed = re.findall(r"^(\S+) \(([^)]+)\)[^\n]*\.\.\. (FAIL|ERROR)$", text, re.MULTILINE)
    # 「... ok」多数在同一行；被测代码往 stderr 打了一行的那几条，「ok」落在下一行行首（v1 自己的单测产物里同样如此）
    passed = len(re.findall(r"\.\.\. ok$", text, re.MULTILINE)) + len(re.findall(r"^ok$", text, re.MULTILINE))
    for test_name, test_class, outcome in not_passed:
        emit(f"name=i5_not_passed test={test_name} class={test_class} outcome={outcome}")
    emit(f"name=i5_counts ran={ran.group(1) if ran else 'unreadable'} passed={passed} not_passed={len(not_passed)} "
         f"final_ok={str(bool(re.search(r'^OK', text, re.MULTILINE))).lower()}")


def positive_control(label: str, kind: str, version_one_path: str, version_two_path: str, batches: set[str], scratch: str, emit) -> None:
    """Q1-pc：v2 输出复制一份、改掉一态的 `landed=`，射程外必须恰好多 1 项（第六节；不等于 1 → V5）。"""
    copy_path = os.path.join(scratch, f"q1-positive-control-{label}.out")
    with open(version_two_path, encoding="utf-8") as handle:
        lines = handle.readlines()
    wanted_name = "name=state " if kind == "states" else "name=image_judgement "
    position = next(index for index, line in enumerate(lines) if line.startswith(wanted_name)
                    and (kind != "states" or fields_of(line).get("batch") in batches))
    changed = re.sub(r"\blanded=(\S+)", lambda match: f"landed={match.group(1)}_changed_by_the_positive_control", lines[position], count=1)
    with open(copy_path, "x", encoding="utf-8") as handle:
        handle.writelines(lines[:position] + [changed] + lines[position + 1:])
    one = keyed_lines(version_one_path, kind, batches)
    before = compare_input(label, kind, one, keyed_lines(version_two_path, kind, batches), {}, emit, quiet=True)["out_of_range_total"]
    after = compare_input(label, kind, one, keyed_lines(copy_path, kind, batches), {}, emit, quiet=True)["out_of_range_total"]
    emit(f"name=q1_positive_control input={label} changed_key={':'.join(fields_of(changed)[part] for part in (('stream', 'state') if kind == 'states' else ('source', 'stream', 'state')))} "
         f"out_of_range_before={before} out_of_range_after={after} added={after - before} exactly_one={str(after - before == 1).lower()} copy={copy_path}")


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑第二段：Q1 比对器")
    parser.add_argument("--pair", action="append", required=True, help="<输入名>:<states|images>:<v1 产物>:<v2 产物>")
    parser.add_argument("--batches", default="1,2")
    parser.add_argument("--names", action="append", default=[], help="<输入名>:<含 bad-expected.tsv 的导出目录>[:<已知坏镜像目录>]")
    parser.add_argument("--unit-tests")
    parser.add_argument("--anchors", action="append", default=[])
    parser.add_argument("--positive-control-input", required=True)
    parser.add_argument("--scratch", required=True)
    parser.add_argument("--output", required=True)
    arguments = parser.parse_args(argument_list)
    batches = set(arguments.batches.split(","))
    names_by_input = {}
    for entry in arguments.names:
        parts = entry.split(":")
        names_by_input[parts[0]] = row_names(parts[1], parts[2] if len(parts) > 2 else None)
    output = open(arguments.output, "x", encoding="utf-8")
    emitted = 0

    def emit(line: str) -> None:
        nonlocal emitted
        emitted += 1
        output.write(line + "\n")

    emit(f"name=config batches={arguments.batches} pairs={len(arguments.pair)} not_compared={','.join(sorted(NOT_COMPARED))} "
         f"renamed=strict->strict_before_witness,provisional->provisional_before_witness first_new_gap=G{FIRST_NEW_GAP_NUMBER}")
    totals: dict[str, int] = {}
    pairs = {}
    for pair in arguments.pair:
        label, kind, version_one_path, version_two_path = pair.split(":", 3)
        pairs[label] = (kind, version_one_path, version_two_path)
        version_one = keyed_lines(version_one_path, kind, batches)
        version_two = keyed_lines(version_two_path, kind, batches)
        names = names_by_input.get(label, {})
        counts = compare_input(label, kind, version_one, version_two, names, emit)
        emit(f"name=b4_count input={label} v1_states={len(version_one)} v2_states={len(version_two)} "
             + (" ".join(f"{source}={sum(key[0] == source for key in version_two)}" for source in ("e170", "known_bad", "base")) if kind == "images" else
                " ".join(f"batch_{batch}={sum(fields.get('batch') == batch for fields in version_two.values())}" for batch in sorted(batches))))
        if kind == "images":
            anchor_checks(label, version_one, version_two, names, emit)
        for key, value in counts.items():
            totals[key] = totals.get(key, 0) + value
    for path in arguments.anchors:
        anchor_assertion_lines(path, emit)
    if arguments.unit_tests:
        unit_test_lines(arguments.unit_tests, emit)
    kind, version_one_path, version_two_path = pairs[arguments.positive_control_input]
    positive_control(arguments.positive_control_input, kind, version_one_path, version_two_path, batches, arguments.scratch, emit)
    emit("name=verdict " + " ".join(f"{key}={value}" for key, value in totals.items()))
    emitted += 1
    output.write(f"name=done emitted={emitted}\n")
    output.close()
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
