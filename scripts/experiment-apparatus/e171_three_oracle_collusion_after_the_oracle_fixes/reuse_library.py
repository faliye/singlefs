#!/usr/bin/env python3
"""E171 第 1 次重跑·修订 11：按输入复用的那一层（登记 research/prompts/e171-r1-prereg.md 修订 11 第 11.1–11.9 节与其后「主 agent 认定：修订 11 交回」）。
单元的键（第 11.2 节）、复用库（第 11.5 节：查、取、原子入库、隔离、读数相等）、复用账本（第 11.4 节）、RC1–RC5 与 S18（第 11.9 节）都在这里；
三层编排（segment_one_units_revision_11.py、rows_orchestrator_revision_11.py、rows_orchestrator_segment_two_revision_11.py）与计数脚本
count_reusable_units.py 都从这里取。参照实现是设计员草稿 reuse_key.py（照着写进仓，不从仓里 import 草稿）。

键 = 键文本的 sha256。键文本第一行是 KEY_SCHEMA_LINE，之后十节，节名与次序写死：
  [kind]      单元种类：device | e170 | driver-row | o3-executor | mutation-driver
  [build]     cargo -V 与 rustc -vV 原样的 sha256；根与 crates/*/ 的 Cargo.toml、Cargo.lock、树里 .cargo/ 下每个文件的 sha256；
              admission.build_environment_lines 给的构建环境各行（名字与内容的 sha256）
  [closure]   树里编的 release 二进制的 dep-info 列的每个源文件，相对树根的路径与 sha256；驱动行用改坏之后的内容（bad_implementation_table.apply_mutation）；
              树外的路径出现就停（S18 ②，ClosureOutsideTheTree）
  [env]       闭包源码里按名字读的 SINGLEFS_、KS_、RAYON_ 字面量，去掉 capped.sh 设的八个线程数变量，逐个记 set / unset；有一个 set 的单元不查库、不进库
  [mutation]  行号、行表那一行原文、变异表那一行原文；空变异记 empty-of <原名> 与那一行原文
  [args]      装置或驱动的参数，去掉输出目录、--threads、--memory-cap、--lanes、工作根、输出根
  [limit]     逐行限时 L 的秒数；不限时的单元记 none
  [upstream]  角色名与上游读数各文件的 sha256（按读数内容，不按上游的键）
  [research]  研究一侧运行期闭包各文件的 sha256（装置单元 device_timing.py；驱动行是驱动与它 import 的四份；跑执行器的另加执行器目录、冻结行那份产物、
              frozen_o3.py、o3_bad_image_judge.py、result_lines.py；变异驱动是研究一侧整个目录与它的变异表）
  [python]    python3 -VV 原样
库（H30，默认 ~/.local/share/singlefs/reuse/e171-r1/）：entries/<键前 2 位>/<键>/ 下 key.txt、reading/<角色>…、meta.tsv、manifest.sha256（最后写）；
  staging/<进程号>-<随机>/（先写这里再 rename；起跑时清残留）；quarantine/（RC5 不符的条目挪进来）；frozen-by-s18（S18 触发时写，在就不取、不入）。
  读数按角色存：product、timing（文件），directory（装置的导出目录），row（驱动行的行目录）；meta.tsv 记这一单元原来的名字（前缀后缀换成占位）。
账本：<导出根>/reuse-markers/<层>/ 下每个单元一份标记（先写标记、再链接、最后链 row-result.tsv 或写绿行），events.tsv 收 RC1–RC5 的每一次；
  每层收尾由标记现合成 E7RESULT name=reuse / name=reuse_check / name=reuse_insert 各行与末行 name=verdict（N + M 不等于这一层登记的单元数判红）。

用法：
  reuse_library.py fetch --library L --entry E --map <角色>=<目标路径> [--map …]
      单元脚本取用：核 RC5（不符的挪进隔离目录、退 1）、按角色把条目里的读数硬链接到这一趟的名字（目标已在就拒、退 1）；标准输出一行条目 meta 里的退出码。
  reuse_library.py selfproof --library L --binary B --scratch S --output O --threads N --memory-cap C
      RP2（修订 8 那一趟 step-0 绿之后的 reuse-selfproof 单元）：H7 那个装置二进制跑三次 stream-constants --streams 1,2,3。
  reuse_library.py --selftest [--cells 格,…] [--rounds N]
      RP1、RP3–RP8（假树、假装置、假驱动、假判法；RP1 另起三层编排各自的端到端格），钉值认搬迁登记的两格（pin-moves：假前缀的红绿红样本；
      pin-moves-real-files：本目录每一处钉值的被钉文件照一次假搬迁换过之后照样认、多改一处就不认）与冻结执行器核对认搬迁登记的单测（frozen-o3-moves：
      子进程跑 test_frozen_o3），倒换只认正着换一遍不动的、逐次倒的两格（pin-moves-partial：钉值核对；frozen-o3-reverse-moves：本目录与 E171 第 1 次那两份
      冻结执行器核对），末尾逐个打开弄坏开关核红在点名格。末行「自证 N 项，红 M」。
退出码：0 成；1 取用不成或 RP2 不过；2 用法错；78 库目录写不了或与 scripts/experiment-apparatus/target 不在同一个文件系统（硬链接不成）。

弄坏开关 REUSE_BREAK=<项>（逗号连几个；只在 --selftest 起的子进程里认，REUSE_SELFTEST=1；别处设了退 2）。项名、弄坏哪一处、打开后红在哪一格：
  drop-kind                键文本里不写 [kind]                                  rp3-kind
  drop-build               不写 [build]                                         rp3-build
  drop-closure             不写 [closure]                                       rp3-closure
  drop-env                 不写 [env]                                           rp3-env
  drop-mutation            不写 [mutation]                                      rp3-mutation
  drop-args                不写 [args]                                          rp3-args
  drop-limit               不写 [limit]                                         rp3-limit
  drop-upstream            不写 [upstream]                                      rp3-upstream
  drop-research            不写 [research]                                      rp3-research
  drop-python              不写 [python]                                        rp3-python
  check-skipped            RC1–RC3 不比读数（库里有同键条目就算相同）           rp7-check-mismatch
  closure-check-skipped    RC4 不核 dep-info                                    rp6-closure-extra-file
  manifest-unchecked       RC5 不核 manifest.sha256                             rp8-corrupted-entry
  insert-in-place          入库直接写进 entries/、不经 staging/                 rp5-atomic-insert
  fresh-always-ignored     永远现跑的几类（step-9 等）也查库                    rp1-two-passes
  pin-moves-ignored        钉值核对不看搬迁登记（pinned_sha256_now 只认逐字节相同）   pin-moves
  frozen-o3-moves-ignored  冻结执行器核对不看搬迁登记（子进程设 FROZEN_O3_BREAK=path-moves-ignored）   frozen-o3-moves
  pin-partial-accepted     钉值核对倒换之前不核此刻的内容正着换一遍动不动（格里设 FROZEN_O3_BREAK=partial-accepted）   pin-moves-partial
  frozen-o3-partial-accepted  冻结执行器核对同上（两份 frozen_o3，格里设 FROZEN_O3_BREAK=partial-accepted）   frozen-o3-reverse-moves
  frozen-o3-whole-chain-only  冻结执行器核对只倒全部登记那一版、不逐次倒（格里设 FROZEN_O3_BREAK=whole-chain-only）   frozen-o3-reverse-moves
自证只换假件的变量（只在 REUSE_SELFTEST=1 时认）：REUSE_SELFTEST_KILL_AT=before-manifest|before-rename|before-row-result:<行目录名>
  （入库或取用走到那一处时进程给自己发 SIGKILL，RP4、RP5 用）。
"""
from __future__ import annotations

import argparse
import dataclasses
import hashlib
import importlib.machinery
import importlib.util
import math
import os
import re
import secrets
import shutil
import signal
import subprocess
import sys
import tempfile
import time
import types

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
import revision_8_selftest_harness as harness  # noqa: E402
import frozen_o3  # noqa: E402（搬迁登记的读法：钉值核对与冻结执行器核对共用 frozen_o3 那一份）

BREAK_VARIABLE = "REUSE_BREAK"
SELFTEST_VARIABLE = "REUSE_SELFTEST"
KILL_POINT_VARIABLE = "REUSE_SELFTEST_KILL_AT"
EXIT_RUN_CONDITION = 78
RESEARCH_RELATIVE = "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes"
EXECUTOR_RELATIVE = "scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487"
FREEZE_REFERENCE_RELATIVE = "research/results/e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out"
MUTATIONS_TABLE_RELATIVE = "scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes.tsv"
ADMISSION_SCRIPT_RELATIVE = "scripts/gate/admission.py"
DEVICE_BINARY = "e171_three_oracle_collusion_after_the_oracle_fixes"
E170_BINARY = "e170_o3_spec_executor_export"
DEFAULT_LIBRARY_ROOT = os.path.expanduser("~/.local/share/singlefs/reuse/e171-r1")
KEY_SCHEMA_LINE = "schema e171-r1-reuse-key 1"
SECTION_ORDER = ("kind", "build", "closure", "env", "mutation", "args", "limit", "upstream", "research", "python")
UNIT_KINDS = ("device", "e170", "driver-row", "o3-executor", "mutation-driver")
# scripts/hosts/capped.sh 第 22–31 行设的八个线程数变量：不进 [env]（线程数在键外，第 11.2 节「键外的几样」）
THREAD_COUNT_VARIABLES = ("CARGO_BUILD_JOBS", "RUST_TEST_THREADS", "SINGLEFS_LAYER0_THREADS", "SINGLEFS_CRASH_INJECTION_THREADS",
                          "SINGLEFS_FAULT_INJECTION_WORKER_THREADS", "SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS",
                          "SINGLEFS_RANDOM_HISTORY_THREADS", "SINGLEFS_THREAD_CAP")
ENVIRONMENT_NAME_PATTERN = re.compile(r'"(SINGLEFS_[A-Z0-9_]+|KS_[A-Z0-9_]+|RAYON_[A-Z0-9_]+)"')
# 驱动行运行期闭包（row_driver.py 与它 import 的四份）；两张行表不整份进键（一行只吃自己那一行，在 [mutation]）
DRIVER_RESEARCH_FILES = ("row_driver.py", "device_timing.py", "frozen_o3.py", "run_directory.py", "bad_implementation_table.py")
EXECUTOR_RESEARCH_FILES = ("frozen_o3.py", "o3_bad_image_judge.py", "result_lines.py")
DEVICE_RESEARCH_FILES = ("device_timing.py",)
SELECTED_ROW_TABLES = ("bad-implementations-117.tsv", "bad-implementations-expansion-164.tsv")
# 改坏的那一处碰到模块集合的，dep-info 认不全（第 11.2 节 RC4 的静态拦）：永远现跑
MODULE_SET_TOKENS = ("mod ", "#[path", "include_str!", "include_bytes!", "include!")
TIMEOUT_EXIT_CODE = 253
WRAPPER_OWN_EXIT_CODES = (250, 251, 252, 254)
# 读数相等（第 11.5 节）：驱动行的 row-result.tsv 去掉这几个键再比
ROW_RESULT_VOLATILE_KEYS = ("copy_ns", "compile_ns", "device_ns", "o3_ns", "lane", "compile_kind", "packages_checked_rebuilt")
REBUILD_CHECK_FIELD = "packages_checked_rebuilt"
STATE_FRESH, STATE_REUSED, STATE_FRESH_CHECK, STATE_FRESH_ALWAYS = "fresh", "reused", "fresh-check", "fresh-always"
FROZEN_FILE = "frozen-by-s18"


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


def selftest_only(name: str) -> str:
    """只在自证起的子进程里认的换假件变量；平常交回空串。"""
    return os.environ.get(name, "") if os.environ.get(SELFTEST_VARIABLE) == "1" else ""


