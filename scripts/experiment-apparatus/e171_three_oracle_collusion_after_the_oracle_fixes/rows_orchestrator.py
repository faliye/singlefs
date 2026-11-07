#!/usr/bin/env python3
"""E171 第 1 次重跑第一段·修订 6 树：第 11–13 步的多机编排（用户 2026-10-06 定案：「分片 双机62核 给80g」；「长时间重型任务。分片 断点重启 这都不用我说。
自适应多机跑」）。由 run_segment_one_on_the_revision_6_tree_multi_host.sh 在前提与第 0–10 步的续跑核对都放行之后起，在仓根跑。
判法（check_product.py rows）、阈值、限时规则（驱动照 --baseline-timing 现算）与每行驱动的参数照原跑法 run_segment_one_on_the_revision_6_tree.sh
（sha256 869d2c0a…）第 11–13 步；改的只有 --lanes、--threads、--rows、输出根与工作根（驱动按键复用，同一输出根里同键做完的行不重做，不另给续跑开关）。

用法：
  rows_orchestrator.py run  --repo R --prefix P --suffix=S --resume-tag=T --dumps D --tree TREE --snapshot-digest H --baseline-timing B
                            --check C --logs L --work-root W [--research-root RR]
      跑第 11–13 步。被停、被杀之后同一条命令再起就是续跑：已做完的行（本机的、已收进来的别的台的）不重跑，写到一半的重做，资源与分配重新现查现算。
  rows_orchestrator.py plan <同上>
      只现查机器与资源、算分配、按每行推的用时推各台挂钟，打出来；一行都不起，不写产物。
  rows_orchestrator.py --selftest [--cells 格,…]
      假驱动、假 task-server 分片（fake_task_server.py 演计划、各片、合并三步）、假机器清单与资源走一遍各格（同目录的 fake_*），末尾逐个打开弄坏开关核它红在点名的格。

退出码：0 跑完（第 11–13 步都绿）；1 停机或某一行没跑成（日志与 sequence.log 写停在哪、为什么）；2 用法错；
  78 运行条件不满足（多机配置判不过、计划分的片没都起来或都落在同一台、别的台的预检没过（工具链、Python、离线编译、盘）、此刻一条线道都放不下、
  跨机对照那一行此刻放不下）——不悄悄退回单机，出路是修好那台或改配置，再用同一条命令起；143 收到 TERM / INT / HUP，收完尾退出。

机器（下面「经 task-server」）：台号照服务给每片的 TASK_SERVER_SHARD_HOST 定（多机配置 HOSTS 里那一台的地址）：与编排那一片的地址相同的是第 1 台（本机），不同的是第 2 台（第二台）；两台对称，不认 local、peer 这类角色名；日志与产物里台名只写「本机」「第二台」，
  不写别名与目录。计划那一步照 scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts 解出的清单定片数（判法退 3，即 ENABLE_ACROSS_MACHINES 没开，
  只分一片、打一行；退 1 退 78）；服务只分本机与第二台两种，清单里多出来的台用不上，片数至多 2。
  编排在片号 0 那一片里跑（下称编排那一片），它所在那一台的行直接起；别的台的行不自己连那一台，经 shard_relay.py 写进共享的计划目录，
  交给服务派到那一台的那一片代跑（shard_relay.py 文件头）。代跑的命令是研究目录里的 peer_side.py：仓与工作根经 NFS 两台挂在同一绝对路径，
  它在这一份所在那棵树的根（真跑时就是 --repo）、同一个绝对路径上起，读这一趟分片目录里的 peer-kit/expected.json 与冻结树快照 frozen-snapshot/，
  每个分片的产物与行目录直接写进它的分片目录下的 peer-out/（共享根下，编排那一片照同一个路径读）；不送、不取回。
  代跑那一台的编译与逐行的工作树放那一台本地盘（代跑那一片照服务派到第二台那一趟的样子给每条命令的 MULTI_HOST_RUN_PEER_SCRATCH）；
  编排那一片落在第二台时，它自己那几行的工作根放这一片的 MULTI_HOST_RUN_PEER_SCRATCH 下（服务派到第二台时给的），不用 --work-root。
资源与分配（每次起跑、每次续跑都算，写进日志）：每台线程取登记的每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads，
  读 .claude/heavy-thread-default；SINGLEFS_THREAD_CAP 设了照它），每台都是这一个数，不看此刻忙闲，登记读不出退 78。
  每台一槽，那一槽的线程 = 那一台的每台线程；每台线程为 0 的记 0 槽（E171（三oracle合谋测试）第 2 次重跑登记修订 11 第 11.2 条：线道与第 7 步都取整份额度）。
  内存不在这里算：编排器不查机器的内存额度、不按内存定槽数、不给驱动与第二台的命令套内存上限；整条命令交 task-server，内存额度按记录由服务给，
  撞上限由服务放大重排。读不到的台（连不上）在别的台的预检那一步退 78，不悄悄少用一台。
行与派发：第 11 步 242、269（带探针）、第 12 步空变异 row-0 与 13 行、跨机对照 X（row-0 在另一台再跑一次），同进一个队列，第 11、12 步同时跑；
  每行一个单线道驱动（--lanes 1），哪一槽空了就从队列头取第一个放得下的（X 只放在第 12 步 row-0 那一台之外）；某一行没跑成就不再派新行，
  在跑的跑完再停。队列按每行推的用时从长到短（最长先做，各台挂钟接近）；用时相同的按 row-0、X、其余行号排。每行推的用时：
  以前量过这一行的（导出目录下早先各趟 segment1-rows*/row-<n>/row-result.tsv 里 status=ok、device_exit=0 的 device_ns，取最大）用它；
  没量过的，带探针的取第 7 步 baseline-run1 的 process_wall（T），不带探针的取第 3 步两份带流三的阳性对照（O1-崩溃判官、RefFS）process_wall 的较大者；
  每行另加 120 秒（编译 36 秒、O3 33 秒取自第 269 行早先那一份 row-result.tsv，加拷树，推的）。
  本机每槽一个固定工作根（同一槽上后一行增量编译，驱动自己核）；第 12 步 row-0 与 X 各用新的工作根（两边都冷编译，跨机对照比得了编译那几列之外的一切）。
第 13 步：第 11、12 步合成的产物里 device_exit=253 的行，照原规则单线道、限时再乘 3；带探针的先、不带的后，各一个驱动，驱动参数与第 11、12 步同一步的完全一样、
  输出根也是同一个：驱动按键复用，在这个根里看到同键的上一次结果是超时，就自己把这一行的限时乘 3（重做中途停下，行目录里的标记文件 timed-out-before 还记着）。
  行目录在编排那一台的行输出根里（别的台做完的行已收进来），所以跑在编排那一台（线程取每台线程，登记值），不挑别的台。
产物与合并（写死的合并规则，merge_products）：每一步最后一份原名产物 <PREFIX>-<步名><RESUME_TAG><SUFFIX>.out 与 -timing.out，由「这一趟」的分片产物合成：
  这一趟跑的每行一个分片；更早几趟已做完的行由本机一个驱动从行目录重报一份（同键做完的行按键复用、直接读回，行与 row-result.tsv 逐字相同；
  上一次是超时、编不过或测量管子缺陷的行，驱动照自己的规则重做，不是原样重报）。
  ① 各分片的 name=row_time_limit 行逐字相同，原样留一行，不同就拒；② snapshot_digest 与 files 各分片相同、且等于冻结树快照（别的台读的那一份）的摘要，
  不同就拒；③ row 行按序号排，同一序号出现两次拒，与这一步登记的行集不等（缺、多）拒；④ freeze_check 前后各一行：differences 相加，detail 并起来；
  ⑤ table 行：lanes、rows、sum_of_row_ns 相加，table_wall_ns 取最大，threads_per_lane 写各分片取值去重升序，另加 shards=<分片数>；
  ⑥ 末尾每个分片一行 merged_shard host=<台号> rows=<行> sha256=<分片产物>；计时按 row_device 分块，块按序号排，行集与产物的不等就拒；
  ⑦ 认不得的行拒。原始分片产物、别的台的输出目录与日志留在 <导出目录>/segment1-rows<RESUME_TAG>-shards/attempt-<趟>/ 下。
跨机一致对照（新设，收严）：第 12 步 row-0 与 X 在两台各跑一次，比四样：row 行去掉 copy_ns、compile_ns、device_ns、o3_ns 四个耗时列；
  device.out 的 E7RESULT 行（E7TIMING 不比）；导出里三条流的 oracle-verdicts.tsv、call-outcomes.tsv、writable-mount-probe.tsv、ops.tsv、states.tsv
  （与 check_product.py z1 同一组）；o3.out 去掉 name=config、name=state_time 两种行与键名以 _ns 结尾的字段。有一样不同就停机（退 1，不作废、不当结果）；
  产物 <PREFIX>-rows-cross-host-check<RESUME_TAG><SUFFIX>.out。只有一台时那一格报 not_run。
续跑：别的台那一行被停时代跑那一片先停那边那条命令、再删那一台本地盘上那条命令的目录；已做完的行目录早已写在共享根下那个分片的 peer-out/rows/ 里。
  每个别的台的分片结束时、每次起跑时（以前各趟的分片），把 peer-out/rows/ 里有 row-result.tsv 的行收进本机行输出根（先记账 row-hosts.tsv、记作 peer，
  再按硬链接拷、拷完改名），本机那一份写到一半的行目录删掉重做。分片记录里没有 peer_out 的（不是这一版编排器写的）认不得、停（退 1），换新的 --resume-tag 从头跑。
  代跑那一片的心跳停了 15 分钟（shard_relay.py 的 STALE_SECONDS），那一行的转交退 255、当没跑成（那一行没写完的部分下一趟重做）。

经 task-server（用户：「现在就把分片收进服务」，records/2026-10-30-编译绕过内存总上限.md）：task-server 认得 rows_orchestrator.py run（task-server/task_server_lib.py 的 SHARDED_ENTRIES），
  照环境里的 TASK_SERVER_SHARD_PHASE 跑服务拆的那一步（shard_phase_before_layer；变量的写法在 task-server/task_server_lib.py 文件头「分片」）：
  ① plan：照上面「机器」定片数，每片只写 label（没有 pin 键：不钉台，records/2026-10-31-task-server巡检.md 问题 94；不写内存下限：内存额度由服务按记录给）；
  ② shard：片号 0 是编排那一片：登记、起心跳，等计划里的各片都登记了（至多 REGISTRATION_WAIT_SECONDS 秒，没等齐退 78），台照各片的 TASK_SERVER_SHARD_HOST 定，
     照下面 run 跑整件（第 11–13 步、合并、判法、跨机对照都在这一片），收尾写 finished；别的片照 shard_relay.py 代跑转交来的命令，见到 finished、手上的跑完退 0。
     计划分了两片而两片都落在同一台：跨机对照做不成，退 78（第二台空出来后同一条命令再交），不悄悄退回单机；
  ③ merge：整件已在编排那一片做完，只核编排那一片写了 finished、各片的产物目录在，退 0。
  不经 task-server 直接调 run（各段入口脚本里调的那一下，TASK_SERVER_SHARD_PHASE 没设）：只用这一台，跨机对照报 not_run，原因写「没经 task-server 分片」。

弄坏开关 ROWS_ORCHESTRATOR_BREAK=<项>（逗号连几个；只在 --selftest 起的子进程里认，别处设了退 2）。逐项写开关名、弄坏哪一处，
下一行起写打开后红的格（自证打「✗ 自证 <格名>：…」；开关轮只核每项第一格）：
  ROWS_ORCHESTRATOR_BREAK=slots-split-by-five  照改前按每槽 5 线程分槽（槽数 = ⌊每台线程 / 5⌋），不是每台一槽
      「✗ 自证 allocation-threads：」「✗ 自证 allocation-registered：」
  ROWS_ORCHESTRATOR_BREAK=merge-allows-duplicates  合并时同一序号两行取前一行、不拒
      「✗ 自证 merge-refuse-duplicate：」
  ROWS_ORCHESTRATOR_BREAK=merge-ignores-time-limit  合并时各分片 row_time_limit 不同也取第一行
      「✗ 自证 merge-refuse-time-limit：」
  ROWS_ORCHESTRATOR_BREAK=cross-host-ignores-dumps  跨机对照不比导出那一样
      「✗ 自证 cross-host-compare：」
  ROWS_ORCHESTRATOR_BREAK=resume-reruns-finished  续跑时已做完的行也重跑
      「✗ 自证 e2e-resume：」
  ROWS_ORCHESTRATOR_BREAK=harvest-skipped  别的台做完的行不收进本机行输出根
      「✗ 自证 e2e-two-hosts：」「✗ 自证 e2e-resume：」「✗ 自证 e2e-cross-host-mismatch：」「✗ 自证 e2e-product-collision：」
  ROWS_ORCHESTRATOR_BREAK=plan-writes-memory-limit  plan 计划的那一片又带上一个内存键（片里只许有 label）
      「✗ 自证 task-server-phases：」
  ROWS_ORCHESTRATOR_BREAK=plan-pins-local  plan 计划的那一片照改前钉在本机（pin=local），服务拒
      「✗ 自证 task-server-phases：」
  ROWS_ORCHESTRATOR_BREAK=driver-gets-memory-cap  给逐行驱动的参数又带上 --memory-cap（驱动与假驱动都不认这个参数）
      「✗ 自证 e2e-two-hosts：」
  ROWS_ORCHESTRATOR_BREAK=peer-command-gets-memory-cap  转交给别的台的命令又带上 --memory-cap 外层上限（shard_relay.py 不认、退 2）
      「✗ 自证 e2e-two-hosts：」
"""
from __future__ import annotations

