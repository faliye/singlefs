#!/usr/bin/env python3
"""E171 第 1 次重跑·修订 10：两趟第二段读数的对照（修订 6 那一趟对修订 8 那一趟；登记 research/prompts/e171-r1-prereg.md 修订 10 第 10.5 节）。
两趟第二段的比对器（segment_two_comparator.py）都写完之后才跑；比对器不引它，它也不改任何一趟的判定、不触发任何条款：每一行带 role=contrast，
没有 name=verdict 行（收尾一行 name=contrast_summary 只数格子）。第一段那份对照脚本 contrast_revision_6_against_revision_8.py 不改、不引。
只读两趟的比对器产物、逐行驱动合成产物的 -timing、行输出根旁的账 row-hosts.tsv，写一份新产物（已存在就拒绝）。

逐格并排的几样（第 10.5 节第 2 条）：
  ① 两趟比对器产物里的每一行：按行种认身份（IDENTITY_FIELDS；没列的行种按出现次序配），身份之外每个字段一行 name=contrast_cell，
     revision_6=… revision_8=…（一边没有写 absent）same=…；Q2 每次调用的顶层字段、内存对象、胶水的观测、O1-对拍（q2_set_call、q2_named_row_call），
     C672 七行（q2_named_row），各行四种「读法 × 批」的类（row 行的 category_*），四个 D（q3_denominator），第 125 行（q3_row_125）都在这里；
  ② Q2：两趟各自的集合（行号）、n、r、q2= 判定（name=contrast_q2）与集合的并里每一行在不在两边（name=contrast_q2_set_membership）；
  ③ Q3：四个 D 并排（name=contrast_q3_denominator）；两棵树之间类变了的行（每种「读法 × 批」一行，name=contrast_rows_category_changed）、
     触达与否翻了的行（每批一行，name=contrast_rows_touch_flipped；触达 = 跑成了且写流、调用结局、那一批看 journal 恢复的报告三样有一样不同，同读法 4）、
     补位各用了哪一行（name=contrast_substitution）；
  ④ 两趟各行装置那一步的耗时（-timing 的 row_device），按台分开（账 row-hosts.tsv 的 rows 那一步）：name=contrast_row_time，每台一行 name=contrast_row_time_by_host；
  ⑤ 两棵树的逐文件差表不另出：一行 name=contrast_tree_files 指到第一段对照产物里的 name=contrast_tree_file 那几行。
用法：
  contrast_segment_two_revision_10.py --config C --revision-8-comparator P --output O [--repo R]
      修订 6 那一趟取配置 H28（比对器产物、合成产物、-timing、行输出根、账、前缀后缀、sha256；先核 revision_10_placeholders.py 的 H28 那一格）；
      修订 8 那一趟取 H23（-timing 与账的位置）、修订 8 那一份配置的 H3（树的快照摘要），比对器产物由 --revision-8-comparator 给。
  contrast_segment_two_revision_10.py --selftest [--cells 格,…] [--rounds N]
退出码：0 写完；2 用法错（含产物已存在）；78 配置读不到、H28 没填齐或核不过、点名的产物不在。

弄坏开关 CONTRAST_R10_BREAK=<项>（只在 --selftest 起的子进程里认，CONTRAST_R10_SELFTEST=1）。项名、弄坏哪一处、打开后红在哪一格：
  identity-ignored        比对器的行一律按出现次序配，不按身份字段                    pairing-by-identity
  role-missing            格子行不带 role=contrast                                      every-line-carries-role
  summary-as-verdict      收尾那一行写成 name=verdict                                  every-line-carries-role
  touch-batch-1-only      批 2 的触达也拿批 1 那一遍恢复的报告算                        touch-flips
  category-first-batch    类变了的行每种读法只看批 1                                    q2-membership-and-categories
  row-host-ignored        行耗时一律记在本机                                            row-times-by-host
  h28-unchecked           H28 没核过也照写                                              h28-required
  sweep-skipped、temporary-kept（自证外壳 revision_8_selftest_harness.py 的两个）           no-leftover-processes、temporary-directory-removed
"""
from __future__ import annotations

