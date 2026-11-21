#!/usr/bin/env bash
# admission: always 被 scripts/experiment-apparatus/replay.sh 在同目录 source 进去的那一段，判的是此刻的仓与这一趟的参数，上一次的结论不替这一次作保
# run-condition: none 不单独执行：由 replay.sh 的 source 载入，环境要求同 replay.sh（文件头「GPU」「编译目录」）
# replay-dispatch.sh：replay.sh 的一段（派行、进度、监看、收齐、判完的汇总与退出码）。含载入时就执行的语句（replay.sh 载入次序里这一段的顺序流程），接着 source 它的那一行往下走；exit 与 replay.sh 里的 exit 一样退出整趟。
# 这一段按 replay.sh 里 source 它的那一行的位置载入。文件头的用法、复跑表、弄坏开关表、退出码都在 replay.sh。
# 进度（文件头「进度」）：打一行到标准错误。弄坏开关 row-progress-silent 下不打
replay_row_progress_line() { # <一行>
  [[ "$REPLAY_BROKEN" == row-progress-silent ]] || printf '%s\n' "$1" >&2
}
# 告诉监看进程（replay_row_costs.py 文件头「用法」）：改名换上，它读不到半行。没起监看进程的不做
replay_row_tell_sampler() { # <tag> <payload|end> <内容>
  [[ -n "${REPLAY_ROW_SAMPLER_PID:-}" ]] || return 0
  printf '%s\n' "$3" >"$REPLAY_ROW_WATCH/$1.$2.tmp" && mv -f -- "$REPLAY_ROW_WATCH/$1.$2.tmp" "$REPLAY_ROW_WATCH/$1.$2"
}
# 这一行平均同时占几个核：bash 内建 times 记下的子进程用户态加内核态用时 ÷ 墙钟，向上取整，至少 1；读不出交 1
replay_row_threads_used() { # <times 的输出文件> <墙钟毫秒>
  local children cpu_milliseconds
  children="$(sed -n 2p -- "$1" 2>/dev/null)"
  if [[ "$children" =~ ^([0-9]+)m([0-9]+)[.,]([0-9]+)s[[:space:]]+([0-9]+)m([0-9]+)[.,]([0-9]+)s$ ]]; then
    cpu_milliseconds=$(( (10#${BASH_REMATCH[1]} + 10#${BASH_REMATCH[4]}) * 60000 + (10#${BASH_REMATCH[2]} + 10#${BASH_REMATCH[5]}) * 1000 + 10#${BASH_REMATCH[3]} + 10#${BASH_REMATCH[6]} ))
    (( $2 > 0 && cpu_milliseconds > $2 )) && { echo $(( (cpu_milliseconds + $2 - 1) / $2 )); return 0; }
  fi
  echo 1
}
# 一行收尾的那一条进度：结局取结果行的判定那一列，峰值取监看进程写下的
replay_row_report_end() { # <tag> <墙钟毫秒> <线程数>
  local line title peak
  line="$(cat -- "$OUT_DIR/$1.line" 2>/dev/null)"
  read -r _ _ title _ <<<"$line"
  peak="$(cat -- "${REPLAY_ROW_WATCH:-/nonexistent}/$1.peak" 2>/dev/null)"
  replay_row_progress_line "REPLAY_ROW $1 判 ${title:-没判} 用时 $(( $2 / 1000 )) 秒 峰值 ${peak:-没量到} 线程 $3｜$line"
}

# 一行判完记进判定库（文件头「按行复用」）：照这一行的行键记下判法写的那个判定（$OUT_DIR/<tag>.verdict）。判绿的（字节一致、仅计时不同）记绿；
#   记红、撤绿：判成对不上（drift）或跑不了（broken）的照同一把键记下、盖掉这把键原有的绿，下一趟这一行现判。本次不判不碰库。
#   记在 replay_judge_fresh_product 外面：它的正文进每一行的行键，存库那一步（起 admission.py library-files store）进了它的闭包，每一行的键都拒算。
#   新产物不在（跑不了的那几种可能一个字都没写）记一份空的，不因为拷不了产物就留着原有的绿。
#   弄坏开关 red-not-recorded 下对不上的不记、broken-not-recorded 下跑不了的不记
remember_row_outcome() { # <tag> <记录目录，空串不记> <新产物> <前三列>
  local verdict product="$3"
  verdict="$(cat -- "$OUT_DIR/$1.verdict" 2>/dev/null)"
  case "$verdict" in
    pass|timing_only) ;;
    drift) [[ "$REPLAY_BROKEN" == red-not-recorded ]] && return 0 ;;
    broken) [[ "$REPLAY_BROKEN" == broken-not-recorded ]] && return 0 ;;
    *) return 0 ;;
  esac
  [[ -f "$product" ]] || product=/dev/null
  remember_row_verdict "$2" "$verdict" "$(cat -- "$OUT_DIR/$1.line")" "$product" "$4"
}



