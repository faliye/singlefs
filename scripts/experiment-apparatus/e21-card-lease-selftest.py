#!/usr/bin/env python3
# admission: always 判的是此刻三份脚本与 e21_card_lease.py 的样子，上一次的结论不替这一次作保
# run-condition: none 用假领卡客户端、假 torch、假 nvidia-smi 在临时目录里的树拷贝上跑，要的只有 python3 与 bash（准入判据的子进程），不导入真 torch、不起真卡
"""E21 三份 torch 脚本（e21-compute.py、e21-prove-gpu.py、e21-transfer.py）按需领卡的自证：python3 scripts/experiment-apparatus/e21-card-lease-selftest.py --selftest

做法：临时目录里放一棵样板树（三份脚本与 e21_card_lease.py 的拷贝，.claude 指回这棵树的 .claude，task-server/task 是假领卡客户端），PYTHONPATH 最前面放一个假 torch
（不起 CUDA；每一步 CUDA 操作往事件记录里记下进程号、CUDA_VISIBLE_DEVICES、CUDA_DEVICE_ORDER、GPU_CARDS_BY_PRIORITY），PATH 最前面放假 nvidia-smi（记下 --id）。
每份脚本在九种局面下各跑一遍（二十七趟，并行，每趟自己的目录与事件记录）：领到卡并跑完、跑到一半出错、跑到一半被 TERM 打断、没有领卡通道、领卡被拒、领卡退 0 却没打卡串、
还卡没成、环境里原有 GPU_CARDS_BY_PRIORITY、领到的卡串认不出序号。假客户端把「take」「return <卡串> alive=<还卡时事件记录里 CUDA 进程还活着的进程号>」按先后记进同一份事件记录，
按先后核。判的格（失败行「判错的样本：<格名>（」）：
  header-declares-on-demand                          三份脚本文件头前 60 行里写 `# task-server-card-lease: on-demand`（服务只认这一句）
  card-taken-once-and-returned-once                  领一次还一次，还的是领到的那一串，产物末尾有 name=done
  card-returned-after-the-card-process-exits         还卡发生在所有 CUDA 操作之后，还卡那一刻上卡的进程已经退了
  only-the-leased-card-visible                       上卡的进程里 CUDA_VISIBLE_DEVICES 只有领到的那个序号、CUDA_DEVICE_ORDER 是 PCI_BUS_ID；prove-gpu 点 nvidia-smi 的也是那个序号
  card-returned-after-an-error                       上卡那一段出错也还卡
  card-returned-after-an-interrupt                   被 TERM 打断退 143，也还卡
  no-channel-exits-78                                没有领卡通道退 78、标准输出空、写出路、没领卡也没碰 CUDA
  refused-lease-runs-nothing                         领卡被拒退 1、标准输出空、没碰 CUDA、没还卡
  empty-card-string-counts-as-refused                退 0 却没打卡串同样退 1、标准输出空、没碰 CUDA
  unparseable-card-string-is-returned-and-fails      卡串认不出序号退 1、标准输出空、没碰 CUDA、领到的那一串照还
  failed-return-only-warns                           还卡没成退出码不变，只多一行「!」
  inherited-card-variable-cleared                    环境里原有的 GPU_CARDS_BY_PRIORITY 到不了上卡的进程
弄坏开关 E21_CARD_LEASE_BREAK=<项>（逗号分隔）弄坏的是 e21_card_lease.py 的领还卡，只有 declaration-removed 由本文件自己弄坏。
库的文件头与本文件文件头各写同样的一张表，两张表与 SWITCH_CELLS 由 switch_table_problems 核成逐条一致；每条开关一行名字与弄坏哪一处，下一行写打开后自证红的格
（失败行「判错的样本：<格名>（」里的格名）：
  E21_CARD_LEASE_BREAK=declaration-removed  自证拷出来的三份脚本里去掉文件头 `# task-server-card-lease: on-demand` 那一行（由自证自己弄坏，库不读）
      header-declares-on-demand
  E21_CARD_LEASE_BREAK=card-return-skipped  不调还卡
      card-taken-once-and-returned-once
  E21_CARD_LEASE_BREAK=card-returned-twice  还卡调两回
      card-taken-once-and-returned-once
  E21_CARD_LEASE_BREAK=card-returned-before-the-card-process-exits  还卡挪到等子进程之前（子进程还在占卡就还）
      card-returned-after-the-card-process-exits
  E21_CARD_LEASE_BREAK=card-kept-after-error  子进程退非 0 时不还
      card-returned-after-an-error
  E21_CARD_LEASE_BREAK=card-kept-after-interrupt  被 TERM 打断时不还
      card-returned-after-an-interrupt
  E21_CARD_LEASE_BREAK=channel-unchecked  不核领卡通道，直接去领
      no-channel-exits-78
  E21_CARD_LEASE_BREAK=refusal-ignored  领卡被拒也照起子进程
      refused-lease-runs-nothing
  E21_CARD_LEASE_BREAK=empty-card-accepted  退 0 却没打卡串的也当领到了
      empty-card-string-counts-as-refused
  E21_CARD_LEASE_BREAK=unparsed-card-run  领到的卡串认不出序号也照起子进程
      unparseable-card-string-is-returned-and-fails
  E21_CARD_LEASE_BREAK=all-cards-visible  不设 CUDA_VISIBLE_DEVICES（所有卡都看得到）
      only-the-leased-card-visible
  E21_CARD_LEASE_BREAK=device-order-unset  不设 CUDA_DEVICE_ORDER（CUDA 序号与 nvidia-smi 序号不是同一套）
      only-the-leased-card-visible
  E21_CARD_LEASE_BREAK=inherited-cards-kept  不清环境里原有的 GPU_CARDS_BY_PRIORITY
      inherited-card-variable-cleared
  E21_CARD_LEASE_BREAK=return-failure-changes-exit  还卡没成时把退出码改成 1
      failed-return-only-warns
  E21_CARD_LEASE_BREAK=return-failure-unreported  还卡没成不打「!」那一行
      failed-return-only-warns
本文件在每份样本趟的环境里设 E21_CARD_LEASE_SELFTEST_MARK（库只在有它时认开关）。环境里没有开关的 --selftest 判完九种局面之后，逐个开关设上、另起子进程跑 --selftest-inner
（并行，至多 SWITCH_ROUND_PARALLEL 路，各自一棵临时树），要求退 1、输出里有「判错的样本：<点名的格>（」。
外面设了 E21_CARD_LEASE_BREAK 再起 --selftest（或别的参数）在一切之前退 2、点名变量只在自证里认；
单跑一条开关：E21_CARD_LEASE_BREAK=<项> python3 scripts/experiment-apparatus/e21-card-lease-selftest.py --selftest-inner（判错退 1，居然通过退 0）。

退出码：0 全部判对（开关轮逐个红在点名的格）；1 有格判错，或有开关打开后没红在点名的格；2 用法错、平常跑设了弄坏开关（含单独起 --selftest）、读不到每台默认重活线程数；
--selftest-inner（只给开关轮用，环境里要设一个开关）：1 判错（开关弄坏了东西），0 居然通过。
"""
import concurrent.futures
import os
import re
import shutil
import subprocess
import sys
import tempfile
import time
from typing import Callable, NamedTuple
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import e21_card_lease  # noqa: E402