import argparse
import json
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import frozen_tree_lines_revision_8 as judge  # noqa: E402
import revision_10_placeholders as placeholders  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402
from result_lines import ResultLines, field  # noqa: E402

BREAK_VARIABLE = "CONTRAST_R10_BREAK"
SELFTEST_VARIABLE = "CONTRAST_R10_SELFTEST"
EXIT_RUN_CONDITION = 78
REVISION_6_TREE_DIGEST = "7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4"
RUNS = ("revision_6", "revision_8")
# 第二段比对器各行种的身份字段（segment_two_comparator.py 写那一行时用来分格的字段）；没列的行种按出现次序配（例 baseline_stability：
# 两趟稳定性基线的路径不同，按出现次序配，路径那一格照实写不同）
IDENTITY_FIELDS = {
    "row": ("row",), "q2_set_row": ("row",), "q2_set_call": ("row", "stream", "call"), "q2_named_row": ("row",), "q2_named_row_call": ("row", "stream", "call"),
    "q2": (), "q2_observation": (), "q3_denominator": ("reading", "batch"), "q3_denominator_class": ("reading", "batch", "class"),
    "q3_self_crash_row": ("row",), "q3_row_125": ("row",), "not_located_row": ("row",), "substitution": ("row",), "untouched_row_red": ("row",),
    "unstable_memory_objects": (), "recovery_panic_sum": (), "driver_product": (), "driver_product_rows": (), "verdict": (),
}
SKIPPED_KINDS = ("config", "done")
READINGS = ("registration", "original_reading_5")
BATCHES = ("1", "2")
TOUCH_COLUMNS = ("write_stream_differs", "call_outcomes_differ")


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


def role() -> str:
    return "" if broken("role-missing") else " role=contrast"


def fields_of(line: str) -> list[tuple[str, str]]:
    body = line.rstrip("\n")
    for prefix in ("E7RESULT ", "E7TIMING "):
        if body.startswith(prefix):
            body = body[len(prefix):]
    return [tuple(token.split("=", 1)) for token in body.split(" ") if "=" in token]


def lines_of_kind(lines: list[str], kind: str) -> list[dict[str, str]]:
    return [dict(fields_of(line)) for line in lines if line.startswith(f"E7RESULT name={kind} ")]


def keyed(lines: list[str]) -> dict[tuple, dict[str, str]]:
    """(行种, 身份字段与值……) → 其余字段。按出现次序配的行种，身份里放第几次出现。"""
    found: dict[tuple, dict[str, str]] = {}
    seen: dict[str, int] = {}
    for line in lines:
        if not line.startswith("E7RESULT name="):
            continue
        values = dict(fields_of(line))
        kind = values.pop("name")
        if kind in SKIPPED_KINDS:
            continue
        names = None if broken("identity-ignored") else IDENTITY_FIELDS.get(kind)
        if names is None:
            seen[kind] = seen.get(kind, 0) + 1
            found[(kind, ("occurrence", str(seen[kind])))] = values
        else:
            found[(kind,) + tuple((name, values.pop(name, "absent")) for name in names)] = values
    return found


def contrast_cells(revision_6_lines: list[str], revision_8_lines: list[str]) -> list[str]:
    """① 逐格并排（不带 E7RESULT 前缀）。"""
    left, right = keyed(revision_6_lines), keyed(revision_8_lines)
    cells = []
    for identity in sorted(set(left) | set(right), key=lambda item: tuple(str(part) for part in item)):
        identity_text = " ".join(f"{name}={field(value)}" for name, value in identity[1:])
        old, new = left.get(identity, {}), right.get(identity, {})
        for name in sorted(set(old) | set(new)) or ["-"]:
            old_value, new_value = old.get(name, "absent"), new.get(name, "absent")
            cells.append(f"name=contrast_cell{role()} kind={identity[0]} {identity_text} field={field(name)} revision_6={field(old_value)} "
                         f"revision_8={field(new_value)} same={str(old_value == new_value).lower()}".replace("  ", " "))
    return cells


