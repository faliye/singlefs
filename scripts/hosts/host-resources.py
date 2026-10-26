#!/usr/bin/env python3
# admission: always 每一次都现查各台此刻的核、占用、内存与卡，上一次查到的不替这一次作保
# run-condition: command bash python3
"""各台此刻能用多少：查多机配置里每一台的核、占用、内存与卡，报这一刻最多能开多少线程、多少内存、哪几张卡多少显存；
给了命令就再报它的历史，逐台判放不放得下（用户 2026-10-06：「每次跑之前必须运行 测一下双机的显卡和cpu 然后回报」「先跑脚本获取资源数，
然后根据资源数和自己的历史运行时间来确实能不能跑起来，如果可以就跑，如果不行就回报」；records/2026-09-16-subagent拆分提案.md 第五十五节）。
只读：不清场、不停任何进程、不排队、不拒任何命令。

用法：
  host-resources.py [--json]                              各台此刻的资源（给人看的几行表；--json 给脚本与派发器读）
  host-resources.py [--json] [--gpu] [--shard <k>/<n>] [--estimated-hours <小时>] -- <命令…>
                                                           另报这条命令的历史，逐台判放不放得下，末尾一行结论；--gpu 连显存一起判；
                                                           --shard、--estimated-hours 的意思同 host-dispatch.py（只用来判下面「长活提醒」）
  host-resources.py --selftest                             假的 /proc、假 nvidia-smi、假 ssh 走一遍各格，末尾逐个打开弄坏开关核它红在点名的格
退出码：0 报完（给了命令时：此刻有一台放得下）；1 给了命令而此刻哪台都放不下（或都没读到）；2 用法错。

机器：本机总在；别的从多机配置读（${SINGLEFS_MULTI_HOST_CONFIG:-主工作树的根/multi-host.env}，不进仓）：
  scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts 判配置、解出机器清单（键 HOSTS 与 HOST_<号>_*，定义在那一份文件头「机器清单」；
  没写 HOSTS 时清单就是 PEER_SSH_HOST 一台）。它不连任何一台，连不上的那一台在这里报「没读到」，不算整份配置坏。
  判法退 3（跨机开关 ENABLE_ACROSS_MACHINES 没开）或判不过时只报本机，打一行原因。
  别的那几台经 scripts/hosts/peer-host-lib.sh 的 peer_host_run 用 ssh 读（错误输出里的主机名、目录换成「第二台」那几个字），
  把 scripts/hosts/memory_peaks_lib.py 的源码与本文件的 probe_this_host 一段送过去跑（python3 -c，base64），同 host-dispatch.py 读第二台的办法。
  各台并行读，每台至多等 HOST_RESOURCES_HOST_TIMEOUT_SECONDS 秒（默认 HOST_TIMEOUT_SECONDS），到点那台记「没读到」；到点只停自己起的那一个 bash，
  它起的 ssh 由 ssh 自己的超时收尾。台名只写「本机」「第二台」「第三台」……（清单里的次序），不写别名。
每台现查：
  核数：/proc/stat 里 cpu<N> 的行数。占用：/proc/stat 隔 HOST_RESOURCES_SAMPLE_SECONDS 秒（默认 1）读两次，(1 − 空闲与 iowait 的增量 / 总增量) × 核数；
    另报 /proc/loadavg 的前三个数。可用线程 = 核数 − 此刻在用的核（四舍五入）。
  内存：memory_peaks_lib.headroom（MemAvailable 减内存包装账上在跑的、还没涨到预测峰值的那一截，算法同派发器），
    再减派发器账上刚派出去、包装还没记账的那一截（host-dispatch.py 的 dispatch_pending，只有本机与 PEER_SSH_HOST 那一台有）。
  卡：nvidia-smi 的每张卡（型号、总显存、空闲显存）与卡上的计算进程（进程号与占用）。进程归不归清场服务按它 /proc/<进程号>/cgroup 里的 systemd 单元判：
    单元名出现在多机配置的 QUIESCE_STOP_COMMAND 里（连同那条命令点名的、存在的文件的正文）就算清场白名单里的服务；只报「有清场服务占着 N MiB」，不报单元名。
    每张卡 不清场可用 = 空闲 − GPU_MARGIN_MEBIBYTES（判器留给驱动的那一份，同 host-dispatch.py），清场可用 = 空闲 + 清场服务占着的 − 同一份余量，都不低于 0。
    别的计算进程（别的会话的活、归不到清场服务的）照占着算，逐个列进程号与占用。
  合计：读到的那几台相加；哪一台哪一项没读到，合计里写明不含它。
给了命令：
  历史：内存峰值表（RUN_WITH_MEMORY_CAP_PEAKS，默认 ${XDG_DATA_HOME:-~/.local/share}/singlefs/memory-peaks.tsv）与派发器的显存峰值表（HOST_DISPATCH_GPU_PEAKS），
    键同内存包装（memory_peaks_lib.command_key；RUN_WITH_MEMORY_CAP_KEY 给了就用它）；内存峰值报的是 memory_peaks_lib.recorded_peak 读的那个数
    （近几次实测里最大的，包装记账、派发器预测用的同一个），另报最近一次是哪天量的、那一次的上限与近几次有几次；
    用时取用时表（memory_peaks_lib.py 文件头「用时表」：只有经 host-dispatch.py 派出去、跑完的才记），没有这条报「没有历史」。
  长活提醒：host-dispatch.py 的 long_run_check（文件头 ⓪，同一份判法）：用时表里的历史用时，没有就按 --estimated-hours；
    派法占几台照它的 hosts_of_dispatch，「此刻能派的机器」按这一次读到的台数算。用时超过 2 小时而只占一台，结论之前报一行
    「提醒：……超过 2 小时，而派法只占一台」，出路同 host-dispatch.py 拒的那一句；只提醒，退出码不变（派发器那一边拒、退 244）。
  逐台判：host-dispatch.py 的 predict 与 judge_host（同 host-dispatch.py --plan 的判法，按文件路径导入，不另写一份）；
    第 <号> 台的上限收到配置的 HOST_<号>_MEMORY_CAP（PEER_SSH_HOST 那一台没写取 PEER_MEMORY_CAP）。放不下的报差多少内存、差多少显存。
    结论：按本机、清单次序第一台放得下的报「可以跑：在<台名>」；都放不下报「跑不起来」与每台差在哪；没读到的那台写「没读到，判不了」。

弄坏开关（只给 --selftest 证红用，逗号连几个；在 --selftest 之外设了就拒、退 2）。项名、弄坏哪一处、打开后红在哪几格
  （格名是「✗ 自证 <格名>：」里的那一段；末尾那一轮照 SWITCH_EXPECTED_CELLS 逐个核）：
  HOST_RESOURCES_BREAK=busy-not-subtracted     可用线程不减此刻在用的核                         both-hosts
  HOST_RESOURCES_BREAK=pending-not-subtracted  可用内存按 MemAvailable，不减包装账上还没涨到量的  both-hosts
  HOST_RESOURCES_BREAK=unread-as-zero          没读到的那一台当全是 0、算进合计                   peer-unreachable、host-timeout、hosts-list
  HOST_RESOURCES_BREAK=no-host-timeout         读别的台不设时限，等到它自己回来                   host-timeout
  HOST_RESOURCES_BREAK=services-as-others      清场白名单当空的：清场服务的进程当别的进程         quiesce-service
  HOST_RESOURCES_BREAK=others-as-services      卡上每个计算进程都当清场服务                       card-other-process
  HOST_RESOURCES_BREAK=service-name-shown      给人看的表里写出清场服务的单元名                   quiesce-service
  HOST_RESOURCES_BREAK=peer-despite-switch-off 跨机开关没开也读别的台（借配置判法的 switch-off-is-on） switch-off
  HOST_RESOURCES_BREAK=first-host-only         清单里只读第一台                                   hosts-list
  HOST_RESOURCES_BREAK=wrong-host              结论取最后一台放得下的                             fits-on-host
  HOST_RESOURCES_BREAK=no-shortfall            放不下时不报差多少                                 nothing-fits
  HOST_RESOURCES_BREAK=no-usage-check          认不出的参数也照跑                                 usage
  HOST_RESOURCES_BREAK=long-history-ignored    长活提醒不看用时表里的历史用时                     long-history-reminder
  另核两份共用的判法改坏时这里也红（同一份判法，不是另写的一套）：
  HOST_DISPATCH_BREAK=ignore-history           host-dispatch.py 的 predict 不看峰值表               fits-on-host、nothing-fits
  HOST_DISPATCH_BREAK=long-estimate-ignored    host-dispatch.py 的 long_run_check 不看预计挂钟      long-estimate-reminder
  HOST_DISPATCH_BREAK=shards-as-single         host-dispatch.py 的 hosts_of_dispatch 把 --shard 当只占一台  two-host-shards-no-reminder
  MULTI_HOST_CONFIGURATION_BREAK=hosts-key-unknown    配置判法不认机器清单的键（写法见 crash-point-replay-shard-configuration-check.sh 文件头）  hosts-list、fits-on-host
  MULTI_HOST_CONFIGURATION_BREAK=hosts-reach-refuses  配置判法逐台 ssh、一台连不上就判整份配置坏                                 hosts-list、peer-unreachable、host-timeout

常量的依据：HOST_TIMEOUT_SECONDS 12：本机读第二台（ssh 加 python3 起一份、一秒采样）实测两三秒，留余量，又要落在提醒钩子
  .claude/hooks/host-resources-reminder.sh 文件头的时限之内；推的。
"""
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import base64  # noqa: E402
import concurrent.futures  # noqa: E402
import contextlib  # noqa: E402
import importlib.util  # noqa: E402
import inspect  # noqa: E402
import json  # noqa: E402
import os  # noqa: E402
import re  # noqa: E402
import shlex  # noqa: E402
import math  # noqa: E402
import shutil  # noqa: E402
import subprocess  # noqa: E402
import sys  # noqa: E402
import tempfile  # noqa: E402
import time  # noqa: E402
from dataclasses import dataclass  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, SCRIPT_DIRECTORY)
import memory_peaks_lib as peaks  # noqa: E402