APPARATUS_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
SCRIPT_NAMES = ("e21-compute.py", "e21-prove-gpu.py", "e21-transfer.py")
PROVE_SCRIPT_NAME = "e21-prove-gpu.py"
LIBRARY_NAME = "e21_card_lease.py"
BREAK_VARIABLE = "E21_CARD_LEASE_BREAK"   # 与库里的同名常量相同（switch_table_problems 核）
INNER_ARGUMENT = e21_card_lease.INNER_ARGUMENT   # 开关轮起子进程带的参数：库的守卫只放行它
LEASED_CARD = "198.51.100.7:3:8G"   # 假客户端交的卡串：序号 3
LEASED_INDEX = "3"
INHERITED_CARDS = "198.51.100.9:7:8G"
GARBAGE_CARDS = "not-a-card"
RUN_TIMEOUT_SECONDS = 120   # 最慢的一趟（prove-gpu 空载加满载采样）约 10 秒
HANG_WAIT_SECONDS = 60
SWITCH_ROUND_PARALLEL = 8
SWITCH_RUN_WORKERS = 6
DECLARATION_MARK = "# task-server-card-lease:"   # task-server/task_server_lib.py 的 DECLARED_CARD_LEASE_MARK
DECLARATION_VALUE = "on-demand"   # 同文件的 CARD_LEASE_ON_DEMAND
DECLARATION_HEADER_LINES = 60   # 同文件的 DECLARED_HEADER_LINES：服务只读文件头这么多行
DONE_LINE = "E21RESULT name=done"

# 开关名 → 打开后必须红在的格。库与本文件两处文件头开关表逐条与它对得上（switch_table_problems 核）。
SWITCH_CELLS = {
    "declaration-removed": "header-declares-on-demand",
    "card-return-skipped": "card-taken-once-and-returned-once",
    "card-returned-twice": "card-taken-once-and-returned-once",
    "card-returned-before-the-card-process-exits": "card-returned-after-the-card-process-exits",
    "card-kept-after-error": "card-returned-after-an-error",
    "card-kept-after-interrupt": "card-returned-after-an-interrupt",
    "channel-unchecked": "no-channel-exits-78",
    "refusal-ignored": "refused-lease-runs-nothing",
    "empty-card-accepted": "empty-card-string-counts-as-refused",
    "unparsed-card-run": "unparseable-card-string-is-returned-and-fails",
    "all-cards-visible": "only-the-leased-card-visible",
    "device-order-unset": "only-the-leased-card-visible",
    "inherited-cards-kept": "inherited-card-variable-cleared",
    "return-failure-changes-exit": "failed-return-only-warns",
    "return-failure-unreported": "failed-return-only-warns",
}

