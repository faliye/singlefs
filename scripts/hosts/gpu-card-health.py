#!/usr/bin/env python3
# admission: always 每一次都现查各台此刻的卡与这次开机的内核日志，上一次的结论不替这一次作保
# run-condition: command bash python3
"""显卡本身的状态：按多机配置点名的卡逐卡判好坏（用户 2026-10-30：「需要检测卡状态 如果卡状态不对 要弹窗提醒」；
records/2026-10-28-崩溃轻型提速.md「教训：卡坏了没人知道」）。只读：不重置、不清场、不停任何进程。

用法：
  gpu-card-health.py              查多机配置点名的全部卡，逐卡一行机读行、一行汇总、一两句给人看的话
  gpu-card-health.py --cards <显卡先后>   只查这几张（scripts/crash/gpu-quiesce-run.sh 清场之前查这一次分到的卡，下面「查哪几张卡」），输出同上
  gpu-card-health.py --selftest   假的 nvidia-smi、journalctl、ssh（PATH 前面）与假多机配置走一遍各格，末尾逐个打开弄坏开关核它红在点名的格
退出码：0 点名的卡都好；1 有坏卡（连不上的那台照报没查）；77 多机配置一张卡都没点名，没查；
  78 查不了：多机配置读不出（文件不在、卡的写法不对、卡的台标签不是 HOSTS 里的地址、认不出 HOSTS 里哪一项是这一台、点名了另一台的卡而双机开关没开或判不过）、
  另一台连不上、某台的内核日志读不出；--cards 里有一项写法不对、或台标签不是 HOSTS 里的地址也是 78；
  2 用法错（含弄坏开关设在自证之外、--cards 后面没有值或值里一项都没有）。

查哪几张卡：多机配置（${SINGLEFS_MULTI_HOST_CONFIG:-主工作树的根/multi-host.env}，取法同 scripts/crash/crash-amplification-lib.sh 的
  crash_amplification_multi_host_configuration_file）里 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 与 GPU_CARDS_BY_PRIORITY 两份并起来、按（台、序号）去重；
  每项写法 <HOSTS 里那一台的地址>:<序号>:<额度>（例 192.0.2.1:0:32G；写法在仓根 multi-host.env.example 的显卡两键）：从右边切两刀（card_labels），
  地址可以是 IPv6、自己带冒号；地址不是 HOSTS 里的一项判配置写坏（退 78）。环境变量 GPU_CARDS_BY_PRIORITY 不读。
  带了 --cards <显卡先后>（写法同上，空格隔开几项）只查这几张，多机配置那两个显卡键不读（HOSTS 与连第二台的键照读）：清场之前查卡只查服务分给这一次尝试的卡，
  没分到的卡坏了不挡这一次（查全部卡时，一张没分到的坏卡让每一条要卡的任务清场都判 card-fault）。
  弄坏开关 assigned-cards-ignored 下不认 --cards 的值，照查配置点名的全部卡（改前的样子）。
哪一台查哪几张（两台对称，没有起活一方与第二台的身份）：这一台是 HOSTS 里哪一项，照这一台网卡上的地址认（scripts/hosts/multi_host_addresses.py 的 resolve，
  判法只那一处）；台标签是这一台地址的那几项在本机查，是另一台地址的经 ssh 在那一台查。两台跑同一份配置查的是同一组卡，
  机读行的 host= 写那张卡的台标签（HOSTS 里的地址），在哪台跑都一样。
怎么连第二台：照 scripts/hosts/peer-host-lib.sh（scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts 判配置、不连任何一台，
  再经 peer_host_run 用 ssh 起，错误输出里的主机名与目录换成「第二台」）；送过去的是本文件 collect_card_readings 的源码（python3 -c，base64），
  同 scripts/hosts/host-dispatch.py 的 probe_peer 读第二台的办法。两台并行查；第二台至多等 GPU_CARD_HEALTH_HOST_TIMEOUT_SECONDS 秒（默认 HOST_TIMEOUT_SECONDS）。
每张卡判坏的几种（机读行 evidence= 里的项，逗号隔开）：
  nvidia-smi 起不来或读不出：nvidia-smi-exit=<码>、nvidia-smi-not-started（三种查询各一项：--query-gpu、--query-compute-apps 前缀 apps-、-q -i 前缀 query-）
  少了点名的卡：missing-from-nvidia-smi（--query-gpu 里没有这个序号）
  温度或功耗读不成数：temperature=<原样>、power=<原样>（ERR!、[N/A]、[Unknown Error]、[GPU requires reset] 这类；空格换成 _）
  GPU Recovery Action 不是 None：recovery-action=<原样>（nvidia-smi -q -i <序号> 里那一行；没有这一行的驱动不判，好卡那一行写 recovery-action=unreported）
  计算进程表里有 pid 为 [N/A] 的行落在这张卡的总线上：compute-app-pid-unknown=<行数>；总线也读不出、归不到哪张卡的，那台另打一行 card=unknown
  这次开机后内核报了致命类 Xid：xid-<号>=<条数>（只算这张卡总线上的，FATAL_XIDS；别的号只计数、写进 counted=，不判坏）
机读行：GPU_CARD_HEALTH host=<HOSTS 里的地址> card=<序号|unknown> bus=<总线|unknown> state=<ok|fault> evidence=<依据> counted=<xid-<号>:<条数>,…|none>
  好卡的 evidence 是读到的温度、功耗与 recovery-action；坏卡的只列判坏的那几项（看门狗按这一串判「同一份依据」）。
  末行汇总：GPU_CARD_HEALTH_SUMMARY state=<ok|fault|unchecked|nothing-named> checked=<查了几张> fault=<坏几张> unchecked=<没查成的，host:原因，逗号隔开|none>
  之后给人看的话：坏卡一张一行「✗ …」，再一行「→ 怎么办：」（弹窗出路，FAULT_NEXT_STEP_TEMPLATE）；查不成的一行「✗ 卡状态查不成：…」与它的出路。
致命类 Xid 的集合与依据（FATAL_XIDS；出处 NVIDIA 的 Xid 目录 docs.nvidia.com/deploy/xid-errors，推的：本机没查过每一种的现场）：
  48 显存双比特 ECC、62 卡上微控制器停机、74 NVLink 出错、79 卡从总线上掉了、94 / 95 显存 ECC（95 是没圈住的）、119 / 120 GSP 超时与出错、
  154 恢复动作变了（变成 GPU Reset Required、Node Reboot Required 这类）：都要重置卡或重启机器才好。
  只计数不判坏的（应用层的、驱动自己清得掉的）：13 图形引擎异常、31 显存缺页、43 卡停了一个上下文、45 抢占清理、109 上下文切换超时，与表外的号。
  2026-10-11、2026-10-30 第二台 0 号卡两次坏都是 62 接 154（「GPU Reset Required」）、之后 109 几千条（records 那一节）。
  154 那一行写着恢复动作变回 0x0 (None) 的，算这张卡已经恢复：它之前这张卡的 Xid 都不再判。
  这张卡在 PCI 层重新枚举过的同样算已经恢复：这张卡总线上功能号 0 的枚举行（pci <域>:<总线>:<设备>.0: [<厂商>:<设备号>] type …，开机与摘下扫回时
  内核各打一行）之前的 Xid 都不再判；同一张卡声卡功能（.1）单独的枚举行不算。PCI 层恢复（解绑驱动、写 sysfs 的 reset、摘下扫回）之后驱动不打
  154 变回 None 那一行（第二台 2026-11-04 量的：0 号卡 62 接 154 之后摘下扫回，nvidia-smi 读恢复动作 None，内核日志里有扫回那一刻的枚举行、没有 154 变回 None）。
  清掉之后的 Xid 与 nvidia-smi 读数照判。推的：nvidia-smi -r 之后驱动打不打 154 变回 None、内核打不打枚举行都没量过；两样都不打的话，
  这次开机里这张卡一直判坏，要重启那台才清。
内核日志：每台在自己那里读这次开机的 journalctl -k -b（nice -n 19），只认 NVRM: Xid (PCI:<域>:<总线>:<设备>): <号> 那一行与上面那种枚举行。
  头一次整份读（本机 2026-10-30 量的：这次开机内核日志 115 万行，journalctl 读一遍 35–40 秒），读到的 Xid 与枚举行按出现次序、相邻同类合并记进
  ${XDG_DATA_HOME:-~/.local/share}/singlefs/gpu-card-health/kernel-xid-<开机号>.json 连同 journalctl 的游标与记了哪几种行（recorded）；
  之后只读游标之后的（--after-cursor），游标认不得、或 recorded 不是这一份记的那几种（只记 Xid 的旧记录：游标之前的枚举行它没记）就整份重读；
  别的开机号的那几份删掉。写入原子（临时文件再 os.replace）。

只给自证用的环境变量：GPU_CARD_HEALTH_STATE_DIRECTORY（Xid 记录放哪，两台各取它下面的 local/、peer/）、GPU_CARD_HEALTH_BOOT_ID_FILE（开机号从哪读）。
弄坏开关（只给 --selftest 证红用，逗号连几个；在 --selftest 之外设了就拒、退 2）。项名、弄坏哪一处、打开后红在哪几格
  （格名是「✗ 自证 <格名>：」里的那一段；末尾那一轮照 SWITCH_EXPECTED_CELLS 逐个核）：
  GPU_CARD_HEALTH_BREAK=nvidia-smi-failure-trusted  nvidia-smi 起不来或退非 0 时照好卡办
      「✗ 自证 nvidia-smi-fails：」
  GPU_CARD_HEALTH_BREAK=missing-card-trusted  --query-gpu 里少了点名的卡也不判坏
      「✗ 自证 missing-card：」
  GPU_CARD_HEALTH_BREAK=readings-trusted  温度、功耗读不成数也不判坏
      「✗ 自证 temperature-err：」、「✗ 自证 power-na：」、「✗ 自证 temperature-unknown-error：」、「✗ 自证 power-requires-reset：」
  GPU_CARD_HEALTH_BREAK=recovery-action-ignored  不看 GPU Recovery Action
      「✗ 自证 recovery-action：」
  GPU_CARD_HEALTH_BREAK=unknown-pid-ignored  计算进程表里 pid 为 [N/A] 的行不算
      「✗ 自证 unknown-pid-process：」、「✗ 自证 unknown-pid-unknown-bus：」
  GPU_CARD_HEALTH_BREAK=fatal-xid-ignored  致命类 Xid 也只计数、不判坏
      「✗ 自证 fatal-xid：」、「✗ 自证 xid-incremental：」、「✗ 自证 fault-and-unreachable：」
  GPU_CARD_HEALTH_BREAK=application-xid-fatal  每一种 Xid 都判坏（应用层的 109、45 也算）
      「✗ 自证 application-xid-only：」、「✗ 自证 xid-incremental：」
  GPU_CARD_HEALTH_BREAK=xid-recovery-ignored  154 写着变回 None 的那一行不清掉它之前的 Xid
      「✗ 自证 xid-recovered：」
  GPU_CARD_HEALTH_BREAK=pci-rescan-ignored  这张卡总线上的枚举行不清掉它之前的 Xid
      「✗ 自证 xid-pci-rescanned：」
  GPU_CARD_HEALTH_BREAK=pci-rescan-any-bus  别的总线上的枚举行也清掉这张卡之前的 Xid
      「✗ 自证 xid-pci-rescanned：」
  GPU_CARD_HEALTH_BREAK=pci-rescan-order-ignored  这张卡总线上有枚举行就不判它的 Xid，不看枚举行在前在后（开机那一行也清掉之后的）
      「✗ 自证 xid-pci-rescanned：」
  GPU_CARD_HEALTH_BREAK=xid-cache-recorded-ignored  不看记录的 recorded，只记 Xid 的旧记录照续读（游标之前的枚举行读不到）
      「✗ 自证 xid-cache-before-pci：」
  GPU_CARD_HEALTH_BREAK=xid-cache-forgets  游标之后那一读不接上记录里已有的 Xid（只算新读到的）
      「✗ 自证 xid-incremental：」
  GPU_CARD_HEALTH_BREAK=kernel-log-failure-trusted  内核日志读不出时当没有 Xid、不报查不成
      「✗ 自证 kernel-log-unreadable：」
  GPU_CARD_HEALTH_BREAK=peer-unreachable-trusted  第二台连不上时当它没有点名的卡（退 0）
      「✗ 自证 peer-unreachable：」、「✗ 自证 fault-and-unreachable：」
  GPU_CARD_HEALTH_BREAK=this-side-by-order  不照这一台网卡上的地址认自己，一律当 HOSTS 的头一项是这一台（写死次序的样子）
      「✗ 自证 second-host-view：」
  GPU_CARD_HEALTH_BREAK=label-outside-hosts-accepted  台标签的地址不在 HOSTS 里也照收（当成另一台的卡）
      「✗ 自证 card-label-outside-hosts：」
  GPU_CARD_HEALTH_BREAK=label-split-from-left  台标签从左边切（地址里不许带冒号：IPv6 的台标签认不出）
      「✗ 自证 ipv6-card-label：」
  GPU_CARD_HEALTH_BREAK=empty-as-healthy  一张卡都没点名时退 0（当查过了都好）
      「✗ 自证 no-cards-named：」
  GPU_CARD_HEALTH_BREAK=missing-configuration-as-empty  多机配置不在时当一张卡都没点名（退 77），不报查不成
      「✗ 自证 configuration-missing：」
  GPU_CARD_HEALTH_BREAK=assigned-cards-ignored  带了 --cards 也照查多机配置点名的全部卡（改前的样子：没分到的坏卡挡住这一次）
      「✗ 自证 assigned-good-card：」、「✗ 自证 assigned-bad-card：」、「✗ 自证 assigned-label-outside-hosts：」

常量的依据：HOST_TIMEOUT_SECONDS 420：第二台头一次整份读内核日志按本机量的 40 秒算，加 nvidia-smi 几次、ssh 起一份 python，留几倍余量；推的。
  JOURNAL_SECONDS 300、NVIDIA_SMI_SECONDS 30 同是推的上限，到点那一步记查不成（内核日志）或读不出（nvidia-smi）。

# gate-similar: host-dispatch.py 它查各台的内存与卡的余量（probe_local、probe_peer：空闲显存、卡上的进程），不看卡坏没坏；读第二台的办法照它（peer_host_run 送 python 源码），判法不同，不并进去
# gate-similar: gpu-quiesce-run.sh 它清场、回读计算进程表，不判卡本身；清场之前调这一份查卡
# gate-similar: agent-watch.py 看门狗只看进程、会话与本机余量；定时调这一份，不在看门狗里另写一套查卡
"""
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import base64  # noqa: E402
import collections  # noqa: E402
import concurrent.futures  # noqa: E402
import importlib.util  # noqa: E402
import inspect  # noqa: E402
import json  # noqa: E402
import math  # noqa: E402
import os  # noqa: E402
import re  # noqa: E402
import shutil  # noqa: E402
import subprocess  # noqa: E402
import sys  # noqa: E402
import tempfile  # noqa: E402
from dataclasses import dataclass, field  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY_ROOT = os.path.normpath(os.path.join(SCRIPT_DIRECTORY, "..", ".."))
# bash 起的三份：多机配置的判法、读配置的共用库、第二台那几段的共用库（写成模块常量）
CONFIGURATION_CHECK = os.path.join(SCRIPT_DIRECTORY, "..", "crash", "crash-point-replay-shard-configuration-check.sh")
CRASH_AMPLIFICATION_LIBRARY = os.path.join(SCRIPT_DIRECTORY, "..", "crash", "crash-amplification-lib.sh")
PEER_HOST_LIB = os.path.join(SCRIPT_DIRECTORY, "peer-host-lib.sh")
MULTI_HOST_ADDRESSES = os.path.join(SCRIPT_DIRECTORY, "multi_host_addresses.py")   # HOSTS 里哪一项是这一台的共用取法（文件头「哪一台查哪几张」）
CARD_KEYS = ("GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED", "GPU_CARDS_BY_PRIORITY")
# 卡的台标签 <HOSTS 里的地址>:<序号>:<额度>：地址吃到最后两个冒号之前（从右边切两刀），IPv6 地址自己带的冒号留在地址里
CARD_ENTRY = re.compile(r"^(?P<address>.+):(?P<index>[0-9]{1,9}):(?P<quota>[0-9]{1,15}[GM])$")
LEFT_SPLIT_CARD_ENTRY = re.compile(r"^(?P<address>[^:]+):(?P<index>[0-9]{1,9}):(?P<quota>[0-9]{1,15}[GM])$")   # 弄坏开关 label-split-from-left
CardLabel = collections.namedtuple("CardLabel", "side address index quota entry")   # side：local 是这一台（HOSTS 里自己那一项），peer 是另一台
FATAL_XIDS = frozenset({48, 62, 74, 79, 94, 95, 119, 120, 154})              # 依据在文件头「致命类 Xid」
RECOVERY_XID = 154
PCI_ENUMERATION = "pci-enumeration"   # 内核日志记录里枚举行那一段的类别（collect_card_readings 里写的同一个词）
HOST_TIMEOUT_SECONDS = 420.0
JOURNAL_SECONDS = 300
NVIDIA_SMI_SECONDS = 30
PROBE_MARK = "GPU_CARD_HEALTH_PROBE "
LINE_MARK = "GPU_CARD_HEALTH"
SUMMARY_MARK = "GPU_CARD_HEALTH_SUMMARY"
EXIT_HEALTHY, EXIT_FAULT, EXIT_USAGE, EXIT_NOTHING_NAMED, EXIT_UNCHECKED = 0, 1, 2, 77, 78
HOST_WORDS = {"local": "本机", "peer": "第二台"}
BREAK_VARIABLE = "GPU_CARD_HEALTH_BREAK"
STATE_DIRECTORY_VARIABLE = "GPU_CARD_HEALTH_STATE_DIRECTORY"
BOOT_ID_FILE_VARIABLE = "GPU_CARD_HEALTH_BOOT_ID_FILE"
HOST_TIMEOUT_VARIABLE = "GPU_CARD_HEALTH_HOST_TIMEOUT_SECONDS"
FAULT_NEXT_STEP_TEMPLATE = ("主 agent 弹窗告诉用户哪台哪张卡与依据（上面逐卡那几行），请用户在那台用 root 重置：{resets}；"
                            "重置不了就重启那台，重启后先跑多机配置 QUIESCE_START_COMMAND 那条私有复原脚本再清场")


