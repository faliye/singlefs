#!/usr/bin/env python3
"""新 O3 执行器目录 `e170_second_run_o3_spec_executor_journal_header_496/` 的登记检查（research/prompts/e171-r3-prereg.md 第六节 V1 至 V5、第九节 M1 至 M5、M9、M10）。

子命令（每条的输出都是 `name=…` 行，最后一行 `name=<检查>_verdict ok=true|false`；退出码 0 = 全过，1 = 有红）：
  footprint      V1：新目录相对基底目录改了哪些文件、六份不动的文件两边 sha256、executor_main.py 只差 A4 钉值一行；`manifest` 打旧目录的 sha256 清单（改动前后各打一份，用 diff 比）
  read-v2        V2：读执行器全量输出，逐格报 base_recovery、锚点 A1 至 A6 / A11、dump_check、torn_reading、完成标记
  independence   V3：I1 至 I5（grep crates 的命中、import 与 open 的静态检查、离仓运行、注释出处、种下 open("crates/…") 的阴性）
  seeded-faults  V4：在新目录的草稿拷贝里种 P-a、P-b、P-c 三条故障，各起执行器全量输出，读 V2 的各格
  unit-mutations 第九节 M4、M5：在草稿拷贝里改 replay_prefix.py / journal_ring.py，跑拷贝自己的单测，必须红
  per-state      V5：冻结目录与新目录各自用自己单测里的合成池，逐态比 (strict, provisional, landed)
  state-table    per-state 的子进程：在给定执行器目录里造合成池、逐态判，打表（不单独用）

不读 crates/、不写仓里的任何文件；草稿拷贝都建在 --scratch 下（调用者给的仓外目录）。
"""

from __future__ import annotations

import argparse
import ast
import difflib
import hashlib
import os
import re
import shutil
import subprocess
import sys
from dataclasses import dataclass

STREAM_NAMES = ("new_pool_file_creation", "fixed_script_through_e_then_unmount")
# V1 的门槛（登记 V1，第十二节修订 8 收成九个）：改动文件 ⊆ 这九个；下面六份两边逐字相同；executor_main.py 只许差 A4 钉值那一行
CHANGED_FILES_ALLOWED = frozenset({
    "on_disk_layout.py", "system_configuration.py", "replay_prefix.py", "journal_ring.py", "spec_gaps.py", "spec-gaps.tsv",
    "synthetic_pool_for_tests.py", "test_executor_on_synthetic_pool.py", "executor_main.py"})
UNCHANGED_FILES = ("executor_core.py", "root_ring.py", "version_table.py", "crash_dump.py", "crash_image.py", "crc32c_castagnoli.py")
EXECUTOR_MAIN_FILE = "executor_main.py"
A4_PIN_BEFORE = "(5, 1, 3, 1)"
A4_PIN_AFTER = "(6, 0, 3, 1)"
COMMENT_MARKS = ("#", '"""')
KB_CITATION = re.compile(r"(\.claude/kb/\S+?\.md)@")
# V3 I2：open 的路径里 os.path.join 只许拼这些固定文件名（导出目录里的文件，加执行器自己目录里的缺口表）
FIXED_FILE_NAMES = frozenset({"manifest.tsv", "base-sectors.bin", "ops-payload.bin", "torn-payload.bin", "ops.tsv", "calls.tsv",
                              "torn.tsv", "states.tsv", "spec-gaps.tsv"})
OPTIONAL_THIRD_PARTY_MODULES = frozenset({"google_crc32c"})
REPOSITORY_PATH_PATTERN = re.compile(r"^(?:/|\./|\.\./)|(?:crates|scripts|\.claude|research)/")
SEEDED_OPEN_LINE = 'open("crates/singlefs-format/src/lib.rs")\n'


def emit(lines: list[str], line: str) -> None:
    lines.append(line)
    print(line, flush=True)


def lower(flag: bool) -> str:
    return "true" if flag else "false"


def sha256_hex(content: bytes | None) -> str:
    return "missing" if content is None else hashlib.sha256(content).hexdigest()


def is_source_name(name: str) -> bool:
    return not name.startswith(".") and not name.endswith(".pyc")


def read_directory_files(directory: str) -> dict[str, bytes]:
    """目录顶层的普通文件（不含 __pycache__ 与 .pyc）。"""
    files = {}
    for name in sorted(os.listdir(directory)):
        path = os.path.join(directory, name)
        if os.path.isfile(path) and is_source_name(name):
            with open(path, "rb") as handle:
                files[name] = handle.read()
    return files


def footprint_lines(base_directory: str, new_directory: str) -> tuple[list[str], bool]:
    """V1：改动文件集合、六份不动文件的 sha256、executor_main.py 只差 A4 钉值那一行。"""
    base, new = read_directory_files(base_directory), read_directory_files(new_directory)
    changed = sorted(name for name in set(base) | set(new) if base.get(name) != new.get(name))
    outside = [name for name in changed if name not in CHANGED_FILES_ALLOWED]
    lines: list[str] = []
    emit(lines, f"name=v1_changed_files count={len(changed)} files={','.join(changed) or '-'}")
    emit(lines, f"name=v1_outside_allowed count={len(outside)} files={','.join(outside) or '-'}")
    unequal = []
    for name in UNCHANGED_FILES:
        base_hash, new_hash = sha256_hex(base.get(name)), sha256_hex(new.get(name))
        equal = base_hash == new_hash and base_hash != "missing"
        if not equal:
            unequal.append(name)
        emit(lines, f"name=v1_sha256 file={name} base={base_hash} new={new_hash} equal={lower(equal)}")
    only_the_pin = executor_main_differs_only_in_the_a4_pin(base.get(EXECUTOR_MAIN_FILE), new.get(EXECUTOR_MAIN_FILE))
    emit(lines, f"name=v1_executor_main_diff only_the_a4_pin={lower(only_the_pin)} before={A4_PIN_BEFORE} after={A4_PIN_AFTER}")
    ok = not outside and not unequal and only_the_pin
    emit(lines, f"name=v1_verdict ok={lower(ok)} outside_allowed={len(outside)} unequal_of_six={len(unequal)} "
                f"executor_main_only_the_a4_pin={lower(only_the_pin)}")
    return lines, ok


