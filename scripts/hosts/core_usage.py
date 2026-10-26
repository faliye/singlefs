#!/usr/bin/env python3
# admission: always 每一次调都现读 /proc、点名的样本与声明文件，上一次的结论不替这一次作保
# run-condition: none 只读 /proc 与点名的文件、往点名的样本、声明与告警记录里追加，除了 python3 没有环境要求；被 import 时不读文件、不起子进程
"""长跑在用几个核：取样、按窗口判、读程序当场声明的串行段、写告警记录。被 import 的函数库，也能直接执行。

谁用：
  scripts/hosts/multi-host-run.sh --peer   第二台那条命令在跑时，每 MULTI_HOST_RUN_CORE_SECONDS 秒经 ssh 在第二台那棵树里跑一次 sample，
                                           样本存本机，再在本机跑 judge（主体写「第二台」）
  scripts/hosts/detached-run.sh watch      每一轮清单之后，给还在跑的每条登记跑一次 judge --pid（样本与声明放在那条登记的目录里）
  scripts/agents/agent-watch.py            import 它：给这个 Claude 实例底下跑过一个窗口的重活进程取样、判；读告警记录叫醒主 agent

用法：
  core_usage.py sample (--pid <进程号> | --pid-file <文件>) [--declarations <文件>]
      打一行 JSON：这棵进程树此刻的样本；给了 --declarations 就把那份声明文件的原文放进 declarations_text 一并带回（第二台上取样时用）
  core_usage.py judge (--pid <进程号> | --incoming <sample 打的那一行所在的文件>) --series <样本文件> [--declarations <声明文件>]
                      --subject <主体> [--alert-pid <进程号>] [--source <谁在判>] [--label <名>] [--next-step <出路>]
      样本追加进 --series，按窗口判一次，往标准输出打一行「<主体>在用 X 核（登记 N）…」；判成核数过少的打 ✗ 一行与出路、追加一条告警记录、退 3，
      别的退 0。--incoming 那一行带着 declarations_text 的，先把它写进 --declarations 再判
  core_usage.py declare-serial <理由>      往 $SINGLEFS_SERIAL_DECLARATIONS 追加「<秒>\t此刻串行：<理由>」，理由至少 8 个字
  core_usage.py declare-parallel           往 $SINGLEFS_SERIAL_DECLARATIONS 追加「<秒>\t此刻并行」
  core_usage.py busy-load <线程数> <秒>     只给自证用的假负载：起这么多条线程各自忙等这么久（哈希一块 1 MiB 的内存，算的时候放开解释器锁）
  core_usage.py --selftest

取样（tree_sample）：从给的进程号往下的整棵进程树，读 /proc/<pid>/stat。
  在用的 CPU 秒数 = 树里每个活着的进程的 utime + stime + cutime + cstime（cutime、cstime 是它收过尸的子进程，连同它们收过尸的子进程，所以中途退出的子进程照算）。
  每条线程（/proc/<pid>/task/<tid>/stat，键「进程号/线程号/起始时刻」）的 utime + stime 另记一份，用来算「同时在算的线程」。
  同一遍另数树里每条线程此刻的状态（那一行第三列）：状态 D（不可打断的睡眠，在等盘）几条记 disk_waiting_threads，状态 R（在跑或排队等核）几条记 computing_threads。
  命令行里有 multi-host-run.sh、后面（-- 之前）带 --peer 的进程连同它的整棵子树不算进来，只数有几个（peer_drivers）：
  那一支在等第二台，第二台用几个核由 multi-host-run.sh 自己报。
判（judge_series）：样本序列前面补一个「起跑时刻、0 CPU 秒」的样本（树根进程的起始时刻）。
  基准 = 不晚于「这一次 − 窗口」的最晚那个样本；没有（树根跑了不到一个窗口）就是「不满一个窗口」，只打一行、不判。
  在用核数 = 两次的 CPU 秒数之差 ÷ 两次之间的挂钟秒数，但不低于还在树里的线程这一窗口里的增量之和 ÷ 挂钟秒数（同时在算的线程那一项用的同一组增量）。
    为什么要这个下限：合计 CPU 秒数只在进程都留在树里时才不会往回走。窗口里有进程离开了这棵树——中间的进程退出而它的孙进程被孤立、改挂到 init 或 subreaper 底下，
    收尸的不在树里，取样时按进程号逐个读 /proc、读到一半它被收了——它到那时为止的 CPU 就从合计里掉出去，两次相减成了负数（第二台的告警里见过「在用 -60.9 核」「-36.0」「-51.1」；
    2026-10-21 在自证里用一个被孤立的忙等孙进程造出同一种情形：合计从 1.54 秒掉到 0.02 秒，旧算法给 -0.76 核）。
    中途退出、被树里的父进程收了尸的子进程不在这一类：它的时间已经进了父进程的 cutime / cstime（取样一节），合计不掉。
    合计掉下去、取了下限的，判的那一行末尾写明掉了多少秒（掉得不到 1 秒的不写）；下限是实打实算过的，真值不低于它，所以只会少报告用量、不会多报。
  同时在算的线程 = 两次之间每条线程 CPU 秒数的增量之和 ÷ 其中最大的那一条（只有一条线程在算是 1，N 条一样忙是 N）：
    机器被别的活占满时在用核数会掉，而 N 条线程照样都在算，这一项不跟着掉，用它分开「装置串行」与「机器忙」。
  阈值 = 每台默认重活线程数（scripts/hosts/heavy_threads.py 的 default_heavy_threads：SINGLEFS_THREAD_CAP 设了照它，否则 .claude/heavy-thread-default）÷ 4。
    依据（推的，没量过）：登记要求每件重活用这么多条线程，并行对了的阶段在用核数贴着它；串行的装置是 1 左右。四分之一离两头都远：
    编译收尾、写盘这类在 10 分钟里短暂掉到几个核的，平均下来过得去；只起了一两条、三五条线程的都报。
  在用核数与同时在算的线程都低于阈值，判「核数过少」；树里有 multi-host-run.sh --peer 那一支的，判「在等第二台」、不报。
  等盘（用户 2026-10-26 定「等盘标出来、不判过少」）：判成核数过少的那一窗口，把窗口里（基准之后、到这一次为止）带线程状态的各次样本的
    disk_waiting_threads、computing_threads 各自加起来，在等盘的线程次数多于在算的，改判「等盘」：打一行标出取样次数与两个数、不打 ✗、不写告警记录、judge 退 0。
    按采样判、逐窗口判：下一窗口在算的占多数就照报核数过少；窗口里一次带线程状态的样本都没有（旧样本、别的写法造的样本）照旧判。没有参数能关掉这一判、也没有参数能扩大它。
  窗口 600 秒（10 分钟：与 .claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的 5 分钟进度间隔对上，两次进度之内就看得出来）。
    SINGLEFS_CORE_USAGE_WINDOW_SECONDS 只许往短里改（正整数，不超过 600），给的比 600 长照 600 判、打一句说明：不许一个参数把整趟静音。
    阈值没有参数可改。
声明（程序当场写）：环境变量 SINGLEFS_SERIAL_DECLARATIONS 指的文件，一行一条：
    <unix 秒>\t此刻串行：<理由>     从这一刻起是串行段（计时窗口必须单线程、共享状态必须独占这一类），理由至少 8 个字
    <unix 秒>\t此刻并行             串行段到这一刻结束
  下一条声明（串行或并行）结束上一段串行；最后一段没结束的算到这一次取样。写坏的行（开头不是秒数、认不出、理由不到 8 个字）不算声明，判的那一行里点出来。
  谁设这个变量：multi-host-run.sh --peer 设成第二台 runs/<标签>.serial，detached-run.sh start 设成登记目录里的 serial-declarations；
    别的跑法由起它的脚本自己 export，agent-watch.py 到重活进程那棵树的环境里找它。shell 里写声明用 declare-serial / declare-parallel。
  声明不让告警不报：核数过少照报 ✗，只是标出这一窗口里声明的串行段有几秒、理由是什么；声明的串行段累计超过整趟（起跑到此刻）一半的，另注一句「超过一半」。
告警记录：判成核数过少时追加一行 JSON 到 ${SINGLEFS_CORE_USAGE_ALERTS:-<工作根>/core-usage-alerts.jsonl}（工作根由 .claude/hooks/lib_work_root.py 定），
  带会话号（CLAUDE_CODE_SESSION_ID）、谁判的、主体、告警进程号（停它、--ack 进程:<进程号> 用这个号）、在用核数、阈值、声明的串行段与那一行原文。
  agent-watch.py 读最近 ALERT_FRESH_SECONDS（660 秒：两次判之间 300 秒的两倍再加一分钟）里的、告警进程还活着的，叫醒那个会话的主 agent。
管不到：
  只看 CPU，不看 GPU：GPU 在算、CPU 只喂数据的段照样判成核数过少，要程序声明。
  机器忙、而活又是一批批短命的线程或子进程（两次取样之间生了又死）时，「同时在算的线程」数不准，可能误报；判的那一行带着两个数，由主 agent 看。
  树里同时有 multi-host-run.sh --peer 那一支与本机自己的活时，整棵判成「在等第二台」，本机那一半不判。
  脱离进程树跑的（重新挂到 init 或别的 subreaper 底下的孙进程）数不到。
  线程状态只在取样那一刻看：两次取样之间进出状态 D 的数不到；状态 D 不分等的是盘还是别的不可打断的事（NFS、内核锁这类也算进等盘）。
弄坏开关 CORE_USAGE_BREAK=<项>（逗号分隔；只在 --selftest 里认：自证进程自己，与它起的、环境里 CORE_USAGE_SELFTEST_MARK 指着自证临时目录里那份标记的子进程；
  别处设了，直接执行的各个用法拒、退 2）。项名、弄坏哪一处、打开后自证哪几格红（「✗ core_usage.py 自证：<格名>」）：
  descendants-ignored          取样只算树根自己              子进程里忙等的也算进来；multi-host-run.sh --peer 那一支不算、判成在等第二台（往下找不到那一支）
  reaped-ignored               不算 cutime、cstime           中途退出、收过尸的子进程的 CPU 照算
  peer-driver-counted          multi-host-run.sh --peer 那一支照算、不认   multi-host-run.sh --peer 那一支不算、判成在等第二台
  parallel-threads-ignored     只看在用核数，不看同时在算的线程   机器忙：在用核数低而登记数那么多条线程都在算的，不报
                               （机器此刻真忙时，登记数那么多条线程忙等那一格也红）
  declaration-silences         窗口里声明了串行就不报         声明了串行段的照报，标出秒数与理由；声明的串行段累计超过整趟一半的，注明超过一半
  declaration-half-unflagged   超过整趟一半不注明             声明的串行段累计超过整趟一半的，注明超过一半
  short-reason-accepted        理由不到 8 个字也算声明         理由不到 8 个字的不算声明，判的那一行点出来
  window-extendable            SINGLEFS_CORE_USAGE_WINDOW_SECONDS 比 600 长也照它   窗口不许调长
  alert-record-skipped         判成核数过少也不写告警记录       一条线程忙等：judge 退 3、打 ✗ 与出路、写告警记录（带会话号）
  negative-cores-allowed       合计 CPU 秒数掉下去时照旧相减，不取还在树里的线程的增量当下限   进程离开树、合计掉了的：在用核数不为负、不低于还在树里的线程的用量
                               （造的样本序列「窗口里有进程离开了这棵树」与真进程「孙进程被孤立」两格红）
  disk-wait-uncounted          判的时候不数状态 D 的线程（改前的样子）   等盘：窗口里在等盘的线程次数多于在算的，标等盘、不判核数过少
"""
import hashlib
import importlib.util
import json
import os
import re
import shlex
import subprocess
import sys
import tempfile
import threading
import time
from datetime import datetime
from zoneinfo import ZoneInfo