# bash -c 起的两份脚本：多机配置的判法、第二台那几段的共用库。写成模块常量（函数单元键按字面折得出、连到那两份；
# 放进 Options 的字段里就折不出，起它们的那两处拒算 shell-path-unfolded）
CONFIGURATION_CHECK = os.path.join(SCRIPT_DIRECTORY, "..", "crash", "crash-point-replay-shard-configuration-check.sh")
PEER_HOST_LIB = os.path.join(SCRIPT_DIRECTORY, "peer-host-lib.sh")
HOST_TIMEOUT_SECONDS = 12.0
SAMPLE_SECONDS = 1.0
PROBE_MARK = "HOST_RESOURCES_PROBE "
EXIT_USAGE = 2
MAXIMUM_NAMED_FILE_BYTES = 1 << 20     # QUIESCE_STOP_COMMAND 点名的文件超过这么大不读进白名单
HOST_NAMES = ["本机", "第二台", "第三台", "第四台", "第五台", "第六台", "第七台", "第八台", "第九台", "第十台"]
BREAK_VARIABLES = ("HOST_RESOURCES_BREAK", "HOST_DISPATCH_BREAK", "MULTI_HOST_CONFIGURATION_BREAK")


def broken(item):
    return item in os.environ.get("HOST_RESOURCES_BREAK", "").split(",")


_dispatcher = None


def dispatcher():
    """scripts/hosts/host-dispatch.py（文件名带减号，按路径导入）：预测、判放不放得下、派发器自己的账都用它那一份。"""
    global _dispatcher
    if _dispatcher is None:
        spec = importlib.util.spec_from_file_location("host_dispatch", os.path.join(SCRIPT_DIRECTORY, "host-dispatch.py"))
        module = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(module)
        _dispatcher = module
    return _dispatcher


def host_name(number):
    return HOST_NAMES[number - 1] if number <= len(HOST_NAMES) else f"第 {number} 台"


# ── 每台上跑的那一段：本机直接调；别的台把这个函数的源码连同 memory_peaks_lib.py 送过去调（所以只用标准库、不引用本文件别处的名字）──
def probe_this_host(environment, sample_seconds, headroom_function):
    """这台此刻的 cpu、内存与卡：{"cpu": …, "memory": …, "gpu": …}，每一项读不出就是 {"problem": 为什么}。
    HOST_RESOURCES_PROC_ROOT 换掉 /proc（只给自证）；那里有 stat.later 时第二读读它（假的 /proc 里内容不会自己走动）。"""
    import csv as table
    import os as system
    import subprocess as process
    import time as clock
    proc_root = environment.get("HOST_RESOURCES_PROC_ROOT") or "/proc"
    report = {}

    def read_stat(path):
        aggregate, cores = None, 0
        with open(path, encoding="utf-8") as handle:
            for line in handle:
                fields = line.split()
                if fields and fields[0] == "cpu":
                    aggregate = [int(value) for value in fields[1:9]]
                elif fields and fields[0].startswith("cpu") and fields[0][3:].isdigit():
                    cores += 1
        return aggregate, cores

    try:
        first, cores = read_stat(system.path.join(proc_root, "stat"))
        clock.sleep(sample_seconds)
        later = system.path.join(proc_root, "stat.later")
        second, _ = read_stat(later if system.path.exists(later) else system.path.join(proc_root, "stat"))
        total = sum(second) - sum(first)
        idle = (second[3] + second[4]) - (first[3] + first[4])
        if cores == 0 or total <= 0:
            report["cpu"] = {"problem": f"/proc/stat 里核数 {cores}、{sample_seconds:g} 秒里总增量 {total}，算不出占用"}
        else:
            report["cpu"] = {"cores": cores, "busy_cores": round(max(0.0, 1 - idle / total) * cores, 1)}
    except (OSError, ValueError, IndexError, TypeError) as error:
        report["cpu"] = {"problem": f"/proc/stat 读不了（{type(error).__name__}）"}
    if "cores" in report["cpu"]:
        try:
            with open(system.path.join(proc_root, "loadavg"), encoding="utf-8") as handle:
                report["cpu"]["loadavg"] = [float(value) for value in handle.read().split()[:3]]
        except (OSError, ValueError) as error:
            report["cpu"]["loadavg_problem"] = f"/proc/loadavg 读不了（{type(error).__name__}）"
    try:
        report["memory"] = headroom_function(environment)
    except (OSError, KeyError, ValueError) as error:
        report["memory"] = {"problem": f"内存余量查不成（{type(error).__name__}: {error}）"}

    def nvidia_smi(query, width):
        try:
            completed = process.run(["nvidia-smi", query, "--format=csv,noheader,nounits"], capture_output=True, text=True,
                                    env=environment, timeout=30, stdin=process.DEVNULL)
        except (OSError, process.TimeoutExpired) as error:
            return None, f"nvidia-smi 起不来或超时（{type(error).__name__}）"
        if completed.returncode != 0:
            return None, f"nvidia-smi 退 {completed.returncode}：{completed.stderr.strip()[:200]}"
        rows = [[piece.strip() for piece in row] for row in table.reader(completed.stdout.splitlines())]
        return [row for row in rows if len(row) == width], ""

    cards, problem = nvidia_smi("--query-gpu=index,name,uuid,memory.total,memory.free", 5)
    if cards is None:
        report["gpu"] = {"problem": problem}
        return report
    cards = [{"index": int(row[0]), "name": row[1], "uuid": row[2], "total": int(row[3]), "free": int(row[4])}
             for row in cards if row[0].isdigit() and row[3].isdigit() and row[4].isdigit()]
    if not cards:
        report["gpu"] = {"problem": "nvidia-smi 一张卡都没报"}
        return report
    report["gpu"] = {"cards": cards}
    apps, problem = nvidia_smi("--query-compute-apps=pid,used_memory,gpu_uuid", 3)
    if apps is None:
        report["gpu"]["apps_problem"] = problem
        return report
    listed = []
    for process_id, used, uuid in apps:
        if not (process_id.isdigit() and used.isdigit()):
            continue
        unit = ""
        try:
            with open(system.path.join(proc_root, process_id, "cgroup"), encoding="utf-8") as handle:
                for line in handle:
                    services = [piece for piece in line.strip().split(":", 2)[-1].split("/") if piece.endswith(".service")]
                    unit = services[-1] if services else unit
        except OSError:
            unit = ""
        listed.append({"pid": int(process_id), "used": int(used), "uuid": uuid, "unit": unit})
    report["gpu"]["apps"] = listed
    return report


PROBE_DRIVER = '''
import json, os, sys
print("HOST_RESOURCES_PROBE " + json.dumps(probe_this_host(dict(os.environ), float(sys.argv[1]), headroom)))
'''


def remote_probe_program():
    with open(os.path.join(SCRIPT_DIRECTORY, "memory_peaks_lib.py"), encoding="utf-8") as handle:
        library = handle.read()
    return "\n".join([library, inspect.getsource(probe_this_host), PROBE_DRIVER])


# ── 机器清单与配置 ──
@dataclass
class Machine:
    number: int                  # 1 是本机，2 起是清单里的次序
    ssh_host: str = ""
    cargo_bin_directory: str = ""
    repository_directory: str = ""
    memory_cap_text: str = ""
    is_peer: bool = False        # 就是 PEER_SSH_HOST 那一台（派发器的账记在 "peer" 名下）


@dataclass
class Options:
    settings: object             # host-dispatch.py 的 Settings：峰值表、显存峰值表、派发器与清场包装的状态目录、环境
    sample_seconds: float = SAMPLE_SECONDS
    host_timeout_seconds: float = HOST_TIMEOUT_SECONDS