def executor_main_differs_only_in_the_a4_pin(base_content: bytes | None, new_content: bytes | None) -> bool:
    """executor_main.py 相对基底恰好一行不同：A4 锚点那一行里的 (5, 1, 3, 1) 换成 (6, 0, 3, 1)，别的字节一个不动。"""
    if base_content is None or new_content is None:
        return False
    base_lines, new_lines = base_content.decode("utf-8").split("\n"), new_content.decode("utf-8").split("\n")
    if len(base_lines) != len(new_lines):
        return False
    differing = [(before, after) for before, after in zip(base_lines, new_lines) if before != after]
    if len(differing) != 1:
        return False
    before, after = differing[0]
    return "id=A4" in before and before.count(A4_PIN_BEFORE) == 1 and before.replace(A4_PIN_BEFORE, A4_PIN_AFTER) == after


def manifest_lines(directories: list[str]) -> list[str]:
    """`sha256sum` 同形的清单（<哈希>两个空格<路径>），按路径字节序；不含 __pycache__ 与 .pyc。"""
    entries = []
    for directory in directories:
        for root, subdirectories, names in os.walk(directory):
            subdirectories[:] = sorted(name for name in subdirectories if name != "__pycache__")
            for name in sorted(names):
                if name.endswith(".pyc"):
                    continue
                path = os.path.join(root, name)
                with open(path, "rb") as handle:
                    entries.append((path, hashlib.sha256(handle.read()).hexdigest()))
    return [f"{digest}  {path}" for path, digest in sorted(entries, key=lambda entry: entry[0].encode())]


def field_of(line: str, key: str) -> str | None:
    """`k=v` 词里第一处 `key=` 的值（值里不含空白的部分）。锚点行的后半截带元组与空格，只取前几个词。"""
    match = re.search(rf"(?:^|\s){re.escape(key)}=(\S*)", line)
    return None if match is None else match.group(1)


def read_v2_lines(output_lines: list[str]) -> tuple[list[str], bool, list[str]]:
    """V2 的读法（登记 V2）：base_recovery 无 recovered=false；锚点流一 A1 至 A6、流二 A6 与 A11 逐条 holds=true
    （holds=not_run 与缺行都算不过）；dump_check 三个计数全 0；torn_reading 的 differing_entries=0；末行 name=complete。
    A7、A12、A13 只报不判。"""
    required_anchors = {STREAM_NAMES[0]: ("A1", "A2", "A3", "A4", "A5", "A6"), STREAM_NAMES[1]: ("A6", "A11")}
    informational_anchors = {STREAM_NAMES[0]: ("A7", "A12", "A13"), STREAM_NAMES[1]: ()}
    lines: list[str] = []
    failing: list[str] = []

    def cell(name: str, stream: str, observed: str, passed: bool) -> None:
        if not passed:
            failing.append(f"{name}:{stream}")
        lines.append(f"name=v2_cell cell={name} stream={stream} observed={observed} ok={lower(passed)}")

    for stream in STREAM_NAMES:
        loaded = any(line.startswith("name=load ") and field_of(line, "stream") == stream for line in output_lines)
        refused = [line for line in output_lines
                   if line.startswith("name=base_recovery ") and field_of(line, "stream") == stream and field_of(line, "recovered") == "false"]
        detail = field_of(refused[0], "detail") if refused else "-"
        cell("base_recovery", stream, f"loaded={lower(loaded)},refused={len(refused)},detail={detail}", loaded and not refused)
        anchor_lines = [line for line in output_lines if line.startswith("name=anchor_assertion ") and field_of(line, "stream") == stream]
        for anchor in required_anchors[stream]:
            holds = [field_of(line, "holds") for line in anchor_lines if field_of(line, "id") == anchor]
            observed = holds[0] if holds else "absent"
            cell(f"anchor_{anchor}", stream, observed, observed == "true")
        for anchor in informational_anchors[stream]:
            holds = [field_of(line, "holds") for line in anchor_lines if field_of(line, "id") == anchor]
            lines.append(f"name=v2_info anchor={anchor} stream={stream} holds={holds[0] if holds else 'absent'}")
        checks = [line for line in output_lines if line.startswith("name=dump_check ") and field_of(line, "stream") == stream]
        counts = [field_of(checks[0], key) for key in ("count_mismatches", "checksum_mismatches", "version_table_anomalies")] if checks else []
        cell("dump_check", stream, ",".join(count or "absent" for count in counts) or "absent", bool(counts) and all(count == "0" for count in counts))
        torn = [line for line in output_lines if line.startswith("name=torn_reading ") and field_of(line, "stream") == stream]
        differing = field_of(torn[0], "differing_entries") if torn else None
        cell("torn_reading", stream, f"differing_entries={differing or 'absent'}", differing == "0")
    complete = bool(output_lines) and output_lines[-1] == "name=complete"
    cell("complete_marker", "-", lower(complete), complete)
    for line in output_lines:
        if line.startswith("name=verdict "):
            lines.append(f"name=v2_info verdict_line={line.replace(' ', ';')}")
    ok = not failing
    lines.append(f"name=v2_verdict ok={lower(ok)} failing_cells={','.join(failing) or '-'}")
    return lines, ok, failing


