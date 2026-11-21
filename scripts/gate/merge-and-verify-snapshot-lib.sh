#!/usr/bin/env bash
# 合入后验证 merge-and-verify.sh 的函数库：快照与槽位（take_snapshot、finish、各槽的挑选与回收）、进度行的后台进程。
# 被 source，只定义函数、不改 shell 选项；读写入口脚本的全局变量（同一个 shell）。

# progress_ticker <主进程号> <全量日志> <日志名>：后台进程的函数体，每个间隔往标准输出打一行进度；收到 TERM 先停自己起的那一个 sleep 再退（sleep 的输出丢掉，不占着标准输出）。
#   主进程没了自己退。TERM 落在 sleep 刚起、进程号还没记进 sleeper 的那一刻：处理函数只记 stopping，起完 sleep 的下一步按 stopping 补停它（只靠处理函数停，这一刻的 sleep 会成孤儿留到间隔满）。
#   停 sleep 用 KILL（只发给 sleeper 记下的那一个进程号）：sleep 刚 fork、还没 exec 的那一刻收到 TERM 只跑继承来的处理函数、之后照常 exec 睡满间隔，ticker 的 wait 与主进程等 ticker 退出的 wait 都要等到间隔满
progress_ticker() {
  local parent="$1" log="$2" label="$3" sleeper="" stopping=0 started=$SECONDS
  # 只给自证演（MERGE_AND_VERIFY_SELFTEST_FORKED_FINISH，settle_break_switches 拒平常设了的）：fork 窗口里收到 TERM、照主 shell 的 trap 跑了 EXIT trap 的那一种，这里直接调 finish。
  #   先不理 TERM：步骤的命令退得快，stop_progress_ticker 的 TERM 会在 finish 跑到一半时到，不理它，finish 做了什么就是什么（主 shell 的 wait 等它跑完）
  [[ -z "${MERGE_AND_VERIFY_SELFTEST_FORKED_FINISH:-}" ]] || { trap '' TERM; finish; }
  trap 'stopping=1; [[ -z "$sleeper" ]] || kill -KILL "$sleeper" 2> /dev/null' TERM
  while ((stopping == 0)) && kill -0 "$parent" 2> /dev/null; do
    sleep "$PROGRESS_INTERVAL" > /dev/null 2>&1 & sleeper=$!
    ((stopping == 0)) || { kill -KILL "$sleeper" 2> /dev/null; wait "$sleeper" 2> /dev/null; break; }
    wait "$sleeper" 2> /dev/null
    sleeper=""
    ((stopping == 0)) || break
    kill -0 "$parent" 2> /dev/null || break
    printf '  · 进度：%s 已跑 %s 秒，日志 %s 行\n' "$label" "$((SECONDS - started))" "$(wc -l < "$log" 2> /dev/null || echo 0)"
  done
}

# stop_progress_ticker：停 run_step 起的进度后台进程（只按它自己的 pid）；它被停的退出码不进任何一步的退出码
stop_progress_ticker() {
  [[ -n "$PROGRESS_TICKER_PID" ]] || return 0
  kill "$PROGRESS_TICKER_PID" 2> /dev/null
  wait "$PROGRESS_TICKER_PID" 2> /dev/null
  PROGRESS_TICKER_PID=""
  return 0
}

# take_snapshot <要验的树> <快照>：文件头「快照」。快照目录这里新建（已有就退 1）；不拷任意一层的 .git、target、target-* 与 __pycache__；
# 不保留属性（不带 --perms、--owner、--group：权限位照建文件时的缺省，可执行位随源文件带过去）与修改时刻（不带 --times）。
# rsync 退 24（拷的时候有文件被别的会话删了）算拷成：那一刻已经没有它，快照里也就没有；别的非 0 退 1。弄坏开关 snapshot-keeps-times：带源树的修改时刻拷
take_snapshot() {
  local rc=0
  local -a times=(--no-times)
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "snapshot-keeps-times" ]] || times=(--times)
  mkdir -- "$2" || return 1
  rsync --recursive --links "${times[@]}" --exclude .git --exclude 'target/' --exclude 'target-*/' --exclude '__pycache__/' -- "$1/" "$2/" || rc=$?
  ((rc == 24)) && echo "  ! rsync 退 24：拷的时候有文件不见了（别的会话删了它），快照里没有它们"
  ((rc == 0 || rc == 24))
}

# 删这一趟自己建的编译目录（弄坏开关 keep-run-target：不删；run-target-open-files-unchecked：删之前不核开着的文件）
remove_run_target() {
  [[ -n "$RUN_TARGET" && "${MERGE_AND_VERIFY_BREAK:-}" != "keep-run-target" ]] || return 0
  python3 "$RUN_BUILD_DIRECTORY" remove "$RUN_TARGET" $([[ "${MERGE_AND_VERIFY_BREAK:-}" == "run-target-open-files-unchecked" ]] && echo --skip-open-check)
}