import argparse
import atexit
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
RESEARCH_RELATIVE = "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes"
CHECK_PRODUCT = os.path.join(HERE, "check_product.py")  # 判法脚本：--check 只认这一份（各跑法脚本给的都是研究目录里的它），起它时照字面路径起
DRIVER_NAME = "row_driver.py"
GIB = 1 << 30
HOSTS_DIRECTORY = os.path.join(HERE, "..", "..", "hosts")  # scripts/hosts：每台线程的登记取法 heavy_threads.py 在这里
PEER_DISK_BYTES_PER_SLOT = 3 * GIB
ROW_OVERHEAD_SECONDS = 120
EXIT_STOPPED_BY_SIGNAL = 143
EXIT_RUN_CONDITION = 78
# 别的台那一头：研究目录里的这一份，别的台在这一份所在那棵树的根、照同一个绝对路径起（文件头「机器」；真跑时那棵树就是 --repo）
PEER_SIDE = os.path.join(HERE, "peer_side.py")
PEER_START_DIRECTORY = os.path.abspath(os.path.join(HERE, "..", "..", ".."))
PEER_OUT_DIRECTORY_NAME = "peer-out"  # 别的台一个分片的产物与行目录写在它的分片目录下的这个目录里
SHARD_RELAY = os.path.join(HERE, "shard_relay.py")  # 别的台的命令经它交给落在那一台的那一片代跑（文件头「机器」）
if HERE not in sys.path:
    sys.path.insert(0, HERE)
import shard_relay  # noqa: E402
ORCHESTRATOR_HOST_NUMBER = 1   # 编排那一片所在的台；登记地址与它不同的那一台是 2（文件头「机器」）
MAXIMUM_SHARDS = 2   # 服务只分两台
REGISTRATION_WAIT_SECONDS = 1800.0   # 编排那一片等计划里的各片都登记的上限（文件头「经 task-server」②）
TIMING_COLUMNS = ("copy_ns", "compile_ns", "device_ns", "o3_ns")
STREAMS = ("new_pool_file_creation", "fixed_script_through_e_then_unmount", "first_file_unlinked_then_inodes_across_one_leaf")
Z1_FILES = ("oracle-verdicts.tsv", "call-outcomes.tsv", "writable-mount-probe.tsv", "ops.tsv", "states.tsv")
HOST_NAMES = ("本机", "第二台", "第三台", "第四台", "第五台", "第六台", "第七台", "第八台", "第九台", "第十台")
BREAK_SWITCHES = ("slots-split-by-five", "merge-allows-duplicates", "merge-ignores-time-limit", "cross-host-ignores-dumps",
                  "resume-reruns-finished", "harvest-skipped", "plan-writes-memory-limit", "driver-gets-memory-cap", "peer-command-gets-memory-cap",
                  "plan-pins-local")


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
    rerun: bool = False   # 第 13 步：行输出根与参数同第 11、12 步那一步，只挑上一次超时的行；做完的标志是 row-result.tsv 里 timed_out_before=true


