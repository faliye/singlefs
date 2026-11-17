#!/usr/bin/env python3
"""E171 第 1 次重跑·修订 11 那三层编排（segment_one_units_revision_11.py、rows_orchestrator_revision_11.py、rows_orchestrator_segment_two_revision_11.py）
与计数脚本 count_reusable_units.py 共用的复用那一层：每个读数单元（装置、E170 导出装置、执行器、变异驱动的一次起法，逐行驱动的一行）一项，
判定只存进 prune-server 那一份判定库（.claude/rules/verification.md「判定库只有一份」）；算键、读写、复用哨兵都经
scripts/experiment-apparatus/long_run_item_verdicts.py（它文件头「请求文件」「判定库」「一趟的三步」「复用哨兵」），这里不另写键的算法、不另存条目。
判法（装置怎么跑、O3 怎么判、比对器怎么比、读数相等怎么比）不在这一层；取来的单元判法照跑。

按项判定：
  种类 KIND（装置名）；项名是单元名把 [A-Za-z0-9._-] 之外的字符换成「.」（item_name）。一项的请求（reading_unit_request、row_request）：
    入口  装置、E170 导出装置、驱动的一行：被判那棵冻结树（要在仓根底下）里那个 bin 的 main（root:<树>、profile:release），树里 crates/*/src/bin/<bin>.rs
          恰好一份才算得出；执行器（第 5、8 步与第 3 步 O3 阳性对照）：冻结执行器 executor_main.py；变异驱动：本目录 mutation_driver.py
    judge: 判这一单元的那几个函数：装置 device_timing.py::run_and_time；驱动的一行 row_driver.py::run_row_on_the_lane，跑执行器的行另加冻结执行器；
          执行器 frozen_o3.py::run_executor，O3 阳性对照 o3_bad_image_judge.py::run_control
    line: kind、装置或驱动的参数（normalized_arguments：去掉落点、线程、线道与已在别处进键的几样）、限时 L（不限时写 none）、上游读数的摘要
          （line:upstream:<角色>=<sha256>，reading_digest：文件按内容，目录按每个文件的相对路径与内容）；驱动的一行另有行号、行表那一行原文的 sha256
          （空变异写 empty-of <名>）、变异表那一行原文的 sha256（改坏之后的源码不进闭包，由变异行原文进键）；line:compare=timing（读数里带耗时）
    @input、@files  跑冻结执行器的项另把冻结行那份产物进键；变异驱动把研究一侧目录每个文件与它的变异表进键
    键 = long_run_item_verdicts 照这些请求经 admission.py 算的那一把；路径、名字、在副本还是主树都不进键。值是读数按角色确定地打成的 tar（gzip，packed_reading）：
    成员按名字排，修改时刻、属主与 gzip 头的时刻置 0，同一份读数两次打出逐字节相同。
  这一层自己拒算（不进计划、照跑、不复用；账本记 fresh-always），缘由按这个次序取第一样：tree-outside-the-repository（冻结树不在仓根底下）、
    entry-not-found（树里找不到那个 bin）、env-set（树里 crates 下源码按名字读的 SINGLEFS_、KS_、RAYON_ 字面量有一个此刻设着，任务服务设的线程数变量那八个除外）、
    second-run（RC2 的第二趟：step-9、step-10-run-2、第一段 row-0 与跨机 X、第二段跨机那一行）、not-located（按名字取不到或不唯一）、anchor-not-once（锚点不是恰 1 次）、
    touches-module-set（改坏那一处碰到模块集合：原文或替换文带 MODULE_SET_TOKENS）。admission 自己拒算的（闭包收不全）照 long_run_item_verdicts 记拒算。
  ItemRun 是一层一次计划（一个单元的几个读数、一步的行）：plan（算键、查库、抽复用哨兵）→ 取来的照角色解进这一趟的名字（fetch）→ 照跑完的 record
  （入库条件过判绿、不过判红：记红撤绿）→ finish（重算键、判复用哨兵）；状态目录是这一趟新建的，finish 之后删。
  输出根与行目录只是这一趟的产物，不是复用来源：做没做完只认这一趟的计划（取来的）与这一趟跑完的。
复用对不对的几条核对（读数照 reading_differences 比，不比字节；不同就是 S18：那一把键撤成红（long_run_item_verdicts.write_record 照同一把键写 red）、
这一层记红停下，交主 agent）：
  RC1  现判的一项入库时，库里这把键已有判绿的那一份（复用哨兵抽中的强制现判那一条，或计划之后别的趟先入了）：比（kind=forced、insert）
  RC2  second-run 那几项跑完：库里这把键有判绿的就比（kind=second-run），没有就照判绿写进去
  RC3  第二段整表取来的行里照 rows_sample 挑 k 行照跑、与库里那一份比（kind=sample），不入库
  复用哨兵、键在跑的中途变了：long_run_item_verdicts 的 finish 判，它退 1 记一行 name=reuse_sentinel problem=finish-red；取来的单元照现跑交了 record 记一行
  name=reuse_sentinel problem=reused-rejudged；两样同 S18。库里那一份读不回、sha256 对不上：long_run_item_verdicts 的 plan 当没有、现判。
账本：<导出根>/reuse-markers/<层>/ 下每个单元一份标记、events.tsv 收 RC1–RC3、哨兵与入库的每一次；每层收尾由标记现合 E7RESULT name=reuse 各行、事件原样与末行
  name=verdict（N + M 不等于这一层登记的单元数判红）；末行另报 judged_now（现判）、reused（复用）、refused（拒算），三个数加起来等于 units。

用法：
  reuse_library.py fetch --library <状态目录> --entry <项名> --map <角色>=<目标路径> [--map …]
      单元脚本取用：编排给的这一趟状态目录里那一项取来的读数按角色解到这一趟的名字（目标已在、值里没有这个角色都拒、退 1）；标准输出一行它的退出码。
  reuse_library.py selfproof --library <目录> --binary B --scratch S --output O --threads N [--repo R]
      RP2：H7 那个装置二进制跑三次 stream-constants --streams 1,2,3，这一趟独占一把键（line:selfproof-run=<日期-进程号-随机>）：① 照现判入库
      ② 同一把键取来、与①逐字节比 ③ 另一个目录现跑、照读数相等比；跑完删掉库里这一把键的两条。三趟的状态目录放 --library 底下。
  reuse_library.py items --repo R --tree T --script S --units <单元>,<单元>
      只在自证里认（临时 prune-server 底下、LONG_RUN_ITEMS_SELFTEST_FAKE_JUDGE=1，不是就退 2）：点名的单元照这一份的请求写法经 judge_items 走一趟，
      给 long_run_item_verdicts.py reuse-cells 当起法（本文件与三层编排自证的 reuse-three-questions）。
  reuse_library.py --selftest [--cells 格,…] [--rounds N]
      假仓（研究一侧目录拷一份，判法闭包照真文件算；.claude 与 scripts 下别的目录指回真仓）、假树、假装置与假驱动行、假判法；装置入口经
      LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成 long_run_item_verdicts 的假入口。只在临时 prune-server 上跑（客户端文件头「命令行」--with-temporary-service；
      开跑前调 selftest_service_problem()，不是临时服务退 2、一格都不跑，不写真服务）。末尾逐个打开弄坏开关核红在点名格；末行「自证 N 项，红 M」。
退出码：0 成；1 取用不成或 RP2 不过；2 用法错、自证没有临时服务、items 不在自证里；75 prune-server 不可用（客户端的码）。

弄坏开关 REUSE_BREAK=<项>（逗号连几个；只在 --selftest 起的子进程里认，REUSE_SELFTEST=1；别处设了退 2）。逐项写开关名、弄坏哪一处，下一行起写打开后红的格（自证打「✗ 自证 <格名>：…」）：
  REUSE_BREAK=kind-unkeyed  请求里不写 line:kind=
      「✗ 自证 request-kind：」
  REUSE_BREAK=arguments-unkeyed  请求里不写 line:arguments=
      「✗ 自证 request-arguments：」
  REUSE_BREAK=limit-unkeyed  请求里不写 line:limit-seconds=
      「✗ 自证 request-limit：」
  REUSE_BREAK=upstream-unkeyed  请求里不写 line:upstream:<角色>=
      「✗ 自证 request-upstream：」
  REUSE_BREAK=mutation-unkeyed  驱动的一行不写行表那一行与变异行的 sha256
      「✗ 自证 request-mutation：」
  REUSE_BREAK=judges-dropped  请求里不写 judge:
      「✗ 自证 request-judge-closure：」
  REUSE_BREAK=freeze-reference-unkeyed  跑执行器的项不写冻结行那份产物的 @input
      「✗ 自证 request-freeze-reference：」
  REUSE_BREAK=environment-set-ignored  树里源码读的变量此刻设着也不拒算
      「✗ 自证 request-environment-set：」
  REUSE_BREAK=module-set-keyed  改坏那一处碰到模块集合的行也进计划
      「✗ 自证 request-module-set-refused：」
  REUSE_BREAK=location-in-key  请求里另写冻结树的绝对路径（键随树所在的目录变）
      「✗ 自证 key-independent-of-location：」
  REUSE_BREAK=packing-keeps-times  值照原文件的修改时刻与此刻打
      「✗ 自证 packing-deterministic：」
  REUSE_BREAK=second-run-planned  second-run 那几项也进计划（取得到就复用）
      「✗ 自证 rp1-two-passes：」
  REUSE_BREAK=forced-reading-unchecked  强制现判那一条跑完不与库里那一份比读数（RC1 forced）
      「✗ 自证 forced-reading-check：」
  REUSE_BREAK=second-run-unchecked  second-run 那几项跑完不比、不入（RC2）
      「✗ 自证 second-run-check：」
  REUSE_BREAK=pin-moves-ignored  钉值核对不看搬迁登记（pinned_sha256_now 只认逐字节相同）
      「✗ 自证 pin-moves：」
  REUSE_BREAK=frozen-o3-moves-ignored  冻结执行器核对不看搬迁登记（子进程设 FROZEN_O3_BREAK=path-moves-ignored）
      「✗ 自证 frozen-o3-moves：」
  REUSE_BREAK=pin-partial-accepted  钉值核对倒换之前不核此刻的内容正着换一遍动不动（格里设 FROZEN_O3_BREAK=partial-accepted）
      「✗ 自证 pin-moves-partial：」
  REUSE_BREAK=frozen-o3-partial-accepted  冻结执行器核对同上（两份 frozen_o3，格里设 FROZEN_O3_BREAK=partial-accepted）
      「✗ 自证 frozen-o3-reverse-moves：」
  REUSE_BREAK=frozen-o3-whole-chain-only  冻结执行器核对只倒全部登记那一版、不逐次倒（格里设 FROZEN_O3_BREAK=whole-chain-only）
      「✗ 自证 frozen-o3-reverse-moves：」
  REUSE_BREAK=sweep-skipped  每一格之后与末尾不收进程（自证外壳 revision_8_selftest_harness.py 的开关）
      「✗ 自证 no-leftover-processes：」
  REUSE_BREAK=temporary-kept  末尾不删临时目录（自证外壳 revision_8_selftest_harness.py 的开关）
      「✗ 自证 temporary-directory-removed：」
另认 long_run_item_verdicts.py 的弄坏开关 LONG_RUN_ITEMS_BREAK（它文件头；reuse-three-questions 那一格起的 reuse-cells 照它坏）。
"""
from __future__ import annotations

import argparse
import contextlib
import dataclasses
import glob
import gzip
import hashlib
import importlib.machinery
import importlib.util
import io
import math
import os
import re
import secrets
import shutil
import subprocess
import sys
import tarfile
import tempfile
import time
import types

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, HERE)
sys.path.append(os.path.join(HERE, "..", "..", "prune-server"))
sys.path.append(os.path.join(HERE, ".."))   # 实验装置目录：按项判定的那一份 long_run_item_verdicts.py
import revision_8_selftest_harness as harness  # noqa: E402
import frozen_o3  # noqa: E402（搬迁登记的读法：钉值核对与冻结执行器核对共用 frozen_o3 那一份）
import prune_server_client  # noqa: E402（自证开跑前问在不在临时 prune-server 底下）
import long_run_item_verdicts as items_library  # noqa: E402（按项判定：键、库、复用哨兵都在它那里，文件头「按项判定」）

