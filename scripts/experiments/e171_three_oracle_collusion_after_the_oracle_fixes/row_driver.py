"""E171 第 1 次重跑的逐行驱动（第 1 次重跑登记 5.3「研究一侧」）：由原目录 `row_driver.py` 照抄起头，改四处——装置换成
`e171_three_oracle_collusion_after_the_oracle_fixes`；行从两张表里按整表编号取（第 1–117 行与第 118–281 行，登记第一节，`--also-selected-rows`
默认就是扩表那一张）；透传 `--streams` 与 `--writable-mount-probe`；不给 `--keep-dumps` 时执行器判完删掉每条流导出里的三个大文件
（`base-sectors.bin`、`ops-payload.bin`、`torn-payload.bin`，判定表与 manifest 留着；登记 5.3「第二段的行导出判完即删」）。
冻结树照登记读法 10 用 `--snapshot-from scripts/experiments/target/e171-r1-frozen-tree`。登记修订 2 第 2.5、2.6 节又改两处：每行装置那一步的限时由
`--baseline-timing <基线那一趟的 -timing 产物>` 现算（max(1800, ⌈3 × T⌉) 秒，`--rerun-of-timed-out-rows` 再乘 3），驱动开头打一行 `name=row_time_limit`；
装置套 `device_timing.py run`，每行 `device.out` 末尾多一行 `process_wall`，`--timing-output <新文件>` 收各行的 `E7TIMING` 行。以下是原驱动的说明，仍然适用
（原说明里「限时 1800 秒」那一处照这一段读）。

原说明：E171 逐行驱动（登记 5.3「研究一侧」、5.7）：坏实现表的每一行——拷仓、改坏、编、跑装置（`row` 模式）、跑冻结的 O3 执行器、留下导出目录与一份 `row-result.tsv`。

跑法（在仓根，经 `scripts/hosts/capped.sh`；装置与执行器本身经 `run-with-memory-cap.sh`）：
  python3 row_driver.py --repo-root <仓根> --work-root <草稿目录> --output-root <输出根> --rows 1,6 [--empty-mutation]
      [--threads <每线道，缺省 = 登记的重活默认并行度 ÷ --lanes>] [--memory-cap 4G] [--lanes 1] [--positive-control O1-对拍|O1-崩溃判官|O2|O3] [--disable-oracle O1|O2]
      [--no-executor] [--o3-control-bad-states <E170 bad-states 的导出目录>] [--skip-finished] [--snapshot-from <冻住的树>]
      [--selected-rows <表>] [--other-selected-rows <别的表> …]
F4 扩表（登记 5.2「扩表」）：`--selected-rows bad-implementations-expansion-164.tsv --other-selected-rows bad-implementations-117.tsv --all`；
补位跳过两张表里的每一个名字（「扩表与补位都不重挑已跑过的行」）。
一次跑里先把仓（或 `--snapshot-from` 给的树）拍一张快照（`Cargo.toml`、`Cargo.lock`、`.cargo`、`crates/`，不带 target），每条线道从快照同步出自己的树与编译目录（线道之间不共享，
同一条线道里跨行复用编译目录，所以第一行是冷编译、之后是增量），每行改一处、编、跑、再把改过的文件还原。
增量编译的前提（登记修订 5）：cargo 只在一个包的源文件修改时间晚于上一次编它的时刻时重编；还原（`copy2`）与同步（`rsync -a`）都把修改时间带回快照那一刻，
所以每条线道记一份 `build-record.tsv`（上一次编完的时刻、自那以后改过还没干净重编的文件），每行同步之后先把那几个文件的修改时间置成现在，编之前核它们内容与快照相同、
修改时间晚于上一次编完，编完核 cargo 报的这些文件所在的包都重编了（`fresh=false`）；三样任一不过是测量管子的缺陷，这一行不跑装置、各线道不再领新行、驱动退 5。
工作根里已有编译目录而没有这份记录（旧驱动建的），或快照与上一次在这个工作根里跑的不同，驱动一行都不跑、退 2：换一个新的 `--work-root`。
表里的行照名字取（登记 5.2：表若已变不重挑），
行文与登记 5.9 的 sha256 不同照今天的行跑、另记。编不过、锚点不唯一、找不到的行按补位规则换同一文件组里的下一行（编之前的失败与编不过合起来至多三次；
换上的行编不过再接着换），产物里带「补位自 <原名>」（`substituted_from`），试过而没用上的每个名字记成 `failed_attempt_<序>=<失败>:<名字>`。
`--rerun-compile-failed-from <输出根> …`：只跑那几个输出根里结局是编不过的行（补位那一趟，登记第一节读法 5），行从 `--selected-rows` 与
`--other-selected-rows` 几张表里按序号取，各行照常从表里的原名起编（编不过就照上面的规则补位）；输出根要换新的。
每行一个目录 `<输出根>/row-<序号>/`：`row-result.tsv`（key<TAB>value）、`device.out`、`device.err`、`compile.log`（最后一次编的诊断）、
`compile-failed-<序>.log`（编不过的那几次的尾部）、`dump/`（装置导出目录）、
`o3.out`（冻结的执行器对这一趟导出的批 1、批 2 的判定，登记 5.4）、`o3.log`、`o3-control.out`（`--positive-control O3` 时）。
O3 一步：开跑前核一次冻结行（Z2，不过整趟不跑）；每行装置退 0、导出了至少一条流，就起冻结的执行器（只列导出了的流）；`--no-executor` 是 Q3-核「驱动不跑执行器」那一次。
线道从一条共用的队列里领行（先到先得），每条线道一次一行。`--skip-finished`：输出根里已有 `row-result.tsv` 的行目录当作做完了、不重跑；没有 `row-result.tsv` 的行目录
（上一次中途停下的）由驱动删掉重做。耗时在这个进程里用 `time.monotonic_ns` 计：拷仓与改坏、编、装置、执行器各一段，整表一段（`table` 行）；装置自己的分段耗时在它的 `E7TIMING` 行里。
"""

from __future__ import annotations

import argparse
import dataclasses
import hashlib
import json
import os
import queue
import shutil
import subprocess
import sys
import threading
import time

import device_timing
import frozen_o3
from run_directory import STREAM_NAMES
from bad_implementation_table import (MAXIMUM_SUBSTITUTIONS_PER_ROW, STATUS_ANCHOR_NOT_UNIQUE, STATUS_LOCATED, STATUS_NOT_FOUND,
                                       Located, MutationRow, SelectedRow, anchor_hit_count, apply_mutation, empty_mutation,
                                       group_names_of_file, locate_by_name, read_mutation_rows, read_selected_rows,
                                       substitute_for)

