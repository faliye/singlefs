#!/usr/bin/env python3
# admission: always 每一次调都现读本机事件目录与多机配置，上一次的结论不替这一次作保
# run-condition: command bash
"""资源释放广播：占资源、会排队的脚本跑完时发一条事件，先写本机事件目录，再按多机配置推给别的机器；排队的一方等「新事件或到点」先到者，
到了就重判（用户 2026-10-23 定「叫醒并重判，不自动挪」，records/2026-09-16-subagent拆分提案.md 第九十九节）。发与收都只在这一份里。
被 import 的函数库，也能直接执行；被 import 时不读文件、不起子进程。

谁发（各自跑完时经 emit 发一条，发不出去、推不出去都不改它们的退出码）：
  scripts/hosts/run-with-memory-cap.sh     命令真跑过之后、包装退出那一刻：memory-released（排队没放行就退的不发）
  scripts/crash/gpu-quiesce-run.sh         这一趟命令起过、出去时删掉自己的 holders/ 登记之后：gpu-released（带出去时别的活着的登记数）
  scripts/hosts/multi-host-run.sh --peer   本机这一侧等到第二台那一趟结束之后由本机代发：peer-run-finished（--skip-peer：不推给第二台自己）
  scripts/hosts/detached-run.sh            脱离单元里那个 bash 写完退出码之后：detached-unit-finished
谁收：
  scripts/hosts/run-with-memory-cap.sh     排队时经 follow 收（HostLookup.watch_events 起一个、只停自己起的这一个），收到别的机器推来的或第二台那一趟跑完的，
                                           当场再跑一次「排队时去找别的机器」那一套查法（它文件头那一段）；不改排队本身
  scripts/hosts/host-dispatch.py           两台都放不下时经 wait_for_events 等「新事件或 POLL_SECONDS 到点」先到者，到了就重查

用法：
  resource_release_broadcast.py emit --kind <种类> --label <标签> [--exit-code <码>] [--memory-bytes <字节>] [--gpu-mebibytes <MiB>]
                                     [--holders-left <个数>] [--host 本机|第二台|第三台|…|第 <号> 台] [--skip-peer] [--problems-to-log]
      写本机事件目录一条、再推给别的机器；总是退 0（用法错退 2）。推不出去的每一台往 stderr 打一行「  ! 没广播到<第几台>：<原因>…」，
      带 --problems-to-log 的改写进本机事件目录里的 push-problems.log（至多留 PROBLEM_LOG_LINES 行）、stderr 一行不打（内存包装用：判法读包装的 stderr）。
      --skip-peer：不推给多机配置里 PEER_SSH_HOST 那一台。--host 默认「本机」：事件说的是哪一台的资源。
  resource_release_broadcast.py latest
      打本机事件目录此刻最新的序号（没有事件打 0），退 0
  resource_release_broadcast.py wait --after <序号> --timeout <秒>
      等到本机事件目录里有比 <序号> 新的事件、或到点：有就逐行打那几条、退 0；到点没有退 3。shell 那边的等法就是它
  resource_release_broadcast.py follow [--after <序号> | --since <epoch 秒>]
      一直跑：每来一条新事件打一行（立刻刷出）；--since 给了，time 不早于它的事件都算新的（起它的一方把「从哪一刻起等」交过来，
      不因 follow 起得慢漏掉起动那一会儿来的）；两样都不给从此刻最新的那一条之后起。起它的进程没了（父进程号变了）就退出
  resource_release_broadcast.py --selftest
python 那边：latest_sequence(目录)、events_after(目录, 序号)、wait_for_events(序号, 秒, 目录) → [事件 dict]（到点没有交空列表）、default_directory(环境)。

事件：一条一个文件 <目录>/<十二位序号>.event，一行，字段 key=value、空格隔开（值里不带空白、等号与斜杠，标签里的这几样换成 _）：
  seq=<序号> kind=<种类> host=<哪一台> source=<local|pushed> label=<标签> [exit=<码>] [memory_bytes=<字节>] [gpu_mebibytes=<MiB>] [holders_left=<个数>]
  time=<epoch 秒> date=<YYYY-MM-DD，Asia/Tokyo>
  种类：memory-released（内存放出）、gpu-released（卡放出）、peer-run-finished（第二台那一趟跑完）、detached-unit-finished（脱离单元结束）。
  host 只写「本机」「第二台」「第三台」……「第 <号> 台」，不写主机名、ssh 别名与路径；推到别的机器的那一份 source=pushed，
  host 照发出那台的说法（发出那台自己是「本机」）。memory_bytes、gpu_mebibytes 取得到才带。
本机事件目录：RESOURCE_BROADCAST_DIRECTORY 给了照它（自证与调用方的假世界用）；否则 ${XDG_RUNTIME_DIR}/singlefs-resource-events，XDG_RUNTIME_DIR 没设时
  /tmp/singlefs-resource-events-<uid>（目录 700）。写法：拿目录里 lock 的 flock，序号取 last-sequence 记的加一（没有就取已有文件里最大的），
  先写同目录临时文件，再 os.link 成正式名（正式名已在就失败，排他），再换上 last-sequence；按序号判「有没有新事件」。
  旧事件自清：每写一条之后只留最新的 KEEP_EVENTS 条，比 KEEP_SECONDS 秒更旧的也删（最新那一条不删）。
推：机器清单照 scripts/hosts/host-resources.py 的 load_machines 读（判法 crash-point-replay-shard-configuration-check.sh --emit-hosts，HOSTS / PEER_SSH_HOST
  那一段，不另写一份读配置），推法是它的 run_on_machine（scripts/hosts/peer-host-lib.sh 的 peer_host_run，ssh 的错误输出换掉私有的字）；
  把这一份的 default_directory 与 store_event 的源码送过去跑（python3 -c，base64），在那台自己的事件目录里照同一种写法写一条。
  多机配置里双机开关没开、配置判不过（没有配置也算）、设了 RESOURCE_BROADCAST_PUSH（任何非空值，例 off）：只写本机，一行不打。
  别的机器连不上不是错：第二台未必配了 ssh 回本机；推不出去的那一台打一行原因（只写原因的种类，不抄 ssh 的输出：里面可能有主机名）。
  各台并行推，每台至多 PUSH_SECONDS 秒（RESOURCE_BROADCAST_PUSH_SECONDS 换掉它），到点只停自己起的那一个 bash，它起的 ssh 由 ssh 自己收尾。
退出码：emit、latest、follow 退 0；wait 有新事件退 0、到点没有退 3；用法错退 2。只给自证的 RESOURCE_BROADCAST_SELFTEST_FAULT=crash（有自证的标记才认）下 emit 与 follow 退 1。
常量的依据：
  PUSH_SECONDS 4：host-resources.py 读第二台（ssh 加 python3 起一份、一秒采样）实测两三秒（它文件头「常量的依据」）；推一行不采样，取 4 秒留余量；推的，没量过。
    发的一方因此至多晚退这么久再加起 python 与读配置的零点几秒。
  KEEP_EVENTS 500、KEEP_SECONDS 86400：等的一方只比「有没有比自己记下的序号新的」，几秒内就读到；500 条够几十条一起跑的变异门禁一两个小时发的（推的），
    一条不到 300 字节，放在 tmpfs 上。
  WAIT_POLL_SECONDS 0.2：等的时候隔这么久读一次 last-sequence（一个小文件），叫醒至多晚这么久。
  FOLLOW_TICK_SECONDS 1：follow 隔这么久核一次起它的进程还在不在。

自证的标记：RESOURCE_BROADCAST_SELFTEST_MARK=<文件>，那份文件在、头一行是 SELFTEST_MARK_CONTENT 那一行。--selftest 自己建一份；调用方的自证要这一份
  认 RESOURCE_BROADCAST_SELFTEST_FAULT 的，照样建一份、export 这个变量。
弄坏开关 RESOURCE_BROADCAST_BREAK=<项>（逗号分隔；只在自证里认：--selftest 自己与带着自证标记的子进程。项名、弄坏哪一处、打开后红在哪几格；
  一格失败打一行以「✗ 自证 <格名>：」起头的细节行，末尾一句「✗ … 自证 N 格不对（查了 M 格）」是汇总句、不带格名、不当一格；
  末尾那一轮照 SWITCH_EXPECTED_CELLS 逐个核；每条下一行起写点名的格，与打开它之后实际红的格逐格相同）：
  RESOURCE_BROADCAST_BREAK=no-local-write  不写本机事件目录
      local-event、concurrent-emits、prune、wait-wakes、follow-prints、push-reaches-hosts、unreachable-not-error、switch-off-local-only、label-no-path
      （「✗ 自证 local-event：」「✗ 自证 concurrent-emits：」「✗ 自证 prune：」「✗ 自证 wait-wakes：」「✗ 自证 follow-prints：」
      「✗ 自证 push-reaches-hosts：」「✗ 自证 unreachable-not-error：」「✗ 自证 switch-off-local-only：」「✗ 自证 label-no-path：」）
  RESOURCE_BROADCAST_BREAK=sequence-unlocked  序号不在锁里取、正式名照覆盖（几条同时发时撞号）
      concurrent-emits（「✗ 自证 concurrent-emits：」）
  RESOURCE_BROADCAST_BREAK=no-prune  旧事件不自清
      prune（「✗ 自证 prune：」）
  RESOURCE_BROADCAST_BREAK=host-name-in-event  推到别的机器的那一行 host 写 ssh 别名
      push-reaches-hosts（「✗ 自证 push-reaches-hosts：」）
  RESOURCE_BROADCAST_BREAK=push-unbounded  推不设时限
      push-time-limit（「✗ 自证 push-time-limit：」）
  RESOURCE_BROADCAST_BREAK=unreachable-exit  推不出去时 emit 退 1
      unreachable-not-error、push-time-limit、problems-to-log（「✗ 自证 unreachable-not-error：」「✗ 自证 push-time-limit：」「✗ 自证 problems-to-log：」）
  RESOURCE_BROADCAST_BREAK=skip-ignored  --skip-peer 也推给第二台
      skip-peer（「✗ 自证 skip-peer：」）
  RESOURCE_BROADCAST_BREAK=wait-ignores-events  wait 只等到点、不看新事件
      wait-wakes（「✗ 自证 wait-wakes：」）
  RESOURCE_BROADCAST_BREAK=push-despite-switch-off  双机开关没开也推
      local-event、switch-off-local-only（「✗ 自证 local-event：」「✗ 自证 switch-off-local-only：」）
  RESOURCE_BROADCAST_BREAK=problems-on-stderr  带 --problems-to-log 也打到 stderr
      problems-to-log（「✗ 自证 problems-to-log：」）
  RESOURCE_BROADCAST_BREAK=fault-without-mark  没有自证的标记也认 RESOURCE_BROADCAST_SELFTEST_FAULT
      fault-needs-mark（「✗ 自证 fault-needs-mark：」）
  RESOURCE_BROADCAST_BREAK=label-path-kept  标签里的斜杠与空白原样写进事件
      label-no-path（「✗ 自证 label-no-path：」）
  RESOURCE_BROADCAST_BREAK=follow-since-ignored  follow 不认 --since，从它起来那一刻最新的之后起
      follow-prints（「✗ 自证 follow-prints：」）
另核调用方的开关：内存包装、清场包装、跨机脚本、脱离长跑与派发器各自的 *_BREAK 里广播那几项，红在各自自证里（写在各自文件头）。
"""
# gate-similar: core_usage.py 它的告警记录（工作根下的 JSONL，开机不清）只给看门狗读、叫醒主 agent，按新鲜窗口留最晚一条，不排序号、不推给别的机器；这里是本机运行期的事件序列（tmpfs、按序号判新旧）加推给别的机器，等的一方是排队的脚本，两样读者不同、寿命不同
# gate-similar: run-with-memory-cap.sh 它状态目录里的 host-lookup.json 只记「谁上一次查过别的机器」给几条一起排队的去重，不是事件；这里发的事件叫醒它排队那一步，去重照旧记在那一份里（加记查到了第几号事件）
# gate-similar: host-resources.py 读机器清单与 ssh 推都调它的 load_machines、run_on_machine（peer-host-lib.sh 的 peer_host_run），不另写一份；它只读各台的资源，不写、不等事件
# gate-similar: host-resources-reminder.sh 它是起重活之前提醒各台资源的钩子，不发不收事件；这里不是钩子，是脚本跑完时发、排队时收
import base64
import contextlib
import fcntl
import importlib.util
import inspect
import os
import math
import select
import shlex
import subprocess
import sys
import tempfile
import threading
import time
from datetime import datetime
from zoneinfo import ZoneInfo

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
HOST_RESOURCES = os.path.join(SCRIPT_DIRECTORY, "host-resources.py")
KINDS = ("memory-released", "gpu-released", "peer-run-finished", "detached-unit-finished")
PUSH_SECONDS = 4.0
KEEP_EVENTS = 500
KEEP_SECONDS = 86400
WAIT_POLL_SECONDS = 0.2
FOLLOW_TICK_SECONDS = 1.0
PROBLEM_LOG_NAME = "push-problems.log"
PROBLEM_LOG_LINES = 200
STORED_MARK = "RESOURCE_BROADCAST_STORED "
EXIT_USAGE = 2
EXIT_TIMED_OUT = 3
BREAK_VARIABLE = "RESOURCE_BROADCAST_BREAK"
FAULT_VARIABLE = "RESOURCE_BROADCAST_SELFTEST_FAULT"
SELFTEST_MARK_VARIABLE = "RESOURCE_BROADCAST_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "resource_release_broadcast.py 自证的标记"
SELFTEST_ACTIVE = False
HOST_PATTERN_NAMES = ("本机", "第二台", "第三台", "第四台", "第五台", "第六台", "第七台", "第八台", "第九台", "第十台")