REPOSITORY = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
WORK_ROOT_LIBRARY = os.path.join(REPOSITORY, ".claude", "hooks", "lib_work_root.py")
HEAVY_THREADS_LIBRARY = os.path.join(REPOSITORY, "scripts", "hosts", "heavy_threads.py")
WINDOW_SECONDS = 600
WINDOW_VARIABLE = "SINGLEFS_CORE_USAGE_WINDOW_SECONDS"
SHARE_DIVISOR = 4
DECLARATIONS_VARIABLE = "SINGLEFS_SERIAL_DECLARATIONS"
ALERTS_VARIABLE = "SINGLEFS_CORE_USAGE_ALERTS"
ALERTS_FILE_NAME = "core-usage-alerts.jsonl"
ALERT_FRESH_SECONDS = 660
REASON_MINIMUM_CHARACTERS = 8
SERIAL_MARK = "此刻串行："
PARALLEL_MARK = "此刻并行"
PEER_DRIVER_NAME = "multi-host-run.sh"
DISK_WAIT_STATE = "D"   # /proc/<pid>/task/<tid>/stat 第三列：不可打断的睡眠，在等盘（文件头「取样」「判」的「等盘」）
COMPUTING_STATE = "R"   # 在跑或排着队等核
SESSION_VARIABLE = "CLAUDE_CODE_SESSION_ID"
BREAK_VARIABLE = "CORE_USAGE_BREAK"
SELFTEST_MARK_VARIABLE = "CORE_USAGE_SELFTEST_MARK"
SELFTEST_MARK_CONTENT = "core_usage.py --selftest 的临时目录"
SELFTEST_ACTIVE = False
DEFAULT_NEXT_STEP = ("装置里互相独立的单位（取样点、轮次、不计时的造池与准备）该分到登记的线程数上，却在一条线程上串行：停下这一趟，"
                     "照 .claude/rules/test-script-discipline.md「编译与算法都要快」改装置再跑；真要串行的段（计时窗口、独占的共享状态），"
                     f"由程序当场往 ${DECLARATIONS_VARIABLE} 写「<秒>\\t{SERIAL_MARK}<理由>」（理由至少 {REASON_MINIMUM_CHARACTERS} 个字），告警照报、标出理由，"
                     "主 agent 核过理由成立再接着跑")


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


def load_module(name, path):
    spec = importlib.util.spec_from_file_location(name, path)
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


def registered_threads():
    """每台默认重活线程数；取不了抛 heavy_threads.HeavyThreadDefaultError（str 是原因，.howto 是出路）。"""
    return int(load_module("heavy_threads", HEAVY_THREADS_LIBRARY).default_heavy_threads())


def window_seconds():
    """(窗口秒数, 说明或 None)：SINGLEFS_CORE_USAGE_WINDOW_SECONDS 只许往短里改。"""
    text = os.environ.get(WINDOW_VARIABLE, "")
    if not text:
        return WINDOW_SECONDS, None
    if not re.fullmatch(r"[1-9][0-9]*", text):
        return WINDOW_SECONDS, f"{WINDOW_VARIABLE}={text} 不是正整数，照 {WINDOW_SECONDS} 秒判"
    value = int(text)
    if value > WINDOW_SECONDS and not broken("window-extendable"):
        return WINDOW_SECONDS, f"{WINDOW_VARIABLE}={text} 比 {WINDOW_SECONDS} 长，窗口只许往短里改，照 {WINDOW_SECONDS} 秒判"
    return value, None