def read_text_lines(path: str) -> list[str]:
    with open(path, encoding="utf-8") as handle:
        return handle.read().splitlines()


# ---- V3：独立性 ----

def source_file_names(directory: str) -> list[str]:
    return [name for name in sorted(os.listdir(directory)) if name.endswith(".py") and os.path.isfile(os.path.join(directory, name))]


def crates_hits(directory: str) -> list[str]:
    """I1：`grep -n 'crates' <目录>/*.py` 的命中行，去掉目录前缀（`文件:行号:正文`）。"""
    hits = []
    for name in source_file_names(directory):
        with open(os.path.join(directory, name), encoding="utf-8") as handle:
            for line_number, text in enumerate(handle.read().splitlines(), start=1):
                if "crates" in text:
                    hits.append(f"{name}:{line_number}:{text}")
    return hits


def import_violations(directory: str) -> list[str]:
    """I2 前半：import 目标只许是标准库、同级模块、google_crc32c。"""
    siblings = {name[:-3] for name in source_file_names(directory)}
    allowed = set(sys.stdlib_module_names) | siblings | OPTIONAL_THIRD_PARTY_MODULES
    violations = []
    for name in source_file_names(directory):
        with open(os.path.join(directory, name), encoding="utf-8") as handle:
            tree = ast.parse(handle.read(), filename=name)
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                targets = [alias.name.split(".")[0] for alias in node.names]
            elif isinstance(node, ast.ImportFrom):
                targets = ["." * node.level + (node.module or "")] if node.level else [(node.module or "").split(".")[0]]
            else:
                continue
            for target in targets:
                if target not in allowed:
                    violations.append(f"{name}:{node.lineno}: import {target}")
    return violations


def dotted_name(node: ast.AST) -> str:
    if isinstance(node, ast.Name):
        return node.id
    if isinstance(node, ast.Attribute):
        return dotted_name(node.value) + "." + node.attr
    return ""


def path_expression_is_allowed(node: ast.AST) -> bool:
    """open 的第一个参数：变量或属性（来自命令行参数、函数参数）、或 os.path.join / dirname / abspath 拼出的、
    其中的字符串常量只许是导出目录里的固定文件名。"""
    if isinstance(node, (ast.Name, ast.Attribute)):
        return True
    if isinstance(node, ast.Constant):
        return isinstance(node.value, str) and node.value in FIXED_FILE_NAMES
    if isinstance(node, ast.Call) and dotted_name(node.func) in ("os.path.join", "os.path.dirname", "os.path.abspath"):
        return all(path_expression_is_allowed(argument) for argument in node.args)
    return False


def open_violations(directory: str) -> list[str]:
    """I2 后半：每个 open( 的路径都来自命令行参数或固定文件名；另报非文档串的仓内路径形字符串常量。"""
    violations = []
    for name in source_file_names(directory):
        with open(os.path.join(directory, name), encoding="utf-8") as handle:
            tree = ast.parse(handle.read(), filename=name)
        documentation = {id(node.body[0].value) for node in ast.walk(tree)
                         if isinstance(node, (ast.Module, ast.ClassDef, ast.FunctionDef, ast.AsyncFunctionDef)) and node.body
                         and isinstance(node.body[0], ast.Expr) and isinstance(node.body[0].value, ast.Constant)}
        for node in ast.walk(tree):
            if isinstance(node, ast.Call) and dotted_name(node.func) == "open":
                first = node.args[0] if node.args else next((keyword.value for keyword in node.keywords if keyword.arg == "file"), None)
                if first is None or not path_expression_is_allowed(first):
                    violations.append(f"{name}:{node.lineno}: open 的路径不是命令行参数或固定文件名")
            elif isinstance(node, ast.Constant) and isinstance(node.value, str) and id(node) not in documentation \
                    and "\n" not in node.value and REPOSITORY_PATH_PATTERN.search(node.value):
                violations.append(f"{name}:{node.lineno}: 仓内路径形字符串 {node.value[:60]!r}")
    return violations


def changed_line_ranges(base_text: str, new_text: str) -> tuple[list[str], list[tuple[int, int]]]:
    """新文件里相对基底新增或改写的行区间（半开，从 0 起）。"""
    new_lines = new_text.splitlines()
    matcher = difflib.SequenceMatcher(None, base_text.splitlines(), new_lines, autojunk=False)
    return new_lines, [(j1, j2) for tag, _i1, _i2, j1, j2 in matcher.get_opcodes() if tag in ("replace", "insert")]