BREAK_VARIABLE = "REUSE_BREAK"
SELFTEST_VARIABLE = "REUSE_SELFTEST"
RESEARCH_RELATIVE = "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes"
SCRIPT_KEY = f"{RESEARCH_RELATIVE}/reuse_library.py"
EXECUTOR_RELATIVE = "scripts/experiment-apparatus/e170_o3_spec_executor_with_witness_predicates_journal_header_487"
FREEZE_REFERENCE_RELATIVE = "research/results/e170-r2-o3-spec-executor-2026-11-05-refreeze-executor.out"
MUTATIONS_TABLE_RELATIVE = "scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes.tsv"
DEVICE_BINARY = "e171_three_oracle_collusion_after_the_oracle_fixes"
E170_BINARY = "e170_o3_spec_executor_export"
KIND = DEVICE_BINARY   # long_run_item_verdicts 的种类（装置名）
UNIT_KINDS = ("device", "e170", "driver-row", "o3-executor", "mutation-driver")
# 判这一单元的那几个函数与程序（文件头「按项判定」judge:）
DEVICE_JUDGE = f"{RESEARCH_RELATIVE}/device_timing.py::run_and_time"
ROW_JUDGE = f"{RESEARCH_RELATIVE}/row_driver.py::run_row_on_the_lane"
EXECUTOR_JUDGE = f"{RESEARCH_RELATIVE}/frozen_o3.py::run_executor"
CONTROL_JUDGE = f"{RESEARCH_RELATIVE}/o3_bad_image_judge.py::run_control"
SELFPROOF_JUDGE = f"{RESEARCH_RELATIVE}/reuse_library.py::selfproof_run"
EXECUTOR_PROGRAM = f"{EXECUTOR_RELATIVE}/executor_main.py"
MUTATION_DRIVER_PROGRAM = f"{RESEARCH_RELATIVE}/mutation_driver.py"
# 任务服务设的线程数变量里的八个（取这一张表时 scripts/hosts/ 下线程上限脚本第 22–31 行那一份）：不当「树里源码读的变量设着」（线程数在键外）
THREAD_COUNT_VARIABLES = ("CARGO_BUILD_JOBS", "RUST_TEST_THREADS", "SINGLEFS_LAYER0_THREADS", "SINGLEFS_CRASH_INJECTION_THREADS",
                          "SINGLEFS_FAULT_INJECTION_WORKER_THREADS", "SINGLEFS_BAD_DISK_INPUT_WORKER_THREADS",
                          "SINGLEFS_RANDOM_HISTORY_THREADS", "SINGLEFS_THREAD_CAP")
ENVIRONMENT_NAME_PATTERN = re.compile(r'"(SINGLEFS_[A-Z0-9_]+|KS_[A-Z0-9_]+|RAYON_[A-Z0-9_]+)"')
SELECTED_ROW_TABLES = ("bad-implementations-step-8.tsv", "bad-implementations-step-4-only.tsv")
# 改坏的那一处碰到模块集合的：未改坏的闭包加变异行原文对不上改坏之后的闭包，拒算、永远现跑
MODULE_SET_TOKENS = ("mod ", "#[path", "include_str!", "include_bytes!", "include!")
TIMEOUT_EXIT_CODE = 253
WRAPPER_OWN_EXIT_CODES = (250, 251, 252, 254)
# 读数相等：驱动行的 row-result.tsv 去掉这几个键再比
ROW_RESULT_VOLATILE_KEYS = ("copy_ns", "compile_ns", "device_ns", "o3_ns", "lane", "compile_kind", "packages_checked_rebuilt")
REBUILD_CHECK_FIELD = "packages_checked_rebuilt"
STATE_FRESH, STATE_REUSED, STATE_FRESH_CHECK, STATE_FRESH_ALWAYS = "fresh", "reused", "fresh-check", "fresh-always"
SECOND_RUN, SAMPLE_REASON = "second-run", "sample"
READING_ROLES = ("product", "timing", "directory", "row")   # 读数的几个角色：单元脚本记下的别的角色不是读数，不进值
FORCED_PREFIX = "强制现判"   # long_run_item_verdicts 的 plan 给复用哨兵抽中的那一条写的缘由的开头
UNIT_NAME_OUTSIDE_ITEM_NAMES = re.compile(r"[^A-Za-z0-9._-]")


def broken(item: str) -> bool:
    return harness.broken(BREAK_VARIABLE, item)


def selftest_only(name: str) -> str:
    """只在自证起的子进程里认的换假件变量；平常交回空串。"""
    return os.environ.get(name, "") if os.environ.get(SELFTEST_VARIABLE) == "1" else ""


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


def item_name(unit: str) -> str:
    """单元名 → long_run_item_verdicts 的项名（只许字母、数字、. _ -）。"""
    return UNIT_NAME_OUTSIDE_ITEM_NAMES.sub(".", unit)


