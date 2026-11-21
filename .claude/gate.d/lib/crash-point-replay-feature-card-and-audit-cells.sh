#!/usr/bin/env bash
# 门禁 crash-point-replay 的函数库：带特性那一趟（feature-gated-tests 格）、产品 GPU 真显卡那一格（product-gpu-real-card-tests）、崩溃流覆盖与只读树那几格、分片那一步与各格的起。
# 由 .claude/gate.d/crash-point-replay.sh 在格名表与 preflight 之后、参数解析之前 source，不单独跑；格名表、参数解析与按格跑的主流程在那份入口里。
# 这份里是函数与它们的注释；函数读的格名、路径与状态变量由入口设，在调用时才取，别份库里的函数同样在调用时才找。

# cpu_judging_lines <日志>：崩溃枚举那一类判段判在 CPU 的行（CRASH_POINT_REPLAY_JUDGE route=cpu，或 CRASH_POINT_REPLAY_PARALLEL_START / _FINISHED 后面带 judge=cpu），
# 一处一行「<它所在的 tests/<名>.rs，或「库与别的二进制」>：<那一行，去掉行首空白>」。记号在行内哪里都认，前面是行首或空白：
# 测试打的一行可能接在别的测试的「test <名> ... 」同一行后面（lib_libtest_result_lines.py 的 CPU_JUDGING_FORMS 同样在行内找，这样的记号在那边照几条同时跑认：打这一行时还没出结果的都算）。
# 弄坏开关 cpu-judging-line-start-only 下只认去掉行首空白后以记号起头的行
cpu_judging_lines() {
  local line_start_only=0
  [[ "$crash_point_replay_break" == cpu-judging-line-start-only ]] && line_start_only=1
  awk -v line_start_only="$line_start_only" -v running_name="$crash_point_replay_running_name" '
    /^[[:space:]]*Running / {
      current = "库与别的二进制"
      if (match($0, "tests/" running_name "+\\.rs \\(")) current = "tests/" substr($0, RSTART + 6, RLENGTH - 8)
      next
    }
    {
      line = $0
      sub(/^[[:space:]]+/, "", line)
      if (line_start_only == 1) {
        judged = line ~ /^CRASH_POINT_REPLAY_JUDGE route=cpu( |$)/ || (line ~ /^CRASH_POINT_REPLAY_PARALLEL_(START|FINISHED) / && line ~ / judge=cpu( |$)/)
      } else {
        judged = line ~ /(^|[[:space:]])CRASH_POINT_REPLAY_JUDGE route=cpu([[:space:]]|$)/
        if (!judged && match(line, /(^|[[:space:]])CRASH_POINT_REPLAY_PARALLEL_(START|FINISHED) /))
          judged = substr(line, RSTART + RLENGTH - 1) ~ /[[:space:]]judge=cpu([[:space:]]|$)/
      }
      if (judged)
        print ((current == "") ? "库与别的二进制" : current) "：" line
    }
  ' "$1"
}


# test_not_run_lines <日志>：用例报的本次未跑行（NOT_RUN_<原因> test=<用例> …，跟在「test <名> ... 」后面的也认，按子串找），
# 一处一行「<它所在的 tests/<名>.rs，或「库与别的二进制」><TAB><从 NOT_RUN_ 起到下一处 NOT_RUN_ 之前>」
test_not_run_lines() {
  awk -v running_name="$crash_point_replay_running_name" '
    /^[[:space:]]*Running / {
      current = "库与别的二进制"
      if (match($0, "tests/" running_name "+\\.rs \\(")) current = "tests/" substr($0, RSTART + 6, RLENGTH - 8)
      next
    }
    {
      rest = $0
      while (match(rest, /NOT_RUN_[A-Z][A-Z0-9_]* test=/) > 0) {
        rest = substr(rest, RSTART)
        tail = substr(rest, 2)
        if (match(tail, /NOT_RUN_[A-Z][A-Z0-9_]* test=/) > 0) { piece = substr(rest, 1, RSTART); rest = substr(tail, RSTART) } else { piece = rest; rest = "" }
        sub(/[[:space:]]+$/, "", piece)
        print ((current == "") ? "库与别的二进制" : current) "\t" piece
      }
    }
  ' "$1"
}


# crash_point_replay_report_not_run <哪一部分没跑：为什么、怎么补>：照共享 lib.sh 的 report_not_run（这一份不 source lib.sh）。
crash_point_replay_report_not_run() {
  echo "  ! 本次未跑（这不是通过）：$1"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$1" >> "$GATE_NOT_RUN_FILE"; fi
}


# count_tests_run_by_file <日志…>：cargo test 输出里逐个集成测试文件跑到的用例数，几份日志相加；一行「文件名<TAB>条数」。
# 一个测试二进制从 cargo 的「Running …tests/<名>.rs (…)」那一行起、到下一行 Running 为止，条数是它 test result 行的 passed 加 failed（ignored 不算跑到）；
# 库（Running unittests …）与别的二进制不打。
count_tests_run_by_file() {
  awk -v running_name="$crash_point_replay_running_name" '
    FNR == 1 { current = "" }
    /^[[:space:]]*Running / {
      current = ""
      if (match($0, "tests/" running_name "+\\.rs \\(")) { current = substr($0, RSTART + 6, RLENGTH - 8); seen[current] = 1 }
      next
    }
    current != "" && /^test result: / {
      if (match($0, /[0-9]+ passed/)) ran[current] += substr($0, RSTART, RLENGTH) + 0
      if (match($0, /[0-9]+ failed/)) ran[current] += substr($0, RSTART, RLENGTH) + 0
    }
    END { for (name in seen) printf "%s\t%d\n", name, ran[name] + 0 }
  ' "$@"
}


# gpu_adapter_not_run_lines <日志>：日志里每一处 NOT_RUN_NO_GPU_ADAPTER（GPU 对拍找不到 GPU 适配器时打的那一行：test=<用例名> reason=<一句>），
# 一处一行「<它所在的 tests/<名>.rs，或「库与别的二进制」><TAB>NOT_RUN_NO_GPU_ADAPTER …」；--nocapture 下它可能跟在「test <名> ... 」后面，按子串找。
gpu_adapter_not_run_lines() {
  awk -v running_name="$crash_point_replay_running_name" '
    /^[[:space:]]*Running / {
      current = "库与别的二进制"
      if (match($0, "tests/" running_name "+\\.rs \\(")) current = "tests/" substr($0, RSTART + 6, RLENGTH - 8)
      next
    }
    {
      rest = $0
      while ((at = index(rest, "NOT_RUN_NO_GPU_ADAPTER ")) > 0) {
        rest = substr(rest, at)
        following = index(substr(rest, 2), "NOT_RUN_NO_GPU_ADAPTER ")
        piece = (following > 0) ? substr(rest, 1, following) : rest
        sub(/[[:space:]]+$/, "", piece)
        print ((current == "") ? "库与别的二进制" : current) "\t" piece
        rest = (following > 0) ? substr(rest, following + 1) : ""
      }
    }
  ' "$1"
}