HERE = os.path.dirname(os.path.abspath(__file__))
DEVICE_BINARY_NAME = "e171_three_oracle_collusion_after_the_oracle_fixes"
# 不给 `--keep-dumps` 时判完删掉的大文件（每条流导出目录里的三个）
BULKY_DUMP_FILES = ("base-sectors.bin", "ops-payload.bin", "torn-payload.bin")
DEVICE_POSITIVE_CONTROLS = ("O1-对拍", "O1-对拍-卸载", "O1-崩溃判官", "RefFS", "O2", "交回值", "内存对象", "panic", "见证")
DEVICE_PACKAGE = "singlefs-checker-tier"
SYNCED_ENTRIES = ("Cargo.toml", "Cargo.lock", ".cargo", "crates")
MUTATIONS_TABLE = "crates/mutations.tsv"
# 每行装置那一步的限时（不含编译，不含 O3 执行器）由命令行给的基线 `-timing` 产物现算（登记修订 2 第 2.5 节，`device_timing.row_time_limit_seconds`）
QUEUE_WAIT_SECONDS = 1800
DEVICE_TIMING_SCRIPT = os.path.join(HERE, "device_timing.py")
MAXIMUM_DEVICE_ATTEMPTS = 2
WRAPPER_OWN_EXIT_CODES = (250, 251, 252, 254)  # 253 是超时，登记第一节读法 5 单列、不重跑
PARALLEL_REASON = "同一会话派出去的几个执行员各跑各的装置"
STATUS_OK = "ok"
STATUS_COMPILE_FAILED = "compile_failed"
COMPILE_LOG_TAIL_LINES = 60
O3_STATUS_RAN = "ran"
O3_STATUS_SKIPPED_BY_REQUEST = "skipped_by_request"
O3_STATUS_NO_EXPORTED_STREAM = "no_exported_stream"
O3_STATUS_EXECUTOR_FAILED = "executor_failed"
O3_STATUS_NOT_REACHED = "not_reached"
O3_CONTROL = "O3"
O3_CONTROL_ROW_NAME = "P1-F"
IMAGE_JUDGE_SCRIPT = os.path.join(HERE, "o3_bad_image_judge.py")
SUBSTITUTION_LOCK = threading.Lock()
LANE_BUILD_RECORD_FILE = "build-record.tsv"
LANE_COMPILE_ARTIFACTS_FILE = "compile-artifacts.jsonl"
SNAPSHOT_DIGEST_FILE = "snapshot-digest"
STATUS_MEASUREMENT_PIPE_DEFECT = "measurement_pipe_defect"
REBUILD_CHECK_FIELD = "packages_checked_rebuilt"  # row-result.tsv 里的键：编完核过重编了的包；编过的行没有这个键，就是没核过（旧驱动跑的）
EXIT_WORK_ROOT_REFUSED = 2
EXIT_MEASUREMENT_PIPE_DEFECT = 5


class MeasurementPipeDefect(Exception):
    """测量管子自己的缺陷：这一行编出来的装置可能带着同一条线道上别的行的变异。`stage` 是哪一道核对没过，`details` 是没过的文件或包。"""

    def __init__(self, stage: str, details: list[str]) -> None:
        super().__init__(f"{stage}:{','.join(details)}")
        self.stage = stage
        self.details = details


@dataclasses.dataclass(frozen=True)
class LaneBuildRecord:
    """一条线道的编译记录：上一次 cargo 编完的时刻（`time.time_ns`，墙上时间，与文件修改时间同一个钟），与自那以后这条线道上改过、还没有一次
    编过并核过「重编了」的文件（相对树根的路径）。编不过或没核过的那一次之后，文件留在记录里。"""

    last_compile_finished_ns: int
    files_to_rebuild: tuple[str, ...]


def now_ns() -> int:
    return time.monotonic_ns()


def read_lane_build_record(lane_directory: str) -> LaneBuildRecord | None:
    path = os.path.join(lane_directory, LANE_BUILD_RECORD_FILE)
    if not os.path.exists(path):
        return None
    finished_ns, files = 0, []
    with open(path, encoding="utf-8") as handle:
        for line in handle:
            key, value = line.rstrip("\n").split("\t", 1)
            if key == "last_compile_finished_ns":
                finished_ns = int(value)
            elif key == "file_to_rebuild":
                files.append(value)
            else:
                raise ValueError(f"{path} 里有认不得的行：{line!r}")
    return LaneBuildRecord(finished_ns, tuple(files))


def write_lane_build_record(lane_directory: str, record: LaneBuildRecord) -> None:
    """先写临时文件再换上：驱动在写的时候被停，留下的仍是上一份完整的记录。"""
    os.makedirs(lane_directory, exist_ok=True)
    path = os.path.join(lane_directory, LANE_BUILD_RECORD_FILE)
    with open(path + ".partial", "w", encoding="utf-8") as handle:
        handle.write(f"last_compile_finished_ns\t{record.last_compile_finished_ns}\n")
        for file in record.files_to_rebuild:
            handle.write(f"file_to_rebuild\t{file}\n")
    os.replace(path + ".partial", path)


def mark_files_newer_than_every_build(lane_tree: str, relative_paths: tuple[str, ...]) -> None:
    """把修改时间置成现在（墙上时间、纳秒）：cargo 只在源文件的修改时间晚于上一次编那个包的时刻时重编它（cargo 1.98 实测，登记修订 5），
    而还原用的 `copy2` 与同步用的 `rsync -a` 都把修改时间带回快照那一刻——不置，上一行改过的包这一行被当成新的、留着上一行的变异。"""
    for relative_path in relative_paths:
        moment_ns = time.time_ns()
        os.utime(os.path.join(lane_tree, relative_path), ns=(moment_ns, moment_ns))


def files_that_would_not_be_rebuilt(lane_tree: str, snapshot_root: str, record: LaneBuildRecord) -> list[str]:
    """编之前的核对：记录里每个文件在线道树里的内容与快照逐字节相同，修改时间晚于上一次编完。交回没过的（`<文件>:<哪一样>`）。"""
    problems = []
    for relative_path in record.files_to_rebuild:
        lane_path = os.path.join(lane_tree, relative_path)
        with open(lane_path, "rb") as lane_handle, open(os.path.join(snapshot_root, relative_path), "rb") as snapshot_handle:
            if lane_handle.read() != snapshot_handle.read():
                problems.append(f"{relative_path}:content_differs_from_the_snapshot")
        if os.stat(lane_path).st_mtime_ns <= record.last_compile_finished_ns:
            problems.append(f"{relative_path}:not_newer_than_the_last_compile")
    return problems


def package_of_file(relative_path: str) -> str:
    """`crates/<包目录>/…` 的包目录（坏实现表只改 `crates/singlefs-core/src/` 与 `crates/singlefs-replay-prefix/src/lib.rs`）。"""
    parts = relative_path.split("/")
    if len(parts) < 3 or parts[0] != "crates":
        raise ValueError(f"不在 crates/<包>/ 下的文件：{relative_path}")
    return "/".join(parts[:2])


def library_freshness_by_package(artifacts_path: str, lane_tree: str) -> dict[str, bool]:
    """读 `cargo build --message-format=json-render-diagnostics` 的标准输出：每个包的库目标那一条 `compiler-artifact` 的 `fresh`
    （true = 这次没重编、用的是编译目录里已有的产物）。键是包目录相对树根的路径。"""
    freshness: dict[str, bool] = {}
    with open(artifacts_path, encoding="utf-8") as handle:
        for line in handle:
            if not line.startswith("{"):
                continue
            message = json.loads(line)
            if message.get("reason") != "compiler-artifact" or "lib" not in message.get("target", {}).get("kind", []):
                continue
            package = os.path.relpath(os.path.dirname(message["manifest_path"]), lane_tree)
            freshness[package] = bool(message["fresh"])
    return freshness