# ── 请求里的内容输入（文件头「按项判定」line:）──
# 参数里不进键的那几项（带一个值）：输出与工作的落点、线程、线道，与已经在别的 line: 里的输入（树是入口闭包、基线计时是限时、上游是上游的摘要）
VOLATILE_ARGUMENT_OPTIONS = ("--output-directory", "--dump-directory", "--output-root", "--work-root", "--threads", "--lanes",
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


def reading_digest(location: str) -> str:
    """一份读数（文件或目录）的摘要：文件按内容；目录按每个文件「相对路径 内容的 sha256」排好的那几行。不在写「不在」。"""
    if os.path.isdir(location):
        return sha256_bytes("".join(f"{relative} {sha256_file(os.path.join(location, relative))}\n" for relative in files_under(location)).encode("utf-8"))
    return sha256_file(location) if os.path.isfile(location) else "不在"


def upstream_tokens(upstream: list[tuple[str, str]]) -> list[str]:
    """(角色, 上游读数的目录或文件) → line:upstream:<角色>=<摘要>（按读数的内容，不按上游的键）。"""
    if broken("upstream-unkeyed"):
        return []
    return [f"line:upstream:{role}={reading_digest(location)}" for role, location in upstream]


def limit_tokens(seconds: int | None) -> list[str]:
    """逐行限时 L 的秒数；不限时的单元写 none。"""
    if broken("limit-unkeyed"):
        return []
    return [f"line:limit-seconds={'none' if seconds is None else seconds}"]


def kind_tokens(kind: str) -> list[str]:
    return [] if broken("kind-unkeyed") else [f"line:kind={kind}"]


def arguments_tokens(text: str) -> list[str]:
    return [] if broken("arguments-unkeyed") else [f"line:arguments={text}"]


def judge_tokens(judges: list[str]) -> list[str]:
    return [] if broken("judges-dropped") else [f"judge:{judge}" for judge in judges]


def freeze_reference_lines(item: str) -> list[str]:
    """跑冻结执行器的项：冻结行那份产物经 @input 进键（long_run_item_verdicts 文件头「请求文件」）。"""
    return [] if broken("freeze-reference-unkeyed") else [f"@input\t{item}\tfreeze-reference\t{FREEZE_REFERENCE_RELATIVE}"]


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
# 冻结层（rows_orchestrator_revision_8.py、rows_orchestrator_segment_two.py、rows_orchestrator_segment_two_revision_10.py、cross_host_o3_check.py）
# 正文里钉的第一段编排器、跨机对照入口、第二段编排器的 sha256，与这里的 ORCHESTRATOR_SHA256、CROSS_HOST_CHECK_SHA256、SEGMENT_TWO_ORCHESTRATOR_SHA256
# 是同一组值，都是被钉文件此刻的内容：跑那一趟的第一段编排器 1dcfb8d0… 那一版与此刻差的不只是登记过的搬迁（每台线程改取登记值等），倒换不回去，
# 所以冻结层与这里一起按此刻内容重钉。修订 11 载入冻结层时先核它们自己的 sha256（load_repinned），再把正文里的钉值换成此刻该认的那一个编（RepinnedLoader）。
# 之后整个目录再搬家、登记在 .claude/gate.d/path-moves.tsv 的（读法在 frozen_o3 文件头「搬迁登记」）：钉值一个不改。核 sha256 时
# （pinned_sha256_now）此刻的内容不是钉住的那一份，就把此刻内容里登记过的新前缀从最近一次起逐次倒着换回旧前缀再算，有一步是钉值就认（只差登记过的前缀）；
# 此刻的内容照全部登记正着换一遍要不动，只换了一半的不倒、不认；
# 冻结层正文里要换的钉值（orchestrator_repin、rows_revision_8_repins）换成被钉文件此刻的 sha256（认的时候），冻结层自己按字节比的那一处照过；
# 不认的还是换成钉值，那一处照旧红。取「把此刻的内容换回去」这一种，不取「把钉住那一版正着换」：钉住那一版的内容这里没有，只有它的 sha256。
# 用全部登记行倒换（不只此刻生效的）：换回去之后逐字节等于钉住的那一版，此刻与它就只差登记过的前缀；还改了别处的换回去也对不上。登记表不在或没有内容行时与改前逐字节同判。
# 关不住的一种：登记表里留着一行老行（旧前缀下已没有文件、新前缀是早就在的另一份实现），再加一行真搬迁，钉住的文件改指那一份早就在的实现，换回去照样等于钉值，这里认；
# 门禁一侧这份文件照算改动，要人看（.claude/rules/path-moves.md「门禁管哪一半」）。换算只认仓根起的前缀（lib-lag-baseline.py 文件头「搬迁登记」）。
ORCHESTRATOR_SHA256 = "00ffd24c3a85361f0ff0355c4ac2cc2efded989c308842f1b2ac6a5ca85b076b"
SEGMENT_TWO_ORCHESTRATOR_SHA256 = "ce9ef2766680cb2305221020d03419b5ff49e447293e61b4a3098a9be6274964"
# 冻结的修订 8（rows_orchestrator_revision_8.py 的 CROSS_HOST_CHECK_SHA256）钉的跨机对照入口：载入修订 8 时同样按此刻认不认换
CROSS_HOST_CHECK_SHA256 = "7d071b620fed9861b2cb1f2c5a791c8741fd3979aec0cbb88042bca463bd1b20"
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
    """冻结层正文里第一段编排器的钉值（同 ORCHESTRATOR_SHA256）→ 此刻该认的那一个（rows_orchestrator_segment_two.py、rows_orchestrator_revision_8.py 载入时换）。"""
    return ((ORCHESTRATOR_SHA256, orchestrator_sha256_now(directory, root)),)


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


# ── 一棵冻结树上各单元的请求（文件头「按项判定」）──
@dataclasses.dataclass
class TreeFacts:
    """一棵冻结树上与单元无关的几样（每趟现算一次）：入口相对树根的路径（找不到是 None）、此刻设着的、树里源码按名字读的变量、变异表。"""
    repo: str
    tree: str
    tree_relative: str
    device_entry: str | None
    e170_entry: str | None
    environment_set: list[str]
    mutation_rows: dict
    mutation_name_counts: dict


def bin_entry(tree: str, binary: str) -> str | None:
    """树里 crates/*/src/bin/<bin>.rs 恰好一份：交它相对树根的路径；没有或不止一份交 None（那几项照 entry-not-found 拒算）。"""
    found = sorted(glob.glob(os.path.join(tree, "crates", "*", "src", "bin", binary + ".rs")))
    return os.path.relpath(found[0], tree) if len(found) == 1 else None


def environment_names_in(tree: str) -> list[str]:
    """树里 crates 下每份 .rs 按名字读的 SINGLEFS_、KS_、RAYON_ 字面量，去掉任务服务设的线程数变量那八个。"""
    names: set[str] = set()
    for directory, subdirectories, files in os.walk(os.path.join(tree, "crates")):
        subdirectories[:] = sorted(name for name in subdirectories if name != "target")
        for name in files:
            if name.endswith(".rs"):
                with open(os.path.join(directory, name), encoding="utf-8", errors="replace") as handle:
                    names |= set(ENVIRONMENT_NAME_PATTERN.findall(handle.read()))
    return sorted(names - set(THREAD_COUNT_VARIABLES))


def tree_facts(repo: str, tree: str, environment: dict[str, str]) -> TreeFacts:
    """tree 是绝对路径或相对 repo 的路径；environment 是这一趟起单元的环境（判 env-set）。"""
    import bad_implementation_table  # noqa: E402（本目录）
    absolute_repo = os.path.abspath(repo)
    absolute = tree if os.path.isabs(tree) else os.path.join(absolute_repo, tree)
    rows = bad_implementation_table.read_mutation_rows(os.path.join(absolute, "crates", "mutations.tsv"))
    counts: dict[str, int] = {}
    for row in rows:
        counts[row.name] = counts.get(row.name, 0) + 1
    names = [] if broken("environment-set-ignored") else environment_names_in(absolute)
    return TreeFacts(absolute_repo, absolute, os.path.relpath(absolute, absolute_repo), bin_entry(absolute, DEVICE_BINARY), bin_entry(absolute, E170_BINARY),
                     [name for name in names if name in environment], {row.name: row for row in rows}, counts)


@dataclasses.dataclass
class UnitRequest:
    """一个单元的请求：lines 是 long_run_item_verdicts 请求文件里这一项的几行；reason 不空是这一层自己拒算（文件头「按项判定」），不进计划；
    facts 是别的台那三样核对要的（驱动的一行：row、mutation_line_sha256、limit_seconds）。"""
    unit: str
    kind: str
    item: str
    lines: list[str]
    reason: str = ""
    facts: dict = dataclasses.field(default_factory=dict)


def tree_refusal(facts: TreeFacts, entry: str | None) -> str:
    """树那一侧拒算的缘由（文件头「按项判定」那一串的前三样），没有交空串。"""
    if facts.tree_relative.startswith(".."):
        return "tree-outside-the-repository"
    if entry is None:
        return "entry-not-found"
    return "env-set" if facts.environment_set else ""


def program_tokens(facts: TreeFacts, entry: str | None) -> list[str]:
    """装置那一侧的入口：被判的冻结树里那个 bin 的 main。弄坏开关 location-in-key 下另写树的绝对路径。"""
    location = [f"line:location={facts.tree}"] if broken("location-in-key") else []
    return [f"{entry or '-'}::main", f"root:{facts.tree_relative}", "profile:release", *location]


def research_files_lines(repo: str, item: str) -> list[str]:
    """变异驱动：研究一侧目录每个文件（不含 __pycache__）各一条 @files（按内容进键）。"""
    return [f"@files\t{item}\t{os.path.join(RESEARCH_RELATIVE, relative)}" for relative in files_under(os.path.join(repo, RESEARCH_RELATIVE))]


def reading_unit_request(facts: TreeFacts, unit: ReadingUnit, upstream: list[tuple[str, str]]) -> UnitRequest:
    """装置、E170 导出装置、执行器、变异驱动的一个读数单元。upstream 是 (角色, 上游读数的目录或文件)。"""
    item = item_name(unit.name)
    contents = (kind_tokens(unit.kind) + arguments_tokens(normalized_arguments(unit.arguments.split())) + limit_tokens(None) + upstream_tokens(upstream)
                + ["line:compare=timing"])
    if unit.kind in ("device", "e170"):
        entry = facts.device_entry if unit.kind == "device" else facts.e170_entry
        reason = tree_refusal(facts, entry) or (SECOND_RUN if unit.fresh_always else "")
        lines = ["\t".join([item, *program_tokens(facts, entry), *judge_tokens([DEVICE_JUDGE]), *contents])]
    elif unit.kind == "o3-executor":
        judge = CONTROL_JUDGE if unit.arguments.split()[0] == "control" else EXECUTOR_JUDGE
        reason = SECOND_RUN if unit.fresh_always else ""
        lines = ["\t".join([item, EXECUTOR_PROGRAM, *judge_tokens([judge]), *contents])] + freeze_reference_lines(item)
    elif unit.kind == "mutation-driver":
        reason = SECOND_RUN if unit.fresh_always else ""
        lines = ["\t".join([item, MUTATION_DRIVER_PROGRAM, *contents])] + research_files_lines(facts.repo, item) + [f"@files\t{item}\t{unit.arguments}"]
    else:
        raise ValueError(f"认不得的单元种类：{unit.kind}")
    return UnitRequest(unit.name, unit.kind, item, lines, reason)


def row_request(facts: TreeFacts, row: RowUnit, selected_lines: dict[int, str], limit_seconds: int | None, upstream: list[tuple[str, str]]) -> UnitRequest:
    """驱动的一行。拒算的缘由按文件头「按项判定」那一串的次序取第一样。"""
    import bad_implementation_table  # noqa: E402（本目录）
    item = item_name(row.name)
    selected = selected_lines.get(row.number)
    name = (selected_lines[1].split("\t")[4] if row.empty else selected.split("\t")[4]) if (row.empty or selected) else None
    reason = tree_refusal(facts, facts.device_entry) or (SECOND_RUN if row.second_run else "")
    mutation = facts.mutation_rows.get(name) if name else None
    row_facts: dict[str, object] = {"row": row.number, "limit_seconds": limit_seconds}
    contents = kind_tokens("driver-row") + [f"line:row={row.number}"]
    if not broken("mutation-unkeyed"):
        contents.append(f"line:selected-row-sha256={sha256_bytes((f'empty-of {name}' if row.empty else (selected or '-')).encode('utf-8'))}")
    if mutation is None or facts.mutation_name_counts.get(name, 0) != 1:
        reason = reason or "not-located"
    else:
        with open(os.path.join(facts.tree, mutation.file), encoding="utf-8") as handle:
            text = handle.read()
        applied = bad_implementation_table.empty_mutation(mutation) if row.empty else mutation
        if bad_implementation_table.anchor_hit_count(text, applied) != 1:
            reason = reason or "anchor-not-once"
        if not broken("module-set-keyed") and any(token in mutation.original_text() + mutation.replacement_text() for token in MODULE_SET_TOKENS):
            reason = reason or "touches-module-set"
        row_facts["mutation_line_sha256"] = sha256_bytes(mutation.raw_line.encode("utf-8"))
        if not broken("mutation-unkeyed"):
            contents.append(f"line:mutation-sha256={row_facts['mutation_line_sha256']}")
    contents += arguments_tokens(normalized_arguments(list(row.flags))) + limit_tokens(limit_seconds) + upstream_tokens(upstream) + ["line:compare=timing"]
    judges = [ROW_JUDGE] + ([EXECUTOR_PROGRAM] if row.executor else [])
    lines = ["\t".join([item, *program_tokens(facts, facts.device_entry), *judge_tokens(judges), *contents])]
    if row.executor:
        lines += freeze_reference_lines(item)
    return UnitRequest(row.name, "driver-row", item, lines, reason, row_facts)


def unit_request(facts: TreeFacts, unit, selected_lines: dict[int, str], limit_seconds: int | None = None,
                 upstream: tuple[tuple[str, str], ...] = ()) -> UnitRequest:
    """读数单元走 reading_unit_request，驱动的一行走 row_request（limit_seconds 只给行）。"""
    if isinstance(unit, RowUnit):
        return row_request(facts, unit, selected_lines, limit_seconds, list(upstream))
    return reading_unit_request(facts, unit, list(upstream))


def requests_text(requests: list[UnitRequest]) -> str:
    return "".join(line + "\n" for request in requests for line in request.lines)


def keyed_items(repo: str, requests: list[UnitRequest]) -> dict[str, tuple[str | None, str]]:
    """{单元: (键或 None, 拒算说明)}：照 long_run_item_verdicts 的 plan 那样解析请求、自证底下换假入口、算键，不查库（RC2 那几项、plan 子命令、计数脚本用）。"""
    if not requests:
        return {}
    items_library.require_selftest_variables_allowed()
    parsed = items_library.parse_requests(requests_text(requests), "请求")
    if os.environ.get(items_library.SUBSTITUTION_VARIABLE):
        items_library.substitute_entries(parsed, os.environ[items_library.SUBSTITUTION_VARIABLE])
    keyed = items_library.item_keys(repo, parsed)
    return {request.unit: (keyed[request.item][0], keyed[request.item][2]) for request in requests}


# ── 判定库：只经 long_run_item_verdicts 读写（文件头「按项判定」）──
class RunCondition(Exception):
    """运行条件不满足（三层编排的 H31、干跑配置这一类）：调用方退 78。"""


def library_store(repo: str):
    """prune-server 里这个种类那一份（long_run_item_verdicts 的 library_of、library_directory：命名空间只在临时服务底下给）。"""
    return items_library.library_of(KIND, items_library.library_directory(repo))


def stored_value(repo: str, key: str) -> bytes | None:
    """库里这把键判绿、产物读得回且 sha256 对得上的那一份值；别的交 None。"""
    store = library_store(repo)
    record = items_library.parsed_record(store.read(items_library.verdict_relative(key)), key)
    if record is None or record.get("verdict") != "green":
        return None
    output = store.read(items_library.output_relative(key))
    return output if output is not None and sha256_bytes(output) == record.get("output_sha256") else None


def store_green(repo: str, key: str, item: str, exit_code: int, output: bytes) -> None:
    """RC2：second-run 那一项库里还没有判绿的，照判绿写进去（long_run_item_verdicts.write_record）。"""
    items_library.write_record(library_store(repo), key, item, "green", exit_code, items_library.COMPARE_TIMING, SCRIPT_KEY, output)


def withdraw(repo: str, key: str, item: str, reason: str) -> None:
    """S18：这把键照同一条写 red（记红撤绿），下一趟现判。"""
    items_library.write_record(library_store(repo), key, item, "red", -1, items_library.COMPARE_TIMING, SCRIPT_KEY, reason=reason)


def check_service_reachable(repo: str) -> None:
    """运行条件：prune-server 连得上。连不上客户端退 75（PruneServerFailure 是 SystemExit），不当「库里没有」现判。"""
    library_store(repo).read(items_library.verdict_relative("service-reachable"))


# ── 值：读数确定地打成的 tar（文件头「按项判定」）──
def entries_under(directory: str) -> list[str]:
    """目录下每个子目录与文件相对目录的路径，排好序（目录排在它底下的文件前面）。"""
    found = []
    for root, directories, names in os.walk(directory):
        directories[:] = sorted(directories)
        found += [os.path.relpath(os.path.join(root, name), directory) for name in directories + names]
    return sorted(found)


def packed_reading(reading: dict[str, str]) -> bytes:
    """读数（角色 → 文件或目录）打成一份 tar（gzip）：成员是「<角色>」与「<角色>/<相对路径>」，按名字排；修改时刻与属主置 0，文件权限只分有没有执行位，
    gzip 头的时刻置 0。同一份读数两次打出逐字节相同。弄坏开关 packing-keeps-times 下照原文件的修改时刻与此刻打。"""
    keep_times = broken("packing-keeps-times")
    raw = io.BytesIO()
    with tarfile.open(fileobj=raw, mode="w", format=tarfile.GNU_FORMAT) as archive:
        for role in sorted(reading):
            source = reading[role]
            members = [(role, source)] + ([(f"{role}/{relative}", os.path.join(source, relative)) for relative in entries_under(source)] if os.path.isdir(source) else [])
            for name, path in members:
                info = tarfile.TarInfo(name)
                info.mtime = int(os.stat(path).st_mtime) if keep_times else 0
                info.uid = info.gid = 0
                info.uname = info.gname = ""
                if os.path.isdir(path):
                    info.type, info.mode = tarfile.DIRTYPE, 0o755
                    archive.addfile(info)
                    continue
                with open(path, "rb") as handle:
                    data = handle.read()
                info.size, info.mode = len(data), 0o755 if os.access(path, os.X_OK) else 0o644
                archive.addfile(info, io.BytesIO(data))
    return gzip.compress(raw.getvalue(), mtime=None if keep_times else 0)


def value_roles(value: bytes) -> list[str]:
    """值里有哪几个角色（成员名第一段）。"""
    with tarfile.open(fileobj=io.BytesIO(value), mode="r:gz") as archive:
        return sorted({member.name.split("/", 1)[0] for member in archive.getmembers()})


def unpack_reading(value: bytes, targets: dict[str, str]) -> None:
    """值按角色解到这一趟的名字（targets：角色 → 目标路径）：每个角色先解进目标旁边的中转目录，再一次改名换上（行目录整份一起出现）；
    目标已在（FileExistsError）、值里没有这个角色（ValueError）都拒，拒的时候一个目标都不写。"""
    present = [target for target in targets.values() if os.path.lexists(target)]
    if present:
        raise FileExistsError(f"已存在、不覆盖：{', '.join(sorted(present))}")
    missing = sorted(set(targets) - set(value_roles(value)))
    if missing:
        raise ValueError(f"值里没有角色 {','.join(missing)}")
    for role, target in sorted(targets.items()):
        parent = os.path.dirname(os.path.abspath(target))
        os.makedirs(parent, exist_ok=True)
        staging = tempfile.mkdtemp(prefix=".reuse-unpack-", dir=parent)
        try:
            with tarfile.open(fileobj=io.BytesIO(value), mode="r:gz") as archive:
                members = [member for member in archive.getmembers() if member.name == role or member.name.startswith(role + "/")]
                archive.extractall(staging, members=members, filter="data")
            os.rename(os.path.join(staging, role), target)
        finally:
            shutil.rmtree(staging, ignore_errors=True)


def fetch_from_state(state: str, item: str, targets: dict[str, str]) -> int:
    """计划放进状态目录的那一项取来的值（long_run_item_verdicts 的 reused_output）按角色解到 targets；交它的退出码。"""
    exit_code, output = items_library.reused_output(state, item)
    unpack_reading(output, targets)
    return exit_code


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


# ── 决定、标记、事件、账本（文件头「账本」）──
@dataclasses.dataclass
class Decision:
    """一个单元这一趟怎么做：state 是 fresh（现判）、reused（从库里取）、fresh-check（取得到而照跑、只比不入：RC3 抽中）、fresh-always（拒算：照跑、不复用）。
    forced 是复用哨兵抽中的强制现判那一条（照现判，跑完与库里那一份比，RC1）。key 算不出写 -。"""
    unit: str
    kind: str
    item: str
    key: str
    state: str
    reason: str
    forced: bool = False
    facts: dict = dataclasses.field(default_factory=dict)


def value(text: object) -> str:
    """结果行里的一个值：空白换成下划线，空串写 -（同 result_lines.field）。"""
    return "".join("_" if character.isspace() else character for character in str(text)) or "-"


def marker_path(markers: str, unit: str) -> str:
    return os.path.join(markers, re.sub(r"[^A-Za-z0-9._-]", "_", unit) + ".tsv")


def write_marker(markers: str, decision: Decision, extra: dict[str, str] | None = None) -> None:
    """单元的标记：原子写（同目录临时文件再 os.replace）；extra 是单元编排层记下的角色路径（role_<角色>）。"""
    os.makedirs(markers, exist_ok=True)
    fields = {"unit": decision.unit, "kind": decision.kind, "item": decision.item, "key": decision.key, "state": decision.state,
              "reason": value(decision.reason), "forced": str(decision.forced).lower()}
    fields.update(extra or {})
    path = marker_path(markers, decision.unit)
    partial = f"{path}.{os.getpid()}.partial"
    with open(partial, "w", encoding="utf-8") as handle:
        handle.writelines(f"{name}\t{text}\n" for name, text in fields.items())
    os.replace(partial, path)


def read_markers(markers: str) -> dict[str, dict[str, str]]:
    found = {}
    for name in sorted(os.listdir(markers)) if os.path.isdir(markers) else []:
        if name.endswith(".tsv") and name != "events.tsv":
            fields = dict(line.rstrip("\n").split("\t", 1) for line in open(os.path.join(markers, name), encoding="utf-8") if "\t" in line)
            if "unit" in fields:
                found[fields["unit"]] = fields
    return found


def append_event(markers: str, line: str) -> None:
    os.makedirs(markers, exist_ok=True)
    with open(os.path.join(markers, "events.tsv"), "a", encoding="utf-8") as handle:
        handle.write(line + "\n")
        handle.flush()
        os.fsync(handle.fileno())


def event_lines(markers: str) -> list[str]:
    path = os.path.join(markers, "events.tsv")
    return open(path, encoding="utf-8").read().splitlines() if os.path.exists(path) else []


def s18_problems(markers: str) -> list[str]:
    """S18（文件头「复用对不对的几条核对」）：RC1–RC3 任一次 identical=false，或一行 name=reuse_sentinel。交回触发的那几行事件。"""
    return [line for line in event_lines(markers) if (" name=reuse_check " in f" {line} " and " identical=false " in f" {line} ")
            or " name=reuse_sentinel " in f" {line} "]


def ledger_text(markers: str, registered: list[str]) -> tuple[str, bool]:
    """复用账本：登记的每个单元一行 name=reuse，事件原样，末行 name=verdict。交回 (全文, N + M 等于登记的单元数否)。"""
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
        lines.append(f"E7RESULT name=reuse unit={value(unit)} item={value(marker.get('item'))} key={marker['key']} state={state} "
                     f"reason={value(marker.get('reason'))} forced={value(marker.get('forced'))}")
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
                 f"env_set_units={environment_set} registered_units={len(registered)} units_cover_registered={str(covers).lower()} "
                 f"missing={value(','.join(missing[:8]))} unregistered={value(','.join(unregistered[:8]))} "
                 f"judged_now={counts[STATE_FRESH] + counts[STATE_FRESH_CHECK]} refused={counts[STATE_FRESH_ALWAYS]} "
                 f"sentinel_problems={sum(1 for line in events if ' name=reuse_sentinel ' in f' {line} ')}")
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


