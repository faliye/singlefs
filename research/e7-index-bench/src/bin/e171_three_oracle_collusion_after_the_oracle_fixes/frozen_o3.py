"""E171 第 1 次重跑 O3 一侧：冻结的执行器版本核对、起执行器、读执行器的判定行（第 1 次重跑登记第一节读法 9，原登记读法 7、8，
作废条款 Z2、Z4）。

冻结的那一版是 E170 第 2 次重跑 Q4 判甲交的 v2-487（调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 22 批「22-4b 交回」那一条）：
执行器目录 `research/e7-index-bench/src/bin/e170_o3_spec_executor_with_witness_predicates_journal_header_487/`，冻结行取
`research/results/e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out` 第 676–692 行（16 个文件行加种子行 `ce39b484ec109c7a`；那份产物里还有
v1-487 的冻结行，只取这一段）。冻结范围照那一版执行器自己的 `executor_main.FILES_OUTSIDE_THE_FREEZE`：不含名字以 `comparator`、
`mutation_driver`、`witness_reference`、`run_` 起头的文件。本实验不改那一份，只读、只起它。
"""

from __future__ import annotations

import hashlib
import os
import subprocess
import time
from dataclasses import dataclass, field

FROZEN_DIRECTORY_RELATIVE = "research/e7-index-bench/src/bin/e170_o3_spec_executor_with_witness_predicates_journal_header_487"
FREEZE_REFERENCE_RELATIVE = "research/results/e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out"
# 冻结行在那份产物里的行号（从 1 数，两端都含）。
FREEZE_REFERENCE_FIRST_LINE = 676
FREEZE_REFERENCE_LAST_LINE = 692
FILES_OUTSIDE_THE_FREEZE = ("comparator", "mutation_driver", "witness_reference", "run_")
FREEZE_PREFIXES = ("name=freeze ", "name=freeze_seed ")
FREEZE_FILE_PREFIX = "name=freeze file="
FREEZE_SEED_PREFIX = "name=freeze_seed "
EXECUTOR_SCRIPT = "executor_main.py"
VERDICT_ERROR = "error"
PARALLEL_REASON = "同一会话派出去的几个执行员各跑各的装置"


def strip_result_prefix(line: str) -> str:
    return line.removeprefix("E7RESULT ").rstrip("\n")


def frozen_lines_of(path: str, first_line: int = FREEZE_REFERENCE_FIRST_LINE, last_line: int = FREEZE_REFERENCE_LAST_LINE) -> list[str]:
    """一份产物第 `first_line`–`last_line` 行里的冻结行（文件行与种子行，允许前面带 `E7RESULT `），按原次序。那一段里有不是冻结行的行，
    或种子行不是最后一行，抛错（行号指错了那一版）。"""
    with open(path, encoding="utf-8") as handle:
        lines = [strip_result_prefix(line) for line in handle][first_line - 1:last_line]
    if len(lines) != last_line - first_line + 1 or not all(line.startswith(FREEZE_PREFIXES) for line in lines) \
            or not lines[-1].startswith(FREEZE_SEED_PREFIX):
        raise ValueError(f"{path} 第 {first_line}–{last_line} 行不是一整段冻结行")
    return lines


def frozen_hashes_of(lines: list[str]) -> dict[str, str]:
    """冻结行 → {文件名: sha256}。"""
    hashes = {}
    for line in lines:
        if line.startswith(FREEZE_FILE_PREFIX):
            fields = dict(token.split("=", 1) for token in line.split(" ")[1:])
            hashes[fields["file"]] = fields["sha256"]
    return hashes


def seed_of(lines: list[str]) -> str | None:
    for line in lines:
        if line.startswith(FREEZE_SEED_PREFIX):
            return line.split("sha256_prefix16=", 1)[1]
    return None


def sha256_of_file(path: str) -> str:
    with open(path, "rb") as handle:
        return hashlib.sha256(handle.read()).hexdigest()


def frozen_files_in(directory: str) -> list[str]:
    """执行器目录里属于冻结范围的文件：普通文件，名字不以 `comparator`、`mutation_driver` 起头，不是 `.pyc`。"""
    return sorted(name for name in os.listdir(directory)
                  if os.path.isfile(os.path.join(directory, name)) and not name.startswith(FILES_OUTSIDE_THE_FREEZE) and not name.endswith(".pyc"))


def seed_from_directory(directory: str) -> str:
    """执行器自己（`executor_main.freeze_lines`）怎么算的这里照算一遍：按文件名排序，内容逐个喂进 sha256，取前 16 个十六进制字符。"""
    hasher = hashlib.sha256()
    for name in frozen_files_in(directory):
        with open(os.path.join(directory, name), "rb") as handle:
            hasher.update(handle.read())
    return hasher.hexdigest()[:16]


