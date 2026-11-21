#!/usr/bin/env bash
# admission: always 被 scripts/experiment-apparatus/replay.sh 在同目录 source 进去的那一段，判的是此刻的仓与这一趟的参数，上一次的结论不替这一次作保
# run-condition: none 不单独执行：由 replay.sh 的 source 载入，环境要求同 replay.sh（文件头「GPU」「编译目录」）
# replay-table-and-dispatch-limits.sh：replay.sh 的一段（计时行的滞后表、同时开几行的缺省与内存判法、复跑表的实验号与行项）。只定义函数与载入时赋值的全局量，不改 shell 选项。
# 这一段按 replay.sh 里 source 它的那一行的位置载入。文件头的用法、复跑表、弄坏开关表、退出码都在 replay.sh。
# 读滞后表 $1（一行 `<留存产物名><制表符><理由>`，# 开头的行与空行跳过）进 TIMING_LAG；$2 是复跑表原文，$3 是留存产物目录。
# 每个问题往 stdout 打一行，有问题返回 1：表不存在；没有制表符隔开的理由列；没写理由（至少 8 个字）；名字重复；名字不是 $2 里某条 timing 行的第 4 列；留存产物在树里且自己已经有字段（该删这一行）。
# 留存产物不在树里是对的（多数已归档进版本库，replay_judge_fresh_product 从 git 历史取回再比），所以这里只认复跑表，不认文件在不在。
load_timing_lag() {
  local file="$1" table_text="$2" results_directory="$3" line name reason line_number=0 problems=0 timing_products reading
  TIMING_LAG=()
  if [[ ! -f "$file" ]]; then echo "滞后表 $file 不存在"; return 1; fi
  timing_products="$(printf '%s\n' "$table_text" | awk -F'|' '$5 == "timing" {print $4}')"
  while IFS= read -r line || [[ -n "$line" ]]; do
    line_number=$((line_number + 1))
    [[ -z "${line//[[:space:]]/}" || "$line" =~ ^[[:space:]]*# ]] && continue
    if [[ "$line" != *$'\t'* ]]; then echo "$file:$line_number 没有用制表符隔开的理由（两列：留存产物名、理由）：$line"; problems=$((problems + 1)); continue; fi
    name="${line%%$'\t'*}"; reason="${line#*$'\t'}"
    name="${name//[[:space:]]/}"; reason="${reason#"${reason%%[![:space:]]*}"}"
    if [[ -z "$name" || ${#reason} -lt 8 ]]; then echo "$file:$line_number 名字空着或理由不足 8 个字：$line"; problems=$((problems + 1)); continue; fi
    if [[ -n "${TIMING_LAG["$name"]+set}" ]]; then echo "$file:$line_number 重复登记 $name"; problems=$((problems + 1)); continue; fi
    TIMING_LAG["$name"]="$reason"
    if ! grep -qxF -- "$name" <<<"$timing_products"; then
      echo "$file:$line_number $name 不是复跑表任何一条 timing 行的留存产物（第 4 列）"; problems=$((problems + 1)); continue
    fi
    if [[ -f "$results_directory/$name" ]]; then
      reading="$(timing_load_reading_raw "$results_directory/$name")"
      if [[ "${reading%% *}" != missing ]]; then echo "$file:$line_number $name 自己的收尾行已经有 loadavg1 / cpus，滞后表只缩不涨，删这一行"; problems=$((problems + 1)); fi
    fi
  done <"$file"
  ((problems == 0))
}

# 同时跑几条实验的缺省：登记的重活默认并行度（.claude/heavy-thread-default，经 scripts/hosts/heavy-threads.sh；SINGLEFS_THREAD_CAP 设了取它）。
# 弄坏开关 REPLAY_BREAK=jobs-from-nproc 下取核数的一半。
default_replay_jobs() {
  if [[ "$REPLAY_BROKEN" == jobs-from-nproc ]]; then
    echo $(( $(nproc 2>/dev/null || echo 4) / 2 ))
  else
    heavy_thread_default
  fi
}

# 派行的判法（文件头「派行」）。下面两个函数读这几个全局量：在跑几行 running、留的内存之和 running_bytes，
#   同时至多几行 REPLAY_JOBS，放得下与否比 REPLAY_FIT_MEMORY（字节：这一台能给的内存，取不到取这一次拿到的上限 REPLAY_MEMORY_CAP，都取不到是空串）。要卡的行不经它们，在最后一段（文件头「GPU」）。
REPLAY_ROW_BASE_BYTES=$((256 << 20))
replay_row_reservation() { # <量到的峰值字节>：这一行留多少内存
  echo $(( $1 + $1 / 4 + (64 << 20) ))
}
# 这一行独占吗：<实验号> <判据> <量到的峰值，没量过是空串> <线程数：量过的是量到的，没量过的调用处给每台默认重活线程数（取不到是空串、照 1 算），只给弄坏开关 threads-bound-kept>。弄坏开关 unmeasured-row-exclusive、threads-bound-kept、exclusive-row-shared
replay_row_is_exclusive() {
  [[ -n "$3" || "$REPLAY_BROKEN" != unmeasured-row-exclusive ]] || return 0
  if [[ "$REPLAY_BROKEN" == threads-bound-kept ]] && (( ${4:-1} > REPLAY_JOBS )); then return 0; fi
  [[ "$REPLAY_BROKEN" != exclusive-row-shared ]] || return 1
  [[ "$2" == timing || "$1" == E181 ]]
}
# 这一行现在起放得下吗：<留的内存字节>。弄坏开关 row-count-unbound、memory-bound-ignored
replay_row_memory_bound_fits() {
  (( running == 0 )) && return 0
  if (( running >= REPLAY_JOBS )) && [[ "$REPLAY_BROKEN" != row-count-unbound ]]; then return 1; fi
  if [[ -n "$REPLAY_FIT_MEMORY" ]] && (( running_bytes + $1 + REPLAY_ROW_BASE_BYTES > REPLAY_FIT_MEMORY )) && [[ "$REPLAY_BROKEN" != memory-bound-ignored ]]; then return 1; fi
  return 0
}

# 同时放得下几行比哪个内存 REPLAY_FIT_MEMORY（文件头「派行」）：这一台能给的内存（scripts/hosts/task_memory.py 的 host_slice_cap_bytes：
# 多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项），不拿这一次拿到的上限 REPLAY_MEMORY_CAP 压（那是服务给这一次尝试的额度，从小处起、撞了才放大）；
# 这一台那一项取不到（多机配置不在、没写、认不出这一台、取的时候出了异常、读数脚本没报这一行）取 REPLAY_MEMORY_CAP。读完往标准错误打一行依据。
# 参数是读数脚本 read 的 host-cap 那一行（replay_row_costs.py 文件头「用法」）：<字节或 -> <依据>。
# 弄坏开关 REPLAY_BREAK=memory-by-task-cap：照 REPLAY_MEMORY_CAP 比；host-cap-fallback-dropped：取不到时不退回 REPLAY_MEMORY_CAP、不比内存
replay_settle_fit_memory() { # <这一台能给的内存：字节或 -> <依据>
  local host_cap="$1" host_text="$2"
  REPLAY_FIT_MEMORY="$REPLAY_MEMORY_CAP"
  if [[ "$host_cap" == - ]]; then
    [[ "$REPLAY_BROKEN" != host-cap-fallback-dropped ]] || REPLAY_FIT_MEMORY=""
    echo "  · 派行：这一台能给的内存取不到（$host_text），同时放得下几行比内存上限 ${REPLAY_MEMORY_CAP:-取不到、不比内存}" >&2
  elif [[ "$REPLAY_BROKEN" == memory-by-task-cap ]]; then
    echo "  · 派行：弄坏开关 memory-by-task-cap：同时放得下几行照这一次拿到的上限 ${REPLAY_MEMORY_CAP:-取不到、不比内存} 比" >&2
  else
    REPLAY_FIT_MEMORY="$host_cap"
    echo "  · 派行：同时放得下几行比这一台能给的内存 $host_cap 字节（$host_text），不拿这一次拿到的上限压" >&2
  fi
}

# 复跑表里的实验号（文件头「入口脚本」）：从本文件原文读 TABLE 那一段（在仓根调，脚本开头已 cd 到仓根），每个实验号一行
# 「<实验号><TAB><几行><TAB><二进制或驱动，逗号连>」，按表里头一次出现的次序。--list-items 打它，「实验号写错退 2」拿实验号那一列比。
replay_table_experiments() {
  awk -F'|' '/^TABLE=\$\(cat <<.TSV.$/ { inside = 1; next }
             inside && /^TSV$/ { exit }
             inside && /^E[0-9A-Za-z]*\|/ { if (!($1 in rows)) order[++count] = $1; rows[$1]++; binaries[$1] = binaries[$1] (binaries[$1] == "" ? "" : ",") $2 }
             END { for (index_in_order = 1; index_in_order <= count; index_in_order++) { name = order[index_in_order]; printf "%s\t%d\t%s\n", name, rows[name], binaries[name] } }' \
    scripts/experiment-apparatus/replay.sh
}

# 复跑表里的每一行（文件头「入口脚本」）：「<实验号>.<行序号><TAB>1<TAB>二进制或驱动」，行序号是这个实验号的行在表里从 1 数的次序，按表里的次序。
# --list-items 接在实验号后面打它，--item 后面的项名拿第一列比。
replay_table_row_items() {
  awk -F'|' '/^TABLE=\$\(cat <<.TSV.$/ { inside = 1; next }
             inside && /^TSV$/ { exit }
             inside && /^E[0-9A-Za-z]*\|/ { ordinal[$1]++; printf "%s.%d\t1\t%s\n", $1, ordinal[$1], $2 }' \
    scripts/experiment-apparatus/replay.sh
}
