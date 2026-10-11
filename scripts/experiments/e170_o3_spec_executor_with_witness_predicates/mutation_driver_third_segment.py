"""E170 第 2 次重跑第三段的变异驱动（不进冻结：登记 5.6 的 mutation_driver* 前缀；`research/prompts/e170-r2-prereg.md` 第九节、5.8 第三段那一行）。

三件事，结果都追加进同一份产物（产物由 run_segment_three.sh 先建好，本驱动只追加）：
  unit    v2 的变异（X22–X26、X28–X31）与 v1 的变异表 X1–X20 各在 v2 冻结文件的副本上跑单测层一遍（逐条：核原文在副本里恰好命中一次、
          替换、py_compile、跑 unittest；结局照 .claude/rules/mutation-sampling.md 记 抓到 / 无效 / 没红 / 超时）。
  batch   X22–X26、X28–X31 在 I1（`scripts/experiments/target/e170-dumps` 批 1、2）上各跑一趟执行器，与同一趟里不变异的 v2 逐态逐字段比（`time_ns`、
          `worker` 除外）；X24–X26、X30、X31 另在第 242、269、270 行的导出上各跑一趟（登记没点名，附带，产物里标明）。
  xc      XC3–XC7（比对器与参照谓词的变异）：在 v2 目录的副本上施加，重跑受影响的参照产物与两份比对器，与第二段入库的比对器产物逐行比。
v2 目录本身一个字节不动；每一条变异都在草稿目录的副本上做，收尾核 v2 目录与 v1 目录逐文件 sha256 与开跑前相同。

用法：python3 mutation_driver_third_segment.py --scratch <草稿目录> --output <已存在的产物> --workers <N> --results-directory <research/results>
"""

from __future__ import annotations

import argparse
import hashlib
import os
import py_compile
import re
import shutil
import subprocess
import sys
import time

sys.dont_write_bytecode = True

V2_DIRECTORY = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.abspath(os.path.join(V2_DIRECTORY, "../../.."))
V1_DIRECTORY = os.path.join(ROOT, "scripts/experiments/e170_second_run_o3_spec_executor")
V1_TABLE = os.path.join(ROOT, "scripts/mutations/e170_second_run_o3_spec_executor.tsv")
V2_TABLE = os.path.join(ROOT, "scripts/mutations/e170_o3_spec_executor_with_witness_predicates.tsv")
PREFIXES_OUTSIDE_THE_FREEZE = ("comparator", "mutation_driver", "witness_reference", "run_")
UNIT_TIMEOUT_SECONDS = 180
EXECUTOR_TIMEOUT_SECONDS = 600
FIELD_PATTERN = re.compile(r"(\w+)=(\S*)")
NOT_COMPARED = {"time_ns", "worker"}
V2_BATCH_MUTATIONS = ("X22", "X23", "X24", "X25", "X26", "X28", "X29", "X30", "X31")
XC_MUTATIONS = ("XC3", "XC4", "XC5", "XC6", "XC7")


class Output:
    def __init__(self, path: str):
        if not os.path.exists(path):
            raise SystemExit(f"产物 {path} 应由 run_segment_three.sh 先建好，不存在")
        self.handle = open(path, "a", encoding="utf-8", buffering=1)

    def emit(self, line: str) -> None:
        self.handle.write(line + "\n")
        print(line, flush=True)


def fields_of(line: str) -> dict[str, str]:
    return dict(FIELD_PATTERN.findall(line.rstrip("\n").removeprefix("E7RESULT ")))


def frozen_names(directory: str) -> list[str]:
    """v2 冻结的文件 = 目录里除 comparator*、mutation_driver*、witness_reference*、run_* 与 .pyc 之外的全部普通文件（登记 5.6）。"""
    return sorted(name for name in os.listdir(directory) if os.path.isfile(os.path.join(directory, name))
                  and not name.startswith(PREFIXES_OUTSIDE_THE_FREEZE) and not name.endswith(".pyc"))


def names_with_comparators(directory: str) -> list[str]:
    """XC 用的副本：冻结文件加上 comparator*、witness_reference*（mutation_driver*、run_* 不拷）。"""
    return sorted(name for name in os.listdir(directory) if os.path.isfile(os.path.join(directory, name))
                  and not name.startswith(("mutation_driver", "run_")) and not name.endswith(".pyc"))


def digests_of(directory: str, names: list[str]) -> dict[str, str]:
    result = {}
    for name in names:
        with open(os.path.join(directory, name), "rb") as handle:
            result[name] = hashlib.sha256(handle.read()).hexdigest()
    return result


def fresh_copy(source_directory: str, names: list[str], scratch: str, label: str) -> str:
    copy = os.path.join(scratch, label)
    if os.path.exists(copy):
        shutil.rmtree(copy)
    os.makedirs(copy)
    for name in names:
        shutil.copy2(os.path.join(source_directory, name), os.path.join(copy, name))
    return copy


