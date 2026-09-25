#!/usr/bin/env bash
# 给一条命令定内存上限，起跑之前先判整机放不放得下、放不下就排队（records/2026-09-16-subagent拆分提案.md 第四十节第 28、30 行）。三层：
#   ① 这一条的上限：放进 systemd 的临时 scope（MemoryMax=<上限>、MemorySwapMax=0、OOMPolicy=stop），撞上限只杀这个 scope 里的进程；
#   ② 总量兜底：每个 scope 都挂进同一个 slice（默认 singlefs-heavy.slice），slice 的 MemoryMax = 整机内存 − 余量。几条合起来撞顶也只在 slice 里杀，
#      杀不到 slice 外面的本地模型服务 vllm-prod 与 Claude Code 会话。slice 设不上就报错退出，不退回无总上限；
#   ③ 起跑前判内存量：拿一把锁，算「slice 里已占的 + 这一条要的」，不超过总上限才起；放不下就放锁、隔一会儿再判，等满上限报「排不上」退出。
# 变异跑道 research/scripts/mutate.sh 与门禁 59 号每条变异都经它跑；子 agent 跑 cargo test / cargo run / 实验二进制也要经它
# （.claude/hooks/heavy-test-guard.sh 在执行前拒不经它的）。提交时的重阶段整条经它跑，例：
#   SINGLEFS_HEAVY_TESTS=commit bash research/scripts/run-with-memory-cap.sh 16G bash <根>/.claude/gate.d/54-layer0-replay.sh --full <根>
#
#   run-with-memory-cap.sh <上限> <命令> [参数…]   排队，在上限里跑那条命令；退出码见下
#   run-with-memory-cap.sh --check <上限>          只试 slice 的总上限设不设得上、带同样上限的 scope 起不起得来（在 slice 里跑 true，不排队）；起不来退 251
#   run-with-memory-cap.sh --status                现量：整机内存、余量、slice 总上限与已占、账上在跑的每一条、slice 外面的用量
#   run-with-memory-cap.sh --selftest              自证（每一例在自己的临时 slice 里，总上限几百 M，吃不满机器）
#
# 上限照 systemd 的 MemoryMax 写法：正整数加 K / M / G / T（1024 进制），例 16G、512M。默认值由调用方定，写在调用方的脚本头。
# 退出码：
#   0–249  那条命令自己的退出码（cargo 只退 0、101、126、127 与 128 + 信号号，碰不到 250–254）；
#   250    撞了这一条自己的上限：scope 的 Result 是 oom-kill，scope 自己的 memory.events 里 oom 不为 0（或读不到）。
#          只认 Result、不认退出码：被杀的是 cargo 起的测试二进制时 cargo 退 101 或被停时退 143，被杀的是命令本身时退 137，外面 kill -9 同样退 137；
#   251    slice 的总上限设不上、systemd-run 起不来（没有用户级 systemd、D-Bus 连不上、上限写法 systemd 不认）、账与锁的目录建不了，或查不到 scope 的结局：
#          那条命令一行都没跑（或结局判不了），不退回无上限、无总上限去跑——那正是要防的事；
#   252    内存不够排不上：等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 秒还放不下，或这一条要的量比总上限还大（等多久都放不下，不等）。命令一行都没跑；
#   253    超过限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT）：scope 的 Result 是 timeout，systemd 停掉了 scope 里的进程；
#   254    被总上限挤掉：scope 的 Result 是 oom-kill，而 scope 自己的 oom 计数是 0、oom_kill 不为 0——整个 slice 满了，内核在 slice 里挑了这一条杀，
#          不是它自己撞了它的上限；它的结果不算数，重跑它；
#   2      用法错（上限、余量、总上限不是 正整数[KMGT] 的写法，没给命令）。
#
# 余量（RUN_WITH_MEMORY_CAP_RESERVE，默认 20G）留给 slice 外面：本地模型服务 vllm-prod、各个 Claude Code 会话与它们起的工具、门禁在包装外面的进程、内核。
#   2026-09-25 10:1x UTC 量的（只在本机成立；换机器、vllm 换模型或换并行度都要重量）：MemTotal 60.1 GiB；slice 外面回收不掉的合计 13.4 GiB
#   （MemTotal − MemAvailable，那时 slice 里几乎是空的），其中 vllm-prod 7.4 GiB anon + 0.6 GiB shmem（它 cgroup 的 memory.stat；
#   `systemctl status vllm-prod` 报的 Memory 37.9G 里 30.3 GiB 是模型文件的页缓存，回收得掉），user.slice 2.4 GiB anon（Claude Code 会话与它们起的进程），
#   其余约 3 GiB 是内核（Slab 1.9 GiB、页表）与别的服务。20G 比 13.4 GiB 多约 6.6 GiB，给会话变多、vllm 在负载下涨、门禁在包装外面的 python 进程留着；
#   本机 slice 总上限因此约 40 GiB。怎么现量：bash research/scripts/run-with-memory-cap.sh --status 打出上面每一项此刻的数。
#   slice 外面涨过了余量，总上限挡不住那一部分，整机照样可能被外面涨满：那一类靠看门狗的常驻内存告警与 .claude/hooks/session-start.sh 的 OOM 报告。
#
# 排队（第 ③ 层）怎么算，判与记账在同一把锁（${状态目录}/lock，flock）里，几条同时来的一条一条判：
#   这一条要的量：峰值表里这条命令（键见下）上一次实测的峰值，不超过它的上限；表里没有的按它的上限算——scope 的 MemoryMax 就是它最多能占的量，按它算不会少算。
#   已占 = slice 里回收不掉的用量（memory.current 减 active_file 与 inactive_file：页缓存回收得掉，而结束了的 scope 的页缓存挂回 slice、一直算在 slice 的
#          memory.current 里；2026-09-25 实测 scope 里 dd 写 120 MiB 退出之后，slice 的 memory.current 还是 131 MiB，其中 inactive_file 126 MiB）
#        + 账上每条放行了还在跑的 max(0, 它要的量 − 它的 scope 此刻回收不掉的用量)：刚起跑的 cargo 还没涨到量，只看 slice 的用量，前后脚来的两条都判得下、合起来超。
#   已占 + 这一条要的量 ≤ 总上限就放行，在账上记一笔（${状态目录}/ledger/<unit>.json：包装的进程号与起始时刻、要的量、上限、键），scope 收尾之后删掉；
#   包装被 KILL、没删掉的那一笔，下一次判的时候按进程号与起始时刻认出来删掉。
#   放不下就放锁、隔 ADMISSION_POLL_SECONDS 秒再判，第一次等与之后每 WAITING_REPORT_EVERY_SECONDS 秒往 stderr 报一行在等什么；
#   等满 RUN_WITH_MEMORY_CAP_WAIT_SECONDS（默认 3600）秒还放不下，列出账上占着的每一条，退 252。默认 3600 秒的依据：门禁 59 号一条变异限时 1800 秒，
#   排在后面的一般等前面一条跑完就轮到；再宽一倍，给别的重活占着 slice 的时候。
#
# 峰值表（RUN_WITH_MEMORY_CAP_PEAKS，默认主仓的 research/scripts/memory-peaks.tsv；从 git worktree 里跑的也读写主仓那一份）：一行一条命令，文件头的 # 行写口径，
#   由这个脚本建、读、写（写在同一把锁里，先写临时文件再改名；按 MiB 向上取整，与表里记的相同就不写）。口径：scope 里那个外壳 bash 在命令退出之后读自己 cgroup 的 memory.peak（字节）。
#   外壳不 exec 命令、留下来读，是因为 scope 一收尾它的 cgroup 就删了、systemd 也不留（2026-09-25 实测命令退出之后 MemoryPeak=[not set]）。
#   含页缓存（cargo 写编译产物的那些）与外壳 bash 自己，偏大不偏小。撞了这一条自己的上限记成上限；超时、被停、被总上限挤掉的不记（量到的是半截）。
#   不进 git（.gitignore）：数只在本机成立；而且门禁 59 号跑的时候每条都写它，放在被跟踪的地方，gate.sh 开跑与收尾的工作区指纹就对不上、整轮判红
#   （.claude/singlefs-ai-sop/scripts/lib.sh 的 worktree_fingerprint 用 git add -A 算，被忽略的文件不算）。
#   键：RUN_WITH_MEMORY_CAP_KEY 给了就用它；不给就把命令的各个词用空格接起来（制表符、换行换成空格，测试二进制名里 -<16 位十六进制> 的哈希去掉，截到 300 字）。
#
# 限时（RUN_WITH_MEMORY_CAP_TIME_LIMIT，秒；不设不限）：给 scope 设 RuntimeMaxSec，从起跑算、不算排队的时间；到点 systemd 给 scope 里每个进程发 TERM，
#   RUN_WITH_MEMORY_CAP_KILL_GRACE（默认 30）秒还不退再发 KILL，退 253。要限时就用这个变量，别在包装外面套 timeout：套在外面的把排队的时间也算进去。
# 等待上限（RUN_WITH_MEMORY_CAP_WAIT_SECONDS，秒，默认 3600）。这三个只管这一条：RUN_WITH_MEMORY_CAP_KEY、_TIME_LIMIT、_WAIT_SECONDS 传进命令之前清掉，
#   命令里再经包装跑的不会接着用。
# 其余环境变量有默认，只在自证与特殊场合设：
#   RUN_WITH_MEMORY_CAP_SLICE          slice 名，默认 singlefs-heavy.slice（名字里的 - 是 systemd 的层级：它挂在 singlefs.slice 底下）；只许 singlefs 开头的 .slice，别的退 2
#   RUN_WITH_MEMORY_CAP_SLICE_TOTAL    直接给总上限（systemd 写法），顶掉「整机内存 − 余量」；自证用它压到几百 M
#   RUN_WITH_MEMORY_CAP_STATE_DIR      锁与账的目录，默认 ${XDG_RUNTIME_DIR:-/run/user/<uid>}/singlefs-heavy-admission（tmpfs，重启清空，与 slice 同寿）
#   RUN_WITH_MEMORY_CAP_CGROUP_ROOT    cgroup v2 挂在哪，默认 /sys/fs/cgroup
#   RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE  判完放得下、记账之前停这么多秒：只给自证把「几条同时来」的竞态撑开
#   RUN_WITH_MEMORY_CAP_SELFTEST_TARGET       --selftest 测哪一份包装，默认自己：拿改前那一份跑新的自证，看它判错
# 包装里再经包装跑的（整条 gate.sh 经它跑，里面 59 号的每条再经它）：里层另起一个 scope、挪出外层，挂在同一个 slice 里各排各的队；外层照它要的量占着账。
#
# 弄坏开关 RUN_WITH_MEMORY_CAP_BREAK（只给 --selftest 证明它会红用）：nocap 不进 scope 直接跑（第一版之前的跑法）、fallback systemd-run 起不来时退回无上限跑、
#   exitcode 按退出码 137 / 143 判撞顶、noresult 不认 Result、noslice 不挂 slice 也不排队（加总上限之前的跑法）、slicefallback slice 设不上时不挂 slice 照跑、
#   nolock 判与记账不拿锁、noledger 已占只看 slice 的用量不算账上还没涨到量的、nodefault 峰值表里没有的按 0 算、ignoretable 不看峰值表一律按上限算、
#   noslicehit 被总上限挤掉的也报成撞了自己的上限、notimelimit 不设限时、keepstale 账上包装已经死了的那一笔不删。
#
# 发信号的地方，每一处只打得到这一条自己起的进程（用户 2026-09-25 定「后面的脚本不能终止前面的脚本」「不能动 ssh」；
# records/2026-09-16-subagent拆分提案.md 第四十节第 32 行）：
#   撞顶（OOMPolicy=stop）与超时（RuntimeMaxSec）由 systemd 停这一条自己开的 scope，scope 里只有这一条的外壳与它的命令；
#   查完结局 reset-failed 的是这一条自己的 unit；外壳里的 trap 只收信号、不发；包装被外层停时的 trap 只删自己的标记、账与峰值文件；
#   slice 名只许 singlefs 开头（case 前面那句核对）：包装给它设 MemoryMax、自证收尾停的是自证自己开的 singlefs_memory_selftest_<进程号>_<序号>.slice，
#   写成 user.slice、app.slice 这类就等于给别人的进程设上限；
#   自证里 kill -9 $$ 停的是那条命令自己；TERM 风暴那一例先核 cgroup 路径在这一例自己的 slice 底下、是自己开的 singlefs-memory-cap-*.scope，
#   不对就一个信号都不发——弄坏开关 nocap、fallback 不开 scope，命令跑在调用方的 cgroup 里（会话里就是 SSH 会话的 session-*.scope）。
set -uo pipefail

