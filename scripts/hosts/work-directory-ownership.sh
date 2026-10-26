#!/usr/bin/env bash
# admission: always 每一次调都判此刻工作根下的目录、归属文件与在跑的进程，上一次的结论不替这一次作保
# run-condition: command python3 du find
# 工作根下每个目录的归属文件：认领（写）、读、清盘。工作根只在 .claude/hooks/lib_work_root.py 里定义
# （SINGLEFS_WORK_ROOT 设了就用它，没设是 ~/.local/share/singlefs/work）。
#
# 归属文件：<工作根>/<目录>/.owner，三行：
#   session: <会话号>
#   created: <建于，YYYY-MM-DD，Asia/Tokyo 的日期>
#   purpose: <用途一句>
# 会话号依次取 WORK_DIRECTORY_OWNER_SESSION、CLAUDE_CODE_SESSION_ID、CLAUDE_SESSION_ID，都没有就拒绝。
# Claude Code 给子 agent 的 CLAUDE_CODE_SESSION_ID 与主会话的相同：主 agent 与它派的子 agent 认领的目录归同一个会话。
#
# 用法：
#   work-directory-ownership.sh claim <目录> --purpose <一句>   # 目录写工作根下一层的名字，或工作根下一层的绝对路径（~/… 也认）；
#                                                               # 没有就建，再写归属文件；已有归属文件、归属是别的会话的拒绝（退 2）
#   work-directory-ownership.sh show <目录>                     # 打出归属文件那三行；没有归属文件或写得不对退 1
#   work-directory-ownership.sh sweep [--dry-run]               # 清盘，见下
#   work-directory-ownership.sh --selftest                      # 临时目录当工作根走一遍认领与清盘
#
# 清盘（sweep）逐项看工作根下一层：
#   归属是本会话、目录里没有正在被进程打开的文件 → 删。删之前先往 <工作根>/sweep-<日期>.log 追加一行
#     「日期 要删 名字 du -sh 的大小 建于 用途」，写不进日志就不删；删完再追加一行「删了」或「没删干净」。
#   「正在被进程打开」按 /proc/<pid>/fd 的链接目标判，另算 /proc/<pid>/cwd 与 /proc/<pid>/exe（当前目录在里面、跑的就是里面的程序）；
#     只看得到本用户的进程（别的用户的 /proc/<pid>/fd 读不到）。不用 lsof、不按名字匹配。删之前对那一项再现查一遍。
#   归属是别的会话的、没有归属文件（或写得不对）的、不是目录的 → 不删，逐项列出名字、大小、建于（没有归属文件的按修改日期，标明），交用户定。
#   归属是本会话、有进程开着里面的文件 → 不删，列出名字、大小与开着它的进程号。
#   清盘日志 sweep-*.log 自己不列、不删。
#   --dry-run：只列，一项不删、不写日志。
# 退出码：0 做完（清盘时没有一项删失败）；1 show 没有归属文件、清盘有删失败的；2 参数不对、取不到会话号或工作根、认领别的会话的目录。
#
# 弄坏开关（只给自证判红用；几个用逗号连，不认识的项名拒绝）：WORK_DIRECTORY_OWNERSHIP_BREAK=
#   delete-unowned     没有归属文件的目录也当成自己的删：自证「sweep:没有归属文件的目录留着、列出名字大小与建于」红
#   skip-log           删之前不写日志：自证「sweep:删前日志逐项带 du -sh 的大小」「sweep:日志里删前那一行排在删了那一行之前」两格红
#   ignore-open-files  不看 /proc，有进程开着文件的也删：自证「sweep:有进程开着文件的目录不删、列出进程号」红
#   dry-run-deletes    --dry-run 照真删：自证「sweep --dry-run:一项不删、不写日志、列出会删的」红
# 开关仅在 --selftest 与它建的临时标记所标识的子进程里生效；普通运行设了开关直接拒绝。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

SCRIPT_PATH="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
REPOSITORY_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OWNER_FILE_NAME=".owner"
KNOWN_BREAK_ITEMS=(delete-unowned skip-log ignore-open-files dry-run-deletes)

refuse() {
  echo "  ✗ $1" >&2
  echo "  → 怎么办：$2" >&2
  exit 2
}

