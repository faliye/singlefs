"""E171 O3 一侧：冻结的执行器版本核对、起执行器、读执行器的判定行（登记第一节读法 7、8，5.4，作废条款 Z2、Z4）。

冻结的那一版是 E170 第 1 次重跑（`research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out`）冻结的执行器目录
`scripts/experiment-apparatus/e170_second_run_o3_spec_executor/`：产物里 `name=freeze file=… sha256=…` 每个文件一行（不含 `comparator*`
与 `mutation_driver*`），最后一行 `name=freeze_seed sha256_prefix16=…` 是盲测族的种子。本实验不改那一份，只读、只起它。

那份参照产物在 research/results/ 下、可以被归档删掉（用户 2026-10-18 定：别的实验的产物缺了报本次未跑）：读不到时
`freeze_reference_lines` 抛 `FreezeReferenceMissing`；comparator.py、o3_bad_image_judge.py（judge）、row_driver.py 开跑前经
`report_missing_freeze_reference` 打一行 `NOT_RUN_ARCHIVED_PRODUCT test=<哪一个> reason=…`、退 77，一行产物都不写；单测 test_o3_columns 报同一行后跳过。
那一份搬了家、只改了路径的文件（`.claude/rules/path-moves.md`「怎么做」第 6 步：路径不在冻结范围内），逐个登记在同目录
`freeze-path-only-changes.tsv`（写法同 `scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/` 那一张）：登记了、冻结时与现在的两个值
都对上的不算不同；登记之后又改了的照判不同（test_o3_columns 钉住）。

搬迁登记（`.claude/gate.d/path-moves.tsv`，读法在 `.claude/gate.d/lib-lag-baseline.py` 文件头「搬迁登记」）：目录整个搬了家、改了名的，
不往上面那张表逐个加行。`freeze_differences` 比摘要与种子时，除了此刻的内容，另把此刻内容里登记过的新前缀倒着换回旧前缀再算
（`RegisteredMoves.contents_before_moves`，换法就是共用库的 `rewrite_moved_bytes`）：从最近一次搬迁起逐次多倒一次（倒最近一次、倒最近两次……倒全部），
每一版都比，哪一版是冻结时的值、或正是上面那张表登记的「现在」那一个，不算不同（钉在两次搬迁之间的也认）。
倒换之前先把此刻的内容照全部登记正着换一遍，要一个字节不动（不留登记过的旧前缀，也不留串起来的中间那一段前缀）；动了的一版都不倒、只比此刻的内容（只换了一半的不认）。
取的是「把此刻的内容换回去」这一种，不是「把冻结那一版正着换」：冻结那一版的内容这里没有（只有它的 sha256）。
登记表不在时不载共用库、一个字节不换，与没有这一节时同判；登记表写坏了抛 ValueError，不猜。
这一节与 E171 第 1 次重跑那一份 frozen_o3.py 的同名一节同一种写法（两个实验各留一份，不互相 import）。
弄坏开关只在自证里认：这个文件是库、没有命令行入口，命令行直接起它时 FROZEN_O3_BREAK 非空就退 2、点名它只在自证里认；被 import（自证的子进程、reuse_library.py 按路径载入）时不看这一道。
弄坏开关（逐项写开关名与值、弄坏哪一处，下一行起写打开后红的项）：
  FROZEN_O3_BREAK=path-moves-ignored  不读搬迁登记（只会多判不同，平常设了也放不过什么）
      test_o3_columns 的 `test_a_registered_directory_move_with_only_path_differences_is_not_a_difference`
  FROZEN_O3_BREAK=partial-accepted  倒换之前不核此刻的内容正着换一遍动不动（只换了一半的也倒、也认）
      E171 第 1 次重跑那一份 reuse_library.py 自证的 frozen-o3-reverse-moves 格（它按路径载入这一份一起判，弄坏开关 frozen-o3-partial-accepted 在格里设这一项）
  FROZEN_O3_BREAK=whole-chain-only  只倒全部登记那一版、不逐次倒（钉在两次搬迁之间的不认）
      同一格（弄坏开关 frozen-o3-whole-chain-only）
"""

from __future__ import annotations

import hashlib
import importlib.util
import os
import subprocess
import sys
import time
import types
from dataclasses import dataclass, field

