#!/usr/bin/env python3
# admission: always 每一次调都现查各台此刻的余量，上一次的结论不替这一次作保
# run-condition: command bash python3 git
"""task-server 载入的判法库：各台此刻余量多少（probe_local、probe_peer）、
这台此刻放不放得下一条命令（judge_host，预测由服务照它自己的记录填进 Prediction）、本机清场包装的登记里有几个活着的（live_quiesce_holders），与多机配置里第二台的读法（Peer、settings_from_environment）。
挑哪台、排队、记账、起任务都在 task-server/task_server.py（它经 lib.load_module 按路径载入这一份）；这里不派活、不排队、不记账。

用法：
  host-dispatch.py --selftest       假的 /proc/meminfo、假 nvidia-smi、假 ssh 走一遍各格，末尾逐个打开弄坏开关核它红在点名的格
  别的参数一律退 2（认不出的参数；这一份只给 task-server 载入，跑项目代码的命令经 task-server/task submit -- <命令…>）。

服务调的几样（task_server.py 按名字调；judge_host 多了 margin 一个参数，服务交那台此刻的余量）：
  settings_from_environment()                  读环境：清场包装的状态目录（GPU_QUIESCE_STATE_DIRECTORY，默认 ${XDG_RUNTIME_DIR:-/tmp}/singlefs-gpu-quiesce）、
                                               多机配置的判法与读法所在。
  Peer(settings)                               多机配置里的第二台（判法 scripts/crash/crash-point-replay-shard-configuration-check.sh，读法 scripts/hosts/peer-host-lib.sh 的
                                               peer_host_load_configuration）：双机开关 ENABLE_ACROSS_MACHINES 没开（判法退 3）或配置判不过时 available 为 False、why 写为什么；
                                               memory_cap_bytes 是多机配置 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项（按地址取，peer-host-lib.sh 的 PEER_HOST_SLICE_CAP）。
  probe_local(settings, gpu)、probe_peer(peer, gpu, apps_only=False)
                                               本机 / 第二台此刻的余量：内存是 memory_peaks_lib.headroom（/proc/meminfo 与 slice 的总上限、回收不掉的用量，连同这台此刻量的余量
                                               memory_peaks_lib.MARGIN_REPORT_KEY 那三项，memory_peaks_lib.py 文件头「余量」；连同 slice 下每个在跑的 scope 此刻回收不掉的用量与标签
                                               memory_peaks_lib.SCOPES_REPORT_KEY，那份文件头「在跑的 scope」：task-server 照它算那台每条在跑尝试还没涨到量的那一截（两台同一段），
                                               与余量同一次 ssh 量回来；记账在 task-server，这里不减任何账），
                                               带 gpu 再读 nvidia-smi 各张卡的 memory.total / memory.free；第二台把 memory_peaks_lib.py 的源码与本文件里取显存的
                                               两个函数送过去（python3 -c，base64 编码），用 peer-host-lib.sh 的 peer_host_run 跑，余量在第二台上照同一个算法量它自己的 slice。
                                               交 (报告, 查不成的原因)。
  report_margin(memory)                        报告里这台此刻的余量：(字节, 量不到的原因)；报告里 MARGIN_REPORT_KEY 不是非负整数的交 None 与 MARGIN_PROBLEM_KEY 写的原因，不退回固定的数。
  Prediction                                   一条命令的预测：键、内存预测、上限、记账量、从哪来（memory_history），带 gpu 再填显存预测；task-server 照它自己的记录
                                               填好交给 judge_host（task_server.py 的 recorded_prediction、unrecorded_prediction、pinned_prediction），显存缺省取 DEFAULT_GPU_PREDICTION_MEBIBYTES。
  judge_host(host, report, prediction, gpu, pending_memory, pending_gpu, configured_cap_bytes, margin)
                                               margin 是这台此刻的余量（字节；服务照 report_margin 取这台自己报告里的那一项，各台各用各的）。
                                               两台同一个判法，不看 host 是哪一台：configured_cap_bytes 给了（服务交那台总上限另收到的数，第二台是 Peer.memory_cap_bytes）
                                               先收到它、记账量比它还大算永远放不下；再收到这台 slice 总上限减 margin 为止；
                                               要的空闲 = min(预测 + margin, 上限)（任务不会超过自己的上限）；放得下 = 要的空闲 ≤ 空闲 − pending_memory，且记账量 ≤ slice 放得下的 − margin − pending_memory
                                               （pending_* 是服务自己账上这台还没涨到量的那一截；slice 放得下的只减了进程的内存，slice 自己挂着的余量也占着总上限，
                                               另减 margin：在跑的上限之和连同这一条不超过总上限减余量）；带 gpu 再要显存预测 ≤ 卡此刻放得下的
                                               （清场登记里没有活着的按卡的总量算，有就按此刻的空闲算，再减 GPU_MARGIN_MEBIBYTES 与 pending_gpu）。
                                               margin 是 None（量不到）或 slice 总上限不比 margin 大：此刻放不下（不是永远放不下），why 写哪一样，不退回固定的数。
                                               永远放不下（never）：记账量比 slice 总上限大、或要的空闲比整机内存大、或显存预测比卡的总量减余量大、或带 gpu 而读不到卡。
  live_quiesce_holders(settings)               本机清场包装 holders/ 下活着的登记数（<进程号>-<起始时刻>，进程在、起始时刻对得上才算活着）。

常量的依据：
  内存的余量不是常量：每台每次读余量时现量（memory_peaks_lib.py 文件头「余量」），这里不写数；BREAK_FIXED_MARGIN_BYTES 只给弄坏开关 margin-fallback-fixed 用。
  GPU_MARGIN_MEBIBYTES 2560：判器留给驱动的那一份（crates/singlefs-verification/src/crash_judge_dispatch.rs 的 MEBIBYTES_LEFT_FOR_THE_DRIVER）。
  DEFAULT_GPU_PREDICTION_MEBIBYTES 8192：最小那张卡（16G）的一半；推的，没量过。

弄坏开关 HOST_DISPATCH_BREAK=<项>（只给 --selftest 证红用，逗号连几个；在 --selftest 之外设了就拒、退 2）。项名、弄坏哪一处、打开后红在哪几格
  （格名是「✗ 自证 <格名>：」里的那一段；末尾那一轮照 SWITCH_EXPECTED_CELLS 逐个核）：
  HOST_DISPATCH_BREAK=ignore-pending  判放不放得下不减服务交来的 pending_memory
      「✗ 自证 pending-subtracted：」
  HOST_DISPATCH_BREAK=never-ignored  永远放不下的也只当此刻放不下
      「✗ 自证 never-fits：」
  HOST_DISPATCH_BREAK=no-configured-cap-clamp  上限不按交进来的那台另收到的数（configured_cap_bytes）收
      「✗ 自证 configured-cap-clamp：」
  HOST_DISPATCH_BREAK=no-slice-cap-clamp  上限不按这台 slice 的总上限收（比 slice 总上限大的照原样给）
      「✗ 自证 slice-cap-clamp：」
  HOST_DISPATCH_BREAK=margin-over-cap  要的空闲量照预测加余量算、不收到这一条的上限（顶格的任务要的比这台能给的多）
      「✗ 自证 margin-not-over-cap：」
  HOST_DISPATCH_BREAK=ignore-gpu  带 gpu 也不看显存
      「✗ 自证 gpu-fit：」
  HOST_DISPATCH_BREAK=free-only  没有活着的清场登记时也只按卡此刻的空闲算
      「✗ 自证 gpu-total-before-quiesce：」
  HOST_DISPATCH_BREAK=peer-despite-switch-off  双机开关没开也当第二台用得上
      「✗ 自证 switch-off-peer-unavailable：」
  HOST_DISPATCH_BREAK=stale-holders-counted  清场登记里进程不在了的那一条也算活着
      「✗ 自证 quiesce-holders：」
  HOST_DISPATCH_BREAK=margin-fallback-fixed  judge_host 收到量不到的余量（None）时退回固定的 2G（BREAK_FIXED_MARGIN_BYTES）
      「✗ 自证 margin-unmeasured-refused：」
  HOST_DISPATCH_BREAK=total-within-margin-accepted  judge_host 不判 slice 总上限不比余量大，照收到总上限减余量（不大于 0）的上限往下判
      「✗ 自证 total-within-margin-refused：」
  HOST_DISPATCH_BREAK=slice-room-ignores-margin  判记账放不放得下时 slice 放得下的不减这台此刻的余量（在跑的上限之和能记到总上限）
      「✗ 自证 slice-room-keeps-margin：」
  HOST_DISPATCH_BREAK=probe-program-uncompressed  送第二台的探测程序照改前不压缩、整份 base64 当一个参数（过单个参数的界就查不成）
      「✗ 自证 probe-peer：」（此刻的探测程序不压缩已过界）、「✗ 自证 probe-program-large：」
"""
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import base64  # noqa: E402
import concurrent.futures  # noqa: E402
import contextlib  # noqa: E402
import gzip  # noqa: E402
import inspect  # noqa: E402
import json  # noqa: E402
import os  # noqa: E402
import shutil  # noqa: E402
import subprocess  # noqa: E402
import sys  # noqa: E402
import tempfile  # noqa: E402
from dataclasses import dataclass  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, SCRIPT_DIRECTORY)
import memory_peaks_lib as peaks  # noqa: E402