def options_from_environment():
    settings = dispatcher().settings_from_environment()
    environment = settings.environment
    def positive_seconds(name, default):
        try:
            seconds = float(environment.get(name) or default)
        except ValueError:
            raise ValueError(f"{name} 要是大于 0 的有限秒数") from None
        if not math.isfinite(seconds) or seconds <= 0:
            raise ValueError(f"{name} 要是大于 0 的有限秒数")
        return seconds
    return Options(settings=settings,
                   sample_seconds=positive_seconds("HOST_RESOURCES_SAMPLE_SECONDS", SAMPLE_SECONDS),
                   host_timeout_seconds=positive_seconds("HOST_RESOURCES_HOST_TIMEOUT_SECONDS", HOST_TIMEOUT_SECONDS))


def load_machines(options):
    """(机器列表, 配置的值 dict, 为什么只有本机)：判法 --emit-hosts 的赋值经 bash eval 后按 NUL 分隔带回来。"""
    environment = dict(options.settings.environment)
    if broken("peer-despite-switch-off"):
        environment["MULTI_HOST_CONFIGURATION_BREAK"] = ",".join(filter(None, [environment.get("MULTI_HOST_CONFIGURATION_BREAK"), "switch-off-is-on"]))
    loader = ('assignments="$(bash "$1" --emit-hosts "$2")"; status=$?; if ((status != 0)); then exit "$status"; fi; eval "$assignments"; '
              'for name in ${!MULTI_HOST_@} CRASH_POINT_REPLAY_SHARD_CONFIGURATION_FILE QUIESCE_STOP_COMMAND PEER_SSH_HOST; do printf "%s=%s\\0" "$name" "${!name}"; done')
    completed = subprocess.run(["bash", "-c", loader, "host-resources", CONFIGURATION_CHECK, options.settings.repository_root],
                               capture_output=True, text=True, env=environment, stdin=subprocess.DEVNULL)
    machines = [Machine(1)]
    if completed.returncode == 3:
        return machines, {}, "多机配置里跨机开关 ENABLE_ACROSS_MACHINES 没开：只报本机"
    if completed.returncode != 0:
        return machines, {}, (f"多机配置判不过（判法 {os.path.basename(CONFIGURATION_CHECK)} 退 {completed.returncode}；"
                              "单跑 bash scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-hosts 看原因）：只报本机")
    values = dict(item.split("=", 1) for item in completed.stdout.split("\0") if "=" in item)
    count = int(values.get("MULTI_HOST_COUNT") or 0)
    for number in range(2, count + 2):
        prefix = f"MULTI_HOST_{number}_"
        machines.append(Machine(number, values.get(prefix + "SSH_HOST", ""), values.get(prefix + "CARGO_BIN_DIRECTORY", ""),
                                values.get(prefix + "REPOSITORY_DIRECTORY", ""), values.get(prefix + "MEMORY_CAP", ""),
                                values.get(prefix + "IS_PEER") == "1"))
    if broken("first-host-only"):
        machines = machines[:2]
    return machines, values, ""


def quiesce_units(values, options):
    """清场白名单里的单元名：QUIESCE_STOP_COMMAND 的正文连同它点名的、存在的文件的正文里出现的 <名字>.service。配置没读到时按配置文件现读这一个键。"""
    if broken("services-as-others"):
        return set()
    command = values.get("QUIESCE_STOP_COMMAND")
    if command is None:
        configuration_file = options.settings.environment.get("SINGLEFS_MULTI_HOST_CONFIG") or os.path.join(options.settings.repository_root, "multi-host.env")
        reader = 'source "$1" || exit 70; crash_amplification_configuration_value "$2" QUIESCE_STOP_COMMAND'
        completed = subprocess.run(["bash", "-c", reader, "host-resources", os.path.join(SCRIPT_DIRECTORY, "..", "crash", "crash-amplification-lib.sh"), configuration_file],
                                   capture_output=True, text=True, env=options.settings.environment, stdin=subprocess.DEVNULL)
        command = completed.stdout.strip() if completed.returncode == 0 else ""
    texts = [command]
    try:
        words = shlex.split(command)
    except ValueError:
        words = command.split()
    for word in words:
        path = os.path.expanduser(word)
        with contextlib.suppress(OSError, UnicodeDecodeError):
            if os.path.isfile(path) and os.path.getsize(path) <= MAXIMUM_NAMED_FILE_BYTES:
                with open(path, encoding="utf-8") as handle:
                    texts.append(handle.read())
    return set(re.findall(r"[A-Za-z0-9@_.:-]+\.service", "\n".join(texts)))


def run_on_machine(options, machine, text):
    """在那台上 bash -c 跑 text（peer-host-lib.sh 的 peer_host_run）：(退出码, 标准输出, 标准错误)；到时限只停自己起的那个 bash。"""
    runner = ('source "$1" || exit 70; PEER_SSH_HOST="$2"; PEER_CARGO_BIN_DIRECTORY="$3"; PEER_REPOSITORY_DIRECTORY="$4"; '
              'peer_host_run / "$5"')
    timeout = None if broken("no-host-timeout") else options.host_timeout_seconds
    try:
        completed = subprocess.run(["bash", "-c", runner, "host-resources", PEER_HOST_LIB, machine.ssh_host,
                                    machine.cargo_bin_directory, machine.repository_directory, text],
                                   capture_output=True, text=True, env=options.settings.environment, timeout=timeout, stdin=subprocess.DEVNULL)
    except subprocess.TimeoutExpired:
        return 124, "", f"{options.host_timeout_seconds:g} 秒没回话"
    return completed.returncode, completed.stdout, completed.stderr


def probe_machine(options, machine):
    """(报告, 问题)：报告是 probe_this_host 那份 dict；整台读不到时报告是 None。"""
    if machine.number == 1:
        return probe_this_host(options.settings.environment, options.sample_seconds, peaks.headroom), ""
    program = base64.b64encode(remote_probe_program().encode("utf-8")).decode("ascii")
    status, stdout, stderr = run_on_machine(options, machine, f'python3 -B -c "$(printf %s {program} | base64 -d)" {options.sample_seconds:g}')
    line = next((line for line in reversed(stdout.splitlines()) if line.startswith(PROBE_MARK)), None)
    if status != 0 or line is None:
        # peer-host-lib.sh 换字时一律写「第二台」：清单里第三台起换成它自己的台名
        tail = stderr.strip().splitlines()[-1][-200:].replace("第二台", host_name(machine.number)) if stderr.strip() else ""
        return None, f"退 {status}" + (f"：{tail}" if tail else "")
    try:
        return json.loads(line[len(PROBE_MARK):]), ""
    except ValueError:
        return None, "回来的那一行不是 JSON"


# ── 算一台能用多少 ──
def summarize_machine(options, machine, report, problem, units, dispatch_entries):
    margin = dispatcher().GPU_MARGIN_MEBIBYTES
    summary = {"number": machine.number, "name": host_name(machine.number), "is_local": machine.number == 1, "is_peer": machine.is_peer,
               "memory_cap": machine.memory_cap_text or None, "read": report is not None, "problem": problem or None, "raw": report}
    if report is None and broken("unread-as-zero"):
        report = {"cpu": {"cores": 0, "busy_cores": 0.0}, "memory": {"mem_total": 0, "mem_available": 0, "pending": 0, "free": 0, "slice_total": 0,
                                                                    "slice_used": 0, "slice_room": 0, "running": 0}, "gpu": {"cards": []}}
        summary.update(read=True, problem=None, raw=report)
    if report is None:
        summary.update(cpu=None, memory=None, gpu=None)
        return summary
    cpu = report.get("cpu") or {"problem": "没报"}
    if "cores" in cpu:
        used = 0 if broken("busy-not-subtracted") else int(round(cpu["busy_cores"]))
        cpu = dict(cpu, available_threads=max(0, cpu["cores"] - used))
    summary["cpu"] = cpu
    memory = report.get("memory") or {"problem": "没报"}
    if "free" in memory:
        dispatch_name = "local" if machine.number == 1 else ("peer" if machine.is_peer else None)
        dispatched = dispatcher().dispatch_pending(options.settings, dispatch_name, dispatch_entries)[0] if dispatch_name else 0
        available = memory["mem_available"] if broken("pending-not-subtracted") else memory["free"] - dispatched
        memory = dict(memory, dispatched=dispatched, available=max(0, available))
    summary["memory"] = memory
    gpu = report.get("gpu") or {"problem": "没报"}
    if "cards" in gpu:
        cards = []
        for card in gpu["cards"]:
            apps = [app for app in gpu.get("apps", []) if app["uuid"] == card["uuid"]]
            service = [app for app in apps if broken("others-as-services") or (app["unit"] and app["unit"] in units)]
            others = [app for app in apps if app not in service]
            service_mebibytes = sum(app["used"] for app in service)
            cards.append({"index": card["index"], "name": card["name"], "total": card["total"], "free": card["free"],
                          "service_used": service_mebibytes, "service_units": sorted({app["unit"] for app in service}),
                          "others": [{"pid": app["pid"], "used": app["used"]} for app in others],
                          "available": max(0, card["free"] - margin), "available_quiesced": max(0, card["free"] + service_mebibytes - margin)})
        gpu = {"cards": cards, "apps_problem": gpu.get("apps_problem")}
    summary["gpu"] = gpu
    return summary


