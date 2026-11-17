#!/usr/bin/env bash
# task-server-compiles: yes
# E209（尖峰成因）量的那一路的入口：给仓副本套上读数，起历史用例，产物按 <点>__<臂>__<脚本> 分文件写进 research/results/，每项的判定照单元键进 prune-server。
# 跑前登记 research/prompts/e209-preregistration.md 第五节「装置写在哪」第 4 条、第十四节。不分片：没有一项估计超过两小时。
#
# 用法：
#   e209-spike-cause-run.sh --list-items
#   e209-spike-cause-run.sh --work <共享根下的工作目录> [--segment first|second|third|fourth|gate | --item <点>__<臂>__<脚本>[__twin]…] [--source <仓根>]
#       不带范围参数是全部登记的项（不指定范围）；--item 给几次取并集，项名写错退 2 并列可点的项。
#       gate 段是第一段的闸转到 files_200_units_100 时那一项；孪生项 <项>__twin 起原项的用例、另起进程、一个测试线程，产物头记 twin=yes。
#   e209-spike-cause-run.sh --selftest        核入口自己的列项、拒绝、完成标记、续跑、按键复用、复用哨兵与孪生项，用假驱动，不编译、不跑实现；
#                                                  只在临时 prune-server 上跑（开跑先经判定库那一份调客户端 selftest_service_problem，不是临时服务退 2、一格不跑）
# 判定库（.claude/rules/verification.md「判定库只有一份」；键、库与哨兵的写法在同目录 e209_spike_cause_library.py 文件头）：
#   每个 <点>__<臂>__<脚本> 一条判定，键是副本里用例 item_<项> 的单元闭包键加工具链行、闭包真引到的依赖行、臂与跑法；副本路径不进键。族 apparatus-items，种类 e209-spike-cause-run。
#   research/results/ 里的产物不当判定读，也不导入：库里没有的项照键现判一次。孪生项没有自己的用例，库里没有它的键，每趟照跑、不入库。
# 一趟只编译一次：一份副本一棵 cargo 目录（服务给的 CARGO_TARGET_DIR），同一条臂的项在同一个进程里按登记线程并行；臂由环境变量 E209_ARM 在进程开头读一次
#   （项名第二段：head 或 plain），所以一条臂一个 cargo test；孪生项单独一组、另起进程、--test-threads=1。
# 同一条命令重起就是续跑：副本照计数脚本的开拷、完成标记；库里有键相同、做完了的项不重跑；上一次尝试的原始输出（副本里 .e209-raw 下，旁边 .keys 记着起 cargo 时的键）里
#   已经跑完、键又相同的现判项直接写成产物、照键入库；产物先写同目录隐藏 .partial 再换名；做完而没有 item_done 的产物不删不覆盖，重起时另写新文件名。
# 进度：历史那一段每过 5 分钟一行 `e209 progress item=<项> publishes_done=<n> of=<总数> peak_rounds=<r>`，转一份到标准输出；判结果只读原始输出。不作计时用。
#
# run-condition: 无（不要 GPU、不要第二台）。
# 退出码：0 跑到的全绿；1 有项没跑出 item_done、复用哨兵红、或计划里要现判的项这一趟没记；2 用法错、项名写错或工作目录里有不是本脚本拷的东西；
#   3 prune-server 客户端别的错，或锚点不是恰好命中一次（登记停机条款第 1 条）；75 prune-server 不可用（照客户端：不现判、不当「库里没有」）；
#   77 一项都没跑到；78 准入条件不满足（仓根或前置脚本不在）。
# 弄坏开关（只给 --selftest 用，平常跑时不认）。逐项写：开关、弄坏哪一处、打开后哪几格该红（失败行句首 name=selftest_case case=<格名> result=FAILED）：
#   E209_RUN_BREAK=done-marker-ignored  完成标记不看（凡判「做完没有」的地方都当做完了）
#       done_marker
#   E209_RUN_BREAK=done-items-rerun  计划复用的项照样重跑（不看 plan 给的 decision）
#       one_finished_item_is_not_run_again
#   E209_RUN_BREAK=harvest-off  不从上一次尝试的原始输出里取跑完的项
#       an_item_finished_inside_a_killed_run_is_taken_and_not_run_again
#   E209_RUN_BREAK=harvest-ignores-the-key  从原始输出取项时不核旁边 .keys 记的键
#       an_item_finished_inside_a_killed_run_under_another_key_is_run_again
#   E209_RUN_BREAK=cut-lines-kept  取项时不丢最后一行不带换行的残行
#       one_line_cut_by_a_kill_does_not_finish_an_item
#   E209_RUN_BREAK=product-written-in-place  产物直接写在产物名下（被杀时留下半份）
#       one_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name
#   E209_RUN_BREAK=filter-by-substring  点名不带 --exact，一项的名字会连带跑以它为前缀的别的项
#       the_item_filter_names_one_test_only
#   E209_RUN_BREAK=copy-resume-off  副本不认完成标记与开拷标记，非空一律拒（在计数脚本里读，入口只是把环境带过去）
#       killed_after_the_copy_and_run_again_completes
#   E209_RUN_BREAK=progress-not-forwarded  进度行不转到标准输出
#       progress_lines_reach_the_standard_output
#   E209_RUN_BREAK=one-process-for-every-arm  几条臂的项并进一个 cargo test（E209_ARM 只有头一条臂的值）
#       each_arm_runs_in_its_own_process
#   E209_RUN_BREAK=twin-in-the-same-process  孪生项与原项一个并发度（--test-threads 不降成 1；登记第九节 M16）
#       the_twin_item_runs_in_another_process_with_one_test_thread
#   E209_RUN_BREAK=test-prefix-kept  写产物时不去掉 libtest 单线程时接在第一行前面的「test <用例名> ... 」（孪生项只一个测试线程，第一行 name=point 会被丢）
#       the_twin_item_runs_in_another_process_with_one_test_thread
#   E209_RUN_BREAK=third-without-gate-item  --segment third 不含登记第三段里属于 files_200_units_100 的 walk_forward（表里它标 gate）
#       third_segment_holds_the_gate_item
#   下面几项在判定库那一份里读（e209_spike_cause_library.py 文件头「弄坏开关」，入口只是把环境带过去）：
#   E209_RUN_BREAK=library-lookup-ignored  决定复用那一路不读库
#       one_finished_item_is_not_run_again、reuse-three-questions
#   E209_RUN_BREAK=record-skipped  跑完的项不往库里写
#       one_finished_item_is_not_run_again、reuse-three-questions
#   E209_RUN_BREAK=key-with-copy-path  请求另加副本的绝对路径
#       the_item_key_does_not_depend_on_the_copy_path
#   E209_RUN_BREAK=key-by-whole-histories-file  整份历史用例进键
#       reuse-three-questions
#   E209_RUN_BREAK=sentinel-skipped  不抽哨兵与强制现判、不判
#       reuse_sentinel_draws_and_the_forced_item_is_run_again、one_rejudged_sentinel_turns_the_run_red
#   E209_RUN_BREAK=sentinel-rejudge-unobserved  跑完把哨兵一律记成复用
#       one_rejudged_sentinel_turns_the_run_red
#   E209_RUN_BREAK=forced-read-as-reused  抽中强制现判的项照库复用
#       reuse_sentinel_draws_and_the_forced_item_is_run_again
#   E209_RUN_BREAK=forced-verdict-unchecked  不比强制现判的判定与库里那一份
#       one_forced_rejudge_with_another_verdict_turns_the_run_red
#   E209_RUN_BREAK=refused-items-reused  拒算的项照复用跳过、不跑
#       an_item_without_a_key_runs_and_is_never_recorded
#   E209_RUN_BREAK=unreachable-as-empty  读库时 prune-server 连不上照「库里没有」现判
#       an_unreachable_library_exits_75_and_runs_nothing
#   E209_RUN_BREAK=unrun-items-unchecked  跑完不核计划里要现判的项这一趟记过没有
#       an_item_planned_for_judging_but_not_run_turns_the_run_red
set -u

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
counters="$here/e209_spike_cause_counters.py"
library="$here/e209_spike_cause_library.py"
histories="$here/e209_spike_cause_histories.rs"
scripts_module="$here/e209_spike_cause_scripts.rs"   # 历史用例 #[path] 引的脚本表与选择子，拷进副本的 tests/e209_scripts/mod.rs
registered_threads_file="$here/../../.claude/heavy-thread-default"
entry_script_key="scripts/experiment-apparatus/e209-spike-cause-run.sh"   # 复用三问规范输出认的脚本名