MEBIBYTE = peaks.MEBIBYTE
GIBIBYTE = 1024 * MEBIBYTE
BREAK_FIXED_MARGIN_BYTES = 2 * GIBIBYTE   # 只给弄坏开关 margin-fallback-fixed：量不到余量时照改前退回的固定数
GPU_MARGIN_MEBIBYTES = 2560
DEFAULT_GPU_PREDICTION_MEBIBYTES = 8192
PROBE_TIMEOUT_SECONDS = 60
# 送去第二台的探测程序先 gzip 再 base64，整段当 bash -c 的一个参数：Linux 单个参数上限 128 KiB（MAX_ARG_STRLEN），留 32 KiB 给外面那层命令文本与 %q 转义
PROBE_PROGRAM_ARGUMENT_LIMIT = 96 * 1024
EXIT_USAGE = 2
PROBE_MARK = "HOST_DISPATCH_PROBE "


def broken(item):
    return item in os.environ.get("HOST_DISPATCH_BREAK", "").split(",")


def human(byte_count):
    return peaks.human(byte_count)


# ── 取显存：本机直接调，第二台把这两个函数的源码送过去调（所以只用标准库、不引用本文件别处的名字）──
def gpu_cards(environment):
    """nvidia-smi 报的每张卡：{"cards": [{"index", "total", "free"}（MiB）…]}；没有 nvidia-smi 或读不出交 {"problem": 为什么}。"""
    import subprocess as process
    try:
        completed = process.run(["nvidia-smi", "--query-gpu=index,memory.total,memory.free", "--format=csv,noheader,nounits"],
                                capture_output=True, text=True, env=environment, timeout=30, stdin=process.DEVNULL)
    except (OSError, process.TimeoutExpired) as error:
        return {"problem": f"nvidia-smi 起不来或超时（{type(error).__name__}）"}
    if completed.returncode != 0:
        return {"problem": f"nvidia-smi 退 {completed.returncode}：{completed.stderr.strip()[:200]}"}
    cards = []
    for line in completed.stdout.splitlines():
        fields = [piece.strip() for piece in line.split(",")]
        if len(fields) == 3 and all(piece.isdigit() for piece in fields):
            cards.append({"index": int(fields[0]), "total": int(fields[1]), "free": int(fields[2])})
    return {"cards": cards} if cards else {"problem": "nvidia-smi 一张卡都没报"}


def gpu_apps(environment):
    """卡上此刻的计算进程：{"apps": {进程号: 已用 MiB}}（几张卡上同一个进程相加）；读不出交 {"problem": 为什么}。"""
    import subprocess as process
    try:
        completed = process.run(["nvidia-smi", "--query-compute-apps=pid,used_memory", "--format=csv,noheader,nounits"],
                                capture_output=True, text=True, env=environment, timeout=30, stdin=process.DEVNULL)
    except (OSError, process.TimeoutExpired) as error:
        return {"problem": f"nvidia-smi 起不来或超时（{type(error).__name__}）"}
    if completed.returncode != 0:
        return {"problem": f"nvidia-smi 退 {completed.returncode}：{completed.stderr.strip()[:200]}"}
    apps = {}
    for line in completed.stdout.splitlines():
        fields = [piece.strip() for piece in line.split(",")]
        if len(fields) == 2 and fields[0].isdigit() and fields[1].isdigit():
            apps[fields[0]] = apps.get(fields[0], 0) + int(fields[1])
    return {"apps": apps}


PROBE_DRIVER = '''
import json, os, sys
_environment = dict(os.environ)
if "--apps" in sys.argv[1:]:
    _report = gpu_apps(_environment)
else:
    _report = {"memory": headroom(_environment)}
    if "--gpu" in sys.argv[1:]:
        _report["gpu"] = gpu_cards(_environment)
print("HOST_DISPATCH_PROBE " + json.dumps(_report))
'''


def peer_probe_program():
    with open(os.path.join(SCRIPT_DIRECTORY, "memory_peaks_lib.py"), encoding="utf-8") as handle:
        library = handle.read()
    return "\n".join([library, inspect.getsource(gpu_cards), inspect.getsource(gpu_apps), PROBE_DRIVER])


@dataclass
class Settings:
    repository_root: str
    configuration_check: str
    peer_host_lib: str
    quiesce_state_directory: str
    environment: dict
    say: object = None

    def tell(self, text):
        (self.say or (lambda line: print(line, file=sys.stderr, flush=True)))(text)


def settings_from_environment():
    environment = dict(os.environ)
    repository_root = os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY))
    runtime = environment.get("XDG_RUNTIME_DIR") or "/tmp"
    return Settings(
        repository_root=repository_root,
        configuration_check=os.path.join(SCRIPT_DIRECTORY, "..", "crash", "crash-point-replay-shard-configuration-check.sh"),
        peer_host_lib=os.path.join(SCRIPT_DIRECTORY, "peer-host-lib.sh"),
        quiesce_state_directory=environment.get("GPU_QUIESCE_STATE_DIRECTORY") or os.path.join(runtime, "singlefs-gpu-quiesce"),
        environment=environment)


