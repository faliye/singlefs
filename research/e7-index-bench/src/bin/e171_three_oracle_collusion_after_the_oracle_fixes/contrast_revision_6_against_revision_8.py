#!/usr/bin/env python3
"""E171 第 1 次重跑第一段：修订 6 那一趟对修订 8 那一趟的对照（登记 research/prompts/e171-r1-prereg.md 修订 8 第 8.5 节）。
两趟各自的比对器（segment_one_comparator.py）判定都写完之后才跑；比对器不引它，它也不改任何一趟的判定、不触发任何条款：每一行带 role=contrast，
没有 name=verdict 行（收尾一行 name=contrast_summary 只数格子）。只读两趟的产物、导出与两棵树，写一份新产物（已存在就拒绝）。

逐格并排的几样：
  ① 两趟比对器产物里的每一行：按行种认身份（下面 IDENTITY_FIELDS；没列的行种按出现次序配），身份之外每个字段一行 name=contrast_cell，
     revision_6=… revision_8=…（一边没有写 absent）same=…；Q1–Q5 的各格（红态数、适用数、峰值、为正段数、期末值、判定……）都在这里；
  ② 第 1、6、7、10 步之后那几样比（修订 6 第 6.6 节 S15 ④⑤⑥ 的比法，用在两趟之间）：调修订 6 草稿里那份 s15_compare.py（按路径、开跑核 sha256），
     它的每一行改记成 name=contrast_s15 step=… method=… s15_line=<原行种> …，原行里的 triggered= 改名 differs_from_the_revision_6_prediction=（只描述）；
  ③ 两棵树的逐文件差表（同第 0 步判法的逐文件表：Cargo.*、.rs、.cargo/ 下全部、crates/mutations.tsv，不进 target）：name=contrast_tree_file；
  ④ 两趟第 11–13 步各行装置那一步的耗时（行产物的 device_ns），按台分开（编排器的账 row-hosts.tsv）：name=contrast_row_time，每台一行汇总 name=contrast_row_time_by_host。
用法：
  contrast_revision_6_against_revision_8.py --config C --revision-8-comparator P --output O [--repo R]
      修订 6 那一趟的清单取配置 H15（比对器产物、导出根、产物前缀与后缀、第 11–13 步的 RESUME_TAG）；修订 8 那一趟取 H3、H16 与单元编排的序列日志
      （各单元绿的那一次的 TAG）。
  contrast_revision_6_against_revision_8.py --selftest [--cells 格,…] [--rounds N]
退出码：0 写完；2 用法错；3 s15_compare.py 不是钉的那一份；78 配置读不到、或点名的产物与目录不在（H15、H16 没核过）。

弄坏开关 CONTRAST_R8_BREAK=<项>（只在 --selftest 起的子进程里认，CONTRAST_R8_SELFTEST=1）。项名、弄坏哪一处、打开后红在哪一格：
  identity-ignored        比对器的行一律按出现次序配，不按身份字段        pairing-by-identity
  role-missing            格子行不带 role=contrast                          every-line-carries-role
  s15-triggered-kept      s15 那几行的 triggered= 原样留着                  s15-relabelled
  row-host-ignored        行耗时一律记在本机                                row-times-by-host
"""
from __future__ import annotations

import argparse
import hashlib
import os
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import frozen_tree_lines_revision_8 as judge  # noqa: E402
import revision_8_placeholders as placeholders  # noqa: E402
import revision_8_selftest_harness as harness  # noqa: E402
from result_lines import ResultLines, field  # noqa: E402

BREAK_VARIABLE = "CONTRAST_R8_BREAK"
SELFTEST_VARIABLE = "CONTRAST_R8_SELFTEST"
EXIT_WRONG_PINNED_FILE = 3
EXIT_RUN_CONDITION = 78
S15_COMPARE = os.path.expanduser("~/.local/share/singlefs/work/e171-r1-segment1-rev6/s15_compare.py")
S15_COMPARE_SHA256 = "346fcc2e66fb9719d8946be19af275b3da20e92846ef863c52209cbc41b72671"
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
RENAMED_S15_KEY = ("triggered", "differs_from_the_revision_6_prediction")
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


class WrongPinnedFile(Exception):
    """s15_compare.py 不是钉的那一份。"""


