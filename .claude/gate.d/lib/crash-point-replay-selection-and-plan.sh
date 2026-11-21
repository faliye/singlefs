#!/usr/bin/env bash
# 门禁 crash-point-replay 的函数库：格名表的打印与点名、测试函数计划的请求与判定种类、分片计划文件、样本档的标记与服务检查、按条复用计划的读写与计数说明。
# 由 .claude/gate.d/crash-point-replay.sh 在格名表与 preflight 之后、参数解析之前 source，不单独跑；格名表、参数解析与按格跑的主流程在那份入口里。
# 这份里是函数与它们的注释；函数读的格名、路径与状态变量由入口设，在调用时才取，别份库里的函数同样在调用时才找。

# crash_point_replay_print_cells：逐行打「格名、制表符、判什么」，九格，次序同文件头（CRASH_AUDIT_CELL_NAMES 的下标 3、4 是只读树的两格，0、1、2 是读覆盖行的三格）
crash_point_replay_print_cells() {
  local index
  printf '%s\t%s\n' "$CRASH_CASE_REPLAY_CELL_NAME" "$CRASH_CASE_REPLAY_CELL_TITLE" "$PRODUCT_GPU_REAL_CARD_CELL_NAME" "$PRODUCT_GPU_REAL_CARD_CELL_TITLE"
  for index in 3 4; do printf '%s\t%s\n' "${CRASH_AUDIT_CELL_NAMES[index]}" "${CRASH_AUDIT_CELL_TITLES[index]}"; done
  printf '%s\t%s\n' "$CRASH_CASE_PROVENANCE_CELL_NAME" "$CRASH_CASE_PROVENANCE_CELL_TITLE" "$FEATURE_GATED_TESTS_CELL_NAME" "$FEATURE_GATED_TESTS_CELL_TITLE"
  for index in 0 1 2; do printf '%s\t%s\n' "${CRASH_AUDIT_CELL_NAMES[index]}" "${CRASH_AUDIT_CELL_TITLES[index]}"; done
}

# crash_point_replay_cell_in <格名> <格名…>：头一个参数在不在后面那一串里
crash_point_replay_cell_in() {
  local wanted="$1" candidate
  shift
  for candidate in "$@"; do [[ "$candidate" == "$wanted" ]] && return 0; done
  return 1
}

