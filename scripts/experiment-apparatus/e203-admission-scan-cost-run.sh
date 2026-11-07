#!/usr/bin/env bash
# E203（准入之前扫类的步代价）量的那一路的入口：给仓副本套上计数或计时，起历史用例，产物按点分文件写进 research/results/，每个点的判定照单元键进 prune-server 那一份判定库。
# 跑前登记 research/prompts/e203-preregistration.md 第五节、第十四节。
#
# 用法：
#   e203-admission-scan-cost-run.sh --list-items
#   e203-admission-scan-cost-run.sh --build counts|timing --work <工作目录> (--segment first|second|third | --item <点>…) [--source <仓根>]
#   e203-admission-scan-cost-run.sh --selftest        核入口自己的列项、拒绝、完成标记、续跑、按键复用与复用哨兵，用假驱动，不编译、不跑实现；
#                                                     只在临时 prune-server 上跑（开跑先经判定库那一份调客户端 selftest_service_problem，不是临时服务退 2、一格不跑）
# 判定库（.claude/rules/verification.md「判定库只有一份」；键、库与哨兵的写法在同目录 e203_admission_scan_cost_library.py 文件头「键」「判定库」「复用哨兵」）：
#   每个点（计数、计时两个构建各一）一条判定，键是副本里历史用例 point_<点> 的单元闭包键（scripts/gate/admission.py 的 compute_unit_key）加工具链行、
#   闭包真引到的依赖行、构建种类与跑法；副本在哪个绝对路径不进键，同一份源码在别处的副本上判的，这里照键复用。族 apparatus-items，种类 e203-admission-scan-cost-run。
#   research/results/ 里的产物不当判定读，也不导入：库里没有的点照键现判一次。
#   开跑：副本拷好、历史用例放进副本之后 plan——库里有键相同、做完了的点复用（不跑、不另写产物），抽复用哨兵与一条强制现判（种子打进日志），其余现判，
#   拒算的点照跑、不入库；每个点的产物写好、回读过就 record（现判的当场照键写进库）；末尾 finish 判哨兵，成功句报「现判 N、复用 M、拒算 R」。
# 同一条命令重起就是续跑（被内存上限杀、手动停都一样）：
#   副本：工作目录里的副本先写开拷标记、拷完改完才写完成标记；半拷的清掉重拷，拷完且输入键相同的原样用，
#         非空而不是本脚本拷的目录拒绝（退 2）、一个字节不清。
#   点：库里有键相同、做完了的判定的点不重跑。上一次尝试的原始输出（副本里 .e203-raw 下；旁边同名加 .keys 的一份记着起 cargo 时每个点的键）里
#       已经完整跑完、键又与此刻相同的现判点，直接写成产物、照键入库、不重跑；被杀时正在跑的点才重跑。原始输出只当被杀那一趟的中转，库里有了的点不再看它。
#   产物：先写同目录的隐藏 .partial 再换名，被杀时产物名下要么没有、要么是整份；做完而没有 item_done 的产物（点没跑出来）
#         不删不覆盖，重起时另写新文件名。
#   diff：内容相同的已有 diff 文件原样引用，不另写一份。
#
# run-condition: 无（不要 GPU、不要第二台）。计时构建不与别的重活同机（服务排）。
# 退出码：0 跑到的全绿；1 有点没跑出 item_done、复用哨兵红、或计划里要现判的点这一趟没记；2 用法错或项名写错或工作目录里有不是本脚本拷的东西；
#   3 prune-server 客户端别的错（配置、服务拒了帧）；75 prune-server 不可用（照客户端：不现判、不当「库里没有」）；
#   77 一个点都没跑到；78 准入条件不满足（仓根或前置脚本不在）。
# 弄坏开关（只给 --selftest 用，平常跑时不认）。逐项写：开关、弄坏哪一处、打开后哪几格该红（失败行句首 name=selftest_case case=<格名> result=FAILED）：
#   E203_RUN_BREAK=done-marker-ignored  完成标记不看（凡判「做完没有」的地方都当做完了）
#       done_marker
#   E203_RUN_BREAK=done-points-rerun  计划复用的点照样重跑（不看 plan 给的 decision）
#       a_finished_point_is_not_run_again、reuse-three-questions、the_point_key_does_not_depend_on_the_copy_path、
#       reuse_sentinel_draws_and_the_forced_point_is_run_again、timing_runs_one_cargo_per_point_and_skips_the_finished_one
#   E203_RUN_BREAK=harvest-off  不从上一次尝试的原始输出里取跑完的点
#       a_point_finished_inside_a_killed_run_is_taken_and_not_run_again、a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name
#   E203_RUN_BREAK=harvest-ignores-the-key  从原始输出取点时不核旁边 .keys 记的键（别的键跑出来的也取）
#       a_point_finished_inside_a_killed_run_under_another_key_is_run_again
#   E203_RUN_BREAK=cut-lines-kept  取点时不丢最后一行不带换行的残行
#       a_line_cut_by_a_kill_does_not_finish_a_point
#   E203_RUN_BREAK=product-written-in-place  产物直接写在产物名下（被杀时留下半份）
#       a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name
#   E203_RUN_BREAK=filter-by-substring  点名不带 --exact，point_files_1 会连带跑 files_100 等
#       the_point_filter_names_one_test_only
#   E203_RUN_BREAK=copy-resume-off  副本不认完成标记与开拷标记，非空一律拒（这一项在计数脚本里读，入口只是把环境带过去）
#       killed_after_the_copy_and_run_again_completes、a_half_copy_is_cleared_and_run_again_completes、a_finished_point_is_not_run_again、
#       a_point_finished_inside_a_killed_run_is_taken_and_not_run_again、a_line_cut_by_a_kill_does_not_finish_a_point、
#       a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name、timing_runs_one_cargo_per_point_and_skips_the_finished_one、
#       reuse-three-questions、reuse_sentinel_draws_and_the_forced_point_is_run_again、a_forced_rejudge_with_another_verdict_turns_the_run_red、
#       a_point_without_a_key_runs_and_is_never_recorded、a_point_finished_inside_a_killed_run_under_another_key_is_run_again
#   下面几项在判定库那一份里读（e203_admission_scan_cost_library.py 文件头「弄坏开关」，入口只是把环境带过去）：
#   E203_RUN_BREAK=library-lookup-ignored  决定复用那一路不读库（一律当不在；自证判「点做完了」的 stored 也经它问库，一律答不在）
#       a_finished_point_is_not_run_again、reuse-three-questions、the_point_key_does_not_depend_on_the_copy_path、
#       reuse_sentinel_draws_and_the_forced_point_is_run_again、timing_runs_one_cargo_per_point_and_skips_the_finished_one、
#       killed_after_the_copy_and_run_again_completes、a_half_copy_is_cleared_and_run_again_completes、a_point_finished_inside_a_killed_run_is_taken_and_not_run_again
#   E203_RUN_BREAK=record-skipped  跑完的点不往库里写（头一遍跑完库里一个点都没有，哨兵那几格抽不出哨兵与强制现判）
#       killed_after_the_copy_and_run_again_completes、a_half_copy_is_cleared_and_run_again_completes、a_finished_point_is_not_run_again、
#       a_point_finished_inside_a_killed_run_is_taken_and_not_run_again、reuse-three-questions、the_point_key_does_not_depend_on_the_copy_path、
#       reuse_sentinel_draws_and_the_forced_point_is_run_again、timing_runs_one_cargo_per_point_and_skips_the_finished_one、
#       a_rejudged_sentinel_turns_the_run_red、a_forced_rejudge_with_another_verdict_turns_the_run_red、a_forced_point_left_unrun_turns_the_run_red
#   E203_RUN_BREAK=key-with-copy-path  键另加副本根的绝对路径
#       the_point_key_does_not_depend_on_the_copy_path
#   E203_RUN_BREAK=key-by-whole-histories-file  键另加副本里整份历史用例的内容（改前的文件级键）
#       reuse-three-questions
#   E203_RUN_BREAK=sentinel-skipped  不抽复用哨兵与强制现判、跑完不判
#       reuse_sentinel_draws_and_the_forced_point_is_run_again、a_rejudged_sentinel_turns_the_run_red、
#       a_forced_rejudge_with_another_verdict_turns_the_run_red、a_forced_point_left_unrun_turns_the_run_red
#   E203_RUN_BREAK=sentinel-rejudge-unobserved  跑完把哨兵一律记成复用（这一趟跑了也不算现判）
#       a_rejudged_sentinel_turns_the_run_red
#   E203_RUN_BREAK=forced-read-as-reused  抽中强制现判的点照库复用
#       reuse_sentinel_draws_and_the_forced_point_is_run_again、a_forced_rejudge_with_another_verdict_turns_the_run_red
#   E203_RUN_BREAK=forced-unrun-unobserved  强制现判那一点没跑时不记它的结局（不判「强制现判没起作用」）
#       a_forced_point_left_unrun_turns_the_run_red
#   E203_RUN_BREAK=forced-verdict-unchecked  不比强制现判的判定与库里那一份
#       a_forced_rejudge_with_another_verdict_turns_the_run_red
#   E203_RUN_BREAK=refused-points-reused  拒算的点照复用跳过、不跑
#       a_point_without_a_key_runs_and_is_never_recorded
#   E203_RUN_BREAK=unreachable-as-empty  读库时 prune-server 连不上照「库里没有」现判
#       an_unreachable_library_exits_75_and_runs_nothing
#   E203_RUN_BREAK=unrun-points-unchecked  跑完不核计划里要现判的点这一趟记过没有
#       a_point_planned_for_judging_but_not_run_turns_the_run_red
set -u

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
counters="$here/e203_admission_scan_cost_counters.py"
library="$here/e203_admission_scan_cost_library.py"
histories="$here/e203_admission_scan_cost_histories.rs"
entry_script_key="scripts/experiment-apparatus/e203-admission-scan-cost-run.sh"   # 复用三问规范输出认的脚本名