def freeze_differences(repo_root: str, reference_lines: list[str] | None = None) -> list[str]:
    """Z2：冻结行逐个核（`sha256sum -c` 同义）：文件缺失、摘要不同、目录里有冻结范围之内而冻结行里没有的文件、种子不等，各记一条。空表示都对。"""
    directory = os.path.join(repo_root, FROZEN_DIRECTORY_RELATIVE)
    lines = reference_lines if reference_lines is not None else frozen_lines_of(os.path.join(repo_root, FREEZE_REFERENCE_RELATIVE))
    hashes = frozen_hashes_of(lines)
    differences = []
    for name, expected in hashes.items():
        path = os.path.join(directory, name)
        if not os.path.exists(path):
            differences.append(f"{name}:missing")
        elif sha256_of_file(path) != expected:
            differences.append(f"{name}:sha256_differs")
    differences.extend(f"{name}:not_in_the_freeze" for name in frozen_files_in(directory) if name not in hashes)
    if seed_from_directory(directory) != seed_of(lines):
        differences.append("freeze_seed:differs")
    return differences


@dataclass
class ExecutorReading:
    """一份执行器产物读回来的东西：每态（流名, 状态号）→（严格, 暂代）、底镜像恢复不出的流、出错态数、摘要对不上的态数、冻结行、是否有完成标记。"""

    verdicts: dict[tuple[str, int], tuple[str, str]] = field(default_factory=dict)
    base_unrecoverable_streams: list[str] = field(default_factory=list)
    error_states: int = 0
    digest_mismatch_states: int = 0
    dump_problems: int = 0
    freeze_lines: list[str] = field(default_factory=list)
    complete: bool = False
    state_lines: int = 0
    in_process_state_ns: int = 0


def key_value_fields(line: str) -> dict[str, str]:
    fields = {}
    for token in line.split(" "):
        key, separator, value = token.partition("=")
        if separator:
            fields[key] = value
    return fields


def read_executor_output(path: str) -> ExecutorReading:
    """执行器产物（每行 `name=… key=value …`，不带 `E7RESULT `）。`name=state`：严格与暂代两个判定、摘要对不对；
    `name=base_recovery … recovered=false`：这条流底镜像恢复不出，执行器自己说「这条流不判」（读法 7：O3 两种跑法记弃权、不红）。"""
    reading = ExecutorReading()
    with open(path, encoding="utf-8") as handle:
        for raw in handle:
            line = strip_result_prefix(raw)
            fields = key_value_fields(line)
            name = fields.get("name")
            if name == "state":
                key = (fields["stream"], int(fields["state"]))
                reading.verdicts[key] = (fields["strict"], fields["provisional"])
                reading.state_lines += 1
                reading.error_states += fields["strict"] == VERDICT_ERROR or fields["provisional"] == VERDICT_ERROR
                reading.digest_mismatch_states += fields.get("digest") == "mismatch"
                reading.in_process_state_ns += int(fields.get("time_ns", "0"))
            elif name == "base_recovery" and fields.get("recovered") == "false":
                reading.base_unrecoverable_streams.append(fields["stream"])
            elif name == "dump_problem":
                reading.dump_problems += 1
            elif line.startswith(FREEZE_PREFIXES):
                reading.freeze_lines.append(line)
            elif name == "complete":
                reading.complete = True
    return reading


def executor_command(repo_root: str, dumps: str, streams: list[str], batches: str, workers: int, output: str, memory_cap: str) -> list[str]:
    """登记 5.4 的命令：`python3 executor_main.py --dumps <导出> --batches 1,2 --workers N --output <新文件>`，经 `capped.sh` 与 `run-with-memory-cap.sh`。
    `--streams` 只列这一趟导出了的流（没导出的流没有文件可读）。"""
    script = os.path.join(repo_root, FROZEN_DIRECTORY_RELATIVE, EXECUTOR_SCRIPT)
    return ["bash", os.path.join(repo_root, "research/scripts/run-with-memory-cap.sh"), memory_cap, "bash",
            os.path.join(repo_root, "research/scripts/capped.sh"), str(workers), "nice", "-n", "19", "python3", script,
            "--dumps", dumps, "--streams", ",".join(streams), "--batches", batches, "--workers", str(workers), "--output", output]


def run_executor(repo_root: str, dumps: str, streams: list[str], workers: int, output: str, memory_cap: str, log_path: str) -> tuple[int, int]:
    """起冻结的执行器，交回 (退出码, 墙钟 ns)。墙钟在这个进程里用 `time.monotonic_ns` 计；每态耗时在执行器里自己计（`time_ns`）。不写 `.pyc`（冻结目录里不留东西）。"""
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", RUN_WITH_MEMORY_CAP_PARALLEL_REASON=PARALLEL_REASON,
                       RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="执行器峰值表里没有这条命令，上限按派发提示的写法给",
                       RUN_WITH_MEMORY_CAP_WAIT_SECONDS="1800")
    started = time.monotonic_ns()
    with open(log_path, "w", encoding="utf-8") as log:
        # 执行器在冻结目录里起（cwd 是那里），所以导出与产物都要给绝对路径
        completed = subprocess.run(executor_command(repo_root, os.path.abspath(dumps), streams, "1,2", workers, os.path.abspath(output), memory_cap),
                                   cwd=os.path.join(repo_root, FROZEN_DIRECTORY_RELATIVE), env=environment, stdout=log, stderr=subprocess.STDOUT)
    return completed.returncode, time.monotonic_ns() - started
