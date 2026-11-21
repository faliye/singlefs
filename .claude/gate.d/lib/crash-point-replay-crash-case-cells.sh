#!/usr/bin/env bash
# 门禁 crash-point-replay 的函数库：被停的认法、跑测试的命令、整包按条复用那一趟的判、登记的崩溃枚举用例逐条按崩溃状态块那一路的跑与报（crash-case-replay 格）。
# 由 .claude/gate.d/crash-point-replay.sh 在格名表与 preflight 之后、参数解析之前 source，不单独跑；格名表、参数解析与按格跑的主流程在那份入口里。
# 这份里是函数与它们的注释；函数读的格名、路径与状态变量由入口设，在调用时才取，别份库里的函数同样在调用时才找。

crash_point_replay_note_stop() {
  local terminated_line=""
  [[ -z "$crash_point_replay_stop_reason" && "$crash_point_replay_break" != stopped-pass-runs-the-rest ]] || return 0
  case "$2" in
    130|137|143) crash_point_replay_stop_reason="$1被停了（cargo 退 $2）"; return 0 ;;
    0) return 0 ;;
  esac
  terminated_line="$(grep -m 1 -E "process didn't exit successfully: .*\(signal: (2, SIGINT|9, SIGKILL|15, SIGTERM)" "$3" 2>/dev/null | sed -n 's/.*(signal: \([^)]*\)).*/\1/p')"
  [[ -z "$terminated_line" ]] || crash_point_replay_stop_reason="$1被停了（测试二进制死于信号 $terminated_line）"
}


# crash_point_replay_stopped_before_cell <格名>：这一趟被停了就报这一格本次未跑、退 0（调用方不起它）；没被停退 1
crash_point_replay_stopped_before_cell() {
  [[ -n "$crash_point_replay_stop_reason" ]] || return 1
  crash_point_replay_report_not_run "$1：这一趟 ${crash_point_replay_stop_reason}，这一格不接着跑；修好或重交之后只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $1"
  return 0
}


# crash_point_replay_announce_cell <格名> <结局>：一格一收尾就打一行（文件头「格的次序」）：后面排着跑得久的目标时，这一格的结局不等整道收尾
crash_point_replay_announce_cell() {
  echo "  · 这一格先出结局：$1：$2"
}


# crash_point_replay_heavy_jobs：每台默认重活线程数（scripts/hosts/heavy-threads.sh 的 heavy_thread_default）打出来；那份脚本不在、取不到打空串
crash_point_replay_heavy_jobs() {
  local library="$crash_point_replay_stage_repository/scripts/hosts/heavy-threads.sh"
  [[ -f "$library" ]] || return 0
  (source "$library" && heavy_thread_default) 2>/dev/null
}


# crash_point_replay_test_runner <数组名> [gpu] [<编好的测试二进制清单>] [<这一片不起的测试目标清单>]：按条复用那一趟起测试用的命令词放进数组（文件头「按条复用两趟按测试目标并行起」）：
# scripts/gate/test-targets-parallel.py --jobs <每台默认重活线程数> [--binaries <清单>] [--drop-sources <不起的目标根文件清单>] --，或照旧 cargo test（取不到线程数或那份脚本、第二个参数是 gpu、样本档而没给清单）
crash_point_replay_test_runner() {
  local -n runner_words="$1"
  local jobs="" library="$crash_point_replay_stage_repository/scripts/hosts/heavy-threads.sh" parallel="$crash_point_replay_stage_repository/scripts/gate/test-targets-parallel.py"
  local binaries="${3:-}" dropped="${4:-}"
  local -a binaries_option=() dropped_option=()
  runner_words=(cargo test)
  [[ "${2:-}" != gpu && -f "$parallel" && -f "$library" ]] || return 0
  [[ "$crash_point_replay_sample_mode" != 1 || -n "$binaries" ]] || return 0
  jobs="$(crash_point_replay_heavy_jobs)"
  [[ "$jobs" =~ ^[1-9][0-9]*$ ]] || return 0
  [[ -z "$binaries" ]] || binaries_option=(--binaries "$binaries")
  [[ -z "$dropped" ]] || dropped_option=(--drop-sources "$dropped")
  runner_words=(python3 "$parallel" --jobs "$jobs" ${binaries_option[@]+"${binaries_option[@]}"} ${dropped_option[@]+"${dropped_option[@]}"} --)
}


# run_checker_package_tests <日志> [<libtest 参数…>]：只跑不标 ignored 的用例（--tests 连 src/bin/ 的内联单测一起带上），键相同的那几条经 -- --exact --skip 跳过
# （参数由调用方从计划里取，skip_arguments_of_plan）；cargo 的整段输出进日志；这一趟不写发现日志，清掉调用方的 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE。
run_checker_package_tests() {
  local log="$1"
  shift
  local -a runner=() sample_runner_setting=() skip_arguments=("$@")
  local dropped_roots=""
  # 分片的一片（文件头「分片」shard）不起别的片领的测试目标：它们的测试函数在这一片的计划里全记 registered、本来就全被 --skip 排掉，起了只是白起一个进程；
  #   test-targets-parallel.py --drop-sources 照清单把它们去掉，--skip 名单随之只收这一片起的目标的（skip_arguments_of_plan 带清单那一支）。起测试的不是 test-targets-parallel.py（退回 cargo test）时
  #   目标一个不去、名单用全部的。弄坏开关 package-pass-other-shards-targets-run 下都不去、不收窄
  if [[ "$crash_point_replay_shard_phase" == shard && -s "$crash_point_replay_other_shard_roots_file" && "$crash_point_replay_break" != package-pass-other-shards-targets-run ]]; then
    dropped_roots="$crash_point_replay_other_shard_roots_file"
  fi
  # 分片的一片交了计划那一步编好的清单（crash_point_replay_settle_package_pass_binaries）就带 --binaries；样本档里 test-targets-parallel.py 的峰值记录与用时表放草稿目录（文件头「按条复用两趟按测试目标并行起」）
  crash_point_replay_test_runner runner "" "$crash_point_replay_package_binaries" "$dropped_roots"
  if [[ -n "$dropped_roots" && "${runner[0]}" == python3 ]]; then
    skip_arguments=(${plan_skip_arguments_this_shard[@]+"${plan_skip_arguments_this_shard[@]}"})
    echo "  · 按条复用：别的片领的 $(grep -c . "$dropped_roots") 个测试目标这一片不起（--drop-sources $dropped_roots）；--skip 名单只收这一片起的目标的 $(( ${#skip_arguments[@]} / 2 )) 个名字（全部 $(( ${#plan_skip_arguments[@]} / 2 )) 个）"
  fi
  [[ "$crash_point_replay_sample_mode" != 1 ]] || sample_runner_setting=("TEST_TARGETS_PARALLEL_PEAKS_DIRECTORY=$crash_point_replay_scratch_directory/test-targets-parallel-peaks"
    "TEST_TARGETS_PARALLEL_DURATIONS=$crash_point_replay_scratch_directory/test-target-durations.tsv")
  env -u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
    ${sample_runner_setting[@]+"${sample_runner_setting[@]}"} \
    "${runner[@]}" "${CRASH_POINT_REPLAY_PACKAGE_PASS_CARGO_ARGUMENTS[@]}" -- --nocapture ${skip_arguments[@]+"${skip_arguments[@]}"} 2>&1 \
    | crash_point_replay_pass_output "$log"
  local pass_exit="${PIPESTATUS[0]}"
  report_bulk_lines "$CRASH_CASE_REPLAY_CELL_NAME" "不带特性那一趟" "$log"
  return "$pass_exit"
}