def s15_compare_path() -> str:
    if os.environ.get(SELFTEST_VARIABLE) == "1" and os.environ.get("CONTRAST_R8_S15_COMPARE"):
        return os.environ["CONTRAST_R8_S15_COMPARE"]
    with open(S15_COMPARE, "rb") as handle:
        actual = hashlib.sha256(handle.read()).hexdigest()
    if actual != S15_COMPARE_SHA256:
        raise WrongPinnedFile(f"{S15_COMPARE} 的 sha256 是 {actual}，不是 {S15_COMPARE_SHA256}")
    return S15_COMPARE


def relabel_s15_line(line: str, step: str, method: str) -> str | None:
    """s15_compare.py 的一行改记成对照行；config 与 done 不收。"""
    pairs = fields_of(line)
    if not pairs or pairs[0][0] != "name" or pairs[0][1] in ("config", "done"):
        return None
    rest = []
    for name, value in pairs[1:]:
        if name == RENAMED_S15_KEY[0] and not broken("s15-triggered-kept"):
            name = RENAMED_S15_KEY[1]
        rest.append(f"{name}={value}")
    role = "" if broken("role-missing") else " role=contrast"
    return f"name=contrast_s15{role} step={step} method={method} s15_line={pairs[0][1]} " + " ".join(rest)


