#!/usr/bin/env python3
# admission: always 每一趟现读这一次尝试的内存上限与共享根里此刻的峰值记录，上一趟的结论不替这一趟作保
# run-condition: command python3
"""复跑入口 scripts/experiment-apparatus/replay.sh 派行要的读数：这一次尝试的内存上限、每一行量过的峰值与线程数；开跑之后盯着在跑的每一行、量峰值、报进度。
只报资源用量，不判、不存判定：判定只在 prune-server 那一份库里（.claude/rules/verification.md「判定库只有一份」）。自证在 replay.sh 自证的派行那一组（它文件头「派行」），这一份不另带。

上限与量法都用 scripts/hosts/task_memory.py 的（task_memory_cap_bytes、process_tree_usage），import 它，不另写一套；不按路径载入 crates-mutation-rows.py：
整份载入会把它的自证与它载入的整份脚本都拉进 scripts/gate/admission.py 的闭包，闭包里有拒算，按条复用整张表拒算。

峰值记录（运行记录，放共享根，两台读写同一份）：<共享根>/replay-row-peaks/<这一行身份的 sha256 前 24 位>/<记录名>，
  记录名用 task_memory.py 的 new_worker_peak_record_name（名字排序就是先后）；内容一行「<峰值字节> <线程数><制表符><身份>」。身份是复跑表前三列「实验号|二进制|参数」。
  只有装置退 0 的那一次才记：退非 0 的多半提前退了，峰值偏小，拿它派行会把大行当小行。
  共享根：多机配置 NFS_SHARED_ROOTS 里不含本仓、也不在本仓之内的几段里排在最前的一段。REPLAY_ROW_COSTS_DIRECTORY 设了照它（自证用）；
  没设而在自证底下的不读不写；配置里没有这一项的不读不写，各行都照没量过办（replay.sh 文件头「派行」）。
  读（row_measurement）：每个键只看名字排在最后的 RECENT_RECORDS 份、更旧的读的时候删掉，峰值与线程数各取最大；有一份写法不对的，这一行照没量过办，打一行点名那一份与出路。

用法（参数都是绝对路径或相对当前目录）：
  replay_row_costs.py read <请求文件>   请求文件一行「<tag><制表符><身份>」。标准输出：一行「cap<制表符><字节或 -><制表符><说明>」、一行「records<制表符><目录或 -><制表符><说明>」，
                                        量过的行各一行「row<制表符><tag><制表符><峰值字节><制表符><线程数>」。
  replay_row_costs.py sample <监看目录> <峰值记录目录或 -> [--progress-seconds <秒，0 不报>] [--no-records] [--record-failed]
                                        起来先写一份 ready，一直跑到监看目录里出现 stop，或起它的进程没了。在跑的行由 replay.sh 往监看目录里写：
                                        <tag>.payload 一行「<进程号><制表符><身份>」；<tag>.end 一行「<退出码> <线程数>」（都是改名换上）。
                                        每 SAMPLE_SECONDS 秒量一次每一行的整棵进程树，写 <tag>.peak（到此刻的最大，字节）；每过进度间隔往标准错误打一行「REPLAY_ROW <tag> 在跑 …」。
                                        比采样间隔还短的行量不到，照没量过办（推的：这种行独占也只占不到一秒）。
                                        --no-records、--record-failed 是 replay.sh 的弄坏开关 row-peak-unrecorded、failed-row-peak-recorded 交下来的。
  replay_row_costs.py record <峰值记录目录> <身份> <峰值字节> <线程数>   写一份记录（自证造量过的行用）
退出码：0 做完；1 跑不下去（task_memory.py 要读的 admission.py 或多机配置读不进来）；2 用法错。
"""
import hashlib
import os
import re
import sys
import time

sys.dont_write_bytecode = True
SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "..", ".claude", "scripts"))
from project_preflight import preflight  # noqa: E402
sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "hosts"))
import task_memory  # noqa: E402

REPOSITORY = os.path.realpath(os.path.join(SCRIPT_DIRECTORY, "..", ".."))
DIRECTORY_VARIABLE = "REPLAY_ROW_COSTS_DIRECTORY"
SELFTEST_MARK_VARIABLE = "REPLAY_SELFTEST_MARK"   # replay.sh 的自证起的每一份副本都继承它
DIRECTORY_NAME = "replay-row-peaks"
SAMPLE_SECONDS = 0.2                  # 隔多久量一次在跑的每一行
PEAK_WRITE_STEP_BYTES = 16 << 20      # 这一行的最大比上一次写下的涨了这么多才重写 <tag>.peak
RECENT_RECORDS = 32                   # 每个键只看最新的这么多份（一趟一份；推的：够盖住几十趟，旧的偏大读数也挤得掉）
PROGRESS_TAIL_BYTES = 200             # 进度行带这一行错误输出的末行，至多读这么多字节
RECORD_NAME_FORM = re.compile(r"[0-9]{20}-[0-9]+-[a-z0-9]{8}")
RECORD_CONTENT_FORM = re.compile(r"([1-9][0-9]*) ([1-9][0-9]*)\t(.+)\n")