def packages_left_stale(freshness: dict[str, bool], relative_paths: tuple[str, ...]) -> list[str]:
    """编完的核对：这些文件所在的每个包，这一次 cargo 都重编了它的库（`fresh=false`）。交回没过的（`<包>:<哪一样>`）。"""
    problems = []
    for package in sorted({package_of_file(relative_path) for relative_path in relative_paths}):
        if package not in freshness:
            problems.append(f"{package}:no_library_artifact")
        elif freshness[package]:
            problems.append(f"{package}:fresh_not_rebuilt")
    return problems


def work_root_refusals(work_root: str, lanes: int, snapshot_digest_now: str) -> list[str]:
    """开跑前对工作根的两样核对：编过的线道（有编译目录）都有编译记录（没有就是旧驱动建的，还原过的文件没置过时间）；
    工作根上一次的快照摘要与这一次的相同（快照变了，变了的文件带着快照的旧修改时间进线道树，cargo 不重编）。交回拒跑的理由。"""
    refusals = []
    built_lanes = [lane for lane in range(lanes) if os.path.isdir(os.path.join(work_root, f"lane{lane}", "target"))]
    for lane in built_lanes:
        if read_lane_build_record(os.path.join(work_root, f"lane{lane}")) is None:
            refusals.append(f"lane{lane}:built_without_a_build_record")
    if built_lanes:
        digest_path = os.path.join(work_root, SNAPSHOT_DIGEST_FILE)
        if not os.path.exists(digest_path):
            refusals.append("snapshot_digest_missing")
        else:
            with open(digest_path, encoding="utf-8") as handle:
                previous_digest = handle.read().strip()
            if previous_digest != snapshot_digest_now:
                refusals.append(f"snapshot_changed:{previous_digest}->{snapshot_digest_now}")
    return refusals


def sync_tree(source_root: str, destination_root: str) -> None:
    """`rsync -a --delete`：源与目的逐项同步，不动 target；没变的文件保留修改时间，所以编译目录里没变的包不重编。"""
    os.makedirs(destination_root, exist_ok=True)
    for entry in SYNCED_ENTRIES:
        source = os.path.join(source_root, entry)
        if not os.path.exists(source):
            continue
        destination = os.path.join(destination_root, entry)
        if os.path.isdir(source):
            subprocess.run(["rsync", "-a", "--delete", "--exclude", "target", source + "/", destination + "/"], check=True)
        else:
            subprocess.run(["rsync", "-a", source, destination], check=True)


def snapshot_digest(root: str) -> tuple[str, int]:
    """快照里每个 `.rs` 与 `Cargo.*` 文件的 (相对路径, sha256) 排好序再算一个总摘要，连同文件数记进产物（行跑的是同一棵树的证据）。"""
    entries = []
    for directory, _, files in os.walk(root):
        for file_name in files:
            if file_name.endswith(".rs") or file_name.startswith("Cargo."):
                path = os.path.join(directory, file_name)
                with open(path, "rb") as handle:
                    entries.append((os.path.relpath(path, root), hashlib.sha256(handle.read()).hexdigest()))
    entries.sort()
    return hashlib.sha256("\n".join(f"{path} {digest}" for path, digest in entries).encode("utf-8")).hexdigest(), len(entries)


def write_result(directory: str, fields: dict[str, object]) -> None:
    with open(os.path.join(directory, "row-result.tsv"), "w", encoding="utf-8") as handle:
        for key, value in fields.items():
            handle.write(f"{key}\t{value}\n")


def run_compile(lane_tree: str, lane_target: str, repo_root: str, threads: int, log_path: str, artifacts_path: str) -> tuple[int, int]:
    """`cargo build --release --offline --message-format=json-render-diagnostics -p singlefs-checker-tier --bin e171_three_oracle_collusion`
    （只编不跑，不是重型测试）：诊断照常写进 `log_path`，每个编译单元的 JSON 消息（带 `fresh`）写进 `artifacts_path`。交回 (退出码, 耗时 ns)。"""
    environment = dict(os.environ, CARGO_TARGET_DIR=lane_target)
    command = ["bash", os.path.join(repo_root, "scripts/hosts/capped.sh"), str(threads), "nice", "-n", "19", "cargo", "build", "--release",
               "--offline", "--message-format=json-render-diagnostics", "-p", DEVICE_PACKAGE, "--bin", DEVICE_BINARY_NAME]
    started = now_ns()
    with open(log_path, "w", encoding="utf-8") as log, open(artifacts_path, "w", encoding="utf-8") as artifacts:
        completed = subprocess.run(command, cwd=lane_tree, env=environment, stdout=artifacts, stderr=log)
    return completed.returncode, now_ns() - started


def device_command(repo_root: str, binary: str, dump_directory: str, threads: int, memory_cap: str, extra: list[str]) -> list[str]:
    """装置在内存包装与线程上限里面、再套 `device_timing.py run`：进程起止的时刻在 scope 里面紧挨着装置取，`device.out` 末尾多一行
    `E7TIMING name=process_wall`（登记修订 2 第 2.6 节）。"""
    return ["bash", os.path.join(repo_root, "scripts/hosts/run-with-memory-cap.sh"), memory_cap, "bash",
            os.path.join(repo_root, "scripts/hosts/capped.sh"), str(threads), sys.executable, "-B", DEVICE_TIMING_SCRIPT, "run", "--",
            binary, "row", "--output-directory", dump_directory] + extra


def run_device(repo_root: str, binary: str, row_directory: str, threads: int, memory_cap: str, extra: list[str],
               time_limit_seconds: int) -> tuple[int, int]:
    """装置的 `row` 模式，经内存包装（限时 `time_limit_seconds`，登记第一节读法 5 的超时、修订 2 第 2.5 节现算；排队最多等 1800 秒）。交回 (最后一次的退出码, 总耗时 ns)。
    作废条款 Z3：包装自己的结局（退出码 250–254，超时 253 除外：那是登记第一节读法 5 的「超时」一类，不重跑）那一次的输出不算，清掉导出目录、重跑一次；
    两次都撞就把最后一次的退出码交回（比对器记进超时一类并单列）。第一次的退出码记进 `device.err` 开头一行 `first_attempt_exit=…`。"""
    environment = dict(os.environ, RUN_WITH_MEMORY_CAP_TIME_LIMIT=str(time_limit_seconds), RUN_WITH_MEMORY_CAP_PARALLEL_REASON=PARALLEL_REASON,
                       RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="逐行驱动统一给一个上限，同一个输出路径重跑时峰值表里已有这条命令",
                       RUN_WITH_MEMORY_CAP_WAIT_SECONDS=str(QUEUE_WAIT_SECONDS))
    dump_directory = os.path.join(row_directory, "dump")
    command = device_command(repo_root, binary, dump_directory, threads, memory_cap, extra)
    started = now_ns()
    returncode = 0
    for attempt in range(MAXIMUM_DEVICE_ATTEMPTS):
        with open(os.path.join(row_directory, "device.out"), "w", encoding="utf-8") as out, \
                open(os.path.join(row_directory, "device.err"), "a" if attempt else "w", encoding="utf-8") as err:
            if attempt:
                err.write(f"first_attempt_exit={returncode}\n")
            completed = subprocess.run(command, cwd=repo_root, env=environment, stdout=out, stderr=err)
        returncode = completed.returncode
        if returncode not in WRAPPER_OWN_EXIT_CODES:
            break
        shutil.rmtree(dump_directory, ignore_errors=True)
    return returncode, now_ns() - started


