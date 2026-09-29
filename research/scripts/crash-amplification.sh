#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上跑全量或点名的分项；哪些块要重判由库里的块键与判法版本定，不在这里
# run-condition: command bash cargo python3
# 崩溃放量全量（用户 2026-09-29 定：其他的崩溃放量放在一个脚本里做、作为分项；以后全量就是一起跑，分项用参数跑；不指定参数就作为全量跑）：
#
#   crash-amplification.sh                                  全量：登记表里每个分项按次序跑一遍
#   crash-amplification.sh --item <分项>… [--item <分项>]…    只跑点名的分项（分项名照 --list-items 列的）
#   crash-amplification.sh --list-items                     只列分项，不跑
#   crash-amplification.sh --selftest                       用假的分项驱动走一遍（列项、点名、拒绝不认得的名、失败计数）
#   其余参数原样交给每一项的分项驱动：[--expand-up-to <N>] [--threads <N>] [--memory-cap <上限>] [--peer-memory-cap <上限>] [--library <库目录>]
#
# 分项登记在 ITEMS：一行「分项名 <TAB> 测试目标 <TAB> 用例函数」，一个分项就是 crates/singlefs-checker-tier/tests/ 下一条走
# crash_amplification::run_from_environment 的用例（用例函数经分项驱动的 --test-filter 点名，cargo test 的 --exact）。
# 崩溃注入（crash_injection_campaign）不在表里：它是随机历史上的抽样、没有固定的点序列与树，是轻测，归普通 cargo test。
# 一份库装全部分项（默认 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library，在 /tmp 之外）：
# 几条流共前缀，同一份库里前缀节点只判一次；重跑同一条命令就是续跑——同一判法版本下判过的块全部复用，判法版本变了旧行留作历史、这一版从头判。
# 每项交 research/scripts/crash-amplification-item.sh 跑：配置几台就跑几台、配置开了 GPU 就由 GPU 判，做不到那一项报错
# （用户 2026-09-29 定：「配置中一台机器就要跑一台，配置中两台机器就要跑两台，配置中有gpu就一定要让GPU工作 如果不行就明确报错拒绝，不接受默认降级」）。
# 清场（用户 2026-09-29 定「每次一定全量要清场」）：开跑之前照多机配置里的清场命令清一次（清不成退 2、一项都不跑），
# 每一项都带 --quiesced 跑（显卡用清场之后那一份先后与额度），整趟退出时复原一次（被打断也复原）。
#
# 输出：每项一行 CRASH_AMPLIFICATION_ITEM item=<分项> exit=<驱动退出码> states=<Σ states_checked_here> red=<Σ red> lines=<那一趟 CRASH_AMPLIFICATION 行数> seconds=<这一项的挂钟>；
#       末尾一行 CRASH_AMPLIFICATION_TOTAL items=<跑了几项> failed=<几项退非 0> states=<Σ> red=<Σ> seconds=<挂钟>；每项的驱动输出原样转打在它那一行之前。
# 退出码：0 每项都退 0 且 0 红；1 有分项退非 0 或有红；2 用法错（分项名不认得、登记表坏、驱动不在）。
# 弄坏开关 CRASH_AMPLIFICATION_BREAK=<项>（只给 --selftest 证红用）：swallow-exit 分项退非 0 也记 0（「失败计数」判错）、
#   skip-filter 调驱动时不带 --test-filter（「每项只跑点名的用例」判错）、accept-unknown-item 不认得的分项名照跑（「拒绝不认得的名」判错）、
#   no-quiesce 开跑前不清场（「每次开跑都清场、每项带 --quiesced」判错）、
#   no-restore 退出时不复原（「清过场退出时复原」判错）。
# 只供测试：CRASH_AMPLIFICATION_ITEM_DRIVER=<脚本> 换掉分项驱动（自证用假的）；
#   CRASH_AMPLIFICATION_QUIESCE_STUB=<脚本> 换掉清场与复原（调法：<脚本> stop、<脚本> start）。
#
# gate-similar: crash-amplification-item.sh 它跑一个分项的一趟（排派活计划、定在哪跑）；这里是分项登记表与全量的循环，每项调它
# gate-similar: crash-amplification-two-hosts.sh 它跑一条用例的双机一趟，由分项驱动在要动用第二台时调；这里不直接调它
# gate-similar: layer0-shard-run.sh 它跑层 0 登记的崩溃枚举用例（CPU 枚举、54 号全绿标记）；这里跑的是走判定库的流水线用例
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

