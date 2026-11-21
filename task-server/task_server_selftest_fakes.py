"""task_server_selftest.py 拆出的一份：自证的各个常量与假脚本（假内存包装、假第二台、假 systemd-run 这些），各格与世界用它们（总说明在 task_server_selftest.py 文件头）。"""
import os
import task_server_lib as lib


FAILURE_PREFIX = "✗ task_server_lib.py 自证："
SESSION_A = "selftest-session-a"
SESSION_B = "selftest-session-b"

DEFAULT_INBOX_TOKEN = "selftest-default-inbox-token"   # 各格客户端缺省带的会话令牌（World.client_environment）：没登会话的格里服务找不到 socket、用不上它
CONFIRM_RETRIES = 3   # World.confirm_submitted：确认的数对不上（这一会儿同会话别的任务起了或收了）照服务打回的数再确认的基本回数（头一回对不上时再按同会话在排在跑的条数放宽）
CONFIRM_BUDGET_SECONDS = 60.0   # World.confirm_submitted：回数用完之后，只要服务打回的还是「数对不上」（同会话别的任务还在起、收），再追的时限；不随机器忙闲定回数
GIBIBYTE = 1024 ** 3
MEBIBYTE = 1024 ** 2

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

BLOCK_SCRIPT = 'for round in $(seq 1 600); do [[ -e "$1" ]] && exit 0; sleep 0.1; done; exit 9'

# stalled-at-cap-retried：库里记一次跑完的峰值 1536 MiB，上限照它 1536M（近五次最大，不乘 1.5）；假 scope 的 memory.peak 钉在 1536 MiB（到上限）；撞了乘 1.5 重排是 2304M
AT_CAP_SEED_PEAK_BYTES = 1536 * MEBIBYTE
AT_CAP_PEAK_BYTES = 1536 * MEBIBYTE
AT_CAP_PINNED_TEXT = "1536M"

# 占着到放行为止、判定比各条起止时刻的格用：至多 200 秒（短于一格的时限 CELL_TIMEOUT_SECONDS），机器忙时也不在放行之前自己退出
HOLD_UNTIL_RELEASED_SCRIPT = 'for round in $(seq 1 2000); do [[ -e "$1" ]] && exit 0; sleep 0.1; done; exit 9'
SEEDED_ENDED_AT = 1793318400   # 自证往服务库里记的那几次尝试的结束时刻：固定的一个时刻，比这一格真跑的尝试都早

SOCKET_ROOT_VARIABLE = "TASK_SERVER_SELFTEST_SOCKET_ROOT"
SOCKET_ROOT_PREFIX = "tss-sock-"
WORLD_SOCKET_PREFIX = "w-"

PLACEMENT_WAIT_SECONDS = 20.0   # peer-when-local-full 等那一条收尾的时限：弄坏开关下它一直排着，等满这么久就判红

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

# 各格假余量里每台此刻的余量（GiB，task_server.py 文件头 ②「余量」）：自证喂的定值，期望照它算；要别的数的格另给
SELFTEST_MARGIN_GIBIBYTES = 2

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

PEAKS = lib.load_module("memory_peaks_lib", os.path.join(lib.HOSTS_DIRECTORY, "memory_peaks_lib.py"))

SERVER = lib.load_module("task_server_constants_for_selftest", lib.SERVER_SCRIPT)   # 只读它的常量（AGED_WAIT_SECONDS）、Store 类（task-locks-released）与 Scheduler.push_outcomes（inbox-outcomes-resent-after-error，套在假调度器上），不在这个进程里起服务