def selftest_mark_valid():
    path = os.environ.get(SELFTEST_MARK_VARIABLE, "")
    if not path or not os.path.isfile(path):
        return False
    try:
        with open(path, encoding="utf-8") as handle:
            return handle.readline().rstrip("\n") == SELFTEST_MARK_CONTENT
    except OSError:
        return False


def broken(item):
    """弄坏开关：只在自证里认（文件头「弄坏开关」）。"""
    if not (SELFTEST_ACTIVE or selftest_mark_valid()):
        return False
    return item in os.environ.get(BREAK_VARIABLE, "").split(",")


def fault_requested():
    """只给自证：RESOURCE_BROADCAST_SELFTEST_FAULT=crash 且带着自证的标记（弄坏开关 fault-without-mark 下不看标记）。"""
    if os.environ.get(FAULT_VARIABLE, "") != "crash":
        return False
    return selftest_mark_valid() or SELFTEST_ACTIVE or "fault-without-mark" in os.environ.get(BREAK_VARIABLE, "").split(",")


# ── 本机与别的机器上都跑的那两段：只用标准库、不引用本文件别处的名字（推的时候把源码送过去）──
def default_directory(environment):
    """本机事件目录（文件头「本机事件目录」）。"""
    explicit = environment.get("RESOURCE_BROADCAST_DIRECTORY", "")
    if explicit:
        return explicit
    runtime = environment.get("XDG_RUNTIME_DIR", "")
    if runtime:
        return os.path.join(runtime, "singlefs-resource-events")
    return os.path.join("/tmp", f"singlefs-resource-events-{os.getuid()}")


