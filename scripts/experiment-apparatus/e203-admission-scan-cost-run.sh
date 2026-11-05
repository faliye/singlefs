#!/usr/bin/env bash
# E203（准入之前扫类的步代价）量的那一路的入口：给仓副本套上计数或计时，起历史用例，产物按点分文件写进 research/results/。
# 跑前登记 research/prompts/e203-preregistration.md 第五节、第十四节。
#
# 用法：
#   e203-admission-scan-cost-run.sh --list-items
#   e203-admission-scan-cost-run.sh --build counts|timing --work <工作目录> (--segment first|second|third | --item <点>…) [--source <仓根>]
#   e203-admission-scan-cost-run.sh --selftest        只核入口自己的列项、拒绝、完成标记、续跑判法，用假驱动，不编译、不跑实现
# 同一条命令重起就是续跑（被内存上限杀、手动停都一样）：
#   副本：工作目录里的副本先写开拷标记、拷完改完才写完成标记；半拷的清掉重拷，拷完且输入键相同的原样用，
#         非空而不是本脚本拷的目录拒绝（退 2）、一个字节不清。
#   点：产物里有 `E203RESULT name=item_done` 的点不重跑；上一次尝试的原始输出（副本里 .e203-raw 下）里已经完整跑完的点，
#       直接写成产物、不重跑；被杀时正在跑的点才重跑。
#   产物：先写同目录的隐藏 .partial 再换名，被杀时产物名下要么没有、要么是整份；做完而没有 item_done 的产物（点没跑出来）
#         不删不覆盖，重起时另写新文件名。
#   diff：内容相同的已有 diff 文件原样引用，不另写一份。
#
# run-condition: 无（不要 GPU、不要第二台）。计时构建不与别的重活同机（服务排）。
# 退出码：0 跑到的全绿；1 有点没跑出 item_done；2 用法错或项名写错或工作目录里有不是本脚本拷的东西；77 一个点都没跑到；78 准入条件不满足（仓根或前置脚本不在）。
# 弄坏开关（只给 --selftest 用，平常跑时不认）。逐项写：开关、弄坏哪一处、打开后哪几格该红（失败行句首 name=selftest_case case=<格名> result=FAILED）：
#   E203_RUN_BREAK=done-marker-ignored  完成标记不看（凡判「做完没有」的地方都当做完了）
#       done_marker、killed_after_the_copy_and_run_again_completes、a_half_copy_is_cleared_and_run_again_completes、
#       a_directory_that_is_not_our_copy_is_refused_and_left_alone、a_finished_point_is_not_run_again、
#       a_point_finished_inside_a_killed_run_is_taken_and_not_run_again、a_line_cut_by_a_kill_does_not_finish_a_point、
#       the_point_filter_names_one_test_only、a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name、
#       timing_runs_one_cargo_per_point_and_skips_the_finished_one
#   E203_RUN_BREAK=done-points-rerun  做完的点照样重跑（不看产物里的完成标记）
#       a_finished_point_is_not_run_again、timing_runs_one_cargo_per_point_and_skips_the_finished_one
#   E203_RUN_BREAK=harvest-off  不从上一次尝试的原始输出里取跑完的点
#       a_point_finished_inside_a_killed_run_is_taken_and_not_run_again、a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name
#   E203_RUN_BREAK=cut-lines-kept  取点时不丢最后一行不带换行的残行
#       a_line_cut_by_a_kill_does_not_finish_a_point
#   E203_RUN_BREAK=product-written-in-place  产物直接写在产物名下（被杀时留下半份）
#       a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name
#   E203_RUN_BREAK=filter-by-substring  点名不带 --exact，point_files_1 会连带跑 files_100 等
#       the_point_filter_names_one_test_only
#   E203_RUN_BREAK=copy-resume-off  副本不认完成标记与开拷标记，非空一律拒（这一项在计数脚本里读，入口只是把环境带过去）
#       killed_after_the_copy_and_run_again_completes、a_half_copy_is_cleared_and_run_again_completes、
#       a_point_finished_inside_a_killed_run_is_taken_and_not_run_again、a_line_cut_by_a_kill_does_not_finish_a_point、
#       a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name、timing_runs_one_cargo_per_point_and_skips_the_finished_one
set -u

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)
counters="$here/e203_admission_scan_cost_counters.py"
histories="$here/e203_admission_scan_cost_histories.rs"

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
many_unlinks_small_unit_area slow second
user_scale_unit_area slow third
EOF
}