STUB_TORCH = r'''
import os, sys, time, types

LOG_PATH = os.environ["E21_STUB_LOG"]
BUSY_PATH = os.environ["E21_STUB_BUSY_FILE"]
HANG_PATH = os.environ["E21_STUB_HANG_FILE"]
MODE = os.environ.get("E21_STUB_MODE", "ok")
PROGRAM = os.path.basename(sys.argv[0])
FAULT_OPERATIONS = ("mem_get_info", "randint", "empty")
PINNED_COPY_SECONDS, PAGEABLE_COPY_SECONDS = 0.01, 0.04
CARD_XOR_SECONDS, HOST_XOR_SECONDS = 0.0005, 0.05
BUSY_REFRESH_SECONDS = 0.1
__version__ = "stub"
int64, uint8 = "int64", "uint8"
noted_operations = set()
last_busy_refresh = [0.0]


def note(text):
    with open(LOG_PATH, "a", encoding="utf-8") as handle:
        handle.write(text + "\n")


def touch(operation):
    if operation not in noted_operations:
        noted_operations.add(operation)
        note(f"cuda pid={os.getpid()} program={PROGRAM} visible={os.environ.get('CUDA_VISIBLE_DEVICES', '-')} "
             f"order={os.environ.get('CUDA_DEVICE_ORDER', '-')} cards={os.environ.get('GPU_CARDS_BY_PRIORITY', '-')} operation={operation}")
    if operation not in FAULT_OPERATIONS:
        return
    if MODE == "raise":
        raise RuntimeError("stub CUDA error")
    if MODE == "hang":
        parent = os.getppid()
        with open(HANG_PATH, "w", encoding="utf-8") as handle:
            handle.write(str(os.getpid()))
        while os.getppid() == parent:
            time.sleep(0.2)
        sys.exit(3)


class Tensor:
    def __init__(self, device="cpu", pinned=False):
        self.device = str(device)
        self.pinned = pinned

    def copy_(self, other):
        if self.device.startswith("cuda") or other.device.startswith("cuda"):
            touch("copy")
        time.sleep(PINNED_COPY_SECONDS if (self.pinned or other.pinned) else PAGEABLE_COPY_SECONDS)
        return self


def randint(low, high, size, dtype=None, device="cpu"):
    if str(device).startswith("cuda"):
        touch("randint")
    return Tensor(device)


def zeros_like(tensor):
    if tensor.device.startswith("cuda"):
        touch("zeros_like")
    return Tensor(tensor.device)


def empty(size, dtype=None, pin_memory=False, device="cpu"):
    if str(device).startswith("cuda"):
        touch("empty")
    return Tensor(device, pin_memory)


def bitwise_xor(left, right, out=None):
    if left.device.startswith("cuda"):
        touch("bitwise_xor")
        time.sleep(CARD_XOR_SECONDS)
        if time.time() - last_busy_refresh[0] > BUSY_REFRESH_SECONDS:
            last_busy_refresh[0] = time.time()
            with open(BUSY_PATH, "w", encoding="utf-8") as handle:
                handle.write("busy")
    else:
        time.sleep(HOST_XOR_SECONDS)
    return out


def is_available():
    touch("is_available")
    return True


def mem_get_info(device):
    touch("mem_get_info")
    return (20 << 30, 24 << 30)


def get_device_name(device):
    touch("get_device_name")
    return "stub-card"


def synchronize(device=None):
    touch("synchronize")


def empty_cache():
    touch("empty_cache")


cuda = types.SimpleNamespace(is_available=is_available, mem_get_info=mem_get_info, get_device_name=get_device_name,
                             synchronize=synchronize, empty_cache=empty_cache)
'''

FAKE_NVIDIA_SMI = r'''#!/usr/bin/env python3
import os, sys, time

identifier = next((word[len("--id="):] for word in sys.argv[1:] if word.startswith("--id=")), "-")
with open(os.environ["E21_STUB_LOG"], "a", encoding="utf-8") as handle:
    handle.write(f"smi id={identifier}\n")
try:
    busy = time.time() - os.stat(os.environ["E21_STUB_BUSY_FILE"]).st_mtime < 0.5
except OSError:
    busy = False
print("95, 4096" if busy else "0, 512")
'''

FAKE_CLIENT = r'''#!/usr/bin/env python3
import os, sys

LOG_PATH = os.environ["E21_STUB_LOG"]


def note(text):
    with open(LOG_PATH, "a", encoding="utf-8") as handle:
        handle.write(text + "\n")


def alive_card_processes():
    alive = []
    with open(LOG_PATH, encoding="utf-8") as handle:
        for line in handle:
            if line.startswith("cuda pid="):
                process_id = line.split()[1].split("=", 1)[1]
                if os.path.exists(f"/proc/{process_id}") and process_id not in alive:
                    alive.append(process_id)
    return alive


arguments = sys.argv[1:]
if arguments == ["lease", "take"]:
    note("take")
    behaviour = os.environ.get("E21_FAKE_TAKE", "grant")
    if behaviour == "refuse":
        sys.stderr.write("  ✗ 服务没给这一回领卡：自证演的拒\n")
        sys.exit(3)
    if behaviour == "empty":
        sys.exit(0)
    print(os.environ["E21_FAKE_CARDS"])
elif arguments[:2] == ["lease", "return"]:
    alive = alive_card_processes()
    note(f"return cards={' '.join(arguments[2:])} alive={','.join(alive) or '-'}")
    if os.environ.get("E21_FAKE_RETURN") == "unanswered":
        sys.stderr.write("  ✗ 还卡的请求 120 秒没有回执（自证演的）\n")
        sys.exit(1)
else:
    sys.stderr.write(f"假领卡客户端只认 lease take、lease return，收到：{' '.join(arguments)}\n")
    sys.exit(2)
'''