finish() {
  local rc=$? kept_size keep_both=0
  # 只在起这一趟的那个 shell 里收尾（文件头「快照」末尾那一段）：fork 出来的子 shell（run_step 放后台的进度进程）跑到这里照原退出码退、什么都不删。
  #   弄坏开关 finish-in-forked-shell：子 shell 里也照收尾
  if [[ "$BASHPID" != "$$" && "${MERGE_AND_VERIFY_BREAK:-}" != "finish-in-forked-shell" ]]; then trap - EXIT; exit "$rc"; fi
  trap - EXIT
  stop_progress_ticker
  report_slot_state
  # 槽位那一种（文件头「槽位」的收尾）：快照与编译目录都不删；量编译目录，带 MERGE_AND_VERIFY_KEEP_COPY=1 的把槽记成留着，最后放锁
  if [[ -n "$SLOT_DIRECTORY" ]]; then
    cd / || true
    trim_slot_target 收尾
    if [[ "${MERGE_AND_VERIFY_KEEP_COPY:-}" == 1 ]]; then
      if printf 'log: %s\ncreated: %s\n' "$logs" "$(TZ=Asia/Tokyo date +%F)" > "$SLOT_DIRECTORY/kept"; then
        # 归属记录写在槽里（完工清理闸照 <工作根>/merge-and-verify/<一层>/.owner 认）；弄坏开关 keep-owner-unrecorded：不写
        [[ "${MERGE_AND_VERIFY_BREAK:-}" == "keep-owner-unrecorded" ]] \
          || run_directory_owner_record "$SLOT_DIRECTORY" "合入后验证留下的快照 $SNAPSHOT_TREE（槽留着，MERGE_AND_VERIFY_KEEP_COPY=1；用完放槽）"
        echo "  ! 留着快照 $SNAPSHOT_TREE（槽 $SLOT_DIRECTORY 记成留着，MERGE_AND_VERIFY_KEEP_COPY=1：别的趟不占它；这一槽的编译目录 $SLOT_TARGET 跟着留）"
        echo "  → 用完放槽：$(slot_release_command "$SLOT_DIRECTORY")（快照与编译目录留给下一趟接着编；要腾盘另 rm -rf -- '$SLOT_DIRECTORY'，这一台的编译目录在下一趟占槽时跟着删）"
      else
        echo "  ✗ 没把槽记成留着（写 $SLOT_DIRECTORY/kept 没成）：快照 $SNAPSHOT_TREE 会被下一趟占这一槽时盖掉"
        echo "  → 怎么办：看工作根可不可写、盘满没满；验证段第 2 步要用这一趟的快照的，修好之后整条重交"
        rc=1
      fi
    else
      echo "  放了槽 $SLOT_DIRECTORY（快照 $SNAPSHOT_TREE 与编译目录 $SLOT_TARGET 留给下一趟接着编）"
    fi
    release_slot
    exit "$rc"
  fi
  # 快照与编译目录都留着的三种（弄坏开关 keep-copy、cleanup-only-on-success 红了的那一趟，MERGE_AND_VERIFY_KEEP_COPY=1）先判，只看变量、不起命令
  if [[ -n "$SNAPSHOT_TREE" ]] && { [[ "${MERGE_AND_VERIFY_BREAK:-}" == "keep-copy" ]] || { [[ "${MERGE_AND_VERIFY_BREAK:-}" == "cleanup-only-on-success" ]] && ((rc != 0)); }; }; then
    keep_both=1
  elif [[ "${MERGE_AND_VERIFY_KEEP_COPY:-}" == 1 ]]; then
    keep_both=2
  fi
  # 第一步删编译目录（前面一步写满盘时编译目录就删不成）；弄坏开关 cleanup-after-other-steps 挪到最后
  if ((keep_both == 0)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "cleanup-after-other-steps" ]]; then
    remove_run_target || rc=1
  fi
  cd / || true
  if [[ -n "${MERGE_AND_VERIFY_SELFTEST_FULL_DISK:-}" ]]; then   # 只给自证演：删编译目录之后、删快照那一步写满盘，整条收尾停在那里
    echo "  ✗ 收尾那一步写不进去：OSError: [Errno 28] No space left on device（自证演的，MERGE_AND_VERIFY_SELFTEST_FULL_DISK）"
    echo "  → 怎么办：这是自证造的局面；编译目录在这一步之前删过了才对"
    exit 1
  fi
  [[ -n "$SNAPSHOT_TREE" || -n "$RUN_TARGET" ]] || exit "$rc"
  if ((keep_both == 1)); then
    exit "$rc"
  fi
  if ((keep_both == 2)); then
    echo "  ! 留着${SNAPSHOT_TREE:+快照 $SNAPSHOT_TREE}${SNAPSHOT_TREE:+${RUN_TARGET:+ 与}}${RUN_TARGET:+编译目录 $RUN_TARGET}（MERGE_AND_VERIFY_KEEP_COPY=1）"
    echo "  → 看完自己删：rm -rf --${SNAPSHOT_TREE:+ '$SNAPSHOT_TREE'}${RUN_TARGET:+ '$RUN_TARGET'}"
    exit "$rc"
  fi
  if [[ -n "$SNAPSHOT_TREE" ]]; then
    kept_size="$(du -sh -- "$SNAPSHOT_TREE" 2>/dev/null | cut -f1)"
    rm -rf -- "${SNAPSHOT_TREE:?}"
    if [[ -e "$SNAPSHOT_TREE" ]]; then
      echo "  ✗ 快照 $SNAPSHOT_TREE 没删成"
      echo "  → 怎么办：看谁还开着里面的文件（python3 $RUN_BUILD_DIRECTORY holders '$SNAPSHOT_TREE'），停了再 rm -rf -- 那一份"
      rc=1
    else
      echo "  删了快照 $SNAPSHOT_TREE（共 ${kept_size:-?}；日志留在 $(dirname "$SNAPSHOT_TREE")）"
    fi
  fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "cleanup-after-other-steps" ]]; then
    remove_run_target || rc=1
  fi
  exit "$rc"
}