break_is_set() {
  local item
  local -a items=()
  IFS=',' read -r -a items <<<"${WORK_DIRECTORY_OWNERSHIP_BREAK:-}"
  for item in "${items[@]}"; do
    [[ "$item" == "$1" ]] && return 0
  done
  return 1
}

check_break_items() {
  local item known found
  local -a items=()
  IFS=',' read -r -a items <<<"${WORK_DIRECTORY_OWNERSHIP_BREAK:-}"
  for item in "${items[@]}"; do
    [[ -z "$item" ]] && continue
    found=0
    for known in "${KNOWN_BREAK_ITEMS[@]}"; do
      [[ "$item" == "$known" ]] && found=1
    done
    if [[ $found -eq 0 ]]; then
      refuse "WORK_DIRECTORY_OWNERSHIP_BREAK 里的「$item」不认识（只认 ${KNOWN_BREAK_ITEMS[*]}）" \
        "弄坏开关只给自证判红用，平时不设；项名照本脚本文件头「弄坏开关」一段写"
    fi
  done
}

# 本会话的会话号落进 SESSION_ID；取不到就拒绝
resolve_session() {
  SESSION_ID="${WORK_DIRECTORY_OWNER_SESSION:-${CLAUDE_CODE_SESSION_ID:-${CLAUDE_SESSION_ID:-}}}"
  if [[ -z "$SESSION_ID" || "$SESSION_ID" == *[[:space:]]* ]]; then
    refuse "取不到会话号（WORK_DIRECTORY_OWNER_SESSION、CLAUDE_CODE_SESSION_ID、CLAUDE_SESSION_ID 都没设，或带空白）：归属认不出是谁的" \
      "在 Claude Code 会话的 Bash 里跑（它设 CLAUDE_CODE_SESSION_ID）；会话外跑的，命令前写 WORK_DIRECTORY_OWNER_SESSION=<会话号>（不带空白）"
  fi
}

# 工作根落进 WORK_ROOT；取不到就拒绝
resolve_work_root() {
  WORK_ROOT=""
  if ! WORK_ROOT="$(python3 "$REPOSITORY_ROOT/.claude/hooks/lib_work_root.py")" || [[ -z "$WORK_ROOT" ]]; then
    refuse "取不到工作根（.claude/hooks/lib_work_root.py 跑不起来）" "先修 .claude/hooks/lib_work_root.py（工作根只在它里面定义），再跑这条"
  fi
  WORK_ROOT="${WORK_ROOT%/}"
}

today() {
  TZ=Asia/Tokyo date +%F
}

# 读 <目录>/.owner：三个键都在、各一行才算；读到的落进 OWNER_SESSION、OWNER_CREATED、OWNER_PURPOSE
read_owner() {
  local key value line line_count=0
  OWNER_SESSION="" OWNER_CREATED="" OWNER_PURPOSE=""
  [[ -f "$1/$OWNER_FILE_NAME" && ! -L "$1/$OWNER_FILE_NAME" ]] || return 1
  while IFS= read -r line || [[ -n "$line" ]]; do
    line_count=$((line_count + 1))
    key="${line%%:*}"
    value="${line#*: }"
    case "$key" in
      session) OWNER_SESSION="$value" ;;
      created) OWNER_CREATED="$value" ;;
      purpose) OWNER_PURPOSE="$value" ;;
    esac
  done < "$1/$OWNER_FILE_NAME"
  [[ $line_count -eq 3 && -n "$OWNER_SESSION" && -n "$OWNER_CREATED" && -n "$OWNER_PURPOSE" ]]
}

size_of() {
  local measured
  measured="$(du -sh -- "$1" 2>/dev/null)"
  measured="${measured%%$'\t'*}"
  printf '%s' "${measured:-?}"
}

modified_date_of() {
  local stamp
  stamp="$(TZ=Asia/Tokyo stat -c '%y' -- "$1" 2>/dev/null)"
  printf '%s' "${stamp:0:10}"
}