# report_feature_items_done <这一格的退出码>：feature-gated 的项逐项打 ITEM_DONE（文件头「逐项点名」），stdout。没跑到起 cargo 那一步（读不出挂特性的文件、计划那一步不可用）的不打。
#   秒数：这一项那几段（cargo 的「Running unittests src/lib.rs」「Running tests/<名>.rs」起到下一段）libtest 汇总行「finished in <秒>s」之和，四舍五入成整数；
#   件数：计划里这一项的目标根文件的行（judge 与 refused 算现判、reuse 算复用）；计划算不出的按日志里这一段的结果行数算现判、复用 0；
#   退出码：这一格判红时这一项那几段有 FAILED 的记 1，一段都认不出红的（编不过、判在 CPU 这一类）点到的每一项都记 1；不红时跑到加复用 0 条、有没卡没跑的 GPU 对拍记 77，别的 0。
#   弄坏开关 item-done-unreported 下一行都不打
report_feature_items_done() {
  local cell_exit="$1" item root counts judged reused seconds failed exit_code any_failed=0
  ((crash_point_replay_feature_done_reached)) || return 0
  [[ "$crash_point_replay_break" == item-done-unreported ]] && return 0
  # 每段一行「目标根文件<TAB>finished in 秒数之和<TAB>FAILED 条数<TAB>结果行条数」
  counts="$(awk -v running_name="$crash_point_replay_running_name" '
    /^[[:space:]]*Running / {
      current = ""
      if (match($0, /Running unittests src\/lib\.rs /)) current = "src/lib.rs"
      else if (match($0, "Running tests/" running_name "+\\.rs \\(")) current = substr($0, RSTART + 8, RLENGTH - 10)
      next
    }
    current == "" { next }
    /^test .* \.\.\. FAILED$/ { failed[current]++ }
    /^test .* \.\.\. (ok|FAILED)$/ { results[current]++ }
    /^test result: / && match($0, /finished in [0-9.]+s/) { seconds[current] += substr($0, RSTART + 12, RLENGTH - 13); seen[current] = 1 }
    END { for (name in seen) printf "%s\t%s\t%d\t%d\n", name, seconds[name] + 0, failed[name] + 0, results[name] + 0
          for (name in failed) if (!(name in seen)) printf "%s\t0\t%d\t%d\n", name, failed[name], results[name] + 0 }' "$crash_point_replay_feature_done_log" 2>/dev/null)"
  awk -F'\t' '$3 > 0 { found = 1 } END { exit found ? 0 : 1 }' <<<"$counts" && any_failed=1
  for item in "${crash_point_replay_feature_done_items[@]}"; do
    root="tests/${item#"$FEATURE_GATED_ITEM_PREFIX"}"
    [[ "$item" == "$FEATURE_GATED_LIBRARY_ITEM" ]] && root="src/lib.rs"
    seconds="$(awk -F'\t' -v root="$root" '$1 == root { printf "%d", $2 + 0.5 }' <<<"$counts")"
    failed="$(awk -F'\t' -v root="$root" '$1 == root { print $3 }' <<<"$counts")"
    if [[ -n "$crash_point_replay_feature_done_plan" && -f "$crash_point_replay_feature_done_plan" ]]; then
      read -r judged reused < <(awk -F'\t' -v root="$root" 'NF >= 7 && $5 == root { if ($2 == "judge" || $2 == "refused") judge++; else if ($2 == "reuse") reuse++ }
        END { print judge + 0, reuse + 0 }' "$crash_point_replay_feature_done_plan")
    else
      judged="$(awk -F'\t' -v root="$root" '$1 == root { print $4 }' <<<"$counts")"; reused=0
    fi
    exit_code=0
    if ((cell_exit != 0)); then
      if ((${failed:-0} > 0)) || ((any_failed == 0)); then exit_code=1; fi
    elif crash_point_replay_cell_in "$item" ${crash_point_replay_feature_done_not_run[@]+"${crash_point_replay_feature_done_not_run[@]}"}; then
      exit_code=77
    fi
    printf 'ITEM_DONE item=%s seconds=%s judged=%s reused=%s exit=%s\n' "$item" "${seconds:-0}" "${judged:-0}" "${reused:-0}" "$exit_code"
  done
}


# record_feature_main_pass_first <种类> <计划文件> <计划输出> <带特性那一趟的日志> <退出码> <跑法>：排在最后的目标另起之前先把带特性那一趟的判定记下
#   （文件头「格的次序」：排在最后的那一趟被停了，这一趟判过的不白跑）。记法同这一格收尾：绿的照键记绿，红的记红撤绿，判段判在 CPU 的照 record_cpu_judging_pass。收尾那一遍照记全部（同键同结论再写一次）。
#   交一份计划的拷贝：旁边没有复用哨兵的状态，哨兵只在收尾那一遍比。返回 1 只有 record_green_tests 判按条复用失灵。弄坏开关 feature-main-pass-recorded-with-last-pass 下不先记
record_feature_main_pass_first() {
  local kind="$1" plan_file="$2" plan_output="$3" log="$4" pass_exit="$5" how="$6" cell="$FEATURE_GATED_TESTS_CELL_NAME"
  local accounting="$crash_point_replay_scratch_directory/features.main.accounting" main_plan="$2.main-pass" red_rows="$crash_point_replay_scratch_directory/red-rows.$FEATURE_GATED_TESTS_CELL_NAME.main"
  local -a red_accounting=() cpu_judging=()
  [[ "$crash_point_replay_break" == feature-main-pass-recorded-with-last-pass ]] && return 0
  if ! cp -- "$plan_file" "$main_plan"; then
    echo "  ! $cell：带特性那一趟的判定没先记（拷不了计划文件），等这一格收尾才记"
    return 0
  fi
  echo "  · $cell：要跑得久的目标排在最后，带特性那一趟的判定先记："
  account_test_results "$log" "$accounting" || return 0
  if [[ "$pass_exit" != 0 && "$crash_point_replay_break" != ignore-feature-pass-exit ]]; then
    report_red_test_targets "$log" "$cell" "$red_rows"
    [[ "$crash_point_replay_break" == feature-pass-red-withholds-green ]] || red_accounting=("$accounting")
    record_red_tests "$kind" "$main_plan" "$plan_output" "$red_rows" "$cell" "$how" ${red_accounting[@]+"${red_accounting[@]}"}
    return 0
  fi
  if (( crash_point_replay_gpu_configured )) && [[ "$crash_point_replay_break" != cpu-judging-accepted ]]; then
    mapfile -t cpu_judging < <(cpu_judging_lines "$log")
    if (( ${#cpu_judging[@]} > 0 )); then
      [[ "$crash_point_replay_break" == feature-pass-cpu-judging-records-nothing ]] || record_cpu_judging_pass "$kind" "$main_plan" "$plan_output" "$accounting" "$cell" "$how"
      return 0
    fi
  fi
  record_green_tests "$kind" "$main_plan" "$plan_output" "$accounting" "$cell" "$how"
}


# run_feature_gated_tests_cell <crash-case-replay 格那一趟的日志，没跑传空串>：起 run_feature_gated_tests_pass，收尾逐项打 ITEM_DONE（report_feature_items_done）；退出码照那一趟
run_feature_gated_tests_cell() {
  local pass_exit
  crash_point_replay_feature_done_reached=0; crash_point_replay_feature_done_items=(); crash_point_replay_feature_done_not_run=()
  crash_point_replay_feature_done_log=""; crash_point_replay_feature_done_plan=""
  if run_feature_gated_tests_pass "$@"; then pass_exit=0; else pass_exit=$?; fi
  report_feature_items_done "$pass_exit"
  return "$pass_exit"
}


# run_feature_gated_tests_pass <crash-case-replay 格那一趟的日志，没跑传空串>：判红打出路、返回 1；不红返回 0，结论放进 feature_cell_state。
# 按条复用（文件头「复用」）：这一组特性下每条测试函数各一条判定，与不带特性那一趟同一类、同一份库（种类 <阶段文件名>:test-functions），
# 键相同的经 --skip 跳过，一条都不用现判就不起 cargo；不带特性那一趟打了 NOT_RUN_ 的测试函数所在的测试目标一并带上补判（文件头「多机配置开了 GPU」）。
# 挂特性的文件「跑到 0 条」按跑到的加复用的算，同一条不数两遍。
run_feature_gated_tests_pass() {
  local tests_directory="crates/singlefs-verification/tests" listing feature_list feature_log feature_exit=0 bindgen_arguments=""
  local amplification_library="$crash_point_replay_stage_repository/scripts/crash/crash-amplification-lib.sh"
  local counts_file="$crash_point_replay_scratch_directory/feature-gated-counts"
  local row row_target file_name file_scope file_features scope_text count ran_total=0 gpu_where gpu_line not_run_total
  local -a gated_rows=() features=() target_arguments=() bindgen_setting=() cleared_environment=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE) zero_files=() gpu_not_run=() gpu_test_arguments=()
  if ! listing="$(list_feature_gated_test_files "$tests_directory")"; then
    echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：读不出 $tests_directory 下哪几个测试文件挂特性 cfg（上面是 python 的报错）"
    echo "     → 怎么办：看报错里点名的文件（多半不是 UTF-8 或读不了），修好之后只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
    return 1
  fi
  if [[ -z "$listing" ]]; then
    feature_cell_state="无对象可判"
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：$tests_directory 下 0 个测试文件挂特性 cfg，无对象可判，带特性那一趟不起（往 GATE_NOT_RUN_FILE 记一行）"
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
      printf '%s\n' "crash-point-replay.sh 的 $FEATURE_GATED_TESTS_CELL_NAME 格无对象可判：$tests_directory 下没有挂特性 cfg 的测试文件" >> "$GATE_NOT_RUN_FILE"
    fi
    return 0
  fi
  mapfile -t gated_rows <<< "$listing"
  # 逐项点名（文件头「逐项点名」）：点了 feature-gated 的项只留点名的测试文件、--lib 只在点了 feature-gated:lib 时带；特性照整格那一组（全部挂特性的文件的并集），
  #   键与整格的相同。弄坏开关 feature-items-ignored 下点了也照整格跑；feature-item-features-narrowed 下特性只取点名的文件的
  local -a library_arguments=(--lib) named_rows=()
  if ((${#crash_point_replay_named_feature_items[@]})) && [[ "$crash_point_replay_break" != feature-items-ignored ]]; then
    for row in "${gated_rows[@]}"; do
      crash_point_replay_cell_in "$FEATURE_GATED_ITEM_PREFIX${row%%$'\t'*}" "${crash_point_replay_named_feature_items[@]}" && named_rows+=("$row")
    done
    crash_point_replay_cell_in "$FEATURE_GATED_LIBRARY_ITEM" "${crash_point_replay_named_feature_items[@]}" || library_arguments=()
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：只编、只跑 --item 点名的 ${#crash_point_replay_named_feature_items[@]} 项：${crash_point_replay_named_feature_items[*]}（$tests_directory 下挂特性的 ${#gated_rows[@]} 个文件与 --lib 共 $(( ${#gated_rows[@]} + 1 )) 项；特性照整格那一组）"
    if [[ "$crash_point_replay_break" == feature-item-features-narrowed ]]; then listing="$(printf '%s\n' ${named_rows[@]+"${named_rows[@]}"})"; fi
    gated_rows=(${named_rows[@]+"${named_rows[@]}"})
  fi
  mapfile -t features < <(cut -f3 <<< "$listing" | tr ',' '\n' | grep . | LC_ALL=C sort -u)
  feature_list="$(IFS=,; printf '%s' "${features[*]}")"
  if [[ "$crash_point_replay_break" == features-not-from-source ]]; then feature_list="sample-store,gpu"; fi
  for row in ${gated_rows[@]+"${gated_rows[@]}"}; do row_target="${row%%.rs$'\t'*}"; target_arguments+=(--test "${row_target%"$crash_point_replay_directory_main"}"); done
  # 这一趟跑到的项（收尾时 report_feature_items_done 逐项打 ITEM_DONE，文件头「逐项点名」）
  crash_point_replay_feature_done_items=()
  ((${#library_arguments[@]})) && crash_point_replay_feature_done_items+=("$FEATURE_GATED_LIBRARY_ITEM")
  for row in ${gated_rows[@]+"${gated_rows[@]}"}; do crash_point_replay_feature_done_items+=("$FEATURE_GATED_ITEM_PREFIX${row%%$'\t'*}"); done
  # 测试线程不在这里控制（文件头「feature-gated-tests 格」）；弄坏开关 gpu-test-threads-one 下特性里有 gpu 时带 --test-threads 1
  if [[ ",$feature_list," == *,gpu,* && "$crash_point_replay_break" == gpu-test-threads-one ]]; then gpu_test_arguments=(--test-threads 1); fi
  # 特性里有 gpu（文件头「显卡」带特性那一趟）：这一道不领卡；环境里有 GPU_CARDS_BY_PRIORITY 就原样交给 cargo，没有照起
  if [[ ",$feature_list," == *,gpu,* ]]; then
    crash_point_replay_pass_caller_cards "$FEATURE_GATED_TESTS_CELL_NAME" "带 gpu 特性那一趟"
  else
    reset_service_card
  fi
# librocksdb-sys 的 bindgen 要的头文件目录：与崩溃放量取同一个值，编译产物才共用。在子 shell 里 source 那份库、只读这一个变量。
  if [[ -f "$amplification_library" ]]; then
    bindgen_arguments="$(source "$amplification_library" >/dev/null 2>&1; printf '%s' "${CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS:-}")"
  fi
  if [[ -n "$bindgen_arguments" ]]; then
    bindgen_setting=("BINDGEN_EXTRA_CLANG_ARGS=$bindgen_arguments")
  else
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：从 $amplification_library 读不到 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS，BINDGEN_EXTRA_CLANG_ARGS 照调用方环境（${BINDGEN_EXTRA_CLANG_ARGS:-没设}）；librocksdb-sys 编不过时这一格判红"
  fi
  crash_point_replay_feature_pass_cleared_environment
  cleared_environment=("${crash_point_replay_feature_cleared_environment[@]}")
  crash_point_replay_test_functions_environment=("${cleared_environment[@]}")   # test-functions-request 在交给 cargo 的同一份环境里起（crash_point_replay_test_functions_request 上面那一段）
  feature_log="$crash_point_replay_feature_pass_log"
  : > "$feature_log"
  local feature_kind feature_plan_ok=1 feature_ran=1 feature_name feature_plan_red=0 feature_accounting="$crash_point_replay_scratch_directory/features.accounting"
  feature_kind="$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$FEATURE_GATED_TESTS_CELL_NAME")"
  local feature_plan_file="$crash_point_replay_scratch_directory/features.plan" feature_plan_output="$crash_point_replay_scratch_directory/features.plan.out"
  local -a feature_plan_options=() feature_roots=() feature_names=() supplement_sources=() supplement_arguments=()
  local supplement_source supplement_name
  local -a supplement_target=()
  ((${#library_arguments[@]})) && feature_roots+=(src/lib.rs)
  for row in ${gated_rows[@]+"${gated_rows[@]}"}; do feature_roots+=("tests/${row%%$'\t'*}"); done
  # 补判（文件头「多机配置开了 GPU」）：不带特性那一趟打了 NOT_RUN_ 的测试函数所在的测试目标一并带上；库那一段本来就在。弄坏开关 no-supplement 下不带；
  #   点了 feature-gated 的项的那一趟不另加目标（文件头「逐项点名」）
  if [[ -s "$crash_point_replay_plain_not_run_file" && "$crash_point_replay_break" != no-supplement ]] && ((${#crash_point_replay_named_feature_items[@]} == 0)); then
    mapfile -t supplement_sources < <(cut -f1 "$crash_point_replay_plain_not_run_file" | LC_ALL=C sort -u)
  fi
  for supplement_source in ${supplement_sources[@]+"${supplement_sources[@]}"}; do
    case "$supplement_source" in
      src/lib.rs) continue ;;
      tests/*.rs) supplement_name="${supplement_source#tests/}"; supplement_name="${supplement_name%.rs}"; supplement_target=(--test "${supplement_name%"$crash_point_replay_directory_main"}") ;;
      src/bin/*/main.rs) supplement_name="${supplement_source#src/bin/}"; supplement_target=(--bin "${supplement_name%/main.rs}") ;;
      src/bin/*.rs) supplement_name="${supplement_source#src/bin/}"; supplement_target=(--bin "${supplement_name%.rs}") ;;
      *) continue ;;
    esac
    if [[ " ${target_arguments[*]} " == *" ${supplement_target[*]} "* ]]; then continue; fi
    target_arguments+=("${supplement_target[@]}")
    supplement_arguments+=("${supplement_target[@]}")
    feature_roots+=("$supplement_source")
  done
  if (( ${#supplement_arguments[@]} > 0 )); then
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：不带特性那一趟打了 NOT_RUN_ 的测试函数由这一趟补判，另带上它们所在的测试目标：${supplement_arguments[*]}"
  fi
  IFS=, read -r -a feature_names <<< "$feature_list"
  for feature_name in "${feature_names[@]}"; do feature_plan_options+=(--feature "singlefs-verification/$feature_name"); done
  plan_skip_arguments=()
  if plan_tests "$feature_kind" "$feature_plan_file" "$feature_plan_output" "${feature_plan_options[@]}"; then
    # 这一趟只跑 --lib 与挂特性的那几个测试目标：计数只数这几个目标里的
    awk -F'\t' -v roots="$(IFS=' '; printf '%s' "${feature_roots[*]}")" 'BEGIN { n = split(roots, list, " "); for (i = 1; i <= n; i++) keep[list[i]] = 1 }
      NF >= 7 && ($5 in keep) { print }' "$feature_plan_output" > "$feature_plan_output.kept"
    plan_judge="$(awk -F'\t' '$2 == "judge"' "$feature_plan_output.kept" | grep -c .)"; plan_reuse="$(awk -F'\t' '$2 == "reuse"' "$feature_plan_output.kept" | grep -c .)"
    plan_refused="$(awk -F'\t' '$2 == "refused"' "$feature_plan_output.kept" | grep -c .)"
    skip_arguments_of_plan "$feature_plan_output"
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME 按条复用：--lib 与挂特性的 ${#gated_rows[@]} 个测试目标里 $((plan_judge + plan_reuse + plan_refused)) 条编得进的测试函数，$(plan_report "$feature_plan_output.kept")"
    plan_listing "$feature_plan_output.kept"
    (( plan_judge + plan_refused == 0 )) && feature_ran=0
  else
    feature_plan_ok=0
    : > "$feature_plan_output.kept"
    echo "  ! $FEATURE_GATED_TESTS_CELL_NAME 按条复用算不出（$plan_problem）：这一趟全部现判、一条都不记库"
    ((${#plan_failure[@]})) && { plan_failure_verdict "$FEATURE_GATED_TESTS_CELL_NAME" "${plan_failure[@]}" || feature_plan_red=1; }
    skip_arguments_of_plan -
  fi
  crash_point_replay_feature_plan_ok="$feature_plan_ok"   # 出口 4 那几格读这一趟的行时要（文件头「出口 4 那六格」）
  # 计划那一步 prune-server 不可用（plan_failure_verdict 记了这一格）：不起 cargo，这一格就此收尾
  crash_point_replay_unavailable_only "$FEATURE_GATED_TESTS_CELL_NAME" && return 1
  # 排在最后的测试目标（FEATURE_PASS_LAST_TARGETS；文件头「格的次序」）：特性里有 gpu 时从这一趟拿出来另起一趟，别的目标一趟。弄坏开关 last-pass-merged 下不拿出来
  local -a main_target_arguments=() last_target_arguments=()
  local last_pass_log="$feature_log.last-pass" feature_main_log="$feature_log" last_pass_exit=0 argument_index
  for (( argument_index = 0; argument_index < ${#target_arguments[@]}; argument_index += 2 )); do
    if [[ ",$feature_list," == *,gpu,* && "${target_arguments[argument_index]}" == --test && " ${FEATURE_PASS_LAST_TARGETS[*]} " == *" ${target_arguments[argument_index + 1]} "* ]] \
       && [[ "$crash_point_replay_break" != last-pass-merged ]]; then
      last_target_arguments+=("${target_arguments[@]:argument_index:2}")
    else
      main_target_arguments+=("${target_arguments[@]:argument_index:2}")
    fi
  done
  (( ${#last_target_arguments[@]} > 0 )) && feature_main_log="$feature_log.main"
  echo "── $FEATURE_GATED_TESTS_CELL_NAME：带特性那一趟（特性从 $tests_directory 下 ${#gated_rows[@]} 个挂特性的测试文件现读）：cargo test --release -p singlefs-verification --features $feature_list${library_arguments[*]:+ ${library_arguments[*]}} ${main_target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}"
  if [[ "$crash_point_replay_break" == no-feature-pass ]]; then
    : > "$feature_log"
    feature_main_log="$feature_log"; last_target_arguments=()
  elif (( ! feature_ran )); then
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：这一组特性下要跑的测试函数都有键相同的判定，全部复用，这一趟没起 cargo"
  else
    # 特性里没有 gpu 时按测试目标并行起，有 gpu 的照旧一条 cargo test（文件头「按条复用两趟按测试目标并行起」）
    local -a feature_runner=()
    crash_point_replay_test_runner feature_runner "$([[ ",$feature_list," == *,gpu,* ]] && echo gpu)"
    env "${cleared_environment[@]}" ${crash_point_replay_card_unset[@]+"${crash_point_replay_card_unset[@]}"} ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
      ${crash_point_replay_card_quiesce_environment[@]+"${crash_point_replay_card_quiesce_environment[@]}"} ${bindgen_setting[@]+"${bindgen_setting[@]}"} ${crash_point_replay_card_setting[@]+"${crash_point_replay_card_setting[@]}"} \
      ${crash_point_replay_card_quiesce_prefix[@]+"${crash_point_replay_card_quiesce_prefix[@]}"} "${feature_runner[@]}" --release --no-fail-fast -p singlefs-verification --features "$feature_list" ${library_arguments[@]+"${library_arguments[@]}"} ${main_target_arguments[@]+"${main_target_arguments[@]}"} -- --nocapture ${gpu_test_arguments[@]+"${gpu_test_arguments[@]}"} ${plan_skip_arguments[@]+"${plan_skip_arguments[@]}"} 2>&1 \
      | crash_point_replay_pass_output "$feature_log"
    feature_exit="${PIPESTATUS[0]}"
    report_bulk_lines "$FEATURE_GATED_TESTS_CELL_NAME" "带特性那一趟（--features $feature_list）" "$feature_log"
    crash_point_replay_note_stop "$FEATURE_GATED_TESTS_CELL_NAME 带特性那一趟" "$feature_exit" "$feature_log"
    # 每一趟当场写结局（文件头「每一趟当场写结局」）：排在最后的那一趟另起之前先打这一趟的，那一趟撞顶被杀时这一趟的结局已在外层日志里
    if (( ${#last_target_arguments[@]} > 0 )); then cp -- "$feature_log" "$feature_main_log"; fi
    report_pass_outcome "$FEATURE_GATED_TESTS_CELL_NAME" "带特性那一趟（--features $feature_list）" "$feature_exit" "$feature_main_log"
    # 这几个目标排在这一格、也排在全部要起 cargo 的格的最后（文件头「格的次序」）：带特性那一趟的判定先记，它被停了不接着起排在最后的那一趟
    if (( ${#last_target_arguments[@]} > 0 )) && [[ -n "$crash_point_replay_stop_reason" ]]; then
      echo "  · $FEATURE_GATED_TESTS_CELL_NAME：${crash_point_replay_stop_reason}：排在最后的目标（${last_target_arguments[*]}）不接着跑"
      last_target_arguments=()
    elif (( ${#last_target_arguments[@]} > 0 && feature_plan_ok )); then
      record_feature_main_pass_first "$feature_kind" "$feature_plan_file" "$feature_plan_output" "$feature_main_log" "$feature_exit" \
        "cargo test --release -p singlefs-verification --features $feature_list（带特性那一趟）" || feature_plan_red=1
      if crash_point_replay_unavailable_only "$FEATURE_GATED_TESTS_CELL_NAME"; then   # 记库那一步 prune-server 不可用：不接着起排在最后的那一趟，不可用那一句已经打了
        return 1
      fi
    fi
    if (( ${#last_target_arguments[@]} > 0 )); then
      echo "── $FEATURE_GATED_TESTS_CELL_NAME：要跑得久的 GPU 测试目标另起一趟，排在最后：cargo test --release -p singlefs-verification --features $feature_list ${last_target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}"
      env "${cleared_environment[@]}" ${crash_point_replay_card_unset[@]+"${crash_point_replay_card_unset[@]}"} ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
        ${crash_point_replay_card_quiesce_environment[@]+"${crash_point_replay_card_quiesce_environment[@]}"} ${bindgen_setting[@]+"${bindgen_setting[@]}"} ${crash_point_replay_card_setting[@]+"${crash_point_replay_card_setting[@]}"} \
        ${crash_point_replay_card_quiesce_prefix[@]+"${crash_point_replay_card_quiesce_prefix[@]}"} "${feature_runner[@]}" --release --no-fail-fast -p singlefs-verification --features "$feature_list" "${last_target_arguments[@]}" -- --nocapture ${gpu_test_arguments[@]+"${gpu_test_arguments[@]}"} ${plan_skip_arguments[@]+"${plan_skip_arguments[@]}"} 2>&1 \
        | crash_point_replay_pass_output "$last_pass_log"
      last_pass_exit="${PIPESTATUS[0]}"
      report_bulk_lines "$FEATURE_GATED_TESTS_CELL_NAME" "排在最后的那一趟（${last_target_arguments[*]}）" "$last_pass_log"
      crash_point_replay_note_stop "$FEATURE_GATED_TESTS_CELL_NAME 排在最后的那一趟" "$last_pass_exit" "$last_pass_log"
      report_pass_outcome "$FEATURE_GATED_TESTS_CELL_NAME" "排在最后的那一趟（${last_target_arguments[*]}）" "$last_pass_exit" "$last_pass_log"
      (( feature_exit != 0 )) || feature_exit="$last_pass_exit"
      # 两趟的输出并成一份（计数、没卡没跑、判在 CPU、红的目标都照它读）；逐条认结局两趟各认一遍
      cat -- "$last_pass_log" >> "$feature_log"
    fi
  fi
  # 跑到了这一步（起过 cargo，或全部复用没起）：收尾逐项打 ITEM_DONE（report_feature_items_done 读这几个全局）
  crash_point_replay_feature_done_reached=1
  crash_point_replay_feature_done_log="$feature_log"
  crash_point_replay_feature_done_plan="$([[ "$feature_plan_ok" == 1 ]] && printf '%s' "$feature_plan_output.kept")"
  # 逐条认结局的入参（文件头「复用」；排在最后的那一趟另认）：判红、判绿两条路同一份
  local feature_accounting_ok=0 last_pass_log_to_account=-
  local -a feature_red_accounting=()
  (( ${#last_target_arguments[@]} > 0 )) && last_pass_log_to_account="$last_pass_log"
  : >"$feature_accounting"
  if [[ "$feature_exit" != 0 && "$crash_point_replay_break" != ignore-feature-pass-exit ]]; then
    tail -40 "$feature_log"
    grep -E '^test .* FAILED$|^error: test failed, to rerun pass' "$feature_log" | sed 's/^/    /' || true
    (( feature_plan_ok )) && failed_tests_changed_units "$feature_plan_output.kept" "$feature_log"
    report_red_test_targets "$feature_log" "$FEATURE_GATED_TESTS_CELL_NAME" "$crash_point_replay_scratch_directory/red-rows.$FEATURE_GATED_TESTS_CELL_NAME"
    # 同一趟判绿的照键记绿（文件头「复用」）：弄坏开关 feature-pass-red-withholds-green 下不认、不记
    if (( feature_plan_ok )) && [[ "$crash_point_replay_break" != feature-pass-red-withholds-green ]] \
         && account_feature_pass "$feature_main_log" "$last_pass_log_to_account" "$feature_accounting"; then
      feature_red_accounting=("$feature_accounting")
    fi
    (( feature_plan_ok )) && record_red_tests "$feature_kind" "$feature_plan_file" "$feature_plan_output" "$crash_point_replay_scratch_directory/red-rows.$FEATURE_GATED_TESTS_CELL_NAME" \
      "$FEATURE_GATED_TESTS_CELL_NAME" "cargo test --release -p singlefs-verification --features $feature_list（带特性那一趟）" ${feature_red_accounting[@]+"${feature_red_accounting[@]}"}
    echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：带特性那一趟 cargo test 退 $feature_exit（上面是它的尾部与全部红的用例、测试目标）"
    echo "     → 怎么办：单跑看细节：BINDGEN_EXTRA_CLANG_ARGS=\"$bindgen_arguments\" cargo test --release -p singlefs-verification --features $feature_list${library_arguments[*]:+ ${library_arguments[*]}} ${target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}；"
    echo "                编不过多半是 librocksdb-sys 现编 C++ 缺 libclang、C++ 编译器或这个变量；用例红照断言消息改；没卡的机器上 GPU 对拍该打一行 NOT_RUN_NO_GPU_ADAPTER 之后返回成功，panic 就是那条用例没接这个约定。"
    return 1
  fi
  # 配置开了 GPU、这一趟编进了 gpu（文件头「多机配置开了 GPU」）：判段必须在 GPU，日志里有判在 CPU 的行判红，同一趟的判定照记
  if [[ ",$feature_list," == *,gpu,* ]] && (( crash_point_replay_gpu_configured )) && [[ "$crash_point_replay_break" != cpu-judging-accepted ]]; then
    local feature_cpu_judging=()
    mapfile -t feature_cpu_judging < <(cpu_judging_lines "$feature_log")
    if (( ${#feature_cpu_judging[@]} > 0 )); then
      printf '      %s\n' "${feature_cpu_judging[@]:0:20}"
      # 判在 CPU 归得到的那几条记红、归不到的整个二进制不记、别的判绿的照键记绿（record_cpu_judging_pass；逐条认结局的入参同判红那一段）：
      # 弄坏开关 feature-pass-cpu-judging-records-nothing 下一条都不记
      if (( feature_plan_ok && feature_ran )) && [[ "$crash_point_replay_break" != feature-pass-cpu-judging-records-nothing ]] \
           && account_feature_pass "$feature_main_log" "$last_pass_log_to_account" "$feature_accounting"; then
        record_cpu_judging_pass "$feature_kind" "$feature_plan_file" "$feature_plan_output" "$feature_accounting" "$FEATURE_GATED_TESTS_CELL_NAME" \
          "cargo test --release -p singlefs-verification --features $feature_list（带特性那一趟）"
      fi
      echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：带 gpu 特性那一趟判在 CPU：多机配置 $crash_point_replay_configuration_file 写了 ENABLE_GPU=1、这一趟编进了 gpu，日志里有 ${#feature_cpu_judging[@]} 处判段判在 CPU（上面列了前 20 处）"
      echo "     → 怎么办：看那几行的 why_not_the_gpu=（选路在 crates/singlefs-verification/src/crash_state_enumeration.rs 的 crash_point_replay_judge_route）：用例读的配置与这里判的不是同一份，就对齐 SINGLEFS_MULTI_HOST_CONFIG；改完只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
      return 1
    fi
  fi
  count_tests_run_by_file "$feature_log" ${1:+"$1"} > "$counts_file"
  # 键相同而复用的也算跑到（这一组特性下的计划，与不带特性那一趟的计划）：一条一行「文件名<TAB>条数」。两趟一份库，同一条测试不数两遍：
  # 这一趟现判（跑了、日志里数过）的，与不带特性那一趟现判、日志也数进来了的，不再按复用数
  local -a reused_plan_files=("$feature_plan_output.kept")
  [[ -f "$crash_point_replay_scratch_directory/plain.plan.out" ]] && reused_plan_files+=("$crash_point_replay_scratch_directory/plain.plan.out")
  awk -F'\t' -v kept="$feature_plan_output.kept" -v plain_counted="${1:+1}" '
    NF >= 7 && $5 ~ /^tests\// {
      if ($2 == "reuse") reuse[$1] = $5
      else if (($2 == "judge" || $2 == "refused") && (FILENAME == kept || plain_counted == 1)) ran[$1] = 1
    }
    END { for (name in reuse) if (!(name in ran)) { source = reuse[name]; sub(/^tests\//, "", source); count[source]++ } for (source in count) printf "%s\t%d\n", source, count[source] }' \
    "${reused_plan_files[@]}" > "$counts_file.reused"
  for row in "${gated_rows[@]}"; do
    IFS=$'\t' read -r file_name file_scope file_features <<< "$row"
    count="$(awk -F'\t' -v name="$file_name" '$1 == name { print $2 }' "$counts_file")"
    count="${count:-0}"
    count=$((count + $(awk -F'\t' -v name="$file_name" '$1 == name { total += $2 } END { print total + 0 }' "$counts_file.reused")))
    ran_total=$((ran_total + count))
    if [[ "$count" == 0 && "$crash_point_replay_break" != ignore-zero-feature-files ]]; then
      zero_files+=("tests/$file_name")
      crash_point_replay_feature_done_not_run+=("$FEATURE_GATED_ITEM_PREFIX$file_name")
      scope_text="逐条挂"; if [[ "$file_scope" == whole-file ]]; then scope_text="整份挂"; fi
      crash_point_replay_report_not_run "验证包 $tests_directory/$file_name ${scope_text}特性 $file_features，跑到它的用例 0 条（cargo 输出里没有它那一段，或只有 ignored）：给它补一条不标 ignored 的用例，或查 cfg 里的特性名与 Cargo.toml 的 [features] 对不对得上"
    fi
  done
  if [[ "$crash_point_replay_break" != ignore-gpu-not-run ]]; then
    while IFS=$'\t' read -r gpu_where gpu_line; do
      [[ -n "$gpu_line" ]] || continue
      gpu_not_run+=("$gpu_line")
      if [[ "$gpu_where" == tests/* ]]; then crash_point_replay_feature_done_not_run+=("$FEATURE_GATED_ITEM_PREFIX${gpu_where#tests/}")
      else crash_point_replay_feature_done_not_run+=("$FEATURE_GATED_LIBRARY_ITEM"); fi
      crash_point_replay_report_not_run "GPU 对拍没跑（${gpu_where}；这台机器找不到 GPU 适配器）：${gpu_line}；在有卡的机器上跑这一格补上"
    done < <(gpu_adapter_not_run_lines "$feature_log")
  fi
  # 逐条认结局与记号（account_feature_pass，入参在判红那一段之前定好），判绿的照键记库
  if (( feature_ran )) && [[ "$crash_point_replay_break" != no-feature-pass ]] \
       && account_feature_pass "$feature_main_log" "$last_pass_log_to_account" "$feature_accounting"; then
    feature_accounting_ok=1
  fi
  if (( feature_plan_ok && feature_ran && feature_accounting_ok )); then
    record_green_tests "$feature_kind" "$feature_plan_file" "$feature_plan_output" "$feature_accounting" "$FEATURE_GATED_TESTS_CELL_NAME" \
      "cargo test --release -p singlefs-verification --features $feature_list（带特性那一趟）" || feature_plan_red=1
  fi
  supplement_plain_not_run "$feature_plan_output" "$feature_accounting" "$feature_plan_ok"
  crash_point_replay_unavailable_only "$FEATURE_GATED_TESTS_CELL_NAME" && return 1   # 记库那一步 prune-server 不可用：不可用那一句已经打了，不是失灵
  if (( feature_plan_red )); then
    echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME 这一格判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；带特性那一趟照旧全部现判跑完了"
    echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再只跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME；不能复用就判红，不退回全部现判当绿"
    return 1
  fi
  not_run_total=$(( ${#zero_files[@]} + ${#gpu_not_run[@]} ))
  if (( not_run_total > 0 )); then
    feature_cell_state="有 ${not_run_total} 处本次未跑"
    echo "  ! $FEATURE_GATED_TESTS_CELL_NAME 不算全绿：${#gated_rows[@]} 个挂特性的测试文件（特性 $feature_list）跑到 ${ran_total} 条，本次未跑 ${not_run_total} 处：编进来 0 条的文件 ${#zero_files[@]} 个（${zero_files[*]:-无}），没卡没跑的 GPU 对拍 ${#gpu_not_run[@]} 条"
    echo "     → 编进来 0 条的给它补不标 ignored 的用例；GPU 对拍在有卡的机器上补跑：bash .claude/gate.d/crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
  else
    feature_cell_state="绿"
    echo "  ✓ $FEATURE_GATED_TESTS_CELL_NAME：${#gated_rows[@]} 个挂特性的测试文件（特性 $feature_list）每个都编进来、跑到了用例，共 ${ran_total} 条（含复用；$( (( feature_plan_ok )) && printf '按条：%s' "$(plan_report "$feature_plan_output.kept" 0)" || printf '按条复用算不出，全部现判')）"
  fi
  return 0
}


# ── 产品 GPU 包带 tests-on-a-real-gpu 的真显卡对拍（product-gpu-real-card-tests 格；文件头「product-gpu-real-card-tests 格」）─────────
# product_gpu_target_rows <包目录> <特性>：这个包里开了这个特性才编的测试目标，一个一行「目标名<TAB>根文件（包目录起）」，照目标名排序：Cargo.toml 的 [[test]] 里
# required-features 有这个特性的（根文件取它的 path，没写取 tests/<名>.rs，目录式取 tests/<名>/main.rs），加 tests/ 里每个测试目标（顶层的 .rs、目录式的 <名>/main.rs 连同它 mod 声明到的子模块；
# // 起头的行不读）里 cfg 写了 feature = "<特性>" 的。按包与特性认，不按文件名写死。
# 退出码：0 有（逐行打）、3 包不在（没有 Cargo.toml）、4 包的 [features] 里没有这个特性、5 包有这个特性而没有测试目标挂它、6 读不了（原因打在标准错误）。
product_gpu_target_rows() { crash_point_replay_list_targets product-gpu "$1" "$2"; }


# reset_service_card：这一趟不交卡，清掉 crash_point_replay_pass_caller_cards 放进的几个全局数组
reset_service_card() {
  crash_point_replay_card_setting=() crash_point_replay_card_unset=() crash_point_replay_card_quiesce_prefix=() crash_point_replay_card_quiesce_environment=()
}


# crash_point_replay_pass_caller_cards <格名> <哪一趟>：带 gpu 特性那一趟与真显卡对拍那一趟（文件头「显卡」）不领卡、不挑卡、不清场：环境里有 GPU_CARDS_BY_PRIORITY 就照
#   crash_point_replay_forward_card 放进全局数组 crash_point_replay_card_setting（交给 cargo 的 env 赋值）、打一行是哪一串；没有打一行、照起。都返回 0。
#   弄坏开关 service-card-not-forwarded、stage-quiesces-by-itself 在 crash_point_replay_forward_card
crash_point_replay_pass_caller_cards() {
  reset_service_card
  local card="${GPU_CARDS_BY_PRIORITY:-}"
  if [[ -z "$card" ]]; then
    echo "  · $1：$2：环境里没有 GPU_CARDS_BY_PRIORITY，照起、不领卡（卡归 GPU 批处理验证服务）"
    return 0
  fi
  crash_point_replay_forward_card "$card"
  echo "  · $1：$2：环境里有 GPU_CARDS_BY_PRIORITY=$card，原样交给 cargo（这一道不领卡、不挑卡、不另套清场包装）"
}


# crash_point_replay_forward_card <卡串，可空>：把环境里的卡放进交给 cargo 的 env（crash_point_replay_card_setting）。
#   弄坏开关 service-card-not-forwarded：有也不交，env 里另 -u 掉（crash_point_replay_card_unset）；
#   stage-quiesces-by-itself：这一道自己在这一趟前面套 scripts/crash/gpu-quiesce-run.sh（crash_point_replay_card_quiesce_prefix；清场归 GPU 批处理验证服务，这一层本不该有；
#   清场配置写成草稿目录里只写假清场命令的一份，状态目录与日志目录也在草稿目录里：开关只在样本根上认）
crash_point_replay_forward_card() {
  if [[ "$crash_point_replay_break" == service-card-not-forwarded ]]; then
    crash_point_replay_card_unset=(-u GPU_CARDS_BY_PRIORITY)
  elif [[ -n "$1" ]]; then
    crash_point_replay_card_setting=("GPU_CARDS_BY_PRIORITY=$1")
  fi
  if [[ "$crash_point_replay_break" == stage-quiesces-by-itself ]]; then
    printf '%s\n' QUIESCE_STOP_COMMAND=true QUIESCE_STOPPED_CHECK_COMMAND=true > "$crash_point_replay_scratch_directory/stage-quiesce.env"
    crash_point_replay_card_quiesce_prefix=(bash "$crash_point_replay_stage_repository/scripts/crash/gpu-quiesce-run.sh" --)
    crash_point_replay_card_quiesce_environment=("SINGLEFS_MULTI_HOST_CONFIG=$crash_point_replay_scratch_directory/stage-quiesce.env"
      "GPU_QUIESCE_STATE_DIRECTORY=$crash_point_replay_scratch_directory/gpu-quiesce-state" "GPU_QUIESCE_LOG_DIRECTORY=$crash_point_replay_scratch_directory/gpu-quiesce-logs")
  fi
}


# run_product_gpu_real_card_cell：判红打出路、返回 1；不红返回 0，结论放进 product_gpu_cell_state。
# 按条复用同 run_feature_gated_tests_cell（同一类判定库 <阶段文件名>:test-functions，键相同的经 --skip 跳过，一条都不用现判就不起 cargo）；
# 只跑对拍目标（--test <名>…，不带 --lib：库单测是不带特性那一趟的事，也不上卡）；不领卡，环境里有 GPU_CARDS_BY_PRIORITY 就原样交给 cargo（文件头「显卡」）。
run_product_gpu_real_card_cell() {
  local package="$PRODUCT_GPU_PACKAGE_NAME" feature="$PRODUCT_GPU_FEATURE_NAME" cell="$PRODUCT_GPU_REAL_CARD_CELL_NAME"
  local package_directory="crates/$PRODUCT_GPU_PACKAGE_NAME" scratch="$crash_point_replay_scratch_directory"
  local log="$crash_point_replay_pass_log_directory/$PRODUCT_GPU_PACKAGE_NAME-real-card.log" accounting="$crash_point_replay_scratch_directory/product-gpu.accounting"
  local plan_file="$crash_point_replay_scratch_directory/product-gpu.plan" plan_output="$crash_point_replay_scratch_directory/product-gpu.plan.out"
  local listing listing_exit=0 row kind cell_exit=0 plan_ok=1 plan_red=0 ran=1 ran_total=0 count file_name not_run_total gpu_where gpu_line
  local target_name target_source
  local plan_package="$PRODUCT_GPU_PACKAGE_NAME"   # plan_tests 与 seed_prior_verdicts 读它：算产品 GPU 包的键
  local -a listed_rows=() target_rows=() target_arguments=() zero_files=() gpu_not_run=() gpu_test_arguments=() cleared_environment=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE)
  local -a feature_roots=()
  if listing="$(product_gpu_target_rows "$package_directory" "$feature" 2>"$scratch/product-gpu.targets.err")"; then
    :
  else
    listing_exit=$?
  fi
  if (( listing_exit == 0 )) && [[ "$crash_point_replay_break" == product-gpu-targets-by-file-name ]]; then
    # 弄坏开关：对拍目标按文件名认（只认 tests/checksum 起头的），不按包与特性认
    listing="$(awk -F'\t' '$2 ~ /^tests\/checksum/' <<< "$listing")"
    [[ -n "$listing" ]] || listing_exit=5
  fi
  case "$listing_exit" in
    0) ;;
    3|5)
      product_gpu_cell_state="无对象可判"
      echo "  · $cell：$( ((listing_exit == 3)) && echo "$package_directory/Cargo.toml 不在（没有产品 GPU 包）" || echo "$package 的 tests/ 与 [[test]] 里没有测试目标挂 $feature" )，无对象可判，这一趟不起（往 GATE_NOT_RUN_FILE 记一行）"
      if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
        printf '%s\n' "crash-point-replay.sh 的 $cell 格无对象可判：$package 包里没有开 $feature 才编的测试目标" >> "$GATE_NOT_RUN_FILE"
      fi
      return 0 ;;
    4)
      echo "  ✗ $cell：$package_directory/Cargo.toml 的 [features] 里没有 $feature：这一格认的特性名对不上包了（改名或删了而这一格还认旧名，真显卡对拍就没有门禁在跑）"
      echo "     → 怎么办：包里改了特性名，就把这一道的 PRODUCT_GPU_FEATURE_NAME 与 crates-mutation-rows.py 的 GPU_TEST_FEATURES 一起改成新名；"
      echo "                对拍测试整个撤了，就把这一格与那几处一起撤；改完只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $cell"
      return 1 ;;
    *)
      echo "  ✗ $cell：读不出 $package_directory 里哪几个测试目标挂 $feature（$(tail -n 1 "$scratch/product-gpu.targets.err" 2>/dev/null)）"
      echo "     → 怎么办：看上面那句里点名的文件（多半 Cargo.toml 写坏了、测试文件不是 UTF-8），修好之后只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $cell"
      return 1 ;;
  esac
  mapfile -t listed_rows <<< "$listing"
  [[ "$crash_point_replay_break" == product-gpu-test-threads-one ]] && gpu_test_arguments=(--test-threads 1)
  crash_point_replay_pass_caller_cards "$cell" "真显卡对拍那一趟"
  for row in "${listed_rows[@]}"; do
    IFS=$'\t' read -r target_name target_source <<< "$row"
    target_rows+=("$target_name"$'\t'"$target_source")
    target_arguments+=(--test "$target_name")
    feature_roots+=("$target_source")
  done
  crash_point_replay_feature_pass_cleared_environment
  cleared_environment=("${crash_point_replay_feature_cleared_environment[@]}")
  crash_point_replay_test_functions_environment=("${cleared_environment[@]}")   # 同 run_feature_gated_tests_cell：test-functions-request 在交给 cargo 的同一份环境里起
  : > "$log"
  # 按条复用（同 run_feature_gated_tests_cell）：这一组特性下对拍目标里每条测试函数各一条判定
  kind="$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$cell")"
  plan_skip_arguments=()
  if plan_tests "$kind" "$plan_file" "$plan_output" --feature "$package/$feature"; then
    awk -F'\t' -v roots="$(IFS=' '; printf '%s' "${feature_roots[*]}")" 'BEGIN { n = split(roots, list, " "); for (i = 1; i <= n; i++) keep[list[i]] = 1 }
      NF >= 7 && ($5 in keep) { print }' "$plan_output" > "$plan_output.kept"
    plan_judge="$(awk -F'\t' '$2 == "judge"' "$plan_output.kept" | grep -c .)"; plan_reuse="$(awk -F'\t' '$2 == "reuse"' "$plan_output.kept" | grep -c .)"
    plan_refused="$(awk -F'\t' '$2 == "refused"' "$plan_output.kept" | grep -c .)"
    skip_arguments_of_plan "$plan_output.kept"
    echo "  · $cell 按条复用：$package 的 ${#target_rows[@]} 个对拍测试目标里 $((plan_judge + plan_reuse + plan_refused)) 条编得进的测试函数，$(plan_report "$plan_output.kept")"
    plan_listing "$plan_output.kept"
    (( plan_judge + plan_refused == 0 )) && ran=0
  else
    plan_ok=0
    : > "$plan_output.kept"
    echo "  ! $cell 按条复用算不出（$plan_problem）：这一趟全部现判、一条都不记库"
    ((${#plan_failure[@]})) && { plan_failure_verdict "$cell" "${plan_failure[@]}" || plan_red=1; }
  fi
  # 计划那一步 prune-server 不可用（plan_failure_verdict 记了这一格）：不起 cargo，这一格就此收尾
  crash_point_replay_unavailable_only "$cell" && return 1
  echo "── $cell：cargo test --release -p $package --features $feature ${target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}"
  if (( ! ran )); then
    echo "  · $cell：这一组特性下要跑的测试函数都有键相同的判定，全部复用，这一趟没起 cargo"
  elif [[ "$crash_point_replay_break" == product-gpu-cell-skipped ]]; then
    : > "$log"   # 弄坏开关：要现判的也不起对拍那一趟（日志空着、退出码当 0）
  else
    # 两段（文件头「product-gpu-real-card-tests 格」两段那一段）：先只录（不带卡：环境里的 GPU_CARDS_BY_PRIORITY 也摘掉），录成了再只判。
    #   弄坏开关 product-gpu-record-holds-the-card：只录那一趟也带着环境里的卡；product-gpu-record-exit-ignored：只录那一趟退非 0 照样往下判；
    #   product-gpu-judge-without-the-stage：只判那一趟不带 SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge（同一个进程里又录一遍）
    local record_log="$crash_point_replay_pass_log_directory/$PRODUCT_GPU_PACKAGE_NAME-real-card.record.log" record_exit=0
    local -a record_card_setting=() judge_stage_setting=(SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge)
    [[ "$crash_point_replay_break" == product-gpu-judge-without-the-stage ]] && judge_stage_setting=()
    if [[ "$crash_point_replay_break" == product-gpu-record-holds-the-card ]]; then
      record_card_setting=(${crash_point_replay_card_setting[@]+"${crash_point_replay_card_setting[@]}"})
    fi
    echo "── $cell 只录那一趟（不持卡）：SINGLEFS_CRASH_AMPLIFICATION_STAGE=record cargo test --release -p $package --features $feature ${target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}"
    env "${cleared_environment[@]}" -u GPU_CARDS_BY_PRIORITY ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
      ${record_card_setting[@]+"${record_card_setting[@]}"} SINGLEFS_CRASH_AMPLIFICATION_STAGE=record \
      cargo test --release --no-fail-fast -p "$package" --features "$feature" "${target_arguments[@]}" -- --nocapture ${gpu_test_arguments[@]+"${gpu_test_arguments[@]}"} \
      ${plan_skip_arguments[@]+"${plan_skip_arguments[@]}"} 2>&1 \
      | crash_point_replay_pass_output "$record_log"
    record_exit="${PIPESTATUS[0]}"
    report_bulk_lines "$cell" "真显卡对拍只录那一趟" "$record_log"
    crash_point_replay_note_stop "$cell 真显卡对拍只录那一趟" "$record_exit" "$record_log"
    if [[ "$record_exit" != 0 && "$crash_point_replay_break" != product-gpu-record-exit-ignored ]]; then
      tail -40 "$record_log"
      echo "  ✗ $cell：真显卡对拍只录那一趟（SINGLEFS_CRASH_AMPLIFICATION_STAGE=record，不持卡）退 $record_exit（上面是它的尾部），只判那一趟没起"
      echo "     → 怎么办：看日志 $record_log 里 panicked 那几行；只录那一趟在 CPU 上造输入、写进判定库，不碰卡，红在这里是造输入或写库那一侧（判定库不收空输入）。改完只重跑这一格：task-server/task submit -- bash .claude/gate.d/crash-point-replay.sh --check $cell"
      return 1
    fi
    echo "  · $cell：只录那一趟录完了（不持卡，$(grep -c '^RECORDED_ONLY ' "$record_log" || true) 条打了 RECORDED_ONLY），接着只判"
    env "${cleared_environment[@]}" ${crash_point_replay_card_unset[@]+"${crash_point_replay_card_unset[@]}"} ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
      ${crash_point_replay_card_quiesce_environment[@]+"${crash_point_replay_card_quiesce_environment[@]}"} ${crash_point_replay_card_setting[@]+"${crash_point_replay_card_setting[@]}"} \
      ${judge_stage_setting[@]+"${judge_stage_setting[@]}"} \
      ${crash_point_replay_card_quiesce_prefix[@]+"${crash_point_replay_card_quiesce_prefix[@]}"} \
      cargo test --release --no-fail-fast -p "$package" --features "$feature" "${target_arguments[@]}" -- --nocapture ${gpu_test_arguments[@]+"${gpu_test_arguments[@]}"} \
      ${plan_skip_arguments[@]+"${plan_skip_arguments[@]}"} 2>&1 \
      | crash_point_replay_pass_output "$log"
    cell_exit="${PIPESTATUS[0]}"
    report_bulk_lines "$cell" "真显卡对拍那一趟" "$log"
    crash_point_replay_note_stop "$cell 真显卡对拍那一趟" "$cell_exit" "$log"
  fi
  # 逐条认结局的入参（文件头「复用」）：判红、判绿两条路同一份
  local accounting_ok=0
  local -a red_accounting=()
  : > "$accounting"
  if [[ "$cell_exit" != 0 && "$crash_point_replay_break" != product-gpu-exit-ignored ]]; then
    tail -40 "$log"
    grep -E '^test .* FAILED$|^error: test failed, to rerun pass' "$log" | sed 's/^/    /' || true
    (( plan_ok )) && failed_tests_changed_units "$plan_output.kept" "$log"
    report_red_test_targets "$log" "$cell" "$scratch/red-rows.$cell"
    # 同一趟判绿的照键记绿（文件头「复用」）：弄坏开关 product-gpu-red-withholds-green 下不认、不记
    if (( plan_ok )) && [[ "$crash_point_replay_break" != product-gpu-red-withholds-green ]] && account_test_results "$log" "$accounting"; then
      red_accounting=("$accounting")
    fi
    (( plan_ok )) && record_red_tests "$kind" "$plan_file" "$plan_output" "$scratch/red-rows.$cell" "$cell" "cargo test --release -p $package --features $feature（真显卡对拍）" \
      ${red_accounting[@]+"${red_accounting[@]}"}
    echo "  ✗ $cell：真显卡对拍那一趟 cargo test 退 $cell_exit（上面是它的尾部与全部红的用例、测试目标）：有卡的机器上对拍判红，GPU 核与 CPU 参照对不上"
    echo "     → 怎么办：只重跑这一格看细节：task-server/task submit -- bash .claude/gate.d/crash-point-replay.sh --check $cell；"
    echo "                断言消息里点名对不上的长度与起点余数；改 GPU 核（WGSL）还是改 CPU 参照先想清楚是哪一种，直接改断言等于把对拍关掉；"
    echo "                没卡的机器上对拍该打一行 NOT_RUN_NO_GPU_ADAPTER 之后返回成功，panic 就是那条用例没接这个约定。"
    return 1
  fi
  # 跑到的条数：日志里数到的加上键相同而复用的（复用的没跑、不在日志里）
  count_tests_run_by_file "$log" > "$scratch/product-gpu.counts"
  awk -F'\t' 'NF >= 7 && $2 == "reuse" && $5 ~ /^tests\// { count[$5]++ } END { for (source in count) printf "%s\t%d\n", substr(source, 7), count[source] }' "$plan_output.kept" > "$scratch/product-gpu.counts.reused"
  for row in "${target_rows[@]}"; do
    file_name="${row#*$'\t'}"; file_name="${file_name#tests/}"
    [[ "$crash_point_replay_break" != directory-target-count-by-basename ]] || file_name="${file_name##*/}"   # 弄坏开关：只取文件名，目录式的 <名>/main.rs 变成 main.rs，对不上日志里数到的 <名>/main.rs
    count="$(awk -F'\t' -v name="$file_name" '$1 == name { print $2 }' "$scratch/product-gpu.counts")"
    count=$(( ${count:-0} + $(awk -F'\t' -v name="$file_name" '$1 == name { total += $2 } END { print total + 0 }' "$scratch/product-gpu.counts.reused") ))
    ran_total=$((ran_total + count))
    if [[ "$count" == 0 && "$crash_point_replay_break" != product-gpu-zero-ignored ]]; then
      zero_files+=("$package_directory/${row#*$'\t'}")
      crash_point_replay_report_not_run "产品 GPU 包 $package_directory/${row#*$'\t'} 挂特性 $feature，真显卡对拍跑到它的用例 0 条（cargo 输出里没有它那一段，或只有 ignored）：给它补一条不标 ignored 的对拍用例，或查它的 required-features 与 cfg"
    fi
  done
  if [[ "$crash_point_replay_break" != product-gpu-not-run-ignored ]]; then
    while IFS=$'\t' read -r gpu_where gpu_line; do
      [[ -n "$gpu_line" ]] || continue
      gpu_not_run+=("$gpu_line")
      crash_point_replay_report_not_run "产品 GPU 包真显卡对拍没跑（${gpu_where}；这台机器找不到 GPU 适配器）：${gpu_line}；在有卡的机器上跑这一格补上"
    done < <(gpu_adapter_not_run_lines "$log")
  fi
  # 逐条认结局与记号（入参在判红那一段之前定好），判绿的照键记库
  if (( ran )) && [[ "$crash_point_replay_break" != product-gpu-cell-skipped ]] && account_test_results "$log" "$accounting"; then accounting_ok=1; fi
  if (( plan_ok && ran && accounting_ok )); then
    record_green_tests "$kind" "$plan_file" "$plan_output" "$accounting" "$cell" "cargo test --release -p $package --features $feature（真显卡对拍）" || plan_red=1
  fi
  crash_point_replay_unavailable_only "$cell" && return 1   # 记库那一步 prune-server 不可用：不可用那一句已经打了，不是失灵
  if (( plan_red )); then
    echo "  ✗ $cell 这一格判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；真显卡对拍那一趟照旧全部现判跑完了"
    echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再只跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $cell；不能复用就判红，不退回全部现判当绿"
    return 1
  fi
  not_run_total=$(( ${#zero_files[@]} + ${#gpu_not_run[@]} ))
  if (( not_run_total > 0 )); then
    product_gpu_cell_state="有 ${not_run_total} 处本次未跑"
    echo "  ! $cell 不算全绿：${#target_rows[@]} 个对拍测试目标（特性 $feature）跑到 ${ran_total} 条，本次未跑 ${not_run_total} 处：编进来 0 条的目标 ${#zero_files[@]} 个（${zero_files[*]:-无}），没卡没跑的对拍 ${#gpu_not_run[@]} 条"
    echo "     → 编进来 0 条的给它补不标 ignored 的对拍用例；没卡没跑的在有卡的机器上补跑：bash .claude/gate.d/crash-point-replay.sh --check $cell"
  else
    product_gpu_cell_state="绿"
    echo "  ✓ $cell：${#target_rows[@]} 个对拍测试目标（特性 $feature）每个都跑到了用例，共 ${ran_total} 条（含复用；$( (( plan_ok )) && printf '按条：%s' "$(plan_report "$plan_output.kept" 0)" || printf '按条复用算不出，全部现判')）"
  fi
  return 0
}


# collect_flow_pass <种类> <计划输出|-> <日志> <哪一趟>：一趟按条复用的日志按测试二进制转成记录追加进 crash_point_replay_flow_records
# （crash-flow-coverage.py collect：整份复用的取回判定库里存着的行，一半复用而库里没有的记 unknown）。返回 1 的两种：
#   它退 75（存取覆盖行时 prune-server 不可用）记进 crash_point_replay_flow_records_unavailable，覆盖、ignore、0 次三格照不可用办（文件头「退出码」75）；
#   别的非 0 是判法脚本出错，记进 crash_point_replay_flow_records_failed，那三格判红。弄坏开关 flow-records-unavailable-as-failure 下 75 也照出错办。
#   样本档标记目录里放了 collect-unreachable 的，这一步连一处连不上的服务（socket 不在、TCP 端口刚放掉、重连总时长 2 秒）造 75（文件头「样本档」）
collect_flow_pass() {
  local collect_exit unreachable_settings=()
  if [[ "$crash_point_replay_sample_mode" == 1 && -f "$PWD/.crash-point-replay-sample-tools/collect-unreachable" ]]; then
    # 改连接地址的变量只在临时服务底下认（scripts/prune-server/prune_server_client.py 文件头「连哪一处」）：临时服务的标记一并设上、指着同一个不在的 socket
    unreachable_settings=(env "SINGLEFS_PRUNE_SERVER_SOCKET=$crash_point_replay_scratch_directory/no-prune-server.sock"
      "PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET=$crash_point_replay_scratch_directory/no-prune-server.sock" SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS=2
      "SINGLEFS_PRUNE_SERVER_TCP_ADDRESS=127.0.0.1:$(python3 -c 'import socket; listener = socket.socket(); listener.bind(("127.0.0.1", 0)); print(listener.getsockname()[1]); listener.close()')")
  fi
  if ${unreachable_settings[@]+"${unreachable_settings[@]}"} python3 "$crash_point_replay_flow_judge" collect "$ROOT" "$1" "$2" "$3" --pass "$4" \
       >>"$crash_point_replay_flow_records" 2>"$crash_point_replay_flow_records.err"; then collect_exit=0; else collect_exit=$?; fi
  cat "$crash_point_replay_flow_records.err"
  if (( collect_exit == 75 )) && [[ "$crash_point_replay_break" != flow-records-unavailable-as-failure ]]; then
    crash_point_replay_flow_records_unavailable="crash-flow-coverage.py collect 退 75（$4）：$(tail -n 2 "$crash_point_replay_flow_records.err" | tr '\n' ' ' | cut -c1-300)"
    return 1
  fi
  if (( collect_exit != 0 )); then
    crash_point_replay_flow_records_failed="crash-flow-coverage.py collect 退 ${collect_exit}（$4）：$(tail -n 2 "$crash_point_replay_flow_records.err" | tr '\n' ' ' | cut -c1-300)"
    return 1
  fi
  return 0
}


# prepare_flow_records：按条复用两趟（crash-case-replay 格与 feature-gated-tests 格）的日志与这一趟判绿的每条崩溃枚举用例的日志转成记录（文件头「出口 4 那五格」），
# 记录文件放这一趟的日志目录（flow-records.tsv，跑完不删），打一行 CRASH_FLOW_RECORDS_FILE <路径>（崩溃点重放入口照这一行把它并进覆盖表）；
# 没判绿的用例记 unknown。点了 --item 时带 --subset（入口没有行、成员没被罩住两样本次未跑）。
prepare_flow_records() {
  local plain_plan="-" feature_plan="-" feature_log="$crash_point_replay_feature_pass_log" row key package target function log
  crash_point_replay_flow_records="$crash_point_replay_log_directory/flow-records.tsv"
  : >"$crash_point_replay_flow_records"
  (( ${#crash_point_replay_named_cases[@]} > 0 )) && crash_point_replay_flow_subset=(--subset)
  [[ -f "$crash_point_replay_flow_judge" ]] || return 0
  if ! crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME"; then
    crash_point_replay_flow_records_problem="这一趟没跑 $CRASH_CASE_REPLAY_CELL_NAME（--check 没点它），按条复用那一趟与用例的覆盖行与计数行没有；连它一起点：--check $CRASH_CASE_REPLAY_CELL_NAME,$FEATURE_GATED_TESTS_CELL_NAME,<这一格>"
    return 0
  fi
  if (( ! crash_point_replay_cell_green )); then
    crash_point_replay_flow_records_problem="$CRASH_CASE_REPLAY_CELL_NAME 判红，覆盖行与计数行不全"
    return 0
  fi
  (( crash_point_replay_plain_plan_ok )) && plain_plan="$crash_point_replay_scratch_directory/plain.plan.out"
  if crash_point_replay_skips_package_pass; then
    # 带了 --crash-cases-only（文件头「--crash-cases-only」）：按条复用那一趟没起，它打的行一行都没有，记行不全；不交 collect（没有计划的 collect 把日志里的每一段当整份）
    printf 'CRASH_FLOW_RECORD\tunknown\t%s\t这一趟带 --crash-cases-only，没起按条复用那一趟，它打的行没有\n' "$CRASH_CASE_REPLAY_CELL_NAME" >>"$crash_point_replay_flow_records"
  else
    collect_flow_pass "$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$CRASH_CASE_REPLAY_CELL_NAME")" "$plain_plan" \
      "$crash_point_replay_plain_pass_log" "不带特性那一趟" || return 0
  fi
  if ! crash_point_replay_runs_cell "$FEATURE_GATED_TESTS_CELL_NAME"; then
    # 没点带特性那一格：有挂特性的测试文件，它们打的行这一趟没有，记行不全；一份都没有就没缺什么
    if [[ -n "$(list_feature_gated_test_files crates/singlefs-verification/tests 2>/dev/null)" ]]; then
      printf 'CRASH_FLOW_RECORD\tunknown\tfeature-gated-tests\t这一趟没跑带特性那一趟（--check 没点 %s），挂特性的测试文件打的行没有\n' "$FEATURE_GATED_TESTS_CELL_NAME" >>"$crash_point_replay_flow_records"
    fi
  elif [[ "$feature_cell_outcome" == red ]]; then
    printf 'CRASH_FLOW_RECORD\tunknown\tfeature-gated-tests\t带特性那一趟判红，挂特性的测试文件打的行不全\n' >>"$crash_point_replay_flow_records"
  elif [[ -f "$feature_log" ]]; then
    (( crash_point_replay_feature_plan_ok )) && feature_plan="$crash_point_replay_scratch_directory/features.plan.out.kept"
    collect_flow_pass "$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$FEATURE_GATED_TESTS_CELL_NAME")" "$feature_plan" "$feature_log" "带特性那一趟" || return 0
  fi
  for row in "${crash_case_rows[@]}"; do
    IFS=$'\t' read -r key package target function <<< "$row"
    log="$crash_point_replay_log_directory/log.${key#crash-case:}"
    if crash_point_replay_cell_in "$key" ${green_cases[@]+"${green_cases[@]}"} && [[ -f "$log" ]]; then
      if ! python3 "$crash_point_replay_flow_judge" extract --source "$target" "$log" >>"$crash_point_replay_flow_records" 2>"$crash_point_replay_flow_records.err"; then
        crash_point_replay_flow_records_failed="crash-flow-coverage.py extract 读 $log 出错：$(tail -n 2 "$crash_point_replay_flow_records.err" | tr '\n' ' ' | cut -c1-300)"
        return 0
      fi
    else
      printf 'CRASH_FLOW_RECORD\tunknown\t%s\t%s 这一趟没判绿（%s），它打的行不算\n' "$target" "$key" "$function" >>"$crash_point_replay_flow_records"
    fi
  done
  [[ -s "$crash_point_replay_flow_records" ]] || return 0
  echo "  · 覆盖行与计数行：$(grep -c . "$crash_point_replay_flow_records") 条记录，记在 $crash_point_replay_flow_records（填两张基准的初值：python3 scripts/crash/crash-flow-coverage.py baseline <ignored|zero-evaluations> 这一份）"
  echo "CRASH_FLOW_RECORDS_FILE $crash_point_replay_flow_records"
  return 0
}


# run_crash_audit_cell <格名>：跑出口 4 的一格，判法的输出原样转打；结局并进 crash_audit_states，红的进 crash_audit_red，
# 本次未跑的往 $GATE_NOT_RUN_FILE 记一行。判法退 0 绿、1 红、77 本次未跑，别的退出码当判法出错、判红
run_crash_audit_cell() {
  local cell="$1" cell_exit reason output="$crash_point_replay_scratch_directory/crash-audit.$1.out"
  crash_audit_command=()
  crash_audit_not_run_reason=""
  if [[ "$cell" == crash-case-must-red || "$cell" == crash-knobs || "$cell" == "$CRASH_CASE_PROVENANCE_CELL_NAME" ]]; then echo "── $cell（只读树）"; else echo "── $cell"; fi
  case "$cell" in
    crash-flow-coverage|crash-ignored-nodes|invariant-zero-evaluations)
      if [[ ! -f "$crash_point_replay_flow_judge" ]]; then
        crash_audit_not_run_reason="判法脚本 $crash_point_replay_flow_judge 不在（这一份阶段被拷到了不带 scripts/ 的树里）"
      elif [[ -n "$crash_point_replay_flow_records_unavailable" ]]; then
        # 覆盖行存取连不上 prune-server：这一格照不可用办（进 crash_point_replay_unavailable_cells），整道收尾时只有不可用退 75、另有真红退 1（文件头「退出码」）
        echo "  ✗ $cell 这一格没判：prune-server 不可用（$crash_point_replay_flow_records_unavailable）；别的格照跑（不是红，也不算通过）"
        echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好再连 crash-case-replay 一起重跑：--check $CRASH_CASE_REPLAY_CELL_NAME,$FEATURE_GATED_TESTS_CELL_NAME,$cell"
        crash_point_replay_cell_in "$cell" ${crash_point_replay_unavailable_cells[@]+"${crash_point_replay_unavailable_cells[@]}"} \
          || crash_point_replay_unavailable_cells+=("$cell")
        crash_audit_states+=("$cell：prune-server 不可用"); crash_audit_red+=("$cell")
        return 0
      elif [[ -n "$crash_point_replay_flow_records_failed" ]]; then
        echo "  ✗ $cell：$crash_point_replay_flow_records_failed"
        echo "     → 怎么办：照那一句修 scripts/crash/crash-flow-coverage.py（单跑它的 --selftest 看是哪一格），再连 crash-case-replay 一起重跑：--check $CRASH_CASE_REPLAY_CELL_NAME,$cell"
        crash_audit_states+=("$cell：红"); crash_audit_red+=("$cell")
        return 0
      elif [[ -n "$crash_point_replay_flow_records_problem" ]]; then
        crash_audit_not_run_reason="$crash_point_replay_flow_records_problem"
      else
        local subcommand="coverage"
        [[ "$cell" == crash-ignored-nodes ]] && subcommand="ignored"
        [[ "$cell" == invariant-zero-evaluations ]] && subcommand="zero-evaluations"
        crash_audit_command=(python3 "$crash_point_replay_flow_judge" check "$subcommand" "$ROOT" ${crash_point_replay_flow_subset[@]+"${crash_point_replay_flow_subset[@]}"} "$crash_point_replay_flow_records")
      fi ;;
    crash-case-must-red|crash-knobs|crash-case-provenance)
      if [[ ! -f "$crash_point_replay_static_judge" ]]; then
        crash_audit_not_run_reason="判法脚本 $crash_point_replay_static_judge 不在（这一份阶段被拷到了不带 scripts/ 的树里）"
      else
        local item="must-red"
        [[ "$cell" == crash-knobs ]] && item="knobs"
        [[ "$cell" == crash-case-provenance ]] && item="provenance"
        crash_audit_command=(python3 "$crash_point_replay_static_judge" --only "$item" "$ROOT")
        # 弄坏开关 provenance-judge-not-run：来历表核对这一格不起判法，直接记绿
        [[ "$cell" == crash-case-provenance && "$crash_point_replay_break" == provenance-judge-not-run ]] && crash_audit_command=(true)
      fi ;;
  esac
  if (( ${#crash_audit_command[@]} > 0 )); then
    if "${crash_audit_command[@]}" >"$output" 2>&1; then cell_exit=0; else cell_exit=$?; fi
    cat "$output"
    case "$cell_exit" in
      0) crash_audit_states+=("$cell：绿"); return 0 ;;
      77) crash_audit_not_run_reason="$(grep -m 1 '本次未跑' "$output" | sed 's/^[[:space:]]*! 本次未跑（这不是通过）：//' | python3 -c 'import sys; print(sys.stdin.read().strip()[:120])')"
          [[ -n "$crash_audit_not_run_reason" ]] || crash_audit_not_run_reason="判法退 77（无对象可判）" ;;
      1) crash_audit_states+=("$cell：红"); crash_audit_red+=("$cell"); return 0 ;;
      *)
        echo "  ✗ $cell：判法 ${crash_audit_command[*]:0:3} 退 ${cell_exit}（不是 0、1、77：判法自己出错，输出在上面）"
        echo "     → 怎么办：照上面的报错修那份判法脚本，单跑同一条命令到它退 0、1 或 77，再只重跑这一格"
        crash_audit_states+=("$cell：红"); crash_audit_red+=("$cell"); return 0 ;;
    esac
  fi
  reason="${crash_audit_not_run_reason:-没说为什么}"
  [[ -s "$output" ]] || echo "  ! 本次未跑（这不是通过）：$cell：$reason"
  crash_audit_states+=("$cell：本次未跑（${reason}）")
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "crash-point-replay.sh 的 $cell 格本次未跑：$reason" >> "$GATE_NOT_RUN_FILE"; fi
  return 0
}

make_log_directory() {
  [[ -z "$crash_point_replay_log_directory" ]] || return 0
  crash_point_replay_log_directory="$git_common_directory/singlefs-crash-point-replay-logs/$(date -u +%Y%m%dT%H%M%SZ)-$$"
  if ! mkdir -p -- "$crash_point_replay_log_directory"; then
    echo "  ✗ 建不了这一趟的日志目录 $crash_point_replay_log_directory：每条用例的日志与发现日志没处放"
    echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑这一道。"
    exit 1
  fi
  echo "  · 这一趟每条用例的日志与发现日志放 ${crash_point_replay_log_directory}/（log.<用例名> 与 log.<用例名>.findings.tsv，跑完不删）"
}

# crash_point_replay_run_shard_step：分片的一片（文件头「分片」shard）：只做 crash-case-replay 格里起 cargo 的两样——归这一片的测试目标按条现判、每条崩溃枚举用例判这一片领的块，
#   判定都写进 prune-server；别的格与核账留给合并那一步。有红返回 1，prune-server 不可用返回 75，绿返回 0；服务交 shard 那一步与没有分片阶段时自己走的单片都调它
crash_point_replay_run_shard_step() {
  package_pass_report=""
  package_pass_plan_red=0
  shard_red_parts=()
  make_log_directory
  echo "  · 分片：第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片（$CRASH_CASE_REPLAY_CELL_NAME 格里起 cargo 的两样，别的格在合并那一步）"
  if run_package_test_functions; then
    echo "  ✓ 第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片的按条复用那一趟：${package_pass_report}"
  else
    shard_red_parts+=("按条复用那一趟")
  fi
  # 整份现判的测试二进制存下覆盖行（文件头「分片」shard），合并那一步照键取回；存不成只打一行
  if (( package_pass_ran )) && [[ -f "$crash_point_replay_flow_judge" ]]; then
    crash_point_replay_flow_records="$crash_point_replay_log_directory/flow-records.shard-${crash_point_replay_shard_index}.tsv"
    : >"$crash_point_replay_flow_records"
    collect_flow_pass "$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$CRASH_CASE_REPLAY_CELL_NAME")" \
      "$crash_point_replay_scratch_directory/plain.plan.out" "$crash_point_replay_plain_pass_log" "不带特性那一趟（第 ${crash_point_replay_shard_index} 片）" \
      || echo "  ! 第 ${crash_point_replay_shard_index} 片的覆盖行没存成（${crash_point_replay_flow_records_unavailable}${crash_point_replay_flow_records_failed}）：合并那一步的覆盖格照「行不全」办"
  fi
  if [[ -n "$crash_point_replay_stop_reason" ]]; then
    crash_point_replay_report_not_run "$CRASH_CASE_REPLAY_CELL_NAME 第 ${crash_point_replay_shard_index} 片：这一趟 ${crash_point_replay_stop_reason}，登记的崩溃枚举用例不接着跑"
  elif crash_point_replay_unavailable_only "$CRASH_CASE_REPLAY_CELL_NAME"; then
    crash_point_replay_report_not_run "$CRASH_CASE_REPLAY_CELL_NAME 第 ${crash_point_replay_shard_index} 片：prune-server 不可用，登记的崩溃枚举用例不接着跑"
  elif ! run_registered_crash_cases; then
    shard_red_parts+=("崩溃枚举用例")
  fi
  flush_deferred_pass_outcomes
  if crash_point_replay_unavailable_only "$CRASH_CASE_REPLAY_CELL_NAME" && (( ${#shard_red_parts[@]} <= 1 )) && [[ "${shard_red_parts[*]:-}" != 崩溃枚举用例 ]]; then
    echo "  ✗ 第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片：prune-server 不可用、没判完（不是红，也不算通过）"
    echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好再重交整件"
    return 75
  fi
  if (( ${#shard_red_parts[@]} > 0 )) || [[ -n "$crash_point_replay_stop_reason" ]]; then
    echo "  ✗ 第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片判红：${shard_red_parts[*]:-}${crash_point_replay_stop_reason:+（这一趟 ${crash_point_replay_stop_reason}）}（各自的出路在上面）"
    echo "     → 怎么办：照上面各自那一句改完，经 task-server/task submit 重交这一道；各片判过的照单元键复用"
    return 1
  fi
  echo "  ✓ 第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片判完：登记的崩溃枚举用例判绿 ${#green_cases[@]} 条，判定都写进 prune-server，九格的结论由合并那一步出"
  return 0
}

# crash_point_replay_run_feature_cell：带特性那一格（一趟只起一次）。弄坏开关 last-pass-before-the-other-cells 下排在产品 GPU 那一格与只读树的三格之前
crash_point_replay_run_feature_cell() {
  crash_point_replay_runs_cell "$FEATURE_GATED_TESTS_CELL_NAME" || return 0
  [[ -z "$feature_cell_outcome" ]] || return 0
  if crash_point_replay_stopped_before_cell "$FEATURE_GATED_TESTS_CELL_NAME"; then
    feature_cell_outcome="stopped"
  elif run_feature_gated_tests_cell "$package_pass_log"; then
    feature_cell_outcome="ran"
  else
    feature_cell_outcome="red"
    red_cells+=("$FEATURE_GATED_TESTS_CELL_NAME")
  fi
  flush_deferred_pass_outcomes
}

# crash_point_replay_run_audit_cells <格名…>：跑出口 4 那几格与来历表那一格里点了的；这一趟被停了的报本次未跑
crash_point_replay_run_audit_cells() {
  local audit_cell
  for audit_cell in "$@"; do
    crash_point_replay_runs_cell "$audit_cell" || continue
    if crash_point_replay_stopped_before_cell "$audit_cell"; then crash_audit_states+=("$audit_cell：本次未跑（这一趟被停了）"); continue; fi
    run_crash_audit_cell "$audit_cell"
    crash_point_replay_announce_cell "${crash_audit_states[-1]%%：*}" "${crash_audit_states[-1]#*：}"
  done
}