MEMORY_CAP_HIT_EXIT=250
MEMORY_CAP_UNAVAILABLE_EXIT=251
MEMORY_ADMISSION_REFUSED_EXIT=252
TIME_LIMIT_HIT_EXIT=253
SLICE_TOTAL_HIT_EXIT=254
MEBIBYTE=$((1024 * 1024))
DEFAULT_RESERVE="20G"                 # 依据见文件头「余量」一段
DEFAULT_WAIT_SECONDS=3600             # 依据见文件头「排队」一段
DEFAULT_KILL_GRACE_SECONDS=30
BROKEN_JUDGEMENT="${RUN_WITH_MEMORY_CAP_BREAK:-}"
SCOPE_RESULT_POLL_ATTEMPTS=50        # 查 scope 结局最多查这么多次，每次隔 0.1 秒：命令刚退出时 scope 可能还没收尾
SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SELF_PATH="$SCRIPT_DIRECTORY/$(basename "${BASH_SOURCE[0]}")"
SLICE="${RUN_WITH_MEMORY_CAP_SLICE:-singlefs-heavy.slice}"
RESERVE="${RUN_WITH_MEMORY_CAP_RESERVE:-$DEFAULT_RESERVE}"
SLICE_TOTAL_OVERRIDE="${RUN_WITH_MEMORY_CAP_SLICE_TOTAL:-}"
CGROUP_ROOT="${RUN_WITH_MEMORY_CAP_CGROUP_ROOT:-/sys/fs/cgroup}"
STATE_DIRECTORY="${RUN_WITH_MEMORY_CAP_STATE_DIR:-${XDG_RUNTIME_DIR:-/run/user/$(id -u)}/singlefs-heavy-admission}"
TIME_LIMIT_SECONDS="${RUN_WITH_MEMORY_CAP_TIME_LIMIT:-}"
KILL_GRACE_SECONDS="${RUN_WITH_MEMORY_CAP_KILL_GRACE:-$DEFAULT_KILL_GRACE_SECONDS}"
WAIT_SECONDS="${RUN_WITH_MEMORY_CAP_WAIT_SECONDS:-$DEFAULT_WAIT_SECONDS}"
# 在 scope 里跑的外壳：命令不 exec，跑完读自己 cgroup 的 memory.peak 与 memory.events 写进峰值文件（scope 一收尾 cgroup 就删了）。
# TERM / INT 设成记一笔的处理函数而不是忽略：忽略会被命令继承（exec 之后忽略的信号仍忽略），处理函数不会；
# 撞顶之后 systemd 停 scope 发的 TERM 于是只打断命令，外壳读完、写完再退，退出码照传命令的。
# 命令退出之后改成忽略 TERM / INT（后面不再起子进程，没有谁会继承它），只用 bash 的内建命令读、写（read、printf），不起 cut / cat / grep / mv：
# systemd 停 scope 时给 scope 里每个进程都发 TERM，那一刻正在跑的外部命令会被它杀掉、内建的 read 会被打断，峰值文件就写不全，
# 被总上限挤掉的那一条会被报成撞了自己的上限（2026-09-25 门禁 47 号里撞上过一次）。
# 外壳自己的 stderr 指到 /dev/null、命令的 stderr 经 fd 4 接回原处：命令被信号杀掉时 bash 会补一行「line 6: … Killed "$@"」，那一行不是命令的输出。
SCOPE_HOLDER='trap "stop_requested=1" TERM INT
: > "$1"
peak_file="$2"
cgroup_root="$3"
shift 3
exec 4>&2 2>/dev/null
"$@" 2>&4 4>&-
command_exit=$?
trap "" TERM INT
own_cgroup_line="" peak="" own_oom="" oom_kill=""
IFS= read -r own_cgroup_line < /proc/self/cgroup
own_group="$cgroup_root${own_cgroup_line#0::}"
IFS= read -r peak < "$own_group/memory.peak"
while read -r event_name event_count; do
  case "$event_name" in
    oom) own_oom="$event_count" ;;
    oom_kill) oom_kill="$event_count" ;;
  esac
done < "$own_group/memory.events"
printf "peak %s\noom %s\noom_kill %s\n" "$peak" "$own_oom" "$oom_kill" > "$peak_file"
exit "$command_exit"'

reject_usage() {
  echo "  ✗ $1" >&2
  echo "  → 怎么办：$2" >&2
  exit 2
}

report_unavailable() {
  echo "  ✗ 带内存上限的 scope 起不来：$1" >&2
  echo "  → 怎么办：$2" >&2
  exit "$MEMORY_CAP_UNAVAILABLE_EXIT"
}

check_size_syntax() { # check_size_syntax <写法> <是什么>
  [[ "$1" =~ ^[1-9][0-9]*[KMGT]?$ ]] \
    || reject_usage "$2要写成 正整数[K|M|G|T]（例 16G、512M），收到的是「$1」" "照 systemd 的 MemoryMax 写法给；默认值与依据写在文件头"
}