usage() { sed -n '2,/^set -u/p' "$0" | sed '$d' >&2; exit 2; }

broken() { [ "${E203_RUN_BREAK:-}" = "$1" ]; }

# 一份产物是否做完：有 item_done 行。弄坏开关打开时不看。
product_is_done() {
  broken done-marker-ignored && return 0
  grep -q '^E203RESULT name=item_done ' "$1" 2>/dev/null
}

# 点在今天是否已有做完的产物。
item_is_done() {
  local build=$1 item=$2 file
  broken done-points-rerun && return 1
  for file in "$repo"/research/results/e203-admission-scan-cost-"$build"-"$item"-*.out; do
    [ -f "$file" ] || continue
    product_is_done "$file" && return 0
  done
  return 1
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
  # 产物头：构建种类、release（opt-level = 3）、线程、副本的 diff、主工作区锚点行号
  local threads=$1
  echo "E203RESULT name=header build=$build profile=release opt_level=3 test_threads=$threads diff_file=$(basename "$diff_file") diff_lines=$(wc -l < "$diff_file")"
  python3 -I "$counters" --check-anchors "$source_tree" | sed 's/^/E203RESULT /'
}

# 把一份原始输出里这个点的行写成产物。先写同目录的隐藏 .partial（带构建与点名），写完换名；
# 弄坏开关 product-written-in-place 直接写在产物名下。写完回读：产物里要有完整的 item_done，否则报这个点没跑出来。
write_product() {
  local build=$1 item=$2 threads=$3 raw=$4 product temporary view
  product=$(new_product_name "$build" "$item")
  temporary="$(dirname "$product")/.$(basename "$product").partial"
  broken product-written-in-place && temporary=$product
  view="$raw_dir/view.tmp"
  complete_lines "$raw" > "$view"
  { header_for "$threads"; grep -E "^E203RESULT name=constants " "$view"; grep -E "^E203RESULT .* point=$item( |$)" "$view"; } > "$temporary"
  [ "$temporary" = "$product" ] || mv -f "$temporary" "$product" || return 1
  product_is_done "$product" || { echo "name=item_failed build=$build point=$item"; return 1; }
  return 0
}