# 项 段（登记第五节「取样点」「实现量与分段」；与历史用例的 item_ 名单同一份，自证 the_table_matches_the_histories 核）
# 项 段（登记第五节「取样点」「实现量与分段」；与历史用例的 item_ 名单同一份，自证 the_table_matches_the_histories 核）。
# 孪生项 <项>__twin 不在历史用例里：它起的是原项那一条用例，另起进程、一个测试线程（登记第七节 ③ 第 5 条）。
items_table() {
  local point script
  for script in walk_forward first_repeated mount_then_first first_mount_first mount_twice_first; do
    echo "files_100_units_100__head__$script first"
  done
  echo "files_100_units_100__plain__walk_forward first"
  echo "files_100_units_100__head__walk_forward__twin first"
  for script in walk_backward walk_from_middle create_then_first filler_create_1 filler_create_2 filler_create_4 filler_create_8 filler_create_16 \
    filler_young_1 filler_young_4 filler_young_8 presweep_1 presweep_2 presweep_4 presweep_8 presweep_16 presweep_32 presweep_64 young_mount_first old_mount_old; do
    echo "files_100_units_100__head__$script second"
  done
  echo "files_200_units_100__head__walk_forward gate"
  for point in files_75_units_100 files_200_units_100 files_100_units_100_unit_area_2_pow_17 files_100_units_100_unit_area_2_pow_20; do
    for script in walk_forward walk_backward first_repeated filler_create_4 filler_young_4 presweep_8 presweep_32 mount_then_first first_mount_first mount_twice_first; do
      [ "$point $script" = "files_200_units_100 walk_forward" ] && continue
      echo "${point}__head__$script third"
    done
  done
  for script in walk_forward walk_backward first_repeated filler_create_4 filler_young_4 mount_then_first first_mount_first mount_twice_first; do
    echo "files_10_units_1000__head__$script third"
  done
  for point in files_250_units_100 files_300_units_100; do
    echo "${point}__head__walk_forward fourth"
    echo "${point}__head__mount_then_first fourth"
  done
}

# 一段的项：登记第三段是五个检验点的 48 项，其中 files_200_units_100 的 walk_forward 表里标 gate（第一段闸转到它时单独跑），第三段也要有它。
items_of_segment() {
  local wanted=$1 item listed_segment
  while read -r item listed_segment; do
    if [ "$listed_segment" = "$wanted" ]; then echo "$item"
    elif [ "$wanted" = third ] && [ "$listed_segment" = gate ] && ! broken third-without-gate-item; then echo "$item"
    fi
  done < <(items_table)
}

usage() { sed -n '2,/^set -u/p' "$0" | sed '$d' >&2; exit 2; }

broken() { [ "${E209_RUN_BREAK:-}" = "$1" ]; }

# 项名 <点>__<臂>__<脚本>[__twin] 的臂（head 或 plain）。
arm_of_item() {
  local rest=${1#*__}
  echo "${rest%%__*}"
}

# 起 cargo 的分组：孪生项（项名末尾 __twin）单独一组、另起进程；别的按臂分。
group_of() {
  case "$1" in
    *__twin) echo twin ;;
    *) arm_of_item "$1" ;;
  esac
}

# 一份产物是否做完：有 item_done 行。弄坏开关打开时不看。
product_is_done() {
  broken done-marker-ignored && return 0
  grep -q '^E209RESULT name=item_done ' "$1" 2>/dev/null
}

histories_key() {
  sha256sum "$histories" | cut -c1-16
}

scripts_key() {
  sha256sum "$scripts_module" | cut -c1-16
}

# 历史用例与它 #[path] 引的量窗取法拷进副本的 tests 目录（内容相同的不动；先写同目录 .partial 再换名）。
place_the_histories() {
  local tests_directory=$1
  mkdir -p "$tests_directory/e209_scripts" || return 1
  if ! cmp -s "$histories" "$tests_directory/e209_spike_cause_histories.rs"; then
    cp "$histories" "$tests_directory/.e209_histories.partial" && mv -f "$tests_directory/.e209_histories.partial" "$tests_directory/e209_spike_cause_histories.rs" || return 1
  fi
  if ! cmp -s "$scripts_module" "$tests_directory/e209_scripts/mod.rs"; then
    cp "$scripts_module" "$tests_directory/e209_scripts/.mod.rs.partial" && mv -f "$tests_directory/e209_scripts/.mod.rs.partial" "$tests_directory/e209_scripts/mod.rs" || return 1
  fi
}