class Situation(NamedTuple):
    name: str
    environment: dict[str, str]   # 在样本趟的环境上另设的变量
    has_channel: bool = True
    interrupt: bool = False   # 等假 torch 挂住的标记出现，再给脚本的进程发 TERM


SITUATIONS = (
    Situation("green", {}),
    Situation("error", {"E21_STUB_MODE": "raise"}),
    Situation("interrupt", {"E21_STUB_MODE": "hang"}, interrupt=True),
    Situation("no-channel", {}, has_channel=False),
    Situation("refused", {"E21_FAKE_TAKE": "refuse"}),
    Situation("empty", {"E21_FAKE_TAKE": "empty"}),
    Situation("return-failure", {"E21_FAKE_RETURN": "unanswered"}),
    Situation("inherited-variable", {e21_card_lease.CARDS_VARIABLE: INHERITED_CARDS}),
    Situation("garbage-card", {"E21_FAKE_CARDS": GARBAGE_CARDS}),
)


class Outcome(NamedTuple):
    exit_code: int
    standard_output: str
    standard_error: str
    events: list[str]   # 事件记录逐行（假客户端、假 torch、假 nvidia-smi 按先后追加）
    problem: str   # 这一趟没跑成样本（超时、没等到挂住的标记、上卡的进程没退干净）；空是跑成了


def write_executable(path: str, text: str) -> None:
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    os.chmod(path, 0o755)


def switch_is_on(name: str) -> bool:
    return name in os.environ.get(BREAK_VARIABLE, "").split(",")


def build_tree(base: str) -> str:
    """样板树：三份脚本与库的拷贝（declaration-removed 开着时去掉文件头的按需领卡声明）、.claude 指回真树、假领卡客户端。"""
    tree = os.path.join(base, "tree")
    apparatus = os.path.join(tree, "scripts", "experiment-apparatus")
    os.makedirs(apparatus)
    for name in SCRIPT_NAMES + (LIBRARY_NAME,):
        with open(os.path.join(APPARATUS_DIRECTORY, name), encoding="utf-8") as source:
            lines = source.readlines()
        if switch_is_on("declaration-removed"):
            lines = [line for line in lines if not line.startswith(DECLARATION_MARK)]
        with open(os.path.join(apparatus, name), "w", encoding="utf-8") as target:
            target.writelines(lines)
    os.symlink(os.path.join(e21_card_lease.REPOSITORY_ROOT, ".claude"), os.path.join(tree, ".claude"))
    write_executable(os.path.join(tree, "task-server", "task"), FAKE_CLIENT)
    write_executable(os.path.join(base, "bin", "nvidia-smi"), FAKE_NVIDIA_SMI)
    stub_directory = os.path.join(base, "stubs", "torch")
    os.makedirs(stub_directory)
    with open(os.path.join(stub_directory, "__init__.py"), "w", encoding="utf-8") as handle:
        handle.write(STUB_TORCH)
    os.makedirs(os.path.join(base, "channel"))
    return tree


def sample_environment(base: str, directory: str, situation: Situation) -> dict[str, str]:
    environment = {name: value for name, value in os.environ.items()
                   if not name.startswith(("E21_", "CUDA_", "TASK_SERVER_CARD_LEASE")) and name != e21_card_lease.CARDS_VARIABLE}
    if BREAK_VARIABLE in os.environ:
        environment[BREAK_VARIABLE] = os.environ[BREAK_VARIABLE]
    environment.update({
        "PATH": os.path.join(base, "bin") + os.pathsep + os.environ.get("PATH", ""),
        "PYTHONPATH": os.path.join(base, "stubs"),
        "PYTHONDONTWRITEBYTECODE": "1",
        "E21_STUB_LOG": os.path.join(directory, "events.log"),
        "E21_STUB_BUSY_FILE": os.path.join(directory, "busy"),
        "E21_STUB_HANG_FILE": os.path.join(directory, "hanging"),
        "E21_FAKE_CARDS": LEASED_CARD,
        e21_card_lease.SELFTEST_MARK_VARIABLE: e21_card_lease.SELFTEST_MARK_CONTENT,
    })
    if situation.has_channel:
        environment[e21_card_lease.CHANNEL_DIRECTORY_VARIABLE] = os.path.join(base, "channel")
        environment[e21_card_lease.CHANNEL_TOKEN_VARIABLE] = "selftest-token"
    environment.update(situation.environment)
    return environment


def card_process_ids(events: list[str], script: str) -> list[str]:
    return [line.split()[1].split("=", 1)[1] for line in events if line.startswith("cuda pid=") and f" program={script} " in line]