# ── 第二台 ──
class Peer:
    """多机配置里的第二台：available 为 False 时 why 写为什么不用它（双机关着、配置判不过）。"""

    def __init__(self, settings):
        self.settings = settings
        self.available = False
        self.why = ""
        self.declarations = ""
        self.memory_cap_bytes = None
        loader = ('source "$1" || exit 70; peer_host_load_configuration "$2" "$3" >&2; status=$?; '
                  'if ((status == 0)); then declare -p PEER_HOST_ADDRESS THIS_HOST_ADDRESS PEER_HOST_CARGO_BIN_DIRECTORY PEER_HOST_BUILD_ROOT PEER_HOST_SLICE_CAP '
                  'THIS_HOST_CARGO_BIN_DIRECTORY THIS_HOST_BUILD_ROOT peer_host_is_this_machine; fi; exit $status')
        environment = settings.environment
        if broken("peer-despite-switch-off"):
            # 弄坏开关：双机关着也当开着去读第二台（借判法自己的弄坏开关 switch-off-is-on）
            environment = dict(environment, MULTI_HOST_CONFIGURATION_BREAK="switch-off-is-on")
        completed = subprocess.run(["bash", "-c", loader, "host-dispatch", settings.peer_host_lib, settings.configuration_check, settings.repository_root],
                                   capture_output=True, text=True, env=environment, stdin=subprocess.DEVNULL)
        if completed.returncode == 3:
            self.why = "多机配置里双机开关 ENABLE_ACROSS_MACHINES 没开"
            return
        if completed.returncode != 0:
            self.why = f"多机配置判不过（判法 {os.path.basename(settings.configuration_check)} 退 {completed.returncode}；单跑它看原因）"
            return
        self.declarations = completed.stdout
        cap_line = next((line for line in completed.stdout.splitlines() if "PEER_HOST_SLICE_CAP=" in line), "")
        self.memory_cap_bytes = peaks.size_bytes(cap_line.split("=", 1)[1].strip().strip('"')) if cap_line else None
        self.available = True

    def run(self, text, timeout=PROBE_TIMEOUT_SECONDS):
        """在第二台上 bash -c 跑 text（peer-host-lib.sh 的 peer_host_run，错误输出换掉私有的字）：(退出码, 标准输出, 标准错误)。"""
        runner = 'source "$1" || exit 70; eval "$2"; peer_host_run / "$3"'
        try:
            completed = subprocess.run(["bash", "-c", runner, "host-dispatch", self.settings.peer_host_lib, self.declarations, text],
                                       capture_output=True, text=True, env=self.settings.environment, timeout=timeout, stdin=subprocess.DEVNULL)
        except subprocess.TimeoutExpired:
            return 124, "", f"第二台 {timeout} 秒没回话"
        return completed.returncode, completed.stdout, completed.stderr


def parse_probe(stdout):
    line = next((line for line in reversed(stdout.splitlines()) if line.startswith(PROBE_MARK)), None)
    if line is None:
        return None
    try:
        return json.loads(line[len(PROBE_MARK):])
    except ValueError:
        return None


def probe_local(settings, gpu):
    try:
        report = {"memory": peaks.headroom(settings.environment)}
    except (OSError, KeyError, ValueError) as error:
        return None, f"本机的内存余量查不成（{type(error).__name__}: {error}）"
    if gpu:
        report["gpu"] = gpu_cards(settings.environment)
    return report, ""


def probe_peer(peer, gpu, apps_only=False):
    source = peer_probe_program().encode("utf-8")
    flags = "--apps" if apps_only else ("--gpu" if gpu else "")
    if broken("probe-program-uncompressed"):
        # 弄坏开关：照改前不压缩、解出来的整份程序当 python3 -c 的一个参数（程序或它的 base64 过单个参数的界就撞 E2BIG）
        program = base64.b64encode(source).decode("ascii")
        text = f'python3 -B -c "$(printf %s {program} | base64 -d)" {flags}'
    else:
        # 压完的 base64 是 bash -c 的一个参数（界 PROBE_PROGRAM_ARGUMENT_LIMIT）；解出来的程序经标准输入交 python3 -c 里的 exec，不当参数，
        #   也不用 python3 -（那样 __file__ 是 <stdin>，memory_peaks_lib.py 末尾的「单独跑只认 --selftest」会拒）
        program = base64.b64encode(gzip.compress(source, 9)).decode("ascii")
        text = (f"printf %s {program} | base64 -d | gzip -dc | "
                f"python3 -B -c 'import sys; exec(compile(sys.stdin.read(), \"<host-dispatch probe>\", \"exec\"))' {flags}")
    if len(program) > PROBE_PROGRAM_ARGUMENT_LIMIT:
        return None, (f"第二台的余量查不成：探测程序压完 {len(program)} 字节，超过单个参数的界 {PROBE_PROGRAM_ARGUMENT_LIMIT}；"
                      "把 memory_peaks_lib.py 里只给自证用的部分挪出探测程序，或改成经标准输入送")
    status, stdout, stderr = peer.run(text)
    report = parse_probe(stdout)
    if status != 0 or report is None:
        return None, f"第二台的余量查不成（退 {status}：{stderr.strip()[-300:]}）"
    return report, ""


def live_quiesce_holders(settings):
    """本机清场包装 holders/ 下活着的登记数（<进程号>-<起始时刻>，进程在、起始时刻对得上才算活着；gpu-quiesce-run.sh 文件头「几趟同时用卡」）。
    弄坏开关 stale-holders-counted：进程不在了的也算。"""
    directory = os.path.join(settings.quiesce_state_directory, "holders")
    count = 0
    with contextlib.suppress(FileNotFoundError):
        for name in os.listdir(directory):
            process_id, _, start_time = name.partition("-")
            if process_id.isdigit() and start_time and (broken("stale-holders-counted") or peaks.process_start_time(process_id) == start_time):
                count += 1
    return count


# ── 预测：task-server 照它自己的记录填（文件头「服务调的几样」的 Prediction）──
@dataclass
class Prediction:
    key: str
    memory_predicted: int        # 字节
    memory_cap: int              # 字节：这一次的上限
    memory_booked: int           # 字节：记账量
    memory_history: str          # 从哪来（服务写它自己的那几种：记录、重试、钉值、没有记录）
    gpu_predicted: int = 0       # MiB
    gpu_history: str = ""
    memory_by_host: object = None   # 钉值按台：{台: 字节}（task-server 照多机配置 SLICE_MEMORY_CAP_BY_HOST 填，判哪一台换成哪一台那一项；别的预测是 None）


# ── 判放不放得下 ──
@dataclass
class Verdict:
    host: str
    fits: bool
    never: bool
    cap: int
    why: str
    memory_short: int = 0        # 字节：内存还差多少（放得下是 0）
    gpu_short: object = 0        # MiB：显存还差多少（放得下或没带 gpu 是 0，读不到卡是 None）