def differences_from_value(kind: str, stored: bytes, reading: dict[str, str]) -> list[str]:
    """库里那一份值解进临时目录（中转，用完即删）与这一次的读数照 reading_differences 比；解不开按不同算。"""
    directory = tempfile.mkdtemp(prefix="e171-reuse-compare-")
    try:
        with tarfile.open(fileobj=io.BytesIO(stored), mode="r:gz") as archive:
            archive.extractall(directory, filter="data")
        return reading_differences(kind, {role: os.path.join(directory, role) for role in sorted(os.listdir(directory))}, reading)
    except (tarfile.TarError, OSError, EOFError) as error:
        return [f"value-unreadable:{type(error).__name__}"]
    finally:
        shutil.rmtree(directory, ignore_errors=True)


# ── 一层一次计划（文件头「按项判定」ItemRun）──
class ItemRun:
    """requests 里 reason 不空的不进计划（拒算），其中 second-run 那几项另照 keyed_items 算键给 RC2。state_parent 下新建这一趟的状态目录，finish 之后删。"""

    def __init__(self, repo: str, script: str, requests: list[UnitRequest], state_parent: str, markers: str, printer=print) -> None:
        self.repo, self.script, self.markers, self.printer = os.path.abspath(repo), script, markers, printer
        self.requests = {request.unit: request for request in requests}
        os.makedirs(state_parent, exist_ok=True)
        self.state = tempfile.mkdtemp(prefix="items-", dir=state_parent)
        self.planned: list[UnitRequest] = []
        self.decisions: dict[str, Decision] = {}

    def plan(self) -> dict[str, Decision]:
        """算键、查库、抽复用哨兵（long_run_item_verdicts 的 plan），写标记。交回 {单元: 决定}。"""
        self.planned = [request for request in self.requests.values()
                        if not request.reason or (request.reason == SECOND_RUN and broken("second-run-planned"))]
        refused = [request for request in self.requests.values() if request not in self.planned]
        second_runs = keyed_items(self.repo, [request for request in refused if request.reason == SECOND_RUN])
        for request in refused:
            key = second_runs.get(request.unit, (None, ""))[0]
            self.decisions[request.unit] = Decision(request.unit, request.kind, request.item, key or "-", STATE_FRESH_ALWAYS, request.reason, facts=request.facts)
        if self.planned:
            _order, answers = items_library.plan(self.state, self.script, KIND, self.repo, requests_text(self.planned), printer=self.printer)
            items = items_library.read_plan(self.state)["items"]
            for request in self.planned:
                answer, why = answers[request.item]
                key = items[request.item]["key"] or "-"
                if answer == items_library.REUSE:
                    decision = Decision(request.unit, request.kind, request.item, key, STATE_REUSED, "hit", facts=request.facts)
                elif answer == items_library.REFUSED:
                    decision = Decision(request.unit, request.kind, request.item, key, STATE_FRESH_ALWAYS, "refused", facts=request.facts)
                else:
                    decision = Decision(request.unit, request.kind, request.item, key, STATE_FRESH, why, why.startswith(FORCED_PREFIX), request.facts)
                self.decisions[request.unit] = decision
        for decision in self.decisions.values():
            write_marker(self.markers, decision)
        return dict(self.decisions)

    def mark_sample(self, unit: str) -> None:
        """RC3：取得到的这一项照跑、只比不入（fresh-check）。"""
        self.decisions[unit] = dataclasses.replace(self.decisions[unit], state=STATE_FRESH_CHECK, reason=SAMPLE_REASON)
        write_marker(self.markers, self.decisions[unit])

    def fetch(self, unit: str, targets: dict[str, str]) -> int:
        """取来的那一项按角色解到 targets；交它的退出码。"""
        return fetch_from_state(self.state, self.decisions[unit].item, targets)

    def fetch_into(self, unit: str, directory: str) -> tuple[int, dict[str, str]]:
        """取来的那一项每个角色解进 directory/<角色>；交 (退出码, 角色 → 路径)。"""
        _exit, output = items_library.reused_output(self.state, self.decisions[unit].item)
        targets = {role: os.path.join(directory, role) for role in value_roles(output)}
        return self.fetch(unit, targets), targets

    def stored(self, unit: str) -> bytes | None:
        """库里这一项判绿的那一份：计划放进状态目录的（取来的、RC3 抽中的）取那一份，别的现读库。"""
        decision = self.decisions[unit]
        path = os.path.join(self.state, items_library.OUTPUTS_DIRECTORY, decision.item + ".out")
        if os.path.exists(path):
            with open(path, "rb") as handle:
                return handle.read()
        return stored_value(self.repo, decision.key) if decision.key != "-" else None

    def compare(self, decision: Decision, stored: bytes, reading: dict[str, str], check_kind: str) -> bool:
        """RC1–RC3：读数与库里那一份比，记一行 name=reuse_check；不同就把那一把键撤成红（S18）。交回相同否。"""
        differing = differences_from_value(decision.kind, stored, reading)
        append_event(self.markers, f"E7RESULT name=reuse_check kind={check_kind} unit={value(decision.unit)} key={decision.key} "
                                   f"identical={str(not differing).lower()} differing={value(','.join(differing[:12]))}")
        if differing:
            withdraw(self.repo, decision.key, decision.item, f"读数与库里同一把键的那一份不同（{check_kind}）：撤成红，下一趟现判")
        return not differing

    def record(self, unit: str, reading: dict[str, str], exit_code: int, insertable: tuple[bool, str]) -> str:
        """一个照跑完的单元：现判的照入库条件判绿或判红（long_run_item_verdicts 的 record），RC1–RC3 照文件头比；记一行 name=reuse_insert。
        交回结局：green、red、inserted、present-identical、present-differs、sample-identical、sample-differs、not-inserted、not-applicable。"""
        decision = self.decisions[unit]
        ok, why = insertable
        if decision.state == STATE_REUSED:
            append_event(self.markers, f"E7RESULT name=reuse_sentinel unit={value(unit)} key={decision.key} problem=reused-rejudged detail=取来的单元照现跑交了入库")
            return "not-applicable"
        if decision.state == STATE_FRESH_CHECK:
            stored = self.stored(unit)
            outcome = "sample-identical" if stored is not None and self.compare(decision, stored, reading, "sample") else "sample-differs"
        elif decision.state == STATE_FRESH_ALWAYS:
            outcome = self.record_second_run(decision, reading, exit_code, ok)
        else:
            unchecked = decision.forced and broken("forced-reading-unchecked")
            before = None if unchecked else self.stored(unit)
            items_library.record(self.state, decision.item, exit_code, packed_reading(reading), ok, printer=self.printer)
            outcome = "green" if ok else "red"
            if before is not None:
                self.compare(decision, before, reading, "forced" if decision.forced else "insert")
        append_event(self.markers, f"E7RESULT name=reuse_insert unit={value(unit)} key={decision.key} state={outcome} reason={value(why or decision.reason)}")
        return outcome

    def record_second_run(self, decision: Decision, reading: dict[str, str], exit_code: int, ok: bool) -> str:
        """RC2：second-run 那一项库里这把键有判绿的就比，没有就照判绿写进去；拒算的别几类、入库条件不过的、键算不出的不存。"""
        if decision.reason != SECOND_RUN or decision.key == "-" or not ok or broken("second-run-unchecked"):
            return "not-inserted"
        stored = stored_value(self.repo, decision.key)
        if stored is None:
            store_green(self.repo, decision.key, decision.item, exit_code, packed_reading(reading))
            return "inserted"
        return "present-identical" if self.compare(decision, stored, reading, SECOND_RUN) else "present-differs"

    def finish(self) -> int:
        """long_run_item_verdicts 的 finish（重算键、判复用哨兵）；退 1 记一行 name=reuse_sentinel problem=finish-red（同 S18）。删状态目录。交它的退出码。"""
        code = 0
        try:
            if self.planned:
                code = items_library.finish(self.state, printer=self.printer)
                if code != 0:
                    append_event(self.markers, f"E7RESULT name=reuse_sentinel unit=- key=- problem=finish-red detail=long_run_item_verdicts_finish_exit_{code}")
        finally:
            shutil.rmtree(self.state, ignore_errors=True)
        return code


