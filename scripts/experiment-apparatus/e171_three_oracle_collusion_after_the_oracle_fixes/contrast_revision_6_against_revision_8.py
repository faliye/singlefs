#!/usr/bin/env python3
"""E171 第 1 次重跑第一段：修订 6 那一趟对修订 8 那一趟的对照（登记 research/prompts/e171-r1-prereg.md 修订 8 第 8.5 节）。
两趟各自的比对器（segment_one_comparator.py）判定都写完之后才跑；比对器不引它，它也不改任何一趟的判定、不触发任何条款：每一行带 role=contrast，
没有 name=verdict 行（收尾一行 name=contrast_summary 只数格子）。只读两趟的产物、导出与两棵树，写一份新产物（已存在就拒绝）。

逐格并排的几样：
  ① 两趟比对器产物里的每一行：按行种认身份（下面 IDENTITY_FIELDS；没列的行种按出现次序配），身份之外每个字段一行 name=contrast_cell，
     revision_6=… revision_8=…（一边没有写 absent）same=…；Q1–Q5 的各格（红态数、适用数、峰值、为正段数、期末值、判定……）都在这里；
  ② 第 1、6、7、10 步之后那几样比、③ 两棵树的逐文件差表：都要修订 6 那一趟的树（快照摘要 7726cdd6…）与它的导出，两样造不回来
     （第 2 次重跑登记 research/prompts/e171-r2-prereg.md 修订 6：用户 2026-11-02 选第一段对照只出 ①④、②③ 记「修订 6 树与导出造不回来」）。
     这两项不比、不起任何比对脚本，各写一行 name=contrast_s15 / name=contrast_tree_file status=not_reconstructible reason=…（② 按步各一行），
     收尾那一行这两格也写 not_reconstructible，不写数（读不到不当 0）；
  ④ 两趟第 11–13 步各行装置那一步的耗时（行产物的 device_ns），按台分开（编排器的账 row-hosts.tsv）：name=contrast_row_time，每台一行汇总 name=contrast_row_time_by_host；
     那一趟的账不在（修订 6 那一趟的导出根已不在，H15 的 dumps_root 可以不填）的，各行的台写 ledger_absent，不当本机；
     每份行产物在不在各写一行 name=contrast_row_product present=…（不在的那几份不悄悄当没有行）。
用法：
  contrast_revision_6_against_revision_8.py --config C --revision-8-comparator P --output O [--repo R]
      修订 6 那一趟的清单取配置 H15（比对器产物、产物前缀与后缀、第 11–13 步的 RESUME_TAG；导出根可以不填）；修订 8 那一趟取 H3、H16 与单元编排的序列日志
      （各单元绿的那一次的 TAG）。产物前缀可以是绝对路径（修订 6 那一趟的行产物已归档出工作区，主 agent 从 git 取出来放在哪就写哪）。
  contrast_revision_6_against_revision_8.py --selftest [--cells 格,…] [--rounds N]
退出码：0 写完；2 用法错；78 配置读不到、H15 缺 ①④ 要的那几项、或比对器产物不在。

弄坏开关 CONTRAST_R8_BREAK=<项>（只在 --selftest 起的子进程里认，CONTRAST_R8_SELFTEST=1）。逐项写开关名、弄坏哪一处，下一行起写打开后红的格（自证打「✗ 自证 <格名>：…」）：
  CONTRAST_R8_BREAK=identity-ignored  比对器的行一律按出现次序配，不按身份字段
      「✗ 自证 pairing-by-identity：」
  CONTRAST_R8_BREAK=role-missing  格子行不带 role=contrast
      「✗ 自证 every-line-carries-role：」
  CONTRAST_R8_BREAK=not-reconstructible-as-zero  ②③ 造不回来的那两格在收尾那一行写成 0
      「✗ 自证 parts-not-reconstructible：」
  CONTRAST_R8_BREAK=row-host-ignored  行耗时一律记在本机
      「✗ 自证 row-times-by-host：」
  CONTRAST_R8_BREAK=absent-ledger-as-host-1  那一趟的账不在时各行记在本机（改前的取法）
      「✗ 自证 row-times-without-ledger：」
  CONTRAST_R8_BREAK=sweep-skipped  每一格之后与末尾不收进程（自证外壳 revision_8_selftest_harness.py 的开关）
      「✗ 自证 no-leftover-processes：」
  CONTRAST_R8_BREAK=temporary-kept  末尾不删临时目录（自证外壳 revision_8_selftest_harness.py 的开关）
      「✗ 自证 temporary-directory-removed：」
"""
from __future__ import annotations

