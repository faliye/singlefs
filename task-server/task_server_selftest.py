"""task-server 自证的各格（python3 task-server/task_server_lib.py --selftest 起；这一份单独不起）。做法、只在自证里认的变量与弄坏开关写在 task_server_lib.py 文件头。"""
import os
import sys

sys.dont_write_bytecode = True

import concurrent.futures  # noqa: E402
import contextlib  # noqa: E402
import hashlib  # noqa: E402
import io  # noqa: E402
import json  # noqa: E402
import re  # noqa: E402
import shlex  # noqa: E402
import shutil  # noqa: E402
import signal  # noqa: E402
import socket  # noqa: E402
import sqlite3  # noqa: E402
import subprocess  # noqa: E402
import tempfile  # noqa: E402
import threading  # noqa: E402
import time  # noqa: E402
import traceback  # noqa: E402
import types  # noqa: E402
import unittest  # noqa: E402

import task_server_lib as lib  # noqa: E402

FAILURE_PREFIX = "✗ task_server_lib.py 自证："
SESSION_A = "selftest-session-a"
SESSION_B = "selftest-session-b"
SESSION_FORGED = "selftest-session-forged"
DEFAULT_INBOX_TOKEN = "selftest-default-inbox-token"   # 各格客户端缺省带的会话令牌（World.client_environment）：没登会话的格里服务找不到 socket、用不上它
CONFIRM_RETRIES = 3   # World.confirm_submitted：确认的数对不上（这一会儿同会话别的任务起了或收了）照服务打回的数再确认，至多这么多回
GIBIBYTE = 1024 ** 3
MEBIBYTE = 1024 ** 2
RECORD_WINDOW_OVERRUN = 6   # 比 task_server.py 的 RECORD_WINDOW（近几次的名额，5）多一次：量不到的那几次要是占了名额，导进来的那一次就被挤出去
STATE_TIMEOUT_SECONDS = 60.0
POLL_SECONDS = 0.1
# 等待的判法（World.keep_waiting 一处判，各格的等待都经它，不逐格写时限）：一次等待给的秒数是「安静时限」——这么久里这一格的世界一点进展都没有才判红；
# 世界在变（任务与尝试的状态、尝试的日志、假脚本的调用记录在涨）就接着等，机器忙、几趟自证并行时慢一点不会判红。
# 绝对上限 = 安静时限 × WAIT_CEILING_FACTOR（60 秒的等待至多 180 秒，20 秒的至多 60 秒，120 秒的至多 360 秒）：一直在变而永远等不到的（真坏了）到这里为止。
WAIT_CEILING_FACTOR = 3
WAIT_SLICE_SECONDS = 1.0          # 等进程退出时每一片等多久；每片之后问一次 keep_waiting
PROGRESS_SAMPLE_SECONDS = 1.0     # 世界的进展记号至多每隔这么久采一次
# 假世界目录里服务活着就一直涨的记录，不是任务有进展：每一轮排队记的读数，与 NFS 复核每 FAST_NFS_RECHECK_SECONDS 起一次假 ssh 记的那一行
#（任务 7283、7282、7293–7296 里 slow-world-patient-wait 的安静那一头都拖到了绝对上限 18 秒；推的：ssh-calls 算进去时世界永远在变）
PROGRESS_IGNORED_FILES = ("scheduler-reads", "ssh-calls")
# 一格自证整体的绝对上限：比单次等待的最大绝对上限（120 秒 × WAIT_CEILING_FACTOR = 360 秒）大一些；单次等待各自按安静时限判，这一道只兜住死循环
CELL_TIMEOUT_SECONDS = 600
SOCKET_CONNECT_SECONDS = 10.0     # restart-keeps-running：等 wait 连上服务的安静时限
DISCONNECT_NOTICE_SECONDS = 20.0  # restart-keeps-running：服务停着时等 wait 打「连不上」的安静时限（它隔 RECONNECT_SECONDS 2 秒重连一次，留够机器忙时的余地）
UNREACHABLE_NOTICE = "task-server 连不上"   # task 的 wait 重连时往标准错误打的那一句的开头
SLOW_WORLD_BASE_SECONDS = 6.0     # slow-world-patient-wait：这一格的安静时限
SLOW_WORLD_STEP_SECONDS = 2       # 慢世界里命令每一步睡多久（小于安静时限，大于 0）
SLOW_WORLD_STEPS = 5              # 一共几步：总共跑 SLOW_WORLD_STEP_SECONDS × SLOW_WORLD_STEPS 秒，超过安静时限
BLOCK_SCRIPT = 'for round in $(seq 1 600); do [[ -e "$1" ]] && exit 0; sleep 0.1; done; exit 9'
SIMULATED_RELEASE = "release-simulated"   # simulated-two-host-workload 放重任务的那份放行文件（相对假仓根）
HUNDRED_RELEASE = "release-hundred"       # hundred-five-minute-tasks-balanced 放那 100 条的放行文件（相对假仓根）
GRACE_CELL_STALL_SECONDS = 2              # stall-waits-for-grace：卡住阈值；宽限里判到静了它的两倍才放行
GRACE_CELL_GRACE_SECONDS = 20             # stall-waits-for-grace：宽限只要比「判到静了两倍阈值、放行、命令退出」这一下长
# stalled-at-cap-retried：库里记一次跑完的峰值 1024 MiB，上限 1024 × 1.5 = 1536M；假 scope 的 memory.peak 钉在 1536 MiB（到上限）；乘 1.5 重排是 2304M
AT_CAP_SEED_PEAK_BYTES = 1024 * MEBIBYTE
AT_CAP_PEAK_BYTES = 1536 * MEBIBYTE
AT_CAP_PINNED_TEXT = "1536M"
AT_CAP_GROWN_PINNED_TEXT = "2304M"
# busy-cpu-pinned-at-cap-retried、busy-cpu-below-cap-keeps-running 同一套数：库里 1024 MiB、上限 1536M；前一格假 scope 的 memory.current 钉在 1536 MiB，后一格只到它的八分之一
# aged-head-at-slice-total-starts：本机 slice 总上限 50G、slice 自己挂着回收不掉的那一截就是假余量 2G（SELFTEST_MARGIN_GIBIBYTES；任务 1713 那一回量到 0.7G，向上取整是 1G）；
# 峰值 40G 乘 1.5 收到总上限减余量是 48G
AT_TOTAL_CAP_PINNED_TEXT = "49152M"
# 占着到放行为止、判定比各条起止时刻的格用：至多 200 秒（短于一格的时限 CELL_TIMEOUT_SECONDS），机器忙时也不在放行之前自己退出
HOLD_UNTIL_RELEASED_SCRIPT = 'for round in $(seq 1 2000); do [[ -e "$1" ]] && exit 0; sleep 0.1; done; exit 9'
SEEDED_ENDED_AT = 1793318400   # 自证往服务库里记的那几次尝试的结束时刻：固定的一个时刻，比这一格真跑的尝试都早
LOG_QUIET_PINNED_SECONDS = 3000   # stall-clock-from-log-mtime 把在跑那一条的日志修改时刻设到这么多秒以前
AGED_BACKDATE_MARGIN_SECONDS = 60   # backdate_past_aged 把提交时刻往前拨过服务的时限再多这么久
SOCKET_ROOT_VARIABLE = "TASK_SERVER_SELFTEST_SOCKET_ROOT"
SOCKET_ROOT_PREFIX = "tss-sock-"
WORLD_SOCKET_PREFIX = "w-"
LONGEST_RUNTIME_NAME = "install-run"   # 各格 socket 目录里最长的那一个名字（World 的 run 与 install 那一格的 install-run）
RANDOM_NAME_CHARACTERS = 8             # tempfile.mkdtemp 在前缀后面接的随机字符数
SOCKET_PATH_ALLOWANCE = (len("/" + SOCKET_ROOT_PREFIX) + RANDOM_NAME_CHARACTERS + len("/" + WORLD_SOCKET_PREFIX) + RANDOM_NAME_CHARACTERS
                         + len("/" + LONGEST_RUNTIME_NAME + "/") + len(lib.SOCKET_NAME))
FALLBACK_TEMPORARY_BASE = "/tmp"
TOO_DEEP_SOCKET_DIRECTORY = "socket-directory-made-too-deep"
PLACEMENT_WAIT_SECONDS = 20.0   # peer-when-local-full 等那一条收尾的时限：弄坏开关下它一直排着，等满这么久就判红
PASS_QUEUED_COUNT = 4   # prediction-computed-once 与 pass-reads-resources-once 交几条放不下的：读数随条数涨的那种写法在这几条上就分得出
PASS_COUNT = 3          # 那两格看几轮：每轮都现算的写法下每条算 3 次
RECHECK_FAILED_TEXT = "起之前复核没过"   # task_server.py 的 place_queued_into 写进排队原因的那几个字
# 钉住主 agent 定的没记录缺省 4G（task_server_lib.py 的 UNRECORDED_MEMORY_DEFAULT_BYTES）与它乘 1.5 放大的两步：改缺省先改定案，再改这里
UNRECORDED_DEFAULT_PINNED_TEXT = "4096M"
# 各格多机配置 HOSTS 的两项：127.0.0.1 在每一台的网卡上（lo），共用取法 scripts/hosts/multi_host_addresses.py 照这一台网卡上的地址认出它是自己；
# 192.0.2.2 是文档用的保留地址段，当假第二台（假 ssh 不看主机名）
SELFTEST_LOCAL_ADDRESS = "127.0.0.1"
SELFTEST_PEER_ADDRESS = "192.0.2.2"
# 卡的台标签写 HOSTS 里那一台的地址（task_server.py 文件头 ②「GPU」）：各格点名的卡
LOCAL_CARD_ZERO = f"{SELFTEST_LOCAL_ADDRESS}:0:8G"
LOCAL_CARD_ONE = f"{SELFTEST_LOCAL_ADDRESS}:1:8G"
PEER_CARD_ZERO = f"{SELFTEST_PEER_ADDRESS}:0:8G"
OUTSIDE_HOSTS_CARD = "192.0.2.99:0:8G"   # 台标签不是 HOSTS 里的一项（card-labels-by-address）
# 各格多机配置里判法要的其余键（scripts/crash/crash-point-replay-shard-configuration-check.sh 整份判过，共用取法才给第二台的地址）：
# 按台的三个键两台各一项，目录是假的绝对路径（判法不连第二台、不看它们在不在），清场两对写空串，双机开着、GPU 关着；
# SLICE_MEMORY_CAP_BY_HOST 只许一行（set_slice_cap、set_slice_caps 换它）
SELFTEST_MULTI_HOST_KEYS = (f"BUILD_ROOT_BY_HOST={SELFTEST_LOCAL_ADDRESS}=/selftest/this-build-root {SELFTEST_PEER_ADDRESS}=/selftest/peer-build-root\n"
                            f"CARGO_BIN_DIRECTORY_BY_HOST={SELFTEST_LOCAL_ADDRESS}=/selftest/this-cargo-bin {SELFTEST_PEER_ADDRESS}=/selftest/peer-cargo-bin\n"
                            "QUIESCE_STOP_COMMAND=\nQUIESCE_STOPPED_CHECK_COMMAND=\nQUIESCE_START_COMMAND=\nQUIESCE_STARTED_CHECK_COMMAND=\n"
                            "SLICE_MEMORY_CAP_BY_HOST={slice_caps}\nENABLE_ACROSS_MACHINES=1\nENABLE_GPU=0\n")
SELFTEST_SLICE_CAP_TEXT = "50G"   # 各格多机配置 SLICE_MEMORY_CAP_BY_HOST 两台那一项的缺省值：只给钉了上限的命令用（钉值那几格另经 set_slice_cap、set_slice_caps 换）
# pinned-cap-by-host 钉住的两台钉值：SLICE_MEMORY_CAP_BY_HOST 那一项减那台的假余量 2G（SELFTEST_MARGIN_GIBIBYTES；task_server.py 的 pin_value 与 pinned_on_host）；本机写 20G、第二台写 30G。
# 钉值的口径只由这一格钉数，别的钉值格照 control pin-cap 报的此刻钉值核（shown_local_pin）
PINNED_LOCAL_TEXT = "18432M"
PINNED_PEER_TEXT = "28672M"
# 各格假余量里每台此刻的余量（GiB，task_server.py 文件头 ②「余量」）：自证喂的定值，期望照它算；要别的数的格另给
SELFTEST_MARGIN_GIBIBYTES = 2
# margin-measured-per-host：本机余量 1G、第二台余量 3G，两台 slice 总上限都是 50G；记录峰值 40G（上限 60G）的那一条在两台的上限是 50G 减各自的余量
PER_HOST_LOCAL_MARGIN_GIBIBYTES = 1
PER_HOST_PEER_MARGIN_GIBIBYTES = 3
PER_HOST_LOCAL_CAP_TEXT = "50176M"   # 50G − 1G
PER_HOST_PEER_CAP_TEXT = "48128M"    # 50G − 3G
PER_HOST_LOCAL_CAP_BYTES = (50 - PER_HOST_LOCAL_MARGIN_GIBIBYTES) * GIBIBYTE   # 记账同上限（task_server.py 的 booked_at_cap）
PER_HOST_PEER_CAP_BYTES = (50 - PER_HOST_PEER_MARGIN_GIBIBYTES) * GIBIBYTE


def process_memory_stat(anon_bytes, file_bytes=0, kernel_bytes=0):
    """假 scope 的 memory.stat（attempt_runner.py 文件头 ③：进程内存读的是 anon 加 shmem）：anon、file、kernel 照给，shmem 0，末尾带换行。"""
    return f"anon {anon_bytes}\nfile {file_bytes}\nkernel {kernel_bytes}\nshmem 0\n"


def by_host_text(local_text, peer_text):
    """SLICE_MEMORY_CAP_BY_HOST 的值：这一台（SELFTEST_LOCAL_ADDRESS）与另一台（SELFTEST_PEER_ADDRESS）各一项。"""
    return f"{SELFTEST_LOCAL_ADDRESS}={local_text} {SELFTEST_PEER_ADDRESS}={peer_text}"
VOLATILE_SAMPLE_PATH = "/tmp/task-server-selftest-volatile-output"   # outside-shared-path-refused：只出现在命令的参数里，命令不写它
UNRECORDED_GROWTH_PINNED_TEXTS = ("4096M", "6144M", "9216M")
UNRECORDED_BURST_COUNT = 5      # unrecorded-burst-balanced 连交几条没记录的命令（各记 4096M）：两台都放得下时按剩余占比分到两台
# 没跑完的尝试量到的峰值：现查的那一次提前退 1 的命令就是这么多（records/2026-10-30-编译绕过内存总上限.md「服务修复两件合起来验」#8），乘 1.5 是 63M
UNFINISHED_PEAK_BYTES = 42 * MEBIBYTE
# 库里记一次跑完的峰值 1365 MiB：定的上限是 1365 × 1.5 = 2047.5，按 MiB 向上取整 2048M；撞了它乘 1.5 是 3072M（钉的是验收原话里的两个数）
CAP_HIT_SEED_PEAK_BYTES = 1365 * MEBIBYTE
CAP_HIT_PINNED_TEXT = "2048M"
CAP_HIT_GROWN_PINNED_TEXT = "3072M"
# 钉住 task 的 LOG_TAIL_LINES（log 不带 --all 打最后几行）：改它先改这里；每次尝试打的行比它多，截得出来
LOG_TAIL_PINNED_LINES = 200
LOG_LINES_PER_ATTEMPT = 250
# 第 1 次尝试命令自己退 254（被总上限挤掉，同一上限重排），第 2 次退 0：一件两次尝试，各打 LOG_LINES_PER_ATTEMPT 行「attempt-<第几次> line-<行号>」
LOG_SCRIPT = 'for line in $(seq 1 "$1"); do echo "attempt-$TASK_SERVER_ATTEMPT line-$line"; done; [[ "$TASK_SERVER_ATTEMPT" == 1 ]] && exit 254; exit 0'
ABSENT_TASK_ID = 9999   # 这一格的库里没有这个号（一格只交几件）
# 钉住 task_server_lib 的 NOT_SHARDED_ARGUMENTS：带这几个第一个参数的分片入口不分片
UNSHARDED_PINNED_ARGUMENTS = ("--selftest", "--list-items", "--help", "-h")
SHARD_PLAN_EXIT = 5     # 表外的退出码：计划、合并那一步自己退非 0，记 command-failed、wait 照它退
SHARD_MERGE_EXIT = 6
PEER_GONE_RESOURCES = "resources-without-peer.json"   # pinned-peer-unavailable：假第二台跑完时换上的余量（相对仓根写：假第二台照工作目录解）
FAKE_MEMORY_WRAPPER = r'''#!/usr/bin/env bash
# task-server 自证的假 run-with-memory-cap.sh：记一行调用（连同线程包装给的线程数与 CARGO_TARGET_DIR），照命令里 fake-wrapper-exits=<码,码,…|run> 按第几次尝试演退出码；
# run 就起命令，不起真的 cargo、rustc（只放行这一格的假 cargo）；
# fake-wrapper-exit-peak=<字节> 演真包装在命令退出时读的峰值：先往服务给的 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 写这个数；
# fake-wrapper-scene=<码> 演真包装退非 0 时把临时目录搬成现场：建服务给的 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY、写一份 left-by-command、往标准错误打真包装那一句，退这个码；
# fake-wrapper-swap-resources=<文件> 第 1 次尝试时拿那一份换掉这一格的两台余量（演「头一次跑完之后本机腾不出地方了」，下一次换台）
# fake-wrapper-oom-stat 演真包装撞了上限（退 250）那一刻打的内存明细：照 fake-wrapper-exits= 演 250 的那一次先往标准错误打 FAKE_OOM_STAT_LINE 那一行
cap="$1"; shift
threads="" command_word="${1:-}"
if [[ "$(basename -- "${1:-}")" == bash && "$(basename -- "${2:-}")" == capped.sh ]]; then threads="${3:-}"; command_word="${4:-}"; fi
printf 'wrapper task=%s attempt=%s cap=%s admitted=%s threads=%s target=%s\n' "${TASK_SERVER_TASK:-}" "${TASK_SERVER_ATTEMPT:-}" "$cap" \
  "${RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER:-}" "$threads" "${CARGO_TARGET_DIR:-}" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
plan="" scene_exit="" oom_stat=""
for word in "$@"; do
  case "$word" in
    fake-wrapper-oom-stat) oom_stat=1 ;;
    fake-wrapper-exits=*) plan="${word#fake-wrapper-exits=}" ;;
    fake-wrapper-exit-peak=*) [[ -n "${RUN_WITH_MEMORY_CAP_PEAK_OUTPUT:-}" ]] && printf '%s\n' "${word#fake-wrapper-exit-peak=}" > "$RUN_WITH_MEMORY_CAP_PEAK_OUTPUT" ;;
    fake-wrapper-scene=*) scene_exit="${word#fake-wrapper-scene=}" ;;
    fake-wrapper-swap-resources=*)
      if [[ "${TASK_SERVER_ATTEMPT:-1}" == 1 ]]; then
        cp -- "${word#fake-wrapper-swap-resources=}" "$TASK_SERVER_SELFTEST_WORLD/resources.json.partial" \
          && mv -- "$TASK_SERVER_SELFTEST_WORLD/resources.json.partial" "$TASK_SERVER_SELFTEST_WORLD/resources.json" || exit 97
      fi ;;
  esac
done
if [[ -n "$scene_exit" ]]; then
  if [[ -n "${RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY:-}" ]]; then
    mkdir -p -- "$RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY" && : > "$RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY/left-by-command" || exit 97
    echo "run-with-memory-cap: 包装退 $scene_exit：这一条命令的临时目录搬进了这一次尝试的现场目录：$RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY" >&2
  fi
  exit "$scene_exit"
fi
if [[ -n "$plan" ]]; then
  IFS=, read -r -a codes <<<"$plan"
  position=$(( ${TASK_SERVER_ATTEMPT:-1} - 1 ))
  (( position < ${#codes[@]} )) || position=$(( ${#codes[@]} - 1 ))
  if [[ -n "$oom_stat" && "${codes[position]}" == 250 ]]; then
    echo "RUN_WITH_MEMORY_CAP_OOM_STAT anon=943718400 file=10200547328 file_dirty=8589934592 file_writeback=268435456 shmem=0 oom_kill=1" >&2
  fi
  if [[ "${codes[position]}" != run ]]; then exit "${codes[position]}"; fi
fi
fake_cargo="$(dirname -- "$TASK_SERVER_SELFTEST_WORLD")/fake-cargo/cargo"
case "$(basename -- "$command_word")" in
  cargo|rustc)
    if [[ "$(command -v -- "$command_word")" != "$fake_cargo" ]]; then echo "假内存包装：自证里不起真的 $command_word" >&2; exit 97; fi ;;
esac
exec "$@"
'''
FAKE_PEER_RUNNER = r'''#!/usr/bin/env bash
# task-server 自证的假 multi-host-run.sh：--peer 记一行调用，照命令里 fake-peer-exit=<码> 或 fake-wrapper-exits= 演结局；
# 每次被起先记一行 peer-args（原样的参数）；只认 --peer、--memory-cap、--label 与 --：送与取回那一套（--send、--fetch 这类）照真的那样认不出、退 2；
# 命令里有 fake-peer-gone=<相对工作目录的文件> 的，演结局之前拿那一份换掉这一格的两台余量（演「跑到一半第二台用不了了」）；
# 命令里有 fake-peer-wait=<假仓根里的相对名> 的，演「第二台那条命令一直在跑」：等假仓根里那份放行文件在了再退 0（至多 200 秒、短于一格的时限，等不到退 9）；
# 命令里有 fake-peer-step-exit=<码> 的，照真的那样往标准错误打第二台那条命令的结束行「MULTI_HOST_RUN_STEP step=peer-command event=end … exit=<码>」，
# 没写 fake-peer-exit 的退这个码；fake-peer-memory-peak=<字节>、fake-peer-build-peak=<字节> 照真的那样打 MULTI_HOST_RUN_PEER_MEMORY_PEAK、
# MULTI_HOST_RUN_PEER_BUILD_PEAK 那一行（这几行在演结局之前打）；
# 命令里有 fake-peer-cleanup=<状态> 的，往 TASK_SERVER_PEER_CLEANUP_FILE 写 {"status": <状态>}（pending 演远端清理回执一直没确认，服务保留这一次尝试的占用）；
# 别的不起命令、当场退 0（第二台上的尝试一起就收尾）；
# 命令里有自证的假分片入口 fake-sharded.py 或记号词 fake-peer-run 时照真的那样跑：在这一刻的工作目录里起命令（第二台在同一棵共享的树里、同一个绝对路径上），
# 不带认台的角色变量（两台对称）；编译目录照真的那样在第二台本地盘上这一趟新建（这一格临时目录旁边的 <格的临时目录>.peer-local/builds/<标签>/target，
# 不在假共享根下：假共享根就是这一格的临时目录）、经 CARGO_TARGET_DIR 与 MULTI_HOST_RUN_PEER_SCRATCH 交给命令、跑完连目录删；记一行 peer-run（工作目录、编译目录）。
# --keep-build-directory、--reuse-build-directory <标签>、--remove-build-directory <标签> 照真的那样认（multi-host-run.sh 文件头「留着的编译目录」）：只在 fake-sharded.py / fake-peer-run 那一支
# 建编译目录，那一支里 fake-wrapper-exits= 的码在建了编译目录之后才演（不跑命令），打真的那样的第二台那条命令结束行；带 --keep-build-directory 的不删、打 MULTI_HOST_RUN_PEER_BUILD_KEPT；
# --reuse-build-directory 点名的不在退 255；--remove-build-directory 记一行 peer-remove、删 builds/<标签>、退 0。
# 每一趟（--peer）照真的那样建这一趟的本地目录 builds/<标签>/ 与 runs/ 下三个文件，退出时（连同被 TERM 停、退 143）删掉、回读不在了打真的那样的 ⑥ 那一行
# （fake_peer_trip_cleanup；带 --keep-build-directory 而命令等到了退出码的留着 builds/ 那一份）。被 TERM 停时照真的那样先停那条命令、等它退，再退 143
# （fake_peer_stop_and_exit；fake-sharded.py / fake-peer-run 那一支的命令放在后台、wait 它，TERM 一到就接手，不等前台命令自己跑完）。
if [[ "${1:-}" == --remove-build-directory ]]; then
  printf 'peer-remove label=%s\n' "${2:-}" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
  rm -rf -- "$(dirname -- "$TASK_SERVER_SELFTEST_WORLD").peer-local/builds/${2:?}"
  exit 0
fi
printf 'peer-args task=%s %s\n' "${TASK_SERVER_TASK:-}" "$*" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
cap="" label="" keep=0 reuse=""
while (( $# )); do
  case "$1" in
    --peer) shift ;;
    --memory-cap) cap="$2"; shift 2 ;;
    --label) label="$2"; shift 2 ;;
    --keep-build-directory) keep=1; shift ;;
    --reuse-build-directory) reuse="$2"; shift 2 ;;
    --) shift; break ;;
    *) echo "假 multi-host-run.sh：认不出的参数「$1」（命令要写在 -- 后面）" >&2; exit 2 ;;
  esac
done
# 照真的那样（multi-host-run.sh 文件头 ⑥）：每一趟在假第二台本地盘上建这一趟的本地目录 builds/<标签>/ 与 runs/ 下进程号、退出码、输出三个文件，
# 退出时（连同被 TERM 停）删掉、回读不在了往标准错误打 ⑥ 那一行；带 --keep-build-directory 而命令等到了退出码的留着 builds/ 那一份、只删 runs/ 下三个
peer_local="$(dirname -- "$TASK_SERVER_SELFTEST_WORLD").peer-local"
command_exited=0
# 命令里有 fake-peer-before-trip=<假仓根里的相对名> 的，演真的停在建这一趟的本地目录之前（比工具链、查共享根与 ssh 那几步）：记一行 peer-before-trip，
# 等假仓根里那份放行文件在了再往下（至多 200 秒）；这时被 TERM 停照真的那样什么都不删、不打 ⑥（还没建），清理回执仍是服务写的 not-started
for word in "$@"; do
  if [[ "$word" == fake-peer-before-trip=* ]]; then
    printf 'peer-before-trip task=%s label=%s\n' "${TASK_SERVER_TASK:-}" "$label" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
    for round in $(seq 1 2000); do [[ -e "$(dirname -- "$TASK_SERVER_SELFTEST_WORLD")/repo/${word#fake-peer-before-trip=}" ]] && break; sleep 0.1; done
  fi
done
if [[ -n "$label" ]]; then
  mkdir -p -- "$peer_local/builds/$label" "$peer_local/runs" && : > "$peer_local/runs/$label.pid" && : > "$peer_local/runs/$label.exit" \
    && : > "$peer_local/runs/$label.log" || exit 255
fi
fake_peer_trip_cleanup() {
  [[ -n "$label" ]] || return 0
  rm -f -- "$peer_local/runs/$label.pid" "$peer_local/runs/$label.exit" "$peer_local/runs/$label.log"
  if (( keep && command_exited )); then
    if [[ -n "$reuse" && "$reuse" != "$label" ]]; then rm -rf -- "$peer_local/builds/$label"; fi
    echo "  · ⑥ 留着第二台这一趟的本地目录 builds/${reuse:-$label}（连同编译目录，--keep-build-directory），删了 runs/ 下进程号、退出码、输出三个文件（回读过）" >&2
    return 0
  fi
  rm -rf -- "$peer_local/builds/$label"
  if [[ -n "$reuse" ]]; then rm -rf -- "$peer_local/builds/$reuse"; fi
  if [[ ! -e "$peer_local/builds/$label" && ! -e "$peer_local/runs/$label.pid" && ! -e "$peer_local/runs/$label.exit" && ! -e "$peer_local/runs/$label.log" ]]; then
    echo "  · ⑥ 删了第二台这一趟的本地目录 builds/$label（连同编译目录）与 runs/ 下进程号、退出码、输出三个文件（回读不在了）" >&2
  fi
}
peer_command_pid=""
fake_peer_stop_and_exit() {   # 照真的那样（multi-host-run.sh 的 multi_host_stop_and_exit）：先停那条命令、等它退，再 exit 143（EXIT 上删这一趟的本地目录）
  trap '' TERM
  if [[ -n "$peer_command_pid" ]]; then
    kill -TERM "$peer_command_pid" 2>/dev/null
    wait "$peer_command_pid" 2>/dev/null
  fi
  exit 143
}
trap fake_peer_trip_cleanup EXIT
trap fake_peer_stop_and_exit TERM
printf 'peer task=%s attempt=%s cap=%s label=%s admitted=%s\n' "${TASK_SERVER_TASK:-}" "${TASK_SERVER_ATTEMPT:-}" "$cap" "$label" \
  "${MULTI_HOST_RUN_ADMITTED_BY_TASK_SERVER:-}" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
printf 'peer-words %s\n' "$*" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
plan="" gone="" wait_for="" sharded=0 peer_exit="" step_exit="" cleanup_state=""
for word in "$@"; do
  case "$word" in
    fake-peer-cleanup=*) cleanup_state="${word#fake-peer-cleanup=}" ;;
    fake-peer-exit=*) peer_exit="${word#fake-peer-exit=}" ;;
    fake-peer-step-exit=*) step_exit="${word#fake-peer-step-exit=}" ;;
    fake-peer-memory-peak=*) echo "MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=${word#fake-peer-memory-peak=}" >&2 ;;
    fake-peer-build-peak=*) echo "MULTI_HOST_RUN_PEER_BUILD_PEAK bytes=${word#fake-peer-build-peak=}" >&2 ;;
    fake-wrapper-exits=*) plan="${word#fake-wrapper-exits=}" ;;
    fake-peer-gone=*) gone="${word#fake-peer-gone=}" ;;
    fake-peer-wait=*) wait_for="${word#fake-peer-wait=}" ;;
    *fake-sharded.py|fake-peer-run) sharded=1 ;;
  esac
done
if [[ -n "$cleanup_state" && -n "${TASK_SERVER_PEER_CLEANUP_FILE:-}" ]]; then
  printf '{"status": "%s", "label": "%s"}\n' "$cleanup_state" "$label" > "$TASK_SERVER_PEER_CLEANUP_FILE"
fi
if [[ -n "$step_exit" ]]; then
  echo "MULTI_HOST_RUN_STEP step=peer-command event=end elapsed_seconds=0.100 seconds=0.050 exit=$step_exit" >&2
  exit "${peer_exit:-$step_exit}"
fi
[[ -n "$peer_exit" ]] && exit "$peer_exit"
if [[ -n "$gone" ]]; then
  cp -- "$gone" "$TASK_SERVER_SELFTEST_WORLD/resources.json.partial" && mv -- "$TASK_SERVER_SELFTEST_WORLD/resources.json.partial" "$TASK_SERVER_SELFTEST_WORLD/resources.json" || exit 97
fi
planned=""
if [[ -n "$plan" ]]; then
  IFS=, read -r -a codes <<<"$plan"
  position=$(( ${TASK_SERVER_ATTEMPT:-1} - 1 ))
  (( position < ${#codes[@]} )) || position=$(( ${#codes[@]} - 1 ))
  if [[ "${codes[position]}" != run ]]; then planned="${codes[position]}"; fi
fi
if [[ -n "$planned" ]] && (( ! sharded )); then exit "$planned"; fi
root="$(dirname -- "$TASK_SERVER_SELFTEST_WORLD")"
if (( sharded )); then
  scratch="$root.peer-local/builds/${reuse:-$label}"
  if [[ -n "$reuse" && ! -d "$scratch/target" ]]; then echo "假 multi-host-run.sh：--reuse-build-directory 点名的 builds/$reuse 不在" >&2; exit 255; fi
  mkdir -p -- "$scratch/target" || exit 255
  printf 'peer-run task=%s cwd=%s target=%s\n' "${TASK_SERVER_TASK:-}" "$(pwd -P)" "$scratch/target" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
  if [[ -n "$planned" ]]; then
    code="$planned"
  else
    # 第二台是另一台机器：本机这个进程的 TASK_SERVER_SHARD_OUTPUTS 到不了那里，命令读得到它只靠 "$@" 开头的 env 带过去的那几个（弄坏开关 shard-outputs-not-carried 下不带）
    # 照真的那样放到后台、wait 它：被 TERM 停时 trap 当场接手（停它、等它退），不等前台命令自己跑完（单元是 KillMode=mixed，TERM 只到这一层）
    ( unset TASK_SERVER_SHARD_OUTPUTS; export CARGO_TARGET_DIR="$scratch/target" MULTI_HOST_RUN_PEER_SCRATCH="$scratch"; exec "$@" ) &
    peer_command_pid=$!
    wait "$peer_command_pid"
    code=$?
    peer_command_pid=""
  fi
  [[ -z "$planned" ]] || echo "MULTI_HOST_RUN_STEP step=peer-command event=end elapsed_seconds=0.100 seconds=0.050 exit=$code" >&2
  command_exited=1   # 第二台那条命令等到了退出码：带 --keep-build-directory 的 ⑥ 只删 runs/ 下三个
  if (( keep )); then
    echo "MULTI_HOST_RUN_PEER_BUILD_KEPT label=${reuse:-$label}" >&2
  else
    rm -rf -- "$scratch"
  fi
  exit "$code"
fi
if [[ -n "$wait_for" ]]; then
  release="$root/repo/$wait_for"
  for round in $(seq 1 2000); do [[ -e "$release" ]] && exit 0; sleep 0.1; done
  exit 9
fi
exit 0
'''
FAKE_CARGO = r'''#!/usr/bin/env bash
# task-server 自证的假 cargo：记下起它的目录与 CARGO_TARGET_DIR；没给编译目录就照 cargo 的默认在起它的目录下建 target/，给了就往那里写一份产物
printf 'cargo dir=%s target=%s args=%s\n' "$PWD" "${CARGO_TARGET_DIR:-}" "$*" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
target="${CARGO_TARGET_DIR:-$PWD/target}"
mkdir -p "$target/debug" && head -c 65536 /dev/zero > "$target/debug/selftest-artifact"
'''
FAKE_SHARDED = r'''#!/usr/bin/env python3
# task-server-shards: *
"""task-server 自证的假分片入口：python3 fake-sharded.py run <规格>。规格是同目录的 JSON：{"shards": [{"pin", "exits": [码或 "run"…], "first_attempt_delay": 秒,
"block": 放行文件的绝对路径, "label", "exit_peak": 字节}…], "plan_text": 计划的原文（不给照 shards 写）, "plan_exit": 码, "merge_exit": 码}。
一片给了 exit_peak 的，跑成（run）那一次往服务给的 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 写这个数（演真包装在命令退出时读的峰值，服务照它记这一次尝试的 memory_peak）。
计划那一步写 plan.json（每片写 label，规格里那一片写了 pin 的连 pin 一起写）、
退 plan_exit；每片按第几次尝试演退出码，往调用记录写一行，给了 block 的先等那份文件在（至多 60 秒，等不到退 9；退出码不是 run 的同样先等），run 就在产物目录写 done-<片号>；
合并那一步读 outputs.tsv 与 TASK_SERVER_SHARD_MISSING、往调用记录写一行（present、missing）：每一片的 done 都在退 merge_exit，
缺的恰好是点名的那几片退 77（本次未跑），别的退 9。
服务没按分片起的（带 --selftest 这类参数的，或弄坏开关 sharding-unrecognized）整件当普通命令跑：不读规格，记一行 whole（连同第一个参数）、退 0。"""
import json
import os
import sys
import time


def record(line):
    with open(os.path.join(os.environ["TASK_SERVER_SELFTEST_WORLD"], "calls"), "a", encoding="utf-8") as handle:
        handle.write(line + "\n")


def main(arguments):
    phase = os.environ.get("TASK_SERVER_SHARD_PHASE", "")
    task = os.environ.get("TASK_SERVER_TASK", "")
    if not phase:
        record(f"sharded-whole task={task} first={arguments[0] if arguments else ''}")
        return 0
    with open(arguments[1], encoding="utf-8") as handle:
        spec = json.load(handle)
    attempt = int(os.environ.get("TASK_SERVER_ATTEMPT", "1"))
    if phase == "plan":
        record(f"sharded-plan task={task}")
        text = spec.get("plan_text") or json.dumps({"shards": [dict({"label": shard.get("label", "")}, **({"pin": shard["pin"]} if "pin" in shard else {}))
                                                               for shard in spec["shards"]]})
        with open(os.path.join(os.environ["TASK_SERVER_SHARD_PLAN_DIRECTORY"], "plan.json"), "w", encoding="utf-8") as handle:
            handle.write(text)
        return spec.get("plan_exit", 0)
    if phase == "shard":
        index = int(os.environ["TASK_SERVER_SHARD"].split("/")[0])
        shard = spec["shards"][index]
        exits = shard.get("exits") or ["run"]
        code = exits[min(attempt, len(exits)) - 1]
        if attempt == 1 and shard.get("first_attempt_delay"):
            time.sleep(shard["first_attempt_delay"])
        record(f"sharded-shard task={task} index={index} attempt={attempt} host={os.environ.get('TASK_SERVER_SHARD_HOST', '')} code={code}")
        if shard.get("block"):   # 退出码不是 run 的也先等放行（演跑了一阵才没成）
            for _ in range(600):
                if os.path.exists(shard["block"]):
                    break
                time.sleep(0.1)
            else:
                return 9
        if code != "run":
            return int(code)
        if shard.get("exit_peak") and os.environ.get("RUN_WITH_MEMORY_CAP_PEAK_OUTPUT"):   # 演真包装在命令退出时读的峰值（与假包装的 fake-wrapper-exit-peak= 同一份文件）
            with open(os.environ["RUN_WITH_MEMORY_CAP_PEAK_OUTPUT"], "w", encoding="utf-8") as handle:
                handle.write(f"{int(shard['exit_peak'])}\n")
        output = os.environ["TASK_SERVER_SHARD_OUTPUT_DIRECTORY"]
        os.makedirs(output, exist_ok=True)
        with open(os.path.join(output, f"done-{index}"), "w", encoding="utf-8") as handle:
            handle.write(os.environ.get("TASK_SERVER_SHARD_HOST", "") + "\n")
        return 0
    if phase == "merge":
        with open(os.environ["TASK_SERVER_SHARD_OUTPUTS"], encoding="utf-8") as handle:
            lines = [line.split("\t") for line in handle.read().splitlines() if line]
        present = sorted(int(index) for index, _, directory in lines if os.path.exists(os.path.join(directory, f"done-{index}")))
        missing = sorted(int(index) for index in os.environ.get("TASK_SERVER_SHARD_MISSING", "").split(",") if index)
        record(f"sharded-merge task={task} present={','.join(str(index) for index in present)} missing={','.join(str(index) for index in missing)} count={len(spec['shards'])}")
        everything = list(range(len(spec["shards"])))
        if present == everything:
            return spec.get("merge_exit", 0)
        if missing and sorted(present + missing) == everything:
            print(f"缺第 {','.join(str(index) for index in missing)} 片（人手单独取消、没有产物）：本次未跑", file=sys.stderr)
            return 77
        return 9
    print(f"假分片入口认不出这一步 {phase}", file=sys.stderr)
    return 2


sys.exit(main(sys.argv[1:]))
'''
FAKE_GPU_QUIESCE = r'''#!/usr/bin/env bash
# task-server 自证的假 gpu-quiesce-run.sh：记一行调用（交给命令的卡），照命令里 fake-quiesce=<结局> 演清场的结局：
#   card-fault（坏卡清单点名 127.0.0.1:0）、card-fault-on:<那一项>（分到那一项时坏卡清单点名它的台与序号，没分到照跑）、
#   card-fault-names:<台>:<序号>（不管分到哪张，坏卡清单都点名这一张）、card-fault-unlisted（清场那一行没有坏卡清单）、read-back-failed、block:<放行文件>、
#   card-fault-on-else-block:<那一项>=<放行文件>（分到那一项时同 card-fault-on，没分到就等放行文件在了再退 0：换卡之后的那一次一直在跑）
printf 'quiesce task=%s attempt=%s cards=%s\n' "${TASK_SERVER_TASK:-}" "${TASK_SERVER_ATTEMPT:-}" "${GPU_CARDS_BY_PRIORITY:-}" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
[[ "${1:-}" == -- ]] && shift
for word in "$@"; do
  case "$word" in
    fake-quiesce=card-fault)
      echo "GPU_QUIESCE step=quiesce result=card-fault faulty-cards=127.0.0.1:0 cards=127.0.0.1/0/selftest-bus/xid-79" >&2
      echo "GPU_QUIESCE step=command result=not-started" >&2
      exit 78 ;;
    fake-quiesce=card-fault-unlisted)
      echo "GPU_QUIESCE step=quiesce result=card-fault cards=127.0.0.1/1/selftest-bus/xid-79" >&2
      echo "GPU_QUIESCE step=command result=not-started" >&2
      exit 78 ;;
    fake-quiesce=card-fault-names:*)
      named="${word#fake-quiesce=card-fault-names:}"
      echo "GPU_QUIESCE step=quiesce result=card-fault faulty-cards=$named cards=${named%:*}/${named##*:}/selftest-bus/xid-62" >&2
      echo "GPU_QUIESCE step=command result=not-started" >&2
      exit 78 ;;
    fake-quiesce=read-back-failed)
      echo "GPU_QUIESCE step=quiesce result=read-back-failed holders=0" >&2
      exit 78 ;;
    fake-quiesce=card-fault-on:*)
      entry="${word#fake-quiesce=card-fault-on:}"
      if [[ " ${GPU_CARDS_BY_PRIORITY:-} " == *" $entry "* ]]; then
        identity="${entry%:*}"
        echo "GPU_QUIESCE step=quiesce result=card-fault faulty-cards=$identity cards=${identity%:*}/${identity##*:}/selftest-bus/xid-62" >&2
        echo "GPU_QUIESCE step=command result=not-started" >&2
        exit 78
      fi
      exit 0 ;;
    fake-quiesce=card-fault-on-else-block:*)
      spec="${word#fake-quiesce=card-fault-on-else-block:}"
      entry="${spec%%=*}"
      release="${spec#*=}"
      if [[ " ${GPU_CARDS_BY_PRIORITY:-} " == *" $entry "* ]]; then
        identity="${entry%:*}"
        echo "GPU_QUIESCE step=quiesce result=card-fault faulty-cards=$identity cards=${identity%:*}/${identity##*:}/selftest-bus/xid-62" >&2
        echo "GPU_QUIESCE step=command result=not-started" >&2
        exit 78
      fi
      for round in $(seq 1 600); do [[ -e "$release" ]] && exit 0; sleep 0.1; done
      exit 9 ;;
    fake-quiesce=block:*)
      release="${word#fake-quiesce=block:}"
      for round in $(seq 1 600); do [[ -e "$release" ]] && exit 0; sleep 0.1; done
      exit 9 ;;
  esac
done
exec "$@"
'''
FAKE_NVIDIA_SMI = r'''#!/usr/bin/env bash
# task-server 自证的假 nvidia-smi：--query-compute-apps 照这一格的 card-busy-pids 与 card-busy-pids-<名字> 那几份（一行一个进程号）打「<进程号>, 100」
#（演卡上有这些进程的计算负载），一份都没有就一行不打；--query-gpu=utilization.gpu 照 -i 给的每个序号打 card-utilization-<序号> 那一份的内容
#（演卡的利用率），有一个序号没有那一份就退 9（演利用率读不到）；别的用法退 9
case "$*" in
  *--query-compute-apps*)
    for listed in "$TASK_SERVER_SELFTEST_WORLD"/card-busy-pids "$TASK_SERVER_SELFTEST_WORLD"/card-busy-pids-*; do
      [[ -f "$listed" ]] && sed 's/$/, 100/' "$listed"
    done
    exit 0 ;;
  *--query-gpu=utilization.gpu*)
    indexes=""
    while (( $# )); do
      [[ "$1" == -i ]] && indexes="${2:-}"
      shift
    done
    [[ -n "$indexes" ]] || exit 9
    for index in ${indexes//,/ }; do
      [[ -f "$TASK_SERVER_SELFTEST_WORLD/card-utilization-$index" ]] || exit 9
      cat -- "$TASK_SERVER_SELFTEST_WORLD/card-utilization-$index"
    done
    exit 0 ;;
  *) exit 9 ;;
esac
'''
# 自证给疑似卡住那两格用的 GPU 跑法（名字照 GPU_SCRIPT_PATTERNS 认成要卡、服务套清场）：给了 $1 就把自己的进程号写进去（演卡上有它的计算负载），之后每 0.25 秒打一行
SUSPECT_GPU_SCRIPT_NAME = "crash-amplification-suspect-selftest.sh"
SUSPECT_GPU_SCRIPT = r'''#!/usr/bin/env bash
[[ -n "${1:-}" ]] && echo "$$" >> "$1"
for round in $(seq 1 40); do echo "tick $round"; sleep 0.25; done
'''
FAKE_SSH = r'''#!/usr/bin/env bash
# task-server 自证的假 ssh（TASK_SERVER_PEER_SSH；task_server_lib.py 的 peer_shell）：跳过 -o 选项与主机名，往 ssh-calls 记一行；这一格世界目录里的几个文件演第二台：
# fake-ssh-unreachable 在时一律退 255（连不上）；stat -f（挂载那一项）照 fake-ssh-mount-exit 的内容退（不在退 0），退 0 时打 fake-ssh-mount-type 的内容
# （不在打 nfs：挂着；写 autofs 演自动挂载的占位没挂上）；mountpoint -q（弄坏开关 nfs-mount-placeholder-accepted 那一支）照 fake-ssh-mount-exit 退；
# fake-ssh-peer-write-lost 在时第二台写探针那一句不写
#（演第二台写的本机读不回）；fake-ssh-hang 在时复核挂载的那一句（stat -f）把自己的进程号写进 fake-ssh-hang-pid、睡着不退（演停服务时还在跑的探测子进程）；
# fake-ssh-read-delay 在时头一个读探针的（for try in …）把它改名成 fake-ssh-read-delay.taken、先睡里面写的秒数再读（演两趟探针交错）；
# fake-ssh-hang-once 在时头一回复核挂载的那一句（stat -f）把它改名成 fake-ssh-hang-once.taken、睡着不退（演第二台一次慢，ssh 到时限被停），之后照常回；
# 别的在本机 bash -c 跑（假共享根在本机，「两台」读写的是同一份）
while [[ "${1:-}" == -o ]]; do shift 2; done
host="${1:-}"; shift
script="$*"
printf 'ssh host=%s script=%s\n' "$host" "$script" >> "$TASK_SERVER_SELFTEST_WORLD/ssh-calls"
if [[ -e "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-unreachable" ]]; then echo "ssh: connect to host $host: Connection refused（自证）" >&2; exit 255; fi
case "$script" in
  "for try in "*)
    if mv -- "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-read-delay" "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-read-delay.taken" 2>/dev/null; then
      sleep "$(cat "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-read-delay.taken")"
    fi ;;
  "mountpoint -q -- "*) exit "$(cat "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-mount-exit" 2>/dev/null || echo 0)" ;;
  "stat -f -c %T -- "*)
    if [[ -e "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-hang" ]]; then echo "$$" > "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-hang-pid"; exec sleep 120; fi
    if mv -- "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-hang-once" "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-hang-once.taken" 2>/dev/null; then exec sleep 120; fi
    code="$(cat "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-mount-exit" 2>/dev/null || echo 0)"
    if [[ "$code" != 0 ]]; then echo "stat: cannot read file system information: No such device（自证）" >&2; exit "$code"; fi
    cat "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-mount-type" 2>/dev/null || echo nfs
    exit 0 ;;
  "printf %s "*) [[ -e "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-peer-write-lost" ]] && exit 0 ;;
esac
exec bash -c "$script"
'''
FAKE_SYSTEMD_RUN_REFUSING = r'''#!/usr/bin/env bash
# task-server 自证的假 systemd-run：一律起不来
echo "假 systemd-run：起不来（自证）" >&2
exit 1
'''
FAKE_SYSTEMD_RUN_SELECTIVE = r'''#!/usr/bin/env bash
# task-server 自证的假 systemd-run（failed-start-not-counted）：工作目录以 /refuse-start 收尾的起不来（演起没成），别的照原样交给真的 systemd-run
for argument in "$@"; do
  if [[ "$argument" == --working-directory=*/refuse-start ]]; then
    echo "假 systemd-run：工作目录 ${argument#--working-directory=} 起不来（自证）" >&2
    exit 1
  fi
done
exec systemd-run "$@"
'''
FAKE_SYSTEMD_RUN_DELAYED = r'''#!/usr/bin/env bash
# task-server 自证的假 systemd-run（TS-06）：照原样交给真的 systemd-run 起单元，起成之后不回执、一直睡（演「管理器收下了、调用端回执超时」）
systemd-run "$@" || exit $?
exec sleep 30
'''
FAKE_SYSTEMD_RUN_SILENT = r'''#!/usr/bin/env bash
# task-server 自证的假 systemd-run（TS-06）：什么都不起、一直睡（演「回执超时、管理器也没收下」）
exec sleep 30
'''
FAKE_PRUNE_CHECK = r'''#!/usr/bin/env bash
# task-server 自证的假 prune-server/control check（TASK_SERVER_PRUNE_SERVER_CHECK）：记一行调用，照这一格世界目录里 prune-check-exit 的内容退（不在退 0：通）
echo "prune-check cwd=$PWD" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
code="$(cat "$TASK_SERVER_SELFTEST_WORLD/prune-check-exit" 2>/dev/null || echo 0)"
[[ "$code" == 0 ]] || echo "  ✗ prune-server 不通（自证）" >&2
exit "$code"
'''
FAKE_TEMPORARY_SERVICE = r'''#!/usr/bin/env python3
"""task-server 自证的假 scripts/prune-server/prune_server_client_selftest.py：认 --with-temporary-service [--server-binary <路径>] -- <命令…>，记一行、照原样起 -- 后面那条；
自带临时服务的自证（--selftest、--selftest-families，后面的参数不看）当场退 0。"""
import os
import sys

arguments = sys.argv[1:]
if arguments[:1] in (["--selftest"], ["--selftest-families"]):
    sys.exit(0)
if arguments[:1] != ["--with-temporary-service"]:
    sys.exit(2)
arguments = arguments[1:]
while arguments[:1] == ["--server-binary"]:
    arguments = arguments[2:]
if arguments[:1] != ["--"] or len(arguments) < 2:
    sys.exit(2)
with open(os.path.join(os.environ["TASK_SERVER_SELFTEST_WORLD"], "calls"), "a", encoding="utf-8") as handle:
    handle.write(f"temporary-service task={os.environ.get('TASK_SERVER_TASK', '')} command={arguments[1]}\n")
os.execvp(arguments[1], arguments[1:])
'''
# temporary-service-own-server-compiles 现写的假 scripts/prune-server/prune-server-load-simulation.py（别的格不用它，所以不放进 World 的 fakes）
LOAD_SIMULATION_SCRIPT = "prune-server-load-simulation.py"
FAKE_LOAD_SIMULATION = r'''#!/usr/bin/env python3
"""task-server 自证的假 scripts/prune-server/prune-server-load-simulation.py：认 --selftest（后面的参数不看），当场退 0。"""
import sys

sys.exit(0 if sys.argv[1:2] == ["--selftest"] else 2)
'''
FAKE_SYSTEMCTL = r'''#!/usr/bin/env bash
# task-server 自证给 control 用的假 systemctl：调用记进 systemctl-calls；is-active 照 systemctl-state 打状态；stop 把它写成 inactive（systemctl-stop-leaves-active 在时不改）；
# start 把它写成 active、退 0（systemctl-start-fails 在时不改、退 1）。服务不由它起：control-start 那一格自己起
printf '%s\n' "$*" >> "$TASK_SERVER_SELFTEST_WORLD/systemctl-calls"
[[ "${1:-}" == --user ]] && shift
state_file="$TASK_SERVER_SELFTEST_WORLD/systemctl-state"
case "${1:-}" in
  start)
    if [[ -e "$TASK_SERVER_SELFTEST_WORLD/systemctl-start-fails" ]]; then echo "假 systemctl：start 起不来（自证）" >&2; exit 1; fi
    echo active > "$state_file"
    exit 0 ;;
  is-active)
    state="$(cat "$state_file" 2>/dev/null || echo inactive)"
    echo "$state"
    [[ "$state" == active ]] && exit 0
    exit 3 ;;
  show) echo ""; exit 0 ;;
  stop)
    [[ -e "$TASK_SERVER_SELFTEST_WORLD/systemctl-stop-leaves-active" ]] || echo inactive > "$state_file"
    exit 0 ;;
  *) exit 0 ;;
esac
'''

# 错误类型 → 喂到它的那一格（type-table-covered 静态核这张表与 task_server_lib.OUTCOME_TYPES 逐个对上）
EXERCISED_BY = {
    "succeeded": "error-types", "command-failed": "error-types", "memory-cap-hit": "error-types", "memory-cap-at-maximum": "memory-cap-at-maximum",
    "squeezed-out": "error-types", "gpu-memory-short": "error-types", "timed-out": "error-types",
    "wrapper-refused": "error-types", "run-condition-unmet": "error-types", "not-run": "error-types", "peer-run-failed": "error-types",
    "card-fault": "error-types", "launcher-lost": "error-types", "cancelled": "cancel-own-session-only", "record-tampered": "tampered-row-not-started",
    "never-fits": "error-types", "unit-start-failed": "unit-start-failed", "shard-plan-refused": "shard-retry-isolated",
    "stalled": "stalled-attempt-stopped", "resubmit-required": "queued-revalidated-on-restart",
    "prune-server-unavailable": "prune-server-unavailable-outcome", "over-two-hours": "over-two-hours-bounced",
}
PEAKS = lib.load_module("memory_peaks_lib", os.path.join(lib.HOSTS_DIRECTORY, "memory_peaks_lib.py"))
HEAVY_THREADS = lib.load_module("heavy_threads", os.path.join(lib.HOSTS_DIRECTORY, "heavy_threads.py"))
SERVER = lib.load_module("task_server_constants_for_selftest", lib.SERVER_SCRIPT)   # 只读它的常量（AGED_WAIT_SECONDS）、Store 类（task-locks-released）与 Scheduler.push_outcomes（inbox-outcomes-resent-after-error，套在假调度器上），不在这个进程里起服务


class SelftestFailure(Exception):
    def __init__(self, detail, extra_lines=()):
        super().__init__(detail)
        self.extra_lines = list(extra_lines)


def write_executable(path, text):
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    os.chmod(path, 0o755)


def memory_report(free_gibibytes, slice_gibibytes, gpu_cards=None, slice_used_gibibytes=0, margin_gibibytes=SELFTEST_MARGIN_GIBIBYTES, scopes=None):
    """slice_used_gibibytes：slice 里此刻进程用的内存（memory_peaks_lib 的 headroom 交的 slice_used），slice 放得下的照总上限减它。
    margin_gibibytes：这台此刻的余量（memory_peaks_lib 的 MARGIN_REPORT_KEY，自证喂的定值）；None 演量不到（原因写进 MARGIN_PROBLEM_KEY）。
    scopes：{尝试的 label: 回收不掉的用量（字节）}，写成这台 slice 下各个在跑的 scope（memory_peaks_lib 的 SCOPES_REPORT_KEY，单元名是假的）；不给就不写这一项。"""
    free = int(free_gibibytes * GIBIBYTE)
    slice_total = int(slice_gibibytes * GIBIBYTE)
    slice_used = int(slice_used_gibibytes * GIBIBYTE)
    margin = None if margin_gibibytes is None else int(margin_gibibytes * GIBIBYTE)
    report = {"memory": {"mem_total": 512 * GIBIBYTE, "mem_available": free, "pending": 0, "running": 0, "slice_total": slice_total, "slice_used": slice_used,
                         "free": free, "slice_room": slice_total - slice_used, PEAKS.MARGIN_REPORT_KEY: margin, PEAKS.MARGIN_MEASURED_KEY: margin,
                         PEAKS.MARGIN_PROBLEM_KEY: "" if margin is not None else "自证：这台 slice 的 memory.current 读不了"}}
    if scopes is not None:
        report["memory"][PEAKS.SCOPES_REPORT_KEY] = {f"singlefs-memory-cap-selftest-{position}.scope": {PEAKS.SCOPE_USED_KEY: int(used), PEAKS.SCOPE_LABEL_KEY: label}
                                                     for position, (label, used) in enumerate(sorted(scopes.items()))}
    if gpu_cards is not None:
        report["gpu"] = {"cards": [{"index": index, "total": mebibytes, "free": mebibytes} for index, mebibytes in gpu_cards]}
    return report


class Patience:
    """一次等待的耐心（World.keep_waiting 的包装）：patience = world.patience(秒数)，循环写 while patience.waiting():，出来之后 patience.elapsed() 是等了多久。
    循环里自己有进展信号的（例如服务进程的 CPU 在涨）把它交给 waiting(extra=…)：这个值变了算有进展。"""

    def __init__(self, world, limit):
        self.world = world
        self.limit = limit
        self.started = time.monotonic()
        self.extra = None
        self.extra_moved_at = 0.0

    def waiting(self, extra=None):
        if extra is not None and extra != self.extra:
            self.extra = extra
            self.extra_moved_at = time.monotonic()
        return self.world.keep_waiting(self.started, self.limit, self.extra_moved_at)

    def elapsed(self):
        return time.monotonic() - self.started


class World:
    """一格的假世界：临时目录里的多机配置（判法要的键写全，另写 TASK_SERVER_ 那几个键）、密钥、假包装、两台的假余量与一份真的 task-server。"""

    def __init__(self, cell, root, prepare=True):
        self.cell = cell
        self.root = root
        self.bin = os.path.join(root, "bin")
        self.world = os.path.join(root, "world")
        self.work = os.path.join(root, "work")
        self.home = os.path.join(root, "home")
        self.fake_cargo = os.path.join(root, "fake-cargo")
        # 格里不另给假 scope 时尝试读的那一份：空目录，峰值量不到、回收不掉的用量算 0（task_server_lib.py 文件头「自证」）
        self.empty_scope = os.path.join(root, "empty-scope")
        # 这一格的假 NFS 共享根（多机配置的 NFS_SHARED_ROOTS）就是这一格的临时目录：格里交的命令点到的路径都在它下面，提交时不因共享根外的路径被拒
        # （task_server.py 文件头 ②「提交时拒」）；假 etab 导出它、假 ssh 在本机读写它（task_server_lib.py 文件头「NFS 共享」）
        self.shared_root = root
        # 共享根外的两处放在这一格临时目录的旁边（整份自证的临时目录里，收尾一起删）：假第二台本地盘上的编译目录，与 *-refused 那几格点名的共享根外路径
        self.peer_local = root.rstrip(os.sep) + ".peer-local"
        self.outside_root = root.rstrip(os.sep) + ".outside"
        # 假仓根放在假共享根下（仓也在 NFS 共享根下，第二台在同一棵树里跑：task_server_lib.py 文件头「挑机器」）
        self.repo = os.path.join(self.shared_root, "repo")
        # 这一格服务的工作根与共享数据根（SINGLEFS_WORK_ROOT、SINGLEFS_DATA_ROOT）：放在假共享根下，提交时的核对（task_server.py 的 shared_roots_problem）过得去
        self.work_root = os.path.join(self.shared_root, "work-root")
        self.data_root = os.path.join(self.shared_root, "data-root")
        self.exports = os.path.join(root, "fake-etab")
        self.sessions = os.path.join(root, "sessions")   # 假的会话登记目录（服务照会话号找收件 socket 读它；FakeInbox 往里登）
        for directory in (self.bin, self.world, self.work, self.repo, self.home, self.fake_cargo, self.empty_scope, self.shared_root, self.work_root, self.data_root,
                          self.sessions):
            os.makedirs(directory, exist_ok=True)
        self.write_exports([self.shared_root])
        # 假仓根里的登记表 .claude/gate.d/verdict-library-scripts.tsv：只有表头（带判定库的一行都没有）；读不了的照真的那样判不了、提交时拒，
        # verdict-library-family-refused 那一格自己写行
        registry = os.path.join(self.repo, SERVER.VERDICT_LIBRARY_REGISTRY)
        os.makedirs(os.path.dirname(registry), exist_ok=True)
        with open(registry, "w", encoding="utf-8") as handle:
            handle.write("# 自证的登记表：路径<TAB>自证命令<TAB>判定库放哪<TAB>复用三问格名\n")
        self.state = os.path.join(root, "state")
        # socket 目录放在短的公共目录下（task_server_lib.py 文件头「自证」）：格的临时目录套几层之后接上 /run/server.sock 会超 socket 路径的上限
        self.short_root = tempfile.mkdtemp(prefix=WORLD_SOCKET_PREFIX, dir=lib.selftest_override(SOCKET_ROOT_VARIABLE) or root)
        self.runtime = os.path.join(self.short_root, "run")
        self.key = os.path.join(root, "config", "signing.key")
        self.builds = os.path.join(root, "builds")
        self.config = os.path.join(root, "multi-host.env")
        self.write_config(self.config, self.state, self.runtime, self.key, self.builds, self.shared_root)
        if prepare:
            lib.create_signing_key(self.key)
        fakes = {"fake-wrapper": FAKE_MEMORY_WRAPPER, "fake-peer-runner": FAKE_PEER_RUNNER,
                 "fake-quiesce": FAKE_GPU_QUIESCE, "fake-systemd-run-refusing": FAKE_SYSTEMD_RUN_REFUSING, "fake-systemd-run-selective": FAKE_SYSTEMD_RUN_SELECTIVE,
                 "fake-systemctl": FAKE_SYSTEMCTL,
                 "fake-nvidia-smi": FAKE_NVIDIA_SMI, "fake-ssh": FAKE_SSH, "fake-systemd-run-delayed": FAKE_SYSTEMD_RUN_DELAYED,
                 "fake-systemd-run-silent": FAKE_SYSTEMD_RUN_SILENT, "fake-prune-check": FAKE_PRUNE_CHECK}
        for name, text in fakes.items():
            write_executable(os.path.join(self.bin, name), text)
        write_executable(os.path.join(self.fake_cargo, "cargo"), FAKE_CARGO)
        write_executable(os.path.join(self.repo, "fake-sharded.py"), FAKE_SHARDED)
        os.makedirs(os.path.join(self.repo, "scripts", "prune-server"), exist_ok=True)
        write_executable(os.path.join(self.repo, "scripts", "prune-server", lib.TEMPORARY_SERVICE_SCRIPT), FAKE_TEMPORARY_SERVICE)
        self.resources_path = os.path.join(self.world, "resources.json")
        self.resources(local=(30, 30))
        mark = os.environ.get(lib.SELFTEST_MARK_VARIABLE, "")
        common = {lib.MULTI_HOST_CONFIG_VARIABLE: self.config, lib.SELFTEST_MARK_VARIABLE: mark, "TASK_SERVER_SELFTEST_FAST": "1",
                  "TASK_SERVER_SELFTEST_WORLD": self.world, "TASK_SERVER_SELFTEST_ALLOW_TMPFS": "1",
                  "TASK_SERVER_NFS_EXPORTS": self.exports, "TASK_SERVER_PEER_SSH": os.path.join(self.bin, "fake-ssh")}
        if os.environ.get(lib.BREAK_VARIABLE):
            common[lib.BREAK_VARIABLE] = os.environ[lib.BREAK_VARIABLE]
        base = {name: os.environ[name] for name in ("PATH", "HOME", "USER", "LOGNAME", "LANG", "XDG_RUNTIME_DIR", "DBUS_SESSION_BUS_ADDRESS") if name in os.environ}
        self.server_environment = dict(base, **common, **{
            "TASK_SERVER_FAKE_RESOURCES": self.resources_path, "TASK_SERVER_MEMORY_WRAPPER": os.path.join(self.bin, "fake-wrapper"),
            "TASK_SERVER_PEER_RUNNER": os.path.join(self.bin, "fake-peer-runner"), "TASK_SERVER_GPU_QUIESCE": os.path.join(self.bin, "fake-quiesce"),
            "TASK_SERVER_SESSION_BOUNDARY_PID": str(os.getpid()), "TASK_SERVER_FAKE_REPOSITORY_ROOT": self.repo, "TASK_SERVER_FAKE_SCOPE_DIRECTORY": self.empty_scope,
            lib.WORK_ROOT_VARIABLE: self.work_root,   # 共享数据根不设 SINGLEFS_DATA_ROOT：服务照多机配置的 SHARED_DATA_ROOT 取（同真服务的单元环境）
            lib.MEMORY_FLOOR_VARIABLE: "0",   # 弄坏开关 recorded-cap-floored 下兜到的下限，小世界里是 0：开关只红点名的几格（task_server_lib.py 文件头「只在自证里认的变量」）；那几格写空串照真的
            "RUN_WITH_MEMORY_CAP_PEAKS": self.peak_table, "HOST_DISPATCH_STATE_DIR": os.path.join(root, "host-dispatch"),
            "GPU_QUIESCE_STATE_DIRECTORY": os.path.join(root, "quiesce"), lib.SESSIONS_DIRECTORY_OVERRIDE: self.sessions})
        # 客户端与 control 的 XDG 目录换到这一格的临时目录：多机配置不在、照缺省路径接着判的那一种（弄坏开关 config-missing-ignored）也连不到本机在跑的那一份服务。
        # 客户端读不到收件地址就拒（task 的 submit）：缺省给一个不在的收件 socket 与一个令牌；会话登记目录里没登这一格的会话，服务照会话号找不到，提醒不发、结局发不出（写告警），
        # wait 照取；要收消息的格起 FakeInbox，它把会话登进会话登记目录
        self.client_environment = dict(common, PATH=os.environ.get("PATH", "/usr/bin:/bin"), HOME=self.home,
                                       **{name: os.path.join(root, "xdg", name.lower()) for name in ("XDG_RUNTIME_DIR", "XDG_STATE_HOME", "XDG_CONFIG_HOME", "XDG_DATA_HOME")},
                                       **{lib.INBOX_SOCKET_VARIABLE: os.path.join(self.short_root, "inbox-unregistered.sock"), lib.INBOX_TOKEN_VARIABLE: DEFAULT_INBOX_TOKEN})
        self.server = None
        self.server_logs = 0
        self.background = []
        self.inboxes = []   # 这一格起的假收件 socket（FakeInbox），close 关掉
        self.seeded = []   # 服务起来之前要记进它库里的尝试（seed_peak、seed_duration），起来之后写进去
        self.last_progress = time.monotonic()   # 世界上一次有进展的时刻（keep_waiting 采到记号变了就更新）
        self.progress_sampled_at = 0.0
        self.progress_marked = None
        self.database_digest = ()   # 任务与尝试的状态摘要；库一时读不了（服务正写）就沿用上一回的，不把读不到当成有变化

    @property
    def peak_table(self):
        return os.path.join(self.root, "memory-peaks.tsv")

    # ── 文件 ──
    @staticmethod
    def write_config(path, state, runtime, key, build, shared_root, data_root=True, slice_caps=None):
        """整份多机配置：共用取法 scripts/hosts/multi_host_addresses.py 整份判过才给第二台的地址，判法要的键写全（SELFTEST_MULTI_HOST_KEYS），另写 task-server 读的
        TASK_SERVER_ 那几个键。shared_root 给空串就不写 NFS_SHARED_ROOTS 与 SHARED_DATA_ROOT（nfs-share-required 那一格）；data_root=False 不写 SHARED_DATA_ROOT
        （data-root-key-required）；slice_caps 是 SLICE_MEMORY_CAP_BY_HOST 那一行的值（不给是两台都 SELFTEST_SLICE_CAP_TEXT）。
        数据根是共享根下的 data-root（同 World.data_root）。注释单起一行：.claude/hooks/lib_work_root.py 与多机配置判法读值都不去行尾注释。"""
        hosts = f"{lib.HOSTS_KEY}={SELFTEST_LOCAL_ADDRESS} {SELFTEST_PEER_ADDRESS}\n"
        shared = ""
        if shared_root:
            shared = f"# 自证：假共享根与它下面的共享数据根\n{lib.NFS_ROOTS_KEY}={shared_root}\n"
            if data_root:
                shared += f"{lib.DATA_ROOT_KEY}={os.path.join(shared_root, 'data-root')}\n"
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(f"# task-server 自证这一格的多机配置：判法要的键写全，另写 task-server 读的那几个键\n{hosts}"
                         + SELFTEST_MULTI_HOST_KEYS.format(slice_caps=slice_caps or by_host_text(SELFTEST_SLICE_CAP_TEXT, SELFTEST_SLICE_CAP_TEXT)) + shared
                         + f"TASK_SERVER_STATE_DIRECTORY={state}\nTASK_SERVER_RUNTIME_DIRECTORY={runtime}\n"
                         f"TASK_SERVER_SIGNING_KEY_FILE={key}\nTASK_SERVER_BUILD_DIRECTORY={build}\n")

    def write_exports(self, roots):
        """这一格的假 /var/lib/nfs/etab（TASK_SERVER_NFS_EXPORTS）：每个根一行，第一列是根，写法照真的那样「<目录><TAB><主机>(<选项>)」。"""
        temporary = self.exports + ".partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            handle.writelines(f"{root}\tselftest-peer(rw,sync,wdelay,hide,no_subtree_check)\n" for root in roots)
        os.replace(temporary, self.exports)

    def set_slice_cap(self, text):
        """这一格的多机配置里 SLICE_MEMORY_CAP_BY_HOST 那一行换成两台都写 <text>（钉值现读它）。"""
        self.set_slice_caps(by_host_text(text, text))

    def set_slice_caps(self, value):
        """这一格的多机配置里 SLICE_MEMORY_CAP_BY_HOST 那一行的整值换成 <value>（钉值现读它；判法要这个键、只许一行；少一台、多一项的那几格照原样写）。
        先写临时文件再改名（服务现读，读不到写了一半的）。"""
        temporary = self.config + ".partial"
        self.write_config(temporary, self.state, self.runtime, self.key, self.builds, self.shared_root, slice_caps=value)
        os.replace(temporary, self.config)

    def resources(self, local=None, peer=None, peer_cap_gibibytes=None, cards="", local_gpu=None, hold=False, path=None, recheck_local=None, local_slice_used=0,
                  peer_down="", peer_disk_bytes=None, peer_df=None, local_disk_bytes=None, peer_gpu=None,
                  local_margin=SELFTEST_MARGIN_GIBIBYTES, peer_margin=SELFTEST_MARGIN_GIBIBYTES, peer_slice_used=0, peer_scopes=None, local_scopes=None, local_cpu=None, local_slice_unstarted=False):
        """写两台的假余量与卡（服务每次现读）；path 给了就写到那里（假第二台跑完时换上的那一份），不给是这一格服务读的那一份。
        recheck_local 给了，起之前复核读本机时照它（排队那一轮照 local）；local_slice_used 是本机 slice 里回收不掉的用量（GiB）。
        peer_down 给了（一句原因），演第二台此刻用不了（照真的那样告警）。
        peer_disk_bytes、local_disk_bytes 给了，那一台的报告里带盘余量（task_server.py 的 DISK_REPORT_KEY）；peer_df 给了，演服务在第二台另起的 df 打出的那几行；都不给的服务当盘够。
        local_margin、peer_margin：两台此刻的余量（GiB，缺省 SELFTEST_MARGIN_GIBIBYTES；None 演那台量不到余量）。
        peer_slice_used：第二台 slice 里此刻进程用的内存（GiB）；peer_scopes、local_scopes：那一台报告里在跑的 scope（{尝试的 label: 回收不掉的用量字节}，memory_report 的 scopes）。"""
        value = {"cards": cards, "hold": hold, "peer_build_root": self.peer_local}   # 第二台的编译根：假 multi-host-run.sh 在它下面 builds/<标签>/ 建这一趟的编译目录，服务照它记（Resources.build_root_of_peer）
        if peer_df is not None:
            value["df"] = {SERVER.PEER: peer_df}
        if peer_down:
            value["down"] = {SERVER.PEER: peer_down}
        if local:
            value["local"] = memory_report(local[0], local[1], local_gpu, local_slice_used, local_margin, local_scopes)
            if local_slice_unstarted:   # 演本机 slice 还没起：slice_total 是估的（memory_peaks_lib 的 SLICE_STARTED_KEY 为假）
                value["local"]["memory"][PEAKS.SLICE_STARTED_KEY] = False
            if local_cpu is not None:   # 本机报告里的 CPU 负载（memory_peaks_lib 的 CPU_REPORT_KEY；不给的服务不判 CPU 闸）
                value["local"]["memory"][PEAKS.CPU_REPORT_KEY] = dict(local_cpu)
            if local_disk_bytes is not None:   # 本机编译目录所在盘的余量（task_server.py 的 DISK_REPORT_KEY）；不给的服务不判本机占盘
                value["local"][SERVER.DISK_REPORT_KEY] = int(local_disk_bytes)
        if recheck_local:
            value["recheck"] = {"local": memory_report(recheck_local[0], recheck_local[1], margin_gibibytes=local_margin)}
        if peer:
            value["peer"] = memory_report(peer[0], peer[1], peer_gpu, peer_slice_used, peer_margin, peer_scopes)
            if peer_disk_bytes is not None:
                value["peer"][SERVER.DISK_REPORT_KEY] = int(peer_disk_bytes)
            value["memory_caps"] = {SERVER.PEER: int((peer_cap_gibibytes or peer[1]) * GIBIBYTE)}
        target = path or self.resources_path
        temporary = target + ".partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            json.dump(value, handle)
        os.replace(temporary, target)

    def seed_peak(self, key, byte_count):
        """往这一格服务自己的库里记一次跑完（succeeded）的尝试、量到 byte_count 的峰值：服务定额度只认自己库里的尝试（task_server_lib.py 文件头「资源记录」）。
        key 是命令键（RUN_WITH_MEMORY_CAP_KEY 的值，或 PEAKS.command_key 换算过的）。"""
        self.seed_attempt({"command_key": key, "outcome": "succeeded", "memory_peak": int(byte_count)})

    def seed_duration(self, key, seconds, command):
        """往这一格服务自己的库里记一次跑完的尝试、用时 seconds 秒（峰值量不到，不占定额度的名额）：预计用时只认自己库里命令逐词相同（command）的尝试。"""
        self.seed_attempt({"command_key": key, "command": list(command), "outcome": "succeeded", "memory_peak": None, "seconds": float(seconds)})

    def seed_attempt(self, fields, metric=lib.PROCESS_MEMORY_PEAK_METRIC):
        """记一次结束了的尝试；metric 是峰值口径一栏（task_server_lib.py 文件头「资源记录」）：缺省照这一版服务记的写，给 None 演批 92 之前记的、没有这一栏的旧行。"""
        record = {"peak_metric": metric} if metric is not None else {}
        record.update(fields)
        self.seeded.append(dict(record, task_id=0, number=1, host="local", state="ended", session="", ended_at=SEEDED_ENDED_AT,
                                started_at=SEEDED_ENDED_AT - float(fields.get("seconds") or 1)))
        self.flush_seeded()

    def flush_seeded(self):
        """服务在跑、库已建好时把记下的尝试写进库（用这一格的签名密钥签）；服务还没起的等 start_server 之后再写。"""
        if self.server is None or self.server.poll() is not None or not os.path.exists(os.path.join(self.state, lib.DATABASE_NAME)):
            return
        while self.seeded:
            self.insert_signed_row("attempts", self.seeded.pop(0))

    def write_fake_scope(self, peak_bytes):
        """尝试的假 scope（attempt_runner.py 文件头「自证换 scope」）：进程内存（memory.stat 的 anon）与 memory.peak 都写成 peak_bytes（没有页缓存），
        各先写临时文件再改名（采样读不到写了一半的）；交目录。"""
        scope = os.path.join(self.root, "fake-scope")
        os.makedirs(scope, exist_ok=True)
        for name, text in (("memory.stat", process_memory_stat(peak_bytes)), ("memory.peak", f"{peak_bytes}\n")):
            temporary = os.path.join(scope, name + ".partial")
            with open(temporary, "w", encoding="utf-8") as handle:
                handle.write(text)
            os.replace(temporary, os.path.join(scope, name))
        return scope


    def write_spec(self, name, value):
        with open(os.path.join(self.repo, name), "w", encoding="utf-8") as handle:
            json.dump(value, handle)
        return ["python3", "fake-sharded.py", "run", name]

    def sharded_calls(self, kind, **fields):
        """调用记录里假分片入口记的那几行（kind 是 plan、shard、merge、whole），按栏过滤，交 [{栏: 值}]。"""
        rows = []
        for line in self.calls():
            if not line.startswith(f"sharded-{kind} "):
                continue
            row = dict(item.split("=", 1) for item in line.split()[1:] if "=" in item)
            if all(row.get(name) == str(value) for name, value in fields.items()):
                rows.append(row)
        return rows

    def calls(self):
        try:
            with open(os.path.join(self.world, "calls"), encoding="utf-8") as handle:
                return handle.read().splitlines()
        except FileNotFoundError:
            return []

    def calls_of(self, task_id, kind="wrapper"):
        return [line for line in self.calls() if line.startswith(f"{kind} task={task_id} ")]

    def scheduler_reads(self):
        """服务记的排队读数（task_server.py 的 selftest_read）：每行「pass view=…」「read view=… source=… what=…」「predict task=…」「recheck task=…」。"""
        try:
            with open(os.path.join(self.world, "scheduler-reads"), encoding="utf-8") as handle:
                return handle.read().splitlines()
        except FileNotFoundError:
            return []

    def full_passes(self, queued_count, passes):
        """等到排队线程连着排了 passes + 1 轮、每轮都看到 queued_count 条在排；交前 passes 轮的编号（第 passes + 1 轮开了头，前面那几轮的读数都已记完）。"""
        patience = self.patience(STATE_TIMEOUT_SECONDS)
        numbers = []
        while patience.waiting():
            numbers = [int(found.group(1)) for found in (re.match(r"^pass view=(\d+) queued=(\d+)$", line) for line in self.scheduler_reads())
                       if found and int(found.group(2)) == queued_count]
            if len(numbers) > passes:
                return numbers[:passes]
            time.sleep(POLL_SECONDS)
        raise SelftestFailure(f"等了 {STATE_TIMEOUT_SECONDS:.0f} 秒，排队线程只排了 {len(numbers)} 轮看到 {queued_count} 条在排（要 {passes + 1} 轮）", self.server_log_lines())

    def next_pass(self):
        """等排队线程开下一轮：此刻在排的那一轮的读数都已记完。"""
        started = len([line for line in self.scheduler_reads() if line.startswith("pass ")])
        patience = self.patience(STATE_TIMEOUT_SECONDS)
        while patience.waiting():
            if len([line for line in self.scheduler_reads() if line.startswith("pass ")]) > started:
                return
            time.sleep(POLL_SECONDS)
        raise SelftestFailure(f"等了 {STATE_TIMEOUT_SECONDS:.0f} 秒，排队线程没再开一轮", self.server_log_lines())

    def alerts(self):
        try:
            with open(os.path.join(self.state, "alerts.jsonl"), encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except FileNotFoundError:
            return []

    def operations(self):
        try:
            with open(os.path.join(self.state, "operations.jsonl"), encoding="utf-8") as handle:
                return [json.loads(line) for line in handle if line.strip()]
        except FileNotFoundError:
            return []

    # ── 等待（上面常量处「等待的判法」）──
    def progress_mark(self):
        """这一格的世界此刻的进展记号：假世界目录里每份文件的大小（假脚本的调用记录等；每一轮排队都涨的 PROGRESS_IGNORED_FILES 不算：那是服务活着，不是有事发生）、
        尝试的日志与说明、告警与操作日志、各次服务的输出，加任务与尝试的状态摘要。只含有事发生才变的东西，服务空转它不变。"""
        files = []
        for directory in (self.world, os.path.join(self.state, "logs"), os.path.join(self.state, "attempts")):
            for current, _, names in os.walk(directory):
                files.extend(os.path.join(current, name) for name in names if name not in PROGRESS_IGNORED_FILES)
        files.extend(os.path.join(self.state, name) for name in ("alerts.jsonl", "operations.jsonl"))
        files.extend(os.path.join(self.root, f"server-{number}.log") for number in range(1, self.server_logs + 1))
        sizes = []
        for path in sorted(files):
            with contextlib.suppress(OSError):
                sizes.append((path, os.stat(path).st_size))
        database = os.path.join(self.state, lib.DATABASE_NAME)
        if os.path.exists(database):
            with contextlib.suppress(sqlite3.Error):
                with contextlib.closing(sqlite3.connect(f"file:{database}?mode=ro", uri=True, timeout=1)) as connection:
                    self.database_digest = tuple(connection.execute(
                        "SELECT 'task', id, state FROM tasks UNION ALL SELECT 'attempt', id, state FROM attempts").fetchall())
        return tuple(sizes), self.database_digest

    def keep_waiting(self, started, limit, also_moved=0.0):
        """还要不要接着等（各格的等待都问这一处，不逐格写时限）：从 started 起，世界（progress_mark）上一次有进展、或 also_moved 那个时刻之后安静不满 limit 秒，
        且总共没到 limit × WAIT_CEILING_FACTOR 秒。弄坏开关 wait-limit-fixed 下退回写死的时限：从 started 起满 limit 秒就判，不看进展、没有上限的说法（改前的样子）。"""
        now = time.monotonic()
        if lib.broken("wait-limit-fixed"):
            return now - started < limit
        if now - self.progress_sampled_at >= PROGRESS_SAMPLE_SECONDS:
            self.progress_sampled_at = now
            marked = self.progress_mark()
            if marked != self.progress_marked:
                self.progress_marked = marked
                self.last_progress = now
        return now - max(started, self.last_progress, also_moved) < limit and now - started < limit * WAIT_CEILING_FACTOR

    def patience(self, limit):
        return Patience(self, limit)

    def communicate(self, process, limit):
        """等 process 退出、收它的输出（process.communicate 的耐心版，limit 是安静时限）；等不到照 subprocess.TimeoutExpired 抛，调用处原有的 except 照用。"""
        started = time.monotonic()
        while True:
            try:
                return process.communicate(timeout=WAIT_SLICE_SECONDS)
            except subprocess.TimeoutExpired:
                if not self.keep_waiting(started, limit):
                    raise

    def wait_process(self, process, limit):
        """等 process 退出、交退出码（process.wait 的耐心版）；等不到照 subprocess.TimeoutExpired 抛。"""
        started = time.monotonic()
        while True:
            try:
                return process.wait(timeout=WAIT_SLICE_SECONDS)
            except subprocess.TimeoutExpired:
                if not self.keep_waiting(started, limit):
                    raise

    def server_cpu_ticks(self):
        """这一格起的服务进程自己用过的 CPU 时钟数（/proc/<进程号>/stat 的 utime 加 stime）；读不到交 None。服务起来的路上它在涨就是还在忙。"""
        if self.server is None:
            return None
        try:
            with open(f"/proc/{self.server.pid}/stat", encoding="utf-8") as handle:
                text = handle.read()
        except OSError:
            return None
        fields = text[text.rindex(")") + 2:].split()
        return int(fields[11]) + int(fields[12])

    # ── 服务 ──
    def server_log_lines(self):
        lines = []
        for number in range(1, self.server_logs + 1):
            with contextlib.suppress(OSError):
                with open(os.path.join(self.root, f"server-{number}.log"), encoding="utf-8", errors="replace") as handle:
                    lines += handle.read().splitlines()
        return lines[-20:]

    def start_server(self, extra=None, runtime=None):
        """起服务、等它的 /health 回 ok；runtime 是这一回服务的 socket 目录（给了别的多机配置的那一格各自传它的，不给是这一格的）。"""
        self.server_logs += 1
        log = open(os.path.join(self.root, f"server-{self.server_logs}.log"), "ab")
        environment = dict(self.server_environment, **(extra or {}))
        self.server = subprocess.Popen([sys.executable, "-B", lib.SERVER_SCRIPT, "serve"], env=environment, stdin=subprocess.DEVNULL, stdout=log, stderr=log,
                                       start_new_session=True)
        log.close()
        socket_path = os.path.join(runtime or self.runtime, lib.SOCKET_NAME)
        patience = self.patience(STATE_TIMEOUT_SECONDS)
        while patience.waiting(extra=self.server_cpu_ticks()):   # 起来的路上服务进程的 CPU 在涨就是还在忙（机器忙时导入与建库慢）
            if self.server.poll() is not None:
                raise SelftestFailure(f"task-server 起来就退了（退 {self.server.returncode}）", self.server_log_lines())
            with contextlib.suppress(lib.ServerUnreachable):
                status, health = lib.call_server(socket_path, "GET", "/health", timeout=5)
                if status == 200 and health.get("ok"):
                    self.flush_seeded()
                    return
            time.sleep(POLL_SECONDS)
        raise SelftestFailure(f"task-server 起了 {patience.elapsed():.0f} 秒（安静 {STATE_TIMEOUT_SECONDS:.0f} 秒没有进展）/health 还不回 ok", self.server_log_lines())

    def stop_server(self):
        if self.server is None or self.server.poll() is not None:
            return
        self.server.send_signal(signal.SIGTERM)   # 只对自己起的那一个进程号发
        try:
            self.server.wait(timeout=30)
        except subprocess.TimeoutExpired:
            self.server.kill()
            self.server.wait(timeout=10)

    def serve_until_exit(self, config, timeout=STATE_TIMEOUT_SECONDS):
        """拿 config 这一份多机配置起一份服务、等它自己退出（起不来的那几种）：→ (退出码, 它的输出)；它起来了（这一格的 socket 上 /health 回话）
        当场停掉（只对自己起的那一个进程号发）、交 (None, 它的输出)。"""
        self.server_logs += 1
        log_path = os.path.join(self.root, f"server-{self.server_logs}.log")
        with open(log_path, "ab") as log:
            server = subprocess.Popen([sys.executable, "-B", lib.SERVER_SCRIPT, "serve"], env=dict(self.server_environment, **{lib.MULTI_HOST_CONFIG_VARIABLE: config}),
                                      stdin=subprocess.DEVNULL, stdout=log, stderr=log, start_new_session=True)
        code = None
        patience = self.patience(timeout)
        while patience.waiting():
            code = server.poll()
            if code is not None:
                break
            with contextlib.suppress(lib.ServerUnreachable):
                status, _ = lib.call_server(os.path.join(self.runtime, lib.SOCKET_NAME), "GET", "/health", timeout=5)
                if status == 200:
                    break
            time.sleep(POLL_SECONDS)
        if code is None:
            server.send_signal(signal.SIGTERM)
            try:
                server.wait(timeout=30)
            except subprocess.TimeoutExpired:
                server.kill()
                server.wait(timeout=10)
        with open(log_path, encoding="utf-8", errors="replace") as handle:
            return code, handle.read()

    def wait_server_exit(self, timeout=30):
        try:
            return self.wait_process(self.server, timeout)
        except subprocess.TimeoutExpired:
            raise SelftestFailure(f"task-server {timeout} 秒没有照自证造的崩溃退出", self.server_log_lines()) from None

    def api(self, method, path, body=None):
        status, reply = lib.call_server(os.path.join(self.runtime, lib.SOCKET_NAME), method, path, body, timeout=30)
        return status, reply

    def summary(self, task_id):
        status, reply = self.api("GET", f"/tasks/{task_id}")
        if status != 200:
            raise SelftestFailure(f"读任务 {task_id} 回 HTTP {status}：{reply}")
        return reply

    def wait_for(self, task_id, predicate, what, timeout=STATE_TIMEOUT_SECONDS):
        """等任务 task_id 的摘要让 predicate 为真；timeout 是安静时限（上面常量处「等待的判法」）：世界还在变就接着等，安静满 timeout 秒或到绝对上限才判红。"""
        patience = self.patience(timeout)
        summary = None
        while patience.waiting():
            with contextlib.suppress(lib.ServerUnreachable):
                summary = self.summary(task_id)
                if predicate(summary):
                    return summary
            time.sleep(POLL_SECONDS)
        raise SelftestFailure(f"任务 {task_id} 等了 {patience.elapsed():.0f} 秒没{what}（安静时限 {timeout:.0f} 秒），"
                              f"最后一次读到：{json.dumps(summary, ensure_ascii=False)[:600] if summary else '读不到'}", self.server_log_lines())

    def finished(self, task_id, timeout=STATE_TIMEOUT_SECONDS):
        return self.wait_for(task_id, lambda summary: summary["task"].get("state") == "finished", "收尾", timeout)

    def running(self, task_id, count=1):
        return self.wait_for(task_id, lambda summary: len([attempt for attempt in summary["attempts"] if attempt.get("state") == "running"]) >= count
                             and summary["task"].get("state") == "running", "跑起来")

    # ── 客户端与 control ──
    def client(self, arguments, session=SESSION_A, extra=None, forged=None, timeout=90, background=False, script=None, cwd=None):
        """经一个「会话 shell」起客户端（或 control）：那个 bash 的环境里是 session，客户端自己的环境里换成 forged（给了的话）。"""
        environment = dict(self.client_environment, **(extra or {}))
        environment.pop(lib.SESSION_VARIABLE, None)
        if session:
            environment[lib.SESSION_VARIABLE] = session
        inner = '"$@"; exit $?' if forged is None else 'CLAUDE_CODE_SESSION_ID="$0" "$@"; exit $?'
        argv = ["bash", "-c", inner, forged or "session-shell", sys.executable, "-B", script or lib.CLIENT_SCRIPT, *arguments]
        if background:
            process = subprocess.Popen(argv, env=environment, cwd=cwd or self.work, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                       text=True)
            self.background.append(process)
            return process
        process = subprocess.Popen(argv, env=environment, cwd=cwd or self.work, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
        try:
            stdout, stderr = self.communicate(process, timeout)   # timeout 是安静时限（上面常量处「等待的判法」）
        except subprocess.TimeoutExpired:
            process.kill()   # 只停自己起的那一个进程号
            stdout, stderr = process.communicate()
            raise SelftestFailure(f"{' '.join(arguments)[:200]} 跑了 {timeout} 秒没退（安静时限；世界一直在变的话，绝对上限 {timeout * WAIT_CEILING_FACTOR} 秒）", [str(stdout or ""), str(stderr or "")]) from None
        return subprocess.CompletedProcess(argv, process.returncode, stdout, stderr)

    def control(self, arguments, **options):
        return self.client(arguments, script=lib.CONTROL_SCRIPT, **options)

    def submit(self, command, session=SESSION_A, extra=None, forged=None, cwd=None):
        """交一条、照确认单一起确认（confirm_submitted；task_server.py 文件头 ⑦），交任务号。"""
        completed = self.client(["submit", "--cwd", cwd or self.work, "--", *command], session=session, extra=extra, forged=forged)
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if completed.returncode != 0 or not found:
            raise SelftestFailure(f"提交 {command} 没拿到任务号（退 {completed.returncode}）", [completed.stdout, completed.stderr] + self.server_log_lines())
        self.confirm_submitted(completed, session=session, extra=extra, forged=forged)
        return int(found.group(1))

    def confirm_submitted(self, completed, session=SESSION_A, extra=None, forged=None, required=True):
        """照提交回复（或确认被拒时）打的 CONFIRM_NEEDED_MARK 那一行确认（自证世界的帮手：自证造的任务照样要确认，不另设跳过确认的开关）：
        数对不上的（这一会儿同会话别的任务起了或收了）照服务打回的那一行再确认，至多 CONFIRM_RETRIES 回；回复里没有那一行的（弄坏开关 confirm-skipped 下交了就进队）不确认。
        → 最后一次 confirm 的 CompletedProcess（不用确认交 None）；required 为真时确认不成判红。"""
        reply = completed
        for _ in range(CONFIRM_RETRIES):
            needed = lib.confirm_needed_fields(reply.stdout)
            if needed is None:
                return None if reply is completed else reply
            task_id, items, tasks = needed
            reply = self.client(["confirm", str(task_id), "--items", str(items), "--tasks", str(tasks)], session=session, extra=extra, forged=forged)
            if reply.returncode == 0:
                return reply
        if required:
            raise SelftestFailure(f"照确认单确认了 {CONFIRM_RETRIES} 回都没成（最后退 {reply.returncode}）", [completed.stdout, reply.stdout, reply.stderr] + self.server_log_lines())
        return reply

    def register_session(self, session, socket_path, name, pid=None, started_milliseconds=None):
        """往假的会话登记目录写一份 <name>.json（同 Claude Code 每个进程写的那一份：pid、sessionId、messagingSocketPath、startedAt）；pid 缺省是这一格的进程（活着）。"""
        record = {"pid": pid or os.getpid(), "sessionId": session, "messagingSocketPath": socket_path,
                  "startedAt": started_milliseconds if started_milliseconds is not None else int(time.time() * 1000)}
        temporary = os.path.join(self.sessions, name + ".json.partial")
        with open(temporary, "w", encoding="utf-8") as handle:
            json.dump(record, handle)
        os.replace(temporary, os.path.join(self.sessions, name + ".json"))

    def unit_active(self, unit):
        if not unit:
            return False
        completed = subprocess.run(["systemctl", "--user", "show", "-p", "ActiveState", "--value", unit], capture_output=True, text=True, stdin=subprocess.DEVNULL,
                                   timeout=30)
        return completed.stdout.strip() in lib.RUNNING_UNIT_STATES

    def database_units(self):
        path = os.path.join(self.state, lib.DATABASE_NAME)
        if not os.path.exists(path):
            return []
        with contextlib.closing(sqlite3.connect(path, timeout=30)) as connection:
            rows = connection.execute("SELECT record FROM attempts").fetchall()
        units = []
        for (text,) in rows:
            with contextlib.suppress(ValueError, AttributeError):
                if json.loads(text).get("unit"):
                    units.append(json.loads(text)["unit"])
        return units

    def rewrite_signed_task(self, task_id, change):
        """服务停着时用这一格的签名密钥改写库里一条任务（演「旧规则下交进来、还在排」）：change 就地改 record。"""
        self.rewrite_signed_row("tasks", task_id, change)

    def rewrite_signed_row(self, table, row_id, change):
        """用这一格的签名密钥改写库里一行（tasks、attempts、history）：change 就地改 record；record 里的 state、session、command_key 照写进几列、重签。"""
        key = lib.read_signing_key(self.key)
        with contextlib.closing(sqlite3.connect(os.path.join(self.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
            connection.execute("BEGIN IMMEDIATE")
            record = json.loads(connection.execute(f"SELECT record FROM {table} WHERE id = ?", (row_id,)).fetchone()[0])
            change(record)
            self.write_signed(connection, key, table, row_id, record)
            connection.execute("COMMIT")

    def insert_signed_row(self, table, record):
        """用这一格的签名密钥往库里写一行（演「改键之前记下的行」、同一次号再插一行）；交它的行号。"""
        key = lib.read_signing_key(self.key)
        with contextlib.closing(sqlite3.connect(os.path.join(self.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
            connection.execute("BEGIN IMMEDIATE")
            cursor = connection.execute(f"INSERT INTO {table}(task_id, state, session, command_key, record, signature) VALUES(?, '', '', '', '{{}}', '')",
                                        (record.get("task_id", 0),))
            self.write_signed(connection, key, table, cursor.lastrowid, dict(record, id=cursor.lastrowid))
            connection.execute("COMMIT")
        return cursor.lastrowid

    @staticmethod
    def write_signed(connection, key, table, row_id, record):
        text = lib.canonical_json(record)
        state, session, command_key = record.get("state", ""), record.get("session", ""), record.get("command_key", "")
        connection.execute(f"UPDATE {table} SET state = ?, session = ?, command_key = ?, record = ?, signature = ? WHERE id = ?",
                           (state, session, command_key, text, lib.row_signature(key, table, row_id, state, session, command_key, text), row_id))

    def signed_rows(self, table):
        """[(record, 校验过没有)]：照服务读回的规则核（校验列、正文里的号与几列对得上）。"""
        key = lib.read_signing_key(self.key)
        with contextlib.closing(sqlite3.connect(os.path.join(self.state, lib.DATABASE_NAME), timeout=30)) as connection:
            rows = connection.execute(f"SELECT id, state, session, command_key, record, signature FROM {table} ORDER BY id").fetchall()
        checked = []
        for row_id, state, session, command_key, text, signature in rows:
            record = json.loads(text)
            valid = (signature == lib.row_signature(key, table, row_id, state, session, command_key, text) and record.get("id") == row_id
                     and record.get("state", "") == state and record.get("session", "") == session and record.get("command_key", "") == command_key)
            checked.append((record, valid))
        return checked

    def insert_bypass_row(self, record):
        """绕过服务直接往库里写一行任务（校验列是乱填的）；交它的行号。"""
        with contextlib.closing(sqlite3.connect(os.path.join(self.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
            connection.execute("BEGIN IMMEDIATE")
            cursor = connection.execute("INSERT INTO tasks(task_id, state, session, command_key, record, signature) VALUES(0, 'queued', ?, 'selftest-bypass', '{}', '')",
                                        (record["session"],))
            record["id"] = cursor.lastrowid
            connection.execute("UPDATE tasks SET record = ?, signature = ? WHERE id = ?", (lib.canonical_json(record), "0" * 64, record["id"]))
            connection.execute("COMMIT")
        return record["id"]

    def close(self):
        for inbox in self.inboxes:
            inbox.close()
        for process in self.background:
            if process.poll() is None:
                process.kill()   # 只停自己起的那一个进程号
                with contextlib.suppress(subprocess.TimeoutExpired):
                    process.wait(timeout=10)
        self.stop_server()
        with contextlib.suppress(OSError, sqlite3.Error, subprocess.SubprocessError):
            for unit in self.database_units():
                if self.unit_active(unit):
                    subprocess.run(["systemctl", "--user", "stop", unit], capture_output=True, stdin=subprocess.DEVNULL, timeout=200)
        shutil.rmtree(self.short_root, ignore_errors=True)
        shutil.rmtree(self.root, ignore_errors=True)


def task_command_ok(summary, outcome, attempts=None):
    task = summary["task"]
    return task.get("outcome") == outcome and (attempts is None or len(summary["attempts"]) == attempts)


def describe(summary):
    task = summary["task"]
    return (f"结局 {task.get('outcome')}、退 {task.get('exit_code')}、尝试 "
            + "、".join(f"{attempt.get('host')} {attempt.get('memory_cap_text')} {attempt.get('outcome')}" for attempt in summary["attempts"]))


# ══ 各格 ══
def cell_check_missing(world):
    absent = os.path.join(world.root, "absent-multi-host.env")
    completed = world.control(["check"], extra={lib.MULTI_HOST_CONFIG_VARIABLE: absent})
    if completed.returncode != 78 or "multi-host.env.example" not in completed.stderr or "cp multi-host.env.example multi-host.env" not in completed.stderr:
        raise SelftestFailure(f"多机配置不在时 control check 应当退 78、出路写 cp multi-host.env.example multi-host.env，实际退 {completed.returncode}", [completed.stdout, completed.stderr])
    completed = world.control(["check"])
    if completed.returncode != 78 or "不通" not in completed.stderr or "control start" not in completed.stderr:
        raise SelftestFailure(f"服务没起时 control check 应当退 78、报不通、出路写 control start，实际退 {completed.returncode}", [completed.stderr])
    world.start_server()
    leftover = [path for path in (os.path.join(world.root, "task-server.env"), os.path.join(lib.REPOSITORY_ROOT, "task-server.env")) if os.path.exists(path)]
    completed = world.control(["check"])
    if completed.returncode != 0:
        raise SelftestFailure(f"多机配置在、服务起了之后 control check 应当退 0（task-server.env 不在也照常，此刻在的：{leftover or '无'}），实际退 {completed.returncode}",
                              [completed.stdout, completed.stderr])
    completed = world.control(["check"], extra={"TASK_SERVER_SELFTEST_MISSING_MODULES": "fastapi"})
    if completed.returncode != 78 or "sudo apt install python3-fastapi python3-uvicorn" not in completed.stderr:
        raise SelftestFailure(f"fastapi 不在时 control check 应当退 78、出路写 sudo apt install python3-fastapi python3-uvicorn，实际退 {completed.returncode}",
                              [completed.stderr])
    return "多机配置不在、服务没起、fastapi 不在各退 78 且出路写全，多机配置在、服务起了退 0（不读 task-server.env）"


def cell_install(world):
    state = os.path.join(world.root, "install-state")
    runtime = os.path.join(world.short_root, LONGEST_RUNTIME_NAME)
    key = os.path.join(world.root, "install-config", "signing.key")
    build = os.path.join(world.root, "install-builds")
    config = os.path.join(world.root, "install-multi-host.env")
    World.write_config(config, state, runtime, key, build, world.shared_root)
    extra = {lib.MULTI_HOST_CONFIG_VARIABLE: config, "TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl"),
             "XDG_CONFIG_HOME": os.path.join(world.root, "xdg-config")}
    unit = os.path.join(world.root, "xdg-config", "systemd", "user", lib.UNIT_NAME)
    calls = os.path.join(world.world, "systemctl-calls")
    completed = world.control(["install", "--dry-run"], extra=extra)
    written = [path for path in (key, state, runtime, build, unit, calls) if os.path.exists(path)]
    if (completed.returncode != 0 or "ExecStart=" not in completed.stdout or f"Environment={lib.MULTI_HOST_CONFIG_VARIABLE}={config}" not in completed.stdout
            or written):
        raise SelftestFailure(f"install --dry-run 应当退 0、打出单元全文（钉住这一份多机配置）、一个文件不写、systemctl 不起，实际退 {completed.returncode}、写了 {written}",
                              [completed.stdout[-600:], completed.stderr])
    completed = world.control(["install", "--dry-run"], extra=dict(extra, **{lib.MULTI_HOST_CONFIG_VARIABLE: os.path.join(world.root, "absent-multi-host.env")}))
    if completed.returncode != 78 or "cp multi-host.env.example multi-host.env" not in completed.stderr:
        raise SelftestFailure(f"多机配置不在时 install 应当退 78、不替用户拷模板，实际退 {completed.returncode}", [completed.stderr])
    completed = world.control(["install"], extra=extra)
    modes = {path: oct(os.stat(path).st_mode & 0o777) for path in (key, os.path.dirname(key), state, runtime) if os.path.exists(path)}
    calls_text = ""
    with contextlib.suppress(OSError):
        with open(calls, encoding="utf-8") as handle:
            calls_text = handle.read()
    if (completed.returncode != 1 or "没回话" not in completed.stderr or modes.get(key) != "0o600" or modes.get(os.path.dirname(key)) != "0o700"
            or modes.get(state) != "0o700" or not os.path.isfile(unit) or "daemon-reload" not in calls_text or "--user start" not in calls_text or "enable --now" in calls_text):
        raise SelftestFailure(f"服务没起来时 install 应当写好密钥（600，目录 700）、状态目录（700）与单元、跑 daemon-reload 与 start（不 enable）、回读不通退 1，"
                              f"实际退 {completed.returncode}、权限 {modes}、单元{'在' if os.path.isfile(unit) else '不在'}、systemctl 收到「{calls_text.strip()}」",
                              [completed.stderr])
    with open(key, "rb") as handle:
        digest = hashlib.sha256(handle.read()).hexdigest()
    world.start_server(extra={lib.MULTI_HOST_CONFIG_VARIABLE: config}, runtime=runtime)
    completed = world.control(["install"], extra=extra)
    with open(key, "rb") as handle:
        digest_after = hashlib.sha256(handle.read()).hexdigest()
    if completed.returncode != 0 or digest_after != digest:
        raise SelftestFailure(f"服务起着、密钥已在时再 install 应当退 0、密钥不动，实际退 {completed.returncode}、密钥{'没动' if digest_after == digest else '被重写了'}",
                              [completed.stdout[-400:], completed.stderr])
    return "--dry-run 只打不写；install 写密钥 600、目录 700、单元，回读不通退 1；服务起着再装退 0、密钥不重写"


def cell_control_readback(world):
    extra = {"TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl")}
    with open(os.path.join(world.world, "systemctl-state"), "w", encoding="utf-8") as handle:
        handle.write("active\n")
    open(os.path.join(world.world, "systemctl-stop-leaves-active"), "w").close()
    completed = world.control(["stop"], extra=extra)
    if completed.returncode != 1 or "回读不对" not in completed.stderr:
        raise SelftestFailure(f"systemctl stop 退 0 而 is-active 仍打 active 时 control stop 应当退 1、报回读不对，实际退 {completed.returncode}", [completed.stderr])
    os.remove(os.path.join(world.world, "systemctl-stop-leaves-active"))
    completed = world.control(["stop"], extra=extra)
    if completed.returncode != 0:
        raise SelftestFailure(f"停下了（is-active 打 inactive、cgroup 里没有进程）时 control stop 应当退 0，实际退 {completed.returncode}", [completed.stderr])
    return "停了而回读仍在跑退 1，回读停了退 0"


def cell_server_unreachable(world):
    marker = os.path.join(world.root, "ran-directly")
    completed = world.client(["submit", "--", "bash", "-c", ': > "$1"', "touch-marker", marker])
    time.sleep(0.5)
    if completed.returncode != 78 or "不通" not in completed.stderr or "control start" not in completed.stderr or os.path.exists(marker):
        raise SelftestFailure(f"服务不通时 submit 应当退 78、出路写 control start、命令一行不跑，实际退 {completed.returncode}、命令{'跑了' if os.path.exists(marker) else '没跑'}",
                              [completed.stdout, completed.stderr])
    completed = world.client(["wait", "1"])
    if completed.returncode != 78:
        raise SelftestFailure(f"服务一直不通时 wait 应当重连一阵之后退 78，实际退 {completed.returncode}", [completed.stderr])
    return "submit 退 78、命令没跑；wait 重连不上退 78"


def cell_exit_code_passthrough(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "echo selftest-output; exit 7"])
    completed = world.client(["wait", str(task_id)])
    last = completed.stdout.strip().splitlines()[-1] if completed.stdout.strip() else ""
    found = re.match(rf"{lib.RESULT_MARK} task={task_id} type=command-failed exit=7 attempts=1 log=(\S+)$", last)
    log = found.group(1) if found else ""
    content = ""
    with contextlib.suppress(OSError):
        with open(log, encoding="utf-8") as handle:
            content = handle.read()
    if completed.returncode != 7 or not found or not lib.inside(log, world.state) or "selftest-output" not in content:
        raise SelftestFailure(f"命令退 7：wait 应当退 7、末行 type=command-failed exit=7 attempts=1、日志在状态目录里、有命令的输出，实际退 {completed.returncode}、末行「{last}」",
                              [completed.stderr])
    calls = world.calls_of(task_id)
    if not calls or f"admitted={task_id}.1" not in calls[0]:
        raise SelftestFailure(f"本机的尝试应当给内存包装设 {lib.ADMITTED_VARIABLE}={task_id}.1，实际调用「{calls}」")
    return f"wait 退 7、末行对、日志在状态目录 {os.path.basename(log)}、内存包装收到已准入的开关"


def server_socket_count(world):
    """服务进程此刻开着的 socket 数（/proc/<进程号>/fd 里指到 socket: 的）：监听的、自管道的，加每个还连着的客户端一个；读不到交 -1。"""
    directory = f"/proc/{world.server.pid}/fd"
    count = 0
    try:
        names = os.listdir(directory)
    except OSError:
        return -1
    for name in names:
        with contextlib.suppress(OSError):
            if os.readlink(os.path.join(directory, name)).startswith("socket:"):
                count += 1
    return count


def cell_restart_keeps_running(world):
    world.start_server()
    release = os.path.join(world.root, "release")
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release])
    summary = world.running(task_id)
    unit = summary["attempts"][0].get("unit", "")
    samples = []
    for _ in range(3):   # 没有客户端连着时服务开着几个 socket：隔一会儿采三次取最小，躲开刚断开还没收掉的那一个
        time.sleep(0.1)
        samples.append(server_socket_count(world))
    idle_sockets = min(samples)
    # wait 照平常的定时：缩短的定时下断线满 2 秒（task 的 FAST_STARTUP_RETRY_SECONDS）就退 78，停服务、隔 0.5 秒、再起到 /health 回话在机器忙时比它长
    # （任务 4519、4521 与 4586–4595 里的 78 都是「task-server 不通 … server.sock：FileNotFoundError」），这一格判的是重启时 wait 接着重连，不是缩短的那个时限
    waiter = world.client(["wait", str(task_id)], background=True, extra={"TASK_SERVER_SELFTEST_FAST": "0"})
    # 等 wait 真连上服务（服务多开了一个 socket）再停服务：机器忙时客户端起得慢，固定睡 1 秒停服务，它可能到服务重启之后才连上，没碰上断线，
    # 弄坏开关 wait-gives-up-on-disconnect（断线就退）就弄不红这一格
    patience = world.patience(SOCKET_CONNECT_SECONDS)
    while waiter.poll() is None and server_socket_count(world) <= idle_sockets and patience.waiting():
        time.sleep(POLL_SECONDS)
    time.sleep(0.5)   # 连上之后再给它一点时间发出长等的请求
    world.stop_server()
    # 服务停着时等 wait 真碰上断线：照平常它重连时打一句「task-server 连不上」（弄坏开关下它就此退 78）才再起服务。停服务时那一问可能回 HTTP 500、
    # wait 隔 RECONNECT_SECONDS 再问，服务要是在那之前已重新起来，它一次断线都没碰上，开关弄不红这一格（任务 7290）
    noticed = b""
    descriptor = waiter.stderr.fileno()
    os.set_blocking(descriptor, False)
    patience = world.patience(DISCONNECT_NOTICE_SECONDS)
    while waiter.poll() is None and UNREACHABLE_NOTICE.encode("utf-8") not in noticed and patience.waiting():
        with contextlib.suppress(BlockingIOError):
            noticed += os.read(descriptor, 65536)
        time.sleep(POLL_SECONDS)
    os.set_blocking(descriptor, True)
    if not world.unit_active(unit):
        raise SelftestFailure(f"服务停下之后在跑的那一次尝试的单元应当照跑，实际单元「{unit or '（没有单元）'}」不在跑")
    if waiter.poll() is not None:
        raise SelftestFailure(f"服务停下时 wait 应当接着重连、不退，实际退了 {waiter.returncode}",
                              [waiter.stdout.read(), noticed.decode("utf-8", "replace") + waiter.stderr.read()])
    if UNREACHABLE_NOTICE.encode("utf-8") not in noticed:
        raise SelftestFailure(f"服务停着时 wait 应当重连、打「{UNREACHABLE_NOTICE}」，安静 {DISCONNECT_NOTICE_SECONDS:.0f} 秒没打（它没碰上断线，这一格判不出断线就退）",
                              [noticed.decode("utf-8", "replace")])
    world.start_server()
    open(release, "w").close()
    try:
        stdout, stderr = world.communicate(waiter, STATE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        raise SelftestFailure("服务重启、任务跑完之后 wait 60 秒没退") from None
    if waiter.returncode != 0 or f"{lib.RESULT_MARK} task={task_id} type=succeeded" not in stdout:
        raise SelftestFailure(f"服务重启之后 wait 应当拿到真结局 succeeded、退 0，实际退 {waiter.returncode}", [stdout, stderr])
    return "服务停了单元照跑、wait 重连，重启之后拿到 succeeded"


def cell_start_once(world):
    for point in ("before-unit", "after-unit"):
        world.stop_server()
        arm = os.path.join(world.root, f"arm-{point}")
        counter = os.path.join(world.root, f"count-{point}")
        world.resources(local=(30, 30), hold=True)
        world.start_server(extra={"TASK_SERVER_SELFTEST_CRASH_POINT": f"{point}:{arm}"})
        task_id = world.submit(["bash", "-c", 'printf x >> "$1"', "count", counter])
        open(arm, "w").close()
        world.resources(local=(30, 30))
        code = world.wait_server_exit()
        if code != 70:
            raise SelftestFailure(f"自证造的崩溃（{point}）应当让服务退 70，实际退 {code}", world.server_log_lines())
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while point == "after-unit" and patience.waiting() and (not os.path.exists(counter) or any(world.unit_active(unit) for unit in world.database_units())):
            time.sleep(POLL_SECONDS)
        world.start_server()
        summary = world.finished(task_id)
        with open(counter, encoding="utf-8") as handle:
            count = handle.read()
        if summary["task"].get("outcome") != "succeeded" or count != "x" or len(summary["attempts"]) != 1:
            raise SelftestFailure(f"服务在{'写了单元名、起单元之前' if point == 'before-unit' else '起了单元之后、标记起了之前'}没了：重启之后应当只起这一次（命令跑 1 次、尝试 1 次、succeeded），"
                                  f"实际命令跑了 {len(count)} 次、{describe(summary)}", world.server_log_lines())
    return "写了单元名、起单元之前与起了之后各崩一次：重启之后命令各只跑 1 次"


def cell_queue_holds_unplaceable(world):
    world.resources(local=(11, 30))   # 空 11G：两条没记录的各记 4G 之后剩 3G，比第三条要的 4G（4G 加余量 2G、不超过上限 4G）小
    world.start_server()
    releases = [os.path.join(world.root, f"release-{number}") for number in (1, 2)]
    first = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", releases[0]])
    world.running(first)
    second = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", releases[1]])
    world.running(second)
    third = world.submit(["bash", "-c", "exit 0"])
    world.next_pass()   # 看交了之后排过一整轮的样子，不比秒数：睡 1.5 秒看不出这期间排没排到第三条，没排到也照「排着」放过
    world.next_pass()
    summary = world.summary(third)
    if summary["task"].get("state") != "queued" or summary["attempts"] or world.calls_of(third):
        raise SelftestFailure(f"本机空 11G，两条没记录的各记 4G 之后只剩 3G，第三条（没记录，要的空闲 = 4G 加余量 2G、不超过上限 4G）此刻放不下：应当留在库里排着、没有它的包装进程，"
                              f"实际状态 {summary['task'].get('state')}、尝试 {len(summary['attempts'])} 次、包装调用 {world.calls_of(third)}")
    open(releases[0], "w").close()
    summary = world.finished(third)
    open(releases[1], "w").close()
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"放出来之后第三条应当起、succeeded，实际 {describe(summary)}")
    return "放不下的那一条留在库里、没起包装，放出来之后才起"


def cell_backfill_when_head_blocked(world):
    """本机 30G（可用上限 28G、界 23.8G）：头一条（峰值 8G，记 12G）占着，排头（峰值 20G，记 28G）放不下、没满时限，后面两条没记录的（各记 4G）先起、跑完。
    头一条与后面两条都是碎片、一组装 20G 不到界：判的只是回填。头一条的记账离整片的界远：压在界上的话界随余量一挪它就从整片变成碎片，交替封组会把回填挡住。"""
    world.resources(local=(30, 30))
    for key, peak in (("selftest-backfill-first", 8 * GIBIBYTE), ("selftest-backfill-head", 20 * GIBIBYTE)):
        world.seed_peak(key, peak)
    world.start_server()
    release = os.path.join(world.root, "release")
    first = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-backfill-first"})
    world.running(first)
    head = world.submit(["bash", "-c", "exit 0"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-backfill-head"})
    later = [world.submit(["bash", "-c", "exit 0"]) for _ in range(2)]   # 没有记录、也没有用时记录的两条
    summaries = [world.finished(task_id) for task_id in later]
    head_state = world.summary(head)["task"].get("state")
    open(release, "w").close()
    if any(summary["task"].get("outcome") != "succeeded" for summary in summaries) or head_state != "queued":
        raise SelftestFailure(f"本机空 30G、头一条记 12G 在跑：排头（记 28G）此刻放不下、没满时限，后面放得下的两条（没有用时记录）应当先起、跑完，排头照排；"
                              f"实际后面两条 {[describe(summary) for summary in summaries]}、排头 {head_state}", world.server_log_lines())
    head_summary = world.finished(head)
    if head_summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"头一条放出来之后排头应当起、跑完，实际 {describe(head_summary)}", world.server_log_lines())
    return "排头放不下、没满时限时，后面放得下的（有没有用时记录都一样）先起跑完；排头空出来之后照起"


def cell_arrival_order(world):
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    task_ids = [world.submit(["bash", "-c", "exit 0"]) for _ in range(3)]
    world.resources(local=(30, 30))
    # 起的先后看服务写尝试记录的次序（记录号）：同一轮里先后起的几条，各自单元里的假包装往调用记录写行的先后不定，不拿它判
    records = [world.finished(task_id)["attempts"] for task_id in task_ids]
    started = [attempts[0].get("id") if attempts else None for attempts in records]
    if None in started or started != sorted(started) or len(set(started)) != len(started):
        raise SelftestFailure(f"都放得下时应当按到达次序 {task_ids} 起，实际各条第一次尝试的记录号 {started}")
    return "都放得下时按到达次序起"


def cell_prediction_computed_once(world):
    world.resources(local=(3, 30))   # 空 3G：没有记录的命令记 4G、要的空闲 = 4G 加余量 2G、不超过上限 4G，哪一条都放不下
    world.start_server()
    task_ids = [world.submit(["bash", "-c", "exit 0"]) for _ in range(PASS_QUEUED_COUNT)]
    world.full_passes(PASS_QUEUED_COUNT, PASS_COUNT)
    lines = world.scheduler_reads()
    counts = {task_id: lines.count(f"predict task={task_id}") for task_id in task_ids}
    states = {task_id: world.summary(task_id)["task"].get("state") for task_id in task_ids}
    if any(count != 1 for count in counts.values()) or any(state != "queued" for state in states.values()):
        raise SelftestFailure(f"本机空 3G、{PASS_QUEUED_COUNT} 条没记录的都放不下，排了 {PASS_COUNT} 轮以上：每条的预测应当只算一次（predict 共 {PASS_QUEUED_COUNT} 次），"
                              f"各条照排；实际各条算了 {counts} 次、状态 {states}", world.server_log_lines())
    return f"{PASS_QUEUED_COUNT} 条放不下的排了 {PASS_COUNT} 轮以上，预测共算 {sum(counts.values())} 次（每条一次）"


def cell_pass_reads_resources_once(world):
    world.resources(local=(3, 30))   # 同上：哪一条都放不下，每一轮都把 PASS_QUEUED_COUNT 条判一遍
    world.start_server()
    for _ in range(PASS_QUEUED_COUNT):
        world.submit(["bash", "-c", "exit 0"])
    numbers = world.full_passes(PASS_QUEUED_COUNT, PASS_COUNT)
    lines = world.scheduler_reads()
    expected = ["what=pending host=local", "what=snapshot gpu=0"]   # 只派得了本机、不要卡：一轮读一次本机的余量、一次本机的待涨量
    reads = {number: sorted(line.split(" ", 3)[3] for line in lines if line.startswith(f"read view={number} source=pass ")) for number in numbers}
    wrong = {number: got for number, got in reads.items() if got != expected}
    if wrong:
        raise SelftestFailure(f"{PASS_QUEUED_COUNT} 条在排的一轮里，余量与待涨量应当各读一次（{expected}），不随条数涨；实际这几轮读的 {wrong}", world.server_log_lines())
    return f"{PASS_QUEUED_COUNT} 条在排的 {PASS_COUNT} 轮里，每轮本机的余量与待涨量各读一次"


def cell_pass_booking_carried(world):
    world.resources(local=(7, 30), hold=True)   # 空 7G：没记录的记 4G、要的空闲 4G（4G 加余量 2G、不超过上限 4G）；起了一条之后剩 3G，第二条放不下
    world.start_server()
    release = os.path.join(world.root, "release")
    first = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release])
    second = world.submit(["bash", "-c", "exit 0"])
    world.resources(local=(7, 30))   # 两条都在排时放开：同一轮里判这两条
    world.running(first)
    world.next_pass()
    summary = world.summary(second)
    rechecks = [line for line in world.scheduler_reads() if line.startswith(f"recheck task={second} ")]
    open(release, "w").close()
    if summary["task"].get("state") != "queued" or summary["attempts"] or rechecks or RECHECK_FAILED_TEXT in (summary.get("waiting_for") or ""):
        raise SelftestFailure(f"本机空 7G 只放得下一条没记录的：同一轮里起了头一条之后，它的 4G 记账应当加进这一轮本机的待涨量（剩 3G 比要的 4G 小），第二条在判的时候就放不下、"
                              f"不走到起之前复核；实际第二条状态 {summary['task'].get('state')}、尝试 {len(summary['attempts'])} 次、复核 {rechecks}、"
                              f"排队原因「{summary.get('waiting_for')}」", world.server_log_lines())
    summaries = [world.finished(task_id) for task_id in (first, second)]
    if any(item["task"].get("outcome") != "succeeded" for item in summaries):
        raise SelftestFailure(f"头一条放出来之后两条都应当 succeeded，实际 {[describe(item) for item in summaries]}", world.server_log_lines())
    return "一台只放得下一条时同一轮只起一条：起了的记账带到这一轮后面，第二条在判的时候就放不下；头一条跑完第二条照起"


def cell_launch_rechecked(world):
    world.resources(local=(30, 30), recheck_local=(3, 30))   # 排队那一轮空 30G 放得下；起之前现读空 3G，没记录的（要的空闲 4G：4G 加余量 2G、不超过上限 4G）放不下
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"])
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    summary = world.summary(task_id)
    while patience.waiting() and not summary["attempts"]:
        if any(line.startswith(f"recheck task={task_id} ") and line.endswith(" fits=0") for line in world.scheduler_reads()):
            world.next_pass()   # 复核没过的那一轮排完：排队原因已换上
            summary = world.summary(task_id)
            break
        time.sleep(POLL_SECONDS)
        summary = world.summary(task_id)
    waiting = summary.get("waiting_for") or ""
    if summary["task"].get("state") != "queued" or summary["attempts"] or world.calls_of(task_id) or RECHECK_FAILED_TEXT not in waiting:
        raise SelftestFailure(f"判的时候本机空 30G 放得下、起之前现读只剩 3G 放不下：这一条应当不起、留在队里，排队原因写「{RECHECK_FAILED_TEXT}」；"
                              f"实际状态 {summary['task'].get('state')}、尝试 {len(summary['attempts'])} 次、包装调用 {world.calls_of(task_id)}、排队原因「{waiting}」",
                              world.server_log_lines())
    world.resources(local=(30, 30))
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"起之前现读也放得下之后这一条应当起、succeeded，实际 {describe(summary)}", world.server_log_lines())
    return f"判的时候放得下、起之前现读放不下的不起，排队原因写「{RECHECK_FAILED_TEXT}」；现读也放得下之后照起"


def cell_error_types(world):
    cards = {"cards": LOCAL_CARD_ZERO, "local_gpu": [(0, 16384)]}
    world.resources(local=(100, 100), **cards)   # 头一段没有第二台：本机的那几种都在本机起（派得到第二台的照余量挑台，不再因工作目录只在本机）
    world.seed_peak("selftest-peer-only", 120 * GIBIBYTE)
    world.seed_peak("selftest-never-fits", 150 * GIBIBYTE)
    world.start_server()
    plain = ["bash", "-c", "exit 0"]
    cases = [("succeeded", plain, {}, None, None), ("command-failed", ["bash", "-c", "exit 3"], {}, None, 3),
             ("command-failed", ["bash", "-c", "exit 199"], {}, None, 199),
             ("memory-cap-hit", plain + ["fake-wrapper-exits=250,run"], {}, None, None), ("squeezed-out", plain + ["fake-wrapper-exits=254,run"], {}, None, None),
             ("gpu-memory-short", plain + ["fake-wrapper-exits=245,run"], {}, None, None),
             ("timed-out", plain + ["fake-wrapper-exits=253"], {}, None, None), ("wrapper-refused", plain + ["fake-wrapper-exits=251"], {}, None, None),
             ("run-condition-unmet", ["bash", "-c", "exit 78"], {}, None, None), ("not-run", ["bash", "-c", "exit 77"], {}, None, None),
             ("card-fault", ["cargo", "test", "--features", "gpu", "fake-quiesce=card-fault"], {}, None, None),
             ("launcher-lost", ["bash", "-c", "kill -9 $PPID"], {}, None, None),
             ("never-fits", plain, {"RUN_WITH_MEMORY_CAP_KEY": "selftest-never-fits"}, None, None)]
    submitted = [(expected, world.submit(command, extra=extra, cwd=cwd), exit_code) for expected, command, extra, cwd, exit_code in cases]
    summaries = {task_id: world.finished(task_id) for _, task_id, _ in submitted}
    # 后一段有第二台：本机永远放不下（120G 记 180G）、第二台放得下的那一条派第二台，假第二台退 255 记 peer-run-failed
    world.resources(local=(100, 100), peer=(200, 200), peer_cap_gibibytes=190, **cards)
    peer_case = ("peer-run-failed", plain + ["fake-peer-exit=255"], {"RUN_WITH_MEMORY_CAP_KEY": "selftest-peer-only"}, world.repo, None)
    cases.append(peer_case)
    peer_id = world.submit(peer_case[1], extra=peer_case[2], cwd=peer_case[3])
    summaries[peer_id] = world.finished(peer_id)
    submitted.append((peer_case[0], peer_id, peer_case[4]))
    wrong = []
    for expected, task_id, exit_code in submitted:
        summary = summaries[task_id]
        attempts = summary["attempts"]
        seen = attempts[0].get("outcome") if attempts else summary["task"].get("outcome")
        if expected in ("never-fits",):
            seen = summary["task"].get("outcome") if not attempts else f"起了 {len(attempts)} 次"
        if seen != expected or (exit_code is not None and (attempts[0].get("exit_code") != exit_code or summary["task"].get("exit_code") != exit_code)):
            wrong.append(f"任务 {task_id} 应当是 {expected}{'、退 ' + str(exit_code) if exit_code is not None else ''}，实际第一次尝试 {seen}、{describe(summary)}")
        if expected == "peer-run-failed" and attempts and attempts[0].get("host") != "peer":
            wrong.append(f"任务 {task_id} 应当派到第二台，实际 {attempts[0].get('host')}")
    card_fault_task = submitted[[expected for expected, _, _ in submitted].index("card-fault")][1]
    if not any(alert.get("kind") == "card-fault" and alert.get("task") == card_fault_task for alert in world.alerts()):
        wrong.append(f"卡坏的任务 {card_fault_task} 应当写一条 card-fault 告警，实际告警 {world.alerts()}")
    if wrong:
        raise SelftestFailure("错误类型对不上：" + "；".join(wrong), world.server_log_lines())
    return f"{len(cases)} 个假命令或假包装记下的类型都与表对得上（表外的 199 记 command-failed、卡坏写了告警）"


def cell_unit_start_failed(world):
    world.start_server(extra={"TASK_SERVER_SYSTEMD_RUN": os.path.join(world.bin, "fake-systemd-run-refusing")})
    marker = os.path.join(world.root, "ran")
    task_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", marker])
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "unit-start-failed" or summary["task"].get("exit_code") != 235 or os.path.exists(marker):
        raise SelftestFailure(f"systemd-run 起不来时应当记 unit-start-failed、退 235、命令没跑，实际 {describe(summary)}、命令{'跑了' if os.path.exists(marker) else '没跑'}")
    return "systemd-run 起不来记 unit-start-failed、命令没跑"


def cell_memory_cap_growth(world):
    world.seed_peak("selftest-growth", GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,run"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-growth"})
    summary = world.finished(task_id)
    caps = [attempt.get("memory_cap_text") for attempt in summary["attempts"]]
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    if caps != ["1536M", "2304M", "3456M"] or outcomes != ["memory-cap-hit", "memory-cap-hit", "succeeded"] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"连撞两次上限：上限应当 1536M、2304M（1.5 倍）、3456M，三次尝试都在记录里、第三次 succeeded，实际上限 {caps}、类型 {outcomes}、{describe(summary)}")
    return "上限 1536M → 2304M → 3456M，三次尝试都留着，第三次跑完"


def cell_unrecorded_default_small(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"])
    summary = world.finished(task_id)
    attempt = summary["attempts"][0] if summary["attempts"] else {}
    calls = world.calls_of(task_id)
    pinned_bytes = int(UNRECORDED_DEFAULT_PINNED_TEXT[:-1]) * MEBIBYTE
    if (summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != UNRECORDED_DEFAULT_PINNED_TEXT or attempt.get("memory_cap") != pinned_bytes
            or attempt.get("booked") != pinned_bytes or attempt.get("prediction") != lib.UNRECORDED_HISTORY
            or not calls or f" cap={UNRECORDED_DEFAULT_PINNED_TEXT} " not in calls[0]):
        raise SelftestFailure(f"服务的记录里没有的命令：第一次尝试的上限与记账都应当是 {UNRECORDED_DEFAULT_PINNED_TEXT}（{pinned_bytes} 字节）、"
                              f"prediction 记 {lib.UNRECORDED_HISTORY}、内存包装收到 cap={UNRECORDED_DEFAULT_PINNED_TEXT}，实际上限 {attempt.get('memory_cap_text')}"
                              f"（{attempt.get('memory_cap')} 字节）、记账 {attempt.get('booked')} 字节、prediction {attempt.get('prediction')}、{describe(summary)}、"
                              f"包装调用 {calls}", world.server_log_lines())
    return f"没记录的命令第一次上限与记账都是 {UNRECORDED_DEFAULT_PINNED_TEXT}，内存包装收到的也是它"


def cell_unrecorded_cap_growth(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,run"])
    summary = world.finished(task_id)
    caps = [attempt.get("memory_cap_text") for attempt in summary["attempts"]]
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    numbers = [attempt.get("number") for attempt in summary["attempts"]]
    wrapper_caps = [line.split(" cap=", 1)[1].split(" ", 1)[0] for line in world.calls_of(task_id) if " cap=" in line]
    expected_caps = list(UNRECORDED_GROWTH_PINNED_TEXTS)
    if (caps != expected_caps or wrapper_caps != expected_caps or outcomes != ["memory-cap-hit", "memory-cap-hit", "succeeded"] or numbers != [1, 2, 3]
            or summary["task"].get("outcome") != "succeeded"):
        raise SelftestFailure(f"没记录的命令假包装连退两次 250：上限应当 {' → '.join(expected_caps)}（小缺省起，每次乘 1.5）、三次尝试都在记录里（第 1、2、3 次）、第三次 succeeded，"
                              f"实际上限 {caps}、包装收到 {wrapper_caps}、类型 {outcomes}、次序 {numbers}、{describe(summary)}", world.server_log_lines())
    return f"没记录的命令撞了再放大：上限 {' → '.join(expected_caps)}，三次尝试都留着，第三次跑完"


def cell_unrecorded_burst_balanced(world):
    world.resources(local=(40, 40), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server()
    # 放行文件写成相对假仓根的名字：本机那几条在 world.repo 里等它，假第二台照 fake-peer-wait 在假仓根里等同一个名字（两边认的是同一份放行文件）。
    # 派到第二台的那几条也要一直占着到放行（fake-peer-wait）：假第二台不起命令、当场退 0 的话，交得慢时每一条起之前前一条已在第二台收尾、第二台总显得空，五条全去第二台
    releases = [f"release-burst-{number}" for number in range(1, UNRECORDED_BURST_COUNT + 1)]
    task_ids = [world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release, f"fake-peer-wait={release}"], cwd=world.repo) for release in releases]
    placed = {}
    try:
        for task_id in task_ids:
            summary = world.wait_for(task_id, lambda summary: bool(summary["attempts"]), "起第一次尝试")
            placed[task_id] = summary["attempts"][0].get("host")
    finally:
        for release in releases:
            open(os.path.join(world.repo, release), "w").close()
    for task_id in task_ids:
        world.finished(task_id)
    counts = {host: list(placed.values()).count(host) for host in ("local", "peer")}
    if counts["local"] < 1 or counts["peer"] < 1 or sum(counts.values()) != UNRECORDED_BURST_COUNT:
        raise SelftestFailure(f"本机总上限 40G、第二台 64G，连交 {UNRECORDED_BURST_COUNT} 条没记录的命令（各记 {UNRECORDED_DEFAULT_PINNED_TEXT}、都派得到第二台）："
                              f"按剩余占比均衡，两台都应当分到，实际各条第一次尝试的台 {placed}", world.server_log_lines())
    return f"连交 {UNRECORDED_BURST_COUNT} 条没记录的命令各记 {UNRECORDED_DEFAULT_PINNED_TEXT}，按剩余占比分到两台：本机 {counts['local']}、第二台 {counts['peer']}"


def cell_memory_cap_at_maximum(world):
    world.resources(local=(55, 50))   # 总上限 50G（真机上这个数取多机配置 SLICE_MEMORY_CAP_BY_HOST 里本机那一项），没有第二台
    world.seed_peak("selftest-at-maximum", 20 * GIBIBYTE)
    world.seed_peak("selftest-at-maximum-switch", 20 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,250"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-at-maximum"})
    summary = world.finished(task_id)
    caps = [attempt.get("memory_cap_text") for attempt in summary["attempts"]]
    if caps != ["30720M", "46080M"] or summary["task"].get("outcome") != "memory-cap-at-maximum" or summary["task"].get("exit_code") != 236:
        raise SelftestFailure(f"总上限 50G、没有第二台：记录峰值 20G 起 30720M，撞了乘 1.5 到 46080M，再撞（再乘 1.5 就过总上限）记 memory-cap-at-maximum 打回、退 236，"
                              f"实际上限 {caps}、{describe(summary)}")
    completed = world.client(["wait", str(task_id)])
    last = completed.stdout.strip().splitlines()[-1] if completed.stdout.strip() else ""
    expected = (f"{lib.RESULT_MARK} task={task_id} type=memory-cap-at-maximum exit=236 ", " host=本机 reached=46080M next=撞到动态上限、打回：本机已给到 46080M（峰值 ")
    if (completed.returncode != 236 or not last.startswith(expected[0]) or expected[1] not in last
            or "核实这一趟的范围（点的项、测试目标）、分片与同时起的个数，收小或分片再交" not in last):
        raise SelftestFailure(f"两台都到顶时 wait 应当退 236、末行写撞了哪台（本机）、到了多少（46080M）、峰值，修改意见写核实范围、分片与同时起的个数，"
                              f"实际退 {completed.returncode}、末行「{last}」", [completed.stderr])
    world.resources(local=(55, 50), peer=(3, 80), peer_cap_gibibytes=80)   # 第二台总上限 80G、此刻只空 3G：头两次只能在本机
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,run"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-at-maximum-switch"}, cwd=world.repo)
    world.wait_for(task_id, lambda summary: len(summary["attempts"]) >= 2 and summary["attempts"][1].get("state") == "ended", "本机那两次收尾")
    world.resources(local=(55, 50), peer=(85, 80), peer_cap_gibibytes=80)
    summary = world.finished(task_id)
    placed = [(attempt.get("host"), attempt.get("memory_cap_text")) for attempt in summary["attempts"]]
    if placed != [("local", "30720M"), ("local", "46080M"), ("peer", "69120M")] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"本机 46080M 还撞、再乘 1.5（69120M）就过本机总上限 50G、第二台总上限 80G：应当换到第二台、上限 69120M、跑完，实际 {placed}、{describe(summary)}")
    return "撞上限乘 1.5 重排；再乘就到总上限的记 memory-cap-at-maximum 打回；有总上限更大的另一台就换过去"


def cell_retry_kinds(world):
    world.start_server()
    plain = ["bash", "-c", "exit 0"]
    squeezed = world.submit(plain + ["fake-wrapper-exits=254,run"])
    short = world.submit(plain + ["fake-wrapper-exits=245,245"])
    wrong = []
    for task_id, attempts, outcome in ((squeezed, 2, "succeeded"), (short, 2, "gpu-memory-short")):
        summary = world.finished(task_id)
        if not task_command_ok(summary, outcome, attempts):
            wrong.append(f"任务 {task_id} 应当尝试 {attempts} 次、结局 {outcome}，实际 {describe(summary)}")
    if wrong:
        raise SelftestFailure("254 同一上限重排、245 再试一次：" + "；".join(wrong))
    return "254 重排一次跑完、245 再试一次"


def cell_gpu_highest_free_card(world):
    world.resources(local=(100, 100), cards=f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE}", local_gpu=[(0, 16384), (1, 16384)])
    world.start_server()
    releases = [os.path.join(world.root, f"release-{number}") for number in (1, 2, 3)]
    gpu_tasks = []
    for release in releases:   # 一条起了再交下一条：分卡的先后只由到达次序定
        gpu_tasks.append(world.submit(["cargo", "test", "--features", "gpu", f"fake-quiesce=block:{release}"]))
        if len(gpu_tasks) < len(releases):
            world.running(gpu_tasks[-1])
    plain = world.submit(["bash", "-c", "exit 0"])
    plain_summary = world.finished(plain)
    waiting = world.summary(gpu_tasks[2])
    cards = [world.summary(task_id)["attempts"][0].get("card") for task_id in gpu_tasks[:2]]
    problems = []
    if cards != [LOCAL_CARD_ZERO, LOCAL_CARD_ONE]:
        problems.append(f"头两条 GPU 任务应当先后分到 {LOCAL_CARD_ZERO}、{LOCAL_CARD_ONE}，实际 {cards}")
    if waiting["task"].get("state") != "queued" or waiting["attempts"]:
        problems.append(f"两张卡都占着时第三条应当排着，实际 {describe(waiting)}")
    if plain_summary["attempts"][0].get("card") or world.calls_of(plain, "quiesce"):
        problems.append("不要卡的那一条不分卡、不经清场")
    if problems:
        raise SelftestFailure("；".join(problems))
    open(releases[0], "w").close()
    summary = world.running(gpu_tasks[2])
    for release in releases[1:]:
        open(release, "w").close()
    if summary["attempts"][0].get("card") != LOCAL_CARD_ZERO:
        raise SelftestFailure(f"{LOCAL_CARD_ZERO} 放出来之后第三条应当分到它，实际 {summary['attempts'][0].get('card')}")
    for task_id in gpu_tasks:
        world.finished(task_id)
    return "GPU 任务分到优先级最高的空卡、等卡时不挡不要卡的任务、不要卡的不分卡"


def cell_gpu_entry_scripts_recognized(world):
    world.resources(local=(100, 100), cards=LOCAL_CARD_ZERO, local_gpu=[(0, 16384)])
    os.makedirs(os.path.join(world.repo, "crates"), exist_ok=True)
    with open(os.path.join(world.repo, "crates", "mutations.tsv"), "w", encoding="utf-8") as handle:
        handle.write("# 自证的变异表\n"
                     "selftest-gpu-row\tsrc/a.rs\tx\ty\t-p singlefs-verification --features gpu --lib -- t\tt\n"
                     "selftest-cpu-row\tsrc/b.rs\tx\ty\t-p singlefs-core --lib -- t\tt\n")
    world.start_server()
    problems = []
    for words, expected in ((["bash", "scripts/crash/crash-point-replay.sh", "--item", "selftest-item"], True),
                            (["bash", "scripts/gate/merge-and-verify.sh", "--skip-apply", "patch"], True),
                            (["bash", "scripts/crash/crash-point-replay.sh", "--list-items"], False),
                            (["bash", ".claude/gate.d/crates-mutation-replay.sh", world.repo], True),
                            (["bash", ".claude/gate.d/crates-mutation-replay.sh", "--list-items", world.repo], False),
                            (["bash", "scripts/mutation/prove-red.sh", "--copy", world.repo, "singlefs-verification", "selftest-gpu-row"], True),
                            (["bash", "scripts/mutation/prove-red.sh", "--copy", world.repo, "singlefs-core", "selftest-cpu-row"], False)):
        task_id = world.submit([*words, "fake-wrapper-exits=0"])
        summary = world.finished(task_id)
        if bool(summary["task"].get("gpu")) != expected:
            problems.append(f"{' '.join(words)} 应当{'认成' if expected else '不认成'} GPU 任务，实际 gpu={summary['task'].get('gpu')}（{describe(summary)}）")
        elif expected and not world.calls_of(task_id, "quiesce"):
            problems.append(f"{' '.join(words)} 认成了 GPU 任务，应当由服务在最外层套清场，实际没经清场包装起（{describe(summary)}）")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return "crash-point-replay.sh、merge-and-verify.sh 认成 GPU 任务、由服务套清场，带 --list-items 的不认；变异门禁 crates-mutation-replay.sh 表里有上卡的行、认成 GPU 任务、由服务套清场，带 --list-items 的不认；prove-red.sh 点名带 gpu 特性那一行的认成要卡、由服务套清场，别的行不认"


def cell_cancel_own_session_only(world):
    world.start_server()
    release = os.path.join(world.root, "release")
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], session=SESSION_A)
    unit = world.running(task_id)["attempts"][0].get("unit", "")
    completed = world.client(["cancel", str(task_id)], session=SESSION_B)
    if completed.returncode != 3 or not world.unit_active(unit):
        open(release, "w").close()
        raise SelftestFailure(f"别的会话取消应当被拒（退 3）、单元照跑，实际退 {completed.returncode}、单元{'在跑' if world.unit_active(unit) else '停了'}", [completed.stderr])
    completed = world.client(["cancel", str(task_id)], session=SESSION_A)
    active = world.unit_active(unit)
    if completed.returncode != 0 or active:
        open(release, "w").close()
        raise SelftestFailure(f"同一会话取消应当退 0、回来时单元已停，实际退 {completed.returncode}、单元{'还在跑' if active else '停了'}", [completed.stdout, completed.stderr])
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "cancelled" or summary["task"].get("exit_code") != 239:
        raise SelftestFailure(f"取消之后结局应当是 cancelled、退 239，实际 {describe(summary)}")
    return "别的会话取消退 3、单元照跑；同会话取消退 0、单元已停、结局 cancelled"


def cell_queued_revalidated_on_restart(world):
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    stale = world.submit(["bash", "-c", "exit 0"])
    fine = world.submit(["bash", "-c", "exit 0"])
    world.stop_server()
    world.rewrite_signed_task(stale, lambda record: record.update(command=record["command"] + ["--memory", "8G"]))   # 照旧规则交进来的那一条
    world.start_server()
    summary = world.finished(stale, timeout=20)
    waiting = world.summary(fine)
    world.resources(local=(30, 30))
    world.finished(fine)
    if summary["task"].get("outcome") != "resubmit-required" or summary["task"].get("exit_code") != 232 or "--memory" not in (summary["task"].get("note") or "") \
            or summary["attempts"] or waiting["task"].get("state") != "queued":
        raise SelftestFailure(f"服务起来时照现行规则再核在排的：带 --memory 的那一条应当打回 resubmit-required（退 232、说明点名 --memory、一次没起），合规的照排；"
                              f"实际 {describe(summary)}、说明「{summary['task'].get('note')}」、合规的 {waiting['task'].get('state')}", world.server_log_lines())
    return "服务起来时照现行规则再核在排的：不合的打回 resubmit-required（退 232、点名哪一条）、合规的照排"


def cell_tampered_row_not_started(world):
    world.start_server()
    marker = os.path.join(world.root, "bypass-ran")
    record = {"command": ["bash", "-c", ': > "$1"', "touch-marker", marker], "cwd": world.work, "environment": {}, "session": SESSION_A, "submitter_pid": 0,
              "submitted_at": time.time(), "command_key": "selftest-bypass", "state": "queued", "outcome": "", "exit_code": None, "collected": False,
              "next_cap": None, "next_host": "", "outcome_counts": {}, "note": "", "cancel_requested": False, "attempt_count": 0, "gpu": False,
              "gpu_reason": ""}
    task_id = world.insert_bypass_row(record)
    summary = world.finished(task_id, timeout=20)
    alerted = [alert for alert in world.alerts() if alert.get("kind") == "record-tampered" and alert.get("task") == task_id]
    if summary["task"].get("outcome") != "record-tampered" or os.path.exists(marker) or world.calls_of(task_id) or not alerted:
        raise SelftestFailure(f"绕过服务直接写进库的任务应当不起、记 record-tampered、写告警，实际 {describe(summary)}、命令{'跑了' if os.path.exists(marker) else '没跑'}、"
                              f"告警 {len(alerted)} 条")
    return "绕过服务写进库的行不起、记 record-tampered、写了告警"


def cell_forged_session(world):
    world.start_server()
    forged = world.submit(["bash", "-c", "exit 0"], session=SESSION_A, forged=SESSION_FORGED)
    human = world.submit(["bash", "-c", "exit 0"], session=None)
    sessions = [world.summary(task_id)["task"].get("session") for task_id in (forged, human)]
    if sessions != [SESSION_A, ""]:
        raise SelftestFailure(f"客户端自己环境里的会话号是伪造的：记的应当是进程树上读到的 {SESSION_A}，人手的记空串，实际 {sessions}")
    return "伪造的会话号不认，记进程树上读到的；没有会话号的记空串"


def cell_environment_whitelist(world):
    world.start_server()
    completed = world.client(["submit", "--", "bash", "-c", "exit 0"], extra={"SELFTEST_UNLISTED_VARIABLE": "1"})
    if completed.returncode != 3 or "SELFTEST_UNLISTED_VARIABLE" not in completed.stderr:
        raise SelftestFailure(f"白名单外的变量应当当场拒（退 3）并点名，实际退 {completed.returncode}", [completed.stdout, completed.stderr])
    completed = world.client(["submit", "--", "bash", "-c", "exit 0"], extra={"RUN_WITH_MEMORY_CAP_WAIT_SECONDS": "5"})
    if completed.returncode != 3 or "RUN_WITH_MEMORY_CAP_WAIT_SECONDS" not in completed.stderr or "排队归 task-server" not in completed.stderr:
        raise SelftestFailure(f"排队类变量应当当场拒（退 3）并说明排队归服务，实际退 {completed.returncode}", [completed.stdout, completed.stderr])
    probe = os.path.join(world.root, "scope-seen")
    task_id = world.submit(["bash", "-c", 'printf "%s" "$SINGLEFS_RUN_SCOPE" > "$1"', "probe", probe], extra={"SINGLEFS_RUN_SCOPE": "selftest-scope"})
    world.finished(task_id)
    seen = ""
    with contextlib.suppress(OSError):
        with open(probe, encoding="utf-8") as handle:
            seen = handle.read()
    if seen != "selftest-scope" or world.calls()[:-1]:
        raise SelftestFailure(f"白名单里的 SINGLEFS_RUN_SCOPE 应当带进任务、被拒的两次一条都没起，实际任务看到「{seen}」、包装调用 {world.calls()}")
    return "白名单外与排队类变量当场拒并点名，白名单里的带进去"


def cell_retired_variables_refused(world):
    """退役的名字（lib.RETIRED_VARIABLES，文件头「环境变量」）：环境里设了的、命令外层写了「名=值」的（env 后面、bash -c 引号里命令位置前）都当场拒（task submit 退 3、不给任务号），
    说明点名它、写的是它那一行的出路（不是「白名单外」），一条都没起。"""
    world.start_server()
    wrong = {}
    for name, why in lib.RETIRED_VARIABLES.items():
        completed = world.client(["submit", "--", "bash", "-c", "exit 0"], extra={name: "1"})
        if completed.returncode != 3 or name not in completed.stderr or why not in completed.stderr or "白名单外" in completed.stderr:
            wrong[f"环境里的 {name}"] = (completed.returncode, completed.stderr.strip()[-300:])
    for words, name in ((["env", "SINGLEFS_HEAVY_TESTS=commit", "true"], "SINGLEFS_HEAVY_TESTS"),
                        (["bash", "-c", "GATE_MUTATION_START_OVER=1 true"], "GATE_MUTATION_START_OVER"),
                        (["GATE_MUTATION_START_OVER=1", "bash", "-c", "exit 0"], "GATE_MUTATION_START_OVER"),
                        (["env", "LANG=C", "SINGLEFS_HEAVY_TESTS=commit", "bash", "-c", "exit 0"], "SINGLEFS_HEAVY_TESTS")):
        completed = world.client(["submit", "--", *words])
        if completed.returncode != 3 or f"{name}=" not in completed.stderr or "去掉这个赋值" not in completed.stderr:
            wrong[" ".join(words)] = (completed.returncode, completed.stderr.strip()[-300:])
    started = [line for line in world.calls() if line.startswith("wrapper ")]
    if wrong or started:
        raise SelftestFailure(f"退役的名字设在环境里、写在命令外层都应当当场拒（退 3、点名它、写它那一行的出路）、一条都不起，实际 {wrong}、包装调用 {started}")
    return f"退役的 {len(lib.RETIRED_VARIABLES)} 个名字设在环境里、写在命令外层与 bash -c 里的都当场拒并写出路，一条都没起"


def cell_nested_dispatch_refused(world):
    world.start_server()
    completed = world.client(["submit", "--", "bash", "scripts/hosts/run-with-memory-cap.sh", "4G", "true"])
    if completed.returncode != 3 or "内存上限由服务定" not in completed.stderr:
        raise SelftestFailure(f"命令自己套内存包装应当被拒（退 3），实际退 {completed.returncode}", [completed.stdout, completed.stderr])
    completed = world.client(["submit", "--", "python3", "scripts/hosts/host-dispatch.py", "--", "true"])
    if completed.returncode != 3:
        raise SelftestFailure(f"命令自己套派发器应当被拒（退 3），实际退 {completed.returncode}", [completed.stdout, completed.stderr])
    refused = {}
    for words, mark in ((["nice", "-n", "5", "true"], "优先级"), (["timeout", "10", "true"], "限时"),
                        (["bash", "scripts/hosts/capped.sh", "4", "true"], "线程"), (["bash", "scripts/crash/gpu-quiesce-run.sh", "--", "true"], "清场"),
                        (["bash", "scripts/gate/merge-and-verify.sh", "--memory", "16G"], "--memory"), (["bash", "x.sh", "--threads=8"], "--threads"),
                        (["bash", "x.sh", "--peer"], "--peer"), (["env", "SINGLEFS_THREAD_CAP=7", "true"], "SINGLEFS_THREAD_CAP"),
                        (["bash", "-c", "cd x && bash scripts/hosts/run-with-memory-cap.sh 8G cargo test"], "内存上限由服务定"),
                        (["bash", "-c", "true; timeout 10 cargo build"], "限时"),
                        (["bash", "scripts/hosts/multi-host-run.sh", "--verdict-library-route", "--", "true"], "不套派发器"),
                        (["python3", "scripts/hosts/host-dispatch.py", "--plan", "--", "true"], "不套派发器")):
        completed = world.client(["submit", "--", *words])
        if completed.returncode != 3 or mark not in completed.stderr:
            refused[" ".join(words)] = (completed.returncode, completed.stderr.strip()[-200:])
    if refused:
        raise SelftestFailure(f"包装前缀与上限、线程、挑台参数应当当场拒（退 3、点名那一层或那个参数），实际 {refused}")
    task_id = world.submit(["bash", "scripts/hosts/run-with-memory-cap.sh", "--selftest", "fake-wrapper-exits=0"])
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"内存包装自己的 --selftest 不算套包装，应当照收、跑完，实际 {describe(summary)}")
    return ("套内存包装、派发器（带不带 --plan、--verdict-library-route 都一样）、nice、timeout、capped.sh、清场包装与写 --memory、--threads、--peer、"
            "SINGLEFS_THREAD_CAP 的都拒，包装自己的 --selftest 照收")


def cell_resource_record(world):
    scope = os.path.join(world.root, "fake-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "memory.stat"), "w", encoding="utf-8") as handle:
        handle.write(process_memory_stat(300 * MEBIBYTE))
    with open(os.path.join(scope, "memory.peak"), "w", encoding="utf-8") as handle:
        handle.write(f"{300 * MEBIBYTE}\n")
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 2000000\nuser_usec 1500000\nsystem_usec 500000\n")
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    command = ["bash", "-c", "sleep 1.2"]
    first = world.finished(world.submit(command))["attempts"][0]
    if (first.get("memory_peak") != 300 * MEBIBYTE or first.get("cpu_usec") != 2000000 or not isinstance(first.get("seconds"), (int, float))
            or first["seconds"] < 1.0 or not first.get("cores") or abs(first["cores"] - 2.0 / first["seconds"]) > 0.05):
        raise SelftestFailure(f"跑完应当记下用时（≥ 1 秒）、内存峰值 {300 * MEBIBYTE}、usage_usec 2000000 与平均在用核数（2 秒 CPU ÷ 用时），实际 "
                              f"用时 {first.get('seconds')}、峰值 {first.get('memory_peak')}、usage {first.get('cpu_usec')}、核数 {first.get('cores')}")
    second_id = world.submit(command)
    second = world.finished(second_id)["attempts"][0]
    if second.get("memory_cap_text") != "450M" or not any("cap=450M" in line for line in world.calls_of(second_id)):
        raise SelftestFailure(f"同一命令下一次应当按记录的峰值 300M 定上限 450M，实际 {second.get('memory_cap_text')}、包装调用 {world.calls_of(second_id)}")
    return f"记下用时 {first['seconds']} 秒、峰值 300M、平均 {first['cores']} 核；下一次上限 450M"


def cell_operation_log(world):
    world.start_server()
    release = os.path.join(world.root, "release")
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], session=SESSION_A)
    world.running(task_id)
    world.client(["cancel", str(task_id)], session=SESSION_B)
    completed = world.client(["cancel", str(task_id)], session=SESSION_A)
    open(release, "w").close()
    world.finished(task_id)
    seen = [(record.get("action"), record.get("session")) for record in world.operations() if record.get("task") == task_id]
    if seen != [("submit", SESSION_A), ("confirm", SESSION_A), ("cancel-refused", SESSION_B), ("cancel", SESSION_A)]:
        raise SelftestFailure(f"增加、确认与取消都应当记操作日志（submit、confirm、cancel-refused、cancel 与各自的会话），实际 {seen}", [completed.stderr])
    return "提交、确认、被拒的取消、取消都记进了仓外的操作日志"


def cell_unclaimed_finished(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_A)
    world.finished(task_id)
    completed = world.client(["status", "--json"], session=SESSION_A)
    listed = [item for item in json.loads(completed.stdout or "{}").get("tasks", []) if item.get("id") == task_id and item.get("unclaimed")]
    if not listed:
        raise SelftestFailure(f"完成了没人取的任务 {task_id} 应当在这个会话的 status 里列着，实际 {completed.stdout.strip()[:300]}")
    world.client(["wait", str(task_id)], session=SESSION_A)
    completed = world.client(["status", "--json"], session=SESSION_A)
    if any(item.get("id") == task_id for item in json.loads(completed.stdout or "{}").get("tasks", [])):
        raise SelftestFailure(f"wait 取过之后任务 {task_id} 不该再列成没人取，实际 {completed.stdout.strip()[:300]}")
    return "完成了没人取的列在 status 里，wait 取过之后不再列"


# ── 发消息给会话（task_server.py 文件头 ⑥）──
INBOX_TOKEN_SAMPLE = "selftest-inbox-token-5f0c2a9e"   # 假会话令牌：只该出现在收件 socket 收到的认证行里，不该出现在 status、日志与服务的输出里
INBOX_WAIT_SECONDS = 20.0     # 等一条消息的安静时限（发消息那一路自证里每 FAST_INBOX_ROUND_SECONDS 0.5 秒一轮）
INBOX_QUIET_SECONDS = 2.0     # 收到一条之后再看这么久（四轮），同一样不该再多收一条
HEALTH_SETTLE_SECONDS = 2.0   # 服务健康的毛病撤掉之后等这么久（四轮），让服务看到好了、忘掉发过的那一样
INBOX_COLLECTED_SECONDS = 10.0   # 结局那一条收到之后等服务记成已取的安静时限
UNCLAIMED_WARNING_SAMPLE_SECONDS = 1   # unclaimed-results-warned：结局满这么多秒没人取就警告（TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS）


class FakeInbox:
    """假的会话收件 socket（task_server.py 文件头 ⑥）：这一格 socket 目录里的一个 unix socket，一个线程收连接；一个连接是一条消息，收到的整行逐行记下，
    同时追加进假世界目录里的 inbox-<名>（世界有进展的记号）。令牌是 INBOX_TOKEN_SAMPLE 接名字。World.close 关它。"""

    def __init__(self, world, name, session=SESSION_A, token=None, register=True):
        self.path = os.path.join(world.short_root, f"inbox-{name}.sock")
        self.record = os.path.join(world.world, f"inbox-{name}")
        self.token = token or f"{INBOX_TOKEN_SAMPLE}-{name}"
        self.session = session
        self.connections = []
        self.lock = threading.Lock()
        self.closing = threading.Event()
        self.listener = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.listener.bind(self.path)
        self.listener.listen(16)
        self.listener.settimeout(0.2)
        self.thread = threading.Thread(target=self.serve, name=f"fake-inbox-{name}", daemon=True)
        self.thread.start()
        world.inboxes.append(self)
        if register:   # 把这个会话登进假的会话登记目录：服务照会话号找到这个 socket（task_server.py 文件头 ⑥「收件地址」）
            world.register_session(session, self.path, f"inbox-{name}")

    def environment(self):
        """交给 task submit 的两个变量（同 Claude Code 导出给 Bash 命令的）。"""
        return {lib.INBOX_SOCKET_VARIABLE: self.path, lib.INBOX_TOKEN_VARIABLE: self.token}

    def serve(self):
        while not self.closing.is_set():
            try:
                connection, _ = self.listener.accept()
            except socket.timeout:
                continue
            except OSError:
                return
            data = b""
            with connection:
                connection.settimeout(5)
                with contextlib.suppress(OSError):
                    while True:
                        chunk = connection.recv(65536)
                        if not chunk:
                            break
                        data += chunk
            lines = data.decode("utf-8", "replace").splitlines()
            with self.lock:
                self.connections.append(lines)
            with open(self.record, "a", encoding="utf-8") as handle:
                handle.write(json.dumps(lines, ensure_ascii=False) + "\n")

    def messages(self):
        """[{auth, text, raw}]：每个连接一条；auth 是恰好两行、头一行是认证行且令牌对得上，text 是 user 那一行的正文（读不出交空串），raw 是收到的原行。"""
        with self.lock:
            connections = list(self.connections)
        found = []
        for lines in connections:
            parsed = []
            for line in lines:
                try:
                    parsed.append(json.loads(line))
                except ValueError:
                    parsed.append(None)
            auth = len(parsed) == 2 and parsed[0] == {"type": "auth", "token": self.token}
            user = parsed[-1] if parsed and isinstance(parsed[-1], dict) and parsed[-1].get("type") == "user" else {}
            message = user.get("message") if isinstance(user.get("message"), dict) else {}
            text = message.get("content", "") if message.get("role") == "user" and isinstance(message.get("content"), str) else ""
            found.append({"auth": auth, "text": text, "raw": lines})
        return found

    def headed(self, head):
        """正文第一行以 head 起头的那几条。"""
        return [message for message in self.messages() if message["text"].split("\n", 1)[0].startswith(head)]

    def wait_for(self, world, head, count=1, limit=INBOX_WAIT_SECONDS):
        """等到正文第一行以 head 起头的收到 count 条（limit 是安静时限）；交此刻收到的那几条（不够数也交，调用处判）。"""
        patience = world.patience(limit)
        while patience.waiting():
            if len(self.headed(head)) >= count:
                break
            time.sleep(POLL_SECONDS)
        return self.headed(head)

    def close(self):
        self.closing.set()
        with contextlib.suppress(OSError):
            self.listener.close()
        self.thread.join(timeout=5)


def inbox_problems(messages, head, what, count=1):
    """收件 socket 的共同判据：第一行以 head 起头的恰好 count 条，每一条都先发了认证行、令牌对得上。→ 问题表。"""
    problems = []
    if len(messages) != count:
        problems.append(f"{what}：收件 socket 应当恰好收到 {count} 条第一行是「{head}…」的消息，实际 {len(messages)} 条")
    for message in messages:
        if not message["auth"]:
            problems.append(f"{what}：每条消息应当先发一行认证行、令牌对得上，再发一行 user，实际收到 {[line[:80] for line in message['raw']]}")
    return problems


def start_inbox_server(world, extra=None):
    """发消息那几格起服务：服务盯自己的单元那一样用假 systemctl、照假世界的 systemctl-state 打 active（不随这一台的真单元变）。"""
    with open(os.path.join(world.world, "systemctl-state"), "w", encoding="utf-8") as handle:
        handle.write("active\n")
    world.start_server(extra=dict({"TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl")}, **(extra or {})))


def server_output_text(world):
    """这一格各次服务的输出整份（server-<n>.log）。"""
    texts = []
    for number in range(1, world.server_logs + 1):
        with contextlib.suppress(OSError):
            with open(os.path.join(world.root, f"server-{number}.log"), encoding="utf-8", errors="replace") as handle:
                texts.append(handle.read())
    return "\n".join(texts)


def inbox_outcome_case(world, script, outcome, exit_text):
    """一条任务有了结局（task_server.py 文件头 ⑥「结局」）：会话的收件 socket 恰好收到一条，先认证行、再 user 那一行，正文第一行「task-server：任务 <号> <结局>」，
    后面写原因、出路、日志与 wait 的末行原样；令牌不在 status、日志与服务的输出里。→ (问题表, 任务号, 正文)。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world)
    task_id = world.submit(["bash", "-c", script], extra=inbox.environment())
    world.finished(task_id)
    head = f"task-server：任务 {task_id} {outcome}"
    found = inbox.wait_for(world, head)
    time.sleep(INBOX_QUIET_SECONDS)   # 结局只发一次：再看几轮不该多一条
    found = inbox.headed(head)
    problems = inbox_problems(found, head, f"结局 {outcome}")
    text = found[0]["text"] if found else ""
    for wanted in ("原因：", "出路：", "日志：", f"{lib.RESULT_MARK} task={task_id} type={outcome} {exit_text}"):
        if wanted not in text:
            problems.append(f"结局那一条的正文里应当有「{wanted}」，实际「{text[:400]}」")
    shown = world.client(["status", str(task_id), "--json"]).stdout + world.client(["log", str(task_id)]).stdout + server_output_text(world)
    if inbox.token in shown:
        problems.append("会话令牌不该出现在 status、task log 与服务的输出里，实际出现了")
    return problems, task_id, text


def cell_inbox_outcome_succeeded(world):
    """会成功的一条（task_server.py 文件头 ⑥）：收件 socket 收到一条结局 succeeded（inbox_outcome_case 的判据）；发出去了服务记成已取，status 不再列成没人取。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、pushed-outcome-not-collected 下这一格红。"""
    problems, task_id, _ = inbox_outcome_case(world, "echo inbox-done; exit 0", "succeeded", "exit=0")
    patience = world.patience(INBOX_COLLECTED_SECONDS)
    listed = True
    while patience.waiting() and listed:
        listed = any(item.get("id") == task_id for item in json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", []))
        time.sleep(POLL_SECONDS)
    if listed:
        problems.append(f"结局发出去之后任务 {task_id} 应当记成已取、status 不再列成完成了没人取，实际还列着")
    if problems:
        raise SelftestFailure("结局 succeeded 发给会话：" + "；".join(problems), world.server_log_lines())
    return f"收件 socket 收到一条「task-server：任务 {task_id} succeeded」（认证行在前、令牌对得上），发出去之后记成已取，令牌不进 status 与日志"


def cell_inbox_outcome_failed(world):
    """会失败的一条（命令退 3）：收件 socket 收到一条结局 command-failed（inbox_outcome_case 的判据），末尾 wait 的末行 exit=3。
    弄坏开关 inbox-not-sent、inbox-auth-omitted 下这一格红。"""
    problems, task_id, _ = inbox_outcome_case(world, "echo inbox-failed; exit 3", "command-failed", "exit=3")
    if problems:
        raise SelftestFailure("结局 command-failed 发给会话：" + "；".join(problems), world.server_log_lines())
    return f"收件 socket 收到一条「task-server：任务 {task_id} command-failed」（认证行在前、令牌对得上），末尾是 exit=3 的末行"


def cell_inbox_queued_too_long_once(world):
    """排队过久（task_server.py 文件头 ⑥「要处置的提醒」，同 attention-reported-once 的世界：第二台用不了、只有它放得下）：头一档（6 秒）收件 socket 收到一条
    「task-server：任务 <号> queued-too-long（第 1 档」，正文写依据（已排多久）、原因、出路、日志与 TASK_SERVER_ATTENTION 一行；再看几轮同一档不多一条；排满第二档（12 秒）
    再收一条第 2 档的。弄坏开关 inbox-not-sent、inbox-auth-omitted、inbox-reported-every-round 下这一格红。"""
    inbox = FakeInbox(world, "a")
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")
    world.seed_peak("selftest-inbox-queue", 30 * GIBIBYTE)
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": str(ATTENTION_ONCE_SECONDS)})
    task_id = world.submit(["bash", "-c", "exit 0", "inbox-queue"], extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: "selftest-inbox-queue"}), cwd=world.repo)
    first_head = f"task-server：任务 {task_id} queued-too-long（第 1 档"
    inbox.wait_for(world, first_head)
    time.sleep(ATTENTION_ONCE_QUIET_SECONDS)
    first = inbox.headed(first_head)
    problems = inbox_problems(first, first_head, "排队过久头一档")
    text = first[0]["text"] if first else ""
    for wanted in ("依据：已排", "原因：", "出路：", f"{lib.ATTENTION_MARK} task={task_id} type={lib.ATTENTION_TYPE} kind=queued-too-long tier=1"):
        if wanted not in text:
            problems.append(f"排队过久那一条的正文里应当有「{wanted}」，实际「{text[:400]}」")
    second_head = f"task-server：任务 {task_id} queued-too-long（第 2 档"
    inbox.wait_for(world, second_head)
    time.sleep(INBOX_QUIET_SECONDS)
    problems += inbox_problems(inbox.headed(second_head), second_head, "排队过久第二档")
    problems += [f"排满第二档之后头一档那一条仍该只有一条，实际 {len(inbox.headed(first_head))} 条"] if len(inbox.headed(first_head)) != 1 else []
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    world.finished(task_id)
    if problems:
        raise SelftestFailure("排队过久发给会话：" + "；".join(problems), world.server_log_lines())
    return f"排队过久头一档、第二档各收到恰好一条（认证行在前），同一档不重发"


INBOX_SHARD_KEY = "selftest-inbox-shard-queue"   # inbox-shard-attention-merged：三片分片的命令键，三片各记过 30G（只有第二台放得下，第二台用不了，三片一齐排着）


def cell_inbox_shard_attention_merged(world):
    """分片的几片同时排久（task_server.py 文件头 ⑥「要处置的提醒」的合成一条，Scheduler.attention_groups）：三片各记过 30G、本机 slice 20G、第二台用不了，
    三片一齐排着；排满头一档（6 秒）收件 socket 只收到一条「task-server：任务 <父任务号> queued-too-long（第 1 档，子任务 <号的区间>，共 3 片）」，
    第一行点到三片的号；再看几轮不多一条。弄坏开关 inbox-attention-per-shard 下逐片各发一条（收到三条），这一格红。"""
    inbox = FakeInbox(world, "a")
    for index in range(3):
        world.seed_peak(lib.shard_child_key(INBOX_SHARD_KEY, "shard", index, 3), 30 * GIBIBYTE)
    world.resources(local=(50, 20), peer_down="自证：第二台此刻用不了")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": str(ATTENTION_ONCE_SECONDS)})
    task_id = world.submit(world.write_spec("spec-inbox-shards.json", {"shards": [{}, {}, {}]}), cwd=world.repo,
                           extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: INBOX_SHARD_KEY}))
    summary = world.wait_for(task_id, lambda summary: queued_shards(summary) == [0, 1, 2], "三片都排着")
    children = sorted(child["task"]["id"] for child in shard_children(summary).values())
    head = f"task-server：任务 {task_id} queued-too-long（第 1 档"
    inbox.wait_for(world, head)
    time.sleep(INBOX_QUIET_SECONDS)
    messages = inbox.headed(head)
    problems = inbox_problems(messages, head, "三片同时排久")
    headline = messages[0]["text"].split("\n", 1)[0] if messages else ""
    children_range = f"子任务 {children[0]}–{children[-1]}" if children == list(range(children[0], children[0] + len(children))) else None
    if messages and (children_range is None or children_range not in headline or "共 3 片" not in headline):
        problems.append(f"合成的那一条第一行应当把三片的号 {children} 写成区间「{children_range}」并带「共 3 片」，实际「{headline}」")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    world.finished(task_id)
    if problems:
        raise SelftestFailure("分片几片同时排久合成一条发给会话：" + "；".join(problems)
                              + f"（收到的第一行：{[message['text'].split(chr(10), 1)[0] for message in inbox.headed(head)]}）", world.server_log_lines())
    return f"三片同时排满头一档，会话只收到一条：「{headline}」"


def cell_inbox_unreachable_task_runs(world):
    """收件 socket 不在（会话已关、会话登记里也没有它）：任务照跑到 succeeded，服务写一条告警 inbox-unreachable、那条任务的日志里有 lib.INBOX_LOG_MARK 起头的一行，令牌不进告警与日志；
    同一个场景的另一头：会话环境里没有那两个变量的提交客户端当场拒（退 3，异常表 inbox-missing：出路写从 Claude Code 会话的 Bash 里交），不交给服务、不进库。
    弄坏开关 inbox-not-sent 下不写告警、inbox-missing-accepted 下没有收件地址的也照交，这一格红。"""
    start_inbox_server(world)
    absent = os.path.join(world.short_root, "inbox-absent.sock")
    task_id = world.submit(["bash", "-c", "echo inbox-unreachable; exit 0"], extra={lib.INBOX_SOCKET_VARIABLE: absent, lib.INBOX_TOKEN_VARIABLE: INBOX_TOKEN_SAMPLE})
    done = world.finished(task_id)
    problems = [] if done["task"].get("outcome") == "succeeded" else [f"收件 socket 不在时任务照跑，应当 succeeded，实际 {describe(done)}"]
    patience = world.patience(INBOX_WAIT_SECONDS)
    alerted = []
    while patience.waiting() and not alerted:
        alerted = [alert for alert in world.alerts() if alert.get("kind") == "inbox-unreachable" and alert.get("task") == task_id]
        time.sleep(POLL_SECONDS)
    if not alerted:
        problems.append(f"发不出去应当写一条 inbox-unreachable 告警、点名任务 {task_id}，实际告警 {world.alerts()}")
    log = world.client(["log", str(task_id), "--all"]).stdout
    if lib.INBOX_LOG_MARK not in log:
        problems.append(f"发不出去应当往任务的日志里接一行 {lib.INBOX_LOG_MARK} 起头的告警，实际日志「{log.strip()[-300:]}」")
    if INBOX_TOKEN_SAMPLE in log or INBOX_TOKEN_SAMPLE in json.dumps(world.alerts(), ensure_ascii=False):
        problems.append("会话令牌不该出现在告警与任务日志里，实际出现了")
    before = len(world.signed_rows("tasks"))
    completed = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "inbox-absent"],
                             extra={lib.INBOX_SOCKET_VARIABLE: "", lib.INBOX_TOKEN_VARIABLE: ""})
    wanted = lib.anomaly_advice("inbox-missing", "")
    if (completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or "读不到会话的收件地址" not in completed.stderr or wanted not in completed.stderr
            or len(world.signed_rows("tasks")) != before):
        problems.append(f"没有收件地址的提交应当当场拒（退 3、不打任务号、不进库）、标准错误打「读不到会话的收件地址」与出路「{wanted}」，"
                        f"实际退 {completed.returncode}、库里任务 {before} → {len(world.signed_rows('tasks'))} 条、「{completed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("收件 socket 不在：" + "；".join(problems), world.server_log_lines())
    return "收件 socket 不在时任务照跑到 succeeded、写了 inbox-unreachable 告警与日志里一行，令牌不进告警与日志；没有收件地址的提交客户端当场拒、不进库"


def health_notified_once(world, label, head, arm, disarm, extra=None):
    """服务健康的一样（task_server.py 文件头 ⑥「服务健康」）：一个假会话名下有一条在跑的任务；arm 造出毛病，收件 socket 恰好收到一条第一行「task-server：服务健康 <名字>」的
    （认证行在前、令牌对得上），再看几轮不多一条；disarm 撤掉、任务放行。→ 那一条的正文；判不过抛 SelftestFailure。"""
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-health")
    start_inbox_server(world, extra)
    try:
        task_id = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "health", release], extra=inbox.environment())
        world.running(task_id)
        arm()
        inbox.wait_for(world, head)
        time.sleep(INBOX_QUIET_SECONDS)
        found = inbox.headed(head)
    finally:
        disarm()
        open(release, "w").close()
    problems = inbox_problems(found, head, label)
    text = found[0]["text"] if found else ""
    if found and ("问题：" not in text or "出路：" not in text or "control check" not in text):
        problems.append(f"服务健康那一条应当写「问题：」与「出路：」（出路点名 control check），实际「{text[:400]}」")
    if problems:
        raise SelftestFailure(f"服务健康（{label}）：" + "；".join(problems), world.server_log_lines())
    return text


def write_unit_state(world, state):
    with open(os.path.join(world.world, "systemctl-state"), "w", encoding="utf-8") as handle:
        handle.write(f"{state}\n")


def cell_health_unit_missing_notified(world):
    """服务自己的单元不在（假 systemctl 的 is-active 打 inactive）：发一条「task-server：服务健康 单元不在」（health_notified_once 的判据）。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、health-unwatched、health-alerted-every-round 下这一格红。"""
    text = health_notified_once(world, "单元不在", "task-server：服务健康 单元不在", lambda: write_unit_state(world, "inactive"), lambda: write_unit_state(world, "active"))
    if "is-active" not in text:
        raise SelftestFailure(f"单元不在那一条应当写 is-active 打的状态，实际「{text[:300]}」", world.server_log_lines())
    return "单元不在：收件 socket 收到恰好一条服务健康，写了 is-active 打的状态"


def cell_health_nfs_notified(world):
    """NFS 共享前三项不过（假 ssh 的 mountpoint 退 1，nfs_loop 复核判出「挂载」不过）：发一条「task-server：服务健康 NFS 共享判不过」，点名「挂载」那一项。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、health-unwatched、health-alerted-every-round、nfs-unchecked 下这一格红。"""
    mount_exit = os.path.join(world.world, "fake-ssh-mount-exit")

    def arm():
        with open(mount_exit, "w", encoding="utf-8") as handle:
            handle.write("1\n")
    text = health_notified_once(world, "NFS 共享", "task-server：服务健康 NFS 共享判不过", arm, lambda: os.remove(mount_exit))
    if "「挂载」" not in text:
        raise SelftestFailure(f"NFS 那一条应当点名「挂载」那一项不过，实际「{text[:300]}」", world.server_log_lines())
    return "第二台没挂上共享根：收件 socket 收到恰好一条服务健康，点名「挂载」"


def cell_health_peer_unavailable_notified(world):
    """第二台用不了满时限（假余量的 peer_down；自证里 1 秒）：发一条「task-server：服务健康 第二台用不了」，写着那一句原因。任务放在本机（第二台放不下）。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、health-unwatched、health-alerted-every-round 下这一格红。"""
    world.resources(local=(100, 100), peer=(1, 30), peer_cap_gibibytes=64)
    why = "自证：第二台连不上"
    text = health_notified_once(world, "第二台用不了", "task-server：服务健康 第二台用不了",
                                lambda: world.resources(local=(100, 100), peer=(1, 30), peer_cap_gibibytes=64, peer_down=why),
                                lambda: world.resources(local=(100, 100), peer=(1, 30), peer_cap_gibibytes=64))
    if why not in text:
        raise SelftestFailure(f"第二台用不了那一条应当写着原因「{why}」，实际「{text[:300]}」", world.server_log_lines())
    return "第二台用不了满时限：收件 socket 收到恰好一条服务健康，写着原因"


def cell_health_database_unwritable_notified(world):
    """库写不进（TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE 点名的文件在时试写当失败）：两个假会话各有一条在跑的任务，两个收件 socket 各收到恰好一条
    「task-server：服务健康 库写不进」；撤掉、等服务看到好了，再造一次，各再收恰好一条（好了再坏再发）。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、health-unwatched、health-alerted-every-round 下这一格红。"""
    arm = os.path.join(world.root, "health-write-fails")
    release = os.path.join(world.root, "release-health-database")
    inboxes = {SESSION_A: FakeInbox(world, "a"), SESSION_B: FakeInbox(world, "b", session=SESSION_B)}
    head = "task-server：服务健康 库写不进"
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_FAIL_HEALTH_WRITE": arm})
    problems = []
    try:
        for session, inbox in inboxes.items():
            world.running(world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "health-database", release], session=session, extra=inbox.environment()))
        for episode in (1, 2):
            open(arm, "w").close()
            for inbox in inboxes.values():
                inbox.wait_for(world, head, count=episode)
            time.sleep(INBOX_QUIET_SECONDS)
            for session, inbox in inboxes.items():
                problems += inbox_problems(inbox.headed(head), head, f"第 {episode} 回坏、会话 {session}", count=episode)
            os.remove(arm)
            time.sleep(HEALTH_SETTLE_SECONDS)
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(arm)
        open(release, "w").close()
    if problems:
        raise SelftestFailure("服务健康（库写不进）：" + "；".join(problems), world.server_log_lines())
    return "库写不进：两个会话各收到一条，好了再坏各再收一条"


def cell_control_watch_removed(world):
    """control watch 已删（task-server/control 文件头）：退 2，标准错误打「这个子命令已删，task-server 自己盯、出事直接发消息给发起任务的会话」，出路点名 control check。"""
    completed = world.control(["watch"])
    wanted = "这个子命令已删，task-server 自己盯、出事直接发消息给发起任务的会话"
    if completed.returncode != lib.USAGE_EXIT or wanted not in completed.stderr or "control check" not in completed.stderr:
        raise SelftestFailure(f"control watch 应当退 {lib.USAGE_EXIT}、打「{wanted}」与出路，实际退 {completed.returncode}、「{completed.stderr.strip()[-300:]}」")
    return f"control watch 退 {lib.USAGE_EXIT}、打出出路句"


def cell_unclaimed_results_warned(world):
    """交任务时会话名下有结局满时限没人取（task_server.py 文件头 ②「提交时警告」②；自证里时限 1 秒）：两条跑完、没起 wait，再交一条照收照跑，标准错误打
    「✗ 严厉警告：会话 <号> 有 2 条任务跑完了没人取（<号>、<号>）」与「task-server/task wait <号> 取走」；三条都 wait 取走之后再交，不打。
    弄坏开关 unclaimed-unwarned 下这一格红。"""
    world.start_server(extra={"TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS": str(UNCLAIMED_WARNING_SAMPLE_SECONDS)})
    left = [world.submit(["bash", "-c", "exit 0", f"unclaimed-{index}"]) for index in (1, 2)]
    for task_id in left:
        world.finished(task_id)
    time.sleep(UNCLAIMED_WARNING_SAMPLE_SECONDS + 0.5)
    problems = []
    warned = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "unclaimed-third"])
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", warned.stdout)
    head = f"✗ 严厉警告：会话 {SESSION_A} 有 2 条任务跑完了没人取（{left[0]}、{left[1]}）"
    if warned.returncode != 0 or not found or head not in warned.stderr or f"task-server/task wait {left[0]}" not in warned.stderr:
        problems.append(f"有两条没人取时交任务应当照收（退 0）、打「{head}」与「task-server/task wait <号> 取走」，实际退 {warned.returncode}、「{warned.stderr.strip()[-400:]}」")
    if found:   # 照收的那一条确认之后才进队、跑完（task_server.py 文件头 ⑦）
        world.confirm_submitted(warned)
    for task_id in left + ([int(found.group(1))] if found else []):
        world.finished(task_id)
        world.client(["wait", str(task_id)])
    quiet = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "unclaimed-fourth"])
    if quiet.returncode != 0 or "严厉警告" in quiet.stderr:
        problems.append(f"都取走之后再交不该打严厉警告，实际退 {quiet.returncode}、「{quiet.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("会话名下有结局没人取：" + "；".join(problems), world.server_log_lines())
    return f"两条跑完没人取时交任务打了严厉警告、点名 {left[0]}、{left[1]}，取走之后再交不打"


def cell_state_placement(world):
    environment = {name: value for name, value in world.client_environment.items() if name != "TASK_SERVER_SELFTEST_ALLOW_TMPFS"}
    in_repository = os.path.join(lib.REPOSITORY_ROOT, ".task-server-selftest-state")
    cases = (("tmpfs", os.path.join("/dev/shm", f"task-server-selftest-{os.getpid()}-state"), world.builds),
             ("落在仓里", in_repository, world.builds),
             ("运行时目录", os.path.join(world.runtime, "state"), world.builds),
             ("编译目录", world.state, os.path.join(lib.REPOSITORY_ROOT, ".task-server-selftest-builds")))
    for number, (what, state, build) in enumerate(cases, 1):
        config = os.path.join(world.root, f"placement-{number}.env")
        World.write_config(config, state, world.runtime, world.key, build, world.shared_root)
        saved = world.client_environment
        world.client_environment = dict(environment, **{lib.MULTI_HOST_CONFIG_VARIABLE: config})
        if what == "编译目录":
            world.client_environment["TASK_SERVER_SELFTEST_ALLOW_TMPFS"] = "1"   # 这一例只判编译目录：状态目录是这一格的临时目录，可能在 tmpfs 上
        try:
            completed = world.control(["check"])
        finally:
            world.client_environment = saved
        if completed.returncode != 78 or what not in completed.stderr or os.path.exists(state if what != "编译目录" else build):
            raise SelftestFailure(f"状态目录放在 {state}、编译目录放在 {build}（{what}）应当退 78 并说明，实际退 {completed.returncode}", [completed.stderr])
    return "状态目录在 tmpfs 上、仓里、运行时目录下，编译目录在仓里，都拒"


def cell_retry_keeps_earlier_attempts(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "echo attempt-$TASK_SERVER_ATTEMPT; exit 0", "fake-wrapper-exits=254,run"])
    summary = world.finished(task_id)
    numbers = [attempt.get("number") for attempt in summary["attempts"]]
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    logs = [attempt.get("log") for attempt in summary["attempts"]]
    if numbers != [1, 2] or outcomes != ["squeezed-out", "succeeded"] or len(set(logs)) != 2 or not all(log and os.path.isfile(log) for log in logs):
        raise SelftestFailure(f"重排一次之后两次尝试的记录都要在（第 1 次 squeezed-out、第 2 次 succeeded，各有各的日志），实际次序 {numbers}、类型 {outcomes}、日志 {logs}")
    return "重排之后前一次的记录照留：两次尝试、两份日志"


KEEP_TEMPORARY_EXTRA = {"SINGLEFS_KEEP_TEST_TEMPORARY_FILES": "1"}   # 提交时设了它：编译目录与现场留到 task clean（task_server_lib.py 文件头「编译目录」「现场」）


def cell_build_directory_outside_repository(world):
    """会编的 cargo 编在仓外这一次尝试的编译目录里；提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的跑完留着、wait 末行带 build=、只许同一会话 task clean 删
    （没设的记下结局就删，finished-build-removed 那一格判）。"""
    world.start_server()
    path = f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"
    task_id = world.submit(["cargo", "build"], extra=dict(KEEP_TEMPORARY_EXTRA, PATH=path))
    summary = world.finished(task_id)
    cargo_calls = [line for line in world.calls() if line.startswith("cargo ")]
    target = cargo_calls[-1].split(" target=", 1)[1].split(" args=", 1)[0] if cargo_calls else ""
    attempt = summary["attempts"][0] if summary["attempts"] else {}
    problems = []
    if summary["task"].get("outcome") != "succeeded" or not target:
        problems.append(f"假 cargo 应当跑完、记下给它的 CARGO_TARGET_DIR，实际 {describe(summary)}、调用 {cargo_calls}")
    elif not lib.inside(target, world.builds) or lib.inside_repository(target):
        problems.append(f"这一次尝试的编译目录应当在 {world.builds} 下（仓外），实际 {target}")
    elif not os.path.isdir(os.path.dirname(target)):
        problems.append(f"提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1，跑完之后这一次尝试的编译目录 {os.path.dirname(target)} 应当留着（交给提交它的 agent 用 task clean 删），实际不在")
    else:
        line = world.client(["wait", str(task_id)]).stdout.strip().splitlines()[-1:]
        if not line or f" build={os.path.dirname(target)}" not in line[0]:
            problems.append(f"wait 的末行应当带 build={os.path.dirname(target)}，实际 {line}")
        refused = world.client(["clean", str(task_id)], session=SESSION_B)
        if refused.returncode != 3 or not os.path.isdir(os.path.dirname(target)):
            problems.append(f"别的会话 task clean 应当被拒（退 3）、目录照留，实际退 {refused.returncode}、目录{'在' if os.path.isdir(os.path.dirname(target)) else '不在'}")
        cleaned = world.client(["clean", str(task_id)])
        if cleaned.returncode != 0 or os.path.lexists(os.path.dirname(target)) or "删了 1 份" not in cleaned.stdout:
            problems.append(f"同一会话 task clean 应当退 0、删了 1 份、目录没了，实际退 {cleaned.returncode}、目录{'还在' if os.path.lexists(os.path.dirname(target)) else '没了'}、输出 {cleaned.stdout.strip()[-200:]} {cleaned.stderr.strip()[-200:]}")
        again = world.client(["clean", str(task_id)])
        if again.returncode != 0 or "删了 0 份" not in again.stdout:
            problems.append(f"再 clean 一次应当退 0、删了 0 份，实际退 {again.returncode}、输出 {again.stdout.strip()[-200:]}")
    if os.path.lexists(os.path.join(world.work, "target")):
        problems.append(f"起它的目录 {world.work} 下长出了 target/（没给编译目录，cargo 照默认编在了起它的目录下）")
    if attempt.get("build_directory") != (os.path.dirname(target) if target else None):
        problems.append(f"尝试记录里的编译目录应当是 {os.path.dirname(target) if target else '?'}，实际 {attempt.get('build_directory')}")
    for refused_extra, refused_command in (({"CARGO_TARGET_DIR": os.path.join(lib.REPOSITORY_ROOT, "target")}, ["cargo", "build", "fake-wrapper-exits=0"]),
                                           ({}, ["cargo", "build", "--target-dir", os.path.join(lib.REPOSITORY_ROOT, "selftest-target"), "fake-wrapper-exits=0"])):
        completed = world.client(["submit", "--", *refused_command], extra=dict(refused_extra, PATH=path))
        if completed.returncode != 3 or "指到仓里" not in completed.stderr:
            problems.append(f"调用方给的编译目录指到仓里（{refused_extra or refused_command[3]}）应当当场拒（退 3、说指到仓里），实际退 {completed.returncode}")
    own = os.path.join(world.root, "own-target")
    kept = world.finished(world.submit(["cargo", "build"], extra={"PATH": path, "CARGO_TARGET_DIR": own}))
    if kept["task"].get("outcome") != "succeeded" or not os.path.isfile(os.path.join(own, "debug", "selftest-artifact")) or kept["attempts"][0].get("build_directory"):
        problems.append(f"调用方给的仓外编译目录应当照它用、不删、不另给，实际 {describe(kept)}、产物{'在' if os.path.isfile(os.path.join(own, 'debug', 'selftest-artifact')) else '不在'}")
    if problems:
        raise SelftestFailure("编译目录：" + "；".join(problems), world.server_log_lines())
    return (f"会编的 cargo 编在仓外的 {os.path.relpath(target, world.root)}、提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的跑完留着、wait 末行带 build=、"
            "别的会话 clean 拒、本会话 clean 删了，起它的目录没长 target/；指到仓里的当场拒；调用方给的仓外目录照用不删")


def cell_retry_build_directory_reused(world):
    """撞上限重排时编译目录接着用（task_server_lib.py 文件头「编译目录」的重试复用；task_server.py 的 kept_build_directory）：本机先退 250 再跑的会编命令，
    重试那一次的编译目录是上一次那一个（尝试记 build_directory_reused，假 cargo 拿到的 CARGO_TARGET_DIR 在它下面）；头一次在本机、撞了上限之后本机腾不出地方、
    重排换到第二台的，本机那一份在第 2 次起的时候删了。两条都设 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1：最后那一次的留到 task clean，判定不随「记下结局就删」那一步变
    （那一步归 finished-build-removed）。"""
    world.resources(local=(30, 30), peer=(10, 10))   # 两台都放得下 4G 与 6G：照剩余占比都挑本机
    crowded = os.path.join(world.root, "resources-local-crowded.json")
    world.resources(local=(5, 30), peer=(100, 100), path=crowded)   # 本机放不下 6G、第二台放得下
    world.start_server()
    extra = dict(KEEP_TEMPORARY_EXTRA, PATH=f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}")
    problems = []
    same = world.finished(world.submit(["cargo", "build", "fake-wrapper-exits=250,run"], extra=extra))
    attempts = same["attempts"]
    directories = [attempt.get("build_directory") or "" for attempt in attempts]
    targets = [line.split(" target=", 1)[1].split(" args=", 1)[0] for line in world.calls() if line.startswith("cargo dir=")]
    if (same["task"].get("outcome") != "succeeded" or len(attempts) != 2 or not all(directories) or directories[0] != directories[1]
            or not attempts[1].get("build_directory_reused") or targets != [os.path.join(directories[0], "target")]):
        problems.append(f"本机先退 250 再跑：重试那一次的编译目录应当是上一次那一个、记 build_directory_reused、假 cargo 拿到的 CARGO_TARGET_DIR 是它下面的 target，"
                        f"实际 {describe(same)}、编译目录 {directories}、第 2 次 build_directory_reused={attempts[-1].get('build_directory_reused') if attempts else None}、"
                        f"假 cargo 拿到 {targets}")
    moved = world.finished(world.submit(["cargo", "build", "--release", "fake-wrapper-exits=250,run", f"fake-wrapper-swap-resources={crowded}"], extra=extra),
                           timeout=PLACEMENT_WAIT_SECONDS)
    hosts = [attempt.get("host") for attempt in moved["attempts"]]
    local_directory = (moved["attempts"] or [{}])[0].get("build_directory") or ""
    if moved["task"].get("outcome") != "succeeded" or hosts != ["local", "peer"] or not local_directory or os.path.lexists(local_directory):
        problems.append(f"头一次在本机撞了上限、重排换到第二台：本机那一份编译目录应当在第 2 次起的时候删了，"
                        f"实际 {describe(moved)}、各次在 {hosts}、本机那一份 {local_directory or '（没记）'}{'还在' if local_directory and os.path.lexists(local_directory) else ''}")
    for summary in (same, moved):
        world.client(["clean", str(summary["task"]["id"])])
    if problems:
        raise SelftestFailure("重试的编译目录：" + "；".join(problems), world.server_log_lines())
    return "本机重排的那一次接着用上一次的编译目录；换到第二台的，本机那一份在第 2 次起的时候删了"


FINISHED_BUILD_GONE_SECONDS = 10.0   # finished-build-removed：结局记下之后至多等这么久看编译目录没了（finalize 先记结局再删）


def cell_finished_build_removed(world):
    """没设 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的任务记下结局就删它各次尝试的编译目录（task_server.py 的 remove_finished_build_directories），wait 末行不带 build=；
    再 task clean 照退 0、删了 0 份。"""
    world.start_server()
    path = f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"
    task_id = world.submit(["cargo", "build"], extra={"PATH": path})
    summary = world.finished(task_id)
    build = (summary["attempts"] or [{}])[0].get("build_directory") or ""
    problems = []
    if summary["task"].get("outcome") != "succeeded" or not build:
        raise SelftestFailure(f"假 cargo 应当跑完、尝试记下编译目录，实际 {describe(summary)}、编译目录「{build}」", world.server_log_lines())
    patience = world.patience(FINISHED_BUILD_GONE_SECONDS)
    while os.path.lexists(build) and patience.waiting():
        time.sleep(POLL_SECONDS)
    if os.path.lexists(build):
        problems.append(f"记下结局之后 {FINISHED_BUILD_GONE_SECONDS:.0f} 秒里编译目录 {build} 应当删了，实际还在")
    line = world.client(["wait", str(task_id)]).stdout.strip().splitlines()[-1:]
    if not line or " build=" in line[0]:
        problems.append(f"wait 的末行不该带 build=（编译目录已删），实际 {line}")
    cleaned = world.client(["clean", str(task_id)])
    if cleaned.returncode != 0 or "删了 0 份" not in cleaned.stdout:
        problems.append(f"再 task clean 应当退 0、删了 0 份，实际退 {cleaned.returncode}、输出 {cleaned.stdout.strip()[-200:]} {cleaned.stderr.strip()[-200:]}")
    if problems:
        raise SelftestFailure("记下结局就删编译目录：" + "；".join(problems), world.server_log_lines())
    return "没设 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的记下结局就删了编译目录、wait 末行不带 build=、再 clean 删了 0 份"


def cell_thirty_second_tick_prompt(world):
    world.start_server(extra={"TASK_SERVER_TICK_SECONDS": "30"})
    started = time.monotonic()
    task_id = world.submit(["bash", "-c", "sleep 2"])
    completed = world.client(["wait", str(task_id)], timeout=60, extra={"TASK_SERVER_SELFTEST_FAST": "0"})   # 照平常 20 秒一问：不叫醒就要等满一问
    elapsed = time.monotonic() - started
    if completed.returncode != 0 or elapsed > 12:
        raise SelftestFailure(f"主循环 30 秒一轮时，一条 2 秒的任务从提交到 wait 拿到结局应当不超过 12 秒（runner 写完结果叫醒主循环、服务记下结局叫醒 wait），"
                              f"实际退 {completed.returncode}、用了 {elapsed:.1f} 秒", world.server_log_lines())
    return f"主循环 30 秒一轮，2 秒的任务 {elapsed:.1f} 秒拿到结局（结束即叫醒、wait 不轮询）"


HEALTH_READ_SECONDS = 5   # health_until 每一次读 /health 的时限：到点没回不当服务没回，服务进程还活着就照耐心再读（read_health_patiently）


def read_health_patiently(world, socket_path):
    """读一次 /health（health_until 用）：一次读满 HEALTH_READ_SECONDS 没回（机器忙，任务 9728 第二台 load average 约 60 时这样超时）不当服务没回——
    服务进程还活着就照 world.patience 再读，它的 CPU 在涨算有进展（同起服务那一段等 /health）；服务进程退了、或安静满 STATE_TIMEOUT_SECONDS 还没回，判红。"""
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    while True:
        try:
            return lib.call_server(socket_path, "GET", "/health", timeout=HEALTH_READ_SECONDS)[1]
        except lib.ServerUnreachable as error:
            if world.server is not None and world.server.poll() is not None:
                raise SelftestFailure(f"读 /health 时服务已经退了（退 {world.server.returncode}）：{error}", world.server_log_lines()) from None
            if not patience.waiting(extra=world.server_cpu_ticks()):
                raise SelftestFailure(f"读 /health 读了 {patience.elapsed():.0f} 秒都没回（安静 {STATE_TIMEOUT_SECONDS:.0f} 秒没有进展）：{error}", world.server_log_lines()) from None


def health_until(world, wanted, seconds):
    """在 seconds 秒里每隔 POLL_SECONDS 读一次 /health，wanted(health) 为真就交它；到点交最后一次读到的。每一次读照 read_health_patiently（机器忙时一次读慢了不判红）。"""
    socket_path = os.path.join(world.runtime, lib.SOCKET_NAME)
    patience = world.patience(seconds)
    while True:
        health = read_health_patiently(world, socket_path)
        if wanted(health) or not patience.waiting():
            return health
        time.sleep(POLL_SECONDS)


STARVED_HEALTH_TEXT = "派活判它此刻放得下"   # task_server.py 的 starved_problem 报「判能起却没起成」那一句里的几个字（看进展那一句没有它）


def cell_starved_head_reported(world):
    """派活判排头能起却连着几轮没起成要报（task_server.py 的 starved_problem 读 head_verdict）：本机空 100G、排头一条小的，排队线程照判、判能起而不起
    （TASK_SERVER_SELFTEST_PLACEMENT_PAUSED），过了阈值（1 秒）/health 回不健康、问题是「派活判它此刻放得下」那一句；放开之后跑完、回 ok。
    弄坏开关 starved-head-unreported 下 /health 不报这一句，这一格红。"""
    world.resources(local=(100, 100))
    paused = os.path.join(world.root, "placement-paused")
    world.start_server(extra={"TASK_SERVER_SELFTEST_PLACEMENT_PAUSED": paused})
    open(paused, "w").close()
    task_id = world.submit(["bash", "-c", "exit 0"])
    health = health_until(world, lambda health: not health.get("ok"), 10)
    os.remove(paused)
    if health.get("ok") or STARVED_HEALTH_TEXT not in (health.get("problem") or ""):
        raise SelftestFailure(f"本机空 100G、排头一条小的、排队线程判能起而不起：过了阈值（1 秒）/health 应当回不健康、问题写「{STARVED_HEALTH_TEXT}」，实际 {health}",
                              world.server_log_lines())
    summary = world.finished(task_id)
    recovered = health_until(world, lambda health: health.get("ok"), 10)
    if summary["task"].get("outcome") != "succeeded" or not recovered.get("ok"):
        raise SelftestFailure(f"排队线程接着派之后那一条应当跑完、/health 回 ok，实际 {describe(summary)}、{recovered}", world.server_log_lines())
    return f"排头放得下却没起时 /health 回「{health.get('problem')}」，接着派之后跑完、回 ok"


def cell_blocked_head_not_starved(world):
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"])
    health = health_until(world, lambda health: not health.get("ok"), 3)
    world.resources(local=(30, 30))
    summary = world.finished(task_id)
    if not health.get("ok"):
        raise SelftestFailure(f"排头放不下（余量 hold）时排着不动是正常的，/health 应当一直回 ok，实际 {health}", world.server_log_lines())
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"放得下之后那一条应当跑完，实际 {describe(summary)}", world.server_log_lines())
    return "排头放不下时排着不动，3 秒里 /health 一直回 ok；放得下之后跑完"


FORMULA_HEAD_BLOCKER_PEAK_GIBIBYTES = 60   # starved-head-judged-by-the-formula：先占着的那一条的峰值（上限 90G，在 100G、余量 2G 的台上起得来）
FORMULA_HEAD_PEAK_GIBIBYTES = 8            # 那一格的排头的峰值（上限与记账 12G：judge_host 放得下，公式只剩 100 − 2 − 90 = 8G 放不下）
FORMULA_HEAD_ROOMY_GIBIBYTES = 200         # 那一格换上的假余量（空闲与 slice 都是这么多）：公式剩 200 − 2 − 90 = 108G，排头放得下
FORMULA_HEAD_WATCH_SECONDS = 4 * SERVER.FAST_STARVED_SECONDS   # 那一格公式放不下时看 /health 看这么久（过阈值几倍）
PAUSE_SETTLE_SECONDS = 1.0                 # 那一格停排队之后等这么久再换余量（一轮 0.1 秒：开了头的那一轮早排完，换上的余量不会被它读去起排头）


def cell_starved_head_judged_by_the_formula(world):
    """/health 只读派活给排头的结论（task_server.py 的 starved_problem 读 head_verdict；judge_placement 照 judged_on_host 过公式）：本机 100G、余量 2G，一条上限 90G 的在跑、
    它的 scope 已用满 90G（不记待涨量），排头上限 12G：judge_host 照余量放得下，公式（配置的总上限 − 余量 − 在跑的上限之和）只剩 8G 放不下，派活判它不能起，
    /health 一直回 ok；换成 200G 的余量（公式剩 108G）、排队线程照判不起（TASK_SERVER_SELFTEST_PLACEMENT_PAUSED），派活判它能起却没起成，/health 照报「派活判它此刻放得下」；
    放开之后都跑完。弄坏开关 blocked-head-counted-as-starved 下前一半也报，starved-head-unreported 下后一半不报，这一格红。"""
    blocker_key, head_key = "selftest-formula-blocker", "selftest-formula-head"
    world.seed_peak(blocker_key, FORMULA_HEAD_BLOCKER_PEAK_GIBIBYTES * GIBIBYTE)
    world.seed_peak(head_key, FORMULA_HEAD_PEAK_GIBIBYTES * GIBIBYTE)
    world.resources(local=(100, 100))
    paused = os.path.join(world.root, "placement-paused")
    world.start_server(extra={"TASK_SERVER_SELFTEST_PLACEMENT_PAUSED": paused})
    release = os.path.join(world.root, "release-formula-blocker")
    blocker = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "formula-blocker", release], extra={lib.KEY_VARIABLE: blocker_key})
    problems = []
    head = None
    try:
        attempt = first_attempt(world.running(blocker))
        cap, label = int(attempt.get("memory_cap") or 0), attempt.get("label")
        world.resources(local=(100, 100), local_scopes={label: cap} if label else None)
        head = world.submit(["bash", "-c", "exit 0", "formula-head"], extra={lib.KEY_VARIABLE: head_key})
        waiting = world.wait_for(head, lambda found: "在跑的上限之和" in (found.get("waiting_for") or "") or found["task"].get("state") != "queued",
                                 "照公式报放不下")
        blocked_health = health_until(world, lambda found: not found.get("ok"), FORMULA_HEAD_WATCH_SECONDS)
        if cap != int(FORMULA_HEAD_BLOCKER_PEAK_GIBIBYTES * lib.CAP_GROWTH_FACTOR) * GIBIBYTE or not label or waiting["task"].get("state") != "queued" or "在跑的上限之和" not in (waiting.get("waiting_for") or ""):
            problems.append(f"先占着的那一条应当上限 90G、有 label，排头照公式排着、排队原因写「在跑的上限之和」；实际上限 {cap} 字节、label {label!r}、"
                            f"排头 {describe(waiting)}、在等「{waiting.get('waiting_for')}」")
        if not blocked_health.get("ok"):
            problems.append(f"公式放不下（在跑的上限之和占满）时排头排着是正常的，/health 应当一直回 ok，实际 {blocked_health}")
        open(paused, "w").close()
        time.sleep(PAUSE_SETTLE_SECONDS)
        world.resources(local=(FORMULA_HEAD_ROOMY_GIBIBYTES, FORMULA_HEAD_ROOMY_GIBIBYTES), local_scopes={label: cap} if label else None)
        starved_health = health_until(world, lambda found: not found.get("ok"), 10)
        if starved_health.get("ok") or STARVED_HEALTH_TEXT not in (starved_health.get("problem") or ""):
            problems.append(f"换成 {FORMULA_HEAD_ROOMY_GIBIBYTES}G 的余量、公式放得下而排队线程判能起不起：/health 应当回不健康、问题写「{STARVED_HEALTH_TEXT}」，实际 {starved_health}")
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(paused)
        open(release, "w").close()
    finished = [world.finished(task_id) for task_id in (blocker, head) if task_id is not None]
    failed = [describe(found) for found in finished if found["task"].get("outcome") != "succeeded"]
    if failed:
        problems.append(f"放开之后都应当跑完，实际没成的 {failed}")
    if problems:
        raise SelftestFailure("判排头放不放得下照公式：" + "；".join(problems), world.server_log_lines())
    return (f"在跑的上限之和占满（公式剩 8G、排头要记 12G）时 /health 一直回 ok；公式放得下而没起时报「{starved_health.get('problem')}」；放开之后都跑完")


def cell_retry_goes_where_it_fits(world):
    world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-retry-anywhere", 8 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,run", "fake-peer-exits=250,run"],
                           extra={lib.KEY_VARIABLE: "selftest-retry-anywhere"}, cwd=world.repo)
    summary = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    attempts = summary["attempts"]
    if summary["task"].get("next_host") or len(attempts) < 2 or "钉在同一台" in (summary["task"].get("note") or ""):
        raise SelftestFailure(f"撞上限重排不钉台：第二次照剩余占比挑台、任务不记 next_host、说明不写「钉在同一台」，实际 {describe(summary)}、note「{summary['task'].get('note')}」",
                              world.server_log_lines())
    return f"撞上限重排不钉台：第 1 次 {attempts[0].get('host')}、第 2 次 {attempts[1].get('host')}，任务不记 next_host"


def cell_peer_runs_in_shared_tree(world):
    """第二台在同一棵共享的树里跑（task_server.py 文件头 ③；records/2026-10-31-task-server巡检.md 问题 74）。本机放不下 4G、第二台放得下：
    派第二台的那一次，假 multi-host-run.sh 记下的参数里没有 --send，它在任务的工作目录（同一个绝对路径）里起了命令，交给命令的 CARGO_TARGET_DIR 在假第二台的本地目录下
    （不在假共享根下），env 那一段不带按台取值的 ROCKSDB_LIB_DIR，尝试的环境里没有树的句柄；改主工作区的工具（hook-swap.py）派得到第二台
    （共享根外的路径提交时就拒，由 outside-shared-path-refused 判）。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server()
    problems = []
    seen = os.path.join(world.repo, "seen-on-peer")
    shared = os.path.realpath(world.shared_root)
    run_id = world.submit(["bash", "-c", 'pwd -P > "$1"; printf "%s\\n" "${CARGO_TARGET_DIR:-没设}" >> "$1"', "show", seen, "fake-peer-run"],
                          extra={"ROCKSDB_LIB_DIR": os.path.join(world.root, "rocksdb", "lib")}, cwd=world.repo)
    summary = world.finished(run_id, timeout=PLACEMENT_WAIT_SECONDS)
    attempt = (summary["attempts"] or [{}])[-1]
    try:
        with open(seen, encoding="utf-8") as handle:
            where, target = (handle.read().splitlines() + ["", ""])[:2]
    except OSError:
        where, target = "（命令没写出来）", ""
    arguments = next((line for line in world.calls() if line.startswith(f"peer-args task={run_id} ")), "")
    words = peer_env_words(world, run_id)
    try:
        with open(attempt.get("spec") or "", encoding="utf-8") as handle:
            handle_names = [name for name in json.load(handle).get("environment", {}) if "PEER_TREE" in name]
    except (OSError, ValueError):
        handle_names = ["（尝试的说明读不到）"]
    if attempt.get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
        problems.append(f"本机放不下、第二台放得下：应当在第二台跑完，实际 {describe(summary)}、假第二台收到「{arguments[:300]}」")
    if "--send" in arguments.split() or "--fetch" in " ".join(arguments.split()) or handle_names:
        problems.append(f"派第二台不该拷树：参数里不该有 --send、--fetch 这一类，尝试的环境里不该有树的句柄，实际参数「{arguments[:300]}」、句柄变量 {handle_names}")
    if where != os.path.realpath(world.repo):
        problems.append(f"第二台那条命令应当在同一个绝对路径 {os.path.realpath(world.repo)} 里起，实际在「{where}」")
    if not target.startswith(world.peer_local + os.sep) or target == shared or target.startswith(shared + os.sep):
        problems.append(f"第二台那条命令拿到的 CARGO_TARGET_DIR 应当在第二台本地这一趟的目录下（不在共享根 {shared} 下），实际「{target}」")
    if "ROCKSDB_LIB_DIR=" in words:
        problems.append(f"按台取值的预编库变量不该带到第二台（交给 multi-host-run.sh 的 env 词里不该有 ROCKSDB_LIB_DIR=），实际「{words[:300]}」")
    writer_id = world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/hook-swap.py", "fake-peer-run"], cwd=world.repo)
    try:
        writer = world.finished(writer_id, timeout=PLACEMENT_WAIT_SECONDS)
        if (writer["attempts"] or [{}])[-1].get("host") != "peer" or writer["task"].get("outcome") != "succeeded":
            problems.append(f"改主工作区的工具（hook-swap.py）应当派得到第二台跑完（改的是同一棵共享的树），实际 {describe(writer)}")
    except SelftestFailure as failure:
        problems.append(f"改主工作区的工具（hook-swap.py）应当派得到第二台跑完，实际等不到结局（{failure}）")
        world.client(["cancel", str(writer_id)])
    if problems:
        raise SelftestFailure("第二台在共享的树里跑：" + "；".join(problems), world.server_log_lines())
    return "派第二台的在同一个绝对路径里起、不带 --send 与树的句柄、编译目录在第二台本地、不带按台取值的变量；改主工作区的工具派得到第二台"


def cell_clean_failure_explained(world):
    """task clean 没删成时（编译目录里有进程开着文件）退 5，逐份写明是哪个编译目录、为什么（remove_run_build_directory 交回的原因），操作日志同样写原因；
    停了那个进程再 clean 退 0、删了。开着文件的是这一格自己起的 sleep（自己的进程组），收尾只停它自己的进程号。
    提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1：编译目录留到 task clean（没设的记下结局就删，没有 clean 要删的）。"""
    world.start_server()
    path = f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"
    task_id = world.submit(["cargo", "build"], extra=dict(KEEP_TEMPORARY_EXTRA, PATH=path))
    summary = world.finished(task_id)
    build = (summary["attempts"] or [{}])[0].get("build_directory") or ""
    if summary["task"].get("outcome") != "succeeded" or not build or not os.path.isdir(build):
        raise SelftestFailure(f"假 cargo 应当跑完、留下这一次尝试的编译目录（设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1），实际 {describe(summary)}、编译目录「{build}」",
                              world.server_log_lines())
    held_file = os.path.join(build, "held-by-selftest")
    problems = []
    with open(held_file, "w", encoding="utf-8") as handle:
        holder = subprocess.Popen(["sleep", "60"], stdout=handle, stdin=subprocess.DEVNULL, start_new_session=True)
    try:
        for _ in range(100):   # 等那个 sleep 真开着那份文件（它的 1 号 fd 指过去）
            if os.path.realpath(f"/proc/{holder.pid}/fd/1") == os.path.realpath(held_file):
                break
            time.sleep(POLL_SECONDS)
        stuck = world.client(["clean", str(task_id)])
        reason = "个进程开着里面的文件"   # remove_run_build_directory 交回的那一句
        noted = [operation.get("note", "") for operation in world.operations() if operation.get("action") == "clean" and operation.get("task") == task_id]
        if stuck.returncode != 5 or f"编译目录 {build}" not in stuck.stderr or reason not in stuck.stderr or not os.path.isdir(build):
            problems.append(f"编译目录里有进程开着文件时 task clean 应当退 5、写明没删成的是编译目录 {build}、为什么（{reason}），目录照留，"
                            f"实际退 {stuck.returncode}、目录{'在' if os.path.isdir(build) else '不在'}、stderr「{stuck.stderr.strip()[-400:]}」")
        if not noted or reason not in noted[-1]:
            problems.append(f"没删成的那一次操作日志应当写原因（{reason}），实际 {noted}")
    finally:
        holder.terminate()
        holder.wait(timeout=30)
    retried = world.client(["clean", str(task_id)])
    if retried.returncode != 0 or os.path.lexists(build):
        problems.append(f"停了那个进程再 clean 应当退 0、删了编译目录，实际退 {retried.returncode}、目录{'还在' if os.path.lexists(build) else '不在'}")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return "编译目录里有进程开着文件时 clean 退 5、写明哪个编译目录为什么、操作日志带原因；停了那个进程再 clean 删了"


def cell_cpu_unread_not_stalled(world):
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01"})   # 假 scope 是空目录：CPU 读不到
    summary = world.finished(world.submit(["bash", "-c", "echo started; sleep 6"]), timeout=40)
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"日志过了阈值不涨而 CPU 读不到的那一条应当照跑完（读不到不等于不涨），实际 {describe(summary)}", world.server_log_lines())
    return "日志过了阈值不涨、CPU 读不到的照跑完（读不到不等于不涨）"


def cell_stalled_attempt_stopped(world):
    scope = os.path.join(world.root, "idle-scope")   # CPU 读得到、一直不涨
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01",
                              "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    quiet = world.finished(world.submit(["bash", "-c", "echo started; sleep 30"]), timeout=40)
    chatty = world.finished(world.submit(["bash", "-c", "for i in 1 2 3 4 5 6; do echo tick $i; sleep 0.5; done"]), timeout=40)
    if quiet["task"].get("outcome") != "stalled" or chatty["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"日志 2 秒没涨的那一条应当被停、记 stalled，每半秒打一行的照跑完；实际不出声的 {describe(quiet)}、一直打的 {describe(chatty)}",
                              world.server_log_lines())
    return "日志满阈值没涨的那一条服务停掉、记 stalled；一直有输出的照跑完"


def cell_computing_quietly_not_stalled(world):
    outer, scope = os.path.join(world.root, "outer-scope"), os.path.join(world.root, "inner-scope")   # 外层不动，里层包装开的平级 scope 在涨
    for directory in (outer, scope):
        os.makedirs(directory)
        with open(os.path.join(directory, "cpu.stat"), "w", encoding="utf-8") as handle:
            handle.write("usage_usec 0\n")
    stat = os.path.join(scope, "cpu.stat")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": outer + os.pathsep + scope})
    stop = threading.Event()

    def burn():   # 假 scope 的 CPU 每 0.2 秒涨 0.2 秒（一个核在算）
        used = 0
        while not stop.is_set():
            used += 200000
            with open(stat + ".partial", "w", encoding="utf-8") as handle:
                handle.write(f"usage_usec {used}\n")
            os.replace(stat + ".partial", stat)
            time.sleep(0.2)

    burner = threading.Thread(target=burn, daemon=True)
    burner.start()
    try:
        summary = world.finished(world.submit(["bash", "-c", "echo started; sleep 7"]), timeout=40)
    finally:
        stop.set()
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"日志 7 秒不涨（阈值 2 秒）而 CPU 一直在涨：在算，应当照跑完，实际 {describe(summary)}", world.server_log_lines())
    return "日志过了阈值不涨而 CPU 一直在涨的照跑完（在算不算卡住）"


OVERRUN_COMMAND = ["bash", "-c", "echo started; sleep 30"]   # overrun-quiet-log-stopped 记的用时与交的命令逐词相同


def cell_overrun_quiet_log_stopped(world):
    key = "selftest-overrun"
    world.seed_duration(key, 1, OVERRUN_COMMAND)   # 记录用时 1 秒：跑过 5 秒就算远超
    scope = os.path.join(world.root, "burning-scope")
    os.makedirs(scope)
    stat = os.path.join(scope, "cpu.stat")
    with open(stat, "w", encoding="utf-8") as handle:
        handle.write("usage_usec 0\n")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    stop = threading.Event()

    def burn():   # 演占着 CPU 的死循环：假 scope 的 CPU 每 0.2 秒涨 0.2 秒
        used = 0
        while not stop.is_set():
            used += 200000
            with open(stat + ".partial", "w", encoding="utf-8") as handle:
                handle.write(f"usage_usec {used}\n")
            os.replace(stat + ".partial", stat)
            time.sleep(0.2)

    burner = threading.Thread(target=burn, daemon=True)
    burner.start()
    try:
        summary = world.finished(world.submit(OVERRUN_COMMAND, extra={lib.KEY_VARIABLE: key}), timeout=45)
    finally:
        stop.set()
    if summary["task"].get("outcome") != "stalled":
        raise SelftestFailure(f"记录用时 1 秒、跑过 5 倍而日志满 2 秒没涨：CPU 在涨也应当停掉、记 stalled，实际 {describe(summary)}", world.server_log_lines())
    return "跑过记录用时 5 倍、日志满阈值没涨的，CPU 在涨也停掉、记 stalled"


def cell_peer_stall_judged_by_peer_cpu(world):
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-peer-stall", 10 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01"})
    busy = 'sleep 1; for i in $(seq 14); do echo "MULTI_HOST_RUN_PEER_CPU poll=$i usage_usec=$((i*1000000)) read=yes"; sleep 0.5; done'   # 头 1 秒还没有 CPU 行（演第二台在拷树；短于阈值 2 秒）
    idle = 'for i in $(seq 14); do echo "MULTI_HOST_RUN_PEER_CPU poll=$i usage_usec=1000 read=yes"; echo "MULTI_HOST_RUN_STEP step=peer-command event=poll"; sleep 0.5; done'
    summaries = {}
    for name, script in (("busy", busy), ("idle", idle)):
        summaries[name] = world.finished(world.submit(["bash", "-c", script, name, "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-peer-stall"}, cwd=world.repo), timeout=60)
    hosts = {name: (summary["attempts"][0].get("host") if summary["attempts"] else None) for name, summary in summaries.items()}
    if hosts != {"busy": "peer", "idle": "peer"} or summaries["busy"]["task"].get("outcome") != "succeeded" or summaries["idle"]["task"].get("outcome") != "stalled":
        raise SelftestFailure(f"第二台的两条：只打 CPU 行且 CPU 在涨的应当照跑完，只打 CPU 不涨的行与 STEP 行的应当记 stalled；"
                              f"实际台 {hosts}、在涨的 {describe(summaries['busy'])}、不涨的 {describe(summaries['idle'])}", world.server_log_lines())
    return "第二台的尝试：MULTI_HOST_RUN_PEER_CPU 与 STEP 行不算日志在涨，CPU 在涨的照跑完、不涨的记 stalled"


def cell_stall_waits_for_grace(world):
    """起来后宽限里不判卡住：日志与 CPU 都不涨的那一条，服务在宽限里判过、日志已静了两个卡住阈值那么久时才放行它，它照跑完。
    放行看服务记的「宽限里判过一次」（elapsed 是从起尝试算的秒数），不看挂钟；宽限只要比放行那一下长就够。"""
    scope = os.path.join(world.root, "idle-scope")   # CPU 读得到、一直不涨：没有宽限就会被判卡住
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": str(GRACE_CELL_STALL_SECONDS), "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": str(GRACE_CELL_GRACE_SECONDS),
                              "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    release = os.path.join(world.root, "release")
    task_id = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "quiet", release])
    quiet_for = 2 * GRACE_CELL_STALL_SECONDS

    def judged_quiet_in_grace():
        for line in world.scheduler_reads():
            found = re.match(rf"^stall-grace task={task_id} attempt=1 elapsed=([0-9.]+)$", line)
            if found and float(found.group(1)) >= quiet_for:
                return True
        return False

    try:
        world.wait_for(task_id, lambda found: found["task"].get("state") == "finished" or judged_quiet_in_grace(), f"在宽限里判到静了 {quiet_for} 秒")
    finally:
        open(release, "w").close()
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"起来后宽限 {GRACE_CELL_GRACE_SECONDS} 秒不判：日志与 CPU 静了 {quiet_for} 秒（卡住阈值 {GRACE_CELL_STALL_SECONDS} 秒的两倍）还在宽限里的那一条应当照跑完，"
                              f"实际 {describe(summary)}", world.server_log_lines())
    return f"起来后宽限里日志与 CPU 静了卡住阈值两倍那么久的照跑完（宽限过了才量窗口）"


def cell_running_status_shows_progress(world):
    key = "selftest-progress"
    release = os.path.join(world.root, "release")
    command = ["bash", "-c", BLOCK_SCRIPT, "block", release]
    world.seed_duration(key, 3, command)
    world.start_server()
    task_id = world.submit(command, extra={lib.KEY_VARIABLE: key})
    world.running(task_id)
    time.sleep(1)
    shown = world.client(["status", str(task_id)])
    open(release, "w").close()
    world.finished(task_id)
    if "在跑：已跑" not in shown.stdout or "记录用时 3 秒" not in shown.stdout or "没涨" not in shown.stdout:
        raise SelftestFailure(f"在跑的任务 status 应当报已跑多久、记录用时、日志多久没涨、在用核数，实际「{shown.stdout.strip()[:400]}」")
    return "在跑的任务 status 报已跑多久、记录用时 3 秒、日志多久没涨与在用核数"


SIMULATED_WORKLOAD = (   # (类, 条数, 峰值, 用时记录秒, 中的与轻的真跑几轮×0.25 秒；重的占着到放行为止)：一台 50G 装不下三条 20G，看两台怎么分、轻的等不等重的
    ("heavy", 3, 20 * GIBIBYTE, 6, 24), ("medium", 4, 8 * GIBIBYTE, 3, 12), ("light", 10, 300 * MEBIBYTE, 1, 2))
DUTY_SPREAD_SCRIPT = "for i in $(seq 8); do echo spread $i; sleep 0.25; done"   # three-duties-two-sessions 分到两台的那 6 条（记的用时与交的命令逐词相同）


def simulated_command(kind, rounds):
    """simulated-two-host-workload 里一类交的命令（记的用时与交的逐词相同）：重的本机在仓根里等放行文件，第二台那一条由假第二台照 fake-peer-wait 在假仓根里等它。"""
    if kind == "heavy":
        return ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, kind, SIMULATED_RELEASE, f"fake-peer-wait={SIMULATED_RELEASE}"]
    return ["bash", "-c", f"for i in $(seq {rounds}); do echo {kind} $i; sleep 0.25; done", kind, "fake-peer-run"]


def cell_simulated_two_host_workload(world):
    """两台各 50G：重 3 条（各记 30G 的上限）、中 4 条、轻 10 条一齐放出来。重的占着到放行为止：两条起、第三条等，中的与轻的在它等着的时候起完、跑完。
    判定只比尝试的记录号（起的先后）与起止时刻的交叠，不比秒数（机器忙闲只改快慢，不改先后）。"""
    world.resources(local=(50, 50), peer=(50, 50), peer_cap_gibibytes=50, hold=True)
    for kind, _, peak, seconds, rounds in SIMULATED_WORKLOAD:
        world.seed_peak(f"selftest-sim-{kind}", peak)
        world.seed_duration(f"selftest-sim-{kind}", seconds, simulated_command(kind, rounds))
    world.start_server()
    submitted = []
    plan = [(kind, rounds) for kind, count, _, _, rounds in SIMULATED_WORKLOAD for _ in range(count)]
    order = plan[0::3] + plan[1::3] + plan[2::3]   # 轻重交错着交
    for kind, rounds in order:
        submitted.append((world.submit(simulated_command(kind, rounds), extra={lib.KEY_VARIABLE: f"selftest-sim-{kind}"}, cwd=world.repo), kind))
    world.resources(local=(50, 50), peer=(50, 50), peer_cap_gibibytes=50)   # 一齐放出来
    world.api("POST", "/wake")
    problems = []
    try:
        quick = [(kind, world.finished(task_id, timeout=120)) for task_id, kind in submitted if kind != "heavy"]
        heavy_started = sorted(task_id for task_id, kind in submitted if kind == "heavy" and world.summary(task_id)["attempts"])
    finally:
        open(os.path.join(world.repo, SIMULATED_RELEASE), "w").close()
    heavy = [(kind, world.finished(task_id, timeout=120)) for task_id, kind in submitted if kind == "heavy"]
    summaries = quick + heavy
    if len(heavy_started) != 2:
        problems.append(f"三条重的各记 30G：一台 50G 只放得下一条，中的与轻的跑完时应当恰好两条起了，实际起了 {heavy_started}")
    failed = [f"{kind} {describe(summary)}" for kind, summary in summaries if summary["task"].get("outcome") != "succeeded" or not summary["attempts"]]
    if failed:
        problems.append(f"都应当跑完，实际没成的 {failed}")
    peaks = {kind: peak for kind, _, peak, _, _ in SIMULATED_WORKLOAD}
    intervals, kinds_on = {"local": [], "peer": []}, {"local": set(), "peer": set()}
    for kind, summary in summaries:
        if not summary["attempts"]:
            continue
        last = summary["attempts"][-1]
        intervals.setdefault(last.get("host"), []).append((float(last.get("started_at") or 0), float(last.get("ended_at") or 0), peaks[kind]))
        kinds_on.setdefault(last.get("host"), set()).add(kind)
    for host, spans in intervals.items():
        for start, _, _ in spans:
            booked = sum(peak for begin, end, peak in spans if begin <= start < end)
            if booked > 50 * GIBIBYTE:
                problems.append(f"{host} 在 {start:.1f} 时同时跑着峰值合计 {booked / GIBIBYTE:.0f}G 的任务，超过总上限 50G")
                break
    if not all(kinds_on.get(host, set()) >= {"heavy", "light"} for host in ("local", "peer")):
        problems.append(f"两台都应当分到重的与轻的，实际 {({host: sorted(kinds) for host, kinds in kinds_on.items()})}")
    counts = {host: len(spans) for host, spans in intervals.items()}
    if min(counts.get("local", 0), counts.get("peer", 0)) * 4 < len(summaries):
        problems.append(f"两台分到的条数都应当不少于四分之一，实际 {counts}")
    waited = [summary for kind, summary in heavy if summary["task"]["id"] not in heavy_started]
    waited_order = attempt_order(waited[0]) if waited else None
    late_lights = [summary["task"]["id"] for kind, summary in quick if kind == "light" and (waited_order is None or (attempt_order(summary) or waited_order) > waited_order)]
    if late_lights:
        problems.append(f"轻任务不该排在等着的那条重任务（尝试记录号 {waited_order}）后面等它起，实际排在它后面的 {late_lights}")
    if problems:
        detail = [f"任务 {summary['task']['id']} {kind} {first_attempt(summary).get('host', '-')} 记录号 {first_attempt(summary).get('id', '-')}" for kind, summary in summaries]
        raise SelftestFailure("模拟负载：" + "；".join(problems), detail + world.server_log_lines())
    return (f"模拟负载 {len(summaries)} 条（重 3、中 4、轻 10，两台各 50G）全跑完：条数 {counts}，第三条重的等着时中的与轻的都起完、跑完（它的记录号 {waited_order}），"
            f"每台同时跑着的峰值合计都不超过 50G")


def cell_three_duties_two_sessions(world):
    """服务的三件职责放进同一个模拟世界：① 两台分任务 ② 没满时限的排头放不下时后面放得下的照起，排头空出来之后照起 ③ 跑不了的打回提交它的那个会话、别的会话看不见。"""
    world.seed_peak("selftest-duty-spread", 8 * GIBIBYTE)
    world.seed_duration("selftest-duty-spread", 2, ["bash", "-c", DUTY_SPREAD_SCRIPT, "spread", "fake-peer-run"])
    world.seed_peak("selftest-duty-blocker", 16 * GIBIBYTE)
    world.seed_peak("selftest-duty-head", 20 * GIBIBYTE)
    world.seed_peak("selftest-duty-small", 100 * MEBIBYTE)
    world.seed_peak("selftest-duty-never-fits", 40 * GIBIBYTE)
    world.resources(local=(30, 30), peer=(30, 30), peer_cap_gibibytes=30)
    world.start_server()
    problems = []
    # ① 会话 A 交 6 条 8G：一台 30G 装不下，两台都要用上
    spread = [world.submit(["bash", "-c", DUTY_SPREAD_SCRIPT, "spread", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-duty-spread"}, cwd=world.repo) for _ in range(6)]
    hosts = [(world.finished(task_id, timeout=120)["attempts"] or [{}])[-1].get("host") for task_id in spread]
    if not {"local", "peer"} <= set(hosts):
        problems.append(f"① 6 条 8G 应当两台都分到，实际落在 {hosts}")
    # ② 第二台只剩 10G、本机被 16G 占着：会话 B 的 20G 排头（没满时限）放不下，会话 A 后交的小任务照起；放出来之后排头照起
    world.resources(local=(30, 30), peer=(10, 30), peer_cap_gibibytes=30)
    release = os.path.join(world.root, "release")
    blocker = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], session=SESSION_B, extra={lib.KEY_VARIABLE: "selftest-duty-blocker"})
    world.running(blocker)
    head = world.submit(["bash", "-c", "exit 0"], session=SESSION_B, extra={lib.KEY_VARIABLE: "selftest-duty-head"}, cwd=world.repo)
    small = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: "selftest-duty-small"}, cwd=world.repo)
    try:
        small_summary = world.finished(small, timeout=PLACEMENT_WAIT_SECONDS)
    except SelftestFailure as failure:
        small_summary = None
        problems.append(f"② 排头没满时限而放不下时后面放得下的小的应当照起、跑完，实际 {failure}")
    head_state = world.summary(head)["task"].get("state")
    open(release, "w").close()
    if small_summary is not None and (small_summary["task"].get("outcome") != "succeeded" or head_state != "queued"):
        problems.append(f"② 小的应当跑完、放不下的排头照排着，实际小的 {describe(small_summary)}、排头 {head_state}")
    head_summary = world.finished(head)
    head_start = float((head_summary["attempts"] or [{}])[0].get("started_at") or 0)
    small_start = float(((small_summary or {}).get("attempts") or [{}])[0].get("started_at") or 0)
    if head_summary["task"].get("outcome") != "succeeded" or not head_start:
        problems.append(f"② 本机放出来之后排头应当起来跑完，实际 {describe(head_summary)}")
    # ③ 会话 A 交一条永远装不下的、会话 B 交一条命令自己失败的：各自的 wait 拿到各自的类型，status 只列自己的
    never = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: "selftest-duty-never-fits"})
    failing = world.submit(["bash", "-c", "echo started; sleep 1; exit 3"], session=SESSION_B)   # 跑到一半出错
    world.finished(never), world.finished(failing)
    for session, own, other in ((SESSION_A, never, failing), (SESSION_B, failing, never)):   # wait 收下之前：各自只看得见自己的
        listed = {item.get("id") for item in json.loads(world.client(["status", "--json"], session=session).stdout or "{}").get("tasks", [])}
        if own not in listed or other in listed:
            problems.append(f"③ {session} 的 status 应当列自己的任务 {own}、不列别的会话的 {other}，实际列了 {sorted(listed)}")
    for task_id, session, outcome, code in ((never, SESSION_A, "never-fits", None), (failing, SESSION_B, "command-failed", 3)):
        waited = world.client(["wait", str(task_id)], session=session)
        if f"type={outcome}" not in waited.stdout + waited.stderr or (code is not None and waited.returncode != code) or waited.returncode == 0:
            problems.append(f"③ 任务 {task_id} 的 wait 应当退非 0{('、退 ' + str(code)) if code else ''}、末行 type={outcome}，实际退 {waited.returncode}、"
                            f"「{(waited.stdout + waited.stderr).strip()[-200:]}」")
    if problems:
        raise SelftestFailure("三件职责：" + "；".join(problems), world.server_log_lines())
    return (f"① 6 条 8G 分在 {sorted(set(hosts))}；② 排头放不下时小的先起（{small_start:.2f}）、本机放出来之后排头起（{head_start:.2f}）；"
            f"③ 装不下的打回会话 A、命令失败的打回会话 B，各自 wait 拿到类型、status 互不可见")


def cell_hundred_five_minute_tasks_balanced(world):
    """100 条记录用时 5 分钟的任务一齐放出来、占着到放行为止（两台各 400G，100 条都放得下）：起的时候没有一条收尾，挑台只看记账，
    两台各分四成到六成；判定不随机器忙闲变（不比秒数，也不靠哪条先跑完）。"""
    world.seed_peak("selftest-hundred", 2 * GIBIBYTE)
    command = ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "hundred", HUNDRED_RELEASE, f"fake-peer-wait={HUNDRED_RELEASE}"]
    world.seed_duration("selftest-hundred", 300, command)
    world.resources(local=(400, 400), peer=(400, 400), peer_cap_gibibytes=400, hold=True)
    world.start_server()
    tasks = [world.submit(command, extra={lib.KEY_VARIABLE: "selftest-hundred"}, cwd=world.repo) for _ in range(100)]
    world.resources(local=(400, 400), peer=(400, 400), peer_cap_gibibytes=400)
    world.api("POST", "/wake")
    try:
        hosts = [first_attempt(world.wait_for(task_id, lambda found: bool(found["attempts"]), "起来")).get("host") for task_id in tasks]
    finally:
        open(os.path.join(world.repo, HUNDRED_RELEASE), "w").close()
    outcomes = [world.finished(task_id, timeout=120)["task"].get("outcome") for task_id in tasks]
    counts = {host: hosts.count(host) for host in ("local", "peer")}
    if sum(counts.values()) != 100 or not all(40 <= count <= 60 for count in counts.values()) or set(outcomes) != {"succeeded"}:
        raise SelftestFailure(f"100 条 5 分钟的任务两台应当各分四成到六成、都跑完，实际 {counts}（落点 {hosts}）、结局 {sorted(set(outcomes))}", world.server_log_lines())
    return f"100 条 5 分钟的任务两台分到 {counts}"


def backdate_past_aged(world, task_ids):
    """服务停着时把这几条的提交时刻往前拨过服务真的时限（AGED_WAIT_SECONDS）：只有它们满时限，之后照真时刻交的离满时限还远，判定不看交得快不快。"""
    world.stop_server()
    for task_id in task_ids:
        world.rewrite_signed_task(task_id, lambda record: record.update(submitted_at=record["submitted_at"] - SERVER.AGED_WAIT_SECONDS - AGED_BACKDATE_MARGIN_SECONDS))


def first_attempt(summary):
    return (summary["attempts"] or [{}])[0]


def attempt_order(summary, number=1):
    """第 number 次尝试的记录号（服务起尝试时按先后编的号：同一个库里比得出谁先起，与机器忙闲、交得快慢无关）；那一次没起交 None。"""
    attempts = summary["attempts"]
    return attempts[number - 1].get("id") if len(attempts) >= number else None


def cell_stalled_at_cap_retried(world):
    """判卡住那一刻 scope 的内存峰值已到上限（不低于 STALL_AT_CAP_FRACTION）：不记 stalled，记 memory-cap-hit、乘 1.5 重排，结局说明写「顶在上限上卡住」；
    峰值远低于上限的照旧记 stalled。假 scope 的进程内存（memory.stat 的 anon）与 memory.peak 钉在第一条的上限上，CPU 一直不涨。"""
    scope = os.path.join(world.root, "at-cap-scope")
    os.makedirs(scope)
    for name, text in (("cpu.stat", "usage_usec 1000\n"), ("memory.peak", f"{AT_CAP_PEAK_BYTES}\n"), ("memory.stat", process_memory_stat(AT_CAP_PEAK_BYTES))):
        with open(os.path.join(scope, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    world.seed_peak("selftest-at-cap", AT_CAP_SEED_PEAK_BYTES)
    world.seed_peak("selftest-far-below-cap", 10 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    # 第 1 次不出声地睡（顶在上限上卡住）；重排之后那一次每半秒打一行、跑完
    script = 'if [[ "$TASK_SERVER_ATTEMPT" == 1 ]]; then echo started; sleep 30; fi; for i in 1 2 3 4 5 6; do echo tick $i; sleep 0.5; done'
    at_cap = world.finished(world.submit(["bash", "-c", script, "at-cap"], extra={lib.KEY_VARIABLE: "selftest-at-cap"}), timeout=60)
    far = world.finished(world.submit(["bash", "-c", "echo started; sleep 30", "far-below-cap"], extra={lib.KEY_VARIABLE: "selftest-far-below-cap"}), timeout=60)
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in at_cap["attempts"]]
    note = (at_cap["attempts"] or [{}])[0].get("stall_note") or ""
    problems = []
    if seen != [(AT_CAP_PINNED_TEXT, "memory-cap-hit"), (AT_CAP_GROWN_PINNED_TEXT, "succeeded")] or at_cap["task"].get("outcome") != "succeeded":
        problems.append(f"峰值顶在上限 {AT_CAP_PINNED_TEXT} 上卡住的应当记 memory-cap-hit、乘 1.5 到 {AT_CAP_GROWN_PINNED_TEXT} 重排、跑完，实际各次 {seen}、{describe(at_cap)}")
    if "顶在上限上卡住" not in note:
        problems.append(f"那一次的说明应当写「顶在上限上卡住」，实际「{note}」")
    if far["task"].get("outcome") != "stalled":
        problems.append(f"峰值远低于上限的应当照旧记 stalled，实际 {describe(far)}")
    if problems:
        raise SelftestFailure("顶在上限上卡住：" + "；".join(problems), world.server_log_lines())
    return f"顶在上限上卡住的记 memory-cap-hit、{AT_CAP_PINNED_TEXT} 乘 1.5 到 {AT_CAP_GROWN_PINNED_TEXT} 重排跑完（「{note[:60]}」）；峰值远低于上限的照旧记 stalled"


def start_burning_scope(world, name, files, usec_per_tick=200000):
    """一份假 scope（attempt_runner.py 文件头「自证换 scope」）：files 里的几份（{文件名: 数或正文}，memory.current、memory.peak、memory.stat 这类）照写、末尾补一个换行，
    cpu.stat 的 usage_usec 每 0.2 秒涨 usec_per_tick 微秒（缺省 0.2 秒：一个核在算），先写临时文件再改名（采样读不到写了一半的）。交 (目录, 停的 Event)：格收尾时设上它。"""
    scope = os.path.join(world.root, name)
    os.makedirs(scope)
    for file_name, value in files.items():
        with open(os.path.join(scope, file_name), "w", encoding="utf-8") as handle:
            handle.write(f"{value}\n")
    stat = os.path.join(scope, "cpu.stat")
    with open(stat, "w", encoding="utf-8") as handle:
        handle.write("usage_usec 0\n")
    stop = threading.Event()

    def keep_burning():
        used = 0
        while not stop.is_set():
            used += usec_per_tick
            with open(stat + ".partial", "w", encoding="utf-8") as handle:
                handle.write(f"usage_usec {used}\n")
            os.replace(stat + ".partial", stat)
            time.sleep(0.2)

    threading.Thread(target=keep_burning, daemon=True).start()
    return scope, stop


def cell_busy_cpu_pinned_at_cap_retried(world):
    """日志满卡住阈值没涨、此刻内存贴着上限、CPU 一直在涨（任务 7723：在回收里打转、跑了 37775 秒没人停；task_server.py 的 judge_stall_stop）：CPU 在涨不算在算，
    停掉、记 memory-cap-hit、乘 1.5 重排、跑完，说明写「顶在上限上卡住」与此刻内存。库里记一次跑完的峰值 1024 MiB（上限 1536M），
    假 scope 的进程内存（memory.stat 的 anon）、memory.current 与 memory.peak 钉在 1536 MiB，CPU 每 0.2 秒涨 0.2 秒。
    弄坏开关 at-cap-busy-cpu-kept-running 下照改前当在算、第一次睡满 30 秒退 0，这一格红。"""
    scope, stop = start_burning_scope(world, "pinned-busy-scope", {"memory.current": AT_CAP_PEAK_BYTES, "memory.peak": AT_CAP_PEAK_BYTES,
                                                                   "memory.stat": process_memory_stat(AT_CAP_PEAK_BYTES).rstrip("\n")})
    world.seed_peak("selftest-pinned-busy", AT_CAP_SEED_PEAK_BYTES)
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
        # 第 1 次不出声地睡（贴着上限、CPU 在涨）；重排之后那一次每半秒打一行、跑完（上限 2304M，此刻内存只到它的三分之二）
        script = 'if [[ "$TASK_SERVER_ATTEMPT" == 1 ]]; then echo started; sleep 30; fi; for i in 1 2 3 4 5 6; do echo tick $i; sleep 0.5; done'
        summary = world.finished(world.submit(["bash", "-c", script, "pinned-busy"], extra={lib.KEY_VARIABLE: "selftest-pinned-busy"}), timeout=60)
    finally:
        stop.set()
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in summary["attempts"]]
    note = first_attempt(summary).get("stall_note") or ""
    problems = []
    if seen != [(AT_CAP_PINNED_TEXT, "memory-cap-hit"), (AT_CAP_GROWN_PINNED_TEXT, "succeeded")] or summary["task"].get("outcome") != "succeeded":
        problems.append(f"此刻内存贴着上限 {AT_CAP_PINNED_TEXT}、CPU 在涨的应当记 memory-cap-hit、乘 1.5 到 {AT_CAP_GROWN_PINNED_TEXT} 重排、跑完，实际各次 {seen}、{describe(summary)}")
    if "顶在上限上卡住" not in note or "此刻内存" not in note:   # CPU 那半句不判：窗口开头那一刻 cpu_usec 可能还没采到，读不到时照停、不写核数
        problems.append(f"那一次的说明应当写「顶在上限上卡住」与「此刻内存」，实际「{note}」")
    if problems:
        raise SelftestFailure("贴着上限而 CPU 在涨：" + "；".join(problems), world.server_log_lines())
    return f"贴着上限而 CPU 在涨的记 memory-cap-hit、{AT_CAP_PINNED_TEXT} 乘 1.5 到 {AT_CAP_GROWN_PINNED_TEXT} 重排跑完（「{note[:80]}」）"


def cell_busy_cpu_below_cap_keeps_running(world):
    """对照：日志满卡住阈值没涨、CPU 一直在涨、此刻内存离上限远（假 scope 的进程内存与 memory.current 是上限 1536M 的八分之一）：在算，照跑完，只一次尝试。
    弄坏开关 at-cap-fraction-ignored 下读得到此刻内存就当贴着上限、停掉重排，cpu-ignored-for-stall 下不看 CPU 照停，这一格红。"""
    scope, stop = start_burning_scope(world, "below-cap-busy-scope", {"memory.current": AT_CAP_PEAK_BYTES // 8, "memory.peak": AT_CAP_PEAK_BYTES // 8,
                                                                      "memory.stat": process_memory_stat(AT_CAP_PEAK_BYTES // 8).rstrip("\n")})
    world.seed_peak("selftest-below-cap-busy", AT_CAP_SEED_PEAK_BYTES)
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
        summary = world.finished(world.submit(["bash", "-c", "echo started; sleep 7", "below-cap-busy"], extra={lib.KEY_VARIABLE: "selftest-below-cap-busy"}), timeout=60)
    finally:
        stop.set()
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in summary["attempts"]]
    if seen != [(AT_CAP_PINNED_TEXT, "succeeded")] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"日志 7 秒不涨（阈值 2 秒）、CPU 在涨、此刻内存只到上限的八分之一：在算，应当照跑完、只一次尝试，实际各次 {seen}、{describe(summary)}",
                              world.server_log_lines())
    return f"此刻内存离上限远、CPU 在涨的照跑完（各次 {seen}）"


# page-cache-not-counted-as-process-memory：假 scope 的 memory.current 与 memory.peak 钉在上限 1536M（库里 1024M × 1.5），里面进程用的只有 anon 96M，
# 其余是页缓存与内核（任务 8853 的样子：anon 921M、file 22163M、kernel 650M，memory.peak 顶到上限 50176M）
PAGE_CACHE_ANON_BYTES = 96 * MEBIBYTE
PAGE_CACHE_KERNEL_BYTES = 32 * MEBIBYTE
PAGE_CACHE_FILE_BYTES = AT_CAP_PEAK_BYTES - PAGE_CACHE_ANON_BYTES - PAGE_CACHE_KERNEL_BYTES


def cell_page_cache_not_counted_as_process_memory(world):
    """页缓存撑满 scope 不算进程内存（attempt_runner.py 文件头 ③ ⑥、task_server.py 文件头 ①「顶在上限上卡住」）：假 scope 的 memory.stat 是 anon 96M、file 1408M、
    kernel 32M，memory.current 与 memory.peak 钉在上限 1536M，CPU 一直不涨。跑完的那一条记的峰值是进程用的 96M、不是 1536M；
    日志不涨的那一条（库里 1024M、上限 1536M）满卡住阈值照记 stalled、只一次尝试，不判顶在上限上卡住、不乘 1.5 重排。
    弄坏开关 memory-peak-with-page-cache 下照改前读 memory.peak 与 memory.current：记 1536M、判顶在上限上卡住重排，这一格红。"""
    scope = os.path.join(world.root, "page-cache-scope")
    os.makedirs(scope)
    for name, text in (("cpu.stat", "usage_usec 1000\n"), ("memory.peak", f"{AT_CAP_PEAK_BYTES}\n"), ("memory.current", f"{AT_CAP_PEAK_BYTES}\n"),
                       ("memory.stat", process_memory_stat(PAGE_CACHE_ANON_BYTES, PAGE_CACHE_FILE_BYTES, PAGE_CACHE_KERNEL_BYTES))):
        with open(os.path.join(scope, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    world.seed_peak("selftest-page-cache-quiet", AT_CAP_SEED_PEAK_BYTES)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    finished = world.finished(world.submit(["bash", "-c", "for i in 1 2 3; do echo tick $i; sleep 0.5; done", "page-cache-finished"],
                                           extra={lib.KEY_VARIABLE: "selftest-page-cache-finished"}), timeout=60)
    quiet = world.finished(world.submit(["bash", "-c", "echo started; sleep 30", "page-cache-quiet"], extra={lib.KEY_VARIABLE: "selftest-page-cache-quiet"}),
                           timeout=60)
    problems = []
    recorded = first_attempt(finished).get("memory_peak", "缺这一栏")
    if finished["task"].get("outcome") != "succeeded" or recorded != PAGE_CACHE_ANON_BYTES:
        problems.append(f"跑完的那一条应当记进程用的 {PAGE_CACHE_ANON_BYTES}（anon），不是 memory.peak 的 {AT_CAP_PEAK_BYTES}，实际 {describe(finished)}、记 {recorded}")
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in quiet["attempts"]]
    note = first_attempt(quiet).get("stall_note") or ""
    if seen != [(AT_CAP_PINNED_TEXT, "stalled")] or quiet["task"].get("outcome") != "stalled" or "顶在上限上卡住" in note:
        problems.append(f"日志不涨、页缓存撑满上限 {AT_CAP_PINNED_TEXT} 而进程只用 {PAGE_CACHE_ANON_BYTES} 的应当记 stalled、只一次尝试、说明不写「顶在上限上卡住」，"
                        f"实际各次 {seen}、{describe(quiet)}、说明「{note}」")
    if problems:
        raise SelftestFailure("页缓存不算进程内存：" + "；".join(problems), world.server_log_lines())
    return f"页缓存撑满上限 {AT_CAP_PINNED_TEXT}：跑完的记进程用的 {recorded}，日志不涨的记 stalled（「{note[:60]}」），不判顶在上限上卡住"


PEER_PINNED_SEED_PEAK_BYTES = 10 * GIBIBYTE   # peer-pinned-at-cap-busy-cpu-retried：上限 15G，本机只空 8G、派到第二台
PEER_PINNED_CURRENT_BYTES = 15 * GIBIBYTE     # 那一格第一次尝试每轮报的此刻内存：钉在上限上


def cell_peer_pinned_at_cap_busy_cpu_retried(world):
    """第二台同样判贴着上限（task_server.py 的 PeerMachine.memory_now 读 MULTI_HOST_RUN_PEER_MEMORY；用户：「所有任务必须不受限制地多机跑」）：
    派到第二台的那一条第一次只打 multi-host-run.sh 那一层每轮的两行——MULTI_HOST_RUN_PEER_CPU（在涨，一个核）与 MULTI_HOST_RUN_PEER_MEMORY（此刻内存钉在上限 15G 上）——
    这两行都不算日志在涨；满卡住阈值之后停掉、记 memory-cap-hit、乘 1.5 重排、跑完。弄坏开关 at-cap-busy-cpu-kept-running、peer-memory-line-unread 下第一次照跑满、退 0，这一格红。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-peer-pinned", PEER_PINNED_SEED_PEAK_BYTES)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01"})
    script = ('if [[ "$TASK_SERVER_ATTEMPT" == 1 ]]; then for i in $(seq 30); do echo "MULTI_HOST_RUN_PEER_CPU poll=$i usage_usec=$((i*500000)) read=yes"; '
              'echo "MULTI_HOST_RUN_PEER_MEMORY poll=$i current_bytes=$1 peak_bytes=$1 read=yes"; sleep 0.5; done; exit 0; fi; '
              'for i in 1 2 3 4 5 6; do echo tick $i; sleep 0.5; done')
    summary = world.finished(world.submit(["bash", "-c", script, "peer-pinned", str(PEER_PINNED_CURRENT_BYTES), "fake-peer-run"],
                                          extra={lib.KEY_VARIABLE: "selftest-peer-pinned"}, cwd=world.repo), timeout=60)
    attempts = summary["attempts"]
    seen = [(attempt.get("host"), attempt.get("outcome")) for attempt in attempts]
    caps = [int(attempt.get("memory_cap") or 0) for attempt in attempts]
    note = first_attempt(summary).get("stall_note") or ""
    problems = []
    if [outcome for _, outcome in seen] != ["memory-cap-hit", "succeeded"] or seen[0][0] != "peer" or summary["task"].get("outcome") != "succeeded":
        problems.append(f"第二台上此刻内存贴着上限、CPU 在涨的应当记 memory-cap-hit、重排跑完，实际各次（台、结局）{seen}、{describe(summary)}")
    elif caps[1] <= caps[0]:
        problems.append(f"重排那一次的上限应当比第一次大（乘 1.5），实际 {caps}")
    if "顶在上限上卡住" not in note or "此刻内存" not in note:
        problems.append(f"第一次的说明应当写「顶在上限上卡住」与第二台报的「此刻内存」，实际「{note}」")
    if problems:
        raise SelftestFailure("第二台贴着上限而 CPU 在涨：" + "；".join(problems), world.server_log_lines())
    return f"第二台报的此刻内存贴着上限、CPU 在涨：记 memory-cap-hit、上限 {caps} 重排跑完（「{note[:80]}」）"


def cell_aged_head_at_slice_total_starts(world):
    """满时限的排头要记满一整台（峰值 40G、乘 1.5 超过本机 slice 总上限 50G）、slice 自己挂着回收不掉的那一截（这台此刻的余量，假余量 2G）：上限与记账收到总上限减余量（48G），
    照样起得来、跑完（task_server.py 的 booked_at_cap）；记满总上限的那种写法永远放不下（slice 放得下的减了余量只剩 48G）。看排两轮之后起没起，不比秒数。"""
    world.resources(local=(50, 50), hold=True)
    world.seed_peak("selftest-head-at-total", 40 * GIBIBYTE)
    world.start_server()
    head = world.submit(["bash", "-c", "exit 0", "head-at-total"], extra={lib.KEY_VARIABLE: "selftest-head-at-total"})
    backdate_past_aged(world, [head])
    world.resources(local=(50, 50))
    world.start_server()
    world.next_pass()
    world.next_pass()
    summary = world.summary(head)
    if not summary["attempts"]:
        world.client(["cancel", str(head)])
        raise SelftestFailure(f"满时限的排头记满一整台、slice 自己挂着余量 {SELFTEST_MARGIN_GIBIBYTES}G：排两轮之后应当已起（上限收到 {AT_TOTAL_CAP_PINNED_TEXT}），"
                              f"实际还排着，排队原因「{summary.get('waiting_for')}」", world.server_log_lines())
    summary = world.finished(head)
    caps = [attempt.get("memory_cap_text") for attempt in summary["attempts"]]
    if caps != [AT_TOTAL_CAP_PINNED_TEXT] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"满时限的排头应当以上限 {AT_TOTAL_CAP_PINNED_TEXT}（总上限 50G 减这台的假余量 2G）起、跑完，实际上限 {caps}、{describe(summary)}", world.server_log_lines())
    return f"满时限的排头记满一整台、slice 自己挂着余量 {SELFTEST_MARGIN_GIBIBYTES}G：上限收到 {AT_TOTAL_CAP_PINNED_TEXT} 起来、跑完"

def cell_aged_task_judged_first(world):
    """本机 30G 只放得下一条 20G：先交的那条没满时限、后交的那条满时限，满时限的先判、先起，先交的那条等它跑完。"""
    world.resources(local=(30, 30), hold=True)
    world.seed_peak("selftest-aged-young", 20 * GIBIBYTE)
    world.seed_peak("selftest-aged-old", 20 * GIBIBYTE)
    world.start_server()
    young = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "block", "release-young"], extra={lib.KEY_VARIABLE: "selftest-aged-young"}, cwd=world.work)
    old = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "block", "release-old"], extra={lib.KEY_VARIABLE: "selftest-aged-old"}, cwd=world.work)
    backdate_past_aged(world, [old])
    world.resources(local=(30, 30))
    world.start_server()
    try:
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        started = []
        while patience.waiting() and not started:
            started = [task_id for task_id in (young, old) if world.summary(task_id)["attempts"]]
            time.sleep(POLL_SECONDS)
        world.next_pass()   # 先起的那一条之后再排过整轮：没满时限的那条也起了的话这里看得见
        world.next_pass()
        started = [task_id for task_id in (young, old) if world.summary(task_id)["attempts"]]
        shown = describe(world.summary(old))
    finally:
        for name in ("release-young", "release-old"):
            open(os.path.join(world.work, name), "w").close()
    for task_id in (young, old):
        world.finished(task_id)
    if started != [old]:
        raise SelftestFailure(f"本机只放得下一条：满时限的任务 {old}（后交）应当先起、没满时限的任务 {young}（先交）等着，实际先起的 {started}（满时限那一条 {shown}）",
                              world.server_log_lines())
    return f"满时限的任务 {old} 虽然后交、先判先起，先交的任务 {young} 等它跑完"


def cell_squeezed_at_total_is_cap_hit(world):
    world.resources(local=(55, 50))   # 总上限 50G（真机上取多机配置 SLICE_MEMORY_CAP_BY_HOST 里本机那一项）
    world.seed_peak("selftest-squeeze-at-total", 40 * GIBIBYTE)   # 40G × 1.5 收到总上限减这台的假余量 2G：上限就是 48G
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=254,254,254,254"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-squeeze-at-total"})
    summary = world.finished(task_id)
    caps = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in summary["attempts"]]
    if [outcome for _, outcome in caps] != ["memory-cap-hit"] or summary["task"].get("outcome") != "memory-cap-at-maximum" or summary["task"].get("exit_code") != 236:
        raise SelftestFailure(f"上限已到收的那个数（总上限 50G 减这台的假余量 2G）的那一条被 slice 那一层杀（254）：就是它自己撞了上限，应当记 memory-cap-hit、再乘 1.5 就过总上限、打回 memory-cap-at-maximum（退 236），"
                              f"实际各次 {caps}、{describe(summary)}", world.server_log_lines())
    return "上限已到总上限的那一条被 slice 那一层杀，记撞上限、打回 memory-cap-at-maximum，不原样重排"


def cell_peer_run_failed_retried(world):
    world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 40G 的只有第二台放得下
    world.seed_peak("selftest-peer-retry", 40 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=255,run"], extra={lib.KEY_VARIABLE: "selftest-peer-retry"}, cwd=world.repo)
    summary = world.finished(task_id)
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    if outcomes != ["peer-run-failed", "succeeded"] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"第二台头一次没跑成（ssh 被重置这类，退 255）应当重排、第二次跑成，实际各次 {outcomes}、{describe(summary)}", world.server_log_lines())
    return "第二台头一次没跑成记 peer-run-failed、同一上限重排，第二次跑成"


def cell_peer_starts_throttled(world):
    """本机放不下、10 条一齐派到第二台：每一轮排队至多在第二台起 PEER_STARTS_PER_PASS 条（数服务记的「哪一轮起的」，不比秒数）。"""
    world.resources(local=(1, 1), peer=(100, 100), peer_cap_gibibytes=64, hold=True)   # 本机放不下：10 条都去第二台
    world.seed_peak("selftest-peer-burst", 2 * GIBIBYTE)
    world.start_server()
    tasks = [world.submit(["bash", "-c", "sleep 0.5", "burst", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-peer-burst"}, cwd=world.repo) for _ in range(10)]
    world.resources(local=(1, 1), peer=(100, 100), peer_cap_gibibytes=64)   # 一齐放出来
    world.api("POST", "/wake")
    summaries = [world.finished(task_id, timeout=120) for task_id in tasks]
    starts = {}
    for line in world.scheduler_reads():
        found = re.match(r"^start task=(\d+) host=(\w+) view=(\d+)$", line)
        if found and int(found.group(1)) in tasks:
            starts.setdefault(int(found.group(3)), []).append((int(found.group(1)), found.group(2)))
    per_pass = {view: len(started) for view, started in sorted(starts.items())}
    hosts = {first_attempt(summary).get("host") for summary in summaries}
    if (hosts != {"peer"} or any(summary["task"].get("outcome") != "succeeded" for summary in summaries) or sum(per_pass.values()) != len(tasks)
            or max(per_pass.values(), default=0) > SERVER.PEER_STARTS_PER_PASS):
        raise SelftestFailure(f"10 条一齐派到第二台：每一轮至多起 {SERVER.PEER_STARTS_PER_PASS} 条、都跑完，实际落在 {hosts}、各轮起的条数 {per_pass}、结局 "
                              f"{[summary['task'].get('outcome') for summary in summaries]}", world.server_log_lines())
    return f"10 条一齐派到第二台：分 {len(per_pass)} 轮起（各轮 {list(per_pass.values())} 条），都跑完"


def cell_unmeasured_not_zero(world):
    key = "selftest-unmeasured"
    world.seed_peak(key, 2 * GIBIBYTE)
    world.start_server()
    caps, peaks = [], []
    for _ in range(RECORD_WINDOW_OVERRUN):
        summary = world.finished(world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key}))
        attempt = summary["attempts"][0] if summary["attempts"] else {}
        caps.append(attempt.get("memory_cap_text"))
        peaks.append(attempt.get("memory_peak", "缺这一栏"))
    if caps != ["3072M"] * RECORD_WINDOW_OVERRUN or peaks != [None] * RECORD_WINDOW_OVERRUN:
        raise SelftestFailure(f"短到量不到峰值的命令跑 {RECORD_WINDOW_OVERRUN} 次（比近几次的名额多）：每次都应当记「没量到」（null，不是 0），上限一直照记下的那一次 3072M，"
                              f"实际上限 {caps}、记的峰值 {peaks}")
    return f"量不到的 {RECORD_WINDOW_OVERRUN} 次都记 null、不占名额，上限一直 3072M"


def wrapper_cap_texts(world, task_id):
    """假内存包装收到的各次上限（调用记录里这一件的 wrapper 行的 cap=）。"""
    return [line.split(" cap=", 1)[1].split(" ", 1)[0] for line in world.calls_of(task_id) if " cap=" in line]


def cell_unfinished_peak_ignored(world):
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": world.write_fake_scope(UNFINISHED_PEAK_BYTES)})
    pinned_bytes = int(UNRECORDED_DEFAULT_PINNED_TEXT[:-1]) * MEBIBYTE
    problems = []
    for key, first_command, first_outcome in (("selftest-unfinished-command-failed", ["bash", "-c", "exit 1"], "command-failed"),
                                              ("selftest-unfinished-squeezed", ["bash", "-c", "exit 0", "fake-wrapper-exits=254"], "squeezed-out")):
        first = world.finished(world.submit(first_command, extra={lib.KEY_VARIABLE: key}))
        peaks = [attempt.get("memory_peak") for attempt in first["attempts"]]
        if first["task"].get("outcome") != first_outcome or not peaks or any(peak != UNFINISHED_PEAK_BYTES for peak in peaks):
            problems.append(f"{key}：头一件应当记 {first_outcome}、每次尝试都量到峰值 {UNFINISHED_PEAK_BYTES}，实际 {describe(first)}、峰值 {peaks}")
            continue
        second_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key})
        second = world.finished(second_id)
        attempt = second["attempts"][0] if second["attempts"] else {}
        wrapper_caps = wrapper_cap_texts(world, second_id)
        if (second["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != UNRECORDED_DEFAULT_PINNED_TEXT
                or attempt.get("memory_cap") != pinned_bytes or attempt.get("prediction") != lib.UNRECORDED_HISTORY
                or wrapper_caps[:1] != [UNRECORDED_DEFAULT_PINNED_TEXT]):
            problems.append(f"{key}：只有一件 {first_outcome}（峰值 {UNFINISHED_PEAK_BYTES}）之后再交同一条，上限应当照没记录取 "
                            f"{UNRECORDED_DEFAULT_PINNED_TEXT}、prediction 记 {lib.UNRECORDED_HISTORY}，不取那次峰值 × 1.5，"
                            f"实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、包装收到 {wrapper_caps}、{describe(second)}")
    if problems:
        raise SelftestFailure("没跑完的尝试的峰值进了定额度：" + "；".join(problems), world.server_log_lines())
    return f"提前退 1 与被总上限挤掉的尝试峰值都不进定额度：再交同一条，上限照没记录取 {UNRECORDED_DEFAULT_PINNED_TEXT}"


def cell_completed_after_cap_hit(world):
    # 跑完那次的峰值大于撞过的上限的一例（取它 × 1.5），与小于撞过的上限的一例（不低于撞过的上限 × 1.5）
    cases = (("selftest-completed-above-cap-hit", 2500 * MEBIBYTE, "3750M"), ("selftest-completed-below-cap-hit", 1000 * MEBIBYTE, CAP_HIT_GROWN_PINNED_TEXT))
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": world.write_fake_scope(cases[0][1])})
    problems = []
    for key, completed_peak, expected in cases:
        world.write_fake_scope(completed_peak)
        world.seed_peak(key, CAP_HIT_SEED_PEAK_BYTES)
        first = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,run"], extra={lib.KEY_VARIABLE: key}))
        seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in first["attempts"]]
        if seen != [(CAP_HIT_PINNED_TEXT, "memory-cap-hit"), (CAP_HIT_GROWN_PINNED_TEXT, "succeeded")]:
            problems.append(f"{key}：头一件应当 {CAP_HIT_PINNED_TEXT} 撞上限、{CAP_HIT_GROWN_PINNED_TEXT} 跑完，两次尝试都在记录里，实际 {seen}")
            continue
        second_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key})
        second = world.finished(second_id)
        cap = second["attempts"][0].get("memory_cap_text") if second["attempts"] else None
        if cap != expected or wrapper_cap_texts(world, second_id)[:1] != [expected]:
            problems.append(f"{key}：撞过 {CAP_HIT_PINNED_TEXT}、在 {CAP_HIT_GROWN_PINNED_TEXT} 跑完、跑完那次峰值 {completed_peak}，再交同一条上限应当是 {expected}"
                            f"（跑完那次的峰值 × 1.5，不低于 {CAP_HIT_GROWN_PINNED_TEXT}），实际 {cap}、包装收到 {wrapper_cap_texts(world, second_id)}")
    if problems:
        raise SelftestFailure("撞过上限后跑完的命令：" + "；".join(problems), world.server_log_lines())
    return f"撞过 {CAP_HIT_PINNED_TEXT} 后跑完：再交同一条，峰值 2500M 的取 3750M，峰值 1000M 的不低于 {CAP_HIT_GROWN_PINNED_TEXT}"


def cell_cap_hit_starts_above(world):
    key = "selftest-cap-hit-only"
    world.resources(local=(30, 2), local_margin=0)   # 本机 slice 总上限 2G、余量 0：2048M 撞了就到顶收尾，这条命令只留一次撞上限的尝试、一次跑完的都没有
    world.seed_peak(key, CAP_HIT_SEED_PEAK_BYTES)
    world.start_server()
    first = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250"], extra={lib.KEY_VARIABLE: key}))
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in first["attempts"]]
    if seen != [(CAP_HIT_PINNED_TEXT, "memory-cap-hit")] or first["task"].get("outcome") != "memory-cap-at-maximum":
        raise SelftestFailure(f"本机总上限 2G：头一件应当只有一次 {CAP_HIT_PINNED_TEXT} 撞上限、到顶记 memory-cap-at-maximum，实际 {seen}、{describe(first)}")
    world.resources(local=(30, 30))
    second_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key})
    second = world.finished(second_id)
    attempt = second["attempts"][0] if second["attempts"] else {}
    wrapper_caps = wrapper_cap_texts(world, second_id)
    if second["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != CAP_HIT_GROWN_PINNED_TEXT or wrapper_caps[:1] != [CAP_HIT_GROWN_PINNED_TEXT]:
        raise SelftestFailure(f"只撞过一次 {CAP_HIT_PINNED_TEXT}、还没跑完过的命令再交：第一次就应当从 {CAP_HIT_GROWN_PINNED_TEXT} 起（撞过的上限 × 1.5），"
                              f"实际上限 {attempt.get('memory_cap_text')}、包装收到 {wrapper_caps}、{describe(second)}", world.server_log_lines())
    return f"只撞过一次 {CAP_HIT_PINNED_TEXT} 的命令再交，第一次就从 {CAP_HIT_GROWN_PINNED_TEXT} 起"


def cell_failed_run_estimated(world):
    """预计用时（task_server_lib.py 文件头「资源记录」的预计用时）：命令自己跑到退出的都算——这条命令只有一次跑了一秒退 1 的尝试，同一条再交、排着时预计用时就是那一次的用时
    （任务 9730 跑约 1807 秒退 1，同一条再交只照退 0 的那一次报 80 秒）。弄坏开关 estimate-only-succeeded 下只认 succeeded、排着时 estimated_seconds 是 null，这一格红。"""
    key = "selftest-failed-estimate"
    world.resources(local=(30, 30))
    world.start_server()
    first = world.finished(world.submit(["bash", "-c", "sleep 1; exit 1"], extra={lib.KEY_VARIABLE: key}))
    seconds = first_attempt(first).get("seconds")
    world.resources(local=(30, 30), hold=True)
    task_id = world.submit(["bash", "-c", "sleep 1; exit 1"], extra={lib.KEY_VARIABLE: key})
    listed = world.client(["status", "--json"])
    items = [item for item in json.loads(listed.stdout or "{}").get("tasks", []) if item.get("id") == task_id]
    world.resources(local=(30, 30))
    world.finished(task_id)
    estimated = items[0].get("estimated_seconds") if items else None
    if (first["task"].get("outcome") != "command-failed" or not isinstance(seconds, (int, float)) or not isinstance(estimated, (int, float))
            or abs(estimated - seconds) > 0.01):
        raise SelftestFailure(f"这条命令只有一次跑了一阵退 1 的尝试（用时 {seconds}）：它跑到了退出，用时就是预计用时，排着时 estimated_seconds 应当是 {seconds}，"
                              f"实际头一次 {describe(first)}、列表 {items}")
    return f"只有一次跑了 {seconds} 秒退 1 的命令再交，排着时预计用时 {estimated} 秒（命令自己跑到退出的都算）"


def cell_queued_shows_waiting_reason(world):
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"])
    time.sleep(1)
    shown = world.client(["status", str(task_id)])
    listed = world.client(["status", "--json"])
    items = [item for item in json.loads(listed.stdout or "{}").get("tasks", []) if item.get("id") == task_id]
    world.resources(local=(30, 30))
    world.finished(task_id)
    if "在等：自证的假余量写了 hold" not in shown.stdout or not items or "hold" not in (items[0].get("waiting_for") or ""):
        raise SelftestFailure(f"放不下而排着的任务 status 应当报在等什么（「在等：自证的假余量写了 hold」），实际「{shown.stdout.strip()[:300]}」、列表 {items}")
    return "排着的任务 status 报「在等：自证的假余量写了 hold」，列表里带 waiting_for"


def cell_attempt_unit_nofile_raised(world):
    import resource
    hard = resource.getrlimit(resource.RLIMIT_NOFILE)[1]
    wanted = "unlimited" if hard == resource.RLIM_INFINITY else str(hard)
    world.resources(local=(30, 30))
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", "echo NOFILE=$(ulimit -Sn):$(ulimit -Hn)"]))
    log = summary["attempts"][0].get("log", "") if summary["attempts"] else ""
    text = open(log, encoding="utf-8", errors="replace").read() if log and os.path.exists(log) else ""
    if f"NOFILE={wanted}:{wanted}" not in text:
        found = [line for line in text.splitlines() if line.startswith("NOFILE=")]
        raise SelftestFailure(f"尝试单元里打开文件数的软、硬上限都应当是服务的硬上限 {wanted}，实际 {found or '没读到'}（{describe(summary)}）", world.server_log_lines())
    return f"尝试单元里打开文件数软、硬上限都是 {wanted}"


def cell_queued_estimate(world):
    key = "selftest-estimate"
    world.seed_duration(key, 3600, ["bash", "-c", "exit 0"])
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key})
    shown = world.client(["status", str(task_id)])
    listed = world.client(["status", "--json"])
    items = [item for item in json.loads(listed.stdout or "{}").get("tasks", []) if item.get("id") == task_id]
    world.resources(local=(30, 30))
    world.finished(task_id)
    if "预计用时 3600 秒" not in shown.stdout or not items or items[0].get("estimated_seconds") != 3600:
        raise SelftestFailure(f"排队时 status 应当报预计用时 3600 秒（库里记的那一次跑完的用时），实际「{shown.stdout.strip()[:300]}」、列表 {items}")
    return "排着的任务 status 报预计用时 3600 秒"


def recorded_host_line(world, task_id):
    """客户端 status <号> 打的第一次尝试那一行（「第 1 次：本机、上限 …」）；没有交空串。"""
    shown = world.client(["status", str(task_id)])
    return next((line.strip() for line in shown.stdout.splitlines() if "第 1 次：" in line), "")


def cell_balanced_when_both_fit(world):
    world.resources(local=(40, 40), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-long-and-big", 10 * GIBIBYTE)
    world.seed_duration("selftest-long-and-big", 7200, ["bash", "-c", "exit 0"])
    world.start_server()
    placed, lines = {}, {}
    for name, extra in (("plain", {}), ("long-and-big", {lib.KEY_VARIABLE: "selftest-long-and-big"})):
        summary = world.finished(world.submit(["bash", "-c", "exit 0"], extra=extra, cwd=world.repo))
        placed[name] = summary["attempts"][0].get("host") if summary["attempts"] else None
        lines[name] = recorded_host_line(world, summary["task"]["id"])
    if placed != {"plain": "peer", "long-and-big": "peer"} or not all("第 1 次：第二台" in line for line in lines.values()):
        raise SelftestFailure(f"本机总上限 40G、第二台 64G，两台都放得下、都派得到：放下之后第二台剩余占比更大，两条都应当去第二台、status 记「第二台」，"
                              f"实际 {placed}、status 那一行 {lines}", world.server_log_lines())
    return "两台都放得下：挑放下之后剩余占比大的第二台，status 记「第二台」"


def cell_peer_when_local_full(world):
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-overflow", 10 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: "selftest-overflow"}, cwd=world.repo)
    try:
        summary = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    except SelftestFailure as failure:
        raise SelftestFailure(f"本机此刻只空 8G、放不下预测 10G 的这一条（slice 30G，等得到），第二台空 100G 放得下：应当派到第二台跑完，实际 {failure}",
                              failure.extra_lines) from None
    host = summary["attempts"][0].get("host") if summary["attempts"] else None
    line = recorded_host_line(world, task_id)
    if host != "peer" or summary["task"].get("outcome") != "succeeded" or "第 1 次：第二台" not in line:
        raise SelftestFailure(f"本机此刻放不下、第二台放得下：应当派到第二台、status 记「第二台」，实际 {describe(summary)}、status 那一行「{line}」", world.server_log_lines())
    return "本机此刻放不下（空 8G、要 10G）、第二台放得下：派到第二台，status 记「第二台」"


def cell_every_command_reaches_peer(world):
    """没有只派一台的名单（task_server.py 的 RESTORED_HOST_LISTS 只是删之前的样子）：删之前只在本机的（scripts/hosts/、.claude/gate.d/ 下的与读本机东西的那几个入口）
    与第二台在同一棵共享的树里跑之后放开的那几类（task-server/、scripts/agents/、.claude/hooks/、带 --write 的、scripts/kb/、不带 --skip-apply 的 merge-and-verify），
    本机放不下（各记 15G、本机空 8G）、第二台放得下时都派到第二台跑完；删之前只在第二台跑的 e185-r2-segment-run.sh，第二台用不了时在本机跑完。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-every-command", 10 * GIBIBYTE)
    world.start_server()
    key = {lib.KEY_VARIABLE: "selftest-every-command"}
    commands = {"scripts/hosts/ 下的": "scripts/hosts/multi-host-run.sh", ".claude/gate.d/ 下的": ".claude/gate.d/doc-text.sh",
                "task-server/ 下的": "task-server/task_server_lib.py", "scripts/agents/ 下的": "scripts/agents/agent-cost.py",
                ".claude/hooks/ 下的": ".claude/hooks/lib_project_code_runs.py", "带 --write 的": "--write", "scripts/kb/ 下的": "scripts/kb/replace-once.py",
                "不带 --skip-apply 的 merge-and-verify": "scripts/gate/merge-and-verify.sh", "删之前只在第二台的": SERVER.RESTORED_HOST_LISTS["peer_entries"][0]}
    commands.update({f"删之前读本机东西的入口 {entry}": entry for entry in SERVER.RESTORED_HOST_LISTS["local_entries"]})
    submitted = {name: world.submit(["bash", "-c", "exit 0", "x", word], extra=key, cwd=world.repo) for name, word in commands.items()}
    problems = []
    pending = dict(submitted)
    patience = world.patience(3 * PLACEMENT_WAIT_SECONDS)
    while pending and patience.waiting():
        for name, task_id in list(pending.items()):
            summary = world.summary(task_id)
            if summary["task"].get("state") == "finished":
                del pending[name]
                if first_attempt(summary).get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
                    problems.append(f"点到{name}应当派到第二台跑完（本机放不下），实际 {describe(summary)}")
        time.sleep(POLL_SECONDS)
    for name, task_id in pending.items():
        found = world.summary(task_id)
        problems.append(f"点到{name}应当派到第二台跑完，{3 * PLACEMENT_WAIT_SECONDS:.0f} 秒里没收尾：{found['task'].get('state')}、"
                        f"在等「{found.get('waiting_for') or ''}」、尝试 {len(found['attempts'])} 次")
        world.client(["cancel", str(task_id)])
    world.resources(local=(100, 100))   # 第二台此刻用不了
    entry = SERVER.RESTORED_HOST_LISTS["peer_entries"][0]
    try:
        local = world.finished(world.submit(["bash", "-c", "exit 0", entry, "peer-gone"], cwd=world.repo), timeout=PLACEMENT_WAIT_SECONDS)
        if first_attempt(local).get("host") != "local" or local["task"].get("outcome") != "succeeded":
            problems.append(f"第二台用不了时点到 {entry} 的应当在本机跑完，实际 {describe(local)}")
    except SelftestFailure as failure:
        problems.append(f"第二台用不了时点到 {entry} 的应当在本机跑完，实际没收尾：{failure}")
    if problems:
        raise SelftestFailure("没有只派一台的名单：" + "；".join(problems), world.server_log_lines())
    return (f"{len(commands)} 类命令（删之前只在本机的、放开的、只在第二台的）本机放不下时都派到第二台跑完；"
            f"第二台用不了时 {entry} 在本机跑完")


def cell_shard_to_peer_by_plan(world):
    """分片的各片不钉台（task_server_lib.py 文件头「分片」）：计划里写了 pin 的整件记 shard-plan-refused、wait 退 234、说明写「片不钉台，哪台由服务挑」；
    没写 pin 的两片里先排的那一片本机更空也照计划先去第二台（另一片去哪台看它排的时候第二台上那一片还在不在跑：一台同一时刻只跑同一件的一片），
    两片都成了再合并。"""
    world.resources(local=(100, 100), peer=(60, 60), peer_cap_gibibytes=64)   # 本机更空：按均衡会挑本机，只有照计划才去第二台
    pinned_spec = world.write_spec("spec-pinned.json", {"shards": [{"pin": "local", "label": "写了 pin 的那一片"}, {"label": "没写的那一片"}]})
    plain_spec = world.write_spec("spec-two-hosts.json", {"shards": [{"label": "第一片"}, {"label": "第二片"}]})
    world.start_server()
    problems = []
    refused_id = world.submit(pinned_spec, cwd=world.repo)
    refused = world.finished(refused_id)
    waited = world.client(["wait", str(refused_id)])
    if refused["task"].get("outcome") != "shard-plan-refused" or waited.returncode != 234 or "片不钉台，哪台由服务挑" not in waited.stdout:
        problems.append(f"计划写了 pin：整件应当记 shard-plan-refused、wait 退 234、说「片不钉台，哪台由服务挑」，实际 {describe(refused)}、wait 退 {waited.returncode}、"
                        f"「{waited.stdout.strip()[-300:]}」")
    summary = world.finished(world.submit(plain_spec, cwd=world.repo))
    hosts = {}
    for child in summary.get("children", []):
        if child["task"].get("role") == "shard":
            hosts[child["task"].get("shard_index")] = [attempt.get("host") for attempt in child["attempts"]]
    merged = world.sharded_calls("merge", present="0,1")
    if summary["task"].get("outcome") != "succeeded" or hosts.get(0) != ["peer"] or len(hosts.get(1) or []) != 1 or not merged:
        problems.append(f"本机空 100G、第二台空 60G，一件分两片、都没写 pin：先排的那一片照计划派到第二台（本机放得下也去）、另一片起一次，两片都成了再合并，"
                        f"实际整件 {summary['task'].get('outcome')}、各片的台 {hosts}、合并 {merged}")
    if problems:
        raise SelftestFailure("分片不钉台：" + "；".join(problems), world.server_log_lines())
    return "计划写了 pin 的记 shard-plan-refused（片不钉台）；没写 pin 的两片里先排的那一片照计划在第二台（本机放得下也去），合并之后整件成了"


def running_shards(summary):
    """分片父任务的各片里此刻有尝试在跑的：{片号: 那一次的台}。"""
    return {index: attempt.get("host") for index, child in shard_children(summary).items()
            for attempt in child["attempts"] if attempt.get("state") == "running"}


def cell_shard_siblings_share_a_host(world):
    """同一件的兄弟片不避开彼此（task_server_lib.py 文件头「挑机器」）：两台都空时两片同时跑；第二台用不了、只有本机接得了而内存放得下时，两片同时在本机跑，不排队等兄弟片。"""
    releases = {name: os.path.join(world.root, f"release-siblings-{name}") for name in ("spread", "together")}
    spread_spec = world.write_spec("spec-siblings-spread.json", {"shards": [{"block": releases["spread"]}, {"block": releases["spread"]}]})
    one_host_spec = world.write_spec("spec-siblings-one-host.json", {"shards": [{"block": releases["together"]}, {"block": releases["together"]}]})
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server()
    problems = []
    spread_id = world.submit(spread_spec, cwd=world.repo)
    try:
        world.wait_for(spread_id, lambda summary: len(running_shards(summary)) == 2, "两片都跑起来")
    finally:
        open(releases["spread"], "w").close()
    spread = world.finished(spread_id)
    if spread["task"].get("outcome") != "succeeded":
        problems.append(f"两台都空、一件两片：整件应当成了，实际 {describe(spread)}")
    world.resources(local=(100, 100), peer_down="自证：第二台此刻用不了")
    one_host_id = world.submit(one_host_spec, cwd=world.repo)
    try:
        summary = world.wait_for(one_host_id, lambda summary: len(running_shards(summary)) == 2, "两片同时在本机跑起来")
        if sorted(running_shards(summary).values()) != ["local", "local"]:
            problems.append(f"只有本机接得了：两片应当同时在本机跑，实际在跑的各片的台 {running_shards(summary)}")
    finally:
        open(releases["together"], "w").close()
    one_host = world.finished(one_host_id)
    if one_host["task"].get("outcome") != "succeeded":
        problems.append(f"只有本机接得了：整件应当成了，实际 {describe(one_host)}")
    if problems:
        raise SelftestFailure("兄弟片同台：" + "；".join(problems), world.server_log_lines())
    return "两台都空时两片同时跑；只有本机接得了而内存放得下时两片同时在本机跑，不排队等兄弟片"


def cell_socket_path_too_long(world):
    runtime = os.path.join(world.root, "runtime")
    while len(os.fsencode(os.path.join(runtime, lib.SOCKET_NAME))) <= lib.SOCKET_PATH_MAXIMUM_BYTES:
        runtime = os.path.join(runtime, TOO_DEEP_SOCKET_DIRECTORY)
    socket_path = os.path.join(runtime, lib.SOCKET_NAME)
    length = len(os.fsencode(socket_path))
    config = os.path.join(world.root, "too-long-socket-multi-host.env")
    World.write_config(config, world.state, runtime, world.key, world.builds, world.shared_root)
    named = (socket_path, f"{length} 字节", f"{lib.SOCKET_PATH_MAXIMUM_BYTES} 字节")
    server = subprocess.Popen([sys.executable, "-B", lib.SERVER_SCRIPT, "serve"], env=dict(world.server_environment, **{lib.MULTI_HOST_CONFIG_VARIABLE: config}),
                              stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True, start_new_session=True)
    try:
        _, server_stderr = world.communicate(server, STATE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        server.kill()   # 只停自己起的那一个进程号
        _, server_stderr = server.communicate(timeout=10)
        raise SelftestFailure(f"socket 路径 {length} 字节（上限 {lib.SOCKET_PATH_MAXIMUM_BYTES}）时服务应当当场退出，实际 {STATE_TIMEOUT_SECONDS:.0f} 秒还没退",
                              [server_stderr]) from None
    client = world.client(["status"], extra={lib.MULTI_HOST_CONFIG_VARIABLE: config})
    problems = []
    if server.returncode == 0 or not all(text in server_stderr for text in named):
        problems.append(f"服务应当退非 0、stderr 点名 socket 路径、它的 {length} 字节与上限 {lib.SOCKET_PATH_MAXIMUM_BYTES} 字节，实际退 {server.returncode}、"
                        f"stderr 里缺 {[text for text in named if text not in server_stderr]}")
    if client.returncode != lib.RUN_CONDITION_EXIT or not all(text in client.stderr for text in named):
        problems.append(f"客户端 status 应当退 78、stderr 点名同样三样，实际退 {client.returncode}、stderr 里缺 {[text for text in named if text not in client.stderr]}")
    if os.path.lexists(socket_path):
        problems.append(f"socket {socket_path} 不该建出来，实际在")
    if problems:
        raise SelftestFailure("socket 路径太长：" + "；".join(problems), [server_stderr, client.stderr])
    return f"socket 路径 {length} 字节（上限 {lib.SOCKET_PATH_MAXIMUM_BYTES}）：服务退 {server.returncode}、客户端退 78，都点名路径、字节数与上限"


def cell_threads_fixed(world):
    world.start_server()
    expected = str(HEAVY_THREADS.default_heavy_threads({}))
    seen = {}
    probe = os.path.join(world.root, "threads-registered")
    world.finished(world.submit(["bash", "-c", 'printf "%s" "${SINGLEFS_THREAD_CAP:-}" > "$1"', "probe", probe]))
    with contextlib.suppress(OSError):
        with open(probe, encoding="utf-8") as handle:
            seen["registered"] = handle.read()
    given = world.client(["submit", "--", "bash", "-c", "exit 0"], extra={"SINGLEFS_THREAD_CAP": "7"})
    if seen != {"registered": expected} or given.returncode == 0 or "SINGLEFS_THREAD_CAP" not in given.stderr:
        raise SelftestFailure(f"每一件都经 capped.sh 设登记的 {expected} 个线程，提交时带 SINGLEFS_THREAD_CAP=7 的当场拒并点名；"
                              f"实际命令看到 {seen}、带 7 的那次退 {given.returncode}", [given.stdout, given.stderr])
    return f"命令看到的线程数是登记的 {expected}；提交时带 SINGLEFS_THREAD_CAP 的当场拒"


def cell_shard_retry_isolated(world):
    world.resources(local=(60, 60))
    world.start_server()
    # 撞上限的那一片晚 3 秒才退：兄弟片这时已经记成 succeeded（定时缩短下对账一轮 0.1 秒），弄坏开关 shard-retry-restarts-all 下它们会被重排、多跑一次
    retried = world.write_spec("spec-retry.json", {"shards": [{}, {}, {"exits": [250, "run"], "first_attempt_delay": 3}]})
    failing = world.write_spec("spec-failing.json", {"shards": [{}, {"exits": [3]}, {}]})
    broken_plan = world.write_spec("spec-broken-plan.json", {"shards": [{}], "plan_text": "不是 JSON"})
    retried_id = world.submit(retried, cwd=world.repo)
    retried_summary = world.finished(retried_id)
    problems = []
    children = {(child["task"].get("role"), child["task"].get("shard_index")): child for child in retried_summary.get("children", [])}
    task_of = {index: str(children[("shard", index)]["task"]["id"]) for index in range(3) if ("shard", index) in children}
    runs = {index: [row.get("attempt") for row in world.sharded_calls("shard", task=task_of.get(index, "-"))] for index in range(3)}
    merge_task = str(children.get(("merge", 0), {"task": {}})["task"].get("id", "-"))
    merges = world.sharded_calls("merge", task=merge_task)
    if retried_summary["task"].get("outcome") != "succeeded" or runs != {0: ["1"], 1: ["1"], 2: ["1", "2"]} or [row.get("present") for row in merges] != ["0,1,2"]:
        problems.append(f"第 2 片撞一次上限：只重排那一片（第 2 片跑 2 次、别的各 1 次），全部片绿了才合并一次、合并时三片都在，"
                        f"实际整件 {retried_summary['task'].get('outcome')}、各片跑的次数 {runs}、合并 {merges}")
    failing_id = world.submit(failing, cwd=world.repo)
    failing_summary = world.finished(failing_id)
    waited = world.client(["wait", str(failing_id)])
    last = waited.stdout.strip().splitlines()[-1] if waited.stdout.strip() else ""
    has_merge = any(child["task"].get("role") == "merge" for child in failing_summary.get("children", []))
    if (failing_summary["task"].get("outcome") != "command-failed" or waited.returncode != 3 or " shard=1/3" not in last or has_merge
            or not last.startswith(f"{lib.RESULT_MARK} task={failing_id} type=command-failed exit=3 ")):
        problems.append(f"第 1 片退 3：整件结局应当是那一片的 command-failed、wait 退 3、末行点名 shard=1/3、不合并，"
                        f"实际 {describe(failing_summary)}、wait 退 {waited.returncode}、末行「{last}」、{'合并了' if has_merge else '没合并'}")
    plan_id = world.submit(broken_plan, cwd=world.repo)
    plan_summary = world.finished(plan_id)
    waited = world.client(["wait", str(plan_id)])
    last = waited.stdout.strip().splitlines()[-1] if waited.stdout.strip() else ""
    if plan_summary["task"].get("outcome") != "shard-plan-refused" or waited.returncode != 234 or " step=plan" not in last:
        problems.append(f"计划写坏：整件应当记 shard-plan-refused、wait 退 234、末行点名 step=plan，实际 {describe(plan_summary)}、wait 退 {waited.returncode}、末行「{last}」")
    if problems:
        raise SelftestFailure("分片：" + "；".join(problems), world.server_log_lines())
    return "一片撞上限只重排那一片、全部片绿了才合并；一片最终没成整件是那一片的类型、末行点名 shard=1/3；计划写坏记 shard-plan-refused"


def log_printed(completed):
    """task log 打的日志行（去掉头一行「== 任务 … 的日志 …」）。"""
    lines = completed.stdout.splitlines()
    return lines[1:] if lines[:1] and lines[0].startswith("== 任务 ") else lines


def attempt_lines(number, first=1, last=LOG_LINES_PER_ATTEMPT):
    """LOG_SCRIPT 在第 number 次尝试里打的第 first 到 last 行。"""
    return [f"attempt-{number} line-{line}" for line in range(first, last + 1)]


def cell_task_log(world):
    FakeInbox(world, "a")   # 会话登进会话登记：结局发得出去，最后一次尝试的日志末尾不多出发不出去的那一行（lib.INBOX_LOG_MARK）
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    problems = []
    queued_id = world.submit(["bash", "-c", "exit 0"])
    completed = world.client(["log", str(queued_id)])
    if completed.returncode != 4 or "没有任何一次尝试" not in completed.stderr:
        problems.append(f"还没起过的任务 {queued_id}：log 应当退 4、说没有任何一次尝试，实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-200:]}」")
    world.resources(local=(30, 30))
    world.finished(queued_id)
    task_id = world.submit(["bash", "-c", LOG_SCRIPT, "log-lines", str(LOG_LINES_PER_ATTEMPT)])
    summary = world.finished(task_id)
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    if outcomes != ["squeezed-out", "succeeded"]:
        raise SelftestFailure("task log：" + "；".join(problems + [f"要读日志的那一件应当有两次尝试（第 1 次命令退 254 记 squeezed-out、重排的第 2 次 succeeded），"
                                                                 f"实际 {describe(summary)}"]))
    for arguments, what, header, expected in (
            (["log", str(task_id)], "不带参数", "第 2 次尝试", attempt_lines(2, LOG_LINES_PER_ATTEMPT - LOG_TAIL_PINNED_LINES + 1)),
            (["log", str(task_id), "--all"], "--all", "第 2 次尝试", attempt_lines(2)),
            (["log", str(task_id), "--attempt", "1", "--all"], "--attempt 1 --all", "第 1 次尝试", attempt_lines(1))):
        completed = world.client(arguments)
        printed = log_printed(completed)
        head = completed.stdout.splitlines()[0] if completed.stdout else ""
        if completed.returncode != 0 or header not in head or printed != expected:
            problems.append(f"log {what} 应当退 0、打{header}的 {len(expected)} 行（{expected[0]} 到 {expected[-1]}），实际退 {completed.returncode}、"
                            f"头一行「{head[:160]}」、打了 {len(printed)} 行（{printed[:1]} 到 {printed[-1:]}）")
    for arguments, what, said in ((["log", str(task_id), "--attempt", "3"], "要第 3 次尝试（只有两次）", "没有第 3 次尝试"),
                                  (["log", str(ABSENT_TASK_ID)], f"任务 {ABSENT_TASK_ID} 不在", f"没有任务 {ABSENT_TASK_ID}")):
        completed = world.client(arguments)
        if completed.returncode != 4 or said not in completed.stderr:
            problems.append(f"log {what}：应当退 4、说「{said}」，实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-200:]}」")
    if problems:
        raise SelftestFailure("task log：" + "；".join(problems))
    return (f"log 默认打最后一次尝试的最后 {LOG_TAIL_PINNED_LINES} 行、--all 打整份、--attempt 1 取第 1 次；"
            "要的那一次不在、任务不在、还没起过都退 4")


def cell_cancel_queued(world):
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    marker = os.path.join(world.root, "queued-ran")
    task_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", marker])
    completed = world.client(["cancel", str(task_id)])
    summary = world.summary(task_id)
    problems = []
    if (completed.returncode != 0 or "在队里时取消" not in completed.stdout or summary["task"].get("outcome") != "cancelled"
            or summary["task"].get("exit_code") != 239 or summary["attempts"]):
        problems.append(f"取消排着的：应当退 0、当场记 cancelled（退 239）、一次没起，实际退 {completed.returncode}、状态 {summary['task'].get('state')}、{describe(summary)}")
    world.resources(local=(30, 30))
    time.sleep(1.5)
    summary = world.summary(task_id)
    if summary["attempts"] or world.calls_of(task_id) or os.path.exists(marker):
        problems.append(f"放开余量之后，取消了的那一件应当一次都不起，实际 {describe(summary)}、包装调用 {world.calls_of(task_id)}、"
                        f"命令{'跑了' if os.path.exists(marker) else '没跑'}")
    if problems:
        raise SelftestFailure("；".join(problems), [completed.stdout, completed.stderr])
    return "取消排着的：当场记 cancelled、退 0，放开余量之后也一次没起"


def cell_cancel_finished(world):
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 77"])
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "not-run":
        raise SelftestFailure(f"要取消的那一件应当先有结局 not-run（命令退 77），实际 {describe(summary)}")
    problems = []
    for times in (1, 2):
        completed = world.client(["cancel", str(task_id)])
        summary = world.summary(task_id)
        if (completed.returncode != 0 or "已经有结局了" not in completed.stdout or summary["task"].get("outcome") != "not-run"
                or summary["task"].get("exit_code") != 77 or len(summary["attempts"]) != 1):
            problems.append(f"第 {times} 回取消已有结局的：应当退 0、说已经有结局了、结局照旧 not-run（退 77）、尝试照旧 1 次，实际退 {completed.returncode}、{describe(summary)}")
    if problems:
        raise SelftestFailure("；".join(problems))
    return "取消已有结局的连取消两回都退 0、结局照旧 not-run（退 77），不改记 cancelled"


def child_roles(summary):
    return sorted(child["task"].get("role") for child in summary.get("children", []))


def shard_children(summary):
    """分片父任务的各片：{片号: 子任务的 summary}。"""
    return {child["task"].get("shard_index"): child for child in summary.get("children", []) if child["task"].get("role") == "shard"}


def cell_cancel_sharded_in_plan(world):
    world.resources(local=(30, 30), hold=True)
    command = world.write_spec("spec-cancel-in-plan.json", {"shards": [{}]})
    world.start_server()
    task_id = world.submit(command, cwd=world.repo)
    children = world.summary(task_id).get("children", [])
    if [(child["task"].get("role"), child["task"].get("state")) for child in children] != [("plan", "queued")]:
        raise SelftestFailure(f"分片的入口交上去应当先拆出一步排着的计划，实际子任务 {[(child['task'].get('role'), child['task'].get('state')) for child in children]}")
    plan_id = children[0]["task"]["id"]
    completed = world.client(["cancel", str(task_id)])
    summary, plan = world.summary(task_id), world.summary(plan_id)
    problems = []
    if (completed.returncode != 0 or summary["task"].get("outcome") != "cancelled" or summary["task"].get("exit_code") != 239
            or plan["task"].get("outcome") != "cancelled" or plan["attempts"]):
        problems.append(f"计划还排着时取消整件：应当退 0、整件记 cancelled（退 239）、计划那一步当场记 cancelled、一次没起，实际退 {completed.returncode}、"
                        f"整件 {describe(summary)}、计划那一步 {plan['task'].get('state')} {describe(plan)}")
    world.resources(local=(30, 30))
    time.sleep(1.5)
    plan = world.summary(plan_id)
    if plan["attempts"] or world.calls_of(plan_id) or world.sharded_calls("plan") or child_roles(world.summary(task_id)) != ["plan"]:
        problems.append(f"放开余量之后计划那一步应当一次都不起、不再拆出别的子任务，实际计划那一步 {describe(plan)}、计划跑的记录 {world.sharded_calls('plan')}、"
                        f"子任务 {child_roles(world.summary(task_id))}")
    if problems:
        raise SelftestFailure("取消分片的整件：" + "；".join(problems), [completed.stdout, completed.stderr])
    return "计划还排着时取消整件：整件与计划那一步当场记 cancelled，放开余量之后也一次没起"


def cell_cancel_sharded_running(world):
    release = os.path.join(world.root, "release-shards")
    command = world.write_spec("spec-cancel-running.json", {"shards": [{"block": release}, {"block": release}]})
    world.resources(local=(30, 30), peer=(30, 30))   # 两片要同时在跑：一台同一时刻只跑同一件的一片，给两台
    world.start_server()
    task_id = world.submit(command, cwd=world.repo)
    problems = []
    try:
        summary = world.wait_for(task_id, lambda summary: len(shard_children(summary)) == 2 and len(world.sharded_calls("shard")) == 2 and all(
            any(attempt.get("state") == "running" for attempt in child["attempts"]) for child in shard_children(summary).values()), "两片都跑起来")
        units = [child["attempts"][-1].get("unit", "") for child in shard_children(summary).values()]
        completed = world.client(["cancel", str(task_id)])
        still = [unit for unit in units if world.unit_active(unit)]
        summary = world.summary(task_id)
        if completed.returncode != 0 or summary["task"].get("outcome") != "cancelled" or summary["task"].get("exit_code") != 239 or still:
            problems.append(f"两片都在跑时取消整件：应当退 0、整件记 cancelled（退 239）、回来时两片的单元都停了，实际退 {completed.returncode}、"
                            f"整件 {summary['task'].get('outcome')}、还在跑的单元 {still}")
    finally:
        open(release, "w").close()   # 弄坏开关下单元没停：放它们跑完，收尾不等满 60 秒
    for index, child in sorted(shard_children(world.summary(task_id)).items()):
        finished = world.finished(child["task"]["id"])
        if finished["task"].get("outcome") != "cancelled":
            problems.append(f"第 {index} 片应当记 cancelled，实际 {describe(finished)}")
    if "merge" in child_roles(world.summary(task_id)) or world.sharded_calls("merge"):
        problems.append(f"取消了的整件不该再合并，实际子任务 {child_roles(world.summary(task_id))}、合并的记录 {world.sharded_calls('merge')}")
    if problems:
        raise SelftestFailure("取消分片的整件：" + "；".join(problems), world.server_log_lines())
    return "两片都在跑时取消整件：整件记 cancelled，回来时两片的单元都停了、各记 cancelled，不合并"


def wait_last_line(world, task_id):
    """客户端 wait 的退出码与末行。"""
    waited = world.client(["wait", str(task_id)])
    return waited.returncode, (waited.stdout.strip().splitlines()[-1] if waited.stdout.strip() else "")


def cell_shard_plan_failed(world):
    world.start_server()
    command = world.write_spec("spec-plan-exit.json", {"shards": [{}], "plan_exit": SHARD_PLAN_EXIT})
    task_id = world.submit(command, cwd=world.repo)
    summary = world.finished(task_id)
    code, last = wait_last_line(world, task_id)
    if (summary["task"].get("outcome") != "command-failed" or summary["task"].get("exit_code") != SHARD_PLAN_EXIT or code != SHARD_PLAN_EXIT
            or not last.startswith(f"{lib.RESULT_MARK} task={task_id} type=command-failed exit={SHARD_PLAN_EXIT} ") or " step=plan" not in last
            or child_roles(summary) != ["plan"] or world.sharded_calls("shard") or world.sharded_calls("merge")):
        raise SelftestFailure(f"计划那一步退 {SHARD_PLAN_EXIT}（plan.json 照写了）：整件应当是那一步的 command-failed、退 {SHARD_PLAN_EXIT}，wait 退 {SHARD_PLAN_EXIT}、"
                              f"末行点名 step=plan，不拆片、不合并，实际 {describe(summary)}、wait 退 {code}、末行「{last}」、子任务 {child_roles(summary)}、"
                              f"片跑的记录 {len(world.sharded_calls('shard'))} 行、合并的 {len(world.sharded_calls('merge'))} 行", world.server_log_lines())
    return f"计划那一步退 {SHARD_PLAN_EXIT}：整件 command-failed、wait 退 {SHARD_PLAN_EXIT}、末行点名 step=plan，没拆片"


def cell_shard_merge_failed(world):
    world.start_server()
    command = world.write_spec("spec-merge-exit.json", {"shards": [{}], "merge_exit": SHARD_MERGE_EXIT})
    task_id = world.submit(command, cwd=world.repo)
    summary = world.finished(task_id)
    code, last = wait_last_line(world, task_id)
    if (summary["task"].get("outcome") != "command-failed" or summary["task"].get("exit_code") != SHARD_MERGE_EXIT or code != SHARD_MERGE_EXIT
            or not last.startswith(f"{lib.RESULT_MARK} task={task_id} type=command-failed exit={SHARD_MERGE_EXIT} ") or " step=merge" not in last
            or child_roles(summary) != ["merge", "plan", "shard"] or len(world.sharded_calls("shard")) != 1 or len(world.sharded_calls("merge", present="0")) != 1):
        raise SelftestFailure(f"一片跑完、合并那一步退 {SHARD_MERGE_EXIT}：整件应当是那一步的 command-failed、退 {SHARD_MERGE_EXIT}，wait 退 {SHARD_MERGE_EXIT}、"
                              f"末行点名 step=merge，实际 {describe(summary)}、wait 退 {code}、末行「{last}」、子任务 {child_roles(summary)}、"
                              f"片跑的记录 {world.sharded_calls('shard')}、合并的 {world.sharded_calls('merge')}", world.server_log_lines())
    return f"合并那一步退 {SHARD_MERGE_EXIT}：整件 command-failed、wait 退 {SHARD_MERGE_EXIT}、末行点名 step=merge"


def cell_unsharded_arguments(world):
    world.start_server()
    problems = []
    control = world.finished(world.submit(world.write_spec("spec-unsharded-control.json", {"shards": [{}]}), cwd=world.repo))
    if not control["task"].get("sharded_entry") or control["task"].get("outcome") != "succeeded" or child_roles(control) != ["merge", "plan", "shard"]:
        problems.append(f"对照：假分片入口的文件头写着「# task-server-shards: *」，带 run 交上去应当拆成计划、一片、合并跑完，实际认成 "
                        f"{control['task'].get('sharded_entry') or '不分片'}、整件 {control['task'].get('outcome')}、子任务 {child_roles(control)}")
    for argument in UNSHARDED_PINNED_ARGUMENTS:
        task_id = world.submit(["python3", "fake-sharded.py", argument], cwd=world.repo)
        summary = world.finished(task_id)
        whole = world.sharded_calls("whole", task=task_id, first=argument)
        if summary["task"].get("sharded_entry") or summary.get("children") or summary["task"].get("outcome") != "succeeded" or len(whole) != 1:
            problems.append(f"带 {argument} 的应当不分片、整件当普通命令跑一次，实际认成 {summary['task'].get('sharded_entry') or '不分片'}、"
                            f"子任务 {child_roles(summary)}、{describe(summary)}、整件跑的记录 {whole}")
    if problems:
        raise SelftestFailure("分片的入口：" + "；".join(problems), world.server_log_lines())
    return f"文件头写了「*」的分片入口带 run 照分片；带 {'、'.join(UNSHARDED_PINNED_ARGUMENTS)} 的不分片、整件跑一次"


def systemctl_start_count(world):
    """假 systemctl 收到过几回 start singlefs-task-server.service。"""
    try:
        with open(os.path.join(world.world, "systemctl-calls"), encoding="utf-8") as handle:
            return sum(1 for line in handle if line.split()[1:3] == ["start", lib.UNIT_NAME])
    except FileNotFoundError:
        return 0


def cell_control_start(world):
    extra = {"TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl")}
    problems = []
    completed = world.control(["start"], extra=extra)
    if completed.returncode != 1 or "起了但回读不对" not in completed.stderr or systemctl_start_count(world) != 1:
        problems.append(f"systemctl start 退 0 而服务一直不回话：control start 应当在限时里回读不到、退 1、说回读不对，实际退 {completed.returncode}、"
                        f"systemctl start 收到 {systemctl_start_count(world)} 回、stderr「{completed.stderr.strip()[-200:]}」")
    waiting = world.control(["start"], extra=dict(extra, TASK_SERVER_SELFTEST_FAST="0"), background=True)   # 不缩定时：回读限时 30 秒，等得到服务起来
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    while systemctl_start_count(world) < 2 and waiting.poll() is None and patience.waiting():
        time.sleep(POLL_SECONDS)
    exited_early = waiting.poll() is not None
    world.start_server()
    try:
        stdout, stderr = world.communicate(waiting, STATE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        raise SelftestFailure(f"服务起来之后 control start {STATE_TIMEOUT_SECONDS:.0f} 秒没退", ["；".join(problems)]) from None
    if waiting.returncode != 0 or exited_early or "/health 回 ok" not in stdout:
        problems.append(f"systemctl start 之后服务才起来：control start 应当等到 is-active 打 active、/health 回 ok 再退 0，实际退 {waiting.returncode}、"
                        f"{'服务起来之前就退了' if exited_early else '等到了服务起来'}、stdout「{stdout.strip()[-200:]}」、stderr「{stderr.strip()[-200:]}」")
    open(os.path.join(world.world, "systemctl-start-fails"), "w").close()
    completed = world.control(["start"], extra=extra)
    if completed.returncode != 1 or f"systemctl --user start {lib.UNIT_NAME} 退 1" not in completed.stderr:
        problems.append(f"systemctl start 退 1（服务此刻照跑）：control start 应当退 1、说 systemctl start 退了 1，不照回读的结论退 0，实际退 {completed.returncode}、"
                        f"stderr「{completed.stderr.strip()[-200:]}」")
    if problems:
        raise SelftestFailure("control start：" + "；".join(problems))
    return "control start 等到服务起来、回读 is-active 与 /health 都对退 0；服务一直不回话退 1；systemctl start 退非 0 退 1"


def cell_gpu_without_cards_unmet(world):
    world.resources(local=(100, 100), local_gpu=[(0, 16384)])   # 本机有卡，假余量里多机配置点名的卡却是空的
    world.start_server()
    path = f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"
    task_id = world.submit(["cargo", "test", "--features", "gpu", "gpu_without_cards_sample"], extra={"PATH": path})
    summary = world.finished(task_id)
    task = summary["task"]
    ran = [line for line in world.calls() if line.startswith(("wrapper ", "quiesce ", "cargo "))]
    if (not task.get("gpu") or task.get("outcome") != "run-condition-unmet" or task.get("exit_code") != lib.RUN_CONDITION_EXIT or summary["attempts"] or ran
            or "没点名能用的卡" not in (task.get("note") or "")):
        raise SelftestFailure(f"要卡的命令而多机配置一张卡都没点名：服务应当认成要卡、判 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}）、说没点名能用的卡、命令一行不跑，"
                              f"实际认成{'要卡' if task.get('gpu') else '不要卡'}、{describe(summary)}、说明「{task.get('note')}」、起了的调用 {ran}", world.server_log_lines())
    return f"要卡而多机配置没点名卡：服务判 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}），清场、内存包装、命令一个都没起"


def cell_pinned_peer_unavailable(world):
    key = "selftest-pinned-peer"
    world.resources(local=(30, 30), peer=(25, 100), peer_cap_gibibytes=64)
    world.seed_peak(key, 20 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250"], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
    world.wait_for(task_id, lambda summary: bool(summary["attempts"]) and summary["attempts"][0].get("state") == "ended", "第一次尝试收尾")
    world.resources(local=(30, 30))   # 第二台此刻用不了了
    try:
        summary = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    except SelftestFailure as failure:
        raise SelftestFailure(f"本机已给到它的总上限 30G 还撞、换到第二台钉在那里重排、第二台随即用不了：应当判 never-fits 收尾，实际 {failure}", failure.extra_lines) from None
    task = summary["task"]
    attempts = [(attempt.get("host"), attempt.get("outcome")) for attempt in summary["attempts"]]
    if attempts != [("local", "memory-cap-hit")] or task.get("outcome") != "never-fits" or task.get("exit_code") != 241 or "钉在第二台" not in (task.get("note") or ""):
        raise SelftestFailure(f"钉在第二台重排、第二台用不了：应当只有本机那一次撞上限的尝试、结局 never-fits（退 241）、说钉在第二台而它用不了，"
                              f"实际尝试 {attempts}、{describe(summary)}、说明「{task.get('note')}」", world.server_log_lines())
    return "本机到顶换到第二台钉着重排、第二台随即用不了：判 never-fits（退 241），不在本机另起"


def submitted_gpu(world, command, extra=None):
    """交一条、读 submit 回显的 gpu=；交 (任务号, 0 或 1)。"""
    completed = world.client(["submit", "--cwd", world.work, "--", *command], extra=extra)
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+) \S+ gpu=([01])", completed.stdout)
    if completed.returncode != 0 or not found:
        raise SelftestFailure(f"提交 {command} 没拿到任务号与 gpu=（退 {completed.returncode}）", [completed.stdout, completed.stderr] + world.server_log_lines())
    world.confirm_submitted(completed, extra=extra)   # 照确认单确认（task_server.py 文件头 ⑦）
    return int(found.group(1)), int(found.group(2))


def cell_gpu_runs_only_when_code_runs(world):
    world.resources(local=(100, 100), cards=LOCAL_CARD_ZERO, local_gpu=[(0, 16384)])
    world.start_server()
    cases = ((["cargo", "clippy", "--all-targets", "--all-features", "--", "-D", "warnings"], 0),
             (["cargo", "build", "--features", "gpu", "-p", "singlefs-verification"], 0),
             (["cargo", "check", "-F", "gpu"], 0), (["cargo", "doc", "--all-features"], 0),
             (["cargo", "test", "--features", "gpu", "--no-run", "-p", "singlefs-verification"], 0),
             (["cargo", "bench", "--all-features", "--no-run"], 0),
             (["bash", "-c", "cargo clippy --all-features && cargo build --features gpu"], 0),
             (["cargo", "test", "--features", "gpu", "-p", "singlefs-verification"], 1),
             (["cargo", "+stable", "t", "--features=gpu"], 1), (["cargo", "run", "--all-features", "--bin", "selftest"], 1),
             (["cargo", "test", "--features", "gpu", "--", "--no-run"], 1),
             (["cargo", "test", "--features", "tests-on-a-real-gpu", "-p", "singlefs-verification"], 1),
             (["cargo", "test", "-F", "singlefs-verification/tests-on-a-real-gpu"], 1))
    problems, submitted = [], []
    for words, expected in cases:
        task_id, gpu = submitted_gpu(world, [*words, "fake-wrapper-exits=0"])
        submitted.append(task_id)
        if gpu != expected:
            problems.append(f"{' '.join(words)} 应当回显 gpu={expected}，实际 gpu={gpu}")
    for task_id in submitted:
        world.finished(task_id)
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return (f"{len(cases)} 条带 gpu 特性的 cargo：clippy、build、check、doc、test / bench --no-run、bash -c 里的 clippy 加 build 回显 gpu=0，"
            "test、t、run、-- 之后的 --no-run 与开了 tests-on-a-real-gpu 的 test 回显 gpu=1")


def cell_stall_clock_from_log_mtime(world):
    quick_grace = {"TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0"}
    world.start_server(extra=quick_grace)
    release = os.path.join(world.root, "release")
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release])
    log = world.running(task_id)["attempts"][0].get("log", "")
    wait_path(world, log, "把尝试的日志建出来")   # 记了在跑和日志文件真建出来之间机器忙时隔得久：不等就去改修改时刻会撞上 FileNotFoundError
    world.stop_server()   # 在跑的尝试在它自己的单元里接着跑
    quiet_since = time.time() - LOG_QUIET_PINNED_SECONDS
    os.utime(log, (quiet_since, quiet_since))
    world.start_server(extra=quick_grace)
    try:
        summary = world.wait_for(task_id, lambda found: any((attempt.get("progress") or {}).get("log_quiet_seconds") is not None for attempt in found["attempts"]),
                                 "报日志多久没涨")
        quiet = next(attempt["progress"]["log_quiet_seconds"] for attempt in summary["attempts"] if (attempt.get("progress") or {}).get("log_quiet_seconds") is not None)
        shown = world.client(["status", str(task_id)]).stdout
    finally:
        open(release, "w").close()
    world.finished(task_id)
    if not LOG_QUIET_PINNED_SECONDS - 100 <= quiet <= LOG_QUIET_PINNED_SECONDS + 300 or "没涨" not in shown:
        raise SelftestFailure(f"日志最后写在 {LOG_QUIET_PINNED_SECONDS} 秒前、服务重启之后：日志多久没涨应当从日志的修改时刻算，实际 {quiet} 秒；status「{shown.strip()[-300:]}」",
                              world.server_log_lines())
    return f"服务重启之后在跑那一条的「日志多久没涨」从日志的修改时刻算：{quiet:.0f} 秒（修改时刻在 {LOG_QUIET_PINNED_SECONDS} 秒前）"


def cell_memory_cap_passed_to_command(world):
    key = "selftest-cap-to-command"
    world.seed_peak(key, GIBIBYTE)
    world.resources(local=(30, 30))   # 头一段没有第二台：本机那一条在本机起（派得到第二台的照余量挑台）
    world.start_server()
    seen = os.path.join(world.root, "seen-cap")
    local_summary = world.finished(world.submit(["bash", "-c", f'printf "%s" "${{{lib.MEMORY_CAP_VARIABLE}:-unset}}" > "$1"', "show-cap", seen], extra={lib.KEY_VARIABLE: key}))
    local_attempt = (local_summary["attempts"] or [{}])[0]
    try:
        with open(seen, encoding="utf-8") as handle:
            local_seen = handle.read()
    except OSError:
        local_seen = "（命令没写出来）"
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：这一条去第二台
    peer_id = world.submit(["bash", "-c", "exit 0", "peer-cap"], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
    peer_summary = world.finished(peer_id, timeout=PLACEMENT_WAIT_SECONDS)
    peer_attempt = (peer_summary["attempts"] or [{}])[0]
    calls = world.calls()
    marks = [index for index, line in enumerate(calls) if line.startswith(f"peer task={peer_id} ")]
    peer_words = calls[marks[0] + 1] if marks and marks[0] + 1 < len(calls) and calls[marks[0] + 1].startswith("peer-words ") else ""
    problems = []
    if local_attempt.get("host") != "local" or local_seen != str(local_attempt.get("memory_cap")):
        problems.append(f"本机那一条：命令环境里的 {lib.MEMORY_CAP_VARIABLE} 应当等于这一次的上限 {local_attempt.get('memory_cap')}（字节），实际读到「{local_seen}」、{describe(local_summary)}")
    if peer_attempt.get("host") != "peer" or f"{lib.MEMORY_CAP_VARIABLE}={peer_attempt.get('memory_cap')}" not in peer_words.split():
        problems.append(f"第二台那一条：交给 multi-host-run.sh 的 env 那一段应当带 {lib.MEMORY_CAP_VARIABLE}={peer_attempt.get('memory_cap')}，"
                        f"实际 {describe(peer_summary)}、那一段「{peer_words[:400]}」")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"本机那一条命令读到 {lib.MEMORY_CAP_VARIABLE}={local_seen}（这一次的上限），第二台那一条经 env 那一段带过去 {peer_attempt.get('memory_cap')}"


def cell_source_repository_passed_to_command(world):
    """每次尝试经环境 TASK_SERVER_SOURCE_REPOSITORY 把提交方那一份仓的根交给命令：本机那一条读到服务所在仓的绝对路径，第二台那一条经 env 那一段带过去同一个值
    （任务 2653、2654：agent-cost.py 在第二台照脚本所在的树拼会话目录、退 2）。"""
    world.resources(local=(30, 30))   # 头一段没有第二台：本机那一条在本机起（派得到第二台的照余量挑台）
    world.start_server()
    expected = os.path.realpath(world.repo)
    seen = os.path.join(world.root, "seen-source")
    local_summary = world.finished(world.submit(["bash", "-c", f'printf "%s" "${{{lib.SOURCE_REPOSITORY_VARIABLE}:-unset}}" > "$1"', "show-source", seen]))
    try:
        with open(seen, encoding="utf-8") as handle:
            local_seen = handle.read()
    except OSError:
        local_seen = "（命令没写出来）"
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：这一条去第二台
    peer_id = world.submit(["bash", "-c", "exit 0", "peer-source"], cwd=world.repo)
    peer_summary = world.finished(peer_id, timeout=PLACEMENT_WAIT_SECONDS)
    calls = world.calls()
    marks = [index for index, line in enumerate(calls) if line.startswith(f"peer task={peer_id} ")]
    peer_words = calls[marks[0] + 1] if marks and marks[0] + 1 < len(calls) and calls[marks[0] + 1].startswith("peer-words ") else ""
    problems = []
    if local_seen != expected:
        problems.append(f"本机那一条：命令环境里的 {lib.SOURCE_REPOSITORY_VARIABLE} 应当是 {expected}，实际读到「{local_seen}」")
    if (peer_summary["attempts"] or [{}])[0].get("host") != "peer" or f"{lib.SOURCE_REPOSITORY_VARIABLE}={expected}" not in peer_words.split():
        problems.append(f"第二台那一条：交给 multi-host-run.sh 的 env 那一段应当带 {lib.SOURCE_REPOSITORY_VARIABLE}={expected}，实际 {describe(peer_summary)}、那一段「{peer_words[:400]}」")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"本机与第二台那一条都拿到 {lib.SOURCE_REPOSITORY_VARIABLE}={expected}"


def peer_env_words(world, task_id):
    """假 multi-host-run.sh 给这条任务记的 peer-words 那一行（交给第二台的 env 那一段连同命令）；没有交空串。"""
    calls = world.calls()
    marks = [index for index, line in enumerate(calls) if line.startswith(f"peer task={task_id} ")]
    return calls[marks[0] + 1] if marks and marks[0] + 1 < len(calls) and calls[marks[0] + 1].startswith("peer-words ") else ""


def cell_attempt_label_passed_to_command(world):
    """每次尝试经环境 TASK_SERVER_ATTEMPT_LABEL 把这一次的尝试标签交给命令（两台都给）：两台的 scope 照它对到尝试上（task_server.py 的 server_pending、
    memory_peaks_lib.py 文件头「在跑的 scope」）。本机那一条读到的是它这一次尝试的 label，第二台那一条经 env 那一段带过去同一个值。"""
    world.resources(local=(30, 30))   # 头一段没有第二台：本机那一条在本机起
    world.start_server()
    seen = os.path.join(world.root, "seen-label")
    local_summary = world.finished(world.submit(["bash", "-c", f'printf "%s" "${{{lib.ATTEMPT_LABEL_VARIABLE}:-unset}}" > "$1"', "show-label", seen]))
    try:
        with open(seen, encoding="utf-8") as handle:
            local_seen = handle.read()
    except OSError:
        local_seen = "（命令没写出来）"
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：这一条去第二台
    peer_id = world.submit(["bash", "-c", "exit 0", "peer-label"], cwd=world.repo)
    peer_summary = world.finished(peer_id, timeout=PLACEMENT_WAIT_SECONDS)
    peer_words = peer_env_words(world, peer_id)
    local_label = (local_summary["attempts"] or [{}])[0].get("label")
    peer_attempt = (peer_summary["attempts"] or [{}])[0]
    problems = []
    if not local_label or local_seen != local_label:
        problems.append(f"本机那一条：命令环境里的 {lib.ATTEMPT_LABEL_VARIABLE} 应当是这一次尝试的 label「{local_label}」，实际读到「{local_seen}」")
    if peer_attempt.get("host") != "peer" or f"{lib.ATTEMPT_LABEL_VARIABLE}={peer_attempt.get('label')}" not in peer_words.split():
        problems.append(f"第二台那一条：交给 multi-host-run.sh 的 env 那一段应当带 {lib.ATTEMPT_LABEL_VARIABLE}=<这一次尝试的 label>，"
                        f"实际 {describe(peer_summary)}、那一段「{peer_words[:400]}」")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"本机与第二台那一条都拿到 {lib.ATTEMPT_LABEL_VARIABLE}=<这一次尝试的 label>"


def cell_card_labels_by_address(world):
    """卡的台标签是 HOSTS 里那一台的地址（task_server.py 文件头 ②「GPU」）：多机配置只点名第二台（SELFTEST_PEER_ADDRESS）的一张卡、本机放得下却没卡
    ⇒ GPU 任务派到第二台，尝试记的卡、清场包装拿到的与 env 那一段带过去的 GPU_CARDS_BY_PRIORITY 都是原样那一项（地址形态）；
    再换成台标签不在 HOSTS 里的一张 ⇒ 当一张卡都没点名，记 run-condition-unmet、没起、说明点名那一项。"""
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64, cards=PEER_CARD_ZERO, peer_gpu=[(0, 16384)])
    world.start_server()
    task_id = world.submit(["cargo", "test", "--features", "gpu", "fake-peer-exit=0"], cwd=world.repo)
    summary = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    attempt = first_attempt(summary)
    calls = world.calls()
    quiesce = world.calls_of(task_id, "quiesce")
    marks = [index for index, line in enumerate(calls) if line.startswith(f"peer task={task_id} ")]
    peer_words = calls[marks[0] + 1] if marks and marks[0] + 1 < len(calls) and calls[marks[0] + 1].startswith("peer-words ") else ""
    problems = []
    if attempt.get("host") != "peer" or attempt.get("card") != PEER_CARD_ZERO or summary["task"].get("outcome") != "succeeded":
        problems.append(f"只点名第二台的卡 {PEER_CARD_ZERO}：GPU 任务应当派到第二台、记这张卡、跑成，实际 {describe(summary)}")
    if not quiesce or not quiesce[0].endswith(f" cards={PEER_CARD_ZERO}"):
        problems.append(f"清场包装拿到的 GPU_CARDS_BY_PRIORITY 应当是 {PEER_CARD_ZERO}，实际调用「{quiesce}」")
    if f"GPU_CARDS_BY_PRIORITY={PEER_CARD_ZERO}" not in peer_words.split():
        problems.append(f"env 那一段应当带 GPU_CARDS_BY_PRIORITY={PEER_CARD_ZERO}，实际「{peer_words[:400]}」")
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64, cards=OUTSIDE_HOSTS_CARD, peer_gpu=[(0, 16384)])
    unmet_id = world.submit(["cargo", "test", "--features", "gpu", "fake-peer-exit=0"], cwd=world.repo)
    unmet = world.finished(unmet_id, timeout=PLACEMENT_WAIT_SECONDS)
    shown = json.dumps(unmet, ensure_ascii=False)
    if unmet["task"].get("outcome") != "run-condition-unmet" or unmet["attempts"] or OUTSIDE_HOSTS_CARD not in shown or "不是 HOSTS 里的一项" not in shown:
        problems.append(f"台标签 {OUTSIDE_HOSTS_CARD} 不在 HOSTS 里：应当当一张卡都没点名、记 run-condition-unmet、没起、说明点名那一项，实际 {shown[:600]}")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"台标签 {PEER_CARD_ZERO} 的卡派到第二台、交给命令的是地址形态；台标签不在 HOSTS 里的当没点名、记 run-condition-unmet 并点名"


def cell_peer_admission_marked(world):
    """派第二台的那一次（task_server.py 文件头 ③「第二台」）：服务只给那一层 multi-host-run.sh --peer 设 lib.PEER_ADMITTED_VARIABLE=<任务号>.<第几次>，
    env 那一段不带到第二台那条命令；本机那一次的命令看不到它。"""
    world.resources(local=(30, 30))   # 头一段没有第二台：本机那一条在本机起
    world.start_server()
    seen = os.path.join(world.root, "seen-peer-admission")
    local_summary = world.finished(world.submit(["bash", "-c", f'printf "%s" "${{{lib.PEER_ADMITTED_VARIABLE}-没设}}" > "$1"', "show-admission", seen]))
    try:
        with open(seen, encoding="utf-8") as handle:
            local_seen = handle.read()
    except OSError:
        local_seen = "（命令没写出来）"
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：这一条去第二台
    peer_id = world.submit(["bash", "-c", "exit 0", "peer-admission"], cwd=world.repo)
    peer_summary = world.finished(peer_id, timeout=PLACEMENT_WAIT_SECONDS)
    calls = world.calls()
    peer_lines = world.calls_of(peer_id, "peer")
    marks = [index for index, line in enumerate(calls) if line.startswith(f"peer task={peer_id} ")]
    peer_words = calls[marks[0] + 1] if marks and marks[0] + 1 < len(calls) and calls[marks[0] + 1].startswith("peer-words ") else ""
    problems = []
    if first_attempt(local_summary).get("host") != "local" or local_seen != "没设":
        problems.append(f"本机那一条的命令不该看到 {lib.PEER_ADMITTED_VARIABLE}，实际读到「{local_seen}」、{describe(local_summary)}")
    if first_attempt(peer_summary).get("host") != "peer" or not peer_lines or not peer_lines[0].endswith(f" admitted={peer_id}.1"):
        problems.append(f"第二台那一条：multi-host-run.sh 那一层应当拿到 {lib.PEER_ADMITTED_VARIABLE}={peer_id}.1，实际 {describe(peer_summary)}、调用「{peer_lines}」")
    if lib.PEER_ADMITTED_VARIABLE in peer_words:
        problems.append(f"env 那一段不该带 {lib.PEER_ADMITTED_VARIABLE}（第二台那条命令里再带 --peer 要被拒），实际「{peer_words[:400]}」")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"第二台那一层拿到 {lib.PEER_ADMITTED_VARIABLE}={peer_id}.1、env 那一段不带它，本机那一条的命令看不到它"


def cell_shared_roots_passed_to_command(world):
    """每次尝试经环境 SINGLEFS_DATA_ROOT、SINGLEFS_WORK_ROOT 交两个共享根（task_server_lib.py 文件头「共享根」；问题 73：第二台的 ~ 不是两台共享的那个目录）：
    本机那一条读到服务那一侧的真路径，第二台那一条经 env 那一段带过去同样的值（不在共享根下的提交时就拒，由 shared-roots-outside-refused 判）。"""
    world.resources(local=(30, 30))   # 头一段没有第二台：本机那一条在本机起（派得到第二台的照余量挑台）
    world.start_server()
    expected = {lib.DATA_ROOT_VARIABLE: os.path.realpath(world.data_root), lib.WORK_ROOT_VARIABLE: os.path.realpath(world.work_root)}
    seen = os.path.join(world.root, "seen-shared-roots")
    show = f'printf "%s\\n%s" "${{{lib.DATA_ROOT_VARIABLE}:-unset}}" "${{{lib.WORK_ROOT_VARIABLE}:-unset}}" > "$1"'
    local_summary = world.finished(world.submit(["bash", "-c", show, "show-shared-roots", seen]))
    try:
        with open(seen, encoding="utf-8") as handle:
            local_seen = dict(zip(lib.SHARED_ROOT_VARIABLES, handle.read().split("\n")))
    except OSError:
        local_seen = {}
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：这一条去第二台
    peer_id = world.submit(["bash", "-c", "exit 0", "peer-shared-roots"], cwd=world.repo)
    peer_summary = world.finished(peer_id, timeout=PLACEMENT_WAIT_SECONDS)
    peer_words = peer_env_words(world, peer_id).split()
    problems = []
    if (local_summary["attempts"] or [{}])[0].get("host") != "local" or local_seen != expected:
        problems.append(f"本机那一条：命令环境里的两个共享根应当是 {expected}，实际读到 {local_seen or '（命令没写出来）'}、{describe(local_summary)}")
    missing = [f"{name}={value}" for name, value in expected.items() if f"{name}={value}" not in peer_words]
    if (peer_summary["attempts"] or [{}])[0].get("host") != "peer" or missing:
        problems.append(f"第二台那一条：交给 multi-host-run.sh 的 env 那一段应当带 {'、'.join(missing) or '两个共享根'}，实际 {describe(peer_summary)}、那一段「{' '.join(peer_words)[:400]}」")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"本机与第二台那一条都拿到 {expected}"


def cell_peer_disk_floor(world):
    """派第二台之前看它的盘余量（task_server_lib.py 文件头「挑机器」的「第二台盘余量」；问题 72：剩 353M 时拷树写满）：服务另起的 df 报的余量不到下限 → 第二台用不了，
    本机放不下就排着、排队原因写「第二台盘只剩」、写一条 peer-unavailable 告警（余量的数变了不另写）、/health 的 peer 一栏写明；报告里带的余量够了 → 照派第二台、/health 认回。"""
    floor = SERVER.DISK_FLOOR_BYTES
    low_values = (floor - GIBIBYTE, floor - 2 * GIBIBYTE)
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_df=f"     Avail\n{low_values[0]}\n")   # 本机放不下 4G：只派得到第二台
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "disk-floor"], cwd=world.repo)
    world.next_pass()
    world.next_pass()
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_df=f"     Avail\n{low_values[1]}\n")   # 余量又少了一点：原因照新的写，不另告警
    world.next_pass()
    world.next_pass()
    problems = []
    held = world.summary(task_id)
    why = held.get("waiting_for") or ""
    if held["task"].get("state") != "queued" or held["attempts"] or "第二台盘只剩" not in why:
        problems.append(f"第二台盘余量不到下限：应当不派第二台、本机放不下就排着、排队原因写「第二台盘只剩」，实际 {held['task'].get('state')}、"
                        f"尝试 {len(held['attempts'])} 次、排队原因「{why}」")
    alerted = [alert for alert in world.alerts() if alert.get("kind") == "peer-unavailable" and "第二台盘只剩" in alert.get("line", "")]
    if len(alerted) != 1:
        problems.append(f"盘余量不到下限应当恰好写一条 peer-unavailable 告警（余量的数变了不另写），实际 {len(alerted)} 条：{alerted}")
    _, health = world.api("GET", "/health")
    peer = health.get("peer") or {}
    shown = SERVER.gibibytes_text(low_values[1])
    if peer.get("available") is not False or f"第二台盘只剩 {shown}" not in str(peer.get("why")):
        problems.append(f"/health 的 peer 一栏应当写用不了、原因「第二台盘只剩 {shown}」，实际 {peer}")
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=floor + GIBIBYTE)   # 报告里带的余量够了
    done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    if done["task"].get("outcome") != "succeeded" or (done["attempts"] or [{}])[-1].get("host") != "peer":
        problems.append(f"盘余量够了应当照派第二台跑完，实际 {describe(done)}")
    back = health_until(world, lambda health: (health.get("peer") or {}).get("available") is True, STATE_TIMEOUT_SECONDS)
    if (back.get("peer") or {}).get("available") is not True:
        problems.append(f"盘余量够了 /health 的 peer 一栏应当认回用得了，实际 {back.get('peer')}")
    if problems:
        raise SelftestFailure("第二台盘余量：" + "；".join(problems), world.server_log_lines())
    return (f"df 报的余量不到下限 {SERVER.gibibytes_text(floor)}：不派第二台、排队原因与 /health 写「第二台盘只剩」、告警一条；报告里带的余量够了照派第二台、/health 认回")


def idle_cpu_scope(world):
    """CPU 读得到、一直不涨的假 scope（判疑似卡住的那几格用）。"""
    scope = os.path.join(world.root, "idle-scope")
    os.makedirs(scope, exist_ok=True)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    return scope


def suspect_lines(completed):
    return [line for line in completed.stdout.splitlines() if line.startswith(lib.SUSPECT_MARK + " ")]


def cell_suspected_stall_notified(world):
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01",
                              "TASK_SERVER_FAKE_SCOPE_DIRECTORY": idle_cpu_scope(world)})
    task_id = world.submit(["bash", "-c", 'for round in $(seq 1 60); do echo "tick $round"; sleep 0.25; done'])   # 日志一直在涨、CPU 不涨，约 15 秒
    first = world.client(["wait", str(task_id)], timeout=60)
    during = world.summary(task_id)
    shown = world.client(["status", str(task_id)])
    alerted = [alert for alert in world.alerts() if alert.get("kind") == "suspected-stall" and alert.get("task") == task_id]
    second = world.client(["wait", str(task_id)], timeout=90)
    problems = []
    if first.returncode != lib.SUSPECT_EXIT or not [line for line in suspect_lines(first) if line.startswith(f"{lib.SUSPECT_MARK} task={task_id} ")]:
        problems.append(f"日志在涨而 CPU 满窗口不涨：头一回 wait 应当打一行 {lib.SUSPECT_MARK} task={task_id} …、退 {lib.SUSPECT_EXIT}，"
                        f"实际退 {first.returncode}、「{first.stdout.strip()[-300:]}」")
    if during["task"].get("state") != "running" or not [attempt for attempt in during["attempts"] if attempt.get("state") == "running"]:
        problems.append(f"报了疑似卡住之后任务应当照跑（running），实际 {describe(during)}、状态 {during['task'].get('state')}")
    if "疑似卡住" not in shown.stdout:
        problems.append(f"status 应当写「疑似卡住」与依据，实际「{shown.stdout.strip()[-400:]}」")
    if not alerted:
        problems.append(f"应当记一条 suspected-stall 告警，实际告警 {world.alerts()}")
    if second.returncode != 0 or suspect_lines(second) or f"{lib.RESULT_MARK} task={task_id} type=succeeded " not in second.stdout:
        problems.append(f"再起 wait 应当拿到真结局（succeeded、退 0）、同一段不再报疑似，实际退 {second.returncode}、「{second.stdout.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("疑似卡住只通知：" + "；".join(problems), world.server_log_lines())
    return (f"日志在涨、CPU 满窗口不涨：头一回 wait 打 {lib.SUSPECT_MARK}、退 {lib.SUSPECT_EXIT}，任务照跑、status 写疑似卡住、记了告警；"
            "再起 wait 拿到 succeeded，同一段只报一次")


def cell_suspected_stall_card_busy_not_reported(world):
    world.resources(local=(100, 100), cards=LOCAL_CARD_ZERO, local_gpu=[(0, 16384)])
    write_executable(os.path.join(world.work, SUSPECT_GPU_SCRIPT_NAME), SUSPECT_GPU_SCRIPT)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01",
                              "TASK_SERVER_FAKE_SCOPE_DIRECTORY": idle_cpu_scope(world), "TASK_SERVER_NVIDIA_SMI": os.path.join(world.bin, "fake-nvidia-smi")})
    busy = world.submit(["bash", SUSPECT_GPU_SCRIPT_NAME, os.path.join(world.world, "card-busy-pids")])   # 卡上有它的计算负载
    busy_wait = world.client(["wait", str(busy)], timeout=60)
    idle = world.submit(["bash", SUSPECT_GPU_SCRIPT_NAME])   # 卡上没有它的负载
    idle_wait = world.client(["wait", str(idle)], timeout=60)
    world.client(["cancel", str(idle)])
    busy_summary = world.summary(busy)
    problems = []
    if not busy_summary["task"].get("gpu") or busy_wait.returncode != 0 or suspect_lines(busy_wait):
        problems.append(f"要卡的、CPU 不涨而卡上有它的计算负载：不该报疑似卡住，wait 应当拿到真结局退 0，"
                        f"实际认成{'要卡' if busy_summary['task'].get('gpu') else '不要卡'}、退 {busy_wait.returncode}、「{busy_wait.stdout.strip()[-300:]}」")
    if idle_wait.returncode != lib.SUSPECT_EXIT or not suspect_lines(idle_wait):
        problems.append(f"要卡的、CPU 不涨、卡上也没有它的负载：应当报疑似卡住（退 {lib.SUSPECT_EXIT}），实际退 {idle_wait.returncode}、「{idle_wait.stdout.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("要卡的疑似卡住看卡：" + "；".join(problems), world.server_log_lines())
    return f"要卡的 CPU 不涨：卡上有它的计算负载不报、跑完退 0；卡上没有它的负载报疑似卡住、退 {lib.SUSPECT_EXIT}"


def shown_local_pin(completed):
    """control pin-cap 打的两台此刻的钉值（lib.pinned_caps_text 的「本机 <数>M、第二台 <数>M」）里本机那一项的 MiB 数；没打出来交 None。"""
    found = re.search(r"本机 (\d+)M", completed.stdout)
    return int(found.group(1)) if found else None


def cell_pinned_cap_from_configuration(world):
    """钉值取多机配置（task_server.py 的 pinned_prediction 与 pin_value）：钉了之后起的那一次的上限就是 control pin-cap 报的此刻钉值；配置从 20G 改成 30G，
    pin-cap 报的钉值跟着多 10240M、起的那一次照它；撞了上限不放大、记 memory-cap-at-maximum；unpin-cap 之后照学到的记录（1536M）。"""
    world.set_slice_cap("20G")
    world.resources(local=(100, 100))
    probe = ["bash", "-c", "exit 0", "pinned-cap-probe"]
    hit = ["bash", "-c", "exit 0", "pinned-cap-hit", "fake-wrapper-exits=250,run"]
    world.seed_peak(PEAKS.command_key(probe, None), GIBIBYTE)   # 学到的峰值 1G：没钉时上限 1536M，钉了不顶掉钉值
    world.start_server()
    problems = []
    for command in (hit, probe):
        pinned = world.control(["pin-cap", "--", *command])
        if pinned.returncode != 0 or "钉上了" not in pinned.stdout:
            problems.append(f"control pin-cap 应当退 0、说钉上了，实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-300:]}」")
    before = shown_local_pin(pinned)   # 配置写 20G 时 pin-cap 报的本机钉值

    def first_attempt(command):
        summary = world.finished(world.submit(command))
        return summary, (summary["attempts"] or [{}])[0]

    summary, attempt = first_attempt(probe)
    shown = world.client(["status", str(summary["task"]["id"])])
    if before is None or attempt.get("memory_cap_text") != f"{before}M" or attempt.get("prediction") != lib.PINNED_HISTORY or "钉值" not in shown.stdout:
        problems.append(f"钉了的命令第一次尝试的上限应当是 pin-cap 报的此刻钉值（配置两台都写 20G 时报 {before}M、prediction {lib.PINNED_HISTORY}）、status 写钉值，"
                        f"实际 {describe(summary)}、prediction {attempt.get('prediction')}、status「{shown.stdout.strip()[:300]}」")
    world.set_slice_cap("30G")
    again = world.control(["pin-cap", "--", *probe])   # 已钉着：照退 0、报配置改了之后此刻的钉值
    after = shown_local_pin(again)
    if before is None or after is None or after - before != 10 * 1024:
        problems.append(f"配置从 20G 改成 30G，pin-cap 报的本机钉值应当跟着多 10240M，实际改前 {before}M、改后 {after}M（「{(again.stdout + again.stderr).strip()[-300:]}」）")
    summary, attempt = first_attempt(probe)
    if after is None or attempt.get("memory_cap_text") != f"{after}M":
        problems.append(f"配置改成 30G 之后钉了的命令应当照此刻的钉值起（{after}M），实际 {describe(summary)}")
    summary = world.finished(world.submit(hit))
    caps = [item.get("memory_cap_text") for item in summary["attempts"]]
    if after is None or caps != [f"{after}M"] or summary["task"].get("outcome") != "memory-cap-at-maximum" or summary["task"].get("exit_code") != 236:
        problems.append(f"钉了的命令撞了上限不放大：只有一次 {after}M、结局 memory-cap-at-maximum（退 236），实际上限 {caps}、{describe(summary)}")
    unpinned = world.control(["unpin-cap", "--", *probe])
    summary, attempt = first_attempt(probe)
    if unpinned.returncode != 0 or attempt.get("memory_cap_text") != "1536M" or attempt.get("prediction") == lib.PINNED_HISTORY:
        problems.append(f"unpin-cap 之后照学到的峰值定（1536M），实际 unpin-cap 退 {unpinned.returncode}、{describe(summary)}、prediction {attempt.get('prediction')}")
    if problems:
        raise SelftestFailure("钉值：" + "；".join(problems), world.server_log_lines())
    return (f"pin-cap 之后第一次上限就是 pin-cap 报的此刻钉值（{before}M）、改配置跟着变（{after}M）、撞上限不放大记 memory-cap-at-maximum；"
            "unpin-cap 之后照记录（1536M）")


def cell_pinned_cap_unreadable_refused(world):
    """钉了上限而多机配置的 SLICE_MEMORY_CAP_BY_HOST 此刻读不到（两台那一项都写法不对）：这条不起、结局 run-condition-unmet（退 78），说明与 wait 的 TASK_SERVER_REASON 点名读不到的原因与出路；
    学到的峰值不顶上来（task_server.py 的 pinned_prediction）。弄坏开关 pin-unreadable-predicted 下照改前照学到的记录起、退 0。"""
    world.resources(local=(100, 100))
    probe = ["bash", "-c", "exit 0", "pinned-cap-unreadable-probe"]
    world.seed_peak(PEAKS.command_key(probe, None), GIBIBYTE)   # 学到的峰值 1G：照改前退回记录会起成 1536M
    world.start_server()
    problems = []
    pinned = world.control(["pin-cap", "--", *probe])
    if pinned.returncode != 0 or "钉上了" not in pinned.stdout:
        problems.append(f"control pin-cap 应当退 0、说钉上了，实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-300:]}」")
    world.set_slice_cap("lots")   # 写法不对：服务现读读不出数
    task_id = world.submit(probe)
    summary = world.finished(task_id)
    task = summary["task"]
    note = task.get("note") or ""
    if (task.get("outcome") != "run-condition-unmet" or task.get("exit_code") != lib.RUN_CONDITION_EXIT or summary["attempts"] or world.calls_of(task_id)
            or lib.PIN_SOURCE not in note or "写法不对" not in note or "unpin-cap" not in note):
        problems.append(f"钉了而 {lib.PIN_SOURCE} 写法不对：应当一次都不起、结局 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}）、说明点名 {lib.PIN_SOURCE} 写法不对与 unpin-cap 的出路，"
                        f"实际 {describe(summary)}、内存包装起了 {len(world.calls_of(task_id))} 次、说明「{note[:300]}」")
    completed = world.client(["wait", str(task_id)])
    lines = completed.stdout.strip().splitlines()
    reason = next((line for line in lines if line.startswith(lib.REASON_MARK + " ")), "")
    if (completed.returncode != lib.RUN_CONDITION_EXIT or not lines or not lines[-1].startswith(f"{lib.RESULT_MARK} task={task_id} type=run-condition-unmet ")
            or "type=run-condition-unmet" not in reason or lib.PIN_SOURCE not in reason or "unpin-cap" not in reason):
        problems.append(f"wait 应当退 {lib.RUN_CONDITION_EXIT}、末行 type=run-condition-unmet、它前面的 {lib.REASON_MARK} 写读不到的原因与出路，"
                        f"实际退 {completed.returncode}、理由行「{reason[:300]}」、末行「{lines[-1] if lines else ''}」")
    if problems:
        raise SelftestFailure("钉值读不到：" + "；".join(problems), world.server_log_lines())
    return f"钉了而 {lib.PIN_SOURCE} 写法不对：一次都不起、结局 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}）、wait 的理由行点名读不到与 unpin-cap 的出路"


def cell_pinned_cap_by_host(world):
    """钉值按台（task_server.py 文件头 ②「内存」的钉值）：SLICE_MEMORY_CAP_BY_HOST 两台写不同的数（本机 20G、第二台 30G），钉值是那一项减那台的假余量 2G（pin_value、pinned_on_host）。
    只有本机时钉了的命令在本机起、上限 18432M；本机放不下、第二台放得下时在第二台起、上限 28672M（取第二台那一项）；control pin-cap 报两台各自的钉值。
    弄坏开关 pinned-cap-this-host-only 下两台都取本机那一项：第二台那一次也是 18432M；pinned-cap-at-slice-total 下照那一项记满：20480M、30720M。"""
    world.set_slice_caps(by_host_text("20G", "30G"))
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64)   # pin-cap 报两台各自的钉值：两台的余量都要量得到
    probe = ["bash", "-c", "exit 0", "pinned-cap-by-host-probe"]
    world.start_server()
    problems = []
    pinned = world.control(["pin-cap", "--", *probe])
    world.resources(local=(100, 100))   # 只有本机：钉了的那一条在本机起
    if pinned.returncode != 0 or "钉上了" not in pinned.stdout or f"本机 {PINNED_LOCAL_TEXT}、第二台 {PINNED_PEER_TEXT}" not in pinned.stdout:
        problems.append(f"control pin-cap 应当退 0、说钉上了、报两台各自的钉值（本机 {PINNED_LOCAL_TEXT}、第二台 {PINNED_PEER_TEXT}），"
                        f"实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-300:]}」")
    summary = world.finished(world.submit(probe))
    attempt = (summary["attempts"] or [{}])[0]
    if attempt.get("host") != "local" or attempt.get("memory_cap_text") != PINNED_LOCAL_TEXT or attempt.get("prediction") != lib.PINNED_HISTORY:
        problems.append(f"只有本机时钉了的命令应当在本机起、上限取本机那一项减余量 {PINNED_LOCAL_TEXT}（prediction {lib.PINNED_HISTORY}），实际 {describe(summary)}、"
                        f"台 {attempt.get('host')}、prediction {attempt.get('prediction')}")
    world.resources(local=(1, 100), peer=(100, 100), peer_cap_gibibytes=64)   # 本机只空 1G：钉了的那一条只放得进第二台
    summary = world.finished(world.submit(probe), timeout=PLACEMENT_WAIT_SECONDS)
    attempt = (summary["attempts"] or [{}])[0]
    if attempt.get("host") != "peer" or attempt.get("memory_cap_text") != PINNED_PEER_TEXT or attempt.get("prediction") != lib.PINNED_HISTORY:
        problems.append(f"本机放不下时钉了的命令应当在第二台起、上限取第二台那一项减余量 {PINNED_PEER_TEXT}（prediction {lib.PINNED_HISTORY}），实际 {describe(summary)}、"
                        f"台 {attempt.get('host')}、prediction {attempt.get('prediction')}")
    if problems:
        raise SelftestFailure("钉值按台：" + "；".join(problems), world.server_log_lines())
    return f"钉值按台：在本机起取本机那一项减余量（{PINNED_LOCAL_TEXT}）、在第二台起取第二台那一项减余量（{PINNED_PEER_TEXT}），control pin-cap 报两台各自的钉值"


def by_host_written_wrong(world, value, wanted, probe_name):
    """按台的键写坏的那两格共用：钉了的命令起之前把 SLICE_MEMORY_CAP_BY_HOST 换成 value（服务已经起着），这一条应当一次都不起、结局 run-condition-unmet，
    说明里有 wanted（共用取法 multi_host_addresses.py 的 by_host_values 判的那一句）与 unpin-cap 的出路。→ 问题列表。"""
    world.resources(local=(100, 100))
    probe = ["bash", "-c", "exit 0", probe_name]
    world.start_server()
    problems = []
    pinned = world.control(["pin-cap", "--", *probe])
    if pinned.returncode != 0 or "钉上了" not in pinned.stdout:
        problems.append(f"control pin-cap 应当退 0、说钉上了，实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-300:]}」")
    world.set_slice_caps(value)
    task_id = world.submit(probe)
    summary = world.finished(task_id)
    note = summary["task"].get("note") or ""
    if (summary["task"].get("outcome") != "run-condition-unmet" or summary["task"].get("exit_code") != lib.RUN_CONDITION_EXIT or summary["attempts"]
            or world.calls_of(task_id) or wanted not in note or "unpin-cap" not in note):
        problems.append(f"{lib.PIN_SOURCE}={value}：钉了的命令应当一次都不起、结局 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}）、说明里有「{wanted}」与 unpin-cap 的出路，"
                        f"实际 {describe(summary)}、内存包装起了 {len(world.calls_of(task_id))} 次、说明「{note[:400]}」")
    return problems


def cell_by_host_missing_host_refused(world):
    """按台的键少一台（task_server_lib.py 文件头「路径」按台的三个键）：SLICE_MEMORY_CAP_BY_HOST 只写了本机那一项 ⇒ 判写坏、点名缺的第二台，钉了的命令不起。
    弄坏开关 by-host-missing-accepted 下少了也照收（说明里没有「少了 <第二台> 那一台」）。"""
    problems = by_host_written_wrong(world, f"{SELFTEST_LOCAL_ADDRESS}=20G", f"少了 {SELFTEST_PEER_ADDRESS} 那一台", "by-host-missing-host-probe")
    if problems:
        raise SelftestFailure("按台的键少一台：" + "；".join(problems), world.server_log_lines())
    return f"{lib.PIN_SOURCE} 少了第二台那一项：判写坏、点名缺的那一台，钉了的命令一次都不起（run-condition-unmet）"


def cell_by_host_outside_address_refused(world):
    """按台的键多一项不在 HOSTS 里的地址（task_server_lib.py 文件头「路径」按台的三个键）：SLICE_MEMORY_CAP_BY_HOST 多写一项 192.0.2.99 ⇒ 判写坏、点名那一项，
    钉了的命令不起。弄坏开关 by-host-outside-accepted 下多了也照收（说明里没有「192.0.2.99 不在 HOSTS 里」）。"""
    outside = "192.0.2.99"
    problems = by_host_written_wrong(world, f"{by_host_text('20G', '20G')} {outside}=20G", f"{outside} 不在 {lib.HOSTS_KEY} 里", "by-host-outside-address-probe")
    if problems:
        raise SelftestFailure("按台的键多一项不在 HOSTS 里的地址：" + "；".join(problems), world.server_log_lines())
    return f"{lib.PIN_SOURCE} 多一项不在 {lib.HOSTS_KEY} 里的地址：判写坏、点名那一项，钉了的命令一次都不起（run-condition-unmet）"


def cell_shard_host_is_address(world):
    """分片的一片拿到它所在那一台在 HOSTS 里的地址（task_server_lib.py 文件头「分片」②的 TASK_SERVER_SHARD_HOST）：两台都放得下、一件分两片，
    每一片每一次尝试记下的 TASK_SERVER_SHARD_HOST 是那一次的台在 HOSTS 里的地址（本机是 SELFTEST_LOCAL_ADDRESS、第二台是 SELFTEST_PEER_ADDRESS），不是 local / peer。
    弄坏开关 shard-host-role-passed 下照改前给 local / peer。"""
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64)
    spec = world.write_spec("spec-shard-host-address.json", {"shards": [{"label": "第一片"}, {"label": "第二片"}]})
    world.start_server()
    summary = world.finished(world.submit(spec, cwd=world.repo))
    addresses = {"local": SELFTEST_LOCAL_ADDRESS, "peer": SELFTEST_PEER_ADDRESS}
    seen, wrong = [], []
    for index, child in sorted(shard_children(summary).items()):
        for attempt in child["attempts"]:
            rows = world.sharded_calls("shard", index=index, attempt=attempt.get("number"))
            given = [row.get("host") for row in rows]
            seen.append(f"片 {index} 第 {attempt.get('number')} 次 {attempt.get('host')}：{given}")
            if given != [addresses.get(attempt.get("host"))]:
                wrong.append(seen[-1])
    if summary["task"].get("outcome") != "succeeded" or not seen or wrong:
        raise SelftestFailure(f"分片的每一次尝试拿到的 TASK_SERVER_SHARD_HOST 应当是那一次的台在 HOSTS 里的地址（本机 {SELFTEST_LOCAL_ADDRESS}、第二台 {SELFTEST_PEER_ADDRESS}），"
                              f"整件成了；实际整件 {summary['task'].get('outcome')}、对不上的 {wrong}、全部 {seen}", world.server_log_lines())
    return f"分片的 {len(seen)} 次尝试拿到的 TASK_SERVER_SHARD_HOST 都是那一次的台在 HOSTS 里的地址（{'；'.join(seen)}）"


def cell_gate_stages_sent_to_peer(world):
    """点到 .claude/gate.d/ 的门禁阶段派得到第二台（没有只派本机的目录）；带判定库的那一种提交时就拒，由 verdict-library-family-refused 判。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：派得到第二台的去第二台
    world.start_server()
    plain = world.submit(["bash", "-c", "exit 0", ".claude/gate.d/doc-text.sh", "--check", "clock-times"], cwd=world.repo)
    plain_summary = world.finished(plain, timeout=PLACEMENT_WAIT_SECONDS)
    plain_host = (plain_summary["attempts"] or [{}])[0].get("host")
    if plain_host != "peer" or plain_summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"点到 .claude/gate.d/ 的门禁阶段应当派第二台跑完，实际 {describe(plain_summary)}", world.server_log_lines())
    return "点到 .claude/gate.d/ 的门禁阶段派到第二台跑完"


def cell_admission_booked_at_cap(world):
    key = "selftest-booked-at-cap"
    world.seed_peak(key, 20 * GIBIBYTE)   # 预测 20G、上限 30G
    world.resources(local=(50, 50))
    world.start_server()
    releases = [os.path.join(world.root, f"release-{number}") for number in (1, 2)]
    first = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", releases[0]], extra={lib.KEY_VARIABLE: key})
    try:
        running = world.running(first)
        second = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", releases[1]], extra={lib.KEY_VARIABLE: key})
        # 看几秒第二条起没起（起了的话队就空了，排队线程不再记「pass」那一行，不拿 next_pass 等）
        waiting = world.summary(second)
        deadline = time.monotonic() + ADMISSION_WATCH_SECONDS
        while time.monotonic() < deadline and not waiting["attempts"]:
            time.sleep(POLL_SECONDS)
            waiting = world.summary(second)
    finally:
        open(releases[0], "w").close()
    open(releases[1], "w").close()
    later = world.finished(second)
    booked = running["attempts"][0].get("booked")
    if waiting["task"].get("state") != "queued" or waiting["attempts"] or booked != 30 * GIBIBYTE or later["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"两条各预测 20G（上限 30G）、本机 50G：头一条按上限记账 30G，第二条（再记 30G 就超过 50G）应当排着、头一条放了才起；"
                              f"实际头一条记账 {booked} 字节、第二条先是 {waiting['task'].get('state')}、尝试 {len(waiting['attempts'])} 次、最后 {describe(later)}",
                              world.server_log_lines())
    return "两条各预测 20G（上限 30G）在 50G 的台上：头一条记账 30G（上限）、第二条排着，头一条放了才起"


def cell_shard_cap_fitted_to_room(world):
    """分片的片记满上限放不下、预测放得下时，上限压到放得下的量（task_server_lib.py 文件头「资源记录」的准入）：两片各预测 20G（上限 30G）、本机 54G，两片同时跑，上限之和不超过 54G；
    不是分片的同样两条照旧排着（admission-booked-at-cap）。弄坏开关 shard-cap-not-fitted 下第二片排着，这一格红。"""
    key = "selftest-shard-cap-fitted"
    for index in (0, 1):
        world.seed_peak(lib.shard_child_key(key, "shard", index, 2), 20 * GIBIBYTE)
    world.resources(local=(54, 54), peer_down="自证：第二台此刻用不了")
    release = os.path.join(world.root, "release-fitted")
    spec = world.write_spec("spec-fitted.json", {"shards": [{"block": release}, {"block": release}]})
    world.start_server()
    task_id = world.submit(spec, cwd=world.repo, extra={lib.KEY_VARIABLE: key})
    try:
        summary = world.wait_for(task_id, lambda summary: len(running_shards(summary)) == 2, "两片同时在本机跑起来")
        caps = sorted(int(child["attempts"][0].get("memory_cap") or 0) for child in shard_children(summary).values())
    finally:
        open(release, "w").close()
    finished = world.finished(task_id)
    if caps != [(54 - SELFTEST_MARGIN_GIBIBYTES - 30) * GIBIBYTE, 30 * GIBIBYTE] or finished["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"两片各预测 20G、上限 30G、本机 54G、余量 {SELFTEST_MARGIN_GIBIBYTES}G：第一片上限 30G，第二片压到总上限减余量减第一片（上限之和恰好等于总上限减余量）、整件成了；实际上限 {caps}、整件 {describe(finished)}", world.server_log_lines())
    return f"两片各预测 20G、本机 54G：上限 {[cap // GIBIBYTE for cap in caps]}G，同时跑、之和不超过总上限"


SHARD_KEY_BASES = ("selftest-shard-keys", "bash scripts/crash/crash-point-replay-shard-run.sh --merged-log merged.log")   # shard-keys-kept-apart：交了键的、没交键按命令取的


def cell_shard_keys_kept_apart(world):
    """分片各步的命令键（lib.shard_child_key）过了键的换算（memory_peaks_lib.py 文件头「键」，服务取键经它）：仍是这件换算过的键加那一段，
    计划、各片、合并两两不同；这件的键本身不再变的，换算前后逐字相同（seed_peak 按它记的峰值服务读得到）。
    弄坏开关 shard-key-position-folded 下片号写成 <片号>/<片数>、被折成〈文件〉，各片落一个键，这一格红。"""
    steps = (("plan", 0, 0), ("shard", 0, 3), ("shard", 1, 3), ("shard", 2, 3), ("merge", 0, 0))
    problems = []
    for base in SHARD_KEY_BASES:
        folded_base = PEAKS.command_key([], base)
        texts = [lib.shard_child_key(base, *step) for step in steps]
        keys = [PEAKS.command_key([], text) for text in texts]
        unchanged = folded_base != base or keys == texts
        if len(set(keys)) != len(steps) or not all(key.startswith(folded_base + " ") for key in keys) or not unchanged:
            problems.append(f"这件的键「{base}」（换算后「{folded_base}」）：各步原文 {texts}，换算后 {keys}")
    if problems:
        raise SelftestFailure("分片各步的命令键应当过了键的换算仍各是各的、是这件换算过的键加那一段，这件的键不变的原文也不变；" + "；".join(problems))
    return f"{len(SHARD_KEY_BASES)} 种这件的键：计划、三片、合并的命令键换算之后两两不同，都是这件的键加那一段"


def cell_health_answers_while_resources_slow(world):
    world.resources(local=(100, 100))
    paused = os.path.join(world.root, "placement-paused")
    world.start_server(extra={"TASK_SERVER_SELFTEST_PLACEMENT_PAUSED": paused, "TASK_SERVER_SELFTEST_SLOW_RESOURCES_SECONDS": str(SLOW_RESOURCES_SECONDS)})
    open(paused, "w").close()
    task_id = world.submit(["bash", "-c", "exit 0"])
    socket_path = os.path.join(world.runtime, lib.SOCKET_NAME)
    answers, health, deadline = [], {}, time.monotonic() + 20
    while time.monotonic() < deadline:
        started = time.monotonic()
        health = lib.call_server(socket_path, "GET", "/health", timeout=10)[1]
        answers.append(time.monotonic() - started)
        if STARVED_HEALTH_TEXT in (health.get("problem") or "") and len(answers) >= HEALTH_PROBES:
            break
        time.sleep(POLL_SECONDS)
    os.remove(paused)
    summary = world.finished(task_id)
    median = sorted(answers)[len(answers) // 2] if answers else None
    if (median is None or median >= HEALTH_ANSWER_SECONDS or max(answers) >= SLOW_RESOURCES_SECONDS or STARVED_HEALTH_TEXT not in (health.get("problem") or "")
            or summary["task"].get("outcome") != "succeeded"):
        raise SelftestFailure(f"余量每读一次要 {SLOW_RESOURCES_SECONDS} 秒、排头判能起却不起：/health 应当不等余量（每次都快于读一次余量，中位数在 {HEALTH_ANSWER_SECONDS} 秒内）、"
                              f"照样报「{STARVED_HEALTH_TEXT}」，接着派之后跑完；实际问了 {len(answers)} 次、中位数 {median}、最慢 {max(answers or [0]):.2f} 秒、最后一次 {health}、"
                              f"{describe(summary)}", world.server_log_lines())
    return (f"余量每读一次 {SLOW_RESOURCES_SECONDS} 秒：/health 问了 {len(answers)} 次，中位数 {median:.2f} 秒、最慢 {max(answers):.2f} 秒，"
            f"排头放得下却不起照样报「{health.get('problem')}」")


ADMISSION_WATCH_SECONDS = 3   # admission-booked-at-cap 看第二条起没起看这么久（一轮排队 0.1 秒）
SLOW_RESOURCES_SECONDS = 3     # health-answers-while-resources-slow 的假余量每读一次睡这么久
HEALTH_ANSWER_SECONDS = 1.0    # 那一格 /health 答话的中位数要短于它（库试写的提交在机器忙时偶尔慢过它，最慢那一次只要求快于读一次余量）
HEALTH_PROBES = 5              # 那一格至少问这么多次


E174_COMMAND = ["python3", "scripts/experiment-apparatus/device-code-key.py", "run", ".", "--binary", "e174-lease-drop-wait-memory-return", "--feature", "e174-gpu",
                "--entry", "e7_index_bench::run_child_on_the_card", "--", "run", "--segment", "first",
                "--limit-jobs", "2"]   # E174（借据丢掉时等待与显存回还）实验页上的那一条（不带外面的清场与内存包装）


def cell_python_entry_gpu_recognized(world):
    world.resources(local=(100, 100), cards=LOCAL_CARD_ZERO, local_gpu=[(0, 16384)])
    world.start_server()
    feature = E174_COMMAND.index("e174-gpu")
    cases = ((E174_COMMAND[:feature] + ["e162-block-stores"] + E174_COMMAND[feature + 1:], 0),    # 同一入口不带 gpu 特性
             (E174_COMMAND[:feature - 1] + E174_COMMAND[feature + 1:] + ["--feature", "gpu"], 0),   # gpu 只写在 -- 之后（交给装置的参数）
             (["python3", "scripts/experiment-apparatus/device-code-key.py", "key", ".", "--binary", "x", "--feature=e174-gpu"], 0),   # 只取键的子命令
             (E174_COMMAND, 1))   # 要卡的放最后：同一命令键用过卡的记录不挡前面几条
    problems, submitted = [], []
    for words, expected in cases:
        task_id, gpu = submitted_gpu(world, [*words, "fake-wrapper-exits=0"])
        submitted.append(task_id)
        if gpu != expected:
            problems.append(f"{' '.join(words)} 应当回显 gpu={expected}，实际 gpu={gpu}")
    for task_id in submitted:
        world.finished(task_id)
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return "E174 那条 device-code-key.py run --feature e174-gpu 回显 gpu=1；同一入口不带 gpu 特性、gpu 只在 -- 之后、只取键的子命令回显 gpu=0"


def cell_gpu_argument_declaration_recognized(world):
    """按脚本文件头的声明认要卡的参数（task_server_lib.py 文件头「GPU」的 # task-server-gpu-arguments:）：bash 起的与 python3 起的假脚本各一份，
    第一个非选项参数是声明的词就要卡，别的参数不要卡；没写那一行的脚本照不要卡。"""
    world.resources(local=(100, 100), cards=LOCAL_CARD_ZERO, local_gpu=[(0, 16384)])
    world.start_server()
    scripts = {"declared-gpu.sh": f"#!/usr/bin/env bash\n{lib.DECLARED_GPU_ARGUMENTS_MARK} card-segment card-drift\nexit 0\n",
               "declared_gpu.py": f"#!/usr/bin/env python3\n{lib.DECLARED_GPU_ARGUMENTS_MARK} run\n",
               "undeclared.sh": "#!/usr/bin/env bash\n# 文件头没有要卡的声明\nexit 0\n"}
    for name, text in scripts.items():
        with open(os.path.join(world.work, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    cases = ((["bash", "declared-gpu.sh", "cpu-segment"], 0), (["bash", "declared-gpu.sh", "--list-items"], 0),
             (["python3", "declared_gpu.py", "key"], 0), (["bash", "undeclared.sh", "card-segment"], 0),
             (["bash", "declared-gpu.sh", "card-segment"], 1), (["bash", "declared-gpu.sh", "--quick", "card-drift"], 1),
             (["python3", "declared_gpu.py", "run"], 1))
    problems, submitted = [], []
    for words, expected in cases:
        task_id, gpu = submitted_gpu(world, [*words, "fake-wrapper-exits=0"])
        submitted.append(task_id)
        if gpu != expected:
            problems.append(f"{' '.join(words)} 应当回显 gpu={expected}，实际 gpu={gpu}")
    for task_id in submitted:
        world.finished(task_id)
    if problems:
        raise SelftestFailure("文件头声明要卡的参数：" + "；".join(problems), world.server_log_lines())
    return (f"{len(cases)} 条：声明了 card-segment、card-drift 的 bash 脚本与声明了 run 的 python3 脚本，第一个非选项参数是声明的词回显 gpu=1；"
            "别的参数、--list-items、没写声明的脚本回显 gpu=0")


LEFTOVER_GONE_SECONDS = 10.0   # leftover-processes-stopped：结局记下之后至多等这么久看留下的那个孙进程没了


def process_gone(process_id):
    """进程号不在了（/proc 里没有它，或只剩僵尸）。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8", errors="replace") as handle:
            return handle.read().rsplit(")", 1)[1].split()[0] == "Z"
    except (OSError, IndexError):
        return True


def cell_leftovers_only_in_own_unit(world):
    """attempt_runner 停留下的进程只在自己待在这一次尝试的单元里时做（attempt_runner.py 的 stop_unit_leftovers）：自己的 cgroup 是起服务那一方的 scope
    （没起成单元：弄坏开关 no-detach）时，里面的进程一个不停。假 cgroup 根下放一份 cgroup.procs，点名一个真起的 sleep；不碰 systemd，开关打开也只杀这一个 sleep。"""
    import importlib.util
    spec = importlib.util.spec_from_file_location("attempt_runner_under_test", lib.RUNNER_SCRIPT)
    runner = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(runner)
    group = "/user.slice/singlefs-memory-cap-1-2.scope"
    os.makedirs(world.root + group, exist_ok=True)
    sleeper = subprocess.Popen(["sleep", "60"], stdin=subprocess.DEVNULL)
    try:
        with open(world.root + group + "/cgroup.procs", "w", encoding="utf-8") as handle:
            handle.write(f"{os.getpid()}\n{sleeper.pid}\n")
        runner.CGROUP_ROOT, runner.cgroup_of = world.root, (lambda process_id: group)
        result = runner.stop_unit_leftovers("singlefs-task-selftest-1-1.service", lib.broken("leftovers-outside-own-unit"))
        time.sleep(0.3)
        if sleeper.poll() is not None or result.get("processes"):
            raise SelftestFailure(f"自己的 cgroup 不是这一次尝试的单元时一个进程都不该停，实际停了 {result.get('processes')}、sleep {'已退' if sleeper.poll() is not None else '还在'}", [])
        result = runner.stop_unit_leftovers("singlefs-memory-cap-1-2.scope")
        sleeper.wait(timeout=15)
        if str(sleeper.pid) not in result.get("processes", []):
            raise SelftestFailure(f"自己就在点名的单元里时应当停掉留下的进程，实际 {result}", [])
    finally:
        if sleeper.poll() is None:
            sleeper.kill()
        sleeper.wait()
    return "自己的 cgroup 不是这一次尝试的单元时不停里面的进程；在自己的单元里照停"


LEFTOVERS_ERROR_EXIT = 3   # leftovers-error-isolated：那一条命令自己的退出码（不是 0，看得出照原样带回）
LEFTOVERS_ERROR_RUNNER = """import importlib.util, sys
specification = importlib.util.spec_from_file_location("attempt_runner_under_test", sys.argv[1])
runner = importlib.util.module_from_spec(specification)
specification.loader.exec_module(runner)
def raising(*arguments, **options):
    raise RuntimeError("自证：stop_unit_leftovers 抛错")
runner.stop_unit_leftovers = raising
sys.exit(runner.main(sys.argv[2:]))
"""   # leftovers-error-isolated：载入 attempt_runner.py、把 stop_unit_leftovers 换成一抛就错的，再照原样起 main


def cell_leftovers_error_isolated(world):
    """attempt_runner 停单元里留下的进程时抛了没料到的异常（records/2026-11-03-task-server与prune-server深入对攻审核.md「TS-02」；attempt_runner.py 的 main）：
    result.json 照写、exit_code 是命令自己的、unit_leftovers 是 null，attempt_runner 照命令的退出码退，日志里记一行告警。
    另起一个 python 载入 attempt_runner.py、把 stop_unit_leftovers 换成一抛就错的，照说明起 bash -c 'exit 3'（不起 systemd 单元）。
    弄坏开关 leftovers-error-uncaught 下（说明里 leftovers_error_uncaught）异常照改前抛出去、result.json 写不出，这一格红。"""
    directory = os.path.join(world.work, "leftovers-error")
    os.makedirs(directory, exist_ok=True)
    wrapper = os.path.join(world.work, "leftovers-error-runner.py")
    with open(wrapper, "w", encoding="utf-8") as handle:
        handle.write(LEFTOVERS_ERROR_RUNNER)
    log = os.path.join(directory, "attempt.log")
    specification_path = os.path.join(directory, "spec.json")
    with open(specification_path, "w", encoding="utf-8") as handle:
        json.dump({"argv": ["bash", "-c", f"exit {LEFTOVERS_ERROR_EXIT}"], "cwd": world.work, "environment": dict(os.environ), "log": log, "directory": directory,
                   "fast": True, "unit": "singlefs-task-selftest-1-1.service", "leftovers_error_uncaught": lib.broken("leftovers-error-uncaught")}, handle)
    completed = subprocess.run([sys.executable, "-B", wrapper, lib.RUNNER_SCRIPT, specification_path], stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)
    result, logged = None, ""
    with contextlib.suppress(OSError, ValueError):
        with open(os.path.join(directory, "result.json"), encoding="utf-8") as handle:
            result = json.load(handle)
    with contextlib.suppress(OSError):
        with open(log, encoding="utf-8", errors="replace") as handle:
            logged = handle.read()
    warned = "停单元里留下的进程时出了事（RuntimeError: 自证：stop_unit_leftovers 抛错）" in logged
    if (completed.returncode != LEFTOVERS_ERROR_EXIT or not isinstance(result, dict) or result.get("exit_code") != LEFTOVERS_ERROR_EXIT
            or result.get("unit_leftovers") is not None or not warned):
        raise SelftestFailure(f"停留下的进程时抛错：result.json 应当照写、exit_code {LEFTOVERS_ERROR_EXIT}、unit_leftovers 是 null，attempt_runner 退 {LEFTOVERS_ERROR_EXIT}、日志里记一行告警；"
                              f"实际退 {completed.returncode}、result.json {'写不出' if result is None else result}、日志里{'有' if warned else '没有'}那一行告警",
                              (completed.stdout + completed.stderr).splitlines()[-12:])
    return f"stop_unit_leftovers 抛错时 result.json 照写（exit_code {LEFTOVERS_ERROR_EXIT}、unit_leftovers 是 null），attempt_runner 照命令退 {LEFTOVERS_ERROR_EXIT}、日志记了告警"


def cell_leftover_processes_stopped(world):
    """尝试正常结束之后单元里还留着命令起的孙进程（task_server.py 的 stop_leftovers）：服务停掉它，结局说明写留下了几个、停了；
    不留进程的那一条说明里没有这一句。"""
    world.start_server()
    child_file = os.path.join(world.root, "leftover-child")
    left = world.finished(world.submit(["bash", "-c", 'sleep 120 & echo "$!" > "$1"; exit 0', "leftover", child_file]))
    clean = world.finished(world.submit(["bash", "-c", "exit 0", "clean"]))
    problems = []
    try:
        with open(child_file, encoding="utf-8") as handle:
            child = handle.read().strip()
    except OSError:
        child = ""
    patience = world.patience(LEFTOVER_GONE_SECONDS)
    while child and not process_gone(child) and patience.waiting():
        time.sleep(POLL_SECONDS)
    note = left["task"].get("note") or ""
    if left["task"].get("outcome") != "succeeded" or "还留着进程" not in note or "服务已停掉" not in note:
        problems.append(f"命令退 0 而留下一个后台 sleep：应当照 succeeded 收尾、结局说明写留下的进程与停了，实际 {describe(left)}、说明「{note}」")
    if not child or not process_gone(child):
        problems.append(f"留下的那个孙进程（{child or '没记下进程号'}）结局记下之后应当不在了")
    if "还留着进程" in (clean["task"].get("note") or ""):
        problems.append(f"没留下进程的那一条说明里不该有这一句，实际「{clean['task'].get('note')}」")
    if problems:
        raise SelftestFailure("留下的进程：" + "；".join(problems), world.server_log_lines())
    return "命令退出之后单元里留下的孙进程由服务停掉、结局说明写明；没留下的不写"


LONG_RUN_SELFTEST_SECONDS = "1"   # long-run-failure-noted：自证里「跑得久」的界（TASK_SERVER_SELFTEST_LONG_RUN_SECONDS）


def cell_long_run_failure_noted(world):
    """跑超过时限而失败、又没登记续跑的，结局说明写「这条跑了 N 分钟、没有续跑：重交会从头跑」（task_server.py 的 finished_preface；时限在自证里缩成 1 秒）：
    登记了续跑的（文件头 # task-server-resumable:）、没到时限的、跑成了的都不写。"""
    world.start_server(extra={"TASK_SERVER_SELFTEST_LONG_RUN_SECONDS": LONG_RUN_SELFTEST_SECONDS})
    with open(os.path.join(world.work, "resumable.sh"), "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_RESUMABLE_MARK} segment\nsleep 1.5\nexit 3\n")
    cases = (("跑了 1.5 秒、退 3、没登记续跑", ["bash", "-c", "sleep 1.5; exit 3"], True),
             ("登记了续跑", ["bash", "resumable.sh", "segment"], False),
             ("没到时限", ["bash", "-c", "exit 3"], False),
             ("跑成了", ["bash", "-c", "sleep 1.5; exit 0"], False))
    problems = []
    for name, command, expected in cases:
        summary = world.finished(world.submit(command, cwd=world.work))
        note = summary["task"].get("note") or ""
        if ("没有续跑：重交会从头跑" in note) != expected:
            problems.append(f"{name}：结局说明{'应当' if expected else '不该'}写「没有续跑：重交会从头跑」，实际「{note}」、{describe(summary)}")
    if problems:
        raise SelftestFailure("跑得久而失败的说明：" + "；".join(problems), world.server_log_lines())
    return "跑过时限、失败、没登记续跑的结局说明写重交会从头跑；登记了续跑、没到时限、跑成了的不写"


ALTERNATION_WHOLE_COUNT = 3     # whole-and-fragment-alternate 交几条整片（峰值 30G、上限与记账 45G）
ALTERNATION_FRAGMENT_COUNT = 12  # 那一格交几条碎片（没记录、各记 4G）
ALTERNATION_THRESHOLD_BYTES = int((50 * GIBIBYTE - 2 * GIBIBYTE) * SERVER.WHOLE_FRACTION)   # 50G 的台：可用上限 48G 的 85%（40.8G）
FAILED_PEAK_BYTES = 3 * GIBIBYTE   # failed-peak-floor：判红那一次量到的峰值
FAILED_PEAK_PINNED_TEXT = "4608M"  # 3G × 1.5
MEMORY_FLOOR_SEED_BYTES = 247 * MEBIBYTE   # retry-cap-not-floored：记下的那一次峰值（任务 2419 那一种；× 1.5 是 371M，低于 4G）
MEMORY_FLOOR_RECORDED_TEXT = "371M"        # 那一格头一次的上限：247M × 1.5 按 MiB 向上取整，有记录的不兜
MEMORY_FLOOR_RETRY_TEXT = "557M"           # 那一格撞了重排那一次的上限：371M × 1.5 按 MiB 向上取整，不兜到 4G
RECORDED_SMALL_PEAK_BYTES = 200 * MEBIBYTE  # recorded-cap-not-floored：记下的那一次峰值（批 91 定的样本：有记录、峰值 200M）
RECORDED_SMALL_CAP_TEXT = "300M"            # 200M × 1.5 按 MiB 向上取整
CAP_HIT_RERUN_BIG_SEED_BYTES = 4096 * MEBIBYTE        # cap-hit-rerun-from-record 大的一条记下的峰值：× 1.5 是 6144M，撞了之后再交 × 1.5 是 9216M
CAP_HIT_RERUN_BIG_TEXTS = ("6144M", "9216M")          # 大的一条头一遍撞上限的上限、再交一遍第一次尝试的上限
CAP_HIT_RERUN_SMALL_TEXTS = (MEMORY_FLOOR_RECORDED_TEXT, MEMORY_FLOOR_RETRY_TEXT)   # 小的一条（记 247M）头一遍撞上限的上限（247M × 1.5）、再交一遍第一次尝试的上限（撞过的 371M × 1.5）
CAP_HIT_RERUN_VERIFY_PEAK_BYTES = 4096 * MEBIBYTE     # 带验证命令的一条：验证命令那把键记下的峰值
CAP_HIT_RERUN_VERIFIED_TEXTS = ("6515M", "15917M")    # 带验证命令的一条头一遍的上限 (247M + 4096M) × 1.5 = 6514.5M 向上取整；再交一遍：(6515M + 4096M) × 1.5 = 15916.5M 向上取整
CAP_HIT_RERUN_SEEDED_HIT_BYTES = 247 * MEBIBYTE       # 记录里只有一次撞上限（没人改工作区的命令）：撞的上限，× 1.5 是 371M，不兜到 4G
GAP_WHOLE_PEAK_BYTES = 63 * GIBIBYTE // 2  # 插空那三格的整片：上限与记账 47.25G（不小于界 40.8G）；整片之外照样空着的那一份 48G − 47.25G = 0.75G 放不下来插空的 1.5G，
                                           # 那三格插空只看时长（task_server.py 的 Alternation.gap_fill 的内存那一支不放）
GAP_HOLDER_PEAK_BYTES = 10 * GIBIBYTE      # 那三格挡路的两条碎片与封组的那一条：各记 15G；两条装进一组 30G，再放一条就 45G、超过界封组
GAP_PROBE_PEAK_BYTES = GIBIBYTE            # 那三格来插空的碎片：各记 1.5G（封组时照公式还剩 50G − 2G − 30G = 18G，放得下）
GAP_BLOCKER_ESTIMATE_SECONDS = 600         # sealed-gap-takes-short-task 与 sealed-gap-holds-longer-task 两条挡路碎片记下的用时
GAP_SHORT_ESTIMATE_SECONDS = 1             # 来插空的短任务记下的用时
GAP_LONG_ESTIMATE_SECONDS = 6000           # sealed-gap-holds-longer-task 的长任务：超过挡路时长
GAP_UNESTIMATED_LONG_SECONDS = 3600        # unestimated-blocker-counted-by-elapsed 的长任务：这一格跑完之前挡路的已跑到不了它
GAP_OVERRUN_ESTIMATE_SECONDS = 0.2         # 那一格一条挡路碎片记下的用时：起来一下就跑过了，照已跑算
GAP_ELAPSED_WAIT_SECONDS = 20.0            # 那一格等短任务插空起来的时限（挡路的已跑到 1 秒就该起）
STARTED_AT_ROUNDING_SECONDS = 0.001        # 尝试的 started_at 记到毫秒（task_server.py 的 start_attempt）：比两条起的先后差时让出这一截
GAP_ADMITTED_TEXT = "插空："                # task_server.py 的 Alternation.gap_fill 插得进时那一句的开头
GAP_REFUSED_TEXT = "没插空："               # 插不进时接在封组那一句后面的开头
DRAINED_WHOLE_PEAK_BYTES = 30 * GIBIBYTE      # drained-sealed-group-reopens 的整片：上限与记账 45G（不小于界 40.8G）
DRAINED_FRAGMENT_PEAK_BYTES = 10 * GIBIBYTE   # 那一格的碎片：上限与记账 15G；两条装进一组是 30G，再放一条就 45G、超过界封组
DRAINED_FREE_GIBIBYTES = 25                   # 那一格腾空之后本机的空闲：放得下碎片（预测 10G 加余量 2G），放不下整片（预测 30G 加余量 2G）
SEALED_GROUP_TEXT = "封组，在等这台腾空起整片任务"   # task_server.py 的 Alternation.blocked 写进排队原因的那几个字


def started_kinds(summaries):
    """各条第一次尝试按记录号（起的先后）排好的类别串：[(类别, 任务号)]；没起的不在里面。"""
    started = [(attempt_order(summary), kind, summary["task"]["id"]) for kind, summary in summaries if attempt_order(summary) is not None]
    return [(kind, task_id) for _, kind, task_id in sorted(started)]


def runs_of(kinds):
    """把类别串压成连着的段：[(类别, 条数)]。"""
    runs = []
    for kind, _ in kinds:
        if runs and runs[-1][0] == kind:
            runs[-1] = (kind, runs[-1][1] + 1)
        else:
            runs.append((kind, 1))
    return runs


def cell_whole_and_fragment_alternate(world):
    """一台 50G（可用上限 48G，界 40.8G）：先交 3 条整片（峰值 30G，上限与记账 45G）、再交 12 条碎片（没记录、各记 4G），都没满时限、一齐放出来。
    起的次序照尝试记录号判（不比秒数）：一组碎片（这一组的记账不超过界）→ 一条整片 → 一组碎片 → 整片（碎片交完了，剩下的整片接着起；task_server.py 的 Alternation）。"""
    world.resources(local=(50, 50), hold=True)
    world.seed_peak("selftest-alternate-whole", 30 * GIBIBYTE)
    world.start_server()
    tasks = [("整片", world.submit(["bash", "-c", "exit 0", f"whole-{number}"], extra={lib.KEY_VARIABLE: "selftest-alternate-whole"}))
             for number in range(ALTERNATION_WHOLE_COUNT)]
    tasks += [("碎片", world.submit(["bash", "-c", "exit 0", f"fragment-{number}"])) for number in range(ALTERNATION_FRAGMENT_COUNT)]
    world.resources(local=(50, 50))   # 一齐放出来
    world.api("POST", "/wake")
    summaries = [(kind, world.finished(task_id, timeout=120)) for kind, task_id in tasks]
    failed = [f"{kind} {describe(summary)}" for kind, summary in summaries if summary["task"].get("outcome") != "succeeded"]
    kinds = started_kinds(summaries)
    runs = runs_of(kinds)
    fragment = UNRECORDED_DEFAULT_PINNED_TEXT
    groups = [count for kind, count in runs if kind == "碎片"]
    problems = []
    if failed:
        problems.append(f"都应当跑完，实际没成的 {failed}")
    if [kind for kind, _ in runs[:4]] != ["碎片", "整片", "碎片", "整片"] or runs[1][1] != 1 or len(kinds) != len(tasks):
        problems.append(f"起的次序应当是「一组碎片 → 一条整片 → 一组碎片 → 整片」（后面没碎片了，剩下的整片接着起），实际按记录号压成段 {runs}")
    if any(count * 4 * GIBIBYTE > ALTERNATION_THRESHOLD_BYTES for count in groups):
        problems.append(f"每一组碎片的记账（各 {fragment}）应当不超过界 {ALTERNATION_THRESHOLD_BYTES / GIBIBYTE:.1f}G，实际各组条数 {groups}")
    if problems:
        raise SelftestFailure("整片与碎片交替：" + "；".join(problems), [f"{kind} 任务 {task_id}" for kind, task_id in kinds] + world.server_log_lines())
    return f"一台 50G：起的次序按记录号压成段 {runs}（每组碎片不超过 {ALTERNATION_THRESHOLD_BYTES / GIBIBYTE:.1f}G）"


def cell_whole_starts_beside_running_fragment(world):
    """一台 50G（可用上限 48G、界 40.8G）上一条碎片（4G）长跑着，整片（峰值 28G，上限与记账 42G）放得下就起，不等那台腾空
    （任务 2461 留成孤儿的两小时里本机只用 2G、10 条排着等「腾空」）。"""
    world.resources(local=(50, 50))
    world.seed_peak("selftest-whole-beside", 28 * GIBIBYTE)
    world.start_server()
    release = os.path.join(world.root, "release-beside")
    fragment = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "block", release])
    world.running(fragment)
    # 整片也占着到放行为止：「exit 0」的整片起了就收尾，等「跑起来」的那一下可能一次都没看到它在跑、等满 60 秒判红（任务 4593 里这一格等满了 60 秒，尝试已记 1 次）
    whole_release = os.path.join(world.root, "release-whole-beside")
    whole = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "whole-beside", whole_release], extra={lib.KEY_VARIABLE: "selftest-whole-beside"})
    try:
        summary = world.running(whole)   # 碎片还在跑时整片就起
    finally:
        for path in (release, whole_release):
            open(path, "w").close()
    booked = (summary["attempts"] or [{}])[0].get("booked") or 0
    finished = {name: world.finished(task_id) for name, task_id in (("碎片", fragment), ("整片", whole))}
    problems = []
    if booked < ALTERNATION_THRESHOLD_BYTES:
        problems.append(f"整片应当记到界以上（{ALTERNATION_THRESHOLD_BYTES / GIBIBYTE:.1f}G），实际记 {booked / GIBIBYTE:.1f}G")
    if any(found["task"].get("outcome") != "succeeded" for found in finished.values()):
        problems.append(f"都应当跑完，实际 {({name: describe(found) for name, found in finished.items()})}")
    if problems:
        raise SelftestFailure("整片不等腾空：" + "；".join(problems), world.server_log_lines())
    return "碎片长跑着、整片放得下就起，不等那台腾空"


def cell_drained_sealed_group_reopens(world):
    """封了组的那台腾空了、整片仍放不下（task_server.py 的 Alternation.blocked）：一台 50G（界 40.8G）上先交一条整片（峰值 30G，上限与记账 45G），
    再交三条碎片（峰值 10G，上限与记账 15G）：两条装进一组（30G）起着，余下那条再放就超过界、封组排着。然后本机空闲降到 25G（放得下碎片、放不下整片）、
    装进组的两条跑完、服务在本机一条都不在跑：封组排着的那条应当照起，整片接着排着。弄坏开关 sealed-group-held-when-drained 下封组排着的那条一直排着、排队原因写封组在等这台腾空。"""
    world.resources(local=(50, 50), hold=True)
    world.seed_peak("selftest-drained-whole", DRAINED_WHOLE_PEAK_BYTES)
    world.seed_peak("selftest-drained-fragment", DRAINED_FRAGMENT_PEAK_BYTES)
    world.start_server()
    release = os.path.join(world.root, "release-drained")
    fragment_key = {lib.KEY_VARIABLE: "selftest-drained-fragment"}
    whole = world.submit(["bash", "-c", "exit 0", "drained-whole"], extra={lib.KEY_VARIABLE: "selftest-drained-whole"})
    held = [world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "block", release], extra=fragment_key) for _ in range(2)]
    sealed = world.submit(["bash", "-c", "exit 0", "drained-fragment"], extra=fragment_key)
    world.resources(local=(50, 50))   # 一齐放出来：整片让一组碎片先起，余下那条碎片封组
    world.api("POST", "/wake")
    problems = []
    try:
        for task_id in held:
            world.running(task_id)
        world.next_pass()
        before = world.summary(sealed)
        reason = before.get("waiting_for") or ""
        if before["attempts"] or SEALED_GROUP_TEXT not in reason:
            problems.append(f"装进组的两条碎片在跑时余下那条应当封组排着（排队原因带「{SEALED_GROUP_TEXT}」），实际尝试 {len(before['attempts'])} 次、排队原因「{reason}」")
        world.resources(local=(DRAINED_FREE_GIBIBYTES, 50))   # 腾空之后整片（45G）照样放不下，碎片（15G）放得下
    finally:
        open(release, "w").close()
    for task_id in held:
        world.finished(task_id)
    world.next_pass()
    world.next_pass()
    drained, waiting_whole = world.summary(sealed), world.summary(whole)
    reason = drained.get("waiting_for") or ""
    if not drained["attempts"]:
        problems.append(f"本机腾空、整片放不下时封组排着的碎片应当照起，实际状态 {drained['task'].get('state')}、排队原因「{reason}」")
    if waiting_whole["attempts"]:
        problems.append(f"本机空闲 {DRAINED_FREE_GIBIBYTES}G 时整片（记 45G）应当排着，实际起了：{describe(waiting_whole)}")
    if problems:
        raise SelftestFailure("封了组的台腾空：" + "；".join(problems), world.server_log_lines())
    world.resources(local=(50, 50))
    summaries = {name: world.finished(task_id) for name, task_id in (("封组排着的碎片", sealed), ("整片", whole))}
    if any(summary["task"].get("outcome") != "succeeded" for summary in summaries.values()):
        raise SelftestFailure(f"放开之后都应当跑完，实际 {({name: describe(summary) for name, summary in summaries.items()})}", world.server_log_lines())
    return f"封了组的台腾空、整片放不下：封组排着的碎片照起（记录号 {attempt_order(summaries['封组排着的碎片'])}），整片等到放得下才起"


def cell_ended_attempt_settled_after_restart(world):
    """最新尝试已记成 ended（memory-cap-hit）、任务却还 running（任务 2461：定去向半途抛错留下的孤儿，本机因此两小时算有任务在跑）：
    服务起来对账时补收尾，照撞上限的规矩上限 ×1.5 重排、跑完。"""
    world.resources(local=(50, 50))
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "orphan"], extra={lib.KEY_VARIABLE: "selftest-orphan"})
    world.finished(task_id)
    world.stop_server()
    world.rewrite_signed_row("tasks", task_id, lambda record: record.update(state="running", outcome="", exit_code=None, finished_at=None))
    attempt_ids = [record["id"] for record, _ in world.signed_rows("attempts") if record.get("task_id") == task_id]
    world.rewrite_signed_row("attempts", attempt_ids[-1],
                             lambda record: record.update(outcome="memory-cap-hit", exit_code=250, memory_cap=GIBIBYTE, memory_cap_text="1024M"))
    world.start_server()
    summary = world.finished(task_id)
    attempts = summary["attempts"]
    if summary["task"].get("outcome") != "succeeded" or len(attempts) != 2 or attempts[1].get("memory_cap_text") != "1536M":
        raise SelftestFailure(f"服务起来应当补收尾：按撞上限 ×1.5（1536M）重排、第 2 次跑完 succeeded，实际 {describe(summary)}", world.server_log_lines())
    return "最新尝试 ended、任务 running 的孤儿：对账补收尾，1536M 重排、跑完"


def cell_aged_tasks_do_not_hold_fitting_tasks(world):
    """满时限的放不下时不挡、不留（task_server.py 文件头 ②「满时限」）：本机 50G 被一条碎片占着 40.5G、只剩 9.5G（照公式 50G − 2G − 40.5G = 7.5G），
    满时限的整片（记 45G）与满时限的碎片（记约 10G）都放不下；后来的没满时限的小任务（没记录，记 4G）排两轮之内照起，那两条还排着；占着的放了之后都跑完。
    弄坏开关 backfill-disabled 下排在最前的满时限整片放不下就挡住后面整队，小任务排着。"""
    world.resources(local=(50, 50))
    world.seed_peak("selftest-unheld-occupant", 27 * GIBIBYTE)    # 上限与记账 40.5G：碎片（界 40.8G）
    world.seed_peak("selftest-unheld-whole", 30 * GIBIBYTE)       # 上限与记账 45G：整片，放不下
    world.seed_peak("selftest-unheld-fragment", 6827 * MEBIBYTE)  # 上限与记账 10241M：碎片，放不下
    world.start_server()
    release = os.path.join(world.root, "release-occupant")
    occupant = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "block", release], extra={lib.KEY_VARIABLE: "selftest-unheld-occupant"})
    world.running(occupant)
    whole = world.submit(["bash", "-c", "exit 0", "unheld-whole"], extra={lib.KEY_VARIABLE: "selftest-unheld-whole"})
    fragment = world.submit(["bash", "-c", "exit 0", "unheld-fragment"], extra={lib.KEY_VARIABLE: "selftest-unheld-fragment"})
    backdate_past_aged(world, [whole, fragment])
    world.start_server()
    try:
        small = world.submit(["bash", "-c", "exit 0", "unheld-small"])
        world.next_pass()
        world.next_pass()
        waiting = {name: world.summary(task_id) for name, task_id in (("小任务", small), ("满时限的整片", whole), ("满时限的碎片", fragment))}
    finally:
        open(release, "w").close()
    summaries = {name: world.finished(task_id) for name, task_id in (("占着的", occupant), ("满时限的整片", whole), ("满时限的碎片", fragment), ("小任务", small))}
    problems = []
    if not waiting["小任务"]["attempts"]:
        problems.append(f"满时限的两条放不下时，放得下的小任务应当排两轮之内照起，实际还排着，排队原因「{waiting['小任务'].get('waiting_for')}」")
    for name in ("满时限的整片", "满时限的碎片"):
        if waiting[name]["attempts"]:
            problems.append(f"{name}本机只剩 9.5G 时应当还排着（这一格的前提），实际起了 {len(waiting[name]['attempts'])} 次")
    failed = {name: describe(summary) for name, summary in summaries.items() if summary["task"].get("outcome") != "succeeded"}
    if failed:
        problems.append(f"占着的放了之后都应当跑完，实际没成的 {failed}")
    if problems:
        raise SelftestFailure("满时限的放不下时不挡、不留：" + "；".join(problems), world.server_log_lines())
    return "满时限的整片与碎片都放不下时不挡、不留：放得下的小任务排两轮之内起了，那两条排着；占着的放了之后都跑完"


def cell_cpu_gate_holds_busy_host(world):
    """本机内存够、负载 200（32 核 × 3 = 96 放不下再加一条的线程）：任务排着、排队原因写「CPU：」；负载降到 0 之后照常跑完（task_server.py 的 cpu_problem）。"""
    world.resources(local=(50, 50), local_cpu={"load1": 200.0, "procs_running": 150, "cpus": 32})
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "cpu-gate"])
    world.next_pass()
    world.next_pass()
    waiting = world.summary(task_id)
    world.resources(local=(50, 50), local_cpu={"load1": 0.0, "procs_running": 1, "cpus": 32})
    summary = world.finished(task_id)
    reason = waiting.get("waiting_for") or ""
    if waiting["attempts"] or "CPU：" not in reason:
        raise SelftestFailure(f"负载 200 的本机上应当排着、排队原因写「CPU：」，实际尝试 {len(waiting['attempts'])} 次、排队原因「{reason}」", world.server_log_lines())
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"负载降下来之后应当跑完，实际 {describe(summary)}", world.server_log_lines())
    return f"负载 200 时排队原因写「{reason[:100]}」，降下来之后跑完"


def cell_unstarted_slice_uses_configured_total(world):
    """本机 slice 还没起（报告里 slice_total 是估的 30G）、多机配置里本机那一项 50G：记过峰值 40G 的命令照 50G 的总上限判、跑完，
    不按估的 30G 判永远放不下（task_server.py 的 unstarted_slices_filled）。"""
    world.set_slice_cap("50G")
    world.resources(local=(50, 30), local_slice_unstarted=True)
    world.seed_peak("selftest-unstarted-slice", 40 * GIBIBYTE)
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "unstarted-slice"], extra={lib.KEY_VARIABLE: "selftest-unstarted-slice"}))
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"slice 还没起、配置 50G、峰值 40G：应当照 50G 判、跑完，实际 {describe(summary)}", world.server_log_lines())
    return f"slice 还没起时照配置的 50G 判，峰值 40G 的命令跑完（{describe(summary)[:80]}）"


def cell_down_host_waits_not_never(world):
    """本机 slice 20G、第二台此刻用不了：记过峰值 30G 的命令只有第二台放得下，应当排着、不判永远放不下；第二台回来（50G）之后派过去跑完（task_server.py 的 place）。"""
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")
    world.seed_peak("selftest-down-host", 30 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "down-host"], extra={lib.KEY_VARIABLE: "selftest-down-host"}, cwd=world.repo)
    world.next_pass()
    world.next_pass()
    waiting = world.summary(task_id)
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    summary = world.finished(task_id)
    if waiting["task"].get("outcome") or waiting["attempts"]:
        raise SelftestFailure(f"第二台用不了时只有它放得下的任务应当排着，实际 {describe(waiting)}、排队原因「{waiting.get('waiting_for')}」", world.server_log_lines())
    if summary["task"].get("outcome") != "succeeded" or (summary["attempts"] and summary["attempts"][0].get("host") != "peer"):
        raise SelftestFailure(f"第二台回来之后应当派过去跑完，实际 {describe(summary)}", world.server_log_lines())
    return f"第二台用不了时排着（「{(waiting.get('waiting_for') or '')[:80]}」），回来之后派过去跑完"


def cell_card_fault_replaced_on_another_card(world):
    """本机两张卡，0 号卡清场判坏：第一次派到 0 号卡的那一条不判 card-fault 收尾，换 1 号卡重排、跑完；之后交的一条直接派 1 号卡
    （0 号卡停用 CARD_QUARANTINE_SECONDS 秒，task_server.py 的 quarantined_cards 与 cards_in_service）；写一条 card-fault 告警。"""
    world.resources(local=(100, 100), cards=f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE}", local_gpu=[(0, 16384), (1, 16384)])
    world.start_server()
    first = world.finished(world.submit(["cargo", "test", "--features", "gpu", f"fake-quiesce=card-fault-on:{LOCAL_CARD_ZERO}"]))
    second = world.finished(world.submit(["cargo", "test", "--features", "gpu", f"fake-quiesce=card-fault-on:{LOCAL_CARD_ZERO}", "second"]))
    cards = [attempt.get("card") for attempt in first["attempts"]]
    problems = []
    if first["task"].get("outcome") != "succeeded" or cards != [LOCAL_CARD_ZERO, LOCAL_CARD_ONE]:
        problems.append(f"坏卡上的那一条应当换 {LOCAL_CARD_ONE} 重排、跑完，实际 {describe(first)}、各次的卡 {cards}")
    if second["task"].get("outcome") != "succeeded" or [attempt.get("card") for attempt in second["attempts"]] != [LOCAL_CARD_ONE]:
        problems.append(f"0 号卡停用着，之后那一条应当直接派 {LOCAL_CARD_ONE}，实际 {describe(second)}")
    if not any(alert.get("kind") == "card-fault" for alert in world.alerts()):
        problems.append(f"应当写一条 card-fault 告警，实际 {world.alerts()}")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"坏卡上的那一条换 {LOCAL_CARD_ONE} 跑完，之后的直接派它"


def card_fault_reason_line(world, task_id):
    """task wait 末行之前那一行 TASK_SERVER_REASON（打回的理由；拿不到交空串）。"""
    lines = world.client(["wait", str(task_id)]).stdout.strip().splitlines()
    return lines[-2] if len(lines) >= 2 and lines[-2].startswith(lib.REASON_MARK) else ""


def cell_card_fault_quarantines_named_card(world):
    """本机两张卡，分到 0 号卡（分到的那张）的那一条，清场每次都点名 1 号卡坏（坏卡清单 faulty-cards=127.0.0.1:1）：停用的是 1 号卡、0 号卡照常可派——
    各次尝试都在 0 号卡上，重排满 CARD_FAULT_RETRIES 次照 card-fault 收尾；告警与结局句（TASK_SERVER_REASON 的 why=）点名 1 号卡；
    之后交的一条直接派 0 号卡跑完（task_server.py 的 finish_attempt、settle_attempt 与 cards_in_service）。"""
    world.resources(local=(100, 100), cards=f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE}", local_gpu=[(0, 16384), (1, 16384)])
    world.start_server()
    named = f"{SELFTEST_LOCAL_ADDRESS}:1"
    first_id = world.submit(["cargo", "test", "--features", "gpu", f"fake-quiesce=card-fault-names:{named}"])
    first = world.finished(first_id, timeout=STATE_TIMEOUT_SECONDS * 2)   # 要起 CARD_FAULT_RETRIES + 1 次尝试，等得比一次的长
    # 之后那一条没有 fake-quiesce= 演法，假清场照真的那样起命令：PATH 前面放这一格的假 cargo，假内存包装才放行（真 cargo 它退 97）
    path = f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"
    second = world.finished(world.submit(["cargo", "test", "--features", "gpu", "after-named-card-fault"], extra={"PATH": path}))
    ran = [line for line in world.calls() if line.startswith("cargo ") and line.endswith(" after-named-card-fault")]
    cards = [attempt.get("card") for attempt in first["attempts"]]
    reason = card_fault_reason_line(world, first_id)
    alerts = [alert.get("line", "") for alert in world.alerts() if alert.get("kind") == "card-fault"]
    problems = []
    if first["task"].get("outcome") != "card-fault" or cards != [LOCAL_CARD_ZERO] * (lib.CARD_FAULT_RETRIES + 1):
        problems.append(f"清场点名 {named} 坏、分到的是 {LOCAL_CARD_ZERO}：各次尝试应当都派 {LOCAL_CARD_ZERO}、重排满 {lib.CARD_FAULT_RETRIES} 次照 card-fault 收尾，"
                        f"实际 {describe(first)}、各次的卡 {cards}")
    if f"清场判出卡 {named} 坏了" not in reason:
        problems.append(f"结局句（TASK_SERVER_REASON 的 why=）应当点名清场判出的 {named}，实际「{reason[:400]}」")
    if not alerts or any(f"清场判出卡 {named} 坏" not in line for line in alerts):
        problems.append(f"card-fault 告警应当每一条都点名 {named}，实际 {alerts}")
    if second["task"].get("outcome") != "succeeded" or [attempt.get("card") for attempt in second["attempts"]] != [LOCAL_CARD_ZERO] or len(ran) != 1:
        problems.append(f"停用的是 {named}，之后那一条应当照常派 {LOCAL_CARD_ZERO} 跑完，实际 {describe(second)}、"
                        f"各次的卡 {[attempt.get('card') for attempt in second['attempts']]}、假 cargo 起了 {len(ran)} 次")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return f"清场点名 {named} 坏：停用它、{LOCAL_CARD_ZERO} 照派，告警与结局句点名 {named}"


def cell_card_fault_unlisted_quarantines_none(world):
    """清场判出卡坏、清场那一行却没有坏卡清单（faulty-cards=）：一张卡都不停用、不重排，一次尝试照 card-fault 收尾，告警与说明写明取不到坏卡清单；
    之后交的一条照常派 0 号卡跑完（task_server.py 的 settle_attempt）。"""
    world.resources(local=(100, 100), cards=f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE}", local_gpu=[(0, 16384), (1, 16384)])
    world.start_server()
    first_id = world.submit(["cargo", "test", "--features", "gpu", "fake-quiesce=card-fault-unlisted"])
    first = world.finished(first_id)
    # 之后那一条没有 fake-quiesce= 演法，假清场照真的那样起命令：PATH 前面放这一格的假 cargo，假内存包装才放行（真 cargo 它退 97）
    path = f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"
    second = world.finished(world.submit(["cargo", "test", "--features", "gpu", "after-unlisted-card-fault"], extra={"PATH": path}))
    ran = [line for line in world.calls() if line.startswith("cargo ") and line.endswith(" after-unlisted-card-fault")]
    cards = [attempt.get("card") for attempt in first["attempts"]]
    reason = card_fault_reason_line(world, first_id)
    alerts = [alert.get("line", "") for alert in world.alerts() if alert.get("kind") == "card-fault"]
    problems = []
    if first["task"].get("outcome") != "card-fault" or cards != [LOCAL_CARD_ZERO] or "取不到坏卡清单" not in (first["task"].get("note") or ""):
        problems.append(f"取不到坏卡清单时应当不重排、一次尝试照 card-fault 收尾、说明写明取不到，实际 {describe(first)}、各次的卡 {cards}、说明「{first['task'].get('note')}」")
    if "取不到坏卡清单" not in reason:
        problems.append(f"结局句（TASK_SERVER_REASON 的 why=）应当写明取不到坏卡清单，实际「{reason[:400]}」")
    if len(alerts) != 1 or "取不到坏卡清单" not in alerts[0]:
        problems.append(f"应当恰好一条 card-fault 告警、写明取不到坏卡清单，实际 {alerts}")
    if second["task"].get("outcome") != "succeeded" or [attempt.get("card") for attempt in second["attempts"]] != [LOCAL_CARD_ZERO] or len(ran) != 1:
        problems.append(f"一张卡都没停用，之后那一条应当照常派 {LOCAL_CARD_ZERO} 跑完，实际 {describe(second)}、假 cargo 起了 {len(ran)} 次")
    if problems:
        raise SelftestFailure("；".join(problems), world.server_log_lines())
    return "取不到坏卡清单：一张卡都没停用、不重排，告警与说明写明取不到"


def cell_failed_peak_floor(world):
    """同一命令键只有一次命令自己退 1 的尝试、量到峰值 3G：再交一条，第一次上限不低于 3G × 1.5（4608M），不退回 4096M（task_server.py 的 failed_peak_floor）。"""
    key = "selftest-failed-peak"
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": world.write_fake_scope(FAILED_PEAK_BYTES)})
    first = world.finished(world.submit(["bash", "-c", "exit 1"], extra={lib.KEY_VARIABLE: key}))
    second = world.finished(world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(second)
    if (first["task"].get("outcome") != "command-failed" or first_attempt(first).get("memory_peak") != FAILED_PEAK_BYTES
            or attempt.get("memory_cap_text") != FAILED_PEAK_PINNED_TEXT or attempt.get("prediction") != lib.FAILED_PEAK_HISTORY):
        raise SelftestFailure(f"头一条退 1、峰值 {FAILED_PEAK_BYTES}，再交同一条第一次上限应当是 {FAILED_PEAK_PINNED_TEXT}（prediction {lib.FAILED_PEAK_HISTORY}），"
                              f"实际头一条 {describe(first)}、峰值 {first_attempt(first).get('memory_peak')}，再交的 {describe(second)}、prediction {attempt.get('prediction')}",
                              world.server_log_lines())
    return f"只判红过一次（峰值 3G）的命令再交，第一次上限 {FAILED_PEAK_PINNED_TEXT}"


def cell_environment_prefix_stripped_from_key(world):
    """命令前面带 env 与不改跑法的变量（多机配置路径、LANG）的，与不带的落同一个命令键，钉值对两条都生效；带了名单外变量的不并（lib.key_words）。"""
    world.set_slice_cap("20G")
    world.resources(local=(100, 100))
    world.start_server()
    plain = ["bash", "-c", "exit 0", "prefix-probe"]
    prefixed = ["env", f"SINGLEFS_MULTI_HOST_CONFIG={os.path.join(world.root, 'multi-host.env')}", "LANG=C", *plain]
    unlisted = ["env", "SELFTEST_UNLISTED_VARIABLE=1", "LANG=C", *plain]
    pinned = world.control(["pin-cap", "--", *plain])
    shown = shown_local_pin(pinned)
    wanted = f"{shown}M" if shown is not None else "pin-cap 没报本机的钉值"   # 钉值的口径由 pinned-cap-by-host 钉数，这一格照服务报的此刻钉值核
    summaries = {name: world.finished(world.submit(command)) for name, command in (("不带前缀的", plain), ("带前缀的", prefixed), ("带名单外变量的", unlisted))}
    keys = {name: summary["task"].get("command_key") for name, summary in summaries.items()}
    caps = {name: first_attempt(summary).get("memory_cap_text") for name, summary in summaries.items()}
    if (pinned.returncode != 0 or shown is None or keys["带前缀的"] != keys["不带前缀的"] or keys["带名单外变量的"] == keys["不带前缀的"]
            or caps["不带前缀的"] != wanted or caps["带前缀的"] != wanted or caps["带名单外变量的"] == wanted):
        raise SelftestFailure(f"带 env 与名单里变量前缀的应当与不带的同一个命令键、都吃到钉值（pin-cap 报的 {wanted}），带名单外变量的另一个键、不吃钉值；"
                              f"实际 pin-cap 退 {pinned.returncode}、键 {keys}、上限 {caps}", world.server_log_lines())
    return f"带 env 前缀与不带的同一个命令键、都吃到钉值 {wanted}；带名单外变量的另算（上限 {caps['带名单外变量的']}）"


def cell_verdict_library_family_refused(world):
    """③ 带判定库的提交时只照登记表 .claude/gate.d/verdict-library-scripts.tsv 第三列判（task_server.py 文件头 ②「提交时拒」③）：第三列写「prune-server」的族照收、
    本机放不下就派到第二台跑完；写「无判定库：」的不算带判定库、照收；第三列写本地库目录的族，直接点到的与包在临时 prune-server 里的都当场拒
    （task submit 退 3、不给任务号，说明点名那一族与第三列、出路写接进 prune-server），一条都没收；登记表写坏了（不到三列）的判不了、照拒。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    registry = os.path.join(world.repo, SERVER.VERDICT_LIBRARY_REGISTRY)
    rows = ("# 自证的登记表：路径<TAB>自证命令<TAB>判定库放哪<TAB>复用三问格名\n"
            "scripts/selftest-served.sh\tbash scripts/selftest-served.sh --selftest\tprune-server（结论表 verdict-library/…）\treuse-three-questions\n"
            "scripts/selftest-local-library.sh\tbash scripts/selftest-local-library.sh --selftest\t<库根的 git common-dir>/selftest-verdicts/\treuse-three-questions\n"
            "scripts/selftest-no-library.sh\tbash scripts/selftest-no-library.sh --selftest\t无判定库：待立\treuse-three-questions\n")
    with open(registry, "w", encoding="utf-8") as handle:
        handle.write(rows)
    world.start_server()
    problems, accepted = [], []
    for name, word in (("第三列写 prune-server 的族", "scripts/selftest-served.sh"), ("第三列写「无判定库：」的", "scripts/selftest-no-library.sh")):
        try:
            task_id = world.submit(["bash", "-c", "exit 0", name, word], cwd=world.repo)
            accepted.append(task_id)
            summary = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
            if first_attempt(summary).get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
                problems.append(f"{name}应当照收、派第二台跑完，实际 {describe(summary)}")
        except SelftestFailure as failure:
            problems.append(f"{name}应当照收、派第二台跑完，实际 {failure}")
    local_library = "scripts/selftest-local-library.sh"
    wrapped = ["python3", f"scripts/prune-server/{lib.TEMPORARY_SERVICE_SCRIPT}", lib.TEMPORARY_SERVICE_FLAG, "--server-binary", "/usr/bin/true", "--",
               "bash", "-c", "exit 0", "wrapped-local-library", local_library]
    for name, command in (("直接点到第三列写本地库目录的族", ["bash", "-c", "exit 0", "local-library", local_library]),
                          ("包在临时 prune-server 里点到那一族", wrapped)):
        completed = world.client(["submit", "--cwd", world.repo, "--", *command])
        wanted = (local_library, "第三列", "接进 prune-server")
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or not all(text in completed.stderr for text in wanted):
            problems.append(f"{name}：应当退 3、不给任务号、说明点名 {local_library} 与第三列、出路写接进 prune-server，"
                            f"实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-400:]}」")
    with open(registry, "a", encoding="utf-8") as handle:
        handle.write("scripts/selftest-broken-row.sh\tbash scripts/selftest-broken-row.sh --selftest\n")
    completed = world.client(["submit", "--cwd", world.repo, "--", "bash", "-c", "exit 0", "broken-registry"])
    if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or "判不了" not in completed.stderr or "不到三列" not in completed.stderr:
        problems.append(f"登记表有一行不到三列：应当判不了、退 3、说明点名那一行，实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-400:]}」")
    with open(registry, "w", encoding="utf-8") as handle:
        handle.write(rows)
    listed = {item.get("id") for item in json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", [])}
    if listed - set(accepted):
        problems.append(f"拒了的一条都不该进队，实际会话里另列着 {sorted(listed - set(accepted))}")
    if problems:
        raise SelftestFailure("带判定库的提交时照登记表第三列判：" + "；".join(problems), world.server_log_lines())
    return "第三列写 prune-server 的族与写「无判定库：」的照收、派到第二台；写本地库目录的族（直接点到、包在临时 prune-server 里）与登记表写坏的提交时拒"


def cell_workspace_writer_not_retried(world):
    """改主工作区的命令（不带 --skip-apply 的 merge-and-verify）撞了上限记 memory-cap-hit、被总上限挤掉记 squeezed-out，都只有一次尝试、打回，不原样重排
    （WORKSPACE_WRITER_NO_RETRY_OUTCOMES）；带 --skip-apply 的照乘 1.5 重排跑完（task_server.py 的 finish_attempt）。"""
    world.resources(local=(30, 30))
    world.start_server()
    writer = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/merge-and-verify.sh", "fake-wrapper-exits=250,run"], cwd=world.repo))
    squeezed = world.finished(world.submit(["bash", "-c", "exit 0", "y", "scripts/gate/merge-and-verify.sh", "fake-wrapper-exits=254,run"], cwd=world.repo))
    skipped = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/merge-and-verify.sh", "--skip-apply", "fake-wrapper-exits=250,run"], cwd=world.repo))
    feed = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/hook-swap.py", "--feed-only", "fake-wrapper-exits=250,run"], cwd=world.repo))
    if (writer["task"].get("outcome") != "memory-cap-hit" or len(writer["attempts"]) != 1
            or squeezed["task"].get("outcome") != "squeezed-out" or len(squeezed["attempts"]) != 1
            or skipped["task"].get("outcome") != "succeeded" or len(skipped["attempts"]) != 2
            or feed["task"].get("outcome") != "succeeded" or len(feed["attempts"]) != 2):
        raise SelftestFailure(f"不带 --skip-apply 的 merge-and-verify 撞上限应当记 memory-cap-hit、被挤掉应当记 squeezed-out，都只有一次尝试；带 --skip-apply 的与 hook-swap --feed-only 的应当乘 1.5 重排跑完；"
                              f"实际撞上限的 {describe(writer)}、被挤掉的 {describe(squeezed)}、带的 {describe(skipped)}、只喂的 {describe(feed)}", world.server_log_lines())
    return "改主工作区的撞上限与被挤掉都只有一次、照那一种结局打回；带 --skip-apply 的与 hook-swap --feed-only 的乘 1.5 重排跑完"


WRITER_HEADROOM_KEY = "selftest-writer-headroom"            # workspace-writer-initial-cap-headroom：改工作区的那条（hook-swap.py 带 --verify）的命令键
WRITER_HEADROOM_READER_KEY = "selftest-writer-headroom-reader"   # 那一格同样带 --verify 而不改工作区的那条的命令键
WRITER_HEADROOM_VERIFY = "python3 scripts/selftest-swapped.py --selftest"   # 那一格两条的 --verify 给的验证命令（服务照命令键换算，同 task_server.py 的 verify_command_keys）
WRITER_HEADROOM_OWN_PEAK_BYTES = 400 * MEBIBYTE            # 那一格两条各自那把键记下的峰值：照同键窗口 × 1.5 是 600M（8367 那样的小上限）
WRITER_HEADROOM_VERIFY_PEAK_BYTES = 500 * MEBIBYTE         # 那一格验证命令那把键记下的峰值
WRITER_HEADROOM_CAP_BYTES = (400 + 500) * 3 // 2 * MEBIBYTE   # 改工作区的那条首趟的上限：(400M + 500M) × 1.5 = 1350M（这一格的小世界把 4G 下限换成 0）
WRITER_HEADROOM_READER_CAP_BYTES = 400 * 3 // 2 * MEBIBYTE    # 不改工作区的那条照同键窗口：600M


def cell_workspace_writer_initial_cap_headroom(world):
    """改主工作区的命令首趟给足（task_server.py 文件头 ②「内存」的同名一段；任务 8367：hook-swap.py 带 --verify 照同键窗口只给 857M，撞了、打回、不重排）：
    hook-swap.py 带 --verify 的一条，同键只记过 400M、它的验证命令那把键记过 500M：首趟上限与记账是 (400M + 500M) × 1.5 = 1350M、prediction 记 WORKSPACE_WRITER_HISTORY
    （不设 4G 底；这一格的小世界里弄坏开关 recorded-cap-floored 的下限是 0，见 World 的 MEMORY_FLOOR_VARIABLE），跑完；同样带 --verify、同键也只记过 400M 而不改工作区的一条照同键窗口 600M。
    弄坏开关 workspace-writer-headroom-skipped 下改工作区的那条也只给 600M，这一格红。"""
    world.seed_peak(WRITER_HEADROOM_KEY, WRITER_HEADROOM_OWN_PEAK_BYTES)
    world.seed_peak(WRITER_HEADROOM_READER_KEY, WRITER_HEADROOM_OWN_PEAK_BYTES)
    world.seed_peak(PEAKS.command_key(shlex.split(WRITER_HEADROOM_VERIFY), None), WRITER_HEADROOM_VERIFY_PEAK_BYTES)
    world.resources(local=(30, 30))
    world.start_server()
    cases = (("改工作区的 hook-swap.py 带 --verify", WRITER_HEADROOM_KEY,
              ["bash", "-c", "exit 0", "x", "scripts/gate/hook-swap.py", "selftest-spec.json", "--scratch", world.work, "--verify", WRITER_HEADROOM_VERIFY],
              WRITER_HEADROOM_CAP_BYTES, lib.WORKSPACE_WRITER_HISTORY),
             ("不改工作区的一条带 --verify", WRITER_HEADROOM_READER_KEY,
              ["bash", "-c", "exit 0", "x", "scripts/selftest-reader.py", "--verify", WRITER_HEADROOM_VERIFY],
              WRITER_HEADROOM_READER_CAP_BYTES, lib.RECORD_HISTORY))
    problems = []
    for name, key, command, cap_bytes, history in cases:
        summary = world.finished(world.submit(command, extra={lib.KEY_VARIABLE: key}, cwd=world.repo))
        attempt = first_attempt(summary)
        if (len(summary["attempts"]) != 1 or attempt.get("memory_cap") != cap_bytes or attempt.get("booked") != cap_bytes
                or attempt.get("prediction") != history or summary["task"].get("outcome") != "succeeded"):
            problems.append(f"{name}：首趟上限与记账应当是 {cap_bytes // MEBIBYTE}M、prediction 记 {history}、一次跑完，实际上限 {attempt.get('memory_cap_text')}"
                            f"（{attempt.get('memory_cap')} 字节）、记账 {attempt.get('booked')}、prediction {attempt.get('prediction')}、{describe(summary)}")
    if problems:
        raise SelftestFailure("改主工作区的命令首趟给足：" + "；".join(problems), world.server_log_lines())
    return (f"改工作区的那条首趟 {WRITER_HEADROOM_CAP_BYTES // MEBIBYTE}M（同键 400M 加验证命令 500M 再 × 1.5），"
            f"不改工作区的照同键窗口 {WRITER_HEADROOM_READER_CAP_BYTES // MEBIBYTE}M")


def cell_retry_cap_not_floored(world):
    """有记录的照记录、不设 4G 底（task_server.py 文件头 ②「内存」；这一格照真的下限 4G，不用各格小世界的 0，弄坏开关 recorded-cap-floored 下兜的就是它）：
    同一命令键记下的峰值 247M，头一次照记录取 371M、假包装退 250 撞上限，重排那一次照 371M × 1.5 = 557M（不兜到 4096M）、记账照上限、跑完；
    钉了上限的命令（多机配置 SLICE_MEMORY_CAP_BY_HOST 两台都写 3G）照 pin-cap 报的钉值（低于 4G）。弄坏开关 recorded-cap-floored 下两次都是 4096M，这一格红。"""
    key = "selftest-memory-floor"
    world.seed_peak(key, MEMORY_FLOOR_SEED_BYTES)
    world.set_slice_cap("3G")   # 钉值 = 那一项减这台的假余量 2G（pin_value）：1024M，低于 4G
    world.start_server(extra={lib.MEMORY_FLOOR_VARIABLE: ""})
    retried = world.finished(world.submit(["bash", "-c", "exit 0", "memory-floor-retried", "fake-wrapper-exits=250,run"], extra={lib.KEY_VARIABLE: key}))
    probe = ["bash", "-c", "exit 0", "memory-floor-pinned"]
    pinned_reply = world.control(["pin-cap", "--", *probe])
    shown = shown_local_pin(pinned_reply)
    pinned = world.finished(world.submit(probe))
    problems = []
    caps = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in retried["attempts"]]
    booked = [int(attempt.get("booked") or 0) for attempt in retried["attempts"]]
    retry_bytes = int(MEMORY_FLOOR_RETRY_TEXT[:-1]) * MEBIBYTE
    if (caps != [(MEMORY_FLOOR_RECORDED_TEXT, "memory-cap-hit"), (MEMORY_FLOOR_RETRY_TEXT, "succeeded")] or booked[1:] != [retry_bytes]
            or retried["task"].get("outcome") != "succeeded"):
        problems.append(f"记下 247M 的命令头一次应当照记录 {MEMORY_FLOOR_RECORDED_TEXT} 撞上限，重排那一次上限 {MEMORY_FLOOR_RETRY_TEXT}（不兜到 4G）、记账照上限、跑完，"
                        f"实际各次 {caps}、记账 {booked}、{describe(retried)}")
    attempt = first_attempt(pinned)
    if (pinned_reply.returncode != 0 or shown is None or shown * MEBIBYTE >= lib.MEMORY_FLOOR_BYTES or attempt.get("memory_cap_text") != f"{shown}M"
            or attempt.get("prediction") != lib.PINNED_HISTORY):
        problems.append(f"钉了上限的命令（SLICE_MEMORY_CAP_BY_HOST 两台都写 3G）应当照 pin-cap 报的钉值（{shown}M，低于 4G）、不兜到 4G，"
                        f"实际 pin-cap 退 {pinned_reply.returncode}、上限 {attempt.get('memory_cap_text')}、"
                        f"prediction {attempt.get('prediction')}")
    if problems:
        raise SelftestFailure("重排不设 4G 底：" + "；".join(problems), world.server_log_lines())
    return (f"记下 247M 的命令键：头一次照记录 {MEMORY_FLOOR_RECORDED_TEXT}、撞了重排那一次 {MEMORY_FLOOR_RETRY_TEXT}（不兜到 4G）、记账照上限；"
            f"配置写 3G 的钉值照 {shown}M")


def cell_recorded_cap_not_floored(world):
    """有真实峰值记录的上限照峰值定（task_server.py 文件头 ②「内存」的有记录的照记录；这一格照真的下限 4G，不用各格小世界的 0）：同一命令键记下的峰值 200M，
    上限与记账都是 300M（200M × 1.5 按 MiB 向上取整）、prediction 记 RECORD_HISTORY、内存包装收到 cap=300M，不兜到 4096M；没记录的照旧 4096M。
    弄坏开关 recorded-cap-floored 下记录定的也兜到 4096M，这一格红。"""
    key = "selftest-recorded-small"
    world.seed_peak(key, RECORDED_SMALL_PEAK_BYTES)
    world.start_server(extra={lib.MEMORY_FLOOR_VARIABLE: ""})
    recorded_id = world.submit(["bash", "-c", "exit 0", "recorded-small"], extra={lib.KEY_VARIABLE: key})
    unrecorded_id = world.submit(["bash", "-c", "exit 0", "recorded-small-unrecorded"])
    expected = {"有记录的": (recorded_id, RECORDED_SMALL_CAP_TEXT, lib.RECORD_HISTORY),
                "没记录的": (unrecorded_id, UNRECORDED_DEFAULT_PINNED_TEXT, lib.UNRECORDED_HISTORY)}
    problems = []
    for name, (task_id, cap_text, history) in expected.items():
        summary = world.finished(task_id)
        attempt = first_attempt(summary)
        cap_bytes = int(cap_text[:-1]) * MEBIBYTE
        wrapper = wrapper_cap_texts(world, task_id)
        if (attempt.get("memory_cap_text") != cap_text or attempt.get("memory_cap") != cap_bytes or attempt.get("booked") != cap_bytes
                or attempt.get("prediction") != history or wrapper[:1] != [cap_text] or summary["task"].get("outcome") != "succeeded"):
            problems.append(f"{name}上限与记账都应当是 {cap_text}（{cap_bytes} 字节）、prediction 记 {history}、内存包装收到 cap={cap_text}、跑完，"
                            f"实际上限 {attempt.get('memory_cap_text')}（{attempt.get('memory_cap')} 字节）、记账 {attempt.get('booked')}、"
                            f"prediction {attempt.get('prediction')}、包装收到 {wrapper}、{describe(summary)}")
    if problems:
        raise SelftestFailure("有记录的照峰值定上限：" + "；".join(problems), world.server_log_lines())
    return f"记下 200M 的命令上限与记账都是 {RECORDED_SMALL_CAP_TEXT}（不兜到 4G），没记录的照旧 {UNRECORDED_DEFAULT_PINNED_TEXT}"


def sealed_gap_world(world, release, blockers, probes):
    """插空那三格的世界（task_server.py 的 Alternation.gap_fill）：一台 50G（界 40.8G）上依次交整片（记 47.25G）、两条挡路碎片（各记 15G，跑到 release 在了才退）、
    一条没有预计用时的碎片（记 15G：装进那一组就超过界，封组）、再交 probes 里来插空的碎片（各记 1.5G）；都没满时限、一齐放出来。
    blockers 是两条挡路碎片的 [(命令里的名, 记下的用时或 None)]，probes 是来插空的 [(名, 记下的用时)]。等两条挡路的都在跑，
    交 {名: 任务号}，另带「整片」「封组」与「挡路」（两条的任务号）。每条命令各一把键：预计用时照命令键取（task_server.py 的 command_estimate），
    同键的别条记下的用时会带到没记用时的那一条上。"""
    world.resources(local=(50, 50), hold=True)
    world.seed_peak("selftest-gap-whole", GAP_WHOLE_PEAK_BYTES)
    world.seed_peak("selftest-gap-sealed", GAP_HOLDER_PEAK_BYTES)
    holder_commands = [(f"selftest-gap-holder-{name}", ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, name, release]) for name, _ in blockers]
    for (key, command), (_, seconds) in zip(holder_commands, blockers):
        world.seed_peak(key, GAP_HOLDER_PEAK_BYTES)
        if seconds is not None:
            world.seed_duration(key, seconds, command)
    probe_commands = {name: (f"selftest-gap-probe-{seconds}", ["bash", "-c", "exit 0", f"gap-probe-{seconds}"]) for name, seconds in probes}
    for name, seconds in probes:
        key, command = probe_commands[name]
        world.seed_peak(key, GAP_PROBE_PEAK_BYTES)
        world.seed_duration(key, seconds, command)
    world.start_server()
    tasks = {"整片": world.submit(["bash", "-c", "exit 0", "gap-whole"], extra={lib.KEY_VARIABLE: "selftest-gap-whole"})}
    tasks["挡路"] = [world.submit(command, extra={lib.KEY_VARIABLE: key}) for key, command in holder_commands]
    tasks["封组"] = world.submit(["bash", "-c", "exit 0", "gap-sealed-unestimated"], extra={lib.KEY_VARIABLE: "selftest-gap-sealed"})
    for name, (key, command) in probe_commands.items():
        tasks[name] = world.submit(command, extra={lib.KEY_VARIABLE: key})
    world.resources(local=(50, 50))   # 一齐放出来：整片让一组碎片先起，两条挡路的装进组，没有预计用时的那条封组
    world.api("POST", "/wake")
    for task_id in tasks["挡路"]:
        world.running(task_id)
    return tasks


def sealed_reason_problem(name, summary, refusal):
    """封着的那一条应当没起、排队原因带封组那一句与 refusal（没插空的那一句）：对交空串，不对交一句。"""
    reason = summary.get("waiting_for") or ""
    if summary["attempts"] or SEALED_GROUP_TEXT not in reason or refusal not in reason:
        return f"{name}应当封着、排队原因带「{SEALED_GROUP_TEXT}」与「{refusal}」，实际尝试 {len(summary['attempts'])} 次、排队原因「{reason}」"
    return ""


def gap_admitted_problem(name, summary, blockers_running):
    """插空的那一条应当在挡路的还在跑时跑完、尝试记 gap_fill（插空那一句）：对交空串，不对交一句。"""
    attempt = first_attempt(summary)
    note = attempt.get("gap_fill") or ""
    if summary["task"].get("outcome") != "succeeded" or not note.startswith(GAP_ADMITTED_TEXT) or "不超过挡路时长" not in note or not blockers_running:
        return (f"{name}应当在挡路的还在跑时插空起来、跑完，尝试的 gap_fill 以「{GAP_ADMITTED_TEXT}」起头、点名挡路时长，"
                f"实际 {describe(summary)}、gap_fill「{note}」、挡路的{'还在跑' if blockers_running else '已不都在跑'}")
    return ""


def release_gap_blockers(world, release, tasks):
    """插空那三格收尾：放行挡路的两条、等它们跑完（sealed_gap_world 没交回任务号的，没有要等的）。"""
    open(release, "w").close()
    for task_id in tasks.get("挡路", []):
        world.finished(task_id)


def cell_sealed_gap_takes_short_task(world):
    """封了组的那台放短任务插空：两条挡路碎片各记了 600 秒的用时、刚起，来插空的那条记了 1 秒：1 秒不超过挡路时长（约 600 秒）、内存照公式放得下，
    挡路的还在跑时它就起、跑完，尝试记 gap_fill；没有预计用时的那条封组碎片照旧封着、排队原因写没插空的原因。
    弄坏开关 sealed-gap-not-backfilled 下它封着、不起。"""
    release = os.path.join(world.root, "release-gap-short")
    tasks, problems = {}, []
    try:
        tasks = sealed_gap_world(world, release, (("gap-blocker-estimated", GAP_BLOCKER_ESTIMATE_SECONDS),
                                                  ("gap-blocker-estimated-also", GAP_BLOCKER_ESTIMATE_SECONDS)),
                                 (("短任务", GAP_SHORT_ESTIMATE_SECONDS),))
        world.next_pass()
        world.next_pass()
        short = world.summary(tasks["短任务"])
        if short["attempts"]:
            short = world.finished(tasks["短任务"])
        blockers_running = all(world.summary(task_id)["task"].get("state") == "running" for task_id in tasks["挡路"])
        problems += [problem for problem in (gap_admitted_problem("预计用时 1 秒的短任务", short, blockers_running),
                                             sealed_reason_problem("没有预计用时的那条碎片", world.summary(tasks["封组"]), f"{GAP_REFUSED_TEXT}这一条没有预计用时"))
                     if problem]
    finally:
        release_gap_blockers(world, release, tasks)
    if problems:
        raise SelftestFailure("封组时放短任务插空：" + "；".join(problems), world.server_log_lines())
    return f"挡路的两条记了 {GAP_BLOCKER_ESTIMATE_SECONDS} 秒、还在跑：预计用时 {GAP_SHORT_ESTIMATE_SECONDS} 秒的插空起来跑完，没有预计用时的照旧封着"


def cell_sealed_gap_holds_longer_task(world):
    """封了组的那台不放超过挡路时长的：两条挡路碎片各记了 600 秒的用时、刚起，来插空的那条记了 6000 秒：超过挡路时长，照旧封着，
    排队原因在封组那一句后面写它的预计用时超过挡路时长。弄坏开关 sealed-gap-length-ignored 下它照起。"""
    release = os.path.join(world.root, "release-gap-long")
    tasks, problems = {}, []
    try:
        tasks = sealed_gap_world(world, release, (("gap-blocker-estimated", GAP_BLOCKER_ESTIMATE_SECONDS),
                                                  ("gap-blocker-estimated-also", GAP_BLOCKER_ESTIMATE_SECONDS)),
                                 (("长任务", GAP_LONG_ESTIMATE_SECONDS),))
        world.next_pass()
        world.next_pass()
        problem = sealed_reason_problem(f"预计用时 {GAP_LONG_ESTIMATE_SECONDS} 秒的长任务", world.summary(tasks["长任务"]),
                                        f"{GAP_REFUSED_TEXT}这一条预计用时 {GAP_LONG_ESTIMATE_SECONDS:.1f} 秒，超过挡路时长")
        problems += [problem] if problem else []
    finally:
        release_gap_blockers(world, release, tasks)
    if problems:
        raise SelftestFailure("封组时超过挡路时长的不插空：" + "；".join(problems), world.server_log_lines())
    return f"挡路的两条记了 {GAP_BLOCKER_ESTIMATE_SECONDS} 秒：预计用时 {GAP_LONG_ESTIMATE_SECONDS} 秒的照旧封着、排队原因写超过挡路时长"


def cell_unestimated_blocker_counted_by_elapsed(world):
    """挡路的没有预计用时（连同已跑过预计用时的）按已跑算：一条挡路碎片没有记录、一条记了 0.2 秒（一起来就跑过了），来插空的两条记了 1 秒与 3600 秒。
    挡路的已跑不到 1 秒时 1 秒那条照封着，已跑过 1 秒就插空起来、跑完（起的时刻比挡路的最早那条晚至少 1 秒）；3600 秒那条一直封着、排队原因写超过挡路时长。
    弄坏开关 blocker-remaining-unbounded 下挡路时长没有尽头、3600 秒那条也起，sealed-gap-not-backfilled 下 1 秒那条不起，sealed-gap-length-ignored 下两条一起就起。"""
    release = os.path.join(world.root, "release-gap-elapsed")
    tasks, problems = {}, []
    try:
        tasks = sealed_gap_world(world, release, (("gap-blocker-unrecorded", None), ("gap-blocker-overrun", GAP_OVERRUN_ESTIMATE_SECONDS)),
                                 (("短任务", GAP_SHORT_ESTIMATE_SECONDS), ("长任务", GAP_UNESTIMATED_LONG_SECONDS)))
        try:
            world.wait_for(tasks["短任务"], lambda summary: bool(summary["attempts"]), "插空起来", timeout=GAP_ELAPSED_WAIT_SECONDS)
            short = world.finished(tasks["短任务"])
        except SelftestFailure:
            short = world.summary(tasks["短任务"])
        blocker_attempts = [first_attempt(world.summary(task_id)) for task_id in tasks["挡路"]]
        blockers_running = all(attempt.get("state") == "running" for attempt in blocker_attempts)
        problem = gap_admitted_problem(f"预计用时 {GAP_SHORT_ESTIMATE_SECONDS} 秒的短任务", short, blockers_running)
        problems += [problem] if problem else []
        earliest = min(float(attempt.get("started_at") or 0) for attempt in blocker_attempts)
        started = float(first_attempt(short).get("started_at") or 0)
        if short["attempts"] and started - earliest < GAP_SHORT_ESTIMATE_SECONDS - STARTED_AT_ROUNDING_SECONDS:
            problems.append(f"短任务应当等挡路的已跑过 {GAP_SHORT_ESTIMATE_SECONDS} 秒才插空，实际比挡路的最早那条只晚 {started - earliest:.2f} 秒起")
        world.next_pass()
        problem = sealed_reason_problem(f"预计用时 {GAP_UNESTIMATED_LONG_SECONDS} 秒的长任务", world.summary(tasks["长任务"]),
                                        f"{GAP_REFUSED_TEXT}这一条预计用时 {GAP_UNESTIMATED_LONG_SECONDS:.1f} 秒，超过挡路时长")
        problems += [problem] if problem else []
    finally:
        release_gap_blockers(world, release, tasks)
    if problems:
        raise SelftestFailure("挡路的没有预计用时按已跑算：" + "；".join(problems), world.server_log_lines())
    return (f"挡路的没有记录、或已跑过记下的用时：按已跑算，预计用时 {GAP_SHORT_ESTIMATE_SECONDS} 秒的等已跑过它才插空，"
            f"{GAP_UNESTIMATED_LONG_SECONDS} 秒的照旧封着")


ITEM_ENTRY_RELATIVE = lib.ITEM_ENTRIES[1]   # 按项那几格的假入口放在假仓根下这个相对路径（.claude/gate.d/formal-verification.sh）：服务照路径认出它
ITEM_SPEC_DIRECTORY = "fake-item-entry"     # 假入口读每项的 ITEM_PLAN、ITEM_DONE 与放行文件的目录（假仓根下）
ITEM_WAIT_SECONDS = 30.0                    # 按项那几格等估算起来、等父任务进队的时限
ITEM_PARENT_KEY = "selftest-item-parent"    # item-estimate-summed-from-rates 与 item-estimate-failure-queues-parent 的父任务的命令键：记下 20G 的峰值（上限 30G）
ITEM_PARENT_PEAK_BYTES = 20 * GIBIBYTE      # 那两格本机空闲 10G：父任务（要的空闲 22G）放不下、排着，估算子任务（没记录，4G）放得下
ITEM_COMMAND_FAILED_EXIT = 1                # item-estimate-failure-queues-parent 里估算子任务退的码（command-failed）
FAKE_ITEM_ENTRY = r'''#!/usr/bin/env bash
# task-server 自证的假按项入口（task_server_lib.py 的 ITEM_ENTRIES 之一）：每次被起往调用记录写一行「item-entry mode=<list|run> task=<号> args=<原样参数>」。
# 带 --list-items 的那一次（估算子任务）：各项有 list-hold-<项> 的先等 list-release 在了（至多 60 秒，等不到退 9），照 plan-<项> 打 ITEM_PLAN 那一行，
# 有 list-exit-<项> 的照它的内容退；别的那一次（父任务自己跑）照 done-<项> 打 ITEM_DONE 那几行、退 0。这几份都在假仓根下的 fake-item-entry/ 里。
spec="$(cd -- "$(dirname -- "$0")/../.." && pwd)/fake-item-entry"
mode=run
items=()
arguments="$*"
while (( $# )); do
  case "$1" in
    --list-items) mode=list ;;
    --item) items+=("${2:-}"); shift ;;
  esac
  shift
done
printf 'item-entry mode=%s task=%s args=%s\n' "$mode" "${TASK_SERVER_TASK:-}" "$arguments" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
if [[ "$mode" == list ]]; then
  code=0
  for item in "${items[@]}"; do
    if [[ -e "$spec/list-hold-$item" ]]; then
      for round in $(seq 1 600); do [[ -e "$spec/list-release" ]] && break; sleep 0.1; done
      [[ -e "$spec/list-release" ]] || exit 9
    fi
    if [[ -e "$spec/plan-$item" ]]; then cat -- "$spec/plan-$item"; fi
    if [[ -e "$spec/list-exit-$item" ]]; then code="$(cat -- "$spec/list-exit-$item")"; fi
  done
  exit "$code"
fi
for item in "${items[@]}"; do
  if [[ -e "$spec/done-$item" ]]; then cat -- "$spec/done-$item"; fi
done
exit 0
'''


def item_entry_world(world, files):
    """按项那几格的假入口：写进假仓根的 ITEM_ENTRY_RELATIVE，files（{文件名: 内容}）写进假仓根的 ITEM_SPEC_DIRECTORY；交那个目录。"""
    write_executable(os.path.join(world.repo, ITEM_ENTRY_RELATIVE), FAKE_ITEM_ENTRY)
    spec = os.path.join(world.repo, ITEM_SPEC_DIRECTORY)
    os.makedirs(spec, exist_ok=True)
    for name, text in files.items():
        with open(os.path.join(spec, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    return spec


def item_entry_command(world, items):
    """交给服务的那一条：bash <假入口> --item <项>…  <假仓根>（同三条入口的写法），在假仓根里起。"""
    return ["bash", ITEM_ENTRY_RELATIVE, *[word for item in items for word in (lib.ITEM_ARGUMENT, item)], world.repo]


def item_plan_line(item, to_judge, reusable):
    return f"ITEM_PLAN item={item} to_judge={to_judge} reusable={reusable}\n"


def item_done_line(item, seconds, judged, reused, exit_code=0):
    return f"ITEM_DONE item={item} seconds={seconds} judged={judged} reused={reused} exit={exit_code}\n"


def wait_for_call(world, prefix, timeout):
    """等调用记录里出现以 prefix 起头的一行，交那一行；等不到交空串。"""
    patience = world.patience(timeout)
    while patience.waiting():
        found = next((line for line in world.calls() if line.startswith(prefix)), "")
        if found:
            return found
        time.sleep(POLL_SECONDS)
    return ""


def item_rate_rows(world):
    """{项名: item_rates 那一行的正文}（校验过的；入口是假入口的那几行）。"""
    return {record.get("item"): record for record, valid in world.signed_rows("item_rates") if valid and record.get("entry") == ITEM_ENTRY_RELATIVE}


def item_parent_queued(world, task_id):
    """等估算完、父任务进队（state=queued）：交那时的 summary；等不到交 None。"""
    try:
        return world.wait_for(task_id, lambda summary: summary["task"].get("state") == "queued", "估算完进队", timeout=ITEM_WAIT_SECONDS)
    except SelftestFailure:
        return None


def cell_item_estimate_before_queue(world):
    """点了 --item 的按项入口先起估算子任务、估算完才进队（task_server_lib.py 文件头「资源记录」的预计用时）：估算那一条照原命令在入口后面接 --list-items --with-keys、
    两个 --item 与根照原样；它在跑（等放行）时父任务记 estimating、一次都没起，status 的 item_estimate 是 role=estimate 的子任务；放行之后父任务才起、跑成，
    调用记录里估算那一行在父任务那一行之前。弄坏开关 item-estimate-skipped 下父任务提交了就起，没有估算那一行。"""
    spec = item_entry_world(world, {"plan-alpha": item_plan_line("alpha", 2, 1), "plan-beta": item_plan_line("beta", 0, 3), "list-hold-alpha": ""})
    world.start_server()
    command = item_entry_command(world, ("alpha", "beta"))
    task_id = world.submit(command, cwd=world.repo)
    problems = []
    try:
        listed = wait_for_call(world, "item-entry mode=list ", ITEM_WAIT_SECONDS)
        summary = world.summary(task_id)
        estimate = (summary.get("item_estimate") or {}).get("task") or {}
        wanted = " ".join(list(lib.ITEM_ESTIMATE_ARGUMENTS) + command[2:])
        if not listed or not listed.endswith(f"args={wanted}"):
            problems.append(f"估算子任务应当起「{ITEM_ENTRY_RELATIVE} {wanted}」，调用记录里估算那一行是「{listed or '没有'}」")
        if summary["task"].get("state") != "estimating" or summary["attempts"]:
            problems.append(f"估算在跑时父任务应当记 estimating、一次都没起，实际 {summary['task'].get('state')}、尝试 {len(summary['attempts'])} 次")
        if estimate.get("role") != lib.ESTIMATE_ROLE or estimate.get("parent_id") != task_id:
            problems.append(f"status 的 item_estimate 应当是这一条名下 role={lib.ESTIMATE_ROLE} 的子任务，实际 {json.dumps(estimate, ensure_ascii=False)[:300]}")
    finally:
        open(os.path.join(spec, "list-release"), "w").close()
    finished = world.finished(task_id)
    calls = world.calls()
    order = [line.split()[1] for line in calls if line.startswith("item-entry ")]
    if finished["task"].get("outcome") != "succeeded" or len(finished["attempts"]) != 1 or order != ["mode=list", "mode=run"]:
        problems.append(f"放行之后父任务应当起一次、跑成，调用记录里先估算后跑，实际 {describe(finished)}、调用次序 {order}")
    if problems:
        raise SelftestFailure("按项先估算后进队：" + "；".join(problems), world.server_log_lines())
    return "点了 --item 的先起估算子任务（--list-items --with-keys 加原样的 --item），估算在跑时父任务记 estimating、不起，估算完才进队跑成"


def cell_item_done_rates_recorded(world):
    """尝试收尾时照 ITEM_DONE 按（入口、项名）记两个速率（task_server_lib.py 文件头「资源记录」的预计用时）：头一趟 alpha 只现判（20 秒 10 条）记每现判一条 2 秒、
    beta 只复用（4 秒 8 条）记每复用一条 0.5 秒、gamma 退 1 不记；第二趟 beta 现判 10 条、复用 8 条共 24 秒：照已记的复用速率扣掉 4 秒，记每现判一条 2 秒，复用速率不动。
    弄坏开关 item-done-unrecorded 下一行都不记。"""
    spec = item_entry_world(world, {"plan-alpha": item_plan_line("alpha", 10, 0), "plan-beta": item_plan_line("beta", 0, 8), "plan-gamma": item_plan_line("gamma", 3, 0),
                                    "done-alpha": item_done_line("alpha", 20, 10, 0), "done-beta": item_done_line("beta", 4, 0, 8),
                                    "done-gamma": item_done_line("gamma", 9, 3, 0, exit_code=1)})
    world.start_server()
    first = world.finished(world.submit(item_entry_command(world, ("alpha", "beta", "gamma")), cwd=world.repo))
    with open(os.path.join(spec, "done-beta"), "w", encoding="utf-8") as handle:
        handle.write(item_done_line("beta", 24, 10, 8))
    second = world.finished(world.submit(item_entry_command(world, ("beta",)), cwd=world.repo))
    rows = item_rate_rows(world)
    alpha, beta = rows.get("alpha") or {}, rows.get("beta") or {}
    problems = []
    if not task_command_ok(first, "succeeded") or not task_command_ok(second, "succeeded"):
        problems.append(f"两趟都应当跑成，实际头一趟 {describe(first)}、第二趟 {describe(second)}")
    if alpha.get("seconds_per_judged") != 2.0 or alpha.get("seconds_per_reused") is not None or (alpha.get("judged_from") or {}).get("task") != first["task"]["id"]:
        problems.append(f"alpha 应当记每现判一条 2 秒（出自任务 {first['task']['id']}）、没有复用速率，实际 {json.dumps(alpha, ensure_ascii=False)[:300]}")
    if beta.get("seconds_per_reused") != 0.5 or beta.get("seconds_per_judged") != 2.0 or (beta.get("judged_from") or {}).get("task") != second["task"]["id"]:
        problems.append(f"beta 应当记每复用一条 0.5 秒（头一趟）与每现判一条 2 秒（第二趟扣掉复用的 4 秒），实际 {json.dumps(beta, ensure_ascii=False)[:300]}")
    if "gamma" in rows:
        problems.append(f"gamma 那一项退 1，不该记速率，实际记了 {json.dumps(rows['gamma'], ensure_ascii=False)[:200]}")
    if problems:
        raise SelftestFailure("ITEM_DONE 记两个速率：" + "；".join(problems), world.server_log_lines())
    return "ITEM_DONE 照（入口、项名）记最近一次的两个速率：只现判的记现判、只复用的记复用、两样都有的扣掉复用再记现判、退非 0 的不记"


def cell_item_estimate_summed_from_rates(world):
    """预计用时照 ITEM_PLAN 与两个速率相加（task_server_lib.py 文件头「资源记录」的预计用时）：库里 alpha 每现判一条 2 秒、每复用一条 0.5 秒，beta 每现判一条 1 秒、没有复用速率，
    gamma 没记过；估算报 alpha 现判 3、复用 4，beta 现判 5、复用 0，gamma 现判 1。点 alpha、beta 的那条预计用时 3×2 + 4×0.5 + 5×1 = 13 秒；
    点 alpha、gamma 的缺 gamma 的速率，预计用时 None。父任务记得放不下（本机空闲 10G）、估算完一直排着，排着时读。
    弄坏开关 item-estimate-by-command 下照命令逐字相同算、两条都是 None，item-rate-missing-skipped 下缺的当 0、第二条是 8 秒，item-estimate-skipped 下不估、都是 None。"""
    item_entry_world(world, {"plan-alpha": item_plan_line("alpha", 3, 4), "plan-beta": item_plan_line("beta", 5, 0), "plan-gamma": item_plan_line("gamma", 1, 0)})
    world.seed_peak(ITEM_PARENT_KEY, ITEM_PARENT_PEAK_BYTES)
    world.resources(local=(10, 50))
    world.start_server()
    for item, judged, reused in (("alpha", 2.0, 0.5), ("beta", 1.0, None)):
        world.insert_signed_row("item_rates", {"task_id": 0, "state": lib.ITEM_RATE_STATE, "session": "", "command_key": lib.item_rate_key(ITEM_ENTRY_RELATIVE, item),
                                               "entry": ITEM_ENTRY_RELATIVE, "item": item, "seconds_per_judged": judged, "seconds_per_reused": reused,
                                               "judged_from": None, "reused_from": None})
    summed = world.submit(item_entry_command(world, ("alpha", "beta")), cwd=world.repo, extra={lib.KEY_VARIABLE: ITEM_PARENT_KEY})
    missing = world.submit(item_entry_command(world, ("alpha", "gamma")), cwd=world.repo, extra={lib.KEY_VARIABLE: ITEM_PARENT_KEY})
    problems = []
    for task_id, wanted, what in ((summed, 13.0, "点 alpha、beta 的"), (missing, None, "点 alpha、gamma 的（gamma 没有速率）")):
        summary = item_parent_queued(world, task_id)
        if summary is None:
            problems.append(f"{what}估算完应当进队排着，等了 {ITEM_WAIT_SECONDS:.0f} 秒实际 {describe(world.summary(task_id))}")
            continue
        if summary.get("estimated_seconds") != wanted:
            problems.append(f"{what}预计用时应当是 {wanted}，实际 {summary.get('estimated_seconds')}（item_plan {json.dumps(summary['task'].get('item_plan'), ensure_ascii=False)}）")
    world.resources(local=(50, 50))
    for task_id in (summed, missing):
        world.finished(task_id)
    if problems:
        raise SelftestFailure("按项的预计用时：" + "；".join(problems), world.server_log_lines())
    return "预计用时 = 各项 to_judge × 现判速率 + reusable × 复用速率之和（13 秒）；缺一项速率的是 None"


def cell_item_estimate_failure_queues_parent(world):
    """估算子任务没成时父任务照没有预计用时排、不卡住（task_server_lib.py 文件头「资源记录」的预计用时）：一条的估算退 1（打了 ITEM_PLAN 也不认），一条的估算退 0 而一行 ITEM_PLAN 都没打：
    两条的估算收尾之后父任务都进队、item_plan 是 null、预计用时 None；内存放出来之后都跑成。父任务记得放不下（本机空闲 10G），排着时读。
    弄坏开关 estimate-failure-holds-parent 下两条一直在估算，item-estimate-skipped 下没有估算子任务。"""
    item_entry_world(world, {"plan-broken": item_plan_line("broken", 1, 0), "list-exit-broken": str(ITEM_COMMAND_FAILED_EXIT)})
    world.seed_peak(ITEM_PARENT_KEY, ITEM_PARENT_PEAK_BYTES)
    world.resources(local=(10, 50))
    world.start_server()
    failed = world.submit(item_entry_command(world, ("broken",)), cwd=world.repo, extra={lib.KEY_VARIABLE: ITEM_PARENT_KEY})
    silent = world.submit(item_entry_command(world, ("silent",)), cwd=world.repo, extra={lib.KEY_VARIABLE: ITEM_PARENT_KEY})
    problems = []
    for task_id, outcome, what in ((failed, "command-failed", "估算退 1 的"), (silent, "succeeded", "估算没打 ITEM_PLAN 的")):
        summary = item_parent_queued(world, task_id)
        if summary is None:
            problems.append(f"{what}估算收尾之后父任务应当进队排着，等了 {ITEM_WAIT_SECONDS:.0f} 秒实际 {describe(world.summary(task_id))}")
            continue
        estimate = (summary.get("item_estimate") or {}).get("task") or {}
        if estimate.get("outcome") != outcome or summary["task"].get("item_plan") is not None or summary.get("estimated_seconds") is not None:
            problems.append(f"{what}估算子任务应当是 {outcome}、父任务 item_plan 是 null、预计用时 None，实际估算 {estimate.get('outcome')}、"
                            f"item_plan {summary['task'].get('item_plan')}、预计用时 {summary.get('estimated_seconds')}")
    world.resources(local=(50, 50))
    for task_id, what in ((failed, "估算退 1 的"), (silent, "估算没打 ITEM_PLAN 的")):
        try:
            finished = world.finished(task_id, timeout=ITEM_WAIT_SECONDS)
        except SelftestFailure:
            finished = world.summary(task_id)
        if finished["task"].get("outcome") != "succeeded":
            problems.append(f"{what}父任务应当照常跑成，实际 {describe(finished)}")
    if problems:
        raise SelftestFailure("估算没成时父任务照没有预计用时排：" + "；".join(problems), world.server_log_lines())
    return "估算子任务退非 0、或没打 ITEM_PLAN 时父任务照样进队、没有预计用时，跑成"


def cell_item_estimate_cancelled(world):
    """还在估算时取消（task_server.py 的 cancel_estimating）：估算子任务在跑（等放行）时取消父任务：客户端退 0，父任务记 cancelled、一次都没起，
    估算子任务停了、记 cancelled；之后父任务不进队、不起。"""
    spec = item_entry_world(world, {"plan-alpha": item_plan_line("alpha", 1, 0), "list-hold-alpha": ""})
    world.start_server()
    task_id = world.submit(item_entry_command(world, ("alpha",)), cwd=world.repo)
    problems = []
    try:
        if not wait_for_call(world, "item-entry mode=list ", ITEM_WAIT_SECONDS):
            problems.append(f"估算子任务应当起来、记一行调用，等了 {ITEM_WAIT_SECONDS:.0f} 秒没有")
        completed = world.client(["cancel", str(task_id)])
    finally:
        open(os.path.join(spec, "list-release"), "w").close()
    summary = world.summary(task_id)
    child = (summary.get("item_estimate") or {}).get("task") or {}
    estimate = world.finished(child["id"]) if child.get("id") else {"task": {}}
    if completed.returncode != 0 or summary["task"].get("outcome") != "cancelled" or summary["attempts"] or estimate["task"].get("outcome") != "cancelled":
        problems.append(f"估算时取消应当退 0、父任务记 cancelled 且一次没起、估算子任务记 cancelled，实际退 {completed.returncode}（{completed.stdout.strip()[:200]}）、"
                        f"父任务 {describe(summary)}、估算子任务 {estimate['task'].get('outcome')}")
    world.next_pass()
    world.next_pass()
    if world.summary(task_id)["attempts"] or any(line.startswith("item-entry mode=run ") for line in world.calls()):
        problems.append("取消之后父任务不该再进队、起来")
    if problems:
        raise SelftestFailure("估算时取消：" + "；".join(problems), world.server_log_lines())
    return "估算时取消：估算子任务停了、记 cancelled，父任务记 cancelled、一次没起"


ESTIMATE_FOLDED_TOOL = "fake-estimate-tool.sh"   # estimate-by-folded-command-key 起的假工具（假仓根里，退 0）：两条只差后面那个目录参数
ESTIMATE_BASIS_KEY = "selftest-estimate-basis"            # estimate-basis-command-sentence 记过 3600 秒用时的命令键
ESTIMATE_BASIS_SECONDS = 3600
ESTIMATE_ITEM_ALPHA_RATES = (2.0, 0.5)                    # estimate-basis-items-sentence：alpha 每现判一条、每复用一条几秒；估算报现判 3、复用 4：3×2 + 4×0.5 = 8 秒
ESTIMATE_ITEM_SECONDS = 8
SHARD_ESTIMATE_KEY = "selftest-shard-estimate"            # 两格分片估算的这件的键
SHARD_PLAN_SECONDS = 120                                  # shard-plan-and-merge-estimated-by-command：计划、合并各记过的用时
SHARD_MERGE_SECONDS = 45
SHARD_PIECE_SECONDS = {0: 300, 1: 900}                    # shard-pieces-estimated-by-own-history：两片各记过的用时（片号不同、用时不同，不混）
SHARD_ESTIMATE_PEAK_BYTES = 20 * GIBIBYTE                 # 要排着等的那一步记过的峰值：本机空闲 10G，要的空闲 22G 放不下
SHARD_ESTIMATE_WAIT_SECONDS = 60.0


def estimate_queue_listing(world, task_ids):
    """客户端 status --json 列的这个会话的任务里，指定几条的 estimated_seconds：{任务号: 秒数或 None}。"""
    listed = json.loads(world.client(["status", "--json"]).stdout or "{}")
    return {item["id"]: item.get("estimated_seconds") for item in listed.get("tasks", []) if item.get("id") in task_ids}


def estimate_status(world, task_id):
    """排着的一条：客户端 status <号> 打的文字，与 status <号> --json 里的 estimated_seconds、estimate_basis。"""
    shown = world.client(["status", str(task_id)])
    reply = json.loads(world.client(["status", str(task_id), "--json"]).stdout or "{}")
    return shown.stdout, reply.get("estimated_seconds"), reply.get("estimate_basis")


def cell_estimate_by_folded_command_key(world):
    """预计用时与内存记录同一把键（task_server.py 的 command_estimate；任务 7585 与 7447 只差补丁目录）：bash fake-estimate-tool.sh <跑过的目录> --dry-run 跑成一次，
    之后排着的 bash fake-estimate-tool.sh <排着的目录> --dry-run（路径照词折成〈文件〉、命令键相同，命令逐词不同）预计用时就是跑成那一次的用时、照命令得的。
    弄坏开关 estimate-by-exact-command 下照改前只认命令逐词相同的，排着的那条没有预计用时，这一格红。"""
    with open(os.path.join(world.repo, ESTIMATE_FOLDED_TOOL), "w", encoding="utf-8") as handle:
        handle.write("exit 0\n")
    ran_directory, queued_directory = (os.path.join(world.root, name) for name in ("patch-ran", "patch-queued"))
    world.resources(local=(30, 30))
    world.start_server()
    ran = world.finished(world.submit(["bash", ESTIMATE_FOLDED_TOOL, ran_directory, "--dry-run"], cwd=world.repo))
    seconds = first_attempt(ran).get("seconds")
    world.resources(local=(30, 30), hold=True)
    queued_id = world.submit(["bash", ESTIMATE_FOLDED_TOOL, queued_directory, "--dry-run"], cwd=world.repo)
    text, estimate, basis = estimate_status(world, queued_id)
    queued = world.summary(queued_id)
    world.resources(local=(30, 30))
    world.finished(queued_id)
    keys = (ran["task"].get("command_key"), queued["task"].get("command_key"))
    if (ran["task"].get("outcome") != "succeeded" or not isinstance(seconds, (int, float)) or keys[0] != keys[1]
            or not isinstance(estimate, (int, float)) or abs(estimate - seconds) > 0.01 or basis != lib.ESTIMATE_BASIS_COMMAND):
        raise SelftestFailure(f"两条只差目录参数的命令同一个命令键：一条跑成（用时 {seconds}）之后，排着的那一条预计用时应当就是它、照命令得的（{lib.ESTIMATE_BASIS_COMMAND}），"
                              f"实际跑成的那条 {describe(ran)}、两条的命令键 {keys}、预计用时 {estimate}、依据 {basis}、status「{text.strip()[:200]}」", world.server_log_lines())
    return f"路径不同、折叠后同键的命令：排着时预计用时 {estimate:.2f} 秒，取自同键跑成那一次的用时"


def cell_estimate_basis_command_sentence(world):
    """排着的任务预计用时是照命令得的：服务在回复里交 estimate_basis=command，客户端 status <号> 在预计用时后面打「照这条命令最近一次真跑完的用时」。
    弄坏开关 estimate-basis-unreported 下服务不交这个字段、客户端没有说明可打，这一格红。"""
    world.seed_duration(ESTIMATE_BASIS_KEY, ESTIMATE_BASIS_SECONDS, ["bash", "-c", "exit 0"])
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: ESTIMATE_BASIS_KEY})
    text, seconds, basis = estimate_status(world, task_id)
    world.resources(local=(30, 30))
    world.finished(task_id)
    if f"预计用时 {ESTIMATE_BASIS_SECONDS} 秒（照这条命令最近一次真跑完的用时）" not in text or seconds != ESTIMATE_BASIS_SECONDS or basis != "command":
        raise SelftestFailure(f"这条命令记过一次跑成的用时 {ESTIMATE_BASIS_SECONDS} 秒：status 应当打「预计用时 {ESTIMATE_BASIS_SECONDS} 秒（照这条命令最近一次真跑完的用时）」、"
                              f"回复里 estimate_basis 是 command，实际文字「{text.strip()[:300]}」、estimated_seconds {seconds}、estimate_basis {basis}")
    return "预计用时照命令得的：回复里 estimate_basis=command，status 打「照这条命令最近一次真跑完的用时」"


def cell_estimate_basis_items_sentence(world):
    """点了 --item 的按项入口估完进队、排着时预计用时是照项相加得的：库里 alpha 每现判一条 2 秒、每复用一条 0.5 秒，估算报现判 3、复用 4，3×2 + 4×0.5 = 8 秒；
    服务交 estimate_basis=items，客户端 status <号> 在预计用时后面打按项那一句（点到的每一项：要现判的条数 × 每现判一条几秒 + 能复用的条数 × 每复用一条几秒，相加），
    不打按命令那一句。父任务记得放不下（本机空闲 10G）、估算完一直排着，排着时读。
    弄坏开关 estimate-basis-unreported 下服务不交、estimate-basis-sentence-fixed 下客户端一律打按命令那一句，item-estimate-by-command 与 item-estimate-skipped 下
    这个数不是按项得的，这一格红。"""
    item_entry_world(world, {"plan-alpha": item_plan_line("alpha", 3, 4)})
    world.seed_peak(ITEM_PARENT_KEY, ITEM_PARENT_PEAK_BYTES)
    world.resources(local=(10, 50))
    world.start_server()
    judged, reused = ESTIMATE_ITEM_ALPHA_RATES
    world.insert_signed_row("item_rates", {"task_id": 0, "state": lib.ITEM_RATE_STATE, "session": "", "command_key": lib.item_rate_key(ITEM_ENTRY_RELATIVE, "alpha"),
                                           "entry": ITEM_ENTRY_RELATIVE, "item": "alpha", "seconds_per_judged": judged, "seconds_per_reused": reused,
                                           "judged_from": None, "reused_from": None})
    task_id = world.submit(item_entry_command(world, ("alpha",)), cwd=world.repo, extra={lib.KEY_VARIABLE: ITEM_PARENT_KEY})
    queued = item_parent_queued(world, task_id)
    text, seconds, basis = estimate_status(world, task_id) if queued is not None else ("", None, None)
    world.resources(local=(50, 50))
    world.finished(task_id)
    if (queued is None or f"预计用时 {ESTIMATE_ITEM_SECONDS} 秒（照点到的每一项" not in text or "照这条命令最近一次真跑完的用时" in text
            or seconds != ESTIMATE_ITEM_SECONDS or basis != "items"):
        raise SelftestFailure(f"按项估的任务（现判 3 × 2 秒 + 复用 4 × 0.5 秒）排着时：status 应当打「预计用时 {ESTIMATE_ITEM_SECONDS} 秒（照点到的每一项…」而不打按命令那一句、"
                              f"回复里 estimate_basis 是 items，实际{'没排上队' if queued is None else ''}文字「{text.strip()[:300]}」、estimated_seconds {seconds}、estimate_basis {basis}")
    return "预计用时照项相加得的：回复里 estimate_basis=items，status 打按项那一句、不打按命令那一句"


def cell_estimate_basis_none_sentence(world):
    """排着的任务没有预计用时（这条命令从没跑成过）：服务交 estimate_basis=none，客户端 status <号> 打「预计用时 没有记录」与没有可用记录的那一句，
    不打「照这条命令最近一次真跑完的用时」（没有这样一次用时）。弄坏开关 estimate-basis-unreported 下服务不交、estimate-basis-sentence-fixed 下客户端一律打按命令那一句，这一格红。"""
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "x", "never-ran"])
    text, seconds, basis = estimate_status(world, task_id)
    world.resources(local=(30, 30))
    world.finished(task_id)
    if "预计用时 没有记录（没有可用的记录" not in text or "照这条命令最近一次真跑完的用时" in text or seconds is not None or basis != "none":
        raise SelftestFailure(f"从没跑成过的命令：status 应当打「预计用时 没有记录（没有可用的记录…」而不打按命令那一句、回复里 estimate_basis 是 none、estimated_seconds 是 null，"
                              f"实际文字「{text.strip()[:300]}」、estimated_seconds {seconds}、estimate_basis {basis}")
    return "没有预计用时的：回复里 estimate_basis=none，status 打「没有记录（没有可用的记录…」、不打按命令那一句"


def merge_child_queued(summary):
    """分片父任务的合并那一步已建出来、还排着。"""
    merge = next((child for child in summary.get("children", []) if child["task"].get("role") == "merge"), None)
    return merge is not None and merge["task"].get("state") == "queued"


def cell_shard_plan_and_merge_estimated_by_command(world):
    """分片的计划与合并两步照同一条命令最近一次真跑完的用时估（task_server_lib.py 文件头「资源记录」的预计用时）：命令键各带那一步一段，计划记过 120 秒、合并记过 45 秒。
    计划排着（本机放不下任何东西）时报 120；放出来之后计划与那一片跑完，合并记过 20G 的峰值、本机空闲 10G 放不下，排着时报 45。再放出来整件成了。
    弄坏开关 shard-plan-merge-unestimated 下两步都没有预计用时，estimate-unreported 下一律没有，这一格红。"""
    spec = world.write_spec("spec-estimate-plan-merge.json", {"shards": [{}]})
    plan_key, merge_key = (lib.shard_child_key(SHARD_ESTIMATE_KEY, role, 0, 0) for role in ("plan", "merge"))
    world.seed_duration(plan_key, SHARD_PLAN_SECONDS, spec)
    world.seed_duration(merge_key, SHARD_MERGE_SECONDS, spec)
    world.seed_peak(merge_key, SHARD_ESTIMATE_PEAK_BYTES)
    world.resources(local=(10, 50), hold=True)
    world.start_server()
    task_id = world.submit(spec, cwd=world.repo, extra={lib.KEY_VARIABLE: SHARD_ESTIMATE_KEY})
    plan = next((child for child in world.summary(task_id).get("children", []) if child["task"].get("role") == "plan"), {})
    plan_estimate = plan.get("estimated_seconds", "计划那一步没建出来")
    world.resources(local=(10, 50))
    middle = world.wait_for(task_id, merge_child_queued, "合并那一步排着", timeout=SHARD_ESTIMATE_WAIT_SECONDS)
    merge = next(child for child in middle["children"] if child["task"].get("role") == "merge")
    merge_estimate = merge.get("estimated_seconds", "合并那一步没有这一栏")
    world.resources(local=(50, 50))
    finished = world.finished(task_id)
    if plan_estimate != SHARD_PLAN_SECONDS or merge_estimate != SHARD_MERGE_SECONDS or finished["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"计划记过 {SHARD_PLAN_SECONDS} 秒、合并记过 {SHARD_MERGE_SECONDS} 秒：计划排着时预计用时应当是 {SHARD_PLAN_SECONDS}、合并排着时应当是 {SHARD_MERGE_SECONDS}、整件成了，"
                              f"实际计划 {plan_estimate}、合并 {merge_estimate}、整件 {describe(finished)}", world.server_log_lines())
    return f"分片的计划排着时报 {SHARD_PLAN_SECONDS} 秒、合并排着时报 {SHARD_MERGE_SECONDS} 秒（各照自己命令键最近一次跑成的用时）"


def cell_shard_pieces_estimated_by_own_history(world):
    """分片的各片照这一片自己命令键最近一次真跑完的用时估（task_server_lib.py 文件头「资源记录」的预计用时；计划那一步不交每片的项，按项估做不到）：两片的命令键里片号不同，
    第 0 片记过 300 秒、第 1 片记过 900 秒，各记过 20G 的峰值、本机空闲 10G 两片都放不下，计划跑完之后两片都排着，各报自己的 300、900（不混）。再放出来整件成了。
    弄坏开关 shard-pieces-unestimated 下各片没有预计用时，shard-key-position-folded 下两片落一个键、各报 900，estimate-unreported 下一律没有，这一格红。"""
    spec = world.write_spec("spec-estimate-pieces.json", {"shards": [{}, {}]})
    for index, seconds in SHARD_PIECE_SECONDS.items():
        piece_key = lib.shard_child_key(SHARD_ESTIMATE_KEY, "shard", index, len(SHARD_PIECE_SECONDS))
        world.seed_peak(piece_key, SHARD_ESTIMATE_PEAK_BYTES)
        world.seed_duration(piece_key, seconds, spec)
    world.resources(local=(10, 50))
    world.start_server()
    task_id = world.submit(spec, cwd=world.repo, extra={lib.KEY_VARIABLE: SHARD_ESTIMATE_KEY})
    middle = world.wait_for(task_id, lambda summary: queued_shards(summary) == sorted(SHARD_PIECE_SECONDS), "两片都排着", timeout=SHARD_ESTIMATE_WAIT_SECONDS)
    found = {index: child.get("estimated_seconds", "这一片没有这一栏") for index, child in shard_children(middle).items()}
    world.resources(local=(100, 100))
    finished = world.finished(task_id)
    if found != SHARD_PIECE_SECONDS or finished["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"两片的命令键里片号不同、各记过 {SHARD_PIECE_SECONDS} 秒：排着时各片预计用时应当是自己的 {SHARD_PIECE_SECONDS}、整件成了，"
                              f"实际各片 {found}、整件 {describe(finished)}", world.server_log_lines())
    return f"分片的各片排着时各报自己那一片记过的用时 {SHARD_PIECE_SECONDS}（片号不同的不混）"


ROOM_SLICE_GIBIBYTES = 200                    # 插空看排头那两格的台：总上限 200G、余量 2G，可用 198G，界 168.3G
ROOM_FILLER_CAP_BYTES = 82 * GIBIBYTE         # 那两格先装进组的两条碎片：上限与记账各 82G（峰值是它的三分之二）
ROOM_FILLER_KEY = "selftest-room-filler"
ROOM_FILLER_SECONDS = 100                     # 装组的两条记下的用时（比挡路的短）：挡路时长取挡路那条的，不随弄坏开关 blocker-remaining-unbounded 变
ROOM_WHOLE_KEY = "selftest-room-whole"
ROOM_BLOCKER_KEY = "selftest-room-blocker"    # 挡路的那条碎片：没记峰值（4G），记下的用时 ROOM_BLOCKER_SECONDS；装进组之后组是 82 + 82 + 4 = 168G，再放一条 4G 就超过界、封组
ROOM_BLOCKER_SECONDS = 300
ROOM_PROBE_KEY = "selftest-room-probe"        # 来插空的五条碎片的命令键前缀（第 n 条接 -n，各一个键）：没记峰值（各 4G），第 n 条记下 600 + n 秒的用时（都比挡路的长）
ROOM_PROBE_SECONDS = 600                      # 一条比一条长一秒：同一轮里刚起的那几条还要跑满自己的用时，后到的比它们长才看得出「超过挡路时长」
ROOM_PROBE_COUNT = 5
ROOM_PROBE_BYTES = 4 * GIBIBYTE
ROOM_WHOLE_ROOMY_BYTES = 170 * GIBIBYTE       # gap-fill-by-room-skips-length 的排头整片：整片之外照样空着的那一份 198 − 170 = 28G，五条都在跑时还剩 8G，五条都放得下
ROOM_WHOLE_TIGHT_BYTES = 188 * GIBIBYTE       # gap-fill-room-exhausted-checks-length 的排头整片：那一份 198 − 188 = 10G，放得下两条，第三条只剩 2G
                                              # （挡路的那条与装组的两条是封组之前起的，不减：task_server.py 的 Alternation.gap_fill）
ROOM_FREE_GIBIBYTES = 40                      # 先装进组的两条跑完之前把本机空闲换成这么多：整片（要的空闲一百多 G）放不下、一直等，碎片放得下
ROOM_START_WAIT_SECONDS = 30.0
ROOM_GAP_ADMITTED_TEXT = "不看预计用时"         # task_server.py 的 Alternation.gap_fill 照排头起来之后还剩多少放进去时那一句的结尾


def room_gap_world(world, release, filler_release, whole_bytes):
    """插空看排头那两格的世界（task_server.py 的 Alternation.gap_fill）：一台 200G 上依次交整片（记 whole_bytes）、两条先装进组的碎片（各 82G，记了 100 秒，跑到 filler_release 在了才退）、
    一条挡路碎片（4G，记了 300 秒，跑到 release 在了才退）、五条来插空的碎片（各 4G，第 n 条记了 600 + n 秒，跑到 release 在了才退）；一齐放出来：整片让一组碎片先起，
    两条装组的与挡路的起了，组 168G，五条照封组判：整片之外照样空着的那一份（198G − 整片的记账 − 已插进去的）放得下的照内存那一支起（装组的还在跑也起：
    它们是封组之前起的，整片本来就等它们跑完），放不下的看时长，600 秒起超过挡路的 300 秒，封着。
    之后本机空闲换成 40G（整片放不下），放行装组的两条、等它们跑完。交 {名: 任务号}（「整片」「挡路」与「插空」五条的列表）。"""
    world.resources(local=(ROOM_SLICE_GIBIBYTES, ROOM_SLICE_GIBIBYTES), hold=True)
    world.seed_peak(ROOM_WHOLE_KEY, whole_bytes * 2 // 3)
    world.seed_peak(ROOM_FILLER_KEY, ROOM_FILLER_CAP_BYTES * 2 // 3)
    blocker_command = ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "room-blocker", release]
    world.seed_duration(ROOM_BLOCKER_KEY, ROOM_BLOCKER_SECONDS, blocker_command)
    probe_commands = [["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, f"room-probe-{number}", release] for number in range(ROOM_PROBE_COUNT)]
    for number, command in enumerate(probe_commands):
        world.seed_duration(f"{ROOM_PROBE_KEY}-{number}", ROOM_PROBE_SECONDS + number, command)
    filler_commands = [["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, f"room-filler-{number}", filler_release] for number in range(2)]
    for command in filler_commands:
        world.seed_duration(ROOM_FILLER_KEY, ROOM_FILLER_SECONDS, command)
    world.start_server()
    tasks = {"整片": world.submit(["bash", "-c", "exit 0", "room-whole"], extra={lib.KEY_VARIABLE: ROOM_WHOLE_KEY})}
    tasks["装组"] = [world.submit(command, extra={lib.KEY_VARIABLE: ROOM_FILLER_KEY}) for command in filler_commands]
    tasks["挡路"] = world.submit(blocker_command, extra={lib.KEY_VARIABLE: ROOM_BLOCKER_KEY})
    tasks["插空"] = [world.submit(command, extra={lib.KEY_VARIABLE: f"{ROOM_PROBE_KEY}-{number}"}) for number, command in enumerate(probe_commands)]
    world.resources(local=(ROOM_SLICE_GIBIBYTES, ROOM_SLICE_GIBIBYTES))
    world.api("POST", "/wake")
    for task_id in tasks["装组"] + [tasks["挡路"]]:
        world.running(task_id)
    world.resources(local=(ROOM_FREE_GIBIBYTES, ROOM_SLICE_GIBIBYTES))
    world.next_pass()
    world.next_pass()
    open(filler_release, "w").close()
    for task_id in tasks["装组"]:
        world.finished(task_id)
    return tasks


def room_probe_started(world, task_id):
    """来插空的那一条在时限里起没起：交 (起了没有, summary)。"""
    try:
        return True, world.wait_for(task_id, lambda summary: bool(summary["attempts"]), "插空起来", timeout=ROOM_START_WAIT_SECONDS)
    except SelftestFailure:
        return False, world.summary(task_id)


def room_admitted_problem(name, started, summary):
    note = first_attempt(summary).get("gap_fill") or ""
    if not started or not note.startswith(GAP_ADMITTED_TEXT) or ROOM_GAP_ADMITTED_TEXT not in note:
        return f"{name}应当照排头起来之后还剩多少插空起来、gap_fill 写「{ROOM_GAP_ADMITTED_TEXT}」，实际 {describe(summary)}、gap_fill「{note}」、排队原因「{summary.get('waiting_for') or ''}」"
    return ""


def release_room_world(release, filler_release):
    for path in (release, filler_release):
        open(path, "w").close()


def cell_gap_fill_by_room_skips_length(world):
    """封着组时不占排头将来要的那部分内存的碎片不看时长（task_server.py 的 Alternation.gap_fill）：排头整片记 170G，五条 4G、记了 600 到 604 秒的碎片先后到，
    挡路的那条记了 300 秒：整片之外照样空着的那一份（198G − 170G = 28G）五条都放得下，五条都起、尝试记「不看预计用时」，比挡路时长长也照起。
    弄坏开关 gap-fill-room-ignored 下照改前看时长、一条都不起，sealed-gap-not-backfilled 下一条都不起。"""
    release, filler_release = os.path.join(world.root, "release-room"), os.path.join(world.root, "release-room-fillers")
    problems = []
    try:
        tasks = room_gap_world(world, release, filler_release, ROOM_WHOLE_ROOMY_BYTES)
        for number, task_id in enumerate(tasks["插空"]):
            started, summary = room_probe_started(world, task_id)
            problem = room_admitted_problem(f"来插空的第 {number} 条", started, summary)
            problems += [problem] if problem else []
        whole = world.summary(tasks["整片"])
        if whole["attempts"]:
            problems.append(f"整片本机空闲不够、应当一直排着，实际 {describe(whole)}")
    finally:
        release_room_world(release, filler_release)
    if problems:
        raise SelftestFailure("排头起来之后还放得下的碎片不看时长：" + "；".join(problems), world.server_log_lines())
    return f"排头整片起来之后照公式还放得下：{ROOM_PROBE_COUNT} 条记了 {ROOM_PROBE_SECONDS} 秒起的碎片都插空起来，不比挡路时长"


def cell_gap_fill_room_exhausted_checks_length(world):
    """封着组时会占排头那部分内存的碎片才看时长（task_server.py 的 Alternation.gap_fill）：排头整片记 188G，整片之外照样空着的那一份（198G − 188G = 10G）只放得下两条 4G 的：
    前两条不看时长插空起来，第三条起只剩 2G，照预计用时比挡路时长：602 秒起超过挡路时长（前两条刚起，还要跑满它们的 600、601 秒），封着，排队原因写超过挡路时长与排头起来之后只剩多少。
    弄坏开关 sealed-gap-length-ignored 下第三条起也照起，gap-fill-room-ignored 下前两条也看时长、不起，sealed-gap-not-backfilled 下一条都不起，
    gap-fill-room-counts-all-running 下照改前连挡路的 4G 一起减、只放得下一条。"""
    release, filler_release = os.path.join(world.root, "release-room"), os.path.join(world.root, "release-room-fillers")
    problems = []
    try:
        tasks = room_gap_world(world, release, filler_release, ROOM_WHOLE_TIGHT_BYTES)
        for number, task_id in enumerate(tasks["插空"][:2]):
            started, summary = room_probe_started(world, task_id)
            problem = room_admitted_problem(f"来插空的第 {number} 条", started, summary)
            problems += [problem] if problem else []
        world.next_pass()
        world.next_pass()
        for number, task_id in enumerate(tasks["插空"][2:], start=2):
            summary = world.summary(task_id)
            reason = summary.get("waiting_for") or ""
            refusal = f"{GAP_REFUSED_TEXT}这一条预计用时 {ROOM_PROBE_SECONDS + number:.1f} 秒，超过挡路时长"
            if summary["attempts"] or refusal not in reason or "放不下这一条的" not in reason:
                problems.append(f"来插空的第 {number} 条照公式已放不下、应当看时长而封着，排队原因带「{refusal}」与排头起来之后放不下的那一句，"
                                f"实际尝试 {len(summary['attempts'])} 次、排队原因「{reason}」")
    finally:
        release_room_world(release, filler_release)
    if problems:
        raise SelftestFailure("排头起来之后放不下的碎片看时长：" + "；".join(problems), world.server_log_lines())
    return f"排头整片起来之后照公式只放得下两条：前两条不看时长插空起来，第三条起看时长、{ROOM_PROBE_SECONDS + 2} 秒起超过挡路时长封着"


def cell_margin_measured_per_host(world):
    """余量按台现量（task_server.py 文件头 ②「余量」）：本机此刻余量 1G、第二台 3G，两台 slice 总上限都是 50G：记录峰值 40G（上限 60G）的那一条
    在本机起上限与记账是 50G 减 1G（PER_HOST_LOCAL_CAP_TEXT）；本机放不下时在第二台起上限与记账是 50G 减 3G（PER_HOST_PEER_CAP_TEXT）。
    弄坏开关 margin-local-for-both 下第二台也减本机的 1G；booked-up-to-slice-total 下两台的记账都不减（judge_host 另把上限收到总上限减余量，只看上限分不出）。"""
    key = "selftest-margin-per-host"
    world.seed_peak(key, 40 * GIBIBYTE)
    world.resources(local=(100, 50), local_margin=PER_HOST_LOCAL_MARGIN_GIBIBYTES)   # 只有本机：在本机起
    world.start_server()
    problems = []
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "margin-per-host-local"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(summary)
    if (attempt.get("host") != "local" or attempt.get("memory_cap_text") != PER_HOST_LOCAL_CAP_TEXT or attempt.get("booked") != PER_HOST_LOCAL_CAP_BYTES
            or summary["task"].get("outcome") != "succeeded"):
        problems.append(f"本机余量 {PER_HOST_LOCAL_MARGIN_GIBIBYTES}G：应当在本机起、上限与记账 {PER_HOST_LOCAL_CAP_TEXT}（50G 减 1G）、跑完，"
                        f"实际台 {attempt.get('host')}、记账 {attempt.get('booked')} 字节、{describe(summary)}")
    world.resources(local=(1, 50), peer=(100, 50), peer_cap_gibibytes=64, local_margin=PER_HOST_LOCAL_MARGIN_GIBIBYTES, peer_margin=PER_HOST_PEER_MARGIN_GIBIBYTES)
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "margin-per-host-peer"], extra={lib.KEY_VARIABLE: key}), timeout=PLACEMENT_WAIT_SECONDS)
    attempt = first_attempt(summary)
    if (attempt.get("host") != "peer" or attempt.get("memory_cap_text") != PER_HOST_PEER_CAP_TEXT or attempt.get("booked") != PER_HOST_PEER_CAP_BYTES
            or summary["task"].get("outcome") != "succeeded"):
        problems.append(f"本机只空 1G、第二台余量 {PER_HOST_PEER_MARGIN_GIBIBYTES}G：应当在第二台起、上限与记账 {PER_HOST_PEER_CAP_TEXT}（50G 减 3G，不是减本机的 1G）、跑完，"
                        f"实际台 {attempt.get('host')}、记账 {attempt.get('booked')} 字节、{describe(summary)}")
    if problems:
        raise SelftestFailure("余量按台：" + "；".join(problems), world.server_log_lines())
    return f"本机余量 1G、第二台 3G：同一条在本机起上限 {PER_HOST_LOCAL_CAP_TEXT}、在第二台起上限 {PER_HOST_PEER_CAP_TEXT}，各用各的"


def cell_margin_unmeasured_not_placed(world):
    """余量量不到不派、不退回固定值（task_server.py 文件头 ②「余量」）：本机报告里余量是 null（量不到）、第二台量得到：这一条不在本机起、在第二台起；
    两台都量不到：一次都不起，结局 run-condition-unmet（退 78），说明点名两台各自量不到余量。看排两轮之后的样子，不比秒数。
    弄坏开关 margin-fallback-fixed 下本机退回 2G、照在本机起；margin-unmeasured-waits 下两台都量不到时照排着。"""
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64, local_margin=None)
    world.start_server()
    problems = []
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "margin-unmeasured-local"]), timeout=PLACEMENT_WAIT_SECONDS)
    attempt = first_attempt(summary)
    if attempt.get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
        problems.append(f"本机量不到余量、第二台量得到：应当只在第二台起、跑完（本机两台都空 100G，退回固定 2G 的写法下会挑本机），实际台 {attempt.get('host')}、{describe(summary)}")
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64, local_margin=None, peer_margin=None)
    task_id = world.submit(["bash", "-c", "exit 0", "margin-unmeasured-both"])
    world.next_pass()
    world.next_pass()
    summary = world.summary(task_id)
    task = summary["task"]
    note = task.get("note") or ""
    if (task.get("outcome") != "run-condition-unmet" or task.get("exit_code") != lib.RUN_CONDITION_EXIT or summary["attempts"] or world.calls_of(task_id)
            or "本机此刻量不到余量" not in note or "第二台此刻量不到余量" not in note):
        if task.get("state") == "queued":
            world.client(["cancel", str(task_id)])
        problems.append(f"两台都量不到余量：应当一次都不起、结局 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}）、说明点名本机与第二台都量不到余量，"
                        f"实际状态 {task.get('state')}、{describe(summary)}、内存包装起了 {len(world.calls_of(task_id))} 次、说明「{note[:300]}」、"
                        f"排队原因「{summary.get('waiting_for')}」")
    if problems:
        raise SelftestFailure("余量量不到：" + "；".join(problems), world.server_log_lines())
    return "本机量不到余量时只派第二台；两台都量不到时一次都不起、记 run-condition-unmet、说明点名两台"


def cell_pin_cap_refused_within_margin(world):
    """钉值不大于 0（task_server.py 文件头 ②「内存」的钉值）：SLICE_MEMORY_CAP_BY_HOST 两台都写 2G、两台此刻的余量都是假余量 2G（减完是 0）：
    control pin-cap 当场拒（退非 0、说明点名 SLICE_MEMORY_CAP_BY_HOST 与把那一项改大的出路）、没钉；配置写 20G 时钉上的命令，配置改成 2G 之后再交：
    两台都不派、结局 run-condition-unmet（退 78）、说明写钉值不大于 0。弄坏开关 pinned-nonpositive-accepted 下照钉、照派。"""
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64)
    world.set_slice_cap("2G")
    world.start_server()
    problems = []
    refused_probe = ["bash", "-c", "exit 0", "pin-within-margin-refused"]
    refused = world.control(["pin-cap", "--", *refused_probe])
    said = (refused.stdout + refused.stderr).strip()
    if refused.returncode == 0 or lib.PIN_SOURCE not in said or "改大" not in said or "钉上了" in said:
        problems.append(f"两台那一项 2G 不比余量 2G 大：control pin-cap 应当退非 0、说明点名 {lib.PIN_SOURCE} 与改大的出路、不说钉上了，实际退 {refused.returncode}、「{said[-400:]}」")
    left = world.control(["unpin-cap", "--", *refused_probe])
    if left.returncode != 0 or "原来就没钉" not in left.stdout:
        problems.append(f"拒了的 pin-cap 不该钉上：unpin-cap 应当说原来就没钉，实际退 {left.returncode}、「{(left.stdout + left.stderr).strip()[-300:]}」")
    world.set_slice_cap("20G")
    probe = ["bash", "-c", "exit 0", "pin-within-margin-dispatch"]
    pinned = world.control(["pin-cap", "--", *probe])
    if pinned.returncode != 0 or "钉上了" not in pinned.stdout:
        problems.append(f"配置写 20G 时 control pin-cap 应当退 0、说钉上了，实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-300:]}」")
    world.set_slice_cap("2G")
    task_id = world.submit(probe)
    world.next_pass()
    world.next_pass()
    summary = world.summary(task_id)
    task = summary["task"]
    note = task.get("note") or ""
    if (task.get("outcome") != "run-condition-unmet" or task.get("exit_code") != lib.RUN_CONDITION_EXIT or summary["attempts"] or world.calls_of(task_id)
            or "不大于 0" not in note):
        if task.get("state") == "queued":
            world.client(["cancel", str(task_id)])
        problems.append(f"钉了的命令在配置改成 2G 之后：两台的钉值都不大于 0，应当一次都不起、结局 run-condition-unmet（退 {lib.RUN_CONDITION_EXIT}）、说明写钉值不大于 0，"
                        f"实际状态 {task.get('state')}、{describe(summary)}、说明「{note[:300]}」、排队原因「{summary.get('waiting_for')}」")
    if problems:
        raise SelftestFailure("钉值不大于 0：" + "；".join(problems), world.server_log_lines())
    return "那一项不比余量大时 control pin-cap 当场拒、没钉；已钉的命令配置改小到不比余量大之后不派、记 run-condition-unmet"


# peer-booked-like-local：峰值 13653M 乘 1.5 按 MiB 向上取整是 20480M（20G）；在跑的每条此刻回收不掉 1G；两台都是 slice 50G、余量 1G、空闲 40G
BOTH_HOSTS_PEAK_BYTES = 13653 * MEBIBYTE
BOTH_HOSTS_CAP_TEXT = "20480M"
BOTH_HOSTS_USED_BYTES = GIBIBYTE
BOTH_HOSTS_RELEASE = "release-both-hosts"   # 相对假仓根：本机那几条在 world.repo 里等它，假第二台照 fake-peer-wait 等同一个名字
SELFTEST_TIGHT_MARGIN_GIBIBYTES = 1         # 这几格两台此刻的余量（两台相同：margin-local-for-both 下第二台读本机那一项，判定不变）


def occupy_hosts(world, command, key, hosts_running):
    """照 hosts_running 的先后在指定的台上起一条条占着到放行为止的任务：派本机那几条时假余量只给本机（第二台没报告），派第二台那几条时本机给不出上限
    （总上限 1G 不比余量 1G 大，这一轮不派本机：不是放不下，peer-overflow-blocked 那一种不挡）。交 {台: [任务号…]}，每一条都核过起在那一台。"""
    started = {"local": [], "peer": []}
    for host in hosts_running:
        if host == "local":
            world.resources(local=(100, 100), local_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES)
        else:
            world.resources(local=(1, 1), local_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES, peer=(100, 100), peer_cap_gibibytes=100,
                            peer_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES)
        task_id = world.submit(command, extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
        attempt = first_attempt(world.running(task_id))
        if attempt.get("host") != host:
            raise SelftestFailure(f"占着{lib.HOST_NAMES.get(host, host)}的那一条应当起在{lib.HOST_NAMES.get(host, host)}，实际起在 {attempt.get('host')}", world.server_log_lines())
        started[host].append(task_id)
    return started


def cell_peer_booked_like_local(world):
    """第二台与本机同一段（task_server.py 文件头 ②「内存」的准入按上限记账）：两台各三条在跑、各上限 20G、各自此刻回收不掉 1G（两台都照那一台报告里 label
    对得上的那个 scope）；两台都是 slice 50G、余量 1G、空闲 40G、进程用 3G：在跑的每条记 19G 待涨量，第四条 20G
    （要的空闲约 14.3G：预测 13653M 加余量 1G）两台都放不下（每台上限之和 60G 加它超过总上限减余量 49G），排两轮之后照排着、排队原因两台都写。放行之后第四条照起、跑完。
    弄坏开关 server-pending-ignored 下在跑的不记待涨量：第四条放得下、起了。"""
    world.seed_peak("selftest-both-hosts", BOTH_HOSTS_PEAK_BYTES)
    world.resources(local=(100, 100), local_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES)
    world.start_server()
    command = ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "both-hosts", BOTH_HOSTS_RELEASE, f"fake-peer-wait={BOTH_HOSTS_RELEASE}"]
    problems = []
    try:
        started = occupy_hosts(world, command, "selftest-both-hosts", ["local"] * 3 + ["peer"] * 3)
        labels = [first_attempt(world.summary(task_id)).get("label") for task_id in started["peer"]]
        local_labels = [first_attempt(world.summary(task_id)).get("label") for task_id in started["local"]]
        caps = [first_attempt(world.summary(task_id)).get("memory_cap_text") for task_id in started["local"] + started["peer"]]
        if caps != [BOTH_HOSTS_CAP_TEXT] * 6 or not all(labels + local_labels):
            problems.append(f"占着两台的六条应当各上限 {BOTH_HOSTS_CAP_TEXT}、都有 label，实际上限 {caps}、label {labels + local_labels}")
        world.resources(local=(40, 50), local_slice_used=3, local_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES, local_scopes={label: BOTH_HOSTS_USED_BYTES for label in local_labels},
                        peer=(40, 50), peer_cap_gibibytes=50, peer_slice_used=3, peer_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES,
                        peer_scopes={label: BOTH_HOSTS_USED_BYTES for label in labels})
        fourth = world.submit(["bash", "-c", "exit 0", "both-hosts-fourth", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-both-hosts"}, cwd=world.repo)
        world.next_pass()
        world.next_pass()
        waiting = world.summary(fourth)
        reason = waiting.get("waiting_for") or ""
        if waiting["task"].get("state") != "queued" or waiting["attempts"] or "本机：" not in reason or "第二台：" not in reason:
            problems.append(f"两台各三条 20G 在跑（各用 1G）、slice 50G、余量 1G：第四条 20G 两台都放不下，应当排着、排队原因两台都写；"
                            f"实际状态 {waiting['task'].get('state')}、{describe(waiting)}、排队原因「{reason[:300]}」")
    finally:
        open(os.path.join(world.repo, BOTH_HOSTS_RELEASE), "w").close()
    summaries = [world.finished(task_id, timeout=120) for task_id in [*started.get("local", []), *started.get("peer", []), fourth]] if not problems else []
    if any(summary["task"].get("outcome") != "succeeded" for summary in summaries):
        problems.append(f"放行之后七条都应当跑完，实际 {[describe(summary) for summary in summaries]}")
    if problems:
        raise SelftestFailure("两台同一个口径：" + "；".join(problems), world.server_log_lines())
    return "两台各三条 20G 在跑、各用 1G：第二台照报告里的 scope 记待涨量、同本机，第四条 20G 两台都排着；放行之后跑完"


# peer-scope-missing-booked-at-cap：峰值 10G，上限与记账 15G（15360M）；报告里那条 scope 此刻回收不掉 6G
MISSING_SCOPE_PEAK_BYTES = 10 * GIBIBYTE
MISSING_SCOPE_CAP_TEXT = "15360M"
MISSING_SCOPE_USED_BYTES = 6 * GIBIBYTE
MISSING_SCOPE_RELEASE = "release-missing-scope"


def cell_peer_scope_missing_booked_at_cap(world):
    """第二台报告里缺某条在跑尝试的 scope（还没起来、读不了）：那一条记整份上限，不记 0（task_server.py 的 server_pending）。只派第二台（本机总上限 1G 不比余量 1G 大）。
    第二台 slice 50G、余量 1G、空闲 30G，两条在跑各上限 15G：报告里只有其中一条的 scope（回收不掉 6G、进程用 6G）时待涨量 9G + 15G、空闲剩 6G，
    第三条 15G（要的空闲 11G：预测 10G 加余量 1G）放不下、排两轮照排着（缺的记 0 的写法下待涨量 9G、空闲剩 21G，放得下）；
    报告补上另一条的 scope（也是 6G、进程共用 12G）之后待涨量 9G + 9G、空闲剩 12G、slice 减余量剩 19G：第三条放得下、起在第二台（一律记整份上限的写法下待涨量 30G，照样放不下）。
    弄坏开关 server-pending-ignored 下在跑的不记待涨量：头一段第三条就起了。"""
    world.seed_peak("selftest-missing-scope", MISSING_SCOPE_PEAK_BYTES)
    world.resources(local=(1, 1), local_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES)
    world.start_server()
    command = ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "missing-scope", MISSING_SCOPE_RELEASE, f"fake-peer-wait={MISSING_SCOPE_RELEASE}"]
    problems = []
    third = None
    try:
        started = occupy_hosts(world, command, "selftest-missing-scope", ["peer"] * 2)["peer"]
        labels = [first_attempt(world.summary(task_id)).get("label") for task_id in started]
        caps = [first_attempt(world.summary(task_id)).get("memory_cap_text") for task_id in started]
        if caps != [MISSING_SCOPE_CAP_TEXT] * 2 or not all(labels):
            problems.append(f"占着第二台的两条应当各上限 {MISSING_SCOPE_CAP_TEXT}、都有 label，实际上限 {caps}、label {labels}")
        scenario = {"local": (1, 1), "local_margin": SELFTEST_TIGHT_MARGIN_GIBIBYTES, "peer": (30, 50), "peer_cap_gibibytes": 50,
                    "peer_margin": SELFTEST_TIGHT_MARGIN_GIBIBYTES}
        world.resources(**scenario, peer_slice_used=6, peer_scopes={labels[0]: MISSING_SCOPE_USED_BYTES})
        third = world.submit(["bash", "-c", "exit 0", "missing-scope-third", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-missing-scope"}, cwd=world.repo)
        world.next_pass()
        world.next_pass()
        waiting = world.summary(third)
        if waiting["task"].get("state") != "queued" or waiting["attempts"]:
            problems.append(f"报告里缺一条在跑的 scope：那一条应当记整份上限 15G，第三条放不下、排着；实际状态 {waiting['task'].get('state')}、{describe(waiting)}、"
                            f"排队原因「{(waiting.get('waiting_for') or '')[:300]}」")
        world.resources(**scenario, peer_slice_used=12, peer_scopes={label: MISSING_SCOPE_USED_BYTES for label in labels})
        placed = world.wait_for(third, lambda found: bool(found["attempts"]) or found["task"].get("state") != "queued", "照补上的 scope 起来")
        if first_attempt(placed).get("host") != "peer":
            problems.append(f"报告补上那一条的 scope 之后第三条应当起在第二台（待涨量 18G），实际 {describe(placed)}、排队原因「{(placed.get('waiting_for') or '')[:300]}」")
    finally:
        open(os.path.join(world.repo, MISSING_SCOPE_RELEASE), "w").close()
    summaries = [world.finished(task_id, timeout=120) for task_id in [*started, third]] if not problems else []
    if any(summary["task"].get("outcome") != "succeeded" for summary in summaries):
        problems.append(f"放行之后三条都应当跑完，实际 {[describe(summary) for summary in summaries]}")
    if problems:
        raise SelftestFailure("第二台缺 scope 的记整份上限：" + "；".join(problems), world.server_log_lines())
    return "第二台报告里缺一条在跑的 scope 时那一条记整份上限、第三条排着；补上之后照「上限 − 用量」记、第三条起在第二台"


# two-hosts-booked-within-capacity：三种各四条（峰值 10G 记 15G、峰值 6G 记 9G、没记录的记 4G，合计 112G）交给两台各 slice 50G、余量 1G（各能记 49G）
INVARIANT_KINDS = (("selftest-invariant-large", 10 * GIBIBYTE), ("selftest-invariant-medium", 6 * GIBIBYTE), ("selftest-invariant-small", None))
INVARIANT_COUNT_PER_KIND = 4
INVARIANT_FILL_PASSES = 6      # 放出来之后看几轮（第二台每轮至多起 PEER_STARTS_PER_PASS 条：12 条几轮里就起到装不下为止）
INVARIANT_DRAIN_PASSES = 40    # 每放行一批之后至多看几轮：在排的起完了就不再看（一轮 0.1 秒，放行的那几条收尾、在排的接着起都在这里面）
INVARIANT_RELEASES = ("release-invariant-early", "release-invariant-late")


def booked_caps_by_host(world):
    """此刻库里在起、在跑的尝试按台的上限之和（字节）与在排的条数：照服务读回的规则核过的那几行，核不过的不算。"""
    caps = {"local": 0, "peer": 0}
    for record, valid in world.signed_rows("attempts"):
        if valid and record.get("state") in ("starting", "running"):
            caps[record.get("host")] = caps.get(record.get("host"), 0) + int(record.get("memory_cap") or 0)
    queued = sum(1 for record, valid in world.signed_rows("tasks") if valid and record.get("state") == "queued")
    return caps, queued


def cell_two_hosts_booked_within_capacity(world):
    """总的不变量（task_server_lib.py 文件头「资源记录」的准入按上限记账）：两台各 slice 50G、余量 1G，12 条合计要记 112G、两台一齐放出来跑一段排队，
    分两批放行：每一轮之后读库，每台在起、在跑的尝试的上限之和都不超过那台总上限减余量 49G（这一格的任务峰值都装得下，没有「峰值本身装不下」的那一种例外）；
    装不下的排着，排过的里有一轮还有在排的，两台都分到，放行之后都跑完。第二台在跑的待涨量照报告里的 scope（这一格的报告不报 scope：记整份上限）。
    弄坏开关 server-pending-ignored 下在跑的不记待涨量，一轮轮接着起、上限之和超过 49G。"""
    for key, peak in INVARIANT_KINDS:
        if peak is not None:
            world.seed_peak(key, peak)
    both = {"local": (200, 50), "local_margin": SELFTEST_TIGHT_MARGIN_GIBIBYTES, "peer": (200, 50), "peer_cap_gibibytes": 50,
            "peer_margin": SELFTEST_TIGHT_MARGIN_GIBIBYTES}
    world.resources(**both, hold=True)
    world.start_server()
    capacity = (50 - SELFTEST_TIGHT_MARGIN_GIBIBYTES) * GIBIBYTE
    tasks = []
    for position in range(INVARIANT_COUNT_PER_KIND):
        for key, _ in INVARIANT_KINDS:
            release = INVARIANT_RELEASES[len(tasks) % len(INVARIANT_RELEASES)]
            command = ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, f"invariant-{position}", release, f"fake-peer-wait={release}"]
            tasks.append(world.submit(command, extra={lib.KEY_VARIABLE: key}, cwd=world.repo))
    world.resources(**both)   # 一齐放出来
    world.api("POST", "/wake")
    samples, over = [], []

    def sample():
        world.next_pass()
        caps, queued = booked_caps_by_host(world)
        samples.append((caps, queued))
        over.extend(f"{lib.HOST_NAMES.get(host, host)}上限之和 {total / GIBIBYTE:.1f}G（还有 {queued} 条在排）" for host, total in caps.items() if total > capacity)
        return queued

    try:
        for _ in range(INVARIANT_FILL_PASSES):
            sample()
        for release in INVARIANT_RELEASES:   # 分两批放行：放了的收尾、在排的接着起，每一轮都照样核
            open(os.path.join(world.repo, release), "w").close()
            for _ in range(INVARIANT_DRAIN_PASSES):
                if not sample():
                    break
    finally:
        for release in INVARIANT_RELEASES:
            open(os.path.join(world.repo, release), "w").close()
    summaries = [world.finished(task_id, timeout=120) for task_id in tasks]
    hosts = {first_attempt(summary).get("host") for summary in summaries}
    problems = []
    if over:
        problems.append(f"每台在跑的上限之和应当不超过总上限减余量 {capacity / GIBIBYTE:.0f}G，实际有 {len(over)} 次超过：{'；'.join(over[:6])}")
    if not any(queued for _, queued in samples):
        problems.append(f"12 条合计 112G 比两台能记的 98G 多：应当有一轮还有在排的，实际各轮在排 {[queued for _, queued in samples]}")
    if hosts != {"local", "peer"} or any(summary["task"].get("outcome") != "succeeded" for summary in summaries):
        problems.append(f"两台都应当分到、都跑完，实际落在 {sorted(host or '-' for host in hosts)}、{[describe(summary) for summary in summaries if summary['task'].get('outcome') != 'succeeded']}")
    if problems:
        raise SelftestFailure("两台的上限之和：" + "；".join(problems), [f"各轮 {caps}、在排 {queued}" for caps, queued in samples] + world.server_log_lines())
    peak = {host: max(caps[host] for caps, _ in samples) / GIBIBYTE for host in ("local", "peer")}
    return f"12 条合计 112G 排在两台各 49G 里：{len(samples)} 轮读库，每台上限之和至多 {peak}G，都跑完"


def cell_judge_error_skips_only_that_task(world):
    """一条这一轮判不了放不放得下（库里它的命令键有一条签过名、上限一栏写坏了的撞上限记录，定额度时 ValueError）：只跳过它，后面那条照判、起、跑完；
    它排着、排队原因写「这一轮判不了放不放得下」（task_server.py 的 place_queued_into）。"""
    world.resources(local=(30, 30))
    world.start_server()
    world.insert_signed_row("attempts", {"task_id": 0, "number": 1, "host": "local", "state": "ended", "session": "", "command_key": "selftest-judge-error",
                                         "outcome": "memory-cap-hit", "memory_cap": "not-a-number", "ended_at": SEEDED_ENDED_AT,
                                         "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC})
    broken_task = world.submit(["bash", "-c", "exit 0", "judge-error"], extra={lib.KEY_VARIABLE: "selftest-judge-error"})
    problems = []
    try:
        later = world.finished(world.submit(["bash", "-c", "exit 0", "after-judge-error"]), timeout=PLACEMENT_WAIT_SECONDS)
        if later["task"].get("outcome") != "succeeded":
            problems.append(f"后面那条应当跑完，实际 {describe(later)}")
    except SelftestFailure as failure:
        problems.append(f"前面一条判不了时后面那条应当照起、跑完，实际没收尾：{failure}")
    found = world.summary(broken_task)
    if found["task"].get("state") != "queued" or "这一轮判不了放不放得下" not in (found.get("waiting_for") or ""):
        problems.append(f"判不了的那条应当排着、排队原因写「这一轮判不了放不放得下」，实际 {found['task'].get('state')}、「{found.get('waiting_for')}」")
    world.client(["cancel", str(broken_task)])
    if problems:
        raise SelftestFailure("一条判不了：" + "；".join(problems), world.server_log_lines())
    return "一条判不了只跳过它（排着、写明判不了），后面那条照起跑完"


EXPLICIT_WAIT_HELD_FRAGMENTS = 3   # explicit-wait-not-starved：一台 20G（可用上限 18G、界 15.3G）上先占着的碎片条数（各记 4G，三条 12G 装进一组）


def cell_explicit_wait_not_starved(world):
    """排头在等交替封组（内存上放得下）：/health 一直回 ok，不当成「放得下却没起」；腾空之后照起跑完（task_server.py 的 starved_problem 只读派活给排头的结论、
    不认排队原因里的字样；任务 2365 那样排头在等一件 /health 不认得的事，合入锁现在不认了，交替封组是同一种：派活判它不能起）。
    看进展（progress_problem）同样不报：这台封着组、服务在这台还有在跑的，碎片照交替等腾空。
    弄坏开关 health-head-judged-apart、health-judges-head 下另判只看内存、blocked-head-counted-as-starved 下判不能起也算，progress-gates-ignored 下看进展不看封组，这一格红。
    一台 20G：三条碎片（没记录、各记 4G）占着装进一组，第四条碎片再放就超过界、又有一条整片（峰值 12G，上限与记账 18G）在等：封组，第四条排着、
    排队原因带「在等这台腾空」；它是排头、内存上放得下（空闲 8G）。"""
    world.resources(local=(20, 20), hold=True)
    world.seed_peak("selftest-explicit-wait-whole", 12 * GIBIBYTE)
    world.start_server()
    release = os.path.join(world.root, "release-explicit-wait")
    held = [world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, f"held-{number}", release]) for number in range(EXPLICIT_WAIT_HELD_FRAGMENTS)]
    head = world.submit(["bash", "-c", "exit 0", "fragment-after-group"])
    whole = world.submit(["bash", "-c", "exit 0", "whole"], extra={lib.KEY_VARIABLE: "selftest-explicit-wait-whole"})
    world.resources(local=(20, 20))   # 一齐放出来
    world.api("POST", "/wake")
    try:
        summary = world.wait_for(head, lambda found: "在等这台腾空" in (found.get("waiting_for") or "") or found["task"].get("state") != "queued",
                                 "报在等这台腾空", timeout=PLACEMENT_WAIT_SECONDS)
        health = health_until(world, lambda found: not found.get("ok"), 4 * SERVER.FAST_STARVED_SECONDS)
    finally:
        open(release, "w").close()
    finished = {task_id: world.finished(task_id, timeout=120) for task_id in [*held, head, whole]}
    problems = []
    if summary["task"].get("state") != "queued" or "在等这台腾空" not in (summary.get("waiting_for") or ""):
        problems.append(f"三条碎片占着一组、整片在等时第四条碎片应当排着、排队原因带「在等这台腾空」，实际 {describe(summary)}、在等「{summary.get('waiting_for')}」")
    if not health.get("ok"):
        problems.append(f"排头在等交替封组时 /health 应当一直回 ok，实际 {health}")
    failed = [describe(found) for found in finished.values() if found["task"].get("outcome") != "succeeded"]
    if failed:
        problems.append(f"腾空之后都应当跑完，实际没成的 {failed}")
    if problems:
        raise SelftestFailure("排头在等明确的事：" + "；".join(problems), world.server_log_lines())
    return "排头在等交替封组（在等这台腾空）时 /health 一直回 ok，腾空之后都跑完"


def cell_selftest_variables_refused(world):
    outside = {name: value for name, value in world.client_environment.items() if name != lib.SELFTEST_MARK_VARIABLE}
    outside[lib.BREAK_VARIABLE] = "exit-code-dropped"
    for script, arguments in ((lib.CLIENT_SCRIPT, ["status"]), (lib.CONTROL_SCRIPT, ["check"]), (lib.SERVER_SCRIPT, ["serve"])):
        completed = subprocess.run([sys.executable, "-B", script, *arguments], env=outside, cwd=world.work, stdin=subprocess.DEVNULL, capture_output=True, text=True,
                                   timeout=60)
        if completed.returncode != 2 or "只在自证" not in completed.stderr:
            raise SelftestFailure(f"平常跑设了 {lib.BREAK_VARIABLE}，{os.path.basename(script)} 应当退 2、点名它只在自证里认，实际退 {completed.returncode}", [completed.stderr])
    return "客户端、control、服务平常跑设了弄坏开关都退 2"


def cell_type_table_covered(world):
    missing = sorted(set(lib.OUTCOME_TYPES) - set(EXERCISED_BY))
    extra = sorted(set(EXERCISED_BY) - set(lib.OUTCOME_TYPES))
    unknown_cells = sorted({cell for cell in EXERCISED_BY.values() if cell not in CELLS})
    if missing or extra or unknown_cells:
        raise SelftestFailure(f"错误类型表每一种都要有格喂到：没格喂的 {missing}、表里没有的 {extra}、点名的格不在 {unknown_cells}")
    return f"错误类型表 {len(lib.OUTCOME_TYPES)} 种都有格喂到"


def cell_nfs_share_required(world):
    """多机配置没写 NFS_SHARED_ROOTS（task_server_lib.py 文件头「NFS 共享」）：服务不起、退 78、点名这个键与 README 那一节；control start 起之前就拒、不起 systemctl。
    两处都要是 NFS 判法拒的（stderr 带 NFS_ROOTS_MISSING_TEXT）：共享数据根与第二台地址的判法也拒这份配置、也点名这个键与 README 那一节，只看这两样认不出 NFS 判法漏没漏。"""
    config = os.path.join(world.root, "no-nfs-multi-host.env")
    World.write_config(config, world.state, world.runtime, world.key, world.builds, "")
    named = (lib.NFS_ROOTS_KEY, "## NFS 共享", lib.NFS_ROOTS_MISSING_TEXT)
    problems = []
    code, output = world.serve_until_exit(config)
    if code != lib.RUN_CONDITION_EXIT or not all(text in output for text in named):
        problems.append(f"服务应当不起、退 78、stderr 点名 {lib.NFS_ROOTS_KEY} 与 README 的「## NFS 共享」、说「{lib.NFS_ROOTS_MISSING_TEXT}」，实际{'起来了' if code is None else f'退 {code}'}、"
                        f"stderr 里缺 {[text for text in named if text not in output]}")
    extra = {lib.MULTI_HOST_CONFIG_VARIABLE: config, "TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl")}
    started = world.control(["start"], extra=extra)
    if started.returncode != lib.RUN_CONDITION_EXIT or not all(text in started.stderr for text in named) or systemctl_start_count(world):
        problems.append(f"control start 应当起之前就退 78、点名同样三样、不起 systemctl start，实际退 {started.returncode}、systemctl start 收到 {systemctl_start_count(world)} 回、"
                        f"stderr「{started.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("没写 NFS 共享：" + "；".join(problems), [output[-1500:]])
    return f"没写 {lib.NFS_ROOTS_KEY}：服务退 78、control start 起之前退 78，都是 NFS 判法拒的（「{lib.NFS_ROOTS_MISSING_TEXT}」），点名这个键与 README「## NFS 共享」"


def cell_nfs_share_verified(world):
    """NFS 共享四项（task_server_lib.py 文件头「NFS 共享」）：假 etab、假 ssh、假探针都过时服务起得来、/health 的 nfs 一栏四项都过、探针文件删了、control check 退 0；
    假 etab 去掉那一行、假 ssh 的 mountpoint 退 1、第二台写的探针本机读不回，服务各自不起、退 78、点名那个根与那一项（起着的服务判出挂载没过发消息给会话由 health-nfs-notified 判）。"""
    problems = []
    world.start_server()
    _, health = world.api("GET", "/health")
    roots = (health.get("nfs") or {}).get("roots") or []
    keys = [key for key, _ in lib.NFS_ITEMS]
    if [entry.get("root") for entry in roots] != [world.shared_root] or any(entry.get(key) != lib.NFS_PASSED for entry in roots for key in keys):
        problems.append(f"/health 的 nfs 一栏应当是 {world.shared_root} 四项都过，实际 {roots}")
    left = [name for name in (lib.NFS_PROBE_NAME, lib.NFS_PEER_PROBE_NAME) if os.path.lexists(os.path.join(world.shared_root, name))]
    if left:
        problems.append(f"两向探针读完应当删掉探针文件，实际还在 {left}")
    git_directory = os.path.join(world.shared_root, ".git")   # 根是 git 工作树：探针写进 .git，不在项目根下留条目
    os.makedirs(git_directory, exist_ok=True)
    try:
        placed = (lib.nfs_probe_directory(world.shared_root), lib.nfs_probe_directory(world.world))
        if placed != (git_directory, world.world):
            problems.append(f"根下有 .git 目录时探针应当写进 {git_directory}、没有时写在根下 {world.world}，实际 {placed}")
    finally:
        os.rmdir(git_directory)
    checked = world.control(["check"])
    if checked.returncode != 0 or "NFS 共享 过" not in checked.stdout:
        problems.append(f"四项都过时 control check 应当退 0、说 NFS 共享过，实际退 {checked.returncode}、「{(checked.stdout + checked.stderr).strip()[-300:]}」")
    mount_exit = os.path.join(world.world, "fake-ssh-mount-exit")
    world.stop_server()
    lost_write = os.path.join(world.world, "fake-ssh-peer-write-lost")
    cases = (("导出", lambda: world.write_exports([]), lambda: world.write_exports([world.shared_root]), ()),
             ("挂载", lambda: open(mount_exit, "w", encoding="utf-8").write("1\n"), lambda: os.remove(mount_exit), ()),
             ("两向探针", lambda: open(lost_write, "w").close(), lambda: os.remove(lost_write), ("第二台→本机",)))
    for item, arm, disarm, more in cases:
        arm()
        try:
            code, output = world.serve_until_exit(world.config)
        finally:
            disarm()
        named = (world.shared_root, f"「{item}」", "## NFS 共享") + more
        if code != lib.RUN_CONDITION_EXIT or not all(text in output for text in named):
            problems.append(f"「{item}」不过时服务应当不起、退 78、点名那个根、那一项与 README 那一节，实际{'起来了' if code is None else f'退 {code}'}、"
                            f"输出里缺 {[text for text in named if text not in output]}")
    if problems:
        raise SelftestFailure("NFS 共享：" + "；".join(problems), world.server_log_lines())
    return "四项都过时起得来、/health 的 nfs 四项都过、探针删了、check 退 0；导出、挂载、两向探针各不过时服务退 78 点名根与那一项"


def cell_peer_unavailable_alerted(world):
    """第二台用不了要告警（task_server.py 的 note_peer）：那一趟没跑成（peer-run-failed）、连不上（假余量的 peer_down）、挂载复核没过（假 ssh 的 mountpoint 退 1）
    各写一条告警 peer-unavailable、/health 的 peer 一栏写明原因（用不了满时限发消息给会话由 health-peer-unavailable-notified 判）。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：派第二台
    world.start_server()
    problems = []
    failed_id = world.submit(["bash", "-c", "exit 0", "fake-peer-exit=255"], cwd=world.repo)
    failed = world.finished(failed_id, timeout=PLACEMENT_WAIT_SECONDS)
    if failed["task"].get("outcome") != "peer-run-failed":
        problems.append(f"假第二台退 255 应当记 peer-run-failed，实际 {describe(failed)}")
    if not any(alert.get("kind") == "peer-unavailable" and alert.get("task") == failed_id and "没跑成" in alert.get("line", "") for alert in world.alerts()):
        problems.append(f"那一趟没跑成应当写一条 peer-unavailable 告警、点名任务 {failed_id}，实际告警 {world.alerts()}")
    _, health = world.api("GET", "/health")
    if ((health.get("peer") or {}).get("last_run_failure") or {}).get("task") != failed_id:
        problems.append(f"/health 的 peer 一栏应当记下最近一次那一趟没跑成（任务 {failed_id}），实际 {health.get('peer')}")
    for what, arm, disarm in (("自证：第二台连不上", lambda: world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_down="自证：第二台连不上"),
                               lambda: world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)),
                              ("没挂上共享根", lambda: open(os.path.join(world.world, "fake-ssh-mount-exit"), "w", encoding="utf-8").write("1\n"),
                               lambda: os.remove(os.path.join(world.world, "fake-ssh-mount-exit")))):
        arm()
        patience = world.patience(PLACEMENT_WAIT_SECONDS)
        alerted = []
        while patience.waiting() and not alerted:
            alerted = [alert for alert in world.alerts() if alert.get("kind") == "peer-unavailable" and what in alert.get("line", "")]
            time.sleep(POLL_SECONDS)
        _, health = world.api("GET", "/health")
        peer = health.get("peer") or {}
        if not alerted or peer.get("available") is not False or what not in str(peer.get("why")):
            problems.append(f"第二台用不了（{what}）应当写一条 peer-unavailable 告警、/health 的 peer 一栏写用不了与原因，实际告警 {alerted}、peer 一栏 {peer}")
        disarm()
        back = health_until(world, lambda health: (health.get("peer") or {}).get("available") is True, STATE_TIMEOUT_SECONDS)
        if (back.get("peer") or {}).get("available") is not True:
            problems.append(f"{what}修好之后第二台应当又用得了（/health 的 peer 一栏），实际 {back.get('peer')}")
            break
    if problems:
        raise SelftestFailure("第二台用不了：" + "；".join(problems), world.server_log_lines())
    return "那一趟没跑成、连不上、挂载复核没过各写 peer-unavailable 告警、/health 写明原因"


REASON_LIVE_CASES = (   # reason-line-per-type 经 task wait 真跑的几种：(类型, 命令, 环境)
    ("command-failed", ["bash", "-c", "echo selftest-reason-last-line; exit 3"], {}),
    ("squeezed-out", ["bash", "-c", "exit 0", "fake-wrapper-exits=254"], {}),
    ("gpu-memory-short", ["bash", "-c", "exit 0", "fake-wrapper-exits=245"], {}),
    ("timed-out", ["bash", "-c", "exit 0", "fake-wrapper-exits=253"], {}),
    ("run-condition-unmet", ["bash", "-c", "echo selftest-reason-unmet; exit 78"], {}),
    ("not-run", ["bash", "-c", "exit 77"], {}),
    ("never-fits", ["bash", "-c", "exit 0"], {lib.KEY_VARIABLE: "selftest-reason-never-fits"}),
)


def reason_problems(line, outcome):
    """一行 TASK_SERVER_REASON 对不对：type 是这一种、REASON_ELEMENTS 给它的要素与 why、next 都在且不空；→ 不对的那几句。"""
    if not line.startswith(lib.REASON_MARK + " "):
        return [f"{outcome}：没有 {lib.REASON_MARK} 那一行（读到「{line[:200]}」）"]
    try:
        fields = dict(word.split("=", 1) for word in shlex.split(line)[1:] if "=" in word)
    except ValueError as error:
        return [f"{outcome}：那一行 shlex 拆不开（{error}）：「{line[:200]}」"]
    wrong = [] if fields.get("type") == outcome else [f"{outcome}：type 应当是 {outcome}，实际 {fields.get('type')}"]
    missing = [name for name in lib.REASON_ELEMENTS.get(outcome, ()) + ("why", "next") if not fields.get(name)]
    return wrong + ([f"{outcome}：缺要素 {missing}（「{line[:300]}」）"] if missing else [])


def cell_reason_line_per_type(world):
    """打回的理由（task_server_lib.py 文件头「错误类型」表后的「打回的理由」）：错误类型表的每一种都在 REASON_ELEMENTS 里、喂一份造好的结局都打得出这一种的要素；
    几种真跑的经 task wait 在末行之前打出 TASK_SERVER_REASON、type 与末行对得上、要素都在（命令自己退的带日志最后一行）。"""
    problems = []
    extra_types = sorted(set(lib.OUTCOME_TYPES) ^ set(lib.REASON_ELEMENTS))
    if extra_types:
        problems.append(f"错误类型表与 REASON_ELEMENTS 对不上：{extra_types}")
    log_text = "GPU_QUIESCE step=quiesce result=selftest-quiesced\nMULTI_HOST_RUN_PEER_CPU poll=1 usage_usec=0 read=no\nselftest-last-line\n"
    for outcome in lib.OUTCOME_TYPES:
        task = {"id": 1, "outcome": outcome, "exit_code": lib.final_exit_code(outcome, 3), "note": f"自证造的说明（{outcome}）", "maximum_host": "local",
                "maximum_cap_text": "46080M", "outcome_counts": {outcome: 2}}
        attempt = {"number": 1, "host": "local", "memory_cap_text": "2048M", "memory_peak": 2 * GIBIBYTE, "card": LOCAL_CARD_ZERO, "unit": "singlefs-task-selftest.service",
                   "squeezed_by": "任务 2（上限 4096M）"}
        problems += reason_problems(lib.reason_line(task, [attempt], log_text), outcome)
    world.resources(local=(100, 100))
    world.seed_peak("selftest-reason-never-fits", 150 * GIBIBYTE)
    world.start_server()
    submitted = [(outcome, world.submit(command, extra=extra)) for outcome, command, extra in REASON_LIVE_CASES]
    for outcome, task_id in submitted:
        world.finished(task_id)
        completed = world.client(["wait", str(task_id)])
        lines = completed.stdout.strip().splitlines()
        if len(lines) < 2 or not lines[-1].startswith(f"{lib.RESULT_MARK} task={task_id} type={outcome} "):
            problems.append(f"任务 {task_id} 应当收尾成 {outcome}、wait 末行写着它，实际末行「{lines[-1] if lines else ''}」")
            continue
        problems += reason_problems(lines[-2], outcome)
        if outcome == "command-failed" and "selftest-reason-last-line" not in lines[-2]:
            problems.append(f"command-failed 的那一行应当带日志最后一行 selftest-reason-last-line，实际「{lines[-2][:300]}」")
    if problems:
        raise SelftestFailure("打回的理由：" + "；".join(problems), world.server_log_lines())
    return (f"错误类型表 {len(lib.OUTCOME_TYPES)} 种各打得出带要素的 {lib.REASON_MARK} 行；真跑的 {len(REASON_LIVE_CASES)} 种 wait 在末行之前打出、type 对得上、要素都在")


def wait_path(world, path, what, timeout=STATE_TIMEOUT_SECONDS):
    """等 path 出现（服务在自证的停顿点建的 reached-<点> 这类）；等不到判红。"""
    patience = world.patience(timeout)
    while patience.waiting():
        if os.path.exists(path):
            return
        time.sleep(POLL_SECONDS)
    raise SelftestFailure(f"等了 {patience.elapsed():.0f} 秒没{what}（{path} 不在；安静时限 {timeout:.0f} 秒）", world.server_log_lines())


def blocking_command(release, counter=None):
    """跑到 release 在了才退 0 的命令；给了 counter 的先往它追加一个 x（数命令跑了几次）。"""
    head = 'printf x >> "$2"; ' if counter else ""
    return ["bash", "-c", head + 'for round in $(seq 1 2000); do [[ -e "$1" ]] && exit 0; sleep 0.1; done; exit 9', "block", release] + ([counter] if counter else [])


def read_text(path):
    with contextlib.suppress(OSError):
        with open(path, encoding="utf-8") as handle:
            return handle.read()
    return ""


def raw_column_update(world, table, column_task_id, row_id):
    """绕过服务只改一行的 task_id 列（正文与签名不动；TS-05 的那一种篡改）。"""
    with contextlib.closing(sqlite3.connect(os.path.join(world.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
        connection.execute(f"UPDATE {table} SET task_id = ? WHERE id = ?", (column_task_id, row_id))


def cell_cancel_start_race(world):
    """TS-01（records/2026-11-01-task-server深入审核.md）：起单元那一侧判过 queued 之后、记尝试之前精确停住再取消：取消回 stopped 之后一次都不起、
    任务不从 finished 回到 running；记了尝试、起单元之前停住再取消：取消等单元起完、停了、回读过才回，结局 cancelled。"""
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    problems = []
    world.resources(local=(30, 30), hold=True)
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"start-checked:{pause}"})
    marker = os.path.join(world.root, "race-ran")
    task_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", marker])
    open(os.path.join(pause, "arm-start-checked"), "w").close()
    world.resources(local=(30, 30))
    wait_path(world, os.path.join(pause, "reached-start-checked"), "停在判过 queued 之后、记尝试之前")
    completed = world.client(["cancel", str(task_id)])
    open(os.path.join(pause, "release-start-checked"), "w").close()
    world.next_pass()
    world.next_pass()
    summary = world.summary(task_id)
    if (completed.returncode != 0 or summary["task"].get("state") != "finished" or summary["task"].get("outcome") != "cancelled"
            or summary["attempts"] or os.path.exists(marker)):
        problems.append(f"判过 queued、记尝试之前取消：取消回了 stopped（退 {completed.returncode}）之后应当一次都不起、任务留在 finished/cancelled，"
                        f"实际 {summary['task'].get('state')}、{describe(summary)}、命令{'跑了' if os.path.exists(marker) else '没跑'}")
    world.stop_server()
    world.resources(local=(30, 30), hold=True)
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"claimed:{pause}"})
    release = os.path.join(world.root, "race-release")
    task_id = world.submit(blocking_command(release))
    open(os.path.join(pause, "arm-claimed"), "w").close()
    world.resources(local=(30, 30))
    try:
        wait_path(world, os.path.join(pause, "reached-claimed"), "停在记了尝试之后、起单元之前")
        canceller = world.client(["cancel", str(task_id)], background=True)
        time.sleep(1.0)
        if canceller.poll() is not None:
            problems.append(f"记了尝试、单元还没起时取消：取消应当等单元起完再停（这时不回），实际已退 {canceller.returncode}")
        open(os.path.join(pause, "release-claimed"), "w").close()
        stdout, stderr = world.communicate(canceller, STATE_TIMEOUT_SECONDS)
        summary = world.summary(task_id)
        units = [attempt.get("unit", "") for attempt in summary["attempts"]]
        still = [unit for unit in units if world.unit_active(unit)]
        if canceller.returncode != 0 or still or not units:
            problems.append(f"记了尝试、单元还没起时取消：应当等单元起了、停了、回读过才退 0，实际退 {canceller.returncode}、单元 {units}、还在跑的 {still}（{stdout.strip()[-200:]}）")
        finished = world.finished(task_id)
        if finished["task"].get("outcome") != "cancelled":
            problems.append(f"那一件的结局应当是 cancelled，实际 {describe(finished)}")
    finally:
        open(release, "w").close()
    if problems:
        raise SelftestFailure("起单元与取消交错：" + "；".join(problems), world.server_log_lines())
    return "判过 queued 之后到的取消：回了 stopped 就一次都不起；记了尝试之后到的取消：等单元起完、停了、回读过才回，结局 cancelled"


SHARD_CRASH_POINTS = ("parent-inserted", "child-finished-plan", "shard-inserted", "child-finished-shard", "merge-outputs-written", "merge-inserted",
                      "child-finished-merge")


def cell_sharded_progress_recovered(world):
    """TS-02：分片推进的每一个断点（父任务插入后、计划这一步终态提交后、建了一片之后、一片终态提交后、写了 outputs.tsv 之后、建了合并之后、合并终态提交后）
    服务当场退出、重启之后都推到整件 succeeded，每一步（计划、各片、合并）只有一个子任务。"""
    problems = []
    for point in SHARD_CRASH_POINTS:
        world.stop_server()
        arm = os.path.join(world.root, f"arm-{point}")
        command = world.write_spec(f"spec-recover-{point}.json", {"shards": [{}, {}]})
        world.start_server(extra={"TASK_SERVER_SELFTEST_CRASH_POINT": f"{point}:{arm}"})
        open(arm, "w").close()
        before = {record["id"] for record, _ in world.signed_rows("tasks")}
        submitted = world.client(["submit", "--cwd", world.repo, "--", *command])
        world.confirm_submitted(submitted, required=False)   # 确认之后才进队、建计划那一步（parent-inserted 那一点就在确认里，服务在那一刻退出）
        code = world.wait_server_exit()
        if code != 70:
            problems.append(f"{point}：自证造的崩溃应当让服务退 70，实际退 {code}")
            continue
        world.start_server()
        parents = [record["id"] for record, _ in world.signed_rows("tasks") if record["id"] not in before and record.get("sharded_entry")]
        if len(parents) != 1:
            problems.append(f"{point}：库里应当有这一件的父任务一条，实际 {parents}（提交退 {submitted.returncode}）")
            continue
        summary = world.finished(parents[0])
        roles = [(child["task"].get("role"), child["task"].get("shard_index", 0)) for child in summary.get("children", [])]
        if summary["task"].get("outcome") != "succeeded" or sorted(roles) != [("merge", 0), ("plan", 0), ("shard", 0), ("shard", 1)]:
            problems.append(f"{point}：重启之后应当推到整件 succeeded、计划一个、两片各一个、合并一个，实际 {describe(summary)}、子任务 {sorted(roles)}")
    if problems:
        raise SelftestFailure("分片推进断在半途：" + "；".join(problems), world.server_log_lines())
    return f"{len(SHARD_CRASH_POINTS)} 个断点各崩一次，重启之后都推到整件 succeeded、每一步只一个子任务"


def cell_attempt_claim_atomic(world):
    """TS-03：在插了尝试、改任务之前（一个事务里）服务当场退出：重启之后库里没有那一次尝试，照常起一次、命令只跑一次；同一任务同一次号再插一行被库拒。"""
    problems = []
    world.stop_server()
    arm = os.path.join(world.root, "arm-crash-inside-claim")
    counter = os.path.join(world.root, "count-crash-inside-claim")
    release = os.path.join(world.root, "release-crash-inside-claim")
    world.resources(local=(30, 30), hold=True)
    world.start_server(extra={"TASK_SERVER_SELFTEST_CRASH_POINT": f"attempt-claimed:{arm}"})
    task_id = world.submit(blocking_command(release, counter))
    open(arm, "w").close()
    world.resources(local=(30, 30))
    code = world.wait_server_exit()
    if code != 70:
        raise SelftestFailure(f"自证造的崩溃应当让服务退 70，实际退 {code}", world.server_log_lines())
    try:
        world.start_server()
        summary = world.running(task_id)
        numbers = [attempt.get("number") for attempt in summary["attempts"]]
        if numbers != [1] or summary["task"].get("attempt_count") != 1:
            problems.append(f"重启之后任务应当 running、只有第 1 次尝试、尝试次数记 1，实际尝试号 {numbers}、尝试次数 {summary['task'].get('attempt_count')}")
    finally:
        open(release, "w").close()
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded" or read_text(counter) != "x" or len(summary["attempts"]) != 1:
        problems.append(f"应当只起一次、命令只跑一次、succeeded，实际命令跑了 {len(read_text(counter))} 次、{describe(summary)}")
    try:
        world.insert_signed_row("attempts", {"task_id": task_id, "number": 1, "state": "ended", "session": "", "command_key": "selftest-duplicate"})
        problems.append(f"同一任务 {task_id} 同一次号 1 再插一行尝试应当被库拒（唯一约束），实际插进去了")
    except sqlite3.IntegrityError:
        pass
    if problems:
        raise SelftestFailure("插尝试与改任务：" + "；".join(problems), world.server_log_lines())
    return "插尝试与改任务一个事务：事务里崩了重启照常起一次；同一次号再插被库拒"


def cell_sharded_cancel_retried(world):
    """TS-04：两片都在跑时取消整件，第一次 systemctl stop 没停成（TASK_SERVER_SELFTEST_STOP_FAILS 演一次）：取消回没停、整件不记结局（留在 cancelling）；
    第二次取消照样重查、重停活动的子任务，都停了、读回过才回 stopped，整件记 cancelled。"""
    release = os.path.join(world.root, "release-cancel-retried")
    stop_fails = os.path.join(world.root, "stop-fails-once")
    command = world.write_spec("spec-cancel-retried.json", {"shards": [{"block": release}, {"block": release}]})
    world.resources(local=(30, 30), peer=(30, 30))   # 两片要同时在跑：一台同一时刻只跑同一件的一片，给两台
    world.start_server(extra={"TASK_SERVER_SELFTEST_STOP_FAILS": stop_fails})
    task_id = world.submit(command, cwd=world.repo)
    problems = []
    try:
        summary = world.wait_for(task_id, lambda summary: len(shard_children(summary)) == 2 and all(
            any(attempt.get("state") == "running" for attempt in child["attempts"]) for child in shard_children(summary).values()), "两片都跑起来")
        units = [child["attempts"][-1].get("unit", "") for child in shard_children(summary).values()]
        open(stop_fails, "w").close()
        first = world.client(["cancel", str(task_id)])
        after_first = world.summary(task_id)
        if first.returncode == 0 or after_first["task"].get("state") == "finished" or not any(world.unit_active(unit) for unit in units):
            problems.append(f"第一次停没停成：取消应当不退 0、整件不记结局、还有子任务的单元在跑，实际退 {first.returncode}、"
                            f"整件 {after_first['task'].get('state')}/{after_first['task'].get('outcome')}、在跑的单元 {[unit for unit in units if world.unit_active(unit)]}")
        second = world.client(["cancel", str(task_id)])
        still = [unit for unit in units if world.unit_active(unit)]
        after_second = world.summary(task_id)
        if second.returncode != 0 or still or after_second["task"].get("outcome") != "cancelled":
            problems.append(f"第二次取消应当重停活动的子任务、都停了才退 0、整件记 cancelled，实际退 {second.returncode}、还在跑的单元 {still}、"
                            f"整件 {after_second['task'].get('state')}/{after_second['task'].get('outcome')}（{second.stdout.strip()[-200:]}）")
    finally:
        open(release, "w").close()
    if problems:
        raise SelftestFailure("分片取消重试：" + "；".join(problems), world.server_log_lines())
    return "第一次停没停成：取消不回 stopped、整件留在 cancelling；第二次取消重停活动的子任务，回读停了才回、整件记 cancelled"


def cell_association_column_verified(world):
    """TS-05：只改库里一行尝试的 task_id 列（正文与签名不动），把任务 A 的尝试挂到任务 B 名下：B 的记录读回时那一行校验不过，取消 B 回不了 stopped、
    A 与 B 的单元都不停；只改一个子任务的 task_id 列把它挂到另一件分片名下：取消那一件同样不停任何单元。"""
    releases = [os.path.join(world.root, f"release-association-{index}") for index in range(4)]
    world.start_server()
    problems = []
    try:
        owner = world.submit(blocking_command(releases[0]))
        other = world.submit(blocking_command(releases[1]))
        owner_summary, other_summary = world.running(owner), world.running(other)
        owner_unit, other_unit = owner_summary["attempts"][0].get("unit", ""), other_summary["attempts"][0].get("unit", "")
        raw_column_update(world, "attempts", 0, other_summary["attempts"][0]["id"])   # (task_id, number) 唯一索引只管 task_id > 0：先把任务 B 自己的 1 号尝试挪到 0 名下，腾出那一号
        raw_column_update(world, "attempts", other, owner_summary["attempts"][0]["id"])
        cancelled = world.client(["cancel", str(other)])
        listed = world.summary(other)["attempts"]
        if cancelled.returncode == 0 or not world.unit_active(owner_unit) or not world.unit_active(other_unit):
            problems.append(f"尝试的 task_id 列改到任务 {other} 名下：取消 {other} 应当不退 0、一个单元都不停，实际退 {cancelled.returncode}、"
                            f"{owner} 的单元{'在跑' if world.unit_active(owner_unit) else '停了'}、{other} 的单元{'在跑' if world.unit_active(other_unit) else '停了'}")
        if not any(attempt.get("id") == owner_summary["attempts"][0]["id"] and attempt.get("valid") is False for attempt in listed):
            problems.append(f"任务 {other} 名下读回的那一行（正文是任务 {owner} 的）应当校验不过，实际 {[(attempt.get('id'), attempt.get('valid')) for attempt in listed]}")
        first = world.submit(world.write_spec("spec-association-first.json", {"shards": [{"block": releases[2]}]}), cwd=world.repo)
        second = world.submit(world.write_spec("spec-association-second.json", {"shards": [{"block": releases[3]}]}), cwd=world.repo)
        running = lambda summary: len(shard_children(summary)) == 1 and any(attempt.get("state") == "running" for child in shard_children(summary).values()
                                                                             for attempt in child["attempts"])
        first_child = list(shard_children(world.wait_for(first, running, "那一片跑起来")).values())[0]
        second_child = list(shard_children(world.wait_for(second, running, "那一片跑起来")).values())[0]
        raw_column_update(world, "tasks", second, first_child["task"]["id"])
        cancelled = world.client(["cancel", str(second)])
        units = [first_child["attempts"][-1].get("unit", ""), second_child["attempts"][-1].get("unit", "")]
        if cancelled.returncode == 0 or not all(world.unit_active(unit) for unit in units):
            problems.append(f"子任务 {first_child['task']['id']} 的 task_id 列改到分片的任务 {second} 名下：取消 {second} 应当不退 0、两片的单元都不停，"
                            f"实际退 {cancelled.returncode}、在跑的单元 {[unit for unit in units if world.unit_active(unit)]}")
    finally:
        for release in releases:
            open(release, "w").close()
    if problems:
        raise SelftestFailure("关联列：" + "；".join(problems), world.server_log_lines())
    return "只改尝试或子任务的 task_id 列：挂过去的那一行读回校验不过，取消那一件不停任何单元"


def cell_start_timeout_read_back(world):
    """TS-06：systemd-run 起了单元却不回执（假 systemd-run 交给真的起完就一直睡，起单元的时限缩到 1 秒）：任务照单元名读回、照常 running、账照记，
    取消照常停它；systemd-run 什么都没起也不回执：尝试留在 starting、写告警 start-uncertain，过了宽限读回单元一直不在才记 unit-start-failed、命令没跑。"""
    problems = []
    release = os.path.join(world.root, "release-timeout")
    world.start_server(extra={"TASK_SERVER_SYSTEMD_RUN": os.path.join(world.bin, "fake-systemd-run-delayed"), "TASK_SERVER_SELFTEST_START_TIMEOUT_SECONDS": "1"})
    try:
        task_id = world.submit(blocking_command(release))
        summary = world.wait_for(task_id, lambda summary: summary["task"].get("state") == "finished" or
                                 any(attempt.get("state") == "running" and world.unit_active(attempt.get("unit", "")) for attempt in summary["attempts"]),
                                 "照单元名读回成在跑")
        if summary["task"].get("state") != "running":
            problems.append(f"回执超时而单元起了：任务应当照读回的单元记 running，实际 {describe(summary)}")
        else:
            cancelled = world.client(["cancel", str(task_id)])
            finished = world.finished(task_id)
            if cancelled.returncode != 0 or finished["task"].get("outcome") != "cancelled" or world.unit_active(summary["attempts"][0].get("unit", "")):
                problems.append(f"回执超时而单元起了：取消应当照常停它、记 cancelled，实际退 {cancelled.returncode}、{describe(finished)}")
    finally:
        open(release, "w").close()
    world.stop_server()
    world.start_server(extra={"TASK_SERVER_SYSTEMD_RUN": os.path.join(world.bin, "fake-systemd-run-silent"), "TASK_SERVER_SELFTEST_START_TIMEOUT_SECONDS": "1"})
    marker = os.path.join(world.root, "timeout-ran")
    task_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", marker])
    summary = world.finished(task_id)
    if (summary["task"].get("outcome") != "unit-start-failed" or os.path.exists(marker)
            or not any(alert.get("kind") == "start-uncertain" and alert.get("task") == task_id for alert in world.alerts())):
        problems.append(f"回执超时而什么都没起：应当先写告警 start-uncertain、读回确认没起才记 unit-start-failed、命令没跑，实际 {describe(summary)}、"
                        f"命令{'跑了' if os.path.exists(marker) else '没跑'}、告警 {[alert.get('kind') for alert in world.alerts()]}")
    if problems:
        raise SelftestFailure("起单元回执超时：" + "；".join(problems), world.server_log_lines())
    return "回执超时而单元起了：照单元名读回 running、取消照常停；什么都没起：告警 start-uncertain，读回确认没起才记 unit-start-failed"


def cell_build_disk_predicted(world):
    """问题 76：编译目录占盘按命令记峰值（本机 attempt_runner 量服务给的编译目录），派活时那一台盘余量放得下「预测 + 下限」才派，放不下排着、
    排队原因点名占盘；本机与第二台同样判。"""
    problems = []
    world.resources(local=(30, 30))
    world.start_server()
    path = f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"
    built = world.finished(world.submit(["cargo", "build"], extra={"PATH": path}))
    peak = (built["attempts"] or [{}])[0].get("build_peak")
    if not isinstance(peak, int) or peak < 65536:
        problems.append(f"会编的那一次应当量下编译目录占盘峰值（假 cargo 写了 64K），实际 {peak}、{describe(built)}")
    floor = SERVER.BUILD_DISK_FLOOR_BYTES
    world.seed_attempt({"command_key": "selftest-disk-local", "outcome": "succeeded", "memory_peak": GIBIBYTE, "build_peak": 30 * GIBIBYTE})
    world.seed_attempt({"command_key": "selftest-disk-peer", "outcome": "succeeded", "memory_peak": 10 * GIBIBYTE, "build_peak": 30 * GIBIBYTE})
    for host, key in (("local", "selftest-disk-local"), ("peer", "selftest-disk-peer")):
        small = floor + 10 * GIBIBYTE
        if host == "local":
            world.resources(local=(30, 30), local_disk_bytes=small)
        else:
            world.resources(local=(8, 30), local_disk_bytes=10 * floor, peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=small)
        task_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
        world.next_pass()
        world.next_pass()
        held = world.summary(task_id)
        if held["task"].get("state") != "queued" or "编译目录占盘" not in (held.get("waiting_for") or ""):
            problems.append(f"{lib.HOST_NAMES[host]}盘余 {small // GIBIBYTE}G、这条命令量过 30G（加下限 {floor // GIBIBYTE}G 放不下）：应当排着、排队原因点名占盘，"
                            f"实际 {held['task'].get('state')}、在等「{held.get('waiting_for')}」、{describe(held)}")
            continue
        if host == "local":
            world.resources(local=(30, 30), local_disk_bytes=10 * floor)
        else:
            world.resources(local=(8, 30), local_disk_bytes=10 * floor, peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=10 * floor)
        finished = world.finished(task_id)
        attempt = (finished["attempts"] or [{}])[0]
        if finished["task"].get("outcome") != "succeeded" or attempt.get("host") != host or attempt.get("disk_booked") != 30 * GIBIBYTE:
            problems.append(f"{lib.HOST_NAMES[host]}盘放得下之后应当在那一台起、占盘记 30G，实际 {describe(finished)}、在 {attempt.get('host')}、记 {attempt.get('disk_booked')}")
    if problems:
        raise SelftestFailure("编译目录占盘：" + "；".join(problems), world.server_log_lines())
    return f"量下编译目录峰值 {peak} 字节；本机与第二台盘余量放不下「30G + 下限」时排着、原因点名占盘，放得下才起、占盘记 30G"


def cell_peer_command_exit_split(world):
    """问题 79：派到第二台的命令自己退 255（multi-host-run.sh 照真的那样打了第二台那条命令的结束行 exit=255）记 command-failed、退 255、不重排；
    命令自己退 3 而 multi-host-run.sh 收尾退了 255 的照命令的 3 记 command-failed；没有那一行的 255 照旧是 peer-run-failed。"""
    world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-peer-exit", 40 * GIBIBYTE)   # 40G 的只有第二台放得下
    world.start_server()
    key = {lib.KEY_VARIABLE: "selftest-peer-exit"}
    cases = [("命令自己退 255", ["fake-peer-step-exit=255"], "command-failed", 255, 1),
             ("命令自己退 3、multi-host-run.sh 收尾退 255", ["fake-peer-step-exit=3", "fake-peer-exit=255"], "command-failed", 3, 1),
             ("multi-host-run.sh 自己退 255（没有结束行）", ["fake-peer-exit=255"], "peer-run-failed", 255, 1 + lib.PEER_RUN_RETRIES)]
    problems = []
    for name, words, outcome, code, attempts in cases:
        summary = world.finished(world.submit(["bash", "-c", "exit 0", *words], extra=key, cwd=world.repo))
        hosts = {attempt.get("host") for attempt in summary["attempts"]}
        if (summary["task"].get("outcome") != outcome or summary["task"].get("exit_code") != code or len(summary["attempts"]) != attempts or hosts != {"peer"}):
            problems.append(f"{name}：应当在第二台、记 {outcome}、退 {code}、尝试 {attempts} 次，实际 {describe(summary)}、在 {hosts}")
    if problems:
        raise SelftestFailure("第二台的退出码分层：" + "；".join(problems), world.server_log_lines())
    return "第二台命令自己退 255 记 command-failed 不重排、收尾的 255 不盖掉命令的码；没有结束行的 255 照旧 peer-run-failed 重排"


def cell_nfs_automount_recognized(world):
    """问题 84：「挂载」那一项触发自动挂载、看文件系统类型：第二台读回 nfs 过；读回 autofs（x-systemd.automount 的占位，NFS 没挂上，mountpoint -q 照样退 0）
    与 stat 读不到（自动挂载触发不上）都不过，出路写在第二台上 reset-failed、ls 触发再挂。"""
    os.environ["TASK_SERVER_SELFTEST_WORLD"] = world.world   # 假 ssh 往这一格的世界目录记调用、读演第二台的几个文件
    environment = world.server_environment
    type_file, exit_file = os.path.join(world.world, "fake-ssh-mount-type"), os.path.join(world.world, "fake-ssh-mount-exit")
    problems = []
    for name, kind, code, passes in (("NFS 挂着", "nfs", None, True), ("只剩自动挂载的占位", "autofs", None, False), ("自动挂载触发不上", None, "1", False)):
        for path, value in ((type_file, kind), (exit_file, code)):
            with contextlib.suppress(FileNotFoundError):
                os.remove(path)
            if value is not None:
                with open(path, "w", encoding="utf-8") as handle:
                    handle.write(value + "\n")
        verdict = lib.nfs_mount_verdict(world.shared_root, "selftest-peer", environment)
        if (verdict == lib.NFS_PASSED) != passes or (not passes and "reset-failed" not in verdict):
            problems.append(f"{name}：「挂载」那一项应当{'过' if passes else '不过、出路写 reset-failed 与触发再挂'}，实际「{verdict}」")
    if problems:
        raise SelftestFailure("NFS 挂载的判法：" + "；".join(problems))
    return "第二台读回 nfs 过；只剩自动挂载的占位、触发不上都不过，出路写在第二台 reset-failed、ls 触发再挂"


def cell_peer_memory_peak_recorded(world):
    """问题 86：第二台那条命令结束时 multi-host-run.sh 打 MULTI_HOST_RUN_PEER_MEMORY_PEAK bytes=<字节>：那次尝试记下这个峰值，同一命令下一次照它定上限
    （3G × 1.5 = 4608M）；不打这一行的照旧记「没量到」（null）。"""
    world.resources(local=(2, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下 4G，只有第二台放得下
    world.start_server()
    key = {lib.KEY_VARIABLE: "selftest-peer-memory"}
    measured = world.finished(world.submit(["bash", "-c", "exit 0", f"fake-peer-memory-peak={3 * GIBIBYTE}"], extra=key, cwd=world.repo))
    silent = world.finished(world.submit(["bash", "-c", "exit 0", "selftest-peer-silent"], extra={lib.KEY_VARIABLE: "selftest-peer-silent"}, cwd=world.repo))
    again = world.finished(world.submit(["bash", "-c", "exit 0"], extra=key, cwd=world.repo))
    first, quiet, next_one = (summary["attempts"][0] if summary["attempts"] else {} for summary in (measured, silent, again))
    problems = []
    if first.get("host") != "peer" or first.get("memory_peak") != 3 * GIBIBYTE:
        problems.append(f"打了那一行的尝试应当在第二台、记峰值 {3 * GIBIBYTE}，实际在 {first.get('host')}、记 {first.get('memory_peak')}")
    if quiet.get("host") != "peer" or quiet.get("memory_peak", "缺这一栏") is not None:
        problems.append(f"没打那一行的应当记「没量到」（null），实际在 {quiet.get('host')}、记 {quiet.get('memory_peak', '缺这一栏')}")
    if next_one.get("memory_cap_text") != "4608M":
        problems.append(f"同一命令下一次应当照第二台量到的 3G × 1.5 定上限 4608M，实际 {next_one.get('memory_cap_text')}")
    if problems:
        raise SelftestFailure("第二台的内存峰值：" + "；".join(problems), world.server_log_lines())
    return "第二台打了峰值那一行的记下 3G、下一次上限 4608M；不打的记 null"


def cell_prune_server_unavailable_outcome(world):
    """命令自己退 75（prune-server 不可用）：结局单列 prune-server-unavailable、wait 退 75、末行 type 带它，不重排；服务起一次 prune-server/control check
    （假的，照 prune-check-exit 退）：不通写一条告警 prune-server-unavailable，通了不写。"""
    world.start_server(extra={"TASK_SERVER_PRUNE_SERVER_CHECK": os.path.join(world.bin, "fake-prune-check")})
    problems = []
    for name, check_exit, alerted in (("check 不通", "78", True), ("check 通", "0", False)):
        with open(os.path.join(world.world, "prune-check-exit"), "w", encoding="utf-8") as handle:
            handle.write(check_exit + "\n")
        checks = len([line for line in world.calls() if line.startswith("prune-check ")])
        task_id = world.submit(["bash", "-c", "exit 75"])
        summary = world.finished(task_id)
        wait_code, last = wait_last_line(world, task_id)
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while patience.waiting() and len([line for line in world.calls() if line.startswith("prune-check ")]) <= checks:
            time.sleep(POLL_SECONDS)
        time.sleep(0.5)   # check 退了之后写告警那一下
        written = [alert for alert in world.alerts() if alert.get("kind") == lib.PRUNE_SERVER_ALERT_KIND and alert.get("task") == task_id]
        if (summary["task"].get("outcome") != lib.PRUNE_SERVER_UNAVAILABLE_OUTCOME or summary["task"].get("exit_code") != 75 or len(summary["attempts"]) != 1
                or wait_code != 75 or f"type={lib.PRUNE_SERVER_UNAVAILABLE_OUTCOME} " not in last + " "):
            problems.append(f"{name}：命令退 75 应当记 {lib.PRUNE_SERVER_UNAVAILABLE_OUTCOME}、退 75、不重排、wait 退 75、末行 type 带它，实际 {describe(summary)}、"
                            f"wait 退 {wait_code}、末行「{last}」")
        if bool(written) != alerted:
            problems.append(f"{name}：应当{'写' if alerted else '不写'}一条 {lib.PRUNE_SERVER_ALERT_KIND} 告警，实际 {len(written)} 条（check 起了 "
                            f"{len([line for line in world.calls() if line.startswith('prune-check ')]) - checks} 次）")
    if problems:
        raise SelftestFailure("退 75：" + "；".join(problems), world.server_log_lines())
    return f"命令退 75 记 {lib.PRUNE_SERVER_UNAVAILABLE_OUTCOME}、wait 退 75；check 不通写告警、通了不写"


def cell_temporary_service_wrapper_unwrapped(world):
    """临时 prune-server 的包装（python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service --server-binary <路径> -- <原命令>）：
    包着的照原命令算键、--server-binary 的路径不进键（两次包着的给的路径不同，命令键相同、第二次照第一次量到的 300M 定上限 450M）；
    键前面接 TEMPORARY_SERVICE_KEY_MARK，与不包的那一条不共用记录（不包的那次也量到 300M，包着的第一次没有自己的记录、取 4096M）；
    包装本身照旧整条执行（认登记表行同样照原命令认，由 verdict-library-family-refused 判）。"""
    scope = world.write_fake_scope(300 * MEBIBYTE)
    world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    inner = ["bash", "-c", "exit 0", "selftest-wrapped-marker", "fake-peer-run"]   # fake-peer-run：派到假第二台的也照真的那样在这棵树里起命令

    def wrapped(binary):
        return ["python3", f"scripts/prune-server/{lib.TEMPORARY_SERVICE_SCRIPT}", lib.TEMPORARY_SERVICE_FLAG, "--server-binary", binary, "--", *inner]

    plain = world.finished(world.submit(inner, cwd=world.repo))
    first = world.finished(world.submit(wrapped("/usr/bin/true"), cwd=world.repo))
    second = world.finished(world.submit(wrapped("/bin/true"), cwd=world.repo))
    keys = [summary["task"].get("command_key") or "" for summary in (plain, first, second)]
    caps = [first_attempt(summary).get("memory_cap_text") for summary in (plain, first, second)]
    problems = []
    if keys[1] != keys[2] or keys[1] == keys[0] or not keys[1].startswith(lib.TEMPORARY_SERVICE_KEY_MARK):
        problems.append(f"两次包着的应当同一个命令键、以「{lib.TEMPORARY_SERVICE_KEY_MARK}」起头、与不包的不同，实际不包的「{keys[0]}」、包着的「{keys[1]}」与「{keys[2]}」")
    if caps[1:] != [UNRECORDED_DEFAULT_PINNED_TEXT, "450M"] or any(summary["task"].get("outcome") != "succeeded" for summary in (plain, first, second)):
        problems.append(f"包着的第一次没有自己的记录应当取 {UNRECORDED_DEFAULT_PINNED_TEXT}（不用不包的那次量到的 300M）、第二次照第一次的 300M 取 450M、都跑完，"
                        f"实际上限 {caps}、{describe(first)}、{describe(second)}")
    if not all(world.calls_of(summary["task"]["id"], "temporary-service") for summary in (first, second)):
        problems.append("包装本身应当照旧整条执行（假包装记一行），实际没起")
    if problems:
        raise SelftestFailure("临时 prune-server 的包装：" + "；".join(problems), world.server_log_lines())
    return (f"包着的照原命令算键、服务路径不进键、前面接「{lib.TEMPORARY_SERVICE_KEY_MARK}」：与不包的不共用记录（第一次 {UNRECORDED_DEFAULT_PINNED_TEXT}、"
            "第二次照第一次的 300M 取 450M），包装照旧整条执行")


WRAPPED_SELFTEST_SCRIPT = "scripts/selftest-wrapped-check.sh"   # temporary-service-heavy-refused：包装里照收的那条普通 --selftest 起的假脚本（相对假仓根）


def cell_temporary_service_heavy_refused(world):
    """④ 临时 prune-server 包装里会编或要卡的提交时拒（task_server.py 文件头 ②「提交时拒」④）：包装里套 cargo test（任务 4229 那一条的样子）与
    带 gpu 特性的 python 入口（E174 那一条）都当场拒（task submit 退 3、不给任务号，说明点名会编或要卡、写明包装只给脚本自证用，出路写去掉包装重交），
    一条都没进队；包装里普通的 --selftest 照收、包装照旧整条执行、跑完。"""
    world.resources(local=(30, 30))
    world.start_server()
    write_executable(os.path.join(world.repo, WRAPPED_SELFTEST_SCRIPT), "#!/usr/bin/env bash\nexit 0\n")

    def wrapped(*inner):
        return ["python3", f"scripts/prune-server/{lib.TEMPORARY_SERVICE_SCRIPT}", lib.TEMPORARY_SERVICE_FLAG, "--server-binary", "/usr/bin/true", "--", *inner]

    problems = []
    refused_cases = (("包装里套 cargo test", wrapped("cargo", "test", "--release", "-p", "singlefs-verification", "--features", "gpu", "--test", "selftest-target",
                                                 "fake-wrapper-exits=0"), "会编的 cargo"),
                     ("包装里套带 gpu 特性的 python 入口", wrapped(*E174_COMMAND, "fake-wrapper-exits=0"), "要卡"))
    for name, command, wanted in refused_cases:
        completed = world.client(["submit", "--cwd", world.repo, "--", *command])
        texts = (wanted, "只给脚本自证用", "去掉包装重交")
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or not all(text in completed.stderr for text in texts):
            problems.append(f"{name}：应当退 3、不给任务号、说明写「{'」「'.join(texts)}」，实际退 {completed.returncode}、"
                            f"stdout「{completed.stdout.strip()[-200:]}」、stderr「{completed.stderr.strip()[-400:]}」")
    accepted = []
    try:
        task_id = world.submit(wrapped("bash", WRAPPED_SELFTEST_SCRIPT, "--selftest"), cwd=world.repo)
        accepted.append(task_id)
        summary = world.finished(task_id)
        if summary["task"].get("outcome") != "succeeded" or not world.calls_of(task_id, "temporary-service"):
            problems.append(f"包装里普通的 --selftest 应当照收、包装照旧整条执行（假包装记一行）、跑完，实际 {describe(summary)}、"
                            f"假包装记了 {len(world.calls_of(task_id, 'temporary-service'))} 行")
    except SelftestFailure as failure:
        problems.append(f"包装里普通的 --selftest 应当照收、跑完，实际 {failure}")
    listed = {item.get("id") for item in json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", [])}
    if listed - set(accepted):
        problems.append(f"拒了的一条都不该进队，实际会话里另列着 {sorted(listed - set(accepted))}")
    if problems:
        raise SelftestFailure("临时 prune-server 包装里会编或要卡的提交时拒：" + "；".join(problems), world.server_log_lines())
    return (f"包装里套 cargo test 与带 gpu 特性的 python 入口提交时退 3、说明点名会编或要卡、出路写去掉包装重交；"
            f"包装里普通的 --selftest 照收、包装照旧整条执行、跑完（{len(refused_cases)} 条拒、{len(accepted)} 条收）")


WHOLE_WORKSPACE_MEMBER = "crates/selftest-member"   # whole-workspace-cargo-refused：假仓根虚拟清单里的成员（相对假仓根），自己有一份包清单
WHOLE_WORKSPACE_NEXT_TEXTS = ("-p <包>", "-p singlefs-verification", "merge-and-verify")   # 那一格拒绝说明的出路里要有的几样（task_server.py 的 WHOLE_WORKSPACE_NEXT_STEP）


def cell_whole_workspace_cargo_refused(world):
    """⑤ 不点名包的全仓编译提交时拒（task_server.py 文件头 ②「提交时拒」⑤；任务 8105）：假仓根写一份虚拟清单（只有 [workspace]、两个成员），在仓根交
    不带 -p 的 cargo build --offline --all-targets（8105 那一条）、不带 -p 的 cargo clippy --all-targets --all-features（8104 那一条）、带 --workspace 的 cargo clippy、
    带 -p 又带 --workspace 的 cargo test、bash -c 里不带 -p 的 cargo check，都当场拒（task submit 退 3、不给任务号，说明点名那条 cargo，出路写 -p <包>、
    -p singlefs-verification 与 merge-and-verify），一条都没进队；带 -p 的 cargo build、在成员目录里不带 -p 的 cargo test、不编的 cargo tree 照收、跑完。
    弄坏开关 whole-workspace-cargo-accepted 下拒的那几条照收，这一格红。"""
    world.resources(local=(30, 30))
    member = os.path.join(world.repo, WHOLE_WORKSPACE_MEMBER)
    os.makedirs(member, exist_ok=True)
    with open(os.path.join(world.repo, "Cargo.toml"), "w", encoding="utf-8") as handle:
        handle.write(f'[workspace]\nresolver = "2"\nmembers = ["{WHOLE_WORKSPACE_MEMBER}", "crates/selftest-other"]\n')
    with open(os.path.join(member, "Cargo.toml"), "w", encoding="utf-8") as handle:
        handle.write('[package]\nname = "selftest-member"\nversion = "0.0.0"\n')
    world.start_server()
    problems = []
    refused_cases = (("不带 -p 的 cargo build（任务 8105）", ["cargo", "build", "--offline", "--all-targets"], "cargo build 没点名包"),
                     ("不带 -p 的 cargo clippy（任务 8104）", ["cargo", "clippy", "--all-targets", "--all-features"], "cargo clippy 没点名包"),
                     ("带 --workspace 的 cargo clippy", ["cargo", "clippy", "--workspace", "--all-targets"], "cargo clippy 带了 --workspace"),
                     ("带 -p 又带 --workspace 的 cargo test", ["cargo", "test", "-p", "selftest-member", "--workspace"], "cargo test 带了 --workspace"),
                     ("bash -c 里不带 -p 的 cargo check", ["bash", "-c", "cargo check --release"], "cargo check 没点名包"))
    for name, command, wanted in refused_cases:
        completed = world.client(["submit", "--cwd", world.repo, "--", *command])
        texts = (wanted, *WHOLE_WORKSPACE_NEXT_TEXTS)
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or not all(text in completed.stderr for text in texts):
            problems.append(f"{name}：应当退 3、不给任务号、说明写「{'」「'.join(texts)}」，实际退 {completed.returncode}、"
                            f"stdout「{completed.stdout.strip()[-200:]}」、stderr「{completed.stderr.strip()[-400:]}」")
    accepted = []
    accepted_cases = (("带 -p 的 cargo build", world.repo, ["cargo", "build", "-p", "selftest-member", "fake-wrapper-exits=0"]),
                      ("在成员目录里不带 -p 的 cargo test", member, ["cargo", "test", "fake-wrapper-exits=0"]),
                      ("不编的 cargo tree", world.repo, ["cargo", "tree", "fake-wrapper-exits=0"]))
    for name, cwd, command in accepted_cases:
        try:
            task_id = world.submit(command, cwd=cwd)
            accepted.append(task_id)
            summary = world.finished(task_id)
            if summary["task"].get("outcome") != "succeeded":
                problems.append(f"{name}应当照收、跑完，实际 {describe(summary)}")
        except SelftestFailure as failure:
            problems.append(f"{name}应当照收、跑完，实际 {failure}")
    listed = {item.get("id") for item in json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", [])}
    if listed - set(accepted):
        problems.append(f"拒了的一条都不该进队，实际会话里另列着 {sorted(listed - set(accepted))}")
    if problems:
        raise SelftestFailure("不点名包的全仓编译提交时拒：" + "；".join(problems), world.server_log_lines())
    return (f"虚拟清单的仓根上不点名包、带 --workspace 的 {len(refused_cases)} 条 cargo 提交时退 3、出路写点名要编的包；"
            f"点了包的、在成员目录里的、不编的 {len(accepted)} 条照收、跑完")


def cell_data_root_key_required(world):
    """多机配置没写 SHARED_DATA_ROOT（task_server_lib.py 文件头「共享数据根」）：服务不起、退 78、点名这个键；control start 起之前就退 78、出路点名往 multi-host.env 加
    SHARED_DATA_ROOT= 那一行、不起 systemctl start；control install --dry-run 同样退 78、点名这个键。"""
    config = os.path.join(world.root, "no-data-root-multi-host.env")
    World.write_config(config, world.state, world.runtime, world.key, world.builds, world.shared_root, data_root=False)
    problems = []
    code, output = world.serve_until_exit(config)
    if code != lib.RUN_CONDITION_EXIT or lib.DATA_ROOT_KEY not in output:
        problems.append(f"服务应当不起、退 78、stderr 点名 {lib.DATA_ROOT_KEY}，实际{'起来了' if code is None else f'退 {code}'}、「{output.strip()[-300:]}」")
    extra = {lib.MULTI_HOST_CONFIG_VARIABLE: config, "TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl"),
             "XDG_CONFIG_HOME": os.path.join(world.root, "xdg-config")}
    started = world.control(["start"], extra=extra)
    if (started.returncode != lib.RUN_CONDITION_EXIT or f"{lib.DATA_ROOT_KEY}=" not in started.stderr or lib.MULTI_HOST_CONFIG_NAME not in started.stderr
            or systemctl_start_count(world)):
        problems.append(f"control start 应当起之前就退 78、出路点名往 {lib.MULTI_HOST_CONFIG_NAME} 加 {lib.DATA_ROOT_KEY}= 那一行、不起 systemctl start，"
                        f"实际退 {started.returncode}、systemctl start 收到 {systemctl_start_count(world)} 回、stderr「{started.stderr.strip()[-300:]}」")
    dry = world.control(["install", "--dry-run"], extra=extra)
    if dry.returncode != lib.RUN_CONDITION_EXIT or lib.DATA_ROOT_KEY not in dry.stderr:
        problems.append(f"control install --dry-run 应当退 78、点名 {lib.DATA_ROOT_KEY}，实际退 {dry.returncode}、stderr「{dry.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("没写共享数据根：" + "；".join(problems), [output[-1500:]])
    return f"没写 {lib.DATA_ROOT_KEY}：服务退 78、control start 起之前退 78、install --dry-run 退 78，都点名这个键"


def cell_outside_shared_path_refused(world):
    """① 共享根外的路径提交时就拒（task_server.py 文件头 ②「提交时拒」①）：整个词、bash -c 词里、工作目录在共享根外的，与「名字=/tmp/…」的值，
    task submit 都退 3、不给任务号，说明点名那个路径与 NFS_SHARED_ROOTS、出路写改放某一段下（/tmp 的那一句另说重启会清），服务一条都没收、没起；
    路径都在共享根下（连同 /dev/null、/usr/ 这类两台相同的系统路径）的照常进队、本机放不下就派到第二台跑完。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：收下的那一条去第二台
    os.makedirs(world.outside_root, exist_ok=True)
    world.start_server()
    outside_word, outside_quoted = os.path.join(world.outside_root, "input.txt"), os.path.join(world.outside_root, "quoted.txt")
    outside_tail = os.path.join(world.outside_root, "tail.txt")
    cases = {"整个词是共享根外的路径": (["bash", "-c", "exit 0", "x", outside_word], world.repo, outside_word, ()),
             "bash -c 词里有共享根外的路径": (["bash", "-c", f"cat {outside_quoted} > /dev/null"], world.repo, outside_quoted, ()),
             "bash -c 词里变量展开的尾巴与写死的共享根外路径同在（点名写死的那个）": (["bash", "-c", f'cp "${{X:?}}/release/a" $(pwd)/b {outside_tail}'], world.repo, outside_tail, ()),
             "工作目录在共享根外": (["bash", "-c", "exit 0", "outside-cwd"], world.outside_root, os.path.realpath(world.outside_root), ()),
             "「名字=/tmp/…」的值": (["bash", "-c", "exit 0", f"OUT={VOLATILE_SAMPLE_PATH}"], world.repo, VOLATILE_SAMPLE_PATH, ("重启会清掉 /tmp",))}
    problems = []
    for name, (command, cwd, named, more) in cases.items():
        completed = world.client(["submit", "--cwd", cwd, "--", *command])
        wanted = (named, lib.NFS_ROOTS_KEY, "改放") + more
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or not all(text in completed.stderr for text in wanted):
            problems.append(f"{name}：应当退 3、不给任务号、说明点名 {named} 与 {lib.NFS_ROOTS_KEY}、出路写改放某一段下{'、另说重启会清 /tmp' if more else ''}，"
                            f"实际退 {completed.returncode}、stdout「{completed.stdout.strip()[-200:]}」、stderr「{completed.stderr.strip()[-400:]}」")
    listed = json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", [])
    if listed or world.calls():
        problems.append(f"拒了的一条都不该进队、不该起，实际会话里列着 {[item.get('id') for item in listed]}、包装调用 {world.calls()[:5]}")
    # 变量展开、命令替换之后的尾巴不是写死的绝对路径：不拒（别的会话报的误拒）；写在 -c 脚本词里（照 shell 词读的只有它），exit 0 在前、这几段不执行
    expansion_tails = 'echo "${X:?}/release/a" $(pwd)/b ${X:-.}/c/d'
    inside = ["bash", "-c", f"exit 0; {expansion_tails}", "x", os.path.join(world.repo, "input.txt"), "/dev/null", "/usr/bin/true", "fake-peer-run"]
    try:
        summary = world.finished(world.submit(inside, cwd=world.repo), timeout=PLACEMENT_WAIT_SECONDS)
        if first_attempt(summary).get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
            problems.append(f"路径都在共享根下的应当照收、本机放不下就派第二台跑完，实际 {describe(summary)}")
    except SelftestFailure as failure:
        problems.append(f"路径都在共享根下的应当照收、派第二台跑完，实际 {failure}")
    if problems:
        raise SelftestFailure("共享根外的路径提交时拒：" + "；".join(problems), world.server_log_lines())
    return ("整个词、bash -c 词里、工作目录在共享根外的与 /tmp 下的提交时退 3、点名那个路径，一条都没收；"
            "都在共享根下的、连同变量展开与命令替换之后的 /… 尾巴照收、派到第二台跑完")


SLASH_IN_ARGUMENT = "fs-meta 一项 （接的是 /）"   # path-check-whole-words：参数值中间夹着一个 /（交 crates-mutation-replay.sh 的 --item 那一种），不是路径


def cell_path_check_whole_words(world):
    """① 共享根外的路径只认整个 shell 词（task_server.py 文件头 ②「提交时拒」①；find_outside_shared_path）：参数值中间夹着 / 的（--item 后面那个词「…（接的是 /）」、
    --item=… 写在一个词里的、bash -c 脚本里引号括起来的同一段）照收、跑完；整个词是共享根外绝对路径的（--item 后面那个词、--output=<路径> 的值）照拒：
    退 3、不给任务号、点名那个路径与 NFS_SHARED_ROOTS。弄坏开关 path-check-substring 下带空白的词都按子串认，照收的那三条被拒，这一格红。"""
    world.resources(local=(30, 30))
    os.makedirs(world.outside_root, exist_ok=True)
    world.start_server()
    outside = os.path.join(world.outside_root, "whole-word.txt")
    accepted = {"--item 后面那个词中间夹着 /": ["bash", "-c", "exit 0", "slash-argument", "--item", SLASH_IN_ARGUMENT],
                "--item=… 一个词里夹着 /": ["bash", "-c", "exit 0", "slash-assignment", f"--item={SLASH_IN_ARGUMENT}"],
                "bash -c 脚本里引号括起来的一段夹着 /": ["bash", "-c", f'exit 0; echo "{SLASH_IN_ARGUMENT}"', "slash-quoted"]}
    refused = {"--item 后面整个词是共享根外的路径": ["bash", "-c", "exit 0", "whole-word", "--item", outside],
               "--output=<共享根外的路径> 的值": ["bash", "-c", "exit 0", "option-value", f"--output={outside}"]}
    problems = []
    for name, command in accepted.items():
        completed = world.client(["submit", "--cwd", world.repo, "--", *command])
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if completed.returncode != 0 or not found:
            problems.append(f"{name}：参数里夹着的 / 不是路径，应当照收（退 0、打任务号），实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-300:]}」")
            continue
        world.confirm_submitted(completed)
        summary = world.finished(int(found.group(1)))
        if summary["task"].get("outcome") != "succeeded":
            problems.append(f"{name}：照收之后应当跑完，实际 {describe(summary)}")
    for name, command in refused.items():
        completed = world.client(["submit", "--cwd", world.repo, "--", *command])
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or outside not in completed.stderr or lib.NFS_ROOTS_KEY not in completed.stderr:
            problems.append(f"{name}：应当退 3、不给任务号、说明点名 {outside} 与 {lib.NFS_ROOTS_KEY}，"
                            f"实际退 {completed.returncode}、stdout「{completed.stdout.strip()[-200:]}」、stderr「{completed.stderr.strip()[-400:]}」")
    if problems:
        raise SelftestFailure("共享根外的路径只认整个 shell 词：" + "；".join(problems), world.server_log_lines())
    return f"参数值中间夹着 / 的 {len(accepted)} 条照收、跑完；整个词是共享根外路径的 {len(refused)} 条退 3、点名那个路径"


def cell_shared_roots_outside_refused(world):
    """② 交给命令的共享根不在共享根下提交时就拒（task_server.py 文件头 ②「提交时拒」②）：任务自己带的 SINGLEFS_DATA_ROOT、服务的工作根 SINGLEFS_WORK_ROOT
    在 NFS_SHARED_ROOTS 外的，task submit 退 3、不给任务号，说明点名变量与值、出路写配多机配置，一条都没收；两个都在共享根下的照常进队、本机放不下就派到第二台跑完。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：收下的那一条去第二台
    os.makedirs(world.outside_root, exist_ok=True)
    outside = os.path.realpath(world.outside_root)
    world.start_server()
    problems = []

    def refused(name, extra, named):
        completed = world.client(["submit", "--cwd", world.repo, "--", "bash", "-c", "exit 0", "shared-roots-outside"], extra=extra)
        wanted = (named, lib.NFS_ROOTS_KEY, "配多机配置")
        if completed.returncode != 3 or lib.SUBMIT_MARK in completed.stdout or not all(text in completed.stderr for text in wanted):
            problems.append(f"{name}：应当退 3、不给任务号、说明点名 {named} 与 {lib.NFS_ROOTS_KEY}、出路写配多机配置，"
                            f"实际退 {completed.returncode}、stderr「{completed.stderr.strip()[-400:]}」")

    refused("任务带的数据根在共享根外", {lib.DATA_ROOT_VARIABLE: outside}, f"{lib.DATA_ROOT_VARIABLE}={outside}")
    try:
        summary = world.finished(world.submit(["bash", "-c", "exit 0", "shared-roots-inside"], cwd=world.repo), timeout=PLACEMENT_WAIT_SECONDS)
        if first_attempt(summary).get("host") != "peer" or summary["task"].get("outcome") != "succeeded":
            problems.append(f"两个共享根都在共享根下的应当照收、派第二台跑完，实际 {describe(summary)}")
    except SelftestFailure as failure:
        problems.append(f"两个共享根都在共享根下的应当照收、派第二台跑完，实际 {failure}")
    world.stop_server()
    world.start_server(extra={lib.WORK_ROOT_VARIABLE: outside})
    refused("服务的工作根在共享根外", None, f"{lib.WORK_ROOT_VARIABLE}={outside}")
    peer_runs = [line for line in world.calls() if line.startswith("peer task=")]
    if len(peer_runs) != 1:
        problems.append(f"拒了的两条都不该派出去（第二台只该收到收下的那一条），实际第二台收到 {peer_runs}")
    if problems:
        raise SelftestFailure("共享根不在共享根下提交时拒：" + "；".join(problems), world.server_log_lines())
    return f"任务带的 {lib.DATA_ROOT_VARIABLE}、服务的 {lib.WORK_ROOT_VARIABLE} 在共享根外时提交退 3、点名变量与值；都在共享根下的照收、派到第二台跑完"


def cell_gpu_binary_declaration_recognized(world):
    """编出来的二进制按同名源文件头的 // task-server-gpu-arguments: 认要不要卡（task_server_lib.py 的 declared_gpu_binary）：假仓根（TASK_SERVER_FAKE_REPOSITORY_ROOT）
    crates/ 下写一份 src/bin/selftest_gpu_binary.rs 声明 card-run；<编译目录>/release/selftest_gpu_binary card-run 回显 gpu=1，cpu-run 不要卡，
    没有同名源的二进制不要卡。"""
    world.resources(local=(100, 100), cards=LOCAL_CARD_ZERO, local_gpu=[(0, 16384)])
    world.start_server()
    source = os.path.join(world.repo, "crates", "selftest-gpu-binary", "src", "bin", "selftest_gpu_binary.rs")
    os.makedirs(os.path.dirname(source), exist_ok=True)
    with open(source, "w", encoding="utf-8") as handle:
        handle.write(f"{lib.DECLARED_BINARY_GPU_ARGUMENTS_MARK} card-run\nfn main() {{}}\n")
    release = os.path.join(world.work, "selftest-target", "release")
    cases = (([os.path.join(release, "selftest_gpu_binary"), "card-run"], 1), ([os.path.join(release, "selftest_gpu_binary"), "cpu-run"], 0),
             ([os.path.join(release, "selftest_undeclared_binary"), "card-run"], 0))
    problems, submitted = [], []
    for words, expected in cases:
        task_id, gpu = submitted_gpu(world, [*words, "fake-wrapper-exits=0"])
        submitted.append(task_id)
        if gpu != expected:
            problems.append(f"{os.path.basename(words[0])} {words[1]} 应当回显 gpu={expected}，实际 gpu={gpu}")
    for task_id in submitted:
        world.finished(task_id)
    if problems:
        raise SelftestFailure("编出来的二进制按同名源的声明认要卡：" + "；".join(problems), world.server_log_lines())
    return "同名源声明了 card-run 的二进制带 card-run 回显 gpu=1，带 cpu-run 与没有同名源的二进制回显 gpu=0"


NFS_PROBE_READ_DELAY_SECONDS = 8   # nfs-probe-concurrent-runs：头一趟 control check 的第二台读探针那一步先睡这么久，另一趟在这段时间里整个跑完（一趟 control check 一两秒，推的）


def cell_nfs_probe_concurrent_runs(world):
    """两趟 control check 交错跑两向探针（task_server_lib.py 的 nfs_probe）：头一趟写好本机探针、第二台读它之前先睡（假 ssh 的 fake-ssh-read-delay），
    另一趟在这段时间里整个跑完、收尾删自己的探针；头一趟醒来照样读得到它自己写的那一份，两趟都报 NFS 共享过。"""
    world.start_server()
    delay = os.path.join(world.world, "fake-ssh-read-delay")
    with open(delay, "w", encoding="utf-8") as handle:
        handle.write(f"{NFS_PROBE_READ_DELAY_SECONDS}\n")
    earlier = world.control(["check"], background=True)
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    while not os.path.exists(delay + ".taken") and earlier.poll() is None and patience.waiting():
        time.sleep(POLL_SECONDS)
    later = world.control(["check"])
    try:
        earlier_output, earlier_error = world.communicate(earlier, STATE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        earlier.kill()   # 只停自己起的那一个进程号
        earlier_output, earlier_error = earlier.communicate(timeout=10)
    problems = []
    if not os.path.exists(delay + ".taken"):
        problems.append("头一趟 control check 的第二台没读到探针就收尾了（假 ssh 没走到读探针那一步）")
    for name, code, output in (("头一趟", earlier.returncode, earlier_output + earlier_error), ("另一趟", later.returncode, later.stdout + later.stderr)):
        if code != 0 or "NFS 共享 过" not in output:
            problems.append(f"{name} control check 应当退 0、说 NFS 共享过，实际退 {code}、「{output.strip()[-300:]}」")
    left = sorted(name for name in os.listdir(world.shared_root) if name.startswith((lib.NFS_PROBE_NAME, lib.NFS_PEER_PROBE_NAME)))
    if left:
        problems.append(f"两趟收尾之后探针文件应当都删了，实际还在 {left}")
    if problems:
        raise SelftestFailure("两趟两向探针交错：" + "；".join(problems), world.server_log_lines())
    return "两趟 control check 的两向探针交错：先写的那一趟等另一趟整个跑完收尾之后照样读到自己的探针，两趟都过、探针都删了"


SHORT_EXIT_PEAK_BYTES = 123 * MEBIBYTE   # short-command-peak-recorded：假包装演的退出时峰值
SHORT_SAMPLED_PEAK_BYTES = 200 * MEBIBYTE   # 同一格后半段：按轮采到的假 scope 峰值


def cell_short_command_peak_recorded(world):
    """跑 0.1 秒就退的命令：按轮一次都采不到（假 scope 是空目录），结局的 memory_peak 照内存包装退出时读的那一个记（attempt_runner.py 文件头 ⑥，
    假包装照 fake-wrapper-exit-peak= 往服务给的 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 写），不是 null；两样都没有的照旧 null。
    后半段换上进程内存 200M 的假 scope 重起服务：按轮采到了的照按轮采到的记，退出时读的（含页缓存的 memory.peak）比它大比它小都不用（attempt_runner.py 文件头 ⑥）。
    弄坏开关 memory-peak-with-page-cache 下照改前取大的，「400M 比 200M 大」那一例记 400M，这一格红。"""
    world.start_server()
    short = world.finished(world.submit(["bash", "-c", "sleep 0.1", f"fake-wrapper-exit-peak={SHORT_EXIT_PEAK_BYTES}"], extra={lib.KEY_VARIABLE: "selftest-short-peak"}))
    silent = world.finished(world.submit(["bash", "-c", "sleep 0.1", "short-silent"], extra={lib.KEY_VARIABLE: "selftest-short-silent"}))
    world.stop_server()
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": world.write_fake_scope(SHORT_SAMPLED_PEAK_BYTES)})
    larger = world.finished(world.submit(["bash", "-c", "sleep 0.1", f"fake-wrapper-exit-peak={2 * SHORT_SAMPLED_PEAK_BYTES}"],
                                         extra={lib.KEY_VARIABLE: "selftest-short-larger"}))
    smaller = world.finished(world.submit(["bash", "-c", "sleep 0.1", f"fake-wrapper-exit-peak={SHORT_SAMPLED_PEAK_BYTES // 2}"],
                                          extra={lib.KEY_VARIABLE: "selftest-short-smaller"}))
    problems = []
    for name, summary, wanted in (("按轮采不到、退出时读到 123M 的", short, SHORT_EXIT_PEAK_BYTES), ("两样都没有的", silent, None),
                                  ("退出时读的 400M 比按轮的 200M 大的（照按轮的）", larger, SHORT_SAMPLED_PEAK_BYTES),
                                  ("退出时读的 100M 比按轮的 200M 小的（照按轮的）", smaller, SHORT_SAMPLED_PEAK_BYTES)):
        attempt = first_attempt(summary)
        if summary["task"].get("outcome") != "succeeded" or attempt.get("memory_peak", "缺这一栏") != wanted:
            problems.append(f"{name}应当跑完、记 memory_peak {wanted}，实际 {describe(summary)}、记 {attempt.get('memory_peak', '缺这一栏')}")
    if problems:
        raise SelftestFailure("跑得短的命令的内存峰值：" + "；".join(problems), world.server_log_lines())
    return "跑 0.1 秒就退、按轮采不到的记了退出时读的 123M；两样都没有的记 null；按轮采到了的照按轮的 200M，退出时读的不用"


def cell_merge_on_peer_reads_shard_outputs(world):
    """合并派到第二台（task_server_lib.py 文件头「分片」③）：本机放不下，计划、两片与合并都在第二台跑（假第二台照真的那样在这一刻的工作目录里起假分片入口）；
    合并那一次的环境里有 TASK_SERVER_SHARD_OUTPUTS，读得到两片的产物，整件成了。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下 4G：都派第二台
    world.start_server()
    summary = world.finished(world.submit(world.write_spec("spec-merge-on-peer.json", {"shards": [{}, {}]}), cwd=world.repo), timeout=120)
    merge_hosts = [attempt.get("host") for child in summary.get("children", []) if child["task"].get("role") == "merge" for attempt in child["attempts"]]
    merged = world.sharded_calls("merge", present="0,1")
    if summary["task"].get("outcome") != "succeeded" or merge_hosts != ["peer"] or not merged:
        raise SelftestFailure(f"合并派到第二台时应当读得到 TASK_SERVER_SHARD_OUTPUTS、两片都在、整件成了，实际整件 {summary['task'].get('outcome')}（{summary['task'].get('note')}）、"
                              f"合并各次的台 {merge_hosts}、合并的记录 {merged}", world.server_log_lines())
    return "计划、两片与合并都派到第二台：合并那一次读到 TASK_SERVER_SHARD_OUTPUTS、两片都在，整件成了"


PROBE_CHILD_GONE_SECONDS = 5.0   # stop-ends-probe-children：服务退出之后至多等这么久看探测子进程没了


def cell_stop_ends_probe_children(world):
    """停服务时服务正有一条复核挂载的 ssh 子进程（假 ssh 的 fake-ssh-hang：stat -f 那一句睡着不退）：服务收到 TERM 之后停掉它再退（task_server.py 的
    stop_own_children），服务退出之后那个进程号不在了。弄坏开关下留着的那一个由这一格按它写下的进程号停掉（只停这一格的假 ssh）。"""
    world.start_server()
    hang = os.path.join(world.world, "fake-ssh-hang")
    open(hang, "w").close()
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    while not os.path.exists(hang + "-pid") and patience.waiting():
        time.sleep(POLL_SECONDS)
    try:
        with open(hang + "-pid", encoding="utf-8") as handle:
            probe = int(handle.read().strip())
    except (OSError, ValueError):
        raise SelftestFailure(f"等了 {STATE_TIMEOUT_SECONDS:.0f} 秒，服务的 NFS 复核没起到假 ssh 复核挂载那一句", world.server_log_lines()) from None
    world.stop_server()
    patience = world.patience(PROBE_CHILD_GONE_SECONDS)
    while not process_gone(probe) and patience.waiting():
        time.sleep(POLL_SECONDS)
    if not process_gone(probe):
        with contextlib.suppress(OSError):
            with open(f"/proc/{probe}/cmdline", "rb") as handle:
                if b"sleep" in handle.read():
                    os.kill(probe, signal.SIGKILL)   # 只停这一格的假 ssh 写下的那一个进程号
        raise SelftestFailure(f"服务退出之后 {PROBE_CHILD_GONE_SECONDS:.0f} 秒，它起的复核挂载子进程（进程号 {probe}）应当已停，实际还在", world.server_log_lines())
    return f"停服务时正在跑的复核挂载子进程（进程号 {probe}）由服务停掉了，服务退出之后它不在了"


SCENE_COMMAND_EXIT = 3   # failure-scene-kept-until-collected：假包装演命令退非 0、留现场时退的码（表外的码，记 command-failed、不重排）
SCENE_GONE_SECONDS = 10.0   # 取走之后至多等这么久看现场删了


def scene_of(world, summary):
    """这一格那一次尝试的现场目录（尝试记录的 scene_directory）与照服务的规矩该在的那个路径（工作根下 SCENES_ROOT_NAME/<尝试标签>）。"""
    attempt = first_attempt(summary)
    return attempt.get("scene_directory") or "", os.path.join(world.work_root, lib.SCENES_ROOT_NAME, attempt.get("label") or "?")


def cell_failure_scene_kept_until_collected(world):
    """判红那一趟的现场（task_server_lib.py 文件头「现场」）：服务给内存包装的现场目录在工作根下 SCENES_ROOT_NAME/<尝试标签>、记进尝试的 scene_directory，
    假包装照它建、在日志里点名；没 wait 之前过了取走之后的时限也留着，wait 取走之后删；提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的取走之后也留着、task clean 删。"""
    world.start_server()
    keep_seconds = SERVER.FAST_SCENE_KEEP_AFTER_COLLECT_SECONDS
    problems = []
    plain_id = world.submit(["bash", "-c", "exit 0", f"fake-wrapper-scene={SCENE_COMMAND_EXIT}"])
    plain = world.finished(plain_id)
    scene, wanted = scene_of(world, plain)
    log_text = ""
    with contextlib.suppress(OSError):
        with open(first_attempt(plain).get("log") or "", encoding="utf-8", errors="replace") as handle:
            log_text = handle.read()
    if plain["task"].get("outcome") != "command-failed" or not scene or os.path.realpath(scene) != os.path.realpath(wanted):
        problems.append(f"退 {SCENE_COMMAND_EXIT} 的尝试应当记 command-failed、现场目录记成 {wanted}，实际 {describe(plain)}、记「{scene}」")
    elif f"现场目录：{scene}" not in log_text:
        problems.append(f"日志里应当点名现场目录 {scene}，实际没有")
    time.sleep(3 * keep_seconds)
    if not scene or not os.path.exists(os.path.join(scene, "left-by-command")):
        problems.append(f"还没 wait 取走结局时现场 {scene} 应当留着（过了取走之后的时限 {keep_seconds:g} 秒也不删），实际不在")
    world.client(["wait", str(plain_id)])
    patience = world.patience(SCENE_GONE_SECONDS)
    while scene and os.path.lexists(scene) and patience.waiting():
        time.sleep(POLL_SECONDS)
    if scene and os.path.lexists(scene):
        problems.append(f"wait 取走结局之后 {SCENE_GONE_SECONDS:.0f} 秒里现场 {scene} 应当删了，实际还在")
    kept_id = world.submit(["bash", "-c", "exit 0", f"fake-wrapper-scene={SCENE_COMMAND_EXIT}"], extra=KEEP_TEMPORARY_EXTRA)
    kept_scene, _ = scene_of(world, world.finished(kept_id))
    world.client(["wait", str(kept_id)])
    time.sleep(3 * keep_seconds)
    if not kept_scene or not os.path.exists(os.path.join(kept_scene, "left-by-command")):
        problems.append(f"设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的现场 {kept_scene} 取走之后也应当留着（等 task clean），实际不在")
    cleaned = world.client(["clean", str(kept_id)])
    if cleaned.returncode != 0 or (kept_scene and os.path.lexists(kept_scene)):
        problems.append(f"task clean 应当退 0、删了现场 {kept_scene}，实际退 {cleaned.returncode}、现场{'还在' if kept_scene and os.path.lexists(kept_scene) else '没了'}、"
                        f"输出 {cleaned.stdout.strip()[-200:]} {cleaned.stderr.strip()[-200:]}")
    if problems:
        raise SelftestFailure("判红那一趟的现场：" + "；".join(problems), world.server_log_lines())
    return "现场在工作根下这一次尝试的目录里、日志点名它、没取走之前留着、wait 取走之后删了；设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1 的取走之后留着、task clean 删了"


QUEUED_SHARD_KEY = "selftest-shard-third-queued"   # 三片分片的命令键：第 0、1 片记过 2G、第 2 片记过 30G（start_three_shards）
TASK_LOCK_ROUNDS = 1000   # task-locks-released 拿放锁的任务条数：表不随它涨才算淘汰了


def start_three_shards(world, name, shards):
    """三片的一件，第 0、1 片同时在本机跑、第 2 片排着：本机 slice 20G、第二台用不了，第 2 片记过的峰值 30G 只有第二台放得下，排着不判永远放不下
    （同 down-host-waits-not-never）。shards 是规格里三片的写法；等到这个样子交父任务号与 {片号: 子任务号}。"""
    for index, peak in ((0, 2), (1, 2), (2, 30)):
        world.seed_peak(lib.shard_child_key(QUEUED_SHARD_KEY, "shard", index, 3), peak * GIBIBYTE)
    world.resources(local=(50, 20), peer_down="自证：第二台此刻用不了")
    world.start_server()
    task_id = world.submit(world.write_spec(f"spec-{name}.json", {"shards": shards}), cwd=world.repo, extra={lib.KEY_VARIABLE: QUEUED_SHARD_KEY})
    summary = world.wait_for(task_id, lambda summary: sorted(running_shards(summary)) == [0, 1] and queued_shards(summary) == [2], "第 0、1 片跑起来、第 2 片排着")
    return task_id, {index: child["task"]["id"] for index, child in shard_children(summary).items()}


def queued_shards(summary):
    """分片父任务的各片里还排着、一次没起的片号。"""
    return sorted(index for index, child in shard_children(summary).items() if child["task"].get("state") == "queued" and not child["attempts"])


def shard_outcomes(summary):
    """分片父任务的各片：{片号: 结局（还没结局是空串）}。"""
    return {index: child["task"].get("outcome") or "" for index, child in shard_children(summary).items()}


def cell_shard_failure_waits_for_running(world):
    """一片 command-failed、一片在跑、一片在排（task_server_lib.py 文件头「分片」的「有一步最终没成」）：在排的那片当场 cancelled、一次不起；在跑的那片照跑到底、
    结局照收（succeeded）；收齐之后父任务才记那一片的 command-failed、wait 末行点名 shard=0/3，不合并。弄坏开关 shard-siblings-abandoned 下父任务当场收尾，这一格红。"""
    releases = {name: os.path.join(world.root, f"release-failure-{name}") for name in ("failing", "running")}
    task_id, _ = start_three_shards(world, "failure-waits", [{"block": releases["failing"], "exits": [3]}, {"block": releases["running"]}, {}])
    problems = []
    try:
        open(releases["failing"], "w").close()
        middle = world.wait_for(task_id, lambda summary: shard_outcomes(summary).get(0) == "command-failed" and shard_outcomes(summary).get(2) == "cancelled",
                                "第 0 片记 command-failed、第 2 片记 cancelled")
        if middle["task"].get("state") == "finished" or 1 not in running_shards(middle):
            problems.append(f"第 0 片没成、第 1 片还在跑时：父任务应当还没结局、第 1 片照跑，实际父任务 {middle['task'].get('state')}/{middle['task'].get('outcome')}、"
                            f"在跑的片 {running_shards(middle)}")
    finally:
        open(releases["running"], "w").close()
    finished = world.finished(task_id)
    code, last = wait_last_line(world, task_id)
    outcomes = shard_outcomes(finished)
    third_runs = world.sharded_calls("shard", index=2)
    if (finished["task"].get("outcome") != "command-failed" or code != 3 or " shard=0/3" not in last
            or outcomes != {0: "command-failed", 1: "succeeded", 2: "cancelled"} or third_runs
            or "merge" in child_roles(finished) or world.sharded_calls("merge")):
        problems.append(f"收齐之后：整件应当是第 0 片的 command-failed、wait 退 3、末行点名 shard=0/3，第 1 片 succeeded、第 2 片 cancelled 一次没起，不合并；"
                        f"实际 {describe(finished)}、wait 退 {code}、末行「{last}」、各片 {outcomes}、第 2 片跑的记录 {third_runs}、子任务 {child_roles(finished)}、"
                        f"合并的记录 {world.sharded_calls('merge')}")
    if problems:
        raise SelftestFailure("一片没成：" + "；".join(problems), world.server_log_lines())
    return "一片没成：在排的兄弟片当场撤、在跑的跑完结局照收，收齐之后整件才记那一片的 command-failed、不合并"


def cell_shard_cancel_alone_merges(world):
    """人手只取消在排的一片（task cancel <那一片的子任务号>）：只那一片 cancelled，别的片照跑；合并照常起、TASK_SERVER_SHARD_MISSING 点名缺第 2 片，
    假合并判本次未跑（退 77），整件 not-run、末行 step=merge。弄坏开关 shard-cancel-ends-whole（整件收尾、不合并）与 shard-missing-unnamed（合并拿不到缺片）下这一格红。"""
    release = os.path.join(world.root, "release-cancel-alone")
    task_id, children = start_three_shards(world, "cancel-alone", [{"block": release}, {"block": release}, {}])
    problems = []
    try:
        completed = world.client(["cancel", str(children[2])])
        middle = world.summary(task_id)
        if (completed.returncode != 0 or shard_outcomes(middle).get(2) != "cancelled" or middle["task"].get("state") == "finished"
                or sorted(running_shards(middle)) != [0, 1]):
            problems.append(f"取消在排的第 2 片：应当退 0、只那一片 cancelled、父任务没结局、第 0 与 1 片照跑，实际退 {completed.returncode}、各片 {shard_outcomes(middle)}、"
                            f"父任务 {middle['task'].get('state')}/{middle['task'].get('outcome')}、在跑的片 {running_shards(middle)}（{completed.stdout.strip()[-200:]}）")
    finally:
        open(release, "w").close()
    finished = world.finished(task_id)
    code, last = wait_last_line(world, task_id)
    merges = world.sharded_calls("merge")
    if ([(row.get("present"), row.get("missing")) for row in merges] != [("0,1", "2")] or finished["task"].get("outcome") != "not-run" or code != 77
            or " step=merge" not in last or shard_outcomes(finished) != {0: "succeeded", 1: "succeeded", 2: "cancelled"}):
        problems.append(f"合并应当照常起一次、拿到第 0 与 1 片的产物与缺片的点名（missing=2），判本次未跑：整件 not-run、wait 退 77、末行 step=merge；"
                        f"实际合并的记录 {merges}、{describe(finished)}、wait 退 {code}、末行「{last}」、各片 {shard_outcomes(finished)}")
    if problems:
        raise SelftestFailure("只取消一片：" + "；".join(problems), world.server_log_lines())
    return "人手只取消在排的一片：别的片照跑，合并照常起、点名缺的那一片，判本次未跑"


def cell_parent_cancel_stops_every_shard(world):
    """两片在跑、一片在排时取消整件（父任务号）：照旧全停——在排的那片当场 cancelled、一次不起，在跑的两片回来时单元都停了、各记 cancelled，整件 cancelled（退 239），
    不合并。弄坏开关 parent-cancel-spares-running 下在跑的不停，这一格红。"""
    release = os.path.join(world.root, "release-parent-cancel")
    task_id, _ = start_three_shards(world, "parent-cancel", [{"block": release}, {"block": release}, {}])
    problems = []
    try:
        units = [child["attempts"][-1].get("unit", "") for index, child in shard_children(world.summary(task_id)).items() if index in (0, 1)]
        completed = world.client(["cancel", str(task_id)])
        still = [unit for unit in units if world.unit_active(unit)]
        summary = world.summary(task_id)
        if completed.returncode != 0 or summary["task"].get("outcome") != "cancelled" or summary["task"].get("exit_code") != 239 or still:
            problems.append(f"取消整件：应当退 0、整件记 cancelled（退 239）、回来时两片的单元都停了，实际退 {completed.returncode}、"
                            f"整件 {summary['task'].get('outcome')}、还在跑的单元 {still}")
    finally:
        open(release, "w").close()
    for index, child in sorted(shard_children(world.summary(task_id)).items()):
        finished = world.finished(child["task"]["id"])
        if finished["task"].get("outcome") != "cancelled":
            problems.append(f"第 {index} 片应当记 cancelled，实际 {describe(finished)}")
    if world.sharded_calls("shard", index=2) or "merge" in child_roles(world.summary(task_id)) or world.sharded_calls("merge"):
        problems.append(f"在排的第 2 片应当一次不起、取消了的整件不合并，实际第 2 片跑的记录 {world.sharded_calls('shard', index=2)}、子任务 {child_roles(world.summary(task_id))}")
    if problems:
        raise SelftestFailure("取消整件：" + "；".join(problems), world.server_log_lines())
    return "两片在跑、一片在排时取消整件：在排的没起，在跑的单元回来时都停了，整件 cancelled、不合并"


def cell_cancel_after_natural_end(world):
    """取消记了 cancel_requested、放了锁、停单元之前精确停住（TASK_SERVER_SELFTEST_PAUSE_POINT=cancel-unlocked），这时命令自己退 0、结局写回 succeeded：
    取消不许回 stopped（task 退非 0、说明点名 succeeded），结局照旧 succeeded（TS-03）。弄坏开关 cancel-outcome-unchecked 下回 stopped，这一格红。"""
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    release = os.path.join(world.root, "release-natural-end")
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"cancel-unlocked:{pause}"})
    task_id = world.submit(blocking_command(release))
    world.running(task_id)
    open(os.path.join(pause, "arm-cancel-unlocked"), "w").close()
    canceller = world.client(["cancel", str(task_id)], background=True)
    try:
        wait_path(world, os.path.join(pause, "reached-cancel-unlocked"), "停在记了取消、放了锁、停单元之前")
        open(release, "w").close()
        finished = world.finished(task_id)
    finally:
        open(release, "w").close()
        open(os.path.join(pause, "release-cancel-unlocked"), "w").close()
    stdout, stderr = world.communicate(canceller, STATE_TIMEOUT_SECONDS)
    if canceller.returncode == 0 or "succeeded" not in stdout + stderr or finished["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"取消停住的这一会儿命令自己退 0、结局写回 succeeded：取消应当退非 0、说明点名 succeeded，结局照旧 succeeded；"
                              f"实际退 {canceller.returncode}、{describe(finished)}（{(stdout + stderr).strip()[-300:]}）", world.server_log_lines())
    return f"取消到的时候命令已自己跑完：task cancel 退 {canceller.returncode}、点名结局 succeeded，不报停了"


def cell_failed_start_not_counted(world):
    """本机放不下、都去第二台：排在前面的 PEER_STARTS_PER_PASS 条 systemd-run 起不来（假 systemd-run 认工作目录 refuse-start），后面同样条数的放得下：
    没起成的不占这一轮在第二台的名额，后面那几条与它们同一轮起（数服务记的「哪一轮起的」）。弄坏开关 failed-start-counted 下后面那几条挪到下一轮，这一格红。"""
    count = SERVER.PEER_STARTS_PER_PASS
    key = "selftest-failed-start"
    world.resources(local=(1, 1), peer=(100, 100), peer_cap_gibibytes=64, hold=True)
    world.seed_peak(key, 2 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SYSTEMD_RUN": os.path.join(world.bin, "fake-systemd-run-selective")})
    refusing = os.path.join(world.repo, "refuse-start")
    os.makedirs(refusing)
    refused = [world.submit(["bash", "-c", "exit 0", "refused", "fake-peer-run"], extra={lib.KEY_VARIABLE: key}, cwd=refusing) for _ in range(count)]
    started = [world.submit(["bash", "-c", "exit 0", "started", "fake-peer-run"], extra={lib.KEY_VARIABLE: key}, cwd=world.repo) for _ in range(count)]
    world.resources(local=(1, 1), peer=(100, 100), peer_cap_gibibytes=64)
    world.api("POST", "/wake")
    outcomes = {task_id: world.finished(task_id, timeout=120)["task"].get("outcome") for task_id in refused + started}
    views = {}
    for line in world.scheduler_reads():
        found = re.match(r"^start task=(\d+) host=(\w+) view=(\d+)$", line)
        if found and int(found.group(1)) in outcomes:
            views[int(found.group(1))] = (found.group(2), int(found.group(3)))
    first = views.get(refused[0], ("", -1))
    if (any(outcomes[task_id] != "unit-start-failed" for task_id in refused) or any(outcomes[task_id] != "succeeded" for task_id in started)
            or any(views.get(task_id) != first for task_id in refused + started) or first[0] != "peer"):
        raise SelftestFailure(f"前 {count} 条在第二台起不来、后 {count} 条放得下：起不来的记 unit-start-failed、后面那几条 succeeded，{2 * count} 条都在第二台同一轮起；"
                              f"实际结局 {outcomes}、各条在哪台哪一轮起 {views}", world.server_log_lines())
    return f"第二台一轮的名额 {count} 条：前 {count} 条起不来不占名额，后 {count} 条与它们同一轮起"


def raw_record_update(world, table, row_id, changes):
    """绕过服务只改一行的正文（并进 changes；关联列与签名不动）：读回时那一行校验不过。"""
    with contextlib.closing(sqlite3.connect(os.path.join(world.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
        (text,) = connection.execute(f"SELECT record FROM {table} WHERE id = ?", (row_id,)).fetchone()
        connection.execute(f"UPDATE {table} SET record = ? WHERE id = ?", (lib.canonical_json(dict(json.loads(text), **changes)), row_id))


def cell_child_rows_verified(world):
    """children_of 不丢校验位（TS-05）：一件分片跑完之后绕过服务改它那一片子任务行的正文：task clean 回 409、一份都不删；status 不列那一行、
    children_unverified 点名它，写了 record-tampered 告警。弄坏开关 child-rows-unverified 下照收那一行，这一格红。"""
    world.start_server()
    task_id = world.submit(world.write_spec("spec-child-rows.json", {"shards": [{}]}), cwd=world.repo)
    finished = world.finished(task_id)
    children = shard_children(finished)
    if finished["task"].get("outcome") != "succeeded" or 0 not in children:
        raise SelftestFailure(f"一片的分片应当成了，实际 {describe(finished)}、各片 {sorted(children)}", world.server_log_lines())
    child_id = children[0]["task"]["id"]
    raw_record_update(world, "tasks", child_id, {"note": "自证：绕过服务改了正文"})
    cleaned = world.client(["clean", str(task_id)])
    status = world.summary(task_id)
    listed = [child["task"].get("id") for child in status.get("children", [])]
    alerted = [alert for alert in world.alerts() if alert.get("kind") == "record-tampered" and alert.get("task") == task_id]
    problems = []
    if cleaned.returncode == 0:
        problems.append(f"子任务行 {child_id} 校验不过：task clean 应当退非 0、一份都不删，实际退 0（{cleaned.stdout.strip()[-200:]}）")
    if f"行 [{child_id}]" not in status.get("children_unverified", "") or child_id in listed or not alerted:
        problems.append(f"status 应当不列子任务行 {child_id}、children_unverified 点名它、写 record-tampered 告警，实际列了 {listed}、"
                        f"children_unverified「{status.get('children_unverified', '')}」、告警 {len(alerted)} 条")
    if problems:
        raise SelftestFailure("子任务行校验：" + "；".join(problems), world.server_log_lines())
    return "子任务行的正文被改过：clean 回 409 不删，status 不列它、点名它、写告警"


def cell_task_locks_released(world):
    """Store.task_lock 用完就从锁表删（task_server.py 的 Store 文件头）：在这一格的临时库上对 TASK_LOCK_ROUNDS 条任务各拿放一次锁，表空着；
    一条任务的锁有人拿着时另一个线程等着、拿着的放下之后它才拿到（删表项不破坏互斥），都放下之后表空着。弄坏开关 task-locks-kept 下表随任务数涨，这一格红。
    只用 task_server.py 的 Store 这一个类，不起服务。"""
    store = SERVER.Store(os.path.join(world.root, "locks.sqlite3"), bytes(32), dict(os.environ))
    for task_id in range(1, TASK_LOCK_ROUNDS + 1):
        with store.task_lock(task_id):
            pass
    after_rounds = len(store.task_locks)
    order = []
    waiter_started = threading.Event()

    def waiter():
        waiter_started.set()
        with store.task_lock(1):
            order.append("waiter")

    with store.task_lock(1):
        thread = threading.Thread(target=waiter, daemon=True)
        thread.start()
        waiter_started.wait(timeout=10)
        time.sleep(0.3)   # 让等的那一个真的等在锁上：这时它不该拿到
        order.append("holder")
    thread.join(timeout=10)
    if after_rounds != 0 or order != ["holder", "waiter"] or thread.is_alive() or store.task_locks:
        raise SelftestFailure(f"{TASK_LOCK_ROUNDS} 条任务各拿放一次锁之后锁表应当是空的，实际 {after_rounds} 项；拿着时另一个线程应当等到放下才拿到，"
                              f"实际先后 {order}、等的线程{'还没退' if thread.is_alive() else '退了'}、最后锁表 {len(store.task_locks)} 项")
    return f"{TASK_LOCK_ROUNDS} 条任务的锁用完锁表空着；拿着时另一个线程等到放下才拿到"


ATTENTION_QUEUE_SECONDS = "2"        # attention-queued-too-long：排队过久的头一档（秒；TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS）
ATTENTION_ONCE_SECONDS = 6           # attention-reported-once：头一档 6 秒、第二档 12 秒（每档翻倍）
ATTENTION_ONCE_QUIET_SECONDS = 2.5   # attention-reported-once：头一档叫过之后再起 wait，这么久里不该再叫（离第二档 12 秒还有 6 秒）
ATTENTION_WAIT_SECONDS = 60          # 各提醒格里 wait 至多等这么久（弄坏开关下它一直等，等满就判红）
ATTENTION_QUIET_SECONDS = "3"        # attention-quiet-busy-log：日志长期不涨的头一档（秒；TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS），比卡住阈值 2 秒长
ATTENTION_IO_WAIT_SECONDS = "3"      # io-wait-local-not-stalled：卡在 IO 上的头一档（秒；TASK_SERVER_SELFTEST_IO_WAIT_ATTENTION_SECONDS），比卡住阈值 2 秒长
FAKE_BLOCKED_PROCESS = 4194301       # io-wait-local-not-stalled：假 scope 的 cgroup.procs 点名的进程号，假 /proc 下只有它一份 stat、是 D 状态（服务照假 /proc 读，与真进程号撞了也只读这一份）
IO_WAIT_SUSPECT_WAITS = 3            # io-wait-local-not-stalled：CPU 不涨先报的疑似卡住照报，至多再起这么多回 wait 等 io-wait 那一行


def attention_lines(completed):
    return [line for line in completed.stdout.splitlines() if line.startswith(lib.ATTENTION_MARK + " ")]


def attention_fields(line):
    """TASK_SERVER_ATTENTION 那一行拆成 {名: 值}（值照 shlex 引，拆得回）。"""
    return dict(word.split("=", 1) for word in shlex.split(line)[1:] if "=" in word)


def attention_problems(completed, task_id, kind, expected_exit):
    """wait 报要处置的提醒的共同判据：恰好一行 TASK_SERVER_ATTENTION，task=<号> type=needs-attention kind=<kind>，依据、原因、出路都不空，退 expected_exit；
    退的是 ATTENTION_EXIT 的不该打 TASK_SERVER_RESULT。→ (问题表, 那一行拆出的栏)"""
    lines = attention_lines(completed)
    fields = attention_fields(lines[0]) if lines else {}
    problems = []
    if completed.returncode != expected_exit or len(lines) != 1:
        problems.append(f"wait 应当打一行 {lib.ATTENTION_MARK}、退 {expected_exit}，实际退 {completed.returncode}、打了 {len(lines)} 行，输出「{completed.stdout.strip()[-300:]}」")
    elif (fields.get("task"), fields.get("type"), fields.get("kind")) != (str(task_id), lib.ATTENTION_TYPE, kind) or not all(fields.get(name) for name in ("basis", "why", "next")):
        problems.append(f"那一行应当 task={task_id} type={lib.ATTENTION_TYPE} kind={kind} 且依据、原因、出路都不空，实际「{lines[0][:400]}」")
    elif not fields["next"].startswith(table_advice(kind, fields.get("part") or task_id)):   # 出路的头一段是异常表这一种那一行的修改意见（task_server_lib.py 文件头「异常表」）
        problems.append(f"那一行的 next= 应当以异常表里 {kind} 那一行的修改意见「{table_advice(kind, fields.get('part') or task_id)[:120]}…」起头，实际「{fields['next'][:300]}」")
    if expected_exit == lib.ATTENTION_EXIT and lib.RESULT_MARK in completed.stdout:
        problems.append(f"任务还没结局，wait 不该打 {lib.RESULT_MARK}，实际输出「{completed.stdout.strip()[-300:]}」")
    return problems, fields


def quiet_log_busy_problems(waited, which):
    """io-wait-local-not-stalled、gpu-busy-quiet-not-stalled 共用：几回 wait 的输出里不该有 kind=quiet-log-busy 的那一行（服务不是因为 CPU 在涨或读不到才没停它）。"""
    lines = [line for completed in waited for line in attention_lines(completed) if attention_fields(line).get("kind") == "quiet-log-busy"]
    return [f"{which}不该报 quiet-log-busy，实际 wait 打了「{lines[0][:300]}」"] if lines else []


def cell_attention_queued_too_long(world):
    """排队过久（task_server.py 文件头 ①「要处置的提醒」）：第二台用不了、只有它放得下的命令一直排着（同 down-host-waits-not-never），排满头一档（自证里 2 秒）
    wait 打一行 TASK_SERVER_ATTENTION kind=queued-too-long、依据写已排多久、原因写排队原因（第二台用不了）、退 ATTENTION_EXIT；任务照旧在排（没起尝试、没结局），
    第二台回来之后照派过去跑完。"""
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")
    world.seed_peak("selftest-attention-queue", 30 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": ATTENTION_QUEUE_SECONDS})
    task_id = world.submit(["bash", "-c", "exit 0", "attention-queue"], extra={lib.KEY_VARIABLE: "selftest-attention-queue"}, cwd=world.repo)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    during = world.summary(task_id)
    problems, fields = attention_problems(completed, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    if "已排" not in fields.get("basis", "") or "第二台" not in fields.get("why", ""):
        problems.append(f"依据应当写「已排 …」、原因应当带排队原因里的「第二台」，实际依据「{fields.get('basis')}」、原因「{fields.get('why')}」")
    if during["task"].get("state") != "queued" or during["task"].get("outcome") or during["attempts"]:
        problems.append(f"报了排队过久之后任务应当照旧在排（没起尝试、没结局），实际 {describe(during)}、状态 {during['task'].get('state')}")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"第二台回来之后应当派过去跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("排队过久：" + "；".join(problems), world.server_log_lines())
    return f"排满头一档 {ATTENTION_QUEUE_SECONDS} 秒：wait 打 {lib.ATTENTION_MARK} kind=queued-too-long（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，任务照旧在排，第二台回来后跑完"


def cell_attention_overrun(world):
    """跑到记录用时的很多倍（日志在涨、服务不停它）：记录用时 1 秒，跑到 3 倍（3 秒，宽限压到 0.01 秒）wait 打一行 kind=overrun、依据写已跑多久与倍数、
    退 ATTENTION_EXIT；任务照旧在跑，放行之后跑完。"""
    key = "selftest-attention-overrun"
    release = os.path.join(world.root, "release-attention-overrun")
    command = ["bash", "-c", BLOCK_SCRIPT, "block", release]
    world.seed_duration(key, 1, command)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01"})
    task_id = world.submit(command, extra={lib.KEY_VARIABLE: key})
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    during = world.summary(task_id)
    problems, fields = attention_problems(completed, task_id, "overrun", lib.ATTENTION_EXIT)
    if "记录用时 1 秒" not in fields.get("basis", "") or "倍" not in fields.get("basis", ""):
        problems.append(f"依据应当写「记录用时 1 秒」与倍数，实际「{fields.get('basis')}」")
    if during["task"].get("state") != "running" or not [attempt for attempt in during["attempts"] if attempt.get("state") == "running"]:
        problems.append(f"报了超过预计之后任务应当照旧在跑（running），实际 {describe(during)}、状态 {during['task'].get('state')}")
    open(release, "w").close()
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("超过预计：" + "；".join(problems), world.server_log_lines())
    return f"记录用时 1 秒跑到 3 倍：wait 打 {lib.ATTENTION_MARK} kind=overrun（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，任务照旧在跑，放行后跑完"


def cell_attention_quiet_busy_log(world):
    """日志长期不涨而服务没停它（task_server.py 的 quiet_log_items；任务 7723 日志 37702 秒没涨、没有记录用时，服务一直不停也没人叫）：没有记录用时的一条只打一行
    就不出声、跑到放行为止，假 scope 的 CPU 一直在涨（卡住阈值 2 秒：CPU 在涨不停），此刻内存读不到（不判贴着上限）；日志满头一档（自证里 3 秒）没涨，
    wait 打一行 kind=quiet-log-busy、依据写日志多久没涨、退 ATTENTION_EXIT；任务照旧在跑，放行之后跑完。
    弄坏开关 attention-quiet-busy-unreported 下 wait 不叫醒、等到命令自己退，这一格红。"""
    scope, stop = start_burning_scope(world, "quiet-busy-scope", {})
    release = os.path.join(world.root, "release-attention-quiet")
    problems, fields = [], {}
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope,
                                  "TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS": ATTENTION_QUIET_SECONDS})
        task_id = world.submit(["bash", "-c", "echo started; " + BLOCK_SCRIPT, "quiet-busy", release])
        completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
        during = world.summary(task_id)
        problems, fields = attention_problems(completed, task_id, "quiet-log-busy", lib.ATTENTION_EXIT)
        if "没涨" not in fields.get("basis", ""):
            problems.append(f"依据应当写日志多久「没涨」，实际「{fields.get('basis')}」")
        if during["task"].get("state") != "running" or not [attempt for attempt in during["attempts"] if attempt.get("state") == "running"]:
            problems.append(f"报了日志长期不涨之后任务应当照旧在跑（running），实际 {describe(during)}、状态 {during['task'].get('state')}")
        open(release, "w").close()
        done = world.finished(task_id)
    finally:
        open(release, "w").close()
        stop.set()
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("日志长期不涨而服务没停它：" + "；".join(problems), world.server_log_lines())
    return f"日志满头一档 {ATTENTION_QUIET_SECONDS} 秒没涨、CPU 在涨：wait 打 {lib.ATTENTION_MARK} kind=quiet-log-busy（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，任务照旧在跑，放行后跑完"


def cell_io_wait_local_not_stalled(world):
    """卡在 IO 上（task_server.py 文件头 ①「卡在 IO 上」；任务 8608、8609、8611、8619 在等 NFS 时被当成卡住停掉）：本机一条只打一行就不出声、跑到放行为止，
    假 scope 的 CPU 一直不涨（卡住阈值 2 秒），假 scope 的 cgroup.procs 点名一个进程号，假 /proc（TASK_SERVER_SELFTEST_PROC_ROOT）下它的 stat 是 D 状态：
    服务不判 stalled、不停；日志没涨的每一轮都有 D 状态的进程满头一档（自证里 3 秒），wait 打一行 kind=io-wait、依据写 D 状态的进程数与 /proc/pressure/io 的 avg60
    （读不到照写读不到）、退 ATTENTION_EXIT（CPU 不涨先报的疑似卡住照报，再起 wait 接着等）；任务照旧在跑，放行之后跑完。
    没有 D 状态的照旧停 stalled 由 stalled-attempt-stopped 那一格判（同一段判法：假 /proc 不换，那一条的真进程都在睡、不在 D 状态）。
    日志长期不涨的头一档同样压到 3 秒（ATTENTION_QUIET_SECONDS）：按 D 状态免停的不报 quiet-log-busy（task_server.py 的 quiet_log_items），各回 wait 都不该打那一种。
    弄坏开关 io-wait-ignored 下停掉、记 stalled，attention-io-wait-unreported 下 wait 不叫醒、等到命令自己退，quiet-log-busy-any-exemption 下 wait 另报 quiet-log-busy，这一格红。"""
    scope = os.path.join(world.root, "io-wait-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:   # CPU 读得到、一直不涨
        handle.write("usage_usec 1000\n")
    with open(os.path.join(scope, "cgroup.procs"), "w", encoding="utf-8") as handle:
        handle.write(f"{FAKE_BLOCKED_PROCESS}\n")
    proc_root = os.path.join(world.root, "io-wait-proc")
    os.makedirs(os.path.join(proc_root, str(FAKE_BLOCKED_PROCESS)))
    with open(os.path.join(proc_root, str(FAKE_BLOCKED_PROCESS), "stat"), "w", encoding="utf-8") as handle:   # 进程名带括号与空格：状态照最后一个右括号切
        handle.write(f"{FAKE_BLOCKED_PROCESS} (git (diff) D 1 {FAKE_BLOCKED_PROCESS} {FAKE_BLOCKED_PROCESS} 0 -1 4194560 0 0 0 0 0 0 0 0 20 0 1 0 0 0 0\n")
    release = os.path.join(world.root, "release-io-wait")
    problems, fields, done = [], {}, None
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope,
                                  "TASK_SERVER_SELFTEST_IO_WAIT_ATTENTION_SECONDS": ATTENTION_IO_WAIT_SECONDS, "TASK_SERVER_SELFTEST_PROC_ROOT": proc_root,
                                  "TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS": ATTENTION_QUIET_SECONDS})
        task_id = world.submit(["bash", "-c", "echo started; " + BLOCK_SCRIPT, "io-wait", release])
        completed, waited = None, []
        for _ in range(IO_WAIT_SUSPECT_WAITS):
            completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
            waited.append(completed)
            if completed.returncode != lib.SUSPECT_EXIT:
                break
        during = world.summary(task_id)
        problems, fields = attention_problems(completed, task_id, "io-wait", lib.ATTENTION_EXIT)
        problems += quiet_log_busy_problems(waited, "按 D 状态免停的")
        basis = fields.get("basis", "")
        if "D 状态" not in basis or "1 个" not in basis or ("avg60=" not in basis and "读不到" not in basis):
            problems.append(f"依据应当写每一轮都有进程在「D 状态」、最近一轮「1 个」与 /proc/pressure/io 的 avg60（或读不到），实际「{basis}」")
        if during["task"].get("state") != "running" or not [attempt for attempt in during["attempts"] if attempt.get("state") == "running"]:
            problems.append(f"卡在 IO 上的应当照旧在跑（running、不判 stalled），实际 {describe(during)}、状态 {during['task'].get('state')}")
        open(release, "w").close()
        done = world.finished(task_id)
    finally:
        open(release, "w").close()
    if done is not None and done["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("本机卡在 IO 上：" + "；".join(problems), world.server_log_lines())
    return (f"本机日志与 CPU 都不涨而每一轮都有进程在 D 状态：不停，满头一档 {ATTENTION_IO_WAIT_SECONDS} 秒 wait 打 {lib.ATTENTION_MARK} kind=io-wait"
            f"（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，放行后跑完")


def peer_io_wait_script(blocked, read):
    """io-wait-peer-not-stalled 的第二台那条命令：14 轮、每轮 0.5 秒，只打起法那一层的三种进度行（日志不算在涨），CPU 一直不涨；
    IO_WAIT 那一行打在 CPU 那一行前面（服务读到 CPU 的那一回 IO_WAIT 一定已在，不因两行之间那一下判成这一轮读不到）。"""
    return ('for i in $(seq 14); do echo "MULTI_HOST_RUN_PEER_IO_WAIT poll=$i blocked=' + str(blocked) + ' read=' + read + '"; '
            'echo "MULTI_HOST_RUN_PEER_CPU poll=$i usage_usec=1000 read=yes"; echo "MULTI_HOST_RUN_PEER_MEMORY poll=$i current_bytes=0 peak_bytes=0 read=no"; sleep 0.5; done')


def cell_io_wait_peer_not_stalled(world):
    """第二台卡在 IO 上（task_server.py 文件头 ①「卡在 IO 上」）：三条派到第二台、日志与 CPU 都不涨（卡住阈值 2 秒）、跑 7 秒，每轮打一行 MULTI_HOST_RUN_PEER_IO_WAIT：
    blocked=2 read=yes 的不停、跑完；blocked=0 read=yes（没有 D 状态的进程）与 blocked=3 read=no（读不到）的照原判法停、记 stalled。
    弄坏开关 io-wait-ignored 下 blocked=2 的也停掉、记 stalled，这一格红。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-peer-io-wait", 10 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01"})
    expected = {"blocked": (2, "yes", "succeeded"), "unblocked": (0, "yes", "stalled"), "unread": (3, "no", "stalled")}
    submitted = {name: world.submit(["bash", "-c", peer_io_wait_script(blocked, read), name, "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-peer-io-wait"}, cwd=world.repo)
                 for name, (blocked, read, _) in expected.items()}
    summaries = {name: world.finished(task_id, timeout=60) for name, task_id in submitted.items()}
    problems = []
    for name, (blocked, read, outcome) in expected.items():
        summary = summaries[name]
        host = summary["attempts"][0].get("host") if summary["attempts"] else None
        if host != "peer" or summary["task"].get("outcome") != outcome:
            problems.append(f"blocked={blocked} read={read} 的那一条应当在第二台跑、记 {outcome}，实际在 {host}、{describe(summary)}")
    if problems:
        raise SelftestFailure("第二台卡在 IO 上：" + "；".join(problems), world.server_log_lines())
    return "第二台日志与 CPU 都不涨：每轮 blocked=2 read=yes 的不停、跑完；blocked=0 与 read=no 的照原判法记 stalled"


ATTENTION_GPU_BUSY_QUIET_SECONDS = "3"   # gpu-busy-quiet-not-stalled：卡上有它的计算进程而日志与 CPU 都不涨的头一档（秒；TASK_SERVER_SELFTEST_GPU_BUSY_QUIET_ATTENTION_SECONDS），比卡住阈值 2 秒长
# gpu-busy-quiet-not-stalled 与 card-zero-utilization-stalled 的跑法（名字照 GPU_SCRIPT_PATTERNS 认成要卡；没分到卡的那一条用 bash -c 起同一段正文、不认成要卡）：$1 演哪一种，
# busy 先把自己的进程号写进 $2、之后不动（每一轮卡上都有它的计算负载），flicker 每半秒写进去又删掉（有几轮卡上没有它的负载）；
# 先写一回、再打一行 started，之后不出声，等放行文件 $3（至多 60 秒，等不到退 9）
GPU_BUSY_QUIET_SCRIPT_NAME = "crash-amplification-gpu-busy-quiet-selftest.sh"
GPU_BUSY_QUIET_SCRIPT = r'''#!/usr/bin/env bash
echo "$$" > "$2"
echo started
for round in $(seq 1 600); do
  [[ -e "$3" ]] && exit 0
  if [[ "$1" == flicker ]]; then
    if (( round / 5 % 2 == 1 )); then rm -f "$2"; else echo "$$" > "$2"; fi
  fi
  sleep 0.1
done
exit 9
'''


def cell_gpu_busy_quiet_not_stalled(world):
    """卡上有它的计算进程（task_server.py 文件头 ①「卡上有它的计算进程」；任务 8519、8520 两条单条 GPU 对拍日志与 CPU 都不涨、GPU 在算时被当成卡住停掉）：
    本机两张卡，三条都只打一行就不出声、跑到放行为止，假 scope 的 CPU 一直不涨（卡住阈值 2 秒），假 nvidia-smi 照各自的 card-busy-pids-<名字> 报卡上的计算进程：
    假 nvidia-smi 不报卡的利用率（这一格不写 card-utilization-<序号>，读利用率退 9）：
    要卡、分到了卡、每一轮卡上都有它的计算负载的（busy）不判 stalled、不停，满头一档（自证里 3 秒）wait 打一行 kind=gpu-busy-quiet、依据写卡的键、「计算进程」
    与「利用率读不到」、退 ATTENTION_EXIT（卡上有负载，疑似卡住不报），放行之后跑完；要卡、分到了卡、每半秒有几轮卡上没有它的负载的（flicker）照原判法停、记 stalled；
    不要卡的（cardless，同一段正文用 bash -c 起，进程号同样写进名单）不读卡，照原判法停、记 stalled。
    日志长期不涨的头一档同样压到 3 秒（ATTENTION_QUIET_SECONDS）：按卡上负载免停的 busy 不报 quiet-log-busy（task_server.py 的 quiet_log_items）。
    弄坏开关 stall-card-load-ignored 下 busy 也停掉、记 stalled，attention-gpu-busy-quiet-unreported 下 wait 不叫醒、等到命令自己退，
    quiet-log-busy-any-exemption 下 wait 另报 quiet-log-busy，这一格红。"""
    world.resources(local=(100, 100), cards=f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE}", local_gpu=[(0, 16384), (1, 16384)])
    write_executable(os.path.join(world.work, GPU_BUSY_QUIET_SCRIPT_NAME), GPU_BUSY_QUIET_SCRIPT)
    release = os.path.join(world.root, "release-gpu-busy-quiet")
    listed = {name: os.path.join(world.world, f"card-busy-pids-{name}") for name in ("busy", "flicker", "cardless")}
    problems, fields, done = [], {}, None
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01",
                                  "TASK_SERVER_FAKE_SCOPE_DIRECTORY": idle_cpu_scope(world), "TASK_SERVER_NVIDIA_SMI": os.path.join(world.bin, "fake-nvidia-smi"),
                                  "TASK_SERVER_SELFTEST_GPU_BUSY_QUIET_ATTENTION_SECONDS": ATTENTION_GPU_BUSY_QUIET_SECONDS,
                                  "TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS": ATTENTION_QUIET_SECONDS})
        busy = world.submit(["bash", GPU_BUSY_QUIET_SCRIPT_NAME, "busy", listed["busy"], release])
        flicker = world.submit(["bash", GPU_BUSY_QUIET_SCRIPT_NAME, "flicker", listed["flicker"], release])
        cardless = world.submit(["bash", "-c", GPU_BUSY_QUIET_SCRIPT, "gpu-busy-quiet-cardless", "busy", listed["cardless"], release])
        completed = world.client(["wait", str(busy)], timeout=ATTENTION_WAIT_SECONDS)
        during = world.summary(busy)
        flickered = world.finished(flicker, timeout=60)
        cardless_done = world.finished(cardless, timeout=60)
        problems, fields = attention_problems(completed, busy, "gpu-busy-quiet", lib.ATTENTION_EXIT)
        problems += quiet_log_busy_problems([completed], "按卡上负载免停的")
        basis = fields.get("basis", "")
        card = ((during["attempts"] or [{}])[-1]).get("card") or ""
        if not card or card not in basis or "计算进程" not in basis or "利用率读不到" not in basis:
            problems.append(f"依据应当写卡的键（这一次分到的「{card}」）、「计算进程」与「利用率读不到」（假 nvidia-smi 不报利用率），实际「{basis}」")
        if not during["task"].get("gpu") or during["task"].get("state") != "running" or not [attempt for attempt in during["attempts"] if attempt.get("state") == "running"]:
            problems.append(f"要卡、每一轮卡上都有它的负载的应当认成要卡、照旧在跑（running、不判 stalled），实际认成{'要卡' if during['task'].get('gpu') else '不要卡'}、"
                            f"{describe(during)}、状态 {during['task'].get('state')}")
        if not flickered["task"].get("gpu") or not (flickered["attempts"] or [{}])[-1].get("card") or flickered["task"].get("outcome") != "stalled":
            problems.append(f"要卡、分到了卡、有几轮卡上没有它的负载的应当照原判法停、记 stalled，实际认成{'要卡' if flickered['task'].get('gpu') else '不要卡'}、{describe(flickered)}")
        if cardless_done["task"].get("gpu") or cardless_done["task"].get("outcome") != "stalled":
            problems.append(f"不要卡的（进程号同样在名单里）应当不读卡、照原判法停、记 stalled，实际认成{'要卡' if cardless_done['task'].get('gpu') else '不要卡'}、{describe(cardless_done)}")
        open(release, "w").close()
        done = world.finished(busy)
    finally:
        open(release, "w").close()
    if done is not None and done["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("卡上有它的计算进程：" + "；".join(problems), world.server_log_lines())
    return (f"要卡、每一轮卡上都有它的计算负载、日志与 CPU 都不涨：不停，满头一档 {ATTENTION_GPU_BUSY_QUIET_SECONDS} 秒 wait 打 {lib.ATTENTION_MARK} kind=gpu-busy-quiet"
            f"（依据「{basis}」）、退 {lib.ATTENTION_EXIT}，放行后跑完；有几轮卡上没有负载的与不要卡的照原判法记 stalled")


CARD_IDLE_WAIT_SECONDS = 30   # card-zero-utilization-stalled：利用率是 0 的那一条至多安静等这么久收尾（卡住阈值 2 秒；弄坏开关下它一直在跑，等满判红）


def write_card_utilization(world, percent):
    """假 nvidia-smi 读卡利用率照的那两份（card-utilization-<序号>，本机两张卡的序号 0、1）都写 percent。"""
    for index in (0, 1):
        with open(os.path.join(world.world, f"card-utilization-{index}"), "w", encoding="utf-8") as handle:
            handle.write(f"{percent}\n")


def cell_card_zero_utilization_stalled(world):
    """卡上有它的进程而卡不干活（task_server.py 文件头 ①「卡上有它的计算进程」；任务 9341：显存占着 14129 MiB、利用率 0%，最后几条用例半小时不动）：
    本机两张卡，假 scope 的 CPU 一直不涨（卡住阈值 2 秒），两条都只打一行就不出声、跑到各自的放行文件为止，假 nvidia-smi 照 card-busy-pids-<名字> 报卡上有它的计算进程、
    照 card-utilization-<序号> 报卡的利用率。先把两张卡的利用率都写 0：每一轮卡上都有它的负载、利用率每一轮都是 0 的那一条（idle）满窗口判 stalled、停掉，
    说明写「利用率都是 0」；再都写 37：同一段正文的另一条（computing）照免停，服务记了两个窗口「GPU 在算，不停」、依据写利用率读到过不为 0，还在跑，放行之后跑完。
    弄坏开关 card-presence-counts-as-busy 下 idle 也免停、一直跑到等满，stall-card-load-ignored 下 computing 也停掉、记 stalled，这一格红。"""
    world.resources(local=(100, 100), cards=f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE}", local_gpu=[(0, 16384), (1, 16384)])
    write_executable(os.path.join(world.work, GPU_BUSY_QUIET_SCRIPT_NAME), GPU_BUSY_QUIET_SCRIPT)
    releases = {name: os.path.join(world.root, f"release-card-{name}") for name in ("idle", "computing")}
    listed = {name: os.path.join(world.world, f"card-busy-pids-{name}") for name in ("idle", "computing")}
    problems, idle_done, computing, computing_done = [], None, None, None
    write_card_utilization(world, 0)
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01",
                                  "TASK_SERVER_FAKE_SCOPE_DIRECTORY": idle_cpu_scope(world), "TASK_SERVER_NVIDIA_SMI": os.path.join(world.bin, "fake-nvidia-smi")})
        idle = world.submit(["bash", GPU_BUSY_QUIET_SCRIPT_NAME, "busy", listed["idle"], releases["idle"]])
        try:
            idle_done = world.finished(idle, timeout=CARD_IDLE_WAIT_SECONDS)
        except SelftestFailure as failure:
            problems.append(f"卡上有它的计算进程、利用率每一轮都是 0、日志与 CPU 都不涨的应当满窗口判 stalled、停掉，实际安静 {CARD_IDLE_WAIT_SECONDS} 秒没收尾（{str(failure)[:300]}）")
        if idle_done is not None:
            note = first_attempt(idle_done).get("stall_note") or ""
            if not idle_done["task"].get("gpu") or idle_done["task"].get("outcome") != "stalled" or "利用率都是 0" not in note:
                problems.append(f"卡上有它的计算进程、利用率每一轮都是 0 的应当认成要卡、记 stalled、说明写「利用率都是 0」，"
                                f"实际认成{'要卡' if idle_done['task'].get('gpu') else '不要卡'}、{describe(idle_done)}、说明「{note}」")
        open(releases["idle"], "w").close()
        write_card_utilization(world, 37)
        computing = world.submit(["bash", GPU_BUSY_QUIET_SCRIPT_NAME, "busy", listed["computing"], releases["computing"]])
        log_path = os.path.join(world.root, f"server-{world.server_logs}.log")
        spared_head = f"任务 {computing} 第 1 次尝试的日志 "

        def spared_twice(summary):   # 服务记了两个窗口「GPU 在算，不停」（依据写利用率读到过不为 0），或已收尾（收尾了下面判红）
            with contextlib.suppress(OSError):
                with open(log_path, encoding="utf-8", errors="replace") as handle:
                    if sum(1 for line in handle if spared_head in line and "利用率读到过不为 0" in line and "GPU 在算，不停" in line) >= 2:
                        return True
            return summary["task"].get("state") == "finished"
        during = world.wait_for(computing, spared_twice, "记两个窗口「GPU 在算，不停」")
        if during["task"].get("state") != "running" or not [attempt for attempt in during["attempts"] if attempt.get("state") == "running"]:
            problems.append(f"卡上有它的计算进程、利用率 37% 的应当照免停、两个窗口过后还在跑，实际 {describe(during)}、状态 {during['task'].get('state')}")
        open(releases["computing"], "w").close()
        computing_done = world.finished(computing)
    finally:
        for release in releases.values():
            open(release, "w").close()
    if computing_done is not None and computing_done["task"].get("outcome") != "succeeded":
        problems.append(f"利用率不为 0 的那一条放行之后应当跑完，实际 {describe(computing_done)}")
    if problems:
        raise SelftestFailure("卡上有进程而利用率是 0：" + "；".join(problems), world.server_log_lines())
    return ("卡上有它的计算进程、利用率每一轮都是 0、日志与 CPU 都不涨的满窗口记 stalled（说明写利用率都是 0）；利用率 37% 的照免停、放行后跑完")


CARD_IDLE_POLLING_USEC_PER_TICK = 10000    # card-idle-polling-stalled：假 scope 每 0.2 秒涨 0.01 秒，命令自己的 CPU 平均 0.05 个核（不分卡的门槛 0.01 之上、占卡不用的门槛 1 之下）
CARD_IDLE_COMPUTING_USEC_PER_TICK = 400000  # card-idle-cpu-computing-not-stalled：每 0.2 秒涨 0.4 秒，平均 2 个核（在 CPU 上算）


def card_idle_world(world):
    """占卡不用那两格的世界：本机两张卡、两张卡的利用率都写 0、写好要卡的跑法（GPU_BUSY_QUIET_SCRIPT）。→ (卡上计算进程的名单文件, 放行文件)。"""
    world.resources(local=(100, 100), cards=f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE}", local_gpu=[(0, 16384), (1, 16384)])
    write_executable(os.path.join(world.work, GPU_BUSY_QUIET_SCRIPT_NAME), GPU_BUSY_QUIET_SCRIPT)
    write_card_utilization(world, 0)
    return os.path.join(world.world, "card-busy-pids-idle"), os.path.join(world.root, "release-card-idle")


def cell_card_idle_polling_stalled(world):
    """占着卡不干活、CPU 上轮询死等（task_server.py 文件头 ①「卡上有它的计算进程」的占卡不用门槛；任务 9341：测试进程轮询死等约 1% 一个核、显存 14129 MiB、利用率 0%）：
    卡上有它的计算进程、利用率每一轮都是 0，假 scope 的 CPU 平均 0.05 个核（不分卡的 0.01 门槛之上），日志不涨（卡住阈值 2 秒）：满窗口判 stalled、停掉，
    说明写三样读数（日志多久没涨、「利用率都是 0」、「命令自己的 CPU」几个核）。弄坏开关 card-idle-stall-threshold-ignored 下照 0.01 的门槛当在算、一直跑到等满，这一格红。"""
    listed, release = card_idle_world(world)
    scope, stop = start_burning_scope(world, "polling-scope", {}, usec_per_tick=CARD_IDLE_POLLING_USEC_PER_TICK)
    problems, done = [], None
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01",
                                  "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope, "TASK_SERVER_NVIDIA_SMI": os.path.join(world.bin, "fake-nvidia-smi")})
        task_id = world.submit(["bash", GPU_BUSY_QUIET_SCRIPT_NAME, "busy", listed, release])
        try:
            done = world.finished(task_id, timeout=CARD_IDLE_WAIT_SECONDS)
        except SelftestFailure as failure:
            problems.append(f"卡上有它的进程、利用率每一轮都是 0、命令自己的 CPU 平均 0.05 个核的应当满窗口判 stalled、停掉，实际安静 {CARD_IDLE_WAIT_SECONDS} 秒没收尾（{str(failure)[:300]}）")
        if done is not None:
            note = first_attempt(done).get("stall_note") or ""
            if done["task"].get("outcome") != "stalled" or not all(text in note for text in ("秒没涨", "利用率都是 0", "命令自己的 CPU", "个核")):
                problems.append(f"占着卡不干活、CPU 上轮询死等的应当记 stalled、说明写日志多久没涨、「利用率都是 0」与「命令自己的 CPU」几个核，实际 {describe(done)}、说明「{note}」")
    finally:
        open(release, "w").close()
        stop.set()
    if problems:
        raise SelftestFailure("占卡不用的判卡住：" + "；".join(problems), world.server_log_lines())
    return "卡上有它的进程、利用率每一轮都是 0、命令自己的 CPU 平均 0.05 个核的满窗口记 stalled，说明写三样读数"


def cell_card_idle_cpu_computing_not_stalled(world):
    """占着卡、利用率 0 而命令在 CPU 上算（任务 9520：利用率 0、日志不涨、命令 CPU 13.8 核）：假 scope 的 CPU 平均 2 个核（占卡不用的门槛 1 之上），
    服务记两个窗口「在 CPU 上算，不停」、还在跑，放行之后跑完。"""
    listed, release = card_idle_world(world)
    scope, stop = start_burning_scope(world, "computing-scope", {}, usec_per_tick=CARD_IDLE_COMPUTING_USEC_PER_TICK)
    problems, done = [], None
    try:
        world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01",
                                  "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope, "TASK_SERVER_NVIDIA_SMI": os.path.join(world.bin, "fake-nvidia-smi")})
        task_id = world.submit(["bash", GPU_BUSY_QUIET_SCRIPT_NAME, "busy", listed, release])
        spared_head = f"任务 {task_id} 第 1 次尝试的日志 "

        def spared_twice(summary):   # 服务记了两个窗口「在 CPU 上算，不停」，或已收尾（收尾了下面判红）
            return (sum(1 for line in server_output_text(world).splitlines() if spared_head in line and "在 CPU 上算，不停" in line) >= 2
                    or summary["task"].get("state") == "finished")
        during = world.wait_for(task_id, spared_twice, "记两个窗口「在 CPU 上算，不停」")
        if during["task"].get("state") != "running":
            problems.append(f"利用率 0 而命令 CPU 平均 2 个核的应当照在算、两个窗口过后还在跑，实际 {describe(during)}、状态 {during['task'].get('state')}")
        open(release, "w").close()
        done = world.finished(task_id)
    finally:
        open(release, "w").close()
        stop.set()
    if done is not None and done["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("占卡不用而 CPU 在算：" + "；".join(problems), world.server_log_lines())
    return "利用率 0、命令 CPU 平均 2 个核：服务记两个窗口在 CPU 上算、不停，放行后跑完"


PEER_HEALTH_HEAD = "task-server：服务健康 NFS 共享判不过"
PEER_SLOW_LOG_TEXT = "第二台慢（超时、复查通了）"


def cell_peer_health_slow_rechecked(world):
    """第二台一次超时先当慢（task_server_lib.py 文件头「NFS 共享」的挂载；三个会话各报过一次 30 秒超时的「用不了」）：ssh 时限压到 1 秒、复查前隔 0.2 秒；
    假 ssh 头一回复核挂载那一句睡着（fake-ssh-hang-once），复查照常回 nfs：服务日志记一行「第二台慢（超时、复查通了）」，会话收不到服务健康；
    再让每一回都睡着（fake-ssh-hang）：复查也超时，会话收到恰好一条「服务健康 NFS 共享判不过」，正文写「复查又超时」。
    弄坏开关 peer-health-single-timeout-reported 下头一回超时就报、服务日志没有那一行，这一格红。"""
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-peer-health")
    once, hang = os.path.join(world.world, "fake-ssh-hang-once"), os.path.join(world.world, "fake-ssh-hang")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_NFS_SSH_SECONDS": "1", "TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS": "0.2"})
    problems = []
    try:
        task_id = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "peer-health", release], extra=inbox.environment())
        world.running(task_id)
        open(once, "w").close()
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while PEER_SLOW_LOG_TEXT not in server_output_text(world) and patience.waiting():
            time.sleep(POLL_SECONDS)
        time.sleep(INBOX_QUIET_SECONDS)
        if PEER_SLOW_LOG_TEXT not in server_output_text(world) or not os.path.exists(once + ".taken"):
            problems.append(f"一次超时、复查回了的应当在服务日志记一行「{PEER_SLOW_LOG_TEXT}」，实际{'没睡着那一回' if not os.path.exists(once + '.taken') else '没有那一行'}")
        early = inbox.headed(PEER_HEALTH_HEAD)
        if early:
            problems.append(f"一次超时、复查回了的不该推服务健康，实际收到 {len(early)} 条：「{early[0]['text'][:300]}」")
        open(hang, "w").close()
        found = inbox.wait_for(world, PEER_HEALTH_HEAD)
        problems += inbox_problems(found, PEER_HEALTH_HEAD, "两次都不通")
        if found and "复查又超时" not in found[0]["text"]:
            problems.append(f"两次都不通的那一条应当写「复查又超时」（连不上），实际「{found[0]['text'][:300]}」")
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(hang)
        open(release, "w").close()
    if problems:
        raise SelftestFailure("第二台一次超时先当慢：" + "；".join(problems), world.server_log_lines())
    return f"一次超时、复查回了：服务日志记「{PEER_SLOW_LOG_TEXT}」、不推；两次都不通：推恰好一条服务健康，写复查又超时"


CAP_HIT_STAT_KEY = "selftest-cap-hit-stat"
CAP_HIT_STAT_TEXTS = ("anon 900M", "file 9728M", "file_dirty 8192M", "file_writeback 256M", "oom_kill 1", "大头是页缓存")


def cell_cap_hit_stat_in_note(world):
    """撞上限那一刻的内存明细（run-with-memory-cap.sh 文件头「撞上限那一刻的内存明细」；任务 9776 推的是页缓存或脏页撑满了 scope）：假包装每次退 250、先打一行
    RUN_WITH_MEMORY_CAP_OOM_STAT（anon 900M、file 9728M、脏页 8192M、正在写回 256M）：总上限 50G、记录峰值 20G，两次撞上限到顶记 memory-cap-at-maximum，
    结局说明与 wait 那一行理由都写这几样读数与「大头是页缓存」。弄坏开关 cap-hit-stat-unreported 下不抄，这一格红。"""
    world.resources(local=(55, 50))
    world.seed_peak(CAP_HIT_STAT_KEY, 20 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,250", "fake-wrapper-oom-stat"], extra={"RUN_WITH_MEMORY_CAP_KEY": CAP_HIT_STAT_KEY})
    summary = world.finished(task_id)
    note = summary["task"].get("note") or ""
    completed = world.client(["wait", str(task_id)])
    reason = next((line for line in completed.stdout.splitlines() if line.startswith(lib.REASON_MARK)), "")
    problems = []
    if summary["task"].get("outcome") != "memory-cap-at-maximum":
        problems.append(f"两次撞上限应当到顶记 memory-cap-at-maximum，实际 {describe(summary)}")
    for where, text in (("结局说明", note), ("wait 的理由行", reason)):
        missing = [wanted for wanted in CAP_HIT_STAT_TEXTS if wanted not in text]
        if missing:
            problems.append(f"{where}应当写撞上限那一刻的内存明细（缺 {missing}），实际「{text[:500]}」")
    if problems:
        raise SelftestFailure("撞上限那一刻的内存明细：" + "；".join(problems), world.server_log_lines())
    return "撞上限那一刻的内存明细抄进了结局说明与 wait 的理由行（anon、file、脏页、正在写回、oom_kill，大头是页缓存）"


RUN_LIMIT_SAMPLE_SECONDS = "3"   # 跑满两小时那几格：界压到 3 秒（TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS），不真等两小时
RUN_LIMIT_ACROSS_SECONDS = "3"   # over-two-hours-across-cap-hits：前两次各跑 2.5 秒撞上限、第三次跑 2 秒，界 3 秒（哪一次都不满，三次加起来满）
LONG_TICKS_SCRIPT = 'for round in $(seq 1 30); do echo "tick $round"; sleep 0.2; done'   # 跑 6 秒、每 0.2 秒打一行，比 3 秒的界长
ACROSS_CAP_HITS_SCRIPT = ('if [[ "$TASK_SERVER_ATTEMPT" -lt 3 ]]; then echo "attempt $TASK_SERVER_ATTEMPT"; sleep 2.5; exit 250; fi; '
                          'for round in $(seq 1 10); do echo "tick $round"; sleep 0.2; done')
SCOPE_DAILY = "种类=items; 跑什么=自证造的一条; 范围=1 条; 为什么=自证：日常开发跑一阵子"
SCOPE_COMMIT = "种类=whole; 跑什么=自证造的门禁; 范围=全部; 为什么=自证：提交前跑整轮"
LONG_RUN_WHY_SAMPLE = "自证：这一条点的就是要跑的全部范围，确实要跑这么久"
FAKE_COMMIT_GATE = ".claude/gate.d/lkmm.sh"   # 假仓里的一道提交时门禁（lib.COMMIT_GATE_ENTRIES 之一）：跑 LONG_TICKS_SCRIPT
FAKE_MERGE_ENTRY = "scripts/gate/merge-and-verify.sh"


def write_fake_entry(world, relative):
    """假仓根下 relative 那份脚本：正文是 LONG_TICKS_SCRIPT（参数不看）。"""
    path = os.path.join(world.repo, relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    write_executable(path, "#!/usr/bin/env bash\n" + LONG_TICKS_SCRIPT + "\n")


def card_world(world):
    """要卡的入口（merge-and-verify.sh、crash-point-replay.sh 照名字认成要卡）那几格的世界：本机两张卡。"""
    world.resources(local=(100, 100), cards=f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE}", local_gpu=[(0, 16384), (1, 16384)])


def bounced_problems(summary, what):
    """跑满两小时被打回的那一条：结局 over-two-hours、退 242、说明写累计用时。→ 问题表。"""
    note = summary["task"].get("note") or ""
    if summary["task"].get("outcome") != "over-two-hours" or summary["task"].get("exit_code") != 242 or "累计跑了" not in note:
        return [f"{what}应当累计满界被停、记 over-two-hours、退 242、说明写累计用时，实际 {describe(summary)}、说明「{note[:300]}」"]
    return []


def requeue(world, task_id, items, tasks, why=LONG_RUN_WHY_SAMPLE, session=SESSION_A):
    """task requeue <号> --long-run-confirmed --items <N> --tasks <M> --why <…>：交 CompletedProcess。"""
    return world.client(["requeue", str(task_id), "--long-run-confirmed", "--items", str(items), "--tasks", str(tasks), "--why", why], session=session)


def requeued_id(completed):
    found = re.search(rf"{lib.REQUEUE_MARK} task=(\d+)", completed.stdout)
    return int(found.group(1)) if found else None


def cell_over_two_hours_bounced(world):
    """跑满两小时打回（task_server_lib.py 文件头「错误类型」over-two-hours；界压到 3 秒）：日常开发的一条（范围声明的「为什么」没写提交前）跑 6 秒，
    满 3 秒被停、记 over-two-hours、退 242；推给会话的那一条写命令、范围声明、累计与各次用时、日志末几行，出路写点名范围与 requeue 那一整行。
    弄坏开关 over-two-hours-not-bounced 下跑完 succeeded，这一格红。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", LONG_TICKS_SCRIPT], extra=dict(inbox.environment(), SINGLEFS_RUN_SCOPE=SCOPE_DAILY))
    summary = world.finished(task_id)
    problems = bounced_problems(summary, "日常开发的一条跑 6 秒，")
    head = f"task-server：任务 {task_id} over-two-hours"
    found = inbox.wait_for(world, head)
    text = found[0]["text"] if found else ""
    wanted = ("累计跑了", "各次 第 1 次", "命令：", "范围声明：", "自证：日常开发跑一阵子", "日志末几行：", "tick", "点名范围",
              f"task-server/task requeue {task_id} --long-run-confirmed --items 0 --tasks 0 --why")
    missing = [piece for piece in wanted if piece not in text]
    if not found or missing:
        problems.append(f"推给会话的那一条应当写命令、范围声明、累计与各次用时、日志末几行与两条出路（缺 {missing}），实际「{text[:600]}」")
    if problems:
        raise SelftestFailure("跑满两小时打回：" + "；".join(problems), world.server_log_lines())
    return f"日常开发的一条累计满 {RUN_LIMIT_SAMPLE_SECONDS} 秒被停、记 over-two-hours、退 242，推给会话的写全了命令、范围声明、用时、日志末几行与 requeue 那一行"


def cell_over_two_hours_across_cap_hits(world):
    """撞了内存上限重来的计时归零（用户：「撞了内存上限 后重跑的任务两小时计时归零。 不限定要三次。」）：界 3 秒，前两次各跑 2.5 秒退 250（撞上限、乘 1.5 重排），
    第三次跑 2 秒：三次加起来 7 秒过了界，而每一次撞上限之后都从零算、哪一次都不满 3 秒，跑完 succeeded，三次结局是两次 memory-cap-hit、一次 succeeded。
    弄坏开关 over-two-hours-cap-hits-counted 下撞上限的那几次也累加，第二次跑到 0.5 秒就满界被停、记 over-two-hours，这一格红。"""
    world.resources(local=(100, 100))
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_ACROSS_SECONDS})
    task_id = world.submit(["bash", "-c", ACROSS_CAP_HITS_SCRIPT], extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY})
    summary = world.finished(task_id)
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    if summary["task"].get("outcome") != "succeeded" or outcomes != ["memory-cap-hit", "memory-cap-hit", "succeeded"]:
        raise SelftestFailure(f"撞上限重来的计时归零：两次撞上限各 2.5 秒、第三次 2 秒应当跑完 succeeded（各次 memory-cap-hit、memory-cap-hit、succeeded），"
                              f"实际 {describe(summary)}、各次 {outcomes}", world.server_log_lines())
    return f"两次撞上限各跑 2.5 秒、第三次跑 2 秒，三次加起来过了 {RUN_LIMIT_ACROSS_SECONDS} 秒的界，每次撞上限后归零、照跑完 succeeded"


def cell_over_two_hours_commit_gate_exempt(world):
    """提交时的门禁豁免：假仓里的 .claude/gate.d/lkmm.sh 不带 --check、范围声明的「为什么」写了「提交前」，跑 6 秒过了 3 秒的界照跑完 succeeded。"""
    world.resources(local=(30, 30))
    write_fake_entry(world, FAKE_COMMIT_GATE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    summary = world.finished(world.submit(["bash", FAKE_COMMIT_GATE], extra={"SINGLEFS_RUN_SCOPE": SCOPE_COMMIT}, cwd=world.repo))
    if summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"提交前跑的提交时门禁（不带 --check）应当豁免、跑完 succeeded，实际 {describe(summary)}", world.server_log_lines())
    return "提交前跑的提交时门禁过了界照跑完"


def cell_over_two_hours_check_not_exempt(world):
    """豁免要三样都对（task_server_lib.py 的 commit_gate_exempt；任务 9604 那种带 --check 的门禁跑了 6.2 小时）：同一道门禁带 --check（「为什么」写了提交前）的、
    不带 --check 而「为什么」没写提交前的，都满 3 秒被停、记 over-two-hours。弄坏开关 over-two-hours-commit-exempt-by-command-only 下两条都豁免、跑完，这一格红。"""
    world.resources(local=(30, 30))
    write_fake_entry(world, FAKE_COMMIT_GATE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    checked = world.submit(["bash", FAKE_COMMIT_GATE, "--check", "lkmm-cell"], extra={"SINGLEFS_RUN_SCOPE": SCOPE_COMMIT}, cwd=world.repo)
    daily = world.submit(["bash", FAKE_COMMIT_GATE], extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY}, cwd=world.repo)
    problems = bounced_problems(world.finished(checked), "带 --check 的门禁，") + bounced_problems(world.finished(daily), "「为什么」没写提交前的门禁，")
    if problems:
        raise SelftestFailure("提交时门禁的豁免：" + "；".join(problems), world.server_log_lines())
    return "带 --check 的门禁与「为什么」没写提交前的门禁都满界被停"


def cell_over_two_hours_merge_and_verify_not_exempt(world):
    """merge-and-verify 不豁免（给了补丁目录、「为什么」写了提交前也一样）：假仓里的 scripts/gate/merge-and-verify.sh 跑 6 秒，满 3 秒被停、记 over-two-hours。"""
    card_world(world)
    write_fake_entry(world, FAKE_MERGE_ENTRY)
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    summary = world.finished(world.submit(["bash", FAKE_MERGE_ENTRY, "selftest-patch-directory"], extra={"SINGLEFS_RUN_SCOPE": SCOPE_COMMIT}, cwd=world.repo))
    problems = bounced_problems(summary, "merge-and-verify，")
    if problems:
        raise SelftestFailure("merge-and-verify 不豁免：" + "；".join(problems), world.server_log_lines())
    return "merge-and-verify 满界被停、记 over-two-hours"


def cell_requeue_goes_to_tail(world):
    """requeue 排在队尾（task_server.py 文件头 ⑦「跑满两小时」）：本机只放得下一条（6G、每条记 4G），A 跑满 3 秒被打回；hold 住余量交 B（号比 A 大），
    再 requeue A（新号 C 比 B 大）；放开余量：B 先起、跑完，C 才起。弄坏开关 requeue-keeps-original-position 下原号 A 重开回原位、比 B 先起，这一格红。"""
    world.resources(local=(6, 6))
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    bounced = world.submit(["bash", "-c", LONG_TICKS_SCRIPT], extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY})
    problems = bounced_problems(world.finished(bounced), "A，")
    world.resources(local=(6, 6), hold=True)
    earlier = world.submit(["bash", "-c", "sleep 1; exit 0"], extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY})
    completed = requeue(world, bounced, 0, 1)
    fresh = requeued_id(completed)
    world.resources(local=(6, 6))
    if completed.returncode != 0 or fresh is None:
        raise SelftestFailure(f"requeue A 应当退 0、打 {lib.REQUEUE_MARK} task=<新号>，实际退 {completed.returncode}：「{completed.stdout.strip()[-300:]}」「{completed.stderr.strip()[-300:]}」",
                              world.server_log_lines())
    b, c = world.finished(earlier), world.finished(fresh)
    b_started, c_started = float(first_attempt(b).get("started_at") or 0), float((c["attempts"] or [{}])[-1].get("started_at") or 0)
    if fresh <= earlier or not b_started or not c_started or c_started < b_started:
        problems.append(f"requeue 的那一条应当新号（{fresh}）比先交的 B（{earlier}）大、B 先起它后起，实际 B 起于 {b_started}、它起于 {c_started}")
    if problems:
        raise SelftestFailure("requeue 排在队尾：" + "；".join(problems), world.server_log_lines())
    return f"A 被打回之后 requeue 成新号 {fresh}，排在先交的 B（{earlier}）后面，B 先起"


def cell_long_run_confirmed_not_bounced(world):
    """带「长跑已确认」标识的不再按两小时打回、标识打进 status 与推送（task_server.py 文件头 ⑦「跑满两小时」）：A 跑满 3 秒被打回，requeue 成 C；
    C 跑 6 秒照跑完 succeeded，task status C 与推给会话的结局那一条都有「长跑已确认（原号 A）：<why>」。
    弄坏开关 long-run-confirmed-still-bounced 下 C 照样被停，long-run-confirmed-flag-hidden 下没有那一行，这一格红。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    bounced = world.submit(["bash", "-c", LONG_TICKS_SCRIPT], extra=dict(inbox.environment(), SINGLEFS_RUN_SCOPE=SCOPE_DAILY))
    problems = bounced_problems(world.finished(bounced), "A，")
    completed = requeue(world, bounced, 0, 0)
    fresh = requeued_id(completed)
    if completed.returncode != 0 or fresh is None:
        raise SelftestFailure(f"requeue A 应当退 0、打新号，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」", world.server_log_lines())
    flag = f"长跑已确认（原号 {bounced}）：{LONG_RUN_WHY_SAMPLE}"
    shown = world.client(["status", str(fresh)])
    summary = world.finished(fresh)
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"带长跑已确认的 C 应当过了界照跑完 succeeded，实际 {describe(summary)}")
    if flag not in shown.stdout + shown.stderr:
        problems.append(f"task status C 应当有一行「{flag}」，实际「{(shown.stdout + shown.stderr).strip()[:400]}」")
    found = inbox.wait_for(world, f"task-server：任务 {fresh} succeeded")
    if not found or flag not in found[0]["text"]:
        problems.append(f"推给会话的 C 的结局那一条应当有「{flag}」，实际「{(found[0]['text'] if found else '没收到')[:400]}」")
    if problems:
        raise SelftestFailure("长跑已确认：" + "；".join(problems), world.server_log_lines())
    return f"requeue 成 {fresh} 的那一条过了界照跑完，status 与推送都有长跑已确认那一行"


def cell_requeue_refusals(world):
    """requeue 的拒（task_server.py 的 requeue）：别的会话 requeue 退 3、点名不是这个会话提交的；结局 succeeded 的号退 3、点名不是 over-two-hours；
    --items 对不上退 3、打服务算的确认单（requeue-counts-mismatch）；--why 太短退 3；对的那一回退 0，同一个号再 requeue 退 3、点名 requeue 过；
    requeue 出来的新号再 requeue 退 3。被拒的那几回原号都没 requeue（库里 requeued_as 不在）。"""
    world.resources(local=(30, 30))
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    bounced = world.submit(["bash", "-c", LONG_TICKS_SCRIPT], extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY})
    finished = world.submit(["bash", "-c", "exit 0"], extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY})
    problems = bounced_problems(world.finished(bounced), "A，")
    world.finished(finished)
    for what, completed, text in (("别的会话", requeue(world, bounced, 0, 0, session=SESSION_B), "不是这个会话提交的"),
                                  ("结局 succeeded 的号", requeue(world, finished, 0, 0), "不是 over-two-hours"),
                                  ("--items 对不上", requeue(world, bounced, 5, 0), "确认单"),
                                  ("--why 太短", requeue(world, bounced, 0, 0, why="太短"), "至少")):
        output = completed.stdout + completed.stderr
        if completed.returncode != 3 or text not in output:
            problems.append(f"{what}的 requeue 应当退 3、写「{text}」，实际退 {completed.returncode}：「{output.strip()[-300:]}」")
    if world.summary(bounced)["task"].get("requeued_as"):
        problems.append("被拒的那几回之后原号不该记 requeued_as")
    first = requeue(world, bounced, 0, 0)
    fresh = requeued_id(first)
    again = requeue(world, bounced, 0, 0)
    nested = requeue(world, fresh, 0, 1) if fresh is not None else None
    if first.returncode != 0 or fresh is None:
        problems.append(f"对的那一回应当退 0、打新号，实际退 {first.returncode}：「{first.stderr.strip()[-300:]}」")
    if again.returncode != 3 or "requeue 过" not in again.stdout + again.stderr:
        problems.append(f"同一个号再 requeue 应当退 3、写 requeue 过，实际退 {again.returncode}：「{(again.stdout + again.stderr).strip()[-300:]}」")
    if nested is not None and nested.returncode != 3:
        problems.append(f"带长跑已确认的新号再 requeue 应当退 3，实际退 {nested.returncode}：「{(nested.stdout + nested.stderr).strip()[-300:]}」")
    if fresh is not None:
        world.finished(fresh)
    if problems:
        raise SelftestFailure("requeue 的拒：" + "；".join(problems), world.server_log_lines())
    return "别的会话、结局不对、两个数对不上、--why 太短、同号二次与带标识的再 requeue 都退 3；对的那一回退 0"


WHOLE_PACKAGE_COMMANDS = (   # ⑬ 整包测试四种形态（task_server.py 文件头 ②「提交时拒」⑥）：(说什么, 命令, 在哪起)
    ("cargo test -p 不点名测试目标", ["cargo", "test", "-p", "singlefs-verification"], "work"),
    ("cargo test --lib --tests", ["cargo", "test", "-p", "singlefs-verification", "--lib", "--tests"], "work"),
    ("crash-point-replay.sh --check 三格全点", ["bash", ".claude/gate.d/crash-point-replay.sh", "--check", "crash-case-replay,feature-gated-tests,product-gpu-real-card-tests"], "repo"),
    ("不给补丁目录的 merge-and-verify.sh", ["bash", "scripts/gate/merge-and-verify.sh"], "repo"))


def submit_raw(world, command, cwd, confirmation="", scope=SCOPE_DAILY):
    """只交、不确认（交了的待确认、不进队、不跑）：→ CompletedProcess。"""
    words = ["submit", "--cwd", cwd] + (["--whole-package-confirmed", confirmation] if confirmation else []) + ["--", *command]
    return world.client(words, extra={"SINGLEFS_RUN_SCOPE": scope})


def cell_whole_package_unconfirmed_refused(world):
    """整包测试没带确认标识当场拒（task_server.py 文件头 ②「提交时拒」⑥；.claude/agent-common.md「日常开发的测试不跑整个包」）：四种形态各交一回，都退 3、
    stderr 写「整包测试没带确认标识」与 --whole-package-confirmed 那条出路；带 --workspace 的被 ⑤ 全仓编译先拒，同样退 3。
    弄坏开关 whole-package-unconfirmed-accepted 下四种都收下，这一格红。"""
    world.start_server()
    problems = []
    for what, command, where in WHOLE_PACKAGE_COMMANDS:
        completed = submit_raw(world, command, world.work if where == "work" else world.repo)
        if completed.returncode != 3 or "整包测试没带确认标识" not in completed.stderr or "--whole-package-confirmed" not in completed.stderr:
            problems.append(f"{what}没带确认标识应当退 3、stderr 写「整包测试没带确认标识」与出路，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」")
    workspace = submit_raw(world, ["cargo", "test", "--workspace"], world.work)
    if workspace.returncode != 3:
        problems.append(f"cargo test --workspace 应当退 3，实际退 {workspace.returncode}：「{workspace.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("整包测试没带确认标识：" + "；".join(problems), world.server_log_lines())
    return f"整包测试 {len(WHOLE_PACKAGE_COMMANDS)} 种形态没带确认标识都退 3，--workspace 的也退 3"


def cell_whole_package_named_accepted(world):
    """点名了测试目标的不拒（cargo test -p … --test <目标>、--lib <测试名>），提交时的门禁不拒（.claude/gate.d/crash-point-replay.sh 不带 --check、「为什么」写了提交前）：
    都照收（退 0、打任务号；只交不确认，不跑）。"""
    world.start_server()
    problems = []
    for what, command, where, scope in (("--test 点名", ["cargo", "test", "-p", "singlefs-verification", "--test", "selftest_target"], world.work, SCOPE_DAILY),
                                        ("--lib 带测试名", ["cargo", "test", "-p", "singlefs-verification", "--lib", "selftest_name"], world.work, SCOPE_DAILY),
                                        ("只带测试名", ["cargo", "test", "-p", "singlefs-verification", "selftest_name"], world.work, SCOPE_DAILY),
                                        ("--lib --tests 带测试名", ["cargo", "test", "-p", "singlefs-verification", "--lib", "--tests", "--", "--exact", "selftest_name"], world.work, SCOPE_DAILY),
                                        ("提交时的门禁", ["bash", ".claude/gate.d/crash-point-replay.sh"], world.repo, SCOPE_COMMIT)):
        completed = submit_raw(world, command, where, scope=scope)
        if completed.returncode != 0 or lib.SUBMIT_MARK not in completed.stdout:
            problems.append(f"{what}的应当照收、退 0，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("点名的与提交时的门禁不拒：" + "；".join(problems), world.server_log_lines())
    return "点名测试目标、带测试名的四种与提交时的门禁都照收"


WHOLE_PACKAGE_CONFIRMATION_TEXT = ("① 改到的文件与函数：task-server/task_server.py 的 judge_stall_stop 与 judge_run_limit\n"
                                   "② 引到它们的测试目标与函数：task_server_lib.py --selftest 的几格，grep 命中七处\n"
                                   "③ 为什么只点名②不够：自证造的整包确认，演三段都写全的一份\n")


def write_confirmation(world, name, text):
    path = os.path.join(world.root, name)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    return path


def cell_whole_package_confirmation_form(world):
    """整包确认文件只判形式（task_server_lib.py 的 whole_package_confirmation_problem）：缺 ③ 那一段的退 3、点名 ③；③ 不到 12 个字的退 3；相对路径退 3；
    三段写全的照收（退 0），task status 那一条有一行「整包已确认：<文件>」。"""
    world.start_server()
    command = ["cargo", "test", "-p", "singlefs-verification"]
    full = write_confirmation(world, "whole-package-full.md", WHOLE_PACKAGE_CONFIRMATION_TEXT)
    problems = []
    for what, path, text in (("缺 ③", write_confirmation(world, "whole-package-missing.md", WHOLE_PACKAGE_CONFIRMATION_TEXT.rsplit("③", 1)[0]), "③"),
                             ("③ 太短", write_confirmation(world, "whole-package-short.md", WHOLE_PACKAGE_CONFIRMATION_TEXT.rsplit("③", 1)[0] + "③ 太短\n"), "不到"),
                             ("相对路径", "whole-package-full.md", "不是绝对路径")):
        completed = submit_raw(world, command, world.work, confirmation=path)
        if completed.returncode != 3 or text not in completed.stderr:
            problems.append(f"确认文件{what}应当退 3、写「{text}」，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」")
    accepted = submit_raw(world, command, world.work, confirmation=full)
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", accepted.stdout)
    shown = world.client(["status", found.group(1)]) if found else None
    if accepted.returncode != 0 or shown is None or f"整包已确认：{full}" not in shown.stdout + shown.stderr:
        problems.append(f"三段写全的应当照收、status 有「整包已确认：{full}」，实际退 {accepted.returncode}、status「{(shown.stdout + shown.stderr).strip()[:300] if shown else ''}」")
    if problems:
        raise SelftestFailure("整包确认文件的形式：" + "；".join(problems), world.server_log_lines())
    return "缺 ③、③ 太短、相对路径都退 3；三段写全的照收，status 有整包已确认那一行"


def cell_whole_package_confirmed_still_bounced(world):
    """整包已确认的照样受两小时限、requeue 之后两种标识都带（task_server.py 文件头 ②「提交时拒」⑥ 与 ⑦「跑满两小时」）：不给补丁目录的假 merge-and-verify.sh
    带三段写全的确认文件交、跑 6 秒，满 3 秒被停、记 over-two-hours；requeue 成新号，新号带原号的整包确认文件与长跑已确认，status 两行都有，过了界照跑完。
    弄坏开关 whole-package-confirmed-exempts-two-hours 下头一条跑完不打回，whole-package-flag-dropped-on-requeue 下新号丢了整包已确认，这一格红。"""
    card_world(world)
    write_fake_entry(world, FAKE_MERGE_ENTRY)
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    full = write_confirmation(world, "whole-package-full.md", WHOLE_PACKAGE_CONFIRMATION_TEXT)
    completed = submit_raw(world, ["bash", FAKE_MERGE_ENTRY], world.repo, confirmation=full)
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
    if completed.returncode != 0 or not found:
        raise SelftestFailure(f"带确认文件的整包测试应当照收，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」", world.server_log_lines())
    world.confirm_submitted(completed, extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY})
    bounced = int(found.group(1))
    problems = bounced_problems(world.finished(bounced), "整包已确认的那一条，")
    again = requeue(world, bounced, 0, 0)
    fresh = requeued_id(again)
    if again.returncode != 0 or fresh is None:
        raise SelftestFailure(f"requeue 应当退 0、打新号，实际退 {again.returncode}：「{again.stderr.strip()[-300:]}」", world.server_log_lines())
    shown = world.client(["status", str(fresh)])
    text = shown.stdout + shown.stderr
    for line in (f"整包已确认：{full}", f"长跑已确认（原号 {bounced}）：{LONG_RUN_WHY_SAMPLE}"):
        if line not in text:
            problems.append(f"requeue 出来的新号 status 应当有「{line}」，实际「{text.strip()[:400]}」")
    summary = world.finished(fresh)
    if summary["task"].get("whole_package_confirmed") != full or summary["task"].get("outcome") != "succeeded":
        problems.append(f"新号应当带原号的整包确认文件、过了界照跑完，实际标识「{summary['task'].get('whole_package_confirmed')}」、{describe(summary)}")
    if problems:
        raise SelftestFailure("整包已确认照样受两小时限：" + "；".join(problems), world.server_log_lines())
    return f"整包已确认的满 {RUN_LIMIT_SAMPLE_SECONDS} 秒照样打回；requeue 成 {fresh} 两种标识都带、照跑完"


def cell_attention_card_replaced(world):
    """换卡重排（任务还在跑）：本机两张卡，0 号卡清场判坏，换 1 号卡重排；换卡之后的那一次一直在跑（假清场等放行文件），wait 打一行 kind=card-replaced、
    依据写坏卡与第几次尝试、退 ATTENTION_EXIT；任务不在结局状态，放行之后在 1 号卡上跑完。"""
    world.resources(local=(100, 100), cards=f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE}", local_gpu=[(0, 16384), (1, 16384)])
    world.start_server()
    release = os.path.join(world.root, "release-attention-card")
    task_id = world.submit(["cargo", "test", "--features", "gpu", f"fake-quiesce=card-fault-on-else-block:{LOCAL_CARD_ZERO}={release}"])
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    during = world.summary(task_id)
    problems, fields = attention_problems(completed, task_id, "card-replaced", lib.ATTENTION_EXIT)
    bad_card = LOCAL_CARD_ZERO.rsplit(":", 1)[0]
    if bad_card not in fields.get("basis", "") or "第 1 次" not in fields.get("basis", ""):
        problems.append(f"依据应当点名坏卡 {bad_card} 与「第 1 次」尝试，实际「{fields.get('basis')}」")
    if during["task"].get("state") == "finished":
        problems.append(f"报了换卡重排之后任务应当还没结局，实际 {describe(during)}")
    open(release, "w").close()
    done = world.finished(task_id)
    cards = [attempt.get("card") for attempt in done["attempts"]]
    if done["task"].get("outcome") != "succeeded" or cards != [LOCAL_CARD_ZERO, LOCAL_CARD_ONE]:
        problems.append(f"放行之后应当换 {LOCAL_CARD_ONE} 跑完，实际 {describe(done)}、各次的卡 {cards}")
    if problems:
        raise SelftestFailure("换卡重排：" + "；".join(problems), world.server_log_lines())
    return f"0 号卡清场判坏换 1 号卡重排：wait 打 {lib.ATTENTION_MARK} kind=card-replaced（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，任务不在结局，放行后在 1 号卡跑完"


def cell_attention_card_replaced_after_finish(world):
    """换卡重排已经跑成（结局记 succeeded，同 card-fault-replaced-on-another-card）才来 wait：照样叫一次——末行之前打一行 kind=card-replaced（打在 TASK_SERVER_REASON 之前），
    退码仍是任务的 0，末行是 succeeded 的 TASK_SERVER_RESULT。"""
    world.resources(local=(100, 100), cards=f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE}", local_gpu=[(0, 16384), (1, 16384)])
    world.start_server()
    task_id = world.submit(["cargo", "test", "--features", "gpu", f"fake-quiesce=card-fault-on:{LOCAL_CARD_ZERO}"])
    done = world.finished(task_id)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, fields = attention_problems(completed, task_id, "card-replaced", 0)
    lines = completed.stdout.strip().splitlines()
    marks = [index for index, line in enumerate(lines) if line.startswith(lib.ATTENTION_MARK + " ")]
    reasons = [index for index, line in enumerate(lines) if line.startswith(lib.REASON_MARK + " ")]
    if not lines or not lines[-1].startswith(f"{lib.RESULT_MARK} task={task_id} type=succeeded ") or not marks or not reasons or marks[0] > reasons[0]:
        problems.append(f"末行应当是 succeeded 的 {lib.RESULT_MARK}，提醒那一行打在 {lib.REASON_MARK} 之前，实际输出「{completed.stdout.strip()[-500:]}」")
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"换卡重排应当跑成，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("换卡重排跑成：" + "；".join(problems), world.server_log_lines())
    return f"换卡重排跑成之后才 wait：{lib.ATTENTION_MARK} kind=card-replaced 打在 {lib.REASON_MARK} 之前，退 0、末行 succeeded"


def cell_attention_cleanup_pending(world):
    """清场没收尾：派第二台的一条，第二台那一层写 pending 的清理回执（假 multi-host-run.sh 的 fake-peer-cleanup=pending）却一直不确认，服务保留这一次尝试的占用、任务不记结局；
    wait 打一行 kind=cleanup-pending、依据带回执未确认的原因、退 ATTENTION_EXIT；回执改成 confirmed 之后任务跑完。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "attention-cleanup", "fake-peer-cleanup=pending"], cwd=world.repo)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    during = world.summary(task_id)
    problems, fields = attention_problems(completed, task_id, "cleanup-pending", lib.ATTENTION_EXIT)
    attempt = (during["attempts"] or [{}])[-1]
    if "清场没收尾" not in fields.get("basis", "") or "未确认" not in fields.get("basis", ""):
        problems.append(f"依据应当写「清场没收尾」与回执未确认的原因，实际「{fields.get('basis')}」")
    if during["task"].get("state") != "running" or attempt.get("state") not in ("starting", "running") or not attempt.get("cleanup_pending"):
        problems.append(f"报了清场没收尾之后任务应当照旧在跑、这一次尝试保持在跑并记着 cleanup_pending，实际 {describe(during)}、状态 {during['task'].get('state')}、尝试 {attempt.get('state')}")
    receipt = attempt.get("peer_cleanup_file") or ""
    if receipt:
        with open(receipt + ".partial", "w", encoding="utf-8") as handle:
            json.dump({"status": "confirmed", "label": "selftest"}, handle)
        os.replace(receipt + ".partial", receipt)
    else:
        problems.append(f"尝试记录里应当有 peer_cleanup_file，实际 {attempt}")
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"回执确认之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("清场没收尾：" + "；".join(problems), world.server_log_lines())
    return f"第二台清理回执一直 pending：wait 打 {lib.ATTENTION_MARK} kind=cleanup-pending（依据「{fields.get('basis', '')[:80]}」）、退 {lib.ATTENTION_EXIT}，任务照旧在跑，回执确认后跑完"


def cell_attention_shard_advance_failed(world):
    """分片父任务推不动：自证造的错（TASK_SERVER_SELFTEST_ADVANCE_FAILS 点名的文件在时每一轮推父任务都抛）让整件停在原地（片在等放行文件）；
    wait 打一行 kind=shard-advance-failed、依据写那个错、退 ATTENTION_EXIT，父任务照旧是 sharded；撤掉那份文件、放行之后整件跑完。"""
    arm = os.path.join(world.root, "advance-fails")
    release = os.path.join(world.root, "release-attention-shard")
    open(arm, "w").close()
    world.start_server(extra={"TASK_SERVER_SELFTEST_ADVANCE_FAILS": arm})
    command = world.write_spec("spec-attention-advance.json", {"shards": [{"block": release}]})
    task_id = world.submit(command, cwd=world.repo)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    during = world.summary(task_id)
    problems, fields = attention_problems(completed, task_id, "shard-advance-failed", lib.ATTENTION_EXIT)
    if "自证造的推不动" not in fields.get("basis", ""):
        problems.append(f"依据应当写出推不动的那个错（「自证造的推不动」），实际「{fields.get('basis')}」")
    if during["task"].get("state") != "sharded":
        problems.append(f"报了推不动之后父任务应当照旧是 sharded，实际状态 {during['task'].get('state')}、{describe(during)}")
    os.remove(arm)
    open(release, "w").close()
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"撤掉造的错、放行之后整件应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("分片父任务推不动：" + "；".join(problems), world.server_log_lines())
    return f"分片父任务每一轮推都出错：wait 打 {lib.ATTENTION_MARK} kind=shard-advance-failed（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，父任务照旧 sharded，撤掉错后跑完"


def cell_attention_reported_once(world):
    """同一条任务同一种提醒、同一档只叫醒一次：排队过久（头一档 6 秒、第二档 12 秒）叫醒过一次，主 agent 重起 wait 之后头一档里不再叫（再起的 wait 这段时间一直等着），
    排满第二档才再叫一次（tier=2，依据不同）；任务一直是在排。"""
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")
    world.seed_peak("selftest-attention-once", 30 * GIBIBYTE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": str(ATTENTION_ONCE_SECONDS)})
    task_id = world.submit(["bash", "-c", "exit 0", "attention-once"], extra={lib.KEY_VARIABLE: "selftest-attention-once"}, cwd=world.repo)
    first = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, first_fields = attention_problems(first, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    second = world.client(["wait", str(task_id)], background=True)
    time.sleep(ATTENTION_ONCE_QUIET_SECONDS)
    quiet = second.poll() is None
    try:
        output, _ = world.communicate(second, ATTENTION_WAIT_SECONDS)
    except subprocess.TimeoutExpired:
        second.kill()
        output, _ = second.communicate()
    second_done = subprocess.CompletedProcess(second.args, second.returncode, output, "")
    more, second_fields = attention_problems(second_done, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    problems += more
    if first_fields.get("tier") != "1" or second_fields.get("tier") != "2" or first_fields.get("basis") == second_fields.get("basis"):
        problems.append(f"头一回应当 tier=1、再起的 wait 排满第二档才叫 tier=2 且依据不同，实际 tier {first_fields.get('tier')} 与 {second_fields.get('tier')}")
    if not quiet:
        problems.append(f"头一档叫过之后再起 wait，{ATTENTION_ONCE_QUIET_SECONDS} 秒里不该再叫，实际它当场退了（退 {second.returncode}）")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    world.finished(task_id)
    if problems:
        raise SelftestFailure("提醒只叫醒一次：" + "；".join(problems), world.server_log_lines())
    return f"头一档叫醒一次（tier=1），重起的 wait 头一档里不再叫，排满第二档才叫 tier=2（依据「{second_fields.get('basis')}」）"


ATTENTION_RESTART_SECONDS = 8   # attention-reported-survives-restart：头一档 8 秒、第二档 16 秒（头一档报过之后重启服务留得出 8 秒）


def cell_attention_reported_survives_restart(world):
    """报过的档记进库（task_server.py 文件头 ①「要处置的提醒」；会话 46 的任务 8819 那 30 片在第 3 档发过一轮，主 agent 重启服务之后同一档又发一轮）：
    排队过久（头一档 8 秒、第二档 16 秒）报过头一档（tier=1），停掉服务、照同一份库重起，再起的 wait 不再报头一档、排满第二档才报 tier=2；任务一直在排。
    弄坏开关 attention-reported-memory-only 下照改前只记在服务内存里：重起之后再起的 wait 当场又报 tier=1，这一格红。"""
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")
    world.seed_peak("selftest-attention-restart", 30 * GIBIBYTE)
    extra = {"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": str(ATTENTION_RESTART_SECONDS)}
    world.start_server(extra=extra)
    task_id = world.submit(["bash", "-c", "exit 0", "attention-restart"], extra={lib.KEY_VARIABLE: "selftest-attention-restart"}, cwd=world.repo)
    first = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, first_fields = attention_problems(first, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    world.stop_server()
    world.start_server(extra=extra)
    second = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    more, second_fields = attention_problems(second, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    problems += more
    if first_fields.get("tier") != "1" or second_fields.get("tier") != "2":
        problems.append(f"重启之前应当报 tier=1，重启之后再起的 wait 不再报头一档、排满第二档才报 tier=2，实际 tier {first_fields.get('tier')} 与 {second_fields.get('tier')}"
                        f"（重启之后那一行的依据「{second_fields.get('basis')}」）")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"第二台回来之后应当派过去跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("报过的提醒档跨重启：" + "；".join(problems), world.server_log_lines())
    return f"头一档报过（tier=1）之后重启服务，再起的 wait 不再报头一档，排满第二档才报 tier=2（依据「{second_fields.get('basis')}」）"


REASON_PENDING_QUIET_SECONDS = 3.0   # attention-queue-waits-for-reason：排满头一档（2 秒）之后再看这么久，排队原因一直判不出来的这段时间 wait 不该叫


def cell_attention_queue_waits_for_reason(world):
    """排队原因还没判出来的不报排队过久（task_server.py 文件头 ①「要处置的提醒」；会话 46 那一回重启之后头一轮的原因写的是「排队原因此刻读不到」）：
    本机放得下、排队线程照判不起（TASK_SERVER_SELFTEST_PLACEMENT_PAUSED：派活判它放得下就不写排队原因，演服务刚重启、这一轮还没判到它），
    排满头一档（2 秒）再过 3 秒，在等的 wait 一直不叫；换成本机放不下、第二台用不了，排队线程判出原因（第二台用不了）的那一轮 wait 才叫，原因不空、点名第二台。
    弄坏开关 attention-queue-reasonless-sent 下照改前照报：头一档一到 wait 就叫、原因写读不到，这一格红。"""
    world.resources(local=(100, 100))
    world.seed_peak("selftest-attention-reason", 30 * GIBIBYTE)
    paused = os.path.join(world.root, "placement-paused-reason")
    open(paused, "w").close()
    world.start_server(extra={"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": ATTENTION_QUEUE_SECONDS, "TASK_SERVER_SELFTEST_PLACEMENT_PAUSED": paused})
    task_id = world.submit(["bash", "-c", "exit 0", "attention-reason"], extra={lib.KEY_VARIABLE: "selftest-attention-reason"}, cwd=world.repo)
    waiting = world.client(["wait", str(task_id)], background=True)
    time.sleep(float(ATTENTION_QUEUE_SECONDS) + REASON_PENDING_QUIET_SECONDS)
    quiet = waiting.poll() is None
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")   # 本机放不下 45G、第二台用不了：排队线程这一轮判出原因
    try:
        output, _ = world.communicate(waiting, ATTENTION_WAIT_SECONDS)
    except subprocess.TimeoutExpired:
        waiting.kill()
        output, _ = waiting.communicate()
    completed = subprocess.CompletedProcess(waiting.args, waiting.returncode, output, "")
    problems, fields = attention_problems(completed, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    if not quiet:
        problems.append(f"排队原因判不出来的那几轮 wait 不该叫，实际它在排满头一档 {ATTENTION_QUEUE_SECONDS} 秒之后 {REASON_PENDING_QUIET_SECONDS} 秒里就退了"
                        f"（退 {waiting.returncode}，那一行的原因「{fields.get('why')}」）")
    elif "第二台" not in fields.get("why", "") or "读不到" in fields.get("why", ""):
        problems.append(f"判出原因那一轮报的原因应当点名「第二台」、不是读不到，实际「{fields.get('why')}」")
    os.remove(paused)
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"第二台回来之后应当派过去跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("排队原因没判出来不报：" + "；".join(problems), world.server_log_lines())
    return f"排队原因判不出来的 {REASON_PENDING_QUIET_SECONDS} 秒里 wait 不叫，判出原因那一轮才报（原因「{fields.get('why')}」）"


def cell_attention_type_in_header(world):
    """文件头「错误类型」与表同步：提醒的类型名、退出码、各种 kind（ATTENTION_KINDS）与那一行的标记都写在文件头；退出码不与错误类型表里的码、疑似卡住的码撞。"""
    header = lib.__doc__ or ""
    taken = {code for code, _ in lib.OUTCOME_TYPES.values() if code is not None} | set(lib.EXIT_TYPES) | {lib.SUSPECT_EXIT}
    problems = []
    if lib.ATTENTION_EXIT in taken:
        problems.append(f"提醒的退出码 {lib.ATTENTION_EXIT} 不该与错误类型表、疑似卡住的码撞")
    if not re.search(rf"{re.escape(lib.ATTENTION_TYPE)}[^\n]*\b{lib.ATTENTION_EXIT}\b", header):
        problems.append(f"文件头「错误类型」该有一行写 {lib.ATTENTION_TYPE} 与退出码 {lib.ATTENTION_EXIT}")
    missing = [name for name in (lib.ATTENTION_MARK, *lib.ATTENTION_KINDS) if name not in header]
    if missing:
        problems.append(f"文件头「错误类型」该写 {missing}")
    if problems:
        raise SelftestFailure("；".join(problems))
    return f"提醒的类型 {lib.ATTENTION_TYPE}、退出码 {lib.ATTENTION_EXIT}、{len(lib.ATTENTION_KINDS)} 种 kind 与标记都在文件头，码没和错误类型表撞"


def cell_temporary_service_wrapper_compiles(world):
    """不带 --server-binary 的临时 prune-server 包装自己要在服务给的编译目录里现编 prune-server：外层这一条按会编的算，派本机、派第二台都给编译目录
    （同一条命令改前派本机不给、派第二台由 multi-host-run.sh 照例给，两台结果不同）；带了 --server-binary 的照旧不编、不给。"""
    world.resources(local=(30, 30), peer_down="自证：先只在本机跑")
    world.start_server()

    def wrapped(marker, *options):
        return ["python3", f"scripts/prune-server/{lib.TEMPORARY_SERVICE_SCRIPT}", lib.TEMPORARY_SERVICE_FLAG, *options, "--",
                "bash", "-c", "exit 0", marker, "fake-peer-run"]

    local_bare = world.finished(world.submit(wrapped("wrapper-compiles-local"), cwd=world.repo))
    local_bound = world.finished(world.submit(wrapped("wrapper-compiles-bound", "--server-binary", "/usr/bin/true"), cwd=world.repo))
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    peer_bare = world.finished(world.submit(wrapped("wrapper-compiles-peer"), cwd=world.repo), timeout=PLACEMENT_WAIT_SECONDS)
    problems = []
    for name, summary, host, compiles in (("派本机、不带 --server-binary", local_bare, "local", True), ("派本机、带 --server-binary", local_bound, "local", False),
                                          ("派第二台、不带 --server-binary", peer_bare, "peer", True)):
        attempt = first_attempt(summary)
        build = attempt.get("build_directory") or ""
        if summary["task"].get("outcome") != "succeeded" or attempt.get("host") != host:
            problems.append(f"{name}：应当在{host}跑完，实际 {describe(summary)}、host {attempt.get('host')}")
        if bool(summary["task"].get("compiles")) != compiles or bool(build) != compiles:
            problems.append(f"{name}：compiles 应当是 {compiles}、尝试记着编译目录的应当是 {compiles}，实际 compiles {summary['task'].get('compiles')}、编译目录「{build}」")
    for name, summary, root in (("派本机", local_bare, world.builds), ("派第二台", peer_bare, world.peer_local)):
        directory = first_attempt(summary).get("build_directory") or ""
        if directory and not lib.inside(directory, root):
            problems.append(f"{name}给的编译目录应当在 {root} 下，实际 {directory}")
    if problems:
        raise SelftestFailure("临时 prune-server 包装现编：" + "；".join(problems), world.server_log_lines())
    return "不带 --server-binary 的包装派本机、派第二台都按会编的算、给编译目录；带 --server-binary 的不给"


def cell_temporary_service_own_server_compiles(world):
    """自带临时服务的两份脚本（task_server_lib.py 的 OWN_SERVER_SELFTESTS）单独的 --selftest 不带 --server-binary 与 PRUNE_SERVER_BINARY 时，自己在服务给的编译目录里
    现编 prune-server：外层这一条按会编的算，派本机、派第二台都给编译目录（同一条命令改前派本机不给、派第二台由 multi-host-run.sh 照例给，两台结果不同）；
    带了 --server-binary 的照旧不编、不给。命令前的赋值与任务环境里给了 PRUNE_SERVER_BINARY 的、--selftest-families、别的模式与别的脚本，认法是
    lib.temporary_service_builds_server 的纯函数，在这一格里直接核一张表。"""
    write_executable(os.path.join(world.repo, "scripts", "prune-server", LOAD_SIMULATION_SCRIPT), FAKE_LOAD_SIMULATION)
    world.resources(local=(30, 30), peer_down="自证：先只在本机跑")
    world.start_server()

    def own(script, mode, *options):
        return ["python3", f"scripts/prune-server/{script}", mode, *options]

    def submitted(script, *options):
        return world.submit(own(script, "--selftest", *options, "fake-peer-run"), cwd=world.repo)

    local_client = world.finished(submitted(lib.TEMPORARY_SERVICE_SCRIPT))
    local_load = world.finished(submitted(LOAD_SIMULATION_SCRIPT))
    local_bound = world.finished(submitted(lib.TEMPORARY_SERVICE_SCRIPT, "--server-binary", "/usr/bin/true"))
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    peer_client = world.finished(submitted(lib.TEMPORARY_SERVICE_SCRIPT, "on-the-peer"), timeout=PLACEMENT_WAIT_SECONDS)
    problems = []
    for name, summary, host, compiles in (("客户端自证派本机、不带 --server-binary", local_client, "local", True),
                                          ("压测脚本自证派本机、不带 --server-binary", local_load, "local", True),
                                          ("客户端自证派本机、带 --server-binary", local_bound, "local", False),
                                          ("客户端自证派第二台、不带 --server-binary", peer_client, "peer", True)):
        attempt = first_attempt(summary)
        build = attempt.get("build_directory") or ""
        if summary["task"].get("outcome") != "succeeded" or attempt.get("host") != host:
            problems.append(f"{name}：应当在{host}跑完，实际 {describe(summary)}、host {attempt.get('host')}")
        if bool(summary["task"].get("compiles")) != compiles or bool(build) != compiles:
            problems.append(f"{name}：compiles 应当是 {compiles}、尝试记着编译目录的应当是 {compiles}，实际 compiles {summary['task'].get('compiles')}、编译目录「{build}」")
    for name, summary, root in (("派本机", local_client, world.builds), ("派第二台", peer_client, world.peer_local)):
        directory = first_attempt(summary).get("build_directory") or ""
        if directory and not lib.inside(directory, root):
            problems.append(f"{name}给的编译目录应当在 {root} 下，实际 {directory}")
    binary_variable = lib.TEMPORARY_SERVICE_BINARY_VARIABLE
    table = (("客户端 --selftest", own(lib.TEMPORARY_SERVICE_SCRIPT, "--selftest"), {}, True),
             ("客户端 --selftest-families", own(lib.TEMPORARY_SERVICE_SCRIPT, "--selftest-families"), {}, True),
             ("压测脚本 --selftest", own(LOAD_SIMULATION_SCRIPT, "--selftest"), {}, True),
             ("客户端 --selftest 带 --server-binary", own(lib.TEMPORARY_SERVICE_SCRIPT, "--selftest", "--server-binary", "/usr/bin/true"), {}, False),
             ("压测脚本 --selftest 带 --server-binary", own(LOAD_SIMULATION_SCRIPT, "--selftest", "--server-binary", "/usr/bin/true"), {}, False),
             ("命令前的赋值给了 PRUNE_SERVER_BINARY", ["env", f"{binary_variable}=/usr/bin/true", *own(lib.TEMPORARY_SERVICE_SCRIPT, "--selftest")], {}, False),
             ("任务环境里给了 PRUNE_SERVER_BINARY", own(LOAD_SIMULATION_SCRIPT, "--selftest"), {binary_variable: "/usr/bin/true"}, False),
             ("压测脚本不是 --selftest（压测用法）", own(LOAD_SIMULATION_SCRIPT, "--count", "5"), {}, False),
             ("别的脚本的 --selftest", ["python3", "scripts/prune-server/other-script.py", "--selftest"], {}, False),
             ("包装不带 --server-binary（照旧）", own(lib.TEMPORARY_SERVICE_SCRIPT, lib.TEMPORARY_SERVICE_FLAG, "--", "true"), {}, True))
    for name, command, task_environment, expected in table:
        found = lib.temporary_service_builds_server(command, task_environment)
        if found != expected:
            problems.append(f"{name}：temporary_service_builds_server 应当交 {expected}，实际 {found}（{command}）")
    if problems:
        raise SelftestFailure("自带临时服务的自证现编：" + "；".join(problems), world.server_log_lines())
    return (f"客户端与压测脚本不带二进制的 --selftest 派本机、派第二台都按会编的算、给编译目录；带 --server-binary 或 PRUNE_SERVER_BINARY 的、"
            f"别的模式与别的脚本不给（纯函数表 {len(table)} 条对）")


def cell_cap_hit_rerun_from_record(world):
    """撞过上限的命令再交一遍（task_server.py 文件头 ②「内存」的有记录的照记录与 workspace_writer_headroom；这一格照真的下限 4G，不用各格小世界的 0，
    弄坏开关 recorded-cap-floored 下兜的就是它）：改主工作区的命令（不带 --skip-apply 的 merge-and-verify、hook-swap.py 带 --verify）撞了上限只有一次尝试、
    服务不重排，调用方自己再交一遍在服务看来是新任务。两条规则各给一个上限，哪个大用哪个，都不设 4G 底：
      ① 记录（近 5 次最高 × 1.5，撞过的上限也算进近 5 次最高），prediction 记 RECORD_HISTORY；
      ② 首趟给足（workspace_writer_headroom）：(这把键近 5 次最高 + 它的验证命令那把键近 5 次最高) × 1.5，比 ① 大才换，prediction 记 WORKSPACE_WRITER_HISTORY。
    每条的数（单位 MiB，向上取整到 MiB）：
      小的（merge-and-verify，记 247，没有验证命令）：头一遍 ① ② 都是 247 × 1.5 = 371，record，撞；再交 窗口最高是撞过的 371，371 × 1.5 = 557，record，跑完；再一遍 557，record。
      大的（merge-and-verify，记 4096）：头一遍 4096 × 1.5 = 6144，record，撞；再交 窗口最高 6144，6144 × 1.5 = 9216，record，跑完；再一遍 9216，record。
      带验证命令的（hook-swap.py 带 --verify，自己记 247、验证命令那把键记 4096）：头一遍 ② (247 + 4096) × 1.5 = 6514.5 → 6515，① 371，取 6515（workspace-writer），撞；
        再交 ② (6515 + 4096) × 1.5 = 15916.5 → 15917，① 6515 × 1.5 = 9772.5 → 9773，取大的 15917（workspace-writer），跑完；再一遍 窗口最高仍是 6515，15917，workspace-writer。
    另有一条没人改工作区的命令，记录里只有一次撞过的 247：① 247 × 1.5 = 371（record），不兜到 4096。
    弄坏开关 recorded-cap-floored 下照改前兜到 4096：小的头一遍 4096、没人改工作区的那条 4096，这一格红；
    workspace-writer-headroom-skipped 下 ② 不起：带验证命令的头一遍只照记录 371，红。"""
    world.resources(local=(30, 30))
    verify_key = PEAKS.command_key(shlex.split(WRITER_HEADROOM_VERIFY), None)
    seeded_key = "selftest-cap-hit-rerun-seeded"

    def merge_and_verify(plan):
        return ["bash", "-c", "exit 0", "x", "scripts/gate/merge-and-verify.sh", f"fake-wrapper-exits={plan}"]

    def hook_swap(plan):
        return ["bash", "-c", "exit 0", "x", "scripts/gate/hook-swap.py", "selftest-spec.json", "--scratch", world.work, "--verify", WRITER_HEADROOM_VERIFY, f"fake-wrapper-exits={plan}"]

    # 每条：名字、命令键、记下的峰值、起命令的写法、三遍各自的 (上限, prediction)：头一遍撞上限、撞过再交一遍、跑完之后再交一遍
    writers = (("小的", "selftest-cap-hit-rerun-small", MEMORY_FLOOR_SEED_BYTES, merge_and_verify,
                ((CAP_HIT_RERUN_SMALL_TEXTS[0], lib.RECORD_HISTORY), (CAP_HIT_RERUN_SMALL_TEXTS[1], lib.RECORD_HISTORY),
                 (CAP_HIT_RERUN_SMALL_TEXTS[1], lib.RECORD_HISTORY))),
               ("大的", "selftest-cap-hit-rerun-big", CAP_HIT_RERUN_BIG_SEED_BYTES, merge_and_verify,
                ((CAP_HIT_RERUN_BIG_TEXTS[0], lib.RECORD_HISTORY), (CAP_HIT_RERUN_BIG_TEXTS[1], lib.RECORD_HISTORY),
                 (CAP_HIT_RERUN_BIG_TEXTS[1], lib.RECORD_HISTORY))),
               ("带验证命令的", "selftest-cap-hit-rerun-verified", MEMORY_FLOOR_SEED_BYTES, hook_swap,
                ((CAP_HIT_RERUN_VERIFIED_TEXTS[0], lib.WORKSPACE_WRITER_HISTORY), (CAP_HIT_RERUN_VERIFIED_TEXTS[1], lib.WORKSPACE_WRITER_HISTORY),
                 (CAP_HIT_RERUN_VERIFIED_TEXTS[1], lib.WORKSPACE_WRITER_HISTORY))))
    for _, key, seed_bytes, _, _ in writers:
        world.seed_peak(key, seed_bytes)
    world.seed_peak(verify_key, CAP_HIT_RERUN_VERIFY_PEAK_BYTES)
    world.seed_attempt({"command_key": seeded_key, "outcome": "memory-cap-hit", "memory_cap": CAP_HIT_RERUN_SEEDED_HIT_BYTES, "memory_peak": None})
    world.start_server(extra={lib.MEMORY_FLOOR_VARIABLE: ""})

    problems = []
    for name, key, _, build, steps in writers:
        extra = {lib.KEY_VARIABLE: key}
        runs = [world.finished(world.submit(build(plan), extra=extra, cwd=world.repo)) for plan in ("250,run", "run", "run")]
        for step, summary, (cap_text, history), outcome in zip(("头一遍", "撞过上限的命令再交一遍", "跑完之后再交一遍"), runs, steps, ("memory-cap-hit", "succeeded", "succeeded")):
            attempt = first_attempt(summary)
            cap_bytes = int(cap_text[:-1]) * MEBIBYTE
            if (summary["task"].get("outcome") != outcome or len(summary["attempts"]) != 1 or attempt.get("memory_cap") != cap_bytes
                    or attempt.get("booked") != cap_bytes or attempt.get("prediction") != history):
                problems.append(f"{name}{step}：应当一次尝试、结局 {outcome}、上限与记账 {cap_text}（{cap_bytes} 字节）、prediction 记 {history}，"
                                f"实际上限 {attempt.get('memory_cap_text')}、记账 {attempt.get('booked')}、prediction {attempt.get('prediction')}、{describe(summary)}")
    seeded = world.finished(world.submit(["bash", "-c", "exit 0", "x", "fake-wrapper-exits=run"], extra={lib.KEY_VARIABLE: seeded_key}, cwd=world.repo))
    seeded_attempt = first_attempt(seeded)
    seeded_bytes = int(MEMORY_FLOOR_RECORDED_TEXT[:-1]) * MEBIBYTE
    if (seeded["task"].get("outcome") != "succeeded" or len(seeded["attempts"]) != 1 or seeded_attempt.get("memory_cap") != seeded_bytes
            or seeded_attempt.get("booked") != seeded_bytes or seeded_attempt.get("prediction") != lib.RECORD_HISTORY):
        problems.append(f"没人改工作区、记录里只有一次撞过的 {CAP_HIT_RERUN_SEEDED_HIT_BYTES // MEBIBYTE}M：再交一遍应当照它 × 1.5 是 {MEMORY_FLOOR_RECORDED_TEXT}（{seeded_bytes} 字节，"
                        f"不兜到 4G）、prediction 记 {lib.RECORD_HISTORY}、跑完，实际上限 {seeded_attempt.get('memory_cap_text')}、记账 {seeded_attempt.get('booked')}、"
                        f"prediction {seeded_attempt.get('prediction')}、{describe(seeded)}")
    if problems:
        raise SelftestFailure("撞过上限再交一遍照记录与首趟给足里大的、不设 4G 底：" + "；".join(problems), world.server_log_lines())
    return (f"改工作区的命令撞过上限再交一遍，两条规则取大、不设 4G 底：小的头一遍 {CAP_HIT_RERUN_SMALL_TEXTS[0]}、再交 {CAP_HIT_RERUN_SMALL_TEXTS[1]}，大的 {CAP_HIT_RERUN_BIG_TEXTS[0]}、"
            f"{CAP_HIT_RERUN_BIG_TEXTS[1]}，带验证命令的 {CAP_HIT_RERUN_VERIFIED_TEXTS[0]}、{CAP_HIT_RERUN_VERIFIED_TEXTS[1]}（首趟给足比记录大，prediction 记 {lib.WORKSPACE_WRITER_HISTORY}）；"
            f"跑完之后再交一遍照记录与首趟给足取大；记录里只有一次撞过的 {CAP_HIT_RERUN_SEEDED_HIT_BYTES // MEBIBYTE}M 的（没人改工作区）照记录 {MEMORY_FLOOR_RECORDED_TEXT}")


def cell_slow_world_patient_wait(world):
    """慢世界（上面常量处「等待的判法」）：一条命令每 SLOW_WORLD_STEP_SECONDS 秒往日志与世界目录各写一笔、共 SLOW_WORLD_STEPS 步，总共跑得比安静时限
    SLOW_WORLD_BASE_SECONDS 久；等它收尾的 wait_for 与客户端的 wait 都拿这个安静时限等：世界一直在变，应当等到收尾、判绿。另一头：收尾之后世界不再变，
    等一个永远不成立的条件，应当安静满时限就判红（不是拖到绝对上限）。弄坏开关 wait-limit-fixed 下等待退回写死的时限，前一头在安静时限处判红。"""
    world.start_server()
    beat = os.path.join(world.world, "slow-beat")
    script = f'for step in $(seq 1 {SLOW_WORLD_STEPS}); do echo "slow step $step"; echo x >> "$1"; sleep {SLOW_WORLD_STEP_SECONDS}; done; exit 0'
    problems = []

    def slow_task(name):
        return world.submit(["bash", "-c", script, name, beat])

    first_id = slow_task("slow-world-first")
    began = time.monotonic()
    try:
        first = world.wait_for(first_id, lambda found: found["task"].get("state") == "finished", "收尾", timeout=SLOW_WORLD_BASE_SECONDS)
        took = time.monotonic() - began
        if first["task"].get("outcome") != "succeeded" or took <= SLOW_WORLD_BASE_SECONDS:
            problems.append(f"慢世界里 wait_for 应当等到收尾（命令跑 {SLOW_WORLD_STEPS * SLOW_WORLD_STEP_SECONDS} 秒、比安静时限 {SLOW_WORLD_BASE_SECONDS:.0f} 秒久），"
                            f"实际 {describe(first)}、等了 {took:.1f} 秒")
    except SelftestFailure as failure:
        problems.append(f"慢世界里 wait_for 应当一直等到收尾（世界一直在变），实际在安静时限处判了红：{failure}")
    second_id = slow_task("slow-world-second")
    began = time.monotonic()
    try:
        completed = world.client(["wait", str(second_id)], timeout=SLOW_WORLD_BASE_SECONDS)
        took = time.monotonic() - began
        if completed.returncode != 0 or took <= SLOW_WORLD_BASE_SECONDS:
            problems.append(f"慢世界里客户端 wait 应当等到收尾、退 0，实际退 {completed.returncode}、等了 {took:.1f} 秒")
    except SelftestFailure as failure:
        problems.append(f"慢世界里客户端 wait 应当一直等到收尾（世界一直在变），实际在安静时限处判了红：{failure}")
    world.finished(second_id)   # 判红的话命令还在跑，等它收尾再测安静的那一头（这一句用平常的安静时限）
    began = time.monotonic()
    ceiling = SLOW_WORLD_BASE_SECONDS * WAIT_CEILING_FACTOR
    try:
        world.wait_for(second_id, lambda found: False, "永远不成立", timeout=SLOW_WORLD_BASE_SECONDS)
        problems.append("收尾之后世界不再变，等一个永远不成立的条件应当判红，实际等到了")
    except SelftestFailure:
        quiet = time.monotonic() - began
        if quiet < SLOW_WORLD_BASE_SECONDS or quiet >= ceiling - 1:
            problems.append(f"世界不再变时应当安静满 {SLOW_WORLD_BASE_SECONDS:.0f} 秒就判红（绝对上限 {ceiling:.0f} 秒之前），实际等了 {quiet:.1f} 秒")
    if problems:
        raise SelftestFailure("慢世界里的等待：" + "；".join(problems), world.server_log_lines())
    return (f"命令跑 {SLOW_WORLD_STEPS * SLOW_WORLD_STEP_SECONDS} 秒、安静时限 {SLOW_WORLD_BASE_SECONDS:.0f} 秒：wait_for 与客户端 wait 都等到了收尾；"
            f"收尾之后世界不再变，{quiet:.1f} 秒就判红（绝对上限 {ceiling:.0f} 秒）")


# ── 峰值起步、重试不重编、改主工作区的命令没正常结束、一条任务要几张卡（task_server.py 文件头 ②「内存」「GPU」、①「要处置的提醒」）──
SHARD_FAMILY_FREE_GIBIBYTES = 7      # shard-siblings-share-peaks：本机空闲 7G，放得下一片 4G 或 6G、放不下两片
SHARD_FAMILY_RAISED_TEXT = "6144M"   # 兄弟片撞了 4096M 的上限：另一片第一次尝试从 4096M × 1.5 起
BUILD_FAMILY_PEAK_BYTES = 8 * GIBIBYTE        # build-family-*：同族那一条跑成量到的峰值
BUILD_FAMILY_RAISED_TEXT = "12288M"           # 8G × 1.5
BUILD_FAMILY_ARGUMENTS = ("-p", "selftest-family", "--release")   # 同族那一条与入口声明写的 cargo 参数
FAMILY_ARGUMENT_ENTRY = "family-argument-entry.sh"   # build-family-from-arguments：声明的包取自第三个参数的入口
FAMILY_OPTION_ENTRY = "family-option-entry.sh"       # 同一格：声明的包取自 --crate 的值的入口
BUILD_FAMILY_HIT_BYTES = 6144 * MEBIBYTE      # build-family-cap-hit-raises：同族另一条刚撞到的上限（族里没有一条跑成的）
BUILD_FAMILY_HIT_RAISED_TEXT = "9216M"        # 6144M × 1.5：新提交的第一次、重排的第 2 次都起在这一档（改前一个照 4096M、一个照 6144M）
BUILD_FAMILY_WINDOW_SMALL_BYTES = 6144 * MEBIBYTE   # build-family-peak-windowed：族里撞过高档之后五条跑成量到的峰值（× 1.5 = 9216M，比没记录的 4096M 大）
SPACED_ITEM_NAME = "带 空格的 条目"   # shard-child-key-folded-by-words：条目名里带空格（后半截不该漏进分片子任务的键里）
SPACED_ITEM_TAIL = "空格的"
PEER_CARD_ONE = f"{SELFTEST_PEER_ADDRESS}:1:8G"
PEER_CARD_TWO = f"{SELFTEST_PEER_ADDRESS}:2:8G"
CARD_FREE_MEBIBYTES = 16384
FAKE_APPLY_WRITER_PATCH = r'''#!/usr/bin/env python3
"""task-server 自证的假 apply-writer-patch.py：只认 --read-outcome <结局文件>：里面有一行以「完成<TAB>」起头的打「打全了」、退 0，
没有的逐行打「没打完」、退 3，读不了退 2（真的那一份的退出码，apply-writer-patch.py 文件头「结局文件」）。别的用法退 0、什么都不写（自证里命令本身由假包装演结局）。"""
import sys

if len(sys.argv) >= 3 and sys.argv[1] == "--read-outcome":
    try:
        with open(sys.argv[2], encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except OSError as error:
        print(f"结局文件读不了：{error}")
        sys.exit(2)
    if any(line.startswith("完成\t") for line in lines):
        print("打全了")
        sys.exit(0)
    print("没打完：" + "；".join(line.replace("\t", " ") for line in lines if line.startswith("文件\t")))
    sys.exit(3)
sys.exit(0)
'''
HALF_APPLIED_OUTCOME = "开始\t/selftest/patch\t/selftest/repo\t1\t0\n文件\tcrates/a.rs\t改\taaaa\tbbbb\n"
FULLY_APPLIED_OUTCOME = HALF_APPLIED_OUTCOME + "回读\tcrates/a.rs\tbbbb\n完成\t1\t1\n"


def attempt_card_and_host(summary):
    attempt = first_attempt(summary)
    return attempt.get("card") or "", attempt.get("host") or ""


def cell_shard_siblings_share_peaks(world):
    """同一计划共用峰值（task_server.py 的 shard_family_peak）：本机空闲 7G、一次只放得下一片；第 0 片第一次撞了 4096M 的上限，第 1 片在它后面起，
    第一次尝试就从兄弟片撞过的上限 × 1.5 起（6144M，prediction 记 shard-family），不从没记录的 4096M 爬。弄坏开关 shard-siblings-unshared 下第 1 片照 4096M 起。"""
    world.resources(local=(SHARD_FAMILY_FREE_GIBIBYTES, 30))
    command = world.write_spec("spec-shard-family.json", {"shards": [{"exits": [250, "run"]}, {}]})
    world.start_server()
    summary = world.finished(world.submit(command, cwd=world.repo))
    shards = shard_children(summary)
    opening_attempts = {index: (child["attempts"] or [{}])[0] for index, child in shards.items()}
    follower = opening_attempts.get(1, {})
    if (summary["task"].get("outcome") != "succeeded" or opening_attempts.get(0, {}).get("outcome") != "memory-cap-hit"
            or follower.get("memory_cap_text") != SHARD_FAMILY_RAISED_TEXT or follower.get("prediction") != lib.SHARD_FAMILY_HISTORY):
        raise SelftestFailure(f"第 0 片第一次撞了 {UNRECORDED_DEFAULT_PINNED_TEXT} 的上限，兄弟片第 1 片第一次尝试应当从 {SHARD_FAMILY_RAISED_TEXT} 起、prediction 记 "
                              f"{lib.SHARD_FAMILY_HISTORY}，整件成了；实际 {describe(summary)}、各片第一次 "
                              f"{ {index: (attempt.get('memory_cap_text'), attempt.get('prediction'), attempt.get('outcome')) for index, attempt in opening_attempts.items()} }",
                              world.server_log_lines())
    return f"兄弟片撞了 {UNRECORDED_DEFAULT_PINNED_TEXT} 之后，第 1 片第一次尝试从 {SHARD_FAMILY_RAISED_TEXT} 起（{lib.SHARD_FAMILY_HISTORY}）"


def run_build_family_seed(world, extra_arguments=("--lib",)):
    """往这一格服务自己的库里记同族（cargo test -p selftest-family --release 接 extra_arguments 那一族）一条跑成的尝试，编译那一段的峰值与 memory_peak 都是 BUILD_FAMILY_PEAK_BYTES
    （编译族只借编译那一段的峰值：task_server.py 的 attempt_measurement；那一栏怎么记由 family-lends-build-phase-only 格钉）；交那一族。"""
    family = lib.cargo_build_family([*BUILD_FAMILY_ARGUMENTS, *extra_arguments])
    seed_ended_attempt(world, "selftest-family-lender-" + "-".join(word.strip("-") for word in extra_arguments), SEEDED_ENDED_AT, "succeeded",
                       memory_peak=BUILD_FAMILY_PEAK_BYTES, build_family=family, build_phase_peak=BUILD_FAMILY_PEAK_BYTES)
    return family


def cell_build_family_peak_shared(world):
    """会编的 cargo 同键没记录时按编译族起步（task_server.py 的 build_family_peak）：同族（包、档、特性相同）别的命令跑成量到 8G，
    这一条（测试目标不同、包与档写的次序不同）第一次尝试从 8G × 1.5 = 12288M 起、prediction 记 build-family；特性不同、档不同的另一族照没记录的 4096M 起。
    弄坏开关 build-family-ignored 下同族那一条也照 4096M 起。"""
    world.resources(local=(100, 100))
    world.start_server()
    seeded_family = run_build_family_seed(world)
    same = world.finished(world.submit(["cargo", "test", "--release", "-p", "selftest-family", "--test", "other-target", "fake-wrapper-exits=0"], cwd=world.repo))
    other = world.finished(world.submit(["cargo", "test", "-p", "selftest-family", "--features", "extra", "--lib", "fake-wrapper-exits=0"], cwd=world.repo))
    problems = []
    attempt = first_attempt(same)
    if attempt.get("memory_cap_text") != BUILD_FAMILY_RAISED_TEXT or attempt.get("prediction") != lib.BUILD_FAMILY_HISTORY:
        problems.append(f"同族的另一条应当从 {BUILD_FAMILY_RAISED_TEXT} 起、prediction 记 {lib.BUILD_FAMILY_HISTORY}，"
                        f"实际 {attempt.get('memory_cap_text')}、{attempt.get('prediction')}（任务记的族「{same['task'].get('build_family')}」，"
                        f"同族那一条记的「{seeded_family}」）")
    attempt = first_attempt(other)
    if attempt.get("memory_cap_text") != UNRECORDED_DEFAULT_PINNED_TEXT:
        problems.append(f"特性、档不同的另一族应当照没记录的 {UNRECORDED_DEFAULT_PINNED_TEXT} 起，实际 {attempt.get('memory_cap_text')}、{attempt.get('prediction')}")
    if problems:
        raise SelftestFailure("编译族起步：" + "；".join(problems), world.server_log_lines())
    return f"同族别的命令跑成 8G：这一条从 {BUILD_FAMILY_RAISED_TEXT} 起；另一族照 {UNRECORDED_DEFAULT_PINNED_TEXT}"


def cell_build_family_declared(world):
    """入口脚本文件头声明了编哪个包的照声明归族（task_server_lib.py 的 declared_build_family）：入口写「# task-server-cargo-family: -p selftest-family --release --features gpu」，
    同族的 cargo（cargo test -p selftest-family --release --features gpu --no-run）跑成 8G 之后，这份入口第一次尝试从 12288M 起、prediction 记 build-family；
    同一份入口单独 --selftest 的不归族、照没记录的 4096M 起。
    弄坏开关 build-family-ignored 下入口也照 4096M 起。"""
    world.resources(local=(100, 100))
    with open(os.path.join(world.repo, "family-entry.sh"), "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_BUILD_FAMILY_MARK} {' '.join(BUILD_FAMILY_ARGUMENTS)} --features gpu\nexit 0\n")
    world.start_server()
    gpu_family = run_build_family_seed(world, ("--features", "gpu", "--no-run"))
    entry = world.finished(world.submit(["bash", "family-entry.sh", "run", "fake-wrapper-exits=0"], cwd=world.repo))
    selftest = world.finished(world.submit(["bash", "family-entry.sh", "--selftest", "fake-wrapper-exits=0"], cwd=world.repo))
    problems = []
    attempt = first_attempt(entry)
    if attempt.get("memory_cap_text") != BUILD_FAMILY_RAISED_TEXT or attempt.get("prediction") != lib.BUILD_FAMILY_HISTORY:
        problems.append(f"声明了编译族的入口应当照同族跑成的 8G 从 {BUILD_FAMILY_RAISED_TEXT} 起、prediction 记 {lib.BUILD_FAMILY_HISTORY}，"
                        f"实际 {attempt.get('memory_cap_text')}、{attempt.get('prediction')}（入口记的族「{entry['task'].get('build_family')}」，"
                        f"同族 cargo 记的「{gpu_family}」）")
    if first_attempt(selftest).get("memory_cap_text") != UNRECORDED_DEFAULT_PINNED_TEXT or selftest["task"].get("build_family"):
        problems.append(f"同一份入口单独 --selftest 不归族、应当照 {UNRECORDED_DEFAULT_PINNED_TEXT} 起，实际 {first_attempt(selftest).get('memory_cap_text')}、"
                        f"族「{selftest['task'].get('build_family')}」")
    if problems:
        raise SelftestFailure("入口声明的编译族：" + "；".join(problems), world.server_log_lines())
    return f"入口文件头声明了包、档、特性：照同族 cargo 跑成的 8G 从 {BUILD_FAMILY_RAISED_TEXT} 起；--selftest 不归族"


def cell_build_family_from_arguments(world):
    """编译族声明里的包取自命令参数（task_server_lib.py 的 filled_family_words）：一份入口写「# task-server-cargo-family: -p $3 --release」（证红那样，包是第三个参数），
    另一份写「-p $--crate --release」（包是 --crate 的值）。同族的 cargo（cargo test -p selftest-family --release --lib）跑成 8G 之后：
    第一份带两个不同的变异名各交一条（命令键各不同、都没有自己的记录），两条都归进那一族、第一次尝试从 12288M 起、prediction 记 build-family；
    第二份写「--crate 值」与「--crate=值」的两条同样；声明点名的参数缺了的（第一份只给两个参数、第二份没给 --crate）不归族、照没记录的 4096M 起。
    弄坏开关 family-arguments-unfilled 下声明照字面算族，带变异名的那几条照 4096M 起。"""
    world.resources(local=(100, 100))
    for name, declared in ((FAMILY_ARGUMENT_ENTRY, f"-p {lib.FAMILY_ARGUMENT_MARK}3 --release"),
                           (FAMILY_OPTION_ENTRY, f"-p {lib.FAMILY_ARGUMENT_MARK}--crate --release")):
        with open(os.path.join(world.repo, name), "w", encoding="utf-8") as handle:
            handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_BUILD_FAMILY_MARK} {declared}\nexit 0\n")
    world.start_server()
    family = run_build_family_seed(world)
    grouped = {
        "第三个参数、变异名甲": world.finished(world.submit(["bash", FAMILY_ARGUMENT_ENTRY, "--copy", "copy-of-the-tree", "selftest-family", "变异 甲：读者侧",
                                                       "fake-wrapper-exits=0"], cwd=world.repo)),
        "第三个参数、变异名乙": world.finished(world.submit(["bash", FAMILY_ARGUMENT_ENTRY, "--copy", "copy-of-the-tree", "selftest-family", "变异 乙：写者侧",
                                                       "fake-wrapper-exits=0"], cwd=world.repo)),
        "--crate 值": world.finished(world.submit(["bash", FAMILY_OPTION_ENTRY, "run", "--crate", "selftest-family", "变异 丙", "fake-wrapper-exits=0"], cwd=world.repo)),
        "--crate=值": world.finished(world.submit(["bash", FAMILY_OPTION_ENTRY, "run", "--crate=selftest-family", "变异 丁", "fake-wrapper-exits=0"], cwd=world.repo)),
    }
    ungrouped = {
        "第三个参数缺了": world.finished(world.submit(["bash", FAMILY_ARGUMENT_ENTRY, "--copy", "fake-wrapper-exits=0"], cwd=world.repo)),
        "--crate 缺了": world.finished(world.submit(["bash", FAMILY_OPTION_ENTRY, "run", "变异 戊", "fake-wrapper-exits=0"], cwd=world.repo)),
    }
    problems = []
    for what, summary in grouped.items():
        attempt = first_attempt(summary)
        if (summary["task"].get("build_family") != family or attempt.get("memory_cap_text") != BUILD_FAMILY_RAISED_TEXT
                or attempt.get("prediction") != lib.BUILD_FAMILY_HISTORY):
            problems.append(f"{what}：应当归进同族「{family}」、第一次尝试从 {BUILD_FAMILY_RAISED_TEXT} 起、prediction 记 {lib.BUILD_FAMILY_HISTORY}，"
                            f"实际族「{summary['task'].get('build_family')}」、{attempt.get('memory_cap_text')}、{attempt.get('prediction')}（命令键「{summary['task'].get('command_key')}」）")
    for what, summary in ungrouped.items():
        if summary["task"].get("build_family") or first_attempt(summary).get("memory_cap_text") != UNRECORDED_DEFAULT_PINNED_TEXT:
            problems.append(f"{what}：声明点名的参数缺了应当不归族、照 {UNRECORDED_DEFAULT_PINNED_TEXT} 起，实际族「{summary['task'].get('build_family')}」、"
                            f"{first_attempt(summary).get('memory_cap_text')}")
    if problems:
        raise SelftestFailure("编译族取自命令参数：" + "；".join(problems), world.server_log_lines())
    return (f"包取自第三个参数与 --crate 的值：{len(grouped)} 条不同变异名的都归进同族、从 {BUILD_FAMILY_RAISED_TEXT} 起；"
            f"参数缺了的 {len(ungrouped)} 条不归族、照 {UNRECORDED_DEFAULT_PINNED_TEXT}")


def family_of_package(package):
    """cargo test -p <package> --release --lib 提交时记的编译族（lib.cargo_build_family；-p 点了名，工作树目录不进族）：往库里记同族的尝试时用。"""
    return lib.cargo_build_family(["-p", package, "--release", "--lib"])


def cell_build_family_cap_hit_raises(world):
    """编译族的窗口也收撞过的上限，起点能升（task_server.py 的 build_family_peak、attempt_measurement；用户：「资源的使用记录 服务要有升降的能力 不能只升不降」）：
    同族另一条（命令键不同）刚撞了 6144M 的上限、族里没有一条跑成的。一、新提交的同族一条（键没有记录）第一次尝试从 6144M × 1.5 = 9216M 起、prediction 记 build-family；
    二、另一族的一条第 1 次照 4096M 起、停在记尝试之前，这时往库里记同族另一条撞了 6144M，第 1 次撞了 4096M，第 2 次直接 9216M、prediction 记 build-family（改前一级级爬 6144M）。
    弄坏开关 family-cap-hit-not-counted 下族里只收跑成的，一、二都照 4096M、6144M；build-family-ignored 下不看族。"""
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    world.resources(local=(40, 40))
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"start-checked:{pause}"})
    problems = []
    fresh_package, retry_package = "selftest-family-hit-fresh", "selftest-family-hit-retry"
    seed_ended_attempt(world, "selftest-family-hit-sibling-fresh", SEEDED_ENDED_AT - 10, "memory-cap-hit", memory_cap=BUILD_FAMILY_HIT_BYTES,
                       build_family=family_of_package(fresh_package), build_phase_peak=BUILD_FAMILY_HIT_BYTES)
    fresh = world.finished(world.submit(["cargo", "test", "-p", fresh_package, "--release", "--lib", "fake-wrapper-exits=0"], cwd=world.repo))
    attempt = first_attempt(fresh)
    if (fresh["task"].get("build_family") != family_of_package(fresh_package) or attempt.get("memory_cap_text") != BUILD_FAMILY_HIT_RAISED_TEXT
            or attempt.get("prediction") != lib.BUILD_FAMILY_HISTORY):
        problems.append(f"同族另一条刚撞了 {BUILD_FAMILY_HIT_BYTES // MEBIBYTE}M、族里没有跑成的：新提交的第一次应当从 {BUILD_FAMILY_HIT_RAISED_TEXT} 起、prediction 记 {lib.BUILD_FAMILY_HISTORY}，"
                        f"实际 {attempt.get('memory_cap_text')}、{attempt.get('prediction')}（任务记的族「{fresh['task'].get('build_family')}」，"
                        f"库里记的族「{family_of_package(fresh_package)}」）")
    open(os.path.join(pause, "arm-start-checked"), "w").close()
    task_id = world.submit(["cargo", "test", "-p", retry_package, "--release", "--lib", "fake-wrapper-exits=250,0"], cwd=world.repo)
    wait_path(world, os.path.join(pause, "reached-start-checked"), "第 1 次停在记尝试之前")
    seed_ended_attempt(world, "selftest-family-hit-sibling-retry", time.time(), "memory-cap-hit", memory_cap=BUILD_FAMILY_HIT_BYTES,
                       build_family=family_of_package(retry_package), build_phase_peak=BUILD_FAMILY_HIT_BYTES)
    open(os.path.join(pause, "release-start-checked"), "w").close()
    retried = world.finished(task_id)
    seen = [(attempt.get("memory_cap_text"), attempt.get("prediction"), attempt.get("outcome")) for attempt in retried["attempts"]]
    if (retried["task"].get("outcome") != "succeeded" or len(seen) != 2 or seen[0][:1] != (UNRECORDED_DEFAULT_PINNED_TEXT,) or seen[0][2] != "memory-cap-hit"
            or seen[1][:2] != (BUILD_FAMILY_HIT_RAISED_TEXT, lib.BUILD_FAMILY_HISTORY)):
        problems.append(f"第 1 次照 {UNRECORDED_DEFAULT_PINNED_TEXT} 撞了上限、这时同族另一条撞了 {BUILD_FAMILY_HIT_BYTES // MEBIBYTE}M：第 2 次应当直接起在 {BUILD_FAMILY_HIT_RAISED_TEXT}、"
                        f"prediction 记 {lib.BUILD_FAMILY_HISTORY}、跑完，实际各次（上限、prediction、结局）{seen}、{describe(retried)}")
    if problems:
        raise SelftestFailure("编译族收撞过的上限：" + "；".join(problems), world.server_log_lines())
    return (f"同族另一条撞了 {BUILD_FAMILY_HIT_BYTES // MEBIBYTE}M、族里没有跑成的：新提交的第一次从 {BUILD_FAMILY_HIT_RAISED_TEXT} 起；"
            f"另一族的一条第 1 次 {UNRECORDED_DEFAULT_PINNED_TEXT} 撞了、第 2 次直接 {BUILD_FAMILY_HIT_RAISED_TEXT}")


def cell_build_family_peak_windowed(world):
    """编译族的起点能降（task_server.py 的 build_family_peak、window_maximum；用户：「资源的使用记录 服务要有升降的能力 不能只升不降」）：同族早先撞过 13824M，
    之后 RECORD_WINDOW 次（别的命令键的）跑成的峰值都是 6144M：撞过的那一次出了窗口，新提交的同族一条（键没有记录）第一次尝试从 6144M × 1.5 = 9216M 起、
    prediction 记 build-family，不是 13824M × 1.5 = 20736M。弄坏开关 history-peak-unwindowed 下不截近几次、起在 20736M，这一格红。"""
    world.resources(local=(40, 40))
    package = "selftest-family-window"
    world.start_server()
    seed_ended_attempt(world, "selftest-family-window-hit", SEEDED_ENDED_AT - 100, "memory-cap-hit", memory_cap=HISTORY_CAP_HIT_BYTES, build_family=family_of_package(package),
                       build_phase_peak=HISTORY_CAP_HIT_BYTES)
    for offset in range(RECORD_WINDOW_PINNED):   # task_server.py 的 RECORD_WINDOW：近几次的名额全是小峰值
        seed_ended_attempt(world, f"selftest-family-window-small-{offset}", SEEDED_ENDED_AT - 50 + offset, "succeeded", memory_peak=BUILD_FAMILY_WINDOW_SMALL_BYTES,
                           build_family=family_of_package(package), build_phase_peak=BUILD_FAMILY_WINDOW_SMALL_BYTES)
    summary = world.finished(world.submit(["cargo", "test", "-p", package, "--release", "--lib", "fake-wrapper-exits=0"], cwd=world.repo))
    attempt = first_attempt(summary)
    if (summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != BUILD_FAMILY_HIT_RAISED_TEXT
            or attempt.get("prediction") != lib.BUILD_FAMILY_HISTORY):
        raise SelftestFailure(f"同族早先撞过 {HISTORY_CAP_HIT_BYTES // MEBIBYTE}M、之后 {RECORD_WINDOW_PINNED} 次跑成都是 {BUILD_FAMILY_WINDOW_SMALL_BYTES // MEBIBYTE}M：撞过的那一次出了窗口，"
                              f"新提交的第一次应当回落到 {BUILD_FAMILY_HIT_RAISED_TEXT}、prediction 记 {lib.BUILD_FAMILY_HISTORY}、跑完，实际上限 {attempt.get('memory_cap_text')}、"
                              f"prediction {attempt.get('prediction')}、{describe(summary)}（只升不降会是 {HISTORY_CAP_HIT_GROWN_TEXT}）", world.server_log_lines())
    return (f"同族早先撞过 {HISTORY_CAP_HIT_BYTES // MEBIBYTE}M、之后 {RECORD_WINDOW_PINNED} 次跑成都是 {BUILD_FAMILY_WINDOW_SMALL_BYTES // MEBIBYTE}M："
            f"新提交的第一次回落到 {BUILD_FAMILY_HIT_RAISED_TEXT}")


# ── 按历史最高起跑、打补丁脚本干跑照常重排、第二台重试接着用编译目录（task_server_lib.py 文件头「资源记录」的按历史最高起跑、「编译目录」的重试复用）──
HISTORY_CAP_HIT_BYTES = 13824 * MEBIBYTE    # history-peak-first-attempt 与 history-peak-window-raises：同键撞过的上限（任务 7349、7416 撞过的那一档）
HISTORY_CAP_HIT_GROWN_TEXT = "20736M"       # 13824M × 1.5
HISTORY_SMALL_PEAK_BYTES = GIBIBYTE         # 跑成的小峰值：history-peak-first-attempt 里近 RECORD_WINDOW 次只有它们，起点回落到 1G × 1.5
HISTORY_SMALL_PEAK_GROWN_TEXT = "1536M"
HISTORY_RETRY_SEEDED_BYTES = 9216 * MEBIBYTE   # history-peak-retry-jumps：第 1 次在跑时同键别的任务撞到的上限
HISTORY_RETRY_JUMP_TEXT = "13824M"             # 9216M × 1.5：第 2 次直接跳到这一档（改前照自己那一次 4096M × 1.5 = 6144M）


def seed_ended_attempt(world, key, ended_at, outcome, memory_peak=None, memory_cap=None, build_family="", build_phase_peak=None):
    """往这一格服务自己的库里记一次结束了的尝试，结束时刻照给的（World.seed_attempt 的结束时刻固定，排不出先后）；build_family 给了的记上那条任务的编译族，
    build_phase_peak 是编译那一段的峰值（编译族只借这一栏；撞在编译那一段的那一次服务记的是那一次的上限）。"""
    world.seeded.append(dict({"command_key": key, "outcome": outcome, "memory_peak": memory_peak, "memory_cap": memory_cap, "task_id": 0, "number": 1,
                              "host": "local", "state": "ended", "session": "", "ended_at": ended_at, "started_at": ended_at - 1,
                              "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC, "build_phase_peak": build_phase_peak},
                             **({"build_family": build_family} if build_family else {})))
    world.flush_seeded()


def cell_history_peak_first_attempt(world):
    """起点能降（task_server.py 的 key_recent_peak、history_raised_prediction；用户：「资源的使用记录 服务要有升降的能力 不能只升不降」）：同键早先撞过 13824M，
    之后 RECORD_WINDOW 次跑成的峰值都是 1G：撞过的那一次出了窗口，新提交的头一次回落到 1G × 1.5 = 1536M、prediction 记 record，跑完。
    弄坏开关 history-peak-unwindowed 下照改前全部记录里取最大、起在 20736M（只升不降），这一格红。"""
    world.resources(local=(40, 40))
    key = "selftest-history-first"
    world.start_server()
    seed_ended_attempt(world, key, SEEDED_ENDED_AT - 100, "memory-cap-hit", memory_cap=HISTORY_CAP_HIT_BYTES)
    for offset in range(5):   # task_server.py 的 RECORD_WINDOW：近几次的名额全是小峰值
        seed_ended_attempt(world, key, SEEDED_ENDED_AT - 50 + offset, "succeeded", memory_peak=HISTORY_SMALL_PEAK_BYTES)
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(summary)
    if (summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != HISTORY_SMALL_PEAK_GROWN_TEXT
            or attempt.get("prediction") != lib.RECORD_HISTORY):
        raise SelftestFailure(f"同键早先撞过 13824M、之后五次跑成都是 1G：撞过的那一次出了窗口，新提交的头一次应当回落到 {HISTORY_SMALL_PEAK_GROWN_TEXT}、"
                              f"prediction 记 {lib.RECORD_HISTORY}、跑完，实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、{describe(summary)}",
                              world.server_log_lines())
    return f"同键早先撞过 13824M、之后五次跑成都是 1G：新提交的头一次回落到 {HISTORY_SMALL_PEAK_GROWN_TEXT}"


def cell_history_peak_window_raises(world):
    """起点能升（task_server.py 的 recorded_prediction、key_recent_peak）：同键近 RECORD_WINDOW 次里有一次撞过 13824M（之后又跑成四次 1G，撞过的那一次还在窗口里），
    新提交的头一次上限 20736M、prediction 记 record，跑完。弄坏开关 cap-hit-not-counted 下撞过的那一次不进定额度、起在 1536M，这一格红。"""
    world.resources(local=(40, 40))
    key = "selftest-history-window"
    world.start_server()
    seed_ended_attempt(world, key, SEEDED_ENDED_AT - 100, "memory-cap-hit", memory_cap=HISTORY_CAP_HIT_BYTES)
    for offset in range(4):   # task_server.py 的 RECORD_WINDOW 是 5：撞过的那一次与四次小峰值都在窗口里
        seed_ended_attempt(world, key, SEEDED_ENDED_AT - 50 + offset, "succeeded", memory_peak=HISTORY_SMALL_PEAK_BYTES)
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(summary)
    if (summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != HISTORY_CAP_HIT_GROWN_TEXT
            or attempt.get("prediction") != lib.RECORD_HISTORY):
        raise SelftestFailure(f"同键近五次里撞过 13824M、另四次跑成 1G：新提交的头一次应当起在 {HISTORY_CAP_HIT_GROWN_TEXT}、prediction 记 {lib.RECORD_HISTORY}、跑完，"
                              f"实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、{describe(summary)}", world.server_log_lines())
    return f"窗口里刚撞过 13824M：新提交的头一次起在 {HISTORY_CAP_HIT_GROWN_TEXT}"


def cell_history_peak_retry_jumps(world):
    """撞上限之后的重排直接跳到这把键历史上最高的那一档（task_server.py 的 history_raised_prediction）：没记录的一条第 1 次照 4096M 起、停在记尝试之前，
    这时同键别的任务撞过 9216M（往库里记一次）；第 1 次撞了上限，第 2 次上限 13824M、prediction 记 key-history，不照自己那一次 × 1.5 的 6144M。
    弄坏开关 history-peak-ignored 下第 2 次照改前 6144M。"""
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    world.resources(local=(40, 40))
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"start-checked:{pause}"})
    key = "selftest-history-retry"
    open(os.path.join(pause, "arm-start-checked"), "w").close()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,run"], extra={lib.KEY_VARIABLE: key})
    wait_path(world, os.path.join(pause, "reached-start-checked"), "第 1 次停在记尝试之前")
    seed_ended_attempt(world, key, time.time(), "memory-cap-hit", memory_cap=HISTORY_RETRY_SEEDED_BYTES)
    open(os.path.join(pause, "release-start-checked"), "w").close()
    summary = world.finished(task_id)
    seen = [(attempt.get("memory_cap_text"), attempt.get("prediction"), attempt.get("outcome")) for attempt in summary["attempts"]]
    if (summary["task"].get("outcome") != "succeeded" or len(seen) != 2 or seen[0][:1] != (UNRECORDED_DEFAULT_PINNED_TEXT,) or seen[0][2] != "memory-cap-hit"
            or seen[1][:2] != (HISTORY_RETRY_JUMP_TEXT, lib.KEY_HISTORY_PEAK_HISTORY)):
        raise SelftestFailure(f"第 1 次照 {UNRECORDED_DEFAULT_PINNED_TEXT} 撞了上限、键里另有撞过 9216M 的记录：第 2 次应当直接起在 {HISTORY_RETRY_JUMP_TEXT}、prediction 记 "
                              f"{lib.KEY_HISTORY_PEAK_HISTORY}、跑完，实际各次（上限、prediction、结局）{seen}、{describe(summary)}", world.server_log_lines())
    return f"第 1 次 {UNRECORDED_DEFAULT_PINNED_TEXT} 撞了、键里有撞过 9216M 的：第 2 次直接 {HISTORY_RETRY_JUMP_TEXT}"


def cell_history_peak_unrecorded_default(world):
    """没有任何记录的命令照旧从没记录的 4096M 起（按历史最高起跑只往上抬；task_server.py 的 unrecorded_prediction）：prediction 记 default。
    弄坏开关 unrecorded-default-grown 下照 4096M × 1.5 起。"""
    world.resources(local=(40, 40))
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: "selftest-history-unrecorded"}))
    attempt = first_attempt(summary)
    if (summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != UNRECORDED_DEFAULT_PINNED_TEXT
            or attempt.get("prediction") != lib.UNRECORDED_HISTORY):
        raise SelftestFailure(f"没有任何记录的命令应当照 {UNRECORDED_DEFAULT_PINNED_TEXT} 起、prediction 记 {lib.UNRECORDED_HISTORY}，"
                              f"实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、{describe(summary)}", world.server_log_lines())
    return f"没有任何记录的命令照 {UNRECORDED_DEFAULT_PINNED_TEXT} 起"


def cell_writer_dry_run_retried(world):
    """改工作区的工具带只读参数的不算改主工作区（task_server.py 的 modifies_workspace、WORKSPACE_READ_ONLY_ARGUMENTS）：apply-writer-patch.py --dry-run 只在临时树上核
    （任务 7442、7444）、stage-mine.py --selftest 只跑自证，撞了上限都照常乘 1.5 重排跑完；不带只读参数的真打照旧只有一次尝试、记 memory-cap-hit 打回。
    弄坏开关 read-only-arguments-counted-as-writer 下干跑与自证也只有一次、打回。"""
    world.resources(local=(30, 30))
    world.start_server()
    dry = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-dir", "--dry-run", "fake-wrapper-exits=250,run"], cwd=world.repo))
    selftest = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/stage-mine.py", "--selftest", "fake-wrapper-exits=250,run"], cwd=world.repo))
    real = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-dir", "fake-wrapper-exits=250,run"], cwd=world.repo))
    if (dry["task"].get("outcome") != "succeeded" or len(dry["attempts"]) != 2
            or selftest["task"].get("outcome") != "succeeded" or len(selftest["attempts"]) != 2
            or real["task"].get("outcome") != "memory-cap-hit" or len(real["attempts"]) != 1):
        raise SelftestFailure(f"打补丁脚本干跑、暂存脚本自证撞上限应当乘 1.5 重排跑完（两次尝试），真打撞上限应当只有一次、记 memory-cap-hit；"
                              f"实际干跑 {describe(dry)}、自证 {describe(selftest)}、真打 {describe(real)}", world.server_log_lines())
    return "打补丁脚本干跑、暂存脚本自证撞上限照常重排跑完；真打撞上限只有一次、打回"


def cell_peer_retry_build_directory_reused(world):
    """派到第二台的会编命令撞上限重排、第 2 次还在第二台：编译目录是第 1 次那一个（task_server.py 的 start_attempt、PeerMachine.command；
    假 multi-host-run.sh 照真的那样认 --keep-build-directory 与 --reuse-build-directory）：第 1 次带 --keep-build-directory、留着；第 2 次带 --reuse-build-directory <第 1 次的标签>、
    记 build_directory_reused，假 cargo 拿到的 CARGO_TARGET_DIR 是那一份下面的 target；有了结局之后服务经 --remove-build-directory 删了它。
    弄坏开关 peer-retry-build-directory-fresh 下第二台每一次新建、跑完删（第 2 次的编译目录不是第 1 次那一个）。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下：两次都派第二台
    world.start_server()
    extra = {"PATH": f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"}
    summary = world.finished(world.submit(["cargo", "build", "fake-peer-run", "fake-wrapper-exits=250,run"], extra=extra, cwd=world.repo),
                             timeout=PLACEMENT_WAIT_SECONDS)
    attempts = summary["attempts"]
    hosts = [attempt.get("host") for attempt in attempts]
    directories = [attempt.get("build_directory") or "" for attempt in attempts]
    labels = [attempt.get("label") or "" for attempt in attempts]
    task_id = summary["task"]["id"]
    arguments = [line for line in world.calls() if line.startswith(f"peer-args task={task_id} ")]
    targets = [line.split(" target=", 1)[1].split(" args=", 1)[0] for line in world.calls() if line.startswith("cargo dir=")]
    problems = []
    if (summary["task"].get("outcome") != "succeeded" or hosts != ["peer", "peer"] or not all(directories) or directories[0] != directories[1]
            or not attempts[1].get("build_directory_reused") or os.path.basename(directories[0]) != labels[0]):
        problems.append(f"第二台先退 250 再跑：第 2 次的编译目录应当是第 1 次那一个（builds/<第 1 次的标签>）、记 build_directory_reused，"
                        f"实际 {describe(summary)}、各次在 {hosts}、编译目录 {directories}、标签 {labels}、"
                        f"第 2 次 build_directory_reused={attempts[-1].get('build_directory_reused') if attempts else None}")
    if (len(arguments) != 2 or "--keep-build-directory" not in arguments[0].split()
            or f"--reuse-build-directory {labels[0] if labels else ''}" not in arguments[1]):
        problems.append(f"交给 multi-host-run.sh 的参数：第 1 次应当带 --keep-build-directory，第 2 次应当带 --reuse-build-directory <第 1 次的标签>，实际 {[line[:300] for line in arguments]}")
    if directories and targets != [os.path.join(directories[0], "target")]:
        problems.append(f"假 cargo 拿到的 CARGO_TARGET_DIR 应当是第 1 次那一份下面的 target（{os.path.join(directories[0], 'target') if directories else ''}），实际 {targets}")
    patience = world.patience(FINISHED_BUILD_GONE_SECONDS)
    while directories and directories[0] and os.path.lexists(directories[0]) and patience.waiting():
        time.sleep(0.2)
    removals = [line for line in world.calls() if line.startswith("peer-remove ")]
    if not directories or not directories[0] or os.path.lexists(directories[0]) or removals != [f"peer-remove label={labels[0]}"]:
        problems.append(f"有了结局之后应当经 multi-host-run.sh --remove-build-directory 删第二台留着的那一份、只删一次，实际那一份"
                        f"{'还在' if directories and directories[0] and os.path.lexists(directories[0]) else '不在'}、删的调用 {removals}")
    if problems:
        raise SelftestFailure("第二台重试的编译目录：" + "；".join(problems), world.server_log_lines())
    return "第二台重排的那一次接着用第 1 次留着的编译目录；有了结局之后服务经 --remove-build-directory 删了它"


BYPASS_WARNING_HEAD = "✗ 严厉警告："   # task_server.py 的 bypass_warning_lines 头一行的开头
BYPASS_NEXT_STEP_HEAD = "→ 怎么办："
# repository-target-bypass-warned 写进假仓根的几份入口脚本：命中的（缺省值退回裸写的 target/release/、$(dirname …)/.. 拼到仓根 target/debug/）、
# 只用服务给的 CARGO_TARGET_DIR 的，与自证那一支里造假路径的 python（--selftest 起的不读正文；名字以 cells 收尾的函数体不读，同 memory_peaks_lib.py 的 key_fold_cells）
BYPASS_SCRIPT_TEXT = ('#!/usr/bin/env bash\n# 自证样本：缺省取仓根 target/ 下的二进制（任务 7937–7944 那种写法）\n'
                      'BINARY="${SAMPLE_BINARY:-target/release/sample_tool}"\nFALLBACK="$(dirname "$0")/../target/debug/sample_tool"\necho "would run $BINARY or $FALLBACK"\n')
SERVICE_TARGET_SCRIPT_TEXT = '#!/usr/bin/env bash\n# 自证样本：只取服务给的编译目录\nBINARY="${CARGO_TARGET_DIR:-/nonexistent}/release/sample_tool"\necho "would run $BINARY"\n'
SELFTEST_SAMPLE_SCRIPT_TEXT = ('import sys\n\n\ndef key_fold_cells(repository):\n'
                               '    return [f"{repository}/target/debug/deps/journal_ring_fields-0123456789abcdef", "--exact", "ring"]\n\n\n'
                               'if __name__ == "__main__":\n    sys.exit(0)\n')


def cell_repository_target_bypass_warned(world):
    """提交时警告（task_server.py 文件头 ②「提交时警告」）：命令里的词落在仓根 target/ 下的、命令位置上起的本仓脚本正文里有指向仓根 target/ 的路径字面的，照收照跑，
    提交回复（客户端打到标准错误）、日志头、status（给号的与会话列表）三处都打「✗ 严厉警告：」那一行（点名会话号与任务号）与「→ 怎么办：」那一行；
    四种不算的（服务给的编译目录下、调用方给的 CARGO_TARGET_DIR 下、草稿目录里仓副本的 target/ 与在副本里起的脚本、自证那一支的假路径）都不打。
    弄坏开关 repository-target-bypass-unwarned 下不判，命中的那两条也不打。"""
    copy = os.path.join(world.work_root, "draft-bypass", "copy")   # 草稿目录里的仓副本：它的 target/ 与在它里面起的同一份脚本都不算
    for root, relative, text in ((world.repo, "scripts/long-run-bypass.sh", BYPASS_SCRIPT_TEXT), (world.repo, "scripts/service-target.sh", SERVICE_TARGET_SCRIPT_TEXT),
                                 (world.repo, "scripts/sample_lib.py", SELFTEST_SAMPLE_SCRIPT_TEXT), (copy, "scripts/long-run-bypass.sh", BYPASS_SCRIPT_TEXT)):
        os.makedirs(os.path.join(root, os.path.dirname(relative)), exist_ok=True)
        write_executable(os.path.join(root, relative), text)
    own_target = os.path.join(world.root, "own-target")
    world.start_server()
    hits = {"命令里的仓根 target/": (["env", f"SAMPLE_BINARY={world.repo}/target/debug/sample_tool", "bash", "-c", "exit 0"], world.repo),
            "入口脚本取仓根 target/": (["bash", "scripts/long-run-bypass.sh"], world.repo)}
    misses = {"服务给的编译目录": (["env", f"SAMPLE_BINARY={world.builds}/task-sample-1/target/release/sample_tool", "bash", "scripts/service-target.sh"], world.repo),
              "调用方给的 CARGO_TARGET_DIR": (["env", f"CARGO_TARGET_DIR={own_target}", f"SAMPLE_BINARY={own_target}/release/sample_tool", "bash", "-c", "exit 0"], world.repo),
              "草稿目录里的仓副本": (["env", f"SAMPLE_BINARY={copy}/target/release/sample_tool", "bash", "scripts/long-run-bypass.sh"], copy),
              "自证那一支的假路径（--selftest 起）": (["python3", "scripts/sample_lib.py", "--selftest"], world.repo),
              "自证那一支的假路径（cells 函数体）": (["python3", "scripts/sample_lib.py"], world.repo)}
    problems, submitted = [], {}
    for name, (command, cwd) in list(hits.items()) + list(misses.items()):
        completed = world.client(["submit", "--cwd", cwd, "--", *command])
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if completed.returncode != 0 or not found:
            problems.append(f"{name}：提交应当照收（退 0、打任务号），实际退 {completed.returncode}、{completed.stderr.strip()[-300:]}")
            continue
        world.confirm_submitted(completed)   # 照确认单确认（task_server.py 文件头 ⑦）：没确认的一直待确认、不进队；警告看的是提交回复的标准错误
        submitted[name] = (int(found.group(1)), completed.stderr)
    listed = world.client(["status"]).stdout
    for name, (task_id, stderr) in submitted.items():
        summary = world.finished(task_id)
        named = f"{BYPASS_WARNING_HEAD}会话 {SESSION_A} 交的任务 {task_id} "
        shown = world.client(["status", str(task_id)]).stdout
        log = world.client(["log", str(task_id), "--all"]).stdout.splitlines()
        head = log[1] if len(log) > 1 else ""
        places = {"提交回复的标准错误": stderr, "status <号>": shown, "会话的 status": listed, "日志头": head}
        if name in hits:
            if summary["task"].get("outcome") != "succeeded":
                problems.append(f"{name}：任务应当照跑（警告不拒），实际 {describe(summary)}")
            missing = [place for place, text in places.items() if named not in text]
            if missing or BYPASS_NEXT_STEP_HEAD not in stderr or not (len(log) > 2 and log[2].startswith(BYPASS_NEXT_STEP_HEAD)):
                problems.append(f"{name}（任务 {task_id}）：三处都应当打「{named}…」与「{BYPASS_NEXT_STEP_HEAD}…」，缺 {missing or '出路那一行'}；"
                                f"标准错误「{stderr.strip()[-300:]}」、日志头两行 {log[1:3]}")
        elif any("严厉警告" in text for text in (stderr, shown, "\n".join(log))) or f"交的任务 {task_id} " in listed:
            problems.append(f"{name}（任务 {task_id}）：不算绕过，不该打严厉警告，实际标准错误「{stderr.strip()[-300:]}」、status「{shown.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("提交时警告：" + "；".join(problems), world.server_log_lines())
    return (f"命中的 {len(hits)} 条照跑、提交回复、日志头、两种 status 都打了点名会话与任务号的严厉警告与出路；"
            f"不算的 {len(misses)} 条（服务给的编译目录、CARGO_TARGET_DIR、草稿里的仓副本、自证那一支两种）都不打")


CANCEL_ARTIFACT_WAIT_SECONDS = 20.0   # cancelled-artifacts-removed：等在跑的那一条把编译目录、现场与第二台这一趟的本地目录建出来的安静时限
# 本机那一条：假 cargo 往服务给的 CARGO_TARGET_DIR 写一份产物，演内存包装留下现场（往服务给的 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY 写 4096 字节），然后占着到放行为止。
# 那 4096 字节用 printf 的宽度造、不点名路径：命令里点到共享根外的路径（/dev/zero 这类）提交时就被拒（task_server.py 文件头 ②「提交时拒」）
CANCEL_LOCAL_SCRIPT = ('cargo build && mkdir -p "$RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY" && printf "%4096s" x > "$RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY/left-by-command" && '
                       + HOLD_UNTIL_RELEASED_SCRIPT)


def cell_cancelled_artifacts_removed(world):
    """取消的不留产物（task_server.py 文件头「接口」的取消；用户：「取消的任务要自己删掉产物 不要残留」）：本机在跑的会编命令（提交时设了 SINGLEFS_KEEP_TEST_TEMPORARY_FILES=1，
    改前记下结局也留着）task cancel 之后编译目录与现场目录都回读不在了，cancel 的输出与日志里逐份写删前删后；派到第二台在跑的那一条，取消之后假第二台本地的
    builds/<标签>/ 与 runs/ 下三个文件不在了（假 multi-host-run.sh 照真的那样被 TERM 停时删、打 ⑥ 那一行），cancel 的输出点名它删了。
    弄坏开关 cancelled-artifacts-kept 下取消之后不删、不报，这一格判红。"""
    world.resources(local=(30, 30), peer=(1, 30))   # 本机那一条在本机起
    world.start_server()
    problems = []
    release = os.path.join(world.root, "release-cancelled-artifacts")   # 一直不放：两条都等被取消
    extra = dict(KEEP_TEMPORARY_EXTRA, PATH=f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}")
    local_id = world.submit(["bash", "-c", CANCEL_LOCAL_SCRIPT, "cancel-local", release], extra=extra)
    attempt = world.running(local_id)["attempts"][-1]
    build, scene = attempt.get("build_directory") or "", attempt.get("scene_directory") or ""
    patience = world.patience(CANCEL_ARTIFACT_WAIT_SECONDS)
    while not (build and os.path.isfile(os.path.join(build, "target", "debug", "selftest-artifact")) and scene and os.path.isfile(os.path.join(scene, "left-by-command"))):
        if not patience.waiting():
            raise SelftestFailure(f"本机那一条（任务 {local_id}）等了 {patience.elapsed():.0f} 秒没写出编译目录里的产物与现场：编译目录「{build}」、现场「{scene}」", world.server_log_lines())
        time.sleep(POLL_SECONDS)
    completed = world.client(["cancel", str(local_id)])
    log = world.client(["log", str(local_id), "--all"]).stdout
    for kind, path in (("编译目录", build), ("现场目录", scene)):
        wanted = f"取消之后删了任务 {local_id} 第 1 次尝试在本机的{kind} {path}：删前 "
        if os.path.lexists(path) or wanted not in completed.stdout or "删后 不在" not in completed.stdout or wanted not in log:
            problems.append(f"本机那一条取消之后{kind} {path} 应当回读不在、cancel 的输出与日志里都有「{wanted}…删后 不在」，"
                            f"实际{'还在' if os.path.lexists(path) else '不在了'}、cancel 退 {completed.returncode}、输出「{completed.stdout.strip()[-400:]}」")
    if completed.returncode != 0:
        problems.append(f"本机那一条 cancel 应当退 0，实际退 {completed.returncode}：{completed.stderr.strip()[-300:]}")
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 第二台那一条只放得下第二台
    peer_id = world.submit(["bash", "-c", "exit 0", f"fake-peer-wait={os.path.basename(release)}-peer"], cwd=world.repo)
    label = world.running(peer_id)["attempts"][-1].get("label") or ""
    trip = [os.path.join(world.peer_local, "builds", label)] + [os.path.join(world.peer_local, "runs", label + suffix) for suffix in (".pid", ".exit", ".log")]
    patience = world.patience(CANCEL_ARTIFACT_WAIT_SECONDS)
    while not (label and all(os.path.lexists(path) for path in trip)):
        if not patience.waiting():
            raise SelftestFailure(f"第二台那一条（任务 {peer_id}）等了 {patience.elapsed():.0f} 秒，假第二台这一趟的本地目录没建出来：{trip}", world.server_log_lines())
        time.sleep(POLL_SECONDS)
    completed = world.client(["cancel", str(peer_id)])
    log = world.client(["log", str(peer_id), "--all"]).stdout
    left = [path for path in trip if os.path.lexists(path)]
    wanted = f"取消之后删了任务 {peer_id} 第 1 次尝试在第二台的本地目录 第二台这一趟的本地目录 builds/{label}"
    if completed.returncode != 0 or left or wanted not in completed.stdout or wanted not in log or SERVER.PEER_TRIP_REMOVED_MARK not in log:
        problems.append(f"第二台那一条取消之后假第二台本地的 builds/{label} 与 runs/ 下三个文件应当都不在、cancel 退 0、输出与日志里有「{wanted}…」、日志里有 ⑥ 那一行，"
                        f"实际还在 {left}、cancel 退 {completed.returncode}、输出「{completed.stdout.strip()[-400:]}」「{completed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("取消的不留产物：" + "；".join(problems), world.server_log_lines())
    return "本机那一条取消之后编译目录与现场回读不在、输出与日志逐份写删前删后；第二台那一条取消之后这一趟的本地目录与 runs/ 下三个文件不在、输出点名删了"


def cell_cancelled_before_peer_trip_built(world):
    """取消时第二台这一趟的本地目录还没建（task_server.py 的 cancelled_attempt_artifacts）：派到第二台的一条，假 multi-host-run.sh 停在建 builds/<标签>/ 之前
    （fake-peer-before-trip=，演真的还在比工具链、查共享根与 ssh），清理回执还是服务写的 not-started；这时 task cancel 退 0、输出里没有「没删掉」，
    假第二台本地没有 builds/<标签>，日志里没有 ⑥ 那一行，任务记 cancelled。
    弄坏开关 cancel-not-started-receipt-unread 下照改前只认 ⑥ 那一行、算没删成、cancel 退 5，这一格红。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 只放得下第二台
    world.start_server()
    release = os.path.join(world.repo, "release-before-peer-trip")   # 一直不放：等被取消
    problems = []
    try:
        task_id = world.submit(["bash", "-c", "exit 0", f"fake-peer-before-trip={os.path.basename(release)}"], cwd=world.repo)
        attempt = world.running(task_id)["attempts"][-1]
        patience = world.patience(CANCEL_ARTIFACT_WAIT_SECONDS)
        while not world.calls_of(task_id, "peer-before-trip"):
            if not patience.waiting():
                raise SelftestFailure(f"第二台那一条（任务 {task_id}）等了 {patience.elapsed():.0f} 秒，假 multi-host-run.sh 没停到建这一趟的本地目录之前", world.server_log_lines())
            time.sleep(POLL_SECONDS)
        label = attempt.get("label") or ""
        receipt = SERVER.read_json_file(attempt.get("peer_cleanup_file") or "") or {}
        completed = world.client(["cancel", str(task_id)])
        log = world.client(["log", str(task_id), "--all"]).stdout
    finally:
        open(release, "w").close()
    shown = completed.stdout + completed.stderr
    trip = os.path.join(world.peer_local, "builds", label)
    if receipt.get("status") != "not-started" or not label:
        problems.append(f"取消之前清理回执应当还是服务写的 not-started、尝试记着标签，实际回执 {receipt}、标签「{label}」")
    if completed.returncode != 0 or "没删掉" in shown:
        problems.append(f"这一趟的本地目录还没建：cancel 应当退 0、不打「没删掉」，实际退 {completed.returncode}、输出「{shown.strip()[-500:]}」")
    if os.path.lexists(trip) or SERVER.PEER_TRIP_REMOVED_MARK in log:
        problems.append(f"假第二台本地不该有 builds/{label}、日志里不该有 ⑥ 那一行（还没建），实际{'有' if os.path.lexists(trip) else '没有'}目录、日志末尾「{log.strip()[-300:]}」")
    outcome = world.summary(task_id)["task"].get("outcome")
    if outcome != "cancelled":
        problems.append(f"任务应当记 cancelled，实际 {outcome}")
    if problems:
        raise SelftestFailure("取消时第二台这一趟的本地目录还没建：" + "；".join(problems), world.server_log_lines())
    return f"清理回执还是 not-started、假第二台还没建 builds/{label} 时取消：cancel 退 0、不报没删掉，没有那份目录、日志里没有 ⑥ 那一行"


class FakeInboxScheduler:
    """inbox-outcomes-resent-after-error 用的假调度器：只备 Scheduler.push_outcomes 读写的几样（待发的号、库里的任务、发消息、记已取）；
    库里的任务记着会话令牌（push_outcomes 只发记着令牌的，收件 socket 由 send_to_session 照会话号现找，这里的 send_to_session 是假的、不找）；
    send_to_session 对 fail_once 点名的那一条抛一次错（演发结局那一轮中途出错），别的记进 sent、交发出去了。"""

    def __init__(self, task_ids, fail_once):
        self.inbox_lock = threading.Lock()
        self.inbox_pending = set(task_ids)
        self.environment = dict(os.environ)
        self.records = {task_id: {"id": task_id, "state": "finished", "inbox": {"token": INBOX_TOKEN_SAMPLE}} for task_id in task_ids}
        self.fail_once = {fail_once}
        self.sent, self.collected = [], []
        self.store = self

    def task(self, task_id):
        return (self.records[task_id], True) if task_id in self.records else None

    def push_attention(self, task, final=False):
        return None

    def summary(self, task_id):
        return {"task": self.records[task_id]}

    def outcome_text(self, summary):
        return f"task-server：任务 {summary['task']['id']} 自证"

    def send_to_session(self, task, text, event):
        if task["id"] in self.fail_once:
            self.fail_once.discard(task["id"])
            raise RuntimeError(f"自证造的发结局出错（任务 {task['id']}）")
        self.sent.append(task["id"])
        return True

    def mark_collected(self, task_id):
        self.collected.append(task_id)
        self.records[task_id]["collected"] = True


INBOX_RESEND_TASKS = (11, 12, 13)   # inbox-outcomes-resent-after-error：同一轮待发的三条结局，发 12 时抛一次错


def cell_inbox_outcomes_resent_after_error(world):
    """发结局的那一轮中途抛错，还没发的下一轮照发（task_server.py 的 push_outcomes；只用它这一个方法与假调度器 FakeInboxScheduler，不起服务）：
    同一轮待发 11、12、13 三条，发 12 时抛一次错：这一轮把错抛给调用方（inbox_loop 打出来、下一轮再判），11 发出去记了已取、不放回，12、13 放回待发；
    下一轮 12、13 照发、记已取，11 不重发。弄坏开关 inbox-pending-dropped-on-error 下抛错时不放回，下一轮一条都不发，这一格红。"""
    scheduler = FakeInboxScheduler(INBOX_RESEND_TASKS, INBOX_RESEND_TASKS[1])
    raised = None
    try:
        SERVER.Scheduler.push_outcomes(scheduler)
    except RuntimeError as error:
        raised = error
    first_sent, first_pending = list(scheduler.sent), sorted(scheduler.inbox_pending)
    SERVER.Scheduler.push_outcomes(scheduler)
    problems = []
    if raised is None or first_sent != [INBOX_RESEND_TASKS[0]] or first_pending != list(INBOX_RESEND_TASKS[1:]):
        problems.append(f"抛错那一轮应当把错抛出来、只发出 {INBOX_RESEND_TASKS[0]}、把 {list(INBOX_RESEND_TASKS[1:])} 放回待发，"
                        f"实际{'抛了' if raised else '没抛'}、发出 {first_sent}、待发 {first_pending}")
    if scheduler.sent != list(INBOX_RESEND_TASKS) or scheduler.collected != list(INBOX_RESEND_TASKS) or scheduler.inbox_pending:
        problems.append(f"下一轮应当接着发 {list(INBOX_RESEND_TASKS[1:])}、{INBOX_RESEND_TASKS[0]} 不重发，三条各记一次已取、待发空了，"
                        f"实际先后发出 {scheduler.sent}、记已取 {scheduler.collected}、待发 {sorted(scheduler.inbox_pending)}")
    if problems:
        raise SelftestFailure("发结局中途抛错：" + "；".join(problems))
    return (f"同一轮 {list(INBOX_RESEND_TASKS)} 发 {INBOX_RESEND_TASKS[1]} 时抛错：{INBOX_RESEND_TASKS[0]} 发了不放回，"
            f"{list(INBOX_RESEND_TASKS[1:])} 放回、下一轮照发，三条各发一次")


def cell_shard_child_key_folded_by_words(world):
    """分片子任务的键照父命令按词折叠（task_server.py 的 create_child）：父命令 --item 后面是带空格的条目名（一个词），计划、片、合并的键都是父任务的键接那一步那一段
    （过同一个换算），后半截「空格的」不漏进键里。弄坏开关 shard-key-from-joined-command 下照改前用空格接父命令的词，后半截漏进键里。"""
    world.resources(local=(30, 30))
    command = world.write_spec("spec-spaced-item.json", {"shards": [{}]}) + ["--item", SPACED_ITEM_NAME]
    world.start_server()
    summary = world.finished(world.submit(command, cwd=world.repo))
    parent_key = summary["task"].get("command_key") or ""
    problems = []
    if summary["task"].get("outcome") != "succeeded" or not summary.get("children"):
        problems.append(f"带空格条目名的分片命令应当拆开跑完，实际 {describe(summary)}")
    for child in summary.get("children", []):
        record = child["task"]
        wanted = PEAKS.command_key([], parent_key + lib.shard_child_key("", record.get("role"), int(record.get("shard_index") or 0), int(record.get("shard_count") or 0)))
        key = record.get("command_key") or ""
        if key != wanted or SPACED_ITEM_TAIL in key.split():
            problems.append(f"{record.get('role')} {record.get('shard_index')} 的键应当是「{wanted}」（父任务的键接那一段），实际「{key}」")
    if SPACED_ITEM_TAIL in parent_key.split():
        problems.append(f"父任务的键里不该有条目名的后半截，实际「{parent_key}」")
    if problems:
        raise SelftestFailure("分片子任务的键按词折叠：" + "；".join(problems), world.server_log_lines())
    return f"条目名带空格的分片命令：{len(summary.get('children', []))} 个子任务的键都是父任务的键「{parent_key}」接那一步那一段"


def write_patch_writer_world(world):
    """假仓根里放假 apply-writer-patch.py 与两份补丁目录：patch-half 的结局文件没有完成（打了一半），patch-full 的有完成（打全了）。"""
    gate = os.path.join(world.repo, "scripts", "gate")
    os.makedirs(gate, exist_ok=True)
    write_executable(os.path.join(gate, "apply-writer-patch.py"), FAKE_APPLY_WRITER_PATCH)
    for name, text in (("patch-half", HALF_APPLIED_OUTCOME), ("patch-full", FULLY_APPLIED_OUTCOME)):
        os.makedirs(os.path.join(world.repo, name), exist_ok=True)
        with open(os.path.join(world.repo, name, SERVER.PATCH_WRITER_OUTCOME_NAME), "w", encoding="utf-8") as handle:
            handle.write(text)


def attention_lines(completed):
    return [line for line in completed.stdout.splitlines() if line.startswith(f"{lib.ATTENTION_MARK} ")]


def cell_workspace_write_interrupted_reported(world):
    """改主工作区的命令没正常结束、可能写了一半，wait 报一行 workspace-write-interrupted（task_server.py 的 workspace_write_item）：apply-writer-patch.py 撞了上限
    而补丁目录的结局文件没有完成（假 --read-outcome 退 3），wait 在理由行之前打 TASK_SERVER_ATTENTION、点名补丁目录、退码仍是任务的（250）；
    stage-mine.py 被信号杀（命令自己 kill -KILL 自己，退 137）同样报。弄坏开关 workspace-write-interrupted-unreported 下不报。"""
    write_patch_writer_world(world)
    world.start_server()
    problems = []
    for name, command, code in (("打了一半的 apply-writer-patch.py", ["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-half", "fake-wrapper-exits=250"], 250),
                                ("被信号杀的 stage-mine.py", ["bash", "-c", "kill -KILL $$", "x", "scripts/gate/stage-mine.py", "fake-wrapper-exits=run"], 137)):
        task_id = world.submit(command, cwd=world.repo)
        world.finished(task_id)
        completed = world.client(["wait", str(task_id)])
        lines = attention_lines(completed)
        result = [line for line in completed.stdout.splitlines() if line.startswith(f"{lib.RESULT_MARK} ")]
        if (completed.returncode != code or len(lines) != 1 or "kind=workspace-write-interrupted" not in lines[0]
                or ("patch-half" not in lines[0] and "apply-writer-patch" in name) or not result):
            problems.append(f"{name}：wait 应当退 {code}、在末行之前打一行 kind=workspace-write-interrupted 的提醒（点名补丁目录），"
                            f"实际退 {completed.returncode}、提醒 {lines}、末行 {result}")
    if problems:
        raise SelftestFailure("改主工作区的命令没正常结束：" + "；".join(problems), world.server_log_lines())
    return "打了一半的 apply-writer-patch.py 撞上限、被信号杀的 stage-mine.py：wait 各打一行 workspace-write-interrupted，退码仍是任务的"


def cell_workspace_write_complete_not_reported(world):
    """打全了不报（task_server.py 的 workspace_write_item）：apply-writer-patch.py 撞了上限而补丁目录的结局文件有完成（假 --read-outcome 退 0）、带 --dry-run 的、
    正常跑完的，wait 都不打 workspace-write-interrupted。弄坏开关 apply-outcome-ignored 下不读结局文件，打全了的也报。"""
    write_patch_writer_world(world)
    world.start_server()
    problems = []
    for name, command in (("打全了的", ["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-full", "fake-wrapper-exits=250"]),
                          ("--dry-run 的", ["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-half", "--dry-run", "fake-wrapper-exits=250"]),
                          ("正常跑完的", ["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "patch-half", "fake-wrapper-exits=0"])):
        task_id = world.submit(command, cwd=world.repo)
        world.finished(task_id)
        lines = attention_lines(world.client(["wait", str(task_id)]))
        if lines:
            problems.append(f"{name} apply-writer-patch.py 不该报 workspace-write-interrupted，实际 {lines}")
    if problems:
        raise SelftestFailure("改主工作区的命令打全了不报：" + "；".join(problems), world.server_log_lines())
    return "结局文件写着完成的、--dry-run 的、正常跑完的 apply-writer-patch.py 都不报"


def write_card_script(world, name, count):
    """工作目录里一份要卡（# task-server-gpu-arguments: *）、声明要 count 张的假脚本；交提交它的命令。"""
    with open(os.path.join(world.work, name), "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_GPU_ARGUMENTS_MARK} *\n{lib.DECLARED_GPU_CARDS_MARK} {count}\nexit 0\n")
    return ["bash", name, "run", "fake-wrapper-exits=0"]


def two_host_cards(world, cards, local_count, peer_count):
    world.resources(local=(100, 100), peer=(100, 100), peer_cap_gibibytes=64, cards=cards,
                    local_gpu=[(index, CARD_FREE_MEBIBYTES) for index in range(local_count)], peer_gpu=[(index, CARD_FREE_MEBIBYTES) for index in range(peer_count)])


def cell_gpu_two_cards_same_host(world):
    """要两张分到同一台两张（task_server.py 的 place_on_card）：卡的先后是本机 0、第二台 0、本机 1；脚本文件头声明要 2 张的整组分到本机 0 与本机 1；
    cargo test 点名的包里测试源头写「// task-server-gpu-cards: 2」的同样。弄坏开关 gpu-card-count-ignored 下只分到一张。"""
    two_host_cards(world, f"{LOCAL_CARD_ZERO} {PEER_CARD_ZERO} {LOCAL_CARD_ONE}", 2, 1)
    package = os.path.join(world.repo, "crates", "selftest-cards")
    os.makedirs(os.path.join(package, "tests"), exist_ok=True)
    with open(os.path.join(package, "Cargo.toml"), "w", encoding="utf-8") as handle:
        handle.write('[package]\nname = "selftest-cards"\nversion = "0.0.0"\n')
    with open(os.path.join(package, "tests", "real_gpu.rs"), "w", encoding="utf-8") as handle:
        handle.write(f"{lib.DECLARED_SOURCE_GPU_CARDS_MARK} 2\n#[test]\nfn each_card_collects() {{}}\n")
    world.start_server()
    wanted = (f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE}", "local")
    script = world.finished(world.submit(write_card_script(world, "two-cards.sh", 2)))
    cargo = world.finished(world.submit(["cargo", "test", "-p", "selftest-cards", "--features", "gpu", "--tests", "fake-wrapper-exits=0"], cwd=world.repo))
    problems = [f"{name}应当整组分到 {wanted}，实际 {attempt_card_and_host(summary)}（{describe(summary)}，张数 {summary['task'].get('gpu_cards')}）"
                for name, summary in (("脚本声明 2 张的", script), ("cargo test 包里源头写 2 张的", cargo)) if attempt_card_and_host(summary) != wanted]
    if problems:
        raise SelftestFailure("要两张卡：" + "；".join(problems), world.server_log_lines())
    return f"脚本声明与 cargo 包的测试源头声明要 2 张的都整组分到本机两张（{wanted[0]}）"


def cell_gpu_three_cards_only_where_enough(world):
    """要三张只落凑得出的那台：本机两张在前、第二台三张在后，声明要 3 张的整组分到第二台的三张；要 1 张的照优先级分到本机 0。
    弄坏开关 gpu-card-count-ignored 下要 3 张的也分到本机 0 一张。"""
    two_host_cards(world, f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE} {PEER_CARD_ZERO} {PEER_CARD_ONE} {PEER_CARD_TWO}", 2, 3)
    world.start_server()
    three = world.finished(world.submit(write_card_script(world, "three-cards.sh", 3)), timeout=PLACEMENT_WAIT_SECONDS)
    one = world.finished(world.submit(write_card_script(world, "one-card.sh", 1)))
    problems = []
    if attempt_card_and_host(three) != (f"{PEER_CARD_ZERO} {PEER_CARD_ONE} {PEER_CARD_TWO}", "peer"):
        problems.append(f"要 3 张的应当整组分到第二台三张，实际 {attempt_card_and_host(three)}（{describe(three)}）")
    if attempt_card_and_host(one) != (LOCAL_CARD_ZERO, "local"):
        problems.append(f"要 1 张的应当照优先级分到 {LOCAL_CARD_ZERO}，实际 {attempt_card_and_host(one)}")
    if problems:
        raise SelftestFailure("要三张卡：" + "；".join(problems), world.server_log_lines())
    return "要 3 张的整组落到凑得出三张的第二台，要 1 张的照优先级落本机 0"


def cell_gpu_cards_short_never_fits(world):
    """哪台都凑不出打回：本机两张、第二台三张，声明要 4 张的记 never-fits（退 241），说明写哪台都凑不出；张数写坏的（0）记 run-condition-unmet、说明点名声明。
    弄坏开关 gpu-card-count-ignored 下要 4 张的照一张起、跑完。"""
    two_host_cards(world, f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE} {PEER_CARD_ZERO} {PEER_CARD_ONE} {PEER_CARD_TWO}", 2, 3)
    world.start_server()
    four = world.finished(world.submit(write_card_script(world, "four-cards.sh", 4)))
    broken_count = world.finished(world.submit(write_card_script(world, "zero-cards.sh", 0)))
    problems = []
    if four["task"].get("outcome") != "never-fits" or four["task"].get("exit_code") != 241 or four["attempts"] or "凑不出" not in (four["task"].get("note") or ""):
        problems.append(f"要 4 张、哪台都凑不出：应当记 never-fits（退 241）、一次没起、说明写凑不出，实际 {describe(four)}、说明「{four['task'].get('note')}」")
    if broken_count["task"].get("outcome") != "run-condition-unmet" or "张数声明" not in (broken_count["task"].get("note") or ""):
        problems.append(f"张数写成 0 的应当记 run-condition-unmet、说明点名张数声明，实际 {describe(broken_count)}、说明「{broken_count['task'].get('note')}」")
    if problems:
        raise SelftestFailure("凑不出的卡：" + "；".join(problems), world.server_log_lines())
    return "要 4 张哪台都凑不出的记 never-fits；张数写坏的记 run-condition-unmet"


# ── 暂停（task_server.py 文件头 ⑤「暂停」；control pause / resume）──
PAUSE_REASON_SAMPLE = "自证：换二进制要一段空档"
PAUSED_WATCH_SECONDS = 1.5   # 暂停着看这么久排着的任务有没有被起（排队一轮 0.1 秒：该起的早起了）
# 第 1 次尝试等放行文件在了退 254（被总上限挤掉，同一上限重排），第 2 次退 0：暂停着第 1 次跑完，重排回队的那一条不该被起
RETRY_AFTER_RELEASE_SCRIPT = ('for round in $(seq 1 2000); do [[ -e "$1" ]] && break; sleep 0.1; done; '
                              '[[ "$TASK_SERVER_ATTEMPT" == 1 ]] && exit 254; exit 0')


def pause_by_api(world, reason, allow_session=""):
    """POST /pause（这一格不是测 control 的，经接口直接暂停；给了 allow_session 就放行那个会话）：回 200 且 paused 为真才交回话。"""
    status, reply = world.api("POST", "/pause", {"reason": reason, "allow_session": allow_session} if allow_session else {"reason": reason})
    if status != 200 or not reply.get("paused"):
        raise SelftestFailure(f"POST /pause 应当回 200、paused 为真，实际 HTTP {status}：{reply}", world.server_log_lines())
    return reply


def pause_health(world):
    """此刻 /health 的整份回话（读的是服务存的暂停态与计数）。"""
    return lib.call_server(os.path.join(world.runtime, lib.SOCKET_NAME), "GET", "/health", timeout=5)[1]


def held_in_queue(world, task_id):
    """这条任务此刻排着没起：state 是 queued、一次尝试都没有。"""
    summary = world.summary(task_id)
    return summary["task"].get("state") == "queued" and not summary["attempts"]


def cell_pause_holds_new_attempts(world):
    """暂停着交的任务排着不起：普通任务与分片的计划那一步（子任务也是新尝试）都留在队里，一次尝试都没有，命令一行没跑。"""
    world.resources(local=(30, 30))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE)
    marker = os.path.join(world.root, "paused-ran")
    plain_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", marker])
    sharded_id = world.submit(world.write_spec("spec-paused.json", {"shards": [{}]}), cwd=world.repo)
    time.sleep(PAUSED_WATCH_SECONDS)
    plain, sharded = world.summary(plain_id), world.summary(sharded_id)
    children = sharded.get("children", [])
    if (plain["task"].get("state") != "queued" or plain["attempts"] or os.path.exists(marker) or world.sharded_calls("plan")
            or child_roles(sharded) != ["plan"] or any(child["task"].get("state") != "queued" or child["attempts"] for child in children)):
        raise SelftestFailure(f"暂停着交的任务应当排着不起：普通任务应当 queued、没有尝试、命令没跑，分片的只有计划那一个子任务、也是 queued 没有尝试，"
                              f"实际普通任务 {plain['task'].get('state')}、尝试 {len(plain['attempts'])} 次、命令{'跑了' if os.path.exists(marker) else '没跑'}，"
                              f"分片的子任务 {child_roles(sharded)}、状态 {[child['task'].get('state') for child in children]}、计划跑了 {len(world.sharded_calls('plan'))} 回",
                              world.server_log_lines())
    return f"暂停着交的普通任务与分片的计划子任务排了 {PAUSED_WATCH_SECONDS} 秒都没起：没有尝试、命令没跑"


def cell_pause_lets_running_finish(world):
    """在跑的照跑到结局：跑着的第 1 次尝试暂停之后才放行、退 254，暂停着对账照做（结局写回、重排回队），回队的那一条不起第 2 次。"""
    world.resources(local=(30, 30))
    world.start_server()
    release = os.path.join(world.root, "pause-release")
    task_id = world.submit(["bash", "-c", RETRY_AFTER_RELEASE_SCRIPT, "retry-after-release", release])
    world.running(task_id)
    pause_by_api(world, PAUSE_REASON_SAMPLE)
    open(release, "w").close()
    world.wait_for(task_id, lambda found: found["task"].get("state") == "queued" and [attempt.get("outcome") for attempt in found["attempts"]] == ["squeezed-out"],
                   "把暂停之前就在跑的第 1 次尝试的结局写回、重排回队（暂停着对账照做）")
    time.sleep(PAUSED_WATCH_SECONDS)
    later = world.summary(task_id)
    if later["task"].get("state") != "queued" or len(later["attempts"]) != 1:
        raise SelftestFailure(f"暂停着重排回队的那一条不该起第 2 次尝试，实际 {later['task'].get('state')}、尝试 {len(later['attempts'])} 次（{describe(later)}）",
                              world.server_log_lines())
    return "暂停之前在跑的第 1 次尝试照跑到结局（squeezed-out 写回、重排回队），回队的那一条暂停着不起第 2 次"


def cell_pause_survives_restart(world):
    """暂停态记在库里：服务重启后照旧暂停（再 pause 回 changed 为假，原因与起点还是原来的）；库里的暂停记录被绕过服务改过的，按暂停算、说明写校验不过。"""
    world.resources(local=(30, 30))
    world.start_server()
    first = pause_by_api(world, PAUSE_REASON_SAMPLE)
    world.stop_server()
    world.start_server()
    status, again = world.api("POST", "/pause", {"reason": "重启之后又暂停"})
    if status != 200 or again.get("changed") or again.get("reason") != PAUSE_REASON_SAMPLE or again.get("since") != first.get("since"):
        raise SelftestFailure(f"服务重启后应当还暂停着：再 pause 应当回 changed 为假、原因「{PAUSE_REASON_SAMPLE}」、起点 {first.get('since')} 不变，实际 HTTP {status}：{again}",
                              world.server_log_lines())
    world.stop_server()
    with contextlib.closing(sqlite3.connect(os.path.join(world.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
        connection.execute("UPDATE meta SET value = replace(value, ?, ?) WHERE name = ?", (PAUSE_REASON_SAMPLE, "绕过服务改过的原因", SERVER.PAUSE_META_NAME))
    world.start_server()
    status, health = lib.call_server(os.path.join(world.runtime, lib.SOCKET_NAME), "GET", "/health", timeout=5)
    if status != 200 or not health.get("paused") or "校验不过" not in ((health.get("pause") or {}).get("reason") or ""):
        raise SelftestFailure(f"暂停记录被改过时应当按暂停算、原因写校验不过，实际 HTTP {status}：paused {health.get('paused')}、pause {health.get('pause')}", world.server_log_lines())
    return "重启后再 pause 回 changed 为假、原因与起点不变；暂停记录被改过的按暂停算、写校验不过"


def cell_resume_releases_queue(world):
    """resume 放开：暂停着交的任务在 resume 之后起来、跑完，resume 回话与 /health 都不是暂停；再 resume 回 changed 为假。"""
    world.resources(local=(30, 30))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE)
    task_id = world.submit(["bash", "-c", "exit 0"])
    status, reply = world.api("POST", "/resume", {})
    health = lib.call_server(os.path.join(world.runtime, lib.SOCKET_NAME), "GET", "/health", timeout=5)[1]
    if status != 200 or reply.get("paused") or not reply.get("changed") or health.get("paused"):
        raise SelftestFailure(f"resume 应当回 200、paused 为假、changed 为真，/health 的 paused 也为假，实际 HTTP {status}：{reply}、/health paused {health.get('paused')}",
                              world.server_log_lines())
    summary = world.finished(task_id)
    status, again = world.api("POST", "/resume", {})
    if summary["task"].get("outcome") != "succeeded" or status != 200 or again.get("changed"):
        raise SelftestFailure(f"resume 之后排着的任务应当起来、跑完 succeeded，再 resume 回 changed 为假，实际 {describe(summary)}、再 resume HTTP {status}：{again}",
                              world.server_log_lines())
    return "resume 之后暂停着交的任务起来、跑完 succeeded；再 resume 回 changed 为假"


def cell_pause_reported(world):
    """control pause / check / resume 与 status 的字样：pause 回读是暂停才退 0、已暂停着的说原来就暂停着；check 退 0、另打「暂停中（原因：…，起于…）、在跑 N 个」；
    排着的任务 status 写「在等：服务暂停中（原因：…）」；resume 之后这几句都没了；pause、resume 各写一条操作日志。"""
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    problems = []
    paused = world.control(["pause", "--reason", PAUSE_REASON_SAMPLE])
    if paused.returncode != 0 or "暂停了" not in paused.stdout or PAUSE_REASON_SAMPLE not in paused.stdout:
        problems.append(f"control pause 应当退 0、说暂停了并点名原因，实际退 {paused.returncode}、stdout「{paused.stdout.strip()[-200:]}」、stderr「{paused.stderr.strip()[-200:]}」")
    task_id = world.submit(["bash", "-c", "exit 0"])
    shown = world.client(["status", str(task_id)])
    if f"在等：服务暂停中（原因：{PAUSE_REASON_SAMPLE}）" not in shown.stdout:
        problems.append(f"暂停着排着的任务 status 应当写「在等：服务暂停中（原因：{PAUSE_REASON_SAMPLE}）」，实际「{shown.stdout.strip()[:300]}」")
    checked = world.control(["check"])
    if checked.returncode != 0 or f"暂停中（原因：{PAUSE_REASON_SAMPLE}，起于 " not in checked.stdout or "）、在跑 0 个" not in checked.stdout:
        problems.append(f"暂停着 control check 应当退 0、另打「暂停中（原因：{PAUSE_REASON_SAMPLE}，起于 …）、在跑 0 个」，实际退 {checked.returncode}、"
                        f"stdout「{checked.stdout.strip()[-300:]}」、stderr「{checked.stderr.strip()[-200:]}」")
    again = world.control(["pause", "--reason", "别的原因"])
    if again.returncode != 0 or "原来就暂停着" not in again.stdout or PAUSE_REASON_SAMPLE not in again.stdout:
        problems.append(f"已经暂停着再 control pause 应当退 0、说原来就暂停着、原因还是原来的，实际退 {again.returncode}、stdout「{again.stdout.strip()[-200:]}」")
    resumed = world.control(["resume"])
    if resumed.returncode != 0 or "放开了" not in resumed.stdout:
        problems.append(f"control resume 应当退 0、说放开了，实际退 {resumed.returncode}、stdout「{resumed.stdout.strip()[-200:]}」、stderr「{resumed.stderr.strip()[-200:]}」")
    after_check = world.control(["check"])
    after_status = world.client(["status", str(task_id)])
    if "暂停中" in after_check.stdout or "服务暂停中" in after_status.stdout:
        problems.append(f"resume 之后 control check 与 status 都不该再写暂停，实际 check「{after_check.stdout.strip()[-200:]}」、status「{after_status.stdout.strip()[:200]}」")
    notes = {operation.get("action"): operation.get("note") for operation in world.operations() if operation.get("action") in ("pause", "resume")}
    if notes.get("pause") != PAUSE_REASON_SAMPLE or "resume" not in notes:
        problems.append(f"操作日志里应当有 pause（备注原因「{PAUSE_REASON_SAMPLE}」）与 resume 各一条，实际 {notes}")
    if problems:
        raise SelftestFailure("暂停的字样：" + "；".join(problems), world.server_log_lines())
    return "control pause 回读退 0、已暂停的说原来就暂停着；check 退 0 另打暂停中；status 写在等服务暂停中；resume 之后都没了；操作日志有 pause 与 resume"


def cell_pause_not_starved(world):
    """暂停着不报「排头放得下却没起」：本机空 100G、排头一条小的放得下，暂停着排着不起，过了阈值几倍 /health 照回 ok，control check 退 0。"""
    world.resources(local=(100, 100))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE)
    world.submit(["bash", "-c", "exit 0"])
    health = health_until(world, lambda found: not found.get("ok"), 4 * SERVER.FAST_STARVED_SECONDS)
    checked = world.control(["check"])
    if not health.get("ok") or checked.returncode != 0 or "放得下" in checked.stdout + checked.stderr:
        raise SelftestFailure(f"暂停着排头放得下而不起是要的：/health 应当一直回 ok、control check 应当退 0 且不提放得下，实际 /health {health.get('ok')}（{health.get('problem')}）、"
                              f"check 退 {checked.returncode}、「{(checked.stdout + checked.stderr).strip()[-300:]}」", world.server_log_lines())
    return f"暂停着排头放得下而不起，过了阈值 {4 * SERVER.FAST_STARVED_SECONDS:.0f} 倍的时间 /health 照回 ok，control check 退 0"


def cell_pause_waits_for_starting_attempt(world):
    """pause 与起尝试共一把锁：排队线程判过放得下、记尝试之前停住，这时 pause 不回；放行、尝试记下之后 pause 才回，回话的在跑数里有这一条
    （等 control check 报在跑 0 个才做活，不会被正在起的一条挤进来）。"""
    pause_point = os.path.join(world.root, "pause-point")
    os.makedirs(pause_point)
    world.resources(local=(30, 30), hold=True)
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"start-checked:{pause_point}"})
    release = os.path.join(world.root, "starting-release")
    task_id = world.submit(blocking_command(release))
    open(os.path.join(pause_point, "arm-start-checked"), "w").close()
    world.resources(local=(30, 30))
    replies = []
    pauser = threading.Thread(target=lambda: replies.append(world.api("POST", "/pause", {"reason": PAUSE_REASON_SAMPLE})))
    try:
        wait_path(world, os.path.join(pause_point, "reached-start-checked"), "停在判过 queued 之后、记尝试之前")
        pauser.start()
        time.sleep(1.0)
        answered_early = bool(replies)
        open(os.path.join(pause_point, "release-start-checked"), "w").close()
        pauser.join(timeout=STATE_TIMEOUT_SECONDS)
        if answered_early or not replies:
            raise SelftestFailure(f"一条尝试正在起（判过放得下、还没记下）时 pause 应当等它记下再回，实际{'没等就回了' if answered_early else '放行之后也没回'}："
                                  f"{replies}", world.server_log_lines())
        status, reply = replies[0]
        if status != 200 or not reply.get("paused") or reply.get("running") != 1:
            raise SelftestFailure(f"pause 等正在起的那一条记下之后才回：应当 200、paused 为真、在跑 1 个，实际 HTTP {status}：{reply}", world.server_log_lines())
        world.running(task_id)
    finally:
        open(release, "w").close()
        if pauser.ident is not None:
            pauser.join(timeout=10)
    return "尝试正在起时 pause 不回，记下之后才回、在跑数里有这一条"


# ── 暂停时放行一个会话（task_server.py 文件头 ⑤「暂停」的放行一段；control pause --allow-session）──
def cell_pause_allows_one_session(world):
    """暂停时放行了一个会话：它交的普通任务、撞上限重排回队的任务、分片的各步（子任务随提交它的会话）照常排、照常起、跑到 succeeded；别的会话交的任务排着不起，一次尝试都没有。"""
    world.resources(local=(30, 30))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE, allow_session=SESSION_A)
    other_marker = os.path.join(world.root, "other-session-ran")
    allowed_marker = os.path.join(world.root, "allowed-session-ran")
    release = os.path.join(world.root, "allowed-retry-release")
    open(release, "w").close()   # 放行文件早就在：第 1 次尝试一起来就退 254（重排回队），第 2 次退 0
    other_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", other_marker], session=SESSION_B)
    plain_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", allowed_marker], session=SESSION_A)
    retried_id = world.submit(["bash", "-c", RETRY_AFTER_RELEASE_SCRIPT, "retry-after-release", release], session=SESSION_A)
    sharded_id = world.submit(world.write_spec("spec-allowed.json", {"shards": [{}]}), session=SESSION_A, cwd=world.repo)
    finished = {name: world.finished(task_id) for name, task_id in (("普通任务", plain_id), ("撞上限重排回队的任务", retried_id), ("分片的任务", sharded_id))}
    time.sleep(PAUSED_WATCH_SECONDS)
    problems = [f"放行会话交的{name}应当照常起、跑到 succeeded，实际 {describe(summary)}" for name, summary in finished.items() if summary["task"].get("outcome") != "succeeded"]
    if child_roles(finished["分片的任务"]) != ["merge", "plan", "shard"]:
        problems.append(f"放行会话交的分片任务的计划、片、合并三步都应当照常起，实际子任务 {child_roles(finished['分片的任务'])}")
    if not os.path.exists(allowed_marker):
        problems.append("放行会话交的普通任务的命令应当跑了（标记文件没留下）")
    if not held_in_queue(world, other_id) or os.path.exists(other_marker):
        problems.append(f"别的会话交的任务暂停着应当排着不起（queued、没有尝试、命令没跑），实际 {describe(world.summary(other_id))}、命令{'跑了' if os.path.exists(other_marker) else '没跑'}")
    if problems:
        raise SelftestFailure("暂停时放行一个会话：" + "；".join(problems), world.server_log_lines())
    return "放行会话的普通任务、撞上限重排回队的任务、分片的三步暂停着照常起跑完；别的会话的任务排了一阵都没起"


def cell_pause_allow_waiting_text(world):
    """暂停时放行了一个会话：别的会话排着的任务 status 写「在等：服务暂停中（原因：…）」；放行会话排着的任务不写暂停，写它真在等什么（这里是假余量写了 hold）。"""
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE, allow_session=SESSION_A)
    allowed_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_A)
    other_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_B)
    time.sleep(1)
    allowed_shown = world.client(["status", str(allowed_id)], session=SESSION_A)
    other_shown = world.client(["status", str(other_id)], session=SESSION_B)
    world.resources(local=(30, 30))
    world.finished(allowed_id)
    problems = []
    if "在等：自证的假余量写了 hold" not in allowed_shown.stdout or "服务暂停中" in allowed_shown.stdout:
        problems.append(f"放行会话排着的任务 status 应当写它真在等什么（「在等：自证的假余量写了 hold」）、不写暂停，实际「{allowed_shown.stdout.strip()[:300]}」")
    if f"在等：服务暂停中（原因：{PAUSE_REASON_SAMPLE}）" not in other_shown.stdout:
        problems.append(f"别的会话排着的任务 status 应当写「在等：服务暂停中（原因：{PAUSE_REASON_SAMPLE}）」，实际「{other_shown.stdout.strip()[:300]}」")
    if problems:
        raise SelftestFailure("放行时的在等字样：" + "；".join(problems), world.server_log_lines())
    return "放行会话排着的任务 status 写它真在等什么，别的会话排着的写在等服务暂停中"


def cell_pause_allow_switches_session(world):
    """control pause --allow-session 起手放行一个会话；已暂停着再 pause --allow-session 另一个会话退 0、说换了放行的会话、原因与起点不变：之后另一个会话早先交的排着的任务起来，原来放行的会话新交的反而排着；
    再放行同一个会话说原来就暂停着；操作日志有一条换放行会话。"""
    world.resources(local=(30, 30))
    world.start_server()
    problems = []
    paused = world.control(["pause", "--allow-session", SESSION_A, "--reason", PAUSE_REASON_SAMPLE])
    if paused.returncode != 0 or "暂停了" not in paused.stdout or SESSION_A not in paused.stdout:
        problems.append(f"control pause --allow-session 应当退 0、说暂停了并点名放行的会话，实际退 {paused.returncode}、stdout「{paused.stdout.strip()[-300:]}」、stderr「{paused.stderr.strip()[-200:]}」")
    started = (pause_health(world).get("pause") or {}).get("since")
    world.finished(world.submit(["bash", "-c", "exit 0"], session=SESSION_A))
    held_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_B)
    time.sleep(PAUSED_WATCH_SECONDS)
    if not held_in_queue(world, held_id):
        problems.append(f"换之前另一个会话交的任务应当排着不起，实际 {describe(world.summary(held_id))}")
    switched = world.control(["pause", "--allow-session", SESSION_B])
    if switched.returncode != 0 or "换了放行的会话" not in switched.stdout or SESSION_A not in switched.stdout or SESSION_B not in switched.stdout:
        raise SelftestFailure("已暂停着再 pause --allow-session 另一个会话应当退 0、说换了放行的会话并点名换前换后："
                              f"实际退 {switched.returncode}、stdout「{switched.stdout.strip()[-300:]}」、stderr「{switched.stderr.strip()[-300:]}」" + "；".join(problems), world.server_log_lines())
    summary = world.finished(held_id)
    late_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_A)
    time.sleep(PAUSED_WATCH_SECONDS)
    shown = pause_health(world).get("pause") or {}
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"换放行的会话之后，新放行的会话早先交的排着的任务应当起来跑完 succeeded，实际 {describe(summary)}")
    if not held_in_queue(world, late_id):
        problems.append(f"换放行的会话之后，原来放行的会话新交的任务应当排着不起，实际 {describe(world.summary(late_id))}")
    if shown.get("reason") != PAUSE_REASON_SAMPLE or shown.get("since") != started or shown.get("allow_session") != SESSION_B:
        problems.append(f"换放行的会话不改原因与起点：/health 应当是原因「{PAUSE_REASON_SAMPLE}」、起点 {started}、放行会话 {SESSION_B}，实际 {shown}")
    again = world.control(["pause", "--allow-session", SESSION_B])
    if again.returncode != 0 or "原来就暂停着" not in again.stdout or "换了" in again.stdout:
        problems.append(f"再放行同一个会话应当退 0、说原来就暂停着、不说换了，实际退 {again.returncode}、stdout「{again.stdout.strip()[-200:]}」")
    notes = [operation.get("note", "") for operation in world.operations() if operation.get("action") == "pause"]
    if not any("换放行会话" in note and SESSION_A in note and SESSION_B in note for note in notes):
        problems.append(f"操作日志里应当有一条 pause 备注「换放行会话 {SESSION_A} → {SESSION_B}」，实际 {notes}")
    if problems:
        raise SelftestFailure("换放行的会话：" + "；".join(problems), world.server_log_lines())
    return "pause --allow-session 起手放行一个会话；再放行另一个会话退 0、说换了，另一个会话排着的起来、原来放行的会话新交的排着，原因与起点不变；同一个会话再放行说原来就暂停着；操作日志有换放行会话"


def cell_resume_clears_allowed_session(world):
    """resume 把放行一并清掉：放行了一个会话的暂停 resume 之后，不带放行再 pause，回话里没有放行的会话，上一回放行的会话交的任务排着不起。"""
    world.resources(local=(30, 30))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE, allow_session=SESSION_A)
    status, resumed = world.api("POST", "/resume", {})
    if status != 200 or resumed.get("paused") or not resumed.get("changed"):
        raise SelftestFailure(f"resume 应当回 200、paused 为假、changed 为真，实际 HTTP {status}：{resumed}", world.server_log_lines())
    second = pause_by_api(world, "放开之后又暂停")
    task_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_A)
    time.sleep(PAUSED_WATCH_SECONDS)
    problems = []
    if second.get("allow_session") != "":
        problems.append(f"resume 之后不带放行再 pause，回话里不该还有放行的会话，实际 {second}")
    if not held_in_queue(world, task_id):
        problems.append(f"resume 之后不带放行再 pause，上一回放行的会话交的任务应当排着不起，实际 {describe(world.summary(task_id))}")
    if problems:
        raise SelftestFailure("resume 清放行：" + "；".join(problems), world.server_log_lines())
    return "resume 之后不带放行再 pause：回话没有放行的会话，上一回放行的会话交的任务排着不起"


def cell_pause_allow_survives_restart(world):
    """放行的会话记在带签名的暂停态里：服务重启后照旧放行（/health 点名它，它新交的照起）；库里的放行会话被绕过服务改成另一个会话的，按暂停算、不放行谁、说明写校验不过，另一个会话的任务排着不起。"""
    world.resources(local=(30, 30))
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE, allow_session=SESSION_A)
    world.stop_server()
    world.start_server()
    kept = pause_health(world).get("pause") or {}
    if kept.get("allow_session") != SESSION_A:
        raise SelftestFailure(f"服务重启后应当照旧放行会话 {SESSION_A}，实际 /health 的 pause {kept}", world.server_log_lines())
    after_restart = world.finished(world.submit(["bash", "-c", "exit 0"], session=SESSION_A))
    held_id = world.submit(["bash", "-c", "exit 0"], session=SESSION_B)
    world.stop_server()
    with contextlib.closing(sqlite3.connect(os.path.join(world.state, lib.DATABASE_NAME), timeout=30, isolation_level=None)) as connection:
        connection.execute("UPDATE meta SET value = replace(value, ?, ?) WHERE name = ?",
                           (f'"allow_session":"{SESSION_A}"', f'"allow_session":"{SESSION_B}"', SERVER.PAUSE_META_NAME))
    world.start_server()
    time.sleep(PAUSED_WATCH_SECONDS)
    health = pause_health(world)
    tampered = health.get("pause") or {}
    problems = []
    if after_restart["task"].get("outcome") != "succeeded":
        problems.append(f"重启后放行会话新交的任务应当照起跑完，实际 {describe(after_restart)}")
    if not health.get("paused") or tampered.get("allow_session") != "" or "校验不过" not in (tampered.get("reason") or ""):
        problems.append(f"库里的放行会话被改过时应当按暂停算、不放行谁、原因写校验不过，实际 paused {health.get('paused')}、pause {tampered}")
    if not held_in_queue(world, held_id):
        problems.append(f"放行会话被改成另一个会话之后，它交的任务不该起，实际 {describe(world.summary(held_id))}")
    if problems:
        raise SelftestFailure("放行跨重启：" + "；".join(problems), world.server_log_lines())
    return "重启后照旧放行、放行会话新交的照起；库里的放行会话被改成另一个会话：按暂停算、不放行谁、写校验不过，另一个会话的任务排着"


def cell_pause_allow_reported(world):
    """放行了一个会话的暂停：control pause 点名放行的会话，--allow-session 不带值退 2；control check 与 /health 写「放行会话 <号>、在跑 N 个、别的会话在跑 M 个」，
    别的会话的任务跑完之后 M 变 0，N 里只剩放行会话的。"""
    world.resources(local=(30, 30))
    world.start_server()
    other_release = os.path.join(world.root, "other-release")
    allowed_release = os.path.join(world.root, "allowed-release")
    other_id = world.submit(blocking_command(other_release), session=SESSION_B)
    allowed_id = world.submit(blocking_command(allowed_release), session=SESSION_A)
    try:
        world.running(other_id)
        world.running(allowed_id)
        problems = []
        unfinished = world.control(["pause", "--allow-session"])
        if unfinished.returncode != lib.USAGE_EXIT:
            problems.append(f"--allow-session 不带值应当退 {lib.USAGE_EXIT}，实际退 {unfinished.returncode}")
        paused = world.control(["pause", "--reason", PAUSE_REASON_SAMPLE, "--allow-session", SESSION_A])
        wanted = f"放行会话 {SESSION_A}、在跑 2 个、别的会话在跑 1 个"
        if paused.returncode != 0 or "暂停了" not in paused.stdout or wanted not in paused.stdout:
            problems.append(f"control pause --allow-session 应当退 0、说暂停了、写「{wanted}」，实际退 {paused.returncode}、stdout「{paused.stdout.strip()[-300:]}」、"
                            f"stderr「{paused.stderr.strip()[-200:]}」")
        checked = world.control(["check"])
        if checked.returncode != 0 or wanted not in checked.stdout:
            problems.append(f"control check 应当退 0、写「{wanted}」，实际退 {checked.returncode}、stdout「{checked.stdout.strip()[-300:]}」")
        shown = pause_health(world).get("pause") or {}
        if shown.get("allow_session") != SESSION_A or shown.get("other_running") != 1:
            problems.append(f"/health 的 pause 应当带放行会话 {SESSION_A}、别的会话在跑 1 个，实际 {shown}")
        open(other_release, "w").close()
        world.finished(other_id)
        later = world.control(["check"])
        wanted = f"放行会话 {SESSION_A}、在跑 1 个、别的会话在跑 0 个"
        if later.returncode != 0 or wanted not in later.stdout:
            problems.append(f"别的会话的任务跑完之后 control check 应当写「{wanted}」，实际退 {later.returncode}、stdout「{later.stdout.strip()[-300:]}」")
    finally:
        open(other_release, "w").close()
        open(allowed_release, "w").close()
    world.finished(allowed_id)
    if problems:
        raise SelftestFailure("放行时的字样与计数：" + "；".join(problems), world.server_log_lines())
    return "control pause --allow-session 点名放行的会话、不带值退 2；check 与 /health 写放行会话与两种在跑数，别的会话的跑完后别的会话在跑 0 个"


# ── 调度让资源空着、任务干等的几处（records/2026-10-31-task-server巡检.md 批 66）：/health 判排头与派活同一个判法、一条整片只封一台、要卡的整片不为凑不够的卡封组、
#    封着的那台没有预计用时的小碎片照公式的余量插空；批 72：要卡的整片在卡被占着的台上照样封组、满时限的整片照样封组 ──
SEALED_ONE_HOST_FRAGMENTS = 30          # whole-seals-one-host：两台各 50G（界 40.8G）上交的碎片条数（没记录、各记 4G）：每台装满一组 10 条之后还有剩
SEALED_ONE_HOST_GROUP = 10              # 那一格一台一组至多装的碎片条数（10 × 4G = 40G 不超过界，再放一条就超过）：封了组的台起的不超过它
SEALED_ONE_HOST_RELEASE = "release-one-whole"   # 那一格碎片的放行文件（相对假仓根；假第二台照 fake-peer-wait 等同一个名字）
SEALED_ONE_HOST_SETTLE_SECONDS = 3.0    # 那一格每隔这么久数一次两台各起了几条（比第二台起满一轮之后再排的 PEER_REPASS_SECONDS 长）
SEALED_ONE_HOST_LIMIT_SECONDS = 90.0    # 那一格等两台起的条数不再变的绝对上限
LOCAL_CARD_TWO = f"{SELFTEST_LOCAL_ADDRESS}:2:8G"
CARDS_SHORT_HEAD_WANTED = 3   # health-head-cards-short-not-starved：排头要同一台的这么多张（本机三张、一张被占着）
CARD_WHOLE_PEAK_BYTES = 63 * GIBIBYTE   # card-whole-seals-while-card-busy 的整片：上限与记账 94.5G（100G 的台可用上限 98G，界 83.3G）；整片之外照样空着的那一份 3.5G
                                        # 放不下 4G 的短碎片，短碎片照预计用时插空（task_server.py 的 Alternation.gap_fill）
CARD_FRAGMENT_PEAK_BYTES = 10 * GIBIBYTE   # 那一格的碎片：各记 15G；五条 75G 装进一组，再放一条就 90G、超过界
CARD_FRAGMENT_COUNT = 6
CARD_OVERFLOW_KEY = "selftest-card-overflow"   # 那一格装不进组的最后一条碎片的命令键，与装组的五条（selftest-card-fragment）分开：同一把键的话，五条里先跑完的给它记下预计用时，
                                               # 它就按预计用时（不超过还在跑的挡路时长）插空进封着的组，合法地抢在整片前面起（task_server.py 的 Alternation.gap_fill），不是封组失效
CARD_GAP_COMMAND = ["bash", "-c", "exit 0", "card-gap"]   # 那一格来插空的短碎片：记了 GAP_SHORT_ESTIMATE_SECONDS 秒的用时（逐词相同才认），没有峰值记录、记 4G
ROOM_UNESTIMATED_WHOLE_PEAK_BYTES = 34 * GIBIBYTE   # sealed-gap-room-takes-unestimated 的整片：上限与记账 51G（60G 的台可用上限 58G，界 49.3G）
ROOM_UNESTIMATED_FILLER_PEAK_BYTES = 8 * GIBIBYTE   # 那一格先装进组的三条碎片：各记 12G（三条与在跑的小碎片 37.5G，界以内）
ROOM_UNESTIMATED_SEALED_PEAK_BYTES = 10 * GIBIBYTE  # 那一格封组的那条：记 15G。37.5G + 15G = 52.5G 超过界 49.3G 而封组，照公式（58G − 37.5G = 20.5G）内存上放得下：
                                                    # 内存放不下的排队原因写内存那一句、不写封组（judge_placement 只在内存放得下时换成交替那一句；任务 7648 里各记 15G，46.5G + 15G 超过 58G）
ROOM_UNESTIMATED_SMALL_PEAK_BYTES = GIBIBYTE        # 那一格一直在跑的那条小碎片与来插空的那条：各记 1536M（有记录的不兜到 4G）
ROOM_UNESTIMATED_FREE_GIBIBYTES = 20                # 装组的三条跑完之前把本机空闲换成这么多：整片（51G）放不下，小碎片放得下


def cell_health_head_cpu_gate_not_starved(world):
    """/health 只读派活给排头的结论（task_server.py 的 starved_problem 读 head_verdict；任务 7568 卡在 CPU 负载闸被报「放得下却没起」）：本机内存够、负载 200
    （32 核 × 3 = 96 放不下再加一条的线程），排头排着、排队原因写「CPU：」；过了阈值四倍那么久 /health 一直回 ok：派活判它不能起，不报；
    看进展（progress_problem）照余量也不报（这台负载过 CPU 闸）。负载降下来之后跑完。
    弄坏开关 health-head-judged-apart、health-judges-head 下 /health 照改前另判、只判内存，blocked-head-counted-as-starved 下判不能起也算，
    progress-gates-ignored 下看进展不看 CPU 闸，/health 都报，这一格红。"""
    world.resources(local=(50, 50), local_cpu={"load1": 200.0, "procs_running": 150, "cpus": 32})
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "cpu-gate-head"])
    waiting = world.wait_for(task_id, lambda found: "CPU：" in (found.get("waiting_for") or "") or found["task"].get("state") != "queued", "报 CPU 闸")
    health = health_until(world, lambda found: not found.get("ok"), 4 * SERVER.FAST_STARVED_SECONDS)
    world.resources(local=(50, 50), local_cpu={"load1": 0.0, "procs_running": 1, "cpus": 32})
    summary = world.finished(task_id)
    problems = []
    if waiting["attempts"] or "CPU：" not in (waiting.get("waiting_for") or ""):
        problems.append(f"负载 200 的本机上排头应当排着、排队原因写「CPU：」，实际 {describe(waiting)}、在等「{waiting.get('waiting_for')}」")
    if not health.get("ok"):
        problems.append(f"CPU 闸挡着时排头排着是正常的，/health 应当一直回 ok，实际 {health}")
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"负载降下来之后应当跑完，实际 {describe(summary)}")
    if problems:
        raise SelftestFailure("CPU 闸挡着的排头：" + "；".join(problems), world.server_log_lines())
    return f"CPU 闸挡着的排头：/health 一直回 ok（排队原因「{(waiting.get('waiting_for') or '')[:80]}」），负载降下来之后跑完"


def cell_health_head_cards_short_not_starved(world):
    """/health 只读派活给排头的结论（task_server.py 的 starved_problem 读 head_verdict；任务 7615 要同一台 3 张卡而凑不出）：本机三张卡、一条要一张的占着本机 0，
    排头声明要同一台的 3 张：排着、排队原因写「等空卡」；/health 一直回 ok（派活判它不能起；看进展照余量也不报：这台空卡凑不够它的张数）；占着的放了之后它跑完。
    弄坏开关 health-head-judged-apart、health-judges-head 下 /health 照改前另判、有一张空卡就算，blocked-head-counted-as-starved 下判不能起也算，
    progress-gates-ignored 下看进展不看卡，/health 都报，这一格红。"""
    world.resources(local=(100, 100), cards=f"{LOCAL_CARD_ZERO} {LOCAL_CARD_ONE} {LOCAL_CARD_TWO}",
                    local_gpu=[(index, CARD_FREE_MEBIBYTES) for index in range(CARDS_SHORT_HEAD_WANTED)])
    world.start_server()
    release = os.path.join(world.root, "release-cards-short")
    holder = world.submit(["cargo", "test", "--features", "gpu", f"fake-quiesce=block:{release}"])
    problems = []
    try:
        world.running(holder)
        head = world.submit(write_card_script(world, "three-cards-head.sh", CARDS_SHORT_HEAD_WANTED))
        waiting = world.wait_for(head, lambda found: "等空卡" in (found.get("waiting_for") or "") or found["task"].get("state") != "queued", "报等空卡")
        health = health_until(world, lambda found: not found.get("ok"), 4 * SERVER.FAST_STARVED_SECONDS)
    finally:
        open(release, "w").close()
    summary = world.finished(head)
    world.finished(holder)
    if waiting["attempts"] or "等空卡" not in (waiting.get("waiting_for") or ""):
        problems.append(f"只剩两张空卡时要 {CARDS_SHORT_HEAD_WANTED} 张的排头应当排着、排队原因写「等空卡」，实际 {describe(waiting)}、在等「{waiting.get('waiting_for')}」")
    if not health.get("ok"):
        problems.append(f"空卡凑不够张数时排头排着是正常的，/health 应当一直回 ok，实际 {health}")
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"占着的放了之后要 {CARDS_SHORT_HEAD_WANTED} 张的应当跑完，实际 {describe(summary)}")
    if problems:
        raise SelftestFailure("空卡凑不够排头的张数：" + "；".join(problems), world.server_log_lines())
    return f"空卡凑不够排头要的同一台 {CARDS_SHORT_HEAD_WANTED} 张：/health 一直回 ok，卡放出来之后跑完"


def started_hosts(world, task_ids):
    """这几条里起了的各在哪一台：{台: 条数}；另交没起的那几条的排队原因。"""
    counts, reasons = {}, []
    for task_id in task_ids:
        summary = world.summary(task_id)
        if summary["attempts"]:
            host = first_attempt(summary).get("host")
            counts[host] = counts.get(host, 0) + 1
        else:
            reasons.append(summary.get("waiting_for") or "")
    return counts, reasons


def cell_whole_seals_one_host(world):
    """一条整片只在一台上等腾空、只封那一台（task_server.py 的 Alternation.waiting_hosts；任务 7585 两台都封组）：两台各 50G（可用上限 48G、界 40.8G），
    一条整片（峰值 30G，上限与记账 45G）在排、两台都放不下它，再交 30 条碎片（没记录、各记 4G，跑到放行为止）。等两台起的条数不再变：
    至多一台的排队原因里封了组，另一台照起碎片、起的超过一组的 10 条（照内存装）。弄坏开关 whole-seals-every-host 下两台都封、每台至多 10 条，这一格红。"""
    world.resources(local=(50, 50), peer=(50, 50), peer_cap_gibibytes=50, hold=True)
    world.seed_peak("selftest-one-whole", 30 * GIBIBYTE)
    world.start_server()
    whole = world.submit(["bash", "-c", "exit 0", "one-whole"], extra={lib.KEY_VARIABLE: "selftest-one-whole"}, cwd=world.repo)
    fragments = [world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, f"one-whole-fragment-{number}", SEALED_ONE_HOST_RELEASE,
                               f"fake-peer-wait={SEALED_ONE_HOST_RELEASE}"], cwd=world.repo) for number in range(SEALED_ONE_HOST_FRAGMENTS)]
    world.resources(local=(50, 50), peer=(50, 50), peer_cap_gibibytes=50)   # 一齐放出来
    world.api("POST", "/wake")
    counts, reasons, previous = {}, [], None
    deadline = time.monotonic() + SEALED_ONE_HOST_LIMIT_SECONDS
    try:
        while time.monotonic() < deadline:
            world.next_pass()
            time.sleep(SEALED_ONE_HOST_SETTLE_SECONDS)
            counts, reasons = started_hosts(world, fragments)
            settled = all(reason and "这一轮已起" not in reason for reason in reasons)
            if settled and counts == previous:
                break
            previous = counts
        whole_summary = world.summary(whole)
    finally:
        open(os.path.join(world.repo, SEALED_ONE_HOST_RELEASE), "w").close()
    outcomes = [world.finished(task_id, timeout=120)["task"].get("outcome") for task_id in [*fragments, whole]]
    sealed = sorted(name for name in lib.HOST_NAMES.values() if any(f"{name}：这台这一组碎片已装到" in reason for reason in reasons))
    problems = []
    if counts != previous:
        problems.append(f"{SEALED_ONE_HOST_LIMIT_SECONDS:.0f} 秒里两台起的条数一直在变：最后一次 {counts}，前一次 {previous}")
    if whole_summary["attempts"]:
        problems.append(f"两台都放不下整片（45G）时它应当一直排着，实际 {describe(whole_summary)}")
    if len(sealed) > 1:
        problems.append(f"一条整片应当只封一台，实际排队原因里封了组的台 {sealed}")
    if max(counts.values(), default=0) <= SEALED_ONE_HOST_GROUP:
        problems.append(f"没封组的那台应当照内存起碎片、超过一组的 {SEALED_ONE_HOST_GROUP} 条，实际两台各起了 {counts}")
    if set(outcomes) != {"succeeded"}:
        problems.append(f"放行之后都应当跑完，实际结局 {sorted(set(outcomes))}")
    if problems:
        raise SelftestFailure("一条整片只封一台：" + "；".join(problems), reasons[:4] + world.server_log_lines())
    return f"一条整片两台都放不下：封了组的台 {sealed or '无'}，两台各起碎片 {counts}"


def cell_card_whole_seals_while_card_busy(world):
    """要卡的整片在卡被占着的台上照样封组（task_server.py 的 Alternation.waiting_hosts 照那台卡的总张数算；任务 7934 要卡、记 45.6 GiB，两台的卡一直有人占，
    照空卡数算哪台都不算整片、从不封组，排了一个多钟头）：本机 100G（可用上限 98G、界 83.3G）一张卡，一条要卡的小任务占着它；一条要卡的整片（峰值 63G，
    上限与记账 94.5G）在等，再交六条碎片（峰值 10G，各记 15G，跑到放行为止；最后一条另用一把命令键 CARD_OVERFLOW_KEY）与一条记了 1 秒用时的短碎片：
    五条装进一组，最后一条再放就超过界、封组排着；短碎片照插空起来跑完（挡路的还在跑）。放行占卡的与五条碎片之后，整片先于最后一条起，都跑完。
    弄坏开关 card-whole-counts-free-cards-only 下照改前只数空卡、不为它封组，最后一条与短碎片照内存起，这一格红。"""
    cards = {"cards": LOCAL_CARD_ZERO, "local_gpu": [(0, CARD_FREE_MEBIBYTES)]}
    world.resources(local=(100, 100), **cards)
    world.seed_peak("selftest-card-whole", CARD_WHOLE_PEAK_BYTES)
    world.seed_peak("selftest-card-fragment", CARD_FRAGMENT_PEAK_BYTES)
    world.seed_peak(CARD_OVERFLOW_KEY, CARD_FRAGMENT_PEAK_BYTES)
    world.seed_duration("selftest-card-gap", GAP_SHORT_ESTIMATE_SECONDS, CARD_GAP_COMMAND)
    world.start_server()
    holder_release = os.path.join(world.root, "release-card-holder")
    fragment_release = os.path.join(world.root, "release-card-fragments")
    holder = world.submit(["cargo", "test", "--features", "gpu", f"fake-quiesce=block:{holder_release}"])
    fragments, whole, gap, overflow, counts, gapped, blockers_running = [], None, None, {}, {}, {}, False
    try:
        world.running(holder)
        world.resources(local=(100, 100), hold=True, **cards)
        # 整片照假清场的 block 演法收尾（放行文件在就退 0）：不写演法的，假清场照真的那样起命令，假内存包装不放行真 cargo、退 97，整片记 command-failed（任务 7648）
        whole = world.submit(["cargo", "test", "--features", "gpu", "--lib", f"fake-quiesce=block:{holder_release}", "card-whole"],
                             extra={lib.KEY_VARIABLE: "selftest-card-whole"})
        fragments = [world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, f"card-fragment-{number}", fragment_release],
                                  extra={lib.KEY_VARIABLE: "selftest-card-fragment" if number < CARD_FRAGMENT_COUNT - 1 else CARD_OVERFLOW_KEY})
                     for number in range(CARD_FRAGMENT_COUNT)]
        gap = world.submit(CARD_GAP_COMMAND, extra={lib.KEY_VARIABLE: "selftest-card-gap"})
        world.resources(local=(100, 100), **cards)   # 一齐放出来
        world.api("POST", "/wake")
        overflow = world.wait_for(fragments[-1], lambda found: bool(found["attempts"]) or SEALED_GROUP_TEXT in (found.get("waiting_for") or ""),
                                  "起来或封组", timeout=PLACEMENT_WAIT_SECONDS)
        gapped = world.finished(gap, timeout=PLACEMENT_WAIT_SECONDS)   # 挡路的已跑过 1 秒就插得进（挡路时长取已跑）
        counts, _ = started_hosts(world, fragments[:-1])
        blockers_running = all(world.summary(task_id)["task"].get("state") == "running" for task_id in fragments[:-1])
        overflow = world.summary(fragments[-1])
        open(holder_release, "w").close()   # 卡空出来；整片照旧等内存（五条碎片 75G 还在）
        world.finished(holder)
    finally:
        open(holder_release, "w").close()
        open(fragment_release, "w").close()
    summaries = {task_id: world.finished(task_id, timeout=120) for task_id in [holder, whole, *fragments] if task_id is not None}
    problems = []
    if sum(counts.values()) != CARD_FRAGMENT_COUNT - 1 or overflow.get("attempts"):
        problems.append(f"卡被占着时整片照样封组：五条碎片应当起、装不进组的最后一条封组排着，实际五条里起了 {counts}、最后一条 {describe(overflow)}、在等「{overflow.get('waiting_for')}」")
    elif SEALED_GROUP_TEXT not in (overflow.get("waiting_for") or ""):
        problems.append(f"装不进组的最后一条的排队原因应当写封组（「{SEALED_GROUP_TEXT}」），实际「{overflow.get('waiting_for')}」")
    problems += [problem for problem in (gap_admitted_problem("记了 1 秒用时的短碎片", gapped, blockers_running),) if problem]
    last_summary = summaries[fragments[-1]]
    whole_order, last_order = attempt_order(summaries[whole]), attempt_order(last_summary)
    if whole_order is None or last_order is None or whole_order > last_order:
        problems.append(f"卡空出来、五条碎片跑完之后整片应当先于装不进组的最后一条起，实际尝试号整片 {whole_order}、最后一条 {last_order}"
                        f"（最后一条的 gap_fill「{(last_summary['attempts'] or [{}])[0].get('gap_fill') or ''}」：非空就是它按预计用时插空抢先了）")
    if {summary["task"].get("outcome") for summary in summaries.values()} != {"succeeded"}:
        problems.append(f"放行之后都应当跑完，实际结局 {sorted({str(summary['task'].get('outcome')) for summary in summaries.values()})}")
    if problems:
        raise SelftestFailure("要卡的整片在卡被占着的台上封组：" + "；".join(problems), world.server_log_lines())
    return f"卡被占着时整片照样封组：五条碎片起、装不进组的最后一条封着，短碎片插空跑完；卡空出来、碎片跑完之后整片（尝试 {whole_order}）先于最后一条（尝试 {last_order}）起"


AGED_SEAL_OCCUPANT_PEAK_BYTES = 27 * GIBIBYTE   # aged-whole-keeps-group-sealed 占着的碎片：上限与记账 40.5G（50G 的台可用上限 48G、界 40.8G：碎片），起了记进这一组
AGED_SEAL_WHOLE_PEAK_BYTES = 30 * GIBIBYTE      # 那一格的整片：上限与记账 45G，占着的跑着时放不下（照公式 48G − 40.5G = 7.5G）
AGED_SEAL_FRAGMENT_COUNT = 3                    # 那一格整片之后交的碎片（没记录、各记 4G）：内存上放得下一条，只因封组排着
AGED_SEAL_SECONDS = "2"                         # 那一格的满时限（TASK_SERVER_SELFTEST_AGED_SECONDS）：交了之后整片与碎片很快都满时限


def cell_aged_whole_keeps_group_sealed(world):
    """满时限的整片照样封组等它（task_server.py 文件头 ②「满时限」；任务 7934 记 45.6 GiB，满两小时不算整片的话封着的组撤掉，碎片又把内存占满，它永远起不来）：
    本机 50G（可用上限 48G、界 40.8G），一条碎片占着 40.5G（记进这一组）；一条整片（记 45G）放不下、在等，再交三条碎片（各记 4G：内存上放得下一条）：封组排着。
    满时限压到 2 秒，等整片与碎片都满时限之后再排两轮：组照封着、碎片一条不起（满时限的碎片照样受排在它前面的整片封的组挡）；放行占着的那条之后整片先起，碎片在它之后，都跑完。
    弄坏开关 aged-whole-not-counted 下满时限的整片不算整片、组撤掉，aged-fragment-bypasses-seal 下满时限的碎片不过交替，碎片都照内存起，这一格红。"""
    world.resources(local=(50, 50))
    world.seed_peak("selftest-aged-seal-occupant", AGED_SEAL_OCCUPANT_PEAK_BYTES)
    world.seed_peak("selftest-aged-seal-whole", AGED_SEAL_WHOLE_PEAK_BYTES)
    world.start_server(extra={"TASK_SERVER_SELFTEST_AGED_SECONDS": AGED_SEAL_SECONDS})
    release = os.path.join(world.root, "release-aged-seal-occupant")
    occupant = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "aged-seal-occupant", release], extra={lib.KEY_VARIABLE: "selftest-aged-seal-occupant"})
    whole, fragments, waiting = None, [], {}
    try:
        world.running(occupant)
        whole = world.submit(["bash", "-c", "exit 0", "aged-seal-whole"], extra={lib.KEY_VARIABLE: "selftest-aged-seal-whole"})
        fragments = [world.submit(["bash", "-c", "exit 0", f"aged-seal-fragment-{number}"]) for number in range(AGED_SEAL_FRAGMENT_COUNT)]
        time.sleep(float(AGED_SEAL_SECONDS) + 1)   # 最后交的那条也满了时限（下限，不比上限：机器忙时只会更久）
        world.next_pass()
        world.next_pass()
        waiting = {task_id: world.summary(task_id) for task_id in [whole, *fragments]}
    finally:
        open(release, "w").close()
    summaries = {task_id: world.finished(task_id) for task_id in [occupant, whole, *fragments] if task_id is not None}
    problems = []
    started_early = [task_id for task_id in fragments if waiting.get(task_id, {}).get("attempts")]
    if started_early or waiting.get(whole, {}).get("attempts"):
        problems.append(f"占着的跑着时整片与碎片都满了时限，组应当照封着、一条都不起，实际起了的碎片 {started_early}、整片 {describe(waiting.get(whole) or {'task': {}, 'attempts': []})}")
    unsealed = [task_id for task_id in fragments if task_id not in started_early and SEALED_GROUP_TEXT not in (waiting.get(task_id, {}).get("waiting_for") or "")]
    if unsealed:
        problems.append(f"满时限的碎片排队原因应当写封组（「{SEALED_GROUP_TEXT}」），实际 {[(task_id, waiting[task_id].get('waiting_for')) for task_id in unsealed]}")
    whole_order = attempt_order(summaries[whole])
    later = [task_id for task_id in fragments
             if whole_order is None or attempt_order(summaries[task_id]) is None or attempt_order(summaries[task_id]) < whole_order]
    if later:
        problems.append(f"放行占着的那条之后整片应当先于碎片起，实际整片尝试号 {whole_order}、比它先起的碎片 {later}")
    if {summary["task"].get("outcome") for summary in summaries.values()} != {"succeeded"}:
        problems.append(f"放行之后都应当跑完，实际结局 {sorted({str(summary['task'].get('outcome')) for summary in summaries.values()})}")
    if problems:
        raise SelftestFailure("满时限的整片照样封组：" + "；".join(problems), world.server_log_lines())
    return f"整片与 {AGED_SEAL_FRAGMENT_COUNT} 条碎片都满时限之后组照封着、碎片不起；占着的放了之后整片（尝试 {whole_order}）先起，都跑完"


def cell_sealed_gap_room_takes_unestimated(world):
    """封着的那台没有预计用时的小碎片照公式的余量插空（task_server.py 的 Alternation.gap_fill 的内存那一支）：本机 60G（可用上限 58G、界 49.3G），
    一条整片（峰值 34G，上限与记账 51G）、三条装组的碎片（各记 12G，跑到放行为止）、一条一直在跑的小碎片（1536M）、一条封组的碎片（15G，没有预计用时：
    装进那一组就 52.5G、超过界，内存照公式还放得下，排队原因写封组那一句）、一条来插空的小碎片（1536M，没有预计用时）。装组的跑完之前把本机空闲换成 20G（整片放不下），放行装组的：排头整片起来之后照公式这台还剩 58G − 1.5G − 51G = 5.5G，
    放得下来插空的那条，不看预计用时就起、跑完，尝试记插空那一句；封组的那条（15G）照旧封着。弄坏开关 gap-fill-room-ignored 下没有预计用时的一律封着，这一格红。
    整片之外照样空着的那一份是 58G − 51G = 7G（在跑的那条小碎片是封组之前起的，不减）：来插空的那条装组的还在跑时就可能起，照样在小碎片还在跑时跑完。"""
    world.resources(local=(60, 60), hold=True)
    for key, peak in (("selftest-room-unestimated-whole", ROOM_UNESTIMATED_WHOLE_PEAK_BYTES), ("selftest-room-unestimated-filler", ROOM_UNESTIMATED_FILLER_PEAK_BYTES),
                      ("selftest-room-unestimated-holder", ROOM_UNESTIMATED_SMALL_PEAK_BYTES), ("selftest-room-unestimated-sealed", ROOM_UNESTIMATED_SEALED_PEAK_BYTES),
                      ("selftest-room-unestimated-probe", ROOM_UNESTIMATED_SMALL_PEAK_BYTES)):
        world.seed_peak(key, peak)
    world.start_server()
    release = os.path.join(world.root, "release-room-unestimated")
    filler_release = os.path.join(world.root, "release-room-unestimated-fillers")
    tasks = {"整片": world.submit(["bash", "-c", "exit 0", "room-unestimated-whole"], extra={lib.KEY_VARIABLE: "selftest-room-unestimated-whole"})}
    tasks["装组"] = [world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, f"room-unestimated-filler-{number}", filler_release],
                                extra={lib.KEY_VARIABLE: "selftest-room-unestimated-filler"}) for number in range(3)]
    tasks["在跑"] = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "room-unestimated-holder", release], extra={lib.KEY_VARIABLE: "selftest-room-unestimated-holder"})
    tasks["封组"] = world.submit(["bash", "-c", "exit 0", "room-unestimated-sealed"], extra={lib.KEY_VARIABLE: "selftest-room-unestimated-sealed"})
    tasks["插空"] = world.submit(["bash", "-c", "exit 0", "room-unestimated-probe"], extra={lib.KEY_VARIABLE: "selftest-room-unestimated-probe"})
    world.resources(local=(60, 60))   # 一齐放出来：整片让一组碎片先起，三条装组的与在跑的那条起了，封组的那条装进去就超过界
    world.api("POST", "/wake")
    problems = []
    try:
        for task_id in [*tasks["装组"], tasks["在跑"]]:
            world.running(task_id)
        world.wait_for(tasks["封组"], lambda found: SEALED_GROUP_TEXT in (found.get("waiting_for") or "") or bool(found["attempts"]), "封组")
        world.resources(local=(ROOM_UNESTIMATED_FREE_GIBIBYTES, 60))
        open(filler_release, "w").close()
        for task_id in tasks["装组"]:
            world.finished(task_id)
        try:
            probe = world.finished(tasks["插空"], timeout=PLACEMENT_WAIT_SECONDS)
        except SelftestFailure:
            probe = world.summary(tasks["插空"])
        holder_running = world.summary(tasks["在跑"])["task"].get("state") == "running"
        world.next_pass()
        sealed = world.summary(tasks["封组"])
    finally:
        open(filler_release, "w").close()
        open(release, "w").close()
    world.resources(local=(60, 60))
    for task_id in (tasks["在跑"], tasks["封组"], tasks["整片"]):
        world.finished(task_id, timeout=120)
    note = first_attempt(probe).get("gap_fill") or ""
    if (probe["task"].get("outcome") != "succeeded" or not note.startswith(GAP_ADMITTED_TEXT) or ROOM_GAP_ADMITTED_TEXT not in note or not holder_running):
        problems.append(f"没有预计用时的小碎片应当在那条小的还在跑时照公式的余量插空起来、跑完，尝试的 gap_fill 以「{GAP_ADMITTED_TEXT}」起头、写「{ROOM_GAP_ADMITTED_TEXT}」，"
                        f"实际 {describe(probe)}、gap_fill「{note}」、在跑的那条{'还在跑' if holder_running else '已不在跑'}、在等「{probe.get('waiting_for')}」")
    problem = sealed_reason_problem("封组的那条（15G，没有预计用时）", sealed, f"{GAP_REFUSED_TEXT}这一条没有预计用时")
    problems += [problem] if problem else []
    if problems:
        raise SelftestFailure("封着的那台没有预计用时的小碎片照余量插空：" + "；".join(problems), world.server_log_lines())
    return f"封着的那台：没有预计用时的 1536M 碎片照公式的余量插空跑完（「{note[:60]}…」），15G 的那条照旧封着"


# ── 批 77（records/2026-10-31-task-server巡检.md）：两台都封着组时没有预计用时的小任务照整片之外照样空着的那一份插空；组的账跑完的跟着减、插空起的不进 ──
BOTH_SEALED_PEER_GIBIBYTES = 64                      # both-hosts-sealed-small-unestimated-starts：第二台 64G（可用上限 62G、界 52.7G）；本机 50G（可用上限 48G、界 40.8G）
BOTH_SEALED_CARD_WHOLE_PEAK_BYTES = 30 * GIBIBYTE    # 那一格要卡的整片：上限与记账 45G；卡只在本机，只在本机算整片、只封本机，整片之外本机照样空着 3G
BOTH_SEALED_WHOLE_PEAK_BYTES = 36 * GIBIBYTE         # 那一格不要卡的整片：上限 54G（本机收到 48G），两台都算整片；一齐放出来时第二台离放得下最近，封第二台，
                                                     # 整片之外第二台照样空着 8G
BOTH_SEALED_HOLDER_PEAK_BYTES = 10 * GIBIBYTE        # 那一格的碎片：各记 15G、跑到放行为止；本机装两条（30G）、第二台装三条（45G）之后再放一条都超过界，两台都封组
BOTH_SEALED_HOLDER_COUNT = 6                         # 碎片条数：两台至多装五条，至少剩一条两台都封着排着
BOTH_SEALED_PROBE_PEAK_BYTES = GIBIBYTE * 2 // 3     # 那一格来插空的小任务：上限与记账 1G、没有预计用时（任务 8370 那一条只补写一行完成）
BOTH_SEALED_RELEASE = "release-both-sealed"          # 那一格碎片的放行文件（相对假仓根；假第二台照 fake-peer-wait 等同一个名字）
GROUP_DROP_WHOLE_PEAK_BYTES = 30 * GIBIBYTE          # fragment-group-drops-finished 的整片：上限与记账 45G（50G 的台可用上限 48G、界 40.8G），两条碎片在跑时放不下
GROUP_DROP_HOLDER_PEAK_BYTES = 10 * GIBIBYTE         # 那一格的两条碎片与封组的那条：各记 15G；两条装进一组 30G，封组的那条再放就 45G、超过界
GROUP_DROP_PROBE_PEAK_BYTES = GIBIBYTE * 2 // 3      # 那一格来插空的小任务：上限与记账 1G，整片之外照样空着的 3G 放得下，照内存那一支插空
# task_server.py 的 Alternation.blocked 写的封组那一句里组的账那一段：已装到多少、此刻还在跑几条、它们记多少
GROUP_ACCOUNT_PATTERN = re.compile(r"这台这一组碎片已装到 ([^（]+)（界 [^；]+；装过的照算，此刻还在跑的 (\d+) 条记 ([^）]+)）")


def cell_both_hosts_sealed_small_unestimated_starts(world):
    """两台都封着组时没有预计用时的小任务起得来（task_server.py 的 Alternation.gap_fill 的内存那一支；任务 8370 两台都封着组、一条只补写一行完成的 hook-swap 一直起不来）：
    本机 50G 一张卡、第二台 64G 没有卡；一条要卡的整片（45G，只在本机算整片）、一条不要卡的整片（54G，等第二台）、六条碎片（各 15G，跑到放行为止）、
    一条小任务（1G，没有预计用时）一齐放出来：本机装两条碎片、第二台装三条之后两台都封组，剩下的碎片排着、排队原因两台都写封组；
    小任务照两台整片之外照样空着的那一份（本机 48G − 45G、第二台 62G − 54G，封组之前起的碎片不减）插空起来、跑完，尝试记「不看预计用时」与 gap_fill_beside_whole，
    碎片都还在跑。放行之后都跑完。弄坏开关 gap-fill-room-counts-all-running 下照改前连碎片一起减，两台都只剩 0、小任务封着，这一格红。"""
    cards = {"cards": LOCAL_CARD_ZERO, "local_gpu": [(0, CARD_FREE_MEBIBYTES)]}
    sizes = {"local": (50, 50), "peer": (BOTH_SEALED_PEER_GIBIBYTES, BOTH_SEALED_PEER_GIBIBYTES), "peer_cap_gibibytes": BOTH_SEALED_PEER_GIBIBYTES}
    world.resources(hold=True, **sizes, **cards)
    for key, peak in (("selftest-both-sealed-card-whole", BOTH_SEALED_CARD_WHOLE_PEAK_BYTES), ("selftest-both-sealed-whole", BOTH_SEALED_WHOLE_PEAK_BYTES),
                      ("selftest-both-sealed-holder", BOTH_SEALED_HOLDER_PEAK_BYTES), ("selftest-both-sealed-probe", BOTH_SEALED_PROBE_PEAK_BYTES)):
        world.seed_peak(key, peak)
    world.start_server()
    release = os.path.join(world.repo, BOTH_SEALED_RELEASE)
    # 要卡的整片照假清场的 block 演法收尾（放行文件在就退 0；card-whole-seals-while-card-busy 同一写法）
    card_whole = world.submit(["cargo", "test", "--features", "gpu", "--lib", f"fake-quiesce=block:{release}", "both-sealed-card-whole"],
                              extra={lib.KEY_VARIABLE: "selftest-both-sealed-card-whole"})
    whole = world.submit(["bash", "-c", "exit 0", "both-sealed-whole"], extra={lib.KEY_VARIABLE: "selftest-both-sealed-whole"}, cwd=world.repo)
    holders = [world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, f"both-sealed-holder-{number}", BOTH_SEALED_RELEASE, f"fake-peer-wait={BOTH_SEALED_RELEASE}"],
                            extra={lib.KEY_VARIABLE: "selftest-both-sealed-holder"}, cwd=world.repo) for number in range(BOTH_SEALED_HOLDER_COUNT)]
    probe = world.submit(["bash", "-c", "exit 0", "both-sealed-probe"], extra={lib.KEY_VARIABLE: "selftest-both-sealed-probe"}, cwd=world.repo)
    world.resources(**sizes, **cards)   # 一齐放出来
    world.api("POST", "/wake")
    reasons, counts, wholes_waiting = [], {}, False
    try:
        try:
            probe_summary = world.wait_for(probe, lambda found: found["task"].get("state") == "finished", "插空起来跑完", timeout=PLACEMENT_WAIT_SECONDS)
        except SelftestFailure:
            probe_summary = world.summary(probe)
        world.next_pass()
        counts, reasons = started_hosts(world, holders)
        wholes_waiting = not world.summary(card_whole)["attempts"] and not world.summary(whole)["attempts"]
    finally:
        open(release, "w").close()
    summaries = {task_id: world.finished(task_id, timeout=120) for task_id in [card_whole, whole, *holders, probe]}
    problems = []
    sealed = sorted(name for name in lib.HOST_NAMES.values() if any(f"{name}：这台这一组碎片已装到" in reason for reason in reasons))
    if sealed != sorted(lib.HOST_NAMES.values()) or not wholes_waiting:
        problems.append(f"两台都应当封着组、两条整片都在等（这一格的前提），实际排着的碎片的排队原因里封了组的台 {sealed}、两台各起碎片 {counts}、"
                        f"整片{'都在等' if wholes_waiting else '有起了的'}、排队原因 {reasons[:2]}")
    note = first_attempt(probe_summary).get("gap_fill") or ""
    if (probe_summary["task"].get("outcome") != "succeeded" or not note.startswith(GAP_ADMITTED_TEXT) or ROOM_GAP_ADMITTED_TEXT not in note
            or first_attempt(probe_summary).get("gap_fill_beside_whole") is not True or sum(counts.values()) != BOTH_SEALED_HOLDER_COUNT - 1):
        problems.append(f"没有预计用时的 1G 小任务应当在五条碎片都还在跑时照整片之外照样空着的那一份插空起来、跑完，gap_fill 以「{GAP_ADMITTED_TEXT}」起头、写「{ROOM_GAP_ADMITTED_TEXT}」、"
                        f"gap_fill_beside_whole 记 True，实际 {describe(probe_summary)}、gap_fill「{note}」、"
                        f"gap_fill_beside_whole {first_attempt(probe_summary).get('gap_fill_beside_whole')}、起了的碎片 {counts}、在等「{probe_summary.get('waiting_for')}」")
    if {summary["task"].get("outcome") for summary in summaries.values()} != {"succeeded"}:
        problems.append(f"放行之后都应当跑完，实际结局 {sorted({str(summary['task'].get('outcome')) for summary in summaries.values()})}")
    if problems:
        raise SelftestFailure("两台都封着组时没有预计用时的小任务插空：" + "；".join(problems), world.server_log_lines())
    return f"两台都封着组（碎片 {counts}）：没有预计用时的 1G 小任务照整片之外照样空着的那一份插空跑完（「{note[:60]}…」）"


def group_account(summary):
    """封组那一句里组的账那一段（GROUP_ACCOUNT_PATTERN）：(已装到, 还在跑的条数, 它们记多少)；排队原因里没有交 None。"""
    found = GROUP_ACCOUNT_PATTERN.search(summary.get("waiting_for") or "")
    return found.groups() if found else None


def cell_fragment_group_drops_finished(world):
    """组的账：插空起的不进、这一组还在跑的碎片跑完跟着减（task_server.py 的 Alternation.__init__、started；任务 8370 那一回本机「已装到 95.1 GiB（界 41.6 GiB）」比 slice 还大）：
    本机 50G（界 40.8G）上一条整片（45G）在等，两条碎片（各 15G，各自跑到自己的放行文件为止）装进一组，封组的那条（15G，没有预计用时）排着，
    一条 1G 的小任务照整片之外照样空着的 3G 插空起来、跑完。封组那一句写「已装到 X（…；装过的照算，此刻还在跑的 2 条记 X）」：插空的那条不加进已装到的数；
    放行一条碎片、它跑完之后写「还在跑的 1 条」、记的数跟着减，已装到的数不变（装过的照算），整片照旧在等。放行之后都跑完。
    弄坏开关 fragment-group-never-drops 下插空的那条加进已装到的数、跑完的碎片照留在还在跑的里，这一格红。"""
    world.resources(local=(50, 50), hold=True)
    for key, peak in (("selftest-group-drop-whole", GROUP_DROP_WHOLE_PEAK_BYTES), ("selftest-group-drop-holder", GROUP_DROP_HOLDER_PEAK_BYTES),
                      ("selftest-group-drop-sealed", GROUP_DROP_HOLDER_PEAK_BYTES), ("selftest-group-drop-probe", GROUP_DROP_PROBE_PEAK_BYTES)):
        world.seed_peak(key, peak)
    world.start_server()
    releases = [os.path.join(world.root, f"release-group-drop-{name}") for name in ("released-midway", "kept-running")]   # 中途放行的那条、跑到收尾的那条
    whole = world.submit(["bash", "-c", "exit 0", "group-drop-whole"], extra={lib.KEY_VARIABLE: "selftest-group-drop-whole"})
    holders = [world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, f"group-drop-holder-{number}", path], extra={lib.KEY_VARIABLE: "selftest-group-drop-holder"})
               for number, path in enumerate(releases)]
    # 封组的那条另用一把键：与碎片同键的话，先跑完的那条给它记下预计用时，它就照时长插空起来，排队原因里读不到组的账
    sealed = world.submit(["bash", "-c", "exit 0", "group-drop-sealed"], extra={lib.KEY_VARIABLE: "selftest-group-drop-sealed"})
    probe = world.submit(["bash", "-c", "exit 0", "group-drop-probe"], extra={lib.KEY_VARIABLE: "selftest-group-drop-probe"})
    world.resources(local=(50, 50))   # 一齐放出来
    world.api("POST", "/wake")
    before = after = None
    whole_waiting = False
    try:
        for task_id in holders:
            world.running(task_id)
        try:
            probe_summary = world.finished(probe, timeout=PLACEMENT_WAIT_SECONDS)
        except SelftestFailure:
            probe_summary = world.summary(probe)
        world.next_pass()
        world.next_pass()
        before = group_account(world.summary(sealed))
        midway_release, midway_holder = releases[0], holders[0]   # 中途放行的那条（releases 与 holders 同一个次序）
        open(midway_release, "w").close()
        world.finished(midway_holder)
        world.next_pass()
        world.next_pass()
        after = group_account(world.summary(sealed))
        whole_waiting = not world.summary(whole)["attempts"]
    finally:
        for path in releases:
            open(path, "w").close()
    summaries = {task_id: world.finished(task_id, timeout=120) for task_id in [whole, *holders, sealed, probe]}
    problems = []
    note = first_attempt(probe_summary).get("gap_fill") or ""
    if probe_summary["task"].get("outcome") != "succeeded" or ROOM_GAP_ADMITTED_TEXT not in note:
        problems.append(f"1G 的小任务应当照整片之外照样空着的 3G 插空起来、跑完（这一格的前提），实际 {describe(probe_summary)}、gap_fill「{note}」")
    if before is None or before[1] != "2" or before[0] != before[2]:
        problems.append(f"两条碎片在跑、插空的那条跑过之后，封组那一句应当写已装到的数与还在跑的 2 条记的数相同（插空的不加进去），实际 {before}")
    if after is None or before is None or after[1] != "1" or after[0] != before[0] or after[2] == before[2]:
        problems.append(f"放行的那条跑完之后应当写还在跑的 1 条、记的数跟着减，已装到的数不变，实际改前 {before}、改后 {after}")
    if not whole_waiting:
        problems.append("还有一条碎片在跑时整片（45G）放不下，应当照旧在等（这一格的前提）")
    if {summary["task"].get("outcome") for summary in summaries.values()} != {"succeeded"}:
        problems.append(f"放行之后都应当跑完，实际结局 {sorted({str(summary['task'].get('outcome')) for summary in summaries.values()})}")
    if problems:
        raise SelftestFailure("组的账跑完的跟着减、插空的不进：" + "；".join(problems), world.server_log_lines())
    return f"组的账：插空起了之后 {before}，放行一条碎片跑完之后 {after}（已装到不变、还在跑的跟着减）"


PROGRESS_FREE_GIBIBYTES = 30            # progress-stall-reported：本机空 30G，排头一条没记录的（4G）照公式放得下
PROGRESS_WATCH_SECONDS = 4 * SERVER.FAST_PROGRESS_STALL_SECONDS   # 那一格等看进展报出来至多等这么久（过时限几倍）
PROGRESS_HEALTH_TEXT = "一条都没起（看进展"   # task_server.py 的 progress_problem 那一句里的几个字


def cell_progress_stall_reported(world):
    """看进展兜派活自己判错（task_server.py 的 progress_problem；队列卡死 4 个多小时、51 条排着只跑 3 条那一次）：本机空 30G、排头一条小的，
    派活被 TASK_SERVER_SELFTEST_PLACEMENT_REFUSED 演成一律判不能起：派活的结论是不能起（starved_problem 不报），看进展只读余量与起没起，
    照公式放得下最小那条而本机过了时限一条都没起，/health 回不健康、那一句点名本机、照公式还能记多少与最小那条的任务号；放开之后跑完、回 ok。
    弄坏开关 progress-unwatched 下 /health 一直回 ok，这一格红。"""
    world.resources(local=(PROGRESS_FREE_GIBIBYTES, PROGRESS_FREE_GIBIBYTES))
    refused = os.path.join(world.root, "placement-refused")
    open(refused, "w").close()
    world.start_server(extra={"TASK_SERVER_SELFTEST_PLACEMENT_REFUSED": refused})
    task_id = world.submit(["bash", "-c", "exit 0", "progress-stall"])
    waiting = world.wait_for(task_id, lambda found: "PLACEMENT_REFUSED" in (found.get("waiting_for") or "") or found["task"].get("state") != "queued", "判成不能起")
    health = health_until(world, lambda found: PROGRESS_HEALTH_TEXT in (found.get("problem") or ""), PROGRESS_WATCH_SECONDS)
    os.remove(refused)
    summary = world.finished(task_id)
    recovered = health_until(world, lambda found: found.get("ok"), 10)
    problem = health.get("problem") or ""
    problems = []
    if waiting["attempts"] or "PLACEMENT_REFUSED" not in (waiting.get("waiting_for") or ""):
        problems.append(f"派活一律判不能起时排头应当排着、排队原因写演法那一句，实际 {describe(waiting)}、在等「{waiting.get('waiting_for')}」")
    if health.get("ok") or PROGRESS_HEALTH_TEXT not in problem or "本机" not in problem or f"任务 {task_id}" not in problem or STARVED_HEALTH_TEXT in problem:
        problems.append(f"派活判错（一律判不能起）而本机照公式放得下：过了看进展的时限（{SERVER.FAST_PROGRESS_STALL_SECONDS:.0f} 秒）/health 应当回不健康、"
                        f"看进展那一句点名本机与任务 {task_id}，不报「{STARVED_HEALTH_TEXT}」（派活的结论是不能起），实际 {health}")
    if summary["task"].get("outcome") != "succeeded" or not recovered.get("ok"):
        problems.append(f"放开之后应当跑完、/health 回 ok，实际 {describe(summary)}、{recovered}")
    if problems:
        raise SelftestFailure("看进展兜派活判错：" + "；".join(problems), world.server_log_lines())
    return f"派活一律判不能起、本机照公式放得下：/health 报「{problem[:100]}」，放开之后跑完、回 ok"


CARD_WAIT_PEER_RELEASE = "release-card-wait-peer"   # card-wait-holds-only-its-host：占着第二台两张卡的那一条等假仓根里这份放行文件（假第二台照 fake-peer-wait 等）


def cell_card_wait_holds_only_its_host(world):
    """前面等卡的只在它要的那几台上让（task_server.py 的 place_queued_into 的 gpu_waiting_hosts；任务 7615 要第二台 3 张卡在等，7618 要 1 张也被挡住）：
    卡的先后是本机 0、第二台 0、1、2；一条声明要 2 张的占着第二台两张（本机只一张，凑不出），一条要同一台 3 张的在等（只有第二台有 3 张），
    后面一条要 1 张的照优先级分到本机 0、照起跑完，那条要 3 张的仍在等。弄坏开关 card-wait-blocks-later-cards 下前面一条等卡就挡住后面全部要卡的，这一格红。"""
    two_host_cards(world, f"{LOCAL_CARD_ZERO} {PEER_CARD_ZERO} {PEER_CARD_ONE} {PEER_CARD_TWO}", 1, 3)
    world.start_server()
    holder_command = write_card_script(world, "two-peer-cards.sh", 2)[:-1] + [f"fake-peer-wait={CARD_WAIT_PEER_RELEASE}"]   # 不写 fake-wrapper-exits：假第二台等放行
    holder = world.submit(holder_command)
    waiting_id, later_id, later, waiting = None, None, {}, {}
    try:
        world.running(holder)
        waiting_id = world.submit(write_card_script(world, "three-peer-cards.sh", 3))
        waiting = world.wait_for(waiting_id, lambda found: "等空卡" in (found.get("waiting_for") or "") or found["task"].get("state") != "queued", "报等空卡")
        later_id = world.submit(write_card_script(world, "one-local-card.sh", 1))
        try:
            later = world.finished(later_id, timeout=PLACEMENT_WAIT_SECONDS)
        except SelftestFailure:
            later = world.summary(later_id)
        waiting = world.summary(waiting_id)
    finally:
        open(os.path.join(world.repo, CARD_WAIT_PEER_RELEASE), "w").close()
    outcomes = {name: world.finished(task_id, timeout=120)["task"].get("outcome")
                for name, task_id in (("占着的", holder), ("要 3 张的", waiting_id), ("后面要 1 张的", later_id)) if task_id is not None}
    held = world.summary(holder)
    problems = []
    if attempt_card_and_host(held)[1] != "peer":
        problems.append(f"要 2 张的应当整组分到第二台（本机只一张），实际 {attempt_card_and_host(held)}")
    if waiting.get("attempts") or "等空卡" not in (waiting.get("waiting_for") or ""):
        problems.append(f"要同一台 3 张的在第二台被占着两张时应当排着、在等「等空卡」，实际 {describe(waiting) if waiting else '没交上'}、在等「{waiting.get('waiting_for')}」")
    if later.get("task", {}).get("outcome") != "succeeded" or attempt_card_and_host(later) != (LOCAL_CARD_ZERO, "local"):
        problems.append(f"后面要 1 张的应当照优先级分到 {LOCAL_CARD_ZERO}、照起跑完（前面等的是第二台的卡），实际 "
                        f"{describe(later) if later else '没交上'}、卡与台 {attempt_card_and_host(later) if later else None}、在等「{later.get('waiting_for')}」")
    if set(outcomes.values()) != {"succeeded"}:
        problems.append(f"放行之后都应当跑完，实际 {outcomes}")
    if problems:
        raise SelftestFailure("等卡的只在它要的那几台上让：" + "；".join(problems), world.server_log_lines())
    return f"要 3 张的在等第二台的卡：后面要 1 张的照起在 {LOCAL_CARD_ZERO}、跑完"


RECORD_WINDOW_PINNED = 5   # 钉住 task_server.py 的 RECORD_WINDOW（近几次的名额）：改它先改这里
SHARD_WINDOW_SMALL_PEAK_BYTES = GIBIBYTE   # shard-siblings-peak-windowed：撞过上限之后各片跑成量到的峰值（× 1.5 不到没记录的 4G）
SHARD_WINDOW_SHARDS = 1 + RECORD_WINDOW_PINNED   # 那一格的片数：第 0 片撞过一次再跑成，之后几片一次一片跑成，最后起的那一片之前有五次小的


def cell_shard_siblings_peak_windowed(world):
    """兄弟片按近几次取（task_server.py 的 shard_family_peak 按结束时刻取近 RECORD_WINDOW 个）：本机空闲 7G、一次只放得下一片，每片跑成都量到 1G（假分片入口的 exit_peak）。
    第 0 片头一次撞了 4096M 的上限、重排跑成，别的片一片片跑。每片头一次起之前：撞过的那一次之后跑成的不到五次（撞过的还在近 5 个里）从 6144M 起（shard-family）；
    满五次（之后五片都小，撞过的出了窗口）回落到没记录的 4096M、prediction 不记 shard-family。两种都要出现（一次一片：最后起的那一片之前有五次小的）。
    弄坏开关 history-peak-unwindowed 下照改前不截、满五次的也 6144M；shard-siblings-unshared 下不到五次的也只 4096M，这一格红。"""
    world.resources(local=(SHARD_FAMILY_FREE_GIBIBYTES, 30))
    shards = [{"exits": [250, "run"], "exit_peak": SHARD_WINDOW_SMALL_PEAK_BYTES}] + [{"exit_peak": SHARD_WINDOW_SMALL_PEAK_BYTES} for _ in range(SHARD_WINDOW_SHARDS - 1)]
    command = world.write_spec("spec-shard-window.json", {"shards": shards})
    world.start_server()
    summary = world.finished(world.submit(command, cwd=world.repo), timeout=120)
    children = shard_children(summary)
    opening = {index: (child["attempts"] or [{}])[0] for index, child in children.items()}
    succeeded = [attempt for child in children.values() for attempt in child["attempts"] if attempt.get("outcome") == "succeeded"]
    hit_at = float(opening.get(0, {}).get("ended_at") or 0)
    problems, seen = [], {}
    if summary["task"].get("outcome") != "succeeded" or opening.get(0, {}).get("outcome") != "memory-cap-hit":
        problems.append(f"第 0 片头一次应当撞 {UNRECORDED_DEFAULT_PINNED_TEXT} 的上限、整件跑成，实际 {describe(summary)}、第 0 片头一次 {opening.get(0, {}).get('outcome')}")
    if [attempt.get("memory_peak") for attempt in succeeded] != [SHARD_WINDOW_SMALL_PEAK_BYTES] * SHARD_WINDOW_SHARDS:
        problems.append(f"每片跑成那一次都应当记下假分片入口写的峰值 {SHARD_WINDOW_SMALL_PEAK_BYTES}，实际 {[attempt.get('memory_peak') for attempt in succeeded]}")
    for index in range(1, SHARD_WINDOW_SHARDS):
        attempt = opening.get(index, {})
        started = float(attempt.get("started_at") or 0)
        smaller = sum(1 for done in succeeded if hit_at < float(done.get("ended_at") or 0) <= started)   # 撞过之后、这一片起之前跑成的几次
        wanted = UNRECORDED_DEFAULT_PINNED_TEXT if smaller >= RECORD_WINDOW_PINNED else SHARD_FAMILY_RAISED_TEXT
        seen[index] = (smaller, attempt.get("memory_cap_text"), attempt.get("prediction"))
        if attempt.get("memory_cap_text") != wanted or (attempt.get("prediction") == lib.SHARD_FAMILY_HISTORY) != (wanted == SHARD_FAMILY_RAISED_TEXT):
            problems.append(f"第 {index} 片起之前撞过之后跑成了 {smaller} 次：头一次应当{'回落到' if smaller >= RECORD_WINDOW_PINNED else '从'} {wanted}"
                            f"{'、prediction 不记 ' if smaller >= RECORD_WINDOW_PINNED else '（'}{lib.SHARD_FAMILY_HISTORY}{'' if smaller >= RECORD_WINDOW_PINNED else '）'}，"
                            f"实际 {attempt.get('memory_cap_text')}（{attempt.get('prediction')}）")
    kinds = {value[0] >= RECORD_WINDOW_PINNED for value in seen.values()}
    if kinds != {True, False}:
        problems.append(f"世界没造出要的两种（撞过的还在窗口里、已出了窗口），各片（撞过之后跑成几次、上限、prediction）{seen}")
    if problems:
        raise SelftestFailure("兄弟片按近几次取：" + "；".join(problems), world.server_log_lines())
    return f"兄弟片撞过 {UNRECORDED_DEFAULT_PINNED_TEXT} 之后各片头一次（撞过之后跑成几次、上限、prediction）：{seen}"


def cell_parallelism_registration_refused(world):
    """各格并行的路数取不出登记的每台默认重活线程数就判红，不退回按核数（本文件的 parallelism；.claude/heavy-thread-default 文件头：写坏或删掉这份文件，取法一律拒绝，不退回按核数算）：
    造四份样本交给 parallelism：SINGLEFS_THREAD_CAP=abc、登记文件不在、登记那一行没写理由，都交 (None, 原因, 出路)，原因里点名是哪一样（变量名与值、文件路径、第几行），出路不空；
    登记「7  # 样本理由」交 7，SINGLEFS_THREAD_CAP=5 盖过登记交 5。弄坏开关 parallelism-falls-back-to-cores 下前三份按核数交数，这一格红。"""
    good, bad, missing = (os.path.join(world.root, name) for name in ("registration-good", "registration-no-reason", "registration-missing"))
    for path, text in ((good, "7  # 样本理由\n"), (bad, "7\n")):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write(text)
    refused = (("SINGLEFS_THREAD_CAP=abc", {"SINGLEFS_THREAD_CAP": "abc"}, good, "SINGLEFS_THREAD_CAP=abc"),
               ("登记文件不在", {}, missing, missing),
               ("登记那一行没写理由", {}, bad, "第 1 行"))
    accepted = (("登记 7", {}, good, 7), ("SINGLEFS_THREAD_CAP=5 盖过登记的 7", {"SINGLEFS_THREAD_CAP": "5"}, good, 5))
    cores = max(1, os.cpu_count() or 1)
    problems = []
    for what, environment, registration, mention in refused:
        found, reason, howto = parallelism(environment, registration)
        if found is not None or mention not in reason or not howto:
            problems.append(f"{what}：应当交 (None, 点名「{mention}」的原因, 不空的出路)，实际 ({found}, 「{reason}」, 「{howto}」)；不退回按核数（本机 {cores} 核）")
    for what, environment, registration, wanted in accepted:
        found, reason, howto = parallelism(environment, registration)
        if found != wanted or reason or howto:
            problems.append(f"{what}：应当交 ({wanted}, 空, 空)，实际 ({found}, 「{reason}」, 「{howto}」)")
    if problems:
        raise SelftestFailure("并行度取登记的线程数：" + "；".join(problems))
    return f"取不出登记的线程数的 {len(refused)} 份样本都判红、点名了原因；登记值与 SINGLEFS_THREAD_CAP 的 {len(accepted)} 份照取"


# ── 批 93（records/2026-10-31-task-server巡检.md）：异常交给交任务的会话、为满时限的整片封着的台不插空、预算、峰值口径 ──
AGED_GAP_OCCUPANT_PEAK_BYTES = 27 * GIBIBYTE   # aged-seal-no-gap-fill 占着的碎片：上限与记账 40.5G（50G 的台可用上限 48G、界 40.8G：碎片），起了记进这一组
AGED_GAP_WHOLE_PEAK_BYTES = 30 * GIBIBYTE      # 那一格的整片：上限与记账 45G，占着的跑着时放不下（照公式 48G − 40.5G = 7.5G）
AGED_GAP_PROBE_PEAK_BYTES = 3 * GIBIBYTE       # 那一格来插空的碎片：上限与记账 4.5G，照公式放得下；排头整片起来之后那一份只剩 48G − 45G = 3G，放不下，照改前要看预计用时
AGED_GAP_PROBE_SECONDS = 1                     # 它记下的用时：照改前不超过挡路时长（占着的那条没有预计用时、按已跑算，已跑过满时限那么久）就插空
AGED_GAP_SECONDS = "3"                         # 那一格的满时限（TASK_SERVER_SELFTEST_AGED_SECONDS）


def cell_aged_seal_no_gap_fill(world):
    """为满时限的整片封着的台一条都不插（task_server.py 文件头 ②「满时限」；用户：「一个任务都排了两个小时了 还要有任务插队 明显不合理」）：本机 50G（可用上限 48G、界 40.8G），
    一条碎片占着 40.5G（记进这一组）；整片（记 45G）与一条记了 1 秒用时的碎片（记 4.5G）交了先不判，等两条都满时限（压到 3 秒）之后一齐放出来：那条碎片装进组就超过界、封组，
    挡它的整片满了时限：照封组那一句排着、后面接 AGED_SEAL_NO_GAP_TEXT，一条不插；放行占着的那条之后整片先起，碎片在它之后，都跑完。
    弄坏开关 aged-seal-gap-filled 下照改前按预计用时插空（1 秒不超过挡路时长），这一格红。"""
    world.resources(local=(50, 50))
    probe_command = ["bash", "-c", "exit 0", "aged-gap-probe"]
    world.seed_peak("selftest-aged-gap-occupant", AGED_GAP_OCCUPANT_PEAK_BYTES)
    world.seed_peak("selftest-aged-gap-whole", AGED_GAP_WHOLE_PEAK_BYTES)
    world.seed_peak("selftest-aged-gap-probe", AGED_GAP_PROBE_PEAK_BYTES)
    world.seed_duration("selftest-aged-gap-probe", AGED_GAP_PROBE_SECONDS, probe_command)
    world.start_server(extra={"TASK_SERVER_SELFTEST_AGED_SECONDS": AGED_GAP_SECONDS})
    release = os.path.join(world.root, "release-aged-gap-occupant")
    occupant = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "aged-gap-occupant", release], extra={lib.KEY_VARIABLE: "selftest-aged-gap-occupant"})
    whole = probe = None
    waiting = {}
    try:
        world.running(occupant)
        world.resources(local=(50, 50), hold=True)   # 交了先不判：没满时限时那条碎片照改前就插空了，要等两条都满时限再放出来
        whole = world.submit(["bash", "-c", "exit 0", "aged-gap-whole"], extra={lib.KEY_VARIABLE: "selftest-aged-gap-whole"})
        probe = world.submit(probe_command, extra={lib.KEY_VARIABLE: "selftest-aged-gap-probe"})
        time.sleep(float(AGED_GAP_SECONDS) + 1)   # 后交的那条也满了时限（下限，不比上限：机器忙时只会更久）
        world.resources(local=(50, 50))
        world.api("POST", "/wake")
        world.next_pass()
        world.next_pass()
        waiting = {task_id: world.summary(task_id) for task_id in (whole, probe)}
    finally:
        open(release, "w").close()
    summaries = {task_id: world.finished(task_id) for task_id in (occupant, whole, probe) if task_id is not None}
    problems = []
    probe_waiting = waiting.get(probe) or {"task": {}, "attempts": []}
    reason = probe_waiting.get("waiting_for") or ""
    if probe_waiting.get("attempts") or SEALED_GROUP_TEXT not in reason or SERVER.AGED_SEAL_NO_GAP_TEXT not in reason:
        problems.append(f"排头整片满了时限、那台封着组：记了 {AGED_GAP_PROBE_SECONDS} 秒用时的碎片应当一条都不插，排队原因带「{SEALED_GROUP_TEXT}」与「{SERVER.AGED_SEAL_NO_GAP_TEXT}」，"
                        f"实际 {describe(probe_waiting)}、排队原因「{reason}」")
    whole_order = attempt_order(summaries[whole]) if whole is not None else None
    probe_order = attempt_order(summaries[probe]) if probe is not None else None
    if whole_order is None or probe_order is None or probe_order < whole_order:
        problems.append(f"放行占着的那条之后整片应当先于那条碎片起，实际尝试号整片 {whole_order}、碎片 {probe_order}")
    if {summary["task"].get("outcome") for summary in summaries.values()} != {"succeeded"}:
        problems.append(f"放行之后都应当跑完，实际结局 {sorted({str(summary['task'].get('outcome')) for summary in summaries.values()})}")
    if problems:
        raise SelftestFailure("满时限整片封台时短任务也不插：" + "；".join(problems), world.server_log_lines())
    return f"满时限的整片封着台：记了 {AGED_GAP_PROBE_SECONDS} 秒用时的碎片不插空（「{reason[-60:]}」），整片（尝试 {whole_order}）先于它（尝试 {probe_order}）起，都跑完"


BUDGET_TINY_SLICE_GIBIBYTES = 5   # budget-over-maxima-refused：两台 slice 总上限都是 5G、假余量 2G，可用上限 3G；没记录的命令记 4G：比两台可用上限都大、又不比总上限大（judge_host 判不出 never）


def cell_budget_over_maxima_refused(world):
    """预算超过整台（task_server.py 文件头 ②「预算」）：两台可用上限都是 3G，一条没记录的命令记账 4G：照改前 judge_host 判不出永远放不下（不比 slice 总上限 5G 大），
    永远排着；改后排队那一刻就打回 memory-cap-at-maximum、退 236、一次都没起，说明写记账与两台的可用上限，wait 末行的出路写核实范围、分片与同时起的个数。
    弄坏开关 budget-over-maxima-waits 下照改前排着，这一格红。"""
    world.resources(local=(50, BUDGET_TINY_SLICE_GIBIBYTES), peer=(50, BUDGET_TINY_SLICE_GIBIBYTES), peer_cap_gibibytes=BUDGET_TINY_SLICE_GIBIBYTES)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "budget-over-maxima"], cwd=world.repo)
    world.next_pass()
    world.next_pass()
    summary = world.summary(task_id)
    problems = []
    note = summary["task"].get("note") or ""
    if (summary["task"].get("outcome") != "memory-cap-at-maximum" or summary["task"].get("exit_code") != 236 or summary["attempts"]
            or "预算超过整台" not in note or "可用上限" not in note):
        problems.append(f"记账 4G 比两台的可用上限 3G 都大：应当当场打回 memory-cap-at-maximum、退 236、一次都没起，说明写「预算超过整台」与可用上限，"
                        f"实际 {describe(summary)}、状态 {summary['task'].get('state')}、说明「{note}」")
        if summary["task"].get("state") != "finished":
            world.client(["cancel", str(task_id)])
    else:
        completed = world.client(["wait", str(task_id)])
        last = completed.stdout.strip().splitlines()[-1] if completed.stdout.strip() else ""
        if completed.returncode != 236 or "核实这一趟的范围（点的项、测试目标）、分片与同时起的个数" not in last:
            problems.append(f"wait 应当退 236、末行出路写核实范围、分片与同时起的个数，实际退 {completed.returncode}、末行「{last}」")
    if problems:
        raise SelftestFailure("预算超过整台：" + "；".join(problems), world.server_log_lines())
    return f"记账 4G、两台可用上限 3G：排队那一刻打回 memory-cap-at-maximum、退 236（「{note[:80]}」）"


def cell_oversized_budget_reported(world):
    """预算贴着整台（task_server.py 文件头 ①「要处置的提醒」的 oversized-budget）：两台 50G（可用上限 48G、界 40.8G），记过峰值 30G 的命令记账 45G：不超过可用上限、不小于整片界，
    照常排、起来跑；wait 打一行 kind=oversized-budget、依据写记账、可用上限与记账从哪来、退 ATTENTION_EXIT；放行之后跑完。
    弄坏开关 oversized-budget-silent 下不报，wait 等到命令自己退，这一格红。"""
    key = "selftest-oversized-budget"
    release = os.path.join(world.root, "release-oversized-budget")
    world.resources(local=(50, 50), peer=(50, 50), peer_cap_gibibytes=50)
    world.seed_peak(key, 30 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, fields = attention_problems(completed, task_id, "oversized-budget", lib.ATTENTION_EXIT)
    basis = fields.get("basis", "")
    if "记账" not in basis or "可用上限" not in basis or "记账从哪来" not in basis or "峰值" not in basis:
        problems.append(f"依据应当写记账、可用上限与记账从哪来（这把命令键记下的峰值），实际「{basis}」")
    open(release, "w").close()
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"只提醒、照跑：放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("预算贴着整台：" + "；".join(problems), world.server_log_lines())
    return f"记账 45G 不小于整片界 40.8G：wait 打 kind=oversized-budget（依据「{basis[:80]}」），照跑完"


LONG_RUN_IDLE_ATTENTION_SECONDS = "2"   # long-run-idle-cores-reported：跑满这么久就看平均在用核数（换掉 LONG_RUN_ATTENTION_SECONDS 的一小时）


def cell_long_run_idle_cores_reported(world):
    """跑得久而核没用起来（task_server.py 文件头 ①「要处置的提醒」的 long-run-idle-cores；任务 8856 给 31 线程平均只用 8.881 核）：假 scope 的 CPU 钉着不涨（平均约 0 核），
    跑满 2 秒（换掉一小时），wait 打一行 kind=long-run-idle-cores、依据写跑了多久与平均核数、退 ATTENTION_EXIT；只提醒、照跑，放行之后跑完。
    弄坏开关 long-run-idle-silent 下不报，wait 等到命令自己退，这一格红。"""
    scope = os.path.join(world.root, "long-run-idle-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    release = os.path.join(world.root, "release-long-run-idle")
    world.start_server(extra={"TASK_SERVER_SELFTEST_LONG_RUN_ATTENTION_SECONDS": LONG_RUN_IDLE_ATTENTION_SECONDS, "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release])
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, fields = attention_problems(completed, task_id, "long-run-idle-cores", lib.ATTENTION_EXIT)
    basis = fields.get("basis", "")
    if "平均只用" not in basis or "核" not in basis or "已跑" not in basis:
        problems.append(f"依据应当写已跑多久、给几线程与平均只用几核，实际「{basis}」")
    open(release, "w").close()
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"只提醒、照跑：放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("跑得久而核没用起来：" + "；".join(problems), world.server_log_lines())
    return f"跑满 {LONG_RUN_IDLE_ATTENTION_SECONDS} 秒、CPU 不涨：wait 打 kind=long-run-idle-cores（依据「{basis[:80]}」），照跑完"


FILLS_CAP_SEEDED_CAP_BYTES = 1000 * MEBIBYTE   # fills-any-cap-reported 记下的三次尝试各自的上限
FILLS_CAP_SEEDED_PEAK_BYTES = 950 * MEBIBYTE   # 那三次的峰值：是上限的 95%，不低于 90%


def cell_fills_any_cap_reported(world):
    """按给的上限开并行（task_server.py 文件头 ①「要处置的提醒」的 fills-any-cap）：同一命令键记下三次跑成的尝试，峰值都是上限的 95%；再交一条，wait 打一行 kind=fills-any-cap、
    依据写三次的上限与峰值、出路写「这条命令按给的上限开并行，核实」、退 ATTENTION_EXIT；只提醒、照跑，放行之后跑完。
    弄坏开关 fills-any-cap-silent 下不报，wait 等到命令自己退，这一格红。"""
    key = "selftest-fills-any-cap"
    for _ in range(SERVER.FILLS_CAP_RUNS):
        world.seed_attempt({"command_key": key, "outcome": "succeeded", "memory_peak": FILLS_CAP_SEEDED_PEAK_BYTES, "memory_cap": FILLS_CAP_SEEDED_CAP_BYTES})
    release = os.path.join(world.root, "release-fills-any-cap")
    world.start_server()
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra={lib.KEY_VARIABLE: key})
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, fields = attention_problems(completed, task_id, "fills-any-cap", lib.ATTENTION_EXIT)
    if "这条命令按给的上限开并行，核实" not in fields.get("next", "") or "上限" not in fields.get("basis", ""):
        problems.append(f"依据应当写三次的上限与峰值、出路写「这条命令按给的上限开并行，核实」，实际依据「{fields.get('basis')}」、出路「{fields.get('next')}」")
    open(release, "w").close()
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"只提醒、照跑：放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("按给的上限开并行：" + "；".join(problems), world.server_log_lines())
    return f"同键三次峰值都是上限的 95%：wait 打 kind=fills-any-cap（「{fields.get('basis', '')[:80]}」），照跑完"


OLD_METRIC_PEAK_BYTES = 20 * GIBIBYTE   # old-metric-peaks-not-counted 那一条旧行记的峰值：照改前起步 30720M


def cell_old_metric_peaks_not_counted(world):
    """旧口径的峰值不进起步（task_server_lib.py 文件头「资源记录」的峰值口径一栏；批 92 之前记的 memory_peak 含页缓存）：同一命令键只有一条没写峰值口径的旧行（峰值 20G），
    新交的照没有记录办：上限与记账 4096M、prediction 记 UNRECORDED_HISTORY；同键另一条写了这一版口径的（峰值 1G）照它 × 1.5 定 1536M。
    弄坏开关 old-metric-peaks-read 下旧行照改前进定额度（30720M），这一格红。"""
    old_key, new_key = "selftest-old-metric", "selftest-new-metric"
    world.seed_attempt({"command_key": old_key, "outcome": "succeeded", "memory_peak": OLD_METRIC_PEAK_BYTES}, metric=None)
    world.seed_peak(new_key, GIBIBYTE)
    world.start_server()
    expected = {"只有旧行的": (old_key, UNRECORDED_DEFAULT_PINNED_TEXT, lib.UNRECORDED_HISTORY), "写了这一版口径的": (new_key, "1536M", lib.RECORD_HISTORY)}
    problems = []
    for name, (key, cap_text, history) in expected.items():
        summary = world.finished(world.submit(["bash", "-c", "exit 0", key], extra={lib.KEY_VARIABLE: key}))
        attempt = first_attempt(summary)
        if attempt.get("memory_cap_text") != cap_text or attempt.get("prediction") != history or summary["task"].get("outcome") != "succeeded":
            problems.append(f"{name}上限应当是 {cap_text}、prediction 记 {history}、跑完，实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、{describe(summary)}")
    if problems:
        raise SelftestFailure("旧口径的峰值不进起步：" + "；".join(problems), world.server_log_lines())
    return f"只有旧口径那一行的命令照没记录的 {UNRECORDED_DEFAULT_PINNED_TEXT} 起，写了这一版口径的照峰值 × 1.5（1536M）"


# ── 异常表、再推、按会话号找收件 socket、交任务的二次确认、编译族只借编译那一段、清场协议单测（task_server_lib.py 文件头「异常表」；task_server.py 文件头 ⑥ ⑦ ②「内存」）──
ADVICE_SAMPLE_TASK = 4242          # anomaly-advice-in-texts 拼正文用的任务号（不进库）
REPUSH_SAMPLE_SECONDS = "2"        # anomaly-repushed-with-history：同一档报过之后再推的头一档（秒；TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS）
UNCLAIMED_REPUSH_SAMPLE_SECONDS = "2"   # unclaimed-outcome-repushed：结局没人取再推的头一档（秒；TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS）
CONFIRM_TIMEOUT_SAMPLE_SECONDS = "2"    # unconfirmed-cancelled：交了没确认满这么多秒服务取消（TASK_SERVER_SELFTEST_CONFIRM_TIMEOUT_SECONDS）
CONFIRM_HOLD_SECONDS = 1.5         # submit-awaits-confirmation：待确认的那一条看这么久（十几轮排队）不该起
CONFIRM_SCOPE_SAMPLE = "种类=items; 跑什么=自证的确认单; 范围=2 项; 为什么=核确认单打不打范围声明"
PHASE_BUILD_BYTES = 6 * GIBIBYTE         # family-lends-build-phase-only：Finished 之前假 scope 的进程内存（编译那一段）
PHASE_RUN_BYTES = 40 * GIBIBYTE          # Finished 之后换上的（跑测试那一段）
PHASE_SEEDED_RUN_BYTES = 42 * GIBIBYTE   # 同族另记一条没有 Finished 行的（编译那一段 null）跑成的峰值：不该借
PHASE_RAISED_TEXT = "9216M"              # 6G × 1.5：同族的另一条照编译那一段起步
PHASE_FAMILY_ENTRY = "family-phase-entry.sh"
CLEANUP_PROTOCOL_TESTS = 17   # task-server/tests/test_cleanup_protocol.py 的单测条数：跑到的少了（漏载、跳过）也判红；那份文件加减单测时一起改这个数


def table_advice(kind, task_id):
    """异常表（task_server_lib.py 文件头）里这一种那一行的修改意见，{task} 换成任务号：照文件头现读（lib.anomaly_rows），不经 lib.anomaly_advice（弄坏开关 attention-next-generic 弄坏的是那一侧）。"""
    rows = {row.kind: row for row in lib.anomaly_rows(lib.__doc__)}
    return rows[kind].advice.replace(lib.ANOMALY_TASK_PLACEHOLDER, str(task_id))


def cell_anomaly_table_matches_kinds(world):
    """异常表（task_server_lib.py 文件头「异常表」）与代码里分别处置的种类逐行对得上：ATTENTION_KINDS 每一种是「提醒照跑」，SUBMIT_REFUSAL_KINDS 是「提交时拒」，
    SUBMIT_WARNING_KINDS 是「提醒照跑」，错误类型表（OUTCOME_TYPES）里 succeeded 之外每一种是「打回」或「服务自己修」，SERVICE_ANOMALY_KINDS 是「提醒照跑」或「服务自己修」；
    表里不多不少、种类不重、动作在 ANOMALY_ACTIONS 里、修改意见不是那一句通用的；task_server.py 的 ANOMALY_REPUSHED_KINDS 都是表里的提醒。静态核，不起服务。"""
    rows = lib.anomaly_rows(lib.__doc__)
    table = {row.kind: row for row in rows}
    expected = {**{kind: ("提醒照跑",) for kind in lib.ATTENTION_KINDS + lib.SUBMIT_WARNING_KINDS}, **{kind: ("提交时拒",) for kind in lib.SUBMIT_REFUSAL_KINDS},
                **{kind: ("打回", "服务自己修") for kind in lib.OUTCOME_TYPES if kind != "succeeded"},
                **{kind: ("提醒照跑", "服务自己修") for kind in lib.SERVICE_ANOMALY_KINDS}}
    problems = []
    duplicated = sorted({row.kind for row in rows if [other.kind for other in rows].count(row.kind) > 1})
    missing, extra = sorted(set(expected) - set(table)), sorted(set(table) - set(expected))
    wrong = sorted(f"{kind}（{table[kind].action}）" for kind in set(expected) & set(table) if table[kind].action not in expected[kind])
    unknown = sorted(row.kind for row in rows if row.action not in lib.ANOMALY_ACTIONS)
    generic = sorted(row.kind for row in rows if row.advice == lib.ANOMALY_GENERIC_ADVICE or "改小命令重交" in row.advice)
    unrepushable = sorted(set(SERVER.ANOMALY_REPUSHED_KINDS) - set(lib.ATTENTION_KINDS))
    for name, found in (("表里重复的种类", duplicated), ("代码里有、表里没有的", missing), ("表里有、代码里没有的", extra), ("动作与代码里的处置对不上的", wrong),
                        ("动作不是四样之一的", unknown), ("修改意见是通用句的", generic), ("再推的种类不是表里的提醒", unrepushable)):
        if found:
            problems.append(f"{name}：{'、'.join(found)}")
    if not rows or problems:
        raise SelftestFailure(f"异常表与代码里的种类对不上（表里读到 {len(rows)} 行）：" + "；".join(problems or ["一行都没读到"]))
    return f"异常表 {len(rows)} 行，与提醒、提交时拒与警告、结局、服务自己的几种逐行对得上，动作都在四样里、修改意见没有通用句"


def cell_anomaly_advice_in_texts(world):
    """每一种异常推出去的正文里带它那一行的修改意见：十二种提醒照服务拼正文的那一段（Scheduler.attention_item、attention_text，套在假调度器上，不起服务）各拼一回，
    「出路：」那一行以异常表那一行的修改意见起头；同一种再推的那一回「此刻还差：」那一行同样带它；疑似卡住的正文（suspect_text）带 suspected-stall 那一行的；
    错误类型表里 succeeded 之外每一种结局的打回理由（lib.reason_fields，发给会话的结局正文「出路：」照它）next 带那一种那一行的。
    弄坏开关 attention-next-generic 下一律退回通用句，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.store = types.SimpleNamespace(task=lambda task_id: None)   # attention_text 读日志路径：库里没有这一条，写「无（还没起过）」
    fake.attention_histories, fake.attention_lock, fake.anomaly_repush_seconds = {}, threading.Lock(), SERVER.ANOMALY_REPUSH_SECONDS
    task = {"id": ADVICE_SAMPLE_TASK}
    problems = []
    for kind in lib.ATTENTION_KINDS:
        advice = table_advice(kind, ADVICE_SAMPLE_TASK)
        item = SERVER.Scheduler.attention_item(fake, kind, 1, ADVICE_SAMPLE_TASK, "依据的数", "原因", "这一回的具体数")
        text = SERVER.Scheduler.attention_text(fake, task, item)
        if f"出路：{advice}" not in text:
            problems.append(f"{kind} 的正文「出路：」应当以「{advice[:80]}…」起头，实际「{text[:300]}」")
        fake.attention_histories[(ADVICE_SAMPLE_TASK, kind)] = [{"tier": 1, "repush": 0, "at": time.time() - 60, "basis": "头一回的依据"}]
        repushed = SERVER.Scheduler.attention_text(fake, task, dict(item, repush=1))
        if f"此刻还差：{advice}" not in repushed or "之前推过：" not in repushed or "第 1 回再推" not in repushed:
            problems.append(f"{kind} 再推的正文应当写「之前推过：…」与「此刻还差：{advice[:60]}…」、第一行带「第 1 回再推」，实际「{repushed[:400]}」")
    suspect = SERVER.Scheduler.suspect_text(fake, task, task, {"number": 1, "host": "local", "log": ""}, {"segment": 1, "why": "CPU 满窗口不涨"})
    if table_advice("suspected-stall", ADVICE_SAMPLE_TASK) not in suspect:
        problems.append(f"疑似卡住的正文应当带 suspected-stall 那一行的修改意见，实际「{suspect[:300]}」")
    outcomes = [kind for kind in lib.OUTCOME_TYPES if kind != "succeeded"]
    for kind in outcomes:
        fields = dict(lib.reason_fields({"id": ADVICE_SAMPLE_TASK, "outcome": kind, "exit_code": 1}, [], ""))
        wanted = table_advice(kind, ADVICE_SAMPLE_TASK).split("<unit=")[0]   # launcher-lost 那一行的 <unit= 点名的单元> 由 reason_fields 填上单元名，只比它前面那一截
        if wanted not in fields.get("next", ""):
            problems.append(f"结局 {kind} 的 next= 应当带异常表那一行的修改意见「{wanted[:80]}…」，实际「{fields.get('next', '')[:300]}」")
    if problems:
        raise SelftestFailure("推出去的正文带不带异常表的修改意见：" + "；".join(problems))
    return f"{len(lib.ATTENTION_KINDS)} 种提醒（头一回与再推）、疑似卡住与 {len(outcomes)} 种结局的正文都带异常表那一行的修改意见"


def cell_anomaly_repushed_with_history(world):
    """不随时长涨的提醒同一档推过之后还在（task_server.py 文件头 ⑥「没处置时」；异常表 fills-any-cap 那一行）：同键三次峰值都贴着上限、再交一条一直跑着，
    收件 socket 先收到一条「fills-any-cap（第 1 档）」，同一档报过满头一档（自证里 2 秒）再收到一条「（第 1 档，没处置、第 1 回再推）」，正文写「之前推过：…」与「此刻还差：<那一行的修改意见>」。
    弄坏开关 anomaly-not-repushed 下不再推，这一格红。"""
    key = "selftest-anomaly-repush"
    for _ in range(SERVER.FILLS_CAP_RUNS):
        world.seed_attempt({"command_key": key, "outcome": "succeeded", "memory_peak": FILLS_CAP_SEEDED_PEAK_BYTES, "memory_cap": FILLS_CAP_SEEDED_CAP_BYTES})
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-anomaly-repush")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS": REPUSH_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: key}))
    first_head, again_head = f"task-server：任务 {task_id} fills-any-cap（第 1 档）", f"task-server：任务 {task_id} fills-any-cap（第 1 档，没处置、第 1 回再推）"
    problems = inbox_problems(inbox.wait_for(world, first_head), first_head, "头一回")
    again = inbox.wait_for(world, again_head)
    problems += inbox_problems(again, again_head, "同一档没处置再推")
    text = again[0]["text"] if again else ""
    advice = table_advice("fills-any-cap", task_id)
    if again and ("之前推过：" not in text or f"此刻还差：{advice}" not in text):
        problems.append(f"再推的正文应当写「之前推过：…」与「此刻还差：{advice[:60]}…」，实际「{text[:400]}」")
    open(release, "w").close()
    world.finished(task_id)
    if problems:
        raise SelftestFailure("没处置的提醒再推：" + "；".join(problems), world.server_log_lines())
    return f"fills-any-cap 头一回推过、满 {REPUSH_SAMPLE_SECONDS} 秒还在，再推了一回（写了之前推过什么、此刻还差什么）"


def cell_unclaimed_outcome_repushed(world):
    """结局没人取（task_server.py 文件头 ⑥「结局没人取」；异常表 unclaimed-outcome）：会话还没登进会话登记时任务跑完，结局那一条发不出去（写 inbox-unreachable 告警）；
    之后会话登上（演会话重启、换了 socket），结局满头一档（自证里 2 秒）没人取，服务再推一回到新的 socket：第一行「结局还没人取（succeeded」，写此刻还差什么（那一行的修改意见）
    与那条结局的末行；推出去了记成已取，status 不再列成没人取。弄坏开关 unclaimed-outcome-not-repushed 下不再推，这一格红。"""
    inbox = FakeInbox(world, "a", register=False)
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS": UNCLAIMED_REPUSH_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", "exit 0", "unclaimed-repush"], extra=inbox.environment())
    world.finished(task_id)
    patience = world.patience(INBOX_WAIT_SECONDS)
    while patience.waiting() and not any(alert.get("kind") == "inbox-unreachable" and alert.get("task") == task_id for alert in world.alerts()):
        time.sleep(POLL_SECONDS)
    world.register_session(SESSION_A, inbox.path, "inbox-a")
    head = f"task-server：任务 {task_id} 结局还没人取（succeeded"
    found = inbox.wait_for(world, head)
    problems = inbox_problems(found, head, "结局没人取再推")
    text = found[0]["text"] if found else ""
    advice = table_advice("unclaimed-outcome", task_id)
    if found and (f"此刻还差：{advice}" not in text or f"{lib.RESULT_MARK} task={task_id} type=succeeded" not in text):
        problems.append(f"再推的正文应当写「此刻还差：{advice[:60]}…」并带那条结局的末行，实际「{text[:400]}」")
    patience = world.patience(INBOX_COLLECTED_SECONDS)
    listed = True
    while patience.waiting() and listed:
        listed = any(item.get("id") == task_id for item in json.loads(world.client(["status", "--json"]).stdout or "{}").get("tasks", []))
        time.sleep(POLL_SECONDS)
    if listed:
        problems.append(f"再推出去之后任务 {task_id} 应当记成已取、status 不再列成完成了没人取，实际还列着")
    if problems:
        raise SelftestFailure("结局没人取再推：" + "；".join(problems), world.server_log_lines())
    return f"结局头一回发不出去、会话登上之后满 {UNCLAIMED_REPUSH_SAMPLE_SECONDS} 秒再推到了新的 socket，记成已取"


def cell_inbox_socket_follows_session(world):
    """按会话号找收件 socket（task_server.py 文件头 ⑥「收件地址」）：交任务时会话的收件 socket 是旧的那一个；任务跑着时那个会话重启（旧 socket 关了、
    会话登记里换成新的一份），另有一份同一会话号、更晚启动而进程已经不在的登记点着旧 socket（不该认）；任务跑完，结局那一条照新的 socket 收到（令牌照交任务时的）。
    弄坏开关 inbox-socket-from-submit-only 下照交任务时的旧 socket 发、发不出去，这一格红。"""
    old = FakeInbox(world, "old")
    release = os.path.join(world.root, "release-socket-follows")
    start_inbox_server(world)
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra=old.environment())
    world.running(task_id)
    old.close()
    os.remove(os.path.join(world.sessions, "inbox-old.json"))
    gone = subprocess.Popen(["true"])   # 一个已经退了的进程号：那一份登记点着旧 socket、启动更晚，进程不在了不该认
    gone.wait()
    world.register_session(SESSION_A, old.path, "gone-process", pid=gone.pid, started_milliseconds=int(time.time() * 1000) + 60000)
    new = FakeInbox(world, "new", token=old.token)
    open(release, "w").close()
    world.finished(task_id)
    head = f"task-server：任务 {task_id} succeeded"
    problems = inbox_problems(new.wait_for(world, head), head, "会话重启之后的结局")
    if problems:
        raise SelftestFailure("按会话号找收件 socket：" + "；".join(problems), world.server_log_lines())
    return "会话重启换了收件 socket（旧的那一份进程已不在）：结局照会话号找到的新 socket 收到，令牌照交任务时的"


def cell_submit_awaits_confirmation(world):
    """交任务的二次确认（task_server.py 文件头 ⑦）：交一条点了两项、带范围声明的，回话打确认单（命令、范围声明四段、项数 2、在排在跑 0 条）与 confirm-needed 一行（items=2 tasks=0），
    任务记 awaiting-confirmation、余量够也不起；同会话另交一条点了同一项、确认过的一直跑着；拿旧的两个数确认被拒（退 3）、打出服务此刻算的 tasks=1 与交叠的那一条的号；
    项数写错的同样拒；照服务算的两个数确认才进队、跑完。弄坏开关 confirm-skipped 下交了就进队、confirm-counts-unchecked 下旧数也收，这一格红。"""
    world.resources(local=(100, 100))
    world.start_server()
    problems = []
    submitted = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "confirm-sheet", "--item", "alpha", "--item", "beta"],
                             extra={"SINGLEFS_RUN_SCOPE": CONFIRM_SCOPE_SAMPLE})
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", submitted.stdout)
    task_id = int(found.group(1)) if found else 0
    needed = lib.confirm_needed_fields(submitted.stdout)
    if submitted.returncode != 0 or needed != (task_id, 2, 0):
        problems.append(f"交了应当退 0、打 confirm-needed task={task_id} items=2 tasks=0，实际退 {submitted.returncode}、读到 {needed}、「{submitted.stdout.strip()[-400:]}」")
    for wanted in ("确认单（任务", "命令：bash -c", "范围声明：种类=items", "项数 2：--item 2", "在排在跑 0 条"):
        if wanted not in submitted.stdout:
            problems.append(f"确认单里应当有「{wanted}」，实际「{submitted.stdout.strip()[-500:]}」")
    time.sleep(CONFIRM_HOLD_SECONDS)
    summary = world.summary(task_id) if task_id else {"task": {}, "attempts": []}
    if summary["task"].get("state") != lib.AWAITING_CONFIRMATION_STATE or summary["attempts"]:
        problems.append(f"没确认的应当一直待确认、一次都不起，实际 {describe(summary)}")
    release = os.path.join(world.root, "release-confirm-other")
    other = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release, "--item", "alpha"])
    world.running(other)
    stale = world.client(["confirm", str(task_id), "--items", "2", "--tasks", "0"])
    if stale.returncode != 3 or lib.confirm_needed_fields(stale.stdout) != (task_id, 2, 1) or f"{other}（都点了 --item alpha）" not in stale.stdout:
        problems.append(f"拿旧的条数确认应当被拒（退 3）、打服务此刻算的 items=2 tasks=1 与交叠的「{other}（都点了 --item alpha）」，"
                        f"实际退 {stale.returncode}、「{stale.stdout.strip()[-400:]}」「{stale.stderr.strip()[-200:]}」")
    wrong_items = world.client(["confirm", str(task_id), "--items", "3", "--tasks", "1"])
    if wrong_items.returncode != 3:
        problems.append(f"项数写错（3）应当被拒（退 3），实际退 {wrong_items.returncode}")
    confirmed = world.client(["confirm", str(task_id), "--items", "2", "--tasks", "1"])
    if confirmed.returncode != 0:
        problems.append(f"照服务算的两个数确认应当退 0，实际退 {confirmed.returncode}、「{confirmed.stderr.strip()[-300:]}」")
    done = world.finished(task_id) if task_id else {"task": {}}
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"确认之后应当进队、跑完，实际 {describe(done)}")
    open(release, "w").close()
    world.finished(other)
    if problems:
        raise SelftestFailure("交任务的二次确认：" + "；".join(problems), world.server_log_lines())
    return "交了待确认、不起；旧数与错的项数确认被拒并打出服务算的数与交叠的那一条；照服务算的数确认之后进队跑完"


def cell_unconfirmed_cancelled(world):
    """交了没确认（task_server.py 文件头 ⑦；异常表 unconfirmed）：满时限（自证里 2 秒）服务取消，结局 cancelled、说明以「没确认，已取消」起头、一次都没起；
    收件 socket 收到那一条结局，正文带「没确认，已取消」。弄坏开关 unconfirmed-kept 下一直待确认，这一格红。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_CONFIRM_TIMEOUT_SECONDS": CONFIRM_TIMEOUT_SAMPLE_SECONDS})
    submitted = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "never-confirmed"], extra=inbox.environment())
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", submitted.stdout)
    if submitted.returncode != 0 or not found:
        raise SelftestFailure(f"交了应当退 0、打任务号，实际退 {submitted.returncode}", [submitted.stdout, submitted.stderr])
    task_id = int(found.group(1))
    summary = world.wait_for(task_id, lambda summary: summary["task"].get("state") == "finished", "没确认被服务取消", timeout=INBOX_WAIT_SECONDS)
    problems = []
    if summary["task"].get("outcome") != "cancelled" or not str(summary["task"].get("note") or "").startswith("没确认，已取消") or summary["attempts"]:
        problems.append(f"满时限没确认的应当记 cancelled、说明以「没确认，已取消」起头、一次都没起，实际 {describe(summary)}、说明「{summary['task'].get('note')}」")
    head = f"task-server：任务 {task_id} cancelled"
    messages = inbox.wait_for(world, head)
    problems += inbox_problems(messages, head, "没确认被取消")
    if messages and "没确认，已取消" not in messages[0]["text"]:
        problems.append(f"推给会话的正文应当带「没确认，已取消」，实际「{messages[0]['text'][:300]}」")
    if problems:
        raise SelftestFailure("交了没确认：" + "；".join(problems), world.server_log_lines())
    return f"交了满 {CONFIRM_TIMEOUT_SAMPLE_SECONDS} 秒没确认：服务取消、推了「没确认，已取消」"


def cell_family_lends_build_phase_only(world):
    """编译族只借编译那一段的峰值（task_server.py 文件头 ②「内存」的编译族）：入口声明了编译族、先等放行再打 cargo 的 Finished 那一行、再等放行退 0；
    Finished 之前假 scope 的进程内存是 6G（attempt_runner 写的 build_phase 读到「6G 0」），见到 Finished 之后（读到「6G 1」）换成 40G：这一次尝试记 build_phase_peak 6G；
    同族另记一条没有 Finished 行（编译那一段 null）、跑成峰值 42G 的，不借；同族另一条（命令键不同、没有自己的记录）第一次照 6G × 1.5 = 9216M 起、prediction 记 build-family。
    弄坏开关 family-lends-run-peak 下借 memory_peak（40G、42G 那一档），build-phase-past-finished 下见到 Finished 之后的 40G 也进编译段峰值，这一格红。"""
    with open(os.path.join(world.repo, PHASE_FAMILY_ENTRY), "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_BUILD_FAMILY_MARK} -p selftest-family-phase --release\n"
                     '[[ "$1" == quick ]] && exit 0\n'
                     'for round in $(seq 1 600); do [[ -e "$1" ]] && break; sleep 0.1; done\n'
                     "echo '    Finished `release` profile [optimized] target(s) in 1.00s'\n"
                     'for round in $(seq 1 600); do [[ -e "$2" ]] && exit 0; sleep 0.1; done\nexit 9\n')
    world.resources(local=(100, 100))
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": world.write_fake_scope(PHASE_BUILD_BYTES)})
    compiled, finished = os.path.join(world.root, "release-phase-compiled"), os.path.join(world.root, "release-phase-finished")
    task_id = world.submit(["bash", PHASE_FAMILY_ENTRY, compiled, finished], cwd=world.repo)
    directory = world.running(task_id)["attempts"][-1].get("directory") or ""

    def sampled(peak, seen):
        """等 attempt_runner 写的 build_phase 那一份（lib.BUILD_PHASE_FILE_NAME，尝试目录下）读到「<peak> <seen>」：编译那一段见过的最大与见没见到 cargo 编完那一行。"""
        path = os.path.join(directory, lib.BUILD_PHASE_FILE_NAME)
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        read = ""
        while patience.waiting():
            with contextlib.suppress(OSError):
                with open(path, encoding="utf-8") as handle:
                    read = handle.read().strip()
            if read == f"{peak} {seen}":
                return
            time.sleep(POLL_SECONDS)
        raise SelftestFailure(f"等了 {patience.elapsed():.0f} 秒，{path} 没读到「{peak} {seen}」，最后读到「{read}」", world.server_log_lines())

    sampled(PHASE_BUILD_BYTES, 0)
    open(compiled, "w").close()
    sampled(PHASE_BUILD_BYTES, 1)   # attempt_runner 见到 Finished 之后才换上跑测试那一段的用量：换之前采到的都算编译那一段
    world.write_fake_scope(PHASE_RUN_BYTES)
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    while patience.waiting():   # attempt_runner 按轮采到了跑测试那一段的用量（memory_peak 涨到 40G）再放它退出
        with contextlib.suppress(OSError, ValueError):
            with open(os.path.join(directory, lib.MEMORY_PEAK_FILE_NAME), encoding="utf-8") as handle:
                if int(handle.read().strip()) >= PHASE_RUN_BYTES:
                    break
        time.sleep(POLL_SECONDS)
    open(finished, "w").close()
    summary = world.finished(task_id)
    problems = []
    attempt = first_attempt(summary)
    if summary["task"].get("outcome") != "succeeded" or attempt.get("build_phase_peak") != PHASE_BUILD_BYTES or attempt.get("memory_peak") != PHASE_RUN_BYTES:
        problems.append(f"Finished 之前读到 {PHASE_BUILD_BYTES}、之后换成 {PHASE_RUN_BYTES}：尝试应当记 build_phase_peak {PHASE_BUILD_BYTES}、memory_peak {PHASE_RUN_BYTES}、跑完，"
                        f"实际 {attempt.get('build_phase_peak')}、{attempt.get('memory_peak')}、{describe(summary)}")
    seed_ended_attempt(world, "selftest-family-phase-unfinished", time.time(), "succeeded", memory_peak=PHASE_SEEDED_RUN_BYTES,
                       build_family=summary["task"].get("build_family") or "")
    quick = world.finished(world.submit(["bash", PHASE_FAMILY_ENTRY, "quick"], cwd=world.repo))
    attempt = first_attempt(quick)
    if attempt.get("memory_cap_text") != PHASE_RAISED_TEXT or attempt.get("prediction") != lib.BUILD_FAMILY_HISTORY:
        problems.append(f"同族另一条应当照编译那一段 6G × 1.5 = {PHASE_RAISED_TEXT} 起、prediction 记 {lib.BUILD_FAMILY_HISTORY}（不借跑测试那一段的 40G、42G），"
                        f"实际 {attempt.get('memory_cap_text')}、{attempt.get('prediction')}（族「{quick['task'].get('build_family')}」）")
    if problems:
        raise SelftestFailure("编译族只借编译那一段：" + "；".join(problems), world.server_log_lines())
    return f"Finished 之前 6G、之后 40G：记编译那一段 6G；同族另一条从 {PHASE_RAISED_TEXT} 起，不借跑测试那一段与没有 Finished 行的峰值"


def cell_cleanup_protocol_unit_tests(world):
    """清场协议的离线单测（task-server/tests/test_cleanup_protocol.py；unittest，在这一格自己的进程里跑，不起服务）：恰好 CLEANUP_PROTOCOL_TESTS 条、全过才绿；
    它的假调度器照 Scheduler.__init__ 的表名现建（scheduler_tables）。弄坏开关 cleanup-tests-tables-hand-listed 下照手抄的旧名单建、收尾那几条红，这一格红。"""
    module = lib.load_module("test_cleanup_protocol", os.path.join(lib.SCRIPT_DIRECTORY, "tests", "test_cleanup_protocol.py"))
    suite = unittest.defaultTestLoader.loadTestsFromModule(module)
    stream = io.StringIO()
    result = unittest.TextTestRunner(stream=stream, verbosity=2).run(suite)
    bad = [test.id() for test, _ in result.failures + result.errors] + [test.id() for test, _ in result.skipped]
    if result.testsRun != CLEANUP_PROTOCOL_TESTS or bad or suite.countTestCases() != CLEANUP_PROTOCOL_TESTS:
        raise SelftestFailure(f"清场协议单测应当跑 {CLEANUP_PROTOCOL_TESTS} 条、全过，实际载入 {suite.countTestCases()} 条、跑了 {result.testsRun} 条，不对或跳过的 {bad}",
                              stream.getvalue().splitlines()[-40:])
    return f"清场协议单测 {result.testsRun} 条全过（假调度器的表照 Scheduler.__init__ 现建）"


SEAL_FIT_HOLDER_PEAK_BYTES = 54 * GIBIBYTE    # whole-not-sealed-where-it-cannot-fit 本机占着的那一条：上限与记账 81G（本机 100G、可用上限 98G、界 83.3G：碎片；第二台 48G 放不下它）
SEAL_FIT_WHOLE_PEAK_BYTES = 60 * GIBIBYTE     # 那一格的整片：上限与记账 90G；峰值本身比第二台此刻能给的最大上限 48G 大，在本机放得下（不小于本机的界）
SEAL_FIT_FRAGMENT_PEAK_BYTES = 10 * GIBIBYTE  # 那一格的碎片：上限与记账各 15G；第二台一组装两条（30G）再放就超过界 40.8G，内存装得下三条（45G）
SEAL_FIT_NOWHERE_PEAK_BYTES = 100 * GIBIBYTE  # 那一格两台都装不下的整片：峰值本身比两台此刻能给的最大上限（本机 98G、第二台 48G）都大，记账 150G 收不到上限为止（booked_at_cap 只收峰值装得下的）
SEAL_FIT_FRAGMENTS = 6
SEAL_FIT_PEER_FITS = 3                        # 第二台不为整片封组时照内存起的碎片条数（3 × 15G = 45G ≤ 48G）；为它封组时只起到两条
SEAL_FIT_RELEASE = "release-seal-fit"         # 那一格占着的与碎片的放行文件（相对假仓根；假第二台照 fake-peer-wait 等同一个名字）
FILLS_CAP_CLIMB_CAPS = (6144 * MEBIBYTE, 9216 * MEBIBYTE, 13824 * MEBIBYTE)   # fills-any-cap-cap-hits-not-reported：照上限一档档撞上去的三次（6.0、9.0、13.5 GiB）
FILLS_CAP_CLIMB_PEAK_PERCENT = 99             # 那三次撞上限时的峰值占各自上限的百分数（撞的那一次峰值本来就贴着上限）


def cell_whole_not_sealed_where_it_cannot_fit(world):
    """只在放得下整片的台上为它封组（task_server.py 文件头 ②「整片与碎片按台交替」；Alternation.__init__ 的 fitting）：本机 100G（可用上限 98G、界 83.3G），第二台 50G（48G、界 40.8G）。
    本机先起一条占着的（记账 81G，第二台放不下它）；再交一条整片（峰值 60G、记账 90G：在两台都算整片，峰值本身比第二台的 48G 大，在本机放得下）与六条碎片（各记 15G，跑到放行为止）。
    峰值只比一台的 host_limit 大：整片排着等本机、只封本机（排队原因里有本机封组的那一句）；第二台不为它封组，照内存起三条碎片，排队原因里没有第二台封组的那一句。
    之后另交一条峰值本身比两台的 host_limit 都大的（峰值 100G、记账 150G；峰值装得下的记账照 booked_at_cap 收到 host_limit 为止、照样起得来，不是这一判的场景）：
    不封组、一次都不起，记 memory-cap-at-maximum。
    弄坏开关 seal-host-that-cannot-fit 下照改前第二台也算等它的台：它在第二台离放得下更近（90G − 48G 比 90G − 17G 小），第二台为它封组、只起两条，这一格红。"""
    world.resources(local=(100, 100), peer=(50, 50), peer_cap_gibibytes=50)
    for key, peak in (("selftest-seal-fit-holder", SEAL_FIT_HOLDER_PEAK_BYTES), ("selftest-seal-fit-whole", SEAL_FIT_WHOLE_PEAK_BYTES),
                      ("selftest-seal-fit-fragment", SEAL_FIT_FRAGMENT_PEAK_BYTES), ("selftest-seal-fit-nowhere", SEAL_FIT_NOWHERE_PEAK_BYTES)):
        world.seed_peak(key, peak)
    world.start_server()
    release = os.path.join(world.repo, SEAL_FIT_RELEASE)
    problems, counts, reasons, previous, whole_summary = [], {}, [], None, {"attempts": []}
    holder = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "seal-fit-holder", SEAL_FIT_RELEASE, f"fake-peer-wait={SEAL_FIT_RELEASE}"],
                          extra={lib.KEY_VARIABLE: "selftest-seal-fit-holder"}, cwd=world.repo)
    try:
        held = world.running(holder)
        if first_attempt(held).get("host") != SERVER.LOCAL:
            problems.append(f"占着的那一条（记账 81G）应当起在本机，实际 {describe(held)}")
        world.resources(local=(100, 100), peer=(50, 50), peer_cap_gibibytes=50, hold=True)
        whole = world.submit(["bash", "-c", "exit 0", "seal-fit-whole"], extra={lib.KEY_VARIABLE: "selftest-seal-fit-whole"}, cwd=world.repo)
        fragments = [world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, f"seal-fit-fragment-{number}", SEAL_FIT_RELEASE, f"fake-peer-wait={SEAL_FIT_RELEASE}"],
                                  extra={lib.KEY_VARIABLE: "selftest-seal-fit-fragment"}, cwd=world.repo) for number in range(SEAL_FIT_FRAGMENTS)]
        world.resources(local=(100, 100), peer=(50, 50), peer_cap_gibibytes=50)   # 一齐放出来
        world.api("POST", "/wake")
        deadline = time.monotonic() + SEALED_ONE_HOST_LIMIT_SECONDS
        while time.monotonic() < deadline:
            world.next_pass()
            time.sleep(SEALED_ONE_HOST_SETTLE_SECONDS)
            counts, reasons = started_hosts(world, fragments)
            if all(reason and "这一轮已起" not in reason for reason in reasons) and counts == previous:
                break
            previous = counts
        whole_summary = world.summary(whole)
    finally:
        open(release, "w").close()
    outcomes = [world.finished(task_id, timeout=120)["task"].get("outcome") for task_id in [holder, *fragments, whole]]
    peer_name = lib.HOST_NAMES.get(SERVER.PEER, SERVER.PEER)
    sealed_on_peer = [reason for reason in reasons if f"{peer_name}：这台这一组碎片已装到" in reason]
    local_name = lib.HOST_NAMES.get(SERVER.LOCAL, SERVER.LOCAL)
    sealed_on_local = [reason for reason in reasons if f"{local_name}：这台这一组碎片已装到" in reason and "封组" in reason]
    if counts != previous:
        problems.append(f"{SEALED_ONE_HOST_LIMIT_SECONDS:.0f} 秒里两台起的碎片条数一直在变：最后一次 {counts}，前一次 {previous}")
    if whole_summary["attempts"]:
        problems.append(f"本机被占着时整片（记账 90G）应当排着，实际 {describe(whole_summary)}")
    if sealed_on_peer:
        problems.append(f"整片峰值本身比第二台能给的最大上限大，第二台不该为它封组，实际排队原因「{sealed_on_peer[0][:300]}」")
    if not sealed_on_local:
        problems.append(f"整片峰值只比第二台的最大上限大、本机放得下：应当只封本机等它，实际没起的碎片的排队原因里没有本机封组（{[reason[:200] for reason in reasons[:2]]}）")
    if counts.get(SERVER.PEER, 0) < SEAL_FIT_PEER_FITS:
        problems.append(f"第二台不为整片封组时应当照内存起 {SEAL_FIT_PEER_FITS} 条碎片，实际两台各起了 {counts}")
    if set(outcomes) != {"succeeded"}:
        problems.append(f"放行之后都应当跑完，实际结局 {sorted(set(outcomes))}")
    nowhere = world.finished(world.submit(["bash", "-c", "exit 0", "seal-fit-nowhere"], extra={lib.KEY_VARIABLE: "selftest-seal-fit-nowhere"}, cwd=world.repo), timeout=120)
    if nowhere["task"].get("outcome") != "memory-cap-at-maximum" or nowhere["attempts"]:
        problems.append(f"峰值本身比两台的最大上限都大的整片（峰值 100G、记账 150G）应当不封组、一次都不起、记 memory-cap-at-maximum，实际 {describe(nowhere)}")
    if problems:
        raise SelftestFailure("只在放得下整片的台上封组：" + "；".join(problems), reasons[:4] + world.server_log_lines())
    return f"整片峰值只比第二台大：只封本机，第二台照内存起了 {counts.get(SERVER.PEER, 0)} 条碎片；峰值比两台都大的打回 memory-cap-at-maximum"


def cell_fills_any_cap_cap_hits_not_reported(world):
    """照上限一档档撞上去的不算按上限开并行（task_server.py 文件头 ①「要处置的提醒」的 fills-any-cap；任务 9616 那条静态核 6.0、9.0、13.5 GiB）：
    同一命令键记下三次撞了上限（memory-cap-hit）的尝试，峰值都是各自上限的 99%；再交一条：确认单「交的时候判得出的异常」里没有 fills-any-cap，wait 不打 kind=fills-any-cap，跑完。
    同键三次跑成都贴着上限的照报，由 fills-any-cap-reported 那一格钉。弄坏开关 fills-any-cap-counts-cap-hits 下撞了上限的也算进三次，确认单里列出 fills-any-cap，这一格红。"""
    key = "selftest-fills-any-cap-climb"
    for cap in FILLS_CAP_CLIMB_CAPS:
        world.seed_attempt({"command_key": key, "outcome": "memory-cap-hit", "memory_peak": cap * FILLS_CAP_CLIMB_PEAK_PERCENT // 100, "memory_cap": cap})
    world.resources(local=(100, 100))
    world.start_server()
    problems = []
    submitted = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", "fills-any-cap-climb"], extra={lib.KEY_VARIABLE: key})
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", submitted.stdout)
    if submitted.returncode != 0 or not found or lib.confirm_needed_fields(submitted.stdout) is None:
        raise SelftestFailure(f"交了应当退 0、打任务号与确认单，实际退 {submitted.returncode}", [submitted.stdout, submitted.stderr] + world.server_log_lines())
    task_id = int(found.group(1))
    if "fills-any-cap：" in submitted.stdout:
        problems.append(f"三次都是撞了上限的，确认单里不该列 fills-any-cap，实际「{submitted.stdout.strip()[-500:]}」")
    world.confirm_submitted(submitted)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    if "kind=fills-any-cap" in completed.stdout:
        problems.append(f"三次都是撞了上限的，wait 不该打 kind=fills-any-cap，实际「{completed.stdout.strip()[-300:]}」")
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("照上限一档档撞上去的：" + "；".join(problems), world.server_log_lines())
    return "同键三次都是撞了上限、峰值贴着上限：确认单与 wait 都不报 fills-any-cap，照跑完"


CELLS = {
    "pause-allows-one-session": cell_pause_allows_one_session, "pause-allow-waiting-text": cell_pause_allow_waiting_text,
    "pause-allow-switches-session": cell_pause_allow_switches_session, "resume-clears-allowed-session": cell_resume_clears_allowed_session,
    "pause-allow-survives-restart": cell_pause_allow_survives_restart, "pause-allow-reported": cell_pause_allow_reported,
    "pause-holds-new-attempts": cell_pause_holds_new_attempts, "pause-lets-running-finish": cell_pause_lets_running_finish,
    "pause-survives-restart": cell_pause_survives_restart, "resume-releases-queue": cell_resume_releases_queue, "pause-reported": cell_pause_reported,
    "pause-not-starved": cell_pause_not_starved, "pause-waits-for-starting-attempt": cell_pause_waits_for_starting_attempt,
    "attention-queued-too-long": cell_attention_queued_too_long, "attention-overrun": cell_attention_overrun, "attention-card-replaced": cell_attention_card_replaced,
    "attention-card-replaced-after-finish": cell_attention_card_replaced_after_finish, "attention-cleanup-pending": cell_attention_cleanup_pending,
    "attention-shard-advance-failed": cell_attention_shard_advance_failed, "attention-reported-once": cell_attention_reported_once,
    "attention-reported-survives-restart": cell_attention_reported_survives_restart, "attention-queue-waits-for-reason": cell_attention_queue_waits_for_reason,
    "page-cache-not-counted-as-process-memory": cell_page_cache_not_counted_as_process_memory,
    "attention-type-in-header": cell_attention_type_in_header, "temporary-service-wrapper-compiles": cell_temporary_service_wrapper_compiles,
    "temporary-service-own-server-compiles": cell_temporary_service_own_server_compiles, "cap-hit-rerun-from-record": cell_cap_hit_rerun_from_record,
    "slow-world-patient-wait": cell_slow_world_patient_wait,
    "shard-siblings-share-peaks": cell_shard_siblings_share_peaks, "build-family-peak-shared": cell_build_family_peak_shared,
    "build-family-declared": cell_build_family_declared, "build-family-from-arguments": cell_build_family_from_arguments,
    "build-family-cap-hit-raises": cell_build_family_cap_hit_raises, "build-family-peak-windowed": cell_build_family_peak_windowed,
    "parallelism-registration-refused": cell_parallelism_registration_refused,
    "shard-child-key-folded-by-words": cell_shard_child_key_folded_by_words,
    "workspace-write-interrupted-reported": cell_workspace_write_interrupted_reported,
    "workspace-write-complete-not-reported": cell_workspace_write_complete_not_reported,
    "gpu-two-cards-same-host": cell_gpu_two_cards_same_host, "gpu-three-cards-only-where-enough": cell_gpu_three_cards_only_where_enough,
    "gpu-cards-short-never-fits": cell_gpu_cards_short_never_fits,
    "check-missing": cell_check_missing, "install": cell_install, "control-readback": cell_control_readback, "server-unreachable": cell_server_unreachable,
    "exit-code-passthrough": cell_exit_code_passthrough, "restart-keeps-running": cell_restart_keeps_running, "start-once": cell_start_once,
    "queue-holds-unplaceable": cell_queue_holds_unplaceable, "arrival-order": cell_arrival_order,
    "prediction-computed-once": cell_prediction_computed_once, "pass-reads-resources-once": cell_pass_reads_resources_once,
    "pass-booking-carried": cell_pass_booking_carried, "launch-rechecked": cell_launch_rechecked, "every-command-reaches-peer": cell_every_command_reaches_peer,
    "backfill-when-head-blocked": cell_backfill_when_head_blocked, "error-types": cell_error_types,
    "unit-start-failed": cell_unit_start_failed, "memory-cap-growth": cell_memory_cap_growth, "memory-cap-at-maximum": cell_memory_cap_at_maximum,
    "unrecorded-default-small": cell_unrecorded_default_small, "unrecorded-cap-growth": cell_unrecorded_cap_growth,
    "unrecorded-burst-balanced": cell_unrecorded_burst_balanced,
    "retry-kinds": cell_retry_kinds, "gpu-highest-free-card": cell_gpu_highest_free_card, "gpu-entry-scripts-recognized": cell_gpu_entry_scripts_recognized, "cancel-own-session-only": cell_cancel_own_session_only,
    "tampered-row-not-started": cell_tampered_row_not_started, "queued-revalidated-on-restart": cell_queued_revalidated_on_restart, "forged-session": cell_forged_session, "environment-whitelist": cell_environment_whitelist,
    "retired-variables-refused": cell_retired_variables_refused,
    "nested-dispatch-refused": cell_nested_dispatch_refused, "resource-record": cell_resource_record, "operation-log": cell_operation_log,
    "unclaimed-finished": cell_unclaimed_finished, "state-placement": cell_state_placement,
    "inbox-outcome-succeeded": cell_inbox_outcome_succeeded, "inbox-outcome-failed": cell_inbox_outcome_failed,
    "inbox-queued-too-long-once": cell_inbox_queued_too_long_once, "inbox-unreachable-task-runs": cell_inbox_unreachable_task_runs,
    "health-database-unwritable-notified": cell_health_database_unwritable_notified, "health-unit-missing-notified": cell_health_unit_missing_notified,
    "health-nfs-notified": cell_health_nfs_notified, "health-peer-unavailable-notified": cell_health_peer_unavailable_notified,
    "control-watch-removed": cell_control_watch_removed, "unclaimed-results-warned": cell_unclaimed_results_warned,
    "retry-keeps-earlier-attempts": cell_retry_keeps_earlier_attempts, "build-directory-outside-repository": cell_build_directory_outside_repository,
    "retry-build-directory-reused": cell_retry_build_directory_reused,
    "history-peak-first-attempt": cell_history_peak_first_attempt, "history-peak-window-raises": cell_history_peak_window_raises,
    "history-peak-retry-jumps": cell_history_peak_retry_jumps,
    "history-peak-unrecorded-default": cell_history_peak_unrecorded_default, "writer-dry-run-retried": cell_writer_dry_run_retried,
    "health-head-cpu-gate-not-starved": cell_health_head_cpu_gate_not_starved, "health-head-cards-short-not-starved": cell_health_head_cards_short_not_starved,
    "whole-seals-one-host": cell_whole_seals_one_host, "card-whole-seals-while-card-busy": cell_card_whole_seals_while_card_busy,
    "aged-whole-keeps-group-sealed": cell_aged_whole_keeps_group_sealed,
    "aged-seal-no-gap-fill": cell_aged_seal_no_gap_fill, "budget-over-maxima-refused": cell_budget_over_maxima_refused,
    "oversized-budget-reported": cell_oversized_budget_reported, "long-run-idle-cores-reported": cell_long_run_idle_cores_reported,
    "fills-any-cap-reported": cell_fills_any_cap_reported,
    "old-metric-peaks-not-counted": cell_old_metric_peaks_not_counted,
    "anomaly-table-matches-kinds": cell_anomaly_table_matches_kinds, "anomaly-advice-in-texts": cell_anomaly_advice_in_texts,
    "anomaly-repushed-with-history": cell_anomaly_repushed_with_history, "unclaimed-outcome-repushed": cell_unclaimed_outcome_repushed,
    "inbox-socket-follows-session": cell_inbox_socket_follows_session, "submit-awaits-confirmation": cell_submit_awaits_confirmation,
    "unconfirmed-cancelled": cell_unconfirmed_cancelled, "family-lends-build-phase-only": cell_family_lends_build_phase_only,
    "cleanup-protocol-unit-tests": cell_cleanup_protocol_unit_tests,
    "whole-not-sealed-where-it-cannot-fit": cell_whole_not_sealed_where_it_cannot_fit,
    "fills-any-cap-cap-hits-not-reported": cell_fills_any_cap_cap_hits_not_reported,
    "busy-cpu-pinned-at-cap-retried": cell_busy_cpu_pinned_at_cap_retried, "busy-cpu-below-cap-keeps-running": cell_busy_cpu_below_cap_keeps_running,
    "peer-pinned-at-cap-busy-cpu-retried": cell_peer_pinned_at_cap_busy_cpu_retried, "attention-quiet-busy-log": cell_attention_quiet_busy_log,
    "io-wait-local-not-stalled": cell_io_wait_local_not_stalled, "io-wait-peer-not-stalled": cell_io_wait_peer_not_stalled,
    "gpu-busy-quiet-not-stalled": cell_gpu_busy_quiet_not_stalled, "card-zero-utilization-stalled": cell_card_zero_utilization_stalled,
    "card-idle-polling-stalled": cell_card_idle_polling_stalled, "card-idle-cpu-computing-not-stalled": cell_card_idle_cpu_computing_not_stalled,
    "peer-health-slow-rechecked": cell_peer_health_slow_rechecked, "cap-hit-stat-in-note": cell_cap_hit_stat_in_note,
    "over-two-hours-bounced": cell_over_two_hours_bounced, "over-two-hours-across-cap-hits": cell_over_two_hours_across_cap_hits,
    "over-two-hours-commit-gate-exempt": cell_over_two_hours_commit_gate_exempt, "over-two-hours-check-not-exempt": cell_over_two_hours_check_not_exempt,
    "over-two-hours-merge-and-verify-not-exempt": cell_over_two_hours_merge_and_verify_not_exempt, "requeue-goes-to-tail": cell_requeue_goes_to_tail,
    "long-run-confirmed-not-bounced": cell_long_run_confirmed_not_bounced, "requeue-refusals": cell_requeue_refusals,
    "whole-package-unconfirmed-refused": cell_whole_package_unconfirmed_refused, "whole-package-named-accepted": cell_whole_package_named_accepted,
    "whole-package-confirmation-form": cell_whole_package_confirmation_form, "whole-package-confirmed-still-bounced": cell_whole_package_confirmed_still_bounced,
    "sealed-gap-room-takes-unestimated": cell_sealed_gap_room_takes_unestimated,
    "both-hosts-sealed-small-unestimated-starts": cell_both_hosts_sealed_small_unestimated_starts, "fragment-group-drops-finished": cell_fragment_group_drops_finished,
    "whole-workspace-cargo-refused": cell_whole_workspace_cargo_refused, "workspace-writer-initial-cap-headroom": cell_workspace_writer_initial_cap_headroom,
    "progress-stall-reported": cell_progress_stall_reported, "card-wait-holds-only-its-host": cell_card_wait_holds_only_its_host,
    "shard-siblings-peak-windowed": cell_shard_siblings_peak_windowed,
    "peer-retry-build-directory-reused": cell_peer_retry_build_directory_reused,
    "repository-target-bypass-warned": cell_repository_target_bypass_warned, "cancelled-artifacts-removed": cell_cancelled_artifacts_removed,
    "cancelled-before-peer-trip-built": cell_cancelled_before_peer_trip_built, "inbox-outcomes-resent-after-error": cell_inbox_outcomes_resent_after_error,
    "thirty-second-tick-prompt": cell_thirty_second_tick_prompt, "starved-head-reported": cell_starved_head_reported,
    "blocked-head-not-starved": cell_blocked_head_not_starved, "starved-head-judged-by-the-formula": cell_starved_head_judged_by_the_formula,
    "aged-task-judged-first": cell_aged_task_judged_first,
    "unmeasured-not-zero": cell_unmeasured_not_zero, "queued-estimate": cell_queued_estimate, "retry-goes-where-it-fits": cell_retry_goes_where_it_fits,
    "peer-runs-in-shared-tree": cell_peer_runs_in_shared_tree, "clean-failure-explained": cell_clean_failure_explained, "stalled-attempt-stopped": cell_stalled_attempt_stopped, "cpu-unread-not-stalled": cell_cpu_unread_not_stalled,
    "running-status-shows-progress": cell_running_status_shows_progress, "stall-waits-for-grace": cell_stall_waits_for_grace, "computing-quietly-not-stalled": cell_computing_quietly_not_stalled, "overrun-quiet-log-stopped": cell_overrun_quiet_log_stopped, "peer-stall-judged-by-peer-cpu": cell_peer_stall_judged_by_peer_cpu, "simulated-two-host-workload": cell_simulated_two_host_workload, "three-duties-two-sessions": cell_three_duties_two_sessions,
    "hundred-five-minute-tasks-balanced": cell_hundred_five_minute_tasks_balanced, "stalled-at-cap-retried": cell_stalled_at_cap_retried, "aged-head-at-slice-total-starts": cell_aged_head_at_slice_total_starts, "peer-run-failed-retried": cell_peer_run_failed_retried, "squeezed-at-total-is-cap-hit": cell_squeezed_at_total_is_cap_hit, "peer-starts-throttled": cell_peer_starts_throttled, "attempt-unit-nofile-raised": cell_attempt_unit_nofile_raised, "queued-shows-waiting-reason": cell_queued_shows_waiting_reason, "failed-run-estimated": cell_failed_run_estimated,
    "unfinished-peak-ignored": cell_unfinished_peak_ignored, "completed-after-cap-hit": cell_completed_after_cap_hit,
    "cap-hit-starts-above": cell_cap_hit_starts_above,
    "balanced-when-both-fit": cell_balanced_when_both_fit, "peer-when-local-full": cell_peer_when_local_full, "shard-to-peer-by-plan": cell_shard_to_peer_by_plan,
    "shard-siblings-share-a-host": cell_shard_siblings_share_a_host,
    "shard-cap-fitted-to-room": cell_shard_cap_fitted_to_room, "shard-keys-kept-apart": cell_shard_keys_kept_apart,
    "socket-path-too-long": cell_socket_path_too_long, "threads-fixed": cell_threads_fixed,
    "shard-retry-isolated": cell_shard_retry_isolated,
    "task-log": cell_task_log, "cancel-queued": cell_cancel_queued, "cancel-finished": cell_cancel_finished,
    "cancel-sharded-in-plan": cell_cancel_sharded_in_plan, "cancel-sharded-running": cell_cancel_sharded_running,
    "shard-plan-failed": cell_shard_plan_failed, "shard-merge-failed": cell_shard_merge_failed, "unsharded-arguments": cell_unsharded_arguments,
    "control-start": cell_control_start, "gpu-without-cards-unmet": cell_gpu_without_cards_unmet, "pinned-peer-unavailable": cell_pinned_peer_unavailable,
    "gpu-runs-only-when-code-runs": cell_gpu_runs_only_when_code_runs,
    "stall-clock-from-log-mtime": cell_stall_clock_from_log_mtime,
    "memory-cap-passed-to-command": cell_memory_cap_passed_to_command, "source-repository-passed-to-command": cell_source_repository_passed_to_command,
    "shared-roots-passed-to-command": cell_shared_roots_passed_to_command, "card-labels-by-address": cell_card_labels_by_address,
    "peer-admission-marked": cell_peer_admission_marked, "peer-disk-floor": cell_peer_disk_floor, "suspected-stall-notified": cell_suspected_stall_notified,
    "suspected-stall-card-busy-not-reported": cell_suspected_stall_card_busy_not_reported, "pinned-cap-from-configuration": cell_pinned_cap_from_configuration,
    "pinned-cap-unreadable-refused": cell_pinned_cap_unreadable_refused,
    "pinned-cap-by-host": cell_pinned_cap_by_host, "by-host-missing-host-refused": cell_by_host_missing_host_refused,
    "by-host-outside-address-refused": cell_by_host_outside_address_refused, "shard-host-is-address": cell_shard_host_is_address,
    "gate-stages-sent-to-peer": cell_gate_stages_sent_to_peer,
    "admission-booked-at-cap": cell_admission_booked_at_cap, "health-answers-while-resources-slow": cell_health_answers_while_resources_slow,
    "python-entry-gpu-recognized": cell_python_entry_gpu_recognized, "gpu-argument-declaration-recognized": cell_gpu_argument_declaration_recognized,
    "leftover-processes-stopped": cell_leftover_processes_stopped, "leftovers-only-in-own-unit": cell_leftovers_only_in_own_unit, "long-run-failure-noted": cell_long_run_failure_noted,
    "leftovers-error-isolated": cell_leftovers_error_isolated, "inbox-shard-attention-merged": cell_inbox_shard_attention_merged,
    "attempt-label-passed-to-command": cell_attempt_label_passed_to_command,
    "whole-and-fragment-alternate": cell_whole_and_fragment_alternate, "whole-starts-beside-running-fragment": cell_whole_starts_beside_running_fragment,
    "drained-sealed-group-reopens": cell_drained_sealed_group_reopens,
    "ended-attempt-settled-after-restart": cell_ended_attempt_settled_after_restart,
    "aged-tasks-do-not-hold-fitting-tasks": cell_aged_tasks_do_not_hold_fitting_tasks,
    "cpu-gate-holds-busy-host": cell_cpu_gate_holds_busy_host, "unstarted-slice-uses-configured-total": cell_unstarted_slice_uses_configured_total, "down-host-waits-not-never": cell_down_host_waits_not_never, "card-fault-replaced-on-another-card": cell_card_fault_replaced_on_another_card,
    "card-fault-quarantines-named-card": cell_card_fault_quarantines_named_card, "card-fault-unlisted-quarantines-none": cell_card_fault_unlisted_quarantines_none,
    "failed-peak-floor": cell_failed_peak_floor,
    "environment-prefix-stripped-from-key": cell_environment_prefix_stripped_from_key,
    "verdict-library-family-refused": cell_verdict_library_family_refused, "workspace-writer-not-retried": cell_workspace_writer_not_retried,
    "retry-cap-not-floored": cell_retry_cap_not_floored, "recorded-cap-not-floored": cell_recorded_cap_not_floored,
    "sealed-gap-takes-short-task": cell_sealed_gap_takes_short_task, "sealed-gap-holds-longer-task": cell_sealed_gap_holds_longer_task,
    "unestimated-blocker-counted-by-elapsed": cell_unestimated_blocker_counted_by_elapsed, "margin-measured-per-host": cell_margin_measured_per_host,
    "gap-fill-by-room-skips-length": cell_gap_fill_by_room_skips_length, "gap-fill-room-exhausted-checks-length": cell_gap_fill_room_exhausted_checks_length,
    "item-estimate-before-queue": cell_item_estimate_before_queue, "item-done-rates-recorded": cell_item_done_rates_recorded,
    "item-estimate-summed-from-rates": cell_item_estimate_summed_from_rates, "item-estimate-failure-queues-parent": cell_item_estimate_failure_queues_parent,
    "item-estimate-cancelled": cell_item_estimate_cancelled,
    "estimate-by-folded-command-key": cell_estimate_by_folded_command_key, "estimate-basis-command-sentence": cell_estimate_basis_command_sentence,
    "estimate-basis-items-sentence": cell_estimate_basis_items_sentence, "estimate-basis-none-sentence": cell_estimate_basis_none_sentence,
    "shard-plan-and-merge-estimated-by-command": cell_shard_plan_and_merge_estimated_by_command,
    "shard-pieces-estimated-by-own-history": cell_shard_pieces_estimated_by_own_history,
    "margin-unmeasured-not-placed": cell_margin_unmeasured_not_placed, "pin-cap-refused-within-margin": cell_pin_cap_refused_within_margin,
    "peer-booked-like-local": cell_peer_booked_like_local, "peer-scope-missing-booked-at-cap": cell_peer_scope_missing_booked_at_cap,
    "two-hosts-booked-within-capacity": cell_two_hosts_booked_within_capacity, "judge-error-skips-only-that-task": cell_judge_error_skips_only_that_task,
    "explicit-wait-not-starved": cell_explicit_wait_not_starved,
    "nfs-share-required": cell_nfs_share_required, "nfs-share-verified": cell_nfs_share_verified,
    "peer-unavailable-alerted": cell_peer_unavailable_alerted, "reason-line-per-type": cell_reason_line_per_type,
    "selftest-variables-refused": cell_selftest_variables_refused, "type-table-covered": cell_type_table_covered,
    "cancel-start-race": cell_cancel_start_race, "sharded-progress-recovered": cell_sharded_progress_recovered,
    "attempt-claim-atomic": cell_attempt_claim_atomic, "sharded-cancel-retried": cell_sharded_cancel_retried,
    "association-column-verified": cell_association_column_verified, "start-timeout-read-back": cell_start_timeout_read_back,
    "build-disk-predicted": cell_build_disk_predicted, "peer-command-exit-split": cell_peer_command_exit_split,
    "nfs-automount-recognized": cell_nfs_automount_recognized, "peer-memory-peak-recorded": cell_peer_memory_peak_recorded,
    "prune-server-unavailable-outcome": cell_prune_server_unavailable_outcome,
    "temporary-service-wrapper-unwrapped": cell_temporary_service_wrapper_unwrapped,
    "temporary-service-heavy-refused": cell_temporary_service_heavy_refused,
    "data-root-key-required": cell_data_root_key_required,
    "outside-shared-path-refused": cell_outside_shared_path_refused, "path-check-whole-words": cell_path_check_whole_words,
    "shared-roots-outside-refused": cell_shared_roots_outside_refused,
    "finished-build-removed": cell_finished_build_removed,
    "nfs-probe-concurrent-runs": cell_nfs_probe_concurrent_runs, "gpu-binary-declaration-recognized": cell_gpu_binary_declaration_recognized,
    "short-command-peak-recorded": cell_short_command_peak_recorded, "merge-on-peer-reads-shard-outputs": cell_merge_on_peer_reads_shard_outputs,
    "stop-ends-probe-children": cell_stop_ends_probe_children, "failure-scene-kept-until-collected": cell_failure_scene_kept_until_collected,
    "shard-failure-waits-for-running": cell_shard_failure_waits_for_running, "shard-cancel-alone-merges": cell_shard_cancel_alone_merges,
    "parent-cancel-stops-every-shard": cell_parent_cancel_stops_every_shard, "cancel-after-natural-end": cell_cancel_after_natural_end,
    "failed-start-not-counted": cell_failed_start_not_counted, "child-rows-verified": cell_child_rows_verified, "task-locks-released": cell_task_locks_released,
}
UNPREPARED_CELLS = ("install",)


def run_cell(name, directory):
    if not lib.in_selftest():
        print("  ✗ --selftest-cell 只由 task_server_lib.py --selftest 起（它建自证的标记）", file=sys.stderr)
        print("  → 怎么办：跑 python3 task-server/task_server_lib.py --selftest；只跑几格设 TASK_SERVER_SELFTEST_CELLS=<格名,…>", file=sys.stderr)
        return lib.USAGE_EXIT
    if name not in CELLS:
        print(f"  ✗ 没有这一格：{name}（有 {'、'.join(CELLS)}）", file=sys.stderr)
        print("  → 怎么办：照上面的格名写", file=sys.stderr)
        return lib.USAGE_EXIT
    os.makedirs(directory, exist_ok=True)
    world = World(name, directory, prepare=name not in UNPREPARED_CELLS)
    try:
        detail = CELLS[name](world)
        print(f"  · 对 {name}：{detail}")
        return 0
    except SelftestFailure as failure:
        print(f"  {FAILURE_PREFIX}{name}：{failure}")   # gate-lint:detail
        for line in failure.extra_lines:
            for piece in str(line).splitlines()[-12:]:
                print(f"       | {piece}")
        return 1
    except Exception as error:   # 格子自己出事也照这一格不对报，不吞
        print(f"  {FAILURE_PREFIX}{name}：格子出了异常（{type(error).__name__}: {error}）")   # gate-lint:detail
        for piece in traceback.format_exc().splitlines()[-12:]:
            print(f"       | {piece}")
        return 1
    finally:
        world.close()


def short_temporary_base():
    """各格 socket 目录的公共上一层建在哪（task_server_lib.py 文件头「自证」）：系统临时目录接上最长那一条 socket 路径放得下就用它，放不下用 /tmp。"""
    base = tempfile.gettempdir()
    if len(os.fsencode(base)) + SOCKET_PATH_ALLOWANCE <= lib.SOCKET_PATH_MAXIMUM_BYTES:
        return base
    return FALLBACK_TEMPORARY_BASE


def parallelism(environment=None, registration_file=None):
    """各格并行跑的路数（run_all）：每台默认重活线程数的登记值，scripts/hosts/heavy_threads.py 的 default_heavy_threads 现读（SINGLEFS_THREAD_CAP 设了照它）。
    → (路数, 取不出的原因, 出路)。取不出登记值（登记文件不在、读不了、写坏，SINGLEFS_THREAD_CAP 不是正整数，heavy_threads.py 载不进来）交 (None, 原因, 出路)，
    由 run_all 判红，不退回按核数（.claude/heavy-thread-default 文件头：写坏或删掉这份文件，取法一律拒绝，不退回按核数算）。
    environment、registration_file 只给 parallelism-registration-refused 这一格造样本用，不给就是此刻的环境与仓里的登记文件。
    弄坏开关 parallelism-falls-back-to-cores 下取不出就按核数（改前的样子）。"""
    environment = os.environ if environment is None else environment
    try:
        module = lib.load_module("heavy_threads", os.path.join(lib.HOSTS_DIRECTORY, "heavy_threads.py"))
        return max(1, int(module.default_heavy_threads(environment, registration_file=registration_file))), "", ""
    except Exception as error:
        if lib.broken("parallelism-falls-back-to-cores"):
            return max(1, os.cpu_count() or 1), "", ""
        return None, str(error) if hasattr(error, "howto") else f"{type(error).__name__}: {error}", getattr(error, "howto", "") or "看 scripts/hosts/heavy_threads.py 与 .claude/heavy-thread-default 的文件头"


def run_all(arguments):
    items = [item for item in os.environ.get(lib.BREAK_VARIABLE, "").split(",") if item]
    unknown = [item for item in items if item not in lib.KNOWN_BREAK_ITEMS]
    if unknown:
        print(f"  ✗ {lib.BREAK_VARIABLE} 里认不出的项：{'、'.join(unknown)}")
        print("  → 怎么办：照 task_server_lib.py 文件头的开关表写")
        return lib.USAGE_EXIT
    selected = [name for name in os.environ.get("TASK_SERVER_SELFTEST_CELLS", "").split(",") if name] or list(CELLS)
    wrong_names = [name for name in selected if name not in CELLS]
    if wrong_names:
        print(f"  ✗ TASK_SERVER_SELFTEST_CELLS 里没有的格：{'、'.join(wrong_names)}（有 {'、'.join(CELLS)}）")
        print("  → 怎么办：照上面的格名写，或不设它跑全部格")
        return lib.USAGE_EXIT
    parallel, reason, howto = parallelism()
    if parallel is None:   # 取不出登记的线程数：判红，不退回按核数；还没建临时目录，没有要收的
        print(f"  ✗ task_server_lib.py 自证：并行度取不出每台默认重活线程数（scripts/hosts/heavy_threads.py）：{reason}")
        print(f"  → 怎么办：{howto}")
        return 1
    root = tempfile.mkdtemp(prefix="tss-")
    socket_root = tempfile.mkdtemp(prefix=SOCKET_ROOT_PREFIX, dir=short_temporary_base())
    mark = os.path.join(root, "selftest-mark")
    with open(mark, "w", encoding="utf-8") as handle:
        handle.write(lib.SELFTEST_MARK_TEXT + "\n")
    environment = dict(os.environ, **{lib.SELFTEST_MARK_VARIABLE: mark, SOCKET_ROOT_VARIABLE: socket_root})
    workers = min(parallel, len(selected))
    library = os.path.join(lib.SCRIPT_DIRECTORY, "task_server_lib.py")

    def one(name):
        output_path = os.path.join(root, f"{name}.out")
        with open(output_path, "w", encoding="utf-8") as output:
            try:
                completed = subprocess.run([sys.executable, "-B", library, "--selftest-cell", name, os.path.join(root, name)], env=environment, stdin=subprocess.DEVNULL,
                                           stdout=output, stderr=subprocess.STDOUT, timeout=CELL_TIMEOUT_SECONDS)
                code = completed.returncode
            except subprocess.TimeoutExpired:
                code = None
        with open(os.path.join(root, f"{name}.exit"), "w", encoding="utf-8") as handle:
            handle.write("timeout" if code is None else str(code))
    try:
        with concurrent.futures.ThreadPoolExecutor(max_workers=workers) as pool:
            list(pool.map(one, selected))
        exits = {}
        for name in selected:
            with contextlib.suppress(OSError):
                with open(os.path.join(root, f"{name}.exit"), encoding="utf-8") as handle:
                    exits[name] = handle.read().strip()
        failures = []
        for name in selected:
            with contextlib.suppress(OSError):
                with open(os.path.join(root, f"{name}.out"), encoding="utf-8", errors="replace") as handle:
                    sys.stdout.write(handle.read())
            if exits.get(name) != "0":
                failures.append(name)
                if exits.get(name) == "timeout":
                    print(f"  {FAILURE_PREFIX}{name}：这一格 {CELL_TIMEOUT_SECONDS} 秒没跑完")   # gate-lint:detail
        if len(exits) != len(selected):
            print(f"  ✗ task_server_lib.py 自证：派出 {len(selected)} 格、收回 {len(exits)} 格，对不上，整份作废")
            print("  → 怎么办：看上面缺了哪一格的输出；整份重跑一次")
            return 1
        if failures:
            print(f"  ✗ task_server_lib.py 自证 {len(selected)} 格里 {len(failures)} 格不对（逐格在上面）")   # gate-lint:summary
            print("  → 怎么办：按不对的那一格修服务、客户端或 control；设着 TASK_SERVER_BREAK 的话点名的格本来就该红（开关表在 task_server_lib.py 文件头）")
            return 1
        print(f"  ✓ task_server_lib.py 自证 {len(selected)} 格都对（并行 {workers} 格，每格一份真的 uvicorn 挂在临时 socket 上；"
              "装与查、服务不通、退出码原样、服务重启任务照跑、只起一次、放不下的留在库里、到达次序、错误类型、起单元失败、撞上限放大与到顶、"
              "预测每条只算一次、余量一轮只读一次、一轮里起了的记账带到后面、起之前现读复核、"
              "没记录的先记小、没记录的撞了再放大、连交几条没记录的按剩余占比分到两台、重排的种类、"
              "GPU 分卡、只许同会话取消、绕过写库不起、伪造会话号不认、环境变量白名单、不许套包装、资源记录定上限、没跑完的峰值不定额度、撞过上限后跑完的取跑完那次、只撞过上限的从撞过的上限 × 1.5 起、操作日志、完成了没人取、状态目录放哪、"
              "重排留着前几次的记录、编译目录在仓外跑完删、量不到不当 0、排队报预计用时、两台都放得下挑放下之后剩余占比大的、本机放不下就去第二台、"
              "分片的各片不钉台、计划写 pin 的拒、socket 路径太长报清楚、线程固定、分片只重排那一片、"
              "task log 的默认、--all、--attempt 与退 4、取消排着的、取消已有结局的、计划时与在跑时取消分片的整件、分片的计划与合并没成、"
              "带 --selftest 这类参数的不分片、control start 回读、要卡而没点名卡、钉第二台而它用不了、"
              "只编不跑的 cargo 不要卡、第二台在同一棵共享的树里跑不送树、重启后日志没涨从修改时刻算、"
              "上限经环境交给命令、疑似卡住只通知不停、要卡的卡上有负载不报疑似、钉值取配置的总上限减余量、钉值按派到的那一台取那一台那一项、"
              "按台的键少一台与多一项不在 HOSTS 里的地址判写坏、分片拿到它所在那一台的地址、满时限的放不下时不挡放得下的、记满一整台的收到总上限减余量起得来、顶在上限上卡住按撞上限重排、工作目录不是仓根也派得到第二台、"
              "门禁目录派得到第二台、准入按上限记账、余量读得慢时 /health 照样快、python 入口按特性认卡、封了组的台腾空而整片放不下时碎片照起、"
              "封了组的台放预计用时不超过挡路时长的短任务插空、超过的与没有预计用时的照旧封着、挡路的没有预计用时按已跑算、"
              "没写 NFS 共享不起、NFS 四项判、共享根下的路径不送不改写、没有只派一台的名单、第二台用不了要告警、打回的理由按类型写、clean 没删成写原因、"
              "自证变量平常拒、错误类型表全覆盖、判放得下之后到的取消不起与起了之后到的取消等单元停、分片推进断在哪都能恢复且每步只一份、"
              "插尝试与改任务一个事务、分片取消没停干净再取消照样重停、关联列改过的不认不停、起单元回执超时照单元名读回、编译目录占盘按记录判、"
              "第二台那条命令自己的退出码与 multi-host-run.sh 的失败分开、自动挂载认得出、第二台的内存峰值记下、命令退 75 单列并查 prune-server、"
              "临时 prune-server 的包装剥开认原命令且不与不包的共键、包装里会编或要卡的提交时拒、有记录的重排照乘 1.5 不兜到 4G、有记录的照峰值 × 1.5 不兜到 4G、没写共享数据根不起、"
              "共享根外的路径（连同 /tmp 下的与 bash -c 词里的）、不在共享根下的共享根、第三列没写 prune-server 的判定库提交时当场拒、"
              "按脚本文件头的声明认要卡的参数、tests-on-a-real-gpu 认成要卡、命令退出之后单元里留下的进程停掉并写明、跑得久而失败又没登记续跑的写一句、"
              "尝试标签两台都交给命令、排头在等交替封组时 /health 照回 ok、"
              "一片没成时在排的兄弟片当场撤而在跑的跑完收齐才收尾、人手只取消一片时别的片照跑而合并点名缺片、取消整件照旧全停、"
              "取消到的时候命令已自己跑完的不报停了、没起成的尝试不占第二台一轮的名额、子任务行校验不过的不认、任务的锁用完就从锁表删、"
              "预计用时照命令键取（路径不同、折叠后同键的有预计用时）、status 写明预计用时是按命令、按项还是没有记录、分片的计划与合并照自己命令键的历史估、各片照自己片号的历史估、"
              "wait 自己报排队过久、超过预计、换卡重排（在跑的与已跑成的）、清场没收尾、分片父任务推不动且同一档只叫一次、不带 --server-binary 的临时 prune-server 包装两台都按会编的算、"
              "暂停着交的任务与分片的子任务排着不起、暂停之前在跑的照跑到结局而重排回队的不起、暂停态跨服务重启、resume 放开排着的、control 与 status 的暂停字样、暂停着不报饿死、"
              "pause 等正在起的一条记下再回、暂停时放行一个会话而别的会话照旧排着、放行会话排着的在等字样、换放行的会话、resume 清放行、放行跨服务重启且改库校验不过、"
              "check 写放行会话与别的会话在跑几个、自带临时服务的两份脚本不带二进制的 --selftest 两台都按会编的算、撞过上限的命令再交一遍第一次尝试照撞过的上限 × 1.5 不兜到 4G、"
              "慢世界里的等待一直在变就等到收尾而不变满安静时限就判红、起点照近几次最高能升能降、/health 只读派活给排头的结论（卡在 CPU 闸、凑不够同一台 3 张卡的不报）、"
              "一条整片只封一台、要卡的整片不为凑不够的卡封组、封着的那台没有预计用时的小碎片照公式的余量插空、看进展兜派活判错、"
              "等卡的只在它要的那几台上让、兄弟片按近几次取、结局与排队过久直接发给会话（认证行在前、同一档只发一次、令牌不外露）、收件 socket 不在时任务照跑并写告警、服务健康（库写不进、单元不在、NFS、第二台用不了）发给有任务在排在跑的会话且好了再坏再发、control watch 已删、交任务时会话名下有结局没人取打严厉警告、"
              "分片几片同一刻同一档的提醒合成一条发给会话、停单元里留下的进程时抛错 result.json 照写、"
              "为满时限的整片封着的台不插空、记账超过每一台可用上限的当场打回、预算贴着整台与跑得久而核没用起来与按给的上限开并行都报给会话、"
              "旧口径的峰值不进起步）")
        return 0
    finally:
        shutil.rmtree(socket_root, ignore_errors=True)
        shutil.rmtree(root, ignore_errors=True)