def broken(item):
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


# ── 每台上跑的那一段：本机与第二台都把这个函数的源码送进 python3 -c 起（所以只用标准库、不引用本文件别处的名字）──
def collect_card_readings(request):
    """读这一台的三种 nvidia-smi 查询与这次开机内核日志里的 Xid，原样交回（判好坏在起活那一台的 judge_host）。
    request：cards（要 -q -i 的序号）、state_directory / boot_id_file（空串取默认）、nvidia_smi_seconds、journal_seconds、forget_cached_runs（弄坏开关 xid-cache-forgets）、
    accept_unrecorded_cache（弄坏开关 xid-cache-recorded-ignored）。
    内核日志读出的每一段：[域, 总线, 设备, Xid 号或 "pci-enumeration"（枚举行）, 是不是 154 变回 None, 相邻同类几条]。"""
    import json as json_module
    import os as os_module
    import re as re_module
    import subprocess as subprocess_module
    import threading as threading_module

    xid_line = re_module.compile(r"NVRM: Xid \(PCI:([0-9A-Fa-f]+):([0-9A-Fa-f]+):([0-9A-Fa-f]+)\): (\d+)")
    recovered_to_none = re_module.compile(r"to 0x0 \(None\)")
    # 功能号 0 的那一行才算这张卡（同一张卡的声卡是 .1）；开机与摘下扫回时内核打的「pci 0000:01:00.0: [10de:2c02] type 00 class 0x030000 …」
    pci_enumeration_line = re_module.compile(r"\bpci ([0-9A-Fa-f]{4,8}):([0-9A-Fa-f]{2}):([0-9A-Fa-f]{2})\.0: \[[0-9A-Fa-f]{4}:[0-9A-Fa-f]{4}\] type ")
    recorded = "xid,pci-enumeration"   # 记录里记的是哪几种行；读到的记录不是这一串的就整份重读

    def query(arguments):
        try:
            completed = subprocess_module.run(arguments, capture_output=True, text=True, timeout=request["nvidia_smi_seconds"],
                                              stdin=subprocess_module.DEVNULL, errors="replace")
        except (OSError, subprocess_module.TimeoutExpired) as error:
            return {"exit": None, "stdout": "", "error": f"{type(error).__name__}: {error}"[:300]}
        return {"exit": completed.returncode, "stdout": completed.stdout, "error": completed.stderr.strip()[-300:]}

    def scan_journal(cursor):
        """(退出码或 None, 读到的 Xid 与枚举行段, 新游标, 出错说明)：逐行读 journalctl 的输出，只留 Xid 行与枚举行；到点停自己起的这一个。"""
        arguments = ["nice", "-n", "19", "journalctl", "-k", "-b", "-o", "cat", "--no-pager", "--show-cursor"]
        if cursor:
            arguments.append(f"--after-cursor={cursor}")
        try:
            process = subprocess_module.Popen(arguments, stdout=subprocess_module.PIPE, stderr=subprocess_module.PIPE, text=True,
                                              errors="replace", stdin=subprocess_module.DEVNULL)
        except OSError as error:
            return None, [], "", f"journalctl 起不来：{error}"
        timer = threading_module.Timer(request["journal_seconds"], process.kill)
        timer.start()
        runs, new_cursor = [], ""
        try:
            for line in process.stdout:
                if line.startswith("-- cursor: "):
                    new_cursor = line[len("-- cursor: "):].strip()
                    continue
                if "NVRM: Xid" in line:
                    found = xid_line.search(line)
                    if not found:
                        continue
                    number = int(found.group(4))
                    key = [int(found.group(1), 16), int(found.group(2), 16), int(found.group(3), 16), number,
                           number == 154 and bool(recovered_to_none.search(line))]
                elif "] type " in line:
                    found = pci_enumeration_line.search(line)
                    if not found:
                        continue
                    key = [int(found.group(1), 16), int(found.group(2), 16), int(found.group(3), 16), "pci-enumeration", False]
                else:
                    continue
                if runs and runs[-1][:5] == key:
                    runs[-1][5] += 1
                else:
                    runs.append(key + [1])
            error_text = process.stderr.read().strip()[-300:]
            code = process.wait()
        finally:
            timer.cancel()
        if code < 0:
            return code, runs, new_cursor, f"journalctl {request['journal_seconds']} 秒没读完，停了"
        return code, runs, new_cursor, error_text

    def kernel_xid_runs():
        state_directory = request["state_directory"] or os_module.path.join(
            os_module.environ.get("XDG_DATA_HOME") or os_module.path.expanduser("~/.local/share"), "singlefs", "gpu-card-health")
        try:
            with open(request["boot_id_file"] or "/proc/sys/kernel/random/boot_id", encoding="utf-8") as handle:
                boot_id = handle.read().strip()
        except OSError as error:
            return {"ok": False, "error": f"读不出开机号：{error}"}
        cache_path = os_module.path.join(state_directory, f"kernel-xid-{boot_id}.json")
        cached_runs, cursor = [], ""
        try:
            with open(cache_path, encoding="utf-8") as handle:
                cached = json_module.load(handle)
            if isinstance(cached.get("runs"), list) and isinstance(cached.get("cursor"), str) \
                    and (cached.get("recorded") == recorded or request["accept_unrecorded_cache"]):
                cached_runs, cursor = cached["runs"], cached["cursor"]
        except (OSError, ValueError, AttributeError):
            pass
        incremental = bool(cursor)
        code, runs, new_cursor, error_text = scan_journal(cursor)
        if cursor and code != 0:
            incremental, cached_runs, cursor = False, [], ""
            code, runs, new_cursor, error_text = scan_journal("")
        if code != 0:
            return {"ok": False, "error": f"journalctl -k -b 退 {code}：{error_text}"}
        if request["forget_cached_runs"]:
            cached_runs = []
        merged = [list(run) for run in cached_runs]
        for run in runs:
            if merged and merged[-1][:5] == run[:5]:
                merged[-1][5] += run[5]
            else:
                merged.append(run)
        try:
            os_module.makedirs(state_directory, exist_ok=True)
            temporary = f"{cache_path}.{os_module.getpid()}.tmp"
            with open(temporary, "w", encoding="utf-8") as handle:
                json_module.dump({"cursor": new_cursor or cursor, "recorded": recorded, "runs": merged}, handle)
            os_module.replace(temporary, cache_path)
            for name in os_module.listdir(state_directory):
                if name.startswith("kernel-xid-") and name.endswith(".json") and name != os_module.path.basename(cache_path):
                    os_module.remove(os_module.path.join(state_directory, name))
        except OSError as error:
            return {"ok": True, "runs": merged, "incremental": incremental, "note": f"Xid 记录写不进 {state_directory}：{error}"}
        return {"ok": True, "runs": merged, "incremental": incremental, "note": ""}

    readings = {
        "query_gpu": query(["nvidia-smi", "--query-gpu=index,pci.bus_id,temperature.gpu,power.draw", "--format=csv,noheader,nounits"]),
        "query_apps": query(["nvidia-smi", "--query-compute-apps=pid,name,gpu_bus_id,used_memory", "--format=csv,noheader,nounits"]),
        "recovery": {},
    }
    for index in request["cards"]:
        outcome = query(["nvidia-smi", "-q", "-i", str(index)])
        outcome["lines"] = [line.strip() for line in outcome.pop("stdout").splitlines() if "GPU Recovery Action" in line]
        readings["recovery"][str(index)] = outcome
    readings["kernel"] = kernel_xid_runs()
    return readings