def store_event(directory, body, keep_events, keep_seconds, unlocked=False, prune=True):
    """写一条：锁里取序号、临时文件 os.link 成正式名（排他）、换上 last-sequence、自清旧的；交序号。unlocked 只给弄坏开关 sequence-unlocked。"""
    import fcntl as lock_module
    import os as system
    import time as clock
    system.makedirs(directory, mode=0o700, exist_ok=True)
    sequence_path = system.path.join(directory, "last-sequence")

    def current_last():
        try:
            with open(sequence_path, encoding="utf-8") as handle:
                return int(handle.read().strip() or 0)
        except (OSError, ValueError):
            numbers = [int(name[:-6]) for name in system.listdir(directory) if name.endswith(".event") and name[:-6].isdigit()]
            return max(numbers, default=0)

    with open(system.path.join(directory, "lock"), "a", encoding="utf-8") as lock:
        if not unlocked:
            lock_module.flock(lock, lock_module.LOCK_EX)
        sequence = current_last() + 1
        if unlocked:
            clock.sleep(0.3)   # 弄坏开关：把「读序号、写正式名」之间的竞态撑开
        while True:
            final = system.path.join(directory, f"{sequence:012d}.event")
            temporary = system.path.join(directory, f".{sequence:012d}.{system.getpid()}.partial")
            with open(temporary, "w", encoding="utf-8") as handle:
                handle.write(f"seq={sequence} {body}\n")
            try:
                if unlocked:
                    system.replace(temporary, final)
                else:
                    system.link(temporary, final)
                    system.unlink(temporary)
                break
            except FileExistsError:   # last-sequence 落后了（被删过）：往后挪一个号再试
                system.unlink(temporary)
                sequence += 1
        sequence_temporary = f"{sequence_path}.{system.getpid()}.partial"
        with open(sequence_temporary, "w", encoding="utf-8") as handle:
            handle.write(f"{sequence}\n")
        system.replace(sequence_temporary, sequence_path)
        if prune:
            now = clock.time()
            names = sorted(name for name in system.listdir(directory) if name.endswith(".event") and name[:-6].isdigit())
            for index, name in enumerate(names[:-1]):
                path = system.path.join(directory, name)
                try:
                    if index < len(names) - keep_events or now - system.path.getmtime(path) > keep_seconds:
                        system.unlink(path)
                except OSError:
                    pass
            for name in system.listdir(directory):
                path = system.path.join(directory, name)
                if name.endswith(".partial"):
                    try:
                        if now - system.path.getmtime(path) > 60:
                            system.unlink(path)
                    except OSError:
                        pass
    return sequence


REMOTE_DRIVER = """
import os, sys
directory = default_directory(os.environ)
sequence = store_event(directory, sys.argv[1], {keep_events}, {keep_seconds})
print("{mark}" + str(sequence))
"""


def remote_program():
    return "\n".join([inspect.getsource(default_directory), inspect.getsource(store_event),
                      REMOTE_DRIVER.format(keep_events=KEEP_EVENTS, keep_seconds=KEEP_SECONDS, mark=STORED_MARK)])


# ── 读 ──
def parse_event(line):
    """一行事件 → dict（seq 是整数）；认不出交 None。"""
    fields = {}
    for word in line.strip().split(" "):
        name, separator, value = word.partition("=")
        if separator:
            fields[name] = value
    try:
        fields["seq"] = int(fields["seq"])
    except (KeyError, ValueError):
        return None
    return fields


def latest_sequence(directory):
    """本机事件目录此刻最新的序号；没有事件交 0。"""
    try:
        with open(os.path.join(directory, "last-sequence"), encoding="utf-8") as handle:
            return int(handle.read().strip() or 0)
    except (OSError, ValueError):
        pass
    try:
        numbers = [int(name[:-6]) for name in os.listdir(directory) if name.endswith(".event") and name[:-6].isdigit()]
    except OSError:
        return 0
    return max(numbers, default=0)


def events_after(directory, after):
    """序号比 after 新的事件，按序号排；读不出的那一条跳过。"""
    try:
        names = sorted(name for name in os.listdir(directory) if name.endswith(".event") and name[:-6].isdigit() and int(name[:-6]) > after)
    except OSError:
        return []
    events = []
    for name in names:
        with contextlib.suppress(OSError, UnicodeDecodeError):
            with open(os.path.join(directory, name), encoding="utf-8") as handle:
                event = parse_event(handle.readline())
            if event is not None:
                event["line"] = f"seq={event['seq']} " + " ".join(f"{key}={value}" for key, value in event.items() if key not in ("seq", "line"))
                events.append(event)
    return events


def wait_for_events(after, timeout_seconds, directory=None):
    """等到有比 after 新的事件或到点：交新事件的列表（到点没有交空列表）。弄坏开关 wait-ignores-events 下只等到点。"""
    directory = directory or default_directory(os.environ)
    deadline = time.monotonic() + max(0.0, timeout_seconds)
    while True:
        if not broken("wait-ignores-events") and latest_sequence(directory) > after:
            events = events_after(directory, after)
            if events:
                return events
        remaining = deadline - time.monotonic()
        if remaining <= 0:
            return events_after(directory, after) if broken("wait-ignores-events") else []
        time.sleep(min(WAIT_POLL_SECONDS, remaining) if not broken("wait-ignores-events") else remaining)


# ── 发 ──
def clean_value(text):
    """事件里的值：空白、等号与斜杠换成 _（弄坏开关 label-path-kept 下原样）。"""
    if broken("label-path-kept"):
        return text
    return "".join("_" if character.isspace() or character in "=/" else character for character in text)[:120] or "_"


