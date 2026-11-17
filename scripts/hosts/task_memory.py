#!/usr/bin/env python3
# admission: always 直接执行只拒（它是被 import 的函数库）；被 import 时每次调都现读此刻的 /proc、cgroup 文件与多机配置，上一次读到的数不替这一次作保
# run-condition: none 被 import 的函数库；直接执行只打一句用法，除了 python3 没有环境要求
"""这一次尝试的内存上限、进程树此刻的用量与峰值记录的名字：被 import 的函数库（只用标准库加同目录的 multi_host_addresses.py；
上限的取法读 scripts/gate/admission.py 的 configuration_value 与 multi_host_configuration_file，第一次用到时才按路径读进来）。
上限怎么取、一棵进程树怎么量、峰值记录叫什么名只在这一份里写，用的人 import 它，不按路径载入别的脚本来取：
  scripts/mutation/crates-mutation-rows.py（同名函数转调这里，弄坏开关在它那一边，以参数交进来）、
  scripts/experiment-apparatus/replay_row_costs.py、scripts/gate/test-targets-parallel.py（经 crates-mutation-rows.py 的同名函数）。
按路径载入整份 crates-mutation-rows.py 取这几个函数，会把它的自证与它载入的整份脚本都拉进载入方的准入闭包（scripts/gate/lib_function_units.py 的 load 规则），闭包里有拒算，按条复用整张表拒算。

用法（<仓根>/scripts/hosts 要在 sys.path 上）：
    import task_memory
    cap, text = task_memory.task_memory_cap_bytes(os.environ)        # (字节或 None, 说明)
    usage = task_memory.process_tree_usage(pid)                      # 字节；量不了交 None
    name = task_memory.new_worker_peak_record_name()
    python3 scripts/hosts/task_memory.py                             直接执行只打用法，退 2

内存上限（task_memory_cap_bytes，次序）：
  ① 统一任务服务交的环境变量 TASK_SERVER_MEMORY_CAP_BYTES（这一次尝试的上限，字节十进制整数；写法不对的说明里点名、改走②）；
  ② 仓根多机配置 multi-host.env 的 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（值是 <地址>=<systemd 写法> 的若干项；读法用 admission.py 的 configuration_value，
     这一台照多机配置判法 --emit-addresses 认：multi_host_addresses.resolve，整份配置判过才给地址）；configuration_path 给了读它，不给按 test_runs_multi_host_configuration 找；
     文件不在、键没写、认不出这一台、写法不认、读不了都往下走，说明里点名；
  ③ 自己 scope 的 memory.max（scope_memory_max：/proc/self/cgroup 里 `0::<路径>` 那一行接 /sys/fs/cgroup 再接 /memory.max；只读自己这一层，不往上找父 cgroup）。
  memory.max 是有限值就是硬顶（超了整组被杀）：① ② 取到的值比它大，取 memory.max（两者取小）；① ② 都没有，取 memory.max 本身；
  memory.max 是 max（不限）或读不到，不收窄，照 ① ② 的值，① ② 也没有就交 None 与原因，调用方照线程数开。
  cgroup_listing、cgroup_root 是默认值（/proc/self/cgroup、/sys/fs/cgroup）时，环境里的 GATE_MUTATION_CGROUP_LISTING、GATE_MUTATION_CGROUP_ROOT 设了就换成它们（只给自证用，生产不设）。
  hard_cap_ignored 为真：① ② 取到了就不拿 memory.max 当硬顶（crates-mutation-rows.py 的弄坏开关 memory-max-not-hard-cap 交进来）。
  说明写各档读到的值、取了哪一个、为什么。

这一台能给的内存（host_slice_cap_bytes）：只读多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项（读法同上 ②），不读服务交的变量、不拿 memory.max 收窄；
  门禁 crates-mutation-replay 按它与每个工作进程的实测峰值定开几个工作进程（crates-mutation-rows.py 文件头「内存上限」），这一次拿到的上限装不下就退 250 由服务放大。

进程树用量（process_tree_usage）：root_pid 连同它整棵进程树此刻每个进程 /proc/<pid>/status 的 RssAnon + RssShmem + VmPTE 加总；子进程顺 /proc/<pid>/task/<tid>/children 往下找，
  读不到的（刚退出）不算。root_pid 自己读不到、或它主线程的 children 文件不在（这台内核没有这一项）交 None。self_only 为真：只算 root_pid 自己
  （crates-mutation-rows.py 的弄坏开关 worker-tree-self-only 交进来）。

峰值记录的名字（new_worker_peak_record_name）：起写的时刻纳秒（20 位，名字排序就是先后）、进程号、8 位随机字符（两台的进程号会撞）。
"""
import importlib.util
import os
import random
import re
import sys
import time

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
REPOSITORY = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
sys.path.insert(0, SCRIPT_DIRECTORY)
import multi_host_addresses  # noqa: E402

