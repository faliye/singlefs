#!/usr/bin/env python3
"""E171 第 1 次重跑第一段·修订 6 树：第 11–13 步的多机编排（用户 2026-10-06 定案：「分片 双机62核 给80g」；「长时间重型任务。分片 断点重启 这都不用我说。
自适应多机跑」）。由 run_segment_one_on_the_revision_6_tree_multi_host.sh 在前提与第 0–10 步的续跑核对都放行之后起，在仓根跑。
判法（check_product.py rows）、阈值、限时规则（驱动照 --baseline-timing 现算）与每行驱动的参数照原跑法 run_segment_one_on_the_revision_6_tree.sh
（sha256 869d2c0a…）第 11–13 步；改的只有 --lanes、--threads、--rows、输出根与工作根，另加 --skip-finished（驱动自带的续跑开关）。

用法：
  rows_orchestrator.py run  --repo R --prefix P --suffix=S --resume-tag=T --dumps D --tree TREE --snapshot-digest H --baseline-timing B
                            --check C --logs L --work-root W [--research-root RR]
      跑第 11–13 步。被停、被杀之后同一条命令再起就是续跑：已做完的行（本机的、已取回的别的台的）不重跑，写到一半的重做，资源与分配重新现查现算。
  rows_orchestrator.py plan <同上>
      只现查机器与资源、算分配、按每行推的用时推各台挂钟，打出来；一行都不起，不写产物。
  rows_orchestrator.py --selftest [--cells 格,…]
      假驱动、假 multi-host-run、假机器清单与资源走一遍各格（同目录的 fake_*），末尾逐个打开弄坏开关核它红在点名的格。

退出码：0 跑完（第 11–13 步都绿）；1 停机或某一行没跑成（日志与 sequence.log 写停在哪、为什么）；2 用法错；
  78 运行条件不满足（多机配置判不过、某台连不上或没读到资源、别的台的预检没过（工具链、Python、离线编译、盘）、此刻一条线道都放不下、
  跨机对照那一行此刻放不下）——不悄悄退回单机，出路是修好那台或改配置，再用同一条命令起；143 收到 TERM / INT / HUP，收完尾退出。

机器：scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts 解出的清单（本机是第 1 台，HOSTS 里依次是第 2、3…台；没写 HOSTS 就是 PEER_SSH_HOST
  一台）。判法退 3（ENABLE_ACROSS_MACHINES 没开）只用本机、打一行；退 1 退 78。日志与产物里台名只写「本机」「第二台」……，不写别名与目录。
  别的台每行经 scripts/hosts/multi-host-run.sh --peer 起；它只读配置的 PEER_*，所以不是 PEER_SSH_HOST 的那几台，这里按 --emit-hosts 的
  MULTI_HOST_<号>_* 另写一份只给那一台用的配置（PEER_* 换成那一台的，草稿状态目录里、权限 600），经 SINGLEFS_MULTI_HOST_CONFIG 交给它；那一台没配仓目录或
  cargo 目录退 78。
资源与分配（每次起跑、每次续跑都现查现算，写进日志）：scripts/hosts/host-resources.py --json 逐台读，读不到一台退 78。
  可用线程 = min(cpu.available_threads, 31)。每台槽数 = min(⌊可用线程 / 5⌋, ⌊memory.slice_room / 包装账⌋, ⌊memory.available / 实占⌋,
  ⌊那台配置的内存上限 / 包装账⌋)；每槽线程 = ⌊可用线程 / 槽数⌋（≥ 5：原跑法每道 5，限时基线在这个量级上量的）。
  包装账（内存包装 run-with-memory-cap.sh 在 slice 里排队按的量）：本机每槽 6G，就是装置与 O3 那一层包装的上限（原跑法 CAP_LARGE；行目录每行不同、
    峰值表里没有这条命令，包装按上限记账）；别的台每槽 10G，是那 6G 加外层包装 --memory-cap 4G（multi-host-run.sh 在那边把驱动与编译整个套进一层包装，
    它在 slice 账上一直占着 4G 减已用；4G 管驱动加一次编译）。配置的内存上限（PEER_MEMORY_CAP、HOST_<号>_MEMORY_CAP）卡的是那台各层包装之和。
  实占（memory.available：MemAvailable 减包装账上还没涨到量的那一截）：每槽 10 GiB = 6G + 编译 3 GiB（cargo build 不在装置包装里；峰值表
    ~/.local/share/singlefs/memory-peaks.tsv 里 capped.sh 10 cargo build --release --offline -p singlefs-checker-tier --bin e171_… 那一行 2.54 GiB，向上取整）
    + 驱动与杂项 1 GiB（推的）。本机的编译在 slice 外面，这一项防几道同时冷编译把 slice 外的余量吃光。
行与派发：第 11 步 242、269（带探针）、第 12 步空变异 row-0 与 13 行、跨机对照 X（row-0 在另一台再跑一次），同进一个队列，第 11、12 步同时跑；
  每行一个单线道驱动（--lanes 1），哪一槽空了就从队列头取第一个放得下的（X 只放在第 12 步 row-0 那一台之外）；某一行没跑成就不再派新行，
  在跑的跑完再停。队列按每行推的用时从长到短（最长先做，各台挂钟接近）；用时相同的按 row-0、X、其余行号排。每行推的用时：
  以前量过这一行的（导出目录下早先各趟 segment1-rows*/row-<n>/row-result.tsv 里 status=ok、device_exit=0 的 device_ns，取最大）用它；
  没量过的，带探针的取第 7 步 baseline-run1 的 process_wall（T），不带探针的取第 3 步两份带流三的阳性对照（O1-崩溃判官、RefFS）process_wall 的较大者；
  每行另加 120 秒（编译 36 秒、O3 33 秒取自第 269 行早先那一份 row-result.tsv，加拷树，推的）。
  本机每槽一个固定工作根（同一槽上后一行增量编译，驱动自己核）；第 12 步 row-0 与 X 各用新的工作根（两边都冷编译，跨机对照比得了编译那几列之外的一切）。
第 13 步：第 11、12 步合成的产物里 device_exit=253 的行，照原规则单线道、限时再乘 3（--rerun-of-timed-out-rows）；带探针的先、不带的后，各一个驱动，
  跑在哪台照同一套分配挑（有槽的台里可用线程最多的，平手取编号小的），线程取那台的可用线程（≤ 31）。
产物与合并（写死的合并规则，merge_products）：每一步最后一份原名产物 <PREFIX>-<步名><RESUME_TAG><SUFFIX>.out 与 -timing.out，由「这一趟」的分片产物合成：
  这一趟跑的每行一个分片；更早几趟已做完的行由本机一个 --skip-finished 驱动从行目录重报一份（驱动自带的续跑做法，行与 row-result.tsv 逐字相同）。
  ① 各分片的 name=row_time_limit 行逐字相同，原样留一行，不同就拒；② snapshot_digest 与 files 各分片相同、且等于冻结树快照（发给别的台的那一份）的摘要，
  不同就拒；③ row 行按序号排，同一序号出现两次拒，与这一步登记的行集不等（缺、多）拒；④ freeze_check 前后各一行：differences 相加，detail 并起来；
  ⑤ table 行：lanes、rows、sum_of_row_ns 相加，table_wall_ns 取最大，threads_per_lane 写各分片取值去重升序，另加 shards=<分片数>；
  ⑥ 末尾每个分片一行 merged_shard host=<台号> rows=<行> sha256=<分片产物>；计时按 row_device 分块，块按序号排，行集与产物的不等就拒；
  ⑦ 认不得的行拒。原始分片产物、取回目录与日志留在 <导出目录>/segment1-rows<RESUME_TAG>-shards/attempt-<趟>/ 下。
跨机一致对照（新设，收严）：第 12 步 row-0 与 X 在两台各跑一次，比四样：row 行去掉 copy_ns、compile_ns、device_ns、o3_ns 四个耗时列；
  device.out 的 E7RESULT 行（E7TIMING 不比）；导出里三条流的 oracle-verdicts.tsv、call-outcomes.tsv、writable-mount-probe.tsv、ops.tsv、states.tsv
  （与 check_product.py z1 同一组）；o3.out 去掉 name=config、name=state_time 两种行与键名以 _ns 结尾的字段。有一样不同就停机（退 1，不作废、不当结果）；
  产物 <PREFIX>-rows-cross-host-check<RESUME_TAG><SUFFIX>.out。只有一台时那一格报 not_run。
续跑：别的台那一行被停时 multi-host-run.sh 先停那边、再把 scripts/experiments/target/e171-s1-peer-out（含已做完的行目录）取回本趟取回目录，再删那边的树；
  每次起跑先把以前各趟取回目录里有 row-result.tsv 的行收进本机行输出根（先记账 row-hosts.tsv 再按硬链接拷、拷完改名），本机那一份写到一半的行目录删掉重做。
  MULTI_HOST_RUN_POLL_SECONDS=30：multi-host-run.sh 连续 60 次 ssh 不通才判失败，即容半小时断线（它判失败时删那边的树而不取回，那几行下一趟重做）。

弄坏开关 ROWS_ORCHESTRATOR_BREAK=<项>（逗号连几个；只在 --selftest 起的子进程里认，别处设了退 2）。项名、弄坏哪一处、打开后红在哪一格：
  allocation-ignores-memory   槽数只看线程、不看内存额度            allocation-memory-bound
  merge-allows-duplicates     合并时同一序号两行取前一行、不拒       merge-refuse-duplicate
  merge-ignores-time-limit    合并时各分片 row_time_limit 不同也取第一行  merge-refuse-time-limit
  cross-host-ignores-dumps    跨机对照不比导出那一样                  cross-host-compare
  resume-reruns-finished      续跑时已做完的行也重跑                  e2e-resume
  unreachable-dropped         读不到资源的那台悄悄不用（不退 78）      e2e-unreachable
  harvest-skipped             别的台取回的行不收进本机行输出根         e2e-two-hosts
"""
from __future__ import annotations

import argparse
import dataclasses
import datetime
import hashlib
import json
import os
import re
import shlex
import shutil
import signal
import subprocess
import sys
import tempfile
import threading
import time
import zoneinfo

HERE = os.path.dirname(os.path.abspath(__file__))
RESEARCH_RELATIVE = "scripts/experiments/e171_three_oracle_collusion_after_the_oracle_fixes"
CHECK_PRODUCT = os.path.join(HERE, "check_product.py")  # 判法脚本：--check 只认这一份（各跑法脚本给的都是研究目录里的它），起它时照字面路径起
DRIVER_NAME = "row_driver.py"
GIB = 1 << 30
LANE_THREADS_MINIMUM = 5
HOST_THREADS_CEILING = 31
SLOT_DEVICE_MEMORY_BYTES = 6 * GIB
SLOT_COMPILE_MEMORY_BYTES = 3 * GIB
SLOT_DRIVER_MEMORY_BYTES = 1 * GIB
SLOT_MEMORY_BYTES = SLOT_DEVICE_MEMORY_BYTES + SLOT_COMPILE_MEMORY_BYTES + SLOT_DRIVER_MEMORY_BYTES
DEVICE_MEMORY_CAP = "6G"
PEER_OUTER_MEMORY_CAP = "4G"
PEER_DISK_BYTES_PER_SLOT = 3 * GIB
CARRY_THREADS = 5
ROW_OVERHEAD_SECONDS = 120
MULTI_HOST_RUN_POLL_SECONDS = "30"
PARALLEL_REASON = "同一会话派出去的几个执行员各跑各的装置"
EXIT_STOPPED_BY_SIGNAL = 143
EXIT_RUN_CONDITION = 78
# --send 落点的上一级要在拷过去的树里已有：multi-host-run.sh 用 rsync 送，rsync 不建缺的上级目录，而 scripts/experiments/target/ 不在 git 列得出的文件里
# （2026-10-06 连通自证实测：送到 scripts/experiments/target/… 时第二台 mkdir failed、退 255）。所以 kit 与快照送到 research/ 下。
PEER_KIT_RELATIVE = "research/e171-s1-peer-kit"
PEER_SNAPSHOT_RELATIVE = "research/e171-s1-frozen-snapshot"
PEER_OUT_RELATIVE = "scripts/experiments/target/e171-s1-peer-out"
TIMING_COLUMNS = ("copy_ns", "compile_ns", "device_ns", "o3_ns")
STREAMS = ("new_pool_file_creation", "fixed_script_through_e_then_unmount", "first_file_unlinked_then_inodes_across_one_leaf")
Z1_FILES = ("oracle-verdicts.tsv", "call-outcomes.tsv", "writable-mount-probe.tsv", "ops.tsv", "states.tsv")
HOST_NAMES = ("本机", "第二台", "第三台", "第四台", "第五台", "第六台", "第七台", "第八台", "第九台", "第十台")
BREAK_SWITCHES = ("allocation-ignores-memory", "merge-allows-duplicates", "merge-ignores-time-limit", "cross-host-ignores-dumps",
                  "resume-reruns-finished", "unreachable-dropped", "harvest-skipped")


def broken(item: str) -> bool:
    return item in os.environ.get("ROWS_ORCHESTRATOR_BREAK", "").split(",")


def host_name(number: int) -> str:
    return HOST_NAMES[number - 1] if number <= len(HOST_NAMES) else f"第 {number} 台"


@dataclasses.dataclass(frozen=True)
class Step:
    """一步：key 在日志与账里用；number 是登记的步号；stem 是产物名里 <PREFIX>- 之后、RESUME_TAG 之前那一段；root_suffix 接在行输出根后面；
    flags 是原跑法那一步每行驱动除 --rows / --empty-mutation / --lanes / --threads / --memory-cap / 输出根 / 工作根之外的参数，一字不改。"""
    key: str
    number: int
    stem: str
    root_suffix: str
    flags: tuple[str, ...]