def boot_time():
    with open("/proc/stat") as handle:
        for line in handle:
            if line.startswith("btime "):
                return int(line.split()[1])
    return 0


def process_table():
    """{进程号: {parent, state, own_ticks, reaped_ticks, start_ticks}}，读不到的（刚退出）跳过。"""
    table = {}
    for entry in os.listdir("/proc"):
        if not entry.isdigit():
            continue
        try:
            with open(f"/proc/{entry}/stat") as handle:
                stat_text = handle.read()
            fields = stat_text[stat_text.rfind(")") + 2:].split()
            table[int(entry)] = {"parent": int(fields[1]), "state": fields[0], "own_ticks": int(fields[11]) + int(fields[12]),
                                 "reaped_ticks": int(fields[13]) + int(fields[14]), "start_ticks": int(fields[19])}
        except (OSError, IndexError, ValueError):
            continue
    return table


def process_arguments(pid):
    try:
        with open(f"/proc/{pid}/cmdline", "rb") as handle:
            raw = handle.read()
    except OSError:
        return []
    return [word.decode(errors="replace") for word in raw.split(b"\0") if word]


def is_peer_driver(arguments):
    """命令行里有 multi-host-run.sh、它后面（-- 之前）带 --peer。"""
    for index, word in enumerate(arguments[:3]):
        if os.path.basename(word) == PEER_DRIVER_NAME:
            rest = arguments[index + 1:]
            options = rest[:rest.index("--")] if "--" in rest else rest
            return "--peer" in options
    return False


def thread_times(pid, hertz, states=None):
    """{「进程号/线程号/起始时刻」: CPU 秒数}，只记用过 CPU 的线程；给了 states（dict）就把每条线程此刻的状态字母计数进去（没用过 CPU 的也数）。"""
    found = {}
    try:
        tasks = os.listdir(f"/proc/{pid}/task")
    except OSError:
        return found
    for task in tasks:
        try:
            with open(f"/proc/{pid}/task/{task}/stat") as handle:
                stat_text = handle.read()
            fields = stat_text[stat_text.rfind(")") + 2:].split()
            ticks = int(fields[11]) + int(fields[12])
            if ticks > 0:
                found[f"{pid}/{task}/{fields[19]}"] = round(ticks / hertz, 2)
            if states is not None:
                states[fields[0]] = states.get(fields[0], 0) + 1
        except (OSError, IndexError, ValueError):
            continue
    return found


def tree_sample(root_pid, table=None):
    """这棵进程树此刻的样本（判法在文件头「取样」）；树根不在了交 None。"""
    table = process_table() if table is None else table
    root = table.get(root_pid)
    if root is None or root["state"] in ("Z", "X"):
        return None
    hertz = os.sysconf("SC_CLK_TCK")
    children = {}
    for pid, info in table.items():
        children.setdefault(info["parent"], []).append(pid)
    included, peer_drivers, frontier = [], [], [root_pid]
    while frontier:
        pid = frontier.pop()
        if not broken("peer-driver-counted") and is_peer_driver(process_arguments(pid)):
            peer_drivers.append(pid)
            continue
        included.append(pid)
        if broken("descendants-ignored"):
            continue
        frontier.extend(children.get(pid, []))
    ticks = 0
    per_thread = {}
    thread_states = {}
    for pid in included:
        info = table[pid]
        ticks += info["own_ticks"] + (0 if broken("reaped-ignored") else info["reaped_ticks"])
        per_thread.update(thread_times(pid, hertz, thread_states))
    return {"root": root_pid, "time": round(time.time(), 3), "started": round(boot_time() + root["start_ticks"] / hertz, 2),
            "cpu": round(ticks / hertz, 2), "processes": len(included), "threads": len(per_thread), "peer_drivers": len(peer_drivers),
            "thread_cpu": per_thread, "disk_waiting_threads": thread_states.get(DISK_WAIT_STATE, 0),
            "computing_threads": thread_states.get(COMPUTING_STATE, 0)}


def gone_sample(root_pid):
    return {"root": root_pid, "time": round(time.time(), 3), "gone": True}


def read_series(path):
    samples = []
    if not path or not os.path.isfile(path):
        return samples
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            try:
                sample = json.loads(line)
            except ValueError:
                continue
            if isinstance(sample, dict) and "time" in sample:
                samples.append(sample)
    return samples


def append_line(path, text):
    os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
    with open(path, "a", encoding="utf-8") as handle:
        handle.write(text.rstrip("\n") + "\n")


def read_declarations(path):
    """([(秒, serial|parallel, 理由)], [写坏的行的说明])。"""
    entries, problems = [], []
    if not path or not os.path.isfile(path):
        return entries, problems
    with open(path, encoding="utf-8", errors="replace") as handle:
        text = handle.read()
    return parse_declarations(text)


def parse_declarations(text):
    entries, problems = [], []
    for number, line in enumerate(text.split("\n"), 1):
        if not line.strip():
            continue
        stamp, _, body = line.partition("\t")
        try:
            moment = float(stamp)
        except ValueError:
            problems.append(f"第 {number} 行开头不是秒数")
            continue
        body = body.strip()
        if body.startswith(SERIAL_MARK):
            reason = body[len(SERIAL_MARK):].strip()
            if len(reason) < REASON_MINIMUM_CHARACTERS and not broken("short-reason-accepted"):
                problems.append(f"第 {number} 行的理由「{reason}」不到 {REASON_MINIMUM_CHARACTERS} 个字")
                continue
            entries.append((moment, "serial", reason))
        elif body == PARALLEL_MARK:
            entries.append((moment, "parallel", ""))
        else:
            problems.append(f"第 {number} 行认不出（要写「{SERIAL_MARK}<理由>」或「{PARALLEL_MARK}」）")
    return entries, problems


def serial_intervals(entries, end_time):
    intervals, open_start, open_reason = [], None, ""
    for moment, kind, reason in sorted(entries):
        if open_start is not None:
            intervals.append((open_start, moment, open_reason))
            open_start = None
        if kind == "serial":
            open_start, open_reason = moment, reason
    if open_start is not None and open_start < end_time:
        intervals.append((open_start, end_time, open_reason))
    return intervals


def overlap(intervals, start, end):
    total, reasons = 0.0, []
    for low, high, reason in intervals:
        covered = min(high, end) - max(low, start)
        if covered > 0:
            total += covered
            if reason not in reasons:
                reasons.append(reason)
    return total, reasons