def q2_of(lines: list[str]) -> tuple[list[int], int, str]:
    """一趟的 Q2：(集合的行号, r = O1-对拍红的行数, q2= 判定)；集合取 name=q2 那一行的红与不红两列之并，判定取 verdict 行。"""
    q2 = (lines_of_kind(lines, "q2") or [{}])[-1]
    rows = sorted(int(token) for column in ("rows_o1_red", "rows_o1_not_red") for token in q2.get(column, "-").split(",") if token.isdigit())
    verdict = (lines_of_kind(lines, "verdict") or [{}])[-1]
    return rows, int(q2.get("o1_red_rows", "0") or 0), verdict.get("q2", "absent")


def q2_lines(revision_6_lines: list[str], revision_8_lines: list[str]) -> list[str]:
    """② 两趟各自的集合、n、r、判定，与集合的并里每一行在不在两边。"""
    found = {run: q2_of(lines) for run, lines in zip(RUNS, (revision_6_lines, revision_8_lines))}
    lines = [f"name=contrast_q2{role()} run={run} set_rows={','.join(map(str, rows)) or '-'} n={len(rows)} r={red} q2={verdict}"
             for run, (rows, red, verdict) in found.items()]
    for row in sorted(set(found["revision_6"][0]) | set(found["revision_8"][0])):
        sides = ["in" if row in found[run][0] else "out" for run in RUNS]
        lines.append(f"name=contrast_q2_set_membership{role()} row={row} revision_6={sides[0]} revision_8={sides[1]} same={str(sides[0] == sides[1]).lower()}")
    return lines


def rows_of(lines: list[str]) -> dict[int, dict[str, str]]:
    return {int(values["row"]): values for values in lines_of_kind(lines, "row") if values.get("row", "").isdigit()}


def touched(values: dict[str, str], batch: str) -> bool:
    """读法 4 的触达（比对器 row 行的三列）：跑成了，且写流、调用结局、那一批看 journal 那一遍恢复的报告有一样不同。"""
    if values.get("ran") != "true":
        return False
    recovery_batch = "1" if broken("touch-batch-1-only") else batch
    return any(values.get(column) == "true" for column in TOUCH_COLUMNS + (f"recovery_report_differs_batch_{recovery_batch}",))


def q3_lines(revision_6_lines: list[str], revision_8_lines: list[str]) -> list[str]:
    """③ 四个 D 并排；类变了的行；触达翻了的行；补位各用了哪一行。"""
    lines = []
    denominators = [{(values.get("reading"), values.get("batch")): values.get("counted_D", "absent") for values in lines_of_kind(source, "q3_denominator")}
                    for source in (revision_6_lines, revision_8_lines)]
    for reading in READINGS:
        for batch in BATCHES:
            old, new = (table.get((reading, batch), "absent") for table in denominators)
            lines.append(f"name=contrast_q3_denominator{role()} reading={reading} batch={batch} revision_6={old} revision_8={new} same={str(old == new).lower()}")
    old_rows, new_rows = rows_of(revision_6_lines), rows_of(revision_8_lines)
    numbers = sorted(set(old_rows) | set(new_rows))
    for reading in READINGS:
        for batch in BATCHES:
            column = f"category_{reading}_batch_{'1' if broken('category-first-batch') else batch}"
            changed = [row for row in numbers if old_rows.get(row, {}).get(column, "absent") != new_rows.get(row, {}).get(column, "absent")]
            lines.append(f"name=contrast_rows_category_changed{role()} reading={reading} batch={batch} count={len(changed)} rows={','.join(map(str, changed)) or '-'}")
    for batch in BATCHES:
        flipped = [row for row in numbers if touched(old_rows.get(row, {}), batch) != touched(new_rows.get(row, {}), batch)]
        lines.append(f"name=contrast_rows_touch_flipped{role()} batch={batch} count={len(flipped)} rows={','.join(map(str, flipped)) or '-'}")
    for row in numbers:
        sides = [rows.get(row, {}) for rows in (old_rows, new_rows)]
        if all(side.get("substituted_from", "-") in ("-", "absent", "") for side in sides):
            continue
        used = [side.get("mutated_row_name", "absent") if side.get("substituted_from", "-") not in ("-", "") else "-" for side in sides]
        lines.append(f"name=contrast_substitution{role()} row={row} revision_6={field(used[0])} revision_8={field(used[1])} same={str(used[0] == used[1]).lower()}")
    return lines