def probe_program():
    """送进 python3 -c 的程序：collect_card_readings 的源码，读 argv[1]（base64 的 JSON 请求），打一行 PROBE_MARK 加 JSON。"""
    return (inspect.getsource(collect_card_readings)
            + "\nimport base64 as probe_base64, json as probe_json, sys as probe_sys\n"
            + "probe_request = probe_json.loads(probe_base64.b64decode(probe_sys.argv[1]).decode('utf-8'))\n"
            + f"print({PROBE_MARK!r} + probe_json.dumps(collect_card_readings(probe_request)))\n")


def probe_command_text(request):
    program = base64.b64encode(probe_program().encode("utf-8")).decode("ascii")
    argument = base64.b64encode(json.dumps(request).encode("utf-8")).decode("ascii")
    return f'python3 -B -c "$(printf %s {program} | base64 -d)" {argument}'


def probe_reply(stdout):
    line = next((line for line in reversed(stdout.splitlines()) if line.startswith(PROBE_MARK)), None)
    if line is None:
        return None
    try:
        return json.loads(line[len(PROBE_MARK):])
    except ValueError:
        return None


# ── 多机配置 ──
def read_configuration(environment):
    """(配置文件路径, {键: 值}, 问题)：经 crash-amplification-lib.sh 找文件、读两个卡的键；文件不在时问题非空。"""
    reader = ('source "$1" || exit 70; file="$(crash_amplification_multi_host_configuration_file "$2")"; printf "FILE=%s\\0" "$file"; '
              '[[ -f "$file" ]] || exit 3; shift 2; for key in "$@"; do printf "%s=%s\\0" "$key" "$(crash_amplification_configuration_value "$file" "$key")"; done')
    completed = subprocess.run(["bash", "-c", reader, "gpu-card-health", CRASH_AMPLIFICATION_LIBRARY, REPOSITORY_ROOT, *CARD_KEYS],
                               capture_output=True, text=True, env=environment, stdin=subprocess.DEVNULL)
    values = dict(item.split("=", 1) for item in completed.stdout.split("\0") if "=" in item)
    path = values.pop("FILE", "")
    if completed.returncode == 3:
        return path, {}, f"多机配置 {path or '（取不到路径）'} 不在"
    if completed.returncode != 0:
        return path, {}, f"读多机配置那一步退 {completed.returncode}：{completed.stderr.strip()[-200:]}"
    return path, values, ""


def card_labels(text, addresses):
    """多机配置一个显卡键的值 → ([CardLabel…], 问题)：每项 <HOSTS 里的地址>:<序号>:<额度>，从右边切两刀（地址可以是 IPv6）；
    addresses 是 Addresses(this, peer)（multi_host_addresses.resolve），地址是 this 的那一项 side 是 local、是 peer 的 side 是 peer。
    写法不对、额度是 0、地址不在 HOSTS 里的那一项不收，问题写头一处。task-server/task_server.py 的 parse_cards 调同一份。
    弄坏开关 label-split-from-left 下从左边切；label-outside-hosts-accepted 下地址不在 HOSTS 里的也收、当另一台的。"""
    labels, problem = [], ""
    pattern = LEFT_SPLIT_CARD_ENTRY if broken("label-split-from-left") else CARD_ENTRY
    sides = dict([(addresses.this, "local"), (addresses.peer, "peer")])
    for entry in (text or "").split():
        found = pattern.match(entry)
        if not found or int(found.group("quota")[:-1]) == 0:
            problem = problem or f"「{entry}」写法不对（要 <HOSTS 里那一台的地址>:<序号>:<额度，正整数加 G 或 M>，例 192.0.2.1:0:32G）"
            continue
        side = sides.get(found.group("address"))
        if side is None:
            if not broken("label-outside-hosts-accepted"):
                problem = problem or f"「{entry}」的台标签 {found.group('address')} 不是 HOSTS 里的一项（台标签写 HOSTS 里那一台的地址）"
                continue
            side = "peer"
        labels.append(CardLabel(side, found.group("address"), int(found.group("index")), found.group("quota"), entry))
    return labels, problem


def named_cards(values, addresses):
    """({'local': [序号…], 'peer': [序号…]}, 问题)：两个键并起来、按（台、序号）去重、序号从小到大；有一项写坏就交问题（文件头「查哪几张卡」）。"""
    named = {"local": set(), "peer": set()}
    for key in CARD_KEYS:
        labels, problem = card_labels(values.get(key) or "", addresses)
        if problem:
            return {}, f"多机配置的 {key} 里{problem}"
        for label in labels:
            named[label.side].add(label.index)
    return {side: sorted(cards) for side, cards in named.items()}, ""


def addresses_library():
    """scripts/hosts/multi_host_addresses.py（HOSTS 里哪一项是这一台的共用取法）：照文件路径载入，一个进程只载入一次。"""
    if "multi_host_addresses" not in sys.modules:
        specification = importlib.util.spec_from_file_location("multi_host_addresses", MULTI_HOST_ADDRESSES)
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        sys.modules["multi_host_addresses"] = module
    return sys.modules["multi_host_addresses"]


def host_addresses(configuration_path, environment):
    """(Addresses(this, peer) 或 None, 问题)：这一台与另一台在 HOSTS 里的地址，经 multi_host_addresses.resolve 判（不另写判法）。
    弄坏开关 this-side-by-order 下照 HOSTS 的次序，头一项当这一台。"""
    library = addresses_library()
    if broken("this-side-by-order"):
        items = (library.configuration_value(configuration_path, "HOSTS") or "").split()
        if len(items) >= 2:
            return library.Addresses(items[0], items[1]), ""
    try:
        return library.resolve(configuration_path, environment), ""
    except library.AddressesUnavailable as error:
        return None, f"认不出多机配置 HOSTS 里哪一项是这一台（{error}）"


BY_HOST_UNREADABLE_EXIT = 4   # peer_connection 起的那一层：判法过了而 peer_host_load_by_host 取不到按台的键那一项（与判法的 1、3 分开）
PEER_CONNECTION_NAMES = ("PEER_HOST_ADDRESS", "PEER_HOST_CARGO_BIN_DIRECTORY", "PEER_HOST_BUILD_ROOT", "THIS_HOST_ADDRESS", "THIS_HOST_CARGO_BIN_DIRECTORY",
                         "THIS_HOST_BUILD_ROOT")


def peer_connection(environment):
    """({PEER_CONNECTION_NAMES 各一项}, 问题)：判法 --emit-hosts 的赋值（不连第二台）经 bash eval，再经 peer-host-lib.sh 的 peer_host_load_by_host 照多机配置的
    按台的键取两台各自那一项（CARGO_BIN_DIRECTORY_BY_HOST、BUILD_ROOT_BY_HOST，按地址取），按 NUL 带回来；第二台的地址照主机数组取（判法文件头「主机数组」：
    peer-host-lib.sh 的 peer_host_resolve_addresses）；第二台的 cargo 目录给 peer_host_run 接进 PATH，两台的目录与这一台的地址只给 peer_host_redact 换字用。"""
    loader = ('source "$1" || exit 70; assignments="$(bash "$2" --emit-hosts "$3")"; status=$?; if ((status != 0)); then exit "$status"; fi; eval "$assignments"; '
              f'peer_host_load_by_host >&2 || exit {BY_HOST_UNREADABLE_EXIT}; shift 3; for name in "$@"; do printf "%s\\0" "${{!name}}"; done')
    completed = subprocess.run(["bash", "-c", loader, "gpu-card-health", PEER_HOST_LIB, CONFIGURATION_CHECK, REPOSITORY_ROOT, *PEER_CONNECTION_NAMES],
                               capture_output=True, text=True, env=environment, stdin=subprocess.DEVNULL)
    if completed.returncode == 3:
        return {}, "多机配置点名了另一台的卡，而跨机开关 ENABLE_ACROSS_MACHINES 没开，连不了另一台"
    if completed.returncode != 0:
        # 退 BY_HOST_UNREADABLE_EXIT 是判法过了、peer_host_load_by_host 取不到那一项：它打的那一句（只点名键与哪一台）接在后面
        reason = next((line.strip() for line in completed.stderr.splitlines() if "✗" in line), "")
        return {}, (f"多机配置判不过（判法 {os.path.basename(CONFIGURATION_CHECK)} --emit-hosts 与按台的键那一层退 {completed.returncode}"
                    f"{'：' + reason if reason else ''}；单跑 bash scripts/crash/crash-point-replay-shard-configuration-check.sh 看原因）")
    fields = completed.stdout.split("\0")
    connection = {name: (fields[position] if position < len(fields) else "") for position, name in enumerate(PEER_CONNECTION_NAMES)}
    if not connection["PEER_HOST_ADDRESS"]:
        return {}, "多机配置判得过，却没解出第二台的地址（PEER_HOST_ADDRESS）"
    return connection, ""


def probe_request(environment, side, cards):
    state_directory = environment.get(STATE_DIRECTORY_VARIABLE, "")
    return {"cards": cards, "state_directory": os.path.join(state_directory, side) if state_directory else "",
            "boot_id_file": environment.get(BOOT_ID_FILE_VARIABLE, ""), "nvidia_smi_seconds": NVIDIA_SMI_SECONDS,
            "journal_seconds": JOURNAL_SECONDS, "forget_cached_runs": broken("xid-cache-forgets"),
            "accept_unrecorded_cache": broken("xid-cache-recorded-ignored")}