import argparse
import os
import sys

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import frozen_tree_lines_revision_8 as judge  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402
from result_lines import ResultLines, field  # noqa: E402

BREAK_VARIABLE = "CONTRAST_R8_BREAK"
SELFTEST_VARIABLE = "CONTRAST_R8_SELFTEST"
EXIT_RUN_CONDITION = 78
NOT_RECONSTRUCTIBLE = "not_reconstructible"
NOT_RECONSTRUCTIBLE_REASON = "revision_6_tree_and_exports_not_reconstructible"   # 第 2 次重跑登记修订 6
S15_STEPS = ("1", "6", "7", "10-run-1", "10-run-2")   # ② 原来比的那几步（第 1、6、7、10 步之后）
LEDGER_ABSENT = "ledger_absent"
H15_KEYS_FOR_CELLS_AND_ROW_TIMES = ("comparator_product", "product_prefix", "product_suffix", "rows_tag")   # ①④ 要的；dumps_root 只给 ④ 的台，可以不填
REVISION_6_TREE_DIGEST = "7726cdd6c439f0bc9e39799755a53ebcfb3f57fe96b06daf13c1317ae81be5f4"
# 比对器各行种的身份字段（segment_one_comparator.py 写那一行时用来分格的字段）；没列的行种按出现次序配
IDENTITY_FIELDS = {
    "a1": ("batch",), "a8": ("control",), "a11": ("stream",), "z1": ("stream", "file"), "q1": ("reading", "streams", "batch"),
    "q1_row": ("row", "stream", "batch"), "q2_named_row": ("row",), "q2_named_row_call": ("row", "stream", "call"), "q3": ("row",),
    "q4": ("trip", "stream", "batch"), "q4_trip": ("trip",), "q4_counter_branch_state": ("trip", "stream", "batch", "ordinal"),
    "q4_trajectory": ("trip", "stream", "batch"), "q4_trajectory_by_inode_creation": ("trip", "stream", "batch"),
    "q5_judged": ("sampling_point",), "q5_trajectory": ("sampling_point",), "q5_trajectory_cell": ("sampling_point", "writes_landed_in_the_last_segment"),
    "o3_third_stream_coverage": ("stream",), "a1_empty_mutation": ("batch",), "a1_empty_mutation_touch": ("row",), "verdict": (), "z1_unstable_memory_objects": (),
}
SKIPPED_KINDS = ("config", "done")
ROW_PRODUCT_STEMS = ("rows-probe-driver", "rows-driver", "rows-cross-host-row-0-driver", "rows-probe-timeout-rerun-driver", "rows-timeout-rerun-driver")
STEP_OF_STEM = {"rows-probe-driver": "s11", "rows-driver": "s12", "rows-cross-host-row-0-driver": "sx", "rows-probe-timeout-rerun-driver": "r11",
                "rows-timeout-rerun-driver": "r12"}


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


def fields_of(line: str) -> list[tuple[str, str]]:
    body = line.rstrip("\n")
    if body.startswith("E7RESULT "):
        body = body[len("E7RESULT "):]
    return [tuple(token.split("=", 1)) for token in body.split(" ") if "=" in token]


def keyed_lines(lines: list[str]) -> dict[tuple, dict[str, str]]:
    """比对器产物的各行：(行种, 身份字段的值……) → 其余字段。按出现次序配的行种，身份里放第几次出现。"""
    keyed: dict[tuple, dict[str, str]] = {}
    occurrences: dict[str, int] = {}
    for line in lines:
        if not line.startswith("E7RESULT name="):
            continue
        pairs = fields_of(line)
        values = dict(pairs)
        kind = values.pop("name")
        if kind in SKIPPED_KINDS:
            continue
        identity_fields = None if broken("identity-ignored") else IDENTITY_FIELDS.get(kind)
        if identity_fields is None:
            occurrences[kind] = occurrences.get(kind, 0) + 1
            identity = (kind, ("occurrence", str(occurrences[kind])))
        else:
            identity = (kind,) + tuple((name, values.pop(name, "absent")) for name in identity_fields)
        keyed[identity] = values
    return keyed