def run_s15(method: str, step: str, arguments: list[str], repo: str) -> list[str]:
    with tempfile.TemporaryDirectory(prefix="e171-r8-contrast-s15-") as work:
        output = os.path.join(work, "s15.out")
        completed = subprocess.run([sys.executable, "-B", s15_compare_path(), method, output] + arguments, cwd=repo, capture_output=True, text=True,
                                   env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        lines = open(output, encoding="utf-8").read().splitlines() if os.path.exists(output) else []
    relabelled = [text for text in (relabel_s15_line(line, step, method) for line in lines) if text]
    if not lines:
        role = "" if broken("role-missing") else " role=contrast"
        relabelled.append(f"name=contrast_s15{role} step={step} method={method} s15_line=none exit={completed.returncode} "
                          f"stderr={field(completed.stderr.strip().splitlines()[-1] if completed.stderr.strip() else '-')}")
    return relabelled


class RunInputs:
    """一趟的输入：前缀、后缀、导出根、第 11–13 步的 RESUME_TAG、各单元绿的那一次的 TAG、树。"""

    def __init__(self, label: str, prefix: str, suffix: str, dumps: str, rows_tag: str, tags: dict[str, str], tree: str, tree_digest: str, comparator: str):
        self.label, self.prefix, self.suffix, self.dumps, self.rows_tag = label, prefix, suffix, dumps, rows_tag
        self.tags, self.tree, self.tree_digest, self.comparator = tags, tree, tree_digest, comparator

    def product(self, stem: str, unit: str | None = None) -> str:
        return f"{self.prefix}-{stem}{self.tags.get(unit, '') if unit else ''}{self.suffix}.out"

    def dump(self, name: str, unit: str) -> str:
        return os.path.join(self.dumps, name + self.tags.get(unit, ""))


def s15_lines(revision_6: RunInputs, revision_8: RunInputs, repo: str) -> list[str]:
    """第 1、6、7、10 步之后那几样比：新的一边是修订 8 那一趟，旧的一边是修订 6 那一趟。"""
    third = "first_file_unlinked_then_inodes_across_one_leaf"
    reverse = "q5-reverse-at-the-return-point-of-call-2"
    main = "q5-main-before-the-first-barrier-of-call-3"
    lines = []
    lines += run_s15("five", "1", [revision_8.dump("s2-device-baseline-streams-1-2", "step-1"), revision_6.dump("s2-device-baseline-streams-1-2", "step-1")], repo)
    lines += run_s15("four", "6", [revision_8.product(reverse, "step-6"), revision_6.product(reverse, "step-6"),
                                   revision_8.dump("q5-reverse", "step-6"), revision_6.dump("q5-reverse", "step-6")], repo)
    lines += run_s15("six", "7", [os.path.join(revision_8.dump("baseline-run1", "step-7"), third), os.path.join(revision_6.dump("baseline-run1", "step-7"), third)], repo)
    lines += run_s15("five", "7", [revision_8.dump("baseline-run1", "step-7"), revision_6.dump("baseline-run1", "step-7")], repo)
    for run, directory in ((1, "q5-main"), (2, "q5-main-run-2")):
        unit = f"step-10-run-{run}"
        lines += run_s15("four", f"10-run-{run}", [revision_8.product(f"{main}-run-{run}", unit), revision_6.product(f"{main}-run-{run}", unit),
                                                   revision_8.dump(directory, unit), revision_6.dump(directory, unit)], repo)
    return lines


def tree_lines(revision_6: RunInputs, revision_8: RunInputs, repo: str) -> list[str]:
    role = "" if broken("role-missing") else " role=contrast"
    differing = judge.table_differences(judge.snapshot_table(judge.resolve(repo, revision_6.tree)), judge.snapshot_table(judge.resolve(repo, revision_8.tree)))
    return [f"name=contrast_tree_file{role} file={field(path)} revision_6={old} revision_8={new}" for path, old, new in differing]


def ledger_of(repo: str, run: RunInputs) -> dict[tuple[str, str], str]:
    path = os.path.join(repo, run.dumps, f"segment1-rows{run.rows_tag}-shards", "row-hosts.tsv")
    ledger = {}
    for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
        step, row, host = line.rstrip("\n").split("\t")[:3]
        ledger[(step, row)] = host
    return ledger


def row_time_lines(repo: str, run: RunInputs) -> list[str]:
    role = "" if broken("role-missing") else " role=contrast"
    ledger = ledger_of(repo, run)
    lines, by_host = [], {}
    for stem in ROW_PRODUCT_STEMS:
        path = os.path.join(repo, f"{run.prefix}-{stem}{run.rows_tag}{run.suffix}.out")
        for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
            if not line.startswith("row "):
                continue
            values = dict(fields_of(line))
            step = STEP_OF_STEM[stem]
            host = "1" if broken("row-host-ignored") else ledger.get((step, values.get("row_number", "?")), "1")
            device_ns = values.get("device_ns", "-")
            lines.append(f"name=contrast_row_time{role} run={run.label} step={step} row={values.get('row_number', '?')} host={host} device_ns={device_ns} "
                         f"status={values.get('status', '-')} device_exit={values.get('device_exit', '-')}")
            if device_ns.isdigit():
                count, total = by_host.get(host, (0, 0))
                by_host[host] = (count + 1, total + int(device_ns))
    for host, (count, total) in sorted(by_host.items()):
        lines.append(f"name=contrast_row_time_by_host{role} run={run.label} host={host} rows={count} device_seconds={total / 1e9:.3f}")
    return lines


def inputs_from_config(repo: str, config_path: str, revision_8_comparator: str) -> tuple[RunInputs, RunInputs]:
    import segment_one_units_revision_8 as units  # noqa: E402  只为读修订 8 那一趟各单元绿的那一次的 TAG（序列日志），不起任何东西
    config = judge.load_config(config_path)
    h15 = judge.section(config, "H15")
    if any(h15.get(key) is None for key in ("comparator_product", "dumps_root", "product_prefix", "product_suffix", "rows_tag")):
        raise RunConditionError("H15（修订 6 那一趟的清单）没填齐；先跑 revision_8_placeholders.py check --phase contrast")
    revision_6 = RunInputs("revision_6", h15["product_prefix"], h15["product_suffix"], h15["dumps_root"], h15["rows_tag"], {},
                           judge.REVISION_6_TREE, REVISION_6_TREE_DIGEST, h15["comparator_product"])
    settings = units.settings_from_config(repo, config_path)
    tags = {unit: record[1] for unit, record in units.green_records(settings).items()}
    revision_8 = RunInputs("revision_8", settings.prefix, settings.suffix, settings.dumps, "", tags, settings.tree, settings.snapshot_digest, revision_8_comparator)
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
    s15 = s15_lines(revision_6, revision_8, repo)
    trees = tree_lines(revision_6, revision_8, repo)
    rows = row_time_lines(repo, revision_6) + row_time_lines(repo, revision_8)
    for body in cells + s15 + trees + rows:
        lines.result(body)
    lines.result(f"name=contrast_summary{role} cells={len(cells)} differing_cells={sum(1 for body in cells if body.endswith(' same=false'))} "
                 f"s15_lines={len(s15)} tree_files_differing={len(trees)} row_time_lines={len(rows)}")
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
        return EXIT_RUN_CONDITION
    except WrongPinnedFile as error:
        print(f"✗ {error}", file=sys.stderr)
        return EXIT_WRONG_PINNED_FILE


# ── 自证 ──
FAKE_S15 = r'''import sys
method, output = sys.argv[1], sys.argv[2]
with open(output, "x", encoding="utf-8") as handle:
    handle.write(f"E7RESULT name=config segment=s1 product=s15_{method}\n")
    handle.write(f"E7RESULT name=s15_{method[0]} part=x inputs={len(sys.argv) - 3}\n")
    handle.write(f"E7RESULT name=verdict s15_{method}_problems=1 triggered=true\n")
    handle.write("E7RESULT name=done emitted=4\n")
'''


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


def cell_s15_relabelled(scratch: str) -> tuple[bool, str]:
    verdict = relabel_s15_line("E7RESULT name=verdict s15_five_problems=1 triggered=true", "1", "five")
    config = relabel_s15_line("E7RESULT name=config segment=s1 product=s15_five", "1", "five")
    good = config is None and verdict is not None and "differs_from_the_revision_6_prediction=true" in verdict and " triggered=" not in verdict \
        and "s15_line=verdict" in verdict and "role=contrast" in verdict
    return good, f"verdict 那一行改成 {verdict}；config 那一行不收 {config is None}"


def fake_runs(scratch: str) -> tuple[str, RunInputs, RunInputs]:
    repo = os.path.join(scratch, "repo")
    for label, name in (("revision_6", "tree-6"), ("revision_8", "tree-8")):
        judge.write_text(os.path.join(repo, name, "Cargo.toml"), "[workspace]\n")
        judge.write_text(os.path.join(repo, name, "crates/a/src/lib.rs"), f"// {label if name == 'tree-8' else 'same'}\n")
    runs = []
    for label, values, rows_tag in (("revision_6", {"q1": "3", "q5": "0", "verdict": "jia"}, "-resume2"), ("revision_8", {"q1": "0", "q5": "0", "verdict": "jia"}, "")):
        comparator = judge.write_text(os.path.join(repo, f"{label}-comparator.out"), "\n".join(comparator_text(values)) + "\n")
        prefix = f"results/{label}"
        judge.write_text(os.path.join(repo, f"{prefix}-rows-driver{rows_tag}-r.out"),
                         "row row_number=0 status=ok device_exit=0 device_ns=2000000000\nrow row_number=15 status=ok device_exit=0 device_ns=1000000000\n")
        judge.write_text(os.path.join(repo, f"dumps-{label}/segment1-rows{rows_tag}-shards/row-hosts.tsv"), "s12\t0\t2\tattempt-1\tfetched\n")
        runs.append(RunInputs(label, prefix, "-r", f"dumps-{label}", rows_tag, {}, "tree-6" if label == "revision_6" else "tree-8", label[-1] * 64, comparator))
    return repo, runs[0], runs[1]


def cell_row_times_by_host(scratch: str) -> tuple[bool, str]:
    repo, revision_6, _ = fake_runs(scratch)
    lines = row_time_lines(repo, revision_6)
    hosts = {dict(fields_of(line))["row"]: dict(fields_of(line))["host"] for line in lines if line.startswith("name=contrast_row_time ")}
    by_host = sorted(line for line in lines if line.startswith("name=contrast_row_time_by_host"))
    return hosts == {"0": "2", "15": "1"} and len(by_host) == 2, f"各行在哪台 {hosts}；按台汇总 {len(by_host)} 行"


def cell_every_line_carries_role(scratch: str) -> tuple[bool, str]:
    repo, revision_6, revision_8 = fake_runs(scratch)
    os.environ["CONTRAST_R8_S15_COMPARE"] = judge.write_text(os.path.join(scratch, "fake_s15.py"), FAKE_S15)
    output = os.path.join(scratch, "contrast.out")
    write_contrast(repo, revision_6, revision_8, output)
    lines = [line.rstrip("\n") for line in open(output, encoding="utf-8")]
    without_role = [line for line in lines if " role=contrast" not in line and not line.startswith("E7RESULT name=done ")]
    verdicts = [line for line in lines if line.startswith("E7RESULT name=verdict ")]
    s15 = [line for line in lines if "name=contrast_s15 " in line]
    trees = [line for line in lines if "name=contrast_tree_file " in line]
    good = not without_role and not verdicts and len(s15) == 12 and len(trees) == 1 and lines[-1].startswith("E7RESULT name=done ")
    return good, f"{len(lines)} 行；不带 role 的 {len(without_role)}；name=verdict 行 {len(verdicts)}；s15 {len(s15)} 行（6 次各 2 行）；树差 {len(trees)} 行"


SELFTEST_CELLS = {"pairing-by-identity": cell_pairing_by_identity, "s15-relabelled": cell_s15_relabelled, "row-times-by-host": cell_row_times_by_host,
                  "every-line-carries-role": cell_every_line_carries_role}
SELFTEST_SWITCHES = {"identity-ignored": "pairing-by-identity", "role-missing": "every-line-carries-role", "s15-triggered-kept": "s15-relabelled",
                     "row-host-ignored": "row-times-by-host"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