def host_timeout(environment):
    try:
        seconds = float(environment.get(HOST_TIMEOUT_VARIABLE) or HOST_TIMEOUT_SECONDS)
    except ValueError:
        return HOST_TIMEOUT_SECONDS
    return seconds if seconds > 0 else HOST_TIMEOUT_SECONDS


def probe_on_this_machine(environment, request):
    """(读数, 问题)：本机起一份 python3 -c（环境照 environment，自证的假 nvidia-smi、journalctl 在它的 PATH 前面）。"""
    try:
        completed = subprocess.run(["bash", "-c", probe_command_text(request)], capture_output=True, text=True, env=environment,
                                   stdin=subprocess.DEVNULL, timeout=host_timeout(environment))
    except subprocess.TimeoutExpired:
        return None, f"本机 {host_timeout(environment):g} 秒没查完"
    readings = probe_reply(completed.stdout)
    if completed.returncode != 0 or readings is None:
        return None, f"本机那一份退 {completed.returncode}：{completed.stderr.strip()[-200:]}"
    return readings, ""


def probe_on_peer(environment, request, connection):
    """(读数, 问题)：经 peer-host-lib.sh 的 peer_host_run 在第二台起同一份 python3 -c；连不上、到点、回来的不是那一行都算连不上。"""
    runner = ('source "$1" || exit 70; command_text="$2"; shift 2; for name in ' + " ".join(PEER_CONNECTION_NAMES)
              + '; do printf -v "$name" "%s" "$1"; shift; done; peer_host_run / "$command_text"')
    try:
        completed = subprocess.run(["bash", "-c", runner, "gpu-card-health", PEER_HOST_LIB, probe_command_text(request),
                                    *(connection[name] for name in PEER_CONNECTION_NAMES)],
                                   capture_output=True, text=True, env=environment, stdin=subprocess.DEVNULL, timeout=host_timeout(environment))
    except subprocess.TimeoutExpired:
        return None, f"第二台 {host_timeout(environment):g} 秒没回话"
    readings = probe_reply(completed.stdout)
    if completed.returncode != 0 or readings is None:
        tail = completed.stderr.strip().splitlines()[-1][-200:] if completed.stderr.strip() else ""
        return None, f"第二台连不上或没回读数（退 {completed.returncode}{'：' + tail if tail else ''}）"
    return readings, ""


# ── 判好坏（纯函数）──
@dataclass
class CardVerdict:
    side: str   # local 是这一台、peer 是另一台（给人看的话用）
    host: str   # 那张卡的台标签：HOSTS 里那一台的地址（机读行的 host=）
    card: str
    bus: str
    faults: list = field(default_factory=list)
    readings: list = field(default_factory=list)
    counted: dict = field(default_factory=dict)

    def machine_line(self):
        state = "fault" if self.faults else "ok"
        evidence = ",".join(self.faults if self.faults else self.readings) or "none"
        counted = ",".join(f"xid-{number}:{count}" for number, count in sorted(self.counted.items())) or "none"
        return f"{LINE_MARK} host={self.host} card={self.card} bus={self.bus} state={state} evidence={evidence} counted={counted}"


def evidence_token(text):
    return re.sub(r"\s+", "_", text.strip()) or "empty"


def bus_address(text):
    """nvidia-smi 的总线（00000000:01:00.0）→ (域, 总线, 设备)；读不出交 None。"""
    found = re.fullmatch(r"\s*([0-9A-Fa-f]{1,8}):([0-9A-Fa-f]{1,2}):([0-9A-Fa-f]{1,2})(?:\.[0-7])?\s*", text or "")
    return (int(found.group(1), 16), int(found.group(2), 16), int(found.group(3), 16)) if found else None


def is_reading_number(text):
    try:
        return math.isfinite(float(text))
    except ValueError:
        return False


def csv_rows(text):
    return [[part.strip() for part in line.split(",")] for line in text.splitlines() if line.strip()]


def query_failure(prefix, outcome):
    return f"{prefix}nvidia-smi-not-started" if outcome.get("exit") is None else f"{prefix}nvidia-smi-exit={outcome['exit']}"


def judge_host(side, address, cards, readings):
    """→ ([CardVerdict], 问题)：问题非空是这台有一部分没查成（内核日志读不出）。side 是 local / peer，address 是这台在 HOSTS 里的地址。"""
    verdicts = []
    query_gpu = readings["query_gpu"]
    gpu_failed = query_gpu.get("exit") != 0 and not broken("nvidia-smi-failure-trusted")
    rows = {}
    if query_gpu.get("exit") == 0:
        for row in csv_rows(query_gpu.get("stdout", "")):
            if len(row) >= 4 and row[0].isdigit():
                rows[int(row[0])] = row
    apps = readings["query_apps"]
    apps_failed = apps.get("exit") != 0 and not broken("nvidia-smi-failure-trusted")
    unknown_pid_rows = [] if broken("unknown-pid-ignored") or apps.get("exit") != 0 else \
        [row for row in csv_rows(apps.get("stdout", "")) if row and not row[0].isdigit()]
    kernel = readings["kernel"]
    kernel_problem = ""
    if not kernel.get("ok") and not broken("kernel-log-failure-trusted"):
        kernel_problem = f"内核日志读不出（{kernel.get('error', '')}）"
    runs = kernel.get("runs") or []
    attributed_unknown = 0
    for index in cards:
        verdict = CardVerdict(side, address, str(index), "unknown")
        if gpu_failed:
            verdict.faults.append(query_failure("", query_gpu))
        row = rows.get(index)
        if row is None:
            if query_gpu.get("exit") == 0 and not broken("missing-card-trusted"):
                verdict.faults.append("missing-from-nvidia-smi")
        else:
            verdict.bus = row[1]
            for label, reading in (("temperature", row[2]), ("power", row[3])):
                if is_reading_number(reading) or broken("readings-trusted"):
                    verdict.readings.append(f"{label}={evidence_token(reading)}")
                else:
                    verdict.faults.append(f"{label}={evidence_token(reading)}")
        recovery = readings["recovery"].get(str(index), {"exit": None, "lines": []})
        if recovery.get("exit") != 0:
            if row is not None and not broken("nvidia-smi-failure-trusted"):
                verdict.faults.append(query_failure("query-", recovery))
        else:
            actions = [line.split(":", 1)[1].strip() for line in recovery.get("lines", []) if ":" in line]
            not_none = [action for action in actions if action != "None"]
            if not actions:
                verdict.readings.append("recovery-action=unreported")
            elif not_none and not broken("recovery-action-ignored"):
                verdict.faults.append(f"recovery-action={evidence_token(not_none[0])}")
            else:
                verdict.readings.append("recovery-action=None")
        if apps_failed:
            verdict.faults.append(query_failure("apps-", apps))
        card_bus = bus_address(verdict.bus)
        if card_bus is not None:
            on_this_card = [row for row in unknown_pid_rows if len(row) >= 3 and bus_address(row[2]) == card_bus]
            attributed_unknown += len(on_this_card)
            if on_this_card:
                verdict.faults.append(f"compute-app-pid-unknown={len(on_this_card)}")
            fatal = {}
            # 这张卡最后一次恢复（154 变回 None、这张卡总线上的枚举行）之前的 Xid 不判：照出现次序走，遇到恢复就清掉已经记下的
            rescanned_anywhere = broken("pci-rescan-order-ignored") and any(
                run[3] == PCI_ENUMERATION and tuple(run[:3]) == card_bus for run in runs)
            for domain, bus, device, number, recovered, count in runs:
                enumeration = number == PCI_ENUMERATION
                if (domain, bus, device) != card_bus and not (enumeration and broken("pci-rescan-any-bus")):
                    continue
                if enumeration:
                    if not broken("pci-rescan-ignored"):
                        fatal, verdict.counted = {}, {}
                    continue
                if rescanned_anywhere:
                    continue
                if number == RECOVERY_XID and recovered and not broken("xid-recovery-ignored"):
                    fatal, verdict.counted = {}, {}
                    continue
                if (number in FATAL_XIDS and not broken("fatal-xid-ignored")) or broken("application-xid-fatal"):
                    fatal[number] = fatal.get(number, 0) + count
                else:
                    verdict.counted[number] = verdict.counted.get(number, 0) + count
            verdict.faults.extend(f"xid-{number}={count}" for number, count in sorted(fatal.items()))
        verdicts.append(verdict)
    unattributed = [row for row in unknown_pid_rows if not (len(row) >= 3 and bus_address(row[2]) is not None)]
    if unattributed:
        verdicts.append(CardVerdict(side, address, "unknown", "unknown", [f"compute-app-pid-unknown-bus-unknown={len(unattributed)}"]))
    return verdicts, kernel_problem


EVIDENCE_WORDS = (
    ("query-nvidia-smi-", "nvidia-smi -q -i 读不出"), ("apps-nvidia-smi-", "nvidia-smi 的计算进程表读不出"), ("nvidia-smi-", "nvidia-smi 起不来或读不出"),
    ("missing-from-nvidia-smi", "nvidia-smi 里没有这张卡"), ("temperature=", "温度读成"), ("power=", "功耗读成"),
    ("recovery-action=", "GPU Recovery Action 是"), ("compute-app-pid-unknown-bus-unknown=", "计算进程表里 pid 与总线都是 [N/A] 的行"),
    ("compute-app-pid-unknown=", "计算进程表里这张卡总线上 pid 为 [N/A] 的行"), ("xid-", "这次开机后内核报的致命类 Xid "),
)


def evidence_in_words(item):
    for prefix, words in EVIDENCE_WORDS:
        if item.startswith(prefix):
            return f"{words} {item[len(prefix):]}".strip()
    return item


def card_in_words(verdict):
    if verdict.card == "unknown":
        return f"{HOST_WORDS[verdict.side]}（归不到哪张卡）"
    return f"{HOST_WORDS[verdict.side]} {verdict.card} 号卡（总线 {verdict.bus}）"


def fault_next_step(faults):
    resets = []
    for verdict in faults:
        if verdict.card == "unknown":
            resets.append(f"{HOST_WORDS[verdict.side]}先跑 nvidia-smi 看哪张卡读不出，再 sudo nvidia-smi -r -i <那张卡的序号>")
        else:
            resets.append(f"{HOST_WORDS[verdict.side]} sudo nvidia-smi -r -i {verdict.card}")
    return FAULT_NEXT_STEP_TEMPLATE.format(resets="、".join(dict.fromkeys(resets)))


# ── 主流程 ──
def assigned_named_cards(assigned, addresses):
    """--cards 的值 → ({'local': [序号…], 'peer': [序号…]}, 问题)：写法与认法同 named_cards 的一个键（文件头「查哪几张卡」）。"""
    labels, problem = card_labels(assigned, addresses)
    if problem:
        return {}, f"--cards 里{problem}"
    named = {"local": set(), "peer": set()}
    for label in labels:
        named[label.side].add(label.index)
    return {side: sorted(cards) for side, cards in named.items()}, ""