# 开跑前那一判：按表里的次序给每一行定 tag 与判定（decide_row），数出要现跑的几行；一行都不用现跑就不试编 gpu 特性、不编（文件头「按行复用」）；
# 要现跑的行里一行要卡的都没有（admission.py replay-row-inputs 第 4 列全是 0）就不试编 gpu 特性（文件头「GPU」）。
# 弄坏开关 REPLAY_BREAK=gpu-for-every-run：要现跑的行一律当要卡
declare -a ROW_TEXTS=() REPLAY_ORDER=() TO_RUN_BINARIES=()
row_sequence=0; to_run=0; to_run_drivers=0; to_run_gpu=0
: >"$OUT_DIR/row-costs.request"
while IFS='|' read -r exp bin args stored kind; do
  [[ -z "$exp" ]] && continue
  want_row "$exp" "$bin" "$args" || continue
  row_sequence=$((row_sequence+1))
  if [[ -n "${ITEM_EXPERIMENT["$exp"]+set}" ]]; then printf '  · --item %s.%s：本趟记号 %s.%s\n' "$exp" "${ROW_ORDINAL["$exp|$bin|$args"]}" "$exp" "$row_sequence" >&2; fi
  admission_key_of_row "$exp" "$bin" "$args"   # 按表里的原样查（$REPLAY_DEV 这类还没展开）
  row_identity="$exp|$bin|$args"
  ROW_TEXTS+=("$exp|$bin|$args|$stored|$kind")
  REPLAY_ORDER+=("$exp.$row_sequence")
  [[ -n "${ROW_REFUSAL["$row_identity"]+set}" ]] && refused_rows=$((refused_rows + 1))
  decide_row "$exp" "$exp.$row_sequence" "$bin" "$stored" "$ROW_ADMISSION_KEY" "${ROW_REUSE_KEY["$row_identity"]-}" "$row_identity"
  if [[ "$(cat -- "$OUT_DIR/$exp.$row_sequence.decision" 2>/dev/null)" == run ]]; then
    to_run=$((to_run + 1))
    printf '%s\t%s\n' "$exp.$row_sequence" "$row_identity" >>"$OUT_DIR/row-costs.request"
    if [[ "$bin" == @* ]]; then to_run_drivers=$((to_run_drivers + 1)); else TO_RUN_BINARIES+=("$bin"); fi
    if [[ "${ROW_USES_GPU["$row_identity"]:-0}" == 1 || "$REPLAY_BROKEN" == gpu-for-every-run ]]; then
      to_run_gpu=$((to_run_gpu + 1))
    fi
    if [[ -n "${ROW_CHANGE_NOTE["$row_identity"]:-}" ]]; then printf '  · %s 要现跑：%s\n' "$exp" "${ROW_CHANGE_NOTE["$row_identity"]}" >&2; fi
    if [[ -n "${ROW_REFUSAL["$row_identity"]+set}" ]]; then printf '  · %s 要现跑：行键%s\n' "$exp" "${ROW_REFUSAL["$row_identity"]}" >&2; fi
  fi
done <<<"$TABLE"