def host_text_problem(text):
    if text in HOST_PATTERN_NAMES:
        return ""
    if text.startswith("第 ") and text.endswith(" 台") and text[2:-2].isdigit():
        return ""
    return f"--host 只许写「本机」「第二台」「第三台」……「第 <号> 台」，收到「{text}」"


def event_body(kind, label, host, source, exit_code=None, memory_bytes=None, gpu_mebibytes=None, holders_left=None):
    fields = [("kind", kind), ("host", host), ("source", source), ("label", clean_value(label))]
    for name, value in (("exit", exit_code), ("memory_bytes", memory_bytes), ("gpu_mebibytes", gpu_mebibytes), ("holders_left", holders_left)):
        if value is not None and value != "":
            fields.append((name, clean_value(str(value))))
    fields += [("time", f"{time.time():.3f}"), ("date", datetime.now(ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d"))]
    return " ".join(f"{name}={value}" for name, value in fields)


def load_host_resources():
    spec = importlib.util.spec_from_file_location("host_resources_for_broadcast", HOST_RESOURCES)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def push_seconds():
    text = os.environ.get("RESOURCE_BROADCAST_PUSH_SECONDS", "")
    try:
        value = float(text) if text else PUSH_SECONDS
    except ValueError:
        return PUSH_SECONDS
    return value if value > 0 else PUSH_SECONDS


def push(body, skip_peer):
    """推给多机配置里别的机器（文件头「推」）：交推不出去的那几行（空列表是都推到了，或不用推）。"""
    if os.environ.get("RESOURCE_BROADCAST_PUSH", ""):
        return []
    try:
        resources = load_host_resources()
        options = resources.options_from_environment()
        limit = push_seconds()
        options.host_timeout_seconds = None if broken("push-unbounded") else limit
        if broken("push-despite-switch-off"):
            environment = options.settings.environment
            environment["MULTI_HOST_CONFIGURATION_BREAK"] = ",".join(filter(None, [environment.get("MULTI_HOST_CONFIGURATION_BREAK"), "switch-off-is-on"]))
        machines, _values, why = resources.load_machines(options)
        if why:
            return []   # 双机开关没开、配置判不过：只写本机，一行不打（文件头「推」）
        targets = [machine for machine in machines[1:] if not (skip_peer and machine.is_peer and not broken("skip-ignored"))]
        program = base64.b64encode(remote_program().encode("utf-8")).decode("ascii")
        results = {}

        def one(machine):
            remote_body = body.replace(" source=local ", " source=pushed ", 1)
            if broken("host-name-in-event"):
                remote_body = remote_body.replace("host=本机", f"host={machine.ssh_host}", 1)
            command = f'python3 -B -c "$(printf %s {program} | base64 -d)" {shlex.quote(remote_body)}'
            try:
                status, stdout, _stderr = resources.run_on_machine(options, machine, command)
            except Exception as error:   # 起不了 bash 这一类：照推不出去报
                results[machine.number] = f"推的那一步出了 {type(error).__name__}"
                return
            if status == 0 and STORED_MARK in stdout:
                results[machine.number] = ""
            elif status == 124:
                results[machine.number] = f"{limit:g} 秒没回话（推的那一步到点停了自己起的 bash；它起的 ssh 由 ssh 自己收尾）"
            elif status == 255:
                results[machine.number] = "ssh 退 255（连不上，或那台没配免密登录）"
            else:
                results[machine.number] = f"那台上写事件退 {status}"

        threads = [threading.Thread(target=one, args=(machine,), daemon=True) for machine in targets]
        for thread in threads:
            thread.start()
        for thread in threads:
            thread.join(None if broken("push-unbounded") else limit + 2.0)
        problems = []
        for machine in targets:
            reason = results.get(machine.number)
            if reason is None:
                reason = f"{limit:g} 秒没回话（推的那一步没收尾，不再等它）"
            if reason:
                problems.append(f"  ! 没广播到{resources.host_name(machine.number)}：{reason}；只写了本机事件目录（资源释放广播）")
        return problems
    except Exception as error:   # 读配置、导入出了异常：只说一行，发的一方结局照旧
        return [f"  ! 没广播到别的机器：推的那一步出了 {type(error).__name__}（{str(error)[:160]}）；只写了本机事件目录（资源释放广播）"]


def tell_problems(problems, directory, to_log):
    if not problems:
        return
    if to_log and not broken("problems-on-stderr"):
        try:
            os.makedirs(directory, mode=0o700, exist_ok=True)
            path = os.path.join(directory, PROBLEM_LOG_NAME)
            try:
                with open(path, encoding="utf-8") as handle:
                    kept = handle.read().splitlines()
            except (OSError, UnicodeDecodeError):
                kept = []
            stamp = datetime.now(ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d")
            kept = (kept + [f"{stamp} {line.strip()}" for line in problems])[-PROBLEM_LOG_LINES:]
            temporary = f"{path}.{os.getpid()}.partial"
            with open(temporary, "w", encoding="utf-8") as handle:
                handle.write("\n".join(kept) + "\n")
            os.replace(temporary, path)
        except OSError:
            pass   # 日志写不进：发的一方结局照旧，不往它的 stderr 打
        return
    for line in problems:
        with contextlib.suppress(OSError, ValueError):
            print(line, file=sys.stderr, flush=True)


def emit(kind, label, exit_code=None, memory_bytes=None, gpu_mebibytes=None, holders_left=None, host="本机", skip_peer=False, problems_to_log=False):
    """写本机一条、推给别的机器；交本机这一条的序号（没写进交 None）。不抛异常。"""
    directory = default_directory(os.environ)
    body = event_body(kind, label, host, "local", exit_code, memory_bytes, gpu_mebibytes, holders_left)
    sequence = None
    problems = []
    if not broken("no-local-write"):
        try:
            sequence = store_event(directory, body, KEEP_EVENTS, KEEP_SECONDS, unlocked=broken("sequence-unlocked"), prune=not broken("no-prune"))
        except OSError as error:
            problems.append(f"  ! 资源释放广播没写进本机事件目录（{type(error).__name__}: {error.strerror or '没有原因'}）；照旧往别的机器推")
    problems += push(body, skip_peer)
    tell_problems(problems, directory, problems_to_log)
    return sequence, problems


# ── 命令行 ──
def refuse(reason, howto):
    print(f"  ✗ resource_release_broadcast.py：{reason}", file=sys.stderr)
    print(f"  → 怎么办：{howto}", file=sys.stderr)
    return EXIT_USAGE


def option_values(arguments, valued, flags):
    values, index = {}, 0
    while index < len(arguments):
        word = arguments[index]
        if word in flags:
            values[word] = True
            index += 1
        elif word in valued and index + 1 < len(arguments):
            values[word] = arguments[index + 1]
            index += 2
        else:
            raise ValueError(f"认不出的参数「{word}」")
    return values


def whole_number(text, name):
    if text is None:
        return None
    if not text.lstrip("-").isdigit():
        raise ValueError(f"{name} 要写整数，收到「{text}」")
    return int(text)


def command_emit(arguments):
    if fault_requested():
        print("resource_release_broadcast.py：自证造的崩溃（RESOURCE_BROADCAST_SELFTEST_FAULT=crash）", file=sys.stderr)
        return 1
    try:
        values = option_values(arguments, ("--kind", "--label", "--exit-code", "--memory-bytes", "--gpu-mebibytes", "--holders-left", "--host"),
                               ("--skip-peer", "--problems-to-log"))
        kind, label = values.get("--kind"), values.get("--label")
        if kind not in KINDS:
            raise ValueError(f"--kind 只许写 {'、'.join(KINDS)}，收到「{kind}」")
        if not label:
            raise ValueError("没给 --label")
        host = values.get("--host", "本机")
        problem = host_text_problem(host)
        if problem:
            raise ValueError(problem)
        numbers = {name: whole_number(values.get(name), name) for name in ("--exit-code", "--memory-bytes", "--gpu-mebibytes", "--holders-left")}
    except ValueError as error:
        return refuse(str(error), "照文件头「用法」写 emit 的参数（种类、标签必写；数都写整数；--host 只写本机、第二台这类说法）")
    _sequence, problems = emit(kind, label, numbers["--exit-code"], numbers["--memory-bytes"], numbers["--gpu-mebibytes"], numbers["--holders-left"],
                               host, bool(values.get("--skip-peer")), bool(values.get("--problems-to-log")))
    return 1 if problems and broken("unreachable-exit") else 0


def command_wait(arguments):
    try:
        values = option_values(arguments, ("--after", "--timeout"), ())
        after = whole_number(values.get("--after"), "--after")
        timeout = float(values["--timeout"])
        if after is None or not math.isfinite(timeout) or timeout < 0:
            raise ValueError("--after 与 --timeout 都要写（--timeout 是不小于 0 的有限秒数）")
    except (ValueError, KeyError) as error:
        return refuse(str(error), "写成 wait --after <序号> --timeout <秒>；序号取 latest 打的")
    events = wait_for_events(after, timeout)
    for event in events:
        print(event["line"], flush=True)
    return 0 if events else EXIT_TIMED_OUT


def sequence_before(directory, since):
    """time 不早于 since（epoch 秒）的事件都算新的：交它们前面那个序号（没有这样的事件交此刻最新的序号）。"""
    latest = latest_sequence(directory)
    newer = [event["seq"] for event in events_after(directory, 0) if _event_time(event) >= since]
    return min(newer) - 1 if newer else latest


def _event_time(event):
    try:
        return float(event.get("time", ""))
    except ValueError:
        return 0.0


def command_follow(arguments):
    if fault_requested():
        print("resource_release_broadcast.py：自证造的崩溃（RESOURCE_BROADCAST_SELFTEST_FAULT=crash）", file=sys.stderr)
        return 1
    try:
        values = option_values(arguments, ("--after", "--since"), ())
        after = whole_number(values.get("--after"), "--after")
        since = float(values["--since"]) if "--since" in values else None
        if since is not None and not math.isfinite(since):
            raise ValueError("--since 要是有限的 epoch 秒数")
        if after is not None and since is not None:
            raise ValueError("--after 与 --since 只给一个")
    except ValueError as error:
        return refuse(str(error), "写成 follow [--after <序号> | --since <epoch 秒>]")
    directory = default_directory(os.environ)
    if after is None:
        after = sequence_before(directory, since) if since is not None and not broken("follow-since-ignored") else latest_sequence(directory)
    parent = os.getppid()
    try:
        while os.getppid() == parent:
            for event in wait_for_events(after, FOLLOW_TICK_SECONDS, directory):
                print(event["line"], flush=True)
                after = max(after, event["seq"])
    except BrokenPipeError:
        pass
    return 0


def main(arguments):
    # 弄坏开关只在自证里认；fault-without-mark 例外（它弄坏的正是「没有标记也认」，自证在没有标记的子进程里核它）
    if os.environ.get(BREAK_VARIABLE) and os.environ[BREAK_VARIABLE] != "fault-without-mark" and not (SELFTEST_ACTIVE or selftest_mark_valid()):
        return refuse(f"{BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 是弄坏开关，只在自证里认；这一次什么都没做",
                      f"去掉它再跑；要证红跑 {BREAK_VARIABLE}=<项> python3 scripts/hosts/resource_release_broadcast.py --selftest")
    if not arguments:
        return refuse("没给子命令", "emit、latest、wait、follow 或 --selftest，写法见文件头「用法」")
    command, rest = arguments[0], arguments[1:]
    if command == "emit":
        return command_emit(rest)
    if command == "latest":
        if rest:
            return refuse("latest 不带参数", "写成 latest")
        print(latest_sequence(default_directory(os.environ)))
        return 0
    if command == "wait":
        return command_wait(rest)
    if command == "follow":
        return command_follow(rest)
    return refuse(f"认不出的子命令「{command}」", "emit、latest、wait、follow 或 --selftest，写法见文件头「用法」")


# ── 自证 ──
FAKE_SSH = """#!/usr/bin/env bash
# 假 ssh：去掉 -o 选项，按主机名找 hosts/<名>/；有 unreachable 就照连不上退 255，有 slow 就先睡它写的秒数；命令照 ssh 那样交给 bash -c，
# 那台的 XDG_RUNTIME_DIR 换成 hosts/<名>/runtime，本机的 RESOURCE_BROADCAST_DIRECTORY 不带过去
while [[ "${1:-}" == -o ]]; do shift 2; done
host="$1"; shift
printf '%s\\n' "$host" >> "$RESOURCE_BROADCAST_SELFTEST_SSH_LOG"
here="$RESOURCE_BROADCAST_SELFTEST_ROOT/hosts/$host"
if [[ ! -d "$here" || -e "$here/unreachable" ]]; then echo "ssh: connect to host $host port 22: Connection refused" >&2; exit 255; fi
[[ -e "$here/slow" ]] && sleep "$(cat "$here/slow")"
[[ -d "$here" ]] || exit 255   # 睡着的时候自证已经收尾、临时目录删了：不再往那里写
exec env -u RESOURCE_BROADCAST_DIRECTORY XDG_RUNTIME_DIR="$here/runtime" bash -c "$*"
"""
PEER_ALIAS = "selftest-peer-host-name"
LISTED_ALIAS = "selftest-listed-host-name"


class SelftestWorld:
    """一格一个：本机事件目录、清单里的两台（第二台是 PEER_SSH_HOST）、假 ssh 与多机配置。"""

    def __init__(self, root, name, across_machines=True):
        self.directory = os.path.join(root, name)
        self.local = os.path.join(self.directory, "local-events")
        self.ssh_log = os.path.join(self.directory, "ssh.log")
        for alias in (PEER_ALIAS, LISTED_ALIAS):
            os.makedirs(os.path.join(self.directory, "hosts", alias, "runtime"))
        os.makedirs(os.path.join(self.directory, "bin"))
        ssh = os.path.join(self.directory, "bin", "ssh")
        with open(ssh, "w", encoding="utf-8") as handle:
            handle.write(FAKE_SSH)
        os.chmod(ssh, 0o755)
        self.mark = os.path.join(self.directory, "selftest-mark")
        with open(self.mark, "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_MARK_CONTENT + "\n")
        lines = [f"PEER_SSH_HOST={PEER_ALIAS}", "PEER_REPOSITORY_DIRECTORY=/selftest/peer/home", "PEER_CARGO_BIN_DIRECTORY=/selftest/peer/bin", "PEER_MEMORY_CAP=40G",
                 "QUIESCE_STOP_COMMAND=", "QUIESCE_STOPPED_CHECK_COMMAND=", "QUIESCE_START_COMMAND=", "QUIESCE_STARTED_CHECK_COMMAND=",
                 f"HOSTS={PEER_ALIAS} {LISTED_ALIAS}"]
        if across_machines:
            lines.append("ENABLE_ACROSS_MACHINES=1")
        self.configuration = os.path.join(self.directory, "multi-host.env")
        with open(self.configuration, "w", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")

    def remote(self, alias):
        return os.path.join(self.directory, "hosts", alias, "runtime", "singlefs-resource-events")

    def environment(self, **extra):
        environment = {name: value for name, value in os.environ.items()
                       if not name.startswith(("RESOURCE_BROADCAST_", "SINGLEFS_", "MULTI_HOST_", "PEER_HOST_LIB_", "HOST_RESOURCES_", "HOST_DISPATCH_"))}
        if os.environ.get(BREAK_VARIABLE):
            environment[BREAK_VARIABLE] = os.environ[BREAK_VARIABLE]
        environment.update({"PATH": os.pathsep.join([os.path.join(self.directory, "bin"), os.environ.get("PATH", "")]),
                            "SINGLEFS_MULTI_HOST_CONFIG": self.configuration, "RESOURCE_BROADCAST_DIRECTORY": self.local,
                            "RESOURCE_BROADCAST_SELFTEST_ROOT": self.directory, "RESOURCE_BROADCAST_SELFTEST_SSH_LOG": self.ssh_log,
                            SELFTEST_MARK_VARIABLE: self.mark})
        environment.update({name: str(value) for name, value in extra.items()})
        return {name: value for name, value in environment.items() if value != ""}

    def run(self, arguments, timeout=60, **extra):
        """起这一份（子进程），交 (退出码, 标准输出, 标准错误, 用了几秒)。"""
        started = time.monotonic()
        completed = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), *arguments], capture_output=True, text=True,
                                   env=self.environment(**extra), timeout=timeout, stdin=subprocess.DEVNULL)
        return completed.returncode, completed.stdout, completed.stderr, time.monotonic() - started

    def ssh_calls(self):
        with contextlib.suppress(FileNotFoundError):
            with open(self.ssh_log, encoding="utf-8") as handle:
                return handle.read().splitlines()
        return []


def event_lines(directory):
    return [event["line"] for event in events_after(directory, 0)]


def selftest_cells(root):
    """各格：交 (失败的格名列表, 查了几格)。"""
    failures = []
    checked = 0

    def check(cell, condition, detail):
        nonlocal checked
        checked += 1
        if not condition:
            failures.append(cell)
            print(f"  ✗ 自证 {cell}：{detail}")   # gate-lint:detail

    emit_arguments = ["emit", "--kind", "memory-released", "--label", "selftest-label", "--exit-code", "3", "--memory-bytes", "1048576"]

    # local-event：双机关着，emit 写本机一条：退 0、序号 1、字段齐、不推
    world = SelftestWorld(root, "local-event", across_machines=False)
    code, stdout, stderr, _ = world.run(emit_arguments)
    lines = event_lines(world.local)
    _, latest_out, _, _ = world.run(["latest"])
    check("local-event", code == 0 and len(lines) == 1 and lines[0].startswith("seq=1 kind=memory-released host=本机 source=local label=selftest-label exit=3 memory_bytes=1048576 time=")
          and " date=" in lines[0] and latest_out.strip() == "1" and not stderr.strip() and not world.ssh_calls(),
          f"emit 应当退 0、本机事件目录里一条（seq=1、字段齐）、latest 打 1、不推；实际退 {code}、事件 {lines}、latest「{latest_out.strip()}」、stderr「{stderr.strip()}」、ssh {world.ssh_calls()}")

    # concurrent-emits：16 条同时发：16 个不同的序号、16 份文件
    world = SelftestWorld(root, "concurrent-emits", across_machines=False)
    processes = [subprocess.Popen([sys.executable, "-B", os.path.realpath(__file__), *emit_arguments], env=world.environment(), stdout=subprocess.DEVNULL,
                                  stderr=subprocess.DEVNULL, stdin=subprocess.DEVNULL) for _ in range(16)]
    codes = [process.wait(timeout=60) for process in processes]
    sequences = sorted(int(line.split(" ", 1)[0][4:]) for line in event_lines(world.local))
    check("concurrent-emits", codes == [0] * 16 and sequences == list(range(1, 17)),
          f"16 条同时发应当各退 0、序号 1–16 各一份；实际退出码 {codes}、序号 {sequences}")

    # prune：目录里先有 KEEP_EVENTS + 20 条（其中最旧的 5 条改成两天前），再发一条：只剩最新的 KEEP_EVENTS 条，序号接着往下
    world = SelftestWorld(root, "prune", across_machines=False)
    for _ in range(KEEP_EVENTS + 20):
        store_event(world.local, "kind=memory-released host=本机 source=local label=seed", KEEP_EVENTS + 100, KEEP_SECONDS, prune=False)
    old = time.time() - 2 * 86400
    for name in sorted(os.listdir(world.local))[:5]:
        if name.endswith(".event"):
            os.utime(os.path.join(world.local, name), (old, old))
    code, _, stderr, _ = world.run(emit_arguments)
    names = sorted(name for name in os.listdir(world.local) if name.endswith(".event"))
    check("prune", code == 0 and len(names) == KEEP_EVENTS and names[-1] == f"{KEEP_EVENTS + 21:012d}.event",
          f"已有 {KEEP_EVENTS + 20} 条再发一条应当只剩最新的 {KEEP_EVENTS} 条、最新的是第 {KEEP_EVENTS + 21} 号；实际退 {code}、剩 {len(names)} 条、最新 {names[-1] if names else '无'}")

    # wait-wakes：wait 等着，0.5 秒后发一条：3 秒内退 0、打出那一条；没有新事件时到点退 3
    world = SelftestWorld(root, "wait-wakes", across_machines=False)
    waiter = subprocess.Popen([sys.executable, "-B", os.path.realpath(__file__), "wait", "--after", "0", "--timeout", "20"], env=world.environment(),
                              stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, stdin=subprocess.DEVNULL)
    time.sleep(0.5)
    started = time.monotonic()
    world.run(["emit", "--kind", "peer-run-finished", "--label", "selftest-wake", "--host", "第二台", "--exit-code", "0"])
    try:
        waited_out, _ = waiter.communicate(timeout=25)
    except subprocess.TimeoutExpired:
        waiter.kill()   # 只停自己起的这一个
        waited_out, _ = waiter.communicate()
    woke_after = time.monotonic() - started
    timeout_code, _, _, timeout_elapsed = world.run(["wait", "--after", "1", "--timeout", "0.5"])
    check("wait-wakes", waiter.returncode == 0 and "kind=peer-run-finished host=第二台" in waited_out and woke_after < 3 and timeout_code == EXIT_TIMED_OUT and timeout_elapsed < 5,
          f"新事件来了 wait 应当 3 秒内退 0 打出那一条、没有新事件到点退 {EXIT_TIMED_OUT}；实际退 {waiter.returncode}、{woke_after:.1f} 秒、打了「{waited_out.strip()}」，"
          f"到点那一次退 {timeout_code}、{timeout_elapsed:.1f} 秒")

    # follow-prints：--since 给在起它之前发的那一条之前；follow 起着再发两条：3 秒内三条各打一行（起它之前那一条也算）
    world = SelftestWorld(root, "follow-prints", across_machines=False)
    since = time.time()
    world.run(["emit", "--kind", "peer-run-finished", "--label", "selftest-before-follow", "--host", "第二台"])
    follower = subprocess.Popen([sys.executable, "-B", os.path.realpath(__file__), "follow", "--since", f"{since:.3f}"], env=world.environment(),
                                stdout=subprocess.PIPE, stderr=subprocess.DEVNULL, stdin=subprocess.DEVNULL)
    time.sleep(0.5)
    world.run(emit_arguments)
    world.run(["emit", "--kind", "gpu-released", "--label", "selftest-cards", "--holders-left", "1"])
    received, deadline = b"", time.monotonic() + 3.0
    while received.count(b"\n") < 3 and time.monotonic() < deadline:
        ready, _, _ = select.select([follower.stdout.fileno()], [], [], max(0.0, deadline - time.monotonic()))
        if ready:
            chunk = os.read(follower.stdout.fileno(), 65536)
            if not chunk:
                break
            received += chunk
    seen = [line.strip() for line in received.decode("utf-8", "replace").splitlines()]
    follower.kill()   # 只停自己起的这一个
    follower.wait()
    follower.stdout.close()
    check("follow-prints", len(seen) == 3 and "label=selftest-before-follow" in seen[0] and "kind=memory-released" in seen[1] and "kind=gpu-released" in seen[2]
          and "holders_left=1" in seen[2],
          f"follow --since 应当连起它之前、--since 之后发的那一条一起打，起着时再发两条 3 秒内各打一行（共三行）；实际打了 {seen}")

    # push-reaches-hosts：清单两台都推到：两台各一条 source=pushed，哪一处都不写别名、目录
    world = SelftestWorld(root, "push-reaches-hosts")
    code, stdout, stderr, elapsed = world.run(emit_arguments)
    peer_lines, listed_lines, local_lines = event_lines(world.remote(PEER_ALIAS)), event_lines(world.remote(LISTED_ALIAS)), event_lines(world.local)
    everything = "\n".join(peer_lines + listed_lines + local_lines + [stdout, stderr])
    check("push-reaches-hosts", code == 0 and len(peer_lines) == 1 and len(listed_lines) == 1 and len(local_lines) == 1
          and all(" source=pushed " in line and " host=本机 " in line for line in peer_lines + listed_lines) and " source=local " in local_lines[0]
          and PEER_ALIAS not in everything and LISTED_ALIAS not in everything and "/selftest/peer" not in everything and not stderr.strip(),
          f"两台都推到时应当各一条 source=pushed、哪一处都不写别名与目录、不打一行；实际退 {code}、第二台 {peer_lines}、第三台 {listed_lines}、本机 {local_lines}、stderr「{stderr.strip()}」")

    # skip-peer：--skip-peer 只推第三台
    world = SelftestWorld(root, "skip-peer")
    code, _, stderr, _ = world.run([*emit_arguments, "--skip-peer"])
    check("skip-peer", code == 0 and not event_lines(world.remote(PEER_ALIAS)) and len(event_lines(world.remote(LISTED_ALIAS))) == 1 and PEER_ALIAS not in world.ssh_calls(),
          f"--skip-peer 应当不推第二台、照推第三台；实际退 {code}、第二台 {event_lines(world.remote(PEER_ALIAS))}、第三台 {event_lines(world.remote(LISTED_ALIAS))}、ssh {world.ssh_calls()}")

    # unreachable-not-error：第三台连不上：退 0、打一行没广播到第三台（不抄 ssh 的输出），本机与第二台照写
    world = SelftestWorld(root, "unreachable-not-error")
    open(os.path.join(world.directory, "hosts", LISTED_ALIAS, "unreachable"), "w").close()
    code, _, stderr, _ = world.run(emit_arguments)
    check("unreachable-not-error", code == 0 and "没广播到第三台：ssh 退 255" in stderr and LISTED_ALIAS not in stderr and "Connection refused" not in stderr
          and len(event_lines(world.local)) == 1 and len(event_lines(world.remote(PEER_ALIAS))) == 1,
          f"第三台连不上应当退 0、打一行「没广播到第三台：ssh 退 255…」（不抄 ssh 的输出）、本机与第二台照写；实际退 {code}、stderr「{stderr.strip()}」")

    # push-time-limit：第二台 8 秒才回话、时限 2 秒：6 秒内退、打一行没广播到第二台：2 秒没回话（假 ssh 睡完见临时目录不在就退，不发信号）
    world = SelftestWorld(root, "push-time-limit")
    with open(os.path.join(world.directory, "hosts", PEER_ALIAS, "slow"), "w", encoding="utf-8") as handle:
        handle.write("8\n")
    code, _, stderr, elapsed = world.run(emit_arguments, timeout=90, RESOURCE_BROADCAST_PUSH_SECONDS=2)
    check("push-time-limit", code == 0 and elapsed < 2 + 4 and "没广播到第二台：2 秒没回话" in stderr and len(event_lines(world.remote(LISTED_ALIAS))) == 1,
          f"第二台 8 秒才回话、时限 2 秒时 emit 应当 6 秒内退 0、打「没广播到第二台：2 秒没回话」、第三台照推；实际退 {code}、{elapsed:.1f} 秒、stderr「{stderr.strip()}」")

    # switch-off-local-only：双机开关没开：只写本机、一次 ssh 都不起、一行不打
    world = SelftestWorld(root, "switch-off-local-only", across_machines=False)
    code, _, stderr, _ = world.run(emit_arguments)
    check("switch-off-local-only", code == 0 and len(event_lines(world.local)) == 1 and not world.ssh_calls() and not stderr.strip(),
          f"双机开关没开应当只写本机、不起 ssh、不打一行；实际退 {code}、ssh {world.ssh_calls()}、stderr「{stderr.strip()}」")

    # problems-to-log：第三台连不上、带 --problems-to-log：stderr 一行不打，本机事件目录的 push-problems.log 里有那一行
    world = SelftestWorld(root, "problems-to-log")
    open(os.path.join(world.directory, "hosts", LISTED_ALIAS, "unreachable"), "w").close()
    code, _, stderr, _ = world.run([*emit_arguments, "--problems-to-log"])
    log_text = ""
    with contextlib.suppress(OSError):
        with open(os.path.join(world.local, PROBLEM_LOG_NAME), encoding="utf-8") as handle:
            log_text = handle.read()
    check("problems-to-log", code == 0 and not stderr.strip() and "没广播到第三台：" in log_text,
          f"带 --problems-to-log 时应当 stderr 一行不打、push-problems.log 里有「没广播到第三台」；实际退 {code}、stderr「{stderr.strip()}」、日志「{log_text.strip()}」")

    # fault-needs-mark：RESOURCE_BROADCAST_SELFTEST_FAULT=crash 有标记退 1、没标记照常写一条退 0
    world = SelftestWorld(root, "fault-needs-mark", across_machines=False)
    marked_code, _, marked_err, _ = world.run(emit_arguments, RESOURCE_BROADCAST_SELFTEST_FAULT="crash")
    unmarked_environment = world.environment(RESOURCE_BROADCAST_SELFTEST_FAULT="crash")
    unmarked_environment.pop(SELFTEST_MARK_VARIABLE, None)
    unmarked_environment.pop(BREAK_VARIABLE, None)
    if broken("fault-without-mark"):   # 只把这一个开关带给没有标记的子进程（main 只放行它）
        unmarked_environment[BREAK_VARIABLE] = "fault-without-mark"
    completed = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), *emit_arguments], capture_output=True, text=True, env=unmarked_environment,
                               timeout=60, stdin=subprocess.DEVNULL)
    unmarked_ok = completed.returncode == 0 and len(event_lines(world.local)) == 1
    check("fault-needs-mark", marked_code == 1 and "自证造的崩溃" in marked_err and unmarked_ok,
          f"有标记时 RESOURCE_BROADCAST_SELFTEST_FAULT=crash 应当退 1、没标记时照常写一条退 0；实际有标记退 {marked_code}、没标记退 {completed.returncode}、"
          f"事件 {event_lines(world.local)}")

    # label-no-path：标签里的斜杠与空白换成 _；--kind 写错、--host 写别名按用法错退 2
    world = SelftestWorld(root, "label-no-path", across_machines=False)
    code, _, _, _ = world.run(["emit", "--kind", "memory-released", "--label", "/home/someone/run log"])
    bad_kind, _, _, _ = world.run(["emit", "--kind", "memory-free", "--label", "x"])
    bad_host, _, _, _ = world.run(["emit", "--kind", "memory-released", "--label", "x", "--host", PEER_ALIAS])
    lines = event_lines(world.local)
    check("label-no-path", code == 0 and len(lines) == 1 and " label=_home_someone_run_log " in lines[0] and bad_kind == EXIT_USAGE and bad_host == EXIT_USAGE,
          f"标签里的斜杠与空白应当换成 _、--kind 写错与 --host 写别名退 {EXIT_USAGE}；实际退 {code}、事件 {lines}、写错种类退 {bad_kind}、写别名退 {bad_host}")
    return failures, checked


