#!/usr/bin/env bash
# E206（发布固定点轮数的代价）量的那一路的入口：给仓副本套上读数与臂，起历史用例，产物按 <点>__<臂> 分文件写进 research/results/，每项的判定照单元键进 prune-server。
# 跑前登记 research/prompts/e206-preregistration.md 第五节「判定入库与按点单跑」、第十四节。
#
# 用法：
#   e206-fixed-point-cost-run.sh --list-items
#   e206-fixed-point-cost-run.sh --work <共享根下的工作目录> [--segment first|second|third | --item <点>__<臂>…] [--source <仓根>]
#       不带范围参数是全部登记的项（不指定范围）；--item 给几次取并集，项名写错退 2 并列可点的项。
#   e206-fixed-point-cost-run.sh --selftest        核入口自己的列项、拒绝、完成标记、续跑、按键复用与复用哨兵，用假驱动，不编译、不跑实现；
#                                                  只在临时 prune-server 上跑（开跑先经判定库那一份调客户端 selftest_service_problem，不是临时服务退 2、一格不跑）
# 判定库（.claude/rules/verification.md「判定库只有一份」；键、库与哨兵的写法在同目录 e206_fixed_point_cost_library.py 文件头）：
#   每个 <点>__<臂> 一条判定，键是副本里用例 item_<点>__<臂> 的单元闭包键加工具链行、闭包真引到的依赖行、臂与跑法；副本路径不进键。族 apparatus-items，种类 e206-fixed-point-cost-run。
#   research/results/ 里的产物不当判定读，也不导入：库里没有的项照键现判一次。
# 一趟只编译一次：一份副本一棵 cargo 目录（服务给的 CARGO_TARGET_DIR），同一条臂的项在同一个进程里按登记线程并行；臂由环境变量 E206_ARM 在进程开头读一次，所以一条臂一个 cargo test。
# 同一条命令重起就是续跑：副本照计数脚本的开拷、完成标记；库里有键相同、做完了的项不重跑；上一次尝试的原始输出（副本里 .e206-raw 下，旁边 .keys 记着起 cargo 时的键）里
#   已经跑完、键又相同的现判项直接写成产物、照键入库；产物先写同目录隐藏 .partial 再换名；做完而没有 item_done 的产物不删不覆盖，重起时另写新文件名。
# 进度：历史那一段每过 5 分钟一行 `e206 progress item=<项> publishes_done=<n> of=<总数> peak_rounds=<r> peak_W=<w>`，转一份到标准输出；判结果只读原始输出。不作计时用。
#
# run-condition: 无（不要 GPU、不要第二台）。
# 退出码：0 跑到的全绿；1 有项没跑出 item_done、复用哨兵红、或计划里要现判的项这一趟没记；2 用法错、项名写错或工作目录里有不是本脚本拷的东西；
#   3 prune-server 客户端别的错，或锚点不是恰好命中一次（登记停机条款第 1 条）；75 prune-server 不可用（照客户端：不现判、不当「库里没有」）；
#   77 一项都没跑到；78 准入条件不满足（仓根或前置脚本不在）。
# 弄坏开关（只给 --selftest 用，平常跑时不认）。逐项写：开关、弄坏哪一处、打开后哪几格该红（失败行句首 name=selftest_case case=<格名> result=FAILED）：
#   E206_RUN_BREAK=done-marker-ignored  完成标记不看（凡判「做完没有」的地方都当做完了）
#       done_marker
#   E206_RUN_BREAK=done-items-rerun  计划复用的项照样重跑（不看 plan 给的 decision）
#       a_finished_item_is_not_run_again
#   E206_RUN_BREAK=harvest-off  不从上一次尝试的原始输出里取跑完的项
#       an_item_finished_inside_a_killed_run_is_taken_and_not_run_again
#   E206_RUN_BREAK=harvest-ignores-the-key  从原始输出取项时不核旁边 .keys 记的键
#       an_item_finished_inside_a_killed_run_under_another_key_is_run_again
#   E206_RUN_BREAK=cut-lines-kept  取项时不丢最后一行不带换行的残行
#       a_line_cut_by_a_kill_does_not_finish_an_item
#   E206_RUN_BREAK=product-written-in-place  产物直接写在产物名下（被杀时留下半份）
#       a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name
#   E206_RUN_BREAK=filter-by-substring  点名不带 --exact，item_files_1__p1 会连带跑 item_files_100__p1 等
#       the_item_filter_names_one_test_only
#   E206_RUN_BREAK=copy-resume-off  副本不认完成标记与开拷标记，非空一律拒（在计数脚本里读，入口只是把环境带过去）
#       killed_after_the_copy_and_run_again_completes
#   E206_RUN_BREAK=progress-not-forwarded  进度行不转到标准输出
#       progress_lines_reach_the_standard_output
#   E206_RUN_BREAK=one-process-for-every-arm  几条臂的项并进一个 cargo test（E206_ARM 只有头一条臂的值）
#       each_arm_runs_in_its_own_process
#   下面几项在判定库那一份里读（e206_fixed_point_cost_library.py 文件头「弄坏开关」，入口只是把环境带过去）：
#   E206_RUN_BREAK=library-lookup-ignored  决定复用那一路不读库
#       a_finished_item_is_not_run_again、reuse-three-questions
#   E206_RUN_BREAK=record-skipped  跑完的项不往库里写
#       a_finished_item_is_not_run_again、reuse-three-questions
#   E206_RUN_BREAK=key-with-copy-path  请求另加副本的绝对路径
#       the_item_key_does_not_depend_on_the_copy_path
#   E206_RUN_BREAK=key-by-whole-histories-file  整份历史用例进键
#       reuse-three-questions
#   E206_RUN_BREAK=sentinel-skipped  不抽哨兵与强制现判、不判
#       reuse_sentinel_draws_and_the_forced_item_is_run_again、a_rejudged_sentinel_turns_the_run_red
#   E206_RUN_BREAK=sentinel-rejudge-unobserved  跑完把哨兵一律记成复用
#       a_rejudged_sentinel_turns_the_run_red
#   E206_RUN_BREAK=forced-read-as-reused  抽中强制现判的项照库复用
#       reuse_sentinel_draws_and_the_forced_item_is_run_again
#   E206_RUN_BREAK=forced-verdict-unchecked  不比强制现判的判定与库里那一份
#       a_forced_rejudge_with_another_verdict_turns_the_run_red
#   E206_RUN_BREAK=refused-items-reused  拒算的项照复用跳过、不跑
#       an_item_without_a_key_runs_and_is_never_recorded
#   E206_RUN_BREAK=unreachable-as-empty  读库时 prune-server 连不上照「库里没有」现判
#       an_unreachable_library_exits_75_and_runs_nothing
#   E206_RUN_BREAK=unrun-items-unchecked  跑完不核计划里要现判的项这一趟记过没有
#       an_item_planned_for_judging_but_not_run_turns_the_run_red
set -u

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
counters="$here/e206_fixed_point_cost_counters.py"
library="$here/e206_fixed_point_cost_library.py"
histories="$here/e206_fixed_point_cost_histories.rs"
registered_threads_file="$here/../../.claude/heavy-thread-default"
entry_script_key="scripts/experiment-apparatus/e206-fixed-point-cost-run.sh"   # 复用三问规范输出认的脚本名