def exported_streams(dump_directory: str) -> list[str]:
    """装置导出了判定表的流（`oracle-verdicts.tsv` 在）；实现自崩没导出的流没有文件可读，不交给执行器。"""
    return [name for name in STREAM_NAMES if os.path.exists(os.path.join(dump_directory, name, "oracle-verdicts.tsv"))]


def run_image_control(arguments: argparse.Namespace, row_directory: str) -> tuple[int, int]:
    """O3 阳性对照（登记 5.6）：这一趟流一导出上，全持久态 F 叠 P1 补丁交冻结的执行器要判「不在」；补丁取 `--o3-control-bad-states` 那份 E170 导出。
    交回 (退出码, 耗时 ns)。"""
    environment = dict(os.environ, PYTHONDONTWRITEBYTECODE="1", RUN_WITH_MEMORY_CAP_PARALLEL_REASON=frozen_o3.PARALLEL_REASON,
                       RUN_WITH_MEMORY_CAP_OVER_PEAK_REASON="O3 阳性对照，峰值表里没有这条命令")
    command = ["bash", os.path.join(arguments.repo_root, "scripts/hosts/run-with-memory-cap.sh"), arguments.memory_cap, "bash",
               os.path.join(arguments.repo_root, "scripts/hosts/capped.sh"), str(arguments.threads), "nice", "-n", "19", sys.executable,
               IMAGE_JUDGE_SCRIPT, "control", "--repo-root", arguments.repo_root, "--dumps", os.path.join(row_directory, "dump"),
               "--e170-dumps", arguments.o3_control_bad_states, "--row-name", O3_CONTROL_ROW_NAME, "--output", os.path.join(row_directory, "o3-control.out")]
    started = now_ns()
    with open(os.path.join(row_directory, "o3-control.log"), "w", encoding="utf-8") as log:
        completed = subprocess.run(command, cwd=arguments.repo_root, env=environment, stdout=log, stderr=subprocess.STDOUT)
    return completed.returncode, now_ns() - started


def run_o3_step(arguments: argparse.Namespace, row_directory: str) -> dict[str, object]:
    """装置跑完之后的 O3 一步。交回要并进 `row-result.tsv` 的字段：状态、退出码、墙钟（驱动进程里 `monotonic_ns` 计）、导出了哪几条流。"""
    if arguments.no_executor:
        return {"o3_status": O3_STATUS_SKIPPED_BY_REQUEST, "o3_exit": "-", "o3_ns": "-", "o3_streams": "-"}
    dump_directory = os.path.join(row_directory, "dump")
    streams = exported_streams(dump_directory)
    if not streams:
        return {"o3_status": O3_STATUS_NO_EXPORTED_STREAM, "o3_exit": "-", "o3_ns": "-", "o3_streams": "-"}
    output = os.path.join(row_directory, "o3.out")
    exit_code, wall_ns = 0, 0
    for _ in range(MAXIMUM_DEVICE_ATTEMPTS):  # 包装自己的结局（Z3）：输出不算，清掉重跑一次
        exit_code, attempt_ns = frozen_o3.run_executor(arguments.repo_root, dump_directory, streams, arguments.threads, output, arguments.memory_cap,
                                                       os.path.join(row_directory, "o3.log"))
        wall_ns += attempt_ns
        if exit_code not in WRAPPER_OWN_EXIT_CODES + (253,):
            break
        if os.path.exists(output):
            os.remove(output)
    complete = os.path.exists(output) and frozen_o3.read_executor_output(output).complete
    fields: dict[str, object] = {"o3_status": O3_STATUS_RAN if exit_code == 0 and complete else O3_STATUS_EXECUTOR_FAILED, "o3_exit": exit_code,
                                 "o3_ns": wall_ns, "o3_streams": ",".join(streams)}
    if arguments.positive_control == O3_CONTROL:
        control_exit, control_ns = run_image_control(arguments, row_directory)
        fields.update({"o3_control_exit": control_exit, "o3_control_ns": control_ns})
    return fields


@dataclasses.dataclass(frozen=True)
class RowChoice:
    """一行这一次要跑的变异：`mutation` 是 None 时补不上。`substituted_from` 是表里那一行的原名（没补位是 `-`），`substitutions_used` 是这一行
    已经换过几次（编之前的失败与编不过合起来至多 `MAXIMUM_SUBSTITUTIONS_PER_ROW` 次），`failed_attempts` 是试过而没用上的每个名字与它的失败（按先后）。"""

    mutation: MutationRow | None
    status: str
    substituted_from: str
    line_matches_the_registration: bool
    substitutions_used: int
    failed_attempts: tuple[tuple[str, str], ...]


def prepare_row(selected: SelectedRow | None, table: list[MutationRow], rows_in_the_table: set[str], used_substitutes: set[str],
                snapshot_root: str, empty_of: MutationRow | None, start_name: str | None = None, substituted_from: str = "-",
                substitutions_used: int = 0, failed_attempts: tuple[tuple[str, str], ...] = ()) -> RowChoice:
    """把表里的一行变成要跑的变异：照名字取、核锚点；编之前的失败（找不到、名字不唯一、锚点不是恰好一次）按补位规则换行。
    编不过之后接着补位时由 `substitute_after_a_compile_failure` 带着已经换过的次数与试过的名字再进来（`start_name` 是换上的那一行）。"""
    if empty_of is not None:
        return RowChoice(empty_mutation(empty_of), STATUS_LOCATED, "-", True, 0, ())
    assert selected is not None
    current_name = start_name or selected.name
    attempts = list(failed_attempts)
    while True:
        located: Located = locate_by_name(SelectedRow(selected.number, selected.slice_number, selected.file, selected.group_index,
                                                      current_name, selected.line_digest_prefix), table)
        status = located.status
        if status == STATUS_LOCATED and located.row is not None:
            with open(os.path.join(snapshot_root, located.row.file), encoding="utf-8") as handle:
                hits = anchor_hit_count(handle.read(), located.row)
            if hits == 1:
                matches = located.line_matches_the_registration if substituted_from == "-" else False
                return RowChoice(located.row, STATUS_LOCATED, substituted_from, matches, substitutions_used, tuple(attempts))
            status = STATUS_ANCHOR_NOT_UNIQUE
        attempts.append((current_name, status))
        if substitutions_used == MAXIMUM_SUBSTITUTIONS_PER_ROW:
            return RowChoice(None, status, substituted_from, False, substitutions_used, tuple(attempts))
        substitute = substitute_for(group_names_of_file(table, selected.file), current_name, rows_in_the_table, used_substitutes)
        if substitute is None:
            return RowChoice(None, status, substituted_from, False, substitutions_used, tuple(attempts))
        used_substitutes.add(substitute)
        if substituted_from == "-":
            substituted_from = selected.name
        substitutions_used += 1
        current_name = substitute