def contrast_cells(revision_6_lines: list[str], revision_8_lines: list[str]) -> list[str]:
    """逐格并排的行（不带 E7RESULT 前缀）。"""
    left, right = keyed_lines(revision_6_lines), keyed_lines(revision_8_lines)
    cells = []
    role = "" if broken("role-missing") else " role=contrast"
    for identity in sorted(set(left) | set(right), key=lambda item: tuple(str(part) for part in item)):
        kind, parts = identity[0], identity[1:]
        identity_text = " ".join(f"{name}={field(value)}" for name, value in parts)
        old, new = left.get(identity, {}), right.get(identity, {})
        for name in sorted(set(old) | set(new)) or ["-"]:
            old_value, new_value = old.get(name, "absent"), new.get(name, "absent")
            cells.append(f"name=contrast_cell{role} kind={kind} {identity_text} field={field(name)} revision_6={field(old_value)} "
                         f"revision_8={field(new_value)} same={str(old_value == new_value).lower()}".replace("  ", " "))
    return cells


class RunConditionError(Exception):
    """配置读不到、或点名的产物与目录不在：退 78。"""


class RunInputs:
    """一趟的输入：前缀、后缀、导出根（修订 6 那一趟的已不在，可以是 None）、第 11–13 步的 RESUME_TAG、各单元绿的那一次的 TAG、树的快照摘要。"""

    def __init__(self, label: str, prefix: str, suffix: str, dumps: str | None, rows_tag: str, tags: dict[str, str], tree_digest: str, comparator: str):
        self.label, self.prefix, self.suffix, self.dumps, self.rows_tag = label, prefix, suffix, dumps, rows_tag
        self.tags, self.tree_digest, self.comparator = tags, tree_digest, comparator


def not_reconstructible_lines() -> list[str]:
    """②③：修订 6 那一趟的树与导出造不回来（第 2 次重跑登记修订 6），不比，按步各写一行、树差表写一行。"""
    role = "" if broken("role-missing") else " role=contrast"
    lines = [f"name=contrast_s15{role} step={step} status={NOT_RECONSTRUCTIBLE} reason={NOT_RECONSTRUCTIBLE_REASON}" for step in S15_STEPS]
    lines.append(f"name=contrast_tree_file{role} status={NOT_RECONSTRUCTIBLE} reason={NOT_RECONSTRUCTIBLE_REASON}")
    return lines


def ledger_of(repo: str, run: RunInputs) -> dict[tuple[str, str], str] | None:
    """编排器的账 row-hosts.tsv（行 → 台）；导出根没填或账不在交 None（不当成全在本机）。"""
    path = os.path.join(repo, run.dumps, f"segment1-rows{run.rows_tag}-shards", "row-hosts.tsv") if run.dumps else ""
    if not path or not os.path.exists(path):
        return None
    ledger = {}
    for line in open(path, encoding="utf-8"):
        step, row, host = line.rstrip("\n").split("\t")[:3]
        ledger[(step, row)] = host
    return ledger


def host_of_row(ledger: dict[tuple[str, str], str] | None, step: str, row: str) -> str:
    if broken("row-host-ignored"):
        return "1"
    if ledger is None:
        return "1" if broken("absent-ledger-as-host-1") else LEDGER_ABSENT
    return ledger.get((step, row), "1")


