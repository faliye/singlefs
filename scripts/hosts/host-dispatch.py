#!/usr/bin/env python3
# admission: always 每一次派都现查各台此刻的余量与这一条的历史峰值，上一次派到哪台不替这一次作保
# run-condition: command bash python3 git
"""统一派发器：一条命令派到哪台机器、给多少内存上限，按「这台此刻放不放得下它预测要的量」定（用户 2026-10-06 定，
records/2026-09-16-subagent拆分提案.md 第五十五节最后两条、第五十六节最后一条）。全局只守一件事：不让哪台机器的内存或显存 OOM。

用法：
  host-dispatch.py [--gpu] [--label <标签>] [--fetch <树里的相对路径>]… [--shard <k>/<n>] [--estimated-hours <小时>] -- <命令…>
  host-dispatch.py --plan [--gpu] [--shard <k>/<n>] [--estimated-hours <小时>] -- <命令…>
                                                  只查、只打这一刻会派到哪台（不派、不等、不记账）；退 0 放得下，1 此刻放不下（会等），241 哪台都放不下，244 长活只派一台
  host-dispatch.py --selftest                     假的 /proc/meminfo、假 nvidia-smi、假 ssh、假峰值表与假的派活脚本走一遍各格，末尾逐个打开弄坏开关核它红在点名的格
  --shard <k>/<n>：这一趟是调用方切好的 n 片里的第 k 片（1 ≤ k ≤ n），每片各派一次；--estimated-hours：调用方给的预计挂钟（小时，可带小数），没有历史用时时按它判 ⓪。

⓪ 长活不单机派（records/2026-09-16-subagent拆分提案.md 第五十九节第 5 条）：在 ① 读完多机配置之后、②–④ 之前判，判法 long_run_check（scripts/host-resources.py 用同一份）。
     用时：用时表（memory_peaks_lib.py 文件头「用时表」，键同峰值表）里这条命令上一次的用时；没有这条就用 --estimated-hours；两样都没有不判。
     派法占几台（hosts_of_dispatch）：
       给了 --shard <k>/<n>、n ≥ 2，而此刻能派的机器至少两台（本机与多机配置开着、判得过的第二台）⇒ 多台；
       否则读被起脚本文件头的长活声明（四个键 # long-run-shards: / # long-run-resume: / # long-run-hosts: / # long-run-unshardable:）。
         找脚本、读文件头、判齐不齐都用 .claude/hooks/lib_long_run.py 那一份（started_script、script_header_declaration、judge；
         scripts/detached-run.sh 的 start --wall-clock over-2h 与 long-run-check 判的也是它），被起的脚本按仓根找：
         judge 判齐了、shards 与 hosts 两键都写了 ⇒ 多台（它自己分片、按多机配置派到几台）；
         judge 判齐了、缺的那几样由 unshardable 一行点名 ⇒ 分不了，放行，打一行理由（例 layer0-shard-run.sh、crash-amplification.sh、broad-mutation.py：台数与额度不按 --emit-hosts 与 host-resources.py 现算）；
         judge 判不齐（一行都没写、半截声明、hosts 一行没点名两份现查或写死额度）⇒ 只占一台，判出的问题写进理由。
       LIB_LONG_RUN_BREAK（那份库的弄坏开关）派发器不认：判之前从环境里摘掉、判完放回（派出去的命令照旧拿得到）。
       别的 ⇒ 只占一台。
     用时超过 LONG_RUN_SECONDS（2 小时）而只占一台 ⇒ 不派，退 244，出路：改成按 layer0-shard-configuration-check.sh --emit-hosts 与 host-resources.py --json
       现算的两台分片（每片各派一次，带 --shard <k>/<n>），或在被起脚本的文件头补齐长活声明（分不了的写 # long-run-unshardable: <缺哪一段、为什么分不了>）。

五步：
  ① 机器：本机总在；第二台从多机配置读（仓根 multi-host.env，不进仓；判法 scripts/layer0-shard-configuration-check.sh，读法 scripts/peer-host-lib.sh
     的 peer_host_load_configuration）。配置里双机开关 ENABLE_ACROSS_MACHINES 没开（判法退 3）就只有本机；配置判不过同样只有本机，打一行原因。
  ② 现查每台此刻的余量（每一轮都重查，不按声明的上限记账）：
     内存：/proc/meminfo 的 MemAvailable 减去那台内存包装账上在跑的、还没涨到预测峰值的那一截（memory_peaks_lib.headroom，算法同包装的排队），
       另减派发器自己账上「派出去还不到 LAUNCH_GRACE_SECONDS 秒」的几条按预测记的量（那几条的包装可能还没记账）；
       同时看包装的 slice 此刻还放得下多少（slice 总上限 − 回收不掉的 − 还没涨到量的），挑中了包装那边不会再排队。
     显存（带 --gpu）：nvidia-smi 各张卡的 memory.total / memory.free 相加。本机清场包装（gpu-quiesce-run.sh）的登记（holders/）里没有活着的，
       清场会把白名单里的服务停掉，按卡的总量算；有活着的（卡已经清过场、别的 --gpu 那一趟在用），按此刻的空闲算；
       再减 GPU_MARGIN_MEBIBYTES 与派发器账上别的 --gpu 那几趟预测了而还没用到的那一截。
     第二台经 ssh 读：把 memory_peaks_lib.py 的源码与本文件里取显存的两个函数送过去（python3 -c，base64 编码），用 peer-host-lib.sh 的 peer_host_run 跑。
  ③ 按历史预测这一条要多少：
     内存：峰值表 scripts/memory-peaks.tsv（RUN_WITH_MEMORY_CAP_PEAKS 可换），键与内存包装同一份取法（memory_peaks_lib.command_key，
       RUN_WITH_MEMORY_CAP_KEY 给了就用它）。有真峰值用峰值；撞过顶的（记的峰值到了那一次的上限）按记的上限 × CAP_FACTOR；查不到用默认值
       （memory_peaks_lib.DEFAULT_MEMORY_PREDICTION_TEXT，8G）。
     显存：派发器自己记的显存峰值表（默认 ${XDG_STATE_HOME:-~/.local/state}/singlefs/gpu-memory-peaks.tsv，跨重启、不进仓；
       HOST_DISPATCH_GPU_PEAKS 可换），键同内存。查不到用 DEFAULT_GPU_PREDICTION_MEBIBYTES。带 --gpu 跑完把量到的显存峰值写进去（见 ⑤）。
  ④ 挑一台：本机在前、第二台在后，挑第一台「预测量 + 余量放得下」的：内存 预测 + MEMORY_MARGIN_BYTES ≤ 那台此刻的空闲，且包装要记的量 ≤ slice 此刻放得下的
     （包装记的量：有真峰值记峰值，没有记上限，见 run-with-memory-cap.sh 文件头「排队」）；带 --gpu 再要 显存预测 ≤ 那台显存此刻放得下的。
     上限 = 预测 × CAP_FACTOR（按 MiB 向上取整）；第二台的上限不超过配置的 PEER_MEMORY_CAP（包装记的量比它还大，第二台算放不下）。
     两台都放不下就等：每 POLL_SECONDS 秒重查一遍（②③ 都重做），第一次等与之后每 WAITING_REPORT_EVERY_SECONDS 秒往 stderr 报一行在等什么；
     HOST_DISPATCH_WAIT_SECONDS 给了就至多等那么久，到点退 242，不给就一直等到放得下。
     哪台都等多久也放不下（包装要记的量比每台的 slice 总上限都大、或预测加余量比每台的整机内存都大、或显存预测比每台卡的总量减余量都大）当场退 241。
     判与记账在同一把锁里（状态目录 HOST_DISPATCH_STATE_DIR，默认 ${XDG_RUNTIME_DIR:-/tmp}/singlefs-host-dispatch），几条同时来的一条一条判。
  ⑤ 派：本机 bash scripts/multi-host-run.sh --memory-cap <上限> --label <标签> [--fetch …] -- <命令…>（不带 --peer：在本机仓根经内存包装
     scripts/run-with-memory-cap.sh 起）；第二台同一条带 --peer（拷树过去经那边的内存包装起，--fetch 点名的取回本机）。
     带 --gpu 的整条套在清场包装外面：bash scripts/gpu-quiesce-run.sh -- <上面那条>（清场命令照多机配置在本机跑、两台的卡都管）。
     清场包装是另一个工具实现员在做的；这里只照它文件头写的接口调（参数只有 -- <命令…>，退出码交回命令的，78 没起；跑完不复原），不改它。
     跑的时候按 GPU_SAMPLE_SECONDS（第二台 GPU_SAMPLE_SECONDS_PEER）秒读一次那台卡上的计算进程（nvidia-smi --query-compute-apps），
     派之前就在的进程不算，之后冒出来的那几个的显存相加，取最大的记成这一条的显存峰值（偏大不偏小：同一段时间别人新起的也算进来）；
     命令退 245（内存包装认出的显存不够）记成撞过顶（记预测与量到的大的那个，下一次按它 × CAP_FACTOR）；一个计算进程都没看到的不记。
     内存峰值由内存包装自己记；派到第二台的那一次记在第二台那棵临时树里、随树删掉，本机的峰值表不更新（只有本机跑的那几次进表）。
     用时：派活那条链从起到退出的挂钟秒数记进用时表（memory_peaks_lib.record_duration）；那条链退 2、251、252、255（命令没跑成），
     或派发器收到了 TERM / INT / HUP（跑到一半）的不记。
  收到 TERM / INT / HUP：只转给自己起的那一个子进程（multi-host-run.sh 或清场包装，它们各自停命令；清场包装不复原），等它退出再退。

输出：stderr 上逐步一行 HOST_DISPATCH step=<probe|decide|wait|plan|exit> …（字段 key=value，空格隔开），第二台写成「peer」，不写配置里的主机名与目录。
退出码：派出去之后是那条链的退出码（multi-host-run.sh 的：命令的、2 用法错、255 它自己没跑成；内存包装的 245、250–254；清场包装的 78）；
  2 用法错（命令一行都没跑）；241 哪台都等多久也放不下；242 等满 HOST_DISPATCH_WAIT_SECONDS 还放不下；243 本机的余量都查不成（/proc/meminfo 读不了）；
  244 长活只派一台（⓪：历史用时或预计挂钟超过 2 小时而派法只占一台，命令一行都没跑；出路见 ⓪）。

常量的依据：
  CAP_FACTOR 1.5：与内存包装 auto 取实测峰值 1.5 倍同一个数。
  MEMORY_MARGIN_BYTES 2 GiB：派出去到包装记账之间别的进程可能涨的那一截、页缓存抖动与包装外壳；推的，没量过。
  GPU_MARGIN_MEBIBYTES 2560：判器留给驱动的那一份（crates/singlefs-checker-tier/src/crash_judge_dispatch.rs 的 MEBIBYTES_LEFT_FOR_THE_DRIVER）。
  DEFAULT_GPU_PREDICTION_MEBIBYTES 8192：最小那张卡（16G）的一半；推的，没量过。
  LAUNCH_GRACE_SECONDS 30：multi-host-run.sh 本机模式起包装到包装记账一般几秒；第二台要拷树，包装在那边记账，这一段按 30 秒算，推的。

弄坏开关 HOST_DISPATCH_BREAK=<项>（只给 --selftest 证红用，逗号连几个；在 --selftest 之外设了就拒、退 2）。项名、弄坏哪一处、打开后红在哪几格
  （格名是「✗ 自证 <格名>：」里的那一段；末尾那一轮照 SWITCH_EXPECTED_CELLS 逐个核）：
  local-only            不看第二台                                  peer-when-local-full、pending-subtracted、gpu-fit、peer-cap-clamp
  peer-first            第二台排在本机前面                          local-fits
  no-wait               两台都放不下时照样派到本机                  waits-then-dispatches、switch-off-local-only
  no-reprobe            等的时候不重查，一直用第一次查到的          waits-then-dispatches
  ignore-history        不看峰值表，一律用默认值                    prediction-from-history、hit-history
  hit-as-peak           撞过顶的键当真峰值                          hit-history
  no-default            查不到历史按 0 算                           default-prediction
  ignore-pending        不减包装账上还没涨到量的那一截              pending-subtracted
  book-by-cap           派发器自己的账按上限记（不按预测）          booking-sum-fits
  no-dispatch-ledger    不减派发器自己账上刚派出去的那几条          booking-overflow
  ignore-gpu            带 --gpu 也不看显存                         gpu-fit
  no-quiesce            带 --gpu 不套清场包装                       gpu-fit
  free-only             没有活着的清场登记时也只按卡此刻的空闲算    gpu-total-before-quiesce
  no-gpu-record         跑完不写显存峰值表                          gpu-record、gpu-oom-marks-hit
  wait-on-hopeless      哪台都放不下也照样等                        never-fits
  exit-dropped          派出去那条链退几都退 0                      exit-passthrough
  no-peer-cap-clamp     第二台的上限不按 PEER_MEMORY_CAP 收         peer-cap-clamp
  peer-despite-switch-off  双机开关没开也查、也派第二台             switch-off-local-only
  no-usage-check        没有 -- 也照派                              usage
  long-history-ignored  ⓪ 不看用时表里的历史用时（只看 --estimated-hours）  long-history-single-host、partial-declaration-single-host
  long-estimate-ignored ⓪ 不看 --estimated-hours                    long-estimate-single-host
  shards-as-single      ⓪ 给了 --shard 也当只占一台                 two-host-shards-pass
  long-run-header-ignored  ⓪ 不读被起脚本文件头的长活声明          shards-hosts-declared-passes、unshardable-declared-passes、partial-declaration-single-host
  long-run-header-unjudged ⓪ 读了声明不交 judge 判，写了几个键都算齐  partial-declaration-single-host
  no-duration-record    跑完不记用时                                duration-recorded
"""
import os as preflight_os, sys as preflight_sys  # noqa: E402
# 开跑之前先判准入与运行条件（.claude/singlefs-ai-sop/rules/preflight-discipline.md）；不写 __pycache__
preflight_sys.dont_write_bytecode = True
preflight_sys.path.insert(0, preflight_os.path.join(preflight_os.path.dirname(preflight_os.path.realpath(__file__)), '..', '..', '.claude', 'scripts'))
from project_preflight import preflight  # noqa: E402