def read_table(path: str, wanted: tuple[str, ...] | None = None, number_range: tuple[int, int] | None = None) -> list[tuple[str, str, str]]:
    """每行：名字、原文、替换文（表里的 \\n 还原成换行）。wanted 按名字起头的编号（X22 起头的行取 X22_…）选；number_range 选 X<n>_ 的 n 在闭区间内的行。"""
    rows = []
    with open(path, encoding="utf-8") as handle:
        for line in handle.read().split("\n"):
            if not line.strip() or line.startswith("#"):
                continue
            name, anchor, replacement = line.split("\t")
            identifier = name.split("_", 1)[0]
            if wanted is not None and identifier not in wanted:
                continue
            if number_range is not None:
                match = re.fullmatch(r"X(\d+)", identifier)
                if match is None or not number_range[0] <= int(match.group(1)) <= number_range[1]:
                    continue
            rows.append((name, anchor.replace("\\n", "\n"), replacement.replace("\\n", "\n")))
    return rows


def anchor_hits(directory: str, names: list[str], anchor: str) -> list[tuple[str, int]]:
    hits = []
    for name in names:
        if not name.endswith((".py", ".tsv")):
            continue
        with open(os.path.join(directory, name), encoding="utf-8") as handle:
            count = handle.read().count(anchor)
        if count:
            hits.append((name, count))
    return hits


def apply_mutation(copy: str, names: list[str], anchor: str, replacement: str) -> str:
    """原文在副本里合起来恰好命中一次才替换；返回被改的文件名。"""
    hits = anchor_hits(copy, names, anchor)
    assert sum(count for _name, count in hits) == 1, f"原文命中 {hits}，不是恰好一次"
    path = os.path.join(copy, hits[0][0])
    with open(path, encoding="utf-8") as handle:
        text = handle.read()
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text.replace(anchor, replacement, 1))
    return hits[0][0]


# ---- 单测层 ----