def totals(summaries):
    def part(key, read):
        return [summary for summary in summaries if summary.get(key) and read(summary[key])]
    cpu_hosts = part("cpu", lambda value: "available_threads" in value)
    memory_hosts = part("memory", lambda value: "available" in value)
    gpu_hosts = part("gpu", lambda value: "cards" in value)
    cards = [card for summary in gpu_hosts for card in summary["gpu"]["cards"]]
    return {"threads": sum(summary["cpu"]["available_threads"] for summary in cpu_hosts),
            "memory": sum(summary["memory"]["available"] for summary in memory_hosts),
            "gpu_available": sum(card["available"] for card in cards),
            "gpu_available_quiesced": sum(card["available_quiesced"] for card in cards),
            "gpu_cards_usable": sum(1 for card in cards if card["available"] > 0),
            "gpu_cards_usable_quiesced": sum(1 for card in cards if card["available_quiesced"] > 0),
            "unread": {"cpu": [summary["name"] for summary in summaries if summary not in cpu_hosts],
                       "memory": [summary["name"] for summary in summaries if summary not in memory_hosts],
                       "gpu": [summary["name"] for summary in summaries if summary not in gpu_hosts]}}


# ── 给了命令：历史与逐台判 ──
def judge_command(options, summaries, command, gpu, shard=None, estimated_hours=None):
    hd = dispatcher()
    settings = options.settings
    prediction = hd.predict(settings, command, gpu)
    row = peaks.read_peak_rows(settings.peak_table).get(prediction.key)
    recorded_bytes, _ = peaks.recorded_peak(settings.peak_table, prediction.key)
    gpu_row = hd.read_gpu_peak_table(settings.gpu_peak_table).get(prediction.key) if gpu else None
    duration_seconds, duration_day = peaks.recorded_duration(settings.duration_table, prediction.key)
    history = {"key": prediction.key,
               "memory_peak": None if row is None else {"bytes": recorded_bytes, "latest_bytes": row[0], "recent_count": len(peaks.recent_of(row)),
                                                        "measured_on": row[1][:10], "cap": row[2], "hit": prediction.memory_history == "hit"},
               "gpu_peak": None if gpu_row is None else {"mebibytes": gpu_row[0], "measured_on": gpu_row[1][:10], "hit": gpu_row[2]},
               "duration": None if duration_seconds is None else {"seconds": duration_seconds, "measured_on": duration_day},
               "duration_note": ("没有历史（用时表里没有这条：只有经 host-dispatch.py 派出去、跑完的才记）" if duration_seconds is None
                                 else f"{duration_seconds / 3600:.1f} 小时（{duration_day} 量的，经 host-dispatch.py 派的那一次）")}
    usable_hosts = sum(1 for summary in summaries if summary["read"])
    long_run = hd.long_run_check(settings, command, prediction.key, shard, estimated_hours, usable_hosts, use_history=not broken("long-history-ignored"))
    long_run_result = {"hours": None if long_run.seconds is None else round(long_run.seconds / 3600, 2), "source": long_run.source,
                       "hosts": long_run.hosts, "why": long_run.why, "remind": long_run.refuse,
                       "reminder": (f"提醒：这条命令的{hd.long_run_text(long_run)}超过 2 小时，而派法只占一台（{long_run.why}）："
                                    f"{hd.LONG_RUN_HOWTO}；经 host-dispatch.py 派会被拒（退 {hd.EXIT_LONG_SINGLE_HOST}）") if long_run.refuse else None}
    holders = hd.live_quiesce_holders(settings) if gpu else 0
    entries = hd.live_dispatch_entries(settings)
    verdicts = []
    for summary in summaries:
        memory = summary.get("memory") or {}
        if not summary["read"] or "free" not in memory:
            verdicts.append({"name": summary["name"], "read": False, "fits": False,
                             "why": f"没读到，判不了（{summary.get('problem') or memory.get('problem') or '内存没读到'}）"})
            continue
        raw_gpu = (summary["raw"] or {}).get("gpu") or {}
        report = {"memory": memory, "quiesce_holders": holders,
                  "gpu": {"cards": [{"index": card["index"], "total": card["total"], "free": card["free"]} for card in raw_gpu["cards"]]}
                  if "cards" in raw_gpu else {"problem": raw_gpu.get("problem", "没读到")}}
        dispatch_name = "local" if summary["is_local"] else ("peer" if summary["is_peer"] else None)
        pending_memory, pending_gpu = hd.dispatch_pending(settings, dispatch_name, entries) if dispatch_name else (0, 0)
        verdict = hd.judge_host("local" if summary["is_local"] else "peer", report, prediction, gpu, pending_memory, pending_gpu,
                                peaks.size_bytes(summary["memory_cap"] or ""))
        verdicts.append({"name": summary["name"], "read": True, "fits": verdict.fits, "never": verdict.never, "cap": verdict.cap,
                         "memory_short": 0 if broken("no-shortfall") else verdict.memory_short,
                         "gpu_short": 0 if broken("no-shortfall") else verdict.gpu_short, "why": verdict.why})
    fitting = [verdict for verdict in verdicts if verdict["fits"]]
    chosen = (fitting[-1] if broken("wrong-host") else fitting[0]) if fitting else None
    if chosen:
        conclusion = f"可以跑：在{chosen['name']}（内存上限给 {peaks.human(chosen['cap'])}）"
    else:
        conclusion = "跑不起来：" + "；".join(f"{verdict['name']}{shortfall_text(verdict, with_why=False)}" for verdict in verdicts)
    return {"command": command, "gpu": gpu, "history": history, "long_run": long_run_result,
            "prediction": {"memory": prediction.memory_predicted, "memory_history": prediction.memory_history, "cap": prediction.memory_cap,
                           "gpu": prediction.gpu_predicted if gpu else None, "gpu_history": prediction.gpu_history if gpu else None},
            "verdicts": verdicts, "fits_on": chosen["name"] if chosen else None, "conclusion": conclusion}


def shortfall_text(verdict, with_why=True):
    if not verdict["read"]:
        return verdict["why"] if with_why else "没读到"
    if verdict["fits"]:
        return "放得下"
    pieces = []
    if verdict.get("memory_short"):
        pieces.append(f"差内存 {peaks.human(verdict['memory_short'])}")
    if verdict.get("gpu_short") is None:
        pieces.append("显存没读到")
    elif verdict.get("gpu_short"):
        pieces.append(f"差显存 {verdict['gpu_short']} MiB")
    return "放不下：" + ("、".join(pieces) if pieces else "差额没算") + (f"（{verdict['why']}）" if with_why else "")


def collect(options, command=None, gpu=False, shard=None, estimated_hours=None):
    machines, values, why_local_only = load_machines(options)
    units = quiesce_units(values, options)
    with concurrent.futures.ThreadPoolExecutor(max_workers=len(machines)) as pool:
        probes = list(pool.map(lambda machine: probe_machine(options, machine), machines))
    if len(probes) != len(machines):
        raise RuntimeError(f"派出去读 {len(machines)} 台、收回 {len(probes)} 台")
    entries = dispatcher().live_dispatch_entries(options.settings)
    summaries = [summarize_machine(options, machine, report, problem, units, entries) for machine, (report, problem) in zip(machines, probes)]
    result = {"hosts": summaries, "local_only_because": why_local_only or None, "totals": totals(summaries),
              "gpu_margin_mebibytes": dispatcher().GPU_MARGIN_MEBIBYTES}
    if command:
        result["command"] = judge_command(options, summaries, command, gpu, shard, estimated_hours)
    return result