# 打开着 <路径> 或它下面文件的进程号（/proc/<pid>/fd、cwd、exe 的链接目标），逗号连；没有打空。
# /proc 里的链接目标是解开符号链接之后的路径，比之前先把 <路径> 也解开
open_by() {
  local snapshot target
  if break_is_set ignore-open-files; then
    return 0
  fi
  target="$(readlink -f -- "$1")"
  target="${target:-$1}"
  snapshot="$(find /proc/[0-9]*/fd -mindepth 1 -maxdepth 1 -printf '%h\t%l\n' 2>/dev/null
              find /proc/[0-9]*/cwd /proc/[0-9]*/exe -maxdepth 0 -printf '%p\t%l\n' 2>/dev/null)"
  awk -F'\t' -v target="$target" '
    $2 == target || index($2, target "/") == 1 { split($1, parts, "/"); seen[parts[3]] = 1 }
    END { first = 1; for (pid in seen) { printf "%s%s", (first ? "" : ","), pid; first = 0 } }' <<<"$snapshot"
}

claim() {
  local target="" purpose="" have_purpose=0 name path existed=0 created temporary
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --purpose) [[ $# -ge 2 ]] || refuse "--purpose 后面没有用途" "写成 claim <目录> --purpose <一句>"
                 purpose="$2"; have_purpose=1; shift 2 ;;
      --purpose=*) purpose="${1#--purpose=}"; have_purpose=1; shift ;;
      -*) refuse "claim 不认识参数「$1」" "写成 claim <目录> --purpose <一句>" ;;
      *) [[ -z "$target" ]] || refuse "claim 只收一个目录，多了「$1」" "一个目录认领一次：claim <目录> --purpose <一句>"
         target="$1"; shift ;;
    esac
  done
  [[ -n "$target" ]] || refuse "claim 没给目录" "写成 claim <目录> --purpose <一句>，目录写工作根下一层的名字"
  [[ $have_purpose -eq 1 && -n "${purpose//[[:space:]]/}" ]] || refuse "claim 没给用途" "加 --purpose <一句>：这个目录放什么、给哪件活用"
  [[ "$purpose" != *$'\n'* ]] || refuse "用途只许一行" "把 --purpose 写成一句、不换行"
  resolve_session
  resolve_work_root
  case "$target" in "~/"*) target="$HOME/${target#\~/}" ;; esac
  while [[ "$target" == */ && "$target" != "/" ]]; do target="${target%/}"; done
  if [[ "$target" == */* ]]; then
    [[ "${target%/*}" == "$WORK_ROOT" ]] || refuse "「$target」不在工作根 $WORK_ROOT 的下一层" \
      "只认领工作根下一层的目录：写名字（claim <名字> …），或 $WORK_ROOT/<名字>"
    name="${target##*/}"
  else
    name="$target"
  fi
  case "$name" in
    ""|.|..) refuse "目录名「$name」不能用" "换一个名字，例：<活的简称>-<日期>" ;;
    sweep-*.log) refuse "「$name」与清盘日志同名" "换一个不以 sweep- 开头、不以 .log 结尾的名字" ;;
  esac
  path="$WORK_ROOT/$name"
  if [[ -e "$path" && ! -d "$path" ]] || [[ -L "$path" ]]; then
    refuse "$path 已存在、不是目录（或是符号链接）" "换一个名字；那一项是谁的，交用户看"
  fi
  [[ -d "$path" ]] && existed=1
  created="$(today)"
  if read_owner "$path"; then
    if [[ "$OWNER_SESSION" != "$SESSION_ID" ]]; then
      refuse "$path 归属是别的会话（$OWNER_SESSION，建于 $OWNER_CREATED，用途：$OWNER_PURPOSE），不认领" \
        "换一个目录名；那个会话的目录由它自己清，或交用户定"
    fi
    created="$OWNER_CREATED"
  elif [[ -e "$path/$OWNER_FILE_NAME" ]]; then
    refuse "$path/$OWNER_FILE_NAME 在，但不是 session、created、purpose 三行" "交用户看那份归属文件是谁写的；不覆盖它"
  fi
  if ! mkdir -p -- "$path"; then
    refuse "建不了目录 $path" "看工作根 $WORK_ROOT 的权限与磁盘空间"
  fi
  temporary="$path/$OWNER_FILE_NAME.tmp.$$"
  if ! printf 'session: %s\ncreated: %s\npurpose: %s\n' "$SESSION_ID" "$created" "$purpose" > "$temporary" \
      || ! mv -f -- "$temporary" "$path/$OWNER_FILE_NAME"; then
    rm -f -- "$temporary"
    refuse "写不进归属文件 $path/$OWNER_FILE_NAME" "看目录权限与磁盘空间"
  fi
  if ! read_owner "$path" || [[ "$OWNER_SESSION" != "$SESSION_ID" || "$OWNER_PURPOSE" != "$purpose" ]]; then
    refuse "回读 $path/$OWNER_FILE_NAME 对不上刚写的三行" "看是不是别的进程同时在写这个目录的归属文件"
  fi
  if [[ $existed -eq 1 ]]; then
    echo "  ✓ 认领了已有的目录 $path（建于 $created，用途：$purpose）；原来就在的目录，先确认是本会话建的"
  else
    echo "  ✓ 建了目录并认领 $path（建于 $created，用途：$purpose）"
  fi
}