import base64  # noqa: E402
import concurrent.futures  # noqa: E402
import contextlib  # noqa: E402
import fcntl  # noqa: E402
import inspect  # noqa: E402
import json  # noqa: E402
import math  # noqa: E402
import os  # noqa: E402
import shutil  # noqa: E402
import signal  # noqa: E402
import subprocess  # noqa: E402
import sys  # noqa: E402
import tempfile  # noqa: E402
import threading  # noqa: E402
import time  # noqa: E402
from dataclasses import dataclass, field  # noqa: E402
from datetime import datetime, timezone  # noqa: E402

SCRIPT_DIRECTORY = os.path.dirname(os.path.realpath(__file__))
sys.path.insert(0, SCRIPT_DIRECTORY)
import memory_peaks_lib as peaks  # noqa: E402
sys.path.insert(0, os.path.join(os.path.dirname(SCRIPT_DIRECTORY), ".claude", "hooks"))
import lib_long_run as long_run_lib  # noqa: E402   长活声明的判法（文件头 ⓪），与 detached-run.sh 共用

MEBIBYTE = peaks.MEBIBYTE
GIBIBYTE = 1024 * MEBIBYTE
CAP_FACTOR = 1.5
MEMORY_MARGIN_BYTES = 2 * GIBIBYTE
GPU_MARGIN_MEBIBYTES = 2560
DEFAULT_GPU_PREDICTION_MEBIBYTES = 8192
POLL_SECONDS = 10.0
WAITING_REPORT_EVERY_SECONDS = 60.0
LAUNCH_GRACE_SECONDS = 30.0
GPU_SAMPLE_SECONDS = 5.0
GPU_SAMPLE_SECONDS_PEER = 15.0
PROBE_TIMEOUT_SECONDS = 60
EXIT_USAGE = 2
EXIT_NEVER_FITS = 241
EXIT_WAITED_OUT = 242
EXIT_PROBE_FAILED = 243
EXIT_LONG_SINGLE_HOST = 244        # 文件头 ⓪
GPU_MEMORY_HIT_EXIT = 245          # 内存包装认出的显存不够（run-with-memory-cap.sh 文件头「退出码」）
NOT_RUN_EXITS = (EXIT_USAGE, 251, 252, 255)   # 派活那条链退这几个是命令没跑成（multi-host-run.sh 用法错与它自己没跑成、包装起不了 scope 与排不上）：不记用时
PROBE_MARK = "HOST_DISPATCH_PROBE "
SELFTEST_MEASURED_AT = "2026-10-06T00:00:00Z"   # clock-times:allow 自证造的峰值表时刻，格式照真表
GPU_PEAK_TABLE_HEADER = [
    "# 每条命令上一次经 scripts/host-dispatch.py --gpu 跑时量到的显存峰值：派发器建、读、写（预测这一条要多少显存），别手改。",
    "# 口径：跑的时候按间隔读那台卡上的计算进程，派之前就在的不算，之后冒出来的那几个 used_memory 相加取最大（MiB）；偏大不偏小。",
    "# 列（制表符分隔）：峰值 MiB、量的时刻（ISO 8601，Z 结尾）、撞过顶（1：内存包装认出显存不够，真峰值不小于记的这个数）、量在哪台（local / peer）、键（同内存峰值表）。",
]


def broken(item):
    return item in os.environ.get("HOST_DISPATCH_BREAK", "").split(",")


def human(byte_count):
    return peaks.human(byte_count)


def mebibytes_text(byte_count):
    return f"{math.ceil(byte_count / MEBIBYTE)}M"


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
    multi_host_run: str
    gpu_quiesce_run: str
    configuration_check: str
    peer_host_lib: str
    peak_table: str
    gpu_peak_table: str
    state_directory: str
    quiesce_state_directory: str
    environment: dict
    poll_seconds: float = POLL_SECONDS
    report_every_seconds: float = WAITING_REPORT_EVERY_SECONDS
    wait_seconds: object = None
    gpu_sample_seconds: float = GPU_SAMPLE_SECONDS
    gpu_sample_seconds_peer: float = GPU_SAMPLE_SECONDS_PEER
    launch_grace_seconds: float = LAUNCH_GRACE_SECONDS
    say: object = None
    duration_table: str = ""     # 用时表（memory_peaks_lib.py 文件头「用时表」）；空串是不读不记

    def tell(self, text):
        (self.say or (lambda line: print(line, file=sys.stderr, flush=True)))(text)