def uncited_changes(base_text: str, new_text: str, lines_before: int = 6, lines_after: int = 3) -> list[tuple[int, str]]:
    """I4 前半：新增或改写的区间里带注释（`#` 或文档串的三引号）的，连同前 6 行后 3 行里要有一处 `.claude/kb/…md@…` 形的出处；
    没有的返回（行号，区间首行）。只改代码、不带注释的区间不查。"""
    new_lines, ranges = changed_line_ranges(base_text, new_text)
    return [(first + 1, new_lines[first]) for first, last in ranges
            if any(mark in line for line in new_lines[first:last] for mark in COMMENT_MARKS)
            and not any(KB_CITATION.search(line) for line in new_lines[max(0, first - lines_before):last + lines_after])]


def cited_kb_paths(text: str) -> list[str]:
    return [match.group(1) for match in KB_CITATION.finditer(text)]


def crates_in_changed_lines(base_text: str, new_text: str) -> list[tuple[int, str]]:
    """I4 后半：新增或改写的行里出现 `crates/`。"""
    new_lines, ranges = changed_line_ranges(base_text, new_text)
    return [(line_number + 1, new_lines[line_number]) for first, last in ranges for line_number in range(first, last)
            if "crates/" in new_lines[line_number]]


def citation_lines(lines: list[str], base_directory: str, new_directory: str, repository_root: str) -> bool:
    """I4：改动文件逐个查；被引的 kb 文件都要真在仓里（小节标题逐字由人读 `git diff` 核）。"""
    base, new = read_directory_files(base_directory), read_directory_files(new_directory)
    problems = []
    for name in sorted(name for name in new if base.get(name) != new[name]):
        for path in sorted(set(cited_kb_paths(new[name].decode("utf-8")))):
            if not os.path.isfile(os.path.join(repository_root, path)):
                problems.append(f"{name}: 引的 kb 文件不在仓里：{path}")
    for name in sorted(name for name in new if base.get(name) != new[name]):
        base_text = base.get(name, b"").decode("utf-8")
        new_text = new[name].decode("utf-8")
        problems += [f"{name}:{line}: 改动区间无 kb 出处：{text[:50]}" for line, text in uncited_changes(base_text, new_text)]
        problems += [f"{name}:{line}: 改动行写了 crates/：{text[:50]}" for line, text in crates_in_changed_lines(base_text, new_text)]
    for problem in problems:
        emit(lines, f"name=v3_i4_problem {problem.replace(' ', '_')}")
    return not problems


def copy_executor_directory(source: str, destination: str) -> None:
    if os.path.exists(destination):
        shutil.rmtree(destination)
    shutil.copytree(source, destination, ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))


def run_executor(executor_directory: str, dumps: str, output: str, workers: int, working_directory: str | None = None,
                 clear_python_path: bool = False) -> subprocess.CompletedProcess:
    """起执行器全量输出（登记 V2 的命令）。字节码不落盘；clear_python_path 时环境里只留 PATH 与 HOME（离仓运行用，PYTHONPATH 不带）。"""
    command = [sys.executable, "-B", os.path.join(executor_directory, "executor_main.py"), "--dumps", dumps,
               "--workers", str(workers), "--output", output]
    environment = {"PATH": os.environ.get("PATH", ""), "HOME": os.environ.get("HOME", ""), "PYTHONDONTWRITEBYTECODE": "1"} if clear_python_path \
        else dict(os.environ, PYTHONDONTWRITEBYTECODE="1")
    return subprocess.run(command, cwd=working_directory or executor_directory, env=environment, capture_output=True, text=True, check=False)


def normalised_output(lines: list[str]) -> list[str]:
    """离仓运行与仓内运行逐行比时去掉的东西：耗时行整行；其余行里的耗时词（_ns 结尾）、工作进程号、导出目录路径；
    各态判定行的次序随进程调度变，比的时候排序。"""
    kept = []
    for line in lines:
        if line.startswith(("name=timing ", "name=state_time ")):
            continue
        words = [word for word in line.split(" ") if not (word.split("=", 1)[0].endswith("_ns") or word.startswith(("worker=", "dumps=")))]
        kept.append(" ".join(words))
    return sorted(kept)


def seeded_copy_with_crates_open(source: str, destination: str) -> None:
    """I5：新目录的拷贝里往 system_configuration.py 末尾加一行 open("crates/…")。"""
    copy_executor_directory(source, destination)
    with open(os.path.join(destination, "system_configuration.py"), "a", encoding="utf-8") as handle:
        handle.write("\n" + SEEDED_OPEN_LINE)


def static_independence_findings(directory: str, base_directory: str) -> dict[str, list[str]]:
    """I1 与 I2 的读数：I1 = 新目录的 crates 命中与基底目录逐行不同的部分；I2 = import 与 open 的违例。"""
    new_hits, base_hits = crates_hits(directory), crates_hits(base_directory)
    return {"I1": sorted(set(new_hits) ^ set(base_hits)) if new_hits != base_hits else [],
            "I2": import_violations(directory) + open_violations(directory)}