# 弄坏开关 → 打开后红的格（文件头「弄坏开关」那一段，两处要一致；这一轮只核点名的都红）
SWITCH_EXPECTED_CELLS = {
    "no-local-write": ["local-event", "concurrent-emits", "prune", "wait-wakes", "follow-prints", "push-reaches-hosts", "unreachable-not-error",
                       "switch-off-local-only", "label-no-path"],
    "sequence-unlocked": ["concurrent-emits"],
    "no-prune": ["prune"],
    "host-name-in-event": ["push-reaches-hosts"],
    "push-unbounded": ["push-time-limit"],
    "unreachable-exit": ["unreachable-not-error", "push-time-limit", "problems-to-log"],
    "skip-ignored": ["skip-peer"],
    "wait-ignores-events": ["wait-wakes"],
    "push-despite-switch-off": ["local-event", "switch-off-local-only"],
    "problems-on-stderr": ["problems-to-log"],
    "fault-without-mark": ["fault-needs-mark"],
    "label-path-kept": ["label-no-path"],
    "follow-since-ignored": ["follow-prints"],
}


def run_cells_in_scratch():
    root = tempfile.mkdtemp(prefix="resource-release-broadcast-selftest-")
    try:
        return selftest_cells(root)
    finally:
        import shutil
        shutil.rmtree(root, ignore_errors=True)