# ── 给人看的表 ──
def render(result):
    gibibytes = peaks.human
    others = len(result["hosts"]) - 1
    lines = [f"各台此刻的资源（本机{f'与清单里 {others} 台' if others else ''}；读不到的写「没读到」，不当 0、不当满）"]
    if result["local_only_because"]:
        lines.append(f"  {result['local_only_because']}")
    for summary in result["hosts"]:
        if not summary["read"]:
            lines.append(f"{summary['name']}：没读到（{summary['problem']}）")
            continue
        lines.append(f"{summary['name']}：")
        cpu = summary["cpu"]
        if "available_threads" in cpu:
            load = " ".join(f"{value:g}" for value in cpu["loadavg"]) if "loadavg" in cpu else f"没读到：{cpu.get('loadavg_problem')}"
            lines.append(f"  线程：{cpu['cores']} 核，此刻在用约 {cpu['busy_cores']:g} 核（loadavg {load}），可用 {cpu['available_threads']} 线程")
        else:
            lines.append(f"  线程：没读到（{cpu.get('problem')}）")
        memory = summary["memory"]
        if "available" in memory:
            lines.append(f"  内存：可用 {gibibytes(memory['available'])}（MemAvailable {gibibytes(memory['mem_available'])}，减内存包装账上还没涨到量的 "
                         f"{gibibytes(memory['pending'])}、派发器刚派出去的 {gibibytes(memory['dispatched'])}）")
        else:
            lines.append(f"  内存：没读到（{memory.get('problem')}）")
        gpu = summary["gpu"]
        if "cards" not in gpu:
            lines.append(f"  卡：没读到（{gpu.get('problem')}）")
            continue
        if gpu.get("apps_problem"):
            lines.append(f"  卡上的计算进程：没读到（{gpu['apps_problem']}）")
        for card in gpu["cards"]:
            service = f"有清场服务占着 {card['service_used']} MiB" if card["service_used"] else "没有清场服务占着"
            if broken("service-name-shown") and card["service_units"]:
                service += f"（{'、'.join(card['service_units'])}）"
            others = "、".join(f"进程 {other['pid']} 占 {other['used']} MiB" for other in card["others"]) or "没有"
            lines.append(f"  卡 {card['index']} {card['name']}：总 {card['total']} MiB，空闲 {card['free']} MiB；{service}；别的计算进程：{others}；"
                         f"可用 不清场 {card['available']} MiB / 清场 {card['available_quiesced']} MiB")
    total = result["totals"]
    unread = "；".join(f"{label}不含{'、'.join(names)}" for label, names in (("线程", total["unread"]["cpu"]), ("内存", total["unread"]["memory"]),
                                                                          ("显存", total["unread"]["gpu"])) if names)
    lines.append(f"合计：可用 {total['threads']} 线程，内存 {gibibytes(total['memory'])}，显存 不清场 {total['gpu_available']} MiB"
                 f"（{total['gpu_cards_usable']} 张卡）/ 清场 {total['gpu_available_quiesced']} MiB（{total['gpu_cards_usable_quiesced']} 张卡）"
                 f"；每张卡已减留给驱动的 {result['gpu_margin_mebibytes']} MiB" + (f"（{unread}：没读到）" if unread else ""))
    judged = result.get("command")
    if judged:
        history = judged["history"]
        memory_peak = history["memory_peak"]
        memory_text = ("没有历史" if memory_peak is None else
                       f"{gibibytes(memory_peak['bytes'])}（近 {memory_peak['recent_count']} 次实测里最大的；最近一次 {memory_peak['measured_on']} 量的，"
                       f"那一次上限 {memory_peak['cap']}{'，撞过顶' if memory_peak['hit'] else ''}）")
        gpu_peak = history["gpu_peak"]
        gpu_text = ("" if not judged["gpu"] else "；显存峰值 " + ("没有历史" if gpu_peak is None else
                                                                 f"{gpu_peak['mebibytes']} MiB（{gpu_peak['measured_on']} 量的{'，撞过顶' if gpu_peak['hit'] else ''}）"))
        prediction = judged["prediction"]
        lines.append(f"这条命令（键「{history['key'][:120]}」）：")
        lines.append(f"  历史：内存峰值 {memory_text}{gpu_text}；用时 {history['duration_note']}")
        lines.append(f"  预测：内存 {gibibytes(prediction['memory'])}（{prediction['memory_history']}，上限 {gibibytes(prediction['cap'])}）"
                     + (f"，显存 {prediction['gpu']} MiB（{prediction['gpu_history']}；判法同 host-dispatch.py --plan：还没清场按卡的总量算）" if judged["gpu"] else ""))
        for verdict in judged["verdicts"]:
            lines.append(f"  {verdict['name']}：{shortfall_text(verdict)}")
        if judged["long_run"]["reminder"]:
            lines.append(judged["long_run"]["reminder"])
        lines.append(f"结论：{judged['conclusion']}")
    return "\n".join(lines)


def public_json(result):
    """--json：去掉每台的原始读数与清场服务的单元名（单元名是私有的）。"""
    hosts = []
    for summary in result["hosts"]:
        summary = {key: value for key, value in summary.items() if key != "raw"}
        if isinstance(summary.get("gpu"), dict) and "cards" in summary["gpu"]:
            summary["gpu"] = dict(summary["gpu"], cards=[{key: value for key, value in card.items() if key != "service_units"} for card in summary["gpu"]["cards"]])
        hosts.append(summary)
    return dict(result, hosts=hosts)


def parse_arguments(arguments):
    """(json, gpu, 命令或 None, shard, estimated_hours)；用法错抛 ValueError。--shard、--estimated-hours 的写法照 host-dispatch.py 的 parse_shard、parse_hours。"""
    as_json, gpu, command, shard, estimated_hours = False, False, None, None, None
    index = 0
    while index < len(arguments):
        word = arguments[index]
        if word == "--":
            command = arguments[index + 1:]
            if not command:
                raise ValueError("-- 后面没有命令")
            break
        if word == "--json":
            as_json = True
        elif word == "--gpu":
            gpu = True
        elif word in ("--shard", "--estimated-hours"):
            if index + 1 >= len(arguments):
                raise ValueError(f"{word} 后面没给值")
            if word == "--shard":
                shard = dispatcher().parse_shard(arguments[index + 1])
            else:
                estimated_hours = dispatcher().parse_hours(arguments[index + 1])
            index += 1
        elif not broken("no-usage-check"):
            raise ValueError(f"认不出的参数「{word}」（命令要写在 -- 后面）")
        index += 1
    if (gpu or shard is not None or estimated_hours is not None) and command is None:
        raise ValueError("--gpu、--shard、--estimated-hours 只跟着 -- <命令…> 用")
    return as_json, gpu, command, shard, estimated_hours


def main(arguments, options=None, output=None):
    output = output or (lambda text: print(text, flush=True))
    try:
        as_json, gpu, command, shard, estimated_hours = parse_arguments(arguments)
        options = options or options_from_environment()
    except ValueError as error:
        print(f"  ✗ host-resources.py 用法错：{error}", file=sys.stderr)
        print("  → 怎么办：host-resources.py [--json]，或 host-resources.py [--json] [--gpu] [--shard <k>/<n>] [--estimated-hours <小时>] -- <命令…>、--selftest",
              file=sys.stderr)
        return EXIT_USAGE
    result = collect(options, command, gpu, shard, estimated_hours)
    output(json.dumps(public_json(result), ensure_ascii=False) if as_json else render(result))
    return 1 if command and result["command"]["fits_on"] is None else 0


# ── 自证 ──
FAKE_SSH = """#!/usr/bin/env bash
# 假 ssh：去掉 -o 选项，按主机名找 hosts/<名>/；有 unreachable 就照连不上退 255，有 slow 就先睡它写的秒数；把命令照 ssh 那样交给 bash -c，
# /proc、/proc/meminfo、包装的状态目录与 nvidia-smi 换成那台的假的
while [[ "${1:-}" == -o ]]; do shift 2; done
host="$1"; shift
printf '%s\\n' "$host" >> "$HOST_RESOURCES_SELFTEST_SSH_LOG"
here="$HOST_RESOURCES_SELFTEST_ROOT/hosts/$host"
if [[ ! -d "$here" || -e "$here/unreachable" ]]; then echo "ssh: connect to host $host port 22: Connection refused" >&2; exit 255; fi
[[ -e "$here/slow" ]] && sleep "$(cat "$here/slow")"
exec env PATH="$here/bin:$PATH" HOST_RESOURCES_PROC_ROOT="$here/proc" SINGLEFS_DISPATCH_PROBE_MEMINFO="$here/proc/meminfo" \\
  RUN_WITH_MEMORY_CAP_STATE_DIR="$here/state" bash -c "$*"
"""
SERVICE_UNIT = "selftest-model-server.service"
GIBIBYTE = 1024 ** 3