# 项 段（登记第五节「取样点」「实现量与分段」；与历史用例的 item_ 名单同一份，自证 the_table_matches_the_histories 核）
items_table() {
  local point arm
  for point in base files_100 files_1000 files_10000 data_units_100 data_units_1000 data_units_10000 unlinks_100 unlinks_1000 unlinks_10000 \
    files_30000 files_1000_units_10 files_1000_units_100 files_1000_unit_area_2_pow_14 files_1000_unit_area_2_pow_20 files_1000_unit_area_2_pow_24; do
    for arm in p1 p2a; do echo "${point}__$arm first"; done
  done
  for point in base files_100 files_1000; do
    for arm in single_step closure_off; do echo "${point}__$arm first"; done
  done
  echo "base__plain first"
  echo "files_1000__plain first"
  for point in diagonal files_then_big_1000 files_then_big_10000 churn_random_1000 churn_random_10000 hot_then_cold_1000 hot_then_cold_10000 churn_half_deleted \
    alternate_create_unlink snapshots_1000 remount_every_1000 batch_of_one_files_10000; do
    for arm in p1 p2a; do echo "${point}__$arm second"; done
  done
  for point in files_10000 data_units_10000 unlinks_10000; do
    for arm in p2b16 p2b256 whole; do echo "${point}__$arm second"; done
  done
  echo "base__p2b_never second"
  echo "files_100__p2b_never second"
  echo "files_1000__whole second"
  for point in data_units_100000 data_units_1000000 unlinks_100000 files_100000 user_scale_many_rounds; do
    for arm in p1 p2a; do echo "${point}__$arm third"; done
  done
}

usage() { sed -n '2,/^set -u/p' "$0" | sed '$d' >&2; exit 2; }

broken() { [ "${E206_RUN_BREAK:-}" = "$1" ]; }

# 一份产物是否做完：有 item_done 行。弄坏开关打开时不看。
product_is_done() {
  broken done-marker-ignored && return 0
  grep -q '^E206RESULT name=item_done ' "$1" 2>/dev/null
}

histories_key() {
  sha256sum "$histories" | cut -c1-16
}

# 新文件名：e206-fixed-point-cost-<点>-<臂>-<日期>.out，同一天再来加 -r2、-r3；已有的一个不动。
new_product_name() {
  local stem day name n=1
  stem="${1/__/-}"
  day=$(TZ=Asia/Tokyo date +%F)
  name="$repo/research/results/e206-fixed-point-cost-$stem-$day.out"
  while [ -e "$name" ]; do
    n=$((n + 1))
    name="$repo/research/results/e206-fixed-point-cost-$stem-$day-r$n.out"
  done
  printf '%s\n' "$name"
}

complete_lines() {
  if broken cut-lines-kept; then cat "$1"; else head -n "$(wc -l < "$1")" "$1"; fi
}

raw_has_finished_item() {
  complete_lines "$1" | grep -qxF "E206RESULT name=item_done item=$2"
}

raw_keyed_for_the_item() {
  local raw=$1 item=$2 key=$3
  broken harvest-ignores-the-key && return 0
  [ "$key" != - ] && grep -qxF "$(printf '%s\t%s' "$item" "$key")" "$raw.keys" 2>/dev/null
}

write_raw_keys() {
  local raw=$1 item
  shift
  for item in "$@"; do printf '%s\t%s\n' "$item" "${item_key[$item]}"; done > "$raw.keys.partial" && mv -f "$raw.keys.partial" "$raw.keys"
}

# 同一天内容相同的 diff 引用已有那份，没有才写新名。交回产物目录里的路径。
publish_diff() {
  local text=$1 existing day name n=1 temporary
  for existing in "$repo"/research/results/e206-fixed-point-cost-counters-diff-*.diff; do
    if [ -f "$existing" ] && cmp -s "$text" "$existing"; then printf '%s\n' "$existing"; return 0; fi
  done
  day=$(TZ=Asia/Tokyo date +%F)
  name="$repo/research/results/e206-fixed-point-cost-counters-diff-$day.diff"
  while [ -e "$name" ]; do n=$((n + 1)); name="$repo/research/results/e206-fixed-point-cost-counters-diff-$day-r$n.diff"; done
  temporary="$(dirname "$name")/.$(basename "$name").partial"
  cp "$text" "$temporary" && mv -f "$temporary" "$name" || return 1
  printf '%s\n' "$name"
}

# 产物头：构建、线程、历史用例的内容键、副本的 diff、这一项的键、主工作区三份被测文件的 sha256（停机条款第 3 条）与锚点行号。字段名不写 item=，判定只取带 item= 的行。
header_for() {
  local item=$1 file
  echo "E206RESULT name=header for_item=$item build=$kind profile=release opt_level=3 test_threads=$threads histories=$(histories_key) diff_file=$(basename "$diff_file") diff_lines=$(wc -l < "$diff_file") item_key=${item_key[$item]:--}"
  for file in transaction.rs allocation_record_tree.rs allocator.rs; do
    echo "E206RESULT name=source_sha256 file=crates/singlefs-core/src/$file sha256=$(sha256sum "$source_tree/crates/singlefs-core/src/$file" 2>/dev/null | cut -d' ' -f1)"
  done
  python3 -I "$counters" --check-anchors "$source_tree" | sed 's/^/E206RESULT /'
}

# 一项跑完（或没跑出来）交判定库那一份记下。没记成的报出来、这一项下一趟照键现判；退 75、3 留给末尾照它退。
record_item() {
  local item=$1 code
  shift
  python3 -I "$library" record --plan "$plan_file" --item "$item" "$@"
  code=$?
  [ "$code" -eq 0 ] && return 0
  echo "  ✗ 判定库那一份没记下 $item（退 $code）：产物已写，这一项下一趟照键现判"
  echo "     → 怎么办：退 75 是 prune-server 不可用（python3 prune-server/control check，修好之后整条命令重跑，库里有的项照键复用）；别的退出码照它上面那几行 ✗ 的出路改"
  case "$code" in 3|75) library_trouble=$code ;; esac
  return 1
}