# 新文件名：e209-spike-cause-<点>-<臂>-<日期>.out，同一天再来加 -r2、-r3；已有的一个不动。
new_product_name() {
  local stem day name attempt=1
  stem="${1//__/-}"
  day=$(TZ=Asia/Tokyo date +%F)
  name="$repo/research/results/e209-spike-cause-$stem-$day.out"
  while [ -e "$name" ]; do
    attempt=$((attempt + 1))
    name="$repo/research/results/e209-spike-cause-$stem-$day-r$attempt.out"
  done
  printf '%s\n' "$name"
}

complete_lines() {
  if broken cut-lines-kept; then cat "$1"; else head -n "$(wc -l < "$1")" "$1"; fi
}

raw_has_finished_item() {
  complete_lines "$1" | grep -qxF "E209RESULT name=item_done item=${2%__twin}"
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
  local text=$1 existing day name attempt=1 temporary
  for existing in "$repo"/research/results/e209-spike-cause-counters-diff-*.diff; do
    if [ -f "$existing" ] && cmp -s "$text" "$existing"; then printf '%s\n' "$existing"; return 0; fi
  done
  day=$(TZ=Asia/Tokyo date +%F)
  name="$repo/research/results/e209-spike-cause-counters-diff-$day.diff"
  while [ -e "$name" ]; do attempt=$((attempt + 1)); name="$repo/research/results/e209-spike-cause-counters-diff-$day-r$attempt.diff"; done
  temporary="$(dirname "$name")/.$(basename "$name").partial"
  cp "$text" "$temporary" && mv -f "$temporary" "$name" || return 1
  printf '%s\n' "$name"
}

