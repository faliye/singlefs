"""E171 O3 一侧：冻结的执行器版本核对、起执行器、读执行器的判定行（登记第一节读法 7、8，5.4，作废条款 Z2、Z4）。

冻结的那一版是 E170 第 1 次重跑（`research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out`）冻结的执行器目录
`scripts/experiments/e170_second_run_o3_spec_executor/`：产物里 `name=freeze file=… sha256=…` 每个文件一行（不含 `comparator*`
与 `mutation_driver*`），最后一行 `name=freeze_seed sha256_prefix16=…` 是盲测族的种子。本实验不改那一份，只读、只起它。

那份参照产物在 research/results/ 下、可以被归档删掉（用户 2026-10-18 定：别的实验的产物缺了报本次未跑）：读不到时
`freeze_reference_lines` 抛 `FreezeReferenceMissing`；comparator.py、o3_bad_image_judge.py（judge）、row_driver.py 开跑前经
`report_missing_freeze_reference` 打一行 `NOT_RUN_ARCHIVED_PRODUCT test=<哪一个> reason=…`、退 77，一行产物都不写；单测 test_o3_columns 报同一行后跳过。
那一份搬了家、只改了路径的文件（`.claude/rules/path-moves.md`「怎么做」第 6 步：路径不在冻结范围内），逐个登记在同目录
`freeze-path-only-changes.tsv`（写法同 `scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes/` 那一张）：登记了、冻结时与现在的两个值
都对上的不算不同；登记之后又改了的照判不同（test_o3_columns 钉住）。
"""

from __future__ import annotations

import hashlib
import os
import subprocess
import sys
import time
from dataclasses import dataclass, field

FROZEN_DIRECTORY_RELATIVE = "scripts/experiments/e170_second_run_o3_spec_executor"
FREEZE_REFERENCE_RELATIVE = "research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out"
FILES_OUTSIDE_THE_FREEZE = ("comparator", "mutation_driver")
FREEZE_PREFIXES = ("name=freeze ", "name=freeze_seed ")
FREEZE_FILE_PREFIX = "name=freeze file="
FREEZE_SEED_PREFIX = "name=freeze_seed "
EXECUTOR_SCRIPT = "executor_main.py"
VERDICT_ERROR = "error"
PARALLEL_REASON = "同一会话派出去的几个执行员各跑各的装置"
# 本次未跑那一行的记号与退出码（写法照 scripts/experiments/lib.rs 的 ARCHIVED_PRODUCT_NOT_RUN_MARKER、NOT_RUN_EXIT_CODE）
NOT_RUN_MARKER = "NOT_RUN_ARCHIVED_PRODUCT"
NOT_RUN_EXIT_CODE = 77
PATH_ONLY_CHANGES_TABLE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "freeze-path-only-changes.tsv")
PATH_ONLY_CHANGES_COLUMNS = 5
SEED_NAME = "freeze_seed"


class FreezeReferenceMissing(Exception):
    """冻结行的参照产物（FREEZE_REFERENCE_RELATIVE）读不到：冻结判不了，这一项本次未跑。"""

    def not_run_line(self, test_name: str) -> str:
        return f"{NOT_RUN_MARKER} test={test_name} reason={self}"


def strip_result_prefix(line: str) -> str:
    return line.removeprefix("E7RESULT ").rstrip("\n")


def frozen_lines_of(path: str) -> list[str]:
    """一份产物里的冻结行（文件行与种子行，允许前面带 `E7RESULT `），按原次序。"""
    with open(path, encoding="utf-8") as handle:
        lines = [strip_result_prefix(line) for line in handle]
    return [line for line in lines if line.startswith(FREEZE_PREFIXES)]


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


def freeze_reference_lines(repo_root: str) -> list[str]:
    """参照产物里的冻结行；那份产物不在抛 FreezeReferenceMissing（说明里写怎么取回）。"""
    path = os.path.join(repo_root, FREEZE_REFERENCE_RELATIVE)
    if not os.path.isfile(path):
        name = os.path.basename(FREEZE_REFERENCE_RELATIVE)
        raise FreezeReferenceMissing(f"{FREEZE_REFERENCE_RELATIVE} 不在，冻结行判不了，这一项本次未跑；要跑先从 git 历史取回："
                                     f"git log --all --diff-filter=D --name-only -- \"*{name}\"")
    return frozen_lines_of(path)


def report_missing_freeze_reference(repo_root: str, test_name: str) -> bool:
    """开跑前调：参照产物不在就往 stderr 打一行本次未跑、交 True（调用方退 NOT_RUN_EXIT_CODE，一行产物都不写）；在交 False。"""
    try:
        freeze_reference_lines(repo_root)
    except FreezeReferenceMissing as missing:
        print(missing.not_run_line(test_name), file=sys.stderr)
        return True
    return False


def path_only_changes_of(path: str = PATH_ONLY_CHANGES_TABLE) -> dict[str, tuple[str, str]]:
    """只差路径的重冻结登记 → {名字: (冻结行里的值, 现在的值)}。一行五列，制表符隔开：名字（冻结范围里的文件名，或 freeze_seed）、
    冻结行里的值、现在的值、改它的提交、改了什么；空行与 `#` 起头的行跳过。表不在就是空登记；列数不对抛错，不猜。"""
    changes: dict[str, tuple[str, str]] = {}
    if not os.path.exists(path):
        return changes
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            if not line.strip() or line.startswith("#"):
                continue
            fields = line.rstrip("\n").split("\t")
            if len(fields) != PATH_ONLY_CHANGES_COLUMNS:
                raise ValueError(f"{path}：一行要 {PATH_ONLY_CHANGES_COLUMNS} 列（名字、冻结时、现在、提交、改了什么），这一行 {len(fields)} 列：{line!r}")
            changes[fields[0]] = (fields[1], fields[2])
    return changes


def freeze_differences(repo_root: str, reference_lines: list[str] | None = None,
                       path_only_changes: dict[str, tuple[str, str]] | None = None) -> list[str]:
    """Z2：冻结行逐个核（`sha256sum -c` 同义）：文件缺失、摘要不同、目录里有冻结范围之内而冻结行里没有的文件、种子不等，各记一条。空表示都对。
    不给参照时读参照产物，它不在抛 FreezeReferenceMissing。摘要与种子不同、而 `path_only_changes`（不给就读登记表）里那一名的
    （冻结时, 现在）正是这一对的，不记。"""
    directory = os.path.join(repo_root, FROZEN_DIRECTORY_RELATIVE)
    lines = reference_lines if reference_lines is not None else freeze_reference_lines(repo_root)
    changes = path_only_changes_of() if path_only_changes is None else path_only_changes
    hashes = frozen_hashes_of(lines)
    differences = []
    for name, expected in hashes.items():
        path = os.path.join(directory, name)
        if not os.path.exists(path):
            differences.append(f"{name}:missing")
        elif sha256_of_file(path) != expected and changes.get(name) != (expected, sha256_of_file(path)):
            differences.append(f"{name}:sha256_differs")
    differences.extend(f"{name}:not_in_the_freeze" for name in frozen_files_in(directory) if name not in hashes)
    if seed_from_directory(directory) != seed_of(lines):
        if changes.get(SEED_NAME) != (seed_of(lines), seed_from_directory(directory)):
            differences.append(f"{SEED_NAME}:differs")
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
    return ["bash", os.path.join(repo_root, "scripts/hosts/run-with-memory-cap.sh"), memory_cap, "bash",
            os.path.join(repo_root, "scripts/hosts/capped.sh"), str(workers), "nice", "-n", "19", "python3", script,
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