def usage_error(reason):
    print(f"  ✗ replay_row_costs.py：{reason}", file=sys.stderr)
    print("     → 怎么办：照 scripts/experiment-apparatus/replay_row_costs.py 文件头「用法」改；它只由 replay.sh 起", file=sys.stderr)
    return 2


def records_directory_of(environment):
    """峰值记录放哪（文件头「峰值记录」）：(目录或 None, 说明)。"""
    given = environment.get(DIRECTORY_VARIABLE, "").strip()
    if given:
        return given, f"{DIRECTORY_VARIABLE} 指的 {given}"
    if environment.get(SELFTEST_MARK_VARIABLE, "").strip():
        return None, f"在自证底下又没设 {DIRECTORY_VARIABLE}：不读不写共享根里的峰值记录"
    configuration, present = task_memory.test_runs_multi_host_configuration(environment)
    if not present:
        return None, f"多机配置 {configuration} 不在：峰值记录没处放，不读不写"
    admission = task_memory.admission_module()
    try:
        shared_roots = admission.configuration_value(configuration, "NFS_SHARED_ROOTS")
    except admission.InputManifestError as error:
        return None, f"{error}：峰值记录不读不写"
    repository = REPOSITORY.rstrip(os.sep) + os.sep

    def overlaps(candidate):
        real = os.path.realpath(candidate).rstrip(os.sep) + os.sep
        return real.startswith(repository) or repository.startswith(real)

    candidates = [path for path in (shared_roots or "").split(":") if os.path.isabs(path) and not overlaps(path)]
    if not candidates:
        return None, f"多机配置 {configuration} 里没有不含本仓的 NFS_SHARED_ROOTS：峰值记录不读不写"
    return os.path.join(candidates[0], DIRECTORY_NAME), f"多机配置 {configuration} 的 NFS_SHARED_ROOTS 里的 {candidates[0]}"


def key_directory(records_directory, identity):
    return os.path.join(records_directory, hashlib.sha256(identity.encode("utf-8", "surrogateescape")).hexdigest()[:24])


def row_measurement(records_directory, identity):
    """一行量过的 (峰值字节, 线程数)；没量过或记录不对交 None，记录不对的打一行点名与出路。"""
    directory = key_directory(records_directory, identity)
    try:
        names = sorted(name for name in os.listdir(directory) if not name.startswith("."))
    except FileNotFoundError:
        return None
    except OSError as error:
        print(f"  ! 峰值记录 {directory} 读不了（{type(error).__name__}：{error}）；{identity} 照没量过办；出路：删掉这个目录再跑", file=sys.stderr)
        return None
    for stale in names[:-RECENT_RECORDS]:
        try:
            os.remove(os.path.join(directory, stale))
        except OSError:
            pass
    peaks, threads = [], []
    for name in names[-RECENT_RECORDS:]:
        path = os.path.join(directory, name)
        try:
            with open(path, encoding="utf-8", errors="surrogateescape") as handle:
                found = RECORD_CONTENT_FORM.fullmatch(handle.read())
        except OSError as error:
            print(f"  ! 峰值记录 {path} 读不了（{type(error).__name__}：{error}）；{identity} 照没量过办；出路：删掉那一份再跑", file=sys.stderr)
            return None
        if not RECORD_NAME_FORM.fullmatch(name) or found is None or found.group(3) != identity:
            print(f"  ! 峰值记录 {path} 不是这一版的写法；{identity} 照没量过办；出路：删掉那一份再跑", file=sys.stderr)
            return None
        peaks.append(int(found.group(1)))
        threads.append(int(found.group(2)))
    return (max(peaks), max(threads)) if peaks else None


def write_replacing(path, text):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    temporary = f"{path}.tmp.{os.getpid()}"
    with open(temporary, "w", encoding="utf-8", errors="surrogateescape") as handle:
        handle.write(text)
    os.replace(temporary, path)


def write_record(records_directory, identity, peak, threads):
    write_replacing(os.path.join(key_directory(records_directory, identity), task_memory.new_worker_peak_record_name()), f"{peak} {threads}\t{identity}\n")


def command_read(request_path):
    cap, cap_text = task_memory.task_memory_cap_bytes(os.environ)
    directory, directory_text = records_directory_of(os.environ)
    print(f"cap\t{cap if cap is not None else '-'}\t{' '.join(str(cap_text).split())}")
    print(f"records\t{directory or '-'}\t{' '.join(directory_text.split())}")
    with open(request_path, encoding="utf-8", errors="surrogateescape") as handle:
        for line in handle:
            tag, _separator, identity = line.rstrip("\n").partition("\t")
            measured = row_measurement(directory, identity) if directory and identity else None
            if measured is not None:
                print(f"row\t{tag}\t{measured[0]}\t{measured[1]}")
    return 0