def row_time_lines(repo: str, run: RunInputs) -> list[str]:
    role = "" if broken("role-missing") else " role=contrast"
    ledger = ledger_of(repo, run)
    lines, by_host = [], {}
    for stem in ROW_PRODUCT_STEMS:
        path = os.path.join(repo, f"{run.prefix}-{stem}{run.rows_tag}{run.suffix}.out")
        lines.append(f"name=contrast_row_product{role} run={run.label} stem={stem} present={str(os.path.exists(path)).lower()}")
        for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
            if not line.startswith("row "):
                continue
            values = dict(fields_of(line))
            step = STEP_OF_STEM[stem]
            host = host_of_row(ledger, step, values.get("row_number", "?"))
            device_ns = values.get("device_ns", "-")
            lines.append(f"name=contrast_row_time{role} run={run.label} step={step} row={values.get('row_number', '?')} host={host} device_ns={device_ns} "
                         f"status={values.get('status', '-')} device_exit={values.get('device_exit', '-')}")
            if device_ns.isdigit():
                count, total = by_host.get(host, (0, 0))
                by_host[host] = (count + 1, total + int(device_ns))
    for host, (count, total) in sorted(by_host.items()):
        lines.append(f"name=contrast_row_time_by_host{role} run={run.label} host={host} rows={count} device_seconds={total / 1e9:.3f}")
    return lines


def missing_h15_keys(h15: dict) -> list[str]:
    """H15 里 ①④ 要的那几项没填的；导出根（只给 ④ 的台）不算，修订 6 那一趟的导出根已不在。"""
    return [key for key in H15_KEYS_FOR_CELLS_AND_ROW_TIMES if h15.get(key) is None]


def inputs_from_config(repo: str, config_path: str, revision_8_comparator: str) -> tuple[RunInputs, RunInputs]:
    import segment_one_units_revision_8 as units  # noqa: E402  只为读修订 8 那一趟各单元绿的那一次的 TAG（序列日志），不起任何东西
    config = judge.load_config(config_path)
    h15 = judge.section(config, "H15")
    absent = missing_h15_keys(h15)
    if absent:
        raise RunConditionError(f"H15（修订 6 那一趟的清单）缺 ①④ 要的 {','.join(absent)}；先跑 revision_8_placeholders.py check --phase contrast")
    revision_6 = RunInputs("revision_6", h15["product_prefix"], h15["product_suffix"], h15.get("dumps_root"), h15["rows_tag"], {},
                           REVISION_6_TREE_DIGEST, h15["comparator_product"])
    settings = units.settings_from_config(repo, config_path)
    tags = {unit: record[1] for unit, record in units.green_records(settings).items()}
    revision_8 = RunInputs("revision_8", settings.prefix, settings.suffix, settings.dumps, "", tags, settings.snapshot_digest, revision_8_comparator)
    for path in (revision_6.comparator, revision_8.comparator):
        if not os.path.isfile(judge.resolve(repo, path)):
            raise RunConditionError(f"比对器产物不在：{path}")
    return revision_6, revision_8


def write_contrast(repo: str, revision_6: RunInputs, revision_8: RunInputs, output: str) -> int:
    lines = ResultLines(output)
    role = "" if broken("role-missing") else " role=contrast"
    lines.result(f"name=config{role} product=contrast_revision_6_against_revision_8 tree_revision_6={revision_6.tree_digest} tree_revision_8={revision_8.tree_digest} "
                 f"comparator_revision_6={field(revision_6.comparator)} comparator_revision_8={field(revision_8.comparator)} "
                 f"note=两趟各自判定之后的描述量：谁也不进谁的判据，不改任何一趟的判定、不触发任何条款")
    cells = contrast_cells(open(judge.resolve(repo, revision_6.comparator), encoding="utf-8").read().splitlines(),
                           open(judge.resolve(repo, revision_8.comparator), encoding="utf-8").read().splitlines())
    missing_parts = not_reconstructible_lines()
    rows = row_time_lines(repo, revision_6) + row_time_lines(repo, revision_8)
    for body in cells + missing_parts + rows:
        lines.result(body)
    unavailable = "0" if broken("not-reconstructible-as-zero") else NOT_RECONSTRUCTIBLE
    lines.result(f"name=contrast_summary{role} cells={len(cells)} differing_cells={sum(1 for body in cells if body.endswith(' same=false'))} "
                 f"s15_lines={unavailable} tree_files_differing={unavailable} row_time_lines={sum(1 for body in rows if body.startswith('name=contrast_row_time '))}")
    lines.finish()
    return 0