def report_margin(memory):
    """headroom 报告（report["memory"]）里这台此刻的余量：→ (字节, 量不到的原因)（memory_peaks_lib.py 文件头「余量」）。不是非负整数的交 None 与报告里写的原因。"""
    value = memory.get(peaks.MARGIN_REPORT_KEY)
    if isinstance(value, int) and not isinstance(value, bool) and value >= 0:
        return value, ""
    return None, memory.get(peaks.MARGIN_PROBLEM_KEY) or f"报告里没有 {peaks.MARGIN_REPORT_KEY} 这一项"


def judge_host(host, report, prediction, gpu, pending_memory, pending_gpu, configured_cap_bytes, margin):
    memory = report["memory"]
    cap = prediction.memory_cap
    booked = prediction.memory_booked
    if margin is None:
        if not broken("margin-fallback-fixed"):
            return Verdict(host, False, False, cap, f"内存：这台此刻量不到余量（{report_margin(memory)[1]}），这一轮不派")
        margin = BREAK_FIXED_MARGIN_BYTES   # 弄坏开关：照改前退回固定的数
    if configured_cap_bytes is not None and not broken("no-configured-cap-clamp"):
        if booked > configured_cap_bytes:
            return Verdict(host, False, True, cap, f"要记的 {human(booked)} 比多机配置 SLICE_MEMORY_CAP_BY_HOST 里这台那一项 {human(configured_cap_bytes)} 大",
                           memory_short=booked - configured_cap_bytes)
        cap = min(cap, configured_cap_bytes)
    if memory.get("slice_total") and memory["slice_total"] <= margin and not broken("total-within-margin-accepted"):
        return Verdict(host, False, False, cap, f"内存：这台 slice 的总上限 {human(memory['slice_total'])} 不比此刻的余量 {human(margin)} 大，一条都给不出上限，这一轮不派")
    if memory.get("slice_total") and not broken("no-slice-cap-clamp"):
        # 上限不超过这台 slice 的总上限减这台此刻的余量：超过总上限 scope 管不住它、撞的是整个 slice；slice 里挂着的那一截回收不掉，记满一整台永远放不下
        cap = min(cap, memory["slice_total"] - margin)
    # slice 放得下的只减了进程的内存：slice 自己挂着的余量同样占着总上限，不减它的话在跑的上限之和能记到总上限，都涨到上限时 slice 撞顶、连累别的任务
    room = memory["slice_room"] - (0 if broken("slice-room-ignores-margin") else margin)
    if broken("ignore-pending"):
        free = memory["free"]
    else:
        free, room = memory["free"] - pending_memory, room - pending_memory
    # 要的空闲量：预测加余量，但不超过这一条的上限（上限已是 slice 总上限减余量，顶格的再加一遍余量就要的比这台能给的还多，永远放不下）
    needed = prediction.memory_predicted + margin if broken("margin-over-cap") else min(prediction.memory_predicted + margin, cap)
    never = booked > memory["slice_total"] or needed > memory["mem_total"]
    reasons = []
    fits = needed <= free and booked <= room
    memory_short = 0 if fits else max(0, needed - free, booked - room)
    gpu_short = 0
    if not fits:
        reasons.append(f"内存：要的空闲 {human(needed)}（预测 {human(prediction.memory_predicted)} + 这台此刻的余量 {human(margin)}，不超过上限 {human(cap)}）、要记 {human(booked)}，"
                       f"此刻空闲 {human(max(free, 0))}、slice 减去余量放得下 {human(max(room, 0))}")
    if gpu and not broken("ignore-gpu"):
        gpu_report = report.get("gpu") or {}
        cards = gpu_report.get("cards")
        if not cards:
            fits = False
            never = True
            gpu_short = None
            reasons.append(f"显存：{gpu_report.get('problem', '没有卡')}")
        else:
            quiesced = report.get("quiesce_holders", 0) > 0 or broken("free-only")
            available = sum(card["free"] if quiesced else card["total"] for card in cards) - GPU_MARGIN_MEBIBYTES - pending_gpu
            never = never or prediction.gpu_predicted > sum(card["total"] for card in cards) - GPU_MARGIN_MEBIBYTES
            if prediction.gpu_predicted > available:
                fits = False
                gpu_short = prediction.gpu_predicted - available
                reasons.append(f"显存：预测 {prediction.gpu_predicted} MiB，此刻放得下 {max(available, 0)} MiB"
                               f"（{'按空闲算：清场登记里有活着的' if quiesced else '按卡的总量算：还没清场'}，已减余量 {GPU_MARGIN_MEBIBYTES} MiB）")
    if broken("never-ignored"):
        never = False
    return Verdict(host, fits, never and not fits, cap, "；".join(reasons) if reasons else f"内存空闲 {human(free)}、slice 减去余量放得下 {human(room)}",
                   memory_short=memory_short, gpu_short=gpu_short)


# ── 自证 ──
FAKE_NVIDIA_SMI = """#!/usr/bin/env bash
here="$(dirname "$0")"
case "$*" in
  *--query-gpu*) cat "$here/gpus.csv" ;;
  *--query-compute-apps*) cat "$here/apps.csv" 2>/dev/null ;;
  *) exit 9 ;;
esac
"""
SELFTEST_PEER_ADDRESS = "192.0.2.62"   # 自证的多机配置里第二台的地址（文档用的地址段，不在任何一台的网卡上）
SELFTEST_SCOPE_NAME = "singlefs-memory-cap-selftest.scope"   # 假 slice 下那一个在跑的 scope（SelftestWorld.slice_reading）
SELFTEST_SCOPE_PROCESS = "4242"                               # 它里面那个进程在假 /proc 里的号
SELFTEST_RUN_LABEL = "task-selftest-62-1"                    # 那个进程环境里的尝试标签（memory_peaks_lib.py 文件头「在跑的 scope」）
FAKE_SSH = """#!/usr/bin/env bash
# 假 ssh：去掉 -o 选项与主机名，把命令照 ssh 那样交给 bash -c；第二台的 /proc/meminfo、slice 的 cgroup 目录、读进程环境的 /proc 与卡换成 peer/ 下的假的
while [[ "${1:-}" == -o ]]; do shift 2; done
printf '%s\\n' "$1" >> "$HOST_DISPATCH_SELFTEST_SSH_LOG"
shift
exec env SINGLEFS_DISPATCH_PROBE_MEMINFO="$HOST_DISPATCH_SELFTEST_PEER/meminfo" SINGLEFS_DISPATCH_PROBE_SLICE_DIRECTORY="$HOST_DISPATCH_SELFTEST_PEER/slice" SINGLEFS_DISPATCH_PROBE_PROC_DIRECTORY="$HOST_DISPATCH_SELFTEST_PEER/proc" PATH="$HOST_DISPATCH_SELFTEST_PEER/bin:$PATH" bash -c "$*"
"""