def substitute_after_a_compile_failure(selected: SelectedRow | None, table: list[MutationRow], rows_in_the_table: set[str],
                                       used_substitutes: set[str], snapshot_root: str, failed: RowChoice) -> RowChoice:
    """编不过之后的补位（登记第一节读法 5「替换之后编不过（第八类，按 5.2 的补位规则换行）」）：换同一文件组里编不过那一行的下标 j + 1、
    两张表里都没有、这一次跑里别的补位也没用过的那一行，换上的行编之前照样核锚点（不过再换）；与编之前的补位合起来一行至多换三次，补不上交回 `mutation=None`、
    状态 `compile_failed`。空变异（`selected` 是 None）不补位。"""
    assert failed.mutation is not None
    attempts = failed.failed_attempts + ((failed.mutation.name, STATUS_COMPILE_FAILED),)
    if selected is None or failed.substitutions_used == MAXIMUM_SUBSTITUTIONS_PER_ROW:
        return RowChoice(None, STATUS_COMPILE_FAILED, failed.substituted_from, False, failed.substitutions_used, attempts)
    substitute = substitute_for(group_names_of_file(table, selected.file), failed.mutation.name, rows_in_the_table, used_substitutes)
    if substitute is None:
        return RowChoice(None, STATUS_COMPILE_FAILED, failed.substituted_from, False, failed.substitutions_used, attempts)
    used_substitutes.add(substitute)
    chosen = prepare_row(selected, table, rows_in_the_table, used_substitutes, snapshot_root, None, start_name=substitute,
                         substituted_from=selected.name if failed.substituted_from == "-" else failed.substituted_from,
                         substitutions_used=failed.substitutions_used + 1, failed_attempts=attempts)
    if chosen.mutation is None:  # 换上的行编之前就不过、又补不上：这一行的结局仍记编不过（最早的失败是编不过），每一次试的都在 failed_attempts 里
        return dataclasses.replace(chosen, status=STATUS_COMPILE_FAILED)
    return chosen


def compile_one_mutation(mutation: MutationRow, lane_tree: str, lane_target: str, snapshot_root: str, lane_directory: str, row_directory: str,
                         arguments: argparse.Namespace) -> tuple[int, int, str]:
    """一次改坏、编、还原（登记修订 5 的编译记录与两道核对在每一次编之前与之后都做）。交回 (cargo 退出码, 编译耗时 ns, 编过时核过重编了的包，
    逗号隔开；编不过是 `-`)。核对不过抛 `MeasurementPipeDefect`。编译日志写进行目录的 `compile.log`（下一次编会覆盖它，编不过的那一份由调用方先挪走）。"""
    record = read_lane_build_record(lane_directory)
    if record is None:
        if os.path.isdir(lane_target):
            raise MeasurementPipeDefect("lane_without_a_build_record", [lane_target])
        record = LaneBuildRecord(0, ())
    mark_files_newer_than_every_build(lane_tree, record.files_to_rebuild)
    problems_before = files_that_would_not_be_rebuilt(lane_tree, snapshot_root, record)
    if problems_before:
        raise MeasurementPipeDefect("before_the_compile", problems_before)
    files_to_rebuild = tuple(sorted(set(record.files_to_rebuild) | {mutation.file}))
    write_lane_build_record(lane_directory, LaneBuildRecord(record.last_compile_finished_ns, files_to_rebuild))
    mutated_path = os.path.join(lane_tree, mutation.file)
    with open(mutated_path, encoding="utf-8") as handle:
        original_text = handle.read()
    with open(mutated_path, "w", encoding="utf-8") as handle:
        handle.write(apply_mutation(original_text, mutation))
    artifacts_path = os.path.join(lane_directory, LANE_COMPILE_ARTIFACTS_FILE)
    exit_code, compile_ns = run_compile(lane_tree, lane_target, arguments.repo_root, arguments.threads, os.path.join(row_directory, "compile.log"),
                                        artifacts_path)
    compile_finished_ns = time.time_ns()
    shutil.copy2(os.path.join(snapshot_root, mutation.file), mutated_path)
    if exit_code != 0:  # 编不过：这一次改过的文件留在记录里，下一次编（这一行的补位或下一行）之前照样置时间、核对
        write_lane_build_record(lane_directory, LaneBuildRecord(compile_finished_ns, files_to_rebuild))
        return exit_code, compile_ns, "-"
    problems_after = packages_left_stale(library_freshness_by_package(artifacts_path, lane_tree), files_to_rebuild)
    if problems_after:
        write_lane_build_record(lane_directory, LaneBuildRecord(compile_finished_ns, files_to_rebuild))
        raise MeasurementPipeDefect("after_the_compile", problems_after)
    write_lane_build_record(lane_directory, LaneBuildRecord(compile_finished_ns, (mutation.file,)))
    return exit_code, compile_ns, ",".join(sorted({package_of_file(file) for file in files_to_rebuild}))


def keep_the_tail_of_the_compile_log(row_directory: str, failed_attempt_number: int) -> None:
    """编不过的那一次的编译日志只留尾部，改名成 `compile-failed-<序>.log`，序与 `row-result.tsv` 里记这一次的 `failed_attempt_<序>` 同号
    （这一行补位之后再编会写新的 `compile.log`）。"""
    with open(os.path.join(row_directory, "compile.log"), encoding="utf-8") as handle:
        tail = handle.readlines()[-COMPILE_LOG_TAIL_LINES:]
    with open(os.path.join(row_directory, f"compile-failed-{failed_attempt_number}.log"), "w", encoding="utf-8") as handle:
        handle.writelines(tail)
    os.remove(os.path.join(row_directory, "compile.log"))


def failed_attempt_fields(failed_attempts: tuple[tuple[str, str], ...]) -> dict[str, object]:
    """`row-result.tsv` 里试过而没用上的名字：`failed_attempts=<个数>`，每一个一行 `failed_attempt_<序>=<失败>:<变异名>`（序从 1 起，按先后）。"""
    fields: dict[str, object] = {"failed_attempts": len(failed_attempts)}
    for position, (name, status) in enumerate(failed_attempts, start=1):
        fields[f"failed_attempt_{position}"] = f"{status}:{name}"
    return fields


def names_in_the_tables(selected_rows: list[SelectedRow], other_table_paths: list[str]) -> set[str]:
    """补位要跳过的名字：这一次跑的表与别的几张表（扩表那一次是 117 行那张）里的每一个变异名。"""
    names = {row.name for row in selected_rows}
    for path in other_table_paths:
        names |= {row.name for row in read_selected_rows(path)}
    return names