FROZEN_DIRECTORY_RELATIVE = "scripts/experiment-apparatus/e170_second_run_o3_spec_executor"
FREEZE_REFERENCE_RELATIVE = "research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out"
FILES_OUTSIDE_THE_FREEZE = ("comparator", "mutation_driver")
FREEZE_PREFIXES = ("name=freeze ", "name=freeze_seed ")
FREEZE_FILE_PREFIX = "name=freeze file="
FREEZE_SEED_PREFIX = "name=freeze_seed "
EXECUTOR_SCRIPT = "executor_main.py"
VERDICT_ERROR = "error"
PARALLEL_REASON = "同一会话派出去的几个执行员各跑各的装置"
# 本次未跑那一行的记号与退出码（写法照 scripts/experiment-apparatus/lib.rs 的 ARCHIVED_PRODUCT_NOT_RUN_MARKER、NOT_RUN_EXIT_CODE）
NOT_RUN_MARKER = "NOT_RUN_ARCHIVED_PRODUCT"
NOT_RUN_EXIT_CODE = 77
PATH_ONLY_CHANGES_TABLE = os.path.join(os.path.dirname(os.path.abspath(__file__)), "freeze-path-only-changes.tsv")
PATH_ONLY_CHANGES_COLUMNS = 5
SEED_NAME = "freeze_seed"
PATH_MOVES_TABLE_RELATIVE = os.path.join(".claude", "gate.d", "path-moves.tsv")
PATH_MOVES_LIBRARY_RELATIVE = os.path.join(".claude", "gate.d", "lib-lag-baseline.py")
OWN_REPOSITORY_ROOT = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))
BREAK_VARIABLE = "FROZEN_O3_BREAK"
_path_moves_libraries: dict[str, types.ModuleType] = {}


@dataclass(frozen=True)
class RegisteredMoves:
    """被判的仓里登记过的搬迁（共用库的 PathMove，按登记的次序）与换算它的那份共用库；moves 空的什么都不换。"""

    moves: tuple = ()
    library: types.ModuleType | None = None

    def contents_before_moves(self, data: bytes) -> list[bytes]:
        """把此刻内容里登记过的新前缀倒着换回旧前缀，从最近一次搬迁起逐次多倒一次（倒最近一次、倒最近两次……倒全部），按这个次序交几版；moves 空的交 []。
        此刻的内容照全部登记正着换一遍要一个字节不动，动了的交 []（只换了一半的不倒）。弄坏开关 partial-accepted、whole-chain-only 见文件头。"""
        if not self.moves:
            return []
        if not broken("partial-accepted") and self.library.rewrite_moved_bytes(data, list(self.moves)) != data:
            return []
        counts = [len(self.moves)] if broken("whole-chain-only") else range(1, len(self.moves) + 1)
        return [self.library.rewrite_moved_bytes(data, [move._replace(old=move.new, new=move.old) for move in reversed(self.moves[-count:])])
                for count in counts]

    def content_before_moves(self, data: bytes) -> bytes:
        """倒全部登记那一版（contents_before_moves 的最后一版）；倒不了的（moves 空、只换了一半）原样交回。"""
        versions = self.contents_before_moves(data)
        return versions[-1] if versions else data


def broken(item: str) -> bool:
    """弄坏开关 FROZEN_O3_BREAK（逗号连几项）里有没有 item；每次现读。"""
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


def path_moves_library(repo_root: str) -> types.ModuleType:
    """读搬迁登记的共用库 lib-lag-baseline.py：先取被判的仓里那一份，没有再取本文件所在仓的（单测的假仓里没有它）；按路径载入、不写 .pyc。"""
    candidates = [os.path.join(root, PATH_MOVES_LIBRARY_RELATIVE) for root in (repo_root, OWN_REPOSITORY_ROOT)]
    found = [path for path in candidates if os.path.isfile(path)]
    if not found:
        raise ValueError(f"{PATH_MOVES_TABLE_RELATIVE} 在，读它的 {PATH_MOVES_LIBRARY_RELATIVE} 却不在（找过 {'、'.join(candidates)}），搬迁登记换算不了")
    path = os.path.realpath(found[0])
    if path not in _path_moves_libraries:
        sys.dont_write_bytecode = True
        specification = importlib.util.spec_from_file_location("lib_lag_baseline_for_frozen_o3", path)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _path_moves_libraries[path] = module
    return _path_moves_libraries[path]


def registered_moves(repo_root: str) -> RegisteredMoves:
    """被判的仓（repo_root）里 .claude/gate.d/path-moves.tsv 登记的搬迁。表不在交空的、不载共用库；表写坏了抛 ValueError。
    弄坏开关 FROZEN_O3_BREAK=path-moves-ignored 下交空的。"""
    if "path-moves-ignored" in os.environ.get(BREAK_VARIABLE, "").split(","):
        return RegisteredMoves()
    table = os.path.join(repo_root, PATH_MOVES_TABLE_RELATIVE)
    if not os.path.isfile(table):
        return RegisteredMoves()
    library = path_moves_library(repo_root)
    moves, problems = library.path_moves(repo_root)
    if problems:
        raise ValueError(f"{table} 写得不对，搬迁换算不了：{'；'.join(problems)}")
    return RegisteredMoves(tuple(moves), library)