def check_cards(environment, assigned=None):
    """→ (退出码, 输出的行)。assigned 是 --cards 的值（只查这几张）；None 是查多机配置点名的全部卡。"""
    output = []
    if broken("assigned-cards-ignored"):
        assigned = None
    path, values, problem = read_configuration(environment)
    if problem and broken("missing-configuration-as-empty"):
        values, problem = {}, ""
    if problem:
        return finish(output, [], {"配置": problem}, 0)
    if assigned is None and not any((values.get(key) or "").split() for key in CARD_KEYS):
        output.append(f"{SUMMARY_MARK} state=nothing-named checked=0 fault=0 unchecked=none")
        output.append(f"  · 卡状态：多机配置 {path} 的 {'、'.join(CARD_KEYS)} 一张卡都没点名，没查")
        return (EXIT_HEALTHY if broken("empty-as-healthy") else EXIT_NOTHING_NAMED), output
    addresses, problem = host_addresses(path, environment)
    if problem:
        return finish(output, [], {"配置": problem}, 0)
    named, problem = named_cards(values, addresses) if assigned is None else assigned_named_cards(assigned, addresses)
    if problem:
        return finish(output, [], {"配置": problem}, 0)
    sides = ["local", "peer"]
    side_addresses = {"local": addresses.this, "peer": addresses.peer}
    targets = {side: named[side] for side in sides if named[side]}
    unchecked, jobs = {}, {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(targets)) as pool:
        for side, cards in targets.items():
            request = probe_request(environment, side, cards)
            if side == "local":
                jobs[side] = pool.submit(probe_on_this_machine, environment, request)
                continue
            connection, problem = peer_connection(environment)
            if problem:
                unchecked[side] = problem
                continue
            jobs[side] = pool.submit(probe_on_peer, environment, request, connection)
        collected = {side: job.result() for side, job in jobs.items()}
    if len(collected) + len(unchecked) != len(targets):
        unchecked["收回"] = f"派出 {len(targets)} 台、收回 {len(collected) + len(unchecked)} 台"
    verdicts = []
    for side in sides:
        if side not in collected:
            continue
        readings, problem = collected[side]
        if readings is None:
            if side == "peer" and broken("peer-unreachable-trusted"):
                continue
            unchecked[side] = problem
            continue
        host_verdicts, kernel_problem = judge_host(side, side_addresses[side], targets[side], readings)
        verdicts.extend(host_verdicts)
        if kernel_problem:
            unchecked[side] = kernel_problem
    return finish(output, verdicts, {HOST_WORDS.get(side, side): reason for side, reason in unchecked.items()},
                  sum(len(cards) for side, cards in targets.items() if side in collected))


def finish(output, verdicts, unchecked, checked):
    faults = [verdict for verdict in verdicts if verdict.faults]
    output.extend(verdict.machine_line() for verdict in verdicts)
    state = "fault" if faults else ("unchecked" if unchecked else "ok")
    unchecked_field = ",".join(f"{host}:{evidence_token(reason)}" for host, reason in unchecked.items()) or "none"
    output.append(f"{SUMMARY_MARK} state={state} checked={checked} fault={len(faults)} unchecked={unchecked_field}")
    for verdict in faults:
        output.append(f"  ✗ 显卡状态不对：{card_in_words(verdict)}：{'；'.join(evidence_in_words(item) for item in verdict.faults)}")
    if faults:
        output.append(f"  → 怎么办：{fault_next_step(faults)}")
    for host, reason in unchecked.items():
        output.append(f"  ✗ 卡状态查不成（{host}）：{reason}")
    if unchecked:
        output.append("  → 怎么办：另一台连不上就先修 ssh 免密登录（ssh -o BatchMode=yes <多机配置 HOSTS 里另一台那一项> true 退 0）；配置读不出照仓根 multi-host.env.example 改"
                      "（卡的台标签写 HOSTS 里那一台的地址）；"
                      "内核日志读不出看那台 journalctl -k -b 能不能读（用户要在 adm 或 systemd-journal 组里）；修好再跑 python3 scripts/hosts/gpu-card-health.py")
    if not faults and not unchecked:
        output.append(f"  ✓ 卡状态：查了 {checked} 张点名的卡，都好")
    if faults:
        return EXIT_FAULT, output
    return (EXIT_UNCHECKED if unchecked else EXIT_HEALTHY), output


def main(arguments, environment=None, output=None):
    output = output or sys.stdout
    assigned = None
    if len(arguments) == 2 and arguments[0] == "--cards" and arguments[1].split():
        assigned = arguments[1]
    elif arguments:
        print(f"  ✗ 用法：gpu-card-health.py 不带参数、--cards <显卡先后>（至少一项）或 --selftest，读到「{' '.join(arguments)}」", file=output)
        print("  → 怎么办：查多机配置点名的全部卡直接跑 python3 scripts/hosts/gpu-card-health.py；只查几张写 --cards '<HOSTS 里的地址>:<序号>:<额度> …'", file=output)
        return EXIT_USAGE
    code, lines = check_cards(dict(os.environ) if environment is None else environment, assigned)
    for line in lines:
        print(line, file=output)
    return code


# ── 自证 ──
FAKE_NVIDIA_SMI = """#!/usr/bin/env bash
# 假 nvidia-smi：读同目录的 gpus.csv、apps.csv、q-<序号>.txt；有 fail 就照驱动连不上退 9；每次调用记一行进 calls.log
here="$(dirname "$0")"
echo "nvidia-smi $*" >> "$here/calls.log"
if [[ -e "$here/fail" ]]; then echo "NVIDIA-SMI has failed because it couldn't communicate with the NVIDIA driver."; exit 9; fi
case "$*" in
  *--query-gpu*) cat "$here/gpus.csv" ;;
  *--query-compute-apps*) cat "$here/apps.csv" ;;
  "-q -i "*) index="${@: -1}"; if [[ -f "$here/q-$index.txt" ]]; then cat "$here/q-$index.txt"; else echo "No devices were found"; exit 6; fi ;;
  *) exit 9 ;;
esac
"""
FAKE_JOURNALCTL = """#!/usr/bin/env bash
# 假 journalctl：kernel.log 一行一条，游标写 line-<读到第几行>；--after-cursor=line-<k> 只打第 k 行之后的；认不得的游标照真的退 1；有 journal-fail 就退 1
here="$(dirname "$0")"
echo "journalctl $*" >> "$here/calls.log"
if [[ -e "$here/journal-fail" ]]; then echo "No journal files were opened due to insufficient permissions." >&2; exit 1; fi
after=0
for argument in "$@"; do
  case "$argument" in
    --after-cursor=line-*) after="${argument#--after-cursor=line-}" ;;
    --after-cursor=*) echo "Failed to seek to cursor: Invalid argument" >&2; exit 1 ;;
  esac
done
total="$(wc -l < "$here/kernel.log")"
tail -n "+$((after + 1))" "$here/kernel.log"
if (( total > after )); then echo "-- cursor: line-$total"; fi
"""
FAKE_SSH = """#!/usr/bin/env bash
# 假 ssh：去掉 -o 选项，按主机名找 hosts/<名>/；有 unreachable 就照连不上退 255；命令照 ssh 那样交给 bash -c，PATH 前面换成那台的假 nvidia-smi、journalctl
while [[ "${1:-}" == -o ]]; do shift 2; done
host="$1"; shift
printf '%s\\n' "$host" >> "$GPU_CARD_HEALTH_SELFTEST_ROOT/ssh.log"
here="$GPU_CARD_HEALTH_SELFTEST_ROOT/hosts/$host"
if [[ ! -d "$here" || -e "$here/unreachable" ]]; then echo "ssh: connect to host $host port 22: Connection refused" >&2; exit 255; fi
exec env PATH="$here/bin:$PATH" bash -c "$*"
"""
SELFTEST_THIS_ADDRESS = "127.0.0.1"   # 自证的多机配置里这一台的地址（在每一台的网卡上）
SELFTEST_PEER_ALIAS = "192.0.2.63"     # 自证的多机配置里另一台的地址（文档用的地址段，不在任何一台的网卡上）
SELFTEST_CARDS_KEYS = (f"GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='{SELFTEST_THIS_ADDRESS}:0:32G {SELFTEST_PEER_ALIAS}:0:16G {SELFTEST_THIS_ADDRESS}:1:16G'\n"
                       f"GPU_CARDS_BY_PRIORITY='{SELFTEST_THIS_ADDRESS}:0:15G {SELFTEST_PEER_ALIAS}:0:10G'\n")
SIDE_ADDRESSES = {"local": SELFTEST_THIS_ADDRESS, "peer": SELFTEST_PEER_ALIAS}   # 自证各格按 local / peer 找机读行时换成的台标签
FAKE_IP = """#!/usr/bin/env bash
# 假 ip（second-host-view）：-o addr show 打这一台网卡上只有另一台的地址，演同一份配置在另一台上判
printf '1: lo    inet %s/32 scope host lo\\       valid_lft forever preferred_lft forever\\n' "$GPU_CARD_HEALTH_SELFTEST_VIEW_ADDRESS"
"""
HEALTHY_CARDS = "0, 00000000:01:00.0, 30, 17.66\n1, 00000000:10:00.0, 29, 9.35\n"
XID_FATAL_LINES = ("NVRM: Xid (PCI:0000:01:00): 62, pid=298553, name=selftest_binary, Info 0x0\n"
                   "NVRM: Xid (PCI:0000:01:00): 154, GPU recovery action changed from 0x0 (None) to 0x1 (GPU Reset Required)\n")
PCI_RESCAN_LINE = "pci 0000:01:00.0: [10de:2c02] type 00 class 0x030000 PCIe Legacy Endpoint\n"   # 0 号卡（PCI 总线号 01）开机与扫回时内核打的枚举行
XID_APPLICATION_LINE = "NVRM: Xid (PCI:0000:01:00): 109, pid=4242, name=selftest_binary, channel 0x00000004, errorString CTX SWITCH TIMEOUT, Info 0xc004\n"


class SelftestWorld:
    """一格一个：临时目录里的本机与第二台（各一份假 nvidia-smi、journalctl 与数据文件）、假 ssh、多机配置、Xid 记录目录与开机号。"""

    def __init__(self, root, name):
        self.directory = os.path.join(root, name)
        self.local = os.path.join(self.directory, "local", "bin")
        self.peer = os.path.join(self.directory, "hosts", SELFTEST_PEER_ALIAS, "bin")
        for host in (self.local, self.peer):
            os.makedirs(host)
            self.write(os.path.join(host, "nvidia-smi"), FAKE_NVIDIA_SMI, executable=True)
            self.write(os.path.join(host, "journalctl"), FAKE_JOURNALCTL, executable=True)
            self.write(os.path.join(host, "gpus.csv"), HEALTHY_CARDS)
            self.write(os.path.join(host, "apps.csv"), "")
            self.write(os.path.join(host, "kernel.log"), "")
            for index in (0, 1):
                self.write(os.path.join(host, f"q-{index}.txt"), f"GPU 00000000:{index:02X}:00.0\n    GPU Recovery Action                : None\n")
        os.makedirs(os.path.join(self.directory, "ssh-bin"))
        self.write(os.path.join(self.directory, "ssh-bin", "ssh"), FAKE_SSH, executable=True)
        self.write(os.path.join(self.directory, "boot-id"), "selftest-boot-one\n")
        self.configuration()

    def second_host_view(self):
        """演在另一台上跑（second-host-view）：PATH 前面换成另一台那一份假卡与假 ip（网卡上只有 SELFTEST_PEER_ALIAS），
        假 ssh 连 127.0.0.1 时走到本机那一份假卡（hosts/127.0.0.1 链到 local/）。交这一趟的环境改动。"""
        view = os.path.join(self.directory, "second-view-bin")
        os.makedirs(view, exist_ok=True)
        self.write(os.path.join(view, "ip"), FAKE_IP, executable=True)
        link = os.path.join(self.directory, "hosts", SELFTEST_THIS_ADDRESS)
        if not os.path.lexists(link):
            os.symlink(os.path.join(self.directory, "local"), link)
        return {"PATH": os.pathsep.join([self.peer, view, os.path.join(self.directory, "ssh-bin"), os.environ.get("PATH", "")]),
                "GPU_CARD_HEALTH_SELFTEST_VIEW_ADDRESS": SELFTEST_PEER_ALIAS}

    @staticmethod
    def write(path, text, executable=False, append=False):
        with open(path, "a" if append else "w", encoding="utf-8") as handle:
            handle.write(text)
        if executable:
            os.chmod(path, 0o755)

    def configuration(self, across_machines=True, cards=SELFTEST_CARDS_KEYS):
        self.write(os.path.join(self.directory, "multi-host.env"),
                   f"HOSTS={SELFTEST_THIS_ADDRESS} {SELFTEST_PEER_ALIAS}\n"
                   f"BUILD_ROOT_BY_HOST={SELFTEST_THIS_ADDRESS}=/selftest/this/home {SELFTEST_PEER_ALIAS}=/selftest/peer/home\n"
                   f"CARGO_BIN_DIRECTORY_BY_HOST={SELFTEST_THIS_ADDRESS}=/selftest/this/bin {SELFTEST_PEER_ALIAS}=/selftest/peer/bin\n"
                   f"SLICE_MEMORY_CAP_BY_HOST={SELFTEST_THIS_ADDRESS}=50G {SELFTEST_PEER_ALIAS}=16G\n"
                   "QUIESCE_STOP_COMMAND=true\nQUIESCE_STOPPED_CHECK_COMMAND=true\nQUIESCE_START_COMMAND=true\n"
                   f"QUIESCE_STARTED_CHECK_COMMAND=true\nENABLE_ACROSS_MACHINES={1 if across_machines else 0}\nENABLE_GPU=1\n"
                   f"NFS_SHARED_ROOTS={self.directory}\nSHARED_DATA_ROOT={self.directory}/data-root\n" + cards)

    def environment(self, **changes):
        environment = {name: value for name, value in os.environ.items()
                       if not name.startswith(("GPU_CARD_HEALTH_", "SINGLEFS_", "GPU_CARDS_", "MULTI_HOST_", "PEER_HOST_LIB_"))
                       or name == BREAK_VARIABLE}
        if os.environ.get("MULTI_HOST_CONFIGURATION_BREAK"):
            environment["MULTI_HOST_CONFIGURATION_BREAK"] = os.environ["MULTI_HOST_CONFIGURATION_BREAK"]
        environment.update({
            "PATH": os.pathsep.join([self.local, os.path.join(self.directory, "ssh-bin"), os.environ.get("PATH", "")]),
            "SINGLEFS_MULTI_HOST_CONFIG": os.path.join(self.directory, "multi-host.env"),
            STATE_DIRECTORY_VARIABLE: os.path.join(self.directory, "state"),
            BOOT_ID_FILE_VARIABLE: os.path.join(self.directory, "boot-id"),
            "GPU_CARD_HEALTH_SELFTEST_ROOT": self.directory,
            HOST_TIMEOUT_VARIABLE: "60",
        })
        environment.update(changes)
        return environment

    def ssh_calls(self):
        path = os.path.join(self.directory, "ssh.log")
        return open(path, encoding="utf-8").read().split() if os.path.isfile(path) else []

    def calls(self, host):
        path = os.path.join(host, "calls.log")
        return open(path, encoding="utf-8").read().splitlines() if os.path.isfile(path) else []