class SelftestWorld:
    """一格一个：临时目录里的本机、清单里的几台、多机配置、清场命令与峰值表。"""

    def __init__(self, root, name, remote_hosts=("selftest-peer-host-name",), listed=False, across_machines=True):
        self.directory = os.path.join(root, name)
        self.hosts = os.path.join(self.directory, "hosts")
        self.local = os.path.join(self.directory, "local")
        self.remote = {}
        for alias in remote_hosts:
            self.remote[alias] = os.path.join(self.hosts, alias)
        for host in [self.local, *self.remote.values()]:
            os.makedirs(os.path.join(host, "bin"))
            os.makedirs(os.path.join(host, "proc"))
            os.makedirs(os.path.join(host, "state", "ledger"))
            self.write_executable(os.path.join(host, "bin", "nvidia-smi"), dispatcher().FAKE_NVIDIA_SMI)   # 读同目录的 gpus.csv、apps.csv
            self.cpu(host, 8, 0)
            self.meminfo(host, 60, 40)
            self.cards(host, [(0, "Selftest GPU", 16384, 16384)])
        os.makedirs(os.path.join(self.directory, "ssh-bin"))
        self.write_executable(os.path.join(self.directory, "ssh-bin", "ssh"), FAKE_SSH)
        self.write_executable(os.path.join(self.directory, "quiesce-stop.sh"),
                              f"#!/usr/bin/env bash\n# 假清场：白名单只有一个单元\nsystemctl stop -- {SERVICE_UNIT}\n")
        self.peak_table = os.path.join(self.directory, "memory-peaks.tsv")
        self.gpu_peak_table = os.path.join(self.directory, "gpu-peaks.tsv")
        self.duration_table = os.path.join(self.directory, "durations.tsv")
        self.ssh_log = os.path.join(self.directory, "ssh.log")
        self.configuration(list(remote_hosts), listed, across_machines)

    @staticmethod
    def write_executable(path, text):
        dispatcher().SelftestWorld.write_executable(path, text)

    @staticmethod
    def cpu(host, cores, busy_cores):
        """两份 /proc/stat：每核 100 个滴答里有 busy_cores 核的忙着（stat.later 是一秒之后那一读）。"""
        def stat(scale):
            idle = (cores - busy_cores) * 100 * scale
            busy = busy_cores * 100 * scale
            lines = [f"cpu  {busy} 0 0 {idle} 0 0 0 0 0 0"] + [f"cpu{index} 0 0 0 0 0 0 0 0 0 0" for index in range(cores)]
            return "\n".join(lines) + "\nintr 0\n"
        with open(os.path.join(host, "proc", "stat"), "w", encoding="utf-8") as handle:
            handle.write(stat(1))
        with open(os.path.join(host, "proc", "stat.later"), "w", encoding="utf-8") as handle:
            handle.write(stat(2))
        with open(os.path.join(host, "proc", "loadavg"), "w", encoding="utf-8") as handle:
            handle.write(f"{busy_cores}.00 {busy_cores}.00 {busy_cores}.00 1/100 4242\n")

    @staticmethod
    def meminfo(host, total_gibibytes, available_gibibytes):
        dispatcher().SelftestWorld.meminfo(os.path.join(host, "proc"), total_gibibytes, available_gibibytes)   # 写 <host>/proc/meminfo

    @staticmethod
    def cards(host, cards, apps=()):
        """cards：(序号, 型号, 总 MiB, 空闲 MiB)；apps：(进程号, 占 MiB, 卡序号, cgroup 那一行)。"""
        with open(os.path.join(host, "bin", "gpus.csv"), "w", encoding="utf-8") as handle:
            handle.write("".join(f"{index}, {name}, GPU-selftest-{index}, {total}, {free}\n" for index, name, total, free in cards))
        with open(os.path.join(host, "bin", "apps.csv"), "w", encoding="utf-8") as handle:
            handle.write("".join(f"{pid}, {used}, GPU-selftest-{index}\n" for pid, used, index, _ in apps))
        for pid, _, _, cgroup in apps:
            os.makedirs(os.path.join(host, "proc", str(pid)), exist_ok=True)
            with open(os.path.join(host, "proc", str(pid), "cgroup"), "w", encoding="utf-8") as handle:
                handle.write(cgroup + "\n")

    def configuration(self, remote_hosts, listed, across_machines, extra=""):
        first = remote_hosts[0] if remote_hosts else "selftest-peer-host-name"
        stop = os.path.join(self.directory, "quiesce-stop.sh")
        with open(os.path.join(self.directory, "multi-host.env"), "w", encoding="utf-8") as handle:
            handle.write(f"PEER_SSH_HOST={first}\nPEER_REPOSITORY_DIRECTORY=/selftest/peer/home\nPEER_CARGO_BIN_DIRECTORY=/selftest/peer/bin\n"
                         f"PEER_MEMORY_CAP=40G\nQUIESCE_STOP_COMMAND='bash {stop}'\nQUIESCE_STOPPED_CHECK_COMMAND=true\nQUIESCE_START_COMMAND=true\n"
                         f"QUIESCE_STARTED_CHECK_COMMAND=true\nENABLE_ACROSS_MACHINES={1 if across_machines else 0}\n"
                         + (f"HOSTS={' '.join(remote_hosts)}\n" if listed else "") + extra)

    def wrapper_ledger_entry(self, host, need_bytes):
        """包装账上一笔活着的，还没涨到量的一截就是整个 need：同 host-dispatch.py 自证的那一份（它不读 self）。"""
        dispatcher().SelftestWorld.wrapper_ledger_entry(None, host, need_bytes)

    def seed_peak(self, key, peak_bytes, cap_text):
        with open(self.peak_table, "a", encoding="utf-8") as handle:
            handle.write(f"{peak_bytes}\t{dispatcher().SELFTEST_MEASURED_AT}\t{cap_text}\t{key}\n")

    def seed_gpu_peak(self, key, mebibytes):
        with open(self.gpu_peak_table, "a", encoding="utf-8") as handle:
            handle.write(f"{mebibytes}\t{dispatcher().SELFTEST_MEASURED_AT}\t0\tlocal\t{key}\n")

    def seed_duration(self, key, hours):
        dispatcher().SelftestWorld.seed_duration(None, key, hours, self.duration_table)   # 同 host-dispatch.py 自证的写法，表是这一格自己的

    def options(self, host_timeout_seconds=HOST_TIMEOUT_SECONDS):
        environment = {name: value for name, value in os.environ.items()
                       if not name.startswith(("RUN_WITH_MEMORY_CAP_", "SINGLEFS_", "GPU_QUIESCE_", "HOST_DISPATCH_GPU", "HOST_DISPATCH_STATE",
                                               "MULTI_HOST_", "PEER_HOST_LIB_", "HOST_RESOURCES_", "CLAUDE_CODE_SESSION_ID"))}
        if os.environ.get("MULTI_HOST_CONFIGURATION_BREAK"):
            environment["MULTI_HOST_CONFIGURATION_BREAK"] = os.environ["MULTI_HOST_CONFIGURATION_BREAK"]
        environment.update({
            "PATH": os.pathsep.join([os.path.join(self.local, "bin"), os.path.join(self.directory, "ssh-bin"), os.environ.get("PATH", "")]),
            "HOST_RESOURCES_PROC_ROOT": os.path.join(self.local, "proc"),
            "SINGLEFS_DISPATCH_PROBE_MEMINFO": os.path.join(self.local, "proc", "meminfo"),
            "RUN_WITH_MEMORY_CAP_STATE_DIR": os.path.join(self.local, "state"),
            "RUN_WITH_MEMORY_CAP_SLICE": "singlefs_host_resources_selftest_absent.slice",
            "RUN_WITH_MEMORY_CAP_RESERVE": "20G",
            "SINGLEFS_MULTI_HOST_CONFIG": os.path.join(self.directory, "multi-host.env"),
            "HOST_RESOURCES_SELFTEST_ROOT": self.directory,
            "HOST_RESOURCES_SELFTEST_SSH_LOG": self.ssh_log,
        })
        hd = dispatcher()
        settings = hd.Settings(
            repository_root=os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY)),
            multi_host_run="/nonexistent/multi-host-run.sh", gpu_quiesce_run="/nonexistent/gpu-quiesce-run.sh",
            configuration_check=os.path.join(SCRIPT_DIRECTORY, "..", "crash", "crash-point-replay-shard-configuration-check.sh"),
            peer_host_lib=os.path.join(SCRIPT_DIRECTORY, "peer-host-lib.sh"),
            peak_table=self.peak_table, gpu_peak_table=self.gpu_peak_table,
            state_directory=os.path.join(self.directory, "dispatch-state"), quiesce_state_directory=os.path.join(self.directory, "quiesce-state"),
            environment=environment, duration_table=self.duration_table)
        return Options(settings=settings,
                       sample_seconds=0.0, host_timeout_seconds=host_timeout_seconds)

    def ssh_calls(self):
        with contextlib.suppress(FileNotFoundError):
            with open(self.ssh_log, encoding="utf-8") as handle:
                return handle.read().splitlines()
        return []