# 点 档 段
items_table() {
  cat <<'EOF'
unit_area_2_pow_14 slow first
base slow first
unit_area_2_pow_20 slow first
unit_area_2_pow_24 slow first
unlinks_1 slow first
unlinks_100 slow first
unlinks_1000 slow first
unlinks_10000 slow first
data_units_1 slow second
data_units_100 slow second
data_units_1000 slow second
data_units_10000 slow second
files_1 slow second
files_100 slow second
files_1000 slow second
files_10000 slow second
diagonal slow second
many_unlinks_unit_area_2_pow_16 slow second
many_unlinks_unit_area_2_pow_17 slow second
user_scale_unit_area slow third
EOF
}
# Y1 两档（单元区 2¹⁶、2¹⁷，登记原来的 2¹⁴ 装不下 10⁵ 次删）各一项，只点名其中一档跑（--item）；--segment second 会两档都跑。

usage() { sed -n '2,/^set -u/p' "$0" | sed '$d' >&2; exit 2; }

broken() { [ "${E203_RUN_BREAK:-}" = "$1" ]; }

# 一份产物是否做完：有 item_done 行。弄坏开关打开时不看。
product_is_done() {
  broken done-marker-ignored && return 0
  grep -q '^E203RESULT name=item_done ' "$1" 2>/dev/null
}

# 历史用例的内容键（sha256 前 16 位）：只写进产物头记来历，判「做完没有」不看它（看判定库里的点键）。
histories_key() {
  sha256sum "$histories" | cut -c1-16
}

# 新文件名：<产物前缀>-<日期>.out，同一天再来加 -r2、-r3；已有的一个不动。
new_product_name() {
  local build=$1 item=$2 day name n=1
  day=$(TZ=Asia/Tokyo date +%F)
  name="$repo/research/results/e203-admission-scan-cost-$build-$item-$day.out"
  while [ -e "$name" ]; do
    n=$((n + 1))
    name="$repo/research/results/e203-admission-scan-cost-$build-$item-$day-r$n.out"
  done
  printf '%s\n' "$name"
}

# 一份原始输出里写完整的行（丢掉被杀时最后一行不带换行的残行）。
complete_lines() {
  if broken cut-lines-kept; then cat "$1"; else head -n "$(wc -l < "$1")" "$1"; fi
}

# 原始输出里这个点是否完整跑完：有一整行 item_done。
raw_has_finished_point() {
  local raw=$1 build=$2 item=$3
  complete_lines "$raw" | grep -qxF "E203RESULT name=item_done build=$build point=$item"
}

# 原始输出起 cargo 时这个点的键（旁边 <原始输出>.keys 一行「点<TAB>键」）与此刻的键相同。弄坏开关 harvest-ignores-the-key 下不核。
raw_keyed_for_the_point() {
  local raw=$1 item=$2 key=$3
  broken harvest-ignores-the-key && return 0
  [ "$key" != - ] && grep -qxF "$(printf '%s\t%s' "$item" "$key")" "$raw.keys" 2>/dev/null
}

# 起 cargo 之前写下这一份原始输出跑的每个点的键（同目录 .partial 再换名）。
write_raw_keys() {
  local raw=$1 item
  shift
  for item in "$@"; do printf '%s\t%s\n' "$item" "${point_key[$item]}"; done > "$raw.keys.partial" && mv -f "$raw.keys.partial" "$raw.keys"
}