# record_task_attempt <日志目录>：任务号有、尝试号是正整数时记一行「<任务号> <尝试号>」；手跑没有任务号，不记。弄坏开关 earlier-attempt-not-recorded：不记
record_task_attempt() {
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "earlier-attempt-not-recorded" ]] || return 0
  [[ -n "${TASK_SERVER_TASK:-}" && "${TASK_SERVER_ATTEMPT:-}" =~ ^[1-9][0-9]*$ ]] || return 0
  printf '%s %s\n' "$TASK_SERVER_TASK" "$TASK_SERVER_ATTEMPT" > "$1/$ATTEMPT_RECORD_NAME" \
    || echo "  ! 没记下这一趟的任务号与尝试号（写 $1/$ATTEMPT_RECORD_NAME 失败）：下一次尝试找不到这一趟留下的快照"
}

# remove_earlier_attempt_snapshots <这一趟的日志目录>：任务号有、尝试号大于 1 时，删同一层里 task-attempt 记着同一任务号、尝试号更小的日志目录的 repo/ 快照；
#   本机有进程开着里面的文件或核不成就不删、打一行写明；删了打一行；其余的日志目录不碰。删不成与没删都不改退出码。
#   弄坏开关：earlier-attempt-snapshot-kept 不删；earlier-attempt-task-ignored 不比任务号；earlier-attempt-not-smaller-removed 尝试号相同的也删；
#   earlier-attempt-open-files-unchecked 删之前不核开着的文件
remove_earlier_attempt_snapshots() {
  local current="$1" record directory task attempt snapshot holders holders_rc size
  [[ -n "${TASK_SERVER_TASK:-}" && "${TASK_SERVER_ATTEMPT:-}" =~ ^[1-9][0-9]*$ ]] || return 0
  ((TASK_SERVER_ATTEMPT > 1)) || return 0
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "earlier-attempt-snapshot-kept" ]] || return 0
  for record in "$(dirname -- "$current")"/*/"$ATTEMPT_RECORD_NAME"; do
    [[ -f "$record" ]] || continue
    directory="$(dirname -- "$record")"
    [[ "$directory" != "$current" ]] || continue
    task=""; attempt=""
    read -r task attempt < "$record" || true
    [[ "$attempt" =~ ^[1-9][0-9]*$ ]] || continue
    [[ "$task" == "$TASK_SERVER_TASK" || "${MERGE_AND_VERIFY_BREAK:-}" == "earlier-attempt-task-ignored" ]] || continue
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "earlier-attempt-not-smaller-removed" ]]; then ((attempt <= TASK_SERVER_ATTEMPT)) || continue
    else ((attempt < TASK_SERVER_ATTEMPT)) || continue; fi
    snapshot="$directory/repo"
    [[ -d "$snapshot" && ! -L "$snapshot" ]] || continue
    holders=""; holders_rc=0
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "earlier-attempt-open-files-unchecked" ]]; then
      holders="$(python3 "$RUN_BUILD_DIRECTORY" holders "$snapshot" 2>&1)" || holders_rc=$?
    fi
    if ((holders_rc == 3)); then
      echo "  ! 没删上一次尝试留下的快照 $snapshot（任务 ${task} 的尝试 ${attempt}）：本机有进程开着里面的文件：${holders//$'\n'/；}；停了那个进程再手删 rm -rf -- '$snapshot'"
      continue
    elif ((holders_rc != 0)); then
      echo "  ! 没删上一次尝试留下的快照 $snapshot（任务 ${task} 的尝试 ${attempt}）：核不成有没有进程开着里面的文件（run_build_directory.py holders 退 ${holders_rc}：${holders//$'\n'/；}）；核清楚再手删 rm -rf -- '$snapshot'"
      continue
    fi
    size="$(du -sh -- "$snapshot" 2>/dev/null | cut -f1)"
    rm -rf -- "${snapshot:?}"
    if [[ -e "$snapshot" ]]; then
      echo "  ! 上一次尝试留下的快照 $snapshot（任务 ${task} 的尝试 ${attempt}）没删成：看谁还开着里面的文件、权限对不对，再手删 rm -rf -- '$snapshot'"
    else
      echo "  删了上一次尝试留下的快照 $snapshot（共 ${size:-?}；任务 ${task} 的尝试 ${attempt}，日志留在 ${directory}）"
    fi
  done
  return 0
}

# ── 槽位（文件头「槽位」）──
# read_slot_table：读槽位登记填 SLOT_LIMIT、SLOT_TARGET_LIMIT_GIB；读不出、缺一项、值不是非负整数、有认不出的行都退 1、一个补丁都不打。
#   弄坏开关 slot-table-unchecked：值不是非负整数的那一项当 0 收
read_slot_table() {
  local line key value
  local -A configured=()
  [[ -r "$SLOT_TABLE" ]] || die "读不出槽位登记 $SLOT_TABLE，一个补丁都没打、没编" "照文件头「槽位」恢复那份登记（slots=<非负整数>、slot-target-limit-gib=<非负整数> 两行）再跑" 1
  while IFS= read -r line || [[ -n "$line" ]]; do
    [[ -z "${line//[[:space:]]/}" || "$line" == \#* ]] && continue
    key="${line%%=*}"; value="${line#*=}"
    if [[ "$line" != *=* || ( "$key" != slots && "$key" != slot-target-limit-gib ) ]]; then
      die "槽位登记 $SLOT_TABLE 里有认不出的一行「$line」，一个补丁都没打、没编" "只写 slots=<非负整数> 与 slot-target-limit-gib=<非负整数> 两行，别的只许 # 起头的注释与空行（文件头「槽位」）" 1
    fi
    if [[ ! "$value" =~ ^[0-9]+$ ]]; then
      [[ "${MERGE_AND_VERIFY_BREAK:-}" == "slot-table-unchecked" ]] \
        || die "槽位登记 $SLOT_TABLE 里 $key 的值「$value」不是非负整数，一个补丁都没打、没编" "写成 $key=<非负整数>（文件头「槽位」）" 1
      value=0
    fi
    configured[$key]="$((10#$value))"
  done < "$SLOT_TABLE"
  [[ -n "${configured[slots]:-}" && -n "${configured[slot-target-limit-gib]:-}" ]] \
    || die "槽位登记 $SLOT_TABLE 缺 slots 或 slot-target-limit-gib，一个补丁都没打、没编" "两项都写（文件头「槽位」）" 1
  SLOT_LIMIT="${configured[slots]}"; SLOT_TARGET_LIMIT_GIB="${configured[slot-target-limit-gib]}"
}

# slot_build_root：打这一台的编译根；MERGE_AND_VERIFY_SLOT_BUILD_ROOT 设了打它（只给自证）。取不到退 1，标准输出是原因
slot_build_root() {
  if [[ -n "${MERGE_AND_VERIFY_SLOT_BUILD_ROOT:-}" ]]; then printf '%s\n' "$MERGE_AND_VERIFY_SLOT_BUILD_ROOT"; return 0; fi
  python3 - "$root" build-root <<<"$SLOT_CONFIGURED_ROOTS_PROGRAM" 2>&1
}

# slot_default_work_root：打这一台缺省的工作根（文件头「槽位」）：多机配置里 SHARED_DATA_ROOT 下的 work，与 slot_build_root 读同一份配置，不看 SINGLEFS_DATA_ROOT。
#   取不到退 1，标准输出是原因。弄坏开关 slot-default-work-root-follows-data-root：照 lib_work_root.py shared work 取（设了 SINGLEFS_DATA_ROOT 就跟着它）
slot_default_work_root() {
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "slot-default-work-root-follows-data-root" ]]; then
    python3 "$root/.claude/hooks/lib_work_root.py" shared work 2>&1
    return
  fi
  python3 - "$root" default-work-root <<<"$SLOT_CONFIGURED_ROOTS_PROGRAM" 2>&1
}

# take_slot <槽那一层> <优先的槽> [trim]：起挑槽的那个进程（带 trim 的只清多出来的槽、不占）。它答的「SLOT note <一行>」逐行照打（清多出来的槽那几行），
#   每一行至多等 SLOT_HOLDER_SECONDS 秒。占到了填 SLOT_DIRECTORY，打「占了槽」那一行、退 0；没占到填 SLOT_FALLBACK_REASON、停掉那个进程、退 1
take_slot() {
  local answer="" held how
  exec {SLOT_HOLDER_FD}< <(python3 -c "$SLOT_HOLDER_PROGRAM" "$1" "$SLOT_LIMIT" "$2" "$$" "${MERGE_AND_VERIFY_BREAK:-}" "${3:-take}" 2>&1)
  SLOT_HOLDER_PID=$!
  while IFS= read -r -t "$SLOT_HOLDER_SECONDS" -u "$SLOT_HOLDER_FD" answer; do
    [[ "$answer" == "SLOT note "* ]] || break
    echo "  ${answer#SLOT note }"
    answer=""
  done
  case "$answer" in
    "SLOT held "*)
      held="${answer#SLOT held }"; how="${held#*$'\t'}"; held="${held%%$'\t'*}"
      SLOT_DIRECTORY="$held"
      echo "  占了槽 $held（$how）"
      return 0 ;;
    "SLOT none "*) SLOT_FALLBACK_REASON="${answer#SLOT none }" ;;
    *) SLOT_FALLBACK_REASON="挑槽的进程 ${SLOT_HOLDER_SECONDS} 秒里没答上来（读到「${answer:-什么都没读到}」）" ;;
  esac
  release_slot
  return 1
}

# release_slot：停占着锁的那个进程（只按它自己的进程号）、等它退（退了锁才放），关掉读它的那一端
release_slot() {
  [[ -n "$SLOT_HOLDER_PID" ]] || return 0
  kill "$SLOT_HOLDER_PID" 2> /dev/null
  wait "$SLOT_HOLDER_PID" 2> /dev/null
  SLOT_HOLDER_PID=""
  [[ -z "$SLOT_HOLDER_FD" ]] || exec {SLOT_HOLDER_FD}<&-
  SLOT_HOLDER_FD=""
  return 0
}

# earlier_attempt_slot <这一趟的日志目录>：任务号有、尝试号是正整数时，打同一层里同一任务、尝试号更小的那几次尝试中尝试号最大、记了 slot 的那一槽（文件头「上一次尝试留下的快照」）；没有不打
earlier_attempt_slot() {
  [[ -n "${TASK_SERVER_TASK:-}" && "${TASK_SERVER_ATTEMPT:-}" =~ ^[1-9][0-9]*$ ]] || return 0
  local record directory task attempt best=0 slot=""
  for record in "$(dirname -- "$1")"/*/"$ATTEMPT_RECORD_NAME"; do
    [[ -f "$record" ]] || continue
    directory="$(dirname -- "$record")"
    [[ "$directory" != "$1" && -f "$directory/$SLOT_RECORD_NAME" ]] || continue
    task=""; attempt=""
    read -r task attempt < "$record" || true
    [[ "$task" == "$TASK_SERVER_TASK" && "$attempt" =~ ^[1-9][0-9]*$ ]] || continue
    ((attempt < TASK_SERVER_ATTEMPT && attempt > best)) || continue
    best="$attempt"; slot="$(head -n 1 -- "$directory/$SLOT_RECORD_NAME")"
  done
  [[ -z "$slot" ]] || printf '%s\n' "$slot"
}