def selftest_cells(root):
    """各格：交 (失败的格名列表, 查了几格)。每格自己一个 SelftestWorld，互不相干。"""
    failures = []
    checked = 0

    def check(cell, condition, detail):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append(cell)
            print(f"  ✗ 自证 {cell}：{detail}")   # gate-lint:detail

    def run(world, command=None, gpu=False, host_timeout_seconds=HOST_TIMEOUT_SECONDS, shard=None, estimated_hours=None):
        result = collect(world.options(host_timeout_seconds), command, gpu, shard, estimated_hours)
        return result, render(result)

    def brief(result):
        return "；".join(f"{summary['name']} 读到={summary['read']} 线程={(summary.get('cpu') or {}).get('available_threads')} "
                        f"内存={(summary.get('memory') or {}).get('available')}" for summary in result["hosts"])

    # both-hosts：两台都在；可用线程减此刻在用的核，可用内存减包装账上还没涨到量的；合计两台相加；表里不出现别名
    world = SelftestWorld(root, "both-hosts")
    alias = "selftest-peer-host-name"
    world.cpu(world.local, 8, 2)
    world.cpu(world.remote[alias], 16, 4)
    world.wrapper_ledger_entry(world.local, 4 * GIBIBYTE)
    world.meminfo(world.remote[alias], 60, 30)
    result, text = run(world)
    local, peer = (result["hosts"] + [{}, {}])[:2]
    json_lines = []
    json_code = main(["--json"], world.options(), json_lines.append)
    parsed = json.loads(json_lines[0]) if json_code == 0 and json_lines else {}
    check("both-hosts", len(result["hosts"]) == 2 and local["cpu"]["available_threads"] == 6 and peer.get("cpu", {}).get("available_threads") == 12
          and local["memory"]["available"] == 36 * GIBIBYTE and peer["memory"]["available"] == 30 * GIBIBYTE
          and result["totals"]["threads"] == 18 and result["totals"]["memory"] == 66 * GIBIBYTE
          and local["gpu"]["cards"][0]["available"] == 16384 - dispatcher().GPU_MARGIN_MEBIBYTES
          and "第二台：" in text and alias not in text and len(parsed.get("hosts", [])) == 2 and alias not in json_lines[0],
          f"本机应当可用 6 线程、36 GiB，第二台 12 线程、30 GiB，合计 18 线程、66 GiB，--json 两台、不出别名；实际 {brief(result)}，"
          f"合计 {result['totals']['threads']} 线程、{result['totals']['memory']}，--json 退 {json_code}")

    # peer-unreachable：第二台连不上 ⇒ 那一台「没读到」，各项是空的（不当 0），合计只算本机并写明不含它
    world = SelftestWorld(root, "peer-unreachable")
    open(os.path.join(world.remote[alias], "unreachable"), "w").close()
    result, text = run(world)
    peer = (public_json(result)["hosts"] + [{}])[1]
    check("peer-unreachable", peer.get("read") is False and peer.get("cpu", 0) is None and peer.get("memory", 0) is None and "第二台：没读到" in text
          and result["totals"]["threads"] == 8 and "第二台" in result["totals"]["unread"]["cpu"] and alias not in text,
          f"第二台应当没读到、各项为空、合计只算本机 8 线程并写明不含第二台；实际 {brief(result)}，合计 {result['totals']}")

    # host-timeout：第二台回话太慢 ⇒ 到时限记「没读到」，不等它
    world = SelftestWorld(root, "host-timeout")
    with open(os.path.join(world.remote[alias], "slow"), "w", encoding="utf-8") as handle:
        handle.write("3\n")
    started = time.monotonic()
    result, text = run(world, host_timeout_seconds=1)
    elapsed = time.monotonic() - started
    peer = (result["hosts"] + [{}])[1]
    check("host-timeout", elapsed < 2.5 and peer.get("read") is False and "1 秒没回话" in (peer.get("problem") or ""),
          f"时限 1 秒、第二台 3 秒才回话：应当 2.5 秒内报完、第二台「1 秒没回话」；实际用了 {elapsed:.1f} 秒，{brief(result)}")

    # card-other-process：卡上有别的会话的计算进程 ⇒ 列出进程号与占用，清不清场都照占着算
    world = SelftestWorld(root, "card-other-process")
    world.cards(world.local, [(0, "Selftest GPU", 16384, 10000)], [(4242, 6000, 0, "0::/user.slice/user-1000.slice/session-1.scope")])
    result, text = run(world)
    card = result["hosts"][0]["gpu"]["cards"][0]
    margin = dispatcher().GPU_MARGIN_MEBIBYTES
    check("card-other-process", card["others"] == [{"pid": 4242, "used": 6000}] and card["service_used"] == 0
          and card["available"] == 10000 - margin and card["available_quiesced"] == 10000 - margin and "进程 4242 占 6000 MiB" in text,
          f"别的进程 4242 占 6000 MiB：应当列出它，不清场与清场都可用 {10000 - margin} MiB；实际 {card}")

    # quiesce-service：清场白名单里的服务占着卡 ⇒ 报「有清场服务占着 N MiB」、清场那份算上它，不报服务名
    world = SelftestWorld(root, "quiesce-service")
    world.cards(world.local, [(0, "Selftest GPU", 16384, 2000)], [(5151, 14000, 0, f"0::/system.slice/{SERVICE_UNIT}")])
    result, text = run(world)
    card = result["hosts"][0]["gpu"]["cards"][0]
    check("quiesce-service", card["service_used"] == 14000 and card["others"] == [] and card["available"] == 0
          and card["available_quiesced"] == 16000 - margin and "有清场服务占着 14000 MiB" in text and SERVICE_UNIT not in text
          and SERVICE_UNIT not in json.dumps(public_json(result), ensure_ascii=False),
          f"清场服务占 14000 MiB：应当不清场可用 0、清场可用 {16000 - margin} MiB，表与 --json 里没有单元名；实际 {card}，表里有单元名={SERVICE_UNIT in text}")

    # switch-off：跨机开关没开 ⇒ 只报本机，一次 ssh 都不起
    world = SelftestWorld(root, "switch-off", across_machines=False)
    result, text = run(world)
    check("switch-off", len(result["hosts"]) == 1 and not world.ssh_calls() and "ENABLE_ACROSS_MACHINES" in (result["local_only_because"] or ""),
          f"跨机开关没开：应当只报本机、不起 ssh；实际 {len(result['hosts'])} 台，ssh 起了 {len(world.ssh_calls())} 次，原因「{result['local_only_because']}」")

    # hosts-list：配置里写了机器清单（两台，第三台连不上、配了自己的内存上限）⇒ 三台都报，第三台「没读到」，配置判法不判整份坏
    world = SelftestWorld(root, "hosts-list", remote_hosts=("selftest-host-two", "selftest-host-three"), listed=True)
    world.configuration(["selftest-host-two", "selftest-host-three"], True, True, extra="HOST_3_MEMORY_CAP=8G\n")
    open(os.path.join(world.remote["selftest-host-three"], "unreachable"), "w").close()
    result, text = run(world)
    names = [summary["name"] for summary in result["hosts"]]
    check("hosts-list", names == ["本机", "第二台", "第三台"] and result["hosts"][1]["read"] and not result["hosts"][2]["read"]
          and result["hosts"][2]["memory_cap"] == "8G" and "第三台：没读到" in text and not result["local_only_because"]
          and "selftest-host-three" not in text,
          f"应当报本机、第二台、第三台，第三台没读到、上限 8G；实际只报了 {'、'.join(names)}（{brief(result)}；只报本机的原因「{result['local_only_because']}」）")

    # fits-on-host：给了命令、有历史 ⇒ 本机放不下、第二台第三台都放得下，结论取清单里第一台放得下的
    world = SelftestWorld(root, "fits-on-host", remote_hosts=("selftest-host-two", "selftest-host-three"), listed=True)
    world.meminfo(world.local, 60, 10)
    world.meminfo(world.remote["selftest-host-two"], 60, 30)
    world.meminfo(world.remote["selftest-host-three"], 60, 50)
    world.seed_peak("selftest-command", 20 * GIBIBYTE, "32G")
    result, text = run(world, ["selftest-command"])
    judged = result["command"]
    check("fits-on-host", judged["fits_on"] == "第二台" and judged["history"]["memory_peak"]["bytes"] == 20 * GIBIBYTE
          and not judged["verdicts"][0]["fits"] and "结论：可以跑：在第二台" in text and "没有历史" in judged["history"]["duration_note"],
          f"历史峰值 20 GiB：本机（空闲 10 GiB）放不下、应当判在第二台；实际结论「{judged['conclusion']}」，历史 {judged['history']['memory_peak']}")

    # nothing-fits：哪台都放不下 ⇒ 退 1，逐台报差多少内存、差多少显存
    world = SelftestWorld(root, "nothing-fits")
    world.meminfo(world.local, 60, 10)
    world.meminfo(world.remote[alias], 60, 30)
    world.seed_peak("selftest-command", 50 * GIBIBYTE, "64G")
    world.seed_gpu_peak("selftest-command", 20000)
    lines = []
    code = main(["--gpu", "--", "selftest-command"], world.options(), lines.append)
    result, text = run(world, ["selftest-command"], gpu=True)
    verdicts = result["command"]["verdicts"]
    gpu_short = 20000 - (16384 - margin)
    check("nothing-fits", code == 1 and result["command"]["fits_on"] is None and verdicts[0]["memory_short"] == 42 * GIBIBYTE
          and verdicts[0]["gpu_short"] == gpu_short and len(verdicts) == 2 and verdicts[1].get("memory_short") == 10 * GIBIBYTE
          and "跑不起来" in text and "差内存 42.0 GiB" in text and f"差显存 {gpu_short} MiB" in text,
          f"峰值 50 GiB、显存 20000 MiB：应当退 1，本机差内存 42 GiB、差显存 {gpu_short} MiB，第二台差内存 10 GiB（收到 PEER_MEMORY_CAP 40G）；"
          f"实际退 {code}，{[(verdict['name'], verdict.get('memory_short'), verdict.get('gpu_short')) for verdict in verdicts]}")

    # long-history-reminder：用时表里这条上一次跑了 3 小时，没给 --shard ⇒ 结论之前报一行提醒（点名历史用时与出路），退出码照旧（放得下退 0）
    world = SelftestWorld(root, "long-history-reminder")
    world.seed_peak("selftest-command", GIBIBYTE, "4G")
    world.seed_duration("selftest-command", 3)
    lines = []
    code = main(["--", "selftest-command"], world.options(), lines.append)
    text = "\n".join(lines)
    check("long-history-reminder", code == 0 and "提醒：这条命令的历史用时 3.0 小时" in text and "crash-point-replay-shard-configuration-check.sh --emit-hosts" in text
          and ".claude/heavy-thread-default" in text and "为什么分不了" in text and "用时 3.0 小时" in text,
          f"历史用时 3 小时、只占一台应当报提醒（历史用时与出路）、退 0；实际退 {code}，末尾几行：{lines[-1].splitlines()[-3:] if lines else '没打'}")

    # long-estimate-reminder：用时表里没有，给了 --estimated-hours 3 ⇒ 按预计挂钟报提醒
    world = SelftestWorld(root, "long-estimate-reminder")
    world.seed_peak("selftest-command", GIBIBYTE, "4G")
    result, text = run(world, ["selftest-command"], estimated_hours=3.0)
    check("long-estimate-reminder", result["command"]["long_run"]["remind"] and "提醒：这条命令的预计挂钟 3.0 小时" in text,
          f"没有历史、预计挂钟 3 小时应当报提醒；实际 {result['command']['long_run']}")

    # short-history-no-reminder：上一次跑了 1.5 小时 ⇒ 不提醒
    world = SelftestWorld(root, "short-history-no-reminder")
    world.seed_peak("selftest-command", GIBIBYTE, "4G")
    world.seed_duration("selftest-command", 1.5)
    result, text = run(world, ["selftest-command"])
    check("short-history-no-reminder", not result["command"]["long_run"]["remind"] and "提醒：" not in text and result["command"]["fits_on"] is not None,
          f"历史用时 1.5 小时不该提醒；实际 {result['command']['long_run']}，结论「{result['command']['conclusion']}」")

    # two-host-shards-no-reminder：上一次跑了 3 小时，--shard 1/2 而两台都读到 ⇒ 不提醒
    world = SelftestWorld(root, "two-host-shards-no-reminder")
    world.seed_peak("selftest-command", GIBIBYTE, "4G")
    world.seed_duration("selftest-command", 3)
    result, text = run(world, ["selftest-command"], shard=(1, 2))
    check("two-host-shards-no-reminder", len(result["hosts"]) == 2 and not result["command"]["long_run"]["remind"] and "提醒：" not in text,
          f"历史用时 3 小时、--shard 1/2、两台都读到不该提醒；实际 {len(result['hosts'])} 台，{result['command']['long_run']}")

    # usage：认不出的参数 ⇒ 退 2
    world = SelftestWorld(root, "usage")
    code = main(["--bogus"], world.options(), lambda line: None)
    check("usage", code == EXIT_USAGE, f"认不出的参数应当退 {EXIT_USAGE}，实际退 {code}")
    return failures, checked


