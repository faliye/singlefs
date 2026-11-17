"""task-server 自证的各格（python3 task-server/task_server_lib.py --selftest 起；这一份单独不起）。做法、只在自证里认的变量与弄坏开关写在 task_server_lib.py 文件头。"""
import os
import sys

sys.dont_write_bytecode = True

import concurrent.futures  # noqa: E402
import contextlib  # noqa: E402
import dataclasses  # noqa: E402
import hashlib  # noqa: E402
import io  # noqa: E402
import itertools  # noqa: E402
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
import unittest.mock  # noqa: E402

import task_server_lib as lib  # noqa: E402

FAILURE_PREFIX = "✗ task_server_lib.py 自证："
SESSION_A = "selftest-session-a"
SESSION_B = "selftest-session-b"
SESSION_FORGED = "selftest-session-forged"
DEFAULT_INBOX_TOKEN = "selftest-default-inbox-token"   # 各格客户端缺省带的会话令牌（World.client_environment）：没登会话的格里服务找不到 socket、用不上它
CONFIRM_RETRIES = 3   # World.confirm_submitted：确认的数对不上（这一会儿同会话别的任务起了或收了）照服务打回的数再确认的基本回数（头一回对不上时再按同会话在排在跑的条数放宽）
CONFIRM_BUDGET_SECONDS = 60.0   # World.confirm_submitted：回数用完之后，只要服务打回的还是「数对不上」（同会话别的任务还在起、收），再追的时限；不随机器忙闲定回数
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
CLIENT_GROUP_STOP_SECONDS = 10    # World.stop_client：整组发了 SIGKILL 之后等管道关上至多这么久
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
HUNDRED_RELEASE = "release-hundred"       # hundred-five-minute-tasks-balanced 放那 100 条的放行文件（相对假仓根）
GRACE_CELL_STALL_SECONDS = 2              # stall-waits-for-grace：卡住阈值；宽限里判到静了它的两倍才放行
GRACE_CELL_GRACE_SECONDS = 20             # stall-waits-for-grace：宽限只要比「判到静了两倍阈值、放行、命令退出」这一下长
# stalled-at-cap-retried：库里记一次跑完的峰值 1536 MiB，上限照它 1536M（近五次最大，不乘 1.5）；假 scope 的 memory.peak 钉在 1536 MiB（到上限）；撞了乘 1.5 重排是 2304M
AT_CAP_SEED_PEAK_BYTES = 1536 * MEBIBYTE
AT_CAP_PEAK_BYTES = 1536 * MEBIBYTE
AT_CAP_PINNED_TEXT = "1536M"
AT_CAP_GROWN_PINNED_TEXT = "2304M"
# busy-cpu-pinned-at-cap-retried、busy-cpu-below-cap-keeps-running 同一套数：库里 1536 MiB、上限 1536M；前一格假 scope 的 memory.current 钉在 1536 MiB，后一格只到它的八分之一
# aged-head-at-slice-total-starts：本机 slice 总上限 50G、slice 自己挂着回收不掉的那一截就是假余量 2G（SELFTEST_MARGIN_GIBIBYTES；任务 1713 那一回量到 0.7G，向上取整是 1G）；
# 峰值正好是总上限减余量 48G（再大一点哪台都装不下，照预算打回）：上限与记账 48G
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
# 开了 GPU 的多机配置里点名的那一张卡（gpu-service-* 那几格的配置判法要它；任务服务自己不读卡）：台标签写 HOSTS 里那一台的地址
LOCAL_CARD_ZERO = f"{SELFTEST_LOCAL_ADDRESS}:0:8G"
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
# margin-measured-per-host：两台 slice 总上限都是 50G；记录峰值 48G 的那一条上限与记账都是 49152M（近五次最大，不乘 1.5）：
# 余量 1G 的台（可用上限 49G）装得下，余量 3G 的台（47G）峰值本身就装不下
PER_HOST_SMALL_MARGIN_GIBIBYTES = 1
PER_HOST_LARGE_MARGIN_GIBIBYTES = 3
PER_HOST_PEAK_BYTES = 48 * GIBIBYTE
PER_HOST_CAP_TEXT = "49152M"


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
# 库里记一次跑完的峰值 2048 MiB：定的上限照它 2048M（近五次最大，不乘 1.5）；撞了它乘 1.5 是 3072M（钉的是验收原话里的两个数）
CAP_HIT_SEED_PEAK_BYTES = 2048 * MEBIBYTE
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
FAKE_MEMORY_WRAPPER = r'''#!/usr/bin/env bash
# task-server 自证的假 run-with-memory-cap.sh：记一行调用（连同线程包装给的线程数与 CARGO_TARGET_DIR），照命令里 fake-wrapper-exits=<码,码,…|run> 按第几次尝试演退出码；
# run 就起命令，不起真的 cargo、rustc（只放行这一格的假 cargo）；
# fake-wrapper-exit-peak=<字节> 演真包装在命令退出时读的峰值：先往服务给的 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 写这个数；
# fake-wrapper-exit-peaks=<字节,字节,…> 同上、按第几次尝试取（照 fake-wrapper-exits= 的取法，次数多过列的取最后一个）：演各次撞上限的峰值一次比一次大；
# fake-wrapper-scene=<码> 演真包装退非 0 时把临时目录搬成现场：建服务给的 RUN_WITH_MEMORY_CAP_SCENE_DIRECTORY、写一份 left-by-command、往标准错误打真包装那一句，退这个码；
# fake-wrapper-swap-resources=<文件> 第 1 次尝试时拿那一份换掉这一格的两台余量（演「头一次跑完之后本机腾不出地方了」，下一次换台）
cap="$1"; shift
threads="" command_word="${1:-}"
if [[ "$(basename -- "${1:-}")" == bash && "$(basename -- "${2:-}")" == capped.sh ]]; then threads="${3:-}"; command_word="${4:-}"; fi
printf 'wrapper task=%s attempt=%s cap=%s admitted=%s threads=%s target=%s\n' "${TASK_SERVER_TASK:-}" "${TASK_SERVER_ATTEMPT:-}" "$cap" \
  "${RUN_WITH_MEMORY_CAP_ADMITTED_BY_TASK_SERVER:-}" "$threads" "${CARGO_TARGET_DIR:-}" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
plan="" scene_exit=""
for word in "$@"; do
  case "$word" in
    fake-wrapper-exits=*) plan="${word#fake-wrapper-exits=}" ;;
    fake-wrapper-exit-peak=*) [[ -n "${RUN_WITH_MEMORY_CAP_PEAK_OUTPUT:-}" ]] && printf '%s\n' "${word#fake-wrapper-exit-peak=}" > "$RUN_WITH_MEMORY_CAP_PEAK_OUTPUT" ;;
    fake-wrapper-exit-peaks=*)
      if [[ -n "${RUN_WITH_MEMORY_CAP_PEAK_OUTPUT:-}" ]]; then
        IFS=, read -r -a exit_peaks <<<"${word#fake-wrapper-exit-peaks=}"
        peak_position=$(( ${TASK_SERVER_ATTEMPT:-1} - 1 ))
        (( peak_position < ${#exit_peaks[@]} )) || peak_position=$(( ${#exit_peaks[@]} - 1 ))
        printf '%s\n' "${exit_peaks[peak_position]}" > "$RUN_WITH_MEMORY_CAP_PEAK_OUTPUT"
      fi ;;
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
# --recover-cleanup <回执> 只在这一格世界目录里有 recover-cleanup-hold 时认（peer-cleanup-off-scheduler 演第二台收尾卡住）：记一行 peer-recover、建 recover-cleanup-reached，
# 等 recover-cleanup-release 在了（至多 200 秒）再把回执原子写成 confirmed、退 0；没有 hold 的照别的参数一样认不出、退 2（回执照旧 pending）。
if [[ "${1:-}" == --recover-cleanup && -e "$TASK_SERVER_SELFTEST_WORLD/recover-cleanup-hold" ]]; then
  printf 'peer-recover receipt=%s\n' "${2:-}" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
  : > "$TASK_SERVER_SELFTEST_WORLD/recover-cleanup-reached"
  for round in $(seq 1 2000); do [[ -e "$TASK_SERVER_SELFTEST_WORLD/recover-cleanup-release" ]] && break; sleep 0.1; done
  printf '{"status": "confirmed"}\n' > "${2:?}.partial" && mv -- "$2.partial" "$2"
  exit 0
fi
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
"block": 放行文件的绝对路径, "label", "exit_peak": 字节}…], "plan_text": 计划的原文（不给照 shards 写）, "plan_exit": 码, "merge_exit": 码,
"plan_binaries": [文件名…], "plan_binaries_link": 计划目录外的目录}。
给了 plan_binaries 的，计划那一步在计划目录下建 test-binaries/、里面照名字各写一份（演预先编好的测试二进制）；给了 plan_binaries_link 的，那个目录里写一份 kept，
test-binaries 建成指到它的符号链接；这两种都往调用记录写一行 sharded-plan-binaries（task、计划目录 directory）。
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
        directory = os.environ["TASK_SERVER_SHARD_PLAN_DIRECTORY"]
        with open(os.path.join(directory, "plan.json"), "w", encoding="utf-8") as handle:
            handle.write(text)
        binaries = os.path.join(directory, "test-binaries")
        if spec.get("plan_binaries_link"):
            os.makedirs(spec["plan_binaries_link"], exist_ok=True)
            with open(os.path.join(spec["plan_binaries_link"], "kept"), "w", encoding="utf-8") as handle:
                handle.write("kept\n")
            os.symlink(spec["plan_binaries_link"], binaries)
            record(f"sharded-plan-binaries task={task} directory={directory}")
        elif spec.get("plan_binaries"):
            os.makedirs(binaries, exist_ok=True)
            for name in spec["plan_binaries"]:
                with open(os.path.join(binaries, name), "w", encoding="utf-8") as handle:
                    handle.write(name + "\n")
            record(f"sharded-plan-binaries task={task} directory={directory}")
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
FAKE_SSH = r'''#!/usr/bin/env bash
# task-server 自证的假 ssh（TASK_SERVER_PEER_SSH；task_server_lib.py 的 peer_shell）：跳过 -o 选项与主机名，往 ssh-calls 记一行；这一格世界目录里的几个文件演第二台：
# fake-ssh-unreachable 在时一律退 255（连不上）；stat -f（挂载那一项）照 fake-ssh-mount-exit 的内容退（不在退 0），退 0 时打 fake-ssh-mount-type 的内容
# （不在打 nfs：挂着；写 autofs 演自动挂载的占位没挂上）；mountpoint -q（弄坏开关 nfs-mount-placeholder-accepted 那一支）照 fake-ssh-mount-exit 退；
# fake-ssh-peer-write-lost 在时第二台写探针那一句不写
#（演第二台写的本机读不回）；fake-ssh-hang 在时复核挂载的那一句（stat -f）把自己的进程号写进 fake-ssh-hang-pid、睡着不退（演停服务时还在跑的探测子进程）；
# fake-ssh-read-delay 在时头一个读探针的（for try in …）把它改名成 fake-ssh-read-delay.taken、先睡里面写的秒数再读（演两趟探针交错）；
# fake-ssh-hang-once 在时头一回复核挂载的那一句（stat -f）把它改名成 fake-ssh-hang-once.taken、睡着不退（演第二台一次慢，ssh 到时限被停），之后照常回；
# fake-ssh-hang-count 里写着一个数 N（大于 0）时，复核挂载的那一句（stat -f）每来一回把它减一、睡着不退（演第二台连着慢 N 回），减到 0 之后照常回；
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
    left="$(cat "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-hang-count" 2>/dev/null || echo 0)"
    if [[ "$left" -gt 0 ]]; then echo "$((left - 1))" > "$TASK_SERVER_SELFTEST_WORLD/fake-ssh-hang-count"; exec sleep 120; fi
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
FAKE_GPU_SERVICE_CONTROL = r'''#!/usr/bin/env bash
# task-server 自证的假 gpu-verification-server/control（TASK_SERVER_GPU_SERVICE_CONTROL）：记一行「gpu-service-control <子命令>」，
# 照这一格世界目录里 gpu-service-exit 的内容退（不在退 0：服务在跑）
echo "gpu-service-control $*" >> "$TASK_SERVER_SELFTEST_WORLD/calls"
code="$(cat "$TASK_SERVER_SELFTEST_WORLD/gpu-service-exit" 2>/dev/null || echo 0)"
[[ "$code" == 0 ]] || echo "  ✗ GPU 批处理验证服务没起（自证）" >&2
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
    "squeezed-out": "error-types", "timed-out": "error-types",
    "wrapper-refused": "error-types", "run-condition-unmet": "error-types", "not-run": "error-types", "peer-run-failed": "error-types",
    "launcher-lost": "error-types", "cancelled": "cancel-own-session-only", "record-tampered": "tampered-row-not-started",
    "never-fits": "pinned-peer-unavailable", "unit-start-failed": "unit-start-failed", "shard-plan-refused": "shard-retry-isolated",
    "stalled": "stalled-attempt-stopped", "resubmit-required": "queued-revalidated-on-restart",
    "prune-server-unavailable": "prune-server-unavailable-outcome", "over-four-hours": "run-hard-limit-stopped",
    "killed-externally": "external-kill-recorded-and-retried",
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


# 各格假余量报告里缺省的 CPU（memory_report）：负载 0、核数够大，服务在一台上在跑的线程之和（每条 31）几百条都不到核数 × 2，CPU 闸只红点名的几格
SELFTEST_IDLE_CPU = {"load1": 0.0, "procs_running": 1, "cpus": 4096}


def memory_report(free_gibibytes, slice_gibibytes, slice_used_gibibytes=0, margin_gibibytes=SELFTEST_MARGIN_GIBIBYTES, scopes=None):
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
    report["memory"][PEAKS.CPU_REPORT_KEY] = dict(SELFTEST_IDLE_CPU)   # 读得到的 CPU（读不到的那一台这一轮不派，task_server.py 的 cpu_problem）；local_cpu 给了的格换掉它
    if scopes is not None:
        report["memory"][PEAKS.SCOPES_REPORT_KEY] = {f"singlefs-memory-cap-selftest-{position}.scope": {PEAKS.SCOPE_USED_KEY: int(used), PEAKS.SCOPE_LABEL_KEY: label}
                                                     for position, (label, used) in enumerate(sorted(scopes.items()))}
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
                 "fake-systemd-run-refusing": FAKE_SYSTEMD_RUN_REFUSING, "fake-systemd-run-selective": FAKE_SYSTEMD_RUN_SELECTIVE,
                 "fake-systemctl": FAKE_SYSTEMCTL,
                 "fake-journalctl-earlyoom": FAKE_JOURNALCTL_EARLYOOM, "fake-ssh": FAKE_SSH, "fake-systemd-run-delayed": FAKE_SYSTEMD_RUN_DELAYED,
                 "fake-systemd-run-silent": FAKE_SYSTEMD_RUN_SILENT, "fake-prune-check": FAKE_PRUNE_CHECK, "fake-gpu-service-control": FAKE_GPU_SERVICE_CONTROL}
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
            "TASK_SERVER_PEER_RUNNER": os.path.join(self.bin, "fake-peer-runner"),
            "TASK_SERVER_SESSION_BOUNDARY_PID": str(os.getpid()), "TASK_SERVER_FAKE_REPOSITORY_ROOT": self.repo, "TASK_SERVER_FAKE_SCOPE_DIRECTORY": self.empty_scope,
            lib.WORK_ROOT_VARIABLE: self.work_root,   # 共享数据根不设 SINGLEFS_DATA_ROOT：服务照多机配置的 SHARED_DATA_ROOT 取（同真服务的单元环境）
            lib.CPU_OVERSUBSCRIPTION_VARIABLE: "2",   # CPU 闸的倍数，小世界里是 2（线 64）：各格照它搭的；真值由 cpu-gate-line-twice-cores 那一格钉
            lib.YOUNG_BOOKING_VARIABLE: "0",   # 起跑不满两分钟照整份上限记，小世界里不看：各格的任务几秒就跑完；那一条由 running-young-booked-at-cap 套在假调度器上判
            lib.SLICE_ROOM_FLOOR_VARIABLE: "0",   # slice 兜底的下限，小世界里是 0（不判）：各格的小 slice 不被 4G 的下限停派，slice-floor-holds-dispatch 那一格自己判
            lib.UNNAMED_RUN_SELFTEST_VARIABLE: "off",   # 提交时拒 ⑦（启动测试与验证没点名）小世界里不判：各格交的命令演别的规矩；unnamed-* 那几格起服务时写空串照真的判
            lib.MEMORY_FLOOR_VARIABLE: "0",   # 内存上限的下限，小世界里是 0：各格的小上限不被 1024M 的下限抬走，弄坏开关 cap-floor-skipped 与 recorded-cap-floored 只红点名的几格（task_server_lib.py 文件头「只在自证里认的变量」）；那几格写空串照真的
            "RUN_WITH_MEMORY_CAP_PEAKS": self.peak_table, "HOST_DISPATCH_STATE_DIR": os.path.join(root, "host-dispatch"),
            lib.SESSIONS_DIRECTORY_OVERRIDE: self.sessions})
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

    def resources(self, local=None, peer=None, peer_cap_gibibytes=None, hold=False, path=None, recheck_local=None, recheck_peer=None, local_slice_used=0,
                  peer_down="", peer_disk_bytes=None, peer_df=None, local_disk_bytes=None,
                  local_margin=SELFTEST_MARGIN_GIBIBYTES, peer_margin=SELFTEST_MARGIN_GIBIBYTES, peer_slice_used=0, peer_scopes=None, local_scopes=None, local_cpu=None, local_slice_unstarted=False):
        """写两台的假余量（服务每次现读）；path 给了就写到那里（假第二台跑完时换上的那一份），不给是这一格服务读的那一份。
        recheck_local 给了，起之前复核读本机时照它（排队那一轮照 local）；recheck_peer 给了，起之前复核读第二台时照它（不带盘余量，同真的第二台探测：服务照 peer_df 现量）；
        local_slice_used 是本机 slice 里回收不掉的用量（GiB）。
        peer_down 给了（一句原因），演第二台此刻用不了（照真的那样告警）。
        peer_disk_bytes、local_disk_bytes 给了，那一台的报告里带盘余量（task_server.py 的 DISK_REPORT_KEY）；peer_df 给了，演服务在第二台另起的 df 打出的那几行；都不给的服务当盘够。
        local_margin、peer_margin：两台此刻的余量（GiB，缺省 SELFTEST_MARGIN_GIBIBYTES；None 演那台量不到余量）。
        peer_slice_used：第二台 slice 里此刻进程用的内存（GiB）；peer_scopes、local_scopes：那一台报告里在跑的 scope（{尝试的 label: 回收不掉的用量字节}，memory_report 的 scopes）。"""
        value = {"hold": hold, "peer_build_root": self.peer_local}   # 第二台的编译根：假 multi-host-run.sh 在它下面 builds/<标签>/ 建这一趟的编译目录，服务照它记（Resources.build_root_of_peer）
        if peer_df is not None:
            value["df"] = {SERVER.PEER: peer_df}
        if peer_down:
            value["down"] = {SERVER.PEER: peer_down}
        if local:
            value["local"] = memory_report(local[0], local[1], local_slice_used, local_margin, local_scopes)
            if local_slice_unstarted:   # 演本机 slice 还没起：slice_total 是估的（memory_peaks_lib 的 SLICE_STARTED_KEY 为假）
                value["local"]["memory"][PEAKS.SLICE_STARTED_KEY] = False
            if local_cpu is not None:   # 本机报告里的 CPU 负载（memory_peaks_lib 的 CPU_REPORT_KEY；不给的照 SELFTEST_IDLE_CPU，memory_report）
                value["local"]["memory"][PEAKS.CPU_REPORT_KEY] = dict(local_cpu)
            if local_disk_bytes is not None:   # 本机编译目录所在盘的余量（task_server.py 的 DISK_REPORT_KEY）；不给的服务不判本机占盘
                value["local"][SERVER.DISK_REPORT_KEY] = int(local_disk_bytes)
        if recheck_local:
            value["recheck"] = {"local": memory_report(recheck_local[0], recheck_local[1], margin_gibibytes=local_margin)}
        if recheck_peer:
            value.setdefault("recheck", {})["peer"] = memory_report(recheck_peer[0], recheck_peer[1], margin_gibibytes=peer_margin)
        if peer:
            value["peer"] = memory_report(peer[0], peer[1], peer_slice_used, peer_margin, peer_scopes)
            if peer_disk_bytes is not None:
                value["peer"][SERVER.DISK_REPORT_KEY] = int(peer_disk_bytes)
            value["memory_caps"] = {SERVER.PEER: int((peer_cap_gibibytes or peer[1]) * GIBIBYTE)}
        target = path or self.resources_path
        temporary = target + ".partial"
        with open(temporary, "w", encoding="utf-8") as handle:
            json.dump(value, handle)
        os.replace(temporary, target)

    def seed_peak(self, key, byte_count, shard_count=None):
        """往这一格服务自己的库里记一次跑完（succeeded）的尝试、量到 byte_count 的峰值：服务定额度只认自己库里的尝试（task_server_lib.py 文件头「资源记录」）。
        key 是命令键（RUN_WITH_MEMORY_CAP_KEY 的值，或 PEAKS.command_key 换算过的）。分片的片（键是 lib.shard_child_key 写的那种）要给 shard_count：
        定额度只认片数与这一趟相同的记录（task_server.py 的 record_fits_task），记的是那一趟共几片。"""
        fields = {"command_key": key, "outcome": "succeeded", "memory_peak": int(byte_count)}
        if shard_count is not None:
            fields["shard_count"] = shard_count
        self.seed_attempt(fields)

    def seed_duration(self, key, seconds, command):
        """往这一格服务自己的库里记一次跑完的尝试、用时 seconds 秒（峰值量不到，不占定额度的名额）：上次运行时长只认自己库里的尝试。"""
        self.seed_attempt({"command_key": key, "command": list(command), "outcome": "succeeded", "memory_peak": None, "seconds": float(seconds)})

    def seed_attempt(self, fields, metric=lib.PROCESS_MEMORY_PEAK_METRIC):
        """记一次结束了的尝试；metric 是峰值口径一栏（task_server_lib.py 文件头「资源记录」）：缺省照这一版服务记的写，给 None 演批 92 之前记的、没有这一栏的旧行。
        fields 里给了 task_id、number 的照给的记（control key-stats 那几格按任务折重试），没给的记任务 0、第 1 次。"""
        record = {"peak_metric": metric} if metric is not None else {}
        record.update(fields)
        ended_at = float(fields.get("ended_at") or SEEDED_ENDED_AT)   # 要排先后的格自己给 ended_at
        self.seeded.append(dict(record, task_id=fields.get("task_id", 0), number=fields.get("number", 1), host="local", state="ended", session="", ended_at=ended_at,
                                started_at=ended_at - float(fields.get("seconds") or 1)))
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
        """经一个「会话 shell」起客户端（或 control，control 是唯一传 script 的）：那个 bash 的环境里是 session，客户端自己的环境里换成 forged（给了的话）。
        每一条自成一个进程组（start_new_session）：bash、它起的客户端与客户端的子进程同组，超时与收尾经 stop_client 对整组发信号，只停 bash 的话客户端拿着管道写端、communicate 等不到 EOF。"""
        environment = dict(self.client_environment, **(extra or {}))
        environment.pop(lib.SESSION_VARIABLE, None)
        if session:
            environment[lib.SESSION_VARIABLE] = session
        if forged is None:
            inner, shell_name = '"$@"; exit $?', "session-shell"
        else:
            inner, shell_name = 'CLAUDE_CODE_SESSION_ID="$0" "$@"; exit $?', forged
        if script is None:
            target = lib.CLIENT_SCRIPT
        else:
            target = lib.CONTROL_SCRIPT
        argv = ["bash", "-c", inner, shell_name, sys.executable, "-B", target, *arguments]
        if background:
            process = subprocess.Popen(argv, env=environment, cwd=cwd or self.work, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                       text=True, start_new_session=True)
            self.background.append(process)
            return process
        process = subprocess.Popen(argv, env=environment, cwd=cwd or self.work, stdin=subprocess.DEVNULL, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True,
                                   start_new_session=True)
        try:
            stdout, stderr = self.communicate(process, timeout)   # timeout 是安静时限（上面常量处「等待的判法」）
        except subprocess.TimeoutExpired:
            stdout, stderr = self.stop_client(process)
            raise SelftestFailure(f"{' '.join(arguments)[:200]} 跑了 {timeout} 秒没退（安静时限；世界一直在变的话，绝对上限 {timeout * WAIT_CEILING_FACTOR} 秒）", [str(stdout or ""), str(stderr or "")]) from None
        return subprocess.CompletedProcess(argv, process.returncode, stdout, stderr)

    def stop_client(self, process):
        """停 client 起的那一条（对它自己的进程组发 SIGKILL，进程组是那一次 Popen 建的）、收它的输出 → (stdout, stderr)。组外还有进程拿着管道写端的，至多等 CLIENT_GROUP_STOP_SECONDS 秒，交已经读到的。"""
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)
        try:
            return process.communicate(timeout=CLIENT_GROUP_STOP_SECONDS)
        except subprocess.TimeoutExpired as error:
            process.poll()
            return tuple(piece.decode("utf-8", "replace") if isinstance(piece, bytes) else (piece or "") for piece in (error.stdout, error.stderr))

    def control(self, arguments, **options):
        return self.client(arguments, script=lib.CONTROL_SCRIPT, **options)

    def submit(self, command, session=SESSION_A, extra=None, forged=None, cwd=None):
        """交一条、照确认单一起确认（confirm_submitted；task_server.py 文件头 ⑦），交任务号。"""
        full_gate = (extra or {}).get("SINGLEFS_RUN_SCOPE") == SCOPE_COMMIT and lib.commit_gate_entry(list(command), cwd or self.work)   # 演提交时那一趟：带全量标识
        completed = self.client(["submit", "--cwd", cwd or self.work, *([] if session else ["--manual"]), *(["--full-gate"] if full_gate else []), "--", *command],
                                session=session, extra=extra, forged=forged)
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if completed.returncode != 0 or not found:
            raise SelftestFailure(f"提交 {command} 没拿到任务号（退 {completed.returncode}）", [completed.stdout, completed.stderr] + self.server_log_lines())
        self.confirm_submitted(completed, session=session, extra=extra, forged=forged)
        return int(found.group(1))

    def confirm_submitted(self, completed, session=SESSION_A, extra=None, forged=None, required=True):
        """照提交回复（或确认被拒时）打的 CONFIRM_NEEDED_MARK 那一行确认（自证世界的帮手：自证造的任务照样要确认，不另设跳过确认的开关）：
        数对不上的（这一会儿同会话别的任务起了或收了）照服务打回的那一行再确认；回复里没有那一行的（弄坏开关 confirm-skipped 下交了就进队）不确认。
        服务核数是在会话锁里算了就比（task_server.py 的 confirm、counts_mismatch），每一回对不上都是同会话又有一条在排在跑的收了（本帮手不在确认期间再交）：
        头一回对不上时服务回的条数就是还能收掉的最多几条，所以重试的回数在 CONFIRM_RETRIES 之外再放宽这么多回（有界；机器忙、几十条一齐收尾时每一回客户端往返比收一条还慢，三回追不上）；
        回数用完了服务打回的还是「数对不上」（打回的回话里还带着新的确认单）的，再追到 CONFIRM_BUDGET_SECONDS 秒为止：对不上的次数跟着同会话别的任务起、收的次数走，机器忙时次数多，不能拿固定回数判。
        弄坏开关 confirm-retries-fixed 下只重试 CONFIRM_RETRIES 回。
        → 最后一次 confirm 的 CompletedProcess（不用确认交 None）；required 为真时确认不成判红。"""
        reply = completed
        allowed, widened = CONFIRM_RETRIES, False
        tries, deadline = 0, time.monotonic() + getattr(self, "confirm_budget_seconds", CONFIRM_BUDGET_SECONDS)
        while tries < allowed or (not lib.broken("confirm-retries-fixed") and reply is not completed and reply.returncode != 0 and time.monotonic() < deadline):
            tries += 1
            needed = lib.confirm_needed_fields(reply.stdout)
            if needed is None:
                return None if reply is completed else reply
            task_id, items, tasks = needed
            if not widened and not lib.broken("confirm-retries-fixed"):
                allowed, widened = allowed + tasks, True
            reply = self.client(["confirm", str(task_id), "--items", str(items), "--tasks", str(tasks)], session=session, extra=extra, forged=forged)
            if reply.returncode == 0:
                return reply
        if required:
            raise SelftestFailure(f"照确认单确认了 {tries} 回都没成（最后退 {reply.returncode}）", [completed.stdout, reply.stdout, reply.stderr] + self.server_log_lines())
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
                self.stop_client(process)
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


def cell_aged_blocked_task_blocks_later(world):
    """排满时限还起不来的任务挡住排在它后面的全部任务（task_server.py 排队那一轮里 blocked 那一支）：本机 30G，头一条（记 12G）占着，排头（记 28G）放不下；
    等它排满时限（自证里 2 秒）之后再交两条放得下的小任务：两条都不起，排队原因写「排在任务 N 后面：它已排满时限」；放开头一条之后排头先起，后面两条再起，都跑完。
    弄坏开关 aged-blocked-lets-others-start 下后面两条照常先起，这一格红。"""
    world.resources(local=(30, 30))
    for key, peak in (("selftest-aged-block-first", 12 * GIBIBYTE), ("selftest-aged-block-head", 28 * GIBIBYTE)):
        world.seed_peak(key, peak)
    world.start_server(extra={"TASK_SERVER_SELFTEST_AGED_SECONDS": "2"})
    release = os.path.join(world.root, "release")
    first = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-aged-block-first"})
    later, waiting = [], {}
    try:
        world.running(first)
        head = world.submit(["bash", "-c", "exit 0"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-aged-block-head"})
        time.sleep(3)   # 排头满了时限（下限：机器忙时只会更久）
        world.next_pass()
        later = [world.submit(["bash", "-c", "exit 0"]) for _ in range(2)]
        world.next_pass()
        world.next_pass()
        waiting = {task_id: world.summary(task_id) for task_id in later}
    finally:
        open(release, "w").close()
    summaries = {task_id: world.finished(task_id) for task_id in [first, head, *later]}
    problems = []
    for task_id in later:
        found = waiting.get(task_id) or {"task": {}, "attempts": []}
        if found["attempts"] or f"排在任务 {head} 后面" not in (found.get("waiting_for") or "") or "已排满时限" not in (found.get("waiting_for") or ""):
            problems.append(f"任务 {task_id}（排在排满时限的任务 {head} 后面）：应当没起、排队原因写「排在任务 {head} 后面：它已排满时限」，实际 {describe(found)}、原因「{found.get("waiting_for")}」")
    orders = {task_id: attempt_order(summaries[task_id]) for task_id in [head, *later]}
    if orders[head] is None or any(orders[task_id] is None or orders[task_id] < orders[head] for task_id in later):
        problems.append(f"放开之后排头应当先于后面两条起，实际尝试号 {orders}")
    if {summary["task"].get("outcome") for summary in summaries.values()} != {"succeeded"}:
        problems.append(f"放开之后都应当跑完，实际 {[describe(summary) for summary in summaries.values()]}")
    raise_cell_problems(world, problems, "排满时限还起不来的任务挡住后面的")
    return f"排头（任务 {head}）排满时限还放不下：后面两条放得下的都不起、原因点名它；放开之后排头先起（尝试 {orders[head]}），后面两条再起，都跑完"


def cell_jump_only_shorter_than_running(world):
    """插队规则（task_server.py 的 jump_refusal；用户定「插队的任务的时长不得超过正在运行的任务剩余的时长。没有上次运行时间的任务不得插队」）：本机 30G，头一条（记 12G、
    上次跑了 600 秒）占着，排头（记 28G）放不下、还没排满时限。后面交三条放得下的小任务：上次跑了 1 秒的先起、跑完（头一条还占着）；没有上次运行时长的与上次跑了
    100000 秒的都不起，排队原因写「不插队」。放开头一条之后都跑完。弄坏开关 jump-length-unchecked 下后两条也先起，这一格红。"""
    world.resources(local=(30, 30))
    for key, peak in (("selftest-jump-first", 12 * GIBIBYTE), ("selftest-jump-head", 28 * GIBIBYTE)):
        world.seed_peak(key, peak)
    release = os.path.join(world.root, "release")
    blocker = ["bash", "-c", BLOCK_SCRIPT, "block", release]
    short, lengthy, unrecorded = (["bash", "-c", "exit 0", name] for name in ("jump-short", "jump-lengthy", "jump-unrecorded"))
    world.seed_duration("selftest-jump-first", 600, blocker)
    world.seed_duration("selftest-jump-short", 1, short)
    world.seed_duration("selftest-jump-lengthy", 100000, lengthy)
    world.start_server()
    first = world.submit(blocker, extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-jump-first"})
    waiting, jumped = {}, None
    try:
        world.running(first)
        head = world.submit(["bash", "-c", "exit 0"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-jump-head"})
        world.next_pass()
        refused = [world.submit(lengthy, extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-jump-lengthy"}), world.submit(unrecorded)]
        allowed = world.submit(short, extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-jump-short"})
        jumped = world.finished(allowed)   # 头一条还占着、排头还没起，这一条先跑完
        world.next_pass()
        waiting = {task_id: world.summary(task_id) for task_id in [head, *refused]}
    finally:
        open(release, "w").close()
    summaries = {task_id: world.finished(task_id) for task_id in [first, head, *refused]}
    problems = []
    if jumped is None or jumped["task"].get("outcome") != "succeeded" or (waiting.get(head) or {"attempts": [1]})["attempts"]:
        problems.append(f"上次跑了 1 秒的那一条应当在排头（任务 {head}）起之前跑完，实际 {describe(jumped) if jumped else '没跑完'}、排头 {describe(waiting.get(head) or {'task': {}, 'attempts': []})}")
    for task_id in refused:
        found = waiting.get(task_id) or {"task": {}, "attempts": []}
        if found["attempts"] or "不插队" not in (found.get("waiting_for") or ""):
            problems.append(f"任务 {task_id}（没有上次运行时长，或上次跑得比在跑的剩余长）应当没起、排队原因写「不插队」，实际 {describe(found)}、原因「{found.get('waiting_for')}」")
    if {summary["task"].get("outcome") for summary in summaries.values()} != {"succeeded"}:
        problems.append(f"放开之后都应当跑完，实际 {[describe(summary) for summary in summaries.values()]}")
    raise_cell_problems(world, problems, "插队只许比在跑的剩余短的")
    return "排头没起时：上次跑了 1 秒的先起、跑完；没有上次运行时长的与上次跑了 100000 秒的不插队；放开之后都跑完"


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
    expected = ["what=pending host=local", "what=snapshot"]   # 只派得了本机：一轮读一次本机的余量、一次本机的待涨量
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
    """表里的错误类型逐个喂一个假命令或假包装的退出码（EXERCISED_BY 点名这一格的那几种），记下的类型与表对得上；表外的 199 记 command-failed，
    退 245（表里没有这个码）同样记 command-failed。"""
    world.resources(local=(100, 100))   # 头一段没有第二台：本机的那几种都在本机起（派得到第二台的照余量挑台，不再因工作目录只在本机）
    world.seed_peak("selftest-peer-only", 120 * GIBIBYTE)
    world.start_server()
    plain = ["bash", "-c", "exit 0"]
    cases = [("succeeded", plain, {}, None, None), ("command-failed", ["bash", "-c", "exit 3"], {}, None, 3),
             ("command-failed", ["bash", "-c", "exit 199"], {}, None, 199), ("command-failed", ["bash", "-c", "exit 245"], {}, None, 245),
             ("memory-cap-hit", plain + ["fake-wrapper-exits=250,run"], {}, None, None), ("squeezed-out", plain + ["fake-wrapper-exits=254,run"], {}, None, None),
             ("timed-out", plain + ["fake-wrapper-exits=253"], {}, None, None), ("wrapper-refused", plain + ["fake-wrapper-exits=251"], {}, None, None),
             ("run-condition-unmet", ["bash", "-c", "exit 78"], {}, None, None), ("not-run", ["bash", "-c", "exit 77"], {}, None, None),
             ("launcher-lost", ["bash", "-c", "kill -9 $PPID"], {}, None, None)]
    submitted = [(expected, world.submit(command, extra=extra, cwd=cwd), exit_code) for expected, command, extra, cwd, exit_code in cases]
    summaries = {task_id: world.finished(task_id) for _, task_id, _ in submitted}
    # 后一段有第二台：记录 120G 比本机可用上限 98G 大、第二台装得下的那一条不派本机（hosts_short_of_record）、派第二台，假第二台退 255 记 peer-run-failed
    world.resources(local=(100, 100), peer=(200, 200), peer_cap_gibibytes=190)
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
        if seen != expected or (exit_code is not None and (attempts[0].get("exit_code") != exit_code or summary["task"].get("exit_code") != exit_code)):
            wrong.append(f"任务 {task_id} 应当是 {expected}{'、退 ' + str(exit_code) if exit_code is not None else ''}，实际第一次尝试 {seen}、{describe(summary)}")
        if expected == "peer-run-failed" and attempts and attempts[0].get("host") != "peer":
            wrong.append(f"任务 {task_id} 应当派到第二台，实际 {attempts[0].get('host')}")
    if wrong:
        raise SelftestFailure("错误类型对不上：" + "；".join(wrong), world.server_log_lines())
    return f"{len(cases)} 个假命令或假包装记下的类型都与表对得上（表外的 199、245 记 command-failed）"


def cell_unit_start_failed(world):
    world.start_server(extra={"TASK_SERVER_SYSTEMD_RUN": os.path.join(world.bin, "fake-systemd-run-refusing")})
    marker = os.path.join(world.root, "ran")
    task_id = world.submit(["bash", "-c", ': > "$1"', "touch-marker", marker])
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "unit-start-failed" or summary["task"].get("exit_code") != 235 or os.path.exists(marker):
        raise SelftestFailure(f"systemd-run 起不来时应当记 unit-start-failed、退 235、命令没跑，实际 {describe(summary)}、命令{'跑了' if os.path.exists(marker) else '没跑'}")
    return "systemd-run 起不来记 unit-start-failed、命令没跑"


def cell_memory_cap_growth(world):
    world.seed_peak("selftest-growth", 1536 * MEBIBYTE)   # 上限照近五次最大 1536M 起，不乘 1.5；撞了才乘
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
    world.seed_peak("selftest-at-maximum", 30 * GIBIBYTE)
    world.seed_peak("selftest-at-maximum-switch", 30 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,250"], extra={"RUN_WITH_MEMORY_CAP_KEY": "selftest-at-maximum"})
    summary = world.finished(task_id)
    caps = [attempt.get("memory_cap_text") for attempt in summary["attempts"]]
    if caps != ["30720M", "46080M"] or summary["task"].get("outcome") != "memory-cap-at-maximum" or summary["task"].get("exit_code") != 236:
        raise SelftestFailure(f"总上限 50G、没有第二台：记录峰值 30G 起 30720M（不乘 1.5），撞了乘 1.5 到 46080M，再撞（再乘 1.5 就过总上限）记 memory-cap-at-maximum 打回、退 236，"
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
    unlisted = world.submit(plain + ["fake-wrapper-exits=245,0"])
    wrong = []
    for task_id, attempts, outcome in ((squeezed, 2, "succeeded"), (unlisted, 1, "command-failed")):
        summary = world.finished(task_id)
        if not task_command_ok(summary, outcome, attempts):
            wrong.append(f"任务 {task_id} 应当尝试 {attempts} 次、结局 {outcome}，实际 {describe(summary)}")
    if wrong:
        raise SelftestFailure("254 同一上限重排、表外的 245 不重排：" + "；".join(wrong))
    return "254 重排一次跑完、表外的 245 记 command-failed 不重排"


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
              "next_cap": None, "next_host": "", "outcome_counts": {}, "note": "", "cancel_requested": False, "attempt_count": 0}
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
    peak_segment = first.get(SERVER.PEAK_SEGMENT_CORES_COLUMN)   # 跑不满一段：整次就是一段，收尾那一刻的累计 2 秒 CPU ÷ 服务量的已跑秒数（比命令自己的用时略长）
    if not isinstance(peak_segment, (int, float)) or isinstance(peak_segment, bool) or not 0 < peak_segment <= 2.0 / first["seconds"] + 0.05:
        raise SelftestFailure(f"收尾应当记峰值段核数（{SERVER.PEAK_SEGMENT_CORES_COLUMN}，跑不满一段的照整次：大于 0、不超过 2 秒 CPU ÷ 用时），实际 {peak_segment}、用时 {first.get('seconds')}")
    second_id = world.submit(command)
    second = world.finished(second_id)["attempts"][0]
    if second.get("memory_cap_text") != "300M" or not any("cap=300M" in line for line in world.calls_of(second_id)):
        raise SelftestFailure(f"同一命令下一次应当按记录的峰值 300M 定上限 300M（近五次最大，不乘 1.5），实际 {second.get('memory_cap_text')}、包装调用 {world.calls_of(second_id)}")
    return f"记下用时 {first['seconds']} 秒、峰值 300M、平均 {first['cores']} 核；下一次上限 300M"


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


def outcome_head(task_id, outcome):
    """推给会话的结局那一条首行到「（<种类>」为止的一段（task_server.py 的 outcome_headline）：类别「结局」、任务已结束、任务号、中文名。"""
    return f"{SERVER.INBOX_HEADLINE}结局（任务已结束）任务 {task_id} {SERVER.kind_label(outcome)}（{outcome}"


def unclaimed_head(task_id, outcome):
    """结局没人取而再推的那一条首行到「（<种类>」为止的一段：状态里另写这条结局还没人取。"""
    return f"{SERVER.INBOX_HEADLINE}结局（任务已结束，这条结局还没人取）任务 {task_id} {SERVER.kind_label(outcome)}（{outcome}"


def reminder_head(task_id, kind, state="任务还在跑"):
    """推给会话的要处置的提醒首行到「（<种类>」为止的一段（task_server.py 的 reminder_headline）：类别「提醒」、任务此刻的状态（任务还在排 / 任务还在跑，不是结局）、任务号、中文名。"""
    return f"{SERVER.INBOX_HEADLINE}提醒（{state}，不是结局）任务 {task_id} {SERVER.kind_label(kind)}（{kind}"


def reminders_after(messages, seconds):
    """messages 里正文写着 reminded_after_seconds=<seconds> 的那几条（时长判的提醒照末行机器读的那一行分得出这一回跨过的是哪道提醒门槛）。"""
    return [message for message in messages if f" reminded_after_seconds={seconds} " in message["text"]]


def inbox_outcome_case(world, script, outcome, exit_text):
    """一条任务有了结局（task_server.py 文件头 ⑥「结局」）：会话的收件 socket 恰好收到一条，先认证行、再 user 那一行，正文第一行「task-server：结局（任务已结束）任务 <号> <中文名>（<结局>，退 <码>；第 1 回）」，
    后面写原因、出路、日志与 wait 的末行原样；令牌不在 status、日志与服务的输出里。→ (问题表, 任务号, 正文)。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world)
    task_id = world.submit(["bash", "-c", script], extra=inbox.environment())
    world.finished(task_id)
    head = outcome_head(task_id, outcome)
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
    return f"收件 socket 收到一条「{outcome_head(task_id, 'succeeded')}…」（认证行在前、令牌对得上），发出去之后记成已取，令牌不进 status 与日志"


def cell_inbox_outcome_failed(world):
    """会失败的一条（命令退 3）：收件 socket 收到一条结局 command-failed（inbox_outcome_case 的判据），末尾 wait 的末行 exit=3。
    弄坏开关 inbox-not-sent、inbox-auth-omitted 下这一格红。"""
    problems, task_id, _ = inbox_outcome_case(world, "echo inbox-failed; exit 3", "command-failed", "exit=3")
    if problems:
        raise SelftestFailure("结局 command-failed 发给会话：" + "；".join(problems), world.server_log_lines())
    return f"收件 socket 收到一条「{outcome_head(task_id, 'command-failed')}…」（认证行在前、令牌对得上），末尾是 exit=3 的末行"


def cell_inbox_queued_too_long_once(world):
    """排队过久（task_server.py 文件头 ⑥「要处置的提醒」，同 attention-reported-once 的世界：第二台用不了、只有它放得下）：排满 6 秒收件 socket 收到一条
    「task-server：提醒（任务还在排，不是结局）任务 <号> 排队过久（queued-too-long；第 1 回」，正文写依据（已排多久、排满 12 秒再提醒一回）、原因、出路、日志与
    TASK_SERVER_ATTENTION 一行（kind=queued-too-long 之后紧接 reminded_after_seconds=6）；再看几轮不多一条；排满 12 秒再收一条 reminded_after_seconds=12 的。
    弄坏开关 inbox-not-sent、inbox-auth-omitted、inbox-reported-every-round 下这一格红。"""
    inbox = FakeInbox(world, "a")
    world.resources(local=(50, 20), peer_down="自证：第二台在重启")
    world.seed_peak("selftest-inbox-queue", 30 * GIBIBYTE)
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": str(ATTENTION_ONCE_SECONDS)})
    task_id = world.submit(["bash", "-c", "exit 0", "inbox-queue"], extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: "selftest-inbox-queue"}), cwd=world.repo)
    head, later = reminder_head(task_id, "queued-too-long", "任务还在排"), 2 * ATTENTION_ONCE_SECONDS
    inbox.wait_for(world, head)
    time.sleep(ATTENTION_ONCE_QUIET_SECONDS)
    first = reminders_after(inbox.headed(head), ATTENTION_ONCE_SECONDS)
    problems = inbox_problems(first, head, f"排满 {ATTENTION_ONCE_SECONDS} 秒那一回")
    text = first[0]["text"] if first else ""
    for wanted in ("依据：已排", f"排满 {later} 秒再提醒一回", "原因：", "出路：",
                   f"{lib.ATTENTION_MARK} task={task_id} type={lib.ATTENTION_TYPE} kind=queued-too-long reminded_after_seconds={ATTENTION_ONCE_SECONDS} basis="):
        if wanted not in text:
            problems.append(f"排队过久那一条的正文里应当有「{wanted}」，实际「{text[:400]}」")
    if "档" in text.split("\n", 1)[0] or any("档" in line for line in text.split("\n") if line.startswith("依据：")):
        problems.append(f"首行与依据只写事实与下一回什么时候提醒，不该有「档」，实际「{text[:400]}」")
    inbox.wait_for(world, head, count=2)
    time.sleep(INBOX_QUIET_SECONDS)
    problems += inbox_problems(reminders_after(inbox.headed(head), later), head, f"排满 {later} 秒那一回")
    if len(reminders_after(inbox.headed(head), ATTENTION_ONCE_SECONDS)) != 1:
        problems.append(f"排满 {later} 秒之后，{ATTENTION_ONCE_SECONDS} 秒那一条仍该只有一条，实际 {len(reminders_after(inbox.headed(head), ATTENTION_ONCE_SECONDS))} 条")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    world.finished(task_id)
    if problems:
        raise SelftestFailure("排队过久发给会话：" + "；".join(problems), world.server_log_lines())
    return f"排满 {ATTENTION_ONCE_SECONDS} 秒、{later} 秒各收到恰好一条（认证行在前），同一道门槛不重发"


INBOX_SHARD_KEY = "selftest-inbox-shard-queue"   # inbox-shard-attention-merged：三片分片的命令键，三片各记过 30G（只有第二台放得下，第二台用不了，三片一齐排着）


def cell_inbox_shard_attention_merged(world):
    """分片的几片同时排久（task_server.py 文件头 ⑥「要处置的提醒」的合成一条，Scheduler.attention_groups）：三片各记过 30G、本机 slice 20G、第二台用不了，
    三片一齐排着；排满 6 秒收件 socket 只收到一条「task-server：提醒（任务还在排，不是结局）任务 <父任务号> 排队过久（queued-too-long，子任务 <号的区间>，共 3 片；第 1 回）」，
    第一行点到三片的号；再看几轮不多一条。弄坏开关 inbox-attention-per-shard 下逐片各发一条（收到三条），
    inbox-headline-state-omitted 下首行不写「提醒（任务还在排，不是结局）」、照首行认不到那一条（收到 0 条），这一格红。"""
    inbox = FakeInbox(world, "a")
    for index in range(3):
        world.seed_peak(lib.shard_child_key(INBOX_SHARD_KEY, "shard", index, 3), 30 * GIBIBYTE, shard_count=3)
    world.resources(local=(50, 20), peer_down="自证：第二台此刻用不了")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS": str(ATTENTION_ONCE_SECONDS)})
    task_id = world.submit(world.write_spec("spec-inbox-shards.json", {"shards": [{}, {}, {}]}), cwd=world.repo,
                           extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: INBOX_SHARD_KEY}))
    summary = world.wait_for(task_id, lambda summary: queued_shards(summary) == [0, 1, 2], "三片都排着")
    children = sorted(child["task"]["id"] for child in shard_children(summary).values())
    head = reminder_head(task_id, "queued-too-long", "任务还在排")
    inbox.wait_for(world, head)
    time.sleep(INBOX_QUIET_SECONDS)
    messages = reminders_after(inbox.headed(head), ATTENTION_ONCE_SECONDS)
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
    return f"三片同时排满 {ATTENTION_ONCE_SECONDS} 秒，会话只收到一条：「{headline}」"


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


FORMULA_HEAD_BLOCKER_PEAK_GIBIBYTES = 90   # starved-head-judged-by-the-formula：先占着的那一条的峰值（上限与记账照它 90G，在 100G、余量 2G 的台上起得来）
FORMULA_HEAD_PEAK_GIBIBYTES = 12           # 那一格的排头的峰值（上限与记账 12G：judge_host 放得下，公式只剩 100 − 2 − 90 = 8G 放不下）
FORMULA_HEAD_ROOMY_GIBIBYTES = 200         # 那一格换上的假余量（空闲与 slice 都是这么多）：公式剩 200 − 2 − 90 = 108G，排头放得下
FORMULA_HEAD_WATCH_SECONDS = 4 * SERVER.FAST_STARVED_SECONDS   # 那一格公式放不下时看 /health 看这么久（过阈值几倍）
PAUSE_SETTLE_SECONDS = 1.0                 # 那一格停排队之后等这么久再换余量（一轮 0.1 秒：开了头的那一轮早排完，换上的余量不会被它读去起排头）


def cell_starved_head_judged_by_the_formula(world):
    """/health 只读派活给排头的结论（task_server.py 的 starved_problem 读 head_verdict；judge_placement 照 judged_on_host 过公式）：本机 100G、余量 2G，一条上限 90G 的在跑、
    它的 scope 已用满 90G（不记待涨量，记账照此刻实测 90G），排头上限 12G：judge_host 照余量放得下，公式（配置的总上限 − 余量 − 在跑的记账之和）只剩 8G 放不下，派活判它不能起，
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
        waiting = world.wait_for(head, lambda found: SERVER.RUNNING_BOOKED_SUM_TEXT in (found.get("waiting_for") or "") or found["task"].get("state") != "queued",
                                 "照公式报放不下")
        blocked_health = health_until(world, lambda found: not found.get("ok"), FORMULA_HEAD_WATCH_SECONDS)
        if cap != FORMULA_HEAD_BLOCKER_PEAK_GIBIBYTES * GIBIBYTE or not label or waiting["task"].get("state") != "queued" or SERVER.RUNNING_BOOKED_SUM_TEXT not in (waiting.get("waiting_for") or ""):
            problems.append(f"先占着的那一条应当上限 90G、有 label，排头照公式排着、排队原因写「{SERVER.RUNNING_BOOKED_SUM_TEXT}」；实际上限 {cap} 字节、label {label!r}、"
                            f"排头 {describe(waiting)}、在等「{waiting.get('waiting_for')}」")
        if not blocked_health.get("ok"):
            problems.append(f"公式放不下（在跑的记账之和占满）时排头排着是正常的，/health 应当一直回 ok，实际 {blocked_health}")
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
    return (f"在跑的记账之和占满（公式剩 8G、排头要记 12G）时 /health 一直回 ok；公式放得下而没起时报「{starved_health.get('problem')}」；放开之后都跑完")


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
    if "在跑：已跑" not in shown.stdout or "没涨" not in shown.stdout:
        raise SelftestFailure(f"在跑的任务 status 应当报已跑多久、日志多久没涨、在用核数，实际「{shown.stdout.strip()[:400]}」")
    return "在跑的任务 status 报已跑多久、日志多久没涨与在用核数"


DUTY_SPREAD_SCRIPT = "for i in $(seq 8); do echo spread $i; sleep 0.25; done"   # three-duties-two-sessions 分到两台的那 6 条（记的用时与交的命令逐词相同）


def cell_three_duties_two_sessions(world):
    """服务的三件职责放进同一个模拟世界：① 两台分任务 ② 没满时限的排头放不下时后面放得下的照起，排头空出来之后照起 ③ 跑不了的打回提交它的那个会话、别的会话看不见。"""
    world.seed_peak("selftest-duty-spread", 8 * GIBIBYTE)
    world.seed_duration("selftest-duty-spread", 2, ["bash", "-c", DUTY_SPREAD_SCRIPT, "spread", "fake-peer-run"])
    world.seed_peak("selftest-duty-blocker", 16 * GIBIBYTE)
    world.seed_duration("selftest-duty-blocker", 600, ["bash", "-c", BLOCK_SCRIPT, "block", os.path.join(world.root, "release")])   # 占着的那条上次跑了 600 秒：剩余时长算得出
    world.seed_peak("selftest-duty-head", 20 * GIBIBYTE)
    world.seed_peak("selftest-duty-small", 100 * MEBIBYTE)
    world.seed_duration("selftest-duty-small", 1, ["bash", "-c", "exit 0"])   # 小的上次跑了 1 秒：比占着的那条的剩余短，排头没起时许插（jump_refusal）
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
    # ③ 会话 A 交一条跑不了的（入口会编的声明写坏了：派活时打回 run-condition-unmet）、会话 B 交一条命令自己失败的：各自的 wait 拿到各自的类型，status 只列自己的
    world.resources(local=(30, 30), peer=(10, 30), peer_cap_gibibytes=30)
    unrunnable = world.submit(write_unmet_script(world, "duty-unmet.sh"))
    failing = world.submit(["bash", "-c", "echo started; sleep 1; exit 3"], session=SESSION_B)   # 跑到一半出错
    world.finished(unrunnable), world.finished(failing)
    for session, own, other in ((SESSION_A, unrunnable, failing), (SESSION_B, failing, unrunnable)):   # wait 收下之前：各自只看得见自己的
        listed = {item.get("id") for item in json.loads(world.client(["status", "--json"], session=session).stdout or "{}").get("tasks", [])}
        if own not in listed or other in listed:
            problems.append(f"③ {session} 的 status 应当列自己的任务 {own}、不列别的会话的 {other}，实际列了 {sorted(listed)}")
    for task_id, session, outcome, code in ((unrunnable, SESSION_A, "run-condition-unmet", None), (failing, SESSION_B, "command-failed", 3)):
        waited = world.client(["wait", str(task_id)], session=session)
        if f"type={outcome}" not in waited.stdout + waited.stderr or (code is not None and waited.returncode != code) or waited.returncode == 0:
            problems.append(f"③ 任务 {task_id} 的 wait 应当退非 0{('、退 ' + str(code)) if code else ''}、末行 type={outcome}，实际退 {waited.returncode}、"
                            f"「{(waited.stdout + waited.stderr).strip()[-200:]}」")
    if problems:
        raise SelftestFailure("三件职责：" + "；".join(problems), world.server_log_lines())
    return (f"① 6 条 8G 分在 {sorted(set(hosts))}；② 排头放不下时小的先起（{small_start:.2f}）、本机放出来之后排头起（{head_start:.2f}）；"
            f"③ 跑不了的打回会话 A、命令失败的打回会话 B，各自 wait 拿到类型、status 互不可见")


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
    cpu.stat 的 usage_usec 照挂钟涨：起了 t 秒就是 t ÷ 0.2 × usec_per_tick 微秒（缺省每 0.2 秒涨 0.2 秒：一个核在算），每 0.2 秒左右写一回，先写临时文件再改名（采样读不到写了一半的）。
    照挂钟算、不照写了几回累加：机器忙时这条线程排不上、写得稀，读到的平均核数照样是设定的那个数（只差最后一回写到读之间那一截）。交 (目录, 停的 Event)：格收尾时设上它。"""
    scope = os.path.join(world.root, name)
    os.makedirs(scope)
    for file_name, value in files.items():
        with open(os.path.join(scope, file_name), "w", encoding="utf-8") as handle:
            handle.write(f"{value}\n")
    stat = os.path.join(scope, "cpu.stat")
    with open(stat, "w", encoding="utf-8") as handle:
        handle.write("usage_usec 0\n")
    stop = threading.Event()

    started = time.monotonic()

    def keep_burning():
        while not stop.is_set():
            used = int((time.monotonic() - started) / 0.2 * usec_per_tick)
            with open(stat + ".partial", "w", encoding="utf-8") as handle:
                handle.write(f"usage_usec {used}\n")
            os.replace(stat + ".partial", stat)
            time.sleep(0.2)

    threading.Thread(target=keep_burning, daemon=True).start()
    return scope, stop


def cell_busy_cpu_pinned_at_cap_retried(world):
    """日志满卡住阈值没涨、此刻内存贴着上限、CPU 一直在涨（任务 7723：在回收里打转、跑了 37775 秒没人停；task_server.py 的 judge_stall_stop）：CPU 在涨不算在算，
    停掉、记 memory-cap-hit、乘 1.5 重排、跑完，说明写「顶在上限上卡住」与此刻内存。库里记一次跑完的峰值 1536 MiB（上限照它 1536M），
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


# page-cache-not-counted-as-process-memory：假 scope 的 memory.current 与 memory.peak 钉在上限 1536M（库里记的峰值 1536M，上限照它），里面进程用的只有 anon 96M，
# 其余是页缓存与内核（任务 8853 的样子：anon 921M、file 22163M、kernel 650M，memory.peak 顶到上限 50176M）
PAGE_CACHE_ANON_BYTES = 96 * MEBIBYTE
PAGE_CACHE_KERNEL_BYTES = 32 * MEBIBYTE
PAGE_CACHE_FILE_BYTES = AT_CAP_PEAK_BYTES - PAGE_CACHE_ANON_BYTES - PAGE_CACHE_KERNEL_BYTES


def cell_page_cache_not_counted_as_process_memory(world):
    """页缓存撑满 scope 不算进程内存（attempt_runner.py 文件头 ③ ⑥、task_server.py 文件头 ①「顶在上限上卡住」）：假 scope 的 memory.stat 是 anon 96M、file 1408M、
    kernel 32M，memory.current 与 memory.peak 钉在上限 1536M，CPU 一直不涨。跑完的那一条记的峰值是进程用的 96M、不是 1536M；
    日志不涨的那一条（库里 1536M、上限 1536M）满卡住阈值照记 stalled、只一次尝试，不判顶在上限上卡住、不乘 1.5 重排。
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
    """满时限的排头要记满一整台：峰值 48G，正好是本机 slice 总上限 50G 减 slice 自己挂着回收不掉的那一截（这台此刻的余量，假余量 2G），上限与记账就是 48G，
    照样起得来、跑完（task_server.py 的 booked_at_cap 与 host_limit）；记满总上限的那种写法永远放不下（slice 放得下的减了余量只剩 48G）。看排两轮之后起没起，不比秒数。"""
    world.resources(local=(50, 50), hold=True)
    world.seed_peak("selftest-head-at-total", 48 * GIBIBYTE)
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
    world.seed_peak("selftest-squeeze-at-total", 48 * GIBIBYTE)   # 峰值正好是总上限减这台的假余量 2G：上限就是 48G
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
    world.seed_peak(key, 3 * GIBIBYTE)   # 上限照它 3072M（近五次最大，不乘 1.5）
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
    # 跑完那次的峰值大于撞过的上限的一例（取它本身），与小于撞过的上限的一例（取撞过的上限）：再交照近五次最大，不乘 1.5；
    # 假世界两次共用一个峰值，要到上限的一半才算真撞（cap_hit_measurement）
    cases = (("selftest-completed-above-cap-hit", 2500 * MEBIBYTE, "2500M"), ("selftest-completed-below-cap-hit", 1100 * MEBIBYTE, CAP_HIT_PINNED_TEXT))
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
                            f"（近五次最大：跑完那次的峰值与撞过的上限取大，不乘 1.5），实际 {cap}、包装收到 {wrapper_cap_texts(world, second_id)}")
    if problems:
        raise SelftestFailure("撞过上限后跑完的命令：" + "；".join(problems), world.server_log_lines())
    return f"撞过 {CAP_HIT_PINNED_TEXT} 后跑完：再交同一条，峰值 2500M 的取 2500M，峰值 1100M 的取撞过的 {CAP_HIT_PINNED_TEXT}"


def cell_cap_hit_starts_above(world):
    key = "selftest-cap-hit-only"
    # 本机 slice 总上限 4G、余量 0：2048M 撞了乘 1.5 到 3072M 再撞，再乘就过 4G、到顶收尾；这条命令只留两次撞上限的尝试（与库里记的那一次跑完的 2048M）
    world.resources(local=(30, 4), local_margin=0)
    world.seed_peak(key, CAP_HIT_SEED_PEAK_BYTES)
    world.start_server()
    first = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250"], extra={lib.KEY_VARIABLE: key}))
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in first["attempts"]]
    if seen != [(CAP_HIT_PINNED_TEXT, "memory-cap-hit"), (CAP_HIT_GROWN_PINNED_TEXT, "memory-cap-hit")] or first["task"].get("outcome") != "memory-cap-at-maximum":
        raise SelftestFailure(f"本机总上限 4G：头一件应当 {CAP_HIT_PINNED_TEXT}、{CAP_HIT_GROWN_PINNED_TEXT} 各撞一次上限、到顶记 memory-cap-at-maximum，实际 {seen}、{describe(first)}")
    world.resources(local=(30, 30))
    second_id = world.submit(["bash", "-c", "exit 0"], extra={lib.KEY_VARIABLE: key})
    second = world.finished(second_id)
    attempt = second["attempts"][0] if second["attempts"] else {}
    wrapper_caps = wrapper_cap_texts(world, second_id)
    above = f"{int(int(CAP_HIT_GROWN_PINNED_TEXT[:-1]) * lib.CAP_GROWTH_FACTOR)}M"   # 撞过、之后这把键没跑完过：照撞过的最大上限再乘 1.5（cap_hit_measurement 的 unresolved）
    if second["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != above or wrapper_caps[:1] != [above]:
        raise SelftestFailure(f"撞过 {CAP_HIT_PINNED_TEXT} 与 {CAP_HIT_GROWN_PINNED_TEXT}、之后没跑完过的命令再交：第一次就应当从 {above} 起（撞过的最大上限乘 1.5），"
                              f"实际上限 {attempt.get('memory_cap_text')}、包装收到 {wrapper_caps}、{describe(second)}", world.server_log_lines())
    return f"撞过 {CAP_HIT_PINNED_TEXT} 与 {CAP_HIT_GROWN_PINNED_TEXT}、之后没跑完过的命令再交，第一次就从撞过的最大上限乘 1.5 的 {above} 起"


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


def recorded_host_line(world, task_id):
    """客户端 status <号> 打的第一次尝试那一行（「第 1 次：本机、上限 …」）；没有交空串。"""
    shown = world.client(["status", str(task_id)])
    return next((line.strip() for line in shown.stdout.splitlines() if "第 1 次：" in line), "")


def cell_balanced_when_both_fit(world):
    world.resources(local=(40, 40), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-long-and-big", 10 * GIBIBYTE)
    world.seed_duration("selftest-long-and-big", 3600, ["bash", "-c", "exit 0"])
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
    与第二台在同一棵共享的树里跑之后放开的那几类（task-server/、scripts/agents/、.claude/hooks/、带 --write 的、scripts/kb/、合入后验证 merge-and-verify），
    本机放不下（各记 15G、本机空 8G）、第二台放得下时都派到第二台跑完；删之前只在第二台跑的 e185-r2-segment-run.sh，第二台用不了时在本机跑完。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-every-command", 10 * GIBIBYTE)
    world.start_server()
    key = {lib.KEY_VARIABLE: "selftest-every-command"}
    commands = {"scripts/hosts/ 下的": "scripts/hosts/multi-host-run.sh", ".claude/gate.d/ 下的": ".claude/gate.d/doc-text.sh",
                "task-server/ 下的": "task-server/task_server_lib.py", "scripts/agents/ 下的": "scripts/agents/agent-cost.py",
                ".claude/hooks/ 下的": ".claude/hooks/lib_project_code_runs.py", "带 --write 的": "--write", "scripts/kb/ 下的": "scripts/kb/replace-once.py",
                "合入后验证 merge-and-verify": "scripts/gate/merge-and-verify.sh", "删之前只在第二台的": SERVER.RESTORED_HOST_LISTS["peer_entries"][0]}
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


SHARD_BINARIES_NAMES = ("crash_enumeration-5f3a91c0", "power_loss_replay-9b21d4e7", "binaries.tsv")   # shard-binaries-removed-at-outcome：计划那一步放进 test-binaries/ 的几份（两份测试二进制与清单）
SHARD_BINARIES_NOTE_SECONDS = 10.0   # shard-binaries-removed-at-outcome：整件跑成之后等那一行操作日志的安静时限（结局记下之后才删、才记）


def shard_plan_directory_of(world, task_id):
    """假分片入口计划那一步记下的计划目录（调用记录里的 sharded-plan-binaries 一行；目录是 <库标签>-<父任务号>/plan）；没记的交空串。"""
    for row in world.sharded_calls("plan-binaries"):
        if row.get("directory", "").endswith(f"-{task_id}/plan"):
            return row["directory"]
    return ""


def shard_binaries_notes(world, task_id):
    """操作日志里这条父任务 shard-binaries 那几行的说明（task_server.py 的 shard_binaries_removal 记的）。"""
    return [operation.get("note", "") for operation in world.operations() if operation.get("action") == "shard-binaries" and operation.get("task") == task_id]


def cell_shard_binaries_removed_at_outcome(world):
    """分片父任务有了结局时服务删计划目录下的 test-binaries/（task_server.py 的 shard_binaries_removal；文件头 ④）：
    一片在跑时取消整件：回来时 test-binaries/ 已删、plan.json 还在，操作日志写删了 3 份；再往那里放一份，task clean 照删、plan.json 还在、操作日志写 1 份；
    计划那一步把 test-binaries 建成指到计划目录外的符号链接、整件跑成：不删，外面那个目录里的文件还在、链接还在，操作日志写没删，服务日志打那一行。
    弄坏开关 shard-binaries-kept 下取消与 clean 之后都还在，shard-binaries-path-unchecked 下计划目录外那一份被删，这一格红。"""
    release = os.path.join(world.root, "release-binaries-shard")
    world.start_server()
    problems = []
    cancelled = world.submit(world.write_spec("spec-binaries-cancel.json", {"shards": [{"block": release}], "plan_binaries": list(SHARD_BINARIES_NAMES)}), cwd=world.repo)
    try:
        world.wait_for(cancelled, lambda summary: any(attempt.get("state") == "running" for child in shard_children(summary).values() for attempt in child["attempts"]),
                       "那一片跑起来")
        completed = world.client(["cancel", str(cancelled)])
    finally:
        open(release, "w").close()   # 取消没停下那一片：放它跑完，收尾不等满 60 秒
    plan = shard_plan_directory_of(world, cancelled)
    binaries = os.path.join(plan, lib.SHARD_PLAN_BINARIES_NAME)
    outcome = world.summary(cancelled)["task"].get("outcome")
    notes = shard_binaries_notes(world, cancelled)
    if (not plan or completed.returncode != 0 or outcome != "cancelled" or os.path.lexists(binaries) or not os.path.exists(os.path.join(plan, lib.PLAN_FILE))
            or not any(f"{len(SHARD_BINARIES_NAMES)} 份" in note for note in notes)):
        problems.append(f"一片在跑时取消整件：应当退 0、记 cancelled，回来时计划目录下的 {lib.SHARD_PLAN_BINARIES_NAME}/ 已删、{lib.PLAN_FILE} 还在、操作日志写删了 "
                        f"{len(SHARD_BINARIES_NAMES)} 份，实际退 {completed.returncode}、结局 {outcome}、计划目录「{plan or '没记下'}」、"
                        f"{lib.SHARD_PLAN_BINARIES_NAME}/ {'还在' if os.path.lexists(binaries) else '不在'}、{lib.PLAN_FILE} {'在' if os.path.exists(os.path.join(plan, lib.PLAN_FILE)) else '不在'}、操作日志 {notes}")
    if plan:
        os.makedirs(binaries, exist_ok=True)
        with open(os.path.join(binaries, "late-binary"), "w", encoding="utf-8") as handle:
            handle.write("late\n")
        cleaned = world.client(["clean", str(cancelled)])
        notes = shard_binaries_notes(world, cancelled)
        if cleaned.returncode != 0 or os.path.lexists(binaries) or not os.path.exists(os.path.join(plan, lib.PLAN_FILE)) or not any("1 份" in note for note in notes):
            problems.append(f"再往 {lib.SHARD_PLAN_BINARIES_NAME}/ 放一份之后 task clean：应当退 0、那个子目录删了、{lib.PLAN_FILE} 还在、操作日志写删了 1 份，"
                            f"实际退 {cleaned.returncode}、{lib.SHARD_PLAN_BINARIES_NAME}/ {'还在' if os.path.lexists(binaries) else '不在'}、操作日志 {notes}、"
                            f"输出「{(cleaned.stdout + cleaned.stderr).strip()[-300:]}」")
    outside = os.path.join(world.root, "binaries-elsewhere")
    linked = world.submit(world.write_spec("spec-binaries-link.json", {"shards": [{}], "plan_binaries_link": outside}), cwd=world.repo)
    done = world.finished(linked)
    patience = world.patience(SHARD_BINARIES_NOTE_SECONDS)
    while patience.waiting() and not shard_binaries_notes(world, linked):
        time.sleep(POLL_SECONDS)
    notes = shard_binaries_notes(world, linked)
    link = os.path.join(shard_plan_directory_of(world, linked), lib.SHARD_PLAN_BINARIES_NAME)
    logged = any("不是这一条任务自己的计划目录下的那一份" in line for line in world.server_log_lines())
    if (done["task"].get("outcome") != "succeeded" or not os.path.exists(os.path.join(outside, "kept")) or not os.path.islink(link)
            or not any(note.startswith("没删") for note in notes) or not logged):
        problems.append(f"{lib.SHARD_PLAN_BINARIES_NAME} 是指到计划目录外的符号链接、整件跑成：应当不删（外面那份 kept 还在、链接还在）、操作日志写没删、服务日志打一行，"
                        f"实际结局 {done['task'].get('outcome')}、kept {'在' if os.path.exists(os.path.join(outside, 'kept')) else '不在'}、"
                        f"链接 {'在' if os.path.islink(link) else '不在'}、操作日志 {notes}、服务日志{'有' if logged else '没有'}那一行")
    raise_cell_problems(world, problems, "分片的测试二进制")
    return (f"取消整件时计划目录下的 {lib.SHARD_PLAN_BINARIES_NAME}/ 删了（{lib.PLAN_FILE} 留着）、task clean 照删；"
            f"它是指到计划目录外的符号链接的不删，操作日志写「{notes[0][:60] if notes else ''}…」")


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


# 开了 GPU 的多机配置里换掉的那一段：判法要 ENABLE_GPU=1 时两个显卡键都写（这里各写本机一张卡，判法不判值）
GPU_ON_CONFIGURATION_KEYS = f"ENABLE_GPU=1\nGPU_CARDS_BY_PRIORITY={LOCAL_CARD_ZERO}\nGPU_CARDS_BY_PRIORITY_WHEN_QUIESCED={LOCAL_CARD_ZERO}\n"
# control 起 gpu-verification-server/control 时给的出路里要有的那一句（control 的 GPU_SERVICE_START_HOWTO 开头）
GPU_SERVICE_START_TEXT = "python3 gpu-verification-server/control start"


def gpu_on_configuration(world):
    """这一格世界的多机配置抄一份、只把 ENABLE_GPU=0 那一行换成 GPU_ON_CONFIGURATION_KEYS，交新那一份的路径：task-server 读的键与世界那一份相同（同一个服务、同一个 socket）。"""
    with open(world.config, encoding="utf-8") as handle:
        text = handle.read()
    if text.count("\nENABLE_GPU=0\n") != 1:
        raise SelftestFailure(f"世界的多机配置 {world.config} 里 ENABLE_GPU=0 那一行应当恰好一行，实际 {text.count('ENABLE_GPU=')} 处写了 ENABLE_GPU")
    path = os.path.join(world.root, "gpu-on-multi-host.env")
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text.replace("\nENABLE_GPU=0\n", "\n" + GPU_ON_CONFIGURATION_KEYS))
    return path


def gpu_service_calls(world):
    """假 gpu-verification-server/control 收到过的调用（calls 里「gpu-service-control 」起头的行，按先后）。"""
    return [line for line in world.calls() if line.startswith("gpu-service-control ")]


def write_gpu_service_exit(world, code):
    """假 gpu-verification-server/control 之后退几（世界目录里的 gpu-service-exit）。"""
    with open(os.path.join(world.world, "gpu-service-exit"), "w", encoding="utf-8") as handle:
        handle.write(f"{code}\n")


def cell_gpu_service_required_when_gpu_on(world):
    """开了 GPU（多机配置 ENABLE_GPU=1）：control check 在任务服务那几样都过之后接着起 gpu-verification-server/control check（假的，照 gpu-service-exit 退），
    它退 78 时 check 退 78、出路写起它的命令；control start 在任务服务起好之后接着起 gpu-verification-server/control start，它退 78 时 start 退 78；
    假的退 0 时 check、start 都退 0、各起它一次。"""
    world.start_server()
    extra = {lib.MULTI_HOST_CONFIG_VARIABLE: gpu_on_configuration(world), "TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl"),
             "TASK_SERVER_GPU_SERVICE_CONTROL": os.path.join(world.bin, "fake-gpu-service-control")}
    problems = []
    for service_exit, want in ((78, lib.RUN_CONDITION_EXIT), (0, 0)):
        write_gpu_service_exit(world, service_exit)
        for subcommand in ("check", "start"):
            before = len(gpu_service_calls(world))
            completed = world.control([subcommand], extra=extra)
            called = gpu_service_calls(world)[before:]
            named = service_exit == 0 or GPU_SERVICE_START_TEXT in completed.stderr
            if completed.returncode != want or called != [f"gpu-service-control {subcommand}"] or not named:
                problems.append(f"假 GPU 服务退 {service_exit} 时 control {subcommand} 应当起它一次（「gpu-service-control {subcommand}」）、退 {want}"
                                f"{'、出路写「' + GPU_SERVICE_START_TEXT + '」' if service_exit else ''}，实际退 {completed.returncode}、起了 {called}、"
                                f"stderr「{completed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("开了 GPU：" + "；".join(problems))
    return "开了 GPU：control check、start 都接着起 gpu-verification-server/control；它退 78 时 check、start 退 78、出路写起它的命令，退 0 时都退 0"


def cell_gpu_service_untouched_when_gpu_off(world):
    """没开 GPU（多机配置 ENABLE_GPU=0，各格世界的缺省）：假 gpu-verification-server/control 设着、照 gpu-service-exit 退 78，
    control check、start 都不起它，照任务服务自己的结论退 0。"""
    world.start_server()
    write_gpu_service_exit(world, 78)
    extra = {"TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl"),
             "TASK_SERVER_GPU_SERVICE_CONTROL": os.path.join(world.bin, "fake-gpu-service-control")}
    problems = []
    for subcommand in ("check", "start"):
        completed = world.control([subcommand], extra=extra)
        if completed.returncode != 0 or gpu_service_calls(world):
            problems.append(f"control {subcommand} 应当不起 gpu-verification-server/control、退 0，实际退 {completed.returncode}、起了 {gpu_service_calls(world)}、"
                            f"stderr「{completed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("没开 GPU：" + "；".join(problems))
    return "没开 GPU：control check、start 都不起 gpu-verification-server/control，退 0"


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
    """疑似卡住只通知、不停（task_server.py 的 check_suspect、judge_stall_stop）：窗口 6 秒（脚本每 0.25 秒打一行，输出间隔要比窗口小得多：机器忙时输出断档不到 6 秒，
    不会被判成日志不涨而停掉），假 scope 的 CPU 不涨，约 15 秒的脚本：头一回 wait 打 TASK_SERVER_SUSPECT、退 SUSPECT_EXIT，任务照跑、status 写疑似卡住、记告警；
    再起 wait 拿到 succeeded，同一段只报一次。弄坏开关 suspect-unreported 下不判，这一格红。"""
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "6", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01",
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
    world.seed_peak(PEAKS.command_key(probe, None), 1536 * MEBIBYTE)   # 学到的峰值 1536M：没钉时上限照它 1536M，钉了不顶掉钉值
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


def cell_pinned_cap_over_limit_reported(world):
    """钉值不超过 slice 总上限减余量（task_server.py 的 pin_value、pin_over_limit_lines；任务 10289 钉到 51200M 等于 slice 总上限）：配置写 60G、那一台总上限 50G、假余量 2G：
    control pin-cap 退 0、报本机 48G（49152M）、打一行「!」写超过与当多少用；起的那一次上限就是它。弄坏开关 pinned-over-limit-unreported 下报 58G、不打那一行，这一格红。"""
    world.set_slice_cap("60G")
    world.resources(local=(55, 50))
    probe = ["bash", "-c", "exit 0", "pinned-over-limit"]
    world.start_server()
    pinned = world.control(["pin-cap", "--", *probe])
    problems = []
    if pinned.returncode != 0 or shown_local_pin(pinned) != 49152 or "超过这台 slice 总上限减余量" not in pinned.stdout:
        problems.append(f"control pin-cap 应当退 0、报本机 49152M、打一行超过那一句，实际退 {pinned.returncode}、「{(pinned.stdout + pinned.stderr).strip()[-500:]}」")
    summary = world.finished(world.submit(probe))
    if first_attempt(summary).get("memory_cap_text") != "49152M":
        problems.append(f"起的那一次上限应当是 49152M，实际 {describe(summary)}")
    if problems:
        raise SelftestFailure("钉值超过 slice 总上限减余量：" + "；".join(problems), world.server_log_lines())
    return "配置写 60G、总上限 50G、余量 2G：pin-cap 回读报 49152M 与超过那一句，起的那一次照它"


def cell_pinned_squeezed_requeued(world):
    """钉了的命令被 slice 挤掉（task_server.py 文件头 ②「内存」的钉值；任务 10289 内存包装退 254、写「这一条的结果不算数，重跑」，服务却打回 memory-cap-at-maximum）：
    钉值等于这台单条能给的最大、第一次退 254：记 squeezed-out、同一上限重排、第二次跑完 succeeded。弄坏开关 pinned-squeeze-as-cap-hit 下记撞上限、打回，这一格红。"""
    world.set_slice_cap("50G")
    world.resources(local=(55, 50))
    command = ["bash", "-c", "exit 0", "pinned-squeezed", "fake-wrapper-exits=254,run"]
    world.start_server()
    pinned = world.control(["pin-cap", "--", *command])
    summary = world.finished(world.submit(command))
    seen = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in summary["attempts"]]
    if pinned.returncode != 0 or [outcome for _, outcome in seen] != ["squeezed-out", "succeeded"] or seen[0][0] != seen[1][0] or summary["task"].get("outcome") != "succeeded":
        raise SelftestFailure(f"钉了的命令退 254 应当记 squeezed-out、同一上限重排、跑完，实际 pin-cap 退 {pinned.returncode}、各次 {seen}、{describe(summary)}", world.server_log_lines())
    return "钉了的命令退 254 记 squeezed-out、同一上限重排、跑完，不打回"


def cell_pin_cap_help_not_pinned(world):
    """control pin-cap --help、-h 打用法、不钉（操作记录有一条把 --help 钉成命令的）。弄坏开关 pin-cap-help-pinned 下钉上了，这一格红。"""
    world.start_server()
    problems = []
    for word in ("--help", "-h"):
        shown = world.control(["pin-cap", word])
        if shown.returncode != 0 or "写成 python3 task-server/control pin-cap" not in shown.stdout or "钉上了" in shown.stdout:
            problems.append(f"pin-cap {word} 应当退 0、打用法、不钉，实际退 {shown.returncode}、「{(shown.stdout + shown.stderr).strip()[-300:]}」")
        left = world.control(["unpin-cap", "--", word])
        if left.returncode != 0 or "原来就没钉" not in left.stdout:
            problems.append(f"pin-cap {word} 之后命令 {word} 应当没钉，实际 unpin-cap 退 {left.returncode}、「{(left.stdout + left.stderr).strip()[-300:]}」")
    if problems:
        raise SelftestFailure("pin-cap 的用法：" + "；".join(problems), world.server_log_lines())
    return "pin-cap --help、-h 打用法退 0，没钉"


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
    world.seed_peak(key, 30 * GIBIBYTE)   # 预测、上限与记账都是 30G（近五次最大，不乘 1.5）
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
        raise SelftestFailure(f"两条各记过峰值 30G（上限与记账都是 30G）、本机 50G：头一条记账 30G，第二条（再记 30G 就超过 50G）应当排着、头一条放了才起；"
                              f"实际头一条记账 {booked} 字节、第二条先是 {waiting['task'].get('state')}、尝试 {len(waiting['attempts'])} 次、最后 {describe(later)}",
                              world.server_log_lines())
    return "两条各记过峰值 30G 在 50G 的台上：头一条记账 30G（上限）、第二条排着，头一条放了才起"


SHARD_FITTED_FLOOR_BYTES = 30 * GIBIBYTE   # shard-cap-fitted-to-room 这一格的内存上限下限（TASK_SERVER_SELFTEST_MEMORY_FLOOR_BYTES）：两片各记过峰值 20G，上限与记账抬到 30G、预测仍是 20G


def cell_shard_cap_fitted_to_room(world):
    """分片的片记满上限放不下、预测放得下时，上限压到放得下的量（task_server_lib.py 文件头「资源记录」的准入；task_server.py 的 shard_cap_fitted_to_room）。
    记录定的预测、上限与记账相等（近五次最大，不乘 1.5），只有内存上限的下限把上限与记账抬到预测之上时才有「记满上限放不下、预测放得下」，这一格照这一种造：
    下限设 30G（SHARD_FITTED_FLOOR_BYTES），两片各记过峰值 20G（上限与记账抬到 30G）、本机 54G、余量 2G：第一片上限 30G，第二片压到 54G − 2G − 30G = 22G，
    两片同时跑、上限之和恰好等于总上限减余量，两片的 prediction 都记 RECORD_HISTORY（照各片自己的键读到了记录），整件成了。
    弄坏开关 shard-cap-not-fitted 下第二片排着，shard-key-position-folded 下各片读不到自己的记录，这一格红。"""
    key = "selftest-shard-cap-fitted"
    for index in (0, 1):
        world.seed_peak(lib.shard_child_key(key, "shard", index, 2), 20 * GIBIBYTE, shard_count=2)
    world.resources(local=(54, 54), peer_down="自证：第二台此刻用不了")
    release = os.path.join(world.root, "release-fitted")
    spec = world.write_spec("spec-fitted.json", {"shards": [{"block": release}, {"block": release}]})
    world.start_server(extra={lib.MEMORY_FLOOR_VARIABLE: str(SHARD_FITTED_FLOOR_BYTES)})
    task_id = world.submit(spec, cwd=world.repo, extra={lib.KEY_VARIABLE: key})
    try:
        summary = world.wait_for(task_id, lambda summary: len(running_shards(summary)) == 2, "两片同时在本机跑起来")
        firsts = [child["attempts"][0] for child in shard_children(summary).values()]
        caps = sorted(int(attempt.get("memory_cap") or 0) for attempt in firsts)
        histories = sorted({str(attempt.get("prediction")) for attempt in firsts})
    finally:
        open(release, "w").close()
    finished = world.finished(task_id)
    floor = SHARD_FITTED_FLOOR_BYTES
    if (caps != [(54 - SELFTEST_MARGIN_GIBIBYTES) * GIBIBYTE - floor, floor] or histories != [lib.RECORD_HISTORY]
            or finished["task"].get("outcome") != "succeeded"):
        raise SelftestFailure(f"下限 30G、两片各记过峰值 20G、本机 54G、余量 {SELFTEST_MARGIN_GIBIBYTES}G：第一片上限 30G，第二片压到总上限减余量减第一片"
                              f"（上限之和恰好等于总上限减余量）、两片 prediction 都记 {lib.RECORD_HISTORY}、整件成了；"
                              f"实际上限 {caps}、prediction {histories}、整件 {describe(finished)}", world.server_log_lines())
    return f"下限抬到 30G 的两片（峰值各 20G）在本机 54G：上限 {[cap // GIBIBYTE for cap in caps]}G，同时跑、之和不超过总上限减余量"


SHARD_KEY_BASES = ("selftest-shard-keys", "bash scripts/crash/crash-amplification-two-hosts.sh --merged-log merged.log")   # shard-keys-kept-apart：交了键的、没交键按命令取的


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


def cell_resume_declaration_first_word_only(world):
    """续跑声明的判法（task_server_lib.py 的 resumable_command、declaration_matches）：只看第一个非选项参数，后面的参数对上词不算；* 照旧。静态喂文件头，不起服务。
    弄坏开关 resume-declaration-any-word 下非选项参数有一个对上就算，这一格红。"""
    script = os.path.join(world.work, "declared-resume.sh")
    with open(script, "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_RESUMABLE_MARK} segment\nexit 0\n")
    everything = os.path.join(world.work, "declared-resume-all.sh")
    with open(everything, "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_RESUMABLE_MARK} *\nexit 0\n")
    cases = ((script, ["segment"], True), (script, ["--quick", "segment"], True), (script, ["other", "segment"], False), (script, [], False), (everything, ["other"], True))
    problems = []
    for path, arguments, expected in cases:
        found = lib.resumable_command(["bash", path, *arguments], world.work)
        if found != expected:
            problems.append(f"{os.path.basename(path)} {arguments} 应当 {expected}，实际 {found}")
    if problems:
        raise SelftestFailure("续跑声明：" + "；".join(problems))
    return f"{len(cases)} 条：只看第一个非选项参数，* 照旧"


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


RUNNER_THREADS_SPINNING = 3    # runner-threads-file-counts：真进程里空转的线程数（pbkdf2_hmac 在 OpenSSL 里算时放开 GIL，一直是 R）
RUNNER_THREADS_SLEEPING = 4    # 睡着等事件的线程数（S）
RUNNER_THREADS_NAME = "spin-and-sleep"   # 那个进程的 comm（不超过 15 个字节，内核不截）
RUNNER_THREADS_SECONDS = 20.0  # 等读数对上的安静时限
RUNNER_THREADS_PROGRAM = r'''
import hashlib, sys, threading
stop = threading.Event()
for _ in range(int(sys.argv[2])):
    threading.Thread(target=lambda: hashlib.pbkdf2_hmac("sha256", b"spin", b"salt", 2 ** 31 - 1), daemon=True).start()
for _ in range(int(sys.argv[3])):
    threading.Thread(target=stop.wait, daemon=True).start()
with open("/proc/self/comm", "w", encoding="utf-8") as handle:
    handle.write(sys.argv[4])
with open(sys.argv[1], "w", encoding="utf-8") as handle:
    handle.write("ready\n")
stop.wait()
'''


def cell_runner_threads_file_counts(world):
    """attempt_runner 每轮写的线程读数（attempt_runner.py 文件头 ③ 的 THREADS_FILE_NAME；note_threads）：起一个真的进程树，最上面是在 wait 的 bash，底下一个 python
    （3 个空转线程、4 个睡着的线程，主线程也睡着，comm 改成 spin-and-sleep），载入 attempt_runner.py 的 Sampler 对这棵树数：
    读数应当是 runnable 3、total 9（bash 1 个加 python 8 个）、top_threads 8、top_name spin-and-sleep、sample 是这一轮的轮数。整组在它自己的进程组里，收尾对整组发 KILL、核没留下进程。
    弄坏开关 runner-threads-state-unread 下（说明里 threads_state_unread）每个线程都算可跑的，runnable 成 9，这一格红。"""
    import importlib.util
    spec = importlib.util.spec_from_file_location("attempt_runner_under_test", lib.RUNNER_SCRIPT)
    runner = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(runner)
    directory = os.path.join(world.work, "runner-threads")
    os.makedirs(directory, exist_ok=True)
    ready = os.path.join(directory, "ready")
    total = 1 + 1 + RUNNER_THREADS_SPINNING + RUNNER_THREADS_SLEEPING
    process = subprocess.Popen(["bash", "-c", '"$1" -c "$2" "$3" "$4" "$5" "$6" & wait', "runner-threads", sys.executable, RUNNER_THREADS_PROGRAM, ready,
                                str(RUNNER_THREADS_SPINNING), str(RUNNER_THREADS_SLEEPING), RUNNER_THREADS_NAME], stdin=subprocess.DEVNULL, start_new_session=True)
    sampler = runner.Sampler({"directory": directory, "threads_state_unread": lib.broken("runner-threads-state-unread")}, process.pid, None)
    reading, family = {}, {}
    try:
        patience = world.patience(RUNNER_THREADS_SECONDS)
        while patience.waiting():
            if os.path.exists(ready):
                sampler.rounds += 1
                family = runner.descendants_with_depth(process.pid)
                sampler.note_threads(family)
                with open(os.path.join(directory, runner.THREADS_FILE_NAME), encoding="utf-8") as handle:
                    reading = json.load(handle)
                if reading == {"sample": sampler.rounds, "runnable": RUNNER_THREADS_SPINNING, "total": total, "top_threads": total - 1, "top_name": RUNNER_THREADS_NAME}:
                    break
            time.sleep(POLL_SECONDS)
    finally:
        with contextlib.suppress(ProcessLookupError):
            os.killpg(process.pid, signal.SIGKILL)   # 整组都是这一格自己起的
        process.wait()
    patience = world.patience(RUNNER_THREADS_SECONDS)
    while patience.waiting() and not all(process_gone(process_id) for process_id in family):
        time.sleep(POLL_SECONDS)
    left = [process_id for process_id in family if not process_gone(process_id)]
    wanted = {"sample": sampler.rounds, "runnable": RUNNER_THREADS_SPINNING, "total": total, "top_threads": total - 1, "top_name": RUNNER_THREADS_NAME}
    if reading != wanted or left:
        raise SelftestFailure(f"线程读数应当是 {wanted}（{RUNNER_THREADS_SPINNING} 个空转、{RUNNER_THREADS_SLEEPING} 个睡着、主线程与 bash 也睡着），收尾之后不留进程；"
                              f"实际最后一回 {reading or '没读到'}、留下的进程 {left}")
    return f"真进程树（{RUNNER_THREADS_SPINNING} 个空转、{RUNNER_THREADS_SLEEPING} 个睡着）的线程读数 {reading}"


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


LONG_RUN_SELFTEST_SECONDS = 20   # long-run-failure-noted：自证里「跑得久」的界（TASK_SERVER_SELFTEST_LONG_RUN_SECONDS）；立刻退的那一条在机器忙时起跑、收尾也远不到它
LONG_RUN_SELFTEST_SLEEP_SECONDS = LONG_RUN_SELFTEST_SECONDS + 1   # 那一格「跑得久」的几条睡这么久：只会比它更久，一定过界


def cell_long_run_failure_noted(world):
    """跑超过时限而失败、又没登记续跑的，结局说明写「这条跑了 N 分钟、没有续跑：重交会从头跑」（task_server.py 的 finished_preface；时限在自证里缩成 20 秒）：
    登记了续跑的（文件头 # task-server-resumable:）、没到时限的（立刻退 3）、跑成了的都不写。跑得久的几条睡 21 秒（睡只会更久，一定过界）；四条先都交上再逐条等收尾。"""
    world.start_server(extra={"TASK_SERVER_SELFTEST_LONG_RUN_SECONDS": str(LONG_RUN_SELFTEST_SECONDS)})
    with open(os.path.join(world.work, "resumable.sh"), "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_RESUMABLE_MARK} segment\nsleep {LONG_RUN_SELFTEST_SLEEP_SECONDS}\nexit 3\n")
    cases = ((f"跑了 {LONG_RUN_SELFTEST_SLEEP_SECONDS} 秒、退 3、没登记续跑", ["bash", "-c", f"sleep {LONG_RUN_SELFTEST_SLEEP_SECONDS}; exit 3"], True),
             ("登记了续跑", ["bash", "resumable.sh", "segment"], False),
             ("没到时限", ["bash", "-c", "exit 3"], False),
             ("跑成了", ["bash", "-c", f"sleep {LONG_RUN_SELFTEST_SLEEP_SECONDS}; exit 0"], False))
    problems = []
    submitted = [(name, world.submit(command, cwd=world.work), expected) for name, command, expected in cases]
    for name, task_id, expected in submitted:
        summary = world.finished(task_id)
        note = summary["task"].get("note") or ""
        if ("没有续跑：重交会从头跑" in note) != expected:
            problems.append(f"{name}：结局说明{'应当' if expected else '不该'}写「没有续跑：重交会从头跑」，实际「{note}」、{describe(summary)}")
    if problems:
        raise SelftestFailure("跑得久而失败的说明：" + "；".join(problems), world.server_log_lines())
    return "跑过时限、失败、没登记续跑的结局说明写重交会从头跑；登记了续跑、没到时限、跑成了的不写"


ALTERNATION_THRESHOLD_BYTES = int((50 * GIBIBYTE - 2 * GIBIBYTE) * SERVER.WHOLE_FRACTION)   # 50G 的台：可用上限 48G 的 85%（40.8G）
MEMORY_FLOOR_RECORDED_SEED_BYTES = 1371 * MEBIBYTE   # retry-cap-not-floored：记下的那一次峰值，比下限 1024M 大、比 4G 小
MEMORY_FLOOR_RECORDED_TEXT = "1371M"       # 那一格头一次的上限：记下的峰值本身（近五次最大，不乘 1.5），有记录的不兜
MEMORY_FLOOR_RETRY_TEXT = "2057M"          # 那一格撞了重排那一次的上限：1371M × 1.5 按 MiB 向上取整，不兜到 4G
RECORDED_SMALL_PEAK_BYTES = 1300 * MEBIBYTE  # recorded-cap-not-floored：记下的那一次峰值（有记录、比下限 1024M 大）
RECORDED_SMALL_CAP_TEXT = "1300M"            # 记下的峰值本身（近五次最大，不乘 1.5）
# 内存上限的下限（lib.MEMORY_FLOOR_BYTES，1024M）：recorded-cap-not-floored 与 retry-cap-not-floored 两格照真的下限（小世界写 0 的换成空串）
FLOOR_TEXT = "1024M"
FLOOR_TINY_PEAK_BYTES = 5 * MEBIBYTE        # 记录的峰值很小（任务 11127 那一种）：照它只有 5M，装不下读文件带进来的页缓存，上限取下限
FLOOR_NEAR_PEAK_BYTES = 200 * MEBIBYTE      # 照它 200M，低于下限，上限仍取下限 1024M
FLOOR_RETRY_TEXT = "1536M"                  # 撞了重排那一次：1024M × 1.5
PINNED_BELOW_FLOOR_SLICE_TEXT = "2176M"     # 多机配置里两台那一项：减假余量 2G 是钉值 128M，低于下限
PINNED_BELOW_FLOOR_BYTES = 128 * MEBIBYTE
DRAINED_FRAGMENT_PEAK_BYTES = 15 * GIBIBYTE   # 那一格占着的小任务：上限与记账 15G


def cell_whole_starts_beside_running_fragment(world):
    """一台 50G（可用上限 48G、界 40.8G）上一条碎片（4G）长跑着，整片（峰值 42G，上限与记账 42G）放得下就起，不等那台腾空
    （任务 2461 留成孤儿的两小时里本机只用 2G、10 条排着等「腾空」）。"""
    world.resources(local=(50, 50))
    world.seed_peak("selftest-whole-beside", 42 * GIBIBYTE)
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


CPU_GATE_BUSY_KEY = "selftest-cpu-gate-busy"   # cpu-gate-holds-busy-host、health-head-cpu-gate-not-starved：库里记过 31 核的命令键（要的核数多，不走要核数少的名额）


def seed_busy_cores(world, key):
    """往这一格服务自己的库里记一次跑成、峰值段 31 核、CPU 闸线程数 31（平均在用核数也是 31）的尝试（task_server.py 的 threads_by_record 读它）：这把键的命令过 CPU 闸照 31 线程记，不走要核数少的名额。"""
    world.seed_attempt({"command_key": key, "outcome": "succeeded", "memory_peak": None, "cores": 31.0, SERVER.PEAK_SEGMENT_CORES_COLUMN: 31.0, SERVER.CPU_GATE_THREADS_COLUMN: 31.0})


def cell_cpu_gate_holds_busy_host(world):
    """本机内存够、负载 200（32 核 × 2 = 64 放不下再加一条的线程）：命令键记过 31 核的任务排着、排队原因写「CPU：」；负载降到 0 之后照常跑完（task_server.py 的 cpu_problem）。"""
    world.resources(local=(50, 50), local_cpu={"load1": 200.0, "procs_running": 150, "cpus": 32})
    seed_busy_cores(world, CPU_GATE_BUSY_KEY)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "cpu-gate"], extra={lib.KEY_VARIABLE: CPU_GATE_BUSY_KEY})
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
    """改主工作区的命令（打补丁的 apply-writer-patch.py）撞了上限记 memory-cap-hit、被总上限挤掉记 squeezed-out，都只有一次尝试、打回，不原样重排
    （WORKSPACE_WRITER_NO_RETRY_OUTCOMES）；合入后验证 merge-and-verify.sh（只在快照上打与验，不算改主工作区）与 hook-swap --feed-only 的照乘 1.5 重排跑完
    （task_server.py 的 modifies_workspace、finish_attempt）。
    弄坏开关 workspace-writers-retried 下打补丁的那两条照重排，feed-only-counted-as-writer 下只喂的那一条、merge-and-verify-counted-as-writer 下合入后验证那一条只跑一次、打回，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    writer = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/apply-writer-patch.py", "fake-wrapper-exits=250,run"], cwd=world.repo))
    squeezed = world.finished(world.submit(["bash", "-c", "exit 0", "y", "scripts/gate/apply-writer-patch.py", "fake-wrapper-exits=254,run"], cwd=world.repo))
    skipped = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/merge-and-verify.sh", "fake-wrapper-exits=250,run"], cwd=world.repo))
    feed = world.finished(world.submit(["bash", "-c", "exit 0", "x", "scripts/gate/hook-swap.py", "--feed-only", "fake-wrapper-exits=250,run"], cwd=world.repo))
    if (writer["task"].get("outcome") != "memory-cap-hit" or len(writer["attempts"]) != 1
            or squeezed["task"].get("outcome") != "squeezed-out" or len(squeezed["attempts"]) != 1
            or skipped["task"].get("outcome") != "succeeded" or len(skipped["attempts"]) != 2
            or feed["task"].get("outcome") != "succeeded" or len(feed["attempts"]) != 2):
        raise SelftestFailure(f"打补丁的工具（apply-writer-patch.py）撞上限应当记 memory-cap-hit、被挤掉应当记 squeezed-out，都只有一次尝试；合入后验证（只在快照上打与验）与 hook-swap --feed-only 的应当乘 1.5 重排跑完；"
                              f"实际撞上限的 {describe(writer)}、被挤掉的 {describe(squeezed)}、合入后验证的 {describe(skipped)}、只喂的 {describe(feed)}", world.server_log_lines())
    return "改主工作区的打补丁工具撞上限与被挤掉都只有一次、照那一种结局打回；合入后验证与 hook-swap --feed-only 的乘 1.5 重排跑完"


def cell_retry_cap_not_floored(world):
    """有记录的照记录、不设 4G 底，只设 1024M 的下限（task_server.py 文件头 ②「内存」；这一格照真的下限，不用各格小世界的 0，弄坏开关 recorded-cap-floored 下兜的就是它）：
    同一命令键记下的峰值 1371M，头一次照记录取 1371M（近五次最大，不乘 1.5）、假包装退 250 撞上限，重排那一次照 1371M × 1.5 = 2057M（不兜到 4096M）、记账照上限、跑完；
    记下的峰值只有 5M 的命令头一次上限与记账都取下限 1024M（5M 低于下限）、撞上限重排那一次 1536M（1024M × 1.5）；
    钉了上限的命令（多机配置 SLICE_MEMORY_CAP_BY_HOST 两台都写 2176M）照 pin-cap 报的钉值 128M：低于下限也照钉值，不抬到 1024M。
    弄坏开关 recorded-cap-floored 下前两条头一次都是 4096M，cap-floor-skipped 下峰值 5M 的那条是 5M、8M，这一格红。"""
    key = "selftest-memory-floor"
    tiny_key = "selftest-memory-floor-tiny"
    world.seed_peak(key, MEMORY_FLOOR_RECORDED_SEED_BYTES)
    world.seed_peak(tiny_key, FLOOR_TINY_PEAK_BYTES)
    world.set_slice_cap(PINNED_BELOW_FLOOR_SLICE_TEXT)   # 钉值 = 那一项减这台的假余量 2G（pin_value）：128M，低于下限
    world.start_server(extra={lib.MEMORY_FLOOR_VARIABLE: ""})
    retried = world.finished(world.submit(["bash", "-c", "exit 0", "memory-floor-retried", "fake-wrapper-exits=250,run"], extra={lib.KEY_VARIABLE: key}))
    floored = world.finished(world.submit(["bash", "-c", "exit 0", "memory-floor-tiny-retried", "fake-wrapper-exits=250,run"], extra={lib.KEY_VARIABLE: tiny_key}))
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
        problems.append(f"记下 1371M 的命令头一次应当照记录 {MEMORY_FLOOR_RECORDED_TEXT} 撞上限，重排那一次上限 {MEMORY_FLOOR_RETRY_TEXT}（不兜到 4G）、记账照上限、跑完，"
                        f"实际各次 {caps}、记账 {booked}、{describe(retried)}")
    floored_caps = [(attempt.get("memory_cap_text"), attempt.get("outcome")) for attempt in floored["attempts"]]
    floored_booked = [int(attempt.get("booked") or 0) for attempt in floored["attempts"]]
    floor_bytes = int(FLOOR_TEXT[:-1]) * MEBIBYTE
    floor_retry_bytes = int(FLOOR_RETRY_TEXT[:-1]) * MEBIBYTE
    if (floored_caps != [(FLOOR_TEXT, "memory-cap-hit"), (FLOOR_RETRY_TEXT, "succeeded")] or floored_booked != [floor_bytes, floor_retry_bytes]
            or floored["task"].get("outcome") != "succeeded"):
        problems.append(f"记下 5M 的命令头一次应当取下限 {FLOOR_TEXT} 撞上限，重排那一次上限 {FLOOR_RETRY_TEXT}（{FLOOR_TEXT} × 1.5）、记账各次照上限、跑完，"
                        f"实际各次 {floored_caps}、记账 {floored_booked}、{describe(floored)}")
    attempt = first_attempt(pinned)
    if (pinned_reply.returncode != 0 or shown != PINNED_BELOW_FLOOR_BYTES // MEBIBYTE or attempt.get("memory_cap_text") != f"{shown}M"
            or attempt.get("memory_cap") != PINNED_BELOW_FLOOR_BYTES or attempt.get("prediction") != lib.PINNED_HISTORY):
        problems.append(f"钉了上限的命令（SLICE_MEMORY_CAP_BY_HOST 两台都写 {PINNED_BELOW_FLOOR_SLICE_TEXT}）应当照 pin-cap 报的钉值（{PINNED_BELOW_FLOOR_BYTES // MEBIBYTE}M，低于下限 {FLOOR_TEXT}）、"
                        f"不抬到下限，实际 pin-cap 退 {pinned_reply.returncode}、报的钉值 {shown}M、上限 {attempt.get('memory_cap_text')}、"
                        f"prediction {attempt.get('prediction')}")
    if problems:
        raise SelftestFailure("重排不设 4G 底、设 1024M 的下限：" + "；".join(problems), world.server_log_lines())
    return (f"记下 1371M 的命令键：头一次照记录 {MEMORY_FLOOR_RECORDED_TEXT}、撞了重排那一次 {MEMORY_FLOOR_RETRY_TEXT}（不兜到 4G）、记账照上限；"
            f"记下 5M 的头一次 {FLOOR_TEXT}、重排 {FLOOR_RETRY_TEXT}；配置写 {PINNED_BELOW_FLOOR_SLICE_TEXT} 的钉值照 {shown}M（低于下限也照钉值）")


def cell_recorded_cap_not_floored(world):
    """有真实峰值记录的上限照峰值定（task_server.py 文件头 ②「内存」的有记录的照记录；这一格照真的下限 1024M，不用各格小世界的 0）：同一命令键记下的峰值 1300M，
    上限与记账都是 1300M（近五次最大，不乘 1.5）、prediction 记 RECORD_HISTORY、内存包装收到 cap=1300M，不兜到 4096M；没记录的照旧 4096M；
    记下的峰值很小的（5M 与 200M，都不到下限）上限与记账都取下限 1024M，prediction 仍记 RECORD_HISTORY。
    弄坏开关 recorded-cap-floored 下记录定的也兜到 4096M，cap-floor-skipped 下峰值很小的两条是 5M 与 200M，这一格红。"""
    key = "selftest-recorded-small"
    tiny_key = "selftest-recorded-tiny"
    near_key = "selftest-recorded-near-floor"
    world.seed_peak(key, RECORDED_SMALL_PEAK_BYTES)
    world.seed_peak(tiny_key, FLOOR_TINY_PEAK_BYTES)
    world.seed_peak(near_key, FLOOR_NEAR_PEAK_BYTES)
    world.start_server(extra={lib.MEMORY_FLOOR_VARIABLE: ""})
    recorded_id = world.submit(["bash", "-c", "exit 0", "recorded-small"], extra={lib.KEY_VARIABLE: key})
    unrecorded_id = world.submit(["bash", "-c", "exit 0", "recorded-small-unrecorded"])
    tiny_id = world.submit(["bash", "-c", "exit 0", "recorded-tiny"], extra={lib.KEY_VARIABLE: tiny_key})
    near_id = world.submit(["bash", "-c", "exit 0", "recorded-near-floor"], extra={lib.KEY_VARIABLE: near_key})
    expected = {"有记录的": (recorded_id, RECORDED_SMALL_CAP_TEXT, lib.RECORD_HISTORY),
                "没记录的": (unrecorded_id, UNRECORDED_DEFAULT_PINNED_TEXT, lib.UNRECORDED_HISTORY),
                "记下 5M 的": (tiny_id, FLOOR_TEXT, lib.RECORD_HISTORY),
                "记下 200M 的": (near_id, FLOOR_TEXT, lib.RECORD_HISTORY)}
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
    return (f"记下 1300M 的命令上限与记账都是 {RECORDED_SMALL_CAP_TEXT}（不兜到 4G），没记录的照旧 {UNRECORDED_DEFAULT_PINNED_TEXT}，"
            f"记下 5M 与 200M 的取下限 {FLOOR_TEXT}")


def cell_runaway_process_notified(world):
    """任务之外的进程空转（task_server.py 的 watch_runaway_processes）：进程表里只有不在任务 slice、不是内核线程、CPU 够久够满的那一条被报，恰好一条，写依据与按号停的出路；
    任务 slice 里的、内核线程、CPU 不够久的、占比不够的都不报。弄坏开关 runaway-process-unreported、runaway-slice-counted 下这一格红。"""
    listing = os.path.join(world.root, "process-list")
    inbox = FakeInbox(world, "a")
    other_inbox = FakeInbox(world, "b", session=SESSION_B, register=False)
    bystander = subprocess.Popen(["sleep", "600"])
    world.register_session(SESSION_B, other_inbox.path, "inbox-b", pid=bystander.pid)
    release = os.path.join(world.root, "release-runaway")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_PROCESS_LIST": listing})
    head = "task-server：任务之外的进程空转"
    try:
        world.running(world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "runaway", release], extra=inbox.environment()))
        world.running(world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "runaway-other", release], session=SESSION_B, extra=other_inbox.environment()))
        with open(listing, "w", encoding="utf-8") as handle:
            handle.write(f"  501 {os.getpid()} 7200 7100 R 0::/user.slice/session-3.scope rev\n  502 {os.getpid()} 7200 7100 R 0::/user.slice/singlefs.slice/a.service in-slice\n"
                         f"  503 2 7200 7100 R - kworker\n  504 {os.getpid()} 7200 600 S 0::/user.slice/session-3.scope brief\n  505 {os.getpid()} 7200 3000 S 0::/user.slice/session-3.scope low-share\n"
                         )
        inbox.wait_for(world, head)
        time.sleep(INBOX_QUIET_SECONDS)
        found = inbox.headed(head)
        strayed = other_inbox.headed(head)
    finally:
        open(release, "w").close()
        bystander.kill()
        bystander.wait()
    problems = inbox_problems(found, head, "任务之外的进程空转")
    if strayed:
        problems.append(f"别的会话不该收到，实际收到 {len(strayed)} 条")
    text = found[0]["text"] if found else ""
    if found and ("进程 501" not in text.split("\n", 1)[0] or "7100 秒" not in text or "proc.py stop 501" not in text):
        problems.append(f"应当点名进程 501、写依据与按号停的出路，实际「{text[:400]}」")
    if problems:
        raise SelftestFailure("任务之外的进程空转：" + "；".join(problems), world.server_log_lines())
    return "五行进程表只报不在 slice、CPU 够久够满的那一条，恰好一次，写了依据与出路"


def cell_runaway_process_stopped_after_reports(world):
    """任务之外的进程空转（task_server.py 的 watch_runaway_processes）：认得出发起会话的先报两次，再由服务按号停掉真进程并告诉那个会话；认不出的（进程 1 的子进程）不等、直接终止，不发给任何会话；
    命令名在不碰名单里的、systemd 单元里的、登记的会话进程不碰。弄坏开关 runaway-process-not-stopped、runaway-unowned-not-stopped、runaway-protected-stopped 下这一格红。"""
    spinning = subprocess.Popen(["sleep", "600"])
    orphan = subprocess.Popen(["sleep", "600"])
    named, in_unit, session_process = (subprocess.Popen(["sleep", "600"]) for _ in range(3))
    world.register_session("selftest-session-keeper", "", "keeper", pid=session_process.pid)
    listing = os.path.join(world.root, "process-list-stop")
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-runaway-stop")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_PROCESS_LIST": listing, "TASK_SERVER_SELFTEST_RUNAWAY_RENOTIFY_SECONDS": "0.3"})
    head = "task-server：任务之外的进程空转 已停"
    try:
        world.running(world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "runaway-stop", release], extra=inbox.environment()))
        with open(listing, "w", encoding="utf-8") as handle:
            handle.write(f"  {spinning.pid} {os.getpid()} 7200 7100 R 0::/user.slice/session-3.scope spinning\n  {orphan.pid} 1 7200 7100 R 0::/user.slice/session-3.scope orphan\n"
                         f"  {named.pid} 1 7200 7100 R 0::/user.slice/session-3.scope claude\n  {in_unit.pid} 1 7200 7100 R 0::/user.slice/user-1000.slice/user@1000.service/app.slice/a.service daemon\n"
                         f"  {session_process.pid} 1 7200 7100 R 0::/user.slice/session-3.scope registered\n")
        inbox.wait_for(world, head)
        time.sleep(0.5)
        stopped = inbox.headed(head)
        reported = inbox.headed(f"task-server：任务之外的进程空转 本机进程 {spinning.pid}")
        spinning_gone, orphan_gone = spinning.poll() is not None, orphan.poll() is not None
        kept_alive = [name for name, process in (("不碰名单", named), ("systemd 单元", in_unit), ("登记的会话进程", session_process)) if process.poll() is None]
        unowned_messages = [message for message in inbox.headed("task-server：任务之外的进程空转") if str(orphan.pid) in message["text"]]
    finally:
        open(release, "w").close()
        for process in (spinning, orphan, named, in_unit, session_process):
            process.kill()
            process.wait()
    problems = []
    if len(stopped) != 1 or str(spinning.pid) not in stopped[0]["text"] or "回读已不在" not in stopped[0]["text"]:
        problems.append(f"应当恰好一条「已停」点名进程 {spinning.pid}、写回读已不在，实际 {[message['text'][:200] for message in stopped]}")
    if len(reported) != 2:
        problems.append(f"停之前应当恰好报两次，实际 {len(reported)} 次")
    if not spinning_gone:
        problems.append("所属会话认得出的那个进程应当被服务停掉，实际还在")
    if not orphan_gone:
        problems.append("认不出发起者的那个进程应当被直接终止，实际还在")
    if unowned_messages:
        problems.append(f"认不出发起者的不该发给任何会话，实际收到 {len(unowned_messages)} 条")
    if len(kept_alive) != 3:
        problems.append(f"不碰的三类都该还在，实际还在的只有 {kept_alive}")
    if problems:
        raise SelftestFailure("报满次数之后按号停：" + "；".join(problems), world.server_log_lines())
    return "认得出发起会话的报两次之后被按号停掉、那个会话收到一条；认不出的直接终止、不发消息；不碰的三类都还在"


def cell_margin_measured_per_host(world):
    """余量按台现量（task_server.py 文件头 ②「余量」；host_capacity、host_margin）：两台 slice 总上限都是 50G，记录峰值 48G 的那一条上限与记账都是 49152M
    （近五次最大，不乘 1.5），它只在余量不大于 2G 的台上装得下。只有本机、本机余量 1G（可用上限 49G）：在本机起、跑完；
    本机余量 3G（可用上限 47G，峰值本身就装不下）、第二台余量 1G（49G）：派第二台、跑完。
    弄坏开关 margin-local-for-both 下第二台也减本机的 3G、两台都比峰值小，照收到的 47G 起跑（上限不是 49152M）；booked-up-to-slice-total 下两台都不减余量、本机装得下、起在本机，这一格红。"""
    key = "selftest-margin-per-host"
    world.seed_peak(key, PER_HOST_PEAK_BYTES)
    world.resources(local=(100, 50), local_margin=PER_HOST_SMALL_MARGIN_GIBIBYTES)
    world.start_server()
    problems = []
    scenarios = (("只有本机、本机余量 1G", "local", {"local": (100, 50), "local_margin": PER_HOST_SMALL_MARGIN_GIBIBYTES}),
                 ("本机余量 3G、第二台余量 1G", "peer", {"local": (100, 50), "peer": (100, 50), "peer_cap_gibibytes": 50,
                                                      "local_margin": PER_HOST_LARGE_MARGIN_GIBIBYTES, "peer_margin": PER_HOST_SMALL_MARGIN_GIBIBYTES}))
    for what, host, scenario in scenarios:
        world.resources(**scenario)
        summary = world.finished(world.submit(["bash", "-c", "exit 0", f"margin-per-host-{host}"], extra={lib.KEY_VARIABLE: key}), timeout=PLACEMENT_WAIT_SECONDS)
        attempt = first_attempt(summary)
        if (attempt.get("host") != host or attempt.get("memory_cap_text") != PER_HOST_CAP_TEXT or attempt.get("booked") != PER_HOST_PEAK_BYTES
                or summary["task"].get("outcome") != "succeeded"):
            problems.append(f"{what}：应当在{lib.HOST_NAMES.get(host, host)}起、上限与记账 {PER_HOST_CAP_TEXT}、跑完，"
                            f"实际台 {attempt.get('host')}、记账 {attempt.get('booked')} 字节、{describe(summary)}")
    if problems:
        raise SelftestFailure("余量按台：" + "；".join(problems), world.server_log_lines())
    return f"峰值 48G 的那一条：余量 1G 的本机起得来；本机余量 3G 装不下时派余量 1G 的第二台，上限都是 {PER_HOST_CAP_TEXT}，两台各用各的余量"


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


# peer-booked-like-local：峰值 20480M（20G），上限与记账照它；在跑的每条此刻回收不掉 1G；两台都是 slice 50G、余量 1G、空闲 40G
BOTH_HOSTS_PEAK_BYTES = 20480 * MEBIBYTE
BOTH_HOSTS_CAP_TEXT = "20480M"
BOTH_HOSTS_USED_BYTES = 19 * GIBIBYTE   # 在跑的六条各自此刻回收不掉的：记账照它记 19G（task_server.py 的 running_booking）
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
    """第二台与本机同一段（task_server.py 文件头 ②「内存」的在跑记账）：两台各三条在跑、各上限 20G、各自此刻回收不掉 19G（两台都照那一台报告里 label
    对得上的那个 scope；记账 = min(上限, 用量) = 19G）；两台都是 slice 50G、余量 1G、空闲 40G、进程用 3G：每台待涨量 0，第四条 20G
    两台都放不下（每台记账之和 57G 加它超过总上限减余量 49G），排两轮之后照排着、排队原因两台都写。放行之后第四条照起、跑完。
    挡住第四条的是公式（记账之和），不经待涨量：弄坏开关 server-pending-ignored 下判法不变，这一格不配它。"""
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
            problems.append(f"两台各三条 20G 在跑（各用 19G）、slice 50G、余量 1G：第四条 20G 两台都放不下，应当排着、排队原因两台都写；"
                            f"实际状态 {waiting['task'].get('state')}、{describe(waiting)}、排队原因「{reason[:300]}」")
    finally:
        open(os.path.join(world.repo, BOTH_HOSTS_RELEASE), "w").close()
    summaries = [world.finished(task_id, timeout=120) for task_id in [*started.get("local", []), *started.get("peer", []), fourth]] if not problems else []
    if any(summary["task"].get("outcome") != "succeeded" for summary in summaries):
        problems.append(f"放行之后七条都应当跑完，实际 {[describe(summary) for summary in summaries]}")
    if problems:
        raise SelftestFailure("两台同一个口径：" + "；".join(problems), world.server_log_lines())
    return "两台各三条 20G 在跑、各用 19G：第二台照报告里的 scope 记账（实测 19G），同本机，第四条 20G 两台都排着；放行之后跑完"


# peer-scope-missing-booked-at-cap：记录的峰值 5G；内存上限的下限（TASK_SERVER_SELFTEST_MEMORY_FLOOR_BYTES）15G 把上限与新起的记账抬到 15G（15360M）；
# 报告里那条 scope 此刻回收不掉 3G
MISSING_SCOPE_PEAK_BYTES = 5 * GIBIBYTE
MISSING_SCOPE_FLOOR_BYTES = 15 * GIBIBYTE
MISSING_SCOPE_CAP_TEXT = "15360M"
MISSING_SCOPE_USED_BYTES = 3 * GIBIBYTE
MISSING_SCOPE_RELEASE = "release-missing-scope"


def cell_peer_scope_missing_booked_at_cap(world):
    """第二台报告里缺某条在跑尝试的 scope（还没起来、读不了）：那一条记整份上限，不记 0；有 scope 的照实测记、不记上限（task_server.py 的 server_pending、
    running_booking）。只派第二台（本机总上限 1G 不比余量 1G 大）。三条同一把键：记录的峰值 5G，内存上限的下限 15G 把上限与新起的记账抬到 15G（实测比上限小，才分得出记实测与记上限）。
    第二台 slice 50G、余量 1G、空闲 20G，两条在跑各上限 15G：报告里只有其中一条的 scope（回收不掉 3G、进程用 3G；记实测 3G）时待涨量 0 + 15G、空闲剩 3G，
    第三条（要的空闲 6G：预测 5G 加余量 1G；记账 15G）放不下、排两轮照排着（缺的记 0 的写法下待涨量 2G、空闲剩 18G，放得下）；
    报告补上另一条的 scope（也是 3G、进程共用 6G）之后两条各记 5G、待涨量 2G + 2G、空闲剩 16G、slice 减余量减待涨量剩 39G、公式剩 39G：第三条放得下、起在第二台
    （有 scope 的也记上限的写法下待涨量 12G + 12G、空闲剩不下，照样放不下；弄坏开关 running-booked-at-cap 下就是这样）。
    弄坏开关 server-pending-ignored 下在跑的不记待涨量：头一段第三条就起了。"""
    world.seed_peak("selftest-missing-scope", MISSING_SCOPE_PEAK_BYTES)
    world.resources(local=(1, 1), local_margin=SELFTEST_TIGHT_MARGIN_GIBIBYTES)
    world.start_server(extra={lib.MEMORY_FLOOR_VARIABLE: str(MISSING_SCOPE_FLOOR_BYTES)})
    command = ["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "missing-scope", MISSING_SCOPE_RELEASE, f"fake-peer-wait={MISSING_SCOPE_RELEASE}"]
    problems = []
    third = None
    try:
        started = occupy_hosts(world, command, "selftest-missing-scope", ["peer"] * 2)["peer"]
        labels = [first_attempt(world.summary(task_id)).get("label") for task_id in started]
        caps = [first_attempt(world.summary(task_id)).get("memory_cap_text") for task_id in started]
        if caps != [MISSING_SCOPE_CAP_TEXT] * 2 or not all(labels):
            problems.append(f"占着第二台的两条应当各上限 {MISSING_SCOPE_CAP_TEXT}、都有 label，实际上限 {caps}、label {labels}")
        scenario = {"local": (1, 1), "local_margin": SELFTEST_TIGHT_MARGIN_GIBIBYTES, "peer": (20, 50), "peer_cap_gibibytes": 50,
                    "peer_margin": SELFTEST_TIGHT_MARGIN_GIBIBYTES}
        world.resources(**scenario, peer_slice_used=3, peer_scopes={labels[0]: MISSING_SCOPE_USED_BYTES})
        third = world.submit(["bash", "-c", "exit 0", "missing-scope-third", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-missing-scope"}, cwd=world.repo)
        world.next_pass()
        world.next_pass()
        waiting = world.summary(third)
        if waiting["task"].get("state") != "queued" or waiting["attempts"]:
            problems.append(f"报告里缺一条在跑的 scope：那一条应当记整份上限 15G（有 scope 的那一条记实测 3G），第三条放不下、排着；实际状态 {waiting['task'].get('state')}、{describe(waiting)}、"
                            f"排队原因「{(waiting.get('waiting_for') or '')[:300]}」")
        world.resources(**scenario, peer_slice_used=6, peer_scopes={label: MISSING_SCOPE_USED_BYTES for label in labels})
        placed = world.wait_for(third, lambda found: bool(found["attempts"]) or found["task"].get("state") != "queued", "照补上的 scope 起来")
        if first_attempt(placed).get("host") != "peer":
            problems.append(f"报告补上那一条的 scope 之后两条各记实测 3G、不记上限，第三条应当起在第二台（待涨量 0），实际 {describe(placed)}、排队原因「{(placed.get('waiting_for') or '')[:300]}」")
    finally:
        open(os.path.join(world.repo, MISSING_SCOPE_RELEASE), "w").close()
    summaries = [world.finished(task_id, timeout=120) for task_id in [*started, third]] if not problems else []
    if any(summary["task"].get("outcome") != "succeeded" for summary in summaries):
        problems.append(f"放行之后三条都应当跑完，实际 {[describe(summary) for summary in summaries]}")
    if problems:
        raise SelftestFailure("第二台缺 scope 的记整份上限：" + "；".join(problems), world.server_log_lines())
    return "第二台报告里缺一条在跑的 scope 时那一条记整份上限 15G、第三条排着；补上之后两条各记实测 3G（不记上限）、第三条起在第二台"


# two-hosts-booked-within-capacity：三种各四条（上限与记账等于记录的峰值：15G、9G，没记录的记 4G，合计 112G）交给两台各 slice 50G、余量 1G（各能记 49G）
INVARIANT_KINDS = (("selftest-invariant-large", 15 * GIBIBYTE), ("selftest-invariant-medium", 9 * GIBIBYTE), ("selftest-invariant-small", None))
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
    这一格封顶的是公式（verdict_within_the_formula：总上限减余量减在跑的记账之和），不经待涨量（server_pending）：报告的空闲（200G）与 slice 余量（50G）不随在跑的涨落，
    弄坏开关 server-pending-ignored 下上限之和照样不超过 49G，这个开关不在这一格判红（它红在待涨量与均衡挑台那几格，开关表在 task_server_lib.py 文件头）。"""
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


def cell_selftest_variables_refused(world):
    outside = {name: value for name, value in world.client_environment.items() if name != lib.SELFTEST_MARK_VARIABLE}
    outside[lib.BREAK_VARIABLE] = "exit-code-dropped"
    for script in (lib.CLIENT_SCRIPT, lib.CONTROL_SCRIPT, lib.SERVER_SCRIPT):
        # 参数从表里取（折不出）：闭包把三份脚本整份连进来，各格经 World.client 起它们时走的是 bash 的 "$@"，闭包不沿那一条认脚本
        arguments = {lib.CLIENT_SCRIPT: ["status"], lib.CONTROL_SCRIPT: ["check"], lib.SERVER_SCRIPT: ["serve"]}[script]
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
    ("timed-out", ["bash", "-c", "exit 0", "fake-wrapper-exits=253"], {}),
    ("run-condition-unmet", ["bash", "-c", "echo selftest-reason-unmet; exit 78"], {}),
    ("not-run", ["bash", "-c", "exit 77"], {}),
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
    log_text = "MULTI_HOST_RUN_PEER_CPU poll=1 usage_usec=0 read=no\nselftest-last-line\n"
    for outcome in lib.OUTCOME_TYPES:
        task = {"id": 1, "outcome": outcome, "exit_code": lib.final_exit_code(outcome, 3), "note": f"自证造的说明（{outcome}）", "maximum_host": "local",
                "maximum_cap_text": "46080M", "outcome_counts": {outcome: 2}}
        attempt = {"number": 1, "host": "local", "memory_cap_text": "2048M", "memory_peak": 2 * GIBIBYTE, "unit": "singlefs-task-selftest.service",
                   "squeezed_by": "任务 2（上限 4096M）"}
        problems += reason_problems(lib.reason_line(task, [attempt], log_text), outcome)
    world.resources(local=(100, 100))
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
    """TS-05：只改库里一行尝试的 task_id 列（正文与签名不动），把任务 A 的尝试挂到任务 B 名下：B 的记录读回时那一行校验不过（对账判出之后照正文挂回 A、记 ended、单元由对账停），
    取消 B 回不了 stopped；只改一个子任务的 task_id 列把它挂到另一件分片名下：取消那一件时对账还没判出的整件回 409、它自己的片不停，
    判出之后那一行照正文挂回原来那一件（核得过、记 record-tampered、挂过来那一片的单元由对账停），取消只停那一件自己的片。"""
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
        if cancelled.returncode == 0:
            problems.append(f"尝试的 task_id 列改到任务 {other} 名下：取消 {other} 应当不退 0，实际退 {cancelled.returncode}")
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while patience.waiting() and (world.unit_active(owner_unit) or world.unit_active(other_unit)):
            time.sleep(POLL_SECONDS)
        if world.unit_active(owner_unit) or world.unit_active(other_unit):
            problems.append(f"关联列被改的两次尝试的单元应当由对账停掉，实际 {owner} 的单元{'在跑' if world.unit_active(owner_unit) else '停了'}、{other} 的单元{'在跑' if world.unit_active(other_unit) else '停了'}")
        if any(attempt.get("id") == owner_summary["attempts"][0]["id"] and attempt.get("valid") is not False for attempt in listed):
            problems.append(f"任务 {other} 名下读回的那一行（正文是任务 {owner} 的）应当校验不过（或已被对账挂回 {owner}、不在 {other} 名下），"
                            f"实际 {[(attempt.get('id'), attempt.get('valid')) for attempt in listed]}")
        first = world.submit(world.write_spec("spec-association-first.json", {"shards": [{"block": releases[2]}]}), cwd=world.repo)
        second = world.submit(world.write_spec("spec-association-second.json", {"shards": [{"block": releases[3]}]}), cwd=world.repo)
        running = lambda summary: len(shard_children(summary)) == 1 and any(attempt.get("state") == "running" for child in shard_children(summary).values()
                                                                             for attempt in child["attempts"])
        first_child = list(shard_children(world.wait_for(first, running, "那一片跑起来")).values())[0]
        second_child = list(shard_children(world.wait_for(second, running, "那一片跑起来")).values())[0]
        raw_column_update(world, "tasks", second, first_child["task"]["id"])
        cancelled = world.client(["cancel", str(second)])
        units = [first_child["attempts"][-1].get("unit", ""), second_child["attempts"][-1].get("unit", "")]
        _, repaired, column = verified_row(world, "tasks", first_child["task"]["id"])
        if (cancelled.returncode == 0 and not (repaired and column == first)) or (cancelled.returncode != 0 and not world.unit_active(units[1])):
            problems.append(f"子任务 {first_child['task']['id']} 的 task_id 列改到分片的任务 {second} 名下：没判出时整件不退 0、"
                            f"它自己的片不停，判出之后那一行挂回 {first}、核得过，实际退 {cancelled.returncode}、在跑的单元 {[unit for unit in units if world.unit_active(unit)]}、"
                            f"那一行核{'得过' if repaired else '不过'}、关联列 {column}")
    finally:
        for release in releases:
            open(release, "w").close()
    if problems:
        raise SelftestFailure("关联列：" + "；".join(problems), world.server_log_lines())
    return "只改尝试或子任务的 task_id 列：挂过去的那一行读回校验不过，取消那一件回不了 stopped，单元由对账停"


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
    （近五次最大 3G，3072M，不乘 1.5）；不打这一行的照旧记「没量到」（null）。"""
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
    if next_one.get("memory_cap_text") != "3072M":
        problems.append(f"同一命令下一次应当照第二台量到的 3G 定上限 3072M，实际 {next_one.get('memory_cap_text')}")
    if problems:
        raise SelftestFailure("第二台的内存峰值：" + "；".join(problems), world.server_log_lines())
    return "第二台打了峰值那一行的记下 3G、下一次上限 3072M；不打的记 null"


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
    包着的照原命令算键、--server-binary 的路径不进键（两次包着的给的路径不同，命令键相同、第二次照第一次量到的 300M 定上限 300M）；
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
    if caps[1:] != [UNRECORDED_DEFAULT_PINNED_TEXT, "300M"] or any(summary["task"].get("outcome") != "succeeded" for summary in (plain, first, second)):
        problems.append(f"包着的第一次没有自己的记录应当取 {UNRECORDED_DEFAULT_PINNED_TEXT}（不用不包的那次量到的 300M）、第二次照第一次的 300M 取 300M、都跑完，"
                        f"实际上限 {caps}、{describe(first)}、{describe(second)}")
    if not all(world.calls_of(summary["task"]["id"], "temporary-service") for summary in (first, second)):
        problems.append("包装本身应当照旧整条执行（假包装记一行），实际没起")
    if problems:
        raise SelftestFailure("临时 prune-server 的包装：" + "；".join(problems), world.server_log_lines())
    return (f"包着的照原命令算键、服务路径不进键、前面接「{lib.TEMPORARY_SERVICE_KEY_MARK}」：与不包的不共用记录（第一次 {UNRECORDED_DEFAULT_PINNED_TEXT}、"
            "第二次照第一次的 300M 取 300M），包装照旧整条执行")


WRAPPED_SELFTEST_SCRIPT = "scripts/selftest-wrapped-check.sh"   # temporary-service-heavy-refused：包装里照收的那条普通 --selftest 起的假脚本（相对假仓根）


def cell_temporary_service_heavy_refused(world):
    """④ 临时 prune-server 包装里会编的提交时拒（task_server.py 文件头 ②「提交时拒」④）：包装里套 cargo test（任务 4229 那一条的样子）
    当场拒（task submit 退 3、不给任务号，说明点名会编的 cargo、写明包装只给脚本自证用，出路写去掉包装重交），
    没进队；包装里普通的 --selftest 照收、包装照旧整条执行、跑完。"""
    world.resources(local=(30, 30))
    world.start_server()
    write_executable(os.path.join(world.repo, WRAPPED_SELFTEST_SCRIPT), "#!/usr/bin/env bash\nexit 0\n")

    def wrapped(*inner):
        return ["python3", f"scripts/prune-server/{lib.TEMPORARY_SERVICE_SCRIPT}", lib.TEMPORARY_SERVICE_FLAG, "--server-binary", "/usr/bin/true", "--", *inner]

    problems = []
    refused_cases = (("包装里套 cargo test", wrapped("cargo", "test", "--release", "-p", "singlefs-verification", "--test", "selftest-target",
                                                 "fake-wrapper-exits=0"), "会编的 cargo"),)
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
        raise SelftestFailure("临时 prune-server 包装里会编的提交时拒：" + "；".join(problems), world.server_log_lines())
    return (f"包装里套 cargo test 提交时退 3、说明点名会编的 cargo、出路写去掉包装重交；"
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


NFS_PROBE_READ_DELAY_SECONDS = 8   # nfs-probe-concurrent-runs：头一趟 control check 的第二台读探针那一步先睡这么久，另一趟在这段时间里整个跑完（一趟 control check 一两秒，推的）


def cell_nfs_probe_concurrent_runs(world):
    """两趟 control check --probe 交错跑两向探针（task_server_lib.py 的 nfs_probe）：头一趟写好本机探针、第二台读它之前先睡（假 ssh 的 fake-ssh-read-delay），
    另一趟在这段时间里整个跑完、收尾删自己的探针；头一趟醒来照样读得到它自己写的那一份，两趟都报 NFS 共享过。"""
    world.start_server()
    delay = os.path.join(world.world, "fake-ssh-read-delay")
    with open(delay, "w", encoding="utf-8") as handle:
        handle.write(f"{NFS_PROBE_READ_DELAY_SECONDS}\n")
    earlier = world.control(["check", "--probe"], background=True)
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    while not os.path.exists(delay + ".taken") and earlier.poll() is None and patience.waiting():
        time.sleep(POLL_SECONDS)
    later = world.control(["check", "--probe"])
    try:
        earlier_output, earlier_error = world.communicate(earlier, STATE_TIMEOUT_SECONDS)
    except subprocess.TimeoutExpired:
        earlier_output, earlier_error = world.stop_client(earlier)
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
    """跑 0.1 秒就退的命令：按轮一次都采不到（假 scope 是空目录），结局的 memory_peak 照内存包装退出时交的那一个记（attempt_runner.py 文件头 ⑥，
    假包装照 fake-wrapper-exit-peak= 往服务给的 RUN_WITH_MEMORY_CAP_PEAK_OUTPUT 写），不是 null；两样都没有的照旧 null。
    后半段换上进程内存 200M 的假 scope 重起服务：按轮采到了的与退出时交的（扣了页缓存）取大的（attempt_runner.py 文件头 ⑥；一轮之间冒出来的峰值在退出时交的那一个里）。
    弄坏开关 exit-peak-only-when-unsampled 下按轮采到了就不看它，「400M 比 200M 大」那一例记 200M，这一格红。"""
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
                                  ("退出时交的 400M 比按轮的 200M 大的（照大的）", larger, 2 * SHORT_SAMPLED_PEAK_BYTES),
                                  ("退出时交的 100M 比按轮的 200M 小的（照大的）", smaller, SHORT_SAMPLED_PEAK_BYTES)):
        attempt = first_attempt(summary)
        if summary["task"].get("outcome") != "succeeded" or attempt.get("memory_peak", "缺这一栏") != wanted:
            problems.append(f"{name}应当跑完、记 memory_peak {wanted}，实际 {describe(summary)}、记 {attempt.get('memory_peak', '缺这一栏')}")
    if problems:
        raise SelftestFailure("跑得短的命令的内存峰值：" + "；".join(problems), world.server_log_lines())
    return "跑 0.1 秒就退、按轮采不到的记了退出时交的 123M；两样都没有的记 null；按轮采到了的与退出时交的取大的（400M、200M）"


def cell_scene_sessionless_swept(world):
    """没有会话的现场（task_server_lib.py 文件头「现场」；task_server.py 的 scene_due）：人手 --manual 交的退非 0、留了现场，不起 wait：结局满保留期（快档 1 秒）之后服务照样删。
    弄坏开关 scene-sessionless-kept 下要等取走、一直不删，这一格红。"""
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", "exit 0", f"fake-wrapper-scene={SCENE_COMMAND_EXIT}"], session=None))
    scene, _ = scene_of(world, summary)
    if summary["task"].get("session") or not scene:
        raise SelftestFailure(f"人手交的应当记无会话、有现场目录，实际会话 {summary['task'].get('session')!r}、现场「{scene}」", world.server_log_lines())
    patience = world.patience(SCENE_GONE_SECONDS)
    while os.path.lexists(scene) and patience.waiting():
        time.sleep(POLL_SECONDS)
    if os.path.lexists(scene):
        raise SelftestFailure(f"没有会话的任务结局满保留期之后现场 {scene} 应当删了，实际还在", world.server_log_lines())
    return "人手交的没起 wait，现场照结局时刻满保留期删了"


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


def start_three_shards(world, name, shards, extra=None):
    """三片的一件，第 0、1 片同时在本机跑、第 2 片排着：本机 slice 20G、第二台用不了，第 2 片记过的峰值 30G 只有第二台放得下，排着不判永远放不下
    （同 down-host-waits-not-never）。shards 是规格里三片的写法，extra 是这一格服务的设置；等到这个样子交父任务号与 {片号: 子任务号}。"""
    for index, peak in ((0, 2), (1, 2), (2, 30)):
        world.seed_peak(lib.shard_child_key(QUEUED_SHARD_KEY, "shard", index, 3), peak * GIBIBYTE, shard_count=3)
    world.resources(local=(50, 20), peer_down="自证：第二台此刻用不了")
    world.start_server(extra=extra)
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
    """一片 command-failed、一片在跑、一片在排（task_server_lib.py 文件头「分片」的「有一步最终没成」）：兄弟片照排、照跑到底，结局照收：
    第 0 片没成之后再排两轮，父任务没结局、第 1 片照跑、第 2 片照排着不撤；第 2 片记过的峰值 30G 只有第二台放得下（start_three_shards），
    这时把本机换成 slice 50G，它在本机起、跑完；第 1 片放行跑完；都收齐之后父任务才记第 0 片的 command-failed、wait 末行点名 shard=0/3，不合并。
    弄坏开关 shard-siblings-abandoned 下父任务当场收尾、在排的那片撤掉，这一格红。"""
    releases = {name: os.path.join(world.root, f"release-failure-{name}") for name in ("failing", "running")}
    task_id, _ = start_three_shards(world, "failure-waits", [{"block": releases["failing"], "exits": [3]}, {"block": releases["running"]}, {}])
    problems = []
    try:
        open(releases["failing"], "w").close()
        world.wait_for(task_id, lambda summary: shard_outcomes(summary).get(0) == "command-failed", "第 0 片记 command-failed")
        world.next_pass()
        world.next_pass()
        middle = world.summary(task_id)
        if middle["task"].get("state") == "finished" or 1 not in running_shards(middle) or queued_shards(middle) != [2]:
            problems.append(f"第 0 片没成、第 1 片还在跑时：父任务应当还没结局、第 1 片照跑、第 2 片照排着不撤，实际父任务 {middle['task'].get('state')}/{middle['task'].get('outcome')}、"
                            f"在跑的片 {running_shards(middle)}、各片 {shard_outcomes(middle)}")
        world.resources(local=(50, 50), peer_down="自证：第二台此刻用不了")   # 第 2 片（30G）在本机放得下了
    finally:
        open(releases["running"], "w").close()
    finished = world.finished(task_id, timeout=120)
    code, last = wait_last_line(world, task_id)
    outcomes = shard_outcomes(finished)
    third_runs = world.sharded_calls("shard", index=2)
    if (finished["task"].get("outcome") != "command-failed" or code != 3 or " shard=0/3" not in last
            or outcomes != {0: "command-failed", 1: "succeeded", 2: "succeeded"} or not third_runs
            or "merge" in child_roles(finished) or world.sharded_calls("merge")):
        problems.append(f"收齐之后：整件应当是第 0 片的 command-failed、wait 退 3、末行点名 shard=0/3，第 1、2 片都 succeeded（第 2 片照排照跑），不合并；"
                        f"实际 {describe(finished)}、wait 退 {code}、末行「{last}」、各片 {outcomes}、第 2 片跑的记录 {third_runs}、子任务 {child_roles(finished)}、"
                        f"合并的记录 {world.sharded_calls('merge')}")
    if problems:
        raise SelftestFailure("一片没成：" + "；".join(problems), world.server_log_lines())
    return "一片没成：兄弟片照排照跑、结局照收，收齐之后整件才记那一片的 command-failed、不合并"


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


ATTENTION_QUEUE_SECONDS = "2"        # attention-queued-too-long：排队过久的提醒门槛（秒；TASK_SERVER_SELFTEST_QUEUE_ATTENTION_SECONDS）
ATTENTION_ONCE_SECONDS = 6           # attention-reported-once：排满 6 秒提醒一回、排满 12 秒再提醒一回（每翻一倍一回）
ATTENTION_ONCE_QUIET_SECONDS = 2.5   # attention-reported-once：6 秒那一回叫过之后再起 wait，这么久里不该再叫（离 12 秒那一回还有 6 秒）
ATTENTION_WAIT_SECONDS = 60          # 各提醒格里 wait 至多等这么久（弄坏开关下它一直等，等满就判红）
ATTENTION_QUIET_SECONDS = "3"        # attention-quiet-busy-log：日志长期不涨的提醒门槛（秒；TASK_SERVER_SELFTEST_QUIET_ATTENTION_SECONDS），比卡住阈值 2 秒长
ATTENTION_IO_WAIT_SECONDS = "3"      # io-wait-local-not-stalled：卡在 IO 上的提醒门槛（秒；TASK_SERVER_SELFTEST_IO_WAIT_ATTENTION_SECONDS），比卡住阈值 2 秒长
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
    """io-wait-local-not-stalled 用：几回 wait 的输出里不该有 kind=quiet-log-busy 的那一行（服务不是因为 CPU 在涨或读不到才没停它）。"""
    lines = [line for completed in waited for line in attention_lines(completed) if attention_fields(line).get("kind") == "quiet-log-busy"]
    return [f"{which}不该报 quiet-log-busy，实际 wait 打了「{lines[0][:300]}」"] if lines else []


def cell_attention_queued_too_long(world):
    """排队过久（task_server.py 文件头 ①「要处置的提醒」）：第二台用不了、只有它放得下的命令一直排着（同 down-host-waits-not-never），排满提醒门槛（自证里 2 秒）
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
    return f"排满 {ATTENTION_QUEUE_SECONDS} 秒：wait 打 {lib.ATTENTION_MARK} kind=queued-too-long（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，任务照旧在排，第二台回来后跑完"


def cell_attention_quiet_busy_log(world):
    """日志长期不涨而服务没停它（task_server.py 的 quiet_log_items；任务 7723 日志 37702 秒没涨，服务一直不停也没人叫）：一条只打一行
    就不出声、跑到放行为止，假 scope 的 CPU 一直在涨（卡住阈值 2 秒：CPU 在涨不停），此刻内存读不到（不判贴着上限）；日志满提醒门槛（自证里 3 秒）没涨，
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
    return f"日志满 {ATTENTION_QUIET_SECONDS} 秒没涨、CPU 在涨：wait 打 {lib.ATTENTION_MARK} kind=quiet-log-busy（依据「{fields.get('basis')}」）、退 {lib.ATTENTION_EXIT}，任务照旧在跑，放行后跑完"


def cell_io_wait_local_not_stalled(world):
    """卡在 IO 上（task_server.py 文件头 ①「卡在 IO 上」；任务 8608、8609、8611、8619 在等 NFS 时被当成卡住停掉）：本机一条只打一行就不出声、跑到放行为止，
    假 scope 的 CPU 一直不涨（卡住阈值 2 秒），假 scope 的 cgroup.procs 点名一个进程号，假 /proc（TASK_SERVER_SELFTEST_PROC_ROOT）下它的 stat 是 D 状态：
    服务不判 stalled、不停；日志没涨的每一轮都有 D 状态的进程满提醒门槛（自证里 3 秒），wait 打一行 kind=io-wait、依据写 D 状态的进程数与 /proc/pressure/io 的 avg60
    （读不到照写读不到）、退 ATTENTION_EXIT（CPU 不涨先报的疑似卡住照报，再起 wait 接着等）；任务照旧在跑，放行之后跑完。
    没有 D 状态的照旧停 stalled 由 stalled-attempt-stopped 那一格判（同一段判法：假 /proc 不换，那一条的真进程都在睡、不在 D 状态）。
    日志长期不涨的提醒门槛同样压到 3 秒（ATTENTION_QUIET_SECONDS）：按 D 状态免停的不报 quiet-log-busy（task_server.py 的 quiet_log_items），各回 wait 都不该打那一种。
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
    return (f"本机日志与 CPU 都不涨而每一轮都有进程在 D 状态：不停，满 {ATTENTION_IO_WAIT_SECONDS} 秒 wait 打 {lib.ATTENTION_MARK} kind=io-wait"
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


PEER_HEALTH_HEAD = "task-server：服务健康 NFS 共享判不过"
PEER_SLOW_LOG_TEXT = "第二台慢（超时、复查通了）"


def cell_peer_health_slow_rechecked(world):
    """第二台一次超时先当慢（task_server_lib.py 文件头「NFS 共享」的挂载；三个会话各报过一次 30 秒超时的「用不了」）：ssh 时限压到 1 秒、复查前隔 0.2 秒、复查期 0.5 秒（一回复查就到期）；
    假 ssh 头一回复核挂载那一句睡着（fake-ssh-hang-once），复查照常回 nfs：服务日志记一行「第二台慢（超时、复查通了）」，会话收不到服务健康；
    再让每一回都睡着（fake-ssh-hang）：复查期满都超时，会话收到恰好一条「服务健康 NFS 共享判不过」，正文写「仍不通」。
    弄坏开关 peer-health-single-timeout-reported 下头一回超时就报、服务日志没有那一行，这一格红。"""
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-peer-health")
    once, hang = os.path.join(world.world, "fake-ssh-hang-once"), os.path.join(world.world, "fake-ssh-hang")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_NFS_SSH_SECONDS": "1", "TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS": "0.2", "TASK_SERVER_SELFTEST_NFS_SLOW_GRACE_SECONDS": "0.5"})
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
        if found and "仍不通" not in found[0]["text"]:
            problems.append(f"复查期满都不通的那一条应当写「仍不通」（连不上），实际「{found[0]['text'][:300]}」")
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(hang)
        open(release, "w").close()
    if problems:
        raise SelftestFailure("第二台一次超时先当慢：" + "；".join(problems), world.server_log_lines())
    return f"一次超时、复查回了：服务日志记「{PEER_SLOW_LOG_TEXT}」、不推；复查期满都不通：推恰好一条服务健康，写仍不通"


def cell_peer_health_slow_within_recheck_period(world):
    """第二台连着慢几回、复查期里回了的只记慢、不群发（task_server_lib.py 的 peer_shell_rechecked；两台的盘被写满那一阵握手超过 30 秒、几分钟后又回来，服务判成连不上、
    给所有有任务的会话群发服务健康）：ssh 时限压到 1 秒、复查前隔 0.2 秒、复查期 3 秒；假 ssh 头两回复核挂载那一句睡着（fake-ssh-hang-count 写 2）、第三回照常回：
    服务日志记「第二台慢（超时、复查通了）」且写「复查到第 2 回回了」、/health 的 nfs 一栏带 slow_last、会话收不到服务健康；之后每一回都睡着（fake-ssh-hang）：
    复查期满仍不通，会话收到恰好一条「服务健康 NFS 共享判不过」。
    弄坏开关 peer-recheck-once-only 下复查期里只复查一回，头两回超时就报，会话收到服务健康、服务日志没有慢的那一行，这一格红。"""
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-peer-health-period")
    counted, hang = os.path.join(world.world, "fake-ssh-hang-count"), os.path.join(world.world, "fake-ssh-hang")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_NFS_SSH_SECONDS": "1", "TASK_SERVER_SELFTEST_NFS_RECHECK_PAUSE_SECONDS": "0.2", "TASK_SERVER_SELFTEST_NFS_SLOW_GRACE_SECONDS": "3"})
    problems = []
    try:
        task_id = world.submit(["bash", "-c", HOLD_UNTIL_RELEASED_SCRIPT, "peer-health-period", release], extra=inbox.environment())
        world.running(task_id)
        with open(counted, "w", encoding="utf-8") as handle:
            handle.write("2\n")
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while "复查到第 2 回回了" not in server_output_text(world) and patience.waiting():
            time.sleep(POLL_SECONDS)
        time.sleep(INBOX_QUIET_SECONDS)
        if PEER_SLOW_LOG_TEXT not in server_output_text(world) or "复查到第 2 回回了" not in server_output_text(world):
            problems.append(f"头两回超时、第三回回了的应当在服务日志记「{PEER_SLOW_LOG_TEXT}」并写「复查到第 2 回回了」，实际 fake-ssh-hang-count 剩 {open(counted).read().strip()}")
        early = inbox.headed(PEER_HEALTH_HEAD)
        if early:
            problems.append(f"头两回超时、第三回回了的不该推服务健康，实际收到 {len(early)} 条：「{early[0]['text'][:300]}」")
        slow_last = health_until(world, lambda health: bool((health.get("nfs") or {}).get("slow_last")), STATE_TIMEOUT_SECONDS).get("nfs", {}).get("slow_last")
        if not slow_last or "复查到第 2 回回了" not in " ".join(slow_last.get("slow") or []):
            problems.append(f"/health 的 nfs 一栏应当带 slow_last 并写「复查到第 2 回回了」，实际 {slow_last}")
        open(hang, "w").close()
        found = inbox.wait_for(world, PEER_HEALTH_HEAD)
        problems += inbox_problems(found, PEER_HEALTH_HEAD, "复查期满都不通")
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(hang)
        open(release, "w").close()
    if problems:
        raise SelftestFailure("第二台连着慢几回的复查期：" + "；".join(problems), world.server_log_lines())
    return "头两回超时、第三回回了：服务日志与 /health 记慢、不推；复查期满都不通：推恰好一条服务健康"


CONTROL_CHECK_LIMIT_SECONDS = 3   # control-check-probe-bounded：现探的总时长上限（--limit）
CONTROL_CHECK_EXTRA_SECONDS = 8   # 到点之后再多给这么久退出（点名卡住那一项、收子进程）


def cell_control_check_probe_bounded(world):
    """control check --probe 的总时长有界、分步可见（task-server/control 的 probed_nfs、task_server_lib.py 的 NfsProbeBudget；两台都堵时 control check 一百秒以上不出结论，
    各会话以为服务卡死）：服务起着、假 ssh 把挂载那一句（stat -f）睡死（fake-ssh-hang）；control check --probe --limit 3：到点退 1（不退 0、不退 78）、不超过 3 秒加 8 秒才退，
    标准输出里打了「目录」「导出」各一对开始与结束、「挂载」开始，标准错误点名卡在哪一个根的「挂载」、写总时长到点。
    弄坏开关 check-probe-unbounded 下现探不设总时长，一直等下去（客户端不在时限里退），这一格红。"""
    world.start_server()
    hang = os.path.join(world.world, "fake-ssh-hang")
    open(hang, "w").close()
    problems = []
    try:
        started = time.monotonic()
        checked = world.control(["check", "--probe", "--limit", str(CONTROL_CHECK_LIMIT_SECONDS)], timeout=CONTROL_CHECK_LIMIT_SECONDS + CONTROL_CHECK_EXTRA_SECONDS)
        took = time.monotonic() - started
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(hang)
    mount = f"共享根 {world.shared_root} 的「挂载」"
    if checked.returncode != 1 or took > CONTROL_CHECK_LIMIT_SECONDS + CONTROL_CHECK_EXTRA_SECONDS:
        problems.append(f"总时长 {CONTROL_CHECK_LIMIT_SECONDS} 秒到点应当退 1、{CONTROL_CHECK_LIMIT_SECONDS + CONTROL_CHECK_EXTRA_SECONDS} 秒内退出，实际退 {checked.returncode}、用了 {took:.0f} 秒")
    for step in (f"共享根 {world.shared_root} 的「目录」开始", f"共享根 {world.shared_root} 的「目录」结束", f"共享根 {world.shared_root} 的「导出」结束", f"{mount}开始"):
        if step not in checked.stdout:
            problems.append(f"现探每一项开始、结束各打一行：标准输出里应当有「{step}」，实际「{checked.stdout.strip()[-400:]}」")
    if f"卡在 {mount}" not in checked.stderr or "到点" not in checked.stderr:
        problems.append(f"到点应当打出卡在「{mount}」、写总时长到点，实际标准错误「{checked.stderr.strip()[-400:]}」")
    if f"{mount}结束" in checked.stdout:
        problems.append("睡死的那一项不该打「结束」")
    if problems:
        raise SelftestFailure("control check --probe 总时长有界、分步可见：" + "；".join(problems), world.server_log_lines())
    return f"挂载那一句睡死：--limit {CONTROL_CHECK_LIMIT_SECONDS} 到点退 1（用了 {took:.0f} 秒），逐项打了开始与结束、点名卡在「挂载」"


def cell_control_check_reads_service_nfs_report(world):
    """control check 默认不现探，读服务 /health 的 nfs 一栏（task-server/control 的 reported_nfs；写明是多久以前判的）：服务起着、NFS 四项都过，假 ssh 把挂载那一句睡死之后
    control check 照样几秒内退 0，输出写「秒前判的前三项」与「两向探针」「要现探加 --probe」，没有「开始」「结束」那种现探的逐项行。
    弄坏开关 check-nfs-probed-live-by-default 下不带 --probe 也现探，睡死的那一句让它一直等，这一格红。"""
    world.start_server()
    hang = os.path.join(world.world, "fake-ssh-hang")
    open(hang, "w").close()
    problems = []
    try:
        started = time.monotonic()
        checked = world.control(["check"], timeout=CONTROL_CHECK_EXTRA_SECONDS + CONTROL_CHECK_LIMIT_SECONDS)
        took = time.monotonic() - started
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(hang)
    if checked.returncode != 0 or took > CONTROL_CHECK_EXTRA_SECONDS + CONTROL_CHECK_LIMIT_SECONDS:
        problems.append(f"默认读服务最近一份结论、第二台睡死也应当几秒内退 0，实际退 {checked.returncode}、用了 {took:.0f} 秒、「{(checked.stdout + checked.stderr).strip()[-300:]}」")
    for text in ("秒前判的前三项", "两向探针", "要现探加 --probe"):
        if text not in checked.stdout:
            problems.append(f"输出应当写明是多久以前判的（「{text}」），实际「{checked.stdout.strip()[-400:]}」")
    if "开始" in checked.stdout:
        problems.append(f"默认路径不该现探（不该有逐项「开始」行），实际「{checked.stdout.strip()[-400:]}」")
    if problems:
        raise SelftestFailure("control check 默认读服务最近一份 NFS 结论：" + "；".join(problems), world.server_log_lines())
    return f"第二台睡死时 control check 默认读 /health 的 nfs 一栏、{took:.0f} 秒退 0，写明多久以前判的，没有现探的逐项行"


PEAK_STAT_KEY = "selftest-peak-memory-stat"
PEAK_STAT_ANON_BYTES = 9216 * MEBIBYTE   # peak-memory-stat-in-note：峰值时的堆
PEAK_STAT_FILE_BYTES = 1024 * MEBIBYTE   # 同一刻的页缓存
PEAK_STAT_TEXTS = ("内存最高那一刻", "anon 9216M", "file 1024M", "大头是进程自己的内存")
# 命令跑到一半把假 scope 换成被杀之后的样子（堆已释放，任务 10279：峰值 45.46 GiB、退出时读到 anon 311296），再退 250
PEAK_STAT_KILLED_SCRIPT = ('printf "%s\\n" "$2" > "$1/memory.current"; printf "anon %s\\nfile %s\\nshmem 0\\n" "$3" "$4" > "$1/memory.stat"; sleep 0.6; '
                           'printf "4096\\n" > "$1/memory.current"; printf "anon 0\\nfile 0\\nshmem 0\\n" > "$1/memory.stat"; sleep 0.5; exit 250')


def cell_peak_memory_stat_in_note(world):
    """内存最高那一刻的明细（task_server.py 文件头 ①「内存最高那一刻」）：假 scope 峰值时 anon 9216M、file 1024M，命令退 250 之前换成 anon 0（被杀之后读的样子）：
    总上限 50G、记录峰值 20G，两次撞上限到顶记 memory-cap-at-maximum，结局说明与 wait 那一行理由都写峰值那一刻的读数与「大头是进程自己的内存」。
    弄坏开关 peak-memory-stat-read-at-exit 下留最后读到的 anon 0，cap-hit-stat-unreported 下不抄，这一格红。"""
    world.resources(local=(55, 50))
    world.seed_peak(PEAK_STAT_KEY, 20 * GIBIBYTE)
    scope = os.path.join(world.root, "peak-stat-scope")
    os.makedirs(scope)
    current = PEAK_STAT_ANON_BYTES + PEAK_STAT_FILE_BYTES
    with open(os.path.join(scope, "memory.current"), "w", encoding="utf-8") as handle:
        handle.write(f"{current}\n")
    with open(os.path.join(scope, "memory.stat"), "w", encoding="utf-8") as handle:
        handle.write(process_memory_stat(PEAK_STAT_ANON_BYTES, PEAK_STAT_FILE_BYTES))
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    task_id = world.submit(["bash", "-c", PEAK_STAT_KILLED_SCRIPT, "peak-stat", scope, str(current), str(PEAK_STAT_ANON_BYTES), str(PEAK_STAT_FILE_BYTES)],
                           extra={"RUN_WITH_MEMORY_CAP_KEY": PEAK_STAT_KEY})
    summary = world.finished(task_id)
    note = summary["task"].get("note") or ""
    completed = world.client(["wait", str(task_id)])
    reason = next((line for line in completed.stdout.splitlines() if line.startswith(lib.REASON_MARK)), "")
    problems = []
    if summary["task"].get("outcome") != "memory-cap-at-maximum":
        problems.append(f"两次撞上限应当到顶记 memory-cap-at-maximum，实际 {describe(summary)}")
    for where, text in (("结局说明", note), ("wait 的理由行", reason)):
        missing = [wanted for wanted in PEAK_STAT_TEXTS if wanted not in text]
        if missing:
            problems.append(f"{where}应当写内存最高那一刻的明细（缺 {missing}），实际「{text[:500]}」")
    if problems:
        raise SelftestFailure("内存最高那一刻的明细：" + "；".join(problems), world.server_log_lines())
    return "假 scope 峰值时堆大、被杀后读 0：结局说明与 wait 的理由行写峰值那一刻的 anon 9216M、file 1024M，大头是进程自己的内存"


def peer_world(world, key):
    """派到第二台那几格的世界：本机只空 8G、第二台 100G，这把命令键记 10G（上限 15G）。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak(key, PEER_PINNED_SEED_PEAK_BYTES)


PEER_STAT_KEY = "selftest-peer-peak-stat"
PEER_STAT_SCRIPT = ('echo "MULTI_HOST_RUN_PEER_MEMORY_STAT poll=1 current_bytes=$(( $1 + $2 )) anon=$1 file=$2 file_dirty=0 file_writeback=0 shmem=0 read=yes"; '
                    'echo "MULTI_HOST_RUN_PEER_MEMORY_STAT poll=2 current_bytes=4096 anon=0 file=0 file_dirty=0 file_writeback=0 shmem=0 read=yes"; '
                    'echo "MULTI_HOST_RUN_PEER_MEMORY_STAT poll=3 current_bytes=0 anon=0 file=0 file_dirty=0 file_writeback=0 shmem=0 read=no"; exit 250')


def cell_peer_peak_memory_stat_in_note(world):
    """第二台内存最高那一刻的明细（task_server.py 文件头 ①「内存最高那一刻」；multi-host-run.sh 每轮一行 MULTI_HOST_RUN_PEER_MEMORY_STAT）：头一行堆大、后两行被杀之后读 0、read=no：
    第 1 次尝试记的 peak_memory_note 写头一行的读数。弄坏开关 peak-memory-stat-read-at-exit 下留最后一行 read=yes 的 anon 0，cap-hit-stat-unreported 下不记，这一格红。"""
    peer_world(world, PEER_STAT_KEY)
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", PEER_STAT_SCRIPT, "peer-stat", str(PEAK_STAT_ANON_BYTES), str(PEAK_STAT_FILE_BYTES), "fake-peer-run"],
                                          extra={lib.KEY_VARIABLE: PEER_STAT_KEY}, cwd=world.repo), timeout=60)
    attempt = first_attempt(summary)
    note = attempt.get("peak_memory_note") or ""
    missing = [wanted for wanted in PEAK_STAT_TEXTS if wanted not in note]
    if attempt.get("host") != "peer" or attempt.get("outcome") != "memory-cap-hit" or missing:
        raise SelftestFailure(f"第二台撞上限的那一次应当记内存最高那一刻的明细（缺 {missing}），实际第 1 次在 {attempt.get('host')}、{attempt.get('outcome')}、记「{note[:300]}」", world.server_log_lines())
    return "第二台每轮报的明细留 current_bytes 最大的那一行：头一行堆大、后面读 0，记的是头一行的读数"


LOOP_LEAK_BOUNCE_KEY = "selftest-loop-leak-bounce"
LOOP_LEAK_BOUNCE_CAPS = ("30720M", "46080M", "69120M")   # 记录峰值 30G 起 30720M（不乘 1.5）、撞了乘 1.5；69120M 再乘 1.5 就过本机可用上限（总上限 80G 减余量 2G）
LOOP_LEAK_BOUNCE_PEAKS = (30200 * MEBIBYTE, 45500 * MEBIBYTE, 68500 * MEBIBYTE)   # 那三次的峰值：一次比一次大、都不低于那一次上限的 98%（STALL_AT_CAP_FRACTION）
LOOP_LEAK_SCOPE_FILES = {"memory.current": 10628 * MEBIBYTE, "memory.stat": process_memory_stat(900 * MEBIBYTE, 9728 * MEBIBYTE).rstrip("\n")}   # 那三次内存最高那一刻的明细：大头是页缓存


def cell_loop_leak_trend_sentences(world):
    """连着撞上限那一段的走势句（task_server_lib.py 文件头「打回的理由」；任务 10412 四次都记 18 MiB、写了一次比一次大）：严格变大写「峰值一次比一次大」，全部相等写「峰值没变」，
    每一次都没顶到上限的另写 LOOP_LEAK_READING_MISMATCH，顶到了的不写。静态喂 lib.loop_leak_check_text，不起服务。
    弄坏开关 loop-leak-equal-peaks-growing 下相等也写一次比一次大，loop-leak-reading-mismatch-silent 下不写那一句，这一格红。"""
    def attempts(*peaks, cap=64 * MEBIBYTE):
        return [{"number": number, "memory_cap": cap, "memory_cap_text": "64M", "memory_peak": peak} for number, peak in enumerate(peaks, 1)]
    low, mid, high = 18 * MEBIBYTE, 40 * MEBIBYTE, 64 * MEBIBYTE
    problems = []
    for what, cap_hits, wanted, unwanted in (
            ("严格变大", attempts(low, mid, high), ["；峰值一次比一次大；"], ["峰值没变"]),
            ("全部相等", attempts(low, low, low), ["；峰值没变；", lib.LOOP_LEAK_READING_MISMATCH], ["；峰值一次比一次大；"]),
            ("都顶到上限", attempts(high, high), ["；峰值没变；", "每次都顶到上限"], [lib.LOOP_LEAK_READING_MISMATCH]),
            ("先大后小", attempts(mid, low), ["；峰值不是一次比一次大；"], ["峰值没变"])):
        text = lib.loop_leak_check_text(cap_hits)
        missing = [piece for piece in wanted if piece not in text]
        extra = [piece for piece in unwanted if piece in text]
        if missing or extra:
            problems.append(f"{what}的那一段缺 {missing}、不该有 {extra}，实际「{text}」")
    if problems:
        raise SelftestFailure("连着撞上限的走势句：" + "；".join(problems))
    return "严格变大写一次比一次大，全部相等写峰值没变，每一次都没顶到上限的另写读数不准那一句"


def cell_loop_leak_check_on_bounce(world):
    """连着撞上限之后打回（task_server_lib.py 文件头「打回的理由」的 loop_leak=）：总上限 80G、记录峰值 30G，假包装三次都退 250、假 scope 给内存最高那一刻的明细，
    峰值 30200M、45500M、68500M（一次比一次大、都顶满）；第三次再乘 1.5 就到顶，记 memory-cap-at-maximum 打回。wait 的理由行带 loop_leak=，写要查的两类、
    逐次的上限与峰值、「峰值一次比一次大」「每次都顶到上限」与最后一次的「大头是页缓存」；推给会话的结局正文另起一行写同一句。
    另静态喂 reason_fields：排队那一刻就打回（一次都没起）的、只撞过一次的不带 loop_leak=。弄坏开关 loop-leak-check-omitted 下不带，这一格红。"""
    problems = []
    single = {"number": 1, "state": "ended", "outcome": "memory-cap-hit", "memory_cap": 30720 * MEBIBYTE, "memory_cap_text": LOOP_LEAK_BOUNCE_CAPS[0],
              "memory_peak": LOOP_LEAK_BOUNCE_PEAKS[0]}
    for what, attempts in (("排队那一刻就打回、一次都没起的", []), ("只撞过一次就打回的", [single])):
        fields = dict(lib.reason_fields({"id": 1, "outcome": "memory-cap-at-maximum", "exit_code": 236}, attempts, ""))
        if "loop_leak" in fields:
            problems.append(f"{what}不该带 loop_leak=，实际「{fields['loop_leak'][:200]}」")
    world.resources(local=(85, 80))
    world.seed_peak(LOOP_LEAK_BOUNCE_KEY, 30 * GIBIBYTE)
    inbox = FakeInbox(world, "a")
    scope = os.path.join(world.root, "loop-leak-scope")
    os.makedirs(scope)
    for name, value in LOOP_LEAK_SCOPE_FILES.items():
        with open(os.path.join(scope, name), "w", encoding="utf-8") as handle:
            handle.write(f"{value}\n")
    start_inbox_server(world, {"TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    peaks = ",".join(str(peak) for peak in LOOP_LEAK_BOUNCE_PEAKS)
    task_id = world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=250,250,250", f"fake-wrapper-exit-peaks={peaks}"],
                           extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: LOOP_LEAK_BOUNCE_KEY}))
    summary = world.finished(task_id)
    caps = [attempt.get("memory_cap_text") for attempt in summary["attempts"]]
    if summary["task"].get("outcome") != "memory-cap-at-maximum" or caps != list(LOOP_LEAK_BOUNCE_CAPS):
        problems.append(f"上限应当 {' → '.join(LOOP_LEAK_BOUNCE_CAPS)}、第三次到顶记 memory-cap-at-maximum，实际上限 {caps}、{describe(summary)}")
    completed = world.client(["wait", str(task_id)])
    reason = next((line for line in completed.stdout.splitlines() if line.startswith(lib.REASON_MARK + " ")), "")
    try:
        paragraph = dict(word.split("=", 1) for word in shlex.split(reason)[1:] if "=" in word).get("loop_leak", "")
    except ValueError as error:
        paragraph = ""
        problems.append(f"wait 的理由行 shlex 拆不开（{error}）：「{reason[:200]}」")
    wanted = [lib.LOOP_LEAK_CHECK_HEAD, "连着撞了 3 次内存上限", lib.LOOP_LEAK_KINDS, "峰值一次比一次大", "每次都顶到上限", "大头是页缓存"]
    wanted += [f"第 {number} 次上限 {cap} 峰值 {lib.size_text(peak)}" for number, (cap, peak) in enumerate(zip(LOOP_LEAK_BOUNCE_CAPS, LOOP_LEAK_BOUNCE_PEAKS), 1)]
    missing = [text for text in wanted if text not in paragraph]
    if missing:
        problems.append(f"wait 的理由行应当带 loop_leak=「{lib.LOOP_LEAK_CHECK_HEAD}…」、写全两类与逐次的依据（缺 {missing}），实际理由行「{reason[:600]}」")
    head = outcome_head(task_id, "memory-cap-at-maximum")
    messages = inbox.wait_for(world, head)
    problems += inbox_problems(messages, head, "连着撞上限打回的结局")
    lines = messages[0]["text"].split("\n") if messages else []
    if paragraph and paragraph not in lines:
        problems.append(f"推给会话的结局正文应当另起一行写 loop_leak= 那一段，实际「{chr(10).join(lines)[:600]}」")
    if problems:
        raise SelftestFailure("连着撞上限打回带检查逻辑循环与内存泄漏的那一段：" + "；".join(problems), world.server_log_lines())
    return "三次撞上限、峰值一次比一次大且都顶满：wait 理由行带 loop_leak=、结局正文另起一行写同一段；一次都没起、只撞过一次的不带"


REPEATED_CAP_HITS_KEY = "selftest-repeated-cap-hits"
REPEATED_CAP_HITS_RELEASE = "release-repeated-cap-hits"   # 第五次尝试等这个放行文件（相对假仓根：本机与假第二台认同一个名字）


def cell_repeated_cap_hits_reminded_once(world):
    """连着撞上限还在重排（task_server.py 文件头 ①「要处置的提醒」的 repeated-cap-hits）：没记录的命令假包装连退四次 250、第五次起命令等放行；
    第五次跑着的时候收件 socket 恰好收到一条「task-server：提醒（…，不是结局）任务 <号> 连着撞上限（repeated-cap-hits；第 1 回」，出路以异常表那一行的修改意见起头，另起一行写
    「检查代码有没有逻辑循环或内存泄漏：」那一段（两类、第 1 次到第 3 次的上限与峰值），末尾的 TASK_SERVER_ATTENTION 带 loop_leak=；第四次再撞不再推；
    放行之后照跑完（四次 memory-cap-hit、第五次 succeeded），结局正文不带那一段。弄坏开关 repeated-cap-hits-silent 下一条都不推，这一格红。"""
    inbox = FakeInbox(world, "a")
    world.resources(local=(100, 100))
    start_inbox_server(world)
    command = ["bash", "-c", BLOCK_SCRIPT, "block", REPEATED_CAP_HITS_RELEASE, f"fake-peer-wait={REPEATED_CAP_HITS_RELEASE}", "fake-wrapper-exits=250,250,250,250,run"]
    task_id = world.submit(command, extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: REPEATED_CAP_HITS_KEY}), cwd=world.repo)
    head = (reminder_head(task_id, "repeated-cap-hits", state="任务还在排"), reminder_head(task_id, "repeated-cap-hits"))   # 撞上限重排的当口是在排还是在跑，推的那一刻都可能
    problems = []
    try:
        world.wait_for(task_id, lambda summary: any(int(attempt.get("number") or 0) >= 5 and attempt.get("state") == "running" for attempt in summary["attempts"]),
                       "第五次尝试跑起来")
        inbox.wait_for(world, head)
        time.sleep(INBOX_QUIET_SECONDS)   # 只推一回：再看几轮不该多一条
        reminders = inbox.headed(head)
        problems += inbox_problems(reminders, head, "连着撞上限还在重排")
        text = reminders[0]["text"] if reminders else ""
        lines = text.split("\n")
        wanted = (f"出路：{table_advice('repeated-cap-hits', task_id)}",
                  f"{lib.ATTENTION_MARK} task={task_id} type={lib.ATTENTION_TYPE} kind=repeated-cap-hits basis=", "loop_leak=")
        missing = [piece for piece in wanted if piece not in text]
        paragraph = next((line for line in lines if line.startswith(lib.LOOP_LEAK_CHECK_HEAD)), "")
        missing += [piece for piece in ("第 1 次上限", "第 3 次上限", lib.LOOP_LEAK_KINDS) if piece not in paragraph]
        if reminders and missing:
            problems.append(f"那一条的正文应当带 {missing}（「{lib.LOOP_LEAK_CHECK_HEAD}…」另起一行），实际「{text[:700]}」")
    finally:
        open(os.path.join(world.repo, REPEATED_CAP_HITS_RELEASE), "w").close()
    done = world.finished(task_id)
    outcomes = [attempt.get("outcome") for attempt in sorted(done["attempts"], key=lambda attempt: int(attempt.get("number") or 0))]
    if outcomes != ["memory-cap-hit"] * 4 + ["succeeded"] or done["task"].get("outcome") != "succeeded":
        problems.append(f"只提醒、照跑：应当四次 memory-cap-hit 之后第五次跑完，实际各次 {outcomes}、{describe(done)}")
    finished = inbox.wait_for(world, outcome_head(task_id, "succeeded"))
    time.sleep(INBOX_QUIET_SECONDS)
    if len(inbox.headed(head)) != 1:
        problems.append(f"同一条任务只推一回：跑完之后 repeated-cap-hits 那一条应当仍只有一条，实际 {len(inbox.headed(head))} 条")
    if finished and lib.LOOP_LEAK_CHECK_HEAD in finished[0]["text"]:
        problems.append(f"跑成了的结局正文不该带「{lib.LOOP_LEAK_CHECK_HEAD}」那一段，实际「{finished[0]['text'][:400]}」")
    if problems:
        raise SelftestFailure("连着撞上限还在重排的提醒：" + "；".join(problems), world.server_log_lines())
    return "四次撞上限、照乘 1.5 重排：第五次跑着时收到恰好一条 repeated-cap-hits（另起一行写检查逻辑循环与内存泄漏），第四次撞不再推，照跑完"


RUN_LIMIT_SAMPLE_SECONDS = "3"   # 跑满两小时那几格：界压到 3 秒（TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS），不真等两小时
RUN_LIMIT_ACROSS_SECONDS = "6"   # long-run-reminder-across-cap-hits：前四次各跑 1.5 秒撞上限、第五次跑约 1 秒，界 6 秒（哪一次都不到界的一半多，四次加起来满；机器忙时单次放慢到界的 3 倍多才会误判）
RUN_LIMIT_SHORT_SECONDS = "1"    # long-run-confirm-silences、long-run-sessionless-noted：界压到 1 秒，命令等放行才退
LONG_TICKS_SCRIPT = 'for round in $(seq 1 30); do echo "tick $round"; sleep 0.2; done'   # 跑 6 秒、每 0.2 秒打一行，比 3 秒的界长
ACROSS_CAP_HITS_SCRIPT = ('if [[ "$TASK_SERVER_ATTEMPT" -lt 5 ]]; then echo "attempt $TASK_SERVER_ATTEMPT"; sleep 1.5; exit 250; fi; '
                          'for round in $(seq 1 5); do echo "tick $round"; sleep 0.2; done')   # 前四次各睡 1.5 秒撞上限，第五次约 1 秒跑完
SCOPE_DAILY = "种类=items; 跑什么=自证造的一条; 范围=1 条; 为什么=自证：日常开发跑一阵子"
SCOPE_COMMIT = "种类=whole; 跑什么=自证造的门禁; 范围=全部; 为什么=自证：提交前跑整轮"
LONG_RUN_WHY_SAMPLE = "自证：这一条点的就是要跑的全部范围，确实要跑这么久"
LONG_RUN_KIND = "long-run-unconfirmed"
CLIENT_NO_SUCH_TASK_EXIT = 4   # 客户端「没有这个任务」的退出码（task-server/task 的 NO_SUCH_TASK_EXIT）
FAKE_COMMIT_GATE = ".claude/gate.d/lkmm.sh"   # 假仓里的一道提交时门禁（lib.COMMIT_GATE_ENTRIES 之一）：跑 LONG_TICKS_SCRIPT
FAKE_MERGE_ENTRY = "scripts/gate/merge-and-verify.sh"


def write_fake_entry(world, relative):
    """假仓根下 relative 那份脚本：正文是 LONG_TICKS_SCRIPT（参数不看）。"""
    path = os.path.join(world.repo, relative)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    write_executable(path, "#!/usr/bin/env bash\n" + LONG_TICKS_SCRIPT + "\n")


def long_run_head(task_id):
    """推给会话的跑满两小时提醒的第一行到「（long-run-unconfirmed」为止的一段（头一回与再推都以它起头）。"""
    return reminder_head(task_id, LONG_RUN_KIND)


def reminded_problems(world, task_id, what):
    """没带全量标识的那几条（wait 那一路）：满界 wait 打一行 kind=long-run-unconfirmed、退 ATTENTION_EXIT，任务照跑完 succeeded。→ 问题表。"""
    completed = world.client(["wait", str(task_id)])
    problems, _ = attention_problems(completed, task_id, LONG_RUN_KIND, lib.ATTENTION_EXIT)
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"满界服务不停它，应当跑完 succeeded，实际 {describe(summary)}")
    return [f"{what}{problem}" for problem in problems]


def running_attempt_log(world, task_id):
    """等这条任务有一次在跑的尝试：→ 那一次的日志路径。"""
    summary = world.wait_for(task_id, lambda seen: bool(seen["attempts"]) and seen["attempts"][-1].get("state") in SERVER.ACTIVE_ATTEMPT_STATES, "有一次在跑的尝试")
    return summary["attempts"][-1].get("log") or ""


def read_text(path):
    with contextlib.suppress(OSError):
        with open(path, encoding="utf-8", errors="replace") as handle:
            return handle.read()
    return ""


def cell_long_run_reminded_not_stopped(world):
    """跑满两小时提醒（task_server_lib.py 文件头「异常表」long-run-unconfirmed；界压到 3 秒）：日常开发的一条跑 6 秒，满 3 秒服务不停它、跑完 succeeded、退 0；
    推给会话恰好一条（再推要等 30 分钟，这一格到不了），写命令、范围声明、累计与各次用时、日志末几行，出路写 task cancel 与 task confirm <号> --why 整行。
    弄坏开关 long-run-unconfirmed-silent 下不推，这一格红。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", LONG_TICKS_SCRIPT], extra=dict(inbox.environment(), SINGLEFS_RUN_SCOPE=SCOPE_DAILY))
    found = inbox.wait_for(world, long_run_head(task_id))
    text = found[0]["text"] if found else ""
    summary = world.finished(task_id)
    problems = []
    if summary["task"].get("outcome") != "succeeded" or summary["task"].get("exit_code") != 0:
        problems.append(f"满界服务不停它，应当跑完 succeeded、退 0，实际 {describe(summary)}")
    wanted = ("累计跑了", "各次 第 1 次", "命令：", "范围声明：", "自证：日常开发跑一阵子", "日志末几行：", "tick", f"task-server/task cancel {task_id}", f"task-server/task confirm {task_id} --why")
    missing = [piece for piece in wanted if piece not in text]
    if not found or missing:
        problems.append(f"推给会话的那一条应当写累计与各次用时、命令、范围声明、日志末几行与两条出路（缺 {missing}），实际「{text[:600]}」")
    pushed = inbox.headed(long_run_head(task_id))
    if len(pushed) != 1:
        problems.append(f"满界应当恰好推一回，实际 {len(pushed)} 回")
    if problems:
        raise SelftestFailure("跑满两小时提醒：" + "；".join(problems), world.server_log_lines())
    return f"日常开发的一条累计满 {RUN_LIMIT_SAMPLE_SECONDS} 秒推一回、写全了依据与两条出路，服务不停它、跑完 succeeded"


def cell_long_run_confirm_silences(world):
    """没确认的再推、确认了不再推（task_server.py 文件头 ⑦「跑满两小时」；界压到 1 秒、再推的门槛 2 秒）：等放行的一条满界推一回，没处置再推「第 1 回再推」；
    照整行 task confirm <号> --why 确认，退 0，之后不再推，task status、日志与结局那一条都有「长跑已确认：<why>」；放行之后跑完 succeeded。
    弄坏开关 anomaly-not-repushed 下不再推，long-run-confirmed-still-reminded 下确认了还推，long-run-confirmed-flag-hidden 下没有那一行，这一格红。"""
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-long-run-confirm")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SHORT_SECONDS, "TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS": REPUSH_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra=dict(inbox.environment(), SINGLEFS_RUN_SCOPE=SCOPE_DAILY))
    problems = []
    flag = f"长跑已确认：{LONG_RUN_WHY_SAMPLE}"
    try:
        log = running_attempt_log(world, task_id)
        first_head, again_head = f"{long_run_head(task_id)}；", f"{long_run_head(task_id)}，没处置、第 1 回再推；第 2 回"
        problems += inbox_problems(inbox.wait_for(world, first_head), first_head, "头一回")
        problems += inbox_problems(inbox.wait_for(world, again_head), again_head, "再推")
        confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE])
        if confirmed.returncode != 0 or LONG_RUN_WHY_SAMPLE not in confirmed.stdout:
            problems.append(f"task confirm <号> --why 应当退 0、打那一句，实际退 {confirmed.returncode}：「{(confirmed.stdout + confirmed.stderr).strip()[-300:]}」")
        settled = len(inbox.headed(long_run_head(task_id)))
        time.sleep(3 * float(REPUSH_SAMPLE_SECONDS))
        if len(inbox.headed(long_run_head(task_id))) != settled:
            problems.append(f"确认了之后不该再推，实际确认时 {settled} 条、等了之后 {len(inbox.headed(long_run_head(task_id)))} 条")
        shown = world.client(["status", str(task_id)])
        if flag not in shown.stdout + shown.stderr:
            problems.append(f"task status 应当有一行「{flag}」，实际「{(shown.stdout + shown.stderr).strip()[:400]}」")
        if LONG_RUN_WHY_SAMPLE not in read_text(log):
            problems.append(f"这一次尝试的日志应当接一行确认的 --why，实际日志「{read_text(log)[-300:]}」")
    finally:
        open(release, "w").close()
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当跑完 succeeded，实际 {describe(summary)}")
    found = inbox.wait_for(world, outcome_head(task_id, "succeeded"))
    if not found or flag not in found[0]["text"]:
        problems.append(f"推给会话的结局那一条应当有「{flag}」，实际「{(found[0]['text'] if found else '没收到')[:400]}」")
    if problems:
        raise SelftestFailure("跑满两小时确认：" + "；".join(problems), world.server_log_lines())
    return "满界推一回、没处置再推，确认之后不再推，status、日志与结局都有长跑已确认那一行，跑完 succeeded"


def cell_long_run_reminder_across_cap_hits(world):
    """撞了内存上限重来的计时归零（task_server.py 的 run_limit_accumulated）：界 6 秒，前四次各跑 1.5 秒退 250（撞上限、乘 1.5 重排），第五次跑约 1 秒：五次加起来 7 秒过了界，
    而每一次撞上限之后都从零算、哪一次都不满 6 秒：一条提醒都没有，跑完 succeeded，五次结局是四次 memory-cap-hit、一次 succeeded。单次用时只占界的四分之一，机器忙时慢到三倍多才会误判。
    弄坏开关 long-run-cap-hits-counted 下撞上限的那几次也累加，第四次跑到 1.5 秒就满界推一回，这一格红。"""
    inbox = FakeInbox(world, "a")
    world.resources(local=(100, 100))
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_ACROSS_SECONDS})
    task_id = world.submit(["bash", "-c", ACROSS_CAP_HITS_SCRIPT], extra=dict(inbox.environment(), SINGLEFS_RUN_SCOPE=SCOPE_DAILY))
    summary = world.finished(task_id)
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    problems = []
    if summary["task"].get("outcome") != "succeeded" or outcomes != ["memory-cap-hit"] * 4 + ["succeeded"]:
        problems.append(f"四次撞上限各 1.5 秒、第五次约 1 秒应当跑完 succeeded（各次四个 memory-cap-hit、一个 succeeded），实际 {describe(summary)}、各次 {outcomes}")
    inbox.wait_for(world, outcome_head(task_id, "succeeded"))   # 结局那一条之前的提醒都发完了
    pushed = inbox.headed(long_run_head(task_id))
    if pushed:
        problems.append(f"每次撞上限后归零，不该提醒，实际推了 {len(pushed)} 回：「{pushed[0]['text'][:300]}」")
    if problems:
        raise SelftestFailure("撞上限重来的计时归零：" + "；".join(problems), world.server_log_lines())
    return f"四次撞上限各跑 1.5 秒、第五次跑约 1 秒，五次加起来过了 {RUN_LIMIT_ACROSS_SECONDS} 秒的界，每次撞上限后归零、不提醒、照跑完"


def cell_long_run_full_gate_not_reminded(world):
    """带全量门禁标识的不提醒：假仓里的 .claude/gate.d/lkmm.sh 带 --full-gate，跑 6 秒过了 3 秒的界，一条提醒都没有、跑完 succeeded。弄坏开关 full-gate-flag-ignored 下照提醒，这一格红。"""
    inbox = FakeInbox(world, "a")
    world.resources(local=(30, 30))
    write_fake_entry(world, FAKE_COMMIT_GATE)
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    task_id = world.submit(["bash", FAKE_COMMIT_GATE], extra=dict(inbox.environment(), SINGLEFS_RUN_SCOPE=SCOPE_COMMIT), cwd=world.repo)
    summary = world.finished(task_id)
    inbox.wait_for(world, outcome_head(task_id, "succeeded"))   # 结局那一条之前的提醒都发完了
    pushed = inbox.headed(long_run_head(task_id))
    if summary["task"].get("outcome") != "succeeded" or pushed:
        raise SelftestFailure(f"提交前跑的提交时门禁带 --full-gate 应当不提醒、跑完 succeeded，实际 {describe(summary)}、推了 {len(pushed)} 回", world.server_log_lines())
    return "带全量标识的提交时门禁过了界不提醒、照跑完"


def cell_long_run_check_not_exempt(world):
    """没带全量标识的门禁提醒（任务 9604 那种带 --check 的门禁跑了 6.2 小时）：同一道门禁带 --check（范围声明写了提交前也不算）的满 3 秒记提醒、照跑完。
    不带 --check 又没带全量标识的提交时就被拒（whole_package_refusal），这一格不交它。"""
    world.resources(local=(30, 30))
    write_fake_entry(world, FAKE_COMMIT_GATE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    checked = world.submit(["bash", FAKE_COMMIT_GATE, "--check", "lkmm-cell"], extra={"SINGLEFS_RUN_SCOPE": SCOPE_COMMIT}, cwd=world.repo)
    problems = reminded_problems(world, checked, "带 --check 的门禁，")
    if problems:
        raise SelftestFailure("提交时门禁的提醒：" + "；".join(problems), world.server_log_lines())
    return "带 --check 的门禁满界提醒、照跑完"


def cell_long_run_merge_and_verify_not_exempt(world):
    """merge-and-verify 同样提醒（给了补丁目录、「为什么」写了提交前也一样）：假仓里的 scripts/gate/merge-and-verify.sh 跑 6 秒，满 3 秒记提醒、照跑完。"""
    world.resources(local=(100, 100))
    write_fake_entry(world, FAKE_MERGE_ENTRY)
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    task_id = world.submit(["bash", FAKE_MERGE_ENTRY, "selftest-patch-directory"], extra={"SINGLEFS_RUN_SCOPE": SCOPE_COMMIT}, cwd=world.repo)
    problems = reminded_problems(world, task_id, "merge-and-verify，")
    if problems:
        raise SelftestFailure("merge-and-verify 的提醒：" + "；".join(problems), world.server_log_lines())
    return "merge-and-verify 满界提醒、照跑完"


def cell_long_run_confirm_refused(world):
    """task confirm <号> --why 不合的（task_server.py 文件头 ⑦「跑满两小时」；界压到 3 秒）：没满界的、--why 去掉空白不到 12 个字或全是空白的、别的会话的、有了结局的、带 --full-gate 的都退 3，
    没有这个任务退 4，--why 后面没给值退 2；合的退 0，再确认一回照退 0、不改原来那一句。
    弄坏开关 long-run-short-why-accepted 下太短的也照收，long-run-confirm-unchecked 下没满界、别的会话与有了结局的也照收，这一格红。"""
    release = os.path.join(world.root, "release-long-run-refused")
    world.resources(local=(30, 30))
    write_fake_entry(world, FAKE_COMMIT_GATE)
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    finished_id = world.submit(["bash", "-c", "exit 0", "already-finished"])
    world.finished(finished_id)
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY})
    problems = []

    def confirm_exit(what, words, wanted, session=SESSION_A):
        completed = world.client(["confirm", *words], session=session)
        if completed.returncode != wanted:
            problems.append(f"{what}应当退 {wanted}，实际退 {completed.returncode}：「{(completed.stdout + completed.stderr).strip()[-300:]}」")
        return completed
    try:
        running_attempt_log(world, task_id)
        confirm_exit("没满界的", [str(task_id), "--why", LONG_RUN_WHY_SAMPLE], 3)
        time.sleep(float(RUN_LIMIT_SAMPLE_SECONDS) + 0.5)
        short = confirm_exit("--why 太短的", [str(task_id), "--why", "太短理由"], 3)
        if f"至少 {lib.LONG_RUN_WHY_MINIMUM_CHARACTERS}" not in short.stderr:
            problems.append(f"--why 太短的应当写「至少 {lib.LONG_RUN_WHY_MINIMUM_CHARACTERS}」，实际「{short.stderr.strip()[-300:]}」")
        confirm_exit("--why 全是空白的", [str(task_id), "--why", "   \t  "], 3)
        confirm_exit("别的会话的", [str(task_id), "--why", LONG_RUN_WHY_SAMPLE], 3, session=SESSION_B)
        confirm_exit("有了结局的", [str(finished_id), "--why", LONG_RUN_WHY_SAMPLE], 3)
        confirm_exit("没有这个任务的", ["999999", "--why", LONG_RUN_WHY_SAMPLE], CLIENT_NO_SUCH_TASK_EXIT)
        confirm_exit("--why 后面没给值的", [str(task_id), "--why"], lib.USAGE_EXIT)
        confirm_exit("合的", [str(task_id), "--why", LONG_RUN_WHY_SAMPLE], 0)
        again = confirm_exit("再确认一回的", [str(task_id), "--why", "另一句写够十二个字的理由，不该换掉原来的"], 0)
        if LONG_RUN_WHY_SAMPLE not in again.stdout:
            problems.append(f"再确认一回应当说原来就确认过、留原来那一句，实际「{again.stdout.strip()[-300:]}」")
    finally:
        open(release, "w").close()
    world.finished(task_id)
    gate_id = world.submit(["bash", FAKE_COMMIT_GATE], extra={"SINGLEFS_RUN_SCOPE": SCOPE_COMMIT}, cwd=world.repo)
    running_attempt_log(world, gate_id)
    time.sleep(float(RUN_LIMIT_SAMPLE_SECONDS) + 0.5)
    confirm_exit("带 --full-gate 的", [str(gate_id), "--why", LONG_RUN_WHY_SAMPLE], 3)
    world.finished(gate_id)
    if problems:
        raise SelftestFailure("task confirm --why 的拒：" + "；".join(problems), world.server_log_lines())
    return "没满界、--why 太短或全是空白、别的会话、有了结局、带 --full-gate 的退 3，没有这个任务退 4，没给值退 2；合的退 0，再确认照退 0、不改原来那一句"


def cell_long_run_sessionless_noted(world):
    """没有会话的跑满两小时（task_server.py 文件头 ①「要处置的提醒」的 long-run-unconfirmed；界压到 1 秒）：人手 --manual 交的一条不推，这一次尝试的日志接一行 lib.LONG_RUN_LOG_MARK 起头的待确认、
    恰好一行，status 打「长跑待确认」；同样可以确认（退 0），status 换成「长跑已确认」，放行后跑完。
    弄坏开关 long-run-sessionless-unnoted 下不写日志那一行，long-run-unconfirmed-silent 下都没有，这一格红。"""
    release = os.path.join(world.root, "release-long-run-sessionless")
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SHORT_SECONDS})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], session=None)
    problems = []
    mark = f"{lib.LONG_RUN_LOG_MARK} 长跑待确认"
    try:
        log = running_attempt_log(world, task_id)
        patience = world.patience(INBOX_WAIT_SECONDS)
        while mark not in read_text(log) and patience.waiting():
            time.sleep(POLL_SECONDS)
        lines = [line for line in read_text(log).splitlines() if line.startswith(mark)]
        if len(lines) != 1 or "累计跑了" not in lines[0] or f"task-server/task confirm {task_id} --why" not in lines[0]:
            problems.append(f"日志里应当恰好一行「{mark}…」写累计与确认的写法，实际 {len(lines)} 行：「{(lines or [''])[0][:300]}」")
        shown = world.client(["status", str(task_id)], session=None)
        if "长跑待确认" not in shown.stdout:
            problems.append(f"task status 应当打「长跑待确认」，实际「{shown.stdout.strip()[:400]}」")
        confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE], session=None)
        again = world.client(["status", str(task_id)], session=None)
        if confirmed.returncode != 0 or f"长跑已确认：{LONG_RUN_WHY_SAMPLE}" not in again.stdout or "长跑待确认" in again.stdout:
            problems.append(f"人手交的同样可以确认，确认之后 status 换成已确认，实际退 {confirmed.returncode}、status「{again.stdout.strip()[:400]}」")
    finally:
        open(release, "w").close()
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当跑完 succeeded，实际 {describe(summary)}")
    if problems:
        raise SelftestFailure("没有会话的跑满两小时：" + "；".join(problems), world.server_log_lines())
    return "人手交的不推，日志接恰好一行待确认、status 打一行，同样可以确认"


WHOLE_PACKAGE_COMMANDS = (   # ⑬ 整包测试四种形态（task_server.py 文件头 ②「提交时拒」⑥）：(说什么, 命令, 在哪起)
    ("cargo test -p 不点名测试目标", ["cargo", "test", "-p", "singlefs-verification"], "work"),
    ("cargo test --lib --tests", ["cargo", "test", "-p", "singlefs-verification", "--lib", "--tests"], "work"),
    ("crash-point-replay.sh --check 三格全点", ["bash", ".claude/gate.d/crash-point-replay.sh", "--check", "crash-case-replay,feature-gated-tests,product-gpu-real-card-tests"], "repo"),
    ("不给补丁目录的 merge-and-verify.sh", ["bash", "scripts/gate/merge-and-verify.sh"], "repo"))


def submit_raw(world, command, cwd, confirmation="", scope=SCOPE_DAILY, full_gate=None):
    """只交、不确认（交了的待确认、不进队、不跑）：→ CompletedProcess。full_gate 没给的，演提交时那一趟（scope 是 SCOPE_COMMIT 而命令是提交时门禁）带全量标识。"""
    if full_gate is None:
        full_gate = scope == SCOPE_COMMIT and lib.commit_gate_entry(list(command), cwd)
    words = ["submit", "--cwd", cwd] + (["--full-gate"] if full_gate else []) + (["--whole-package-confirmed", confirmation] if confirmation else []) + ["--", *command]
    return world.client(words, extra={"SINGLEFS_RUN_SCOPE": scope})


def submit_confirmed(world, command, cwd, confirmation, scope=SCOPE_DAILY):
    """带整包确认文件交一条、照确认单确认（同 World.submit），交任务号。"""
    completed = submit_raw(world, command, cwd, confirmation, scope)
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
    if completed.returncode != 0 or not found:
        raise SelftestFailure(f"带确认文件提交 {command} 没拿到任务号（退 {completed.returncode}）", [completed.stdout, completed.stderr] + world.server_log_lines())
    world.confirm_submitted(completed, extra={"SINGLEFS_RUN_SCOPE": scope})
    return int(found.group(1))


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


WHOLE_PACKAGE_CONFIRMATION_TEXT = ("① 改到的文件与函数：task-server/task_server.py 的 judge_stall_stop 与 run_limit_accumulated\n"
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


def cell_whole_package_confirmed_still_reminded(world):
    """整包已确认的照样受两小时提醒（task_server.py 文件头 ②「提交时拒」⑥ 与 ⑦「跑满两小时」）：不给补丁目录的假 merge-and-verify.sh 带三段写全的确认文件交、跑 6 秒，满 3 秒记提醒、服务不停；
    照提醒 task confirm <号> --why 确认，status 有整包已确认与长跑已确认两行，照跑完。
    弄坏开关 whole-package-confirmed-exempts-two-hours 下不提醒，long-run-confirmed-flag-hidden 下没有长跑已确认那一行，这一格红。"""
    world.resources(local=(100, 100))
    release = os.path.join(world.root, "release-whole-package-reminded")
    entry = os.path.join(world.repo, FAKE_MERGE_ENTRY)
    os.makedirs(os.path.dirname(entry), exist_ok=True)
    write_executable(entry, f"#!/usr/bin/env bash\nfor round in $(seq 1 600); do [[ -e {shlex.quote(release)} ]] && exit 0; sleep 0.1; done; exit 9\n")
    world.start_server(extra={"TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS": RUN_LIMIT_SAMPLE_SECONDS})
    full = write_confirmation(world, "whole-package-full.md", WHOLE_PACKAGE_CONFIRMATION_TEXT)
    task_id = submit_confirmed(world, ["bash", FAKE_MERGE_ENTRY], world.repo, full)
    waited = world.client(["wait", str(task_id)])
    problems, _ = attention_problems(waited, task_id, LONG_RUN_KIND, lib.ATTENTION_EXIT)
    confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE])
    shown = world.client(["status", str(task_id)])
    text = shown.stdout + shown.stderr
    if confirmed.returncode != 0:
        problems.append(f"task confirm <号> --why 应当退 0，实际退 {confirmed.returncode}：「{(confirmed.stdout + confirmed.stderr).strip()[-300:]}」")
    for line in (f"整包已确认：{full}", f"长跑已确认：{LONG_RUN_WHY_SAMPLE}"):
        if line not in text:
            problems.append(f"确认之后 status 应当有「{line}」，实际「{text.strip()[:400]}」")
    open(release, "w").close()
    summary = world.finished(task_id)
    if summary["task"].get("whole_package_confirmed") != full or summary["task"].get("outcome") != "succeeded":
        problems.append(f"应当带整包确认文件、过了界照跑完，实际标识「{summary['task'].get('whole_package_confirmed')}」、{describe(summary)}")
    if problems:
        raise SelftestFailure("整包已确认照样受两小时提醒：" + "；".join(problems), world.server_log_lines())
    return f"整包已确认的满 {RUN_LIMIT_SAMPLE_SECONDS} 秒照样提醒、服务不停；确认之后两种标识都打、照跑完"


def cell_attention_cleanup_pending(world):
    """清理没收尾：派第二台的一条，第二台那一层写 pending 的清理回执（假 multi-host-run.sh 的 fake-peer-cleanup=pending）却一直不确认，服务保留这一次尝试的占用、任务不记结局；
    wait 打一行 kind=cleanup-pending、依据带回执未确认的原因、退 ATTENTION_EXIT；回执改成 confirmed 之后任务跑完。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "attention-cleanup", "fake-peer-cleanup=pending"], cwd=world.repo)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    during = world.summary(task_id)
    problems, fields = attention_problems(completed, task_id, "cleanup-pending", lib.ATTENTION_EXIT)
    attempt = (during["attempts"] or [{}])[-1]
    if "清理没收尾" not in fields.get("basis", "") or "未确认" not in fields.get("basis", ""):
        problems.append(f"依据应当写「清理没收尾」与回执未确认的原因，实际「{fields.get('basis')}」")
    if during["task"].get("state") != "running" or attempt.get("state") not in ("starting", "running") or not attempt.get("cleanup_pending"):
        problems.append(f"报了清理没收尾之后任务应当照旧在跑、这一次尝试保持在跑并记着 cleanup_pending，实际 {describe(during)}、状态 {during['task'].get('state')}、尝试 {attempt.get('state')}")
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
        raise SelftestFailure("清理没收尾：" + "；".join(problems), world.server_log_lines())
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
    """同一条任务同一种提醒、同一道提醒门槛只叫醒一次：排队过久（排满 6 秒、12 秒各一道门槛）叫醒过一次，主 agent 重起 wait 之后到 12 秒之前不再叫（再起的 wait 这段时间一直等着），
    排满 12 秒才再叫一次（reminded_after_seconds 从 6 到 12，依据不同、写着排满 24 秒再提醒一回）；任务一直是在排。"""
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
        output, _ = world.stop_client(second)
    second_done = subprocess.CompletedProcess(second.args, second.returncode, output, "")
    more, second_fields = attention_problems(second_done, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    problems += more
    later = 2 * ATTENTION_ONCE_SECONDS
    reached = (first_fields.get("reminded_after_seconds"), second_fields.get("reminded_after_seconds"))
    if reached != (str(ATTENTION_ONCE_SECONDS), str(later)) or first_fields.get("basis") == second_fields.get("basis") \
            or f"排满 {2 * later} 秒再提醒一回" not in second_fields.get("basis", ""):
        problems.append(f"头一回应当 reminded_after_seconds={ATTENTION_ONCE_SECONDS}、再起的 wait 排满 {later} 秒才叫 reminded_after_seconds={later}、"
                        f"依据不同且写着排满 {2 * later} 秒再提醒一回，实际 {reached}、依据「{first_fields.get('basis')}」与「{second_fields.get('basis')}」")
    if not quiet:
        problems.append(f"{ATTENTION_ONCE_SECONDS} 秒那一回叫过之后再起 wait，{ATTENTION_ONCE_QUIET_SECONDS} 秒里不该再叫，实际它当场退了（退 {second.returncode}）")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    world.finished(task_id)
    if problems:
        raise SelftestFailure("提醒只叫醒一次：" + "；".join(problems), world.server_log_lines())
    return f"排满 {ATTENTION_ONCE_SECONDS} 秒叫醒一次，重起的 wait 到 {later} 秒之前不再叫，排满 {later} 秒才再叫（依据「{second_fields.get('basis')}」）"


ATTENTION_RESTART_SECONDS = 8   # attention-reported-survives-restart：排满 8 秒、16 秒各一道提醒门槛（8 秒那一回报过之后重启服务留得出 8 秒）


def cell_attention_reported_survives_restart(world):
    """报到的提醒门槛记进库（task_server.py 文件头 ①「要处置的提醒」；会话 46 的任务 8819 那 30 片发过一轮，主 agent 重启服务之后同一道门槛又发一轮）：
    排队过久（排满 8 秒、16 秒各一道门槛）报过 8 秒那一回，停掉服务、照同一份库重起，再起的 wait 不再报 8 秒那一回、排满 16 秒才报；任务一直在排。
    弄坏开关 attention-reported-memory-only 下照改前只记在服务内存里：重起之后再起的 wait 当场又报 8 秒那一回，这一格红。"""
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
    reached = (first_fields.get("reminded_after_seconds"), second_fields.get("reminded_after_seconds"))
    if reached != (str(ATTENTION_RESTART_SECONDS), str(2 * ATTENTION_RESTART_SECONDS)):
        problems.append(f"重启之前应当报 reminded_after_seconds={ATTENTION_RESTART_SECONDS}，重启之后再起的 wait 不再报那一回、排满 {2 * ATTENTION_RESTART_SECONDS} 秒才报，"
                        f"实际 {reached}（重启之后那一行的依据「{second_fields.get('basis')}」）")
    world.resources(local=(50, 20), peer=(50, 50), peer_cap_gibibytes=50)
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"第二台回来之后应当派过去跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("报到的提醒门槛跨重启：" + "；".join(problems), world.server_log_lines())
    return f"排满 {ATTENTION_RESTART_SECONDS} 秒那一回报过之后重启服务，再起的 wait 不再报那一回，排满 {2 * ATTENTION_RESTART_SECONDS} 秒才报（依据「{second_fields.get('basis')}」）"


REASON_PENDING_QUIET_SECONDS = 3.0   # attention-queue-waits-for-reason：排满提醒门槛（2 秒）之后再看这么久，排队原因一直判不出来的这段时间 wait 不该叫


def cell_attention_queue_waits_for_reason(world):
    """排队原因还没判出来的不报排队过久（task_server.py 文件头 ①「要处置的提醒」；会话 46 那一回重启之后头一轮的原因写的是「排队原因此刻读不到」）：
    本机放得下、排队线程照判不起（TASK_SERVER_SELFTEST_PLACEMENT_PAUSED：派活判它放得下就不写排队原因，演服务刚重启、这一轮还没判到它），
    排满提醒门槛（2 秒）再过 3 秒，在等的 wait 一直不叫；换成本机放不下、第二台用不了，排队线程判出原因（第二台用不了）的那一轮 wait 才叫，原因不空、点名第二台。
    弄坏开关 attention-queue-reasonless-sent 下照改前照报：一到提醒门槛 wait 就叫、原因写读不到，这一格红。"""
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
        output, _ = world.stop_client(waiting)
    completed = subprocess.CompletedProcess(waiting.args, waiting.returncode, output, "")
    problems, fields = attention_problems(completed, task_id, "queued-too-long", lib.ATTENTION_EXIT)
    if not quiet:
        problems.append(f"排队原因判不出来的那几轮 wait 不该叫，实际它在排满 {ATTENTION_QUEUE_SECONDS} 秒之后 {REASON_PENDING_QUIET_SECONDS} 秒里就退了"
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


def cell_attention_type_in_anomaly_table(world):
    """异常表（lib.ANOMALY_TABLE_TEXT）与代码同步：提醒的类型名、退出码、各种 kind（ATTENTION_KINDS）与那一行的标记都写在表里；退出码不与错误类型表里的码、疑似卡住的码撞。"""
    table = lib.ANOMALY_TABLE_TEXT
    taken = {code for code, _ in lib.OUTCOME_TYPES.values() if code is not None} | set(lib.EXIT_TYPES) | {lib.SUSPECT_EXIT}
    problems = []
    if lib.ATTENTION_EXIT in taken:
        problems.append(f"提醒的退出码 {lib.ATTENTION_EXIT} 不该与错误类型表、疑似卡住的码撞")
    if not re.search(rf"{re.escape(lib.ATTENTION_TYPE)}[^\n]*\b{lib.ATTENTION_EXIT}\b", table):
        problems.append(f"异常表该有一行写 {lib.ATTENTION_TYPE} 与退出码 {lib.ATTENTION_EXIT}")
    missing = [name for name in (lib.ATTENTION_MARK, *lib.ATTENTION_KINDS) if name not in table]
    if missing:
        problems.append(f"异常表该写 {missing}")
    if problems:
        raise SelftestFailure("；".join(problems))
    return f"提醒的类型 {lib.ATTENTION_TYPE}、退出码 {lib.ATTENTION_EXIT}、{len(lib.ATTENTION_KINDS)} 种 kind 与标记都在异常表里，码没和错误类型表撞"


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


# ── 峰值起步、重试不重编、改主工作区的命令没正常结束（task_server.py 文件头 ②「内存」、①「要处置的提醒」）──
SPACED_ITEM_NAME = "带 空格的 条目"   # shard-child-key-folded-by-words：条目名里带空格（后半截不该漏进分片子任务的键里）
SPACED_ITEM_TAIL = "空格的"
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


# ── 近几次最大、打补丁脚本干跑照常重排、第二台重试接着用编译目录（task_server_lib.py 文件头「资源记录」的定额度、「编译目录」的重试复用）──
HISTORY_CAP_HIT_BYTES = 13824 * MEBIBYTE    # history-peak-first-attempt 与 history-peak-window-raises：同键撞过的上限（任务 7349、7416 撞过的那一档）
HISTORY_CAP_HIT_TEXT = "13824M"             # 照撞过的上限本身，不乘 1.5
HISTORY_SMALL_PEAK_BYTES = GIBIBYTE         # 跑成的小峰值：history-peak-first-attempt 里近 RECORD_WINDOW 次只有它们，起点回落到 1G
HISTORY_SMALL_PEAK_TEXT = "1024M"


def seed_ended_attempt(world, key, ended_at, outcome, memory_peak=None, memory_cap=None):
    """往这一格服务自己的库里记一次结束了的尝试，结束时刻照给的（World.seed_attempt 的结束时刻固定，排不出先后）。
    """
    record = {"command_key": key, "outcome": outcome, "memory_peak": memory_peak, "memory_cap": memory_cap, "task_id": 0, "number": 1,
              "host": "local", "state": "ended", "session": "", "ended_at": ended_at, "started_at": ended_at - 1,
              "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC}
    world.seeded.append(record)
    world.flush_seeded()


def cell_cap_hit_low_peak_not_recorded_at_cap(world):
    """撞上限那一次峰值远低于上限（task_server.py 的 cap_hit_measurement；任务 10731：上限 30.4G、峰值 1.0G）：同键上一次上限 13824M、峰值 1G 记了撞上限，
    新提交的头一次按峰值起在 1G（1024M）、跑完。弄坏开关 cap-hit-low-peak-counted-at-cap 下照上限起在 13824M，这一格红。"""
    world.resources(local=(40, 40))
    key = "selftest-cap-hit-low-peak"
    world.start_server()
    seed_ended_attempt(world, key, SEEDED_ENDED_AT, "memory-cap-hit", memory_cap=HISTORY_CAP_HIT_BYTES, memory_peak=HISTORY_SMALL_PEAK_BYTES)
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(summary)
    if summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != HISTORY_SMALL_PEAK_TEXT:
        raise SelftestFailure(f"同键上一次上限 13824M、峰值 1G 记了撞上限：新提交的头一次应当起在 {HISTORY_SMALL_PEAK_TEXT}、跑完，实际上限 {attempt.get('memory_cap_text')}、{describe(summary)}",
                              world.server_log_lines())
    return f"撞上限那一次峰值远低于上限：新提交的头一次按峰值起在 {HISTORY_SMALL_PEAK_TEXT}"


def cell_history_peak_first_attempt(world):
    """起点能降（task_server.py 的 recorded_prediction 只取近 RECORD_WINDOW 次）：同键早先撞过 13824M，
    之后 RECORD_WINDOW 次跑成的峰值都是 1G：撞过的那一次出了窗口，新提交的头一次回落到 1G（1024M）、prediction 记 record，跑完。"""
    world.resources(local=(40, 40))
    key = "selftest-history-first"
    world.start_server()
    seed_ended_attempt(world, key, SEEDED_ENDED_AT - 100, "memory-cap-hit", memory_cap=HISTORY_CAP_HIT_BYTES)
    for offset in range(5):   # task_server.py 的 RECORD_WINDOW：近几次的名额全是小峰值
        seed_ended_attempt(world, key, SEEDED_ENDED_AT - 50 + offset, "succeeded", memory_peak=HISTORY_SMALL_PEAK_BYTES)
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(summary)
    if (summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != HISTORY_SMALL_PEAK_TEXT
            or attempt.get("prediction") != lib.RECORD_HISTORY):
        raise SelftestFailure(f"同键早先撞过 13824M、之后五次跑成都是 1G：撞过的那一次出了窗口，新提交的头一次应当回落到 {HISTORY_SMALL_PEAK_TEXT}、"
                              f"prediction 记 {lib.RECORD_HISTORY}、跑完，实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、{describe(summary)}",
                              world.server_log_lines())
    return f"同键早先撞过 13824M、之后五次跑成都是 1G：新提交的头一次回落到 {HISTORY_SMALL_PEAK_TEXT}"


def cell_history_peak_window_raises(world):
    """起点能升（task_server.py 的 recorded_prediction）：同键近 RECORD_WINDOW 次里有一次撞过 13824M（之后又跑成四次 1G，撞过的那一次还在窗口里），
    新提交的头一次上限 13824M、prediction 记 record，跑完。弄坏开关 cap-hit-not-counted 下撞过的那一次不进定额度、起在 1024M，这一格红。"""
    world.resources(local=(40, 40))
    key = "selftest-history-window"
    world.start_server()
    seed_ended_attempt(world, key, SEEDED_ENDED_AT - 100, "memory-cap-hit", memory_cap=HISTORY_CAP_HIT_BYTES)
    for offset in range(4):   # task_server.py 的 RECORD_WINDOW 是 5：撞过的那一次与四次小峰值都在窗口里
        seed_ended_attempt(world, key, SEEDED_ENDED_AT - 50 + offset, "succeeded", memory_peak=HISTORY_SMALL_PEAK_BYTES)
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(summary)
    if (summary["task"].get("outcome") != "succeeded" or attempt.get("memory_cap_text") != HISTORY_CAP_HIT_TEXT
            or attempt.get("prediction") != lib.RECORD_HISTORY):
        raise SelftestFailure(f"同键近五次里撞过 13824M、另四次跑成 1G：新提交的头一次应当起在 {HISTORY_CAP_HIT_TEXT}、prediction 记 {lib.RECORD_HISTORY}、跑完，"
                              f"实际上限 {attempt.get('memory_cap_text')}、prediction {attempt.get('prediction')}、{describe(summary)}", world.server_log_lines())
    return f"窗口里刚撞过 13824M：新提交的头一次起在 {HISTORY_CAP_HIT_TEXT}"


def cell_history_peak_unrecorded_default(world):
    """没有任何记录的命令从没记录的 4096M 起（task_server.py 的 unrecorded_prediction）：prediction 记 default。
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
        wanted = PEAKS.command_key([], parent_key + lib.shard_child_key("", record.get("role"), int(record.get("shard_index") or 0), int(record.get("shard_count") or 0),
                                                                        compiles=bool(record.get("compiles"))))
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


def write_unmet_script(world, name):
    """工作目录里一份会编的声明写坏了（# task-server-compiles: maybe）的假脚本：派活时打回 run-condition-unmet（task_server.py 的 judge_placement）；交提交它的命令。"""
    with open(os.path.join(world.work, name), "w", encoding="utf-8") as handle:
        handle.write(f"#!/usr/bin/env bash\n{lib.DECLARED_COMPILES_MARK} maybe\nexit 0\n")
    return ["bash", name, "run", "fake-wrapper-exits=0"]


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



def cell_health_head_cpu_gate_not_starved(world):
    """/health 只读派活给排头的结论（task_server.py 的 starved_problem 读 head_verdict；任务 7568 卡在 CPU 负载闸被报「放得下却没起」）：本机内存够、负载 200
    （32 核 × 2 = 64 放不下再加一条的线程），排头（命令键记过 31 核，不走要核数少的名额）排着、排队原因写「CPU：」；过了阈值四倍那么久 /health 一直回 ok：派活判它不能起，不报；
    看进展（progress_problem）照余量也不报（这台负载过 CPU 闸）。负载降下来之后跑完。
    弄坏开关 health-head-judged-apart、health-judges-head 下 /health 另判、只判内存，blocked-head-counted-as-starved 下判不能起也算，
    progress-gates-ignored 下看进展不看 CPU 闸，/health 都报，这一格红。"""
    world.resources(local=(50, 50), local_cpu={"load1": 200.0, "procs_running": 150, "cpus": 32})
    seed_busy_cores(world, CPU_GATE_BUSY_KEY)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "cpu-gate-head"], extra={lib.KEY_VARIABLE: CPU_GATE_BUSY_KEY})
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


                                                     # 整片之外第二台照样空着 8G


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


CLAMP_HOST_SLICE_GIBIBYTES = 50          # cap-hit-record-over-every-host-starts-clamped：只有本机、slice 总上限 50G、假余量 2G：可用上限 48G
CLAMP_CAP_HIT_BYTES = 45 * GIBIBYTE       # 同键上一次撞的上限（峰值到了上限）、之后没跑完过：进定额度的量按它乘 1.5 是 67.5G，比 48G 大
CLAMP_HOST_LIMIT_TEXT = "49152M"          # 48G：总上限 50G 减余量 2G
CLAMP_TINY_SLICE_GIBIBYTES = 5            # booking-within-host-limit-every-history：本机 slice 总上限 5G、假余量 2G，可用上限 3G；四种来历的预测都比它大
CLAMP_PINNED_ITEM_BYTES = 8 * GIBIBYTE    # 钉值那一条：多机配置里这台那一项 8G，减余量 2G 是 6G
CLAMP_RECORDED_BYTES = 7 * GIBIBYTE       # 记录那一条：近几次最大 7G
CLAMP_RETRY_BYTES = 4608 * MEBIBYTE       # 撞上限重排那一条：3072M 撞了乘 1.5


def cell_cap_hit_record_over_every_host_starts_clamped(world):
    """记录比每一台都大照收到可用上限起跑（task_server.py 文件头 ②「内存」的准入记账；booked_at_cap、hosts_short_of_record）：只有本机、slice 总上限 50G、余量 2G
    （可用上限 48G），同键上一次撞了 45G 的上限（峰值到了上限）、之后这把键没跑完过：进定额度的量是上限乘 1.5 的 67.5G，比本机大，也没有别的台；
    新交的一条不判 never-fits，在本机按 48G（49152M，总上限减余量）起跑、跑完，尝试行的上限与记账都不大于 48G。
    弄坏开关 booking-over-host-limit 下记账照 67.5G 记、judge_host 判永远放不下，record-over-every-host-never-fits 下本机照永远放不下判，都记 never-fits，这一格红。"""
    key = "selftest-record-over-every-host"
    world.resources(local=(100, CLAMP_HOST_SLICE_GIBIBYTES))
    world.start_server()
    seed_ended_attempt(world, key, SEEDED_ENDED_AT, "memory-cap-hit", memory_cap=CLAMP_CAP_HIT_BYTES, memory_peak=CLAMP_CAP_HIT_BYTES)
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "fake-wrapper-exits=0"], extra={lib.KEY_VARIABLE: key}))
    attempt = first_attempt(summary)
    limit = (CLAMP_HOST_SLICE_GIBIBYTES - SELFTEST_MARGIN_GIBIBYTES) * GIBIBYTE
    if (summary["task"].get("outcome") != "succeeded" or len(summary["attempts"]) != 1 or attempt.get("host") != "local"
            or attempt.get("memory_cap_text") != CLAMP_HOST_LIMIT_TEXT or int(attempt.get("memory_cap") or 0) > limit or int(attempt.get("booked") or 0) > limit):
        raise SelftestFailure(f"同键撞过 45G、之后没跑完过（进定额度 67.5G），只有本机、可用上限 48G：应当在本机按 {CLAMP_HOST_LIMIT_TEXT} 起跑一次、跑完，"
                              f"上限与记账不大于 48G（{limit} 字节），不判 never-fits；实际台 {attempt.get('host')}、上限 {attempt.get('memory_cap_text')}"
                              f"（{attempt.get('memory_cap')} 字节）、记账 {attempt.get('booked')} 字节、{describe(summary)}、说明「{summary['task'].get('note')}」",
                              world.server_log_lines())
    return f"进定额度 67.5G 比本机可用上限 48G 大：照 {CLAMP_HOST_LIMIT_TEXT} 起跑、跑完，不判 never-fits"


def cell_booking_within_host_limit_every_history(world):
    """任何来历算出的上限与记账都不大于判的那一台的可用上限（task_server.py 的 booked_at_cap；文件头 ②「内存」的准入记账）：本机 slice 总上限 5G、余量 2G，可用上限 3G；
    钉值（多机配置那一项 8G 减余量是 6G）、记录（7G）、没有记录的缺省（UNRECORDED_MEMORY_DEFAULT_BYTES，4G）、撞上限重排（4608M）四条的预测都比 3G 大：
    booked_at_cap 交给 judge_host 的预测峰值、上限与记账都不大于 3G。可用上限照这一格自己算（总上限减余量），不读 host_limit。套在假调度器上，不起服务。
    弄坏开关 booking-over-host-limit 下钉值之外的三条照原样记，booked-at-peak 下四条都照原样，booked-up-to-slice-total 下收到 5G（不减余量），这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.host_dispatch = lib.load_module("host_dispatch_for_booking_cell", os.path.join(lib.HOSTS_DIRECTORY, "host-dispatch.py"))
    report = memory_report(50, CLAMP_TINY_SLICE_GIBIBYTES)
    slice_total = int(report["memory"]["slice_total"])
    snapshot = types.SimpleNamespace(reports={SERVER.LOCAL: report}, down={}, memory_caps={}, total=lambda host: slice_total)
    limit = (CLAMP_TINY_SLICE_GIBIBYTES - SELFTEST_MARGIN_GIBIBYTES) * GIBIBYTE
    prediction_type = fake.host_dispatch.Prediction
    default = lib.UNRECORDED_MEMORY_DEFAULT_BYTES
    pinned = CLAMP_PINNED_ITEM_BYTES - SELFTEST_MARGIN_GIBIBYTES * GIBIBYTE
    cases = (("钉值", prediction_type("selftest-booking-pinned", pinned, pinned, pinned, lib.PINNED_HISTORY, memory_by_host={SERVER.LOCAL: CLAMP_PINNED_ITEM_BYTES})),
             ("记录", prediction_type("selftest-booking-record", CLAMP_RECORDED_BYTES, CLAMP_RECORDED_BYTES, CLAMP_RECORDED_BYTES, lib.RECORD_HISTORY)),
             ("没有记录的缺省", prediction_type("selftest-booking-default", default, default, default, lib.UNRECORDED_HISTORY)),
             ("撞上限重排", prediction_type("selftest-booking-retry", CLAMP_RETRY_BYTES, CLAMP_RETRY_BYTES, CLAMP_RETRY_BYTES, "retry")))
    problems = []
    for what, prediction in cases:
        if int(prediction.memory_predicted) <= limit:
            problems.append(f"{what}：样本的预测 {prediction.memory_predicted} 字节应当比可用上限 {limit} 字节大（这一格的样本写坏了）")
            continue
        booked = SERVER.Scheduler.booked_at_cap(fake, prediction, SERVER.LOCAL, snapshot)
        over = {name: int(getattr(booked, name) or 0) for name in ("memory_predicted", "memory_cap", "memory_booked") if int(getattr(booked, name) or 0) > limit}
        if over:
            problems.append(f"{what}（{prediction.memory_history}）：预测峰值、上限与记账都应当不大于可用上限 {limit} 字节，实际超过的 {over}")
    if problems:
        raise SelftestFailure("记账收到可用上限：" + "；".join(problems))
    return f"钉值、记录、缺省、撞上限重排 {len(cases)} 条的预测都比可用上限 3G 大：交给 judge_host 的预测峰值、上限与记账都不大于 3G"


DROP_HISTORY_LIMIT_GIBIBYTES = 1           # control-drop-history-oversized-records：--attempt-records-over-gib 1
DROP_HISTORY_CAP_BYTES = 800 * MEBIBYTE    # 撞上限那几行的上限：本身不过 1G，乘 1.5 的 1200M 过 1G
DROP_HISTORY_NEAR_CAP_PEAK_BYTES = 790 * MEBIBYTE   # 峰值到了上限（不小于一半）：按上限进定额度
DROP_HISTORY_LOW_PEAK_BYTES = 100 * MEBIBYTE        # 峰值远低于上限（不到一半）：按峰值进定额度
DROP_HISTORY_SMALL_PEAK_BYTES = 500 * MEBIBYTE


def drop_history_rows(world):
    """control-drop-history-oversized-records：照服务读回的规则读 attempts，交 [(命令键, 结局)]（照行号先后）。"""
    return [(record.get("command_key"), record.get("outcome")) for record, _ in world.signed_rows("attempts")]


def cell_control_drop_history_oversized_records(world):
    """control drop-history --attempt-records-over-gib（task-server/control 的 drop_history；撞上限那一行进定额度的量照 task_server_lib.py 的 cap_hit_record_bytes 算，
    服务的 cap_hit_measurement 经同一处）：库里造六行——跑完、峰值 2G（过 1G）；撞了 800M 上限、峰值到了上限、之后这把键没跑完过（进定额度 1200M，过 1G）；
    跑完、峰值 500M；撞了 800M 上限而峰值只有 100M（远低于上限，按峰值进 100M）；同一把键撞了 800M 上限、之后又跑完一次 500M（撞的那一行按上限本身进 800M）两行。
    服务在跑（is-active 打 active）时拒：退 1、说服务还没停、一行不删、不拷备份；停了之后 --attempt-records-over-gib 1 删前两行、其余四行都在，
    删之前的库拷在 .before-drop-history- 那一份里、六行都在。
    弄坏开关 drop-history-cap-hits-ignored 下撞上限那一行不删，cap-hit-low-peak-counted-at-cap 下峰值 100M 的那一行也删，这一格红。"""
    world.start_server()
    rows = (("selftest-drop-peak-over", "succeeded", 2 * GIBIBYTE, None),
            ("selftest-drop-cap-hit-over", "memory-cap-hit", DROP_HISTORY_NEAR_CAP_PEAK_BYTES, DROP_HISTORY_CAP_BYTES),
            ("selftest-drop-small", "succeeded", DROP_HISTORY_SMALL_PEAK_BYTES, None),
            ("selftest-drop-cap-hit-low-peak", "memory-cap-hit", DROP_HISTORY_LOW_PEAK_BYTES, DROP_HISTORY_CAP_BYTES),
            ("selftest-drop-cap-hit-resolved", "memory-cap-hit", DROP_HISTORY_NEAR_CAP_PEAK_BYTES, DROP_HISTORY_CAP_BYTES),
            ("selftest-drop-cap-hit-resolved", "succeeded", DROP_HISTORY_SMALL_PEAK_BYTES, None))
    for offset, (key, outcome, peak, cap) in enumerate(rows):
        seed_ended_attempt(world, key, SEEDED_ENDED_AT + offset, outcome, memory_peak=peak, memory_cap=cap)
    world.stop_server()
    seeded = drop_history_rows(world)
    extra = {"TASK_SERVER_CONTROL_SYSTEMCTL": os.path.join(world.bin, "fake-systemctl")}
    state_file = os.path.join(world.world, "systemctl-state")
    backup_prefix = lib.DATABASE_NAME + ".before-drop-history-"
    arguments = ["drop-history", "--attempt-records-over-gib", str(DROP_HISTORY_LIMIT_GIBIBYTES)]
    problems = []
    if len(seeded) != len(rows):
        problems.append(f"造的 {len(rows)} 行应当都进了库，实际读回 {seeded}")
    with open(state_file, "w", encoding="utf-8") as handle:
        handle.write("active\n")
    refused = world.control(arguments, extra=extra)
    refused_text = refused.stdout + refused.stderr
    backups = [name for name in os.listdir(world.state) if name.startswith(backup_prefix)]
    if refused.returncode != 1 or "服务还没停干净" not in refused_text or drop_history_rows(world) != seeded or backups:
        problems.append(f"服务在跑（is-active 打 active）时应当拒：退 1、说服务还没停干净、一行不删、不拷备份；实际退 {refused.returncode}、"
                        f"库里 {drop_history_rows(world)}、备份 {backups}、输出「{refused_text.strip()[-300:]}」")
    with open(state_file, "w", encoding="utf-8") as handle:
        handle.write("inactive\n")
    dropped = world.control(arguments, extra=extra)
    dropped_text = dropped.stdout + dropped.stderr
    left = drop_history_rows(world)
    wanted = [row for row in seeded if row[0] not in ("selftest-drop-peak-over", "selftest-drop-cap-hit-over")]
    if dropped.returncode != 0 or left != wanted or f"进定额度超过 {DROP_HISTORY_LIMIT_GIBIBYTES} GiB 的尝试 2 行" not in dropped_text:
        problems.append(f"停了之后 --attempt-records-over-gib {DROP_HISTORY_LIMIT_GIBIBYTES} 应当退 0、只删峰值 2G 的与撞了 800M 上限而之后没跑完过的那两行（报删了 2 行），"
                        f"峰值 500M 的、峰值远低于上限的撞上限那一行、之后跑完过的撞上限那一行与它跑完的那一行都留着；实际退 {dropped.returncode}、库里 {left}、"
                        f"输出「{dropped_text.strip()[-400:]}」")
    backups = [name for name in os.listdir(world.state) if name.startswith(backup_prefix)]
    if len(backups) != 1:
        problems.append(f"删之前的库应当拷一份 {backup_prefix}<时刻>，实际 {backups}")
    else:
        with contextlib.closing(sqlite3.connect(os.path.join(world.state, backups[0]), timeout=30)) as connection:
            kept = connection.execute("SELECT COUNT(*) FROM attempts").fetchone()[0]
        if kept != len(rows):
            problems.append(f"备份 {backups[0]} 里应当是删之前的 {len(rows)} 行，实际 {kept} 行")
    if problems:
        raise SelftestFailure("drop-history 删进定额度超限的尝试行：" + "；".join(problems))
    return f"服务在跑时拒；停了之后删进定额度超过 {DROP_HISTORY_LIMIT_GIBIBYTES} GiB 的两行、留四行、备份六行都在"


def cell_oversized_budget_reported(world):
    """预算贴着整台（task_server.py 文件头 ①「要处置的提醒」的 oversized-budget）：两台 50G（可用上限 48G、界 40.8G），记过峰值 45G 的命令记账 45G（不乘 1.5）：不超过可用上限、不小于整片界，
    照常排、起来跑；wait 打一行 kind=oversized-budget、依据写记账、可用上限与记账从哪来、退 ATTENTION_EXIT；放行之后跑完。
    弄坏开关 oversized-budget-silent 下不报，wait 等到命令自己退，这一格红。"""
    key = "selftest-oversized-budget"
    release = os.path.join(world.root, "release-oversized-budget")
    world.resources(local=(50, 50), peer=(50, 50), peer_cap_gibibytes=50)
    world.seed_peak(key, 45 * GIBIBYTE)
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


def cell_oversized_budget_acknowledged_after_restart(world):
    """换服务之后照跑照样记得上（task_server.py 文件头 ⑥「没处置时」）：预算贴着整台的那一条照 oversized-budget-reported 造（记账 45G、界 40.8G），wait 报过 oversized-budget；
    服务停了再起（budget_notices 只在内存，换上之后这一条没有开着的提醒），提交它的会话 confirm --why 退 0、回「已记照跑（oversized-budget）」。
    弄坏开关 anomaly-acknowledge-open-only 下只认开着的，回 409「不用确认」，这一格红。"""
    key = "selftest-oversized-budget-restart"
    release = os.path.join(world.root, "release-oversized-budget-restart")
    world.resources(local=(50, 50), peer=(50, 50), peer_cap_gibibytes=50)
    world.seed_peak(key, 45 * GIBIBYTE)
    world.start_server()
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    problems, _ = attention_problems(completed, task_id, "oversized-budget", lib.ATTENTION_EXIT)
    world.stop_server()
    world.start_server()
    confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE])
    said = confirmed.stdout + confirmed.stderr
    if confirmed.returncode != 0 or "已记照跑（oversized-budget）" not in said:
        problems.append(f"换服务之后 confirm --why 应当退 0、回「已记照跑（oversized-budget）」，实际退 {confirmed.returncode}：{said[-300:]}")
    open(release, "w").close()
    world.finished(task_id)
    if problems:
        raise SelftestFailure("换服务之后记照跑：" + "；".join(problems), world.server_log_lines())
    return "oversized-budget 报过、服务停了再起，confirm --why 照样记上照跑"


RUNAWAY_RESTART_PID = 601          # 换服务接着数那两格的假空转进程号（进程表是假的：不起真进程、不停真进程）
RUNAWAY_RESTART_OLD_PID = 602      # runaway-reused-pid-counted-afresh 里换服务前后是同一个的那一个
RUNAWAY_RESTART_GAP_SECONDS = 10000.0   # runaway-reused-pid-counted-afresh：换服务之前那两次报离此刻这么久（服务停着的空档）
SHARD_ADVANCE_PARENT_ID = 4242     # advance-episodes-survive-restart 的假分片父任务号


def restart_surviving_scheduler(world, database_name):
    """换服务不丢的那几份（task_server.py 的 RESTART_SURVIVING_STATES）那几格共用的假调度器：这一格临时库上的 Store（同一份库上再建一个就是服务停了再起），
    Scheduler.__init__ 里那几张表与锁照建，照它末尾那样读回库里那几份（restore_restart_surviving_state）；告警记进 alerts。不起服务。"""
    scheduler = object.__new__(SERVER.Scheduler)
    scheduler.environment = dict(os.environ)
    scheduler.store = SERVER.Store(os.path.join(world.root, database_name), bytes(32), scheduler.environment)
    scheduler.attention_lock = threading.Lock()
    scheduler.runaway_notified, scheduler.advance_failures, scheduler.advance_episodes = {}, {}, {}
    scheduler.suspect_reported, scheduler.suspect_lock, scheduler.health_notified = {}, threading.Lock(), {}
    scheduler.booking_drift_notified, scheduler.booking_drift_lock = {}, threading.Lock()
    scheduler.restart_surviving_lock, scheduler.restart_surviving_written = threading.Lock(), {}
    scheduler.alerts = []
    scheduler.alert = lambda kind, task_id, line, howto: scheduler.alerts.append((kind, task_id, line))
    SERVER.Scheduler.restore_restart_surviving_state(scheduler)
    return scheduler


def runaway_restart_scheduler(world, database_name, listing):
    """restart_surviving_scheduler 再套上 watch_runaway_processes 读写的几样：进程表是这一格给的那一份、所属会话认得出（SESSION_A）、发消息与按号停都是假的，
    记下发了什么（sent）、停了哪几个（stopped）；隔多久再报是 0（每扫一次都到点）。"""
    scheduler = restart_surviving_scheduler(world, database_name)
    scheduler.runaway_renotify_seconds = 0.0
    scheduler.sent, scheduler.stopped = [], []
    scheduler.process_listings = lambda: [(SERVER.LOCAL, listing)]
    scheduler.runaway_owner = lambda pid, listing_text: SESSION_A
    scheduler.runaway_targets = lambda owner: [{"id": 1, "session": owner}]
    scheduler.send_to_session = lambda task, text, follow: scheduler.sent.append(text)

    def stop_runaway_process(host, pid):   # 假的按号停：只记下停了谁，回「回读已不在」
        scheduler.stopped.append(pid)
        return True

    scheduler.stop_runaway_process = stop_runaway_process
    return scheduler


def runaway_listing_line(pid, elapsed_seconds):
    """假进程表的一行（lib.parsed_process_list 的列）：登录会话 scope 里、已跑 elapsed_seconds 秒、累计 CPU 比它少 100 秒（够 RUNAWAY_MINIMUM_CPU_SECONDS 与占比）。"""
    return f"  {pid} 1 {elapsed_seconds} {elapsed_seconds - 100} R 0::/user.slice/session-3.scope spinning\n"


def cell_runaway_reports_survive_restart(world):
    """任务之外的空转进程报过几次换服务后接着数（task_server.py 的 watch_runaway_processes、restore_restart_surviving_state）：认得出所属会话的那一个
    扫两次报两次（RUNAWAY_REPORTS_BEFORE_STOP），服务停了再起（同一份库上另建调度器），下一次扫到就按号停、发「已停」，不再从零报两次。
    套在假调度器上，进程表与停进程都是假的，不起服务。弄坏开关 runaway-reports-memory-only 下换服务后又报一次、不停，这一格红。"""
    listing = runaway_listing_line(RUNAWAY_RESTART_PID, 7200)
    before = runaway_restart_scheduler(world, "runaway-restart.sqlite3", listing)
    for _ in range(SERVER.RUNAWAY_REPORTS_BEFORE_STOP):
        SERVER.Scheduler.watch_runaway_processes(before)
    after = runaway_restart_scheduler(world, "runaway-restart.sqlite3", listing)
    SERVER.Scheduler.watch_runaway_processes(after)
    problems = []
    if len(before.sent) != SERVER.RUNAWAY_REPORTS_BEFORE_STOP or before.stopped:
        problems.append(f"换服务之前应当报 {SERVER.RUNAWAY_REPORTS_BEFORE_STOP} 次、不停，实际报 {len(before.sent)} 次、停了 {before.stopped}")
    if after.stopped != [RUNAWAY_RESTART_PID] or len(after.sent) != 1 or "已停" not in after.sent[0]:
        problems.append(f"换服务后应当接着数、这一次就按号停 {RUNAWAY_RESTART_PID} 并发一条「已停」，实际停了 {after.stopped}、发了 {[text[:80] for text in after.sent]}")
    if problems:
        raise SelftestFailure("空转进程的报数换服务不丢：" + "；".join(problems))
    return f"报过 {SERVER.RUNAWAY_REPORTS_BEFORE_STOP} 次的进程换服务后接着数，下一次扫到就按号停"


def cell_runaway_reused_pid_counted_afresh(world):
    """换服务的空档里同号换了进程的从零数（task_server.py 的 watch_runaway_processes）：两个进程各报过两次，那两次离此刻 RUNAWAY_RESTART_GAP_SECONDS 秒（服务停着的空档）；
    服务再起时 602 已跑得比那两次早（同一个），按号停；601 已跑的时长短于那段空档（上次报它之后才起的，是号被复用的另一个），从零数、报一次、不停。
    套在假调度器上，进程表与停进程都是假的，不起服务。弄坏开关 runaway-reused-pid-carried 下 601 照号接着数、一露面就被停，这一格红。"""
    before_listing = runaway_listing_line(RUNAWAY_RESTART_PID, 7200) + runaway_listing_line(RUNAWAY_RESTART_OLD_PID, 7200)
    before = runaway_restart_scheduler(world, "runaway-reused.sqlite3", before_listing)
    for _ in range(SERVER.RUNAWAY_REPORTS_BEFORE_STOP):
        SERVER.Scheduler.watch_runaway_processes(before)
    for entry in before.runaway_notified.values():
        entry["reported_at"] -= RUNAWAY_RESTART_GAP_SECONDS
    SERVER.Scheduler.persist_restart_surviving_state(before, "runaway_notified")
    after_listing = (runaway_listing_line(RUNAWAY_RESTART_PID, int(RUNAWAY_RESTART_GAP_SECONDS // 2))
                     + runaway_listing_line(RUNAWAY_RESTART_OLD_PID, int(RUNAWAY_RESTART_GAP_SECONDS * 2)))
    after = runaway_restart_scheduler(world, "runaway-reused.sqlite3", after_listing)
    SERVER.Scheduler.watch_runaway_processes(after)
    reported = [text for text in after.sent if f"本机进程 {RUNAWAY_RESTART_PID}" in text.split("\n", 1)[0] and "已停" not in text]
    problems = []
    if after.stopped != [RUNAWAY_RESTART_OLD_PID]:
        problems.append(f"换服务后只该停跑得比上次报早的 {RUNAWAY_RESTART_OLD_PID}，号被复用的 {RUNAWAY_RESTART_PID} 不停，实际停了 {after.stopped}")
    if len(reported) != 1 or after.runaway_notified.get((SERVER.LOCAL, RUNAWAY_RESTART_PID), {}).get("count") != 1:
        problems.append(f"号被复用的 {RUNAWAY_RESTART_PID} 应当从零数、报一次（count 1），实际报了 {len(reported)} 次、"
                        f"账 {after.runaway_notified.get((SERVER.LOCAL, RUNAWAY_RESTART_PID))}")
    if problems:
        raise SelftestFailure("换服务的空档里同号换了进程：" + "；".join(problems))
    return f"同一个的 {RUNAWAY_RESTART_OLD_PID} 接着数、按号停；号被复用的 {RUNAWAY_RESTART_PID} 从零数、报一次"


def cell_advance_episodes_survive_restart(world):
    """分片父任务推不动的这一回与开始过几回换服务后照接（task_server.py 的 note_advance_failure、note_advance_recovered、restore_restart_surviving_state）：
    推不动一回、推通、再推不动连着两轮（第 2 回、count 2）；服务停了再起（同一份库上另建调度器）：读回的是第 2 回、count 2，再推不动 count 3，推通之后再推不动是第 3 回。
    套在假调度器上，不起服务。弄坏开关 advance-episodes-memory-only 下换服务后从第 1 回数起，这一格红。"""
    parent = SHARD_ADVANCE_PARENT_ID
    before = restart_surviving_scheduler(world, "shard-advance.sqlite3")
    SERVER.Scheduler.note_advance_failure(before, parent, "推不动：头一回")
    SERVER.Scheduler.note_advance_recovered(before, parent)
    SERVER.Scheduler.note_advance_failure(before, parent, "推不动：第二回头一轮")
    SERVER.Scheduler.note_advance_failure(before, parent, "推不动：第二回第二轮")
    after = restart_surviving_scheduler(world, "shard-advance.sqlite3")
    restored = dict(after.advance_failures.get(parent) or {})
    SERVER.Scheduler.note_advance_failure(after, parent, "推不动：第二回第三轮")
    continued = dict(after.advance_failures.get(parent) or {})
    SERVER.Scheduler.note_advance_recovered(after, parent)
    SERVER.Scheduler.note_advance_failure(after, parent, "推不动：第三回")
    third = dict(after.advance_failures.get(parent) or {})
    problems = []
    if restored != {"episode": 2, "why": "推不动：第二回第二轮", "count": 2}:
        problems.append(f"换服务后读回的应当是第 2 回、连着 2 轮，实际 {restored}")
    if continued.get("episode") != 2 or continued.get("count") != 3:
        problems.append(f"换服务后再推不动应当还是第 2 回、count 3，实际 {continued}")
    if third.get("episode") != 3 or after.advance_episodes.get(parent) != 3:
        problems.append(f"推通之后再推不动应当是第 3 回，实际 {third}、开始过 {after.advance_episodes.get(parent)} 回")
    if problems:
        raise SelftestFailure("推不动的回数换服务不丢：" + "；".join(problems))
    return "推不动的这一回（第 2 回、count 2）换服务后照接，推通之后再推不动是第 3 回"


REMINDER_RESTART_HEALTH = [("database-unwritable", "库写不进", "试写一行失败")]   # reminders-not-resent-after-restart：服务健康查出的那一样（种类、名字、问题）


def reminder_restart_scheduler(world, database_name, drifts=None):
    """reminders-not-resent-after-restart 的假调度器：restart_surviving_scheduler（同一份临时库上再建一个就是服务停了再起）再套上发消息那几路读写的几样——
    发消息记下发了什么（sent），收件人只有 SESSION_A，服务健康查出 REMINDER_RESTART_HEALTH 那一样，记账漂移判出 drifts（{(种类, 台): 条目}）。"""
    scheduler = restart_surviving_scheduler(world, database_name)
    scheduler.sent = []
    scheduler.inbox_lock, scheduler.inbox_failed = threading.Lock(), set()
    scheduler.send_to_session = lambda task, text, event: scheduler.sent.append(text) or True
    scheduler.inbox_reachable = lambda task: True
    scheduler.health_recipients = lambda: {SESSION_A: {"id": 1, "session": SESSION_A}}
    scheduler.health_problems = lambda: list(REMINDER_RESTART_HEALTH)
    scheduler.booking_drift = {"memory": dict(drifts or {})}
    scheduler.booking_audit_state = {}
    scheduler.drift_recipients = lambda: {SESSION_A: {"id": 1, "session": SESSION_A}}
    return scheduler


def cell_reminders_not_resent_after_restart(world):
    """推给会话的提醒发过一回，服务停了再起（同一份库上另建调度器），同一件不再发；跨过下一道提醒门槛、好了又坏才再发（task_server.py 的 push_suspects、watch_health、push_booking_drift、
    forget_drift、persist_restart_surviving_state、restore_restart_surviving_state；任务 13493 的子任务 13522 的 suspected-stall 同一段、依据一字不差，同一天到了三回，后两回各落在服务重启之后）。
    三种去重状态原先只在内存里，现在都记进库：疑似卡住报过的段（suspect_reported）、服务健康每一样发给了哪几个会话（health_notified）、记账漂移每一件推给每个会话推到了哪道提醒门槛
    （booking_drift_notified）。其余的提醒去重状态本来就在库里：要处置的提醒报到的门槛与推过的几回在任务行（reminded_thresholds、reminder_history）、结局没人取推到第几回在任务行（outcome_repushed）、
    空转进程的报数、分片推不动的回数、停用的卡在换服务不丢的那几份；发不出去过的事件（inbox_failed）是没送到的，重启后再试不算重发。
    三趟：疑似卡住第 1 段发过一回，换服务后同一段不再发，第 2 段发；服务健康一样发过一回，换服务后不再发，好了（这一轮查不出）之后又坏再发；记账漂移判出当场推过一回，换服务后同一道门槛不再推，
    过了门槛（BOOKING_DRIFT_PUSH_SECONDS）再推一回。套在假调度器上，不起服务。
    弄坏开关 suspect-reported-memory-only、health-notified-memory-only、booking-drift-notified-memory-only 下各自那一趟换服务后又发一回，这一格红。"""
    problems = []
    store = SERVER.Store(os.path.join(world.root, "reminder-restart.sqlite3"), bytes(32), dict(os.environ))
    task = store.insert("tasks", {"task_id": 0, "state": "running", "session": SESSION_A, "command_key": "", "inbox": {"token": INBOX_TOKEN_SAMPLE}})
    attempt = store.insert("attempts", {"task_id": task["id"], "number": 1, "state": "running", "session": SESSION_A, "command_key": "", "host": SERVER.LOCAL, "log": "x.log",
                                        "suspect": {"active": True, "segment": 1, "why": "整个单元的 CPU 这 600 秒平均 0.0001 个核"}})
    del store
    before = reminder_restart_scheduler(world, "reminder-restart.sqlite3")
    SERVER.Scheduler.push_suspects(before, task)
    after = reminder_restart_scheduler(world, "reminder-restart.sqlite3")
    SERVER.Scheduler.push_suspects(after, task)
    after.store.update("attempts", dict(after.store.attempts_of(task["id"])[0][0], suspect={"active": True, "segment": 2, "why": "整个单元的 CPU 又静了 600 秒"}))
    SERVER.Scheduler.push_suspects(after, task)
    if len(before.sent) != 1 or len(after.sent) != 1 or "第 2 段" not in after.sent[0].split("\n", 1)[0]:
        problems.append(f"疑似卡住：第 1 段发过一回、换服务后同一段不再发、第 2 段发一回，实际换服务前发 {len(before.sent)} 回、换服务后发 {[text.split(chr(10), 1)[0] for text in after.sent]}")
    before = reminder_restart_scheduler(world, "reminder-health.sqlite3")
    SERVER.Scheduler.watch_health(before)
    after = reminder_restart_scheduler(world, "reminder-health.sqlite3")
    SERVER.Scheduler.watch_health(after)
    after.health_problems = lambda: []
    SERVER.Scheduler.watch_health(after)
    after.health_problems = lambda: list(REMINDER_RESTART_HEALTH)
    SERVER.Scheduler.watch_health(after)
    if len(before.sent) != 1 or len(after.sent) != 1:
        problems.append(f"服务健康：发过一回、换服务后不再发、好了又坏再发一回，实际换服务前发 {len(before.sent)} 回、换服务后共发 {len(after.sent)} 回")
    entry = {"since": 1000.0, "rounds": 3, "booked": 0, "used": 0, "attempts": [], "text": "本机内存记账漂移"}
    before = reminder_restart_scheduler(world, "reminder-drift.sqlite3", {"本机": entry})
    SERVER.Scheduler.push_booking_drift(before, 1000.0)
    after = reminder_restart_scheduler(world, "reminder-drift.sqlite3", {"本机": entry})
    SERVER.Scheduler.push_booking_drift(after, 1000.0 + 1)
    SERVER.Scheduler.push_booking_drift(after, 1000.0 + SERVER.BOOKING_DRIFT_PUSH_SECONDS * 1.5)
    if len(before.sent) != 1 or len(after.sent) != 1 or "再推一回" not in after.sent[0].split("\n", 1)[0]:
        problems.append(f"记账漂移：判出当场推过一回、换服务后同一道门槛不再推、过了门槛再推一回（首行写再推一回），实际换服务前推 {len(before.sent)} 回、换服务后推 {[text.split(chr(10), 1)[0] for text in after.sent]}")
    if problems:
        raise SelftestFailure("提醒换服务后不重发：" + "；".join(problems))
    return "疑似卡住的段、服务健康的会话、记账漂移推到的门槛发过一回，换服务后同一件不再发；新一段、好了又坏、过了门槛才再发"


REMINDER_LEDGER_KIND = "queued-too-long"   # reminder-thresholds-every-doubling 与 reminder-ledger-other-shape-read-afresh：照排队过久那一种算提醒门槛


def reminder_ledger_task(world, database_name, fields=None):
    """那两格的临时库里插一条排着的任务（提交它的会话是 SESSION_A），fields 并进任务行；→ 任务号。"""
    store = SERVER.Store(os.path.join(world.root, database_name), bytes(32), dict(os.environ))
    return store.insert("tasks", dict({"task_id": 0, "state": "queued", "session": SESSION_A, "command_key": ""}, **(fields or {})))["id"]


def queued_reminder_scheduler(world, database_name, waited):
    """那两格的假调度器：restart_surviving_scheduler（同一份临时库上再建一个就是服务停了再起）再备上要处置的提醒读写的两张内存表；
    attention_items 换成照 waited[0]（已排多少秒）与服务的 QUEUE_ATTENTION_MINUTES 现算的一项排队过久（没到门槛的交空表）。"""
    scheduler = restart_surviving_scheduler(world, database_name)
    scheduler.reminded_thresholds, scheduler.reminder_histories = {}, {}
    first = SERVER.QUEUE_ATTENTION_MINUTES * 60

    def items(task_id):
        crossed = SERVER.reminder_threshold(waited[0], first)
        if not crossed:
            return []
        return [SERVER.Scheduler.attention_item(scheduler, REMINDER_LEDGER_KIND, crossed, task_id, f"已排 {waited[0]:.0f} 秒", "自证：一直排着", "",
                                                first * 2 ** (crossed - 1))]

    scheduler.attention_items = items
    return scheduler


def cell_reminder_thresholds_every_doubling(world):
    """提醒门槛照 QUEUE_ATTENTION_MINUTES（30 分钟）起、每翻一倍一道，同一道只报一次（task_server.py 的 reminder_threshold、next_reminder_seconds、unreported_attention；
    文件头 ①「要处置的提醒」）：已排 0、1799 秒没到门槛；1800、3599 秒跨过一道、下一回在 3600 秒；3600、7199 秒两道、下一回 7200 秒；7200 秒三道、下一回 14400 秒。
    套在假调度器上（queued_reminder_scheduler），不起服务：已排 0 到 7300 秒每隔 100 秒问一回 wait 那一侧，只在 1800、3600、7200 秒各报一回，
    报的那一项 reminded_after_seconds 依次是 1800、3600、7200，其间不重发。弄坏开关 attention-reported-every-wait 下每一回都报，这一格红。"""
    first = SERVER.QUEUE_ATTENTION_MINUTES * 60
    wanted = {0: (0, 1800), 1799: (0, 1800), 1800: (1, 3600), 3599: (1, 3600), 3600: (2, 7200), 7199: (2, 7200), 7200: (3, 14400)}
    found = {seconds: (SERVER.reminder_threshold(seconds, first), SERVER.next_reminder_seconds(SERVER.reminder_threshold(seconds, first), first)) for seconds in wanted}
    problems = [] if found == wanted else [f"从 1800 秒起每翻一倍一道门槛：{{已排秒数: (跨过几道, 下一回在几秒)}} 应当 {wanted}，实际 {found}"]
    task_id = reminder_ledger_task(world, "reminder-thresholds.sqlite3")
    waited = [0.0]
    scheduler = queued_reminder_scheduler(world, "reminder-thresholds.sqlite3", waited)
    reported = []
    for seconds in range(0, 7301, 100):
        waited[0] = float(seconds)
        reported += [(seconds, item.get("reminded_after_seconds")) for item in SERVER.Scheduler.unreported_attention(scheduler, task_id, SESSION_A)]
    if reported != [(1800, 1800), (3600, 3600), (7200, 7200)]:
        problems.append(f"每隔 100 秒问一回 wait 那一侧，应当只在 1800、3600、7200 秒各报一回，实际（已排秒数, reminded_after_seconds）{reported}")
    if problems:
        raise SelftestFailure("提醒门槛每翻一倍一道：" + "；".join(problems))
    return "门槛 1800、3600、7200 秒（下一回各在 3600、7200、14400 秒）；问到 7300 秒只在这三处各报一回"


def cell_reminder_ledger_other_shape_read_afresh(world):
    """任务行里的提醒记账不是这一版的两栏（task_server.py 的 stored_reminded_thresholds、reminder_history_of、attention_round、unreported_attention；文件头 ①「要处置的提醒」）：
    任务行只有别的写法记的账（attention_reported 记着种类到 2、attention_history 里一回带 tier 键），没有 reminded_thresholds 与 reminder_history；同一份临时库上建假调度器
    （queued_reminder_scheduler，就是服务换上新代码），不起服务：别的写法的账不读、当没报过——已排 3000 秒问 wait 那一侧报一回（reminded_after_seconds=1800），回数从第 1 回起；
    再问不报；排到 3600 秒报一回（reminded_after_seconds=3600）；任务行写下 reminded_thresholds 记着 2；同一份库上再建一个（服务再重启）问 3600 秒不报。问答都不抛错。
    弄坏开关 attention-reported-every-wait 下再问照报，attention-reported-memory-only 下再重启之后照报，这一格红。"""
    other_shape = {"attention_reported": {REMINDER_LEDGER_KIND: 2},
                   "attention_history": {REMINDER_LEDGER_KIND: [{"tier": 2, "repush": 0, "at": time.time() - 600, "basis": "已排 3700 秒", "round": 2}]}}
    task_id = reminder_ledger_task(world, "reminder-ledger.sqlite3", other_shape)
    waited = [3000.0]
    scheduler = queued_reminder_scheduler(world, "reminder-ledger.sqlite3", waited)
    problems = []
    rounds = SERVER.Scheduler.attention_round(scheduler, scheduler.attention_items(task_id)[0])
    asked = [item.get("reminded_after_seconds") for item in SERVER.Scheduler.unreported_attention(scheduler, task_id, SESSION_A)]
    again = SERVER.Scheduler.unreported_attention(scheduler, task_id, SESSION_A)
    waited[0] = 3600.0
    later = [item.get("reminded_after_seconds") for item in SERVER.Scheduler.unreported_attention(scheduler, task_id, SESSION_A)]
    if rounds != (1, None) or asked != [1800] or again or later != [3600]:
        problems.append(f"别的写法的账不读：已排 3000 秒应当第 1 回、报一回 reminded_after_seconds=1800，再问不报，3600 秒报一回 3600；"
                        f"实际（回数, 上一回）{rounds}、头一问 {asked}、再问 {len(again)} 项、3600 秒 {later}")
    found = scheduler.store.task(task_id)
    stored = (found[0] if found is not None and found[1] else {}).get("reminded_thresholds")
    if stored != {REMINDER_LEDGER_KIND: 2}:
        problems.append(f"报过之后任务行应当写下 reminded_thresholds {{{REMINDER_LEDGER_KIND}: 2}}，实际 {stored}")
    restarted = queued_reminder_scheduler(world, "reminder-ledger.sqlite3", waited)
    if SERVER.Scheduler.unreported_attention(restarted, task_id, SESSION_A):
        problems.append("同一份库上再建调度器（服务再重启），已排 3600 秒那一回报过了不该再报，实际又报了")
    if problems:
        raise SelftestFailure("提醒记账不是这一版的两栏：" + "；".join(problems))
    return "别的写法的账不读、当没报过：3000 秒报一回（第 1 回，reminded_after_seconds=1800）、再问不报、3600 秒报一回；任务行写下 reminded_thresholds，再重启不重报"


SPLIT_EXEMPT_ITEM = "selftest-exempt-item"   # long-run-split-reminded-at-outcome：豁免表里点名的项


def long_run_split_scheduler(world, exempt_lines=()):
    """long-run-split-reminded-at-outcome 的假调度器：仓根是这一格自己的目录（豁免表 .claude/long-run-split-exempt 照 exempt_lines 写），任务行与尝试行由调用方给；
    attention_of_record 里出了结局的任务走到的几样（改主工作区的核、预算提醒表）照建。→ (假调度器, 它的仓根)。"""
    root = os.path.join(world.root, f"split-root-{len(os.listdir(world.root))}")
    os.makedirs(os.path.join(root, ".claude"))
    with open(os.path.join(root, ".claude", "long-run-split-exempt"), "w", encoding="utf-8") as handle:
        handle.write("# 跑过两小时也不拆的项，只登记用户点名的\n" + "".join(f"{line}\n" for line in exempt_lines))
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peer_root = dict(os.environ), root
    fake.budget_notices, fake.workspace_write_checks = {}, {}
    return fake, root


def split_items_of(fake, record, attempts):
    """attention_of_record 里 long-run-split 那一项（没有交 None）。"""
    fake.store = types.SimpleNamespace(attempts_of=lambda task_id: [(dict(attempt), True) for attempt in attempts])
    found = [item for item in SERVER.Scheduler.attention_of_record(fake, record, time.time()) if item["kind"] == "long-run-split"]
    return found[0] if found else None


def cell_long_run_split_reminded_at_outcome(world):
    """跑过 7200 秒的项出结局时推一条要拆的提醒（task_server.py 的 attention_of_record、long_run_split_items、settled_run_seconds、long_run_split_exempt；.claude/rules/long-run-split.md「判据」「豁免」）：
    出了结局的任务服务记的用时（各次尝试记下的 seconds，撞了内存上限、照大一档重来的从最后一次撞上限之后那一次起算）超过 LONG_RUN_SPLIT_SECONDS 又不在豁免表里的，交一项 long-run-split，
    依据写这是按片或按整条任务的用时判的，出路指到那份规则。假秒，不真等：整条任务跑了 7300 秒的出一项；7100 秒的不出；命令里点名了豁免表里的项的不出（同一份用时，表里没有就出）；
    撞上限重来的：前两次撞上限各 5000、4000 秒，最后一次跑成 3000 秒的不出（从最后一次撞上限之后那一次起算），最后一次跑成 7300 秒的出；分片的一片跑了 7300 秒的出、依据写第几片、按片判。
    弄坏开关 long-run-split-silent 下一条都不出，long-run-split-cap-hits-counted 下撞上限的那几次也累加（撞上限重来、最后一次 3000 秒的也出），
    long-run-split-exempt-ignored 下豁免表不看（点名了豁免项的也出），这一格红。"""
    def attempt(number, seconds, outcome="succeeded"):
        return {"number": number, "seconds": seconds, "outcome": outcome, "state": "ended"}

    problems = []
    command = ["bash", "scripts/gate/some-entry.sh", "--item", "ordinary-item"]
    exempt_command = ["bash", "scripts/gate/some-entry.sh", "--item", SPLIT_EXEMPT_ITEM]
    record = {"id": 701, "state": "finished", "outcome": "succeeded", "exit_code": 0, "command": command}
    fake, _ = long_run_split_scheduler(world, [f"{SPLIT_EXEMPT_ITEM}  # 用户点名豁免（records/selftest.md:1）"])
    item = split_items_of(fake, record, [attempt(1, 7300.0)])
    if item is None or item["threshold"] != 1 or "整条任务" not in item["basis"] or "按整条任务的用时判" not in item["basis"] or ".claude/rules/long-run-split.md" not in item["next"]:
        problems.append(f"整条任务跑了 7300 秒：应当出一项 long-run-split（提醒门槛 1、依据写按整条任务的用时判、出路指到 long-run-split.md），实际 {item}")
    if split_items_of(fake, record, [attempt(1, 7100.0)]) is not None:
        problems.append("整条任务跑了 7100 秒：不到界，不该出")
    if split_items_of(fake, dict(record, command=exempt_command), [attempt(1, 7300.0)]) is not None:
        problems.append(f"命令点名了豁免表里的项 {SPLIT_EXEMPT_ITEM}：不该出")
    reset_hits = [attempt(1, 5000.0, "memory-cap-hit"), attempt(2, 4000.0, "memory-cap-hit")]
    if split_items_of(fake, record, reset_hits + [attempt(3, 3000.0)]) is not None:
        problems.append("撞上限重来的（5000、4000 秒各撞一次上限，最后一次跑成 3000 秒）：从最后一次撞上限之后那一次起算，3000 秒不到界，不该出")
    if split_items_of(fake, record, [attempt(1, 9000.0, "memory-cap-hit"), attempt(2, 7300.0)]) is None:
        problems.append("撞一次上限（9000 秒）之后最后一次跑成 7300 秒：从最后一次撞上限之后那一次起算 7300 秒，应当出")
    shard = split_items_of(fake, dict(record, id=702, parent_id=700, role="shard", shard_index=3), [attempt(1, 7300.0)])
    if shard is None or "第 3 片" not in shard["basis"] or "按片的用时判" not in shard["basis"]:
        problems.append(f"分片的第 3 片跑了 7300 秒：应当出一项、依据写第 3 片、按片的用时判，实际 {shard}")
    if problems:
        raise SelftestFailure("跑过 7200 秒的项出结局时推要拆的提醒：" + "；".join(problems))
    return "整条任务 7300 秒出、7100 秒不出、豁免表里的项不出；撞上限重来的从最后一次之后起算；分片的按片判"


def cell_workspace_write_blocked_reminded(world):
    """换钩子被上一趟没写完成的挡住，提醒发给被挡的这个会话（task_server.py 的 attention_of_record、workspace_write_blocked_items；scripts/gate/hook-swap.py 退 7）：
    出了结局的任务命令是 scripts/gate/hook-swap.py（不带 --feed-only）、结局 command-failed 退 7：出一项 workspace-write-blocked，依据点名上一趟疑似是库里更早的哪条被杀的换钩子任务；
    带 --feed-only 的、退 0 的、命令是别的脚本的不出。
    弄坏开关 workspace-write-blocked-unreported 下一条都不出，这一格红。"""
    fake, root = long_run_split_scheduler(world)
    earlier = {"id": 905, "state": "finished", "outcome": "command-failed", "exit_code": 137, "command": ["python3", "scripts/gate/hook-swap.py", "--spec", "old.json"], "cwd": root}
    swap = ["python3", "scripts/gate/hook-swap.py", "--spec", "new.json"]
    problems = []

    def blocked(record):
        found = dict(record, state="finished", cwd=root)
        fake.store = types.SimpleNamespace(attempts_of=lambda task_id: [], select=lambda table, where="1", parameters=(), order="id": [(earlier, True)])
        items = [item for item in SERVER.Scheduler.attention_of_record(fake, found, time.time()) if item["kind"] == "workspace-write-blocked"]
        return items[0] if items else None

    item = blocked({"id": 910, "outcome": "command-failed", "exit_code": 7, "command": swap})
    if item is None or item["threshold"] != 1 or "任务 905" not in item["basis"] or "退 7" not in item["basis"]:
        problems.append(f"换钩子退 7：应当出一项 workspace-write-blocked（依据点名上一趟疑似是任务 905），实际 {item}")
    for what, record in (("带 --feed-only", {"id": 911, "outcome": "command-failed", "exit_code": 7, "command": swap + ["--feed-only"]}),
                         ("退 0", {"id": 912, "outcome": "succeeded", "exit_code": 0, "command": swap}),
                         ("命令是别的脚本", {"id": 913, "outcome": "command-failed", "exit_code": 7, "command": ["python3", "scripts/gate/other.py"]})):
        if blocked(record) is not None:
            problems.append(f"{what}：不该出")
    if problems:
        raise SelftestFailure("换钩子被没写完成的挡住：" + "；".join(problems))
    return "换钩子退 7 出一项 workspace-write-blocked、点名上一趟的任务；--feed-only、退 0、别的脚本不出"


def cell_restart_surviving_tampered_not_adopted(world):
    """库里换服务不丢的那一份被绕过服务改过（task_server.py 的 Store.restart_surviving_state、restore_restart_surviving_state）：分片父任务推不动的一回记进库，
    再直接改库里那一行的正文多记一个父任务开始推不动过一回、签名不动；服务停了再起：那一份不认（一项都不读回）、写一条 record-tampered 告警点名 shard_advance。
    套在假调度器上，不起服务。弄坏开关 restart-surviving-signature-unchecked 下改过的照样读回，这一格红。"""
    database = os.path.join(world.root, "restart-surviving-tampered.sqlite3")
    before = restart_surviving_scheduler(world, os.path.basename(database))
    SERVER.Scheduler.note_advance_failure(before, SHARD_ADVANCE_PARENT_ID, "推不动：头一回")
    meta_name = SERVER.RESTART_SURVIVING_META_PREFIX + "shard_advance"
    with contextlib.closing(sqlite3.connect(database)) as connection:
        row = connection.execute("SELECT value FROM meta WHERE name = ?", (meta_name,)).fetchone()
        if row is None:
            raise SelftestFailure(f"记下推不动的一回之后库里应当有 {meta_name} 那一行，实际没有：改不了它、这一格判不了")
        stored = json.loads(row[0])
        stored["state"]["episodes"].append([SHARD_ADVANCE_PARENT_ID + 1, 1])
        connection.execute("UPDATE meta SET value = ? WHERE name = ?", (lib.canonical_json(stored), meta_name))
        connection.commit()
    after = restart_surviving_scheduler(world, os.path.basename(database))
    problems = []
    if after.advance_failures or after.advance_episodes:
        problems.append(f"签名对不上的那一份不该读回，实际推不动的记着 {after.advance_failures}、开始过的回数 {after.advance_episodes}")
    if not any(kind == "record-tampered" and "shard_advance" in line for kind, _, line in after.alerts):
        problems.append(f"应当写一条 record-tampered 告警点名 shard_advance，实际 {after.alerts}")
    if problems:
        raise SelftestFailure("改过的换服务不丢的那一份：" + "；".join(problems))
    return "库里那一份被绕过服务改过：不读回、写了 record-tampered 告警"


def cell_budget_source_names_folded_key(world):
    """记录来源写出命令键原文，键里有折掉的点名项时写明同一入口各项共用这一份记录、怎么按项记（task_server.py 的 folded_key_note）；没折的只写键原文。
    套在假调度器上，不起服务。弄坏开关 folded-key-unnamed 下两样都不写，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.host_dispatch = lib.load_module("host_dispatch_for_folded_key_cell", os.path.join(lib.HOSTS_DIRECTORY, "host-dispatch.py"))
    token = fake.host_dispatch.peaks.FOLD_TOKENS[0]
    folded = SERVER.Scheduler.folded_key_note(fake, {"command_key": f"bash .claude/gate.d/sample.sh --item {token}"})
    plain = SERVER.Scheduler.folded_key_note(fake, {"command_key": "bash .claude/gate.d/sample.sh --check one"})
    problems = []
    if f"--item {token}」里的 {token}" not in folded or lib.KEY_VARIABLE not in folded or "ITEM_KEPT_CELLS" not in folded:
        problems.append(f"键里折过项时应当写键原文、点名 {token}、出路带 {lib.KEY_VARIABLE} 与 ITEM_KEPT_CELLS，实际「{folded}」")
    if plain != "；命令键「bash .claude/gate.d/sample.sh --check one」":
        problems.append(f"没折的键应当只写键原文，实际「{plain}」")
    if problems:
        raise SelftestFailure("记录来源写不写命令键：" + "；".join(problems))
    return f"记录来源写出命令键原文，折过项的写明各项共用记录与出路（「{folded[:80]}…」）"


LONG_RUN_IDLE_ATTENTION_SECONDS = "2"   # long-run-idle-cores 那几格：跑满这么久就看（换掉 LONG_RUN_ATTENTION_SECONDS 的 600 秒）


def cell_long_run_idle_cores_reported(world):
    """跑得久而核没用起来（task_server.py 文件头 ①「要处置的提醒」的 long-run-idle-cores）：门槛是跑满 600 秒、平均在用核数不到线程数的 24/31（31 线程时 23.9 核算、24.1 核不算，判法 cores_short_of_threads）；
    假 scope 的 CPU 钉着不涨（平均约 0 核），跑满 2 秒（换掉 600 秒），wait 打一行 kind=long-run-idle-cores、依据写跑了多久、给几线程与平均核数，原因问资源运用是否合理、脚本能不能并行或切片，退 ATTENTION_EXIT；
    只提醒、照跑，放行之后跑完。弄坏开关 long-run-idle-silent 下不报，wait 等到命令自己退；long-run-idle-quarter-threshold 下 23.9 核不算，这一格红。"""
    problems = []
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    if SERVER.LONG_RUN_ATTENTION_SECONDS != 600:
        problems.append(f"门槛应当是 600 秒，实际 {SERVER.LONG_RUN_ATTENTION_SECONDS}")
    if not SERVER.Scheduler.cores_short_of_threads(fake, 23.9, 31) or SERVER.Scheduler.cores_short_of_threads(fake, 24.1, 31):
        problems.append("31 线程：平均 23.9 核应当算没用满、24.1 核不算")
    scope = os.path.join(world.root, "long-run-idle-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    release = os.path.join(world.root, "release-long-run-idle")
    world.start_server(extra={"TASK_SERVER_SELFTEST_LONG_RUN_ATTENTION_SECONDS": LONG_RUN_IDLE_ATTENTION_SECONDS, "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release])
    completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
    found, fields = attention_problems(completed, task_id, "long-run-idle-cores", lib.ATTENTION_EXIT)
    problems += found
    basis = fields.get("basis", "")
    if "平均只用" not in basis or "线程" not in basis or "核" not in basis or "已跑" not in basis:
        problems.append(f"依据应当写已跑多久、给几线程与平均只用几核，实际「{basis}」")
    if "资源运用是否合理" not in fields.get("why", "") or "并行或切片" not in fields.get("why", ""):
        problems.append(f"原因应当问资源运用是否合理、脚本能不能并行或切片，实际「{fields.get('why')}」")
    open(release, "w").close()
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"只提醒、照跑：放行之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("跑得久而核没用起来：" + "；".join(problems), world.server_log_lines())
    return f"界是 24/31；跑满 {LONG_RUN_IDLE_ATTENTION_SECONDS} 秒、CPU 不涨：wait 打 kind=long-run-idle-cores（依据「{basis[:80]}」），照跑完"


def attention_of_kind(world, task_id, kind):
    """接着起 wait，等到一行 kind 是这一种的提醒：交那一行拆出的栏；任务先有了结局、起了三回都没等到的交空表。"""
    for _ in range(3):
        completed = world.client(["wait", str(task_id)], timeout=ATTENTION_WAIT_SECONDS)
        for line in attention_lines(completed):
            fields = attention_fields(line)
            if fields.get("kind") == kind:
                return fields
        if completed.returncode != lib.ATTENTION_EXIT:
            return {}
    return {}


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


OLD_METRIC_PEAK_BYTES = 20 * GIBIBYTE   # old-metric-peaks-not-counted 那一条旧行记的峰值：照改前读它就起步 20480M


def cell_old_metric_peaks_not_counted(world):
    """旧口径的峰值不进起步（task_server_lib.py 文件头「资源记录」的峰值口径一栏；批 92 之前记的 memory_peak 含页缓存）：同一命令键只有一条没写峰值口径的旧行（峰值 20G），
    新交的照没有记录办：上限与记账 4096M、prediction 记 UNRECORDED_HISTORY；同键另一条写了这一版口径的（峰值 1536M）照它定 1536M（不乘 1.5）。
    弄坏开关 old-metric-peaks-read 下旧行照改前进定额度（20480M），这一格红。"""
    old_key, new_key = "selftest-old-metric", "selftest-new-metric"
    world.seed_attempt({"command_key": old_key, "outcome": "succeeded", "memory_peak": OLD_METRIC_PEAK_BYTES}, metric=None)
    world.seed_peak(new_key, 1536 * MEBIBYTE)
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
    return f"只有旧口径那一行的命令照没记录的 {UNRECORDED_DEFAULT_PINNED_TEXT} 起，写了这一版口径的照峰值（1536M）"


# ── 异常表、再推、按会话号找收件 socket、交任务的二次确认、收尾协议单测（task_server_lib.py 文件头「异常表」；task_server.py 文件头 ⑥ ⑦ ②「内存」）──
ADVICE_SAMPLE_TASK = 4242          # anomaly-advice-in-texts 拼正文用的任务号（不进库）
REPUSH_SAMPLE_SECONDS = "2"        # anomaly-repushed-with-history：同一道门槛报过之后再推的门槛（秒；TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS）
UNCLAIMED_REPUSH_SAMPLE_SECONDS = "2"   # unclaimed-outcome-repushed：结局没人取再推的门槛（秒；TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS）
CONFIRM_TIMEOUT_SAMPLE_SECONDS = "2"    # unconfirmed-cancelled：交了没确认满这么多秒服务取消（TASK_SERVER_SELFTEST_CONFIRM_TIMEOUT_SECONDS）
CONFIRM_HOLD_SECONDS = 1.5         # submit-awaits-confirmation：待确认的那一条看这么久（十几轮排队）不该起
CONFIRM_SCOPE_SAMPLE = "种类=items; 跑什么=自证的确认单; 范围=2 项; 为什么=核确认单打不打范围声明"
CLEANUP_PROTOCOL_TESTS = 25   # task-server/tests/test_cleanup_protocol.py 的单测条数：跑到的少了（漏载、跳过）也判红；那份文件加减单测时一起改这个数


def table_advice(kind, task_id):
    """异常表（lib.ANOMALY_TABLE_TEXT）里这一种那一行的修改意见，{task} 换成任务号：照表现读（lib.anomaly_rows），不经 lib.anomaly_advice（弄坏开关 attention-next-generic 弄坏的是那一侧）。"""
    rows = {row.kind: row for row in lib.anomaly_rows(lib.ANOMALY_TABLE_TEXT)}
    return rows[kind].advice.replace(lib.ANOMALY_TASK_PLACEHOLDER, str(task_id))


def cell_anomaly_table_matches_kinds(world):
    """异常表（lib.ANOMALY_TABLE_TEXT）与代码里分别处置的种类逐行对得上：ATTENTION_KINDS 每一种是「提醒照跑」（BOUNCING_ATTENTION_KINDS 那几种是「打回」），SUBMIT_REFUSAL_KINDS 是「提交时拒」，
    SUBMIT_WARNING_KINDS 是「提醒照跑」，错误类型表（OUTCOME_TYPES）里 succeeded 之外每一种是「打回」或「服务自己修」，SERVICE_ANOMALY_KINDS 是「提醒照跑」或「服务自己修」；
    表里不多不少、种类不重、动作在 ANOMALY_ACTIONS 里、修改意见不是那一句通用的；task_server.py 的 ANOMALY_REPUSHED_KINDS 都是表里的提醒。静态核，不起服务。"""
    rows = lib.anomaly_rows(lib.ANOMALY_TABLE_TEXT)
    table = {row.kind: row for row in rows}
    expected = {**{kind: ("提醒照跑",) for kind in lib.ATTENTION_KINDS + lib.SUBMIT_WARNING_KINDS}, **{kind: ("打回",) for kind in lib.BOUNCING_ATTENTION_KINDS}, **{kind: ("提交时拒",) for kind in lib.SUBMIT_REFUSAL_KINDS},
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
    fake.reminder_histories, fake.attention_lock, fake.anomaly_repush_seconds = {}, threading.Lock(), SERVER.ANOMALY_REPUSH_SECONDS
    task = {"id": ADVICE_SAMPLE_TASK}
    problems = []
    for kind in lib.ATTENTION_KINDS:
        advice = table_advice(kind, ADVICE_SAMPLE_TASK)
        item = SERVER.Scheduler.attention_item(fake, kind, 1, ADVICE_SAMPLE_TASK, "依据的数", "原因", "这一回的具体数")
        text = SERVER.Scheduler.attention_text(fake, task, item)
        if f"出路：{advice}" not in text:
            problems.append(f"{kind} 的正文「出路：」应当以「{advice[:80]}…」起头，实际「{text[:300]}」")
        fake.reminder_histories[(ADVICE_SAMPLE_TASK, kind)] = [{"threshold": 1, "repush": 0, "at": time.time() - 60, "basis": "头一回的依据"}]
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
        if wanted not in fields.get("next", "") or lib.BOUNCE_NEW_TASK_REMINDER not in fields.get("next", ""):
            problems.append(f"结局 {kind} 的 next= 应当带异常表那一行的修改意见「{wanted[:80]}…」与「{lib.BOUNCE_NEW_TASK_REMINDER}」，实际「{fields.get('next', '')[:300]}」")
    if problems:
        raise SelftestFailure("推出去的正文带不带异常表的修改意见：" + "；".join(problems))
    return f"{len(lib.ATTENTION_KINDS)} 种提醒（头一回与再推）、疑似卡住与 {len(outcomes)} 种结局的正文都带异常表那一行的修改意见"


HEADLINE_FACTS = re.compile(r"^task-server：(结局|提醒|疑似卡住)（([^）]*)）任务 (\d+) (.+?)（([a-z][a-z-]*)([^）]*)）$")   # 首行：类别、任务此刻的状态、任务号、中文名、英文种类名、括号里其余的


def headline_facts(line):
    """只看推给会话的消息首行，判三件事 → (是结局还是提醒还是疑似卡住, 任务此刻在排在跑还是已结束, 第几回, 上一回的日期或 None, 中文名, 英文种类名)；首行不合写法交 None。"""
    found = HEADLINE_FACTS.match(line)
    if found is None:
        return None
    category, state, _, name, english, rest = found.groups()
    after = "任务已结束" if "任务已结束" in state else "任务还在排" if "任务还在排" in state else "任务还在跑" if "任务还在跑" in state else ""
    rounds, previous = re.search(r"第 (\d+) 回(?!再推)", rest), re.search(r"上一回在 (\d{4}-\d{2}-\d{2})", rest)
    return category, after, int(rounds.group(1)) if rounds else None, previous.group(1) if previous else None, name, english


def cell_inbox_headlines_tell_kind_state_round(world):
    """推给会话的消息首行说清是什么（task_server.py 的 outcome_headline、reminder_headline、suspect_headline、attention_text、suspect_text；起因是别的会话把一条 suspected-stall 读成了结局）：
    四类各造一条，只看首行判得出三件事——是结局、提醒还是疑似卡住；任务此刻在排、在跑还是已结束；第几回（重推的另有上一回在哪一天）：
    结局「结局（任务已结束）…（succeeded，退 0；第 1 回）」；排着的任务的提醒「提醒（任务还在排，不是结局）…」第 1 回；在跑的任务第 2 回再推的提醒第 3 回、上一回在三天前那一天；
    疑似卡住「疑似卡住（任务还在跑、服务没停它）…」；结局没人取而再推的「结局（任务已结束，这条结局还没人取）…」第 3 回、上一回在两天前那一天。每个种类带中文名、英文种类名在括号里；
    错误类型表的每一种结局、每一种提醒、疑似卡住、结局没人取在 lib.KIND_NAMES 里都有中文名。正文里确认标识那几行（长跑已确认）在依据与出路之后；机器读的最后一行不动
    （TASK_SERVER_ATTENTION、TASK_SERVER_SUSPECT 起头）。套在假调度器上，不起服务。
    弄坏开关 inbox-headline-state-omitted 下首行不写类别与状态，inbox-headline-round-omitted 下不写第几回，inbox-headline-name-omitted 下不写中文名，
    inbox-flag-lines-before-evidence 下确认标识那几行在依据与出路之前，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.store = types.SimpleNamespace(task=lambda task_id: None)
    fake.reminder_histories, fake.attention_lock, fake.anomaly_repush_seconds = {}, threading.Lock(), SERVER.ANOMALY_REPUSH_SECONDS
    fake.flag_lines = lambda found: ["长跑已确认：自证里的一句理由"]
    now = time.time()
    day = lambda seconds: time.strftime("%Y-%m-%d", time.localtime(seconds))
    problems = []

    def check(what, line, expected):
        facts = headline_facts(line)
        if facts != expected:
            problems.append(f"{what}：首行应当判得出 {expected}，实际 {facts}（首行「{line}」）")

    check("结局", SERVER.outcome_headline(701, "succeeded", 0), ("结局", "任务已结束", 1, None, "成功", "succeeded"))
    queued_item = SERVER.Scheduler.attention_item(fake, "queued-too-long", 1, 702, "已排 30 分钟", "原因")
    queued_text = SERVER.Scheduler.attention_text(fake, {"id": 702, "state": "queued"}, queued_item)
    check("排着的任务的提醒", queued_text.split("\n", 1)[0], ("提醒", "任务还在排", 1, None, "排队过久", "queued-too-long"))
    fake.reminder_histories[(703, "fills-any-cap")] = [{"threshold": 1, "repush": 0, "at": now - 5 * 86400, "round": 1, "basis": "头一回"},
                                                       {"threshold": 1, "repush": 1, "at": now - 3 * 86400, "round": 2, "basis": "第 1 回再推"}]
    again_item = dict(SERVER.Scheduler.attention_item(fake, "fills-any-cap", 1, 703, "三次峰值贴着上限", "原因"), repush=2)
    again_text = SERVER.Scheduler.attention_text(fake, {"id": 703, "state": "running"}, again_item)
    check("在跑的任务再推的提醒", again_text.split("\n", 1)[0], ("提醒", "任务还在跑", 3, day(now - 3 * 86400), "按给的上限开并行", "fills-any-cap"))
    suspect_text = SERVER.Scheduler.suspect_text(fake, {"id": 704, "state": "running"}, {"id": 704}, {"id": 9, "number": 1, "host": SERVER.LOCAL, "log": "x.log"},
                                                 {"segment": 2, "why": "整个单元的 CPU 这 600 秒平均 0.0001 个核"})
    check("疑似卡住", suspect_text.split("\n", 1)[0], ("疑似卡住", "任务还在跑", 1, None, "疑似卡住", "suspected-stall"))
    check("结局没人取再推", SERVER.outcome_headline(705, "succeeded", 0, 3, now - 2 * 86400, unclaimed=True),
          ("结局", "任务已结束", 3, day(now - 2 * 86400), "成功", "succeeded"))
    if "第 2 段" not in suspect_text.split("\n", 1)[0] or "没处置、第 2 回再推" not in again_text.split("\n", 1)[0] or "还没人取" not in SERVER.outcome_headline(705, "succeeded", 0, 3, now, unclaimed=True):
        problems.append("疑似卡住应写第几段，再推的写没处置、第几回再推，没人取的写这条结局还没人取")
    for what, text in (("提醒", again_text), ("疑似卡住", suspect_text)):
        lines = text.split("\n")
        flag = lines.index("长跑已确认：自证里的一句理由") if "长跑已确认：自证里的一句理由" in lines else None
        advice = next((index for index, line in enumerate(lines) if line.startswith("出路：")), None)
        if what == "提醒" and (flag is None or advice is None or flag < advice or any(line.startswith("依据：") for line in lines[flag:])):
            problems.append(f"提醒：确认标识那一行应当在依据与出路之后，实际各行 {[line[:20] for line in lines]}")
    if not again_text.split("\n")[-1].startswith(f"{lib.ATTENTION_MARK} task=703 ") or not suspect_text.split("\n")[-1].startswith(f"{lib.SUSPECT_MARK} task=704 "):
        problems.append("每条消息最末那一行机器读的行（TASK_SERVER_ATTENTION、TASK_SERVER_SUSPECT）应当一个字不动、仍在最后")
    kinds = [*lib.OUTCOME_TYPES, *lib.ATTENTION_KINDS, "suspected-stall", "unclaimed-outcome"]
    unnamed = sorted(kind for kind in kinds if kind not in lib.KIND_NAMES or lib.KIND_NAMES[kind] == kind or not lib.KIND_NAMES[kind].strip())
    if unnamed:
        problems.append(f"这些种类在 lib.KIND_NAMES 里没有中文名：{'、'.join(unnamed)}")
    if problems:
        raise SelftestFailure("推给会话的消息首行：" + "；".join(problems))
    return "结局、提醒（在排、在跑再推）、疑似卡住、结局没人取再推四类：只看首行就判得出类别、任务在排在跑还是已结束、第几回；中文名在前、英文种类名在括号里；确认标识那几行在依据与出路之后"


def cell_anomaly_repushed_with_history(world):
    """门槛不随时长涨的提醒同一道门槛推过之后还在（task_server.py 文件头 ⑥「没处置时」；异常表 fills-any-cap 那一行）：同键三次峰值都贴着上限、再交一条一直跑着，
    收件 socket 先收到一条「（fills-any-cap；第 1 回）」，同一道门槛报过满再推的门槛（自证里 2 秒）再收到一条「（fills-any-cap，没处置、第 1 回再推；第 2 回」，正文写「之前推过：…」与「此刻还差：<那一行的修改意见>」。
    弄坏开关 anomaly-not-repushed 下不再推，inbox-headline-state-omitted 下首行不写类别与状态（「提醒（任务还在跑，不是结局）」那一段）、照首行认不到那一条，这一格红。"""
    key = "selftest-anomaly-repush"
    for _ in range(SERVER.FILLS_CAP_RUNS):
        world.seed_attempt({"command_key": key, "outcome": "succeeded", "memory_peak": FILLS_CAP_SEEDED_PEAK_BYTES, "memory_cap": FILLS_CAP_SEEDED_CAP_BYTES})
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-anomaly-repush")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS": REPUSH_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: key}))
    states = ("任务还在排", "任务还在跑")   # 这一条交的时候就判得出（同键三次峰值贴着上限），排着时就可能推，起了以后也可能推
    first_head = tuple(f"{reminder_head(task_id, 'fills-any-cap', state)}；" for state in states)
    again_head = tuple(f"{reminder_head(task_id, 'fills-any-cap', state)}，没处置、第 1 回再推；第 2 回" for state in states)
    problems = inbox_problems(inbox.wait_for(world, first_head), first_head, "头一回")
    again = inbox.wait_for(world, again_head)
    problems += inbox_problems(again, again_head, "同一道门槛没处置再推")
    text = again[0]["text"] if again else ""
    advice = table_advice("fills-any-cap", task_id)
    if again and ("之前推过：" not in text or f"此刻还差：{advice}" not in text):
        problems.append(f"再推的正文应当写「之前推过：…」与「此刻还差：{advice[:60]}…」，实际「{text[:400]}」")
    open(release, "w").close()
    world.finished(task_id)
    if problems:
        raise SelftestFailure("没处置的提醒再推：" + "；".join(problems), world.server_log_lines())
    return f"fills-any-cap 头一回推过、满 {REPUSH_SAMPLE_SECONDS} 秒还在，再推了一回（写了之前推过什么、此刻还差什么）"


def cell_anomaly_acknowledged_not_repushed(world):
    """出路是照跑的提醒记了照跑之后不再推（task_server.py 文件头 ⑥「没处置时」，ACKNOWLEDGED_KINDS）：同键三次峰值贴着上限、再交一条一直跑着，收到 fills-any-cap 头一回之后
    提交它的会话 confirm --why（还没跑满两小时的界）退 0、回话写「已记照跑」；之后过了再推的时限（自证里 2 秒）两倍多，没再收到「没处置、第 1 回再推」那一条。
    弄坏开关 anomaly-acknowledgement-ignored 下照推，这一格红。"""
    key = "selftest-anomaly-acknowledged"
    for _ in range(SERVER.FILLS_CAP_RUNS):
        world.seed_attempt({"command_key": key, "outcome": "succeeded", "memory_peak": FILLS_CAP_SEEDED_PEAK_BYTES, "memory_cap": FILLS_CAP_SEEDED_CAP_BYTES})
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-anomaly-acknowledged")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_ANOMALY_REPUSH_SECONDS": REPUSH_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", BLOCK_SCRIPT, "block", release], extra=dict(inbox.environment(), **{lib.KEY_VARIABLE: key}))
    states = ("任务还在排", "任务还在跑")   # 这一条交的时候就判得出（同键三次峰值贴着上限），排着时就可能推，起了以后也可能推
    first_head = tuple(f"{reminder_head(task_id, 'fills-any-cap', state)}；" for state in states)
    again_head = tuple(f"{reminder_head(task_id, 'fills-any-cap', state)}，没处置、第 1 回再推；第 2 回" for state in states)
    problems = inbox_problems(inbox.wait_for(world, first_head), first_head, "头一回")
    confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE])
    if confirmed.returncode != 0 or "已记照跑" not in confirmed.stdout + confirmed.stderr:
        problems.append(f"没满两小时的界、开着 fills-any-cap 时 confirm --why 应当退 0、回「已记照跑」，实际退 {confirmed.returncode}：{(confirmed.stdout + confirmed.stderr)[-300:]}")
    time.sleep(3 * float(REPUSH_SAMPLE_SECONDS))
    if inbox.headed(again_head):
        problems.append(f"记了照跑之后不该再推，实际收到「{again_head}」")
    open(release, "w").close()
    world.finished(task_id)
    if problems:
        raise SelftestFailure("记了照跑的提醒不再推：" + "；".join(problems), world.server_log_lines())
    return f"fills-any-cap 头一回推过、confirm --why 记了照跑，过了 {3 * float(REPUSH_SAMPLE_SECONDS):.0f} 秒没再推"


PROGRESS_PUSH_SAMPLE_SECONDS = "2"   # running-progress-pushed：在跑的任务每满这么多秒发一条进度（TASK_SERVER_SELFTEST_PROGRESS_PUSH_SECONDS）


PROGRESS_LOG_LINE_ROUNDS = 6   # running-progress-pushed：等日志末行出现在进度正文里至多等到第几回


def cell_running_progress_pushed(world):
    """在跑进度（task_server.py 文件头 ⑥「在跑进度」）：一条任务在跑满时限（自证里 2 秒）之后，提交它的会话收到一条首行「进度（任务还在跑，不是结局）任务 N 起跑以来」的消息，
    写第 1 回、它在哪一台与日志末行；再满一回时限收到第 2 回；放它跑完，结局照发。弄坏开关 progress-not-pushed 下不发，这一格红。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_PROGRESS_PUSH_SECONDS": PROGRESS_PUSH_SAMPLE_SECONDS})
    release = os.path.join(world.root, "release-progress")
    task_id = world.submit(["bash", "-c", "echo progress-line-sample; " + BLOCK_SCRIPT, "block", release], extra=inbox.environment())
    world.running(task_id)
    head = f"{SERVER.INBOX_HEADLINE}进度（任务还在跑，不是结局）任务 {task_id} 起跑以来"
    found = inbox.wait_for(world, head, count=2)
    problems = []
    if len(found) < 2:
        problems.append(f"在跑满 {PROGRESS_PUSH_SAMPLE_SECONDS} 秒两回应当收到两条首行「{head}」的进度，实际 {len(found)} 条")
    else:
        first, second = found[0]["text"], found[1]["text"]
        if "第 1 回" not in first.splitlines()[0] or "第 2 回" not in second.splitlines()[0]:
            problems.append(f"两条的首行应当依次写第 1 回、第 2 回，实际「{first.splitlines()[0]}」「{second.splitlines()[0]}」")
        # 日志末行不钉在第 1 回上：时限从尝试起了那一刻算，机器忙时命令的第一行可能晚于第 1 回才写进日志（load 139 时第 1 回读到「日志还是空的」）；
        # 接着等后面的回数，至多到第 PROGRESS_LOG_LINE_ROUNDS 回，有一回带上那一行就算。
        for wanted_count in range(3, PROGRESS_LOG_LINE_ROUNDS + 1):
            if any("progress-line-sample" in message["text"] for message in found):
                break
            found = inbox.wait_for(world, head, count=wanted_count)
        if f"任务 {task_id} 在 " not in first or not any("progress-line-sample" in message["text"] for message in found):
            problems.append(f"进度正文应当写这一次尝试在哪一台、收到的 {len(found)} 回里有一回带日志末行（progress-line-sample），实际第 1 回「{first[:400]}」、最后一回「{found[-1]['text'][:400]}」")
    with open(release, "w", encoding="utf-8"):
        pass
    world.finished(task_id)
    if problems:
        raise SelftestFailure("在跑进度：" + "；".join(problems), world.server_log_lines())
    return f"在跑的任务每满 {PROGRESS_PUSH_SAMPLE_SECONDS} 秒给提交它的会话发一条进度，写第几回、在哪一台与日志末行"


def cell_unclaimed_outcome_repushed(world):
    """结局没人取（task_server.py 文件头 ⑥「结局没人取」；异常表 unclaimed-outcome）：会话还没登进会话登记时任务跑完，结局那一条发不出去（写 inbox-unreachable 告警）；
    之后会话登上（演会话重启、换了 socket），结局满再推的门槛（自证里 2 秒）没人取，服务再推一回到新的 socket：第一行「结局还没人取（succeeded」，写此刻还差什么（那一行的修改意见）
    与那条结局的末行；推出去了记成已取，status 不再列成没人取。弄坏开关 unclaimed-outcome-not-repushed 下不再推，这一格红。"""
    inbox = FakeInbox(world, "a", register=False)
    start_inbox_server(world, {"TASK_SERVER_SELFTEST_UNCLAIMED_WARNING_SECONDS": UNCLAIMED_REPUSH_SAMPLE_SECONDS})
    task_id = world.submit(["bash", "-c", "exit 0", "unclaimed-repush"], extra=inbox.environment())
    world.finished(task_id)
    patience = world.patience(INBOX_WAIT_SECONDS)
    while patience.waiting() and not any(alert.get("kind") == "inbox-unreachable" and alert.get("task") == task_id for alert in world.alerts()):
        time.sleep(POLL_SECONDS)
    world.register_session(SESSION_A, inbox.path, "inbox-a")
    head = unclaimed_head(task_id, "succeeded")
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
    head = outcome_head(task_id, "succeeded")
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
    head = outcome_head(task_id, "cancelled")
    messages = inbox.wait_for(world, head)
    problems += inbox_problems(messages, head, "没确认被取消")
    if messages and "没确认，已取消" not in messages[0]["text"]:
        problems.append(f"推给会话的正文应当带「没确认，已取消」，实际「{messages[0]['text'][:300]}」")
    if problems:
        raise SelftestFailure("交了没确认：" + "；".join(problems), world.server_log_lines())
    return f"交了满 {CONFIRM_TIMEOUT_SAMPLE_SECONDS} 秒没确认：服务取消、推了「没确认，已取消」"


def cell_cleanup_protocol_unit_tests(world):
    """收尾协议的离线单测（task-server/tests/test_cleanup_protocol.py；unittest，在这一格自己的进程里跑，不起服务）：恰好 CLEANUP_PROTOCOL_TESTS 条、全过才绿；
    它的假调度器照 Scheduler.__init__ 的表名现建（scheduler_tables）。弄坏开关 cleanup-tests-tables-hand-listed 下照手抄的旧名单建、收尾那几条红，这一格红。"""
    module = lib.load_module("test_cleanup_protocol", os.path.join(lib.SCRIPT_DIRECTORY, "tests", "test_cleanup_protocol.py"))
    suite = unittest.defaultTestLoader.loadTestsFromModule(module)
    stream = io.StringIO()
    result = unittest.TextTestRunner(stream=stream, verbosity=2).run(suite)
    bad = [test.id() for test, _ in result.failures + result.errors] + [test.id() for test, _ in result.skipped]
    if result.testsRun != CLEANUP_PROTOCOL_TESTS or bad or suite.countTestCases() != CLEANUP_PROTOCOL_TESTS:
        raise SelftestFailure(f"收尾协议单测应当跑 {CLEANUP_PROTOCOL_TESTS} 条、全过，实际载入 {suite.countTestCases()} 条、跑了 {result.testsRun} 条，不对或跳过的 {bad}",
                              stream.getvalue().splitlines()[-40:])
    return f"收尾协议单测 {result.testsRun} 条全过（假调度器的表照 Scheduler.__init__ 现建）"


FILLS_CAP_CLIMB_CAPS = (6144 * MEBIBYTE, 9216 * MEBIBYTE, 13824 * MEBIBYTE)   # fills-any-cap-cap-hits-not-reported：照上限一档档撞上去的三次（6.0、9.0、13.5 GiB）
FILLS_CAP_CLIMB_PEAK_PERCENT = 99             # 那三次撞上限时的峰值占各自上限的百分数（撞的那一次峰值本来就贴着上限）


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


FILLS_CAP_RECORD_CAPS = (1400 * MEBIBYTE, 1500 * MEBIBYTE, 1000 * MEBIBYTE)   # fills-any-cap-record-caps-not-reported：照记录定的三次上限（任务 13774 那条命令）
FILLS_CAP_RECORD_PEAKS = (1400 * MEBIBYTE, 1400 * MEBIBYTE, 1000 * MEBIBYTE)   # 那三次的峰值：贴着各自的上限
FILLS_CAP_GROWN_CAPS = (4096 * MEBIBYTE, 6144 * MEBIBYTE, 9216 * MEBIBYTE)     # 没记录时起在 4096M、一档档放大的三次上限


def cell_fills_any_cap_record_caps_not_reported(world):
    """上限照记录定的不算按上限开并行（task_server.py 的 fills_any_cap_items；任务 13774 的依据是「上限 1.4 GiB 峰值 1.4 GiB、上限 1.5 GiB 峰值 1.4 GiB、上限 1.0 GiB 峰值 1.0 GiB」，
    而那条命令不读上限）：同一命令键记下三次跑成的尝试，各自的上限照记下的峰值定（尝试的 prediction 一栏是 record）、峰值贴着上限：确认单里没有 fills-any-cap；
    另一个命令键三次跑成、上限是没记录时从 4096M 起一档档放大的（prediction 是 unrecorded-default、retry）、峰值都跟着涨到贴着上限：确认单里照列 fills-any-cap。
    弄坏开关 fills-any-cap-counts-record-caps 下照记录定的上限也算，头一个命令键也被报，这一格红。"""
    keys = {"record": "selftest-fills-any-cap-record", "grown": "selftest-fills-any-cap-grown"}
    for cap, peak in zip(FILLS_CAP_RECORD_CAPS, FILLS_CAP_RECORD_PEAKS):
        world.seed_attempt({"command_key": keys["record"], "outcome": "succeeded", "memory_peak": peak, "memory_cap": cap, "prediction": lib.RECORD_HISTORY})
    for cap, history in zip(FILLS_CAP_GROWN_CAPS, (lib.UNRECORDED_HISTORY, "retry", "retry")):
        world.seed_attempt({"command_key": keys["grown"], "outcome": "succeeded", "memory_peak": cap * 97 // 100, "memory_cap": cap, "prediction": history})
    world.resources(local=(100, 100))
    world.start_server()
    problems = []
    for what, key, reported in (("上限照记录定的", keys["record"], False), ("上限一档档放大的", keys["grown"], True)):
        submitted = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", f"fills-any-cap-{key[-6:]}"], extra={lib.KEY_VARIABLE: key})
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", submitted.stdout)
        if submitted.returncode != 0 or not found or lib.confirm_needed_fields(submitted.stdout) is None:
            raise SelftestFailure(f"交了应当退 0、打任务号与确认单，实际退 {submitted.returncode}", [submitted.stdout, submitted.stderr] + world.server_log_lines())
        if ("fills-any-cap：" in submitted.stdout) != reported:
            problems.append(f"{what}三次峰值都贴着上限：确认单里应当{'列' if reported else '不列'} fills-any-cap，实际「{submitted.stdout.strip()[-400:]}」")
        world.confirm_submitted(submitted)
        world.client(["wait", found.group(1)], timeout=ATTENTION_WAIT_SECONDS)
        done = world.finished(int(found.group(1)))
        if done["task"].get("outcome") != "succeeded":
            problems.append(f"{what}：应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("上限照记录定的不报按上限开并行：" + "；".join(problems), world.server_log_lines())
    return "三次上限照记录定、峰值贴着：不报 fills-any-cap；三次上限是没记录时一档档放大的、峰值跟着涨：照报"


STAGED_PEAK_BYTES = 777 * MEBIBYTE   # attempt-row-not-overwritten：演对账在删编译目录的这一会儿记下的那一次的峰值


class StagedRemovalMachine:
    """attempt-row-not-overwritten 的假本机：删编译目录的那一下（remove_build_directory）先演对账在这一会儿记下那一次的结局（staged 里那个目录对应的 ended 整行，
    照 finish_attempt 那样经 Store.update 写进库），再真删、回读不在了交删成；build_directory_exists 照目录在不在。"""

    def __init__(self, store, staged):
        self.store = store
        self.staged = staged   # {目录: 对账记下的那一份尝试正文}
        self.name = SERVER.LOCAL
        self.label = "本机"

    def build_directory_exists(self, directory):
        return os.path.isdir(directory)

    def remove_build_directory(self, directory):
        ended = self.staged.pop(directory, None)
        if ended is not None:
            self.store.update("attempts", ended)
        shutil.rmtree(directory, ignore_errors=True)
        return not os.path.lexists(directory), []

    def removal_howto(self, directory):
        return f"rm -r {shlex.quote(directory)}"


def staged_removal_scheduler(world, staged):
    """只备删编译目录那几段读写的几样（这一格临时库上的 Store、假本机、删目录的锁、环境、记告警）的假调度器：Scheduler 的方法照原样套在它上面跑
    （task_server.py 的 cancelled_attempt_artifacts、remove_finished_build_directories_locked、mark_build_directory_removed）。"""
    scheduler = object.__new__(SERVER.Scheduler)
    scheduler.store = SERVER.Store(os.path.join(world.root, "staged-removal.sqlite3"), bytes(32), dict(os.environ))
    scheduler.environment = dict(os.environ)
    scheduler.machines = {SERVER.LOCAL: StagedRemovalMachine(scheduler.store, staged)}
    scheduler.build_removal_lock = threading.RLock()
    scheduler.alerts = []
    scheduler.alert = lambda kind, task_id, line, howto: scheduler.alerts.append((kind, task_id, line))
    return scheduler


def cell_attempt_row_not_overwritten(world):
    """HTTP 线程手里是先读的那份 running 的尝试、正删它的编译目录，这一会儿对账把那一次记成 ended（结局 cancelled、峰值记下），删完记「已删」
    （取消分片父任务时删子任务尝试的产物 cancelled_attempt_artifacts、记下结局时删 remove_finished_build_directories_locked 两处，同一个 mark_build_directory_removed）：
    库里那一次照旧 ended、结局与峰值都在、build_directory_removed 记上，按 state IN ('starting', 'running') 数的记账不再算它；
    拿那份 running 的旧字典直接 Store.update 抛 StoreRefused("ended")。次序由假本机在删的那一下写进 ended 那一行精确造出；只用 task_server.py 的 Store 与
    Scheduler 的这几个方法，套在假调度器上，不起服务。弄坏开关 attempt-row-overwritten 下整行写回 running、Store.update 不守，这一格红。"""
    staged = {}
    scheduler = staged_removal_scheduler(world, staged)
    store = scheduler.store
    problems = []
    for task_id, name in ((1, "取消之后删产物（cancelled_attempt_artifacts）"), (2, "记下结局时删（remove_finished_build_directories_locked）")):
        directory = os.path.join(world.root, f"staged-build-{task_id}")
        os.makedirs(os.path.join(directory, "target"))
        store.insert("attempts", {"task_id": task_id, "number": 1, "host": SERVER.LOCAL, "state": "running", "session": SESSION_A, "command_key": "selftest-staged-removal",
                                  "memory_cap": GIBIBYTE, "build_directory": directory})
        stale = store.attempts_of(task_id)[0][0]
        staged[directory] = dict(stale, state="ended", outcome="cancelled", ended_at=round(time.time(), 3), seconds=1.0, memory_peak=STAGED_PEAK_BYTES)
        if task_id == 1:
            SERVER.Scheduler.cancelled_attempt_artifacts(scheduler, dict(stale), {"sizes": {}, "active": {stale["id"]}}, set())
        else:
            SERVER.Scheduler.remove_finished_build_directories_locked(scheduler, {"id": task_id, "environment": {}})
        found = store.attempts_of(task_id)[0][0]
        seen = {key: found.get(key) for key in ("state", "outcome", "memory_peak", "build_directory_removed")}
        wanted = {"state": "ended", "outcome": "cancelled", "memory_peak": STAGED_PEAK_BYTES, "build_directory_removed": True}
        if seen != wanted or os.path.lexists(directory) or staged:
            problems.append(f"{name}：删的这一会儿对账记了 ended，删完库里那一次应当是 {wanted}、目录不在，实际 {seen}、目录{'还在' if os.path.lexists(directory) else '不在'}、"
                            f"{'假本机没被叫去删' if staged else '删过'}")
        try:
            store.update("attempts", dict(stale))
            problems.append(f"{name}：拿那份 running 的旧字典 Store.update 应当被拒（库里已 ended），实际写进去了：库里此刻 {store.attempts_of(task_id)[0][0].get('state')}")
        except SERVER.StoreRefused as refused:
            if refused.kind != "ended":
                problems.append(f"{name}：Store.update 拒的种类应当是 ended，实际 {refused.kind}（{refused}）")
    active = [record.get("id") for record, _ in store.select("attempts", "state IN ('starting', 'running')")]
    if active:
        problems.append(f"两次都收尾之后按在跑数的记账一条都不算，实际算在跑的尝试 {active}")
    if problems:
        raise SelftestFailure("删编译目录时尝试表不整行写回：" + "；".join(problems), [str(alert) for alert in scheduler.alerts])
    return "删编译目录的这一会儿对账记了 ended：取消删产物与记下结局删两处删完都只并进「已删」，结局与峰值都在、不算在跑；旧字典整行写回被拒"


CHILD_SETTLE_HOLD_SECONDS = 3.0   # parent-cancel-waits-for-children：对账停在收尾子任务那一次的开头，看这么久里取消回没回话


def cell_parent_cancel_waits_for_children(world):
    """取消在跑的分片整件（一片在跑）：停了那一片的单元之后，对账收尾那一次之前精确停住（TASK_SERVER_SELFTEST_PAUSE_POINT=attempt-finishing），
    CHILD_SETTLE_HOLD_SECONDS 秒里 task cancel 不该回话（task_server.py 的 cancel_children 等子任务在库里记成 finished）；放开之后回 stopped、退 0，
    回话那一刻各子任务都已 finished、整件 cancelled，整件的结局时刻不早于任何一个子任务。弄坏开关 parent-finalized-before-children 下停了单元就回，这一格红。"""
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    release = os.path.join(world.root, "release-cancel-waits")
    command = world.write_spec("spec-cancel-waits.json", {"shards": [{"block": release}]})
    world.resources(local=(30, 30))
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"attempt-finishing:{pause}"})
    task_id = world.submit(command, cwd=world.repo)
    problems = []
    canceller = None
    try:
        world.wait_for(task_id, lambda summary: len(shard_children(summary)) == 1 and len(world.sharded_calls("shard")) == 1 and all(
            any(attempt.get("state") == "running" for attempt in child["attempts"]) for child in shard_children(summary).values()), "那一片跑起来")
        open(os.path.join(pause, "arm-attempt-finishing"), "w").close()
        canceller = world.client(["cancel", str(task_id)], background=True)
        wait_path(world, os.path.join(pause, "reached-attempt-finishing"), "停了那一片的单元之后、对账收尾那一次之前停住")
        early = exited_within({"task cancel": canceller}, CHILD_SETTLE_HOLD_SECONDS)
        if early:
            middle = world.summary(task_id)
            problems.append(f"对账还没收尾那一片时 task cancel 不该回话（要等子任务在库里记成 finished），实际 {CHILD_SETTLE_HOLD_SECONDS:.0f} 秒里就回了（退 {canceller.returncode}）、"
                            f"整件 {middle['task'].get('state')}/{middle['task'].get('outcome')}、各子任务 {[(child['task'].get('role'), child['task'].get('state')) for child in middle.get('children', [])]}")
    finally:
        open(os.path.join(pause, "release-attempt-finishing"), "w").close()
        open(release, "w").close()   # 弄坏开关下单元没停的话放它跑完，收尾不等满时限
    stdout, stderr = world.communicate(canceller, STATE_TIMEOUT_SECONDS)
    replied = world.summary(task_id)
    children = replied.get("children", [])
    unsettled = [(child["task"].get("role"), child["task"]["id"], child["task"].get("state")) for child in children if child["task"].get("state") != "finished"]
    if not problems and (canceller.returncode != 0 or replied["task"].get("outcome") != "cancelled" or unsettled):
        problems.append(f"放开之后 task cancel 应当退 0、回话那一刻整件 cancelled、各子任务都已 finished，实际退 {canceller.returncode}、整件 {describe(replied)}、"
                        f"还没收尾的子任务 {unsettled}（{(stdout + stderr).strip()[-300:]}）")
    finished = world.finished(task_id)
    parent_at = float(finished["task"].get("finished_at") or 0)
    later = [(child["task"]["id"], child["task"].get("finished_at")) for child in world.summary(task_id).get("children", [])
             if float(child["task"].get("finished_at") or 0) > parent_at]
    if later:
        problems.append(f"整件的结局时刻 {parent_at} 不该早于子任务，实际比它晚收尾的子任务 {later}")
    if problems:
        raise SelftestFailure("取消整件等子任务收尾：" + "；".join(problems), world.server_log_lines())
    return f"取消在跑的分片整件：对账收尾那一片之前 task cancel 等着，放开之后回 stopped，回话时各子任务都已 finished、整件结局不早于子任务"


def exited_within(processes, seconds):
    """seconds 秒里 processes（{名: Popen}）有哪几个退了：→ [名…]（照给的次序），一有退的就交；到点都还在跑交空的。"""
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        done = [name for name, process in processes.items() if process.poll() is not None]
        if done:
            return done
        time.sleep(POLL_SECONDS)
    return [name for name, process in processes.items() if process.poll() is not None]


CLEAN_RACE_HOLD_SECONDS = 3.0   # clean-waits-for-finished-removal：记下结局那一侧停在删编译目录之前，看这么久里 wait 与 clean 回没回话
WAITER_SETTLE_SECONDS = 2.0     # clean-waits-for-finished-removal：起了 wait 之后等它连上服务、等在那一条上，再放命令跑完


def cell_clean_waits_for_finished_removal(world):
    """记下结局那一侧拿着 build_removal_lock、删编译目录之前精确停住（TASK_SERVER_SELFTEST_PAUSE_POINT=finished-build-removal；主循环 30 秒一轮，
    wait 的兜底那一读这一会儿轮不到）：先等着的 wait 不该拿到结局（task_server.py 的 finalize 先删完再叫醒），这时起的 task clean 等在锁上、不该回话
    （clean 拿同一把锁、锁里现读）；放开之后 wait 退 0、clean 退 0 而删了 0 份（记下结局那一侧删过，两边不重复删），没有 build-directory-left 告警（不报假失败），
    库里那一次 ended、succeeded、build_directory_removed 记上（记录不互相覆盖）、目录不在。弄坏开关 clean-races-finished-removal 下先叫醒 wait、clean 不拿锁，这一格红。"""
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    release = os.path.join(world.root, "release-clean-race")
    world.start_server(extra={"TASK_SERVER_TICK_SECONDS": "30", "TASK_SERVER_SELFTEST_PAUSE_POINT": f"finished-build-removal:{pause}"})
    path = f"{world.fake_cargo}:{os.environ.get('PATH', '/usr/bin:/bin')}"
    task_id = world.submit(["bash", "-c", "cargo build && " + HOLD_UNTIL_RELEASED_SCRIPT, "clean-race", release], extra={"PATH": path})
    problems = []
    waiter = cleaner = None
    try:
        attempt = world.running(task_id)["attempts"][-1]
        build = attempt.get("build_directory") or ""
        patience = world.patience(CANCEL_ARTIFACT_WAIT_SECONDS)
        while not (build and os.path.isfile(os.path.join(build, "target", "debug", "selftest-artifact"))):
            if not patience.waiting():
                raise SelftestFailure(f"任务 {task_id} 等了 {patience.elapsed():.0f} 秒没往编译目录「{build}」写出产物", world.server_log_lines())
            time.sleep(POLL_SECONDS)
        open(os.path.join(pause, "arm-finished-build-removal"), "w").close()
        waiter = world.client(["wait", str(task_id)], background=True, extra={"TASK_SERVER_SELFTEST_FAST": "0"})   # 照平常 20 秒一问：等在那一条上，不隔一会儿就重查库
        time.sleep(WAITER_SETTLE_SECONDS)
        if waiter.poll() is not None:
            raise SelftestFailure(f"命令还在跑时起的 wait 不该退，实际退 {waiter.returncode}", world.server_log_lines())
        open(release, "w").close()
        wait_path(world, os.path.join(pause, "reached-finished-build-removal"), "记下结局之后、删编译目录之前停住")
        cleaner = world.client(["clean", str(task_id)], background=True)
        early = exited_within({"task wait": waiter, "task clean": cleaner}, CLEAN_RACE_HOLD_SECONDS)
        if early:
            problems.append(f"记下结局那一侧还没删完编译目录时 {'、'.join(early)} 不该回话（wait 要删完才叫醒，clean 要等同一把锁），"
                            f"实际 {CLEAN_RACE_HOLD_SECONDS:.0f} 秒里就回了、编译目录{'还在' if os.path.lexists(build) else '已不在'}")
    finally:
        open(os.path.join(pause, "release-finished-build-removal"), "w").close()
        open(release, "w").close()
    wait_out = world.communicate(waiter, STATE_TIMEOUT_SECONDS) if waiter is not None else ("", "")
    clean_out = world.communicate(cleaner, STATE_TIMEOUT_SECONDS) if cleaner is not None else ("", "")
    if waiter.returncode != 0:
        problems.append(f"wait 应当退 0，实际退 {waiter.returncode}（{(wait_out[0] + wait_out[1]).strip()[-200:]}）")
    if cleaner.returncode != 0 or "删了 0 份" not in clean_out[0]:
        problems.append(f"clean 应当退 0、删了 0 份（记下结局那一侧删过，两边不重复删），实际退 {cleaner.returncode}、输出「{(clean_out[0] + clean_out[1]).strip()[-300:]}」")
    left = [alert for alert in world.alerts() if alert.get("kind") == "build-directory-left"]
    if left:
        problems.append(f"不该有 build-directory-left 告警（两边没删出假失败），实际 {left}")
    found = (world.summary(task_id)["attempts"] or [{}])[-1]
    if found.get("state") != "ended" or found.get("outcome") != "succeeded" or not found.get("build_directory_removed") or os.path.lexists(build):
        problems.append(f"那一次应当 ended、succeeded、记了已删、目录不在，实际 {found.get('state')}、{found.get('outcome')}、已删 {found.get('build_directory_removed')}、"
                        f"目录{'还在' if os.path.lexists(build) else '不在'}")
    if problems:
        raise SelftestFailure("task clean 与记下结局时删编译目录：" + "；".join(problems), world.server_log_lines())
    return "记下结局那一侧删编译目录时：等着的 wait 删完才拿到结局，clean 等在锁上、放开后删了 0 份，不报假失败，记录 ended、已删"


def cell_recheck_judges_disk(world):
    """派第二台的那一条起之前复核现读第二台（task_server.py 的 Resources.recheck）：排队那一轮第二台的报告带着够的盘余量、照派；复核读到的报告不带盘余量
    （同真的第二台探测），服务现量（假 df）：
    不到下限的这一条不起、排队原因写「第二台盘只剩」；过了下限而放不下这条命令量过的编译目录占盘（30G 加下限）的不起、排队原因写「编译目录占盘」；
    复核记 fits=0、一次都没起，假 df 放够之后照派第二台跑完。弄坏开关 recheck-disk-unchecked 下复核不判盘、照起，这一格红。"""
    floor = SERVER.DISK_FLOOR_BYTES
    world.seed_attempt({"command_key": "selftest-recheck-build", "outcome": "succeeded", "memory_peak": GIBIBYTE, "build_peak": 30 * GIBIBYTE})
    world.start_server()
    problems = []
    cases = (("盘余量不到下限", "selftest-recheck-floor", floor - GIBIBYTE, "第二台盘只剩"),
             ("编译目录占盘放不下", "selftest-recheck-build", floor + 10 * GIBIBYTE, "编译目录占盘"))
    for name, key, shown, wanted in cases:
        # 本机放不下（空 1G）：只派得到第二台；排队那一轮第二台报告里的盘余量够，复核那一份不带、df 演 shown
        world.resources(local=(1, 30), local_disk_bytes=10 * floor, peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=10 * floor,
                        peer_df=f"     Avail\n{shown}\n", recheck_peer=(100, 100))
        task_id = world.submit(["bash", "-c", "exit 0", name], extra={lib.KEY_VARIABLE: key}, cwd=world.repo)
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        rechecks = []
        while patience.waiting() and not rechecks:
            rechecks = [line for line in world.scheduler_reads() if line.startswith(f"recheck task={task_id} host=peer fits=")]
            time.sleep(POLL_SECONDS)
        world.next_pass()
        held = world.summary(task_id)
        waiting = held.get("waiting_for") or ""
        if rechecks[:1] != [f"recheck task={task_id} host=peer fits=0"] or held["attempts"] or world.calls_of(task_id, "peer") or wanted not in waiting:
            problems.append(f"{name}（复核时 df 报 {SERVER.gibibytes_text(shown)}）：复核应当记 fits=0、一次都不起、排队原因写「{wanted}」，实际复核 {rechecks[:1]}、"
                            f"尝试 {len(held['attempts'])} 次、在等「{waiting[:300]}」")
        world.resources(local=(1, 30), local_disk_bytes=10 * floor, peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=10 * floor,
                        peer_df=f"     Avail\n{10 * floor}\n", recheck_peer=(100, 100))
        done = world.finished(task_id)
        if done["task"].get("outcome") != "succeeded" or (done["attempts"] or [{}])[-1].get("host") != "peer":
            problems.append(f"{name}：df 放够之后应当照派第二台跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("起之前复核判盘余量：" + "；".join(problems), world.server_log_lines())
    return "起之前复核现量第二台的盘：不到下限、放不下编译目录占盘的都不起，排队原因写盘；df 放够之后照派第二台"


def cell_recheck_judges_nfs(world):
    """派第二台的那一条判过放得下、起之前复核现读之前精确停住（TASK_SERVER_SELFTEST_PAUSE_POINT=launch-recheck），这时第二台没挂上共享根（假 ssh 的 stat -f 退 1，
    NFS 复核记下，/health 的 nfs 一栏挂载那一项不过）；放开之后复核那一份过 peer_view（task_server.py 的 launch_recheck）：记 fits=0、不在第二台起，
    排队原因写「没挂上共享根」；挂载好了之后照派第二台跑完。弄坏开关 recheck-nfs-unchecked 下复核那一份不过 peer_view、照派第二台，这一格红。"""
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    mount_exit = os.path.join(world.world, "fake-ssh-mount-exit")
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)   # 本机放不下 4G：只派得到第二台
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"launch-recheck:{pause}"})
    open(os.path.join(pause, "arm-launch-recheck"), "w").close()
    task_id = world.submit(["bash", "-c", "exit 0", "recheck-nfs"], cwd=world.repo)
    problems = []
    try:
        wait_path(world, os.path.join(pause, "reached-launch-recheck"), "判过放得下、起之前复核现读之前停住")
        with open(mount_exit, "w", encoding="utf-8") as handle:
            handle.write("1\n")
        health = health_until(world, lambda health: any(entry.get("mount") != lib.NFS_PASSED for entry in (health.get("nfs") or {}).get("roots", [])),
                              STATE_TIMEOUT_SECONDS)
        if not any(entry.get("mount") != lib.NFS_PASSED for entry in (health.get("nfs") or {}).get("roots", [])):
            raise SelftestFailure(f"假 ssh 的挂载那一项退 1 之后 /health 的 nfs 一栏应当判挂载不过，实际 {health.get('nfs')}", world.server_log_lines())
    finally:
        open(os.path.join(pause, "release-launch-recheck"), "w").close()
    patience = world.patience(STATE_TIMEOUT_SECONDS)
    rechecks = []
    while patience.waiting() and not rechecks:
        rechecks = [line for line in world.scheduler_reads() if line.startswith(f"recheck task={task_id} host=peer fits=")]
        time.sleep(POLL_SECONDS)
    world.next_pass()
    held = world.summary(task_id)
    waiting = held.get("waiting_for") or ""
    if rechecks[:1] != [f"recheck task={task_id} host=peer fits=0"] or held["attempts"] or world.calls_of(task_id, "peer") or "没挂上共享根" not in waiting:
        problems.append(f"复核这一刻第二台没挂上共享根：复核应当记 fits=0、不在第二台起、排队原因写「没挂上共享根」，实际复核 {rechecks[:1]}、尝试 {len(held['attempts'])} 次、"
                        f"在等「{waiting[:300]}」")
    os.remove(mount_exit)
    done = world.finished(task_id)
    if done["task"].get("outcome") != "succeeded" or (done["attempts"] or [{}])[-1].get("host") != "peer":
        problems.append(f"挂载好了之后应当照派第二台跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("起之前复核过 NFS 判法：" + "；".join(problems), world.server_log_lines())
    return "起之前复核那一刻第二台没挂上共享根：复核记 fits=0、不在第二台起、排队原因写明；挂载好了照派第二台"


def cell_stall_stop_spares_next_attempt(world):
    """顶在上限上卡住（同 stalled-at-cap-retried 的假 scope）判停、起了停单元的后台线程，它读库之前精确停住（TASK_SERVER_SELFTEST_PAUSE_POINT=stop-attempts-read）；
    这时第 1 次自己退 250（撞上限）、照重试规矩重排起了第 2 次（每 0.25 秒打一行，不卡）；放开之后后台线程只停判它的那一次（task_server.py 的 judged_attempt_only）：
    排队读数里那一行 stopped=-，第 2 次的单元照跑，放行之后跑完、各次 memory-cap-hit、succeeded。弄坏开关 stall-stop-by-task 下按任务号停、把第 2 次也停了，这一格红。"""
    scope = os.path.join(world.root, "at-cap-scope")
    os.makedirs(scope)
    for name, text in (("cpu.stat", "usage_usec 1000\n"), ("memory.peak", f"{AT_CAP_PEAK_BYTES}\n"), ("memory.stat", process_memory_stat(AT_CAP_PEAK_BYTES))):
        with open(os.path.join(scope, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    world.seed_peak("selftest-stall-next", AT_CAP_SEED_PEAK_BYTES)
    pause = os.path.join(world.root, "pause")
    os.makedirs(pause)
    capped, retried = os.path.join(world.root, "release-capped-attempt"), os.path.join(world.root, "release-retried-attempt")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope,
                              "TASK_SERVER_SELFTEST_PAUSE_POINT": f"stop-attempts-read:{pause}"})
    open(os.path.join(pause, "arm-stop-attempts-read"), "w").close()
    # 第 1 次不出声地等（顶在上限上卡住），放行就退 250；重排之后那一次每 0.25 秒打一行、放行就退 0
    script = ('if [[ "$TASK_SERVER_ATTEMPT" == 1 ]]; then echo started; for round in $(seq 1 2000); do [[ -e "$1" ]] && exit 250; sleep 0.1; done; exit 9; fi; '
              'for round in $(seq 1 2000); do [[ -e "$2" ]] && exit 0; echo tick; sleep 0.25; done; exit 9')
    task_id = world.submit(["bash", "-c", script, "stall-next", capped, retried], extra={lib.KEY_VARIABLE: "selftest-stall-next"})
    problems = []
    try:
        wait_path(world, os.path.join(pause, "reached-stop-attempts-read"), "判停的后台线程读库之前停住")
        open(capped, "w").close()
        summary = world.wait_for(task_id, lambda summary: len(summary["attempts"]) >= 2 and summary["attempts"][-1].get("state") == "running", "重排起第 2 次")
        unit = summary["attempts"][-1].get("unit", "")
        open(os.path.join(pause, "release-stop-attempts-read"), "w").close()
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        stops = []
        while patience.waiting() and not stops:
            stops = [line for line in world.scheduler_reads() if line.startswith(f"stop task={task_id} ")]
            time.sleep(POLL_SECONDS)
        current = world.summary(task_id)["attempts"][-1]
        if not stops or not stops[0].endswith(" stopped=-") or not world.unit_active(unit) or current.get("state") != "running":
            problems.append(f"判停的后台线程读库时第 1 次已自己退了、第 2 次在跑：它应当一个都不停（stopped=-）、第 2 次的单元照跑，"
                            f"实际停单元那一行 {stops[:1]}、第 2 次的单元{'在跑' if world.unit_active(unit) else '不在跑'}、状态 {current.get('state')}")
    finally:
        open(os.path.join(pause, "release-stop-attempts-read"), "w").close()
        open(capped, "w").close()
        open(retried, "w").close()
    done = world.finished(task_id, timeout=60)
    seen = [attempt.get("outcome") for attempt in done["attempts"]]
    if seen != ["memory-cap-hit", "succeeded"] or done["task"].get("outcome") != "succeeded":
        problems.append(f"放行之后应当各次 memory-cap-hit、succeeded，任务 succeeded，实际各次 {seen}、{describe(done)}")
    if problems:
        raise SelftestFailure("判停的后台线程只停判它的那一次：" + "；".join(problems), world.server_log_lines())
    return "判停的后台线程读库时第 1 次已退、第 2 次在跑：它一个都不停，第 2 次照跑完"


FAKE_RECOVER_CLEANUP = r'''#!/usr/bin/env bash
# peer-cleanup-recovered-once 的假 multi-host-run.sh：只认 --recover-cleanup <回执>：往回执同目录的 recover-calls 记一行，睡 1 秒（演连第二台停、删那一趟），
# 往标准输出打一行 selftest-recovered，再把回执原子写成 confirmed
[[ "${1:-}" == --recover-cleanup ]] || { echo "假 multi-host-run.sh：只认 --recover-cleanup" >&2; exit 2; }
receipt="$2"
printf 'recover %s\n' "$receipt" >> "$(dirname -- "$receipt")/recover-calls"
sleep 1
echo "selftest-recovered $receipt"
printf '{"status": "confirmed"}\n' > "$receipt.partial" && mv -- "$receipt.partial" "$receipt"
'''


def cell_peer_cleanup_recovered_once(world):
    """派第二台的一次尝试清理回执是 pending，两个线程（演对账与停单元的后台线程）同一刻确认它（task_server.py 的 confirm_peer_cleanup）：
    --recover-cleanup 只起一趟、那一次的日志只追加一份，后到的等前一趟放下、读到 confirmed，两边都交确认了；锁用完从锁表（PEER_CLEANUP_LOCKS）删。
    恢复脚本是假的（FAKE_RECOVER_CLEANUP：睡 1 秒再写 confirmed，两边同时起的话都读得到 pending）；只用 task_server.py 的 confirm_peer_cleanup，
    套在只备环境与恢复脚本的假调度器上，不起服务。弄坏开关 peer-cleanup-recovered-concurrently 下两趟都起，这一格红。"""
    directory = os.path.join(world.root, "peer-cleanup")
    os.makedirs(directory)
    receipt, log = os.path.join(directory, "peer-cleanup.json"), os.path.join(directory, "attempt.log")
    with open(receipt, "w", encoding="utf-8") as handle:
        json.dump({"status": "pending"}, handle)
    runner = os.path.join(world.bin, "fake-recover-cleanup")
    write_executable(runner, FAKE_RECOVER_CLEANUP)
    scheduler = object.__new__(SERVER.Scheduler)
    scheduler.environment = dict(os.environ)
    scheduler.peer_runner = runner
    attempt = {"id": 7, "task_id": 7, "number": 1, "host": SERVER.PEER, "peer_cleanup_file": receipt, "log": log}
    barrier = threading.Barrier(2)
    results = {}

    def confirm(name):
        barrier.wait(timeout=10)
        results[name] = SERVER.Scheduler.confirm_peer_cleanup(scheduler, dict(attempt))
    threads = [threading.Thread(target=confirm, args=(name,), daemon=True) for name in ("对账", "停单元的后台线程")]
    for thread in threads:
        thread.start()
    for thread in threads:
        thread.join(timeout=STATE_TIMEOUT_SECONDS)
    calls_path = os.path.join(directory, "recover-calls")
    calls = open(calls_path, encoding="utf-8").read().splitlines() if os.path.exists(calls_path) else []
    logged = [line for line in (open(log, encoding="utf-8").read().splitlines() if os.path.exists(log) else []) if line.startswith("selftest-recovered ")]
    alive = [thread.name for thread in threads if thread.is_alive()]
    if alive or len(calls) != 1 or len(logged) != 1 or sorted(results.values()) != [(True, ""), (True, "")] or SERVER.PEER_CLEANUP_LOCKS:
        raise SelftestFailure(f"两个线程同时确认同一次尝试的清理：--recover-cleanup 应当只起一趟、日志只追加一份、两边都交确认了、锁表空着，"
                              f"实际起了 {len(calls)} 趟、日志 {len(logged)} 份、两边交 {results}、还没退的线程 {alive}、锁表 {list(SERVER.PEER_CLEANUP_LOCKS)}")
    return "两个线程同时确认同一次尝试的清理：恢复只起一趟、日志一份，后到的读到 confirmed；锁表空着"


# ── 全量严格审核那一批（records/2026-11-07-task-server全量严格审核报告.md 缺陷一、三、四、五、七、八、九与整包测试的判法）──
TAMPERED_HOLDER_KEY = "selftest-tampered-holder"   # tampered-attempt-ended-once：在跑那一条的命令键（记 10G 峰值，上限与记账 15G）
TAMPERED_WAITER_KEY = "selftest-tampered-waiter"   # 那一格后交的那一条：同样 15G；本机 30G 放不下两条，前一条还算在跑就排着
TAMPERED_FOREIGN_COLUMN = 999999                   # 改到任务行关联列上的号（库里没有这一号的任务）
TAMPERED_SUMMARY_READS = 3                         # 收尾之后再读几回 status：每读一回都不该多一条告警
TAMPERED_WAITER_SECONDS = 30.0                     # 后交的那一条等跑起来的安静时限
CONFIRM_LOCK_PROBE_SECONDS = 5.0                   # concurrent-confirms-serialized：头一条停在核过数之后，第二条至多等这么久看它自己退没退


def verifying_store(world, environment=None):
    """照服务读回的规则（task_server.py 的 Store.verified：校验列、正文与几列、关联列）读这一格的库，不建表、不写：交一个只读用的 Store。
    environment 缺省是空的（弄坏开关不进判法，判的是真的规则）。"""
    store = object.__new__(SERVER.Store)
    store.path, store.key, store.local = os.path.join(world.state, lib.DATABASE_NAME), lib.read_signing_key(world.key), threading.local()
    store.environment = {} if environment is None else environment
    return store


def verified_row(world, table, row_id):
    """(正文, 照服务的规则核不核得过, 关联列 task_id)；库里没有交 (None, False, None)。"""
    store = verifying_store(world)
    found = store.select(table, "id = ?", (row_id,))
    if not found:
        return None, False, None
    (column,) = store.connection().execute(f"SELECT task_id FROM {table} WHERE id = ?", (row_id,)).fetchone()
    return found[0][0], found[0][1], column


def tampered_alerts(world, task_id):
    return [alert for alert in world.alerts() if alert.get("kind") == "record-tampered" and alert.get("task") == task_id]


def cell_tampered_attempt_ended_once(world):
    """核不过的行重签之后核得过、只告警一次（缺陷一；task_server.py 的 Store.write_row、Scheduler.tampered、Scheduler.end_tampered_attempt）：
    一条在排的任务只改它的关联列：服务记 record-tampered，那一行重签之后照服务的规则核得过，之后再读 TAMPERED_SUMMARY_READS 回 status 告警仍只一条；
    一条在跑的任务（本机 30G 上记 15G）只改它那一次尝试的关联列：对账记任务 record-tampered、那一次记 ended（核得过、关联列照正文挂回它名下），告警一条；
    之后交的那一条（同样记 15G）不再被它占着的内存挡住、照起。弄坏开关 tampered-rows-left-unrepaired 下关联列不写回、尝试留在在跑里，这一格红。"""
    world.resources(local=(30, 30), hold=True)
    world.seed_peak(TAMPERED_HOLDER_KEY, DRAINED_FRAGMENT_PEAK_BYTES)
    world.seed_peak(TAMPERED_WAITER_KEY, DRAINED_FRAGMENT_PEAK_BYTES)
    world.start_server()
    problems = []
    queued = world.submit(["bash", "-c", "exit 0", "tampered-queued"])
    raw_column_update(world, "tasks", TAMPERED_FOREIGN_COLUMN, queued)
    summary = world.finished(queued, timeout=20)
    for _ in range(TAMPERED_SUMMARY_READS):
        world.summary(queued)
    _, valid, column = verified_row(world, "tasks", queued)
    alerts = tampered_alerts(world, queued)
    if summary["task"].get("outcome") != "record-tampered" or not valid or column != 0 or len(alerts) != 1:
        problems.append(f"在排的任务只改了关联列：应当记 record-tampered、重签之后核得过（关联列 0）、再读 {TAMPERED_SUMMARY_READS} 回告警仍只 1 条，"
                        f"实际 {describe(summary)}、核{'得过' if valid else '不过'}（关联列 {column}）、告警 {len(alerts)} 条")
    world.resources(local=(30, 30))
    release = os.path.join(world.root, "release-tampered")
    try:
        holder = world.submit(blocking_command(release), extra={lib.KEY_VARIABLE: TAMPERED_HOLDER_KEY})
        attempt_id = world.running(holder)["attempts"][0]["id"]
        raw_column_update(world, "attempts", 0, attempt_id)
        finished = world.finished(holder, timeout=20)
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        record, valid, column = verified_row(world, "attempts", attempt_id)
        while patience.waiting() and not (valid and (record or {}).get("state") == "ended"):
            time.sleep(POLL_SECONDS)
            record, valid, column = verified_row(world, "attempts", attempt_id)
        if finished["task"].get("outcome") != "record-tampered" or not valid or (record or {}).get("state") != "ended" \
                or (record or {}).get("outcome") != "record-tampered" or column != holder:
            problems.append(f"在跑那一次只改了关联列：任务应当记 record-tampered、那一次记 ended 与 record-tampered、核得过、关联列挂回任务 {holder}，"
                            f"实际任务 {describe(finished)}、那一次 {json.dumps(record, ensure_ascii=False)[:300]}、核{'得过' if valid else '不过'}、关联列 {column}")
        for _ in range(TAMPERED_SUMMARY_READS):
            world.summary(holder)
        if len(tampered_alerts(world, holder)) != 1:
            problems.append(f"在跑那一次核不过：告警应当只 1 条，实际 {len(tampered_alerts(world, holder))} 条")
        waiter = world.submit(blocking_command(release), extra={lib.KEY_VARIABLE: TAMPERED_WAITER_KEY})
        try:
            world.wait_for(waiter, lambda summary: any(attempt.get("state") == "running" for attempt in summary["attempts"]), "跑起来",
                           timeout=TAMPERED_WAITER_SECONDS)
        except SelftestFailure:
            problems.append(f"核不过的那一次记了 ended 之后不再占内存：后交的那一条（记 15G）应当在本机 30G 上照起，实际 {describe(world.summary(waiter))}、"
                            f"排队原因「{world.summary(waiter).get('waiting_for')}」")
    finally:
        open(release, "w").close()
    world.finished(waiter)
    if problems:
        raise SelftestFailure("核不过的行收尾：" + "；".join(problems), world.server_log_lines())
    return "关联列被改的任务行与尝试行：重签之后核得过、告警各一条，在跑那一次记 ended、不再占内存，后交的照起"


def cell_tampered_attempt_unit_stopped(world):
    """核不过的在跑尝试，服务停它的单元（task_server.py 的 stop_tampered_unit）：只改那一次尝试的关联列，对账记 ended 之后，它的单元回读不在跑、
    告警写 tampered-unit-stop 与「已停」。行的正文不可信那一半（单元名不合、是别的在跑尝试的不停）静态调 stop_tampered_unit 核：假 systemctl 没被调。
    弄坏开关 tampered-unit-left-running 下单元接着跑、没有这条告警，这一格红。"""
    scheduler = unittest.mock.Mock(environment={})
    scheduler.store.tag = "selftest"
    scheduler.store.running_attempts.return_value = [{"unit": "singlefs-task-selftest-1-1.service"}]
    for name in ("sshd.service", "singlefs-task-other-1-1.service", "singlefs-task-selftest-1-1.service", None):
        SERVER.Scheduler.stop_tampered_unit(scheduler, name, 1, None)
    lines = [call.args[2] for call in scheduler.alert.call_args_list]
    if len(lines) != 4 or not all("没停" in line for line in lines):
        raise SelftestFailure(f"单元名不是这个服务起的、或是校验过的在跑尝试的，应当都不停、照实报「没停」，实际 {lines}")
    world.resources(local=(30, 30))
    world.start_server()
    release = os.path.join(world.root, "release-tampered-unit")
    try:
        holder = world.submit(blocking_command(release))
        attempt = world.running(holder)["attempts"][0]
        unit = attempt.get("unit") or ""
        raw_column_update(world, "attempts", 0, attempt["id"])
        world.finished(holder, timeout=20)
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        alerts = []
        while patience.waiting() and not alerts:
            alerts = [alert for alert in world.alerts() if alert.get("kind") == "tampered-unit-stop" and alert.get("task") == holder]
            time.sleep(POLL_SECONDS)
        active = subprocess.run(["systemctl", "--user", "is-active", unit], capture_output=True, text=True).stdout.strip() if unit else ""
        if not unit or not alerts or "已停" not in alerts[0].get("line", "") or active in lib.RUNNING_UNIT_STATES:
            raise SelftestFailure(f"核不过的在跑尝试应当被停掉：单元 {unit or '（没记）'} 回读 {active or '没读'}、告警 {alerts}", world.server_log_lines())
    finally:
        open(release, "w").close()
    return "核不过的在跑尝试记 ended 之后，它的单元被停掉、回读不在跑，告警写已停"


def cell_full_gate_flag(world):
    """全量门禁标识（task_server.py 的 full_gate_refusal；异常表 full-gate-not-commit-entry）：提交时门禁带 --full-gate 照收、status 打「全量门禁」；
    范围声明写了提交前而没带它的退 3、出路写 --full-gate；带它又带 --check 的、带在别的命令上的都退 3。
    弄坏开关 full-gate-any-command-accepted 下后两条照收，这一格红。"""
    world.start_server()
    problems = []
    gate = ["bash", FAKE_COMMIT_GATE]
    accepted = submit_raw(world, gate, world.repo, full_gate=True)
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", accepted.stdout)
    if accepted.returncode != 0 or not found or "全量门禁" not in world.client(["status", found.group(1)]).stdout:
        problems.append(f"提交时门禁带 --full-gate 应当照收、status 打「全量门禁」，实际退 {accepted.returncode}：「{accepted.stderr.strip()[-200:]}」")
    declared = submit_raw(world, gate, world.repo, scope=SCOPE_COMMIT, full_gate=False)
    if declared.returncode != 3 or "--full-gate" not in declared.stderr:
        problems.append(f"范围声明写了提交前而没带 --full-gate 应当退 3、出路写它，实际退 {declared.returncode}：「{declared.stderr.strip()[-200:]}」")
    undeclared = submit_raw(world, gate, world.repo, full_gate=False)
    if undeclared.returncode != 3 or "--full-gate" not in undeclared.stderr:
        problems.append(f"不指定范围的提交时门禁没带 --full-gate（范围声明没写提交前）也应当退 3、出路写它，实际退 {undeclared.returncode}：「{undeclared.stderr.strip()[-200:]}」")
    confirmed_only = submit_raw(world, ["bash", "scripts/crash/crash-amplification.sh"], world.repo, full_gate=False,
                                confirmation=write_confirmation(world, "amplification-confirmed.md", WHOLE_PACKAGE_CONFIRMATION_TEXT))
    if confirmed_only.returncode != 3 or "--full-gate" not in confirmed_only.stderr:
        problems.append(f"崩溃放量不带 --item、只带整包确认文件而没带 --full-gate 应当退 3、出路写它，实际退 {confirmed_only.returncode}：「{confirmed_only.stderr.strip()[-200:]}」")
    for what, command in (("带 --check", [*gate, "--check", "lkmm-cell"]), ("别的命令", ["bash", "-c", "exit 0"])):
        completed = submit_raw(world, command, world.repo, full_gate=True)
        if completed.returncode != 3 or "--full-gate" not in completed.stderr:
            problems.append(f"{what}带 --full-gate 应当退 3，实际退 {completed.returncode}：「{completed.stderr.strip()[-200:]}」")
    if problems:
        raise SelftestFailure("全量门禁标识：" + "；".join(problems), world.server_log_lines())
    return "提交时门禁带全量标识照收；写了提交前没带的、带在别处的都退 3"


def cell_manual_submit_path(world):
    """人手交的认法（task_server.py 的 manual_refusal；异常表 manual-from-session、sessionless-unmarked）：没有会话带 --manual 的照收、记无会话、跑完；
    会话里带 --manual 的退 3；没有会话也没带 --manual 的，环境里填了收件地址也退 3、出路写 --manual。
    弄坏开关 manual-from-session-accepted、sessionless-unmarked-accepted 下对应那一条照收，这一格红。"""
    world.start_server()
    problems = []
    summary = world.finished(world.submit(["bash", "-c", "exit 0", "manual"], session=None))
    if summary["task"].get("outcome") != "succeeded" or summary["task"].get("session") != "" or summary["task"].get("inbox"):
        problems.append(f"人手带 --manual 应当照收、记无会话、不记收件地址，实际 {describe(summary)}、会话 {summary['task'].get('session')!r}")
    from_session = world.client(["submit", "--cwd", world.work, "--manual", "--", "bash", "-c", "exit 0"])
    if from_session.returncode != 3 or "--manual" not in from_session.stderr:
        problems.append(f"会话里带 --manual 应当退 3，实际退 {from_session.returncode}：「{from_session.stderr.strip()[-200:]}」")
    unmarked = world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0"], session=None,
                            extra={lib.INBOX_SOCKET_VARIABLE: "/nonexistent/inbox.sock", lib.INBOX_TOKEN_VARIABLE: INBOX_TOKEN_SAMPLE})
    if unmarked.returncode != 3 or "--manual" not in unmarked.stderr:
        problems.append(f"没有会话、环境里填了收件地址而没带 --manual 应当退 3、出路写 --manual，实际退 {unmarked.returncode}：「{unmarked.stderr.strip()[-200:]}」")
    if problems:
        raise SelftestFailure("人手交的认法：" + "；".join(problems), world.server_log_lines())
    return "人手带 --manual 照收、记无会话；会话带它、没有会话又没带它的都退 3"


RUNNING_ATTEMPTS_QUERY = "state IN ('starting', 'running')"
RUNNING_ATTEMPTS_QUERY_HOLDERS = ["reconcile", "running_attempts"]


def code_objects_of(module):
    """module 里定义的函数与方法（staticmethod、classmethod、property 拆开），连同套在里面的函数的代码对象。读编译出来的代码对象、不读源码原文：原文进这一份判定的键，注释一改键就变。"""
    pending, found = [], []
    for value in vars(module).values():
        members = vars(value).values() if isinstance(value, type) and value.__module__ == module.__name__ else (value,)
        for member in members:
            unwrapped = getattr(member, "__func__", member)
            function = getattr(unwrapped, "fget", unwrapped)
            if isinstance(function, types.FunctionType) and function.__module__ == module.__name__:
                pending.append(function.__code__)
    while pending:
        code = pending.pop()
        found.append(code)
        pending += [constant for constant in code.co_consts if isinstance(constant, types.CodeType)]
    return found


def cell_running_attempts_verified_only(world):
    """取在跑尝试只收校验过的行（缺陷五；task_server.py 的 Store.running_attempts，十处记账都调它）：这一格的临时库里两条在跑的尝试，后一条只改了关联列：
    running_attempts 只交前一条；server_pending（内存）、server_disk_pending 都只算前一条（套在假调度器上，不起服务）；
    task_server.py 里取在跑尝试的那一句查询只在 running_attempts 与对账（reconcile）两个函数里（读编译出来的代码对象，code_objects_of）。弄坏开关 running-attempts-unverified 下连那一行一起算，这一格红。"""
    store = SERVER.Store(os.path.join(world.root, "running.sqlite3"), bytes(32), dict(os.environ))
    common = {"state": "running", "session": "", "command_key": "", "host": SERVER.LOCAL, "number": 1, "started_at": time.time()}
    honest = store.insert("attempts", dict(common, task_id=1, memory_cap=4 * GIBIBYTE, disk_booked=2 * GIBIBYTE, threads=4))
    tampered = store.insert("attempts", dict(common, task_id=2, memory_cap=8 * GIBIBYTE, disk_booked=3 * GIBIBYTE, threads=8))
    with contextlib.closing(sqlite3.connect(store.path, timeout=30, isolation_level=None)) as connection:
        connection.execute("UPDATE attempts SET task_id = ? WHERE id = ?", (3, tampered["id"]))   # 只改关联列（TS-05 那一种）
    fake = object.__new__(SERVER.Scheduler)
    fake.store, fake.environment = store, dict(os.environ)
    fake.unit_stopped = set()   # booking_attempts 不数的、单元已确认停了的尝试号（Scheduler.__init__ 建的）
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(build_bytes=lambda attempt: 0)}
    fake.report_scope_usage = lambda report: {}
    problems = []
    listed = [attempt.get("id") for attempt in store.running_attempts()]
    if listed != [honest["id"]]:
        problems.append(f"running_attempts 应当只交校验过的那一条 {honest['id']}，实际 {listed}")
    checks = (("server_pending", SERVER.Scheduler.server_pending(fake, SERVER.LOCAL, None), 4 * GIBIBYTE),
              ("server_disk_pending", SERVER.Scheduler.server_disk_pending(fake, SERVER.LOCAL), 2 * GIBIBYTE))
    problems += [f"{name} 应当只算校验过的那一条（{wanted}），实际 {seen}" for name, seen, wanted in checks if seen != wanted]
    holders = sorted(code.co_name for code in code_objects_of(SERVER) if RUNNING_ATTEMPTS_QUERY in code.co_consts)
    if holders != RUNNING_ATTEMPTS_QUERY_HOLDERS:
        problems.append(f"task_server.py 里取在跑尝试的查询应当只在 {RUNNING_ATTEMPTS_QUERY_HOLDERS} 里，实际 {holders}：别处各写的一份不看校验位")
    if problems:
        raise SelftestFailure("在跑尝试只收校验过的：" + "；".join(problems))
    return "关联列被改的那一条在跑尝试不进 running_attempts，内存与盘的记账都不算它；那一句查询只在两个函数里"


def cell_concurrent_confirms_serialized(world):
    """同一会话的确认串行（缺陷七；task_server.py 的 session_lock）：同会话两条待确认、都照确认单（在排在跑 0 条）确认。头一条核过数之后停在改状态之前
    （TASK_SERVER_SELFTEST_PAUSE_POINT=confirm-counted），第二条在 CONFIRM_LOCK_PROBE_SECONDS 秒里应当一直等着会话锁、不退；放开之后头一条退 0，
    第二条照头一条进队之后的条数（1 条）核、对不上退 3、打出服务算的条数 1。弄坏开关 confirm-counts-outside-session-lock 下第二条不等、照 0 条核过退 0，这一格红。"""
    pause = os.path.join(world.root, "pause-confirm")
    os.makedirs(pause)
    world.resources(local=(30, 30), hold=True)
    world.start_server(extra={"TASK_SERVER_SELFTEST_PAUSE_POINT": f"confirm-counted:{pause}"})
    submitted = [world.client(["submit", "--cwd", world.work, "--", "bash", "-c", "exit 0", f"concurrent-{name}"]) for name in ("first", "second")]
    needed = [lib.confirm_needed_fields(completed.stdout) for completed in submitted]
    if any(fields is None for fields in needed):
        raise SelftestFailure("两条都应当待确认、打出确认单那一行", [completed.stdout + completed.stderr for completed in submitted])
    (first, first_items, first_tasks), (second, second_items, second_tasks) = needed
    open(os.path.join(pause, "arm-confirm-counted"), "w").close()
    leading = world.client(["confirm", str(first), "--items", str(first_items), "--tasks", str(first_tasks)], background=True)
    problems = []
    try:
        wait_path(world, os.path.join(pause, "reached-confirm-counted"), "头一条核过数、停在改状态之前")
        trailing = world.client(["confirm", str(second), "--items", str(second_items), "--tasks", str(second_tasks)], background=True)
        deadline = time.monotonic() + CONFIRM_LOCK_PROBE_SECONDS
        while time.monotonic() < deadline and trailing.poll() is None:
            time.sleep(POLL_SECONDS)
        early = trailing.poll() is not None
    finally:
        open(os.path.join(pause, "release-confirm-counted"), "w").close()
    leading_out, _ = leading.communicate(timeout=90)
    trailing_out, trailing_err = trailing.communicate(timeout=90)
    recounted = lib.confirm_needed_fields(trailing_out + "\n" + trailing_err)
    if early:
        problems.append(f"头一条拿着会话锁时第二条应当等着、不退，实际 {CONFIRM_LOCK_PROBE_SECONDS:.0f} 秒里退了 {trailing.returncode}")
    if leading.returncode != 0:
        problems.append(f"头一条应当确认成、退 0，实际退 {leading.returncode}：「{leading_out.strip()[-200:]}」")
    if trailing.returncode != 3 or recounted is None or recounted[2] != second_tasks + 1:
        problems.append(f"第二条应当照头一条进队之后的条数核：退 3、确认单那一行写 tasks={second_tasks + 1}，实际退 {trailing.returncode}、"
                        f"那一行 {recounted}（{(trailing_out + trailing_err).strip()[-300:]}）")
    world.resources(local=(30, 30))
    world.finished(first)
    world.client(["cancel", str(second)])
    if problems:
        raise SelftestFailure("同会话的确认串行：" + "；".join(problems), world.server_log_lines())
    return "同会话两条并发确认：第二条等头一条改完状态、照进队之后的条数核，对不上退 3"


def cell_resident_entries_forgotten(world):
    """按任务号记在内存里的表随结局回收（缺陷九；task_server.py 的 forget_settled_entries，套在假调度器上，不起服务）：这一格的临时库里五条任务——
    有了结局并被取走的、有了结局没人取而结局才一分钟的、有了结局没人取而结局满 RESIDENT_RETENTION_SECONDS 的、还在排的、库里没有的；
    七张按任务号记的表与 inbox_failed 里各有它们的项，清一回只留没人取的新结局与还在排的；按尝试号记的只留在跑的那一次。
    弄坏开关 resident-entries-kept 下一项都不删，这一格红。"""
    store = SERVER.Store(os.path.join(world.root, "resident.sqlite3"), bytes(32), dict(os.environ))
    now = time.time()

    def task(**fields):
        return store.insert("tasks", dict({"task_id": 0, "state": "finished", "session": "", "command_key": "", "collected": False, "finished_at": now - 60}, **fields))["id"]

    collected, fresh = task(collected=True), task()
    stale, queued = task(finished_at=now - SERVER.RESIDENT_RETENTION_SECONDS - 60), task(state="queued", finished_at=None)
    gone = queued + 1000
    running = store.insert("attempts", {"task_id": queued, "number": 1, "state": "running", "session": "", "command_key": ""})["id"]
    ended = store.insert("attempts", {"task_id": collected, "number": 1, "state": "ended", "session": "", "command_key": ""})["id"]
    ids = (collected, fresh, stale, queued, gone)
    fake = object.__new__(SERVER.Scheduler)
    fake.store, fake.environment, fake.resident_pruned_at, fake.resident_prune_seconds = store, dict(os.environ), 0.0, 0.0
    fake.alerted_child_refusals = {(task_id, "名下有校验不过的子任务记录") for task_id in ids}
    fake.reminded_thresholds = {(task_id, "quiet-log-busy"): 1 for task_id in ids}
    fake.reminder_histories = {(task_id, "quiet-log-busy"): [] for task_id in ids}
    fake.workspace_write_checks = {task_id: None for task_id in ids}
    fake.advance_episodes = {task_id: 1 for task_id in ids}
    # 删了 advance_failures、advance_episodes 的项，库里那一份跟着写（persist_restart_surviving_state）：Scheduler.__init__ 建的那几样照建
    fake.advance_failures = {task_id: {"episode": 1, "why": "推不动", "count": 1} for task_id in ids}
    fake.attention_lock = threading.Lock()
    fake.restart_surviving_lock, fake.restart_surviving_written = threading.Lock(), {}
    fake.unclaimed_repushed = {task_id: 1 for task_id in ids}
    fake.suspect_reported, fake.suspect_lock, fake.health_notified = {running: 1, ended: 1}, threading.Lock(), {}   # 报过的段随记进库（persist_restart_surviving_state）：这几样照建
    fake.booking_drift_notified, fake.booking_drift_lock = {}, threading.Lock()
    fake.inbox_failed = {("outcome", task_id) for task_id in ids} | {(task_id, "quiet-log-busy", 1) for task_id in ids} | {("suspect", running, 1), ("suspect", ended, 1)}
    SERVER.Scheduler.forget_settled_entries(fake, now)
    kept = {fresh, queued}
    seen = {name: {key[position] if position is not None else key for key in vars(fake)[name]} for name, position in SERVER.RESIDENT_TASK_TABLES.items()}
    problems = [f"{name} 应当只留任务 {sorted(kept)} 的项，实际 {sorted(found)}" for name, found in seen.items() if found != kept]
    if set(fake.suspect_reported) != {running}:
        problems.append(f"suspect_reported 应当只留在跑的那一次 {running}，实际 {sorted(fake.suspect_reported)}")
    wanted_events = {("outcome", task_id) for task_id in kept} | {(task_id, "quiet-log-busy", 1) for task_id in kept} | {("suspect", running, 1)}
    if fake.inbox_failed != wanted_events:
        problems.append(f"inbox_failed 应当只留 {sorted(map(str, wanted_events))}，实际 {sorted(map(str, fake.inbox_failed))}")
    if problems:
        raise SelftestFailure("内存里的表随结局回收：" + "；".join(problems))
    return "取走的、结局满保留时限的、库里没有的任务的项都删了，没人取的新结局与在排的留着；按尝试号记的只留在跑的"


WHOLE_PACKAGE_SINGLE_CELLS = (   # whole-package-single-cells-refused：(说什么, 命令, 该不该拒)，都在假仓根起
    ("单点 feature-gated-tests", ["bash", ".claude/gate.d/crash-point-replay.sh", ".", "--check", "feature-gated-tests"], True),
    ("单点 product-gpu-real-card-tests", ["bash", ".claude/gate.d/crash-point-replay.sh", ".", "--check", "product-gpu-real-card-tests"], True),
    ("单点 crash-case-replay 不带 --crash-cases-only", ["bash", ".claude/gate.d/crash-point-replay.sh", ".", "--check", "crash-case-replay"], True),
    ("merge-and-verify.sh 带 --all-packages", ["bash", "scripts/gate/merge-and-verify.sh", "--all-packages", "selftest-patch-directory"], True),
    ("--check crash-case-replay --crash-cases-only --item crash-case:x", ["bash", ".claude/gate.d/crash-point-replay.sh", ".", "--check", "crash-case-replay",
                                                                         "--crash-cases-only", "--item", "crash-case:x"], False))
FEATURE_GATED_CHECK = ["bash", lib.WHOLE_PACKAGE_CRASH_GATE, ".", "--check", "feature-gated-tests"]   # whole-package-feature-items-* 两格在假仓根起的那一格
FEATURE_ITEMS_ACCEPTED_CASES = (   # whole-package-feature-items-accepted：(说什么, 命令, 该不该拒)
    ("--check feature-gated-tests 点一个挂特性的文件", FEATURE_GATED_CHECK + ["--item", lib.CRASH_GATE_FEATURE_ITEM_PREFIX + "selftest_feature.rs"], False),
    ("--check feature-gated-tests 点库与一个挂特性的文件（--item= 写法）", FEATURE_GATED_CHECK + ["--item=" + lib.CRASH_GATE_FEATURE_ITEM_PREFIX + "lib",
                                                                                    "--item", lib.CRASH_GATE_FEATURE_ITEM_PREFIX + "selftest_feature.rs"], False))
FEATURE_ITEMS_OTHERS_REFUSED_CASES = (   # whole-package-feature-items-others-refused：(说什么, 命令, 该不该拒)
    ("--check feature-gated-tests 只点崩溃枚举用例", FEATURE_GATED_CHECK + ["--item", "crash-case:x"], True),
    ("--check feature-gated-tests,crash-case-replay --crash-cases-only 只点崩溃枚举用例",
     ["bash", lib.WHOLE_PACKAGE_CRASH_GATE, ".", "--check", "feature-gated-tests,crash-case-replay", "--crash-cases-only", "--item", "crash-case:x"], True))
UNSCOPED_RUN_CASES = (   # unscoped-runs-refused：(说什么, 命令, 范围声明, 该不该拒)，都在假仓根起
    ("提交时的门禁不带 --check、带全量标识", ["bash", ".claude/gate.d/lkmm.sh"], SCOPE_COMMIT, False),
    ("crash-amplification.sh 不带 --item、带全量标识", ["bash", "scripts/crash/crash-amplification.sh"], SCOPE_COMMIT, False),
    ("crash-amplification.sh 带 --item", ["bash", "scripts/crash/crash-amplification.sh", "--item", "selftest-part"], SCOPE_DAILY, False),
    ("scripts/crash/crash-point-replay.sh 不带 --item", ["bash", "scripts/crash/crash-point-replay.sh"], SCOPE_DAILY, True),
    ("replay.sh 不给实验号", ["bash", "scripts/experiment-apparatus/replay.sh"], SCOPE_DAILY, True),
    ("replay.sh 给了实验号", ["bash", "scripts/experiment-apparatus/replay.sh", "E1"], SCOPE_DAILY, False))


def submitted_refusal_problems(world, cases, refusal_text):
    """逐条只交不确认（submit_raw）：该拒的应当退 3、stderr 带 refusal_text 与 --whole-package-confirmed 那条出路；该收的退 0、打任务号（收下的随即取消）。→ 不对的那几句。"""
    problems = []
    for case in cases:
        what, command, scope, refused = case if len(case) == 4 else (case[0], case[1], SCOPE_DAILY, case[2])
        completed = submit_raw(world, command, world.repo, scope=scope)
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if found:
            world.client(["cancel", found.group(1)])
        if refused and (completed.returncode != 3 or refusal_text not in completed.stderr or "--whole-package-confirmed" not in completed.stderr):
            problems.append(f"{what}应当退 3、stderr 写「{refusal_text}」与出路，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」")
        elif not refused and (completed.returncode != 0 or not found):
            problems.append(f"{what}应当照收、退 0，实际退 {completed.returncode}：「{completed.stderr.strip()[-300:]}」")
    return problems


def cell_whole_package_single_cells_refused(world):
    """整包测试照命令实际跑什么判（task_server_lib.py 的 whole_package_script；.claude/agent-common.md「日常开发的测试不跑整个包」）：crash-point-replay.sh 的 --check
    单点 feature-gated-tests、单点 product-gpu-real-card-tests、单点 crash-case-replay 而不带 --crash-cases-only，merge-and-verify.sh 带 --all-packages，没带确认标识都退 3；
    --check crash-case-replay --crash-cases-only --item crash-case:x 照收。弄坏开关 whole-package-cells-all-three-only 下单点的三条照收，
    merge-all-packages-accepted 下带 --all-packages 的照收，这一格红。"""
    world.start_server()
    problems = submitted_refusal_problems(world, WHOLE_PACKAGE_SINGLE_CELLS, "整包测试没带确认标识")
    if problems:
        raise SelftestFailure("整包测试单点一格也算：" + "；".join(problems), world.server_log_lines())
    return f"{len(WHOLE_PACKAGE_SINGLE_CELLS) - 1} 种整包写法退 3，只判点名崩溃枚举用例的那一条照收"


def cell_whole_package_feature_items_accepted(world):
    """crash-point-replay.sh --check feature-gated-tests 点了 feature-gated:<文件名> 项的不是整包测试（lib.whole_package_script；那一格只编、只跑点名的挂特性文件）：
    FEATURE_ITEMS_ACCEPTED_CASES 两条（--item 与 --item= 两种写法）都照收、退 0。弄坏开关 whole-package-feature-items-uncounted 下照整包拒，这一格红。"""
    world.start_server()
    problems = submitted_refusal_problems(world, FEATURE_ITEMS_ACCEPTED_CASES, "整包测试没带确认标识")
    if problems:
        raise SelftestFailure("feature-gated-tests 点了挂特性的文件：" + "；".join(problems), world.server_log_lines())
    return f"--check feature-gated-tests 点了挂特性文件的 {len(FEATURE_ITEMS_ACCEPTED_CASES)} 种写法都照收"


def cell_whole_package_feature_items_others_refused(world):
    """crash-point-replay.sh --check 点了 feature-gated-tests、--item 只点了崩溃枚举用例的照旧是整包测试（那一格照跑整格）：FEATURE_ITEMS_OTHERS_REFUSED_CASES 两条都退 3、
    stderr 写「整包测试没带确认标识」与 --whole-package-confirmed 那条出路。弄坏开关 whole-package-any-item-exempts 下带了 --item 就照收，这一格红。"""
    world.start_server()
    problems = submitted_refusal_problems(world, FEATURE_ITEMS_OTHERS_REFUSED_CASES, "整包测试没带确认标识")
    if problems:
        raise SelftestFailure("feature-gated-tests 没点挂特性的文件：" + "；".join(problems), world.server_log_lines())
    return f"--check feature-gated-tests 只点崩溃枚举用例的 {len(FEATURE_ITEMS_OTHERS_REFUSED_CASES)} 种写法都退 3"


CARD_GATE_ENTRY = ".claude/gate.d/crash-point-replay.sh"   # attempt-gets-no-card：假仓根下的假门禁（把自己的环境写进一份文件，不起真门禁）
CARD_CHOICE_SAMPLE = {"GPU_CARDS_BY_PRIORITY": LOCAL_CARD_ZERO, "CUDA_VISIBLE_DEVICES": "0"}   # 那一格服务自己的环境里带着的选卡变量
CARD_CHANNEL_PREFIX = "TASK_SERVER_CARD_LEASE"   # 领卡通道交给命令的变量与客户端打的领卡行都以它起头：命令的环境与尝试的日志里都不该有


def cell_attempt_gets_no_card(world):
    """任务不拿卡（task_server.py 的 start_attempt、attempt_command；.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：服务自己的环境里带着
    GPU_CARDS_BY_PRIORITY 与 CUDA_VISIBLE_DEVICES，交一条点名的假门禁 crash-point-replay（假仓根下的脚本，把自己的环境写进一份文件，不起真门禁）：
    命令的环境里两个选卡变量与 CARD_CHANNEL_PREFIX 起头的都没有；尝试的说明里起的那一条外面没套清场包装（gpu-quiesce-run.sh）；
    尝试的日志里没有 CARD_CHANNEL_PREFIX 起头的行；任务跑完、退 0。弄坏开关 card-variables-passed-to-attempt 下服务环境里的选卡变量照交给命令，这一格红。"""
    seen = os.path.join(world.root, "card-gate-environment")
    entry = os.path.join(world.repo, CARD_GATE_ENTRY)
    os.makedirs(os.path.dirname(entry), exist_ok=True)
    write_executable(entry, f'#!/usr/bin/env bash\nenv > {shlex.quote(seen)}\necho "假门禁：$*"\nexit 0\n')
    world.resources(local=(30, 30))
    world.start_server(extra=CARD_CHOICE_SAMPLE)
    task_id = world.submit(["bash", CARD_GATE_ENTRY, ".", "--check", "crash-case-replay", "--crash-cases-only", "--item", "crash-case:selftest"],
                           extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY}, cwd=world.repo)
    summary = world.finished(task_id)
    attempt = first_attempt(summary)
    problems = []
    if summary["task"].get("outcome") != "succeeded":
        problems.append(f"假门禁应当照跑完、退 0，实际 {describe(summary)}")
    try:
        with open(seen, encoding="utf-8") as handle:
            names = {line.split("=", 1)[0] for line in handle if "=" in line}
        leaked = sorted(name for name in names if name in lib.CARD_CHOICE_VARIABLES or name.startswith(CARD_CHANNEL_PREFIX))
        if leaked:
            problems.append(f"命令的环境里不该有选卡变量与领卡通道，实际有 {leaked}")
    except OSError as error:
        problems.append(f"假门禁没写下它的环境（{error}）")
    try:
        with open(attempt.get("spec") or "", encoding="utf-8") as handle:
            argv = json.load(handle).get("argv") or []
        wrapped = [word for word in argv if os.path.basename(str(word)) == "gpu-quiesce-run.sh"]
        if not argv or wrapped:
            problems.append(f"起的那一条外面不该套清场包装，实际说明里的 argv {argv[:8]}")
    except (OSError, ValueError) as error:
        problems.append(f"尝试的说明读不了（{error}）")
    with contextlib.suppress(OSError):
        with open(attempt.get("log") or "", encoding="utf-8", errors="replace") as handle:
            marked = [line.strip() for line in handle if line.startswith(CARD_CHANNEL_PREFIX)]
        if marked:
            problems.append(f"尝试的日志里不该有 {CARD_CHANNEL_PREFIX} 起头的行，实际 {marked[:3]}")
    if problems:
        raise SelftestFailure("任务不拿卡：" + "；".join(problems), world.server_log_lines())
    return "假门禁照跑完：命令的环境里没有选卡变量与领卡通道、外面没套清场、日志里没有领卡行"


CARD_DECLARATION_SAMPLES = (   # card-declarations-refused：(脚本名, 文件头那一行)，各写一种卡的声明
    ("declared-card-lease.sh", f"{lib.CARD_DECLARATION_MARKS[0]} on-demand"),
    ("declared-gpu-cards.sh", f"{lib.CARD_DECLARATION_MARKS[1]} 2"),
    ("declared-gpu-arguments.sh", f"{lib.CARD_DECLARATION_MARKS[2]} *"),
)


def cell_card_declarations_refused(world):
    """入口文件头写了领卡、要卡的声明当场拒（task_server.py 文件头 ②「提交时拒」⑧；task_server_lib.py 的 card_declaration_problem）：假仓根下三份脚本
    各写一种（CARD_DECLARATION_SAMPLES），直接交与套在 bash -c 里交都退 3、不给任务号、stderr 写 lib.CARD_DECLARATION_REFUSAL、点名那份脚本与写了的那一种、
    出路写删掉那一行；同目录一份不写这几种的照收、跑完；客户端 lease take 认不出（退 2）。弄坏开关 card-declarations-accepted 下写了的照收，这一格红。"""
    world.resources(local=(30, 30))
    world.start_server()
    problems = []
    for name, header in CARD_DECLARATION_SAMPLES:
        write_executable(os.path.join(world.repo, name), f"#!/usr/bin/env bash\n{header}\nexit 0\n")
        for form, command in (("直接交", ["bash", name]), ("bash -c 里交", ["bash", "-c", f"bash {name}"])):
            completed = submit_raw(world, command, world.repo)
            found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
            if found:
                world.client(["cancel", found.group(1)])
            texts = (lib.CARD_DECLARATION_REFUSAL, name, header.split()[1], "删掉")
            if completed.returncode != 3 or found or not all(text in completed.stderr for text in texts):
                problems.append(f"{name}（{form}）应当退 3、不给任务号、stderr 写「{'」「'.join(texts)}」，实际退 {completed.returncode}、"
                                f"stderr「{completed.stderr.strip()[-300:]}」")
    write_executable(os.path.join(world.repo, "declared-nothing.sh"), "#!/usr/bin/env bash\nexit 0\n")
    try:
        plain = world.finished(world.submit(["bash", "declared-nothing.sh"], extra={"SINGLEFS_RUN_SCOPE": SCOPE_DAILY}, cwd=world.repo))
        if plain["task"].get("outcome") != "succeeded":
            problems.append(f"不写卡的声明的那一份应当照收、跑完，实际 {describe(plain)}")
    except SelftestFailure as failure:
        problems.append(f"不写卡的声明的那一份应当照收，实际 {failure}")
    lease = world.client(["lease", "take"])
    if lease.returncode != lib.USAGE_EXIT:
        problems.append(f"客户端 lease take 应当认不出、退 {lib.USAGE_EXIT}，实际退 {lease.returncode}、「{(lease.stdout + lease.stderr).strip()[-200:]}」")
    if problems:
        raise SelftestFailure("卡的声明当场拒：" + "；".join(problems), world.server_log_lines())
    return f"{len(CARD_DECLARATION_SAMPLES)} 种卡的声明直接交与 bash -c 里交都退 3、写明卡归 GPU 服务；不写的照收；lease take 认不出"


def cell_unscoped_runs_refused(world):
    """不指定范围的跑法在提交之外交当场拒（task_server_lib.py 的 unscoped_run_entry 与 UNSCOPED_RUN_ENTRIES；.claude/main-agent.md「一轮怎么开、怎么收」）：
    UNSCOPED_RUN_CASES 里不带选条参数的都退 3、stderr 写「不指定范围的跑法没带确认标识」，带了选条参数的与提交时的门禁（崩溃放量入口在内，带全量标识）照收；
    不是提交时门禁的入口（scripts/crash/crash-point-replay.sh）不指定范围、带上整包确认文件照收；提交时门禁没带全量标识的拒法归 full-gate-flag 那一格。弄坏开关 unscoped-gate-accepted 下不带选条参数的照收，这一格红。"""
    world.start_server()
    problems = submitted_refusal_problems(world, UNSCOPED_RUN_CASES, "不指定范围的跑法没带确认标识")
    confirmed = submit_raw(world, ["bash", "scripts/crash/crash-point-replay.sh"], world.repo,
                           confirmation=write_confirmation(world, "unscoped-confirmed.md", WHOLE_PACKAGE_CONFIRMATION_TEXT))
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", confirmed.stdout)
    if found:
        world.client(["cancel", found.group(1)])
    if confirmed.returncode != 0 or not found:
        problems.append(f"不指定范围的那一条带上整包确认文件应当照收、退 0，实际退 {confirmed.returncode}：「{confirmed.stderr.strip()[-300:]}」")
    if problems:
        raise SelftestFailure("不指定范围的跑法：" + "；".join(problems), world.server_log_lines())
    return "不带选条参数的门禁与入口退 3，带了选条参数的、提交时的门禁、带了确认文件的照收"


UNNAMED_RUN_JUDGED = {lib.UNNAMED_RUN_SELFTEST_VARIABLE: ""}   # unnamed-* 几格起服务时写空串：提交时拒 ⑦ 照真的判（小世界缺省写 off）
UNNAMED_PATCH_DIRECTORY = "selftest-patch-directory"


def unnamed_run_problems(world, cases):
    """unnamed-* 几格共用（task_server.py 文件头 ②「提交时拒」⑦，判据 lib.unnamed_run_problem）：起照真的判 ⑦ 的服务，逐条只交不确认（submit_raw；
    cargo 与 bash -c 在这一格的工作目录起，同 whole-package-named-accepted，别的在假仓根起）。
    cases 是 (说什么, 命令, 拒绝说明里要有的改法；照收的写 None)：该拒的退 3、stderr 带 lib.UNNAMED_RUN_PROBLEM_HEAD 与那一句改法；该收的退 0、打任务号（随即取消）。→ 不对的那几句。"""
    world.start_server(extra=UNNAMED_RUN_JUDGED)
    problems = []
    for what, command, way_out in cases:
        completed = submit_raw(world, command, world.work if command[0] == "cargo" or command[:2] == ["bash", "-c"] else world.repo)
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if found:
            world.client(["cancel", found.group(1)])
        if way_out is not None and (completed.returncode != 3 or lib.UNNAMED_RUN_PROBLEM_HEAD not in completed.stderr or way_out not in completed.stderr):
            problems.append(f"{what}应当退 3、stderr 写「{lib.UNNAMED_RUN_PROBLEM_HEAD}」与改法「{way_out}」，实际退 {completed.returncode}：「{completed.stderr.strip()[-400:]}」")
        elif way_out is None and (completed.returncode != 0 or not found):
            problems.append(f"{what}应当照收、退 0，实际退 {completed.returncode}：「{completed.stderr.strip()[-400:]}」")
    return problems


def cell_unnamed_merge_unnamed_refused(world):
    """合入后验证不带 --item 当场拒（lib.unnamed_run_problem 的 UNNAMED_RUN_ITEM_ENTRIES）：给了补丁目录而没带 --item 的退 3、说明给出加 --item <包>:<目标根文件> 的样子。
    弄坏开关 unnamed-entry-item-unchecked、unnamed-run-refusal-skipped 下照收，这一格红。"""
    problems = unnamed_run_problems(world, (("合入后验证给了补丁目录、不带 --item", ["bash", FAKE_MERGE_ENTRY, UNNAMED_PATCH_DIRECTORY], "--item <包>:<目标根文件>"),))
    if problems:
        raise SelftestFailure("合入后验证不带 --item：" + "；".join(problems), world.server_log_lines())
    return "合入后验证不带 --item 退 3，说明给出加 --item 的样子"


def cell_unnamed_merge_named_accepted(world):
    """合入后验证带 --item 照收（--item <项> 与 --item=<项> 两种写法）。弄坏开关 unnamed-entry-item-uncounted 下带了也拒，这一格红。"""
    problems = unnamed_run_problems(world, (
        ("合入后验证带 --item", ["bash", FAKE_MERGE_ENTRY, "--skip-apply", "--item", "selftest-package:tests/selftest_target.rs", UNNAMED_PATCH_DIRECTORY], None),
        ("合入后验证带 --item=", ["bash", FAKE_MERGE_ENTRY, "--item=selftest-package:src/lib.rs", UNNAMED_PATCH_DIRECTORY], None)))
    if problems:
        raise SelftestFailure("合入后验证带 --item：" + "；".join(problems), world.server_log_lines())
    return "合入后验证带 --item 的两种写法都照收"


def cell_unnamed_cargo_target_refused(world):
    """cargo test 没点到 --exact 的单条测试函数当场拒（lib.unnamed_cargo_test_problem）：只点 --test 目标的、--lib 带筛选词的、bash -c 里只点目标的都退 3，
    说明给出 `-- <测试全名> --exact` 的样子。弄坏开关 unnamed-cargo-filter-accepted、unnamed-run-refusal-skipped 下照收，shell-inner-scope-unread 下 bash -c 那一条照收，这一格红。"""
    target = ["cargo", "test", "-p", "singlefs-verification", "--test", "selftest_target"]
    problems = unnamed_run_problems(world, (
        ("cargo test 只点 --test 目标", target, "-- <测试全名> --exact"),
        ("cargo test --lib 带筛选词不带 --exact", ["cargo", "test", "-p", "singlefs-verification", "--lib", "--", "selftest_name"], "-- selftest_name --exact"),
        ("bash -c 里 cargo test 只点 --test 目标", ["bash", "-c", shlex.join(target)], "-- <测试全名> --exact")))
    if problems:
        raise SelftestFailure("cargo test 没点到单条测试函数：" + "；".join(problems), world.server_log_lines())
    return "cargo test 只点目标、只给筛选词、bash -c 里只点目标的都退 3，说明给出 --exact 的样子"


def cell_unnamed_cargo_exact_accepted(world):
    """cargo test `--` 之后带 --exact 与测试全名的照收（两条全名一起点也算）；只编不跑的 --no-run 照收。弄坏开关 unnamed-cargo-exact-uncounted 下点名的也拒，这一格红。"""
    base = ["cargo", "test", "-p", "singlefs-verification", "--test", "selftest_target"]
    problems = unnamed_run_problems(world, (
        ("cargo test 点一条 --exact", base + ["--", "selftest_module::selftest_case", "--exact"], None),
        ("cargo test 点两条 --exact", base + ["--", "--exact", "selftest_module::replay_case", "selftest_module::merge_case"], None),
        ("cargo test --no-run", base + ["--no-run"], None)))
    if problems:
        raise SelftestFailure("cargo test 点名 --exact：" + "；".join(problems), world.server_log_lines())
    return "cargo test 点一条、两条 --exact 与 --no-run 都照收"


def cell_unnamed_prove_red_unnamed_refused(world):
    """证红不给变异名当场拒（lib.unnamed_run_problem 照 prove-red.sh 收参数的写法：--copy <副本> <crate> 之后至少一个变异名）：退 3、说明给出 <变异名…> 的样子。
    弄坏开关 unnamed-prove-red-unchecked、unnamed-run-refusal-skipped 下照收，这一格红。"""
    problems = unnamed_run_problems(world, (("证红不给变异名", ["bash", lib.UNNAMED_RUN_PROVE_RED_ENTRY, "--copy", world.repo, "singlefs-verification"], "<变异名…>"),))
    if problems:
        raise SelftestFailure("证红不给变异名：" + "；".join(problems), world.server_log_lines())
    return "证红不给变异名退 3，说明给出写变异名的样子"


def cell_unnamed_prove_red_named_accepted(world):
    """证红点名一个变异照收。弄坏开关 unnamed-prove-red-name-offset 下只点一个的算没点、拒，这一格红。"""
    problems = unnamed_run_problems(world, (("证红点名一个变异", ["bash", lib.UNNAMED_RUN_PROVE_RED_ENTRY, "--copy", world.repo, "singlefs-verification", "selftest-row"], None),))
    if problems:
        raise SelftestFailure("证红点名一个变异：" + "；".join(problems), world.server_log_lines())
    return "证红点名一个变异照收"


def cell_unnamed_exemptions_accepted(world):
    """两种例外不判点没点名：提交时的门禁不指定范围、带 --full-gate 照收；合入后验证不带 --item、带三段写全的 --whole-package-confirmed 照收；
    同一条合入后验证两样都不带的退 3（对照）。弄坏开关 unnamed-run-exemptions-ignored 下带了标识的也拒，这一格红。"""
    world.start_server(extra=UNNAMED_RUN_JUDGED)
    confirmation = write_confirmation(world, "unnamed-confirmed.md", WHOLE_PACKAGE_CONFIRMATION_TEXT)
    merge = ["bash", FAKE_MERGE_ENTRY, UNNAMED_PATCH_DIRECTORY]
    problems = []
    for what, completed, accepted in (
            ("提交时的门禁带 --full-gate", submit_raw(world, ["bash", ".claude/gate.d/crash-point-replay.sh"], world.repo, scope=SCOPE_COMMIT), True),
            ("合入后验证带 --whole-package-confirmed", submit_raw(world, merge, world.repo, confirmation=confirmation), True),
            ("合入后验证两样都不带", submit_raw(world, merge, world.repo), False)):
        found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
        if found:
            world.client(["cancel", found.group(1)])
        if accepted and (completed.returncode != 0 or not found):
            problems.append(f"{what}应当照收、退 0，实际退 {completed.returncode}：「{completed.stderr.strip()[-400:]}」")
        elif not accepted and (completed.returncode != 3 or lib.UNNAMED_RUN_PROBLEM_HEAD not in completed.stderr):
            problems.append(f"{what}应当退 3、写「{lib.UNNAMED_RUN_PROBLEM_HEAD}」，实际退 {completed.returncode}：「{completed.stderr.strip()[-400:]}」")
    if problems:
        raise SelftestFailure("点名的两种例外：" + "；".join(problems), world.server_log_lines())
    return "带 --full-gate 的提交时门禁与带整包确认文件的合入后验证照收，两样都不带的退 3"


def cell_unnamed_gate_cells_judged(world):
    """门禁阶段（lib.TEST_STARTING_GATE_CELLS）：--check 点了起测试的格（qemu-device-streams 的 device-streams）而没带 --item 的退 3、说明给出 --item 的样子；
    --item、--check 都不带的（doc-text 全部格）退 3；只点不起测试的格（crash-point-replay 的 crash-knobs）、带 --item 的照收。
    弄坏开关 unnamed-check-cells-accepted 下点起测试的格照收，unnamed-stage-whole-accepted 下两样都不带的照收，unnamed-run-refusal-skipped 下都照收，这一格红。"""
    problems = unnamed_run_problems(world, (
        ("--check 点起测试的格、不带 --item", ["bash", ".claude/gate.d/qemu-device-streams.sh", ".", "--check", "device-streams"], "--item <项>"),
        ("--item、--check 都不带", ["bash", ".claude/gate.d/doc-text.sh", "."], "--item <项>"),
        ("只点不起测试的格", ["bash", ".claude/gate.d/crash-point-replay.sh", ".", "--check", "crash-knobs"], None),
        ("带 --item", ["bash", ".claude/gate.d/qemu-device-streams.sh", ".", "--item", "selftest-mode"], None)))
    if problems:
        raise SelftestFailure("门禁阶段点没点名：" + "；".join(problems), world.server_log_lines())
    return "点起测试的格不带 --item、两样都不带的退 3，只点不起测试的格、带 --item 的照收"


UNNAMED_CRASH_GATE = ["bash", lib.WHOLE_PACKAGE_CRASH_GATE, "."]   # unnamed-crash-gate-* 两格在假仓根起的门禁 crash-point-replay


def cell_unnamed_crash_gate_whole_items_refused(world):
    """门禁 crash-point-replay 带了 --item 而 --check 没把起整个验证包的格排掉的当场拒（lib.crash_gate_whole_package_cells）：只点崩溃枚举用例、不带 --check 的
    （照旧九格，连按条复用那一趟）与带 --crash-cases-only 而不带 --check 的（产品 GPU 与挂特性两格照跑整格）都退 3，说明给出加 --check crash-case-replay --crash-cases-only 的样子。
    弄坏开关 unnamed-crash-gate-item-whole-accepted、unnamed-run-refusal-skipped 下照收，这一格红。"""
    problems = unnamed_run_problems(world, (
        ("点崩溃枚举用例、不带 --check", UNNAMED_CRASH_GATE + ["--item", "crash-case:x"], lib.CRASH_GATE_CASES_ONLY_FORM),
        ("带 --crash-cases-only、不带 --check", UNNAMED_CRASH_GATE + ["--crash-cases-only", "--item=crash-case:x"], lib.CRASH_GATE_CASES_ONLY_FORM)))
    if problems:
        raise SelftestFailure("门禁 crash-point-replay 带 --item 而没排掉整包的格：" + "；".join(problems), world.server_log_lines())
    return "门禁 crash-point-replay 点崩溃枚举用例而不带 --check 的两种写法都退 3，说明给出加 --check crash-case-replay --crash-cases-only 的样子"


def cell_unnamed_crash_gate_cases_only_accepted(world):
    """门禁 crash-point-replay 带了 --item 而起整个验证包的格都排掉了的照收：--check crash-case-replay --crash-cases-only 点崩溃枚举用例；
    不带 --check、只点挂特性的文件（只跑 feature-gated-tests 一格、只编点名的文件）。弄坏开关 unnamed-crash-gate-cases-only-uncounted 下前一条拒，这一格红。"""
    problems = unnamed_run_problems(world, (
        ("--check crash-case-replay --crash-cases-only 点崩溃枚举用例", UNNAMED_CRASH_GATE + ["--check", "crash-case-replay", "--crash-cases-only", "--item", "crash-case:x"], None),
        ("不带 --check、只点挂特性的文件", UNNAMED_CRASH_GATE + ["--item", lib.CRASH_GATE_FEATURE_ITEM_PREFIX + "selftest_feature.rs"], None)))
    if problems:
        raise SelftestFailure("门禁 crash-point-replay 只跑点名的项：" + "；".join(problems), world.server_log_lines())
    return "门禁 crash-point-replay 带 --crash-cases-only 点崩溃枚举用例、不带 --check 只点挂特性的文件都照收"


def cell_test_starting_cells_listed(world):
    """起测试的格表（lib.test_starting_gate_cells；手列，各道文件头的格名表不标哪一格起测试）里每一道的阶段文件在仓里、每个格名都在那一道文件头的格名表里
    （`# gate-cell:` 行，读法只用 scripts/gate/lib_stage_header.py 的 header_cells；那一道 --list 打的就是这张表，scripts/gate/gate-structure-check.py 判两边相同）。
    静态核，不起服务、不起门禁。弄坏开关 test-starting-cells-misnamed 下 lkmm 那一行的格名写错一个字，这一格红。"""
    header = lib.load_module("lib_stage_header", os.path.join(lib.REPOSITORY_ROOT, "scripts", "gate", "lib_stage_header.py"))
    table = lib.test_starting_gate_cells()
    problems = []
    for stage, cells in table.items():
        path = os.path.join(lib.REPOSITORY_ROOT, stage)
        if not os.path.isfile(path):
            problems.append(f"{stage} 不在仓里")
            continue
        listed = {name for _, name, _ in header.header_cells(path)}
        missing = [cell for cell in cells if cell not in listed]
        if missing:
            problems.append(f"{stage} 的格名表里没有 {'、'.join(missing)}（有 {'、'.join(sorted(listed))}）")
    if problems:
        raise SelftestFailure("起测试的格表与各道的格名表对不上（照那一道的 --list 改 task_server_lib.py 的 TEST_STARTING_GATE_CELLS）：" + "；".join(problems))
    return f"起测试的格表 {len(table)} 道、{sum(len(cells) for cells in table.values())} 格，格名都在各自那一道的格名表里"


def cell_queued_whole_package_rechecked(world):
    """服务起来时重核在排的照交任务那一刻同一份提交时拒判（task_server.py 的 submit_refusal 与 revalidate_queued）：在排的一条绕过服务改成整包测试
    （cargo test -p 不点名测试目标；演规则加上之前交进来的那一种），重启服务：它记 resubmit-required（退 232、一次没起、说明写整包测试那一句），
    task wait 拿到这个结局；合规的那一条照排。弄坏开关 queued-refusals-not-rechecked 下只核环境变量、包装与编译目录，整包测试照排，这一格红。"""
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    stale = world.submit(["bash", "-c", "exit 0", "rechecked-stale"])
    fine = world.submit(["bash", "-c", "exit 0", "rechecked-fine"])
    world.stop_server()
    world.rewrite_signed_task(stale, lambda record: record.update(command=["cargo", "test", "-p", "singlefs-verification"]))
    world.start_server()
    problems = []
    try:
        summary = world.finished(stale, timeout=20)
    except SelftestFailure:
        summary = world.summary(stale)
    waited = world.client(["wait", str(stale)]) if summary["task"].get("state") == "finished" else None   # 没收尾的不等（wait 会一直等下去）
    waiting = world.summary(fine)
    world.resources(local=(30, 30))
    world.finished(fine)
    if summary["task"].get("outcome") != "resubmit-required" or summary["task"].get("exit_code") != 232 or summary["attempts"] \
            or "整包测试没带确认标识" not in (summary["task"].get("note") or ""):
        problems.append(f"在排的整包测试重启之后应当记 resubmit-required（退 232、一次没起、说明写整包测试那一句），实际 {describe(summary)}、"
                        f"说明「{summary['task'].get('note')}」")
    if waited is None or "type=resubmit-required" not in waited.stdout:
        problems.append(f"task wait 应当拿到 resubmit-required 那一行，实际「{(waited.stdout.strip()[-300:] if waited else '没收尾，没等')}」")
    if waiting["task"].get("state") != "queued":
        problems.append(f"合规的那一条应当照排，实际 {waiting['task'].get('state')}")
    if problems:
        raise SelftestFailure("重启之后照同一份提交时拒重核：" + "；".join(problems), world.server_log_lines())
    return "在排的整包测试重启之后记 resubmit-required、wait 拿到结局，合规的照排"


def cell_retired_outcome_status_generic(world):
    """库里记着退役结局的旧任务（lib.RETIRED_OUTCOME_KINDS，结局 over-two-hours）：task status <号> 不崩、退 0，TASK_SERVER_REASON 的 next= 用通用那一句（lib.ANOMALY_GENERIC_ADVICE）。
    弄坏开关 retired-outcome-kinds-unknown 下抛 KeyError，这一格红。"""
    world.start_server()
    outcome = lib.RETIRED_OUTCOME_KINDS[0]
    task_id = world.insert_signed_row("tasks", {"command": ["bash", "-c", "exit 0", "retired-outcome"], "cwd": world.work, "environment": {}, "session": SESSION_A, "submitter_pid": 0,
                                                "submitted_at": time.time(), "finished_at": time.time(), "command_key": "selftest-retired-outcome", "state": "finished", "outcome": outcome,
                                                "exit_code": 1, "collected": True, "next_cap": None, "next_host": "", "outcome_counts": {}, "note": "", "cancel_requested": False,
                                                "attempt_count": 0, "task_id": 0})
    shown = world.client(["status", str(task_id)])
    advice = lib.ANOMALY_GENERIC_ADVICE.replace(lib.ANOMALY_TASK_PLACEHOLDER, str(task_id))
    reason = next((line for line in shown.stdout.splitlines() if line.startswith(lib.REASON_MARK)), "")
    if shown.returncode != 0 or f"type={outcome}" not in reason or advice not in reason:
        raise SelftestFailure(f"结局是 {outcome} 的旧任务：status 应当退 0、理由行 next= 用通用那一句「{advice}」，实际退 {shown.returncode}、理由行「{reason}」、"
                              f"输出「{(shown.stdout + shown.stderr).strip()[-400:]}」", world.server_log_lines())
    return f"结局是 {outcome} 的旧任务 status 不崩、next= 用通用那一句"


RELEASE_HOLD_SCRIPT = 'release="$1"; shift; printf "%s\\n" "$@"; for round in $(seq 1 600); do [[ -e "$release" ]] && exit 0; sleep 0.1; done; exit 9'   # 打几行、等放行才退
RELEASE_WAIT_SECONDS = 90.0            # 等判到哪一支的安静时限：开关下等不到的要等命令自己 60 秒退（RELEASE_HOLD_SCRIPT），比它长
HARD_LIMIT_OUT_OF_REACH_SECONDS = "1000000000"   # 起头那一段不该判到硬上限的格的上限（TASK_SERVER_SELFTEST_RUN_HARD_LIMIT_SECONDS）：格里跑多久都够不着


def held_command(release, *lines, name="held"):
    return ["bash", "-c", RELEASE_HOLD_SCRIPT, name, release, *lines]


RUN_HARD_LIMIT_EXIT = 229                     # over-four-hours 的退出码（task_server_lib.py 的 OUTCOME_TYPES）
RUN_HARD_LIMIT_NOTE = "硬上限"                 # 结局说明、推送正文与确认单都写这个词
RUN_HARD_LIMIT_ADVICE = ".claude/rules/long-run-split.md"   # 修改意见里的出路：指到拆分规则，不再写缩域、收小范围
RUN_HARD_LIMIT_SHARD_SECONDS = 15             # run-hard-limit-shard-stopped：界要够那一件走到第 0、1 片在跑、第 2 片排着
RUN_HARD_LIMIT_CONFIRM_SECONDS = 10           # run-hard-limit-why-still-stops：界要够跑满两小时那一道（压到 1 秒）之后填 why
RUN_HARD_LIMIT_ACROSS_SECONDS = 6             # run-hard-limit-cap-hit-resets：四次撞上限各跑 1.5 秒、第五次约 1 秒：每一次都不到界的一半多，算上撞上限的那几次第四次就满（机器忙时单次放慢到界的 3 倍多才会误判）


def hard_limit_settings(seconds, **changed):
    """四小时硬上限那几格服务的设置：上限压到 seconds 秒。"""
    return dict({"TASK_SERVER_SELFTEST_RUN_HARD_LIMIT_SECONDS": str(seconds)}, **changed)


def hard_limit_problems(summary, what, attempts=1):
    """四小时硬上限打回的共同判据：记 over-four-hours、退 229、不重排（attempts 次尝试，最后那一次记同一个结局）、说明写硬上限。→ 问题表。"""
    task, problems = summary["task"], []
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    if task.get("state") != "finished" or task.get("outcome") != lib.RUN_HARD_LIMIT_OUTCOME or task.get("exit_code") != RUN_HARD_LIMIT_EXIT:
        problems.append(f"{what}：应当记 {lib.RUN_HARD_LIMIT_OUTCOME}、退 {RUN_HARD_LIMIT_EXIT}，实际状态 {task.get('state')}、{describe(summary)}")
    if len(outcomes) != attempts or (attempts and outcomes[-1] != lib.RUN_HARD_LIMIT_OUTCOME):
        problems.append(f"{what}：应当有 {attempts} 次尝试、最后那一次记 {lib.RUN_HARD_LIMIT_OUTCOME}，实际 {outcomes}")
    if RUN_HARD_LIMIT_NOTE not in str(task.get("note")):
        problems.append(f"{what}：说明应当写{RUN_HARD_LIMIT_NOTE}，实际「{task.get('note')}」")
    return problems


def held_until_finished(world, task_id, release):
    """等任务收尾，不论等到没有都放行它的命令。→ 那时的 summary。"""
    try:
        return world.finished(task_id, timeout=RELEASE_WAIT_SECONDS)
    finally:
        open(release, "w").close()


def cell_run_hard_limit_arithmetic(world):
    """四小时硬上限的算术（task_server_lib.py 的 over_run_hard_limit、covered_seconds）：常量是 14400 秒，恰好等于算到、差半秒不算；并集：同时在跑的不重复算、空档不算、套着的不多算。
    弄坏开关 run-hard-limit-exclusive、run-hard-limit-loose、run-hard-limit-parent-summed 各红。"""
    problems = []
    limit = float(SERVER.RUN_HARD_LIMIT_SECONDS)
    if limit != 14400:
        problems.append(f"上限应当是 14400 秒，实际 {limit}")
    for seconds, over in ((14400.0, True), (14400.5, True), (14399.5, False), (7200.0, False)):
        if lib.over_run_hard_limit(seconds, limit) != over:
            problems.append(f"{seconds} 秒：到上限应当 {over}，实际 {lib.over_run_hard_limit(seconds, limit)}")
    for intervals, seconds in (([], 0.0), ([(0.0, 60.0), (0.0, 60.0)], 60.0), ([(0.0, 10.0), (5.0, 20.0)], 20.0), ([(0.0, 10.0), (100.0, 110.0)], 20.0), ([(0.0, 100.0), (10.0, 20.0), (90.0, 200.0)], 200.0)):
        if abs(lib.covered_seconds(intervals) - seconds) > 0.001:
            problems.append(f"时段 {intervals} 的并集应当 {seconds} 秒，实际 {lib.covered_seconds(intervals)}")
    if problems:
        raise SelftestFailure("四小时硬上限的算术：" + "；".join(problems))
    return "上限 14400 秒，恰好等于算到；挂钟取时段的并集，同时在跑的不重复算、空档不算"


def hard_limit_fake_scheduler(records=()):
    """判四小时硬上限那两格的假调度器：上限 100 秒，尝试记录 1 的进度 1/1000，records 是 store.task 读得到的几条任务。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    table = {record["id"]: record for record in records}
    fake.store = types.SimpleNamespace(task=lambda task_id: (table[task_id], True) if task_id in table else None, attempts_of=lambda task_id: [])
    fake.run_hard_limit_seconds, fake.run_limit_seconds, fake.peer_root = 100.0, 60.0, "/"
    fake.attempt_progress = {1: (1, 1000)}
    fake.attempt_progress_baseline = {1: {"at": time.time() - 5, "done": 0, "offset": 0}}
    return fake


def cell_shard_hard_limit_by_each_piece(world):
    """分片的四小时硬上限按每一片自己的累计（task_server.py 的 parent_run_hard_limit、run_hard_limit_verdict；用户定）：假调度器上限 100 秒（同四小时，缩了比例）。
    三片依次排队、各跑 75 秒（照挂钟从头到尾 225 秒、每片都不到 100）：整件不判（parent_run_hard_limit 交 None），每一片照自己的累计都不停；
    一片单跑满 120 秒：那一片停，跑了 50 秒的兄弟片照跑；最先没成的是满界的那一片（first_failed_shard），整件收齐后照它收尾。
    弄坏开关 shard-whole-by-wall-clock 下整件按挂钟（225 秒到 100）判到，run-hard-limit-skipped、run-hard-limit-children-spared 下满界的那一片不停，这一格红。"""
    now = time.time()
    command = {"command": ["bash", "-c", "exit 0"], "cwd": "/"}
    pieces = [{"id": 10 + index, "parent_id": 9, "role": "shard", "shard_index": index, **command} for index in range(3)]

    def row(number, start, end):
        return ({"number": number, "state": "ended" if end is not None else "running", "started_at": now - start, "ended_at": None if end is None else now - end,
                 "outcome": "succeeded", "seconds": None if end is None else float(start - end)}, True)
    sequential = {10: [row(1, 230, 155)], 11: [row(1, 155, 80)], 12: [row(1, 75, None)]}
    problems = []
    fake = hard_limit_fake_scheduler([{"id": 9, **command}])
    fake.store = types.SimpleNamespace(task=lambda task_id: ({"id": 9, **command}, True) if task_id == 9 else None,
                                       attempts_of=lambda task_id: sequential.get(task_id, []))
    whole = SERVER.Scheduler.parent_run_hard_limit(fake, {"id": 9, **command}, pieces)
    if whole is not None:
        problems.append(f"三片依次排队各跑 75 秒：整件不应当判到上限，实际 {whole}")
    for piece in pieces:
        attempts = sequential[piece["id"]]
        running = next((record for record, _ in attempts if record["state"] == "running"), None)
        if running is not None and SERVER.Scheduler.run_hard_limit_verdict(fake, piece, dict(running, id=99)):
            problems.append(f"第 {piece['shard_index']} 片自己只跑了 75 秒：不应当停")
    fake.store = types.SimpleNamespace(task=lambda task_id: ({"id": 9, **command}, True) if task_id == 9 else None, attempts_of=lambda task_id: [])
    if not SERVER.Scheduler.run_hard_limit_verdict(fake, pieces[0], {"id": 98, "number": 1, "started_at": now - 120}):
        problems.append("一片单跑满 120 秒（上限 100）：那一片应当停")
    if SERVER.Scheduler.run_hard_limit_verdict(fake, pieces[1], {"id": 97, "number": 1, "started_at": now - 50}):
        problems.append("兄弟片跑了 50 秒：应当照跑、不停")
    shards = [dict(pieces[0], state="finished", outcome=lib.RUN_HARD_LIMIT_OUTCOME, finished_at=now), dict(pieces[1], state="running"), dict(pieces[2], state="queued")]
    failed = SERVER.Scheduler.first_failed_shard(fake, shards)
    if failed is None or failed.get("id") != pieces[0]["id"]:
        problems.append(f"最先没成的应当是满界停的第 0 片（整件收齐后照它的 {lib.RUN_HARD_LIMIT_OUTCOME} 收尾），实际 {failed}")
    if problems:
        raise SelftestFailure("分片的四小时硬上限按每一片自己的累计：" + "；".join(problems))
    return "三片依次各跑 75 秒的整件不判；一片单跑满界的停那一片、兄弟片照跑，整件照那一片收尾"


def cell_run_hard_limit_stopped(world):
    """满上限停（task_server.py 文件头 ⑦「四小时硬上限」；上限压到 2 秒）：命令一直等放行：服务停这一次、记 over-four-hours、退 229、不重排，推给提交它的会话，wait 退 229、末行 type=over-four-hours。
    弄坏开关 run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下结局不对，inbox-not-sent、inbox-auth-omitted 下收不到或收得不对，exit-code-dropped 下 wait 退 0，这一格红。"""
    inbox = FakeInbox(world, "a")
    release = os.path.join(world.root, "release-hard-limit-stopped")
    start_inbox_server(world, hard_limit_settings(2))
    task_id = world.submit(held_command(release, "tick"), extra=inbox.environment())
    summary = held_until_finished(world, task_id, release)
    problems = hard_limit_problems(summary, "满上限的")
    head = outcome_head(task_id, lib.RUN_HARD_LIMIT_OUTCOME)
    messages = inbox.wait_for(world, head)
    problems += inbox_problems(messages, head, "结局那一条")
    if messages and not all(word in messages[0]["text"] for word in (RUN_HARD_LIMIT_NOTE, RUN_HARD_LIMIT_ADVICE)):
        problems.append(f"推送正文应当写{RUN_HARD_LIMIT_NOTE}与出路「{RUN_HARD_LIMIT_ADVICE}」，实际「{messages[0]['text'][:400]}」")
    code, last = wait_last_line(world, task_id)
    if code != RUN_HARD_LIMIT_EXIT or f"type={lib.RUN_HARD_LIMIT_OUTCOME}" not in last:
        problems.append(f"wait 应当退 {RUN_HARD_LIMIT_EXIT}、末行 type={lib.RUN_HARD_LIMIT_OUTCOME}，实际退 {code}、末行「{last}」")
    if problems:
        raise SelftestFailure("满上限停：" + "；".join(problems), world.server_log_lines())
    return "累计满上限：停这一次、记 over-four-hours、不重排，推给会话，wait 退 229"


def cell_run_hard_limit_why_still_stops(world):
    """带 --why 照停：跑满两小时那一道压到 1 秒、硬上限压到 10 秒：跑过 1 秒填 why 退 0、记长跑已确认，到了硬上限照样停、记 over-four-hours。
    弄坏开关 run-hard-limit-confirmed-spared、run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下结局不对，这一格红。"""
    release = os.path.join(world.root, "release-hard-limit-why")
    world.start_server(extra=hard_limit_settings(RUN_HARD_LIMIT_CONFIRM_SECONDS, TASK_SERVER_SELFTEST_RUN_LIMIT_SECONDS=RUN_LIMIT_SHORT_SECONDS))
    task_id = world.submit(held_command(release, "tick"))
    world.running(task_id)
    time.sleep(float(RUN_LIMIT_SHORT_SECONDS) + 0.5)
    confirmed = world.client(["confirm", str(task_id), "--why", LONG_RUN_WHY_SAMPLE])
    problems = [] if confirmed.returncode == 0 else [f"跑满界填 why 应当退 0，实际退 {confirmed.returncode}：「{(confirmed.stdout + confirmed.stderr).strip()[-300:]}」"]
    summary = held_until_finished(world, task_id, release)
    problems += hard_limit_problems(summary, "填过 why 的")
    if not summary["task"].get("long_run_confirmed"):
        problems.append(f"应当记着长跑已确认，实际 {summary['task'].get('long_run_confirmed')}")
    if problems:
        raise SelftestFailure("带 --why 照停：" + "；".join(problems), world.server_log_lines())
    return "填了 why 的到了硬上限照停"


def cell_run_hard_limit_full_gate_not_stopped(world):
    """带 --full-gate 的不判：假仓里的提交时门禁跑 3 秒、上限压到 1 秒：跑成。弄坏开关 run-hard-limit-full-gate-judged 下停了，这一格红。"""
    path = os.path.join(world.repo, FAKE_COMMIT_GATE)
    os.makedirs(os.path.dirname(path), exist_ok=True)
    write_executable(path, "#!/usr/bin/env bash\nsleep 3\nexit 0\n")
    world.resources(local=(30, 30))
    world.start_server(extra=hard_limit_settings(1))
    task_id = world.submit(["bash", FAKE_COMMIT_GATE], extra={"SINGLEFS_RUN_SCOPE": SCOPE_COMMIT}, cwd=world.repo)
    summary = world.finished(task_id)
    if summary["task"].get("outcome") != "succeeded" or not summary["task"].get("full_gate") or len(summary["attempts"]) != 1:
        raise SelftestFailure(f"带 --full-gate 的过了上限不判、跑成、只有一次尝试，实际 {describe(summary)}、全量标识 {summary['task'].get('full_gate')}", world.server_log_lines())
    return "带 --full-gate 的跑 3 秒过了 1 秒的上限：不停、跑成"


def cell_run_hard_limit_cap_hit_resets(world):
    """撞上限归零后重新计（task_server.py 的 run_limit_accumulated）：上限 6 秒，四次撞上限各跑 1.5 秒、第五次跑约 1 秒，五次加起来过了上限，每一次都不到上限的一半多：跑成，五次结局是四次 memory-cap-hit、一次 succeeded。
    单次用时只占上限的四分之一，机器忙时慢到三倍多才会误判。弄坏开关 long-run-cap-hits-counted 下第四次就满上限、停了，retry-overwrites-attempts 下尝试的记录没了，这一格红。"""
    world.resources(local=(100, 100))
    world.start_server(extra=hard_limit_settings(RUN_HARD_LIMIT_ACROSS_SECONDS))
    task_id = world.submit(["bash", "-c", ACROSS_CAP_HITS_SCRIPT])
    summary = world.finished(task_id)
    outcomes = [attempt.get("outcome") for attempt in summary["attempts"]]
    if summary["task"].get("outcome") != "succeeded" or outcomes != ["memory-cap-hit"] * 4 + ["succeeded"]:
        raise SelftestFailure(f"撞上限之后归零：应当跑成，五次结局四个 memory-cap-hit、一个 succeeded，实际 {describe(summary)}、各次 {outcomes}", world.server_log_lines())
    return f"四次撞上限各跑 1.5 秒、第五次跑约 1 秒：每次撞上限后归零，不满 {RUN_HARD_LIMIT_ACROSS_SECONDS} 秒，跑成"


def cell_run_hard_limit_writer_stopped(world):
    """改主工作区的也停（task_server.py 的 workspace_write_item）：hook-swap.py 跑满 2 秒的上限：停、记 over-four-hours；wait 退 229、在末行之前打一行 kind=workspace-write-interrupted。
    弄坏开关 run-hard-limit-writers-spared、run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下结局不对，run-hard-limit-writer-unreminded、workspace-write-interrupted-unreported 下不报，这一格红。"""
    release = os.path.join(world.root, "release-hard-limit-writer")
    world.start_server(extra=hard_limit_settings(2))
    task_id = world.submit(held_command(release, "tick", "scripts/gate/hook-swap.py"), cwd=world.repo)
    summary = held_until_finished(world, task_id, release)
    problems = hard_limit_problems(summary, "改主工作区的")
    completed = world.client(["wait", str(task_id)])
    lines = attention_lines(completed)
    if completed.returncode != RUN_HARD_LIMIT_EXIT or len(lines) != 1 or "kind=workspace-write-interrupted" not in lines[0]:
        problems.append(f"wait 应当退 {RUN_HARD_LIMIT_EXIT}、打一行 kind=workspace-write-interrupted，实际退 {completed.returncode}、提醒 {lines}")
    if problems:
        raise SelftestFailure("改主工作区的也停：" + "；".join(problems), world.server_log_lines())
    return "改主工作区的满上限照停，wait 照 workspace-write-interrupted 提醒核现场"


def cell_run_hard_limit_shard_stopped(world):
    """分片的（task_server_lib.py 文件头「分片」、task_server.py 文件头 ⑦「四小时硬上限」：按每一片自己的累计算）：上限压到 15 秒，第 0、1 片一直在跑、第 2 片排着
    （只有第二台放得下它、第二台用不了）。第 0、1 片各自累计满界：各记 over-four-hours；排着的第 2 片不撤，两片停了之后本机换成 50G 的余量，它照起、跑完（succeeded）；
    收齐之后整件照没成的那一片收尾：over-four-hours、退 229，不合并，那一片的说明写硬上限。
    弄坏开关 run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下各片结局不对，shard-whole-by-wall-clock 下整件按挂钟满界、排着的第 2 片被撤（cancelled），这一格红。"""
    release = os.path.join(world.root, "release-hard-limit-shard")
    task_id, children = start_three_shards(world, "hard-limit", [{"block": release}, {"block": release}, {}], extra=hard_limit_settings(RUN_HARD_LIMIT_SHARD_SECONDS))
    problems = []
    try:
        stopped = world.wait_for(task_id, lambda found: all(shard_outcomes(found).get(index) for index in (0, 1)) or found["task"].get("state") == "finished",
                                 "第 0、1 片各自满界收尾", timeout=RELEASE_WAIT_SECONDS)
        if stopped["task"].get("state") != "finished":
            world.resources(local=(50, 50), peer_down="自证：第二台此刻用不了")   # 第 2 片（记过 30G）在本机放得下了
    finally:
        open(release, "w").close()
    summary = world.finished(task_id, timeout=RELEASE_WAIT_SECONDS)
    outcomes = shard_outcomes(summary)
    failed_note = str(shard_children(summary).get(0, {}).get("task", {}).get("note") or "") + str(shard_children(summary).get(1, {}).get("task", {}).get("note") or "")
    if (summary["task"].get("outcome") != lib.RUN_HARD_LIMIT_OUTCOME or summary["task"].get("exit_code") != RUN_HARD_LIMIT_EXIT
            or lib.RUN_HARD_LIMIT_OUTCOME not in str(summary["task"].get("note")) or RUN_HARD_LIMIT_NOTE not in failed_note):
        problems.append(f"整件应当照没成的那一片记 {lib.RUN_HARD_LIMIT_OUTCOME}、退 {RUN_HARD_LIMIT_EXIT}、说明点名那一片的结局，那一片的说明写{RUN_HARD_LIMIT_NOTE}；"
                        f"实际 {describe(summary)}、说明「{summary['task'].get('note')}」、第 0、1 片的说明「{failed_note[:300]}」")
    if outcomes != {0: lib.RUN_HARD_LIMIT_OUTCOME, 1: lib.RUN_HARD_LIMIT_OUTCOME, 2: "succeeded"}:
        problems.append(f"各片按自己的累计判：在跑满界的两片应当记 {lib.RUN_HARD_LIMIT_OUTCOME}、排着的第 2 片照起、跑完（succeeded），实际 {outcomes}")
    if not world.sharded_calls("shard", index=2) or "merge" in child_roles(summary) or world.sharded_calls("merge"):
        problems.append(f"第 2 片应当起过、不合并，实际第 2 片跑的记录 {world.sharded_calls('shard', index=2)}、子任务 {child_roles(summary)}")
    if problems:
        raise SelftestFailure("分片的到上限：" + "；".join(problems), world.server_log_lines())
    return "分片的满上限按每一片自己的累计：满界的两片停、记 over-four-hours，排着的那一片照起、跑完，整件照没成的那一片收尾、不合并"


def cell_run_hard_limit_applies_after_restart(world):
    """服务重启时已在跑的任务同样判：服务起来时上限够大，交一条一直等放行的；停了服务（尝试在它自己的单元里接着跑），换 1 秒的上限再起：判到、停、记 over-four-hours。
    弄坏开关 run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下结局不对，no-detach 下服务一停命令也没了，这一格红。"""
    release = os.path.join(world.root, "release-hard-limit-restart")
    world.start_server(extra=hard_limit_settings(HARD_LIMIT_OUT_OF_REACH_SECONDS))
    task_id = world.submit(blocking_command(release))
    world.running(task_id)
    world.stop_server()
    world.start_server(extra=hard_limit_settings(1))
    problems = hard_limit_problems(held_until_finished(world, task_id, release), "重启时已在跑的")
    if problems:
        raise SelftestFailure("重启时已在跑的同样判：" + "；".join(problems), world.server_log_lines())
    return "服务重启时已在跑、累计过了上限的：照判、照停"


def cell_run_hard_limit_stop_retried(world):
    """判了停而头一回没停成的下一轮再停（task_server.py 的 stop_over_limit_attempt；TASK_SERVER_SELFTEST_STOP_FAILS 演一次）：照样记 over-four-hours，演停失败的文件用掉了。
    弄坏开关 hard-limit-stop-once 下不再停，run-hard-limit-skipped、cancel-no-readback 下不停，run-hard-limit-outcome-unmarked 下结局不对，这一格红。"""
    release = os.path.join(world.root, "release-hard-limit-stop-retried")
    stop_fails = os.path.join(world.root, "stop-fails-hard-limit")
    open(stop_fails, "w").close()
    world.start_server(extra=hard_limit_settings(2, TASK_SERVER_SELFTEST_STOP_FAILS=stop_fails))
    task_id = world.submit(held_command(release, "tick"))
    problems = hard_limit_problems(held_until_finished(world, task_id, release), "头一回没停成的")
    if os.path.exists(stop_fails):
        problems.append("演停失败的文件应当用掉了：服务应当起过一回 stop_running")
    if problems:
        raise SelftestFailure("没停成的下一轮再停：" + "；".join(problems), world.server_log_lines())
    return "头一回没停成，下一轮再停：照样记 over-four-hours"


def submitted_sheet(world, command, cwd=None):
    """交一条、不确认：→ (确认单那几行的文字, 任务号)；交不成判红。"""
    completed = world.client(["submit", "--cwd", cwd or world.work, "--", *command])
    found = re.search(rf"{lib.SUBMIT_MARK} task=(\d+)", completed.stdout)
    if completed.returncode != 0 or not found:
        raise SelftestFailure(f"交 {command} 应当退 0、打任务号，实际退 {completed.returncode}", [completed.stdout, completed.stderr] + world.server_log_lines())
    return completed.stdout, int(found.group(1))


def cell_confirmation_scope_text(world):
    """确认单的范围（task_server.py 文件头 ⑦）：命令里没点、脚本里面才展开的写「范围由脚本里面定，服务算不出」，不写「整包、不指定范围」；合入后验证另打一行崩溃用例与域；别的 cargo 照写整包。
    确认单上 --crash-expand-up-to 的值与点名的 crash-case 项数照打。弄坏开关 scope-inside-script-unnamed 下照写整包，crash-scope-unreported 下不打崩溃用例与域，confirm-skipped 下没有确认单，这一格红。"""
    problems = []
    for named, whole, expected in (([], True, lib.WHOLE_SCOPE_TEXT), ([], False, lib.SCOPE_INSIDE_SCRIPT_TEXT), ([("--item", "alpha")], False, "--item 1")):
        if lib.named_scope_text(named, whole) != expected:
            problems.append(f"named_scope_text({named}, {whole}) 应当是「{expected}」，实际「{lib.named_scope_text(named, whole)}」")
    merge = ["bash", FAKE_MERGE_ENTRY, "--skip-apply"]
    for command, words in ((["bash", "-c", "exit 0"], None), ([*merge, "patch"], [lib.SCOPE_INSIDE_SCRIPT_TEXT]),
                           ([*merge, "--crash-expand-up-to", "6", "patch"], ["--crash-expand-up-to 6", lib.SCOPE_INSIDE_SCRIPT_TEXT]),
                           ([*merge, "--item", "crash-case:alpha", "--item=crash-case:beta", "--item", "unit:gamma", "patch"], ["2 条", "由脚本定"])):
        text = lib.crash_scope_text(command, world.repo)
        if (words is None and text) or (words is not None and not all(word in text for word in words)):
            problems.append(f"crash_scope_text({command}) 应当{'是空串' if words is None else '写 ' + '、'.join(words)}，实际「{text}」")
    write_fake_entry(world, FAKE_MERGE_ENTRY)
    patch = os.path.join(world.root, "patch-scope")
    os.makedirs(patch)
    world.start_server()
    for what, command, cwd, expected, unexpected in (("脚本自己定范围的", ["bash", "-c", "exit 0", "scope-sample"], None, [lib.SCOPE_INSIDE_SCRIPT_TEXT], [lib.WHOLE_SCOPE_TEXT, "崩溃用例与域"]),
                                                      ("合入后验证", [*merge, patch], world.repo, [f"项数 0：{lib.SCOPE_INSIDE_SCRIPT_TEXT}", f"崩溃用例与域：{lib.SCOPE_INSIDE_SCRIPT_TEXT}"], [lib.WHOLE_SCOPE_TEXT]),
                                                      ("别的 cargo", ["cargo", "build", "-p", "selftest-scope"], None, [lib.WHOLE_SCOPE_TEXT], [lib.SCOPE_INSIDE_SCRIPT_TEXT])):
        sheet, task_id = submitted_sheet(world, command, cwd)
        world.client(["cancel", str(task_id)])
        if not all(word in sheet for word in expected) or any(word in sheet for word in unexpected):
            problems.append(f"{what}的确认单应当写 {expected}、不写 {unexpected}，实际「{sheet.strip()[:600]}」")
    if problems:
        raise SelftestFailure("确认单的范围：" + "；".join(problems), world.server_log_lines())
    return "脚本里面才展开的写算不出、不写整包；合入后验证打崩溃用例与域，命令里写了的照打"


def raise_cell_problems(world, problems, what):
    if problems:
        raise SelftestFailure(f"{what}：" + "；".join(problems), world.server_log_lines())


def release_all(paths):
    for path in paths:
        open(path, "w").close()


OUTER_UNIT_LAUNCHER = ("import json, os, sys\n"
                       "environment = json.load(open(sys.argv[1], encoding='utf-8'))\n"
                       "log = os.open(sys.argv[2], os.O_WRONLY | os.O_CREAT | os.O_APPEND, 0o600)\n"
                       "os.dup2(log, 1)\nos.dup2(log, 2)\n"
                       "os.execve(sys.executable, [sys.executable, '-B', sys.argv[3], 'serve'], environment)\n")   # 外层单元里起这一格的服务：照这一格服务的环境、输出进这一格的服务日志
OUTER_UNIT_STOP_SECONDS = 20   # selftest-units-stop-with-outer：停了外层单元之后，等尝试单元停下至多这么久（带 BindsTo 的当场停）
STOPPED_UNIT_STATES = ("inactive", "failed", "")   # systemctl --user show -p ActiveState 打这几样算停了（单元收掉了的打 inactive）
ENCLOSING_UNIT_SAMPLES = (("0::/user.slice/user-1000.slice/user@1000.service/app.slice/outer-sample.service\n", "outer-sample.service"),
                          ("0::/user.slice/user-1000.slice/user@1000.service/singlefs.slice/memory-cap-sample.scope/payload\n", "memory-cap-sample.scope"),
                          ("0::/user.slice/user-1000.slice/session-3.scope\n", ""), ("0::/system.slice/sample.service\n", ""))


def unit_active_state(unit):
    completed = subprocess.run(["systemctl", "--user", "show", "-p", "ActiveState", "--value", unit], capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=30)
    return completed.stdout.strip()


def cell_selftest_units_stop_with_outer(world):
    """自证里起的尝试单元随外层停（task_server.py 文件头 ③「套在外层单元里的服务」）：这一格的服务起在一个这一格自己起的外层单元里（systemd-run --user，单元名带这一格的进程号），
    交一条一直等放行文件的命令：它的尝试单元 BindsTo 那个外层单元；停外层单元之后 OUTER_UNIT_STOP_SECONDS 秒里尝试单元停了（不等命令自己退）。
    另核 lib.enclosing_user_unit 认几种 cgroup 写法。弄坏开关 nested-units-unbound 下不带 BindsTo、外层停了尝试单元照跑，这一格红。"""
    problems = [f"cgroup「{text.strip()}」应当认成「{unit}」，实际「{lib.enclosing_user_unit(text)}」" for text, unit in ENCLOSING_UNIT_SAMPLES
                if lib.enclosing_user_unit(text) != unit]
    outer = f"singlefs-task-selftest-outer-{os.getpid()}-{time.time_ns()}.service"
    environment_path = os.path.join(world.root, "outer-environment.json")
    with open(environment_path, "w", encoding="utf-8") as handle:
        json.dump(world.server_environment, handle)
    world.server_logs += 1
    log_path = os.path.join(world.root, f"server-{world.server_logs}.log")
    release = os.path.join(world.root, "release-outer")
    write_executable(os.path.join(world.work, "outer-hold.sh"), f"#!/usr/bin/env bash\n{HOLD_UNTIL_RELEASED_SCRIPT}\n")
    with open("/proc/self/cgroup", encoding="utf-8") as handle:
        own_unit = lib.enclosing_user_unit(handle.read())   # 外层单元自己也随这一格所在的单元停：整份自证被停时它不留下
    bound = ["-p", f"BindsTo={own_unit}", "-p", f"After={own_unit}"] if own_unit else []
    started = subprocess.run(["systemd-run", "--user", f"--unit={outer}", "--collect", "--quiet", "-p", "KillMode=mixed", *bound, f"--working-directory={world.root}", "--",
                              sys.executable, "-B", "-c", OUTER_UNIT_LAUNCHER, environment_path, log_path, lib.SERVER_SCRIPT],
                             capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60)
    if started.returncode != 0:
        raise SelftestFailure(f"外层单元起不来（systemd-run 退 {started.returncode}）：{(started.stderr + started.stdout).strip()[-300:]}")
    unit = ""
    try:
        patience = world.patience(STATE_TIMEOUT_SECONDS)
        while patience.waiting():
            with contextlib.suppress(lib.ServerUnreachable):
                status, health = world.api("GET", "/health")
                if status == 200 and health.get("ok"):
                    break
            time.sleep(POLL_SECONDS)
        else:
            raise SelftestFailure(f"外层单元里的服务 {patience.elapsed():.0f} 秒没起来", world.server_log_lines())
        task_id = world.submit(["bash", "outer-hold.sh", release])
        unit = next((attempt.get("unit") for attempt in world.running(task_id)["attempts"] if attempt.get("state") == "running"), "") or ""
        binds = subprocess.run(["systemctl", "--user", "show", "-p", "BindsTo", "--value", unit], capture_output=True, text=True, stdin=subprocess.DEVNULL,
                               timeout=30).stdout.split()
        if outer not in binds:
            problems.append(f"尝试单元 {unit} 应当 BindsTo 外层单元 {outer}，实际 BindsTo「{' '.join(binds)}」")
        subprocess.run(["systemctl", "--user", "stop", outer], capture_output=True, stdin=subprocess.DEVNULL, timeout=120)
        deadline = time.monotonic() + OUTER_UNIT_STOP_SECONDS
        while unit_active_state(unit) not in STOPPED_UNIT_STATES and time.monotonic() < deadline:
            time.sleep(POLL_SECONDS)
        if unit_active_state(unit) not in STOPPED_UNIT_STATES:
            problems.append(f"外层单元停了 {OUTER_UNIT_STOP_SECONDS} 秒，尝试单元 {unit} 还是 {unit_active_state(unit)}（只能等命令自己超时退）")
    finally:
        release_all([release])
        subprocess.run(["systemctl", "--user", "stop", outer], capture_output=True, stdin=subprocess.DEVNULL, timeout=120)
        if unit and unit_active_state(unit) not in STOPPED_UNIT_STATES:
            subprocess.run(["systemctl", "--user", "stop", unit], capture_output=True, stdin=subprocess.DEVNULL, timeout=200)
    raise_cell_problems(world, problems, "自证里起的单元随外层停")
    return f"外层单元里的服务起的尝试单元 BindsTo 外层单元，停外层之后 {OUTER_UNIT_STOP_SECONDS} 秒里跟着停了"


def loaded_host_dispatch():
    """假调度器要的 host-dispatch.py（Prediction、Verdict）：这一格的进程里载一份。"""
    return lib.load_module("host_dispatch_for_scheduling_cells", os.path.join(lib.HOSTS_DIRECTORY, "host-dispatch.py"))


RUNNING_BOOKING_LABEL = "task-selftest-running-booking-1"


def running_booking_scheduler(peak_bytes, scope_used_bytes, slice_used_gibibytes=0, cap_bytes=20 * GIBIBYTE):
    """running-booked-at-measured-usage 与 running-booked-unscoped-at-cap 的假调度器与一份本机报告：slice 总上限 30G、余量 1G，一条在跑、上限 cap_bytes，
    它这一次到此刻的峰值 peak_bytes（Machine.memory_now；在跑记账不读它），报告里它的 scope 此刻回收不掉 scope_used_bytes（None 是报告里没有它的 scope）。
    → (假调度器, Snapshot, 那份报告)。"""
    attempt = {"id": 1, "task_id": 1, "host": SERVER.LOCAL, "label": RUNNING_BOOKING_LABEL, "memory_cap": cap_bytes}
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.peaks = PEAKS
    fake.host_dispatch = loaded_host_dispatch()
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(memory_now=lambda found: (scope_used_bytes, peak_bytes))}
    fake.store = types.SimpleNamespace(running_attempts=lambda: [dict(attempt)])
    fake.host_margin = lambda host, snapshot: (GIBIBYTE, "")
    fake.slice_room_floor = SERVER.SLICE_ROOM_FLOOR_BYTES
    fake.unit_stopped = set()   # booking_attempts 不数的、单元已确认停了的尝试号（Scheduler.__init__ 建的）
    report = memory_report(30, 30, slice_used_gibibytes=slice_used_gibibytes,
                           scopes=None if scope_used_bytes is None else {RUNNING_BOOKING_LABEL: scope_used_bytes})
    return fake, SERVER.Snapshot({SERVER.LOCAL: report}, {}, {SERVER.LOCAL: True}, {}), report


# running-booked-at-measured-usage：(说什么, 上限, 此刻 scope 用量, 该记多少, 取的是哪一个, 公式剩多少, 待涨量)，单位 GiB；slice 30G、余量 1G，公式的总额就是总上限 30G
RUNNING_BOOKING_CASES = (("上限 20G、此刻 2G", 20, 2, 2, SERVER.BOOKED_FROM_USAGE, 27, 0),
                         ("上限 20G、此刻 12G", 20, 12, 12, SERVER.BOOKED_FROM_USAGE, 17, 0),
                         ("上限 8G、此刻 9G", 8, 9, 8, SERVER.BOOKED_FROM_CAP, 21, 0))


def cell_running_booked_at_measured_usage(world):
    """在跑的有 scope 的尝试只照此刻实测记账、不超过上限、不加固定余量、不看任何预先估的数（task_server.py 的 running_booking、room_by_the_formula、server_pending；
    文件头 ②「内存」的在跑记账）：本机 slice 30G、余量 1G，一条在跑（RUNNING_BOOKING_CASES 三种）——上限 20G、此刻 2G 记 2G（measured）、公式剩 27G、排头 12G 放得下、待涨量 0；
    此刻 12G 记 12G（measured）、公式剩 17G；上限 8G、此刻 9G 记上限 8G（cap）、公式剩 21G。
    弄坏开关 running-booked-at-cap 下头一种记 20G，running-booked-with-margin 下记 3G，这一格红。"""
    dispatch = loaded_host_dispatch()
    head = types.SimpleNamespace(memory_booked=12 * GIBIBYTE)
    fits = dispatch.Verdict(SERVER.LOCAL, True, False, 12 * GIBIBYTE, "内存放得下")
    problems, shown = [], []
    for what, cap, used, want, basis, want_room, want_pending in RUNNING_BOOKING_CASES:
        fake, snapshot, report = running_booking_scheduler(used * GIBIBYTE, used * GIBIBYTE, cap_bytes=cap * GIBIBYTE)
        booked, got_basis = SERVER.Scheduler.running_booking(fake, fake.store.running_attempts()[0], SERVER.Scheduler.report_scope_usage(fake, report))
        room = SERVER.Scheduler.room_by_the_formula(fake, SERVER.LOCAL, snapshot)
        verdict = SERVER.Scheduler.verdict_within_the_formula(fake, fits, head, SERVER.LOCAL, snapshot)
        pending = SERVER.Scheduler.server_pending(fake, SERVER.LOCAL, report)
        if (booked, got_basis, room, pending) != (want * GIBIBYTE, basis, want_room * GIBIBYTE, want_pending * GIBIBYTE) or not verdict.fits:
            problems.append(f"{what}：应当记 {want}G（{basis}）、公式剩 {want_room}G、排头 12G 放得下、待涨量 {want_pending}G，"
                            f"实际记 {booked / GIBIBYTE:g}G（{got_basis}）、公式剩 {room / GIBIBYTE:g}G、判决 {verdict}、待涨量 {pending / GIBIBYTE:g}G")
        shown.append(f"{what}记 {want}G（{basis}）")
    if problems:
        raise SelftestFailure("在跑的只照此刻实测记账：" + "；".join(problems))
    return "；".join(shown) + "；排头 12G 都放得下"


def cell_running_young_booked_at_cap(world):
    """起跑不满两分钟的在跑尝试照整份上限记（task_server.py 的 running_booking、running_booking_text；任务 15262 到 15299 共 38 条十秒内全起那一回）：
    时限照 120 秒，上限 20G、此刻 2G 的一条——起跑 10 秒的记 20G（young），为什么那一句写「不算漂移」；起跑 200 秒的记实测 2G（measured）。套在假调度器上，不起服务。
    弄坏开关 running-young-booked-by-usage 下起跑 10 秒的记 2G，这一格红。"""
    problems = []
    for age, want, basis in ((10, 20, SERVER.BOOKED_WHILE_YOUNG), (200, 2, SERVER.BOOKED_FROM_USAGE)):
        fake, _, report = running_booking_scheduler(2 * GIBIBYTE, 2 * GIBIBYTE)
        fake.environment = dict(fake.environment, **{lib.YOUNG_BOOKING_VARIABLE: "120"})
        attempt = dict(fake.store.running_attempts()[0], started_at=time.time() - age)
        usage = SERVER.Scheduler.report_scope_usage(fake, report)
        booked, got = SERVER.Scheduler.running_booking(fake, attempt, usage)
        why = SERVER.Scheduler.running_booking_text(got, usage.get(RUNNING_BOOKING_LABEL), 20 * GIBIBYTE)
        if (booked, got) != (want * GIBIBYTE, basis) or (basis == SERVER.BOOKED_WHILE_YOUNG and "不算漂移" not in why):
            problems.append(f"起跑 {age} 秒的应当记 {want}G（{basis}），刚起的那一句写「不算漂移」，实际记 {booked / GIBIBYTE:g}G（{got}）、「{why}」")
    if problems:
        raise SelftestFailure("起跑不满两分钟照整份上限记：" + "；".join(problems))
    return "上限 20G、此刻 2G：起跑 10 秒的记 20G（young，不算漂移）；起跑 200 秒的记实测 2G"


def cell_running_booked_unscoped_at_cap(world):
    """报告里没有在跑尝试的 scope（刚起、读不了）的照整份上限记账（task_server.py 的 running_booked、room_by_the_formula、verdict_within_the_formula）：本机 slice 30G、余量 1G，
    一条在跑、上限 20G、报告里没有它的 scope：记 20G，公式剩 9G，排头 25G 放不下、原因写「在跑的记账之和」。slice 兜底（slice_floor_problem）：slice 实测只剩 3G
    （不到 SLICE_ROOM_FLOOR_BYTES）这一轮不派、原因以「slice 兜底」起头，剩 20G 的不挡。
    弄坏开关 slice-floor-ignored 下不判 slice 兜底，这一格红。"""
    dispatch = loaded_host_dispatch()
    head = types.SimpleNamespace(memory_booked=25 * GIBIBYTE, memory_predicted=25 * GIBIBYTE, memory_cap=25 * GIBIBYTE)
    fits = dispatch.Verdict(SERVER.LOCAL, True, False, 25 * GIBIBYTE, "内存放得下")
    problems = []
    fake, snapshot, _ = running_booking_scheduler(GIBIBYTE // 2, None)
    room = SERVER.Scheduler.room_by_the_formula(fake, SERVER.LOCAL, snapshot)
    verdict = SERVER.Scheduler.verdict_within_the_formula(fake, fits, head, SERVER.LOCAL, snapshot)
    if room != 9 * GIBIBYTE or verdict.fits or SERVER.RUNNING_BOOKED_SUM_TEXT not in verdict.why:
        problems.append(f"报告里没有它的 scope：应当照上限 20G 记、公式剩 9G、排头 25G 放不下、原因写「{SERVER.RUNNING_BOOKED_SUM_TEXT}」，实际公式剩 {room}、判决 {verdict}")
    for used, held in ((27, True), (10, False)):
        fake, snapshot, _ = running_booking_scheduler(GIBIBYTE // 2, int(0.4 * GIBIBYTE), slice_used_gibibytes=used)
        why = SERVER.Scheduler.slice_floor_problem(fake, SERVER.LOCAL, snapshot)
        if held != why.startswith(SERVER.SLICE_FLOOR_TEXT):
            problems.append(f"slice 里进程用 {used}G（总上限 30G）：应当{'停派、原因以「' + SERVER.SLICE_FLOOR_TEXT + '」起头' if held else '不挡'}，实际「{why}」")
    if problems:
        raise SelftestFailure("没有 scope 的照上限记账、slice 兜底：" + "；".join(problems))
    return "读不到 scope 的照上限 20G 记、公式剩 9G、排头 25G 放不下；slice 只剩 3G 时停派"


ADMISSION_PROBE = ("selftest-admission-probe", 10)   # admission-by-measured-bookings 与 memory-wait-names-both-numbers 再来的那一条：(命令键, 上限与记账 GiB)
# 那两格的三种在跑：(说什么, [(上限, scope 此刻用量)]（GiB）, slice 里进程用的 GiB)；slice 总额 50G、余量 1G、整机空闲 100G
ADMISSION_ROOMY = ("上限之和 48G、实测只占 6G", [(12, 1.5)] * 4, 6)
ADMISSION_CAPS_OVER_TOTAL = ("上限之和 70G（比总额 50G 大）、实测只占 6G", [(14, 1.2)] * 5, 6)
ADMISSION_MEASURED_TIGHT = ("上限之和 48G、实测占 42G（slice 实测余量 8G）", [(12, 10.5)] * 4, 42)


def admission_scheduler(running, slice_used_gibibytes):
    """照实测记账准入那两格的假调度器（真的 Scheduler 方法，套在假的库与余量报告上）：本机 slice 总额 50G、余量 1G、整机空闲 100G，在跑的 running 是 [(上限, scope 此刻用量)]（GiB），
    任务号 701 起依次编，slice 里进程用 slice_used_gibibytes。→ (假调度器, Snapshot, 报告)。"""
    attempts = [{"id": number, "task_id": 700 + number, "host": SERVER.LOCAL, "label": f"task-selftest-admission-{number}", "memory_cap": int(cap * GIBIBYTE)}
                for number, (cap, _) in enumerate(running, 1)]
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.peaks = PEAKS
    fake.host_dispatch = loaded_host_dispatch()
    fake.store = types.SimpleNamespace(running_attempts=lambda: [dict(attempt) for attempt in attempts])
    fake.host_margin = lambda host, snapshot: (GIBIBYTE, "")
    fake.slice_room_floor = SERVER.SLICE_ROOM_FLOOR_BYTES
    fake.unit_stopped = set()
    report = memory_report(100, 50, slice_used_gibibytes=slice_used_gibibytes,
                           scopes={attempt["label"]: int(used * GIBIBYTE) for attempt, (_, used) in zip(attempts, running)})
    return fake, SERVER.Snapshot({SERVER.LOCAL: report}, {}, {SERVER.LOCAL: True}, {}), report


def admission_judged(case):
    """照 case（ADMISSION_* 之一）建假调度器，判再来的那一条（ADMISSION_PROBE）：judged_on_host（judge_host、公式）再过 slice_floor_verdict，同 host_admission 的内存那几步。
    → (假调度器, Snapshot, 这台上的记账, 判决)。"""
    _, running, slice_used = case
    fake, snapshot, report = admission_scheduler(running, slice_used)
    key, size = ADMISSION_PROBE
    prediction = fake.host_dispatch.Prediction(key, size * GIBIBYTE, size * GIBIBYTE, size * GIBIBYTE, lib.RECORD_HISTORY)
    pending = SERVER.Scheduler.server_pending(fake, SERVER.LOCAL, report)
    on_host, verdict = SERVER.Scheduler.judged_on_host(fake, {"id": 1, "command_key": key}, prediction, SERVER.LOCAL, snapshot, pending)
    return fake, snapshot, on_host, SERVER.Scheduler.slice_floor_verdict(fake, verdict, on_host, SERVER.LOCAL, snapshot)


def cell_admission_by_measured_bookings(world):
    """准入只看在跑的实测记账之和，在跑的上限之和多大都不挡（task_server.py 的 running_booking、room_by_the_formula、judged_on_host；文件头 ②「内存」的公式；
    2026-11-12 两台各记 73G、74G 而实占 2.5G、4.6G，排着的几十条起不来）：本机 slice 总额 50G、余量 1G，再来一条上限 10G 的——
    在跑上限之和 48G、实测只占 6G：起得来；上限之和 70G（比总额大）、实测只占 6G：照样起得来（50 − 1 − 6 ≥ 10）；上限之和 48G 而实测占 42G：排着，
    原因写「账面还能记 7.0 GiB」。套在假调度器上，不起服务。
    弄坏开关 running-booked-at-cap 下第二种照上限之和 70G 记、放不下，这一格红。"""
    problems, shown = [], []
    for case, fits in ((ADMISSION_ROOMY, True), (ADMISSION_CAPS_OVER_TOTAL, True), (ADMISSION_MEASURED_TIGHT, False)):
        _, _, _, verdict = admission_judged(case)
        if verdict.fits != fits:
            problems.append(f"{case[0]}：上限 10G 的应当{'起得来' if fits else '排着'}，实际判决 {verdict}")
        shown.append(f"{case[0]}：{'起得来' if verdict.fits else '排着'}")
    _, _, _, tight = admission_judged(ADMISSION_MEASURED_TIGHT)
    if "账面还能记 7.0 GiB" not in tight.why:
        problems.append(f"{ADMISSION_MEASURED_TIGHT[0]}：原因应当写「账面还能记 7.0 GiB」，实际「{tight.why}」")
    if problems:
        raise SelftestFailure("准入只看实测记账：" + "；".join(problems))
    return "；".join(shown)


def cell_memory_wait_names_both_numbers(world):
    """内存放不下的排队原因写明两个数各是什么、点名记账最多的在跑任务（task_server.py 的 memory_wait_text、memory_verdict_reworded、verdict_within_the_formula；
    judge_host 那一句的「此刻空闲」是整机空闲减待涨量，看着像机器的可用内存）：本机 slice 总额 50G、余量 1G，在跑四条各上限 12G、各实占 10.5G，再来一条 10G 的——
    原因写「账面还能记 7.0 GiB」（连同总上限、余量与在跑的记账之和 42.0 GiB）、「slice 实测还放得下 8.0 GiB」，点名记账最多的两条（任务 701、702 各记 10.5 GiB、实占 10.5 GiB），
    第三条不点名，不写「此刻空闲」、不写乘几；judge_host 放得下、只有公式放不下的那一判（verdict_within_the_formula）写同一句。套在假调度器上，不起服务。
    弄坏开关 memory-wait-text-unnamed 下照 judge_host 与公式原来那一句写，这一格红。"""
    fake, snapshot, on_host, verdict = admission_judged(ADMISSION_MEASURED_TIGHT)
    wanted = ("账面还能记 7.0 GiB", f"{SERVER.RUNNING_BOOKED_SUM_TEXT} 42.0 GiB", "slice 实测还放得下 8.0 GiB",
              "任务 701 记 10.5 GiB、实占 10.5 GiB", "任务 702 记 10.5 GiB、实占 10.5 GiB")
    problems = []
    missing = [piece for piece in wanted if piece not in verdict.why]
    if verdict.fits or missing or "任务 703" in verdict.why or "此刻空闲" in verdict.why or "×" in verdict.why:
        problems.append(f"judge_host 判放不下：原因应当写 {'、'.join(wanted)}、不点名任务 703、不写「此刻空闲」、不写乘几，缺 {missing}，实际「{verdict.why}」")
    fits = fake.host_dispatch.Verdict(SERVER.LOCAL, True, False, on_host.memory_cap, "内存放得下")
    formula = SERVER.Scheduler.verdict_within_the_formula(fake, fits, on_host, SERVER.LOCAL, snapshot)
    if formula.fits or "账面还能记 7.0 GiB" not in formula.why or "slice 实测还放得下 8.0 GiB" not in formula.why:
        problems.append(f"只有公式放不下：原因应当写账面还能记 7.0 GiB 与 slice 实测还放得下 8.0 GiB，实际 {formula}")
    if problems:
        raise SelftestFailure("内存放不下的排队原因：" + "；".join(problems))
    return f"原因写「{verdict.why[:160]}」"


CARGO_CLASS_BASE = ("cargo", "test", "-p", "singlefs-verification", "--release")   # cargo-test-key-by-package：同包同档的那一类
CARGO_CLASS_PEAK_BYTES = 3 * GIBIBYTE   # 头一条量到的峰值：比没记录的 4096M 小，第二条照它起就是 3072M
CARGO_CLASS_FAKE_WORDS = ("fake-wrapper-exits=0",)   # 假包装照它退 0、不起 cargo；它在 cargo test 的位置参数上（测试名筛选），不进键


def cell_cargo_test_key_by_package(world):
    """cargo test 同包同特性同档归一个键（task_server_lib.py 的 cargo_test_class_words、task_server.py 的 command_key；点名验证包测试目标的每条新命令各从 4096M 起、
    撞三四次才编得过）：cargo test -p singlefs-verification --release --test alpha -- 用例 --exact 跑成、量到 3G；同包同档、点名 --test beta 的那一条命令键与它相同、
    头一次照它的记录起 3072M；再加 --features selftest-extra 的那一条键不同、没有记录，照 4096M 起。键里不带 --test 的值与 `--` 之后的用例；别的命令的词照原样。
    弄坏开关 cargo-test-key-whole-command 下整条命令当键，点名 beta 的那一条也照 4096M 起，这一格红。"""
    world.resources(local=(100, 100))
    world.start_server()
    first = world.finished(world.submit([*CARGO_CLASS_BASE, "--test", "selftest-target-alpha", "--", "selftest_case", "--exact",
                                         f"fake-wrapper-exit-peak={CARGO_CLASS_PEAK_BYTES}", *CARGO_CLASS_FAKE_WORDS]))
    second = world.finished(world.submit([*CARGO_CLASS_BASE, "--test", "selftest-target-beta", *CARGO_CLASS_FAKE_WORDS]))
    third = world.finished(world.submit([*CARGO_CLASS_BASE, "--features", "selftest-extra", "--test", "selftest-target-beta", *CARGO_CLASS_FAKE_WORDS]))
    keys = [summary["task"].get("command_key") or "" for summary in (first, second, third)]
    caps = [first_attempt(summary).get("memory_cap_text") for summary in (second, third)]
    other = ["bash", "-c", "exit 0", "--test", "selftest-target-alpha"]
    problems = []
    if any(summary["task"].get("outcome") != "succeeded" for summary in (first, second, third)):
        problems.append(f"三条都应当跑成，实际 {[describe(summary) for summary in (first, second, third)]}")
    if keys[0] != keys[1] or keys[1] == keys[2] or "selftest-target" in keys[0] or "selftest_case" in keys[0]:
        problems.append(f"点名 alpha 与 beta 的同包同档两条应当同一个键、键里不带测试目标与用例，加了特性的另一个键；实际 {keys}")
    if caps != [f"{CARGO_CLASS_PEAK_BYTES // MEBIBYTE}M", UNRECORDED_DEFAULT_PINNED_TEXT]:
        problems.append(f"点名 beta 的那一条应当照 alpha 量到的 3G 起 {CARGO_CLASS_PEAK_BYTES // MEBIBYTE}M、加了特性的照没记录的 {UNRECORDED_DEFAULT_PINNED_TEXT} 起，实际 {caps}")
    if lib.cargo_test_class_words(other) != other:
        problems.append(f"不是 cargo test 的命令应当照原样，实际 {lib.cargo_test_class_words(other)}")
    if problems:
        raise SelftestFailure("cargo test 同包同特性同档归一个键：" + "；".join(problems), world.server_log_lines())
    return f"点名不同测试目标的同包同档两条共用键「{keys[0]}」、第二条照 3G 起 {caps[0]}；加了特性的另一个键、照 {caps[1]} 起"


def cell_selection_key_by_script(world):
    """本仓门禁与入口脚本点名不同格、不同项归一个键（task_server_lib.py 的 selection_class_words、task_server.py 的 command_key）：
    bash .claude/gate.d/doc-text.sh --check date-once 与 --check=trilingual-files 两条的词相同、都是「--check *」；--item 与 --only 照同样去值；
    不带选法的整道照原样，与点名的不是一个键；不在本仓脚本目录下的命令（cargo fmt --check）照原样。
    弄坏开关 selection-key-whole-command 下照原样，点名不同格的两条词不同，这一格红。"""
    gate = ["bash", ".claude/gate.d/doc-text.sh"]
    first = lib.selection_class_words([*gate, "--check", "date-once"])
    second = lib.selection_class_words([*gate, "--check=trilingual-files"])
    entry = ["env", "A=1", "bash", "/repository/scripts/crash/crash-amplification.sh"]
    items = lib.selection_class_words([*entry, "--item", "one", "--item", "two"])
    whole = lib.selection_class_words(gate)
    other = ["cargo", "fmt", "--check"]
    problems = []
    if first != second or first != [*gate, "--check", "*"]:
        problems.append(f"点名不同格的两条应当同为 {[*gate, '--check', '*']}，实际 {first} 与 {second}")
    if items != [*entry, "--item", "*"]:
        problems.append(f"点名两项的入口应当去值只留一个「--item *」，实际 {items}")
    if whole != gate or whole == first:
        problems.append(f"不带选法的整道应当照原样、与点名的不同，实际 {whole}")
    if lib.selection_class_words(other) != other:
        problems.append(f"不是本仓脚本的命令应当照原样，实际 {lib.selection_class_words(other)}")
    if problems:
        raise SelftestFailure("本仓门禁与入口脚本点名不同格、不同项归一个键：" + "；".join(problems), [])
    return f"点名不同格的两条共用「{' '.join(first)}」，点名两项的入口共用「--item *」，整道与别的命令照原样"


ACTIVITY_GROWTH_PER_TICK_BYTES = 16 * MEBIBYTE   # memory-growth-not-stalled：假 scope 的进程内存每 0.2 秒涨这么多（一个 2 秒的窗口涨 160 MiB，过 ACTIVITY_MEMORY_BYTES）
ACTIVITY_QUIET_COMMAND = "echo started; sleep 8"   # 那一格两条的正文：只打一行、静 8 秒（四个卡住窗口）


def cell_memory_growth_not_stalled(world):
    """进程内存在涨算有进展（task_server.py 的 note_activity、activity_text、judge_stall_stop）：
    卡住窗口 2 秒、宽限 0.01 秒，假 scope 的 CPU 不涨。头一条只打一行、静 8 秒，这段时间假 scope 的进程内存
    每 0.2 秒涨 16 MiB：日志与 CPU 都不涨，照样不停、跑完；之后进程内存不动，第二条同样静 8 秒：记 stalled。
    另静态核 activity_text：进程内存一个窗口里从 100 MiB 涨到 200 MiB 算有进展，只涨 10 MiB 不算，窗口不是这一个的不算。
    弄坏开关 progress-ignored-for-stall 下头一条也被停、记 stalled，这一格红。"""
    key = "selftest-memory-growth"
    scope = os.path.join(world.root, "growing-memory-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    stat = os.path.join(scope, "memory.stat")
    with open(stat, "w", encoding="utf-8") as handle:
        handle.write(process_memory_stat(64 * MEBIBYTE))
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    stop = threading.Event()

    def grow():
        used = 64 * MEBIBYTE
        while not stop.is_set():
            used += ACTIVITY_GROWTH_PER_TICK_BYTES
            with open(stat + ".partial", "w", encoding="utf-8") as handle:
                handle.write(process_memory_stat(used))
            os.replace(stat + ".partial", stat)
            time.sleep(0.2)

    grower = threading.Thread(target=grow, daemon=True)
    grower.start()
    try:
        growing = world.finished(world.submit(["bash", "-c", ACTIVITY_QUIET_COMMAND, "memory-growing"], extra={lib.KEY_VARIABLE: key}), timeout=45)
    finally:
        stop.set()
        grower.join()
    flat = world.finished(world.submit(["bash", "-c", ACTIVITY_QUIET_COMMAND, "memory-flat"], extra={lib.KEY_VARIABLE: key}), timeout=45)
    problems = []
    if growing["task"].get("outcome") != "succeeded":
        problems.append(f"进程内存每 0.2 秒涨 16 MiB 的那一条（日志与 CPU 都不涨）应当照跑完，实际 {describe(growing)}")
    if flat["task"].get("outcome") != "stalled":
        problems.append(f"进程内存不动的那一条应当记 stalled，实际 {describe(flat)}")
    moving, still = (SERVER.ActivityWindow(1.0, first, None, last, None) for first, last in ((100 * MEBIBYTE, 200 * MEBIBYTE), (100 * MEBIBYTE, 110 * MEBIBYTE)))
    if not SERVER.activity_text(moving, 1.0) or SERVER.activity_text(still, 1.0) or SERVER.activity_text(moving, 2.0):
        problems.append(f"进程内存从 100 MiB 涨到 200 MiB 应当算有进展、只涨 10 MiB 与别的窗口的不算，实际「{SERVER.activity_text(moving, 1.0)}」"
                        f"「{SERVER.activity_text(still, 1.0)}」「{SERVER.activity_text(moving, 2.0)}」")
    if problems:
        raise SelftestFailure("有进展的不判卡住：" + "；".join(problems), world.server_log_lines())
    return "进程内存在涨的那一条照跑完、不动的记 stalled；静态核进程内存涨够了才算有进展"


def signal_note_scheduler(stopped_attempt_ids=()):
    """signal-exit-named 的假调度器：stop_running 对哪几次尝试发过停（service_stopped_attempts）。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.service_stopped_attempts = {attempt_id: time.time() for attempt_id in stopped_attempt_ids}
    return fake


def cell_signal_exit_named(world):
    """命令被信号终止时原因句写出信号名、点名服务发没发过停的信号（task_server.py 的 signal_note、service_stop_step；task_server_lib.py 的 reason_fields；
    任务 13455 第 6 次的 SIGTERM，结局只写「命令自己退 1」）：起服务交一条 bash -c 'kill -TERM $$'：结局 command-failed，那一次尝试的 signal_note 与理由行
    的 why 都写「SIGTERM」与「服务这一次没有发过停的信号」。静态再核两种：尝试单元收到 SIGTERM（stopped_signal 15）、命令退 1，而 stop_running 对它发过停：
    写「SIGTERM」与「停尝试的单元」；判卡住停的写「判卡住」；退 1、没收到信号的交空串。弄坏开关 signal-exit-unnamed 下一句都不写，这一格红。"""
    world.start_server()
    summary = world.finished(world.submit(["bash", "-c", "kill -TERM $$", "signal-self"]))
    attempt = first_attempt(summary)
    note = attempt.get("signal_note") or ""
    why = dict(lib.reason_fields(summary["task"], summary["attempts"], "")).get("why", "")
    problems = []
    if summary["task"].get("outcome") != "command-failed" or "SIGTERM" not in note or "服务这一次没有发过停的信号" not in note or note not in why:
        problems.append(f"自己收到 SIGTERM 的命令应当记 command-failed、signal_note 写 SIGTERM 与「服务这一次没有发过停的信号」、理由行接上它，"
                        f"实际 {describe(summary)}、signal_note「{note}」、why「{why}」")
    stopped = SERVER.Scheduler.signal_note(signal_note_scheduler([7]), {"id": 1}, {"id": 7}, {"stopped_signal": 15}, 1)
    stalled = SERVER.Scheduler.signal_note(signal_note_scheduler(), {"id": 1}, {"id": 8, "stalled": True}, {"stopped_signal": 15}, 1)
    plain = SERVER.Scheduler.signal_note(signal_note_scheduler(), {"id": 1}, {"id": 9}, {"stopped_signal": 0}, 1)
    if "SIGTERM" not in stopped or "停尝试的单元" not in stopped or "判卡住" not in stalled or plain:
        problems.append(f"单元收到 SIGTERM、stop_running 发过的应当写 SIGTERM 与「停尝试的单元」，判卡住的写「判卡住」，没收到信号的空串；"
                        f"实际「{stopped}」「{stalled}」「{plain}」")
    if problems:
        raise SelftestFailure("被信号终止的写出信号名：" + "；".join(problems), world.server_log_lines())
    return f"自己收到 SIGTERM 的写「{note}」；服务停的点名是哪一步"


def cell_shard_key_without_count(world):
    """片的命令键只带片号、不带片数（task_server_lib.py 的 shard_child_key；admission 审核 F7）：同一件第 3 片在共 5 片与共 64 片时键相同（过了键的换算也相同），
    与第 2 片不同。弄坏开关 shard-key-count-kept 下键里带片数、片数一变就换新键，这一格红。"""
    base = "bash .claude/gate.d/selftest-gate.sh"
    five, sixty_four, other = (PEAKS.command_key([], lib.shard_child_key(base, "shard", index, count)) for index, count in ((3, 5), (3, 64), (2, 64)))
    if five != sixty_four or five == other:
        raise SelftestFailure(f"第 3 片共 5 片与共 64 片的键应当相同、与第 2 片不同，实际「{five}」「{sixty_four}」「{other}」")
    return f"第 3 片的键不随片数变：「{five}」"


def cell_stopped_peaks_floor_record(world):
    """被停、判红的那一次量到的峰值当下限进定额度（task_server.py 的 stopped_peak_floor、recorded_prediction；admission 审核 F6）：撞过 13824M（峰值 13.48G）、之后被取消的那一次
    量到 15.21G：下一次起步不低于 15.21G（不再从撞过的那一档重撞）；跑成 2G、命令退 1 的那一次 100M：照 2G（小的不拉低）；只有一次取消、量到 6G：照 6G 起（比没记录的 4G 大）；
    只有一次取消、量到 1G：交 None，照没记录的 4G 起。弄坏开关 stopped-peaks-not-counted 下被停的峰值丢掉（头一种起步 13824M、第三种照 4G），这一格红。"""
    dispatch = loaded_host_dispatch()
    default = lib.UNRECORDED_MEMORY_DEFAULT_BYTES
    base = dispatch.Prediction("selftest-stopped-peaks", default, default, default, lib.UNRECORDED_HISTORY)

    def row(outcome, peak, ended_at, cap=None):
        return ({"outcome": outcome, "memory_peak": int(peak), "memory_cap": cap, "ended_at": ended_at, "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC}, True)
    problems = []
    for what, rows, wanted in (("撞过 13824M、之后取消时量到 15.21G", [row("cancelled", 15.21 * GIBIBYTE, 5), row("memory-cap-hit", 13.48 * GIBIBYTE, 4, 13824 * MEBIBYTE)],
                                int(13824 * MEBIBYTE * lib.CAP_GROWTH_FACTOR)),   # 撞过、之后没跑完过：照撞过的上限乘 1.5（20736M），比取消时量到的 15.21G 大
                               ("跑成 2G、命令退 1 时 100M", [row("command-failed", 100 * MEBIBYTE, 5), row("succeeded", 2 * GIBIBYTE, 4)], 2 * GIBIBYTE),
                               ("只有一次取消、量到 6G", [row("cancelled", 6 * GIBIBYTE, 5)], 6 * GIBIBYTE),
                               ("只有一次取消、量到 1G", [row("cancelled", GIBIBYTE, 5)], None)):
        fake = object.__new__(SERVER.Scheduler)
        fake.environment = dict(os.environ)
        fake.store = types.SimpleNamespace(select=lambda *arguments, table=rows, **options: list(table))
        found = SERVER.Scheduler.recorded_prediction(fake, {"command_key": "selftest-stopped-peaks"}, base)
        predicted = None if found is None else int(found.memory_predicted)
        if predicted != wanted:
            problems.append(f"{what}：起步应当 {wanted}，实际 {predicted}")
    if problems:
        raise SelftestFailure("被停的峰值当下限：" + "；".join(problems))
    return "取消那一次的 15.21G 抬过撞过的 13824M；小的不拉低；只有被停的、比缺省大的照它起"


def cell_compiles_declared_entry(world):
    """入口会编的声明（task_server_lib.py 的 declared_compiles、task_server.py 的 compiles；lifecycle 审核 F1）：入口文件头写「# task-server-compiles: yes」的，
    命令那一层认不出 cargo 也算会编（服务照给编译目录、撞上限重排接着用上一次那一份）；没写的不算；只带 --selftest 的不算；写成别的词的交写坏的那一句（派活打回）。
    弄坏开关 compiles-declaration-ignored 下声明了也不算，这一格红。"""
    def entry(name, header):
        path = os.path.join(world.root, name)
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("#!/usr/bin/env bash\n" + header + "exit 0\n")
        return path
    declared, plain, written_wrong = (entry(name, header) for name, header in (("entry-compiles.sh", f"{lib.DECLARED_COMPILES_MARK} {lib.COMPILES_DECLARED_WORD}\n"),
                                                                              ("entry-plain.sh", ""), ("entry-wrong.sh", f"{lib.DECLARED_COMPILES_MARK} maybe\n")))
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.cargo_lock = types.SimpleNamespace(cargo_invocations=lambda *arguments: [], BUILDING_SUBCOMMANDS={"build"})
    fake.project_code_runs = types.SimpleNamespace(shell_words=None)

    def task(path, *rest):
        return {"command": ["bash", path, *rest], "cwd": world.root, "environment": {}}
    problems = []
    for what, found, wanted in (("声明了会编的入口", SERVER.Scheduler.compiles(fake, task(declared, "--item", "selftest")), True),
                                ("没声明的入口", SERVER.Scheduler.compiles(fake, task(plain, "--item", "selftest")), False),
                                ("声明了、只带 --selftest", SERVER.Scheduler.compiles(fake, task(declared, "--selftest")), False)):
        if found != wanted:
            problems.append(f"{what}：会编应当 {wanted}，实际 {found}")
    wrong = SERVER.Scheduler.compiles_declared(fake, task(written_wrong))
    if wrong[0] or "maybe" not in wrong[1]:
        problems.append(f"写成 maybe 的：应当不算、交写坏的那一句，实际 {wrong}")
    if problems:
        raise SelftestFailure("入口会编的声明：" + "；".join(problems))
    return "声明了 yes 的入口算会编；没声明的、只自证的不算；写坏的交那一句"


def cell_parent_cancelling_child_not_claimed(world):
    """整件正在取消时排着的子任务不起（task_server.py 的 Store.claim_attempt；admission 审核 F10）：父任务记了 cancel_requested、阶段 cancelling，排着的那一片照
    claim_attempt 起：抛 StoreRefused、kind 是 parent-cancelling，库里没插尝试。服务撤的片理由行照 cancel_origin 写（task_server_lib.py 的 reason_fields）：
    hard-limit、sibling、parent 各写服务撤的那一句，不写提交它的会话取消。
    弄坏开关 parent-cancel-unchecked-at-claim 下照起，cancel-origin-unread 下一律写提交它的会话，这一格红。"""
    store = SERVER.Store(os.path.join(world.root, "claim-parent-cancelling.sqlite3"), bytes(32), dict(os.environ))
    parent = store.insert("tasks", {"state": "sharded", "session": "", "command_key": "", "task_id": 0, "cancel_requested": True, "phase": "cancelling"})
    child = store.insert("tasks", {"state": "queued", "session": "", "command_key": "", "task_id": parent["id"], "parent_id": parent["id"], "role": "shard",
                                   "shard_index": 0, "attempt_count": 0, "cancel_requested": False})
    problems = []
    kind = ""
    try:
        store.claim_attempt(child["id"], 1, {"state": "starting", "session": "", "command_key": ""}, {"state": "running"})
    except SERVER.StoreRefused as refused:
        kind = refused.kind
    if kind != "parent-cancelling" or store.attempts_of(child["id"]):
        problems.append(f"父任务整件正在取消：排着的片应当不起（parent-cancelling），实际 kind「{kind}」、库里尝试 {len(store.attempts_of(child['id']))} 次")
    for origin in ("hard-limit", "sibling", "parent"):
        fields = dict(lib.reason_fields({"id": child["id"], "outcome": "cancelled", "cancel_origin": origin, "note": "自证"}, [], ""))
        if fields.get("by") != lib.CANCEL_ORIGIN_TEXTS[origin] or "服务撤的" not in str(fields.get("by")):
            problems.append(f"cancel_origin={origin} 的理由行应当写「{lib.CANCEL_ORIGIN_TEXTS[origin]}」，实际「{fields.get('by')}」")
    if problems:
        raise SelftestFailure("整件取消时不起子任务、服务撤的写对理由：" + "；".join(problems))
    return "父任务正在取消时排着的片不起；服务撤的片理由行写服务撤的那一句"


SHARD_PEAK_KEY = "selftest-shard-peak-count"   # shard-peak-record-count-matched：库里记过峰值的那把片的命令键


def cell_shard_peak_record_count_matched(world):
    """定额度读片的记录只认片数相同的那几趟（task_server.py 的 record_fits_task、recorded_measurements、stopped_peak_floor；片的键不带片数，两片那一趟的第 0 片记过 49.5G，
    换成几十片之后同一片号落在同一把键上）：共 7 片的一片，库里同一把键有两片那一趟跑成三次各 49.5G（尝试自己记了 shard_count）、7 片那一趟跑成 5G（尝试记了 shard_count）、
    6G（尝试没记、读子任务的 shard_count 是 7）、尝试没记而子任务是两片那一趟的 48G、尝试没记而子任务读不到的 47G，另有被取消的两片那一趟 30G、7 片那一趟 2G：
    近几次的量应当只有 5G 与 6G，被停的下限是 2G；不是片的任务照旧三样都认。静态核，不起服务。
    弄坏开关 record-shard-count-unchecked 下片数不同的也拿来，这一格红。"""
    def ended(outcome, peak, ended_at, **fields):
        return ({"command_key": SHARD_PEAK_KEY, "outcome": outcome, "memory_peak": int(peak), "ended_at": ended_at, "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC, **fields}, True)
    rows = ([ended("succeeded", 49.5 * GIBIBYTE, 100 + number, shard_count=2) for number in range(3)]
            + [ended("succeeded", 5 * GIBIBYTE, 103, shard_count=7), ended("succeeded", 6 * GIBIBYTE, 104, task_id=71),
               ended("succeeded", 48 * GIBIBYTE, 105, task_id=72), ended("succeeded", 47 * GIBIBYTE, 106, task_id=73),
               ended("cancelled", 30 * GIBIBYTE, 107, shard_count=2), ended("cancelled", 2 * GIBIBYTE, 108, shard_count=7)])
    shard_counts = {71: 7, 72: 2}
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.store = types.SimpleNamespace(
        select=lambda table, where="1", parameters=(), order="id": [row for row in rows if row[0].get("command_key") == parameters[0]] if table == "attempts" else [],
        task=lambda task_id: ({"id": task_id, "shard_count": shard_counts[task_id]}, True) if task_id in shard_counts else None)
    problems = []
    piece = {"role": "shard", "shard_count": 7, "command_key": SHARD_PEAK_KEY}
    measured = sorted(value for _, value in SERVER.Scheduler.recorded_measurements(fake, piece))
    floor = SERVER.Scheduler.stopped_peak_floor(fake, piece)
    if measured != [5 * GIBIBYTE, 6 * GIBIBYTE] or floor != 2 * GIBIBYTE:
        problems.append(f"共 7 片的一片：近几次的量应当只有 5G 与 6G、被停的下限 2G，实际量 {[round(value / GIBIBYTE, 1) for value in measured]}G、下限 {None if floor is None else round(floor / GIBIBYTE, 1)}G")
    plain = {"role": "", "command_key": SHARD_PEAK_KEY}
    measured_plain = sorted(value for _, value in SERVER.Scheduler.recorded_measurements(fake, plain))
    floor_plain = SERVER.Scheduler.stopped_peak_floor(fake, plain)
    if len(measured_plain) != 7 or floor_plain != 30 * GIBIBYTE:
        problems.append(f"不是片的任务照旧不看片数：应当 7 个量、被停的下限 30G，实际 {len(measured_plain)} 个量、下限 {None if floor_plain is None else round(floor_plain / GIBIBYTE, 1)}G")
    if problems:
        raise SelftestFailure("定额度读片的记录只认片数相同的：" + "；".join(problems))
    return "共 7 片的一片只取 7 片那几趟的 5G、6G（下限 2G），两片那一趟与读不到片数的不取；不是片的照旧都取"


HEARTBEAT_ONLY_SCRIPT = 'echo started; for round in $(seq 1 40); do echo "TASK_SERVER_HEARTBEAT 编译中 $round"; sleep 0.5; done'   # 20 秒只打心跳行
HEARTBEAT_MIXED_SCRIPT = 'echo started; for round in 1 2 3 4 5 6; do echo "TASK_SERVER_HEARTBEAT 编译中 $round"; echo "real output $round"; sleep 0.5; done'   # 心跳行里夹着真输出


def cell_heartbeat_lines_not_log_growth(world):
    """命令自己打的心跳行不算日志在涨（task_server.py 的 log_content、judge_stall_stop；task_server_lib.py 文件头「命令自己打的两种登记行」）：卡住窗口 2 秒、宽限 0.01 秒，
    假 scope 的 CPU 读得到、一直不涨：每 0.5 秒打一行 TASK_SERVER_HEARTBEAT、跑 20 秒的一条被停、记 stalled；心跳行里夹着每 0.5 秒一行真输出的一条照跑完；
    理由行里的「日志最后一行」跳过心跳行。弄坏开关 heartbeat-counted-as-log 下只打心跳行的那一条也照跑完，这一格红。"""
    scope = os.path.join(world.root, "idle-scope")
    os.makedirs(scope)
    with open(os.path.join(scope, "cpu.stat"), "w", encoding="utf-8") as handle:
        handle.write("usage_usec 1000\n")
    world.start_server(extra={"TASK_SERVER_SELFTEST_STALL_SECONDS": "2", "TASK_SERVER_SELFTEST_STALL_GRACE_SECONDS": "0.01", "TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    only = world.finished(world.submit(["bash", "-c", HEARTBEAT_ONLY_SCRIPT, "heartbeat-only"]), timeout=45)
    mixed = world.finished(world.submit(["bash", "-c", HEARTBEAT_MIXED_SCRIPT, "heartbeat-mixed"]), timeout=45)
    problems = []
    if only["task"].get("outcome") != "stalled":
        problems.append(f"只打心跳行、CPU 不涨的那一条满 2 秒应当被停、记 stalled，实际 {describe(only)}")
    if mixed["task"].get("outcome") != "succeeded":
        problems.append(f"心跳行里夹着真输出的那一条应当照跑完，实际 {describe(mixed)}")
    last = lib.log_last_line("real output 6\nTASK_SERVER_HEARTBEAT 编译中 6\n")
    if last != "real output 6":
        problems.append(f"理由行里的日志最后一行应当跳过心跳行取「real output 6」，实际「{last}」")
    if problems:
        raise SelftestFailure("心跳行不算日志在涨：" + "；".join(problems), world.server_log_lines())
    return "只打心跳行、CPU 不涨的满窗口记 stalled；夹着真输出的照跑完；最后一行跳过心跳行"


CPU_GATE_PROBE_KEY = "selftest-cpu-gate-recorded"   # cpu-gate-new-task-by-recorded-cores：库里记过平均在用核数的那把命令键
CPU_GATE_HEAVY_KEY = "selftest-cpu-gate-heavy"      # 库里另记过 heavy_cores 的那把命令键（cpu-gate-running-started-by-recorded-cores）


def seed_peak_readings(fake, attempts, usage_at, now):
    """假调度器的峰值段读数表（Scheduler.note_peak_segment 读写的 running_peak_samples）照服务一路在看记两回：起跑那一刻（累计 0）与 now（usage_at(尝试)）。
    起了满 CPU_MEASURED_AFTER_SECONDS 的，下一回 running_threads 读到的那一段就是起跑到 now 这一段（各格的假本机一路同一个核数，峰值段就等于它）；不满的照服务还没量满一段办。"""
    for attempt in attempts:
        fake.running_peak_samples[attempt["id"]] = {"first_seen": attempt["started_at"], "samples": [(attempt["started_at"], 0.0), (now, usage_at(attempt))]}


def cpu_gate_scheduler(running_count, cpu, ages=None, cores=1.0, recorded_cores=(), keys=None, recorded_rows=None, heavy_cores=()):
    """cpu-gate-counts-running-threads 的假调度器与 Snapshot：本机在跑 running_count 条、各登记 31 线程，起了多少秒照 ages（缺省各 100 秒，不到 CPU_MEASURED_AFTER_SECONDS），
    一路在用核数都是 cores（假本机的 cpu_usec 照它与建这个假世界那一刻之前的已跑秒数算；峰值段读数照 seed_peak_readings 记起跑与建世界两回）；报告里的 CPU 是 cpu（None 是读不到）。
    recorded_cores 是 CPU_GATE_PROBE_KEY 这把命令键库里跑成的那几次记的核数，heavy_cores 是 CPU_GATE_HEAVY_KEY 那几次的（峰值段核数、CPU 闸线程数与平均在用核数三栏同值；
    admission_threads、running_threads 读；空的是没有记录）；keys 是各条在跑的尝试的命令键（缺省都不带命令键）；cores 给 None 是假本机量不到 CPU（cpu_usec 交 None）。
    recorded_rows 给了就整个换掉 recorded_cores 造的那几行（每行是尝试记录的 outcome、exit_code、cores，峰值段核数与 CPU 闸线程数照 cores 补，命令键补成 CPU_GATE_PROBE_KEY）。
    线程读数（note_threads 的 running_peak_runnable、thread_overruns）是空的，假本机不报线程读数（各格要的自己换 thread_reading）。"""
    now = time.time()
    peak, gate = SERVER.PEAK_SEGMENT_CORES_COLUMN, SERVER.CPU_GATE_THREADS_COLUMN
    running = [dict({"id": position + 1, "host": SERVER.LOCAL, "threads": 31, "started_at": now - age}, **({"command_key": keys[position]} if keys and keys[position] else {}))
               for position, age in enumerate(ages or [100] * running_count)]
    recorded = [({"outcome": "succeeded", "cores": value, peak: value, gate: value, "command_key": CPU_GATE_PROBE_KEY}, True) for value in recorded_cores]
    if recorded_rows is not None:
        recorded = [(dict({peak: row.get("cores"), gate: row.get("cores")}, **row, command_key=CPU_GATE_PROBE_KEY), True) for row in recorded_rows]
    by_key = {(CPU_GATE_PROBE_KEY,): recorded,
              (CPU_GATE_HEAVY_KEY,): [({"outcome": "succeeded", "cores": value, peak: value, gate: value, "command_key": CPU_GATE_HEAVY_KEY}, True) for value in heavy_cores]}
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.peaks = PEAKS
    fake.threads = 31
    fake.admission_threads_cache = {}
    fake.running_threads_cache = {}
    fake.running_usage_lock = threading.Lock()   # CPU 闸读数表的锁（Scheduler.__init__ 建的，假调度器照建）
    fake.unit_stopped = set()   # booking_attempts 不数的、单元已确认停了的尝试号（Scheduler.__init__ 建的）
    fake.running_peak_samples, fake.running_peak_cores = {}, {}   # note_peak_segment 的读数表与各段最大（Scheduler.__init__ 建的）
    fake.running_latest_cores, fake.running_latest_runnable = {}, {}   # 最近一段的在用核数与最近一回的可跑线程数（latest_running_cores 读）
    fake.running_peak_runnable, fake.thread_overruns = {}, {}   # note_threads 记的可跑线程数与线程远超登记数（Scheduler.__init__ 建的）
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(cpu_usec=lambda attempt: None if cores is None else cores * 1e6 * (now - attempt["started_at"]),
                                                         thread_reading=lambda attempt: None, label="本机")}
    if cores is not None:
        seed_peak_readings(fake, running, fake.machines[SERVER.LOCAL].cpu_usec, now)
    fake.store = types.SimpleNamespace(running_attempts=lambda: [dict(attempt) for attempt in running],
                                       select=lambda table, where="1", parameters=(), order="id": list(by_key.get(tuple(parameters), [])))
    report = memory_report(30, 30)
    if cpu is None:
        report["memory"].pop(PEAKS.CPU_REPORT_KEY, None)
    else:
        report["memory"][PEAKS.CPU_REPORT_KEY] = dict(cpu)
    return fake, SERVER.Snapshot({SERVER.LOCAL: report}, {}, {SERVER.LOCAL: True}, {})


def cell_cpu_gate_counts_running_threads(world):
    """CPU 闸一直记着在跑的线程、读不到不放行（task_server.py 的 cpu_problem；调度溢出审核 CPU-1、CPU-2）：32 核（线 64）、负载 3，在跑的与要起的这一条的命令键都记过 31 核：
    在跑一条（31 线程）再起一条 31 线程放得下；在跑两条（起了 100 秒、不到 CPU_MEASURED_AFTER_SECONDS，照记录各记 31 线程共 62，负载里看不出）再起一条超过、原因以「CPU：」起头；
    报告里没有 CPU 那一项：这一轮不派、原因以 CPU_UNREAD_TEXT 起头。
    弄坏开关 cpu-threads-settling-only 下只记刚起 5 秒的线程（两条那一种放行），cpu-unread-passed 下读不到的放行，这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for what, count, cpu, starts in (("在跑一条", 1, load, ""), ("在跑两条", 2, load, "CPU："), ("读不到 CPU", 0, None, SERVER.CPU_UNREAD_TEXT)):
        fake, snapshot = cpu_gate_scheduler(count, cpu, recorded_cores=(31.0,), keys=[CPU_GATE_PROBE_KEY] * count)
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 1, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if (not starts and why) or (starts and not why.startswith(starts)):
            problems.append(f"{what}：应当{'放行' if not starts else '挡住、原因以「' + starts + '」起头'}，实际「{why}」")
    if problems:
        raise SelftestFailure("CPU 闸：" + "；".join(problems))
    return "在跑一条再起一条放得下；在跑两条再起超过；读不到 CPU 的不派"


def cell_cpu_gate_measured_cores(world):
    """CPU 闸里在跑的照实测核数记（task_server.py 的 running_threads、cpu_problem；任务 13256 排在三条 31 线程的单线程任务后面）：32 核（线 64）、负载 3，
    三条在跑、各登记 31 线程而一路实测 1 核（峰值段 1 核）、都起了 1000 秒（满 CPU_MEASURED_AFTER_SECONDS）：记 3 线程、第四条照起；其中一条只起了 30 秒、命令键记过 31 核：它照记录记 31、共 33 线程。
    弄坏开关 cpu-threads-registered-only 下一律照登记数记（共 93），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for what, ages, threads in (("三条都满了时长", [1000, 1000, 1000], 3), ("一条只起了 30 秒", [1000, 1000, 30], 33)):
        fake, snapshot = cpu_gate_scheduler(3, load, ages=ages, cores=1.0, recorded_cores=(31.0,), keys=[None, None, CPU_GATE_PROBE_KEY])
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 1}, SERVER.LOCAL, snapshot)
        if counted != threads or why:
            problems.append(f"{what}：在跑的应当记 {threads} 线程、第四条照起，实际记 {counted} 线程、「{why}」")
    if problems:
        raise SelftestFailure("CPU 闸照实测核数记在跑的：" + "；".join(problems))
    return "满了时长的照实测 1 核记、第四条照起；没满的照命令键的记录 31 记"


def cell_cpu_gate_new_task_by_recorded_cores(world):
    """CPU 闸里新起的这一条照同一命令键近几次跑成的峰值段核数取最大记（task_server.py 的 admission_threads、threads_by_record、cpu_problem；任务 13489 单线程的小脚本照 31 线程排了 30 分钟）：
    32 核（线 64）、负载 3，三条在跑、都起了 1000 秒、实测各 17.5 核（记 18、共 54 线程）。库里跑成三次、峰值段 1、1、0.7 核的那一条记 1 线程，54 + 1 不超过 64，照起；
    库里跑成三次、各 20 核的那一条记 20 线程，54 + 20 超过 64，排着、原因写「20 线程」。弄坏开关 cpu-wanted-by-registered-threads 下有记录的也照 31 记（头一种排着），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for what, recorded, blocked in (("记过最大 1 核的", (1.0, 1.0, 0.7), False), ("记过 20 核的", (20.0, 20.0, 20.0), True)):
        fake, snapshot = cpu_gate_scheduler(3, load, ages=[1000, 1000, 1000], cores=17.5, recorded_cores=recorded)
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 1, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if counted != 54 or bool(why) != blocked or (blocked and "（20 线程" not in why):
            problems.append(f"{what}：在跑的应当记 54 线程、这一条{'照 20 线程排着、原因写「20 线程」' if blocked else '照 1 线程起'}，实际记 {counted} 线程、「{why}」")
    if problems:
        raise SelftestFailure("CPU 闸照记录的核数算新起的：" + "；".join(problems))
    return "在跑 54 线程：记过最大 1 核的照 1 线程起；记过 20 核的照 20 线程排着"


def cell_cpu_gate_new_task_counts_failed_runs(world):
    """CPU 闸里新起的这一条照历史核数记时，命令自己退非 0、跑到头的几次也算（task_server.py 的 threads_by_record；弄坏开关轮这类按设计退非 0 的命令也照记录记）：
    32 核（线 64）、负载 3，三条在跑、都起了 1000 秒、实测各 17.5 核（记 18、共 54 线程）。库里三次 command-failed（退 1）各记 20 核的那一条记 20 线程，54 + 20 超过 64，排着；
    一次 command-failed 退 143（被 TERM 杀的）、一次 killed-externally、一次 memory-cap-hit 各记 20 核的不算，只剩一次跑成的 1 核，记 1 线程，照起（算进去最大 20 核、排着）；
    跑成两次（1.0 核）加一次 command-failed（退 1、0.3 核）最大 1 核，记 1 线程，照起。
    弄坏开关 threads-record-succeeded-only 下只认跑成的：头一种没有记录、照登记的 31 线程排着、原因里不是「20 线程」，这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    failed = {"outcome": "command-failed", "exit_code": 1, "cores": 20.0}
    unfinished = [{"outcome": "command-failed", "exit_code": 143, "cores": 20.0}, {"outcome": "killed-externally", "exit_code": 1, "cores": 20.0},
                  {"outcome": "memory-cap-hit", "exit_code": 250, "cores": 20.0}, {"outcome": "succeeded", "exit_code": 0, "cores": 1.0}]
    problems = []
    for what, rows, blocked in (("三次命令自己退 1", [failed] * 3, True), ("被信号杀的、外部杀的、撞上限的加一次跑成", unfinished, False),
                                ("跑成两次加一次退 1", [{"outcome": "succeeded", "exit_code": 0, "cores": 1.0}] * 2 + [dict(failed, cores=0.3)], False)):
        fake, snapshot = cpu_gate_scheduler(3, load, ages=[1000, 1000, 1000], cores=17.5, recorded_rows=rows)
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 1, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if bool(why) != blocked or (blocked and "（20 线程" not in why):
            problems.append(f"{what}：这一条应当{'照 20 线程排着、原因写「20 线程」' if blocked else '照 1 线程起'}，实际「{why}」")
    if problems:
        raise SelftestFailure("CPU 闸照历史核数记新起的（退非 0 跑到头的也算）：" + "；".join(problems))
    return "历史里命令自己退非 0 的三次照 20 线程排着；被信号杀的、外部杀的、撞上限的不算，照跑成的 1 线程起；跑成加退 1 的混着算"


def cell_cpu_gate_unrecorded_by_registered_threads(world):
    """CPU 闸里没有核数记录的新命令量到之前按登记的线程数记（task_server.py 的 threads_by_record；用户定「按 31 核，量到后改实测」；按 1 核算时任务 15262 到 15299
    共 38 条新命令十秒内齐起）：命令键库里一次跑到头的都没有、登记 31 线程的记 31 线程，那一句写「先按登记的 31 线程」；没有命令键的同样；登记 4 线程的记 4。
    库里记过 5 核的照记录记 5。套在假调度器上，不起服务。弄坏开关 cpu-unrecorded-by-one-core 下没有记录的记 1 线程，这一格红。"""
    fake, _ = cpu_gate_scheduler(0, {"load1": 1.0, "procs_running": 1, "cpus": 32}, recorded_cores=(5.0,))
    problems = []
    for what, key, registered, wanted in (("没有记录、登记 31", "selftest-cpu-gate-never-ran", 31, 31), ("没有命令键、登记 31", "", 31, 31),
                                          ("没有记录、登记 4", "selftest-cpu-gate-never-ran", 4, 4), ("记过 5 核", CPU_GATE_PROBE_KEY, 31, 5)):
        threads, how = SERVER.Scheduler.threads_by_record(fake, key, registered)
        if threads != wanted or (wanted == registered and key != CPU_GATE_PROBE_KEY and f"先按登记的 {registered} 线程" not in how):
            problems.append(f"{what}：应当记 {wanted} 线程，没有记录的那一句写「先按登记的 {registered} 线程」，实际 {threads}、「{how}」")
    if problems:
        raise SelftestFailure("CPU 闸里没有记录的新命令按登记线程数：" + "；".join(problems))
    return "没有核数记录的新命令（连同没有命令键的）量到之前按登记的线程数记：登记 31 的记 31、登记 4 的记 4；记过 5 核的照记录记 5"


def cell_cpu_gate_running_after_restart_by_average(world):
    """服务刚重启、还没量满一段时，起跑已满两分钟的在跑尝试照累计 CPU ÷ 已跑秒数记（task_server.py 的 running_threads）：两条登记 31 线程、没有命令键、起了 1000 秒、一路 3.2 核，
    读数表是空的（演服务重启）：各记 4 线程、共 8；另一条只起了 30 秒的照旧按登记的 31 线程记。套在假调度器上，不起服务。
    弄坏开关 cpu-restart-unmeasured-by-record 下起了 1000 秒的两条各记 31 线程，这一格红。"""
    fake, _ = cpu_gate_scheduler(3, {"load1": 1.0, "procs_running": 1, "cpus": 32}, ages=[1000, 1000, 30], cores=3.2)
    fake.running_peak_samples, fake.running_peak_cores, fake.running_latest_cores = {}, {}, {}   # 服务重启：读数表从空的起
    total = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
    if total != 4 + 4 + 31:
        raise SelftestFailure(f"服务刚重启时的在跑线程数：起了 1000 秒、一路 3.2 核的两条应当各记 4 线程，起了 30 秒的一条记登记的 31 线程，共 39，实际 {total}")
    return "读数表是空的（服务刚重启）：起了 1000 秒、一路 3.2 核的两条各记 4 线程，起了 30 秒的照登记的 31 线程，共 39"


def cell_cpu_gate_running_by_latest_reading(world):
    """CPU 闸里在跑、量满了一段的那一条照最近一段的在用核数向上取整、与最近一回的可跑线程数取大，不超过登记数（task_server.py 的 latest_running_cores；用户定
    「各个任务的负载的取值要动态的取」）：一条登记 31 线程、起了 1000 秒、一路 17.5 核——没有线程读数时记 18；最近一回可跑 25 个的记 25；可跑 40 个的收到登记的 31。
    套在假调度器上，不起服务。弄坏开关 cpu-latest-runnable-ignored 下可跑 25 个的记 18，这一格红。"""
    fake, _ = cpu_gate_scheduler(1, {"load1": 1.0, "procs_running": 1, "cpus": 32}, ages=[1000], cores=17.5)
    attempt = fake.store.running_attempts()[0]
    now = time.time()
    elapsed = now - attempt["started_at"]
    usage = fake.machines[SERVER.LOCAL].cpu_usec(attempt)
    problems = []
    for runnable, wanted in ((0, 18), (25, 25), (40, 31)):
        first = SERVER.Scheduler.latest_running_cores(fake, attempt, usage, elapsed, now, 31)   # 先算一回：记下最近一段的在用核数
        fake.running_latest_runnable[attempt["id"]] = runnable
        found = SERVER.Scheduler.latest_running_cores(fake, attempt, usage, elapsed, now, 31)
        if found != wanted:
            problems.append(f"最近一回可跑 {runnable} 个：应当记 {wanted} 线程，实际 {found}（头一回算出 {first}）")
    if problems:
        raise SelftestFailure("CPU 闸里在跑的照最近的读数记：" + "；".join(problems))
    return "在跑、一路 17.5 核的一条：没有线程读数记 18，最近一回可跑 25 个记 25，可跑 40 个收到登记的 31"


def cell_cpu_gate_running_started_by_recorded_cores(world):
    """CPU 闸里在跑却还没实测到核数的，照同一命令键近几次跑成的平均在用核数记（task_server.py 的 running_threads、unmeasured_threads、threads_by_record）：
    32 核（线 64）、负载 3，三条在跑、各登记 31 线程、都只起了 30 秒（不到 CPU_MEASURED_AFTER_SECONDS），其中两条的命令键库里记过 1、1、0.7 核（取最大记 1 线程），
    一条的命令键记过 20 核（记 20）：共 1 + 1 + 20 = 22 线程，第四条照起；起了 1000 秒而假本机量不到 CPU 的同样三条也记 22；
    在跑名单里不在了的尝试，存的那一份在下一次算时删。没有记录的在跑尝试量到之前按登记的线程数记，由 cpu-gate-unrecorded-by-registered-threads 钉着。
    弄坏开关 cpu-running-unmeasured-registered 下没实测到的一律照登记数记（共 93），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    keys = [CPU_GATE_PROBE_KEY, CPU_GATE_PROBE_KEY, CPU_GATE_HEAVY_KEY]
    problems = []
    for what, ages, cores in (("三条都只起了 30 秒", [30, 30, 30], 1.0), ("起了 1000 秒而量不到 CPU", [1000, 1000, 1000], None)):
        fake, snapshot = cpu_gate_scheduler(3, load, ages=ages, cores=cores, recorded_cores=(1.0, 1.0, 0.7), keys=keys, heavy_cores=(20.0,))
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 9, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if counted != 22 or why:
            problems.append(f"{what}：在跑的应当记 1 + 1 + 20 = 22 线程、第四条照起，实际记 {counted} 线程、「{why}」")
    fake, snapshot = cpu_gate_scheduler(3, load, ages=[30, 30, 30], keys=keys, recorded_cores=(1.0, 1.0, 0.7), heavy_cores=(20.0,))
    SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
    kept_before = sorted(fake.running_threads_cache)
    remaining = fake.store.running_attempts()[:1]
    fake.store.running_attempts = lambda: remaining
    SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
    if kept_before != [1, 2, 3] or sorted(fake.running_threads_cache) != [1]:
        problems.append(f"存的取法按尝试号：算过三条应当存 [1, 2, 3]，只剩尝试 1 在跑之后下一次算应当只留 [1]，实际 {kept_before}、{sorted(fake.running_threads_cache)}")
    if problems:
        raise SelftestFailure("CPU 闸照记录的核数记没实测到的在跑尝试：" + "；".join(problems))
    return "起了 30 秒与量不到 CPU 的在跑三条：记过 0.9 核的各记 1、记过 20 核的记 20，共 22 线程、第四条照起；存的按尝试号、收尾的删"


def cell_cpu_gate_running_average_capped_by_registered(world):
    """实测到的在跑尝试在 CPU 闸里记的核数不超过它登记的线程数，峰值段那一支与窗口那一支同一个上界（task_server.py 的 running_threads、latest_running_cores）：
    32 核（线 64）、负载 3，三条在跑、都起了 1000 秒、一路实测 40 核（峰值段 40 核；命令里编译器、子进程占的比登记线程多）：各登记 4 线程的记 4、共 12；各登记 31 线程的记 31、共 93
    （不记 40、共 120）；不到登记数的照峰值段记（2.5 核、登记 4 线程：记 3、共 9）。
    弄坏开关 cpu-running-cores-over-registered 下峰值段那一支不限登记数（共 120、共 120），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for registered, cores, expected in ((4, 40.0, 12), (31, 40.0, 93), (4, 2.5, 9)):
        fake, _ = cpu_gate_scheduler(3, load, ages=[1000, 1000, 1000], cores=cores)
        listed = fake.store.running_attempts
        fake.store.running_attempts = lambda listed=listed, registered=registered: [dict(attempt, threads=registered) for attempt in listed()]
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
        if counted != expected:
            problems.append(f"三条各登记 {registered} 线程、峰值段 {cores:g} 核：应当共记 {expected} 线程，实际 {counted}")
    if problems:
        raise SelftestFailure("CPU 闸记在跑尝试的峰值段核数不超过登记数：" + "；".join(problems))
    return "峰值段 40 核的：登记 4 线程的记 4、登记 31 线程的记 31；2.5 核、登记 4 线程的照峰值段记 3"


def cell_finish_peak_segment_cpu_unread(world):
    """收尾时累计 CPU 读不到、或接口交的不是数：峰值段核数记 None、结局照记（task_server.py 的 finish_peak_segment_cores、cpu_reading、finish_attempt）。
    直接算：读到 None、一个别的对象、一串字、NaN 的，一段都没量满过的都交 None、不抛错；量满过一段（各段最大 7.5）而这一刻读不到的交 7.5。
    整条跑：假 scope 只写内存那几份、没有 cpu.stat（服务一路读不到 CPU），交一条退 0 的：结局 succeeded、退 0，尝试的 peak_segment_cores 是 None。
    弄坏开关 peak-segment-reading-unchecked 下接口交的东西不核、照数用（别的对象、一串字抛错，NaN 交 NaN），这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.running_usage_lock = threading.Lock()
    fake.running_peak_samples, fake.running_peak_cores = {}, {}
    attempt = {"id": 1, "host": SERVER.LOCAL, "started_at": time.time() - 30}
    problems = []
    for what, value, measured, expected in (("读到 None", None, None, None), ("接口交了一个别的对象", object(), None, None), ("接口交了一串字", "2000000", None, None),
                                            ("读到 NaN", float("nan"), None, None), ("量满过一段、这一刻接口交了别的对象", object(), 7.5, 7.5)):
        fake.running_peak_samples.clear()
        fake.running_peak_cores.clear()
        if measured is not None:
            fake.running_peak_cores[attempt["id"]] = measured
        machine = types.SimpleNamespace(cpu_usec=lambda attempt, value=value: value)
        try:
            recorded = SERVER.Scheduler.finish_peak_segment_cores(fake, attempt, machine, time.time())
        except Exception as error:   # 这一格判的就是收尾这一步抛不抛：抛了照不对记下
            problems.append(f"{what}：收尾应当照常往下走、峰值段核数记 {expected}，实际抛了 {type(error).__name__}: {error}")
            continue
        if recorded != expected:
            problems.append(f"{what}：峰值段核数应当记 {expected}，实际 {recorded}")
    scope = os.path.join(world.root, "unread-cpu-scope")
    os.makedirs(scope)
    for name, text in (("memory.stat", process_memory_stat(300 * MEBIBYTE)), ("memory.peak", f"{300 * MEBIBYTE}\n")):
        with open(os.path.join(scope, name), "w", encoding="utf-8") as handle:
            handle.write(text)
    world.start_server(extra={"TASK_SERVER_FAKE_SCOPE_DIRECTORY": scope})
    done = world.finished(world.submit(["bash", "-c", "exit 0", "unread-cpu"]))
    attempts = done["attempts"]
    if done["task"].get("outcome") != "succeeded" or done["task"].get("exit_code") != 0 or len(attempts) != 1 or attempts[0].get(SERVER.PEAK_SEGMENT_CORES_COLUMN) is not None:
        problems.append(f"一路读不到 CPU、退 0 的：结局应当照记 succeeded、退 0，峰值段核数记 None，实际 {describe(done)}、"
                        f"峰值段核数 {[attempt.get(SERVER.PEAK_SEGMENT_CORES_COLUMN) for attempt in attempts]}")
    if problems:
        raise SelftestFailure("收尾时累计 CPU 读不到：" + "；".join(problems), world.server_log_lines())
    return "收尾时读不到或交的不是数：峰值段核数记 None（量满过一段的记各段最大），结局照记 succeeded"


CPU_GATE_PEAK_KEY = "selftest-cpu-gate-peak-segment"   # cpu-gate-peak-segment-cores：前 240 秒 2 核、之后 30 核的那把命令键


def cell_cpu_gate_peak_segment_cores(world):
    """CPU 闸记峰值段核数、不记整次平均（task_server.py 的 note_peak_segment、latest_running_cores、running_threads、finish_peak_segment_cores、threads_by_record；
    本机 32 核上四条各登记 31 线程的崩溃点重放同时在抢核：前半段编译、录制用核少，按整次平均每条只记几核，四条都放进来）：
    一条登记 31 线程的在跑尝试前 240 秒用 2 核、之后用 30 核，服务每 60 秒读一回累计 CPU：起了 480 秒时在跑的记 30 线程（整次平均是 16）；
    收尾时记的峰值段核数是 30（CPU 闸线程数同是 30：没有线程读数），同键下一条新任务过闸照记录记 30 线程、取法那半句写「CPU 闸线程数」。
    弄坏开关 cpu-cores-by-average 下在跑的照整次平均记 16、记录照平均在用核数（cores 16）记 16，这一格红。"""
    fake, _ = cpu_gate_scheduler(0, {"load1": 2.0, "procs_running": 3, "cpus": 32})
    clock = [float(int(time.time()))]
    attempt = {"id": 1, "host": SERVER.LOCAL, "threads": 31, "started_at": clock[0], "command_key": CPU_GATE_PEAK_KEY}

    def usage(running):
        age = clock[0] - running["started_at"]
        return 1e6 * (2.0 * min(age, 240.0) + 30.0 * max(age - 240.0, 0.0))

    fake.store.running_attempts = lambda: [dict(attempt)]
    fake.machines[SERVER.LOCAL].cpu_usec = usage
    counted = None
    for age in range(0, 481, 60):
        clock[0] = attempt["started_at"] + age
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, clock[0])
    recorded = SERVER.Scheduler.finish_peak_segment_cores(fake, attempt, fake.machines[SERVER.LOCAL], clock[0])
    average = usage(attempt) / 1e6 / 480.0
    gate = SERVER.Scheduler.finish_cpu_gate_threads(fake, attempt, recorded)
    rows = [({"outcome": "succeeded", "command_key": CPU_GATE_PEAK_KEY, "cores": average, SERVER.PEAK_SEGMENT_CORES_COLUMN: recorded, SERVER.CPU_GATE_THREADS_COLUMN: gate}, True)]
    fake.store.select = lambda table, where="1", parameters=(), order="id": list(rows) if tuple(parameters) == (CPU_GATE_PEAK_KEY,) else []
    wanted, how = SERVER.Scheduler.admission_threads(fake, {"id": 2, "command_key": CPU_GATE_PEAK_KEY})
    problems = []
    if counted != 30:
        problems.append(f"起了 480 秒（前 240 秒 2 核、之后 30 核，整次平均 {average:g}）：在跑的应当记 30 线程，实际 {counted}")
    if recorded != 30.0 or gate != 30.0 or wanted != 30 or "CPU 闸线程数" not in how:
        problems.append(f"收尾记的峰值段核数与 CPU 闸线程数应当都是 30、同键新任务照它记 30 线程（「CPU 闸线程数」），实际记 {recorded}、{gate}、新任务 {wanted} 线程「{how}」")
    if problems:
        raise SelftestFailure("CPU 闸记峰值段核数：" + "；".join(problems))
    return f"前 240 秒 2 核、之后 30 核：在跑的记 30（整次平均 {average:g}）；收尾记 30，同键新任务照记录 30 线程"


def cell_cpu_gate_runnable_threads(world):
    """CPU 闸连同可跑线程数记（task_server.py 的 note_threads、peak_segment_running_cores、finish_cpu_gate_threads、threads_by_record；
    五条各要 31 核的挤在一台上各只拿到七到十核，照在用核数记，下一条同键的照十核放进来）：32 核、负载 3，一条登记 31 线程的在跑尝试起了 1000 秒、一路在用 cores 核，
    服务读到一回线程读数：在用 8 核、可跑线程 31 的在跑记 31、收尾记 cpu_gate_threads 31、同键新任务过闸照记录记 31；在用 3 核、可跑线程 2 的三样都是 3；
    读不到线程读数的照在用核数，三样都是 8。
    弄坏开关 cpu-runnable-threads-unread 下只看在用核数（头一种三样都是 8），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for what, cores, runnable, wanted in (("在用 8 核、可跑线程 31", 8.0, 31, 31), ("在用 3 核、可跑线程 2", 3.0, 2, 3), ("读不到可跑线程数", 8.0, None, 8)):
        fake, _ = cpu_gate_scheduler(1, load, ages=[1000], cores=cores, keys=[CPU_GATE_PEAK_KEY])
        fake.machines[SERVER.LOCAL].thread_reading = lambda attempt, runnable=runnable: None if runnable is None else SERVER.ThreadReading(1, runnable, runnable, runnable, "selftest")
        attempt = fake.store.running_attempts()[0]
        now = time.time()
        SERVER.Scheduler.note_threads(fake, attempt)
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, now)
        recorded = SERVER.Scheduler.finish_cpu_gate_threads(fake, attempt, SERVER.Scheduler.finish_peak_segment_cores(fake, attempt, fake.machines[SERVER.LOCAL], now))
        rows = [({"outcome": "succeeded", "command_key": CPU_GATE_PEAK_KEY, SERVER.CPU_GATE_THREADS_COLUMN: recorded}, True)]
        fake.store.select = lambda table, where="1", parameters=(), order="id", rows=rows: list(rows) if tuple(parameters) == (CPU_GATE_PEAK_KEY,) else []
        admitted, how = SERVER.Scheduler.admission_threads(fake, {"id": 9, "command_key": CPU_GATE_PEAK_KEY})
        if counted != wanted or recorded is None or round(recorded) != wanted or admitted != wanted:
            problems.append(f"{what}：在跑的、收尾记的与同键新任务都应当是 {wanted}，实际在跑记 {counted}、收尾记 {recorded}、新任务 {admitted}（{how}）")
    if problems:
        raise SelftestFailure("CPU 闸连同可跑线程数记：" + "；".join(problems))
    return "在用 8 核、可跑线程 31 记 31（收尾与同键新任务同是 31）；在用 3 核、可跑线程 2 记 3；读不到线程读数照在用核数 8"


THREADS_OVER_REGISTERED_BINARY = "selftest-test-binary"   # threads-over-registered-reported 里占线程最多的那个进程名


def cell_threads_over_registered_reported(world):
    """线程总数远超登记数推一回提醒（task_server.py 的 note_threads、threads_over_registered_items；文件头 ①「要处置的提醒」的 threads-over-registered；
    量到过一个测试进程开了 347 个线程、占 1780% CPU，登记 31）：一条登记 31 线程的在跑尝试，服务逐轮读线程读数（采样号、可跑的、总数）：
    连着三轮 347 个线程、可跑的 40 个，之后掉回 10 个：推一项 threads-over-registered，依据写 347、登记数 31 与占线程最多的进程名；
    同一个采样号读了三回：只算一轮，不推；连着五轮 347 个线程而可跑的只有 5 个（编译器、链接器开一堆在睡的线程）：不推；超两轮、掉一轮、再超两轮：不推。
    连着五轮 392 个线程、可跑的 39 个而占线程最多的进程只有 34 个线程（编译时一堆 rustc）：不推。
    弄坏开关 threads-over-registered-silent 下头一种不推，threads-over-registered-idle-counted 下可跑的只有 5 个的那一种也推，
    threads-over-registered-many-processes-counted 下占线程最多的进程只有 34 个线程的那一种也推，这一格红。"""
    def busy(sample, runnable=40):
        return SERVER.ThreadReading(sample, runnable, 347, 300, THREADS_OVER_REGISTERED_BINARY)

    def calm(sample):
        return SERVER.ThreadReading(sample, 2, 10, 10, THREADS_OVER_REGISTERED_BINARY)

    cases = (("连着三轮 347 个线程、可跑的 40 个，之后掉回 10 个", [busy(1), busy(2), busy(3), calm(4), calm(5)], True),
             ("同一个采样号读了三回", [busy(1), busy(1), busy(1)], False),
             ("连着五轮 347 个线程、可跑的只有 5 个", [busy(sample, runnable=5) for sample in range(1, 6)], False),
             ("连着五轮 392 个线程、可跑的 39 个、占线程最多的进程只有 34 个线程（编译）", [SERVER.ThreadReading(sample, 39, 392, 34, "cargo") for sample in range(1, 6)], False),
             ("超两轮、掉一轮、再超两轮", [busy(1), busy(2), calm(3), busy(4), busy(5)], False))
    problems = []
    for what, readings, pushed in cases:
        fake, _ = cpu_gate_scheduler(0, {"load1": 2.0, "procs_running": 3, "cpus": 32})
        attempt = {"id": 1, "host": SERVER.LOCAL, "threads": 31, "state": "running", "number": 1}
        queue = list(readings)
        fake.machines[SERVER.LOCAL].thread_reading = lambda running, queue=queue: queue.pop(0) if queue else None
        for _ in readings:
            SERVER.Scheduler.note_threads(fake, attempt)
        items = SERVER.Scheduler.threads_over_registered_items(fake, {"id": 7}, [attempt])
        basis = items[0]["basis"] if items else ""
        if pushed and (len(items) != 1 or items[0]["kind"] != "threads-over-registered" or items[0]["threshold"] != 1
                       or not all(text in basis for text in ("347", "登记数 31", THREADS_OVER_REGISTERED_BINARY))):
            problems.append(f"{what}：应当推一项 threads-over-registered、提醒门槛 1、依据写 347、登记数 31 与 {THREADS_OVER_REGISTERED_BINARY}，实际 {items}")
        if not pushed and items:
            problems.append(f"{what}：不该推，实际 {items}")
    if problems:
        raise SelftestFailure("线程总数远超登记数：" + "；".join(problems))
    return "连着三轮 347 个线程、可跑的 40 个推一回；同一个采样号、可跑的只有 5 个、中间掉过一轮的都不推"


def peer_threads_script(runnable):
    """threads-over-registered-peer-reported 的第二台那条命令：12 轮、每轮 0.5 秒，每轮打一行 MULTI_HOST_RUN_PEER_THREADS（线程总数 100000、可跑的 runnable）与一行 CPU。"""
    return ('for i in $(seq 12); do echo "MULTI_HOST_RUN_PEER_THREADS poll=$i runnable=' + str(runnable) + ' total=100000 top=90000:selftest-peer-binary read=yes"; '
            'echo "MULTI_HOST_RUN_PEER_CPU poll=$i usage_usec=$((i * 1000000)) read=yes"; sleep 0.5; done')


def cell_threads_over_registered_peer_reported(world):
    """第二台的线程读数（task_server.py 的 PEER_THREADS_LINE、log_content 的 peer_threads、PeerMachine.thread_reading）：两条派到第二台、各跑 6 秒，每轮打一行
    MULTI_HOST_RUN_PEER_THREADS total=100000：可跑的 50000 的那一条 wait 打一行 kind=threads-over-registered、依据写 100000 与 selftest-peer-binary、退 ATTENTION_EXIT，
    再起 wait 跑完；可跑的 0 的那一条不推、跑完。弄坏开关 threads-over-registered-silent 下头一条不推，这一格红。"""
    world.resources(local=(8, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.seed_peak("selftest-peer-threads", 10 * GIBIBYTE)
    world.start_server()
    busy = world.submit(["bash", "-c", peer_threads_script(50000), "busy", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-peer-threads"}, cwd=world.repo)
    fields = attention_of_kind(world, busy, "threads-over-registered")
    problems = []
    if not fields or "100000" not in fields.get("basis", "") or "selftest-peer-binary" not in fields.get("basis", ""):
        problems.append(f"可跑的 50000 的那一条应当推 threads-over-registered、依据写 100000 与 selftest-peer-binary，实际 {fields}")
    done = world.finished(busy, timeout=60)
    host = done["attempts"][0].get("host") if done["attempts"] else None
    if host != "peer" or done["task"].get("outcome") != "succeeded":
        problems.append(f"可跑的 50000 的那一条应当在第二台跑完，实际在 {host}、{describe(done)}")
    calm = world.submit(["bash", "-c", peer_threads_script(0), "calm", "fake-peer-run"], extra={lib.KEY_VARIABLE: "selftest-peer-threads"}, cwd=world.repo)
    completed = world.client(["wait", str(calm)], timeout=60)
    pushed = [line for line in attention_lines(completed) if attention_fields(line).get("kind") == "threads-over-registered"]
    if pushed or completed.returncode != 0:
        problems.append(f"可跑的 0 的那一条不该推、应当跑完退 0，实际退 {completed.returncode}、推了 {pushed}")
    if problems:
        raise SelftestFailure("第二台的线程读数：" + "；".join(problems), world.server_log_lines())
    return "第二台每轮 100000 个线程：可跑的 50000 的推 threads-over-registered（依据写 100000 与进程名）、跑完；可跑的 0 的不推、跑完"


def cell_cpu_gate_line_twice_cores(world):
    """CPU 闸的线是核数 × 倍数（task_server.py 的 cpu_judgement；小世界的倍数是 2，由 TASK_SERVER_SELFTEST_CPU_OVERSUBSCRIPTION 给）：32 核（线 64）、负载 3，在跑两条各记 31 线程（起了 100 秒、照命令键的记录 31），
    第三条 31 线程的排着，原因以「CPU：」起头、写「32 核 × 2 = 64」；在跑一条时 31 + 31 = 62 照起。
    弄坏开关 cpu-oversubscription-three 下线是 192（32 × CPU_OVERSUBSCRIPTION_LOOSE），在跑两条时第三条也照起，这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    problems = []
    for count, blocked in ((1, False), (2, True)):
        fake, snapshot = cpu_gate_scheduler(count, load, recorded_cores=(31.0,), keys=[CPU_GATE_PROBE_KEY] * count)
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 9, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if blocked and (not why.startswith("CPU：") or "32 核 × 2 = 64" not in why):
            problems.append(f"在跑两条各 31 线程：第三条 31 线程应当排着、原因以「CPU：」起头并写「32 核 × 2 = 64」，实际「{why}」")
        if not blocked and why:
            problems.append(f"在跑一条 31 线程：再起一条 31 线程（共 62）应当照起，实际「{why}」")
    if SERVER.CPU_OVERSUBSCRIPTION != 3.4375:
        problems.append(f"真服务的倍数应当是 3.4375（32 核的台线是 110，用户定的），实际 {SERVER.CPU_OVERSUBSCRIPTION}")
    if problems:
        raise SelftestFailure("CPU 闸的线是核数 × 倍数：" + "；".join(problems))
    return "小世界照 2 倍（32 核线 64）：在跑一条再起一条照起（62）；在跑两条再起一条排着、原因写「32 核 × 2 = 64」；真服务的倍数是 3.4375（32 核线 110）"


def query_plan_text(store, query, parameters):
    """一条查询在这份库上的查询计划（EXPLAIN QUERY PLAN 各行的说明接起来）。"""
    return " ".join(str(row[-1]) for row in store.connection().execute("EXPLAIN QUERY PLAN " + query, parameters).fetchall())


def cell_lookup_indexes_used(world):
    """查得勤的几种走索引（task_server.py 的 Store.LOOKUP_INDEXES；loop 审核发现 5）：按 task_id 取尝试与子任务、按命令键取结束了的尝试、按会话取任务，
    查询计划里各用 attempts_task、tasks_task、attempts_key_state、tasks_session，不整表扫。
    弄坏开关 lookup-indexes-skipped 下不建，这一格红。"""
    store = SERVER.Store(os.path.join(world.root, "lookup-indexes.sqlite3"), bytes(32), dict(os.environ))
    problems = []
    for query, parameters, index in (("SELECT id FROM attempts WHERE task_id = ? ORDER BY id", (1,), "attempts_task"),
                                     ("SELECT id FROM tasks WHERE task_id = ? ORDER BY id", (1,), "tasks_task"),
                                     ("SELECT id FROM attempts WHERE command_key = ? AND state = 'ended' ORDER BY id DESC LIMIT 200", ("selftest",), "attempts_key_state"),
                                     ("SELECT id FROM tasks WHERE session = ? ORDER BY id", ("selftest",), "tasks_session")):
        plan = query_plan_text(store, query, parameters)
        if index not in plan:
            problems.append(f"「{query}」应当用 {index}，实际查询计划「{plan}」")
    if problems:
        raise SelftestFailure("查得勤的几种走索引：" + "；".join(problems))
    return "按 task_id、命令键、会话取的四种都走索引"


def cell_running_attempts_read_once(world):
    """在跑的尝试库没变就不重读（task_server.py 的 Store.running_attempts；loop 审核发现 8）：连读两次只查一次库；本进程写过一行之后、别的连接提交过之后各重读一次；
    改了交出来的那一份不染到下一次。弄坏开关 running-attempts-reread 下每次都查，这一格红。"""
    path = os.path.join(world.root, "running-read-once.sqlite3")
    store = SERVER.Store(path, bytes(32), dict(os.environ))
    attempt = store.insert("attempts", {"state": "running", "session": "", "command_key": "", "task_id": 0, "host": SERVER.LOCAL})
    reads = []
    original = store.select

    def counted(*arguments, **options):
        reads.append(arguments[0] if arguments else "")
        return original(*arguments, **options)
    store.select = counted
    problems = []
    first = store.running_attempts()
    store.running_attempts()
    if len(reads) != 1:
        problems.append(f"库没变连读两次：应当只查 1 次，实际 {len(reads)} 次")
    first[0]["host"] = SERVER.PEER
    if store.running_attempts()[0].get("host") != SERVER.LOCAL:
        problems.append("改了交出来的那一份：下一次交的不应当跟着变")
    store.update("attempts", dict(attempt, note="自证"))
    store.running_attempts()
    if len(reads) != 2:
        problems.append(f"本进程写过一行之后：应当重读（共 2 次），实际 {len(reads)} 次")
    with contextlib.closing(sqlite3.connect(path, timeout=5)) as connection:
        connection.execute("INSERT OR REPLACE INTO meta(name, value) VALUES('selftest-poke', '1')")
        connection.commit()
    store.running_attempts()
    if len(reads) != 3:
        problems.append(f"别的连接提交过之后：应当重读（共 3 次），实际 {len(reads)} 次")
    if problems:
        raise SelftestFailure("在跑的尝试库没变就不重读：" + "；".join(problems))
    return "库没变只读一次；本进程写过、别的连接提交过各重读一次"


def cell_budget_source_only_when_reported(world):
    """预算那一判不记提醒时不为「记账从哪来」扫库（task_server.py 的 budget_check；loop 审核发现 1）：记 4G、本机可用上限 48G、整片界 40G：
    不记 oversized-budget、一次都不算记账来源。弄坏开关 budget-source-every-pass 下每一轮都算，这一格红。"""
    calls = []
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.host_dispatch = types.SimpleNamespace(human=lambda size: f"{size / GIBIBYTE:.1f} GiB")
    fake.prediction_of = lambda task: types.SimpleNamespace(memory_predicted=4 * GIBIBYTE)
    fake.candidate_hosts = lambda task, snapshot: [SERVER.LOCAL]
    fake.host_limit = lambda prediction, host, snapshot: (48 * GIBIBYTE, "")
    fake.booked_at_cap = lambda prediction, host, snapshot: types.SimpleNamespace(memory_booked=4 * GIBIBYTE)
    fake.whole_threshold = lambda host, snapshot: 40 * GIBIBYTE
    fake.budget_notices = {}
    fake.budget_source = lambda task, prediction: calls.append(task["id"]) or "自证"
    snapshot = types.SimpleNamespace(reports={SERVER.LOCAL: {}}, reachable={SERVER.LOCAL: True, SERVER.PEER: True})
    SERVER.Scheduler.budget_check(fake, {"id": 7}, types.SimpleNamespace(snapshot=lambda: snapshot))
    if fake.budget_notices or calls:
        raise SelftestFailure(f"记 4G、可用上限 48G：应当不记提醒、不算记账来源，实际提醒 {fake.budget_notices}、算了 {len(calls)} 次")
    return "不记提醒的一条不算记账来源"


def cell_repass_cleared_when_idle(world):
    """队空的一轮也清 repass_soon（task_server.py 的 place_queued_into；loop 审核发现 12）：上一轮第二台起满记了 repass_soon，这一轮队空：清掉，主循环照 TICK_SECONDS 睡。
    弄坏开关 repass-kept-when-idle 下照留，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.repass_soon = True
    fake.pause_in_force = lambda: None
    fake.store = types.SimpleNamespace(select=lambda *arguments, **options: [])
    fake.selftest_read = lambda line: None
    SERVER.Scheduler.place_queued_into(fake, {}, {})
    if fake.repass_soon:
        raise SelftestFailure("队空的一轮：repass_soon 应当清掉，实际还留着")
    return "队空的一轮清掉 repass_soon"


def peer_probe_resources():
    """peer-probe-off-scheduler 的假 Resources：只有第二台一台，探测照 outcome 交（每次记进 probes）；不读多机配置、不判盘余量。→ (Resources, probes, outcome)。"""
    probes, outcome = [], {"report": {"memory": {}}, "why": ""}
    resources = object.__new__(SERVER.Resources)
    resources.environment = dict(os.environ)
    resources.fake_path, resources.cached, resources.peer = "", None, None
    resources.peer_lock, resources.peer_reading, resources.peer_failed = threading.Lock(), None, None
    resources.peer_refresh_seconds = 0.1
    resources.refresh_peer = lambda: None
    resources.disk_problem = lambda machine, report, fake=None: ""
    resources.machines = {SERVER.PEER: types.SimpleNamespace(
        name=SERVER.PEER, label="第二台", always_reachable=False, reachable=lambda found: (True, ""), configured_cap=lambda found: None,
        probe=lambda found: (probes.append(True), (outcome["report"], outcome["why"]))[1])}
    return resources, probes, outcome


def cell_peer_probe_off_scheduler(world):
    """第二台的余量不在排队线程里读（task_server.py 的 Resources.peer_probe_round、peer_snapshot_entry；loop 审核发现 3）：探测线程读了一回之后，snapshot 取那一份、不再连；
    第二台读不成的那一回记下，PEER_PROBE_FAILURE_HOLD_SECONDS 之内探测线程不再去连，snapshot 当它此刻用不了、原因写不再去连。
    弄坏开关 peer-probe-in-scheduler 下 snapshot 在调用方的线程里现连，这一格红。"""
    resources, probes, outcome = peer_probe_resources()
    problems = []
    resources.peer_probe_round()
    snapshot = resources.snapshot()
    if len(probes) != 1 or SERVER.PEER not in snapshot.reports:
        problems.append(f"探测线程读过一回：snapshot 应当取那一份、不再连，实际连了 {len(probes)} 次、报告里的台 {sorted(snapshot.reports)}")
    outcome["report"], outcome["why"] = None, "自证：第二台卡住"
    resources.peer_probe_round()
    resources.peer_probe_round()
    resources.cached = None
    snapshot = resources.snapshot()
    if len(probes) != 2 or SERVER.PEER in snapshot.reports or "不再去连" not in str(snapshot.down.get(SERVER.PEER)):
        problems.append(f"读不成之后：探测线程应当只再连 1 回、snapshot 当第二台用不了（原因写不再去连），实际共连了 {len(probes)} 次、"
                        f"报告里的台 {sorted(snapshot.reports)}、原因「{snapshot.down.get(SERVER.PEER)}」")
    if problems:
        raise SelftestFailure("第二台的余量不在排队线程里读：" + "；".join(problems))
    return "snapshot 取探测线程那一份；读不成的那一回之后一段时间不再去连"


PROMOTE_WAIT_SECONDS = 20   # promote-skips-cpu-gate：提前之后等它收尾的安静时限（弄坏开关 promote-cpu-gate-applied 下它一直排着，等满就判红）


def cell_promote_skips_cpu_gate(world):
    """task promote 提前（task_server.py 的 promote、路由 /tasks/{task_id}/promote、cpu_judgement）：本机内存够、负载 200（同 cpu-gate-holds-busy-host），命令键记过 31 核的任务排着、
    排队原因写「CPU：」；别的会话提前它：客户端退非 0、说不是这个会话提交的，任务照旧在排；提交它的会话提前：退 0、回话写「提前到队头」；负载不降，它照起、跑完（提前过的不过 CPU 闸）。
    弄坏开关 promote-session-unchecked 下别的会话也提前得了，promote-cpu-gate-applied 下提前过的照过 CPU 闸、一直排着，这一格红。"""
    world.resources(local=(50, 50), local_cpu={"load1": 200.0, "procs_running": 150, "cpus": 32})
    seed_busy_cores(world, CPU_GATE_BUSY_KEY)
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "promote-cpu-gate"], extra={lib.KEY_VARIABLE: CPU_GATE_BUSY_KEY})
    world.next_pass()
    world.next_pass()
    waiting = world.summary(task_id)
    reason = waiting.get("waiting_for") or ""
    if waiting["attempts"] or "CPU：" not in reason:
        raise SelftestFailure(f"负载 200 的本机上应当排着、排队原因写「CPU：」，实际尝试 {len(waiting['attempts'])} 次、排队原因「{reason}」", world.server_log_lines())
    problems = []
    foreign = world.client(["promote", str(task_id)], session=SESSION_B)
    after_foreign = world.summary(task_id)
    if foreign.returncode == 0 or "不是这个会话提交的" not in foreign.stdout + foreign.stderr or after_foreign["task"].get("state") != "queued":
        problems.append(f"别的会话提前：应当退非 0、说不是这个会话提交的、任务照旧在排，实际退 {foreign.returncode}、「{(foreign.stdout + foreign.stderr).strip()[-200:]}」、"
                        f"状态 {after_foreign['task'].get('state')}")
    own = world.client(["promote", str(task_id)])
    if own.returncode != 0 or "提前到队头" not in own.stdout:
        problems.append(f"提交它的会话提前：应当退 0、回话写「提前到队头」，实际退 {own.returncode}、「{(own.stdout + own.stderr).strip()[-200:]}」")
    try:
        done = world.finished(task_id, timeout=PROMOTE_WAIT_SECONDS)
    except SelftestFailure as failure:
        done = None
        problems.append(f"提前之后负载照旧 200：应当不过 CPU 闸、照起、跑完，实际安静 {PROMOTE_WAIT_SECONDS} 秒没收尾（{str(failure)[:200]}）")
    if done is not None and done["task"].get("outcome") != "succeeded":
        problems.append(f"提前之后应当跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("task promote 提前：" + "；".join(problems), world.server_log_lines())
    return f"负载 200 排着（「{reason[:60]}」）；别的会话提前被拒；提交它的会话提前之后不过 CPU 闸、照起、跑完"


def cell_promote_queue_order(world):
    """排队的先后（task_server.py 的 queue_order）：三条在排，提交 7200 秒的（排满 3600 秒的高优先）、提交 60 秒的、提交 30 秒而 task promote 提前过的：先后是提前过的、
    高优先的、其余的。套在假调度器上，不起服务。弄坏开关 promote-queue-order-ignored 下不看提前过没有（高优先的、其余的、提前过的），这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.aged_seconds = 3600.0
    now = time.time()
    queued = [({"id": 1, "submitted_at": now - 7200}, True), ({"id": 2, "submitted_at": now - 60}, True), ({"id": 3, "submitted_at": now - 30, "promoted": {"at": now}}, True)]
    order = [row[0]["id"] for row in SERVER.Scheduler.queue_order(fake, queued, now)]
    if order != [3, 1, 2]:
        raise SelftestFailure(f"排队的先后应当是提前过的 3、高优先的 1、其余的 2，实际 {order}")
    return "提前过的排最前，再是排满时限的高优先，其余照原来的次序"


def cell_progress_line_names_progress(world):
    """在跑进度里一次在跑的尝试那一行（task_server.py 的 progress_attempt_line）：起跑以来 90 分钟，日志 1800 秒没涨而 120 秒前读到子进程写着的文件在涨：
    那一行写起跑以来多久、写读到的进展与多少秒前读的、写日志末行。套在假调度器上（running_progress 换成定值），不起服务。
    弄坏开关 progress-line-progress-unnamed 下不写读到的进展，这一格红。"""
    log = os.path.join(world.root, "progress-line.log")
    with open(log, "w", encoding="utf-8") as handle:
        handle.write("头一行\n末一行在这里\n")
    note = "子进程写着的 /selftest/record.log 从 10 涨到 20 字节"
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.running_progress = lambda task, attempt, now: {"elapsed_seconds": 5400.0, "cores": 0.0, "log_quiet_seconds": 1800.0, "progress_note": note, "progress_age_seconds": 120.0}
    line = SERVER.Scheduler.progress_attempt_line(fake, {"id": 17}, {"id": 1, "task_id": 17, "host": SERVER.PEER, "log": log}, time.time())
    wanted = ["起跑以来 90 分钟", note, "120 秒前读到", "末一行在这里"]
    missing = [piece for piece in wanted if piece not in line]
    if missing:
        raise SelftestFailure(f"在跑进度那一行应当写 {wanted}，缺 {missing}，实际「{line}」")
    return f"那一行：{line[:160]}"


def cell_attempt_progress_readings(world):
    """日志不涨时的进展读数（task_server_lib.py 的 attempt_progress_readings、attempt_progress_probe_text）：假 /proc 里进程 4100（树根）→ 4101，另有不在树里的 4999；
    4100 只写着顶层日志（不算），4101 只写着 record.log、只读着 input.txt，4999 写着 outside.log：读数是 record.log 一份（大小照真文件）、
    进程树 2 个。送到第二台的那一份（源码与请求各 base64 一份的 bash 命令）在本机 bash 起，读数逐项相同。
    弄坏开关 open-file-write-mode-unchecked 下只读打开的 input.txt 也算写着的，这一格红。"""
    proc, files = os.path.join(world.root, "progress-proc"), os.path.join(world.root, "progress-files")
    os.makedirs(files, exist_ok=True)
    record, top, source, outside = (os.path.join(files, name) for name in ("record.log", "attempt.log", "input.txt", "outside.log"))
    for path, size in ((record, 10), (top, 20), (source, 30), (outside, 40)):
        with open(path, "w", encoding="utf-8") as handle:
            handle.write("x" * size)
    write_only, read_only = "0100001", "0100000"
    layout = {4100: (1, {"1": (top, write_only)}), 4101: (4100, {"1": (record, write_only), "3": (source, read_only)}), 4999: (1, {"1": (outside, write_only)})}
    for process, (parent, descriptors) in layout.items():
        for part in ("fd", "fdinfo"):
            os.makedirs(os.path.join(proc, str(process), part), exist_ok=True)
        with open(os.path.join(proc, str(process), "stat"), "w", encoding="utf-8") as handle:
            handle.write(f"{process} (fake) S {parent} 0 0\n")
        for descriptor, (target, flags) in descriptors.items():
            os.symlink(target, os.path.join(proc, str(process), "fd", descriptor))
            with open(os.path.join(proc, str(process), "fdinfo", descriptor), "w", encoding="utf-8") as handle:
                handle.write(f"pos:\t0\nflags:\t{flags}\nmnt_id:\t1\n")
    request = {"root_process_ids": [4100], "excluded_files": [top], "proc_root": proc, "file_limit": 16,
               "write_mode_unchecked": lib.broken("open-file-write-mode-unchecked")}
    direct = lib.attempt_progress_readings(request)
    problems = []
    if direct.get("files") != {os.path.realpath(record): 10} or direct.get("processes") != 2:
        problems.append(f"读数应当是写着的 record.log 一份（10 字节）、进程树 2 个，实际 {direct}")
    completed = subprocess.run(["bash", "-c", lib.attempt_progress_probe_text(request)], capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=60)
    shipped = lib.attempt_progress_probe_reply(completed.stdout)
    if shipped != direct:
        problems.append(f"送到第二台的那一份在本机起应当读数逐项相同，实际退 {completed.returncode}、读数 {shipped}、标准错误「{completed.stderr.strip()[-200:]}」")
    if problems:
        raise SelftestFailure("日志不涨时的进展读数：" + "；".join(problems))
    return "写着的 record.log 一份、进程树 2 个；送到第二台的那一份读数相同"


def progress_quiet_scheduler(readings):
    """quiet-log-busy-spared-by-progress 的假调度器：尝试 7（任务 70）的日志 5000 秒没涨、最近一个窗口因 CPU 在涨没停它（log_growth 的缘由 SPARED_CPU_BUSY），
    卡住阈值 600 秒、日志长期不涨的提醒门槛 1800 秒；本机的进展读数依次交 readings 里的各份（读到最后一份之后一直交它）。→ (假调度器, 读数开始的那一刻（monotonic）)。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.stall_seconds, fake.quiet_attention_seconds = 600.0, 1800.0
    now = time.monotonic()
    fake.log_growth = {7: (100, now - 5000, now - 600, 1.0, SERVER.SPARED_CPU_BUSY)}
    fake.progress_read_at, fake.progress_files, fake.progress_noted = {}, {}, {}
    pending = list(readings)
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(attempt_progress_readings=lambda attempt: (pending.pop(0) if len(pending) > 1 else pending[0], ""))}
    return fake, now


def cell_quiet_log_busy_spared_by_progress(world):
    """日志不涨而有进展的不报 quiet-log-busy（task_server.py 的 note_attempt_progress、quiet_log_items；任务 17334 顶层日志 66200 秒没涨而测试进程写着的 record.log 在涨）：
    尝试 7 日志 5000 秒没涨、CPU 在涨。子进程写着的 record.log 隔 400 秒读两回从 100 涨到 250 字节：记下进展（写出那份文件与两个大小）、不报；
    对照：record.log 两回都是 100 字节：报一项 quiet-log-busy。套在假调度器上，不起服务。
    弄坏开关 open-file-growth-ignored 下不看文件（头一种照报），quiet-log-busy-progress-ignored 下不看读到的进展（照报），这一格红。"""
    attempt = {"id": 7, "task_id": 70, "number": 1, "host": SERVER.LOCAL, "state": "running"}
    record = "/selftest-run/record.log"
    problems = []
    for what, readings, spared, wanted in (
            ("record.log 在涨", [{"files": {record: 100}}, {"files": {record: 250}}], True, [record, "100", "250"]),
            ("都没动", [{"files": {record: 100}}, {"files": {record: 100}}], False, [])):
        fake, now = progress_quiet_scheduler(readings)
        SERVER.Scheduler.note_attempt_progress(fake, attempt, now - 400)
        SERVER.Scheduler.note_attempt_progress(fake, attempt, now)
        noted = fake.progress_noted.get(7)
        items = SERVER.Scheduler.quiet_log_items(fake, {"id": 70}, [attempt])
        kinds = [item.get("kind") for item in items]
        if spared and (noted is None or any(piece not in noted[1] for piece in wanted) or kinds):
            problems.append(f"{what}：应当记下进展（写出 {wanted}）、不报 quiet-log-busy，实际记下「{noted[1] if noted else '没有'}」、报了 {kinds}")
        if not spared and (noted is not None or kinds != ["quiet-log-busy"]):
            problems.append(f"{what}：不该记进展、应当报一项 quiet-log-busy，实际记下「{noted[1] if noted else '没有'}」、报了 {kinds}")
    if problems:
        raise SelftestFailure("日志不涨而有进展：" + "；".join(problems))
    return "写着的 record.log 在涨的记下进展、不报 quiet-log-busy；没动的照报"


def cell_peer_ssh_multiplexed(world):
    """服务起的 ssh 一律复用连接（task_server_lib.py 的 enable_ssh_multiplexing、ssh_multiplex_options、peer_shell_timed、ssh_multiplexed_environment、ssh_shim_text；
    第二台 sshd 的 MaxStartups 被一条条新连接打满过）：在一个短的临时目录当服务状态目录起复用，控制目录是它下面的 ssh/：
    服务自己起的那一路（peer_shell，ssh 换成记参数的假 ssh）参数里带 ControlMaster=auto、ControlPath=<控制目录>/%C、ControlPersist=<秒>，主机名是调用方给的多机配置第二台那一项；
    host-dispatch.py 那一路（ssh_multiplexed_environment 之后的环境里 bash 起 PATH 上的 ssh）：PATH 头一项是控制目录下的 bin，复用层起 PATH 上下一个 ssh（记参数的假的），参数是这三项
    接原来的参数；复用层正文里没有写死的主机名。控制目录放不下 socket 路径的判出来（不复用），环境里没记控制目录的（control 这类不是服务起的）不接。
    弄坏开关 ssh-unmultiplexed 下两路都不接这三项，这一格红。"""
    # 控制 socket 路径有上限：状态目录要短（系统临时目录接上 tsm-<8 个字符>/ssh/<%C>.<临时后缀> 放不下的用 /tmp）
    base = tempfile.gettempdir()
    if len(os.fsencode(base)) + len("/tsm-12345678/") + len(lib.SSH_CONTROL_DIRECTORY_NAME) + 1 + lib.SSH_CONTROL_HASH_CHARACTERS + lib.SSH_CONTROL_TEMPORARY_SUFFIX_CHARACTERS \
            > lib.SOCKET_PATH_MAXIMUM_BYTES:
        base = FALLBACK_TEMPORARY_BASE
    state = tempfile.mkdtemp(prefix="tsm-", dir=base)
    problems = []
    try:
        environment = dict(os.environ)
        problem = lib.enable_ssh_multiplexing(environment, state)
        control = os.path.join(state, lib.SSH_CONTROL_DIRECTORY_NAME)
        expected = ["-o", "ControlMaster=auto", "-o", f"ControlPath={control}/%C", "-o", f"ControlPersist={lib.SSH_CONTROL_PERSIST_SECONDS}"]
        if problem or environment.get(lib.SSH_CONTROL_VARIABLE) != control:
            problems.append(f"短的状态目录应当起得了复用、记下控制目录 {control}，实际「{problem}」、记的 {environment.get(lib.SSH_CONTROL_VARIABLE)}")
        recorder = '#!/usr/bin/env bash\nprintf "%s\\n" "$@" > "$(dirname "$0")/argv"\n'
        direct_directory, next_directory = os.path.join(state, "direct"), os.path.join(state, "next")
        for directory in (direct_directory, next_directory):
            os.makedirs(directory)
            with open(os.path.join(directory, "ssh"), "w", encoding="utf-8") as handle:
                handle.write(recorder)
            os.chmod(os.path.join(directory, "ssh"), 0o755)

        def recorded(directory):
            with contextlib.suppress(OSError):
                with open(os.path.join(directory, "argv"), encoding="utf-8") as handle:
                    return handle.read().splitlines()
            return []
        lib.peer_shell(SELFTEST_PEER_ADDRESS, "true", dict(environment, TASK_SERVER_PEER_SSH=os.path.join(direct_directory, "ssh")))
        direct = recorded(direct_directory)
        if direct != [*lib.NFS_SSH_OPTIONS, *expected, SELFTEST_PEER_ADDRESS, "true"]:
            problems.append(f"服务自己起的 ssh 参数应当是 {[*lib.NFS_SSH_OPTIONS, *expected, SELFTEST_PEER_ADDRESS, 'true']}，实际 {direct}")
        merged = lib.ssh_multiplexed_environment({"PATH": os.pathsep.join([next_directory, "/usr/bin", "/bin"])}, environment)
        subprocess.run(["bash", "-c", 'ssh -o BatchMode=yes "$0" true', SELFTEST_PEER_ADDRESS], env=merged, capture_output=True, text=True, stdin=subprocess.DEVNULL, timeout=30)
        relayed = recorded(next_directory)
        if merged.get("PATH", "").split(os.pathsep)[0] != os.path.join(control, lib.SSH_SHIM_DIRECTORY_NAME) or relayed != [*expected, "-o", "BatchMode=yes", SELFTEST_PEER_ADDRESS, "true"]:
            problems.append(f"host-dispatch.py 那一路：PATH 头一项应当是 {os.path.join(control, lib.SSH_SHIM_DIRECTORY_NAME)}、下一个 ssh 收到的参数应当是三项接原来的参数，"
                            f"实际 PATH「{merged.get('PATH')}」、参数 {relayed}")
        with contextlib.suppress(OSError):
            with open(os.path.join(control, lib.SSH_SHIM_DIRECTORY_NAME, "ssh"), encoding="utf-8") as handle:
                if SELFTEST_PEER_ADDRESS in handle.read():
                    problems.append("复用层正文里不该写死主机名")
        if not lib.ssh_control_path_problem(os.path.join("/" + "x" * 80, lib.SSH_CONTROL_DIRECTORY_NAME)):
            problems.append("80 字节的状态目录接上 %C 与临时后缀放不下 socket 路径，应当判出来，实际没判")
        if lib.ssh_multiplex_options({}):
            problems.append(f"环境里没记控制目录的不该接复用的三项，实际 {lib.ssh_multiplex_options({})}")
    finally:
        shutil.rmtree(state, ignore_errors=True)
    if problems:
        raise SelftestFailure("服务起的 ssh 复用连接：" + "；".join(problems))
    return f"两路 ssh 都带 ControlMaster=auto、ControlPath=<状态目录>/ssh/%C、ControlPersist={lib.SSH_CONTROL_PERSIST_SECONDS}；主机名照调用方给的"


# ── 收尾不在排队线程里做（task_server.py 的 finish_attempt、finish_off_loop）──
PEER_RECOVER_HOLD_SECONDS = 20.0           # peer-cleanup-off-scheduler：第二台收尾卡着的这段里，本机那一条跑完的安静时限


def unit_unreadable_lines(world, attempt_id, presence=""):
    """这一格各次服务输出里 TASK_UNIT_UNREADABLE 起头、点名这一次尝试（attempt=<记录号>）的那几行（task_server.py 的 note_unit_unreadable）；给了 presence 的只要 presence=<它> 的。"""
    lines = []
    for number in range(1, world.server_logs + 1):
        text = read_text(os.path.join(world.root, f"server-{number}.log"))
        lines += [line for line in text.splitlines() if line.startswith("TASK_UNIT_UNREADABLE ") and f" attempt={attempt_id} " in line
                  and (not presence or f" presence={presence} " in line)]
    return lines


def wait_unit_unreadable_line(world, attempt_id, presence=""):
    """等服务日志打出点名这一次尝试的 TASK_UNIT_UNREADABLE 行（PLACEMENT_WAIT_SECONDS 的安静时限）；交此刻有的那几行（没有交空表，调用处判）。"""
    patience = world.patience(PLACEMENT_WAIT_SECONDS)
    while not unit_unreadable_lines(world, attempt_id, presence) and patience.waiting():
        time.sleep(POLL_SECONDS)
    return unit_unreadable_lines(world, attempt_id, presence)


UNIT_UNREADABLE_WAIT_SCRIPT = 'for round in $(seq 1 2000); do [[ -e "$1" ]] && exit "$2"; sleep 0.1; done; exit 9'   # 等放行文件在了再退给的码（至多 200 秒）
UNIT_UNREADABLE_HOLD_SECONDS = 3.0   # unit-unreadable-unverified-health-reported：/health 报了之后再看这么久（对账十来轮、过了 1 秒的宽限），核不出的那一次仍该照记 running


def cell_unit_unreadable_exit_file_settles(world):
    """读不到单元状态、留了 result.json 的照它的退出码收尾（task_server.py 文件头 ①「读不到单元状态」）：一条命令跑起来之后世界目录里放 unit-states-unreadable
    （unit_states 一律交 None），等服务日志打出点名它的 TASK_UNIT_UNREADABLE 行（进了读不到的那一段）再放它退 3：收尾 command-failed、退 3、只一次。
    弄坏开关 unit-unreadable-exit-file-unread 下读不到的那一段不认 result.json，命令退了、单元收掉之后照没留结果记 launcher-lost，这一格红。"""
    world.start_server()
    unreadable, release = (os.path.join(world.world, name) for name in (SERVER.UNIT_STATES_UNREADABLE_FILE, "unit-unreadable-release"))
    problems = []
    task_id = world.submit(["bash", "-c", UNIT_UNREADABLE_WAIT_SCRIPT, "unit-unreadable-exit-file", release, "3"])
    try:
        attempt = first_attempt(world.running(task_id))
        open(unreadable, "w").close()
        if not wait_unit_unreadable_line(world, attempt.get("id")):
            problems.append(f"读不到单元状态时服务日志应当打点名第 1 次（attempt={attempt.get('id')}）的 TASK_UNIT_UNREADABLE 行，实际没有")
    finally:
        open(release, "w").close()
    try:
        done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(unreadable)
    first = first_attempt(done)
    if done["task"].get("outcome") != "command-failed" or first.get("exit_code") != 3 or len(done["attempts"]) != 1:
        problems.append(f"留了退出码的那一次应当照它收尾（command-failed、退 3、只一次），实际 {describe(done)}、第 1 次 {first.get('outcome')}、退 {first.get('exit_code')}、"
                        f"{len(done['attempts'])} 次")
    raise_cell_problems(world, problems, "读不到单元状态、留了退出码的那一次")
    return "读不到单元状态时留了 result.json 的照它的退出码收尾：command-failed、退 3、只一次"


def cell_unit_unreadable_gone_recorded_lost(world):
    """读不到单元状态、没留 result.json、不经 systemd 核到都没了的记 launcher-lost（task_server.py 文件头 ①「读不到单元状态」）：世界目录里先放 unit-states-unreadable，
    再交一条把起它的 attempt_runner 杀掉（kill -9 $PPID）、睡 2 秒的：起跑标记记的 runner 进程号不在、单元收掉之后它的 cgroup 在本机用户级管理器的 cgroup 树里找不到（算空），
    满宽限记 launcher-lost（第 1 次 ended、launcher-lost），服务日志有点名它的 presence=gone 那一行。
    弄坏开关 unit-unreadable-absent-cgroup-unconfirmed 下不在的 cgroup 目录算核不出，那一次一直照记 running、等收尾等满安静时限，这一格红。"""
    world.start_server()
    unreadable = os.path.join(world.world, SERVER.UNIT_STATES_UNREADABLE_FILE)
    open(unreadable, "w").close()
    problems = []
    try:
        task_id = world.submit(["bash", "-c", 'kill -9 "$PPID"; sleep 2', "unit-unreadable-gone"])
        done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    finally:
        os.remove(unreadable)
    first = first_attempt(done)
    if done["task"].get("outcome") != "launcher-lost" or first.get("state") != "ended" or first.get("outcome") != "launcher-lost":
        problems.append(f"核到都没了的那一次应当记 launcher-lost，实际 {describe(done)}、第 1 次 {first.get('state')}/{first.get('outcome')}")
    if not unit_unreadable_lines(world, first.get("id"), lib.PRESENCE_GONE):
        problems.append(f"照没留结果收尾之前服务日志应当打点名它（attempt={first.get('id')}）、presence={lib.PRESENCE_GONE} 的 TASK_UNIT_UNREADABLE 行（核到都不在），"
                        f"实际点名它的是 {unit_unreadable_lines(world, first.get('id'))[-3:]}")
    raise_cell_problems(world, problems, "读不到单元状态、核到都没了的那一次")
    return "读不到单元状态、没留结果：不经 systemd 核到 runner 进程号不在、单元的 cgroup 已收掉之后记 launcher-lost，服务日志有 presence=gone 那一行"


def cell_unit_unreadable_running_reminded(world):
    """读不到单元状态、进程还在的不杀、照记 running、提醒发起任务的会话（task_server.py 文件头 ①「读不到单元状态」与「要处置的提醒」的 unit-unreadable）：
    会话的收件 socket（FakeInbox）在，一条命令跑起来之后世界目录里放 unit-states-unreadable：起跑标记记的 runner 进程号在、单元的 cgroup 里有进程，核到还在；
    满 FAST_UNIT_UNREADABLE_ATTENTION_SECONDS（2 秒）收到恰好一条「task-server：提醒（任务还在跑，不是结局）任务 <号> 读不到单元状态（unit-unreadable；第 1 回」（reminded_after_seconds=2），
    依据写本机、单元名、读不到什么与核到的进程号，再看几轮不多一条；满 4 秒再收一条 reminded_after_seconds=4 的；这一段里任务与那一次尝试都照记 running。放行之后照 result.json 收尾 succeeded。
    弄坏开关 unit-unreadable-lost-unverified 下读不到就当场照没留结果收尾、停掉单元里的进程（launcher-lost），unit-unreadable-unreminded 下收不到提醒，
    unit-unreadable-exit-file-unread 下放行之后不认 result.json、记 launcher-lost，这一格红。"""
    inbox = FakeInbox(world, "a")
    start_inbox_server(world)
    unreadable, release = (os.path.join(world.world, name) for name in (SERVER.UNIT_STATES_UNREADABLE_FILE, "unit-unreadable-release"))
    problems = []
    task_id = world.submit(["bash", "-c", UNIT_UNREADABLE_WAIT_SCRIPT, "unit-unreadable-running", release, "0"], extra=inbox.environment())
    try:
        attempt = first_attempt(world.running(task_id))
        open(unreadable, "w").close()
        head = reminder_head(task_id, "unit-unreadable")
        first_after, second_after = round(SERVER.FAST_UNIT_UNREADABLE_ATTENTION_SECONDS), round(2 * SERVER.FAST_UNIT_UNREADABLE_ATTENTION_SECONDS)
        inbox.wait_for(world, head)
        time.sleep(INBOX_QUIET_SECONDS)
        first = reminders_after(inbox.headed(head), first_after)
        problems += inbox_problems(first, head, f"读不到单元状态满 {first_after} 秒那一回")
        text = first[0]["text"] if first else ""
        for wanted in ("依据：", "本机读不到它的单元", str(attempt.get("unit")), "进程号", "出路：",
                       f"{lib.ATTENTION_MARK} task={task_id} type={lib.ATTENTION_TYPE} kind=unit-unreadable reminded_after_seconds={first_after} basis="):
            if wanted not in text:
                problems.append(f"读不到单元状态那一条的正文里应当有「{wanted}」，实际「{text[:500]}」")
        inbox.wait_for(world, head, count=2)
        time.sleep(INBOX_QUIET_SECONDS)
        problems += inbox_problems(reminders_after(inbox.headed(head), second_after), head, f"读不到单元状态满 {second_after} 秒那一回")
        if len(reminders_after(inbox.headed(head), first_after)) != 1:
            problems.append(f"满 {second_after} 秒之后，{first_after} 秒那一条仍该只有一条，实际 {len(reminders_after(inbox.headed(head), first_after))} 条")
        during = world.summary(task_id)
        if during["task"].get("state") != "running" or first_attempt(during).get("state") != "running":
            problems.append(f"读不到单元状态而进程还在的那一次应当照记 running，实际任务 {during['task'].get('state')}、第 1 次 {first_attempt(during).get('state')}"
                            f"/{first_attempt(during).get('outcome')}")
    finally:
        open(release, "w").close()
    try:
        done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    finally:
        with contextlib.suppress(FileNotFoundError):
            os.remove(unreadable)
    if done["task"].get("outcome") != "succeeded" or len(done["attempts"]) != 1:
        problems.append(f"放行之后应当照 result.json 收尾 succeeded、只一次，实际 {describe(done)}、{len(done['attempts'])} 次")
    raise_cell_problems(world, problems, "读不到单元状态、进程还在的那一次")
    return "读不到单元状态而进程还在：不杀、照记 running，满 2 秒、满 4 秒各推恰好一条 unit-unreadable（依据写本机、单元与核到的进程号），放行后照退出码收尾 succeeded"


def cell_unit_unreadable_unverified_health_reported(world):
    """读不到单元状态、没留 result.json、cgroup 也读不到的一直等（task_server.py 文件头 ①「读不到单元状态」）：世界目录里先放 unit-states-unreadable 与 unit-cgroups-unreadable
    （本机核进程的 cgroup 根换成不在的一处），再交一条把起它的 attempt_runner 杀掉、睡 1 秒的：runner 进程号不在而 cgroup 核不出，不杀、不收尾，那一次照记 running；
    /health 回不健康，problem 写本机读不到在跑尝试的单元状态，unit_unreadable 栏点名这一条（presence=unverified）；control check 退 78、报同一句；
    再看 UNIT_UNREADABLE_HOLD_SECONDS 仍照记 running。撤掉两份文件之后照读得到的单元状态判，单元不在了记 launcher-lost。
    弄坏开关 unit-unreadable-health-unreported 下 /health 不报，unit-unreadable-lost-unverified 下当场记 launcher-lost，这一格红。"""
    world.start_server()
    flags = [os.path.join(world.world, name) for name in (SERVER.UNIT_STATES_UNREADABLE_FILE, SERVER.UNIT_CGROUPS_UNREADABLE_FILE)]
    release_all(flags)
    problems = []
    try:
        task_id = world.submit(["bash", "-c", 'kill -9 "$PPID"; sleep 1', "unit-unreadable-unverified"])
        health = health_until(world, lambda reading: any(entry.get("task") == task_id and entry.get("presence") == lib.PRESENCE_UNVERIFIED
                                                         for entry in reading.get("unit_unreadable") or []), PLACEMENT_WAIT_SECONDS)
        entries = [entry for entry in health.get("unit_unreadable") or [] if entry.get("task") == task_id]
        if (health.get("ok") or SERVER.UNIT_UNREADABLE_HEALTH_TEXT not in (health.get("problem") or "") or not entries
                or entries[0].get("presence") != lib.PRESENCE_UNVERIFIED):
            problems.append(f"读不到单元状态、cgroup 也读不到时 /health 应当回不健康、problem 写「{SERVER.UNIT_UNREADABLE_HEALTH_TEXT}」、unit_unreadable 栏点名任务 {task_id}"
                            f"（presence={lib.PRESENCE_UNVERIFIED}），实际 ok={health.get('ok')}、problem「{(health.get('problem') or '')[:300]}」、栏 {entries}")
        checked = world.control(["check"])
        if checked.returncode != 78 or SERVER.UNIT_UNREADABLE_HEALTH_TEXT not in checked.stderr:
            problems.append(f"control check 应当退 78、报「{SERVER.UNIT_UNREADABLE_HEALTH_TEXT}」，实际退 {checked.returncode}、「{(checked.stdout + checked.stderr).strip()[-300:]}」")
        time.sleep(UNIT_UNREADABLE_HOLD_SECONDS)
        during = world.summary(task_id)
        if during["task"].get("state") != "running" or first_attempt(during).get("state") not in SERVER.ACTIVE_ATTEMPT_STATES:
            problems.append(f"核不出的那一次应当一直照记 running（不杀、不收尾），实际任务 {during['task'].get('state')}、第 1 次 {first_attempt(during).get('state')}"
                            f"/{first_attempt(during).get('outcome')}")
    finally:
        for path in flags:
            with contextlib.suppress(FileNotFoundError):
                os.remove(path)
    done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
    if done["task"].get("outcome") != "launcher-lost":
        problems.append(f"撤掉两份文件之后单元不在了应当记 launcher-lost，实际 {describe(done)}")
    raise_cell_problems(world, problems, "读不到单元状态、cgroup 也读不到的那一次")
    return (f"读不到单元状态、cgroup 也读不到：照记 running 不杀，/health 报「{SERVER.UNIT_UNREADABLE_HEALTH_TEXT}…」、unit_unreadable 栏 presence={lib.PRESENCE_UNVERIFIED}，"
            "control check 退 78；读得到之后照单元不在记 launcher-lost")


def cell_peer_cleanup_off_scheduler(world):
    """第二台收尾卡住时本机照起（task_server.py 的 finish_off_loop；loop 审核发现 4）：派第二台的一条跑完、清理回执 pending，假 multi-host-run.sh 的 --recover-cleanup 卡着不回
    （世界目录里 recover-cleanup-hold 在、等 recover-cleanup-release）；卡着的这段里第二台演用不了、本机放得下，交一条本机的：它照起、在 PEER_RECOVER_HOLD_SECONDS 的安静时限里跑完，
    第二台那一条照旧在跑、不记结局；放开之后它收尾 succeeded。弄坏开关 finish-in-scheduler 下收尾在排队线程里当场做，排队线程卡在 --recover-cleanup 上，本机那一条一直排着，这一格红。"""
    world.resources(local=(1, 30), peer=(100, 100), peer_cap_gibibytes=64)
    world.start_server()
    hold, reached, release = (os.path.join(world.world, name) for name in ("recover-cleanup-hold", "recover-cleanup-reached", "recover-cleanup-release"))
    open(hold, "w").close()
    problems = []
    try:
        peer_id = world.submit(["bash", "-c", "exit 0", "peer-cleanup-stuck", "fake-peer-cleanup=pending"], cwd=world.repo)
        wait_path(world, reached, "第二台那一条收尾卡在 --recover-cleanup 上")
        world.resources(local=(30, 30), peer_down="自证：第二台这一刻不收新活")
        local_id = world.submit(["bash", "-c", "exit 0", "beside-stuck-peer-cleanup"])
        local = world.finished(local_id, timeout=PEER_RECOVER_HOLD_SECONDS)
        during = world.summary(peer_id)
        if local["task"].get("outcome") != "succeeded" or first_attempt(local).get("host") != SERVER.LOCAL:
            problems.append(f"第二台收尾卡着时本机那一条应当在本机跑完，实际 {describe(local)}、派到 {first_attempt(local).get('host')}")
        if during["task"].get("state") != "running" or first_attempt(during).get("state") not in ("starting", "running") or os.path.exists(release):
            problems.append(f"第二台那一条收尾还卡着：应当照旧在跑、不记结局，实际 {describe(during)}、尝试 {first_attempt(during).get('state')}")
    finally:
        open(release, "w").close()
    done = world.finished(peer_id)
    if done["task"].get("outcome") != "succeeded" or first_attempt(done).get("host") != SERVER.PEER:
        problems.append(f"放开之后第二台那一条应当收尾 succeeded，实际 {describe(done)}")
    raise_cell_problems(world, problems, "第二台收尾卡住时本机照起")
    return "第二台收尾卡在 --recover-cleanup 上：本机那一条照起、跑完，第二台那一条放开之后收尾"


STOPPED_UNIT_ATTEMPT = 3   # unit-stopped-booking-released：单元已停、收尾还没做完的那一条在跑尝试的号


def cell_unit_stopped_booking_released(world):
    """单元确认停了就放掉这一次尝试的 CPU 与内存记账（task_server.py 的 release_stopped_booking、unit_confirmed_stopped、booking_attempts；任务 13716 到 13718 取消后单元已 inactive、
    两台都没有进程，结局隔了十几分钟才落库，期间 CPU 闸照记各 31 线程）：本机三条在跑（起了 100 秒、命令键记过 31 核，照记录各记 31 线程，共 93 线程），第三条的单元读回 inactive 而且 cgroup.procs 空：
    放掉它之后 CPU 闸记 62 线程；内存一侧（slice 30G、余量 1G，一条在跑、上限 20G、scope 此刻用满 20G）记账 20G、公式剩 9G、待涨量 0，
    放掉之后公式剩 29G、待涨量 0。读不到状态（unit_states 交 None）、cgroup.procs 读不到、cgroup.procs 里还有进程、单元还在跑的，都不算停：照数。收尾记 ended 之后（finish_attempt 清）这一条不再留。
    弄坏开关 unit-stopped-still-booked 下停了的也照数（93 线程、公式剩 9G），这一格红。"""
    load = {"load1": 2.0, "procs_running": 3, "cpus": 32}
    attempt = {"id": STOPPED_UNIT_ATTEMPT, "label": RUNNING_BOOKING_LABEL, "unit": "stopped-unit-selftest.service", "host": SERVER.LOCAL}
    problems = []
    readings = (("单元 inactive、cgroup.procs 空", ("loaded", "inactive"), [], True), ("读不到单元状态", None, [], False),
                ("cgroup.procs 读不到", ("loaded", "inactive"), None, False), ("cgroup.procs 里还有进程", ("loaded", "inactive"), [4242], False),
                ("单元还在跑", ("loaded", "active"), [], False))
    for what, states, processes, stopped in readings:
        fake, _ = cpu_gate_scheduler(3, load, recorded_cores=(31.0,), keys=[CPU_GATE_PROBE_KEY] * 3)
        with unittest.mock.patch.object(SERVER, "unit_states", lambda unit, environment, states=states: states), \
                unittest.mock.patch.object(SERVER, "unit_processes", lambda unit, environment, processes=processes: processes):
            confirmed = SERVER.Scheduler.release_stopped_booking(fake, attempt)
        counted = SERVER.Scheduler.running_threads(fake, SERVER.LOCAL, time.time())
        wanted_threads = 62 if stopped else 93
        if confirmed != stopped or counted != wanted_threads:
            problems.append(f"{what}：应当{'确认停了、记 62 线程' if stopped else '不算停、照记 93 线程'}，实际确认 {confirmed}、记 {counted} 线程")
        if stopped:
            fake.store.running_attempts = lambda: []   # 收尾记了 ended：不在在跑名单里，布告栏里这一条清掉
            SERVER.Scheduler.booking_attempts(fake)
            if fake.unit_stopped:
                problems.append(f"{what}：尝试不在在跑名单里之后应当从放掉的名单里清掉，实际还留着 {sorted(fake.unit_stopped)}")
    memory_attempt = {"id": 1, "label": RUNNING_BOOKING_LABEL, "unit": "stopped-unit-selftest.service", "host": SERVER.LOCAL}
    for released, wanted_room in ((False, 9 * GIBIBYTE), (True, 29 * GIBIBYTE)):
        fake, snapshot, report = running_booking_scheduler(20 * GIBIBYTE, 20 * GIBIBYTE)
        if released:
            with unittest.mock.patch.object(SERVER, "unit_states", lambda unit, environment: ("loaded", "inactive")), \
                    unittest.mock.patch.object(SERVER, "unit_processes", lambda unit, environment: []):
                SERVER.Scheduler.release_stopped_booking(fake, memory_attempt)
        room = SERVER.Scheduler.room_by_the_formula(fake, SERVER.LOCAL, snapshot)
        pending = SERVER.Scheduler.server_pending(fake, SERVER.LOCAL, report)
        if room != wanted_room or pending != 0:
            problems.append(f"内存{'放掉之后' if released else '没放掉时'}：公式应当剩 {wanted_room / GIBIBYTE:g}G、待涨量 0，"
                            f"实际剩 {room / GIBIBYTE:g}G、待涨量 {pending / GIBIBYTE:g}G")
    if problems:
        raise SelftestFailure("单元已停先放记账：" + "；".join(problems))
    return "单元回读停了：CPU 闸 93 线程变 62、内存公式剩 9G 变 29G；读不到的、还有进程的、还在跑的照数"


def cell_cpu_reason_splits_io_wait(world):
    """CPU 闸的排队原因把等 IO 的线程拆开写（task_server.py 的 cpu_problem；余量报告 CPU 一项的 blocked_threads，memory_peaks_lib 的 cpu_report）：32 核（线 64）、负载 297，
    其中状态 D 的线程 249 个：再起一条排着，原因写「负载 297（其中等 IO 的线程 249）」；闸的判法不变（同样的负载、blocked_threads 是 0 或没有这一项，排不排一样，没有这一项的原因里不写括号）。
    弄坏开关 cpu-load-io-unsplit 下不拆开写，原因只写「负载 297」，这一格红。"""
    problems = []
    for what, blocked, wanted_text in (("249 个线程在等 IO", 249, "负载 297（其中等 IO 的线程 249）"), ("报告里没有等 IO 的线程数", None, "负载 297、")):
        cpu = {"load1": 297.0, "procs_running": 3, "cpus": 32}
        if blocked is not None:
            cpu["blocked_threads"] = blocked
        fake, snapshot = cpu_gate_scheduler(0, cpu, recorded_cores=(31.0,))   # 这一条照记录要 31 核：不走要核数少的名额
        why = SERVER.Scheduler.cpu_problem(fake, {"id": 1, "command_key": CPU_GATE_PROBE_KEY}, SERVER.LOCAL, snapshot)
        if not why.startswith("CPU：") or wanted_text not in why:
            problems.append(f"{what}：应当排着、原因以「CPU：」起头并写「{wanted_text}」，实际「{why}」")
    if problems:
        raise SelftestFailure("CPU 闸的原因拆开写等 IO 的线程：" + "；".join(problems))
    return "负载 297 里 249 个线程在等 IO：原因写「负载 297（其中等 IO 的线程 249）」；报告里没有这一项的不写括号；两种都照旧排着"


NO_BORROW_ENTRY_SCRIPT = "scripts/gate/selftest-no-borrow-entry.sh"   # shard-start-borrows-nothing：被分片的入口脚本（父任务的 sharded_entry）
NO_BORROW_HEAD = f"bash {NO_BORROW_ENTRY_SCRIPT} --check selftest-cell-a"   # 这一趟分片的命令键原文（分片那一段之前）
NO_BORROW_OTHER_HEAD = f"bash {NO_BORROW_ENTRY_SCRIPT} --check selftest-cell-b"   # 同一入口脚本、别的参数的那一趟
NO_BORROW_PARENT, NO_BORROW_SIBLING = 90, 92   # 这一趟的父任务与同一趟的兄弟片（共 4 片的第 0 片）的任务号


def cell_shard_start_borrows_nothing(world):
    """自己这把键没有相当记录的片谁的都不借：同一趟的兄弟片、同入口别的参数那一趟、别的片数都不取，照没记录的缺省起（task_server.py 的 computed_prediction，
    定额度的入口；task_server_lib.py 文件头「资源记录」的定额度）：共 4 片的第 2 片。库里同一趟的兄弟片（同一个父任务名下、共 4 片的第 0 片）跑完峰值 60G，
    同一入口脚本、别的参数那一趟共 4 片的第 2 片（键不同）跑完 40G、共 8 片的第 1 片跑完 70G：起点应当是没记录的缺省 4096M、来历 unrecorded；
    对照：库里再有这一片自己这把键、片数相同的 6G 记录，起点就是 6G、来历是 record。
    弄坏开关 record-ignored-for-cap 下对照那一种也照缺省起，这一格红。"""
    store_rows = []
    children = [({"id": 91, "task_id": NO_BORROW_PARENT}, True), ({"id": NO_BORROW_SIBLING, "task_id": NO_BORROW_PARENT}, True)]

    def row(head, index, count, peak, when, task_id):
        store_rows.append(({"command_key": lib.shard_child_key(head, "shard", index, count), "state": "ended", "outcome": "succeeded", "memory_peak": peak, "memory_cap": peak + GIBIBYTE,
                            "shard_count": count, "role": "shard", "task_id": task_id, "ended_at": float(when), "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC}, True))

    def select(table, where="1", parameters=(), order="id"):
        if table == "attempts":
            return [item for item in store_rows if item[0].get("command_key") == parameters[0]]
        return list(children) if table == "tasks" and parameters == (NO_BORROW_PARENT,) else []

    parent = {"id": NO_BORROW_PARENT, "command_key": NO_BORROW_HEAD, "sharded_entry": NO_BORROW_ENTRY_SCRIPT}
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peer_root = dict(os.environ), lib.REPOSITORY_ROOT   # 定额度末了的内存下限（memory_floor_applied）读命令与仓根，认是不是换文件、打补丁的工具
    fake.host_dispatch = loaded_host_dispatch()
    fake.reads_log = None
    fake.prediction_lock = threading.Lock()
    fake.store = types.SimpleNamespace(select=select, task=lambda task_id: (parent, True) if task_id == NO_BORROW_PARENT else None)
    shard = {"id": 91, "role": "shard", "parent_id": NO_BORROW_PARENT, "shard_index": 2, "shard_count": 4, "command_key": lib.shard_child_key(NO_BORROW_HEAD, "shard", 2, 4),
             "command": ["bash", NO_BORROW_ENTRY_SCRIPT, "--check", "selftest-cell-a"], "cwd": lib.REPOSITORY_ROOT}

    def started():
        found = SERVER.Scheduler.computed_prediction(fake, shard)
        return found.memory_cap, found.memory_history

    row(NO_BORROW_HEAD, 0, 4, 60 * GIBIBYTE, 100, NO_BORROW_SIBLING)
    row(NO_BORROW_OTHER_HEAD, 2, 4, 40 * GIBIBYTE, 110, 81)
    row(NO_BORROW_OTHER_HEAD, 1, 8, 70 * GIBIBYTE, 115, 82)
    borrowed = started()
    row(NO_BORROW_HEAD, 2, 4, 6 * GIBIBYTE, 120, 71)
    own = started()
    problems = []
    for what, got, want in (("自己没有记录、库里有同一趟的兄弟片 60G、同一入口别的参数那一趟同片数 40G 与片数不同的 70G", borrowed, (lib.UNRECORDED_MEMORY_DEFAULT_BYTES, lib.UNRECORDED_HISTORY)),
                            ("对照：自己这把键有片数相同的 6G 记录", own, (6 * GIBIBYTE, lib.RECORD_HISTORY))):
        if got != want:
            problems.append(f"{what}：起点与来历应当是 {want}，实际 {got}")
    if problems:
        raise SelftestFailure("没有自己记录的片谁的都不借：" + "；".join(problems))
    return "共 4 片的第 2 片自己没有记录：同一趟的兄弟片 60G、同入口别的趟 40G、片数不同的 70G 都不取，照缺省 4096M 起；自己有 6G 记录就照 6G（record）"


PLAN_KEY_HEAD = f"bash {NO_BORROW_ENTRY_SCRIPT} --check selftest-plan-keys"   # plan-key-by-compiles：这一趟分片的命令键原文（计划那一段之前）
PLAN_KEY_COMPILED_BYTES = 9 * GIBIBYTE     # 编的那一种计划步记下的峰值
PLAN_KEY_UNCOMPILED_BYTES = 2 * GIBIBYTE   # 不编的那一种计划步记下的峰值


def cell_plan_key_by_compiles(world):
    """计划那一步的命令键带「这一步编不编」（task_server_lib.py 的 shard_child_key；task_server.py 的 child_compiles、child_key_text）：同一父键，编与不编两种计划步落两把键，
    过了键的换算（memory_peaks_lib 的 command_key）仍不同；定额度各读各的（task_server.py 的 computed_prediction）：库里只有不编的那把键跑完 2G 时，
    编的那一种照没有记录的缺省 4096M 起（unrecorded）、不编的取 2G；两把键各有记录（编的 9G、不编的 2G）时各取各的。套在假调度器上，不起服务。
    弄坏开关 plan-key-compile-unmarked 下两种计划步落同一把键：编的那一种取了不编的 2G、不编的那一种取了编的 9G，这一格红。"""
    store_rows = []

    def select(table, where="1", parameters=(), order="id"):
        return [item for item in store_rows if item[0].get("command_key") == parameters[0]] if table == "attempts" else []

    def seed(key, peak, when):
        store_rows.append(({"command_key": key, "state": "ended", "outcome": "succeeded", "memory_peak": peak, "memory_cap": peak + GIBIBYTE,
                            "role": "plan", "task_id": 80, "ended_at": float(when), "peak_metric": lib.PROCESS_MEMORY_PEAK_METRIC}, True))

    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peer_root = dict(os.environ), lib.REPOSITORY_ROOT   # 定额度末了的内存下限（memory_floor_applied）读命令与仓根
    fake.host_dispatch = loaded_host_dispatch()
    fake.reads_log = None
    fake.prediction_lock = threading.Lock()
    fake.store = types.SimpleNamespace(select=select, task=lambda task_id: None)
    parent = {"compiles": True}
    compiled_key = lib.shard_child_key(PLAN_KEY_HEAD, "plan", 0, 0, compiles=SERVER.Scheduler.child_compiles(parent, "plan"))
    uncompiled_key = lib.shard_child_key(PLAN_KEY_HEAD, "plan", 0, 0, compiles=SERVER.Scheduler.child_compiles(dict(parent, compiles=False), "plan"))
    plans = {what: {"id": number, "role": "plan", "parent_id": 80, "shard_index": 0, "shard_count": 0, "command_key": key,
                    "command": ["bash", NO_BORROW_ENTRY_SCRIPT, "--check", "selftest-plan-keys"], "cwd": lib.REPOSITORY_ROOT}
             for number, (what, key) in enumerate((("编", compiled_key), ("不编", uncompiled_key)), 81)}

    def started():
        found = {what: SERVER.Scheduler.computed_prediction(fake, task) for what, task in plans.items()}
        return {what: (prediction.memory_cap, prediction.memory_history) for what, prediction in found.items()}

    problems = []
    folded = {PEAKS.command_key([], compiled_key), PEAKS.command_key([], uncompiled_key)}
    if compiled_key == uncompiled_key or len(folded) != 2:
        problems.append(f"编与不编的计划步应当是两把键（过了键的换算也是），实际「{compiled_key}」「{uncompiled_key}」、换算后 {sorted(folded)}")
    seed(uncompiled_key, PLAN_KEY_UNCOMPILED_BYTES, 100)
    only_uncompiled = started()
    seed(compiled_key, PLAN_KEY_COMPILED_BYTES, 110)
    both = started()
    for what, got, want in (("库里只有不编的 2G", only_uncompiled, {"编": (lib.UNRECORDED_MEMORY_DEFAULT_BYTES, lib.UNRECORDED_HISTORY),
                                                                    "不编": (PLAN_KEY_UNCOMPILED_BYTES, lib.RECORD_HISTORY)}),
                            ("编的 9G、不编的 2G 各有记录", both, {"编": (PLAN_KEY_COMPILED_BYTES, lib.RECORD_HISTORY), "不编": (PLAN_KEY_UNCOMPILED_BYTES, lib.RECORD_HISTORY)})):
        if got != want:
            problems.append(f"{what}：两种计划步的起点与来历应当是 {want}，实际 {got}")
    if problems:
        raise SelftestFailure("计划那一步编与不编各记各的：" + "；".join(problems))
    return "同一父键编与不编的计划步落两把键：只有不编的 2G 时编的照缺省 4096M 起；各有记录时编的取 9G、不编的取 2G"


DRIFT_LABELS = ("task-selftest-drift-1", "task-selftest-drift-2")   # booking-drift-reported：两条在跑尝试的标签
DRIFT_QUEUED_TASK = 90   # 同一格里被挡住、还在排的那条任务


def booking_audit_scheduler(attempts):
    """booking-drift-reported 的假调度器：本机 slice 30G，在跑的 attempts（各上限 12G），排着一条任务（被「在跑的记账之和」挡着、记着会话令牌）；
    send_to_session 记下发出去的正文。→ (假调度器, 发出去的正文表)。"""
    fake, sent = object.__new__(SERVER.Scheduler), []
    queued = {"id": DRIFT_QUEUED_TASK, "state": "queued", "session": SESSION_A, "inbox": {"token": INBOX_TOKEN_SAMPLE}}
    fake.environment, fake.peaks = dict(os.environ), PEAKS
    fake.host_dispatch = loaded_host_dispatch()   # audit_bookings 末了的记账明细（refresh_booking_ledger）要读余量报告里的余量
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(label="本机")}
    fake.store = types.SimpleNamespace(running_attempts=lambda: [dict(attempt) for attempt in attempts],
                                       select=lambda table, where="1", parameters=(), order="id": [(queued, True)] if table == "tasks" else [],
                                       task=lambda task_id: (queued, True) if task_id == DRIFT_QUEUED_TASK else None,
                                       write_restart_surviving_state=lambda name, state: None)   # 推到的提醒门槛记进库（persist_restart_surviving_state）：这里不起库，写成空
    fake.restart_surviving_lock, fake.restart_surviving_written = threading.Lock(), {}
    fake.unit_stopped = set()
    fake.booking_audit_state, fake.booking_drift, fake.booking_drift_notified = {}, {"memory": {}}, {}
    fake.booking_drift_lock = threading.Lock()
    fake.waiting_reasons = {DRIFT_QUEUED_TASK: f"内存：要记 12288 MiB，配置的总上限减余量减{SERVER.RUNNING_BOOKED_SUM_TEXT}只剩 0 MiB"}
    fake.inbox_reachable = lambda task: True
    fake.send_to_session = lambda task, text, event: sent.append(text) or True
    return fake, sent


def cell_booking_drift_reported(world):
    """服务自己定时对账（task_server.py 的 audit_bookings、audit_memory_bookings、booking_row、push_booking_drift；不等别的会话汇报）：
    有 scope 的在跑尝试照实测记账，对账的基准是各 scope 实测之和，漂移只来自报告里没有它的 scope、记整份上限的那几条。
    本机两条在跑各上限 12G、报告里都没有它们的 scope（各记 12G、实占读不到，实测合计 0），
    连着三轮：头两轮不判，第三轮判记账漂移，/health 的 booking_drift 一栏（booking_drift_snapshot）列出两条各记 12G、实占读不到、为什么（报告里没有它的 scope，记整份上限）；
    推给排着的任务所在的会话恰好一次（正文写「记账漂移」与两条的记账与实占），再对两轮不重推；过了门槛（600 秒）再推一回、首行写再推一回；
    报告里有 scope、各实占 11G 的照实测各记 11G，不报、不推。
    弄坏开关 booking-audit-skipped 下不对账，这一格红。"""
    gib, problems = GIBIBYTE, []
    attempts = [{"id": number, "task_id": 70 + number, "host": SERVER.LOCAL, "label": label, "memory_cap": 12 * gib} for number, label in enumerate(DRIFT_LABELS, 1)]
    start = time.time()

    def audit(fake, used_each, rounds, first=0):
        """对 rounds 轮账；used_each 是报告里每条的 scope 回收不掉的用量，None 是报告里没有这两条的 scope。"""
        report = memory_report(30, 30, scopes={} if used_each is None else {label: used_each for label in DRIFT_LABELS})
        snapshot = SERVER.Snapshot({SERVER.LOCAL: report}, {}, {SERVER.LOCAL: True}, {})
        for number in range(first, first + rounds):
            SERVER.Scheduler.audit_bookings(fake, snapshot, start + 60 * number)

    fake, sent = booking_audit_scheduler(attempts)
    audit(fake, None, 2)
    early = SERVER.Scheduler.booking_drift_snapshot(fake)["memory"]
    audit(fake, None, 1, first=2)
    drift = SERVER.Scheduler.booking_drift_snapshot(fake)["memory"].get(SERVER.LOCAL) or {}
    rows = drift.get("attempts") or []
    if early or [(row["label"], row["booked"], row["used"]) for row in rows] != [(label, 12 * gib, None) for label in DRIFT_LABELS]:
        problems.append(f"连着三轮记 12G、报告里没有 scope：前两轮不判、第三轮判漂移并逐条列两条的记账与实占（读不到），"
                        f"实际前两轮 {bool(early)}、第三轮 {[(row.get('label'), row.get('booked'), row.get('used')) for row in rows]}")
    if rows and not all("没有它的 scope" in row["why"] and "12.0G" in row["why"] for row in rows):
        problems.append(f"逐条应当写为什么（报告里没有它的 scope，记整份上限 12.0G），实际 {[row['why'] for row in rows]}")
    if len(sent) != 1 or "记账漂移" not in sent[0] or not all(label in sent[0] for label in DRIFT_LABELS) or "12.0G" not in sent[0] or "读不到" not in sent[0]:
        problems.append(f"判出的当场应当推给被挡住的会话一回、正文写记账漂移与两条的记账与实占，实际推了 {len(sent)} 回：{[text[:200] for text in sent]}")
    audit(fake, None, 2, first=3)
    if len(sent) != 1:
        problems.append(f"同一件事同一道门槛不重推：再对两轮之后应当还是 1 回，实际 {len(sent)} 回")
    audit(fake, None, 1, first=2 + int(SERVER.BOOKING_DRIFT_PUSH_SECONDS // 60) + 1)
    if len(sent) != 2 or "再推一回" not in sent[-1].split("\n", 1)[0]:
        problems.append(f"过了门槛（{SERVER.BOOKING_DRIFT_PUSH_SECONDS:g} 秒）应当再推一回、首行写再推一回，实际共 {len(sent)} 回：{[text.splitlines()[0] for text in sent]}")
    calm, calm_sent = booking_audit_scheduler(attempts)
    audit(calm, 11 * gib, 5)
    if SERVER.Scheduler.booking_drift_snapshot(calm)["memory"] or calm_sent:
        problems.append(f"报告里有 scope、各实占 11G（照实测各记 11G）：不该报、不该推，实际 {SERVER.Scheduler.booking_drift_snapshot(calm)['memory']}、推了 {len(calm_sent)} 回")
    if problems:
        raise SelftestFailure("服务定时对账：" + "；".join(problems))
    return "报告里没有 scope 的两条各记 12G 连三轮：判记账漂移、逐条列记账实占与原因、推给排着的会话一回（过了门槛再推）；有 scope 的照实测记、不报"


YOUNG_AUDIT_SECONDS = "120"     # booking-audit-young-not-drift：起跑不满这么多秒照整份上限记（YOUNG_BOOKING_VARIABLE），同 running_booking 的两分钟
YOUNG_AUDIT_AGE_SECONDS = 10    # 两条在跑尝试起跑了这么久：在时限之内
YOUNG_AUDIT_USED_BYTES = 512 * MEBIBYTE   # 报告里每条 scope 此刻回收不掉的用量：两条合 1G，记整份上限的 24G 是它的 24 倍、多出 23G，照记账算就够漂移的界


def cell_booking_audit_young_not_drift(world):
    """刚起的不报漂移，对账那一层（task_server.py 的 audit_memory_bookings；文件头 ②「内存」的记账对账；running_booking 那一层由 running-young-booked-at-cap 判）：
    本机两条在跑各上限 12G、起跑 10 秒（时限 120 秒）、报告里有各自的 scope、各实占 512M：booking_row 照规矩记整份上限 12G（young），
    连着对 BOOKING_DRIFT_ROUNDS 轮再多两轮账：对账照实占算，不判记账漂移、/health 的 booking_drift 一栏空着、不推给排着的会话。套在假调度器上，不起服务。
    弄坏开关 booking-audit-young-counted 下对账照整份上限算，第三轮判漂移、推一回，这一格红。"""
    gib = GIBIBYTE
    start = time.time()
    attempts = [{"id": number, "task_id": 70 + number, "host": SERVER.LOCAL, "label": label, "memory_cap": 12 * gib, "started_at": start - YOUNG_AUDIT_AGE_SECONDS}
                for number, label in enumerate(DRIFT_LABELS, 1)]
    fake, sent = booking_audit_scheduler(attempts)
    fake.environment = dict(fake.environment, **{lib.YOUNG_BOOKING_VARIABLE: YOUNG_AUDIT_SECONDS})
    report = memory_report(30, 30, scopes={label: YOUNG_AUDIT_USED_BYTES for label in DRIFT_LABELS})
    snapshot = SERVER.Snapshot({SERVER.LOCAL: report}, {}, {SERVER.LOCAL: True}, {})
    usage = SERVER.Scheduler.report_scope_usage(fake, report)
    problems = []
    rows = [SERVER.Scheduler.booking_row(fake, attempt, usage) for attempt in attempts]
    if [(row["booked"], row["basis"]) for row in rows] != [(12 * gib, SERVER.BOOKED_WHILE_YOUNG)] * len(attempts):
        problems.append(f"起跑 {YOUNG_AUDIT_AGE_SECONDS} 秒的两条应当照规矩记整份上限 12G（{SERVER.BOOKED_WHILE_YOUNG}；这一格的样本前提），"
                        f"实际 {[(row['booked'], row['basis']) for row in rows]}")
    rounds = SERVER.BOOKING_DRIFT_ROUNDS + 2
    for number in range(rounds):
        SERVER.Scheduler.audit_bookings(fake, snapshot, start + 60 * number)
    drift = SERVER.Scheduler.booking_drift_snapshot(fake)["memory"]
    if drift or sent:
        problems.append(f"刚起的两条各记 12G、各实占 512M，连对 {rounds} 轮：对账照实占算，不该判漂移、不该推，"
                        f"实际 booking_drift {drift}、推了 {len(sent)} 回：{[text[:200] for text in sent]}")
    if problems:
        raise SelftestFailure("刚起的对账不报漂移：" + "；".join(problems))
    return f"起跑 {YOUNG_AUDIT_AGE_SECONDS} 秒的两条各记整份上限 12G、各实占 512M：连对 {rounds} 轮，对账照实占算、不判漂移、不推"


FAKE_JOURNALCTL_EARLYOOM = r'''#!/usr/bin/env bash
# task-server 自证的假 journalctl（TASK_SERVER_JOURNALCTL；服务读 earlyoom 与内核日志认被外部信号杀掉的尝试，task_server.py 的 kill_log_lines）：
# 参数里有 -t earlyoom 时，世界目录里的 earlyoom-pid 在、里面有进程号，就打一行 earlyoom 杀进程的日志点名这个号（没有就一行不打）；有 -k 的一行不打；别的参数退 9
case " $* " in
  *" -t earlyoom "*)
    if [[ -s "$TASK_SERVER_SELFTEST_WORLD/earlyoom-pid" ]]; then
      echo "sending SIGTERM to process $(cat -- "$TASK_SERVER_SELFTEST_WORLD/earlyoom-pid") uid 1000 \"rustc\": badness 1414, VmRSS 9708 MiB"
    fi
    exit 0 ;;
  *" -k "*) exit 0 ;;
  *) exit 9 ;;
esac
'''
EXTERNAL_KILL_COMMAND = 'echo $$ > "$1"; sleep 2; if [[ -e "$2" ]]; then exit 0; fi; : > "$2"; exit 1'   # 头一次把自己的进程号写给假 journalctl 当 earlyoom 杀掉的那个、退 1；第二次退 0


def machine_memory_scheduler():
    """machine-memory-line-holds-dispatch 的假调度器：只备 machine_memory_problem 用到的几样。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.peaks = PEAKS
    fake.host_dispatch = loaded_host_dispatch()
    return fake


def machine_memory_snapshot(total_gibibytes, available_gibibytes, swap_total_gibibytes=None, swap_free_gibibytes=None):
    """本机一份报告：整机内存总量与可用量（GiB），swap 总量与空闲（给了才写这两项）。"""
    memory = {"mem_total": int(total_gibibytes * GIBIBYTE), "mem_available": int(available_gibibytes * GIBIBYTE)}
    if swap_total_gibibytes is not None:
        memory[PEAKS.SWAP_TOTAL_KEY], memory[PEAKS.SWAP_FREE_KEY] = int(swap_total_gibibytes * GIBIBYTE), int(swap_free_gibibytes * GIBIBYTE)
    return types.SimpleNamespace(reports={SERVER.LOCAL: {"memory": memory}})


def cell_machine_memory_line_holds_dispatch(world):
    """整机可用内存接近 earlyoom 的线时不派新任务（task_server.py 的 machine_memory_problem；文件头 ②「内存」的整机内存线）：整机 100G、swap 10G，线是 12%。
    可用 9G（9%）、swap 空闲 0.5G（5%）：不派、原因以「整机内存线」起头，写 9.0% 与 5.0% 两个比例；可用 9%、swap 空闲 6G（60%）：仍派；可用 20%、swap 空闲 5%：仍派；
    没有 swap（总量 0）、可用 9%：不派（swap 空闲按 0% 算）；报告里没有 swap 两项：不判。弄坏开关 machine-memory-line-ignored 下不判，这一格红。"""
    fake = machine_memory_scheduler()
    problems = []
    for what, snapshot, held in (("可用 9%、swap 空闲 5%", machine_memory_snapshot(100, 9, 10, 0.5), True), ("可用 9%、swap 空闲 60%", machine_memory_snapshot(100, 9, 10, 6), False),
                                 ("可用 20%、swap 空闲 5%", machine_memory_snapshot(100, 20, 10, 0.5), False), ("可用 9%、没有 swap", machine_memory_snapshot(100, 9, 0, 0), True),
                                 ("报告里没有 swap 两项", machine_memory_snapshot(100, 9), False)):
        text = SERVER.Scheduler.machine_memory_problem(fake, SERVER.LOCAL, snapshot)
        if bool(text) != held or (held and not text.startswith(SERVER.MACHINE_MEMORY_LINE_TEXT)):
            problems.append(f"{what}：应当{'不派、原因以「' + SERVER.MACHINE_MEMORY_LINE_TEXT + '」起头' if held else '照派'}，实际「{text}」")
        if what == "可用 9%、swap 空闲 5%" and ("9.0%" not in text or "5.0%" not in text):
            problems.append(f"{what}：原因应当写出两个比例 9.0% 与 5.0%，实际「{text}」")
    if problems:
        raise SelftestFailure("整机内存线：" + "；".join(problems))
    return "可用 9% 且 swap 空闲 5%：不派、原因写两个比例；swap 空闲 60%、可用 20%、报告里没有 swap：照派；没有 swap 的按空闲 0% 算"


def external_kill_scheduler(work, log_pid):
    """external-kill-recorded-and-retried 的假调度器：本机的尝试进程树里见过 4242、4300；假 journalctl（在 work 下）头一路（-t earlyoom）打一行点名 log_pid 的 earlyoom 日志、-k 不打。"""
    journalctl = os.path.join(work, "fake-journalctl-unit")
    write_executable(journalctl, f'#!/usr/bin/env bash\n[[ " $* " == *" -t earlyoom "* ]] && echo \'sending SIGTERM to process {log_pid} uid 1000 "rustc": badness 1414, VmRSS 9708 MiB\'\nexit 0\n')
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ, TASK_SERVER_JOURNALCTL=journalctl)
    fake.journalctl = journalctl
    fake.service_stopped_attempts = {}
    fake.peer_external_kill, fake.log_content = {}, lambda attempt: 0   # 第二台的尝试问 peer_external_kill_note：这个假世界里第二台的日志里没有那一行
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(process_ids_seen=lambda attempt, result: {4242, 4300}),
                     SERVER.PEER: types.SimpleNamespace(process_ids_seen=lambda attempt, result: None)}
    return fake


def external_kill_unit_problems(world):
    """external-kill-recorded-and-retried 的假调度器那一半：日志点名的进程在这一次尝试的进程树里才记；点名别的进程、服务发过停的信号、结局不是 command-failed、
    单元收到过停的信号、第二台的尝试（进程号读不到）都不记；retry_decision 对 killed-externally 同一上限重排两次、第三次起打回。"""
    problems = []
    now = time.time()
    task, attempt, result = {}, {"id": 1, "host": SERVER.LOCAL, "started_at": now - 10}, {"ended_at": now, "stopped_signal": 0}
    fake = external_kill_scheduler(world.root, 4242)
    note = SERVER.Scheduler.external_kill_note(fake, task, attempt, result, "command-failed")
    if "earlyoom" not in note or "4242" not in note:
        problems.append(f"日志点名进程 4242、它在进程树里：应当记、说明写日志来源与进程号，实际「{note}」")
    others = (("日志点名的进程不在进程树里", external_kill_scheduler(world.root, 5555), task, attempt, result, "command-failed"),
              ("服务发过停的信号", fake, task, dict(attempt, stalled=True), result, "command-failed"),
              ("结局不是 command-failed", fake, task, attempt, result, "memory-cap-hit"),
              ("单元收到过停的信号", fake, task, attempt, dict(result, stopped_signal=15), "command-failed"),
              ("第二台的尝试", fake, task, dict(attempt, host=SERVER.PEER), result, "command-failed"))
    for what, scheduler, found_task, found_attempt, found_result, outcome in others:
        found = SERVER.Scheduler.external_kill_note(scheduler, found_task, found_attempt, found_result, outcome)
        if found:
            problems.append(f"{what}：不该记 killed-externally，实际「{found}」")
    for counts, retried in (({lib.EXTERNAL_KILL_OUTCOME: 1}, True), ({lib.EXTERNAL_KILL_OUTCOME: lib.EXTERNAL_KILL_RETRIES}, True), ({lib.EXTERNAL_KILL_OUTCOME: lib.EXTERNAL_KILL_RETRIES + 1}, False)):
        decision = lib.retry_decision(counts, lib.EXTERNAL_KILL_OUTCOME, 2 * GIBIBYTE, SERVER.LOCAL, "", {})
        if decision.retry != retried or (retried and decision.cap_bytes != 2 * GIBIBYTE):
            problems.append(f"killed-externally 出过 {counts[lib.EXTERNAL_KILL_OUTCOME]} 次：应当{'同一上限重排' if retried else '打回'}，实际 retry={decision.retry}、上限 {decision.cap_bytes}")
    return problems


def cell_external_kill_recorded_and_retried(world):
    """命令被不是服务发的信号杀掉、earlyoom 或内核日志里有它的进程号：记 killed-externally 并重排（task_server.py 的 external_kill_note、kill_log_lines，task_server_lib.py 的 retry_decision；
    attempt_runner.py 每轮采样把进程树里见过的进程号记进 result.json 的 process_ids）。假调度器那一半（external_kill_unit_problems）：判法的各支。
    服务那一半：假 journalctl（只在世界目录的 earlyoom-pid 在时才打一行点名它的 earlyoom 日志）；一条命令把自己的进程号写进 earlyoom-pid、睡 2 秒后头一次退 1、第二次退 0：
    头一次尝试记 killed-externally（尝试记 external_kill_note）、第二次跑成，共两次；再一条睡 2 秒退 1、进程号不在日志里的：记 command-failed、不重排，一次尝试。
    弄坏开关 external-kill-unrecognized 下头一条照退出码记 command-failed、不重排，这一格红。"""
    problems = external_kill_unit_problems(world)
    raise_cell_problems(world, problems, "被外部信号杀掉（假调度器）")
    world.start_server(extra={"TASK_SERVER_JOURNALCTL": os.path.join(world.bin, "fake-journalctl-earlyoom")})
    pid_file = os.path.join(world.world, "earlyoom-pid")
    flag = os.path.join(world.root, "external-kill-flag")
    killed = world.submit(["bash", "-c", EXTERNAL_KILL_COMMAND, "external-kill", pid_file, flag])
    killed_summary = world.finished(killed)
    unlogged = world.submit(["bash", "-c", "sleep 2; exit 1", "external-kill-unlogged"])
    unlogged_summary = world.finished(unlogged)
    outcomes = [attempt.get("outcome") for attempt in killed_summary["attempts"]]
    first_note = (killed_summary["attempts"] or [{}])[0].get("external_kill_note") or ""
    if outcomes != [lib.EXTERNAL_KILL_OUTCOME, "succeeded"] or killed_summary["task"].get("outcome") != "succeeded" or "earlyoom" not in first_note:
        problems.append(f"日志点名了这一次尝试的进程：头一次应当记 {lib.EXTERNAL_KILL_OUTCOME}（尝试记 external_kill_note 写 earlyoom）、重排一次、第二次跑成，"
                        f"实际各次 {outcomes}、说明「{first_note}」、{describe(killed_summary)}")
    unlogged_outcomes = [attempt.get("outcome") for attempt in unlogged_summary["attempts"]]
    if unlogged_outcomes != ["command-failed"] or unlogged_summary["task"].get("exit_code") != 1:
        problems.append(f"日志里没有这一次尝试的进程号：应当仍记 command-failed、退 1、不重排，实际各次 {unlogged_outcomes}、{describe(unlogged_summary)}")
    raise_cell_problems(world, problems, "被外部信号杀掉（服务）")
    return f"日志点名的进程在进程树里：记 {lib.EXTERNAL_KILL_OUTCOME}、重排一次跑成；日志里没有的仍记 command-failed、不重排；假调度器上各支（服务发过停的、第二台的、单元收到停的）都不记，重排至多 {lib.EXTERNAL_KILL_RETRIES} 次"


PEER_KILL_LINE = ('MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=earlyoom pid=4242 line=sending SIGTERM to process 4242 uid 1000 "rustc": badness 1414, VmRSS 9708 MiB')   # external-kill-peer-recorded：第二台那一行
# 那一格内存组到了它自己上限的那一行（内核「Memory cgroup out of memory」：命令自己造的小内存组让进程撞上限，不是整机内存紧张）与整机 OOM 那一行（本机的认法对照）
PEER_CGROUP_KILL_LINE = ("MULTI_HOST_RUN_PEER_EXTERNAL_KILL source=kernel pid=4243 line=Memory cgroup out of memory: Killed process 4243 (python3) total-vm:812345kB, "
                         "anon-rss:65536kB, file-rss:0kB, shmem-rss:0kB, UID:1000 pgtables:400kB oom_score_adj:500")
MACHINE_OOM_KILL_LINE = "Out of memory: Killed process 4244 (rustc) total-vm:9812345kB, anon-rss:9708000kB, file-rss:0kB, shmem-rss:0kB, UID:1000 pgtables:20000kB oom_score_adj:0"


def peer_external_kill_scheduler(world, name, lines):
    """external-kill-peer-recorded 的假调度器：第二台的一次尝试（id 7），日志是 lines 写成的文件，经真的 log_content 记表（peer_external_kill），再问 external_kill_note。"""
    log = os.path.join(world.root, f"peer-external-kill-{name}.log")
    with open(log, "w", encoding="utf-8") as handle:
        handle.write("".join(f"{line}\n" for line in lines))
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.service_stopped_attempts = {}
    for field in ("log_offsets", "peer_memory", "peer_memory_stat", "attempt_progress", "attempt_progress_baseline", "peer_io_wait",
                  "peer_external_kill", "peer_io_rounds", "peer_thread_rounds"):
        setattr(fake, field, {})
    fake.machines = {SERVER.PEER: SERVER.PeerMachine(fake)}
    return fake, {"id": 7, "task_id": 77, "number": 1, "host": SERVER.PEER, "label": f"task-selftest-peer-kill-{name}", "log": log, "started_at": time.time() - 10}


def cell_external_kill_peer_line_recorded_and_retried(world):
    """被外部信号杀掉的尝试第二台也认，同本机一套判法（task_server.py 的 log_content、peer_external_kill_note、external_kill_note；scripts/hosts/multi-host-run.sh 在第二台那一侧记下这一趟进程树
    的进程号、命令退非 0 之后读第二台的 earlyoom 与内核日志、点名这些进程号之一被杀时打一行 MULTI_HOST_RUN_PEER_EXTERNAL_KILL）：第二台的尝试日志里有这一行、结局 command-failed、
    服务没发过停的信号：记 killed-externally（说明写第二台、来源、被杀的进程号），retry_decision 对第二台同一上限重排；日志里没有这一行的、结局不是 command-failed 的、
    服务发过停的信号的都不记；那一行是内核「Memory cgroup out of memory」（某个内存组到了它自己的上限）的不记，本机的认法（lib.external_kill_pid）同样不交它的进程号、
    整机 OOM 的「Out of memory: Killed process」照交。假调度器上跑，不起服务。
    弄坏开关 peer-external-kill-line-ignored、external-kill-unrecognized 下有这一行的也不记，memory-cgroup-kill-counted 下内存组那一行也记，这一格红。"""
    problems = []
    task, result = {}, {"stopped_signal": 0}
    fake, attempt = peer_external_kill_scheduler(world, "line", ["第二台那条命令的输出", PEER_KILL_LINE, "第二台那条命令退 143"])
    note = SERVER.Scheduler.external_kill_note(fake, task, attempt, result, "command-failed")
    if "第二台" not in note or "earlyoom" not in note or "4242" not in note:
        problems.append(f"第二台日志里有 MULTI_HOST_RUN_PEER_EXTERNAL_KILL 那一行：应当记 killed-externally、说明写第二台、来源与进程号 4242，实际「{note}」")
    unlogged_fake, unlogged_attempt = peer_external_kill_scheduler(world, "none", ["第二台那条命令的输出"])   # 没有那一行的尝试用它自己的日志文件，不借有那一行的那份
    for what, scheduler, found_attempt, outcome in (("日志里没有那一行", unlogged_fake, unlogged_attempt, "command-failed"),
                                                     ("结局不是 command-failed", fake, attempt, "memory-cap-hit"),
                                                     ("服务发过停的信号", fake, dict(attempt, stalled=True), "command-failed")):
        found = SERVER.Scheduler.external_kill_note(scheduler, task, found_attempt, result, outcome)
        if found:
            problems.append(f"{what}：不该记 killed-externally，实际「{found}」")
    cgroup_fake, cgroup_attempt = peer_external_kill_scheduler(world, "cgroup", ["第二台那条命令的输出", PEER_CGROUP_KILL_LINE, "第二台那条命令退 1"])
    cgroup_note = SERVER.Scheduler.external_kill_note(cgroup_fake, task, cgroup_attempt, result, "command-failed")
    if cgroup_note:
        problems.append(f"那一行是 Memory cgroup out of memory（内存组到了它自己的上限）：不该记 killed-externally，实际「{cgroup_note}」")
    local_pids = (lib.external_kill_pid(PEER_CGROUP_KILL_LINE.split("line=", 1)[1]), lib.external_kill_pid(MACHINE_OOM_KILL_LINE))
    if local_pids != (None, 4244):
        problems.append(f"本机的认法：Memory cgroup out of memory 那一行应当不交进程号、整机 Out of memory 那一行交 4244，实际 {local_pids}")
    decision = lib.retry_decision({lib.EXTERNAL_KILL_OUTCOME: 1}, lib.EXTERNAL_KILL_OUTCOME, 2 * GIBIBYTE, SERVER.PEER, "", {})
    if not decision.retry or decision.cap_bytes != 2 * GIBIBYTE:
        problems.append(f"第二台的 killed-externally 出过 1 次：应当同一上限重排，实际 retry={decision.retry}、上限 {decision.cap_bytes}")
    if problems:
        raise SelftestFailure("第二台被外部信号杀掉：" + "；".join(problems))
    return (f"第二台日志里有 MULTI_HOST_RUN_PEER_EXTERNAL_KILL 那一行：记 {lib.EXTERNAL_KILL_OUTCOME}、同一上限重排；没有那一行、结局不是 command-failed、服务发过停的信号的、"
            "那一行是 Memory cgroup out of memory 的都不记；本机的认法同样不认内存组那一行、认整机 OOM 那一行")


def cell_confirm_sheet_reads_bash_c_inner_commands(world):
    """确认单读 bash -c 里命令位置上的脚本与参数（task_server_lib.py 的 shell_inner_commands、named_scope、crash_scope_text，task_server.py 的 scope_is_whole；合入后验证常被包在 bash -c 里，
    先现生成补丁再合入）：bash -c 里 merge-and-verify.sh 给了补丁目录的不是整包（scope_is_whole 假）、没给补丁目录的是整包（真）、带 --all-packages 的是整包；
    重定向的目标不当补丁目录（`merge-and-verify.sh > out.log` 仍是没给）；`cd … && crash-point-replay.sh --item crash-case:a --item crash-case:b --crash-expand-up-to 5` 点名 2 项（named_scope）、
    确认单「崩溃用例与域」写「用例 2 条（命令里点名）；域 --crash-expand-up-to 5」；不是 -c 的命令不钻（shell_inner_commands 交空表）。套在假调度器上，不起服务。
    弄坏开关 shell-inner-scope-unread 下 bash -c 里的范围一律认不出（项数 0、整包的也判成脚本自己定范围），这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peer_root = dict(os.environ), lib.REPOSITORY_ROOT
    fake.cargo_runs = lambda task: []
    problems = []

    def whole(command):
        return SERVER.Scheduler.scope_is_whole(fake, {"command": command, "cwd": lib.REPOSITORY_ROOT})

    patch_form = ["bash", "-c", "python3 scripts/gate/patch-of-the-current-tree.py --out p.patch > /dev/null; bash scripts/gate/merge-and-verify.sh p.patch 2>&1 | tail -n 5"]
    if whole(patch_form):
        problems.append("bash -c 里 merge-and-verify.sh 给了补丁目录：不是整包，实际判成整包")
    for what, command in (("bash -c 里 merge-and-verify.sh 没给补丁目录", ["bash", "-c", "bash scripts/gate/merge-and-verify.sh"]),
                          ("bash -c 里 merge-and-verify.sh 带 --all-packages", ["bash", "-c", "bash scripts/gate/merge-and-verify.sh p.patch --all-packages"]),
                          ("bash -c 里 merge-and-verify.sh 只有重定向的目标", ["bash", "-c", "bash scripts/gate/merge-and-verify.sh > out.log"])):
        if not whole(command):
            problems.append(f"{what}：应当是整包，实际判成脚本自己定范围")
    item_form = ["bash", "-c", "cd /somewhere && bash scripts/crash/crash-point-replay.sh --item crash-case:a --item crash-case:b --crash-expand-up-to 5"]
    named = lib.named_scope(item_form)
    crash_text = lib.crash_scope_text(item_form, lib.REPOSITORY_ROOT)
    if len(named) != 2 or crash_text != "用例 2 条（命令里点名）；域 --crash-expand-up-to 5":
        problems.append(f"bash -c 里 crash-point-replay.sh 点了 2 个 --item 与 --crash-expand-up-to 5：项数应当是 2、崩溃用例与域写「用例 2 条（命令里点名）；域 --crash-expand-up-to 5」，实际 {named}、「{crash_text}」")
    if lib.shell_inner_commands(["bash", "scripts/gate/merge-and-verify.sh", "p.patch"]):
        problems.append("不是 -c 的命令不钻 bash 的脚本：shell_inner_commands 应当交空表")
    if problems:
        raise SelftestFailure("确认单读 bash -c 里的合入后验证：" + "；".join(problems))
    return "bash -c 里的合入后验证：给了补丁目录的不是整包、没给与带 --all-packages 的是整包、重定向目标不当补丁；crash-point-replay.sh 点的 --item 与域读得到"


def cell_workspace_writer_cap_has_headroom(world):
    """换文件、打补丁的工具内存上限留余量（task_server.py 的 workspace_writer_floor、memory_floor_applied、workspace_swap_tool；撞上限被杀会把换到一半的结局文件留在主工作区，
    之后所有会话的换上都被挡）：记录的峰值 600 MiB 的预测，换钩子（scripts/gate/hook-swap.py）与打补丁（scripts/gate/apply-writer-patch.py）的上限与记账不低于
    WORKSPACE_WRITER_FLOOR_BYTES（4 GiB），预测的峰值照原样；同样的预测，别的脚本、hook-swap.py 带 --feed-only（只喂不换）、apply-writer-patch.py 带 --dry-run（只读）、
    合入后验证（scripts/gate/merge-and-verify.sh，它的内存照这把键的记录定，cap-hit-rerun-from-record 钉着）只取内存下限 MEMORY_FLOOR_BYTES（1 GiB）。套在假调度器上，不起服务。
    弄坏开关 workspace-writer-cap-unpadded 下换钩子的也只到 1 GiB，recorded-cap-floored 下下限抬到 4G、只取下限的那几样兜到 4096 MiB，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.peer_root = {name: value for name, value in os.environ.items() if name != lib.MEMORY_FLOOR_VARIABLE}, lib.REPOSITORY_ROOT
    prediction = loaded_host_dispatch().Prediction("selftest-writer-key", 600 * MEBIBYTE, 600 * MEBIBYTE, 600 * MEBIBYTE, lib.RECORD_HISTORY)
    problems = []
    for what, command, wanted in (("换钩子", ["python3", "scripts/gate/hook-swap.py", "--spec", "x.json"], SERVER.WORKSPACE_WRITER_FLOOR_BYTES),
                                  ("打补丁", ["python3", "scripts/gate/apply-writer-patch.py", "p.patch"], SERVER.WORKSPACE_WRITER_FLOOR_BYTES),
                                  ("别的脚本", ["python3", "scripts/gate/other-tool.py"], lib.MEMORY_FLOOR_BYTES),
                                  ("打补丁只读", ["python3", "scripts/gate/apply-writer-patch.py", "--dry-run", "p.patch"], lib.MEMORY_FLOOR_BYTES),
                                  ("合入后验证", ["bash", "scripts/gate/merge-and-verify.sh", "p.patch"], lib.MEMORY_FLOOR_BYTES),
                                  ("换钩子只喂不换", ["python3", "scripts/gate/hook-swap.py", "--feed-only", "--spec", "x.json"], lib.MEMORY_FLOOR_BYTES)):
        applied = SERVER.Scheduler.memory_floor_applied(fake, prediction, {"command": command, "cwd": lib.REPOSITORY_ROOT})
        if applied.memory_cap != wanted or applied.memory_booked != wanted or applied.memory_predicted != 600 * MEBIBYTE:
            problems.append(f"{what}：600 MiB 的预测应当取上限与记账 {wanted // MEBIBYTE} MiB、预测的峰值仍 600 MiB，实际上限 {applied.memory_cap // MEBIBYTE} MiB、"
                            f"记账 {applied.memory_booked // MEBIBYTE} MiB、预测 {applied.memory_predicted // MEBIBYTE} MiB")
    if problems:
        raise SelftestFailure("换文件、打补丁的工具内存上限留余量：" + "；".join(problems))
    return "换钩子与打补丁的上限与记账取 4 GiB（预测峰值不变），别的脚本、只喂不换的、只读的打补丁、合入后验证只取 1 GiB 的下限"


def key_stats_seeder(world):
    """control key-stats 那几格往库里记尝试的帮手：每记一次结束时刻往后推 100 秒（起跑是结束前 1 秒，seed_attempt），交 run(键, 结局, 峰值 MiB, 任务号, 第几次, 峰值口径) → 这一次的起跑时刻。"""
    clock = [SEEDED_ENDED_AT]

    def run(key, outcome, peak=None, task_id=0, number=1, metric=lib.PROCESS_MEMORY_PEAK_METRIC):
        clock[0] += 100
        fields = {"command_key": key, "outcome": outcome, "ended_at": clock[0], "task_id": task_id, "number": number}
        if peak is not None:
            fields["memory_peak"] = peak * MEBIBYTE
        world.seed_attempt(fields, metric=metric)
        return clock[0] - 1
    return run


def cell_control_key_stats_reported(world):
    """control key-stats 只读打命令键的统计（task-server/control 的 key_stats_report，task_server_lib.py 的 key_stats、key_stats_lines；records/2026-11-17-subagent账本自上而下清欠.md 八十节 ①④ 回看要数的几样）：
    库里造六把键，每次尝试各是一个任务（任务号各不相同、都是第 1 次）——A 撞上限、跑成、撞上限、跑成（交替；撞过上限而最近一个任务跑成；跑成两次峰值各 100 MiB）；B 撞上限、跑成、跑成；
    C 跑成三次峰值 100、150、200 MiB；D 只跑过一次；E 跑成两次峰值 100、110 MiB；F 键里带工作根下副本路径、只跑过一次。读数：首行筛前筛后都是 14 行（没给 --since）、进统计的 14 行；
    命令键 6 个（只跑过一次的 2 个）、带副本路径的 1 个；按任务数交替的键 1 个；按任务数撞过上限、最近一个任务跑成的键 2 个；按尝试数交替的键 1 个；
    跑成两次以上的 4 个键最大比最小：中位 1.00 倍、第 90 百分位 2.00 倍、到 1.5 倍以上的 1 个；不打告警那一行。服务在跑时用（库只读打开），退 0。
    弄坏开关 key-stats-alternation-ignored 下按任务数交替的键数成 0，这一格红。"""
    world.start_server()
    run, task = key_stats_seeder(world), [0]

    def one(key, outcome, peak=None):
        task[0] += 1
        run(key, outcome, peak, task_id=task[0])

    for outcome, peak in (("memory-cap-hit", None), ("succeeded", 100), ("memory-cap-hit", None), ("succeeded", 100)):
        one("selftest-stats-a", outcome, peak)
    for outcome, peak in (("memory-cap-hit", None), ("succeeded", 100), ("succeeded", 100)):
        one("selftest-stats-b", outcome, peak)
    for peak in (100, 150, 200):
        one("selftest-stats-c", "succeeded", peak)
    one("selftest-stats-d", "succeeded", 100)
    for peak in (100, 110):
        one("selftest-stats-e", "succeeded", peak)
    one("selftest-stats-copy-singlefs/work/draft/f", "succeeded", 100)
    completed = world.control(["key-stats"])
    text = completed.stdout + completed.stderr
    problems = []
    if completed.returncode != 0:
        problems.append(f"只读的 key-stats 应当退 0，实际退 {completed.returncode}：「{text.strip()[-300:]}」")
    for wanted in ("attempts 筛前 14 行、筛后 14 行（没给 --since，不筛起跑时刻）", f"峰值口径是「{lib.PROCESS_MEMORY_PEAK_METRIC}」的）14 行",
                   "命令键 6 个（只跑过一次的 2 个），其中带工作根下副本或草稿目录路径的 1 个", "近 5 个任务里撞上限与跑成交替至少两回的键 1 个",
                   "近 5 个任务里撞过上限、最近一个任务跑成的键 2 个", "按尝试数：近 5 次尝试里撞上限与跑成交替至少两回的键 1 个",
                   "中位 1.00 倍、第 90 百分位 2.00 倍", "到 1.5 倍以上的 1 个"):
        if wanted not in text:
            problems.append(f"读数里应当有「{wanted}」")
    if "告警" in text:
        problems.append("读数里不该有告警那一行（服务不写这种告警）")
    if problems:
        raise SelftestFailure("control key-stats：" + "；".join(problems) + f"；实际输出：{text.strip()[-800:]}", world.server_log_lines())
    return "key-stats 只读打出：筛前筛后 14 行、6 把键（单跑 2、副本路径 1）、按任务数交替 1、撞过又跑成 2、按尝试数交替 1、峰值离散度中位 1.00 倍与第 90 百分位 2.00 倍、1.5 倍以上 1 个"


def cell_control_key_stats_by_task(world):
    """control key-stats 只取这一版峰值口径的尝试、交替按任务数（task_server_lib.py 的 key_stats、key_stats_after、current_peak_metric；
    research/prompts/ledger80-key-records-r1-main-verification.md「三、改法」第 2 条）：库里造四把键——
    「中间重试」任务甲跑成、任务乙第一次撞上限重试跑成、任务丙跑成（按尝试数成撞成成、交替；按任务数成撞成、也交替）；
    「起头重试」任务甲第一次撞、重试成，之后两个任务都成（按尝试数撞成成成、按任务数撞成成，都不交替；按任务数撞过而最近一个任务跑成）；
    「只有重试」两个任务都是第一次撞、重试成（按尝试数撞成撞成、交替；按任务数撞撞，不交替）；「老口径」三个任务撞、成、撞，尝试没有峰值口径一栏（不进统计）。
    读数：筛前筛后 15 行、进统计的 12 行；命令键 3 个；按任务数交替的键 1 个（中间重试）；按任务数撞过、最近一个任务跑成的 2 个；按尝试数交替的 2 个。
    --since 给「起头重试」第一次尝试的起跑时刻：筛后 11 行、进统计的 8 行、命令键 2 个、按任务数交替的 0 个；--since 给种下的那一天的下一天（YYYY-MM-DD）筛后 0 行；认不出的 --since 退 2。
    弄坏开关 key-stats-metric-unfiltered 下老口径的进统计（命令键 4 个），key-stats-retries-unfolded 下重试不折（按任务数交替的 2 个），
    key-stats-alternation-ignored 下按任务数交替的 0 个，key-stats-since-ignored 下 --since 不筛，这一格红。"""
    world.start_server()
    run = key_stats_seeder(world)
    for task_id, number, outcome in ((101, 1, "succeeded"), (102, 1, "memory-cap-hit"), (102, 2, "succeeded"), (103, 1, "succeeded")):
        run("selftest-stats-retry-middle", outcome, task_id=task_id, number=number)
    since = None
    for task_id, number, outcome in ((201, 1, "memory-cap-hit"), (201, 2, "succeeded"), (202, 1, "succeeded"), (203, 1, "succeeded")):
        started = run("selftest-stats-retry-first", outcome, task_id=task_id, number=number)
        since = started if since is None else since
    for task_id, number, outcome in ((301, 1, "memory-cap-hit"), (301, 2, "succeeded"), (302, 1, "memory-cap-hit"), (302, 2, "succeeded")):
        run("selftest-stats-retries-only", outcome, task_id=task_id, number=number)
    for task_id, outcome in ((401, "memory-cap-hit"), (402, "succeeded"), (403, "memory-cap-hit")):
        run("selftest-stats-old-metric", outcome, task_id=task_id, metric=None)
    metric = f"峰值口径是「{lib.PROCESS_MEMORY_PEAK_METRIC}」的）"
    next_day = time.strftime("%Y-%m-%d", time.gmtime(SEEDED_ENDED_AT + 86400))
    problems = []
    for what, arguments, wanted in (
            ("不筛起跑时刻", [], ("attempts 筛前 15 行、筛后 15 行", f"{metric}12 行", "命令键 3 个（只跑过一次的 0 个）",
                                  "近 5 个任务里撞上限与跑成交替至少两回的键 1 个（按任务数：同一任务的几次尝试折成一条，取第一次尝试的结局）：selftest-stats-retry-middle",
                                  "近 5 个任务里撞过上限、最近一个任务跑成的键 2 个", "按尝试数：近 5 次尝试里撞上限与跑成交替至少两回的键 2 个")),
            ("--since 起头重试的第一次", ["--since", f"{since:.0f}"], ("attempts 筛前 15 行、筛后 11 行", f"{metric}8 行", "命令键 2 个",
                                                                     "近 5 个任务里撞上限与跑成交替至少两回的键 0 个")),
            (f"--since {next_day}", ["--since", next_day], ("attempts 筛前 15 行、筛后 0 行", f"{metric}0 行", "命令键 0 个"))):
        completed = world.control(["key-stats", *arguments])
        text = completed.stdout + completed.stderr
        missing = [line for line in wanted if line not in text]
        if completed.returncode != 0 or missing:
            problems.append(f"{what}：应当退 0、读数里有{'、'.join('「' + line + '」' for line in missing)}，实际退 {completed.returncode}、输出「{text.strip()[-600:]}」")
    refused = world.control(["key-stats", "--since", "yesterday"])
    if refused.returncode != lib.USAGE_EXIT or "--since" not in refused.stderr:
        problems.append(f"认不出的 --since 应当退 {lib.USAGE_EXIT}、点名 --since，实际退 {refused.returncode}、「{(refused.stdout + refused.stderr).strip()[-300:]}」")
    if problems:
        raise SelftestFailure("control key-stats 按任务数、筛口径与起跑时刻：" + "；".join(problems), world.server_log_lines())
    return ("按任务数：中间重试的交替、起头重试与只有重试的不交替，按尝试数交替 2 个；老口径的键不进统计；--since 筛到 11 行、下一天筛到 0 行；认不出的 --since 退 2")


def cell_control_key_stats_dump(world):
    """control key-stats --dump 只读导出收尾了的尝试（task-server/control 的 key_stats_dump，task_server_lib.py 的 key_stats_dump_rows、KEY_STATS_DUMP_COLUMNS；
    research/prompts/ledger80-key-records-r1-main-verification.md「三、改法」第 3 条）：服务在跑，库里造 3 条尝试（任务 11 跑成、没记上限；任务 12 第一次撞上限、记了 stalled_at_cap，
    第二次跑成、记了 shard_count 与 item_plan）：导出 3 行，每行的列与 KEY_STATS_DUMP_COLUMNS 一样、次序一样，任务号与第几次照记的，尝试没有的栏写 null；
    同一个文件再导退 2、文件一个字节不动；--since 给任务 12 第一次的起跑时刻导到新文件里只有 2 行。
    弄坏开关 key-stats-dump-overwrites 下已在的照覆盖、退 0，key-stats-since-ignored 下 --since 那一份有 3 行，这一格红。"""
    world.start_server()
    world.seed_attempt({"command_key": "selftest-dump-a", "outcome": "succeeded", "exit_code": 0, "memory_peak": 100 * MEBIBYTE, "task_id": 11, "ended_at": SEEDED_ENDED_AT + 100})
    world.seed_attempt({"command_key": "selftest-dump-b", "outcome": "memory-cap-hit", "exit_code": 250, "memory_cap": GIBIBYTE, "stalled_at_cap": True,
                        "task_id": 12, "ended_at": SEEDED_ENDED_AT + 200})
    world.seed_attempt({"command_key": "selftest-dump-b", "outcome": "succeeded", "exit_code": 0, "memory_peak": 300 * MEBIBYTE, "shard_count": 4,
                        "item_plan": {"selftest-item": {"to_judge": 1, "reusable": 2}}, "task_id": 12, "number": 2, "ended_at": SEEDED_ENDED_AT + 300})
    since = SEEDED_ENDED_AT + 200 - 1   # 任务 12 第一次的起跑时刻（seed_attempt：结束前 1 秒）
    path, narrowed = os.path.join(world.root, "key-stats-dump.jsonl"), os.path.join(world.root, "key-stats-dump-since.jsonl")
    problems = []
    completed = world.control(["key-stats", "--dump", path])
    rows = []
    with contextlib.suppress(OSError, ValueError):
        with open(path, encoding="utf-8") as handle:
            rows = [json.loads(line) for line in handle]
    columns = list(lib.KEY_STATS_DUMP_COLUMNS)
    if completed.returncode != 0 or len(rows) != 3 or any(list(row) != columns for row in rows):
        problems.append(f"导出应当退 0、写 3 行、每行的列依次是 {'、'.join(columns)}，实际退 {completed.returncode}、{len(rows)} 行、"
                        f"列 {[list(row) for row in rows][:1]}、输出「{(completed.stdout + completed.stderr).strip()[-300:]}」")
    elif ([(row["task_id"], row["number"], row["command_key"]) for row in rows] != [(11, 1, "selftest-dump-a"), (12, 1, "selftest-dump-b"), (12, 2, "selftest-dump-b")]
          or rows[1]["stalled_at_cap"] is not True or rows[2]["shard_count"] != 4 or rows[0]["stalled_at_cap"] is not None
          or rows[2]["item_plan"] != {"selftest-item": {"to_judge": 1, "reusable": 2}} or rows[0]["memory_cap"] is not None
          or rows[0]["memory_peak"] != 100 * MEBIBYTE or rows[0]["peak_metric"] != lib.PROCESS_MEMORY_PEAK_METRIC or not all(isinstance(row["id"], int) for row in rows)):
        problems.append(f"导出的内容不对：应当按行号排、任务 11、12、12，记了的栏照记、没有的写 null，实际 {rows}")
    with open(path, "rb") as handle:
        before = handle.read()
    again = world.control(["key-stats", "--dump", path])
    with open(path, "rb") as handle:
        after = handle.read()
    if again.returncode != lib.USAGE_EXIT or after != before:
        problems.append(f"文件已在时应当退 {lib.USAGE_EXIT}、一个字节不动，实际退 {again.returncode}、文件{'没动' if after == before else '变了'}")
    narrowed_run = world.control(["key-stats", "--since", f"{since:.0f}", "--dump", narrowed])
    kept = []
    with contextlib.suppress(OSError, ValueError):
        with open(narrowed, encoding="utf-8") as handle:
            kept = [json.loads(line) for line in handle]
    if narrowed_run.returncode != 0 or [(row["task_id"], row["number"]) for row in kept] != [(12, 1), (12, 2)]:
        problems.append(f"--since 给任务 12 第一次的起跑时刻，应当只导任务 12 那 2 行，实际退 {narrowed_run.returncode}、{[(row.get('task_id'), row.get('number')) for row in kept]}")
    if problems:
        raise SelftestFailure("control key-stats --dump：" + "；".join(problems), world.server_log_lines())
    return "服务在跑：导出 3 行、列齐、次序对、没有的栏写 null；文件已在退 2 不动；--since 只导 2 行"


def ledger_fake():
    """booking-ledger-rows 的假调度器与 Snapshot：本机 slice 总额 50G、余量 1G，在跑五条——L1（上限 20G，scope 用 5G）、L2（上限 12G，报告里没有它的 scope）、
    L3（上限 5G，单元已确认停了）、L4（上限 2G，scope 用满 2G）、L5（上限 10G，scope 用 4G）。"""
    attempts = [{"id": number, "task_id": 100 + number, "number": 1, "host": SERVER.LOCAL, "label": f"L{number}", "memory_cap": cap * GIBIBYTE}
                for number, cap in ((1, 20), (2, 12), (3, 5), (4, 2), (5, 10))]
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.peaks = PEAKS
    fake.machines = {SERVER.LOCAL: types.SimpleNamespace(label="本机")}
    fake.store = types.SimpleNamespace(running_attempts=lambda: [dict(attempt) for attempt in attempts])
    fake.host_margin = lambda host, snapshot: (GIBIBYTE, "")
    fake.unit_stopped = {3}
    fake.booking_drift_lock = threading.Lock()
    fake.booking_ledger = {}
    report = memory_report(30, 50, scopes={"L1": 5 * GIBIBYTE, "L4": 2 * GIBIBYTE, "L5": 4 * GIBIBYTE})
    return fake, types.SimpleNamespace(reports={SERVER.LOCAL: report}, total=lambda host: 50 * GIBIBYTE)


def cell_booking_ledger_rows(world):
    """在跑记账的逐条明细（task_server.py 的 ledger_row、running_booking_text、refresh_booking_ledger、booking_ledger_snapshot；task-server/control bookings 打出来）：
    假世界里五条各走一支——L1 此刻实测 5G，记 5G（measured）；L2 报告里没有它的 scope，记整份上限 12G（no-scope）；L3 单元已停，不计（0，stopped）；
    L4 实测 2G 到了它的上限 2G，记上限 2G（cap、at_cap）；L5 此刻实测 4G，记 4G（measured）。每条的原因写清取的是哪一个与几个数。
    合计记 23G，照公式（50G − 1G − 23G）放得下 26G，在跑上限之和 49G，各 scope 实占合计 11G。
    control bookings 照 /health 的这一栏逐台打：头一行写 slice 总额、余量与上限之和，每条一行写上限、实占、记多少、取的是哪一个与原因，末行写合计与放得下多少。
    弄坏开关 bookings-ledger-unreported 下这一栏是空表，running-booked-at-cap 下 L1 记 20G，running-booked-with-margin 下 L1 记 6G，这一格红。"""
    import importlib.machinery
    import importlib.util
    now = time.time()
    fake, snapshot = ledger_fake()
    SERVER.Scheduler.refresh_booking_ledger(fake, snapshot, now)
    entry = SERVER.Scheduler.booking_ledger_snapshot(fake).get(SERVER.LOCAL) or {}
    rows = {row["label"]: row for row in entry.get("rows") or []}
    problems = []
    wanted_rows = (("L1", SERVER.BOOKED_FROM_USAGE, 5, False, ("此刻实测的用量 5.0G",)),
                   ("L2", SERVER.BOOKED_WITHOUT_SCOPE, 12, True, ("没有它的 scope", "12.0G")), ("L3", "stopped", 0, False, ("单元已确认停了",)),
                   ("L4", SERVER.BOOKED_FROM_CAP, 2, True, ("到了它的上限", "记上限 2.0G")),
                   ("L5", SERVER.BOOKED_FROM_USAGE, 4, False, ("此刻实测的用量 4.0G",)))
    for label, basis, booked, at_cap, words in wanted_rows:
        row = rows.get(label) or {}
        if row.get("basis") != basis or row.get("booked") != booked * GIBIBYTE or row.get("at_cap") != at_cap or not all(word in (row.get("text") or "") for word in words):
            problems.append(f"{label}：应当 {basis}、记 {booked}G、at_cap {at_cap}、原因写 {'、'.join(words)}，实际 {row}")
    if entry.get("booked") != 23 * GIBIBYTE or entry.get("room") != 26 * GIBIBYTE or entry.get("caps") != 49 * GIBIBYTE or entry.get("used") != 11 * GIBIBYTE:
        problems.append(f"合计应当记 23G、照公式放得下 26G、上限之和 49G、实占合计 11G，实际 记 {entry.get('booked')}、放得下 {entry.get('room')}、上限之和 {entry.get('caps')}、实占 {entry.get('used')}")
    if "放得下 26.0G" not in (entry.get("total") or "") or "×" in (entry.get("total") or ""):
        problems.append(f"合计那一行应当写照公式放得下 26.0G、不写乘几，实际「{entry.get('total')}」")
    loader = importlib.machinery.SourceFileLoader("control_for_bookings", lib.CONTROL_SCRIPT)
    control = importlib.util.module_from_spec(importlib.util.spec_from_loader("control_for_bookings", loader))
    loader.exec_module(control)
    health = {"booking_ledger": SERVER.Scheduler.booking_ledger_snapshot(fake)}
    printed = io.StringIO()
    with unittest.mock.patch.object(lib, "require_server", return_value=health), unittest.mock.patch.object(lib, "read_configuration", return_value=None), contextlib.redirect_stdout(printed):
        code = control.bookings()
    lines = [line for line in printed.getvalue().splitlines() if line.strip()]
    if code != 0 or len(lines) != 7 or "slice 总额 50.0G" not in lines[0] or "放得下 26.0G" not in lines[-1] or sum(1 for line in lines if "第 1 次：上限" in line) != 5:
        problems.append(f"control bookings 应当退 0、打 7 行（头一行、五条、末行合计）、头一行写 slice 总额 50.0G、末行写放得下 26.0G，实际退 {code}、{len(lines)} 行：{lines}")
    for label, basis, *_ in wanted_rows:
        if not any(f"任务 {rows.get(label, {}).get('task')} " in line and f"（{basis}）" in line for line in lines):
            problems.append(f"control bookings 应当有一行写 {label} 那条任务与（{basis}），实际 {lines}")
    if problems:
        raise SelftestFailure("在跑记账的逐条明细：" + "；".join(problems))
    return ("五条各走一支：实测 5G 记 5G、没有 scope 记上限 12G、单元已停不计、实测到了上限记上限 2G、实测 4G 记 4G；"
            "合计记 23G、照公式放得下 26G；control bookings 逐条写取的是哪一个与原因、末行写合计")


def cell_confirm_retries_follow_churn(world):
    """自证帮手确认单（World.confirm_submitted）数对不上时按同会话在排在跑的条数放宽重试回数（有界）：服务核数在会话锁里算了就比（task_server.py 的 confirm、counts_mismatch），
    每回对不上都是同会话又有一条收了，机器忙、十几条一齐收尾时三回追不上。假客户端头 8 回回 409（带新的条数，从 9 条一回少一条）、第 9 回回 0：帮手应当确认成功，共 9 回；
    头 14 回回 409（多于回数）、第 15 回回 0：回数用完之后打回的还带着新的确认单，再追到时限（CONFIRM_BUDGET_SECONDS），确认成功共 15 回；
    一直回 409 而时限给 0 的：在 CONFIRM_RETRIES 加头一回条数那么多回之后判红（有界）。弄坏开关 confirm-retries-fixed 下只重试 CONFIRM_RETRIES 回，这一格红。"""
    def needed(tasks):
        return subprocess.CompletedProcess([], 0, f"{lib.CONFIRM_NEEDED_MARK} task=5 items=1 tasks={tasks} timeout=1800\n", "")

    def stub(mismatches):
        calls = []

        def client(arguments, **options):
            calls.append(arguments)
            if len(calls) <= mismatches:
                return subprocess.CompletedProcess([], 3, needed(max(1, 9 - len(calls))).stdout, "mismatch")
            return subprocess.CompletedProcess([], 0, "confirmed", "")
        fake = object.__new__(World)
        fake.client, fake.server_log_lines = client, lambda: []
        return fake, calls
    problems = []
    fake, calls = stub(8)
    reply = fake.confirm_submitted(needed(9))
    if reply is None or reply.returncode != 0 or len(calls) != 9:
        problems.append(f"头 8 回对不上、第 9 回对上：应当确认成功、共 9 回，实际 {len(calls)} 回、{reply}")
    fake, calls = stub(14)
    reply = fake.confirm_submitted(needed(9))
    if reply is None or reply.returncode != 0 or len(calls) != 15:
        problems.append(f"14 回对不上（多于回数 {CONFIRM_RETRIES + 9}）、第 15 回对上：回数用完之后服务打回的还带着新的确认单，应当再追到时限、确认成功、共 15 回，实际 {len(calls)} 回、{reply}")
    fake, calls = stub(1000)
    fake.confirm_budget_seconds = 0
    try:
        fake.confirm_submitted(needed(9))
        problems.append("一直对不上的应当判红")
    except SelftestFailure:
        if len(calls) != CONFIRM_RETRIES + 9:
            problems.append(f"一直对不上：应当在 {CONFIRM_RETRIES + 9} 回之后判红（有界），实际 {len(calls)} 回")
    if problems:
        raise SelftestFailure("确认单重试随同会话条数放宽：" + "；".join(problems))
    return f"同会话条数 9 的时候头 8 回对不上、第 9 回确认成功；头 14 回对不上的追到第 15 回确认成功；一直对不上而时限给 0 的 {CONFIRM_RETRIES + 9} 回判红"


SUBMIT_TRACE_STEPS = ("进线程", "提交时拒", "认会话", "认任务", "记库", "提交时警告", "确认单", "回话")   # submit-trace-logged：一条收下的提交走过的几步，照这个先后
SUBMIT_TRACE_RECEIVED = re.compile(r"\d\d:\d\d:\d\d\.\d{3} 收到；")                                  # 那一行里收到的时刻（时、分、秒与毫秒）


def cell_submit_trace_logged(world):
    """提交那一路的进出日志（task_server.py 的 SubmitTrace、POST /tasks 的 submit 与 submit_in_thread、Scheduler.submit 的 trace）：真服务收下一条提交，服务日志里恰好一行
    以 SUBMIT_TRACE_TEXT 起头、写「任务 <号>，收下」、收到的时刻，与 SUBMIT_TRACE_STEPS 那几步照先后各带距收到多少秒；另在这一格的进程里造一条 SubmitTrace、
    演上一步之后过了 SUBMIT_TRACE_SLOW_SECONDS 以上：那一步当场另打一行「…这一步用了 …秒…回话还没出去」，回话时照样打那一行。
    弄坏开关 submit-trace-silent 下一行都不打，这一格红。"""
    world.start_server()
    task_id = world.submit(["bash", "-c", "exit 0", "submit-trace"])
    done = world.finished(task_id)
    lines = []
    for number in range(1, world.server_logs + 1):
        with open(os.path.join(world.root, f"server-{number}.log"), encoding="utf-8", errors="replace") as handle:
            lines += [line for line in handle.read().splitlines() if SERVER.SUBMIT_TRACE_TEXT in line]
    mine = [line for line in lines if f"任务 {task_id}，收下" in line]
    problems = []
    if done["task"].get("outcome") != "succeeded":
        problems.append(f"那一条应当跑成，实际 {describe(done)}")
    if len(mine) != 1:
        problems.append(f"服务日志里应当恰好一行「{SERVER.SUBMIT_TRACE_TEXT}（任务 {task_id}，收下）」，实际 {len(mine)} 行：{lines[-5:]}")
    else:
        positions = [mine[0].find(f"{step} +") for step in SUBMIT_TRACE_STEPS]
        if not SUBMIT_TRACE_RECEIVED.search(mine[0]) or -1 in positions or positions != sorted(positions):
            problems.append(f"那一行应当写收到的时刻、照先后写 {'、'.join(SUBMIT_TRACE_STEPS)} 各距收到多少秒，实际「{mine[0]}」")
    printed = io.StringIO()
    with contextlib.redirect_stderr(printed):
        trace = SERVER.SubmitTrace(dict(os.environ))
        trace.previous -= SERVER.SUBMIT_TRACE_SLOW_SECONDS + 1   # 演上一步之后过了这么久
        trace.step("提交时拒")
        trace.finish("收下")
    shown = printed.getvalue().splitlines()
    if len(shown) != 2 or "提交时拒这一步用了" not in shown[0] or "回话还没出去" not in shown[0] or SERVER.SUBMIT_TRACE_TEXT not in shown[1] or "回话 +" not in shown[1]:
        problems.append(f"一步用了 {SERVER.SUBMIT_TRACE_SLOW_SECONDS + 1:g} 秒以上：应当当场另打一行写那一步慢、回话时再打进出那一行，实际 {shown}")
    if problems:
        raise SelftestFailure("提交那一路的进出日志：" + "；".join(problems), world.server_log_lines())
    return f"收下的提交在服务日志里恰好一行：收到的时刻与 {'、'.join(SUBMIT_TRACE_STEPS)} 各距收到多少秒；一步慢的当场另打一行"


def cell_store_write_rolls_back_on_error(world):
    """库的写事务出事先回滚（task_server.py 的 Store.write_transaction；health_write、delete_row 这几处都经它）：meta 表上一个触发器让写 health 那一行 RAISE(ABORT)
    （语句回滚、事务还开着，同 SQLITE_BUSY、IO 错那几种），operations 表上一个让删行 RAISE(ABORT)：health_write 交写不进、delete_row 抛出来，
    之后这条连接不在事务里，另一条连接 BEGIN IMMEDIATE 0.5 秒内拿得到写锁。弄坏开关 store-transaction-left-open 下出事不回滚：连接留在事务里、别的连接拿不到写锁，这一格红。"""
    path = os.path.join(world.root, "rollback.sqlite3")
    store = SERVER.Store(path, bytes(32), dict(os.environ))
    row = store.insert("operations", {"kind": "selftest"})
    with contextlib.closing(sqlite3.connect(path, timeout=30, isolation_level=None)) as connection:
        connection.execute("CREATE TRIGGER selftest_health_fails BEFORE UPDATE ON meta WHEN NEW.name = 'health' BEGIN SELECT RAISE(ABORT, 'selftest health write fails'); END")
        connection.execute("CREATE TRIGGER selftest_delete_fails BEFORE DELETE ON operations BEGIN SELECT RAISE(ABORT, 'selftest delete fails'); END")
    problems = []
    for what, action in (("health_write", lambda: store.health_write()), ("delete_row", lambda: store.delete_row("operations", row["id"]))):
        raised = ""
        try:
            outcome = action()
        except sqlite3.Error as error:
            outcome, raised = None, f"{type(error).__name__}: {error}"
        open_after = store.connection().in_transaction
        locked = ""
        with contextlib.closing(sqlite3.connect(path, timeout=0.5, isolation_level=None)) as other:
            try:
                other.execute("BEGIN IMMEDIATE")
                other.execute("ROLLBACK")
            except sqlite3.OperationalError as error:
                locked = str(error)
        failed = (outcome is not None and outcome[0] is False) if what == "health_write" else bool(raised)
        if not failed or open_after or locked:
            problems.append(f"{what} 撞上 RAISE(ABORT)：应当{'交写不进' if what == 'health_write' else '抛出来'}、之后连接不在事务里、别的连接拿得到写锁，"
                            f"实际交 {outcome}{('、抛 ' + raised) if raised else ''}、连接{'还在事务里' if open_after else '不在事务里'}、别的连接{'拿不到写锁（' + locked + '）' if locked else '拿得到写锁'}")
    if problems:
        raise SelftestFailure("写事务出事先回滚：" + "；".join(problems))
    return "health_write 与 delete_row 撞上 RAISE(ABORT) 之后连接不在事务里，别的连接照样拿得到写锁"


DISK_FLOOR_RECORD_BYTES = 35 * GIBIBYTE   # disk-floor-peer-not-never-fits：同键记录的峰值，比本机可用上限（30G 减余量 2G）大、比第二台的（64G 减 2G）小


def cell_disk_floor_peer_not_never_fits(world):
    """第二台盘余量不到下限时记录比本机大的任务排着、不判 never-fits（task_server.py 的 judge_placement、hosts_short_of_record）：同键上一次跑完峰值 35G，
    本机可用上限 28G、第二台 62G 而第二台盘只剩不到下限：任务排着（queued、没有尝试），排队原因写第二台盘只剩；第二台盘够了照派第二台跑完。
    弄坏开关 disk-floor-host-judged-never 下盘不够的第二台不算「回来可能放得下」，本机判永远放不下、任务记 never-fits，这一格红。"""
    key = "selftest-disk-floor-record"
    floor = SERVER.DISK_FLOOR_BYTES
    world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_df=f"     Avail\n{floor - GIBIBYTE}\n")
    world.start_server()
    world.seed_peak(key, DISK_FLOOR_RECORD_BYTES)
    task_id = world.submit(["bash", "-c", "exit 0", "disk-floor-record"], cwd=world.repo, extra={lib.KEY_VARIABLE: key})
    world.next_pass()
    world.next_pass()
    held = world.summary(task_id)
    problems = []
    if held["task"].get("state") != "queued" or held["attempts"] or "第二台盘只剩" not in (held.get("waiting_for") or ""):
        problems.append(f"第二台盘不够、本机放不下：应当排着、没有尝试、排队原因写「第二台盘只剩」，实际 {describe(held)}、排队原因「{held.get('waiting_for')}」、"
                        f"说明「{held['task'].get('note')}」")
    else:
        world.resources(local=(30, 30), peer=(100, 100), peer_cap_gibibytes=64, peer_disk_bytes=floor + GIBIBYTE)
        done = world.finished(task_id, timeout=PLACEMENT_WAIT_SECONDS)
        if done["task"].get("outcome") != "succeeded" or (done["attempts"] or [{}])[-1].get("host") != "peer":
            problems.append(f"第二台盘够了应当照派第二台跑完，实际 {describe(done)}")
    if problems:
        raise SelftestFailure("第二台盘不够时不判 never-fits：" + "；".join(problems), world.server_log_lines())
    return "第二台盘不够、本机放不下 35G 的记录：排着；第二台盘够了派过去跑完"


def peer_log_scheduler(world, name, lines=()):
    """第二台日志那几格的假调度器：第二台的一次尝试（id 7），日志是这一格临时目录里的一份（先写进 lines），经真的 log_content 记表。→ (调度器, 尝试)。"""
    log = os.path.join(world.root, f"peer-log-{name}.log")
    with open(log, "w", encoding="utf-8") as handle:
        handle.write("".join(f"{line}\n" for line in lines))
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    for field in ("log_offsets", "peer_memory", "peer_memory_stat", "peer_io_wait", "peer_threads",
                  "peer_external_kill", "peer_io_rounds", "peer_thread_rounds"):
        setattr(fake, field, {})
    fake.machines = {SERVER.PEER: SERVER.PeerMachine(fake)}
    return fake, {"id": 7, "task_id": 77, "number": 1, "host": SERVER.PEER, "label": f"task-selftest-peer-{name}", "log": log}


def append_lines(path, lines):
    with open(path, "a", encoding="utf-8") as handle:
        handle.write("".join(f"{line}\n" for line in lines))


def cell_stall_reads_log_before_cpu(world):
    """判卡住每一轮开头先把日志读到底再读 CPU（task_server.py 的 fresh_cpu_usec）：第二台的日志里刚到一行 MULTI_HOST_RUN_PEER_CPU usage_usec=5000000、还没被读过：
    读到的 CPU 是 5000000。套在假调度器上，不起服务。弄坏开关 stall-cpu-read-before-log 下不先读日志、交读不到，这一格红。"""
    fake, attempt = peer_log_scheduler(world, "cpu-first", ["MULTI_HOST_RUN_PEER_CPU poll=1 usage_usec=5000000 read=yes"])
    usage = SERVER.Scheduler.fresh_cpu_usec(fake, attempt)
    if usage != 5000000:
        raise SelftestFailure(f"第二台日志里刚到的那一行 CPU 应当读到 5000000，实际 {usage}")
    return "第二台刚到的 CPU 行当轮读到"


def cell_peer_readings_merged_across_reads(world):
    """两次读之间读日志几回，IO 等待与线程读数一行不丢（task_server.py 的 read_log_content、merge_peer_io_rounds、merge_peer_thread_rounds）：读一回日志接到 blocked=0、
    runnable=9 那一行，再读一回接到 blocked=3、runnable=2 那一行，之后 io_blocked 交 0（取最少）、thread_reading 的可跑线程数 9（取最大）。套在假调度器上，不起服务。
    弄坏开关 peer-readings-overwritten 下后一回盖掉前一回（交 3 与 2），这一格红。"""
    fake, attempt = peer_log_scheduler(world, "merged", ["MULTI_HOST_RUN_PEER_IO_WAIT poll=1 blocked=0 read=yes",
                                                         "MULTI_HOST_RUN_PEER_THREADS poll=1 runnable=9 total=20 top=12:cargo read=yes"])
    SERVER.Scheduler.log_content(fake, attempt)
    append_lines(attempt["log"], ["MULTI_HOST_RUN_PEER_IO_WAIT poll=2 blocked=3 read=yes", "MULTI_HOST_RUN_PEER_THREADS poll=2 runnable=2 total=5 top=3:rustc read=yes"])
    SERVER.Scheduler.log_content(fake, attempt)
    machine = fake.machines[SERVER.PEER]
    blocked, threads = machine.io_blocked(attempt), machine.thread_reading(attempt)
    if blocked != 0 or threads is None or threads.runnable != 9:
        raise SelftestFailure(f"两回读到的 IO 等待应当并起来取最少（0）、可跑线程取最大（9），实际 {blocked}、{threads}")
    return "两回读到的 IO 等待并起来取 0、可跑线程取 9"


def cell_log_content_serialized(world):
    """同一次尝试同一时刻只一个线程读日志（task_server.py 的 log_content、LOG_CONTENT_LOCKS）：两个线程同时读（读那一段换成睡 0.3 秒、记同时在读几个的假的）：至多一个在读。
    套在假调度器上，不起服务。弄坏开关 log-content-unlocked 下两个同时读，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    guard, active, peak = threading.Lock(), [0], [0]

    def slow_read(attempt):
        with guard:
            active[0] += 1
            peak[0] = max(peak[0], active[0])
        time.sleep(0.3)
        with guard:
            active[0] -= 1
        return 0
    fake.read_log_content = slow_read
    readers = [threading.Thread(target=SERVER.Scheduler.log_content, args=(fake, {"id": 7})) for _ in range(2)]
    for reader in readers:
        reader.start()
    for reader in readers:
        reader.join(10)
    if peak[0] != 1:
        raise SelftestFailure(f"同一次尝试的日志同一时刻应当只一个线程在读，实际同时 {peak[0]} 个")
    return "两个线程读同一次尝试的日志一个接一个"


def cell_log_noise_marks_not_growth(world):
    """第二台收尾那几行与 multi-host-run.sh 每一轮照打的显存行不算日志在涨（task_server.py 的 read_log_content 的 noise_marks）：日志里只有 MULTI_HOST_RUN_PEER_MEMORY_PEAK、
    MULTI_HOST_RUN_PEER_BUILD_PEAK、MULTI_HOST_RUN_PEER_BUILD_KEPT 与 MULTI_HOST_RUN_PEER_GPU 各一行：算「在涨」的字节数是 0；另接一行命令自己的输出：等于那一行的字节数。
    套在假调度器上，不起服务。弄坏开关 log-noise-marks-incomplete 下收尾那三行照算，这一格红。"""
    lines = [f"{lib.PEER_MEMORY_PEAK_MARK} bytes=1024", f"{lib.PEER_BUILD_PEAK_MARK} bytes=2048", f"{lib.PEER_BUILD_KEPT_MARK} label=task-selftest-1-1",
             "MULTI_HOST_RUN_PEER_GPU poll=1 used_mib=0 read=yes utilization=none on_cards=empty"]
    fake, attempt = peer_log_scheduler(world, "noise-marks", lines)
    quiet = SERVER.Scheduler.log_content(fake, attempt)
    output = "命令自己的一行输出"
    append_lines(attempt["log"], [output])
    grown = SERVER.Scheduler.log_content(fake, attempt)
    if quiet != 0 or grown != len((output + "\n").encode("utf-8")):
        raise SelftestFailure(f"收尾三行与显存行应当都不算在涨（0 字节）、接一行输出之后是那一行的字节数，实际 {quiet}、{grown}")
    return "收尾的峰值行、留着编译目录行与每轮的显存行不算日志在涨"


def cell_runaway_targets_skip_children(world):
    """空转进程的提醒找得到会话（task_server.py 的 runaway_targets）：会话 A 先交一条带收件地址的任务，之后它的分片任务建了 25 个子任务（没有收件地址）：
    找到的是那条带收件地址的任务。套在假调度器上（这一格临时库上的 Store），不起服务。弄坏开关 runaway-targets-counting-children 下子任务占满最近 20 行、找不到，这一格红。"""
    store = SERVER.Store(os.path.join(world.root, "runaway-targets.sqlite3"), bytes(32), dict(os.environ))
    parent = store.insert("tasks", {"task_id": 0, "state": "sharded", "session": SESSION_A, "command_key": "", "inbox": {"token": INBOX_TOKEN_SAMPLE}})
    for index in range(25):
        store.insert("tasks", {"task_id": parent["id"], "parent_id": parent["id"], "role": "shard", "shard_index": index, "state": "queued", "session": SESSION_A, "command_key": ""})
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.store = dict(os.environ), store
    fake.inbox_reachable = lambda task: bool(task.get("inbox"))
    found = [task.get("id") for task in SERVER.Scheduler.runaway_targets(fake, SESSION_A)]
    if found != [parent["id"]]:
        raise SelftestFailure(f"会话最近交了 25 片的分片任务：应当找到带收件地址的任务 {parent['id']}，实际 {found}")
    return "子任务不占最近那 20 行，找得到带收件地址的任务"


def promote_fake(rows):
    """promote-wakes-and-promotes-shards 的假调度器：库是 rows（{任务号: 正文}）上的假 Store（task、change 照真的那样判 guard）。→ (调度器, 叫醒过几回的表)。"""
    def change(table, row_id, changes, guard=None, reopen=False):
        record = dict(rows[row_id])
        why = guard(record) if guard is not None else ""
        if why:
            raise SERVER.StoreRefused("guard", why)
        record.update(changes)
        rows[row_id] = record
        return record
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.store = types.SimpleNamespace(task=lambda task_id: (dict(rows[task_id]), True) if task_id in rows else None, change=change,
                                       insert=lambda table, record: dict(record, id=99))
    fake.children_of = lambda parent_id: [dict(row) for row in rows.values() if row.get("parent_id") == parent_id]
    fake.operation = lambda *arguments: None
    woken = []
    fake.wake = lambda: woken.append(True)
    return fake, woken


def cell_promote_wakes_and_promotes_shards(world):
    """task promote 当场叫醒排队、分片的父任务也提前（task_server.py 的 promote、create_child）：分片的父任务 1（sharded）下一片 2 在排、一片 3 在跑：提前退 200、叫醒一回，
    父任务与在排的 2 记了 promoted，在跑的 3 不记；之后建的子任务带上父任务的 promoted。套在假调度器上，不起服务。
    弄坏开关 promote-unwoken 下不叫醒，promote-shards-refused 下分片的父任务照「不在排」回 409、子任务不继承，这一格红。"""
    rows = {1: {"id": 1, "session": SESSION_A, "state": "sharded"}, 2: {"id": 2, "session": SESSION_A, "state": "queued", "parent_id": 1},
            3: {"id": 3, "session": SESSION_A, "state": "running", "parent_id": 1}}
    fake, woken = promote_fake(rows)
    status, reply = SERVER.Scheduler.promote(fake, 1, SESSION_A, 0)
    fake.find_child = lambda children, role, index: None
    fake.child_key_text = lambda parent, role, index, count: "selftest-promoted-child"
    fake.command_key = lambda command, key: "selftest-promoted-child-key"
    child = SERVER.Scheduler.create_child(fake, dict(rows[1], command=["x"], cwd=world.root, environment={}), "shard", 1, 2)
    problems = []
    if status != 200 or not woken:
        problems.append(f"分片的父任务提前应当退 200、当场叫醒排队，实际 {status}「{reply}」、叫醒 {len(woken)} 回")
    if not rows[1].get("promoted") or not rows[2].get("promoted") or rows[3].get("promoted"):
        problems.append(f"父任务与在排的子任务应当记 promoted、在跑的不记，实际父 {rows[1].get('promoted')}、在排 {rows[2].get('promoted')}、在跑 {rows[3].get('promoted')}")
    if not child.get("promoted"):
        problems.append("提前过的父任务之后建的子任务应当带 promoted，实际没带")
    if problems:
        raise SelftestFailure("task promote 叫醒与分片：" + "；".join(problems))
    return "分片的父任务提前：退 200、叫醒排队，父任务与在排的片记 promoted，之后建的片继承"


def cell_promote_placement_rules(world):
    """提前过的排队规矩（task_server.py 的 aged、blocks_later）：提交 3 小时、提前过的不算排满时限（不挡后面）、起不来不算
    「排在前面还没起的」（后面的不照插队判）；同样 3 小时没提前过的照算排满时限、照算排在前面。套在假调度器上，不起服务。
    弄坏开关 promote-aged-blocks-queue、promote-blocked-counts-as-earlier 下各坏一样，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.aged_seconds = 3600.0
    now = time.time()
    promoted = {"id": 1, "submitted_at": now - 3 * 3600, "promoted": {"at": now - 60}}
    plain = {"id": 2, "submitted_at": now - 3 * 3600}
    seen = {name: (getattr(SERVER.Scheduler, name)(fake, promoted, *extra), getattr(SERVER.Scheduler, name)(fake, plain, *extra))
            for name, extra in (("aged", (now,)), ("blocks_later", ()))}
    wanted = {"aged": (False, True), "blocks_later": (False, True)}
    problems = [f"{name}：提前过的与没提前过的应当是 {wanted[name]}，实际 {found}" for name, found in seen.items() if found != wanted[name]]
    if problems:
        raise SelftestFailure("提前过的排队规矩：" + "；".join(problems))
    return "提前过的不算排满时限、起不来不挡后面"


def cell_pause_keeps_queue_positions(world):
    """暂停放行一个会话时别的会话排着的任务排位不丢（task_server.py 的 place_queued_into、positions_in）：假余量 hold，放行会话 A；会话 B 先交一条、A 后交一条：
    两条的 status 排位照全队算（B 那条 1/2、A 那条 2/2）。弄坏开关 pause-queue-positions-filtered 下只照放行的那几条算（B 那条没有排位、A 那条 1/1），这一格红。"""
    world.resources(local=(30, 30), hold=True)
    world.start_server()
    pause_by_api(world, PAUSE_REASON_SAMPLE, allow_session=SESSION_A)
    other_id = world.submit(["bash", "-c", "exit 0", "paused-other"], session=SESSION_B)
    allowed_id = world.submit(["bash", "-c", "exit 0", "paused-allowed"], session=SESSION_A)
    world.next_pass()
    world.next_pass()
    other, allowed = (world.summary(task_id).get("queue_position") or {} for task_id in (other_id, allowed_id))
    seen = ((other.get("position"), other.get("queued")), (allowed.get("position"), allowed.get("queued")))
    if seen != ((1, 2), (2, 2)):
        raise SelftestFailure(f"暂停放行 A 时两条的排位应当照全队算（B 那条 1/2、A 那条 2/2），实际 {seen}", world.server_log_lines())
    return "暂停放行一个会话：别的会话排着的排位照全队算"


def cell_stop_joins_peer_probe(world):
    """停服务时等第二台的余量探测线程（task_server.py 的 Scheduler.stop）：探测线程见到停的信号之后还要 0.5 秒才退（演它正在收一个 ssh）：停自己的子进程那一步之前它已退了。
    套在假调度器上，不起服务。弄坏开关 peer-probe-thread-unjoined 下不等它，停子进程时它还在，这一格红。"""
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.stopping, fake.inbox_event = threading.Event(), threading.Event()
    fake.wake = lambda: None
    for name in ("thread", "nfs_thread", "inbox_thread", "runaway_thread", "booking_audit_thread"):
        setattr(fake, name, threading.Thread(target=lambda: None))   # 没起：is_alive 为假
    finished = threading.Event()

    def probe():
        fake.stopping.wait(30)
        time.sleep(0.5)
        finished.set()
    fake.peer_probe_thread = threading.Thread(target=probe, daemon=True)
    fake.peer_probe_thread.start()
    fake.children = []
    seen = []
    fake.stop_own_children = lambda: seen.append(finished.is_set())
    SERVER.Scheduler.stop(fake)
    if seen != [True]:
        raise SelftestFailure(f"停自己的子进程之前探测线程应当已退，实际{'没停子进程' if not seen else '还在'}")
    return "停服务等探测线程退了才停子进程"


def cell_peer_cooldown_skips_connect(world):
    """第二台上一回读不成、还在冷却里，收尾那几处不去连（task_server.py 的 Resources.peer_cooling_down、PeerMachine 的 attempt_presence_readings、
    remove_build_directory）：两处都当场交「不再去连」、一次都没去重读第二台。套在假调度器上，不起服务。
    弄坏开关 peer-cooldown-ignored-off-scheduler 下照样去连，这一格红。"""
    resources = object.__new__(SERVER.Resources)
    resources.environment, resources.fake_path, resources.peer = dict(os.environ), "", None
    resources.peer_failed = (time.monotonic(), "自证：ssh 被重置")
    connects = []
    resources.refresh_peer = lambda: connects.append(True)
    fake = object.__new__(SERVER.Scheduler)
    fake.environment, fake.resources = dict(os.environ), resources
    fake.peer_runner = os.path.join(world.root, "no-such-multi-host-run.sh")
    machine = SERVER.PeerMachine(fake)
    presence = machine.attempt_presence_readings({})
    removed, lines = machine.remove_build_directory(os.path.join(world.root, "builds", "task-1-1"))
    problems = []
    if presence[0] is not None or "不再去连" not in presence[1]:
        problems.append(f"核进程应当当场交「不再去连」，实际 {presence}")
    if removed or not lines or "不再去连" not in lines[0]:
        problems.append(f"删第二台的编译目录应当当场交没删成、原因写「不再去连」，实际 {removed}、{lines}")
    if connects:
        problems.append(f"冷却里不该去重读第二台，实际去了 {len(connects)} 回")
    if problems:
        raise SelftestFailure("冷却里不去连第二台：" + "；".join(problems))
    return "第二台还在冷却里：核进程、删编译目录都当场交不再去连"


def cell_clean_removes_peer_kept_build(world):
    """task clean 删第二台留着的编译目录（task_server.py 的 clean）：结了局的任务名下一次第二台的尝试留着编译目录（build_directory_kept、没删）：clean 照第二台的删法删它
    （PeerMachine.remove_build_directory，这里换成记下路径的假的）、记已删、回话里列出它。套在假调度器上，不起服务。
    弄坏开关 clean-peer-build-left 下照本机那一路判（本机没有这个路径），一份不删，这一格红。"""
    directory = "/selftest-peer-build-root/builds/task-80-1"
    task = {"id": 80, "session": SESSION_A, "state": "finished"}
    attempt = {"id": 81, "task_id": 80, "host": SERVER.PEER, "build_directory": directory, "build_directory_kept": True, "build_directory_removed": False}
    fake = object.__new__(SERVER.Scheduler)
    fake.environment = dict(os.environ)
    fake.store = types.SimpleNamespace(task=lambda task_id: (dict(task), True), attempts_of=lambda owner: [(dict(attempt), True)] if owner == 80 else [])
    fake.children_of = lambda task_id: []
    fake.build_removal_lock = threading.Lock()
    asked, marked = [], []
    fake.machines = {SERVER.PEER: types.SimpleNamespace(remove_build_directory=lambda path: (asked.append(path) is None, []))}
    fake.build_directories = types.SimpleNamespace(remove_run_build_directory=lambda path: (True, []))
    fake.mark_build_directory_removed = lambda found: marked.append(found["id"])
    fake.remove_scenes = lambda task_id: ([], [])
    fake.operation = lambda *arguments: None
    status, reply = SERVER.Scheduler.clean(fake, 80, SESSION_A, 0)
    if status != 200 or asked != [directory] or marked != [81] or [item.get("path") for item in reply.get("removed") or []] != [directory]:
        raise SelftestFailure(f"第二台留着的编译目录应当照第二台的删法删、记已删、回话列出，实际 {status}、删了 {asked}、记了 {marked}、回话 {reply}")
    return "task clean 照第二台的删法删了它留着的编译目录"


COMMAND_WORD_SAMPLES = (   # command-word-index-skips-option-values：(argv, 命令词的位置)
    (["env", "-C", "/tmp", "cargo", "build"], 3), (["env", "A=1", "-u", "B", "cargo"], 4), (["nice", "-n10", "cargo", "build"], 2),
    (["nice", "--adjustment", "10", "cargo"], 3), (["nice", "-n", "5", "cargo"], 3), (["timeout", "-k", "5", "60", "cargo"], 4),
    (["python3", "-W", "ignore", "x.py"], 3), (["bash", "-o", "pipefail", "x.sh"], 3),
)


def cell_command_word_index_skips_option_values(world):
    """剥外层包装时连同选项值跳过（task_server_lib.py 的 command_word_index、whole_package_cargo_test、CARGO_VALUE_OPTIONS）：COMMAND_WORD_SAMPLES 每条的命令词位置照表；
    cargo test -p 包 --exclude 别的 不当点名了测试（--exclude 的值不是测试名）；-p包 连写的照旧不吞下一个词（后面那个是测试名）。
    弄坏开关 command-wrapper-values-unskipped 下停在选项值上，cargo-exclude-value-unskipped 下 --exclude 的值当测试名，这一格红。"""
    wrong = [(argv, wanted, lib.command_word_index(argv)) for argv, wanted in COMMAND_WORD_SAMPLES if lib.command_word_index(argv) != wanted]
    problems = [f"{' '.join(argv)}：命令词应当在第 {wanted} 个位置，实际 {found}" for argv, wanted, found in wrong]
    if not lib.whole_package_cargo_test(["-p", "singlefs-core", "--exclude", "singlefs-verification"]):
        problems.append("cargo test -p singlefs-core --exclude singlefs-verification 应当判整包（--exclude 的值不是测试名），实际当点名了测试")
    if lib.whole_package_cargo_test(["-psinglefs-core", "some_test"]):
        problems.append("cargo test -psinglefs-core some_test 应当判点名了测试，实际判整包")
    if problems:
        raise SelftestFailure("剥包装的选项值：" + "；".join(problems))
    return f"{len(COMMAND_WORD_SAMPLES)} 种带值选项的写法命令词位置都对；--exclude 的值不当测试名"


def cell_runner_start_error_recorded(world):
    """命令起不来照写 result.json（attempt_runner.py 的 main）：argv 头一个不在：attempt_runner 退 127、result.json 的 exit_code 127、带 start_error，日志记一行「命令起不来」。
    弄坏开关 runner-start-error-uncaught 下（说明里 start_error_uncaught）照抛出去、result.json 写不出，这一格红。"""
    directory = os.path.join(world.work, "runner-start-error")
    os.makedirs(directory, exist_ok=True)
    log = os.path.join(directory, "attempt.log")
    specification_path = os.path.join(directory, "spec.json")
    with open(specification_path, "w", encoding="utf-8") as handle:
        json.dump({"argv": [os.path.join(world.work, "no-such-command")], "cwd": world.work, "environment": dict(os.environ), "log": log, "directory": directory,
                   "fast": True, "unit": "", "start_error_uncaught": lib.broken("runner-start-error-uncaught")}, handle)
    completed = subprocess.run([sys.executable, "-B", lib.RUNNER_SCRIPT, specification_path], stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)
    result, logged = None, ""
    with contextlib.suppress(OSError, ValueError):
        with open(os.path.join(directory, "result.json"), encoding="utf-8") as handle:
            result = json.load(handle)
    with contextlib.suppress(OSError):
        with open(log, encoding="utf-8", errors="replace") as handle:
            logged = handle.read()
    if completed.returncode != 127 or not isinstance(result, dict) or result.get("exit_code") != 127 or not result.get("start_error") or "命令起不来" not in logged:
        raise SelftestFailure(f"命令起不来：应当退 127、result.json 的 exit_code 127 且带 start_error、日志记一行，实际退 {completed.returncode}、"
                              f"result.json {'写不出' if result is None else result}、日志「{logged.strip()[-200:]}」", (completed.stdout + completed.stderr).splitlines()[-12:])
    return "命令起不来：退 127，result.json 照写、带原因"


RUNNER_FINAL_SAMPLE_WRAPPER = """import importlib.util, json, os, sys
specification = importlib.util.spec_from_file_location("attempt_runner_under_test", sys.argv[1])
runner = importlib.util.module_from_spec(specification)
specification.loader.exec_module(runner)
record = sys.argv[2]
seen = []
def sample(self):
    seen.append(os.path.exists(f"/proc/{self.root_process_id}"))
    self.memory_peak = 1
runner.Sampler.sample = sample
code = runner.main(sys.argv[3:])
with open(record, "w", encoding="utf-8") as handle:
    json.dump(seen, handle)
sys.exit(code)
"""   # runner-final-sample-after-exit：载入 attempt_runner.py、把 Sampler.sample 换成记「这一刻起的那个子进程还在不在 /proc」的假的，再照原样起 main


def cell_runner_final_sample_after_exit(world):
    """命令退了之后再采一次（attempt_runner.py 的 main）：采样换成记下子进程此刻在不在 /proc 的假的（头一次就记上内存峰值），命令睡 0.3 秒退 0：采样记录里最后一次是子进程
    已收走之后（不在 /proc）。弄坏开关 runner-final-sample-skipped 下（说明里 final_sample_skipped）采到过就不补，最后一次还在跑着，这一格红。"""
    directory = os.path.join(world.work, "runner-final-sample")
    os.makedirs(directory, exist_ok=True)
    wrapper, record = os.path.join(world.work, "runner-final-sample.py"), os.path.join(directory, "samples.json")
    with open(wrapper, "w", encoding="utf-8") as handle:
        handle.write(RUNNER_FINAL_SAMPLE_WRAPPER)
    specification_path = os.path.join(directory, "spec.json")
    with open(specification_path, "w", encoding="utf-8") as handle:
        json.dump({"argv": ["bash", "-c", "sleep 0.3"], "cwd": world.work, "environment": dict(os.environ), "log": os.path.join(directory, "attempt.log"),
                   "directory": directory, "fast": True, "unit": "", "leftovers_kept": True, "final_sample_skipped": lib.broken("runner-final-sample-skipped")}, handle)
    completed = subprocess.run([sys.executable, "-B", wrapper, lib.RUNNER_SCRIPT, record, specification_path], stdin=subprocess.DEVNULL, capture_output=True, text=True, timeout=60)
    seen = None
    with contextlib.suppress(OSError, ValueError):
        with open(record, encoding="utf-8") as handle:
            seen = json.load(handle)
    if completed.returncode != 0 or not seen or seen[-1] is not False:
        raise SelftestFailure(f"命令退了之后应当再采一次（最后一次采样时子进程已不在），实际退 {completed.returncode}、采样记录 {seen}",
                              (completed.stdout + completed.stderr).splitlines()[-12:])
    return f"采了 {len(seen)} 次，最后一次在命令退了之后"


CELLS = {
    "retired-outcome-status-generic": cell_retired_outcome_status_generic,
    "pause-allows-one-session": cell_pause_allows_one_session, "pause-allow-waiting-text": cell_pause_allow_waiting_text,
    "pause-allow-switches-session": cell_pause_allow_switches_session, "resume-clears-allowed-session": cell_resume_clears_allowed_session,
    "pause-allow-survives-restart": cell_pause_allow_survives_restart, "pause-allow-reported": cell_pause_allow_reported,
    "pause-holds-new-attempts": cell_pause_holds_new_attempts, "pause-lets-running-finish": cell_pause_lets_running_finish,
    "pause-survives-restart": cell_pause_survives_restart, "resume-releases-queue": cell_resume_releases_queue, "pause-reported": cell_pause_reported,
    "pause-not-starved": cell_pause_not_starved, "pause-waits-for-starting-attempt": cell_pause_waits_for_starting_attempt,
    "attention-queued-too-long": cell_attention_queued_too_long,
    "attention-cleanup-pending": cell_attention_cleanup_pending,
    "attention-shard-advance-failed": cell_attention_shard_advance_failed, "attention-reported-once": cell_attention_reported_once,
    "attention-reported-survives-restart": cell_attention_reported_survives_restart, "attention-queue-waits-for-reason": cell_attention_queue_waits_for_reason,
    "page-cache-not-counted-as-process-memory": cell_page_cache_not_counted_as_process_memory,
    "attention-type-in-anomaly-table": cell_attention_type_in_anomaly_table, "temporary-service-wrapper-compiles": cell_temporary_service_wrapper_compiles,
    "temporary-service-own-server-compiles": cell_temporary_service_own_server_compiles,
    "slow-world-patient-wait": cell_slow_world_patient_wait,
    "parallelism-registration-refused": cell_parallelism_registration_refused,
    "shard-child-key-folded-by-words": cell_shard_child_key_folded_by_words,
    "workspace-write-interrupted-reported": cell_workspace_write_interrupted_reported,
    "workspace-write-complete-not-reported": cell_workspace_write_complete_not_reported,
    "resume-declaration-first-word-only": cell_resume_declaration_first_word_only,
    "check-missing": cell_check_missing, "install": cell_install, "control-readback": cell_control_readback, "server-unreachable": cell_server_unreachable,
    "exit-code-passthrough": cell_exit_code_passthrough, "restart-keeps-running": cell_restart_keeps_running, "start-once": cell_start_once,
    "queue-holds-unplaceable": cell_queue_holds_unplaceable, "arrival-order": cell_arrival_order,
    "prediction-computed-once": cell_prediction_computed_once, "pass-reads-resources-once": cell_pass_reads_resources_once,
    "pass-booking-carried": cell_pass_booking_carried, "launch-rechecked": cell_launch_rechecked, "every-command-reaches-peer": cell_every_command_reaches_peer,
    "error-types": cell_error_types,
    "unit-start-failed": cell_unit_start_failed, "memory-cap-growth": cell_memory_cap_growth, "memory-cap-at-maximum": cell_memory_cap_at_maximum,
    "unrecorded-default-small": cell_unrecorded_default_small, "unrecorded-cap-growth": cell_unrecorded_cap_growth,
    "unrecorded-burst-balanced": cell_unrecorded_burst_balanced,
    "retry-kinds": cell_retry_kinds, "cancel-own-session-only": cell_cancel_own_session_only,
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
    "history-peak-unrecorded-default": cell_history_peak_unrecorded_default, "writer-dry-run-retried": cell_writer_dry_run_retried,
    "health-head-cpu-gate-not-starved": cell_health_head_cpu_gate_not_starved,
    "cap-hit-record-over-every-host-starts-clamped": cell_cap_hit_record_over_every_host_starts_clamped,
    "booking-within-host-limit-every-history": cell_booking_within_host_limit_every_history,
    "control-drop-history-oversized-records": cell_control_drop_history_oversized_records,
    "oversized-budget-reported": cell_oversized_budget_reported, "long-run-idle-cores-reported": cell_long_run_idle_cores_reported,
    "fills-any-cap-reported": cell_fills_any_cap_reported,
    "old-metric-peaks-not-counted": cell_old_metric_peaks_not_counted,
    "anomaly-table-matches-kinds": cell_anomaly_table_matches_kinds, "anomaly-advice-in-texts": cell_anomaly_advice_in_texts,
    "anomaly-repushed-with-history": cell_anomaly_repushed_with_history, "anomaly-acknowledged-not-repushed": cell_anomaly_acknowledged_not_repushed, "oversized-budget-acknowledged-after-restart": cell_oversized_budget_acknowledged_after_restart, "runaway-reports-survive-restart": cell_runaway_reports_survive_restart, "runaway-reused-pid-counted-afresh": cell_runaway_reused_pid_counted_afresh, "advance-episodes-survive-restart": cell_advance_episodes_survive_restart, "restart-surviving-tampered-not-adopted": cell_restart_surviving_tampered_not_adopted, "budget-source-names-folded-key": cell_budget_source_names_folded_key, "unclaimed-outcome-repushed": cell_unclaimed_outcome_repushed, "running-progress-pushed": cell_running_progress_pushed,
    "inbox-socket-follows-session": cell_inbox_socket_follows_session, "submit-awaits-confirmation": cell_submit_awaits_confirmation,
    "unconfirmed-cancelled": cell_unconfirmed_cancelled,
    "cleanup-protocol-unit-tests": cell_cleanup_protocol_unit_tests,
    "fills-any-cap-cap-hits-not-reported": cell_fills_any_cap_cap_hits_not_reported,
    "busy-cpu-pinned-at-cap-retried": cell_busy_cpu_pinned_at_cap_retried, "busy-cpu-below-cap-keeps-running": cell_busy_cpu_below_cap_keeps_running,
    "peer-pinned-at-cap-busy-cpu-retried": cell_peer_pinned_at_cap_busy_cpu_retried, "attention-quiet-busy-log": cell_attention_quiet_busy_log,
    "io-wait-local-not-stalled": cell_io_wait_local_not_stalled, "io-wait-peer-not-stalled": cell_io_wait_peer_not_stalled,
    "peer-health-slow-rechecked": cell_peer_health_slow_rechecked, "peak-memory-stat-in-note": cell_peak_memory_stat_in_note,
    "peer-peak-memory-stat-in-note": cell_peer_peak_memory_stat_in_note,
    "loop-leak-trend-sentences": cell_loop_leak_trend_sentences,
    "pinned-cap-over-limit-reported": cell_pinned_cap_over_limit_reported, "pinned-squeezed-requeued": cell_pinned_squeezed_requeued,
    "pin-cap-help-not-pinned": cell_pin_cap_help_not_pinned, "scene-sessionless-swept": cell_scene_sessionless_swept,
    "loop-leak-check-on-bounce": cell_loop_leak_check_on_bounce, "repeated-cap-hits-reminded-once": cell_repeated_cap_hits_reminded_once,
    "long-run-reminded-not-stopped": cell_long_run_reminded_not_stopped, "long-run-reminder-across-cap-hits": cell_long_run_reminder_across_cap_hits,
    "long-run-full-gate-not-reminded": cell_long_run_full_gate_not_reminded, "long-run-check-not-exempt": cell_long_run_check_not_exempt,
    "long-run-merge-and-verify-not-exempt": cell_long_run_merge_and_verify_not_exempt,
    "long-run-confirm-silences": cell_long_run_confirm_silences, "long-run-confirm-refused": cell_long_run_confirm_refused,
    "long-run-sessionless-noted": cell_long_run_sessionless_noted,
    "whole-package-unconfirmed-refused": cell_whole_package_unconfirmed_refused, "whole-package-named-accepted": cell_whole_package_named_accepted,
    "whole-package-confirmation-form": cell_whole_package_confirmation_form, "whole-package-confirmed-still-reminded": cell_whole_package_confirmed_still_reminded,
    "whole-workspace-cargo-refused": cell_whole_workspace_cargo_refused,
    "progress-stall-reported": cell_progress_stall_reported,
    "peer-retry-build-directory-reused": cell_peer_retry_build_directory_reused,
    "repository-target-bypass-warned": cell_repository_target_bypass_warned, "cancelled-artifacts-removed": cell_cancelled_artifacts_removed,
    "cancelled-before-peer-trip-built": cell_cancelled_before_peer_trip_built, "inbox-outcomes-resent-after-error": cell_inbox_outcomes_resent_after_error,
    "thirty-second-tick-prompt": cell_thirty_second_tick_prompt, "starved-head-reported": cell_starved_head_reported,
    "blocked-head-not-starved": cell_blocked_head_not_starved, "starved-head-judged-by-the-formula": cell_starved_head_judged_by_the_formula,
    "aged-task-judged-first": cell_aged_task_judged_first,
    "unmeasured-not-zero": cell_unmeasured_not_zero, "retry-goes-where-it-fits": cell_retry_goes_where_it_fits,
    "peer-runs-in-shared-tree": cell_peer_runs_in_shared_tree, "clean-failure-explained": cell_clean_failure_explained, "stalled-attempt-stopped": cell_stalled_attempt_stopped, "cpu-unread-not-stalled": cell_cpu_unread_not_stalled,
    "running-status-shows-progress": cell_running_status_shows_progress, "stall-waits-for-grace": cell_stall_waits_for_grace, "computing-quietly-not-stalled": cell_computing_quietly_not_stalled, "peer-stall-judged-by-peer-cpu": cell_peer_stall_judged_by_peer_cpu, "three-duties-two-sessions": cell_three_duties_two_sessions,
    "hundred-five-minute-tasks-balanced": cell_hundred_five_minute_tasks_balanced, "stalled-at-cap-retried": cell_stalled_at_cap_retried, "aged-head-at-slice-total-starts": cell_aged_head_at_slice_total_starts, "peer-run-failed-retried": cell_peer_run_failed_retried, "squeezed-at-total-is-cap-hit": cell_squeezed_at_total_is_cap_hit, "peer-starts-throttled": cell_peer_starts_throttled, "attempt-unit-nofile-raised": cell_attempt_unit_nofile_raised, "queued-shows-waiting-reason": cell_queued_shows_waiting_reason,
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
    "control-start": cell_control_start, "gpu-service-required-when-gpu-on": cell_gpu_service_required_when_gpu_on,
    "gpu-service-untouched-when-gpu-off": cell_gpu_service_untouched_when_gpu_off, "pinned-peer-unavailable": cell_pinned_peer_unavailable,
    "stall-clock-from-log-mtime": cell_stall_clock_from_log_mtime,
    "memory-cap-passed-to-command": cell_memory_cap_passed_to_command, "source-repository-passed-to-command": cell_source_repository_passed_to_command,
    "shared-roots-passed-to-command": cell_shared_roots_passed_to_command,
    "peer-admission-marked": cell_peer_admission_marked, "peer-disk-floor": cell_peer_disk_floor, "suspected-stall-notified": cell_suspected_stall_notified,
    "pinned-cap-from-configuration": cell_pinned_cap_from_configuration,
    "pinned-cap-unreadable-refused": cell_pinned_cap_unreadable_refused,
    "pinned-cap-by-host": cell_pinned_cap_by_host, "by-host-missing-host-refused": cell_by_host_missing_host_refused,
    "by-host-outside-address-refused": cell_by_host_outside_address_refused, "shard-host-is-address": cell_shard_host_is_address,
    "gate-stages-sent-to-peer": cell_gate_stages_sent_to_peer,
    "admission-booked-at-cap": cell_admission_booked_at_cap, "health-answers-while-resources-slow": cell_health_answers_while_resources_slow,
    "leftover-processes-stopped": cell_leftover_processes_stopped, "leftovers-only-in-own-unit": cell_leftovers_only_in_own_unit, "long-run-failure-noted": cell_long_run_failure_noted,
    "leftovers-error-isolated": cell_leftovers_error_isolated, "inbox-shard-attention-merged": cell_inbox_shard_attention_merged,
    "attempt-label-passed-to-command": cell_attempt_label_passed_to_command,
    "whole-starts-beside-running-fragment": cell_whole_starts_beside_running_fragment,
    "ended-attempt-settled-after-restart": cell_ended_attempt_settled_after_restart,
    "cpu-gate-holds-busy-host": cell_cpu_gate_holds_busy_host, "unstarted-slice-uses-configured-total": cell_unstarted_slice_uses_configured_total, "down-host-waits-not-never": cell_down_host_waits_not_never,
    "environment-prefix-stripped-from-key": cell_environment_prefix_stripped_from_key,
    "verdict-library-family-refused": cell_verdict_library_family_refused, "workspace-writer-not-retried": cell_workspace_writer_not_retried,
    "retry-cap-not-floored": cell_retry_cap_not_floored, "recorded-cap-not-floored": cell_recorded_cap_not_floored,
    "cap-hit-low-peak-not-recorded-at-cap": cell_cap_hit_low_peak_not_recorded_at_cap, "runaway-process-notified": cell_runaway_process_notified, "runaway-process-stopped-after-reports": cell_runaway_process_stopped_after_reports,
    "margin-measured-per-host": cell_margin_measured_per_host,
    "margin-unmeasured-not-placed": cell_margin_unmeasured_not_placed, "pin-cap-refused-within-margin": cell_pin_cap_refused_within_margin,
    "peer-booked-like-local": cell_peer_booked_like_local, "peer-scope-missing-booked-at-cap": cell_peer_scope_missing_booked_at_cap,
    "two-hosts-booked-within-capacity": cell_two_hosts_booked_within_capacity, "judge-error-skips-only-that-task": cell_judge_error_skips_only_that_task,
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
    "nfs-probe-concurrent-runs": cell_nfs_probe_concurrent_runs,
    "short-command-peak-recorded": cell_short_command_peak_recorded, "merge-on-peer-reads-shard-outputs": cell_merge_on_peer_reads_shard_outputs,
    "stop-ends-probe-children": cell_stop_ends_probe_children, "failure-scene-kept-until-collected": cell_failure_scene_kept_until_collected,
    "shard-failure-waits-for-running": cell_shard_failure_waits_for_running, "shard-cancel-alone-merges": cell_shard_cancel_alone_merges,
    "parent-cancel-stops-every-shard": cell_parent_cancel_stops_every_shard, "cancel-after-natural-end": cell_cancel_after_natural_end,
    "failed-start-not-counted": cell_failed_start_not_counted, "child-rows-verified": cell_child_rows_verified, "task-locks-released": cell_task_locks_released,
    "attempt-row-not-overwritten": cell_attempt_row_not_overwritten, "parent-cancel-waits-for-children": cell_parent_cancel_waits_for_children,
    "clean-waits-for-finished-removal": cell_clean_waits_for_finished_removal, "recheck-judges-disk": cell_recheck_judges_disk,
    "recheck-judges-nfs": cell_recheck_judges_nfs,
    "stall-stop-spares-next-attempt": cell_stall_stop_spares_next_attempt, "peer-cleanup-recovered-once": cell_peer_cleanup_recovered_once,
    "tampered-attempt-ended-once": cell_tampered_attempt_ended_once, "tampered-attempt-unit-stopped": cell_tampered_attempt_unit_stopped,
    "manual-submit-path": cell_manual_submit_path, "full-gate-flag": cell_full_gate_flag,
    "running-attempts-verified-only": cell_running_attempts_verified_only,
    "concurrent-confirms-serialized": cell_concurrent_confirms_serialized,
    "resident-entries-forgotten": cell_resident_entries_forgotten,
    "whole-package-single-cells-refused": cell_whole_package_single_cells_refused, "unscoped-runs-refused": cell_unscoped_runs_refused,
    "whole-package-feature-items-accepted": cell_whole_package_feature_items_accepted,
    "whole-package-feature-items-others-refused": cell_whole_package_feature_items_others_refused,
    "attempt-gets-no-card": cell_attempt_gets_no_card, "card-declarations-refused": cell_card_declarations_refused,
    "queued-whole-package-rechecked": cell_queued_whole_package_rechecked,
    "unnamed-merge-unnamed-refused": cell_unnamed_merge_unnamed_refused, "unnamed-merge-named-accepted": cell_unnamed_merge_named_accepted,
    "unnamed-cargo-target-refused": cell_unnamed_cargo_target_refused, "unnamed-cargo-exact-accepted": cell_unnamed_cargo_exact_accepted,
    "unnamed-prove-red-unnamed-refused": cell_unnamed_prove_red_unnamed_refused, "unnamed-prove-red-named-accepted": cell_unnamed_prove_red_named_accepted,
    "unnamed-exemptions-accepted": cell_unnamed_exemptions_accepted, "unnamed-gate-cells-judged": cell_unnamed_gate_cells_judged,
    "unnamed-crash-gate-whole-items-refused": cell_unnamed_crash_gate_whole_items_refused, "unnamed-crash-gate-cases-only-accepted": cell_unnamed_crash_gate_cases_only_accepted,
    "test-starting-cells-listed": cell_test_starting_cells_listed,
    "run-hard-limit-arithmetic": cell_run_hard_limit_arithmetic,
    "shard-hard-limit-by-each-piece": cell_shard_hard_limit_by_each_piece,
    "run-hard-limit-stopped": cell_run_hard_limit_stopped,
    "run-hard-limit-why-still-stops": cell_run_hard_limit_why_still_stops,
    "run-hard-limit-full-gate-not-stopped": cell_run_hard_limit_full_gate_not_stopped,
    "run-hard-limit-cap-hit-resets": cell_run_hard_limit_cap_hit_resets,
    "run-hard-limit-writer-stopped": cell_run_hard_limit_writer_stopped,
    "run-hard-limit-shard-stopped": cell_run_hard_limit_shard_stopped,
    "run-hard-limit-applies-after-restart": cell_run_hard_limit_applies_after_restart,
    "run-hard-limit-stop-retried": cell_run_hard_limit_stop_retried,
    "confirmation-scope-text": cell_confirmation_scope_text,
    "selftest-units-stop-with-outer": cell_selftest_units_stop_with_outer,
    "running-booked-unscoped-at-cap": cell_running_booked_unscoped_at_cap,
    "shard-key-without-count": cell_shard_key_without_count,
    "stopped-peaks-floor-record": cell_stopped_peaks_floor_record, "compiles-declared-entry": cell_compiles_declared_entry,
    "parent-cancelling-child-not-claimed": cell_parent_cancelling_child_not_claimed, "cpu-gate-counts-running-threads": cell_cpu_gate_counts_running_threads,
    "cpu-gate-measured-cores": cell_cpu_gate_measured_cores,
    "lookup-indexes-used": cell_lookup_indexes_used, "running-attempts-read-once": cell_running_attempts_read_once,
    "budget-source-only-when-reported": cell_budget_source_only_when_reported, "repass-cleared-when-idle": cell_repass_cleared_when_idle,
    "peer-probe-off-scheduler": cell_peer_probe_off_scheduler,
    "peer-cleanup-off-scheduler": cell_peer_cleanup_off_scheduler,
    "unit-unreadable-exit-file-settles": cell_unit_unreadable_exit_file_settles,
    "unit-unreadable-gone-recorded-lost": cell_unit_unreadable_gone_recorded_lost, "unit-unreadable-running-reminded": cell_unit_unreadable_running_reminded,
    "unit-unreadable-unverified-health-reported": cell_unit_unreadable_unverified_health_reported,
    "running-booked-at-measured-usage": cell_running_booked_at_measured_usage, "running-young-booked-at-cap": cell_running_young_booked_at_cap, "cpu-gate-unrecorded-by-registered-threads": cell_cpu_gate_unrecorded_by_registered_threads, "cpu-gate-running-by-latest-reading": cell_cpu_gate_running_by_latest_reading, "cpu-gate-running-after-restart-by-average": cell_cpu_gate_running_after_restart_by_average, "submit-trace-logged": cell_submit_trace_logged, "cargo-test-key-by-package": cell_cargo_test_key_by_package, "selection-key-by-script": cell_selection_key_by_script,
    "memory-growth-not-stalled": cell_memory_growth_not_stalled,
    "cpu-gate-new-task-by-recorded-cores": cell_cpu_gate_new_task_by_recorded_cores,
    "cpu-gate-running-started-by-recorded-cores": cell_cpu_gate_running_started_by_recorded_cores,
    "shard-peak-record-count-matched": cell_shard_peak_record_count_matched, "heartbeat-lines-not-log-growth": cell_heartbeat_lines_not_log_growth,
    "signal-exit-named": cell_signal_exit_named,
    "finish-peak-segment-cpu-unread": cell_finish_peak_segment_cpu_unread,
    "fills-any-cap-record-caps-not-reported": cell_fills_any_cap_record_caps_not_reported,
    "unit-stopped-booking-released": cell_unit_stopped_booking_released,
    "cpu-reason-splits-io-wait": cell_cpu_reason_splits_io_wait,
    "peer-health-slow-within-recheck-period": cell_peer_health_slow_within_recheck_period,
    "control-check-probe-bounded": cell_control_check_probe_bounded,
    "control-check-reads-service-nfs-report": cell_control_check_reads_service_nfs_report,
    "promote-skips-cpu-gate": cell_promote_skips_cpu_gate,
    "promote-queue-order": cell_promote_queue_order,
    "progress-line-names-progress": cell_progress_line_names_progress,
    "attempt-progress-readings": cell_attempt_progress_readings,
    "quiet-log-busy-spared-by-progress": cell_quiet_log_busy_spared_by_progress,
    "peer-ssh-multiplexed": cell_peer_ssh_multiplexed,
    "shard-start-borrows-nothing": cell_shard_start_borrows_nothing,
    "plan-key-by-compiles": cell_plan_key_by_compiles,
    "booking-drift-reported": cell_booking_drift_reported,
    "booking-audit-young-not-drift": cell_booking_audit_young_not_drift,
    "cpu-gate-running-average-capped-by-registered": cell_cpu_gate_running_average_capped_by_registered,
    "reminders-not-resent-after-restart": cell_reminders_not_resent_after_restart,
    "reminder-thresholds-every-doubling": cell_reminder_thresholds_every_doubling,
    "reminder-ledger-other-shape-read-afresh": cell_reminder_ledger_other_shape_read_afresh,
    "long-run-split-reminded-at-outcome": cell_long_run_split_reminded_at_outcome,
    "workspace-write-blocked-reminded": cell_workspace_write_blocked_reminded,
    "inbox-headlines-tell-kind-state-round": cell_inbox_headlines_tell_kind_state_round,
    "external-kill-peer-line-recorded-and-retried": cell_external_kill_peer_line_recorded_and_retried,
    "confirm-sheet-reads-bash-c-inner-commands": cell_confirm_sheet_reads_bash_c_inner_commands,
    "workspace-writer-cap-has-headroom": cell_workspace_writer_cap_has_headroom,
    "control-key-stats-reported": cell_control_key_stats_reported,
    "control-key-stats-by-task": cell_control_key_stats_by_task,
    "control-key-stats-dump": cell_control_key_stats_dump,
    "machine-memory-line-holds-dispatch": cell_machine_memory_line_holds_dispatch,
    "external-kill-recorded-and-retried": cell_external_kill_recorded_and_retried,
    "booking-ledger-rows": cell_booking_ledger_rows,
    "admission-by-measured-bookings": cell_admission_by_measured_bookings,
    "aged-blocked-task-blocks-later": cell_aged_blocked_task_blocks_later, "jump-only-shorter-than-running": cell_jump_only_shorter_than_running,
    "memory-wait-names-both-numbers": cell_memory_wait_names_both_numbers,
    "cpu-gate-new-task-counts-failed-runs": cell_cpu_gate_new_task_counts_failed_runs,
    "confirm-retries-follow-churn": cell_confirm_retries_follow_churn,
    "cpu-gate-peak-segment-cores": cell_cpu_gate_peak_segment_cores,
    "cpu-gate-line-twice-cores": cell_cpu_gate_line_twice_cores,
    "cpu-gate-runnable-threads": cell_cpu_gate_runnable_threads,
    "threads-over-registered-reported": cell_threads_over_registered_reported,
    "threads-over-registered-peer-reported": cell_threads_over_registered_peer_reported,
    "shard-binaries-removed-at-outcome": cell_shard_binaries_removed_at_outcome,
    "runner-threads-file-counts": cell_runner_threads_file_counts,
    "store-write-rolls-back-on-error": cell_store_write_rolls_back_on_error,
    "disk-floor-peer-not-never-fits": cell_disk_floor_peer_not_never_fits,
    "stall-reads-log-before-cpu": cell_stall_reads_log_before_cpu,
    "peer-readings-merged-across-reads": cell_peer_readings_merged_across_reads,
    "log-content-serialized": cell_log_content_serialized,
    "log-noise-marks-not-growth": cell_log_noise_marks_not_growth,
    "runaway-targets-skip-children": cell_runaway_targets_skip_children,
    "promote-wakes-and-promotes-shards": cell_promote_wakes_and_promotes_shards,
    "promote-placement-rules": cell_promote_placement_rules,
    "pause-keeps-queue-positions": cell_pause_keeps_queue_positions,
    "stop-joins-peer-probe": cell_stop_joins_peer_probe,
    "peer-cooldown-skips-connect": cell_peer_cooldown_skips_connect,
    "clean-removes-peer-kept-build": cell_clean_removes_peer_kept_build,
    "command-word-index-skips-option-values": cell_command_word_index_skips_option_values,
    "runner-start-error-recorded": cell_runner_start_error_recorded,
    "runner-final-sample-after-exit": cell_runner_final_sample_after_exit,
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
    # 套在假调度器上的格（fake.environment 拷的是本进程环境）同样照小世界的两个数：CPU 闸的倍数 2、刚起跑照上限记那一条不看
    os.environ.setdefault(lib.CPU_OVERSUBSCRIPTION_VARIABLE, "2")
    os.environ.setdefault(lib.YOUNG_BOOKING_VARIABLE, "0")
    items = [item for item in os.environ.get(lib.BREAK_VARIABLE, "").split(",") if item]
    unknown = [item for item in items if item not in lib.KNOWN_BREAK_ITEMS]
    if unknown:
        print(f"  ✗ {lib.BREAK_VARIABLE} 里认不出的项：{'、'.join(unknown)}")
        print("  → 怎么办：照 task_server_lib.py 文件头的开关表写")
        return lib.USAGE_EXIT
    # 同一格点了两次只跑一次：两份同名的格并行写同一个格目录、同一份 .out 与 .exit，收回时按格名只数得到一份（任务 13267 点了 46 个名、两格重名，收回 44 份）
    selected = list(dict.fromkeys(name for name in os.environ.get("TASK_SERVER_SELFTEST_CELLS", "").split(",") if name)) or list(CELLS)
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
        print(f"  ✓ task_server_lib.py 自证 {len(selected)} 格都对（{lib.SELFTEST_PARALLELISM_TEXT.format(workers=workers)}，每格一份真的 uvicorn 挂在临时 socket 上；各格讲什么见上面逐格那一行）")
        return 0
    finally:
        shutil.rmtree(socket_root, ignore_errors=True)
        shutil.rmtree(root, ignore_errors=True)