TASK_MEMORY_CAP_VARIABLE = "TASK_SERVER_MEMORY_CAP_BYTES"
SLICE_MEMORY_CAP_KEY = "SLICE_MEMORY_CAP_BY_HOST"   # 仓根多机配置 multi-host.env 里各台 slice 的总上限（<地址>=<systemd 写法> 的若干项，取这一台那一项）
CGROUP_LISTING = "/proc/self/cgroup"
CGROUP_ROOT = "/sys/fs/cgroup"
CGROUP_LISTING_OVERRIDE_VARIABLE = "GATE_MUTATION_CGROUP_LISTING"   # 只给自证换 CGROUP_LISTING，生产不设
CGROUP_ROOT_OVERRIDE_VARIABLE = "GATE_MUTATION_CGROUP_ROOT"         # 只给自证换 CGROUP_ROOT，生产不设
SIZE_TEXT = re.compile(r"^([1-9][0-9]*)([KMGT])$")             # systemd 的大小写法，1024 进制
SIZE_UNIT_BYTES = {"K": 1024, "M": 1024 ** 2, "G": 1024 ** 3, "T": 1024 ** 4}
RECORD_NAME_ALPHABET = "abcdefghijklmnopqrstuvwxyz0123456789"
PROCESS_USAGE_FIELDS = ("RssAnon", "RssShmem", "VmPTE")
_ADMISSION_MODULE = []


def format_size(byte_count):
    for unit, size in (("GiB", 1 << 30), ("MiB", 1 << 20), ("KiB", 1 << 10)):
        if byte_count >= size:
            return f"{byte_count / size:.1f} {unit}"
    return f"{byte_count} B"


def bytes_of_size(text):
    """systemd 写法的大小（正整数加 K / M / G / T，1024 进制）→ 字节；别的写法交 None。"""
    matched = SIZE_TEXT.match(text.strip())
    return int(matched.group(1)) * SIZE_UNIT_BYTES[matched.group(2)] if matched else None


def admission_module():
    """scripts/gate/admission.py（多机配置文件的找法、配置里一个键的读法都用它的），第一次用到时才读进来。"""
    if not _ADMISSION_MODULE:
        specification = importlib.util.spec_from_file_location("admission_for_task_memory", os.path.join(SCRIPT_DIRECTORY, "..", "gate", "admission.py"))
        module = importlib.util.module_from_spec(specification)
        specification.loader.exec_module(module)
        _ADMISSION_MODULE.append(module)
    return _ADMISSION_MODULE[0]


def test_runs_multi_host_configuration(environment):
    """真跑测试的格交给测试的多机配置：(绝对路径, 文件在不在)。
    调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它；没设取 GATE_MUTATION_MULTI_HOST_CONFIG（自证用），再没设照 admission.py 的 multi_host_configuration_file
    取本脚本所在仓的那一份（git common-dir 的上一层下的 multi-host.env）。相对路径按本进程的当前目录取成绝对路径：测试在源码副本里起，相对路径到那里就指错了。"""
    path = (environment.get("SINGLEFS_MULTI_HOST_CONFIG", "").strip() or environment.get("GATE_MUTATION_MULTI_HOST_CONFIG", "").strip()
            or admission_module().multi_host_configuration_file(REPOSITORY, {}))
    path = os.path.abspath(path)
    return path, os.path.isfile(path)