def judge_series(samples, entries, problems, registered, window):
    """判一次（判法在文件头「判」）：返回一个 dict，status 是 ok、low、disk-wait、warming、waiting-peer、gone 之一。"""
    current = samples[-1]
    threshold = registered / SHARE_DIVISOR
    verdict = {"status": "gone", "registered": registered, "threshold": threshold, "window": window, "problems": problems,
               "cores": 0.0, "dropped_cpu": 0.0, "parallel": 0.0, "span": 0.0, "declared_in_window": 0.0, "reasons": [], "declared_fraction": 0.0,
               "state_samples": 0, "disk_waiting": 0, "computing": 0}
    if current.get("gone"):
        return verdict
    same_root = [sample for sample in samples[:-1] if not sample.get("gone") and sample.get("root") == current["root"]
                 and abs(sample.get("started", 0) - current["started"]) < 2]
    start_sample = {"time": current["started"], "cpu": 0.0, "thread_cpu": {}}
    eligible = [sample for sample in [start_sample] + same_root if sample["time"] <= current["time"] - window]
    base = max(eligible, key=lambda sample: sample["time"]) if eligible else start_sample
    span = current["time"] - base["time"]
    cores = (current["cpu"] - base.get("cpu", 0.0)) / span if span > 0 else 0.0
    deltas = [value - base.get("thread_cpu", {}).get(key, 0.0) for key, value in current.get("thread_cpu", {}).items()]
    deltas = [delta for delta in deltas if delta > 0]
    parallel = sum(deltas) / max(deltas) if deltas else 0.0
    # 合计 CPU 秒数只在进程都留在树里时才单调：窗口里有进程离开了这棵树（中间的进程退出、孙进程被孤立改挂到 init 或 subreaper 底下；收尸的不在树里；
    # 取样时按进程号逐个读 /proc、读到一半它被收了）它到那时为止的 CPU 就从合计里掉出去，两次相减成了负数。窗口里仍在树里的线程的增量（deltas 里正的那些之和）
    # 是实打实算过的下限，在用核数不低于它：合计掉下去的时候取它，并记下掉了多少秒（verdict_lines 报）
    live_cores = sum(deltas) / span if span > 0 else 0.0
    dropped_cpu = 0.0
    if cores < live_cores and not broken("negative-cores-allowed"):
        dropped_cpu = (live_cores - cores) * span
        cores = live_cores
    intervals = serial_intervals(entries, current["time"])
    declared, reasons = overlap(intervals, base["time"], current["time"])
    run_span = current["time"] - current["started"]
    run_declared, _ = overlap(intervals, current["started"], current["time"])
    verdict.update(cores=cores, dropped_cpu=dropped_cpu, parallel=parallel, span=span, declared_in_window=declared, reasons=reasons,
                   declared_fraction=(run_declared / run_span if run_span > 0 else 0.0))
    if current.get("peer_drivers", 0) > 0:
        verdict["status"] = "waiting-peer"
    elif not eligible:
        verdict["status"] = "warming"
    else:
        low = cores < threshold and (broken("parallel-threads-ignored") or parallel < threshold)
        if low and declared > 0 and broken("declaration-silences"):
            low = False
        verdict["status"] = "low" if low else "ok"
        # 等盘（文件头「判」）：这一窗口里各次取样数到的状态 D 的线程次数多于状态 R 的，在用核数低是在等盘，标等盘、不判核数过少。
        #   按取样判：只数这一窗口里带线程状态的样本（基准之后、到这一次为止），一次都没有就照旧判。弄坏开关 disk-wait-uncounted 下不数
        window_samples = [sample for sample in same_root + [current]
                          if base["time"] < sample["time"] <= current["time"] and "disk_waiting_threads" in sample]
        disk_waiting = sum(int(sample.get("disk_waiting_threads", 0)) for sample in window_samples)
        computing = sum(int(sample.get("computing_threads", 0)) for sample in window_samples)
        verdict.update(state_samples=len(window_samples), disk_waiting=disk_waiting, computing=computing)
        if low and window_samples and disk_waiting > computing and not broken("disk-wait-uncounted"):
            verdict["status"] = "disk-wait"
    return verdict


def verdict_lines(verdict, subject, next_step=DEFAULT_NEXT_STEP):
    """判的那一行（核数过少的另带出路一行）：[行…]。"""
    registered, threshold = verdict["registered"], verdict["threshold"]
    head = f"{subject}在用 {verdict['cores']:.1f} 核（登记 {registered}）"
    status = verdict["status"]
    if status == "gone":
        return [f"  · {subject}的进程已经不在了，不判"]
    if status == "waiting-peer":
        return [f"  · {head}：这一棵在等第二台（multi-host-run.sh --peer 那一支不算进来，第二台用几个核由它自己报），不判"]
    if status == "warming":
        return [f"  · {head}：只看了 {verdict['span']:.0f} 秒，不满一个窗口（{verdict['window']} 秒），还不判"]
    if status == "disk-wait":
        return [f"  · {head}：最近 {verdict['span']:.0f} 秒平均、同时在算的线程约 {verdict['parallel']:.1f} 条，低于阈值 {threshold:g}；"
                f"这一窗口 {verdict['state_samples']} 次取样里在等盘（状态 D）的线程合计 {verdict['disk_waiting']} 次、在算（状态 R）的 {verdict['computing']} 次，"
                f"等盘的占多数：标等盘，不判核数过少{dropped_note(verdict)}"]
    if status == "ok":
        return [f"  · {head}：最近 {verdict['span']:.0f} 秒平均，同时在算的线程约 {verdict['parallel']:.1f} 条，阈值 {threshold:g}，不报{dropped_note(verdict)}"]
    return [f"  ✗ {head}：最近 {verdict['span']:.0f} 秒平均、同时在算的线程约 {verdict['parallel']:.1f} 条，都低于阈值 {threshold:g}（登记的四分之一）"
            f"{declaration_note(verdict)}{dropped_note(verdict)}",
            f"     → 怎么办：{next_step}"]


def dropped_note(verdict):
    """窗口里有进程离开了这棵树、合计 CPU 秒数掉下去时的后半句（judge_series 取了还在树里的线程的增量当下限）；掉得不到 1 秒的不提。"""
    if verdict.get("dropped_cpu", 0.0) < 1.0:
        return ""
    return (f"；这一窗口里有进程离开了这棵进程树（中间的进程退出而孙进程被孤立、或收尸的不在树里），合计 CPU 秒数掉了 {verdict['dropped_cpu']:.0f} 秒，"
            f"在用核数按还在树里的线程这一窗口的增量算（下限，真值不低于它）")


def declaration_note(verdict):
    """核数过少那一行的后半句：这一窗口里声明的串行段（秒数与理由）、累计超过整趟一半、写坏的声明行。"""
    if verdict["declared_in_window"] > 0:
        note = f"；这一窗口里声明了串行 {verdict['declared_in_window']:.0f} 秒：{'；'.join(verdict['reasons'])}"
    else:
        note = "；这一窗口里没有声明串行段"
    if verdict["declared_fraction"] > 0.5 and not broken("declaration-half-unflagged"):
        note += f"；声明的串行段累计占整趟 {verdict['declared_fraction'] * 100:.0f}%，超过一半"
    if verdict["problems"]:
        note += f"；声明文件里写坏的 {len(verdict['problems'])} 行不算声明：{'；'.join(verdict['problems'][:3])}"
    return note


def alerts_path():
    """告警记录的位置；工作根那份库导入失败交 None。"""
    explicit = os.environ.get(ALERTS_VARIABLE, "")
    if explicit:
        return explicit
    try:
        return os.path.join(load_module("lib_work_root", WORK_ROOT_LIBRARY).work_root(), ALERTS_FILE_NAME)
    except Exception:   # 文件不在、语法错、弄坏开关值认不出：交给调用方说「告警记录写不进」
        return None


def alert_record(verdict, subject, source, label, alert_pid, next_step, line):
    return {"time": round(time.time(), 3), "date": datetime.now(ZoneInfo("Asia/Tokyo")).strftime("%Y-%m-%d"), "session_id": os.environ.get(SESSION_VARIABLE, ""),
            "source": source, "subject": subject, "label": label, "alert_pid": alert_pid, "cores": round(verdict["cores"], 2),
            "parallel": round(verdict["parallel"], 2), "registered": verdict["registered"], "threshold": verdict["threshold"],
            "window": verdict["window"], "declared_in_window": round(verdict["declared_in_window"], 1), "reasons": verdict["reasons"],
            "declared_fraction": round(verdict["declared_fraction"], 3), "line": line.strip(), "next_step": next_step}