def independence_lines(new_directory: str, base_directory: str, dumps: str, reference_output: str, scratch: str, workers: int,
                       repository_root: str) -> bool:
    lines: list[str] = []
    verdicts = {}
    findings = static_independence_findings(new_directory, base_directory)
    for check, items in findings.items():
        for item in items:
            emit(lines, f"name=v3_{check.lower()}_finding {item.replace(' ', '_')}")
        verdicts[check] = not items
        emit(lines, f"name=v3_{check.lower()} findings={len(items)} ok={lower(not items)} new_hits={len(crates_hits(new_directory))} "
                    f"base_hits={len(crates_hits(base_directory))}")
    # I3 离仓运行：拷到仓外草稿目录，清空 PYTHONPATH，对同一份 dump 跑 V2 的命令，与仓内跑的产物比
    off_repository = os.path.join(scratch, "off-repository-copy")
    copy_executor_directory(new_directory, off_repository)
    output = os.path.join(scratch, "off-repository.out")
    if os.path.exists(output):
        os.remove(output)
    completed = run_executor(off_repository, dumps, output, workers, working_directory=off_repository, clear_python_path=True)
    same = False
    if completed.returncode == 0 and os.path.exists(output):
        same = normalised_output(read_text_lines(output)) == normalised_output(read_text_lines(reference_output))
    verdicts["I3"] = same
    emit(lines, f"name=v3_i3 exit_code={completed.returncode} same_as_in_repository={lower(same)} ok={lower(same)}")
    # I4 注释出处
    verdicts["I4"] = citation_lines(lines, base_directory, new_directory, repository_root)
    emit(lines, f"name=v3_i4 ok={lower(verdicts['I4'])}")
    # I5 阴性：种下 open("crates/…") 的拷贝，I1 与 I2 必须都红
    seeded = os.path.join(scratch, "seeded-crates-open")
    seeded_copy_with_crates_open(new_directory, seeded)
    seeded_findings = static_independence_findings(seeded, base_directory)
    i5_ok = bool(seeded_findings["I1"]) and bool(seeded_findings["I2"])
    verdicts["I5"] = i5_ok
    emit(lines, f"name=v3_i5 seeded_i1_findings={len(seeded_findings['I1'])} seeded_i2_findings={len(seeded_findings['I2'])} ok={lower(i5_ok)}")
    ok = all(verdicts.values())
    emit(lines, f"name=v3_verdict ok={lower(ok)} red={','.join(check for check, passed in verdicts.items() if not passed) or '-'}")
    return ok


# ---- V4 与第九节 M4、M5：种下的故障 ----

@dataclass(frozen=True)
class Edit:
    file: str
    old: str
    new: str


@dataclass(frozen=True)
class SeededFault:
    name: str
    mutation: str
    edits: tuple[Edit, ...]
    expected_cells: tuple[str, ...]  # 登记点名的红格，写成 `格:流`
    expectation: str  # "all"：全红才算红在点名的格；"any"：任一红即可
    keeps_import_checks: bool = False  # 保留 on_disk_layout.py 末尾的两个载入时断言（信息臂，不判）
    crash_counts: bool = False  # 修订 9：执行器崩溃（退出码非 0）也算抓到；崩溃与红在点名的格仍分开记


NEUTRALISE_IMPORT_CHECKS = Edit("on_disk_layout.py", "\nassert_kb_anchors_hold()\nassert_every_offset_the_layout_lines_list_holds()\n", "\n")
REMOVE_JOURNAL_THRESHOLD_AND_PINNED_ROWS = (
    Edit("on_disk_layout.py", '    ("new_root_reclamation_threshold", 8), ("new_root_pinned_slot", 1),\n', ""),
    Edit("on_disk_layout.py", "JOURNAL_HEADER_BYTES = 496", "JOURNAL_HEADER_BYTES = 487"),
    Edit("on_disk_layout.py", "JOURNAL_NEW_ROOT_SEGMENT_BYTES = 373", "JOURNAL_NEW_ROOT_SEGMENT_BYTES = 364"))
REMOVE_ROOT_THRESHOLD_AND_PINNED_ROWS = (
    Edit("on_disk_layout.py", '("allocation_records_root_pointer", 86), ("reclamation_threshold", 8), ("pinned_slot", 1),',
         '("allocation_records_root_pointer", 86),'),
    Edit("on_disk_layout.py", "ROOT_RECORD_BYTES = 466", "ROOT_RECORD_BYTES = 457"))
INCOMPATIBLE_BITS_BACK_TO_BIT_ONE = Edit("system_configuration.py", "FIRST_VERSION_INCOMPATIBLE_BITS = (1 << 1) | (1 << 2)",
                                          "FIRST_VERSION_INCOMPATIBLE_BITS = 1 << 1")
MOVE_ROOT_THRESHOLD_AND_PINNED_ROWS_BEFORE_THE_SELF_CHECKSUM = (
    Edit("on_disk_layout.py", '("allocation_records_root_pointer", 86), ("reclamation_threshold", 8), ("pinned_slot", 1),',
         '("allocation_records_root_pointer", 86),'),
    Edit("on_disk_layout.py", '("rollback_floor", 8),\n    ("self_checksum", 32),',
         '("rollback_floor", 8),\n    ("reclamation_threshold", 8), ("pinned_slot", 1), ("self_checksum", 32),'))