show() {
  [[ $# -eq 1 ]] || refuse "show 只收一个目录" "写成 show <目录>"
  resolve_work_root
  local target="$1"
  case "$target" in "~/"*) target="$HOME/${target#\~/}" ;; esac
  [[ "$target" == */* ]] || target="$WORK_ROOT/$target"
  if ! read_owner "$target"; then
    echo "  ✗ $target 没有归属文件，或归属文件不是 session、created、purpose 三行"
    echo "  → 怎么办：是本会话建的就 claim 它；不是的交用户定"
    return 1
  fi
  printf 'session: %s\ncreated: %s\npurpose: %s\n' "$OWNER_SESSION" "$OWNER_CREATED" "$OWNER_PURPOSE"
}

sweep() {
  local dry_run=0 listing name path size holders logged log_file stamp entry total=0
  local -a mine=() others=() unowned=() busy=() deleted=() would=() failed=()
  while [[ $# -gt 0 ]]; do
    case "$1" in
      --dry-run) dry_run=1; shift ;;
      *) refuse "sweep 不认识参数「$1」" "写成 sweep 或 sweep --dry-run" ;;
    esac
  done
  break_is_set dry-run-deletes && dry_run=0
  resolve_session
  resolve_work_root
  [[ -d "$WORK_ROOT" ]] || refuse "工作根 $WORK_ROOT 不存在" "没有要清的；工作根换过的看 SINGLEFS_WORK_ROOT"
  stamp="$(today)"
  log_file="$WORK_ROOT/sweep-$stamp.log"
  if ! listing="$(find "$WORK_ROOT" -mindepth 1 -maxdepth 1 -printf '%f\n' | LC_ALL=C sort)"; then
    refuse "列不全工作根 $WORK_ROOT 下的项" "看工作根的权限；列不全就不清"
  fi
  while IFS= read -r name; do
    [[ -n "$name" ]] || continue
    case "$name" in sweep-*.log) continue ;; esac
    total=$((total + 1))
    path="$WORK_ROOT/$name"
    if [[ -L "$path" || ! -d "$path" ]]; then
      unowned+=("$name	$(size_of "$path")	修改于 $(modified_date_of "$path")（不是目录，没有归属文件）")
    elif read_owner "$path"; then
      if [[ "$OWNER_SESSION" == "$SESSION_ID" ]]; then
        mine+=("$name")
      else
        others+=("$name	$(size_of "$path")	建于 $OWNER_CREATED	会话 $OWNER_SESSION	用途：$OWNER_PURPOSE")
      fi
    elif break_is_set delete-unowned; then
      mine+=("$name")
    else
      unowned+=("$name	$(size_of "$path")	修改于 $(modified_date_of "$path")（没有归属文件，建于不明）")
    fi
  done <<<"$listing"
  for name in ${mine[@]+"${mine[@]}"}; do
    path="$WORK_ROOT/$name"
    size="$(size_of "$path")"
    holders="$(open_by "$path")"
    if [[ -n "$holders" ]]; then
      busy+=("$name	$size	开着它的进程 $holders")
      continue
    fi
    if read_owner "$path"; then
      entry="$name	$size	建于 $OWNER_CREATED	用途：$OWNER_PURPOSE"
    else
      entry="$name	$size	（没有归属文件）"
    fi
    if [[ $dry_run -eq 1 ]]; then
      would+=("$entry")
      continue
    fi
    logged=0
    if break_is_set skip-log; then
      logged=1
    elif printf '%s\t要删\t%s\n' "$stamp" "$entry" >> "$log_file"; then
      logged=1
    fi
    if [[ $logged -ne 1 ]]; then
      failed+=("$name	写不进日志 $log_file，没删")
      continue
    fi
    rm -rf -- "${WORK_ROOT:?}/${name:?}"
    if [[ -e "$path" || -L "$path" ]]; then
      printf '%s\t没删干净\t%s\n' "$stamp" "$name" >> "$log_file"
      failed+=("$name	rm -rf 之后还在")
    else
      printf '%s\t删了\t%s\n' "$stamp" "$name" >> "$log_file"
      deleted+=("$entry")
    fi
  done
  print_section() {
    local title="$1" row
    shift
    echo "  $title：$# 项"
    for row in "$@"; do
      printf '    %s\n' "${row//	/  }"
    done
  }
  print_section "归属是别的会话的（不删）" ${others[@]+"${others[@]}"}
  print_section "没有归属文件的（不删，交用户定）" ${unowned[@]+"${unowned[@]}"}
  print_section "本会话的、有进程开着里面的文件（不删）" ${busy[@]+"${busy[@]}"}
  if [[ $dry_run -eq 1 ]]; then
    print_section "本会话的、会删的（--dry-run，没删）" ${would[@]+"${would[@]}"}
  else
    print_section "本会话的、删了的（日志 $log_file）" ${deleted[@]+"${deleted[@]}"}
  fi
  if [[ ${#failed[@]} -gt 0 ]]; then
    print_section "删失败的" "${failed[@]}"
    echo "  ✗ 清盘有 ${#failed[@]} 项没删成"
    echo "  → 怎么办：逐项看上面「删失败的」那一节：写不进日志的先修工作根的权限与空间，删不干净的看里面有没有别的用户的文件"
    return 1
  fi
  if [[ $dry_run -eq 1 ]]; then
    echo "  ✓ 清盘预演（--dry-run，一项没删、没写日志）：查了 $total 项，会删 ${#would[@]} 项；别的会话的 ${#others[@]} 项、没有归属文件的 ${#unowned[@]} 项、本会话在用的 ${#busy[@]} 项不删"
  else
    echo "  ✓ 清盘完：查了 $total 项，删了 ${#deleted[@]} 项（逐项记在 $log_file）；别的会话的 ${#others[@]} 项、没有归属文件的 ${#unowned[@]} 项、本会话在用的 ${#busy[@]} 项没删"
  fi
}

# 自证的临时目录与占着文件的进程号放全局：EXIT 时 selftest 的局部变量已经没了，收尾要读得到
SELFTEST_SCRATCH=""
SELFTEST_HOLDER=""
cleanup_selftest() {
  if [[ -n "$SELFTEST_HOLDER" ]]; then
    kill "$SELFTEST_HOLDER" 2>/dev/null
    wait "$SELFTEST_HOLDER" 2>/dev/null
    SELFTEST_HOLDER=""
  fi
  if [[ -n "$SELFTEST_SCRATCH" ]]; then
    rm -rf -- "${SELFTEST_SCRATCH:?}"
    SELFTEST_SCRATCH=""
  fi
}

selftest() {
  local root holder held_link attempt dry_output dry_rc sweep_output sweep_rc output rc log_line entry
  local -a cells=() failures=()
  trap cleanup_selftest EXIT
  if ! SELFTEST_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/work-directory-ownership-selftest.XXXXXX")"; then
    echo "  ✗ 自证建不了临时目录"
    echo "  → 怎么办：看 \${TMPDIR:-/tmp} 的权限与空间"
    return 1
  fi
  root="$SELFTEST_SCRATCH/work"
  mkdir -p "$root"
  export WORK_DIRECTORY_OWNERSHIP_SELFTEST_MARK="$SELFTEST_SCRATCH/selftest-mark"
  printf '%s\n' work-directory-ownership-selftest > "$WORK_DIRECTORY_OWNERSHIP_SELFTEST_MARK"
  as_session() {
    local session="$1"
    shift
    SINGLEFS_WORK_ROOT="$root" WORK_DIRECTORY_OWNER_SESSION="$session" bash "$SCRIPT_PATH" "$@"
  }
  record_cell() {
    cells+=("$1")
    [[ "$2" == "$3" ]] || failures+=("$1	应当是 $2，实际 $3")
  }

  output="$(as_session selftest-own claim mine-idle --purpose '自证：本会话的、没人在用' 2>&1)"; rc=$?
  record_cell "claim:写出三行（session、created、purpose）" "0 3 selftest-own" \
    "$rc $(wc -l < "$root/mine-idle/.owner" 2>/dev/null) $(sed -n 's/^session: //p' "$root/mine-idle/.owner" 2>/dev/null)"
  record_cell "claim:建于是今天、用途原样" "$(today) 自证：本会话的、没人在用" \
    "$(sed -n 's/^created: //p' "$root/mine-idle/.owner" 2>/dev/null) $(sed -n 's/^purpose: //p' "$root/mine-idle/.owner" 2>/dev/null)"
  as_session selftest-own claim mine-busy --purpose '自证：本会话的、有进程开着文件' >/dev/null 2>&1
  as_session selftest-other claim theirs --purpose '自证：别的会话的' >/dev/null 2>&1
  output="$(env -u WORK_DIRECTORY_OWNER_SESSION -u CLAUDE_CODE_SESSION_ID -u CLAUDE_SESSION_ID SINGLEFS_WORK_ROOT="$root" \
    bash "$SCRIPT_PATH" claim nobody --purpose '自证' 2>&1)"; rc=$?
  record_cell "claim:取不到会话号就拒、写明怎么给" "2 1 0" \
    "$rc $([[ "$output" == *"WORK_DIRECTORY_OWNER_SESSION=<会话号>"* ]] && echo 1 || echo 0) $([[ -e "$root/nobody" ]] && echo 1 || echo 0)"
  output="$(as_session selftest-own claim theirs --purpose '自证：抢别人的' 2>&1)"; rc=$?
  record_cell "claim:归属是别的会话的不认领" "2 selftest-other" "$rc $(sed -n 's/^session: //p' "$root/theirs/.owner" 2>/dev/null)"
  mkdir -p "$root/unowned"
  printf 'x\n' > "$root/unowned/f"
  printf 'data\n' > "$root/mine-idle/f"
  printf 'data\n' > "$root/mine-busy/held"
  printf 'data\n' > "$root/theirs/f"
  ( exec 3<"$root/mine-busy/held"; exec sleep 120 ) &
  holder=$!
  SELFTEST_HOLDER="$holder"
  held_link=""
  for attempt in $(seq 1 100); do
    held_link="$(readlink "/proc/$holder/fd/3" 2>/dev/null)"
    [[ "$held_link" == "$root/mine-busy/held" ]] && break
    sleep 0.05
  done
  record_cell "自证装置:占着文件的进程起来了" "$root/mine-busy/held" "$held_link"

  dry_output="$(as_session selftest-own sweep --dry-run 2>&1)"; dry_rc=$?
  record_cell "sweep --dry-run:一项不删、不写日志、列出会删的" "0 1111 0 1" \
    "$dry_rc $([[ -d "$root/mine-idle" ]] && echo 1 || echo 0)$([[ -d "$root/mine-busy" ]] && echo 1 || echo 0)$([[ -d "$root/theirs" ]] && echo 1 || echo 0)$([[ -d "$root/unowned" ]] && echo 1 || echo 0) $(find "$root" -maxdepth 1 -name 'sweep-*.log' | wc -l) $([[ "$dry_output" == *"会删的"*"mine-idle"* ]] && echo 1 || echo 0)"

  sweep_output="$(as_session selftest-own sweep 2>&1)"; sweep_rc=$?
  record_cell "sweep:删了本会话的、没人在用的" "0 0" "$sweep_rc $([[ -e "$root/mine-idle" ]] && echo 1 || echo 0)"
  record_cell "sweep:归属是别的会话的留着、列出名字大小与建于" "1 1" \
    "$([[ -f "$root/theirs/f" ]] && echo 1 || echo 0) $(grep -cE "^    theirs  [0-9.,]+[KMGTP]?  建于 $(today)  会话 selftest-other" <<<"$sweep_output")"
  record_cell "sweep:没有归属文件的目录留着、列出名字大小与建于" "1 1" \
    "$([[ -f "$root/unowned/f" ]] && echo 1 || echo 0) $(grep -cE "^    unowned  [0-9.,]+[KMGTP]?  修改于 $(today)（没有归属文件，建于不明）" <<<"$sweep_output")"
  record_cell "sweep:有进程开着文件的目录不删、列出进程号" "1 1" \
    "$([[ -f "$root/mine-busy/held" ]] && echo 1 || echo 0) $(grep -cE "^    mine-busy  .*开着它的进程 ([0-9]+,)*$holder(,|$)" <<<"$sweep_output")"
  log_line="$(grep -P "^$(today)\t要删\tmine-idle\t[0-9.,]+[KMGTP]?\t建于 " "$root/sweep-$(today).log" 2>/dev/null)"
  record_cell "sweep:删前日志逐项带 du -sh 的大小" "1" "$([[ -n "$log_line" ]] && echo 1 || echo 0)"
  record_cell "sweep:日志里删前那一行排在删了那一行之前" "要删 删了" \
    "$(awk -F'\t' '$3 == "mine-idle" || ($2 == "删了" && $3 == "mine-idle") {printf "%s%s", (n++ ? " " : ""), $2}' "$root/sweep-$(today).log" 2>/dev/null)"

  kill "$holder" 2>/dev/null
  wait "$holder" 2>/dev/null
  SELFTEST_HOLDER=""
  output="$(as_session selftest-own sweep 2>&1)"; rc=$?
  record_cell "sweep:占着文件的进程退了之后再清就删" "0 0" "$rc $([[ -e "$root/mine-busy" ]] && echo 1 || echo 0)"
  output="$(WORK_DIRECTORY_OWNERSHIP_BREAK=no-such-item as_session selftest-own sweep --dry-run 2>&1)"; rc=$?
  record_cell "弄坏开关:不认识的项名拒绝" "2" "$rc"

  if [[ ${#failures[@]} -gt 0 ]]; then
    for entry in "${failures[@]}"; do
      echo "  ✗ 自证格不对：${entry%%	*}（${entry#*	}）"   # gate-lint:detail
    done
    echo "  ✗ 自证 ${#cells[@]} 格里 ${#failures[@]} 格不对"
    echo "  → 怎么办：WORK_DIRECTORY_OWNERSHIP_BREAK 设着的话（项与红的格见文件头「弄坏开关」）这里本来就该红；没设的看 claim、sweep、open_by 与 read_owner"
    return 1
  fi
  echo "  ✓ 工作目录归属自证通过（查了 ${#cells[@]} 格）：claim 写三行、取不到会话号与别的会话的目录拒绝；sweep 只删本会话没人在用的，别的会话的、没有归属文件的、有进程开着文件的留着并列出，删前日志逐项带大小，--dry-run 不删"
}

if [[ -n "${WORK_DIRECTORY_OWNERSHIP_BREAK:-}" && "${1:-}" != --selftest ]]; then
  if [[ -z "${WORK_DIRECTORY_OWNERSHIP_SELFTEST_MARK:-}" ]] \
      || [[ "$(cat -- "$WORK_DIRECTORY_OWNERSHIP_SELFTEST_MARK" 2>/dev/null)" != work-directory-ownership-selftest ]]; then
    refuse "WORK_DIRECTORY_OWNERSHIP_BREAK 只在 --selftest 里认，这一次什么都没做" \
      "unset WORK_DIRECTORY_OWNERSHIP_BREAK 后再跑"
  fi
fi
check_break_items
case "${1:-}" in
  claim) shift; claim "$@" ;;
  show) shift; show "$@" ;;
  sweep) shift; sweep "$@" ;;
  --selftest) selftest ;;
  -h|--help) sed -n '4,36p' "$SCRIPT_PATH" ;;
  *) refuse "不认识的子命令「${1:-}」" "写成 claim <目录> --purpose <一句>、show <目录>、sweep [--dry-run] 或 --selftest" ;;
esac