def selftest():
    global SELFTEST_ACTIVE
    SELFTEST_ACTIVE = True
    failures, checked = run_cells_in_scratch()
    if failures:
        print(f"  ✗ resource_release_broadcast.py 自证 {len(failures)} 格不对（查了 {checked} 格），逐格见上面以「✗ 自证」起头的几行")  # gate-lint:summary
        print("  → 怎么办：照上面逐格改 store_event / wait_for_events / push / emit；RESOURCE_BROADCAST_BREAK 设着的话本来就该红")
        return 1
    if os.environ.get(BREAK_VARIABLE):
        print(f"  ✓ resource_release_broadcast.py 各格（开关轮里起的一份）：查了 {checked} 格都对")
        return 0
    jobs = max(1, min(len(SWITCH_EXPECTED_CELLS), int(os.environ.get("SINGLEFS_THREAD_CAP") or 0) or os.cpu_count() or 1))

    def one(switch):
        completed = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--selftest-cells"], capture_output=True, text=True,
                                   env=dict(os.environ, **{BREAK_VARIABLE: switch}), timeout=600, stdin=subprocess.DEVNULL)
        red = set(line.split("：", 1)[0].removeprefix("  ✗ 自证 ") for line in completed.stdout.splitlines() if line.startswith("  ✗ 自证 "))
        return switch, red

    import concurrent.futures
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        results = list(pool.map(one, SWITCH_EXPECTED_CELLS))
    switch_failures = []
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
    print(f"  ✓ resource_release_broadcast.py 自证通过（查了 {checked} 格：写本机一条字段齐、16 条同时发序号不撞、旧事件自清、wait 来了新事件就醒到点退 {EXIT_TIMED_OUT}、"
          f"follow 每来一条打一行且 --since 之后起它之前的也算、清单里两台都推到且不写别名与目录、--skip-peer 不推第二台、连不上的退 0 打一行、推的那一步到点就停、双机关着只写本机、"
          f"--problems-to-log 不打到 stderr、自证的崩溃要标记、标签里的路径换掉；弄坏开关 {len(results)} 个逐个打开都红在点名的格）")
    return 0


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    if sys.argv[1:] == ["--selftest-cells"]:
        SELFTEST_ACTIVE = True
        cell_failures, _ = run_cells_in_scratch()
        sys.exit(1 if cell_failures else 0)
    sys.exit(main(sys.argv[1:]))