def compile_failed_row_numbers(rows_roots: list[str]) -> set[int]:
    """几个输出根里 `row-result.tsv` 记 `status=compile_failed` 的行号；同一个行号在两个根里都有就拒。"""
    root_of_number: dict[int, str] = {}
    failed: set[int] = set()
    for rows_root in sorted(rows_roots):
        for entry in sorted(os.listdir(rows_root)):
            result_path = os.path.join(rows_root, entry, "row-result.tsv")
            if not entry.startswith("row-") or not os.path.exists(result_path):
                continue
            number = int(entry.split("-")[1])
            if number in root_of_number:
                raise ValueError(f"第 {number} 行在两个输出根里都有：{root_of_number[number]} 与 {rows_root}")
            root_of_number[number] = rows_root
            if read_result(os.path.join(rows_root, entry)).get("status") == STATUS_COMPILE_FAILED:
                failed.add(number)
    return failed


def read_result(directory: str) -> dict[str, object]:
    with open(os.path.join(directory, "row-result.tsv"), encoding="utf-8") as handle:
        return dict(line.rstrip("\n").split("\t", 1) for line in handle if "\t" in line)


def run_row_on_the_lane(number: int, selected: SelectedRow | None, empty_of: MutationRow | None, table: list[MutationRow],
                        rows_in_the_table: set[str], used_substitutes: set[str], arguments: argparse.Namespace,
                        lane: int) -> dict[str, object]:
    """一条线道一次只跑一行（同一条线道共用一棵树与一个编译目录）。"""
    row_directory = os.path.join(arguments.output_root, f"row-{number}")
    if arguments.skip_finished and os.path.isdir(row_directory):
        if os.path.exists(os.path.join(row_directory, "row-result.tsv")):
            return read_result(row_directory)
        shutil.rmtree(row_directory)  # 上一次中途停下的行目录：驱动自己建的，没有 row-result.tsv，删掉重做
    os.makedirs(row_directory, exist_ok=False)
    lane_tree = os.path.join(arguments.work_root, f"lane{lane}", "tree")
    lane_target = os.path.join(arguments.work_root, f"lane{lane}", "target")
    snapshot_root = os.path.join(arguments.work_root, "snapshot")
    fields: dict[str, object] = {"row_number": number, "row_name": selected.name if selected else (empty_mutation(empty_of).name if empty_of else "-"),
                                 "lane": lane}
    started = now_ns()
    compile_kind = "incremental" if os.path.isdir(lane_target) else "cold"
    sync_tree(snapshot_root, lane_tree)
    with SUBSTITUTION_LOCK:  # 补位挑行读写同一张「已用过的补位」集合，线道之间不能同时挑
        choice = prepare_row(selected, table, rows_in_the_table, used_substitutes, snapshot_root, empty_of)
    copy_ns, compile_ns_of_every_attempt = now_ns() - started, 0
    while choice.mutation is not None:
        attempt_started = now_ns()
        exit_code, compile_ns, rebuilt_packages = compile_one_mutation(choice.mutation, lane_tree, lane_target, snapshot_root,
                                                                       os.path.join(arguments.work_root, f"lane{lane}"), row_directory, arguments)
        copy_ns += now_ns() - attempt_started - compile_ns
        compile_ns_of_every_attempt += compile_ns
        if exit_code == 0:
            fields[REBUILD_CHECK_FIELD] = rebuilt_packages
            break
        keep_the_tail_of_the_compile_log(row_directory, len(choice.failed_attempts) + 1)
        if getattr(arguments, "keep_compile_failures_as_they_are", False):  # 只给复跑 s4、F4 两份产物用：它们是编不过不补位那一版驱动跑的
            choice = RowChoice(None, STATUS_COMPILE_FAILED, choice.substituted_from, False, choice.substitutions_used,
                               choice.failed_attempts + ((choice.mutation.name, STATUS_COMPILE_FAILED),))
            break
        with SUBSTITUTION_LOCK:
            choice = substitute_after_a_compile_failure(selected, table, rows_in_the_table, used_substitutes, snapshot_root, choice)
    fields.update({"substituted_from": choice.substituted_from, "line_matches_the_registration": str(choice.line_matches_the_registration).lower(),
                   "mutated_row_name": choice.mutation.name if choice.mutation else "-"})
    fields.update(failed_attempt_fields(choice.failed_attempts))
    if choice.mutation is None:
        compiled_at_least_once = any(status == STATUS_COMPILE_FAILED for _, status in choice.failed_attempts)
        fields.update({"status": choice.status, "device_exit": "-", "copy_ns": copy_ns,
                       "compile_ns": compile_ns_of_every_attempt if compiled_at_least_once else "-",
                       "compile_kind": compile_kind if compiled_at_least_once else "-", "device_ns": "-", "o3_ns": "-", "o3_status": O3_STATUS_NOT_REACHED})
        write_result(row_directory, fields)
        return fields
    fields.update({"mutation_line_sha256": hashlib.sha256(choice.mutation.raw_line.encode("utf-8")).hexdigest(), "copy_ns": copy_ns,
                   "compile_ns": compile_ns_of_every_attempt, "compile_kind": compile_kind})
    extra = ["--streams", arguments.streams]
    if arguments.writable_mount_probe:
        extra.append("--writable-mount-probe")
    if arguments.positive_control and arguments.positive_control != O3_CONTROL:
        extra += ["--positive-control", arguments.positive_control]
    if arguments.disable_oracle:
        extra += ["--disable-oracle", arguments.disable_oracle]
    binary = os.path.join(lane_target, "release", DEVICE_BINARY_NAME)
    device_exit, device_ns = run_device(arguments.repo_root, binary, row_directory, arguments.threads, arguments.memory_cap, extra,
                                        arguments.row_time_limit_seconds)
    fields.update({"status": STATUS_OK, "device_exit": device_exit, "device_ns": device_ns})
    # 退 3 是装置报了停机条款（S1、S3、S5、S9），导出照写：执行器照跑，结局记在 device_exit 里交比对器与主 agent
    if device_exit in (0, 3):
        fields.update(run_o3_step(arguments, row_directory))
    else:
        fields.update({"o3_status": O3_STATUS_NOT_REACHED, "o3_exit": "-", "o3_ns": "-", "o3_streams": "-"})
    if not arguments.keep_dumps:
        fields["bulky_dump_files_removed"] = remove_bulky_dump_files(os.path.join(row_directory, "dump"))
    write_result(row_directory, fields)
    return fields