def ledger_hosts(path: str) -> dict[int, str]:
    """账 row-hosts.tsv（步、行、台号、趟、怎么来的）里整表那一步（rows）每行最后记的台号；账上没有的是本机驱动直接写的（第一段编排器 host_of 同一个读法）。"""
    hosts: dict[int, str] = {}
    for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
        parts = line.rstrip("\n").split("\t")
        if len(parts) >= 3 and parts[0] == "rows" and parts[1].isdigit():
            hosts[int(parts[1])] = parts[2]
    return hosts


def row_time_lines(run: str, timing_path: str, ledger_path: str) -> list[str]:
    """④ 一趟各行装置那一步的耗时，按台分开。"""
    hosts = {} if broken("row-host-ignored") else ledger_hosts(ledger_path)
    lines, by_host = [], {}
    for line in open(timing_path, encoding="utf-8") if os.path.exists(timing_path) else []:
        if not line.startswith("E7TIMING name=row_device "):
            continue
        values = dict(fields_of(line))
        row, device_ns = int(values.get("row", "0")), values.get("device_ns", "-")
        host = hosts.get(row, "1")
        lines.append(f"name=contrast_row_time{role()} run={run} row={row} host={host} device_ns={device_ns} device_exit={values.get('device_exit', '-')}")
        if device_ns.isdigit():
            count, total = by_host.get(host, (0, 0))
            by_host[host] = (count + 1, total + int(device_ns))
    lines += [f"name=contrast_row_time_by_host{role()} run={run} host={host} rows={count} device_seconds={total / 1e9:.3f}" for host, (count, total) in sorted(by_host.items())]
    return lines


def sides_from_config(repo: str, config_path: str, revision_8_comparator: str) -> dict[str, dict[str, str]]:
    """两趟的输入：比对器产物、-timing、账、树的快照摘要（相对仓根或绝对路径）。H28 先照 revision_10_placeholders.py 那一格核。"""
    config, revision_8_config = placeholders.load_configs(repo, config_path)
    outcome = placeholders.check_h28(repo, config, revision_8_config, "contrast")
    if not (outcome.filled and outcome.holds) and not broken("h28-unchecked"):
        raise judge.RunConditionError(f"H28（修订 6 那一趟第二段的清单）没填齐或核不过：{outcome.detail}；先跑 revision_10_placeholders.py check --phase contrast")
    h28 = judge.section(config, "H28")
    segment_two = placeholders.segment_two_of(config)
    sides = {"revision_6": {"comparator": str(h28.get("comparator_product")), "timing": str(h28.get("driver_timing")),
                            "ledger": str(h28.get("rows_hosts_ledger")), "tree": REVISION_6_TREE_DIGEST},
             "revision_8": {"comparator": revision_8_comparator, "timing": f"{segment_two.prefix}-rows-driver{segment_two.suffix}-timing.out",
                            "ledger": os.path.join(segment_two.rows_root + "-shards", "row-hosts.tsv"),
                            "tree": placeholders.segment_one_of(revision_8_config).snapshot_digest}}
    for run, side in sides.items():
        if not os.path.isfile(judge.resolve(repo, side["comparator"])):
            raise judge.RunConditionError(f"{run} 那一趟第二段的比对器产物不在：{side['comparator']}")
    return sides