def run_script(tree: str, base: str, script: str, situation: Situation) -> Outcome:
    """一份脚本在一种局面下跑一趟：自己的目录、自己的事件记录；只给自己起的那个进程发信号（TERM、超时的 KILL）。"""
    directory = os.path.join(base, f"{situation.name}-{script}")
    os.makedirs(directory)
    log_path = os.path.join(directory, "events.log")
    open(log_path, "w", encoding="utf-8").close()
    problem = ""
    with open(os.path.join(directory, "stdout.txt"), "w", encoding="utf-8") as standard_output, \
            open(os.path.join(directory, "stderr.txt"), "w", encoding="utf-8") as standard_error:
        process = subprocess.Popen([sys.executable, os.path.join(tree, "scripts", "experiment-apparatus", script)], cwd=directory,
                                   env=sample_environment(base, directory, situation), stdin=subprocess.DEVNULL, stdout=standard_output,
                                   stderr=standard_error, start_new_session=True)
        try:
            if situation.interrupt:
                deadline = time.time() + HANG_WAIT_SECONDS
                while not os.path.exists(os.path.join(directory, "hanging")) and process.poll() is None and time.time() < deadline:
                    time.sleep(0.05)
                if process.poll() is not None or not os.path.exists(os.path.join(directory, "hanging")):
                    problem = "没等到假 torch 挂住的标记，脚本就退了或超时了"
                    if process.poll() is None:
                        process.kill()
                else:
                    process.terminate()
            process.wait(timeout=RUN_TIMEOUT_SECONDS)
        except subprocess.TimeoutExpired:
            problem = problem or f"{RUN_TIMEOUT_SECONDS} 秒没跑完"
            process.kill()
            process.wait()
    with open(log_path, encoding="utf-8") as handle:
        events = handle.read().splitlines()
    with open(os.path.join(directory, "stdout.txt"), encoding="utf-8", errors="replace") as handle:
        output_text = handle.read()
    with open(os.path.join(directory, "stderr.txt"), encoding="utf-8", errors="replace") as handle:
        error_text = handle.read()
    still_alive = [process_id for process_id in card_process_ids(events, script) if os.path.exists(f"/proc/{process_id}")]
    if still_alive and not problem:
        problem = f"上卡的子进程 {','.join(still_alive)} 在这一趟结束后还活着"
    return Outcome(process.returncode, output_text, error_text, events, problem)


def card_events(outcome: Outcome, script: str) -> list[str]:
    return [line for line in outcome.events if line.startswith("cuda ") and f" program={script} " in line]


def fields_of(line: str) -> dict[str, str]:
    return dict(word.split("=", 1) for word in line.split()[1:] if "=" in word)


def return_lines(outcome: Outcome) -> list[str]:
    return [line for line in outcome.events if line.startswith("return ")]


def take_count(outcome: Outcome) -> int:
    return outcome.events.count("take")


def smi_identifiers(outcome: Outcome) -> list[str]:
    return [line.split("id=", 1)[1] for line in outcome.events if line.startswith("smi ")]


def nothing_ran(outcome: Outcome, script: str) -> list[str]:
    """没有产物、没碰 CUDA：标准输出空，假 torch 没记任何一步。"""
    problems = []
    if outcome.standard_output:
        problems.append("标准输出不空（写了产物）")
    if card_events(outcome, script):
        problems.append("碰了 CUDA")
    return problems


def check_declared_once_and_returned(outcome: Outcome, script: str) -> list[str]:
    problems = []
    if outcome.exit_code != 0:
        problems.append(f"退出码 {outcome.exit_code}（期望 0）")
    if DONE_LINE not in outcome.standard_output:
        problems.append("标准输出没有 name=done 行")
    if take_count(outcome) != 1:
        problems.append(f"领卡 {take_count(outcome)} 次（期望 1）")
    returns = return_lines(outcome)
    if len(returns) != 1:
        problems.append(f"还卡 {len(returns)} 次（期望 1）")
    elif fields_of(returns[0]).get("cards") != LEASED_CARD:
        problems.append(f"还的不是领到的那一串：{returns[0]}")
    return problems


def check_returned_after_the_card_process_exits(outcome: Outcome, script: str) -> list[str]:
    events = card_events(outcome, script)
    returns = return_lines(outcome)
    if not events or not returns:
        return ["没有 CUDA 操作或没有还卡，核不了先后"]
    problems = []
    last_card_index = max(index for index, line in enumerate(outcome.events) if line in events)
    if outcome.events.index(returns[0]) < last_card_index:
        problems.append("还卡之后上卡的进程还在碰 CUDA")
    if fields_of(returns[0]).get("alive") != "-":
        problems.append(f"还卡那一刻上卡的进程还活着：{returns[0]}")
    return problems


def check_only_the_leased_card_visible(outcome: Outcome, script: str) -> list[str]:
    events = card_events(outcome, script)
    if not events:
        return ["没有 CUDA 操作，核不了可见的卡"]
    problems = []
    visible = sorted({fields_of(line).get("visible", "?") for line in events})
    if visible != [LEASED_INDEX]:
        problems.append(f"CUDA_VISIBLE_DEVICES 取值 {visible}（期望只有 {LEASED_INDEX}）")
    order = sorted({fields_of(line).get("order", "?") for line in events})
    if order != ["PCI_BUS_ID"]:
        problems.append(f"CUDA_DEVICE_ORDER 取值 {order}（期望 PCI_BUS_ID）")
    if script == PROVE_SCRIPT_NAME and sorted(set(smi_identifiers(outcome))) != [LEASED_INDEX]:
        problems.append(f"nvidia-smi 点的序号 {sorted(set(smi_identifiers(outcome)))}（期望只有 {LEASED_INDEX}）")
    return problems