size_in_bytes() { # size_in_bytes <正整数[KMGT]> → 字节（1024 进制，与 systemd 相同）；写法先由 check_size_syntax 验过，超过 64 位交非 0
  local number="${1%[KMGT]}" suffix="${1##*[0-9]}" multiplier=1
  case "$suffix" in
    K) multiplier=1024 ;;
    M) multiplier=$MEBIBYTE ;;
    G) multiplier=$((MEBIBYTE * 1024)) ;;
    T) multiplier=$((MEBIBYTE * MEBIBYTE)) ;;
  esac
  ((${#number} <= 18 && number <= 9223372036854775807 / multiplier)) || return 1
  echo $((number * multiplier))
}

# slice 的总上限（字节，按 MiB 向下取整：cgroup 的 memory.max 按页取整，取整到 MiB 之后读回来与写进去的逐字相同）；
# 算不出来（MemTotal 读不到、余量比整机还大）交非 0
slice_total_bytes() {
  local memory_total_kibibytes reserve_bytes total
  if [[ -n "$SLICE_TOTAL_OVERRIDE" ]]; then
    total="$(size_in_bytes "$SLICE_TOTAL_OVERRIDE")"
  else
    memory_total_kibibytes="$(awk '$1 == "MemTotal:" {print $2}' /proc/meminfo 2>/dev/null)"
    [[ "$memory_total_kibibytes" =~ ^[0-9]+$ ]] || return 1
    reserve_bytes="$(size_in_bytes "$RESERVE")"
    total=$((memory_total_kibibytes * 1024 - reserve_bytes))
  fi
  total=$((total / MEBIBYTE * MEBIBYTE))
  ((total > 0)) || return 1
  echo "$total"
}

# 把 slice 的总上限设上、确认 cgroup 里真的是这个数：成功时设好 SLICE_TOTAL_BYTES、SLICE_DIRECTORY；失败时设 SLICE_PROBLEM、交非 0。
# 已经是这个数就不再设（每条变异都经这里，免得一条一次 D-Bus 写）。
ensure_slice() {
  local cgroup_path control_output read_back
  SLICE_PROBLEM=""
  if ! SLICE_TOTAL_BYTES="$(slice_total_bytes)"; then
    SLICE_PROBLEM="总上限算不出来：/proc/meminfo 的 MemTotal 读不到，或整机内存减余量 $RESERVE 已不剩（RUN_WITH_MEMORY_CAP_SLICE_TOTAL=${SLICE_TOTAL_OVERRIDE:-没设}）"
    return 1
  fi
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$SLICE" 2>/dev/null)"
  if [[ -n "$cgroup_path" && "$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)" == "$SLICE_TOTAL_BYTES" ]]; then
    SLICE_DIRECTORY="$CGROUP_ROOT$cgroup_path"
    return 0
  fi
  if ! control_output="$(systemctl --user set-property --runtime "$SLICE" MemoryMax="$SLICE_TOTAL_BYTES" 2>&1)"; then
    SLICE_PROBLEM="systemctl --user set-property --runtime $SLICE MemoryMax=$SLICE_TOTAL_BYTES 失败：$control_output"
    return 1
  fi
  if ! control_output="$(systemctl --user start "$SLICE" 2>&1)"; then
    SLICE_PROBLEM="systemctl --user start $SLICE 失败：$control_output"
    return 1
  fi
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$SLICE" 2>/dev/null)"
  if [[ -z "$cgroup_path" ]]; then
    SLICE_PROBLEM="$SLICE 起了却查不到它的 cgroup（systemctl --user show -p ControlGroup 是空的）"
    return 1
  fi
  read_back="$(cat "$CGROUP_ROOT$cgroup_path/memory.max" 2>/dev/null)"
  if [[ "$read_back" != "$SLICE_TOTAL_BYTES" ]]; then
    SLICE_PROBLEM="设了 MemoryMax=$SLICE_TOTAL_BYTES，读回 $CGROUP_ROOT$cgroup_path/memory.max 是「${read_back:-读不到}」（cgroup 的 memory 控制器没下放给用户级 systemd？）"
    return 1
  fi
  SLICE_DIRECTORY="$CGROUP_ROOT$cgroup_path"
}

prepare_state_directory() {
  mkdir -p "$STATE_DIRECTORY/ledger" "$STATE_DIRECTORY/peaks" 2>/dev/null && : >> "$STATE_DIRECTORY/lock" 2>/dev/null
}

default_peak_table() {
  local common_directory
  if common_directory="$(git -C "$SCRIPT_DIRECTORY" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" \
      && [[ "$(basename "$common_directory")" == ".git" && -d "$(dirname "$common_directory")/research/scripts" ]]; then
    echo "$(dirname "$common_directory")/research/scripts/memory-peaks.tsv"
  else
    echo "$SCRIPT_DIRECTORY/memory-peaks.tsv"
  fi
}

command_key() {
  local joined
  if [[ -n "${RUN_WITH_MEMORY_CAP_KEY:-}" ]]; then
    joined="$RUN_WITH_MEMORY_CAP_KEY"
  else
    joined="$*"
    joined="$(sed -E 's/-[0-9a-f]{16}\b//g' <<<"$joined")"
  fi
  joined="${joined//$'\t'/ }"
  joined="${joined//$'\n'/ }"
  printf '%s' "${joined:0:300}"
}

# 排队、记账、峰值表与 --status 的那一半（锁、JSON、算术在 python 里写得清楚）。第一个参数是动作：admit、finish、status。
admission() {
  RUN_WITH_MEMORY_CAP_STATE_DIR_RESOLVED="$STATE_DIRECTORY" RUN_WITH_MEMORY_CAP_PEAKS_RESOLVED="$PEAK_TABLE" \
    RUN_WITH_MEMORY_CAP_WAIT_SECONDS_RESOLVED="$WAIT_SECONDS" python3 /dev/fd/3 "$@" 3<<'PY'
import contextlib, fcntl, json, os, sys, time
from datetime import datetime, timezone

MEMORY_CAP_UNAVAILABLE_EXIT = 251
MEMORY_ADMISSION_REFUSED_EXIT = 252
ADMISSION_POLL_SECONDS = 1.0            # 放不下时隔多久再判一次
WAITING_REPORT_EVERY_SECONDS = 60       # 排队时隔多久往 stderr 报一行
MEBIBYTE = 1024 * 1024
state_directory = os.environ["RUN_WITH_MEMORY_CAP_STATE_DIR_RESOLVED"]
ledger_directory = os.path.join(state_directory, "ledger")
lock_path = os.path.join(state_directory, "lock")
peak_table_path = os.environ["RUN_WITH_MEMORY_CAP_PEAKS_RESOLVED"]
broken = os.environ.get("RUN_WITH_MEMORY_CAP_BREAK", "")
PEAK_TABLE_HEADER = [
    "# 每条命令上一次实测的内存峰值：research/scripts/run-with-memory-cap.sh 建、读、写（排队时按它算「这条要的量」），别手改。",
    "# 口径：scope 里的外壳 bash 在命令退出之后读自己 cgroup 的 memory.peak，按 MiB 向上取整的字节数；含页缓存（cargo 写编译产物的那些）与外壳 bash 自己，偏大不偏小。",
    "#   撞了这一条自己的上限的记成上限；超时、被停、被总上限挤掉的不记（量到的是半截）。只在本机成立，换机器要重量，所以不进 git（.gitignore）。",
    "# 列（制表符分隔）：峰值字节、量的时刻（UTC）、那一次的上限、键（命令的各个词用空格接起来，测试二进制名里的 16 位哈希去掉；RUN_WITH_MEMORY_CAP_KEY 可以指定）。",
]


def human(byte_count):
    if byte_count >= 1024 ** 3:
        return f"{byte_count / 1024 ** 3:.1f} GiB"
    return f"{byte_count / 1024 ** 2:.0f} MiB"


def unreclaimable_bytes(cgroup_directory):
    """cgroup 里回收不掉的用量：memory.current 减 active_file 与 inactive_file（依据见文件头「排队」一段）。
    目录不在（scope 还没起，或已经收尾删了）交 0；目录在而文件读不了交 None。"""
    try:
        with open(os.path.join(cgroup_directory, "memory.current"), encoding="utf-8") as handle:
            current = int(handle.read())
        file_pages = 0
        with open(os.path.join(cgroup_directory, "memory.stat"), encoding="utf-8") as handle:
            for line in handle:
                name, value = line.split()
                if name in ("active_file", "inactive_file"):
                    file_pages += int(value)
    except FileNotFoundError:
        return None if os.path.isdir(cgroup_directory) else 0
    except (OSError, ValueError):
        return None
    return max(0, current - file_pages)


def process_start_time(process_id):
    """/proc/<pid>/stat 的第 22 段（进程起始时刻，开机以来的时钟滴答）：进程号被复用时它不同。进程不在交 None。"""
    try:
        with open(f"/proc/{process_id}/stat", encoding="utf-8") as handle:
            return handle.read().rsplit(")", 1)[1].split()[19]
    except (OSError, IndexError):
        return None


@contextlib.contextmanager
def admission_lock():
    if broken == "nolock":
        yield
        return
    with open(lock_path, "a", encoding="utf-8") as handle:
        fcntl.flock(handle, fcntl.LOCK_EX)
        try:
            yield
        finally:
            fcntl.flock(handle, fcntl.LOCK_UN)


def live_ledger_entries():
    """账上还在跑的每一笔；包装已经死了（进程号不在、或起始时刻对不上）的那一笔删掉。"""
    entries = []
    for name in sorted(os.listdir(ledger_directory)):
        if not name.endswith(".json"):
            continue
        path = os.path.join(ledger_directory, name)
        try:
            with open(path, encoding="utf-8") as handle:
                entry = json.load(handle)
        except (OSError, ValueError):
            continue   # 刚删掉的；写的时候先写临时文件再改名，读不到半截
        if broken != "keepstale" and process_start_time(entry["pid"]) != entry["pid_start_time"]:
            with contextlib.suppress(FileNotFoundError):
                os.remove(path)
            continue
        entries.append(entry)
    return entries


def read_peak_table():
    rows = {}
    try:
        with open(peak_table_path, encoding="utf-8") as handle:
            lines = handle.read().splitlines()
    except FileNotFoundError:
        return rows
    for line in lines:
        fields = line.split("\t")
        if line.startswith("#") or len(fields) != 4 or not fields[0].isdigit():
            continue
        rows[fields[3]] = (int(fields[0]), fields[1], fields[2])
    return rows


def write_peak_table(rows):
    lines = PEAK_TABLE_HEADER + [f"{peak}\t{measured}\t{cap}\t{key}" for key, (peak, measured, cap) in sorted(rows.items())]
    temporary = f"{peak_table_path}.{os.getpid()}.partial"
    with open(temporary, "w", encoding="utf-8") as handle:
        handle.write("\n".join(lines) + "\n")
    os.replace(temporary, peak_table_path)


def need_of(key, cap_bytes, cap_text):
    """这一条要的量与一句为什么；依据见文件头「排队」一段。"""
    if broken == "ignoretable":
        return cap_bytes, f"按它的上限 {cap_text} 算（弄坏开关 ignoretable）"
    recorded = read_peak_table().get(key)
    if recorded is None:
        if broken == "nodefault":
            return 0, "峰值表里没有这条，按 0 算（弄坏开关 nodefault）"
        return cap_bytes, f"峰值表里没有这条，按它的上限 {cap_text} 算"
    need = min(recorded[0], cap_bytes)
    return need, f"按峰值表里上一次实测的峰值 {human(recorded[0])}（{recorded[1]} 量的）算"


def occupants_text(entries, slice_directory):
    if not entries:
        return ["      （账上没有在跑的；已占的全是 slice 里别的用量，多半是刚收尾还没放干净的，或不经这个包装挂进 slice 的进程）"]
    return [f"      {entry['unit']}：要 {human(entry['need'])}，此刻占 {human(unreclaimable_bytes(os.path.join(slice_directory, entry['unit'] + '.scope')) or 0)}，"
            f"包装进程 {entry['pid']}，{entry['admitted_at']} 起跑，键「{entry['key'][:120]}」" for entry in entries]


def admit(unit, cap_bytes, cap_text, total_bytes, slice_directory, key, wrapper_pid):
    os.makedirs(ledger_directory, exist_ok=True)
    wait_seconds = float(os.environ["RUN_WITH_MEMORY_CAP_WAIT_SECONDS_RESOLVED"])
    pause_seconds = float(os.environ.get("RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE") or 0)
    with admission_lock():
        need, need_text = need_of(key, cap_bytes, cap_text)
    if need > total_bytes:
        print(f"  ✗ 内存不够排不上：这一条要 {human(need)}（{need_text}），比 slice 的总上限 {human(total_bytes)} 还大，等多久都放不下；命令没跑", file=sys.stderr)
        print("  → 怎么办：上限写小一点再跑（真要这么多就先调小余量 RUN_WITH_MEMORY_CAP_RESERVE，依据写在 research/scripts/run-with-memory-cap.sh 文件头）", file=sys.stderr)
        return MEMORY_ADMISSION_REFUSED_EXIT
    started = time.monotonic()
    last_report = None
    while True:
        with admission_lock():
            entries = live_ledger_entries()
            slice_used = unreclaimable_bytes(slice_directory)
            if slice_used is None:
                print(f"  ✗ 读不了 slice 的用量（{slice_directory} 下的 memory.current / memory.stat），判不了放不放得下；命令没跑", file=sys.stderr)
                print("  → 怎么办：systemctl --user status 看用户级 systemd 还在不在、slice 还在不在；修好再跑，别绕开排队去跑", file=sys.stderr)
                return MEMORY_CAP_UNAVAILABLE_EXIT
            reserved = 0 if broken == "noledger" else sum(
                max(0, entry["need"] - (unreclaimable_bytes(os.path.join(slice_directory, entry["unit"] + ".scope")) or 0)) for entry in entries)
            occupied = slice_used + reserved
            if occupied + need <= total_bytes:
                if pause_seconds:
                    time.sleep(pause_seconds)
                entry = {"unit": unit, "pid": wrapper_pid, "pid_start_time": process_start_time(wrapper_pid), "need": need, "cap": cap_text,
                         "key": key, "admitted_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")}
                temporary = os.path.join(ledger_directory, f".{unit}.partial")
                with open(temporary, "w", encoding="utf-8") as handle:
                    json.dump(entry, handle, ensure_ascii=False)
                os.replace(temporary, os.path.join(ledger_directory, unit + ".json"))
                if last_report is not None:
                    print(f"run-with-memory-cap: 排了 {time.monotonic() - started:.0f} 秒，起跑（{unit}）", file=sys.stderr)
                return 0
        now = time.monotonic()
        if now - started >= wait_seconds:
            print(f"  ✗ 内存不够排不上：等了 {now - started:.0f} 秒（上限 RUN_WITH_MEMORY_CAP_WAIT_SECONDS={wait_seconds:g}），slice 已占 {human(occupied)}"
                  f"（回收不掉的用量 {human(slice_used)} + 账上还没涨到量的 {human(reserved)}），这一条要 {human(need)}（{need_text}），总上限 {human(total_bytes)}；命令没跑。账上占着的：",
                  file=sys.stderr)
            for line in occupants_text(entries, slice_directory):
                print(line, file=sys.stderr)   # gate-lint:detail
            print("  → 怎么办：等上面那几条跑完再跑这一条（或把 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 设长）；这一条的上限写得比它真要的大，就写小一点；"
                  "别绕开这个包装去跑——排不上说明此刻整机放不下，硬起就是再来一次整机 OOM", file=sys.stderr)
            return MEMORY_ADMISSION_REFUSED_EXIT
        if last_report is None or now - last_report >= WAITING_REPORT_EVERY_SECONDS:
            print(f"run-with-memory-cap: 排队（已等 {now - started:.0f} 秒，至多 {wait_seconds:g} 秒）：slice 已占 {human(occupied)}，这一条要 {human(need)}"
                  f"（{need_text}），总上限 {human(total_bytes)}；账上在跑 {len(entries)} 条", file=sys.stderr)
            last_report = now
        time.sleep(ADMISSION_POLL_SECONDS)


def finish(unit, key, cap_text, peak_text):
    """scope 收尾之后：账上这一笔删掉，量到了峰值就记进峰值表。
    峰值按 MiB 向上取整记，与表里这一行记的相同就不写：整份重写要先写临时文件再改名，盘忙的时候一次要一两秒
    （2026-09-25 实测 IO 压力 full avg10 约 14% 时，写临时文件加改名 0.03–1.8 秒），而撞了上限的那几条每次记的都是上限。"""
    with contextlib.suppress(FileNotFoundError):
        os.remove(os.path.join(ledger_directory, unit + ".json"))
    if not peak_text:
        return 0
    peak = -(-int(peak_text) // MEBIBYTE) * MEBIBYTE
    try:
        with admission_lock():
            rows = read_peak_table()
            if key in rows and rows[key][0] == peak:
                return 0
            rows[key] = (peak, datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"), cap_text)
            write_peak_table(rows)
    except OSError as error:
        print(f"run-with-memory-cap: 峰值没记进 {peak_table_path}（{error}），这一条的退出码照旧；下一次它按上限排队", file=sys.stderr)
    return 0


def meminfo():
    values = {}
    with open("/proc/meminfo", encoding="utf-8") as handle:
        for line in handle:
            name, rest = line.split(":", 1)
            values[name] = int(rest.split()[0]) * 1024
    return values


def cgroup_stat(path, names):
    try:
        with open(path, encoding="utf-8") as handle:
            stat = dict(line.split() for line in handle)
        return sum(int(stat.get(name, 0)) for name in names)
    except (OSError, ValueError):
        return None


def status(slice_name, total_text, reserve_text, slice_directory, cgroup_root):
    memory = meminfo()
    total_memory, available = memory["MemTotal"], memory["MemAvailable"]
    print(f"  … 整机内存 {human(total_memory)}（/proc/meminfo 的 MemTotal），此刻 MemAvailable {human(available)}")
    total_bytes = int(total_text) if total_text.isdigit() else None
    print(f"  … slice {slice_name} 的总上限 {human(total_bytes) if total_bytes else '算不出来'}（整机内存 − 余量 {reserve_text}；RUN_WITH_MEMORY_CAP_RESERVE 改余量）")
    slice_used = unreclaimable_bytes(slice_directory) if slice_directory else 0
    entries = live_ledger_entries() if os.path.isdir(ledger_directory) else []
    print(f"  … slice 里回收不掉的用量 {human(slice_used or 0)}{'' if slice_directory else '（slice 还没起）'}；账上在跑 {len(entries)} 条"
          f"{'：' if entries else ''}")
    for line in occupants_text(entries, slice_directory) if entries else []:
        print(line)
    outside = total_memory - available - (slice_used or 0)
    vllm = cgroup_stat(os.path.join(cgroup_root, "system.slice/vllm-prod.service/memory.stat"), ("anon", "shmem"))
    user = cgroup_stat(os.path.join(cgroup_root, "user.slice/memory.stat"), ("anon",))
    print(f"  … slice 外面回收不掉的合计约 {human(outside)}（MemTotal − MemAvailable − slice 里的用量），其中 vllm-prod 的 anon + shmem "
          f"{human(vllm) if vllm is not None else '读不到'}、user.slice 的 anon {human(user) if user is not None else '读不到'}")
    if total_bytes is not None:
        reserve_bytes = total_memory - total_bytes
        if outside > reserve_bytes:
            print(f"  ✗ slice 外面的用量已经超过余量 {human(reserve_bytes)}：slice 的总上限挡不住整机被外面涨满")
            print("  → 怎么办：查外面是谁在涨（systemd-cgtop、ps -eo rss,pid,cmd --sort=-rss）；是常态就调大 RUN_WITH_MEMORY_CAP_RESERVE 并改文件头的依据")
            return 1
        print(f"  ✓ 余量比 slice 外面此刻的用量多 {human(reserve_bytes - outside)}")
    return 0


operation, arguments = sys.argv[1], sys.argv[2:]
if operation == "admit":
    unit, cap_bytes, cap_text, total_bytes, slice_directory, key, wrapper_pid = arguments
    sys.exit(admit(unit, int(cap_bytes), cap_text, int(total_bytes), slice_directory, key, int(wrapper_pid)))
if operation == "finish":
    sys.exit(finish(*arguments))
if operation == "status":
    sys.exit(status(*arguments))
print(f"  ✗ admission 不认识的动作 {operation}", file=sys.stderr)
print("  → 怎么办：这是 run-with-memory-cap.sh 自己的内部调用写错了，看调 admission 的那一行", file=sys.stderr)
sys.exit(2)
PY
}

# scope 的结局：命令退出之后查 systemd 记的 Result，直到 scope 收尾（不在了、inactive、failed）或查满次数。
# 打印 Result（oom-kill、timeout、success 等）；systemctl 本身出错时打印 unreadable。
# 读到 oom-kill 也要等 scope 收尾再走：它那时多半还在 deactivating，不等它落到 failed 就 reset-failed 不掉，撞顶的 scope 会一直挂着。
scope_result() {
  local unit="$1" attempt properties load_state active_state result="" oom_seen=""
  for ((attempt = 1; attempt <= SCOPE_RESULT_POLL_ATTEMPTS; attempt++)); do
    if ! properties="$(systemctl --user show -p LoadState -p ActiveState -p Result "$unit.scope" 2>/dev/null)"; then
      echo "unreadable"
      return
    fi
    load_state="$(sed -n 's/^LoadState=//p' <<<"$properties")"
    active_state="$(sed -n 's/^ActiveState=//p' <<<"$properties")"
    result="$(sed -n 's/^Result=//p' <<<"$properties")"
    [[ "$result" == "oom-kill" ]] && oom_seen="oom-kill"
    if [[ "$load_state" == "not-found" || "$active_state" == "inactive" || "$active_state" == "failed" ]]; then
      break
    fi
    sleep 0.1
  done
  [[ -n "$oom_seen" ]] && result="$oom_seen"
  if [[ "$active_state" == "failed" ]]; then
    systemctl --user reset-failed "$unit.scope" >/dev/null 2>&1   # 撞顶、超时的 scope 停在 failed，不清掉就一直挂在用户级 systemd 里
  fi
  echo "${result:-unreadable}"
}

peak_file_value() { # peak_file_value <峰值文件> <名字> → 那一行的数；没有就空
  [[ -f "$1" ]] && awk -v wanted="$2" '$1 == wanted {print $2}' "$1"
}

# run_capped <上限> <run|check> <命令…>：check 只试起不起得来，不排队、不记峰值、不限时
run_capped() {
  local cap="$1" mode="$2" cap_bytes unit started_marker peak_file key exit_code result use_slice=1 admission_status
  local peak_bytes own_oom_count oom_kill_count scope_properties
  shift 2
  if [[ "$BROKEN_JUDGEMENT" == "nocap" ]]; then
    "$@"
    return
  fi
  cap_bytes="$(size_in_bytes "$cap")" || cap_bytes=0   # 只有 --check 走得到 0：跑命令的入口先判过上限不超过 64 位
  prepare_state_directory || report_unavailable "账与锁的目录 $STATE_DIRECTORY 建不了（mkdir 或写 lock 失败），命令一行都没跑" \
    "查 \$XDG_RUNTIME_DIR（$STATE_DIRECTORY 的上一层）在不在、可写不可写；或设 RUN_WITH_MEMORY_CAP_STATE_DIR 到一个可写的目录。不许绕开排队去跑"
  if [[ "$BROKEN_JUDGEMENT" == "noslice" ]]; then
    use_slice=0
  elif ! ensure_slice; then
    if [[ "$BROKEN_JUDGEMENT" == "slicefallback" ]]; then
      use_slice=0
    else
      report_unavailable "slice $SLICE 的总上限设不上：$SLICE_PROBLEM；命令一行都没跑" \
        "照上面那句修（systemctl --user status、cat /sys/fs/cgroup/user.slice/user-\$(id -u).slice/user@\$(id -u).service/cgroup.subtree_control 里要有 memory）；修好之前别跑这一步，不许退回无总上限去跑"
    fi
  fi
  unit="singlefs-memory-cap-$$-$RANDOM$RANDOM"
  key="$(command_key "$@")"
  if [[ "$mode" == "run" && $use_slice -eq 1 ]]; then
    admission admit "$unit" "$cap_bytes" "$cap" "$SLICE_TOTAL_BYTES" "$SLICE_DIRECTORY" "$key" "$$"
    admission_status=$?
    if [[ $admission_status -eq $MEMORY_ADMISSION_REFUSED_EXIT ]]; then
      return "$MEMORY_ADMISSION_REFUSED_EXIT"
    fi
    if [[ $admission_status -ne 0 ]]; then
      report_unavailable "排队那一步退 $admission_status（上面是它的报错），命令一行都没跑" "照上面那几行修；不许绕开排队去跑"
    fi
  fi
  started_marker="${TMPDIR:-/tmp}/run-with-memory-cap-$$-$RANDOM$RANDOM.started"
  peak_file="$STATE_DIRECTORY/peaks/$unit"
  # 被外层 timeout 连同进程组一起停的时候，也把标记、账上这一笔与峰值文件删掉
  trap 'rm -f -- "$started_marker" "$STATE_DIRECTORY/ledger/$unit.json" "$peak_file" "$peak_file.partial"; exit 143' TERM
  trap 'rm -f -- "$started_marker" "$STATE_DIRECTORY/ledger/$unit.json" "$peak_file" "$peak_file.partial"; exit 130' INT
  scope_properties=(-p MemoryMax="$cap" -p MemorySwapMax=0 -p OOMPolicy=stop)
  ((use_slice)) && scope_properties+=(--slice="$SLICE")
  if [[ "$mode" == "run" && -n "$TIME_LIMIT_SECONDS" && "$BROKEN_JUDGEMENT" != "notimelimit" ]]; then
    scope_properties+=(-p RuntimeMaxSec="$TIME_LIMIT_SECONDS" -p TimeoutStopSec="$KILL_GRACE_SECONDS")
  fi
  # 外壳里先建标记再跑命令：标记在，说明命令确实是在上限里起的；不在，说明 systemd-run 自己没起来。
  # 外面那层把本 shell 的 stderr 临时指到 /dev/null：外壳被 SIGKILL 时 bash 会补一行「… Killed  systemd-run …」，那一行不是命令的输出；
  # 命令自己的 stderr 经 fd 3 照旧接回原来的 stderr。只管这一条的三个变量在这里清掉，不传进命令
  { env -u RUN_WITH_MEMORY_CAP_KEY -u RUN_WITH_MEMORY_CAP_TIME_LIMIT -u RUN_WITH_MEMORY_CAP_WAIT_SECONDS \
      systemd-run --user --scope --unit="$unit" "${scope_properties[@]}" --quiet \
      bash -c "$SCOPE_HOLDER" run-with-memory-cap "$started_marker" "$peak_file" "$CGROUP_ROOT" "$@" 2>&3 3>&-; } 3>&2 2>/dev/null
  exit_code=$?
  if [[ ! -e "$started_marker" ]]; then
    rm -f -- "$STATE_DIRECTORY/ledger/$unit.json" "$peak_file" "$peak_file.partial"
    trap - TERM INT
    if [[ "$BROKEN_JUDGEMENT" == "fallback" ]]; then
      "$@"
      return
    fi
    report_unavailable "systemd-run --user --scope 退出码 $exit_code，命令一行都没跑（上面是 systemd-run 自己的报错）" \
      "查用户级 systemd 与 D-Bus（systemctl --user status、echo \$DBUS_SESSION_BUS_ADDRESS \$XDG_RUNTIME_DIR）；起不来就别跑这一步，不许退回无上限去跑"
  fi
  rm -f -- "$started_marker"
  trap - TERM INT
  result="$(scope_result "$unit")"
  peak_bytes="$(peak_file_value "$peak_file" peak)"
  own_oom_count="$(peak_file_value "$peak_file" oom)"
  oom_kill_count="$(peak_file_value "$peak_file" oom_kill)"
  rm -f -- "$peak_file" "$peak_file.partial"
  if [[ "$mode" != "run" ]]; then
    rm -f -- "$STATE_DIRECTORY/ledger/$unit.json"
  fi
  if [[ "$BROKEN_JUDGEMENT" == "exitcode" ]]; then
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" "$peak_bytes"
    if [[ $exit_code -eq 137 || $exit_code -eq 143 ]]; then
      echo "run-with-memory-cap: 撞了内存上限 $cap（弄坏开关 exitcode：按退出码判）" >&2
      return "$MEMORY_CAP_HIT_EXIT"
    fi
    return "$exit_code"
  fi
  if [[ "$result" == "unreadable" ]]; then
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" ""
    report_unavailable "命令退出码 $exit_code，之后查不到 scope $unit 的结局（systemctl --user show 出错），判不了是不是撞了上限" \
      "查用户级 systemd 还在不在（systemctl --user status）；这一条的结果不算数，修好再重跑"
  fi
  if [[ "$result" == "oom-kill" && "$BROKEN_JUDGEMENT" != "noresult" ]]; then
    if ((use_slice)) && [[ "$own_oom_count" == "0" && "${oom_kill_count:-0}" != "0" && "$BROKEN_JUDGEMENT" != "noslicehit" ]]; then
      [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" ""
      echo "run-with-memory-cap: 被 slice $SLICE 的总上限挤掉了（scope $unit 的 Result=oom-kill，而它自己没撞它的上限 $cap：整个 slice 满了，内核在 slice 里挑了这一条杀；命令退出码 $exit_code）" >&2
      echo "run-with-memory-cap: → 这一条的结果不算数，重跑它；常这样说明排队按峰值表放进来的几条实际占的比表里记的多，bash $SELF_PATH --status 看 slice 里是谁" >&2
      return "$SLICE_TOTAL_HIT_EXIT"
    fi
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" "$cap_bytes"
    echo "run-with-memory-cap: 撞了内存上限 $cap（scope $unit 的 Result=oom-kill，命令退出码 $exit_code）" >&2
    return "$MEMORY_CAP_HIT_EXIT"
  fi
  if [[ "$result" == "timeout" ]]; then
    [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" ""
    echo "run-with-memory-cap: 超过限时 ${TIME_LIMIT_SECONDS} 秒（scope $unit 的 Result=timeout，systemd 停掉了 scope 里的进程；命令退出码 $exit_code）" >&2
    return "$TIME_LIMIT_HIT_EXIT"
  fi
  [[ "$mode" == "run" ]] && admission finish "$unit" "$key" "$cap" "$peak_bytes"
  return "$exit_code"
}

show_status() {
  local total cgroup_path slice_directory=""
  check_size_syntax "$RESERVE" "余量 RUN_WITH_MEMORY_CAP_RESERVE "
  [[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
  total="$(slice_total_bytes)" || total="unknown"
  cgroup_path="$(systemctl --user show -p ControlGroup --value "$SLICE" 2>/dev/null)"
  [[ -n "$cgroup_path" ]] && slice_directory="$CGROUP_ROOT$cgroup_path"
  admission status "$SLICE" "$total" "$RESERVE" "$slice_directory" "$CGROUP_ROOT"
}

# ── 自证 ──
# 吃内存的探针：每次拿 16 MiB 写满（真占常驻页），拿到 MEBIBYTES 就停下正常退出；上限 64M 时它在半路被杀
bounded_memory_hog_code() {
  printf '%s\n' "import sys" "held = []" "for _ in range(int(sys.argv[1]) // 16):" "    held.append(b'\\x01' * (16 * 1024 * 1024))" \
    "print('allocated', len(held) * 16, 'MiB')"
}

# 排队那几例用的作业：起跑时刻写进第一个文件，等一会儿、拿够内存、再占一会儿，收尾时刻写进第二个文件
selftest_job_code() {
  printf '%s\n' "import sys, time" "start_path, end_path = sys.argv[1], sys.argv[2]" \
    "delay_seconds, mebibytes, hold_seconds = float(sys.argv[3]), int(sys.argv[4]), float(sys.argv[5])" \
    "open(start_path, 'w').write(repr(time.time()))" "time.sleep(delay_seconds)" \
    "held = [bytes([1]) * (1024 * 1024) for _ in range(mebibytes)]" "time.sleep(hold_seconds)" \
    "open(end_path, 'w').write(repr(time.time()))"
}

run_selftest() {
  local failures=0 checked=0 scratch output status marker hog_code job_code target slice_counter=0 slice context_directory
  local first_pid second_pid first_status second_status started_seconds elapsed_seconds
  local -a selftest_slices=() context=()
  target="${RUN_WITH_MEMORY_CAP_SELFTEST_TARGET:-$SELF_PATH}"
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/run-with-memory-cap-selftest-XXXXXX")"
  hog_code="$(bounded_memory_hog_code)"
  job_code="$(selftest_job_code)"
  fail() { echo "  ✗ 自检：$1"; failures=$((failures + 1)); }   # gate-lint:detail
  # 每一例一个新 slice（名字不带 -：- 是 systemd 的层级）、新的账与峰值表；收尾时 stop 加 revert 掉
  fresh_context() { # fresh_context <总上限>
    slice_counter=$((slice_counter + 1))
    slice="singlefs_memory_selftest_$$_${slice_counter}.slice"
    selftest_slices+=("$slice")
    context_directory="$scratch/context-$slice_counter"
    mkdir -p "$context_directory"
    context=(RUN_WITH_MEMORY_CAP_SLICE="$slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL="$1" RUN_WITH_MEMORY_CAP_STATE_DIR="$context_directory/state"
             RUN_WITH_MEMORY_CAP_PEAKS="$context_directory/peaks.tsv")
  }
  in_context() { env "${context[@]}" "$@"; }
  intervals_overlap() { # intervals_overlap <前一条起> <前一条收> <后一条起> <后一条收>：四个文件都在、两段时间有交集才退 0
    python3 -c 'import sys; a0, a1, b0, b1 = (float(open(path).read()) for path in sys.argv[1:5]); sys.exit(0 if a0 < b1 and b0 < a1 else 1)' "$@" 2>/dev/null
  }
  all_exist() { local path; for path in "$@"; do [[ -f "$path" ]] || return 1; done; }
  seed_peak() { # seed_peak <键> <字节>：往这一例的峰值表里写一行
    printf '%s\t%s\t%s\t%s\n' "$2" "2026-09-25T00:00:00Z" "200M" "$1" >> "$context_directory/peaks.tsv"
  }

  fresh_context 512M
  checked=$((checked + 1))
  if ! in_context bash "$target" --check 64M 2>"$scratch/check.err"; then
    fail "--check 64M 在这台机器上应当起得来，实际起不来：$(cat "$scratch/check.err")"
  fi

  checked=$((checked + 1))
  if in_context bash "$target" 64M python3 -c "$hog_code" 512 >"$scratch/hog.out" 2>"$scratch/hog.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]] || ! grep -q '撞了内存上限 64M' "$scratch/hog.err"; then
    fail "上限 64M 里分配 512 MiB 的探针应当退 $MEMORY_CAP_HIT_EXIT 并报「撞了内存上限 64M」，实际退 $status：$(cat "$scratch/hog.out" "$scratch/hog.err")"
  fi

  # 吃内存的是孙进程、父进程被杀之后自己还要退 101：照样按 Result 判撞顶
  checked=$((checked + 1))
  if in_context bash "$target" 64M bash -c 'python3 -c "$1" 512; exit 101' hog-parent "$hog_code" >"$scratch/grandchild.out" 2>"$scratch/grandchild.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_HIT_EXIT ]]; then
    fail "吃内存的是孙进程时应当照样退 $MEMORY_CAP_HIT_EXIT（按 scope 的 Result 判，不按父进程的退出码），实际退 $status：$(cat "$scratch/grandchild.err")"
  fi

  # 不是撞顶的 SIGKILL（自己 kill -9 自己）退 137，不许判成撞顶
  checked=$((checked + 1))
  if in_context bash "$target" 64M bash -c 'kill -9 $$' 2>"$scratch/sigkill.err"; then status=0; else status=$?; fi
  if [[ $status -ne 137 || "$(cat "$scratch/sigkill.err")" == *Killed* ]]; then
    fail "上限之内被 kill -9 的命令应当原样退 137、不算撞顶，stderr 里不多出外壳补的「Killed」那一行，实际退 $status：$(cat "$scratch/sigkill.err")"
  fi

  # 正常退出码原样传回
  for wanted in 0 7 101; do
    checked=$((checked + 1))
    if in_context bash "$target" 64M bash -c "exit $wanted"; then status=0; else status=$?; fi
    [[ $status -eq $wanted ]] || fail "命令退 $wanted 时应当原样传回 $wanted，实际 $status"
  done

  # 上限真的设上了：scope 里读自己 cgroup 的 memory.max 与 memory.swap.max
  checked=$((checked + 1))
  output="$(in_context bash "$target" 64M bash -c 'group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"; cat "$group/memory.max" "$group/memory.swap.max"' 2>&1)"
  if [[ "$output" != $'67108864\n0' ]]; then
    fail "scope 里的 memory.max、memory.swap.max 应当是 67108864 与 0，实际「$output」"
  fi

  # 总上限（第 ② 层）真的设上了：scope 挂在这一例的 slice 底下，slice 的 memory.max 是 512M
  checked=$((checked + 1))
  output="$(in_context bash "$target" 64M bash -c 'group="$(cut -d: -f3 /proc/self/cgroup)"; echo "$group"; cat "/sys/fs/cgroup$(dirname "$group")/memory.max"' 2>&1)"
  if [[ "$output" != *"/$slice/"*$'\n536870912' ]]; then
    fail "scope 应当挂在 slice $slice 底下、slice 的 memory.max 是 536870912（512M），实际「$output」"
  fi

  # slice 设不上（cgroup 里读回来的不是那个数，这里用一个不存在的 cgroup 根演）：退 251、报 slice、命令没跑，不退回无总上限
  checked=$((checked + 1))
  marker="$scratch/ran-without-slice"
  if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$scratch/no-cgroup-here" bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/noslice.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q 'slice' "$scratch/noslice.err"; then
    fail "slice 的总上限设不上时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报 slice、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/noslice.err")"
  fi
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_CGROUP_ROOT="$scratch/no-cgroup-here" bash "$target" --check 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "slice 的总上限设不上时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"
  # 余量比整机内存还大：总上限算不出来，同样退 251、命令没跑
  checked=$((checked + 1))
  marker="$scratch/ran-with-oversized-reserve"
  if in_context env RUN_WITH_MEMORY_CAP_SLICE_TOTAL= RUN_WITH_MEMORY_CAP_RESERVE=1024T bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/reserve.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]]; then
    fail "余量 1024T 比整机内存还大时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/reserve.err")"
  fi

  # systemd-run 起不来：退 251、报错，而且那条命令一行都没跑（不退回无上限）
  checked=$((checked + 1))
  marker="$scratch/ran-without-cap"
  if in_context env DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent \
      bash "$target" 64M bash -c ': > "$1"' touch-marker "$marker" 2>"$scratch/unavailable.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT || -e "$marker" ]] || ! grep -q '起不来' "$scratch/unavailable.err"; then
    fail "D-Bus 连不上时应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报「起不来」、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/unavailable.err")"
  fi
  checked=$((checked + 1))
  if in_context env DBUS_SESSION_BUS_ADDRESS=unix:path=/nonexistent XDG_RUNTIME_DIR=/nonexistent \
      bash "$target" --check 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq $MEMORY_CAP_UNAVAILABLE_EXIT ]] || fail "D-Bus 连不上时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT，实际 $status"
  # slice 设得上、而 systemd-run 自己起不来（systemd 不认这个上限：换成字节超过 64 位）：--check 退 251，不退回无上限去跑
  checked=$((checked + 1))
  if in_context bash "$target" --check 99999999999999T 2>"$scratch/scope-refused.err"; then status=0; else status=$?; fi
  if [[ $status -ne $MEMORY_CAP_UNAVAILABLE_EXIT ]] || ! grep -q 'systemd-run' "$scratch/scope-refused.err"; then
    fail "slice 设得上而 systemd-run 不认上限 99999999999999T 时 --check 应当退 $MEMORY_CAP_UNAVAILABLE_EXIT、报 systemd-run，实际退 $status：$(cat "$scratch/scope-refused.err")"
  fi

  # 两条同时来、都判放得下（第 ③ 层的锁）：总上限 256M，两条各要 200M（峰值表里没有、按上限算），判完停 0.6 秒再记账把竞态撑开；
  # 应当一条跑完另一条才起，两条都退 0。判与记账不在一把锁里时两条都放进来、合起来 300 MiB 超总上限
  fresh_context 256M
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE=0.6 bash "$target" 200M python3 -c "$job_code" "$scratch/lock-1.start" "$scratch/lock-1.end" 0.2 150 1.0 \
    >/dev/null 2>"$scratch/lock-1.err" &
  first_pid=$!
  in_context env RUN_WITH_MEMORY_CAP_SELFTEST_ADMIT_PAUSE=0.6 bash "$target" 200M python3 -c "$job_code" "$scratch/lock-2.start" "$scratch/lock-2.end" 0.2 150 1.0 \
    >/dev/null 2>"$scratch/lock-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne 0 || $second_status -ne 0 ]] || ! all_exist "$scratch"/lock-{1,2}.{start,end} \
      || intervals_overlap "$scratch/lock-1.start" "$scratch/lock-1.end" "$scratch/lock-2.start" "$scratch/lock-2.end"; then
    fail "两条同时来（各要 200M、总上限 256M）应当一条一条跑、都退 0，实际退 $first_status 与 $second_status、$(intervals_overlap "$scratch/lock-1.start" "$scratch/lock-1.end" "$scratch/lock-2.start" "$scratch/lock-2.end" && echo '两条同时在跑' || echo '没有同时在跑')：$(cat "$scratch/lock-1.err" "$scratch/lock-2.err")"
  fi

  # 前一条放行了、还没涨到量（先睡 1 秒再拿 150 MiB）时后一条来：只看 slice 的用量两条都判得下、合起来超；
  # 账上还没涨到量的那部分算进已占，后一条就等前一条跑完
  fresh_context 256M
  checked=$((checked + 1))
  in_context bash "$target" 200M python3 -c "$job_code" "$scratch/ramp-1.start" "$scratch/ramp-1.end" 1.0 150 0.8 >/dev/null 2>"$scratch/ramp-1.err" &
  first_pid=$!
  sleep 0.4
  in_context bash "$target" 200M python3 -c "$job_code" "$scratch/ramp-2.start" "$scratch/ramp-2.end" 0 150 0.8 >/dev/null 2>"$scratch/ramp-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne 0 || $second_status -ne 0 ]] || ! all_exist "$scratch"/ramp-{1,2}.{start,end} \
      || intervals_overlap "$scratch/ramp-1.start" "$scratch/ramp-1.end" "$scratch/ramp-2.start" "$scratch/ramp-2.end"; then
    fail "前一条还没涨到量时来的后一条应当等它跑完、两条都退 0，实际退 $first_status 与 $second_status、$(intervals_overlap "$scratch/ramp-1.start" "$scratch/ramp-1.end" "$scratch/ramp-2.start" "$scratch/ramp-2.end" && echo '两条同时在跑' || echo '没有同时在跑')：$(cat "$scratch/ramp-1.err" "$scratch/ramp-2.err")"
  fi

  # 峰值表里没有这条：按它的上限算。前一条（表里记 100 MiB）在跑，后一条表里没有、上限 200M：100 + 200 放不进 256M，
  # 等 1 秒排不上，退 252、命令没跑、说清是按上限算的
  fresh_context 256M
  checked=$((checked + 1))
  seed_peak selftest-recorded $((100 * MEBIBYTE))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-recorded bash "$target" 200M python3 -c "$job_code" "$scratch/table-1.start" "$scratch/table-1.end" 0 10 2.5 \
    >/dev/null 2>"$scratch/table-1.err" &
  first_pid=$!
  sleep 0.6
  marker="$scratch/ran-unrecorded"
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-unrecorded RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1 bash "$target" 200M bash -c ': > "$1"' touch-marker "$marker" \
      2>"$scratch/unrecorded.err"; then status=0; else status=$?; fi
  wait "$first_pid"; first_status=$?
  if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || -e "$marker" ]] || ! grep -q '排不上' "$scratch/unrecorded.err" || ! grep -q '峰值表里没有这条，按它的上限 200M 算' "$scratch/unrecorded.err"; then
    fail "峰值表里没有的那一条应当按上限 200M 算、排不上退 $MEMORY_ADMISSION_REFUSED_EXIT、命令不跑，实际退 $status、命令$([[ -e "$marker" ]] && echo '跑了' || echo '没跑')：$(cat "$scratch/unrecorded.err")"
  fi
  [[ $first_status -eq 0 ]] || fail "表里记着 100 MiB 的那一条应当照常跑完退 0，实际退 $first_status：$(cat "$scratch/table-1.err")"

  # 要的量比总上限还大：等多久都放不下，不等，立刻退 252
  checked=$((checked + 1))
  started_seconds="$(date +%s%N)"
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-oversized bash "$target" 512M true 2>"$scratch/oversized.err"; then status=0; else status=$?; fi
  elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
  if [[ $status -ne $MEMORY_ADMISSION_REFUSED_EXIT || $elapsed_seconds -ge 5 ]] || ! grep -q '比 slice 的总上限' "$scratch/oversized.err"; then
    fail "要 512M 而总上限 256M 时应当不等、立刻退 $MEMORY_ADMISSION_REFUSED_EXIT，实际退 $status、用了 $elapsed_seconds 秒：$(cat "$scratch/oversized.err")"
  fi

  # 峰值记进表，下一次按它算：跑一条拿 30 MiB 的，表里这个键记下的峰值在 30 MiB 与上限 200M 之间；
  # 之后两条同样的同时来，各按记下的量算，都放得下、同时在跑
  fresh_context 256M
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-small bash "$target" 200M python3 -c "$job_code" "$scratch/small-0.start" "$scratch/small-0.end" 0 30 0 \
      2>"$scratch/small-0.err"; then status=0; else status=$?; fi
  output="$(awk -F '\t' '$4 == "selftest-small" {print $1}' "$context_directory/peaks.tsv" 2>/dev/null)"
  if [[ $status -ne 0 || ! "$output" =~ ^[0-9]+$ ]] || ((output < 30 * MEBIBYTE || output > 200 * MEBIBYTE)); then
    fail "跑完一条拿 30 MiB 的，峰值表里 selftest-small 应当记下 30 MiB 到 200M 之间的峰值，实际退 $status、记的是「$output」：$(cat "$scratch/small-0.err")"
  fi
  checked=$((checked + 1))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-small bash "$target" 200M python3 -c "$job_code" "$scratch/small-1.start" "$scratch/small-1.end" 0 30 1.2 \
    >/dev/null 2>"$scratch/small-1.err" &
  first_pid=$!
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-small bash "$target" 200M python3 -c "$job_code" "$scratch/small-2.start" "$scratch/small-2.end" 0 30 1.2 \
    >/dev/null 2>"$scratch/small-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne 0 || $second_status -ne 0 ]] || ! intervals_overlap "$scratch/small-1.start" "$scratch/small-1.end" "$scratch/small-2.start" "$scratch/small-2.end"; then
    fail "峰值表里记着约 30 MiB 的两条（上限各 200M、总上限 256M）应当按记下的量一起放行、同时在跑，实际退 $first_status 与 $second_status、$(intervals_overlap "$scratch/small-1.start" "$scratch/small-1.end" "$scratch/small-2.start" "$scratch/small-2.end" && echo '同时在跑' || echo '一条一条跑的')：$(cat "$scratch/small-1.err" "$scratch/small-2.err")"
  fi

  # 外壳写峰值那一步挨得住 systemd 停 scope 时发给 scope 里每个进程的 TERM：命令在 scope 里留一个进程，等命令退出之后 1 到 2 秒里
  # 不停地（不 sleep）给 scope 里除它以外的每个进程发 TERM——外壳那时起的任何外部命令活不过一轮；峰值照样要记进表。
  # 只在这一例自己开的 scope 里发：cgroup 路径要是 …/<这一例的 slice>/singlefs-memory-cap-*.scope（slice 名经 $1 传进去），不对就不发信号、退 0。
  # 不开 scope 的路径（弄坏开关 nocap）下命令就在调用方的 cgroup 里：2026-09-25 UTC 11:12 在 SSH 会话的 session-1.scope 里发过一轮 TERM，
  # 打掉了 VSCode 扩展宿主与 Claude 会话；只核最后一段名字时，外面包着一层别的 singlefs-memory-cap scope（整条门禁经包装跑）会打到外层那一整条
  fresh_context 256M
  checked=$((checked + 1))
  if in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-term-storm bash "$target" 64M bash -c '
      group="/sys/fs/cgroup$(cut -d: -f3 /proc/self/cgroup)"
      case "$group" in
        */"$1"/singlefs-memory-cap-*.scope) ;;
        *) echo "不在这一例自己的 slice $1 底下自己开的 scope 里（$group），不发信号" >&2; exit 0 ;;
      esac
      command_pid=$$
      ( while kill -0 "$command_pid" 2>/dev/null; do :; done
        storm_end=$((SECONDS + 2))
        while ((SECONDS < storm_end)); do
          while read -r process_id; do [[ "$process_id" != "$BASHPID" ]] && kill -TERM "$process_id" 2>/dev/null; done < "$group/cgroup.procs"  # process-safety:own-scope 上面 case 核过路径在这一例自己的 slice 底下、是自己开的 singlefs-memory-cap-*.scope
        done ) >/dev/null 2>&1 &
      exit 0' term-storm "$slice" 2>"$scratch/term-storm.err"; then status=0; else status=$?; fi
  output="$(awk -F '\t' '$4 == "selftest-term-storm" {print $1}' "$context_directory/peaks.tsv" 2>/dev/null)"
  if [[ ! "$output" =~ ^[0-9]+$ ]]; then
    fail "scope 里一直有进程收 TERM 时，外壳照样要把峰值记进表（selftest-term-storm 那一行），实际退 $status、记的是「$output」：$(cat "$scratch/term-storm.err")"
  fi

  # 被总上限挤掉（第 ② 层兜住、退 254，不报成撞了自己的上限）：表里把这两条记成 10 MiB，排队一起放行，实际各拿 150 MiB，合起来超总上限 256M
  fresh_context 256M
  checked=$((checked + 1))
  seed_peak selftest-underrecorded $((10 * MEBIBYTE))
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-underrecorded bash "$target" 200M python3 -c "$job_code" "$scratch/squeeze-1.start" "$scratch/squeeze-1.end" 0.3 150 1.5 \
    >/dev/null 2>"$scratch/squeeze-1.err" &
  first_pid=$!
  in_context env RUN_WITH_MEMORY_CAP_KEY=selftest-underrecorded bash "$target" 200M python3 -c "$job_code" "$scratch/squeeze-2.start" "$scratch/squeeze-2.end" 0.3 150 1.5 \
    >/dev/null 2>"$scratch/squeeze-2.err" &
  second_pid=$!
  wait "$first_pid"; first_status=$?
  wait "$second_pid"; second_status=$?
  if [[ $first_status -ne $SLICE_TOTAL_HIT_EXIT && $second_status -ne $SLICE_TOTAL_HIT_EXIT ]] || [[ $first_status -eq $MEMORY_CAP_HIT_EXIT || $second_status -eq $MEMORY_CAP_HIT_EXIT ]] \
      || ! grep -q '总上限挤掉' "$scratch/squeeze-1.err" "$scratch/squeeze-2.err"; then
    fail "两条合起来超总上限 256M（各自没超上限 200M）时，被杀的那条应当退 $SLICE_TOTAL_HIT_EXIT、报「被总上限挤掉」，实际退 $first_status 与 $second_status：$(cat "$scratch/squeeze-1.err" "$scratch/squeeze-2.err")"
  fi

  # 限时：超过 RUN_WITH_MEMORY_CAP_TIME_LIMIT 退 253，不跑满命令自己的时长
  checked=$((checked + 1))
  started_seconds="$(date +%s%N)"
  if in_context env RUN_WITH_MEMORY_CAP_TIME_LIMIT=1 RUN_WITH_MEMORY_CAP_KILL_GRACE=2 bash "$target" 64M sleep 4 2>"$scratch/timelimit.err"; then status=0; else status=$?; fi
  elapsed_seconds=$((($(date +%s%N) - started_seconds) / 1000000000))
  if [[ $status -ne $TIME_LIMIT_HIT_EXIT || $elapsed_seconds -ge 4 ]] || ! grep -q '超过限时 1 秒' "$scratch/timelimit.err"; then
    fail "限时 1 秒跑 sleep 4 应当 4 秒之内退 $TIME_LIMIT_HIT_EXIT、报「超过限时 1 秒」，实际退 $status、用了 $elapsed_seconds 秒：$(cat "$scratch/timelimit.err")"
  fi

  # 账上包装已经死了的那一笔（进程号是 pid_max，这台机器上不会有这个进程）不算：记着要 250M 的死账不挡一条要 200M 的
  checked=$((checked + 1))
  mkdir -p "$context_directory/state/ledger"
  printf '{"unit": "singlefs-memory-cap-dead", "pid": %s, "pid_start_time": "1", "need": %s, "cap": "250M", "key": "dead", "admitted_at": "2026-09-25T00:00:00Z"}' \
    "$(cat /proc/sys/kernel/pid_max)" $((250 * MEBIBYTE)) > "$context_directory/state/ledger/singlefs-memory-cap-dead.json"
  if in_context env RUN_WITH_MEMORY_CAP_WAIT_SECONDS=1 bash "$target" 200M true 2>"$scratch/stale.err"; then status=0; else status=$?; fi
  if [[ $status -ne 0 || -e "$context_directory/state/ledger/singlefs-memory-cap-dead.json" ]]; then
    fail "账上那一笔的包装已经死了，应当删掉它、放行这一条退 0，实际退 $status、那一笔$([[ -e "$context_directory/state/ledger/singlefs-memory-cap-dead.json" ]] && echo '还在' || echo '删了')：$(cat "$scratch/stale.err")"
  fi

  # 收尾之后不留账、不留 scope：每一例的账是空的，每个 slice 底下没有 scope
  checked=$((checked + 1))
  output=""
  for context_directory in "$scratch"/context-*; do
    if compgen -G "$context_directory/state/ledger/*.json" >/dev/null; then output+=" $context_directory 的账没清空"; fi
  done
  for slice in "${selftest_slices[@]}"; do
    if compgen -G "/sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/$slice/*.scope" >/dev/null; then output+=" $slice 底下还有 scope"; fi
  done
  [[ -z "$output" ]] || fail "跑完应当不留账、不留 scope，实际：$output"

  # 用法错
  checked=$((checked + 1))
  if bash "$target" abc true 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq 2 ]] || fail "上限写成 abc 应当退 2，实际 $status"
  checked=$((checked + 1))
  if bash "$target" 64M 2>/dev/null; then status=0; else status=$?; fi
  [[ $status -eq 2 ]] || fail "没给命令应当退 2，实际 $status"
  # slice 名不是 singlefs 开头（别人的 slice）：退 2，不设上限、不起 scope。改前那一份会真去设，名字取一个不碍事的，收尾照样停掉、还原
  checked=$((checked + 1))
  foreign_slice="notsinglefs_selftest_$$.slice"
  selftest_slices+=("$foreign_slice")
  if env RUN_WITH_MEMORY_CAP_SLICE="$foreign_slice" RUN_WITH_MEMORY_CAP_SLICE_TOTAL=256M RUN_WITH_MEMORY_CAP_STATE_DIR="$scratch/foreign-state" \
      RUN_WITH_MEMORY_CAP_PEAKS="$scratch/foreign-peaks.tsv" bash "$target" --check 64M 2>"$scratch/foreign.err"; then status=0; else status=$?; fi
  if [[ $status -ne 2 ]] || ! grep -q 'singlefs 开头' "$scratch/foreign.err"; then
    fail "slice 名写成不是 singlefs 开头的 $foreign_slice 时应当退 2、报「singlefs 开头」，实际退 $status：$(cat "$scratch/foreign.err")"
  fi

  for slice in "${selftest_slices[@]}"; do
    systemctl --user stop "$slice" >/dev/null 2>&1
    systemctl --user revert "$slice" >/dev/null 2>&1
  done
  rm -rf -- "${scratch:?}"
  if ((failures)); then
    echo "  ✗ run-with-memory-cap.sh 自检 $failures 处不对（查了 $checked 项；测的是 $target）"
    echo "  → 怎么办：照上面逐条改 run_capped / ensure_slice / scope_result / admission；RUN_WITH_MEMORY_CAP_BREAK 设着、或 RUN_WITH_MEMORY_CAP_SELFTEST_TARGET 指着改前那一份的话这里本来就该红"
    exit 1
  fi
  echo "  ✓ run-with-memory-cap.sh 自检通过（查了 $checked 项：--check 起得来、64M 上限里分配 512 MiB 的探针与它当孙进程时都判撞顶退 $MEMORY_CAP_HIT_EXIT、\
上限之内 kill -9 原样退 137 且 stderr 不多一行、退出码 0 / 7 / 101 原样传回、scope 里 memory.max 与 memory.swap.max 设上了、scope 挂在 slice 底下且 slice 的总上限设上了、\
slice 设不上与余量比整机还大时退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、D-Bus 连不上时跑命令与 --check 都退 $MEMORY_CAP_UNAVAILABLE_EXIT 且命令没跑、systemd-run 不认上限时 --check 退 $MEMORY_CAP_UNAVAILABLE_EXIT、\
两条同时来与前一条还没涨到量时都一条一条跑、峰值表里没有的按上限算排不上退 $MEMORY_ADMISSION_REFUSED_EXIT、要的量比总上限大立刻退 $MEMORY_ADMISSION_REFUSED_EXIT、\
峰值记进表之后两条按记下的量一起跑、scope 里一直收 TERM 时峰值照记（TERM 只在这一例自己的 slice 底下自己开的 scope 里发）、被总上限挤掉退 $SLICE_TOTAL_HIT_EXIT、超过限时退 $TIME_LIMIT_HIT_EXIT、死账不挡路、跑完不留账不留 scope、\
上限写错、没给命令与 slice 名不是 singlefs 开头退 2）"
}