# read_crash_case_command <键>：准入模块 crash-case-command 交出的这一条的命令与环境（以 NUL 分隔）读进 case_machine_cores、
# case_threads、case_threads_origin（这三个只打进开跑那一行）与数组 case_command。分片的一片（文件头「分片」shard）带 --share <片号>/<片数>，
# 核账那一趟不带（整份）。录入段与判段在同一趟里先后跑（文件头「显卡」）：不带 --stage，这一道不领卡、不往准入模块的环境里设 GPU_CARDS_BY_PRIORITY。
# 交不出、写法不对返回 1，原因放进 case_command_problem。
read_crash_case_command() {
  local command_file="$crash_point_replay_scratch_directory/command.${1#crash-case:}" command_output
  local -a share_option=()
  [[ "$crash_point_replay_shard_phase" == shard ]] && share_option=(--share "$crash_point_replay_shard_index/$crash_point_replay_shard_count")
  # 样本档里准入模块照样本自己的找法找多机配置（样本根的 multi-host.env），不读调用方环境里 SINGLEFS_MULTI_HOST_CONFIG 指的那一份（文件头「多机配置开了 GPU」）
  local -a configuration_unset=()
  if [[ "$crash_point_replay_sample_mode" == 1 && "$crash_point_replay_break" != sample-caller-configuration-kept ]]; then
    configuration_unset=(-u SINGLEFS_MULTI_HOST_CONFIG)
  fi
  if ! env ${configuration_unset[@]+"${configuration_unset[@]}"} python3 "$crash_point_replay_admission_module" crash-case-command "$ROOT" "$1" \
       ${share_option[@]+"${share_option[@]}"} > "$command_file"; then
    case_command_problem="$(tr '\0' ' ' < "$command_file")"
    return 1
  fi
  local -a command_words=()
  mapfile -d '' -t command_words < "$command_file"
  if (( ${#command_words[@]} < 4 )) || [[ ! "${command_words[0]}" =~ ^[1-9][0-9]*$ || ! "${command_words[1]}" =~ ^[1-9][0-9]*$ \
      || ! "${command_words[2]}" =~ ^(explicit|default)$ ]]; then
    command_output="$(tr '\0' ' ' < "$command_file")"
    case_command_problem="准入模块交的不是「核数、线程数、explicit|default、命令…」：${command_output}"
    return 1
  fi
  case_machine_cores="${command_words[0]}"
  case_threads="${command_words[1]}"
  case_threads_origin="${command_words[2]}"
  case_command=("${command_words[@]:3}")
  return 0
}


# crash_point_replay_case_command_on_binary <用例名，不带 crash-case:> <包> <测试目标> <测试二进制>：把 case_command（env … cargo test … -- <测试参数>）改成在包目录里直接起那个测试二进制，
#   env 里的赋值照留，补上 cargo 给测试进程设的 CARGO_MANIFEST_DIR、CARGO_PKG_NAME、CARGO_CRATE_NAME、CARGO_PRIMARY_PACKAGE、CARGO（CARGO 的取法在 crash-amplification-lib.sh 的 crash_amplification_cargo_path）。
#   改写成了交 0；命令认不出形状交 1（case_command 不动，打一行原因）
crash_point_replay_case_command_on_binary() {
  local executable="$4" cargo_path index cargo_at=-1 separator_at=-1
  for index in "${!case_command[@]}"; do
    if (( cargo_at < 0 )); then
      [[ "${case_command[index]}" != cargo ]] || cargo_at="$index"
    elif (( separator_at < 0 )) && [[ "${case_command[index]}" == -- ]]; then
      separator_at="$index"
    fi
  done
  if [[ "${case_command[0]}" != env ]] || (( cargo_at < 1 || separator_at < cargo_at )); then
    echo "  ! crash-case:$1：准入模块交的命令认不出形状（env … cargo … -- …）：不直接起编好的测试二进制，仍经 cargo test 起"
    return 1
  fi
  cargo_path="$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_cargo_path "$ROOT")"
  case_command=(env -C "$ROOT/crates/$2" "${case_command[@]:1:cargo_at-1}"
    "CARGO_MANIFEST_DIR=$ROOT/crates/$2" "CARGO_PKG_NAME=$2" "CARGO_CRATE_NAME=$3" CARGO_PRIMARY_PACKAGE=1 "CARGO=$cargo_path"
    "$executable" "${case_command[@]:separator_at+1}")
  return 0
}


# crash_point_replay_use_shared_case_binary <键> <用例名，不带 crash-case:> <包> <测试目标>：分片的一片与合并那一步（文件头「分片」shard、merge）起一条用例之前（只录那一趟也是），
#   计划那一步放下了这条用例要的测试二进制（crash_point_replay_shared_binaries_directory 的 binaries.tsv 里包、特性、测试目标都对得上的那一行）、文件在、
#   判器键所读的文件此刻都没变（crash_amplification_judging_inputs_unchanged 打 same），就把 case_command 改成直接起它（crash_point_replay_case_command_on_binary）、
#   记进 case_binary_path、打一行，交 0；有一样不成打一行原因（文件头「分片」shard 那几句）、交 1，case_command 不动，这一步自己经 cargo test 编。
#   没有计划那一步放的目录（没有分片阶段时这一件不预先编）交 1、不打。弄坏开关 shards-build-themselves 下一律交 1、不打
crash_point_replay_use_shared_case_binary() {
  local case_key="$1" case_label="$2" step="合并那一步" manifest identity executable key_state key_exit
  [[ -n "$crash_point_replay_shared_binaries_directory" && "$crash_point_replay_break" != shards-build-themselves ]] || return 1
  [[ "$crash_point_replay_shard_phase" != shard ]] || step="第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片"
  manifest="$crash_point_replay_shared_binaries_directory/binaries.tsv"
  if [[ ! -r "$manifest" ]]; then
    echo "  ! $case_key：计划那一步没留下测试二进制（$manifest 不在或读不了）：${step}自己经 cargo test 编"
    return 1
  fi
  identity="$(crash_point_replay_case_binary_identity)"
  executable="$(awk -F'\t' -v identity="$identity" '($1 "\t" $2 "\t" $3) == identity { print $4; exit }' "$manifest")"
  if [[ -z "$executable" ]]; then
    echo "  ! $case_key：计划那一步没留下它的测试二进制（$manifest 里没有「${identity//$'\t'/ }」那一行：那一组只编没成，或日志里没有它的 Executable 行）：${step}自己经 cargo test 编"
    return 1
  fi
  if [[ ! -f "$executable" || ! -x "$executable" ]]; then
    echo "  ! $case_key：计划那一步留下的测试二进制 $executable 不在了：${step}自己经 cargo test 编"
    return 1
  fi
  if key_state="$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_judging_inputs_unchanged "$executable" "$ROOT")"; then key_exit=0; else key_exit=$?; fi
  if (( key_exit == 1 )); then
    echo "  ! $case_key：计划那一步编完之后判器键所读的文件变了（$key_state）：${step}自己经 cargo test 编"
    return 1
  elif (( key_exit != 0 )); then
    echo "  ! $case_key：计划那一步留下的测试二进制，判器键所读的文件变没变核不出（$key_state）：${step}自己经 cargo test 编"
    return 1
  fi
  crash_point_replay_case_command_on_binary "$case_label" "$3" "$4" "$executable" || return 1
  case_binary_path[$case_label]="$executable"
  echo "  · $case_key：${step}起计划那一步编好的测试二进制（判器键所读的文件都没变：$key_state），不经 cargo、不编：$executable"
  return 0
}


# crash_point_replay_case_has_no_card_for_observer_images <用例日志>：那一趟没跑成而日志里是观察者交回的盘取不到卡那一类（文件头「显卡」；
#   认法只在共用库 crash_amplification_failure_kind 的 observer-images-without-a-card 那一类，经 cargo test 出来的 101 只能按那一行认），是退 0。
#   弄坏开关 no-card-for-observer-images-as-red 下一律答不是（那一趟照判红）
crash_point_replay_case_has_no_card_for_observer_images() {
  [[ "$crash_point_replay_break" == no-card-for-observer-images-as-red ]] && return 1
  [[ -f "$1" ]] || return 1
  [[ "$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_failure_kind "$1")" == observer-images-without-a-card ]]
}


# crash_point_replay_observer_images_howto：观察者交回的盘取不到卡那一类的出路（共用库 crash_amplification_observer_images_without_a_card_howto 那一句；取不到照抄要点）
crash_point_replay_observer_images_howto() {
  local said
  said="$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_observer_images_without_a_card_howto)"
  printf '%s\n' "${said:-这条用例的观察者交回了盘、要上卡判，任务不拿卡；等 GPU 批处理服务收这一族待判行}"
}


# run_crash_case <日志> <发现日志> <命令的词…>：环境里设 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE=<发现日志>；输出经 crash_point_replay_pass_output 进日志，进度行边跑边转出来。
run_crash_case() {
  local log_file="$1"
  local -a findings_setting=(SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE="$2")
  if [[ "$crash_point_replay_break" == no-findings-file ]]; then findings_setting=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE); fi
  shift 2
  env "${findings_setting[@]}" "$@" 2>&1 \
    | crash_point_replay_pass_output "$log_file"
  return "${PIPESTATUS[0]}"
}