def write_contrast(repo: str, sides: dict[str, dict[str, str]], output: str) -> int:
    texts = [open(judge.resolve(repo, sides[run]["comparator"]), encoding="utf-8").read().splitlines() for run in RUNS]
    lines = ResultLines(output)
    lines.result(f"name=config{role()} product=contrast_segment_two_revision_6_against_revision_8 tree_revision_6={sides['revision_6']['tree']} "
                 f"tree_revision_8={sides['revision_8']['tree']} comparator_revision_6={field(sides['revision_6']['comparator'])} "
                 f"comparator_revision_8={field(sides['revision_8']['comparator'])} "
                 f"note=两趟第二段各自判定之后的描述量：谁也不进谁的判据，不改任何一趟的判定、不触发任何条款")
    cells, q2, q3 = contrast_cells(texts[0], texts[1]), q2_lines(texts[0], texts[1]), q3_lines(texts[0], texts[1])
    times = [line for run in RUNS for line in row_time_lines(run, judge.resolve(repo, sides[run]["timing"]), judge.resolve(repo, sides[run]["ledger"]))]
    trees = [f"name=contrast_tree_files{role()} product=…-s1-contrast-revision-6-against-revision-8.out kind=contrast_tree_file "
             f"note=两棵树的逐文件差表不另出，引第一段对照产物里这一种行"]
    for body in cells + q2 + q3 + times + trees:
        lines.result(body)
    summary = "verdict" if broken("summary-as-verdict") else "contrast_summary"
    lines.result(f"name={summary}{role()} cells={len(cells)} differing_cells={sum(1 for body in cells if body.endswith(' same=false'))} "
                 f"q2_lines={len(q2)} q3_lines={len(q3)} row_time_lines={len(times)}")
    lines.finish()
    return 0