# refresh_slot_snapshot <要验的树> <槽里的快照> <写了或删了的 .rs 单子>：拷进槽（文件头「槽位」）。退 0 拷成（rsync 退 24 同 take_snapshot），别的退 1。
#   拷之前删掉槽里快照下任意一层的 target/、target-*/、__pycache__/（rsync 排除了它们，--delete 删不到；.git 照留），逐个打要删的、删完回读一个都不剩，剩了退 1。
#   弄坏开关 slot-deleted-files-kept：不带 --delete；slot-copy-keeps-old-times：拷完把槽里每份文件的修改时刻拨回拷之前；slot-build-outputs-kept：拷之前不删那几样
refresh_slot_snapshot() {
  local rc=0 itemized="$3.itemized" times="$3.times" stamp name leftover
  local -a delete=(--delete)
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "slot-deleted-files-kept" ]] || delete=()
  mkdir -p -- "$2" || return 1
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "slot-build-outputs-kept" ]]; then
    find "$2" -name .git -prune -o -type d \( -name target -o -name 'target-*' -o -name __pycache__ \) -prune \
      -printf '  删槽里上一趟留下的编译产物 %P\n' -exec rm -rf -- {} + || return 1
    leftover="$(find "$2" -name .git -prune -o -type d \( -name target -o -name 'target-*' -o -name __pycache__ \) -prune -print -quit)"
    [[ -z "$leftover" ]] || { echo "  ✗ 槽里上一趟留下的编译产物 $leftover 没删掉"; return 1; }
  fi
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "slot-copy-keeps-old-times" ]]; then
    find "$2" -path "$2/.git" -prune -o -type f -printf '%T@ %P\n' > "$times" || return 1
  fi
  rsync --recursive --links --checksum --no-times --executability "${delete[@]}" --itemize-changes \
    --exclude .git --exclude 'target/' --exclude 'target-*/' --exclude '__pycache__/' -- "$1/" "$2/" > "$itemized" || rc=$?
  ((rc == 24)) && echo "  ! rsync 退 24：拷的时候有文件不见了（别的会话删了它），槽里照拷的那一刻没有它们"
  ((rc == 0 || rc == 24)) || return 1
  if [[ -f "$times" ]]; then
    while read -r stamp name; do [[ ! -f "$2/$name" ]] || touch -c -d "@$stamp" -- "$2/$name"; done < "$times"
  fi
  # 逐个列的那一份：每行头 11 个字是改了什么（>f 写了一份文件，*deleting 删了一份），隔一个空格是相对快照根的路径
  awk '{ code = substr($0, 1, 11); path = substr($0, 13) } (code ~ /^>f/ || code ~ /^\*deleting/) && path ~ /\.rs$/ { print path }' "$itemized" > "$3"
}