def main(argument_list: list[str]) -> int:
    refusal = harness.refuse_break_outside_selftest(BREAK_VARIABLE, SELFTEST_VARIABLE)
    if refusal:
        print(refusal, file=sys.stderr)
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 第一段：修订 6 那一趟对修订 8 那一趟的对照（role=contrast）")
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
        revision_6, revision_8 = inputs_from_config(repo, arguments.config, arguments.revision_8_comparator)
        return write_contrast(repo, revision_6, revision_8, arguments.output)
    except (RunConditionError, judge.RunConditionError) as error:
        print(f"✗ {error}（退 78）", file=sys.stderr)
        print('     → 怎么办：照上面那句补齐运行条件（配置、比对器产物）再用同一条命令起', file=sys.stderr)
        return EXIT_RUN_CONDITION


# ── 自证 ──
def comparator_text(rows: dict[str, str]) -> list[str]:
    return ["E7RESULT name=config segment=s1", f"E7RESULT name=q1_row row=15 stream=a batch=1 O2={rows['q1']} O3甲=0",
            f"E7RESULT name=q5_judged sampling_point=main lost=10 reference_filesystem_red={rows['q5']}",
            "E7RESULT name=unknown_kind x=1", "E7RESULT name=unknown_kind x=2", f"E7RESULT name=verdict q1={rows['verdict']}", "E7RESULT name=done emitted=7"]


def cell_pairing_by_identity(scratch: str) -> tuple[bool, str]:
    old = comparator_text({"q1": "3", "q5": "0", "verdict": "jia"}) + ["E7RESULT name=q1_row row=23 stream=a batch=1 O2=1 O3甲=0"]
    new = comparator_text({"q1": "0", "q5": "0", "verdict": "jia"})
    new = [new[0], "E7RESULT name=q1_row row=23 stream=a batch=1 O2=1 O3甲=0"] + new[1:]
    cells = contrast_cells(old, new)
    differing = sorted((dict(fields_of(cell))["kind"], dict(fields_of(cell))["field"]) for cell in cells if cell.endswith(" same=false"))
    return differing == [("q1_row", "O2")], f"格 {len(cells)} 个；不同的 {differing}（要只有第 15 行的 O2；第 23 行在新的一边换了位置）"


def fake_runs(scratch: str, revision_6_dumps: bool = True) -> tuple[str, RunInputs, RunInputs]:
    """两趟的假产物与假账；revision_6_dumps=False 时修订 6 那一趟没有导出根（H15 的 dumps_root 没填，同真跑的样子）。"""
    repo = os.path.join(scratch, "repo")
    runs = []
    for label, values, rows_tag in (("revision_6", {"q1": "3", "q5": "0", "verdict": "jia"}, "-resume2"), ("revision_8", {"q1": "0", "q5": "0", "verdict": "jia"}, "")):
        comparator = judge.write_text(os.path.join(repo, f"{label}-comparator.out"), "\n".join(comparator_text(values)) + "\n")
        prefix = f"results/{label}"
        judge.write_text(os.path.join(repo, f"{prefix}-rows-driver{rows_tag}-r.out"),
                         "row row_number=0 status=ok device_exit=0 device_ns=2000000000\nrow row_number=15 status=ok device_exit=0 device_ns=1000000000\n")
        judge.write_text(os.path.join(repo, f"dumps-{label}/segment1-rows{rows_tag}-shards/row-hosts.tsv"), "s12\t0\t2\tattempt-1\tfetched\n")
        dumps = f"dumps-{label}" if revision_6_dumps or label == "revision_8" else None
        runs.append(RunInputs(label, prefix, "-r", dumps, rows_tag, {}, label[-1] * 64, comparator))
    return repo, runs[0], runs[1]