STEPS = {
    "s11": Step("s11", 11, "rows-probe-driver", "", ("--keep-dumps", "--streams", "1,2,3", "--writable-mount-probe")),
    "s12": Step("s12", 12, "rows-driver", "", ("--keep-dumps", "--streams", "1,2,3")),
    "sx": Step("sx", 12, "rows-cross-host-row-0-driver", "-cross-host", ("--keep-dumps", "--streams", "1,2,3")),
    "r11": Step("r11", 13, "rows-probe-timeout-rerun-driver", "-timeout-rerun",
                ("--rerun-of-timed-out-rows", "--keep-dumps", "--streams", "1,2,3", "--writable-mount-probe")),
    "r12": Step("r12", 13, "rows-timeout-rerun-driver", "-timeout-rerun", ("--rerun-of-timed-out-rows", "--keep-dumps", "--streams", "1,2,3")),
}
# 原跑法第 11、12 步的 --rows（第 12 步另带 --empty-mutation，即 row-0）；X 是第 12 步 row-0 在另一台的那一次
STEP_ROWS = {"s11": (242, 269), "s12": (0, 15, 271, 42, 23, 270, 125, 49, 110, 155, 161, 162, 174, 252), "sx": (0,)}
CROSS_HOST_CHECK_STEM = "rows-cross-host-check"


class RunCondition(Exception):
    """运行条件不满足：退 78。"""


class Stopped(Exception):
    """停机或某一行没跑成：退 1。"""


class Interrupted(Exception):
    """收到 TERM / INT / HUP，收完尾：退 143。"""


class MergeRefused(Exception):
    """合并规则拒（文件头「产物与合并」①–⑦）。"""


def step_arguments(step: Step, rows: tuple[int, ...], lanes: int, threads: int) -> list[str]:
    """一个分片驱动除路径之外的参数：行（0 写成 --empty-mutation，同原跑法第 12 步）、原跑法那一步的参数、线道与线程、装置上限、续跑开关。"""
    numbered = sorted(row for row in rows if row != 0)
    arguments: list[str] = []
    if numbered:
        arguments += ["--rows", ",".join(str(row) for row in numbered)]
    if 0 in rows:
        arguments.append("--empty-mutation")
    return arguments + list(step.flags) + ["--lanes", str(lanes), "--threads", str(threads), "--memory-cap", DEVICE_MEMORY_CAP, "--skip-finished"]


def capacity_bytes(text: str) -> int:
    """正整数加 K / M / G / T（1024 进制，同 run-with-memory-cap.sh 的写法）。"""
    match = re.fullmatch(r"([1-9][0-9]*)([KMGT])", text)
    if not match:
        raise ValueError(f"内存上限写法认不得：{text!r}")
    power = "KMGT".index(match.group(2)) + 1
    return int(match.group(1)) * 1024 ** power


@dataclasses.dataclass(frozen=True)
class Host:
    """清单里的一台。alias、目录是私有的，只用来起命令，不打。"""
    number: int
    alias: str
    repository_directory: str
    cargo_bin_directory: str
    memory_cap_text: str
    is_peer: bool

    @property
    def is_local(self) -> bool:
        return self.number == 1


def parse_assignments(text: str) -> dict[str, str]:
    """--emit-hosts 打的 KEY=<printf %q 的值> 一行一个。"""
    values = {}
    for line in text.splitlines():
        if "=" not in line:
            continue
        key, quoted = line.split("=", 1)
        words = shlex.split(quoted)
        values[key] = words[0] if words else ""
    return values


def hosts_from_configuration(exit_code: int, text: str) -> tuple[list[Host], dict[str, str], str]:
    """(机器清单, 配置的键, 只有本机的原因)。判法退 3 只用本机；退 0 之外的别的码是判不过，退 78。"""
    local = Host(1, "", "", "", "", False)
    if exit_code == 3:
        return [local], {}, "双机开关 ENABLE_ACROSS_MACHINES 没开：只用本机"
    if exit_code != 0:
        raise RunCondition(f"多机配置判不过（crash-point-replay-shard-configuration-check.sh --emit-hosts 退 {exit_code}；单跑它看原因）")
    values = parse_assignments(text)
    hosts = [local]
    for number in range(2, int(values.get("MULTI_HOST_COUNT") or 0) + 2):
        prefix = f"MULTI_HOST_{number}_"
        hosts.append(Host(number, values.get(prefix + "SSH_HOST", ""), values.get(prefix + "REPOSITORY_DIRECTORY", ""),
                          values.get(prefix + "CARGO_BIN_DIRECTORY", ""), values.get(prefix + "MEMORY_CAP", ""), values.get(prefix + "IS_PEER") == "1"))
    for host in hosts[1:]:
        if not (host.alias and host.repository_directory and host.cargo_bin_directory):
            raise RunCondition(f"{host_name(host.number)}在配置里缺 ssh 别名、仓目录或 cargo 目录（HOST_{host.number}_*），multi-host-run.sh 起不了那一台")
    return hosts, values, ""


@dataclasses.dataclass(frozen=True)
class HostResources:
    number: int
    available_threads: int
    available_memory: int
    slice_room: int


def resources_from_json(data: dict, hosts: list[Host]) -> tuple[dict[int, HostResources], list[str]]:
    """(每台的资源, 读不到的台说明)。读不到的台交回调用方退 78；弄坏开关 unreachable-dropped 下悄悄丢掉。"""
    by_number = {entry.get("number"): entry for entry in data.get("hosts", [])}
    found: dict[int, HostResources] = {}
    unread: list[str] = []
    for host in hosts:
        entry = by_number.get(host.number) or {}
        cpu, memory = entry.get("cpu") or {}, entry.get("memory") or {}
        if not entry.get("read") or "available_threads" not in cpu or "available" not in memory or "slice_room" not in memory:
            unread.append(f"{host_name(host.number)}没读到资源（{entry.get('problem') or '没报 cpu 或 memory'}）")
            continue
        found[host.number] = HostResources(host.number, int(cpu["available_threads"]), int(memory["available"]), int(memory["slice_room"]))
    if broken("unreachable-dropped"):
        return found, []
    return found, unread


@dataclasses.dataclass(frozen=True)
class HostAllocation:
    number: int
    threads: int
    room_bytes: int
    slots: int
    threads_per_slot: int
    terms: str = ""


def slot_wrapper_bytes(host: Host) -> int:
    """一槽在那台 slice 账上的量：本机 6G；别的台多一层 multi-host-run.sh 的外层包装 4G。"""
    return SLOT_DEVICE_MEMORY_BYTES + (0 if host.is_local else capacity_bytes(PEER_OUTER_MEMORY_CAP))