# 把一份原始输出里这一项的行写成产物：先写同目录隐藏 .partial，写完换名；回读有 item_done 才交判定库记下。
write_product() {
  local item=$1 raw=$2 seconds=$3 product temporary view
  product=$(new_product_name "$item")
  temporary="$(dirname "$product")/.$(basename "$product").partial"
  broken product-written-in-place && temporary=$product
  view="$raw_dir/view.tmp"
  complete_lines "$raw" > "$view"
  { header_for "$item"; grep -E "^E206RESULT .* item=$item( |$)" "$view"; } > "$temporary"
  [ "$temporary" = "$product" ] || mv -f "$temporary" "$product" || return 1
  if ! product_is_done "$product"; then
    echo "name=item_failed item=$item"
    echo "name=item_line item=$item exit=1 judged=1 red=1 reused=0 seconds=$seconds"
    record_item "$item" --failed
    return 1
  fi
  echo "name=item_line item=$item exit=0 judged=1 red=0 reused=0 seconds=$seconds"
  record_item "$item" --product "$product"
}

# 上一次尝试（被杀）留下的原始输出里已经完整跑完、键与此刻相同的现判项：直接写成产物、入库，从 todo 里拿掉。
harvest_finished_items() {
  local item raw kept=() taken
  broken harvest-off && return 0
  for item in "${todo[@]}"; do
    taken=0
    if [ "${item_decision[$item]}" = judge ]; then
      for raw in "$raw_dir"/*.out; do
        [ -f "$raw" ] || continue
        raw_keyed_for_the_item "$raw" "$item" "${item_key[$item]}" || continue
        if raw_has_finished_item "$raw" "$item"; then
          write_product "$item" "$raw" 0 || failed=1
          echo "name=item_taken_from_an_earlier_attempt item=$item raw=$(basename "$raw")"
          taken=1
          break
        fi
      done
    fi
    [ "$taken" -eq 1 ] || kept+=("$item")
  done
  todo=("${kept[@]+"${kept[@]}"}")
}

next_raw_name() {
  local label=$1 n=1
  while [ -e "$raw_dir/$label-$n.out" ]; do n=$((n + 1)); done
  printf '%s\n' "$raw_dir/$label-$n.out"
}

# 一条臂一个 cargo test：同一个进程里 E206_ARM 只有一个值，项按登记线程并行；进度行另转一份到标准输出。
run_one_arm() {
  local arm=$1 raw filters=() exact=(--exact) item started
  shift
  broken filter-by-substring && exact=()
  for item in "$@"; do filters+=("item_$item"); done
  raw=$(next_raw_name "$arm")
  write_raw_keys "$raw" "$@" || return 1
  started=$SECONDS
  export E206_ARM="$arm"
  if broken progress-not-forwarded; then
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E206 历史用例; 范围=1 个测试目标; 为什么=量固定点逐轮读数，只跑这一趟判定库里没有的项' \
      cargo test --release -p singlefs-verification --test e206_fixed_point_cost_histories -- "${filters[@]}" "${exact[@]}" --nocapture --test-threads="$threads" > "$raw" 2>&1
  else
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E206 历史用例; 范围=1 个测试目标; 为什么=量固定点逐轮读数，只跑这一趟判定库里没有的项' \
      cargo test --release -p singlefs-verification --test e206_fixed_point_cost_histories -- "${filters[@]}" "${exact[@]}" --nocapture --test-threads="$threads" 2>&1 | tee "$raw" | grep --line-buffered '^e206 progress'
  fi
  for item in "$@"; do write_product "$item" "$raw" "$((SECONDS - started))" || failed=1; done
}

# 一份副本（counts 或 plain）的主体：从 kind、work、source_tree 与这份副本的项 chosen_of_kind 起。退出码见文件头。
run_flow_of_kind() {
  kind=$1
  shift
  local item todo=() copy diff_text tests_directory plan_output plan_code finish_code ran_before item_arguments=() tag item_field decision_field key_field arm
  local -a arms=()
  local -A items_of_arm=()
  item_key=() item_decision=()
  threads=$(sed -n 's/^\([0-9][0-9]*\).*/\1/p' "$registered_threads_file" | head -n 1)
  run_flow_shell=$BASHPID
  copy="$work/$kind"
  diff_text="$work/$kind-counters.diff"
  python3 -I "$counters" --build "$kind" --source "$source_tree" --into "$copy" --diff-out "$diff_text"
  plan_code=$?
  [ "$plan_code" -eq 0 ] || return "$plan_code"
  diff_file=$(publish_diff "$diff_text") || return 1
  tests_directory="$copy/crates/singlefs-verification/tests"
  mkdir -p "$tests_directory" || return 1
  if ! cmp -s "$histories" "$tests_directory/e206_fixed_point_cost_histories.rs"; then
    cp "$histories" "$tests_directory/.e206_histories.partial" && mv -f "$tests_directory/.e206_histories.partial" "$tests_directory/e206_fixed_point_cost_histories.rs" || return 1
  fi
  plan_file="$work/$kind-plan.json"
  plan_output="$work/$kind-plan.out"
  for item in "$@"; do item_arguments+=(--item "$item"); done
  python3 -I "$library" plan --copy "$copy" --library-root "$repo" --plan "$plan_file" "${item_arguments[@]}" > "$plan_output"
  plan_code=$?
  cat "$plan_output"
  [ "$plan_code" -eq 0 ] || { echo "name=run_done build=$kind ran=0 failed=1 plan_exit=$plan_code"; return "$plan_code"; }
  while read -r tag item_field decision_field key_field; do
    [ "$tag" = E206PLAN ] || continue
    item_decision[${item_field#item=}]=${decision_field#decision=}
    item_key[${item_field#item=}]=${key_field#key=}
  done < "$plan_output"
  for item in "$@"; do
    if [ -z "${item_decision[$item]:-}" ]; then
      echo "  ✗ 判定库那一份的计划里没有项 $item（$plan_output），不知道它该复用还是现判"
      echo "     → 怎么办：看 $plan_output 里 E206PLAN 那几行少了哪一项、e206_fixed_point_cost_library.py plan 为什么没打它；修好整条命令重跑"
      return 1
    fi
    if [ "${item_decision[$item]}" = reuse ] && ! broken done-items-rerun; then
      echo "name=item_skipped item=$item reason=library key=${item_key[$item]}"
      echo "name=item_line item=$item exit=0 judged=0 red=0 reused=1 seconds=0"
    else
      todo+=("$item")
    fi
  done
  ran_before=${#todo[@]}
  if [ "${#todo[@]}" -gt 0 ]; then
    for item in "${todo[@]}"; do rm -f "$repo"/research/results/.e206-fixed-point-cost-"${item/__/-}"-*.partial; done
    cd "$copy" || return 1
    raw_dir="$copy/.e206-raw"
    mkdir -p "$raw_dir" || return 1
    harvest_finished_items
    for item in ${todo[@]+"${todo[@]}"}; do
      arm=${item##*__}
      broken one-process-for-every-arm && arm=${todo[0]##*__}
      [ -n "${items_of_arm[$arm]:-}" ] || arms+=("$arm")
      items_of_arm[$arm]="${items_of_arm[$arm]:-} $item"
    done
    for arm in ${arms[@]+"${arms[@]}"}; do
      # shellcheck disable=SC2086  # 项名里没有空白，这里按空白切成几项
      run_one_arm "$arm" ${items_of_arm[$arm]}
    done
    cd "$repo" || return 1
  fi
  python3 -I "$library" finish --plan "$plan_file"
  finish_code=$?
  [ "$finish_code" -eq 0 ] || failed=1
  echo "name=run_done build=$kind items=$# ran=$ran_before failed=$failed"
  [ "$library_trouble" -eq 0 ] || return "$library_trouble"
  return "$failed"
}

# 入口的主体：从 chosen 起，plain 那几项走另一份副本（只拷不插），别的走 counts。
run_flow() {
  local item counts_items=() plain_items=() rc=0 one
  failed=0 library_trouble=0
  declare -gA item_key=() item_decision=()
  for item in "${chosen[@]}"; do
    if [ "${item##*__}" = plain ]; then plain_items+=("$item"); else counts_items+=("$item"); fi
  done
  if [ "${#counts_items[@]}" -gt 0 ]; then
    run_flow_of_kind counts "${counts_items[@]}"
    one=$?
    [ "$one" -eq 0 ] || rc=$one
  fi
  if [ "${#plain_items[@]}" -gt 0 ]; then
    run_flow_of_kind plain "${plain_items[@]}"
    one=$?
    [ "$rc" -ne 0 ] || rc=$one
  fi
  return "$rc"
}

# ---- 自证：假驱动（cargo 换成读模式文件的函数，计数脚本跑真的、拷一棵小的假仓根；算键与判定库跑真的，判定库是临时 prune-server），不编译、不跑实现 ----

# 模拟这一趟被系统杀掉：点名两个进程，不在循环里发；假 cargo 跑在管道里，先发 run_flow 那个、再发自己。
simulate_this_run_being_killed() {
  kill -KILL "${run_flow_shell:?}"
  kill -KILL "$BASHPID"
}

# 假 cargo：把参数记进 calls.log、把 E206_ARM 记进 arms.log；按 $world_dir/mode 决定怎么跑。点名的 item_<项> 各打 row 与 item_done 两行。
#   ok 全跑完；rows-differ 全跑完、row 那一行的读数换一个；kill-immediately 一开始就被杀；kill-after-first 第一项跑完、第二项打一行就被杀；
#   kill-cut-line 打出一行被截断的 item_done 就被杀。
fake_cargo() {
  local mode filter name index=0 filters=() argument seen_separator=0 reading=1
  echo "$*" >> "$calls_log"
  echo "${E206_ARM:-unset}" >> "$world_dir/arms.log"
  mode=$(cat "$world_dir/mode")
  [ "$mode" = rows-differ ] && reading=2
  for argument in "$@"; do
    if [ "$seen_separator" -eq 1 ]; then case "$argument" in --*) ;; *) filters+=("$argument") ;; esac; fi
    [ "$argument" = "--" ] && seen_separator=1
  done
  printf 'e206 progress item=fake publishes_done=1 of=2 peak_rounds=1 peak_W=1\n'
  [ "$mode" = kill-immediately ] && simulate_this_run_being_killed
  for filter in "${filters[@]}"; do
    name=${filter#item_}
    index=$((index + 1))
    if [ "$mode" = kill-cut-line ]; then
      printf 'E206RESULT name=item_done item=%s' "${name%??}"
      simulate_this_run_being_killed
    fi
    printf 'E206RESULT name=row item=%s v=%s\n' "$name" "$reading"
    if [ "$mode" = kill-after-first ] && [ "$index" -eq 2 ]; then simulate_this_run_being_killed; fi
    printf 'E206RESULT name=item_done item=%s\n' "$name"
  done
  return 0
}

# 假历史用例：两个小项共用一段历史（复用三问改的就是它），大项另走一段，再加一条别的臂；入口照真的那样拷进副本、在副本上算键。
write_selftest_histories() {
  cat > "$1" <<'EOF'
//! 入口自证用的假历史用例：两个小项共用一段历史，大项另走一段。
use singlefs_core::scan_step;

fn shared_history_of_the_small_items() -> u64 {
    scan_step() + 1
}

fn history_of_the_large_item() -> u64 {
    scan_step() + 2
}

#[test]
fn item_unlinks_1__p1() {
    assert_eq!(shared_history_of_the_small_items(), 2);
}

#[test]
fn item_unlinks_100__p1() {
    assert_eq!(shared_history_of_the_small_items() + 1, 3);
}

#[test]
fn item_unlinks_1000__p1() {
    assert_eq!(history_of_the_large_item(), 3);
}

#[test]
fn item_unlinks_1__p2a() {
    assert_eq!(shared_history_of_the_small_items(), 2);
}
EOF
}

# 假仓根（计数脚本写的那一棵）上补成一个能算键的工作区：两个包，实现包只有一个 scan_step；计数脚本插的那几份文件不在任何模块树里，不进任何项的闭包。
write_selftest_workspace() {
  local root=$1
  mkdir -p "$root/crates/singlefs-verification/src" "$root/crates/singlefs-core/src" || return 1
  cat > "$root/Cargo.toml" <<'EOF'
[workspace]
members = ["crates/singlefs-verification", "crates/singlefs-core"]
exclude = ["prune-server"]
resolver = "2"

[profile.release]
overflow-checks = true
EOF
  cat > "$root/crates/singlefs-verification/Cargo.toml" <<'EOF'
[package]
name = "singlefs-verification"
version = "0.1.0"
edition = "2021"

[dev-dependencies]
singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters", "test-only-publish-thresholds"] }
EOF
  printf 'pub fn harness_marker() -> u32 {\n    1\n}\n' > "$root/crates/singlefs-verification/src/lib.rs" || return 1
  cat > "$root/crates/singlefs-core/Cargo.toml" <<'EOF'
[package]
name = "singlefs-core"
version = "0.1.0"
edition = "2021"

[features]
lookup-work-counters = []
test-only-publish-thresholds = []
EOF
  printf 'pub fn scan_step() -> u64 {\n    1\n}\n' > "$root/crates/singlefs-core/src/lib.rs"
}

# 只答版本的 rustc 与 cargo（算键要 rustc -vV 的 host: 与 cargo -V）；起测试的 cargo 是 shell 函数 cargo（fake_cargo），不经 PATH。
write_version_only_toolchain() {
  local tools=$1
  mkdir -p "$tools" || return 1
  cat > "$tools/rustc" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  -V) echo "rustc 0.0.0-e206-selftest" ;;
  -vV) printf 'rustc 0.0.0-e206-selftest\nbinary: rustc\nhost: x86_64-unknown-linux-gnu\nrelease: 0.0.0-e206-selftest\n' ;;
esac
exit 0
EOF
  cat > "$tools/cargo" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  -V|--version) echo "cargo 0.0.0-e206-selftest" ;;
esac
exit 0
EOF
  chmod +x "$tools/rustc" "$tools/cargo"
}

replace_in_file() {
  local file=$1 old=$2 new=$3 text
  text=$(cat "$file") || return 1
  case "$text" in *"$old"*) ;; *) echo "自证样本 $file 里没有「$old」"; return 1 ;; esac
  printf '%s\n' "${text/"$old"/"$new"}" > "$file"
}

# 一间自证用的小世界：假仓根、假历史用例、假源仓根（补成能算键的工作区）、只答版本的工具链、工作目录、调用记录、模式文件。哨兵条数默认取得很大：键相同的全当哨兵、不抽强制现判。
world() {
  world_dir=$1
  mkdir -p "$world_dir/repo/research/results" "$world_dir/src" || return 1
  repo="$world_dir/repo"
  histories="$world_dir/histories.rs"
  write_selftest_histories "$histories" || return 1
  source_tree="$world_dir/src"
  python3 -I "$counters" --write-fake-source "$source_tree" || return 1
  write_selftest_workspace "$source_tree" || return 1
  write_version_only_toolchain "$world_dir/tools" || return 1
  export PATH="$world_dir/tools:$PATH" CARGO_HOME="$world_dir/tools/cargo-home" SINGLEFS_REUSE_SENTINEL_COUNT=1000
  unset SINGLEFS_REUSE_SENTINEL_SEED
  work="$world_dir/work"
  calls_log="$world_dir/calls.log"
  : > "$calls_log"
  : > "$world_dir/arms.log"
  set_mode ok
}

set_mode() { printf '%s\n' "$1" > "$world_dir/mode"; }

# 跑一遍入口主体（子 shell：被杀只杀这一遍）。输出进 $world_dir/last.out，退出码进 last_rc。
go() {
  chosen=("$@")
  { ( run_flow ) > "$world_dir/last.out" 2>&1; last_rc=$?; } 2>/dev/null
}

calls() { wc -l < "$calls_log" | tr -d ' '; }
products() { find "$repo/research/results" -maxdepth 1 -name "e206-fixed-point-cost-${1/__/-}-*.out" | wc -l | tr -d ' '; }
latest_product() { find "$repo/research/results" -maxdepth 1 -name "e206-fixed-point-cost-${1/__/-}-*.out" -printf '%T@ %p\n' | sort -n | tail -n 1 | cut -d' ' -f2-; }

stored_in_library() {
  local arguments=() item
  for item in "$@"; do arguments+=(--item "$item"); done
  python3 -I "$library" stored --copy "$work/counts" --library-root "$repo" "${arguments[@]}" > "$world_dir/stored.out" 2>&1 || { cat "$world_dir/stored.out"; return 1; }
}

plan_directly() {
  local arguments=() item
  for item in "$@"; do arguments+=(--item "$item"); done
  python3 -I "$library" plan --copy "$work/counts" --library-root "$repo" --plan "$world_dir/direct-plan.json" "${arguments[@]}" > "$world_dir/direct-plan.out" 2>&1 \
    || { echo "plan 该退 0"; cat "$world_dir/direct-plan.out"; return 1; }
}
planned_sentinels() { sed -n 's/^E206SENTINEL .* sentinels=\([^ ]*\) forced=.*$/\1/p' "$1"; }
planned_forced() { sed -n 's/^E206SENTINEL .* forced=\([^ ]*\)$/\1/p' "$1"; }
finish_directly() {
  python3 -I "$library" finish --plan "$world_dir/direct-plan.json" > "$world_dir/finish.out" 2>&1
  finish_rc=$?
}

case_killed_after_the_copy_and_run_again_completes() {
  set_mode kill-immediately
  go unlinks_1__p1 unlinks_100__p1
  [ "$last_rc" -eq 137 ] || { echo "第一遍该被杀（137），得 $last_rc"; return 1; }
  [ -f "$work/counts/.e203-copy-done" ] || { echo "被杀时副本该已拷完、有完成标记"; return 1; }
  set_mode ok
  go unlinks_1__p1 unlinks_100__p1
  [ "$last_rc" -eq 0 ] || { echo "第二遍该跑完（0），得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  grep -q 'name=run_tally judged=2 forced=0 reused=0 refused=0 not_done=0 not_run=0' "$world_dir/last.out" || { echo "计账行不对"; cat "$world_dir/last.out"; return 1; }
  stored_in_library unlinks_1__p1 unlinks_100__p1 || { echo "两项该照键进了判定库"; return 1; }
}

case_a_directory_that_is_not_our_copy_is_refused_and_left_alone() {
  mkdir -p "$work/counts" && printf 'mine\n' > "$work/counts/mine.txt" || return 1
  go unlinks_1__p1
  [ "$last_rc" -eq 2 ] || { echo "该拒绝（2），得 $last_rc"; return 1; }
  [ -f "$work/counts/mine.txt" ] || { echo "不是我们拷的东西不许清"; return 1; }
  [ "$(calls)" -eq 0 ] || { echo "拒绝时不许起 cargo"; return 1; }
}

case_a_finished_item_is_not_run_again() {
  go unlinks_1__p1 unlinks_100__p1
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] && [ "$(products unlinks_1__p1)" -eq 1 ] || { echo "第一遍该跑一次 cargo、各一份产物"; cat "$world_dir/last.out"; return 1; }
  go unlinks_1__p1 unlinks_100__p1
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "第二遍不该再起 cargo（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  grep -q 'name=run_tally judged=0 forced=0 reused=2 refused=0' "$world_dir/last.out" || { echo "第二遍该现判 0、复用 2"; cat "$world_dir/last.out"; return 1; }
  [ "$(products unlinks_1__p1)" -eq 1 ] || { echo "复用的项不另写产物"; return 1; }
}

case_an_item_finished_inside_a_killed_run_is_taken_and_not_run_again() {
  set_mode kill-after-first
  go unlinks_1__p1 unlinks_100__p1
  [ "$last_rc" -eq 137 ] || { echo "第一遍该被杀，得 $last_rc"; return 1; }
  set_mode ok
  go unlinks_1__p1 unlinks_100__p1
  [ "$last_rc" -eq 0 ] || { echo "第二遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  [ "$(calls)" -eq 2 ] || { echo "该共起两次 cargo"; return 1; }
  ! sed -n 2p "$calls_log" | grep -q 'item_unlinks_1__p1 ' || { echo "第二次不该再点名跑完的那一项"; sed -n 2p "$calls_log"; return 1; }
  sed -n 2p "$calls_log" | grep -q 'item_unlinks_100__p1' || { echo "第二次该点名没跑完的那一项"; return 1; }
  stored_in_library unlinks_1__p1 unlinks_100__p1 || { echo "两项该照键进了判定库（取来的那一项也入库）"; return 1; }
}

case_an_item_finished_inside_a_killed_run_under_another_key_is_run_again() {
  set_mode kill-after-first
  go unlinks_1__p1 unlinks_100__p1
  [ "$last_rc" -eq 137 ] || { echo "第一遍该被杀，得 $last_rc"; return 1; }
  replace_in_file "$histories" 'scan_step() + 1' 'scan_step() + 3' || return 1
  set_mode ok
  go unlinks_1__p1 unlinks_100__p1
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "第二遍该跑完、共起两次 cargo（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  sed -n 2p "$calls_log" | grep -q 'item_unlinks_1__p1 ' && sed -n 2p "$calls_log" | grep -q 'item_unlinks_100__p1' \
    || { echo "键变了，第二次该把两项都点名"; sed -n 2p "$calls_log"; return 1; }
}

case_a_line_cut_by_a_kill_does_not_finish_an_item() {
  set_mode kill-cut-line
  go unlinks_1__p1
  [ "$last_rc" -eq 137 ] || { echo "第一遍该被杀，得 $last_rc"; return 1; }
  set_mode ok
  # 残行的项名少了末两个字符；项名短一截时它正好是别的项名，这里手写一份完整项名的残行
  printf 'E206RESULT name=item_done item=unlinks_1__p1' > "$work/counts/.e206-raw/p1-9.out"
  printf 'unlinks_1__p1\t%s\n' "$(python3 -I "$library" keys --copy "$work/counts" --item unlinks_1__p1 | sed -n 's/.* key=\([^ ]*\).*/\1/p')" > "$work/counts/.e206-raw/p1-9.out.keys"
  go unlinks_1__p1
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "残行不算跑完，第二遍该再起 cargo（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
}

case_the_item_filter_names_one_test_only() {
  go unlinks_1__p1
  sed -n 1p "$calls_log" | grep -q -- ' --exact ' || { echo "点名该带 --exact"; sed -n 1p "$calls_log"; return 1; }
}

case_a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name() {
  { ( header_for() { kill -KILL "$BASHPID"; }; chosen=(unlinks_1__p1); run_flow ) > "$world_dir/last.out" 2>&1; last_rc=$?; } 2>/dev/null
  [ "$last_rc" -eq 137 ] || { echo "第一遍该在写产物时被杀，得 $last_rc"; return 1; }
  [ "$(products unlinks_1__p1)" -eq 0 ] || { echo "被杀时产物名下不该有文件"; return 1; }
  go unlinks_1__p1
  [ "$last_rc" -eq 0 ] || { echo "第二遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  [ "$(calls)" -eq 1 ] || { echo "原始输出已跑完，第二遍不该再起 cargo（calls=$(calls)）"; return 1; }
  [ "$(products unlinks_1__p1)" -eq 1 ] || { echo "该恰好一份产物"; return 1; }
  [ -z "$(find "$repo/research/results" -maxdepth 1 -name '.*.partial')" ] || { echo "残留的隐藏 .partial 该已清掉"; return 1; }
}

case_progress_lines_reach_the_standard_output() {
  go unlinks_1__p1
  grep -q '^e206 progress item=fake publishes_done=1 of=2 peak_rounds=1 peak_W=1$' "$world_dir/last.out" || { echo "进度行该转到标准输出"; cat "$world_dir/last.out"; return 1; }
  product=$(latest_product unlinks_1__p1)
  ! grep -q '^e206 progress' "$product" || { echo "进度行不进产物"; return 1; }
}

case_the_product_header_records_the_tested_files() {
  go unlinks_1__p1
  product=$(latest_product unlinks_1__p1)
  [ "$(grep -c '^E206RESULT name=source_sha256 ' "$product")" -eq 3 ] || { echo "产物头该记三份被测文件的 sha256"; cat "$product"; return 1; }
  grep -q '^E206RESULT name=header for_item=unlinks_1__p1 ' "$product" || { echo "产物头该有 header 行"; cat "$product"; return 1; }
}

case_each_arm_runs_in_its_own_process() {
  go unlinks_1__p1 unlinks_1__p2a
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "两条臂该起两次 cargo（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  [ "$(sed -n 1p "$world_dir/arms.log")" = p1 ] && [ "$(sed -n 2p "$world_dir/arms.log")" = p2a ] || { echo "每次的 E206_ARM 该是那条臂"; cat "$world_dir/arms.log"; return 1; }
  sed -n 1p "$calls_log" | grep -q 'item_unlinks_1__p1 ' && ! sed -n 1p "$calls_log" | grep -q 'item_unlinks_1__p2a' || { echo "一次 cargo 只点名一条臂的项"; cat "$calls_log"; return 1; }
}

case_the_plain_item_goes_to_a_copy_without_the_counters() {
  go unlinks_1__plain
  [ -f "$work/plain/.e203-copy-done" ] || { echo "plain 该有自己的副本"; cat "$world_dir/last.out"; return 1; }
  ! grep -q e206_fixed_point "$work/plain/crates/singlefs-core/src/transaction.rs" || { echo "plain 的副本里不该有读数"; return 1; }
  grep -q 'pub fn build_name' "$work/plain/crates/singlefs-verification/tests/e206_readings/mod.rs" || { echo "plain 的副本要有读数的空实现"; return 1; }
}

# 复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：头一趟库是空的、同一份输入、闭包外改动、闭包内改动。
case_reuse_three_questions() {
  local runs="$world_dir/three-questions"
  mkdir -p "$runs" || return 1
  go unlinks_1__p1 unlinks_100__p1 unlinks_1000__p1
  cp "$world_dir/last.out" "$runs/first.out" || return 1
  go unlinks_1__p1 unlinks_100__p1 unlinks_1000__p1
  cp "$world_dir/last.out" "$runs/second.out" || return 1
  printf '\n// 历史用例里加的一行注释\nfn spare_history_nobody_runs() -> u64 {\n    9\n}\n' >> "$histories" || return 1
  printf '// 源码变了的一行\n' >> "$source_tree/crates/singlefs-core/src/transaction.rs" || return 1
  go unlinks_1__p1 unlinks_100__p1 unlinks_1000__p1
  cp "$world_dir/last.out" "$runs/unrelated.out" || return 1
  replace_in_file "$histories" 'scan_step() + 1' 'scan_step() + 3' || return 1
  go unlinks_1__p1 unlinks_100__p1 unlinks_1000__p1
  cp "$world_dir/last.out" "$runs/unit-change.out" || return 1
  python3 -I "$here/../gate/lib_reuse_checks.py" three-questions --script "$entry_script_key" --first "$runs/first.out" --second "$runs/second.out" \
    --unrelated "$runs/unrelated.out" --unit-change "$runs/unit-change.out" --affected unlinks_1__p1,unlinks_100__p1 --from-output
}

# 键不随副本的路径变：同一份副本拷到两个别的目录，键逐字相同；工作目录换到别处，库里判过的照键复用。
case_the_item_key_does_not_depend_on_the_copy_path() {
  local one="$world_dir/one/place" another="$world_dir/another/place/deeper"
  go unlinks_1__p1 unlinks_1000__p1
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "头一遍该跑完、起一次 cargo（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  mkdir -p "$one" "$another" && cp -r "$work/counts/." "$one/" && cp -r "$work/counts/." "$another/" || return 1
  python3 -I "$library" keys --copy "$one" --item unlinks_1__p1 --item unlinks_1000__p1 > "$world_dir/keys-one.out" 2>&1 || { cat "$world_dir/keys-one.out"; return 1; }
  python3 -I "$library" keys --copy "$another" --item unlinks_1__p1 --item unlinks_1000__p1 > "$world_dir/keys-another.out" 2>&1 || { cat "$world_dir/keys-another.out"; return 1; }
  [ "$(grep -c '^E206KEY ' "$world_dir/keys-one.out")" -eq 2 ] && ! grep -q 'key=refused' "$world_dir/keys-one.out" || { echo "两项都该算得出键"; cat "$world_dir/keys-one.out"; return 1; }
  cmp -s "$world_dir/keys-one.out" "$world_dir/keys-another.out" || { echo "同一份源码放两个目录，键该相同"; cat "$world_dir/keys-one.out" "$world_dir/keys-another.out"; return 1; }
  work="$world_dir/elsewhere/work"
  go unlinks_1__p1 unlinks_1000__p1
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "换了副本的路径，该照键复用、不再起 cargo（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  grep -q 'name=run_tally judged=0 forced=0 reused=2 refused=0' "$world_dir/last.out" || { echo "该现判 0、复用 2"; cat "$world_dir/last.out"; return 1; }
}

case_reuse_sentinel_draws_and_the_forced_item_is_run_again() {
  local forced
  go unlinks_1__p1 unlinks_100__p1 unlinks_1000__p1
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "头一遍该跑完（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  go unlinks_1__p1 unlinks_100__p1 unlinks_1000__p1
  [ "$last_rc" -eq 0 ] || { echo "哨兵复用、强制现判与库里相同，该退 0，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  forced=$(planned_forced "$world_dir/last.out")
  [ -n "$forced" ] && [ "$forced" != - ] || { echo "三项键相同、哨兵 1 条，该抽出一条强制现判"; cat "$world_dir/last.out"; return 1; }
  [ "$(calls)" -eq 2 ] && [ "$(sed -n 2p "$calls_log" | grep -o 'item_[a-z0-9_]*' | wc -l)" -eq 1 ] && sed -n 2p "$calls_log" | grep -q "item_$forced " \
    || { echo "第二遍该只起一次 cargo、只点名强制现判的 $forced"; cat "$calls_log"; return 1; }
  grep -q 'name=run_tally judged=1 forced=1 reused=2 refused=0 not_done=0 not_run=0' "$world_dir/last.out" || { echo "计账行不对"; cat "$world_dir/last.out"; return 1; }
}

case_a_rejudged_sentinel_turns_the_run_red() {
  local sentinel forced
  go unlinks_1__p1 unlinks_100__p1 unlinks_1000__p1
  [ "$last_rc" -eq 0 ] || { echo "头一遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  plan_directly unlinks_1__p1 unlinks_100__p1 unlinks_1000__p1 || return 1
  sentinel=$(planned_sentinels "$world_dir/direct-plan.out")
  forced=$(planned_forced "$world_dir/direct-plan.out")
  [ -n "$sentinel" ] && [ "$sentinel" != - ] && [ -n "$forced" ] && [ "$forced" != - ] || { echo "该抽出一条哨兵、一条强制现判"; cat "$world_dir/direct-plan.out"; return 1; }
  python3 -I "$library" record --plan "$world_dir/direct-plan.json" --item "$sentinel" --product "$(latest_product "$sentinel")" || return 1
  python3 -I "$library" record --plan "$world_dir/direct-plan.json" --item "$forced" --product "$(latest_product "$forced")" || return 1
  finish_directly
  [ "$finish_rc" -eq 1 ] && grep -q '复用失灵' "$world_dir/finish.out" || { echo "哨兵被现判，该退 1、报复用失灵（得 $finish_rc）"; cat "$world_dir/finish.out"; return 1; }
}

case_a_forced_rejudge_with_another_verdict_turns_the_run_red() {
  go unlinks_1__p1 unlinks_100__p1 unlinks_1000__p1
  [ "$last_rc" -eq 0 ] || { echo "头一遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  set_mode rows-differ
  go unlinks_1__p1 unlinks_100__p1 unlinks_1000__p1
  [ "$last_rc" -eq 1 ] && grep -q '判定不稳' "$world_dir/last.out" || { echo "强制现判的读数与库里不同，该退 1、报判定不稳（得 $last_rc）"; cat "$world_dir/last.out"; return 1; }
}

case_an_item_planned_for_judging_but_not_run_turns_the_run_red() {
  go unlinks_1000__p1
  [ "$last_rc" -eq 0 ] || { echo "头一遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  plan_directly unlinks_1__p1 unlinks_100__p1 || return 1
  finish_directly
  [ "$finish_rc" -eq 1 ] && grep -q '没跑到' "$world_dir/finish.out" || { echo "要现判的两项一条都没记，该退 1、报没跑到（得 $finish_rc）"; cat "$world_dir/finish.out"; return 1; }
}

case_an_item_without_a_key_runs_and_is_never_recorded() {
  # 假历史用例里没有 item_files_1__p1：入口找不到，拒算
  go files_1__p1
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "拒算的项照跑、退 0（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  grep -q 'E206PLAN item=files_1__p1 decision=refused key=-' "$world_dir/last.out" && grep -q 'name=run_tally judged=0 forced=0 reused=0 refused=1' "$world_dir/last.out" \
    || { echo "该报拒算 1"; cat "$world_dir/last.out"; return 1; }
  go files_1__p1
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "拒算的项不入库，下一趟照跑（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
}

case_an_unreachable_library_exits_75_and_runs_nothing() {
  local port
  port=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()') || return 1
  export SINGLEFS_PRUNE_SERVER_SOCKET="$world_dir/no-server.sock" SINGLEFS_PRUNE_SERVER_TCP_ADDRESS="127.0.0.1:$port" SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS=2
  go unlinks_1__p1
  [ "$last_rc" -eq 75 ] || { echo "判定库连不上该退 75，得 $last_rc"; return 1; }
  [ "$(calls)" -eq 0 ] || { echo "连不上时不许起 cargo（不当库里没有现判）"; return 1; }
}

selftest() {
  local failures=0 out name display reuse_line="" line real_histories=$histories real_entry=$0
  selftest_root=$(mktemp -d "${TMPDIR:-/tmp}/e206-run-selftest.XXXXXX") || exit 1
  trap 'rm -rf "$selftest_root"' EXIT
  # 判定库那一份只在有这份标记文件时认弄坏开关
  printf 'e206-selftest\n' > "$selftest_root/mark"
  export E206_SELFTEST_MARK="$selftest_root/mark"
  # 自证只在临时 prune-server 上跑：经判定库那一份调客户端 selftest_service_problem，不是临时服务退 2、一格不跑
  python3 -I "$library" selftest-service-problem || return 2
  cargo() { fake_cargo "$@"; }
  out=$(items_table | wc -l)
  [ "$out" -eq 86 ] || { echo "name=selftest_case case=item_count result=FAILED got=$out"; failures=$((failures + 1)); }
  [ "$(items_table | awk '$2=="first"' | wc -l)" -eq 40 ] || { echo "name=selftest_case case=first_segment_items result=FAILED"; failures=$((failures + 1)); }
  if [ "$(items_table | cut -d' ' -f1 | sort)" = "$(sed -n 's/^fn item_\(.*\)() {$/\1/p' "$real_histories" | sort)" ]; then
    echo "name=selftest_case case=the_table_matches_the_histories result=ok"
  else
    echo "name=selftest_case case=the_table_matches_the_histories result=FAILED"
    failures=$((failures + 1))
  fi
  local tmp="$selftest_root/product"
  printf 'E206RESULT name=row x=1\n' > "$tmp"
  if product_is_done "$tmp"; then echo "name=selftest_case case=done_marker result=FAILED"; failures=$((failures + 1)); else echo "name=selftest_case case=a_product_without_the_marker_is_not_done result=ok"; fi
  printf 'E206RESULT name=item_done item=base__p1\n' >> "$tmp"
  if product_is_done "$tmp"; then echo "name=selftest_case case=a_product_with_the_marker_is_done result=ok"; else echo "name=selftest_case case=done_marker result=FAILED"; failures=$((failures + 1)); fi
  if E206_RUN_BREAK='' bash "$real_entry" --item no_such_item__p1 --work "$selftest_root/unused" > /dev/null 2>&1; then
    echo "name=selftest_case case=an_unknown_item_name_exits_2 result=FAILED"; failures=$((failures + 1))
  else
    echo "name=selftest_case case=an_unknown_item_name_exits_2 result=ok"
  fi
  if [ "$(E206_RUN_BREAK='' bash "$real_entry" --list-items | wc -l)" -eq 86 ]; then
    echo "name=selftest_case case=list_items_lists_every_item result=ok"
  else
    echo "name=selftest_case case=list_items_lists_every_item result=FAILED"; failures=$((failures + 1))
  fi
  for name in \
    case_killed_after_the_copy_and_run_again_completes \
    case_a_directory_that_is_not_our_copy_is_refused_and_left_alone \
    case_a_finished_item_is_not_run_again \
    case_an_item_finished_inside_a_killed_run_is_taken_and_not_run_again \
    case_an_item_finished_inside_a_killed_run_under_another_key_is_run_again \
    case_a_line_cut_by_a_kill_does_not_finish_an_item \
    case_the_item_filter_names_one_test_only \
    case_a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name \
    case_progress_lines_reach_the_standard_output \
    case_the_product_header_records_the_tested_files \
    case_each_arm_runs_in_its_own_process \
    case_the_plain_item_goes_to_a_copy_without_the_counters \
    case_reuse_three_questions \
    case_the_item_key_does_not_depend_on_the_copy_path \
    case_reuse_sentinel_draws_and_the_forced_item_is_run_again \
    case_a_rejudged_sentinel_turns_the_run_red \
    case_a_forced_rejudge_with_another_verdict_turns_the_run_red \
    case_an_item_planned_for_judging_but_not_run_turns_the_run_red \
    case_an_item_without_a_key_runs_and_is_never_recorded \
    case_an_unreachable_library_exits_75_and_runs_nothing; do
    display=${name#case_}
    [ "$display" = reuse_three_questions ] && display=reuse-three-questions
    mkdir -p "$selftest_root/$name"
    if ( world "$selftest_root/$name" && "$name" ) > "$selftest_root/$name/out.txt" 2>&1; then
      echo "name=selftest_case case=$display result=ok"
    else
      echo "name=selftest_case case=$display result=FAILED"
      sed 's/^/  | /' "$selftest_root/$name/out.txt" | tail -n 15
      failures=$((failures + 1))
    fi
    line=$(grep '^REUSE_SELFTEST ' "$selftest_root/$name/out.txt" | tail -n 1)
    [ -z "$line" ] || reuse_line=$line
  done
  [ -z "$reuse_line" ] || echo "$reuse_line"
  echo "name=selftest_done failures=$failures"
  [ "$failures" -eq 0 ]
}

# 弄坏开关只在 --selftest 里认；别处设了拒（判定库那一份由自证设的标记文件放行，见 selftest）。
if [ -n "${E206_RUN_BREAK:-}" ] && [ "${1:-}" != --selftest ]; then
  echo "  ✗ E206_RUN_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "     → 怎么办：去掉 E206_RUN_BREAK 再跑；要证红就跑 E206_RUN_BREAK=<项> bash $0 --selftest（经临时 prune-server 的包装）" >&2
  exit 2
fi

work="" source_tree="$repo" segment="" requested=() chosen=()
list_only=0
needs_value() {
  [ "$1" -ge 2 ] && return 0
  echo "  ✗ $2 后面缺值；可点名的项：" >&2
  items_table >&2
  echo "     → 怎么办：在 $2 后面写上面列的一项，或用 --list-items 看全部" >&2
  exit 2
}
while [ $# -gt 0 ]; do
  case "$1" in
    --list-items) list_only=1; shift ;;
    --selftest) selftest; exit $? ;;
    --work) needs_value $# "$1"; work=$2; shift 2 ;;
    --source) needs_value $# "$1"; source_tree=$2; shift 2 ;;
    --segment) needs_value $# "$1"; segment=$2; shift 2 ;;
    --item) needs_value $# "$1"; requested+=("$2"); shift 2 ;;
    *) usage ;;
  esac
done

if [ "$list_only" -eq 1 ]; then items_table; exit 0; fi
for item in "${requested[@]+"${requested[@]}"}"; do
  items_table | awk -v i="$item" '$1==i {found=1} END {exit !found}' || { echo "项名写错：$item；可点名的项：" >&2; items_table >&2; exit 2; }
done
[ -n "$work" ] || usage
[ -f "$counters" ] && [ -f "$library" ] && [ -f "$histories" ] || { echo "前置文件不在：$counters、$library 或 $histories" >&2; exit 78; }

if [ -n "$segment" ]; then
  while read -r item seg; do [ "$seg" = "$segment" ] && chosen+=("$item"); done < <(items_table)
  [ "${#chosen[@]}" -gt 0 ] || usage
fi
for item in "${requested[@]+"${requested[@]}"}"; do
  case " ${chosen[*]+"${chosen[*]}"} " in *" $item "*) ;; *) chosen+=("$item") ;; esac
done
if [ -z "$segment" ] && [ "${#requested[@]}" -eq 0 ]; then
  while read -r item _; do chosen+=("$item"); done < <(items_table)
fi

mkdir -p "$work" && work=$(cd "$work" && pwd) || { echo "工作目录建不出：$work" >&2; exit 2; }
[ -d "$source_tree" ] || { echo "仓根不在：$source_tree" >&2; exit 78; }
source_tree=$(cd "$source_tree" && pwd)
run_flow