crash_point_replay_check_cells() { # <逗号隔开的格名> [<点名的来源，缺省 --check>]：认格名表里的九格，记进 crash_point_replay_selected_cells；写错、空着退 2 并列出可用的格名
  local requested_cell requested_cells=() selection_source="${2:---check}"
  IFS=, read -r -a requested_cells <<< "$1"
  ((${#requested_cells[@]})) || requested_cells=("")
  for requested_cell in "${requested_cells[@]}"; do
    if [[ -z "$requested_cell" ]] || ! crash_point_replay_cell_in "$requested_cell" "$CRASH_CASE_REPLAY_CELL_NAME" "$FEATURE_GATED_TESTS_CELL_NAME" "$PRODUCT_GPU_REAL_CARD_CELL_NAME" "${CRASH_AUDIT_CELL_NAMES[@]}" "$CRASH_CASE_PROVENANCE_CELL_NAME"; then
      echo "  ✗ crash-point-replay.sh：$selection_source 里有没登记的格名：「$requested_cell」"
      echo "     → 怎么办：格名照下面写（也可以 bash .claude/gate.d/crash-point-replay.sh --list 看）；按崩溃枚举用例点名用 --item <用例名>："
      crash_point_replay_print_cells | sed 's/\t/  /; s/^/       /'
      exit 2
    fi
    crash_point_replay_selected_cells+=("$requested_cell")
  done
}

crash_point_replay_runs_cell() { # <格名>：这一趟跑不跑这一格（没给 --check 九格都跑）
  local selected_cell
  ((${#crash_point_replay_selected_cells[@]})) || return 0
  for selected_cell in "${crash_point_replay_selected_cells[@]}"; do [[ "$selected_cell" == "$1" ]] && return 0; done
  return 1
}

# crash_point_replay_feature_pass_cleared_environment：带特性那一趟 cargo 前面那一串 env -u（文件头「feature-gated-tests 格」：SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 与
#   此刻环境里每个 SINGLEFS_CRASH_AMPLIFICATION_*），放进全局数组 crash_point_replay_feature_cleared_environment；跑那一格与 --list-items --with-keys 算它的 ITEM_PLAN 都取这一份
crash_point_replay_feature_pass_cleared_environment() {
  local environment_name
  crash_point_replay_feature_cleared_environment=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE)
  while IFS= read -r environment_name; do
    if [[ "$environment_name" == SINGLEFS_CRASH_AMPLIFICATION_* ]]; then crash_point_replay_feature_cleared_environment+=(-u "$environment_name"); fi
  done < <(compgen -e)
}

crash_point_replay_test_functions_request() {
  local request_output request_tag request_value
  local -a request_environment=()
  [[ "${GATE_CRASH_POINT_REPLAY_BREAK:-}" == test-functions-request-gate-environment ]] || request_environment=(${crash_point_replay_test_functions_environment[@]+"${crash_point_replay_test_functions_environment[@]}"})
  crash_point_replay_test_functions_kind=""; crash_point_replay_test_functions_arguments=()
  if request_output="$(env ${request_environment[@]+"${request_environment[@]}"} python3 "$2/scripts/gate/admission.py" test-functions-request "$1" "$2" 2>&1)"; then
    while IFS=$'\t' read -r request_tag request_value; do
      case "$request_tag" in
        kind) crash_point_replay_test_functions_kind="$request_value" ;;
        argument) crash_point_replay_test_functions_arguments+=("$request_value") ;;
      esac
    done <<<"$request_output"
  fi
  if [[ -z "$crash_point_replay_test_functions_kind" ]] || ((${#crash_point_replay_test_functions_arguments[@]} == 0)); then
    crash_point_replay_test_functions_kind="$1:test-functions-request-unreadable"
    crash_point_replay_test_functions_arguments=(--token "test-functions-request-unreadable:$(tr '\n\t' '  ' <<<"$request_output" | cut -c1-200)")
    return 1
  fi
  return 0
}

# crash_point_replay_plan_tokens <判法根> [<GPU 开关的值>]：每条测试函数的判定请求另加的项（文件头「复用」）放进全局数组 crash_point_replay_plan_token_arguments：
# 取 crash_point_replay_test_functions_request 交的（release 档、判法单元 lib_libtest_result_lines.py 的 recordable_results、判法根）。GPU 开没开不进键；
# 弄坏开关 gpu-switch-in-key 下另加 line:enable-gpu=<第二个参数>
crash_point_replay_plan_tokens() {
  crash_point_replay_test_functions_request "$(basename "$0")" "$1"
  crash_point_replay_plan_token_arguments=("${crash_point_replay_test_functions_arguments[@]}")
  if [[ "${GATE_CRASH_POINT_REPLAY_BREAK:-}" == gpu-switch-in-key ]]; then crash_point_replay_plan_token_arguments+=(--token "line:enable-gpu=${2:-0}"); fi
  return 0
}

# crash_point_replay_verdict_kind <阶段文件名> <格名>：这一格按条复用记进、读的那一类判定库（文件头「复用」：几趟同一类、同一份库；种类取 crash_point_replay_test_functions_request 交的，
# 判法根取本阶段所在的仓）；弄坏开关 library-per-pass 下一格一类（<阶段文件名>:<格名>）
crash_point_replay_verdict_kind() {
  if [[ "${GATE_CRASH_POINT_REPLAY_BREAK:-}" == library-per-pass ]]; then printf '%s:%s' "$1" "$2"; return 0; fi
  crash_point_replay_test_functions_request "$1" "${crash_point_replay_stage_repository:-${crash_point_replay_list_repository:-$(cd "$(dirname "$0")/../.." && pwd)}}"
  printf '%s' "$crash_point_replay_test_functions_kind"
}

# crash_point_replay_list_targets <子命令 feature-gated|product-gpu> <参数…>：两张挂特性的测试目标表（下面两个函数）都由 scripts/gate/lib_feature_gated_targets.py 列
# （测试目标的认法：tests/<名>.rs 一个文件；目录式 tests/<名>/main.rs 连同它 mod 声明到的子模块，在 scripts/gate/lib_test_target_sources.py）。
# 弄坏开关 directory-target-listing-unseen 下加 --top-level-only：只认 tests/ 顶层的 .rs，目录式测试目标当不存在。
crash_point_replay_list_targets() {
  local -a top_level_only=()
  [[ "${GATE_CRASH_POINT_REPLAY_BREAK:-}" != directory-target-listing-unseen ]] || top_level_only=(--top-level-only)
  python3 "${crash_point_replay_stage_repository:-$(cd "$(dirname "$0")/../.." && pwd)}/scripts/gate/lib_feature_gated_targets.py" "$@" ${top_level_only[@]+"${top_level_only[@]}"}
}

# list_feature_gated_test_files <tests 目录>：每个测试目标里 #[cfg(…)] / #![cfg(…)] 点到的特性，一个测试目标一行「根文件名<TAB>whole-file|items<TAB>特性,特性」
# （根文件名：平铺的是 <名>.rs，目录式的是 <名>/main.rs；根文件里有 #![cfg(…)] 点到特性的记 whole-file，子模块里的 #![cfg(…)] 只管那个子模块、不算），
# 照根文件名排序；挂特性的一个都没有时不打。括号按层数配对（rustfmt 把长的 cfg 折成几行也认）；// 起头的行（注释与文档注释）不读。
# 写在参数解析之前：--list-items 列 feature-gated 的项（文件头「逐项点名」）也读它
list_feature_gated_test_files() { crash_point_replay_list_targets feature-gated "$1"; }

# crash_point_replay_name_item <项>：--item 的值按前缀分进两类（文件头「逐项点名」）：feature-gated: 起头的进 crash_point_replay_named_feature_items，别的当崩溃枚举用例
crash_point_replay_name_item() {
  if [[ "$1" == "$FEATURE_GATED_ITEM_PREFIX"* ]]; then crash_point_replay_named_feature_items+=("$1"); else crash_point_replay_named_cases+=("crash-case:${1#crash-case:}"); fi
}

# crash_point_replay_feature_items_of <tests 目录>：feature-gated 的可点的项，一项一行「项<TAB>说明」（库那一项在前；list_feature_gated_test_files 读不出返回 1）
crash_point_replay_feature_items_of() {
  local listing
  listing="$(list_feature_gated_test_files "$1")" || return 1
  [[ -n "$listing" ]] || return 0
  printf '%s\t--lib\n' "$FEATURE_GATED_LIBRARY_ITEM"
  awk -F'\t' -v prefix="$FEATURE_GATED_ITEM_PREFIX" '{ printf "%s%s\t%s %s\n", prefix, $1, $2, $3 }' <<<"$listing"
}

# crash_point_replay_shard_refuse <卡在哪> <下一步> <退出码>
crash_point_replay_shard_refuse() {
  echo "  ✗ 分片的 ${crash_point_replay_shard_phase} 那一步：$1"
  echo "     → 怎么办：$2"
  exit "$3"
}

# crash_point_replay_write_shard_plan <片数> <一句说明>：往计划目录写 plan.json（每片只有 label）与 shards（片数一行），退 0
crash_point_replay_write_shard_plan() {
  [[ -n "$crash_point_replay_shard_plan_directory" ]] \
    || crash_point_replay_shard_refuse "环境里没有 TASK_SERVER_SHARD_PLAN_DIRECTORY" "这一步只由 task-server 照分片协议起（task-server/task_server_lib.py 文件头「分片」）" 2
  mkdir -p -- "$crash_point_replay_shard_plan_directory" \
    || crash_point_replay_shard_refuse "建不了计划目录 $crash_point_replay_shard_plan_directory" "看共享工作根可不可写" 78
  python3 - "$crash_point_replay_shard_plan_directory" "$1" "$2" <<'PY_SHARD_PLAN' || crash_point_replay_shard_refuse "写不成 plan.json（上面是 python 的报错）" "看计划目录 $crash_point_replay_shard_plan_directory 可不可写" 1
import json, os, sys
directory, count, note = sys.argv[1], int(sys.argv[2]), sys.argv[3]
shards = [{"label": f"崩溃点重放第 {index} 片（共 {count} 片，SINGLEFS_CRASH_AMPLIFICATION_SHARE={index}/{count}）"} for index in range(count)]
temporary = os.path.join(directory, "plan.json.partial")
with open(temporary, "w", encoding="utf-8") as handle:
    json.dump({"shards": shards, "note": note}, handle, ensure_ascii=False)
os.replace(temporary, os.path.join(directory, "plan.json"))
with open(os.path.join(directory, "shards"), "w", encoding="utf-8") as handle:
    handle.write(f"{count}\n")
PY_SHARD_PLAN
  echo "  ✓ 分片的计划：$1 片（$2），写进 $crash_point_replay_shard_plan_directory"
  crash_point_replay_shard_plan_written=1
  if [[ "${crash_point_replay_sample_mode:-0}" == 1 ]]; then
    echo "  ! 样本档（.crash-point-replay-sample-tools，假 cargo）：样本根的 .gate-shard-phase 只演计划那一步，写完计划退 3 不退 0——这一趟没跑过真的崩溃点重放"
    exit 3
  fi
  exit 0
}

# crash_point_replay_sample_service_check：样本档只在临时 prune-server 底下判（文件头「样本档」）：不在就打原因与带包装的整条、退 2，一份样本都不判。
#   弄坏开关 sample-service-unchecked 下不核
crash_point_replay_sample_service_check() {
  [[ "$crash_point_replay_break" == sample-service-unchecked ]] && return 0
  local service_problem
  if ! service_problem="$(python3 -c 'import os, sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem({} if os.path.isfile(sys.argv[2]) else None))' \
        "$crash_point_replay_stage_repository/scripts/prune-server" "$ROOT/.sample-environment-bare" 2>&1)" || [[ -n "$service_problem" ]]; then
    echo "  ✗ 判的根是样本（$ROOT），样本档不在临时 prune-server 底下，样本的判定会写进真服务、与真门禁同一个种类：${service_problem:-读不了 scripts/prune-server/prune_server_client.py}；一份样本都没判"
    echo "     → 怎么办：样本档整条套临时服务起（交任务服务时前面加 task-server/task submit --）：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash .claude/singlefs-ai-sop/scripts/stage-selftest.sh .claude/gate.d --item crash-point-replay.sh"
    echo "       单跑某一份样本根时，把 bash .claude/gate.d/crash-point-replay.sh <样本根> 接在同一个包装的 -- 后面。"
    exit 2
  fi
}

# crash_point_replay_apply_sample_marker：样本根的 .gate-cells 点名的格记进 crash_point_replay_selected_cells（文件头「样本只跑点名的格」）；
#   本阶段所在的仓（共用库的 STAGE_CELLS_STAGE_REPOSITORY）取 source 共用库时按本阶段的路径算的那一个
crash_point_replay_apply_sample_marker() {
  local marker_line marked_cells=()
  ((${#crash_point_replay_selected_cells[@]})) && return 0
  [[ -f .gate-cells ]] || return 0
  [[ "$crash_point_replay_break" == sample-marker-ignored ]] && return 0
  if [[ "$(pwd -P)" == "$STAGE_CELLS_STAGE_REPOSITORY" || -n "${GATE_STAGED_FROM:-}" ]]; then
    echo "  · $ROOT 里有 .gate-cells，但这里是本阶段所在的仓或 --staged 的临时树，不认它，照跑九格"
    return 0
  fi
  while IFS= read -r marker_line || [[ -n "$marker_line" ]]; do
    marker_line="${marker_line%%#*}"; marker_line="${marker_line//[[:space:]]/}"
    [[ -n "$marker_line" ]] && marked_cells+=("$marker_line")
  done < .gate-cells
  if ((${#marked_cells[@]} == 0)); then
    echo "  ✗ crash-point-replay.sh：样本根 $ROOT 的 .gate-cells 一个格名都没点"
    echo "     → 怎么办：一行写一个格名，可用的格名照下面写（也可以 bash .claude/gate.d/crash-point-replay.sh --list 看）："
    crash_point_replay_print_cells | sed 's/\t/  /; s/^/       /'
    exit 2
  fi
  crash_point_replay_check_cells "$(IFS=,; printf '%s' "${marked_cells[*]}")" "样本根 $ROOT 的 .gate-cells"
  echo "  · 样本只跑 .gate-cells 点名的格：${crash_point_replay_selected_cells[*]}（$ROOT 是样本，不是本阶段所在的仓）"
}

# crash_point_replay_apply_sample_items：样本根的 .gate-items 当作命令行上的 --item（文件头「逐项点名」）：命令行没给 --item、被判的根是样本时才认
crash_point_replay_apply_sample_items() {
  local item_line marked_items=()
  (( ${#crash_point_replay_named_cases[@]} + ${#crash_point_replay_named_feature_items[@]} )) && return 0
  [[ -f .gate-items ]] || return 0
  if [[ "$(pwd -P)" == "$STAGE_CELLS_STAGE_REPOSITORY" || -n "${GATE_STAGED_FROM:-}" ]]; then
    echo "  · $ROOT 里有 .gate-items，但这里是本阶段所在的仓或 --staged 的临时树，不认它"
    return 0
  fi
  while IFS= read -r item_line || [[ -n "$item_line" ]]; do
    item_line="${item_line%%#*}"; item_line="${item_line//[[:space:]]/}"
    [[ -n "$item_line" ]] && marked_items+=("$item_line")
  done < .gate-items
  ((${#marked_items[@]})) || return 0
  for item_line in "${marked_items[@]}"; do crash_point_replay_name_item "$item_line"; done
  crash_point_replay_range_given=1
  echo "  · 样本点名 .gate-items 里的项：${marked_items[*]}（$ROOT 是样本，不是本阶段所在的仓）"
}

# crash_point_replay_apply_sample_crash_cases_only：样本根的 .gate-crash-cases-only（内容不读）当作命令行上的 --crash-cases-only（文件头「--crash-cases-only」）：
#   命令行没给、被判的根是样本时才认
crash_point_replay_apply_sample_crash_cases_only() {
  (( crash_point_replay_crash_cases_only )) && return 0
  [[ -f .gate-crash-cases-only ]] || return 0
  if [[ "$(pwd -P)" == "$STAGE_CELLS_STAGE_REPOSITORY" || -n "${GATE_STAGED_FROM:-}" ]]; then
    echo "  · $ROOT 里有 .gate-crash-cases-only，但这里是本阶段所在的仓或 --staged 的临时树，不认它"
    return 0
  fi
  crash_point_replay_crash_cases_only=1
  echo "  · 样本带 .gate-crash-cases-only：当作 --crash-cases-only（$ROOT 是样本，不是本阶段所在的仓）"
}

# crash_point_replay_apply_sample_shard_phase：样本根的 .gate-shard-phase（内容只认 plan）当作服务交的 TASK_SERVER_SHARD_PHASE=plan，计划目录取样本根下的 .gate-shard-plan-output/
#   （文件头「样本档」）：环境里没有分片阶段、被判的根是样本时才认。计划那一步照服务交的那样走：算 test-plan、预先编测试二进制、定片数、写 plan.json
crash_point_replay_apply_sample_shard_phase() {
  local marked
  [[ -z "$crash_point_replay_shard_phase" && -f .gate-shard-phase ]] || return 0
  if [[ "$crash_point_replay_sample_mode" != 1 || "$(pwd -P)" == "$STAGE_CELLS_STAGE_REPOSITORY" || -n "${GATE_STAGED_FROM:-}" ]]; then
    echo "  · $ROOT 里有 .gate-shard-phase，但这里不是样本根（本阶段所在的仓、--staged 的临时树，或没有 .crash-point-replay-sample-tools），不认它"
    return 0
  fi
  marked="$(sed 's/#.*//' .gate-shard-phase | tr -d '[:space:]')"   # # 之后与空白不算（同 .gate-items）
  if [[ "$marked" != plan ]]; then
    echo "  ✗ crash-point-replay.sh：样本根 $ROOT 的 .gate-shard-phase 写的是「$marked」，只认 plan"
    echo "     → 怎么办：样本只演服务交的计划那一步，文件里写 plan；各片与合并照没有分片阶段那一路演（样本根的 .gate-shard-count 点名几片）"
    exit 2
  fi
  crash_point_replay_shard_phase=plan
  crash_point_replay_service_shard_phase=plan
  crash_point_replay_shard_plan_directory="$ROOT/.gate-shard-plan-output"
  crash_point_replay_shared_plan_directory="$crash_point_replay_shard_plan_directory/test-plan"
  crash_point_replay_shared_binaries_directory="$crash_point_replay_shard_plan_directory/test-binaries"
  echo "  · 样本带 .gate-shard-phase：当作服务交的 TASK_SERVER_SHARD_PHASE=plan，计划目录 $crash_point_replay_shard_plan_directory（$ROOT 是样本，不是本阶段所在的仓）"
}

# crash_point_replay_skips_package_pass：这一趟 crash-case-replay 格不起按条复用那一趟（带了 --crash-cases-only；弄坏开关 crash-cases-only-ignored 下照起）
crash_point_replay_skips_package_pass() {
  (( crash_point_replay_crash_cases_only )) && [[ "$crash_point_replay_break" != crash-cases-only-ignored ]]
}

# crash_point_replay_drop_shared_binaries：计划那一步放的测试二进制（crash_point_replay_shared_binaries_directory）谁删（文件头「分片」：计划那一步放的测试二进制谁删）：
#   服务交的合并那一步收尾一律删；一片没判绿就收尾（判红、被停、出错）的删（不会再有合并来用它，还没起的兄弟片照计划目录找不到它、退回自己编）；
#   计划那一步没写成 plan.json 就收尾的删；判绿收尾的片不删（合并还要用）。没有分片阶段时这一件不删（样本根 .gate-shard-count 那一份放在草稿目录里，跟着草稿目录删）
crash_point_replay_drop_shared_binaries() {
  [[ -n "$crash_point_replay_shared_binaries_directory" ]] || return 0
  case "$crash_point_replay_service_shard_phase" in
    merge) ;;
    shard) (( crash_point_replay_shard_step_green )) && return 0 ;;
    plan) (( crash_point_replay_shard_plan_written )) && return 0 ;;
    *) return 0 ;;
  esac
  rm -rf -- "$crash_point_replay_shared_binaries_directory" "$crash_point_replay_shared_binaries_directory.partial"
}

# crash_point_replay_exit_trap：这一道的 EXIT trap：删掉计划那一步放的测试二进制里该删的那一份；删掉草稿目录；每一趟 cargo 的整份输出判绿收尾（或样本档、或一份都没写）时删，
# 别的（判红、被 TERM 打断，被杀时 bash 也走这个 trap，那时 $? 读不准，只认判绿收尾前置的那个变量）留着、打一行路径；
# 样本档里再逐份核判红时留下的日志在草稿目录删掉之后还在、内容与留下那一刻逐字节相同，核完删掉（文件头「样本档」）；真仓上留着给人看。不改这一道的退出码
crash_point_replay_exit_trap() {
  crash_point_replay_drop_shared_binaries
  rm -rf -- "${crash_point_replay_scratch_directory:?}"
  if [[ -n "$crash_point_replay_pass_log_directory" && -d "$crash_point_replay_pass_log_directory" ]]; then
    if [[ "$crash_point_replay_sample_mode" == 1 || "$crash_point_replay_pass_logs_disposable" == 1 || -z "$(ls -A -- "$crash_point_replay_pass_log_directory")" ]]; then
      rm -rf -- "${crash_point_replay_pass_log_directory:?}"
    else
      echo "  · 每一趟 cargo 的整份输出留在 ${crash_point_replay_pass_log_directory}/（这一道没判绿收尾：判红、被打断或被杀；看完自己删）"
    fi
  fi
  [[ "$crash_point_replay_sample_mode" == 1 ]] || return 0
  local kept_entry kept_path kept_digest
  for kept_entry in ${crash_point_replay_kept_logs[@]+"${crash_point_replay_kept_logs[@]}"}; do
    kept_path="${kept_entry%%$'\t'*}"
    kept_digest="${kept_entry#*$'\t'}"
    if [[ -f "$kept_path" && "$(sha256sum < "$kept_path" | cut -d' ' -f1)" == "$kept_digest" ]]; then
      echo "  · 样本档：判红时留下的 $kept_path 在这一道的草稿目录删掉之后还在、内容与留下那一刻逐字节相同（sha256 ${kept_digest:0:12}）；核完删掉"
      rm -f -- "$kept_path"
    else
      echo "  ✗ 样本档：判红时留下的 $kept_path 在这一道的草稿目录删掉之后不在了（或内容变了）：出路指的那一份跑完就没了"
      echo "     → 怎么办：判红时把日志拷到 \${TMPDIR:-/tmp} 下（mktemp），出路指拷出来的那一份，不指这一道草稿目录里的"
    fi
  done
}

# crash_point_replay_pass_output <日志>：一趟 cargo 的输出接在它后面（文件头「进度行与逐提交的行」）；要 cargo 自己的退出码照旧取 PIPESTATUS[0]
crash_point_replay_pass_output() {
  if [[ "$crash_point_replay_break" == pass-output-program-in-scratch ]]; then
    local program_file="$crash_point_replay_scratch_directory/pass-output.py" waited
    if compgen -G "$PWD/.crash-point-replay-sample-tools/cases/*.term-parent" > /dev/null; then
      for (( waited = 0; waited < 200; waited++ )); do
        [[ -f "$program_file" ]] || break
        sleep 0.1
      done
    fi
    python3 -B "$program_file" "$1" "$crash_point_replay_break" "$crash_point_replay_bulk_tail_lines" "$crash_point_replay_activity_seconds"
    return
  fi
  python3 -B -c "$crash_point_replay_pass_output_program" "$1" "$crash_point_replay_break" "$crash_point_replay_bulk_tail_lines" "$crash_point_replay_activity_seconds"
}

# report_bulk_lines <格名> <哪一趟> <日志>：这一趟有逐提交的行没进日志时打一行计数、末尾现场在哪、最后一行是什么；一行都没有的不打
report_bulk_lines() {
  local counts="$3.bulk-lines.counts" kept_in_tail=0 last_bulk_line="" lifecycle_lines accounting_lines kept_with_marks
  local -a tail_files=()
  [[ -f "$counts" ]] || return 0
  lifecycle_lines="$(awk -F'\t' '$1 == "GPU_LIFECYCLE" { print $2 }' "$counts")"
  accounting_lines="$(awk -F'\t' '$1 == "CRASH_JUDGE_ACCOUNTING_SLICE" { print $2 }' "$counts")"
  kept_with_marks="$(awk -F'\t' '$1 == "kept-with-marks" { print $2 }' "$counts")"
  (( ${lifecycle_lines:-0} + ${accounting_lines:-0} + ${kept_with_marks:-0} > 0 )) || return 0
  [[ -f "$3.bulk-lines.previous" ]] && tail_files+=("$3.bulk-lines.previous")
  [[ -f "$3.bulk-lines" ]] && tail_files+=("$3.bulk-lines")
  if (( ${#tail_files[@]} > 0 )); then
    kept_in_tail="$(cat -- "${tail_files[@]}" | wc -l)"
    last_bulk_line="$(tail -n 1 -- "${tail_files[-1]}" | cut -c1-300)"
  fi
  echo "  · $1：$2的逐提交的行没进日志：GPU_LIFECYCLE ${lifecycle_lines:-0} 行、CRASH_JUDGE_ACCOUNTING_SLICE ${accounting_lines:-0} 行（夹了别的记号、留在日志里的 ${kept_with_marks:-0} 行）；末尾现场 ${kept_in_tail} 行在 $3.bulk-lines 与它的 .previous；最后一行：${last_bulk_line:-（没留）}"
}



# ── 按条复用（文件头「复用」）：每条测试函数一条判定 ─────────
# plan_tests <种类> <计划文件> <计划输出> [--feature <包>/<特性>…]：admission.py test-plan 算 验证包（局部变量 plan_package 设了就算那个包：product-gpu-real-card-tests 格设成产品 GPU 包）里每条测试函数的键、查库，输出一条一行
#   （<名字>、<reuse|judge|refused|not-compiled>、<键>、<目标号>、<目标根文件>、<libtest 全名>、<原因>）落进 <计划输出>，末行 PLAN 那一句的数放进
#   plan_judge、plan_reuse、plan_refused、plan_not_compiled；算不出（工作区读不出、模块出错）返回 1、原因放进 plan_problem、那一趟的子命令与退出码与输出放进 plan_failure，
#   调用方全部现判、什么都不记，再经 plan_failure_verdict 判按条复用失灵（失灵那一格结论判红）。
#   样本档放了 .unit-verdict-sample/prior/ 的，先拿上一版的树算一遍、照判绿记进这一版的判定库（文件头「样本档」）。
#   弄坏开关 GATE_CRASH_POINT_REPLAY_BREAK=setting-file-whole-closure 下每条测试函数的键另加它所在的整份设点文件（admission.py 的 ADMISSION_BREAK=test-key-with-whole-file）。
plan_tests() {
  local kind="$1" plan_file="$2" plan_output="$3" summary plan_exit
  shift 3
  local -a feature_options=("$@") break_setting=() registered_exclusion=()
  # 登记的崩溃枚举用例与放量分项（不标 #[ignore]，交按块那一路）不进按条复用的计划：不当复用哨兵、不被强制现判、样本档不替它记绿；
  # 只给整包那几趟（plan_package 空着），产品 GPU 包那一格没有登记的用例。弄坏开关 registered-rows-in-plan 下不带
  [[ -z "${plan_package:-}" && "$crash_point_replay_break" != registered-rows-in-plan ]] && registered_exclusion=(--exclude-registered)
  crash_point_replay_plan_tokens "$crash_point_replay_stage_repository" "$crash_point_replay_gpu_configured"
  local -a tokens=("${crash_point_replay_plan_token_arguments[@]}")
  [[ "$crash_point_replay_break" == setting-file-whole-closure ]] && break_setting=("ADMISSION_BREAK=test-key-with-whole-file")
  # 样本档带 .unit-verdict-sample/red-over-green（文件头「样本档」）：库里已有上一版照判绿记进来的同一把键，不带特性那一趟（crash-case-replay 格）的计划不读库、
  #   一律现判（ADMISSION_BREAK=verdict-lookup-skipped，样本根不抽复用哨兵），上一版也只在这一趟记一次；带特性那一趟照常查库，看判红那一趟记下了什么
  local plain_red_over_green=0
  [[ "$crash_point_replay_sample_mode" == 1 && -f "$PWD/.unit-verdict-sample/red-over-green" && ${#feature_options[@]} -eq 0 && -z "${plan_package:-}" ]] \
    && plain_red_over_green=1 && break_setting=("ADMISSION_BREAK=verdict-lookup-skipped")
  plan_judge=0; plan_reuse=0; plan_refused=0; plan_not_compiled=0; plan_problem=""; plan_failure=()
  if [[ "$crash_point_replay_sample_mode" == 1 && -d "$PWD/.unit-verdict-sample/prior" ]] \
       && { ((plain_red_over_green)) || [[ ! -f "$PWD/.unit-verdict-sample/red-over-green" ]]; }; then
    seed_prior_verdicts "$kind" ${feature_options[@]+"${feature_options[@]}"} || return 1
  fi
  if env ${break_setting[@]+"${break_setting[@]}"} python3 "$crash_point_replay_admission_module" test-plan "$ROOT" "$kind" "${plan_package:-singlefs-verification}" "$plan_file" \
       "${tokens[@]}" ${feature_options[@]+"${feature_options[@]}"} ${registered_exclusion[@]+"${registered_exclusion[@]}"} >"$plan_output" 2>"$plan_output.err"; then plan_exit=0; else plan_exit=$?; fi
  if ((plan_exit != 0)); then
    plan_problem="admission.py test-plan 没成：$(tail -n 3 "$plan_output" "$plan_output.err" 2>/dev/null | tr '\n' ' ' | cut -c1-400)"
    plan_failure=(test-plan "$plan_exit" "$plan_output" "$plan_output.err")
    return 1
  fi
  [[ -z "${plan_package:-}" ]] && drop_registered_rows_from_plan "$plan_output"
  crash_point_replay_read_plan_counts "$plan_output" || { plan_problem="admission.py test-plan 没打 PLAN 那一行"; plan_failure=(test-plan 0 "$plan_output" "$plan_output.err"); return 1; }
  return 0
}


# crash_point_replay_read_plan_counts <计划输出>：末行 PLAN 那一句的数放进 plan_judge、plan_reuse、plan_refused、plan_not_compiled；没有 PLAN 那一行交 1
crash_point_replay_read_plan_counts() {
  local summary
  summary="$(grep '^PLAN ' "$1" | tail -n 1)"
  [[ -n "$summary" ]] || return 1
  plan_judge="$(sed -n 's/.* judge=\([0-9]*\).*/\1/p' <<<"$summary")"
  plan_reuse="$(sed -n 's/.* reuse=\([0-9]*\).*/\1/p' <<<"$summary")"
  plan_refused="$(sed -n 's/.* refused=\([0-9]*\).*/\1/p' <<<"$summary")"
  plan_not_compiled="$(sed -n 's/.* not-compiled=\([0-9]*\).*/\1/p' <<<"$summary")"
  return 0
}


# ── 分片的计划那一步算一遍不带特性那一趟的 test-plan、各片读它（文件头「分片」plan、shard）──────────
# 各片起跑前各自起一遍 admission.py test-plan 要读一遍全部源码（第二台经共享盘读几个 GiB），十几片同时读互相拖；键只取决于输入，算一遍就够。
# crash_point_replay_plan_inputs <种类>：不带特性那一趟的 test-plan 的输入，一行一项；计划目录里存的与此刻逐行相同才算同一份输入
crash_point_replay_plan_inputs() {
  crash_point_replay_plan_tokens "$crash_point_replay_stage_repository" "$crash_point_replay_gpu_configured"
  printf 'kind=%s\n' "$1"
  printf 'root=%s\n' "$ROOT"
  printf 'tokens=%s\n' "${crash_point_replay_plan_token_arguments[*]}"
  printf 'break=%s\n' "$crash_point_replay_break"
  printf 'sample=%s\n' "$crash_point_replay_sample_mode"
  printf 'registered-functions=%s\n' "$(sha256sum <"$crash_point_replay_registered_pairs_file" | cut -d' ' -f1)"
}

# crash_point_replay_source_newer_than <标记文件>：验证包的单元键读的那几处（crates、prune-server、Cargo.toml、Cargo.lock、.cargo、.claude/heavy-thread-default，不含 target 目录）
#   有比标记文件新的文件就打第一份的路径、交 0；没有交 1
crash_point_replay_source_newer_than() {
  local newer
  newer="$(find "$ROOT/crates" "$ROOT/prune-server" "$ROOT/Cargo.toml" "$ROOT/Cargo.lock" "$ROOT/.cargo" "$ROOT/.claude/heavy-thread-default" \
    -type f -newer "$1" ! -path '*/target/*' -print -quit 2>/dev/null)"
  [[ -n "$newer" ]] || return 1
  printf '%s\n' "$newer"
}

# crash_point_replay_store_shared_plan：计划那一步（服务交的 plan，与没有分片阶段时自己走的计划）算一遍不带特性那一趟的 test-plan，连同它的输入与算之前打的标记
#   放进 crash_point_replay_shared_plan_directory（服务交的计划目录下，两台同一绝对路径）；带了 --crash-cases-only 的这一趟没有这一趟 test-plan，不算。
#   算不出（test-plan 退非 0、没打 PLAN）或放不进目录只打一行、不判红：各片读不到就自己算，算不出的由各片自己判红。合并那一步不读这一份
crash_point_replay_store_shared_plan() {
  local directory="$crash_point_replay_shared_plan_directory" staging kind plan_file plan_output
  crash_point_replay_skips_package_pass && return 0
  [[ -n "$directory" ]] || return 0
  plan_file="$crash_point_replay_scratch_directory/plain.plan"
  plan_output="$crash_point_replay_scratch_directory/plain.plan.out"
  kind="$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$CRASH_CASE_REPLAY_CELL_NAME")"
  staging="$directory.partial"
  rm -rf -- "$staging"
  if ! mkdir -p -- "$staging"; then
    echo "  ! 计划：建不了 $staging，不放 test-plan 的结果：各片自己算"
    return 0
  fi
  : >"$staging/stamp"
  if ! plan_tests "$kind" "$plan_file" "$plan_output"; then
    echo "  ! 计划：不带特性那一趟的 test-plan 算不出（$plan_problem），不放结果：各片自己算（算不出的各片各自判红）"
    rm -rf -- "$staging"
    return 0
  fi
  if cp -- "$plan_file" "$staging/plain.plan" && cp -- "$plan_output" "$staging/plain.plan.out" && crash_point_replay_plan_inputs "$kind" >"$staging/inputs" \
     && rm -rf -- "$directory" && mv -- "$staging" "$directory"; then
    crash_point_replay_shared_plan_stored=1
    echo "  ✓ 计划：不带特性那一趟的 test-plan 算了一遍（要现判 ${plan_judge}、复用 ${plan_reuse}、拒算 ${plan_refused}、这一组特性下不编 ${plan_not_compiled}），放进 $directory：各片读它、不再各自起 admission.py test-plan（合并那一步照当时的源码现算，不读它）"
  else
    echo "  ! 计划：test-plan 的结果放不进 $directory：各片自己算"
    rm -rf -- "$staging"
  fi
  return 0
}

# crash_point_replay_read_shared_plan <种类> <计划文件> <计划输出>：分片的一片读计划那一步放好的 test-plan，拷成这一片自己的计划文件与计划输出、plan_judge 等照它重读，交 0；
#   计划目录里的不在、读不了、输入与此刻不同（种类、根、请求、登记函数表）、或计划之后验证包读的源码又改了，打一行原因、交 1，调用方自己算（plan_tests）。
#   弄坏开关 shard-plans-itself 下不读（片里仍各自算）
crash_point_replay_read_shared_plan() {
  local kind="$1" plan_file="$2" plan_output="$3" directory="$crash_point_replay_shared_plan_directory" inputs_now changed
  [[ "$crash_point_replay_break" != shard-plans-itself ]] || return 1
  if [[ -z "$directory" ]] || [[ ! -r "$directory/plain.plan" || ! -r "$directory/plain.plan.out" || ! -r "$directory/inputs" || ! -e "$directory/stamp" ]]; then
    echo "  ! 分片：计划目录里没有计划那一步算好的 test-plan（${directory:-没有计划目录}，或读不了）：这一片自己算"
    return 1
  fi
  inputs_now="$crash_point_replay_scratch_directory/plain.plan.inputs"
  crash_point_replay_plan_inputs "$kind" >"$inputs_now"
  if ! cmp -s -- "$inputs_now" "$directory/inputs"; then
    echo "  ! 分片：计划目录里的 test-plan 是另一份输入算的（$directory/inputs 与此刻的 $inputs_now 不同）：这一片自己算"
    return 1
  fi
  if changed="$(crash_point_replay_source_newer_than "$directory/stamp")"; then
    echo "  ! 分片：计划之后源码又改了（$changed 比计划目录里的标记新）：这一片自己算 test-plan"
    return 1
  fi
  if ! cp -- "$directory/plain.plan" "$plan_file" || ! cp -- "$directory/plain.plan.out" "$plan_output"; then
    echo "  ! 分片：读不了计划目录 $directory 里的 test-plan：这一片自己算"
    return 1
  fi
  if ! crash_point_replay_read_plan_counts "$plan_output"; then
    echo "  ! 分片：计划目录里的 test-plan 没有 PLAN 那一行：这一片自己算"
    return 1
  fi
  plan_problem=""; plan_failure=()
  echo "  · 分片：第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片读计划那一步算好的 test-plan（$directory），不再起 admission.py test-plan"
  return 0
}

# crash_point_replay_plan_for_package_pass <种类> <计划文件> <计划输出>：不带特性那一趟要的计划：分片的一片先读计划目录里的（读得到就不再算），读不到现算；
#   合并那一步不读计划目录，照当时的源码现算、打一行说明（核账要的是此刻的键）。弄坏开关 merge-reads-the-shared-plan 下合并也先读计划目录里的
crash_point_replay_plan_for_package_pass() {
  crash_point_replay_shared_plan_read=0
  if [[ "$crash_point_replay_shard_phase" == shard || ( "$crash_point_replay_shard_phase" == merge && "$crash_point_replay_break" == merge-reads-the-shared-plan ) ]] \
     && crash_point_replay_read_shared_plan "$@"; then
    crash_point_replay_shared_plan_read=1
    return 0
  fi
  [[ "$crash_point_replay_shard_phase" != merge ]] || echo "  · 合并：不带特性那一趟的 test-plan 照当时的源码现算（不读计划那一步放好的）"
  plan_tests "$@"
}

# crash_point_replay_sample_plan_count_note <哪一步>：样本档的标记目录里放了假 python3（.crash-point-replay-sample-tools/python3：admission.py test-plan 每起一遍往同目录的 test-plan-calls 追加一行）时，
#   打到这一步结束为止 test-plan 一共起了几遍，样本据此钉「计划一遍、单片不起、合并一遍」；没有假 python3 的样本什么都不打
crash_point_replay_sample_plan_count_note() {
  local calls="$PWD/.crash-point-replay-sample-tools/test-plan-calls" count=0
  [[ "$crash_point_replay_sample_mode" == 1 && -x "$PWD/.crash-point-replay-sample-tools/python3" ]] || return 0
  [[ ! -f "$calls" ]] || count="$(awk 'END { print NR }' "$calls")"
  echo "  · 样本档：到${1}结束为止 admission.py test-plan 起了 ${count} 遍"
}

# crash_point_replay_sample_cargo_count_note <哪一步>：样本档的标记目录里有 cargo-calls（假 cargo 每起一遍 test 往里追加一行收到的参数；假测试二进制转给假 cargo 的那几遍不记）时，
#   打到这一步结束为止 cargo test 一共起了几遍、其中只编不跑（--no-run）几遍，样本据此钉「计划编一回、各片与合并一回都不编」；没有这份文件的样本什么都不打
crash_point_replay_sample_cargo_count_note() {
  local calls="$PWD/.crash-point-replay-sample-tools/cargo-calls" total building
  [[ "$crash_point_replay_sample_mode" == 1 && -f "$calls" ]] || return 0
  total="$(awk 'END { print NR }' "$calls")"
  building="$(awk '/(^| )--no-run( |$)/ { count++ } END { print count + 0 }' "$calls")"
  echo "  · 样本档：到${1}结束为止 cargo test 起了 ${total} 遍（其中只编不跑 ${building} 遍）"
}