def stop_here_if_asked(point: str) -> None:
    """自证 RP4、RP5：走到点名的那一处时进程给自己发 SIGKILL（平常不认）。"""
    if selftest_only(KILL_POINT_VARIABLE) == point:
        os.kill(os.getpid(), signal.SIGKILL)


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def sha256_file(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def files_under(directory: str) -> list[str]:
    """目录下每个文件相对目录的路径，排好序（__pycache__ 与 .pyc 不算）。"""
    found = []
    for root, directories, names in os.walk(directory):
        directories[:] = sorted(name for name in directories if name != "__pycache__")
        found += [os.path.relpath(os.path.join(root, name), directory) for name in names if not name.endswith(".pyc")]
    return sorted(found)


# ── 键（第 11.2 节）──
class ClosureOutsideTheTree(Exception):
    """dep-info 里有树外的路径：闭包认不全（S18 ②）。"""


def dep_info_files(tree: str, dep_info: str) -> list[str]:
    """dep-info（Makefile 形）第一条规则的依赖，换成相对树根的路径（按真实路径比，#[path] 绕出去又绕回来的也认）；树外的路径抛 ClosureOutsideTheTree。"""
    with open(dep_info, encoding="utf-8") as handle:
        text = handle.read().split("\n\n", 1)[0].replace("\\\n", " ")
    _, dependencies = text.split(": ", 1)
    root = os.path.realpath(tree) + os.sep
    files = []
    for token in re.split(r"(?<!\\) ", dependencies.strip()):
        token = token.replace("\\ ", " ").strip()
        if not token:
            continue
        real = os.path.realpath(token if os.path.isabs(token) else os.path.join(tree, token))
        if not real.startswith(root):
            raise ClosureOutsideTheTree(f"dep-info {dep_info} 里有树外的文件：{token}")
        files.append(os.path.relpath(real, root.rstrip(os.sep)))
    return sorted(set(files))


def closure_lines(tree: str, files: list[str], overrides: dict[str, bytes]) -> list[str]:
    """[closure]：每个文件「相对路径 sha256」；overrides 里的文件按给的内容算（驱动行改坏之后的那一份）。"""
    lines = []
    for relative in files:
        digest = sha256_bytes(overrides[relative]) if relative in overrides else sha256_file(os.path.join(tree, relative))
        lines.append(f"{relative} {digest}")
    return lines


def toolchain_text(tree: str) -> str:
    """cargo -V 与 rustc -vV 原样（在树里起，树里有 rust-toolchain 的照它选）。"""
    outputs = []
    for command in (["cargo", "-V"], ["rustc", "-vV"]):
        completed = subprocess.run(command, cwd=tree, capture_output=True, text=True)
        if completed.returncode != 0:
            raise RuntimeError(f"{' '.join(command)} 退 {completed.returncode}：{completed.stderr.strip()}")
        outputs.append(completed.stdout)
    return "".join(outputs)


_admission_module: dict[str, types.ModuleType] = {}


def build_environment_lines(repo: str, tree: str) -> list[str]:
    """scripts/gate/admission.py 的 build_environment_lines（按路径载入、不写 .pyc）给的构建环境各行：「名字 内容的 sha256」。"""
    if "module" not in _admission_module:
        specification = importlib.util.spec_from_file_location("admission_for_reuse_keys", os.path.join(repo, ADMISSION_SCRIPT_RELATIVE))
        module = importlib.util.module_from_spec(specification)
        sys.dont_write_bytecode = True
        sys.modules[specification.name] = module
        specification.loader.exec_module(module)
        _admission_module["module"] = module
    return [f"{name} {sha256_bytes(content)}" for name, content in _admission_module["module"].build_environment_lines(tree)]


def build_lines(tree: str, toolchain: str, environment_lines: list[str]) -> list[str]:
    """[build]：工具链、清单与锁、树里 .cargo/ 下每个文件、构建环境。"""
    lines = [f"toolchain {sha256_bytes(toolchain.encode('utf-8'))}"]
    manifests = ["Cargo.toml", "Cargo.lock"] + sorted(os.path.join("crates", name, "Cargo.toml") for name in os.listdir(os.path.join(tree, "crates"))
                                                    if os.path.isfile(os.path.join(tree, "crates", name, "Cargo.toml")))
    lines += [f"{relative} {sha256_file(os.path.join(tree, relative))}" for relative in manifests]
    cargo_directory = os.path.join(tree, ".cargo")
    if os.path.isdir(cargo_directory):
        lines += [f".cargo/{relative} {sha256_file(os.path.join(cargo_directory, relative))}" for relative in files_under(cargo_directory)]
    else:
        lines.append(".cargo absent")
    return lines + [f"environment {line}" for line in environment_lines]


def environment_names_in(tree: str, files: list[str]) -> list[str]:
    names: set[str] = set()
    for relative in files:
        with open(os.path.join(tree, relative), encoding="utf-8", errors="replace") as handle:
            names |= set(ENVIRONMENT_NAME_PATTERN.findall(handle.read()))
    return sorted(names - set(THREAD_COUNT_VARIABLES))


def environment_lines(names: list[str], environment: dict[str, str]) -> list[str]:
    """[env]：闭包里按名字读的每个变量此刻设没设。"""
    return [f"{name} {'set' if name in environment else 'unset'}" for name in names]


def research_lines(repo: str, relative_paths: list[str]) -> list[str]:
    """[research]：每个文件（目录就是它底下每个文件）「相对仓根的路径 sha256」。"""
    lines = []
    for relative in sorted(set(relative_paths)):
        path = os.path.join(repo, relative)
        if os.path.isdir(path):
            lines += [f"{os.path.join(relative, inner)} {sha256_file(os.path.join(path, inner))}" for inner in files_under(path)]
        else:
            lines.append(f"{relative} {sha256_file(path)}")
    return lines


# [args] 不进键的那几项（带一个值）：输出与工作的落点、线程、内存上限、线道，与已经在别的节里的输入（树进 [closure]、基线计时进 [limit]、上游进 [upstream]）
VOLATILE_ARGUMENT_OPTIONS = ("--output-directory", "--dump-directory", "--output-root", "--work-root", "--threads", "--memory-cap", "--lanes",
                             "--timing-output", "--baseline-timing", "--snapshot-from", "--repo-root", "--o3-control-bad-states", "--other-selected-rows")


def normalized_arguments(words: list[str]) -> str:
    kept, skip_next = [], False
    for word in words:
        if skip_next:
            skip_next = False
        elif word in VOLATILE_ARGUMENT_OPTIONS:
            skip_next = True
        else:
            kept.append(word)
    return " ".join(kept)


def limit_lines(seconds: int | None) -> list[str]:
    """[limit]：逐行限时 L 的秒数；不限时记 none。"""
    if seconds is None:
        return ["none"]
    return [f"seconds {seconds}"]


def upstream_lines(upstream: list[tuple[str, str, str]]) -> list[str]:
    """[upstream]：(角色, 上游读数单元名, 上游读数的目录或文件) → 角色与上游读数每个文件的 sha256（按内容，不按上游的键）。"""
    lines = []
    for role, unit_name, location in upstream:
        if os.path.isdir(location):
            lines += [f"{role} {relative} {sha256_file(os.path.join(location, relative))}" for relative in files_under(location)]
        else:
            lines += [f"{role} {os.path.basename(location)} {sha256_file(location)}"]
    return lines


def key_text(sections: dict[str, list[str]]) -> str:
    unknown = set(sections) - set(SECTION_ORDER)
    if unknown:
        raise ValueError(f"认不得的节：{sorted(unknown)}")
    lines = [KEY_SCHEMA_LINE]
    for name in SECTION_ORDER:
        if broken(f"drop-{name}"):
            continue
        lines.append(f"[{name}]")
        values = list(sections.get(name, []))
        lines.extend(values if name == "args" else sorted(values))
    return "\n".join(lines) + "\n"


def key_of(sections: dict[str, list[str]]) -> str:
    return sha256_bytes(key_text(sections).encode("utf-8"))


def first_differing_section(first: dict[str, list[str]], second: dict[str, list[str]]) -> str:
    for name in SECTION_ORDER:
        if sorted(first.get(name, [])) != sorted(second.get(name, [])):
            return name
    return "-"


def python_text() -> str:
    return subprocess.run([sys.executable, "-VV"], capture_output=True, text=True).stdout.strip()


# ── 单元表（第 11.1 节读数单元表；名字照 segment_one_units_revision_8.py 的 unit_table 与两份逐行编排器）──
@dataclasses.dataclass(frozen=True)
class ReadingUnit:
    """一个装置、E170 导出装置、执行器或变异驱动进程。owner 是修订 8 单元编排里装它的单元；step_name 是单元脚本里那一步的名字
    （产物名 <PREFIX>-<step_name><TAG><SUFFIX>.out）；upstream 是 (角色, 上游读数单元名)；fresh_always 是 RC2 永远现跑的第二趟。"""
    name: str
    owner: str
    step_name: str
    kind: str
    arguments: str
    upstream: tuple[tuple[str, str], ...] = ()
    fresh_always: bool = False
    expected_exits: tuple[int, ...] = (0,)


@dataclasses.dataclass(frozen=True)
class RowUnit:
    """逐行驱动的一行。flags 是驱动参数（去掉易变项之后的那一串）；second_run 是第二趟、跨机那一份（RC2 永远现跑）；rerun 是超时重跑（限时再乘 3）。"""
    name: str
    number: int
    empty: bool
    flags: tuple[str, ...]
    executor: bool
    second_run: bool
    rerun: bool = False
    upstream: tuple[tuple[str, str], ...] = ()
    owner: str = ""
    step_name: str = ""


def positive_control(slug: str, streams: str, control: str, probe: bool = False, expected_exits: tuple[int, ...] = (0,)) -> ReadingUnit:
    arguments = f"baseline --streams {streams}{' --writable-mount-probe' if probe else ''} --positive-control {control}"
    return ReadingUnit(f"step-3-{slug}", f"step-3-{slug}", f"positive-control-{slug}", "device", arguments, expected_exits=expected_exits)


STREAMS_ONE_TWO = "new_pool_file_creation,fixed_script_through_e_then_unmount"
STREAMS_ONE_TWO_THREE = STREAMS_ONE_TWO + ",first_file_unlinked_then_inodes_across_one_leaf"
SEGMENT_ONE_READING_UNITS = (
    ReadingUnit("step-0/stream-constants", "step-0", "stream-constants", "device", "stream-constants --streams 1,2,3"),
    ReadingUnit("step-0/s3-s11", "step-0", "s3-s11-baseline-stream-1-with-probe", "device", "baseline --streams 1 --writable-mount-probe"),
    ReadingUnit("step-1", "step-1", "s2-device-baseline-streams-1-2", "device", "baseline --streams 1,2"),
    ReadingUnit("step-2", "step-2", "s2-e170-export-dump", "e170", "dump"),
    ReadingUnit("step-2b", "step-2b", "e170-bad-states-second-run", "e170", "bad-states-second-run --seed-hex 85441ae491419411", (("e170-export", "step-2"),)),
    positive_control("o1-comparison", "2", "O1-对拍"),
    positive_control("o1-comparison-unmount", "2", "O1-对拍-卸载"),
    positive_control("o2", "1", "O2"),
    positive_control("return-value", "2", "交回值"),
    positive_control("memory-object", "2", "内存对象"),
    positive_control("panic", "2", "panic", expected_exits=(3,)),
    positive_control("witness", "1", "见证", probe=True),
    positive_control("o1-crash-judge", "1,2,3", "O1-崩溃判官"),
    positive_control("reference-filesystem", "1,2,3", "RefFS"),
    ReadingUnit("step-3-o3-p1", "step-3-o3-p1", "positive-control-o3-p1", "o3-executor", "control --row-name P1-F",
                (("device-baseline", "step-1"), ("e170-second-run", "step-2b"))),
    ReadingUnit("step-5", "step-5", "o3-baseline-streams-1-2", "o3-executor", f"executor --streams {STREAMS_ONE_TWO}", (("device-baseline", "step-1"),)),
    ReadingUnit("step-6", "step-6", "q5-reverse-at-the-return-point-of-call-2", "device", "confirmed-loss-form --truncate-at-the-return-point"),
    ReadingUnit("step-7", "step-7", "baseline-run1", "device", "baseline --streams 1,2,3 --writable-mount-probe"),
    ReadingUnit("step-8", "step-8", "o3-baseline-run1", "o3-executor", f"executor --streams {STREAMS_ONE_TWO_THREE}", (("baseline-run1", "step-7"),)),
    ReadingUnit("step-9", "step-9", "baseline-run2", "device", "baseline --streams 1,2,3 --writable-mount-probe", fresh_always=True),
    ReadingUnit("step-10-run-1", "step-10-run-1", "q5-main-before-the-first-barrier-of-call-3-run-1", "device", "confirmed-loss-form"),
    ReadingUnit("step-10-run-2", "step-10-run-2", "q5-main-before-the-first-barrier-of-call-3-run-2", "device", "confirmed-loss-form", fresh_always=True),
    ReadingUnit("research-mutations", "research-mutations", "research-mutations", "mutation-driver", MUTATIONS_TABLE_RELATIVE),
)
# 第 4 步十项：(单元名里那一段, 阳性对照, 流与探针)；单元脚本里那一行是 empty_control <段> <对照> <流与探针>（count_reusable_units.py 开头核）
STEP_4_ITEMS = (("o1-comparison", "O1-对拍", "--streams 1,2"), ("o1-comparison-unmount", "O1-对拍-卸载", "--streams 1,2"),
                ("o1-crash-judge", "O1-崩溃判官", "--streams 1,2"), ("reference-filesystem", "RefFS", "--streams 1,2"), ("o2", "O2", "--streams 1,2"),
                ("return-value", "交回值", "--streams 1,2"), ("memory-object", "内存对象", "--streams 1,2"), ("panic", "panic", "--streams 1,2"),
                ("witness", "见证", "--streams 1 --writable-mount-probe"), ("o3-p1", "O3", "--streams 1,2"))


def step_4_row_units() -> list[RowUnit]:
    rows = []
    for slug, control, streams in STEP_4_ITEMS:
        flags = ("--empty-mutation", "--keep-dumps", "--positive-control", control) + tuple(streams.split())
        upstream = (("e170-second-run", "step-2b"),) if control == "O3" else ()
        rows.append(RowUnit(f"step-4-{slug}", 0, True, flags, True, False, upstream=upstream, owner=f"step-4-{slug}", step_name=f"empty-control-{slug}-driver"))
    return rows


def readings_of_owner(owner: str) -> list:
    """修订 8 单元编排里一个单元装的读数单元（step-9-z1、step-10-z7 只有判法，交回空表）。"""
    return [unit for unit in list(SEGMENT_ONE_READING_UNITS) + step_4_row_units() if unit.owner == owner]


def segment_one_reading_names() -> list[str]:
    return [unit.name for unit in list(SEGMENT_ONE_READING_UNITS) + step_4_row_units()]


_pinned_modules: dict[str, types.ModuleType] = {}
# 第一段编排器 rows_orchestrator.py 搬目录（10-11、10-14 两次提交改了里面的路径）之后按现内容重钉：修订 11 这几层认 ORCHESTRATOR_SHA256。
# 冻结的修订 8、修订 10 与它们钉住的 cross_host_o3_check.py、rows_orchestrator_segment_two.py 里写的是搬之前的 ORCHESTRATOR_SHA256_BEFORE_MOVE，
# 一个字节不改：修订 11 载入它们时先核它们自己的 sha256（load_repinned），再把正文里那个旧值换成新值编（RepinnedLoader）。
# 之后整个目录再搬家、登记在 .claude/gate.d/path-moves.tsv 的（读法在 frozen_o3 文件头「搬迁登记」）：钉值一个不改。核 sha256 时
# （pinned_sha256_now）此刻的内容不是钉住的那一份，就把此刻内容里登记过的新前缀从最近一次起逐次倒着换回旧前缀再算，有一步是钉值就认（只差登记过的前缀）；
# 此刻的内容照全部登记正着换一遍要不动，只换了一半的不倒、不认；
# 冻结层正文里要换的钉值（orchestrator_repin、rows_revision_8_repins）换成被钉文件此刻的 sha256（认的时候），冻结层自己按字节比的那一处照过；
# 不认的还是换成钉值，那一处照旧红。取「把此刻的内容换回去」这一种，不取「把钉住那一版正着换」：钉住那一版的内容这里没有，只有它的 sha256。
# 用全部登记行倒换（不只此刻生效的）：换回去之后逐字节等于钉住的那一版，此刻与它就只差登记过的前缀；还改了别处的换回去也对不上。登记表不在或没有内容行时与改前逐字节同判。
# 关不住的一种：登记表里留着一行老行（旧前缀下已没有文件、新前缀是早就在的另一份实现），再加一行真搬迁，钉住的文件改指那一份早就在的实现，换回去照样等于钉值，这里认；
# 门禁一侧这份文件照算改动，要人看（.claude/rules/path-moves.md「门禁管哪一半」）。换算只认仓根起的前缀（lib-lag-baseline.py 文件头「搬迁登记」）。
ORCHESTRATOR_SHA256_BEFORE_MOVE = "1dcfb8d0c08a706077f41fe69d3566d7a99e16f3dcbffc10808a2f0d88d17853"
# scripts/experiments → scripts/experiment-apparatus 那一次搬迁，两份编排器造假仓的函数里把路径拆成 "scripts", "experiments" 几段写，
# 跟着换了，不只差登记过的前缀，按现内容重钉这两个值（records/2026-10-23-实验装置目录改名.md 批 2）。
# 第一段编排器改成每台线程取登记值、不按此刻空闲现算之后，ORCHESTRATOR_SHA256 再按现内容重钉一次（records/2026-10-22-子agent结果落地审计.md 第十二批）。
ORCHESTRATOR_SHA256 = "e6f3a370ca6f66577e5cd9f3d44a30dd315660958ebb50922df3fc53ec6ef8ff"
SEGMENT_TWO_ORCHESTRATOR_SHA256 = "7dda75bcbfdca6b6705f2e0b995977d1e279044897c8a1ec717e30e1d5fcda08"
# 冻结的修订 8（rows_orchestrator_revision_8.py 的 CROSS_HOST_CHECK_SHA256）钉的跨机对照入口：载入修订 8 时同样按此刻认不认换
CROSS_HOST_CHECK_SHA256 = "4ab952419acc40890bb68e03135e1f18555dd29a5e02ff86581950f03ab297a5"
PIN_REPOSITORY_ROOT = os.path.abspath(os.path.join(HERE, "..", "..", ".."))   # 搬迁登记表从这一个仓根取
# 钉住的编排器：模块名 → 路径、模块名 → sha256 两张表（路径表照字面拼，函数单元库认得出载的是哪几份）；PINNED_REPINS 是载入时正文里要换的钉值（现算）
PINNED_PATHS = {"rows_orchestrator_1dcfb8d0": os.path.join(HERE, "rows_orchestrator.py"),
                "rows_orchestrator_segment_two_7d7b458b": os.path.join(HERE, "rows_orchestrator_segment_two.py")}
PINNED_SHA256 = {"rows_orchestrator_1dcfb8d0": ORCHESTRATOR_SHA256,
                 "rows_orchestrator_segment_two_7d7b458b": SEGMENT_TWO_ORCHESTRATOR_SHA256}


def pinned_sha256_now(path: str, pinned: str, root: str = PIN_REPOSITORY_ROOT) -> str | None:
    """钉值 pinned 的那份文件此刻认不认：此刻的 sha256 就是 pinned，或把 root 里登记过的最近几次搬迁倒着换回去之后是 pinned，交此刻的 sha256；
    都不是交 None。登记表写坏了抛 RuntimeError（调它的几层照「sha256 不对」退 3）。弄坏开关 pin-moves-ignored 下不看搬迁登记。"""
    actual = sha256_file(path)
    if actual == pinned:
        return actual
    if broken("pin-moves-ignored"):
        return None
    try:
        moves = frozen_o3.registered_moves(root)
    except ValueError as problem:
        raise RuntimeError(f"{path} 的 sha256 是 {actual}，不是 {pinned}，搬迁登记又换算不了：{problem}") from problem
    # 从最近一次起逐次多倒换一次，每一步都比，哪一步是钉值都认；此刻的内容照全部登记正着换一遍要不动，动了的（只换了一半）一步都不倒
    # （倒法与冻结执行器核对同一份：frozen_o3.RegisteredMoves.contents_before_moves）。
    if moves.moves:
        with open(path, "rb") as handle:
            data = handle.read()
        for version in moves.contents_before_moves(data):
            if hashlib.sha256(version).hexdigest() == pinned:
                return actual
    return None


def sha256_now_or_pinned(path: str, pinned: str, root: str = PIN_REPOSITORY_ROOT) -> str:
    """冻结层正文里的钉值该换成什么：此刻认（pinned_sha256_now）就是此刻的 sha256；不认还是 pinned（冻结层那一处照旧红）。"""
    return pinned_sha256_now(path, pinned, root) or pinned


def orchestrator_sha256_now(directory: str = HERE, root: str = PIN_REPOSITORY_ROOT) -> str:
    """第一段编排器（directory 里的 rows_orchestrator.py）此刻该认的 sha256：cross_host_o3_check 的模块常量 ORCHESTRATOR_SHA256 设成它。"""
    return sha256_now_or_pinned(os.path.join(directory, "rows_orchestrator.py"), ORCHESTRATOR_SHA256, root)


def orchestrator_repin(directory: str = HERE, root: str = PIN_REPOSITORY_ROOT) -> tuple[tuple[str, str], ...]:
    """冻结层正文里第一段编排器的旧钉值 → 此刻该认的那一个（rows_orchestrator_segment_two.py、rows_orchestrator_revision_8.py 载入时换）。"""
    return ((ORCHESTRATOR_SHA256_BEFORE_MOVE, orchestrator_sha256_now(directory, root)),)


def rows_revision_8_repins(directory: str = HERE, root: str = PIN_REPOSITORY_ROOT) -> tuple[tuple[str, str], ...]:
    """载入冻结的修订 8 时正文里要换的两个钉值：第一段编排器与跨机对照入口 cross_host_o3_check.py。"""
    return orchestrator_repin(directory, root) + (
        (CROSS_HOST_CHECK_SHA256, sha256_now_or_pinned(os.path.join(directory, "cross_host_o3_check.py"), CROSS_HOST_CHECK_SHA256, root)),)


PINNED_REPINS = {"rows_orchestrator_segment_two_7d7b458b": orchestrator_repin}


class RepinnedLoader(importlib.machinery.SourceFileLoader):
    """按路径载入一份冻结的脚本，正文里旧的钉值换成新的再编（文件本身一个字节不改，也不读、不写 .pyc）。"""

    def __init__(self, fullname: str, path: str, replacements: tuple[tuple[str, str], ...]):
        super().__init__(fullname, path)
        self.replacements = replacements

    def get_code(self, fullname: str) -> types.CodeType:
        text = self.get_data(self.path).decode("utf-8")
        for old, new in self.replacements:
            if old not in text:
                raise RuntimeError(f"{self.path} 里没有要换的钉值 {old}")
            text = text.replace(old, new)
        return compile(text, self.path, "exec", dont_inherit=True)


def load_repinned(path: str, expected: str, module_name: str, replacements: tuple[tuple[str, str], ...] = (),
                  root: str = PIN_REPOSITORY_ROOT) -> types.ModuleType:
    """核 path 的 sha256 是 expected（冻结的那一份；只差 root 里登记过的搬迁也认，pinned_sha256_now），再按 replacements 换掉正文里的钉值载入；
    不认抛 RuntimeError。"""
    if pinned_sha256_now(path, expected, root) is None:
        raise RuntimeError(f"{path} 的 sha256 是 {sha256_file(path)}，不是 {expected}（把登记的搬迁倒着换回去之后也不是）")
    specification = importlib.util.spec_from_file_location(module_name, path, loader=RepinnedLoader(module_name, path, tuple(replacements)))
    module = importlib.util.module_from_spec(specification)
    sys.dont_write_bytecode = True
    sys.modules[module_name] = module
    specification.loader.exec_module(module)
    return module


def pinned_module(module_name: str) -> types.ModuleType:
    """按 sha256 钉住的编排器（已经有层载入了就用那一份，不再载第二份）；sha256 不对抛 RuntimeError。"""
    if module_name in sys.modules:
        return sys.modules[module_name]
    replacements = PINNED_REPINS[module_name]() if module_name in PINNED_REPINS else ()
    return load_repinned(PINNED_PATHS[module_name], PINNED_SHA256[module_name], module_name, replacements)


def segment_one_row_units(steps: tuple[str, ...], rows_of_step: dict[str, tuple[int, ...]] | None = None) -> list[RowUnit]:
    """第一段第 11–13 步的行：flags 取第一段编排器 STEPS 那一步（row-0 另带 --empty-mutation，同 step_arguments）；s12 的 row-0 与 sx 是 RC2 的两份。"""
    orchestrator = pinned_module("rows_orchestrator_1dcfb8d0")
    units = []
    for step in steps:
        flags = orchestrator.STEPS[step].flags
        for number in (rows_of_step or orchestrator.STEP_ROWS)[step]:
            units.append(RowUnit(f"seg1-{step}-row-{number}", number, number == 0, (("--empty-mutation",) if number == 0 else ()) + tuple(flags),
                                 True, number == 0 and step in ("s12", "sx"), rerun=step in ("r11", "r12"), owner=step))
    return units


def segment_two_row_units(rows: tuple[int, ...], cross_host_row: int | None, steps: tuple[str, ...] = ("rows",)) -> list[RowUnit]:
    """第二段的行：flags 取第二段编排器那一步；跨机对照那一行在整表与跨机那一次都是 RC2 的一份。"""
    two = pinned_module("rows_orchestrator_segment_two_7d7b458b")
    units = []
    for step in steps:
        flags = {"rows": two.ROWS_STEP, "rerun": two.RERUN_STEP, "cross": two.ROWS_STEP}[step].flags
        for number in rows:
            name = f"seg2-row-{number}" if step == "rows" else f"seg2-{step}-row-{number}"
            units.append(RowUnit(name, number, False, tuple(flags), False, number == cross_host_row and step in ("rows", "cross"),
                                 rerun=step == "rerun", owner=step))
    return units


def selected_row_lines(repo: str) -> dict[int, str]:
    """两张行表的整表编号 → 那一行原文。"""
    lines = {}
    for name in SELECTED_ROW_TABLES:
        with open(os.path.join(repo, RESEARCH_RELATIVE, name), encoding="utf-8") as handle:
            for line in handle:
                fields = line.rstrip("\n").split("\t")
                if len(fields) == 6 and fields[0].isdigit():
                    lines[int(fields[0])] = line.rstrip("\n")
    return lines


# ── 一棵树上各单元的键 ──
@dataclasses.dataclass
class TreeInputs:
    """一棵树上算键要的、与单元无关的几样（每趟现算一次）。closure_files：装置 bin（device）与 E170 导出 bin（e170）各自 dep-info 列的文件。
    hypothetical_change 只给计数脚本：假想那一个文件（相对树根）改了，它在 [closure] 或 [build] 里那一行换成另一个值。"""
    repo: str
    tree: str
    closure_files: dict[str, list[str]]
    build: list[str]
    environment_names: dict[str, list[str]]
    python: str
    environment: dict[str, str]
    mutation_rows: dict[str, object]
    mutation_name_counts: dict[str, int]
    hypothetical_change: str | None = None

    def changed(self, line: str) -> str:
        return f"{line.split(' ', 1)[0]} changed-{sha256_bytes(line.encode('utf-8'))[:16]}" if line.split(" ", 1)[0] == self.hypothetical_change else line

    def closure(self, binary: str, overrides: dict[str, bytes] | None = None) -> list[str]:
        return [self.changed(line) for line in closure_lines(self.tree, self.closure_files[binary], overrides or {})]

    def build_section(self) -> list[str]:
        return [self.changed(line) for line in self.build]

    def environment_section(self, binary: str) -> list[str]:
        return environment_lines(self.environment_names[binary], self.environment)


def tree_inputs(repo: str, tree: str, dep_infos: dict[str, str], toolchain: str, python: str, environment: dict[str, str],
                build_environment: list[str], hypothetical_change: str | None = None) -> TreeInputs:
    """dep_infos：device / e170 → 那个 bin 的 dep-info（树里 target/release/<bin>.d）。树外路径抛 ClosureOutsideTheTree（S18 ②）。"""
    import bad_implementation_table  # noqa: E402（本目录）
    files = {binary: dep_info_files(tree, path) for binary, path in dep_infos.items()}
    rows = bad_implementation_table.read_mutation_rows(os.path.join(tree, "crates", "mutations.tsv"))
    counts: dict[str, int] = {}
    for row in rows:
        counts[row.name] = counts.get(row.name, 0) + 1
    return TreeInputs(repo, tree, files, build_lines(tree, toolchain, build_environment),
                      {binary: environment_names_in(tree, listed) for binary, listed in files.items()}, python, environment,
                      {row.name: row for row in rows}, counts, hypothetical_change)


def reading_unit_sections(inputs: TreeInputs, unit: ReadingUnit, upstream: list[str]) -> dict[str, list[str]]:
    """装置、E170 导出装置、执行器、变异驱动的一个读数单元的十节（upstream 是 upstream_lines 给的，或计数脚本给的上游键）。"""
    if unit.kind in ("device", "e170"):
        binary = "e170" if unit.kind == "e170" else "device"
        return {"kind": [unit.kind], "build": inputs.build_section(), "closure": inputs.closure(binary), "env": inputs.environment_section(binary),
                "args": [normalized_arguments(unit.arguments.split())], "limit": limit_lines(None), "upstream": upstream,
                "research": research_lines(inputs.repo, [os.path.join(RESEARCH_RELATIVE, name) for name in DEVICE_RESEARCH_FILES]),
                "python": [inputs.python]}
    if unit.kind == "o3-executor":
        return {"kind": [unit.kind], "args": [normalized_arguments(unit.arguments.split())], "limit": limit_lines(None), "upstream": upstream,
                "research": executor_research(inputs.repo), "python": [inputs.python]}
    if unit.kind == "mutation-driver":
        return {"kind": [unit.kind], "args": [unit.arguments], "limit": limit_lines(None),
                "research": research_lines(inputs.repo, [RESEARCH_RELATIVE, unit.arguments]), "python": [inputs.python]}
    raise ValueError(f"认不得的单元种类：{unit.kind}")


def executor_research(repo: str) -> list[str]:
    return research_lines(repo, [EXECUTOR_RELATIVE, FREEZE_REFERENCE_RELATIVE] + [os.path.join(RESEARCH_RELATIVE, name) for name in EXECUTOR_RESEARCH_FILES])


def driver_research(repo: str, executor: bool) -> list[str]:
    lines = research_lines(repo, [os.path.join(RESEARCH_RELATIVE, name) for name in DRIVER_RESEARCH_FILES])
    return lines + executor_research(repo) if executor else lines


def row_sections(inputs: TreeInputs, row: RowUnit, selected_lines: dict[int, str], limit: list[str], upstream: list[str]) -> tuple[dict[str, list[str]], str]:
    """驱动的一行的十节（第 11.2、11.3 节），与永远现跑的缘由（不是就交回空串）：second-run（RC2）、not-located（按名字取不到或不唯一）、
    anchor-not-once（锚点不是恰 1 次）、touches-module-set（改坏那一处碰到模块集合，dep-info 认不全）。后三样要补位，读别的行用过的补位名。"""
    import bad_implementation_table  # noqa: E402（本目录）
    selected = selected_lines.get(row.number)
    name = (selected_lines[1].split("\t")[4] if row.empty else selected.split("\t")[4]) if (row.empty or selected) else None
    overrides: dict[str, bytes] = {}
    reason = "second-run" if row.second_run else ""
    mutation = inputs.mutation_rows.get(name) if name else None
    if mutation is None or inputs.mutation_name_counts.get(name, 0) != 1:
        mutation_lines = [f"row {row.number}", f"selected {selected or '-'}", "mutation not-located"]
        reason = reason or "not-located"
    else:
        text = open(os.path.join(inputs.tree, mutation.file), encoding="utf-8").read()
        applied = bad_implementation_table.empty_mutation(mutation) if row.empty else mutation
        if bad_implementation_table.anchor_hit_count(text, applied) != 1:
            reason = reason or "anchor-not-once"
        else:
            overrides[mutation.file] = bad_implementation_table.apply_mutation(text, applied).encode("utf-8")
        if any(token in mutation.original_text() + mutation.replacement_text() for token in MODULE_SET_TOKENS):
            reason = reason or "touches-module-set"
        mutation_lines = [f"row {row.number}", f"selected {'empty-of ' + name if row.empty else selected}", f"mutation {mutation.raw_line}"]
    sections = {"kind": ["driver-row"], "build": inputs.build_section(), "closure": inputs.closure("device", overrides),
                "env": inputs.environment_section("device"), "mutation": mutation_lines, "args": [normalized_arguments(list(row.flags))],
                "limit": limit, "upstream": upstream, "research": driver_research(inputs.repo, row.executor), "python": [inputs.python]}
    return sections, reason


def mutated_closure_paths(inputs: TreeInputs) -> tuple[str, ...]:
    """RC4 比的路径集合：装置 bin 的 dep-info 列的那一组（改坏只改内容、不改文件集合）。"""
    return tuple(inputs.closure_files["device"])


# ── 复用库（第 11.5 节）──
class RunCondition(Exception):
    """库目录写不了，或与 scripts/experiment-apparatus/target 不在同一个文件系统：退 78。"""


class CorruptedEntry(Exception):
    """RC5：条目的读数与 manifest.sha256 对不上（已挪进隔离目录）。"""


def link_or_copy(source: str, destination: str) -> None:
    try:
        os.link(source, destination)
    except OSError:
        shutil.copy2(source, destination)


def link_reading(source: str, destination: str, last_file: str | None = None) -> None:
    """文件硬链接过去；目录逐个文件硬链接（last_file 那一个最后链：行目录的 row-result.tsv）。目标已在就拒。"""
    if os.path.lexists(destination):
        raise FileExistsError(f"已存在、不覆盖：{destination}")
    if os.path.isfile(source):
        os.makedirs(os.path.dirname(destination), exist_ok=True)
        link_or_copy(source, destination)
        return
    inner = files_under(source)
    for relative in sorted(inner, key=lambda path: path == last_file):
        if relative == last_file:
            stop_here_if_asked(f"before-row-result:{os.path.basename(destination)}")
        os.makedirs(os.path.dirname(os.path.join(destination, relative)), exist_ok=True)
        link_or_copy(os.path.join(source, relative), os.path.join(destination, relative))
    os.makedirs(destination, exist_ok=True)


def manifest_text(entry: str) -> str:
    reading = os.path.join(entry, "reading")
    return "".join(f"{sha256_file(os.path.join(reading, relative))}  reading/{relative}\n" for relative in files_under(reading))


def manifest_problems(entry: str) -> list[str]:
    """RC5：manifest.sha256 在、列的每个文件 sha256 对、reading/ 下不多不少。交回不符的那几样（空表是好的）。"""
    path = os.path.join(entry, "manifest.sha256")
    if not os.path.isfile(path):
        return ["manifest-missing"]
    listed, problems = {}, []
    for line in open(path, encoding="utf-8"):
        digest, relative = line.rstrip("\n").split("  ", 1)
        listed[relative] = digest
    present = {f"reading/{relative}" for relative in files_under(os.path.join(entry, "reading"))}
    problems += [f"extra:{relative}" for relative in sorted(present - set(listed))]
    for relative, digest in sorted(listed.items()):
        full = os.path.join(entry, relative)
        if not os.path.isfile(full):
            problems.append(f"missing:{relative}")
        elif sha256_file(full) != digest:
            problems.append(f"sha256:{relative}")
    return problems


def entry_problems(entry: str) -> list[str]:
    """取用之前与查库时都走这一处（RC5）。"""
    if broken("manifest-unchecked"):
        return []
    problems = manifest_problems(entry)
    return problems


def read_meta(entry: str) -> dict[str, str]:
    path = os.path.join(entry, "meta.tsv")
    return dict(line.rstrip("\n").split("\t", 1) for line in open(path, encoding="utf-8") if "\t" in line) if os.path.exists(path) else {}


def entry_reading(entry: str) -> dict[str, str]:
    reading = os.path.join(entry, "reading")
    return {role: os.path.join(reading, role) for role in sorted(os.listdir(reading))} if os.path.isdir(reading) else {}


class Library:
    """H30 那个目录。frozen-by-s18 在时不取、不入（S18：库冻住，直到主 agent 查出漏的输入、键 schema 加一或另判）。"""

    def __init__(self, root: str) -> None:
        self.root = os.path.abspath(os.path.expanduser(root))
        self.entries, self.staging, self.quarantine_directory = (os.path.join(self.root, name) for name in ("entries", "staging", "quarantine"))

    def entry_path(self, key: str) -> str:
        return os.path.join(self.entries, key[:2], key)

    def frozen_reason(self) -> str | None:
        path = os.path.join(self.root, FROZEN_FILE)
        return open(path, encoding="utf-8").read().strip() or "frozen" if os.path.exists(path) else None

    def freeze(self, reason: str) -> None:
        os.makedirs(self.root, exist_ok=True)
        with open(os.path.join(self.root, FROZEN_FILE), "a", encoding="utf-8") as handle:
            handle.write(reason.replace("\n", " ") + "\n")

    def check_run_condition(self, repo: str) -> None:
        """运行条件（第 11.9 节）：库目录写得了、与 scripts/experiment-apparatus/target 同一个文件系统（硬链接成得了）。"""
        try:
            for directory in (self.entries, self.staging, self.quarantine_directory):
                os.makedirs(directory, exist_ok=True)
            probe = tempfile.NamedTemporaryFile(dir=self.staging, delete=True)
            probe.close()
        except OSError as error:
            raise RunCondition(f"复用库 {self.root} 写不了：{error}") from error
        target = os.path.join(repo, "scripts", "experiment-apparatus", "target")
        if os.stat(self.root).st_dev != os.stat(target).st_dev:
            raise RunCondition(f"复用库 {self.root} 与 {target} 不在同一个文件系统（硬链接不成）")

    def clear_stale_staging(self) -> list[str]:
        """起跑时删 staging/ 下的残留：只删进程已经不在的那几个（几层编排同时用一个库，别人正写的不动）。"""
        removed = []
        for name in sorted(os.listdir(self.staging)) if os.path.isdir(self.staging) else []:
            process = name.split("-", 1)[0]
            if process.isdigit() and os.path.exists(f"/proc/{process}"):
                continue
            shutil.rmtree(os.path.join(self.staging, name), ignore_errors=True)
            removed.append(name)
        return removed

    def quarantine(self, entry: str) -> str:
        os.makedirs(self.quarantine_directory, exist_ok=True)
        destination = os.path.join(self.quarantine_directory, f"{os.path.basename(entry)}-{time.time_ns()}")
        os.rename(entry, destination)
        return destination

    def lookup(self, key: str) -> tuple[str | None, str, list[str]]:
        """交回 (条目路径, hit|miss|corrupted|frozen, RC5 不符的那几样)。不符的条目挪进隔离目录。"""
        if self.frozen_reason():
            return None, "frozen", []
        path = self.entry_path(key)
        if not os.path.isdir(path):
            return None, "miss", []
        problems = entry_problems(path)
        if problems:
            self.quarantine(path)
            return None, "corrupted", problems
        return path, "hit", []

    def insert(self, key: str, text: str, kind: str, reading: dict[str, str], meta: dict[str, str]) -> tuple[str, list[str]]:
        """原子入库：先写进 staging/<进程号>-<随机>/（key.txt、reading/<角色>、meta.tsv，manifest.sha256 最后写），再 rename 到 entries/…/<键>/。
        目标已在（别人先入了）就不覆盖、照 RC1 比读数。交回 (inserted|present-identical|present-differs|frozen, 不同的那几样)。"""
        if self.frozen_reason():
            return "frozen", []
        target = self.entry_path(key)
        if os.path.isdir(target):
            if entry_problems(target):
                self.quarantine(target)
            else:
                return self.compare_with(target, kind, reading)
        building = os.path.join(self.staging, f"{os.getpid()}-{secrets.token_hex(8)}")
        if broken("insert-in-place"):
            building = target
        os.makedirs(os.path.join(building, "reading"))
        with open(os.path.join(building, "key.txt"), "w", encoding="utf-8") as handle:
            handle.write(text)
        for role, source in sorted(reading.items()):
            link_reading(source, os.path.join(building, "reading", role), "row-result.tsv" if role == "row" else None)
        with open(os.path.join(building, "meta.tsv"), "w", encoding="utf-8") as handle:
            handle.writelines(f"{name}\t{value}\n" for name, value in sorted(meta.items()))
        stop_here_if_asked("before-manifest")
        with open(os.path.join(building, "manifest.sha256"), "w", encoding="utf-8") as handle:
            handle.write(manifest_text(building))
        stop_here_if_asked("before-rename")
        if building == target:
            return "inserted", []
        os.makedirs(os.path.dirname(target), exist_ok=True)
        try:
            os.rename(building, target)
        except OSError:
            shutil.rmtree(building, ignore_errors=True)
            return self.compare_with(target, kind, reading)
        return "inserted", []

    def compare_with(self, entry: str, kind: str, reading: dict[str, str]) -> tuple[str, list[str]]:
        """RC1–RC3 的比法：条目里的读数与这一次的。"""
        if broken("check-skipped"):
            return "present-identical", []
        differing = reading_differences(kind, entry_reading(entry), reading)
        return ("present-differs" if differing else "present-identical"), differing


def fetch(library: Library, entry: str, targets: dict[str, str]) -> dict[str, str]:
    """取用：核 RC5（不符的挪进隔离目录、抛 CorruptedEntry）→ 按角色硬链接到这一趟的名字（目标已在就拒）。交回条目的 meta。"""
    problems = entry_problems(entry)
    if problems:
        library.quarantine(entry)
        raise CorruptedEntry(f"条目 {entry} 与 manifest.sha256 对不上：{','.join(problems[:4])}")
    stored = entry_reading(entry)
    missing = sorted(set(targets) - set(stored))
    if missing:
        raise CorruptedEntry(f"条目 {entry} 里没有角色 {','.join(missing)}")
    for role, target in sorted(targets.items()):
        link_reading(stored[role], target, "row-result.tsv" if role == "row" else None)
    return read_meta(entry)


# ── 读数相等（第 11.5 节「读数相等」）──
def lines_without_the_last_column(path: str) -> bytes:
    """impl-verdicts.tsv 去掉耗时那一列（同 Z1，segment_one_comparator.py 的同名一比）。"""
    with open(path, encoding="utf-8") as handle:
        return "".join(line.rstrip("\n").rsplit("\t", 1)[0] + "\n" for line in handle).encode("utf-8")


def unstable_return_value_differences(first_directory: str, second_directory: str) -> tuple[bool, str]:
    """call-return-values.tsv 照读法 4 拿掉两次就不稳的内存对象之后比（同 Z1：collusion_matrix 的两个函数）。交回 (相同, 拿掉的名字)。
    导出目录读不成（例如第二段只导两条流）就交回 (False, 读不成的缘由)，按不同算（收严）。"""
    import collusion_matrix  # noqa: E402（本目录）
    import run_directory  # noqa: E402
    try:
        first, second = run_directory.load_run_directory(first_directory), run_directory.load_run_directory(second_directory)
    except Exception as error:  # noqa: BLE001 —— 读不成按不同算
        return False, f"load-failed:{type(error).__name__}"
    unstable = collusion_matrix.unstable_memory_objects(first, second)
    present = tuple(name for name in run_directory.STREAM_NAMES if os.path.isdir(os.path.join(first_directory, name)))
    return not collusion_matrix.return_value_differences(first, second, unstable, present), ",".join(sorted(unstable)) or "-"


def directory_differences(first: str, second: str, label: str) -> list[str]:
    """导出目录每个文件逐字节比（impl-verdicts.tsv 去耗时列、call-return-values.tsv 照读法 4）；文件集合也要相同。"""
    first_files, second_files = set(files_under(first)), set(files_under(second))
    differing = [f"{label}/{relative}:only-one-side" for relative in sorted(first_files ^ second_files)]
    same_without_unstable_objects: dict[str, bool] = {}  # 一次跑的导出根（流目录的上一级）→ 拿掉不稳的内存对象之后相同否
    for relative in sorted(first_files & second_files):
        left, right = os.path.join(first, relative), os.path.join(second, relative)
        if sha256_file(left) == sha256_file(right):
            continue
        if os.path.basename(relative) == "impl-verdicts.tsv" and lines_without_the_last_column(left) == lines_without_the_last_column(right):
            continue
        if os.path.basename(relative) == "call-return-values.tsv":
            run_root = os.path.dirname(os.path.dirname(relative))
            if run_root not in same_without_unstable_objects:
                same_without_unstable_objects[run_root] = unstable_return_value_differences(os.path.join(first, run_root), os.path.join(second, run_root))[0]
            if same_without_unstable_objects[run_root]:
                continue
        differing.append(f"{label}/{relative}")
    return differing


def normalized_o3_lines(path: str) -> list[str]:
    """o3.out 与执行器产物照 cross_host_o3_check.py 那一套去法：编排器的 normalized_o3_lines（去 name=config、name=state_time 与 *_ns），
    再去三个调度键、按整行排序（cross_host_o3_check.o3_lines_without_scheduling，调它的函数）。"""
    import cross_host_o3_check  # noqa: E402（本目录）
    directory = tempfile.mkdtemp(prefix="reuse-o3-")
    try:
        shutil.copyfile(path, os.path.join(directory, "o3.out"))
        return cross_host_o3_check.o3_lines_without_scheduling(pinned_module("rows_orchestrator_1dcfb8d0"), directory)
    finally:
        shutil.rmtree(directory, ignore_errors=True)


def row_result_without_volatile_keys(path: str) -> list[str]:
    return [line for line in open(path, encoding="utf-8").read().splitlines() if line.split("\t", 1)[0] not in ROW_RESULT_VOLATILE_KEYS]


def device_result_lines(path: str) -> list[str]:
    return [line for line in open(path, encoding="utf-8").read().splitlines() if line.startswith("E7RESULT ")] if os.path.exists(path) else []


def reading_differences(kind: str, first: dict[str, str], second: dict[str, str]) -> list[str]:
    """两份读数（角色 → 路径）不同的那几样，空表是相同。计时（timing）不比。
    装置、E170：产物全文、导出目录每个文件；驱动行：row-result.tsv 去易变键、device.out 的 E7RESULT 行、导出每个文件、o3.out 与 o3-control.out 照去法；
    执行器：产物照 o3 去法；变异驱动：产物全文（它的结果行不带耗时）。"""
    differing = [f"{role}:only-one-side" for role in sorted((set(first) ^ set(second)) - {"timing"})]
    for role in sorted((set(first) & set(second)) - {"timing"}):
        left, right = first[role], second[role]
        if role == "directory":
            differing += directory_differences(left, right, role)
        elif role == "row":
            differing += row_differences(left, right)
        elif kind == "o3-executor":
            differing += [] if normalized_o3_lines(left) == normalized_o3_lines(right) else [role]
        else:
            differing += [] if sha256_file(left) == sha256_file(right) else [role]
    return differing


def row_differences(first: str, second: str) -> list[str]:
    differing = []
    if row_result_without_volatile_keys(os.path.join(first, "row-result.tsv")) != row_result_without_volatile_keys(os.path.join(second, "row-result.tsv")):
        differing.append("row/row-result.tsv")
    if device_result_lines(os.path.join(first, "device.out")) != device_result_lines(os.path.join(second, "device.out")):
        differing.append("row/device.out")
    for name in ("o3.out", "o3-control.out"):
        left, right = os.path.join(first, name), os.path.join(second, name)
        if os.path.exists(left) != os.path.exists(right) or (os.path.exists(left) and normalized_o3_lines(left) != normalized_o3_lines(right)):
            differing.append(f"row/{name}")
    left_dump, right_dump = os.path.join(first, "dump"), os.path.join(second, "dump")
    if os.path.isdir(left_dump) or os.path.isdir(right_dump):
        differing += directory_differences(left_dump if os.path.isdir(left_dump) else first + "/.absent",
                                           right_dump if os.path.isdir(right_dump) else second + "/.absent", "row/dump")
    return differing


# ── 入库条件（第 11.5 节）──
def device_exit_insertable(exit_code: int, expected_exits: tuple[int, ...]) -> tuple[bool, str]:
    """装置、执行器的退出码：超时（253，挂钟）与包装自己的结局（250–252、254，Z3）不入；别的要在那一步预期的集合里。"""
    timed_out = exit_code == TIMEOUT_EXIT_CODE  # 先取值再判：同一目录里第二段变异表 W7 的锚点是比对器里那一行 if 判断，这里不能逐字再出现一次
    if timed_out:
        return False, "timeout"
    if exit_code in WRAPPER_OWN_EXIT_CODES:
        return False, f"wrapper-outcome-{exit_code}"
    return (True, "") if exit_code in expected_exits else (False, f"unexpected-exit-{exit_code}")


def row_result_fields(row_directory: str) -> dict[str, str]:
    path = os.path.join(row_directory, "row-result.tsv")
    return dict(line.rstrip("\n").split("\t", 1) for line in open(path, encoding="utf-8") if "\t" in line) if os.path.exists(path) else {}


def row_insertable(row_directory: str) -> tuple[bool, str]:
    """驱动行：有 row-result.tsv、status=ok、没补位（substituted_from=-、没有试过而没用上的名字）、核过重编（packages_checked_rebuilt）、
    装置退出码照 device_exit_insertable（0 或 3）。"""
    fields = row_result_fields(row_directory)
    if not fields:
        return False, "no-row-result"
    if fields.get("status") != "ok":
        return False, f"status-{fields.get('status')}"
    if fields.get("substituted_from", "-") != "-" or fields.get("failed_attempts", "0") not in ("0", ""):
        return False, "substituted"
    if REBUILD_CHECK_FIELD not in fields:
        return False, "rebuild-not-checked"
    exit_text = fields.get("device_exit", "-")
    if not exit_text.lstrip("-").isdigit():
        return False, f"device-exit-{exit_text}"
    return device_exit_insertable(int(exit_text), (0, 3))


# ── 决定、标记、事件（第 11.4、11.5 节）──
@dataclasses.dataclass
class Decision:
    """一个单元这一趟怎么做：state 是 fresh（现跑、查过库没有）、reused（从库里取）、fresh-check（RC3 抽到、现跑后与条目比）、fresh-always（永远现跑）。"""
    unit: str
    kind: str
    key: str
    text: str
    state: str
    reason: str
    entry: str = "-"
    meta: dict = dataclasses.field(default_factory=dict)


# 这几种缘由的单元现跑完也不入库（读数依赖别的行与次序，或环境变量设了）
NEVER_INSERTED_REASONS = ("env-set", "not-located", "anchor-not-once", "touches-module-set")


def decide(library: Library | None, unit: str, kind: str, sections: dict[str, list[str]], always_reason: str, lookup_enabled: bool) -> tuple[Decision, list[str]]:
    """查库（第 11.2、11.4 节）。交回 (决定, RC5 不符的那几样：条目坏了时非空，调用方记一行 reuse_corrupted)。"""
    text = key_text(sections)
    key = sha256_bytes(text.encode("utf-8"))
    environment_set = any(line.endswith(" set") for line in sections.get("env", []))
    if always_reason and not broken("fresh-always-ignored"):
        return Decision(unit, kind, key, text, STATE_FRESH_ALWAYS, always_reason), []
    if environment_set:
        return Decision(unit, kind, key, text, STATE_FRESH, "env-set"), []
    if library is None or not lookup_enabled:
        return Decision(unit, kind, key, text, STATE_FRESH, "lookup-off"), []
    entry, outcome, problems = library.lookup(key)
    if outcome == "hit":
        return Decision(unit, kind, key, text, STATE_REUSED, "hit", entry, read_meta(entry)), []
    return Decision(unit, kind, key, text, STATE_FRESH, {"miss": "no-entry", "corrupted": "corrupted-entry", "frozen": "library-frozen"}[outcome]), problems


def marker_path(markers: str, unit: str) -> str:
    return os.path.join(markers, re.sub(r"[^A-Za-z0-9._-]", "_", unit) + ".tsv")


def write_marker(markers: str, decision: Decision, extra: dict[str, str] | None = None) -> None:
    """单元的标记（先写标记、再链接）：原子写（同目录临时文件再 os.replace）。"""
    os.makedirs(markers, exist_ok=True)
    fields = {"unit": decision.unit, "kind": decision.kind, "key": decision.key, "state": decision.state, "reason": decision.reason,
              "entry": decision.entry, "origin_run": decision.meta.get("origin_run", "-"), "origin_product": decision.meta.get("origin_product", "-"),
              "origin_host": decision.meta.get("origin_host", "-")}
    fields.update(extra or {})
    path = marker_path(markers, decision.unit)
    partial = f"{path}.{os.getpid()}.partial"
    with open(partial, "w", encoding="utf-8") as handle:
        handle.writelines(f"{name}\t{value}\n" for name, value in fields.items())
    os.replace(partial, path)


def read_markers(markers: str) -> dict[str, dict[str, str]]:
    found = {}
    for name in sorted(os.listdir(markers)) if os.path.isdir(markers) else []:
        if name.endswith(".tsv") and name != "events.tsv":
            fields = dict(line.rstrip("\n").split("\t", 1) for line in open(os.path.join(markers, name), encoding="utf-8") if "\t" in line)
            if "unit" in fields:
                found[fields["unit"]] = fields
    return found


def value(text: object) -> str:
    """结果行里的一个值：空白换成下划线，空串写 -（同 result_lines.field）。"""
    return "".join("_" if character.isspace() else character for character in str(text)) or "-"


def append_event(markers: str, line: str) -> None:
    os.makedirs(markers, exist_ok=True)
    with open(os.path.join(markers, "events.tsv"), "a", encoding="utf-8") as handle:
        handle.write(line + "\n")
        handle.flush()
        os.fsync(handle.fileno())


def note_decision(markers: str, decision: Decision, problems: list[str]) -> None:
    """写标记；条目坏了（RC5）另记一行事件。"""
    if problems:
        append_event(markers, f"E7RESULT name=reuse_corrupted unit={value(decision.unit)} key={decision.key} problems={value(','.join(problems[:6]))}")
    write_marker(markers, decision)


def record_finished(library: Library | None, markers: str, decision: Decision, reading: dict[str, str], insertable: tuple[bool, str],
                    meta: dict[str, str]) -> str:
    """一个现跑完的单元（取来的不进这里）：照入库条件入库；库里已有同键条目时比读数——普通单元是 RC1（kind=insert），永远现跑的第二趟是 RC2
    （kind=second-run），RC3 抽到的是 kind=sample。交回入库的结局（not-inserted 也是一种）。"""
    if decision.state == STATE_REUSED:
        return "not-applicable"
    ok, why = insertable
    if decision.reason in NEVER_INSERTED_REASONS:
        ok, why = False, decision.reason
    if library is None or not ok:
        state, differing = "not-inserted", []
    else:
        meta = dict(meta, unit=decision.unit, kind=decision.kind, inserted=time.strftime("%Y-%m-%d"))
        state, differing = library.insert(decision.key, decision.text, decision.kind, reading, meta)
    if state in ("present-identical", "present-differs"):
        check_kind = {STATE_FRESH_ALWAYS: "second-run", STATE_FRESH_CHECK: "sample"}.get(decision.state, "insert")
        append_event(markers, f"E7RESULT name=reuse_check kind={check_kind} unit={value(decision.unit)} key={decision.key} "
                              f"identical={str(state == 'present-identical').lower()} differing={value(','.join(differing[:12]))}")
    append_event(markers, f"E7RESULT name=reuse_insert unit={value(decision.unit)} key={decision.key} state={state} reason={value(why)}")
    return state


def closure_check(markers: str, decision: Decision, compiled_tree: str, dep_info: str, key_paths: tuple[str, ...]) -> bool:
    """RC4：现跑编译的单元编完，读那一次的 dep-info（相对编译用的那棵树），路径集合必须等于键里 [closure] 的；树外的路径也算不符。记一行事件，交回相符否。"""
    if broken("closure-check-skipped"):
        return True
    try:
        compiled = set(dep_info_files(compiled_tree, dep_info))
        detail = f"extra={value(','.join(sorted(compiled - set(key_paths))[:6]))} missing={value(','.join(sorted(set(key_paths) - compiled)[:6]))}"
        matches = compiled == set(key_paths)
    except ClosureOutsideTheTree as error:
        matches, detail = False, f"outside-the-tree={value(str(error)[-120:])}"
    except OSError as error:
        matches, detail = False, f"unreadable={value(type(error).__name__)}"
    append_event(markers, f"E7RESULT name=reuse_closure_check unit={value(decision.unit)} key={decision.key} matches={str(matches).lower()} {detail}")
    return matches


def event_lines(markers: str) -> list[str]:
    path = os.path.join(markers, "events.tsv")
    return open(path, encoding="utf-8").read().splitlines() if os.path.exists(path) else []


def s18_problems(markers: str) -> list[str]:
    """S18（第 11.9 节）：① RC1–RC3 任一次 identical=false；② RC4 不符。交回触发的那几行事件。"""
    return [line for line in event_lines(markers) if (" name=reuse_check " in f" {line} " and " identical=false " in f" {line} ")
            or (" name=reuse_closure_check " in f" {line} " and " matches=false " in f" {line} ")]


def ledger_text(markers: str, registered: list[str]) -> tuple[str, bool]:
    """复用账本（第 11.4 节）：登记的每个单元一行 name=reuse，事件原样，末行 name=verdict。交回 (全文, N + M 等于登记的单元数否)。"""
    found = read_markers(markers)
    counts = {STATE_FRESH: 0, STATE_REUSED: 0, STATE_FRESH_CHECK: 0, STATE_FRESH_ALWAYS: 0}
    lines = [f"E7RESULT name=reuse_columns registered_units={len(registered)} states=fresh,reused,fresh-check,fresh-always"]
    environment_set = 0
    for unit in registered:
        marker = found.get(unit)
        if marker is None:
            continue
        state = marker["state"]
        counts[state] += 1
        environment_set += marker.get("reason") == "env-set"
        lines.append(f"E7RESULT name=reuse unit={value(unit)} key={marker['key']} state={state} reason={value(marker.get('reason'))} "
                     f"entry={value(marker.get('entry'))} origin_run={value(marker.get('origin_run'))} origin_product={value(marker.get('origin_product'))} "
                     f"origin_host={value(marker.get('origin_host'))}")
    events = event_lines(markers)
    lines += events
    judged_fresh = counts[STATE_FRESH] + counts[STATE_FRESH_CHECK] + counts[STATE_FRESH_ALWAYS]
    missing = [unit for unit in registered if unit not in found]
    unregistered = sorted(set(found) - set(registered))
    sample_sizes = [int(line.split(" k=", 1)[1].split()[0]) for line in events if " name=reuse_sample " in f" {line} "]
    covers = judged_fresh + counts[STATE_REUSED] == len(registered)
    lines.append(f"E7RESULT name=verdict units={judged_fresh + counts[STATE_REUSED]} judged_fresh={judged_fresh} reused={counts[STATE_REUSED]} "
                 f"fresh_check={counts[STATE_FRESH_CHECK]} fresh_always={counts[STATE_FRESH_ALWAYS]} sample_size={sample_sizes[-1] if sample_sizes else 0} "
                 f"check_mismatches={sum(1 for line in events if ' name=reuse_check ' in f' {line} ' and ' identical=false ' in f' {line} ')} "
                 f"closure_mismatches={sum(1 for line in events if ' name=reuse_closure_check ' in f' {line} ' and ' matches=false ' in f' {line} ')} "
                 f"corrupted_entries={sum(1 for line in events if ' name=reuse_corrupted ' in f' {line} ')} env_set_units={environment_set} "
                 f"registered_units={len(registered)} units_cover_registered={str(covers).lower()} missing={value(','.join(missing[:8]))} "
                 f"unregistered={value(','.join(unregistered[:8]))}")
    return "\n".join(lines) + "\n", covers


def install_ledger(path: str, text: str) -> str:
    """账本装进 research/results：新文件才写；已存在而逐字节相同算装过；不同就不覆盖、写在旁边的 .differs-<纳秒> 里交主 agent。"""
    if os.path.exists(path):
        if open(path, encoding="utf-8").read() == text:
            return path
        path = f"{path}.differs-{time.time_ns()}"
    with open(path, "x", encoding="utf-8") as handle:
        handle.write(text)
    return path


# ── 两份逐行编排共用：派发之前取来的行（第 11.5 节「取用」「与分片、续跑怎么叠」）──
def fetch_rows(library: Library | None, markers: str, decisions: list[tuple[Decision, str]]) -> tuple[list[str], list[Decision]]:
    """decisions：(这一行的决定, 它在行输出根下的行目录)。行目录已有 row-result.tsv 的是做完了的（这一趟或更早几趟，现跑的、取来的都算），
    标记不动；别的写标记，reused 的再按角色 row 硬链接进去（row-result.tsv 最后链）。标记写着 reused 而行目录没有 row-result.tsv 的
    （上一次断在取用中间），删掉已链的几份重取。取用时条目坏了（RC5）那一行改成现跑。交回 (取来的行名, 改成现跑的决定)。"""
    fetched, demoted = [], []
    known = read_markers(markers)
    for decision, row_directory in decisions:
        if os.path.exists(os.path.join(row_directory, "row-result.tsv")):
            if decision.unit not in known:
                write_marker(markers, dataclasses.replace(decision, state=STATE_FRESH, reason="finished-before-the-marker", entry="-", meta={}))
            continue
        if decision.state != STATE_REUSED:
            write_marker(markers, decision)
            continue
        if os.path.isdir(row_directory):
            shutil.rmtree(row_directory)
        write_marker(markers, decision)
        try:
            fetch(library, decision.entry, {"row": row_directory})
        except CorruptedEntry as error:
            shutil.rmtree(row_directory, ignore_errors=True)
            changed = dataclasses.replace(decision, state=STATE_FRESH, reason="corrupted-entry", entry="-", meta={})
            note_decision(markers, changed, [str(error)[-200:]])
            demoted.append(changed)
            continue
        fetched.append(decision.unit)
    return fetched, demoted


def rows_sample(decisions: list[Decision], suffix: str) -> list[str]:
    """RC3：取来的行数 M ≥ 1 时挑 k = max(3, ⌈0.02 × M⌉) 行现跑（不够 k 行就全挑），挑法按 sha256(后缀 ‖ 键) 从小到大取前 k 个。交回挑中的行名。"""
    reused = [decision for decision in decisions if decision.state == STATE_REUSED]
    if not reused:
        return []
    size = max(3, math.ceil(0.02 * len(reused)))
    ordered = sorted(reused, key=lambda decision: (sha256_bytes((suffix + decision.key).encode("utf-8")), decision.unit))
    return [decision.unit for decision in ordered[:size]]


def tree_inputs_for_run(repo: str, tree: str, environment: dict[str, str]) -> TreeInputs:
    """三层编排起跑时现算一次：dep-info 取树里 target/release/<bin>.d（H7 那次编译），工具链在树里现跑，构建环境照 admission.py。
    自证里工具链与构建环境换成 REUSE_SELFTEST_TOOLCHAIN、REUSE_SELFTEST_BUILD_ENVIRONMENT（假树不在真仓里，admission.py 也不在）。"""
    absolute_tree = os.path.join(repo, tree) if not os.path.isabs(tree) else tree
    dep_infos = {"device": os.path.join(absolute_tree, "target/release", DEVICE_BINARY + ".d"),
                 "e170": os.path.join(absolute_tree, "target/release", E170_BINARY + ".d")}
    toolchain = selftest_only("REUSE_SELFTEST_TOOLCHAIN") or toolchain_text(absolute_tree)
    build_environment = [selftest_only("REUSE_SELFTEST_BUILD_ENVIRONMENT")] if selftest_only("REUSE_SELFTEST_BUILD_ENVIRONMENT") \
        else build_environment_lines(repo, absolute_tree)
    return tree_inputs(repo, absolute_tree, dep_infos, toolchain, python_text(), dict(environment), build_environment)


def key_section_values(text: str, section: str) -> list[str]:
    """键文本里一节的各行。"""
    lines, inside = [], False
    for line in text.splitlines():
        if line.startswith("["):
            inside = line == f"[{section}]"
        elif inside:
            lines.append(line)
    return lines


def peer_row_problems(row_directory: str, row_number: int, decision_text: str, shard_product_text: str, frozen_snapshot_digest: str) -> list[str]:
    """别的台取回的行入库之前另核三样（第 11.5 节「两台」）：行结果里 row_number 与 mutation_line_sha256 对得上键里 [mutation]；那一分片的
    row_time_limit 秒数等于键里的 L；那一分片的 snapshot_digest 等于发给别的台的冻结树快照的摘要（合并规则 ② 比的那一份）。交回不过的那几样。"""
    fields = row_result_fields(row_directory)
    problems = [] if fields.get("row_number") == str(row_number) else ["row_number"]
    raw_lines = [line[len("mutation "):] for line in key_section_values(decision_text, "mutation") if line.startswith("mutation ") and line != "mutation not-located"]
    if raw_lines and fields.get("mutation_line_sha256") != sha256_bytes(raw_lines[0].encode("utf-8")):
        problems.append("mutation_line_sha256")
    limit = re.search(r"^name=row_time_limit seconds=(\d+)", shard_product_text, re.MULTILINE)
    if not limit or f"seconds {limit.group(1)}" not in key_section_values(decision_text, "limit"):
        problems.append("row_time_limit")
    snapshot = re.search(r"^snapshot_digest=(\S+)", shard_product_text, re.MULTILINE)
    if not snapshot or snapshot.group(1) != frozen_snapshot_digest:
        problems.append("snapshot_digest")
    return problems


def h31_problems(repo: str, config: dict) -> list[str]:
    """H31（第 11.10 节新文件的 sha256，主 agent 认）：没填、文件不在、sha256 对不上的那几样。三层编排起跑之前都核（路径不是绝对的相对仓根）。"""
    files = (config.get("H31") or {}).get("files")
    if not files:
        return ["H31 没填（主 agent 认过新文件的 sha256 之前不起跑）"]
    problems = []
    for path, digest in sorted(files.items()):
        full = path if os.path.isabs(path) else os.path.join(repo, path)
        actual = sha256_file(full) if os.path.isfile(full) else "missing"
        if actual != digest:
            problems.append(f"{path}（记的 {digest[:12]}，现在 {actual[:12]}）")
    return problems


def readings_file_lines(path: str) -> list[tuple[str, str, str, dict[str, str]]]:
    """单元脚本记的 REUSE_READINGS：每行 步名、fresh|reused、退出码、角色=路径…。交回 [(步名, 状态, 退出码, 角色 → 路径)]。"""
    found = []
    for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
        fields = line.rstrip("\n").split("\t")
        if len(fields) >= 3:
            found.append((fields[0], fields[1], fields[2], dict(field.split("=", 1) for field in fields[3:] if "=" in field)))
    return found


# ── 命令行：单元脚本取用、RP2 ──
def command_fetch(arguments: argparse.Namespace) -> int:
    targets = {}
    for mapping in arguments.map:
        role, _, path = mapping.partition("=")
        if not path:
            print(f"--map 要写成 <角色>=<目标路径>：{mapping}", file=sys.stderr)
            return 2
        targets[role] = os.path.abspath(path)
    try:
        meta = fetch(Library(arguments.library), arguments.entry, targets)
    except (CorruptedEntry, FileExistsError) as error:
        print(f"✗ 取用不成：{error}", file=sys.stderr)
        print('     → 怎么办：目标已在就先挪走再取；库里那一条坏了就删掉那一条，让下一趟重算', file=sys.stderr)
        return 1
    print(meta.get("exit", "0"))
    return 0


def selfproof_run(repo: str, binary: str, threads: str, memory_cap: str, directory: str) -> tuple[int, str]:
    """RP2 的一次：装置 stream-constants --streams 1,2,3（只打 S1 常量），经内存包装与线程上限；产物是它的 E7RESULT 行。"""
    os.makedirs(directory)
    product = os.path.join(directory, "stream-constants.out")
    command = ["nice", "-n", "19", "bash", os.path.join(repo, "scripts/hosts/run-with-memory-cap.sh"), memory_cap, "bash",
               os.path.join(repo, "scripts/hosts/capped.sh"), threads, binary, "stream-constants", "--streams", "1,2,3"]
    completed = subprocess.run(command, cwd=repo, capture_output=True, text=True,
                               env=dict(os.environ, RUN_WITH_MEMORY_CAP_PARALLEL_REASON="同一会话派出去的几个执行员各跑各的装置"))
    with open(product, "x", encoding="utf-8") as handle:
        handle.writelines(line + "\n" for line in completed.stdout.splitlines() if line.startswith("E7RESULT "))
    return completed.returncode, product


def command_selfproof(arguments: argparse.Namespace) -> int:
    """RP2（第 11.8 节）：①复用关、入一个只给这一单元用的库；②复用开、取来，与①逐字节比；③另一个目录现跑、照 RC1 比。"""
    repo = os.path.abspath(arguments.repo)
    root = os.path.join(os.path.abspath(os.path.expanduser(arguments.library)), f"{time.strftime('%Y%m%d')}-{os.getpid()}")
    library = Library(root)
    library.check_run_condition(repo)
    markers = os.path.join(arguments.scratch, "markers")
    sections = {"kind": ["device"], "build": [f"binary {sha256_file(arguments.binary)}"], "args": ["stream-constants --streams 1,2,3"],
                "limit": limit_lines(None), "python": [python_text()]}
    lines = [f"E7RESULT name=reuse_selfproof_config library={value(root)} binary_sha256={sha256_file(arguments.binary)} key={key_of(sections)}"]
    first_exit, first_product = selfproof_run(repo, arguments.binary, arguments.threads, arguments.memory_cap, os.path.join(arguments.scratch, "run-1"))
    first, _ = decide(library, "reuse-selfproof", "device", sections, "", False)
    inserted = record_finished(library, markers, first, {"product": first_product}, device_exit_insertable(first_exit, (0,)),
                               {"exit": str(first_exit), "origin_product": "reuse-selfproof-run-1"})
    lines.append(f"E7RESULT name=reuse_selfproof step=1 lookup=off exit={first_exit} insert={inserted}")
    second, _ = decide(library, "reuse-selfproof", "device", sections, "", True)
    second_product = os.path.join(arguments.scratch, "run-2", "stream-constants.out")
    fetched_identical = False
    if second.state == STATE_REUSED:
        fetch(library, second.entry, {"product": second_product})
        fetched_identical = open(second_product, "rb").read() == open(first_product, "rb").read()
    lines.append(f"E7RESULT name=reuse_selfproof step=2 lookup=on state={second.state} fetched_identical_to_step_1={str(fetched_identical).lower()}")
    third_exit, third_product = selfproof_run(repo, arguments.binary, arguments.threads, arguments.memory_cap, os.path.join(arguments.scratch, "run-3"))
    third, _ = decide(library, "reuse-selfproof", "device", sections, "", False)
    compared = record_finished(library, markers, third, {"product": third_product}, device_exit_insertable(third_exit, (0,)), {"exit": str(third_exit)})
    lines.append(f"E7RESULT name=reuse_selfproof step=3 lookup=off exit={third_exit} insert={compared} identical={str(compared == 'present-identical').lower()}")
    holds = first_exit == 0 and inserted == "inserted" and fetched_identical and third_exit == 0 and compared == "present-identical"
    lines.append(f"E7RESULT name=verdict selfproof_holds={str(holds).lower()}")
    with open(arguments.output, "x", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    return 0 if holds else 1


def main(argument_list: list[str]) -> int:
    refusal = harness.refuse_break_outside_selftest(BREAK_VARIABLE, SELFTEST_VARIABLE)
    if refusal:
        print(refusal, file=sys.stderr)
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 修订 11：按输入复用（键、库、账本）")
    commands = parser.add_subparsers(dest="command", required=True)
    fetch_parser = commands.add_parser("fetch")
    fetch_parser.add_argument("--library", required=True)
    fetch_parser.add_argument("--entry", required=True)
    fetch_parser.add_argument("--map", action="append", required=True)
    proof = commands.add_parser("selfproof")
    for name in ("--library", "--binary", "--scratch", "--output", "--threads", "--memory-cap"):
        proof.add_argument(name, required=True)
    proof.add_argument("--repo", default=os.getcwd())
    arguments = parser.parse_args(argument_list)
    try:
        return command_fetch(arguments) if arguments.command == "fetch" else command_selfproof(arguments)
    except RunCondition as error:
        print(f"✗ 运行条件不满足（退 78）：{error}", file=sys.stderr)
        return EXIT_RUN_CONDITION


# ── 自证：假仓、假树、假装置、假驱动、假判法；键、库、账本都调上面的真函数 ──
def write_text(path: str, text: str) -> str:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    return path


FAKE_MUTATIONS = ("m-row-one\tcrates/core/src/mutated.rs\tx\ty\t-p core\tt_one\n"
                  "m-row-two\tcrates/core/src/other.rs\talpha\tbeta\t-p core\tt_two\n"
                  "m-module\tcrates/core/src/other.rs\tgamma\tmod gamma\t-p core\tt_three\n")
FAKE_SELECTED_117 = ("1\t1\tcrates/core/src/mutated.rs\t0\tm-row-one\taaaaaaaaaaaa\n2\t1\tcrates/core/src/other.rs\t0\tm-row-two\tbbbbbbbbbbbb\n"
                     "3\t1\tcrates/core/src/other.rs\t1\tm-module\tcccccccccccc\n4\t2\tcrates/core/src/other.rs\t0\tm-row-two\tdddddddddddd\n"
                     "5\t2\tcrates/core/src/other.rs\t0\tm-row-two\teeeeeeeeeeee\n")
FAKE_SELECTED_164 = "118\t2\tcrates/core/src/other.rs\t2\tm-missing\tffffffffffff\n"
# 假仓里研究一侧那几份（驱动闭包、执行器那三份）：自己列一份，不取上面的键用常量（那几个常量是被测的，改坏了不该连假世界一起变）
FAKE_RESEARCH_FILES = ("row_driver.py", "device_timing.py", "frozen_o3.py", "run_directory.py", "bad_implementation_table.py", "o3_bad_image_judge.py",
                       "result_lines.py")


@dataclasses.dataclass
class FakeWorld:
    root: str
    repo: str
    tree: str

    def inputs(self, toolchain: str = "cargo 1\nrustc 1\n", python: str = "Python fake", environment: dict[str, str] | None = None) -> TreeInputs:
        dep_infos = {"device": os.path.join(self.tree, "target/release", DEVICE_BINARY + ".d"), "e170": os.path.join(self.tree, "target/release", E170_BINARY + ".d")}
        return tree_inputs(self.repo, self.tree, dep_infos, toolchain, python, environment or {}, ["<构建环境：假> 0"])


def fake_world(root: str, mutated_text: str = "fn f() { yx }\n") -> FakeWorld:
    repo, tree = os.path.join(root, "repo"), os.path.join(root, "repo", "scripts/experiment-apparatus/target/fake-tree")
    for relative, text in (("Cargo.toml", "[workspace]\n"), ("Cargo.lock", "# lock\n"), (".cargo/config.toml", "[build]\n"),
                           ("crates/core/Cargo.toml", "[package]\nname='core'\n"), ("crates/core/src/mutated.rs", mutated_text),
                           ("crates/core/src/lib.rs", 'fn read() { std::env::var("SINGLEFS_FAKE_SWITCH"); std::env::var("SINGLEFS_THREAD_CAP"); }\n'),
                           ("crates/core/src/other.rs", "fn g() { alpha; gamma }\n"), ("crates/mutations.tsv", FAKE_MUTATIONS)):
        write_text(os.path.join(tree, relative), text)
    listed = " ".join(os.path.join(tree, "crates/core/src", name) for name in ("lib.rs", "mutated.rs", "other.rs"))
    for binary in (DEVICE_BINARY, E170_BINARY):
        write_text(os.path.join(tree, "target/release", binary + ".d"), f"{tree}/target/release/{binary}: {listed}\n\n{listed}:\n")
    research = os.path.join(repo, RESEARCH_RELATIVE)
    for name in FAKE_RESEARCH_FILES:
        write_text(os.path.join(research, name), f"# 假的 {name}\n")
    write_text(os.path.join(research, SELECTED_ROW_TABLES[0]), FAKE_SELECTED_117)
    write_text(os.path.join(research, SELECTED_ROW_TABLES[1]), FAKE_SELECTED_164)
    write_text(os.path.join(repo, EXECUTOR_RELATIVE, "executor_main.py"), "# 假执行器\n")
    write_text(os.path.join(repo, FREEZE_REFERENCE_RELATIVE), "E7RESULT name=freeze fake=1\n")
    write_text(os.path.join(repo, MUTATIONS_TABLE_RELATIVE), "# 假变异表\n")
    return FakeWorld(root, repo, tree)


def fake_device_reading(directory: str, unit: str, exit_code: int = 0) -> dict[str, str]:
    """假装置的一趟：产物（E7RESULT 行，只看单元名）、计时（每次不同）、导出（impl-verdicts.tsv 末列每次不同，同真装置的耗时列）。"""
    product = write_text(os.path.join(directory, "product.out"), f"E7RESULT name=fake unit={unit} exit={exit_code}\n")
    timing = write_text(os.path.join(directory, "timing.out"), f"E7TIMING name=process_wall seconds={secrets.randbelow(10**6)} exit={exit_code}\n")
    export = os.path.join(directory, "export")
    for stream in ("new_pool_file_creation", "fixed_script_through_e_then_unmount"):
        write_text(os.path.join(export, stream, "states.tsv"), f"{unit} {stream} states\n")
        write_text(os.path.join(export, stream, "impl-verdicts.tsv"), f"{unit}\tverdict\t{secrets.randbelow(10**9)}\n")
    return {"product": product, "timing": timing, "directory": export}


def fake_row_reading(directory: str, unit: str, device_exit: int = 0, executor: bool = True) -> dict[str, str]:
    """假驱动的一行（形状同 row_driver.py 的行目录）：易变键、E7TIMING、o3.out 的 name=config 与 *_ns 每次不同。"""
    row = os.path.join(directory, "row")
    fields = {"row_number": unit, "row_name": unit, "lane": secrets.randbelow(4), REBUILD_CHECK_FIELD: "crates/core", "substituted_from": "-",
              "failed_attempts": 0, "copy_ns": secrets.randbelow(10**9), "compile_ns": secrets.randbelow(10**9), "compile_kind": "cold",
              "status": "ok", "device_exit": device_exit, "device_ns": secrets.randbelow(10**9)}
    write_text(os.path.join(row, "row-result.tsv"), "".join(f"{key}\t{text}\n" for key, text in fields.items()))
    write_text(os.path.join(row, "device.out"), f"E7RESULT name=row_result unit={unit}\nE7TIMING name=process_wall seconds={secrets.randbelow(10**6)}\n")
    write_text(os.path.join(row, "dump", "new_pool_file_creation", "states.tsv"), f"{unit} states\n")
    write_text(os.path.join(row, "dump", "new_pool_file_creation", "impl-verdicts.tsv"), f"{unit}\tok\t{secrets.randbelow(10**9)}\n")
    if executor:
        write_text(os.path.join(row, "o3.out"), f"name=config dumps={row} workers={secrets.randbelow(9)}\nname=verdict_summary unit={unit} load_ns={secrets.randbelow(10**9)}\n")
    return {"row": row}


def furnish_rows_world(repo: str, tree: str, numbers: tuple[int, ...]) -> None:
    """两份逐行编排自证用：往编排器自己的假仓与假树里补上算键要的几样（假 core、变异表、两份 dep-info、研究一侧驱动闭包与执行器、行表）。
    行表里点名的行号都指 m-row-two（other.rs 里 alpha → beta），第 1 行指 m-row-one（空变异取它）。先补再算 H3、H8。"""
    for relative, text in (("crates/core/Cargo.toml", "[package]\nname='core'\n"), ("crates/core/src/mutated.rs", "fn f() { yx }\n"),
                           ("crates/core/src/lib.rs", 'fn read() { std::env::var("SINGLEFS_FAKE_SWITCH"); }\n'),
                           ("crates/core/src/other.rs", "fn g() { alpha; gamma }\n"), ("crates/mutations.tsv", FAKE_MUTATIONS)):
        write_text(os.path.join(tree, relative), text)
    listed = " ".join(os.path.join(tree, "crates/core/src", name) for name in ("lib.rs", "mutated.rs", "other.rs"))
    for binary in (DEVICE_BINARY, E170_BINARY):
        write_text(os.path.join(tree, "target/release", binary + ".d"), f"{tree}/target/release/{binary}: {listed}\n")
    research = os.path.join(repo, RESEARCH_RELATIVE)
    for name in FAKE_RESEARCH_FILES:
        write_text(os.path.join(research, name), f"# 假的 {name}\n")
    table = "1\t1\tcrates/core/src/mutated.rs\t0\tm-row-one\taaaaaaaaaaaa\n" + "".join(
        f"{number}\t2\tcrates/core/src/other.rs\t0\tm-row-two\t{number:012d}\n" for number in sorted(set(numbers) - {0, 1}))
    write_text(os.path.join(research, SELECTED_ROW_TABLES[0]), table)
    write_text(os.path.join(research, SELECTED_ROW_TABLES[1]), "")
    write_text(os.path.join(repo, EXECUTOR_RELATIVE, "executor_main.py"), "# 假执行器\n")
    if not os.path.exists(os.path.join(repo, FREEZE_REFERENCE_RELATIVE)):
        write_text(os.path.join(repo, FREEZE_REFERENCE_RELATIVE), "E7RESULT name=freeze fake=1\n")


FAKE_DRIVER_WRAPPER = r'''#!/usr/bin/env python3
"""自证用：先起编排器自证那个假逐行驱动，再在 <工作根>/lane0 下放一份「这一次编译」的 dep-info（路径换成线道那棵树），给 RC4 核。"""
import os, shutil, subprocess, sys
arguments = sys.argv[1:]
code = subprocess.run([sys.executable, "-B", "FAKE_DRIVER_PATH"] + arguments).returncode
value = lambda option: arguments[arguments.index(option) + 1]
snapshot, lane = value("--snapshot-from"), os.path.join(value("--work-root"), "lane0")
release = os.path.join(snapshot, "target", "release")  # 别的台那一份快照不带 target/：那边不做 RC4（H35），不放
if os.path.isdir(release):
    shutil.copytree(os.path.join(snapshot, "crates"), os.path.join(lane, "tree", "crates"), dirs_exist_ok=True)
    os.makedirs(os.path.join(lane, "target", "release"), exist_ok=True)
for name in os.listdir(release) if os.path.isdir(release) else []:
    if name.endswith(".d"):
        text = open(os.path.join(snapshot, "target", "release", name), encoding="utf-8").read().replace(snapshot, os.path.join(lane, "tree"))
        open(os.path.join(lane, "target", "release", name), "w", encoding="utf-8").write(text)
sys.exit(code)
'''


def fake_judge(reading: dict[str, str]) -> str:
    """假判法：读产物或 row-result.tsv（取来的与现跑的都读）。"""
    if "row" in reading:
        return " ".join(row_result_without_volatile_keys(os.path.join(reading["row"], "row-result.tsv")))
    return open(reading["product"], encoding="utf-8").read().strip()


def library_in(scratch: str) -> Library:
    library = Library(os.path.join(scratch, "library"))
    for directory in (library.entries, library.staging, library.quarantine_directory):
        os.makedirs(directory, exist_ok=True)
    return library


def reuse_across(scratch: str, first_sections: dict, second_sections: dict, kind: str = "device") -> tuple[bool, str]:
    """第一份键现跑、入库，再拿第二份查：交回 (第二份取到了没有, 细节)。"""
    library, markers = library_in(scratch), os.path.join(scratch, "markers")
    first, _ = decide(library, "unit", kind, first_sections, "", False)
    record_finished(library, markers, first, fake_device_reading(os.path.join(scratch, "first"), "unit"), (True, ""), {"exit": "0"})
    second, _ = decide(library, "unit", kind, second_sections, "", True)
    return second.state == STATE_REUSED, f"键 {first.key[:12]} → {second.key[:12]}，第二份 {second.state}（{second.reason}）"


def missed(outcome: tuple[bool, str]) -> tuple[bool, str]:
    return not outcome[0], outcome[1]


def row_two(world: FakeWorld, inputs: TreeInputs | None = None, selected: dict[int, str] | None = None, limit: int = 1800) -> dict:
    row = RowUnit("seg2-row-2", 2, False, ("--streams", "1,2", "--no-executor"), False, False)
    return row_sections(inputs or world.inputs(), row, selected or selected_row_lines(world.repo), limit_lines(limit), [])[0]


def cell_rp3_kind(scratch: str) -> tuple[bool, str]:
    world = fake_world(os.path.join(scratch, "world"))
    inputs = world.inputs()
    device = ReadingUnit("u", "u", "u", "device", "baseline --streams 1")
    return missed(reuse_across(scratch, reading_unit_sections(inputs, device, []),
                               reading_unit_sections(inputs, dataclasses.replace(device, kind="e170"), [])))


def cell_rp3_build(scratch: str) -> tuple[bool, str]:
    world = fake_world(os.path.join(scratch, "world"))
    device = ReadingUnit("u", "u", "u", "device", "baseline --streams 1")
    return missed(reuse_across(scratch, reading_unit_sections(world.inputs(toolchain="cargo 1\n"), device, []),
                               reading_unit_sections(world.inputs(toolchain="cargo 2\n"), device, [])))


def cell_rp3_closure(scratch: str) -> tuple[bool, str]:
    world = fake_world(os.path.join(scratch, "world"))
    first = row_two(world)
    write_text(os.path.join(world.tree, "crates/core/src/lib.rs"), open(os.path.join(world.tree, "crates/core/src/lib.rs")).read() + "// 改了\n")
    return missed(reuse_across(scratch, first, row_two(world), "driver-row"))


def cell_rp3_closure_mutated_content(scratch: str) -> tuple[bool, str]:
    """改坏之前那份文件不同、改坏之后逐字节相同（编进去的源码一样）：照第 11.2 节 [closure] 取改坏之后的内容，要取到。"""
    first_world, second_world = fake_world(os.path.join(scratch, "a"), "fn f() { yx }\n"), fake_world(os.path.join(scratch, "b"), "fn f() { xy }\n")
    row = RowUnit("seg2-row-1", 1, False, ("--streams", "1,2", "--no-executor"), False, False)
    first = row_sections(first_world.inputs(), row, selected_row_lines(first_world.repo), limit_lines(1800), [])[0]
    second = row_sections(second_world.inputs(), row, selected_row_lines(second_world.repo), limit_lines(1800), [])[0]
    return reuse_across(scratch, first, second, "driver-row")


def cell_rp3_env(scratch: str) -> tuple[bool, str]:
    """设了一个闭包里读的变量：键不同，且那个单元不查库（即便库里有它那个键）。"""
    world = fake_world(os.path.join(scratch, "world"))
    device = ReadingUnit("u", "u", "u", "device", "baseline --streams 1")
    unset_sections = reading_unit_sections(world.inputs(environment={}), device, [])
    set_sections = reading_unit_sections(world.inputs(environment={"SINGLEFS_FAKE_SWITCH": "1"}), device, [])
    hit, detail = reuse_across(scratch, unset_sections, set_sections)
    decision, _ = decide(library_in(scratch), "unit", "device", set_sections, "", True)
    return not hit and key_of(unset_sections) != key_of(set_sections) and decision.reason == "env-set", f"{detail}；设了变量的那一份 {decision.reason}"


def cell_rp3_mutation(scratch: str) -> tuple[bool, str]:
    world = fake_world(os.path.join(scratch, "world"))
    selected = selected_row_lines(world.repo)
    changed = {**selected, 2: selected[2].replace("bbbbbbbbbbbb", "bbbbbbbbbbbc")}
    return missed(reuse_across(scratch, row_two(world, selected=selected), row_two(world, selected=changed), "driver-row"))


def cell_rp3_args(scratch: str) -> tuple[bool, str]:
    world = fake_world(os.path.join(scratch, "world"))
    inputs = world.inputs()
    first, second = (ReadingUnit("u", "u", "u", "device", f"baseline --streams 2 --positive-control {control}") for control in ("O1-对拍", "O1-对拍-卸载"))
    return missed(reuse_across(scratch, reading_unit_sections(inputs, first, []), reading_unit_sections(inputs, second, [])))


def cell_rp3_limit(scratch: str) -> tuple[bool, str]:
    world = fake_world(os.path.join(scratch, "world"))
    return missed(reuse_across(scratch, row_two(world, limit=1800), row_two(world, limit=2400), "driver-row"))


def cell_rp3_upstream(scratch: str) -> tuple[bool, str]:
    world = fake_world(os.path.join(scratch, "world"))
    inputs = world.inputs()
    export = fake_device_reading(os.path.join(scratch, "upstream"), "step-1")["directory"]
    unit = [unit for unit in SEGMENT_ONE_READING_UNITS if unit.name == "step-5"][0]
    first = reading_unit_sections(inputs, unit, upstream_lines([("device-baseline", "step-1", export)]))
    with open(os.path.join(export, "new_pool_file_creation", "states.tsv"), "a", encoding="utf-8") as handle:
        handle.write("x")
    second = reading_unit_sections(inputs, unit, upstream_lines([("device-baseline", "step-1", export)]))
    return missed(reuse_across(scratch, first, second, "o3-executor"))


def cell_rp3_research(scratch: str) -> tuple[bool, str]:
    """驱动 import 的一个模块（device_timing.py）改了：驱动行的键要变。"""
    world = fake_world(os.path.join(scratch, "world"))
    first = row_two(world)
    write_text(os.path.join(world.repo, RESEARCH_RELATIVE, "device_timing.py"), "# 改过的 device_timing.py\n")
    return missed(reuse_across(scratch, first, row_two(world), "driver-row"))


def cell_rp3_python(scratch: str) -> tuple[bool, str]:
    world = fake_world(os.path.join(scratch, "world"))
    return missed(reuse_across(scratch, row_two(world, inputs=world.inputs(python="Python 3.12.3 甲")),
                               row_two(world, inputs=world.inputs(python="Python 3.12.4 乙")), "driver-row"))


def killed_in_child(point: str, action) -> int:
    """在一个 fork 出的子进程里走 action，走到 point 那一处它给自己发 SIGKILL（只在这个子进程里设变量）。交回子进程的退出码（-9 是被杀）。"""
    import multiprocessing  # noqa: E402

    def child() -> None:
        os.environ[SELFTEST_VARIABLE], os.environ[KILL_POINT_VARIABLE] = "1", point
        action()

    process = multiprocessing.get_context("fork").Process(target=child)
    process.start()
    process.join(60)
    return process.exitcode


def half_entries(library: Library) -> list[str]:
    """entries/ 下读数与 manifest 对不上（含没有 manifest）的条目。"""
    found = []
    for prefix in sorted(os.listdir(library.entries)) if os.path.isdir(library.entries) else []:
        for key in sorted(os.listdir(os.path.join(library.entries, prefix))):
            if manifest_problems(os.path.join(library.entries, prefix, key)):
                found.append(key[:12])
    return found


def cell_rp5_atomic_insert(scratch: str) -> tuple[bool, str]:
    """在 manifest 写之前、rename 之前各杀一次：entries/ 下没有半条条目；再起时 staging/ 残留删掉；那一单元照常入库。"""
    library = library_in(scratch)
    reading = fake_device_reading(os.path.join(scratch, "reading"), "unit")
    sections = {"kind": ["device"], "args": ["baseline --streams 1"]}
    codes, half = [], []
    for point in ("before-manifest", "before-rename"):  # 每杀一次就数一遍：第二次入库会把第一次留下的坏条目挪走，攒到最后再数就看不见
        codes.append(killed_in_child(point, lambda: library.insert(key_of(sections), key_text(sections), "device", reading, {"exit": "0"})))
        half += [f"{point}:{key}" for key in half_entries(library)]
    leftover = sorted(os.listdir(library.staging))
    removed = library.clear_stale_staging()
    state, _ = library.insert(key_of(sections), key_text(sections), "device", reading, {"exit": "0"})
    good = codes == [-signal.SIGKILL, -signal.SIGKILL] and not half and len(leftover) == 2 and len(removed) == 2 and state == "inserted" \
        and not os.listdir(library.staging) and not half_entries(library)
    return good, f"子进程退出码 {codes}；被杀之后 entries/ 里半条条目 {half or '无'}；staging/ 残留 {len(leftover)}、清掉 {len(removed)}；再入库 {state}"


def cell_rp6_closure(scratch: str, extra_file) -> tuple[bool, str]:
    """extra_file(假树) 交回现跑那一次的 dep-info 比键多出来的那个文件。"""
    world = fake_world(os.path.join(scratch, "world"))
    extra_line = extra_file(world)
    inputs = world.inputs()
    markers = os.path.join(scratch, "markers")
    sections = row_two(world, inputs=inputs)
    decision, _ = decide(None, "seg2-row-2", "driver-row", sections, "", False)
    write_marker(markers, decision)
    listed = " ".join(os.path.join(world.tree, "crates/core/src", name) for name in ("lib.rs", "mutated.rs", "other.rs")) + " " + extra_line
    dep_info = write_text(os.path.join(scratch, "compiled.d"), f"{world.tree}/target/release/x: {listed}\n")
    matches = closure_check(markers, decision, world.tree, dep_info, mutated_closure_paths(inputs))
    text, _ = ledger_text(markers, [decision.unit])
    return not matches and " closure_mismatches=1 " in text and bool(s18_problems(markers)), \
        f"相符 {str(matches).lower()}；账本末行 {text.strip().splitlines()[-1][:120]}；S18 {len(s18_problems(markers))} 行"


def cell_rp6_closure_extra_file(scratch: str) -> tuple[bool, str]:
    """现跑那一次的 dep-info 多一个树里有、键里没有的文件。"""
    return cell_rp6_closure(scratch, lambda world: write_text(os.path.join(world.tree, "crates/core/src/extra.rs"), "fn extra() {}\n"))


def cell_rp6_closure_outside_the_tree(scratch: str) -> tuple[bool, str]:
    """现跑那一次的 dep-info 里有树外的路径。"""
    return cell_rp6_closure(scratch, lambda world: write_text(os.path.join(scratch, "outside.rs"), "fn outside() {}\n"))


def cell_rp7_check_mismatch(scratch: str) -> tuple[bool, str]:
    """同键第二次跑时导出里一个文件多一字节（E7RESULT 产物不变）：present-differs、check_mismatches=1、S18。"""
    library, markers = library_in(scratch), os.path.join(scratch, "markers")
    sections = {"kind": ["device"], "args": ["baseline --streams 1,2,3 --writable-mount-probe"]}
    first, _ = decide(library, "step-7", "device", sections, "", False)
    record_finished(library, markers, first, fake_device_reading(os.path.join(scratch, "first"), "baseline"), (True, ""), {"exit": "0"})
    second_reading = fake_device_reading(os.path.join(scratch, "second"), "baseline")
    with open(os.path.join(second_reading["directory"], "new_pool_file_creation", "states.tsv"), "a", encoding="utf-8") as handle:
        handle.write("x")
    second, _ = decide(library, "step-9", "device", sections, "second-run", True)
    write_marker(markers, first)
    write_marker(markers, second)
    state = record_finished(library, markers, second, second_reading, (True, ""), {"exit": "0"})
    text, _ = ledger_text(markers, ["step-7", "step-9"])
    same_product = open(second_reading["product"], "rb").read() == open(entry_reading(library.entry_path(first.key))["product"], "rb").read()
    good = state == "present-differs" and " check_mismatches=1 " in text and bool(s18_problems(markers)) and same_product
    return good, f"第二次入库 {state}；产物相同 {same_product}；账本末行 {text.strip().splitlines()[-1][:110]}"


def cell_rp8_corrupted_entry(scratch: str) -> tuple[bool, str]:
    """库里一条条目的读数改一字节：不取它、挪进隔离目录、现跑；账本 corrupted_entries=1；现跑完照常入库。"""
    library, markers = library_in(scratch), os.path.join(scratch, "markers")
    sections = {"kind": ["device"], "args": ["baseline --streams 1,2"]}
    first, _ = decide(library, "step-1", "device", sections, "", False)
    record_finished(library, markers, first, fake_device_reading(os.path.join(scratch, "first"), "s2"), (True, ""), {"exit": "0"})
    entry_file = os.path.join(library.entry_path(first.key), "reading", "product")
    damaged = open(entry_file, "rb").read() + b"x"
    os.remove(entry_file)
    with open(entry_file, "wb") as handle:
        handle.write(damaged)
    second, problems = decide(library, "step-1", "device", sections, "", True)
    note_decision(markers, second, problems)
    quarantined = len(os.listdir(library.quarantine_directory))
    state = record_finished(library, markers, second, fake_device_reading(os.path.join(scratch, "second"), "s2"), (True, ""), {"exit": "0"})
    text, _ = ledger_text(markers, ["step-1"])
    good = second.state == STATE_FRESH and second.reason == "corrupted-entry" and quarantined == 1 and " corrupted_entries=1 " in text and state == "inserted"
    return good, f"第二次 {second.state}（{second.reason}）；隔离目录 {quarantined} 条；现跑完 {state}；账本末行 {text.strip().splitlines()[-1][:100]}"


def fake_row_into(row_directory: str, identity: str, device_exit: int = 0, executor: bool = True) -> dict[str, str]:
    """假驱动把一行写进行输出根下的 row-<n>/（row-result.tsv 最后写，同真驱动）。"""
    os.makedirs(os.path.dirname(row_directory), exist_ok=True)
    staging = tempfile.mkdtemp(dir=os.path.dirname(row_directory))
    produced = fake_row_reading(staging, identity, device_exit, executor)["row"]
    os.makedirs(row_directory)
    for relative in sorted(files_under(produced), key=lambda path: path == "row-result.tsv"):
        os.makedirs(os.path.dirname(os.path.join(row_directory, relative)), exist_ok=True)
        os.rename(os.path.join(produced, relative), os.path.join(row_directory, relative))
    shutil.rmtree(staging)
    return {"row": row_directory}


def row_decisions(world: FakeWorld, library: Library, rows: list[RowUnit], lookup: bool) -> list[Decision]:
    inputs, selected = world.inputs(), selected_row_lines(world.repo)
    decisions = []
    for row in rows:
        sections, reason = row_sections(inputs, row, selected, limit_lines(1800), [])
        decisions.append(decide(library, row.name, "driver-row", sections, reason, lookup)[0])
    return decisions


def cell_rp4_resume_fetch(scratch: str) -> tuple[bool, str]:
    """第二遍在现跑两行、取来两行之后断在第三行的取用中间，同一套再起：已取来的、已现跑完的都不再做；断在中间的那一行重取；
    库里条目数等于入得了库的不同键数；账本 N + M 不变。"""
    world, library = fake_world(os.path.join(scratch, "world")), library_in(scratch)
    rows = [RowUnit(f"seg2-row-{number}", number, False, ("--streams", "1,2", "--no-executor"), False, False) for number in (1, 2, 4, 5, 3)]
    exits = {"seg2-row-5": TIMEOUT_EXIT_CODE}
    first_markers, second_markers = os.path.join(scratch, "markers-1"), os.path.join(scratch, "markers-2")
    for decision, row in zip(row_decisions(world, library, rows, False), rows):
        note_decision(first_markers, decision, [])
        reading = fake_row_into(os.path.join(scratch, "rows-1", f"row-{row.number}"), decision.key[:16], exits.get(row.name, 0), False)
        record_finished(library, first_markers, decision, reading, row_insertable(reading["row"]), {"exit": "0"})
    decisions = row_decisions(world, library, rows, True)
    root = os.path.join(scratch, "rows-2")
    pairs = [(decision, os.path.join(root, f"row-{row.number}")) for decision, row in zip(decisions, rows)]

    def interrupted_pass() -> None:
        os.makedirs(root, exist_ok=True)
        for decision, row_directory in pairs:
            if decision.state != STATE_REUSED:
                write_marker(second_markers, decision)
                fake_row_into(row_directory, decision.key[:16], exits.get(decision.unit, 0), False)
        fetch_rows(library, second_markers, pairs)

    code = killed_in_child("before-row-result:row-4", interrupted_pass)
    inodes_before = {row: os.stat(os.path.join(root, f"row-{row}", "row-result.tsv")).st_ino for row in (1, 2, 5, 3)}
    partial = os.path.isdir(os.path.join(root, "row-4")) and not os.path.exists(os.path.join(root, "row-4", "row-result.tsv"))
    fetched, demoted = fetch_rows(library, second_markers, pairs)
    inodes_after = {row: os.stat(os.path.join(root, f"row-{row}", "row-result.tsv")).st_ino for row in (1, 2, 5, 3)}
    entries = sum(len(os.listdir(os.path.join(library.entries, prefix))) for prefix in os.listdir(library.entries))
    text_one, covers_one = ledger_text(first_markers, [row.name for row in rows])
    text_two, covers_two = ledger_text(second_markers, [row.name for row in rows])
    good = (code == -signal.SIGKILL and partial and fetched == ["seg2-row-4"] and not demoted and inodes_before == inodes_after and entries == 3
            and covers_one and covers_two and " reused=3 " in text_two and not reading_differences("driver-row", {"row": os.path.join(scratch, "rows-1", "row-4")},
                                                                                                   {"row": os.path.join(root, "row-4")}))
    return good, (f"子进程 {code}；断在中间的 row-4 {'是半份' if partial else '不是半份'}；续跑重取 {fetched}；别的行没再动 {inodes_before == inodes_after}；"
                  f"库里 {entries} 条；账本 {text_two.strip().splitlines()[-1][:90]}")


RP1_ROWS = (
    RowUnit("seg1-s11-row-1", 1, False, ("--keep-dumps", "--streams", "1,2,3", "--writable-mount-probe"), True, False, owner="s11"),
    RowUnit("seg1-s11-row-2", 2, False, ("--keep-dumps", "--streams", "1,2,3", "--writable-mount-probe"), True, False, owner="s11"),
    RowUnit("seg1-s12-row-0", 0, True, ("--empty-mutation", "--keep-dumps", "--streams", "1,2,3"), True, True, owner="s12"),
    RowUnit("seg1-s12-row-1", 1, False, ("--keep-dumps", "--streams", "1,2,3"), True, False, owner="s12"),
    RowUnit("seg1-s12-row-2", 2, False, ("--keep-dumps", "--streams", "1,2,3"), True, False, owner="s12"),
    RowUnit("seg1-s12-row-3", 3, False, ("--keep-dumps", "--streams", "1,2,3"), True, False, owner="s12"),
    RowUnit("seg2-row-1", 1, False, ("--streams", "1,2", "--no-executor"), False, False, owner="rows"),
    RowUnit("seg2-row-2", 2, False, ("--streams", "1,2", "--no-executor"), False, True, owner="rows"),
    RowUnit("seg2-row-3", 3, False, ("--streams", "1,2", "--no-executor"), False, False, owner="rows"),
    RowUnit("seg2-row-4", 4, False, ("--streams", "1,2", "--no-executor"), False, False, owner="rows"),
    RowUnit("seg2-row-5", 5, False, ("--streams", "1,2", "--no-executor"), False, False, owner="rows"),
    RowUnit("seg2-row-118", 118, False, ("--streams", "1,2", "--no-executor"), False, False, owner="rows"),
)
# 第一遍里这几个单元的退出码（别的都是 0）：panic 对照预期退 3；step-6 撞包装（251，不入库）；第二段第 4 行超时（253，不入库）
RP1_EXITS = {"step-3-panic": 3, "step-6": 251, "seg2-row-4": TIMEOUT_EXIT_CODE}
RP1_FRESH_IN_THE_SECOND_PASS = ("step-9", "step-10-run-2", "step-6", "seg1-s12-row-0", "seg1-s12-row-3", "seg2-row-2", "seg2-row-3", "seg2-row-4",
                                "seg2-row-118")


def rp1_pass(world: FakeWorld, library: Library, markers: str, lookup: bool, root: str) -> tuple[dict, dict, dict]:
    """假编排的一遍：第 0–10 步的全部读数单元（单元表原样）与两段各 6 行，依次算键、查库、取来或假跑、入库，判法每个单元都现跑。"""
    inputs, selected = world.inputs(), selected_row_lines(world.repo)
    readings, judges, decisions = {}, {}, {}
    for unit in list(SEGMENT_ONE_READING_UNITS) + step_4_row_units() + list(RP1_ROWS):
        upstream = upstream_lines([(role, name, readings[name].get("directory") or readings[name]["row"]) for role, name in unit.upstream])
        if isinstance(unit, RowUnit):
            sections, reason = row_sections(inputs, unit, selected, limit_lines(1800), upstream)
            kind = "driver-row"
        else:
            sections, reason, kind = reading_unit_sections(inputs, unit, upstream), "second-run" if unit.fresh_always else "", unit.kind
        decision, problems = decide(library, unit.name, kind, sections, reason, lookup)
        note_decision(markers, decision, problems)
        directory = os.path.join(root, unit.name.replace("/", "-"))
        exit_code = RP1_EXITS.get(unit.name, 0)
        if decision.state == STATE_REUSED:
            reading = {role: os.path.join(directory, role) for role in entry_reading(decision.entry)}
            fetch(library, decision.entry, reading)
        elif isinstance(unit, RowUnit):
            reading = fake_row_reading(directory, decision.key[:16], exit_code, unit.executor)
            record_finished(library, markers, decision, reading, row_insertable(reading["row"]), {"exit": str(exit_code)})
        else:
            reading = fake_device_reading(directory, decision.key[:16], exit_code)
            record_finished(library, markers, decision, reading, device_exit_insertable(exit_code, unit.expected_exits), {"exit": str(exit_code)})
        readings[unit.name], judges[unit.name], decisions[unit.name] = reading, fake_judge(reading), decision
    return readings, judges, decisions


def same_bytes(first: dict[str, str], second: dict[str, str]) -> bool:
    for role in set(first) | set(second):
        if role == "timing":
            continue
        if role not in first or role not in second:
            return False
        left, right = first[role], second[role]
        files = files_under(left) if os.path.isdir(left) else [""]
        if (files_under(right) if os.path.isdir(right) else [""]) != files:
            return False
        if any(sha256_file(os.path.join(left, relative) if relative else left) != sha256_file(os.path.join(right, relative) if relative else right)
               for relative in files):
            return False
    return True


def cell_rp1_two_passes(scratch: str) -> tuple[bool, str]:
    """复用关跑一遍（入库）、复用开再跑一遍：第二遍除永远现跑与没入库的几个外全部取来；取来的读数与第一遍逐字节相同，现跑的照读数相等的比法相同；
    每个单元的判法两遍逐字相同；账本 N + M 等于单元数。"""
    world, library = fake_world(os.path.join(scratch, "world")), library_in(scratch)
    first_readings, first_judges, _ = rp1_pass(world, library, os.path.join(scratch, "markers-1"), False, os.path.join(scratch, "pass-1"))
    second_readings, second_judges, decisions = rp1_pass(world, library, os.path.join(scratch, "markers-2"), True, os.path.join(scratch, "pass-2"))
    names = list(decisions)
    fresh = sorted(name for name, decision in decisions.items() if decision.state != STATE_REUSED)
    expected_fresh = sorted(RP1_FRESH_IN_THE_SECOND_PASS)
    differing = [name for name in names if not (same_bytes(first_readings[name], second_readings[name]) if decisions[name].state == STATE_REUSED
                                                 else not reading_differences(decisions[name].kind, first_readings[name], second_readings[name]))]
    judge_differences = [name for name in names if first_judges[name] != second_judges[name]]
    text, covers = ledger_text(os.path.join(scratch, "markers-2"), names)
    verdict = text.strip().splitlines()[-1]
    good = (fresh == expected_fresh and not differing and not judge_differences and covers and f" reused={len(names) - len(expected_fresh)} " in verdict
            and " check_mismatches=0 " in verdict and not s18_problems(os.path.join(scratch, "markers-1")))
    return good, (f"{len(names)} 个单元；第二遍现跑的 {fresh if fresh != expected_fresh else len(fresh)}（要 {len(expected_fresh)} 个）；读数不同的 {differing or '无'}；"
                  f"判法不同的 {judge_differences or '无'}；账本 {verdict[:120]}")


LAYER_FILES = ("segment_one_units_revision_11.py", "rows_orchestrator_revision_11.py", "rows_orchestrator_segment_two_revision_11.py")


def cell_rp1_layers_end_to_end(scratch: str) -> tuple[bool, str]:
    """三层编排各自的端到端格（reuse-e2e：假单元脚本或假驱动，复用关、复用开各跑一遍，取来的单元判法照跑）。"""
    outcomes = []
    for name in LAYER_FILES:
        completed = subprocess.run([sys.executable, "-B", os.path.join(HERE, name), "--selftest", "--cells", "reuse-e2e"], capture_output=True, text=True,
                                   timeout=900, env=dict(os.environ, TMPDIR=scratch))
        last = (completed.stdout.strip().splitlines() or ["没有输出"])[-1]
        outcomes.append((name, completed.returncode == 0 and last.endswith("红 0"), last))
    return all(ok for _, ok, _ in outcomes), "；".join(f"{name} {last}" for name, _, last in outcomes)


# ── 钉值认搬迁登记（pinned_sha256_now 上方那一段注释）的两格 ──
# 假前缀：真目录改名时全仓的文本替换不会把样本换坏；fixtures/old-homestead 不在前缀边界上，换不换都一样
PIN_MOVE_OLD, PIN_MOVE_NEW = "fixtures/old-home", "fixtures/new-home"
PIN_MOVE_SOURCE = "records/2026-10-23-实验装置目录改名.md"
PIN_MOVE_SELFTEST_SUFFIX = "-moved-in-selftest"
# 本目录每一处钉值：(写钉值的那份文件, 钉值的名字, 被钉的文件)；钉值按一整行「名字 = "<64 位十六进制>"」从那份文件里现读
PIN_SITES = (("reuse_library.py", "ORCHESTRATOR_SHA256", "rows_orchestrator.py"),
             ("reuse_library.py", "SEGMENT_TWO_ORCHESTRATOR_SHA256", "rows_orchestrator_segment_two.py"),
             ("reuse_library.py", "CROSS_HOST_CHECK_SHA256", "cross_host_o3_check.py"),
             ("rows_orchestrator_revision_11.py", "ROWS_R8_SHA256", "rows_orchestrator_revision_8.py"),
             ("rows_orchestrator_segment_two_revision_11.py", "SEGMENT_TWO_R10_SHA256", "rows_orchestrator_segment_two_revision_10.py"),
             ("segment_one_units_revision_11.py", "UNITS_R8_SHA256", "segment_one_units_revision_8.py"))


def write_move_table(root: str, rows: list[tuple[str, str]]) -> str:
    """假仓的搬迁登记表（写法同 .claude/gate.d/path-moves.tsv）。"""
    return write_text(os.path.join(root, frozen_o3.PATH_MOVES_TABLE_RELATIVE),
                      "# 搬迁登记（自证）\n" + "".join(f"{old}\t{new}\t2026-10-23\t{PIN_MOVE_SOURCE}\n" for old, new in rows))


def raises_runtime_error(action) -> bool:
    try:
        action()
    except RuntimeError:
        return True
    return False


def cell_pin_moves(scratch: str) -> tuple[bool, str]:
    """红：登记表空（不在、只有注释）、被钉文件只改了路径，不认；绿：登记了搬迁、只差路径，核对、载入、冻结层要换的钉值都认此刻那一份；
    红：登记了搬迁、除了路径还改了别处，不认、载入拒。另：此刻就是钉住的那一份照认，登记表写坏了拒。"""
    frozen_text = f'HOME = "{PIN_MOVE_OLD}/target"\nNEIGHBOUR = "fixtures/old-homestead/notes"\n'
    moved_text = frozen_text.replace(f"{PIN_MOVE_OLD}/", f"{PIN_MOVE_NEW}/")
    pinned = sha256_bytes(frozen_text.encode("utf-8"))
    path = write_text(os.path.join(scratch, PIN_MOVE_NEW, "pinned_layer.py"), moved_text)
    moved = sha256_file(path)
    table = os.path.join(scratch, frozen_o3.PATH_MOVES_TABLE_RELATIVE)
    checks = [("登记表不在、只改了路径：不认，冻结层的钉值照旧",
               pinned_sha256_now(path, pinned, scratch) is None and sha256_now_or_pinned(path, pinned, scratch) == pinned),
              ("登记表不在：载入拒", raises_runtime_error(lambda: load_repinned(path, pinned, "pin_moves_selftest_unregistered", root=scratch)))]
    write_text(table, "# 只有注释\n\n")
    checks.append(("登记表只有注释：不认", pinned_sha256_now(path, pinned, scratch) is None))
    write_move_table(scratch, [(PIN_MOVE_OLD, PIN_MOVE_NEW)])
    checks.append(("登记了搬迁、只差路径：认，交此刻的 sha256",
                   pinned_sha256_now(path, pinned, scratch) == moved and sha256_now_or_pinned(path, pinned, scratch) == moved))
    try:
        module = load_repinned(path, pinned, "pin_moves_selftest_registered", root=scratch)
        checks.append(("登记了搬迁：载入，跑的是此刻的正文", module.HOME == f"{PIN_MOVE_NEW}/target"))
    except RuntimeError as error:
        checks.append((f"登记了搬迁：载入被拒（{error}）", False))
    write_text(path, moved_text + "EDITED = True\n")
    checks.append(("登记了搬迁、还改了别处：不认，冻结层的钉值照旧",
                   pinned_sha256_now(path, pinned, scratch) is None and sha256_now_or_pinned(path, pinned, scratch) == pinned))
    checks.append(("登记了搬迁、还改了别处：载入拒", raises_runtime_error(lambda: load_repinned(path, pinned, "pin_moves_selftest_edited", root=scratch))))
    write_text(path, frozen_text)
    checks.append(("此刻就是钉住的那一份：照认", pinned_sha256_now(path, pinned, scratch) == pinned))
    write_text(path, moved_text)
    write_text(table, f"{PIN_MOVE_OLD}\t{PIN_MOVE_NEW}\t2026-10-23\n")
    checks.append(("登记表写坏了（三列）：拒，不猜", raises_runtime_error(lambda: pinned_sha256_now(path, pinned, scratch))))
    wrong = [name for name, ok in checks if not ok]
    return not wrong, f"{len(checks)} 项" + (f"，不对的：{'；'.join(wrong)}" if wrong else "都对")


def pin_value(pinning_file: str, name: str) -> str | None:
    with open(os.path.join(HERE, pinning_file), encoding="utf-8") as handle:
        found = re.search(rf'^{re.escape(name)} = "([0-9a-f]{{64}})"$', handle.read(), re.M)
    return found.group(1) if found else None


def cell_pin_moves_real_files(scratch: str) -> tuple[bool, str]:
    """本目录每一处钉值（PIN_SITES）：被钉文件照一次假搬迁（本目录上一级的前缀 → 它后面接 -moved-in-selftest）换过、放进假仓，
    假仓的登记表 = 真仓登记过的搬迁 + 这一次：认，交换过之后的 sha256；只有真仓登记的那几行：换过的不认；换过之后再多改一处：不认。
    冻结层要换的钉值（rows_revision_8_repins、orchestrator_sha256_now）在假仓上交换过之后那一份的 sha256。钉值照真文件现读，
    所以这一格同时核真仓此刻每一处钉值都认（真仓还没搬或搬了已登记）。"""
    real_rows = [(move.old, move.new) for move in frozen_o3.registered_moves(PIN_REPOSITORY_ROOT).moves]
    library = frozen_o3.path_moves_library(PIN_REPOSITORY_ROOT)
    parent = os.path.relpath(os.path.dirname(HERE), PIN_REPOSITORY_ROOT).replace(os.sep, "/")
    moved_parent = parent + PIN_MOVE_SELFTEST_SUFFIX
    selftest_move = library.PathMove(0, parent, moved_parent, "2026-10-23", PIN_MOVE_SOURCE)
    directory = os.path.join(scratch, moved_parent, os.path.basename(HERE))
    os.makedirs(directory)
    problems, moved_sha256, rewritten = [], {}, 0
    for pinning_file, name, pinned_file in PIN_SITES:
        pinned = pin_value(pinning_file, name)
        if pinned is None:
            problems.append(f"{pinning_file} 里没有一整行 {name} = \"<64 位十六进制>\"")
            continue
        with open(os.path.join(HERE, pinned_file), "rb") as handle:
            current = handle.read()
        moved = library.rewrite_moved_bytes(current, [selftest_move])
        rewritten += moved != current
        path = os.path.join(directory, pinned_file)
        with open(path, "wb") as handle:
            handle.write(moved)
        moved_sha256[pinned_file] = sha256_bytes(moved)
        write_move_table(scratch, real_rows + [(parent, moved_parent)])
        if pinned_sha256_now(path, pinned, scratch) != moved_sha256[pinned_file]:
            problems.append(f"{pinned_file}：登记了这次假搬迁也不认（{name}）")
        write_move_table(scratch, real_rows)
        if moved != current and pinned_sha256_now(path, pinned, scratch) is not None:
            problems.append(f"{pinned_file}：没登记这次假搬迁也认了")
        with open(path, "ab") as handle:
            handle.write("\n# 自证：除了路径还多改一处\n".encode("utf-8"))
        write_move_table(scratch, real_rows + [(parent, moved_parent)])
        if pinned_sha256_now(path, pinned, scratch) is not None:
            problems.append(f"{pinned_file}：除了路径还多改一处也认了")
        with open(path, "wb") as handle:
            handle.write(moved)
    wanted = ((ORCHESTRATOR_SHA256_BEFORE_MOVE, moved_sha256.get("rows_orchestrator.py")),
              (CROSS_HOST_CHECK_SHA256, moved_sha256.get("cross_host_o3_check.py")))
    if rows_revision_8_repins(directory, scratch) != wanted:
        problems.append(f"冻结的修订 8 载入时要换的钉值不是换过之后那两份的 sha256：{rows_revision_8_repins(directory, scratch)}")
    if orchestrator_sha256_now(directory, scratch) != moved_sha256.get("rows_orchestrator.py"):
        problems.append("跨机对照入口要设的编排器 sha256 不是换过之后那一份的")
    detail = f"{len(PIN_SITES)} 处钉值、{len(moved_sha256)} 份被钉文件照 {parent} → {moved_parent} 换过（{rewritten} 份里真有这个前缀），真仓登记 {len(real_rows)} 行"
    if not rewritten:
        problems.append(f"被钉文件里一份都没有 {parent} 前缀：这一格什么都没核")
    return not problems, detail + (f"；不对的：{'；'.join(problems)}" if problems else "")


def cell_frozen_o3_moves(scratch: str) -> tuple[bool, str]:
    """冻结执行器核对认搬迁登记的那几条单测（test_frozen_o3，红绿红样本在它的 RegisteredDirectoryMoves）在子进程里跑一遍、全绿；
    本层弄坏开关 frozen-o3-moves-ignored 把 frozen_o3 自己的开关 FROZEN_O3_BREAK=path-moves-ignored 交给那个子进程。"""
    environment = dict(os.environ, TMPDIR=scratch, PYTHONDONTWRITEBYTECODE="1")
    environment.pop(frozen_o3.BREAK_VARIABLE, None)
    if broken("frozen-o3-moves-ignored"):
        environment[frozen_o3.BREAK_VARIABLE] = "path-moves-ignored"
    completed = subprocess.run([sys.executable, "-B", "-m", "unittest", "test_frozen_o3"], cwd=HERE, capture_output=True, text=True,
                               timeout=300, env=environment)
    summary = [line for line in completed.stderr.splitlines() if line.startswith(("Ran ", "OK", "FAILED"))]
    return completed.returncode == 0 and "OK" in summary, f"test_frozen_o3 退 {completed.returncode}：{'；'.join(summary) or '没有汇总行'}"


# ── 倒换只认正着换一遍不动的、逐次倒的两格（pin-moves-partial：钉值核对；frozen-o3-reverse-moves：两份冻结执行器核对）──
PIN_MOVE_MIDDLE = "fixtures/middle-home"
FIRST_RUN_FROZEN_O3 = os.path.join(os.path.dirname(HERE), "e171_three_oracle_collusion", "frozen_o3.py")


def frozen_o3_break_set(items: list[str]):
    """格里把 FROZEN_O3_BREAK 设成 items（逗号连），交一个还原它的函数。"""
    saved = os.environ.get(frozen_o3.BREAK_VARIABLE)
    if items:
        os.environ[frozen_o3.BREAK_VARIABLE] = ",".join(items)
    else:
        os.environ.pop(frozen_o3.BREAK_VARIABLE, None)

    def restore() -> None:
        if saved is None:
            os.environ.pop(frozen_o3.BREAK_VARIABLE, None)
        else:
            os.environ[frozen_o3.BREAK_VARIABLE] = saved
    return restore


def two_home_text(first: str, second: str) -> str:
    return f'RUN = "{first}/target/run"\nDUMPS = "{second}/target/dumps"\n'


def cell_pin_moves_partial(scratch: str) -> tuple[bool, str]:
    """钉值核对（pinned_sha256_now）：钉住的那一版写着两处旧前缀，登记了这一次搬迁。两处都换了：认；只换了一处（另一处还是旧前缀）：不认；
    两行串起来的搬迁、钉在两次之间（钉住的那一版写中间那一段前缀）：认。弄坏开关 pin-partial-accepted 在这一格里设 FROZEN_O3_BREAK=partial-accepted。"""
    restore = frozen_o3_break_set(["partial-accepted"] if broken("pin-partial-accepted") else [])
    try:
        pinned_text = two_home_text(PIN_MOVE_OLD, PIN_MOVE_OLD)
        pinned = sha256_bytes(pinned_text.encode("utf-8"))
        path = os.path.join(scratch, PIN_MOVE_NEW, "pinned_layer.py")
        write_move_table(scratch, [(PIN_MOVE_OLD, PIN_MOVE_NEW)])
        write_text(path, two_home_text(PIN_MOVE_NEW, PIN_MOVE_NEW))
        checks = [("两处都换了：认", pinned_sha256_now(path, pinned, scratch) is not None)]
        write_text(path, two_home_text(PIN_MOVE_NEW, PIN_MOVE_OLD))
        checks.append(("只换了一处：不认", pinned_sha256_now(path, pinned, scratch) is None))
        write_move_table(scratch, [(PIN_MOVE_OLD, PIN_MOVE_MIDDLE), (PIN_MOVE_MIDDLE, PIN_MOVE_NEW)])
        midway = sha256_bytes(two_home_text(PIN_MOVE_MIDDLE, PIN_MOVE_MIDDLE).encode("utf-8"))
        write_text(path, two_home_text(PIN_MOVE_NEW, PIN_MOVE_NEW))
        checks.append(("钉在两次搬迁之间：认", pinned_sha256_now(path, midway, scratch) is not None))
        write_text(path, two_home_text(PIN_MOVE_NEW, PIN_MOVE_MIDDLE))
        checks.append(("串起来的搬迁、还留着中间那一段前缀：不认", pinned_sha256_now(path, midway, scratch) is None))
    finally:
        restore()
    wrong = [name for name, ok in checks if not ok]
    return not wrong, f"{len(checks)} 项" + (f"，不对的：{'；'.join(wrong)}" if wrong else "都对")


def first_run_frozen_o3() -> types.ModuleType:
    """E171 第 1 次那一份 frozen_o3.py（两个实验各留一份、不互相 import；这一格按路径载入它，与本目录那一份一起判）。"""
    name = "frozen_o3_of_the_first_run"
    if name not in sys.modules:   # dataclass 按模块名到 sys.modules 里找这一份的名字空间，载入之前先放进去
        specification = importlib.util.spec_from_file_location(name, FIRST_RUN_FROZEN_O3)
        module = importlib.util.module_from_spec(specification)
        sys.modules[name] = module
        specification.loader.exec_module(module)
    return sys.modules[name]


def frozen_seed_of(texts: dict[str, str]) -> str:
    hasher = hashlib.sha256()
    for name in sorted(texts):
        hasher.update(texts[name].encode("utf-8"))
    return hasher.hexdigest()[:16]


def frozen_differences_with(module: types.ModuleType, root: str, frozen: dict[str, str], current: dict[str, str], rows: list[tuple[str, str]]) -> list[str]:
    """假仓 root 里照 module 的执行器目录放 current，冻结行照 frozen 现算，登记 rows，交 module.freeze_differences。"""
    directory = os.path.join(root, module.FROZEN_DIRECTORY_RELATIVE)
    for name, text in current.items():
        write_text(os.path.join(directory, name), text)
    write_move_table(root, rows)
    lines = [f"name=freeze file={name} sha256={sha256_bytes(text.encode('utf-8'))}" for name, text in sorted(frozen.items())]
    lines.append(f"name=freeze_seed sha256_prefix16={frozen_seed_of(frozen)}")
    return module.freeze_differences(root, lines, {})


def cell_frozen_o3_reverse_moves(scratch: str) -> tuple[bool, str]:
    """两份冻结执行器核对（本目录的 frozen_o3 与 E171 第 1 次那一份）：冻结那一版写着两处旧前缀，登记了这一次搬迁。两处都换了：文件与种子都不记不同；
    只换了一处：文件与种子各记一条不同；两行串起来的搬迁、冻结在两次之间（冻结那一版写中间那一段前缀）：不记不同。
    弄坏开关 frozen-o3-partial-accepted、frozen-o3-whole-chain-only 在这一格里设 FROZEN_O3_BREAK 的同名项（去掉前缀 frozen-o3-）。"""
    items = [item.removeprefix("frozen-o3-") for item in ("frozen-o3-partial-accepted", "frozen-o3-whole-chain-only") if broken(item)]
    restore = frozen_o3_break_set(items)
    differs = ["executor_main.py:sha256_differs", "freeze_seed:differs"]
    core = {"executor_core.py": "def judge(): return 'green'\n"}
    problems, judged = [], 0
    try:
        for label, module in (("本目录那一份", frozen_o3), ("E171 第 1 次那一份", first_run_frozen_o3())):
            cases = (("两处都换了", two_home_text(PIN_MOVE_OLD, PIN_MOVE_OLD), two_home_text(PIN_MOVE_NEW, PIN_MOVE_NEW), [(PIN_MOVE_OLD, PIN_MOVE_NEW)], []),
                     ("只换了一处", two_home_text(PIN_MOVE_OLD, PIN_MOVE_OLD), two_home_text(PIN_MOVE_NEW, PIN_MOVE_OLD), [(PIN_MOVE_OLD, PIN_MOVE_NEW)], differs),
                     ("冻结在两次搬迁之间", two_home_text(PIN_MOVE_MIDDLE, PIN_MOVE_MIDDLE), two_home_text(PIN_MOVE_NEW, PIN_MOVE_NEW),
                      [(PIN_MOVE_OLD, PIN_MOVE_MIDDLE), (PIN_MOVE_MIDDLE, PIN_MOVE_NEW)], []))
            for name, frozen_text, current_text, rows, expected in cases:
                root = tempfile.mkdtemp(dir=scratch)
                got = frozen_differences_with(module, root, dict(core, **{"executor_main.py": frozen_text}), dict(core, **{"executor_main.py": current_text}), rows)
                judged += 1
                if got != expected:
                    problems.append(f"{label}·{name}：记了 {got}，应当 {expected}")
    finally:
        restore()
    return not problems, f"判了 {judged} 种局面" + (f"，不对的：{'；'.join(problems)}" if problems else "，都对")


SELFTEST_CELLS = {
    "rp1-two-passes": cell_rp1_two_passes, "rp1-layers-e2e": cell_rp1_layers_end_to_end,
    "rp3-kind": cell_rp3_kind, "rp3-build": cell_rp3_build, "rp3-closure": cell_rp3_closure, "rp3-closure-mutated-content": cell_rp3_closure_mutated_content,
    "rp3-env": cell_rp3_env, "rp3-mutation": cell_rp3_mutation, "rp3-args": cell_rp3_args, "rp3-limit": cell_rp3_limit, "rp3-upstream": cell_rp3_upstream,
    "rp3-research": cell_rp3_research, "rp3-python": cell_rp3_python,
    "rp4-resume-fetch": cell_rp4_resume_fetch, "rp5-atomic-insert": cell_rp5_atomic_insert,
    "rp6-closure-extra-file": cell_rp6_closure_extra_file, "rp6-closure-outside-the-tree": cell_rp6_closure_outside_the_tree,
    "rp7-check-mismatch": cell_rp7_check_mismatch, "rp8-corrupted-entry": cell_rp8_corrupted_entry,
    "pin-moves": cell_pin_moves, "pin-moves-real-files": cell_pin_moves_real_files, "frozen-o3-moves": cell_frozen_o3_moves,
    "pin-moves-partial": cell_pin_moves_partial, "frozen-o3-reverse-moves": cell_frozen_o3_reverse_moves,
}
SELFTEST_SWITCHES = {**{f"drop-{name}": f"rp3-{name}" for name in SECTION_ORDER}, "check-skipped": "rp7-check-mismatch",
                     "closure-check-skipped": "rp6-closure-extra-file", "manifest-unchecked": "rp8-corrupted-entry", "insert-in-place": "rp5-atomic-insert",
                     "fresh-always-ignored": "rp1-two-passes", "pin-moves-ignored": "pin-moves", "frozen-o3-moves-ignored": "frozen-o3-moves",
                     "pin-partial-accepted": "pin-moves-partial", "frozen-o3-partial-accepted": "frozen-o3-reverse-moves",
                     "frozen-o3-whole-chain-only": "frozen-o3-reverse-moves"}


def selftest(argument_list: list[str]) -> int:
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, SELFTEST_CELLS, SELFTEST_SWITCHES, BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