# 产物头：构建、线程、历史用例的内容键、副本的 diff、这一项的键、主工作区三份被测文件的 sha256（停机条款第 3 条）与锚点行号。字段名不写 item=，判定只取带 item= 的行。
header_for() {
  local item=$1 file twin=no shown_threads=$threads
  case "$item" in *__twin) twin=yes; item=${item%__twin}; broken twin-in-the-same-process || shown_threads=1 ;; esac
  echo "E209RESULT name=header for_item=$item twin=$twin build=$kind profile=release opt_level=3 test_threads=$shown_threads histories=$(histories_key) scripts=$(scripts_key) diff_file=$(basename "$diff_file") diff_lines=$(wc -l < "$diff_file") item_key=${item_key[$item]:--}"
  for file in transaction.rs allocation_record_tree.rs allocator.rs; do
    echo "E209RESULT name=source_sha256 file=crates/singlefs-core/src/$file sha256=$(sha256sum "$source_tree/crates/singlefs-core/src/$file" 2>/dev/null | cut -d' ' -f1)"
  done
  python3 -I "$counters" --check-anchors "$source_tree" | sed 's/^/E209RESULT /'
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
  view="$raw_directory/view.tmp"
  # 孪生项只一个测试线程：libtest 在第一行输出前接「test <用例名> ... 」，不去掉第一行（name=point）就丢了。
  if broken test-prefix-kept; then complete_lines "$raw" > "$view"; else complete_lines "$raw" | sed 's/^test [^ ]* \.\.\. //' > "$view"; fi
  { header_for "$item"; grep -E "^E209RESULT .* item=${item%__twin}( |$)" "$view"; } > "$temporary"
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
      for raw in "$raw_directory"/*.out; do
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
  local label=$1 attempt=1
  while [ -e "$raw_directory/$label-$attempt.out" ]; do attempt=$((attempt + 1)); done
  printf '%s\n' "$raw_directory/$label-$attempt.out"
}

# 一条臂一个 cargo test：同一个进程里 E209_ARM 只有一个值，项按登记线程并行；进度行另转一份到标准输出。
run_one_arm() {
  local arm=$1 raw filters=() exact=(--exact) item started test_threads=$threads
  shift
  broken filter-by-substring && exact=()
  for item in "$@"; do filters+=("item_${item%__twin}"); done
  [ "$arm" = twin ] && ! broken twin-in-the-same-process && test_threads=1
  raw=$(next_raw_name "$arm")
  write_raw_keys "$raw" "$@" || return 1
  started=$SECONDS
  export E209_ARM=$arm
  [ "$arm" != twin ] || E209_ARM=head
  if broken progress-not-forwarded; then
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E209 历史用例; 范围=1 个测试目标; 为什么=量固定点逐轮读数，只跑这一趟判定库里没有的项' \
      cargo test --release -p singlefs-verification --test e209_spike_cause_histories -- "${filters[@]}" "${exact[@]}" --nocapture --test-threads="$test_threads" > "$raw" 2>&1
  else
    SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E209 历史用例; 范围=1 个测试目标; 为什么=量固定点逐轮读数，只跑这一趟判定库里没有的项' \
      cargo test --release -p singlefs-verification --test e209_spike_cause_histories -- "${filters[@]}" "${exact[@]}" --nocapture --test-threads="$test_threads" 2>&1 | tee "$raw" | grep --line-buffered '^e209 progress'
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
  place_the_histories "$tests_directory" || return 1
  plan_file="$work/$kind-plan.json"
  plan_output="$work/$kind-plan.out"
  for item in "$@"; do item_arguments+=(--item "$item"); done
  python3 -I "$library" plan --copy "$copy" --library-root "$repo" --plan "$plan_file" "${item_arguments[@]}" > "$plan_output"
  plan_code=$?
  cat "$plan_output"
  [ "$plan_code" -eq 0 ] || { echo "name=run_done build=$kind ran=0 failed=1 plan_exit=$plan_code"; return "$plan_code"; }
  while read -r tag item_field decision_field key_field; do
    [ "$tag" = E209PLAN ] || continue
    item_decision[${item_field#item=}]=${decision_field#decision=}
    item_key[${item_field#item=}]=${key_field#key=}
  done < "$plan_output"
  for item in "$@"; do
    if [ -z "${item_decision[$item]:-}" ]; then
      echo "  ✗ 判定库那一份的计划里没有项 $item（$plan_output），不知道它该复用还是现判"
      echo "     → 怎么办：看 $plan_output 里 E209PLAN 那几行少了哪一项、e209_spike_cause_library.py plan 为什么没打它；修好整条命令重跑"
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
    for item in "${todo[@]}"; do rm -f "$repo"/research/results/.e209-spike-cause-"${item//__/-}"-*.partial; done
    cd "$copy" || return 1
    raw_directory="$copy/.e209-raw"
    mkdir -p "$raw_directory" || return 1
    harvest_finished_items
    for item in ${todo[@]+"${todo[@]}"}; do
      arm=$(group_of "$item")
      broken one-process-for-every-arm && arm=$(group_of "${todo[0]}")
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
    if [ "$(arm_of_item "$item")" = plain ]; then plain_items+=("$item"); else counts_items+=("$item"); fi
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

# 入口的主体：整项整趟在这一台跑，建副本与编译。
dispatch() {
  # 脚本永远不报自己要多少内存
  run_flow
}

# ---- 自证：假驱动（cargo 换成读模式文件的函数，计数脚本跑真的、拷一棵小的假仓根；算键与判定库跑真的，判定库是临时 prune-server），不编译、不跑实现 ----

# 模拟这一趟被系统杀掉：点名两个进程，不在循环里发；假 cargo 跑在管道里，先发 run_flow 那个、再发自己。
simulate_this_run_being_killed() {
  kill -KILL "${run_flow_shell:?}"
  kill -KILL "$BASHPID"
}

# 假 cargo：把参数记进 calls.log、把 E209_ARM 记进 arms.log；按 $world_directory/mode 决定怎么跑。点名的 item_<项> 各打 row 与 item_done 两行。
#   ok 全跑完；rows-differ 全跑完、row 那一行的读数换一个；kill-immediately 一开始就被杀；kill-after-first 第一项跑完、第二项打一行就被杀；
#   kill-cut-line 打出一行被截断的 item_done 就被杀。
fake_cargo() {
  local mode filter name index=0 filters=() argument seen_separator=0 reading=1
  echo "$*" >> "$calls_log"
  echo "${E209_ARM:-unset}" >> "$world_directory/arms.log"
  mode=$(cat "$world_directory/mode")
  [ "$mode" = rows-differ ] && reading=2
  for argument in "$@"; do
    if [ "$seen_separator" -eq 1 ]; then case "$argument" in --*) ;; *) filters+=("$argument") ;; esac; fi
    [ "$argument" = "--" ] && seen_separator=1
  done
  printf 'e209 progress item=fake publishes_done=1 of=2 peak_rounds=1 peak_W=1\n'
  [ "$mode" = kill-immediately ] && simulate_this_run_being_killed
  for filter in "${filters[@]}"; do
    name=${filter#item_}
    index=$((index + 1))
    if [ "$mode" = kill-cut-line ]; then
      printf 'E209RESULT name=item_done item=%s' "${name%??}"
      simulate_this_run_being_killed
    fi
    # 单线程时 libtest 把「test <用例名> ... 」接在第一行输出前面（不换行）
    case " $* " in *" --test-threads=1 "*) printf 'test %s ... ' "$filter" ;; esac
    printf 'E209RESULT name=row item=%s reading=%s\n' "$name" "$reading"
    if [ "$mode" = kill-after-first ] && [ "$index" -eq 2 ]; then simulate_this_run_being_killed; fi
    printf 'E209RESULT name=item_done item=%s\n' "$name"
  done
  return 0
}

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
fn item_unlinks_1__head__walk() {
    assert_eq!(shared_history_of_the_small_items(), 2);
}

#[test]
fn item_unlinks_100__head__walk() {
    assert_eq!(shared_history_of_the_small_items() + 1, 3);
}

#[test]
fn item_unlinks_1000__head__walk() {
    assert_eq!(history_of_the_large_item(), 3);
}

#[test]
fn item_unlinks_1__control__walk() {
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
  -V) echo "rustc 0.0.0-e209-selftest" ;;
  -vV) printf 'rustc 0.0.0-e209-selftest\nbinary: rustc\nhost: x86_64-unknown-linux-gnu\nrelease: 0.0.0-e209-selftest\n' ;;
esac
exit 0
EOF
  cat > "$tools/cargo" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  -V|--version) echo "cargo 0.0.0-e209-selftest" ;;
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
  world_directory=$1
  mkdir -p "$world_directory/repo/research/results" "$world_directory/src" || return 1
  repo="$world_directory/repo"
  histories="$world_directory/histories.rs"
  write_selftest_histories "$histories" || return 1
  source_tree="$world_directory/src"
  python3 -I "$counters" --write-fake-source "$source_tree" || return 1
  write_selftest_workspace "$source_tree" || return 1
  write_version_only_toolchain "$world_directory/tools" || return 1
  export PATH="$world_directory/tools:$PATH" CARGO_HOME="$world_directory/tools/cargo-home" SINGLEFS_REUSE_SENTINEL_COUNT=1000
  unset SINGLEFS_REUSE_SENTINEL_SEED
  work="$world_directory/work"
  : > "$world_directory/compile.log"
  calls_log="$world_directory/calls.log"
  : > "$calls_log"
  : > "$world_directory/arms.log"
  set_mode ok
}

set_mode() { printf '%s\n' "$1" > "$world_directory/mode"; }

# 跑一遍入口主体（子 shell：被杀只杀这一遍）。输出进 $world_directory/last.out，退出码进 last_exit_code。
go() {
  chosen=("$@")
  { ( run_flow ) > "$world_directory/last.out" 2>&1; last_exit_code=$?; } 2>/dev/null
}

calls() { wc -l < "$calls_log" | tr -d ' '; }
products() { find "$repo/research/results" -maxdepth 1 -name "e209-spike-cause-${1//__/-}-*.out" | wc -l | tr -d ' '; }
latest_product() { find "$repo/research/results" -maxdepth 1 -name "e209-spike-cause-${1//__/-}-*.out" -printf '%T@ %p\n' | sort -n | tail -n 1 | cut -d' ' -f2-; }

stored_in_library() {
  local arguments=() item
  for item in "$@"; do arguments+=(--item "$item"); done
  python3 -I "$library" stored --copy "$work/counts" --library-root "$repo" "${arguments[@]}" > "$world_directory/stored.out" 2>&1 || { cat "$world_directory/stored.out"; return 1; }
}

plan_directly() {
  local arguments=() item
  for item in "$@"; do arguments+=(--item "$item"); done
  python3 -I "$library" plan --copy "$work/counts" --library-root "$repo" --plan "$world_directory/direct-plan.json" "${arguments[@]}" > "$world_directory/direct-plan.out" 2>&1 \
    || { echo "plan 该退 0"; cat "$world_directory/direct-plan.out"; return 1; }
}
planned_sentinels() { sed -n 's/^E209SENTINEL .* sentinels=\([^ ]*\) forced=.*$/\1/p' "$1"; }
planned_forced() { sed -n 's/^E209SENTINEL .* forced=\([^ ]*\)$/\1/p' "$1"; }
finish_directly() {
  python3 -I "$library" finish --plan "$world_directory/direct-plan.json" > "$world_directory/finish.out" 2>&1
  finish_exit_code=$?
}

case_killed_after_the_copy_and_run_again_completes() {
  set_mode kill-immediately
  go unlinks_1__head__walk unlinks_100__head__walk
  [ "$last_exit_code" -eq 137 ] || { echo "第一遍该被杀（137），得 $last_exit_code"; return 1; }
  [ -f "$work/counts/.e203-copy-done" ] || { echo "被杀时副本该已拷完、有完成标记"; return 1; }
  set_mode ok
  go unlinks_1__head__walk unlinks_100__head__walk
  [ "$last_exit_code" -eq 0 ] || { echo "第二遍该跑完（0），得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  grep -q 'name=run_tally judged=2 forced=0 reused=0 refused=0 not_done=0 not_run=0' "$world_directory/last.out" || { echo "计账行不对"; cat "$world_directory/last.out"; return 1; }
  stored_in_library unlinks_1__head__walk unlinks_100__head__walk || { echo "两项该照键进了判定库"; return 1; }
}

case_a_directory_that_is_not_our_copy_is_refused_and_left_alone() {
  mkdir -p "$work/counts" && printf 'mine\n' > "$work/counts/mine.txt" || return 1
  go unlinks_1__head__walk
  [ "$last_exit_code" -eq 2 ] || { echo "该拒绝（2），得 $last_exit_code"; return 1; }
  [ -f "$work/counts/mine.txt" ] || { echo "不是我们拷的东西不许清"; return 1; }
  [ "$(calls)" -eq 0 ] || { echo "拒绝时不许起 cargo"; return 1; }
}

case_one_finished_item_is_not_run_again() {
  go unlinks_1__head__walk unlinks_100__head__walk
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] && [ "$(products unlinks_1__head__walk)" -eq 1 ] || { echo "第一遍该跑一次 cargo、各一份产物"; cat "$world_directory/last.out"; return 1; }
  go unlinks_1__head__walk unlinks_100__head__walk
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "第二遍不该再起 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  grep -q 'name=run_tally judged=0 forced=0 reused=2 refused=0' "$world_directory/last.out" || { echo "第二遍该现判 0、复用 2"; cat "$world_directory/last.out"; return 1; }
  [ "$(products unlinks_1__head__walk)" -eq 1 ] || { echo "复用的项不另写产物"; return 1; }
}

case_an_item_finished_inside_a_killed_run_is_taken_and_not_run_again() {
  set_mode kill-after-first
  go unlinks_1__head__walk unlinks_100__head__walk
  [ "$last_exit_code" -eq 137 ] || { echo "第一遍该被杀，得 $last_exit_code"; return 1; }
  set_mode ok
  go unlinks_1__head__walk unlinks_100__head__walk
  [ "$last_exit_code" -eq 0 ] || { echo "第二遍该跑完，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  [ "$(calls)" -eq 2 ] || { echo "该共起两次 cargo"; return 1; }
  ! sed -n 2p "$calls_log" | grep -q 'item_unlinks_1__head__walk ' || { echo "第二次不该再点名跑完的那一项"; sed -n 2p "$calls_log"; return 1; }
  sed -n 2p "$calls_log" | grep -q 'item_unlinks_100__head__walk' || { echo "第二次该点名没跑完的那一项"; return 1; }
  stored_in_library unlinks_1__head__walk unlinks_100__head__walk || { echo "两项该照键进了判定库（取来的那一项也入库）"; return 1; }
}

case_an_item_finished_inside_a_killed_run_under_another_key_is_run_again() {
  set_mode kill-after-first
  go unlinks_1__head__walk unlinks_100__head__walk
  [ "$last_exit_code" -eq 137 ] || { echo "第一遍该被杀，得 $last_exit_code"; return 1; }
  replace_in_file "$histories" 'scan_step() + 1' 'scan_step() + 3' || return 1
  set_mode ok
  go unlinks_1__head__walk unlinks_100__head__walk
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "第二遍该跑完、共起两次 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  sed -n 2p "$calls_log" | grep -q 'item_unlinks_1__head__walk ' && sed -n 2p "$calls_log" | grep -q 'item_unlinks_100__head__walk' \
    || { echo "键变了，第二次该把两项都点名"; sed -n 2p "$calls_log"; return 1; }
}

case_one_line_cut_by_a_kill_does_not_finish_an_item() {
  set_mode kill-cut-line
  go unlinks_1__head__walk
  [ "$last_exit_code" -eq 137 ] || { echo "第一遍该被杀，得 $last_exit_code"; return 1; }
  set_mode ok
  # 残行的项名少了末两个字符；项名短一截时它正好是别的项名，这里手写一份完整项名的残行
  printf 'E209RESULT name=item_done item=unlinks_1__head__walk' > "$work/counts/.e209-raw/head-9.out"
  printf 'unlinks_1__head__walk\t%s\n' "$(python3 -I "$library" keys --copy "$work/counts" --item unlinks_1__head__walk | sed -n 's/.* key=\([^ ]*\).*/\1/p')" > "$work/counts/.e209-raw/head-9.out.keys"
  go unlinks_1__head__walk
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "残行不算跑完，第二遍该再起 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
}