def rows_sample(decisions: list[Decision], suffix: str) -> list[str]:
    """RC3：取来的行数 M ≥ 1 时挑 k = max(3, ⌈0.02 × M⌉) 行照跑（不够 k 行就全挑），挑法按 sha256(后缀 ‖ 键) 从小到大取前 k 个。交回挑中的单元名。"""
    reused = [decision for decision in decisions if decision.state == STATE_REUSED]
    if not reused:
        return []
    size = max(3, math.ceil(0.02 * len(reused)))
    ordered = sorted(reused, key=lambda decision: (sha256_bytes((suffix + decision.key).encode("utf-8")), decision.unit))
    return [decision.unit for decision in ordered[:size]]


def peer_row_problems(row_directory: str, decision: Decision, shard_product_text: str, frozen_snapshot_digest: str) -> list[str]:
    """别的台跑的行入库之前另核三样：行结果里 row_number 与 mutation_line_sha256 对得上这一行的请求（Decision.facts）；那一分片的
    row_time_limit 秒数等于请求里的 L；那一分片的 snapshot_digest 等于发给别的台的冻结树快照的摘要（合并规则 ② 比的那一份）。交回不过的那几样。"""
    fields = row_result_fields(row_directory)
    problems = [] if fields.get("row_number") == str(decision.facts.get("row")) else ["row_number"]
    expected = decision.facts.get("mutation_line_sha256")
    if expected is not None and fields.get("mutation_line_sha256") != expected:
        problems.append("mutation_line_sha256")
    limit = re.search(r"^name=row_time_limit seconds=(\d+)", shard_product_text, re.MULTILINE)
    if not limit or limit.group(1) != str(decision.facts.get("limit_seconds")):
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


def unit_by_name(name: str):
    """单元表里的读数单元或逐行驱动的一行（seg1-<步>-row-<号>、seg2-row-<号>，参数取钉住的那两份编排器）。"""
    for unit in list(SEGMENT_ONE_READING_UNITS) + step_4_row_units():
        if unit.name == name:
            return unit
    found = re.fullmatch(r"seg1-(s11|s12|sx|r11|r12)-row-([0-9]+)", name)
    if found:
        return segment_one_row_units((found.group(1),), {found.group(1): (int(found.group(2)),)})[0]
    found = re.fullmatch(r"seg2-row-([0-9]+)", name)
    if found:
        return segment_two_row_units((int(found.group(1)),), None)[0]
    raise ValueError(f"认不得的单元名：{name}")


# ── 命令行：单元脚本取用、RP2、自证的起法 ──
def command_fetch(arguments: argparse.Namespace) -> int:
    targets = {}
    for mapping in arguments.map:
        role, _, path = mapping.partition("=")
        if not path:
            print(f"--map 要写成 <角色>=<目标路径>：{mapping}", file=sys.stderr)
            return 2
        targets[role] = os.path.abspath(path)
    try:
        exit_code = fetch_from_state(arguments.library, arguments.entry, targets)
    except (OSError, ValueError, EOFError, tarfile.TarError) as error:
        print(f"✗ 取用不成：{error}", file=sys.stderr)
        print('     → 怎么办：目标已在就先挪走再取；状态目录里没有这一项、或那一份解不开的，照现判重跑这个单元（编排下一趟现查库）', file=sys.stderr)
        return 1
    print(exit_code)
    return 0


def command_items(arguments: argparse.Namespace) -> int:
    """文件头「用法」items：只在自证里认。"""
    problem = prune_server_client.selftest_service_problem()
    if problem or os.environ.get(items_library.FAKE_JUDGE_VARIABLE) != "1":
        print(f"✗ items 只在自证里认（临时 prune-server 底下、{items_library.FAKE_JUDGE_VARIABLE}=1）：{problem or '假判法没设'}", file=sys.stderr)
        print("     → 怎么办：它只给 long_run_item_verdicts.py reuse-cells 当起法（--fake-judge），平常不起它", file=sys.stderr)
        return 2
    facts = tree_facts(arguments.repo, arguments.tree, dict(os.environ))
    selected = selected_row_lines(arguments.repo)
    requests = [unit_request(facts, unit_by_name(name), selected) for name in arguments.units.split(",") if name]
    code, _results = items_library.judge_items(arguments.script, KIND, arguments.repo, requests_text(requests), lambda item: (0, b"", True))
    return code


def selfproof_run(repo: str, binary: str, threads: str, directory: str) -> tuple[int, str]:
    """RP2 的一次：装置 stream-constants --streams 1,2,3（只打 S1 常量），直接起、线程变量设成 `threads`（frozen_o3.thread_environment；内存额度由任务服务给）；产物是它的 E7RESULT 行。"""
    os.makedirs(directory)
    product = os.path.join(directory, "stream-constants.out")
    command = ["nice", "-n", "19", binary, "stream-constants", "--streams", "1,2,3"]
    completed = subprocess.run(command, cwd=repo, env=frozen_o3.thread_environment(threads), capture_output=True, text=True)
    with open(product, "x", encoding="utf-8") as handle:
        handle.writelines(line + "\n" for line in completed.stdout.splitlines() if line.startswith("E7RESULT "))
    return completed.returncode, product


def selfproof_request(binary: str, nonce: str) -> UnitRequest:
    """RP2 那一把键：入口写 -（判的是这一份装置二进制与 selfproof_run），装置的内容、参数与这一趟的随机串经 line: 进键（这一趟独占）。"""
    item = "reuse-selfproof"
    line = "\t".join([item, "-", f"judge:{SELFPROOF_JUDGE}", f"line:binary-sha256={sha256_file(binary)}", "line:arguments=stream-constants --streams 1,2,3",
                      f"line:selfproof-run={nonce}", "line:compare=exact"])
    return UnitRequest(item, "device", item, [line])


def command_selfproof(arguments: argparse.Namespace) -> int:
    """RP2（第 11.8 节；文件头「用法」selfproof）：① 照现判入库 ② 同一把键取来、与①逐字节比 ③ 另一个目录现跑、照读数相等比；跑完删掉库里这一把键的两条。"""
    repo = os.path.abspath(arguments.repo)
    check_service_reachable(repo)
    request = selfproof_request(arguments.binary, f"{time.strftime('%Y%m%d')}-{os.getpid()}-{secrets.token_hex(4)}")
    markers = os.path.join(arguments.scratch, "markers")
    key = "-"
    try:
        first_exit, first_product = selfproof_run(repo, arguments.binary, arguments.threads, os.path.join(arguments.scratch, "run-1"))
        first_run = ItemRun(repo, SCRIPT_KEY, [request], arguments.library, markers)
        first = first_run.plan()[request.unit]
        key = first.key
        inserted = first_run.record(request.unit, {"product": first_product}, first_exit, device_exit_insertable(first_exit, (0,))) \
            if first.state == STATE_FRESH else first.state
        first_run.finish()
        lines = [f"E7RESULT name=reuse_selfproof_config library=apparatus-items/{KIND} binary_sha256={sha256_file(arguments.binary)} key={key}",
                 f"E7RESULT name=reuse_selfproof step=1 state={first.state} exit={first_exit} insert={inserted}"]
        second_run = ItemRun(repo, SCRIPT_KEY, [request], arguments.library, markers)
        second = second_run.plan()[request.unit]
        fetched_identical = False
        if second.state == STATE_REUSED:
            second_product = os.path.join(arguments.scratch, "run-2", "stream-constants.out")
            second_run.fetch(request.unit, {"product": second_product})
            fetched_identical = open(second_product, "rb").read() == open(first_product, "rb").read()
        second_run.finish()
        lines.append(f"E7RESULT name=reuse_selfproof step=2 state={second.state} fetched_identical_to_step_1={str(fetched_identical).lower()}")
        third_exit, third_product = selfproof_run(repo, arguments.binary, arguments.threads, os.path.join(arguments.scratch, "run-3"))
        stored = stored_value(repo, key) if key != "-" else None
        identical = stored is not None and not differences_from_value("device", stored, {"product": third_product})
        lines.append(f"E7RESULT name=reuse_selfproof step=3 exit={third_exit} identical={str(identical).lower()}")
    finally:
        if key != "-":
            library_store(repo).delete_prefix(f"items/{key}.")   # 这一趟独占的那一把键的两条（.out、.verdict）
    holds = first_exit == 0 and inserted == "green" and fetched_identical and third_exit == 0 and identical
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
    parser = argparse.ArgumentParser(description="E171 修订 11：按项判定（请求、值、核对、账本）")
    commands = parser.add_subparsers(dest="command", required=True)
    fetch_parser = commands.add_parser("fetch")
    fetch_parser.add_argument("--library", required=True, help="编排给的这一趟状态目录")
    fetch_parser.add_argument("--entry", required=True, help="项名")
    fetch_parser.add_argument("--map", action="append", required=True)
    proof = commands.add_parser("selfproof")
    for name in ("--library", "--binary", "--scratch", "--output", "--threads"):
        proof.add_argument(name, required=True)
    proof.add_argument("--repo", default=os.getcwd())
    items = commands.add_parser("items")
    for name in ("--repo", "--tree", "--script", "--units"):
        items.add_argument(name, required=True)
    arguments = parser.parse_args(argument_list)
    return {"fetch": command_fetch, "selfproof": command_selfproof, "items": command_items}[arguments.command](arguments)


# ── 自证：假仓、假树、假装置与假驱动行、假判法；请求、键、库、账本都调上面的真函数（文件头「用法」--selftest）──
REAL_REPOSITORY = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
FAKE_TREE_RELATIVE = "scripts/experiment-apparatus/target/fake-tree"
ALL_SENTINELS = "1000"   # 键相同的全当哨兵、不抽强制现判（只看复用与现判的那几格）
REUSE_CELLS_SECONDS = 1500


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


def plant_key_skeleton(repo: str) -> None:
    """假仓里算键要的几样：研究一侧目录拷一份（判法闭包照真文件算，改得动），.claude、scripts 下别的目录、实验装置目录里的按项判定与冻结执行器
    指回真仓（符号链接）；再写 long_run_item_verdicts 的假装置入口。已在的不动。"""
    research = os.path.join(repo, RESEARCH_RELATIVE)
    if not os.path.isdir(research):
        shutil.copytree(HERE, research, ignore=shutil.ignore_patterns("__pycache__", "*.pyc"))
    links = [".claude"] + [os.path.join("scripts", name) for name in sorted(os.listdir(os.path.join(REAL_REPOSITORY, "scripts"))) if name != "experiment-apparatus"]
    links += [os.path.join("scripts", "experiment-apparatus", "long_run_item_verdicts.py"), EXECUTOR_RELATIVE]
    for relative in links:
        path = os.path.join(repo, relative)
        if not os.path.lexists(path):
            os.makedirs(os.path.dirname(path), exist_ok=True)
            os.symlink(os.path.join(REAL_REPOSITORY, relative), path)
    items_library.plant_fake_module(repo)


