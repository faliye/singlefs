#!/usr/bin/env python3
# admission: always 每次照此刻的进程与 cgroup 现读，不存结论
# run-condition: none 只读 /proc 与 cgroup 文件、起入口交给它的那一条程序，不连服务、不起 cargo
"""E207（B8严格判据下的步数）入口分片那一步起测试程序时套的一层：起被测程序，标准输入、输出、错误原样交给它，每隔一段往标准错误打一行进程读数，
退出码照被测程序的交回。读数只给任务日志看「这一片在等什么」（平均只用半个核时，是等 CPU、等内存回收与换页、还是等读写），不进产物、不进判定。

跑前登记 research/prompts/e207-preregistration.md 第十二节执行员修订 10。入口 e207-strict-scan-steps-run.sh 的 shard_phase 起它；入口自证的
a_shard_prints_process_samples_and_keeps_the_exit_code 那一格证它（入口的弄坏开关 process-sample-off 下那一格红）。

用法：
  e207_strict_scan_steps_process_sampler.py --label <项> -- <程序> [参数…]
间隔：环境变量 E207_PROCESS_SAMPLE_SECONDS（正数，秒）设了就用它（入口自证用小数），没设是 PROCESS_SAMPLE_SECONDS_BY_DEFAULT。

每行的形态（一行一刻；stage=sample 是每个间隔一行，stage=end 是被测程序退出之后一行）：
  e207 process item=<项> stage=sample|end wall_seconds=<从起到此刻> user_seconds=<> system_seconds=<> minor_faults=<> major_faults=<>
    voluntary_switches=<> involuntary_switches=<> swap_kib=<> cgroup_cpu_waiting_usec=<> cgroup_memory_stalled_usec=<> cgroup_io_stalled_usec=<>
  被测程序的读数：sample 行取 /proc/<号>/stat（用户态与内核态 CPU 秒、缺页）、/proc/<号>/task/*/status 各线程相加（自愿与被抢的切换次数）、
    /proc/<号>/status 的 VmSwap；end 行取 getrusage(RUSAGE_CHILDREN)（只等过这一个子进程），swap_kib 写 unread。
  所在 cgroup 的压力（/proc/self/cgroup 指的那一层的 cpu.pressure、memory.pressure、io.pressure 的 some 一行 total=，微秒，累计值）：
    cgroup_cpu_waiting_usec 是这一组里有进程能跑而没轮到 CPU 的累计时长，memory 是等回收与换入，io 是等读写。
  读不到的写 unread，不写 0（.claude/rules/test-discipline.md「读不到 ≠ 读到 0」）。
退出码：被测程序的退出码；被信号停的照 shell 的写法交 128 + 信号号；被测程序起不来时照 shell：打「<程序>: <原因>」，
  找不到交 127，别的（含 Text file busy）交 126；用法错交 2。
"""
import errno
import os
import resource
import signal
import subprocess
import sys
import time

PROCESS_SAMPLE_SECONDS_BY_DEFAULT = 300.0
SAMPLE_SECONDS_VARIABLE = "E207_PROCESS_SAMPLE_SECONDS"
UNREAD = "unread"
EXIT_CODE_COMMAND_NOT_FOUND = 127
EXIT_CODE_COMMAND_NOT_EXECUTABLE = 126
EXIT_CODE_USAGE = 2
EXIT_CODE_BASE_OF_A_SIGNAL = 128
# /proc/<号>/stat 去掉「号 (名字)」之后从状态那一项起数的下标（proc(5)：minflt 是第 10 项、majflt 第 12、utime 第 14、stime 第 15）
STAT_FIELD_MINOR_FAULTS = 7
STAT_FIELD_MAJOR_FAULTS = 9
STAT_FIELD_USER_TICKS = 11
STAT_FIELD_SYSTEM_TICKS = 12


def sample_interval_seconds():
    text = os.environ.get(SAMPLE_SECONDS_VARIABLE, "")
    if not text:
        return PROCESS_SAMPLE_SECONDS_BY_DEFAULT
    try:
        value = float(text)
    except ValueError:
        value = 0.0
    if value <= 0:
        print(f"  ✗ {SAMPLE_SECONDS_VARIABLE}={text} 不是正数", file=sys.stderr)
        print(f"     → 怎么办：去掉它（用 {PROCESS_SAMPLE_SECONDS_BY_DEFAULT:.0f} 秒）或写一个正数", file=sys.stderr)
        sys.exit(EXIT_CODE_USAGE)
    return value


def read_text(path):
    try:
        with open(path, encoding="utf-8", errors="replace") as handle:
            return handle.read()
    except OSError:
        return None


def status_value(text, key):
    """/proc 的 status 里「键:  值 …」那一行的头一个数；没有这一行交 None。"""
    for line in text.splitlines():
        if line.startswith(key + ":"):
            fields = line.split()
            if len(fields) >= 2 and fields[1].isdigit():
                return int(fields[1])
    return None


def cgroup_directory():
    """这一进程所在的 cgroup v2 那一层的目录；读不到交 None。"""
    text = read_text("/proc/self/cgroup")
    if text is None:
        return None
    for line in text.splitlines():
        if line.startswith("0::"):
            return "/sys/fs/cgroup" + line[len("0::"):].strip()
    return None


def pressure_some_total_usec(directory, resource_name):
    if directory is None:
        return UNREAD
    text = read_text(os.path.join(directory, f"{resource_name}.pressure"))
    if text is None:
        return UNREAD
    for line in text.splitlines():
        if line.startswith("some "):
            for field in line.split():
                if field.startswith("total=") and field[len("total="):].isdigit():
                    return field[len("total="):]
    return UNREAD