STEPS = {
    "s11": Step("s11", 11, "rows-probe-driver", "", ("--keep-dumps", "--streams", "1,2,3", "--writable-mount-probe")),
    "s12": Step("s12", 12, "rows-driver", "", ("--keep-dumps", "--streams", "1,2,3")),
    "sx": Step("sx", 12, "rows-cross-host-row-0-driver", "-cross-host", ("--keep-dumps", "--streams", "1,2,3")),
    "r11": Step("r11", 13, "rows-probe-timeout-rerun-driver", "", ("--keep-dumps", "--streams", "1,2,3", "--writable-mount-probe"), rerun=True),
    "r12": Step("r12", 13, "rows-timeout-rerun-driver", "", ("--keep-dumps", "--streams", "1,2,3"), rerun=True),
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
    """一个分片驱动除路径之外的参数：行（0 写成 --empty-mutation，同原跑法第 12 步）、原跑法那一步的参数、线道与线程。"""
    numbered = sorted(row for row in rows if row != 0)
    arguments: list[str] = []
    if numbered:
        arguments += ["--rows", ",".join(str(row) for row in numbered)]
    if 0 in rows:
        arguments.append("--empty-mutation")
    memory_arguments = "--memory-cap 6G".split() if broken("driver-gets-memory-cap") else []   # 弄坏开关造的错参数：整句写成一个字面量再拆
    return arguments + list(step.flags) + ["--lanes", str(lanes), "--threads", str(threads)] + memory_arguments


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
            raise RunCondition(f"{host_name(host.number)}在配置里缺 ssh 别名、仓目录或 cargo 目录（HOST_{host.number}_*），服务派不到那一台")
    return hosts, values, ""


@dataclasses.dataclass(frozen=True)
class HostAllocation:
    number: int
    threads: int
    slots: int
    threads_per_slot: int
    terms: str = ""


def registered_host_threads(environment=None) -> int:
    """每台线程（文件头「资源与分配」）：登记的每台默认重活线程数，每台都取这一个数，不看此刻忙闲；登记读不出抛 RunCondition（退 78）。"""
    if HOSTS_DIRECTORY not in sys.path:
        sys.path.insert(0, HOSTS_DIRECTORY)
    import heavy_threads  # noqa: E402（现用现引：只在算分配时要它）
    try:
        return heavy_threads.default_heavy_threads(os.environ if environment is None else environment)
    except heavy_threads.HeavyThreadDefaultError as error:
        raise RunCondition(f"每台线程的登记读不出：{error}（{error.howto}）") from error


def allocate(hosts: list[Host], threads: int | None = None) -> list[HostAllocation]:
    """文件头「资源与分配」：每台线程 = 登记值（threads 不给就现读登记，不看忙闲）；每台一槽，那一槽的线程 = 每台线程；每台线程为 0 的记 0 槽。"""
    threads = registered_host_threads() if threads is None else threads
    allocations = []
    for host in hosts:
        if broken("slots-split-by-five"):
            slots = threads // 5  # 弄坏开关：改前的分法，每槽 5 线程
        else:
            slots = 1 if threads > 0 else 0
        allocations.append(HostAllocation(host.number, threads, slots, threads // slots if slots else 0, f"每台一槽={slots}"))
    return allocations


def single_lane_host(allocations: list[HostAllocation], own_host: int) -> HostAllocation:
    """第 13 步挑台：编排那一台（上一次超时的行目录在它的行输出根里，驱动在同一个根里按键复用才看得见；每台线程都是登记值）。"""
    chosen = [allocation for allocation in allocations if allocation.number == own_host and allocation.slots > 0]
    if not chosen:
        raise RunCondition(f"第 13 步：编排那一台（{host_name(own_host)}）的线程不够一条线道（每台线程是 0）")
    return chosen[0]


def allocation_text(allocations: list[HostAllocation]) -> list[str]:
    return [f"分配 {host_name(allocation.number)}：每台线程 {allocation.threads}（登记值）；槽 {allocation.slots}（{allocation.terms}），每槽线程 {allocation.threads_per_slot}"
            for allocation in allocations]


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


# ── 跑的时候：路径、日志、账、收别的台的行 ──
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
    hosts_command: str
    poll_seconds: float
    peer_compile_check: bool
    attempt_dir: str = ""
    own_host: int = 1   # 编排那一片所在的台（文件头「机器」）；这一台的行直接起，别的台的转交
    relay_plan: str = ""   # 经 task-server 分片时的计划目录（转交的请求写在它下面）
    relay_targets: dict = dataclasses.field(default_factory=dict)   # 别的台号 → 代跑那一片的片号

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


def driver_files(context: Context) -> list[str]:
    """驱动与它在同一目录里引的模块、两张行表的绝对路径（两台相同）。跑的中途改了就停（每行起之前核）。"""
    if context.driver_overridden:
        return [context.driver_path]
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
    return sorted([os.path.join(directory, module + ".py") for module in seen]
                  + [os.path.join(directory, name) for name in os.listdir(directory) if name.endswith(".tsv")])


def driver_hashes(context: Context) -> dict[str, str]:
    return {path: sha256_of_file(path) for path in driver_files(context)}


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


def is_done(context: Context, step, row: int) -> bool:
    """这一步这一行做完了：行目录里有 row-result.tsv。第 13 步（step.rerun）同一个根里本来就有上一次超时的那份，所以另要它记着限时乘过 3（timed_out_before=true）。"""
    path = os.path.join(context.step_root(step), f"row-{row}", "row-result.tsv")
    if not os.path.exists(path):
        return False
    if not step.rerun:
        return True
    with open(path, encoding="utf-8") as handle:
        return "timed_out_before\ttrue\n" in handle.read().splitlines(keepends=True)


def host_of(context: Context, ledger: dict[tuple[str, int], int], step: str, row: int) -> int:
    """账上没有的做完的行是本机驱动直接写进行输出根的（别的台的先记账再拷）。"""
    return ledger.get((step, row), 1)


def link_or_copy(source: str, destination: str) -> None:
    try:
        os.link(source, destination)
    except OSError:
        shutil.copy2(source, destination)


def peer_rows_directory(record: dict) -> str:
    """别的台一个分片写行目录的地方（共享根下它的 peer-out/rows/）；分片记录里没有 peer_out 的不是这一版编排器写的，认不得、停。"""
    if "peer_out" not in record:
        raise Stopped(f"分片记录 {record.get('label')}（{record.get('attempt')}）里没有 peer_out：不是这一版编排器写的，认不得；换一个新的 --resume-tag 从头跑")
    return os.path.join(record["peer_out"], "rows")


def harvest_peer_rows(context: Context, record: dict) -> list[int]:
    """把一个别的台分片的输出目录里有 row-result.tsv 的行收进本机行输出根：先记账、硬链接拷进 <行>.harvest-partial、再改名；本机写到一半的同名行目录删掉。"""
    if broken("harvest-skipped"):
        return []
    rows_directory = peer_rows_directory(record)
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
        append_ledger(context, step.key, row, record["host"], "peer")
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


def snapshot_staging_directory(context: Context) -> str:
    """冻结树快照在这一趟分片目录里的那一份（共享根下，别的台照同一个路径读）。"""
    return os.path.join(context.shards, "frozen-snapshot")


def build_snapshot_staging(context: Context) -> tuple[str, str]:
    """冻结树里驱动要拷的那几样（Cargo.toml、Cargo.lock、.cargo、crates，不带 target）照驱动自己的 sync_tree 拷一份给别的台读；摘要也照驱动的算。"""
    module = row_driver_module(context)
    staging = snapshot_staging_directory(context)
    module.sync_tree(os.path.join(context.repo, context.tree), staging)
    digest, count = module.snapshot_digest(staging)
    return digest, str(count)


def expected_description(context: Context, snapshot: tuple[str, str]) -> dict:
    """别的台那一头要核的（peer_side.py 文件头「expected.json 的键」）：路径都是两台相同的绝对路径。"""
    baseline_timing = os.path.join(context.repo, context.baseline_timing)
    return {"python": f"{sys.version_info.major}.{sys.version_info.minor}", "google_crc32c": has_google_crc32c(),
            "baseline_timing": [baseline_timing, sha256_of_file(baseline_timing)], "files": driver_hashes(context),
            "snapshot": [snapshot_staging_directory(context), snapshot[0], snapshot[1]],
            "research_directory": HERE, "driver": context.driver_path,
            "disk_bytes_per_slot": PEER_DISK_BYTES_PER_SLOT}


def build_peer_kit(context: Context, expected: dict) -> str:
    """这一趟给别的台读的那一份（共享根下）：kit 目录里只有 expected.json；交回 kit 目录。"""
    kit = os.path.join(context.attempt_dir, "peer-kit")
    os.makedirs(kit)
    with open(os.path.join(kit, "expected.json"), "x", encoding="utf-8") as handle:
        json.dump(expected, handle, ensure_ascii=False, indent=1)
    return kit


def driver_environment() -> dict[str, str]:
    return dict(os.environ, PYTHONDONTWRITEBYTECODE="1")


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
    record = {"label": label, "step": step.key, "rows": list(rows), "host": context.own_host, "kind": kind, "product": product, "timing": timing,
              "attempt": os.path.basename(context.attempt_dir), "command": command[5:]}
    write_record(directory, record)
    with open(product, "x", encoding="utf-8") as out, open(os.path.join(directory, "driver.stderr"), "x", encoding="utf-8") as err:
        process = subprocess.Popen(command, cwd=context.repo, stdout=out, stderr=err, env=driver_environment(), start_new_session=True)
    return process, record


def peer_label(context: Context, label: str) -> str:
    return re.sub(r"[^A-Za-z0-9._-]", "-", f"e171s1{context.tag}-{os.path.basename(context.attempt_dir)}-{label}")


def peer_side_arguments(kit: str, mode: str, out: str) -> list[str]:
    """别的台那一头 peer_side.py 的参数：读 kit 里的 expected.json，产物写进共享根下的 out。"""
    return [mode, "--expected", os.path.join(kit, "expected.json"), "--out", out]


def peer_side_invocation() -> list[str]:
    """转交的命令里 peer_side.py 参数之前那一截：研究目录里的 peer_side.py（别的台照同一个绝对路径起）。"""
    return ["env", "PYTHONDONTWRITEBYTECODE=1", "nice", "-n", "19", "python3", "-B", PEER_SIDE]


def peer_command(context: Context, peer_name: str, host: int, inner: list[str], memory_arguments: list[str] | None = None) -> list[str]:
    """在别的台 host 起 peer_side.py 那一条：经 shard_relay.py 交给落在那一台的那一片代跑（文件头「机器」），不自己连那一台。peer_name 是转交的标签；
    memory_arguments 是调用方弄坏开关下多带的参数（不给取这一份的开关 peer-command-gets-memory-cap）。"""
    if host not in context.relay_targets:
        raise RunCondition(f"{host_name(host)}没有代跑的那一片（这一趟没经 task-server 分片，或计划里的那一片没落在那一台）")
    if memory_arguments is None:
        memory_arguments = "--memory-cap 4G".split() if broken("peer-command-gets-memory-cap") else []   # 弄坏开关造的错参数：整句写成一个字面量再拆
    return (["python3", "-B", SHARD_RELAY, "relay", "--plan", context.relay_plan, "--to", str(context.relay_targets[host])] + memory_arguments
            + ["--label", peer_name, "--poll", str(context.poll_seconds), "--"] + peer_side_invocation() + inner)


def launch_peer(context: Context, kit: str, label: str, step: Step, rows: tuple[int, ...], threads: int, host: int) -> tuple[subprocess.Popen, dict]:
    directory = os.path.join(context.attempt_dir, label)
    os.makedirs(directory)
    out = os.path.join(directory, PEER_OUT_DIRECTORY_NAME)
    inner = peer_side_arguments(kit, "run-driver", out) + ["--"] + step_arguments(step, rows, 1, threads)
    command = peer_command(context, peer_label(context, label), host, inner)
    record = {"label": label, "step": step.key, "rows": list(rows), "host": host, "kind": "peer", "peer_out": out,
              "product": os.path.join(out, "product.out"), "timing": os.path.join(out, "timing.out"),
              "attempt": os.path.basename(context.attempt_dir), "command": inner}
    write_record(directory, record)
    with open(os.path.join(directory, "relay.out"), "x", encoding="utf-8") as out_file, \
            open(os.path.join(directory, "relay.err"), "x", encoding="utf-8") as err:
        process = subprocess.Popen(command, cwd=PEER_START_DIRECTORY, stdout=out_file, stderr=err, start_new_session=True)
    return process, record


def preflight_peer(context: Context, kit: str, host: Host, slots: int, threads: int) -> list[str]:
    """别的台的预检（经转交在那一台起）：peer_side.py preflight 核 Python、google_crc32c、rsync、基线计时与驱动的 sha256、
    快照摘要、O3 冻结行、本地盘，--compile 时把冻结快照离线编一遍。没过退 78。"""
    label = f"preflight-h{host.number}"
    directory = os.path.join(context.attempt_dir, label)
    os.makedirs(directory)
    out = os.path.join(directory, PEER_OUT_DIRECTORY_NAME)
    inner = peer_side_arguments(kit, "preflight", out) + ["--slots", str(slots), "--threads", str(threads)]
    if context.peer_compile_check:
        inner.append("--compile")
    command = peer_command(context, peer_label(context, label), host.number, inner)
    with open(os.path.join(directory, "relay.out"), "x", encoding="utf-8") as out_file, \
            open(os.path.join(directory, "relay.err"), "x", encoding="utf-8") as err:
        code = subprocess.run(command, cwd=PEER_START_DIRECTORY, stdout=out_file, stderr=err).returncode
    report_path = os.path.join(out, "preflight.txt")
    report = open(report_path, encoding="utf-8").read().splitlines() if os.path.exists(report_path) else []
    if code != 0:
        tail = open(os.path.join(directory, "relay.err"), encoding="utf-8").read().splitlines()[-4:]
        failed = [line for line in report if " ok=false" in line] or tail
        raise RunCondition(f"{host_name(host.number)}预检没过（转交退 {code}；255 是代跑那一片没了心跳，2 是转交的参数错）："
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
    """一个分片结束：记退出码；别的台的把它输出目录里做完的行收进本机；本机的把做完的行记账。交回跑成没有（退 0、点名的行都有 row-result.tsv、产物在）。"""
    with open(os.path.join(context.attempt_dir, record["label"], "exit"), "w", encoding="utf-8") as handle:
        handle.write(f"{code}\n")
    step = STEPS[record["step"]]
    if record["kind"] == "peer":
        harvest_peer_rows(context, record)
    elif record["kind"] == "local":  # 重报（carry）不真跑，不记台：那几行在哪台跑的，账上早先已记
        for row in record["rows"]:
            if is_done(context, step, row):
                append_ledger(context, step.key, row, record["host"], "local")
    done = all(is_done(context, step, row) for row in record["rows"])
    ok = code == 0 and done and os.path.exists(record["product"])
    note = "（250–254、245 是内存包装自己的结局，那一次不算）" if 245 == code or 250 <= code <= 254 else ""
    say(context, f"  第 {step.number} 步 {step.key} 第 {rows_text(record['rows'])} 行 {host_name(record['host'])} {'跑完' if ok else '没跑成'}："
                 f"退 {code}{note}，行目录 {'都有' if done else '不全有'} row-result.tsv")
    return ok


def stop_running(context: Context, running: list[Running]) -> None:
    """收到停止信号：给每个在跑的分片（各自一个进程组）发一次 TERM；转交的那一条收到后写停止请求，代跑那一片停那边那条命令、删那一台本地盘上它的目录。
    等它们都退出，再把别的台已写进共享根的做完的行收进来。"""
    for item in running:
        try:
            os.killpg(item.process.pid, signal.SIGTERM)
        except ProcessLookupError:
            pass
    for item in running:
        finish_shard(context, item.record, item.process.wait())


def run_queue(context: Context, allocations: list[HostAllocation], jobs: list[Job], row_zero_host: int | None, expected_hashes: dict[str, str],
              kit: str | None) -> tuple[list[dict], list[Job], list[str]]:
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
                if slot[0] == context.own_host:
                    fresh = 0 in job.rows or job.step in ("r11", "r12")
                    work_root = os.path.join(context.work_root, f"{os.path.basename(context.attempt_dir)}-{label}" if fresh else f"host1-slot{slot[1]}")
                    process, record = launch_local(context, expected_hashes, label, step, job.rows, 1, threads_of[slot[0]], work_root, "local")
                else:
                    assert kit is not None, "有别的台的槽就建过给它读的那一份"
                    process, record = launch_peer(context, kit, label, step, job.rows, threads_of[slot[0]], slot[0])
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
    """更早几趟已做完的行：本机一个驱动从行目录重报（同键做完的行按键复用，不编、不跑装置），这一趟才有一份覆盖全部行的分片。
    线程取每台线程（登记值）：重报在那一步的队列收完之后才起（merge_step 在 run_queue 交回之后调），不与线道同时跑。"""
    work_root = os.path.join(context.attempt_dir, f"carry-work-{step.key}")
    process, record = launch_local(context, expected_hashes, f"{step.key}-carry", step, rows, 1, registered_host_threads(), work_root, "carry")
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
    """这一趟给别的台读的那一份（共享根下的快照与 expected.json）、各台的预检只做一次。"""

    def __init__(self, context: Context, hosts: list[Host], values: dict[str, str], snapshot: tuple[str, str]) -> None:
        self.context, self.hosts, self.values, self.snapshot = context, {host.number: host for host in hosts}, values, snapshot
        self.kit: str | None = None
        self.prepared: set[int] = set()

    def ready(self, number: int, slots: int, threads: int) -> None:
        if number == self.context.own_host or number in self.prepared:
            return
        if self.kit is None:
            self.kit = build_peer_kit(self.context, expected_description(self.context, self.snapshot))
        for line in preflight_peer(self.context, self.kit, self.hosts[number], slots, threads):
            say(self.context, f"  预检 {host_name(number)}：{line}")
        if STOP.is_set():
            raise Interrupted()
        self.prepared.add(number)


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
            raise RunCondition("各台的线程都不够一条线道（每台线程都是 0）")
        placements, deferred = simulate(allocations, jobs, row_zero_host)
        say(context, f"  这一趟要跑 {len(jobs)} 行（已做完的不重跑）：" + " ".join(f"{job.step}:{rows_text(job.rows)}" for job in queue_order(jobs)))
        for line in prediction_text(allocations, placements, deferred):
            say(context, "  " + line)
        for allocation in allocations:
            if allocation.slots > 0:
                setup.ready(allocation.number, allocation.slots, allocation.threads_per_slot)
        records, deferred, failures = run_queue(context, allocations, jobs, row_zero_host, expected_hashes, setup.kit)
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
        text, _ = cross_host_product(None, None, None, single_host_reason(context))
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


def single_host_reason(context: Context) -> str:
    """跨机对照报 not_run 时写的原因（文件头「经 task-server」）。"""
    return "只有一台（双机开关没开或清单里只有本机）" if context.relay_plan else "没经 task-server 分片（只用这一台）"


def phase_timeout_reruns(context: Context, hosts: list[Host], setup: PeerSetup, expected_hashes: dict[str, str]) -> None:
    """第 13 步（原规则）：第 11、12 步里退 253 的行单线道再跑一次，限时再乘 3；带探针的先、不带的后；台照 single_lane_host 挑（编排那一台），线程取每台线程（登记值）。"""
    timed = {"r11": timed_out_rows(context.product("rows-probe-driver")), "r12": timed_out_rows(context.product("rows-driver"))}
    say(context, f"  超时的行：带探针 {rows_text(timed['r11']) or 'none'}；不带探针 {rows_text(timed['r12']) or 'none'}")
    for key in ("r11", "r12"):
        step, rows = STEPS[key], timed[key]
        if not rows or os.path.exists(context.product(step.stem)):
            continue
        remaining = tuple(row for row in rows if not is_done(context, step, row))
        records: list[dict] = []
        if remaining:
            allocations = allocate(hosts)
            for line in allocation_text(allocations):
                say(context, "  " + line)
            chosen = single_lane_host(allocations, context.own_host)
            single = HostAllocation(chosen.number, chosen.threads, 1, chosen.threads)
            say(context, f"  第 13 步 {key} 第 {rows_text(remaining)} 行 → {host_name(chosen.number)}，单线道 {chosen.threads} 线程（限时再乘 3 由驱动现算）")
            setup.ready(chosen.number, 1, chosen.threads)
            records, deferred, failures = run_queue(context, [single], [Job(key, remaining, 0.0)], None, expected_hashes, setup.kit)
            if failures or deferred:
                raise Stopped("第 13 步没跑成：" + "；".join(failures or ["放不下"]))
        merge_step(context, step, rows, records, setup.snapshot, expected_hashes)
    say(context, "✓ 第 13 步绿")


def run(context: Context, plan_only: bool) -> int:
    for signal_number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(signal_number, on_signal)
    snapshot_check(context)
    if plan_only:
        hosts, values, note = load_hosts(context)
        if note:
            print(note)
        allocations = allocate(hosts)
        seconds = row_seconds(context)
        keys = ("s11", "s12", "sx") if len(hosts) > 1 else ("s11", "s12")
        jobs = [Job(key, (row,), seconds[(key, row)]) for key in keys for row in STEP_ROWS[key] if not is_done(context, STEPS[key], row)]
        placements, deferred = simulate(allocations, jobs, None)
        for line in allocation_text(allocations) + prediction_text(allocations, placements, deferred):
            print(line)
        print("每行推的用时（秒）：" + " ".join(f"{key}:{row}={value:.0f}" for (key, row), value in sorted(seconds.items())))
        return 0
    hosts, values, note = session_hosts(context)
    context.attempt_dir = next_attempt_directory(context)
    say(context, f"多机编排开跑：{os.path.basename(context.attempt_dir)}，行输出根 {os.path.relpath(context.rows_root, context.repo)}")
    if note:
        say(context, "  " + note)
    for record in shard_records(context):
        if record["kind"] == "peer" and record["attempt"] != os.path.basename(context.attempt_dir):
            harvested = harvest_peer_rows(context, record)
            if harvested:
                say(context, f"  收进以前各趟别的台做完的行：{record['label']}（{record['attempt']}）第 {rows_text(harvested)} 行")
    allocations = allocate(hosts)
    for line in allocation_text(allocations):
        say(context, "  " + line)
    expected_hashes = driver_hashes(context)
    setup = PeerSetup(context, hosts, values, build_snapshot_staging(context))
    phase_rows_and_checks(context, hosts, allocations, setup, expected_hashes)
    phase_timeout_reruns(context, hosts, setup, expected_hashes)
    snapshot_check(context)
    return 0


DEFAULT_HOSTS_COMMAND = "bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts"


def hosts_command_of(environment) -> str:
    return environment.get("ROWS_ORCHESTRATOR_HOSTS_COMMAND") or DEFAULT_HOSTS_COMMAND


def poll_seconds_of(environment) -> float:
    return float(environment.get("ROWS_ORCHESTRATOR_POLL_SECONDS") or 10)


def context_from_arguments(arguments: argparse.Namespace) -> Context:
    environment = os.environ
    driver_override = environment.get("ROWS_ORCHESTRATOR_DRIVER", "")
    research_root = os.path.abspath(arguments.research_root or arguments.repo)
    return Context(
        repo=os.path.abspath(arguments.repo), research_root=research_root, prefix=arguments.prefix, suffix=arguments.suffix, tag=arguments.resume_tag,
        dumps=arguments.dumps, tree=arguments.tree, snapshot_digest=arguments.snapshot_digest, baseline_timing=arguments.baseline_timing,
        check=os.path.abspath(arguments.check), logs=os.path.abspath(arguments.logs), work_root=os.path.abspath(arguments.work_root),
        driver_path=os.path.abspath(driver_override) if driver_override else os.path.join(research_root, RESEARCH_RELATIVE, DRIVER_NAME),
        driver_overridden=bool(driver_override), hosts_command=hosts_command_of(environment), poll_seconds=poll_seconds_of(environment),
        peer_compile_check=environment.get("ROWS_ORCHESTRATOR_PEER_COMPILE_CHECK", "1") != "0")


# ── 经 task-server（文件头「经 task-server」）──
TASK_SERVER_PHASE_VARIABLE = "TASK_SERVER_SHARD_PHASE"
SHARD_POSITION_VARIABLE = "TASK_SERVER_SHARD"
SHARD_PLAN_VARIABLE = "TASK_SERVER_SHARD_PLAN_DIRECTORY"
SHARD_OUTPUT_VARIABLE = "TASK_SERVER_SHARD_OUTPUT_DIRECTORY"
SHARD_OUTPUTS_VARIABLE = "TASK_SERVER_SHARD_OUTPUTS"
SHARD_HOST_VARIABLE = "TASK_SERVER_SHARD_HOST"
SESSION: dict = {}   # 编排那一片：plan、count、address、beat（orchestrator_session 填一次）


def shard_position(text: str) -> tuple[int, int] | None:
    """TASK_SERVER_SHARD=<片号>/<片数> → (片号, 片数)；写坏了交 None。"""
    parts = text.split("/")
    if len(parts) != 2 or not all(part.isdigit() for part in parts) or int(parts[0]) >= int(parts[1]):
        return None
    return int(parts[0]), int(parts[1])


def planned_shards(repo: str, environment) -> tuple[int, str]:
    """计划那一步的片数（文件头「机器」）：清单判法解出几台就几片，至多 MAXIMUM_SHARDS。交回 (片数, 给日志的一句)。"""
    completed = subprocess.run(["bash", "-c", hosts_command_of(environment)], cwd=repo, capture_output=True, text=True,
                               env=dict(environment))
    hosts, _, note = hosts_from_configuration(completed.returncode, completed.stdout)
    count = min(len(hosts), MAXIMUM_SHARDS)
    if len(hosts) > MAXIMUM_SHARDS:
        note = (note + "；" if note else "") + f"清单里有 {len(hosts)} 台，服务只分本机与第二台两种，多出来的 {len(hosts) - MAXIMUM_SHARDS} 台用不上"
    return count, note


def write_shard_plan(repo: str, plan: str, environment, say) -> int:
    count, note = planned_shards(repo, environment)
    shards = []
    for index in range(count):
        entry = {"label": "编排那一片（第 11–13 步、合并、判法、跨机对照）" if index == 0 else f"代跑那一片：编排那一片转交的第 {index} 片的行"}
        if broken("plan-pins-local"):
            entry["pin"] = "local"
        if broken("plan-writes-memory-limit"):
            entry.update(json.loads('{"memory_limit": "6G"}'))   # 弄坏开关造的错键：整句写成一个字面量再解
        shards.append(entry)
    temporary = os.path.join(plan, "plan.json.partial")
    with open(temporary, "w", encoding="utf-8") as handle:
        json.dump({"shards": shards, "note": note or "清单解出的台数"}, handle, ensure_ascii=False)
    os.replace(temporary, os.path.join(plan, "plan.json"))
    if note:
        say(f"· {note}")
    say(f"· 计划：{count} 片（不钉台，服务挑机器）；片号 0 编排，别的片代跑转交的命令")
    return 0


def merge_shards(plan: str, environment, say) -> int:
    """合并那一步（文件头「经 task-server」③）：整件已在编排那一片做完；核它写了 finished、各片的产物目录在。"""
    table = environment.get(SHARD_OUTPUTS_VARIABLE, "")
    missing = []
    for line in open(table, encoding="utf-8").read().splitlines() if table and os.path.exists(table) else []:
        fields = line.split("\t")
        if len(fields) >= 3 and not os.path.isdir(fields[2]):
            missing.append(fields[0])
    if not os.path.exists(shard_relay.finished_path(plan)):
        say(f"✗ 合并：编排那一片没写 {shard_relay.finished_path(plan)}（退 1）")
        return 1
    if missing:
        say(f"✗ 合并：第 {','.join(missing)} 片的产物目录不在（退 1）")
        return 1
    say("· 合并：整件已在编排那一片做完（合成、判法、跨机对照），没有要另合的")
    return 0


def orchestrator_session(plan: str, count: int, address: str) -> dict:
    """编排那一片：登记、起心跳、收尾写 finished（atexit，退出码不论）。只做一次。服务重试这一片时先删上一次留下的 finished，
    还在的代跑那一片接着代跑（已经见到 finished 退了的，这一次转交给它的命令照心跳停了退 255）。"""
    if not SESSION:
        if os.path.exists(shard_relay.finished_path(plan)):
            os.remove(shard_relay.finished_path(plan))
        shard_relay.register(plan, 0, address)
        SESSION.update(plan=plan, count=count, address=address, beat=shard_relay.Beat(plan, 0).start())
        atexit.register(shard_relay.mark_finished, plan)
    return SESSION


def shard_phase_before_layer(repo: str, environment=None, say=print) -> int | None:
    """服务拆的那几步（文件头「经 task-server」）：计划、合并与代跑的片在这里跑完、交退出码；编排那一片与不经服务直接调的交 None，往下照编排跑。
    各层编排器的 main 在载入配置、写任何东西之前调它。"""
    environment = os.environ if environment is None else environment
    phase = environment.get(TASK_SERVER_PHASE_VARIABLE, "")
    if phase == "":
        return None
    if phase not in ("plan", "shard", "merge"):
        say(f"✗ {TASK_SERVER_PHASE_VARIABLE}={phase} 认不出（只认 plan、shard、merge；这个变量只由 task-server 设）（退 2）")
        return 2
    plan = environment.get(SHARD_PLAN_VARIABLE, "")
    if not plan or not os.path.isabs(plan) or not os.path.isdir(plan):
        say(f"✗ {phase} 那一步没给在的计划目录（{SHARD_PLAN_VARIABLE}={plan or '没设'}）；这一步只由 task-server 起（退 2）")
        return 2
    if phase == "plan":
        try:
            return write_shard_plan(repo, plan, environment, say)
        except RunCondition as error:
            say(f"✗ 运行条件不满足（退 78）：{error}")
            return EXIT_RUN_CONDITION
    if phase == "merge":
        return merge_shards(plan, environment, say)
    position, address = shard_position(environment.get(SHARD_POSITION_VARIABLE, "")), environment.get(SHARD_HOST_VARIABLE, "")
    if position is None or not shard_relay.is_host_address(address):
        say(f"✗ 片那一步的 {SHARD_POSITION_VARIABLE}={environment.get(SHARD_POSITION_VARIABLE, '')!r}、{SHARD_HOST_VARIABLE}={address!r} 认不出（退 2）")
        return 2
    output = environment.get(SHARD_OUTPUT_VARIABLE, "")
    if output:
        os.makedirs(output, exist_ok=True)
    index, count = position
    if index == 0:
        orchestrator_session(plan, count, address)
        return None
    shard_relay.register(plan, index, address)
    return shard_relay.serve(plan, index, poll_seconds_of(environment), say)


def wait_for_registrations(context: Context, plan: str, count: int) -> dict[int, str]:
    deadline, told = time.monotonic() + REGISTRATION_WAIT_SECONDS, False
    while True:
        found = shard_relay.registrations(plan)
        if len(found) >= count:
            return found
        if STOP.is_set():
            raise Interrupted()
        if time.monotonic() > deadline:
            missing = [index for index in range(count) if index not in found]
            raise RunCondition(f"计划分了 {count} 片，第 {rows_text(missing)} 片 {REGISTRATION_WAIT_SECONDS:.0f} 秒里没起来（服务还没派出去）：同一条命令再交")
        if not told:
            say(context, f"  等计划里的 {count} 片都起来（已登记 {len(found)} 片）")
            told = True
        time.sleep(context.poll_seconds)


def session_hosts(context: Context) -> tuple[list[Host], dict[str, str], str]:
    """跑的时候用哪几台（文件头「机器」）：不经服务直接调只用这一台；编排那一片等各片登记，台照各片的 TASK_SERVER_SHARD_HOST 定，别的台记下代跑的片号。"""
    if os.environ.get(TASK_SERVER_PHASE_VARIABLE, "") != "shard":
        context.own_host, context.relay_plan, context.relay_targets = 1, "", {}
        return [Host(1, "", "", "", "", False)], {}, "没经 task-server 分片（TASK_SERVER_SHARD_PHASE 没设）：只用这一台，跨机对照报 not_run"
    position = shard_position(os.environ.get(SHARD_POSITION_VARIABLE, ""))
    address = os.environ.get(SHARD_HOST_VARIABLE, "")
    if position is None or position[0] != 0 or not shard_relay.is_host_address(address):
        raise RunCondition("编排只在片号 0 那一片里跑（别的片代跑转交的命令）")
    session = orchestrator_session(os.environ[SHARD_PLAN_VARIABLE], position[1], address)
    context.own_host, context.relay_plan = ORCHESTRATOR_HOST_NUMBER, session["plan"]
    own_scratch = os.environ.get(shard_relay.SCRATCH_VARIABLE, "")
    if own_scratch and os.path.isdir(own_scratch):   # 看这一台自己的性质：服务给了这一片本地盘上的草稿目录，自己那几行的工作根就放那里
        context.work_root = os.path.join(own_scratch, "rows-work")
    found = wait_for_registrations(context, session["plan"], session["count"])
    targets: dict[int, int] = {}
    numbers_of_other_addresses: dict[str, int] = {}
    for index in sorted(found):
        if found[index] == address:
            continue
        number = numbers_of_other_addresses.setdefault(found[index], ORCHESTRATOR_HOST_NUMBER + 1 + len(numbers_of_other_addresses))
        if index != 0 and number not in targets:
            targets[number] = index
    if session["count"] > 1 and not targets:
        raise RunCondition(f"计划分了 {session['count']} 片，都落在{host_name(context.own_host)}：跨机对照做不成。第二台空出来后同一条命令再交")
    context.relay_targets = targets
    hosts = [Host(number, "", "", "", "", number != 1) for number in sorted({context.own_host} | set(targets))]
    note = (f"经 task-server 分片：{session['count']} 片，编排在{host_name(context.own_host)}"
            + "".join(f"，{host_name(number)}由第 {index} 片代跑" for number, index in sorted(targets.items())))
    return hosts, {}, note


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
    if arguments.mode == "run":
        phase_exit = shard_phase_before_layer(os.path.abspath(arguments.repo), say=lambda line: print(line, flush=True))
        if phase_exit is not None:
            return phase_exit
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
        say(context, "收到停止信号：在跑的各行已停、别的台已做完的行已收进来；同一条命令再起就是续跑")
        return EXIT_STOPPED_BY_SIGNAL


# ── 自证 ──
REAL_REPOSITORY = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", ".."))  # 本文件在 scripts/experiment-apparatus/<装置>/ 下
FAKES = HERE  # 假驱动 fake_*.py / fake_*.sh 与本文件同目录
FAKE_TASK_SERVER = os.path.join(FAKES, "fake_task_server.py")


SAMPLE_HOST_THREADS = 31  # 自证样本：按每台 31 线程算槽，不随登记值变


def two_hosts(peer_cap: str = "50G") -> list[Host]:
    return [Host(1, "", "", "", "", False), Host(2, "fakepeer", "/fake/peer", "/fake/cargo", peer_cap, True)]


def cells_allocation() -> list[tuple[str, bool, str]]:
    results = []
    threads = [(item.slots, item.threads_per_slot) for item in allocate(two_hosts(), SAMPLE_HOST_THREADS)]
    results.append(("allocation-threads", threads == [(1, 31), (1, 31)], f"31 线程两台 (槽, 每槽线程) = {threads}，要 [(1, 31), (1, 31)]"))
    short = [(item.slots, item.threads_per_slot) for item in allocate(two_hosts(), 0)]
    results.append(("allocation-short", short == [(0, 0), (0, 0)], f"每台线程 0 记 0 槽 (槽, 每槽线程) = {short}，要 [(0, 0), (0, 0)]"))
    # 每台线程是登记值、与此刻空闲无关，也与配置的内存上限无关
    registered = registered_host_threads()
    found = [(item.threads, item.slots, item.threads_per_slot) for item in allocate(two_hosts("30G"))]
    wanted = [(registered, 1, registered)] * 2
    results.append(("allocation-registered", registered > 1 and found == wanted, f"每台取登记值 {registered}、配置上限 30G 不卡槽数 → {found}，要 {wanted}"))
    chosen = single_lane_host(allocate(two_hosts("30G")), 1)
    on_peer = single_lane_host(allocate(two_hosts("30G")), 2)
    results.append(("allocation-single-lane", (chosen.number, chosen.threads, on_peer.number, on_peer.threads) == (1, registered, 2, registered),
                    f"第 13 步挑编排那一台：编排在本机挑 {chosen.number} 台 {chosen.threads} 线程，编排在第二台挑 {on_peer.number} 台 {on_peer.threads} 线程，要 {registered}（登记值）"))
    return results


def cells_queue() -> list[tuple[str, bool, str]]:
    seconds = row_seconds_table(1000.0, 800.0, {269: 500.0})
    jobs = [Job(key, (row,), seconds[(key, row)]) for key in ("s11", "s12", "sx") for row in STEP_ROWS[key]]
    order = [f"{job.step}:{job.rows[0]}" for job in queue_order(jobs)]
    good_order = order[:3] == ["s11:242", "s12:0", "sx:0"] and order[-1] == "s11:269" and len(order) == 17
    results = [("queue-order", good_order, f"头三个 {order[:3]}，末一个 {order[-1]}，共 {len(order)}")]
    two = [HostAllocation(1, 10, 1, 10), HostAllocation(2, 10, 1, 10)]
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
    product = [f"name=row_time_limit seconds={limit} baseline_seconds=15374.455 source=x-timing.out",
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
             ("merge-refuse-unknown-line", [fake_shard("a", 1, (0,), extra="not_a_known_line rows=0")], (0,), "⑦")]
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


def served_command(root: str, command: list[str], roles: str = "local,peer", peer_without_rsync: bool = False) -> list[str]:
    """command 照服务的分片三步起（fake_task_server.py：计划、各片并行、合并），各片的台照 roles 轮着取。"""
    return (["python3", "-B", FAKE_TASK_SERVER, "--root", os.path.join(root, "task-server"), "--roles", roles]
            + (["--peer-without", "rsync"] if peer_without_rsync else []) + ["--"] + command)


def relay_labels(root: str) -> list[str]:
    """假 task-server 各趟里转交过的请求的标签（计划目录 relay/to-<片号>/<标签>）。"""
    base = os.path.join(root, "task-server")
    labels = []
    for run_name in sorted(os.listdir(base)) if os.path.isdir(base) else []:
        relay = os.path.join(base, run_name, "plan", "relay")
        for inbox in sorted(os.listdir(relay)) if os.path.isdir(relay) else []:
            if inbox.startswith("to-") and os.path.isdir(os.path.join(relay, inbox)):
                labels += sorted(os.listdir(os.path.join(relay, inbox)))
    return labels


def fake_world(root: str, hosts_code: int = 0, extra_environment: dict[str, str] | None = None, roles: str = "local,peer",
               peer_without_rsync: bool = False) -> FakeWorld:
    repo = os.path.join(root, "repo")
    results = os.path.join(repo, "research", "results")
    os.makedirs(results)
    tree = os.path.join(repo, "scripts", "experiment-apparatus", "target", "fake-frozen-tree")
    for relative, text in (("Cargo.toml", "[workspace]\n"), ("Cargo.lock", "# lock\n"), (".cargo/config.toml", "[env]\n"),
                           ("crates/a/src/lib.rs", "pub fn a() {}\n"), ("target/release/build/x.rs", "// 不进快照\n")):
        os.makedirs(os.path.dirname(os.path.join(tree, relative)), exist_ok=True)
        with open(os.path.join(tree, relative), "w", encoding="utf-8") as handle:
            handle.write(text)
    prefix, suffix = "research/results/fake-e171-s1", "-r6-r3"
    for stem, seconds in (("baseline-run1", "100.500"), ("positive-control-o1-crash-judge", "80.000"), ("positive-control-reference-filesystem", "70.000")):
        with open(os.path.join(repo, f"{prefix}-{stem}{suffix}-timing.out"), "w", encoding="utf-8") as handle:
            handle.write(f"E7TIMING name=stream_time stream=a elapsed_ns=1000000000\nE7TIMING name=process_wall seconds={seconds} exit=0\n")
    frozen_reference = "e170-r2-o3-spec-executor-2026-11-05-refreeze-executor.out"
    os.symlink(os.path.join(REAL_REPOSITORY, "research", "results", frozen_reference), os.path.join(results, frozen_reference))
    os.makedirs(os.path.join(repo, "scripts", "experiment-apparatus", "target", "fake-dumps"))
    sys.path.insert(0, os.path.join(REAL_REPOSITORY, RESEARCH_RELATIVE))
    import row_driver  # noqa: E402
    digest = row_driver.snapshot_digest(tree)[0]
    assignments = ("HOSTS='127.0.0.1 192.0.2.67'\nTHIS_HOST_ADDRESS=127.0.0.1\nPEER_HOST_ADDRESS=192.0.2.67\nCARGO_BIN_DIRECTORY_BY_HOST='127.0.0.1=/fake/cargo 192.0.2.67=/fake/cargo'\nBUILD_ROOT_BY_HOST='127.0.0.1=/fake/builds 192.0.2.67=/fake/peer'\nSLICE_MEMORY_CAP_BY_HOST='127.0.0.1=50G 192.0.2.67=50G'\n"
                   "QUIESCE_STOP_COMMAND=''\nQUIESCE_STOPPED_CHECK_COMMAND=''\nQUIESCE_START_COMMAND=''\nQUIESCE_STARTED_CHECK_COMMAND=''\n"
                   f"ENABLE_ACROSS_MACHINES=1\nENABLE_GPU=0\nNFS_SHARED_ROOTS={root}\nMULTI_HOST_COUNT=1\nMULTI_HOST_2_SSH_HOST=192.0.2.67\n"
                   "MULTI_HOST_2_REPOSITORY_DIRECTORY=/fake/peer\nMULTI_HOST_2_CARGO_BIN_DIRECTORY=/fake/cargo\nMULTI_HOST_2_MEMORY_CAP=50G\nMULTI_HOST_2_IS_PEER=1\n")
    with open(os.path.join(root, "hosts.txt"), "w", encoding="utf-8") as handle:
        handle.write(assignments)
    environment = {name: value for name, value in os.environ.items() if not name.startswith(("TASK_SERVER_SHARD", "MULTI_HOST_RUN_PEER_SCRATCH"))}
    environment.update(ROWS_ORCHESTRATOR_SELFTEST="1", ROWS_ORCHESTRATOR_DRIVER=os.path.join(FAKES, "fake_row_driver.py"),
                       ROWS_ORCHESTRATOR_HOSTS_COMMAND=f"cat {shlex.quote(os.path.join(root, 'hosts.txt'))}; exit {hosts_code}",
                       ROWS_ORCHESTRATOR_POLL_SECONDS="0.05", ROWS_ORCHESTRATOR_PEER_COMPILE_CHECK="0",
                       FAKE_RESEARCH_DIRECTORY=os.path.join(REAL_REPOSITORY, RESEARCH_RELATIVE), FAKE_REAL_REPOSITORY=REAL_REPOSITORY,
                       FAKE_RESULTS_DIRECTORY=results, FAKE_DRIVER_LOG=os.path.join(root, "driver.log"), PYTHONDONTWRITEBYTECODE="1")
    environment.update(extra_environment or {})
    dumps = "scripts/experiment-apparatus/target/fake-dumps"
    command = ["python3", "-B", os.path.abspath(__file__), "run", "--repo", repo, "--prefix", prefix, f"--suffix={suffix}", "--resume-tag=-selftest",
               "--dumps", dumps, "--tree", "scripts/experiment-apparatus/target/fake-frozen-tree", "--snapshot-digest", digest,
               "--baseline-timing", f"{prefix}-baseline-run1{suffix}-timing.out", "--check", os.path.join(HERE, "check_product.py"),
               "--logs", os.path.join(root, "logs"), "--work-root", os.path.join(root, "work"), "--research-root", REAL_REPOSITORY]
    return FakeWorld(root, repo, served_command(root, command, roles, peer_without_rsync), environment, prefix + "-", dumps)


def world_run(world: FakeWorld) -> tuple[int, str]:
    completed = subprocess.run(world.command, cwd=world.repo, env=world.environment, capture_output=True, text=True, timeout=180)
    return completed.returncode, completed.stdout + completed.stderr


def world_product(world: FakeWorld, stem: str) -> str:
    return os.path.join(world.repo, f"{world.prefix}{stem}-selftest-r6-r3.out")


def ran_counts(world: FakeWorld) -> dict[tuple[int, str], int]:
    """假驱动真跑过（按键复用读回的不算）的次数，按 (行, 限时是不是乘过 3) 数；第 12 步 row-0 与 X 都是 row-0，合起来该是 2。"""
    counts: dict[tuple[int, str], int] = {}
    path = os.path.join(world.root, "driver.log")
    for line in open(path, encoding="utf-8") if os.path.exists(path) else []:
        if line.startswith("ran "):
            fields = key_values(line)
            key = (int(fields["row"]), fields["rerun"])
            counts[key] = counts.get(key, 0) + 1
    return counts


def read_row_result_fields(row_directory: str) -> dict[str, str]:
    path = os.path.join(row_directory, "row-result.tsv")
    if not os.path.exists(path):
        return {}
    with open(path, encoding="utf-8") as handle:
        return dict(line.rstrip("\n").split("\t", 1) for line in handle if "\t" in line)


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
    if want("e2e-orchestrator-on-peer"):
        results.append(orchestrator_on_peer_cell(scratch))
    if want("e2e-config-refused") or want("e2e-switch-off") or want("e2e-preflight-refused") or want("e2e-product-collision"):
        results += refusal_cells(scratch)
    return results


def two_hosts_cell(scratch: str) -> tuple[str, bool, str]:
    world = fake_world(os.path.join(scratch, "two-hosts"), extra_environment={"FAKE_DRIVER_TIMEOUT_ROWS": "15"})
    code, output = world_run(world)
    counts = ran_counts(world)
    expected_runs = expected_ran_counts() | {(15, "true"): 1}
    rows_root = os.path.join(world.repo, world.dumps, "segment1-rows-selftest")
    results_of = {row: read_row_result_fields(os.path.join(rows_root, f"row-{row}")) for row in (15, 23)}
    s12 = world_product(world, "rows-driver")
    s12_rows = [key_values(line)["row_number"] for line in open(s12, encoding="utf-8") if line.startswith("row ")] if os.path.exists(s12) else []
    ledger = open(os.path.join(world.repo, world.dumps, "segment1-rows-selftest-shards", "row-hosts.tsv"), encoding="utf-8").read() \
        if os.path.exists(os.path.join(world.repo, world.dumps, "segment1-rows-selftest-shards", "row-hosts.tsv")) else ""
    good = (code == 0 and counts == expected_runs
            and s12_rows == [str(row) for row in sorted(STEP_ROWS["s12"])] and "\tpeer\n" in ledger
            and "cross_host_identical=true" in verdict_of(world_product(world, CROSS_HOST_CHECK_STEM))
            and os.path.exists(world_product(world, "rows-timeout-rerun-driver")) and not os.path.exists(world_product(world, "rows-probe-timeout-rerun-driver"))
            # 第 13 步在第 12 步同一个行输出根里重跑：上一次超时的第 15 行限时是别的行的 3 倍、记着乘过 3，没有另一个输出根
            and results_of[15].get("timed_out_before") == "true" and results_of[23].get("timed_out_before") == "false"
            and int(results_of[15].get("time_limit_seconds", "0")) == 3 * int(results_of[23].get("time_limit_seconds", "0")) > 0
            and not os.path.exists(rows_root + "-timeout-rerun"))
    return ("e2e-two-hosts", good, f"退 {code}；真跑过的 (行, 重跑) 与次数和预期{'相同' if counts == expected_runs else '不同：' + str(counts)}；第 12 步产物行 {len(s12_rows)}；"
                                           f"跨机 {verdict_of(world_product(world, CROSS_HOST_CHECK_STEM))[:80]}；输出末行 {output.strip().splitlines()[-1:]}")


def orchestrator_on_peer_cell(scratch: str) -> tuple[str, bool, str]:
    """编排那一片落在第二台、代跑那一片落在本机（服务「各片第二台在前」）：两台的行都跑、跨机对照两次在两台，编排那一台自己的行的工作根在它的本地目录下。"""
    world = fake_world(os.path.join(scratch, "on-peer"), roles="peer,local")
    code, output = world_run(world)
    lines = open(os.path.join(world.root, "driver.log"), encoding="utf-8").read().splitlines() if os.path.exists(os.path.join(world.root, "driver.log")) else []
    hosts_ran = sorted({key_values(line).get("host", "?") for line in lines if line.startswith("ran ")})
    verdict = verdict_of(world_product(world, CROSS_HOST_CHECK_STEM))
    unused_work_root = not os.path.exists(os.path.join(world.root, "work"))
    good = (code == 0 and ran_counts(world) == expected_ran_counts() and hosts_ran == ["1", "2"] and "cross_host_identical=true" in verdict
            and unused_work_root)
    return ("e2e-orchestrator-on-peer", good, f"退 {code}；真跑过的台 {hosts_ran}；跨机 {verdict[:80]}；--work-root 没用上 {unused_work_root}；"
                                               f"输出末行 {output.strip().splitlines()[-1:]}")


def mismatch_cell(scratch: str) -> tuple[str, bool, str]:
    mismatch = fake_world(os.path.join(scratch, "mismatch"), extra_environment={"FAKE_DRIVER_SALT_HOST": "2", "FAKE_DRIVER_SALT_PART": "device"})
    code, output = world_run(mismatch)
    verdict = verdict_of(world_product(mismatch, CROSS_HOST_CHECK_STEM))
    return ("e2e-cross-host-mismatch", code == 1 and "cross_host_identical=false" in verdict and "differing_parts=device_results" in verdict
            and not os.path.exists(world_product(mismatch, "rows-timeout-rerun-driver")), f"退 {code}；{verdict[:120]}")


def refusal_cells(scratch: str) -> list[tuple[str, bool, str]]:
    results = []
    refused = fake_world(os.path.join(scratch, "config-refused"), hosts_code=1)
    code, output = world_run(refused)
    results.append(("e2e-config-refused", code == EXIT_RUN_CONDITION and "多机配置判不过" in output, f"退 {code}"))
    single = fake_world(os.path.join(scratch, "switch-off"), hosts_code=3)
    code, output = world_run(single)
    results.append(("e2e-switch-off", code == 0 and "cross_host_identical=not_run" in verdict_of(world_product(single, CROSS_HOST_CHECK_STEM))
                    and not os.path.exists(world_product(single, "rows-cross-host-row-0-driver")) and "只用本机" in output, f"退 {code}"))
    preflight = fake_world(os.path.join(scratch, "preflight-refused"), peer_without_rsync=True)
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
    """跑到一半收 TERM（退 143），同一条命令再起（退 0）：每个 (根, 行) 只真跑过一次；第二台已做完的行（写在共享根下它的分片输出目录里）在 TERM 时或续跑时收进来。"""
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
    harvested = "\tpeer\n" in open(os.path.join(world.repo, world.dumps, "segment1-rows-selftest-shards", "row-hosts.tsv"), encoding="utf-8").read()
    good = first == EXIT_STOPPED_BY_SIGNAL and second == 0 and counts == expected_ran_counts() and sum(before.values()) >= 4 and harvested
    return ("e2e-resume", good, f"第一次退 {first}（停前做完 {sum(before.values())} 行），续跑退 {second}；真跑过的次数与预期{'相同' if counts == expected_ran_counts() else '不同：' + str(counts)}；"
                               f"收进过第二台的行 {harvested}；续跑末行 {output.strip().splitlines()[-1:]}")


def cells_task_server(scratch: str) -> list[tuple[str, bool, str]]:
    """经 task-server（文件头「经 task-server」）：两台的清单计划写两片、一台写一片，片里只有 label（没有 pin 键、没有内存键）；
    合并见到 finished 退 0、没见到退 1；认不出的步退 2；不经服务直接调（变量没设）照常往下跑。"""
    hosts_file = os.path.join(scratch, "task-server-hosts.txt")
    with open(hosts_file, "w", encoding="utf-8") as handle:
        handle.write("MULTI_HOST_COUNT=1\nMULTI_HOST_2_SSH_HOST=192.0.2.67\nMULTI_HOST_2_REPOSITORY_DIRECTORY=/fake/peer\n"
                     "MULTI_HOST_2_CARGO_BIN_DIRECTORY=/fake/cargo\nMULTI_HOST_2_MEMORY_CAP=50G\nMULTI_HOST_2_IS_PEER=1\n")
    said: list[str] = []
    planned = {}
    for name, code in (("two", 0), ("one", 3)):
        plan_directory = os.path.join(scratch, f"task-server-plan-{name}")
        os.makedirs(plan_directory)
        environment = {"TASK_SERVER_SHARD_PHASE": "plan", "TASK_SERVER_SHARD_PLAN_DIRECTORY": plan_directory,
                       "ROWS_ORCHESTRATOR_HOSTS_COMMAND": f"cat {shlex.quote(hosts_file)}; exit {code}", "PATH": os.environ.get("PATH", "")}
        exit_code = shard_phase_before_layer(scratch, environment, said.append)
        try:
            with open(os.path.join(plan_directory, "plan.json"), encoding="utf-8") as handle:
                shards = [sorted(shard) for shard in json.load(handle)["shards"]]
        except (OSError, ValueError, KeyError) as error:
            shards = [f"读不出：{type(error).__name__}"]
        planned[name] = (exit_code, shards)
    merge_directory = os.path.join(scratch, "task-server-merge")
    os.makedirs(os.path.join(merge_directory, "relay"))
    merge_environment = {"TASK_SERVER_SHARD_PHASE": "merge", "TASK_SERVER_SHARD_PLAN_DIRECTORY": merge_directory}
    unfinished = shard_phase_before_layer(scratch, merge_environment, said.append)
    shard_relay.mark_finished(merge_directory)
    finished = shard_phase_before_layer(scratch, merge_environment, said.append)
    unknown = shard_phase_before_layer(scratch, {"TASK_SERVER_SHARD_PHASE": "split", "TASK_SERVER_SHARD_PLAN_DIRECTORY": merge_directory}, said.append)
    direct = shard_phase_before_layer(scratch, {}, said.append)
    good = (planned == {"two": (0, [["label"], ["label"]]), "one": (0, [["label"]])} and unfinished == 1 and finished == 0 and unknown == 2
            and direct is None)
    return [("task-server-phases", good, f"计划（两台、一台）{planned}；合并没 finished 退 {unfinished}、有了退 {finished}；认不出的步退 {unknown}；"
                                         f"不经服务直接调 {direct}")]


SWITCH_CELLS = {"slots-split-by-five": "allocation-threads", "merge-allows-duplicates": "merge-refuse-duplicate",
                "merge-ignores-time-limit": "merge-refuse-time-limit", "cross-host-ignores-dumps": "cross-host-compare",
                "resume-reruns-finished": "e2e-resume", "harvest-skipped": "e2e-two-hosts",
                "plan-writes-memory-limit": "task-server-phases", "driver-gets-memory-cap": "e2e-two-hosts", "peer-command-gets-memory-cap": "e2e-two-hosts",
                "plan-pins-local": "task-server-phases"}
CELL_GROUPS = {"allocation": cells_allocation, "queue": cells_queue}


def selftest(argument_list: list[str]) -> int:
    wanted = set(argument_list[argument_list.index("--cells") + 1].split(",")) if "--cells" in argument_list else None
    scratch = tempfile.mkdtemp(prefix="e171-rows-orchestrator-selftest-")
    try:
        results = (cells_allocation() + cells_queue() + cells_merge(scratch, os.path.join(HERE, "check_product.py"))
                   + cells_cross_host(scratch) + cells_task_server(scratch))
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