# trim_slot_target <什么时候>：这一槽的编译目录占盘超过登记的 slot-target-limit-gib 就整个删掉（文件头「槽位」的占盘）；本机有进程开着里面的文件不删、打一行。
#   弄坏开关 slot-target-unbounded：不量、不删
trim_slot_target() {
  [[ -n "$SLOT_TARGET" && -d "$SLOT_TARGET" && "${MERGE_AND_VERIFY_BREAK:-}" != "slot-target-unbounded" ]] || return 0
  local bytes output limit_bytes=$((SLOT_TARGET_LIMIT_GIB * 1073741824))
  bytes="$(du -s --block-size=1 -- "$SLOT_TARGET" 2> /dev/null | cut -f1)"
  if [[ ! "$bytes" =~ ^[0-9]+$ ]]; then
    echo "  ! 量不出这一槽的编译目录 $SLOT_TARGET 占了多少盘，${1}这一次不清"
    return 0
  fi
  ((bytes > limit_bytes)) || return 0
  if output="$(python3 "$RUN_BUILD_DIRECTORY" remove "$SLOT_TARGET" 2>&1)"; then
    echo "  清了这一槽的编译目录 $SLOT_TARGET（${1}：占 $((bytes / 1048576)) MiB，超过槽位登记的 slot-target-limit-gib=$SLOT_TARGET_LIMIT_GIB；这一槽在这一台下一次从头编）"
  else
    echo "  ! 这一槽的编译目录 $SLOT_TARGET 超过槽位登记的上限，${1}没清成：${output//$'\n'/；}"
  fi
}