def lines_of(lines, host, card):
    """机读行里台标签是 host 的那一张：host 写 local / peer 的换成 SIDE_ADDRESSES 里的地址，写地址的照原样。"""
    return [line for line in lines if line.startswith(f"{LINE_MARK} host={SIDE_ADDRESSES.get(host, host)} card={card} ")]


def selftest_cells(root):
    """各格：交 (失败的格名列表, 查了几格)。每格自己一个 SelftestWorld，互不相干。"""
    failures, checked = [], 0

    def check(cell, condition, detail):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append(cell)
            print(f"  ✗ 自证 {cell}：{detail}")   # gate-lint:detail

    def run(world, **changes):
        return check_cards(world.environment(**changes))

    def has_fault(lines, host, card, evidence):
        return any("state=fault" in line and evidence in line.split(" evidence=", 1)[1].split(" ", 1)[0] for line in lines_of(lines, host, card))

    def is_ok(lines, host, card):
        found = lines_of(lines, host, card)
        return len(found) == 1 and " state=ok " in found[0]

    def text(lines):
        return " | ".join(lines)[-900:]

    # healthy：点名的三张卡（本机 0、1，第二台 0）都好，退 0；读数来自假的 nvidia-smi 与 journalctl，第二台只经假 ssh 连了一次
    world = SelftestWorld(root, "healthy")
    code, lines = run(world)
    check("healthy", code == EXIT_HEALTHY and is_ok(lines, "local", 0) and is_ok(lines, "local", 1) and is_ok(lines, "peer", 0)
          and not lines_of(lines, "peer", 1) and world.ssh_calls() == [SELFTEST_PEER_ALIAS]
          and any("--query-gpu" in call for call in world.calls(world.local)) and any("journalctl" in call for call in world.calls(world.peer))
          and f"{SUMMARY_MARK} state=ok checked=3 fault=0 unchecked=none" in lines and "evidence=temperature=30,power=17.66,recovery-action=None" in text(lines),
          f"应当三张卡都好、退 0、第二台经假 ssh 连一次（退 {code}，ssh {world.ssh_calls()}）：{text(lines)}")

    # nvidia-smi-fails：本机 nvidia-smi 退 9 ⇒ 本机两张卡都坏（nvidia-smi-exit=9），第二台照好，退 1
    world = SelftestWorld(root, "nvidia-smi-fails")
    world.write(os.path.join(world.local, "fail"), "")
    code, lines = run(world)
    check("nvidia-smi-fails", code == EXIT_FAULT and has_fault(lines, "local", 0, "nvidia-smi-exit=9") and has_fault(lines, "local", 1, "nvidia-smi-exit=9")
          and is_ok(lines, "peer", 0), f"本机 nvidia-smi 退 9 应当判本机两张卡坏、退 1（退 {code}）：{text(lines)}")

    # missing-card：本机 --query-gpu 里没有 1 号卡 ⇒ 1 号卡坏（missing-from-nvidia-smi）
    world = SelftestWorld(root, "missing-card")
    world.write(os.path.join(world.local, "gpus.csv"), HEALTHY_CARDS.splitlines()[0] + "\n")
    os.remove(os.path.join(world.local, "q-1.txt"))
    code, lines = run(world)
    check("missing-card", code == EXIT_FAULT and has_fault(lines, "local", 1, "missing-from-nvidia-smi") and is_ok(lines, "local", 0),
          f"少了点名的本机 1 号卡应当判它坏、退 1（退 {code}）：{text(lines)}")

    # 温度、功耗读成四种坏值（各一格）；temperature-err 那一格另核给人看的话与弹窗出路
    for cell, host_side, row in (("temperature-err", "peer", "0, 00000000:01:00.0, ERR!, 17.66"),
                                 ("power-na", "local", "0, 00000000:01:00.0, 30, [N/A]"),
                                 ("temperature-unknown-error", "local", "0, 00000000:01:00.0, [Unknown Error], 17.66"),
                                 ("power-requires-reset", "peer", "0, 00000000:01:00.0, 30, [GPU requires reset]")):
        world = SelftestWorld(root, cell)
        world.write(os.path.join(world.peer if host_side == "peer" else world.local, "gpus.csv"), row + "\n" + HEALTHY_CARDS.splitlines()[1] + "\n")
        code, lines = run(world)
        label = "temperature=" if cell.startswith("temperature") else "power="
        wanted = label + evidence_token(row.split(", ")[2 if label == "temperature=" else 3])
        condition = code == EXIT_FAULT and has_fault(lines, host_side, 0, wanted)
        if cell == "temperature-err":
            condition = condition and any(line.startswith("  ✗ 显卡状态不对：第二台 0 号卡（总线 00000000:01:00.0）：温度读成 ERR!") for line in lines) \
                and any("第二台 sudo nvidia-smi -r -i 0" in line and "QUIESCE_START_COMMAND" in line and line.startswith("  → 怎么办：") for line in lines)
        check(cell, condition, f"{HOST_WORDS[host_side]} 0 号卡读成「{row}」应当判坏（{wanted}）、退 1、给人看的话带弹窗出路（退 {code}）：{text(lines)}")

    # recovery-action：第二台 0 号卡 GPU Recovery Action 是 GPU Reset Required ⇒ 坏
    world = SelftestWorld(root, "recovery-action")
    world.write(os.path.join(world.peer, "q-0.txt"), "    GPU Recovery Action                : GPU Reset Required\n")
    code, lines = run(world)
    check("recovery-action", code == EXIT_FAULT and has_fault(lines, "peer", 0, "recovery-action=GPU_Reset_Required"),
          f"GPU Recovery Action 不是 None 应当判坏、退 1（退 {code}）：{text(lines)}")

    # unknown-pid-process：第二台计算进程表里 pid 为 [N/A] 的行落在 0 号卡总线上 ⇒ 0 号卡坏
    world = SelftestWorld(root, "unknown-pid-process")
    world.write(os.path.join(world.peer, "apps.csv"), "[N/A], [N/A], 00000000:01:00.0, [N/A]\n4242, python3, 00000000:01:00.0, 500\n")
    code, lines = run(world)
    check("unknown-pid-process", code == EXIT_FAULT and has_fault(lines, "peer", 0, "compute-app-pid-unknown=1"),
          f"计算进程表里 0 号卡总线上 pid 为 [N/A] 的行应当判 0 号卡坏、正常进程不算（退 {code}）：{text(lines)}")

    # unknown-pid-unknown-bus：pid 与总线都是 [N/A] 的行 ⇒ 那台另一行 card=unknown 判坏
    world = SelftestWorld(root, "unknown-pid-unknown-bus")
    world.write(os.path.join(world.peer, "apps.csv"), "[N/A], [N/A], [N/A]\n")
    code, lines = run(world)
    check("unknown-pid-unknown-bus", code == EXIT_FAULT and has_fault(lines, "peer", "unknown", "compute-app-pid-unknown-bus-unknown=1")
          and is_ok(lines, "peer", 0), f"pid 与总线都是 [N/A] 的行应当另打一行 card=unknown 判坏（退 {code}）：{text(lines)}")

    # fatal-xid：第二台内核日志里 0 号卡总线上 62 接 154 ⇒ 坏；别的总线（没点名的卡）上的 62 不算
    world = SelftestWorld(root, "fatal-xid")
    world.write(os.path.join(world.peer, "kernel.log"), "kernel: something else\n" + XID_FATAL_LINES + "NVRM: Xid (PCI:0000:02:00): 62, pid=1, name=x\n")
    code, lines = run(world)
    check("fatal-xid", code == EXIT_FAULT and has_fault(lines, "peer", 0, "xid-62=1") and has_fault(lines, "peer", 0, "xid-154=1"),
          f"0 号卡总线上的 Xid 62、154 应当判坏（退 {code}）：{text(lines)}")

    # application-xid-only：109 三条、45 两条 ⇒ 只计数、不判坏
    world = SelftestWorld(root, "application-xid-only")
    world.write(os.path.join(world.local, "kernel.log"), XID_APPLICATION_LINE * 3 + "NVRM: Xid (PCI:0000:01:00): 45, pid=4242, name=x\n" * 2)
    code, lines = run(world)
    check("application-xid-only", code == EXIT_HEALTHY and is_ok(lines, "local", 0) and "counted=xid-45:2,xid-109:3" in " ".join(lines_of(lines, "local", 0)),
          f"应用层的 Xid 109、45 应当只计数不判坏（退 {code}）：{text(lines)}")

    # xid-recovered：62、154 → GPU Reset Required，之后 154 变回 None ⇒ 那之前的不判，好
    world = SelftestWorld(root, "xid-recovered")
    world.write(os.path.join(world.peer, "kernel.log"), XID_FATAL_LINES
                + "NVRM: Xid (PCI:0000:01:00): 154, GPU recovery action changed from 0x1 (GPU Reset Required) to 0x0 (None)\n")
    code, lines = run(world)
    check("xid-recovered", code == EXIT_HEALTHY and is_ok(lines, "peer", 0),
          f"154 写着变回 None 之后，它之前的 Xid 应当不再判坏（退 {code}）：{text(lines)}")

    # xid-pci-rescanned：第二台 0 号卡（PCI 总线号 01）三个世界：开机枚举行、62 接 154、扫回的枚举行 ⇒ 好；
    #   枚举行只在 62、154 之前 ⇒ 照判坏；62、154 之后只有别的总线（02:00.0）与同卡声卡功能（01:00.1）的枚举行 ⇒ 照判坏
    outcomes = {}
    for case, log in (("after", PCI_RESCAN_LINE + XID_FATAL_LINES + PCI_RESCAN_LINE),
                      ("before", PCI_RESCAN_LINE + XID_FATAL_LINES),
                      ("other", XID_FATAL_LINES + "pci 0000:02:00.0: [10de:2c02] type 00 class 0x030000 PCIe Legacy Endpoint\n"
                       + "pci 0000:01:00.1: [10de:22e8] type 00 class 0x040300 PCIe Endpoint\n")):
        world = SelftestWorld(root, f"xid-pci-rescanned-{case}")
        world.write(os.path.join(world.peer, "kernel.log"), log)
        outcomes[case] = run(world)
    (after_code, after_lines), (before_code, before_lines), (other_code, other_lines) = outcomes["after"], outcomes["before"], outcomes["other"]
    check("xid-pci-rescanned", after_code == EXIT_HEALTHY and is_ok(after_lines, "peer", 0)
          and before_code == EXIT_FAULT and has_fault(before_lines, "peer", 0, "xid-62=1") and has_fault(before_lines, "peer", 0, "xid-154=1")
          and other_code == EXIT_FAULT and has_fault(other_lines, "peer", 0, "xid-62=1") and has_fault(other_lines, "peer", 0, "xid-154=1"),
          f"62、154 之后这张卡总线上的枚举行应当清掉它们（退 {after_code}）、枚举行在前应当照判坏（退 {before_code}）、"
          f"别的总线与声卡功能的枚举行应当不清（退 {other_code}）：{text(after_lines)} ‖ {text(before_lines)} ‖ {text(other_lines)}")

    # xid-cache-before-pci：只记 Xid 的旧记录（没有 recorded，游标 line-3 已过了扫回的枚举行）⇒ 整份重读、好；
    #   重读写下的记录第二趟照续读（--after-cursor=line-3）、照好
    world = SelftestWorld(root, "xid-cache-before-pci")
    world.write(os.path.join(world.peer, "kernel.log"), XID_FATAL_LINES + PCI_RESCAN_LINE)
    os.makedirs(os.path.join(world.directory, "state", "peer"))
    world.write(os.path.join(world.directory, "state", "peer", "kernel-xid-selftest-boot-one.json"),
                json.dumps({"cursor": "line-3", "runs": [[0, 1, 0, 62, False, 1], [0, 1, 0, 154, False, 1]]}))
    first_code, first_lines = run(world)
    code, lines = run(world)
    journal_calls = [call for call in world.calls(world.peer) if call.startswith("journalctl")]
    check("xid-cache-before-pci", first_code == EXIT_HEALTHY and is_ok(first_lines, "peer", 0) and code == EXIT_HEALTHY and is_ok(lines, "peer", 0)
          and len(journal_calls) == 2 and "--after-cursor" not in journal_calls[0] and "--after-cursor=line-3" in journal_calls[1],
          f"只记 Xid 的旧记录应当整份重读、读到扫回的枚举行判好，重读写下的记录第二趟照续读（两趟退 {first_code}、{code}，"
          f"journalctl 调用 {journal_calls}）：{text(first_lines)} ‖ {text(lines)}")

    # xid-incremental：头一趟整份读到 62；日志再多一条 109，第二趟只读游标之后的、62 照旧判坏、109 计一条
    world = SelftestWorld(root, "xid-incremental")
    world.write(os.path.join(world.local, "kernel.log"), XID_FATAL_LINES.splitlines(keepends=True)[0])
    first_code, _ = run(world)
    world.write(os.path.join(world.local, "kernel.log"), XID_APPLICATION_LINE, append=True)
    code, lines = run(world)
    journal_calls = [call for call in world.calls(world.local) if call.startswith("journalctl")]
    check("xid-incremental", first_code == EXIT_FAULT and code == EXIT_FAULT and has_fault(lines, "local", 0, "xid-62=1")
          and "counted=xid-109:1" in " ".join(lines_of(lines, "local", 0)) and len(journal_calls) == 2
          and "--after-cursor" not in journal_calls[0] and "--after-cursor=line-1" in journal_calls[1],
          f"第二趟应当只读游标之后的、接上记录里的 62（两趟退 {first_code}、{code}，journalctl 调用 {journal_calls}）：{text(lines)}")

    # kernel-log-unreadable：本机 journalctl 退 1 ⇒ 查不成，退 78
    world = SelftestWorld(root, "kernel-log-unreadable")
    world.write(os.path.join(world.local, "journal-fail"), "")
    code, lines = run(world)
    check("kernel-log-unreadable", code == EXIT_UNCHECKED and any(line.startswith(f"{SUMMARY_MARK} state=unchecked ") and "unchecked=本机:" in line for line in lines),
          f"本机内核日志读不出应当报查不成、退 78（退 {code}）：{text(lines)}")

    # peer-unreachable：第二台连不上 ⇒ 退 78、本机两张卡照报、汇总写第二台没查成
    world = SelftestWorld(root, "peer-unreachable")
    world.write(os.path.join(os.path.dirname(world.peer), "unreachable"), "")
    code, lines = run(world)
    check("peer-unreachable", code == EXIT_UNCHECKED and is_ok(lines, "local", 0) and is_ok(lines, "local", 1) and not lines_of(lines, "peer", 0)
          and any("unchecked=第二台:" in line for line in lines) and any(line.startswith("  ✗ 卡状态查不成（第二台）") for line in lines),
          f"第二台连不上应当退 78、本机照报（退 {code}）：{text(lines)}")

    # fault-and-unreachable：本机有坏卡、第二台连不上 ⇒ 退 1（坏卡优先），汇总照写第二台没查成
    world = SelftestWorld(root, "fault-and-unreachable")
    world.write(os.path.join(os.path.dirname(world.peer), "unreachable"), "")
    world.write(os.path.join(world.local, "kernel.log"), XID_FATAL_LINES)
    code, lines = run(world)
    check("fault-and-unreachable", code == EXIT_FAULT and has_fault(lines, "local", 0, "xid-62=1") and any("unchecked=第二台:" in line for line in lines),
          f"有坏卡又有一台连不上应当退 1、两样都报（退 {code}）：{text(lines)}")

    # second-host-view：同一份配置先在这一台查一趟，再演在另一台上查（假 ip 让网卡上只有 192.0.2.63）⇒ 两趟都退 0、机读行逐行相同（host= 写台标签，
    #   在哪台跑都一样）；另一台上那一趟只往 127.0.0.1 ssh 一次（192.0.2.63 那一张在它本机查）。
    #   两台的假卡读数不同（另一台 0 号卡温度 41、这一台 30）：认错了哪一台是自己的那一趟（弄坏开关 this-side-by-order），在本机读到的是另一台的卡、
    #   却贴上这一台的台标签，机读行跟着错；两台读数一样时贴错台标签的行与对的行逐字相同，这一格看不出来
    world = SelftestWorld(root, "second-host-view")
    world.write(os.path.join(world.peer, "gpus.csv"), "0, 00000000:01:00.0, 41, 17.66\n" + HEALTHY_CARDS.splitlines()[1] + "\n")
    first_code, first_lines = run(world)
    first_ssh = world.ssh_calls()
    code, lines = run(world, **world.second_host_view())

    def machine_lines(found):
        return sorted(line for line in found if line.startswith(f"{LINE_MARK} host="))

    def own_reading(found, host, temperature):
        return any(f" evidence=temperature={temperature}," in line for line in lines_of(found, host, 0))
    check("second-host-view", first_code == EXIT_HEALTHY and code == EXIT_HEALTHY and len(machine_lines(lines)) == 3
          and machine_lines(lines) == machine_lines(first_lines) and first_ssh == [SELFTEST_PEER_ALIAS]
          and world.ssh_calls() == [SELFTEST_PEER_ALIAS, SELFTEST_THIS_ADDRESS]
          and all(own_reading(found, "local", 30) and own_reading(found, "peer", 41) for found in (first_lines, lines)),
          f"同一份配置在另一台上查应当退 0、机读行与这一台查的逐行相同（{SELFTEST_THIS_ADDRESS} 的 0 号卡温度 30、{SELFTEST_PEER_ALIAS} 的 41）、"
          f"只往 {SELFTEST_THIS_ADDRESS} ssh 一次（两趟退 {first_code}、{code}，ssh {world.ssh_calls()}）："
          f"{text(machine_lines(first_lines))} ‖ {text(machine_lines(lines))}")

    # card-label-outside-hosts：台标签 192.0.2.99 不是 HOSTS 里的一项 ⇒ 配置写坏、退 78、点名那一项，不连另一台
    #   （ENABLE_GPU=1 时多机配置判法要两个显卡键都写：另一个键写一项对的，判法过得了、走到切台标签那一步）
    world = SelftestWorld(root, "card-label-outside-hosts")
    world.configuration(cards=f"GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='{SELFTEST_THIS_ADDRESS}:0:32G'\n"
                              f"GPU_CARDS_BY_PRIORITY='{SELFTEST_THIS_ADDRESS}:0:8G 192.0.2.99:0:8G'\n")
    code, lines = run(world)
    check("card-label-outside-hosts", code == EXIT_UNCHECKED and not world.ssh_calls()
          and any("192.0.2.99:0:8G" in line and "不是 HOSTS 里的一项" in line for line in lines),
          f"台标签不在 HOSTS 里应当判配置写坏、退 78、点名那一项、不 ssh（退 {code}，ssh {world.ssh_calls()}）：{text(lines)}")

    # ipv6-card-label：IPv6 地址自己带冒号，从右边切两刀照样切得开、分得清哪一台（card_labels 是纯函数，不起世界）
    library = addresses_library()
    labels, problem = card_labels("fd00::2:0:16G fd00::1:1:8G", library.Addresses("fd00::1", "fd00::2"))
    found = [(label.side, label.address, label.index, label.quota) for label in labels]
    check("ipv6-card-label", not problem and found == [("peer", "fd00::2", 0, "16G"), ("local", "fd00::1", 1, "8G")],
          f"IPv6 的台标签应当切成（另一台 fd00::2、0 号、16G）与（这一台 fd00::1、1 号、8G），实际 {found}，问题「{problem}」")

    # no-cards-named：两个键都没写 ⇒ 退 77，不连第二台
    world = SelftestWorld(root, "no-cards-named")
    world.configuration(cards="")
    code, lines = run(world)
    check("no-cards-named", code == EXIT_NOTHING_NAMED and not world.ssh_calls() and any(line.startswith(f"{SUMMARY_MARK} state=nothing-named ") for line in lines),
          f"一张卡都没点名应当退 77、不连第二台（退 {code}）：{text(lines)}")

    # configuration-missing：多机配置不在 ⇒ 退 78
    world = SelftestWorld(root, "configuration-missing")
    code, lines = run(world, SINGLEFS_MULTI_HOST_CONFIG=os.path.join(world.directory, "absent.env"))
    check("configuration-missing", code == EXIT_UNCHECKED and any("unchecked=配置:" in line for line in lines),
          f"多机配置不在应当退 78、汇总写配置没查成（退 {code}）：{text(lines)}")

    # bad-card-entry：卡的写法不对 ⇒ 退 78（另一个显卡键写一项对的，同 card-label-outside-hosts）
    world = SelftestWorld(root, "bad-card-entry")
    world.configuration(cards=f"GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED='{SELFTEST_THIS_ADDRESS}:0:32G'\n"
                              f"GPU_CARDS_BY_PRIORITY='{SELFTEST_THIS_ADDRESS}:x:8G'\n")
    code, lines = run(world)
    check("bad-card-entry", code == EXIT_UNCHECKED and any(f"{SELFTEST_THIS_ADDRESS}:x:8G" in line for line in lines),
          f"卡的写法不对应当退 78、点名那一项（退 {code}）：{text(lines)}")

    # across-machines-off：点名了第二台的卡而双机没开 ⇒ 本机照查、第二台没查成、退 78、不 ssh
    world = SelftestWorld(root, "across-machines-off")
    world.configuration(across_machines=False)
    code, lines = run(world)
    check("across-machines-off", code == EXIT_UNCHECKED and is_ok(lines, "local", 0) and not world.ssh_calls() and any("unchecked=第二台:" in line for line in lines),
          f"双机没开而点名了第二台的卡应当本机照查、退 78、不 ssh（退 {code}，ssh {world.ssh_calls()}）：{text(lines)}")

    # assigned-good-card：配置点名本机 0、1 与第二台 0，第二台 0 号卡坏（致命类 Xid）；--cards 只给本机 1 号卡 ⇒ 只查它、退 0、不连第二台
    world = SelftestWorld(root, "assigned-good-card")
    world.write(os.path.join(world.peer, "kernel.log"), XID_FATAL_LINES)
    code, lines = check_cards(world.environment(), f"{SELFTEST_THIS_ADDRESS}:1:8G")
    check("assigned-good-card", code == EXIT_HEALTHY and is_ok(lines, "local", 1) and not lines_of(lines, "local", 0) and not lines_of(lines, "peer", 0)
          and not world.ssh_calls() and f"{SUMMARY_MARK} state=ok checked=1 fault=0 unchecked=none" in lines,
          f"--cards 只给本机 1 号卡时应当只查它、没分到的第二台坏卡不挡、退 0、不 ssh（退 {code}，ssh {world.ssh_calls()}）：{text(lines)}")

    # assigned-bad-card：同一个世界，--cards 给第二台 0 号卡 ⇒ 判它坏、退 1，本机的卡不查
    world = SelftestWorld(root, "assigned-bad-card")
    world.write(os.path.join(world.peer, "kernel.log"), XID_FATAL_LINES)
    code, lines = check_cards(world.environment(), f"{SELFTEST_PEER_ALIAS}:0:16G")
    check("assigned-bad-card", code == EXIT_FAULT and has_fault(lines, "peer", 0, "xid-62=1") and not lines_of(lines, "local", 0) and not lines_of(lines, "local", 1)
          and f"{SUMMARY_MARK} state=fault checked=1 fault=1 unchecked=none" in lines,
          f"--cards 给第二台 0 号坏卡时应当判它坏、退 1、本机的卡不查（退 {code}）：{text(lines)}")

    # assigned-label-outside-hosts：--cards 的台标签不在 HOSTS 里 ⇒ 退 78、点名那一项、不 ssh
    world = SelftestWorld(root, "assigned-label-outside-hosts")
    code, lines = check_cards(world.environment(), "192.0.2.99:0:8G")
    check("assigned-label-outside-hosts", code == EXIT_UNCHECKED and not world.ssh_calls()
          and any("--cards 里" in line and "192.0.2.99:0:8G" in line and "不是 HOSTS 里的一项" in line for line in lines),
          f"--cards 的台标签不在 HOSTS 里应当退 78、点名那一项、不 ssh（退 {code}，ssh {world.ssh_calls()}）：{text(lines)}")

    # usage：认不出的参数、--cards 后面没有值或值是空白都退 2
    usage_codes = []
    for arguments in (["--bogus"], ["--cards"], ["--cards", " "]):
        capture = os.path.join(root, "usage.out")
        with open(capture, "w", encoding="utf-8") as handle:
            code = main(arguments, {}, handle)
        with open(capture, encoding="utf-8") as handle:
            usage_codes.append((arguments, code, "用法" in handle.read()))
    check("usage", all(code == EXIT_USAGE and said for _, code, said in usage_codes),
          f"认不出的参数、--cards 缺值或值是空白应当都退 2、说用法（{usage_codes}）")
    return failures, checked