def sha256_before_moves(path: str, moves: RegisteredMoves) -> tuple[str, ...]:
    """path 的内容逐次倒换回搬迁之前的几版（contents_before_moves）各自的 sha256；moves 空的、只换了一半的交空的。"""
    with open(path, "rb") as handle:
        return tuple(hashlib.sha256(version).hexdigest() for version in moves.contents_before_moves(handle.read()))


def matches_the_freeze(frozen: str | None, now: str, before_moves: str | tuple[str, ...], registered_pair: tuple[str, str] | None) -> bool:
    """此刻的值，或换回搬迁之前的任何一版的值（before_moves 给一个或几个），是冻结时的值、或正是只差路径的登记那一对（冻结时, 现在）里的「现在」。"""
    candidates = (before_moves,) if isinstance(before_moves, str) else tuple(before_moves)
    return any(value == frozen or registered_pair == (frozen, value) for value in (now, *candidates))


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


def seed_from_directory(directory: str, moves: RegisteredMoves = RegisteredMoves()) -> str:
    """执行器自己（`executor_main.freeze_lines`）怎么算的这里照算一遍：按文件名排序，内容逐个喂进 sha256，取前 16 个十六进制字符。
    给了登记的搬迁时，每个文件的内容先倒着换回搬迁之前再喂（不给就是此刻的内容）。"""
    hasher = hashlib.sha256()
    for name in frozen_files_in(directory):
        with open(os.path.join(directory, name), "rb") as handle:
            hasher.update(moves.content_before_moves(handle.read()))
    return hasher.hexdigest()[:16]


def seeds_before_moves(directory: str, moves: RegisteredMoves) -> tuple[str, ...]:
    """目录里每个文件逐次倒换回搬迁之前的几版（contents_before_moves），同一档拼起来照 seed_from_directory 算种子，交几档种子；
    moves 空的、有一个文件只换了一半（倒不了）的交空的。"""
    versions = []
    for name in frozen_files_in(directory):
        with open(os.path.join(directory, name), "rb") as handle:
            versions.append(moves.contents_before_moves(handle.read()))
    if not moves.moves or any(not file_versions for file_versions in versions):
        return ()
    seeds = []
    for step in range(len(versions[0]) if versions else 0):
        hasher = hashlib.sha256()
        for file_versions in versions:
            hasher.update(file_versions[step])
        seeds.append(hasher.hexdigest()[:16])
    return tuple(seeds)


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
                       path_only_changes: dict[str, tuple[str, str]] | None = None, moves: RegisteredMoves | None = None) -> list[str]:
    """Z2：冻结行逐个核（`sha256sum -c` 同义）：文件缺失、摘要不同、目录里有冻结范围之内而冻结行里没有的文件、种子不等，各记一条。空表示都对。
    不给参照时读参照产物，它不在抛 FreezeReferenceMissing。摘要与种子不同、而 `path_only_changes`（不给就读登记表）里那一名的
    （冻结时, 现在）正是这一对的，不记；此刻的内容把 `moves`（不给就读 repo_root 里的搬迁登记）逐次倒着换回去的每一版再比，
    对上冻结时的值或登记的那一对的，同样不记（文件头「搬迁登记」）。"""
    directory = os.path.join(repo_root, FROZEN_DIRECTORY_RELATIVE)
    lines = reference_lines if reference_lines is not None else freeze_reference_lines(repo_root)
    changes = path_only_changes_of() if path_only_changes is None else path_only_changes
    registered = registered_moves(repo_root) if moves is None else moves
    hashes = frozen_hashes_of(lines)
    differences = []
    for name, expected in hashes.items():
        path = os.path.join(directory, name)
        if not os.path.exists(path):
            differences.append(f"{name}:missing")
        elif not matches_the_freeze(expected, sha256_of_file(path), sha256_before_moves(path, registered), changes.get(name)):
            differences.append(f"{name}:sha256_differs")
    differences.extend(f"{name}:not_in_the_freeze" for name in frozen_files_in(directory) if name not in hashes)
    if seed_from_directory(directory) != seed_of(lines):
        if not matches_the_freeze(seed_of(lines), seed_from_directory(directory), seeds_before_moves(directory, registered), changes.get(SEED_NAME)):
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


if __name__ == "__main__" and os.environ.get(BREAK_VARIABLE):
    print(f"  ✗ {BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 是弄坏开关，只在自证里认；这个文件是库、没有命令行入口，平常跑设了就拒，这一次什么都没做", file=sys.stderr)
    print(f"     → 怎么办：unset {BREAK_VARIABLE} 再来；要看开关红在哪一格，看文件头「弄坏开关」每一项后面点名的自证（test_o3_columns、reuse_library.py --selftest）", file=sys.stderr)
    sys.exit(2)
