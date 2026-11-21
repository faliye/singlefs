#!/usr/bin/env bash
# 攻方腿几份 shell 模型共用的几样：读参数、载入冻结的 merge-and-verify-snapshot-lib.sh 与挑槽那段 python、样本进程的起停与核组里没人。
# 被 source；不改冻结副本里的任何文件，只读。用法：model_arguments "$@"（--frozen <冻结副本> --repo <仓根> --work <新目录>），再 model_load。
model_arguments() {
  MODEL_FROZEN="" MODEL_REPO="" MODEL_WORK=""
  while (($#)); do
    case "$1" in
      --frozen) MODEL_FROZEN="$2"; shift 2 ;;
      --repo) MODEL_REPO="$2"; shift 2 ;;
      --work) MODEL_WORK="$2"; shift 2 ;;
      *) echo "  ✗ 认不出参数：$1" >&2; exit 2 ;;
    esac
  done
  [[ -f "$MODEL_FROZEN/merge-and-verify-snapshot-lib.sh" && -f "$MODEL_FROZEN/merge-and-verify.sh" ]] || { echo "  ✗ --frozen 里没有冻结的两份脚本" >&2; exit 2; }
  [[ -f "$MODEL_REPO/scripts/hosts/run_build_directory.py" && -f "$MODEL_REPO/.claude/hooks/lib_work_root.py" ]] || { echo "  ✗ --repo 不像本仓" >&2; exit 2; }
  [[ -n "$MODEL_WORK" && ! -e "$MODEL_WORK" ]] || { echo "  ✗ --work 要给一个还不在的目录" >&2; exit 2; }
  mkdir -p -- "$MODEL_WORK"
  MODEL_WORK="$(cd "$MODEL_WORK" && pwd -P)"
}

# 载入冻结的快照库（只定义函数）与挑槽那段 python；入口脚本的全局变量照它的取值写在这里（merge-and-verify.sh 的赋值段，名字相同）
model_load() {
  local here
  here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
  SLOT_HOLDER_PROGRAM="$(python3 "$here/extract_program.py" "$MODEL_FROZEN/merge-and-verify.sh" SLOT_HOLDER_PROGRAM)" || exit 1
  # shellcheck source=/dev/null
  source "$MODEL_FROZEN/merge-and-verify-snapshot-lib.sh"
  root="$MODEL_REPO"
  RUN_BUILD_DIRECTORY="$MODEL_REPO/scripts/hosts/run_build_directory.py"
  SLOT_ROOT_NAME="merge-and-verify"; SLOT_BUILD_DIRECTORY_NAME="merge-and-verify-slots"; SLOT_RECORD_NAME="slot"
  SLOT_HOLDER_SECONDS=120; SLOT_TARGET_LIMIT_GIB=150
  unset MERGE_AND_VERIFY_BREAK MERGE_AND_VERIFY_SLOT_TABLE MERGE_AND_VERIFY_SLOT_BUILD_ROOT CARGO_TARGET_DIR MULTI_HOST_RUN_PEER_SCRATCH
  unset TASK_SERVER_TASK TASK_SERVER_ATTEMPT CARGO_INCREMENTAL SINGLEFS_WORK_ROOT_BREAK
}

die() { echo "DIE: $1"; exit 2; }

# 样本进程：开着一份文件（holder_open <文件> <就绪标记>）或拿着 lockf（holder_lock <锁文件> <就绪标记>），各自新会话起，起的那一层紧跟着记 $!
MODEL_HOLDER_PROGRAM='
import fcntl, sys, time
handle = open(sys.argv[2], "a")
if sys.argv[1] == "lock":
    fcntl.lockf(handle, fcntl.LOCK_EX)
open(sys.argv[3], "w").close()
time.sleep(3600)
'
wait_ready() {   # <就绪标记>：至多等 10 秒
  local tries
  for tries in $(seq 100); do [[ -f "$1" ]] && return 0; sleep 0.1; done
  echo "  ✗ 样本进程 10 秒里没就绪（$1）" >&2
  return 1
}
group_left() {   # <进程组号>：打组里还在的进程号（空是没人）
  local file text
  local -a parts
  for file in /proc/[0-9]*/stat; do
    text="$(cat -- "$file" 2> /dev/null)" || continue
    text="${text##*) }"
    read -r -a parts <<< "$text"
    [[ "${parts[2]:-}" == "$1" ]] && printf '%s ' "${file//[^0-9]/}"
  done
  return 0
}
stop_holder() {   # <起它时记下的 $!>：对它的组发 TERM，等它退，再核组里没人
  kill -TERM -- -"$1" 2> /dev/null
  wait "$1" 2> /dev/null
  local left
  left="$(group_left "$1")"
  [[ -z "$left" ]] || echo "name=model_group_left group=$1 processes=$left"
}