# sweep_slot_targets <槽那一层> <这一台放各槽编译目录的那一层>：槽目录已不在的那几份编译目录删掉（文件头「槽位」的占盘）。弄坏开关 slot-target-unbounded：不删
sweep_slot_targets() {
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "slot-target-unbounded" ]] || return 0
  local directory name output
  for directory in "$2"/slot.*/; do
    [[ -d "$directory" ]] || continue
    directory="${directory%/}"; name="$(basename -- "$directory")"
    [[ ! -e "$1/$name" ]] || continue
    if output="$(python3 "$RUN_BUILD_DIRECTORY" remove "$directory/target" 2>&1)" && rmdir -- "$directory" 2> /dev/null; then
      echo "  删了槽已不在的编译目录 $directory"
    else
      echo "  ! 槽 $1/$name 已不在，这一台的编译目录 $directory 没删成：${output//$'\n'/；}"
    fi
  done
}

# slot_release_command <槽>：打放槽那一条（文件头「槽位」的留着的槽）；收尾那一行、合进主工作区那一段的末一句与槽位汇总都用它，是同一条。
#   弄坏开关 slot-release-keeps-build-outputs：不带 <槽>/repo/target
slot_release_command() {
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "slot-release-keeps-build-outputs" ]]; then
    printf "rm -rf -- '%s/kept' '%s/.owner' '%s/.deliverables'\n" "$1" "$1" "$1"
  else
    printf "rm -rf -- '%s/kept' '%s/.owner' '%s/.deliverables' '%s/repo/target'\n" "$1" "$1" "$1" "$1"
  fi
}