SWITCH_EXPECTED_CELLS = {
    "GPU_CARD_HEALTH_BREAK=nvidia-smi-failure-trusted": ["nvidia-smi-fails"],
    "GPU_CARD_HEALTH_BREAK=missing-card-trusted": ["missing-card"],
    "GPU_CARD_HEALTH_BREAK=readings-trusted": ["temperature-err", "power-na", "temperature-unknown-error", "power-requires-reset"],
    "GPU_CARD_HEALTH_BREAK=recovery-action-ignored": ["recovery-action"],
    "GPU_CARD_HEALTH_BREAK=unknown-pid-ignored": ["unknown-pid-process", "unknown-pid-unknown-bus"],
    "GPU_CARD_HEALTH_BREAK=fatal-xid-ignored": ["fatal-xid", "xid-incremental", "fault-and-unreachable"],
    "GPU_CARD_HEALTH_BREAK=application-xid-fatal": ["application-xid-only", "xid-incremental"],
    "GPU_CARD_HEALTH_BREAK=xid-recovery-ignored": ["xid-recovered"],
    "GPU_CARD_HEALTH_BREAK=pci-rescan-ignored": ["xid-pci-rescanned"],
    "GPU_CARD_HEALTH_BREAK=pci-rescan-any-bus": ["xid-pci-rescanned"],
    "GPU_CARD_HEALTH_BREAK=pci-rescan-order-ignored": ["xid-pci-rescanned"],
    "GPU_CARD_HEALTH_BREAK=xid-cache-recorded-ignored": ["xid-cache-before-pci"],
    "GPU_CARD_HEALTH_BREAK=xid-cache-forgets": ["xid-incremental"],
    "GPU_CARD_HEALTH_BREAK=kernel-log-failure-trusted": ["kernel-log-unreadable"],
    "GPU_CARD_HEALTH_BREAK=peer-unreachable-trusted": ["peer-unreachable", "fault-and-unreachable"],
    "GPU_CARD_HEALTH_BREAK=this-side-by-order": ["second-host-view"],
    "GPU_CARD_HEALTH_BREAK=label-outside-hosts-accepted": ["card-label-outside-hosts"],
    "GPU_CARD_HEALTH_BREAK=label-split-from-left": ["ipv6-card-label"],
    "GPU_CARD_HEALTH_BREAK=empty-as-healthy": ["no-cards-named"],
    "GPU_CARD_HEALTH_BREAK=missing-configuration-as-empty": ["configuration-missing"],
    "GPU_CARD_HEALTH_BREAK=assigned-cards-ignored": ["assigned-good-card", "assigned-bad-card", "assigned-label-outside-hosts"],
}