def cell_row_times_without_ledger(scratch: str) -> tuple[bool, str]:
    repo, revision_6, _ = fake_runs(scratch, revision_6_dumps=False)
    lines = row_time_lines(repo, revision_6)
    hosts = {dict(fields_of(line))["row"]: dict(fields_of(line))["host"] for line in lines if line.startswith("name=contrast_row_time ")}
    by_host = [dict(fields_of(line))["host"] for line in lines if line.startswith("name=contrast_row_time_by_host")]
    good = hosts == {"0": LEDGER_ABSENT, "15": LEDGER_ABSENT} and by_host == [LEDGER_ABSENT]
    return good, f"导出根没填：各行的台 {hosts}；按台汇总的台 {by_host}（要都是 {LEDGER_ABSENT}，不当本机）"


def cell_parts_not_reconstructible(scratch: str) -> tuple[bool, str]:
    repo, revision_6, revision_8 = fake_runs(scratch, revision_6_dumps=False)
    output = os.path.join(scratch, "contrast.out")
    write_contrast(repo, revision_6, revision_8, output)
    lines = [line.rstrip("\n") for line in open(output, encoding="utf-8")]
    s15 = [line for line in lines if "name=contrast_s15 " in line]
    trees = [line for line in lines if "name=contrast_tree_file " in line]
    summary = dict(fields_of(next((line for line in lines if "name=contrast_summary " in line), "")))
    good = (len(s15) == len(S15_STEPS) and len(trees) == 1 and all(f" status={NOT_RECONSTRUCTIBLE} " in line for line in s15 + trees)
            and summary.get("s15_lines") == NOT_RECONSTRUCTIBLE and summary.get("tree_files_differing") == NOT_RECONSTRUCTIBLE)
    return good, (f"② {len(s15)} 行、③ {len(trees)} 行都写 {NOT_RECONSTRUCTIBLE}：{good}；收尾那一行 s15_lines={summary.get('s15_lines')} "
                  f"tree_files_differing={summary.get('tree_files_differing')}（要 {NOT_RECONSTRUCTIBLE}，不写数）")


def cell_row_times_by_host(scratch: str) -> tuple[bool, str]:
    repo, revision_6, _ = fake_runs(scratch)
    lines = row_time_lines(repo, revision_6)
    hosts = {dict(fields_of(line))["row"]: dict(fields_of(line))["host"] for line in lines if line.startswith("name=contrast_row_time ")}
    by_host = sorted(line for line in lines if line.startswith("name=contrast_row_time_by_host"))
    return hosts == {"0": "2", "15": "1"} and len(by_host) == 2, f"各行在哪台 {hosts}；按台汇总 {len(by_host)} 行"


def cell_every_line_carries_role(scratch: str) -> tuple[bool, str]:
    repo, revision_6, revision_8 = fake_runs(scratch)
    output = os.path.join(scratch, "contrast.out")
    write_contrast(repo, revision_6, revision_8, output)
    lines = [line.rstrip("\n") for line in open(output, encoding="utf-8")]
    without_role = [line for line in lines if " role=contrast" not in line and not line.startswith("E7RESULT name=done ")]
    verdicts = [line for line in lines if line.startswith("E7RESULT name=verdict ")]
    products = [line for line in lines if "name=contrast_row_product " in line]
    good = (not without_role and not verdicts and len(products) == 2 * len(ROW_PRODUCT_STEMS)
            and sum(" present=true" in line for line in products) == 2 and lines[-1].startswith("E7RESULT name=done "))
    return good, (f"{len(lines)} 行；不带 role 的 {len(without_role)}；name=verdict 行 {len(verdicts)}；行产物在不在 {len(products)} 行"
                  f"（两趟各 {len(ROW_PRODUCT_STEMS)} 份，在的 2 份）")


SELFTEST_CELLS = {"pairing-by-identity": cell_pairing_by_identity, "row-times-by-host": cell_row_times_by_host,
                  "row-times-without-ledger": cell_row_times_without_ledger, "parts-not-reconstructible": cell_parts_not_reconstructible,
                  "every-line-carries-role": cell_every_line_carries_role}
SELFTEST_SWITCHES = {"identity-ignored": "pairing-by-identity", "role-missing": "every-line-carries-role",
                     "not-reconstructible-as-zero": "parts-not-reconstructible", "row-host-ignored": "row-times-by-host",
                     "absent-ledger-as-host-1": "row-times-without-ledger"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