# 上一次尝试（被杀）留下的原始输出里已经完整跑完的点：直接写成产物，从 todo 里拿掉。
harvest_finished_points() {
  local build=$1 threads=$2 item raw kept=() taken
  broken harvest-off && return 0
  for item in "${todo[@]}"; do
    taken=0
    for raw in "$raw_dir"/*.out; do
      [ -f "$raw" ] || continue
      if raw_has_finished_point "$raw" "$build" "$item"; then
        write_product "$build" "$item" "$threads" "$raw" || failed=1
        echo "name=item_taken_from_an_earlier_attempt build=$build point=$item raw=$(basename "$raw")"
        taken=1
        break
      fi
    done
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
  local item todo=() failed=0 copy diff_text diff_file raw raw_dir filters=() exact=(--exact) tests_directory
  for item in "${chosen[@]}"; do
    if item_is_done "$build" "$item"; then echo "name=item_skipped build=$build point=$item reason=already_done"; else todo+=("$item"); fi
  done
  [ "${#todo[@]}" -gt 0 ] || { echo "name=run_done build=$build ran=0 skipped=${#chosen[@]}"; return 0; }
  broken filter-by-substring && exact=()

  # 副本：套计数或计时；同一个工作目录里拷完的直接用，半拷的清掉重拷（计数脚本判）。diff 先落在工作目录，再引用或写进产物目录
  copy="$work/$build"
  diff_text="$work/$build-counters.diff"
  python3 -I "$counters" --build "$build" --source "$source_tree" --into "$copy" --diff-out "$diff_text" || return $?
  diff_file=$(publish_diff "$diff_text" "$build") || return 1
  tests_directory="$copy/crates/singlefs-verification/tests"
  mkdir -p "$tests_directory" || return 1
  if ! cmp -s "$histories" "$tests_directory/e203_admission_scan_cost_histories.rs"; then
    cp "$histories" "$tests_directory/.e203_histories.partial" && mv -f "$tests_directory/.e203_histories.partial" "$tests_directory/e203_admission_scan_cost_histories.rs" || return 1
  fi
  for item in "${todo[@]}"; do rm -f "$repo"/research/results/.e203-admission-scan-cost-"$build"-"$item"-*.partial; done

  cd "$copy" || return 1
  export E203_BUILD="$build"
  # 原始输出放在副本里、按历史用例内容分目录：副本清掉它们跟着清，历史用例换了就不认旧的
  raw_dir="$copy/.e203-raw/$(sha256sum "$histories" | cut -c1-16)"
  mkdir -p "$raw_dir" || return 1
  local ran_before=${#todo[@]}
  if [ "$build" = "counts" ]; then
    harvest_finished_points counts default
    if [ "${#todo[@]}" -gt 0 ]; then
      for item in "${todo[@]}"; do filters+=("point_$item"); done
      raw=$(next_raw_name counts)
      SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E203 历史用例（计数构建）; 范围=1 个测试目标; 为什么=量准入之前扫类的步数' \
        cargo test --release -p singlefs-verification --test e203_admission_scan_cost_histories -- "${filters[@]}" "${exact[@]}" --nocapture > "$raw" 2>&1
      for item in "${todo[@]}"; do write_product counts "$item" default "$raw" || failed=1; done
    fi
  else
    harvest_finished_points timing 1
    for item in "${todo[@]}"; do
      raw=$(next_raw_name "timing-$item")
      SINGLEFS_RUN_SCOPE='种类=target; 跑什么=E203 历史用例（计时构建，单线程）; 范围=1 个测试目标; 为什么=量准入之前扫类的挂钟' \
        cargo test --release -p singlefs-verification --test e203_admission_scan_cost_histories -- "point_$item" "${exact[@]}" --nocapture --test-threads=1 > "$raw" 2>&1
      write_product timing "$item" 1 "$raw" || failed=1
    done
  fi
  echo "name=run_done build=$build ran=$ran_before failed=$failed"
  return "$failed"
}

# ---- 自证：假驱动（cargo 换成读模式文件的函数，计数脚本跑真的、拷一棵小的假仓根），不编译、不跑实现 ----

# 假 cargo：把参数记进 calls.log；按 $world_dir/mode 决定怎么跑。点名的 point_<点> 各打 row 与 item_done 两行。
#   ok 全跑完；kill-immediately 一开始就被杀；kill-on-second-call 第二次调用被杀；
#   kill-after-first 第一个点跑完、第二个点打一行就被杀；kill-cut-line 打出一行被截断的 item_done（点名被截掉最后两个字符）就被杀。
fake_cargo() {
  local mode filter name index=0 filters=() argument seen_separator=0 calls
  echo "$*" >> "$calls_log"
  calls=$(wc -l < "$calls_log")
  mode=$(cat "$world_dir/mode")
  for argument in "$@"; do
    if [ "$seen_separator" -eq 1 ]; then case "$argument" in --*) ;; *) filters+=("$argument") ;; esac; fi
    [ "$argument" = "--" ] && seen_separator=1
  done
  printf 'E203RESULT name=constants x=1\n'
  case "$mode" in
    kill-immediately) kill -KILL "$BASHPID" ;;
    kill-on-second-call) [ "$calls" -eq 2 ] && kill -KILL "$BASHPID" ;;
  esac
  for filter in "${filters[@]}"; do
    name=${filter#point_}
    index=$((index + 1))
    if [ "$mode" = kill-cut-line ]; then
      printf 'E203RESULT name=item_done build=%s point=%s' "$E203_BUILD" "${name%??}"
      kill -KILL "$BASHPID"
    fi
    printf 'E203RESULT name=row point=%s v=1\n' "$name"
    if [ "$mode" = kill-after-first ] && [ "$index" -eq 2 ]; then kill -KILL "$BASHPID"; fi
    printf 'E203RESULT name=item_done build=%s point=%s\n' "$E203_BUILD" "$name"
  done
  return 0
}

# 一间自证用的小世界：假仓根（只有 research/results）、假历史用例、假源仓根、工作目录、调用记录、模式文件
world() {
  world_dir=$1
  mkdir -p "$world_dir/repo/research/results" "$world_dir/src" || return 1
  repo="$world_dir/repo"
  histories="$world_dir/histories.rs"
  printf '// 假历史用例\n' > "$histories"
  source_tree="$world_dir/src"
  python3 -I "$counters" --write-fake-source "$source_tree" || return 1
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

case_killed_after_the_copy_and_run_again_completes() {
  set_mode kill-immediately
  go counts unlinks_1 unlinks_100
  [ "$last_rc" -eq 137 ] || { echo "第一遍该被杀（137），得 $last_rc"; return 1; }
  [ -f "$work/counts/.e203-copy-done" ] || { echo "被杀时副本该已拷完、有完成标记"; return 1; }
  set_mode ok
  go counts unlinks_1 unlinks_100
  [ "$last_rc" -eq 0 ] || { echo "第二遍该跑完（0），得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  grep -q 'run_done build=counts ran=2 failed=0' "$world_dir/last.out" || { echo "末行不对"; cat "$world_dir/last.out"; return 1; }
  item_is_done counts unlinks_1 && item_is_done counts unlinks_100 || { echo "两个点该做完"; return 1; }
}

case_a_half_copy_is_cleared_and_run_again_completes() {
  mkdir -p "$work/counts/crates" && printf 'half\n' > "$work/counts/crates/half.rs" && printf 'begun\n' > "$work/counts/.e203-copy-begun" || return 1
  go counts unlinks_1
  [ "$last_rc" -eq 0 ] || { echo "半拷的副本该清掉重拷、跑完（0），得 $last_rc"; cat "$world_dir/last.out"; return 1; }
  [ ! -e "$work/counts/crates/half.rs" ] || { echo "半拷留下的文件该已清掉"; return 1; }
  item_is_done counts unlinks_1 || { echo "点该做完"; return 1; }
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
  [ "$last_rc" -eq 0 ] && [ "$(calls)" -eq 1 ] || { echo "第二遍不该再起 cargo（calls=$(calls) rc=$last_rc）"; return 1; }
  grep -q 'run_done build=counts ran=0 skipped=2' "$world_dir/last.out" || { echo "第二遍末行该是 ran=0 skipped=2"; cat "$world_dir/last.out"; return 1; }
  [ "$(products counts unlinks_1)" -eq 1 ] || { echo "产物不该多出一份"; return 1; }
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
  item_is_done counts unlinks_1 && item_is_done counts unlinks_100 || { echo "两个点该做完"; return 1; }
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

selftest() {
  local failures=0 out name
  # selftest_root 不写 local：退出时 trap 才删，那时函数的局部变量已经没了
  selftest_root=$(mktemp -d "${TMPDIR:-/tmp}/e203-run-selftest.XXXXXX") || exit 1
  trap 'rm -rf "$selftest_root"' EXIT
  # 计数脚本只在命令行是 --selftest 或有这份标记文件时认弄坏开关（它的 --build 是平常跑）
  printf 'e203-selftest\n' > "$selftest_root/mark"
  export E203_SELFTEST_MARK="$selftest_root/mark"
  cargo() { fake_cargo "$@"; }
  out=$(items_table | wc -l)
  [ "$out" -eq 19 ] || { echo "name=selftest_case case=item_count result=FAILED got=$out"; failures=$((failures + 1)); }
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
    case_a_line_cut_by_a_kill_does_not_finish_a_point \
    case_the_point_filter_names_one_test_only \
    case_a_product_cut_off_by_a_kill_leaves_nothing_under_a_product_name \
    case_timing_runs_one_cargo_per_point_and_skips_the_finished_one; do
    mkdir -p "$selftest_root/$name"
    if ( world "$selftest_root/$name" && "$name" ) > "$selftest_root/$name/out.txt" 2>&1; then
      echo "name=selftest_case case=${name#case_} result=ok"
    else
      echo "name=selftest_case case=${name#case_} result=FAILED"
      sed 's/^/  | /' "$selftest_root/$name/out.txt" | tail -n 15
      failures=$((failures + 1))
    fi
  done
  echo "name=selftest_done failures=$failures"
  [ "$failures" -eq 0 ]
}

# 弄坏开关只在 --selftest 里认；别处设了拒（计数脚本那一份由自证设的标记文件放行，见 selftest）。
if [ -n "${E203_RUN_BREAK:-}" ] && [ "${1:-}" != --selftest ]; then
  echo "  ✗ E203_RUN_BREAK 只给 --selftest 证红用，这一趟不是自证起的" >&2
  echo "     → 怎么办：去掉 E203_RUN_BREAK 再跑；要证红就跑 E203_RUN_BREAK=<项> bash $0 --selftest" >&2
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
[ -f "$counters" ] && [ -f "$histories" ] || { echo "前置文件不在：$counters 或 $histories" >&2; exit 78; }

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