crash_amplification_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd "$crash_amplification_script_directory/../.." && pwd)"

# 分项登记表：分项名 <TAB> 测试目标 <TAB> 用例函数（次序就是全量的跑序：先短的流，长的流复用它们的前缀）
ITEMS=(
  $'fixed-script-through-rollback\tcrash_amplification_fixed_script_through_rollback\tthe_fixed_script_through_the_rollback_is_recorded_checked_in_parallel_and_reused'
  $'fixed-script-through-unmount\tcrash_amplification_fixed_script_through_unmount\tthe_whole_fixed_script_through_the_unmount_is_recorded_checked_in_parallel_and_reused'
  $'multi-record-publish\tcrash_enumeration_multi_record_publish_stream\tcrash_amplification_of_the_multi_record_publish_stream_is_clean'
  $'tree-split-streams\tcrash_enumeration_tree_split_streams\tcrash_amplification_of_every_tree_split_stream_is_clean'
  $'position-addressed-trees\tcrash_enumeration_position_addressed_trees\tcrash_amplification_of_every_position_addressed_tree_stream_is_clean'
  $'floor-raise-pushed-by-the-session\tcrash_enumeration_floor_raise_pushed_by_admission\tcrash_amplification_of_the_floor_raise_pushed_by_the_session_is_clean'
  $'sigma-of-the-misaligned-reuse\trecord_checker_judges_absence_by_the_persisted_set\tcrash_amplification_of_sigma_on_the_misaligned_reuse_history_is_clean'
)

crash_amplification_break_is() { [[ ",${CRASH_AMPLIFICATION_BREAK:-}," == *",$1,"* ]]; }

usage() {
  echo "  ✗ 用法：$0 [--item <分项>]… [--list-items] [--selftest] [驱动的参数…]" >&2
  echo "  → 怎么办：不带参数就是全量；分项名照 --list-items 列的写" >&2
  exit 2
}

item_fields() { # <登记行> <字段号 1..3>
  printf '%s' "$1" | cut -f"$2"
}

list_items() {
  local row
  for row in "${ITEMS[@]}"; do
    printf '%s\t%s\t%s\n' "$(item_fields "$row" 1)" "$(item_fields "$row" 2)" "$(item_fields "$row" 3)"
  done
}

selected=(); driver_arguments=(); list_only=0; selftest=0
while (($# > 0)); do
  case "$1" in
    --item) selected+=("${2:?--item 要一个分项名}"); shift 2 ;;
    --list-items) list_only=1; shift ;;
    --selftest) selftest=1; shift ;;
    --expand-up-to|--threads|--memory-cap|--peer-memory-cap|--library|--label) driver_arguments+=("$1" "${2:?$1 要一个值}"); shift 2 ;;
    *) usage ;;
  esac
done

if (( list_only )); then list_items; exit 0; fi

# 登记表自检：三段都非空、分项名不重
declare -A seen=()
for row in "${ITEMS[@]}"; do
  name="$(item_fields "$row" 1)"; target="$(item_fields "$row" 2)"; function_name="$(item_fields "$row" 3)"
  if [[ -z "$name" || -z "$target" || -z "$function_name" ]]; then echo "  ✗ 登记表有一行不全：$row" >&2; echo "  → 怎么办：三段都填上（分项名、测试目标、用例函数）" >&2; exit 2; fi
  if [[ -n "${seen[$name]:-}" ]]; then echo "  ✗ 分项名重了：$name" >&2; echo "  → 怎么办：分项名一个只登一次" >&2; exit 2; fi
  seen["$name"]=1
done