def main(argument_list: list[str]) -> int:
    refusal = harness.refuse_break_outside_selftest(BREAK_VARIABLE, SELFTEST_VARIABLE)
    if refusal:
        print(refusal, file=sys.stderr)
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 修订 10：两趟第二段读数的对照（role=contrast）")
    parser.add_argument("--config", required=True)
    parser.add_argument("--revision-8-comparator", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--repo", default=os.getcwd())
    arguments = parser.parse_args(argument_list)
    if os.path.exists(arguments.output):
        print(f"产物已存在，不覆盖：{arguments.output}", file=sys.stderr)
        return 2
    repo = os.path.abspath(arguments.repo)
    try:
        return write_contrast(repo, sides_from_config(repo, arguments.config, arguments.revision_8_comparator), arguments.output)
    except judge.RunConditionError as error:
        print(f"✗ {error}（退 78）", file=sys.stderr)
        print('     → 怎么办：照上面那句补齐运行条件（配置、冻结树、设备文件）再用同一条命令起', file=sys.stderr)
        return EXIT_RUN_CONDITION


# ── 自证：假的两份比对器产物、-timing、账与配置 ──
def fake_comparator(rows: dict[int, dict[str, str]], red: list[int], quiet: list[int], verdict: str, order: str = "up") -> list[str]:
    lines = ["E7RESULT name=config segment=s2 comparator=segment_two"]
    for number in sorted(rows, reverse=order == "down"):
        values = {"status": "ok", "device_exit": "0", "substituted_from": "-", "mutated_row_name": f"m{number}", "ran": "true",
                  "write_stream_differs": "false", "call_outcomes_differ": "false", "recovery_report_differs_batch_1": "false",
                  "recovery_report_differs_batch_2": "false"}
        values.update({f"category_{reading}_batch_{batch}": "untouched" for reading in READINGS for batch in BATCHES})
        values.update(rows[number])
        lines.append(f"E7RESULT name=row row={number} " + " ".join(f"{key}={value}" for key, value in values.items()))
    lines.append(f"E7RESULT name=q2 set_rows={len(red) + len(quiet)} o1_red_rows={len(red)} rows_o1_red={','.join(map(str, red)) or '-'} "
                 f"rows_o1_not_red={','.join(map(str, quiet)) or '-'} verdict={verdict}")
    lines += [f"E7RESULT name=q3_denominator reading={reading} batch={batch} rows={len(rows)} counted_D={len(rows) - 1}" for reading in READINGS for batch in BATCHES]
    lines += [f"E7RESULT name=verdict q2={verdict} q2_set_rows={len(red) + len(quiet)}", f"E7RESULT name=done emitted={len(lines) + 2}"]
    return lines


def cell_pairing_by_identity(scratch: str) -> tuple[bool, str]:
    old = fake_comparator({1: {}, 2: {"category_registration_batch_1": "counted"}}, [], [], "bing")
    new = fake_comparator({1: {}, 2: {"category_registration_batch_1": "self_crash"}}, [], [], "bing", order="down")
    differing = sorted({(dict(fields_of(cell))["kind"], dict(fields_of(cell))["field"]) for cell in contrast_cells(old, new) if cell.endswith(" same=false")})
    return differing == [("row", "category_registration_batch_1")], f"不同的格 {differing}（要只有第 2 行登记读法批 1 的类；修订 8 那一份行的次序倒过来）"


def cell_q2_membership_and_categories(scratch: str) -> tuple[bool, str]:
    rows = {3: {"category_registration_batch_2": "counted"}, 49: {}, 155: {}}
    old = fake_comparator(rows, [155], [49], "yi")
    new = fake_comparator({3: {}, 49: {}, 155: {}}, [], [49], "jia")
    lines = q2_lines(old, new) + q3_lines(old, new)
    wanted = {f"name=contrast_q2 role=contrast run=revision_6 set_rows=49,155 n=2 r=1 q2=yi", "name=contrast_q2 role=contrast run=revision_8 set_rows=49 n=1 r=0 q2=jia",
              "name=contrast_q2_set_membership role=contrast row=155 revision_6=in revision_8=out same=false",
              "name=contrast_rows_category_changed role=contrast reading=registration batch=2 count=1 rows=3",
              "name=contrast_rows_category_changed role=contrast reading=registration batch=1 count=0 rows=-"}
    return wanted <= set(lines), f"缺的行 {sorted(wanted - set(lines))}"


def cell_touch_flips(scratch: str) -> tuple[bool, str]:
    old = fake_comparator({5: {"recovery_report_differs_batch_2": "true"}, 6: {"ran": "false"}}, [], [], "bing")
    new = fake_comparator({5: {}, 6: {"ran": "false", "write_stream_differs": "true"}}, [], [], "bing")
    flipped = [line for line in q3_lines(old, new) if line.startswith("name=contrast_rows_touch_flipped")]
    return flipped == ["name=contrast_rows_touch_flipped role=contrast batch=1 count=0 rows=-", "name=contrast_rows_touch_flipped role=contrast batch=2 count=1 rows=5"], \
        f"{flipped}（第 5 行只在批 2 那一遍恢复的报告不同；第 6 行没跑成，不算触达）"


def cell_row_times_by_host(scratch: str) -> tuple[bool, str]:
    timing = judge.write_text(os.path.join(scratch, "timing.out"), "E7TIMING name=row_device row=1 device_exit=0 device_ns=2000000000\nE7TIMING name=stage_time x=1\n"
                                                                  "E7TIMING name=row_device row=2 device_exit=0 device_ns=1000000000\n")
    ledger = judge.write_text(os.path.join(scratch, "row-hosts.tsv"), "rows\t2\t1\tattempt-1\tlocal\nrows\t2\t2\tattempt-2\tfetched\ncross\t1\t2\tattempt-2\tfetched\n")
    lines = row_time_lines("revision_8", timing, ledger)
    hosts = {dict(fields_of(line))["row"]: dict(fields_of(line))["host"] for line in lines if line.startswith("name=contrast_row_time ")}
    by_host = [line for line in lines if line.startswith("name=contrast_row_time_by_host")]
    return hosts == {"1": "1", "2": "2"} and len(by_host) == 2, f"各行在哪台 {hosts}（第 2 行账上后记的是第二台；跨机那一步不算）；按台汇总 {len(by_host)} 行"


def fake_sides(scratch: str) -> tuple[str, dict[str, dict[str, str]]]:
    repo = os.path.join(scratch, "repo")
    sides = {}
    for run, verdict in zip(RUNS, ("yi", "jia")):
        comparator = judge.write_text(os.path.join(repo, f"{run}-comparator.out"), "\n".join(fake_comparator({1: {}, 2: {}}, [2], [], verdict)) + "\n")
        timing = judge.write_text(os.path.join(repo, f"{run}-timing.out"), "E7TIMING name=row_device row=1 device_exit=0 device_ns=5\n")
        sides[run] = {"comparator": comparator, "timing": timing, "ledger": os.path.join(repo, f"{run}-row-hosts.tsv"), "tree": run[-1] * 64}
    return repo, sides


def cell_every_line_carries_role(scratch: str) -> tuple[bool, str]:
    repo, sides = fake_sides(scratch)
    output = os.path.join(scratch, "contrast.out")
    write_contrast(repo, sides, output)
    lines = [line.rstrip("\n") for line in open(output, encoding="utf-8")]
    without_role = [line for line in lines if " role=contrast" not in line and not line.startswith("E7RESULT name=done ")]
    verdicts = [line for line in lines if line.startswith("E7RESULT name=verdict ")]
    good = (not without_role and not verdicts and any(line.startswith("E7RESULT name=contrast_summary ") for line in lines)
            and any("name=contrast_tree_files " in line for line in lines) and lines[-1].startswith("E7RESULT name=done "))
    return good, f"{len(lines)} 行；不带 role 的 {len(without_role)}；name=verdict 行 {len(verdicts)}（要 0）"


def cell_h28_required(scratch: str) -> tuple[bool, str]:
    repo, sides = fake_sides(scratch)
    revision_8_config = judge.write_text(os.path.join(scratch, "revision-8.json"), '{"H3": {"snapshot_digest": "' + "8" * 64 + '"}}')
    paths = {"comparator_product": sides["revision_6"]["comparator"], "driver_product": sides["revision_6"]["comparator"],
             "driver_timing": sides["revision_6"]["timing"], "rows_hosts_ledger": judge.write_text(sides["revision_6"]["ledger"], "")}
    h28 = dict(paths, rows_root=repo, product_prefix="p-", product_suffix="-r6", sha256={path: "0" * 64 for path in paths.values()})
    config = judge.write_text(os.path.join(scratch, "revision-10.json"), json.dumps(
        {"revision_8_config": revision_8_config, "H23": {"run_date": "2026-10-09", "run_suffix": "-r8", "dumps_root": "dumps"}, "H28": h28}))
    output = os.path.join(scratch, "contrast.out")
    code = main(["--config", config, "--revision-8-comparator", sides["revision_8"]["comparator"], "--output", output, "--repo", repo])
    return code == EXIT_RUN_CONDITION and not os.path.exists(output), f"H28 的 sha256 对不上：退 {code}（要 78），产物 {'写了' if os.path.exists(output) else '没写'}"


SELFTEST_CELLS = {"pairing-by-identity": cell_pairing_by_identity, "q2-membership-and-categories": cell_q2_membership_and_categories,
                  "touch-flips": cell_touch_flips, "row-times-by-host": cell_row_times_by_host, "every-line-carries-role": cell_every_line_carries_role,
                  "h28-required": cell_h28_required}
SELFTEST_SWITCHES = {"identity-ignored": "pairing-by-identity", "role-missing": "every-line-carries-role", "summary-as-verdict": "every-line-carries-role",
                     "touch-batch-1-only": "touch-flips", "category-first-batch": "q2-membership-and-categories", "row-host-ignored": "row-times-by-host",
                     "h28-unchecked": "h28-required"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