# 同一天内容相同的 diff 引用已有那份，没有才写新名（同目录隐藏 .partial 再换名）。交回产物目录里的路径。
publish_diff() {
  local text=$1 build=$2 existing day name n=1 temporary
  for existing in "$repo"/research/results/e203-admission-scan-cost-"$build"-counters-diff-*.diff; do
    if [ -f "$existing" ] && cmp -s "$text" "$existing"; then printf '%s\n' "$existing"; return 0; fi
  done
  day=$(TZ=Asia/Tokyo date +%F)
  name="$repo/research/results/e203-admission-scan-cost-$build-counters-diff-$day.diff"
  while [ -e "$name" ]; do n=$((n + 1)); name="$repo/research/results/e203-admission-scan-cost-$build-counters-diff-$day-r$n.diff"; done
  temporary="$(dirname "$name")/.$(basename "$name").partial"
  cp "$text" "$temporary" && mv -f "$temporary" "$name" || return 1
  printf '%s\n' "$name"
}

header_for() {
  # 产物头：构建种类、release（opt-level = 3）、线程、历史用例的内容键、副本的 diff、这个点的键（拒算写 -）、主工作区锚点行号
  local threads=$1 item=$2
  echo "E203RESULT name=header build=$build profile=release opt_level=3 test_threads=$threads histories=$(histories_key) diff_file=$(basename "$diff_file") diff_lines=$(wc -l < "$diff_file") point_key=${point_key[$item]:--}"
  python3 -I "$counters" --check-anchors "$source_tree" | sed 's/^/E203RESULT /'
}

# 一个点跑完（或没跑出来）交判定库那一份记下：现判的照键写进库。没记成的报出来、这个点下一趟照键现判；退 75、3 留给末尾照它退。
record_point() {
  local item=$1 code
  shift
  python3 -I "$library" record --plan "$plan_file" --point "$item" "$@"
  code=$?
  [ "$code" -eq 0 ] && return 0
  echo "  ✗ 判定库那一份没记下 $build/$item（退 $code）：产物已写，这个点下一趟照键现判"
  echo "     → 怎么办：退 75 是 prune-server 不可用（python3 prune-server/control check，修好之后整条命令重跑，库里有的点照键复用）；别的退出码照它上面那几行 ✗ 的出路改"
  case "$code" in 3|75) library_trouble=$code ;; esac
  return 1
}

# 把一份原始输出里这个点的行写成产物。先写同目录的隐藏 .partial（带构建与点名），写完换名；
# 弄坏开关 product-written-in-place 直接写在产物名下。写完回读：产物里要有完整的 item_done，否则报这个点没跑出来；回读过的交判定库那一份记下。
write_product() {
  local build=$1 item=$2 threads=$3 raw=$4 product temporary view
  product=$(new_product_name "$build" "$item")
  temporary="$(dirname "$product")/.$(basename "$product").partial"
  broken product-written-in-place && temporary=$product
  view="$raw_dir/view.tmp"
  complete_lines "$raw" > "$view"
  { header_for "$threads" "$item"; grep -E "^E203RESULT name=constants " "$view"; grep -E "^E203RESULT .* point=$item( |$)" "$view"; } > "$temporary"
  [ "$temporary" = "$product" ] || mv -f "$temporary" "$product" || return 1
  if ! product_is_done "$product"; then
    echo "name=item_failed build=$build point=$item"
    record_point "$item" --failed
    return 1
  fi
  record_point "$item" --product "$product"
}