# report_slot_state：槽位汇总（文件头「槽位」的槽位汇总），收尾第一步打：这一趟没占槽的打为什么；槽那一层里有留着的槽的，逐个打槽名、留了多久
#   （从 kept 里的 created 那一天算，读不出取 kept 的修改时刻）、.owner 里的会话与放槽那一条。只报，不收、不改退出码。弄坏开关 slot-fallback-unsummarized：不打
report_slot_state() {
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "slot-fallback-unsummarized" ]] || return 0
  local slot created started since session days now
  local -a kept_lines=()
  if [[ -n "$SLOT_ROOT_DIRECTORY" && -d "$SLOT_ROOT_DIRECTORY" ]]; then
    now="$(date +%s)"
    for slot in "$SLOT_ROOT_DIRECTORY"/slot.*/; do
      slot="${slot%/}"
      [[ -f "$slot/kept" && -d "$slot/repo" ]] || continue
      created="$(sed -n 's/^created: //p' "$slot/kept" 2> /dev/null | head -n 1)"
      if [[ -n "$created" ]] && started="$(TZ=Asia/Tokyo date -d "$created" +%s 2> /dev/null)"; then since="kept 里的 created $created 那一天"
      else started="$(stat -c %Y -- "$slot/kept" 2> /dev/null || echo "$now")"; since="kept 的修改时刻"; fi
      days=$(((now - started) / 86400))
      session="$(sed -n 's/^session: //p' "$slot/.owner" 2> /dev/null | head -n 1)"
      [[ -n "$session" ]] && session="会话 $session" || session=".owner 里没有会话"
      kept_lines+=("    $(basename -- "$slot")：留了 $days 天（从 ${since}算），${session}；放槽：$(slot_release_command "$slot")")
    done
  fi
  [[ -n "$SLOT_FALLBACK_REASON" ]] || ((${#kept_lines[@]})) || return 0
  if [[ -n "$SLOT_FALLBACK_REASON" ]]; then
    echo "  槽位汇总：这一趟不占槽（$SLOT_FALLBACK_REASON），快照与编译目录照一趟一份、编译从头编；槽那一层里留着的槽 ${#kept_lines[@]} 个${kept_lines[*]:+（只报，不收：放槽之前别的趟都不占它们）：}"
  else
    echo "  槽位汇总：占了槽 $SLOT_DIRECTORY；槽那一层里另有留着的槽 ${#kept_lines[@]} 个（只报，不收：放槽之前别的趟都不占它们）："
  fi
  ((${#kept_lines[@]} == 0)) || printf '%s\n' "${kept_lines[@]}"
  return 0
}

# slot_incremental_off：占到槽之后、编之前，这一槽的编译关掉增量（文件头「槽位」的增量编译）。调用方设了 CARGO_INCREMENTAL（非空）的照它、不盖、不删缓存，打一行；
#   没设的 export CARGO_INCREMENTAL=0（这一趟起的每条 cargo 都继承），再删这一槽编译目录下已有的 <档>/incremental 与 <目标三元组>/<档>/incremental，
#   逐个打删了哪个、多大（run_build_directory.py remove：本机有进程开着里面的文件就不删、打一行 !，照往下编）。
#   弄坏开关 slot-incremental-on：不设 CARGO_INCREMENTAL；slot-incremental-cache-kept：不删已有的 incremental；slot-incremental-caller-overridden：调用方设了也盖成 0、照删
slot_incremental_off() {
  local directory size output
  if [[ -n "${CARGO_INCREMENTAL:-}" && "${MERGE_AND_VERIFY_BREAK:-}" != "slot-incremental-caller-overridden" ]]; then
    echo "  增量编译：调用方设了 CARGO_INCREMENTAL=$CARGO_INCREMENTAL，照它编；这一槽编译目录里的 incremental 不删"
    return 0
  fi
  [[ "${MERGE_AND_VERIFY_BREAK:-}" == "slot-incremental-on" ]] || export CARGO_INCREMENTAL=0
  echo "  增量编译：这一槽的编译关掉增量（CARGO_INCREMENTAL=0），删掉这一槽编译目录里已有的 incremental"
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "slot-incremental-cache-kept" ]] || return 0
  for directory in "$SLOT_TARGET"/*/incremental "$SLOT_TARGET"/*/*/incremental; do
    [[ -d "$directory" && ! -L "$directory" ]] || continue
    size="$(du -sh -- "$directory" 2> /dev/null | cut -f1)"
    if output="$(python3 "$RUN_BUILD_DIRECTORY" remove "$directory" 2>&1)"; then
      echo "  删了这一槽编译目录里的增量编译缓存 $directory（${size:-?}）"
    else
      echo "  ! 这一槽编译目录里的增量编译缓存 $directory（${size:-?}）没删成，照往下编：${output//$'\n'/；}"
    fi
  done
  return 0
}

# choose_slot <这一趟的日志目录>：这一趟占不占槽（文件头「槽位」）。占到的填 SLOT_DIRECTORY、SLOT_TARGET，编译目录建好、量过、关掉增量；没占到的填 SLOT_FALLBACK_REASON。
#   取到工作根的填 SLOT_ROOT_DIRECTORY（槽位汇总读）；登记的 slots 是 0 的照样清多出来的槽与这一台编译根下槽已不在的编译目录（文件头「槽位」的多出来的槽）。
#   槽那一层在要验的树里面退 2。弄坏开关 slot-target-on-shared-root：编译目录建在 <槽>/target；peer-target-called-caller：第二台那一层给的编译目录也说成调用方给的；
#   slot-work-root-unchecked：没给自证的编译根时不判工作根是不是这一台缺省的那一个；slot-default-work-root-refused：工作根就是缺省的那一个也判成不是；
#   slot-default-work-root-follows-data-root：缺省的工作根跟着 SINGLEFS_DATA_ROOT 取（slot_default_work_root）
choose_slot() {
  local work_root build_root slot_root problem="" default_work_root default_work_root_problem
  if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
    # 第二台那一层（scripts/hosts/multi-host-run.sh 文件头 ⑤）给的那一份落在它给的 MULTI_HOST_RUN_PEER_SCRATCH 下面
    if [[ -n "${MULTI_HOST_RUN_PEER_SCRATCH:-}" && "$CARGO_TARGET_DIR/" == "${MULTI_HOST_RUN_PEER_SCRATCH%/}/"* && "${MERGE_AND_VERIFY_BREAK:-}" != "peer-target-called-caller" ]]; then
      SLOT_FALLBACK_REASON="第二台那一层（scripts/hosts/multi-host-run.sh）给了编译目录 CARGO_TARGET_DIR（槽只在本机占）"; return 0
    fi
    SLOT_FALLBACK_REASON="调用方给了编译目录 CARGO_TARGET_DIR（不是哪一槽的那一份，按内容拷进槽的快照对着它编会把内容变了的文件判成已编）"; return 0
  fi
  # 只给自证的槽位登记与编译根（MERGE_AND_VERIFY_SLOT_TABLE、MERGE_AND_VERIFY_SLOT_BUILD_ROOT）要连同 SINGLEFS_WORK_ROOT 三样一起给：
  #   缺一样，下面清槽、清编译目录就会拿自证的槽那一层对着真的编译根（或真的槽那一层对着自证的编译根）删
  if [[ -n "${MERGE_AND_VERIFY_SLOT_TABLE:-}${MERGE_AND_VERIFY_SLOT_BUILD_ROOT:-}" ]] \
     && [[ -z "${MERGE_AND_VERIFY_SLOT_TABLE:-}" || -z "${MERGE_AND_VERIFY_SLOT_BUILD_ROOT:-}" || -z "${SINGLEFS_WORK_ROOT:-}" ]]; then
    SLOT_FALLBACK_REASON="自证只给了 MERGE_AND_VERIFY_SLOT_TABLE、MERGE_AND_VERIFY_SLOT_BUILD_ROOT、SINGLEFS_WORK_ROOT 里的一部分，三样不齐：不读工作根与多机配置，不占槽也不清"; return 0
  fi
  if ! work_root="$(python3 "$root/.claude/hooks/lib_work_root.py")"; then problem="取不到工作根（.claude/hooks/lib_work_root.py 跑不起来）"
  elif ! build_root="$(slot_build_root)"; then problem="${build_root:-取不到这一台的编译根}"; fi
  # 没给自证的编译根时编译根是这一台真的那一个，工作根就要是这一台缺省的那一个（slot_default_work_root：与编译根读同一份多机配置、不看 SINGLEFS_DATA_ROOT，两边都按真路径比）：
  #   换了工作根（连同 SINGLEFS_DATA_ROOT 一起换的也算），下面清编译目录拿那个工作根的槽那一层对着真的编译根比，缺省工作根下各槽（别的趟占着的、留着的）的编译目录都会当成槽已不在删掉。
  #   不是的不占也不清；只换了 SINGLEFS_DATA_ROOT、工作根仍是缺省的那一个的照占
  if [[ -z "$problem" && -z "${MERGE_AND_VERIFY_SLOT_BUILD_ROOT:-}" && "${MERGE_AND_VERIFY_BREAK:-}" != "slot-work-root-unchecked" ]]; then
    if ! default_work_root="$(slot_default_work_root)" || [[ -z "$default_work_root" ]]; then
      default_work_root_problem="${default_work_root//$'\n'/；}"
      SLOT_FALLBACK_REASON="取不到这一台缺省的工作根（${default_work_root_problem:-取它的那一步什么都没打}），比不了工作根 $work_root 是不是它，不占也不清"; return 0
    fi
    if [[ "$(realpath -m -- "$work_root")" != "$(realpath -m -- "$default_work_root")" || "${MERGE_AND_VERIFY_BREAK:-}" == "slot-default-work-root-refused" ]]; then
      SLOT_FALLBACK_REASON="工作根 $work_root 不是这一台缺省的那一个（$default_work_root），不占也不清：编译根是这一台真的那一个，拿这个工作根的槽那一层比会删掉缺省工作根下各槽的编译目录"; return 0
    fi
  fi
  if ((SLOT_LIMIT == 0)); then
    # 只清、不占：槽那一层里有槽才起挑槽的进程（trim）
    if [[ -n "$problem" ]]; then
      echo "  ! 登记的 slots 是 0，多出来的槽与它们的编译目录这一趟没清：$problem"
    else
      SLOT_ROOT_DIRECTORY="$work_root/$SLOT_ROOT_NAME"
      if compgen -G "$SLOT_ROOT_DIRECTORY/slot.*" > /dev/null; then
        take_slot "$SLOT_ROOT_DIRECTORY" "" trim || true
        [[ "$SLOT_FALLBACK_REASON" == 只清多出来的槽* ]] || echo "  ! 清多出来的槽没做完：$SLOT_FALLBACK_REASON"
      fi
      sweep_slot_targets "$SLOT_ROOT_DIRECTORY" "$build_root/$SLOT_BUILD_DIRECTORY_NAME"
    fi
    SLOT_FALLBACK_REASON="槽位登记 $SLOT_TABLE 的 slots 是 0"; return 0
  fi
  [[ -z "$problem" ]] || { SLOT_FALLBACK_REASON="$problem"; return 0; }
  slot_root="$work_root/$SLOT_ROOT_NAME"
  mkdir -p -- "$slot_root" || { SLOT_FALLBACK_REASON="建不了槽那一层 $slot_root"; return 0; }
  SLOT_ROOT_DIRECTORY="$slot_root"
  [[ "$(cd "$slot_root" && pwd -P)/" != "$(cd "$SOURCE_TREE" && pwd -P)/"* ]] \
    || die "槽那一层 $slot_root 在要验的树 $SOURCE_TREE 里面：快照会把自己拷进自己，什么都没拷" "工作根设到树外（SINGLEFS_WORK_ROOT 不设就是数据根下的 work）"
  if ! take_slot "$slot_root" "$(earlier_attempt_slot "$1")"; then
    sweep_slot_targets "$slot_root" "$build_root/$SLOT_BUILD_DIRECTORY_NAME"
    return 0
  fi
  printf '%s\n' "$SLOT_DIRECTORY" > "$1/$SLOT_RECORD_NAME" || echo "  ! 没记下这一趟占的槽（写 $1/$SLOT_RECORD_NAME 失败）：撞上限重来的下一次尝试不一定先挑它"
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "slot-target-on-shared-root" ]]; then SLOT_TARGET="$SLOT_DIRECTORY/target"
  else SLOT_TARGET="$build_root/$SLOT_BUILD_DIRECTORY_NAME/$(basename -- "$SLOT_DIRECTORY")/target"; fi
  sweep_slot_targets "$slot_root" "$build_root/$SLOT_BUILD_DIRECTORY_NAME"
  trim_slot_target 开跑前
  mkdir -p -- "$SLOT_TARGET" && SLOT_TARGET="$(cd "$SLOT_TARGET" && pwd -P)" \
    || die "建不了这一槽的编译目录 $SLOT_TARGET" "看这一台的编译根（多机配置 BUILD_ROOT_BY_HOST 里这一台那一项）在不在、可不可写、盘满没满" 1
  slot_incremental_off
  SNAPSHOT_RELEASE_TEXT="放这一趟留着的槽：$(slot_release_command "$SLOT_DIRECTORY")（快照与编译目录留给下一趟接着编）"
}