class SelftestWorld:
    """一格一个：临时目录里的本机、第二台与配置。"""

    def __init__(self, root, name):
        self.directory = os.path.join(root, name)
        self.local = os.path.join(self.directory, "local")
        self.peer = os.path.join(self.directory, "peer")
        for host in (self.local, self.peer):
            os.makedirs(os.path.join(host, "bin"))
            self.write_executable(os.path.join(host, "bin", "nvidia-smi"), FAKE_NVIDIA_SMI)
            self.cards(host, [(0, 16384, 16384)])
        # 两台都放假 cargo：多机配置判法双机开着时照 CARGO_BIN_DIRECTORY_BY_HOST 两台都查（这一台本地 test -x、第二台经假 ssh）
        for host in (self.local, self.peer):
            self.write_executable(os.path.join(host, "bin", "cargo"), "#!/usr/bin/env bash\nexit 0\n")
        os.makedirs(os.path.join(self.directory, "ssh-bin"))
        self.write_executable(os.path.join(self.directory, "ssh-bin", "ssh"), FAKE_SSH)
        self.meminfo(self.local, 60, 40)
        self.meminfo(self.peer, 60, 40)
        for host in (self.local, self.peer):
            self.slice_reading(host, 0)
        self.configuration(True)
        self.ssh_log = os.path.join(self.directory, "ssh.log")
        self.quiesce_state = os.path.join(self.directory, "quiesce-state")

    @staticmethod
    def write_executable(path, text):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
        os.chmod(path, 0o755)

    @staticmethod
    def meminfo(host, total_gibibytes, available_gibibytes):
        path = os.path.join(host, "meminfo")
        with open(path + ".next", "w", encoding="utf-8") as handle:
            handle.write(f"MemTotal: {total_gibibytes * 1024 * 1024} kB\nMemFree: 1 kB\nMemAvailable: {int(available_gibibytes * 1024 * 1024)} kB\n")
        os.replace(path + ".next", path)

    @staticmethod
    def slice_reading(host, margin_gibibytes):
        """这台的假 slice（<host>/slice，memory_peaks_lib 的 PROBE_SLICE_DIRECTORY_VARIABLE 指它）：总上限 50G，slice 自己挂着 margin_gibibytes 回收不掉的、
        下面一个在跑的 scope（SELFTEST_SCOPE_NAME）回收不掉 4G（不算进余量），它里面的进程（假 /proc，PROBE_PROC_DIRECTORY_VARIABLE 指 <host>/proc）的环境
        带着 multi-host-run.sh 设的这一趟本地目录、末段是 SELFTEST_RUN_LABEL。"""
        directory = os.path.join(host, "slice")
        peaks.fake_cgroup(directory, int((margin_gibibytes + 4) * GIBIBYTE))
        peaks.fake_cgroup(os.path.join(directory, SELFTEST_SCOPE_NAME), 4 * GIBIBYTE)
        with open(os.path.join(directory, SELFTEST_SCOPE_NAME, "cgroup.procs"), "w", encoding="utf-8") as handle:
            handle.write(f"{SELFTEST_SCOPE_PROCESS}\n")
        os.makedirs(os.path.join(host, "proc", SELFTEST_SCOPE_PROCESS), exist_ok=True)
        with open(os.path.join(host, "proc", SELFTEST_SCOPE_PROCESS, "environ"), "wb") as handle:
            handle.write(f"{peaks.RUN_LABEL_VARIABLE}={SELFTEST_RUN_LABEL}\0".encode("ascii"))
        with open(os.path.join(directory, "memory.max"), "w", encoding="utf-8") as handle:
            handle.write(f"{50 * GIBIBYTE}\n")

    @staticmethod
    def cards(host, cards):
        with open(os.path.join(host, "bin", "gpus.csv"), "w", encoding="utf-8") as handle:
            handle.write("".join(f"{index}, {total}, {free}\n" for index, total, free in cards))

    def configuration(self, across_machines, peer_cap="40G"):
        with open(os.path.join(self.directory, "multi-host.env"), "w", encoding="utf-8") as handle:
            # 主机数组（判法文件头「主机数组」）：127.0.0.1 在每一台的网卡上，是自己；SELFTEST_PEER_ADDRESS（文档用的地址段）是第二台
            #   按台的三个键（multi-host.env.example）：这一台那一项写在头一项、与第二台的不同（50G 对 40G），Peer 按地址取第二台那一项
            handle.write(f"HOSTS=127.0.0.1 {SELFTEST_PEER_ADDRESS}\nBUILD_ROOT_BY_HOST=127.0.0.1={self.local}/home {SELFTEST_PEER_ADDRESS}={self.peer}/home\n"
                         f"CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1={self.local}/bin {SELFTEST_PEER_ADDRESS}={self.peer}/bin\n"
                         f"SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=50G {SELFTEST_PEER_ADDRESS}={peer_cap}\nQUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\n"
                         f"QUIESCE_STARTED_CHECK_COMMAND=\nENABLE_ACROSS_MACHINES={1 if across_machines else 0}\nENABLE_GPU=0\n"
                         f"NFS_SHARED_ROOTS={self.directory}\nSHARED_DATA_ROOT={self.directory}/data-root\n")

    def quiesce_holder(self, process_id, start_time):
        os.makedirs(os.path.join(self.quiesce_state, "holders"), exist_ok=True)
        with open(os.path.join(self.quiesce_state, "holders", f"{process_id}-{start_time}"), "w", encoding="utf-8"):
            pass

    def settings(self):
        environment = {name: value for name, value in os.environ.items()
                       if not name.startswith(("RUN_WITH_MEMORY_CAP_", "SINGLEFS_", "GPU_QUIESCE_", "HOST_DISPATCH_", "MULTI_HOST_", "PEER_HOST_LIB_",
                                               "CLAUDE_CODE_SESSION_ID"))}
        environment.update({
            "PATH": os.pathsep.join([os.path.join(self.local, "bin"), os.path.join(self.directory, "ssh-bin"), os.environ.get("PATH", "")]),
            "SINGLEFS_DISPATCH_PROBE_MEMINFO": os.path.join(self.local, "meminfo"),
            peaks.PROBE_SLICE_DIRECTORY_VARIABLE: os.path.join(self.local, "slice"),
            peaks.PROBE_PROC_DIRECTORY_VARIABLE: os.path.join(self.local, "proc"),
            "RUN_WITH_MEMORY_CAP_SLICE": "singlefs_host_dispatch_selftest_absent.slice",
            "RUN_WITH_MEMORY_CAP_RESERVE": "20G",
            "SINGLEFS_MULTI_HOST_CONFIG": os.path.join(self.directory, "multi-host.env"),
            "HOST_DISPATCH_SELFTEST_SSH_LOG": self.ssh_log,
            "HOST_DISPATCH_SELFTEST_PEER": self.peer,
        })
        return Settings(
            repository_root=os.path.dirname(os.path.dirname(SCRIPT_DIRECTORY)),
            configuration_check=os.path.join(SCRIPT_DIRECTORY, "..", "crash", "crash-point-replay-shard-configuration-check.sh"),
            peer_host_lib=os.path.join(SCRIPT_DIRECTORY, "peer-host-lib.sh"), quiesce_state_directory=self.quiesce_state,
            environment=environment, say=lambda line: None)

    def ssh_calls(self):
        with contextlib.suppress(FileNotFoundError):
            with open(self.ssh_log, encoding="utf-8") as handle:
                return handle.read().splitlines()
        return []


SELFTEST_MARGIN_BYTES = 2 * GIBIBYTE   # 判法那几格喂给 judge_host 的这台此刻的余量（另写了数的那几格照它自己的）