def read_alert_records(path, now=None, fresh_seconds=ALERT_FRESH_SECONDS):
    """最近 fresh_seconds 里的告警记录：每个（来源、告警进程号、名）只留最晚的一条。"""
    now = time.time() if now is None else now
    latest = {}
    if not path or not os.path.isfile(path):
        return []
    with open(path, encoding="utf-8", errors="replace") as handle:
        for line in handle:
            try:
                record = json.loads(line)
            except ValueError:
                continue
            if not isinstance(record, dict) or not isinstance(record.get("time"), (int, float)):
                continue
            if record["time"] < now - fresh_seconds:
                continue
            key = (record.get("source"), record.get("alert_pid"), record.get("label"))
            if key not in latest or latest[key]["time"] <= record["time"]:
                latest[key] = record
    return sorted(latest.values(), key=lambda record: record["time"])


def refuse(reason, howto, code=2):
    print(f"  ✗ core_usage.py：{reason}", file=sys.stderr)
    print(f"     → 怎么办：{howto}", file=sys.stderr)
    return code


def option_values(arguments, names_with_value):
    values, rest, index = {}, [], 0
    while index < len(arguments):
        word = arguments[index]
        if word in names_with_value:
            if index + 1 >= len(arguments):
                raise ValueError(f"{word} 后面没给值")
            values[word] = arguments[index + 1]
            index += 2
            continue
        rest.append(word)
        index += 1
    return values, rest


def pid_from(values):
    if "--pid" in values:
        text = values["--pid"]
    else:
        try:
            with open(values["--pid-file"], encoding="utf-8") as handle:
                text = handle.read().strip()
        except OSError:
            return None
    return int(text) if re.fullmatch(r"[0-9]+", text or "") else None


def command_sample(arguments):
    values, rest = option_values(arguments, ("--pid", "--pid-file", "--declarations"))
    if rest or ("--pid" in values) == ("--pid-file" in values):
        return refuse(f"sample 认不出这组参数：{' '.join(arguments)}", "写成 sample (--pid <进程号> | --pid-file <文件>) [--declarations <文件>]")
    pid = pid_from(values)
    sample = (tree_sample(pid) if pid is not None else None) or gone_sample(pid)
    if "--declarations" in values and os.path.isfile(values["--declarations"]):
        with open(values["--declarations"], encoding="utf-8", errors="replace") as handle:
            sample["declarations_text"] = handle.read()
    print(json.dumps(sample, ensure_ascii=False, separators=(",", ":")))
    return 0


def command_judge(arguments):
    names = ("--pid", "--incoming", "--series", "--declarations", "--subject", "--alert-pid", "--source", "--label", "--next-step")
    values, rest = option_values(arguments, names)
    if rest or ("--pid" in values) == ("--incoming" in values) or "--series" not in values or "--subject" not in values:
        return refuse(f"judge 认不出这组参数：{' '.join(arguments)}",
                      "写成 judge (--pid <进程号> | --incoming <文件>) --series <样本文件> [--declarations <声明文件>] --subject <主体> "
                      "[--alert-pid <进程号>] [--source <谁在判>] [--label <名>] [--next-step <出路>]")
    try:
        registered = registered_threads()
    except Exception as error:   # heavy_threads 拒了（登记写坏、SINGLEFS_THREAD_CAP 不是正整数）或读不到那份库
        return refuse(f"取不了每台默认重活线程数：{error}", getattr(error, "howto", "恢复 scripts/hosts/heavy_threads.py 与 .claude/heavy-thread-default"), 1)
    declarations_path = values.get("--declarations", "")
    if "--pid" in values:
        pid = pid_from(values)
        sample = (tree_sample(pid) if pid is not None else None) or gone_sample(pid)
    else:
        try:
            with open(values["--incoming"], encoding="utf-8", errors="replace") as handle:
                sample = json.loads(handle.read().strip().split("\n")[-1])
        except (OSError, ValueError, IndexError) as error:
            print(f"  · {values['--subject']}这一次的样本读不出（{type(error).__name__}），下一次再判")
            return 0
        if not isinstance(sample, dict) or "time" not in sample:
            print(f"  · {values['--subject']}这一次的样本不成形，下一次再判")
            return 0
        text = sample.pop("declarations_text", None)
        if text is not None and declarations_path:
            os.makedirs(os.path.dirname(os.path.abspath(declarations_path)), exist_ok=True)
            with open(declarations_path, "w", encoding="utf-8") as handle:
                handle.write(text)
    samples = read_series(values["--series"]) + [sample]
    append_line(values["--series"], json.dumps(sample, ensure_ascii=False, separators=(",", ":")))
    entries, problems = read_declarations(declarations_path)
    window, note = window_seconds()
    if note:
        print(f"  · {note}")
    verdict = judge_series(samples, entries, problems, registered, window)
    next_step = values.get("--next-step") or DEFAULT_NEXT_STEP
    lines = verdict_lines(verdict, values["--subject"], next_step)
    print("\n".join(lines))
    if verdict["status"] != "low":
        return 0
    if broken("alert-record-skipped"):
        return 3
    path = alerts_path()
    alert_pid = values.get("--alert-pid", "")
    record = alert_record(verdict, values["--subject"], values.get("--source", ""), values.get("--label", ""),
                          int(alert_pid) if alert_pid.isdigit() else None, next_step, lines[0])
    try:
        if path is None:
            raise OSError(f"读不到 {WORK_ROOT_LIBRARY}，不知道工作根在哪")
        append_line(path, json.dumps(record, ensure_ascii=False, separators=(",", ":")))
    except OSError as error:
        print(f"  ! 告警记录写不进（{error}）：看门狗读不到这一条，靠上面那一行红行叫人看")
        print(f"     → 怎么办：设 {ALERTS_VARIABLE} 指到可写的文件")
    return 3


def command_declare(kind, arguments):
    path = os.environ.get(DECLARATIONS_VARIABLE, "")
    if not path:
        return refuse(f"没设 {DECLARATIONS_VARIABLE}，声明没处写",
                      "经 multi-host-run.sh --peer、detached-run.sh start 起的命令里它已经设好；别的跑法由起它的脚本 export 一个文件路径，"
                      "agent-watch.py 到那棵进程树的环境里找它")
    if kind == "serial":
        reason = " ".join(arguments).strip()
        if len(reason) < REASON_MINIMUM_CHARACTERS:
            return refuse(f"串行的理由「{reason}」不到 {REASON_MINIMUM_CHARACTERS} 个字", "写清哪一步、为什么必须串行（计时窗口必须单线程、共享状态必须独占这一类）")
        line = f"{time.time():.3f}\t{SERIAL_MARK}{reason}"
    else:
        if arguments:
            return refuse("declare-parallel 不带参数", "写成 core_usage.py declare-parallel")
        line = f"{time.time():.3f}\t{PARALLEL_MARK}"
    try:
        append_line(path, line)
    except OSError as error:
        return refuse(f"声明写不进 {path}（{error}）", "看那个目录在不在、可不可写", 1)
    return 0


BUSY_BLOCK = b"\0" * (1 << 20)


def busy_load(threads, seconds):
    """自证用的假负载：threads 条线程各忙等 seconds 秒（哈希放开解释器锁，几条线程真的同时在算）。"""
    deadline = time.monotonic() + seconds

    def spin():
        while time.monotonic() < deadline:
            hashlib.sha256(BUSY_BLOCK).digest()

    workers = [threading.Thread(target=spin) for _ in range(threads)]
    for worker in workers:
        worker.start()
    for worker in workers:
        worker.join()