def scope_memory_max(cgroup_listing, cgroup_root):
    """自己 scope 的 memory.max（cgroup v2：cgroup_listing 即 /proc/self/cgroup 里 `0::<路径>` 那一行接 cgroup_root 再接 /memory.max；只读自己这一层，不往上找父 cgroup）：
    (字节或 None, 说明)。有限值交 (字节, 「自己 scope 的 <路径>」)；值是 max（不限）、文件读不到、没有 v2 的那一行、写法不认交 (None, 原因)。"""
    try:
        with open(cgroup_listing, encoding="utf-8") as handle:
            listing = handle.read().splitlines()
    except OSError as error:
        return None, f"读不到 {cgroup_listing}（{type(error).__name__}：{error}）"
    scope = next((line[3:] for line in listing if line.startswith("0::")), None)
    if scope is None:
        return None, f"{cgroup_listing} 里没有 cgroup v2 的那一行（0::<路径>）"
    memory_max = os.path.join(cgroup_root, scope.lstrip("/"), "memory.max")
    try:
        with open(memory_max, encoding="utf-8") as handle:
            text = handle.read().strip()
    except OSError as error:
        return None, f"读不到 {memory_max}（{type(error).__name__}：{error}）"
    if text == "max":
        return None, f"{memory_max} 是 max（不限）"
    if not re.fullmatch(r"[1-9][0-9]*", text):
        return None, f"{memory_max} 写着「{text}」，不是 max 也不是正整数"
    return int(text), f"自己 scope 的 {memory_max}"


def this_host_slice_cap_text(configuration, by_host_text):
    """多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项：(值或 None, 这一台的地址或 None, 取不到的原因)。
    这一台照多机配置判法 --emit-addresses 认（multi_host_addresses.resolve：整份配置判过才给地址）；每项从第一个 = 切。"""
    try:
        address = multi_host_addresses.resolve(configuration).this
    except multi_host_addresses.AddressesUnavailable as error:
        return None, None, f"认不出这一台（{error}）"
    for item in by_host_text.split():
        item_address, _separator, value = item.partition("=")
        if item_address == address:
            return value, address, ""
    return None, address, f"没有这一台 {address} 那一项"


def host_slice_cap_bytes(environment, configuration_path=None):
    """这一台 slice 能给的内存（文件头「这一台能给的内存」）：多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项，(字节或 None, 说明或取不到的原因)。
    不读服务交的 TASK_SERVER_MEMORY_CAP_BYTES，也不拿自己 scope 的 memory.max 收窄：那两样是这一次尝试拿到的上限，不是这一台能给多少。"""
    configuration = configuration_path if configuration_path is not None else test_runs_multi_host_configuration(environment)[0]
    try:
        by_host_text = admission_module().configuration_value(configuration, SLICE_MEMORY_CAP_KEY)
    except Exception as error:   # InputManifestError：文件在而读不了；不吞，写进说明
        return None, f"多机配置 {configuration} 的 {SLICE_MEMORY_CAP_KEY} 读不了（{error}）"
    if not by_host_text:
        return None, f"多机配置 {configuration} 没写 {SLICE_MEMORY_CAP_KEY}"
    slice_text, this_address, unavailable = this_host_slice_cap_text(configuration, by_host_text)
    if not slice_text:
        return None, f"多机配置 {configuration} 的 {SLICE_MEMORY_CAP_KEY} 取不到这一台那一项：{unavailable}"
    slice_cap = bytes_of_size(slice_text)
    if slice_cap is None:
        return None, f"多机配置 {configuration} 的 {SLICE_MEMORY_CAP_KEY} 里这一台 {this_address} 那一项写成「{slice_text}」，不是正整数加 K / M / G / T"
    return slice_cap, f"多机配置 {configuration} 的 {SLICE_MEMORY_CAP_KEY} 里这一台 {this_address} 那一项={slice_text}"


