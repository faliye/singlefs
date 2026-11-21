#!/usr/bin/env bash
# 合入后验证 merge-and-verify.sh 的函数库：逐项点名与按条复用（项的收集、test-plan 与记库）、verify（测试那一步）、--list-items、合进主工作区的命令。
# 被 source，只定义函数、不改 shell 选项；读写入口脚本的全局变量（同一个 shell）。

# word_in_list <词> <词…>：第一个词在后面那几个里退 0
word_in_list() {
  local wanted="$1" candidate
  shift
  for candidate in "$@"; do [[ "$candidate" == "$wanted" ]] && return 0; done
  return 1
}

# item_mode：这一趟点了项（文件头「逐项点名与按条复用」）退 0。弄坏开关 items-ignored 下点了也当没点
item_mode() { ((${#NAMED_ITEMS[@]})) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != items-ignored ]]; }

# unresolved_left_active：带了 --unresolved-to-commit-gate、又点了 --item 的真跑那一趟退 0（文件头「判不出的留给提交时整轮门禁」）：判不出改到哪个测试目标的几处不挡测试那一步与验证包那一步，
#   逐条打成「留给提交时整轮门禁」，点名的项照跑；--list-items 不看这个选项。
#   弄坏开关 unresolved-option-ignored：带了选项也当没带；unresolved-without-option：没带选项也照带了算；unresolved-without-item：带了选项而没点 --item 也生效
unresolved_left_active() {
  ((LIST_ITEMS == 0)) || return 1
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != unresolved-option-ignored ]] || return 1
  ((LEAVE_UNRESOLVED_TO_COMMIT)) || [[ "${MERGE_AND_VERIFY_BREAK:-}" == unresolved-without-option ]] || return 1
  item_mode || [[ "${MERGE_AND_VERIFY_BREAK:-}" == unresolved-without-item ]]
}

# collect_test_items：挑出的项（ALL_ITEM_LINES，select_test_targets 填）里，点了 --item 的只留点名的进 RUN_ITEMS，写错的进 UNKNOWN_ITEMS；没点的全留。
#   弄坏开关 item-unknown-accepted 下写错的悄悄丢掉（丢完一项不剩就当没点）；items-ignored 下点了也全留
collect_test_items() {
  local item line
  local -a known=() kept_names=()
  RUN_ITEMS=(); UNKNOWN_ITEMS=()
  for line in ${ALL_ITEM_LINES[@]+"${ALL_ITEM_LINES[@]}"}; do known+=("${line%%$'\t'*}"); done
  for item in ${NAMED_ITEMS[@]+"${NAMED_ITEMS[@]}"}; do
    word_in_list "$item" ${known[@]+"${known[@]}"} || UNKNOWN_ITEMS+=("$item")
  done
  if ((${#UNKNOWN_ITEMS[@]})) && [[ "${MERGE_AND_VERIFY_BREAK:-}" == item-unknown-accepted ]]; then
    for item in "${NAMED_ITEMS[@]}"; do word_in_list "$item" "${UNKNOWN_ITEMS[@]}" || kept_names+=("$item"); done
    NAMED_ITEMS=(${kept_names[@]+"${kept_names[@]}"}); UNKNOWN_ITEMS=()
  fi
  if ! item_mode; then RUN_ITEMS=(${ALL_ITEM_LINES[@]+"${ALL_ITEM_LINES[@]}"}); return 0; fi
  for line in ${ALL_ITEM_LINES[@]+"${ALL_ITEM_LINES[@]}"}; do
    word_in_list "${line%%$'\t'*}" "${NAMED_ITEMS[@]}" && RUN_ITEMS+=("$line")
  done
  return 0
}

# run_item_roots <包>：RUN_ITEMS 里这个包的项的目标根文件（文档测试那一项不算），一行一个
run_item_roots() {
  local line item rest
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    item="${line%%$'\t'*}"; rest="${line#*$'\t'}"
    [[ "${rest%%$'\t'*}" == "$1" && "${item#"$1":}" != doc ]] && printf '%s\n' "${item#"$1":}"
  done
  return 0
}

# read_test_functions_request <阶段文件名> <去掉 --profile：1 或 0> [<交给 env 的赋值>…]：按条复用的请求（admission.py test-functions-request；同门禁 crash-point-replay 按条复用的取法）
#   → REQUEST_KIND 与 REQUEST_ARGUMENTS；交不出就打 ✗、两样都空。赋值照写进起它的环境（改判定域的变量要与交给 cargo 的同一份）
read_test_functions_request() {
  local stage_name="$1" strip_profile="$2" request_output tag value skip_next=0
  shift 2
  REQUEST_KIND=""; REQUEST_ARGUMENTS=()
  if request_output="$(env "$@" python3 "$ADMISSION" test-functions-request "$stage_name" "$root" 2>&1)"; then
    while IFS=$'\t' read -r tag value; do
      case "$tag" in
        kind) REQUEST_KIND="$value" ;;
        argument)
          if ((skip_next)); then skip_next=0
          elif ((strip_profile)) && [[ "$value" == --profile ]]; then skip_next=1
          else REQUEST_ARGUMENTS+=("$value"); fi ;;
      esac
    done <<<"$request_output"
  fi
  if [[ -z "$REQUEST_KIND" ]] || ((${#REQUEST_ARGUMENTS[@]} == 0)); then
    echo "  ✗ 按条复用：admission.py test-functions-request $stage_name 交不出请求（$(tr '\n' ' ' <<<"$request_output" | clip_text 300)），用它的包一条都不跳、全部现判"
    echo "  → 怎么办：单独跑 python3 $ADMISSION test-functions-request $stage_name $root，看它为什么没打出 kind 与 argument 两种行（读不了阶段文件、种类登记缺了这类）；修好它再重跑合入后验证，按条复用才接得上"
    REQUEST_KIND=""; REQUEST_ARGUMENTS=()
  fi
  return 0
}

# package_kind <包>：这个包的按条复用种类（文件头「逐项点名与按条复用」：验证包是 VERIFICATION_FUNCTIONS_KIND，别的是 TEST_FUNCTIONS_KIND）
package_kind() { if [[ "$1" == "$VERIFICATION_PACKAGE" ]]; then printf '%s' "$VERIFICATION_FUNCTIONS_KIND"; else printf '%s' "$TEST_FUNCTIONS_KIND"; fi; }

# plan_test_functions <树>：跑到的项所在、有测试属性的包各起一次 admission.py test-plan（文件头「逐项点名与按条复用」的按条复用），计划放 PLAN_DIRECTORY 下
#   （test-plan.<包>.out 与 .plan）；算成的进 PLANNED_PACKAGES，失灵的进 FAILED_PLAN_PACKAGES 并置 PLAN_RED，判 prune-server 不可用的置 PLAN_UNAVAILABLE；
#   跑到的项里计划的条数进 PLAN_JUDGE、PLAN_REUSE、PLAN_REFUSED。验证包用 crash-point-replay.sh 的请求（--profile release 照留）另加 --exclude-registered，
#   缩过域时请求在带那两个变量的环境里起（crash_expand_assignments）。弄坏开关 plan-failure-ignored 下失灵不置 PLAN_RED；plan-unavailable-run-anyway 下不可用当失灵、照全部现判跑
plan_test_functions() {
  local tree_root="$1" line package plan_exit verdict_exit out failure_text judge reuse refused kind
  local -a packages=() plan_arguments=()
  PLANNED_PACKAGES=(); FAILED_PLAN_PACKAGES=(); PLAN_UNAVAILABLE=0; PLAN_RED=0; PLAN_JUDGE=0; PLAN_REUSE=0; PLAN_REFUSED=0
  TEST_FUNCTIONS_KIND=""; TEST_PLAN_ARGUMENTS=(); VERIFICATION_FUNCTIONS_KIND=""; VERIFICATION_PLAN_ARGUMENTS=()
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    line="${line#*$'\t'}"; package="${line%%$'\t'*}"
    if word_in_list "$package" ${TESTED_PACKAGES[@]+"${TESTED_PACKAGES[@]}"} && ! word_in_list "$package" ${packages[@]+"${packages[@]}"}; then packages+=("$package"); fi
  done
  ((${#packages[@]})) || return 0
  for package in "${packages[@]}"; do
    if [[ "$package" == "$VERIFICATION_PACKAGE" ]]; then
      crash_expand_assignments
      read_test_functions_request crash-point-replay.sh 0 ${CRASH_EXPAND_ASSIGNMENTS[@]+"${CRASH_EXPAND_ASSIGNMENTS[@]}"}
      VERIFICATION_FUNCTIONS_KIND="$REQUEST_KIND"; VERIFICATION_PLAN_ARGUMENTS=(${REQUEST_ARGUMENTS[@]+"${REQUEST_ARGUMENTS[@]}"})
      ((${#VERIFICATION_PLAN_ARGUMENTS[@]} == 0)) || VERIFICATION_PLAN_ARGUMENTS+=(--exclude-registered)
    elif [[ -z "$TEST_FUNCTIONS_KIND" ]]; then
      read_test_functions_request merge-and-verify.sh 1
      TEST_FUNCTIONS_KIND="$REQUEST_KIND"; TEST_PLAN_ARGUMENTS=(${REQUEST_ARGUMENTS[@]+"${REQUEST_ARGUMENTS[@]}"})
    fi
  done
  for package in "${packages[@]}"; do
    kind="$(package_kind "$package")"
    if [[ "$package" == "$VERIFICATION_PACKAGE" ]]; then plan_arguments=(${VERIFICATION_PLAN_ARGUMENTS[@]+"${VERIFICATION_PLAN_ARGUMENTS[@]}"})
    else plan_arguments=(${TEST_PLAN_ARGUMENTS[@]+"${TEST_PLAN_ARGUMENTS[@]}"}); fi
    if [[ -z "$kind" ]] || ((${#plan_arguments[@]} == 0)); then
      FAILED_PLAN_PACKAGES+=("$package")
      [[ "${MERGE_AND_VERIFY_BREAK:-}" == plan-failure-ignored ]] || PLAN_RED=1
      continue
    fi
    out="$PLAN_DIRECTORY/test-plan.$package.out"
    printf '%s\n' "${plan_arguments[@]}" > "$PLAN_DIRECTORY/test-plan.$package.arguments"   # 交给 test-plan 的请求参数，一行一个词（缩过域时里面有两个变量的 variable: 那两项）
    plan_exit=0
    python3 "$ADMISSION" test-plan "$tree_root" "$kind" "$package" "$PLAN_DIRECTORY/test-plan.$package.plan" "${plan_arguments[@]}" >"$out" 2>"$out.err" || plan_exit=$?
    if ((plan_exit == 0)) && grep -q '^PLAN ' "$out"; then
      PLANNED_PACKAGES+=("$package")
      read -r judge reuse refused < <(awk -F'\t' -v roots="$(run_item_roots "$package" | tr '\n' ' ')" 'BEGIN { n = split(roots, list, " "); for (i = 1; i <= n; i++) keep[list[i]] = 1 }
        NF >= 7 && ($5 in keep) { if ($2 == "judge") judge++; else if ($2 == "reuse") reuse++; else if ($2 == "refused") refused++ } END { print judge + 0, reuse + 0, refused + 0 }' "$out")
      PLAN_JUDGE=$((PLAN_JUDGE + judge)); PLAN_REUSE=$((PLAN_REUSE + reuse)); PLAN_REFUSED=$((PLAN_REFUSED + refused))
      continue
    fi
    # 失灵与不可用的判法只在 admission.py plan-failure（它文件头「按条复用失灵」）；它打的红句照转
    verdict_exit=0
    failure_text="$(python3 "$ADMISSION" plan-failure "merge-and-verify.sh 测试那一步的包 $package" test-plan "$plan_exit" "$out" "$out.err" 2>&1)" || verdict_exit=$?
    [[ -z "$failure_text" ]] || printf '%s\n' "$failure_text" | sed 's/^ */  /'
    if ((verdict_exit == 75)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != plan-unavailable-run-anyway ]]; then
      PLAN_UNAVAILABLE=1
    else
      FAILED_PLAN_PACKAGES+=("$package")
      if ((verdict_exit == 1)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != plan-failure-ignored ]]; then PLAN_RED=1; fi
    fi
  done
  echo "  · 按条复用（种类 ${TEST_FUNCTIONS_KIND:-}${VERIFICATION_FUNCTIONS_KIND:+${TEST_FUNCTIONS_KIND:+、}$VERIFICATION_FUNCTIONS_KIND（验证包）}）：${#packages[@]} 个包起了 test-plan，算成 ${#PLANNED_PACKAGES[@]} 个${FAILED_PLAN_PACKAGES[*]:+、失灵 ${#FAILED_PLAN_PACKAGES[@]} 个（${FAILED_PLAN_PACKAGES[*]}，一条都不跳）}$( ((PLAN_UNAVAILABLE)) && echo '、prune-server 不可用')；跑到的项里现判 $PLAN_JUDGE、复用 $PLAN_REUSE、拒算 $PLAN_REFUSED"
  return 0
}

# skip_words_of <包>：键相同、库里不红的测试函数（与验证包登记的用例）的 --exact --skip <名>…（admission.py plan-skip-names）进 SKIP_WORDS；
#   只算这一包跑到的目标，这一包的计划没算成就一条都不跳。弄坏开关 reuse-skips-dropped 下一条都不跳
skip_words_of() {
  local package="$1" names="" name
  local -a roots=() root_options=()
  SKIP_WORDS=()
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != reuse-skips-dropped ]] || return 0
  word_in_list "$package" ${PLANNED_PACKAGES[@]+"${PLANNED_PACKAGES[@]}"} || return 0
  mapfile -t roots < <(run_item_roots "$package")
  for name in ${roots[@]+"${roots[@]}"}; do root_options+=(--target-root "$name"); done
  ((${#root_options[@]})) || return 0
  names="$(python3 "$ADMISSION" plan-skip-names "$PLAN_DIRECTORY/test-plan.$package.out" "${root_options[@]}" 2>&1)" || names=""
  while IFS= read -r name; do [[ -n "$name" ]] && SKIP_WORDS+=(--skip "$name"); done <<<"$names"
  ((${#SKIP_WORDS[@]})) && SKIP_WORDS=(--exact "${SKIP_WORDS[@]}")
  return 0
}

# run_item_test_commands <日志目录> <测试的标志…>：测试那一步（文件头 test 一段与「逐项点名与按条复用」）：每个包一条只带这个包点到的目标，点了文档测试的另起一条 --doc；
#   验证包那一条带 --release（缩过域时带那两个变量起），计划算不出时不起（登记的用例排不掉）。依次跑进 test-<序号>.log，跑完接成 test.log 与 test.problems.log。
#   交头一条退非 0 的退出码。弄坏开关 verification-release-dropped：验证包那一条不带 --release
run_item_test_commands() {
  local logs="$1" line package selector index=0 rc=0 step_rc
  shift
  local -a flags=("$@") packages=() doc_packages=() selectors=() selector_words=() command=() tail_words=() logs_written=() package_flags=() package_environment=()
  for line in "${RUN_ITEMS[@]}"; do
    line="${line#*$'\t'}"; package="${line%%$'\t'*}"
    word_in_list "$package" ${packages[@]+"${packages[@]}"} || packages+=("$package")
  done
  for package in ${packages[@]+"${packages[@]}"}; do
    selectors=()
    for line in "${RUN_ITEMS[@]}"; do
      line="${line#*$'\t'}"
      [[ "${line%%$'\t'*}" == "$package" ]] || continue
      selector="${line#*$'\t'}"
      if [[ "$selector" == --doc ]]; then word_in_list "$package" ${doc_packages[@]+"${doc_packages[@]}"} || doc_packages+=("$package"); continue; fi
      read -r -a selector_words <<<"$selector"; selectors+=("${selector_words[@]}")
    done
    ((${#selectors[@]})) || continue
    package_flags=(${flags[@]+"${flags[@]}"}); package_environment=()
    if [[ "$package" == "$VERIFICATION_PACKAGE" ]]; then
      if word_in_list "$package" ${FAILED_PLAN_PACKAGES[@]+"${FAILED_PLAN_PACKAGES[@]}"}; then
        echo "  ✗ 测试那一步：验证包 $package 的计划算不出（上面那一句），登记的崩溃枚举用例与放量分项排不掉，这一包的 ${selectors[*]} 不起"
        echo "  → 怎么办：照上面 admission.py plan-failure 那一句修好按条复用，再重跑同一条命令"
        continue
      fi
      [[ "${MERGE_AND_VERIFY_BREAK:-}" == "verification-release-dropped" ]] || package_flags=(--release ${flags[@]+"${flags[@]}"})
      # 只点测试目标、没点装置二进制（--bin）的那一条关掉缺省特性：验证包的缺省特性只挂着 src/bin/ 下的装置二进制，
      # cargo 给集成测试编目标时会把它们全编一遍，而测试用不到它们
      if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "verification-default-features-kept" ]] && ! word_in_list --bin "${selectors[@]}"; then
        package_flags+=(--no-default-features)
      fi
      crash_expand_assignments
      [[ -z "$CRASH_EXPAND_UP_TO" ]] || package_environment=(env "${CRASH_EXPAND_ASSIGNMENTS[@]}")
    fi
    skip_words_of "$package"
    tail_words=()
    [[ "${MERGE_AND_VERIFY_BREAK:-}" != "test-arguments-added" ]] || tail_words+=(--show-output)
    tail_words+=(${SKIP_WORDS[@]+"${SKIP_WORDS[@]}"})
    if [[ "${MERGE_AND_VERIFY_TEST_TARGETS:-}" == serial || "${MERGE_AND_VERIFY_BREAK:-}" == "tests-one-binary-at-a-time" ]]; then
      command=(${package_environment[@]+"${package_environment[@]}"} cargo test --offline --no-fail-fast ${package_flags[@]+"${package_flags[@]}"} -p "$package" "${selectors[@]}")
    else
      command=(${package_environment[@]+"${package_environment[@]}"} python3 "$TEST_TARGETS_PARALLEL" --jobs "$THREADS" -- --offline --no-fail-fast ${package_flags[@]+"${package_flags[@]}"} -p "$package" "${selectors[@]}")
    fi
    ((${#tail_words[@]})) && command+=(-- "${tail_words[@]}")
    index=$((index + 1))
    echo "  · 测试第 $index 条：-p $package ${package_flags[*]} ${selectors[*]}（跳过键相同的 $(( ${#SKIP_WORDS[@]} / 2 )) 条）"
    run_step "$logs/test-$index.log" "${command[@]}"; step_rc=$?
    logs_written+=("$logs/test-$index")
    ((rc != 0)) || rc="$step_rc"
  done
  if ((${#doc_packages[@]})); then
    command=(cargo test --offline --no-fail-fast ${flags[@]+"${flags[@]}"})
    for package in "${doc_packages[@]}"; do command+=(-p "$package"); done
    command+=(--doc)
    index=$((index + 1))
    echo "  · 测试第 $index 条：文档测试 ${doc_packages[*]}（不复用）"
    run_step "$logs/test-$index.log" "${command[@]}"; step_rc=$?
    logs_written+=("$logs/test-$index")
    ((rc != 0)) || rc="$step_rc"
  fi
  : > "$logs/test.log"; : > "$logs/test.problems.log"
  for line in ${logs_written[@]+"${logs_written[@]}"}; do
    cat -- "$line.log" >> "$logs/test.log" 2>/dev/null
    [[ ! -f "$line.problems.log" ]] || { printf '== %s.log（行号是那一份里的）\n' "$line"; cat -- "$line.problems.log"; } >> "$logs/test.problems.log"
  done
  return "$rc"
}

# record_test_functions <日志目录> <树>：测试那一步跑完照键记库（文件头「逐项点名与按条复用」的按条复用）：test.log 逐段归到项与包（<日志目录>/test-sections.tsv、test-split/<包>.log），
#   算成计划的每个包：逐条结局并进 test-functions.tsv，lib_libtest_result_lines.py recordable 认结局、plan-green-names 取判绿可记的、计划里要现判而 FAILED 的交 --red-names，
#   同一次 verdict-record 记进库（记红撤绿）。verdict-record 报哨兵红的经 plan-failure 置 PLAN_RED，判不可用的置 PLAN_UNAVAILABLE。弄坏开关 red-not-recorded 下不交 --red-names
record_test_functions() {
  local logs="$1" tree_root="$2" line package item unattributed loose record_exit verdict_exit failure_text package_log kind how
  local split_directory="$logs/test-split"
  local -a items=()
  : > "$logs/test-functions.tsv"; : > "$logs/test-sections.tsv"
  [[ -s "$logs/test.log" ]] || return 0
  rm -rf -- "$split_directory"
  if ! python3 - split "$tree_root" "$RERUN_FAILED_TESTS" "$logs/test.log" "$split_directory" <<<"$MERGE_AND_VERIFY_ITEMS_PROGRAM" > "$logs/test-sections.tsv" 2>&1; then
    echo "  ! 按条复用：test.log 逐段归不到项（$(tail -n 2 "$logs/test-sections.tsv" | tr '\n' ' ')），这一趟一条都不记库，下一趟照现判"
    : > "$logs/test-sections.tsv"
    return 0
  fi
  unattributed="$(awk -F'\t' '$1 == "SECTION" && $2 == "-"' "$logs/test-sections.tsv" | grep -c .)"
  loose="$(awk -F'\t' '$1 == "LOOSE" { print $2 }' "$logs/test-sections.tsv")"
  if ((unattributed > 0 || ${loose:-0} > 0)); then
    echo "  ! 按条复用：test.log 里 $unattributed 段认不出、拿不准属于哪一项，${loose:-0} 条结果行在段头之前（rerun-failed-tests.py 的 section_item），这几条不记库、下一趟照现判"
  fi
  for package in ${PLANNED_PACKAGES[@]+"${PLANNED_PACKAGES[@]}"}; do
    items=()
    for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
      item="${line%%$'\t'*}"; line="${line#*$'\t'}"
      [[ "${line%%$'\t'*}" == "$package" ]] && items+=("$item")
    done
    package_log="$split_directory/$package.log"; [[ -f "$package_log" ]] || package_log=-
    python3 - outcomes "$RERUN_FAILED_TESTS" "$PLAN_DIRECTORY/test-plan.$package.out" "$package_log" "$package" "$PLAN_DIRECTORY/test-plan.$package.red" "${items[@]}" \
      <<<"$MERGE_AND_VERIFY_ITEMS_PROGRAM" >> "$logs/test-functions.tsv" 2>&1 || : > "$PLAN_DIRECTORY/test-plan.$package.red"
    [[ "${MERGE_AND_VERIFY_BREAK:-}" != red-not-recorded ]] || : > "$PLAN_DIRECTORY/test-plan.$package.red"
    : > "$PLAN_DIRECTORY/test-plan.$package.green"
    if [[ "$package_log" != - ]] && python3 "$LIBTEST_RESULT_LINES" recordable "$package_log" > "$PLAN_DIRECTORY/test-plan.$package.accounting" 2>&1; then
      python3 "$ADMISSION" plan-green-names "$PLAN_DIRECTORY/test-plan.$package.out" "$PLAN_DIRECTORY/test-plan.$package.accounting" "$PLAN_DIRECTORY/test-plan.$package.withheld" \
        > "$PLAN_DIRECTORY/test-plan.$package.green" 2>&1 || : > "$PLAN_DIRECTORY/test-plan.$package.green"
    fi
    record_exit=0; kind="$(package_kind "$package")"; how="合入后验证测试那一步（cargo test，dev 档）"
    [[ "$package" != "$VERIFICATION_PACKAGE" ]] || how="合入后验证测试那一步（cargo test --release，验证包点到的目标，登记的崩溃枚举用例与放量分项经 --skip 排掉）"
    python3 "$ADMISSION" verdict-record "$tree_root" "$kind" "$PLAN_DIRECTORY/test-plan.$package.plan" "$PLAN_DIRECTORY/test-plan.$package.green" \
      --how "$how" --red-names "$PLAN_DIRECTORY/test-plan.$package.red" > "$PLAN_DIRECTORY/test-plan.$package.record" 2>&1 || record_exit=$?
    if ((record_exit == 0)); then
      echo "  · 按条复用记库（$package）：判绿 $(grep -c . "$PLAN_DIRECTORY/test-plan.$package.green") 条、判红 $(grep -c . "$PLAN_DIRECTORY/test-plan.$package.red") 条（红的照同一把键记红、撤掉原有的绿，下一趟现判）；$(grep '^RECORD ' "$PLAN_DIRECTORY/test-plan.$package.record" | tail -n 1)"
      continue
    fi
    verdict_exit=0
    # 记红没成（admission.py 文件头「按条复用失灵」）：带 --red 交，它判失灵、撤掉判红那几把键原有的绿；判红的名单空着的照只记绿那一支办
    failure_text="$(python3 "$ADMISSION" plan-failure "merge-and-verify.sh 测试那一步的包 $package" verdict-record "$record_exit" \
      --red "$tree_root" "$kind" "$PLAN_DIRECTORY/test-plan.$package.plan" "$PLAN_DIRECTORY/test-plan.$package.red" "$PLAN_DIRECTORY/test-plan.$package.record" 2>&1)" \
      || verdict_exit=$?
    [[ -z "$failure_text" ]] || printf '%s\n' "$failure_text" | sed 's/^ */  /'
    case "$verdict_exit" in
      0) echo "  ! 按条复用（$package）：判定没记进库（verdict-record 退 $record_exit：$(tail -n 2 "$PLAN_DIRECTORY/test-plan.$package.record" | tr '\n' ' ')），下一趟照判" ;;
      75) PLAN_UNAVAILABLE=1 ;;
      *) [[ "${MERGE_AND_VERIFY_BREAK:-}" == plan-failure-ignored ]] || PLAN_RED=1 ;;
    esac
  done
  return 0
}

# report_items_done <cargo test 的退出码>：测试那一步收尾每项一行 ITEM_DONE（文件头「逐项点名与按条复用」的机读行），读 test-sections.tsv 与各包的计划。
#   弄坏开关 item-done-unreported 下一行都不打
report_items_done() {
  local test_rc="$1" line item package seconds sections failed results judged reused exit_code any_failed=0 sections_file="$PLAN_DIRECTORY/test-sections.tsv"
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != item-done-unreported ]] || return 0
  [[ -f "$sections_file" ]] || : > "$sections_file"
  awk -F'\t' '$1 == "FAILED" && $2 != "-" { found = 1 } END { exit found ? 0 : 1 }' "$sections_file" && any_failed=1
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    item="${line%%$'\t'*}"; line="${line#*$'\t'}"; package="${line%%$'\t'*}"
    read -r sections seconds failed results < <(awk -F'\t' -v item="$item" '$1 == "SECTION" && $2 == item { count++; seconds += $4; failed += $5; results += $6 }
      END { printf "%d %d %d %d\n", count, seconds + 0.5, failed, results }' "$sections_file")
    if [[ "$item" == "$package:doc" ]]; then
      judged=1; reused=0
    elif word_in_list "$package" ${PLANNED_PACKAGES[@]+"${PLANNED_PACKAGES[@]}"}; then
      read -r judged reused < <(awk -F'\t' -v root="${item#"$package":}" 'NF >= 7 && $5 == root { if ($2 == "judge" || $2 == "refused") judge++; else if ($2 == "reuse") reuse++ }
        END { print judge + 0, reuse + 0 }' "$PLAN_DIRECTORY/test-plan.$package.out")
    else
      judged="$results"; reused=0
    fi
    exit_code=0
    if ((failed > 0)) || { ((test_rc != 0)) && ((any_failed == 0)); }; then exit_code=1
    elif ((sections == 0)); then exit_code=77; fi
    printf 'ITEM_DONE item=%s seconds=%s judged=%s reused=%s exit=%s\n' "$item" "$seconds" "$judged" "$reused" "$exit_code"
  done
  return 0
}

# remove_list_items_scratch：--list-items 退出时（EXIT trap）删它建的临时目录（临时快照与计划都在里面）
remove_list_items_scratch() { [[ -z "$LIST_ITEMS_SCRATCH" ]] || rm -rf -- "${LIST_ITEMS_SCRATCH:?}"; }

# list_items_scratch_directory：--list-items 的临时目录，第一次要用时在 ${TMPDIR:-/tmp} 下建、挂上退出时删的 trap；建不成退 1
list_items_scratch_directory() {
  [[ -z "$LIST_ITEMS_SCRATCH" ]] || return 0
  LIST_ITEMS_SCRATCH="$(mktemp -d "${TMPDIR:-/tmp}/merge-and-verify-list-items.XXXXXX")" || die "在 ${TMPDIR:-/tmp} 下建不了列项用的临时目录，一项都没列" "看 ${TMPDIR:-/tmp} 在不在、可不可写、有没有空间" 1
  trap remove_list_items_scratch EXIT
}

# list_items_and_exit：--list-items（文件头「逐项点名与按条复用」）：照这一趟的挑法挑测试目标、列项，带 --with-keys 另每项一行 ITEM_PLAN；不编、不跑、不建日志目录。
#   不带 --skip-apply 而给了补丁目录的：临时快照（take_snapshot）、git init、照给的次序逐份打上补丁，在它上面挑、列、算键；补丁打不进退 2。
#   判不出测试目标的逐条打、落得到的照列（带 --with-keys 的照打 ITEM_PLAN），末了退 3。弄坏开关 item-plan-reusable-zero 下 ITEM_PLAN 的 reusable 一律 0；
#   list-items-unpatched 下不拷不打、照要验的树此刻的样子挑；list-items-refusal-drops-items 下有判不出的就一项都不列（select_test_targets）
# list_items_finish：列完收尾：有判不出测试目标的退 3（逐条已打在列项之前），没有退 0
list_items_finish() {
  ((${#TARGET_REFUSALS[@]})) || exit 0
  die "这一趟有 ${#TARGET_REFUSALS[@]} 处判不出改到哪个测试目标（逐条打在列项之前），落得到的 ${#ALL_ITEM_LINES[@]} 项照列在上面" \
    "整包归提交时的整轮门禁；提交之外要整包跑先弹窗问用户。落得到的照上面列的第一列点名（--item <项>）；判不出的那几处照真跑那一趟的出路改补丁（工作区级的改动拆出去、变异行点名 --lib 或 --test <目标>）；用户同意只跑落得到的项、整包留给提交时整轮门禁的，点名落得到的项、加 $UNRESOLVED_OPTION 整条重交" 3
}

list_items_and_exit() {
  local tree_root="$SOURCE_TREE" line item package judged reused dir rc
  if ((skip_apply == 0)) && ((${#CHECK_PATCH_DIRS[@]})) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "list-items-unpatched" ]]; then
    list_items_scratch_directory
    tree_root="$LIST_ITEMS_SCRATCH/repo"
    take_snapshot "$SOURCE_TREE" "$tree_root" > "$LIST_ITEMS_SCRATCH/snapshot.log" 2>&1 \
      || die "列项用的临时快照没拷成（$(tail -n 2 "$LIST_ITEMS_SCRATCH/snapshot.log" | tr '\n' ' ' | clip_text 300)），一项都没列" "看 rsync 在不在、${TMPDIR:-/tmp} 盘满没满，再跑同一条" 1
    git init -q "$tree_root" > "$LIST_ITEMS_SCRATCH/git-init.log" 2>&1 \
      || die "列项用的临时快照 git init 没成（$(tail -n 2 "$LIST_ITEMS_SCRATCH/git-init.log" | tr '\n' ' ' | clip_text 300)），一项都没列" "看 git 在不在、${TMPDIR:-/tmp} 可不可写，再跑同一条" 1
    for dir in "${CHECK_PATCH_DIRS[@]}"; do
      [[ -d "$dir" ]] || die "补丁目录不在：$dir，一项都没列" "给实现员交回的补丁目录（里面有 crates.patch 或 mutations-append.tsv）"
      python3 "$APPLY" "$dir" --root "$tree_root" > "$LIST_ITEMS_SCRATCH/apply.log" 2>&1; rc=$?
      ((rc == 0 || rc == 1)) || die "补丁打不进列项用的临时快照：$dir（apply-writer-patch.py 退 $rc：$(tail -n 3 "$LIST_ITEMS_SCRATCH/apply.log" | tr '\n' ' ' | clip_text 400)），一项都没列" \
        "同真跑那一趟打补丁核不过：补丁照写段打的次序给；三方合并冲突的照真跑那一趟的出路（以主工作区此刻建新段树、冲突那一份退给实现员重做）办完再列" 2
    done
  fi
  VERIFICATION_STAGE="${MERGE_AND_VERIFY_VERIFICATION_STAGE:-$tree_root/$VERIFICATION_STAGE_PATH}"
  verification_member_present "$tree_root" || die "认不出验证包：$MEMBERS_PROBLEM" "同真跑那一句：工作区成员要逐个写出目录、各有 [package] name，验证包 $VERIFICATION_PACKAGE 要在成员里" 1
  select_test_targets "$tree_root" || die "挑不出测试目标：$TARGET_PROBLEM" "看补丁目录在不在、crates.patch、变异行与工作区成员的 Cargo.toml 读不读得了" 1
  for line in ${TARGET_REFUSALS[@]+"${TARGET_REFUSALS[@]}"}; do echo "  ✗ 判不出测试目标：$line"; done   # gate-lint:detail
  collect_test_items
  for line in ${ALL_ITEM_LINES[@]+"${ALL_ITEM_LINES[@]}"}; do
    item="${line%%$'\t'*}"; line="${line#*$'\t'}"
    printf '%s\t%s\n' "$item" "${line#*$'\t'}"
  done
  if ((${#UNKNOWN_ITEMS[@]})); then
    die "--item 点名的项不在这一趟可点的项里：${UNKNOWN_ITEMS[*]}（可点的项是上面列的第一列，一共 ${#ALL_ITEM_LINES[@]} 项）" "项照上面列的第一列写（<包>:<目标根文件> 或 <包>:doc）" 2
  fi
  ((LIST_WITH_KEYS)) || list_items_finish
  list_items_scratch_directory
  PLAN_DIRECTORY="$LIST_ITEMS_SCRATCH/plan"
  mkdir -- "$PLAN_DIRECTORY" || die "在 $LIST_ITEMS_SCRATCH 下建不了放计划的目录" "看 ${TMPDIR:-/tmp} 可不可写、有没有空间" 1
  plan_test_functions "$tree_root" >&2
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    item="${line%%$'\t'*}"; line="${line#*$'\t'}"; package="${line%%$'\t'*}"
    if [[ "$item" == "$package:doc" ]]; then judged=1; reused=0
    elif word_in_list "$package" ${PLANNED_PACKAGES[@]+"${PLANNED_PACKAGES[@]}"}; then
      read -r judged reused < <(awk -F'\t' -v root="${item#"$package":}" 'NF >= 7 && $5 == root { if ($2 == "judge" || $2 == "refused") judge++; else if ($2 == "reuse") reuse++ }
        END { print judge + 0, reuse + 0 }' "$PLAN_DIRECTORY/test-plan.$package.out")
    elif word_in_list "$package" ${TESTED_PACKAGES[@]+"${TESTED_PACKAGES[@]}"}; then
      printf '%s\t按项的计划算不出（这个包的 test-plan 失灵或 prune-server 不可用，上面那一句），不打 ITEM_PLAN\n' "$item"
      continue
    else judged=0; reused=0; fi
    [[ "${MERGE_AND_VERIFY_BREAK:-}" != item-plan-reusable-zero ]] || reused=0
    printf 'ITEM_PLAN item=%s to_judge=%s reusable=%s\n' "$item" "$judged" "$reused"
  done
  list_items_finish
}

# cargo_command_for <子命令> <参数词…>：拼一条 cargo 命令进 CARGO_BUILT_COMMAND：cargo <子命令> --offline 接 check.sh 那一行读来的参数词（读来的 --offline 不重复带）
cargo_command_for() {
  local subcommand="$1" word; shift
  CARGO_BUILT_COMMAND=(cargo "$subcommand" --offline)
  for word in "$@"; do [[ "$word" == --offline ]] || CARGO_BUILT_COMMAND+=("$word"); done
}

# clippy_command_for <特性组合>：照 check.sh 那一行（CARGO_CLIPPY_WORDS）拼这个组合的 clippy 命令进 clippy_command、步名进 clippy_step（都是 verify() 的局部变量；文件头 clippy 一段）。
# 两个占位词换成值：特性参数——组合 default 不带，别的按空白切成词带上；lint 表——verify() 的 clippy_lints
clippy_command_for() {
  local word
  local -a feature_arguments=() clippy_words=()
  [[ "$1" == default ]] || read -r -a feature_arguments <<< "$1"
  clippy_step="$(clippy_step_name "$1")"
  for word in ${CARGO_CLIPPY_WORDS[@]+"${CARGO_CLIPPY_WORDS[@]}"}; do
    case "$word" in
      "$CLIPPY_FEATURES_PLACEHOLDER") clippy_words+=(${feature_arguments[@]+"${feature_arguments[@]}"}) ;;
      "$CLIPPY_LINTS_PLACEHOLDER") clippy_words+=(${clippy_lints[@]+"${clippy_lints[@]}"}) ;;
      *) clippy_words+=("$word") ;;
    esac
  done
  cargo_command_for clippy ${clippy_words[@]+"${clippy_words[@]}"}
  clippy_command=("${CARGO_BUILT_COMMAND[@]}")
}

# clippy_checking_as_compiling <输出文件> <clippy 日志…>：重编核对（lib_rebuilt_packages.py）认「Compiling <包名> v…」行，cargo clippy 重编一个包时打的是「Checking <包名> v…」
# （没重编的包两种都不打）；把这些日志里行首的 Checking 换写成 Compiling 写进输出文件，交给重编核对当这一趟编过的证据（文件头「改过的包都重编了没有」）。
clippy_checking_as_compiling() {
  local output="$1"
  shift
  sed -E 's/^([[:space:]]*)Checking ([^[:space:]]+ v)/\1Compiling \2/' "$@" > "$output"
}

verify() {
  local logs="$1"; shift
  local -a dirs=("$@")
  local dir rc applied=0 warned=0 failed_tests reds=0 test_count combination clippy_step="" word rerun_mode
  MUTATION_TABLES_STATE=""
  local -a test_flags=() clippy_lints=() patch_arguments=() compile_logs=() fmt_command=() clippy_command=() build_command=() clippy_logs=() red_items=()
  local -a clippy_red_combinations=() clippy_green_combinations=() build_packages=()
  mkdir -p "$logs"
  rm -f -- "$logs/test.log" "$logs/build.log" "$logs/clippy-checking-as-compiling.log"   # 重编核对只认这一趟的编译输出（日志目录可能是调用方给的旧目录）
  # 重编核对的输入（文件头「改过的包都重编了没有」）：补丁目录；槽位那一种另加拷进槽时写了或删了的 .rs，带 --skip-apply 的不读补丁目录（文件头「槽位」）。
  #   弄坏开关 slot-changed-sources-unchecked：槽位那一种不读写了或删了的 .rs
  if [[ -z "$SLOT_DIRECTORY" || $skip_apply -eq 0 ]]; then
    for dir in ${CHECK_PATCH_DIRS[@]+"${CHECK_PATCH_DIRS[@]}"}; do patch_arguments+=(--patch-dir "$dir"); done
  fi
  if [[ -n "$SLOT_DIRECTORY" && "${MERGE_AND_VERIFY_BREAK:-}" != "slot-changed-sources-unchecked" && -f "$logs/$SLOT_CHANGED_SOURCES_NAME" ]]; then
    while IFS= read -r word; do [[ -z "$word" ]] || patch_arguments+=(--path "$word"); done < "$logs/$SLOT_CHANGED_SOURCES_NAME"
  fi
  if ! read_check_sh "$CHECK_SH"; then
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "lint-table-optional" ]]; then
      echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=lint-table-optional：$CHECK_SH_PROBLEM，照常往下验"
    else
      die "check.sh 读不出、或与这里对不上：$CHECK_SH_PROBLEM；一个补丁都没打、没编" \
        "上面一句点名了是 check.sh 的哪一处（lint 表、缺省特性组合、clippy 循环的结构行，或 fmt、clippy、build、test 哪一条 cargo 命令的哪一行哪个词），照它在 read_check_sh、read_cargo_command 里补上这一种写法的识别（cargo 命令只换包装函数名、加删标志不用改这里：现读）；先判得上游这一改是想让提交时跑的事变了，还是只换了写法，再补，让合入后验证与提交时的判据仍是同一套" 1
    fi
  fi
  # 弄坏开关 cargo-arguments-fixed：四条 cargo 命令的参数用这里写死的，不用从 check.sh 读来的
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "cargo-arguments-fixed" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=cargo-arguments-fixed：四条 cargo 命令的参数用写死的，不用 check.sh 里读来的"
    CARGO_FMT_WORDS=(--all -- --check); CARGO_BUILD_WORDS=(--all-targets); CARGO_TEST_FLAGS=()
    CARGO_CLIPPY_WORDS=(--all-targets "$CLIPPY_FEATURES_PLACEHOLDER" -- -D warnings "$CLIPPY_LINTS_PLACEHOLDER")
  fi
  # check.sh 认的特性组合登记（文件头 clippy 一段）：这里不读，要验的树里有就判红、不编。弄坏开关 clippy-registration-ignored：照读缺省组合往下验
  if [[ -n "$CLIPPY_REGISTRATION" && -e "$tree/$CLIPPY_REGISTRATION" ]]; then
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "clippy-registration-ignored" ]]; then
      echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=clippy-registration-ignored：要验的树里有特性组合登记 $CLIPPY_REGISTRATION，照读缺省组合往下验"
    else
      die "要验的树里有 check.sh 认的特性组合登记 $tree/$CLIPPY_REGISTRATION：check.sh 只跑登记的组合，这里不读登记，跑的组合会与它不同；一个补丁都没打、没编" \
        "先把登记的读法（check.sh 里 CLIPPY_COMBINATIONS_FILE 那一段）接进这份脚本的 read_check_sh，让合入后验证与提交时跑同一套组合；或删掉那份登记、回到 check.sh 的缺省组合" 1
    fi
  fi
  # 弄坏开关 single-clippy-pass：clippy 只跑 --all-features 一趟
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "single-clippy-pass" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=single-clippy-pass：clippy 只跑 --all-features 一趟，不用 check.sh 的特性组合（${CLIPPY_COMBINATIONS[*]-}）"
    CLIPPY_COMBINATIONS=("--all-features")
  fi
  clippy_lints=(${DISCIPLINE_LINTS[@]+"${DISCIPLINE_LINTS[@]}"})
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "drop-discipline-lint" && ${#clippy_lints[@]} -ge 2 ]]; then
    clippy_lints=("${clippy_lints[@]:0:${#clippy_lints[@]}-2}")
  fi
  # 合进主工作区的那一份补丁目录（文件头「先验后合」）：打补丁之前存下补丁动到的文件在快照里的原样；同一个日志目录里上一趟留下的照删，合过的那一份退 2
  if ((${#dirs[@]})); then
    for dir in "${dirs[@]}"; do [[ -d "$dir" ]] || die "补丁目录不在：$dir" "给实现员交回的补丁目录（里面有 crates.patch 或 mutations-append.tsv）"; done
    if [[ -e "$logs/$MERGE_PATCH_NAME/apply-outcome.tsv" ]]; then
      die "日志目录 $logs 里的 $MERGE_PATCH_NAME 已经合过（有 apply-outcome.tsv），一个补丁都没打、没编" "换一个日志目录（MERGE_AND_VERIFY_LOG_DIRECTORY 不设就是新的 <日期>-<进程号>）" 2
    fi
    rm -rf -- "${logs:?}/$MERGE_BASELINE_NAME" "${logs:?}/$MERGE_BASELINE_PATHS_NAME" "${logs:?}/$MERGE_PATCH_NAME"
    python3 - "$REBUILT" "$tree" "$logs/$MERGE_BASELINE_NAME" "$logs/$MERGE_BASELINE_PATHS_NAME" "${dirs[@]}" <<<"$MERGE_BASELINE_PROGRAM" > "$logs/merge-baseline.log" 2>&1 \
      || die "存不成打补丁之前补丁动到的文件的原样（日志 $logs/merge-baseline.log），一个补丁都没打、没编" "照那份日志修（补丁目录读不读得了、补丁有没有点到树外的路径、日志目录可不可写）再跑" 1
  fi
  for dir in "${dirs[@]}"; do
    [[ -d "$dir" ]] || die "补丁目录不在：$dir" "给实现员交回的补丁目录（里面有 crates.patch 或 mutations-append.tsv）"
    python3 "$APPLY" "$dir" --root "$tree" > "$logs/apply-$(basename "$dir").log" 2>&1; rc=$?
    case "$rc" in
      0) applied=$((applied+1)); echo "  打上：$dir" ;;
      1) applied=$((applied+1)); warned=$((warned+1))
         echo "  ✗ 打上了，但打补丁脚本退 1：$dir（它不在本进程里判 mutation-tables，不该退 1；日志 $logs/apply-$(basename "$dir").log）"
         echo "  → 怎么办：看那份日志里它退 1 的原因（环境里设着它的弄坏开关就去掉），修好之后整趟重跑" ;;
      # 文件头「先验后合」打补丁核不过那一段：三方合并冲突时出路写段树那一句。弄坏开关 apply-refusal-way-out-unchanged：不写
      #   （建新段树的那一条照 segment_tree_command；弄坏开关 segment-tree-command-narrow：只带 --with-test-inputs）
      #   只含拒算表的补丁目录（主 agent 自己从主工作区做的，不是实现员出的、没有段树基线）出路另写：主 agent 从主工作区此刻重做这一份。弄坏开关 refused-patch-way-out-unchanged：照段树那一句写
      *) if refused_tests_only_directory "$dir" && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "refused-patch-way-out-unchanged" ]]; then
           die "补丁核不过、一个字没改：$dir（apply-writer-patch.py 退 $rc，日志 $logs/apply-$(basename "$dir").log），整批停，后面的补丁一份都没打" \
             "这一份是只含拒算表 $REFUSED_TESTS_TABLE 的补丁目录（主 agent 自己从主工作区做的，不是实现员出的，没有段树基线）：主 agent 从主工作区此刻重做这一份（.claude/skills/merge-verify/SKILL.md「合入后验证之后判新出现的拒算」第 1 条：此刻的拒算表拷三份、收的行追加到 b 那一份、git diff --no-index 做成 crates.patch），替掉这一批里原来那一份，整批再交这一条；别的补丁目录照原次序给" 2
         fi
         if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "apply-refusal-way-out-unchanged" ]]; then
           die "补丁核不过、一个字没改：$dir（apply-writer-patch.py 退 $rc，日志 $logs/apply-$(basename "$dir").log）" "按那份日志给的下一步让实现员改补丁再交" 2
         fi
         die "补丁核不过、一个字没改：$dir（apply-writer-patch.py 退 $rc，日志 $logs/apply-$(basename "$dir").log），整批停，后面的补丁一份都没打" \
           "按那份日志给的下一步办。那份日志报三方合并冲突的：补丁照打进段树的次序给、后一份以段树上打过前一份的样子为基线时，几份改同一行是叠着的、照打得上，冲突是主工作区在段树建好之后又改了补丁改的同一段，或同一个基线上各自改同一行的两份；以主工作区此刻建一棵新段树（$(segment_tree_command)），这一份之前的补丁照次序打进去，这一份退给实现员以新段树为基线重做，它之后的照次序再打（打不上的同样重做），新的补丁目录整批再交这一条；别的原因照那份日志的出路让实现员改补丁再交" 2 ;;
    esac
  done
  # 全部打上之后照「原样 → 快照此刻」算合进主工作区的那一份补丁目录（文件头「先验后合」）；做不成记一项红，不打合进主工作区的命令
  if ((${#dirs[@]})); then
    if python3 - "$REBUILT" "$tree" "$logs/$MERGE_BASELINE_NAME" "$logs/$MERGE_BASELINE_PATHS_NAME" "$logs/$MERGE_PATCH_NAME" "${dirs[@]}" <<<"$MERGE_PATCH_PROGRAM" > "$logs/merge-patch.log" 2>&1; then
      echo "  合进主工作区的那一份补丁目录：$logs/$MERGE_PATCH_NAME（$(tail -n 1 "$logs/merge-patch.log")）"
      # 变异锚点核（文件头「先验后合」）：这一批补丁合起来让哪些变异行新红了，红了记一项、不打合进主工作区的命令
      anchor_check_new_breaks "$logs" "$tree" || reds=$((reds+1))
    else
      reds=$((reds+1))
      echo "  ✗ 合进主工作区的那一份补丁目录做不成（$(tail -n 1 "$logs/merge-patch.log")；日志 $logs/merge-patch.log）"
      echo "  → 怎么办：照那份日志修（git 起不起得来、日志目录可不可写、补丁有没有点到树外的路径）再整趟重跑；做不成就不打合进主工作区的命令"
    fi
  fi
  # 带 --skip-apply 的那一趟（补丁已在要验的树上，没有 merge-patch）：变异锚点核照样做，补丁之前的树用倒打出来的（文件头「先验后合」变异锚点核），红了记一项
  if ((skip_apply)); then skip_apply_anchor_check "$logs" "$tree" || reds=$((reds+1)); fi
  if ((applied)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != "mutation-tables-claimed-judged" ]]; then
    MUTATION_TABLES_STATE="code-source-discipline 的 mutation-tables 格本次未跑（打补丁脚本不在本进程里起它）"
    echo "  · $MUTATION_TABLES_STATE；补丁打进主工作区之后另交："
    echo "    SINGLEFS_RUN_SCOPE='种类=items; 跑什么=门禁 code-source-discipline.sh 的 mutation-tables 格; 范围=1 格; 为什么=合入后验证没在本进程里跑这一格，补丁打进主工作区之后补判' task-server/task submit --cwd $(printf '%q' "$SOURCE_TREE") -- bash .claude/gate.d/code-source-discipline.sh --check mutation-tables $(printf '%q' "$SOURCE_TREE")"
  fi
  verification_member_present "$tree" || die "认不出验证包：$MEMBERS_PROBLEM；没编" \
    "工作区成员要逐个写出目录、各有 [package] name，验证包 $VERIFICATION_PACKAGE 要在成员里（这里靠它的名字给它的目标带 --release、排掉登记的崩溃枚举用例）；改了名就同步改这份脚本的 VERIFICATION_PACKAGE" 1
  select_test_targets "$tree" "$logs/$MERGE_BASELINE_NAME" || die "挑不出测试目标：$TARGET_PROBLEM；没编" \
    "看补丁目录在不在、crates.patch、变异行与工作区成员的 Cargo.toml 读不读得了" 1
  report_target_selection
  # 逐项点名（文件头「逐项点名与按条复用」）：认 --item（在打了补丁的快照上认，写错退 2、一条 cargo 都不起）；判不出测试目标的这一趟不跑测试，不认
  collect_test_items
  if ((${#UNKNOWN_ITEMS[@]})) && ((${#TARGET_REFUSALS[@]} == 0)); then
    echo "  这一趟可点的项（${#ALL_ITEM_LINES[@]} 项）："
    for word in ${ALL_ITEM_LINES[@]+"${ALL_ITEM_LINES[@]}"}; do printf '       %s\n' "${word%%$'\t'*}"; done   # gate-lint:detail
    die "--item 点名的项不在这一趟可点的项里：${UNKNOWN_ITEMS[*]}；没编" \
      "项照上面列的写（<包>:<目标根文件> 或 <包>:doc；bash scripts/gate/merge-and-verify.sh --list-items <同一批补丁目录> 也列）" 2
  fi
  # 四步 cargo 只写这一份（clippy 每个特性组合一条），逐条直接跑
  # test 那一行读来的标志里 --offline、--no-fail-fast 这里本来就带，不重复
  for word in ${CARGO_TEST_FLAGS[@]+"${CARGO_TEST_FLAGS[@]}"}; do
    [[ "$word" == --offline || "$word" == --no-fail-fast ]] || test_flags+=("$word")
  done
  fmt_command=(cargo fmt ${CARGO_FMT_WORDS[@]+"${CARGO_FMT_WORDS[@]}"})   # cargo fmt 不认 --offline，不经 cargo_command_for
  # clippy 每个特性组合一条，clippy_command_for 现拼（文件头 clippy 一段）
  # build 那一条的环境里关掉调试信息（文件头 build 一段）：只设在这一条上，test、fmt、clippy 不设。弄坏开关 build-debuginfo-kept：不设
  cargo_command_for build ${CARGO_BUILD_WORDS[@]+"${CARGO_BUILD_WORDS[@]}"}
  build_command=(env CARGO_PROFILE_DEV_DEBUG=0 "${CARGO_BUILT_COMMAND[@]}")
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "build-debuginfo-kept" ]] || build_command=("${CARGO_BUILT_COMMAND[@]}")
  # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：clippy --all-features 编工作区全部成员，连同 验证包（编到 librocksdb-sys；build 点到验证包或依赖它的包时同样编到）。
  #   起四条 cargo 之前判一次这台装了、此刻进程环境的取值对得上（补丁打完之后判：补丁改了 Cargo.lock，键跟着变），判红退 1、一条 cargo 都不起。
  #   树里没有依赖 rocksdb 的清单（自证的样本工作区）、
  #   PATH 上是带标记的假 cargo，entry 报不判。弄坏开关 rocksdb-entry-unchecked 下不判（自证「起 cargo 之前 RocksDB 预编库的取值缺了」那一格判错）
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "rocksdb-entry-unchecked" ]] && ! python3 "$root/scripts/hosts/rocksdb-prebuilt.py" entry --root "$tree"; then
    die "起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一条 cargo 都没起" \
      "照上面的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"）再跑同一条命令" 1
  fi
  # 按条复用（文件头「逐项点名与按条复用」）：跑到的项所在、有测试属性的包各算一份计划（admission.py test-plan 要 cargo -V 与 rustc -vV，放在预编库的取值判过之后）
  PLAN_DIRECTORY="$logs"
  ((${#TARGET_REFUSALS[@]} || ${#RUN_ITEMS[@]} == 0)) || plan_test_functions "$tree"
  # 测试二进制的参数：不带（标 #[ignore] 的只剩要 NVIDIA 卡、量显存与量速度、要补丁表副本那几条，带参数把它们起起来不对），只带按条复用跳过键相同的 --exact --skip <名>…
  #   （每个包各算，run_item_test_commands；弄坏开关 test-arguments-added：在 -- 之后多带一个参数）
  rc=0; test_count=0; failed_tests=""
  if ((${#TARGET_REFUSALS[@]} == 0)) && ((${#RUN_ITEMS[@]})) && ((PLAN_UNAVAILABLE == 0)); then
    echo "  编译并跑挑出的 ${#RUN_ITEMS[@]} 项（$(printf '%s\n' "${RUN_ITEMS[@]}" | cut -f1 | tr '\n' ' ' | sed 's/ $//')$(item_mode && echo "；--item 点名的")；一个包一条；并行度 $THREADS）…"
    run_item_test_commands "$logs" ${test_flags[@]+"${test_flags[@]}"}; rc=$?
    test_count="$(awk 'match($0, /^test result: [A-Za-z]+\. [0-9]+ passed; [0-9]+ failed/) { split($0, f, " "); passed += f[4]; failed += f[6] } END { print passed + failed + 0 }' "$logs/test.log")"
    failed_tests="$(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' "$logs/test.log")"
    # 按条复用照键记库、逐项打 ITEM_DONE（文件头「逐项点名与按条复用」）
    record_test_functions "$logs" "$tree"
    report_items_done "$rc"
  fi
  if ((${#TARGET_REFUSALS[@]})); then
    TESTS_REFUSED=1
    echo "  ✗ 测试那一步与验证包那一步没跑：${#TARGET_REFUSALS[@]} 处判不出改到哪个测试目标（下面逐条），合入后验证不跑整个包"
    for word in "${TARGET_REFUSALS[@]}"; do echo "      $word"; done   # gate-lint:detail
    echo "  → 怎么办：整包归提交时的整轮门禁；提交之外要整包跑先弹窗问用户。补丁本该只改到测试目标的，照上面那几处改（工作区级的改动拆出去、变异行点名 --lib 或 --test <目标>）再交；别的步照跑，整条退 3"
    # 用户同意整包跑的包从上面逐条点名的取；没点名包的几种脚本点不出包（弄坏开关 refusal-packages-unnamed：这一句不打）
    [[ "${MERGE_AND_VERIFY_BREAK:-}" == "refusal-packages-unnamed" ]] \
      || echo "    用户同意整包跑的，点上面逐条点名的包（「<包> 包里的…」「读它的 <包> 的…」「只点了包 <包>」）；没点名包的几种（没给补丁目录、工作区根的文件、不落在任何包里的路径、变异行没点包或点了一类目标）脚本点不出包，整包跑的范围交用户定，弹窗时一并问"
    # 用户同意只跑落得到的项、整包留给提交时的（文件头「判不出的留给提交时整轮门禁」）：带 --unresolved-to-commit-gate 与 --item 整条重交；带了选项而没点 --item 不生效
    if ((LEAVE_UNRESOLVED_TO_COMMIT)) && ! item_mode; then
      echo "    带了 $UNRESOLVED_OPTION 但没点 --item，不生效：点名落得到的项（--list-items 列）再整条重交"
    elif ((LEAVE_UNRESOLVED_TO_COMMIT == 0)); then
      echo "    用户同意只跑落得到的项、整包留给提交时整轮门禁的：点名落得到的项（--list-items 列）、加 $UNRESOLVED_OPTION 整条重交（判不出的那几处逐条打成「留给提交时整轮门禁」，点名的项照跑）"
    fi
  elif ((${#RUN_ITEMS[@]} == 0)); then
    echo "  - 测试本次未跑：这一批补丁没点到要跑的测试目标（要特性的、删掉的见上面挑测试目标那几行）"
  elif ((PLAN_UNAVAILABLE)) && [[ ! -f "$logs/test.log" ]]; then
    echo "  - 测试本次未跑：按条复用那一层连不上 prune-server（上面 admission.py plan-failure 那一句），一条 cargo test 都没起（不记红、不算测过；整条退 75）"
  elif [[ -n "$failed_tests" && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ 红的测试 $(printf '%s\n' "$failed_tests" | wc -l) 条："
    printf '%s\n' "$failed_tests" | sed 's/^/      /'   # gate-lint:detail
    # 红了只重跑红的（文件头「逐项点名与按条复用」）：红的测试函数归到了项的点名那几项，归不到的照旧 -p <包>
    mapfile -t red_items < <(awk -F'\t' '$1 == "FAILED" && $2 != "-" { print $2 }' "$logs/test-sections.tsv" 2>/dev/null | LC_ALL=C sort -u)
    # 不带 --skip-apply 的那一趟（文件头「先验后合」）：补丁不在主工作区上，只打合入后验证那一种，不打对着主工作区起 cargo test 的 rerun-failed-tests.py；
    #   红的归不到项的整批再验点这一趟跑过的全部项（文件头「只重跑的命令点名」）
    rerun_mode=batch; ((${#red_items[@]} == 0)) || rerun_mode=red
    if ! way_out_assumes_applied; then
      echo "  → 怎么办：红的测试函数派 implementation-writer 照实现员报告里的算法改钉值，副本从这一趟留下的快照建、修补以它为基线写（没留快照的照原命令带 env MERGE_AND_VERIFY_KEEP_COPY=1 重交一趟）；修补的补丁目录接在这一条的补丁目录之后再验：$(merge_rerun_command - "" "$rerun_mode" ${red_items[@]+"${red_items[@]}"})（补丁不在主工作区上，照这一趟的形态在快照上打、验；判绿记过库的照复用；修补还改到别的测试目标的去掉 --fingerprint-of，--item 照 --list-items 对全部补丁目录（含修补的）重列、范围写项数）（错误警告日志：$logs/test.problems.log，行前带全量日志里的行号）"
    elif ((${#red_items[@]})); then
      echo "  → 怎么办：只重跑红的这几条：python3 scripts/gate/rerun-failed-tests.py $logs/test.log$(printf ' --item %s' "${red_items[@]}")（红的 ${#red_items[@]} 项）；照实现员报告里的算法改钉值，改完再只跑这几条，或只重跑这几项的合入后验证：$(merge_rerun_command --skip-apply "" red "${red_items[@]}")（判绿记过库的照复用）（错误警告日志：$logs/test.problems.log，行前带全量日志里的行号）"
    else
      echo "  → 怎么办：只重跑红的这几条：python3 scripts/gate/rerun-failed-tests.py $logs/test.log -p <包>；照实现员报告里的算法改钉值，改完再只跑这几条（错误警告日志：$logs/test.problems.log，行前带全量日志里的行号）"
    fi
  elif [[ $rc -ne 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "ignore-red" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 退 $rc 而一条 FAILED 都没抓到（编不过）：$(tail -3 "$logs/test.log" | tr '\n' ' ')"
    echo "  → 怎么办：看 $logs/test.problems.log（错误警告，行前带全量日志里的行号）与 $logs/test.log（全量）"
  elif [[ "$((test_count + PLAN_REUSE))" == 0 && "${MERGE_AND_VERIFY_BREAK:-}" != "zero-tests-green" ]]; then
    reds=$((reds+1))
    echo "  ✗ cargo test 退 0，却一条测试都没跑（test result 行里 passed 与 failed 加起来是 0，按条复用也一条都没复用；日志 $logs/test.log）"
    echo "  → 怎么办：看那份日志里编了哪些包、有没有 test result 行；一条都没跑的「全绿」与跑过了长得一样（check.sh 同一条）"
  else
    echo "  ✓ 测试全绿（$(grep -c '^test result: ok' "$logs/test.log") 个测试二进制，共 $test_count 条测试；跑了 ${#RUN_ITEMS[@]} 项（$(printf '%s\n' "${RUN_ITEMS[@]}" | cut -f2 | LC_ALL=C sort -u | grep -c .) 个包）$(item_mode && echo '，--item 点名的')；按条复用：现判 $PLAN_JUDGE、复用 $PLAN_REUSE、拒算 $PLAN_REFUSED）"
  fi
  # 按条复用失灵（文件头「逐项点名与按条复用」）：失灵的包照旧全部现判跑完了（验证包不起），这一步判红。弄坏开关 plan-failure-ignored 下 PLAN_RED 不置
  if ((PLAN_RED)); then
    reds=$((reds+1))
    echo "  ✗ 测试那一步按条复用失灵（上面 admission.py plan-failure 那一句点名了哪一条哨兵、或 test-plan / verdict-record 退了几）：${FAILED_PLAN_PACKAGES[*]:-记库那一步} 一条都不跳、全部现判跑完了；不能复用就判红，不退回全部现判当绿"
    echo "  → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再重跑同一条命令（键相同的照复用）"
  fi
  run_step "$logs/fmt.log" "${fmt_command[@]}"; rc=$?
  if [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ cargo fmt --check 红（日志 $logs/fmt.log、错误警告 $logs/fmt.problems.log）"; echo "  → 怎么办：cargo fmt --all，再看 diff 只动了格式"; else echo "  ✓ fmt 过"; fi
  # clippy 照 check.sh 的特性组合逐个跑（文件头 clippy 一段）：任一组合红整步红、点名是哪几个组合
  for combination in "${CLIPPY_COMBINATIONS[@]}"; do
    clippy_command_for "$combination"
    run_step "$logs/$clippy_step.log" "${clippy_command[@]}"; rc=$?
    clippy_logs+=("$logs/$clippy_step.log")   # 重编核对也读它（clippy_checking_as_compiling）
    if [[ $rc -ne 0 ]]; then
      echo "  ✗ clippy 特性组合 $combination 红（退 $rc；日志 $logs/$clippy_step.log、错误警告 $logs/$clippy_step.problems.log）"   # gate-lint:detail
      # 弄坏开关 clippy-combination-red-uncounted：照打上面那一行，不记进整步的红
      [[ "${MERGE_AND_VERIFY_BREAK:-}" == "clippy-combination-red-uncounted" ]] || clippy_red_combinations+=("$combination")
    else
      clippy_green_combinations+=("$combination")
    fi
  done
  if ((${#clippy_red_combinations[@]})); then
    reds=$((reds+1))
    echo "  ✗ clippy 红：${#CLIPPY_COMBINATIONS[@]} 个特性组合里 ${#clippy_red_combinations[@]} 个红（${clippy_red_combinations[*]}），日志逐个在上面"
    echo "  → 怎么办：按红的那几个组合的日志逐条改（只在不带特性那一趟红的，多是只在 #[cfg(feature)] 圈起来那一边用到的代码），不加 allow 绕过"
  elif ((${#clippy_green_combinations[@]} == ${#CLIPPY_COMBINATIONS[@]})); then
    echo "  ✓ clippy 过（${#CLIPPY_COMBINATIONS[@]} 个特性组合：${CLIPPY_COMBINATIONS[*]}；每个带 check.sh 的编码纪律 lint $((${#clippy_lints[@]} / 2)) 条）"
  fi
  # build 只编补丁改到的包与工作区里传递依赖它们的包（文件头 build 一段）：包集合照 MAIN_TREE_FINGERPRINT_PROGRAM 的 build-packages 一段在快照上算，
  #   读来的参数之后逐个加 -p <包>；一个包都没有的本次未跑。弄坏开关 skip-build：不跑；build-whole-workspace：不带 -p、照编整个工作区（一个包都没有也编）
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-build" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=skip-build：不跑 cargo build"
  elif [[ "${MERGE_AND_VERIFY_BREAK:-}" == "build-whole-workspace" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=build-whole-workspace：build 不带 -p、照编整个工作区"
    run_step "$logs/build.log" "${build_command[@]}"; rc=$?
    if [[ $rc -ne 0 ]]; then reds=$((reds+1)); echo "  ✗ cargo build --all-targets 红（整个工作区；日志 $logs/build.log、错误警告 $logs/build.problems.log）"; else echo "  ✓ build 过（整个工作区）"; fi
  elif ! python3 - "$REBUILT" "$tree" build-packages ${CHECK_PATCH_DIRS[@]+"${CHECK_PATCH_DIRS[@]}"} <<<"$MAIN_TREE_FINGERPRINT_PROGRAM" \
         > "$logs/build-packages.txt" 2> "$logs/build-packages.log"; then
    reds=$((reds+1))
    echo "  ✗ 算不出 build 要编哪几个包（$(tail -n 1 "$logs/build-packages.log" | clip_text 300)；日志 $logs/build-packages.log），build 没跑"
    echo "  → 怎么办：照那份日志修（补丁目录读不读得了、快照里各包与树根的 Cargo.toml 读不读得了）再整趟重跑"
  else
    mapfile -t build_packages < "$logs/build-packages.txt"
    if ((${#build_packages[@]} == 0)); then
      echo "  - build 本次未跑：这一批补丁没改到工作区里的包（没给补丁目录、只改不落在包里的文件、只改变异表），不记红也不算编过"
    else
      for word in "${build_packages[@]}"; do build_command+=(-p "$word"); done
      run_step "$logs/build.log" "${build_command[@]}"; rc=$?
      if [[ $rc -ne 0 ]]; then
        reds=$((reds+1)); echo "  ✗ cargo build --all-targets 红（编的 ${#build_packages[@]} 个包：${build_packages[*]}；日志 $logs/build.log、错误警告 $logs/build.problems.log）"
        echo "  → 怎么办：从日志里第一条 rustc 报错改起，后面的多半是它的连锁反应（依赖改到的包的包里报的单态化错、链接错，改法落在被改的那个包或调它的地方）"
      else
        echo "  ✓ build 过（编了 ${#build_packages[@]} 个包：${build_packages[*]}）"
      fi
    fi
  fi
  # 改过的包这一趟都重编了没有（文件头）：test、build 与 clippy 三趟的编译输出合起来核；clippy 打的「Checking」行先换写成「Compiling」（clippy_checking_as_compiling）。
  #   弄坏开关 rebuild-evidence-test-only：只读 test 与 build 的编译输出，不读 clippy 的
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "skip-rebuild-check" ]]; then
    echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=skip-rebuild-check：不核改过的包重编了没有"
  elif ((${#patch_arguments[@]} == 0)) && [[ -n "$SLOT_DIRECTORY" ]]; then
    echo "  - 重编核对本次未判：拷进槽时没有内容变了、新加或删掉的 .rs$( ((skip_apply)) && echo '（带 --skip-apply 的不读补丁目录）' || echo '，也没给补丁目录')，算不出改过的包"
  elif ((${#patch_arguments[@]} == 0)); then
    echo "  - 重编核对本次未判：没给补丁目录，算不出改过的包"
  else
    compile_logs=()
    [[ ! -f "$logs/test.log" ]] || compile_logs+=(--log "$logs/test.log")   # 测试本次未跑时没有这一份
    [[ ! -f "$logs/build.log" ]] || compile_logs+=(--log "$logs/build.log")   # build 本次未跑时没有这一份
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "rebuild-evidence-test-only" ]]; then
      echo "  ! 弄坏开关 MERGE_AND_VERIFY_BREAK=rebuild-evidence-test-only：重编核对只读测试与 build 的编译输出，不读 clippy 的"
    elif ((${#clippy_logs[@]})); then
      clippy_checking_as_compiling "$logs/clippy-checking-as-compiling.log" "${clippy_logs[@]}"
      compile_logs+=(--log "$logs/clippy-checking-as-compiling.log")
    fi
    if ((${#compile_logs[@]} == 0)); then
      echo "  - 重编核对本次未判：这一趟测试、build 与 clippy 都没有编译输出（测试本次未跑、build 没跑、clippy 一个特性组合都没跑），没有编译输出可核"
    else
      python3 "$REBUILT" check --tree "$tree" --target-dir "${CARGO_TARGET_DIR:-$tree/target}" "${compile_logs[@]}" "${patch_arguments[@]}" > "$logs/rebuilt.log" 2>&1; rc=$?
      sed 's/^/  /' "$logs/rebuilt.log"
      case "$rc" in
        0|77) ;;
        1) reds=$((reds+1)) ;;
        *) reds=$((reds+1)); echo "  ✗ 重编核对判不了（lib_rebuilt_packages.py 退 $rc，日志 $logs/rebuilt.log）"
           echo "  → 怎么办：照上面那一行修（编译日志在不在、补丁目录读不读得了）再整趟重跑；判不了不算绿" ;;
      esac
    fi
  fi
  # 验证包那一步：编译与测试之后（文件头「验证包那一步」）
  verification_step "$logs" || reds=$((reds+1))
  local applied_note="其中打补丁脚本退 1 的 $warned 个"
  [[ "${MERGE_AND_VERIFY_BREAK:-}" == "mutation-tables-claimed-judged" ]] && applied_note="其中 code-source-discipline 红 $warned 个"
  echo "  汇总：打上 ${applied} 个补丁目录（${applied_note}${MUTATION_TABLES_STATE:+；$MUTATION_TABLES_STATE}），验证红 $reds 项$( ((TESTS_REFUSED)) && echo '，测试那一步与验证包那一步没跑（判不出测试目标，整包归提交时的整轮门禁）')$( ((${#LEFT_TO_COMMIT[@]})) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != unresolved-summary-dropped ]] && echo "，有 ${#LEFT_TO_COMMIT[@]} 处留给提交时整轮门禁（判不出测试目标，点名的项照跑）")${VERIFICATION_STATE:+，验证包那一步 $VERIFICATION_STATE}${VERIFICATION_SCALED:+，缩过域、不算门禁通过（$CRASH_EXPAND_UP_TO_VARIABLE=$CRASH_EXPAND_UP_TO，只往小缩）}$( ((PLAN_UNAVAILABLE)) && echo '，测试那一步按条复用连不上 prune-server（不是红，也不算通过）')，日志在 $logs"
  report_left_to_commit "  留给提交时整轮门禁的 ${#LEFT_TO_COMMIT[@]} 处（抄进调度记录；整轮门禁在提交时判）："
  if [[ $reds -ne 0 || $warned -ne 0 ]]; then return 1; fi
  ((TESTS_REFUSED == 0)) || return 3   # 文件头「退出码」3：判不出测试目标，测试那一步与验证包那一步没跑
  ((PLAN_UNAVAILABLE == 0)) || return 75   # 文件头「退出码」75：按条复用那一层连不上 prune-server，别的没有红
  return 0
}

# way_out_assumes_applied：出路照「补丁已在主工作区上」打（带 --skip-apply 的那一趟）就退 0；不带的那一趟照文件头「先验后合」，退 1。
#   弄坏开关 merge-rerun-assumes-applied：不带的那一趟也退 0
# refused_tests_only_directory <补丁目录>：这份补丁目录只改拒算表（crates.patch 里每个 diff --git 头点的都是它、没有变异表文件）交 0，别的交 1。
#   这是主 agent 自己从主工作区做的那一份（.claude/skills/merge-verify/SKILL.md「合入后验证之后判新出现的拒算」第 1 条），不是实现员出的，也没有段树基线
refused_tests_only_directory() {
  local file_name
  [[ -s "$1/crates.patch" ]] || return 1
  for file_name in mutations-append.tsv mutations-replacements.tsv mutations-delete.txt; do [[ ! -e "$1/$file_name" ]] || return 1; done
  awk -v table="$REFUSED_TESTS_TABLE" '/^diff --git / { found = 1; for (field = 3; field <= 4; field++) { path = $field; sub(/^[ab]\//, "", path); if (path != table) other = 1 } } END { exit (found && !other) ? 0 : 1 }' "$1/crates.patch"
}

way_out_assumes_applied() {
  ((skip_apply)) || [[ "${MERGE_AND_VERIFY_BREAK:-}" == "merge-rerun-assumes-applied" ]]
}

# merge_rerun_command <--skip-apply 或 -> <展开深度上限或空> <点法：red、batch 或 whole-package> [<项…>]：只重跑的合入后验证的整条命令
#   （文件头「逐项点名与按条复用」「先验后合」「只重跑的命令点名」）：范围声明种类 items，补丁目录照给的次序全给。头一个参数是 --skip-apply 的带 --skip-apply（补丁已在主工作区上），
#   是 - 的不带（照这一趟的形态在快照上打、验；带不带由调用处照 way_out_assumes_applied 定）。给了展开深度上限的带 --crash-expand-up-to <它>；这一趟带了 --unresolved-to-commit-gate 而命令点了项的，
#   在 --item 之前带上它（文件头「判不出的留给提交时整轮门禁」）。
#   点法 red：--item 点给的项（红的那几项），范围是项数；batch（红的归不到项，整批再验）：--item 点这一趟跑过的全部项（RUN_ITEMS），范围是项数，这一趟一项都没跑的照 whole-package；
#   whole-package（要罩的那一格点了 --item 就不起：产品 GPU 包的真卡对拍）：不点项、submit 带 --whole-package-confirmed、范围是补丁目录份数、命令后面接那一句出路。
#   不带 --skip-apply 的那一种在 submit -- 之后带 env MERGE_AND_VERIFY_KEEP_COPY=1（验证段第 2 步与修补都要这一趟的快照），点法 red 的末尾另带 --fingerprint-of <这一趟的日志目录>
#   （文件头「开跑那一刻的指纹」）。弄坏开关 rerun-keeps-no-snapshot：两样都不带；rerun-batch-unnamed：batch 不点项、范围写补丁份数、不带 --whole-package-confirmed
merge_rerun_command() {
  local applied="$1" expand="$2" mode="$3" item line text directory_count confirmed=0 why="合入后验证这几项红了，修完只重跑它们：键相同的照复用，只现判红的那几条与改后键变了的"
  shift 3
  local carried=0
  local -a items=("$@") words=(bash "$root/scripts/gate/merge-and-verify.sh")
  if [[ "$mode" == batch ]]; then
    items=()
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "rerun-batch-unnamed" ]]; then
      for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do items+=("${line%%$'\t'*}"); done
      ((${#items[@]})) || mode=whole-package
    fi
    why="合入后验证红了、红的归不到项，修完整批再验这一趟跑过的项：键相同的照复用，只现判红的那几条与改后键变了的"
  fi
  if [[ "$mode" == whole-package ]]; then
    items=(); confirmed=1
    why="合入后验证红在点了项就不起的那一格（产品 GPU 包的真卡对拍），修完不点项整批再验：键相同的照复用，只现判红的那几条与改后键变了的"
  fi
  [[ "$applied" == --skip-apply || "${MERGE_AND_VERIFY_BREAK:-}" == "rerun-keeps-no-snapshot" ]] || carried=1
  ((carried == 0)) || words=(env MERGE_AND_VERIFY_KEEP_COPY=1 "${words[@]}")
  [[ "$applied" != --skip-apply ]] || words+=(--skip-apply)
  [[ -z "$expand" ]] || words+=(--crash-expand-up-to "$expand")
  # 这一趟带了 --unresolved-to-commit-gate 的，只重跑的命令点了项就带上它：不带它判不出测试目标的那几处又退 3、点名的项一条不跑。弄坏开关 unresolved-option-dropped-from-rerun：不带
  if ((${#items[@]} && LEAVE_UNRESOLVED_TO_COMMIT)) && [[ "${MERGE_AND_VERIFY_BREAK:-}" != unresolved-option-dropped-from-rerun ]]; then words+=("$UNRESOLVED_OPTION"); fi
  for item in ${items[@]+"${items[@]}"}; do words+=(--item "$item"); done
  words+=(${CHECK_PATCH_DIRS[@]+"${CHECK_PATCH_DIRS[@]}"})
  if ((carried && ${#items[@]})) && [[ "$mode" == red ]]; then words+=(--fingerprint-of "$logs"); fi
  text="$(submit_command_text "$confirmed" "${words[@]}")"
  if ((${#items[@]})); then
    printf "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=合入后验证%s %s; 范围=%s 项; 为什么=%s' %s" \
      "$([[ "$mode" == batch ]] && echo '这一趟跑过的' || echo '点名的')" "${items[*]}" "${#items[@]}" "$why" "$text"
    return 0
  fi
  directory_count="$(printf '%s\n' ${CHECK_PATCH_DIRS[@]+"${CHECK_PATCH_DIRS[@]}"} | LC_ALL=C sort -u | grep -c .)"
  printf "SINGLEFS_RUN_SCOPE='种类=items; 跑什么=合入后验证 %s 份补丁; 范围=%s 份补丁; 为什么=%s' %s%s" \
    "$directory_count" "$directory_count" "$why" "$text" "$( ((confirmed)) && printf '%s' "$WHOLE_PACKAGE_CONFIRMATION_NOTE")"
}

# verification_merge_rerun_command <交给门禁的参数…>：不带 --skip-apply 的那一趟验证包那一步红了，只重跑的合入后验证（文件头「先验后合」「只重跑的命令点名」）：
#   --item 点这一趟跑到的验证包目标那几项（一项都没有的照整批再验）；红的那一次点了产品 GPU 包的真卡对拍那一格（点了 --item 时那一格不起）或三格整格的不点项、
#   带 --whole-package-confirmed；展开深度缩过的带 --crash-expand-up-to <同一个 n>
verification_merge_rerun_command() {
  local word line rest
  local -a items=()
  for word in "$@"; do
    case "$word" in *"$PRODUCT_GPU_CELL"*) merge_rerun_command - "$CRASH_EXPAND_UP_TO" whole-package; return 0 ;; esac
  done
  for line in ${RUN_ITEMS[@]+"${RUN_ITEMS[@]}"}; do
    rest="${line#*$'\t'}"
    [[ "${rest%%$'\t'*}" == "$VERIFICATION_PACKAGE" ]] && items+=("${line%%$'\t'*}")
  done
  if ((${#items[@]})); then merge_rerun_command - "$CRASH_EXPAND_UP_TO" red "${items[@]}"; else merge_rerun_command - "$CRASH_EXPAND_UP_TO" batch; fi
}

# segment_tree_command：建新段树的那一条（SEGMENT_TREE_COMMAND）；弄坏开关 segment-tree-command-narrow：只带 --with-test-inputs
segment_tree_command() {
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "segment-tree-command-narrow" ]]; then
    printf '%s' 'python3 scripts/hosts/minimal-copy.py <仓根> <新段树> --with-test-inputs'
  else
    printf '%s' "$SEGMENT_TREE_COMMAND"
  fi
}

# merge_anchored_rows_command <路径单子> <变异表>：合进主工作区那一段 ① 与 ③ 共用的那一条命令（文件头「合入前的 --dry-run」）：变异表里第二段（锚着的文件）落在路径单子里的行
#   （# 起头的不算），按字节排序去重；只读、秒级。① 在它后面接 > <日志目录>/anchored-rows-before.txt，③ 在它后面接 | diff -u <那一份> -
merge_anchored_rows_command() {
  printf '%s' "LC_ALL=C awk -F'\\t' 'NR == FNR { touched[\$0]; next } \$0 !~ /^#/ && \$2 in touched' $(printf '%q' "$1") $(printf '%q' "$2") | LC_ALL=C sort -u"
}

# merge_quick_checks_commands <日志目录>：合进主工作区那一段 ④ 的两条只读核（文件头「开跑那一刻的指纹」），排在 ② 合入前的 --dry-run 之后、紧挨着合的那一条：
#   核主工作区自开跑以来没变；这一批有要换掉、删掉的变异行的，再核那几行没被改。开跑那一刻的指纹在不在由调用方先核。
#   弄坏开关 main-tree-check-dropped：不打核主工作区没变的那一条；fingerprint-mutation-rows-dropped：不打核变异行的那一条
merge_quick_checks_commands() {
  local logs="$1" find_words
  local -a quick_roots=()
  echo "    ④ 核主工作区自开跑以来没变、这一批要换掉、删掉的变异行没被改（只读、秒级，排在 ② 之后、紧挨着合的那一条）："
  # 起点照记下的现算同一张单子，与指纹 diff，整条的退出码是 diff 的
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "main-tree-check-dropped" ]]; then
    mapfile -t quick_roots < "$logs/$MAIN_TREE_ROOTS_NAME"
    if ((${#quick_roots[@]})); then
      find_words="$(printf '%q ' find "${quick_roots[@]}" '(' -type d '(' -name target -o -name 'target-*' -o -name __pycache__ ')' -o -name .git ')' -prune -o -type f -print0)"
      printf '    %s| LC_ALL=C sort -z | xargs -0 -r sha256sum | diff -u %s -\n' "$find_words" "$(printf '%q' "$logs/$MAIN_TREE_FINGERPRINT_NAME")"
    else
      echo "    （补丁改到的包与文件开跑那一刻要验的树里一份都没有，不用核主工作区没变）"
    fi
  fi
  # 记下的变异行是空的不打
  if [[ -s "$logs/$MAIN_TREE_MUTATION_ROWS_NAME" && "${MERGE_AND_VERIFY_BREAK:-}" != "fingerprint-mutation-rows-dropped" ]]; then
    printf '    LC_ALL=C grep -xF -f %s %s | LC_ALL=C sort -u | diff -u %s -\n' "$(printf '%q' "$logs/$MAIN_TREE_MUTATION_ROWS_NAME")" "$(printf '%q' "$SOURCE_TREE/crates/mutations.tsv")" "$(printf '%q' "$logs/$MAIN_TREE_MUTATION_ROWS_NAME")"
  fi
  echo "    ④ 退非 0：不合，照原命令去掉 --fingerprint-of、--item 照 --list-items 对这一批补丁目录重列，整批重交，第 2 步在新留下的快照上重验。"
}

# merge_in_commands <日志目录> <verify 的退出码>：不带 --skip-apply 的那一趟验绿（0）或判不出测试目标（3），打验证段第 2 步在快照上起的几条与合进主工作区的整条命令
#   （文件头「先验后合」），这里自己不打、不合：没设 MERGE_AND_VERIFY_KEEP_COPY=1 的只打一句没留快照、带它整条重交；退 3 的先打一句退 3 那一支；
#   第 2 步那几条；合进主工作区那一段照 ①到⑤ 的次序打（文件头「合入前的 --dry-run」）：① 记变异表里锚在这一批补丁改到的文件上的行、② 合入前的 apply-writer-patch.py --dry-run <日志目录>/merge-patch、
#   ③ 照 ① 同一条命令重算再与 ① 比、④ 核主工作区没变与核要换掉删掉的变异行（merge_quick_checks_commands）、⑤ 一条 apply-writer-patch.py <日志目录>/merge-patch，
#   各条退非 0 与 ⑤ 退 2、不是 2、合完各怎么办写在那一条后面。
#   弄坏开关 merge-in-without-kept-snapshot：没留快照的也照打；merge-in-commands-dropped：留了快照的也不打；main-tree-check-dropped：不打核主工作区没变的那一条；
#   fingerprint-mutation-rows-dropped：不打核变异行的那一条；merge-in-dry-run-dropped：不打合入前的 --dry-run 那一条；
#   merge-in-anchored-rows-before-dropped：不打 ①；merge-in-anchored-rows-after-dropped：不打 ③；merge-in-quick-checks-before-dry-run：④ 排在 ① 之前、不在 ② 之后；
#   merge-in-dry-run-way-out-dropped：② 之后不写代价与各结局的出路；merge-in-dry-run-outcomes-collapsed：② 的出路只写点名「变异表有 N 处静态核红」的整批重交，不分「没做成」「判不了」、不是 2 的结局；
#   merge-in-redo-way-out-narrow：⑤ 退 2 等完与三方合并冲突的出路只写重做核的那几条、再交同一条，不从 ① 重做；
#   mutation-names-keep-deleted：打出的变异行名逐份补丁目录收、不减后面的补丁删掉的
merge_in_commands() {
  local logs="$1" verify_rc="$2" text snapshot merge_patch dir table_name rows_command rows_before
  local -a mutation_names=()
  ((${#CHECK_PATCH_DIRS[@]})) || return 0
  if [[ "${MERGE_AND_VERIFY_KEEP_COPY:-}" != 1 && "${MERGE_AND_VERIFY_BREAK:-}" != "merge-in-without-kept-snapshot" ]]; then
    echo "  ! 这一趟没留快照（没设 MERGE_AND_VERIFY_KEEP_COPY=1）：验证段第 2 步要在这一趟留下的快照上跑，没有树可跑，不打合进主工作区的命令"
    echo "  → 怎么办：照原命令在 task-server/task submit -- 之后加 env MERGE_AND_VERIFY_KEEP_COPY=1、整条重交（键相同的照复用）"
    return 0
  fi
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "merge-in-commands-dropped" ]] || return 0
  snapshot="${SNAPSHOT_TREE:-$tree}"; merge_patch="$logs/$MERGE_PATCH_NAME"
  if ((verify_rc == 3)); then
    echo "  ! 这一趟退 3：测试那一步与验证包那一步没跑（判不出测试目标，原因上面逐条列了）；合之前照 .claude/skills/merge-verify/SKILL.md「写与验分两段」验证段第 1 步「退 3」那一支办，没办完不合"
  fi
  # 验证段第 2 步在快照上起的几条（crash-case-check.py、refused-test-registry.py list，改到必红世界表的另加一条 formal-verification）：写法在 lib_merge_verify_formal_tables.py 的 second_step_lines
  python3 "$here/lib_merge_verify_formal_tables.py" second-step "${MERGE_AND_VERIFY_BREAK:-}" "$snapshot" ${FORMAL_ITEMS[@]+"${FORMAL_ITEMS[@]}"}
  # 这一批新写、改写的变异行名：取合进主工作区的那一份补丁目录里的追加与替换（照补丁次序合完之后相对开跑那一刻的表新加、改写的行，后面的补丁删掉的不在里面）。
  #   弄坏开关 mutation-names-keep-deleted：照各份补丁目录里各自的追加与替换逐份收，不减后面的补丁删掉的
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "mutation-names-keep-deleted" ]]; then
    for dir in "${CHECK_PATCH_DIRS[@]}"; do
      for table_name in mutations-append.tsv mutations-replacements.tsv; do
        [[ -f "$dir/$table_name" ]] && mapfile -t -O "${#mutation_names[@]}" mutation_names < <(awk -F'\t' 'NF >= 6 && $1 !~ /^#/ { print $1 }' "$dir/$table_name")
      done
    done
  else
    for table_name in mutations-append.tsv mutations-replacements.tsv; do
      [[ -f "$merge_patch/$table_name" ]] && mapfile -t -O "${#mutation_names[@]}" mutation_names < <(awk -F'\t' 'NF >= 6 && $1 !~ /^#/ { print $1 }' "$merge_patch/$table_name")
    done
  fi
  if ((${#mutation_names[@]})); then
    echo "    这一批新写、改写的变异行 ${#mutation_names[@]} 行（crates-mutation-replay 带 --item 点它们，命令照 .claude/agents/implementation-writer.md 第 1d 步）：$(printf '「%s」' "${mutation_names[@]}")"
  fi
  if [[ ! -d "$merge_patch" ]]; then
    echo "  ! 合进主工作区的那一份补丁目录 $merge_patch 不在：不合；照原命令整批重交"
    return 0
  fi
  if [[ ! -s "$merge_patch/crates.patch" && ! -s "$merge_patch/mutations-append.tsv" && ! -s "$merge_patch/mutations-replacements.tsv" && ! -s "$merge_patch/mutations-delete.txt" ]]; then
    echo "  · 合进主工作区：这一批补丁合起来没有改动（$merge_patch 里没有 crates.patch 与变异表文件），不用合"
    return 0
  fi
  echo "  · 合进主工作区（补丁这时只在快照上验过，主工作区一个字节没动；这一条任务末行 TASK_SERVER_RESULT 是 exit=0$( ((verify_rc == 3)) && echo '（退 3 的照上面那一支办完）')、第 2 步也都绿了才合，收尾删快照或编译目录没成退 1 的先照那一行处置）："
  echo "    在仓根照 ①到⑤ 的次序逐条做，前一条退 0 才做下一条（①③④ 只读、秒级，直接跑；②⑤ 经服务交）；每一步退非 0 怎么办写在那一步的后面："
  # 开跑那一刻的指纹（④ 用）与补丁改到的路径单子（①③ 用）都在才往下打
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "main-tree-check-dropped" && ( ! -f "$logs/$MAIN_TREE_ROOTS_NAME" || ! -f "$logs/$MAIN_TREE_FINGERPRINT_NAME" ) ]]; then
    echo "  ! 开跑那一刻的指纹不在（$logs/$MAIN_TREE_ROOTS_NAME、$logs/$MAIN_TREE_FINGERPRINT_NAME）：核不了主工作区没变，不合；照原命令整批重交"
    return 0
  fi
  [[ -f "$logs/$MERGE_BASELINE_PATHS_NAME" ]] || { echo "  ! 补丁动到的路径单子 $logs/$MERGE_BASELINE_PATHS_NAME 不在：记不了变异表里锚在这一批改到的文件上的行，不合；照原命令整批重交"; return 0; }
  rows_command="$(merge_anchored_rows_command "$logs/$MERGE_BASELINE_PATHS_NAME" "$SOURCE_TREE/crates/mutations.tsv")"
  rows_before="$(printf '%q' "$logs/anchored-rows-before.txt")"
  [[ "${MERGE_AND_VERIFY_BREAK:-}" != "merge-in-quick-checks-before-dry-run" ]] || merge_quick_checks_commands "$logs"
  # ① 记：② 读主工作区的表只有一刻，它跑着的几百秒里别的会话往表里追加、改了锚在这一批改到的文件上的行，② 看不见，③ 拿 ① 记下的比。弄坏开关 merge-in-anchored-rows-before-dropped：不打
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "merge-in-anchored-rows-before-dropped" ]]; then
    echo "    ① 记下此刻变异表里锚在这一批补丁改到的文件上的行（写进日志目录；② 跑着的几百秒里别的会话往表里追加、改了的行，③ 拿它比）："
    echo "    $rows_command > $rows_before"
  fi
  # ② 合入前的 --dry-run（文件头「合入前的 --dry-run」）：变异锚点核与指纹用的是开跑那一刻的快照，开跑之后别的会话改了锚着的文件、往表里追加了点名同一目标的行、改了登记表，合进去之后才断；
  #   这一条在合入那一刻的主工作区上照这一批补丁前后比锚点（两棵树同一刻拷出，static-check 读被判那棵树的登记表），只拦这一批新弄断的，同一份文件里不受影响的追加不拦。
  #   弄坏开关 merge-in-dry-run-dropped：不打；merge-in-dry-run-way-out-dropped：不写代价与各结局的出路；merge-in-dry-run-outcomes-collapsed：出路只写整批重交那一种
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "merge-in-dry-run-dropped" ]]; then
    echo "    ② 合入前的 --dry-run（经服务交）："
    text="$(printf '%q ' "$root/task-server/task" submit -- python3 "$root/scripts/gate/apply-writer-patch.py" --dry-run "$merge_patch")"; echo "    ${text% }"
    if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "merge-in-dry-run-way-out-dropped" ]]; then
      echo "    上一条是合入前对着此刻的主工作区再核一遍变异锚点（补丁之前、补丁之后两棵树同一刻拷出、各跑一次 static-check，多一趟约 400–740 秒、内存峰值约 15 GiB）：退 0 才做 ③。"
      if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "merge-in-dry-run-outcomes-collapsed" ]]; then
        echo "    退 2 并点名「变异表有 N 处静态核红」的，是开跑之后别的会话改了锚着的文件、追加了点名同一目标的行或改了登记表，让这一批新弄断了那几行：不合，以合入那一刻的主工作区为基线整批重交（去掉 --fingerprint-of、--item 照 --list-items 重列；重交那一趟的核照新的树再核）；退 2 的别的原因照下面合的那一条退 2 的办。"
      else
        echo "    ② 各种结局怎么办（合入这一段打的出路以这一段为准，--dry-run 自己拒绝说明里「改好再跑一次」那一句只在合入这一段之外适用）：退 2 并点名「变异表有 N 处静态核红」的，是开跑之后别的会话改了锚着的文件、追加了点名同一目标的行或改了登记表，让这一批新弄断了那几行：不合，以合入那一刻的主工作区为基线整批重交（去掉 --fingerprint-of、--item 照 --list-items 重列；重交那一趟的核照新的树再核）；退 2 写「没做成」「判不了」的，照它点名的原因办（工具的毛病当场报给管它的会话），办完从 ① 重做；退 2 的别的原因（开着的三方轮开工快照、同一个仓有别的在换、三方合并冲突）照 ⑤ 退 2 的办；不是 2 的结局（被停、撞内存上限这类）照原样再交同一条 --dry-run。"
      fi
    fi
  fi
  # ③ 照 ① 同一条命令重算再比：② 跑着的几百秒里表里锚在这一批改到的文件上的行变了，② 没看全，从 ① 重做、不整批重交。弄坏开关 merge-in-anchored-rows-after-dropped：不打
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" != "merge-in-anchored-rows-after-dropped" ]]; then
    echo "    ③ 照 ① 同一条命令重算，与 ① 记下的比（只读、秒级）："
    echo "    $rows_command | diff -u $rows_before -"
    echo "    上一条退非 0：② 跑着的时候别的会话往变异表里追加了或改了锚在这一批改到的文件上的行，② 没看全：不整批重交，从 ① 重做（重新记、再交 ②、再比；表变了，再核一遍就够）。"
  fi
  [[ "${MERGE_AND_VERIFY_BREAK:-}" == "merge-in-quick-checks-before-dry-run" ]] || merge_quick_checks_commands "$logs"
  echo "    ⑤ 合（经服务交）："
  text="$(printf '%q ' "$root/task-server/task" submit -- python3 "$root/scripts/gate/apply-writer-patch.py" "$merge_patch")"; echo "    ${text% }"
  if [[ "${MERGE_AND_VERIFY_BREAK:-}" == "merge-in-redo-way-out-narrow" ]]; then
    echo "    合的那一条退 2（一个字没改）照它拒绝说明里的原因分开办：开着的三方轮开工快照里有这几份文件的，等那一轮判决写完；同一个仓有别的在换的，等它换完；这两样等完重做核的那几条、再交同一条，不重验。"
    echo "    三方合并冲突、核与换之间主工作区被改了的，重做核的那几条：退非 0 照上面整批重交，退 0 再交同一条。走不到的理由那一节行号对不上的，在 $merge_patch/result.md 末尾照拒绝说明点名的 文件:行 补一节「走不到的理由」（理由照各份补丁报告里那一处写）再交同一条。"
    echo "    合的那一条不是 2 的结局（被停、撞内存上限这类）：先交 $root/task-server/task submit -- python3 $root/scripts/gate/apply-writer-patch.py --pending-status，退 3 的照它的出路 --rollback-pending 退干净（或 --finish-pending 换完），退 0 的主工作区没动过，再照上一句办。"
  else
    echo "    ⑤ 退 2（一个字没改）照它拒绝说明里的原因分开办：开着的三方轮开工快照里有这几份文件的，等那一轮判决写完；同一个仓有别的在换的，等它换完；这两样等完从 ① 重做到 ⑤，不重验。"
    echo "    三方合并冲突、核与换之间主工作区被改了的，也从 ① 重做到 ⑤（④ 退非 0 照上面整批重交）。走不到的理由那一节行号对不上的，在 $merge_patch/result.md 末尾照拒绝说明点名的 文件:行 补一节「走不到的理由」（理由照各份补丁报告里那一处写），再从 ① 重做到 ⑤。"
    echo "    ⑤ 不是 2 的结局（被停、撞内存上限这类）：先交 $root/task-server/task submit -- python3 $root/scripts/gate/apply-writer-patch.py --pending-status，退 3 的照它的出路 --rollback-pending 退干净（或 --finish-pending 换完），退 0 的主工作区没动过，再从 ① 重做到 ⑤。"
  fi
  echo "    合完：交 $root/task-server/task submit -- python3 $root/scripts/gate/apply-writer-patch.py --read-outcome $merge_patch/apply-outcome.tsv，退 0 才算合全；再交本次输出里「补丁打进主工作区之后另交」的 mutation-tables 那一条，${SNAPSHOT_RELEASE_TEXT:-删这一趟留下的快照 $snapshot}"
}