def command_busy_load(arguments):
    if len(arguments) != 2 or not re.fullmatch(r"[1-9][0-9]*", arguments[0]) or not re.fullmatch(r"[0-9]+(\.[0-9]+)?", arguments[1]) \
            or float(arguments[1]) > 120:
        return refuse(f"busy-load 认不出这组参数：{' '.join(arguments)}", "写成 busy-load <线程数> <秒>（秒不超过 120）")
    busy_load(int(arguments[0]), float(arguments[1]))
    return 0


def main(arguments):
    if arguments == ["--selftest"]:
        return selftest()
    if os.environ.get(BREAK_VARIABLE) and not selftest_mark_valid():
        return refuse(f"{BREAK_VARIABLE}={os.environ[BREAK_VARIABLE]} 设在 --selftest 之外：弄坏开关只在自证里认，这一次什么都没做",
                      f"去掉它再跑（env -u {BREAK_VARIABLE} …）；要证红就跑 {BREAK_VARIABLE}=<项> python3 scripts/hosts/core_usage.py --selftest")
    commands = {"sample": command_sample, "judge": command_judge, "busy-load": command_busy_load,
                "declare-serial": lambda rest: command_declare("serial", rest), "declare-parallel": lambda rest: command_declare("parallel", rest)}
    if not arguments or arguments[0] not in commands:
        return refuse(f"认不出的用法：{' '.join(arguments)}", "用法在 scripts/hosts/core_usage.py 文件头：sample、judge、declare-serial、declare-parallel、busy-load、--selftest")
    try:
        return commands[arguments[0]](arguments[1:])
    except ValueError as error:
        return refuse(str(error), "照 scripts/hosts/core_usage.py 文件头的用法写")


# ── 自证 ──────────────────────────────────────────────────────────────────────
# 真负载（不起编译与重活，只起 busy-load 与 bash、sleep）：
#   登记数那么多条线程忙等 / 一条线程忙等：两份同时起，窗口压到 2 秒，经 judge --pid 先取一次、隔 2.3 秒再判，判完就停；前者退 0、不写告警记录，后者退 3、打 ✗ 与出路、写一条带会话号的告警记录
#   bash 起的子进程忙等：树根是 bash，取样的 CPU 秒数不少于子进程自己的
#   子进程忙 1 秒退出、bash 收过尸之后：取样的 CPU 秒数照算（cutime）
#   假的 multi-host-run.sh --peer 底下忙等：那一支不算，判成在等第二台
# 造的样本序列（只判判法）：机器忙、窗口里声明了串行、声明累计超过一半、理由太短、窗口调长、不满一个窗口
# 直接执行的用法：declare-serial 没设变量、理由太短都拒，写对了落一行；CORE_USAGE_BREAK 设在自证之外拒