def read_text(path):
    try:
        with open(path, encoding="utf-8", errors="surrogateescape") as handle:
            return handle.read()
    except OSError:
        return ""


def error_tail(path):
    """这一行错误输出里最后一个不空的行（进度行带上它）；控制字符去掉。"""
    try:
        with open(path, "rb") as handle:
            handle.seek(0, os.SEEK_END)
            handle.seek(max(handle.tell() - PROGRESS_TAIL_BYTES, 0))
            lines = handle.read().decode("utf-8", "replace").splitlines()
    except OSError:
        return ""
    for line in reversed(lines):
        if line.strip():
            return "".join(character for character in line.strip() if character.isprintable())
    return ""


class WatchedRow:
    def __init__(self, pid, identity):
        self.pid, self.identity = pid, identity
        self.peak, self.written = 0, None
        self.started, self.reported = time.monotonic(), time.monotonic()


def command_sample(watch, records_directory, progress_seconds, record_success, record_failed):
    write_replacing(os.path.join(watch, "ready"), "")
    parent = os.getppid()
    rows, finished = {}, set()
    while True:
        stopping = os.path.exists(os.path.join(watch, "stop")) or os.getppid() != parent   # 先读：stop 之前写下的 .end 这一遍都看得到
        for name in sorted(os.listdir(watch)):
            tag = name[:-len(".payload")]
            if name.endswith(".payload") and tag not in rows and tag not in finished:
                pid_text, _separator, identity = read_text(os.path.join(watch, name)).rstrip("\n").partition("\t")
                if pid_text.isdigit() and identity:
                    rows[tag] = WatchedRow(int(pid_text), identity)
        now = time.monotonic()
        for tag, row in list(rows.items()):
            usage = task_memory.process_tree_usage(row.pid)
            if usage and usage > row.peak:
                row.peak = usage
                if row.written is None or row.peak >= row.written + PEAK_WRITE_STEP_BYTES:
                    write_replacing(os.path.join(watch, f"{tag}.peak"), f"{row.peak}\n")
                    row.written = row.peak
            words = read_text(os.path.join(watch, f"{tag}.end")).split()
            if len(words) == 2:
                if row.peak:
                    write_replacing(os.path.join(watch, f"{tag}.peak"), f"{row.peak}\n")
                if record_success and records_directory and row.peak and (words[0] == "0" or record_failed):
                    try:
                        write_record(records_directory, row.identity, row.peak, max(int(words[1]), 1))
                    except OSError as error:
                        print(f"  ! 峰值记录写不进 {records_directory}（{type(error).__name__}：{error}）；{row.identity} 下一趟照没量过办；出路：看共享根能不能写", file=sys.stderr, flush=True)
                finished.add(tag)
                del rows[tag]
            elif progress_seconds and now - row.reported >= progress_seconds:
                row.reported = now
                print(f"REPLAY_ROW {tag} 在跑 用时 {int(now - row.started)} 秒 峰值 {row.peak} 字节 末行「{error_tail(os.path.join(os.path.dirname(watch), f'{tag}.err'))}」", file=sys.stderr, flush=True)
        if stopping:
            return 0
        time.sleep(SAMPLE_SECONDS)


def main(arguments):
    if arguments[:1] == ["read"] and len(arguments) == 2:
        return command_read(arguments[1])
    if arguments[:1] == ["record"] and len(arguments) == 5 and arguments[3].isdigit() and arguments[4].isdigit():
        write_record(arguments[1], arguments[2], int(arguments[3]), int(arguments[4]))
        return 0
    if arguments[:1] == ["sample"] and len(arguments) >= 3:
        progress_seconds, record_success, record_failed, position = 0, True, False, 3
        while position < len(arguments):
            if arguments[position] == "--progress-seconds" and position + 1 < len(arguments) and arguments[position + 1].isdigit():
                progress_seconds = int(arguments[position + 1])
                position += 2
            elif arguments[position] == "--no-records":
                record_success = False
                position += 1
            elif arguments[position] == "--record-failed":
                record_failed = True
                position += 1
            else:
                return usage_error(f"认不得的参数「{arguments[position]}」")
        return command_sample(arguments[1], None if arguments[2] == "-" else arguments[2], progress_seconds, record_success, record_failed)
    return usage_error(f"认不得的参数：{' '.join(arguments)}")


if __name__ == "__main__":
    preflight(__file__)
    try:
        sys.exit(main(sys.argv[1:]))
    except Exception as error:   # task_memory.py 要读的 admission.py 或多机配置读不进来（读的库有一份写坏了的各种异常都在这里）
        print(f"  ✗ replay_row_costs.py 跑不下去（{type(error).__name__}：{error}）", file=sys.stderr)
        print("     → 怎么办：python3 -m py_compile scripts/hosts/task_memory.py scripts/gate/admission.py 看它们读不读得进来；这一趟 replay.sh 各行照没量过办、一行一行跑", file=sys.stderr)
        sys.exit(1)