def task_memory_cap_bytes(environment, cgroup_listing=CGROUP_LISTING, cgroup_root=CGROUP_ROOT, configuration_path=None, hard_cap_ignored=False):
    """这一条命令拿到的内存上限：(字节或 None, 说明或没有这一项的原因)。取法与次序见文件头「内存上限」。"""
    if cgroup_listing == CGROUP_LISTING:
        cgroup_listing = environment.get(CGROUP_LISTING_OVERRIDE_VARIABLE, "").strip() or cgroup_listing
    if cgroup_root == CGROUP_ROOT:
        cgroup_root = environment.get(CGROUP_ROOT_OVERRIDE_VARIABLE, "").strip() or cgroup_root
    notes, readings, chosen = [], [], None   # chosen：①②里取到的 (字节, 原话)
    given = environment.get(TASK_MEMORY_CAP_VARIABLE, "").strip()
    if given:
        if re.fullmatch(r"[1-9][0-9]*", given):
            chosen = (int(given), f"服务交的 {TASK_MEMORY_CAP_VARIABLE}={given}")
            readings.append(f"{TASK_MEMORY_CAP_VARIABLE}={given}（{format_size(int(given))}）")
        else:
            notes.append(f"{TASK_MEMORY_CAP_VARIABLE} 写成「{given}」，不是正整数字节数，往下读多机配置的 {SLICE_MEMORY_CAP_KEY}")
            readings.append(f"{TASK_MEMORY_CAP_VARIABLE} 写法不认")
    else:
        readings.append(f"{TASK_MEMORY_CAP_VARIABLE} 没给")
    configuration = configuration_path if configuration_path is not None else test_runs_multi_host_configuration(environment)[0]
    try:
        by_host_text = admission_module().configuration_value(configuration, SLICE_MEMORY_CAP_KEY)
    except Exception as error:   # InputManifestError：文件在而读不了；不吞，写进说明、往下走
        if chosen is None:
            notes.append(str(error))
        readings.append(f"多机配置 {SLICE_MEMORY_CAP_KEY} 读不了")
    else:
        slice_text, this_address, unavailable = this_host_slice_cap_text(configuration, by_host_text) if by_host_text else (None, None, "")
        label = f"{SLICE_MEMORY_CAP_KEY} 里这一台 {this_address} 那一项"
        slice_cap = bytes_of_size(slice_text) if slice_text else None
        if slice_cap is not None:
            readings.append(f"多机配置 {label}={slice_text}（{format_size(slice_cap)}）")
            if chosen is None:
                chosen = (slice_cap, f"多机配置 {configuration} 的 {label}={slice_text}")
        elif slice_text:
            if chosen is None:
                notes.append(f"多机配置 {configuration} 的 {label}写成「{slice_text}」，不是正整数加 K / M / G / T")
            readings.append(f"多机配置 {label}写法不认")
        elif by_host_text:
            if chosen is None:
                notes.append(f"多机配置 {configuration} 的 {SLICE_MEMORY_CAP_KEY} 取不到这一台那一项：{unavailable}")
            readings.append(f"多机配置 {SLICE_MEMORY_CAP_KEY} 取不到这一台那一项")
        else:
            readings.append(f"多机配置没写 {SLICE_MEMORY_CAP_KEY}")
    scope_cap, scope_text = scope_memory_max(cgroup_listing, cgroup_root)
    readings.append(f"scope 的 memory.max={scope_cap}（{format_size(scope_cap)}）" if scope_cap is not None else f"scope 的 memory.max 没有硬顶（{scope_text}）")
    if chosen is None:
        # ① ② 都没有：memory.max 是有限值就取它（它自己就是这一条拿到的上限），不然这一项不收
        return scope_cap, "；".join(notes + [scope_text])
    readings_text = "各档读到：" + "、".join(readings)
    hard_cap = None if hard_cap_ignored else scope_cap
    if hard_cap is not None and hard_cap < chosen[0]:
        return scope_cap, "；".join(notes + [f"{scope_text}（硬顶：超了整组被杀）；比它大的 {chosen[1]}（{format_size(chosen[0])}）不取，取两者里小的", readings_text])
    why = ("scope 的 memory.max 没有硬顶可比，取服务交的变量，没有就取多机配置的" if scope_cap is None
           else "scope 的 memory.max 不比它小、不收窄，取服务交的变量，没有就取多机配置的")
    return chosen[0], "；".join(notes + [chosen[1], readings_text, why])