# report_crash_case_findings <发现日志> <用例日志> <登记了 findings=not-written-by-the-case 就是 0>：逐节打发现表（行格式与节的写法以 crates/singlefs-verification/src/crash.rs 的发现日志为准：
# 字段制表符分，第一个字段是行的种类，其余 key=value）。每一处不对各打一行「问题：…」，有问题返回 1；发现日志不在又不该有时返回 0。
report_crash_case_findings() {
  local findings_file="$1" log_file="$2" findings_not_written_exit="$3" stdout_findings_lines
  stdout_findings_lines="$(grep -c '^CRASH_POINT_REPLAY_FINDINGS ' "$log_file" 2>/dev/null)"
  stdout_findings_lines="${stdout_findings_lines:-0}"
  if [[ ! -s "$findings_file" ]]; then
    if [[ "$stdout_findings_lines" == 0 ]]; then
      echo "    发现日志：没有（$findings_file 不在或是空的，用例日志里也没有 CRASH_POINT_REPLAY_FINDINGS 行：这条用例没走读 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 的枚举入口）"
      return 0
    fi
    if [[ "$findings_not_written_exit" == 0 ]]; then
      echo "    发现日志：没有（$findings_file 不在或是空的；用例日志里有 ${stdout_findings_lines} 行 CRASH_POINT_REPLAY_FINDINGS，这条用例登记了 findings=not-written-by-the-case：环境守卫清掉了门禁给的变量，这一节不判）"
      return 0
    fi
    echo "    发现日志：$findings_file 不在或是空的"
    echo "      问题：用例日志里有 ${stdout_findings_lines} 行 CRASH_POINT_REPLAY_FINDINGS（枚举跑完了 ${stdout_findings_lines} 趟），发现日志却一节都没写：SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 没传进用例"
    return 1
  fi
  echo "    发现日志：$findings_file（用例日志里 CRASH_POINT_REPLAY_FINDINGS ${stdout_findings_lines} 行）"
  awk -F'\t' -v shown_limit=30 -v findings_path="$findings_file" -v break_item="$crash_point_replay_break" '
    function field(name,   position, equals) {
      for (position = 2; position <= NF; position++) {
        equals = index($position, "=")
        if (equals > 0 && substr($position, 1, equals - 1) == name) return substr($position, equals + 1)
      }
      return ""
    }
    function signature_text(tail_name) {
      return "#" field("finding") " pass=" field("pass") " violated=" field("violated") " segment=" field("segment") " publish=" field("publish") " " tail_name
    }
    $1 == "crash_point_replay_findings_begin" {
      sections++
      section_name[sections] = "stream=" field("stream") (field("shard") != "" ? " shard=" field("shard") : "") " states=" field("states")
      next
    }
    sections == 0 { lines_before_first_section++; next }
    $1 == "crash_point_replay_finding" { final_rows[sections, ++final_count[sections]] = signature_text("states=" field("states") " sample_states=" field("sample_states")); next }
    $1 == "crash_point_replay_finding_new" { new_rows[sections, ++new_count[sections]] = signature_text("first_state=" field("first_state")); next }
    $1 == "crash_point_replay_finding_threshold" { next }
    $1 == "crash_point_replay_findings_summary" {
      summary_count[sections]++
      summary_signatures[sections] = field("signatures"); summary_red_states[sections] = field("red_states"); summary_states[sections] = field("states")
      next
    }
    { unknown_kinds[$1]++ }
    function print_rows(kind, section, count,   row, limit) {
      limit = (break_item == "no-truncation") ? count : shown_limit
      for (row = 1; row <= count && row <= limit; row++) print "        " (kind == "final" ? final_rows[section, row] : new_rows[section, row])
      if (count > limit) print "        其余 " (count - limit) " 个见 " findings_path
    }
    END {
      problems = 0
      if (sections == 0) { print "      问题：一行 crash_point_replay_findings_begin 都没有，认不出节"; problems++ }
      if (lines_before_first_section > 0) { print "      问题：第一行 crash_point_replay_findings_begin 之前有 " lines_before_first_section " 行"; problems++ }
      for (kind in unknown_kinds) print "      （认不出的行种类 " kind "：" unknown_kinds[kind] " 行，没判）"
      for (section = 1; section <= sections; section++) {
        if (summary_count[section] == 0) {
          if (break_item == "ignore-unfinished") continue
          print "      第 " section " 节（" section_name[section] "）：这一趟没跑完（没有 crash_point_replay_findings_summary 行）；死之前找到 " (new_count[section] + 0) " 个签名："
          print_rows("new", section, new_count[section] + 0)
          print "      问题：第 " section " 节这一趟没跑完（没有 crash_point_replay_findings_summary 行：还在跑、被杀或 panic 判红）"
          problems++
          continue
        }
        print "      第 " section " 节（" section_name[section] "）：signatures=" summary_signatures[section] " red_states=" summary_red_states[section] " states=" summary_states[section]
        if (final_count[section] + 0 == 0) print "        没有签名"
        print_rows("final", section, final_count[section] + 0)
        if (summary_count[section] > 1) { print "      问题：第 " section " 节有 " summary_count[section] " 行 crash_point_replay_findings_summary"; problems++ }
        if (summary_signatures[section] != ((final_count[section] + 0) "")) {
          print "      问题：第 " section " 节定稿 " (final_count[section] + 0) " 行 crash_point_replay_finding，summary 却说 signatures=" summary_signatures[section]; problems++
        }
        if (break_item != "ignore-red-states" && summary_red_states[section] != "0") {
          print "      问题：第 " section " 节 red_states=" summary_red_states[section] "：有状态判红（不是 0 都判红；读不出数也判红）"; problems++
        }
      }
      exit (problems > 0 ? 1 : 0)
    }
  ' "$findings_file"
}


# registered_test_functions：登记的崩溃枚举用例与放量分项各落到哪个「测试目标<TAB>用例函数」，一行一条（文件头「复用」）：
# 登记的崩溃枚举用例取 admission.py crash-cases 交的那几行（第三、四列），放量分项取 scripts/crash/lib_crash_point_replay_items.py 的 pipeline_items 读 crash-amplification.sh 的 ITEMS
# （放量入口不在被判的根里，就是空）。按条复用那一趟用 --skip 把它们排掉：登记的崩溃枚举用例交按块那一路（run_registered_crash_cases），放量分项这道门禁不跑（归崩溃放量入口）。
# 弄坏开关 amplification-items-not-skipped 下不取放量分项那一半
registered_test_functions() {
  local library="$crash_point_replay_stage_repository/scripts/crash/lib_crash_point_replay_items.py"
  printf '%s\n' "$crash_case_listing" | awk -F'\t' 'NF >= 4 { printf "%s\t%s\n", $3, $4 }'
  [[ "$crash_point_replay_break" == amplification-items-not-skipped || ! -f "$library" ]] && return 0
  python3 - "$library" "$ROOT" <<'PY_AMPLIFICATION_ITEMS'
import importlib.util
import sys

sys.dont_write_bytecode = True
library_path, root = sys.argv[1:3]
specification = importlib.util.spec_from_file_location("lib_crash_point_replay_items_for_the_skip_list", library_path)
library = importlib.util.module_from_spec(specification)
specification.loader.exec_module(library)
for target, function in sorted(library.pipeline_items(root).values()):
    print(f"{target}\t{function}")
PY_AMPLIFICATION_ITEMS
}


# drop_registered_rows_from_plan <计划输出>：登记的崩溃枚举用例与放量分项不在按条复用这一趟里（它们走按块那一路），从 test-plan 的逐条行里去掉：
# 目标根文件是 tests/<测试目标>.rs（目录式测试目标是 tests/<测试目标>/main.rs）、libtest 全名（去掉模块路径）是登记的用例函数的行；末行 PLAN 照剩下的行重数
drop_registered_rows_from_plan() {
  local plan_output="$1" filtered="$1.registered-dropped"
  [[ -s "$crash_point_replay_registered_pairs_file" ]] || return 0
  awk -F'\t' '
    FNR == NR { registered["tests/" $1 ".rs" SUBSEP $2] = registered["tests/" $1 "/main.rs" SUBSEP $2] = 1; next }
    /^PLAN / { plan_line = $0; next }
    NF >= 7 {
      name = $6; sub(/^.*::/, "", name)
      if (($5 SUBSEP name) in registered) next
      count[$2]++
    }
    { print }
    END {
      ignored = ""
      if (match(plan_line, / ignored=[0-9]+/)) ignored = substr(plan_line, RSTART, RLENGTH)
      printf "PLAN judge=%d reuse=%d refused=%d not-compiled=%d%s\n", count["judge"], count["reuse"], count["refused"], count["not-compiled"], ignored
    }' "$crash_point_replay_registered_pairs_file" "$plan_output" >"$filtered" && mv -- "$filtered" "$plan_output"
}