def write_the_timing_output(path: str, output_root: str, row_results: list[dict[str, object]]) -> None:
    """登记修订 2 第 2.6 节：每行一行 `E7TIMING name=row_device row=<序号> device_exit=… device_ns=…`，接着那一行 `device.out` 里的 `E7TIMING` 行原样
    （含 `device_timing.py` 补的 `process_wall`）；没跑到装置的行只有第一行。新文件，已存在就拒。"""
    with open(path, "x", encoding="utf-8") as handle:
        for fields in row_results:
            number = fields.get("row_number", "-")
            handle.write(f"E7TIMING name=row_device row={number} device_exit={fields.get('device_exit', '-')} device_ns={fields.get('device_ns', '-')}\n")
            device_out = os.path.join(output_root, f"row-{number}", "device.out")
            if os.path.exists(device_out):
                with open(device_out, encoding="utf-8") as device_lines:
                    handle.writelines(line for line in device_lines if line.startswith("E7TIMING "))


def remove_bulky_dump_files(dump_directory: str) -> int:
    """判完删掉每条流导出里的三个大文件（`BULKY_DUMP_FILES`），交回删了几个。判定表、调用表、manifest 留着（触达的写流摘要比对器从 manifest 读不到时
    要整份导出，所以第一段的行一律带 `--keep-dumps`）。"""
    removed = 0
    for stream in STREAM_NAMES:
        for name in BULKY_DUMP_FILES:
            path = os.path.join(dump_directory, stream, name)
            if os.path.exists(path):
                os.remove(path)
                removed += 1
    return removed


def lane_worker(lane: int, jobs: "queue.Queue[tuple[int, int, SelectedRow | None, MutationRow | None]]", results: dict[int, dict[str, object]],
                table: list[MutationRow], rows_in_the_table: set[str], used_substitutes: set[str], arguments: argparse.Namespace,
                stop_taking_rows: threading.Event | None = None) -> None:
    """一条线道：从共用队列里领行（先到先得），一次一行，直到队列空。队列里的项是 (在作业表里的位置, 登记 5.9 表的序号, 表里的行, 空变异的行)。
    任一条线道报测量管子的缺陷，就设 `stop_taking_rows`：各线道跑完手上那一行之后不再领新行。"""
    while stop_taking_rows is None or not stop_taking_rows.is_set():
        try:
            position, number, selected, empty_of = jobs.get_nowait()
        except queue.Empty:
            return
        try:
            results[position] = run_row_on_the_lane(number, selected, empty_of, table, rows_in_the_table, used_substitutes, arguments, lane)
        except MeasurementPipeDefect as defect:
            results[position] = {"row_number": number, "lane": lane, "status": STATUS_MEASUREMENT_PIPE_DEFECT, "stage": defect.stage,
                                 "detail": ",".join(defect.details)}
            if stop_taking_rows is not None:
                stop_taking_rows.set()
        except Exception as error:  # 一行的驱动出错不拖垮别的线道；记下来、最后退非 0
            results[position] = {"row_number": number, "status": "driver_error", "error": f"{type(error).__name__}:{error}"}