def new_worker_peak_record_name():
    """一份峰值记录的名字：起写的时刻纳秒（20 位，名字排序就是先后）、进程号、8 位随机字符（两台的进程号会撞）。"""
    chooser = random.SystemRandom()
    return f"{time.time_ns():020d}-{os.getpid()}-{''.join(chooser.choice(RECORD_NAME_ALPHABET) for _ in range(8))}"


def process_status_bytes(pid, field_names):
    """/proc/<pid>/status 里 field_names 那几行加总（kB 换成字节；内核线程与僵尸进程没有这几行，算 0）。读不到（进程刚退出）交 None。"""
    try:
        with open(f"/proc/{pid}/status", encoding="utf-8", errors="replace") as handle:
            text = handle.read()
    except OSError:
        return None
    total = 0
    for line in text.splitlines():
        name, _separator, value = line.partition(":")
        words = value.split()
        if name in field_names and words and words[0].isdigit():
            total += int(words[0]) * 1024
    return total


def process_usage_bytes(pid):
    """一个进程此刻的用量：/proc/<pid>/status 的 RssAnon + RssShmem + VmPTE。读不到（进程刚退出）交 None。"""
    return process_status_bytes(pid, PROCESS_USAGE_FIELDS)


def process_children(pid):
    """(pid 各线程的子进程号, 主线程的 children 文件在不在)：顺 /proc/<pid>/task/<tid>/children 读，线程目录列不了、别的线程读不了的（刚退出）不算。"""
    children, main_thread_file_present = [], True
    try:
        threads = os.listdir(f"/proc/{pid}/task")
    except OSError:
        return children, main_thread_file_present
    for thread in threads:
        try:
            with open(f"/proc/{pid}/task/{thread}/children", encoding="ascii") as handle:
                children += [int(word) for word in handle.read().split()]
        except FileNotFoundError:
            main_thread_file_present = main_thread_file_present and thread != str(pid)
        except OSError:
            continue
    return children, main_thread_file_present


def process_tree_usage(root_pid, self_only=False):
    """root_pid 连同它整棵进程树此刻的用量（文件头「进程树用量」）：字节；root_pid 自己读不到、或它主线程的 children 文件不在交 None。"""
    total, pending, seen = 0, [root_pid], set()
    while pending:
        pid = pending.pop()
        if pid in seen:
            continue
        seen.add(pid)
        usage = process_usage_bytes(pid)
        if usage is None:
            if pid == root_pid:
                return None
            continue
        total += usage
        if self_only:
            break
        children, main_thread_file_present = process_children(pid)
        if pid == root_pid and not main_thread_file_present:
            return None
        pending += children
    return total


if __name__ == "__main__":
    sys.dont_write_bytecode = True
    sys.path.insert(0, os.path.join(SCRIPT_DIRECTORY, "..", "..", ".claude", "scripts"))
    from project_preflight import preflight
    preflight(__file__)
    print("  ✗ task_memory.py 是被 import 的函数库，不单独跑", file=sys.stderr)
    print("     → 怎么办：import 它再调 task_memory_cap_bytes、process_tree_usage、new_worker_peak_record_name（文件头「用法」）", file=sys.stderr)
    sys.exit(2)