def check_returned_after_an_error(outcome: Outcome, script: str) -> list[str]:
    problems = []
    if outcome.exit_code == 0:
        problems.append("出错却退 0")
    if take_count(outcome) != 1:
        problems.append(f"领卡 {take_count(outcome)} 次（期望 1）")
    if not return_lines(outcome):
        problems.append("出错之后没还卡")
    return problems


def check_returned_after_an_interrupt(outcome: Outcome, script: str) -> list[str]:
    problems = []
    if outcome.exit_code != 143:
        problems.append(f"退出码 {outcome.exit_code}（期望 143，即 128 加 TERM 的 15）")
    returns = return_lines(outcome)
    if not returns:
        problems.append("被打断之后没还卡")
    elif fields_of(returns[0]).get("alive") != "-":
        problems.append(f"还卡那一刻上卡的进程还活着：{returns[0]}")
    return problems


def check_no_channel(outcome: Outcome, script: str) -> list[str]:
    problems = nothing_ran(outcome, script)
    if outcome.exit_code != 78:
        problems.append(f"退出码 {outcome.exit_code}（期望 78）")
    if "→ 怎么办" not in outcome.standard_error:
        problems.append("标准错误没写出路")
    if take_count(outcome) != 0:
        problems.append("没有通道还去领了卡")
    return problems


def check_refused(outcome: Outcome, script: str) -> list[str]:
    problems = nothing_ran(outcome, script)
    if outcome.exit_code != 1:
        problems.append(f"退出码 {outcome.exit_code}（期望 1）")
    if return_lines(outcome):
        problems.append("没领到卡却还了卡")
    return problems


def check_empty_card_string(outcome: Outcome, script: str) -> list[str]:
    problems = nothing_ran(outcome, script)
    if outcome.exit_code != 1:
        problems.append(f"退出码 {outcome.exit_code}（期望 1）")
    return problems


def check_garbage_card(outcome: Outcome, script: str) -> list[str]:
    problems = nothing_ran(outcome, script)
    if outcome.exit_code != 1:
        problems.append(f"退出码 {outcome.exit_code}（期望 1）")
    returns = return_lines(outcome)
    if len(returns) != 1 or fields_of(returns[0]).get("cards") != GARBAGE_CARDS:
        problems.append(f"领到的那一串没照原样还一次：{returns}")
    return problems


def check_failed_return_only_warns(outcome: Outcome, script: str) -> list[str]:
    problems = []
    if outcome.exit_code != 0:
        problems.append(f"退出码 {outcome.exit_code}（期望 0：还卡没成不改退出码）")
    if DONE_LINE not in outcome.standard_output:
        problems.append("标准输出没有 name=done 行")
    warnings = [line for line in outcome.standard_error.splitlines() if line.startswith("  ! 还卡没成")]
    if len(warnings) != 1:
        problems.append(f"「!」行 {len(warnings)} 行（期望 1）")
    return problems


def check_inherited_variable_cleared(outcome: Outcome, script: str) -> list[str]:
    events = card_events(outcome, script)
    if not events:
        return ["没有 CUDA 操作，核不了环境"]
    reached = sorted({fields_of(line).get("cards", "?") for line in events})
    return [] if reached == ["-"] else [f"上卡的进程里 GPU_CARDS_BY_PRIORITY 取值 {reached}（期望没有）"]


class Cell(NamedTuple):
    case: str
    expected: str
    situation: str
    check: Callable[[Outcome, str], list[str]]


CELLS = (
    Cell("card-taken-once-and-returned-once", "领一次还一次，还的是领到的那一串，产物有 name=done", "green", check_declared_once_and_returned),
    Cell("card-returned-after-the-card-process-exits", "还卡在上卡的进程退出之后", "green", check_returned_after_the_card_process_exits),
    Cell("only-the-leased-card-visible", "只看得到领到的那一张", "green", check_only_the_leased_card_visible),
    Cell("card-returned-after-an-error", "出错也还卡", "error", check_returned_after_an_error),
    Cell("card-returned-after-an-interrupt", "被打断退 143 也还卡", "interrupt", check_returned_after_an_interrupt),
    Cell("no-channel-exits-78", "没有领卡通道退 78、不写产物", "no-channel", check_no_channel),
    Cell("refused-lease-runs-nothing", "领卡被拒退 1、不起上卡的那一段", "refused", check_refused),
    Cell("empty-card-string-counts-as-refused", "没有卡串退 1、不起上卡的那一段", "empty", check_empty_card_string),
    Cell("unparseable-card-string-is-returned-and-fails", "认不出序号退 1、照原串还、不起上卡的那一段", "garbage-card", check_garbage_card),
    Cell("failed-return-only-warns", "还卡没成只多一行「!」、退出码不变", "return-failure", check_failed_return_only_warns),
    Cell("inherited-card-variable-cleared", "环境里原有的 GPU_CARDS_BY_PRIORITY 到不了上卡的进程", "inherited-variable", check_inherited_variable_cleared),
)
HEADER_CASE = "header-declares-on-demand"


