#!/usr/bin/env bash
# crash-point-replay.sh 的自证：run_selftest（各格，一块一块「if selftest_cell_wanted <格名>」）、自证并行（selftest_worker_cells、selftest_run_workers、selftest_collect_workers、run_selftest_parallel）、
#   开关轮分段与开关轮（selftest_wheel_*、selftest_switch_wheel_with_progress、selftest_wheel_and_summary）。
# 被 crash-point-replay.sh source 的函数库（在 --selftest 那一支里 source），只定义函数与几个全局变量，不单独调；
#   读的全局变量与函数（replay_script、replay_script_directory、replay_selftest_library、replay_break、replay_break_is、repository_root、各 shard_* 与 crash_amplification_* 的函数）
#   都在 crash-point-replay.sh 与它 source 的 crash-amplification-lib.sh 里定义，source 它的那一刻之后才有。
#   自证读脚本文本的地方：「if selftest_cell_wanted」块与代码里 replay_break_is 认的开关读本文件（replay_selftest_library），文件头开关表读 crash-point-replay.sh（replay_script）。

# ── 自证并行（文件头「自证并行」）
# selftest_service_refusal：自证只在临时 prune-server 上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service）；不是就打原因与出路、交 1
selftest_service_refusal() {
  local service_problem
  if ! service_problem="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem())' "$replay_script_directory/../prune-server" 2>&1)" || [[ -n "$service_problem" ]]; then
    echo "  ✗ crash-point-replay.sh 自证没跑：${service_problem:-读不了 scripts/prune-server/prune_server_client.py}"
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）"
    return 1
  fi
  return 0
}
# selftest_parallel_width：自证并行的路数 = 开关轮的并行度：CRASH_AMPLIFICATION_SELFTEST_JOBS，没给取每台默认重活线程数；取不出交 1（调用方拒，不退回 1 路）
selftest_parallel_width() {
  local width="${CRASH_AMPLIFICATION_SELFTEST_JOBS:-$(heavy_thread_default)}"
  [[ "$width" =~ ^[1-9][0-9]*$ ]] || return 1
  printf '%s\n' "$width"
}
# selftest_worker_cells <脚本文件> <路数>：把脚本里 run_selftest 的格分到至多 <路数> 路，一路一行，打逗号隔开的格名（交给 CRASH_POINT_REPLAY_SELFTEST_CELLS）。
#   格 = 「  if selftest_cell_wanted <格名…>; then」那一块（行首两个空格，块里的 if 与 fi 成对）；一块点名几个格名、或几块点名同一个格名的，是同一组（块与块之间读别人留下的东西，
#   点名其中一格时前一块照跑，所以这些格名必须分在同一路，各格的失败行与趟数才不重复）。每组的分量 = 它各块的块数加块里起入口的调用数（entry、default_shard_plan、default_shard_run、default_shard_merge、three_round 各算一次，起入口是一格里最重的事）；
#   分量大的先分，每一组放到此刻分量最小的那一路（LPT），
#   路内的格名保持分组次序。弄坏开关 cell-groups-by-name 下一个格名一组、不看块里点名了几个格名
selftest_worker_cells() {
  local by_name=0
  replay_break_is cell-groups-by-name && by_name=1
  awk -v workers="$2" -v by_name="$by_name" '
    function find(x) { while (parent[x] != x) x = parent[x]; return x }
    /^  selftest_cell_wanted\(\) \{/ { in_cells = 1; depth = 0; next }
    in_cells && /^\}/ { in_cells = 0 }
    in_cells {
      if ($0 ~ /^  if /) {
        if ($0 ~ /^  if selftest_cell_wanted /) {
          line = $0; sub(/^  if selftest_cell_wanted /, "", line); sub(/[[:space:]]*(\\|;).*$/, "", line)
          n = split(line, names, " ")
          for (i = 1; i <= n; i++) {
            if (!(names[i] in parent)) { parent[names[i]] = names[i]; order[++count] = names[i] }
            if (i > 1 && !by_name) { a = find(names[1]); b = find(names[i]); if (a != b) parent[b] = a }
          }
          if (depth == 0) { block_name = names[1]; weight_of[block_name]++ }
        }
        if ($0 !~ /[;[:space:]]fi[[:space:]]*(#.*)?$/) depth++
      } else if ($0 ~ /^  fi([[:space:]]|$)/) {
        depth--
      }
      if (depth > 0 && block_name != "" && $0 !~ /^[[:space:]]*#/ && $0 !~ /cell_failed/ && $0 ~ /(^|[^a-z_])(entry|default_shard_plan|default_shard_run|default_shard_merge|three_round)[[:space:]]/) weight_of[block_name]++
      if (depth <= 0) block_name = ""
    }
    END {
      for (i = 1; i <= count; i++) {
        root = find(order[i])
        if (!(root in seen)) { seen[root] = 1; group_root[++groups] = root }
        members[root] = members[root] (members[root] == "" ? "" : ",") order[i]
      }
      for (name in weight_of) group_weight[find(name)] += weight_of[name]
      width = (workers + 0 < groups) ? workers + 0 : groups
      for (worker = 1; worker <= width; worker++) load[worker] = 0
      for (assigned = 0; assigned < groups; assigned++) {
        best = 0
        for (g = 1; g <= groups; g++) if (!used[g] && (best == 0 || group_weight[group_root[g]] > group_weight[group_root[best]])) best = g
        used[best] = 1
        lightest = 1
        for (worker = 2; worker <= width; worker++) if (load[worker] < load[lightest]) lightest = worker
        list[lightest] = list[lightest] (list[lightest] == "" ? "" : ",") members[group_root[best]]
        load[lightest] += group_weight[group_root[best]] + 1
      }
      for (worker = 1; worker <= width; worker++) print list[worker]
    }' "$1"
}
# selftest_run_workers <放输出的目录> <路数> -- <命令…>：selftest_worker_assignments（每路一项，逗号隔开的格名）每路起一份 <命令…>（子 shell 里设 CRASH_POINT_REPLAY_SELFTEST_CELLS 成该路的格名再 exec），
#   标准输出进 <目录>/worker-<序号>.out、标准错误进 <目录>/worker-<序号>.err，至多 <路数> 路同时在跑（第 i 路等第 i-路数 路收完再起）；退出码按序号收进 selftest_worker_exits，
#   收回几路放 selftest_workers_collected。弄坏开关 parallel-workers-serial 下一路一路地起
selftest_worker_assignments=(); selftest_worker_exits=(); selftest_workers_collected=0
selftest_run_workers() {
  local directory="$1" width="$2" index count="${#selftest_worker_assignments[@]}" pids=()
  shift 3
  selftest_worker_exits=(); selftest_workers_collected=0
  replay_break_is parallel-workers-serial && width=1
  for ((index = 0; index < count; index++)); do
    if (( index >= width )); then
      wait "${pids[index - width]}"; selftest_worker_exits[index - width]=$?
    fi
    (
      export CRASH_POINT_REPLAY_SELFTEST_CELLS="${selftest_worker_assignments[index]}"
      exec "$@"
    ) >"$directory/worker-$index.out" 2>"$directory/worker-$index.err" &
    pids[index]=$!
  done
  for ((index = count > width ? count - width : 0; index < count; index++)); do
    wait "${pids[index]}"; selftest_worker_exits[index]=$?
  done
  selftest_workers_collected="${#selftest_worker_exits[@]}"
}
# selftest_collect_workers <放输出的目录>：按序号回读每路的输出：标准错误原样转到标准错误（失败行与出路在里面），标准输出里每路自己的成功句（「✓ --selftest 只跑点名的」起头）不转、别的行原样转；
#   每路的趟数取它成功句的「起了 N 趟都对」或失败句的「N 格里 M 格判错」，加起来放 selftest_workers_cells；判错的格数放 selftest_workers_failures：退出码非 0 的一路按它标准错误里
#   「✗ 自证」的行数（没有就算 1 格），退 0 而没打成功句的算 1 格，派出收回对不上算 1 格。弄坏开关 parallel-failure-dropped 下只数趟数、不看退出码与失败行
selftest_workers_cells=0; selftest_workers_failures=0
selftest_collect_workers() {
  local directory="$1" index count="${#selftest_worker_assignments[@]}" worker_cells worker_failed
  selftest_workers_cells=0; selftest_workers_failures=0
  for ((index = 0; index < count; index++)); do
    grep -vF '✓ --selftest 只跑点名的' "$directory/worker-$index.out"
    cat -- "$directory/worker-$index.err" >&2
    worker_cells="$(sed -nE 's/.*起了 ([0-9]+) 趟都对.*/\1/p' "$directory/worker-$index.out" | tail -n 1)"
    [[ -n "$worker_cells" ]] || worker_cells="$(sed -nE 's/.*✗ --selftest：([0-9]+) 格里 [0-9]+ 格判错.*/\1/p' "$directory/worker-$index.err" | tail -n 1)"
    selftest_workers_cells=$((selftest_workers_cells + ${worker_cells:-0}))
    replay_break_is parallel-failure-dropped && continue
    if [[ "${selftest_worker_exits[index]:-}" != 0 ]]; then
      worker_failed="$(grep -c '✗ 自证 ' "$directory/worker-$index.err")"
      selftest_workers_failures=$((selftest_workers_failures + (worker_failed > 0 ? worker_failed : 1)))
      (( worker_failed > 0 )) || echo "  ✗ 自证并行：第 $((index + 1)) 路（${selftest_worker_assignments[index]}）退 ${selftest_worker_exits[index]:-（没收到）}，标准错误里没有失败行" >&2
    elif [[ -z "$worker_cells" ]]; then
      selftest_workers_failures=$((selftest_workers_failures + 1))
      echo "  ✗ 自证并行：第 $((index + 1)) 路（${selftest_worker_assignments[index]}）退 0 却没打成功句" >&2
    fi
  done
  if (( ${#selftest_worker_exits[@]} != count )); then
    echo "  ✗ 自证并行：派出 $count 路、收回 ${#selftest_worker_exits[@]} 路" >&2
    selftest_workers_failures=$((selftest_workers_failures + 1))
  fi
}

run_selftest() {
  local work root calls cells=0 failures=0 exit_code=0
  # 按条复用那几格经 admission.py 写 prune-server：只在临时服务上跑，不是就退 2、一格都不跑
  selftest_service_refusal || return 2
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-point-replay-selftest.XXXXXX")"
  trap 'rm -rf -- "${work:?}"' EXIT
  root="$work/root"; calls="$work/calls"
  mkdir -p "$root/.claude/gate.d" "$root/scripts" "$root/scripts/crash" "$root/scripts/gate" "$root/crates/singlefs-verification/tests" "$work/bin" "$work/home"
  printf '%s\n' "$SELFTEST_MARK_CONTENT" >"$work/mark"
  # 造出来的小仓：三项（两段都有、只有崩溃点重放、只有放量），合表核法照真的那一份判
  printf '%s\n' "# 自证的合表" $'alpha\tcrash-case:alpha\talpha-amp\t# 两段都有' $'beta\tcrash-case:beta\t-\t# 只有崩溃点重放' \
    $'gamma\t-\tgamma-amp\t# 只有放量' >"$root/.claude/gate.d/crash-point-replay-items.tsv"
  printf '%s\n' "# 自证的登记表" $'crash-case:alpha\tcrates/\ttest=singlefs-verification:alpha:alpha_full\t# 自证' \
    $'crash-case:beta\tcrates/\ttest=singlefs-verification:beta:beta_full\t# 自证' >"$root/.claude/gate.d/stage-inputs.tsv"
  : >"$root/crates/singlefs-verification/tests/alpha.rs"; : >"$root/crates/singlefs-verification/tests/beta.rs"; : >"$root/crates/singlefs-verification/tests/gamma_amp.rs"
  # 按段登记文件：一行登记（alpha 那一项的流 alpha_flow 第 0 段，五列），segments-registry-refused 那一格临时换成核不过的一份、跑完换回
  printf '%s\n' "# 自证的按段登记" "alpha	alpha_flow	0	7	$(printf '0123456789abcdef%.0s' 1 2 3 4)" >"$root/.claude/gate.d/crash-point-replay-segments.tsv"
  cat >"$root/scripts/crash/crash-amplification.sh" <<'STUB'
#!/usr/bin/env bash
# 假的崩溃放量入口（自证用）：ITEMS 给合表核法读；起它时记一行参数，按 --item（没给就 ITEMS 全部）逐项打 CRASH_AMPLIFICATION_ITEM 行
ITEMS=(
  $'alpha-amp\talpha\talpha_amplified'
  $'gamma-amp\tgamma_amp\tgamma_amplified'
)
config_summary() { local file="${SINGLEFS_MULTI_HOST_CONFIG:-}"; if [[ -f "$file" ]]; then printf 'across=%s peer=%s cards=%s' "$(grep -c '^ENABLE_ACROSS_MACHINES=1' "$file")" "$(grep -c 'peer-selftest' "$file")" "$(grep -c '^GPU_CARDS_BY_PRIORITY' "$file")"; else printf 'none'; fi; }
printf 'pipeline-entry %s\n' "$*" >>"${STUB_CALLS:?}"
printf 'pipeline-env config=[%s]\n' "$(config_summary)" >>"$STUB_CALLS"
printf 'pipeline-env segments=[%s]\n' "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" >>"$STUB_CALLS"
printf 'pipeline-env slice-states=[%s]\n' "${SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES-unset}" >>"$STUB_CALLS"
printf 'pipeline-env expand=[%s]\n' "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO-unset}" >>"$STUB_CALLS"
printf 'pipeline-env judge-expand=[%s]\n' "${SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO-unset}" >>"$STUB_CALLS"
printf 'pipeline-env scoped-checked=[%s]\n' "${SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED-unset}" >>"$STUB_CALLS"
if [[ -n "${STUB_AMP_PERSISTENT:-}" ]]; then   # 照引擎按键复用：判过的分项在 STUB_AMP_SERVICE 指的目录（演 prune-server 里那一份判定库）里留一份文件，再判就报复用、现判 0 态
  persistent_service="${STUB_AMP_SERVICE:?}"
fi
items=(); previous=""
for argument in "$@"; do [[ "$previous" == --item ]] && items+=("$argument"); previous="$argument"; done
(( ${#items[@]} )) || items=(alpha-amp gamma-amp)
exit_code=0
for item in "${items[@]}"; do
  [[ " ${STUB_AMP_DROP:-} " == *" $item "* ]] && continue
  red=0; verdict=green; item_exit=0
  if [[ " ${STUB_AMP_RED:-} " == *" $item "* ]]; then red=2; verdict=red; item_exit=1; exit_code=1; fi
  [[ " ${STUB_AMP_NOT_RUN:-} " == *" $item "* ]] && verdict=not-run
  # STUB_AMP_NOT_RUN_REASON_TAIL：接在 NOT_RUN 那一行末尾的原样字节（record-text-with-bad-bytes 格带两个不成字符的字节）
  # STUB_AMP_SOURCE_CHANGED（pipeline-source-changed-not-run 格）：照真入口的样子，分项驱动判段跑的过程中源码变了——入口打「== 分项」那一行、驱动打 CRASH_AMPLIFICATION_NOT_RUN step=judge 那一行、退 77、ITEM 行 verdict=not-run
  if [[ " ${STUB_AMP_SOURCE_CHANGED:-} " == *" $item "* ]]; then
    echo "== 分项 $item（stub::run，展开 6）自证"
    echo "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment reason=跑的过程中源码变了，判段没在录入段那份源码上判完（录入段之后 aaaa，本机那一趟收尾后 bbbb）${STUB_AMP_NOT_RUN_REASON_TAIL:-}"
    verdict=not-run; item_exit=77
  fi
  states=10; reused=4
  if [[ -n "${STUB_AMP_PERSISTENT:-}" ]]; then
    unit_key=""   # STUB_AMP_UNITS 设了：键另带这一项的单元文件（<STUB_AMP_UNITS>/<分项>）的内容摘要（复用三问那一格用）
    [[ -n "${STUB_AMP_UNITS:-}" ]] && unit_key=".$(sha256sum "$STUB_AMP_UNITS/$item" | cut -c1-64)"
    if [[ -e "$persistent_service/$item$unit_key.judged" ]]; then states=0; reused=10; else states=10; reused=0; mkdir -p -- "$persistent_service" && : >"$persistent_service/$item$unit_key.judged"; fi
  fi
  echo "CRASH_AMPLIFICATION_ITEM item=$item exit=$item_exit states=$states reused=$reused red=$red lines=1 expand_up_to=6 exhaustive=true verdict=$verdict seconds=0 ignored_points=0 states_skipped_by_ignore=0"
done
exit "${STUB_AMP_EXIT:-$exit_code}"
STUB
  cat >"$root/.claude/gate.d/crash-point-replay.sh" <<'STUB'
#!/usr/bin/env bash
# 假的门禁 crash-point-replay（自证用）：记一行参数；STUB_CRASH_POINT_REPLAY_REUSED 设了就给每条点名的用例打一行复用；退 STUB_CRASH_POINT_REPLAY_EXIT
config_summary() { local file="${SINGLEFS_MULTI_HOST_CONFIG:-}"; if [[ -f "$file" ]]; then printf 'across=%s peer=%s cards=%s' "$(grep -c '^ENABLE_ACROSS_MACHINES=1' "$file")" "$(grep -c 'peer-selftest' "$file")" "$(grep -c '^GPU_CARDS_BY_PRIORITY' "$file")"; else printf 'none'; fi; }
printf 'enumeration-gate %s\n' "$*" >>"${STUB_CALLS:?}"
printf 'enumeration-env config=[%s]\n' "$(config_summary)" >>"$STUB_CALLS"
printf 'enumeration-env expand=[%s]\n' "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO-unset}" >>"$STUB_CALLS"
printf 'enumeration-env judge-expand=[%s]\n' "${SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO-unset}" >>"$STUB_CALLS"
if [[ -n "${STUB_CASE_LOG_DIRECTORY:-}" ]]; then   # 照真门禁：打每条用例的日志放哪个目录，各放一份带 CRASH_POINT_REPLAY_PARALLEL_FINISHED 的日志（scoped-domain-enumeration-records 格）
  mkdir -p -- "$STUB_CASE_LOG_DIRECTORY"
  echo "  · 这一趟每条用例的日志与发现日志放 ${STUB_CASE_LOG_DIRECTORY}/（log.<用例名> 与 log.<用例名>.findings.tsv，跑完不删）"
  previous=""
  for argument in "$@"; do
    [[ "$previous" == --item ]] && printf 'CRASH_POINT_REPLAY_PARALLEL_FINISHED states=%s slices=3 worker_threads=2 resumed_slices=0 freshly_run_slices=3\n' "${STUB_CASE_STATES:-1234}" >"$STUB_CASE_LOG_DIRECTORY/log.${argument#crash-case:}"
    previous="$argument"
  done
fi
previous=""
for argument in "$@"; do
  [[ "$previous" == --item && -n "${STUB_CRASH_POINT_REPLAY_REUSED:-}" ]] && echo "  · $argument 复用：全部块都有键相同的判定（自证）"
  previous="$argument"
done
[[ -n "${STUB_GATE_BETA_SLEEP:-}" && " $* " == *" crash-case:beta "* ]] && sleep "$STUB_GATE_BETA_SLEEP"
exit "${STUB_CRASH_POINT_REPLAY_EXIT:-0}"
STUB
  cat >"$root/scripts/gate/admission.py" <<'STUB'
# 假的准入模块（自证用）：记一行参数；这一份不该再调它（用例一级没有全绿标记可删），调了就在 whole 那一格判红
import os, sys
open(os.environ["STUB_CALLS"], "a").write("admission " + " ".join(sys.argv[1:]) + "\n")
STUB
  cat >"$work/bin/cargo" <<'STUB'
#!/usr/bin/env bash
# 假 cargo（自证用）：每次调记一行（参数、这一趟的显卡先后、是不是在清场包装之下、分片的段一趟判几个态、编译并行度、按段登记文件的路径）；按 --test 点名的目标打 libtest 的样子
# 只问版本的（scripts/gate/admission.py 算单元键时起 cargo -V）只答版本、不记调用
[[ "${1:-}" == -V || "${1:-}" == --version ]] && { echo "cargo 0.0.0-selftest"; exit 0; }
# 指定范围的崩溃域上限起的读库程序 items 子命令（scoped-domain-*、item-domain-line 与 item-domain-single-reader 那几格）：不记进调用记录（别的格数 cargo 调用不受影响），
#   items 之后的参数整串记一行进 STUB_ITEM_CALLS；照命令行上 --item 的次序每组答一行 CRASH_LIBRARY_ITEM：STUB_ITEM_TABLE 那份文件里有「<项名> <录下> <要现判> <能复用>」
#   一行的照它答，没有的按 STUB_ITEM_RECORDED、STUB_ITEM_TO_JUDGE、STUB_ITEM_REUSABLE（没给是 1、0、1：录下的块都对得上）答
if [[ " $* " == *" --bin crash_library_reader -- items "* ]]; then
  stub_joined="$*"   # ${*#…} 会对每个参数分别去前缀，先拼成一串再去
  [[ -z "${STUB_ITEM_CALLS:-}" ]] || printf 'items %s\n' "${stub_joined#* -- items }" >>"$STUB_ITEM_CALLS"
  stub_after_items=0; previous=""
  for argument in "$@"; do
    if (( stub_after_items )) && [[ "$previous" == --item ]]; then
      stub_counts="$(awk -v name="$argument" '$1 == name { print $2 "\t" $3 "\t" $4; exit }' "${STUB_ITEM_TABLE:-/dev/null}")"
      [[ -n "$stub_counts" ]] || stub_counts="${STUB_ITEM_RECORDED:-1}"$'\t'"${STUB_ITEM_TO_JUDGE:-0}"$'\t'"${STUB_ITEM_REUSABLE:-1}"
      IFS=$'\t' read -r stub_recorded stub_to_judge stub_reusable <<<"$stub_counts"
      printf 'CRASH_LIBRARY_ITEM\titem=%s\trecorded_blocks=%s\tto_judge=%s\treusable=%s\n' "$argument" "$stub_recorded" "$stub_to_judge" "$stub_reusable"
    fi
    [[ "$previous" == -- && "$argument" == items ]] && stub_after_items=1
    previous="$argument"
  done
  exit 0
fi
printf 'cargo %s cards=[%s] quiesced=[%s] slice-states=[%s] jobs=[%s] segments=[%s]\n' "$*" "${GPU_CARDS_BY_PRIORITY-unset}" "${GPU_QUIESCE_HOLDER:+yes}" \
  "${SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES-unset}" "${CARGO_BUILD_JOBS-unset}" "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" >>"${STUB_CALLS:?}"
config_summary() { local file="${SINGLEFS_MULTI_HOST_CONFIG:-}"; if [[ -f "$file" ]]; then printf 'across=%s peer=%s cards=%s' "$(grep -c '^ENABLE_ACROSS_MACHINES=1' "$file")" "$(grep -c 'peer-selftest' "$file")" "$(grep -c '^GPU_CARDS_BY_PRIORITY' "$file")"; else printf 'none'; fi; }
printf 'cargo-env config=[%s]\n' "$(config_summary)" >>"$STUB_CALLS"
printf 'cargo-env expand=[%s]\n' "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO-unset}" >>"$STUB_CALLS"
printf 'cargo-env judge-expand=[%s]\n' "${SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO-unset}" >>"$STUB_CALLS"
# --list-items --with-keys 的读库程序（keys-reader-same-build 格）：问到 crash_library_reader 就把 STUB_KEYS_READER 那份文件当它的输出
if [[ " $* " == *" --bin crash_library_reader "* ]]; then cat -- "${STUB_KEYS_READER:?}"; exit 0; fi
# 编译目录那几格（run-target-*）：记下收到的 CARGO_TARGET_DIR、往里写一份产物；给了 STUB_SLEEP 每一趟睡这么多秒（被 TERM 停那一格）
if [[ -n "${STUB_TARGET_LOG:-}" ]]; then
  printf '%s\n' "${CARGO_TARGET_DIR-unset}" >>"$STUB_TARGET_LOG"
  [[ -z "${CARGO_TARGET_DIR:-}" ]] || { mkdir -p "$CARGO_TARGET_DIR/release" && echo built >"$CARGO_TARGET_DIR/release/artifact"; }
fi
[[ -z "${STUB_SLEEP:-}" ]] || sleep "$STUB_SLEEP"
# --no-run：只编不跑，退 STUB_BUILD_EXIT（没设退 0）
if [[ " $* " == *" --no-run "* ]]; then
  exit "${STUB_BUILD_EXIT:-0}"
fi
target=""; previous=""
for argument in "$@"; do [[ "$previous" == --test ]] && target="$argument"; previous="$argument"; done
# 按条复用那几格（STUB_UNIT_ROOT 设了、那份小仓里有 tests/<目标>.rs，或目录式目标的 tests/<目标>/main.rs）：照那份测试文件里 #[test] 下一行的 fn 名打 libtest 的样子（先打 cargo 的 Running 行），
#   照 -- 之后的 --skip 跳过、照 STUB_RED_TESTS 判红，汇总行的数照实
unit_test_file="${STUB_UNIT_ROOT:-/nonexistent}/crates/singlefs-verification/tests/$target.rs"; [[ -f "$unit_test_file" ]] || unit_test_file="${unit_test_file%.rs}/main.rs"
if [[ -n "${STUB_UNIT_ROOT:-}" && -f "$unit_test_file" ]]; then
  skipped=(); ran=(); filtered=0; failed=0; previous=""
  for argument in "$@"; do [[ "$previous" == --skip ]] && skipped+=("$argument"); previous="$argument"; done
  while IFS= read -r name; do
    [[ -n "$name" ]] || continue
    if [[ " ${skipped[*]-} " == *" $name "* ]]; then filtered=$((filtered + 1)); else ran+=("$name"); fi
  done < <(awk 'previous ~ /#\[test\]/ && match($0, /fn [a-z0-9_]+/) { print substr($0, RSTART + 3, RLENGTH - 3) } { previous = $0 }' "$unit_test_file")
  printf '     Running %s (target/release/deps/%s-0000000000000000)\n\nrunning %d tests\n' "${unit_test_file#*/crates/singlefs-verification/}" "$target" "${#ran[@]}"
  for name in ${ran[@]+"${ran[@]}"}; do
    if [[ " ${STUB_RED_TESTS:-} " == *" $name "* ]]; then printf 'test %s ... FAILED\n' "$name"; failed=$((failed + 1)); else printf 'test %s ... ok\n' "$name"; fi
  done
  if (( failed )); then
    printf '\ntest result: FAILED. %d passed; %d failed; 0 ignored; 0 measured; %d filtered out\n' "$((${#ran[@]} - failed))" "$failed" "$filtered"
    exit 101
  fi
  printf '\ntest result: ok. %d passed; 0 failed; 0 ignored; 0 measured; %d filtered out\n' "${#ran[@]}" "$filtered"
  exit 0
fi
if [[ " ${STUB_RUN_CONDITION_TARGETS:-} " == *" $target "* ]]; then   # 给定状态表的判器借不到显存：引擎打那一行后报错（cargo 退 101）
  printf 'running 1 test\nCRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET table=stub-table reason=配了 GPU，给定状态表的判器借不到显存（自证）\ntest given ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n'; exit 101
fi
if [[ " ${STUB_RED_TARGETS:-} " == *" $target "* ]]; then
  printf 'running 1 test\ntest broken ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out\n'; exit 101
fi
if [[ " ${STUB_ZERO_TARGETS:-} " == *" $target "* || " ${STUB_ZERO_TARGETS:-} " == *" all "* ]]; then
  printf 'running 0 tests\n\ntest result: ok. 0 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out\n'; exit 0
fi
if [[ " ${STUB_NOT_RUN_TARGETS:-} " == *" $target "* ]]; then
  printf 'running 1 test\nNOT_RUN_NO_GPU_ADAPTER test=gpu_pair reason=自证\ntest gpu_pair ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'; exit 0
fi
if [[ " ${STUB_NOT_RUN_ON_A_TEST_LINE_TARGETS:-} " == *" $target "* ]]; then   # 几条同时跑：打这一行的 gpu_pair 还没出结果，这一行接在别的测试的结果行起头后面
  printf 'running 2 tests\ntest other ... NOT_RUN_NO_GPU_ADAPTER test=gpu_pair reason=自证\nok\ntest gpu_pair ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'; exit 0
fi
if [[ " ${STUB_REUSE_TARGETS:-} " == *" $target "* ]]; then   # 走判定库的用例：跑完那一行带库里复用的块（resumed_slices=3）与这一趟判的块
  printf 'running 2 tests\nCRASH_POINT_REPLAY_PARALLEL_FINISHED states=80 slices=8 worker_threads=2 configured_worker_threads=2 worker_threads_source=environment_variable resumed_slices=3 freshly_run_slices=5 progress_file_after_completion=none elapsed_seconds=0.1 judge=cpu\ntest first ... ok\ntest second ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'; exit 0
fi
[[ -n "${STUB_FLOW_LINE:-}" && "$target" == alpha ]] && echo 'CRASH_FLOW_COVERAGE entry=selftest_entry step=UnitWrite segment=0 expansion=expanded'   # flow-coverage-table 那一格
printf 'running 2 tests\ntest first ... ok\ntest second ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'
STUB
  chmod +x "$work/bin/cargo"
  if [[ "$(PATH="$work/bin:$PATH" type -P cargo)" != "$work/bin/cargo" ]]; then
    echo "  ✗ 自证 setup：PATH 最前面放了假 cargo，解到的却不是它（$(PATH="$work/bin:$PATH" type -P cargo)），不起子进程（起了就是真 cargo）" >&2
    echo "  → 怎么办：看 $work/bin/cargo 可不可执行、TMPDIR 是不是挂成了 noexec" >&2
    exit 1
  fi
  # 假 python3（只在 record-text-clipped-in-character 一格放进 PATH 最前面）：plan_test_functions 起的是这份脚本所在仓的真 admission.py（准入模块不随 CRASH_POINT_REPLAY_ROOT 换），
  #   小仓里的假准入模块碰不到它；这一格要它的 test-functions-request 没成，只能在 python3 这一层拦：被问 admission.py 的 test-functions-request 且设了 STUB_REQUEST_FAILURE_FILE，
  #   原样打这份文件的字节、退 1；别的调用（脚本里其余的 python3）原样交给真的 python3（STUB_REAL_PYTHON3）
  local request_failure_bin="$work/request-failure-bin" real_python3
  real_python3="$(type -P python3)"
  mkdir -p "$request_failure_bin"
  printf '%s\n' '#!/usr/bin/env bash' \
    'if [[ "${1:-}" == */admission.py && "${2:-}" == test-functions-request && -n "${STUB_REQUEST_FAILURE_FILE:-}" ]]; then cat -- "$STUB_REQUEST_FAILURE_FILE"; exit 1; fi' \
    'exec "${STUB_REAL_PYTHON3:?}" "$@"' >"$request_failure_bin/python3"
  chmod +x "$request_failure_bin/python3"
  if [[ -z "$real_python3" || "$(PATH="$request_failure_bin:$PATH" type -P python3)" != "$request_failure_bin/python3" ]]; then
    echo "  ✗ 自证 setup：PATH 最前面放了假 python3，解到的却不是它（$(PATH="$request_failure_bin:$PATH" type -P python3)；真的 python3：$real_python3）" >&2
    echo "  → 怎么办：看 PATH 上有没有 python3、$request_failure_bin/python3 可不可执行、TMPDIR 是不是挂成了 noexec" >&2
    exit 1
  fi
  # 别的格的小仓 $root 也做成 git 工作树的顶（文件头「只供自证」）：测试函数要被判的根是 git 工作树的顶才有判定库的名字，不是就整趟退 2（plan_test_functions 的 nolibrary）。
  #   加一个只有 singlefs-verification 一个包的小工作区（真的 admission.py test-plan 读它；测试文件是空的，计划里一条测试函数都没有），git init 提交一次；
  #   包清单留一份底稿，rocksdb-entry-refused 那一格临时换掉、跑完照底稿换回
  printf '%s\n' '[workspace]' 'members = ["crates/singlefs-verification"]' 'resolver = "2"' >"$root/Cargo.toml"
  printf '%s\n' 'version = 4' '' '[[package]]' 'name = "singlefs-verification"' 'version = "0.0.0"' >"$root/Cargo.lock"
  printf '%s\n' '[package]' 'name = "singlefs-verification"' 'version = "0.0.0"' 'edition = "2021"' '' '[features]' 'gpu = []' >"$root/crates/singlefs-verification/Cargo.toml"
  cp -- "$root/crates/singlefs-verification/Cargo.toml" "$work/root-crate-manifest.toml"
  mkdir -p "$root/crates/singlefs-verification/src"
  printf '%s\n' 'pub fn root_helper() -> u32 {' '    1' '}' >"$root/crates/singlefs-verification/src/lib.rs"
  # 引擎放量展开上限常量（指定范围的上限判法读它认哪份记录是全域：lib_crash_point_replay_items.py 的 engine_expand_constant）；不是 lib.rs 的模块，不进任何测试函数的键
  printf '%s\n' 'pub const EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION: usize = 30;' >"$root/crates/singlefs-verification/src/crash_amplification.rs"
  if ! { git -C "$root" init -q -b master && git -C "$root" add -A && git -C "$root" -c core.hooksPath=/dev/null -c user.name=selftest -c user.email=selftest@invalid commit -qm root; } >"$work/root-git.log" 2>&1; then
    echo "  ✗ 自证 setup：小仓 $root 做不成 git 仓（$(tr '\n' ' ' <"$work/root-git.log" | selftest_clip_text 300)）" >&2
    echo "  → 怎么办：看 git 在不在 PATH 上、TMPDIR 可不可写" >&2
    exit 1
  fi
  # 按条复用那几格（文件头「按条复用」：reuse-three-questions 的测试函数那一半、test-functions-reuse-shared-with-the-gate、red-rejudged、test-functions-plan-failure-red）的小仓 unit-root：
  #   照 $root 拷一份（不带它的 .git：unit-root 自己另做一份 git 仓，判定库的名字与 $root 的不同），加一个只有 singlefs-verification 一个包的小工作区（三个测试目标：alpha 两条、beta 一条（目录式：tests/beta/main.rs）、gamma_amp 一条，alpha_reads_the_helper 引库里的 alpha_helper），
  #   git init 提交一次（判定库在它的 git common-dir 下），放一个空的 .unit-verdict-sample/（admission.py 文件头「复用哨兵」：样本根不抽哨兵，现判与复用的条数钉死）；
  #   真的 scripts/gate/admission.py 在它上面算键、查库、记库。unit-bin 里放只答版本的假 rustc（算键时起 rustc -vV），这几格的 PATH 把它放最前面
  local unit_root="$work/unit-root" unit_crate
  mkdir -p "$work/unit-bin"
  printf '%s\n' '#!/usr/bin/env bash' '# 假 rustc（自证用）：只答版本' \
    'case "${1:-}" in -V) echo "rustc 0.0.0-selftest" ;; -vV) printf "rustc 0.0.0-selftest\nbinary: rustc\nhost: x86_64-unknown-linux-gnu\nrelease: 0.0.0-selftest\n" ;; esac' 'exit 0' >"$work/unit-bin/rustc"
  chmod +x "$work/unit-bin/rustc"
  cp -dR --preserve=timestamps,links -- "$root" "$unit_root"
  rm -rf -- "${unit_root:?}/.git"
  unit_crate="$unit_root/crates/singlefs-verification"
  mkdir -p "$unit_crate/src" "$unit_root/.unit-verdict-sample"
  printf '%s\n' '[workspace]' 'members = ["crates/singlefs-verification"]' 'resolver = "2"' >"$unit_root/Cargo.toml"
  printf '%s\n' 'version = 4' '' '[[package]]' 'name = "singlefs-verification"' 'version = "0.0.0"' >"$unit_root/Cargo.lock"
  printf '%s\n' '[package]' 'name = "singlefs-verification"' 'version = "0.0.0"' 'edition = "2021"' '' '[features]' 'gpu = []' >"$unit_crate/Cargo.toml"
  printf '%s\n' 'pub fn alpha_helper() -> u32 {' '    1' '}' '' 'pub fn not_read_by_any_test() -> u32 {' '    7' '}' >"$unit_crate/src/lib.rs"
  cp -- "$unit_crate/src/lib.rs" "$work/unit-lib.rs"
  printf '%s\n' 'use singlefs_verification::alpha_helper;' '' '#[test]' 'fn alpha_reads_the_helper() {' '    assert_eq!(alpha_helper(), 1);' '}' '' '#[test]' 'fn alpha_stands_alone() {' '    assert_eq!(2, 2);' '}' \
    >"$unit_crate/tests/alpha.rs"
  mkdir -p "$unit_crate/tests/beta"; rm -f -- "$unit_crate/tests/beta.rs"; printf '%s\n' '#[test]' 'fn beta_stands_alone() {' '    assert_eq!(3, 3);' '}' >"$unit_crate/tests/beta/main.rs"
  printf '%s\n' '#[test]' 'fn gamma_stands_alone() {' '    assert_eq!(4, 4);' '}' >"$unit_crate/tests/gamma_amp.rs"
  if ! { git -C "$unit_root" init -q -b master && git -C "$unit_root" add -A && git -C "$unit_root" -c core.hooksPath=/dev/null -c user.name=selftest -c user.email=selftest@invalid commit -qm unit-root; } >"$work/unit-root-git.log" 2>&1; then
    echo "  ✗ 自证 setup：按条复用那几格的小仓 $unit_root 做不成 git 仓（$(tr '\n' ' ' <"$work/unit-root-git.log" | selftest_clip_text 300)）" >&2
    echo "  → 怎么办：看 git 在不在 PATH 上、TMPDIR 可不可写" >&2
    exit 1
  fi
  # 多机配置两份：没开 GPU（只写假清场命令）、开了 GPU（加显卡先后两份）；清场往调用记录记一笔
  printf '%s\n' "QUIESCE_STOP_COMMAND='echo quiesce-stop >> \"\$STUB_CALLS\"'" "QUIESCE_STOPPED_CHECK_COMMAND=true" >"$work/multi-host-gpu-off.env"
  #   开了 GPU 的那一份要过入口文件头 run-condition 的配置判法（crash_amplification_run_condition_holds 经 crash-point-replay-shard-configuration-check.sh --emit-addresses 判整份），
  #   键照那份判法的自证样本写全：127.0.0.1 在每一台的网卡上（lo）、是这一台，192.0.2.1 是文档地址段、是另一台；显卡先后的地址取 HOSTS 里的 127.0.0.1
  { cat "$work/multi-host-gpu-off.env"
    printf '%s\n' 'QUIESCE_START_COMMAND=' 'QUIESCE_STARTED_CHECK_COMMAND=' 'HOSTS=127.0.0.1 192.0.2.1' \
      'CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.1=/selftest/peer-cargo' \
      'BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.1=/selftest/peer-builds' 'SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 192.0.2.1=1G' \
      "NFS_SHARED_ROOTS=$work" ENABLE_ACROSS_MACHINES=0 ENABLE_GPU=1 'GPU_CARDS_BY_PRIORITY="127.0.0.1:0:8G"' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="127.0.0.1:0:24G"'
  } >"$work/multi-host-gpu-on.env"
  export GPU_QUIESCE_STATE_DIRECTORY="$work/gpu-quiesce-state" GPU_QUIESCE_LOG_DIRECTORY="$work/gpu-quiesce-logs"
  unset STUB_AMP_PERSISTENT STUB_AMP_SERVICE STUB_REUSE_TARGETS
  unset GATE_NOT_RUN_FILE STUB_RUN_CONDITION_TARGETS STUB_PEER_EXIT STUB_RED_TARGETS STUB_ZERO_TARGETS STUB_NOT_RUN_TARGETS STUB_AMP_DROP STUB_AMP_RED STUB_AMP_NOT_RUN STUB_AMP_NOT_RUN_REASON_TAIL STUB_REQUEST_FAILURE_FILE STUB_AMP_EXIT STUB_CRASH_POINT_REPLAY_EXIT STUB_CRASH_POINT_REPLAY_REUSED STUB_BUILD_EXIT

  # 这一趟之外的输出（调用记录、子进程的输出）照字节截、照字节转打，三行整组过 selftest_valid_text：自证这几行的标准错误逐字节是合法的 UTF-8（门禁 break-switch-replay 判开关趟时不许有坏字节）
  cell_failed() { # <格名> <判错的一句> <出路>
    {
      echo "  ✗ 自证 $1：$2（退 $exit_code；调用记录：$(tr '\n' '|' <"$calls" | selftest_clip_text 600)）"
      echo "     → 怎么办：$3"
      # 这一格那一趟子进程打的红行与输出末尾（只读不判）：退出码不对的格看得出是哪一段判的红，不用再复跑一次去找
      echo "     那一趟打的红行：$(grep -F '✗' "$work/out" 2>/dev/null | head -n 5 | tr '\n' '|' | selftest_clip_text 600)；输出末尾：$(tail -n 5 "$work/out" 2>/dev/null | tr '\n' '|' | selftest_clip_text 600)"   # gate-lint:detail
    } | selftest_valid_text >&2
    failures=$((failures + 1))
  }
  # 选格（文件头「选格」）：selection_named 是点名的格，selection_seen 是这一趟问到过的格（收尾拿它认点名的格写没写错）
  local -A selection_named=() selection_seen=()
  local selection_word selection_unknown=""
  for selection_word in ${replay_selftest_cells//,/ }; do selection_named["$selection_word"]=1; done
  selftest_cell_wanted() { # <格名…>：这一块跑不跑。没选格的一律跑；选了格的，这一块的格名里有一个是点名的才跑。弄坏开关 selftest-cells-run-extra 下 unknown-item 那一块也跑
    local cell wanted=1
    if [[ -z "$replay_selftest_cells" ]]; then return 0; fi
    for cell in "$@"; do
      selection_seen["$cell"]=1
      if [[ -n "${selection_named[$cell]:-}" ]]; then wanted=0; fi
      if replay_break_is selftest-cells-run-extra && [[ "$cell" == unknown-item ]]; then wanted=0; fi
    done
    return "$wanted"
  }
  entry() { # [<变量=值>…] -- <参数…>：在小仓上起这一份（子进程认自证标记），输出进 $work/out、退出码进 exit_code
    local settings=()
    while (($#)) && [[ "$1" != -- ]]; do settings+=("$1"); shift; done
    shift
    : >"$calls"
    cells=$((cells + 1))
    # 服务交下来的 CARGO_BUILD_JOBS 不带进子进程：test-functions-threads-from-service 那一格自己设一个认得出的值，别的格桩 cargo 记 unset
    env -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP -u SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE -u SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES \
      -u GPU_QUIESCE_ORIGIN_READ_BACK -u GPU_CARDS_BY_PRIORITY -u SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO -u SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO \
      -u CARGO_BUILD_JOBS \
      PATH="$work/bin:$PATH" STUB_CALLS="$calls" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
      SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" CRASH_POINT_REPLAY_SELFTEST_MARK="$work/mark" CRASH_POINT_REPLAY_ROOT="$root" \
      CRASH_POINT_REPLAY_BREAK="$replay_break" ${settings[@]+"${settings[@]}"} bash "$replay_script" "$@" >"$work/out" 2>&1
    exit_code=$?
  }
  calls_have() { grep -qE -- "$1" "$calls"; }
  # 编译目录那几格（文件头「编译目录」）：调用方的 CARGO_TARGET_DIR 不带进子进程（只有 run-target-caller 那一格自己设）
  unset CARGO_TARGET_DIR
  calls_count() { grep -cE -- "$1" "$calls"; }
  out_has() { grep -qF -- "$1" "$work/out"; }
  item_line() { grep -E "^CRASH_POINT_REPLAY_ITEM item=$1 " "$work/out"; }
  item_has() { grep -qE "^CRASH_POINT_REPLAY_ITEM item=$1 .*$2" "$work/out"; }   # <项> <那一行里要有的一段>（不接管道：pipefail 下 grep -q 收尾的管道会把命中读成没命中）

  # 整趟才跑的那几类的测试目标个数，从 WHOLE_RUN_GROUPS 现数（期望与名单同源：名单加减一个名字，自证不用跟着改）
  local extras_count=0 whole_run_line whole_run_name whole_run_targets whole_run_words
  for whole_run_line in "${WHOLE_RUN_GROUPS[@]}"; do
    IFS=$'\t' read -r whole_run_name whole_run_targets <<<"$whole_run_line"
    [[ "$whole_run_targets" == - ]] && continue
    read -r -a whole_run_words <<<"$whole_run_targets"
    extras_count=$((extras_count + ${#whole_run_words[@]}))
  done
  if selftest_cell_wanted list-items; then
  # list-items：退 0、三行、不起 cargo 与驱动
  entry -- --list-items
  if [[ "$exit_code" != 0 || "$(grep -cE $'^(alpha|beta|gamma)\t测试函数 ' "$work/out")" != 3 ]] || [[ -s "$calls" ]] \
     || [[ "$(grep -c '^whole-run:' "$work/out")" != "${#WHOLE_RUN_GROUPS[@]}" ]] || ! out_has $'alpha\t测试函数 alpha ∪ 崩溃点重放 crash-case:alpha ∪ 放量 alpha-amp'; then
    cell_failed list-items "--list-items 应当退 0、逐项列出（含整趟才跑的几类 whole-run:<类>）、不起 cargo 与驱动" "看 --list-items 那一段：只读合表、打表就退"
  fi
  fi
  # list-items-with-keys：读库程序的输出取它文件头「输出」那一段原样喂进来（写法与读法钉在同一段字上）⇒ 退 0、三项照列，
  # 报录下的块 6、键对不上的块 4（只在别的判法单元指纹下判过 3、一种判器的结论都没有 1）、键相同的 2，与镜像判器（image_judgement）那一把指纹；不起 cargo。
  #   库的名字照那一段 CRASH_LIBRARY_KEYS 行的 library= 现取（判定库在 prune-server 里，读库程序报哪一处由它定，这里不抄死）
  local keys_reader_source="$replay_script_directory/../../crates/singlefs-verification/src/bin/crash_library_reader.rs"
  sed -n '/^\/\/! keys-format-begin$/,/^\/\/! keys-format-end$/p' "$keys_reader_source" | sed '1d;$d' | sed 's|^//! ||' >"$work/keys-reader-output"
  local keys_library_name
  keys_library_name="$(awk -F'\t' '$1 == "CRASH_LIBRARY_KEYS" { sub(/^library=/, "", $2); print $2; exit }' "$work/keys-reader-output")"
  if selftest_cell_wanted list-items-with-keys; then
  entry CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys
  if [[ "$exit_code" != 0 || "$(grep -cE $'^(alpha|beta|gamma)\t测试函数 ' "$work/out")" != 3 ]] || [[ -s "$calls" || -z "$keys_library_name" ]] \
     || ! out_has $'键\t崩溃判定库 '"$keys_library_name"$'：录下的块 6，键对不上的块 4（只在别的判法单元指纹下判过 3、一种判器的结论都没有 1），键相同的 2' \
     || ! out_has $'键\timage_judgement 这一份的判法单元指纹 0505050505050505'; then
    cell_failed list-items-with-keys "--list-items --with-keys 应当照读库程序文件头「输出」那一段的写法报键对不上的块数、不起 cargo" \
      "看 report_the_keys_of_the_library 与 crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」那一段"
  fi
  # 按态数那一行（文件头「--list-items --with-keys」）：同一份输出后面接一行 CRASH_LIBRARY_STATES ⇒ 退 0、另打态数那一句（四个数照原样）；
  #   那一行字段写坏（少了 states_never_judged=）⇒ 退 1、打认不出那一句；弄坏开关 keys-states-line-unread、keys-states-line-unchecked 各弄坏一处
  { cat -- "$work/keys-reader-output"; printf 'CRASH_LIBRARY_STATES\trecorded_states=83204785477\tstates_judged_under_the_current_key=35893860\tstates_judged_only_under_another_key=5980821966\tstates_never_judged=77188069651\n'; } \
    >"$work/keys-reader-output-with-states"
  entry CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output-with-states" -- --list-items --with-keys
  if [[ "$exit_code" != 0 || -s "$calls" ]] \
     || ! out_has $'键\t崩溃判定库 '"$keys_library_name"$' 按态数：录下的态 83204785477，当前键下判过的 35893860、只在别的键下判过的 5980821966、一种判器的结论都没有的 77188069651'; then
    cell_failed list-items-with-keys "--list-items --with-keys 遇到读库程序按态数那一行，应当另打一句四个态数、退 0、不起 cargo" \
      "看 report_the_keys_of_the_library 里 CRASH_LIBRARY_STATES 那一支"
  fi
  { cat -- "$work/keys-reader-output"; printf 'CRASH_LIBRARY_STATES\trecorded_states=6\tstates_judged_under_the_current_key=2\tstates_judged_only_under_another_key=3\n'; } \
    >"$work/keys-reader-output-states-malformed"
  entry CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output-states-malformed" -- --list-items --with-keys
  if [[ "$exit_code" != 1 ]] || ! out_has '读库程序按态数那一行认不出'; then
    cell_failed list-items-with-keys "--list-items --with-keys 遇到字段写坏的按态数那一行，应当照实报认不出、退 1" \
      "看 report_the_keys_of_the_library 里 CRASH_LIBRARY_STATES 那一支的字段核对"
  fi
  fi
  if selftest_cell_wanted keys-reader-same-build; then
  # keys-reader-same-build：--list-items --with-keys 的读库程序与真跑编同一套特性（文件头「--list-items --with-keys」）：① 特性串 = 真跑带的特性加上 验证包清单
  #   [dev-dependencies] 里每个带特性的依赖的 <依赖>/<特性>（cargo test 的树里打开、cargo run 的树里没有的那几个），没有带特性的依赖就只剩真跑带的；
  #   ② 开了 GPU 的小仓里，真跑测试函数那一趟带的 --features 与读库程序那一趟 cargo run 带的相同
  local keys_wrong="" keys_manifest="$work/keys-manifest.toml" keys_test_features keys_reader_features_seen
  printf '%s\n' '[package]' 'name = "singlefs-verification"' '[dev-dependencies]' \
    'singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters", "second-sample"] }' 'plain = "1"' >"$keys_manifest"
  [[ "$(keys_reader_features "$keys_manifest" gpu)" == "gpu,singlefs-core/lookup-work-counters,singlefs-core/second-sample" ]] \
    || keys_wrong+="特性串没把 dev-dependencies 里带特性的依赖算进去（得到「$(keys_reader_features "$keys_manifest" gpu)」）；"
  [[ -z "$(keys_reader_features "$root/crates/singlefs-verification/Cargo.toml" "")" ]] || keys_wrong+="没有 dev-dependencies、没开 GPU 的清单应当得到空串；"
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" GPU_CARDS_BY_PRIORITY=127.0.0.1:1:20G --
  keys_test_features="$(sed -nE 's/^cargo test --release -p singlefs-verification --features ([^ ]+) .*/\1/p' "$calls" | head -n 1)"
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" STUB_KEYS_READER="$work/keys-reader-output" -- --list-items --with-keys
  keys_reader_features_seen="$(sed -nE 's/^cargo run --release --quiet --offline -p singlefs-verification --features ([^ ]+) --bin crash_library_reader .*/\1/p' "$calls" | head -n 1)"
  [[ "$exit_code" == 0 && -n "$keys_test_features" && "$keys_reader_features_seen" == "$keys_test_features" ]] \
    || keys_wrong+="真跑测试函数带的特性「$keys_test_features」与读库程序带的「$keys_reader_features_seen」不同（--list-items --with-keys 退 $exit_code）；"
  [[ -z "$keys_wrong" ]] || cell_failed keys-reader-same-build "读库程序应当与真跑编同一套特性：$keys_wrong" "看 report_the_keys_of_the_library 里 keys_reader_features 那一段与文件头「--list-items --with-keys」"
  fi
  # 按件分片的计划目录（默认分片那几格用）
  local shard_root="$work/shard"
  shard_plan_settings() { printf '%s\n' TASK_SERVER_SHARD_PLAN_DIRECTORY="$shard_root/plan"; }
  # 默认分片那几格（文件头「经 task-server 按件分片」）：小仓合表的件是 crash-case:alpha、alpha-amp、crash-case:beta、gamma-amp，加整趟才跑的那几类
  default_shard_plan() { # [<变量=值>…] -- <参数…>：新建计划
    local settings=()
    while (($#)) && [[ "$1" != -- ]]; do settings+=("$1"); shift; done
    shift
    rm -rf -- "${shard_root:?}"; mkdir -p -- "$shard_root"
    entry TASK_SERVER_SHARD_PHASE=plan "$(shard_plan_settings)" ${settings[@]+"${settings[@]}"} -- "$@"
  }
  default_shard_count() { grep -o '"label"' "$shard_root/plan/plan.json" 2>/dev/null | wc -l; }
  default_shard_of() { awk -F'\t' -v object="$1" '$4 == object && $3 != "test-functions" { print $1; exit }' "$shard_root/plan/shard-plan.tsv" 2>/dev/null; }   # <件>：它在第几片
  default_shard_run() { # <片号> <片数> [<变量=值>…] -- <参数…>
    local number="$1" total="$2" settings=()
    shift 2
    while (($#)) && [[ "$1" != -- ]]; do settings+=("$1"); shift; done
    shift
    entry TASK_SERVER_SHARD_PHASE=shard "$(shard_plan_settings)" TASK_SERVER_SHARD="$number/$total" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$shard_root/output-$number" ${settings[@]+"${settings[@]}"} -- "$@"
  }
  default_shard_merge() { # <片数> <参数…>
    local number total="$1"
    shift
    for ((number = 0; number < total; number++)); do printf '%s\t192.0.2.1\t%s\n' "$number" "$shard_root/output-$number"; done >"$shard_root/outputs.tsv"
    entry TASK_SERVER_SHARD_PHASE=merge "$(shard_plan_settings)" TASK_SERVER_SHARD_OUTPUTS="$shard_root/outputs.tsv" -- "$@"
  }
  local default_count default_number default_wrong
  if selftest_cell_wanted shard-default-plan; then
  # shard-default-plan：不带参数（不带参数）也分：每件恰好落在一片，每片带上它的项的测试函数；不起测试函数与两套驱动
  default_shard_plan --
  default_count="$(default_shard_count)"
  if [[ "$exit_code" != 0 || "$default_count" -lt 4 ]] || calls_have '^cargo test ' || calls_have '^(pipeline-entry|enumeration-gate)' \
     || [[ "$(awk -F'\t' '$3 != "test-functions" && ($4 == "crash-case:alpha" || $4 == "alpha-amp" || $4 == "crash-case:beta" || $4 == "gamma-amp") { print $4 }' "$shard_root/plan/shard-plan.tsv" 2>/dev/null | LC_ALL=C sort | tr '\n' ' ')" != "alpha-amp crash-case:alpha crash-case:beta gamma-amp " ]] \
     || ! awk -F'\t' -v shard="$(default_shard_of crash-case:beta)" '$1 == shard && $2 == "beta" && $3 == "test-functions" { found = 1 } END { exit !found }' "$shard_root/plan/shard-plan.tsv" 2>/dev/null; then
    cell_failed shard-default-plan "不带参数应当默认分片：每件恰好落在一片、每片带上它的项的测试函数，计划那一步不起测试函数与两套驱动（退 $exit_code，$default_count 片）" "看 shard_write_the_plan 与文件头第 2 行的声明"
  fi
  fi
  if selftest_cell_wanted shard-default-one-part; then
  # shard-default-one-part：一片只跑自己的件（crash-case:beta 那一片：只起它与 beta 的测试函数）
  default_shard_plan --
  default_count="$(default_shard_count)"
  default_shard_run "$(default_shard_of crash-case:beta)" "$default_count" --
  if [[ "$exit_code" != 0 ]] || ! calls_have '^enumeration-gate --item crash-case:beta ' || calls_have 'crash-case:alpha' || calls_have '^pipeline-entry ' \
     || [[ ! -f "$shard_root/output-$(default_shard_of crash-case:beta)/shard-finished" ]] || ! calls_have '^cargo test --release -p singlefs-verification --no-run --test beta cards='; then
    cell_failed shard-default-one-part "默认分片的一片应当只跑自己的件与它的项的测试函数、存下 shard-finished（退 $exit_code）" "看主流程 shard_phase == shard 那两支与 shard_read_the_plan"
  fi
  fi
  if selftest_cell_wanted shard-default-merge-red; then
  # shard-default-merge-red：崩溃点重放那一道全红：每片照退 0，别的片照跑；合并不起测试函数与两套驱动，判出红、退 1；ITEM 与 TOTAL 同不分片
  default_shard_plan --
  default_count="$(default_shard_count)"
  default_wrong=""
  for ((default_number = 0; default_number < default_count; default_number++)); do
    default_shard_run "$default_number" "$default_count" STUB_CRASH_POINT_REPLAY_EXIT=1 --
    [[ "$exit_code" == 0 ]] || default_wrong+="第 $default_number 片退 $exit_code；"
  done
  default_shard_merge "$default_count"
  if [[ -n "$default_wrong" || "$exit_code" != 1 ]] || calls_have '^cargo test ' || calls_have '^(pipeline-entry|enumeration-gate)' \
     || ! item_has beta 'verdict=red' || ! item_has gamma 'verdict=green' || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL scope=whole .* red=2 ' "$work/out"; then
    cell_failed shard-default-merge-red "判红的片应当退 0、别的片照跑，合并判出红退 1、不起测试函数与两套驱动（${default_wrong}合并退 $exit_code）" "看 shard_collect_the_outputs 与收尾把一片的 1 改成 0 那一段"
  fi
  fi
  if selftest_cell_wanted shard-default-merge-green; then
  # shard-default-merge-green：全绿：合并退 0、TOTAL 报 0 红
  default_shard_plan --
  default_count="$(default_shard_count)"
  for ((default_number = 0; default_number < default_count; default_number++)); do default_shard_run "$default_number" "$default_count" --; done
  default_shard_merge "$default_count"
  if [[ "$exit_code" != 0 ]] || ! item_has alpha 'verdict=green' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL scope=whole .* red=0 ' "$work/out"; then
    cell_failed shard-default-merge-green "全绿的默认分片，合并应当退 0、TOTAL 报 0 红（退 $exit_code）" "看 shard_collect_the_outputs 与 summarize"
  fi
  fi
  if selftest_cell_wanted shard-default-many-parts; then
  # shard-default-many-parts：上限压到 2：件都合进 2 片、一件不丢，合成的片跑它的几件，合并照判
  default_shard_plan CRASH_POINT_REPLAY_MOST_SHARDS=2 --
  default_count="$(default_shard_count)"
  if [[ "$exit_code" != 0 || "$default_count" != 2 ]] \
     || [[ "$(awk -F'\t' '$3 != "test-functions" && ($4 == "crash-case:alpha" || $4 == "alpha-amp" || $4 == "crash-case:beta" || $4 == "gamma-amp") { print $4 }' "$shard_root/plan/shard-plan.tsv" 2>/dev/null | LC_ALL=C sort | tr '\n' ' ')" != "alpha-amp crash-case:alpha crash-case:beta gamma-amp " ]]; then
    cell_failed shard-default-many-parts "件数超过片数上限应当合成上限那么多片、一件不丢（退 $exit_code，$default_count 片）" "看 shard_write_the_plan 定片数那一段"
  else
    default_shard_run 0 2 --
    default_wrong="第 0 片退 $exit_code、起了 $(calls_count '^(pipeline-entry|enumeration-gate) ') 件"
    if [[ "$exit_code" != 0 ]] || [[ "$(calls_count '^(pipeline-entry|enumeration-gate) ')" -lt 2 ]]; then
      cell_failed shard-default-many-parts "合成的那一片应当跑它的几件（$default_wrong）" "看主流程一片逐行起崩溃点重放那一支与 run_pipeline_part"
    fi
  fi
  fi
  if selftest_cell_wanted shard-position-refused; then
  # shard-position-refused：片号不在计划里、片数对不上、阶段认不得 ⇒ 都退 2、一件都不跑
  default_shard_plan --
  default_count="$(default_shard_count)"
  local shard_position_wrong=""
  default_shard_run 99 "$default_count" --; [[ "$exit_code" == 2 ]] && ! calls_have '^(pipeline-entry|enumeration-gate)' || shard_position_wrong+="片号 99 退 $exit_code；"
  default_shard_run 0 999 --; [[ "$exit_code" == 2 ]] && ! calls_have '^(pipeline-entry|enumeration-gate)' || shard_position_wrong+="片数 999 退 $exit_code；"
  entry TASK_SERVER_SHARD_PHASE=no-such-phase "$(shard_plan_settings)" --
  [[ "$exit_code" == 2 ]] && ! calls_have '^(pipeline-entry|enumeration-gate)' || shard_position_wrong+="认不得的阶段退 $exit_code；"
  [[ -z "$shard_position_wrong" ]] || cell_failed shard-position-refused "片号对不上计划或阶段认不得应当退 2、一件都不跑（$shard_position_wrong）" "看主流程 shard_phase == shard 那一支与参数之后核阶段那一段"
  fi
  if selftest_cell_wanted shard-merge-refuses-incomplete; then
  # shard-merge-refuses-incomplete：第 0 片缺 shard-finished ⇒ 合并退 1、不判
  default_shard_plan --
  default_count="$(default_shard_count)"
  for ((default_number = 0; default_number < default_count; default_number++)); do default_shard_run "$default_number" "$default_count" --; done
  rm -f -- "$shard_root/output-0/shard-finished"
  default_shard_merge "$default_count"
  if [[ "$exit_code" != 1 ]] || ! out_has "第 0 片的产物不全"; then
    cell_failed shard-merge-refuses-incomplete "有一片的产物不全，合并应当退 1、点名那一片（退 $exit_code）" "看 shard_collect_the_outputs 核 shard-finished 那一段"
  fi
  fi
  if selftest_cell_wanted unsharded-run-refused; then
  # unsharded-run-refused：不经服务起（没有 TASK_SERVER_SHARD_PHASE）照平常跑判 ⇒ 不带参数与带 --item 都退 2、什么都不起、出路写经 task-server/task submit 交
  local unsharded_wrong="" unsharded_arguments
  for unsharded_arguments in "" "--item alpha"; do
    # shellcheck disable=SC2086  # 参数是空格隔开的几个词，逐个交
    entry CRASH_POINT_REPLAY_UNSHARDED_REFUSED=1 -- $unsharded_arguments
    { [[ "$exit_code" == 2 && ! -s "$calls" ]] && out_has "不经服务起" && out_has "task-server/task submit"; } || unsharded_wrong+="「${unsharded_arguments:-不带参数}」退 $exit_code、起了 $(grep -c . "$calls") 条；"
  done
  [[ -z "$unsharded_wrong" ]] || cell_failed unsharded-run-refused "不经服务起应当退 2、什么都不起、出路写经 task-server/task submit 交（$unsharded_wrong）" "看参数之后核 TASK_SERVER_SHARD_PHASE 那一段"
  fi
  if selftest_cell_wanted unknown-item; then
  # unknown-item / missing-item-value / refused-options：用法错退 2、什么都不起
  entry -- --item no-such-item
  if [[ "$exit_code" != 2 ]] || ! out_has "alpha" || ! out_has "gamma" || ! out_has "whole-run:pruning-self-proof" || [[ -s "$calls" ]]; then
    cell_failed unknown-item "项名写错应当退 2、列出可点名的项（含 whole-run:<类>）、什么都不起" "看核项名那一段"
  fi
  fi
  if selftest_cell_wanted missing-item-value; then
  entry -- --item
  [[ "$exit_code" == 2 && ! -s "$calls" ]] || cell_failed missing-item-value "--item 后面缺项名应当退 2" "看参数那一段的 --item"
  fi
  if selftest_cell_wanted refused-options; then
  # refused-options：删掉的选项（整趟现判、清空复用、快速那一档、积压跑法、跟在 --item 后面的）一律当认不得的参数退 2、什么都不起、出路只写正确用法
  #   选项名用两段拼，不在这份文件里写出整串
  local refused_wrong="" refused_arguments refused_option
  for refused_option in "--""full" "--""start-over" "--""quick" "--""back""log" "--""light" "--""no-shard"; do
    for refused_arguments in "$refused_option" "--item beta $refused_option"; do
      # shellcheck disable=SC2086  # 参数是空格隔开的几个词，逐个交
      entry -- $refused_arguments
      { [[ "$exit_code" == 2 && ! -s "$calls" ]] && out_has "认不得的参数 $refused_option" && ! out_has "已删"; } || refused_wrong+="「$refused_arguments」退 $exit_code；"
    done
  done
  [[ -z "$refused_wrong" ]] || cell_failed refused-options "删掉的选项应当照认不得的参数退 2、什么都不起、不说已删（$refused_wrong）" "看参数那一段：只认 --list-items、--with-keys、--selftest、--item、--crash-expand-up-to"
  fi
  if selftest_cell_wanted expand-up-to-refused; then
  # expand-up-to-refused：--crash-expand-up-to 的值不是非负整数（abc、-1、后面是别的选项）、后面没有值，环境里的缩域变量写坏 ⇒ 都退 2、什么都不起
  local expand_wrong="" expand_case
  for expand_case in "--crash-expand-up-to abc" "--crash-expand-up-to -1" "--crash-expand-up-to --item" "--crash-expand-up-to"; do
    # shellcheck disable=SC2086  # 参数是空格隔开的几个词，逐个交
    entry -- $expand_case
    { [[ "$exit_code" == 2 && ! -s "$calls" ]] && out_has "用法不对"; } || expand_wrong+="「$expand_case」退 $exit_code；"
  done
  entry SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=abc --
  { [[ "$exit_code" == 2 && ! -s "$calls" ]]; } || expand_wrong+="环境变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 写 abc 退 $exit_code；"
  entry SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=abc --
  { [[ "$exit_code" == 2 && ! -s "$calls" ]]; } || expand_wrong+="环境变量 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO 写 abc 退 $exit_code；"
  [[ -z "$expand_wrong" ]] || cell_failed expand-up-to-refused "缩域的值不是非负整数、后面没有值都应当退 2、什么都不起、点名用法（$expand_wrong）" "看参数之后核缩域值的那一段（crash_expand_up_to_origin）"
  fi
  if selftest_cell_wanted expand-up-to-passed; then
  # expand-up-to-passed：给了 --crash-expand-up-to 6（或环境里已设变量）⇒ 测试函数、崩溃点重放、放量三段都收到同一个值，ITEM 与 TOTAL 行末尾接 expand_up_to=6，
  #   成功句写缩过域、不算门禁通过；参数与环境变量都给取参数；不缩的那一趟三段看到的是 unset，ITEM、TOTAL 行里与句里都没有缩域的说法
  #   （放量入口转打过来的 CRASH_AMPLIFICATION_ITEM 行照真入口的样子总带 expand_up_to=<引擎报的展开深度>，不算缩域的说法，不在这里判）
  local expand_passed_wrong=""
  expand_check() { # <放量那个变量的值> <说明>：上一趟 entry 的结果对不对；判器那个变量三段都收到同一个值（跟着缩域设的，行末标签只有 expand_up_to）
    local value="$1" cargo_runs
    cargo_runs="$(calls_count '^cargo-env expand=')"
    { [[ "$exit_code" == 0 ]] && (( cargo_runs > 0 )) && [[ "$(calls_count "^cargo-env expand=\[$value\]\$")" == "$cargo_runs" ]] \
      && calls_have "^enumeration-env expand=\[$value\]\$" && calls_have "^pipeline-env expand=\[$value\]\$" \
      && [[ "$(calls_count "^cargo-env judge-expand=\[$value\]\$")" == "$cargo_runs" ]] \
      && calls_have "^enumeration-env judge-expand=\[$value\]\$" && calls_have "^pipeline-env judge-expand=\[$value\]\$" \
      && grep -qE "^CRASH_POINT_REPLAY_TOTAL .* expand_up_to=$value\$" "$work/out" && grep -qE "^CRASH_POINT_REPLAY_ITEM item=alpha .* expand_up_to=$value\$" "$work/out" \
      && out_has "缩过域" && out_has "不算门禁通过"; } || expand_passed_wrong+="$2（退 $exit_code）；"
  }
  entry -- --crash-expand-up-to 6; expand_check 6 "参数 6"
  entry SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=4 --; expand_check 4 "只设环境变量 4"
  entry SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=4 -- --crash-expand-up-to 6; expand_check 6 "参数 6 与环境变量 4 都给"
  # 判器那个变量：给了参数盖过环境里已设的（参数 6、环境里判器 2 ⇒ 三段都收到 6，标签不写 judge_expand_up_to）；没给参数时环境里已设的照用（放量 4、判器 2 ⇒ 4 与 2，标签两个都写）；
  #   只设了判器那个的照旧只认它（放量那个三段都是 unset）
  entry SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=2 -- --crash-expand-up-to 6; expand_check 6 "参数 6 与环境里判器变量 2 都给（判器那个也是 6）"
  entry SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=4 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=2 --
  { [[ "$exit_code" == 0 ]] && [[ "$(calls_count '^cargo-env expand=\[4\]$')" == "$(calls_count '^cargo-env expand=')" ]] \
    && [[ "$(calls_count '^cargo-env judge-expand=\[2\]$')" == "$(calls_count '^cargo-env judge-expand=')" ]] \
    && calls_have '^enumeration-env judge-expand=\[2\]$' && calls_have '^pipeline-env judge-expand=\[2\]$' \
    && grep -qE "^CRASH_POINT_REPLAY_TOTAL .* expand_up_to=4 judge_expand_up_to=2\$" "$work/out"; } || expand_passed_wrong+="环境里放量 4、判器 2 都设、没给参数：三段应当收到 4 与 2、标签写 expand_up_to=4 judge_expand_up_to=2（退 $exit_code）；"
  entry SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3 --
  { [[ "$exit_code" == 0 ]] && [[ "$(calls_count '^cargo-env expand=\[unset\]$')" == "$(calls_count '^cargo-env expand=')" ]] \
    && [[ "$(calls_count '^cargo-env judge-expand=\[3\]$')" == "$(calls_count '^cargo-env judge-expand=')" ]] \
    && calls_have '^enumeration-env expand=\[unset\]$' && calls_have '^enumeration-env judge-expand=\[3\]$' \
    && grep -qE "^CRASH_POINT_REPLAY_TOTAL .* judge_expand_up_to=3\$" "$work/out"; } || expand_passed_wrong+="环境里只设判器 3、没给参数：三段应当收到判器 3、放量那个 unset、标签写 judge_expand_up_to=3（退 $exit_code）；"
  entry --
  { [[ "$exit_code" == 0 ]] && [[ "$(calls_count '^cargo-env expand=\[unset\]$')" == "$(calls_count '^cargo-env expand=')" ]] \
    && [[ "$(calls_count '^cargo-env judge-expand=\[unset\]$')" == "$(calls_count '^cargo-env judge-expand=')" ]] \
    && calls_have '^enumeration-env expand=\[unset\]$' && calls_have '^pipeline-env expand=\[unset\]$' \
    && ! grep -qE '^CRASH_POINT_REPLAY_(ITEM|TOTAL) .*expand_up_to=' "$work/out" && ! out_has "缩过域"; } || expand_passed_wrong+="不缩的那一趟三段应当看到 unset、行里与句里没有缩域的说法（退 $exit_code）；"
  [[ -z "$expand_passed_wrong" ]] || cell_failed expand-up-to-passed "给了缩域值，三段都应当收到变量、ITEM 与 TOTAL 行带 expand_up_to、成功句写缩过域（$expand_passed_wrong）" \
    "看参数之后「缩崩溃域」那一段与 test_functions_environment"
  fi
  if selftest_cell_wanted item-done-lines; then
  # item-done-lines：真跑每项跑完一行 ITEM_DONE（stderr），行数等于 ITEM 行数；alpha（崩溃枚举用例全复用、放量判了 10 态）judged=1 reused=1，beta（用例全复用）judged=0 reused=1，
  #   gamma（放量判了）judged=1 reused=0，都退 0；beta 的崩溃点重放那一道睡 3 秒：beta 的 seconds 不小于 3，每项的 seconds 等于同一项 CRASH_POINT_REPLAY_ITEM 行的 seconds
  #   （各记各的，不是整趟的：整趟之和至少比 alpha、gamma 自己的多出 beta 那 3 秒）。只比同一趟里的两个数，不比两项谁快谁慢（机器忙时各段的挂钟谁长谁短说不准）
  local done_wrong="" done_beta_seconds done_item done_seconds item_seconds
  entry STUB_CRASH_POINT_REPLAY_REUSED=1 STUB_GATE_BETA_SLEEP=3 --
  [[ "$exit_code" == 0 && "$(grep -c '^ITEM_DONE ' "$work/out")" == "$(grep -c '^CRASH_POINT_REPLAY_ITEM ' "$work/out")" ]] \
    || done_wrong+="ITEM_DONE 的行数应当等于 ITEM 行数（退 $exit_code，$(grep -c '^ITEM_DONE ' "$work/out") 对 $(grep -c '^CRASH_POINT_REPLAY_ITEM ' "$work/out")）；"
  item_done_has() { grep -qE "^ITEM_DONE item=$1 seconds=[0-9]+ judged=$2 reused=$3 exit=0\$" "$work/out"; }   # <项> <现判> <复用>
  { item_done_has alpha 1 1 && item_done_has beta 0 1 && item_done_has gamma 1 0; } || done_wrong+="alpha 应当 judged=1 reused=1、beta judged=0 reused=1、gamma judged=1 reused=0，都退 0；"
  done_beta_seconds="$(sed -nE 's/^ITEM_DONE item=beta seconds=([0-9]+) .*/\1/p' "$work/out")"
  { [[ "$done_beta_seconds" =~ ^[0-9]+$ ]] && (( done_beta_seconds >= 3 )); } || done_wrong+="beta 的 seconds「$done_beta_seconds」应当不小于 3（它的崩溃点重放那一道睡了 3 秒）；"
  for done_item in alpha beta gamma; do
    done_seconds="$(sed -nE "s/^ITEM_DONE item=$done_item seconds=([0-9]+) .*/\\1/p" "$work/out")"
    item_seconds="$(sed -nE "s/^CRASH_POINT_REPLAY_ITEM item=$done_item .* seconds=([0-9]+)( .*)?\$/\\1/p" "$work/out")"
    { [[ "$done_seconds" =~ ^[0-9]+$ ]] && [[ "$done_seconds" == "$item_seconds" ]]; } \
      || done_wrong+="$done_item 的 ITEM_DONE seconds「$done_seconds」应当等于它那一行 CRASH_POINT_REPLAY_ITEM 的 seconds「$item_seconds」（各记各的）；"
  done
  [[ -z "$done_wrong" ]] || cell_failed item-done-lines "真跑应当每项一行 ITEM_DONE、件数与退出码照各段的记录、挂钟各记各的：$done_wrong" "看 summarize 里 ITEM_DONE 那一段与 record_part 第十一列"
  fi
  if selftest_cell_wanted gpu-test-functions; then
  # gpu-test-functions：开了 GPU，测试函数带 gpu 特性、编那一趟与每一趟都把环境里的 GPU_CARDS_BY_PRIORITY 原样交给 cargo（多机配置里写的显卡先后不读）、不自己清场
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" GPU_CARDS_BY_PRIORITY=127.0.0.1:1:20G --
  if [[ "$exit_code" != 0 ]] || calls_have '^quiesce-stop$' \
     || [[ "$(calls_count "^cargo test --release -p singlefs-verification --features gpu --test [a-z0-9_]+ -- --nocapture cards=\[127.0.0.1:1:20G\] quiesced=\[\]")" != $((3 + extras_count)) ]] \
     || ! calls_have '^cargo test .*--features gpu --no-run .*cards=\[127.0.0.1:1:20G\] quiesced=\[\]'; then
    cell_failed gpu-test-functions "开了 GPU 测试函数应当带 gpu 特性、不自己清场、测试参数不带测试线程、把环境里的 GPU_CARDS_BY_PRIORITY（127.0.0.1:1:20G，不是多机配置里的显卡先后）原样交给 cargo" \
      "看 run_test_functions 里 gpu_switch 那一段"
  fi
  fi
  if selftest_cell_wanted gpu-without-cards-runs; then
  # gpu-without-cards-runs：开了 GPU、环境里没有 GPU_CARDS_BY_PRIORITY（卡归 GPU 批处理验证服务，任务不拿卡）⇒ 照起：编那一趟与每一趟测试函数都带 gpu 特性、不带卡串、
  #   测试参数不带测试线程、不自己清场，整趟退 0
  local without_cards_wrong=""
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" --
  [[ "$exit_code" == 0 ]] || without_cards_wrong+="整趟应当退 0（退 $exit_code）；"
  [[ "$(calls_count "^cargo test --release -p singlefs-verification --features gpu --test [a-z0-9_]+ -- --nocapture cards=\[unset\] quiesced=\[\]")" == $((3 + extras_count)) ]] \
    || without_cards_wrong+="每一趟测试函数都应当带 gpu 特性、不带卡串、测试参数不带测试线程（共 $((3 + extras_count)) 趟）；"
  calls_have '^cargo test .*--features gpu --no-run .*cards=\[unset\] quiesced=\[\]' || without_cards_wrong+="编那一趟应当带 gpu 特性、不带卡串；"
  if calls_have '^quiesce-stop$'; then without_cards_wrong+="不该自己清场；"; fi
  [[ -z "$without_cards_wrong" ]] || cell_failed gpu-without-cards-runs "开了 GPU 而环境里没有卡也应当照起：$without_cards_wrong" \
    "看主体「GPU 与配置」那一段与 run_test_functions 里 gpu_switch 那一段（环境里没有卡就不设 GPU_CARDS_BY_PRIORITY、不退 78）"
  fi
  if selftest_cell_wanted test-functions-red; then
  # test-functions-red：beta 那个测试目标红 ⇒ beta 红、退 1，别的测试目标照绿（alpha 的测试函数两条都过、不红；它的放量部分因测试函数红没起，记本次未跑，见 test-functions-red-stops-light）
  entry STUB_RED_TARGETS=beta --
  if [[ "$exit_code" != 1 ]] || ! item_has beta 'verdict=red' || item_has alpha 'verdict=red' || ! item_has alpha 'test_functions_tests=2 test_functions_red=0 ' || ! out_has "✗ beta："; then
    cell_failed test-functions-red "测试函数一个测试目标红了，那一项应当红、整趟退 1" "看 run_test_functions 判 state 那一段"
  fi
  fi
  if selftest_cell_wanted test-functions-not-run; then
  # test-functions-not-run：beta 跑到 0 条、alpha 有 NOT_RUN_NO_GPU_ADAPTER ⇒ 两项本次未跑、退 0，成功句点名
  entry STUB_ZERO_TARGETS=beta STUB_NOT_RUN_TARGETS=alpha --
  if [[ "$exit_code" != 0 ]] || ! item_has beta 'verdict=not-run' || ! item_has alpha 'verdict=not-run' \
     || ! grep -q '本次未跑 3 项：alpha（有 1 处 NOT_RUN_NO_GPU_ADAPTER' "$work/out"; then
    cell_failed test-functions-not-run "测试函数跑到 0 条或没卡没跑的，应当记本次未跑、成功句点名" "看 run_test_functions 判 state 那一段与 summarize 的成功句"
  fi
  fi
  if selftest_cell_wanted test-functions-marker-on-a-test-line; then
  # test-functions-marker-on-a-test-line：几条同时跑，alpha 的 NOT_RUN_NO_GPU_ADAPTER 接在别的测试的「test <名> ... 」后面 ⇒ 照样记本次未跑，成功句点名
  entry STUB_NOT_RUN_ON_A_TEST_LINE_TARGETS=alpha --
  if [[ "$exit_code" != 0 ]] || ! item_has alpha 'verdict=not-run' || ! grep -q 'alpha（有 1 处 NOT_RUN_NO_GPU_ADAPTER' "$work/out"; then
    cell_failed test-functions-marker-on-a-test-line "接在别的测试结果行起头后面的 NOT_RUN_NO_GPU_ADAPTER 也算本次未跑，成功句点名" "看 judge_test_functions_log 的 printed_by_the_test"
  fi
  fi
  if selftest_cell_wanted all-not-run; then
  # all-not-run：测试函数全是 0 条、放量全是 not-run ⇒ 退 77
  entry STUB_ZERO_TARGETS=all STUB_AMP_NOT_RUN="alpha-amp gamma-amp" --
  [[ "$exit_code" == 77 ]] || cell_failed all-not-run "派出的项全是本次未跑，应当退 77" "看 summarize 里 not green 那一支"
  fi
  if selftest_cell_wanted whole; then
  # whole：不带参数 ⇒ 测试函数那一段（连同整趟才跑的那几类）、崩溃点重放每项一趟 --item、放量不带 --item；复用数照实报
  entry STUB_CRASH_POINT_REPLAY_REUSED=1 --
  if [[ "$exit_code" != 0 ]] || [[ "$(calls_count '^cargo test --release -p singlefs-verification --test [a-z0-9_]+ -- --nocapture ')" != $((3 + extras_count)) ]] \
     || ! calls_have "^enumeration-gate --item crash-case:alpha $root\$" || ! calls_have "^enumeration-gate --item crash-case:beta $root\$" \
     || ! calls_have '^pipeline-entry $' || calls_have '^admission ' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL scope=whole dispatched=6 collected=6 .* pipeline_reused_known=8 .* enumeration_cases=2 enumeration_reused=2 ' "$work/out"; then
    cell_failed whole "不带参数应当跑全部项（测试函数那一段照跑、崩溃点重放每项 --item、放量照全部项、复用数照实报）" "看 replay_plan 与 run_enumeration_part"
  fi
  fi
  if selftest_cell_wanted items; then
  # items：--item gamma ⇒ 只跑 gamma：测试函数那一段只 gamma_amp、不跑整趟才跑的那几类、不起崩溃点重放、放量 --item gamma-amp
  entry -- --item gamma
  if [[ "$exit_code" != 0 ]] || [[ "$(calls_count ' --test [a-z0-9_]+ -- --nocapture ')" != 1 ]] || ! calls_have ' --test gamma_amp -- --nocapture ' \
     || calls_have '^enumeration-gate ' || ! calls_have '^pipeline-entry --item gamma-amp$' || item_line 'whole-run:pruning-self-proof' >/dev/null \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL scope=items dispatched=1 ' "$work/out"; then
    cell_failed items "--item 应当只跑点名的项" "看 scope 那一段 items 与 run_pipeline_part 的 --item"
  fi
  fi
  if selftest_cell_wanted whole-run-item; then
  # whole-run-item：--item whole-run:pruning-self-proof ⇒ 只跑那一类的两个测试目标，不起崩溃点重放与放量，ITEM 行点名那一项
  entry -- --item whole-run:pruning-self-proof
  if [[ "$exit_code" != 0 ]] || [[ "$(calls_count ' --test [a-z0-9_]+ -- --nocapture ')" != 2 ]] \
     || ! calls_have ' --test crash_amplification_two_flows_share_their_prefix -- --nocapture ' || ! calls_have ' --test crash_amplification_pipeline_records_checks_and_reuses -- --nocapture ' \
     || calls_have '^enumeration-gate ' || calls_have '^pipeline-entry ' || ! item_has 'whole-run:pruning-self-proof' 'scope=items'; then
    cell_failed whole-run-item "--item whole-run:<类> 应当只跑那一类的测试目标（退 $exit_code）" "看 load_whole_run_items 与 replay_plan 对 whole-run 项的处理"
  fi
  fi
  # 登记的用例不进测试函数那一段（--exclude-registered）：登记给 crash-case:alpha 的 alpha_full 与放量分项 alpha-amp 的 alpha_amplified 经 --skip 排掉，不判、不记；
  #   alpha_stands_alone 照判。unit-root 的 alpha.rs 临时加两条登记的用例，跑完换回
  # （这一格在下面 unit-root 那一批里做，需要真的 admission.py）
  if selftest_cell_wanted enumeration-red; then
  # enumeration-red：崩溃点重放那一道退 1 ⇒ alpha、beta 红，退 1
  entry STUB_CRASH_POINT_REPLAY_EXIT=1 -- --item alpha --item beta
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'verdict=red' || ! item_has beta 'enumeration_red=1' \
     || ! out_has "只重跑红的那 2 项：bash scripts/crash/crash-point-replay.sh --item alpha --item beta"; then
    cell_failed enumeration-red "崩溃点重放那一道红了，那一项应当红、出路给出只重跑那两项的命令" "看 run_enumeration_part 判 exit_code 那一段与 summarize 的出路"
  fi
  fi
  if selftest_cell_wanted enumeration-gate-cases-only; then
  # enumeration-gate-cases-only：点名了项的跑法（scope=items）起门禁带 --check crash-case-replay --crash-cases-only（只走按块那一路、不把整个验证包按条判一遍）；
  #   不带 --item 的跑法照旧整道（验证包里不属于任何项的测试目标靠它罩住），一条门禁调用都不带这三个参数
  entry -- --item alpha --item beta
  if [[ "$exit_code" != 0 ]] || ! calls_have "^enumeration-gate --check crash-case-replay --crash-cases-only --item crash-case:alpha $root\$" \
     || ! calls_have "^enumeration-gate --check crash-case-replay --crash-cases-only --item crash-case:beta $root\$"; then
    cell_failed enumeration-gate-cases-only "点名了项的跑法应当起门禁带 --check crash-case-replay --crash-cases-only，每条登记的用例各一趟（退 $exit_code）" "看 run_enumeration_part 里 scope == items 那一段"
  fi
  fi
  if selftest_cell_wanted enumeration-gate-whole-keeps-package; then
  entry --
  if [[ "$exit_code" != 0 ]] || calls_have '^enumeration-gate .*(--check|--crash-cases-only)'; then
    cell_failed enumeration-gate-whole-keeps-package "不带 --item 的跑法起门禁不该带 --check 与 --crash-cases-only（整道照旧，验证包整包由它罩住）" "看 run_enumeration_part 里 scope == items 那一段"
  fi
  fi
  if selftest_cell_wanted pipeline-red; then
  # pipeline-red：放量 alpha-amp red=2 ⇒ alpha 红
  entry STUB_AMP_RED=alpha-amp --
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'pipeline_red=2' || ! item_has alpha 'verdict=red' || ! item_has gamma 'verdict=green'; then
    cell_failed pipeline-red "放量那一项红了，那一项应当红" "看 run_pipeline_part 判 state 那一段"
  fi
  fi
  if selftest_cell_wanted record-text-clipped-in-character; then
  # record-text-clipped-in-character：按条复用失灵（admission.py test-functions-request 没成）的说明经 clip_utf8 截到 300 字节。桩准入模块打一个 x 加 120 个汉字（361 字节，
  #   第 300 个字节落在第 100 个汉字中间）退 1 ⇒ 整趟退 1、点到测试目标的项红、说明停在 x 加 99 个汉字、屏幕与日志逐字节是合法的 UTF-8、汇总没有回溯
  #   （弄坏开关 clip-cuts-in-character：只按字节切，输出里留着半个汉字）。plan_test_functions 起的是这份脚本所在仓的真 admission.py，小仓里的假准入模块不会被调到，
  #   所以 test-functions-request 的失败由 PATH 最前面的假 python3 造（setup 里的 request-failure-bin）；没走到按条复用失灵那一支（测试目标被 cargo 起了、或说明不在屏幕上）也判错
  { printf 'x'; for ((request_character = 0; request_character < 120; request_character++)); do printf '中'; done; } >"$work/request-failure"
  entry PATH="$request_failure_bin:$work/bin:$PATH" STUB_REAL_PYTHON3="$real_python3" STUB_REQUEST_FAILURE_FILE="$work/request-failure" --
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'verdict=red' || ! out_has '按条复用失灵：scripts/gate/admission.py test-functions-request 没成（x中' \
     || out_has 'Traceback' || ! iconv -f UTF-8 -t UTF-8 "$work/out" >/dev/null 2>&1 || calls_have '^cargo test .* --test alpha '; then
    cell_failed record-text-clipped-in-character "说明经 clip_utf8 截在汉字中间时应当丢掉半个字：整趟退 1、alpha 红、输出逐字节是合法 UTF-8、汇总没有回溯；实际退 $exit_code，输出末尾：$(tail -n 8 "$work/out" | tr '\n' ' ' | selftest_clip_text 900)" "看 clip_utf8 与 plan_test_functions 里 test-functions-request 没成那一支"
  fi
  fi
  if selftest_cell_wanted record-text-with-bad-bytes; then
  # record-text-with-bad-bytes：汇总读记录文件，记录里有不成字符的字节也不崩。桩放量入口在 alpha-amp 的 NOT_RUN 那一行末尾带汉字的头两个字节，
  #   放量部分经 pipeline_source_changed_reason 的 awk 把那一行的 reason= 原样写进 alpha 的原因 ⇒ 整趟退 0、alpha 本次未跑、原因照打、末行 TOTAL 照打、没有回溯
  #   （弄坏开关 records-decoded-strictly：严格解码，汇总抛 UnicodeDecodeError）。这一趟在 C 语言环境（LC_ALL=C）下起：那一段 awk 在 UTF-8 环境里读到不成字符的字节，
  #   把它们改写成 U+FFFD，字节到不了记录文件、严格解码也读得过；C 语言环境（以及不改写字节的 awk）下才原样带进记录文件。记录文件里没有那两个字节，说明这一格没走到那条路，也判错
  local bad_bytes_run_directory
  entry LC_ALL=C STUB_AMP_SOURCE_CHANGED=alpha-amp STUB_AMP_NOT_RUN_REASON_TAIL=$'\xe4\xb8' --
  bad_bytes_run_directory="$(ls -td "$work/home/runs"/crash-point-replay-* 2>/dev/null | head -n 1)"
  if [[ "$exit_code" != 0 ]] || ! item_has alpha 'verdict=not-run' || ! out_has '本次未跑 alpha：放量 alpha-amp 本次未跑（exit=77）：跑的过程中源码变了' \
     || ! out_has 'CRASH_POINT_REPLAY_TOTAL' || out_has 'Traceback' \
     || [[ -z "$bad_bytes_run_directory" ]] || ! LC_ALL=C grep -qaF $'\xe4\xb8；这不是红' "$bad_bytes_run_directory/parts.tsv" 2>/dev/null; then
    cell_failed record-text-with-bad-bytes "记录文件里的说明带不成字符的字节时汇总应当照常打：整趟退 0、alpha 本次未跑、原因与末行 TOTAL 都在、没有回溯，记录文件 ${bad_bytes_run_directory:-（日志目录没找到）}/parts.tsv 里要真有那两个字节" "看 summarize 里读 records_file 的解码方式（records_decoding）与 pipeline_source_changed_reason 把日志里的字节带进记录的那一段"
  fi
  fi
  if selftest_cell_wanted collection-count; then
  # collection-count：放量入口没打 gamma-amp 那一行 ⇒ gamma 红（收不回）
  entry STUB_AMP_DROP=gamma-amp --
  if [[ "$exit_code" != 1 ]] || ! item_has gamma 'verdict=red' || ! out_has "收不回"; then
    cell_failed collection-count "放量那一项收不回，应当红、派出收回对不上" "看 run_pipeline_part 里 line 为空那一支"
  fi
  fi
  if selftest_cell_wanted test-functions-red-continues; then
  # test-functions-red-continues：beta 那个测试目标红 ⇒ 整趟退 1，崩溃点重放那一道与放量入口照旧起（两种跑法一样），派出收回相等
  local continue_wrong=() continue_case
  for continue_case in "" "--item alpha --item beta"; do
    # shellcheck disable=SC2086  # 参数是空格隔开的一串，逐个当参数
    entry STUB_RED_TARGETS=beta -- $continue_case
    if [[ "$exit_code" != 1 ]] || ! calls_have '^enumeration-gate (--check crash-case-replay --crash-cases-only )?--item crash-case:beta ' || ! calls_have '^pipeline-entry ' || ! item_has beta 'verdict=red' \
       || ! grep -qE 'CRASH_POINT_REPLAY_TOTAL .*dispatched=([0-9]+) collected=\1 ' "$work/out"; then
      continue_wrong+=("「crash-point-replay.sh $continue_case」退 $exit_code、调用记录 $(tr '\n' '|' <"$calls" | selftest_clip_text 200)")
    fi
  done
  if (( ${#continue_wrong[@]} )); then
    exit_code="${#continue_wrong[@]} 处"
    cell_failed test-functions-red-continues "测试函数那一段红了，别的两段照旧起（崩溃点重放那一道与放量入口）、整趟退 1、派出收回相等：${continue_wrong[*]}" \
      "看主流程：测试函数、崩溃点重放、放量三段各自跑，不看别的段红不红"
  fi
  fi
  # run-target-green / run-target-red / run-target-term：跑完运行目录里不剩它建的编译目录（文件头「编译目录」）：调用方没设 CARGO_TARGET_DIR，不带参数（不带参数）；
  #   假 cargo 记下每一趟收到的编译目录、往里写一份产物。正常退 0、测试函数红退 1、被 TERM 停（假 cargo 每一趟睡 3 秒，头一趟起了之后只给入口发 TERM，
  #   入口等手里那一步退出后走 EXIT，退 143）：每一趟收到的都是 $work/home/runs/crash-point-replay-*/target 那同一份，跑完不在了
  run_target_check() { # <格名> <要的退出码> <一句结局>
    local seen
    seen="$(sort -u "$work/targets.log" 2>/dev/null)"
    if [[ "$exit_code" != "$2" || "$(grep -c . <<<"$seen")" != 1 || "$seen" != "$work"/home/runs/crash-point-replay-*/target || -e "$seen" ]]; then
      cell_failed "$1" "跑完运行目录里不剩它建的编译目录（$3）：假 cargo 收到「$(tr '\n' ' ' <<<"$seen")」，要的退出码 $2，跑完$([[ -n "$seen" && -e "$seen" ]] && echo 还在 || echo 不在)" \
        "看建日志目录之后的 run_build_directory_begin 与收尾的 replay_finish（run_build_directory_end），TERM / INT 的 trap 退 143 / 130、等手里那一步退出再走 EXIT"
    fi
  }
  if selftest_cell_wanted run-target-green; then
  : >"$work/targets.log"; entry STUB_TARGET_LOG="$work/targets.log" --; run_target_check run-target-green 0 "正常退 0"
  fi
  if selftest_cell_wanted run-target-red; then
  : >"$work/targets.log"; entry STUB_TARGET_LOG="$work/targets.log" STUB_RED_TARGETS=beta --; run_target_check run-target-red 1 "测试函数红退 1"
  fi
  if selftest_cell_wanted run-target-term; then
  local term_entry term_attempt
  : >"$work/targets.log"; : >"$calls"; cells=$((cells + 1))
  env -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP -u SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE -u SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES \
    -u GPU_QUIESCE_ORIGIN_READ_BACK \
    PATH="$work/bin:$PATH" STUB_CALLS="$calls" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
    SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" CRASH_POINT_REPLAY_SELFTEST_MARK="$work/mark" CRASH_POINT_REPLAY_ROOT="$root" \
    CRASH_POINT_REPLAY_BREAK="$replay_break" STUB_TARGET_LOG="$work/targets.log" STUB_SLEEP=3 bash "$replay_script" >"$work/out" 2>&1 &
  term_entry=$!
  for (( term_attempt = 0; term_attempt < 600; term_attempt++ )); do [[ -s "$work/targets.log" ]] && break; sleep 0.1; done
  kill -TERM "$term_entry" 2>/dev/null
  if wait "$term_entry"; then exit_code=0; else exit_code=$?; fi
  run_target_check run-target-term 143 "被 TERM 停"
  fi
  if selftest_cell_wanted run-target-caller; then
  # run-target-caller：调用方设了 CARGO_TARGET_DIR ⇒ 每一趟照收到它、跑完不删
  mkdir -p "$work/caller-target"; : >"$work/targets.log"
  entry STUB_TARGET_LOG="$work/targets.log" CARGO_TARGET_DIR="$work/caller-target" --
  if [[ "$exit_code" != 0 || "$(sort -u "$work/targets.log")" != "$work/caller-target" || ! -f "$work/caller-target/release/artifact" ]]; then
    cell_failed run-target-caller "调用方设了 CARGO_TARGET_DIR 时每一趟应当照收到它、跑完不删（收到「$(sort -u "$work/targets.log" | tr '\n' ' ')」）" \
      "看 run_build_directory_begin：设了 CARGO_TARGET_DIR 就不建、不删"
  fi
  fi
  if selftest_cell_wanted two-logs-per-part; then
  # two-logs-per-part：每一段的日志出两份（全量 <段>.log，旁边同名的 <段>.problems.log 只收错误与警告：行前带全量日志里的行号，末行写收了几行全量几行）；
  #   测试函数红的那个测试目标的 FAILED 行在错误警告日志里，输出里点名这份日志的路径
  local problems_run problems_file problems_last_line
  entry STUB_RED_TARGETS=beta --
  problems_run="$(ls -td "$work/home/runs"/crash-point-replay-* 2>/dev/null | head -n 1)"
  problems_file="$problems_run/test-functions.beta.problems.log"
  problems_last_line="$(tail -n 1 "$problems_file" 2>/dev/null)"   # 管道收尾的 grep -q 在 pipefail 下会把命中读成没命中：先落到变量
  if [[ ! -f "$problems_run/test-functions.beta.log" || ! -f "$problems_file" ]] || ! grep -qE '^[0-9]+:test broken \.\.\. FAILED$' "$problems_file" \
     || ! grep -qE '收了 [0-9]+ 行，全量日志共 [0-9]+ 行' <<<"$problems_last_line" || ! out_has "错误警告 $problems_file"; then
    cell_failed two-logs-per-part "测试函数红的那一段应当出两份日志（test-functions.beta.log 全量、旁边的 test-functions.beta.problems.log 带行号收那条 FAILED、末行写收了几行全量几行），输出里点名错误警告日志的路径 $problems_file" \
      "看 run_logged 经 scripts/hosts/two-logs.sh 起命令，与红的原因里的 \${log%.log}.problems.log"
  fi
  fi
  if selftest_cell_wanted pipeline-source-changed-not-run; then
  # pipeline-source-changed-not-run：放量分项驱动判段跑的过程中源码变了（驱动退 77、打 CRASH_AMPLIFICATION_NOT_RUN step=judge …跑的过程中源码变了，入口记 verdict=not-run）⇒
  #   alpha 这一项本次未跑、不红，原因点名源码变了（「放量 alpha-amp 本次未跑」之后接那一行的 reason=）
  entry STUB_AMP_SOURCE_CHANGED=alpha-amp --
  if [[ "$exit_code" != 0 ]] || ! item_has alpha 'verdict=not-run' || item_has alpha 'verdict=red' \
     || ! out_has '本次未跑 alpha：放量 alpha-amp 本次未跑（exit=77）：跑的过程中源码变了，判段没在录入段那份源码上判完'; then
    cell_failed pipeline-source-changed-not-run "放量分项驱动报跑的过程中源码变了，那一项应当记本次未跑（verdict=not-run）、不记红，原因点名「跑的过程中源码变了」、整趟退 0" "看 run_pipeline_part 里 item_verdict 为 not-run 那一支与 pipeline_source_changed_reason"
  fi
  fi
  if selftest_cell_wanted pipeline-entry-78-not-run; then
  # pipeline-entry-78-not-run：放量入口退 78（运行条件不满足）而没打 CRASH_AMPLIFICATION_ITEM 行 ⇒ 两项都记本次未跑、整趟退 78，不说「都不红」；
  #   红先判：alpha-amp 红、gamma-amp 收不回而入口退 78 ⇒ 退 1；入口退 1 而收不回照旧红
  entry STUB_AMP_EXIT=78 STUB_AMP_DROP="alpha-amp gamma-amp" --
  if [[ "$exit_code" != 78 ]] || ! item_has alpha 'exit=78 verdict=not-run' || ! item_has gamma 'exit=78 verdict=not-run' || item_has alpha 'verdict=red' \
     || ! out_has '本次未跑 alpha：运行条件不满足：放量入口退 78' || out_has '项都不红' || ! out_has '有一段运行条件不满足（退 78）'; then
    cell_failed pipeline-entry-78-not-run "放量入口退 78 而没打 ITEM 行，两项应当记本次未跑（exit=78 verdict=not-run）、整趟退 78、不说都不红" "看 run_pipeline_part 里 line 为空那一支对退出码 78 的分支"
  fi
  fi
  if selftest_cell_wanted pipeline-entry-78-red-first; then
  entry STUB_AMP_RED=alpha-amp STUB_AMP_DROP=gamma-amp STUB_AMP_EXIT=78 --
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'verdict=red' || ! item_has gamma 'exit=78 verdict=not-run'; then
    cell_failed pipeline-entry-78-red-first "放量入口退 78 而 alpha-amp 红时，整趟应当退 1、alpha 红、gamma 本次未跑" "看 run_pipeline_part 与 summarize：红先判，退 78 的只在没有红时才让整趟退 78"
  fi
  fi
  if selftest_cell_wanted pipeline-entry-1-still-red; then
  entry STUB_AMP_EXIT=1 STUB_AMP_DROP=gamma-amp --
  if [[ "$exit_code" != 1 ]] || ! item_has gamma 'verdict=red' || ! out_has '收不回'; then
    cell_failed pipeline-entry-1-still-red "放量入口退 1（不是 78）而没打 ITEM 行，仍应当红（收不回）" "看 run_pipeline_part 里 line 为空那一支：只有退 78 才记本次未跑"
  fi
  fi
  if selftest_cell_wanted pipeline-reuses-the-long-term-library; then
  # pipeline-reuses-the-long-term-library：假的放量入口照键复用（判过的分项在假服务 STUB_AMP_SERVICE 里留一份，演 prune-server 里那一份判定库）⇒ 剪枝接连两趟：
  #   头一趟现判 20 态复用 0 态，第二趟现判 0 态复用 20 态（第二次跑复用了第一次判过的块），成功句照实报，服务里判过的留着
  local reuse_first_ok=0 reuse_first_total reuse_service="$work/stub-service"
  rm -rf -- "${reuse_service:?}"
  entry STUB_AMP_PERSISTENT=1 STUB_AMP_SERVICE="$reuse_service" --
  reuse_first_total="$(grep -E '^CRASH_POINT_REPLAY_TOTAL ' "$work/out")"
  if [[ "$exit_code" == 0 && "$reuse_first_total" == *" pipeline_states=20 pipeline_reused_known=0 "* ]] && out_has "放量现判 20 态复用 0 态"; then reuse_first_ok=1; fi
  entry STUB_AMP_PERSISTENT=1 STUB_AMP_SERVICE="$reuse_service" --
  if (( ! reuse_first_ok )) || [[ "$exit_code" != 0 ]] \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL .* pipeline_states=0 pipeline_reused_known=20 ' "$work/out" || ! out_has "放量现判 0 态复用 20 态" \
     || [[ ! -e "$reuse_service/alpha-amp.judged" || ! -e "$reuse_service/gamma-amp.judged" ]]; then
    cell_failed pipeline-reuses-the-long-term-library "剪枝那一趟第二次跑应当复用第一次判过的块：头一趟现判 20 态复用 0 态、第二趟现判 0 态复用 20 态，服务里判过的留着（头一趟：${reuse_first_total:-没有 TOTAL 行}）" \
      "看 run_pipeline_part 收 CRASH_AMPLIFICATION_ITEM 行那一段与文件头「三段」放量部分"
  fi
  rm -rf -- "${reuse_service:?}"
  fi
  if selftest_cell_wanted reuse-three-questions; then
  # reuse-three-questions（scripts/gate/lib_reuse_checks.py 文件头「一」）：假的放量入口照键复用、键另带每个分项的单元文件（STUB_AMP_UNITS）⇒ 剪枝接连四趟：
  #   头一趟、同一份输入、加一份哪个分项都不读的单元文件（闭包外）、改 alpha-amp 的单元文件（闭包内）；从 ITEM 行取有放量那一段的项（alpha、gamma）复用还是现判
  #   （pipeline_states=0 而 pipeline_reused 是正整数是复用），照三问判、打规范输出。这一份入口不判复用（判定在引擎里）：核的是每一趟都不碰判定库（判过的照服务复用）、
  #   不因闭包外的改动重判，并照实把放量入口报的现判与复用写进 ITEM 行；pipeline-reuses-the-long-term-library 那一格是这里的 ①
  local three_units="$work/amp-units" three_exits=() three_round_name
  mkdir -p -- "$three_units"; printf 'alpha-amp 的单元\n' >"$three_units/alpha-amp"; printf 'gamma-amp 的单元\n' >"$three_units/gamma-amp"
  three_round() { # <名>
    entry STUB_AMP_PERSISTENT=1 STUB_AMP_SERVICE="$work/stub-service" STUB_AMP_UNITS="$three_units" --
    three_exits+=("$exit_code")
    awk '/^CRASH_POINT_REPLAY_ITEM / { item = ""; states = ""; reused = ""
           for (i = 2; i <= NF; i++) { split($i, pair, "="); if (pair[1] == "item") item = pair[2]; if (pair[1] == "pipeline_states") states = pair[2]; if (pair[1] == "pipeline_reused") reused = pair[2] }
           if (states != "-") print item "\t" ((states == "0" && reused ~ /^[1-9][0-9]*$/) ? "复用" : "现判") }' "$work/out" >"$work/three-$1.tsv"
  }
  three_round first; three_round second
  printf '哪个分项都不读\n' >"$three_units/not-read-by-any-item"; three_round unrelated
  printf 'alpha-amp 的单元，改过\n' >"$three_units/alpha-amp"; three_round unit
  if ! python3 "$replay_script_directory/../gate/lib_reuse_checks.py" three-questions --script scripts/crash/crash-point-replay.sh --first "$work/three-first.tsv" \
         --second "$work/three-second.tsv" --unrelated "$work/three-unrelated.tsv" --unit-change "$work/three-unit.tsv" --affected alpha >"$work/three.log" 2>&1 \
     || [[ "${three_exits[*]}" != "0 0 0 0" ]]; then
    cell_failed reuse-three-questions "复用三问没过（四趟退 ${three_exits[*]}）：$(tr '\n' ' ' <"$work/three.log")" \
      "照规范输出那一行 red 的那一问查：run_pipeline_part 有没有碰判定库，与 ITEM 行的 pipeline_states=、pipeline_reused= 取法"
  fi
  grep '^REUSE_SELFTEST ' "$work/three.log"
  rm -rf -- "${work:?}/stub-service"
  fi
  # ── 按条复用那几格（文件头「按条复用」）：小仓 unit-root 上真的 admission.py 算键、查库、记库；结局取每趟逐条打的「· <测试函数>：绿|红，现判|复用」那几行
  local unit_settings=(PATH="$work/unit-bin:$work/bin:$PATH" CRASH_POINT_REPLAY_ROOT="$unit_root" STUB_UNIT_ROOT="$unit_root")
  # 判定库在 prune-server 里（被判那棵树的 git common-dir 只定临时服务底下的命名空间），删不掉：要「库先清空」的格各给 unit-root 换一处新的 git 目录
  #   （unit_fresh_library：把它此刻的 git 目录整个拷到一处新地方，unit-root/.git 换成指过去的 gitdir 文件；内容、提交与索引照旧，common-dir 换了，
  #   这一份的库在临时服务里就是新的一个命名空间）
  local unit_library_root="" unit_library_count=0 unit_current_git
  unit_fresh_library() {
    unit_library_count=$((unit_library_count + 1))
    unit_library_root="$work/unit-library-$unit_library_count"
    unit_current_git="$(git -C "$unit_root" rev-parse --path-format=absolute --git-common-dir)" || return 1
    cp -R -- "$unit_current_git" "$unit_library_root" && rm -rf -- "${unit_root:?}/.git" && printf 'gitdir: %s\n' "$unit_library_root" >"$unit_root/.git" || return 1
    unit_settings=(PATH="$work/unit-bin:$work/bin:$PATH" CRASH_POINT_REPLAY_ROOT="$unit_root" STUB_UNIT_ROOT="$unit_root")
  }
  # unit_library_query count|red|plant-red <键> <测试函数>：门禁那一类在 prune-server 里的那份库（unit_library_root 这一处 git 目录的命名空间）按键前缀读：
  #   count 打 keys/ 下几条、red 打其中 verdict 是 red 的几条；plant-red 照 admission.py 判定库的写法（VerdictLibrary.record_red）造一条红
  unit_library_query() {
    python3 - "$replay_admission_module" "$unit_library_root" "$TEST_FUNCTIONS_GATE_FILE_NAME:test-functions" "$@" <<'PY_UNIT_LIBRARY'
import importlib.util, json, sys
specification = importlib.util.spec_from_file_location("admission_for_crash_point_replay_selftest", sys.argv[1])
admission = importlib.util.module_from_spec(specification)
specification.loader.exec_module(admission)
library = admission.VerdictLibrary(sys.argv[2], sys.argv[3])
action = sys.argv[4]
if action == "count":
    print(len(library.stored_keys()))
elif action == "red":
    print(sum(1 for _relative, value in library.store.scan("keys/") if json.loads(value).get("verdict") == "red"))
else:
    print(library.record_red("", sys.argv[6], sys.argv[5], "自证：造的一条红") or "recorded")
PY_UNIT_LIBRARY
  }
  local unit_alpha_reads_the_helper="crates/singlefs-verification/tests/alpha.rs::alpha_reads_the_helper" unit_alpha_stands_alone="crates/singlefs-verification/tests/alpha.rs::alpha_stands_alone"
  local unit_beta_stands_alone="crates/singlefs-verification/tests/beta/main.rs::beta_stands_alone" unit_exits=() unit_round_name unit_wrong unit_key unit_last_run
  local gate_request gate_kind gate_arguments=() request_tag request_value
  unit_states() { grep -E '^\s*· [^：]+：(绿|红)，' "$work/out" >"$work/unit-$1.out"; }   # <趟名>：这一趟逐条打的测试函数结局落进 unit-<趟名>.out
  unit_state_is() { grep -qF -- "· $2：$3" "$work/unit-$1.out"; }                        # <趟名> <测试函数> <绿，现判|绿，复用|红，现判>
  unit_count() { grep -cF -- "$2" "$work/unit-$1.out"; }                                   # <趟名> <一段字>：几条测试函数的结局里有它
  if selftest_cell_wanted reuse-three-questions; then
  # reuse-three-questions 的测试函数那一半（scripts/gate/lib_reuse_checks.py 文件头「一」）：unit-root 上剪枝接连四趟：库是空的那一趟、同一份输入、
  #   改一个哪条测试都不引的函数 not_read_by_any_test（闭包外）、改 alpha_reads_the_helper 引的 alpha_helper（闭包内）；照三问判（--from-output，只认输出里打了的项，所以另钉头一趟 beta_stands_alone 那一条在）、打规范输出
  unit_fresh_library
  for unit_round_name in fresh same unrelated unit; do
    case "$unit_round_name" in
      unrelated) printf '%s\n' 'pub fn alpha_helper() -> u32 {' '    1' '}' '' 'pub fn not_read_by_any_test() -> u32 {' '    8' '}' >"$unit_crate/src/lib.rs" ;;
      unit) printf '%s\n' 'pub fn alpha_helper() -> u32 {' '    1 + 0' '}' '' 'pub fn not_read_by_any_test() -> u32 {' '    8' '}' >"$unit_crate/src/lib.rs" ;;
    esac
    entry "${unit_settings[@]}" --
    unit_exits+=("$exit_code")
    unit_states "three-$unit_round_name"
  done
  cp -- "$work/unit-lib.rs" "$unit_crate/src/lib.rs"
  if ! python3 "$replay_script_directory/../gate/lib_reuse_checks.py" three-questions --script scripts/crash/crash-point-replay.sh --first "$work/unit-three-fresh.out" \
         --second "$work/unit-three-same.out" --unrelated "$work/unit-three-unrelated.out" --unit-change "$work/unit-three-unit.out" --affected "$unit_alpha_reads_the_helper" --from-output \
         >"$work/unit-three.log" 2>&1 || [[ "${unit_exits[*]}" != "0 0 0 0" ]] || ! unit_state_is three-fresh "$unit_beta_stands_alone" '绿，现判'; then
    cell_failed reuse-three-questions "按条复用的复用三问没过（四趟退 ${unit_exits[*]}；目录式目标 beta 的 beta_stands_alone 头一趟应有「绿，现判」一条，头一趟打了 $(unit_count three-fresh "$unit_beta_stands_alone") 条）：$(tr '\n' ' ' <"$work/unit-three.log" | selftest_clip_text 600)" \
      "照规范输出那一行 red 的那一问查：run_test_functions 里 plan_test_functions（test-plan）、test_functions_target_skips（--skip）与 record_test_functions_target（verdict-record）"
  fi
  grep '^REUSE_SELFTEST ' "$work/unit-three.log"
  fi
  if selftest_cell_wanted test-functions-reuse-shared-with-the-gate; then
  # test-functions-reuse-shared-with-the-gate：unit-root 上（库先清空）逐条点名三项的一趟把四条测试函数判绿写回门禁那一类判定库（keys/ 下恰好四份）；同一份输入不带参数的那一趟
  #   四条都复用、alpha、beta、gamma_amp 都不编不起；门禁不带特性那一趟的计划（真门禁取种类与请求的 test-functions-request、admission.py test-plan、不带 --feature）四条都是 reuse；
  unit_wrong=""
  unit_fresh_library
  entry "${unit_settings[@]}" -- --item alpha --item beta --item gamma
  unit_states shared-named
  [[ "$exit_code" == 0 && "$(unit_count shared-named '，现判')" == 4 && "$(unit_library_query count)" == 4 ]] \
    || unit_wrong+="逐条那一趟退 $exit_code、现判 $(unit_count shared-named '，现判') 条、门禁那一类的库（prune-server 里 $unit_library_root 的命名空间）keys/ 下 $(unit_library_query count) 条（要退 0、4 条、4 条）；"
  entry "${unit_settings[@]}" --
  unit_states shared-pruned
  [[ "$(unit_count shared-pruned '：绿，复用')" == 4 ]] && ! calls_have '^cargo test .*--test (alpha|beta|gamma_amp)( |$)' \
    || unit_wrong+="剪枝那一趟复用 $(unit_count shared-pruned '：绿，复用') 条（要 4 条，alpha、beta、gamma_amp 都不编不起）；"
  # 直接起的两条 admission.py 与 entry 同一份多机环境：起活那一方的回读握手不带进来，多机配置指 entry 那一份（第二台那一趟由 multi-host-run.sh 带上本台的两样）
  gate_request="$(env -u GPU_QUIESCE_ORIGIN_READ_BACK SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" \
    python3 "$replay_admission_module" test-functions-request "$TEST_FUNCTIONS_GATE_FILE_NAME" "$replay_judge_root" 2>&1)"
  gate_kind=""; gate_arguments=()
  while IFS=$'\t' read -r request_tag request_value; do
    case "$request_tag" in
      kind) gate_kind="$request_value" ;;
      argument) gate_arguments+=("$request_value") ;;
    esac
  done <<<"$gate_request"
  grep -q 'scripts/gate/admission.py" test-functions-request' "$replay_judge_root/.claude/gate.d/crash-point-replay.sh" "$replay_judge_root"/.claude/gate.d/lib/crash-point-replay-*.sh 2>/dev/null \
    || unit_wrong+="门禁 crash-point-replay（入口与 lib/ 下的库）的种类与请求不取 admission.py test-functions-request；"
  env -u GPU_QUIESCE_ORIGIN_READ_BACK SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" \
    PATH="$work/unit-bin:$work/bin:$PATH" python3 "$replay_admission_module" test-plan "$unit_root" "$gate_kind" singlefs-verification "$work/unit-gate.plan" \
    ${gate_arguments[@]+"${gate_arguments[@]}"} >"$work/unit-gate.plan.out" 2>&1
  [[ "$(awk -F'\t' 'NF >= 7 && $2 == "reuse"' "$work/unit-gate.plan.out" | grep -c .)" == 4 ]] \
    || unit_wrong+="门禁的计划（种类 $gate_kind）没把四条都复用：$(grep '^PLAN ' "$work/unit-gate.plan.out")；"
  [[ -z "$unit_wrong" ]] || cell_failed test-functions-reuse-shared-with-the-gate "逐条写进的判定，同一份输入不带参数的与门禁的计划应当照键复用（同一类、同一份库）：$unit_wrong" \
    "看 plan_test_functions 取的种类（admission.py test-functions-request）、record_test_functions_target 的 verdict-record 与 test_functions_target_all_reused"
  fi
  if selftest_cell_wanted item-plan-lines shard-plan-lines; then
  # item-plan-lines：--list-items --with-keys 每项一行 ITEM_PLAN（文件头「输出」）。unit-root 上库先清空、逐条记库一趟，再一趟（拿这一趟 test-plan 的计划当真值：几条复用、几条现判），
  #   再 --list-items --with-keys：alpha、beta、gamma 三项的 reusable 之和等于那一趟计划里复用的条数，to_judge 之和等于 4 件（崩溃枚举用例 2 + 放量分项 2）加那一趟计划里要现判的条数；
  #   整趟才跑的三类各一行；--item beta 只打 beta 一行；--list-items 带 --item 不带 --with-keys 退 2
  local plan_wrong="" plan_run_directory plan_reuse_truth plan_judge_truth plan_reusable_sum plan_to_judge_sum plan_list_lines plan_item_lines
  unit_fresh_library
  entry "${unit_settings[@]}" --
  entry "${unit_settings[@]}" --
  plan_run_directory="$(ls -td "$work/home/runs"/crash-point-replay-* 2>/dev/null | head -n 1)"
  plan_reuse_truth="$(awk -F'\t' 'NF >= 7 && $2 == "reuse"' "$plan_run_directory/test-functions.plan.out" 2>/dev/null | grep -c .)"
  plan_judge_truth="$(awk -F'\t' 'NF >= 7 && ($2 == "judge" || $2 == "refused")' "$plan_run_directory/test-functions.plan.out" 2>/dev/null | grep -c .)"
  entry "${unit_settings[@]}" CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys
  [[ "$exit_code" == 0 ]] || plan_wrong+="--list-items --with-keys 退 $exit_code；"
  plan_list_lines="$(grep '^ITEM_PLAN ' "$work/out")"
  plan_reusable_sum="$(awk '/^ITEM_PLAN item=(alpha|beta|gamma) / { for (i = 1; i <= NF; i++) if ($i ~ /^reusable=/) { sub(/^reusable=/, "", $i); sum += $i } } END { print sum + 0 }' <<<"$plan_list_lines")"
  plan_to_judge_sum="$(awk '/^ITEM_PLAN item=(alpha|beta|gamma) / { for (i = 1; i <= NF; i++) if ($i ~ /^to_judge=/) { sub(/^to_judge=/, "", $i); sum += $i } } END { print sum + 0 }' <<<"$plan_list_lines")"
  [[ "$(grep -c . <<<"$plan_list_lines")" == 6 ]] && grep -qE '^ITEM_PLAN item=whole-run:pinned-red-examples to_judge=0 reusable=0$' <<<"$plan_list_lines" \
    || plan_wrong+="应当 6 项各一行（alpha、beta、gamma 与整趟才跑的三类），钉回来的红例 0、0；"
  (( plan_reusable_sum == plan_reuse_truth )) || plan_wrong+="reusable 之和 $plan_reusable_sum 与计划里复用的 $plan_reuse_truth 条不同；"
  (( plan_to_judge_sum == 4 + plan_judge_truth )) || plan_wrong+="to_judge 之和 $plan_to_judge_sum 应当是 4 件加计划里现判的 $plan_judge_truth 条；"
  entry "${unit_settings[@]}" CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys --item beta
  plan_item_lines="$(grep '^ITEM_PLAN ' "$work/out")"
  { [[ "$exit_code" == 0 && "$(grep -c . <<<"$plan_item_lines")" == 1 ]] && grep -q '^ITEM_PLAN item=beta ' <<<"$plan_item_lines"; } \
    || plan_wrong+="--item beta 应当只打 beta 一行（退 $exit_code，打了 $(grep -c . <<<"$plan_item_lines") 行）；"
  entry "${unit_settings[@]}" -- --list-items --item beta
  [[ "$exit_code" == 2 ]] || plan_wrong+="--list-items --item beta（不带 --with-keys）应当退 2（退 $exit_code）；"
  [[ -z "$plan_wrong" ]] || cell_failed item-plan-lines "--list-items --with-keys 应当每项一行 ITEM_PLAN、条数照 test-plan 的计划数、点名了只算点名的项：$plan_wrong" \
    "看 report_the_item_plan 与 emit_item_plan_lines，与 test-plan 对测试函数那一段的计划"
  fi
  if selftest_cell_wanted shard-plan-lines; then
  # shard-plan-lines：按件分片的计划那一步另打 ITEM_PLAN shard=<片号> item=…（文件头「输出」）：每片每项一行（同一片同一项不出两行）、alpha 的两件落在不同的片上，
  #   各片各项之和等于上面不分片那一趟 --list-items --with-keys 的 ITEM_PLAN（一项的测试函数那一段只记在它头一次出现的那一片）
  local shard_lines_wrong="" shard_lines_sum list_lines_sum shard_lines_awk
  shard_lines_awk='{ item = ""; judge = 0; reuse = 0; for (i = 2; i <= NF; i++) { split($i, pair, "="); if (pair[1] == "item") item = pair[2]; if (pair[1] == "to_judge") judge = pair[2]; if (pair[1] == "reusable") reuse = pair[2] } j[item] += judge; r[item] += reuse } END { for (item in j) print item, j[item], r[item] }'
  default_shard_plan "${unit_settings[@]}" --
  [[ "$exit_code" == 0 ]] || shard_lines_wrong+="计划那一步退 $exit_code；"
  [[ "$(awk '/^ITEM_PLAN shard=/ { print $2, $3 }' "$work/out" | LC_ALL=C sort | uniq -d | grep -c .)" == 0 ]] || shard_lines_wrong+="同一片同一项出了两行；"
  [[ "$(awk '/^ITEM_PLAN shard=[0-9]+ item=alpha / { print $2 }' "$work/out" | LC_ALL=C sort -u | grep -c .)" -ge 2 ]] || shard_lines_wrong+="alpha 的两件应当落在两片、各一行；"
  shard_lines_sum="$(awk "/^ITEM_PLAN shard=/ $shard_lines_awk" "$work/out" | LC_ALL=C sort)"
  list_lines_sum="$(awk "/^ITEM_PLAN item=/ $shard_lines_awk" <<<"$plan_list_lines" | LC_ALL=C sort)"
  { [[ -n "$shard_lines_sum" ]] && [[ "$shard_lines_sum" == "$list_lines_sum" ]]; } \
    || shard_lines_wrong+="各片各项之和「$(tr '\n' '|' <<<"$shard_lines_sum")」与不分片那一趟「$(tr '\n' '|' <<<"$list_lines_sum")」不同；"
  [[ -z "$shard_lines_wrong" ]] || cell_failed shard-plan-lines "分片计划行应当每片每项一行、条数之和等于不分片那一趟：$shard_lines_wrong" "看 shard_report_the_item_plan 与 emit_item_plan_lines 的 shard 模式"
  fi
  if selftest_cell_wanted red-rejudged; then
  # red-rejudged：红过的不当绿复用（库先清空）：① 逐条一趟 alpha_reads_the_helper 判红（STUB_RED_TESTS）、alpha_stands_alone 判绿 ⇒ 再一趟（不再红）alpha_reads_the_helper 照现判、alpha_stands_alone 复用；
  #   ② beta_stands_alone 的键在门禁那一类的库里记着一条红（照 admission.py 判定库的写法造一份 verdict=red 的 keys/<键>.json）⇒ 下一趟 beta_stands_alone 照现判
  unit_wrong=""
  unit_fresh_library
  entry "${unit_settings[@]}" STUB_RED_TESTS=alpha_reads_the_helper -- --item alpha
  unit_states red-failing
  unit_state_is red-failing "$unit_alpha_reads_the_helper" '红，现判' || unit_wrong+="判红的那一趟 alpha_reads_the_helper 没判红；"
  unit_key="$(awk -F'\t' -v name="$unit_beta_stands_alone" '$1 == name { print $3 }' "$(ls -td "$work/home/runs"/crash-point-replay-* 2>/dev/null | head -n 1)/test-functions.plan.out" 2>/dev/null)"
  entry "${unit_settings[@]}" -- --item alpha
  unit_states red-again
  unit_state_is red-again "$unit_alpha_reads_the_helper" '绿，现判' || unit_wrong+="再一趟 alpha_reads_the_helper 没照现判（$(grep -F "$unit_alpha_reads_the_helper" "$work/unit-red-again.out")）；"
  unit_state_is red-again "$unit_alpha_stands_alone" '绿，复用' || unit_wrong+="再一趟 alpha_stands_alone 没复用（$(grep -F "$unit_alpha_stands_alone" "$work/unit-red-again.out")）；"
  if [[ "$unit_key" =~ ^[0-9a-f]{64}$ ]]; then
    unit_library_query plant-red "$unit_key" "$unit_beta_stands_alone" >/dev/null || unit_wrong+="造那一条红没写进临时服务里的库；"
    entry "${unit_settings[@]}" -- --item beta
    unit_states red-planted
    unit_state_is red-planted "$unit_beta_stands_alone" '绿，现判' || unit_wrong+="库里记着红的 beta_stands_alone 没照现判（$(grep -F "$unit_beta_stands_alone" "$work/unit-red-planted.out")）；"
  else
    unit_wrong+="判红那一趟的计划里取不出 beta_stands_alone 的键（「$unit_key」）；"
  fi
  [[ -z "$unit_wrong" ]] || cell_failed red-rejudged "红过的不当绿复用、下一趟照现判：$unit_wrong" \
    "看 record_test_functions_target（判绿的写回、判红的记红）与 scripts/gate/admission.py 的 VerdictLibrary.read（只认判绿的记录）、lookup 里认红的那一句"
  fi
  if selftest_cell_wanted red-revokes-green; then
  # red-revokes-green：记红、撤绿、下趟现判（文件头「按条复用」；库先清空）：逐条一趟 alpha 两条判绿入库 ⇒ 同键现判那一趟（准入模块的弄坏开关 verdict-lookup-skipped 不读库，
  #   造「同一把键又判了一遍」）alpha_reads_the_helper 判红（STUB_RED_TESTS）、照计划里的键记红、撤掉那把键的绿（判定库那一类里有一份 verdict 是 red 的）⇒
  #   下一趟逐条 alpha_reads_the_helper 照现判、alpha_stands_alone 复用
  unit_wrong=""
  unit_fresh_library
  entry "${unit_settings[@]}" -- --item alpha
  entry "${unit_settings[@]}" ADMISSION_BREAK=verdict-lookup-skipped STUB_RED_TESTS=alpha_reads_the_helper -- --item alpha
  unit_states red-rejudged-same-key
  unit_state_is red-rejudged-same-key "$unit_alpha_reads_the_helper" '红，现判' || unit_wrong+="同键现判那一趟 alpha_reads_the_helper 没判红；"
  (( $(unit_library_query red || echo 0) > 0 )) || unit_wrong+="判定库（prune-server 里 $unit_library_root 的命名空间）keys/ 下没有记红的那一条；"
  entry "${unit_settings[@]}" -- --item alpha
  unit_states red-after-rejudge
  unit_state_is red-after-rejudge "$unit_alpha_reads_the_helper" '绿，现判' || unit_wrong+="判红之后的那一趟 alpha_reads_the_helper 没照现判（$(grep -F "$unit_alpha_reads_the_helper" "$work/unit-red-after-rejudge.out")）；"
  unit_state_is red-after-rejudge "$unit_alpha_stands_alone" '绿，复用' || unit_wrong+="判红之后的那一趟 alpha_stands_alone 没复用（$(grep -F "$unit_alpha_stands_alone" "$work/unit-red-after-rejudge.out")）；"
  [[ -z "$unit_wrong" ]] || cell_failed red-revokes-green "判红要记红、撤掉同一把键原有的绿，下一趟照现判：$unit_wrong" \
    "看 record_test_functions_target 交 verdict-record 的 --red-names 与 scripts/gate/admission.py 的 VerdictLibrary.record_red、lookup 里认红的那一句"
  fi
  if selftest_cell_wanted test-functions-outside-git-exits-2; then
  # test-functions-outside-git-exits-2（文件头「按条复用」）：照 unit-root 拷一份不带 .git 的副本（临时目录里，往上找不到别的仓），在副本上跑剪枝
  #   ⇒ 整趟退 2、点名不是 git 工作树的顶、测试函数一个测试目标都不编不起（弄坏开关 test-functions-without-library-tolerated 下全部现判、往下跑）
  local unit_copy="$work/unit-copy"
  rm -rf -- "$unit_copy"
  cp -dR --preserve=timestamps,links -- "$unit_root" "$unit_copy" && rm -rf -- "${unit_copy:?}/.git"
  entry PATH="$work/unit-bin:$work/bin:$PATH" CRASH_POINT_REPLAY_ROOT="$unit_copy" STUB_UNIT_ROOT="$unit_copy" --
  if [[ "$exit_code" != 2 ]] || ! out_has "不是一棵 git 工作树的顶" || calls_have '^cargo test '; then
    cell_failed test-functions-outside-git-exits-2 "被判的根不是 git 工作树的顶时测试函数应当整趟退 2、点名它、一个测试目标都不编不起（退 $exit_code）" \
      "看 plan_test_functions 里 nolibrary 那一支与 run_test_functions 里照它退 2 那一段"
  fi
  rm -rf -- "${unit_copy:?}"
  fi
  if selftest_cell_wanted test-functions-domain-environment-keyed; then
  # test-functions-domain-environment-keyed（文件头「按条复用」test-functions-request 在测试函数 cargo 的环境里起）：alpha_helper 改成读按段登记文件与片长两个变量（原文里有两个名字的字面串），
  #   库先清空：逐条一趟 ⇒ 同一份输入再一趟 alpha 两条都复用 ⇒ 按段登记那一行的摘要换一个 ⇒ alpha_reads_the_helper 现判、alpha_stands_alone 复用 ⇒
  #   调用方设片长 131072 ⇒ alpha_reads_the_helper 又现判、alpha_stands_alone 复用
  unit_wrong=""
  printf '%s\n' 'pub fn alpha_helper() -> u32 {' \
    '    let _ = (std::env::var_os("SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY"), std::env::var_os("SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES"));' \
    '    1' '}' '' 'pub fn not_read_by_any_test() -> u32 {' '    7' '}' >"$unit_crate/src/lib.rs"
  cp -- "$unit_root/.claude/gate.d/crash-point-replay-segments.tsv" "$work/unit-segments-plain.tsv"
  unit_fresh_library
  entry "${unit_settings[@]}" -- --item alpha
  entry "${unit_settings[@]}" -- --item alpha
  unit_states domain-same
  [[ "$(unit_count domain-same '：绿，复用')" == 2 ]] || unit_wrong+="同一份输入再一趟 alpha 复用 $(unit_count domain-same '：绿，复用') 条（要 2 条）；"
  printf '%s\n' "# 自证的按段登记" "alpha	alpha_flow	0	7	$(printf 'fedcba9876543210%.0s' 1 2 3 4)" >"$unit_root/.claude/gate.d/crash-point-replay-segments.tsv"
  entry "${unit_settings[@]}" -- --item alpha
  unit_states domain-registry
  unit_state_is domain-registry "$unit_alpha_reads_the_helper" '绿，现判' || unit_wrong+="按段登记文件改了，读它的 alpha_reads_the_helper 没现判（$(grep -F "$unit_alpha_reads_the_helper" "$work/unit-domain-registry.out")）；"
  unit_state_is domain-registry "$unit_alpha_stands_alone" '绿，复用' || unit_wrong+="按段登记文件改了，不读它的 alpha_stands_alone 没复用；"
  entry "${unit_settings[@]}" SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=131072 -- --item alpha
  unit_states domain-slice
  unit_state_is domain-slice "$unit_alpha_reads_the_helper" '绿，现判' || unit_wrong+="片长没设换成 131072，读它的 alpha_reads_the_helper 没现判（$(grep -F "$unit_alpha_reads_the_helper" "$work/unit-domain-slice.out")）；"
  unit_state_is domain-slice "$unit_alpha_stands_alone" '绿，复用' || unit_wrong+="片长换了，不读它的 alpha_stands_alone 没复用；"
  cp -- "$work/unit-lib.rs" "$unit_crate/src/lib.rs"
  cp -- "$work/unit-segments-plain.tsv" "$unit_root/.claude/gate.d/crash-point-replay-segments.tsv"
  [[ -z "$unit_wrong" ]] || cell_failed test-functions-domain-environment-keyed "按段登记文件的内容、片长要进闭包读它们的那几条的键：$unit_wrong" \
    "看 plan_test_functions 起 test-functions-request 的那一份环境（要是交给测试函数 cargo 的同一份）与 scripts/gate/admission.py 的 domain_variable_tokens、domain_variable_lines"
  fi
  if selftest_cell_wanted test-functions-plan-failure-red; then
  # test-functions-plan-failure-red：unit-root 放 .unit-verdict-sample/plan-failure 写 exit-1（admission.py 文件头「按条复用失灵」：test-plan 不算键、退 1）⇒ 没有计划就排不掉登记的用例，
  #   测试函数那一段一个测试目标都不起，点到测试目标的项（alpha、gamma 都在里面）都红、整趟退 1、原因写按条复用失灵
  printf 'exit-1\n' >"$unit_root/.unit-verdict-sample/plan-failure"
  entry "${unit_settings[@]}" --
  rm -f -- "$unit_root/.unit-verdict-sample/plan-failure"
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'verdict=red' || ! item_has gamma 'verdict=red' || ! out_has '按条复用失灵' \
     || calls_have '^cargo test .*--test alpha -- --nocapture'; then
    cell_failed test-functions-plan-failure-red "按条复用失灵（test-plan 退 1）应当不起测试目标（没有计划排不掉登记的用例）、点到测试目标的项都红、整趟退 1、原因写按条复用失灵" \
      "看 plan_test_functions 里交 admission.py plan-failure 那一段与 run_test_functions 末尾 functions_plan_state=failed 那一支"
  fi
  fi
  if selftest_cell_wanted test-functions-plan-unavailable; then
  # test-functions-plan-unavailable：unit-root 放 .unit-verdict-sample/plan-failure 写 exit-75（admission.py 文件头「按条复用失灵」：test-plan 装 prune-server 不可用、退 75）
  #   ⇒ 整趟退 75、说 prune-server 不可用、不说按条复用失灵，测试函数一个测试目标都不编不起（弄坏开关 test-functions-plan-unavailable-as-failure 下照失灵办、退 1）
  printf 'exit-75\n' >"$unit_root/.unit-verdict-sample/plan-failure"
  entry "${unit_settings[@]}" --
  rm -f -- "$unit_root/.unit-verdict-sample/plan-failure"
  if [[ "$exit_code" != 75 ]] || ! out_has 'prune-server 不可用' || out_has '按条复用失灵' || calls_have '^cargo test '; then
    cell_failed test-functions-plan-unavailable "测试函数 test-plan 退 75（prune-server 不可用）应当整趟退 75、说不可用、不判按条复用失灵、一个测试目标都不编（实测退 $exit_code）" \
      "看 plan_test_functions 里 plan-failure 退 75 那一支（functions_plan_state=unavailable）与 run_test_functions 里照它退 75 那一段"
  fi
  fi
  if selftest_cell_wanted registered-excluded; then
  # registered-excluded：登记成 crash-case: 的用例（stage-inputs.tsv 的 crash-case:alpha 登记了 alpha_full）与放量分项（ITEMS 的 alpha-amp 登记了 alpha_amplified）不进测试函数那一段：
  #   unit-root 的 alpha.rs 临时加这两条，逐条点名 alpha ⇒ 它们经 --skip 排掉（cargo 参数里有 --skip alpha_full 与 --skip alpha_amplified）、不判不记（逐条结局里没有它们）、
  #   alpha_stands_alone 照判（弄坏开关 registered-not-excluded 下它们也被判，参数里没有 --skip）
  cp -- "$unit_crate/tests/alpha.rs" "$work/unit-alpha.rs"
  printf '\n#[test]\nfn alpha_full() {\n    assert_eq!(5, 5);\n}\n\n#[test]\nfn alpha_amplified() {\n    assert_eq!(6, 6);\n}\n' >>"$unit_crate/tests/alpha.rs"
  unit_fresh_library
  entry "${unit_settings[@]}" -- --item alpha
  unit_states registered
  if [[ "$exit_code" != 0 ]] || ! calls_have '^cargo test .*--test alpha .*--skip alpha_amplified --skip alpha_full' \
     || grep -qE 'alpha_(full|amplified)' "$work/unit-registered.out" || ! unit_state_is registered "$unit_alpha_stands_alone" '绿，现判'; then
    cell_failed registered-excluded "按项跑时登记成 crash-case: 的用例与放量分项应当经 --skip 排掉、不进测试函数那一段的判定与逐条结局（退 $exit_code；调用 $(grep '^cargo test .*--test alpha ' "$calls" | selftest_clip_text 200)）" \
      "看 plan_test_functions 交给 test-plan 的 --exclude-registered 与 test_functions_target_skips 取的 plan-skip-names（admission.py 把 registered 的行也列进去）"
  fi
  cp -- "$work/unit-alpha.rs" "$unit_crate/tests/alpha.rs"
  fi
  if selftest_cell_wanted last-run-list; then
  # last-run-list（scripts/gate/lib_reuse_checks.py 文件头「三」）：剪枝一趟 gamma 那一项放量红，上次红的清单是 gamma；再一趟 gamma 的放量本次未跑（gamma 那一项没判），
  #   清单照留 gamma；再一趟全绿，清单成空文件
  local last_run_file="$work/last-runs/scripts__crash__crash-point-replay.sh.tsv" last_red
  rm -f -- "$last_run_file"
  entry STUB_AMP_RED=gamma-amp --
  last_red="$(cat "$last_run_file" 2>/dev/null)"
  entry STUB_AMP_NOT_RUN=gamma-amp --
  unit_last_run="$(cat "$last_run_file" 2>/dev/null)"
  entry --
  if [[ "$last_red" != gamma || "$unit_last_run" != gamma || ! -f "$last_run_file" || -s "$last_run_file" ]]; then
    cell_failed last-run-list "上次红的清单不对：gamma 红的那一趟之后是「$last_red」，gamma 本次未跑的那一趟之后是「$unit_last_run」（要照留 gamma），全绿那一趟之后$([[ -f "$last_run_file" ]] && echo "是「$(cat "$last_run_file")」" || echo 没有这份文件)" \
      "看收尾写 last-run 那一段：summarize 交出的红的项（exit.red-items）与这一趟真判过的项（exit.judged-items），经 --ran 只换判过的项"
  fi
  fi
  if selftest_cell_wanted test-functions-reuse-counted; then
  # test-functions-reuse-counted：测试函数 alpha 那个测试目标的日志里有复用计数行（resumed_slices=3）⇒ alpha 那一行 test_functions_reused=3；beta 没有那一行 ⇒ unknown、计进取不出的、成功句点名
  entry STUB_REUSE_TARGETS=alpha --
  if [[ "$exit_code" != 0 ]] || ! item_has alpha 'test_functions_reused=3 test_functions_reuse_unknown=0 ' || ! item_has beta 'test_functions_reused=unknown test_functions_reuse_unknown=1 ' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL .* test_functions_reused_known=3 test_functions_reuse_unknown=[1-9][0-9]* ' "$work/out" \
     || ! out_has "测试函数 " || ! grep -qE '复用 3 块（另有 [0-9]+ 个测试目标取不出复用数：[^）]*beta' "$work/out"; then
    cell_failed test-functions-reuse-counted "测试函数日志里有复用计数行时 ITEM 行应当报同一个数（alpha：test_functions_reused=3），没有时报取不出（beta：test_functions_reused=unknown、计进 test_functions_reuse_unknown）、成功句点名取不出的测试目标" \
      "看 judge_test_functions_log 里数 resumed_slices= 那一段与 summarize 的 functions_reused"
  fi
  fi
  if selftest_cell_wanted test-variables-outside-selftest; then
  # test-variables-outside-selftest：不带自证标记、带了 CRASH_POINT_REPLAY_ROOT ⇒ 退 2、什么都不起、输出点名 CRASH_POINT_REPLAY_ROOT
  : >"$calls"; cells=$((cells + 1))
  env PATH="$work/bin:$PATH" STUB_CALLS="$calls" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
    SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" CRASH_POINT_REPLAY_ROOT="$root" \
    CRASH_POINT_REPLAY_BREAK="$replay_break" bash "$replay_script" >"$work/out" 2>&1
  exit_code=$?
  # 光看退 2 不够：守卫没了，后面的「不经服务起」也退 2、什么都不起；认得出是守卫拒的只有它点名变量的那句话
  [[ "$exit_code" == 2 && ! -s "$calls" ]] && out_has "平常跑时带了只供自证的变量：" && out_has "CRASH_POINT_REPLAY_ROOT" \
    || cell_failed test-variables-outside-selftest "平常跑时带了只供自证的变量应当退 2、并点名那几个变量" "看开头拒只供自证变量那一段"
  # 往引擎送的变量（文件头「往引擎送的变量」）：测试函数每一趟 cargo test 与放量入口那一趟都要带到
  fi
  local test_functions_runs_pattern='^cargo test --release -p singlefs-verification --test [a-z0-9_]+ -- --nocapture '
  local registry="$root/.claude/gate.d/crash-point-replay-segments.tsv"
  every_test_functions_run_has() { # <那一行里要有的一段（正则）>：测试函数至少跑了一趟，且每一趟 cargo test 的调用记录都有它
    local all
    all="$(calls_count "$test_functions_runs_pattern")"
    (( all > 0 )) && [[ "$(calls_count "${test_functions_runs_pattern}.*$1")" == "$all" ]]
  }
  if selftest_cell_wanted segments-registry-sent test-functions-threads-from-service; then
  # segments-registry-sent：每一趟都带按段登记文件的绝对路径；这一趟另设 CARGO_BUILD_JOBS=3 当服务交的线程数，下面 test-functions-threads-from-service 判同一趟
  entry CARGO_BUILD_JOBS=3 --
  if [[ "$exit_code" != 0 ]] || ! every_test_functions_run_has " segments=\[$registry\]\$" \
     || ! calls_have "^pipeline-env segments=\[$registry\]\$"; then
    cell_failed segments-registry-sent "测试函数与放量每一趟都应当带按段登记文件的绝对路径（SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=$registry）" \
      "看「往引擎送的变量」那一段的 export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY"
  fi
  fi
  if selftest_cell_wanted test-functions-threads-from-service; then
  # test-functions-threads-from-service：测试函数每一趟 cargo 直接起、不套线程包装，调用方环境里服务交的 CARGO_BUILD_JOBS（上一趟设的 3）原样带到
  if [[ "$exit_code" != 0 ]] || ! every_test_functions_run_has " jobs=\[3\] segments="; then
    cell_failed test-functions-threads-from-service "测试函数每一趟 cargo 都应当原样收到服务交的线程数（调用方环境 CARGO_BUILD_JOBS=3）" \
      "看 run_test_functions 拼的 env：不清掉、不改写 CARGO_BUILD_JOBS 与 RUST_TEST_THREADS，cargo 前面不套线程包装"
  fi
  fi
  if selftest_cell_wanted segments-registry-refused; then
  # segments-registry-refused：按段登记文件核不过（项名不在合表里）⇒ 退 2、什么都不起
  cp -- "$registry" "$work/segments-saved.tsv"
  printf '%s\n' "# 自证：核不过的按段登记" "ghost	ghost_flow	0	7	$(printf '0123456789abcdef%.0s' 1 2 3 4)" >"$registry"
  entry --
  cp -- "$work/segments-saved.tsv" "$registry"
  if [[ "$exit_code" != 2 || -s "$calls" ]] || ! out_has "项名 ghost 不是合表"; then
    cell_failed segments-registry-refused "按段登记文件核不过应当退 2、什么都不起、逐处列出问题" \
      "看「往引擎送的变量」那一段开头的 lib_crash_point_replay_items.py segments"
  fi
  fi
  if selftest_cell_wanted test-functions-run-condition-not-met; then
  # test-functions-run-condition-not-met：alpha 那个测试目标的日志里有给定状态表借不到显存那一行（cargo 退 101）⇒ alpha 记本次未跑、写原因、退 78，别的照绿
  entry STUB_RUN_CONDITION_TARGETS=alpha --
  if [[ "$exit_code" != 78 ]] || ! item_has alpha 'exit=78 verdict=not-run ' || ! item_has beta 'verdict=green' \
     || ! out_has "本次未跑 alpha：运行条件不满足：给定状态表的判器借不到显存（$CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX table=stub-table"; then
    cell_failed test-functions-run-condition-not-met "测试函数日志里有给定状态表借不到显存那一行（$CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX），那一项应当记本次未跑、写原因，整趟退 78" \
      "看 judge_test_functions_log 里 condition_lines 那一支与 summarize 的 any_78"
  fi
  fi
  if selftest_cell_wanted flow-coverage-table; then
  # flow-coverage-table：剪枝跑完，这一趟的日志目录里落了覆盖表与记录（scripts/crash/crash-flow-coverage.py 的 run-directory、table），测试函数打的覆盖行在里面
  entry STUB_FLOW_LINE=1 --
  local flow_run_directory
  flow_run_directory="$(ls -td "$work/home/runs"/crash-point-replay-* 2>/dev/null | head -n 1)"
  if [[ "$exit_code" != 0 || -z "$flow_run_directory" ]] || ! grep -q '| selftest_entry' "$flow_run_directory/crash-flow-coverage.md" 2>/dev/null \
     || ! grep -q $'^CRASH_FLOW_RECORD\tcoverage\talpha\tselftest_entry\t' "$flow_run_directory/crash-flow-records.tsv" 2>/dev/null; then
    cell_failed flow-coverage-table "跑完应当在这一趟的日志目录里落覆盖表 crash-flow-coverage.md 与记录 crash-flow-records.tsv，测试函数打的覆盖行在里面（日志目录 ${flow_run_directory:-没找到}）" \
      "看 summarize 之后落覆盖表那一段与 scripts/crash/crash-flow-coverage.py 的 run-directory、table"
  fi
  fi
  if selftest_cell_wanted rocksdb-entry-refused; then
  # rocksdb-entry-refused：小仓的 验证包依赖 rocksdb、进程环境里四个取值是空的，PATH 最前面的假 cargo 不带 rocksdb-prebuilt.py 认的标记
  #   ⇒ 起任何 cargo 之前 rocksdb-prebuilt.py entry 判红、退 78，假 cargo 与两套驱动一次都没起（别的格小仓里这份清单没有 rocksdb 依赖，entry 报不判；跑完照底稿换回）
  printf '%s\n' '[package]' 'name = "singlefs-verification"' 'version = "0.0.0"' '[dependencies]' 'rocksdb = { version = "=0.25.0", optional = true }' \
    >"$root/crates/singlefs-verification/Cargo.toml"
  entry ROCKSDB_LIB_DIR= ROCKSDB_STATIC= SNAPPY_LIB_DIR= SNAPPY_STATIC= --
  if [[ "$exit_code" != 78 ]] || [[ -s "$calls" ]] || ! out_has "rocksdb-prebuilt entry：" || ! out_has "这一趟不起 cargo"; then
    cell_failed rocksdb-entry-refused "验证包依赖 rocksdb 而进程环境里没有预编库的取值，应当在起 cargo 之前退 78、一个 cargo 与驱动都不起、输出里有 rocksdb-prebuilt entry 的红句" \
      "看定档之后 replay_rocksdb_entry_holds 那一段（scripts/hosts/rocksdb-prebuilt.py entry）"
  fi
  cp -- "$work/root-crate-manifest.toml" "$root/crates/singlefs-verification/Cargo.toml"

  fi
  # ── 指定范围的崩溃域上限（文件头「不带 --item 的跑法…」；估法与记录来源在 lib_crash_point_replay_items.py 文件头「指定范围的崩溃域上限」）
  #   小仓里 gamma 只有放量分项 gamma-amp，beta 只有崩溃点重放 crash-case:beta，alpha 两样都有。假 cargo 的读库程序 items 子命令每项按 STUB_ITEM_TABLE（逐项）或 STUB_ITEM_RECORDED、
  #   STUB_ITEM_TO_JUDGE、STUB_ITEM_REUSABLE（各项同一份）答块账（都不给是录下的块全部可复用）；放量分项的运行记录放 $work/home/runs/entry-scoped-*/<分项>.log（共享根里同一处），
  #   每格要的记录建在格里，这一段末了删
  local scoped_runs="$work/home/runs" scoped_wrong scoped_run_directory scoped_scope_file domain_lines domain_order domain_items domain_name domain_reference
  scoped_record() { # <目录名> <分项> <展开深度> <态数> <几天前>：放一份放量分项的运行记录（一行 CRASH_AMPLIFICATION，修改时刻照几天前定，新旧次序靠它）
    mkdir -p -- "$scoped_runs/$1"
    printf 'CRASH_AMPLIFICATION flow=%s mode=single_host_cpu states_checked_here=0 blocks_checked_here=0 unjudged_blocks=0 red=0 states_reused=%s exhaustive=true expand_up_to=%s\n' \
      "$2" "$4" "$3" >"$scoped_runs/$1/$2.log"
    touch -d "$5 days ago" -- "$scoped_runs/$1/$2.log"
  }
  if selftest_cell_wanted scoped-domain-over-limit; then
  # scoped-domain-over-limit：gamma 上一趟不缩域真跑 300000 态（展开深度 30，较新）、更早缩到 6 的那一趟 225 态，库里 4 块键都对不上 ⇒ 估 300000 态，超上限退 79，
  #   说明写能压到上限以内的 n（6）与两条出路，不起放量入口、门禁与 cargo test
  scoped_wrong=""
  scoped_record entry-scoped-old gamma-amp 6 225 2
  scoped_record entry-scoped-new gamma-amp 30 300000 1
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=4 STUB_ITEM_REUSABLE=0 -- --item gamma
  [[ "$exit_code" == 79 ]] || scoped_wrong+="估 300000 态超上限应当退 79（退 $exit_code）；"
  out_has "300000" || scoped_wrong+="说明里应当写估的态数；"
  out_has "--crash-expand-up-to 6" || scoped_wrong+="说明里应当写能压到上限以内的 --crash-expand-up-to 6；"
  out_has "留给提交时不指定范围的那一趟" || scoped_wrong+="说明里应当写第二条出路（留给提交时不指定范围的那一趟）；"
  if calls_have '^(pipeline-entry|enumeration-gate|cargo test) '; then scoped_wrong+="拒了之后不该起放量入口、门禁与 cargo test；"; fi
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-over-limit "带 --item、没缩域，估数超过 200000 态应当退 79、说明里写 --crash-expand-up-to 6 与留给提交时那一趟、什么都不起：$scoped_wrong" \
    "看 scoped_domain_guard 与 lib_crash_point_replay_items.py 的 run_scoped_domain、refusal_text"
  fi
  if selftest_cell_wanted scoped-domain-reusable-passes; then
  # scoped-domain-reusable-passes：同样的记录，库里 4 块键都对得上 ⇒ 现判 0 态，放行，放量入口照起
  scoped_wrong=""
  scoped_record entry-scoped-old gamma-amp 6 225 2
  scoped_record entry-scoped-new gamma-amp 30 300000 1
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=0 STUB_ITEM_REUSABLE=4 -- --item gamma
  [[ "$exit_code" == 0 ]] || scoped_wrong+="点名项全部可复用应当放行（退 $exit_code）；"
  out_has "全部复用，现判 0 态" || scoped_wrong+="应当写出这一项全部复用、现判 0 态；"
  calls_have '^pipeline-entry ' || scoped_wrong+="放行之后放量入口应当照起；"
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-reusable-passes "库里录下的块键都对得上，不论上一趟的态数多大都应当放行、现判 0 态：$scoped_wrong" \
    "看 lib_crash_point_replay_items.py 的 estimate_subject：录下的块大于 0 且键对不上的块是 0 就是现判 0 态"
  fi
  if selftest_cell_wanted scoped-domain-narrowed-passes; then
  # scoped-domain-narrowed-passes：给了 --crash-expand-up-to 6（小于引擎常量 30）⇒ 缩过域，放行，不起读库程序
  scoped_wrong=""
  : >"$work/item-calls"
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=4 STUB_ITEM_REUSABLE=0 STUB_ITEM_CALLS="$work/item-calls" -- --item gamma --crash-expand-up-to 6
  [[ "$exit_code" == 0 ]] || scoped_wrong+="缩过域应当放行（退 $exit_code）；"
  out_has "缩过域" || scoped_wrong+="应当写缩过域；"
  [[ ! -s "$work/item-calls" ]] || scoped_wrong+="缩过域不判上限，不该起读库程序（起了：$(tr '\n' '|' <"$work/item-calls" | selftest_clip_text 200)）；"
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-narrowed-passes "带了 --crash-expand-up-to 6 应当放行、不读库、不判上限：$scoped_wrong" \
    "看 scoped_domain_guard 交 --expand-up-to 与 run_scoped_domain 开头缩域那一支"
  fi
  if selftest_cell_wanted scoped-domain-unknown-refused; then
  # scoped-domain-unknown-refused：alpha 没有任何运行记录、库里一块都没录 ⇒ 估不出，不放行，退 79
  scoped_wrong=""
  entry STUB_ITEM_RECORDED=0 STUB_ITEM_TO_JUDGE=0 STUB_ITEM_REUSABLE=0 -- --item alpha
  [[ "$exit_code" == 79 ]] || scoped_wrong+="估不出应当退 79（退 $exit_code）；"
  out_has "算不出" || scoped_wrong+="说明里应当写算不出；"
  out_has "--crash-expand-up-to" || scoped_wrong+="说明里应当写 --crash-expand-up-to 这条出路；"
  if calls_have '^(pipeline-entry|enumeration-gate|cargo test) '; then scoped_wrong+="拒了之后不该起放量入口、门禁与 cargo test；"; fi
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-unknown-refused "库里没录这一项的块、也没有上一趟的运行记录，应当退 79、说明写算不出与出路、什么都不起：$scoped_wrong" \
    "看 lib_crash_point_replay_items.py 的 estimate_subject 算不出那一支与 refusal_text"
  fi
  if selftest_cell_wanted scoped-domain-shrunk-record-skipped; then
  # scoped-domain-shrunk-record-skipped：只剩缩到 6 的那一趟记录（225 态）、库里 4 块键都对不上 ⇒ 缩过域的记录不当全域，估不出，退 79
  scoped_wrong=""
  scoped_record entry-scoped-old gamma-amp 6 225 2
  rm -rf -- "${scoped_runs:?}/entry-scoped-new"
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=4 STUB_ITEM_REUSABLE=0 -- --item gamma
  [[ "$exit_code" == 79 ]] || scoped_wrong+="只有缩过域的记录应当退 79（退 $exit_code）；"
  out_has "没有不缩域那一趟的真跑记录" || scoped_wrong+="说明里应当写没有不缩域那一趟的真跑记录；"
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-shrunk-record-skipped "缩过域那一趟的态数少，拿来当全域会估小，只有它时应当算不出、退 79：$scoped_wrong" \
    "看 lib_crash_point_replay_items.py 的 full_prior：只认展开深度等于引擎常量的记录"
  fi
  if selftest_cell_wanted scoped-domain-enumeration-records; then
  # scoped-domain-enumeration-records：beta 只有崩溃点重放部分。头一趟 --item beta（库里全部可复用，放行）假门禁打每条用例日志放哪个目录、放一份 states=1234 的日志 ⇒
  #   日志旁写 .scope（expand_up_to=full、case_states=beta:1234）；再 --list-items --with-keys --item beta（库里 3 块都要判）⇒ ITEM_DOMAIN 取那份日志的 1234 态
  scoped_wrong=""
  rm -rf -- "${work:?}/case-logs"
  entry STUB_CASE_LOG_DIRECTORY="$work/case-logs" -- --item beta
  [[ "$exit_code" == 0 ]] || scoped_wrong+="--item beta（库里全部可复用）应当放行（退 $exit_code）；"
  scoped_run_directory="$(ls -td "$scoped_runs"/crash-point-replay-* 2>/dev/null | head -n 1)"
  scoped_scope_file="$scoped_run_directory/enumeration.beta.log.scope"
  { grep -qxF 'expand_up_to=full' "$scoped_scope_file" && grep -qxF 'case_states=beta:1234' "$scoped_scope_file"; } 2>/dev/null \
    || scoped_wrong+="崩溃点重放部分跑完应当在日志旁写 .scope（expand_up_to=full、case_states=beta:1234；文件 ${scoped_scope_file:-没找到}：$(tr '\n' '|' <"$scoped_scope_file" 2>/dev/null)）；"
  entry STUB_ITEM_RECORDED=3 STUB_ITEM_TO_JUDGE=3 STUB_ITEM_REUSABLE=0 CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys --item beta
  grep -qxF 'ITEM_DOMAIN item=beta states_to_judge=1234 source=prior-run' "$work/out" \
    || scoped_wrong+="下一趟估上限应当读到那份日志的 1234 态（ITEM_DOMAIN 行：$(grep '^ITEM_DOMAIN ' "$work/out" | tr '\n' '|')）；"
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-enumeration-records "崩溃点重放部分每跑完一趟应当在日志旁写 .scope，下一趟估上限时读那一趟的用例日志里的态数：$scoped_wrong" \
    "看 record_enumeration_scope 与 lib_crash_point_replay_items.py 的 enumeration_totals"
  fi
  if selftest_cell_wanted item-domain-line; then
  # item-domain-line：--list-items --with-keys 每个列出的项一行 ITEM_DOMAIN（ITEM_PLAN 之后、「键」几行之前）；gamma 上一趟全域 300000 态、库里 4 块里 1 块键对不上 ⇒ 75000 态（按块数比例），
  #   alpha 没有记录 ⇒ unknown，整趟才跑的类 0 态；点名了只打点名的项
  scoped_wrong=""
  scoped_record entry-scoped-new gamma-amp 30 300000 0
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=1 STUB_ITEM_REUSABLE=3 CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys
  domain_lines="$(grep '^ITEM_DOMAIN ' "$work/out")"
  [[ "$exit_code" == 0 && "$(grep -c . <<<"$domain_lines")" == 6 ]] || scoped_wrong+="应当 6 项各一行（退 $exit_code，打了 $(grep -c . <<<"$domain_lines") 行）；"
  grep -qxF 'ITEM_DOMAIN item=gamma states_to_judge=75000 source=prior-run-scaled' <<<"$domain_lines" || scoped_wrong+="gamma 应当是 75000 态（按块数比例）；"
  grep -qxF 'ITEM_DOMAIN item=alpha states_to_judge=unknown source=unknown' <<<"$domain_lines" || scoped_wrong+="alpha 没有记录应当写 unknown；"
  grep -qxF 'ITEM_DOMAIN item=whole-run:pinned-red-examples states_to_judge=0 source=no-crash-domain' <<<"$domain_lines" || scoped_wrong+="整趟才跑的类没有崩溃域应当写 0；"
  domain_order="$(awk '/^ITEM_PLAN /{ plan = NR } /^ITEM_DOMAIN /{ if (!first) first = NR; last = NR } /^键\t/{ if (!key) key = NR } END { print (plan < first && last < key) ? "ok" : "bad" }' "$work/out")"
  [[ "$domain_order" == ok ]] || scoped_wrong+="ITEM_DOMAIN 应当在 ITEM_PLAN 之后、「键」几行之前；"
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=1 STUB_ITEM_REUSABLE=3 CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys --item gamma
  [[ "$exit_code" == 0 && "$(grep -c '^ITEM_DOMAIN ' "$work/out")" == 1 ]] && grep -q '^ITEM_DOMAIN item=gamma ' "$work/out" || scoped_wrong+="--item gamma 应当只打 gamma 一行（退 $exit_code）；"
  [[ -z "$scoped_wrong" ]] || cell_failed item-domain-line "--list-items --with-keys 应当每项一行 ITEM_DOMAIN、态数照估法、点名了只打点名的项：$scoped_wrong" \
    "看 report_the_item_domain 与 lib_crash_point_replay_items.py 的 run_scoped_domain report 那一支"
  fi
  if selftest_cell_wanted item-domain-single-reader; then
  # item-domain-single-reader：--list-items --with-keys 列了 6 项（有崩溃域的是 alpha、beta、gamma）⇒ 只起一次读库程序，items 之后照项的次序 alpha、beta、gamma
  #   各一组 --item <项> <设点文件>…；ITEM_DOMAIN 与逐项各交一趟判法（scoped-domain report --item <项>，每趟各起一次读库程序）打的行依次拼起来逐字相同。
  #   假 cargo 照 STUB_ITEM_TABLE 给三项不同的块账，行归错了项照样红：gamma 上一趟全域 300000 态、4 块里 1 块要判 ⇒ 75000 态；beta 3 块全部复用 ⇒ 0 态
  scoped_wrong=""
  scoped_record entry-scoped-new gamma-amp 30 300000 0
  printf '%s\n' 'alpha 2 2 0' 'beta 3 0 3' 'gamma 4 1 3' >"$work/item-table"
  : >"$work/item-calls"
  : >"$work/item-calls-per-item"
  entry STUB_ITEM_TABLE="$work/item-table" STUB_ITEM_CALLS="$work/item-calls" CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys
  [[ "$exit_code" == 0 ]] || scoped_wrong+="--list-items --with-keys 应当退 0（退 $exit_code）；"
  [[ "$(grep -c . "$work/item-calls")" == 1 ]] \
    || scoped_wrong+="应当只起一次读库程序（起了 $(grep -c . "$work/item-calls") 次：$(tr '\n' '|' <"$work/item-calls" | selftest_clip_text 300)）；"
  grep -qE '^items --item alpha [^-].* --item beta [^-].* --item gamma [^-]' "$work/item-calls" \
    || scoped_wrong+="那一次应当是 items 子命令、照项的次序 alpha、beta、gamma 各一组 --item <项> <设点文件>…；"
  domain_lines="$(grep '^ITEM_DOMAIN ' "$work/out")"
  domain_items="$(sed -nE $'s/^([a-z0-9][a-z0-9:-]*)\t(测试函数|本次未跑).*/\\1/p' "$work/out")"   # 列项那几行的项名，照列的次序
  domain_reference=""
  while IFS= read -r domain_name; do
    [[ -n "$domain_name" ]] || continue
    domain_reference+="$(PATH="$work/bin:$PATH" STUB_ITEM_TABLE="$work/item-table" STUB_ITEM_CALLS="$work/item-calls-per-item" \
      python3 "$replay_script_directory/lib_crash_point_replay_items.py" scoped-domain report --root "$root" --runs "$scoped_runs" --gpu 0 --item "$domain_name" 2>/dev/null)"$'\n'
  done <<<"$domain_items"
  [[ -n "$domain_lines" && "$domain_lines"$'\n' == "$domain_reference" ]] \
    || scoped_wrong+="ITEM_DOMAIN 应当与逐项各交一趟判法打的行依次拼起来逐字相同（打的：$(tr '\n' '|' <<<"$domain_lines" | selftest_clip_text 300)；逐项：$(tr '\n' '|' <<<"$domain_reference" | selftest_clip_text 300)）；"
  grep -qxF 'ITEM_DOMAIN item=gamma states_to_judge=75000 source=prior-run-scaled' <<<"$domain_lines" || scoped_wrong+="gamma 应当是 75000 态（块账照 gamma 那一行）；"
  grep -qxF 'ITEM_DOMAIN item=beta states_to_judge=0 source=reusable' <<<"$domain_lines" || scoped_wrong+="beta 应当全部复用、0 态（块账照 beta 那一行）；"
  [[ -z "$scoped_wrong" ]] || cell_failed item-domain-single-reader "--list-items --with-keys 应当只起一次读库程序（items 子命令）、每项一行、与逐项起的行逐字相同：$scoped_wrong" \
    "看 report_the_item_domain（列出的项一起交一趟判法）与 lib_crash_point_replay_items.py 的 read_library_items"
  rm -f -- "${work:?}/item-table" "$work/item-calls" "$work/item-calls-per-item"
  fi
  if selftest_cell_wanted scoped-domain-checked-handshake; then
  # scoped-domain-checked-handshake：放量部分起放量入口时交 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1（放量入口不再判一遍）；运行记录先删，这一格只看交没交
  scoped_wrong=""
  rm -rf -- "${scoped_runs:?}"/entry-scoped-*
  entry -- --item gamma
  calls_have '^pipeline-env scoped-checked=\[1\]$' || scoped_wrong+="放量入口应当收到 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1（退 $exit_code）；"
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-checked-handshake "放量部分起放量入口应当交已判过上限的标记：$scoped_wrong" "看 run_pipeline_part 里 checked_environment"
  rm -rf -- "${scoped_runs:?}"/entry-scoped-* "${work:?}/case-logs" "${work:?}/item-calls"
  fi
  if selftest_cell_wanted failure-text-clipped-in-character; then
  # failure-text-clipped-in-character：cell_failed 转打的调用记录与子进程输出比上限长、上限落在汉字中间 ⇒ 它打的三行逐字节是合法的 UTF-8，说明那一句原样在
  #   （弄坏开关 failure-text-cut-bytes：半个字原样打出来）。探针在子 shell 里起（failures 不算数），起完清掉调用记录与输出，这一格自己判红时打的是干净的
  local clip_wrong="" clip_character
  cells=$((cells + 1))
  { printf 'x'; for ((clip_character = 0; clip_character < 400; clip_character++)); do printf '中'; done; printf '\n'; } >"$calls"
  { printf '  ✗ x'; for ((clip_character = 0; clip_character < 400; clip_character++)); do printf '中'; done; printf '\n'; } >"$work/out"
  ( cell_failed probe "探针那一句" "探针的出路" ) 2>"$work/clip-report"
  : >"$calls"; : >"$work/out"
  iconv -f UTF-8 -t UTF-8 "$work/clip-report" >/dev/null 2>&1 || clip_wrong+="三行里有不合法的 UTF-8；"
  grep -qF '✗ 自证 probe：探针那一句' "$work/clip-report" || clip_wrong+="说明那一句没原样打出来；"
  grep -qF '调用记录：x中' "$work/clip-report" || clip_wrong+="调用记录没转打；"
  [[ -z "$clip_wrong" ]] || cell_failed failure-text-clipped-in-character "失败行转打的片段截在汉字中间时应当丢掉半个字：$clip_wrong" "看 cell_failed 与 selftest_clip_text、selftest_valid_text"
  fi
  if selftest_cell_wanted selftest-cells-selected; then
  # selftest-cells-selected：CRASH_POINT_REPLAY_SELFTEST_CELLS 只跑点名的格、不跑开关轮、设了开关只红在点名的格里、格名写错退 2（文件头「选格」）。
  #   起的是这份脚本自己的 --selftest，只交这一格管的两条开关（别的开关只弄坏外面这一趟，不进里面那一趟）；单跑的格挑只起一趟 entry 的。
  #   弄坏开关 selftest-cells-run-extra：里面那一趟多跑一块，趟数对不上；selftest-cells-unknown-accepted：写错的格名不退 2
  local selection_wrong="" selection_break="" selection_run
  replay_break_is selftest-cells-run-extra && selection_break=selftest-cells-run-extra
  replay_break_is selftest-cells-unknown-accepted && selection_break=selftest-cells-unknown-accepted
  selection_run="$work/selection-run"
  : >"$calls"; cells=$((cells + 1))
  CRASH_POINT_REPLAY_BREAK="$selection_break" CRASH_POINT_REPLAY_SELFTEST_CELLS=missing-item-value bash "$replay_script" --selftest >"$selection_run" 2>&1
  exit_code=$?
  { [[ "$exit_code" == 0 ]] && grep -qF '✓ --selftest 只跑点名的 1 格（missing-item-value）：起了 1 趟都对' "$selection_run" && ! grep -qF '开关轮 ' "$selection_run"; } \
    || selection_wrong+="点名 missing-item-value 应当退 0、只起 1 趟、不跑开关轮（退 $exit_code，$(tail -n 1 "$selection_run" | selftest_clip_text 200)）；"
  CRASH_POINT_REPLAY_BREAK=accept-unknown-item CRASH_POINT_REPLAY_SELFTEST_CELLS=unknown-item bash "$replay_script" --selftest >"$selection_run" 2>&1
  exit_code=$?
  { [[ "$exit_code" == 1 ]] && [[ "$(grep -c '✗ 自证 ' "$selection_run")" == 1 ]] && grep -qF '✗ 自证 unknown-item：' "$selection_run"; } \
    || selection_wrong+="设开关 accept-unknown-item、点名 unknown-item 应当退 1、只红在这一格（退 $exit_code）；"
  CRASH_POINT_REPLAY_BREAK=accept-unknown-item CRASH_POINT_REPLAY_SELFTEST_CELLS=missing-item-value bash "$replay_script" --selftest >"$selection_run" 2>&1
  exit_code=$?
  [[ "$exit_code" == 0 ]] || selection_wrong+="设开关 accept-unknown-item、点名没被它弄坏的 missing-item-value 应当退 0（退 $exit_code）；"
  CRASH_POINT_REPLAY_BREAK="$selection_break" CRASH_POINT_REPLAY_SELFTEST_CELLS=no-such-cell bash "$replay_script" --selftest >"$selection_run" 2>&1
  exit_code=$?
  { [[ "$exit_code" == 2 ]] && grep -qF 'no-such-cell' "$selection_run"; } || selection_wrong+="格名写错应当退 2、点名写错的那个（退 $exit_code）；"
  [[ -z "$selection_wrong" ]] || cell_failed selftest-cells-selected "选格应当只跑点名的格、不跑开关轮、写错退 2：$selection_wrong" "看文件头「选格」、selftest_cell_wanted 与收尾认点名那一段"
  fi
  if selftest_cell_wanted selftest-worker-cells; then
  # selftest-worker-cells：整份自证把格分到几路（文件头「自证并行」）。样本脚本里 b 与 c 同在一块、c 另有一块（同一组，必须同一路）、a 起 1 回入口、d 起 4 回；两路 ⇒
  #   分量大的 d（块 1 + 入口 4）先分到头一路，{b,c}（块 2 + 入口 2）第二路，a（块 1 + 入口 1）补到此刻最轻的第二路 ⇒ 两行「d」「b,c,a」；
  #   真脚本分五路：每个格名恰好在一路里出现一次，点名其中后一格时前一块照跑的两对（item-plan-lines 与 shard-plan-lines、segments-registry-sent 与 test-functions-threads-from-service）
  #   各在同一路，这一格自己也在。弄坏开关 cell-groups-by-name：一个格名一组，两处都红
  local cells_wrong="" cells_sample="$work/worker-cells-sample" cells_sample_lines cells_real_lines cells_duplicated cells_pair cells_line
  printf '%s\n' 'run_selftest() {' '  selftest_cell_wanted() { # 样本' '    :' '  }' \
    '  if selftest_cell_wanted a; then' '  entry -- one' '  fi' \
    '  if selftest_cell_wanted b c; then' '  entry -- two' '  if [[ x ]]; then' '  :' '  fi' '  fi' \
    '  if selftest_cell_wanted c \' '     && [[ x ]]; then' '  entry -- three' '  fi' \
    '  if selftest_cell_wanted d; then' '  entry -- four' '  entry -- five' '  entry -- six' '  entry -- seven' '  fi' '}' >"$cells_sample"
  cells_sample_lines="$(selftest_worker_cells "$cells_sample" 2 | tr '\n' '|')"
  [[ "$cells_sample_lines" == 'd|b,c,a|' ]] || cells_wrong+="样本脚本分两路应当是「d|b,c,a|」（得到「$cells_sample_lines」）；"
  cells_real_lines="$(selftest_worker_cells "$replay_selftest_library" 5)"
  [[ "$(grep -c . <<<"$cells_real_lines")" == 5 ]] || cells_wrong+="真脚本分五路应当打五行（打了 $(grep -c . <<<"$cells_real_lines") 行）；"
  cells_duplicated="$(tr ',' '\n' <<<"$cells_real_lines" | LC_ALL=C sort | uniq -d | tr '\n' ' ')"
  [[ -z "$cells_duplicated" ]] || cells_wrong+="这些格名分到了不止一路：$cells_duplicated；"
  for cells_pair in item-plan-lines,shard-plan-lines segments-registry-sent,test-functions-threads-from-service; do
    cells_line="$(grep -E "(^|,)${cells_pair%,*}(,|\$)" <<<"$cells_real_lines")"
    [[ ",$cells_line," == *",${cells_pair#*,},"* ]] || cells_wrong+="${cells_pair%,*} 与 ${cells_pair#*,} 应当在同一路；"
  done
  grep -qE '(^|,)selftest-worker-cells(,|$)' <<<"$cells_real_lines" || cells_wrong+="真脚本的分路里找不到这一格自己；"
  : >"$calls"; cells=$((cells + 1)); exit_code=0
  [[ -z "$cells_wrong" ]] || cell_failed selftest-worker-cells "整份自证应当把格按组分到几路、点名其中后一格时前一块照跑的格在同一路、每个格名只在一路里：$cells_wrong" \
    "看 selftest_worker_cells 的读法：行首两个空格的 if selftest_cell_wanted 块、一块点名几个格名与几块点名同一个格名的并成一组"
  fi
  if selftest_cell_wanted selftest-workers-run-together; then
  # selftest-workers-run-together：并行起几路、按序号收（文件头「自证并行」）。三路假的子进程：left 与 right 各建自己的标记再等对方的标记（同时在跑才等得到；一路一路地起时
  #   left 等不到 right，退 1），bad 往标准错误打一行失败行与汇总句、退 1；并行度 3 ⇒ 收回 3 路，退出码按序号 0、0、1，趟数 2+2+3，判错 1 格，回读时 bad 的失败行原样转到标准错误、
  #   每路自己的成功句不转。弄坏开关 parallel-workers-serial（一路一路地起）与 parallel-failure-dropped（回读不看退出码与失败行）
  local workers_wrong="" workers_directory="$work/workers-run-together"
  rm -rf -- "${workers_directory:?}"; mkdir -p -- "$workers_directory"
  printf '%s\n' '#!/usr/bin/env bash' '# 假的一路（自证用）：CRASH_POINT_REPLAY_SELFTEST_CELLS 是它的名字' \
    'name="$CRASH_POINT_REPLAY_SELFTEST_CELLS"; here="$(dirname "$0")"' ': >"$here/$name.here"' 'case "$name" in' '  left|right)' \
    '    other=right; [[ "$name" == right ]] && other=left' \
    '    for ((tick = 0; tick < 50; tick++)); do [[ -e "$here/$other.here" ]] && break; sleep 0.1; done' \
    '    [[ -e "$here/$other.here" ]] || { echo "  ✗ 自证 $name：等不到另一路在跑" >&2; exit 1; }' \
    '    echo "  ✓ --selftest 只跑点名的 1 格（$name）：起了 2 趟都对；没点名的格与开关轮这一趟都没跑，不算门禁通过" ;;' \
    '  bad)' '    echo "  ✗ 自证 bad：造的一格判错" >&2' '    echo "  ✗ --selftest：3 格里 1 格判错" >&2; exit 1 ;;' 'esac' >"$workers_directory/worker"
  selftest_worker_assignments=(left right bad)
  selftest_run_workers "$workers_directory" 3 -- bash "$workers_directory/worker"
  selftest_collect_workers "$workers_directory" >"$workers_directory/replayed.out" 2>"$workers_directory/replayed.err"
  [[ "$selftest_workers_collected" == 3 && "${selftest_worker_exits[*]}" == "0 0 1" ]] \
    || workers_wrong+="三路应当都收回、退出码按序号 0 0 1（收回 $selftest_workers_collected 路，退出码「${selftest_worker_exits[*]}」）；"
  [[ "$selftest_workers_cells" == 7 && "$selftest_workers_failures" == 1 ]] \
    || workers_wrong+="趟数应当加起来是 7、判错 1 格（得到 $selftest_workers_cells 趟、$selftest_workers_failures 格）；"
  grep -qF '✗ 自证 bad：造的一格判错' "$workers_directory/replayed.err" || workers_wrong+="bad 那一路的失败行应当原样转到标准错误；"
  ! grep -qF '✓ --selftest 只跑点名的' "$workers_directory/replayed.out" || workers_wrong+="每路自己的成功句不该转出来；"
  : >"$calls"; cells=$((cells + 1)); exit_code=0
  [[ -z "$workers_wrong" ]] || cell_failed selftest-workers-run-together "几路应当同时在跑、按序号收回退出码与输出、失败行转到标准错误、趟数加起来：$workers_wrong" \
    "看 selftest_run_workers（至多 路数 路同时在跑、第 i 路等第 i-路数 路收完再起）与 selftest_collect_workers"
  selftest_worker_assignments=()
  fi
  if selftest_cell_wanted switch-table-names-cells; then
  # switch-table-names-cells：文件头开关表点名的每一格都写成「✗ 自证 <格名>：…」（门禁指定范围时从这一段读格名）；缺的那种写法读得出来（样本的 gamma-cell 只写了句首）。
  #   弄坏开关 switch-table-check-blind：缺的认不出来
  local table_wrong="" table_sample="$work/switch-table-sample" table_rows
  printf '%s\n' '#!/usr/bin/env bash' '#   CRASH_POINT_REPLAY_BREAK=sample-switch  样本' '#       alpha-cell（「✗ 自证 alpha-cell：…句首」）、gamma-cell（「只写了句首」）' 'set -u' >"$table_sample"
  table_rows="$(replay_switch_rows_without_cells "$table_sample")"
  [[ "$table_rows" == "sample-switch gamma-cell" ]] || table_wrong+="样本里 gamma-cell 只写了句首、没写成失败行的起头，应当读出来（读出「$table_rows」）；"
  table_rows="$(replay_switch_rows_without_cells "$replay_script")"
  [[ -z "$table_rows" ]] || table_wrong+="开关表里读不出格名的：$(tr '\n' '|' <<<"$table_rows" | selftest_clip_text 300)；"
  [[ -z "$table_wrong" ]] || cell_failed switch-table-names-cells "开关表点名的每一格都要写成「✗ 自证 <格名>：…」：$table_wrong" "照文件头「弄坏开关」那一段的写法改那一行"
  fi
  if selftest_cell_wanted switch-wheel-progress-lines; then
  # switch-wheel-progress-lines：开关轮每收回一条开关当场打一行进度（开关名、收回第几条共几条、表里第几条、用了几秒、红在点名的格没有），行数与开关表的条数相同、每行整行不交错，
  #   对不上的那一条写「没红在点名的格」并算一处对不上（文件头「自证末尾的开关轮」）。三条假开关、一份照开关打「✗ 自证 <格>：」退 1 的假自证（gamma 红的不是点名的格），并行 2 路。
  #   弄坏开关 switch-wheel-progress-silent：不打进度行
  local wheel_wrong="" wheel_directory="$work/switch-wheel-progress" wheel_output wheel_line wheel_switch wheel_wanted wheel_position=0
  local -a wheel_saved_assignments=(${switch_wheel_assignments[@]+"${switch_wheel_assignments[@]}"}) wheel_saved_expected=(${switch_wheel_expected[@]+"${switch_wheel_expected[@]}"})
  mkdir -p -- "$wheel_directory"
  printf '%s\n' '#!/usr/bin/env bash' '# 开关轮进度那一格的假自证：照 SAMPLE_WHEEL_SWITCH 打一行失败行、退 1' 'case "${SAMPLE_WHEEL_SWITCH:-}" in' \
    '  alpha|gamma) echo "  ✗ 自证 alpha-cell：样本"; exit 1 ;;' '  beta) echo "  ✗ 自证 beta-cell：样本"; exit 1 ;;' 'esac' 'exit 0' >"$wheel_directory/sample-selftest.sh"
  switch_wheel_assignments=(SAMPLE_WHEEL_SWITCH=alpha SAMPLE_WHEEL_SWITCH=beta SAMPLE_WHEEL_SWITCH=gamma)
  switch_wheel_expected=(alpha-cell beta-cell gamma-cell)
  wheel_output="$wheel_directory/wheel.out"
  cells=$((cells + 1))
  CRASH_AMPLIFICATION_SELFTEST_JOBS=2 selftest_switch_wheel_with_progress "$wheel_directory" '✗ 自证 ([a-z-]+)：' -- bash "$wheel_directory/sample-selftest.sh" >"$wheel_output" 2>&1
  (( $(grep -cE '^  · 开关轮进度：收回 [1-3]/3（表里第 [1-3] 条）SAMPLE_WHEEL_SWITCH=[a-z]+，用了 [0-9]+ 秒，' "$wheel_output") == 3 )) \
    || wheel_wrong+="整行的进度行应当恰好 3 行（开关表 3 条）；"
  for wheel_switch in alpha beta gamma; do
    wheel_position=$((wheel_position + 1))
    case "$wheel_switch" in
      gamma) wheel_wanted="没红在点名的格：退 1、红在「alpha-cell」，文件头点名的是「gamma-cell」" ;;
      *) wheel_wanted="红在点名的格「${wheel_switch}-cell」" ;;
    esac
    wheel_line="$(grep -F "（表里第 ${wheel_position} 条）SAMPLE_WHEEL_SWITCH=${wheel_switch}，用了 " "$wheel_output")"
    [[ "$wheel_line" == *"$wheel_wanted"* && "$wheel_line" != *$'\n'* ]] || wheel_wrong+="${wheel_switch} 那一行应当恰好一行、写「${wheel_wanted}」（读到「$(tr '\n' '|' <<<"$wheel_line" | selftest_clip_text 200)」）；"
  done
  grep -qF '收回 3/3（' "$wheel_output" || wheel_wrong+="最后收回的那一行应当写「收回 3/3」；"
  (( switch_wheel_collected == 3 && switch_wheel_failures == 1 )) || wheel_wrong+="应当收回 3 条、对不上 1 处（收回 $switch_wheel_collected、对不上 $switch_wheel_failures）；"
  switch_wheel_assignments=(${wheel_saved_assignments[@]+"${wheel_saved_assignments[@]}"}); switch_wheel_expected=(${wheel_saved_expected[@]+"${wheel_saved_expected[@]}"})
  [[ -z "$wheel_wrong" ]] || cell_failed switch-wheel-progress-lines "开关轮每收回一条开关要当场打一行进度：$wheel_wrong" "看 selftest_switch_wheel_with_progress 收回那一段（进度行只由这一层打、一行一次写）"
  fi
  if selftest_cell_wanted switch-wheel-named-cells; then
  # switch-wheel-named-cells：开关轮的范围（文件头「自证并行」那一段）。假自证把收到的 CRASH_POINT_REPLAY_SELFTEST_CELLS 记进文件：
  #   不在整轮门禁里，每条开关收到的是开关表给它点名的格（两格的用逗号隔）；环境里 GATE_IN_STAGE=1 时一条都不收（跑整份）。
  #   弄坏开关 switch-wheel-always-whole：不在整轮门禁里也不收
  local named_wrong="" named_directory="$work/switch-wheel-named-cells" named_received
  local -a named_saved_assignments=(${switch_wheel_assignments[@]+"${switch_wheel_assignments[@]}"}) named_saved_expected=(${switch_wheel_expected[@]+"${switch_wheel_expected[@]}"})
  mkdir -p -- "$named_directory"
  printf '%s\n' '#!/usr/bin/env bash' '# 开关轮范围那一格的假自证：记下收到的选格，照开关打失败行、退 1' \
    'printf "%s\n" "${CRASH_POINT_REPLAY_SELFTEST_CELLS-没收到}" >"$(dirname "$0")/received-$SAMPLE_WHEEL_SWITCH"' 'case "${SAMPLE_WHEEL_SWITCH:-}" in' \
    '  alpha) echo "  ✗ 自证 alpha-cell：样本"; exit 1 ;;' '  beta) echo "  ✗ 自证 beta-cell：样本"; echo "  ✗ 自证 other-cell：样本"; exit 1 ;;' 'esac' 'exit 0' >"$named_directory/sample-selftest.sh"
  switch_wheel_assignments=(SAMPLE_WHEEL_SWITCH=alpha SAMPLE_WHEEL_SWITCH=beta)
  switch_wheel_expected=(alpha-cell 'beta-cell  other-cell')
  cells=$((cells + 1))
  GATE_IN_STAGE="" CRASH_AMPLIFICATION_SELFTEST_JOBS=2 selftest_switch_wheel_with_progress "$named_directory" '✗ 自证 ([a-z-]+)：' -- bash "$named_directory/sample-selftest.sh" >"$named_directory/named.out" 2>&1
  named_received="$(cat "$named_directory/received-alpha" "$named_directory/received-beta" 2>/dev/null | tr '\n' '|')"
  [[ "$named_received" == 'alpha-cell|beta-cell,other-cell|' && "$switch_wheel_failures" == 0 ]] \
    || named_wrong+="不在整轮门禁里，两条开关应当各收到点名的格「alpha-cell」「beta-cell,other-cell」、都对得上（收到「$named_received」、对不上 $switch_wheel_failures 处）；"
  cells=$((cells + 1))
  GATE_IN_STAGE=1 CRASH_AMPLIFICATION_SELFTEST_JOBS=2 selftest_switch_wheel_with_progress "$named_directory" '✗ 自证 ([a-z-]+)：' -- bash "$named_directory/sample-selftest.sh" >"$named_directory/whole.out" 2>&1
  named_received="$(cat "$named_directory/received-alpha" "$named_directory/received-beta" 2>/dev/null | tr '\n' '|')"
  [[ "$named_received" == '没收到|没收到|' && "$switch_wheel_failures" == 0 ]] \
    || named_wrong+="整轮门禁里（GATE_IN_STAGE=1）两条开关应当都不收选格、跑整份（收到「$named_received」、对不上 $switch_wheel_failures 处）；"
  switch_wheel_assignments=(${named_saved_assignments[@]+"${named_saved_assignments[@]}"}); switch_wheel_expected=(${named_saved_expected[@]+"${named_saved_expected[@]}"})
  [[ -z "$named_wrong" ]] || cell_failed switch-wheel-named-cells "开关轮不在整轮门禁里只跑点名的格、整轮门禁里跑整份：$named_wrong" "看 selftest_switch_wheel_with_progress 里定 switch_wheel_scope 与设 CRASH_POINT_REPLAY_SELFTEST_CELLS 的那两段"
  fi
  if selftest_cell_wanted switch-wheel-segments-partition; then
  # switch-wheel-segments-partition：开关轮分段（文件头「开关轮分段」）。真开关表上：各段的并集是全部开关、互不相交（每条开关恰好落在一段里）、每段不超过 WHEEL_SEGMENT_SWITCHES_MOST 条；
  #   样本：10 条分 3 段是「0 3 6 9」「1 4 7」「2 5 8」，2 条分 3 段时第三段空，段号 0 一条不取；选段把两个同序数组筛成那一段（10 条取第 2/3 段是第 2、5、8 条，点名的格里的空格原样留着）；
  #   写法：1/段数、5/段数、段数/段数、base 认，0/段数、段数加一/段数、1/段数加一、1、x/段数、01/段数、BASE、1/0 起头的段数、空串不认；写错、与选格同给、与弄坏开关同给时整份自证退 2、列出可点的写法。
  #   弄坏开关 switch-wheel-segment-drops-one：表里最后一条开关不落进任何一段
  local segment_wrong="" segment_total segment_number segment_position segment_size segment_text segment_value segment_expected segment_run
  local -a segment_cover=() segment_saved_assignments=(${switch_wheel_assignments[@]+"${switch_wheel_assignments[@]}"}) segment_saved_expected=(${switch_wheel_expected[@]+"${switch_wheel_expected[@]}"})
  local -a segment_saved_positions=(${switch_wheel_positions[@]+"${switch_wheel_positions[@]}"})
  local segment_saved_number="$wheel_segment_number"
  segment_total="$(crash_amplification_read_switch_table "$replay_script" CRASH_POINT_REPLAY_BREAK | grep -c .)"
  (( segment_total > 0 )) || segment_wrong+="开关表里读不出开关；"
  for ((segment_position = 0; segment_position < segment_total; segment_position++)); do segment_cover[segment_position]=0; done
  for ((segment_number = 1; segment_number <= WHEEL_SEGMENT_COUNT; segment_number++)); do
    segment_size=0
    while read -r segment_position; do
      [[ -n "$segment_position" ]] || continue
      segment_cover[segment_position]=$((${segment_cover[segment_position]:-0} + 1)); segment_size=$((segment_size + 1))
    done < <(selftest_wheel_segment_indexes "$segment_number" "$WHEEL_SEGMENT_COUNT" "$segment_total")
    (( segment_size <= WHEEL_SEGMENT_SWITCHES_MOST )) \
      || segment_wrong+="第 $segment_number 段有 $segment_size 条，超过每段至多 $WHEEL_SEGMENT_SWITCHES_MOST 条（改 WHEEL_SEGMENT_COUNT）；"
  done
  segment_text=""
  for ((segment_position = 0; segment_position < segment_total; segment_position++)); do
    (( ${segment_cover[segment_position]} == 1 )) || segment_text+="表里第 $((segment_position + 1)) 条（被 ${segment_cover[segment_position]} 段取走）、"
  done
  [[ -z "$segment_text" ]] || segment_wrong+="每条开关应当恰好落在一段里，这些不是：${segment_text%、}；"
  segment_text="$(selftest_wheel_segment_indexes 1 3 10 | tr '\n' ' ')|$(selftest_wheel_segment_indexes 2 3 10 | tr '\n' ' ')|$(selftest_wheel_segment_indexes 3 3 10 | tr '\n' ' ')|$(selftest_wheel_segment_indexes 3 3 2 | tr '\n' ' ')|$(selftest_wheel_segment_indexes 0 3 10 | tr '\n' ' ')"
  [[ "$segment_text" == '0 3 6 9 |1 4 7 |2 5 8 ||' ]] || segment_wrong+="样本 10 条分 3 段、2 条分 3 段取第 3 段、段号 0，应当是「0 3 6 9 |1 4 7 |2 5 8 ||」（得到「$segment_text」）；"
  switch_wheel_assignments=(V=a V=b V=c V=d V=e V=f V=g V=h V=i V=j); switch_wheel_expected=(x1 "x2 y2" x3 x4 x5 x6 x7 x8 x9 x10)
  selftest_wheel_select_segment 2 3
  { [[ "${switch_wheel_assignments[*]}" == 'V=b V=e V=h' && "${#switch_wheel_expected[@]}" == 3 && "${switch_wheel_expected[0]}" == 'x2 y2' && "${switch_wheel_expected[2]}" == x8 \
      && "${switch_wheel_positions[*]}" == '2 5 8' ]]; } \
    || segment_wrong+="样本 10 条取第 2/3 段应当是 V=b V=e V=h、点名的格 x2 y2 与 x8 原样、表里第 2 5 8 条（得到「${switch_wheel_assignments[*]}」「${switch_wheel_positions[*]}」）；"
  switch_wheel_assignments=(V=a V=b V=c); switch_wheel_expected=(x1 x2 x3)
  selftest_wheel_select_segment 0 3
  (( ${#switch_wheel_assignments[@]} == 0 && ${#switch_wheel_expected[@]} == 0 && ${#switch_wheel_positions[@]} == 0 )) || segment_wrong+="段号 0（base）不取任何一条开关；"
  for segment_value in "1/$WHEEL_SEGMENT_COUNT" "5/$WHEEL_SEGMENT_COUNT" "$WHEEL_SEGMENT_COUNT/$WHEEL_SEGMENT_COUNT" base; do
    segment_expected="${segment_value%%/*}"; [[ "$segment_value" == base ]] && segment_expected=0
    { selftest_wheel_segment_choose "$segment_value" 2>/dev/null && [[ "$wheel_segment_number" == "$segment_expected" ]]; } || segment_wrong+="写法「$segment_value」应当认、段号 $segment_expected（得到「$wheel_segment_number」）；"
  done
  for segment_value in "0/$WHEEL_SEGMENT_COUNT" "$((WHEEL_SEGMENT_COUNT + 1))/$WHEEL_SEGMENT_COUNT" "1/$((WHEEL_SEGMENT_COUNT + 1))" 1 "x/$WHEEL_SEGMENT_COUNT" "01/$WHEEL_SEGMENT_COUNT" BASE "1/0$WHEEL_SEGMENT_COUNT" ""; do
    ! selftest_wheel_segment_choose "$segment_value" 2>/dev/null || segment_wrong+="写法「$segment_value」应当不认；"
  done
  wheel_segment_number="$segment_saved_number"
  switch_wheel_assignments=(${segment_saved_assignments[@]+"${segment_saved_assignments[@]}"}); switch_wheel_expected=(${segment_saved_expected[@]+"${segment_saved_expected[@]}"})
  switch_wheel_positions=(${segment_saved_positions[@]+"${segment_saved_positions[@]}"})
  segment_run="$work/segment-run"
  cells=$((cells + 1))
  CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT="$((WHEEL_SEGMENT_COUNT + 1))/$WHEEL_SEGMENT_COUNT" timeout 120 bash "$replay_script" --selftest >"$segment_run" 2>&1
  exit_code=$?
  { [[ "$exit_code" == 2 ]] && grep -qF "1/$WHEEL_SEGMENT_COUNT" "$segment_run" && grep -qF "$WHEEL_SEGMENT_COUNT/$WHEEL_SEGMENT_COUNT" "$segment_run" && grep -qF base "$segment_run"; } \
    || segment_wrong+="段号写错的整份自证应当退 2、列出 1/$WHEEL_SEGMENT_COUNT 到 $WHEEL_SEGMENT_COUNT/$WHEEL_SEGMENT_COUNT 与 base（退 $exit_code，$(tail -n 1 "$segment_run" | selftest_clip_text 200)）；"
  CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT="1/$WHEEL_SEGMENT_COUNT" CRASH_POINT_REPLAY_SELFTEST_CELLS=missing-item-value timeout 120 bash "$replay_script" --selftest >"$segment_run" 2>&1
  exit_code=$?
  { [[ "$exit_code" == 2 ]] && grep -qF CRASH_POINT_REPLAY_SELFTEST_CELLS "$segment_run"; } || segment_wrong+="段与选格同给的整份自证应当退 2、点名选格变量（退 $exit_code）；"
  CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT="1/$WHEEL_SEGMENT_COUNT" CRASH_POINT_REPLAY_BREAK=accept-unknown-item timeout 120 bash "$replay_script" --selftest >"$segment_run" 2>&1
  exit_code=$?
  { [[ "$exit_code" == 2 ]] && grep -qF CRASH_POINT_REPLAY_BREAK "$segment_run"; } || segment_wrong+="段与弄坏开关同给的整份自证应当退 2、点名弄坏开关变量（退 $exit_code）；"
  exit_code=0
  [[ -z "$segment_wrong" ]] || cell_failed switch-wheel-segments-partition "开关轮分段应当把开关表分成互不相交、并集是全部的几段，写错的段退 2：$segment_wrong" \
    "看 selftest_wheel_segment_indexes（分法）、selftest_wheel_select_segment（筛段）、selftest_wheel_segment_choose（写法）与 WHEEL_SEGMENT_COUNT、WHEEL_SEGMENT_SWITCHES_MOST 两个常量"
  fi

  # 选格的收尾（文件头「选格」）：点名的格这一趟都问到过；有没问到的，写错了，退 2（弄坏开关 selftest-cells-unknown-accepted 下不判）
  if [[ -n "$replay_selftest_cells" ]]; then
    for selection_word in ${replay_selftest_cells//,/ }; do
      [[ -n "${selection_seen[$selection_word]:-}" ]] || selection_unknown+="$selection_word "
    done
    if (( ${#selection_named[@]} == 0 )); then selection_unknown+="（逗号之间一个格名都没有）"; fi
    if [[ -n "$selection_unknown" ]] && ! replay_break_is selftest-cells-unknown-accepted; then
      echo "  ✗ --selftest：CRASH_POINT_REPLAY_SELFTEST_CELLS 里点名的 $selection_unknown不是这份自证的格" >&2
      echo "     → 怎么办：格名照失败行「✗ 自证 <格名>：」与文件头开关表写，逗号隔开：$(printf '%s\n' "${!selection_seen[@]}" | LC_ALL=C sort | tr '\n' ' ')" >&2
      exit 2
    fi
  fi

  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  if [[ -n "$replay_selftest_cells" ]]; then
    echo "  ✓ --selftest 只跑点名的 ${#selection_named[@]} 格（$replay_selftest_cells）：起了 $cells 趟都对；没点名的格与开关轮这一趟都没跑，不算门禁通过"
    exit 0
  fi
  if crash_amplification_switch_wheel_inner; then echo "  ✓ --selftest（开关轮里起的一份，不再起开关轮）：$cells 格都对"; exit 0; fi

  selftest_wheel_and_summary "$work" "$cells" ""
}

# 开关轮分段（文件头「开关轮分段」）：段数与每段至多几条开关。每条开关单跑一份整份自证取至多 864 秒，一次运行不超过 7200 秒，7200 ÷ 864 向下取整得 8；
#   段数 = ⌈开关条数 ÷ 8⌉：开关表 75 条时 ⌈75 ÷ 8⌉ = 10，每段 7 或 8 条。开关条数涨到每段超过 8 条时 switch-wheel-segments-partition 那一格红，改 WHEEL_SEGMENT_COUNT。
WHEEL_SEGMENT_COUNT=10
WHEEL_SEGMENT_SWITCHES_MOST=8
wheel_segment_number=""    # 点名的段号（1 起；0 是 base；空是整轮不分段），selftest_wheel_segment_choose 写
switch_wheel_positions=()  # selftest_wheel_select_segment 筛出来的每条开关在整张表里的序号（1 起），与 switch_wheel_assignments 同序；空就是整轮，进度行取它的下标加一

# selftest_wheel_segment_indexes <段号> <段数> <开关条数>：开关轮第 <段号> 段（1 起）含的开关在表里的序号（0 起，一行一个）：序号除以 <段数> 的余数等于 <段号> 减一的那些；
#   段号 0 一条都不取。弄坏开关 switch-wheel-segment-drops-one 下最后一条（序号 <开关条数> 减一）不取
selftest_wheel_segment_indexes() {
  local segment="$1" segments="$2" count="$3" index
  for ((index = 0; index < count; index++)); do
    if replay_break_is switch-wheel-segment-drops-one && (( index == count - 1 )); then continue; fi
    if (( index % segments == segment - 1 )); then echo "$index"; fi
  done
}

# selftest_wheel_select_segment <段号> <段数>：把 switch_wheel_assignments、switch_wheel_expected 筛成第 <段号> 段的那几条（取哪几条照 selftest_wheel_segment_indexes），
#   switch_wheel_positions 放它们在整张表里的序号（1 起）
selftest_wheel_select_segment() {
  local segment="$1" segments="$2" position
  local -a kept_assignments=() kept_expected=() kept_positions=()
  while read -r position; do
    [[ -n "$position" ]] || continue
    kept_assignments+=("${switch_wheel_assignments[position]}"); kept_expected+=("${switch_wheel_expected[position]}"); kept_positions+=("$((position + 1))")
  done < <(selftest_wheel_segment_indexes "$segment" "$segments" "${#switch_wheel_assignments[@]}")
  switch_wheel_assignments=(${kept_assignments[@]+"${kept_assignments[@]}"})
  switch_wheel_expected=(${kept_expected[@]+"${kept_expected[@]}"})
  switch_wheel_positions=(${kept_positions[@]+"${kept_positions[@]}"})
}

# selftest_wheel_segment_choose <CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT 的值>：认 <段号>/<段数>（段数是 WHEEL_SEGMENT_COUNT、段号 1 到段数、不写前导 0）与 base；
#   认得交回 0、段号写进 wheel_segment_number（base 写 0）；认不得交回 1，标准错误打写错的值与可点的全部写法
selftest_wheel_segment_choose() {
  local value="$1" listed="" number
  wheel_segment_number=""
  if [[ "$value" == base ]]; then wheel_segment_number=0; return 0; fi
  if [[ "$value" =~ ^([1-9][0-9]{0,2})/([1-9][0-9]{0,2})$ ]] && (( BASH_REMATCH[2] == WHEEL_SEGMENT_COUNT && BASH_REMATCH[1] <= WHEEL_SEGMENT_COUNT )); then
    wheel_segment_number="${BASH_REMATCH[1]}"; return 0
  fi
  for ((number = 1; number <= WHEEL_SEGMENT_COUNT; number++)); do listed+="$number/$WHEEL_SEGMENT_COUNT "; done
  echo "  ✗ --selftest：CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT=「$value」认不得" >&2
  echo "     → 怎么办：写 <段号>/$WHEEL_SEGMENT_COUNT（可点：${listed}）跑开关轮那一段的开关，或写 base 只跑基准趟各格；不点名就整份跑（文件头「开关轮分段」）" >&2
  return 1
}

# selftest_switch_wheel_with_progress <放输出的目录> <取格的正则> -- <自证命令…>：开关轮（文件头「弄坏开关」与「自证末尾的开关轮」）。两个同序数组的写法同共用库
#   crash-amplification-lib.sh 文件头「弄坏开关轮」：switch_wheel_assignments 每个开关一项「<变量>=<开关名>」，switch_wheel_expected 是它点名的格（「-」是应当退 0、一格不红）。
#   每个开关一个子 shell：清掉数组里出现过的每个变量、只 export 这一个与开关轮标记（CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER，调用方用 crash_amplification_switch_wheel_inner 认），
#   跑自证命令，输出进 <目录>/switch-wheel-<序号>.out、退出码进 <目录>/switch-wheel-<序号>.exit；至多并行度那么多个同时在跑，有一个收尾（wait -n）就扫一遍退出码文件，
#   新收回的按序号逐个判（共用库的 crash_amplification_switch_wheel_verdict）、当场打一行进度（只由这一层打，一行一次写），再补起下一个。
#   并行度：CRASH_AMPLIFICATION_SELFTEST_JOBS，没给取每台默认重活线程数；取不出拒（一个都不起，全算对不上）。
#   交回之后 switch_wheel_failures 是对不上的处数（含派出收回对不上）、switch_wheel_collected 是收回的个数。弄坏开关 switch-wheel-progress-silent 下收回时不打进度行（✗ 照打）
selftest_switch_wheel_with_progress() {
  local directory="$1" pattern="$2" jobs count index next=0 running=0 wait_status progressed assignment variable observed shown expected_shown verdict_text exit_code
  local variables=" " unset_names=() named_cells
  local -a started_at=() reported=()
  shift 2
  # 开关轮的范围：整轮门禁里（GATE_IN_STAGE=1）每条开关跑整份，别处只跑点名的格；弄坏开关 switch-wheel-always-whole 下一律整份
  switch_wheel_scope=named
  if [[ "${GATE_IN_STAGE:-}" == 1 ]] || replay_break_is switch-wheel-always-whole; then switch_wheel_scope=whole; fi
  [[ "${1:-}" == -- ]] && shift
  switch_wheel_failures=0; switch_wheel_collected=0
  count="${#switch_wheel_assignments[@]}"
  jobs="${CRASH_AMPLIFICATION_SELFTEST_JOBS:-$(heavy_thread_default)}"
  if [[ ! "$jobs" =~ ^[1-9][0-9]*$ ]]; then
    echo "  ✗ 开关轮的并行度取不出（CRASH_AMPLIFICATION_SELFTEST_JOBS 或每台默认重活线程数给的是「$jobs」），一个开关都没起" >&2
    echo "     → 怎么办：CRASH_AMPLIFICATION_SELFTEST_JOBS 写成正整数或不设；不设时照 .claude/heavy-thread-default 文件头写好登记再跑" >&2
    switch_wheel_failures=$((count > 0 ? count : 1))
    return 0
  fi
  for assignment in ${switch_wheel_assignments[@]+"${switch_wheel_assignments[@]}"}; do
    variable="${assignment%%=*}"
    [[ "$variables" == *" $variable "* ]] && continue
    variables+="$variable "; unset_names+=("$variable")
  done
  printf '%s\n' "$CRASH_AMPLIFICATION_SWITCH_WHEEL_MARK_CONTENT" >"$directory/switch-wheel-mark"
  for ((index = 0; index < count; index++)); do rm -f -- "$directory/switch-wheel-$index.exit"; done
  while (( switch_wheel_collected < count )); do
    while (( next < count && running < jobs )); do
      # 子 shell 里清变量、设这一个开关再跑自证命令，退出码写进自己的文件（.claude/singlefs-ai-sop/rules/command-safety.md「并行不许把失败吃掉」）
      (
        unset ${unset_names[@]+"${unset_names[@]}"}
        export "${switch_wheel_assignments[next]}" CRASH_AMPLIFICATION_SWITCH_WHEEL_INNER="$directory/switch-wheel-mark"
        # 不在整轮门禁里：这条开关只跑开关表给它点名的格（文件头「自证并行」那一段）
        if [[ "$switch_wheel_scope" == named && -n "${switch_wheel_expected[next]:-}" && "${switch_wheel_expected[next]}" != - ]]; then
          named_cells="$(tr -s ' \t' ',,' <<<"${switch_wheel_expected[next]}")"; named_cells="${named_cells#,}"
          export CRASH_POINT_REPLAY_SELFTEST_CELLS="${named_cells%,}"
        fi
        "$@"
        echo "$?" >"$directory/switch-wheel-$next.exit"
      ) >"$directory/switch-wheel-$next.out" 2>&1 &
      started_at[next]="$SECONDS"
      next=$((next + 1)); running=$((running + 1))
    done
    if wait -n; then wait_status=0; else wait_status=$?; fi
    progressed=0
    for ((index = 0; index < next; index++)); do
      [[ -z "${reported[index]:-}" && -f "$directory/switch-wheel-$index.exit" ]] || continue
      reported[index]=1; progressed=1
      running=$((running - 1)); switch_wheel_collected=$((switch_wheel_collected + 1))
      exit_code="$(<"$directory/switch-wheel-$index.exit")"
      expected_shown="${switch_wheel_expected[index]:-}"
      [[ "$expected_shown" == - ]] && expected_shown="（表里没列它：应当退 0、一格不红）"
      if observed="$(crash_amplification_switch_wheel_verdict "${switch_wheel_expected[index]:-}" "$exit_code" "$directory/switch-wheel-$index.out" "$pattern" 0)"; then
        shown="${observed// /、}"
        verdict_text="红在点名的格「${shown:-（一格不红，表里没列它）}」"
      else
        verdict_text="没红在点名的格：退 ${exit_code}、红在「${observed:-（没有）}」，文件头点名的是「${expected_shown:-（没写）}」"
        echo "  ✗ 开关轮：${switch_wheel_assignments[index]} 打开后 --selftest 退 ${exit_code}、红在「${observed:-（没有）}」，文件头点名的是「${expected_shown:-（没写）}」" >&2 # gate-lint:detail
        switch_wheel_failures=$((switch_wheel_failures + 1))
      fi
      replay_break_is switch-wheel-progress-silent \
        || echo "  · 开关轮进度：收回 ${switch_wheel_collected}/${count}（表里第 ${switch_wheel_positions[index]:-$((index + 1))} 条）${switch_wheel_assignments[index]}，用了 $((SECONDS - started_at[index])) 秒，${verdict_text}"
    done
    # 一个子进程都不剩（wait -n 退 127）而这一扫没收到新的：有子 shell 没写下退出码就没了，不再等
    (( progressed || wait_status != 127 )) || break
  done
  if (( switch_wheel_collected != count )); then
    echo "  ✗ 开关轮：派出 $count 个开关、收回 $switch_wheel_collected 个" >&2 # gate-lint:detail
    switch_wheel_failures=$((switch_wheel_failures + 1))
  fi
  return 0
}

# selftest_wheel_and_summary <放输出的目录> <趟数> <自证并行的说明，可空>：开关轮与成功句（格都对之后；自证并行的父进程与不并行的整份串行共用），退出码就是整份自证的
selftest_wheel_and_summary() {
  local work="$1" cells="$2" parallel_note="$3"
  # 开关轮（文件头「弄坏开关」）：从文件头现读开关与点名的格，与代码里 replay_break_is 认的开关比齐，逐个打开再跑一遍 --selftest
  local table assignment named in_code in_table="" round_failures=0
  switch_wheel_assignments=(); switch_wheel_expected=()
  table="$(crash_amplification_read_switch_table "$replay_script" CRASH_POINT_REPLAY_BREAK)"
  while IFS=$'\t' read -r assignment named; do
    [[ -n "$assignment" ]] || continue
    switch_wheel_assignments+=("$assignment"); switch_wheel_expected+=("$named"); in_table+="${assignment#*=}"$'\n'
  done <<<"$table"
  in_code="$(grep -hoE 'replay_break_is [a-z0-9-]+' "$replay_script" "$replay_selftest_library" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table="$(printf '%s' "$in_table" | LC_ALL=C sort -u)"
  if (( ${#switch_wheel_assignments[@]} == 0 )) || [[ "$in_code" != "$in_table" ]]; then
    echo "  ✗ 开关轮：文件头表里的开关与代码里 replay_break_is 认的不一样（只在代码里：$(LC_ALL=C comm -23 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table") | tr '\n' ' ')；只在表里：$(LC_ALL=C comm -13 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table") | tr '\n' ' ')）" >&2
    echo "     → 怎么办：代码里每一处 replay_break_is 认的开关都在表里写一行（连同点名的格），表里每一行都在代码里有对应的一处" >&2
    round_failures=$((round_failures + 1))
  fi
  # 点名了开关轮的一段（文件头「开关轮分段」）：只留这一段的开关；base 一条都不留，开关轮不起
  local total_switches="${#switch_wheel_assignments[@]}" segment_label=""
  switch_wheel_positions=(); switch_wheel_failures=0; switch_wheel_collected=0
  if [[ -n "$wheel_segment_number" ]]; then
    selftest_wheel_select_segment "$wheel_segment_number" "$WHEEL_SEGMENT_COUNT"
    segment_label="开关轮第 $wheel_segment_number/$WHEEL_SEGMENT_COUNT 段"
    [[ "$wheel_segment_number" != 0 ]] || segment_label="基准趟（开关轮 $total_switches 条开关分 $WHEEL_SEGMENT_COUNT 段，一条都没跑）"
  fi
  [[ "$wheel_segment_number" == 0 ]] || selftest_switch_wheel_with_progress "$work" '✗ 自证 ([a-z0-9-]+)：' -- bash "$replay_script" --selftest
  round_failures=$((round_failures + switch_wheel_failures))
  if (( round_failures > 0 )); then
    echo "  ✗ --selftest：${segment_label:+$segment_label：}$cells 格都对，开关轮有 $round_failures 处对不上" >&2
    echo "     → 怎么办：红的格与表里点名的不同，要么改那一处判法让开关红在点名的格，要么照实改文件头那一行；单看一个开关：CRASH_POINT_REPLAY_BREAK=<开关> $0 --selftest" >&2
    exit 1
  fi
  if [[ "$wheel_segment_number" == 0 ]]; then
    echo "  ✓ --selftest 只跑基准趟：$cells 格都对；开关轮 $total_switches 条开关分 $WHEEL_SEGMENT_COUNT 段，这一趟一条都没跑，不算门禁通过"
    [[ -z "$parallel_note" ]] || echo "    $parallel_note"
    exit 0
  elif [[ -n "$wheel_segment_number" ]]; then
    echo "  ✓ --selftest 只跑$segment_label：这一段跑了 ${#switch_wheel_assignments[@]} 条开关、开关表共 $total_switches 条（表里第 $(IFS=,; echo "${switch_wheel_positions[*]-}") 条），派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的格；基准趟各格与别的段这一趟都没跑，不算门禁通过"
    exit 0
  fi
  echo "  ✓ --selftest $cells 格都对：列项（含整趟才跑的几类）不起编译、项名写错与缺项名退 2、删掉的选项一律当认不得的参数退 2、放量的判定在 prune-server 里（接连两趟第二趟复用头一趟判过的块）、"
  echo "    按条复用（真的 admission.py 在做成 git 仓的小仓上算键、查库、记库）：复用三问过、逐条写进的判定剪枝与门禁的计划照键复用、红过的与库里记着红的照现判、"
  echo "    同键又判红记红并撤掉那把键的绿（下一趟照现判）、登记成崩溃枚举用例与放量分项的测试函数经 --skip 排掉、被判的根不是 git 工作树的顶整趟退 2、按段登记文件的内容与片长进闭包读它们的那几条的键、"
  echo "    按条复用失灵判红且不起测试目标、上次红的清单只换真判过的项（本次未跑的照留）、"
  echo "    开了 GPU 测试函数那一段带 gpu 特性、环境里有卡就原样交给 cargo、没有卡照起、不领卡不挑卡不自己清场（不送走 GPU 的计算类）、"
  echo "    每一趟带按段登记文件的绝对路径而它核不过退 2 不起、测试函数红退 1 而另两段照旧起、跑到 0 条与没卡记本次未跑、全是本次未跑退 77、不带参数每项交崩溃点重放 --item 与放量全部项、"
  echo "    --item 只跑点名的项（含 whole-run:<类>）、测试函数的复用数照复用计数行报（没有那一行报取不出、成功句点名）、崩溃点重放红与放量红各红在那一项、放量收不回判红、平常跑带只供自证的变量退 2、开了 GPU 时测试参数不带测试线程，"
  echo "    测试函数日志里有给定状态表借不到显存那一行记本次未跑退 78、放量入口退 78 记本次未跑（红先判）而退 1 照旧红、"
  echo "    跑完运行目录里不剩它建的编译目录（正常、测试函数红、被 TERM 停）而调用方设的 CARGO_TARGET_DIR 不删，"
  echo "    按件分片（计划一件一片、一片只跑自己的件、判红的片退 0、合并判出红、件数超上限合片、片号对不上与产物不全退错、不经服务起退 2），"
  echo "    缩崩溃域（--crash-expand-up-to 与环境里的缩域变量：三段都收到、行里与句里写缩过域、值不是非负整数退 2）、--list-items --with-keys 的 ITEM_PLAN（每项一行、可与 --item 合用）与读库程序同一套特性、"
  echo "    真跑每项一行 ITEM_DONE、按件分片的计划另打每片每项一行 ITEM_PLAN，"
  echo "    （cargo、两套驱动与准入模块都是假的，不编不跑）；开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读）逐个打开，派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的格$([[ "${switch_wheel_scope:-whole}" == named ]] && printf '%s' '（这一趟每条开关只跑开关表给它点名的格；别的格跟没跟着红没核，留给整轮门禁里整份那一趟）')"
  [[ -z "$parallel_note" ]] || echo "    $parallel_note"
  exit 0
}

# run_selftest_parallel：整份自证（没点名格、没设弄坏开关、不是开关轮里起的一份）：各格按组分到至多 selftest_parallel_width 路（selftest_worker_cells），每路是这份脚本自己的
#   --selftest 点名它那几个格（CRASH_POINT_REPLAY_SELFTEST_CELLS：每路各建自己的临时目录与小仓、输出各写各的文件），按序号回读、格数加起来；都对了再跑开关轮（开关轮里起的一份与
#   设了弄坏开关的整份不并行，自己是一路），成功句报并行路数。设了弄坏开关的整份走 run_selftest（门禁 break-switch-replay 不指定范围时每条开关整份起，并行度由它定，这里不再叠一层）。
#   点名了开关轮的一段（wheel_segment_number 是 1 到段数）的不跑各格、直接跑那一段的开关；点名 base（0）的各格照跑、开关轮不起
selftest_parent_work=""
run_selftest_parallel() {
  local width cells
  selftest_service_refusal || return 2
  if ! width="$(selftest_parallel_width)"; then
    echo "  ✗ 自证并行的路数取不出（CRASH_AMPLIFICATION_SELFTEST_JOBS 或每台默认重活线程数给的不是正整数），一路都没起" >&2
    echo "     → 怎么办：CRASH_AMPLIFICATION_SELFTEST_JOBS 写成正整数或不设；不设时照 .claude/heavy-thread-default 文件头写好登记再跑" >&2
    return 2
  fi
  selftest_parent_work="$(mktemp -d "${TMPDIR:-/tmp}/crash-point-replay-selftest-workers.XXXXXX")" || {
    echo "  ✗ --selftest：建不了放各路输出的临时目录（TMPDIR=${TMPDIR:-/tmp}）" >&2
    echo "  → 怎么办：看 TMPDIR 可不可写" >&2
    return 2
  }
  trap 'rm -rf -- "${selftest_parent_work:?}"' EXIT
  if [[ -n "$wheel_segment_number" && "$wheel_segment_number" != 0 ]]; then   # 点名了开关轮的一段：不跑基准趟各格，只跑这一段的开关（文件头「开关轮分段」）
    selftest_wheel_and_summary "$selftest_parent_work" 0 "$(selftest_report_parallel_wheel "$width")"
  fi
  mapfile -t selftest_worker_assignments < <(selftest_worker_cells "$replay_selftest_library" "$width")
  if (( ${#selftest_worker_assignments[@]} == 0 )); then
    echo "  ✗ --selftest：从 $replay_selftest_library 里读不出任何一块「if selftest_cell_wanted <格名>」，一路都没起" >&2
    echo "     → 怎么办：看 selftest_worker_cells 读的写法（行首两个空格的 if selftest_cell_wanted）" >&2
    return 1
  fi
  selftest_run_workers "$selftest_parent_work" "$width" -- bash "$replay_script" --selftest
  selftest_collect_workers "$selftest_parent_work"
  cells="$selftest_workers_cells"
  if (( selftest_workers_failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $selftest_workers_failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest；单跑一格：CRASH_POINT_REPLAY_SELFTEST_CELLS=<格名> $0 --selftest" >&2
    return 1
  fi
  selftest_wheel_and_summary "$selftest_parent_work" "$cells" "$(selftest_report_parallel_cells "${#selftest_worker_assignments[@]}" "$width")"
}