def running_process_readings(process_id):
    """被测程序还在时从 /proc 读：交回 {读数名: 字符串}，读不到的项写 unread。"""
    readings = {name: UNREAD for name in ("user_seconds", "system_seconds", "minor_faults", "major_faults",
                                         "voluntary_switches", "involuntary_switches", "swap_kib")}
    stat = read_text(f"/proc/{process_id}/stat")
    if stat is not None and ")" in stat:
        fields = stat[stat.rindex(")") + 2:].split()
        if len(fields) > STAT_FIELD_SYSTEM_TICKS:
            ticks_per_second = os.sysconf("SC_CLK_TCK")
            readings["user_seconds"] = f"{int(fields[STAT_FIELD_USER_TICKS]) / ticks_per_second:.2f}"
            readings["system_seconds"] = f"{int(fields[STAT_FIELD_SYSTEM_TICKS]) / ticks_per_second:.2f}"
            readings["minor_faults"] = fields[STAT_FIELD_MINOR_FAULTS]
            readings["major_faults"] = fields[STAT_FIELD_MAJOR_FAULTS]
    status = read_text(f"/proc/{process_id}/status")
    if status is not None:
        swap = status_value(status, "VmSwap")
        readings["swap_kib"] = UNREAD if swap is None else str(swap)
    # 切换次数在 /proc/<号>/status 里只算主线程；测试框架把用例放在另一个线程里跑，所以各线程相加
    try:
        thread_identities = os.listdir(f"/proc/{process_id}/task")
    except OSError:
        thread_identities = []
    voluntary, involuntary, threads_read = 0, 0, 0
    for thread_identity in thread_identities:
        thread_status = read_text(f"/proc/{process_id}/task/{thread_identity}/status")
        if thread_status is None:
            continue
        thread_voluntary = status_value(thread_status, "voluntary_ctxt_switches")
        thread_involuntary = status_value(thread_status, "nonvoluntary_ctxt_switches")
        if thread_voluntary is None or thread_involuntary is None:
            continue
        voluntary += thread_voluntary
        involuntary += thread_involuntary
        threads_read += 1
    if threads_read > 0:
        readings["voluntary_switches"] = str(voluntary)
        readings["involuntary_switches"] = str(involuntary)
    return readings


def finished_child_readings():
    """被测程序退出、已等过之后：getrusage(RUSAGE_CHILDREN) 只含这一个子进程。"""
    usage = resource.getrusage(resource.RUSAGE_CHILDREN)
    return {
        "user_seconds": f"{usage.ru_utime:.2f}",
        "system_seconds": f"{usage.ru_stime:.2f}",
        "minor_faults": str(usage.ru_minflt),
        "major_faults": str(usage.ru_majflt),
        "voluntary_switches": str(usage.ru_nvcsw),
        "involuntary_switches": str(usage.ru_nivcsw),
        "swap_kib": UNREAD,
    }


def print_process_line(label, stage, started, readings, cgroup):
    fields = [f"item={label}", f"stage={stage}", f"wall_seconds={time.monotonic() - started:.0f}"]
    fields += [f"{name}={value}" for name, value in readings.items()]
    fields += [f"cgroup_{name}={pressure_some_total_usec(cgroup, resource_name)}"
               for name, resource_name in (("cpu_waiting_usec", "cpu"), ("memory_stalled_usec", "memory"), ("io_stalled_usec", "io"))]
    print("e207 process " + " ".join(fields), file=sys.stderr, flush=True)


def exit_code_of(return_code):
    return EXIT_CODE_BASE_OF_A_SIGNAL - return_code if return_code < 0 else return_code


def start_the_program(command):
    """起被测程序（自己一个进程组，停的时候整组发信号）；起不来照 shell 打一行、交退出码。"""
    try:
        return subprocess.Popen(command, start_new_session=True), None
    except OSError as error:
        reason = "Text file busy" if error.errno == errno.ETXTBSY else (error.strerror or str(error))
        print(f"{command[0]}: {reason}", file=sys.stderr, flush=True)
        return None, EXIT_CODE_COMMAND_NOT_FOUND if error.errno == errno.ENOENT else EXIT_CODE_COMMAND_NOT_EXECUTABLE


def main(argv):
    if len(argv) < 4 or argv[0] != "--label" or argv[2] != "--":
        print("用法：e207_strict_scan_steps_process_sampler.py --label <项> -- <程序> [参数…]", file=sys.stderr)
        return EXIT_CODE_USAGE
    label, command = argv[1], argv[3:]
    interval = sample_interval_seconds()
    cgroup = cgroup_directory()
    started = time.monotonic()
    program, failed_to_start = start_the_program(command)
    if program is None:
        return failed_to_start

    def forward_and_stop(signal_number, _frame):
        try:
            os.killpg(program.pid, signal_number)
        except ProcessLookupError:
            pass

    for signal_number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
        signal.signal(signal_number, forward_and_stop)
    while True:
        try:
            program.wait(timeout=interval)
            break
        except subprocess.TimeoutExpired:
            print_process_line(label, "sample", started, running_process_readings(program.pid), cgroup)
    print_process_line(label, "end", started, finished_child_readings(), cgroup)
    # 被测程序自己一个进程组：它退出之后组里不留进程（它若起过子进程而没收，整组停掉）
    try:
        os.killpg(program.pid, signal.SIGKILL)
    except ProcessLookupError:
        pass
    return exit_code_of(program.returncode)


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