def settings_from_environment():
    environment = dict(os.environ)
    repository_root = os.path.dirname(SCRIPT_DIRECTORY)
    state_home = environment.get("XDG_STATE_HOME") or os.path.join(environment.get("HOME") or "/", ".local", "state")
    runtime = environment.get("XDG_RUNTIME_DIR") or "/tmp"
    wait_text = environment.get("HOST_DISPATCH_WAIT_SECONDS", "")
    return Settings(
        repository_root=repository_root,
        multi_host_run=os.path.join(SCRIPT_DIRECTORY, "multi-host-run.sh"),
        gpu_quiesce_run=os.path.join(SCRIPT_DIRECTORY, "..", "crash", "gpu-quiesce-run.sh"),
        configuration_check=os.path.join(SCRIPT_DIRECTORY, "..", "crash", "layer0-shard-configuration-check.sh"),
        peer_host_lib=os.path.join(SCRIPT_DIRECTORY, "peer-host-lib.sh"),
        peak_table=environment.get("RUN_WITH_MEMORY_CAP_PEAKS") or peaks.default_peak_table(SCRIPT_DIRECTORY),
        gpu_peak_table=environment.get("HOST_DISPATCH_GPU_PEAKS") or os.path.join(state_home, "singlefs", "gpu-memory-peaks.tsv"),
        state_directory=environment.get("HOST_DISPATCH_STATE_DIR") or os.path.join(runtime, "singlefs-host-dispatch"),
        quiesce_state_directory=environment.get("GPU_QUIESCE_STATE_DIRECTORY") or os.path.join(runtime, "singlefs-gpu-quiesce"),
        environment=environment,
        wait_seconds=float(wait_text) if wait_text else None,
        duration_table=peaks.default_duration_table(environment))


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
                  'if ((status == 0)); then declare -p PEER_SSH_HOST PEER_REPOSITORY_DIRECTORY PEER_CARGO_BIN_DIRECTORY PEER_MEMORY_CAP peer_host_is_this_machine; fi; exit $status')
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
        cap_line = next((line for line in completed.stdout.splitlines() if "PEER_MEMORY_CAP=" in line), "")
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
    program = base64.b64encode(peer_probe_program().encode("utf-8")).decode("ascii")
    flags = "--apps" if apps_only else ("--gpu" if gpu else "")
    status, stdout, stderr = peer.run(f'python3 -B -c "$(printf %s {program} | base64 -d)" {flags}')
    report = parse_probe(stdout)
    if status != 0 or report is None:
        return None, f"第二台的余量查不成（退 {status}：{stderr.strip()[-300:]}）"
    return report, ""


def live_quiesce_holders(settings):
    """本机清场包装 holders/ 下活着的登记数（<进程号>-<起始时刻>，进程在、起始时刻对得上才算活着；gpu-quiesce-run.sh 文件头「几趟同时用卡」）。"""
    directory = os.path.join(settings.quiesce_state_directory, "holders")
    count = 0
    with contextlib.suppress(FileNotFoundError):
        for name in os.listdir(directory):
            process_id, _, start_time = name.partition("-")
            if process_id.isdigit() and start_time and peaks.process_start_time(process_id) == start_time:
                count += 1
    return count


# ── 派发器自己的账（刚派出去、包装还没记账的那一段；--gpu 的显存预测与已用）──
def ledger_directory(settings):
    return os.path.join(settings.state_directory, "ledger")


@contextlib.contextmanager
def dispatch_lock(settings):
    os.makedirs(settings.state_directory, exist_ok=True)
    with open(os.path.join(settings.state_directory, "lock"), "a", encoding="utf-8") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def write_json_atomically(path, value):
    temporary = f"{path}.{os.getpid()}.partial"
    with open(temporary, "w", encoding="utf-8") as handle:
        json.dump(value, handle, ensure_ascii=False)
    os.replace(temporary, path)


def live_dispatch_entries(settings):
    entries = []
    with contextlib.suppress(FileNotFoundError):
        for name in sorted(os.listdir(ledger_directory(settings))):
            if not name.endswith(".json"):
                continue
            path = os.path.join(ledger_directory(settings), name)
            try:
                with open(path, encoding="utf-8") as handle:
                    entry = json.load(handle)
            except (OSError, ValueError):
                continue
            if peaks.process_start_time(entry.get("pid")) != entry.get("pid_start_time"):
                with contextlib.suppress(FileNotFoundError):
                    os.remove(path)
                continue
            entries.append(entry)
    return entries


def dispatch_pending(settings, host, entries):
    """(内存字节, 显存 MiB)：这台上派发器账上还没被包装记账的内存（派出去不到 launch_grace_seconds 秒的那几条按记的量）与 --gpu 那几趟预测了还没用到的显存。"""
    if broken("no-dispatch-ledger"):
        return 0, 0
    now = time.time()
    memory = sum(entry["memory_booked"] for entry in entries if entry["host"] == host and now - entry["launched_at"] < settings.launch_grace_seconds)
    gpu = sum(max(0, entry["gpu_need"] - entry.get("gpu_observed", 0)) for entry in entries if entry["host"] == host and entry["gpu_need"])
    return memory, gpu


# ── 预测 ──
@dataclass
class Prediction:
    key: str
    memory_predicted: int        # 字节
    memory_cap: int              # 字节：预测 × CAP_FACTOR，按 MiB 向上取整
    memory_booked: int           # 包装要记的量：有真峰值记峰值，没有记上限
    memory_history: str          # peak、hit、default
    gpu_predicted: int = 0       # MiB
    gpu_history: str = ""


def read_gpu_peak_table(path):
    rows = {}
    with contextlib.suppress(FileNotFoundError):
        with open(path, encoding="utf-8") as handle:
            for line in handle.read().splitlines():
                fields = line.split("\t")
                if line.startswith("#") or len(fields) != 5 or not fields[0].isdigit():
                    continue
                rows[fields[4]] = (int(fields[0]), fields[1], fields[2] == "1", fields[3])
    return rows


def predict(settings, command, gpu):
    key = peaks.command_key(command, settings.environment.get("RUN_WITH_MEMORY_CAP_KEY"))
    default_bytes = 0 if broken("no-default") else peaks.size_bytes(peaks.DEFAULT_MEMORY_PREDICTION_TEXT)
    peak, hit = (None, False) if broken("ignore-history") else peaks.recorded_peak(settings.peak_table, key)
    if peak is None:
        predicted, history = default_bytes, "default"
    elif hit and not broken("hit-as-peak"):
        predicted, history = math.ceil(peak * CAP_FACTOR), "hit"
    else:
        predicted, history = peak, "peak"
    cap = math.ceil(predicted * CAP_FACTOR / MEBIBYTE) * MEBIBYTE
    booked = predicted if history == "peak" else cap
    prediction = Prediction(key, predicted, cap, booked, history)
    if gpu:
        row = None if broken("ignore-history") else read_gpu_peak_table(settings.gpu_peak_table).get(key)
        if row is None:
            prediction.gpu_predicted, prediction.gpu_history = (0 if broken("no-default") else DEFAULT_GPU_PREDICTION_MEBIBYTES), "default"
        elif row[2] and not broken("hit-as-peak"):
            prediction.gpu_predicted, prediction.gpu_history = math.ceil(row[0] * CAP_FACTOR), "hit"
        else:
            prediction.gpu_predicted, prediction.gpu_history = row[0], "peak"
    return prediction


# ── 判放不放得下 ──
@dataclass
class Verdict:
    host: str
    fits: bool
    never: bool
    cap: int
    why: str
    memory_short: int = 0        # 字节：内存还差多少（放得下是 0）；scripts/host-resources.py 报差额读它
    gpu_short: object = 0        # MiB：显存还差多少（放得下或没带 --gpu 是 0，读不到卡是 None）


def judge_host(host, report, prediction, gpu, pending_memory, pending_gpu, peer_cap_bytes):
    memory = report["memory"]
    cap = prediction.memory_cap
    booked = prediction.memory_booked
    if host == "peer" and peer_cap_bytes is not None and not broken("no-peer-cap-clamp"):
        if booked > peer_cap_bytes:
            return Verdict(host, False, True, cap, f"包装要记的 {human(booked)} 比配置的 PEER_MEMORY_CAP {human(peer_cap_bytes)} 大",
                           memory_short=booked - peer_cap_bytes)
        cap = min(cap, peer_cap_bytes)
    if broken("ignore-pending"):
        free, room = memory["mem_available"], memory["slice_total"] - memory["slice_used"]
    else:
        free, room = memory["free"] - pending_memory, memory["slice_room"] - pending_memory
    never = booked > memory["slice_total"] or prediction.memory_predicted + MEMORY_MARGIN_BYTES > memory["mem_total"]
    reasons = []
    fits = prediction.memory_predicted + MEMORY_MARGIN_BYTES <= free and booked <= room
    memory_short = 0 if fits else max(0, prediction.memory_predicted + MEMORY_MARGIN_BYTES - free, booked - room)
    gpu_short = 0
    if not fits:
        reasons.append(f"内存：预测 {human(prediction.memory_predicted)} + 余量 {human(MEMORY_MARGIN_BYTES)}、包装要记 {human(booked)}，"
                       f"此刻空闲 {human(max(free, 0))}、slice 放得下 {human(max(room, 0))}")
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
    return Verdict(host, fits, never and not fits, cap, "；".join(reasons) if reasons else f"内存空闲 {human(free)}、slice 放得下 {human(room)}",
                   memory_short=memory_short, gpu_short=gpu_short)


@dataclass
class Request:
    command: list
    gpu: bool = False
    label: str = ""
    fetch: list = field(default_factory=list)
    shard: object = None             # (k, n)：--shard <k>/<n>
    estimated_hours: object = None   # --estimated-hours


# ── ⓪ 长活不单机派 ──
@dataclass
class LongRun:
    seconds: object              # 判用的用时（秒）；None 是没有历史也没给预计挂钟
    source: str                  # history、estimate、none
    measured_on: object          # history 时是量的那一天
    hosts: str                   # multi、cannot、single（hosts_of_dispatch）
    why: str                     # 占几台是怎么判出来的
    refuse: bool                 # 用时超过 LONG_RUN_SECONDS 而只占一台


@contextlib.contextmanager
def without_library_breaks():
    """lib_long_run 的弄坏开关只给 detached-run.sh 与 runner-dispatch-guard.sh 的自证用：判声明时从环境里摘掉，判完放回。"""
    saved = os.environ.pop(long_run_lib.BREAK_VARIABLE, None)
    try:
        yield
    finally:
        if saved is not None:
            os.environ[long_run_lib.BREAK_VARIABLE] = saved