run_selftest() {
  local work stub log
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-amplification-selftest.XXXXXX")"
  trap 'rm -rf -- "${work:?}"' EXIT
  stub="$work/driver.sh"
  cat >"$stub" <<'STUB'
#!/usr/bin/env bash
# 假的分项驱动：记下参数，打一行 CRASH_AMPLIFICATION；测试目标名里带 red 的退 1（红数照 0：失败要靠退出码认出来）
printf 'driver %s\n' "$*" >> "${STUB_LOG:?}"
target="$1"
if [[ "$target" == *red* ]]; then echo "CRASH_AMPLIFICATION mode=single_host_cpu states_checked_here=5 red=0"; exit 1; fi
echo "CRASH_AMPLIFICATION mode=single_host_cpu states_checked_here=7 red=0"
exit 0
STUB
  chmod +x "$stub"
  # 假的清场与复原：记一笔；QUIESCE_FAILS=1 时清场失败
  cat >"$work/quiesce.sh" <<'STUB'
#!/usr/bin/env bash
printf 'quiesce %s\n' "$1" >> "${QUIESCE_LOG:?}"
if [[ "$1" == stop && "${QUIESCE_FAILS:-0}" == 1 ]]; then exit 1; fi
exit 0
STUB
  log="$work/calls.log"
  export CRASH_AMPLIFICATION_QUIESCE_STUB="$work/quiesce.sh" QUIESCE_LOG="$work/quiesce.log"
  local failures=0 cells=0
  selftest_cell_failed() { # <这一格判错的一句> <出路>
    echo "  ✗ 自证：$1" >&2
    echo "     → 怎么办：$2" >&2
    failures=$((failures + 1))
  }
  # ① --list-items 列出登记表的每一行，不清场
  cells=$((cells + 1)); : > "$QUIESCE_LOG"
  if [[ "$(bash "$0" --list-items | wc -l)" != "${#ITEMS[@]}" ]] || [[ -s "$QUIESCE_LOG" ]]; then
    selftest_cell_failed "--list-items 的行数不是登记表的行数，或它清了场" "看 list_items 是不是逐行打 ITEMS、在清场之前就退"
  fi
  # ② 不认得的分项名拒、退 2
  cells=$((cells + 1))
  if CRASH_AMPLIFICATION_ITEM_DRIVER="$stub" STUB_LOG="$log" bash "$0" --item no-such-item >/dev/null 2>&1; then
    selftest_cell_failed "不认得的分项名没拒" "看选分项那一段：登记表里找不到的名字要退 2"
  fi
  # ③ 全量：驱动被调登记表那么多次，每次带 --test-filter <用例函数>
  cells=$((cells + 1))
  : > "$log"
  CRASH_AMPLIFICATION_ITEM_DRIVER="$stub" STUB_LOG="$log" bash "$0" >/dev/null 2>&1; local full_exit=$?
  if [[ "$full_exit" != 0 ]]; then
    selftest_cell_failed "全量全绿却退 $full_exit" "看每项退出码与红数怎么汇总进 items_failed"
  fi
  if [[ "$(wc -l <"$log")" != "${#ITEMS[@]}" ]] || [[ "$(grep -c -- '--test-filter' "$log")" != "${#ITEMS[@]}" ]]; then
    selftest_cell_failed "全量没把每个分项各调一次驱动并带 --test-filter（调了 $(wc -l <"$log") 次）" "看主循环：不带参数要跑 ITEMS 的每一行，filter_arguments 要带 --test-filter"
  fi
  # ④ 点名一项：只调一次、目标对
  cells=$((cells + 1))
  : > "$log"
  CRASH_AMPLIFICATION_ITEM_DRIVER="$stub" STUB_LOG="$log" bash "$0" --item tree-split-streams >/dev/null 2>&1
  if [[ "$(wc -l <"$log")" != 1 ]] || ! grep -q 'crash_enumeration_tree_split_streams' "$log"; then
    selftest_cell_failed "--item 没有只跑点名的那一项" "看选分项那一段：点名的分项只取登记表里同名那一行"
  fi
  # ⑤ 清场：开跑前清一次、每一项都带 --quiesced、退出时复原一次（点名的分项也一样）
  cells=$((cells + 1))
  : > "$log"; QUIESCE_LOG="$log"
  local quiesce_output quiesce_exit
  quiesce_output="$(CRASH_AMPLIFICATION_ITEM_DRIVER="$stub" STUB_LOG="$log" QUIESCE_LOG="$log" \
    bash "$0" --item fixed-script-through-rollback --item multi-record-publish --item tree-split-streams 2>/dev/null)"; quiesce_exit=$?
  local expected_calls="quiesce-stop driver-quiesced driver-quiesced driver-quiesced quiesce-start "
  local actual_calls
  actual_calls="$(sed -E 's/^driver .* --quiesced( .*)?$/driver-quiesced/; s/^driver .*$/driver-plain/; s/^quiesce (.*)$/quiesce-\1/' "$log" | tr '\n' ' ')"
  if [[ "$quiesce_exit" != 0 || "$actual_calls" != "$expected_calls" ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL items=3 failed=0 states=21 ' <<<"$quiesce_output"; then
    selftest_cell_failed "没有照「开跑前清一次、每项带 --quiesced、退出时复原一次」走（退 $quiesce_exit，调用次序「$actual_calls」）" "看主循环之前的 quiesce_once、quiesced_arguments 与退出时的 restore_if_quiesced"
  fi
  QUIESCE_LOG="$work/quiesce.log"
  # ⑥ 清场失败：退 2，一项都不跑
  cells=$((cells + 1))
  : > "$log"
  CRASH_AMPLIFICATION_ITEM_DRIVER="$stub" STUB_LOG="$log" QUIESCE_FAILS=1 bash "$0" --item tree-split-streams >/dev/null 2>&1; local failed_quiesce_exit=$?
  if [[ "$failed_quiesce_exit" != 2 ]] || [[ -s "$log" ]]; then
    selftest_cell_failed "清场失败却没退 2、或照跑了分项（退 $failed_quiesce_exit）" "看主循环之前 quiesce_once 失败那一支：要退 2，不跑任何分项"
  fi
  # ⑦ 一项退非 0：TOTAL 记 failed=1、整体退 1
  cells=$((cells + 1))
  local red_output
  red_output="$(CRASH_AMPLIFICATION_ITEM_DRIVER="$stub" STUB_LOG="$log" CRASH_AMPLIFICATION_SELFTEST_RED_ITEM=1 bash "$0" --item tree-split-streams 2>/dev/null)"; local red_exit=$?
  if [[ "$red_exit" != 1 ]] || ! grep -q 'CRASH_AMPLIFICATION_TOTAL items=1 failed=1 ' <<<"$red_output"; then
    selftest_cell_failed "分项退非 0 没记成失败（退 $red_exit）：$red_output" "看主循环里 item_exit 非 0 时 items_failed 有没有加一、末尾退 1"
  fi
  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：列项不清场、拒不认得的名、全量逐项带 --test-filter、点名只跑一项、开跑前清场每项带 --quiesced 退出复原、清场失败一项都不跑、失败计数，$cells 格都对"
  exit 0
}
if (( selftest )); then run_selftest; fi

driver="${CRASH_AMPLIFICATION_ITEM_DRIVER:-$crash_amplification_script_directory/crash-amplification-item.sh}"
[[ -f "$driver" ]] || { echo "  ✗ 分项驱动不在：$driver" >&2; echo "  → 怎么办：research/scripts/crash-amplification-item.sh 要在；自证用 CRASH_AMPLIFICATION_ITEM_DRIVER 指假的" >&2; exit 2; }

# 选分项：没点名就是全量；点名的每个都要在登记表里
rows_to_run=()
if (( ${#selected[@]} == 0 )); then
  rows_to_run=("${ITEMS[@]}")
else
  for name in "${selected[@]}"; do
    found=""
    for row in "${ITEMS[@]}"; do [[ "$(item_fields "$row" 1)" == "$name" ]] && found="$row"; done
    if [[ -z "$found" ]]; then
      if crash_amplification_break_is accept-unknown-item; then found=$'\t'"$name"$'\t'"$name"; else
        echo "  ✗ 不认得的分项：$name" >&2; echo "  → 怎么办：分项名照 --list-items 列的写" >&2; exit 2
      fi
    fi
    rows_to_run+=("$found")
  done
fi

# 清场与复原：开跑前清一次、退出时复原一次。
quiesced_now=0; quiesced_arguments=()
restore_if_quiesced() {
  (( quiesced_now )) || return 0
  crash_amplification_break_is no-restore && return 0
  if [[ -n "${CRASH_AMPLIFICATION_QUIESCE_STUB:-}" ]]; then bash "$CRASH_AMPLIFICATION_QUIESCE_STUB" start; else peer_host_restore; fi
  quiesced_now=0
}
quiesce_once() { # 清过了就不再清；清不成打原因与出路、交回非 0
  (( quiesced_now )) && return 0
  if [[ -n "${CRASH_AMPLIFICATION_QUIESCE_STUB:-}" ]]; then
    bash "$CRASH_AMPLIFICATION_QUIESCE_STUB" stop || return 1
  else
    source "$crash_amplification_script_directory/peer-host-lib.sh"
    if ! peer_host_load_configuration "$crash_amplification_script_directory/layer0-shard-configuration-check.sh" "$repository_root"; then
      echo "  ✗ 多机配置判不过（上面是判法的原因），清不了场" >&2
      echo "  → 怎么办：照判法打出的出路改 multi-host.env" >&2
      return 1
    fi
    if ! peer_host_quiesce; then
      echo "  ✗ 清场没过（回读没通过）" >&2
      echo "  → 怎么办：看配置里 QUIESCE_STOP_COMMAND 与 QUIESCE_STOPPED_CHECK_COMMAND 跑不跑得通；上一趟清场之后没复原完的，先跑配置里的复原命令" >&2
      return 1
    fi
  fi
  quiesced_now=1; quiesced_arguments=(--quiesced)
  trap 'restore_if_quiesced' EXIT
  echo "  · 清了场：每一项带 --quiesced 跑，退出时复原"
}
if ! crash_amplification_break_is no-quiesce; then
  quiesce_once || { echo "  ✗ 开跑前清场没清成，一项都不跑" >&2; echo "  → 怎么办：照上面清场那几行的出路修好再跑" >&2; exit 2; }
fi

started_at=$(date +%s)
items_run=0; items_failed=0; total_states=0; total_red=0
for row in "${rows_to_run[@]}"; do
  name="$(item_fields "$row" 1)"; target="$(item_fields "$row" 2)"; function_name="$(item_fields "$row" 3)"
  # 自证用：把点名那一项的目标改成带 red 的名字，让假驱动报红退 1
  [[ -n "${CRASH_AMPLIFICATION_SELFTEST_RED_ITEM:-}" ]] && target="${target}-red"
  filter_arguments=(--test-filter "$function_name")
  crash_amplification_break_is skip-filter && filter_arguments=()
  echo "== 分项 $name（$target::$function_name）$(date -u +%FT%TZ)"
  item_started_at=$(date +%s)
  item_output="$(bash "$driver" "$target" "${filter_arguments[@]+"${filter_arguments[@]}"}" "${driver_arguments[@]+"${driver_arguments[@]}"}" "${quiesced_arguments[@]+"${quiesced_arguments[@]}"}" 2>&1)"; item_exit=$?
  printf '%s\n' "$item_output"
  crash_amplification_break_is swallow-exit && item_exit=0
  item_lines="$(grep -c '^CRASH_AMPLIFICATION ' <<<"$item_output" || true)"
  item_states="$(grep -h '^CRASH_AMPLIFICATION ' <<<"$item_output" | grep -o 'states_checked_here=[0-9]*' | cut -d= -f2 | awk '{s+=$1} END {print s+0}')"
  item_red="$(grep -h '^CRASH_AMPLIFICATION ' <<<"$item_output" | grep -o ' red=[0-9]*' | cut -d= -f2 | awk '{s+=$1} END {print s+0}')"
  echo "CRASH_AMPLIFICATION_ITEM item=$name exit=$item_exit states=$item_states red=$item_red lines=$item_lines seconds=$(( $(date +%s) - item_started_at ))"
  items_run=$((items_run + 1))
  if [[ "$item_exit" != 0 ]] || (( item_red > 0 )); then items_failed=$((items_failed + 1)); fi
  total_states=$((total_states + item_states)); total_red=$((total_red + item_red))
done
echo "CRASH_AMPLIFICATION_TOTAL items=$items_run failed=$items_failed states=$total_states red=$total_red seconds=$(( $(date +%s) - started_at ))"
if (( items_failed > 0 )); then
  echo "  ✗ $items_failed 个分项没过（退非 0 或有红）" >&2
  echo "  → 怎么办：看上面那一项驱动的输出与它日志目录里的 placement.log / run.log（动用了第二台的是 share-0.log / share-1.log / import.log）；修好之后只重跑那一项：$0 --item <分项>" >&2
  exit 1
fi
exit 0