# 派行要的读数（文件头「派行」）：编译之前读，一行比上限还大就不编、不跑。读不来的各行照没量过办、不比内存
REPLAY_ROW_COSTS_SCRIPT=scripts/experiment-apparatus/replay_row_costs.py
REPLAY_MEMORY_CAP=""; REPLAY_FIT_MEMORY=""; REPLAY_ROW_PEAK_RECORDS=""
declare -A ROW_PEAK_OF_TAG=() ROW_THREADS_OF_TAG=()   # 量到的线程数只给弄坏开关 threads-bound-kept 用
if (( to_run )); then
  replay_read_options=()
  [[ "$REPLAY_BROKEN" != host-cap-unreported ]] || replay_read_options=(--no-host-cap)
  replay_host_cap=-; replay_host_text="读数脚本没报 host-cap 那一行"
  if [[ ! -f "$REPLAY_ROW_COSTS_SCRIPT" ]]; then
    echo "  · 派行：这棵树里没有 $REPLAY_ROW_COSTS_SCRIPT，各行照没量过办" >&2
  elif replay_costs_output="$(python3 "$REPLAY_ROW_COSTS_SCRIPT" read "$OUT_DIR/row-costs.request" "${replay_read_options[@]}")"; then
    while IFS=$'\t' read -r replay_costs_kind replay_costs_first replay_costs_second replay_costs_third; do
      case "$replay_costs_kind" in
        cap) [[ "$replay_costs_first" == - ]] || REPLAY_MEMORY_CAP="$replay_costs_first"; echo "  · 派行：内存上限 ${REPLAY_MEMORY_CAP:-取不到、不比内存}（$replay_costs_second）" >&2 ;;
        host-cap) replay_host_cap="$replay_costs_first"; replay_host_text="$replay_costs_second" ;;
        records) [[ "$replay_costs_first" == - ]] || REPLAY_ROW_PEAK_RECORDS="$replay_costs_first"; echo "  · 派行：峰值记录 ${REPLAY_ROW_PEAK_RECORDS:-不读不写}（$replay_costs_second）" >&2 ;;
        row) ROW_PEAK_OF_TAG["$replay_costs_first"]="$replay_costs_second"; ROW_THREADS_OF_TAG["$replay_costs_first"]="$replay_costs_third" ;;
      esac
    done <<<"$replay_costs_output"
    replay_settle_fit_memory "$replay_host_cap" "$replay_host_text"
  else
    echo "  ! 派行：python3 $REPLAY_ROW_COSTS_SCRIPT read 退 $?（原因在上面），这一趟各行照没量过办" >&2
  fi
  # 一行比上限还大（文件头「派行」）：不编、不白跑前面的行。弄坏开关 oversized-row-not-refused 下照起
  replay_oversized_rows=()
  if [[ -n "$REPLAY_MEMORY_CAP" ]]; then
    for tag in "${!ROW_PEAK_OF_TAG[@]}"; do
      (( $(replay_row_reservation "${ROW_PEAK_OF_TAG[$tag]}") + REPLAY_ROW_BASE_BYTES > REPLAY_MEMORY_CAP )) && replay_oversized_rows+=("$tag（量过 ${ROW_PEAK_OF_TAG[$tag]} 字节）")
    done
  fi
  if (( ${#replay_oversized_rows[@]} )) && [[ "$REPLAY_BROKEN" != oversized-row-not-refused ]]; then
    echo "  ✗ 撞内存上限（memory-cap-hit）：内存上限 $REPLAY_MEMORY_CAP 字节装不下 ${replay_oversized_rows[*]}，一行都没跑" >&2
    echo "     → 怎么办：经统一任务服务跑的不用管，退 250 由服务放大上限重排（task-server/task_server_lib.py 文件头「错误类型」memory-cap-hit）；到了那台总上限的由服务打回 memory-cap-at-maximum，查这一行为什么要这么多" >&2
    exit 250
  fi
fi

if (( to_run == 0 )); then
  echo "  · 开跑前判完：${#REPLAY_ORDER[@]} 行里要现跑的 0 行，不试编 gpu 特性、不编" >&2
else
  if (( to_run_gpu )); then
    prepare_checker_tier_features "$OUT_DIR/gpu-build.log" || exit 78
  elif [[ "$REPLAY_GPU_SWITCH" == 1 ]]; then
    echo "  · 开跑前判完：要现跑的 $to_run 行都不点 CHECKER_TIER_FEATURES，不试编 gpu 特性" >&2
  fi
  if (( to_run_drivers )) || [[ "$REPLAY_BROKEN" == build-everything ]]; then
    build_research_binaries "$OUT_DIR/build.log" || exit 2
  else
    build_research_binaries "$OUT_DIR/build.log" $(printf '%s\n' "${TO_RUN_BINARIES[@]}" | sort -u) || exit 2
  fi
fi

if (( to_run )); then replay_restore_referenced_products; fi

# 同时至多几行：缺省取登记的重活默认并行度（default_replay_jobs，不看忙闲），REPLAY_JOBS 压过它。各条实验的产物、镜像（e9/e45/e58/e140.img）与结果文件各用各的，没有共用的可写状态。
# 哪几行一起起照文件头「派行」。
REPLAY_JOBS="${REPLAY_JOBS:-$(default_replay_jobs)}"
[[ "$REPLAY_JOBS" -lt 1 ]] && REPLAY_JOBS=1
declare -a REPLAY_PIDS=() REPLAY_EXCLUSIVE_ROWS=() REPLAY_GPU_ROWS=() REPLAY_DRIVER_LEASED_ROWS=()
declare -A REPLAY_RUNNING_BYTES=()   # 在跑的行，按进程号
running=0; running_bytes=0
replay_row_start() { # <复跑表那一行> <序号> <留的内存字节>
  local exp bin args stored kind row_identity row_header_writer row_reuse_key_of_this_row
  IFS='|' read -r exp bin args stored kind <<<"$1"
  admission_key_of_row "$exp" "$bin" "$args"
  row_identity="$exp|$bin|$args"
  row_header_writer="${ADMISSION_ROW_HEADER_WRITER["$row_identity"]-device}"
  row_reuse_key_of_this_row="${ROW_REUSE_KEY["$row_identity"]-}"   # 按行复用的键（文件头「按行复用」），按表里的原样查
  args="${args//\$REPLAY_DEV140/$REPLAY_DEV140}"  # 先换长的，否则前缀会被短的吃掉
  args="${args//\$REPLAY_DEV58/$REPLAY_DEV58}"
  args="${args//\$REPLAY_DEV45/$REPLAY_DEV45}"
  args="${args//\$REPLAY_DEV/$REPLAY_DEV}"   # 表里写字面量 $REPLAY_DEV，这里才展开
  replay_one "$exp" "$2" "$bin" "$args" "$stored" "$kind" "$ROW_ADMISSION_KEY" "$row_header_writer" "$row_reuse_key_of_this_row" "$row_identity" &
  REPLAY_PIDS+=($!)
  REPLAY_RUNNING_BYTES[$!]="$3"
  running=$((running + 1)); running_bytes=$((running_bytes + $3))
}
# 等到至少一行收尾、把它的留量还回去。按 /proc 里的状态认（不在、或已是僵尸）：不用 wait -n，监看进程也是这个 shell 的子进程，它会把 wait -n 等住
replay_row_reap_finished() {
  local pid stat_line reaped=0
  while :; do
    for pid in "${!REPLAY_RUNNING_BYTES[@]}"; do
      if IFS= read -r stat_line 2>/dev/null <"/proc/$pid/stat" && [[ "${stat_line##*) }" != Z* ]]; then continue; fi
      running=$((running - 1)); running_bytes=$((running_bytes - REPLAY_RUNNING_BYTES[$pid]))
      unset "REPLAY_RUNNING_BYTES[$pid]"
      reaped=1
    done
    (( reaped )) && return 0
    sleep 0.1
  done
}
# 可并行的跑完、独占的起之前等 1 分钟负载落下来（文件头「派行」）：不等的话前面可并行的行留下的负载让计时行记本次不判
replay_wait_for_load_to_settle() {
  local waited=0 limit="${REPLAY_TIMING_SETTLE_SECONDS:-90}" load cpus
  cpus="$(heavy_thread_default)"
  while (( waited < limit )); do
    read -r load _ </proc/loadavg
    awk -v load_average="$load" -v cpus="$cpus" -v ratio="$TIMING_LOAD_RATIO" 'BEGIN { exit !(load_average / cpus <= ratio) }' && return 0
    sleep 5
    waited=$((waited + 5))
  done
}
# 监看进程（replay_row_costs.py sample）：量每一行的峰值、报在跑的进度；等它读完上限的取法再派行（至多 10 秒），不然头几行量不到
REPLAY_ROW_SAMPLER_PID=""; REPLAY_ROW_WATCH=""
if (( to_run )) && [[ -f "$REPLAY_ROW_COSTS_SCRIPT" ]]; then
  REPLAY_ROW_WATCH="$OUT_DIR/row-watch"
  rm -rf -- "${REPLAY_ROW_WATCH:?}"
  mkdir -p -- "$REPLAY_ROW_WATCH"
  replay_sampler_options=(--progress-seconds "${REPLAY_PROGRESS_SECONDS:-300}")
  [[ "$REPLAY_BROKEN" != row-progress-silent ]] || replay_sampler_options=(--progress-seconds 0)
  [[ "$REPLAY_BROKEN" != row-peak-unrecorded ]] || replay_sampler_options+=(--no-records)
  [[ "$REPLAY_BROKEN" != failed-row-peak-recorded ]] || replay_sampler_options+=(--record-failed)
  python3 "$REPLAY_ROW_COSTS_SCRIPT" sample "$REPLAY_ROW_WATCH" "${REPLAY_ROW_PEAK_RECORDS:--}" "${replay_sampler_options[@]}" >&2 &
  REPLAY_ROW_SAMPLER_PID=$!
  for _ in {1..100}; do
    [[ -e "$REPLAY_ROW_WATCH/ready" ]] && break
    sleep 0.1
  done
fi
REPLAY_DISPATCH_STARTED=1
# 本脚本不领卡（文件头「GPU」）：环境里原有的 GPU_CARDS_BY_PRIORITY 先收起，不要卡的行看不到卡；要卡那一段原样交回去（没有的不设）
REPLAY_CALLER_CARDS="${GPU_CARDS_BY_PRIORITY-}"; replay_caller_cards_given="${GPU_CARDS_BY_PRIORITY+given}"
unset GPU_CARDS_BY_PRIORITY
# 要卡的行（开了 GPU、admission.py replay-row-inputs 第 4 列为 1）先挑出来，挪到最后一段（文件头「派行」）
row_sequence=0
for row_text in ${ROW_TEXTS[@]+"${ROW_TEXTS[@]}"}; do
  row_sequence=$((row_sequence + 1))
  IFS='|' read -r exp bin args stored kind <<<"$row_text"
  [[ "$(cat -- "$OUT_DIR/$exp.$row_sequence.decision" 2>/dev/null)" == run ]] || continue
  [[ "$REPLAY_GPU_SWITCH" == 1 && "${ROW_USES_GPU["$exp|$bin|$args"]:-0}" == 1 ]] || continue
  # 本脚本不交卡的行（文件头「GPU」）另放一批
  if replay_row_driver_leases_its_own_card "$bin"; then REPLAY_DRIVER_LEASED_ROWS+=("$row_sequence"); else REPLAY_GPU_ROWS+=("$row_sequence"); fi
done
# 弄坏开关 cards-shown-to-every-row（文件头「派行」）：派行之前就交出卡串，不要卡的行也看得到卡
if (( ${#REPLAY_GPU_ROWS[@]} )) && [[ -n "$replay_caller_cards_given" && "$REPLAY_BROKEN" == cards-shown-to-every-row ]]; then
  export GPU_CARDS_BY_PRIORITY="$REPLAY_CALLER_CARDS"
fi
replay_shared_rows=0
row_sequence=0
for row_text in ${ROW_TEXTS[@]+"${ROW_TEXTS[@]}"}; do
  row_sequence=$((row_sequence + 1))
  IFS='|' read -r exp bin args stored kind <<<"$row_text"
  tag="$exp.$row_sequence"
  [[ "$(cat -- "$OUT_DIR/$tag.decision" 2>/dev/null)" == run ]] || continue   # 复用、输入没变、跑不了：开跑前那一判已经写好了这一行
  [[ " ${REPLAY_GPU_ROWS[*]-} ${REPLAY_DRIVER_LEASED_ROWS[*]-} " == *" $row_sequence "* ]] && continue   # 要卡的行在最后一段
  if replay_row_is_exclusive "$exp" "$kind" "${ROW_PEAK_OF_TAG[$tag]-}" "${ROW_THREADS_OF_TAG[$tag]-$(heavy_thread_default)}"; then
    REPLAY_EXCLUSIVE_ROWS+=("$row_sequence")
    continue
  fi
  row_reservation="$(replay_row_reservation "${ROW_PEAK_OF_TAG[$tag]:-0}")"
  until replay_row_memory_bound_fits "$row_reservation"; do replay_row_reap_finished; done
  replay_row_start "$row_text" "$row_sequence" "$row_reservation"
  replay_shared_rows=$((replay_shared_rows + 1))
done
while (( running )); do replay_row_reap_finished; done
if (( to_run )); then
  echo "  · 派行：要现跑的 $to_run 行里可并行的 $replay_shared_rows 行跑完了（同时至多 $REPLAY_JOBS 行），接着一行一行跑独占的 ${#REPLAY_EXCLUSIVE_ROWS[@]} 行，最后是要卡的 ${#REPLAY_GPU_ROWS[@]} 行（交环境里的卡串）与 ${#REPLAY_DRIVER_LEASED_ROWS[@]} 行（本脚本不交卡）" >&2
fi
if (( replay_shared_rows && ${#REPLAY_EXCLUSIVE_ROWS[@]} )); then replay_wait_for_load_to_settle; fi
for row_sequence in ${REPLAY_EXCLUSIVE_ROWS[@]+"${REPLAY_EXCLUSIVE_ROWS[@]}"}; do
  replay_row_start "${ROW_TEXTS[row_sequence - 1]}" "$row_sequence" 0
  while (( running )); do replay_row_reap_finished; done
done
# 要卡的那一段（文件头「GPU」）：环境里原有的卡串交回去（没有的不设），按表序一行一行跑，跑完再收起。
#   弄坏开关 gpu-rows-not-serial：这一段几行一起起
if (( ${#REPLAY_GPU_ROWS[@]} )); then
  [[ -z "$replay_caller_cards_given" ]] || export GPU_CARDS_BY_PRIORITY="$REPLAY_CALLER_CARDS"
  for row_sequence in "${REPLAY_GPU_ROWS[@]}"; do
    replay_row_start "${ROW_TEXTS[row_sequence - 1]}" "$row_sequence" 0
    [[ "$REPLAY_BROKEN" == gpu-rows-not-serial ]] || while (( running )); do replay_row_reap_finished; done
  done
  while (( running )); do replay_row_reap_finished; done
  unset GPU_CARDS_BY_PRIORITY
fi
# 本脚本不交卡的行（文件头「GPU」「派行」）：GPU_CARDS_BY_PRIORITY 不设，按表序一行一行跑
for row_sequence in ${REPLAY_DRIVER_LEASED_ROWS[@]+"${REPLAY_DRIVER_LEASED_ROWS[@]}"}; do
  replay_row_start "${ROW_TEXTS[row_sequence - 1]}" "$row_sequence" 0
  while (( running )); do replay_row_reap_finished; done
done
# 逐个 wait 写死的 pid，不写不带参数的 wait（它的退出码恒为 0，红了几个一个字都不说）
for pid in ${REPLAY_PIDS[@]+"${REPLAY_PIDS[@]}"}; do wait "$pid" 2>/dev/null || true; done
replay_row_sampler_stop

# 派出去多少条就要收回来多少条：对不上整道红，不许少跑一条还报绿
collected=0
for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
  [[ -f "$OUT_DIR/$tag.line" ]] && collected=$((collected+1))
done
if (( collected != ${#REPLAY_ORDER[@]} )); then
  echo "  ✗ 派出去 ${#REPLAY_ORDER[@]} 条复跑，只收回 $collected 条结果行"
  echo "     → 怎么办：这是并发收束自己的完整性闸红了，不是实验的问题；REPLAY_JOBS=1 再跑一遍看串行下全不全，"
  echo "       全的话去查 replay_one 与它的 pid 收束那一段。"
  exit 1
fi

# 按登记表的顺序回读：输出与并发度无关，REPLAY_JOBS=1 与 =16 逐字相同
for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
  cat "$OUT_DIR/$tag.line"
  if [[ -f "$OUT_DIR/$tag.verdict" ]]; then
    case "$(cat "$OUT_DIR/$tag.verdict")" in
      pass) pass=$((pass+1)) ;; timing_only) timing_only=$((timing_only+1)) ;;
      drift) drift=$((drift+1)) ;; broken) broken=$((broken+1)) ;;
      unchanged) unchanged=$((unchanged+1)); UNCHANGED_ROWS+=("${tag%.*}") ;;
      reused:*) reused=$((reused+1)); REUSED_ROWS+=("${tag%.*}") ;;
      load_skipped) load_skipped=$((load_skipped+1)); LOAD_SKIPPED_ROWS+=("$(cat "$OUT_DIR/$tag.skip")") ;;
      not_run) not_run=$((not_run+1)); NOT_RUN_ROWS+=("${tag%.*}") ;;
    esac
  fi
  if [[ -f "$OUT_DIR/$tag.load" ]]; then   # 放行的计时行（judged 或 lagged）
    load_passed=$((load_passed+1)); if [[ "$(cat "$OUT_DIR/$tag.load")" == lagged ]]; then load_lagged=$((load_lagged+1)); fi
  fi
  [[ -f "$OUT_DIR/$tag.claim" ]] && CLAIM_QUEUE+=("$(cat "$OUT_DIR/$tag.claim")")
done
# E182 的片结果库并回服务（文件头「E182 的片结果库」）
replay_store_e182_shards
# 有一行判完记不进判定库是因为服务连不上（remember_row_verdict 记的标记，文件头「按行复用」）：整趟退 75，不判绿也不判红
if [[ -f "$OUT_DIR/prune-server-unavailable" ]]; then
  replay_prune_server_unavailable "python3 $ADMISSION_MODULE library-files store $REPLAY_ROW_LIBRARY（上面逐行点名了哪几条没记进去）"
fi

# 复用哨兵判：每一行的结局（复用来的、现判的、强制现判的）一次记好，哨兵被现判、强制现判那一行判定与库里不同，整道红
: >"$OUT_DIR/reuse-sentinel.outcomes"
for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
  row_verdict="$(cat -- "$OUT_DIR/$tag.verdict" 2>/dev/null)"
  case "$row_verdict" in
    reused:*) printf '%s\t复用\t%s\n' "$tag" "${row_verdict#reused:}" ;;
    "") ;;   # 没有结局的行不报结局：它没判，哨兵抽中它也照「没跑到」不判
    *) printf '%s\t%s\t%s\n' "$tag" "$([[ "$tag" == "$REPLAY_FORCED_TAG" ]] && echo 强制现判 || echo 现判)" "$row_verdict" ;;
  esac
done >>"$OUT_DIR/reuse-sentinel.outcomes"
sentinel_bad=0
if [[ -f "$REPLAY_SENTINEL" ]]; then
  python3 "$REPLAY_REUSE_CHECKS" sentinel-observe-many "$REPLAY_SENTINEL" "$OUT_DIR/reuse-sentinel.outcomes" || echo "  ! 复用哨兵没记下这一趟的结局（这一趟哨兵不判）" >&2
  python3 "$REPLAY_REUSE_CHECKS" sentinel-finish "$REPLAY_SENTINEL" || sentinel_bad=1
fi

printf '%s\n' "-------------------------------------------------------------------------"
echo "结论区间断言（计时实验复跑不出同样的字节，靠这些把 kb 里的数钉住）："
CLAIM_BAD_ROWS=()
for q in "${CLAIM_QUEUE[@]}"; do
  check_claims "${q%%|*}" "${q#*|}" || { claim_bad=$((claim_bad+1)); CLAIM_BAD_ROWS+=("${q%%|*}"); }
done
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」，.claude/hooks/run-scope-guard.sh 的拒绝说明读它）：一行一个红的实验号（对不上、跑不了、结论断言不中）；
# 点名了实验号的只换这几个，别的照留
{ for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
    case "$(cat -- "$OUT_DIR/$tag.verdict" 2>/dev/null)" in drift|broken) printf '%s\n' "${tag%.*}" ;; esac
  done
  printf '%s\n' ${CLAIM_BAD_ROWS[@]+"${CLAIM_BAD_ROWS[@]}"}; } | sort -u >"$OUT_DIR/last-run.red"
if [[ ! -f "$REPLAY_REUSE_CHECKS" ]]; then
  :   # 不带共用库的小树（上面复用哨兵那一段说过一句）
elif (( ${#ONLY[@]} )); then
  # --item 点到的实验号只判了其中几行：红了加进清单，绿了不从清单里划掉（清单里别的行的红还在）
  for replay_ran_experiment in "${ONLY[@]}"; do
    [[ -n "${ITEM_EXPERIMENT["$replay_ran_experiment"]+set}" ]] || printf '%s\n' "$replay_ran_experiment"
  done >"$OUT_DIR/last-run.ran"
  python3 "$REPLAY_REUSE_CHECKS" last-run scripts/experiment-apparatus/replay.sh "$OUT_DIR/last-run.red" --ran "$OUT_DIR/last-run.ran" >&2 || echo "  ! 上次红的清单没写成" >&2
else
  python3 "$REPLAY_REUSE_CHECKS" last-run scripts/experiment-apparatus/replay.sh "$OUT_DIR/last-run.red" >&2 || echo "  ! 上次红的清单没写成" >&2
fi
printf '%s\n' "-------------------------------------------------------------------------"
echo "字节一致 $pass ／ 仅计时不同 $timing_only ／ 对不上 $drift ／ 跑不了 $broken ／ 结论断言不中 $claim_bad ／ 输入没变没跑 $unchanged ／ 负载高本次不判 $load_skipped ／ 按行复用 $reused"
echo "REPLAY_TOTAL dispatched=${#REPLAY_ORDER[@]} collected=$collected judged=$((pass + timing_only + drift)) reused=$reused refused=$refused_rows pass=$pass timing_only=$timing_only drift=$drift broken=$broken claims_bad=$claim_bad unchanged=$unchanged load_skipped=$load_skipped not_run=$not_run"
if (( reused )); then
  echo "  · 按行复用 $reused 行（行键相同，判定库在 prune-server 里、库名 ${REPLAY_ROW_LIBRARY:-没有}，结论区间断言照常对记录里留的那一份产物判）：${REUSED_ROWS[*]}"
fi
echo "本轮输出：$OUT_DIR"
for build_log_name in build gpu-build; do   # 全绿又用自动分配的临时目录时下面会整个删掉它，不列
  [[ -f "$OUT_DIR/$build_log_name.problems.log" ]] && { [[ -n "${REPLAY_OUT:-}" ]] || (( drift + broken + claim_bad + sentinel_bad )); } && echo "  · 编译日志两份（先看错误警告那份）：$OUT_DIR/$build_log_name.log 全量；$OUT_DIR/$build_log_name.problems.log 错误与警告"
done
if (( load_passed + load_skipped )); then
  echo "  计时行负载关：判了 $load_passed、本次不判 $load_skipped、滞后 $load_lagged 份（滞后表 $TIMING_LAG_FILE，登记 ${#TIMING_LAG[@]} 份）"
fi
if [[ $load_skipped -ne 0 ]]; then
  echo "  ! 「本次不判」$load_skipped 行：新产物收尾行的负载超过阈值 $TIMING_LOAD_RATIO（REPLAY_TIMING_LOAD_RATIO），机器正忙，计时数不可信；这些行既不判绿也不判红，结构与区间断言都没比："
  printf '      %s\n' "${LOAD_SKIPPED_ROWS[@]}"   # gate-lint:detail
  echo "     → 怎么办：等负载降下来，只重跑这几行：bash scripts/experiment-apparatus/replay.sh <实验号>"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    for skipped_row in "${LOAD_SKIPPED_ROWS[@]}"; do
      printf 'replay.sh %s，负载过高本次不判（阈值 %s）\n' "$skipped_row" "$TIMING_LOAD_RATIO" >>"$GATE_NOT_RUN_FILE"
    done
  fi
fi
if (( not_run )); then
  echo "  ! 「本次未跑」$not_run 行：${NOT_RUN_ROWS[*]}。输入缺（导出根不在 replay-frozen-roots 下，REPLAY_DEFERRED_ROW_ROOT），里程碑四收口做全量时重建再跑；这几行既不判绿也不判红"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    for skipped_row in "${NOT_RUN_ROWS[@]}"; do
      printf 'replay.sh %s，输入缺本次未跑（里程碑四收口做全量时再跑）\n' "$skipped_row" >>"$GATE_NOT_RUN_FILE"
    done
  fi
fi
if [[ $unchanged -ne 0 ]]; then
  echo "  ! 「输入没变」$unchanged 行没跑：${UNCHANGED_ROWS[*]}。实验键（这一行的行键去掉期望产物：装置入口单元闭包、驱动闭包、参数、GPU 开关、工具链）"
  echo "     与留存产物头上记的相同，准入（scripts/gate/admission.py）拒绝重跑，结论沿用留存产物；结论区间断言照常对留存产物判。"
  echo "     这不是判绿也不是判红。输入没变不重跑，结论沿用留存产物；要重跑就先改输入（上面实验键里的那几样），键变了准入才放行。"
fi
if (( sentinel_bad )); then
  echo "  ✗ 复用哨兵判红（上面 ✗ 那几行：键相同却现判了，或强制现判那一行判定与库里不同）"
  echo "     → 怎么办：照上面的出路查 decide_row 读库那一段与 remember_row_verdict；判法在 scripts/gate/lib_reuse_checks.py 文件头「二」"
fi
if [[ $drift -ne 0 || $broken -ne 0 || $claim_bad -ne 0 ]]; then
  echo "  → 怎么办：「跑不了」看上面那一行给的 $OUT_DIR/<实验号>.err；「对不上」按上面给的 diff 命令看差在哪，" \
       "结构性差异是代码改动带来的就更新入库产物，不是就说明代码退化了；" \
       "「结论断言不中」逐条去 check_claims() 里对应实验号那一段读注释——是该改 kb 里记的区间，还是真的退化了，" \
       "两种都要人判，不许为了让这里变绿就调宽容差（test-discipline.md）。"
fi
if [[ $drift -eq 0 && $broken -eq 0 && $claim_bad -eq 0 && $sentinel_bad -eq 0 ]]; then
  # 全绿且用的是自动分配的临时目录 ⇒ 收拾掉。有一条不绿就留着，上面的提示指着它。
  [[ -z "${REPLAY_OUT:-}" ]] && rm -rf "$OUT_DIR"
  if (( (load_skipped || not_run) && pass + timing_only + unchanged + reused == 0 )); then
    echo "  ! 判的全是本次不判或本次未跑（负载过高 $load_skipped 行、输入缺 $not_run 行），一行都没比：不算通过，退 77"
    exit 77
  fi
  echo "  ✓ 复跑现判 $((pass + timing_only)) 行都对得上（字节一致 $pass、仅计时不同 $timing_only）、复用 $reused 行、行键拒算照跑 $refused_rows 行、负载高本次不判 $load_skipped 行、输入没变没跑 $unchanged 行，结论断言全中；负载关：判了 $load_passed、滞后 $load_lagged 份"
  exit 0
fi
exit 1