SEEDED_FAULTS = (
    SeededFault("P-a", "M1", (INCOMPATIBLE_BITS_BACK_TO_BIT_ONE,),
                (f"base_recovery:{STREAM_NAMES[0]}", f"base_recovery:{STREAM_NAMES[1]}"), "all"),
    SeededFault("P-b", "M2", REMOVE_JOURNAL_THRESHOLD_AND_PINNED_ROWS + (NEUTRALISE_IMPORT_CHECKS,),
                (f"anchor_A3:{STREAM_NAMES[0]}", f"dump_check:{STREAM_NAMES[0]}"), "any"),
    SeededFault("P-c", "M3", REMOVE_ROOT_THRESHOLD_AND_PINNED_ROWS + (NEUTRALISE_IMPORT_CHECKS,),
                (f"anchor_A2:{STREAM_NAMES[0]}",), "all", crash_counts=True),
    # 修订 9 补的臂：t、p 两行放在自证校验和那一行之前（字段表次序读错），根记录总宽不变，校验和偏移后移 9，真 dump 上每个根都不自证，不崩溃
    SeededFault("P-c-misplaced-rows", "M3b", MOVE_ROOT_THRESHOLD_AND_PINNED_ROWS_BEFORE_THE_SELF_CHECKSUM + (NEUTRALISE_IMPORT_CHECKS,),
                (f"anchor_A2:{STREAM_NAMES[0]}", f"base_recovery:{STREAM_NAMES[0]}"), "any"),
    SeededFault("P-b-import-checks-kept", "M2", REMOVE_JOURNAL_THRESHOLD_AND_PINNED_ROWS, (), "any", keeps_import_checks=True),
    SeededFault("P-c-import-checks-kept", "M3", REMOVE_ROOT_THRESHOLD_AND_PINNED_ROWS, (), "any", keeps_import_checks=True),
)


def apply_edits(directory: str, edits: tuple[Edit, ...]) -> None:
    """每个替换在文件里恰好命中一次，否则抛 ValueError（种下的故障必须落在它说的那一处）。"""
    for edit in edits:
        path = os.path.join(directory, edit.file)
        with open(path, encoding="utf-8") as handle:
            text = handle.read()
        if text.count(edit.old) != 1:
            raise ValueError(f"{edit.file}: 要替换的文字命中 {text.count(edit.old)} 次（须恰好 1 次）：{edit.old[:60]!r}")
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text.replace(edit.old, edit.new))


def seeded_fault_lines(fault: SeededFault, new_directory: str, dumps: str, scratch: str, workers: int) -> tuple[list[str], bool]:
    """种一条故障、起执行器全量输出、读 V2 各格。崩溃（退出码非 0）与「红在点名的格」分开记。返回（行，红在点名的格）。"""
    work = os.path.join(scratch, f"fault-{fault.name}")
    copy_executor_directory(new_directory, work)
    apply_edits(work, fault.edits)
    output = work + "-output.out"
    if os.path.exists(output):
        os.remove(output)
    completed = run_executor(work, dumps, output, workers)
    output_lines = read_text_lines(output) if os.path.exists(output) else []
    _lines, _ok, failing = read_v2_lines(output_lines)
    finished = completed.returncode == 0 and bool(output_lines) and output_lines[-1] == "name=complete"
    named_red = [cell for cell in fault.expected_cells if cell in failing]
    red_in_named = finished and bool(fault.expected_cells) and \
        (len(named_red) == len(fault.expected_cells) if fault.expectation == "all" else bool(named_red))
    caught = red_in_named or (fault.crash_counts and completed.returncode != 0)
    stderr_first = (completed.stderr.strip().splitlines() or ["-"])[-1][:160].replace(" ", "_")
    lines: list[str] = []
    emit(lines, f"name=v4_fault fault={fault.name} mutation={fault.mutation} exit_code={completed.returncode} finished={lower(finished)} "
                f"crashed={lower(completed.returncode != 0)} red_cells={','.join(failing) or '-'} expected_cells={','.join(fault.expected_cells) or '-'} "
                f"expectation={fault.expectation} red_in_named_cell={lower(red_in_named)} crash_counts={lower(fault.crash_counts)} "
                f"caught={lower(caught)} last_stderr_line={stderr_first}")
    return lines, caught


def unit_mutation_table() -> tuple[tuple[str, Edit, str], ...]:
    """第九节 M4、M5：（名, 替换, 必须红的单测名片段）。M5 里取「多放一个点名项」，因为代码里没有字面的 64 与 65，上限由字节边界给出。"""
    return (
        ("M4-record-branch", Edit("replay_prefix.py", 'reclamation_threshold=last_applied.unsigned("new_root_reclamation_threshold")',
                                  "reclamation_threshold=0"), "test_u_f2"),
        ("M4-record-branch-pinned-slot", Edit("replay_prefix.py", 'pinned_slot=last_applied.unsigned("new_root_pinned_slot")', "pinned_slot=255"),
         "test_u_f2"),
        ("M4-root-branch", Edit("replay_prefix.py", 'root.unsigned("reclamation_threshold"), root.unsigned("pinned_slot"))', "0, 255)"),
         "test_u_f8"),
        ("M5-one-more-item", Edit("journal_ring.py", "named_item_count * layout.NAMED_ITEM_BYTES > record_bytes:",
                                  "named_item_count * layout.NAMED_ITEM_BYTES > record_bytes + layout.NAMED_ITEM_BYTES:"), "test_u_f4"),
        # 修订 8：A4 钉值改回取号一轮的 (5, 1, 3, 1)，取号两轮的小池上 A4 不再 holds=true
        ("M11-a4-pin-back-to-one-round", Edit("executor_main.py", "all(value == (6, 0, 3, 1) for value", "all(value == (5, 1, 3, 1) for value"),
         "test_a4_"),
    )