def furnish_tree(tree: str, mutated_text: str = "fn f() { yx }\n") -> None:
    """假冻结树：假 core、两个 bin 的入口文件（装置入口找得到，自证里经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成假入口）、变异表。"""
    for relative, text in (("Cargo.toml", "[workspace]\n"), ("Cargo.lock", "# lock\n"), (".cargo/config.toml", "[build]\n"),
                           ("crates/core/Cargo.toml", "[package]\nname='core'\n"), ("crates/core/src/mutated.rs", mutated_text),
                           ("crates/core/src/lib.rs", 'fn read() { std::env::var("SINGLEFS_FAKE_SWITCH"); std::env::var("SINGLEFS_THREAD_CAP"); }\n'),
                           ("crates/core/src/other.rs", "fn g() { alpha; gamma }\n"), ("crates/mutations.tsv", FAKE_MUTATIONS),
                           (f"crates/core/src/bin/{DEVICE_BINARY}.rs", "fn main() {}\n"), (f"crates/core/src/bin/{E170_BINARY}.rs", "fn main() {}\n")):
        write_text(os.path.join(tree, relative), text)


@dataclasses.dataclass
class FakeWorld:
    root: str
    repo: str
    tree: str
    entries: str   # LONG_RUN_ITEMS_SELFTEST_ENTRIES 那一份

    def facts(self, environment: dict[str, str] | None = None) -> TreeFacts:
        return tree_facts(self.repo, self.tree, environment or {})


def fake_world(root: str, mutated_text: str = "fn f() { yx }\n") -> FakeWorld:
    repo = os.path.join(root, "repo")
    tree = os.path.join(repo, FAKE_TREE_RELATIVE)
    plant_key_skeleton(repo)
    furnish_tree(tree, mutated_text)
    write_text(os.path.join(repo, RESEARCH_RELATIVE, SELECTED_ROW_TABLES[0]), FAKE_SELECTED_117)
    write_text(os.path.join(repo, RESEARCH_RELATIVE, SELECTED_ROW_TABLES[1]), FAKE_SELECTED_164)
    write_text(os.path.join(repo, FREEZE_REFERENCE_RELATIVE), "E7RESULT name=freeze fake=1\n")
    return FakeWorld(root, repo, tree, write_text(os.path.join(root, "entries.tsv"), ""))


def furnish_rows_world(repo: str, tree: str, numbers: tuple[int, ...]) -> None:
    """两份逐行编排自证用：往编排器自己的假仓与假树里补上算键要的几样（plant_key_skeleton、假树、行表）。行表里点名的行号都指 m-row-two
    （other.rs 里 alpha → beta），第 1 行指 m-row-one（空变异取它）。先补再算 H3、H8。"""
    plant_key_skeleton(repo)
    furnish_tree(tree)
    research = os.path.join(repo, RESEARCH_RELATIVE)
    table = "1\t1\tcrates/core/src/mutated.rs\t0\tm-row-one\taaaaaaaaaaaa\n" + "".join(
        f"{number}\t2\tcrates/core/src/other.rs\t0\tm-row-two\t{number:012d}\n" for number in sorted(set(numbers) - {0, 1}))
    write_text(os.path.join(research, SELECTED_ROW_TABLES[0]), table)
    write_text(os.path.join(research, SELECTED_ROW_TABLES[1]), "")
    # 假仓里的参照可能是指向真仓 research/results/ 下产物的链接（编排器自证的假仓就是）：链在就不写，悬空时 open(..., "w") 会顺着链把假内容写进真仓
    if not os.path.lexists(os.path.join(repo, FREEZE_REFERENCE_RELATIVE)):
        write_text(os.path.join(repo, FREEZE_REFERENCE_RELATIVE), "E7RESULT name=freeze fake=1\n")


def selftest_entries(path: str, units: list[str]) -> str:
    """自证：这几项的装置入口经 LONG_RUN_ITEMS_SELFTEST_ENTRIES 换成 long_run_item_verdicts 的假入口 measure_alpha（line:、judge:、@input、@files 照留）。
    path 里已有的项不再写；交回 path。"""
    known = {line.split("\t", 1)[0] for line in open(path, encoding="utf-8")} if os.path.exists(path) else set()
    with open(path, "a", encoding="utf-8") as handle:
        handle.writelines(f"{item_name(unit)}\t{items_library.FAKE_MODULE}::measure_alpha\n" for unit in units if item_name(unit) not in known)
    return path


@contextlib.contextmanager
def items_environment(world: FakeWorld, sentinels: str = ALL_SENTINELS, seed: str | None = None):
    """这一格读写临时服务里它自己的那一份命名空间、换假入口、定哨兵条数与种子；出 with 还原。"""
    checks = items_library.lib_reuse_checks
    settings = {items_library.LIBRARY_DIRECTORY_VARIABLE: os.path.join(world.root, "library"), items_library.SUBSTITUTION_VARIABLE: world.entries,
                checks.SENTINEL_COUNT_VARIABLE: sentinels, checks.SENTINEL_SEED_VARIABLE: seed}
    saved = {name: os.environ.get(name) for name in settings}
    for name, setting in settings.items():
        if setting is None:
            os.environ.pop(name, None)
        else:
            os.environ[name] = setting
    try:
        yield
    finally:
        for name, setting in saved.items():
            if setting is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = setting


def quiet(_line: str) -> None:
    """自证里不打 long_run_item_verdicts 的计划与收尾那几行。"""


def keys_of(world: FakeWorld, requests: list[UnitRequest]) -> dict[str, str | None]:
    selftest_entries(world.entries, [request.unit for request in requests])
    with items_environment(world):
        return {unit: key for unit, (key, _why) in keyed_items(world.repo, requests).items()}


def keys_differ(world: FakeWorld, first: UnitRequest, second: UnitRequest) -> tuple[bool, str]:
    keys = keys_of(world, [first, second])
    a, b = keys[first.unit], keys[second.unit]
    return a is not None and b is not None and a != b, f"{first.unit} 键 {str(a)[:12]}，{second.unit} 键 {str(b)[:12]}（两把都算得出且不同才过）"


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


def fake_judge(reading: dict[str, str]) -> str:
    """假判法：读产物或 row-result.tsv（取来的与现跑的都读）。"""
    if "row" in reading:
        return " ".join(row_result_without_volatile_keys(os.path.join(reading["row"], "row-result.tsv")))
    return open(reading["product"], encoding="utf-8").read().strip()


def row_two(world: FakeWorld, name: str = "seg2-row-2", limit: int | None = 1800, selected: dict[int, str] | None = None,
            facts: TreeFacts | None = None) -> UnitRequest:
    row = RowUnit(name, 2, False, ("--streams", "1,2", "--no-executor"), False, False)
    return row_request(facts or world.facts(), row, selected or selected_row_lines(world.repo), limit, [])


def device_unit(name: str, arguments: str, kind: str = "device", fresh_always: bool = False) -> ReadingUnit:
    return ReadingUnit(name, name, name, kind, arguments, fresh_always=fresh_always)


def cell_request_kind(scratch: str) -> tuple[bool, str]:
    """参数相同、种类不同（装置与 E170 导出装置）的两个读数单元：键不同。"""
    world = fake_world(os.path.join(scratch, "world"))
    facts = world.facts()
    return keys_differ(world, reading_unit_request(facts, device_unit("u-device", "baseline --streams 1"), []),
                       reading_unit_request(facts, device_unit("u-e170", "baseline --streams 1", "e170"), []))


def cell_request_arguments(scratch: str) -> tuple[bool, str]:
    world = fake_world(os.path.join(scratch, "world"))
    facts = world.facts()
    return keys_differ(world, reading_unit_request(facts, device_unit("u-first", "baseline --streams 2 --positive-control O1-对拍"), []),
                       reading_unit_request(facts, device_unit("u-second", "baseline --streams 2 --positive-control O1-对拍-卸载"), []))


def cell_request_limit(scratch: str) -> tuple[bool, str]:
    world = fake_world(os.path.join(scratch, "world"))
    return keys_differ(world, row_two(world, "seg2-row-2", 1800), row_two(world, "seg2-rerun-row-2", 2400))


def cell_request_upstream(scratch: str) -> tuple[bool, str]:
    """执行器的上游导出目录里一个文件多一个字：键不同（上游按读数的内容进键）。"""
    world = fake_world(os.path.join(scratch, "world"))
    facts = world.facts()
    export = fake_device_reading(os.path.join(scratch, "upstream"), "step-1")["directory"]
    unit = [unit for unit in SEGMENT_ONE_READING_UNITS if unit.name == "step-5"][0]
    first = reading_unit_request(facts, unit, [("device-baseline", export)])
    with open(os.path.join(export, "new_pool_file_creation", "states.tsv"), "a", encoding="utf-8") as handle:
        handle.write("x")
    second = reading_unit_request(facts, dataclasses.replace(unit, name="step-5-again"), [("device-baseline", export)])
    return keys_differ(world, first, second)


def cell_request_mutation(scratch: str) -> tuple[bool, str]:
    """只改行表里第 2 行那一行（行文 sha256 前缀那一列）：键不同；只改变异表里那一行的替换文（改坏之后的源码不进闭包，由变异行原文进键）：键不同。"""
    world = fake_world(os.path.join(scratch, "world"))
    selected = selected_row_lines(world.repo)
    changed = {**selected, 2: selected[2].replace("bbbbbbbbbbbb", "bbbbbbbbbbbc")}
    table_line, table_detail = keys_differ(world, row_two(world, "seg2-row-2", selected=selected), row_two(world, "seg2-again-row-2", selected=changed))
    before = row_two(world, "seg2-before-row-2")
    mutations = os.path.join(world.tree, "crates", "mutations.tsv")
    write_text(mutations, open(mutations, encoding="utf-8").read().replace("\talpha\tbeta\t", "\talpha\tbetb\t"))
    mutation_line, mutation_detail = keys_differ(world, before, row_two(world, "seg2-after-row-2"))
    return table_line and mutation_line, f"行表那一行：{table_detail}；变异表那一行：{mutation_detail}"


def cell_request_judge_closure(scratch: str) -> tuple[bool, str]:
    """判这一行的函数（假仓里拷的 row_driver.py 的 run_row_on_the_lane）改了一处代码：键不同。"""
    world = fake_world(os.path.join(scratch, "world"))
    first = keys_of(world, [row_two(world)])["seg2-row-2"]
    driver = os.path.join(world.repo, RESEARCH_RELATIVE, "row_driver.py")
    anchor = "timed_out_before = bool(" + "arguments.timed_out_before)"
    text = open(driver, encoding="utf-8").read()
    if text.count(anchor) != 1:
        return False, f"假仓里的 row_driver.py 里「{anchor}」不是恰好一处（{text.count(anchor)} 处）"
    write_text(driver, text.replace(anchor, anchor + " and True"))
    second = keys_of(world, [row_two(world)])["seg2-row-2"]
    return first is not None and second is not None and first != second, f"动它之前的键 {str(first)[:12]}，动了之后的键 {str(second)[:12]}"


def cell_request_freeze_reference(scratch: str) -> tuple[bool, str]:
    """冻结行那份产物多一行：跑执行器的两样，键都变；不动它再算一遍，键都不变。
    执行器那一个读数单元（step-5，判法 frozen_o3.py::run_executor）的判法闭包不读这份产物，只经 @input 进键：弄坏开关 freeze-reference-unkeyed 红在它；
    跑执行器的一行（seg1-s12-row-1）的判法闭包另经起执行器与 O3 对照那一路读到这份产物，它的键不只靠 @input。"""
    world = fake_world(os.path.join(scratch, "world"))
    executor = [unit for unit in SEGMENT_ONE_READING_UNITS if unit.name == "step-5"][0]
    export = fake_device_reading(os.path.join(scratch, "upstream"), "step-1")["directory"]
    row = RowUnit("seg1-s12-row-1", 1, False, ("--keep-dumps", "--streams", "1,2,3"), True, False, owner="s12")

    def executor_and_row_keys() -> tuple[str | None, str | None]:
        found = keys_of(world, [reading_unit_request(world.facts(), executor, [("device-baseline", export)]),
                                row_request(world.facts(), row, selected_row_lines(world.repo), 1800, [])])
        return found[executor.name], found[row.name]

    before, again = executor_and_row_keys(), executor_and_row_keys()
    with open(os.path.join(world.repo, FREEZE_REFERENCE_RELATIVE), "a", encoding="utf-8") as handle:
        handle.write("E7RESULT name=freeze fake=2\n")
    after = executor_and_row_keys()
    good = None not in before + after and before == again and before[0] != after[0] and before[1] != after[1]
    return good, (f"执行器 step-5：多一行之前 {str(before[0])[:12]}、不动再算 {str(again[0])[:12]}、多一行之后 {str(after[0])[:12]}；"
                  f"跑执行器的一行 seg1-s12-row-1：之前 {str(before[1])[:12]}、不动再算 {str(again[1])[:12]}、之后 {str(after[1])[:12]}")