def header_problems(tree: str) -> list[str]:
    """样板树里三份脚本文件头前 DECLARATION_HEADER_LINES 行有没有恰好是 `# task-server-card-lease: on-demand` 的一行（读法同服务）。"""
    problems = []
    for script in SCRIPT_NAMES:
        declared = None
        with open(os.path.join(tree, "scripts", "experiment-apparatus", script), encoding="utf-8") as handle:
            for _ in range(DECLARATION_HEADER_LINES):
                line = handle.readline()
                if not line:
                    break
                if line.startswith(DECLARATION_MARK):
                    declared = tuple(line[len(DECLARATION_MARK):].split())
                    break
        if declared != (DECLARATION_VALUE,):
            problems.append(f"{script}：文件头的按需领卡声明是 {declared}（期望 {(DECLARATION_VALUE,)}）")
    return problems


def header_switches(text: str) -> list[tuple[str, str, str]]:
    """文件头开关表：两行一条，`  E21_CARD_LEASE_BREAK=<项>  <弄坏哪一处>` 下一行写点名的格 → [(开关名, 弄坏哪一处, 点名的格)]。"""
    lines = text.split("\n")
    found = []
    for number, line in enumerate(lines):
        entry = re.match(rf"^  {BREAK_VARIABLE}=([a-z0-9-]+)  (\S.*)$", line)
        if entry:
            found.append((entry.group(1), entry.group(2), "".join(lines[number + 1:number + 2]).strip()))
    return found


def switch_table_problems() -> list[str]:
    """库与本文件两处文件头开关表、SWITCH_CELLS 逐条对得上：开关名、弄坏哪一处、点名的格都相同；开关变量名与库里的相同。"""
    problems = []
    if BREAK_VARIABLE != e21_card_lease.BREAK_VARIABLE:
        problems.append(f"开关变量名：本文件写 {BREAK_VARIABLE}，库写 {e21_card_lease.BREAK_VARIABLE}")
    tables = []
    for path in (os.path.join(APPARATUS_DIRECTORY, LIBRARY_NAME), os.path.realpath(__file__)):
        label = os.path.basename(path)
        with open(path, encoding="utf-8") as handle:
            rows = header_switches(handle.read())
        names = [name for name, _described, _case in rows]
        problems += [f"{label} 文件头开关表里 {name} 写了不止一条" for name in sorted(set(names)) if names.count(name) > 1]
        table = {name: (described, case) for name, described, case in rows}
        problems += [f"{label} 文件头开关表里有、SWITCH_CELLS 里没有：{name}" for name in sorted(set(table) - set(SWITCH_CELLS))]
        problems += [f"SWITCH_CELLS 里有、{label} 文件头开关表里没有：{name}" for name in sorted(set(SWITCH_CELLS) - set(table))]
        problems += [f"{label} 文件头开关表里 {name} 点名的格 {table[name][1]} 与 SWITCH_CELLS 里的 {SWITCH_CELLS[name]} 不同"
                     for name in sorted(set(table) & set(SWITCH_CELLS)) if table[name][1] != SWITCH_CELLS[name]]
        tables.append(table)
    library, own = tables
    problems += [f"开关 {name} 弄坏哪一处的说明，库与本文件文件头写得不一样" for name in sorted(set(library) & set(own)) if library[name][0] != own[name][0]]
    named_cases = {cell.case for cell in CELLS} | {HEADER_CASE}
    problems += [f"开关 {name} 点名的格 {case} 不是自证的格" for name, case in SWITCH_CELLS.items() if case not in named_cases]
    return problems


def judge(tree: str, outcomes: dict[tuple[str, str], Outcome]) -> list[tuple[str, str, str]]:
    """→ 判错的格 [(格名, 期望, 实际)]。"""
    misjudged = []
    declaration_problems = header_problems(tree)
    if declaration_problems:
        misjudged.append((HEADER_CASE, "三份脚本文件头都声明 on-demand", "；".join(declaration_problems)))
    for cell in CELLS:
        problems = []
        for script in SCRIPT_NAMES:
            outcome = outcomes[(cell.situation, script)]
            if outcome.problem:
                problems.append(f"{script}：这一趟没跑成样本：{outcome.problem}")
            else:
                problems += [f"{script}：{problem}" for problem in cell.check(outcome, script)]
        if problems:
            misjudged.append((cell.case, cell.expected, "；".join(problems)))
    return misjudged


def run_all(tree: str, base: str, workers: int) -> dict[tuple[str, str], Outcome]:
    """九种局面 × 三份脚本并行各跑一趟；收回的趟数与派出去的对不上就退出。"""
    keys = [(situation.name, script) for situation in SITUATIONS for script in SCRIPT_NAMES]
    situation_by_name = {situation.name: situation for situation in SITUATIONS}
    with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
        futures = [pool.submit(run_script, tree, base, script, situation_by_name[name]) for name, script in keys]
        outcomes = {key: future.result() for key, future in zip(keys, futures)}
    if len(outcomes) != len(keys):
        print(f"  ✗ 派出 {len(keys)} 趟，收回 {len(outcomes)} 趟")
        sys.exit(1)
    return outcomes


def workers_for_this_run() -> int:
    if os.environ.get(BREAK_VARIABLE):
        return SWITCH_RUN_WORKERS
    sys.path.insert(0, os.path.join(e21_card_lease.REPOSITORY_ROOT, "scripts", "hosts"))
    import heavy_threads  # noqa: E402
    try:
        return heavy_threads.default_heavy_threads()
    except heavy_threads.HeavyThreadDefaultError as error:
        print(f"  ✗ 读不到每台默认重活线程数：{error}", file=sys.stderr)
        print(f"     → 怎么办：{error.howto}", file=sys.stderr)
        sys.exit(2)