def header_hosts(command, repository_root):
    """(multi|cannot|single, 怎么判的)：被起脚本文件头的长活声明（文件头 ⓪，判法 lib_long_run）。"""
    with without_library_breaks():
        script = long_run_lib.started_script(command, repository_root)
        if script is None:
            return "single", "命令里认不出被起的脚本（读不到文件头的长活声明）"
        try:
            declared = long_run_lib.script_header_declaration(script)
        except OSError as error:
            return "single", f"读不了被起的脚本 {script}：{error}"
        if not declared:
            return "single", f"{script} 文件头没写长活声明（# long-run-shards: 与 # long-run-hosts:，或 # long-run-unshardable:）"
        problems = [] if broken("long-run-header-unjudged") else long_run_lib.judge(declared, long_run_lib.HEADER_SPELLING)
        if problems:
            return "single", f"{script} 文件头的长活声明不齐：{'；'.join(problems)}"
        if all(long_run_lib.is_written(declared.get(key)) for key in ("shards", "hosts")):
            return "multi", f"脚本头写了它自己分片、派到几台：分片：{declared['shards']}；多机：{declared['hosts']}"
        return "cannot", f"脚本头写了分不了：{declared.get('unshardable', '')}"


def hosts_of_dispatch(command, shard, usable_hosts, repository_root):
    """(multi|cannot|single, 怎么判的)：文件头 ⓪「派法占几台」。usable_hosts 是此刻能派的机器台数。"""
    if shard is not None and shard[1] >= 2 and not broken("shards-as-single"):
        if usable_hosts >= 2:
            return "multi", f"--shard {shard[0]}/{shard[1]}，此刻能派的机器 {usable_hosts} 台"
        shard_note = f"给了 --shard {shard[0]}/{shard[1]}，但此刻能派的机器只有 {usable_hosts} 台（多机配置的双机开关没开或判不过）"
    else:
        shard_note = "没给 --shard <k>/<n>（n ≥ 2）"
    if broken("long-run-header-ignored"):
        return "single", shard_note
    hosts, why = header_hosts(command, repository_root)
    return (hosts, why) if hosts != "single" else ("single", f"{shard_note}；{why}")


def long_run_check(settings, command, key, shard, estimated_hours, usable_hosts, use_history=True):
    """文件头 ⓪：交 LongRun。scripts/host-resources.py 带命令时调同一份（use_history 只给它的弄坏开关关掉历史那一半）。"""
    seconds, measured_on = (None, None)
    if use_history and not broken("long-history-ignored"):
        seconds, measured_on = peaks.recorded_duration(settings.duration_table, key)
    source = "history" if seconds is not None else "none"
    if seconds is None and estimated_hours is not None and not broken("long-estimate-ignored"):
        seconds, source = estimated_hours * 3600, "estimate"
    hosts, why = hosts_of_dispatch(command, shard, usable_hosts, settings.repository_root)
    return LongRun(seconds, source, measured_on, hosts, why, seconds is not None and seconds > peaks.LONG_RUN_SECONDS and hosts == "single")


def long_run_text(long_run):
    """给人看的用时那一段：「历史用时 3.0 小时（2026-10-05 量的）」或「预计挂钟 3.0 小时（--estimated-hours）」。"""
    if long_run.source == "history":
        return f"历史用时 {long_run.seconds / 3600:.1f} 小时（{long_run.measured_on} 量的）"
    if long_run.source == "estimate":
        return f"预计挂钟 {long_run.seconds / 3600:.1f} 小时（--estimated-hours）"
    return "没有历史用时、也没给预计挂钟"


LONG_RUN_HOWTO = ("改成按 `layer0-shard-configuration-check.sh --emit-hosts` 与 `host-resources.py --json` 现算的两台分片"
                  "（每片各派一次：host-dispatch.py --shard <k>/<n> -- <那一片的命令>），或在被起脚本的文件头补齐长活声明"
                  "（# long-run-shards: 与 # long-run-hosts:，分不了的写 # long-run-unshardable: <缺哪一段、为什么分不了>；写法与判法在 .claude/hooks/lib_long_run.py 文件头）")


@dataclass
class Decision:
    host: object                 # "local"、"peer"；None 是没派
    cap: int = 0
    exit_code: int = 0           # 没派时的退出码（1 是 --plan 此刻放不下，241、242、243）
    entry_path: str = ""
    probes: int = 0


def host_name(host):
    return "第二台" if host == "peer" else "本机"


def decide(settings, request, prediction, peer, plan=False):
    """②–④：查、判、挑；放不下就等（plan 不等）。挑中了在派发器账上记一笔（plan 不记）。"""
    started = time.monotonic()
    last_report = None
    probes = 0
    first_reports = None
    while True:
        with dispatch_lock(settings):
            if first_reports is None or not broken("no-reprobe"):
                reports = {}
                local_report, problem = probe_local(settings, request.gpu)
                probes += 1
                if local_report is None:
                    settings.tell(f"  ✗ {problem}")
                    settings.tell("  → 怎么办：看 /proc/meminfo 读不读得了；本机都查不成就别派，修好再跑")
                    return Decision(None, exit_code=EXIT_PROBE_FAILED, probes=probes)
                reports["local"] = local_report
                if peer is not None and peer.available and not broken("local-only"):
                    peer_report, problem = probe_peer(peer, request.gpu)
                    if peer_report is None:
                        settings.tell(f"HOST_DISPATCH step=probe host=peer result=unreadable why={problem.replace(' ', '_')[:300]}")
                    else:
                        reports["peer"] = peer_report
                if request.gpu:
                    holders = live_quiesce_holders(settings)   # 清场在本机跑、两台的卡一起管
                    for report in reports.values():
                        report["quiesce_holders"] = holders
                first_reports = reports
            else:
                reports = first_reports
            entries = live_dispatch_entries(settings)
            verdicts = []
            for host in (["peer", "local"] if broken("peer-first") else ["local", "peer"]):
                if host not in reports:
                    continue
                pending_memory, pending_gpu = dispatch_pending(settings, host, entries)
                verdicts.append(judge_host(host, reports[host], prediction, request.gpu, pending_memory, pending_gpu,
                                           peer.memory_cap_bytes if (peer is not None and host == "peer") else None))
            chosen = next((verdict for verdict in verdicts if verdict.fits), None)
            if chosen is None and broken("no-wait"):
                chosen = next((verdict for verdict in verdicts if verdict.host == "local"), None)
            if chosen is not None:
                settings.tell(f"HOST_DISPATCH step={'plan' if plan else 'decide'} host={chosen.host} memory_predicted={mebibytes_text(prediction.memory_predicted)} "
                              f"memory_history={prediction.memory_history} cap={mebibytes_text(chosen.cap)} booked={mebibytes_text(prediction.memory_booked)}"
                              + (f" gpu_predicted={prediction.gpu_predicted}M gpu_history={prediction.gpu_history}" if request.gpu else "")
                              + f" waited={time.monotonic() - started:.0f}s")
                entry_path = ""
                if not plan:
                    os.makedirs(ledger_directory(settings), exist_ok=True)
                    entry_path = os.path.join(ledger_directory(settings), f"{os.getpid()}-{time.monotonic_ns()}.json")
                    write_json_atomically(entry_path, {
                        "pid": os.getpid(), "pid_start_time": peaks.process_start_time(os.getpid()), "host": chosen.host,
                        "memory_booked": chosen.cap if broken("book-by-cap") else prediction.memory_predicted,
                        "gpu_need": prediction.gpu_predicted if request.gpu else 0, "gpu_observed": 0,
                        "launched_at": time.time(), "key": prediction.key[:200]})
                return Decision(chosen.host, cap=chosen.cap, entry_path=entry_path, probes=probes)
            if verdicts and all(verdict.never for verdict in verdicts) and not broken("wait-on-hopeless"):
                settings.tell("  ✗ 哪台都等多久也放不下：" + "；".join(f"{host_name(verdict.host)}：{verdict.why}" for verdict in verdicts))
                settings.tell("  → 怎么办：这一条要的比每台都大——先查它为什么要这么多（峰值表 scripts/memory-peaks.tsv 里键「"
                              f"{prediction.key[:80]}」那一行、显存峰值表 {settings.gpu_peak_table}），或拆小再派；不绕开派发器硬起")
                return Decision(None, exit_code=EXIT_NEVER_FITS, probes=probes)
        if plan:
            for verdict in verdicts:
                settings.tell(f"HOST_DISPATCH step=plan host={verdict.host} fits=0 why={verdict.why}")
            return Decision(None, exit_code=1, probes=probes)
        now = time.monotonic()
        if settings.wait_seconds is not None and now - started >= settings.wait_seconds:
            settings.tell(f"  ✗ 等了 {now - started:.0f} 秒（HOST_DISPATCH_WAIT_SECONDS={settings.wait_seconds:g}）还是哪台都放不下："
                          + "；".join(f"{host_name(verdict.host)}：{verdict.why}" for verdict in verdicts))
            settings.tell("  → 怎么办：等占着的那几条跑完再派（run-with-memory-cap.sh --status、nvidia-smi 看是谁），或不设 HOST_DISPATCH_WAIT_SECONDS 让它一直等")
            return Decision(None, exit_code=EXIT_WAITED_OUT, probes=probes)
        if last_report is None or now - last_report >= settings.report_every_seconds:
            settings.tell(f"HOST_DISPATCH step=wait waited={now - started:.0f}s "
                          + " ".join(f"{verdict.host}=「{verdict.why}」" for verdict in verdicts)
                          + ("" if peer is None or peer.available else f" peer=「不用：{peer.why}」"))
            last_report = now
        time.sleep(settings.poll_seconds)


# ── 派与量显存 ──
def launch_arguments(settings, request, decision):
    arguments = ["bash", settings.multi_host_run] + (["--peer"] if decision.host == "peer" else []) \
        + ["--memory-cap", mebibytes_text(decision.cap), "--label", request.label]
    for path in request.fetch:
        arguments += ["--fetch", path]
    arguments += ["--", *request.command]
    if request.gpu and not broken("no-quiesce"):
        arguments = ["bash", settings.gpu_quiesce_run, "--", *arguments]
    return arguments


def sample_apps(settings, host, peer):
    if host == "peer":
        report, _ = probe_peer(peer, True, apps_only=True)
        return None if report is None else report.get("apps")
    return gpu_apps(settings.environment).get("apps")


