#!/usr/bin/env bash
# 门禁 crash-point-replay 的函数库：整包那几趟的测试二进制预编与收尾、分片数、崩溃枚举用例的测试二进制、已记判定的预取、判定的记绿记红、没跑的补报与每一趟结局的报告。
# 由 .claude/gate.d/crash-point-replay.sh 在格名表与 preflight 之后、参数解析之前 source，不单独跑；格名表、参数解析与按格跑的主流程在那份入口里。
# 这份里是函数与它们的注释；函数读的格名、路径与状态变量由入口设，在调用时才取，别份库里的函数同样在调用时才找。

# crash_point_replay_prebuild_package_pass_binaries <目录>：计划那一步把按条复用那一趟的测试二进制编好一份放进 <目录>（文件头「分片」plan）：
#   不带 --crash-cases-only、这一趟的 test-plan 放成了（crash_point_replay_shared_plan_stored）、里面有要现判或拒算的测试函数才编：
#   scripts/gate/test-targets-parallel.py --prebuild-into <目录> -- CRASH_POINT_REPLAY_PACKAGE_PASS_CARGO_ARGUMENTS，环境同 run_checker_package_tests（清掉发现日志变量、带多机配置），
#   输出进 prebuild.package-pass.log；编不过、放不成只打一行（各片照旧经 cargo 编），不判红。<目录> 是空串（没有计划那一步放的目录）的不编。
#   弄坏开关 package-pass-shards-build-themselves 下不编
crash_point_replay_prebuild_package_pass_binaries() {
  local directory="$1" parallel="$crash_point_replay_stage_repository/scripts/gate/test-targets-parallel.py" build_log build_exit prebuilt_line
  [[ -n "$directory" ]] || return 0
  crash_point_replay_skips_package_pass && return 0
  if [[ "$crash_point_replay_break" == package-pass-shards-build-themselves ]]; then
    echo "  · 计划：弄坏开关 package-pass-shards-build-themselves：按条复用那一趟的测试二进制不预先编，各片照旧经 cargo 编"
    return 0
  fi
  if (( ! crash_point_replay_shared_plan_stored )); then
    echo "  · 计划：按条复用那一趟的 test-plan 没放成（上面那一句），各片自己算计划、照旧经 cargo 编，它的测试二进制不预先编"
    return 0
  fi
  if (( plan_judge + plan_refused == 0 )); then
    echo "  · 计划：按条复用那一趟没有要现判或拒算的测试函数（复用 ${plan_reuse}），各片不起它，它的测试二进制不预先编"
    return 0
  fi
  if [[ ! -f "$parallel" ]]; then
    echo "  ! 计划：没有 $parallel：按条复用那一趟的测试二进制不预先编，各片照旧经 cargo 编"
    return 0
  fi
  build_log="$crash_point_replay_pass_log_directory/prebuild.package-pass.log"
  echo "  · 计划：预先编按条复用那一趟的测试二进制（不持卡，只编不跑）：python3 scripts/gate/test-targets-parallel.py --prebuild-into $directory -- ${CRASH_POINT_REPLAY_PACKAGE_PASS_CARGO_ARGUMENTS[*]}"
  env -u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
    python3 "$parallel" --prebuild-into "$directory" -- "${CRASH_POINT_REPLAY_PACKAGE_PASS_CARGO_ARGUMENTS[@]}" 2>&1 | crash_point_replay_pass_output "$build_log"
  build_exit="${PIPESTATUS[0]}"
  prebuilt_line="$(grep -m 1 '^TEST_TARGETS_PARALLEL_PREBUILT ' "$build_log" 2>/dev/null)"
  if (( build_exit != 0 )) || [[ -z "$prebuilt_line" || ! -f "$directory/targets.tsv" ]]; then
    tail -n 20 "$build_log"
    echo "  ! 计划：按条复用那一趟只编那一步退 $build_exit（上面是它的尾部，整份 $build_log）：不放，各片照旧经 cargo 编（编不过的由各片判红）"
    return 0
  fi
  echo "  ✓ 计划：按条复用那一趟的测试二进制编了一回（test-targets-parallel.py --prebuild-into，不持卡；${prebuilt_line#TEST_TARGETS_PARALLEL_PREBUILT }）：各片读了计划那一步的 test-plan 就照清单直接起、不再编"
}