def allocate(hosts: list[Host], resources: dict[int, HostResources]) -> list[HostAllocation]:
    """文件头「资源与分配」：槽数 = min(⌊可用线程 / 5⌋, ⌊slice_room / 包装账⌋, ⌊available / 10 GiB⌋, ⌊配置上限 / 包装账⌋)，每槽线程 = ⌊可用线程 / 槽数⌋。"""
    allocations = []
    for host in hosts:
        if host.number not in resources:
            continue
        found = resources[host.number]
        threads = max(0, min(found.available_threads, HOST_THREADS_CEILING))
        wrapper = slot_wrapper_bytes(host)
        limits = {f"⌊{threads}/5⌋": threads // LANE_THREADS_MINIMUM, f"⌊slice_room/{wrapper // GIB}G⌋": max(0, found.slice_room) // wrapper,
                  "⌊available/10G⌋": max(0, found.available_memory) // SLOT_MEMORY_BYTES}
        room = max(0, found.slice_room)
        if host.memory_cap_text:
            limits[f"⌊上限 {host.memory_cap_text}/{wrapper // GIB}G⌋"] = capacity_bytes(host.memory_cap_text) // wrapper
            room = min(room, capacity_bytes(host.memory_cap_text))
        slots = threads // LANE_THREADS_MINIMUM if broken("allocation-ignores-memory") else min(limits.values())
        terms = "、".join(f"{name}={value}" for name, value in limits.items())
        allocations.append(HostAllocation(host.number, threads, room, slots, threads // slots if slots else 0, terms))
    return allocations


def single_lane_host(allocations: list[HostAllocation]) -> HostAllocation:
    """第 13 步挑台：有槽的台里可用线程最多的，平手取编号小的。"""
    candidates = [allocation for allocation in allocations if allocation.slots > 0]
    if not candidates:
        raise RunCondition("第 13 步：此刻各台都放不下一条线道（每槽要 5 线程、slice 里 6G（别的台 10G）、实占 10 GiB）")
    return sorted(candidates, key=lambda allocation: (-allocation.threads, allocation.number))[0]


def allocation_text(allocations: list[HostAllocation], resources: dict[int, HostResources]) -> list[str]:
    lines = []
    for allocation in allocations:
        found = resources[allocation.number]
        lines.append(f"分配 {host_name(allocation.number)}：可用线程 {found.available_threads}（取 {allocation.threads}）、"
                     f"内存 available {found.available_memory / GIB:.1f} GiB、slice_room {found.slice_room / GIB:.1f} GiB；"
                     f"槽 {allocation.slots}（{allocation.terms}），每槽线程 {allocation.threads_per_slot}")
    return lines


# ── 每行推的用时、队列、派发次序与挂钟推算（纯函数；派发与推算共用 pick_job）──
@dataclasses.dataclass(frozen=True)
class Job:
    step: str
    rows: tuple[int, ...]
    seconds: float


def process_wall_seconds(path: str) -> float | None:
    if not os.path.exists(path):
        return None
    walls = [float(token.split("=", 1)[1]) for line in open(path, encoding="utf-8") if line.startswith("E7TIMING name=process_wall ")
             for token in line.split() if token.startswith("seconds=")]
    return walls[0] if len(walls) == 1 else None


def measured_device_seconds(dumps_absolute: str, resume_tag: str) -> dict[int, float]:
    """早先各趟行输出根（segment1-rows*，这一趟自己的除外）里 status=ok、device_exit=0 的 device_ns，同一行取最大。"""
    measured: dict[int, float] = {}
    if not os.path.isdir(dumps_absolute):
        return measured
    own = f"segment1-rows{resume_tag}"
    for root_name in sorted(os.listdir(dumps_absolute)):
        if not root_name.startswith("segment1-rows") or root_name == own or root_name.startswith(own + "-"):
            continue
        root = os.path.join(dumps_absolute, root_name)
        for entry in sorted(os.listdir(root)) if os.path.isdir(root) else []:
            result = os.path.join(root, entry, "row-result.tsv")
            if not entry.startswith("row-") or not os.path.exists(result):
                continue
            fields = dict(line.rstrip("\n").split("\t", 1) for line in open(result, encoding="utf-8") if "\t" in line)
            if fields.get("status") == "ok" and fields.get("device_exit") == "0" and fields.get("device_ns", "").isdigit():
                number = int(entry.split("-", 1)[1])
                measured[number] = max(measured.get(number, 0.0), int(fields["device_ns"]) / 1e9)
    return measured


def row_seconds_table(probe_seconds: float, plain_seconds: float, measured: dict[int, float]) -> dict[tuple[str, int], float]:
    table = {}
    for step_key, rows in STEP_ROWS.items():
        for row in rows:
            base = probe_seconds if step_key == "s11" else plain_seconds
            table[(step_key, row)] = measured.get(row, base) + ROW_OVERHEAD_SECONDS if step_key != "sx" else plain_seconds + ROW_OVERHEAD_SECONDS
    return table


def queue_order(jobs: list[Job]) -> list[Job]:
    """最长先做；用时相同的按 row-0、X、其余（再按步、行号）排。"""
    def tiebreak(job: Job) -> int:
        if job.step == "s12" and 0 in job.rows:
            return 0
        return 1 if job.step == "sx" else 2
    return sorted(jobs, key=lambda job: (-job.seconds, tiebreak(job), job.step, job.rows))


def pick_job(queue: list[Job], host: int, row_zero_host: int | None) -> int | None:
    """这一台空出一槽时取队列里第一个放得下的：X 只放在第 12 步 row-0 已放下的那一台之外。"""
    for index, job in enumerate(queue):
        if job.step == "sx" and (row_zero_host is None or row_zero_host == host):
            continue
        return index
    return None


@dataclasses.dataclass(frozen=True)
class Placement:
    job: Job
    host: int
    start: float
    end: float


def simulate(allocations: list[HostAllocation], jobs: list[Job], row_zero_host: int | None) -> tuple[list[Placement], list[Job]]:
    """按派发器同一个取法推每行放在哪台、何时起止（每行用时取 Job.seconds）。交回 (放下的, 放不下的)。"""
    free = sorted((0.0, allocation.number, index) for allocation in allocations for index in range(allocation.slots))
    queue = queue_order(jobs)
    placements: list[Placement] = []
    while queue:
        for slot in sorted(free):
            chosen = pick_job(queue, slot[1], row_zero_host)
            if chosen is None:
                continue
            job = queue.pop(chosen)
            if job.step == "s12" and 0 in job.rows:
                row_zero_host = slot[1]
            placements.append(Placement(job, slot[1], slot[0], slot[0] + job.seconds))
            free.remove(slot)
            free.append((slot[0] + job.seconds, slot[1], slot[2]))
            break
        else:
            break
    return placements, queue


def prediction_text(allocations: list[HostAllocation], placements: list[Placement], deferred: list[Job]) -> list[str]:
    lines = []
    for allocation in allocations:
        mine = [placement for placement in placements if placement.host == allocation.number]
        wall = max((placement.end for placement in mine), default=0.0)
        rows = ",".join(f"{placement.job.step}:{'/'.join(str(row) for row in placement.job.rows)}" for placement in sorted(mine, key=lambda p: p.start))
        lines.append(f"推的挂钟 {host_name(allocation.number)}：{wall:.0f} 秒（约 {wall / 3600:.1f} 小时），{len(mine)} 行 {rows or '-'}")
    if deferred:
        lines.append("放不下（这一趟派不出去）：" + ",".join(f"{job.step}:{'/'.join(str(row) for row in job.rows)}" for job in deferred))
    return lines


# ── 合并（文件头「产物与合并」①–⑦；纯函数，自证喂已知答案）──
def key_values(line: str) -> dict[str, str]:
    return dict(token.split("=", 1) for token in line.split() if "=" in token)


@dataclasses.dataclass(frozen=True)
class Shard:
    """一个分片：host 台号，product、timing 是分片驱动的标准输出与 --timing-output 原文。"""
    label: str
    host: int
    product: str
    timing: str


@dataclasses.dataclass(frozen=True)
class ParsedShard:
    label: str
    host: int
    limit_line: str
    freeze_before: dict[str, str]
    snapshot: dict[str, str]
    rows: dict[int, str]
    table: dict[str, str]
    freeze_after: dict[str, str]
    sha256: str


def parse_shard(shard: Shard) -> ParsedShard:
    lines = shard.product.splitlines()
    kinds = []
    for line in lines:
        if line.startswith("name=row_time_limit "):
            kinds.append("limit")
        elif line.startswith("freeze_check "):
            kinds.append("freeze")
        elif line.startswith("snapshot_digest="):
            kinds.append("snapshot")
        elif line.startswith("row "):
            kinds.append("row")
        elif line.startswith("table "):
            kinds.append("table")
        else:
            raise MergeRefused(f"⑦ 分片 {shard.label} 有认不得的行：{line[:120]!r}")
    row_count = kinds.count("row")
    expected_kinds = ["limit", "freeze", "snapshot"] + ["row"] * row_count + ["table", "freeze"]
    if kinds != expected_kinds or row_count == 0:
        raise MergeRefused(f"⑦ 分片 {shard.label} 的行不是驱动打的次序（row_time_limit、freeze_check、snapshot_digest、row…、table、freeze_check）")
    rows: dict[int, str] = {}
    for line in lines[3:3 + row_count]:
        number = int(key_values(line)["row_number"])
        if number in rows and not broken("merge-allows-duplicates"):
            raise MergeRefused(f"③ 分片 {shard.label} 里第 {number} 行出现两次")
        rows.setdefault(number, line)
    snapshot = key_values(lines[2])
    if int(snapshot.get("rows", -1)) != row_count:
        raise MergeRefused(f"② 分片 {shard.label} 的 snapshot_digest 行 rows={snapshot.get('rows')}，row 行却有 {row_count} 条")
    return ParsedShard(shard.label, shard.host, lines[0], key_values(lines[1]), snapshot, rows, key_values(lines[-2]), key_values(lines[-1]),
                       hashlib.sha256(shard.product.encode("utf-8")).hexdigest())


def timing_blocks(shard: Shard) -> dict[int, list[str]]:
    blocks: dict[int, list[str]] = {}
    current = None
    for line in shard.timing.splitlines():
        if line.startswith("E7TIMING name=row_device "):
            current = int(key_values(line)["row"])
            if current in blocks:
                raise MergeRefused(f"⑥ 分片 {shard.label} 的计时里第 {current} 行出现两次")
            blocks[current] = [line]
        elif current is None:
            raise MergeRefused(f"⑥ 分片 {shard.label} 的计时开头不是 row_device 行")
        else:
            blocks[current].append(line)
    return blocks


def shard_freeze(shard: ParsedShard, which: str) -> dict[str, str]:
    """which 是 freeze_before / freeze_after：按名字取那一组冻结行（逐个点名，不 getattr）。"""
    return {"freeze_before": shard.freeze_before, "freeze_after": shard.freeze_after}[which]


def merged_freeze(parsed: list[ParsedShard], which: str, before: bool) -> str:
    total = sum(int(shard_freeze(shard, which).get("differences", "-1")) for shard in parsed)
    details = sorted({detail for shard in parsed for detail in shard_freeze(shard, which).get("detail", "none").split(",") if detail and detail != "none"})
    return f"freeze_check before={'true' if before else 'false'} differences={total} detail={','.join(details) or 'none'}"


def merge_products(shards: list[Shard], expected_rows: tuple[int, ...], expected_snapshot: tuple[str, str]) -> tuple[str, str]:
    """交回 (合成的产物, 合成的计时)。expected_snapshot 是 (摘要, 文件数)。"""
    if not shards:
        raise MergeRefused("没有分片")
    parsed = [parse_shard(shard) for shard in shards]
    limits = sorted({shard.limit_line for shard in parsed})
    if len(limits) != 1 and not broken("merge-ignores-time-limit"):
        raise MergeRefused("① 各分片的 row_time_limit 行不同：" + " | ".join(limits))
    for shard in parsed:
        if (shard.snapshot.get("snapshot_digest"), shard.snapshot.get("files")) != expected_snapshot:
            raise MergeRefused(f"② 分片 {shard.label} 的快照 {shard.snapshot.get('snapshot_digest')}/{shard.snapshot.get('files')} 不是冻结树那一份 "
                               f"{expected_snapshot[0]}/{expected_snapshot[1]}")
    rows: dict[int, str] = {}
    for shard in parsed:
        for number, line in shard.rows.items():
            if number in rows and not broken("merge-allows-duplicates"):
                raise MergeRefused(f"③ 第 {number} 行在两个分片里都有")
            rows.setdefault(number, line)
    if sorted(rows) != sorted(expected_rows):
        missing = sorted(set(expected_rows) - set(rows))
        extra = sorted(set(rows) - set(expected_rows))
        raise MergeRefused(f"③ 行集不等：缺 {missing or '无'}，多 {extra or '无'}")
    threads = sorted({int(shard.table.get("threads_per_lane", "0")) for shard in parsed})
    lines = [parsed[0].limit_line if broken("merge-ignores-time-limit") else limits[0],
             merged_freeze(parsed, "freeze_before", True),
             f"snapshot_digest={expected_snapshot[0]} files={expected_snapshot[1]} "
             f"snapshot_ns={max(int(shard.snapshot.get('snapshot_ns', '0')) for shard in parsed)} rows={len(rows)}"]
    lines += [rows[number] for number in sorted(rows)]
    lines.append(f"table lanes={sum(int(shard.table.get('lanes', '0')) for shard in parsed)} threads_per_lane={','.join(str(value) for value in threads)} "
                 f"rows={sum(int(shard.table.get('rows', '0')) for shard in parsed)} "
                 f"table_wall_ns={max(int(shard.table.get('table_wall_ns', '0')) for shard in parsed)} "
                 f"sum_of_row_ns={sum(int(shard.table.get('sum_of_row_ns', '0')) for shard in parsed)} shards={len(parsed)}")
    lines.append(merged_freeze(parsed, "freeze_after", False))
    for shard in sorted(parsed, key=lambda item: (min(item.rows), item.label)):
        lines.append(f"merged_shard host={shard.host} rows={','.join(str(number) for number in sorted(shard.rows))} sha256={shard.sha256}")
    blocks: dict[int, list[str]] = {}
    for shard, parsed_shard in zip(shards, parsed):
        mine = timing_blocks(shard)
        if sorted(mine) != sorted(parsed_shard.rows):
            raise MergeRefused(f"⑥ 分片 {shard.label} 的计时行集 {sorted(mine)} 与产物的 {sorted(parsed_shard.rows)} 不等")
        for number, block in mine.items():
            blocks.setdefault(number, block)
    timing_lines = [line for number in sorted(blocks) for line in blocks[number]]
    return "\n".join(lines) + "\n", "\n".join(timing_lines) + ("\n" if timing_lines else "")


# ── 跨机一致对照（文件头「跨机一致对照」）──
def row_line_without_timing(line: str) -> str:
    return " ".join(token for token in line.split(" ") if token.split("=", 1)[0] not in TIMING_COLUMNS)


def device_result_lines(row_directory: str) -> list[str]:
    path = os.path.join(row_directory, "device.out")
    return [line for line in open(path, encoding="utf-8")] if os.path.exists(path) else []


def normalized_o3_lines(row_directory: str) -> list[str]:
    path = os.path.join(row_directory, "o3.out")
    if not os.path.exists(path):
        return []
    kept = []
    for line in open(path, encoding="utf-8"):
        if line.startswith("name=config ") or line.startswith("name=state_time "):
            continue
        kept.append(" ".join(token for token in line.rstrip("\n").split(" ") if not token.split("=", 1)[0].endswith("_ns")))
    return kept


def dump_differences(first: str, second: str) -> tuple[int, list[str]]:
    """同 check_product.py z1：三条流各五个文件，两边都没有的不算，一边没有或字节不同记一处。交回 (比了几个, 不同的)。"""
    compared, differing = 0, []
    for stream in STREAMS:
        for name in Z1_FILES:
            left, right = os.path.join(first, "dump", stream, name), os.path.join(second, "dump", stream, name)
            if not os.path.exists(left) and not os.path.exists(right):
                continue
            compared += 1
            if not (os.path.exists(left) and os.path.exists(right)) or open(left, "rb").read() != open(right, "rb").read():
                differing.append(f"{stream}/{name}")
    return compared, differing


def cross_host_parts(line_a: str, directory_a: str, line_b: str, directory_b: str) -> list[tuple[str, bool, str]]:
    """四样各一行：(样, 相同, 细节)。"""
    parts = []
    parts.append(("row_line", row_line_without_timing(line_a) == row_line_without_timing(line_b), "dropped=" + ",".join(TIMING_COLUMNS)))
    device_a = [line for line in device_result_lines(directory_a) if line.startswith("E7RESULT ")]
    device_b = [line for line in device_result_lines(directory_b) if line.startswith("E7RESULT ")]
    parts.append(("device_results", bool(device_a) and device_a == device_b, f"lines={len(device_a)}/{len(device_b)}"))
    compared, differing = dump_differences(directory_a, directory_b)
    if broken("cross-host-ignores-dumps"):
        compared, differing = max(compared, 1), []
    parts.append(("dump_files", compared > 0 and not differing, f"compared={compared} differing={','.join(differing) or 'none'}"))
    o3_a, o3_b = normalized_o3_lines(directory_a), normalized_o3_lines(directory_b)
    parts.append(("o3", bool(o3_a) and o3_a == o3_b, f"lines={len(o3_a)}/{len(o3_b)} dropped=name=config,name=state_time,*_ns"))
    return parts


def cross_host_product(parts: list[tuple[str, bool, str]] | None, host_a: int | None, host_b: int | None, reason: str) -> tuple[str, bool]:
    """交回 (产物原文, 是否停机)。parts 是 None 时这一格本次未跑。"""
    lines = ["E7RESULT name=cross_host_columns row=0 row_line_dropped=" + ",".join(TIMING_COLUMNS)
             + " device=E7RESULT_lines dump=" + ",".join(Z1_FILES) + " o3_dropped=name=config,name=state_time,*_ns"]
    if parts is None:
        lines.append(f"E7RESULT name=verdict cross_host_identical=not_run row=0 reason={reason}")
        return "\n".join(lines) + "\n", False
    for part, identical, detail in parts:
        lines.append(f"E7RESULT name=cross_host_part part={part} identical={'true' if identical else 'false'} {detail}")
    differing = [part for part, identical, _ in parts if not identical]
    lines.append(f"E7RESULT name=verdict cross_host_identical={'false' if differing else 'true'} row=0 host_of_step_12_row_0={host_a} "
                 f"host_of_cross_host_row_0={host_b} parts={len(parts)} differing_parts={','.join(differing) or 'none'}")
    return "\n".join(lines) + "\n", bool(differing)


# ── 跑的时候：路径、日志、账、取回 ──
@dataclasses.dataclass
class Context:
    repo: str
    research_root: str
    prefix: str
    suffix: str
    tag: str
    dumps: str
    tree: str
    snapshot_digest: str
    baseline_timing: str
    check: str
    logs: str
    work_root: str
    driver_path: str
    driver_overridden: bool
    multi_host_run: str
    hosts_command: str
    resources_command: str
    poll_seconds: float
    peer_compile_check: bool
    attempt_dir: str = ""

    @property
    def rows_root(self) -> str:
        return os.path.join(self.repo, self.dumps, f"segment1-rows{self.tag}")

    @property
    def shards(self) -> str:
        return self.rows_root + "-shards"

    def step_root(self, step: Step) -> str:
        return self.rows_root + step.root_suffix

    def product(self, stem: str) -> str:
        return os.path.join(self.repo, f"{self.prefix}-{stem}{self.tag}{self.suffix}.out")

    def timing(self, stem: str) -> str:
        return os.path.join(self.repo, f"{self.prefix}-{stem}{self.tag}{self.suffix}-timing.out")


def today() -> str:
    return datetime.datetime.now(zoneinfo.ZoneInfo("Asia/Tokyo")).date().isoformat()


def say(context: Context, text: str) -> None:
    line = f"{today()} {text}"
    print(line, flush=True)
    os.makedirs(context.logs, exist_ok=True)
    with open(os.path.join(context.logs, "sequence.log"), "a", encoding="utf-8") as handle:
        handle.write(line + "\n")
    if context.attempt_dir:
        with open(os.path.join(context.attempt_dir, "orchestrator.log"), "a", encoding="utf-8") as handle:
            handle.write(line + "\n")


_research_modules: dict[str, object] = {}


def row_driver_module(context: Context):
    """真的 row_driver（快照怎么拷、摘要怎么算只认它这一份）。"""
    if "row_driver" not in _research_modules:
        sys.path.insert(0, os.path.join(context.research_root, RESEARCH_RELATIVE))
        import row_driver  # noqa: E402
        _research_modules["row_driver"] = row_driver
    return _research_modules["row_driver"]


def snapshot_check(context: Context) -> None:
    """作废 Z5（原跑法 snapshot_check）：冻结树整棵的快照摘要没变。"""
    digest = row_driver_module(context).snapshot_digest(os.path.join(context.repo, context.tree))[0]
    if digest != context.snapshot_digest:
        raise Stopped(f"冻结树快照摘要变了（Z5）：{digest}")


def sha256_of_file(path: str) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as handle:
        for block in iter(lambda: handle.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def driver_files(context: Context) -> list[tuple[str, str]]:
    """驱动与它在同一目录里引的模块、两张行表：(本机路径, 别的台那棵树里的相对路径)。跑的中途改了就停（每行起之前核）。"""
    if context.driver_overridden:
        return [(context.driver_path, f"{PEER_KIT_RELATIVE}/driver/{os.path.basename(context.driver_path)}")]
    directory = os.path.dirname(context.driver_path)
    pending, seen = [os.path.splitext(os.path.basename(context.driver_path))[0]], set()
    while pending:
        module = pending.pop()
        path = os.path.join(directory, module + ".py")
        if module in seen or not os.path.exists(path):
            continue
        seen.add(module)
        for line in open(path, encoding="utf-8"):
            match = re.match(r"(?:from|import)\s+([A-Za-z_][A-Za-z0-9_]*)", line)
            if match:
                pending.append(match.group(1))
    files = sorted([os.path.join(directory, module + ".py") for module in seen]
                   + [os.path.join(directory, name) for name in os.listdir(directory) if name.endswith(".tsv")])
    return [(path, os.path.relpath(path, context.research_root)) for path in files]


def driver_hashes(context: Context) -> dict[str, str]:
    return {peer_path: sha256_of_file(local_path) for local_path, peer_path in driver_files(context)}


def read_ledger(context: Context) -> dict[tuple[str, int], int]:
    """row-hosts.tsv：步、行、台号、趟、怎么来的；同一 (步, 行) 后写的算。"""
    path = os.path.join(context.shards, "row-hosts.tsv")
    ledger: dict[tuple[str, int], int] = {}
    if os.path.exists(path):
        for line in open(path, encoding="utf-8"):
            step, row, host = line.rstrip("\n").split("\t")[:3]
            ledger[(step, int(row))] = int(host)
    return ledger


def append_ledger(context: Context, step: str, row: int, host: int, how: str) -> None:
    os.makedirs(context.shards, exist_ok=True)
    with open(os.path.join(context.shards, "row-hosts.tsv"), "a", encoding="utf-8") as handle:
        handle.write(f"{step}\t{row}\t{host}\t{os.path.basename(context.attempt_dir)}\t{how}\n")


def is_done(context: Context, step: Step, row: int) -> bool:
    return os.path.exists(os.path.join(context.step_root(step), f"row-{row}", "row-result.tsv"))


def host_of(context: Context, ledger: dict[tuple[str, int], int], step: str, row: int) -> int:
    """账上没有的做完的行是本机驱动直接写进行输出根的（取回的先记账再拷）。"""
    return ledger.get((step, row), 1)


def link_or_copy(source: str, destination: str) -> None:
    try:
        os.link(source, destination)
    except OSError:
        shutil.copy2(source, destination)


def harvest_fetch(context: Context, record: dict) -> list[int]:
    """把一个别的台分片取回目录里有 row-result.tsv 的行收进本机行输出根：先记账、硬链接拷进 <行>.harvest-partial、再改名；本机写到一半的同名行目录删掉。"""
    if broken("harvest-skipped"):
        return []
    rows_directory = os.path.join(record["fetch_dir"], PEER_OUT_RELATIVE, "rows")
    step = STEPS[record["step"]]
    harvested = []
    for entry in sorted(os.listdir(rows_directory)) if os.path.isdir(rows_directory) else []:
        source = os.path.join(rows_directory, entry)
        if not entry.startswith("row-") or not os.path.exists(os.path.join(source, "row-result.tsv")):
            continue
        row = int(entry.split("-", 1)[1])
        target = os.path.join(context.step_root(step), entry)
        if os.path.exists(os.path.join(target, "row-result.tsv")):
            continue
        append_ledger(context, step.key, row, record["host"], "fetched")
        os.makedirs(context.step_root(step), exist_ok=True)
        partial = target + ".harvest-partial"
        shutil.rmtree(partial, ignore_errors=True)
        shutil.copytree(source, partial, copy_function=link_or_copy)
        if os.path.isdir(target):
            shutil.rmtree(target)
        os.rename(partial, target)
        harvested.append(row)
    return harvested


def shard_records(context: Context) -> list[dict]:
    records = []
    for attempt in sorted(os.listdir(context.shards)) if os.path.isdir(context.shards) else []:
        directory = os.path.join(context.shards, attempt)
        if not attempt.startswith("attempt-") or not os.path.isdir(directory):
            continue
        for entry in sorted(os.listdir(directory)):
            path = os.path.join(directory, entry, "shard.json")
            if os.path.exists(path):
                records.append(json.load(open(path, encoding="utf-8")))
    return records


def has_google_crc32c() -> bool:
    try:
        import google_crc32c  # noqa: F401
    except ImportError:
        return False
    return True


def build_snapshot_staging(context: Context) -> tuple[str, str]:
    """冻结树里驱动要拷的那几样（Cargo.toml、Cargo.lock、.cargo、crates，不带 target）照驱动自己的 sync_tree 拷一份，发给别的台；摘要也照驱动的算。"""
    module = row_driver_module(context)
    staging = os.path.join(context.shards, "frozen-snapshot")
    module.sync_tree(os.path.join(context.repo, context.tree), staging)
    digest, count = module.snapshot_digest(staging)
    return digest, str(count)


def expected_description(context: Context, snapshot: tuple[str, str]) -> dict:
    driver_peer_path = driver_files(context)[0][1] if context.driver_overridden else os.path.relpath(context.driver_path, context.research_root)
    return {"python": f"{sys.version_info.major}.{sys.version_info.minor}", "google_crc32c": has_google_crc32c(),
            "baseline_timing": [context.baseline_timing, sha256_of_file(os.path.join(context.repo, context.baseline_timing))],
            "files": driver_hashes(context), "snapshot": [PEER_SNAPSHOT_RELATIVE, snapshot[0], snapshot[1]], "research_directory": RESEARCH_RELATIVE,
            "driver": driver_peer_path, "disk_bytes_per_slot": PEER_DISK_BYTES_PER_SLOT, "out": PEER_OUT_RELATIVE}


def build_peer_kit(context: Context, expected: dict) -> str:
    kit = os.path.join(context.attempt_dir, "peer-kit")
    os.makedirs(kit)
    shutil.copy2(os.path.join(HERE, "peer_side.py"), os.path.join(kit, "peer_side.py"))
    if context.driver_overridden:
        os.makedirs(os.path.join(kit, "driver"))
        shutil.copy2(context.driver_path, os.path.join(kit, "driver", os.path.basename(context.driver_path)))
    with open(os.path.join(kit, "expected.json"), "x", encoding="utf-8") as handle:
        json.dump(expected, handle, ensure_ascii=False, indent=1)
    return kit


def configuration_for(context: Context, host: Host, values: dict[str, str]) -> str | None:
    """不是 PEER_SSH_HOST 的那一台：另写一份只给它用的配置（文件头「机器」）。PEER_SSH_HOST 那一台用原配置，交回 None。"""
    if host.is_peer:
        return None
    keys = {"PEER_SSH_HOST": host.alias, "PEER_REPOSITORY_DIRECTORY": host.repository_directory, "PEER_CARGO_BIN_DIRECTORY": host.cargo_bin_directory,
            "PEER_MEMORY_CAP": host.memory_cap_text or values.get("PEER_MEMORY_CAP", "")}
    for key in ("QUIESCE_STOP_COMMAND", "QUIESCE_STOPPED_CHECK_COMMAND", "QUIESCE_START_COMMAND", "QUIESCE_STARTED_CHECK_COMMAND"):
        keys[key] = values.get(key, "")
    keys.update({"ENABLE_ACROSS_MACHINES": "1", "ENABLE_GPU": values.get("ENABLE_GPU", "0")})
    if any("'" in value or "\n" in value for value in keys.values()):
        raise RunCondition(f"{host_name(host.number)}的配置值里有单引号或换行，写不成单台配置")
    path = os.path.join(context.attempt_dir, f"host-{host.number}.env")
    descriptor = os.open(path, os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
    with os.fdopen(descriptor, "w", encoding="utf-8") as handle:
        handle.write("# rows_orchestrator.py 按 --emit-hosts 为这一台另写的单台配置（PEER_* 换成这一台的），只给 multi-host-run.sh 用\n")
        for key, value in keys.items():
            handle.write(f"{key}='{value}'\n")
    return path


def driver_environment() -> dict[str, str]:
    return dict(os.environ, PYTHONDONTWRITEBYTECODE="1", RUN_WITH_MEMORY_CAP_PARALLEL_REASON=PARALLEL_REASON)


def write_record(directory: str, record: dict) -> None:
    with open(os.path.join(directory, "shard.json"), "x", encoding="utf-8") as handle:
        json.dump(record, handle, ensure_ascii=False, indent=1)


def launch_local(context: Context, expected_hashes: dict[str, str], label: str, step: Step, rows: tuple[int, ...], lanes: int, threads: int,
                 work_root: str, kind: str) -> tuple[subprocess.Popen, dict]:
    directory = os.path.join(context.attempt_dir, label)
    os.makedirs(directory)
    if driver_hashes(context) != expected_hashes:
        raise Stopped("驱动或它引的模块、行表在这一趟里改过了（每行的命令一字不改）：停")
    product, timing = os.path.join(directory, "product.out"), os.path.join(directory, "timing.out")
    command = ["nice", "-n", "19", "python3", "-B", context.driver_path, "--repo-root", context.repo, "--work-root", work_root,
               "--output-root", context.step_root(step), "--snapshot-from", os.path.join(context.repo, context.tree),
               "--baseline-timing", os.path.join(context.repo, context.baseline_timing), "--timing-output", timing] + step_arguments(step, rows, lanes, threads)
    record = {"label": label, "step": step.key, "rows": list(rows), "host": 1, "kind": kind, "product": product, "timing": timing,
              "attempt": os.path.basename(context.attempt_dir), "command": command[5:]}
    write_record(directory, record)
    with open(product, "x", encoding="utf-8") as out, open(os.path.join(directory, "driver.stderr"), "x", encoding="utf-8") as err:
        process = subprocess.Popen(command, cwd=context.repo, stdout=out, stderr=err, env=driver_environment(), start_new_session=True)
    return process, record


def peer_label(context: Context, label: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]", "-", f"e171s1{context.tag}-{os.path.basename(context.attempt_dir)}-{label}")


def multi_host_command(context: Context, label: str, kit: str, inner: list[str]) -> list[str]:
    return (["bash", context.multi_host_run, "--peer", "--memory-cap", PEER_OUTER_MEMORY_CAP, "--label", peer_label(context, label),
             "--send", kit, PEER_KIT_RELATIVE, "--send", os.path.join(context.shards, "frozen-snapshot"), PEER_SNAPSHOT_RELATIVE,
             "--fetch", PEER_OUT_RELATIVE, "--", "env", "PYTHONDONTWRITEBYTECODE=1", f"RUN_WITH_MEMORY_CAP_PARALLEL_REASON={PARALLEL_REASON}",
             "nice", "-n", "19", "python3", "-B", f"{PEER_KIT_RELATIVE}/peer_side.py"] + inner)


def multi_host_environment(fetch_directory: str, configuration_file: str | None) -> dict[str, str]:
    environment = dict(os.environ, MULTI_HOST_RUN_FETCH_DIRECTORY=fetch_directory, MULTI_HOST_RUN_POLL_SECONDS=MULTI_HOST_RUN_POLL_SECONDS)
    if configuration_file:
        environment["SINGLEFS_MULTI_HOST_CONFIG"] = configuration_file
    return environment


def launch_peer(context: Context, kit: str, label: str, step: Step, rows: tuple[int, ...], threads: int, host: int,
                configuration_file: str | None) -> tuple[subprocess.Popen, dict]:
    directory = os.path.join(context.attempt_dir, label)
    os.makedirs(directory)
    fetch = os.path.join(directory, "fetch")
    inner = ["run-driver", "--expected", f"{PEER_KIT_RELATIVE}/expected.json", "--"] + step_arguments(step, rows, 1, threads)
    command = multi_host_command(context, label, kit, inner)
    record = {"label": label, "step": step.key, "rows": list(rows), "host": host, "kind": "peer", "fetch_dir": fetch,
              "product": os.path.join(fetch, PEER_OUT_RELATIVE, "product.out"), "timing": os.path.join(fetch, PEER_OUT_RELATIVE, "timing.out"),
              "attempt": os.path.basename(context.attempt_dir), "command": inner}
    write_record(directory, record)
    with open(os.path.join(directory, "multi-host-run.out"), "x", encoding="utf-8") as out, \
            open(os.path.join(directory, "multi-host-run.err"), "x", encoding="utf-8") as err:
        process = subprocess.Popen(command, cwd=context.repo, stdout=out, stderr=err, env=multi_host_environment(fetch, configuration_file),
                                   start_new_session=True)
    return process, record


def preflight_peer(context: Context, kit: str, host: Host, slots: int, threads: int, configuration_file: str | None) -> list[str]:
    """别的台的预检：multi-host-run.sh 自己比工具链（不同退 255），peer_side.py preflight 核 Python、google_crc32c、rsync、基线计时与驱动的 sha256、
    快照摘要、O3 冻结行、盘，--compile 时把冻结快照离线编一遍。没过退 78。"""
    label = f"preflight-h{host.number}"
    directory = os.path.join(context.attempt_dir, label)
    os.makedirs(directory)
    fetch = os.path.join(directory, "fetch")
    inner = ["preflight", "--expected", f"{PEER_KIT_RELATIVE}/expected.json", "--slots", str(slots), "--threads", str(threads)]
    if context.peer_compile_check:
        inner.append("--compile")
    with open(os.path.join(directory, "multi-host-run.out"), "x", encoding="utf-8") as out, \
            open(os.path.join(directory, "multi-host-run.err"), "x", encoding="utf-8") as err:
        code = subprocess.run(multi_host_command(context, label, kit, inner), cwd=context.repo, stdout=out, stderr=err,
                              env=multi_host_environment(fetch, configuration_file)).returncode
    report_path = os.path.join(fetch, PEER_OUT_RELATIVE, "preflight.txt")
    report = open(report_path, encoding="utf-8").read().splitlines() if os.path.exists(report_path) else []
    if code != 0:
        tail = open(os.path.join(directory, "multi-host-run.err"), encoding="utf-8").read().splitlines()[-4:]
        failed = [line for line in report if " ok=false" in line] or tail
        raise RunCondition(f"{host_name(host.number)}预检没过（multi-host-run.sh 退 {code}；255 是它自己没跑成：配置、工具链、拷树、连不上）："
                           + " | ".join(failed))
    return report


# ── 派发、收尾、重报、合并装进 research/results ──
STOP = threading.Event()


def on_signal(signal_number: int, frame) -> None:  # noqa: ARG001
    STOP.set()


@dataclasses.dataclass
class Running:
    job: Job
    slot: tuple[int, int]
    process: subprocess.Popen
    record: dict


def rows_text(rows) -> str:
    return ",".join(str(row) for row in rows)


def finish_shard(context: Context, record: dict, code: int) -> bool:
    """一个分片结束：记退出码；别的台的把取回的行收进本机；本机的把做完的行记账。交回跑成没有（退 0、点名的行都有 row-result.tsv、产物在）。"""
    with open(os.path.join(context.attempt_dir, record["label"], "exit"), "w", encoding="utf-8") as handle:
        handle.write(f"{code}\n")
    step = STEPS[record["step"]]
    if record["kind"] == "peer":
        harvest_fetch(context, record)
    elif record["kind"] == "local":  # 重报（carry）不真跑，不记台：那几行在哪台跑的，账上早先已记
        for row in record["rows"]:
            if is_done(context, step, row):
                append_ledger(context, step.key, row, 1, "local")
    done = all(is_done(context, step, row) for row in record["rows"])
    ok = code == 0 and done and os.path.exists(record["product"])
    note = "（250–254、245 是内存包装自己的结局，那一次不算）" if 245 == code or 250 <= code <= 254 else ""
    say(context, f"  第 {step.number} 步 {step.key} 第 {rows_text(record['rows'])} 行 {host_name(record['host'])} {'跑完' if ok else '没跑成'}："
                 f"退 {code}{note}，行目录 {'都有' if done else '不全有'} row-result.tsv")
    return ok


def stop_running(context: Context, running: list[Running]) -> None:
    """收到停止信号：给每个在跑的分片（各自一个进程组）发一次 TERM；multi-host-run.sh 收到后先停那边、取回、再删树。等它们都退出，再把取回的行收进来。"""
    for item in running:
        try:
            os.killpg(item.process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
    for item in running:
        finish_shard(context, item.record, item.process.wait())


def run_queue(context: Context, allocations: list[HostAllocation], jobs: list[Job], row_zero_host: int | None, expected_hashes: dict[str, str],
              kit: str | None, configurations: dict[int, str | None]) -> tuple[list[dict], list[Job], list[str]]:
    """交回 (跑成的分片记录, 放不下的, 没跑成的说明)。某一行没跑成就不再派新行，在跑的跑完。"""
    free = [(allocation.number, index) for allocation in allocations for index in range(allocation.slots)]
    threads_of = {allocation.number: allocation.threads_per_slot for allocation in allocations}
    queue = queue_order(jobs)
    running: list[Running] = []
    succeeded: list[dict] = []
    failures: list[str] = []
    while not STOP.is_set():
        for slot in sorted(free) if not failures else []:
            chosen = pick_job(queue, slot[0], row_zero_host)
            if chosen is None:
                continue
            job = queue.pop(chosen)
            step = STEPS[job.step]
            label = f"{job.step}-row-{'-'.join(str(row) for row in job.rows)}-h{slot[0]}"
            try:
                snapshot_check(context)  # 作废 Z5：同原跑法 driver_step，每个驱动起之前核一次
                if slot[0] == 1:
                    fresh = 0 in job.rows or job.step in ("r11", "r12")
                    work_root = os.path.join(context.work_root, f"{os.path.basename(context.attempt_dir)}-{label}" if fresh else f"host1-slot{slot[1]}")
                    process, record = launch_local(context, expected_hashes, label, step, job.rows, 1, threads_of[1], work_root, "local")
                else:
                    assert kit is not None, "有别的台的槽就建过发给它的那一份"
                    process, record = launch_peer(context, kit, label, step, job.rows, threads_of[slot[0]], slot[0], configurations.get(slot[0]))
            except Stopped as error:
                failures.append(str(error))
                break
            if job.step == "s12" and 0 in job.rows:
                row_zero_host = slot[0]
            free.remove(slot)
            running.append(Running(job, slot, process, record))
            say(context, f"  ▶ 第 {step.number} 步 {job.step} 第 {rows_text(job.rows)} 行 → {host_name(slot[0])} 第 {slot[1]} 槽（{threads_of[slot[0]]} 线程）")
        for item in list(running):
            code = item.process.poll()
            if code is None:
                continue
            running.remove(item)
            free.append(item.slot)
            if finish_shard(context, item.record, code):
                succeeded.append(item.record)
            else:
                failures.append(f"{item.job.step} 第 {rows_text(item.job.rows)} 行（{host_name(item.slot[0])}，退 {code}，见 {item.record['label']}/）")
            try:
                snapshot_check(context)  # 作废 Z5：每个驱动收尾再核一次
            except Stopped as error:
                failures.append(str(error))
        if not running and (failures or not queue or all(pick_job(queue, slot[0], row_zero_host) is None for slot in free)):
            break
        time.sleep(context.poll_seconds)
    if STOP.is_set():
        stop_running(context, running)
        raise Interrupted()
    return succeeded, queue, failures


def install(context: Context, path: str, text: str) -> None:
    """新文件才写（先写进 shards/merged/ 再硬链接过去，已存在就不覆盖）；已存在而与这一次合成的逐字节相同算装过，不同就停。"""
    if os.path.exists(path):
        if open(path, encoding="utf-8").read() == text:
            return
        raise Stopped(f"产物已存在且与这一次合成的不同：{os.path.relpath(path, context.repo)}")
    staging = os.path.join(context.shards, "merged")
    os.makedirs(staging, exist_ok=True)
    partial = os.path.join(staging, f"{os.path.basename(path)}.{os.path.basename(context.attempt_dir)}.partial")
    with open(partial, "w", encoding="utf-8") as handle:
        handle.write(text)
    try:
        os.link(partial, path)
    except FileExistsError:
        raise Stopped(f"产物在合成的同时被别人写了：{os.path.relpath(path, context.repo)}")
    except OSError:
        with open(path, "x", encoding="utf-8") as handle:
            handle.write(text)
    os.remove(partial)


def run_carry(context: Context, step: Step, rows: tuple[int, ...], expected_hashes: dict[str, str]) -> dict:
    """更早几趟已做完的行：本机一个 --skip-finished 驱动从行目录重报（不编、不跑装置），这一趟才有一份覆盖全部行的分片。"""
    work_root = os.path.join(context.attempt_dir, f"carry-work-{step.key}")
    process, record = launch_local(context, expected_hashes, f"{step.key}-carry", step, rows, 1, CARRY_THREADS, work_root, "carry")
    code = process.wait()
    if STOP.is_set():
        raise Interrupted()
    if not finish_shard(context, record, code):
        raise Stopped(f"第 {step.number} 步 {step.key}：本机重报已做完的第 {rows_text(rows)} 行那个驱动没跑成（退 {code}）")
    return record


def merge_step(context: Context, step: Step, expected_rows: tuple[int, ...], records: list[dict], snapshot: tuple[str, str],
               expected_hashes: dict[str, str]) -> str:
    mine = [record for record in records if record["step"] == step.key]
    covered = {row for record in mine for row in record["rows"]}
    carry_rows = tuple(sorted(row for row in expected_rows if row not in covered))
    if carry_rows:
        mine.append(run_carry(context, step, carry_rows, expected_hashes))
    shards = [Shard(record["label"], record["host"], open(record["product"], encoding="utf-8").read(),
                    open(record["timing"], encoding="utf-8").read()) for record in mine]
    try:
        product_text, timing_text = merge_products(shards, expected_rows, snapshot)
    except MergeRefused as error:
        raise Stopped(f"第 {step.number} 步 {step.key} 合并被拒：{error}")
    install(context, context.timing(step.stem), timing_text)
    install(context, context.product(step.stem), product_text)
    product = os.path.relpath(context.product(step.stem), context.repo)
    say(context, f"  {step.stem}{context.tag} 合成：{len(shards)} 个分片，产物 {product}（{product_text.count(chr(10))} 行），计时 "
                 f"{os.path.relpath(context.timing(step.stem), context.repo)}")
    return context.product(step.stem)


# ── 主流程 ──
def load_hosts(context: Context) -> tuple[list[Host], dict[str, str], str]:
    completed = subprocess.run(["bash", "-c", context.hosts_command], cwd=context.repo, capture_output=True, text=True)
    return hosts_from_configuration(completed.returncode, completed.stdout)


def query_resources(context: Context, hosts: list[Host]) -> dict[int, HostResources]:
    completed = subprocess.run(["bash", "-c", context.resources_command], cwd=context.repo, capture_output=True, text=True)
    if completed.returncode != 0:
        raise RunCondition(f"host-resources.py --json 退 {completed.returncode}")
    found, unread = resources_from_json(json.loads(completed.stdout), hosts)
    if unread:
        raise RunCondition("；".join(unread) + "（不悄悄少用一台：修好那台，或在多机配置里把它去掉）")
    return found


def timed_out_rows(product: str) -> tuple[int, ...]:
    rows = [int(key_values(line)["row_number"]) for line in open(product, encoding="utf-8") if line.startswith("row ") and " device_exit=253 " in line]
    return tuple(sorted(rows))


def row_seconds(context: Context) -> dict[tuple[str, int], float]:
    probe = process_wall_seconds(os.path.join(context.repo, context.baseline_timing))
    plain = [process_wall_seconds(os.path.join(context.repo, f"{context.prefix}-{name}{context.suffix}-timing.out"))
             for name in ("positive-control-o1-crash-judge", "positive-control-reference-filesystem")]
    plain_seconds = max([value for value in plain if value is not None], default=probe or 0.0)
    return row_seconds_table(probe or 0.0, plain_seconds, measured_device_seconds(os.path.join(context.repo, context.dumps), context.tag))


def next_attempt_directory(context: Context) -> str:
    os.makedirs(context.shards, exist_ok=True)
    numbers = [int(name.split("-", 1)[1]) for name in os.listdir(context.shards) if re.fullmatch(r"attempt-[0-9]+", name)]
    path = os.path.join(context.shards, f"attempt-{max(numbers, default=0) + 1}")
    os.makedirs(path)
    return path


def check_rows(context: Context, product: str) -> None:
    completed = subprocess.run(["python3", "-B", CHECK_PRODUCT, "rows", product], cwd=context.repo, capture_output=True, text=True)
    say(context, f"  check_product.py rows {os.path.relpath(product, context.repo)}：{completed.stdout.strip()}")
    if completed.returncode != 0:
        raise Stopped(f"判法不过：check_product.py rows {os.path.relpath(product, context.repo)}")


class PeerSetup:
    """这一趟发给别的台的那一份（快照、预检、单台配置）只做一次。"""

    def __init__(self, context: Context, hosts: list[Host], values: dict[str, str], snapshot: tuple[str, str]) -> None:
        self.context, self.hosts, self.values, self.snapshot = context, {host.number: host for host in hosts}, values, snapshot
        self.kit: str | None = None
        self.configurations: dict[int, str | None] = {}

    def ready(self, number: int, slots: int, threads: int) -> None:
        if number == 1 or number in self.configurations:
            return
        if self.kit is None:
            self.kit = build_peer_kit(self.context, expected_description(self.context, self.snapshot))
        configuration = configuration_for(self.context, self.hosts[number], self.values)
        for line in preflight_peer(self.context, self.kit, self.hosts[number], slots, threads, configuration):
            say(self.context, f"  预检 {host_name(number)}：{line}")
        if STOP.is_set():
            raise Interrupted()
        self.configurations[number] = configuration


def phase_rows_and_checks(context: Context, hosts: list[Host], allocations: list[HostAllocation], setup: PeerSetup,
                          expected_hashes: dict[str, str]) -> None:
    """第 11、12 步与跨机对照 X 同一个队列；跑完合成三份产物、判 check_product.py rows、做跨机对照。"""
    multi = len(hosts) > 1
    keys = ("s11", "s12", "sx") if multi else ("s11", "s12")
    if broken("resume-reruns-finished"):
        for key in keys:
            for row in STEP_ROWS[key]:
                shutil.rmtree(os.path.join(context.step_root(STEPS[key]), f"row-{row}"), ignore_errors=True)
    seconds = row_seconds(context)
    jobs = [Job(key, (row,), seconds[(key, row)]) for key in keys if not os.path.exists(context.product(STEPS[key].stem))
            for row in STEP_ROWS[key] if not is_done(context, STEPS[key], row)]
    ledger = read_ledger(context)
    row_zero_host = host_of(context, ledger, "s12", 0) if is_done(context, STEPS["s12"], 0) else None
    records: list[dict] = []
    if jobs:
        if sum(allocation.slots for allocation in allocations) == 0:
            raise RunCondition("此刻各台都放不下一条线道（每槽要 5 线程、slice 里 6G（别的台 10G）、实占 10 GiB）")
        placements, deferred = simulate(allocations, jobs, row_zero_host)
        say(context, f"  这一趟要跑 {len(jobs)} 行（已做完的不重跑）：" + " ".join(f"{job.step}:{rows_text(job.rows)}" for job in queue_order(jobs)))
        for line in prediction_text(allocations, placements, deferred):
            say(context, "  " + line)
        for allocation in allocations:
            if allocation.slots > 0:
                setup.ready(allocation.number, allocation.slots, allocation.threads_per_slot)
        records, deferred, failures = run_queue(context, allocations, jobs, row_zero_host, expected_hashes, setup.kit, setup.configurations)
        if failures:
            raise Stopped("这几行没跑成：" + "；".join(failures) + "。做完的行都留着，同一条命令再起只补没做完的")
    for key in ("s11", "s12"):
        if not os.path.exists(context.product(STEPS[key].stem)):
            missing = [row for row in STEP_ROWS[key] if not is_done(context, STEPS[key], row)]
            if missing:
                raise Stopped(f"第 {STEPS[key].number} 步还有行没做完：{rows_text(missing)}")
            merge_step(context, STEPS[key], STEP_ROWS[key], records, setup.snapshot, expected_hashes)
        check_rows(context, context.product(STEPS[key].stem))
    cross_host_check(context, multi, records, setup.snapshot, expected_hashes)
    say(context, "✓ 第 11 步绿（多机：各台分片合成、check_product.py rows 过、跨机一致对照没停）")
    say(context, "✓ 第 12 步绿（同上）")


def row_line(product: str, row: int) -> str:
    lines = [line.rstrip("\n") for line in open(product, encoding="utf-8") if line.startswith("row ") and key_values(line).get("row_number") == str(row)]
    if len(lines) != 1:
        raise Stopped(f"{os.path.basename(product)} 里第 {row} 行有 {len(lines)} 条")
    return lines[0]


def cross_host_check(context: Context, multi: bool, records: list[dict], snapshot: tuple[str, str], expected_hashes: dict[str, str]) -> None:
    path = context.product(CROSS_HOST_CHECK_STEM)
    if not multi:
        text, _ = cross_host_product(None, None, None, "只有一台（双机开关没开或清单里只有本机）")
        install(context, path, text)
        say(context, f"  跨机一致对照：本次未跑（只有一台），产物 {os.path.relpath(path, context.repo)}")
        return
    sx = STEPS["sx"]
    if not os.path.exists(context.product(sx.stem)):
        if not is_done(context, sx, 0):
            raise RunCondition("跨机对照 X（第 12 步 row-0 在另一台再跑一次）这一趟放不下：row-0 那一台之外此刻没有槽。资源空出来后同一条命令再起只补这一行")
        merge_step(context, sx, STEP_ROWS["sx"], records, snapshot, expected_hashes)
    ledger = read_ledger(context)
    host_a, host_b = host_of(context, ledger, "s12", 0), host_of(context, ledger, "sx", 0)
    if host_a == host_b:
        raise Stopped(f"跨机对照的两次都在{host_name(host_a)}（账 row-hosts.tsv）：对照没做成")
    parts = cross_host_parts(row_line(context.product("rows-driver"), 0), os.path.join(context.step_root(STEPS["s12"]), "row-0"),
                             row_line(context.product(sx.stem), 0), os.path.join(context.step_root(sx), "row-0"))
    text, stop = cross_host_product(parts, host_a, host_b, "")
    install(context, path, text)
    say(context, f"  跨机一致对照（{host_name(host_a)} 对 {host_name(host_b)}）：" + "；".join(f"{part}={'同' if same else '不同'}" for part, same, _ in parts))
    if stop:
        raise Stopped(f"跨机一致对照不同（停机，不作废、不当结果；见 {os.path.relpath(path, context.repo)}）")


def phase_timeout_reruns(context: Context, hosts: list[Host], setup: PeerSetup, expected_hashes: dict[str, str]) -> None:
    """第 13 步（原规则）：第 11、12 步里退 253 的行单线道再跑一次，限时再乘 3；带探针的先、不带的后；台照 single_lane_host 挑，线程取那台的可用。"""
    timed = {"r11": timed_out_rows(context.product("rows-probe-driver")), "r12": timed_out_rows(context.product("rows-driver"))}
    say(context, f"  超时的行：带探针 {rows_text(timed['r11']) or 'none'}；不带探针 {rows_text(timed['r12']) or 'none'}")
    for key in ("r11", "r12"):
        step, rows = STEPS[key], timed[key]
        if not rows or os.path.exists(context.product(step.stem)):
            continue
        remaining = tuple(row for row in rows if not is_done(context, step, row))
        records: list[dict] = []
        if remaining:
            resources = query_resources(context, hosts)
            allocations = allocate(hosts, resources)
            for line in allocation_text(allocations, resources):
                say(context, "  " + line)
            chosen = single_lane_host(allocations)
            single = HostAllocation(chosen.number, chosen.threads, chosen.room_bytes, 1, chosen.threads)
            say(context, f"  第 13 步 {key} 第 {rows_text(remaining)} 行 → {host_name(chosen.number)}，单线道 {chosen.threads} 线程（限时再乘 3 由驱动现算）")
            setup.ready(chosen.number, 1, chosen.threads)
            records, deferred, failures = run_queue(context, [single], [Job(key, remaining, 0.0)], None, expected_hashes, setup.kit, setup.configurations)
            if failures or deferred:
                raise Stopped("第 13 步没跑成：" + "；".join(failures or ["放不下"]))
        merge_step(context, step, rows, records, setup.snapshot, expected_hashes)
    say(context, "✓ 第 13 步绿")


def run(context: Context, plan_only: bool) -> int:
    for signal_number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(signal_number, on_signal)
    snapshot_check(context)
    hosts, values, note = load_hosts(context)
    if plan_only:
        if note:
            print(note)
        resources = query_resources(context, hosts)
        allocations = allocate(hosts, resources)
        seconds = row_seconds(context)
        keys = ("s11", "s12", "sx") if len(hosts) > 1 else ("s11", "s12")
        jobs = [Job(key, (row,), seconds[(key, row)]) for key in keys for row in STEP_ROWS[key] if not is_done(context, STEPS[key], row)]
        placements, deferred = simulate(allocations, jobs, None)
        for line in allocation_text(allocations, resources) + prediction_text(allocations, placements, deferred):
            print(line)
        print("每行推的用时（秒）：" + " ".join(f"{key}:{row}={value:.0f}" for (key, row), value in sorted(seconds.items())))
        return 0
    context.attempt_dir = next_attempt_directory(context)
    say(context, f"多机编排开跑：{os.path.basename(context.attempt_dir)}，行输出根 {os.path.relpath(context.rows_root, context.repo)}")
    if note:
        say(context, "  " + note)
    for record in shard_records(context):
        if record["kind"] == "peer" and record["attempt"] != os.path.basename(context.attempt_dir):
            harvested = harvest_fetch(context, record)
            if harvested:
                say(context, f"  收进以前取回的行：{record['label']}（{record['attempt']}）第 {rows_text(harvested)} 行")
    resources = query_resources(context, hosts)
    allocations = allocate(hosts, resources)
    for line in allocation_text(allocations, resources):
        say(context, "  " + line)
    expected_hashes = driver_hashes(context)
    setup = PeerSetup(context, hosts, values, build_snapshot_staging(context))
    phase_rows_and_checks(context, hosts, allocations, setup, expected_hashes)
    phase_timeout_reruns(context, hosts, setup, expected_hashes)
    snapshot_check(context)
    return 0


def context_from_arguments(arguments: argparse.Namespace) -> Context:
    environment = os.environ
    driver_override = environment.get("ROWS_ORCHESTRATOR_DRIVER", "")
    research_root = os.path.abspath(arguments.research_root or arguments.repo)
    return Context(
        repo=os.path.abspath(arguments.repo), research_root=research_root, prefix=arguments.prefix, suffix=arguments.suffix, tag=arguments.resume_tag,
        dumps=arguments.dumps, tree=arguments.tree, snapshot_digest=arguments.snapshot_digest, baseline_timing=arguments.baseline_timing,
        check=os.path.abspath(arguments.check), logs=os.path.abspath(arguments.logs), work_root=os.path.abspath(arguments.work_root),
        driver_path=os.path.abspath(driver_override) if driver_override else os.path.join(research_root, RESEARCH_RELATIVE, DRIVER_NAME),
        driver_overridden=bool(driver_override),
        multi_host_run=environment.get("ROWS_ORCHESTRATOR_MULTI_HOST_RUN") or os.path.join(os.path.abspath(arguments.repo), "scripts/hosts/multi-host-run.sh"),
        hosts_command=environment.get("ROWS_ORCHESTRATOR_HOSTS_COMMAND") or "bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts",
        resources_command=environment.get("ROWS_ORCHESTRATOR_RESOURCES_COMMAND") or "python3 scripts/hosts/host-resources.py --json",
        poll_seconds=float(environment.get("ROWS_ORCHESTRATOR_POLL_SECONDS") or 10),
        peer_compile_check=environment.get("ROWS_ORCHESTRATOR_PEER_COMPILE_CHECK", "1") != "0")


def main(argument_list: list[str]) -> int:
    if os.environ.get("ROWS_ORCHESTRATOR_BREAK") and os.environ.get("ROWS_ORCHESTRATOR_SELFTEST") != "1":
        print("✗ ROWS_ORCHESTRATOR_BREAK 只在 --selftest 起的子进程里认；平常跑时设了它就拒（退 2）", file=sys.stderr)
        return 2
    if argument_list[:1] == ["--selftest"]:
        return selftest(argument_list[1:])
    parser = argparse.ArgumentParser(description="E171 第一段第 11–13 步的多机编排")
    parser.add_argument("mode", choices=("run", "plan"))
    for name in ("--repo", "--prefix", "--suffix", "--resume-tag", "--dumps", "--tree", "--snapshot-digest", "--baseline-timing", "--check", "--logs",
                 "--work-root"):
        parser.add_argument(name, required=True)
    parser.add_argument("--research-root")
    arguments = parser.parse_args(argument_list)
    if not arguments.resume_tag:
        parser.error("--resume-tag 要给（新的一趟的行输出根与产物名接它，不与上一趟撞名）")
    if os.path.realpath(arguments.check) != os.path.realpath(CHECK_PRODUCT):
        parser.error(f"--check 只认这一份旁边的 check_product.py（{CHECK_PRODUCT}），给的是 {arguments.check}")
    context = context_from_arguments(arguments)
    try:
        return run(context, arguments.mode == "plan")
    except RunCondition as error:
        say(context, f"✗ 运行条件不满足（退 78）：{error}")
        return EXIT_RUN_CONDITION
    except Stopped as error:
        say(context, f"✗ 停：{error}")
        return 1
    except Interrupted:
        say(context, "收到停止信号：在跑的各行已停、别的台已做完的行已取回收进来；同一条命令再起就是续跑")
        return EXIT_STOPPED_BY_SIGNAL


# ── 自证 ──
REAL_REPOSITORY = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))  # 本文件在 scripts/experiments/<装置>/ 下
FAKES = HERE  # 假驱动 fake_*.py / fake_*.sh 与本文件同目录


def resources_entry(number: int, threads: int, available_gib: float, room_gib: float, read: bool = True) -> dict:
    entry = {"number": number, "read": read}
    if read:
        entry.update(cpu={"available_threads": threads}, memory={"available": int(available_gib * GIB), "slice_room": int(room_gib * GIB)})
    return entry


def two_hosts(peer_cap: str = "50G") -> list[Host]:
    return [Host(1, "", "", "", "", False), Host(2, "fakepeer", "/fake/peer", "/fake/cargo", peer_cap, True)]


def cells_allocation() -> list[tuple[str, bool, str]]:
    results = []
    today_found, _ = resources_from_json({"hosts": [resources_entry(1, 28, 19.5, 3.1), resources_entry(2, 32, 58.1, 42.5)]}, two_hosts())
    today = [(item.slots, item.threads_per_slot) for item in allocate(two_hosts(), today_found)]
    results.append(("allocation-today", today == [(0, 0), (4, 7)], f"本机/第二台 (槽, 每槽线程) = {today}，要 [(0, 0), (4, 7)]"))
    bound_found, _ = resources_from_json({"hosts": [resources_entry(1, 31, 25, 40)]}, two_hosts()[:1])
    bound = [(item.slots, item.threads_per_slot) for item in allocate(two_hosts()[:1], bound_found)]
    results.append(("allocation-memory-bound", bound == [(2, 15)], f"31 线程、额度 25 GiB → {bound}，要 [(2, 15)]"))
    capped_found, _ = resources_from_json({"hosts": [resources_entry(1, 64, 200, 200), resources_entry(2, 64, 200, 200)]}, two_hosts("30G"))
    capped = [(item.threads, item.slots, item.threads_per_slot) for item in allocate(two_hosts("30G"), capped_found)]
    results.append(("allocation-caps", capped == [(31, 6, 5), (31, 3, 10)],
                    f"64 线程取 31；第二台配置上限 30G → {capped}，要 [(31, 6, 5), (31, 3, 10)]"))
    account_found, _ = resources_from_json({"hosts": [resources_entry(1, 31, 100, 30), resources_entry(2, 31, 100, 30)]}, two_hosts())
    account = [(item.slots, item.threads_per_slot) for item in allocate(two_hosts(), account_found)]
    results.append(("allocation-wrapper-account", account == [(5, 6), (3, 10)],
                    f"slice_room 30 GiB：本机每槽 6G、第二台每槽 10G（多外层 4G）→ {account}，要 [(5, 6), (3, 10)]"))
    chosen = single_lane_host(allocate(two_hosts("30G"), capped_found))
    results.append(("allocation-single-lane", (chosen.number, chosen.threads) == (1, 31), f"第 13 步挑 {chosen.number} 台 {chosen.threads} 线程，要本机 31"))
    unread = resources_from_json({"hosts": [resources_entry(1, 8, 20, 20), resources_entry(2, 0, 0, 0, read=False)]}, two_hosts())[1]
    results.append(("resources-unread", len(unread) == 1 and "第二台" in unread[0], f"读不到的台：{unread}"))
    return results


def cells_derived_configuration(scratch: str) -> list[tuple[str, bool, str]]:
    """清单里不是 PEER_SSH_HOST 的那一台：--emit-hosts 解得出它，另写的单台配置经真的配置判法（--emit-hosts，不连机器）读出来 PEER_* 正是那一台的。"""
    text = ("PEER_SSH_HOST=fakepeer\nPEER_REPOSITORY_DIRECTORY=/fake/peer\nPEER_CARGO_BIN_DIRECTORY=/fake/cargo\nPEER_MEMORY_CAP=50G\n"
            "QUIESCE_STOP_COMMAND=''\nQUIESCE_STOPPED_CHECK_COMMAND=''\nQUIESCE_START_COMMAND=''\nQUIESCE_STARTED_CHECK_COMMAND=''\nENABLE_ACROSS_MACHINES=1\n"
            "ENABLE_GPU=0\nMULTI_HOST_COUNT=2\nMULTI_HOST_2_SSH_HOST=fakepeer\nMULTI_HOST_2_REPOSITORY_DIRECTORY=/fake/peer\n"
            "MULTI_HOST_2_CARGO_BIN_DIRECTORY=/fake/cargo\nMULTI_HOST_2_MEMORY_CAP=50G\nMULTI_HOST_2_IS_PEER=1\nMULTI_HOST_3_SSH_HOST=third.host\n"
            "MULTI_HOST_3_REPOSITORY_DIRECTORY=/third/repo\nMULTI_HOST_3_CARGO_BIN_DIRECTORY=/third/cargo\nMULTI_HOST_3_MEMORY_CAP=''\nMULTI_HOST_3_IS_PEER=0\n")
    hosts, values, _ = hosts_from_configuration(0, text)
    attempt = os.path.join(scratch, "derived")
    os.makedirs(attempt)
    path = configuration_for(dataclasses.replace(Context(*([""] * 8), "", "", "", "", "", False, "", "", "", 1.0, False), attempt_dir=attempt), hosts[2], values)
    completed = subprocess.run(["bash", os.path.join(REAL_REPOSITORY, "scripts/crash/crash-point-replay-shard-configuration-check.sh"), "--emit-hosts", REAL_REPOSITORY],
                               capture_output=True, text=True, env=dict(os.environ, SINGLEFS_MULTI_HOST_CONFIG=path))
    emitted = parse_assignments(completed.stdout)
    good = (len(hosts) == 3 and not hosts[2].is_peer and completed.returncode == 0 and emitted.get("PEER_SSH_HOST") == "third.host"
            and emitted.get("PEER_REPOSITORY_DIRECTORY") == "/third/repo" and emitted.get("PEER_MEMORY_CAP") == "50G"
            and oct(os.stat(path).st_mode & 0o777) == "0o600" and configuration_for(None, hosts[1], values) is None)
    return [("derived-host-config", good, f"清单 {len(hosts)} 台；判法退 {completed.returncode}，读出 PEER_SSH_HOST={emitted.get('PEER_SSH_HOST')}、"
                                          f"上限 {emitted.get('PEER_MEMORY_CAP')}，权限 {oct(os.stat(path).st_mode & 0o777)}")]


def cells_queue() -> list[tuple[str, bool, str]]:
    seconds = row_seconds_table(1000.0, 800.0, {269: 500.0})
    jobs = [Job(key, (row,), seconds[(key, row)]) for key in ("s11", "s12", "sx") for row in STEP_ROWS[key]]
    order = [f"{job.step}:{job.rows[0]}" for job in queue_order(jobs)]
    good_order = order[:3] == ["s11:242", "s12:0", "sx:0"] and order[-1] == "s11:269" and len(order) == 17
    results = [("queue-order", good_order, f"头三个 {order[:3]}，末一个 {order[-1]}，共 {len(order)}")]
    two = [HostAllocation(1, 10, 20 * GIB, 1, 10), HostAllocation(2, 10, 20 * GIB, 1, 10)]
    placements, deferred = simulate(two, jobs, None)
    hosts_of = {(p.job.step, p.job.rows[0]): p.host for p in placements}
    walls = sorted(max(p.end for p in placements if p.host == number) for number in (1, 2))
    results.append(("simulate-two-hosts", not deferred and hosts_of[("s12", 0)] != hosts_of[("sx", 0)] and walls[1] - walls[0] <= 1120.0,
                    f"row-0 在 {hosts_of[('s12', 0)]}、X 在 {hosts_of[('sx', 0)]}，两台推的挂钟 {walls}，放不下 {len(deferred)}"))
    placements, deferred = simulate(two[:1], jobs, None)
    results.append(("simulate-one-host", [f"{job.step}:{job.rows[0]}" for job in deferred] == ["sx:0"] and len(placements) == 16,
                    f"只有一台有槽：放下 {len(placements)}，放不下 {[job.step for job in deferred]}"))
    return results


def fake_shard(label: str, host: int, rows: tuple[int, ...], limit: str = "46124", digest: str = "d" * 64, extra: str = "") -> Shard:
    product = [f"name=row_time_limit seconds={limit} baseline_seconds=15374.455 source=x-timing.out rerun_of_timed_out_rows=false",
               "freeze_check before=true differences=0 detail=none", f"snapshot_digest={digest} files=9 snapshot_ns=5 rows={len(rows)}"]
    product += [f"row row_number={row} row_name=r{row} lane=0 status=ok device_exit=0 device_ns={row + 1} o3_status=ran" for row in rows]
    product += [f"table lanes=1 threads_per_lane=7 rows={len(rows)} table_wall_ns=10 sum_of_row_ns=3", "freeze_check before=false differences=0 detail=none"]
    if extra:
        product.append(extra)
    timing = [line for row in rows for line in (f"E7TIMING name=row_device row={row} device_exit=0 device_ns=1", f"E7TIMING name=process_wall seconds=1 row{row}")]
    return Shard(label, host, "\n".join(product) + "\n", "\n".join(timing) + "\n")


def merge_refused(shards: list[Shard], rows: tuple[int, ...], snapshot: tuple[str, str] = ("d" * 64, "9")) -> str:
    try:
        merge_products(shards, rows, snapshot)
    except MergeRefused as error:
        return str(error)
    return ""


def cells_merge(scratch: str, check: str) -> list[tuple[str, bool, str]]:
    shards = [fake_shard("a", 2, (42,)), fake_shard("b", 1, (0, 15))]
    product, timing = merge_products(shards, (0, 15, 42), ("d" * 64, "9"))
    lines = product.splitlines()
    rows = [key_values(line)["row_number"] for line in lines if line.startswith("row ")]
    path = os.path.join(scratch, "merged.out")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(product)
    checked = subprocess.run(["python3", "-B", check, "rows", path], capture_output=True, text=True)
    good = (rows == ["0", "15", "42"] and lines[3 + 3] == "table lanes=2 threads_per_lane=7 rows=3 table_wall_ns=10 sum_of_row_ns=6 shards=2"
            and lines[0] == shards[0].product.splitlines()[0] and lines[2].startswith(f"snapshot_digest={'d' * 64} files=9 ") and lines[2].endswith(" rows=3")
            and [line.split()[1] for line in timing.splitlines() if "row_device" in line] == ["name=row_device"] * 3
            and [key_values(line)["row"] for line in timing.splitlines() if "row_device" in line] == ["0", "15", "42"]
            and checked.returncode == 0 and len([line for line in lines if line.startswith("merged_shard ")]) == 2)
    results = [("merge-good", good, f"行 {rows}；check_product.py rows：{checked.stdout.strip()}")]
    cases = [("merge-refuse-time-limit", [fake_shard("a", 1, (0,)), fake_shard("b", 2, (15,), limit="46125")], (0, 15), "①"),
             ("merge-refuse-missing", [fake_shard("a", 1, (0,))], (0, 15), "缺 [15]"),
             ("merge-refuse-duplicate", [fake_shard("a", 1, (0, 15)), fake_shard("b", 2, (15,))], (0, 15), "两个分片"),
             ("merge-refuse-snapshot", [fake_shard("a", 1, (0,)), fake_shard("b", 2, (15,), digest="e" * 64)], (0, 15), "②"),
             ("merge-refuse-unknown-line", [fake_shard("a", 1, (0,), extra="rerun_compile_failed rows=0")], (0,), "⑦")]
    for name, case_shards, case_rows, mark in cases:
        refused = merge_refused(case_shards, case_rows)
        results.append((name, mark in refused, f"拒因：{refused or '没拒'}"))
    bad_timing = Shard("a", 1, fake_shard("a", 1, (0, 15)).product, "E7TIMING name=row_device row=0 device_exit=0 device_ns=1\n")
    refused = merge_refused([bad_timing], (0, 15))
    results.append(("merge-refuse-timing", "⑥" in refused, f"拒因：{refused or '没拒'}"))
    return results


def write_fake_row(directory: str, salt_part: str = "", workers: int = 5) -> str:
    """跨机对照格用的一个行目录（同假驱动的形状），交回它的 row 行；salt_part 往那一样里掺一个不同，耗时与 workers 两边本来就不同。"""
    os.makedirs(directory)
    with open(os.path.join(directory, "device.out"), "w", encoding="utf-8") as handle:
        handle.write("E7RESULT name=row_result row=0" + (" salt=1" if salt_part == "device" else "") + "\n")
        handle.write(f"E7TIMING name=process_wall seconds={workers}.5 exit=0\n")
    for stream in STREAMS:
        os.makedirs(os.path.join(directory, "dump", stream))
        for name in Z1_FILES:
            with open(os.path.join(directory, "dump", stream, name), "w", encoding="utf-8") as handle:
                handle.write(f"{stream} {name}" + (" salt" if salt_part == "dump" and name == "ops.tsv" else "") + "\n")
    with open(os.path.join(directory, "o3.out"), "w", encoding="utf-8") as handle:
        handle.write(f"name=config dumps={directory} workers={workers}\nname=load stream=a load_ns={workers}00 ops=3\n"
                     f"name=verdict_summary strict_not_in=0{' salt=1' if salt_part == 'o3' else ''}\nname=state_time batch=1 workers={workers}\n")
    return (f"row row_number=0 row_name=r0{'_掺' if salt_part == 'row' else ''} lane=0 copy_ns={workers} compile_ns={workers} status=ok "
            f"device_exit=0 device_ns={workers} o3_status=ran o3_ns={workers}")


def cells_cross_host(scratch: str) -> list[tuple[str, bool, str]]:
    base_line = write_fake_row(os.path.join(scratch, "cross-a"), workers=5)
    outcomes = {}
    for part in ("", "device", "dump", "o3", "row"):
        other = write_fake_row(os.path.join(scratch, f"cross-b-{part or 'same'}"), part, workers=7)
        parts = cross_host_parts(base_line, os.path.join(scratch, "cross-a"), other, os.path.join(scratch, f"cross-b-{part or 'same'}"))
        outcomes[part or "same"] = sorted(name for name, same, _ in parts if not same)
    expected = {"same": [], "device": ["device_results"], "dump": ["dump_files"], "o3": ["o3"], "row": ["row_line"]}
    text, stop = cross_host_product(None, None, None, "只有一台")
    good_not_run = not stop and "cross_host_identical=not_run" in text
    return [("cross-host-compare", outcomes == expected, f"各掺一样时不同的那几样：{outcomes}"),
            ("cross-host-not-run", good_not_run, "只有一台时报 not_run、不停机")]


@dataclasses.dataclass
class FakeWorld:
    root: str
    repo: str
    command: list[str]
    environment: dict[str, str]
    prefix: str
    dumps: str


def fake_world(root: str, hosts_code: int = 0, resources: list[dict] | None = None, extra_environment: dict[str, str] | None = None) -> FakeWorld:
    repo = os.path.join(root, "repo")
    results = os.path.join(repo, "research", "results")
    os.makedirs(results)
    tree = os.path.join(repo, "scripts", "experiments", "target", "fake-frozen-tree")
    for relative, text in (("Cargo.toml", "[workspace]\n"), ("Cargo.lock", "# lock\n"), (".cargo/config.toml", "[env]\n"),
                           ("crates/a/src/lib.rs", "pub fn a() {}\n"), ("target/release/build/x.rs", "// 不进快照\n")):
        os.makedirs(os.path.dirname(os.path.join(tree, relative)), exist_ok=True)
        with open(os.path.join(tree, relative), "w", encoding="utf-8") as handle:
            handle.write(text)
    prefix, suffix = "research/results/fake-e171-s1", "-r6-r3"
    for stem, seconds in (("baseline-run1", "100.500"), ("positive-control-o1-crash-judge", "80.000"), ("positive-control-reference-filesystem", "70.000")):
        with open(os.path.join(repo, f"{prefix}-{stem}{suffix}-timing.out"), "w", encoding="utf-8") as handle:
            handle.write(f"E7TIMING name=stream_time stream=a elapsed_ns=1000000000\nE7TIMING name=process_wall seconds={seconds} exit=0\n")
    frozen_reference = "e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out"
    os.symlink(os.path.join(REAL_REPOSITORY, "research", "results", frozen_reference), os.path.join(results, frozen_reference))
    os.makedirs(os.path.join(repo, "scripts", "experiments", "target", "fake-dumps"))
    sys.path.insert(0, os.path.join(REAL_REPOSITORY, RESEARCH_RELATIVE))
    import row_driver  # noqa: E402
    digest = row_driver.snapshot_digest(tree)[0]
    assignments = ("PEER_SSH_HOST=fakepeer\nPEER_REPOSITORY_DIRECTORY=/fake/peer\nPEER_CARGO_BIN_DIRECTORY=/fake/cargo\nPEER_MEMORY_CAP=50G\n"
                   "QUIESCE_STOP_COMMAND=''\nQUIESCE_STOPPED_CHECK_COMMAND=''\nQUIESCE_START_COMMAND=''\nQUIESCE_STARTED_CHECK_COMMAND=''\n"
                   "ENABLE_ACROSS_MACHINES=1\nENABLE_GPU=0\nMULTI_HOST_COUNT=1\nMULTI_HOST_2_SSH_HOST=fakepeer\n"
                   "MULTI_HOST_2_REPOSITORY_DIRECTORY=/fake/peer\nMULTI_HOST_2_CARGO_BIN_DIRECTORY=/fake/cargo\nMULTI_HOST_2_MEMORY_CAP=50G\nMULTI_HOST_2_IS_PEER=1\n")
    with open(os.path.join(root, "hosts.txt"), "w", encoding="utf-8") as handle:
        handle.write(assignments)
    entries = resources or [resources_entry(1, 10, 25, 25), resources_entry(2, 10, 25, 25)]
    with open(os.path.join(root, "resources.json"), "w", encoding="utf-8") as handle:
        json.dump({"hosts": entries}, handle)
    os.makedirs(os.path.join(root, "peer"))
    environment = dict(os.environ, ROWS_ORCHESTRATOR_SELFTEST="1", ROWS_ORCHESTRATOR_DRIVER=os.path.join(FAKES, "fake_row_driver.py"),
                       ROWS_ORCHESTRATOR_MULTI_HOST_RUN=os.path.join(FAKES, "fake_multi_host_run.sh"),
                       ROWS_ORCHESTRATOR_HOSTS_COMMAND=f"cat {shlex.quote(os.path.join(root, 'hosts.txt'))}; exit {hosts_code}",
                       ROWS_ORCHESTRATOR_RESOURCES_COMMAND=f"cat {shlex.quote(os.path.join(root, 'resources.json'))}",
                       ROWS_ORCHESTRATOR_POLL_SECONDS="0.05", ROWS_ORCHESTRATOR_PEER_COMPILE_CHECK="0",
                       FAKE_RESEARCH_DIRECTORY=os.path.join(REAL_REPOSITORY, RESEARCH_RELATIVE), FAKE_REAL_REPOSITORY=REAL_REPOSITORY,
                       FAKE_RESULTS_DIRECTORY=results, FAKE_PEER_ROOT=os.path.join(root, "peer"), FAKE_DRIVER_LOG=os.path.join(root, "driver.log"),
                       FAKE_MHR_LOG=os.path.join(root, "mhr.log"), PYTHONDONTWRITEBYTECODE="1")
    environment.update(extra_environment or {})
    dumps = "scripts/experiments/target/fake-dumps"
    command = ["python3", "-B", os.path.abspath(__file__), "run", "--repo", repo, "--prefix", prefix, f"--suffix={suffix}", "--resume-tag=-selftest",
               "--dumps", dumps, "--tree", "scripts/experiments/target/fake-frozen-tree", "--snapshot-digest", digest,
               "--baseline-timing", f"{prefix}-baseline-run1{suffix}-timing.out", "--check", os.path.join(HERE, "check_product.py"),
               "--logs", os.path.join(root, "logs"), "--work-root", os.path.join(root, "work"), "--research-root", REAL_REPOSITORY]
    return FakeWorld(root, repo, command, environment, prefix + "-", dumps)


def world_run(world: FakeWorld) -> tuple[int, str]:
    completed = subprocess.run(world.command, cwd=world.repo, env=world.environment, capture_output=True, text=True, timeout=180)
    return completed.returncode, completed.stdout + completed.stderr


def world_product(world: FakeWorld, stem: str) -> str:
    return os.path.join(world.repo, f"{world.prefix}{stem}-selftest-r6-r3.out")


def ran_counts(world: FakeWorld) -> dict[tuple[int, str], int]:
    """假驱动真跑过（不是 --skip-finished 跳过）的次数，按 (行, 是不是第 13 步重跑) 数；第 12 步 row-0 与 X 都是 row-0，合起来该是 2。"""
    counts: dict[tuple[int, str], int] = {}
    path = os.path.join(world.root, "driver.log")
    for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
        if line.startswith("ran "):
            fields = key_values(line)
            key = (int(fields["row"]), fields["rerun"])
            counts[key] = counts.get(key, 0) + 1
    return counts


def verdict_of(path: str) -> str:
    if not os.path.exists(path):
        return "产物不在"
    return [line for line in open(path, encoding="utf-8") if " name=verdict " in line][-1].strip()


def cells_end_to_end(scratch: str, wanted: set[str] | None) -> list[tuple[str, bool, str]]:
    """端到端各格；wanted 给了就只跑点名的那几格（弄坏开关那一轮只跑一格）。"""
    def want(name: str) -> bool:
        return wanted is None or name in wanted
    results = []
    if want("e2e-two-hosts"):
        results.append(two_hosts_cell(scratch))
    if want("e2e-resume"):
        results.append(resume_cell(scratch))
    if want("e2e-cross-host-mismatch"):
        results.append(mismatch_cell(scratch))
    if want("e2e-unreachable") or want("e2e-config-refused") or want("e2e-switch-off") or want("e2e-preflight-refused") or want("e2e-product-collision"):
        results += refusal_cells(scratch)
    return results


def two_hosts_cell(scratch: str) -> tuple[str, bool, str]:
    world = fake_world(os.path.join(scratch, "two-hosts"), extra_environment={"FAKE_DRIVER_TIMEOUT_ROWS": "15"})
    code, output = world_run(world)
    counts = ran_counts(world)
    expected_runs = expected_ran_counts() | {(15, "true"): 1}
    s12 = world_product(world, "rows-driver")
    s12_rows = [key_values(line)["row_number"] for line in open(s12, encoding="utf-8") if line.startswith("row ")] if os.path.exists(s12) else []
    ledger = open(os.path.join(world.repo, world.dumps, "segment1-rows-selftest-shards", "row-hosts.tsv"), encoding="utf-8").read() \
        if os.path.exists(os.path.join(world.repo, world.dumps, "segment1-rows-selftest-shards", "row-hosts.tsv")) else ""
    good = (code == 0 and counts == expected_runs
            and s12_rows == [str(row) for row in sorted(STEP_ROWS["s12"])] and "fetched" in ledger
            and "cross_host_identical=true" in verdict_of(world_product(world, CROSS_HOST_CHECK_STEM))
            and os.path.exists(world_product(world, "rows-timeout-rerun-driver")) and not os.path.exists(world_product(world, "rows-probe-timeout-rerun-driver")))
    return ("e2e-two-hosts", good, f"退 {code}；真跑过的 (行, 重跑) 与次数和预期{'相同' if counts == expected_runs else '不同：' + str(counts)}；第 12 步产物行 {len(s12_rows)}；"
                                           f"跨机 {verdict_of(world_product(world, CROSS_HOST_CHECK_STEM))[:80]}；输出末行 {output.strip().splitlines()[-1:]}")


def mismatch_cell(scratch: str) -> tuple[str, bool, str]:
    mismatch = fake_world(os.path.join(scratch, "mismatch"), extra_environment={"FAKE_DRIVER_SALT_HOST": "2", "FAKE_DRIVER_SALT_PART": "device"})
    code, output = world_run(mismatch)
    verdict = verdict_of(world_product(mismatch, CROSS_HOST_CHECK_STEM))
    return ("e2e-cross-host-mismatch", code == 1 and "cross_host_identical=false" in verdict and "differing_parts=device_results" in verdict
            and not os.path.exists(world_product(mismatch, "rows-timeout-rerun-driver")), f"退 {code}；{verdict[:120]}")


def refusal_cells(scratch: str) -> list[tuple[str, bool, str]]:
    results = []
    unreachable = fake_world(os.path.join(scratch, "unreachable"), resources=[resources_entry(1, 10, 25, 25), resources_entry(2, 0, 0, 0, read=False)])
    code, output = world_run(unreachable)
    results.append(("e2e-unreachable", code == EXIT_RUN_CONDITION and "没读到资源" in output, f"退 {code}；{output.strip().splitlines()[-1:]}"))
    refused = fake_world(os.path.join(scratch, "config-refused"), hosts_code=1)
    code, output = world_run(refused)
    results.append(("e2e-config-refused", code == EXIT_RUN_CONDITION and "多机配置判不过" in output, f"退 {code}"))
    single = fake_world(os.path.join(scratch, "switch-off"), hosts_code=3, resources=[resources_entry(1, 10, 25, 25)])
    code, output = world_run(single)
    results.append(("e2e-switch-off", code == 0 and "cross_host_identical=not_run" in verdict_of(world_product(single, CROSS_HOST_CHECK_STEM))
                    and not os.path.exists(world_product(single, "rows-cross-host-row-0-driver")) and "只用本机" in output, f"退 {code}"))
    preflight = fake_world(os.path.join(scratch, "preflight-refused"), extra_environment={"FAKE_MHR_EXIT_FOR_LABEL": "preflight:255"})
    code, output = world_run(preflight)
    results.append(("e2e-preflight-refused", code == EXIT_RUN_CONDITION and "预检没过" in output and not os.path.exists(os.path.join(preflight.root, "driver.log")),
                    f"退 {code}；一行都没起：{not os.path.exists(os.path.join(preflight.root, 'driver.log'))}"))
    collision = fake_world(os.path.join(scratch, "collision"))
    with open(os.path.join(collision.repo, f"{collision.prefix}rows-probe-driver-selftest-r6-r3-timing.out"), "w", encoding="utf-8") as handle:
        handle.write("早先的一份\n")
    code, output = world_run(collision)
    results.append(("e2e-product-collision", code == 1 and "产物已存在且与这一次合成的不同" in output, f"退 {code}"))
    return results


def expected_ran_counts() -> dict[tuple[int, str], int]:
    counts = {(row, "false"): 1 for key in ("s11", "s12") for row in STEP_ROWS[key]}
    counts[(0, "false")] = 2
    return counts


def resume_cell(scratch: str) -> tuple[str, bool, str]:
    """跑到一半收 TERM（退 143），同一条命令再起（退 0）：每个 (根, 行) 只真跑过一次；第二台已做完的行在 TERM 时取回、续跑时收进来。"""
    world = fake_world(os.path.join(scratch, "resume"), extra_environment={"FAKE_DRIVER_ROW_SECONDS": "0.4"})
    process = subprocess.Popen(world.command, cwd=world.repo, env=world.environment, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, text=True)
    deadline = time.monotonic() + 120
    while time.monotonic() < deadline:
        lines = open(os.path.join(world.root, "driver.log"), encoding="utf-8").read().splitlines() if os.path.exists(os.path.join(world.root, "driver.log")) else []
        if sum(line.startswith("ran ") and " host=2 " in line + " " for line in lines) >= 2 and sum(line.startswith("ran ") for line in lines) >= 4:
            break
        time.sleep(0.05)
    process.send_signal(signal.SIGTERM)
    first_output, _ = process.communicate(timeout=120)
    first = process.returncode
    before = ran_counts(world)
    world.environment["FAKE_DRIVER_ROW_SECONDS"] = "0"
    second, output = world_run(world)
    counts = ran_counts(world)
    fetched = "fetched" in open(os.path.join(world.repo, world.dumps, "segment1-rows-selftest-shards", "row-hosts.tsv"), encoding="utf-8").read()
    good = first == EXIT_STOPPED_BY_SIGNAL and second == 0 and counts == expected_ran_counts() and sum(before.values()) >= 4 and fetched
    return ("e2e-resume", good, f"第一次退 {first}（停前做完 {sum(before.values())} 行），续跑退 {second}；真跑过的次数与预期{'相同' if counts == expected_ran_counts() else '不同：' + str(counts)}；"
                               f"取回过第二台的行 {fetched}；续跑末行 {output.strip().splitlines()[-1:]}")


SWITCH_CELLS = {"allocation-ignores-memory": "allocation-memory-bound", "merge-allows-duplicates": "merge-refuse-duplicate",
                "merge-ignores-time-limit": "merge-refuse-time-limit", "cross-host-ignores-dumps": "cross-host-compare",
                "resume-reruns-finished": "e2e-resume", "unreachable-dropped": "e2e-unreachable", "harvest-skipped": "e2e-two-hosts"}
CELL_GROUPS = {"allocation": cells_allocation, "queue": cells_queue}


def selftest(argument_list: list[str]) -> int:
    wanted = set(argument_list[argument_list.index("--cells") + 1].split(",")) if "--cells" in argument_list else None
    scratch = tempfile.mkdtemp(prefix="e171-rows-orchestrator-selftest-")
    try:
        results = (cells_allocation() + cells_derived_configuration(scratch) + cells_queue() + cells_merge(scratch, os.path.join(HERE, "check_product.py"))
                   + cells_cross_host(scratch))
        if wanted is None or any(name.startswith("e2e-") for name in wanted):
            results += cells_end_to_end(scratch, wanted)
    finally:
        shutil.rmtree(scratch, ignore_errors=True)
    red = 0
    for name, ok, detail in results:
        if wanted is not None and name not in wanted:
            continue
        red += 0 if ok else 1
        print(f"{'✓' if ok else '✗'} 自证 {name}：{detail}", flush=True)  # gate-lint:detail
    if wanted is None and not os.environ.get("ROWS_ORCHESTRATOR_BREAK"):
        for switch, cell in SWITCH_CELLS.items():
            completed = subprocess.run(["python3", "-B", os.path.abspath(__file__), "--selftest", "--cells", cell], capture_output=True, text=True,
                                       env=dict(os.environ, ROWS_ORCHESTRATOR_BREAK=switch, ROWS_ORCHESTRATOR_SELFTEST="1"), timeout=600)
            caught = completed.returncode == 1 and f"✗ 自证 {cell}：" in completed.stdout
            red += 0 if caught else 1
            print(f"{'✓' if caught else '✗'} 弄坏开关 {switch} → 红在 {cell}：{'红了' if caught else '没红（退 ' + str(completed.returncode) + '）'}", flush=True)
    counted = len([1 for name, _, _ in results if wanted is None or name in wanted]) + (len(SWITCH_CELLS) if wanted is None and not os.environ.get("ROWS_ORCHESTRATOR_BREAK") else 0)
    print(f"自证 {counted} 项，红 {red}")
    return 1 if red else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