def cell_request_environment_set(scratch: str) -> tuple[bool, str]:
    """树里源码读的变量此刻设着：装置单元与驱动的一行都拒算 env-set；只设了线程数变量的不拒。"""
    world = fake_world(os.path.join(scratch, "world"))
    unit = device_unit("u", "baseline --streams 1")
    reasons = {label: (reading_unit_request(world.facts(environment), unit, []).reason, row_two(world, facts=world.facts(environment)).reason)
               for label, environment in (("unset", {}), ("set", {"SINGLEFS_FAKE_SWITCH": "1"}), ("threads-only", {"SINGLEFS_THREAD_CAP": "4"}))}
    return reasons == {"unset": ("", ""), "set": ("env-set", "env-set"), "threads-only": ("", "")}, f"各样的拒算缘由：{reasons}"


def cell_request_module_set_refused(scratch: str) -> tuple[bool, str]:
    """改坏那一处碰到模块集合的第 3 行（m-module：替换文带 mod）拒算 touches-module-set；第 2 行照常进计划。"""
    world = fake_world(os.path.join(scratch, "world"))
    selected = selected_row_lines(world.repo)
    reasons = {number: row_request(world.facts(), RowUnit(f"seg2-row-{number}", number, False, ("--streams", "1,2", "--no-executor"), False, False),
                                   selected, 1800, []).reason for number in (2, 3)}
    return reasons == {2: "", 3: "touches-module-set"}, f"各行的拒算缘由：{reasons}"


def location_keys(root: str) -> dict[str, str | None]:
    """root 底下一份假仓与假树上各类单元的键：装置、带上游导出目录的 E170 导出装置与执行器、变异驱动、第二段一行、第一段一行（空变异）。"""
    world = fake_world(root)
    facts = world.facts()
    upstream = [("e170-export", write_text(os.path.join(root, "upstream-export", "new_pool_file_creation", "states.tsv"), "上游导出\n").rsplit("/", 2)[0])]
    units = {unit.name: unit for unit in SEGMENT_ONE_READING_UNITS}
    requests = [reading_unit_request(facts, units["step-1"], []), reading_unit_request(facts, units["step-2b"], upstream),
                reading_unit_request(facts, units["step-5"], upstream), reading_unit_request(facts, units["research-mutations"], []), row_two(world, facts=facts),
                row_request(facts, RowUnit("seg1-s12-row-0", 0, True, ("--empty-mutation", "--keep-dumps", "--streams", "1,2,3"), True, True, owner="s12"),
                            selected_row_lines(world.repo), 1800, upstream)]
    return keys_of(world, requests)


def cell_key_independent_of_location(scratch: str) -> tuple[bool, str]:
    """同一份假仓与假树（连上游导出）放在两处：一处在主树的位置，一处在另一处的副本目录底下、多套几层；各类单元的键逐个相同、都算得出。"""
    here, there = location_keys(os.path.join(scratch, "main-tree")), location_keys(os.path.join(scratch, "copies", "nested", "copy-of-the-tree"))
    differing = sorted(unit for unit in here if here[unit] is None or here[unit] != there.get(unit))
    return not differing, (f"{len(here)} 个单元的键都相同" if not differing else f"键随所在目录变了或算不出：{', '.join(differing)}")


def cell_packing_deterministic(scratch: str) -> tuple[bool, str]:
    """同一份读数改了修改时刻再打一次，两份值逐字节相同；解回来文件、内容、空目录都与原来一样。"""
    reading = fake_device_reading(os.path.join(scratch, "reading"), "unit")
    os.makedirs(os.path.join(reading["directory"], "empty-stream"))
    first = packed_reading(reading)
    for path in [reading["product"], reading["timing"]] + [os.path.join(reading["directory"], relative) for relative in files_under(reading["directory"])]:
        os.utime(path, (1_000_000_000, 1_000_000_000))
    second = packed_reading(reading)
    targets = {role: os.path.join(scratch, "unpacked", role) for role in reading}
    unpack_reading(second, targets)
    same = same_bytes(reading, targets) and os.path.isdir(os.path.join(targets["directory"], "empty-stream"))
    return first == second and same, f"两份值逐字节相同 {first == second}；解回来与原来相同 {same}"


def cell_reading_equality(scratch: str) -> tuple[bool, str]:
    """读数相等：两趟假装置（计时、impl-verdicts.tsv 耗时列不同）相等，导出里一个文件多一字节不等；两趟假驱动行（易变键、o3 的耗时不同）相等，device.out 掺一个字不等。"""
    first, second = fake_device_reading(os.path.join(scratch, "a"), "unit"), fake_device_reading(os.path.join(scratch, "b"), "unit")
    equal_devices = reading_differences("device", first, second)
    with open(os.path.join(second["directory"], "new_pool_file_creation", "states.tsv"), "a", encoding="utf-8") as handle:
        handle.write("x")
    unequal_devices = reading_differences("device", first, second)
    row_a, row_b = fake_row_reading(os.path.join(scratch, "c"), "row"), fake_row_reading(os.path.join(scratch, "d"), "row")
    equal_rows = reading_differences("driver-row", row_a, row_b)
    with open(os.path.join(row_b["row"], "device.out"), "a", encoding="utf-8") as handle:
        handle.write("E7RESULT name=salt\n")
    unequal_rows = reading_differences("driver-row", row_a, row_b)
    good = (not equal_devices and unequal_devices == ["directory/new_pool_file_creation/states.tsv"] and not equal_rows and unequal_rows == ["row/device.out"])
    return good, f"装置 {equal_devices} → {unequal_devices}；驱动行 {equal_rows} → {unequal_rows}"


def cell_insert_conditions(scratch: str) -> tuple[bool, str]:
    """入库条件：装置退 0、预期退 3 入；超时 253、包装自己的 250–252、254、不在预期里的不入；驱动行做完、没补位、核过重编的入，超时、补位、编不过、没核重编的不入。"""
    devices = {code: device_exit_insertable(code, expected)[0] for code, expected in ((0, (0,)), (3, (3,)), (TIMEOUT_EXIT_CODE, (0,)), (251, (0,)), (3, (0,)))}
    rows = {}
    for label, changes in (("ok", {}), ("timeout", {"device_exit": str(TIMEOUT_EXIT_CODE)}), ("substituted", {"substituted_from": "m-row-one"}),
                           ("compile-failed", {"status": "compile_failed", "device_exit": "-"}), ("rebuild-not-checked", {REBUILD_CHECK_FIELD: None})):
        row = fake_row_reading(os.path.join(scratch, label), label)["row"]
        fields = row_result_fields(row)
        for name, setting in changes.items():
            if setting is None:
                fields.pop(name, None)
            else:
                fields[name] = setting
        write_text(os.path.join(row, "row-result.tsv"), "".join(f"{name}\t{text}\n" for name, text in fields.items()))
        rows[label] = row_insertable(row)[0]
    good = (devices == {0: True, 3: False, TIMEOUT_EXIT_CODE: False, 251: False} and device_exit_insertable(3, (3,))[0]
            and rows == {"ok": True, "timeout": False, "substituted": False, "compile-failed": False, "rebuild-not-checked": False})
    return good, f"装置 {devices}；驱动行 {rows}"


def cell_fetch_refuses_existing_targets(scratch: str) -> tuple[bool, str]:
    """取用：目标已在（哪怕只有一个）就拒、一个目标都不写；值里没有的角色拒；都不在的照解。"""
    value = packed_reading(fake_device_reading(os.path.join(scratch, "reading"), "unit"))
    targets = {"product": os.path.join(scratch, "out", "product"), "directory": os.path.join(scratch, "out", "directory")}
    unpack_reading(value, targets)
    outcomes = {}
    for label, wanted in (("existing", {"product": os.path.join(scratch, "again", "product"), "directory": targets["directory"]}),
                          ("missing-role", {"row": os.path.join(scratch, "again", "row")})):
        try:
            unpack_reading(value, wanted)
            outcomes[label] = "解了"
        except (FileExistsError, ValueError) as error:
            outcomes[label] = type(error).__name__
    untouched = not os.path.lexists(os.path.join(scratch, "again", "product"))
    return outcomes == {"existing": "FileExistsError", "missing-role": "ValueError"} and untouched, f"{outcomes}；被拒的那一次一个目标都没写 {untouched}"


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
# 第一遍里这几个单元的退出码（别的都是 0）：panic 对照预期退 3；step-6 撞包装（251，判红）；第二段第 4 行超时（253，判红）
RP1_EXITS = {"step-3-panic": 3, "step-6": 251, "seg2-row-4": TIMEOUT_EXIT_CODE}
RP1_FRESH_IN_THE_SECOND_PASS = ("step-9", "step-10-run-2", "step-6", "seg1-s12-row-0", "seg1-s12-row-3", "seg2-row-2", "seg2-row-3", "seg2-row-4",
                                "seg2-row-118")


def rp1_pass(world: FakeWorld, markers: str, root: str) -> tuple[dict, dict, dict]:
    """假编排的一遍：第 0–10 步的全部读数单元（单元表原样）与两段各 6 行，上游都有了的那一批一份计划：算键、查库、取来或假跑、入库（照单元表的次序，
    RC2 的第二趟在它第一趟之后），判法每个单元都现跑。"""
    facts, selected = world.facts(), selected_row_lines(world.repo)
    readings, judges, decisions = {}, {}, {}
    pending = list(SEGMENT_ONE_READING_UNITS) + step_4_row_units() + list(RP1_ROWS)
    selftest_entries(world.entries, [unit.name for unit in pending])
    while pending:
        ready = [unit for unit in pending if all(name in readings for _, name in unit.upstream)]
        requests = [unit_request(facts, unit, selected, 1800 if isinstance(unit, RowUnit) else None,
                                 tuple((role, readings[name].get("directory") or readings[name]["row"]) for role, name in unit.upstream)) for unit in ready]
        run = ItemRun(world.repo, SCRIPT_KEY, requests, os.path.join(root, "states"), markers, quiet)
        planned = run.plan()
        for unit in ready:
            decision = planned[unit.name]
            directory = os.path.join(root, unit.name.replace("/", "-"))
            exit_code = RP1_EXITS.get(unit.name, 0)
            if decision.state == STATE_REUSED:
                _exit, reading = run.fetch_into(unit.name, directory)
            elif isinstance(unit, RowUnit):
                reading = fake_row_reading(directory, decision.key[:16], exit_code, unit.executor)
                run.record(unit.name, reading, exit_code, row_insertable(reading["row"]))
            else:
                reading = fake_device_reading(directory, decision.key[:16], exit_code)
                run.record(unit.name, reading, exit_code, device_exit_insertable(exit_code, unit.expected_exits))
            readings[unit.name], judges[unit.name], decisions[unit.name] = reading, fake_judge(reading), decision
        run.finish()
        pending = [unit for unit in pending if unit.name not in readings]
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
    """库空跑一遍（入库）、再跑一遍：第二遍除拒算（second-run、补位那几类）与第一遍判红的几个外全部取来；取来的读数与第一遍逐字节相同，照跑的照读数相等
    的比法相同（RC2 比上）；每个单元的判法两遍逐字相同；账本 N + M 等于单元数、复用数对、没有 S18。"""
    world = fake_world(os.path.join(scratch, "world"))
    with items_environment(world):
        first_readings, first_judges, _ = rp1_pass(world, os.path.join(scratch, "markers-1"), os.path.join(scratch, "pass-1"))
        second_readings, second_judges, decisions = rp1_pass(world, os.path.join(scratch, "markers-2"), os.path.join(scratch, "pass-2"))
    names = list(decisions)
    fresh = sorted(name for name, decision in decisions.items() if decision.state != STATE_REUSED)
    expected_fresh = sorted(RP1_FRESH_IN_THE_SECOND_PASS)
    differing = [name for name in names if not (same_bytes(first_readings[name], second_readings[name]) if decisions[name].state == STATE_REUSED
                                                 else not reading_differences(decisions[name].kind, first_readings[name], second_readings[name]))]
    judge_differences = [name for name in names if first_judges[name] != second_judges[name]]
    text, covers = ledger_text(os.path.join(scratch, "markers-2"), names)
    verdict = text.strip().splitlines()[-1]
    problems = s18_problems(os.path.join(scratch, "markers-1")) + s18_problems(os.path.join(scratch, "markers-2"))
    good = (fresh == expected_fresh and not differing and not judge_differences and covers and f" reused={len(names) - len(expected_fresh)} " in verdict
            and " check_mismatches=0 " in verdict and not problems)
    return good, (f"{len(names)} 个单元；第二遍照跑的 {fresh if fresh != expected_fresh else len(fresh)}（要 {len(expected_fresh)} 个）；读数不同的 {differing or '无'}；"
                  f"判法不同的 {judge_differences or '无'}；S18 {problems[:2] or '无'}；账本 {verdict[:140]}")