# crash_point_replay_settle_package_pass_binaries：分片的一片起按条复用那一趟之前（文件头「分片」shard）：计划那一步放了 test-binaries/package-pass/targets.tsv、这一片读的是计划那一步的
#   test-plan（crash_point_replay_shared_plan_read：源码在计划之后没改）时，核清单里放进去的二进制的判器键所读的文件（crash_amplification_judging_inputs_of_binaries 一次核完：
#   变了的那个改写成 -、经 cargo 起；核不出的照用），写成这一片自己的清单、路径放进 crash_point_replay_package_binaries，打一行；别的情形 crash_point_replay_package_binaries 清空、
#   打一行原因（没有计划那一步放的目录的不打）。别的片领的测试目标（crash_point_replay_other_shard_roots_file 里的目标根文件）这一片不起，不核它们、清单里照留
#   （test-targets-parallel.py --drop-sources 在起的时候去掉，run_checker_package_tests）。一律交 0。
#   弄坏开关 package-pass-shards-build-themselves 下不用；package-pass-other-shards-targets-run 下别的片领的目标也核、也起
crash_point_replay_settle_package_pass_binaries() {
  local manifest="$crash_point_replay_shared_binaries_directory/package-pass/targets.tsv" settled="$crash_point_replay_scratch_directory/package-pass-binaries.tsv"
  local answers="$crash_point_replay_scratch_directory/package-pass-binaries.answers" jobs batch_exit=0 index row_source row_root answer_at
  local step="第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片" header package selector binary rest key_state key_exit answer
  local same=0 changed=0 unknown=0 absent=0 other_shards=0 other_note=""
  local -a row_package=() row_selector=() row_binary=() row_rest=() check_binaries=() answer_lines=()
  local -A other_shard_root=() answer_of_row=()
  crash_point_replay_package_binaries=""
  [[ -n "$crash_point_replay_shared_binaries_directory" && "$crash_point_replay_break" != package-pass-shards-build-themselves ]] || return 0
  if [[ ! -r "$manifest" ]]; then
    echo "  · ${step}：计划那一步没留下按条复用那一趟的测试二进制清单（$manifest 不在或读不了），照旧经 cargo 编"
    return 0
  fi
  if (( ! crash_point_replay_shared_plan_read )); then
    echo "  ! ${step}：这一片没读计划那一步的 test-plan（上面那一句：源码在计划之后改了，或读不了），计划那一步编好的按条复用那一趟的测试二进制不用，照旧经 cargo 编"
    return 0
  fi
  if [[ -s "$crash_point_replay_other_shard_roots_file" && "$crash_point_replay_break" != package-pass-other-shards-targets-run ]]; then
    while IFS= read -r row_root; do [[ -z "$row_root" ]] || other_shard_root["$row_root"]=1; done <"$crash_point_replay_other_shard_roots_file"
  fi
  {
    IFS= read -r header
    while IFS=$'\t' read -r package selector binary rest; do
      row_package+=("$package"); row_selector+=("$selector"); row_binary+=("$binary"); row_rest+=("$rest")
    done
  } <"$manifest"
  for index in "${!row_package[@]}"; do
    binary="${row_binary[index]}"
    row_source="${row_rest[index]%%$'\t'*}"
    row_root="${row_source#unittests }"
    if [[ -n "$row_root" && -n "${other_shard_root[$row_root]+x}" ]]; then
      other_shards=$((other_shards + 1))
    elif [[ "$binary" == - ]]; then
      absent=$((absent + 1))
    else
      answer_of_row[$index]="${#check_binaries[@]}"
      check_binaries+=("$binary")
    fi
  done
  if (( ${#check_binaries[@]} > 0 )); then
    jobs="$(crash_point_replay_heavy_jobs)"
    [[ "$jobs" =~ ^[1-9][0-9]*$ ]] || jobs=1
    if (source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_judging_inputs_of_binaries "$ROOT" "$jobs" "${check_binaries[@]}") >"$answers"; then batch_exit=0; else batch_exit=$?; fi
    mapfile -t answer_lines <"$answers"
    if (( batch_exit != 0 || ${#answer_lines[@]} != ${#check_binaries[@]} )); then
      # 一批核不成、或收回的行数对不上：一个答案都不信，每个二进制都当键所读的文件变了、经 cargo 起（宁慢，不把没核过的当没变）
      answer_lines=()
      for binary in "${check_binaries[@]}"; do answer_lines+=("1"$'\t'"判器输入核对没跑成（退 ${batch_exit}，收回的行数与二进制个数 ${#check_binaries[@]} 对不上）"); done
    fi
  fi
  {
    printf '%s\n' "$header"
    for index in "${!row_package[@]}"; do
      binary="${row_binary[index]}"
      if [[ -n "${answer_of_row[$index]+x}" ]]; then
        answer_at="${answer_of_row[$index]}"
        answer="${answer_lines[answer_at]}"
        key_exit="${answer%%$'\t'*}"; key_state="${answer#*$'\t'}"
        case "$key_exit" in
          0) same=$((same + 1)) ;;
          1) changed=$((changed + 1)); echo "  ! ${step}：${row_package[index]} ${row_selector[index]} 的测试二进制编完之后判器键所读的文件变了（$key_state）：这个目标经 cargo 起" >&2; binary=- ;;
          *) unknown=$((unknown + 1)) ;;
        esac
      fi
      printf '%s\t%s\t%s\t%s\n' "${row_package[index]}" "${row_selector[index]}" "$binary" "${row_rest[index]}"
    done
  } >"$settled"
  crash_point_replay_package_binaries="$settled"
  (( other_shards == 0 )) || other_note="；别的片领的 $other_shards 个测试目标这一片不起、不核"
  echo "  · ${step}的按条复用那一趟直接起计划那一步编好的测试二进制（$manifest）：判器键所读的文件没变 $same 个、核不出照用 $unknown 个（源码在计划之后没改）、变了改经 cargo $changed 个、计划那一步没放的 $absent 个经 cargo${other_note}"
  return 0
}

# crash_point_replay_recorded_case_blocks <用例名，不带 crash-case:>：上一趟整份跑这条用例记下的块数：日志根 <git common-dir>/singlefs-crash-point-replay-logs/ 下按目录名
#   （开跑时刻起头）从新到旧找，只认有 whole-case-logs 记号的目录（核账那一趟、没有分片阶段时的合并留的：用例日志是整份 0/1 跑的，不是一片领的那一份），
#   取 log.<用例名> 里第一行 CRASH_CASE_BLOCKS 的 blocks=；缩过域的那一行（带 expand_up_to=）不认。找到打块数、交 0；没有记录交 1
crash_point_replay_recorded_case_blocks() {
  local logs_root="$git_common_directory/singlefs-crash-point-replay-logs" log_name blocks_line
  [[ -d "$logs_root" ]] || return 1
  while IFS= read -r log_name; do
    [[ -f "$logs_root/$log_name/whole-case-logs" && -f "$logs_root/$log_name/log.$1" ]] || continue
    blocks_line="$(grep -m 1 '^CRASH_CASE_BLOCKS ' "$logs_root/$log_name/log.$1" 2>/dev/null)" || continue
    [[ "$blocks_line" != *expand_up_to=* && "$blocks_line" =~ [[:space:]]blocks=([0-9]+)([[:space:]]|$) ]] || continue
    printf '%s\n' "${BASH_REMATCH[1]}"
    return 0
  done < <(ls -1 -- "$logs_root" 2>/dev/null | LC_ALL=C sort -r)
  return 1
}

# crash_point_replay_decide_shard_count：计划那一步的片数（文件头「分片」plan），改写 crash_point_replay_shard_plan_count 与 crash_point_replay_shard_plan_origin（一句写明取的是哪一个）。
#   上界是上面定的卡数与 shards-at-least 的大者；不点名用例的整轮取上界；点名了的照这一趟要现判的量：点名的每条用例上一趟整份记下的块数之和（crash_point_replay_recorded_case_blocks）、
#   与不带 --crash-cases-only 时计划那一步算的 test-plan 里要现判（judge、refused）的测试目标数，各除以一片担的量（CRASH_POINT_REPLAY_BLOCKS_PER_SHARD、CRASH_POINT_REPLAY_TARGETS_PER_SHARD）向上取整，
#   取大的、至少 1，不超过上界；有一条用例的块数没有记录、或 test-plan 计划那一步没算成，取上界。弄坏开关 shard-count-from-cards-only 下一律取上界
crash_point_replay_decide_shard_count() {
  local upper="$crash_point_replay_shard_plan_count" upper_origin="$crash_point_replay_shard_plan_origin" crash_case_row case_label case_blocks
  local total_blocks=0 targets=0 unknown="" need need_blocks need_targets amount
  if [[ "$crash_point_replay_break" == shard-count-from-cards-only ]]; then
    crash_point_replay_shard_plan_origin="取上界 ${upper} 片：${upper_origin}；弄坏开关 shard-count-from-cards-only 下不看这一趟要现判的量"
    return 0
  fi
  if (( ${#crash_point_replay_named_cases[@]} == 0 )); then
    crash_point_replay_shard_plan_origin="取上界 ${upper} 片：${upper_origin}；不点名用例的整轮（全部登记的 ${#crash_case_rows[@]} 条崩溃枚举用例${crash_point_replay_separate_run_note}$(crash_point_replay_skips_package_pass || printf '%s' '与包里按条复用那一趟')）照上界分"
    return 0
  fi
  for crash_case_row in "${crash_case_rows[@]}"; do
    case_label="${crash_case_row%%$'\t'*}"; case_label="${case_label#crash-case:}"
    if case_blocks="$(crash_point_replay_recorded_case_blocks "$case_label")"; then
      total_blocks=$((total_blocks + case_blocks))
    else
      unknown+="${unknown:+、}crash-case:$case_label 上一趟整份跑记下的块数没有记录"
    fi
  done
  amount="点名的 ${#crash_case_rows[@]} 条用例上一趟整份记下的块共 ${total_blocks} 个（一片担 ${CRASH_POINT_REPLAY_BLOCKS_PER_SHARD} 个）"
  if ! crash_point_replay_skips_package_pass; then
    if [[ -r "$crash_point_replay_shared_plan_directory/plain.plan.out" ]]; then
      targets="$(awk -F'\t' 'NF >= 7 && ($2 == "judge" || $2 == "refused") { print $5 }' "$crash_point_replay_shared_plan_directory/plain.plan.out" | LC_ALL=C sort -u | awk 'END { print NR }')"
      amount+="、包里按条复用那一趟要现判的测试目标 ${targets} 个（一片担 ${CRASH_POINT_REPLAY_TARGETS_PER_SHARD} 个）"
    else
      unknown+="${unknown:+、}不带特性那一趟的 test-plan 计划那一步没算成，要现判的测试目标数不知道"
    fi
  fi
  if [[ -n "$unknown" ]]; then
    crash_point_replay_shard_plan_origin="取上界 ${upper} 片：${upper_origin}；这一趟要现判的量不知道（${unknown}）"
    return 0
  fi
  need_blocks=$(( (total_blocks + CRASH_POINT_REPLAY_BLOCKS_PER_SHARD - 1) / CRASH_POINT_REPLAY_BLOCKS_PER_SHARD ))
  need_targets=$(( (targets + CRASH_POINT_REPLAY_TARGETS_PER_SHARD - 1) / CRASH_POINT_REPLAY_TARGETS_PER_SHARD ))
  need=$(( need_blocks > need_targets ? need_blocks : need_targets ))
  (( need >= 1 )) || need=1
  if (( need >= upper )); then
    crash_point_replay_shard_plan_origin="取上界 ${upper} 片：${upper_origin}；这一趟要现判的量（${amount}）要 ${need} 片，不少于上界"
  else
    crash_point_replay_shard_plan_count="$need"
    crash_point_replay_shard_plan_origin="取这一趟要现判的量算出的 ${need} 片：${amount}；上界 ${upper} 片（${upper_origin}）"
  fi
}

# crash_point_replay_case_binary_identity：case_command（crash-case-command 交的那一条）编出的测试二进制认哪一个：打「<包><TAB><特性，没带写 -><TAB><测试目标>」，
#   计划那一步照它记进 binaries.tsv，各片与合并照它找（crash_point_replay_use_shared_case_binary）；特性是 -- 之前 --features 后面那一个词（登记的，与开了 GPU 加的 gpu）
crash_point_replay_case_binary_identity() {
  local index package="" features="-" target=""
  for (( index = 0; index + 1 < ${#case_command[@]}; index++ )); do
    [[ "${case_command[index]}" != -- ]] || break
    case "${case_command[index]}" in
      -p) package="${case_command[index + 1]}" ;;
      --features) features="${case_command[index + 1]}" ;;
      --test) target="${case_command[index + 1]}" ;;
    esac
  done
  printf '%s\t%s\t%s\n' "$package" "$features" "$target"
}

# crash_point_replay_prebuild_case_binaries <目录>：计划那一步把这一趟登记的崩溃枚举用例（crash_case_rows：点名的，没点是全部）要用的测试二进制编好一份，放进 <目录>：
#   每条用例照 crash-case-command 交的命令（特性照登记与多机配置，开了 GPU 的带 gpu）取 -- 之前那一截、去掉 --test <目标> 当分组，同一组的目标并成一条
#   「<那一截> --test <目标>… --no-run」（不持卡）；照它打的 Executable 行取每个目标的测试二进制，硬链接进 <目录>/deps/（不在同一个文件系统就拷一份），
#   连同它所在目录上一层 build/singlefs-verification-*/out 里判器核对要读的两张表（judging_units_keys.tsv、watched_file_digests.tsv）拷进 <目录>/build/ 下同名的目录，
#   <目录>/binaries.tsv 一行「<包><TAB><特性><TAB><测试目标><TAB><放进去的路径>」。编不过、没打 Executable 行、放不进的那几个不放、打一行，各片自己经 cargo test 起；不判红。
#   先放在 <目录>.partial，放完换上。弄坏开关 shards-build-themselves 下不编、不放
crash_point_replay_prebuild_case_binaries() {
  local directory="$1" staging="$1.partial" crash_case_row case_key case_package case_target case_function word index separator_at skip_next
  local group_key group_number build_log build_exit executable binary_name table_directory table_name build_name target
  local placed=0 linked=0 copied=0 missing=0 built=0 stripped=0 table_copies=0 place_exit table_stamp
  local -a prefix=() build_command=() group_keys=() group_target_list=()
  local -A group_of=() group_targets=() identity_of=() copied_tables=()
  [[ -n "$directory" ]] || return 0
  if [[ "$crash_point_replay_break" == shards-build-themselves ]]; then
    echo "  · 计划：弄坏开关 shards-build-themselves：登记的崩溃枚举用例的测试二进制不预先编，各片与合并各自经 cargo test 起"
    return 0
  fi
  rm -rf -- "$staging"
  if ! mkdir -p -- "$staging/deps" "$staging/build"; then
    echo "  ! 计划：建不了 $staging：登记的崩溃枚举用例的测试二进制不预先编，各片与合并各自经 cargo test 起"
    return 0
  fi
  for crash_case_row in "${crash_case_rows[@]}"; do
    IFS=$'\t' read -r case_key case_package case_target case_function <<< "$crash_case_row"
    if ! read_crash_case_command "$case_key"; then
      echo "  ! 计划：$case_key：准入模块交不出起它的命令（$case_command_problem），不预先编：各片自己经 cargo test 起（交不出的由各片判红）"
      continue
    fi
    separator_at=-1
    for index in "${!case_command[@]}"; do
      if [[ "${case_command[index]}" == -- ]]; then separator_at="$index"; break; fi
    done
    if (( separator_at < 1 )); then
      echo "  ! 计划：$case_key：准入模块交的命令认不出形状（没有 --），不预先编：各片自己经 cargo test 起"
      continue
    fi
    prefix=(); skip_next=0
    for word in "${case_command[@]:0:separator_at}"; do
      if (( skip_next )); then skip_next=0; continue; fi
      if [[ "$word" == --test ]]; then skip_next=1; continue; fi
      prefix+=("$word")
    done
    group_key="$(printf '%q ' "${prefix[@]}")"
    if [[ -z "${group_of[$group_key]:-}" ]]; then
      group_keys+=("$group_key")
      group_of[$group_key]="${#group_keys[@]}"
      printf '%s\0' "${prefix[@]}" >"$staging/group.${#group_keys[@]}"
    fi
    group_number="${group_of[$group_key]}"
    [[ " ${group_targets[$group_number]:-} " == *" $case_target "* ]] || group_targets[$group_number]+=" $case_target"
    identity_of["$group_number/$case_target"]="$(crash_point_replay_case_binary_identity)"
  done
  : >"$staging/binaries.tsv"
  for (( group_number = 1; group_number <= ${#group_keys[@]}; group_number++ )); do
    mapfile -d '' -t prefix <"$staging/group.$group_number"
    read -r -a group_target_list <<< "${group_targets[$group_number]}"
    build_command=("${prefix[@]}")
    for target in "${group_target_list[@]}"; do build_command+=(--test "$target"); done
    build_command+=(--no-run)
    build_log="$crash_point_replay_pass_log_directory/prebuild.$group_number.log"
    echo "  · 计划：预先编登记的崩溃枚举用例的测试二进制（不持卡，只编不跑）：${build_command[*]}"
    "${build_command[@]}" 2>&1 | crash_point_replay_pass_output "$build_log"
    build_exit="${PIPESTATUS[0]}"
    built=$((built + 1))
    if (( build_exit != 0 )); then
      tail -n 20 "$build_log"
      echo "  ! 计划：这一组只编那一步退 $build_exit（上面是它的尾部，整份 $build_log）：这几个测试目标（${group_target_list[*]}）不放，各片自己经 cargo test 起（编不过的由各片判红）"
      missing=$((missing + ${#group_target_list[@]}))
      continue
    fi
    for target in "${group_target_list[@]}"; do
      if ! executable="$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_test_binary_in_log "$build_log" "$target")"; then
        echo "  ! 计划：$target：只编那一步的日志 $build_log 里没有它的 Executable 行（或文件不在），不放：各片自己经 cargo test 起"
        missing=$((missing + 1)); continue
      fi
      binary_name="$(basename -- "$executable")"
      if ln -f -- "$executable" "$staging/deps/$binary_name" 2>/dev/null; then
        linked=$((linked + 1))
      else
        # 链不成（不在同一个文件系统）就拷，拷的时候去掉调试段（crash_amplification_place_binary_without_debug_info：交 0 去了、3 整份拷、1 拷不成）
        place_exit=0
        (source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_place_binary_without_debug_info "$executable" "$staging/deps/$binary_name") || place_exit=$?
        case "$place_exit" in
          0) copied=$((copied + 1)); stripped=$((stripped + 1)) ;;
          3) copied=$((copied + 1)) ;;
          *) echo "  ! 计划：$target：$executable 放不进 $staging/deps/，不放：各片自己经 cargo test 起"
             missing=$((missing + 1)); continue ;;
        esac
      fi
      # 判器核对（crash_amplification_judging_inputs_unchanged）照二进制所在目录上一层的 build/singlefs-verification-*/out 找键表与摘要表，放进去的照同样的相对位置放
      for table_directory in "$(dirname -- "$executable")"/../build/singlefs-verification-*/out; do
        [[ -d "$table_directory" ]] || continue
        build_name="$(basename -- "$(dirname -- "$table_directory")")"
        mkdir -p -- "$staging/build/$build_name/out" || continue
        for table_name in judging_units_keys.tsv watched_file_digests.tsv; do
          [[ -f "$table_directory/$table_name" ]] || continue
          # 同一份表（路径、修改时刻、大小）只拷一份，不为每个二进制各拷一遍；弄坏开关 case-tables-copied-per-binary 下各拷各的
          table_stamp="$table_directory/$table_name $(stat -c '%y %s' -- "$table_directory/$table_name")"
          [[ -z "${copied_tables[$table_stamp]+x}" || "$crash_point_replay_break" == case-tables-copied-per-binary ]] || continue
          cp -p -- "$table_directory/$table_name" "$staging/build/$build_name/out/$table_name"
          copied_tables["$table_stamp"]=1
          table_copies=$((table_copies + 1))
        done
      done
      printf '%s\t%s\n' "${identity_of[$group_number/$target]}" "$directory/deps/$binary_name" >>"$staging/binaries.tsv"
      placed=$((placed + 1))
    done
  done
  rm -f -- "$staging"/group.*
  if ! { rm -rf -- "$directory" && mv -- "$staging" "$directory"; }; then
    echo "  ! 计划：预先编好的测试二进制放不进 $directory：各片与合并各自经 cargo test 起"
    rm -rf -- "$staging"
    return 0
  fi
  echo "  ✓ 计划：登记的崩溃枚举用例要用的测试二进制按命令分 ${built} 组各编一回（cargo test … --no-run，不持卡），${placed} 个放进 ${directory}（硬链接 ${linked} 份、拷 ${copied} 份）；没放的 ${missing} 个由各片自己经 cargo test 起，放进去的各片与合并直接起、不再编（拷的里去掉调试段 ${stripped} 份；判器核对的表拷 ${table_copies} 份）"
}


# seed_prior_verdicts <种类> [--feature …]：样本档（文件头「样本档」）：拿 .unit-verdict-sample/prior/ 那一版算每条测试函数的键、全部照判绿记进这一版样本的判定库。
# 样本另放了 .unit-verdict-sample/prior-multi-host.env（上一版那一趟的多机配置）的，这一趟照它起（SINGLEFS_MULTI_HOST_CONFIG），GPU 开关取它的 ENABLE_GPU
# （只在弄坏开关 gpu-switch-in-key 下进键）
seed_prior_verdicts() {
  local kind="$1" prior="$PWD/.unit-verdict-sample/prior" seed="$crash_point_replay_scratch_directory/seed" seed_exit
  local prior_configuration="$PWD/.unit-verdict-sample/prior-multi-host.env" prior_switch="$crash_point_replay_gpu_configured" prior_note=""
  shift
  local -a break_setting=() prior_setting=() registered_exclusion=()
  [[ "$crash_point_replay_break" == setting-file-whole-closure ]] && break_setting=("ADMISSION_BREAK=test-key-with-whole-file")
  [[ -z "${plan_package:-}" && "$crash_point_replay_break" != registered-rows-in-plan ]] && registered_exclusion=(--exclude-registered)
  if [[ -f "$prior_configuration" ]]; then
    prior_setting=("SINGLEFS_MULTI_HOST_CONFIG=$prior_configuration")
    prior_switch=0
    if [[ "$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_configuration_value "$prior_configuration" ENABLE_GPU)" == 1 ]]; then prior_switch=1; fi
    prior_note="（上一版那一趟的多机配置 .unit-verdict-sample/prior-multi-host.env，ENABLE_GPU=${prior_switch}）"
  fi
  crash_point_replay_plan_tokens "$crash_point_replay_stage_repository" "$prior_switch"
  if env ${prior_setting[@]+"${prior_setting[@]}"} ${break_setting[@]+"${break_setting[@]}"} python3 "$crash_point_replay_admission_module" test-plan "$prior" "$kind" "${plan_package:-singlefs-verification}" "$seed.plan" \
       "${crash_point_replay_plan_token_arguments[@]}" "$@" ${registered_exclusion[@]+"${registered_exclusion[@]}"} >"$seed.out" 2>&1; then seed_exit=0; else seed_exit=$?; fi
  if ((seed_exit != 0)); then
    plan_problem="样本档：上一版的树上算不出键：$(tail -n 3 "$seed.out" | tr '\n' ' ' | cut -c1-400)"
    plan_failure=(test-plan "$seed_exit" "$seed.out")
    return 1
  fi
  awk -F'\t' '$2 == "judge" || $2 == "reuse" { print $1 }' "$seed.out" >"$seed.names"
  if python3 "$crash_point_replay_admission_module" verdict-record "$prior" "$kind" "$seed.plan" "$seed.names" --how "样本档：上一版" >"$seed.record" 2>&1; then
    seed_exit=0; else seed_exit=$?; fi
  if ((seed_exit != 0)); then
    plan_problem="样本档：上一版那一趟的判定没记进库：$(tail -n 2 "$seed.record" | tr '\n' ' ')"; plan_failure=(verdict-record "$seed_exit" "$seed.record"); return 1; fi
  echo "  · 样本档：上一版（.unit-verdict-sample/prior）的 $(wc -l <"$seed.names") 条测试函数照判绿记进这一版的判定库（$(tail -n 1 "$seed.record")）${prior_note}"
}


# skip_arguments_of_plan <计划输出|-> [<不起的测试目标清单> <放结果的数组名>]：登记的崩溃枚举用例与放量分项（crash_point_replay_registered_pairs_file 的用例函数，只在 验证包那两趟）逐条 --skip，
# 再是键相同（reuse）、这一组特性下不编（not-compiled）的测试函数逐条 --skip（加 --exact）；要现判的同名就不跳（几个二进制里同名的测试都跑，宁多跑）。
# 计划里的名单只在 scripts/gate/admission.py 的 plan-skip-names 一处（它文件头「按条复用的共用段」，scripts/crash/crash-point-replay.sh 按项跑的那一段调同一条）；
# 给 - 是计划算不出，只排登记的。结果放进全局数组 plan_skip_arguments（cargo test 的 -- 之后接；给了数组名的放进那个数组）。取不出计划里的名单就只排登记的（其余全部现判，宁多跑）、打一行。
# 给了不起的测试目标清单（目标根文件，一行一个）：这些目标不起，名单里只收其余目标的行带来的名字（plan-skip-names --except-target-roots-file；名字被现判的同名行排掉照旧看全部目标）。
# 弄坏开关 registered-cases-not-skipped 下不排登记的
skip_arguments_of_plan() {
  local name names="" registered_names=""
  local -n skip_result="${3:-plan_skip_arguments}"
  local -a except_option=()
  skip_result=()
  [[ -z "${2:-}" ]] || except_option=(--except-target-roots-file "$2")
  if [[ "$1" != - ]] && ! names="$(python3 "$crash_point_replay_admission_module" plan-skip-names "$1" ${except_option[@]+"${except_option[@]}"})"; then
    echo "  ! 按条复用：admission.py plan-skip-names 没成（$(tr '\n' ' ' <<<"$names" | clip_text 300)），这一趟计划里的一条都不跳、全部现判"
    names=""
  fi
  if [[ -z "${plan_package:-}" && "$crash_point_replay_break" != registered-cases-not-skipped && -s "$crash_point_replay_registered_pairs_file" ]]; then
    registered_names="$(cut -f2 "$crash_point_replay_registered_pairs_file" | LC_ALL=C sort -u)"
  fi
  while IFS= read -r name; do
    [[ -n "$name" ]] && skip_result+=(--skip "$name")
  done <<<"$registered_names"$'\n'"$names"
  ((${#skip_result[@]})) && skip_result=(--exact "${skip_result[@]}")
  return 0
}


# plan_failure_verdict <格名> <子命令> <退出码> <输出文件>…：按条复用失灵没有（判法只在 scripts/gate/admission.py 的 plan-failure，文件头「按条复用失灵」）；
# 失灵它打 ✗ 与出路、这里返回 1，那一格照旧跑完、结论判红。它退 75 是 prune-server 不可用（不是失灵）：这一格记进 crash_point_replay_unavailable_cells、
# 这里返回 75，调用方让这一格就此收尾（计划那一步退 75 的不起 cargo），别的格照跑；整道收尾时有别的红退 1、两样都报，只有不可用退 75（文件头「退出码」）。
# 弄坏开关 plan-unavailable-as-failure 下照失灵办（返回 1）；plan-unavailable-exits-at-once 下当场整道退 75（前面已判红的格被吞）
plan_failure_verdict() {
  local cell="$1" verdict_exit
  shift
  if python3 "$crash_point_replay_admission_module" plan-failure "$crash_point_replay_stage_file_name 的 $cell 格" "$@"; then return 0; else verdict_exit=$?; fi
  if (( verdict_exit == 75 )) && [[ "$crash_point_replay_break" != plan-unavailable-as-failure ]]; then
    if [[ "$crash_point_replay_break" == plan-unavailable-exits-at-once ]]; then
      echo "  ✗ $cell 这一格没判：prune-server 不可用（上面那一句）；这一道不往下跑，退 75（不是红，也不算通过）"
      echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好再重跑这一道"
      exit 75
    fi
    echo "  ✗ $cell 这一格没判：prune-server 不可用（上面那一句）；这一格就此收尾，别的格照跑（不是红，也不算通过）"
    echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $cell [项目根]"
    crash_point_replay_cell_in "$cell" ${crash_point_replay_unavailable_cells[@]+"${crash_point_replay_unavailable_cells[@]}"} \
      || crash_point_replay_unavailable_cells+=("$cell")
    return 75
  fi
  return 1
}


# crash_point_replay_unavailable_only <格名>：这一格判不了只因为 prune-server 不可用（在 crash_point_replay_unavailable_cells 里、不在 crash_point_replay_red_regardless_cells 里）退 0
crash_point_replay_unavailable_only() {
  crash_point_replay_cell_in "$1" ${crash_point_replay_unavailable_cells[@]+"${crash_point_replay_unavailable_cells[@]}"} \
    && ! crash_point_replay_cell_in "$1" ${crash_point_replay_red_regardless_cells[@]+"${crash_point_replay_red_regardless_cells[@]}"}
}


# red_cell_state <格名>：逐格汇总里没过的那一格写什么：只因 prune-server 不可用的写「prune-server 不可用」，别的写「红」
red_cell_state() {
  if crash_point_replay_unavailable_only "$1"; then printf 'prune-server 不可用'; else printf '红'; fi
}


# rerun_check_list <格名>…：只重跑这几格的 --check 列表（逗号隔开）。覆盖、ignore、0 次三格读按条复用那两趟与每条用例的行：点了其中一格而没点 crash-case-replay 的，
#   连 crash-case-replay 与 feature-gated-tests 一起点
rerun_check_list() {
  local rerun_cells=("$@") flow_cell
  for flow_cell in "${CRASH_AUDIT_FLOW_CELLS[@]}"; do
    if crash_point_replay_cell_in "$flow_cell" "$@" && ! crash_point_replay_cell_in "$CRASH_CASE_REPLAY_CELL_NAME" "${rerun_cells[@]}"; then
      crash_point_replay_cell_in "$FEATURE_GATED_TESTS_CELL_NAME" "${rerun_cells[@]}" || rerun_cells=("$FEATURE_GATED_TESTS_CELL_NAME" "${rerun_cells[@]}")
      rerun_cells=("$CRASH_CASE_REPLAY_CELL_NAME" "${rerun_cells[@]}")
    fi
  done
  (IFS=,; printf '%s' "${rerun_cells[*]}")
}


# account_test_results <日志> <清单文件>：逐条认结局与记号归谁（判法只在 scripts/gate/lib_libtest_result_lines.py 的
# recordable_results，文件头「按条记库」；这里只调它的 recordable 子命令），一行一条落进 <清单文件>（green / withheld / marker，写法在它文件头「命令行」）。
# 没成返回 1、打一行原因（这一趟一条都不记，本次未跑照 test_not_run_lines 报）。弄坏开关 withhold-whole-binary 下转成它的 marker-withholds-whole-binary 与
# count-mismatch-withholds-whole-binary（一段有记号、或结果行条数与汇总行对不上就整段不记）。
# 多机配置没开 GPU（crash_point_replay_gpu_configured 是 0）时带 --cpu-judging-recordable：判段判在 CPU 的那一条判绿照键记库（文件头「复用」）；
# 开了 GPU 不带（判段判在 CPU 的那一趟判红，判归哪一条那一条不记绿；归得到的另记红、归不到的整个二进制不记，record_cpu_judging_pass）。弄坏开关 cpu-judging-withheld 下没开 GPU 也不带（一律不记），
# cpu-judging-recorded-when-gpu-configured 下开了 GPU 也带
account_test_results() {
  local -a break_setting=() cpu_judging_option=()
  [[ "$crash_point_replay_break" == withhold-whole-binary ]] && break_setting=("LIBTEST_RESULT_LINES_BREAK=marker-withholds-whole-binary,count-mismatch-withholds-whole-binary")
  if { (( ! crash_point_replay_gpu_configured )) && [[ "$crash_point_replay_break" != cpu-judging-withheld ]]; } \
       || [[ "$crash_point_replay_break" == cpu-judging-recorded-when-gpu-configured ]]; then
    cpu_judging_option=(--cpu-judging-recordable)
  fi
  if env ${break_setting[@]+"${break_setting[@]}"} python3 "$crash_point_replay_stage_repository/scripts/gate/lib_libtest_result_lines.py" recordable "$1" \
       ${cpu_judging_option[@]+"${cpu_judging_option[@]}"} >"$2" 2>"$2.err" && grep -q '^RECORDABLE ' "$2"; then
    return 0
  fi
  echo "  ! 按条记库：lib_libtest_result_lines.py recordable 没成（$(tail -n 2 "$2" "$2.err" 2>/dev/null | tr '\n' ' ' | clip_text 300)），这一趟一条都不记"
  return 1
}


# account_feature_pass <带特性那一趟的日志> <排在最后另起的那一趟的日志，没另起写 -> <清单文件>：
# feature-gated-tests 格带特性那一趟逐条认结局（account_test_results）；另起了排在最后的那一趟的，那一趟另认，认出的行并进同一份清单
# （判绿记库与补判都按行读）。哪一趟认不成返回 1（这一趟一条都不记；带特性那一趟已认出的行留在清单里，补判照读）
account_feature_pass() {
  account_test_results "$1" "$3" || return 1
  [[ "$2" != - ]] || return 0
  account_test_results "$2" "$3.last-pass" || return 1
  cat -- "$3.last-pass" >> "$3"
}


# green_names_of_pass <计划输出> <清单文件> <名字文件> <行头>：这一趟现判而判绿、可记库的测试函数名字落进 <名字文件>（<清单文件> 是 account_test_results 认出的，
# 只取 green 那几条），现判而不记的逐条列出为什么（至多 20 条）。取不出名单打一行「! <行头>：…」、<名字文件> 清空、返回 1（这一趟一条都不记）。
# 判绿的名单只在 scripts/gate/admission.py 的 plan-green-names 一处（它文件头「按条复用的共用段」，scripts/crash/crash-point-replay.sh 按项跑的那一段调同一条）
green_names_of_pass() {
  local plan_output="$1" accounting="$2" names="$3" label="$4" withheld_count
  if ! python3 "$crash_point_replay_admission_module" plan-green-names "$plan_output" "$accounting" "$names.withheld" >"$names"; then
    echo "  ! ${label}：取不出判绿的名单（admission.py plan-green-names：$(tr '\n' ' ' <"$names" | clip_text 300)），这一趟一条都不记"
    : >"$names"
    return 1
  fi
  withheld_count="$(grep -c . "$names.withheld" 2>/dev/null)"
  if (( ${withheld_count:-0} > 0 )); then
    echo "  · 判绿不记库 ${withheld_count} 条（这一趟现判、按条记库不认的；下一趟照跑）："
    head -n 20 "$names.withheld" | awk -F'\t' '{ printf "    不记 %s（%s）\n", $1, substr($2, 1, 300) }'
    (( withheld_count > 20 )) && echo "    …另有 $((withheld_count - 20)) 条没列"
  fi
  return 0
}


# record_green_tests <种类> <计划文件> <计划输出> <清单文件> <格名> <跑法>：这一趟现判而判绿的测试函数照键记进库（文件头「复用」：名单经 green_names_of_pass 取）。
# 打一行「· 判绿记库：…」；记不下只报、不判红；复用哨兵报红（强制现判那一条判定不稳）的经 plan_failure_verdict 判失灵、返回 1。
# 那一趟 cargo test 退非 0 的不走这里，走 record_red_tests（判红的与判绿的同一次记）
record_green_tests() {
  local kind="$1" plan_file="$2" plan_output="$3" accounting="$4" cell="$5" how="$6" names="$crash_point_replay_scratch_directory/green-names.$5" record_exit
  green_names_of_pass "$plan_output" "$accounting" "$names" 判绿记库 || return 0
  if [[ ! -s "$names" ]]; then echo "  · 判绿记库：这一趟没有可记的（现判的都不认，或一条都没现判）"; return 0; fi
  if python3 "$crash_point_replay_admission_module" verdict-record "$ROOT" "$kind" "$plan_file" "$names" --how "$how" \
       >"$names.record" 2>&1; then record_exit=0; else record_exit=$?; fi
  if ((record_exit == 0)); then
    echo "  · 判绿记库：$(tail -n 1 "$names.record")"
  else
    echo "  ! 判绿记库：admission.py verdict-record 没成（$(tail -n 2 "$names.record" | tr '\n' ' ')），下一趟这几条照跑"
    plan_failure_verdict "$cell" verdict-record "$record_exit" "$names.record" || return 1
  fi
  return 0
}


# record_red_tests <种类> <计划文件> <计划输出> <红的测试函数清单> <格名> <跑法> [<清单文件>]：按条复用那一趟判红时（文件头「复用」记红、撤绿、下趟现判），
#   report_red_test_targets 落下的「<源文件><TAB><libtest 全名>」按（目标根文件，libtest 全名）对到计划里 judge、reuse、refused 的那一行，取它第一列的名字，
#   经 admission.py verdict-record --red-names 照计划里的键记红、撤掉同一把键原有的绿（拒算的没有键，verdict-record 不记），下一趟这几条现判、原因以「上次判红」起头。
#   给了 <清单文件>（这一趟 account_test_results 认成了）：同一趟判绿的经 green_names_of_pass 取名单、交同一次 verdict-record 照键记绿（文件头「复用」：
#   只认有明确结局行的；编不过、一条结局行都没有的，名单是空的，一条都不记）；没给的（认不成，或弄坏开关 *-red-withholds-green 下调用处不给）判绿的一条都不记。
#   打「· 判红记库：…」与「· 判绿记库（<格名> 这一趟判红）：…」各一行；记不下的交 plan_failure_verdict（这一格已经判红）：交了红的带 --red，
#   记红没成由 admission.py plan-failure 判失灵、撤掉这几把键原有的绿（撤不掉说整道判红，它文件头「按条复用失灵」），复用哨兵报的失灵照打出路。
#   弄坏开关 red-not-recorded 下不交红的（门禁判红的不撤同一把键原有的绿），判绿的照记；
#   red-record-failure-kept-green 下记红没成只打一行「!」、不交 plan-failure（同一把键原有的绿留着，下一趟照复用）
record_red_tests() {
  local kind="$1" plan_file="$2" plan_output="$3" red_rows="$4" cell="$5" how="$6" accounting="${7:-}" red_names="$crash_point_replay_scratch_directory/red-names.$5"
  local green_names="$crash_point_replay_scratch_directory/red-names.$5.green" red_count=0 green_count=0 record_exit
  local -a red_option=()
  : >"$red_names"; : >"$green_names"
  if [[ "$crash_point_replay_break" != red-not-recorded && -s "$red_rows" ]]; then
    awk -F'\t' 'FNR == NR { if (NF >= 7 && ($2 == "judge" || $2 == "reuse" || $2 == "refused")) name[$5 SUBSEP $6] = $1; next }
      (($1 SUBSEP $2) in name) && !seen[name[$1 SUBSEP $2]]++ { print name[$1 SUBSEP $2] }' "$plan_output" "$red_rows" >"$red_names"
    red_count="$(grep -c . "$red_names")"
    if (( red_count == 0 )); then
      echo "  · 判红记库：红的 $(grep -c . "$red_rows") 条测试函数在这一趟的计划里一条都对不上，不记"
    else
      red_option=(--red-names "$red_names")
    fi
  fi
  if [[ -n "$accounting" ]] && green_names_of_pass "$plan_output" "$accounting" "$green_names" "判绿记库（$cell 这一趟判红）"; then
    green_count="$(grep -c . "$green_names")"
    (( green_count > 0 )) || { echo "  · 判绿记库（$cell 这一趟判红）：没有可记的（判绿的都不认、一条都没判绿，或编不过、一条结局行都没有）"; }
  fi
  (( red_count + green_count > 0 )) || return 0
  if python3 "$crash_point_replay_admission_module" verdict-record "$ROOT" "$kind" "$plan_file" "$green_names" ${red_option[@]+"${red_option[@]}"} --how "$how" \
       >"$red_names.record" 2>&1; then record_exit=0; else record_exit=$?; fi
  if ((record_exit == 0)); then
    (( red_count > 0 )) && echo "  · 判红记库：${red_count} 条记红、撤掉同一把键原有的绿，下一趟照现判（$(tail -n 1 "$red_names.record")）"
    (( green_count > 0 )) && echo "  · 判绿记库（$cell 这一趟判红）：判绿的 ${green_count} 条照键记库（$(tail -n 1 "$red_names.record")）"
  elif [[ "$crash_point_replay_break" == red-record-failure-kept-green ]] && (( red_count > 0 )); then
    # 弄坏开关：记红没成只打一行「!」、不交 plan-failure，同一把键原有的绿留着、下一趟照复用
    echo "  ! 判红记库：admission.py verdict-record 退 $record_exit（$(tail -n 2 "$red_names.record" | tr '\n' ' ' | clip_text 300)），红的没记下、同一把键原有的绿还在；判绿的 ${green_count} 条也没记下，下一趟照跑"
  else
    echo "  · 判红记库：admission.py verdict-record 退 $record_exit（$(tail -n 2 "$red_names.record" | tr '\n' ' ' | clip_text 300)）；判绿的 ${green_count} 条照它报的记没记，下一趟照判；红的 ${red_count} 条交 plan-failure 判、撤同一把键原有的绿"
    # 记红没成（admission.py 文件头「按条复用失灵」）：带 --red 交 plan-failure，它判失灵、撤掉这几把键原有的绿（撤不掉说整道判红）；
    # 这一格已经判红：记库那一步 prune-server 不可用也照红算（crash_point_replay_red_regardless_cells），不当只是不可用
    local -a red_withdrawal=()
    (( red_count > 0 )) && red_withdrawal=(--red "$ROOT" "$kind" "$plan_file" "$red_names")
    plan_failure_verdict "$cell" verdict-record "$record_exit" ${red_withdrawal[@]+"${red_withdrawal[@]}"} "$red_names.record" \
      || crash_point_replay_red_regardless_cells+=("$cell")
  fi
  return 0
}


# record_cpu_judging_pass <种类> <计划文件> <计划输出> <清单文件> <格名> <跑法>：多机配置开了 GPU、那一趟 cargo test 退 0 而日志里有判段判在 CPU 的行时
#   （文件头「多机配置开了 GPU」：这一格照旧判红），同一趟的判定照记，记法同判红那一趟（record_red_tests：红的与绿的交同一次 verdict-record）。
#   <清单文件> 是那一趟 account_test_results / account_feature_pass 认出的：开了 GPU 不带 --cpu-judging-recordable，判在 CPU 的记号认作谁打的，那几条已经不在判绿里。
#   判在 CPU 的记号（清单里 marker … cpu-judging 那几行）分两层认：complete 是 1、认作谁打的恰好一条的（打它时
#   只剩那一条还没出结果），归到那一条测试函数：照计划里的键记红、撤掉同一把键原有的绿，下一趟现判、原因以「上次判红」起头；
#   别的（几条同时在跑、这一段认不全）归不到测试函数、只归到它所在的测试二进制：那个二进制里判绿的这一趟一条都不记（清单里改成 withheld、写明为什么），
#   别的二进制判绿的照键记绿。打一行「· <格名>：判段判在 CPU 的记号 N 处：…」报两层各几处、归到二进制的是哪几个。
#   一处记号都没认出（判在 CPU 的行不在哪个测试二进制那一段里）的，归不到谁，这一趟一条都不记、打一行「!」。
#   调用处的弄坏开关 plain-pass-cpu-judging-records-nothing、feature-pass-cpu-judging-records-nothing 下不调它（这一趟一条都不记）
record_cpu_judging_pass() {
  local kind="$1" plan_file="$2" plan_output="$3" accounting="$4" cell="$5" how="$6"
  local red_rows="$crash_point_replay_scratch_directory/cpu-judging-red-rows.$5" kept="$crash_point_replay_scratch_directory/cpu-judging-accounting.$5"
  local function_count=0 binary_count=0 binaries=""
  : >"$red_rows"
  if ! awk -F'\t' -v red_rows="$red_rows" -v counts="$kept.counts" 'BEGIN { OFS = "\t" }
      FNR == NR {
        if ($1 == "marker" && $3 == "cpu-judging") {
          if ($4 == 1 && NF == 6) { functions++; if (!seen[$2 SUBSEP $6]++) print $2, $6 >> red_rows }
          else { located++; if (!($2 in binary)) { binary[$2] = 1; order[++binaries] = $2 } }
        }
        next
      }
      $1 == "green" && ($2 in binary) { print "withheld", $2, $3, "这个测试二进制打了判段判在 CPU 的一行、归不到哪一条测试函数（几条同时在跑，或这一段认不全），整个二进制判绿的这一趟都不记"; next }
      { print }
      END { listed = ""; for (index_in_order = 1; index_in_order <= binaries; index_in_order++) listed = listed (index_in_order > 1 ? "、" : "") order[index_in_order]
            printf "%d\t%d\t%s\n", functions, located, listed > counts }' "$accounting" "$accounting" >"$kept"; then
    echo "  ! $cell：判段判在 CPU 的记号分不了层（awk 没成），这一趟一条都不记，下一趟照跑"
    return 0
  fi
  IFS=$'\t' read -r function_count binary_count binaries <"$kept.counts" || :
  if (( function_count + binary_count == 0 )); then
    echo "  ! $cell：日志里有判段判在 CPU 的行，逐条认结局却一处记号都没认出（那几行不在哪个测试二进制那一段里），归不到谁，这一趟一条都不记，下一趟照跑"
    return 0
  fi
  echo "  · $cell：判段判在 CPU 的记号 $((function_count + binary_count)) 处：归到测试函数的 ${function_count} 处（那几条照键记红、撤绿，下一趟现判），归不到测试函数、只归到测试二进制的 ${binary_count} 处（${binaries:-无}：这几个二进制里判绿的这一趟都不记）；别的判绿的照键记绿"
  record_red_tests "$kind" "$plan_file" "$plan_output" "$red_rows" "$cell" "$how" "$kept"
}


# not_run_markers_of <清单文件> <落进的文件>：account_test_results 认出的 NOT_RUN_ 记号，一处一行「<源文件><TAB><complete：1|0><TAB><那一截>[<TAB><认作谁打的>…]」
not_run_markers_of() {
  awk -F'\t' 'BEGIN { OFS = "\t" } $1 == "marker" && $3 == "not-run" { line = $2 OFS $4 OFS $5; for (i = 6; i <= NF; i++) line = line OFS $i; print line }' "$1" >"$2"
}


# supplement_plain_not_run <带特性那一趟的计划输出> <它的清单文件> <计划算没算成：1|0>：不带特性那一趟打了 NOT_RUN_ 的测试函数（crash_point_replay_plain_not_run_file）
# 带特性那一趟补判到没有（文件头「多机配置开了 GPU」）：认作它打的每一条在这一趟判绿（清单文件里的 green）、或这一组特性下有键相同的判定复用，才算判到。
# 一处一行「<1|0><TAB><哪几条怎么判到的，或为什么没判到>」落进 crash_point_replay_supplement_file，与 crash_point_replay_plain_not_run_file 逐行对应
supplement_plain_not_run() {
  python3 - "$crash_point_replay_plain_not_run_file" "$1" "$2" "$3" >"$crash_point_replay_supplement_file" <<'PY' || : >"$crash_point_replay_supplement_file"
import sys
markers_file, plan_output, accounting, plan_ok = sys.argv[1:5]
decisions = {}
if plan_ok == "1":
    for line in open(plan_output, encoding="utf-8", errors="replace"):
        columns = line.rstrip("\n").split("\t")
        if len(columns) >= 7:
            decisions[(columns[4], columns[5])] = columns[1]
green = set()
for line in open(accounting, encoding="utf-8", errors="replace"):
    columns = line.rstrip("\n").split("\t")
    if len(columns) >= 3 and columns[0] == "green":
        green.add((columns[1], columns[2]))
for line in open(markers_file, encoding="utf-8", errors="replace"):
    columns = line.rstrip("\n").split("\t")
    source, complete, candidates = columns[0], (columns[1] if len(columns) > 1 else "0"), [name for name in columns[3:] if name]
    if complete != "1" or not candidates:
        print("0\t认不出是这一段哪一条测试函数打的，补判不了")
        continue
    judged, missing = [], []
    for name in candidates:
        decision = decisions.get((source, name))
        if decision == "reuse":
            judged.append(f"{source}::{name} 键相同、复用")
        elif (source, name) in green and decision in (None, "judge", "refused"):
            judged.append(f"{source}::{name} 判绿")
        elif decision == "not-compiled":
            missing.append(f"{source}::{name} 这一组特性下不编")
        else:
            missing.append(f"{source}::{name} 这一趟没跑到或没判绿")
    print(("0\t" + "、".join(missing)) if missing else ("1\t" + "、".join(judged)))
PY
}


# report_plain_pass_not_run <带特性那一格：ran|red|stopped|空（没点它）>：不带特性那一趟打了 NOT_RUN_ 的，补判到的打一行「·」，没补判到的逐条报本次未跑
# （文件头「多机配置开了 GPU」），条数放进 crash_point_replay_plain_not_run_after
report_plain_pass_not_run() {
  local where complete text _candidates verdict why reason_without="" index=0
  local -a verdicts=()
  crash_point_replay_plain_not_run_after=0
  [[ -s "$crash_point_replay_plain_not_run_file" ]] || return 0
  case "$1" in
    "") reason_without="这一趟没跑带特性那一格（--check 没点 $FEATURE_GATED_TESTS_CELL_NAME）" ;;
    red) reason_without="带特性那一格判红" ;;
    stopped) reason_without="这一趟被停了，带特性那一格没跑" ;;
    *)
      if [[ -f "$crash_point_replay_supplement_file" ]]; then mapfile -t verdicts <"$crash_point_replay_supplement_file"
      elif [[ "$feature_cell_state" == 无对象可判 ]]; then reason_without="验证包 tests/ 下没有挂特性的测试文件，带特性那一趟没起"
      else reason_without="带特性那一趟没交出补判的结论"; fi ;;
  esac
  while IFS=$'\t' read -r where complete text _candidates; do
    verdict="${verdicts[index]:-}"
    index=$((index + 1))
    if [[ -z "$reason_without" && "${verdict%%$'\t'*}" == 1 ]]; then
      echo "  · 不带特性那一趟（${where}）报的本次未跑由带特性那一趟补判到了：${text}（${verdict#*$'\t'}）"
      continue
    fi
    why="$reason_without"
    if [[ -z "$why" && "$verdict" == *$'\t'* ]]; then why="${verdict#*$'\t'}"; fi
    [[ -n "$why" ]] || why="补判的结论没取出来"
    crash_point_replay_plain_not_run_after=$((crash_point_replay_plain_not_run_after + 1))
    crash_point_replay_report_not_run "验证包不带特性那一趟（${where}）报了本次未跑：${text}；带特性那一趟没补判到（${why}），这一趟不算跑过"
  done <"$crash_point_replay_plain_not_run_file"
  return 0
}


# plan_report <计划输出> [<本次未跑>]：成功句里那一截「现判 N、复用 M、拒算 R」（给了第二个参数另接「、本次未跑 K」），拒算另带头三处形态
plan_report() {
  local forms
  forms="$(awk -F'\t' 'NF >= 7 && $2 == "refused" { sub(/^拒算：/, "", $7); split($7, parts, "；"); count[parts[1]]++ } END { for (form in count) print count[form] "\t" form }' "$1" \
    | sort -rn | head -n 3 | awk -F'\t' '{ printf "%s%s 条：%s", (NR > 1 ? "；" : ""), $1, substr($2, 1, 120) }')"
  printf '现判 %s、复用 %s、拒算 %s%s%s' "$plan_judge" "$plan_reuse" "$plan_refused" "${2:+、本次未跑 $2}" "${forms:+（拒算的头几处：$forms）}"
}


# plan_listing <计划输出>：逐条打现判（连同变了的单元）与拒算（连同头一处形态），各至多 20 条；复用的只在样本档或总共不过 40 条时逐条打
plan_listing() {
  local show_reuse=0
  if [[ "$crash_point_replay_sample_mode" == 1 ]] || (( plan_judge + plan_reuse + plan_refused <= 40 )); then show_reuse=1; fi
  awk -F'\t' -v show_reuse="$show_reuse" '
    NF >= 7 && $2 == "judge" { if (judged++ < 20) printf "    现判 %s（%s）\n", $1, substr($7, 1, 300) }
    NF >= 7 && $2 == "refused" { if (refused++ < 20) printf "    拒算 %s（%s）\n", $1, substr($7, 1, 300) }
    NF >= 7 && $2 == "reuse" && show_reuse { printf "    复用 %s\n", $1 }
    END {
      if (judged > 20) printf "    …现判另有 %d 条没列\n", judged - 20
      if (refused > 20) printf "    …拒算另有 %d 条没列\n", refused - 20
    }' "$1"
}


# failed_tests_changed_units <计划输出> <日志>：日志里判 FAILED 的那几条测试，各自这一趟为什么要现判（键对不上时是变了的单元，计划里那一列原样）
failed_tests_changed_units() {
  local name
  while IFS= read -r name; do
    awk -F'\t' -v name="$name" 'NF >= 7 && $6 == name && ($2 == "judge" || $2 == "refused") { printf "    红的 %s：%s\n", $1, substr($7, 1, 400) }' "$1"
  done < <(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' "$2" | LC_ALL=C sort -u)
}


# report_red_test_targets <cargo 的整份输出> <格名> [<红的测试函数落进的文件>]：按条复用两趟判红时逐个红的测试目标抄出 FAILED 的测试函数名与 panic 段，整份输出拷到 ${TMPDIR:-/tmp} 下留着
# （文件头「复用」之后「按条复用两趟判红时抄出红的测试」那一段）。弄坏开关 red-tail-only 下什么都不抄、不留，只剩调用处的尾巴与 FAILED 那一条 grep。
# 给了第三个参数：逐个红的测试目标里 FAILED 的测试函数一条一行「<源文件><TAB><libtest 全名>」落进那份文件（record_red_tests 照它记红；red-tail-only 下照落）。
report_red_test_targets() {
  local log="$1" cell="$2" red_rows="${3:-}" kept_log copied=1
  [[ "$crash_point_replay_break" == red-tail-only ]] && copied=0
  if ! python3 - "$crash_point_replay_stage_repository/scripts/gate/lib_libtest_result_lines.py" "$log" "$cell" "$red_rows" "$copied" <<'PY_RED_TEST_TARGETS'
import importlib.util
import re
import sys

sys.dont_write_bytecode = True
library_path, log_path, cell, red_rows_path, copied = sys.argv[1:6]
specification = importlib.util.spec_from_file_location("lib_libtest_result_lines_for_red_test_targets", library_path)
libtest = importlib.util.module_from_spec(specification)
specification.loader.exec_module(libtest)
PANIC_START = re.compile(r"^thread '(?P<thread>[^']*)' panicked at ")
FAILURE_LIST_ENTRY = re.compile(r"^    (?P<name>\S+)\s*$")
SEGMENT_LINES = 40          # 一段 panic 至多抄几行
SEGMENTS_PER_TARGET = 8     # 一个测试目标至多抄几段 panic
TAIL_WITHOUT_SUMMARY = 15   # 没有汇总行的那一段抄末几行
LINE_CHARACTERS = 400       # 一行至多抄几个字


def shown(line):
    return line if len(line) <= LINE_CHARACTERS else line[:LINE_CHARACTERS] + "…（这一行截了）"


with open(log_path, encoding="utf-8", errors="replace") as handle:
    text = handle.read()
sections = []
for number, line in enumerate(text.split("\n"), 1):
    running = libtest.RUNNING_LINE_FORM.match(line)
    doctests = libtest.DOCTESTS_LINE_FORM.match(line)
    if running or doctests:
        sections.append([running.group("source") if running else f"doctest:{doctests.group('package')}", number, []])
    elif sections:
        sections[-1][2].append((number, line))
red = []
for source, start, numbered in sections:
    summary, failed, pending, in_failure_list = None, [], None, False
    for _number, line in numbered:
        parsed = libtest.result_line(line)
        if parsed is not None:
            pending = None
            if parsed.outcome == "FAILED":
                failed.append(parsed.full_name)
            continue
        if summary is None and libtest.summary_line(line) is not None:
            summary = libtest.summary_line(line)
            in_failure_list = False
            continue
        started = libtest.SCATTERED_START_FORM.match(line)
        if started is not None and not line.startswith("test result: "):
            pending = started.group("name")
            continue
        alone = libtest.STANDALONE_OUTCOME_FORM.match(line)
        if alone is not None and pending is not None:
            if alone.group("outcome") == "FAILED":
                failed.append(pending)
            pending = None
            continue
        if line.rstrip() == "failures:":
            in_failure_list = True
            continue
        if in_failure_list:
            entry = FAILURE_LIST_ENTRY.match(line)
            if entry is not None:
                failed.append(entry.group("name"))
            elif line.strip():
                in_failure_list = False
    failed = list(dict.fromkeys(failed))
    if summary is not None and summary.outcome != "FAILED" and summary.failed == 0 and not failed:
        continue
    segments, segment = [], None
    for number, line in numbered:
        if PANIC_START.match(line):
            segment = [number, [line]]
            segments.append(segment)
        elif segment is not None and (line.startswith("test ") or line.startswith("---- ") or line.rstrip() == "failures:"
                                      or libtest.STANDALONE_OUTCOME_FORM.match(line)):
            segment = None
        elif segment is not None:
            segment[1].append(line)
    red.append((source, start, numbered[-1][0] if numbered else start, summary, failed, segments, numbered, pending))
if red_rows_path:
    with open(red_rows_path, "w", encoding="utf-8", errors="surrogateescape") as handle:
        handle.write("".join(f"{target[0]}\t{name}\n" for target in red for name in target[4]))
if copied != "1":
    sys.exit(0)
failed_total = sum(len(target[4]) for target in red)
print(f"  · {cell}：红的测试目标 {len(red)} 个、FAILED 的测试函数 {failed_total} 条（逐个目标抄在下面：FAILED 的名字与 panic 段，"
      "取自这一趟 cargo 的整份输出，不只是上面的尾巴）")
for source, start, end, summary, failed, segments, numbered, pending in red:
    summary_text = (f"汇总行 {summary.outcome}. {summary.passed} passed; {summary.failed} failed" if summary is not None
                    else "没有汇总行：测试二进制没跑完（被信号杀、撞了上限，或中途退了）")
    print(f"    红的测试目标 {source}（日志第 {start}–{end} 行；{summary_text}）：")
    for name in failed:
        print(f"      FAILED {name}")
    if summary is not None and summary.failed and not failed:
        print(f"      （汇总行报 {summary.failed} 条失败，名字一条都没认出：结果行被测试自己的输出打散了，看整份输出）")
    for number, lines in segments[:SEGMENTS_PER_TARGET]:
        thread = PANIC_START.match(lines[0]).group("thread")
        while lines and not lines[-1].strip():
            lines.pop()
        print(f"      panic 段（thread '{thread}'，日志第 {number} 行起 {len(lines)} 行{'，抄前 ' + str(SEGMENT_LINES) + ' 行' if len(lines) > SEGMENT_LINES else ''}）：")
        for line in lines[:SEGMENT_LINES]:
            print(f"        {shown(line)}")
    if len(segments) > SEGMENTS_PER_TARGET:
        print(f"      …另有 {len(segments) - SEGMENTS_PER_TARGET} 段 panic 没抄，看整份输出")
    if summary is None and pending is not None:
        print(f"      跑到一半的测试函数 {pending}（只打了「test {pending} ... 」起头，没有结局）")
    if summary is None:
        print(f"      这一段末 {min(TAIL_WITHOUT_SUMMARY, len(numbered))} 行：")
        for _number, line in numbered[-TAIL_WITHOUT_SUMMARY:]:
            print(f"        {shown(line)}")
PY_RED_TEST_TARGETS
  then
    echo "    ! $cell：抄红的测试函数名与 panic 段那一步没成（上面是 python 的报错）；整份输出看下一行的路径"
  fi
  (( copied )) || return 0
  kept_log="$(mktemp "${TMPDIR:-/tmp}/crash-point-replay-$cell.XXXXXX.log")" || kept_log=""
  if [[ -n "$kept_log" ]] && cp -- "$log" "$kept_log"; then
    crash_point_replay_kept_logs+=("$kept_log"$'\t'"$(sha256sum < "$log" | cut -d' ' -f1)")
    echo "    · $cell：这一趟 cargo 的整份输出留在 $kept_log（这一道的草稿目录跑完就删，这一份不删；看完自己删）"
  else
    [[ -n "$kept_log" ]] && rm -f -- "$kept_log"
    echo "    ! $cell：这一趟 cargo 的整份输出没留下（拷不到 ${TMPDIR:-/tmp} 下）；上面抄出的名字与 panic 段是这一趟仅有的记录"
  fi
}


# report_pass_outcome <格名> <哪一趟> <退出码> <整份输出>：一趟 cargo 一收尾就往标准输出打这一趟的结局（文件头「每一趟当场写结局」）：退出码、test result 汇总行
# （几行、合计通过 / 失败 / ignored、FAILED 的几行）、FAILED 的测试函数（结果行「test <名> ... FAILED」与 libtest 末尾 failures: 名单取并集，至多列 20 条）、整份输出的路径。
# 不判、不改这一格的结论。弄坏开关 pass-outcome-at-cell-end 下不当场打：先记进 crash_point_replay_deferred_pass_outcomes，整格收尾才由 flush_deferred_pass_outcomes 打
report_pass_outcome() {
  local cell="$1" pass="$2" pass_exit="$3" log="$4" line summary_count=0 passed_total=0 failed_total=0 ignored_total=0 failed_summary_count=0 names_text outcome
  local -a failed_names=()
  if [[ -f "$log" ]]; then
    while IFS= read -r line; do
      summary_count=$((summary_count + 1))
      if [[ "$line" =~ ([0-9]+)\ passed\;\ ([0-9]+)\ failed\;\ ([0-9]+)\ ignored ]]; then
        passed_total=$((passed_total + BASH_REMATCH[1])); failed_total=$((failed_total + BASH_REMATCH[2])); ignored_total=$((ignored_total + BASH_REMATCH[3]))
      fi
      [[ "$line" == "test result: FAILED"* ]] && failed_summary_count=$((failed_summary_count + 1))
    done < <(grep '^test result: ' "$log")
    mapfile -t failed_names < <(awk '
      /^test .* \.\.\. FAILED$/ { name = $0; sub(/^test /, "", name); sub(/ \.\.\. FAILED$/, "", name); print name; next }
      /^failures:$/ { in_list = 1; next }
      in_list && /^    [^ ]+$/ { name = $0; sub(/^    /, "", name); print name; next }
      in_list && /^[^ ]/ { in_list = 0 }' "$log" | awk '!seen[$0]++')
  fi
  names_text=""
  if (( ${#failed_names[@]} > 0 )); then
    names_text="：$(printf '%s、' "${failed_names[@]:0:20}")"
    names_text="${names_text%、}"
    (( ${#failed_names[@]} > 20 )) && names_text+="（另有 $(( ${#failed_names[@]} - 20 )) 条，看整份输出）"
  fi
  if (( summary_count == 0 )); then
    outcome="test result 汇总行 0 行（没跑完：被信号杀、撞了上限，或中途退了；一条都没起的也是 0 行）"
  else
    outcome="test result 汇总行 ${summary_count} 行（合计通过 ${passed_total}、失败 ${failed_total}、ignored ${ignored_total}；FAILED 的汇总行 ${failed_summary_count} 行）"
  fi
  line="  · ${cell}：${pass}收尾，当场写结局：cargo 退 ${pass_exit}；${outcome}；FAILED 的测试函数 ${#failed_names[@]} 条${names_text}；整份输出：${log}"
  if [[ "$crash_point_replay_break" == pass-outcome-at-cell-end ]]; then
    printf '%s\n' "$line" >> "$crash_point_replay_deferred_pass_outcomes"
    return 0
  fi
  printf '%s\n' "$line"
}


# flush_deferred_pass_outcomes：整格收尾时打弄坏开关 pass-outcome-at-cell-end 下记下的那几趟结局（等整格收尾才写），打完清空；没开开关时这份是空的、什么都不打
flush_deferred_pass_outcomes() {
  [[ -s "$crash_point_replay_deferred_pass_outcomes" ]] || return 0
  cat -- "$crash_point_replay_deferred_pass_outcomes"
  : > "$crash_point_replay_deferred_pass_outcomes"
}