def memory_report(free_gibibytes, slice_total_gibibytes, mem_total_gibibytes=60, slice_used_gibibytes=0):
    """judge_host 读的那几样（memory_peaks_lib.headroom 交的 dict 的写法；余量另作 judge_host 的参数交，各格写明喂的数）。"""
    slice_total = int(slice_total_gibibytes * GIBIBYTE)
    slice_used = int(slice_used_gibibytes * GIBIBYTE)
    return {"memory": {"mem_total": mem_total_gibibytes * GIBIBYTE, "mem_available": int(free_gibibytes * GIBIBYTE), "free": int(free_gibibytes * GIBIBYTE),
                       "slice_total": slice_total, "slice_used": slice_used, "slice_room": slice_total - slice_used}}


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

    key = peaks.command_key(["selftest-command", "--flag"])

    one_gibibyte = Prediction(key, GIBIBYTE, 1536 * MEBIBYTE, GIBIBYTE, "peak")
    # local-fits：空闲 40G、slice 40G、余量 2G ⇒ 1G 放得下（要的空闲 = min(1G + 2G, 上限 1.5G) = 1.5G）；空闲 1G 比上限 1.5G 小 ⇒ 此刻放不下、
    #   不是永远放不下、差多少写在 memory_short（不加余量的写法下要的是 1G、空闲 1G 就放下了，这一格分得出）
    roomy = judge_host("local", memory_report(40, 40), one_gibibyte, False, 0, 0, None, SELFTEST_MARGIN_BYTES)
    tight = judge_host("local", memory_report(1, 40), one_gibibyte, False, 0, 0, None, SELFTEST_MARGIN_BYTES)
    check("local-fits", roomy.fits and not roomy.never and roomy.cap == 1536 * MEBIBYTE and not tight.fits and not tight.never and tight.memory_short > 0,
          f"空闲 40G 应当放得下（上限 1536M），空闲 1G 比要的 1.5G（1G 加余量 2G，不超过上限 1.5G）小应当此刻放不下、差的写在 memory_short、不是永远放不下；实际 {roomy}、{tight}")

    # pending-subtracted：空闲 3G，服务账上这台还没涨到量的 2G ⇒ 剩 1G，比要的 1.5G（不超过上限）小、放不下；不减的写法下 3G 放得下
    pending = judge_host("local", memory_report(3, 40), one_gibibyte, False, 2 * GIBIBYTE, 0, None, SELFTEST_MARGIN_BYTES)
    check("pending-subtracted", not pending.fits and not pending.never,
          f"空闲 3G 减服务账上还没涨到量的 2G 之后剩 1G，要的 1.5G（1G 加余量 2G，不超过上限 1.5G）应当放不下（不是永远放不下）；实际 {pending}")

    # never-fits：记账 64G 比 slice 总上限 50G 大 ⇒ 永远放不下
    huge = Prediction(key, 64 * GIBIBYTE, 96 * GIBIBYTE, 64 * GIBIBYTE, "peak")
    never = judge_host("local", memory_report(40, 50, mem_total_gibibytes=128), huge, False, 0, 0, None, SELFTEST_MARGIN_BYTES)
    check("never-fits", not never.fits and never.never, f"记账 64G 比 slice 总上限 50G 大应当判永远放不下；实际 {never}")

    # configured-cap-clamp：峰值 30G（上限 45G），那台另收到的数 40G ⇒ 上限收到 40G；记账 41G 的比它大 ⇒ 永远放不下（两台同一个判法：一条照本机判、一条照第二台判）
    thirty = Prediction(key, 30 * GIBIBYTE, 45 * GIBIBYTE, 30 * GIBIBYTE, "peak")
    over = Prediction(key, 41 * GIBIBYTE, 62 * GIBIBYTE, 41 * GIBIBYTE, "peak")
    clamped = judge_host("local", memory_report(60, 60, mem_total_gibibytes=80), thirty, False, 0, 0, 40 * GIBIBYTE, SELFTEST_MARGIN_BYTES)
    refused = judge_host("peer", memory_report(60, 60, mem_total_gibibytes=80), over, False, 0, 0, 40 * GIBIBYTE, SELFTEST_MARGIN_BYTES)
    check("configured-cap-clamp", clamped.fits and clamped.cap == 40 * GIBIBYTE and not refused.fits and refused.never,
          f"上限应当收到那台另收到的数 40G、记账 41G 比它大应当永远放不下（本机与第二台同一个判法）；实际 {clamped}、{refused}")

    # slice-cap-clamp：峰值 40G（上限 60G、记账 40G），本机 slice 总上限 50G、这台此刻的余量 3G ⇒ 放得下、上限收到 50G 减 3G＝47G（照交进来的余量减，不是常量）
    forty = Prediction(key, 40 * GIBIBYTE, 60 * GIBIBYTE, 40 * GIBIBYTE, "peak")
    sliced = judge_host("local", memory_report(55, 50, mem_total_gibibytes=64), forty, False, 0, 0, None, 3 * GIBIBYTE)
    check("slice-cap-clamp", sliced.fits and sliced.cap == 47 * GIBIBYTE,
          f"上限 60G 比 slice 总上限 50G 大，应当收到总上限减这台此刻的余量 3G＝47G；实际 {sliced}")

    # margin-not-over-cap：预测 48G（顶格：slice 总上限 50G 减余量 2G），此刻空闲 48G ⇒ 放得下：要的空闲量不超过这一条的上限 48G，不再另加余量
    at_cap = Prediction(key, 48 * GIBIBYTE, 48 * GIBIBYTE, 48 * GIBIBYTE, "peak")
    topped = judge_host("local", memory_report(48, 50, mem_total_gibibytes=60), at_cap, False, 0, 0, None, SELFTEST_MARGIN_BYTES)
    check("margin-not-over-cap", topped.fits and topped.cap == 48 * GIBIBYTE,
          f"顶格 48G 的任务在空闲 48G、slice 总上限 50G、余量 2G 的台上应当放得下（要的空闲量不超过上限 48G）；实际 {topped}")

    # margin-unmeasured-refused：这台此刻量不到余量（None）⇒ 此刻放不下、不是永远放不下、why 写量不到；不退回固定的数（退回 2G 的写法下空闲 40G 放得下）
    unmeasured = judge_host("local", memory_report(40, 40), one_gibibyte, False, 0, 0, None, None)
    check("margin-unmeasured-refused", not unmeasured.fits and not unmeasured.never and "量不到余量" in unmeasured.why,
          f"余量量不到时应当此刻放不下、不是永远放不下、why 写量不到余量；实际 {unmeasured}")

    # total-within-margin-refused：slice 总上限 2G、这台此刻的余量 2G ⇒ 一条都给不出上限：此刻放不下、不是永远放不下（照收到 0 往下判的写法下要的空闲是 0、放得下）
    no_room = judge_host("local", memory_report(40, 2), one_gibibyte, False, 0, 0, None, 2 * GIBIBYTE)
    check("total-within-margin-refused", not no_room.fits and not no_room.never and "不比此刻的余量" in no_room.why,
          f"slice 总上限 2G 不比余量 2G 大时应当此刻放不下、不是永远放不下、why 写总上限不比余量大；实际 {no_room}")

    # slice-room-keeps-margin：slice 总上限 50G、进程已用 30G（slice 放得下 20G）、这台此刻的余量 1G、空闲 100G：记账 20G 的放不下（它加 slice 自己挂着的余量 1G
    #   超过 slice 放得下的 20G：用着的 30G 连同它超过总上限减余量 49G），记账 19G 的放得下；不减余量的写法下 20G 的也放得下
    margin_room = memory_report(100, 50, mem_total_gibibytes=128, slice_used_gibibytes=30)
    twenty = Prediction(key, 20 * GIBIBYTE, 20 * GIBIBYTE, 20 * GIBIBYTE, "peak")
    nineteen = Prediction(key, 19 * GIBIBYTE, 19 * GIBIBYTE, 19 * GIBIBYTE, "peak")
    over_room = judge_host("local", margin_room, twenty, False, 0, 0, None, GIBIBYTE)
    within_room = judge_host("local", margin_room, nineteen, False, 0, 0, None, GIBIBYTE)
    check("slice-room-keeps-margin", not over_room.fits and not over_room.never and within_room.fits,
          f"slice 放得下 20G、余量 1G：记账 20G 的应当此刻放不下（不是永远放不下）、记账 19G 的放得下；实际 {over_room}、{within_room}")

    # gpu-fit：卡总量 16384、空闲 4000、清场登记里有活着的 ⇒ 按空闲算，8192 放不下；没有活着的 ⇒ 按总量算放得下
    gpu_prediction = Prediction(key, GIBIBYTE, 1536 * MEBIBYTE, GIBIBYTE, "peak", gpu_predicted=8192, gpu_history="default")
    busy_cards = dict(memory_report(40, 40), gpu={"cards": [{"index": 0, "total": 16384, "free": 4000}]}, quiesce_holders=1)
    gpu_busy = judge_host("local", busy_cards, gpu_prediction, True, 0, 0, None, SELFTEST_MARGIN_BYTES)
    check("gpu-fit", not gpu_busy.fits and not gpu_busy.never and gpu_busy.gpu_short == 8192 - (4000 - GPU_MARGIN_MEBIBYTES),
          f"清场登记里有活着的、卡空闲 4000 MiB 时显存预测 8192 应当放不下、差 {8192 - (4000 - GPU_MARGIN_MEBIBYTES)}；实际 {gpu_busy}")

    # gpu-total-before-quiesce：没有活着的清场登记（还没清场），卡被服务占着只空 1000M、总量 32768M ⇒ 按总量算、放得下
    unquiesced = dict(memory_report(40, 40), gpu={"cards": [{"index": 0, "total": 32768, "free": 1000}]}, quiesce_holders=0)
    gpu_unquiesced = judge_host("local", unquiesced, dataclass_replace(gpu_prediction, gpu_predicted=3072), True, 0, 0, None, SELFTEST_MARGIN_BYTES)
    check("gpu-total-before-quiesce", gpu_unquiesced.fits, f"还没清场时应当按卡的总量算、显存预测 3072 放得下；实际 {gpu_unquiesced}")

    # probe-local：本机的余量照 /proc/meminfo（假的）与卡（假 nvidia-smi）报；不读任何账（free 就是 MemAvailable）
    #   内存的余量照本机的假 slice 量：slice 自己挂着 0.7G（在跑的 scope 的 4G 不算）⇒ 向上取整 1G
    world = SelftestWorld(root, "probe-local")
    world.meminfo(world.local, 60, 12)
    world.cards(world.local, [(0, 16384, 9000), (1, 24576, 20000)])
    world.slice_reading(world.local, 0.7)
    report, problem = probe_local(world.settings(), True)
    cards = (report or {}).get("gpu", {}).get("cards")
    local_margin = report_margin(report["memory"]) if report is not None else (None, "没报告")
    check("probe-local", report is not None and not problem and report["memory"]["free"] == 12 * GIBIBYTE and report["memory"]["mem_total"] == 60 * GIBIBYTE
          and cards == [{"index": 0, "total": 16384, "free": 9000}, {"index": 1, "total": 24576, "free": 20000}] and local_margin == (GIBIBYTE, ""),
          f"本机余量应当照假 meminfo 报 free 12G、MemTotal 60G、两张卡照假 nvidia-smi 报、余量照假 slice 量 0.7G 记 1G；实际 {report}、余量 {local_margin}、问题「{problem}」")

    # probe-peer：双机开着 ⇒ Peer 用得上、上限读成 SLICE_MEMORY_CAP_BY_HOST 里第二台那一项 40G（这一台那一项 50G 写在头一项，不取它）；
    #   经假 ssh 在「第二台」跑 headroom，free 照第二台的假 meminfo；输出里没有配置的主机名
    #   余量在第二台上照它自己的假 slice 量（2.1G ⇒ 3G），本机的假 slice 是 0.7G（⇒ 1G）：两台各量各的，第二台那一份不是本机的
    world = SelftestWorld(root, "probe-peer")
    world.meminfo(world.peer, 80, 33)
    world.slice_reading(world.local, 0.7)
    world.slice_reading(world.peer, 2.1)
    settings = world.settings()
    peer = Peer(settings)
    peer_report, peer_problem = probe_peer(peer, False) if peer.available else (None, f"Peer 用不上：{peer.why}")
    peer_margin = report_margin(peer_report["memory"]) if peer_report is not None else (None, "没报告")
    peer_scopes = (peer_report or {}).get("memory", {}).get(peaks.SCOPES_REPORT_KEY)
    wanted_scopes = {SELFTEST_SCOPE_NAME: {peaks.SCOPE_USED_KEY: 4 * GIBIBYTE, peaks.SCOPE_LABEL_KEY: SELFTEST_RUN_LABEL}}
    check("probe-peer", peer.available and peer.memory_cap_bytes == 40 * GIBIBYTE and peer_report is not None
          and peer_report["memory"]["free"] == 33 * GIBIBYTE and peer_report["memory"]["mem_total"] == 80 * GIBIBYTE and world.ssh_calls()
          and SELFTEST_PEER_ADDRESS not in peer_problem and peer_margin == (3 * GIBIBYTE, "") and peer_scopes == wanted_scopes,
          f"双机开着时 Peer 应当用得上、上限取第二台那一项 40G，第二台余量照它的假 meminfo 报 free 33G、MemTotal 80G、经 ssh 去查，余量照第二台的假 slice 量 2.1G 记 3G，"
          f"同一次报回第二台那个在跑的 scope 的用量与标签 {wanted_scopes}；"
          f"实际 available {peer.available}（{peer.why}）、上限 {peer.memory_cap_bytes}、报告 {peer_report}、余量 {peer_margin}、问题「{peer_problem}」、ssh {len(world.ssh_calls())} 次")

    # probe-program-large：探测程序长过单个参数的界（照源码行写成注释，压缩比与真源码相近），压完再送、第二台照样量得到
    #   （弄坏开关 probe-program-uncompressed 下照改前整份 base64 送，过界就查不成，这一格红）
    world = SelftestWorld(root, "probe-program-large")
    world.meminfo(world.peer, 80, 33)
    world.slice_reading(world.local, 0.7)
    world.slice_reading(world.peer, 2.1)
    original_program = globals()["peer_probe_program"]
    original_lines = original_program().splitlines()
    padding = "\n".join("# " + line for line in original_lines[:len(original_lines) * 2 // 3])
    globals()["peer_probe_program"] = lambda: original_program() + "\n" + padding
    try:
        large_size = len(peer_probe_program().encode("utf-8"))
        peer = Peer(world.settings())
        large_report, large_problem = probe_peer(peer, False) if peer.available else (None, f"Peer 用不上：{peer.why}")
    finally:
        globals()["peer_probe_program"] = original_program
    check("probe-program-large", large_size > 128 * 1024 and large_report is not None and large_report["memory"]["free"] == 33 * GIBIBYTE,
          f"探测程序 {large_size} 字节（过单个参数的界 128 KiB）时压完再送、第二台照样报 free 33G；实际报告 {large_report}、问题「{large_problem}」")

    # switch-off-peer-unavailable：双机开关没开 ⇒ Peer 用不上、why 写双机开关、一次都不连第二台
    world = SelftestWorld(root, "switch-off-peer-unavailable")
    world.configuration(False)
    peer = Peer(world.settings())
    check("switch-off-peer-unavailable", not peer.available and "ENABLE_ACROSS_MACHINES" in peer.why and not world.ssh_calls(),
          f"双机开关没开时 Peer 应当用不上、why 点名 ENABLE_ACROSS_MACHINES、不连第二台；实际 available {peer.available}、why「{peer.why}」、ssh {len(world.ssh_calls())} 次")

    # quiesce-holders：清场登记里一条活着的（这个自证自己的进程号与起始时刻）、一条进程不在的（pid_max）⇒ 数 1
    world = SelftestWorld(root, "quiesce-holders")
    world.quiesce_holder(os.getpid(), peaks.process_start_time(os.getpid()))
    with open("/proc/sys/kernel/pid_max", encoding="utf-8") as handle:
        world.quiesce_holder(int(handle.read()), "1")
    count = live_quiesce_holders(world.settings())
    check("quiesce-holders", count == 1, f"清场登记里一条活着、一条进程不在，应当数 1；实际 {count}")

    # unknown-argument-refused：认不出的参数退 2，出路点名 --selftest 与 task-server/task submit，带的命令没跑
    marker = os.path.join(root, "unknown-argument-ran")
    completed = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--no-such-option-ever-used", "bash", "-c", f": > {marker}"], capture_output=True, text=True,
                               env={name: value for name, value in os.environ.items() if name != "HOST_DISPATCH_BREAK"}, timeout=60, stdin=subprocess.DEVNULL)
    check("unknown-argument-refused", completed.returncode == EXIT_USAGE and "--selftest" in completed.stderr and "task-server/task submit" in completed.stderr and not os.path.exists(marker),
          f"认不出的参数应当退 {EXIT_USAGE}、出路点名 --selftest 与 task-server/task submit、带的命令没跑；实际退 {completed.returncode}、"
          f"命令{'跑了' if os.path.exists(marker) else '没跑'}：{completed.stderr.strip()[-300:]}")
    return failures, checked


def dataclass_replace(instance, **changes):
    import dataclasses
    return dataclasses.replace(instance, **changes)


SWITCH_EXPECTED_CELLS = {
    "ignore-pending": ["pending-subtracted"],
    "never-ignored": ["never-fits"],
    "no-configured-cap-clamp": ["configured-cap-clamp"],
    "no-slice-cap-clamp": ["slice-cap-clamp"],
    "margin-over-cap": ["margin-not-over-cap"],
    "ignore-gpu": ["gpu-fit"],
    "free-only": ["gpu-total-before-quiesce"],
    "peer-despite-switch-off": ["switch-off-peer-unavailable"],
    "stale-holders-counted": ["quiesce-holders"],
    "margin-fallback-fixed": ["margin-unmeasured-refused"],
    "total-within-margin-accepted": ["total-within-margin-refused"],
    "slice-room-ignores-margin": ["slice-room-keeps-margin"],
    "probe-program-uncompressed": ["probe-peer", "probe-program-large"],
}


def run_cells_in_scratch():
    root = tempfile.mkdtemp(prefix="host-dispatch-selftest-")
    try:
        return selftest_cells(root)
    finally:
        shutil.rmtree(root, ignore_errors=True)


def selftest():
    failures, checked = run_cells_in_scratch()
    if failures:
        print(f"  ✗ host-dispatch.py 自证 {len(failures)} 格不对（查了 {checked} 格）：{'、'.join(failures)}")
        print("  → 怎么办：照上面逐格改 judge_host / probe_local / probe_peer / Peer / live_quiesce_holders；HOST_DISPATCH_BREAK 设着的话本来就该红")
        return 1
    # 弄坏开关轮：每个开关起一个子进程只跑各格，核它红在点名的格
    jobs = max(1, min(len(SWITCH_EXPECTED_CELLS), int(os.environ.get("SINGLEFS_THREAD_CAP") or 0) or os.cpu_count() or 1))

    def one(switch):
        completed = subprocess.run([sys.executable, "-B", os.path.realpath(__file__), "--selftest-cells"], capture_output=True, text=True,
                                   env=dict(os.environ, HOST_DISPATCH_BREAK=switch), timeout=600, stdin=subprocess.DEVNULL)
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
    print(f"  ✓ host-dispatch.py 自证通过（查了 {checked} 格：放得下与此刻放不下、减服务账上还没涨到量的、记账比总上限大判永远放不下、上限收到交进来的那台另收到的数（两台同一个判法）、"
          f"上限收到总上限减交进来的余量、余量量不到与总上限不比余量大都此刻放不下、slice 放得下的另减余量、清场之后按空闲算显存、还没清场按卡的总量算、"
          f"本机与第二台的余量照假 meminfo、假 slice 与假卡报（两台各量各的，第二台同一次报回在跑的 scope 的用量与标签）、双机关着第二台用不上、清场登记只数活着的、认不出的参数退 {EXIT_USAGE}；"
          f"弄坏开关 {len(results)} 个逐个打开都红在点名的格）")
    return 0


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    if sys.argv[1:] == ["--selftest-cells"]:
        cell_failures, _ = run_cells_in_scratch()
        sys.exit(1 if cell_failures else 0)
    if os.environ.get("HOST_DISPATCH_BREAK"):
        print("  ✗ 弄坏开关 HOST_DISPATCH_BREAK 设在 --selftest 之外，命令一行都没跑", file=sys.stderr)
        print("  → 怎么办：去掉 HOST_DISPATCH_BREAK 再跑；它只给 host-dispatch.py --selftest 证红用", file=sys.stderr)
        sys.exit(EXIT_USAGE)
    print(f"  ✗ host-dispatch.py 认不出的参数：{' '.join(sys.argv[1:]) or '（没给参数）'}；这一份只给 task-server 载入，单独跑只认 --selftest，命令一行都没跑", file=sys.stderr)
    print("  → 怎么办：要自证跑 host-dispatch.py --selftest；要跑项目代码的命令，task-server/task submit -- <命令…>（不带内存、线程、机器参数）", file=sys.stderr)
    sys.exit(EXIT_USAGE)