def forcing_seed(items: list[str], wanted: str, count: int) -> str:
    """头一个让复用哨兵（lib_reuse_checks.ReuseSentinel，条数 count）把 wanted 抽成强制现判的种子（照 long_run_item_verdicts 的 plan 那样抽）。"""
    for seed in range(100000):
        sentinel = items_library.lib_reuse_checks.ReuseSentinel(SCRIPT_KEY, KIND, count=count, seed=seed)
        sentinel.plan({item: "-" for item in items}, printer=quiet)
        if sentinel.forced_item == wanted:
            return str(seed)
    raise RuntimeError(f"十万个种子里没有一个把 {wanted} 抽成强制现判")


def cell_forced_reading_check(scratch: str) -> tuple[bool, str]:
    """RC1：四个装置单元判绿入库；第二遍哨兵 1 条、种子让 step-0-a 成强制现判，它照跑出来的导出多一字节（产物、退出码不变：long_run_item_verdicts 的
    timing 比法比不出）：记一行 kind=forced identical=false（S18）、那一把键撤成红；别的三项照旧复用。"""
    world = fake_world(os.path.join(scratch, "world"))
    units = [device_unit(f"step-0-{label}", f"baseline --streams {streams}") for label, streams in (("a", "1"), ("b", "2"), ("c", "1,2"), ("d", "1,2,3"))]
    requests = [reading_unit_request(world.facts(), unit, []) for unit in units]
    selftest_entries(world.entries, [unit.name for unit in units])
    markers = os.path.join(scratch, "markers")
    with items_environment(world):
        first = ItemRun(world.repo, SCRIPT_KEY, requests, os.path.join(scratch, "states"), markers, quiet)
        for unit, decision in first.plan().items():
            first.record(unit, fake_device_reading(os.path.join(scratch, "first", unit), decision.key[:16]), 0, (True, ""))
        first.finish()
    with items_environment(world, sentinels="1", seed=forcing_seed(sorted(request.item for request in requests), "step-0-a", 1)):
        second = ItemRun(world.repo, SCRIPT_KEY, requests, os.path.join(scratch, "states"), markers, quiet)
        decisions = second.plan()
        forced = decisions["step-0-a"]
        reading = fake_device_reading(os.path.join(scratch, "second", "step-0-a"), forced.key[:16])
        with open(os.path.join(reading["directory"], "new_pool_file_creation", "states.tsv"), "a", encoding="utf-8") as handle:
            handle.write("x")
        if forced.state == STATE_FRESH:
            second.record("step-0-a", reading, 0, (True, ""))
        finish = second.finish()
        withdrawn = stored_value(world.repo, forced.key) is None
    problems = [line for line in s18_problems(markers) if " kind=forced " in f" {line} "]
    others = sorted(decision.state for unit, decision in decisions.items() if unit != "step-0-a")
    good = forced.forced and bool(problems) and withdrawn and others == [STATE_REUSED] * 3 and finish == 0
    return good, f"强制现判 {forced.unit}（forced={forced.forced}，{forced.state}）；S18 {problems[:1] or '无'}；撤成红 {withdrawn}；别的三项 {others}；finish 退 {finish}"


def cell_second_run_check(scratch: str) -> tuple[bool, str]:
    """RC2：step-7 判绿入库，step-9（同一把键、second-run）照跑的导出多一字节：kind=second-run identical=false（S18）、那一把键撤成红；
    step-10-run-1 与 run-2 读数相同：present-identical、不记 S18。"""
    world = fake_world(os.path.join(scratch, "world"))
    units = {unit.name: unit for unit in SEGMENT_ONE_READING_UNITS}
    names = ("step-7", "step-9", "step-10-run-1", "step-10-run-2")
    requests = [reading_unit_request(world.facts(), units[name], []) for name in names]
    selftest_entries(world.entries, list(names))
    markers = os.path.join(scratch, "markers")
    outcomes = {}
    with items_environment(world):
        run = ItemRun(world.repo, SCRIPT_KEY, requests, os.path.join(scratch, "states"), markers, quiet)
        decisions = run.plan()
        for name in names:
            reading = fake_device_reading(os.path.join(scratch, name), decisions[name].key[:16])
            if name == "step-9":
                with open(os.path.join(reading["directory"], "new_pool_file_creation", "states.tsv"), "a", encoding="utf-8") as handle:
                    handle.write("x")
            outcomes[name] = run.record(name, reading, 0, (True, ""))
        run.finish()
        withdrawn = stored_value(world.repo, decisions["step-7"].key) is None
    problems = s18_problems(markers)
    good = (decisions["step-9"].key == decisions["step-7"].key != "-" and outcomes["step-9"] == "present-differs" and outcomes["step-10-run-2"] == "present-identical"
            and len(problems) == 1 and " kind=second-run " in f" {problems[0]} " and withdrawn)
    return good, f"结局 {outcomes}；S18 {problems[:2] or '无'}；step-7 那一把键撤成红 {withdrawn}"


def run_reuse_cells(scratch: str, script: str, units: tuple[str, str]) -> tuple[bool, str]:
    """复用三问与复用哨兵的六格（long_run_item_verdicts.py reuse-cells，文件头「复用格」）：样板树是一份假仓，起法是本文件的 items（假判法），
    项甲、项乙是点名的两个单元；那一份的输出原样转出（规范输出 REUSE_SELFTEST script=<script> 那一行，门禁 code-tooling 的 reuse-selftests 格认标准输出里它）。"""
    world = fake_world(os.path.join(scratch, "template"))
    command = [sys.executable, "-B", "{tree}/" + SCRIPT_KEY, "items", "--repo", "{tree}", "--tree", "{tree}/" + FAKE_TREE_RELATIVE, "--script", script,
               "--units", ",".join(units)]
    completed = subprocess.run([sys.executable, "-B", os.path.join(os.path.dirname(HERE), "long_run_item_verdicts.py"), "reuse-cells", "--script", script,
                                "--template", world.repo, "--items", ",".join(item_name(unit) for unit in units), "--work", os.path.join(scratch, "cells"),
                                "--fake-judge", "--", *command], capture_output=True, text=True, timeout=REUSE_CELLS_SECONDS)
    print(completed.stdout, end="", flush=True)
    last = (completed.stdout.strip().splitlines() or ["没有输出"])[-1]
    return completed.returncode == 0 and last.endswith("6 格，红 0 格"), f"reuse-cells 退 {completed.returncode}：{last}；标准错误末尾 {completed.stderr.strip()[-200:]}"


def cell_reuse_three_questions(scratch: str) -> tuple[bool, str]:
    """复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）与复用哨兵的六格：项甲、项乙是 step-1 与 step-7 两个装置单元。"""
    return run_reuse_cells(scratch, SCRIPT_KEY, ("step-1", "step-7"))


LAYER_FILES = ("segment_one_units_revision_11.py", "rows_orchestrator_revision_11.py", "rows_orchestrator_segment_two_revision_11.py")


def cell_rp1_layers_end_to_end(scratch: str) -> tuple[bool, str]:
    """三层编排各自的端到端格（reuse-e2e：假单元脚本或假驱动，库空、库里有各跑一遍，取来的单元判法照跑）。"""
    outcomes = []
    for name in LAYER_FILES:
        completed = subprocess.run([sys.executable, "-B", os.path.join(HERE, name), "--selftest", "--cells", "reuse-e2e"], capture_output=True, text=True,
                                   timeout=900, env=dict(os.environ, TMPDIR=scratch))
        last = (completed.stdout.strip().splitlines() or ["没有输出"])[-1]
        outcomes.append((name, completed.returncode == 0 and last.endswith("红 0"), last))
        print(f"  格 rp1-layers-e2e：{name} 那一层 reuse-e2e 退 {completed.returncode}，{last}", flush=True)   # 一层判完就打（同 revision_8_selftest_harness.cells_reporting_progress 那一行）
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
    wanted = ((ORCHESTRATOR_SHA256, moved_sha256.get("rows_orchestrator.py")),
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


def selftest_cell_functions():
    """自证的各格，格名到函数。表写在函数体里：写成模块级赋值，会在 import 这个模块时把全部自证格连同它们起的子进程拖进 import 一方的判法单元闭包。"""
    return {
        "reuse-three-questions": cell_reuse_three_questions, "request-kind": cell_request_kind, "request-arguments": cell_request_arguments,
        "request-limit": cell_request_limit, "request-upstream": cell_request_upstream, "request-mutation": cell_request_mutation,
        "request-judge-closure": cell_request_judge_closure, "request-freeze-reference": cell_request_freeze_reference,
        "request-environment-set": cell_request_environment_set, "request-module-set-refused": cell_request_module_set_refused,
        "key-independent-of-location": cell_key_independent_of_location, "packing-deterministic": cell_packing_deterministic,
        "reading-equality": cell_reading_equality, "insert-conditions": cell_insert_conditions, "fetch-refuses-existing-targets": cell_fetch_refuses_existing_targets,
        "rp1-two-passes": cell_rp1_two_passes, "forced-reading-check": cell_forced_reading_check, "second-run-check": cell_second_run_check,
        "rp1-layers-e2e": cell_rp1_layers_end_to_end,
        "pin-moves": cell_pin_moves, "pin-moves-real-files": cell_pin_moves_real_files, "frozen-o3-moves": cell_frozen_o3_moves,
        "pin-moves-partial": cell_pin_moves_partial, "frozen-o3-reverse-moves": cell_frozen_o3_reverse_moves,
    }


SELFTEST_SWITCHES = {"kind-unkeyed": "request-kind", "arguments-unkeyed": "request-arguments", "limit-unkeyed": "request-limit",
                     "upstream-unkeyed": "request-upstream", "mutation-unkeyed": "request-mutation", "judges-dropped": "request-judge-closure",
                     "freeze-reference-unkeyed": "request-freeze-reference", "environment-set-ignored": "request-environment-set",
                     "module-set-keyed": "request-module-set-refused", "location-in-key": "key-independent-of-location",
                     "packing-keeps-times": "packing-deterministic", "second-run-planned": "rp1-two-passes", "forced-reading-unchecked": "forced-reading-check",
                     "second-run-unchecked": "second-run-check", "pin-moves-ignored": "pin-moves", "frozen-o3-moves-ignored": "frozen-o3-moves",
                     "pin-partial-accepted": "pin-moves-partial", "frozen-o3-partial-accepted": "frozen-o3-reverse-moves",
                     "frozen-o3-whole-chain-only": "frozen-o3-reverse-moves"}


def selftest(argument_list: list[str]) -> int:
    problem = prune_server_client.selftest_service_problem()
    if problem:   # 判定库只在 prune-server 里：自证只在临时服务上跑，不写真服务（文件头「用法」--selftest）
        print(f"✗ reuse_library.py 自证没跑：{problem}", file=sys.stderr)
        print(f"     → 怎么办：{prune_server_client.SELFTEST_SERVICE_REMEDY}", file=sys.stderr)
        return 2
    os.environ[SELFTEST_VARIABLE] = "1"
    return harness.run_selftest(os.path.abspath(__file__), argument_list, harness.cells_reporting_progress(selftest_cell_functions()), SELFTEST_SWITCHES,
                                BREAK_VARIABLE, SELFTEST_VARIABLE)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