# 弄坏开关 → 打开后至少要红的格（文件头「弄坏开关」那一段，两处要一致）
SWITCH_EXPECTED_CELLS = {
    "HOST_RESOURCES_BREAK=busy-not-subtracted": ["both-hosts"],
    "HOST_RESOURCES_BREAK=pending-not-subtracted": ["both-hosts"],
    "HOST_RESOURCES_BREAK=unread-as-zero": ["peer-unreachable", "host-timeout", "hosts-list"],
    "HOST_RESOURCES_BREAK=no-host-timeout": ["host-timeout"],
    "HOST_RESOURCES_BREAK=services-as-others": ["quiesce-service"],
    "HOST_RESOURCES_BREAK=others-as-services": ["card-other-process"],
    "HOST_RESOURCES_BREAK=service-name-shown": ["quiesce-service"],
    "HOST_RESOURCES_BREAK=peer-despite-switch-off": ["switch-off"],
    "HOST_RESOURCES_BREAK=first-host-only": ["hosts-list"],
    "HOST_RESOURCES_BREAK=wrong-host": ["fits-on-host"],
    "HOST_RESOURCES_BREAK=no-shortfall": ["nothing-fits"],
    "HOST_RESOURCES_BREAK=no-usage-check": ["usage"],
    "HOST_RESOURCES_BREAK=long-history-ignored": ["long-history-reminder"],
    "HOST_DISPATCH_BREAK=ignore-history": ["fits-on-host", "nothing-fits"],
    "HOST_DISPATCH_BREAK=long-estimate-ignored": ["long-estimate-reminder"],
    "HOST_DISPATCH_BREAK=shards-as-single": ["two-host-shards-no-reminder"],
    "MULTI_HOST_CONFIGURATION_BREAK=hosts-key-unknown": ["hosts-list", "fits-on-host"],
    "MULTI_HOST_CONFIGURATION_BREAK=hosts-reach-refuses": ["hosts-list", "peer-unreachable", "host-timeout"],
}


def run_cells_in_scratch():
    root = tempfile.mkdtemp(prefix="host-resources-selftest-")
    try:
        return selftest_cells(root)
    finally:
        shutil.rmtree(root, ignore_errors=True)


def selftest():
    failures, checked = run_cells_in_scratch()
    if failures:
        print(f"  ✗ host-resources.py 自证 {len(failures)} 格不对（查了 {checked} 格）：{'、'.join(failures)}")
        print("  → 怎么办：照上面逐格改 probe_this_host / summarize_machine / judge_command / render；弄坏开关设着的话本来就该红")
        return 1
    jobs = max(1, min(len(SWITCH_EXPECTED_CELLS), int(os.environ.get("SINGLEFS_THREAD_CAP") or 0) or os.cpu_count() or 1))
    clean = {name: value for name, value in os.environ.items() if name not in BREAK_VARIABLES}

    def one(switch):
        variable, item = switch.split("=", 1)
        completed = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--selftest-cells"], capture_output=True, text=True,
                                   env=dict(clean, **{variable: item}), timeout=600, stdin=subprocess.DEVNULL)
        red = set(line.split("：", 1)[0].removeprefix("  ✗ 自证 ") for line in completed.stdout.splitlines() if line.startswith("  ✗ 自证 "))
        return switch, red

    switch_failures = []
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        results = list(pool.map(one, SWITCH_EXPECTED_CELLS))
    if len(results) != len(SWITCH_EXPECTED_CELLS):
        switch_failures.append(f"开关轮派出 {len(SWITCH_EXPECTED_CELLS)} 个、收回 {len(results)} 个")
    for switch, red in results:
        print(f"  · 弄坏开关 {switch}：红在 {'、'.join(sorted(red)) or '一格都没有'}")
        missing = [cell for cell in SWITCH_EXPECTED_CELLS[switch] if cell not in red]
        if missing:
            switch_failures.append(f"{switch} 没红在 {'、'.join(missing)}（红的是 {'、'.join(sorted(red)) or '一格都没有'}）")
    for failure in switch_failures:
        print(f"  ✗ 弄坏开关 {failure}")   # gate-lint:detail
    if switch_failures:
        print("  → 怎么办：照上面那几个开关看它在代码里弄坏的那一处还在不在判法上；改了判法就改 SWITCH_EXPECTED_CELLS 与文件头的表")
        return 1
    print(f"  ✓ host-resources.py 自证通过（查了 {checked} 格：两台都在、第二台连不上、第二台回话超时、卡被别的进程占着、清场服务占着卡、"
          f"跨机开关没开、机器清单三台里一台连不上、给了命令放得下判对台、哪台都放不下报差额、历史用时或预计挂钟超过 2 小时而只占一台报提醒、"
          f"不超过 2 小时与两台分片不提醒、用法错；弄坏开关 {len(results)} 个逐个打开都红在点名的格）")
    return 0


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    if sys.argv[1:] == ["--selftest-cells"]:
        cell_failures, _ = run_cells_in_scratch()
        sys.exit(1 if cell_failures else 0)
    switched = [name for name in BREAK_VARIABLES if os.environ.get(name)]
    if switched:
        print(f"  ✗ 弄坏开关 {'、'.join(switched)} 设在 --selftest 之外，什么都没查", file=sys.stderr)
        print("  → 怎么办：去掉这几个变量再跑；它们只给各自脚本的 --selftest 证红用", file=sys.stderr)
        sys.exit(EXIT_USAGE)
    sys.exit(main(sys.argv[1:]))