class GpuSampler(threading.Thread):
    """跑的时候按间隔读那台卡上的计算进程；派之前就在的不算，之后冒出来的那几个相加，取最大。"""

    def __init__(self, settings, host, peer, entry_path):
        super().__init__(daemon=True)
        self.settings, self.host, self.peer, self.entry_path = settings, host, peer, entry_path
        self.baseline = set((sample_apps(settings, host, peer) or {}).keys())
        self.observed = 0
        self.stopped = threading.Event()

    def run(self):
        interval = self.settings.gpu_sample_seconds_peer if self.host == "peer" else self.settings.gpu_sample_seconds
        while not self.stopped.is_set():
            self.sample()
            self.stopped.wait(interval)

    def sample(self):
        apps = sample_apps(self.settings, self.host, self.peer)
        if apps is None:
            return
        used = sum(mebibytes for process_id, mebibytes in apps.items() if process_id not in self.baseline)
        if used > self.observed:
            self.observed = used
            with contextlib.suppress(OSError, ValueError):
                with open(self.entry_path, encoding="utf-8") as handle:
                    entry = json.load(handle)
                entry["gpu_observed"] = used
                write_json_atomically(self.entry_path, entry)


def record_gpu_peak(settings, key, host, mebibytes, hit):
    if broken("no-gpu-record"):
        return
    os.makedirs(os.path.dirname(settings.gpu_peak_table) or ".", exist_ok=True)
    with open(settings.gpu_peak_table + ".lock", "a", encoding="utf-8") as lock:
        fcntl.flock(lock, fcntl.LOCK_EX)
        rows = read_gpu_peak_table(settings.gpu_peak_table)
        rows[key] = (mebibytes, datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"), hit, host)
        lines = GPU_PEAK_TABLE_HEADER + ["\t".join([str(value[0]), value[1], "1" if value[2] else "0", value[3], row_key])
                                         for row_key, value in sorted(rows.items())]
        temporary = f"{settings.gpu_peak_table}.{os.getpid()}.partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.write("\n".join(lines) + "\n")
        os.replace(temporary, settings.gpu_peak_table)


def run_request(settings, request, plan=False, install_signal_handlers=False):
    prediction = predict(settings, request.command, request.gpu)
    peer = Peer(settings)
    if not peer.available:
        settings.tell(f"HOST_DISPATCH step=probe host=peer result=unused why={peer.why.replace(' ', '_')}")
    long_run = long_run_check(settings, request.command, prediction.key, request.shard, request.estimated_hours, 2 if peer.available else 1)
    settings.tell(f"HOST_DISPATCH step=long-run source={long_run.source} hours={'-' if long_run.seconds is None else f'{long_run.seconds / 3600:.1f}'} "
                  f"hosts={long_run.hosts} refuse={int(long_run.refuse)}")
    if long_run.hosts == "cannot" and long_run.seconds is not None and long_run.seconds > peaks.LONG_RUN_SECONDS:
        settings.tell(f"  · 长活：{long_run_text(long_run)}超过 2 小时，{long_run.why}：照常派一台")
    if long_run.refuse:
        settings.tell(f"  ✗ 长活只派一台：这条命令的{long_run_text(long_run)}超过 2 小时，而这一次派法只占一台（{long_run.why}）：不派")
        settings.tell(f"  → 怎么办：{LONG_RUN_HOWTO}")
        return EXIT_LONG_SINGLE_HOST
    decision = decide(settings, request, prediction, peer, plan=plan)
    if decision.host is None or plan:
        return decision.exit_code
    request.label = request.label or f"host-dispatch-{datetime.now(timezone.utc):%Y%m%dT%H%M%SZ}-{os.getpid()}"
    arguments = launch_arguments(settings, request, decision)
    sampler = None
    exit_code = 255
    interrupted = []
    started = time.monotonic()
    try:
        if request.gpu:
            sampler = GpuSampler(settings, decision.host, peer, decision.entry_path)
        child = subprocess.Popen(arguments, env=settings.environment, cwd=settings.repository_root)
        if install_signal_handlers:
            def forward(signal_number, _frame):
                interrupted.append(signal_number)
                with contextlib.suppress(ProcessLookupError):
                    child.send_signal(signal_number)   # 只转给自己起的那一个子进程（文件头「收到 TERM / INT / HUP」）
            for signal_number in (signal.SIGTERM, signal.SIGINT, signal.SIGHUP):
                signal.signal(signal_number, forward)
        if sampler is not None:
            sampler.start()
        exit_code = child.wait()
        elapsed = time.monotonic() - started
        if exit_code not in NOT_RUN_EXITS and not interrupted and not broken("no-duration-record"):
            with contextlib.suppress(OSError):
                peaks.record_duration(settings.duration_table, prediction.key, math.ceil(elapsed), exit_code, decision.host,
                                      datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"))
    finally:
        if sampler is not None:
            sampler.stopped.set()
            if sampler.is_alive():
                sampler.join()
        with contextlib.suppress(FileNotFoundError):
            os.remove(decision.entry_path)
    if sampler is not None:
        if exit_code == GPU_MEMORY_HIT_EXIT:
            record_gpu_peak(settings, prediction.key, decision.host, max(sampler.observed, prediction.gpu_predicted), True)
        elif sampler.observed > 0:
            record_gpu_peak(settings, prediction.key, decision.host, sampler.observed, False)
    settings.tell(f"HOST_DISPATCH step=exit host={decision.host} code={exit_code}"
                  + (f" gpu_observed={sampler.observed}M" if sampler is not None else ""))
    return 0 if broken("exit-dropped") else exit_code


def parse_shard(text):
    """--shard 的值 <k>/<n>（1 ≤ k ≤ n，都是正整数）→ (k, n)；写错抛 ValueError。host-resources.py 用同一份。"""
    number, _, count = text.partition("/")
    if not (number.isdigit() and count.isdigit() and 1 <= int(number) <= int(count)):
        raise ValueError(f"--shard 读到「{text}」：要写 <k>/<n>，1 ≤ k ≤ n")
    return int(number), int(count)


def parse_hours(text):
    """--estimated-hours 的值（正数，可带小数）→ float；写错抛 ValueError。host-resources.py 用同一份。"""
    try:
        hours = float(text)
    except ValueError:
        hours = -1.0
    if not hours > 0 or hours != hours or hours == float("inf"):
        raise ValueError(f"--estimated-hours 读到「{text}」：要写正数小时（例 3 或 0.5）")
    return hours


def parse_arguments(arguments):
    """交 (Request, plan) 或抛 ValueError（用法错）。"""
    request = Request(command=[])
    plan = False
    index = 0
    while index < len(arguments):
        word = arguments[index]
        if word == "--":
            request.command = arguments[index + 1:]
            break
        if word == "--gpu":
            request.gpu = True
        elif word == "--plan":
            plan = True
        elif word in ("--label", "--fetch", "--shard", "--estimated-hours"):
            if index + 1 >= len(arguments):
                raise ValueError(f"{word} 后面没给值")
            value = arguments[index + 1]
            if word == "--label":
                request.label = value
            elif word == "--fetch":
                request.fetch.append(value)
            elif word == "--shard":
                request.shard = parse_shard(value)
            else:
                request.estimated_hours = parse_hours(value)
            index += 1
        elif broken("no-usage-check"):
            request.command = arguments[index:]
            break
        else:
            raise ValueError(f"认不出的参数「{word}」（命令要写在 -- 后面）")
        index += 1
    if not request.command:
        raise ValueError("-- 后面没有命令")
    if request.label and not all(character.isalnum() or character in "._-" for character in request.label):
        raise ValueError(f"--label 读到「{request.label}」：只用字母、数字、点、减号与下划线")
    return request, plan


def main(arguments):
    if os.environ.get("HOST_DISPATCH_BREAK"):
        print("  ✗ 弄坏开关 HOST_DISPATCH_BREAK 设在 --selftest 之外，命令没派", file=sys.stderr)
        print("  → 怎么办：去掉 HOST_DISPATCH_BREAK 再跑；它只给 host-dispatch.py --selftest 证红用", file=sys.stderr)
        return EXIT_USAGE
    try:
        request, plan = parse_arguments(arguments)
    except ValueError as error:
        print(f"  ✗ host-dispatch.py 用法错：{error}", file=sys.stderr)
        print("  → 怎么办：host-dispatch.py [--gpu] [--label <标签>] [--fetch <树里的相对路径>]… [--shard <k>/<n>] [--estimated-hours <小时>] -- <命令…>，"
              "或 --plan [--gpu] [--shard <k>/<n>] [--estimated-hours <小时>] -- <命令…>、--selftest", file=sys.stderr)
        return EXIT_USAGE
    return run_request(settings_from_environment(), request, plan=plan, install_signal_handlers=not plan)


# ── 自证 ──
FAKE_NVIDIA_SMI = """#!/usr/bin/env bash
here="$(dirname "$0")"
case "$*" in
  *--query-gpu*) cat "$here/gpus.csv" ;;
  *--query-compute-apps*) cat "$here/apps.csv" 2>/dev/null ;;
  *) exit 9 ;;
esac
"""
FAKE_SSH = """#!/usr/bin/env bash
# 假 ssh：去掉 -o 选项与主机名，把命令照 ssh 那样交给 bash -c；第二台的 /proc/meminfo、包装的状态目录与卡换成 peer/ 下的假的
while [[ "${1:-}" == -o ]]; do shift 2; done
printf '%s\\n' "$1" >> "$HOST_DISPATCH_SELFTEST_SSH_LOG"
shift
exec env SINGLEFS_DISPATCH_PROBE_MEMINFO="$HOST_DISPATCH_SELFTEST_PEER/meminfo" RUN_WITH_MEMORY_CAP_STATE_DIR="$HOST_DISPATCH_SELFTEST_PEER/state" \\
  bash -c "$*"
"""
FAKE_LAUNCHER = """#!/usr/bin/env bash
# 假 multi-host-run.sh：参数记一行；睡 FAKE_LAUNCH_SECONDS 秒（FAKE_LAUNCH_APPS 给了就先往那台的假 apps.csv 写它、睡完清掉）；退 FAKE_LAUNCH_EXIT
printf 'launch %s\\n' "$*" >> "$HOST_DISPATCH_SELFTEST_LAUNCH_LOG"
target_apps="$HOST_DISPATCH_SELFTEST_LOCAL/bin/apps.csv"
[[ " $* " == *" --peer "* ]] && target_apps="$HOST_DISPATCH_SELFTEST_PEER/bin/apps.csv"
if [[ -n "${FAKE_LAUNCH_APPS:-}" ]]; then
  printf '%s\\n' "$FAKE_LAUNCH_APPS" >> "$target_apps"
  sleep "${FAKE_LAUNCH_SECONDS:-0}"
  : > "$target_apps"
else
  sleep "${FAKE_LAUNCH_SECONDS:-0}"
fi
exit "${FAKE_LAUNCH_EXIT:-0}"
"""
FAKE_QUIESCE = """#!/usr/bin/env bash
# 假 gpu-quiesce-run.sh：记一行 quiesce，去掉 -- 照原样起后面那条
printf 'quiesce\\n' >> "$HOST_DISPATCH_SELFTEST_LAUNCH_LOG"
[[ "${1:-}" == -- ]] && shift
"$@"
"""


class SelftestWorld:
    """一格一个：临时目录里的本机、第二台、配置、峰值表与假的派活脚本。"""

    def __init__(self, root, name):
        self.directory = os.path.join(root, name)
        self.local = os.path.join(self.directory, "local")
        self.peer = os.path.join(self.directory, "peer")
        for host in (self.local, self.peer):
            os.makedirs(os.path.join(host, "bin"))
            os.makedirs(os.path.join(host, "state", "ledger"))
            self.write_executable(os.path.join(host, "bin", "nvidia-smi"), FAKE_NVIDIA_SMI)
            self.cards(host, [(0, 16384, 16384)])
        self.write_executable(os.path.join(self.peer, "bin", "cargo"), "#!/usr/bin/env bash\nexit 0\n")
        os.makedirs(os.path.join(self.directory, "ssh-bin"))
        self.write_executable(os.path.join(self.directory, "ssh-bin", "ssh"), FAKE_SSH)
        self.write_executable(os.path.join(self.directory, "multi-host-run.sh"), FAKE_LAUNCHER)
        self.write_executable(os.path.join(self.directory, "gpu-quiesce-run.sh"), FAKE_QUIESCE)
        self.meminfo(self.local, 60, 40)
        self.meminfo(self.peer, 60, 40)
        self.configuration(True)
        self.launch_log = os.path.join(self.directory, "launch.log")
        self.ssh_log = os.path.join(self.directory, "ssh.log")
        self.peak_table = os.path.join(self.directory, "memory-peaks.tsv")
        self.gpu_peak_table = os.path.join(self.directory, "gpu-peaks.tsv")
        self.duration_table = os.path.join(self.directory, "durations.tsv")
        self.quiesce_state = os.path.join(self.directory, "quiesce-state")
        self.lines = []

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
    def cards(host, cards):
        with open(os.path.join(host, "bin", "gpus.csv"), "w", encoding="utf-8") as handle:
            handle.write("".join(f"{index}, {total}, {free}\n" for index, total, free in cards))

    def configuration(self, across_machines, peer_cap="40G"):
        with open(os.path.join(self.directory, "multi-host.env"), "w", encoding="utf-8") as handle:
            handle.write(f"PEER_SSH_HOST=selftest-peer-host-name\nPEER_REPOSITORY_DIRECTORY={self.peer}/home\nPEER_CARGO_BIN_DIRECTORY={self.peer}/bin\n"
                         f"PEER_MEMORY_CAP={peer_cap}\nQUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\n"
                         f"QUIESCE_STARTED_CHECK_COMMAND=\nENABLE_ACROSS_MACHINES={1 if across_machines else 0}\n")

    def seed_peak(self, key, peak_bytes, cap_text):
        with open(self.peak_table, "a", encoding="utf-8") as handle:
            handle.write(f"{peak_bytes}\t{SELFTEST_MEASURED_AT}\t{cap_text}\t{key}\n")

    def seed_gpu_peak(self, key, mebibytes, hit=False):
        with open(self.gpu_peak_table, "a", encoding="utf-8") as handle:
            handle.write(f"{mebibytes}\t{SELFTEST_MEASURED_AT}\t{1 if hit else 0}\tlocal\t{key}\n")

    def seed_duration(self, key, hours, table=None):
        """用时表里记一行（host-resources.py 的自证也调它，给自己的表路径）。"""
        with open(table or self.duration_table, "a", encoding="utf-8") as handle:
            handle.write(f"{int(hours * 3600)}\t{SELFTEST_MEASURED_AT}\t0\tlocal\t{key}\n")

    def wrapper_ledger_entry(self, host, need_bytes):
        """包装账上一笔活着的（进程号写这个自证自己，起始时刻对得上），它的 scope 不在：此刻占 0，还没涨到量的一截就是整个 need。"""
        entry = {"unit": "singlefs-memory-cap-selftest", "pid": os.getpid(), "pid_start_time": peaks.process_start_time(os.getpid()),
                 "need": need_bytes, "cap": "16G", "key": "selftest-running"}
        with open(os.path.join(host, "state", "ledger", "singlefs-memory-cap-selftest.json"), "w", encoding="utf-8") as handle:
            json.dump(entry, handle)

    def live_quiesce_holder(self):
        os.makedirs(os.path.join(self.quiesce_state, "holders"), exist_ok=True)
        with open(os.path.join(self.quiesce_state, "holders", f"{os.getpid()}-{peaks.process_start_time(os.getpid())}"), "w", encoding="utf-8"):
            pass

    def settings(self):
        environment = {name: value for name, value in os.environ.items()
                       if not name.startswith(("RUN_WITH_MEMORY_CAP_", "SINGLEFS_", "GPU_QUIESCE_", "HOST_DISPATCH_", "MULTI_HOST_", "PEER_HOST_LIB_",
                                               "CLAUDE_CODE_SESSION_ID"))}
        environment.update({
            "PATH": os.pathsep.join([os.path.join(self.local, "bin"), os.path.join(self.directory, "ssh-bin"), os.environ.get("PATH", "")]),
            "SINGLEFS_DISPATCH_PROBE_MEMINFO": os.path.join(self.local, "meminfo"),
            "RUN_WITH_MEMORY_CAP_STATE_DIR": os.path.join(self.local, "state"),
            "RUN_WITH_MEMORY_CAP_SLICE": "singlefs_host_dispatch_selftest_absent.slice",
            "RUN_WITH_MEMORY_CAP_RESERVE": "20G",
            "SINGLEFS_MULTI_HOST_CONFIG": os.path.join(self.directory, "multi-host.env"),
            "HOST_DISPATCH_SELFTEST_SSH_LOG": self.ssh_log,
            "HOST_DISPATCH_SELFTEST_LAUNCH_LOG": self.launch_log,
            "HOST_DISPATCH_SELFTEST_LOCAL": self.local,
            "HOST_DISPATCH_SELFTEST_PEER": self.peer,
        })
        return Settings(
            repository_root=os.path.dirname(SCRIPT_DIRECTORY),
            multi_host_run=os.path.join(self.directory, "multi-host-run.sh"),
            gpu_quiesce_run=os.path.join(self.directory, "gpu-quiesce-run.sh"),
            configuration_check=os.path.join(SCRIPT_DIRECTORY, "..", "crash", "layer0-shard-configuration-check.sh"),
            peer_host_lib=os.path.join(SCRIPT_DIRECTORY, "peer-host-lib.sh"),
            peak_table=self.peak_table, gpu_peak_table=self.gpu_peak_table,
            state_directory=os.path.join(self.directory, "dispatch-state"), quiesce_state_directory=self.quiesce_state,
            environment=environment, poll_seconds=0.1, report_every_seconds=0.0, wait_seconds=3.0,
            gpu_sample_seconds=0.1, gpu_sample_seconds_peer=0.1, launch_grace_seconds=30.0, say=self.lines.append,
            duration_table=self.duration_table)

    def launches(self):
        with contextlib.suppress(FileNotFoundError):
            with open(self.launch_log, encoding="utf-8") as handle:
                return handle.read().splitlines()
        return []

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

    def dispatch(world, arguments, settings=None, **launch_environment):
        settings = settings or world.settings()
        settings.environment.update({name: str(value) for name, value in launch_environment.items()})
        try:
            request, plan = parse_arguments(arguments)
        except ValueError as error:
            world.lines.append(f"用法错：{error}")
            return EXIT_USAGE
        return run_request(settings, request, plan=plan)

    command = ["selftest-command", "--flag"]
    key = peaks.command_key(command)

    # local-fits / prediction-from-history：两台都空，峰值表里 1G ⇒ 本机，上限 1536M
    world = SelftestWorld(root, "local-fits")
    world.seed_peak(key, GIBIBYTE, "4G")
    code = dispatch(world, ["--", *command])
    launches = world.launches()
    check("local-fits", code == 0 and len(launches) == 1 and " --peer " not in f" {launches[0]} ",
          f"两台都放得下应当派到本机，实际退 {code}、派了「{launches}」；{world.lines[-3:]}")
    check("prediction-from-history", len(launches) == 1 and "--memory-cap 1536M" in launches[0],
          f"峰值表里 1G 应当预测 1G、上限 1536M，实际「{launches}」")

    # peer-when-local-full：本机只剩 2G（1G + 余量 2G 放不下），第二台 40G ⇒ 第二台；输出里不写配置的主机名
    world = SelftestWorld(root, "peer-when-local-full")
    world.seed_peak(key, GIBIBYTE, "4G")
    world.meminfo(world.local, 60, 2)
    code = dispatch(world, ["--", *command])
    launches = world.launches()
    check("peer-when-local-full", code == 0 and len(launches) == 1 and " --peer " in f" {launches[0]} " and "selftest-peer-host-name" not in "\n".join(world.lines),
          f"本机放不下、第二台放得下应当派到第二台（输出里不写配置的主机名），实际退 {code}、派了「{launches}」；{world.lines[-3:]}")

    # waits-then-dispatches：两台都只剩 2G；0.6 秒之后本机空出来 ⇒ 等、重查、派到本机
    world = SelftestWorld(root, "waits-then-dispatches")
    world.seed_peak(key, GIBIBYTE, "4G")
    world.meminfo(world.local, 60, 2)
    world.meminfo(world.peer, 60, 2)
    timer = threading.Timer(0.6, world.meminfo, (world.local, 60, 40))
    timer.start()
    started = time.monotonic()
    code = dispatch(world, ["--", *command])
    elapsed = time.monotonic() - started
    timer.join()
    launches = world.launches()
    waited = any("step=wait" in line for line in world.lines)
    check("waits-then-dispatches", code == 0 and waited and elapsed >= 0.5 and len(launches) == 1 and " --peer " not in f" {launches[0]} ",
          f"两台都放不下应当等、重查、空出来之后派到本机，实际退 {code}、用了 {elapsed:.1f} 秒、{'报了' if waited else '没报'}在等、派了「{launches}」")

    # switch-off-local-only：双机开关没开，本机放不下 ⇒ 不碰第二台、等满 0.5 秒退 242、没派
    world = SelftestWorld(root, "switch-off-local-only")
    world.seed_peak(key, GIBIBYTE, "4G")
    world.configuration(False)
    world.meminfo(world.local, 60, 2)
    settings = world.settings()
    settings.wait_seconds = 0.5
    code = dispatch(world, ["--", *command], settings=settings)
    check("switch-off-local-only", code == EXIT_WAITED_OUT and not world.launches() and not world.ssh_calls(),
          f"双机关着、本机放不下应当只等本机、等满退 {EXIT_WAITED_OUT}、第二台一次都不连，实际退 {code}、派了「{world.launches()}」、ssh {len(world.ssh_calls())} 次")

    # default-prediction：峰值表里没有 ⇒ 默认 8G、上限 12288M
    world = SelftestWorld(root, "default-prediction")
    code = dispatch(world, ["--", *command])
    launches = world.launches()
    check("default-prediction", code == 0 and len(launches) == 1 and "--memory-cap 12288M" in launches[0] and any("memory_history=default" in line for line in world.lines),
          f"查不到历史应当用默认 8G（上限 12288M），实际退 {code}、派了「{launches}」")

    # hit-history：上一次撞了 4G 的顶 ⇒ 预测 6G、上限 9216M
    world = SelftestWorld(root, "hit-history")
    world.seed_peak(key, 4 * GIBIBYTE, "4G")
    code = dispatch(world, ["--", *command])
    launches = world.launches()
    check("hit-history", code == 0 and len(launches) == 1 and "--memory-cap 9216M" in launches[0],
          f"撞过 4G 的顶应当预测 6G（上限 9216M），实际退 {code}、派了「{launches}」")

    # pending-subtracted：本机 MemAvailable 12G，但包装账上一条要 10G 还没涨到量 ⇒ 本机只剩 2G、派第二台
    world = SelftestWorld(root, "pending-subtracted")
    world.seed_peak(key, GIBIBYTE, "4G")
    world.meminfo(world.local, 60, 12)
    world.wrapper_ledger_entry(world.local, 10 * GIBIBYTE)
    code = dispatch(world, ["--", *command])
    launches = world.launches()
    check("pending-subtracted", code == 0 and len(launches) == 1 and " --peer " in f" {launches[0]} ",
          f"本机账上还没涨到量的 10G 要从 MemAvailable 里减掉、派到第二台，实际退 {code}、派了「{launches}」")

    # booking-sum-fits / booking-overflow：本机 10G、第二台 3G。第一条预测 4G 派到本机、还没被包装记账；
    # 第二条预测 4G：10 − 4 = 6 ≥ 4 + 2 ⇒ 照样本机、不等（预测之和放得下就并行）；删掉第二条那一笔，再来一条预测 5G：6 < 5 + 2，第二台 3G 也放不下 ⇒ 等满退 242
    world = SelftestWorld(root, "booking")
    world.seed_peak(key, 4 * GIBIBYTE, "16G")
    world.seed_peak(peaks.command_key(["bigger"]), 5 * GIBIBYTE, "16G")
    world.meminfo(world.local, 60, 10)
    world.meminfo(world.peer, 60, 3)
    settings = world.settings()
    settings.wait_seconds = 0.5
    peer = Peer(settings)
    first = decide(settings, Request(command=command), predict(settings, command, False), peer)
    second = decide(settings, Request(command=command), predict(settings, command, False), peer)
    check("booking-sum-fits", first.host == "local" and second.host == "local",
          f"两条预测之和（8G）加余量放得下本机 10G 应当都派本机、不等，实际 {first.host} 与 {second.host}；{world.lines[-2:]}")
    with contextlib.suppress(FileNotFoundError):
        os.remove(second.entry_path)
    third = decide(settings, Request(command=["bigger"]), predict(settings, ["bigger"], False), peer)
    check("booking-overflow", third.host is None and third.exit_code == EXIT_WAITED_OUT,
          f"本机账上刚派出去 4G，再来预测 5G 的（10 − 4 < 5 + 2）不应当派本机，实际派到 {third.host}、退 {third.exit_code}")

    # gpu-fit：--gpu、显存峰值表里 3072M；本机清场登记有活着的、卡上只空 2000M ⇒ 派第二台（16000M 空），套清场包装
    world = SelftestWorld(root, "gpu-fit")
    world.seed_peak(key, GIBIBYTE, "4G")
    world.seed_gpu_peak(key, 3072)
    world.live_quiesce_holder()
    world.cards(world.local, [(0, 16384, 2000)])
    world.cards(world.peer, [(0, 16384, 16000)])
    code = dispatch(world, ["--gpu", "--", *command])
    launches = world.launches()
    check("gpu-fit", code == 0 and len(launches) == 2 and launches[0] == "quiesce" and " --peer " in f" {launches[1]} ",
          f"本机显存放不下 3072M、第二台放得下应当套清场包装派到第二台，实际退 {code}、派了「{launches}」；{world.lines[-3:]}")

    # gpu-total-before-quiesce：没有活着的清场登记（还没清场），本机卡被服务占着只空 1000M、总量 32768M ⇒ 按总量算、派本机
    world = SelftestWorld(root, "gpu-total-before-quiesce")
    world.seed_peak(key, GIBIBYTE, "4G")
    world.seed_gpu_peak(key, 3072)
    world.cards(world.local, [(0, 32768, 1000)])
    code = dispatch(world, ["--gpu", "--", *command])
    launches = world.launches()
    check("gpu-total-before-quiesce", code == 0 and len(launches) == 2 and " --peer " not in f" {launches[1]} ",
          f"还没清场时应当按卡的总量算、派本机，实际退 {code}、派了「{launches}」")

    # gpu-record：跑的时候本机卡上冒出一个用 5000M 的计算进程 ⇒ 显存峰值表记 5000（这一次的预测用默认值）
    world = SelftestWorld(root, "gpu-record")
    world.seed_peak(key, GIBIBYTE, "4G")
    code = dispatch(world, ["--gpu", "--", *command], FAKE_LAUNCH_APPS="4242, 5000", FAKE_LAUNCH_SECONDS="1")
    row = read_gpu_peak_table(world.gpu_peak_table).get(key)
    check("gpu-record", code == 0 and row is not None and row[0] == 5000 and not row[2] and any("gpu_history=default" in line for line in world.lines),
          f"跑的时候冒出一个用 5000M 的计算进程应当记进显存峰值表（预测用默认），实际退 {code}、记的「{row}」；{world.lines[-2:]}")

    # gpu-oom-marks-hit：内存包装认出显存不够（退 245）⇒ 原样退、记成撞过顶、记预测（默认 8192）
    world = SelftestWorld(root, "gpu-oom-marks-hit")
    world.seed_peak(key, GIBIBYTE, "4G")
    code = dispatch(world, ["--gpu", "--", *command], FAKE_LAUNCH_EXIT=GPU_MEMORY_HIT_EXIT)
    row = read_gpu_peak_table(world.gpu_peak_table).get(key)
    check("gpu-oom-marks-hit", code == GPU_MEMORY_HIT_EXIT and row is not None and row[2] and row[0] == DEFAULT_GPU_PREDICTION_MEBIBYTES,
          f"显存不够（{GPU_MEMORY_HIT_EXIT}）应当原样退、显存峰值表记成撞过顶（{DEFAULT_GPU_PREDICTION_MEBIBYTES}），实际退 {code}、记的「{row}」")

    # never-fits：峰值 100G，比每台的 slice 总上限（60 − 20 = 40G）都大 ⇒ 当场退 241、没派
    world = SelftestWorld(root, "never-fits")
    world.seed_peak(key, 100 * GIBIBYTE, "200G")
    settings = world.settings()
    settings.wait_seconds = 1.5
    started = time.monotonic()
    code = dispatch(world, ["--", *command], settings=settings)
    elapsed = time.monotonic() - started
    check("never-fits", code == EXIT_NEVER_FITS and not world.launches() and elapsed < 1.4,
          f"哪台都放不下应当当场退 {EXIT_NEVER_FITS}、不等、没派，实际退 {code}、用了 {elapsed:.1f} 秒、派了「{world.launches()}」")

    # exit-passthrough：派出去那条链退 7 ⇒ 派发器退 7
    world = SelftestWorld(root, "exit-passthrough")
    world.seed_peak(key, GIBIBYTE, "4G")
    code = dispatch(world, ["--", *command], FAKE_LAUNCH_EXIT=7)
    check("exit-passthrough", code == 7, f"派出去那条链退 7 应当原样退 7，实际退 {code}")

    # peer-cap-clamp：峰值 30G（上限 45G），本机放不下，第二台 PEER_MEMORY_CAP 40G ⇒ 派第二台、上限收到 40960M
    world = SelftestWorld(root, "peer-cap-clamp")
    world.seed_peak(key, 30 * GIBIBYTE, "64G")
    world.meminfo(world.local, 60, 5)
    world.meminfo(world.peer, 80, 60)
    world.configuration(True, peer_cap="40G")
    code = dispatch(world, ["--", *command])
    launches = world.launches()
    check("peer-cap-clamp", code == 0 and len(launches) == 1 and " --peer " in f" {launches[0]} " and "--memory-cap 40960M" in launches[0],
          f"第二台的上限应当收到配置的 PEER_MEMORY_CAP 40G（40960M），实际退 {code}、派了「{launches}」；{world.lines[-2:]}")

    # long-history-single-host：用时表里这条上一次跑了 3 小时，没给 --shard、命令不是写了长活声明的脚本 ⇒ 退 244、没派，报历史用时与出路
    world = SelftestWorld(root, "long-history-single-host")
    world.seed_peak(key, GIBIBYTE, "4G")
    world.seed_duration(key, 3)
    code = dispatch(world, ["--", *command])
    text = "\n".join(world.lines)
    check("long-history-single-host", code == EXIT_LONG_SINGLE_HOST and not world.launches() and "历史用时 3.0 小时" in text
          and "layer0-shard-configuration-check.sh --emit-hosts" in text and "host-resources.py --json" in text and "为什么分不了" in text,
          f"历史用时 3 小时、只占一台应当退 {EXIT_LONG_SINGLE_HOST}、没派、报历史用时与出路，实际退 {code}、派了「{world.launches()}」；{world.lines[-2:]}")

    # long-estimate-single-host：用时表里没有这条，调用方给了 --estimated-hours 3 ⇒ 按预计挂钟判，退 244、没派
    world = SelftestWorld(root, "long-estimate-single-host")
    world.seed_peak(key, GIBIBYTE, "4G")
    code = dispatch(world, ["--estimated-hours", "3", "--", *command])
    text = "\n".join(world.lines)
    check("long-estimate-single-host", code == EXIT_LONG_SINGLE_HOST and not world.launches() and "预计挂钟 3.0 小时" in text,
          f"没有历史、预计挂钟 3 小时、只占一台应当退 {EXIT_LONG_SINGLE_HOST}、没派，实际退 {code}、派了「{world.launches()}」；{world.lines[-2:]}")

    # short-history-passes：上一次跑了 1.5 小时（≤ 2 小时）⇒ 照派本机
    world = SelftestWorld(root, "short-history-passes")
    world.seed_peak(key, GIBIBYTE, "4G")
    world.seed_duration(key, 1.5)
    code = dispatch(world, ["--", *command])
    launches = world.launches()
    check("short-history-passes", code == 0 and len(launches) == 1,
          f"历史用时 1.5 小时应当照派，实际退 {code}、派了「{launches}」；{world.lines[-2:]}")

    # two-host-shards-pass：上一次跑了 3 小时，--shard 1/2 而第二台开着（能派的机器两台）⇒ 照派
    world = SelftestWorld(root, "two-host-shards-pass")
    world.seed_peak(key, GIBIBYTE, "4G")
    world.seed_duration(key, 3)
    code = dispatch(world, ["--shard", "1/2", "--", *command])
    launches = world.launches()
    check("two-host-shards-pass", code == 0 and len(launches) == 1,
          f"历史用时 3 小时、--shard 1/2、两台能派应当照派，实际退 {code}、派了「{launches}」；{world.lines[-2:]}")

    # 被起脚本文件头的长活声明（判法 lib_long_run）：上一次跑了 3 小时、没给 --shard
    declarations = (
        # (格名, 文件头, 该不该派, 派法占几台)
        ("shards-hosts-declared-passes",
         "# long-run-shards: 自证用的按用例切片\n# long-run-resume: 重起同一条命令只补没做完的片\n"
         "# long-run-hosts: 机器清单从 layer0-shard-configuration-check.sh --emit-hosts 读、每台额度从 host-resources.py --json 现算\n", True, "multi"),
        ("unshardable-declared-passes",
         "# long-run-shards: 自证用的按用例切片\n# long-run-resume: 重起同一条命令只补没做完的片\n"
         "# long-run-unshardable: 多机：自证用的一整段、台数固定、额度不现算\n", True, "cannot"),
        ("partial-declaration-single-host", "# long-run-shards: 自证用的按用例切片\n", False, "single"),
    )
    for cell, header, passes, hosts in declarations:
        world = SelftestWorld(root, cell)
        long_script = os.path.join(world.directory, "long-job.sh")
        world.write_executable(long_script, f"#!/usr/bin/env bash\n{header}exit 0\n")
        long_command = ["bash", long_script]
        world.seed_duration(peaks.command_key(long_command), 3)
        code = dispatch(world, ["--", *long_command])
        launches = world.launches()
        text = "\n".join(world.lines)
        if passes:
            ok = code == 0 and len(launches) == 1 and f"hosts={hosts}" in text
        else:
            ok = code == EXIT_LONG_SINGLE_HOST and not launches and "hosts=single" in text and "长活声明不齐" in text and "# long-run-unshardable:" in text
        check(cell, ok, f"文件头「{header.strip()}」应当{'照派、hosts=' + hosts if passes else f'退 {EXIT_LONG_SINGLE_HOST}、没派、报声明不齐'}，"
                        f"实际退 {code}、派了「{launches}」；{world.lines[-2:]}")

    # duration-recorded：派出去那条链跑 1 秒、退 0 ⇒ 用时表里这条记 1–5 秒、退出码 0、本机
    world = SelftestWorld(root, "duration-recorded")
    world.seed_peak(key, GIBIBYTE, "4G")
    code = dispatch(world, ["--", *command], FAKE_LAUNCH_SECONDS="1")
    row = peaks.read_duration_table(world.duration_table).get(key)
    check("duration-recorded", code == 0 and row is not None and 1 <= row[0] <= 5 and row[2] == "0" and row[3] == "local",
          f"跑了 1 秒应当记进用时表（1–5 秒、退 0、local），实际退 {code}、记的「{row}」")

    # usage：没有 -- ⇒ 退 2、没派
    world = SelftestWorld(root, "usage")
    code = dispatch(world, [*command])
    check("usage", code == EXIT_USAGE and not world.launches(), f"没有 -- 应当按用法错退 {EXIT_USAGE}、没派，实际退 {code}、派了「{world.launches()}」")
    return failures, checked


# 弄坏开关 → 打开后至少要红的格（文件头「弄坏开关」那一段，两处要一致）
SWITCH_EXPECTED_CELLS = {
    "local-only": ["peer-when-local-full", "pending-subtracted", "gpu-fit", "peer-cap-clamp"],
    "peer-first": ["local-fits"],
    "no-wait": ["waits-then-dispatches", "switch-off-local-only"],
    "no-reprobe": ["waits-then-dispatches"],
    "ignore-history": ["prediction-from-history", "hit-history"],
    "hit-as-peak": ["hit-history"],
    "no-default": ["default-prediction"],
    "ignore-pending": ["pending-subtracted"],
    "book-by-cap": ["booking-sum-fits"],
    "no-dispatch-ledger": ["booking-overflow"],
    "ignore-gpu": ["gpu-fit"],
    "no-quiesce": ["gpu-fit"],
    "free-only": ["gpu-total-before-quiesce"],
    "no-gpu-record": ["gpu-record", "gpu-oom-marks-hit"],
    "wait-on-hopeless": ["never-fits"],
    "exit-dropped": ["exit-passthrough"],
    "no-peer-cap-clamp": ["peer-cap-clamp"],
    "peer-despite-switch-off": ["switch-off-local-only"],
    "no-usage-check": ["usage"],
    "long-history-ignored": ["long-history-single-host", "partial-declaration-single-host"],
    "long-estimate-ignored": ["long-estimate-single-host"],
    "shards-as-single": ["two-host-shards-pass"],
    "long-run-header-ignored": ["shards-hosts-declared-passes", "unshardable-declared-passes", "partial-declaration-single-host"],
    "long-run-header-unjudged": ["partial-declaration-single-host"],
    "no-duration-record": ["duration-recorded"],
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
        print("  → 怎么办：照上面逐格改 predict / judge_host / decide / launch_arguments / GpuSampler / record_gpu_peak；HOST_DISPATCH_BREAK 设着的话本来就该红")
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
    print(f"  ✓ host-dispatch.py 自证通过（查了 {checked} 格：本机放得下派本机、本机放不下第二台放得下派第二台、两台都放不下等着重查再派、双机关着不碰第二台、"
          f"有历史按历史预测、撞过顶的往上放一档、没历史用默认值、包装账上还没涨到量的从余量里减、派发器自己刚派出去的按预测记账（两条之和放得下就都派、放不下不派）、"
          f"--gpu 按显存余量挑机器并套清场包装、还没清场按卡的总量算、跑完记显存峰值、显存不够记成撞过顶、哪台都放不下当场退 {EXIT_NEVER_FITS}、退出码原样带回、"
          f"第二台的上限收到 PEER_MEMORY_CAP、历史用时或预计挂钟超过 2 小时而只占一台退 {EXIT_LONG_SINGLE_HOST}、不超过 2 小时与两台分片与脚本头写齐了分片多机或分不了照派、脚本头声明不齐退 {EXIT_LONG_SINGLE_HOST}、"
          f"跑完记用时、没有 -- 退 {EXIT_USAGE}；弄坏开关 {len(results)} 个逐个打开都红在点名的格）")
    return 0


if __name__ == "__main__":
    preflight(__file__)
    if sys.argv[1:] == ["--selftest"]:
        sys.exit(selftest())
    if sys.argv[1:] == ["--selftest-cells"]:
        cell_failures, _ = run_cells_in_scratch()
        sys.exit(1 if cell_failures else 0)
    sys.exit(main(sys.argv[1:]))