def switch_round(switch: str, expected_case: str) -> tuple[str, int, bool]:
    """另起子进程开着这一个开关跑 --selftest：→ (开关名, 退出码, 输出里有没有点名的格)。"""
    environment = dict(os.environ)
    environment[BREAK_VARIABLE] = switch
    completed = subprocess.run([sys.executable, os.path.realpath(__file__), INNER_ARGUMENT], env=environment, stdin=subprocess.DEVNULL,
                               capture_output=True, text=True)
    return switch, completed.returncode, f"判错的样本：{expected_case}（" in completed.stdout


def run_selftest(inner: bool) -> int:
    """inner 是开关轮起的那一趟（--selftest-inner，环境里设着一个开关）：判九种局面、打判错的样本；不是 inner 是整份自证（环境里没有开关）。"""
    switch = os.environ.get(BREAK_VARIABLE, "")
    if inner and not switch:
        print(f"用法：{INNER_ARGUMENT} 只给开关轮用，环境里要设 {BREAK_VARIABLE}=<项>；整份自证是 --selftest", file=sys.stderr)
        return 2
    table_problems = [] if inner else switch_table_problems()
    if table_problems:
        for problem in table_problems:
            print(f"      {problem}")  # gate-lint:detail
        print(f"  ✗ e21-card-lease-selftest.py：开关表与自证对不上（{len(table_problems)} 处）")
        print("     → 怎么办：e21_card_lease.py 与本文件两处文件头的开关表、本文件的 SWITCH_CELLS 逐条改成一样")
        return 1
    workers = workers_for_this_run()
    base = tempfile.mkdtemp(prefix="e21-card-lease-selftest-")
    try:
        tree = build_tree(base)
        misjudged = judge(tree, run_all(tree, base, workers))
    finally:
        shutil.rmtree(base, ignore_errors=True)
    checked = (len(CELLS) + 1) * len(SCRIPT_NAMES)
    if inner:
        for case, expected, actual in misjudged:
            print(f"      判错的样本：{case}（期望 {expected}，实际 {actual}）")  # gate-lint:detail
        if misjudged:
            print(f"  ✗ 弄坏开关 {switch} 下自证判错 {len(misjudged)} 个样本")
            print("     → 怎么办：这是弄坏开关下的预期结果，由 --selftest 的开关轮核它红在点名的格上；整份自证不设这个环境变量。")
            return 1
        print(f"  ✓ 弄坏开关 {switch} 下自证居然通过了（{len(CELLS) + 1} 个样本）")
        return 0
    if misjudged:
        for case, expected, actual in misjudged:
            print(f"      判错的样本：{case}（期望 {expected}，实际 {actual}）")  # gate-lint:detail
        print(f"  ✗ e21-card-lease-selftest.py 自证判错 {len(misjudged)} 个样本（共 {len(CELLS) + 1} 个）")
        print("     → 怎么办：看判错的样本，改 e21_card_lease.py 的领还卡或三份脚本的接法，再跑 --selftest")
        return 1
    with concurrent.futures.ThreadPoolExecutor(max_workers=SWITCH_ROUND_PARALLEL) as pool:
        rounds = list(pool.map(lambda pair: switch_round(*pair), SWITCH_CELLS.items()))
    if len(rounds) != len(SWITCH_CELLS):
        print(f"  ✗ 派出 {len(SWITCH_CELLS)} 个开关轮，收回 {len(rounds)} 个")
        return 1
    silent = [(name, code) for name, code, named in rounds if code != 1 or not named]
    if silent:
        for name, code in silent:
            print(f"      {name}：退出码 {code}，或者没有红在「{SWITCH_CELLS[name]}」上")  # gate-lint:detail
        print(f"  ✗ 有 {len(silent)} 个弄坏开关打开后自证没红在点名的格上（共 {len(SWITCH_CELLS)} 个开关）")
        print("     → 怎么办：改自证让它红在 e21_card_lease.py 文件头开关表点名的格，或改开关让它真的弄坏点名的那一处；"
              f"单跑：{BREAK_VARIABLE}=<开关名> python3 scripts/experiment-apparatus/e21-card-lease-selftest.py {INNER_ARGUMENT}")
        return 1
    print(f"  ✓ e21-card-lease-selftest.py 自证通过（查了 {checked} 项：{len(SITUATIONS)} 种局面 × {len(SCRIPT_NAMES)} 份脚本各一趟，"
          f"{len(CELLS) + 1} 个格；{len(SWITCH_CELLS)} 个弄坏开关逐个红在点名的格；并行 {workers} 路、开关轮并行 {SWITCH_ROUND_PARALLEL} 路）")
    return 0


def main(arguments: list[str]) -> int:
    refused = e21_card_lease.refuse_break_outside_selftest(arguments)
    if refused is not None:
        return refused
    if arguments == ["--selftest"]:
        return run_selftest(inner=False)
    if arguments == [INNER_ARGUMENT]:
        return run_selftest(inner=True)
    print("用法：e21-card-lease-selftest.py --selftest（--selftest-inner 只给它的开关轮用）", file=sys.stderr)
    return 2


if __name__ == "__main__":
    preflight(__file__)
    sys.exit(main(sys.argv[1:]))