# slice 名只许 singlefs 开头的 .slice（文件头「发信号的地方」一段）
[[ "$SLICE" =~ ^singlefs[-_A-Za-z0-9]*\.slice$ ]] \
  || reject_usage "slice 名 RUN_WITH_MEMORY_CAP_SLICE 要写成 singlefs 开头的 .slice，收到的是「$SLICE」" \
       "不设它（默认 singlefs-heavy.slice），或写成 singlefs_<用途>.slice；别的 slice 里是别人的进程，给它设上限、停它就打到别人"
case "${1:-}" in
  --selftest)
    run_selftest
    exit 0 ;;
  --status)
    PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-$(default_peak_table)}"
    show_status
    exit $? ;;
  --check)
    check_size_syntax "${2:-}" "内存上限"
    check_size_syntax "$RESERVE" "余量 RUN_WITH_MEMORY_CAP_RESERVE "
    [[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
    PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-$(default_peak_table)}"
    run_capped "$2" check true
    exit $? ;;
  "")
    reject_usage "没给上限与命令" "写成 run-with-memory-cap.sh <上限> <命令> [参数…]，上限例 16G" ;;
esac

cap="$1"
shift
check_size_syntax "$cap" "内存上限"
size_in_bytes "$cap" >/dev/null || reject_usage "内存上限 $cap 太大，换成字节超过 64 位" "写成真要的量，例 16G"
check_size_syntax "$RESERVE" "余量 RUN_WITH_MEMORY_CAP_RESERVE "
[[ -z "$SLICE_TOTAL_OVERRIDE" ]] || check_size_syntax "$SLICE_TOTAL_OVERRIDE" "总上限 RUN_WITH_MEMORY_CAP_SLICE_TOTAL "
[[ -z "$TIME_LIMIT_SECONDS" || "$TIME_LIMIT_SECONDS" =~ ^[1-9][0-9]*$ ]] \
  || reject_usage "限时 RUN_WITH_MEMORY_CAP_TIME_LIMIT 要写成正整数秒，收到的是「$TIME_LIMIT_SECONDS」" "写成例 1800；不限时就别设"
[[ "$WAIT_SECONDS" =~ ^[0-9]+$ ]] || reject_usage "等待上限 RUN_WITH_MEMORY_CAP_WAIT_SECONDS 要写成非负整数秒，收到的是「$WAIT_SECONDS」" "写成例 3600"
[[ $# -gt 0 ]] || reject_usage "没给要在上限里跑的命令" "写成 run-with-memory-cap.sh $cap <命令> [参数…]"
PEAK_TABLE="${RUN_WITH_MEMORY_CAP_PEAKS:-$(default_peak_table)}"
run_capped "$cap" run "$@"
exit $?