# 上一次尝试（被杀）留下的原始输出里已经完整跑完、键与此刻相同的现判点：直接写成产物、入库，从 todo 里拿掉。
# 强制现判与拒算的点不取（强制现判要这一趟现跑，拒算的没有键可核）。
harvest_finished_points() {
  local build=$1 threads=$2 item raw kept=() taken
  broken harvest-off && return 0
  for item in "${todo[@]}"; do
    taken=0
    if [ "${point_decision[$item]}" = judge ]; then
      for raw in "$raw_dir"/*.out; do
        [ -f "$raw" ] || continue
        raw_keyed_for_the_point "$raw" "$item" "${point_key[$item]}" || continue
        if raw_has_finished_point "$raw" "$build" "$item"; then
          write_product "$build" "$item" "$threads" "$raw" || failed=1
          echo "name=item_taken_from_an_earlier_attempt build=$build point=$item raw=$(basename "$raw")"
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

# 入口的主体：从 build、work、source_tree、chosen 起。退出码见文件头。
run_flow() {
  local item todo=() failed=0 library_trouble=0 copy diff_text diff_file raw raw_dir filters=() exact=(--exact) tests_directory
  local plan_file plan_output plan_code finish_code ran_before point_arguments=() tag point_field decision_field key_field
  local -A point_key=() point_decision=()
  broken filter-by-substring && exact=()

  # 副本：套计数或计时；同一个工作目录里拷完的直接用，半拷的清掉重拷（计数脚本判）。diff 先落在工作目录，再引用或写进产物目录。
  copy="$work/$build"
  diff_text="$work/$build-counters.diff"
  python3 -I "$counters" --build "$build" --source "$source_tree" --into "$copy" --diff-out "$diff_text" || return $?
  diff_file=$(publish_diff "$diff_text" "$build") || return 1
  # 历史用例先放进副本：点的键在副本上算，闭包从副本里的 point_<点> 起收
  tests_directory="$copy/crates/singlefs-verification/tests"
  mkdir -p "$tests_directory" || return 1
  if ! cmp -s "$histories" "$tests_directory/e203_admission_scan_cost_histories.rs"; then
    cp "$histories" "$tests_directory/.e203_histories.partial" && mv -f "$tests_directory/.e203_histories.partial" "$tests_directory/e203_admission_scan_cost_histories.rs" || return 1
  fi

  # 判定库：算键、查库、抽哨兵（文件头「判定库」）。计划文件只是这一趟的决定与记账，每趟整份换上。
  plan_file="$work/$build-plan.json"
  plan_output="$work/$build-plan.out"
  for item in "${chosen[@]}"; do point_arguments+=(--point "$item"); done
  python3 -I "$library" plan --copy "$copy" --library-root "$repo" --build "$build" --plan "$plan_file" "${point_arguments[@]}" > "$plan_output"
  plan_code=$?
  cat "$plan_output"
  [ "$plan_code" -eq 0 ] || { echo "name=run_done build=$build ran=0 failed=1 plan_exit=$plan_code"; return "$plan_code"; }
  while read -r tag point_field decision_field key_field; do
    [ "$tag" = E203PLAN ] || continue
    point_decision[${point_field#point=}]=${decision_field#decision=}
    point_key[${point_field#point=}]=${key_field#key=}
  done < "$plan_output"
  for item in "${chosen[@]}"; do
    if [ -z "${point_decision[$item]:-}" ]; then
      echo "  ✗ 判定库那一份的计划里没有点 $item（$plan_output），不知道它该复用还是现判"
      echo "     → 怎么办：看 $plan_output 里 E203PLAN 那几行少了哪一点、e203_admission_scan_cost_library.py plan 为什么没打它；修好整条命令重跑"
      return 1
    fi
    if [ "${point_decision[$item]}" = reuse ] && ! broken done-points-rerun; then
      echo "name=item_skipped build=$build point=$item reason=library key=${point_key[$item]}"
    else
      todo+=("$item")
    fi
  done
  ran_before=${#todo[@]}

  if [ "${#todo[@]}" -gt 0 ]; then
    for item in "${todo[@]}"; do rm -f "$repo"/research/results/.e203-admission-scan-cost-"$build"-"$item"-*.partial; done
    cd "$copy" || return 1
    export E203_BUILD="$build"
    # 原始输出放在副本里：副本清掉它们跟着清；每一份旁边的 .keys 记着起 cargo 时每个点的键，键变了就不取
    raw_dir="$copy/.e203-raw"
    mkdir -p "$raw_dir" || return 1
    if [ "$build" = "counts" ]; then
      harvest_finished_points counts default
      if [ "${#todo[@]}" -gt 0 ]; then
        for item in "${todo[@]}"; do filters+=("point_$item"); done
        raw=$(next_raw_name counts)
        write_raw_keys "$raw" "${todo[@]}" || return 1
        SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E203 历史用例（计数构建）; 范围=1 个测试目标; 为什么=量准入之前扫类的步数，只跑这一趟判定库里没有的点' \
          cargo test --release -p singlefs-verification --test e203_admission_scan_cost_histories -- "${filters[@]}" "${exact[@]}" --nocapture > "$raw" 2>&1
        for item in "${todo[@]}"; do write_product counts "$item" default "$raw" || failed=1; done
      fi
    else
      harvest_finished_points timing 1
      for item in ${todo[@]+"${todo[@]}"}; do
        raw=$(next_raw_name "timing-$item")
        write_raw_keys "$raw" "$item" || return 1
        SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E203 历史用例（计时构建，单线程）; 范围=1 个测试目标; 为什么=量准入之前扫类的挂钟，单线程逐点跑免得互相抢核' \
          cargo test --release -p singlefs-verification --test e203_admission_scan_cost_histories -- "point_$item" "${exact[@]}" --nocapture --test-threads=1 > "$raw" 2>&1
        write_product timing "$item" 1 "$raw" || failed=1
      done
    fi
  fi
  python3 -I "$library" finish --plan "$plan_file"
  finish_code=$?
  [ "$finish_code" -eq 0 ] || failed=1
  echo "name=run_done build=$build ran=$ran_before failed=$failed"
  [ "$library_trouble" -eq 0 ] || return "$library_trouble"
  return "$failed"
}

# ---- 自证：假驱动（cargo 换成读模式文件的函数，计数脚本跑真的、拷一棵小的假仓根；算键与判定库跑真的，判定库是临时 prune-server），不编译、不跑实现 ----

# 假 cargo：把参数记进 calls.log；按 $world_dir/mode 决定怎么跑。点名的 point_<点> 各打 row 与 item_done 两行。
#   ok 全跑完；rows-differ 全跑完、row 那一行的读数换一个（强制现判判出与库里不同）；kill-immediately 一开始就被杀；kill-on-second-call 第二次调用被杀；
#   kill-after-first 第一个点跑完、第二个点打一行就被杀；kill-cut-line 打出一行被截断的 item_done（点名被截掉最后两个字符）就被杀。
# 模拟这一趟被系统杀掉：对自己这一个子 shell 发 KILL（不跑 EXIT trap），点名一个进程，不在循环里发。
simulate_this_run_being_killed() {
  kill -KILL "$BASHPID"
}

fake_cargo() {
  local mode filter name index=0 filters=() argument seen_separator=0 calls reading=1
  echo "$*" >> "$calls_log"
  calls=$(wc -l < "$calls_log")
  mode=$(cat "$world_dir/mode")
  [ "$mode" = rows-differ ] && reading=2
  for argument in "$@"; do
    if [ "$seen_separator" -eq 1 ]; then case "$argument" in --*) ;; *) filters+=("$argument") ;; esac; fi
    [ "$argument" = "--" ] && seen_separator=1
  done
  printf 'E203RESULT name=constants x=1\n'
  case "$mode" in
    kill-immediately) simulate_this_run_being_killed ;;
    kill-on-second-call) [ "$calls" -eq 2 ] && simulate_this_run_being_killed ;;
  esac
  for filter in "${filters[@]}"; do
    name=${filter#point_}
    index=$((index + 1))
    if [ "$mode" = kill-cut-line ]; then
      printf 'E203RESULT name=item_done build=%s point=%s' "$E203_BUILD" "${name%??}"
      simulate_this_run_being_killed
    fi
    printf 'E203RESULT name=row point=%s v=%s\n' "$name" "$reading"
    if [ "$mode" = kill-after-first ] && [ "$index" -eq 2 ]; then simulate_this_run_being_killed; fi
    printf 'E203RESULT name=item_done build=%s point=%s\n' "$E203_BUILD" "$name"
  done
  return 0
}

# 假历史用例：三个点，两个小点共用一段历史（复用三问改的就是它），大点另走一段；入口照真的那样拷进副本、在副本上算键。
write_selftest_histories() {
  cat > "$1" <<'EOF'
//! 入口自证用的假历史用例：两个小点共用一段历史，大点另走一段。
use singlefs_core::scan_step;

fn shared_history_of_the_small_points() -> u64 {
    scan_step() + 1
}

fn history_of_the_large_point() -> u64 {
    scan_step() + 2
}

#[test]
fn point_unlinks_1() {
    assert_eq!(shared_history_of_the_small_points(), 2);
}

#[test]
fn point_unlinks_100() {
    assert_eq!(shared_history_of_the_small_points() + 1, 3);
}

#[test]
fn point_unlinks_1000() {
    assert_eq!(history_of_the_large_point(), 3);
}
EOF
}

# 假仓根（计数脚本写的那一棵）上补成一个能算键的工作区：两个包，验证包的特性行照计数脚本的锚点写（计时构建要恰好命中一次），
# 实现包只有一个 scan_step；计数脚本插的那几份文件不在任何模块树里，不进任何点的闭包。
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
  -V) echo "rustc 0.0.0-e203-selftest" ;;
  -vV) printf 'rustc 0.0.0-e203-selftest\nbinary: rustc\nhost: x86_64-unknown-linux-gnu\nrelease: 0.0.0-e203-selftest\n' ;;
esac
exit 0
EOF
  cat > "$tools/cargo" <<'EOF'
#!/usr/bin/env bash
case "${1:-}" in
  -V|--version) echo "cargo 0.0.0-e203-selftest" ;;
esac
exit 0
EOF
  chmod +x "$tools/rustc" "$tools/cargo"
}

# 自证样本里把一段原文换成另一段（要恰好有这一段）。
replace_in_file() {
  local file=$1 old=$2 new=$3 text
  text=$(cat "$file") || return 1
  case "$text" in *"$old"*) ;; *) echo "自证样本 $file 里没有「$old」"; return 1 ;; esac
  printf '%s\n' "${text/"$old"/"$new"}" > "$file"
}

# 一间自证用的小世界：假仓根（只有 research/results，判定库的命名空间按它给）、假历史用例、假源仓根（补成能算键的工作区）、
# 只答版本的工具链、工作目录、调用记录、模式文件。哨兵条数默认取得很大：键相同的全当哨兵、不抽强制现判（哨兵那几格自己改）。
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
  set_mode ok
}

set_mode() { printf '%s\n' "$1" > "$world_dir/mode"; }

# 跑一遍入口主体（子 shell：被杀只杀这一遍）。输出进 $world_dir/last.out，退出码进 last_rc。
go() {
  build=$1
  shift
  chosen=("$@")
  { ( run_flow ) > "$world_dir/last.out" 2>&1; last_rc=$?; } 2>/dev/null
}

calls() { wc -l < "$calls_log" | tr -d ' '; }
products() { find "$repo/research/results" -maxdepth 1 -name "e203-admission-scan-cost-$1-$2-*.out" | wc -l | tr -d ' '; }
latest_product() { find "$repo/research/results" -maxdepth 1 -name "e203-admission-scan-cost-$1-$2-*.out" -printf '%T@ %p\n' | sort -n | tail -n 1 | cut -d' ' -f2-; }

# 自证里判「点做完了」：照此刻副本上的键问判定库（入口主体在子 shell 里跑，计划带不回来）。
stored_in_library() {
  local build=$1 arguments=() item
  shift
  for item in "$@"; do arguments+=(--point "$item"); done
  python3 -I "$library" stored --copy "$work/$build" --library-root "$repo" --build "$build" "${arguments[@]}" > "$world_dir/stored.out" 2>&1 || { cat "$world_dir/stored.out"; return 1; }
}

# 在 $work/counts 上直接起判定库那一份的 plan（计划 $world_dir/direct-plan.json，输出 direct-plan.out），给哨兵那几格自己摆 record 与 finish。
plan_directly() {
  local arguments=() item
  for item in "$@"; do arguments+=(--point "$item"); done
  python3 -I "$library" plan --copy "$work/counts" --library-root "$repo" --build counts --plan "$world_dir/direct-plan.json" "${arguments[@]}" > "$world_dir/direct-plan.out" 2>&1 \
    || { echo "plan 该退 0"; cat "$world_dir/direct-plan.out"; return 1; }
}
planned_sentinels() { sed -n 's/^E203SENTINEL .* sentinels=\([^ ]*\) forced=.*$/\1/p' "$1"; }
planned_forced() { sed -n 's/^E203SENTINEL .* forced=\([^ ]*\)$/\1/p' "$1"; }
finish_directly() {
  python3 -I "$library" finish --plan "$world_dir/direct-plan.json" > "$world_dir/finish.out" 2>&1
  finish_rc=$?
}

case_killed_after_the_copy_and_run_again_completes() {
  set_mode kill-immediately
  go counts unlinks_1 unlinks_100
  [ "$last_rc" -eq 137 ] || { echo "第一遍该被杀（137），得 $last_rc"; return 1; }
  [ -f "$work/counts/.e203-copy-done" ] || { echo "被杀时副本该已拷完、有完成标记"; return 1; }
  set_mode ok
  go counts unlinks_1 unlinks_100
  [ "$last_rc" -eq 0 ] || { echo "第二遍该跑完（0），得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  grep -q 'name=run_tally build=counts judged=2 forced=0 reused=0 refused=0 not_done=0 not_run=0' "$world_dir/last.out" || { echo "计账行不对"; cat "$world_dir/last.out"; return 1; }
  stored_in_library counts unlinks_1 unlinks_100 || { echo "两个点该照键进了判定库"; return 1; }
}

case_a_half_copy_is_cleared_and_run_again_completes() {
  mkdir -p "$work/counts/crates" && printf 'half\n' > "$work/counts/crates/half.rs" && printf 'begun\n' > "$work/counts/.e203-copy-begun" || return 1
  go counts unlinks_1
  [ "$last_rc" -eq 0 ] || { echo "半拷的副本该清掉重拷、跑完（0），得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  [ ! -e "$work/counts/crates/half.rs" ] || { echo "半拷留下的文件该已清掉"; return 1; }
  stored_in_library counts unlinks_1 || { echo "点该照键进了判定库"; return 1; }
}

case_a_directory_that_is_not_our_copy_is_refused_and_left_alone() {
  mkdir -p "$work/counts" && printf 'mine\n' > "$work/counts/mine.txt" || return 1
  go counts unlinks_1
  [ "$last_rc" -eq 2 ] || { echo "该拒绝（2），得 $last_rc"; return 1; }
  [ -f "$work/counts/mine.txt" ] || { echo "不是我们拷的东西不许清"; return 1; }
  [ "$(calls)" -eq 0 ] || { echo "拒绝时不许起 cargo"; return 1; }
}

case_a_finished_point_is_not_run_again() {
  go counts unlinks_1 unlinks_100
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] && [ "$(products counts unlinks_1)" -eq 1 ] || { echo "第一遍该跑一次 cargo、各一份产物"; cat "$world_dir/last.out"; return 1; }
  go counts unlinks_1 unlinks_100
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "第二遍不该再起 cargo（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  grep -q 'name=run_tally build=counts judged=0 forced=0 reused=2 refused=0' "$world_dir/last.out" || { echo "第二遍该现判 0、复用 2"; cat "$world_dir/last.out"; return 1; }
  [ "$(products counts unlinks_1)" -eq 1 ] || { echo "复用的点不另写产物"; return 1; }
}

case_a_point_finished_inside_a_killed_run_is_taken_and_not_run_again() {
  set_mode kill-after-first
  go counts unlinks_1 unlinks_100
  [ "$last_rc" -eq 137 ] || { echo "第一遍该被杀，得 $last_rc"; return 1; }
  set_mode ok
  go counts unlinks_1 unlinks_100
  [ "$last_rc" -eq 0 ] || { echo "第二遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  [ "$(calls)" -eq 2 ] || { echo "该共起两次 cargo"; return 1; }
  ! sed -n 2p "$calls_log" | grep -q 'point_unlinks_1 ' || { echo "第二次不该再点名跑完的 unlinks_1"; sed -n 2p "$calls_log"; return 1; }
  sed -n 2p "$calls_log" | grep -q 'point_unlinks_100' || { echo "第二次该点名没跑完的 unlinks_100"; return 1; }
  stored_in_library counts unlinks_1 unlinks_100 || { echo "两个点该照键进了判定库（取来的那一点也入库）"; return 1; }
}

case_a_point_finished_inside_a_killed_run_under_another_key_is_run_again() {
  set_mode kill-after-first
  go counts unlinks_1 unlinks_100
  [ "$last_rc" -eq 137 ] || { echo "第一遍该被杀，得 $last_rc"; return 1; }
  # 两个小点共用的那段历史改了：被杀那一趟里跑完的 unlinks_1 是旧键跑出来的，不取
  replace_in_file "$histories" 'scan_step() + 1' 'scan_step() + 3' || return 1
  set_mode ok
  go counts unlinks_1 unlinks_100
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "第二遍该跑完、共起两次 cargo（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  sed -n 2p "$calls_log" | grep -q 'point_unlinks_1 ' && sed -n 2p "$calls_log" | grep -q 'point_unlinks_100' \
    || { echo "键变了，第二次该把两个点都点名"; sed -n 2p "$calls_log"; return 1; }
}

case_a_line_cut_by_a_kill_does_not_finish_a_point() {
  set_mode kill-cut-line
  go counts unlinks_100 unlinks_1
  [ "$last_rc" -eq 137 ] || { echo "第一遍该被杀，得 $last_rc"; return 1; }
  set_mode ok
  go counts unlinks_100 unlinks_1
  [ "$last_rc" -eq 0 ] || { echo "第二遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  sed -n 2p "$calls_log" | grep -q 'point_unlinks_100 point_unlinks_1 ' || { echo "第二次该把两个点都点名（残行不算跑完）"; sed -n 2p "$calls_log"; return 1; }
}

case_the_point_filter_names_one_test_only() {
  go counts unlinks_1
  sed -n 1p "$calls_log" | grep -q -- ' --exact ' || { echo "点名该带 --exact（不然 unlinks_1 会连带 unlinks_100）"; sed -n 1p "$calls_log"; return 1; }
}

case_a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name() {
  { ( header_for() { kill -KILL "$BASHPID"; }; build=counts; chosen=(unlinks_1); run_flow ) > "$world_dir/last.out" 2>&1; last_rc=$?; } 2>/dev/null
  [ "$last_rc" -eq 137 ] || { echo "第一遍该在写产物时被杀，得 $last_rc"; return 1; }
  [ "$(products counts unlinks_1)" -eq 0 ] || { echo "被杀时产物名下不该有文件"; return 1; }
  go counts unlinks_1
  [ "$last_rc" -eq 0 ] || { echo "第二遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  [ "$(calls)" -eq 1 ] || { echo "原始输出已跑完，第二遍不该再起 cargo（calls=$(calls)）"; return 1; }
  [ "$(products counts unlinks_1)" -eq 1 ] || { echo "该恰好一份产物"; return 1; }
  [ -z "$(find "$repo/research/results" -maxdepth 1 -name '.*.partial')" ] || { echo "残留的隐藏 .partial 该已清掉"; return 1; }
}

case_timing_runs_one_cargo_per_point_and_skips_the_finished_one() {
  set_mode kill-on-second-call
  go timing unlinks_1 unlinks_100
  [ "$last_rc" -eq 137 ] || { echo "第一遍该在第二个点被杀，得 $last_rc"; return 1; }
  set_mode ok
  go timing unlinks_1 unlinks_100
  [ "$last_rc" -eq 0 ] || { echo "第二遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  [ "$(calls)" -eq 3 ] || { echo "该共 3 次 cargo（unlinks_1、被杀的 unlinks_100、重跑的 unlinks_100），得 $(calls)"; return 1; }
  grep -q -- '--test-threads=1' "$calls_log" || { echo "计时构建该单线程"; return 1; }
  [ "$(products timing unlinks_1)" -eq 1 ] && [ "$(products timing unlinks_100)" -eq 1 ] || { echo "各一份产物"; return 1; }
}

# 复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：头一趟库是空的、同一份输入、闭包外改动（历史用例加一个没人调的函数与一行注释，
# 实现里不在任何闭包的文件加一行：副本重拷、diff 跟着变）、闭包内改动（两个小点共用的那段历史）。末尾打 REUSE_SELFTEST 那一行。
case_reuse_three_questions() {
  local runs="$world_dir/three-questions"
  mkdir -p "$runs" || return 1
  go counts unlinks_1 unlinks_100 unlinks_1000
  cp "$world_dir/last.out" "$runs/first.out" || return 1
  go counts unlinks_1 unlinks_100 unlinks_1000
  cp "$world_dir/last.out" "$runs/second.out" || return 1
  printf '\n// 历史用例里加的一行注释\nfn spare_history_nobody_runs() -> u64 {\n    9\n}\n' >> "$histories" || return 1
  printf '// 源码变了的一行\n' >> "$source_tree/crates/singlefs-core/src/transaction.rs" || return 1
  go counts unlinks_1 unlinks_100 unlinks_1000
  cp "$world_dir/last.out" "$runs/unrelated.out" || return 1
  replace_in_file "$histories" 'scan_step() + 1' 'scan_step() + 3' || return 1
  go counts unlinks_1 unlinks_100 unlinks_1000
  cp "$world_dir/last.out" "$runs/unit-change.out" || return 1
  python3 -I "$here/../gate/lib_reuse_checks.py" three-questions --script "$entry_script_key" --first "$runs/first.out" --second "$runs/second.out" \
    --unrelated "$runs/unrelated.out" --unit-change "$runs/unit-change.out" --affected counts/unlinks_1,counts/unlinks_100 --from-output
}

# 键不随副本的路径变：同一份副本拷到两个别的目录，键逐字相同；工作目录换到别处（副本在另一个绝对路径上现拷），库里判过的照键复用。
case_the_point_key_does_not_depend_on_the_copy_path() {
  local one="$world_dir/one/place" another="$world_dir/another/place/deeper"
  go counts unlinks_1 unlinks_1000
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "头一遍该跑完、起一次 cargo（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  mkdir -p "$one" "$another" && cp -r "$work/counts/." "$one/" && cp -r "$work/counts/." "$another/" || return 1
  python3 -I "$library" keys --copy "$one" --build counts --point unlinks_1 --point unlinks_1000 > "$world_dir/keys-one.out" 2>&1 || { cat "$world_dir/keys-one.out"; return 1; }
  python3 -I "$library" keys --copy "$another" --build counts --point unlinks_1 --point unlinks_1000 > "$world_dir/keys-another.out" 2>&1 || { cat "$world_dir/keys-another.out"; return 1; }
  [ "$(grep -c '^E203KEY ' "$world_dir/keys-one.out")" -eq 2 ] && ! grep -q 'key=refused' "$world_dir/keys-one.out" || { echo "两个点都该算得出键"; cat "$world_dir/keys-one.out"; return 1; }
  cmp -s "$world_dir/keys-one.out" "$world_dir/keys-another.out" || { echo "同一份源码放两个目录，键该相同"; cat "$world_dir/keys-one.out" "$world_dir/keys-another.out"; return 1; }
  work="$world_dir/elsewhere/work"
  go counts unlinks_1 unlinks_1000
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "换了副本的路径，该照键复用、不再起 cargo（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  grep -q 'name=run_tally build=counts judged=0 forced=0 reused=2 refused=0' "$world_dir/last.out" || { echo "该现判 0、复用 2"; cat "$world_dir/last.out"; return 1; }
}

case_reuse_sentinel_draws_and_the_forced_point_is_run_again() {
  local forced
  go counts unlinks_1 unlinks_100 unlinks_1000
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "头一遍该跑完（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  go counts unlinks_1 unlinks_100 unlinks_1000
  [ "$last_rc" -eq 0 ] || { echo "哨兵复用、强制现判与库里相同，该退 0，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  forced=$(planned_forced "$world_dir/last.out")
  [ -n "$forced" ] && [ "$forced" != - ] || { echo "三个点键相同、哨兵 1 条，该抽出一条强制现判"; cat "$world_dir/last.out"; return 1; }
  [ "$(calls)" -eq 2 ] && [ "$(sed -n 2p "$calls_log" | grep -o 'point_[a-z0-9_]*' | wc -l)" -eq 1 ] && sed -n 2p "$calls_log" | grep -q "point_$forced " \
    || { echo "第二遍该只起一次 cargo、只点名强制现判的 $forced"; cat "$calls_log"; return 1; }
  grep -q 'name=run_tally build=counts judged=1 forced=1 reused=2 refused=0 not_done=0 not_run=0' "$world_dir/last.out" || { echo "计账行不对"; cat "$world_dir/last.out"; return 1; }
  grep -q '哨兵 1 条都复用；强制现判' "$world_dir/last.out" || { echo "该报哨兵复用、强制现判与库里同"; cat "$world_dir/last.out"; return 1; }
}

case_a_rejudged_sentinel_turns_the_run_red() {
  local sentinel forced
  go counts unlinks_1 unlinks_100 unlinks_1000
  [ "$last_rc" -eq 0 ] || { echo "头一遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  plan_directly unlinks_1 unlinks_100 unlinks_1000 || return 1
  sentinel=$(planned_sentinels "$world_dir/direct-plan.out")
  forced=$(planned_forced "$world_dir/direct-plan.out")
  [ -n "$sentinel" ] && [ "$sentinel" != - ] && [ -n "$forced" ] && [ "$forced" != - ] || { echo "该抽出一条哨兵、一条强制现判"; cat "$world_dir/direct-plan.out"; return 1; }
  # 入口照计划该复用的哨兵却跑了：记一次它
  python3 -I "$library" record --plan "$world_dir/direct-plan.json" --point "$sentinel" --product "$(latest_product counts "$sentinel")" || return 1
  python3 -I "$library" record --plan "$world_dir/direct-plan.json" --point "$forced" --product "$(latest_product counts "$forced")" || return 1
  finish_directly
  [ "$finish_rc" -eq 1 ] && grep -q '复用失灵' "$world_dir/finish.out" || { echo "哨兵被现判，该退 1、报复用失灵（得 $finish_rc）"; cat "$world_dir/finish.out"; return 1; }
}

case_a_forced_rejudge_with_another_verdict_turns_the_run_red() {
  go counts unlinks_1 unlinks_100 unlinks_1000
  [ "$last_rc" -eq 0 ] || { echo "头一遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  set_mode rows-differ
  go counts unlinks_1 unlinks_100 unlinks_1000
  [ "$last_rc" -eq 1 ] && grep -q '判定不稳' "$world_dir/last.out" || { echo "强制现判的读数与库里不同，该退 1、报判定不稳（得 $last_rc）"; cat "$world_dir/last.out"; return 1; }
}

case_a_forced_point_left_unrun_turns_the_run_red() {
  local forced
  go counts unlinks_1 unlinks_100 unlinks_1000
  [ "$last_rc" -eq 0 ] || { echo "头一遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  export SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=7
  plan_directly unlinks_1 unlinks_100 unlinks_1000 || return 1
  forced=$(planned_forced "$world_dir/direct-plan.out")
  [ -n "$forced" ] && [ "$forced" != - ] || { echo "该抽出一条强制现判"; cat "$world_dir/direct-plan.out"; return 1; }
  # 入口没跑强制现判那一点（照库复用了）：一条都不记就收尾
  finish_directly
  [ "$finish_rc" -eq 1 ] && grep -q '强制现判没起作用' "$world_dir/finish.out" || { echo "强制现判那一点没跑，该退 1、报强制现判没起作用（得 $finish_rc）"; cat "$world_dir/finish.out"; return 1; }
}

case_a_point_planned_for_judging_but_not_run_turns_the_run_red() {
  go counts unlinks_1000
  [ "$last_rc" -eq 0 ] || { echo "头一遍该跑完，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  plan_directly unlinks_1 unlinks_100 || return 1
  finish_directly
  [ "$finish_rc" -eq 1 ] && grep -q '没跑到' "$world_dir/finish.out" || { echo "要现判的两点一条都没记，该退 1、报没跑到（得 $finish_rc）"; cat "$world_dir/finish.out"; return 1; }
}

case_a_point_without_a_key_runs_and_is_never_recorded() {
  # 假历史用例里没有 point_files_1：入口找不到，拒算
  go counts files_1
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "拒算的点照跑、退 0（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
  grep -q 'E203PLAN point=files_1 decision=refused key=-' "$world_dir/last.out" && grep -q 'name=run_tally build=counts judged=0 forced=0 reused=0 refused=1' "$world_dir/last.out" \
    || { echo "该报拒算 1"; cat "$world_dir/last.out"; return 1; }
  go counts files_1
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 2 ] || { echo "拒算的点不入库，下一趟照跑（calls=$(calls) rc=$last_rc）"; cat "$world_dir/last.out"; return 1; }
}

case_an_unreachable_library_exits_75_and_runs_nothing() {
  local port
  port=$(python3 -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print(s.getsockname()[1]); s.close()') || return 1
  # 临时服务底下才认这三个变量：socket 指到不在的路径、TCP 指到刚放掉的端口、重连总时长 2 秒
  export SINGLEFS_PRUNE_SERVER_SOCKET="$world_dir/no-server.sock" SINGLEFS_PRUNE_SERVER_TCP_ADDRESS="127.0.0.1:$port" SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS=2
  go counts unlinks_1
  [ "$last_rc" -eq 75 ] || { echo "判定库连不上该退 75，得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  [ "$(calls)" -eq 0 ] || { echo "连不上时不许起 cargo（不当库里没有现判）"; return 1; }
}

selftest() {
  local failures=0 out name display reuse_line="" line
  # selftest_root 不写 local：退出时 trap 才删，那时函数的局部变量已经没了
  selftest_root=$(mktemp -d "${TMPDIR:-/tmp}/e203-run-selftest.XXXXXX") || exit 1
  trap 'rm -rf "$selftest_root"' EXIT
  # 计数脚本与判定库那一份只在命令行是 --selftest 或有这份标记文件时认弄坏开关（它们的平常用法不认）
  printf 'e203-selftest\n' > "$selftest_root/mark"
  export E203_SELFTEST_MARK="$selftest_root/mark"
  # 自证只在临时 prune-server 上跑：经判定库那一份调客户端 selftest_service_problem，不是临时服务退 2、一格不跑
  python3 -I "$library" selftest-service-problem || return 2
  cargo() { fake_cargo "$@"; }
  out=$(items_table | wc -l)
  [ "$out" -eq 20 ] || { echo "name=selftest_case case=item_count result=FAILED got=$out"; failures=$((failures + 1)); }
  [ "$(items_table | awk '$3=="first"' | wc -l)" -eq 8 ] || { echo "name=selftest_case case=first_segment_items result=FAILED"; failures=$((failures + 1)); }
  local tmp="$selftest_root/product"
  printf 'E203RESULT name=row x=1\n' > "$tmp"
  if product_is_done "$tmp"; then echo "name=selftest_case case=done_marker result=FAILED"; failures=$((failures + 1)); else echo "name=selftest_case case=a_product_without_the_marker_is_not_done result=ok"; fi
  printf 'E203RESULT name=item_done build=counts point=base\n' >> "$tmp"
  if product_is_done "$tmp"; then echo "name=selftest_case case=a_product_with_the_marker_is_done result=ok"; else echo "name=selftest_case case=done_marker result=FAILED"; failures=$((failures + 1)); fi
  for name in \
    case_killed_after_the_copy_and_run_again_completes \
    case_a_half_copy_is_cleared_and_run_again_completes \
    case_a_directory_that_is_not_our_copy_is_refused_and_left_alone \
    case_a_finished_point_is_not_run_again \
    case_a_point_finished_inside_a_killed_run_is_taken_and_not_run_again \
    case_a_point_finished_inside_a_killed_run_under_another_key_is_run_again \
    case_a_line_cut_by_a_kill_does_not_finish_a_point \
    case_the_point_filter_names_one_test_only \
    case_a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name \
    case_timing_runs_one_cargo_per_point_and_skips_the_finished_one \
    case_reuse_three_questions \
    case_the_point_key_does_not_depend_on_the_copy_path \
    case_reuse_sentinel_draws_and_the_forced_point_is_run_again \
    case_a_rejudged_sentinel_turns_the_run_red \
    case_a_forced_rejudge_with_another_verdict_turns_the_run_red \
    case_a_forced_point_left_unrun_turns_the_run_red \
    case_a_point_planned_for_judging_but_not_run_turns_the_run_red \
    case_a_point_without_a_key_runs_and_is_never_recorded \
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

# 弄坏开关只在 --selftest 里认；别处设了拒（计数脚本与判定库那一份由自证设的标记文件放行，见 selftest）。
if [ -n "${E203_RUN_BREAK:-}" ] && [ "${1:-}" != --selftest ]; then
  echo "  ✗ E203_RUN_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "     → 怎么办：去掉 E203_RUN_BREAK 再跑；要证红就跑 E203_RUN_BREAK=<项> bash $0 --selftest（经临时 prune-server 的包装）" >&2
  exit 2
fi

build="" work="" source_tree="$repo" segment="" requested=() chosen=()
list_only=0
while [ $# -gt 0 ]; do
  case "$1" in
    --list-items) list_only=1; shift ;;
    --selftest) selftest; exit $? ;;
    --build) build=${2:-}; shift 2 ;;
    --work) work=${2:-}; shift 2 ;;
    --source) source_tree=${2:-}; shift 2 ;;
    --segment) segment=${2:-}; shift 2 ;;
    --item) requested+=("${2:-}"); shift 2 ;;
    *) usage ;;
  esac
done

if [ "$list_only" -eq 1 ]; then items_table; exit 0; fi
case "$build" in counts|timing) ;; *) usage ;; esac
[ -n "$work" ] || usage
[ -f "$counters" ] && [ -f "$library" ] && [ -f "$histories" ] || { echo "前置文件不在：$counters、$library 或 $histories" >&2; exit 78; }

if [ -n "$segment" ]; then
  while read -r item _ seg; do [ "$seg" = "$segment" ] && chosen+=("$item"); done < <(items_table)
  [ "${#chosen[@]}" -gt 0 ] || usage
fi
for item in "${requested[@]+"${requested[@]}"}"; do
  items_table | awk -v i="$item" '$1==i {found=1} END {exit !found}' || { echo "项名写错：$item；可点名的项：" >&2; items_table >&2; exit 2; }
  chosen+=("$item")
done
[ "${#chosen[@]}" -gt 0 ] || usage

mkdir -p "$work" && work=$(cd "$work" && pwd) || { echo "工作目录建不出：$work" >&2; exit 2; }
[ -d "$source_tree" ] || { echo "仓根不在：$source_tree" >&2; exit 78; }
source_tree=$(cd "$source_tree" && pwd)
run_flow
exit $?