def threads_per_lane(threads: int | None, lanes: int, environment=os.environ) -> int:
    """每条线道的线程数：给了照它；没给取登记的重活默认并行度（SINGLEFS_THREAD_CAP 设了取它）÷ 线道数，向下取整，至少 1。"""
    if threads is not None:
        return threads
    # 现用现引：第二台上这份驱动单独拷在套件目录里（rows_orchestrator 总是显式给 --threads），那里没有 scripts/hosts，不能在导入时引
    sys.path.insert(0, os.path.join(HERE, "..", "..", "hosts"))
    from heavy_threads import default_heavy_threads
    return max(1, default_heavy_threads(environment) // max(1, lanes))


def main(argument_list: list[str]) -> int:
    parser = argparse.ArgumentParser(description="E171 逐行驱动")
    parser.add_argument("--repo-root", required=True)
    parser.add_argument("--work-root", required=True, help="草稿目录：快照与每条线道的树与编译目录放这里")
    parser.add_argument("--output-root", required=True, help="每行一个 row-<序号>/ 目录；已存在的行目录拒绝覆盖")
    parser.add_argument("--selected-rows", default=os.path.join(HERE, "bad-implementations-117.tsv"))
    parser.add_argument("--other-selected-rows", action="append", default=[],
                        help="别的表（同一格式）：它们的名字补位时一样跳过；扩表那一次给 117 行那张")
    parser.add_argument("--also-selected-rows", action="append", default=None,
                        help="与 --selected-rows 并在一起按整表编号选行的表（默认扩表 164 行那一张：第 118–281 行，第 1 次重跑登记第一节）")
    parser.add_argument("--streams", default="1,2,3", help="透传给装置的 --streams（第一段三条流；第二段整表 1,2）")
    parser.add_argument("--writable-mount-probe", action="store_true", help="透传给装置的 --writable-mount-probe（Q4，第 242、269 行与基线）")
    parser.add_argument("--keep-dumps", action="store_true", help="判完不删导出里的三个大文件（第一段各行都带）")
    parser.add_argument("--rows", default="", help="登记 5.9 表里的序号，逗号隔开")
    parser.add_argument("--first-slice", action="store_true", help="第一刀：每个文件 i = 0 那一行（24 行）")
    parser.add_argument("--all", action="store_true", help="117 行")
    parser.add_argument("--empty-mutation", action="store_true", help="另跑空变异：拿表里第一行、把替换文换成原文本身（行目录 row-0）")
    parser.add_argument("--threads", type=int, default=None, help="每条线道的线程数；不给取登记的重活默认并行度 ÷ --lanes（向下取整，至少 1）")
    parser.add_argument("--memory-cap", default="4G")
    parser.add_argument("--lanes", type=int, default=1)
    parser.add_argument("--positive-control", choices=list(DEVICE_POSITIVE_CONTROLS) + [O3_CONTROL])
    parser.add_argument("--disable-oracle", choices=["O1", "O2"])
    parser.add_argument("--no-executor", action="store_true", help="不跑 O3 执行器（Q3-核「驱动不跑执行器」那一次）")
    parser.add_argument("--o3-control-bad-states", help="`--positive-control O3` 要的 E170 `bad-states` 导出目录（里面有每条流的 bad-states.tsv 与 bad-patches.bin）")
    parser.add_argument("--skip-finished", action="store_true", help="输出根里已有 row-result.tsv 的行目录不重跑；没有的删掉重做")
    parser.add_argument("--snapshot-from", help="快照从哪棵树同步（默认 --repo-root）：几趟要编同一棵树时，先把 Cargo.toml、Cargo.lock、.cargo、crates 拷一份冻住，各趟都指它")
    parser.add_argument("--keep-compile-failures-as-they-are", action="store_true",
                        help="编不过的行不补位、记编不过（只给 replay.sh 复跑 s4、F4 两份产物用：那两份是编不过之后不补位的那一版驱动跑的，登记第一节读法 5 要补位）")
    parser.add_argument("--rerun-compile-failed-from", action="append", default=[],
                        help="补位那一趟：只跑这几个输出根里结局是编不过的行（行从 --selected-rows 与 --other-selected-rows 的表里按序号取）")
    parser.add_argument("--baseline-timing", required=True,
                        help="同一棵冻结树、同一组 --streams 的基线那一趟的 -timing 产物：T 取它的 process_wall（登记修订 2 第 2.5 节）")
    parser.add_argument("--rerun-of-timed-out-rows", action="store_true",
                        help="超时的点名行单独再跑一次（一条线道）：限时再乘 3（修订 2 第 2.5 节）")
    parser.add_argument("--timing-output", required=True,
                        help="新文件（已存在就拒）：每行装置 device.out 里的 E7TIMING 行原样收进来，每行前一行 row_device（修订 2 第 2.6 节）")
    arguments = parser.parse_args(argument_list)
    arguments.threads = threads_per_lane(arguments.threads, arguments.lanes)
    if os.path.exists(arguments.timing_output):
        parser.error(f"--timing-output {arguments.timing_output} 已存在，不覆盖")
    with open(arguments.baseline_timing, encoding="utf-8") as handle:
        baseline = device_timing.read_baseline_timing(handle.read().splitlines())
    if not baseline.contains_the_stream_times():
        print(f"基线的进程挂钟没包住装置各流之和（或读不到）：{baseline}；那一次不算，重量一次（修订 2 第 2.5 节）", file=sys.stderr)
        return 2
    assert baseline.process_wall_seconds is not None, "contains_the_stream_times 为真时读到了 process_wall"
    arguments.row_time_limit_seconds = device_timing.row_time_limit_seconds(baseline.process_wall_seconds, arguments.rerun_of_timed_out_rows)
    print(f"name=row_time_limit seconds={arguments.row_time_limit_seconds} baseline_seconds={baseline.process_wall_seconds:.3f} "
          f"source={os.path.basename(arguments.baseline_timing)} rerun_of_timed_out_rows={str(arguments.rerun_of_timed_out_rows).lower()}",
          flush=True)
    if os.path.abspath(arguments.output_root) in {os.path.abspath(root) for root in arguments.rerun_compile_failed_from}:
        parser.error("补位那一趟的 --output-root 要是新的，不能是 --rerun-compile-failed-from 里的输出根")
    if arguments.positive_control == O3_CONTROL and not arguments.o3_control_bad_states:
        parser.error("--positive-control O3 要同时给 --o3-control-bad-states")
    if arguments.positive_control == O3_CONTROL and arguments.no_executor:
        parser.error("--positive-control O3 与 --no-executor 不能一起")
    also_selected = arguments.also_selected_rows if arguments.also_selected_rows is not None else [
        os.path.join(HERE, "bad-implementations-expansion-164.tsv")]
    selected_rows = read_selected_rows(arguments.selected_rows) + [row for path in also_selected for row in read_selected_rows(path)]
    chosen: list[SelectedRow]
    if arguments.rerun_compile_failed_from:
        failed_numbers = compile_failed_row_numbers(arguments.rerun_compile_failed_from)
        rows_of_every_table = selected_rows + [row for path in arguments.other_selected_rows for row in read_selected_rows(path)]
        chosen = sorted((row for row in rows_of_every_table if row.number in failed_numbers), key=lambda row: row.number)
        if sorted(row.number for row in chosen) != sorted(failed_numbers):
            print(f"编不过的行号在给的表里找不全：要 {sorted(failed_numbers)}，表里有 {[row.number for row in chosen]}", file=sys.stderr)
            return 2
        print(f"rerun_compile_failed rows={','.join(str(row.number) for row in chosen) or 'none'}")
    elif arguments.all:
        chosen = selected_rows
    elif arguments.first_slice:
        chosen = [row for row in selected_rows if row.slice_number == 1]
    else:
        wanted = {int(token) for token in arguments.rows.split(",") if token}
        chosen = [row for row in selected_rows if row.number in wanted]
    started = now_ns()
    if not arguments.no_executor:
        differences = frozen_o3.freeze_differences(arguments.repo_root)
        print(f"freeze_check before=true differences={len(differences)} detail={','.join(differences) or 'none'}")
        if differences:
            print("冻结的执行器与冻结行对不上（作废条款 Z2），这一趟不跑", file=sys.stderr)
            return 4
    snapshot_root = os.path.join(arguments.work_root, "snapshot")
    sync_tree(arguments.snapshot_from or arguments.repo_root, snapshot_root)
    digest, file_count = snapshot_digest(snapshot_root)
    refusals = work_root_refusals(arguments.work_root, arguments.lanes, digest)
    if refusals:
        print(f"work_root_refused reasons={','.join(refusals)}")
        print("工作根里的编译目录不是这一版驱动在这张快照上建的（登记修订 5），换一个新的 --work-root", file=sys.stderr)
        return EXIT_WORK_ROOT_REFUSED
    with open(os.path.join(arguments.work_root, SNAPSHOT_DIGEST_FILE), "w", encoding="utf-8") as handle:
        handle.write(digest + "\n")
    os.makedirs(arguments.output_root, exist_ok=True)
    table = read_mutation_rows(os.path.join(snapshot_root, MUTATIONS_TABLE))
    rows_in_the_table = names_in_the_tables(selected_rows, arguments.other_selected_rows)
    used_substitutes: set[str] = set()
    jobs: list[tuple[int, SelectedRow | None, MutationRow | None]] = []
    if arguments.empty_mutation:
        first_row = locate_by_name(selected_rows[0], table).row
        if first_row is None:
            print("空变异要的表里第一行找不到", file=sys.stderr)
            return 2
        jobs.append((0, None, first_row))
    jobs.extend((row.number, row, None) for row in chosen)
    print(f"snapshot_digest={digest} files={file_count} snapshot_ns={now_ns() - started} rows={len(jobs)}")
    work: "queue.Queue[tuple[int, int, SelectedRow | None, MutationRow | None]]" = queue.Queue()
    for position, (number, selected, empty_of) in enumerate(jobs):
        work.put((position, number, selected, empty_of))
    results: dict[int, dict[str, object]] = {}
    table_started = now_ns()
    stop_taking_rows = threading.Event()
    threads = [threading.Thread(target=lane_worker, args=(lane, work, results, table, rows_in_the_table, used_substitutes, arguments, stop_taking_rows))
               for lane in range(arguments.lanes)]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join()
    table_ns = now_ns() - table_started
    for position in sorted(results):
        print("row " + " ".join(f"{key}={str(value).replace(' ', '_')}" for key, value in results[position].items()))
    print(f"table lanes={arguments.lanes} threads_per_lane={arguments.threads} rows={len(jobs)} table_wall_ns={table_ns} "
          f"sum_of_row_ns={sum(int(value) for fields in results.values() for key, value in fields.items() if key in ('copy_ns', 'compile_ns', 'device_ns', 'o3_ns') and str(value).isdigit())}")
    write_the_timing_output(arguments.timing_output, arguments.output_root, [results[position] for position in sorted(results)])
    if not arguments.no_executor:
        after = frozen_o3.freeze_differences(arguments.repo_root)
        print(f"freeze_check before=false differences={len(after)} detail={','.join(after) or 'none'}")
    if any(fields.get("status") == STATUS_MEASUREMENT_PIPE_DEFECT for fields in results.values()):
        return EXIT_MEASUREMENT_PIPE_DEFECT
    return 1 if len(results) != len(jobs) or any(fields.get("status") == "driver_error" for fields in results.values()) else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