case_the_item_filter_names_one_test_only() {
  go unlinks_1__head__walk
  sed -n 1p "$calls_log" | grep -q -- ' --exact ' || { echo "点名该带 --exact"; sed -n 1p "$calls_log"; return 1; }
}

case_one_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name() {
  { ( header_for() { kill -KILL "$BASHPID"; }; chosen=(unlinks_1__head__walk); run_flow ) > "$world_directory/last.out" 2>&1; last_exit_code=$?; } 2>/dev/null
  [ "$last_exit_code" -eq 137 ] || { echo "第一遍该在写产物时被杀，得 $last_exit_code"; return 1; }
  [ "$(products unlinks_1__head__walk)" -eq 0 ] || { echo "被杀时产物名下不该有文件"; return 1; }
  go unlinks_1__head__walk
  [ "$last_exit_code" -eq 0 ] || { echo "第二遍该跑完，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  [ "$(calls)" -eq 1 ] || { echo "原始输出已跑完，第二遍不该再起 cargo（calls=$(calls)）"; return 1; }
  [ "$(products unlinks_1__head__walk)" -eq 1 ] || { echo "该恰好一份产物"; return 1; }
  [ -z "$(find "$repo/research/results" -maxdepth 1 -name '.*.partial')" ] || { echo "残留的隐藏 .partial 该已清掉"; return 1; }
}

case_progress_lines_reach_the_standard_output() {
  go unlinks_1__head__walk
  grep -q '^e209 progress item=fake publishes_done=1 of=2 peak_rounds=1 peak_W=1$' "$world_directory/last.out" || { echo "进度行该转到标准输出"; cat "$world_directory/last.out"; return 1; }
  product=$(latest_product unlinks_1__head__walk)
  ! grep -q '^e209 progress' "$product" || { echo "进度行不进产物"; return 1; }
}

case_the_product_header_records_the_tested_files() {
  go unlinks_1__head__walk
  product=$(latest_product unlinks_1__head__walk)
  [ "$(grep -c '^E209RESULT name=source_sha256 ' "$product")" -eq 3 ] || { echo "产物头该记三份被测文件的 sha256"; cat "$product"; return 1; }
  grep -q '^E209RESULT name=header for_item=unlinks_1__head__walk ' "$product" || { echo "产物头该有 header 行"; cat "$product"; return 1; }
}

case_each_arm_runs_in_its_own_process() {
  go unlinks_1__head__walk unlinks_1__control__walk
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "两条臂该起两次 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  [ "$(sed -n 1p "$world_directory/arms.log")" = head ] && [ "$(sed -n 2p "$world_directory/arms.log")" = control ] || { echo "每次的 E209_ARM 该是那条臂"; cat "$world_directory/arms.log"; return 1; }
  sed -n 1p "$calls_log" | grep -q 'item_unlinks_1__head__walk ' && ! sed -n 1p "$calls_log" | grep -q 'item_unlinks_1__control__walk' || { echo "一次 cargo 只点名一条臂的项"; cat "$calls_log"; return 1; }
}

case_the_plain_item_goes_to_a_copy_without_the_counters() {
  go unlinks_1__plain__walk
  [ -f "$work/plain/.e203-copy-done" ] || { echo "plain 该有自己的副本"; cat "$world_directory/last.out"; return 1; }
  ! grep -q e207_strict_scan_steps "$work/plain/crates/singlefs-core/src/transaction.rs" || { echo "plain 的副本里不该有读数"; return 1; }
  grep -q 'pub fn build_name' "$work/plain/crates/singlefs-verification/tests/e209_readings/mod.rs" || { echo "plain 的副本要有读数的空实现"; return 1; }
}

# 复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：头一趟库是空的、同一份输入、闭包外改动、闭包内改动。
case_reuse_three_questions() {
  local runs="$world_directory/three-questions"
  mkdir -p "$runs" || return 1
  go unlinks_1__head__walk unlinks_100__head__walk unlinks_1000__head__walk
  cp "$world_directory/last.out" "$runs/first.out" || return 1
  go unlinks_1__head__walk unlinks_100__head__walk unlinks_1000__head__walk
  cp "$world_directory/last.out" "$runs/second.out" || return 1
  printf '\n// 历史用例里加的一行注释\nfn spare_history_nobody_runs() -> u64 {\n    9\n}\n' >> "$histories" || return 1
  printf '// 源码变了的一行\n' >> "$source_tree/crates/singlefs-core/src/transaction.rs" || return 1
  go unlinks_1__head__walk unlinks_100__head__walk unlinks_1000__head__walk
  cp "$world_directory/last.out" "$runs/unrelated.out" || return 1
  replace_in_file "$histories" 'scan_step() + 1' 'scan_step() + 3' || return 1
  go unlinks_1__head__walk unlinks_100__head__walk unlinks_1000__head__walk
  cp "$world_directory/last.out" "$runs/unit-change.out" || return 1
  python3 -I "$here/../gate/lib_reuse_checks.py" three-questions --script "$entry_script_key" --first "$runs/first.out" --second "$runs/second.out" \
    --unrelated "$runs/unrelated.out" --unit-change "$runs/unit-change.out" --affected unlinks_1__head__walk,unlinks_100__head__walk --from-output
}

# 键不随副本的路径变：同一份副本拷到两个别的目录，键逐字相同；工作目录换到别处，库里判过的照键复用。
case_the_item_key_does_not_depend_on_the_copy_path() {
  local one="$world_directory/one/place" another="$world_directory/another/place/deeper"
  go unlinks_1__head__walk unlinks_1000__head__walk
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "头一遍该跑完、起一次 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  mkdir -p "$one" "$another" && cp -r "$work/counts/." "$one/" && cp -r "$work/counts/." "$another/" || return 1
  python3 -I "$library" keys --copy "$one" --item unlinks_1__head__walk --item unlinks_1000__head__walk > "$world_directory/keys-one.out" 2>&1 || { cat "$world_directory/keys-one.out"; return 1; }
  python3 -I "$library" keys --copy "$another" --item unlinks_1__head__walk --item unlinks_1000__head__walk > "$world_directory/keys-another.out" 2>&1 || { cat "$world_directory/keys-another.out"; return 1; }
  [ "$(grep -c '^E209KEY ' "$world_directory/keys-one.out")" -eq 2 ] && ! grep -q 'key=refused' "$world_directory/keys-one.out" || { echo "两项都该算得出键"; cat "$world_directory/keys-one.out"; return 1; }
  cmp -s "$world_directory/keys-one.out" "$world_directory/keys-another.out" || { echo "同一份源码放两个目录，键该相同"; cat "$world_directory/keys-one.out" "$world_directory/keys-another.out"; return 1; }
  work="$world_directory/elsewhere/work"
  go unlinks_1__head__walk unlinks_1000__head__walk
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "换了副本的路径，该照键复用、不再起 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  grep -q 'name=run_tally judged=0 forced=0 reused=2 refused=0' "$world_directory/last.out" || { echo "该现判 0、复用 2"; cat "$world_directory/last.out"; return 1; }
}

case_reuse_sentinel_draws_and_the_forced_item_is_run_again() {
  local forced
  go unlinks_1__head__walk unlinks_100__head__walk unlinks_1000__head__walk
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "头一遍该跑完（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  go unlinks_1__head__walk unlinks_100__head__walk unlinks_1000__head__walk
  [ "$last_exit_code" -eq 0 ] || { echo "哨兵复用、强制现判与库里相同，该退 0，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  forced=$(planned_forced "$world_directory/last.out")
  [ -n "$forced" ] && [ "$forced" != - ] || { echo "三项键相同、哨兵 1 条，该抽出一条强制现判"; cat "$world_directory/last.out"; return 1; }
  [ "$(calls)" -eq 2 ] && [ "$(sed -n 2p "$calls_log" | grep -o 'item_[a-z0-9_]*' | wc -l)" -eq 1 ] && sed -n 2p "$calls_log" | grep -q "item_$forced " \
    || { echo "第二遍该只起一次 cargo、只点名强制现判的 $forced"; cat "$calls_log"; return 1; }
  grep -q 'name=run_tally judged=1 forced=1 reused=2 refused=0 not_done=0 not_run=0' "$world_directory/last.out" || { echo "计账行不对"; cat "$world_directory/last.out"; return 1; }
}

case_one_rejudged_sentinel_turns_the_run_red() {
  local sentinel forced
  go unlinks_1__head__walk unlinks_100__head__walk unlinks_1000__head__walk
  [ "$last_exit_code" -eq 0 ] || { echo "头一遍该跑完，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  plan_directly unlinks_1__head__walk unlinks_100__head__walk unlinks_1000__head__walk || return 1
  sentinel=$(planned_sentinels "$world_directory/direct-plan.out")
  forced=$(planned_forced "$world_directory/direct-plan.out")
  [ -n "$sentinel" ] && [ "$sentinel" != - ] && [ -n "$forced" ] && [ "$forced" != - ] || { echo "该抽出一条哨兵、一条强制现判"; cat "$world_directory/direct-plan.out"; return 1; }
  python3 -I "$library" record --plan "$world_directory/direct-plan.json" --item "$sentinel" --product "$(latest_product "$sentinel")" || return 1
  python3 -I "$library" record --plan "$world_directory/direct-plan.json" --item "$forced" --product "$(latest_product "$forced")" || return 1
  finish_directly
  [ "$finish_exit_code" -eq 1 ] && grep -q '复用失灵' "$world_directory/finish.out" || { echo "哨兵被现判，该退 1、报复用失灵（得 $finish_exit_code）"; cat "$world_directory/finish.out"; return 1; }
}

case_one_forced_rejudge_with_another_verdict_turns_the_run_red() {
  go unlinks_1__head__walk unlinks_100__head__walk unlinks_1000__head__walk
  [ "$last_exit_code" -eq 0 ] || { echo "头一遍该跑完，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  set_mode rows-differ
  go unlinks_1__head__walk unlinks_100__head__walk unlinks_1000__head__walk
  [ "$last_exit_code" -eq 1 ] && grep -q '判定不稳' "$world_directory/last.out" || { echo "强制现判的读数与库里不同，该退 1、报判定不稳（得 $last_exit_code）"; cat "$world_directory/last.out"; return 1; }
}

case_an_item_planned_for_judging_but_not_run_turns_the_run_red() {
  go unlinks_1000__head__walk
  [ "$last_exit_code" -eq 0 ] || { echo "头一遍该跑完，得 $last_exit_code"; cat "$world_directory/last.out"; return 1; }
  plan_directly unlinks_1__head__walk unlinks_100__head__walk || return 1
  finish_directly
  [ "$finish_exit_code" -eq 1 ] && grep -q '没跑到' "$world_directory/finish.out" || { echo "要现判的两项一条都没记，该退 1、报没跑到（得 $finish_exit_code）"; cat "$world_directory/finish.out"; return 1; }
}

case_an_item_without_a_key_runs_and_is_never_recorded() {
  # 假历史用例里没有 item_files_1__head__walk：入口找不到，拒算
  go files_1__head__walk
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "拒算的项照跑、退 0（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  grep -q 'E209PLAN item=files_1__head__walk decision=refused key=-' "$world_directory/last.out" && grep -q 'name=run_tally judged=0 forced=0 reused=0 refused=1' "$world_directory/last.out" \
    || { echo "该报拒算 1"; cat "$world_directory/last.out"; return 1; }
  go files_1__head__walk
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "拒算的项不入库，下一趟照跑（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
}

case_an_unreachable_library_exits_75_and_runs_nothing() {
  local port
  port=$(python3 -c 'import socket; listener = socket.socket(); listener.bind(("127.0.0.1", 0)); print(listener.getsockname()[1]); listener.close()') || return 1
  export SINGLEFS_PRUNE_SERVER_SOCKET="$world_directory/no-server.sock" SINGLEFS_PRUNE_SERVER_TCP_ADDRESS="127.0.0.1:$port" SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS=2
  go unlinks_1__head__walk
  [ "$last_exit_code" -eq 75 ] || { echo "判定库连不上该退 75，得 $last_exit_code"; return 1; }
  [ "$(calls)" -eq 0 ] || { echo "连不上时不许起 cargo（不当库里没有现判）"; return 1; }
}

# 孪生项（登记第七节 ③ 第 5 条）：另起一次 cargo、点名原项的用例、只一个测试线程，产物头记 twin=yes 与原项名。
case_the_twin_item_runs_in_another_process_with_one_test_thread() {
  go unlinks_1__head__walk unlinks_1__head__walk__twin
  [ "$last_exit_code" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "孪生项该另起一次 cargo（calls=$(calls) rc=$last_exit_code）"; cat "$world_directory/last.out"; return 1; }
  sed -n 1p "$calls_log" | grep -q 'item_unlinks_1__head__walk ' && ! sed -n 1p "$calls_log" | grep -q '__twin' || { echo "原项那一次不点名孪生项"; cat "$calls_log"; return 1; }
  sed -n 2p "$calls_log" | grep -q 'item_unlinks_1__head__walk ' && sed -n 2p "$calls_log" | grep -q -- '--test-threads=1' || { echo "孪生项那一次点名原项的用例、只一个测试线程"; cat "$calls_log"; return 1; }
  [ "$(sed -n 1p "$world_directory/arms.log")" = head ] && [ "$(sed -n 2p "$world_directory/arms.log")" = head ] || { echo "两次的 E209_ARM 都该是 head"; cat "$world_directory/arms.log"; return 1; }
  product=$(latest_product unlinks_1__head__walk__twin)
  grep -q '^E209RESULT name=header for_item=unlinks_1__head__walk twin=yes ' "$product" || { echo "孪生项的产物头该记 twin=yes、for_item 是原项"; cat "$product"; return 1; }
  grep -q ' test_threads=1 ' "$product" || { echo "孪生项的产物头该记它真用的 test_threads=1"; cat "$product"; return 1; }
  grep -q '^E209RESULT name=row item=unlinks_1__head__walk ' "$product" || { echo "孪生项的第一行输出（单线程时前面有 libtest 的 test … ... 前缀）不许丢"; cat "$product"; return 1; }
}

selftest() {
  local failures=0 out name display reuse_line="" line real_histories=$histories real_entry=$0
  selftest_root=$(mktemp -d "${TMPDIR:-/tmp}/e209-run-selftest.XXXXXX") || exit 1
  if [ "${SINGLEFS_KEEP_TEST_TEMPORARY_FILES:-}" = 1 ]; then
    echo "name=selftest_kept path=$selftest_root"
  else
    trap 'rm -rf "$selftest_root"' EXIT
  fi
  # 判定库那一份只在有这份标记文件时认弄坏开关
  printf 'e209-selftest\n' > "$selftest_root/mark"
  export E209_SELFTEST_MARK="$selftest_root/mark"
  # 自证只在临时 prune-server 上跑：经判定库那一份调客户端 selftest_service_problem，不是临时服务退 2、一格不跑
  python3 -I "$library" selftest-service-problem || return 2
  cargo() { fake_cargo "$@"; }
  out=$(items_table | wc -l)
  [ "$out" -eq 79 ] || { echo "name=selftest_case case=item_count result=FAILED got=$out"; failures=$((failures + 1)); }
  [ "$(items_table | awk '$2=="first"' | wc -l)" -eq 7 ] || { echo "name=selftest_case case=first_segment_items result=FAILED"; failures=$((failures + 1)); }
  if [ "$(items_of_segment third | wc -l)" -eq 48 ] && items_of_segment third | grep -qx files_200_units_100__head__walk_forward; then
    echo "name=selftest_case case=third_segment_holds_the_gate_item result=ok"
  else
    echo "name=selftest_case case=third_segment_holds_the_gate_item result=FAILED"; failures=$((failures + 1))
  fi
  if [ "$(items_table | cut -d' ' -f1 | grep -v '__twin$' | sort)" = "$(sed -n 's/^fn item_\(.*\)() {$/\1/p' "$real_histories" | sort)" ]; then
    echo "name=selftest_case case=the_table_matches_the_histories result=ok"
  else
    echo "name=selftest_case case=the_table_matches_the_histories result=FAILED"
    failures=$((failures + 1))
  fi
  local product_file="$selftest_root/product"
  printf 'E209RESULT name=row reading=1\n' > "$product_file"
  if product_is_done "$product_file"; then echo "name=selftest_case case=done_marker result=FAILED"; failures=$((failures + 1)); else echo "name=selftest_case case=one_product_without_the_marker_is_not_done result=ok"; fi
  printf 'E209RESULT name=item_done item=base__head\n' >> "$product_file"
  if product_is_done "$product_file"; then echo "name=selftest_case case=one_product_with_the_marker_is_done result=ok"; else echo "name=selftest_case case=done_marker result=FAILED"; failures=$((failures + 1)); fi
  if E209_RUN_BREAK='' bash "$real_entry" --item no_such_item__head --work "$selftest_root/unused" > /dev/null 2>&1; then
    echo "name=selftest_case case=an_unknown_item_name_exits_2 result=FAILED"; failures=$((failures + 1))
  else
    echo "name=selftest_case case=an_unknown_item_name_exits_2 result=ok"
  fi
  if [ "$(E209_RUN_BREAK='' bash "$real_entry" --list-items | wc -l)" -eq 79 ]; then
    echo "name=selftest_case case=list_items_lists_every_item result=ok"
  else
    echo "name=selftest_case case=list_items_lists_every_item result=FAILED"; failures=$((failures + 1))
  fi
  # 各格各造各的小世界（自己的假仓根、工作目录、调用记录，判定库命名空间按各自的仓根分），互不依赖：按每台默认重活线程数并行起，结果按下面的固定次序打。
  local threads running=0 cases=(
    case_killed_after_the_copy_and_run_again_completes
    case_a_directory_that_is_not_our_copy_is_refused_and_left_alone
    case_one_finished_item_is_not_run_again
    case_an_item_finished_inside_a_killed_run_is_taken_and_not_run_again
    case_an_item_finished_inside_a_killed_run_under_another_key_is_run_again
    case_one_line_cut_by_a_kill_does_not_finish_an_item
    case_the_item_filter_names_one_test_only
    case_one_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name
    case_progress_lines_reach_the_standard_output
    case_the_product_header_records_the_tested_files
    case_each_arm_runs_in_its_own_process
    case_the_plain_item_goes_to_a_copy_without_the_counters
    case_reuse_three_questions
    case_the_item_key_does_not_depend_on_the_copy_path
    case_reuse_sentinel_draws_and_the_forced_item_is_run_again
    case_one_rejudged_sentinel_turns_the_run_red
    case_one_forced_rejudge_with_another_verdict_turns_the_run_red
    case_an_item_planned_for_judging_but_not_run_turns_the_run_red
    case_an_item_without_a_key_runs_and_is_never_recorded
    case_an_unreachable_library_exits_75_and_runs_nothing
    case_the_twin_item_runs_in_another_process_with_one_test_thread
  )
  # shellcheck source=../hosts/heavy-threads.sh
  source "$here/../hosts/heavy-threads.sh"
  threads=$(heavy_thread_default) || return 2
  for name in "${cases[@]}"; do
    mkdir -p "$selftest_root/$name"
    (
      if ( world "$selftest_root/$name" && "case_${name#case_}" ) > "$selftest_root/$name/out.txt" 2>&1; then echo ok > "$selftest_root/$name/status"; else echo FAILED > "$selftest_root/$name/status"; fi
    ) &
    running=$((running + 1))
    if [ "$running" -ge "$threads" ]; then wait -n; running=$((running - 1)); fi
  done
  wait
  for name in "${cases[@]}"; do
    display=${name#case_}
    [ "$display" = reuse_three_questions ] && display=reuse-three-questions
    if [ "$(cat "$selftest_root/$name/status" 2>/dev/null)" = ok ]; then
      echo "name=selftest_case case=$display result=ok"
    else
      echo "name=selftest_case case=$display result=FAILED"
      sed 's/^/  | /' "$selftest_root/$name/out.txt" | head -n 40
      failures=$((failures + 1))
    fi
    line=$(grep '^REUSE_SELFTEST ' "$selftest_root/$name/out.txt" | tail -n 1)
    [ -z "$line" ] || reuse_line=$line
  done
  [ -z "$reuse_line" ] || echo "$reuse_line"
  echo "name=selftest_done failures=$failures 并行 $threads 路（${#cases[@]} 格）"
  [ "$failures" -eq 0 ]
}

# 弄坏开关只在 --selftest 里认；别处设了拒（判定库那一份由自证设的标记文件放行，见 selftest）。
if [ -n "${E209_RUN_BREAK:-}" ] && [ "${1:-}" != --selftest ]; then
  echo "  ✗ E209_RUN_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "     → 怎么办：去掉 E209_RUN_BREAK 再跑；要证红就跑 E209_RUN_BREAK=<项> bash $0 --selftest（经临时 prune-server 的包装）" >&2
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
  items_table | awk -v wanted="$item" '$1==wanted {found=1} END {exit !found}' || { echo "项名写错：$item；可点名的项：" >&2; items_table >&2; exit 2; }
done
[ -n "$work" ] || usage
[ -f "$counters" ] && [ -f "$library" ] && [ -f "$histories" ] && [ -f "$scripts_module" ] || { echo "前置文件不在：$counters、$library、$histories 或 $scripts_module" >&2; exit 78; }

if [ -n "$segment" ]; then
  while read -r item; do chosen+=("$item"); done < <(items_of_segment "$segment")
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
dispatch
exit $?