def unit_mutation_lines(new_directory: str, scratch: str) -> bool:
    lines: list[str] = []
    all_red = True
    test_scratch = os.path.join(scratch, "unit-test-scratch")
    os.makedirs(test_scratch, exist_ok=True)
    for name, edit, test_fragment in unit_mutation_table():
        work = os.path.join(scratch, f"unit-mutation-{name}")
        copy_executor_directory(new_directory, work)
        apply_edits(work, (edit,))
        completed = subprocess.run([sys.executable, "-B", "-m", "unittest", "-q", "test_executor_on_synthetic_pool"], cwd=work,
                                   env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1", E170_TEST_SCRATCH=test_scratch),
                                   capture_output=True, text=True, check=False)
        red_tests = sorted(set(re.findall(r"^(?:FAIL|ERROR): (\S+)", completed.stderr, flags=re.MULTILINE)))
        caught = completed.returncode != 0 and any(test_fragment in test for test in red_tests)
        all_red = all_red and caught
        emit(lines, f"name=unit_mutation mutation={name} exit_code={completed.returncode} must_be_red={test_fragment} "
                    f"caught={lower(caught)} red_tests={','.join(red_tests) or '-'}")
    emit(lines, f"name=unit_mutation_verdict ok={lower(all_red)}")
    return all_red


# ---- V5：逐态判定相同 ----

def sampled_landing_vectors(write_count: int, torn_write_ordinals: list[int]) -> list[str]:
    """同一组取样态（两边的合成池写数相同时同一组）：前缀持久 0 至 N 写；全持久缺一写；撕裂写在前缀末与全持久上各一。"""
    vectors = ["2" * persisted + "0" * (write_count - persisted) for persisted in range(write_count + 1)]
    vectors += ["2" * missing + "0" + "2" * (write_count - missing - 1) for missing in range(write_count)]
    for ordinal in torn_write_ordinals:
        vectors.append("2" * ordinal + "1" + "0" * (write_count - ordinal - 1))
        vectors.append("2" * ordinal + "1" + "2" * (write_count - ordinal - 1))
    return list(dict.fromkeys(vectors))


def state_table_rows(executor_directory: str, scratch: str) -> list[str]:
    """在给定执行器目录里用它自己的合成池造流、逐态判。第一行 history 是写流的骨架（设备、种类、偏移、长度、调用分段）的指纹。"""
    sys.path.insert(0, os.path.abspath(executor_directory))
    import synthetic_pool_for_tests as synthetic
    from crash_dump import load_stream_dump
    from crash_image import build_crash_image
    from executor_core import judge_state, prepare_stream_context, recover
    from replay_prefix import UnitChecksumCache
    import spec_gaps
    stream, _parts = synthetic.new_pool_file_creation_stream()
    write_count = synthetic.write_count_of(stream)
    torn_ordinals = sorted(synthetic.torn_writes_of(stream))
    vectors = sampled_landing_vectors(write_count, torn_ordinals)
    skeleton = repr([(device, kind, offset, length) for device, kind, offset, length, _payload in stream.operations]) + repr(stream.calls)
    dump_directory = os.path.join(scratch, "dump")
    synthetic.write_dump(dump_directory, stream, vectors)
    dump = load_stream_dump(dump_directory)
    context = prepare_stream_context(dump)
    rows = [f"history\twrites={write_count}\ttorn={','.join(str(ordinal) for ordinal in torn_ordinals)}\tvectors={len(vectors)}\t"
            f"skeleton_sha256={hashlib.sha256(skeleton.encode()).hexdigest()}"]
    for state in dump.states:
        try:
            judgement = judge_state(context, state, UnitChecksumCache())
            image = build_crash_image(dump, context.prepared_writes, state.landing_vector)
            landed = recover(image, dump.device_identities, dump.sector_bytes, UnitChecksumCache(), spec_gaps.GapsHitInOneState()).landed
            projection = hashlib.sha256(repr(landed).encode()).hexdigest()[:16]
            rows.append(f"{state.landing_vector}\t{judgement.strict_verdict}\t{judgement.provisional_verdict}\t{judgement.landed_text}\t{projection}")
        except Exception as error:  # 判不出的态照打一行，两边都出错的同一态也算相同
            rows.append(f"{state.landing_vector}\terror\terror\t{type(error).__name__}\t-")
    return rows