def selftest():
    global SELFTEST_ACTIVE
    SELFTEST_ACTIVE = True
    cells = []
    work = tempfile.mkdtemp(prefix="core-usage-selftest-")
    started = []

    def cell(name, ok, detail):
        cells.append((name, bool(ok), detail))

    def run_cli(arguments, extra_environment=None, timeout=60):
        environment = dict(os.environ, **{SELFTEST_MARK_VARIABLE: mark}, **(extra_environment or {}))
        return subprocess.run([sys.executable, os.path.abspath(__file__), *arguments], capture_output=True, text=True,
                              env=environment, timeout=timeout, stdin=subprocess.DEVNULL)

    def descendants(pid):
        table = process_table()
        found, frontier = [], [pid]
        while frontier:
            parent = frontier.pop()
            for child, info in table.items():
                if info["parent"] == parent:
                    found.append(child)
                    frontier.append(child)
        return found

    def stop_tree(process):
        """只停这份自证自己起的那个进程与它底下的（逐个按进程号，从最底层起）。"""
        def stop_one_then_the_rest(pids):
            # 进程号都是从这份自证自己起的树根列出的后代；每一层只发一个信号，不在循环里批量发
            if not pids:
                return
            try:
                os.kill(pids[0], 9)
            except OSError:
                pass
            stop_one_then_the_rest(pids[1:])
        stop_one_then_the_rest(list(reversed(descendants(process.pid))))
        if process.poll() is None:
            process.kill()
        process.wait()

    def start(argv):
        process = subprocess.Popen(argv, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, stdin=subprocess.DEVNULL,
                                   env=dict(os.environ, **{SELFTEST_MARK_VARIABLE: mark}))
        started.append(process)
        return process

    mark = os.path.join(work, "selftest-mark")
    try:
        with open(mark, "w", encoding="utf-8") as handle:
            handle.write(SELFTEST_MARK_CONTENT + "\n")
        try:
            registered = registered_threads()
        except Exception as error:   # 取不了登记数：真负载那几格判不了
            print(f"  ✗ core_usage.py 自证：取不了每台默认重活线程数（{error}），真负载那几格判不了")
            print("     → 怎么办：照 scripts/hosts/heavy_threads.py 文件头修好 .claude/heavy-thread-default 再跑")
            return 1
        script = os.path.abspath(__file__)
        # 真负载：登记数那么多条线程忙等与一条线程忙等，同时起、窗口 2 秒
        load_started = time.monotonic()
        many = start([sys.executable, script, "busy-load", str(registered), "30"])
        one = start([sys.executable, script, "busy-load", "1", "30"])
        alerts = os.path.join(work, "alerts.jsonl")
        judged = {}

        def judge_both():
            for label, process in (("many", many), ("one", one)):
                judged[label] = run_cli(["judge", "--pid", str(process.pid), "--series", os.path.join(work, f"{label}.series"), "--subject", "本机",
                                         "--alert-pid", str(process.pid), "--source", "core_usage.py 自证", "--label", label],
                                        {WINDOW_VARIABLE: "2", ALERTS_VARIABLE: alerts, SESSION_VARIABLE: "core-usage-selftest-session"})

        # 先取一次（起动那一段的 CPU 不进窗口：机器忙时主线程起动用的 CPU 会压过每条忙等线程分到的），过了一个窗口再判；
        #   两份各忙 30 秒、判完就停，机器忙时 judge 起得慢也赶得上
        time.sleep(max(0.0, load_started + 0.8 - time.monotonic()))
        judge_both()
        time.sleep(2.3)
        judge_both()
        records = read_alert_records(alerts)
        many_out, one_out = judged["many"].stdout, judged["one"].stdout
        cell(f"{registered} 条线程忙等：judge 退 0、打「本机在用 X 核（登记 {registered}）」、不打 ✗、不写告警记录",
             judged["many"].returncode == 0 and f"核（登记 {registered}）" in many_out and "✗" not in many_out
             and not any(record.get("label") == "many" for record in records),
             f"退 {judged['many'].returncode}，输出：{many_out.strip()}{judged['many'].stderr.strip()}；告警记录 {len(records)} 条")
        one_records = [record for record in records if record.get("label") == "one"]
        cell("一条线程忙等：judge 退 3、打 ✗ 与出路、写告警记录（带会话号）",
             judged["one"].returncode == 3 and f"✗ 本机在用" in one_out and f"核（登记 {registered}）" in one_out and "→ 怎么办：" in one_out
             and len(one_records) == 1 and one_records[0].get("session_id") == "core-usage-selftest-session"
             and one_records[0].get("alert_pid") == one.pid,
             f"退 {judged['one'].returncode}，输出：{one_out.strip()}{judged['one'].stderr.strip()}；这一条的告警记录：{one_records}")
        for process in (many, one):
            stop_tree(process)
        # bash 起的子进程忙等：等子进程用过 0.05 秒 CPU 再取（机器忙时 python 起得慢，不按固定秒数等）
        child_tree = start(["bash", "-c", f'"$0" {script} busy-load 1 20; exit 0', sys.executable])
        table, child_pids, child_cpu = {}, [], 0.0
        for _ in range(400):
            table = process_table()
            child_pids = [pid for pid, info in table.items() if info["parent"] == child_tree.pid]
            child_cpu = sum(table[pid]["own_ticks"] for pid in child_pids) / os.sysconf("SC_CLK_TCK")
            if child_cpu >= 0.05:
                break
            time.sleep(0.05)
        sample = tree_sample(child_tree.pid, table)
        stop_tree(child_tree)
        cell("子进程里忙等的也算进来",
             sample is not None and child_pids and child_cpu > 0 and sample["cpu"] >= child_cpu * 0.9,
             f"树根 bash {child_tree.pid} 的样本 {sample and sample['cpu']} 秒，子进程 {child_pids} 自己 {child_cpu:.2f} 秒")
        cell("取样数线程状态：子进程一条线程忙等的那棵树，样本里在算（状态 R）的线程至少 1 条、带在等盘（状态 D）的线程数",
             sample is not None and sample.get("computing_threads", 0) >= 1 and isinstance(sample.get("disk_waiting_threads"), int),
             f"样本里 computing_threads={sample and sample.get('computing_threads')}、disk_waiting_threads={sample and sample.get('disk_waiting_threads')}")
        # 子进程忙 1 秒退出、bash 收过尸：等 bash 底下只剩 sleep 再取
        reaped_tree = start(["bash", "-c", f'"$0" {script} busy-load 1 1; exec sleep 30', sys.executable])
        for _ in range(600):
            if process_arguments(reaped_tree.pid)[:1] == ["sleep"]:
                break
            time.sleep(0.05)
        sample = tree_sample(reaped_tree.pid)
        stop_tree(reaped_tree)
        cell("中途退出、收过尸的子进程的 CPU 照算",
             sample is not None and sample["cpu"] >= 0.05,
             f"树根 {reaped_tree.pid}（bash 收过尸之后 exec 成 sleep）的样本 {sample and sample['cpu']} 秒（子进程忙 1 秒之后退出了）")
        # 假的 multi-host-run.sh --peer：等它底下的忙等起来再取
        fake_driver = os.path.join(work, PEER_DRIVER_NAME)
        with open(fake_driver, "w", encoding="utf-8") as handle:
            handle.write('while [[ "$1" != -- ]]; do shift; done; shift; "$@"; exit 0\n')
        peer_tree = start(["bash", "-c", f'bash {fake_driver} --peer -- "$0" {script} busy-load 1 20; exit 0', sys.executable])
        for _ in range(400):
            if len(descendants(peer_tree.pid)) >= 2:
                break
            time.sleep(0.05)
        time.sleep(0.5)
        sample = tree_sample(peer_tree.pid)
        stop_tree(peer_tree)
        verdict = judge_series([sample], [], [], registered, 1) if sample else None
        cell("multi-host-run.sh --peer 那一支不算、判成在等第二台",
             sample is not None and sample["peer_drivers"] == 1 and sample["cpu"] < 0.3 and verdict["status"] == "waiting-peer",
             f"样本 {sample and {key: sample[key] for key in ('cpu', 'processes', 'peer_drivers')}}，判成 {verdict and verdict['status']}")
        # 造的样本序列
        origin = 1_000_000.0

        def synthetic(cpu, threads, length=600.0):
            share = cpu / threads
            return [{"root": 1, "time": origin + length, "started": origin, "cpu": cpu, "peer_drivers": 0,
                     "thread_cpu": {f"1/{index}/0": share for index in range(threads)}}]

        verdict = judge_series(synthetic(1200.0, registered), [], [], registered, 600)
        cell("机器忙：在用核数低而登记数那么多条线程都在算的，不报",
             verdict["status"] == "ok", f"在用 {verdict['cores']:.1f} 核、同时在算 {verdict['parallel']:.1f} 条，判成 {verdict['status']}")
        entries, problems = parse_declarations(f"{origin + 100}\t{SERIAL_MARK}计时窗口必须单线程跑完\n{origin + 200}\t{PARALLEL_MARK}\n")
        verdict = judge_series(synthetic(600.0, 1), entries, problems, registered, 600)
        lines = verdict_lines(verdict, "第二台")
        cell("声明了串行段的照报，标出秒数与理由",
             verdict["status"] == "low" and "声明了串行 100 秒：计时窗口必须单线程跑完" in lines[0] and "超过一半" not in lines[0],
             f"判成 {verdict['status']}，那一行：{lines[0]}")
        entries, problems = parse_declarations(f"{origin + 100}\t{SERIAL_MARK}计时窗口必须单线程跑完\n")
        verdict = judge_series(synthetic(600.0, 1), entries, problems, registered, 600)
        lines = verdict_lines(verdict, "第二台")
        cell("声明的串行段累计超过整趟一半的，注明超过一半",
             verdict["status"] == "low" and "累计占整趟 83%，超过一半" in lines[0], f"判成 {verdict['status']}，那一行：{lines[0]}")
        entries, problems = parse_declarations(f"{origin + 100}\t{SERIAL_MARK}太短\n")
        verdict = judge_series(synthetic(600.0, 1), entries, problems, registered, 600)
        lines = verdict_lines(verdict, "第二台")
        cell("理由不到 8 个字的不算声明，判的那一行点出来",
             verdict["status"] == "low" and "没有声明串行段" in lines[0] and "写坏的 1 行" in lines[0], f"那一行：{lines[0]}")
        saved = os.environ.get(WINDOW_VARIABLE)
        os.environ[WINDOW_VARIABLE] = "3600"
        try:
            window, note = window_seconds()
        finally:
            if saved is None:
                os.environ.pop(WINDOW_VARIABLE, None)
            else:
                os.environ[WINDOW_VARIABLE] = saved
        verdict = judge_series(synthetic(700.0, 1, 700.0), [], [], registered, window)
        cell("窗口不许调长", window == WINDOW_SECONDS and note and verdict["status"] == "low",
             f"{WINDOW_VARIABLE}=3600 取到 {window} 秒（{note}），判成 {verdict['status']}")
        verdict = judge_series(synthetic(300.0, 1, 300.0), [], [], registered, 600)
        cell("不满一个窗口不判", verdict["status"] == "warming" and "还不判" in verdict_lines(verdict, "本机")[0], f"判成 {verdict['status']}")

        # 等盘（造的样本序列）：一条线程算满整个窗口（1 核，低于阈值），窗口里三次取样各记状态 D 与状态 R 的线程几条
        def state_series(disk_counts, computing_counts):
            samples = []
            for index, (disk, computing) in enumerate(zip(disk_counts, computing_counts), 1):
                moment = origin + 200.0 * index
                samples.append({"root": 1, "time": moment, "started": origin, "cpu": moment - origin, "peer_drivers": 0,
                                "thread_cpu": {"1/1/0": moment - origin}, "disk_waiting_threads": disk, "computing_threads": computing})
            return samples

        verdict = judge_series(state_series([1, 1, 0], [0, 0, 1]), [], [], registered, 600)
        line = verdict_lines(verdict, "本机")[0]
        cell("等盘：窗口里在等盘的线程次数（2）多于在算的（1），标等盘、不判核数过少",
             verdict["status"] == "disk-wait" and "标等盘" in line and "✗" not in line and "在等盘（状态 D）的线程合计 2 次、在算（状态 R）的 1 次" in line,
             f"判成 {verdict['status']}，那一行：{line}")
        verdict = judge_series(state_series([0, 1, 0], [1, 0, 1]), [], [], registered, 600)
        line = verdict_lines(verdict, "本机")[0]
        cell("等盘：窗口里在算的线程次数（2）多于在等盘的（1），照报核数过少",
             verdict["status"] == "low" and line.startswith("  ✗") and "标等盘" not in line, f"判成 {verdict['status']}，那一行：{line}")
        # 窗口里有进程离开了这棵树（造的样本序列）：基准 1000 秒里 900 秒是一个子树里的，到这一次它不在了，合计掉到 130 秒；还在树里的线程这一窗口涨了 60 秒
        base_sample = {"root": 1, "time": origin + 100, "started": origin, "cpu": 1000.0, "peer_drivers": 0, "thread_cpu": {"1/1/0": 50.0, "2/2/0": 900.0}}
        dropped_sample = {"root": 1, "time": origin + 700, "started": origin, "cpu": 130.0, "peer_drivers": 0, "thread_cpu": {"1/1/0": 110.0}}
        verdict = judge_series([base_sample, dropped_sample], [], [], registered, 600)
        line = verdict_lines(verdict, "第二台")[0]
        cell("窗口里有进程离开了这棵树、合计 CPU 秒数从 1000 掉到 130：在用核数不为负、不低于还在树里的线程的用量（60 秒 ÷ 600 秒 = 0.1 核），判的那一行写明掉了 930 秒",
             verdict["cores"] >= 0.1 - 1e-9 and "合计 CPU 秒数掉了 930 秒" in line,
             f"在用 {verdict['cores']:.3f} 核（下限 0.1），掉了 {verdict['dropped_cpu']:.0f} 秒，那一行：{line}")
        # 真进程：树里的孙进程（忙等）在两次取样之间被孤立——中间那个进程退出，孙进程改挂到树外。旧算法相减是负数（2026-10-21 核实：-0.76 核）
        orphan_directory = os.path.join(work, "orphan")
        os.makedirs(orphan_directory)
        release, grand_file = os.path.join(orphan_directory, "release"), os.path.join(orphan_directory, "grand.pid")
        middle = (f'{shlex.quote(sys.executable)} {shlex.quote(script)} busy-load 1 8 & echo $! > {shlex.quote(grand_file)}; '
                  f'while [ ! -e {shlex.quote(release)} ]; do sleep 0.05; done; exit 0')
        orphan_tree = start(["bash", "-c", f"bash -c {shlex.quote(middle)}; exec sleep 60"])
        first = second = None
        try:
            for _ in range(300):
                grand_text = open(grand_file, encoding="utf-8").read().strip() if os.path.exists(grand_file) else ""
                first = tree_sample(orphan_tree.pid) if grand_text else None
                if first is not None and first["cpu"] >= 0.6:
                    break
                time.sleep(0.05)
            with open(release, "w", encoding="utf-8"):
                pass
            for _ in range(300):
                if process_arguments(orphan_tree.pid)[:1] == ["sleep"]:
                    break
                time.sleep(0.05)
            time.sleep(max(0.0, (first["time"] if first else time.time()) + 1.2 - time.time()))
            second = tree_sample(orphan_tree.pid)
        finally:
            stop_tree(orphan_tree)
            grand_text = open(grand_file, encoding="utf-8").read().strip() if os.path.exists(grand_file) else ""
            if grand_text.isdigit():
                try:
                    os.kill(int(grand_text), 9)   # 这份自证自己起的那个忙等孙进程（现在不是树里的后代了）
                except OSError:
                    pass
        verdict = judge_series([first, second], [], [], registered, 1) if first and second else None
        cell("真进程：孙进程被孤立（中间那个进程退出）、合计 CPU 秒数掉了：在用核数不为负，不低于还在树里的线程的用量，掉了的秒数写进 verdict",
             verdict is not None and first["cpu"] - second["cpu"] > 0.3 and verdict["cores"] >= 0 and verdict["dropped_cpu"] > 0.3,
             f"两次取样的合计 {first and first['cpu']} → {second and second['cpu']} 秒（要掉 0.3 秒以上才算造出这种情形）；判成 {verdict and (round(verdict['cores'], 3), round(verdict['dropped_cpu'], 2))}")
        # 直接执行的用法
        declarations = os.path.join(work, "declarations")
        unset = run_cli(["declare-serial", "计时窗口必须单线程跑完"], {DECLARATIONS_VARIABLE: ""})
        short = run_cli(["declare-serial", "太短"], {DECLARATIONS_VARIABLE: declarations})
        good = run_cli(["declare-serial", "计时窗口必须单线程跑完"], {DECLARATIONS_VARIABLE: declarations})
        back = run_cli(["declare-parallel"], {DECLARATIONS_VARIABLE: declarations})
        entries, problems = read_declarations(declarations)
        cell("declare-serial：没设变量、理由不到 8 个字都拒（带出路），写对了落一行、declare-parallel 收尾",
             unset.returncode == 2 and short.returncode == 2 and "→ 怎么办" in unset.stderr and good.returncode == 0 and back.returncode == 0
             and [kind for _, kind, _ in entries] == ["serial", "parallel"] and not problems,
             f"没设变量退 {unset.returncode}、太短退 {short.returncode}、写对了退 {good.returncode}、收尾退 {back.returncode}；读回 {entries} {problems}")
        environment = {key: value for key, value in os.environ.items() if key != SELFTEST_MARK_VARIABLE}
        environment[BREAK_VARIABLE] = "alert-record-skipped"
        outside = subprocess.run([sys.executable, script, "declare-parallel"], capture_output=True, text=True, env=environment,
                                 timeout=60, stdin=subprocess.DEVNULL)
        cell(f"{BREAK_VARIABLE} 设在自证之外：拒、退 2",
             outside.returncode == 2 and "设在 --selftest 之外" in outside.stderr, f"退 {outside.returncode}：{outside.stderr.strip()}")
    finally:
        for process in started:
            if process.poll() is None:
                process.kill()
            process.wait()
        subprocess.run(["rm", "-rf", "--", work], check=False)
    failures = [(name, detail) for name, ok, detail in cells if not ok]
    for name, ok, detail in cells:
        if ok:
            print(f"  · 对：{name}")
        else:
            print(f"  ✗ core_usage.py 自证：{name}：{detail}")  # gate-lint:detail
    if failures:
        print(f"  ✗ core_usage.py 自证有 {len(failures)} 格判错（共 {len(cells)} 格）")
        print(f"     → 怎么办：设了 {BREAK_VARIABLE} 的，这就是要的结果（那个开关把对应的一步弄坏了）；没设的，照上面那几格去看 scripts/hosts/core_usage.py")
        return 1
    print(f"  ✓ core_usage.py 自证 {len(cells)} 格都对（登记数那么多条线程忙等不报、一条线程忙等报并写告警记录、子进程与收过尸的子进程照算、"
          "multi-host-run.sh --peer 那一支判成在等第二台、机器忙不误报、声明照报并标出理由与超过一半、理由太短不算、窗口不许调长、不满一个窗口不判、"
          "取样数线程状态、窗口里等盘占多数标等盘而在算占多数照报、声明的用法、开关设在自证之外拒）")
    return 0


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.join(os.path.dirname(os.path.realpath(__file__)), "..", "..", ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