def run_cells_in_scratch():
    root = tempfile.mkdtemp(prefix="gpu-card-health-selftest-")
    try:
        return selftest_cells(root)
    finally:
        shutil.rmtree(root, ignore_errors=True)


def selftest():
    failures, checked = run_cells_in_scratch()
    if failures:
        print(f"  ✗ gpu-card-health.py 自证 {len(failures)} 格不对（查了 {checked} 格）：{'、'.join(failures)}")
        print("  → 怎么办：照上面逐格改 collect_card_readings / judge_host / check_cards；弄坏开关设着的话本来就该红")
        return 1
    table_switches = set(re.findall(r"^\s+(GPU_CARD_HEALTH_BREAK=[a-z0-9-]+)\s{2}", __doc__, re.M))
    code_switches = {f"{BREAK_VARIABLE}={name}" for name in re.findall(r'broken\("([a-z0-9-]+)"\)', open(os.path.realpath(__file__), encoding="utf-8").read())}
    switch_failures = []
    if not (table_switches == code_switches == set(SWITCH_EXPECTED_CELLS)):
        switch_failures.append(f"文件头的开关表、代码里认的开关与 SWITCH_EXPECTED_CELLS 三处不一致（表 {sorted(table_switches)}；代码 {sorted(code_switches)}；"
                               f"SWITCH_EXPECTED_CELLS {sorted(SWITCH_EXPECTED_CELLS)}）")
    # 开关轮的并行度取每台默认重活线程数的登记位（SINGLEFS_THREAD_CAP 设了照它），再被开关条数夹住
    sys.path.insert(0, SCRIPT_DIRECTORY)
    import heavy_threads
    try:
        registered_threads = heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 开关轮取不了每台默认重活线程数：{error}")
        print(f"  → 怎么办：{error.howto}")
        return 1
    jobs = max(1, min(len(SWITCH_EXPECTED_CELLS), registered_threads))
    clean = {name: value for name, value in os.environ.items() if name != BREAK_VARIABLE}

    def red_cells_under(switch):
        variable, setting = switch.split("=", 1)
        completed = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--selftest-cells"], capture_output=True, text=True,
                                   env=dict(clean, **{variable: setting}), timeout=600, stdin=subprocess.DEVNULL)
        return switch, {line.split("：", 1)[0].removeprefix("  ✗ 自证 ") for line in completed.stdout.splitlines() if line.startswith("  ✗ 自证 ")}

    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        collected = list(pool.map(red_cells_under, SWITCH_EXPECTED_CELLS))
    if len(collected) != len(SWITCH_EXPECTED_CELLS):
        switch_failures.append(f"开关轮派出 {len(SWITCH_EXPECTED_CELLS)} 个、收回 {len(collected)} 个")
    for switch, red in collected:
        print(f"  · 弄坏开关 {switch}：红在 {'、'.join(sorted(red)) or '一格都没有'}")
        missing = [cell for cell in SWITCH_EXPECTED_CELLS[switch] if cell not in red]
        if missing:
            switch_failures.append(f"{switch} 没红在 {'、'.join(missing)}（红的是 {'、'.join(sorted(red)) or '一格都没有'}）")
    for failure in switch_failures:
        print(f"  ✗ 弄坏开关 {failure}")   # gate-lint:detail
    if switch_failures:
        print("  → 怎么办：照上面那几个开关看它在代码里弄坏的那一处还在不在判法上；改了判法就改 SWITCH_EXPECTED_CELLS 与文件头的表")
        return 1
    print(f"  ✓ gpu-card-health.py 自证通过（查了 {checked} 格：三张卡都好、nvidia-smi 读不出、少卡、温度与功耗读成四种坏值、Recovery Action 不是 None、"
          f"计算进程表里 pid 为 [N/A]（落在卡上的与总线也读不出的）、致命类 Xid、应用层 Xid 只计数、154 变回 None 之后清掉、"
          f"这张卡总线上的枚举行之后清掉（枚举行在前、别的总线与声卡功能的不清）、只记 Xid 的旧记录整份重读、游标之后接着读、内核日志读不出、"
          f"第二台连不上、坏卡与连不上同时、同一份配置在另一台上查得一样、台标签不在 HOSTS 里判写坏、IPv6 的台标签切得开、一张卡都没点名、配置不在、卡的写法不对、双机没开、"
          f"--cards 只查分到的卡（没分到的坏卡不挡、分到的坏卡判坏、台标签不在 HOSTS 里判写坏）、用法错；"
          f"弄坏开关 {len(collected)} 个逐个打开都红在点名的格）")
    return 0


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    if sys.argv[1:] == ["--selftest-cells"]:
        cell_failures, _ = run_cells_in_scratch()
        sys.exit(1 if cell_failures else 0)
    for break_var in (BREAK_VARIABLE, "MULTI_HOST_CONFIGURATION_BREAK"):
        if os.environ.get(break_var):
            print(f"  ✗ 弄坏开关 {break_var}={os.environ[break_var]} 只在自证（--selftest）里认，平常跑设了就拒，什么都没查", file=sys.stderr)
            print(f"  → 怎么办：去掉 {break_var} 再跑；要证红就跑 {break_var}=<项> python3 scripts/hosts/gpu-card-health.py --selftest", file=sys.stderr)
            sys.exit(EXIT_USAGE)
    sys.exit(main(sys.argv[1:]))