def per_state_lines(frozen_directory: str, new_directory: str, scratch: str) -> bool:
    """V5：两个目录各起一个子进程（模块名相同，不能放进同一个解释器）造合成池、逐态判，比表。"""
    tables = {}
    for label, directory in (("frozen", frozen_directory), ("new", new_directory)):
        work = os.path.join(scratch, f"per-state-{label}")
        os.makedirs(work, exist_ok=True)
        completed = subprocess.run([sys.executable, "-B", os.path.abspath(__file__), "state-table", "--executor-directory", directory,
                                    "--scratch", work], capture_output=True, text=True, check=False,
                                   env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1"))
        tables[label] = completed.stdout.splitlines() if completed.returncode == 0 else []
        if completed.returncode != 0:
            print(f"name=v5_table_failed label={label} exit_code={completed.returncode} "
                  f"last_stderr_line={(completed.stderr.strip().splitlines() or ['-'])[-1][:160].replace(' ', '_')}", flush=True)
    return compare_state_tables(tables["frozen"], tables["new"])


def compare_state_tables(frozen_rows: list[str], new_rows: list[str]) -> bool:
    lines: list[str] = []
    history_frozen = frozen_rows[0].split("\t")[1:] if frozen_rows else []
    history_new = new_rows[0].split("\t")[1:] if new_rows else []
    same_history = bool(history_frozen) and history_frozen == history_new
    emit(lines, f"name=v5_history same_abstract_history={lower(same_history)} frozen={'/'.join(history_frozen) or 'absent'} "
                f"new={'/'.join(history_new) or 'absent'}")
    frozen = {row.split("\t")[0]: row.split("\t")[1:] for row in frozen_rows[1:]}
    new = {row.split("\t")[0]: row.split("\t")[1:] for row in new_rows[1:]}
    differing = [vector for vector in sorted(set(frozen) | set(new)) if frozen.get(vector) != new.get(vector)]
    for vector in differing:
        emit(lines, f"name=v5_difference vector={vector} frozen={'/'.join(frozen.get(vector, ['absent']))} new={'/'.join(new.get(vector, ['absent']))}")
    errors = sum(1 for row in list(frozen.values()) + list(new.values()) if row[0] == "error")
    counts = {verdict: sum(1 for row in new.values() if row[1] == verdict) for verdict in ("in", "not_in")}
    ok = same_history and bool(frozen) and not differing and errors == 0
    emit(lines, f"name=v5_verdict ok={lower(ok)} states_frozen={len(frozen)} states_new={len(new)} differing_states={len(differing)} "
                f"error_rows={errors} provisional_in={counts['in']} provisional_not_in={counts['not_in']}")
    return ok


# ---- 命令行 ----

def build_argument_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="新 O3 执行器的登记检查（V1 至 V5、M4、M5）")
    subcommands = parser.add_subparsers(dest="command", required=True)
    footprint = subcommands.add_parser("footprint")
    footprint.add_argument("--base-directory", required=True)
    footprint.add_argument("--new-directory", required=True)
    manifest = subcommands.add_parser("manifest")
    manifest.add_argument("--directories", required=True, help="逗号隔开的目录")
    read_v2 = subcommands.add_parser("read-v2")
    read_v2.add_argument("--output-file", required=True)
    independence = subcommands.add_parser("independence")
    for name in ("--new-directory", "--base-directory", "--dumps", "--reference-output", "--scratch", "--repository-root"):
        independence.add_argument(name, required=True)
    independence.add_argument("--workers", type=int, required=True)
    seeded = subcommands.add_parser("seeded-faults")
    for name in ("--new-directory", "--dumps", "--scratch"):
        seeded.add_argument(name, required=True)
    seeded.add_argument("--workers", type=int, required=True)
    seeded.add_argument("--faults", default="", help="逗号隔开的故障名；不给就是全部")
    unit = subcommands.add_parser("unit-mutations")
    for name in ("--new-directory", "--scratch"):
        unit.add_argument(name, required=True)
    per_state = subcommands.add_parser("per-state")
    for name in ("--frozen-directory", "--new-directory", "--scratch"):
        per_state.add_argument(name, required=True)
    table = subcommands.add_parser("state-table")
    for name in ("--executor-directory", "--scratch"):
        table.add_argument(name, required=True)
    return parser


def run_seeded_faults(arguments: argparse.Namespace) -> bool:
    chosen = [name for name in arguments.faults.split(",") if name]
    faults = [fault for fault in SEEDED_FAULTS if not chosen or fault.name in chosen]
    unknown = sorted(set(chosen) - {fault.name for fault in SEEDED_FAULTS})
    if unknown:
        raise SystemExit(f"不认得的故障名：{','.join(unknown)}")
    registered_red = {}
    for fault in faults:
        _lines, red = seeded_fault_lines(fault, arguments.new_directory, arguments.dumps, arguments.scratch, arguments.workers)
        if fault.expected_cells:
            registered_red[fault.name] = red
    ok = all(registered_red.values())
    print(f"name=v4_verdict ok={lower(ok)} registered_faults_run={len(registered_red)} "
          f"caught={','.join(name for name, red in registered_red.items() if red) or '-'} "
          f"not_caught={','.join(name for name, red in registered_red.items() if not red) or '-'}", flush=True)
    return ok


def main(argument_list: list[str]) -> int:
    arguments = build_argument_parser().parse_args(argument_list)
    if arguments.command == "footprint":
        return 0 if footprint_lines(arguments.base_directory, arguments.new_directory)[1] else 1
    if arguments.command == "manifest":
        for line in manifest_lines([directory for directory in arguments.directories.split(",") if directory]):
            print(line)
        return 0
    if arguments.command == "read-v2":
        lines, ok, _failing = read_v2_lines(read_text_lines(arguments.output_file))
        for line in lines:
            print(line)
        return 0 if ok else 1
    os.makedirs(arguments.scratch, exist_ok=True)
    if arguments.command == "independence":
        return 0 if independence_lines(arguments.new_directory, arguments.base_directory, arguments.dumps, arguments.reference_output,
                                       arguments.scratch, arguments.workers, arguments.repository_root) else 1
    if arguments.command == "seeded-faults":
        return 0 if run_seeded_faults(arguments) else 1
    if arguments.command == "unit-mutations":
        return 0 if unit_mutation_lines(arguments.new_directory, arguments.scratch) else 1
    if arguments.command == "per-state":
        return 0 if per_state_lines(arguments.frozen_directory, arguments.new_directory, arguments.scratch) else 1
    for row in state_table_rows(arguments.executor_directory, arguments.scratch):
        print(row)
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