# run_package_test_functions：验证包里全部测试函数按单元键复用（文件头「复用」），登记的崩溃枚举用例与放量分项经 --skip 排掉。判红打出路、返回 1；判绿把现判、复用、拒算与每个测试二进制
# 那一行 test result 的 passed / ignored 加总进 package_pass_report。一条都不用现判就不起 cargo（package_pass_ran=0）。
run_package_test_functions() {
  local pass_log result_lines passed_count ignored_count passed_total=0 ignored_total=0 binary_count=0 plan_ok=1 accounting_ok=0
  local kind package_pass_exit=0
  local -a red_pass_accounting=()
  # 带了 --crash-cases-only（文件头「--crash-cases-only」）：这一趟不起，不算计划、不记库，这一格只走按块那一路
  if crash_point_replay_skips_package_pass; then
    package_pass_ran=0
    crash_point_replay_cell_state="绿（--crash-cases-only：只判了按块那一路，按条复用那一趟没起）"
    echo "  · $CRASH_CASE_REPLAY_CELL_NAME：带了 --crash-cases-only，只走按块那一路（--item 点名的 ${#crash_point_replay_named_cases[@]} 条崩溃枚举用例）；包里按条复用那一趟这一趟不起：不算计划、不起 cargo、不记库"
    return 0
  fi
  kind="$(crash_point_replay_verdict_kind "$crash_point_replay_stage_file_name" "$CRASH_CASE_REPLAY_CELL_NAME")"
  local plan_file="$crash_point_replay_scratch_directory/plain.plan" plan_output="$crash_point_replay_scratch_directory/plain.plan.out"
  local accounting="$crash_point_replay_scratch_directory/plain.accounting"
  pass_log="$crash_point_replay_plain_pass_log"
  package_pass_ran=1
  : >"$pass_log"
  plan_skip_arguments=()
  plan_skip_arguments_this_shard=()
  crash_point_replay_other_shard_roots_file=""
  if crash_point_replay_plan_for_package_pass "$kind" "$plan_file" "$plan_output"; then
    if [[ "$crash_point_replay_shard_phase" == shard ]]; then
      crash_point_replay_keep_shard_rows "$plan_output"
      awk -F'\t' 'NF >= 7 && ($2 == "judge" || $2 == "refused") { print $5 "\t" $6 }' "$plan_output" >>"$crash_point_replay_shard_ran_file"
    fi
    skip_arguments_of_plan "$plan_output"
    # 别的片领的目标这一片不起时的 --skip 名单（run_checker_package_tests 起的是 test-targets-parallel.py 时用它）：不收那几个目标里的名字，别的与上一行相同
    [[ ! -s "$crash_point_replay_other_shard_roots_file" ]] || skip_arguments_of_plan "$plan_output" "$crash_point_replay_other_shard_roots_file" plan_skip_arguments_this_shard
    echo "  · 按条复用：验证包 $((plan_judge + plan_reuse + plan_refused)) 条编得进的测试函数，$(plan_report "$plan_output")；这一组特性下不编 $plan_not_compiled 条；登记的崩溃枚举用例与放量分项 $(grep -c . "$crash_point_replay_registered_pairs_file") 条经 --skip 排掉、走按块那一路"
    plan_listing "$plan_output"
  else
    plan_ok=0
    echo "  ✗ 按条复用算不出（$plan_problem）：分不了片、判不了复用，这一趟不起 cargo，这一格判红"
    ((${#plan_failure[@]})) && { plan_failure_verdict "$CRASH_CASE_REPLAY_CELL_NAME" "${plan_failure[@]}" || package_pass_plan_red=1; }
  fi
  crash_point_replay_plain_plan_ok="$plan_ok"
  # 计划那一步 prune-server 不可用（plan_failure_verdict 记了这一格）：不起 cargo，这一格就此收尾
  if crash_point_replay_unavailable_only "$CRASH_CASE_REPLAY_CELL_NAME"; then package_pass_ran=0; return 1; fi   # 出口 4 那几格读这一趟的行时要（文件头「出口 4 那五格」）
  if (( ! plan_ok )); then
    package_pass_ran=0
    echo "     → 怎么办：照上面那一句修 scripts/gate/admission.py test-plan 点名的那一处（单跑 python3 scripts/gate/admission.py test-plan 看报错），再经 task-server/task submit 交这一道"
    return 1
  fi
  if [[ "$crash_point_replay_shard_phase" != shard ]]; then
    # 核账（文件头「分片」merge）：合并那一步不现判（没有分片阶段时先走单片、再走合并）
    crash_point_replay_package_pass_check "$plan_output" || return 1
  elif (( plan_judge + plan_refused == 0 )); then
    package_pass_ran=0
    package_pass_report="第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片：没有归这一片要现判的测试函数，这一片没起 cargo（复用 $plan_reuse）"
  elif crash_point_replay_settle_package_pass_binaries && run_checker_package_tests "$pass_log" ${plan_skip_arguments[@]+"${plan_skip_arguments[@]}"}; then
    # 上面先认计划那一步编好的清单（一律交 0，结局就是 run_checker_package_tests 的）
    report_pass_outcome "$CRASH_CASE_REPLAY_CELL_NAME" "不带特性那一趟" 0 "$pass_log"
  else
    package_pass_exit=$?
    crash_point_replay_note_stop "$CRASH_CASE_REPLAY_CELL_NAME 不带特性那一趟" "$package_pass_exit" "$pass_log"
    report_pass_outcome "$CRASH_CASE_REPLAY_CELL_NAME" "不带特性那一趟" "$package_pass_exit" "$pass_log"
    tail -40 "$pass_log"
    # --no-fail-fast：一个测试二进制红了其余照跑，红的用例与测试目标在这里一次列全
    grep -E '^test .* FAILED$|^error: test failed, to rerun pass' "$pass_log" | sed 's/^/    /' || true
    (( plan_ok )) && failed_tests_changed_units "$plan_output" "$pass_log"
    report_red_test_targets "$pass_log" "$CRASH_CASE_REPLAY_CELL_NAME" "$crash_point_replay_scratch_directory/red-rows.$CRASH_CASE_REPLAY_CELL_NAME"
    # 同一趟判绿的照键记绿（文件头「复用」）：弄坏开关 plain-pass-red-withholds-green 下照改前不认、不记
    if (( plan_ok )) && [[ "$crash_point_replay_break" != plain-pass-red-withholds-green ]] && account_test_results "$pass_log" "$accounting"; then
      red_pass_accounting=("$accounting")
    fi
    (( plan_ok )) && record_red_tests "$kind" "$plan_file" "$plan_output" "$crash_point_replay_scratch_directory/red-rows.$CRASH_CASE_REPLAY_CELL_NAME" \
      "$CRASH_CASE_REPLAY_CELL_NAME" "cargo test --release -p singlefs-verification --lib --tests（不带特性那一趟）" ${red_pass_accounting[@]+"${red_pass_accounting[@]}"}
    echo "  ✗ 验证包判红（上面是 cargo test 的尾部、全部红的用例与测试目标，「红的」那几行是各自变了的单元；键相同而复用的那几条这一趟没跑）"
    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-verification -- --nocapture（只看一个测试二进制加 --test <名>）；"
    echo "                断言消息里是第一处对不上的计数或违例；改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
    return 1
  fi
  if (( package_pass_ran )); then
    result_lines="$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed; \([0-9]*\) ignored;.*/\1 \2/p' "$pass_log")"
    while read -r passed_count ignored_count; do
      [[ -n "${passed_count:-}" ]] || continue
      passed_total=$((passed_total + passed_count)); ignored_total=$((ignored_total + ignored_count)); binary_count=$((binary_count + 1))
    done <<< "$result_lines"
    if (( passed_total == 0 )); then
      echo "  ✗ 验证包里的测试函数跑过了，却读不到 cargo 的 test result 行，或要现判的用例一条都没通过：扫到 0 条不是通过"
      echo "     → 怎么办：cargo test --release -p singlefs-verification -- --list 看这个包里还剩几条测试函数；"
      echo "                一条都没有，就是被整批标了 ignored 或删掉了，补回来。"
      return 1
    fi
  fi
  # 配置开了 GPU（文件头「多机配置开了 GPU」）：判段判在 CPU 的判红，同一趟的判定照记；用例报的本次未跑等带特性那一趟补判之后再逐条报，不判红、不算通过
  local cpu_judging=() plain_where plain_line
  if (( crash_point_replay_gpu_configured )) && [[ "$crash_point_replay_break" != cpu-judging-accepted ]]; then
    mapfile -t cpu_judging < <(cpu_judging_lines "$pass_log")
    if (( ${#cpu_judging[@]} > 0 )); then
      printf '      %s\n' "${cpu_judging[@]:0:20}"
      # 判在 CPU 归得到的那几条记红、归不到的整个二进制不记、别的判绿的照键记绿（record_cpu_judging_pass）：弄坏开关 plain-pass-cpu-judging-records-nothing 下照改前一条都不记
      if (( plan_ok && package_pass_ran )) && [[ "$crash_point_replay_break" != plain-pass-cpu-judging-records-nothing ]] && account_test_results "$pass_log" "$accounting"; then
        record_cpu_judging_pass "$kind" "$plan_file" "$plan_output" "$accounting" "$CRASH_CASE_REPLAY_CELL_NAME" \
          "cargo test --release -p singlefs-verification --lib --tests（不带特性那一趟）"
      fi
      echo "  ✗ 验证包不带 gpu 特性那一趟：多机配置 $crash_point_replay_configuration_file 写了 ENABLE_GPU=1，日志里有 ${#cpu_judging[@]} 处判段判在 CPU（上面列了前 20 处），配了 GPU 不退回 CPU"
      echo "     → 怎么办：不带 gpu 特性那一趟遇到 ENABLE_GPU=1 打一行 NOT_RUN_<原因> test=<用例> reason=<一句> 报本次未跑、不判（crates 那一侧改），判归带特性那一趟（feature-gated-tests 格）；改完只重跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME"
      return 1
    fi
  fi
  # 逐条认结局与记号（文件头「复用」）；这一趟打了 NOT_RUN_ 的测试函数落进 crash_point_replay_plain_not_run_file，等带特性那一趟补判完再报（文件头「多机配置开了 GPU」）。
  # 认不成时退回按行收 NOT_RUN_（不知道是哪一条打的，补判不了）
  if (( package_pass_ran )) && account_test_results "$pass_log" "$accounting"; then accounting_ok=1; fi
  if [[ "$crash_point_replay_break" == plain-pass-not-run-unreported ]]; then
    :
  elif (( accounting_ok )); then
    not_run_markers_of "$accounting" "$crash_point_replay_plain_not_run_file"
  elif (( package_pass_ran )); then
    while IFS=$'\t' read -r plain_where plain_line; do
      [[ -n "$plain_line" ]] && printf '%s\t0\t%s\n' "$plain_where" "$plain_line" >>"$crash_point_replay_plain_not_run_file"
    done < <(test_not_run_lines "$pass_log")
  fi
  crash_point_replay_plain_not_run_total="$(grep -c . "$crash_point_replay_plain_not_run_file")"
  if (( package_pass_ran && plan_ok )); then
    package_pass_report="验证包 $(plan_report "$plan_output" "$crash_point_replay_plain_not_run_total")；跑了的 ${binary_count} 个测试二进制 ${passed_total} 条通过、${ignored_total} 条 ignored（登记的崩溃枚举用例留给按块那一路）"
  fi
  if (( plan_ok && package_pass_ran && accounting_ok )); then
    record_green_tests "$kind" "$plan_file" "$plan_output" "$accounting" "$CRASH_CASE_REPLAY_CELL_NAME" \
      "cargo test --release -p singlefs-verification --lib --tests（不带特性那一趟）" || package_pass_plan_red=1
  fi
  echo "  ✓ 验证包里的测试函数跑完（release，登记的崩溃枚举用例留给按块那一路）：${package_pass_report}"
  return 0
}


# crash_point_replay_keep_shard_rows <计划输出>：分片的一片（文件头「分片」shard）只现判归它的测试目标：计划里 judge、refused 的行按目标根文件（第五列）分，
#   目标按 C 字节序排好、序号（0 起）模片数等于片号的归这一片；别的片的那几行第二列改记 registered（plan-skip-names 照它经 --skip 排掉，判绿判红不记它们），
#   末行 PLAN 照剩下的重数，plan_judge、plan_refused 照新的那一行重读；别的片领的目标根文件另写进 <计划输出>.other-shard-roots，路径放进 crash_point_replay_other_shard_roots_file（有的话）
crash_point_replay_keep_shard_rows() {
  local plan_output="$1" roots="$1.shard-roots" kept="$1.this-shard" other_roots="$1.other-shard-roots" summary
  awk -F'\t' 'NF >= 7 && ($2 == "judge" || $2 == "refused") { print $5 }' "$plan_output" | LC_ALL=C sort -u >"$roots"
  [[ -s "$roots" ]] || return 0
  # 别的片领的目标根文件（序号模片数不等于片号的）：这一片不起它们（run_checker_package_tests）
  awk -v wanted="$crash_point_replay_shard_index" -v count="$crash_point_replay_shard_count" '(FNR - 1) % count != wanted { print }' "$roots" >"$other_roots"
  if [[ -s "$other_roots" ]]; then crash_point_replay_other_shard_roots_file="$other_roots"; fi
  awk -F'\t' -v OFS='\t' -v wanted="$crash_point_replay_shard_index" -v count="$crash_point_replay_shard_count" '
    FNR == NR { owner[$0] = (FNR - 1) % count; next }
    /^PLAN / { plan_line = $0; next }
    NF >= 7 && ($2 == "judge" || $2 == "refused") && owner[$5] != wanted { $2 = "registered" }
    NF >= 7 { tally[$2]++ }
    { print }
    END {
      ignored = ""
      if (match(plan_line, / ignored=[0-9]+/)) ignored = substr(plan_line, RSTART, RLENGTH)
      printf "PLAN judge=%d reuse=%d refused=%d not-compiled=%d%s\n", tally["judge"], tally["reuse"], tally["refused"], tally["not-compiled"], ignored
    }' "$roots" "$plan_output" >"$kept" && mv -- "$kept" "$plan_output"
  summary="$(grep '^PLAN ' "$plan_output" | tail -n 1)"
  plan_judge="$(sed -n 's/.* judge=\([0-9]*\).*/\1/p' <<<"$summary")"
  plan_refused="$(sed -n 's/.* refused=\([0-9]*\).*/\1/p' <<<"$summary")"
  echo "  · 分片：第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片现判归它的测试目标（要现判的共 $(grep -c . "$roots") 个目标，这一片 ${plan_judge} 条现判、${plan_refused} 条拒算；别的片的经 --skip 排掉）"
}


# crash_point_replay_package_pass_check <计划输出>：核账（文件头「分片」merge）：键对不上的测试函数一条都没有才过，不起 cargo；
#   没有分片阶段时单片那一步现判过的（crash_point_replay_shard_ran_file）不算没人判：它们判绿而按条记库不认，库里本来就没有键相同的判定；
#   复用哨兵抽中的强制现判行（计划里决定写 judge、原因以「强制现判」起头，每趟计划都抽）键相同、库里是绿，也不算：核账不起 cargo，不现判它们
#   拒算的不入库：合并那一步算各片跑过（片都退 0 才起合并）。判红返回 1
crash_point_replay_package_pass_check() {
  package_pass_ran=0
  local judge_unrun=0 judge_forced=0 judge_by_shard=0 judge_counts
  if (( plan_judge > 0 )); then
    judge_counts="$(awk -F'\t' -v ran="$crash_point_replay_shard_ran_file" 'BEGIN { while ((getline line < ran) > 0) seen[line] = 1 } NF >= 7 && $2 == "judge" { if ($7 ~ /^强制现判/) forced++; else if (!(($5 "\t" $6) in seen)) unrun++ } END { print unrun + 0, forced + 0 }' "$1")"
    read -r judge_unrun judge_forced <<<"$judge_counts"
    judge_by_shard=$((plan_judge - judge_unrun - judge_forced))
  fi
  if (( judge_unrun > 0 )); then
    echo "  ✗ 按条复用：还有 $judge_unrun 条测试函数在判定库里没有键相同的判定（$(plan_report "$1")），这一趟不现判"
    awk -F'\t' -v ran="$crash_point_replay_shard_ran_file" 'BEGIN { while ((getline line < ran) > 0) seen[line] = 1 } NF >= 7 && $2 == "judge" && $7 !~ /^强制现判/ && !(($5 "\t" $6) in seen) { print "      " $5 "\t" $6 }' "$1" | head -n 20
    echo "     → 怎么办：经 task-server/task submit 交这一道，服务照文件头「分片」拆成各片现判、合并核账；已经是合并那一步的，是各片判过的按此刻的键在服务里读不到"
    echo "                （分片之后源码变了，或片连的不是同一份服务）：等树里的改动停下再重交整件，判过的照键复用"
    return 1
  fi
  if (( judge_forced > 0 )); then
    echo "  · 按条复用：复用哨兵抽中 $judge_forced 条强制现判（库里键相同的判定是绿），核账这一步不起 cargo、不现判它们"
  fi
  if (( judge_by_shard > 0 )); then
    echo "  · 按条复用：$judge_by_shard 条测试函数单片那一步现判过、按条记库不认（打了 NOT_RUN_ 或结果行认不出结局，单片那一步「判绿不记库」列了），库里没有键相同的判定，不判红"
  fi
  if (( plan_refused > 0 )); then
    echo "  · 按条复用：拒算的 $plan_refused 条不入库，各片跑过（片都退 0 才起合并）"
  fi
  package_pass_report="验证包 $plan_reuse 条测试函数都有键相同的判定，全部复用，这一趟没起 cargo（现判 0、复用 $plan_reuse、拒算 $plan_refused，各片跑过）"
  return 0
}


# package_pass_reuse_held：按条复用失灵了（package_pass_plan_red，plan_failure_verdict 判的）就判红、返回 1；按块那一半照旧先跑完
package_pass_reuse_held() {
  (( ! package_pass_plan_red )) && return 0
  crash_point_replay_unavailable_only "$CRASH_CASE_REPLAY_CELL_NAME" && return 1   # 记库那一步 prune-server 不可用：不可用那一句已经打了，不是失灵
  echo "  ✗ $CRASH_CASE_REPLAY_CELL_NAME 这一格判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；这一趟照旧全部现判跑完、用例都过了"
  echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再只跑这一格：bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME；不能复用就判红，不退回全部现判当绿"
  return 1
}


# crash_point_replay_package_pass_holds_cases_back <不带特性那一趟绿没绿：1|0>：那一趟红了时用例接不接着跑（文件头「红了接着跑，停了不接着跑」）：
#   被停了、或 prune-server 不可用（用例的块判定同样读写它，跑了也判不了）不接着跑，退 0；别的红照跑，退 1。弄坏开关 package-pass-red-skips-cases 下红了一律不跑、也不打这一句
crash_point_replay_package_pass_holds_cases_back() {
  (( $1 )) && return 1
  if [[ -n "$crash_point_replay_stop_reason" ]]; then
    crash_point_replay_report_not_run "$CRASH_CASE_REPLAY_CELL_NAME：这一趟 ${crash_point_replay_stop_reason}，登记的 ${#crash_case_rows[@]} 条崩溃枚举用例不接着跑；重交之后判过的块照键复用"
    return 0
  fi
  if crash_point_replay_cell_in "$CRASH_CASE_REPLAY_CELL_NAME" ${crash_point_replay_unavailable_cells[@]+"${crash_point_replay_unavailable_cells[@]}"}; then
    crash_point_replay_report_not_run "$CRASH_CASE_REPLAY_CELL_NAME：prune-server 不可用，登记的 ${#crash_case_rows[@]} 条崩溃枚举用例不接着跑"
    return 0
  fi
  [[ "$crash_point_replay_break" == package-pass-red-skips-cases ]] && return 0
  echo "  · $CRASH_CASE_REPLAY_CELL_NAME：不带特性那一趟判红，登记的 ${#crash_case_rows[@]} 条崩溃枚举用例照跑；这一格照样判红，两部分的红都报"
  return 1
}


# crash_case_mark_registered <admission.py 答标记的子命令> <键>：退 0 登记了、退 1 没登记；判不出（它退别的）退 2、原因放进 crash_case_mark_problem。
crash_case_mark_registered() {
  local answer answer_exit
  if answer="$(python3 "$crash_point_replay_admission_module" "$1" "$ROOT" "$2")"; then answer_exit=0; else answer_exit=$?; fi
  if (( answer_exit == 0 || answer_exit == 1 )); then return "$answer_exit"; fi
  crash_case_mark_problem="admission.py $1 退 ${answer_exit}：${answer}"
  return 2
}


# crash_case_judged_by_test <键>：这条用例登记了 red-blocks=judged-by-test 没有（退出码同 crash_case_mark_registered）。
# 弄坏开关 judged-by-test-unrecognized 下一律答没登记（照没认这个标记，红块照判）。
crash_case_judged_by_test() {
  [[ "$crash_point_replay_break" == judged-by-test-unrecognized ]] && return 1
  crash_case_mark_registered crash-case-judged-by-test "$1"
}


# crash_case_findings_not_written <键>：这条用例登记了 findings=not-written-by-the-case 没有（退出码同 crash_case_mark_registered）。
# 弄坏开关 findings-not-written-unrecognized 下一律答没登记，findings-not-written-for-every-case 下一律答登记了。
crash_case_findings_not_written() {
  [[ "$crash_point_replay_break" == findings-not-written-unrecognized ]] && return 1
  [[ "$crash_point_replay_break" == findings-not-written-for-every-case ]] && return 0
  crash_case_mark_registered crash-case-findings-not-written "$1"
}


# crash_case_blocks_line_not_printed <键>：这条用例登记了 blocks-line=not-printed-by-the-case 没有（退出码同 crash_case_mark_registered）。
# 弄坏开关 blocks-line-not-printed-unrecognized 下一律答没登记，blocks-line-optional-for-every-case 下一律答登记了。
crash_case_blocks_line_not_printed() {
  [[ "$crash_point_replay_break" == blocks-line-not-printed-unrecognized ]] && return 1
  [[ "$crash_point_replay_break" == blocks-line-optional-for-every-case ]] && return 0
  crash_case_mark_registered crash-case-blocks-not-printed "$1"
}


# run_registered_crash_cases：登记的崩溃枚举用例逐条跑（文件头「用例逐条按块跑」），块按单元键照判定库复用、只判键对不上的块。
# 每条的日志与发现日志放 crash_point_replay_log_directory，跑完不删；全部判绿返回 0，有一条红返回 1（逐条的结论放进 red_cases、green_cases、reused_cases 与块的合计）。
# 带 gpu 特性的用例与别的一样一趟两段先后跑（文件头「显卡」）：判段把要判的块写成待判行、等 GPU 批处理服务判完，这一道不领卡；服务读不了时判段一直等，卡住由任务服务判。
run_registered_crash_cases() {
  local crash_case_row case_key case_package case_target case_function case_label case_log case_findings_file case_run_exit case_findings_exit
  local judged_lines_file judge_output blocks_judged_exit blocks_output blocks_exit blocks_text threads_text started_date finished_date
  local findings_judged_by_test_exit findings_not_written_exit blocks_not_printed_exit crash_case_mark_problem
  local case_threads_origin_text case_threads_note case_way
  local -a blocks_options=()
  red_cases=(); green_cases=(); reused_cases=(); not_run_cases=(); no_card_cases=()
  blocks_reused_total=0; blocks_fresh_total=0; blocks_refused_total=0
  started_date="$(TZ=Asia/Tokyo date +%F)"
  # 计划那一步编好、这一趟直接起的测试二进制（crash_point_replay_use_shared_case_binary 记）
  local -A case_binary_path=()
  # 核账那一趟（合并那一步，与没有分片阶段时自己走的合并）的用例日志是整份 0/1 跑的：日志目录里留一份 whole-case-logs 记号，
  #   下一趟分片的计划那一步照它认哪几个目录里的块数是整条用例的（crash_point_replay_recorded_case_blocks）
  [[ "$crash_point_replay_shard_phase" == shard ]] || : >"$crash_point_replay_log_directory/whole-case-logs"
  if [[ "$crash_point_replay_shard_phase" == shard ]]; then
    echo "  · 登记的 ${#crash_case_rows[@]} 条崩溃枚举用例逐条判第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片领的块，判的块写进判定库（prune-server），日志与块那一行由核账那一趟判"
  else
    echo "  · 登记的 ${#crash_case_rows[@]} 条崩溃枚举用例逐条核账（整份 0/1）：各片判的块按单元键照判定库（prune-server）复用，现判一块就判红"
  fi
  # 每条用例的挂钟（ITEM_DONE 的秒数，文件头「逐项点名」）：下一条开跑、或全部跑完时记上一条的
  local -A case_seconds=()
  local clock_key="" clock_started=0
  for crash_case_row in "${crash_case_rows[@]}"; do
    IFS=$'\t' read -r case_key case_package case_target case_function <<< "$crash_case_row"
    [[ -z "$clock_key" ]] || case_seconds["$clock_key"]=$((EPOCHSECONDS - clock_started))
    clock_key=""
    # 前一条被停了（文件头「红了接着跑，停了不接着跑」）：这一条不接着起
    if [[ -n "$crash_point_replay_stop_reason" ]]; then not_run_cases+=("$case_key"); continue; fi
    clock_key="$case_key"; clock_started="$EPOCHSECONDS"
    case_label="${case_key#crash-case:}"
    if ! read_crash_case_command "$case_key"; then
      echo "  ✗ $case_key：准入模块交不出起用例的命令：$case_command_problem"
      echo "     → 怎么办：单跑 python3 scripts/gate/admission.py crash-case-command <根> $case_key 看它报什么（取不到 git common-dir、"
      echo "                本机核数读不到、SINGLEFS_CRASH_POINT_REPLAY_THREADS 不是正整数、每台默认重活线程数的登记取不了都在这里报）；修好之后重跑这一道。"
      red_cases+=("$case_key")
      continue
    fi
    # 照计划那一步编好的测试二进制直接起（文件头「分片」shard、merge；没有计划那一步放的、或放的用不上，打一行、经 cargo test 起）
    crash_point_replay_use_shared_case_binary "$case_key" "$case_label" "$case_package" "$case_target" || true
    case_log="$crash_point_replay_log_directory/log.$case_label"
    case_findings_file="$case_log.findings.tsv"
    rm -f -- "$case_log" "$case_findings_file"
    case_threads_origin_text="没设，取每台默认重活线程数的登记值"
    if [[ "$case_threads_origin" == explicit ]]; then case_threads_origin_text="显式设的"; fi
    case_threads_note="SINGLEFS_CRASH_POINT_REPLAY_THREADS=${case_threads}（${case_threads_origin_text}），本机 ${case_machine_cores} 核"
    case_way="核账（整份 0/1）：${case_command[*]}"
    [[ "$crash_point_replay_shard_phase" == shard ]] && case_way="第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片：${case_command[*]}"
    echo "  · $case_key 开跑（${case_way}；${case_threads_note}）"
    echo "    日志：${case_log}；发现日志：${case_findings_file}"
    if run_crash_case "$case_log" "$case_findings_file" "${case_command[@]}"; then case_run_exit=0; else case_run_exit=$?; fi
    report_bulk_lines "$CRASH_CASE_REPLAY_CELL_NAME" "$case_key 那一趟" "$case_log"
    crash_point_replay_note_stop "$case_key 那一趟" "$case_run_exit" "$case_log"
    report_pass_outcome "$CRASH_CASE_REPLAY_CELL_NAME" "$case_key 那一趟" "$case_run_exit" "$case_log"
    # 观察者交回的盘取不到卡（文件头「显卡」）：不判红、记本次未跑，分片的一片与核账那一趟一样；别的都不再判
    if [[ "$case_run_exit" != 0 ]] && crash_point_replay_case_has_no_card_for_observer_images "$case_log"; then
      crash_point_replay_report_not_run "$CRASH_CASE_REPLAY_CELL_NAME：$case_key 那一趟（cargo 退 $case_run_exit）$(crash_point_replay_observer_images_howto)"
      no_card_cases+=("$case_key")
      continue
    fi
    # 分片的一片（文件头「分片」shard）：只看 cargo test 的退出码，日志、块那一行与发现日志由核账那一趟判
    if [[ "$crash_point_replay_shard_phase" == shard ]]; then
      if [[ "$case_run_exit" != 0 ]]; then
        [[ -f "$case_log" ]] && tail -40 "$case_log"
        echo "  ✗ $case_key 第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片判红：cargo test 退 $case_run_exit（上面是它的尾部）"
        echo "     → 怎么办：看日志 $case_log；断言消息里是第一处对不上的计数或违例。改完经 task-server/task submit 交 bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME --item $case_key（判过的块照键复用）"
        red_cases+=("$case_key")
      else
        echo "  ✓ $case_key 第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片判完它领的块"
        green_cases+=("$case_key")
      fi
      continue
    fi
    # 登记了 red-blocks=judged-by-test 的用例（文件头「复用」）：用例的判定是 cargo test 的退出码，块那一行只核在不在、数对不对得上（crash-case-blocks --ran），红块不判；
    # 没登记的块那一行有红判红。弄坏开关 judged-by-test-ignores-exit 下带标记的用例 cargo test 退非 0 也当绿；ran-for-every-case 下每条用例都只核数、不判红块
    crash_case_mark_problem=""
    if crash_case_judged_by_test "$case_key"; then blocks_judged_exit=0; else blocks_judged_exit=$?; fi
    findings_judged_by_test_exit="$blocks_judged_exit"
    if (( blocks_judged_exit == 0 )) && [[ "$crash_point_replay_break" == judged-by-test-ignores-exit ]]; then case_run_exit=0; fi
    [[ "$crash_point_replay_break" == ran-for-every-case ]] && blocks_judged_exit=0
    if crash_case_findings_not_written "$case_key"; then findings_not_written_exit=0; else findings_not_written_exit=$?; fi
    if crash_case_blocks_line_not_printed "$case_key"; then blocks_not_printed_exit=0; else blocks_not_printed_exit=$?; fi
    # 发现表跑完就打（判红的那一趟最要看它），判定留到下面。登记了 red-blocks=judged-by-test 的用例发现表只打印、不判红（文件头「复用」）；
    # 弄坏开关 judged-by-test-findings-judged 下照判，findings-unjudged-for-every-case 下每条都不判
    if report_crash_case_findings "$case_findings_file" "$case_log" "$findings_not_written_exit"; then case_findings_exit=0; else case_findings_exit=$?; fi
    if (( case_findings_exit != 0 && findings_judged_by_test_exit == 0 )) && [[ "$crash_point_replay_break" != judged-by-test-findings-judged ]]; then
      echo "    $case_key：登记了 red-blocks=judged-by-test，发现表只打印、上面的「问题：」不判红（按 cargo test 的退出码判）"
      case_findings_exit=0
    fi
    [[ "$crash_point_replay_break" == findings-unjudged-for-every-case ]] && case_findings_exit=0
    if [[ "$case_run_exit" != 0 ]]; then
      [[ -f "$case_log" ]] && tail -40 "$case_log"
      echo "  ✗ $case_key 判红：cargo test 退非 0（上面是它的尾部）"
      echo "     → 怎么办：单跑看细节：cargo test --release -p $case_package --test $case_target -- --exact $case_function --nocapture"
      echo "                断言消息里是第一处对不上的计数或违例。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
      red_cases+=("$case_key")
      continue
    fi
    judged_lines_file="$crash_point_replay_scratch_directory/judged.$case_label"
    if ! judge_output="$(python3 "$crash_point_replay_admission_module" crash-case-judge "$ROOT" "$case_key" "$case_log" "$judged_lines_file")"; then
      printf '%s\n' "$judge_output" | sed 's/^/       /'
      echo "  ✗ $case_key 的用例跑过了，日志却判不绿（上面逐条列出）"
      echo "     → 怎么办：计数行不是恰好一行、过滤之后没跑到恰好一条用例，对一对 stage-inputs.tsv 里 $case_key 那一行第三列与用例打印的行；"
      echo "                不是穷举枚举的去 crates/singlefs-verification/src/crash_state_enumeration.rs 的 enumerate_crash_states 看；只起了 1 个线程看 crash_point_replay_worker_threads 读没读到 SINGLEFS_CRASH_POINT_REPLAY_THREADS，"
      echo "                真要单线程跑（比对单进程读数），显式写 SINGLEFS_CRASH_POINT_REPLAY_THREADS=1 bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME <根>。"
      red_cases+=("$case_key")
      continue
    fi
    if [[ "$case_findings_exit" != 0 ]]; then
      echo "  ✗ $case_key 的用例跑过了、日志也判得绿，发现日志却判红（上面发现表里的「问题：」逐条列出）"
      echo "     → 怎么办：「这一趟没跑完」看日志 $case_log 的尾部是 panic 还是被杀；red_states 不是 0 是有状态判红而用例没红，"
      echo "                对一对用例的计数断言与 crates/singlefs-verification/src/crash.rs 的发现表；发现日志不在而日志里有 CRASH_POINT_REPLAY_FINDINGS 行，"
      echo "                是 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 没传进用例（看本阶段 run_crash_case），或用例的环境守卫清掉了它（读过用例确认之后在登记行第三列写 findings=not-written-by-the-case）。"
      red_cases+=("$case_key")
      continue
    fi
    blocks_options=()
    if (( blocks_judged_exit == 2 || findings_not_written_exit == 2 || blocks_not_printed_exit == 2 )); then
      echo "  ✗ $case_key：判不出它登记了哪些判法标记（${crash_case_mark_problem}），块判定那一行没法定判法"
      echo "     → 怎么办：单跑上面那条子命令 python3 scripts/gate/admission.py <子命令> <根> $case_key，照它报的改 .claude/gate.d/stage-inputs.tsv 里那一行"
      red_cases+=("$case_key")
      continue
    fi
    if (( blocks_judged_exit == 0 )); then
      blocks_options=(--ran)
      echo "    $case_key：登记了 red-blocks=judged-by-test，按 cargo test 的退出码判（已退 0），块那一行只核在不在、数对不对得上，红块不判"
    fi
    if (( blocks_not_printed_exit == 0 )); then blocks_options+=(--no-line-registered); fi
    if blocks_output="$(python3 "$crash_point_replay_admission_module" crash-case-blocks "$ROOT" "$case_log" ${blocks_options[@]+"${blocks_options[@]}"})"; then blocks_exit=0; else blocks_exit=$?; fi
    if (( blocks_exit != 0 )); then
      echo "  ✗ $case_key 的块判定那一行不对（crash-case-blocks${blocks_options[*]:+ ${blocks_options[*]}} 退 $blocks_exit：$blocks_output）"
      echo "     → 怎么办：退 1 是有块判红（没登记 red-blocks=judged-by-test 的用例）：看日志里那几块的发现；退 4 是用例没打恰好一行 CRASH_CASE_BLOCKS、或 blocks 不等于 reused + fresh + refused（接口在 scripts/gate/admission.py 文件头）；"
      echo "                用例的枚举按设计 panic 在打那一行之前的，读过用例确认之后在登记行第三列写 blocks-line=not-printed-by-the-case。"
      red_cases+=("$case_key")
      continue
    fi
    blocks_text="$blocks_output"
    if [[ "$blocks_output" =~ 复用\ ([0-9]+)、现判\ ([0-9]+)、红\ ([0-9]+)、拒算\ ([0-9]+) ]]; then
      blocks_reused_total=$((blocks_reused_total + BASH_REMATCH[1])); blocks_fresh_total=$((blocks_fresh_total + BASH_REMATCH[2]))
      blocks_refused_total=$((blocks_refused_total + BASH_REMATCH[4]))
      # 核账（文件头「分片」merge）：各片判的块按此刻的键全在服务里读到，这一趟一块不现判。
      # 登记了 red-blocks=judged-by-test 的用例例外：记着红的块不当绿复用、每一趟都现判（.claude/rules/verification.md「一个门禁：两种跑法」），
      # 核账这一趟现判的块数不超过红块数的就是那几块红块重判，不算各片判的块读不到；多出来的才算。
      # 弄坏开关 judged-by-test-fresh-unallowed 下带标记的用例也按 0 块许现判
      blocks_fresh_allowed=0
      if (( blocks_judged_exit == 0 )) && [[ "$crash_point_replay_break" != judged-by-test-fresh-unallowed ]]; then blocks_fresh_allowed="${BASH_REMATCH[3]}"; fi
      if (( BASH_REMATCH[2] > blocks_fresh_allowed )); then
        echo "  ✗ $case_key 核账那一趟现判了 ${BASH_REMATCH[2]} 块（$blocks_output；记着红、每趟重判的块许 $blocks_fresh_allowed 块）：各片判的块有按此刻的键在判定库里读不到的"
        echo "     → 怎么办：经 task-server/task submit 交这一道（服务照文件头「分片」拆）；已经是合并那一步的，是分片之后源码变了或片连的不是同一份服务，"
        echo "                等树里的改动停下再重交整件（这一趟现判的块已经写进服务，键没变的照复用）"
        red_cases+=("$case_key")
        continue
      fi
    elif (( blocks_not_printed_exit != 0 )); then
      echo "  ✗ $case_key 的块那一行读不出现判数（$blocks_output）：核不了各片判的块读没读到"
      echo "     → 怎么办：对一对 scripts/gate/admission.py crash-case-blocks 的成功句与这里认的写法「复用 N、现判 N、红 N、拒算 N」"
      red_cases+=("$case_key")
      continue
    fi
    if grep -qE '现判 0、' <<<"$blocks_output"; then
      echo "  · $case_key 复用：全部块都有键相同的判定（$blocks_output）"
      reused_cases+=("$case_key")
    fi
    threads_text="${judge_output//$'\n'/；}${judge_output:+；}${case_threads_note}"
    echo "  ✓ $case_key 判绿（${threads_text}；${blocks_text}），判日志认出的行原样："
    sed 's/^/      /' "$judged_lines_file"
    green_cases+=("$case_key")
    rm -f -- "$case_log.bulk-lines" "$case_log.bulk-lines.previous"   # 判绿了不用末尾现场；日志目录跑完不删，一百多条用例各留一份占地方
  done
  [[ -z "$clock_key" ]] || case_seconds["$clock_key"]=$((EPOCHSECONDS - clock_started))
  # 每条一行 ITEM_DONE（文件头「逐项点名」）：全部块都复用的一件复用、别的一件现判，判红 exit 1；分片的一片不打（只判了一份块，挂钟不是这一项的）。弄坏开关 item-done-unreported 下不打
  if [[ "$crash_point_replay_break" != item-done-unreported && "$crash_point_replay_shard_phase" != shard ]]; then
    for crash_case_row in "${crash_case_rows[@]}"; do
      case_key="${crash_case_row%%$'\t'*}"
      if crash_point_replay_cell_in "$case_key" ${red_cases[@]+"${red_cases[@]}"}; then
        printf 'ITEM_DONE item=%s seconds=%s judged=1 reused=0 exit=1\n' "$case_key" "${case_seconds[$case_key]:-0}"
      elif crash_point_replay_cell_in "$case_key" ${no_card_cases[@]+"${no_card_cases[@]}"}; then
        printf 'ITEM_DONE item=%s seconds=%s judged=0 reused=0 exit=78\n' "$case_key" "${case_seconds[$case_key]:-0}"
      elif crash_point_replay_cell_in "$case_key" ${not_run_cases[@]+"${not_run_cases[@]}"}; then
        printf 'ITEM_DONE item=%s seconds=0 judged=0 reused=0 exit=77\n' "$case_key"
      elif crash_point_replay_cell_in "$case_key" ${reused_cases[@]+"${reused_cases[@]}"}; then
        printf 'ITEM_DONE item=%s seconds=%s judged=0 reused=1 exit=0\n' "$case_key" "${case_seconds[$case_key]:-0}"
      else
        printf 'ITEM_DONE item=%s seconds=%s judged=1 reused=0 exit=0\n' "$case_key" "${case_seconds[$case_key]:-0}"
      fi
    done
  fi
  finished_date="$(TZ=Asia/Tokyo date +%F)"
  (( ${#not_run_cases[@]} == 0 )) || crash_point_replay_report_not_run "$CRASH_CASE_REPLAY_CELL_NAME：这一趟 ${crash_point_replay_stop_reason}，后面 ${#not_run_cases[@]} 条崩溃枚举用例不接着跑：${not_run_cases[*]}"
  # 观察者交回的盘取不到卡而记了本次未跑的几条（文件头「显卡」）：这里只点名，整道退 78 由末尾定（别的有红照退 1）
  local gpu_not_run_cases=(${no_card_cases[@]+"${no_card_cases[@]}"})
  local gpu_not_run_reasons="" gpu_not_run_note=""
  if (( ${#no_card_cases[@]} > 0 )); then
    gpu_not_run_reasons="观察者交回的盘取不到卡"
    gpu_not_run_note="；另有 ${#no_card_cases[@]} 条本次未跑（观察者交回的盘取不到卡）：${no_card_cases[*]}"
  fi
  if (( ${#red_cases[@]} > 0 )); then
    echo "  ✗ ${#red_cases[@]} 条崩溃枚举用例判红：${red_cases[*]}（这一趟判绿 ${#green_cases[@]} 条，其中全部块复用 ${#reused_cases[@]} 条${gpu_not_run_note}；开跑 ${started_date}，跑完 ${finished_date}）"
    echo "     → 怎么办：逐条照它自己那一句「→ 怎么办」改；改完只重跑红的几条：bash .claude/gate.d/crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME --item <用例名>（判过的块照键复用，只判改到的块）。"
    return 1
  fi
  if [[ "$crash_point_replay_shard_phase" == shard ]]; then
    echo "  ✓ 第 ${crash_point_replay_shard_index}/${crash_point_replay_shard_count} 片：登记的 ${#crash_case_rows[@]} 条崩溃枚举用例各判完这一片领的块${gpu_not_run_note}（开跑 ${started_date}，跑完 ${finished_date}；日志在 ${crash_point_replay_log_directory}/）"
    return 0
  fi
  echo "  ✓ 崩溃状态块：现判 ${blocks_fresh_total}、复用 ${blocks_reused_total}、拒算 ${blocks_refused_total}、本次未跑 0"
  if (( ${#gpu_not_run_cases[@]} > 0 )); then
    echo "  · 登记的 ${#crash_case_rows[@]} 条崩溃枚举用例${crash_point_replay_separate_run_note}这一趟判绿 ${#green_cases[@]} 条（${green_cases[*]:-无}）、本次未跑 ${#gpu_not_run_cases[@]} 条（${gpu_not_run_cases[*]}：${gpu_not_run_reasons}），不是都判绿；开跑 ${started_date}，跑完 ${finished_date}；日志在 ${crash_point_replay_log_directory}/"
    crash_point_replay_cell_state="绿；${#gpu_not_run_cases[@]} 条崩溃枚举用例本次未跑（${gpu_not_run_reasons}）"
    return 0
  fi
  echo "  ✓ 登记的 ${#crash_case_rows[@]} 条崩溃枚举用例${crash_point_replay_separate_run_note}这一趟都判绿（${green_cases[*]:-无}），其中全部块复用 ${#reused_cases[@]} 条（${reused_cases[*]:-无}）；开跑 ${started_date}，跑完 ${finished_date}；日志在 ${crash_point_replay_log_directory}/"
  return 0
}