def run_unit_tests(copy: str, scratch: str) -> tuple[str, int | None, str]:
    test_scratch = os.path.join(scratch, "unit-test-scratch")
    shutil.rmtree(test_scratch, ignore_errors=True)
    os.makedirs(test_scratch)
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", E170_TEST_SCRATCH=test_scratch)
    try:
        completed = subprocess.run([sys.executable, "-m", "unittest", "test_executor_on_synthetic_pool"], cwd=copy, env=environment,
                                   capture_output=True, text=True, timeout=UNIT_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        return "timeout", None, ""
    return "finished", completed.returncode, completed.stderr + completed.stdout


def red_test_names(output: str) -> list[str]:
    return sorted(set(re.findall(r"^(?:FAIL|ERROR): (\S+)", output, flags=re.MULTILINE)))


def unit_level(out: Output, label: str, rows: list[tuple[str, str, str]], scratch: str) -> dict[str, dict]:
    """逐条在 v2 冻结文件的副本上施加、跑单测。返回 名字 → {outcome: caught/invalid/not_red/timeout, red, file, seconds, line}。"""
    names = frozen_names(V2_DIRECTORY)
    before = digests_of(V2_DIRECTORY, names)
    baseline = fresh_copy(V2_DIRECTORY, names, scratch, "unit-baseline")
    state, code, text = run_unit_tests(baseline, scratch)
    ran = re.findall(r"^Ran (\d+) tests?", text, flags=re.MULTILINE)
    out.emit(f"name=unit_baseline label={label} state={state} exit={code} ran={ran[-1] if ran else '-'} frozen_files={len(names)}")
    if state != "finished" or code != 0:
        raise SystemExit("v2 副本上的单测基线就是红的，不跑变异")
    results: dict[str, dict] = {}
    for name, anchor, replacement in rows:
        started = time.perf_counter()
        copy = fresh_copy(V2_DIRECTORY, names, scratch, "unit-mutated")
        hits = anchor_hits(copy, names, anchor)
        if sum(count for _file, count in hits) != 1:
            results[name] = {"outcome": "anchor_problem", "red": [], "file": "-", "seconds": 0.0,
                             "line": f"name=anchor_problem mutation={name} hits={hits}（原文在 v2 冻结文件里不是恰好命中一次，没施加）"}
            continue
        file_name = apply_mutation(copy, names, anchor, replacement)
        path = os.path.join(copy, file_name)
        if file_name.endswith(".py"):
            try:
                py_compile.compile(path, cfile=os.path.join(scratch, "compile-check.pyc"), doraise=True)
            except py_compile.PyCompileError as error:
                results[name] = {"outcome": "invalid", "red": [], "file": file_name, "seconds": time.perf_counter() - started,
                                 "line": f"{file_name} 替换之后编不过（{str(error).splitlines()[-1][:160]}）"}
                continue
        state, code, text = run_unit_tests(copy, scratch)
        seconds = time.perf_counter() - started
        if state == "timeout":
            results[name] = {"outcome": "timeout", "red": [], "file": file_name, "seconds": seconds,
                             "line": f"{file_name} 单测超时（{UNIT_TIMEOUT_SECONDS} 秒）"}
        elif code == 0:
            ran = re.findall(r"^Ran (\d+) tests?", text, flags=re.MULTILINE)
            results[name] = {"outcome": "not_red", "red": [], "file": file_name, "seconds": seconds,
                             "line": f"{file_name} 单测 {ran[-1] if ran else '?'} 条全过"}
        else:
            red = red_test_names(text)
            load_time = "" if red else "，载入时红：" + next((line.strip()[:200] for line in reversed(text.splitlines()) if "Error" in line), "输出里没有带 Error 的行")
            results[name] = {"outcome": "caught", "red": red, "file": file_name, "seconds": seconds,
                             "line": f"{file_name}，红的单测 {red}{load_time}"}
        out.emit(f"name=unit_result label={label} mutation={name} outcome={results[name]['outcome']} seconds={seconds:.1f} detail={results[name]['line']}")
    shutil.rmtree(os.path.join(scratch, "unit-mutated"), ignore_errors=True)
    after = digests_of(V2_DIRECTORY, names)
    out.emit(f"name=unit_restore label={label} v2_directory_unchanged={str(after == before).lower()} files={len(names)}")
    return results


SYMBOL = {"caught": "✅", "invalid": "⏭", "not_red": "❌", "timeout": "⏱", "anchor_problem": "❌"}


# ---- 批层：执行器在导出上判，与不变异的 v2 逐态逐字段比 ----

def run_executor(copy: str, dumps: str, batches: str, workers: int, output_path: str) -> tuple[int | None, str]:
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    try:
        completed = subprocess.run([sys.executable, "executor_main.py", "--dumps", dumps, "--batches", batches, "--workers", str(workers),
                                    "--output", output_path], cwd=copy, env=environment, capture_output=True, text=True,
                                   timeout=EXECUTOR_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        return None, "timeout"
    return completed.returncode, (completed.stderr + completed.stdout)[-300:]


def state_table(path: str, batches: set[str]) -> dict[tuple[str, str], dict[str, str]]:
    table = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            if line.startswith("name=state "):
                fields = fields_of(line)
                if fields.get("batch") in batches:
                    table[(fields["stream"], fields["state"])] = fields
    return table


def anchor_table(path: str) -> dict[tuple[str, str, str], str]:
    table = {}
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            if line.startswith(("name=anchor_assertion ", "name=witness_anchor_assertion ")):
                fields = fields_of(line)
                table[(fields["name"], fields["id"], fields.get("stream", "-"))] = fields.get("holds", "?")
    return table


def changed_states(base: dict, mutated: dict) -> list[tuple[tuple[str, str], dict[str, tuple[str, str]]]]:
    changed = []
    for key in sorted(base, key=lambda item: (item[0], int(item[1]))):
        if key not in mutated:
            changed.append((key, {"__missing__": ("present", "absent")}))
            continue
        differing = {field: (value, mutated[key].get(field, "<none>")) for field, value in base[key].items()
                     if field not in NOT_COMPARED and mutated[key].get(field, "<none>") != value}
        if differing:
            changed.append((key, differing))
    return changed


DUMPS = {
    "I1": ("scripts/experiments/target/e170-dumps", "s2-v2-i1"),
    "row-242": ("scripts/experiments/target/e171-dumps/rows-s4-f4/row-242/dump", "s2-v2-row-242"),
    "row-269": ("scripts/experiments/target/e171-dumps/rows-s4-f4/row-269/dump", "s2-v2-row-269"),
    "row-270": ("scripts/experiments/target/e171-dumps/rows-s4-f4/row-270/dump", "s2-v2-row-270"),
}


def batch_level(out: Output, mutations: list[tuple[str, str, str]], jobs: dict[str, list[str]], scratch: str, workers: int,
                results_directory: str) -> dict[tuple[str, str], dict]:
    """jobs：变异编号 → 要跑的导出标签。返回 (编号, 导出标签) → {changed, in_to_not_in, ..., sample_point}。"""
    names = frozen_names(V2_DIRECTORY)
    baselines: dict[str, tuple[str, dict, dict]] = {}
    needed = sorted({label for labels in jobs.values() for label in labels})
    baseline_copy = fresh_copy(V2_DIRECTORY, names, scratch, "batch-baseline")
    for label in needed:
        dumps = os.path.join(ROOT, DUMPS[label][0])
        path = os.path.join(scratch, f"baseline-{label}.out")
        code, tail = run_executor(baseline_copy, dumps, "1,2", workers, path)
        stored = os.path.join(results_directory, f"e170-r2-o3-spec-executor-2026-10-04-{DUMPS[label][1]}.out")
        base = state_table(path, {"1", "2"})
        stored_states = state_table(stored, {"1", "2"})
        differing = changed_states(stored_states, base)
        out.emit(f"name=batch_baseline dumps={label} executor_exit={code} states={len(base)} stored_states={len(stored_states)} "
                 f"differing_from_stored={len(differing) + len(set(base) - set(stored_states))} stored={os.path.basename(stored)}")
        if code != 0 or differing or len(base) != len(stored_states):
            raise SystemExit(f"{label} 的不变异那一趟与入库产物不同或退出码非 0，不往下跑")
        baselines[label] = (path, base, anchor_table(path))
    results: dict[tuple[str, str], dict] = {}
    for name, anchor, replacement in mutations:
        identifier = name.split("_", 1)[0]
        for label in jobs.get(identifier, []):
            copy = fresh_copy(V2_DIRECTORY, names, scratch, "batch-mutated")
            file_name = apply_mutation(copy, names, anchor, replacement)
            path = os.path.join(scratch, f"mutated-{identifier}-{label}.out")
            started = time.perf_counter()
            code, tail = run_executor(copy, os.path.join(ROOT, DUMPS[label][0]), "1,2", workers, path)
            seconds = time.perf_counter() - started
            if code != 0:
                results[(identifier, label)] = {"exit": code, "changed": None, "sample_point": "executor_failed"}
                out.emit(f"name=batch_run mutation={identifier} dumps={label} file={file_name} executor_exit={code} tail={tail!r}")
                continue
            mutated = state_table(path, {"1", "2"})
            _base_path, base, base_anchors = baselines[label]
            changed = changed_states(base, mutated)
            anchors_now = anchor_table(path)
            newly_false = sorted(f"{key[1]}@{key[2]}" for key, holds in anchors_now.items() if holds == "false" and base_anchors.get(key) != "false")
            results[(identifier, label)] = summarise_batch(identifier, label, base, mutated, changed, newly_false, code)
            summary = results[(identifier, label)]
            out.emit(f"name=batch_run mutation={identifier} dumps={label} file={file_name} executor_exit={code} states={len(mutated)} "
                     f"changed_states={len(changed)} strict_in_to_not_in={summary['in_to_not_in']} strict_not_in_to_in={summary['not_in_to_in']} "
                     f"witness_true_false_to_true={summary['w_flip']} tail_check_true_to_false={summary['t_flip']} "
                     f"anchors_newly_false={','.join(newly_false) or '-'} sample_point_changed={summary['sample_point']} seconds={seconds:.1f}")
            for key, differing in changed:
                out.emit(f"name=batch_changed_state mutation={identifier} dumps={label} stream={key[0]} state={key[1]} "
                         f"landed={base[key].get('landed')} " + " ".join(f"{field}:{before}->{after}" for field, (before, after) in sorted(differing.items())))
    shutil.rmtree(os.path.join(scratch, "batch-mutated"), ignore_errors=True)
    return results


def summarise_batch(identifier: str, label: str, base: dict, mutated: dict, changed: list, newly_false: list[str], code: int | None) -> dict:
    changed_dict = dict(changed)
    in_to_not = {key for key, differing in changed if differing.get("strict") == ("in", "not_in")}
    not_to_in = {key for key, differing in changed if differing.get("strict") == ("not_in", "in")}
    w_flip = {key for key, differing in changed if differing.get("witness_true") == ("false", "true")}
    t_flip = {key for key, differing in changed if differing.get("tail_check") == ("true", "false")}
    if identifier == "X22":  # 登记第九节：B1（I1 流一 F 态）W 由假变真、终判定在 → 不在
        sample = "B1@new_pool_file_creation" in newly_false and any(key[0] == "new_pool_file_creation" for key in w_flip & in_to_not)
    elif identifier == "X23":  # 落点是版本 0 的态（B3 的子集）终判定在 → 不在
        sample = any(base[key].get("landed", "").startswith("(0,0,") for key in in_to_not)
    elif identifier in ("X28", "X29"):  # B2 那一类：T 由真变假、终判定在 → 不在
        sample = bool(t_flip & in_to_not)
    else:  # 登记写明敏感点只在合成池单测里：真实流上输出不该变
        sample = None
    if sample is None:
        sample_text = "unit_only_output_unchanged" if not changed_dict and not newly_false else "unit_only_but_output_changed"
    else:
        sample_text = str(sample).lower()
    return {"exit": code, "changed": len(changed), "in_to_not_in": len(in_to_not), "not_to_in_count": len(not_to_in), "not_in_to_in": len(not_to_in),
            "w_flip": len(w_flip), "t_flip": len(t_flip), "newly_false": newly_false, "sample_point": sample_text}


# ---- XC：比对器与参照谓词的变异 ----

RESULTS_STEM = "e170-r2-o3-spec-executor-2026-10-04-s2"
REFERENCE_SPECS = {  # 参照产物后缀 → (mode, 标签, 导出目录, v2 产物后缀, 已知坏镜像目录)
    "i1": ("states", "I1", "scripts/experiments/target/e170-dumps", "v2-i1", None),
    "i2": ("states", "I2", "scripts/experiments/target/e170-second-run-dumps", "v2-i2", None),
    "row-242": ("states", "row-242", "scripts/experiments/target/e171-dumps/rows-s4-f4/row-242/dump", "v2-row-242", None),
    "row-269": ("states", "row-269", "scripts/experiments/target/e171-dumps/rows-s4-f4/row-269/dump", "v2-row-269", None),
    "row-270": ("states", "row-270", "scripts/experiments/target/e171-dumps/rows-s4-f4/row-270/dump", "v2-row-270", None),
    "e171-s4-baseline": ("states", "e171-s4-baseline", "scripts/experiments/target/e171-dumps/baseline-run1", "v2-e171-s4-baseline", None),
    "i3-images": ("images", "I3", "scripts/experiments/target/e170-second-run-dumps", "v2-i3-images", None),
    "i4-images": ("images", "I4", "scripts/experiments/target/e171-dumps/e170-bad-states-copy-s2", "v2-i4-images", "scripts/experiments/target/e171-dumps/known-bad-s2"),
}


def stored_path(results_directory: str, suffix: str) -> str:
    return os.path.join(results_directory, f"{RESULTS_STEM}-{suffix}.out")


def run_checked(copy: str, command: list[str]) -> int:
    completed = subprocess.run(command, cwd=copy, env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"), capture_output=True, text=True,
                               timeout=EXECUTOR_TIMEOUT_SECONDS)
    if completed.returncode != 0:
        print((completed.stderr + completed.stdout)[-400:], file=sys.stderr)
    return completed.returncode


def regenerate_references(copy: str, results_directory: str, scratch: str, tag: str) -> dict[str, str]:
    produced = {}
    for suffix, (mode, label, dumps, v2_suffix, known) in REFERENCE_SPECS.items():
        path = os.path.join(scratch, f"{tag}-reference-{suffix}.out")
        command = [sys.executable, "witness_reference.py", mode, "--dumps", os.path.join(ROOT, dumps)]
        if known:
            command += ["--known-bad-dir", os.path.join(ROOT, known)]
        command += ["--v2-output", stored_path(results_directory, v2_suffix), "--input-label", label, "--output", path]
        assert run_checked(copy, command) == 0, f"参照谓词 {suffix} 退出码非 0"
        produced[suffix] = path
    return produced


def run_q1_comparator(copy: str, results_directory: str, scratch: str, tag: str) -> str:
    P = lambda suffix: stored_path(results_directory, suffix)  # noqa: E731
    path = os.path.join(scratch, f"{tag}-comparator-q1.out")
    positive_scratch = os.path.join(scratch, f"{tag}-q1-positive-control")
    os.makedirs(positive_scratch)
    target = os.path.join(ROOT, "scripts/experiments/target")
    command = [sys.executable, "comparator_two_versions.py",
               "--pair", f"I1:states:{P('v1-i1')}:{P('v2-i1')}", "--pair", f"I2:states:{P('v1-i2')}:{P('v2-i2')}",
               "--pair", f"I3:images:{P('v1-i3-images')}:{P('v2-i3-images')}", "--pair", f"I4:images:{P('v1-i4-images')}:{P('v2-i4-images')}",
               "--batches", "1,2", "--names", f"I3:{target}/e170-second-run-dumps",
               "--names", f"I4:{target}/e171-dumps/e170-bad-states-copy-s2:{target}/e171-dumps/known-bad-s2",
               "--unit-tests", P("i5-v1-tests-on-v2"), "--anchors", P("v2-i1"), "--anchors", P("v2-i2"),
               "--positive-control-input", "I2", "--scratch", positive_scratch, "--output", path]
    assert run_checked(copy, command) == 0, "Q1 比对器退出码非 0"
    return path


def run_q2_q3_comparator(copy: str, results_directory: str, scratch: str, tag: str, references: dict[str, str]) -> str:
    P = lambda suffix: stored_path(results_directory, suffix)  # noqa: E731
    R = lambda suffix: references.get(suffix, P(f"reference-{suffix}"))  # noqa: E731
    path = os.path.join(scratch, f"{tag}-comparator-q2-q3.out")
    positive_scratch = os.path.join(scratch, f"{tag}-q2-q3-positive-control")
    os.makedirs(positive_scratch)
    command = [sys.executable, "comparator_witness_classes.py"]
    for row in ("242", "269", "270"):
        command += ["--row", f"{row}:{P(f'v1-row-{row}')}:{P(f'v2-row-{row}')}:{R(f'row-{row}')}"]
    command += ["--mutated", f"242:X21:{P('x21-row-242')}", "--mutated", f"269:X21:{P('x21-row-269')}", "--mutated", f"270:X27:{P('x27-row-270')}"]
    command += ["--legal", f"I1:previous_legal_history:states:1,2:{P('v2-i1')}:{R('i1')}",
                "--legal", f"I2:previous_legal_history:states:1,2:{P('v2-i2')}:{R('i2')}",
                "--legal", f"I3-base:previous_legal_history:images:-:{P('v2-i3-images')}:{R('i3-images')}",
                "--legal", f"I4-base:previous_legal_history:images:-:{P('v2-i4-images')}:{R('i4-images')}",
                "--legal", f"I1-batch-3:batch_three:states:3:{P('v2-i1')}:{R('i1')}",
                "--legal", f"I2-batch-3:batch_three:states:3:{P('v2-i2')}:{R('i2')}",
                "--legal", f"e171-s4-baseline:e171_s4_baseline:states:1,2,3:{P('v2-e171-s4-baseline')}:{R('e171-s4-baseline')}",
                "--positive-control-legal-input", "I2", "--scratch", positive_scratch, "--output", path]
    assert run_checked(copy, command) == 0, "Q2、Q3 比对器退出码非 0"
    return path


def normalised_lines(path: str) -> list[str]:
    with open(path, encoding="utf-8") as handle:
        return [re.sub(r"\bcopy=\S+", "copy=<scratch>", line.rstrip("\n")) for line in handle]


def lines_named(path: str, name: str) -> list[dict[str, str]]:
    with open(path, encoding="utf-8") as handle:
        return [fields_of(line) for line in handle if line.startswith(f"name={name} ")]


BULK_NAMES = {"q3c_disagreement", "q2c_disagreement", "q2a_state", "trajectory", "legal_flip"}


def emit_diff(out: Output, mutation: str, file_label: str, before_path: str, after_path: str) -> int:
    before, after = normalised_lines(before_path), normalised_lines(after_path)
    if len(before) == len(after):
        pairs = [(left, right) for left, right in zip(before, after) if left != right]
    else:
        pairs = [(left, "<行数不同，见比对行数>") for left in before if left not in after] + [("<新增>", right) for right in after if right not in before]
    out.emit(f"name=xc_diff mutation={mutation} file={file_label} lines_before={len(before)} lines_after={len(after)} changed_lines={len(pairs)}")
    bulk = {}
    for left, right in pairs:
        name = (left if left.startswith("name=") else right).split(" ", 1)[0].removeprefix("name=")
        if name in BULK_NAMES:
            bulk[name] = bulk.get(name, 0) + 1
            continue
        out.emit(f"name=xc_changed_line mutation={mutation} file={file_label} before=[{left}] after=[{right}]")
    for name, count in sorted(bulk.items()):
        out.emit(f"name=xc_changed_bulk mutation={mutation} file={file_label} line_name={name} changed_lines={count}")
    return len(pairs)


def p8_reference_true(references: dict[str, str] | None, results_directory: str) -> tuple[int, int]:
    """I3、I4 的 P8 格（source=e170、行号 5 与 11，两流）里 W_ref 为真的个数，与格数。"""
    true_count = total = 0
    for suffix in ("i3-images", "i4-images"):
        path = references[suffix] if references else stored_path(results_directory, f"reference-{suffix}")
        for fields in lines_named(path, "witness_reference"):
            if fields.get("source") == "e170" and fields.get("state") in ("5", "11"):
                total += 1
                true_count += fields.get("w_ref") == "true"
    return true_count, total


def xc_level(out: Output, rows: list[tuple[str, str, str]], scratch: str, results_directory: str) -> dict[str, dict]:
    names = names_with_comparators(V2_DIRECTORY)
    pristine = fresh_copy(V2_DIRECTORY, names, scratch, "xc-pristine")
    stored_q1, stored_q23 = stored_path(results_directory, "comparator-q1"), stored_path(results_directory, "comparator-q2-q3")
    q1_path = run_q1_comparator(pristine, results_directory, scratch, "xc-pristine")
    q23_path = run_q2_q3_comparator(pristine, results_directory, scratch, "xc-pristine", {})
    for label, stored, fresh in (("comparator-q1", stored_q1, q1_path), ("comparator-q2-q3", stored_q23, q23_path)):
        out.emit(f"name=xc_baseline file={label} identical_to_stored_modulo_copy_path={str(normalised_lines(stored) == normalised_lines(fresh)).lower()} "
                 f"lines={len(normalised_lines(fresh))}")
    p8_before = p8_reference_true(None, results_directory)
    results: dict[str, dict] = {}
    for name, anchor, replacement in rows:
        identifier = name.split("_", 1)[0]
        copy = fresh_copy(V2_DIRECTORY, names, scratch, f"xc-{identifier}")
        file_name = apply_mutation(copy, names, anchor, replacement)
        path = os.path.join(copy, file_name)
        try:
            py_compile.compile(path, cfile=os.path.join(scratch, "compile-check.pyc"), doraise=True)
        except py_compile.PyCompileError as error:
            results[identifier] = {"outcome": "invalid", "detail": f"{file_name} 替换之后编不过（{str(error).splitlines()[-1][:160]}）", "name": name}
            continue
        references = regenerate_references(copy, results_directory, scratch, identifier) if identifier in ("XC6", "XC7") else {}
        if identifier == "XC3":
            mutated_q1 = run_q1_comparator(copy, results_directory, scratch, identifier)
            emit_diff(out, identifier, "comparator-q1", stored_q1, mutated_q1)
            control = lines_named(mutated_q1, "q1_positive_control")[0]
            caught, detail = control["exactly_one"] == "false", f"Q1-pc added={control['added']} exactly_one={control['exactly_one']}（不变异时 exactly_one=true）"
        else:
            mutated_q23 = run_q2_q3_comparator(copy, results_directory, scratch, identifier, references)
            emit_diff(out, identifier, "comparator-q2-q3", stored_q23, mutated_q23)
            if identifier == "XC4":
                runs = [fields for fields in lines_named(mutated_q23, "mutation_run") if fields["mutation"] == "X21"]
                caught = len(runs) == 2 and all(fields["at_least_one"] == "false" for fields in runs)
                detail = "X21 那两趟 judged_in_inside_the_class=" + ",".join(f"{fields['row']}:{fields['judged_in_inside_the_class']}" for fields in runs) + "（不变异时 15、63）"
            elif identifier == "XC5":
                control = lines_named(mutated_q23, "q3_pc2")[0]
                caught, detail = control["exactly_one"] == "false", f"Q3-pc2 added={control['added']} exactly_one={control['exactly_one']}（不变异时 exactly_one=true）"
            elif identifier == "XC6":
                counts = {fields["row"]: fields["K"] for fields in lines_named(mutated_q23, "q2_counts")}
                p8_after = p8_reference_true(references, results_directory)
                caught = counts.get("242") == "0" and counts.get("269") == "0"
                detail = (f"Q2-b：第 242 行 K={counts.get('242')}、第 269 行 K={counts.get('269')}（不变异时 15、63）；I3+I4 的 P8 格 W_ref 为真 "
                          f"{p8_after[0]}/{p8_after[1]}（不变异时 {p8_before[0]}/{p8_before[1]}）")
            else:  # XC7
                total = sum(int(fields["q3c_tail_check_disagreements"]) for name_ in ("legal_counts", "q3_row_counts") for fields in lines_named(mutated_q23, name_))
                caught, detail = total >= 1, f"Q3-c 分歧态数合计 {total}（不变异时 0）"
        results[identifier] = {"outcome": "caught" if caught else "not_red", "detail": detail, "name": name}
        out.emit(f"name=xc_result mutation={identifier} file={file_name} outcome={results[identifier]['outcome']} detail={detail}")
        shutil.rmtree(copy, ignore_errors=True)
    shutil.rmtree(pristine, ignore_errors=True)
    return results


# ---- 主流程 ----

SYMBOL["anchor_problem"] = "⛔"  # 原文没命中一次不是「没红」，不进三个数


def all_digests(directory: str) -> dict[str, str]:
    return digests_of(directory, sorted(name for name in os.listdir(directory) if os.path.isfile(os.path.join(directory, name))))


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E170 第 2 次重跑第三段：变异")
    parser.add_argument("--scratch", required=True)
    parser.add_argument("--output", required=True)
    parser.add_argument("--workers", type=int, required=True)
    parser.add_argument("--results-directory", required=True)
    arguments = parser.parse_args(argument_list)
    out = Output(arguments.output)
    scratch = os.path.abspath(arguments.scratch)
    os.makedirs(scratch, exist_ok=True)
    started = time.perf_counter()
    v1_before, v2_before = all_digests(V1_DIRECTORY), all_digests(V2_DIRECTORY)
    v1_rows = read_table(V1_TABLE, number_range=(1, 20))
    v2_rows = read_table(V2_TABLE, wanted=V2_BATCH_MUTATIONS)
    xc_rows = read_table(V2_TABLE, wanted=XC_MUTATIONS)
    out.emit(f"name=third_segment_config python={sys.version.split()[0]} workers={arguments.workers} v1_table_rows={len(v1_rows)} "
             f"v2_table_rows={len(v2_rows)} xc_table_rows={len(xc_rows)} v1_directory_files={len(v1_before)} v2_directory_files={len(v2_before)} "
             f"unit_timeout_seconds={UNIT_TIMEOUT_SECONDS} executor_timeout_seconds={EXECUTOR_TIMEOUT_SECONDS}")
    assert (len(v1_rows), len(v2_rows), len(xc_rows)) == (20, 9, 5), "三张表的行数应是 20、9、5"
    # 一、v1 的 X1–X20 在 v2 冻结文件的副本上跑单测层
    results_v1 = unit_level(out, "v1-X1-X20-on-v2", v1_rows, scratch)
    counts_v1 = {"✅": 0, "⏭": 0, "❌": 0, "⏱": 0, "⛔": 0}
    for name, _anchor, _replacement in v1_rows:
        result = results_v1[name]
        counts_v1[SYMBOL[result["outcome"]]] += 1
        out.emit(f"{SYMBOL[result['outcome']]} [{name}] 单测层 {result['outcome']}：{result['line']}（{result['seconds']:.1f} 秒）")
    out.emit(f"计数（v1 的 X1–X20 在 v2 副本上，单测层）：抓到 {counts_v1['✅']}、无效 {counts_v1['⏭']}、没红 {counts_v1['❌']}、超时 {counts_v1['⏱']}、"
             f"原文没命中一次 {counts_v1['⛔']}，合计 {sum(counts_v1.values())} / 表 {len(v1_rows)} 条")
    # 二、v2 的 X22–X26、X28–X31：单测层一遍、批层一趟
    results_u2 = unit_level(out, "v2-X22-X31", v2_rows, scratch)
    jobs = {identifier: ["I1"] for identifier in ("X22", "X23", "X28", "X29")}
    jobs.update({identifier: ["I1", "row-242", "row-269", "row-270"] for identifier in ("X24", "X25", "X26", "X30", "X31")})
    results_b2 = batch_level(out, v2_rows, jobs, scratch, arguments.workers, arguments.results_directory)
    counts_v2 = {"✅": 0, "⏭": 0, "❌": 0, "⏱": 0, "⛔": 0}
    for name, _anchor, _replacement in v2_rows:
        identifier = name.split("_", 1)[0]
        unit = results_u2[name]
        batch = results_b2.get((identifier, "I1"), {})
        sample = batch.get("sample_point", "not_run")
        if unit["outcome"] in ("invalid", "anchor_problem", "timeout"):
            outcome = unit["outcome"]
        else:
            outcome = "caught" if unit["outcome"] == "caught" or sample == "true" else "not_red"
        counts_v2[SYMBOL[outcome]] += 1
        extra = {label: results_b2[(identifier, label)]["sample_point"] for label in jobs[identifier] if (identifier, label) in results_b2}
        note = ""
        if identifier in ("X22", "X23", "X28", "X29") and sample != "true":
            note = "；注意：登记第九节写的批层取样点上输出没按登记变"
        if identifier not in ("X22", "X23", "X28", "X29") and any(value != "unit_only_output_unchanged" for value in extra.values()):
            note = "；注意：登记写「真实流上不改变输出」，这一趟里有导出上输出变了"
        out.emit(f"{SYMBOL[outcome]} [{name}] {outcome}：单测层 {unit['outcome']}（{unit['line']}）；批层取样点 {extra}{note}")
    out.emit(f"计数（v2 的 X22–X26、X28–X31，单测层加批层）：抓到 {counts_v2['✅']}、无效 {counts_v2['⏭']}、没红 {counts_v2['❌']}、超时 {counts_v2['⏱']}、"
             f"原文没命中一次 {counts_v2['⛔']}，合计 {sum(counts_v2.values())} / 表 {len(v2_rows)} 条")
    # 三、XC3–XC7
    results_xc = xc_level(out, xc_rows, scratch, arguments.results_directory)
    counts_xc = {"✅": 0, "⏭": 0, "❌": 0, "⏱": 0, "⛔": 0}
    for name, _anchor, _replacement in xc_rows:
        identifier = name.split("_", 1)[0]
        result = results_xc[identifier]
        counts_xc[SYMBOL[result["outcome"]]] += 1
        out.emit(f"{SYMBOL[result['outcome']]} [{name}] {result['outcome']}：{result['detail']}")
    out.emit(f"计数（XC3–XC7，比对器与参照谓词）：抓到 {counts_xc['✅']}、无效 {counts_xc['⏭']}、没红 {counts_xc['❌']}、超时 {counts_xc['⏱']}、"
             f"原文没命中一次 {counts_xc['⛔']}，合计 {sum(counts_xc.values())} / 表 {len(xc_rows)} 条")
    total = {key: counts_v1[key] + counts_v2[key] + counts_xc[key] for key in counts_v1}
    v1_after, v2_after = all_digests(V1_DIRECTORY), all_digests(V2_DIRECTORY)
    out.emit(f"name=restore v1_directory_unchanged={str(v1_after == v1_before).lower()} v2_directory_unchanged={str(v2_after == v2_before).lower()} "
             f"v1_files={len(v1_after)} v2_files={len(v2_after)}")
    if v1_after == v1_before and v2_after == v2_before:
        out.emit("已还原，基线仍全绿（v1、v2 两个目录逐文件 sha256 与开跑前相同；单测基线见 name=unit_baseline）")
    out.emit(f"计数（合计）：抓到 {total['✅']}、无效 {total['⏭']}、没红 {total['❌']}、超时 {total['⏱']}、原文没命中一次 {total['⛔']}，"
             f"合计 {sum(total.values())} / 表 {len(v1_rows) + len(v2_rows) + len(xc_rows)} 条")
    out.emit(f"name=third_segment_complete seconds={time.perf_counter() - started:.1f}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
