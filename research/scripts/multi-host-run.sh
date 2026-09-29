#!/usr/bin/env bash
# 把一件确定性的活挪到第二台跑（用户 2026-09-28 定）：默认在本机仓根跑，带 --peer 才把这棵树拷到第二台、在那边跑、取回点名的产物。
#
#   multi-host-run.sh [--peer] [--memory-cap <上限>] [--label <标签>] [--fetch <树里的相对路径>]… [--send <本机目录> <树里的相对路径>]… -- <命令与参数…>
#   multi-host-run.sh --selftest
#   --send 只在 --peer 时有意义：④ 拷完树、提交之后，把点名的本机目录 rsync 到第二台那棵树里的相对路径下（例如上一次跑积累的判定库，
#   让第二台复用已核过的块）；不带 --peer 给 --send 是用法错（退 2）。
#
# 不带 --peer：不读多机配置，在本机仓根（这份脚本所在的树）经 research/scripts/run-with-memory-cap.sh <上限> 跑那条命令，
#   上限默认 8G（照 research/scripts/replay.sh 的 REPLAY_MEMORY_CAP 默认值）。
# 带 --peer：
#   ① 配置经 research/scripts/peer-host-lib.sh 的 peer_host_load_configuration 读（判法是 research/scripts/layer0-shard-configuration-check.sh，
#      配置 ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，模板是仓根 multi-host.env.example）；判不过退 255。
#      读之前先单跑一次判法看双机开关（ENABLE_ACROSS_MACHINES，定义在那份判法里；判法退 3 是「双机：关」）：没开就拒、退 255，
#      出路是在配置里写一行 ENABLE_ACROSS_MACHINES=1，不偷偷退回本机跑（用户 2026-09-28 定：双机要参数开启）。
#   ② 上限默认取配置的 PEER_MEMORY_CAP；--memory-cap 给的比它大就拒（退 2）。
#   ③ 两台工具链比一遍（peer_host_compare_toolchains：rustc -Vv 前三行与 host 行、cargo -V），不同就拒、说清差在哪，退 255。
#   ④ 树拷到第二台 <PEER_REPOSITORY_DIRECTORY>/runs/<标签>/（新建目录，标签已在就拒、退 2）：拷的是 git 列得出的文件
#      （git ls-files -co --exclude-standard：跟踪的与没被忽略的新文件，所以不带 .git、target、本地配置 multi-host.env 与别的被忽略的文件），
#      rsync 不带 --delete；规范副本另拷（peer_host_sop_source：树里有拷树里的，没有拷主仓的），拷完回读 scripts/preflight.py；
#      在副本里 git init、git add -A、提交一次（给 preflight 的输入指纹用）。
#   ⑤ 在副本的树根经副本里的 run-with-memory-cap.sh <上限> 跑那条命令，脱离 ssh 会话起（peer_host_start_detached）：
#      进程号、退出码、输出（标准输出与标准错误合成一份）各写第二台 runs/<标签>.pid / .exit / .log，每 MULTI_HOST_RUN_POLL_SECONDS（默认 5）秒
#      经 ssh 看一次（peer_host_wait_detached）；跑完输出拷回、经 peer_host_redact 打到标准输出。
#      带到第二台的环境：RUN_WITH_MEMORY_CAP_RESERVE 设了就带；本机别的环境变量不带（真的第二台经 ssh 也拿不到），要设写成 -- env 变量=值 <命令>。
#   ⑥ --fetch 点名的路径（文件或目录，相对树根，不许绝对路径与 ..）取回本机取回目录；不带 --peer 时从本机仓根拷过去，两种一样放。
#      取回目录：${MULTI_HOST_RUN_FETCH_DIRECTORY:-${TMPDIR:-/tmp}/singlefs-multi-host-run/<标签>}，路径在里面照原样；已有东西就拒（退 2）。
#   ⑦ 删第二台这一趟的树与 .pid / .exit / .log；④ 之后在哪一步失败退出，这一步都挂在 EXIT 上照做。
# 收到 TERM / INT / HUP（退 143 / 130 / 129）：先停那条命令——本机的从它的进程号起、第二台的从它在第二台上记下的进程号起（peer_host_stop_detached），
#   第二台的停下之后先取回 --fetch 点名的路径（写到一半的产物不随树一起丢），
#   连同全部后代冻住、从最底层起 TERM、放开，至多等 30 秒，还在的 KILL——再删第二台这一趟的树，做完才退。
# 退出码：那条命令的退出码（250–254 是内存包装自己的结局，含义见 run-with-memory-cap.sh 文件头「退出码」，那一次的输出不算结果）；
#   2 用法错（命令一行都没跑）；255 这个脚本自己没跑成（配置、工具链、拷树、起不来、等不到结局，或命令退 0 而点名取回的路径不在）。
#   命令退非 0 时点名的路径不在，只提一句，退出码照旧是命令的。
# 驱动打的每一行都不写配置里第二台的主机、别名与目录：第二台的输出与错误输出经 peer_host_redact 换成「第二台」「第二台的目录」再打。
# 共用库 research/scripts/peer-host-lib.sh 只调它的对外函数，不改它。
#
# gate-similar: layer0-shard-run.sh 它把一条登记了分片的崩溃枚举用例切成两片、两台各跑一片再 merge 并写 54 号的全绿标记，步骤与输入指纹都绑在那一类用例上；它按内容进三条用例的输入指纹（admission.py 的 SHARD_DRIVER_FILES），并进来改一行就废三格标记
# gate-similar: mutation-shard-run.sh 它是 checker-tier-crates-mutation-replay 专用的变异表分行驱动：按用时分两堆、两台各跑一份 checker-tier-crates-mutation-replay、拷回按条记录核过再导入，判定归 checker-tier-crates-mutation-replay；这里只挪一条任意命令，不分行、不导记录，两份共用的第二台那几段已经抽在 peer-host-lib.sh
#
# 弄坏开关 MULTI_HOST_RUN_BREAK=<项>（只给证红用，逗号分隔可给几个；各项打开时 --selftest 对应那一格判错）：
#   local-reads-configuration  不带 --peer 也读配置（「默认本机跑、不读配置」判错）
#   local-without-memory-cap   不带 --peer 时不经 run-with-memory-cap.sh 跑（「默认本机跑」那一格的 cgroup 判错）
#   peer-exit-dropped          第二台那条命令退几都退 0（「退出码原样带回」判错）
#   peer-without-memory-cap    第二台那条命令不经副本里的 run-with-memory-cap.sh 跑（「--peer 跑成」那一格的 cgroup 判错）
#   no-peer-cap-limit          --memory-cap 比配置的 PEER_MEMORY_CAP 大也照跑（「上限不大于 PEER_MEMORY_CAP」判错）
#   no-toolchain-check         不比两台工具链（「工具链不同就拒」判错）
#   fetch-altered              取回之后往每个取回的文件末尾加一个字节（「--fetch 取回的文件逐字相同」判错）
#   no-peer-cleanup            跑完（成功、失败都算）不删第二台这一趟的树（「跑完删第二台那一趟的目录」判错）
#   no-peer-stop-on-signal     收到 TERM 不停第二台那条命令（「收到 TERM 先停第二台那条命令」判错）
#   no-redact                  第二台的输出不经 peer_host_redact 就打（「输出里没有配置里的主机与目录」判错）
#   switch-off-falls-back-to-local  配置里双机没开时退回本机跑（「双机开关没开 ⇒ --peer 拒」判错）
#   sync-ignored-files         树整个拷过去（只排除 .git、target），被 git 忽略的文件也拷（「--peer 跑成」那一格的「被忽略的文件没拷过去」判错）
# 只供测试的开关：SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1（照层 0 分片）：「第二台」是本机上的另一个目录，命令在本机跑、环境照 ssh 那样只给最少几样。
#
# admission: always 每次调都在这一刻这棵树上跑一条调用方给的命令；要不要重跑由调用方判，不在这里
# run-condition: command bash git rsync python3
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
multi_host_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=peer-host-lib.sh
source "$multi_host_script_directory/peer-host-lib.sh"

MULTI_HOST_LOCAL_DEFAULT_MEMORY_CAP=8G

multi_host_break_is() { [[ ",${MULTI_HOST_RUN_BREAK:-}," == *",$1,"* ]]; }

multi_host_usage_error() { # <原因>：用法错，命令一行都没跑，退 2
  echo "  ✗ multi-host-run.sh 用法错：$1" >&2
  echo "     → 怎么办：multi-host-run.sh [--peer] [--memory-cap <上限>] [--label <标签>] [--fetch <树里的相对路径>]… [--send <本机目录> <树里的相对路径>]… -- <命令与参数…>，或 --selftest；上限写正整数加 K / M / G / T，标签只用字母、数字、点、减号与下划线；--send 只配 --peer" >&2
  exit 2
}

multi_host_fail() { # <原因> <出路>：这个脚本自己没跑成，退 255（EXIT 上删第二台这一趟的树）
  echo "  ✗ multi-host-run.sh：$1" >&2
  echo "     → 怎么办：$2" >&2
  exit 255
}

multi_host_cap_bytes() { # <上限>：正整数加 K / M / G / T，打成字节数
  local number="${1%[KMGT]}" power
  case "${1: -1}" in K) power=1 ;; M) power=2 ;; G) power=3 ;; *) power=4 ;; esac
  echo $(( number * 1024 ** power ))
}

multi_host_redact() { # 第二台的输出：经 peer_host_redact 换掉私有的字再打；弄坏开关 no-redact 下原样打
  if multi_host_break_is no-redact; then cat; else peer_host_redact; fi
}

multi_host_remove_peer_run() { # ⑦ 删第二台这一趟的树与 .pid / .exit / .log，回读不在了才算删了
  peer_run_pending=0
  if multi_host_break_is no-peer-cleanup; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-peer-cleanup：第二台 runs/$run_label 不删" >&2
    return 0
  fi
  local removal_text
  removal_text="rm -rf -- $(printf '%q' "${peer_run_directory:?}") $(printf '%q' "${peer_pid_file:?}") $(printf '%q' "${peer_exit_file:?}") $(printf '%q' "${peer_output_file:?}")"
  if peer_host_run / "$removal_text && test ! -e $(printf '%q' "$peer_run_directory")" >/dev/null; then
    echo "  · ⑦ 删了第二台 runs/$run_label 这一趟的树与进程号、退出码、输出三个文件（回读不在了）" >&2
  else
    echo "  ✗ 第二台 runs/$run_label 这一趟的树没删掉" >&2
    echo "     → 怎么办：到第二台 PEER_REPOSITORY_DIRECTORY 下的 runs/ 里手动删 $run_label 与 $run_label.pid / .exit / .log" >&2
  fi
}

multi_host_on_exit() {
  trap '' TERM INT HUP PIPE
  if (( peer_run_pending )); then multi_host_remove_peer_run; fi
  if [[ -n "${multi_host_scratch:-}" ]]; then rm -rf -- "${multi_host_scratch:?}"; fi
}

multi_host_stop_and_exit() { # <信号名> <退出码>：先停那条命令，再 exit（EXIT 上删第二台这一趟的树）
  local stop_output
  trap '' TERM INT HUP PIPE
  echo "  ! 收到 $1：先停那条命令，再删第二台这一趟的树，做完再退" >&2
  if [[ -n "$local_command_process" ]]; then
    if stop_output="$(peer_host_stop_process_tree "$local_command_process")"; then
      echo "  · 本机那条命令：$stop_output" >&2
    else
      echo "  ✗ 本机那条命令没停下：$stop_output" >&2
      echo "     → 怎么办：照上面列的进程号逐个 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>" >&2
    fi
  fi
  if (( peer_command_started )); then
    if multi_host_break_is no-peer-stop-on-signal; then
      echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-peer-stop-on-signal：第二台那条命令不停" >&2
    elif stop_output="$(peer_host_stop_detached "$peer_run_directory" "$peer_pid_file")"; then
      echo "  · 第二台那条命令：$stop_output" >&2
    else
      echo "  ✗ 第二台那条命令没停下：$stop_output" >&2
      echo "     → 怎么办：到第二台 runs/$run_label.pid 里的进程号，连同后代逐个 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>" >&2
    fi
    # 停下之后先把 --fetch 点名的路径取回来再删树：那条命令写到一半的产物（崩溃放量判到一半的库）不随树一起丢
    if (( peer_run_pending )) && (( ${#fetch_paths[@]} > 0 )); then multi_host_fetch; fi
  fi
  exit "$2"
}

multi_host_fetch() { # ⑥ 取回 --fetch 点名的路径：本机模式从仓根拷，--peer 从第二台这一趟的树拷。缺的路径写进 fetch_missing
  local relative_path destination
  fetch_missing=()
  (( ${#fetch_paths[@]} > 0 )) || return 0
  for relative_path in "${fetch_paths[@]}"; do
    destination="$fetch_directory/$relative_path"
    mkdir -p -- "$(dirname "$destination")" || multi_host_fail "建不了取回目录 $(dirname "$destination")" "看 ${TMPDIR:-/tmp} 可不可写、盘满没满"
    if (( ! peer_mode )); then
      if [[ -d "$tree_root/$relative_path" ]]; then
        { mkdir -p -- "$destination" && cp -a -- "$tree_root/$relative_path/." "$destination/"; } || fetch_missing+=("$relative_path")
      elif [[ -e "$tree_root/$relative_path" ]]; then
        cp -a -- "$tree_root/$relative_path" "$destination" || fetch_missing+=("$relative_path")
      else
        fetch_missing+=("$relative_path")
      fi
    elif peer_host_run "$peer_run_directory" "test -d $(printf '%q' "$relative_path")" >/dev/null 2>&1; then
      peer_host_copy "$(peer_host_path "$peer_run_directory/$relative_path")/" "$destination/" || fetch_missing+=("$relative_path")
    elif peer_host_run "$peer_run_directory" "test -e $(printf '%q' "$relative_path")" >/dev/null 2>&1; then
      peer_host_copy "$(peer_host_path "$peer_run_directory/$relative_path")" "$destination" || fetch_missing+=("$relative_path")
    else
      fetch_missing+=("$relative_path")
    fi
  done
  if multi_host_break_is fetch-altered; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=fetch-altered：取回的每个文件末尾加一个字节" >&2
    find "$fetch_directory" -type f -exec sh -c 'printf x >> "$1"' sh {} \;
  fi
  echo "  · ⑥ 取回 $(( ${#fetch_paths[@]} - ${#fetch_missing[@]} )) 条（共点名 ${#fetch_paths[@]} 条）到 $fetch_directory" >&2
}

multi_host_finish() { # <命令的退出码>：缺了取回的路径、包装自己的结局各提一句，按文件头「退出码」那一段退
  local command_exit="$1"
  case "$command_exit" in
    25[0-4]) echo "  ! 那条命令退 $command_exit：内存包装 run-with-memory-cap.sh 自己的结局（文件头「退出码」），那一次的输出不算结果" >&2 ;;
  esac
  if (( ${#fetch_missing[@]} > 0 )); then
    if [[ "$command_exit" == 0 ]]; then
      multi_host_fail "那条命令退 0，点名取回的 ${fetch_missing[*]} 却不在" "看命令是不是真把它们写在树根下这几条相对路径上（--fetch 相对树根写）"
    fi
    echo "  ! 点名取回的 ${fetch_missing[*]} 不在（那条命令退 $command_exit，多半没写到那一步）" >&2
  fi
  exit "$command_exit"
}

multi_host_local_main() { # 不带 --peer：在本机仓根经 run-with-memory-cap.sh 跑，不读配置
  local command_exit wrapper=(bash "$tree_root/research/scripts/run-with-memory-cap.sh" "$memory_cap")
  if multi_host_break_is local-reads-configuration; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=local-reads-configuration：不带 --peer 也读配置" >&2
    peer_host_load_configuration "$multi_host_script_directory/layer0-shard-configuration-check.sh" "$tree_root" >&2 || exit 255
  fi
  if multi_host_break_is local-without-memory-cap; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=local-without-memory-cap：不经 run-with-memory-cap.sh 跑" >&2
    wrapper=()
  fi
  echo "  · 本机仓根跑（不跨机，不读多机配置）：经 run-with-memory-cap.sh $memory_cap 起" >&2
  ( cd "$tree_root" && exec ${wrapper[@]+"${wrapper[@]}"} "$@" ) <&0 &
  local_command_process=$!
  if wait "$local_command_process"; then command_exit=0; else command_exit=$?; fi
  local_command_process=""
  echo "  · 那条命令退 $command_exit" >&2
  multi_host_fetch
  multi_host_finish "$command_exit"
}

multi_host_peer_main() { # 带 --peer：步骤 ①–⑦ 见文件头
  local cap_limit_bytes files_list sop_source listed_path command_text reserve_text="" command_exit switch_exit
  # ① 配置：双机开关先判（判法退 3 是「双机：关」）；关着就拒，不退回本机
  if bash "$multi_host_script_directory/layer0-shard-configuration-check.sh" "$tree_root" >/dev/null 2>&1; then switch_exit=0; else switch_exit=$?; fi
  if [[ "$switch_exit" == 3 ]]; then
    if multi_host_break_is switch-off-falls-back-to-local; then
      echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=switch-off-falls-back-to-local：双机关着，退回本机跑" >&2
      return 0
    fi
    multi_host_fail "带了 --peer，配置里的双机开关却没开（ENABLE_ACROSS_MACHINES 没写或不是 1）：不去第二台，也不退回本机跑" \
      "要跨机就在配置（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}）里写一行 ENABLE_ACROSS_MACHINES=1；不跨机就去掉 --peer"
  fi
  peer_host_load_configuration "$multi_host_script_directory/layer0-shard-configuration-check.sh" "$tree_root" >&2 || exit 255
  if [[ "$peer_host_is_this_machine" == 1 ]]; then
    echo "  ! 第二台是本机上的另一个目录（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1）" >&2
  fi
  # ② 上限
  if [[ -z "$memory_cap" ]]; then
    memory_cap="$PEER_MEMORY_CAP"
  else
    cap_limit_bytes="$(multi_host_cap_bytes "$PEER_MEMORY_CAP")"
    if (( $(multi_host_cap_bytes "$memory_cap") > cap_limit_bytes )); then
      if multi_host_break_is no-peer-cap-limit; then
        echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-peer-cap-limit：--memory-cap $memory_cap 比配置的 PEER_MEMORY_CAP $PEER_MEMORY_CAP 大也照跑" >&2
      else
        multi_host_usage_error "--memory-cap $memory_cap 比配置的 PEER_MEMORY_CAP $PEER_MEMORY_CAP 大（第二台上的上限不许超过它）"
      fi
    fi
  fi
  # ③ 工具链
  if multi_host_break_is no-toolchain-check; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=no-toolchain-check：不比两台工具链" >&2
  else
    peer_host_compare_toolchains >&2 || exit 255
    echo "  · ③ 两台的工具链相同（${peer_host_toolchain_text}）" >&2
  fi
  # ④ 树拷到第二台
  sop_source="$(peer_host_sop_source "$tree_root" "$tree_root")" \
    || multi_host_fail "树里与它的主仓里都没有规范副本 .claude/singlefs-ai-sop/（要有 scripts/preflight.py）：第二台那棵树的脚本一步都判不了准入" "在主仓里装好它（bash .claude/singlefs-ai-sop/install.sh）"
  local peer_runs="$PEER_REPOSITORY_DIRECTORY/runs"
  peer_run_directory="$peer_runs/$run_label"
  peer_pid_file="$peer_runs/$run_label.pid"
  peer_exit_file="$peer_runs/$run_label.exit"
  peer_output_file="$peer_runs/$run_label.log"
  if ! peer_host_run / "test ! -e $(printf '%q' "$peer_run_directory") && test ! -e $(printf '%q' "$peer_pid_file") && test ! -e $(printf '%q' "$peer_exit_file") && test ! -e $(printf '%q' "$peer_output_file")" >/dev/null; then
    multi_host_usage_error "第二台 runs/ 下已经有 $run_label（或它的 .pid / .exit / .log），或第二台连不上"
  fi
  peer_host_run / "mkdir -p $(printf '%q' "$peer_run_directory")" >/dev/null \
    || multi_host_fail "第二台建不了 runs/$run_label" "看配置的 PEER_REPOSITORY_DIRECTORY 可不可写、盘满没满"
  peer_run_pending=1
  files_list="$multi_host_scratch/files"
  git -C "$tree_root" ls-files -co --exclude-standard -z --deduplicate > "$multi_host_scratch/listed" \
    || multi_host_fail "git ls-files 列不出这棵树的文件" "在 git 工作树里跑（树根是这份脚本所在的仓）"
  : > "$files_list"
  while IFS= read -r -d '' listed_path; do
    case "$listed_path" in multi-host.env|.claude/singlefs-ai-sop|.claude/singlefs-ai-sop/*) continue ;; esac
    if [[ -e "$tree_root/$listed_path" || -L "$tree_root/$listed_path" ]]; then printf '%s\0' "$listed_path" >> "$files_list"; fi
  done < "$multi_host_scratch/listed"
  if multi_host_break_is sync-ignored-files; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=sync-ignored-files：树整个拷过去，被 git 忽略的文件也拷" >&2
    printf '%s\0' . > "$files_list"
  fi
  peer_host_copy --exclude target --exclude .git --exclude /.claude/singlefs-ai-sop --from0 --files-from="$files_list" -r "$tree_root/" "$(peer_host_path "$peer_run_directory")/" \
    || multi_host_fail "树 rsync 到第二台失败" "看第二台盘满没满、配置的 PEER_REPOSITORY_DIRECTORY 可不可写"
  peer_host_copy "$sop_source/" "$(peer_host_path "$peer_run_directory/.claude/singlefs-ai-sop")/" \
    || multi_host_fail "规范副本 rsync 到第二台失败" "看第二台盘满没满"
  peer_host_run "$peer_run_directory" "test -f .claude/singlefs-ai-sop/scripts/preflight.py && git init -q . && git add -A && git -c user.name=multi-host-run -c user.email=multi-host-run@localhost.invalid commit -q --no-verify --no-gpg-sign -m multi-host-run" >/dev/null \
    || multi_host_fail "第二台那棵树回读不到规范副本的 scripts/preflight.py，或 git init、提交失败" "看第二台那棵树可不可写、装没装 git"
  echo "  · ④ 树拷到第二台 runs/$run_label（git 列得出的 $(tr -cd '\0' < "$files_list" | wc -c) 个文件，规范副本另拷），在那里 git init 并提交一次" >&2
  # ④ 之后：--send 点名的本机目录送到那棵树里（提交之后送：它不是树的一部分，例如上一次跑积累的判定库，让第二台复用已核过的块）
  for send_pair in ${send_pairs[@]+"${send_pairs[@]}"}; do
    peer_host_copy "${send_pair%%|*}/" "$(peer_host_path "$peer_run_directory/${send_pair#*|}")/" \
      || multi_host_fail "--send 的目录 ${send_pair%%|*} rsync 到第二台失败" "看第二台盘满没满、配置的 PEER_REPOSITORY_DIRECTORY 可不可写"
    echo "  · ④ 之后送到第二台那棵树的 ${send_pair#*|}：$(du -sh -- "${send_pair%%|*}" | cut -f1)" >&2
  done
  # ⑤ 在第二台跑
  if [[ -n "${RUN_WITH_MEMORY_CAP_RESERVE:-}" ]]; then reserve_text="RUN_WITH_MEMORY_CAP_RESERVE=$(printf '%q' "$RUN_WITH_MEMORY_CAP_RESERVE") "; fi
  command_text="${reserve_text}bash research/scripts/run-with-memory-cap.sh $(printf '%q' "$memory_cap") $(printf '%q ' "$@")"
  if multi_host_break_is peer-without-memory-cap; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-without-memory-cap：第二台那条命令不经 run-with-memory-cap.sh 跑" >&2
    command_text="$(printf '%q ' "$@")"
  fi
  peer_command_started=1   # 先置上：起的那一刻收到信号，也按第二台上记下的进程号去停
  peer_host_start_detached "$peer_run_directory" "$peer_pid_file" "$peer_exit_file" "$peer_output_file" "$command_text" >/dev/null \
    || multi_host_fail "第二台那条命令起不来" "看 ssh 连不连得上、第二台那棵树在不在"
  echo "  · ⑤ 第二台开跑（脱离 ssh 会话起，经副本里的 run-with-memory-cap.sh $memory_cap），每 $poll_seconds 秒看一次" >&2
  peer_host_wait_detached "$peer_pid_file" "$peer_exit_file" "$poll_seconds" >&2
  peer_command_started=0
  if peer_host_copy "$(peer_host_path "$peer_output_file")" "$multi_host_scratch/peer.log" 2>/dev/null; then
    multi_host_redact < "$multi_host_scratch/peer.log"
  else
    echo "  ! 第二台那条命令的输出没拷回来" >&2
  fi
  case "$peer_host_detached_outcome" in
    "exit "*) command_exit="${peer_host_detached_outcome#exit }" ;;
    *) multi_host_fail "第二台那条命令没等到退出码：$peer_host_detached_outcome（vanished 进程没了又没留退出码；unreachable 连续 60 次 ssh 不通；never-started 60 次都没起来）" "看上面拷回的输出；ssh 到第二台 runs/ 下看 $run_label.log" ;;
  esac
  echo "  · 第二台那条命令退 $command_exit" >&2
  if multi_host_break_is peer-exit-dropped; then
    echo "  ! 弄坏开关 MULTI_HOST_RUN_BREAK=peer-exit-dropped：退出码不带回，一律退 0" >&2
    command_exit=0
  fi
  # ⑥ 取回、⑦ 删第二台这一趟的树
  multi_host_fetch
  multi_host_remove_peer_run
  multi_host_finish "$command_exit"
}

multi_host_main() {
  peer_mode=0; memory_cap=""; run_label=""; fetch_paths=(); fetch_missing=(); send_pairs=()
  local separator_seen=0 fetch_path send_pair
  while (( $# > 0 )); do
    case "$1" in
      --peer) peer_mode=1; shift ;;
      --memory-cap) (( $# >= 2 )) || multi_host_usage_error "--memory-cap 后面没给上限"; memory_cap="$2"; shift 2 ;;
      --label) (( $# >= 2 )) || multi_host_usage_error "--label 后面没给标签"; run_label="$2"; shift 2 ;;
      --fetch) (( $# >= 2 )) || multi_host_usage_error "--fetch 后面没给路径"; fetch_paths+=("$2"); shift 2 ;;
      --send) (( $# >= 3 )) || multi_host_usage_error "--send 后面要给 <本机目录> <树里的相对路径>"; send_pairs+=("$2|$3"); shift 3 ;;
      --) separator_seen=1; shift; break ;;
      *) multi_host_usage_error "认不出的参数「$1」（命令要写在 -- 后面）" ;;
    esac
  done
  (( separator_seen && $# > 0 )) || multi_host_usage_error "-- 后面没有命令"
  if [[ -n "$memory_cap" && ! "$memory_cap" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
    multi_host_usage_error "--memory-cap 读到「$memory_cap」，要写正整数加 K / M / G / T（单位必写，例 8G）"
  fi
  [[ -n "$run_label" ]] || run_label="multi-host-$(date -u +%Y%m%dT%H%M%SZ)-$$"
  [[ "$run_label" =~ ^[A-Za-z0-9][A-Za-z0-9._-]*$ ]] || multi_host_usage_error "--label 读到「$run_label」"
  for fetch_path in ${fetch_paths[@]+"${fetch_paths[@]}"}; do
    if [[ -z "$fetch_path" || "$fetch_path" == /* || "$fetch_path" =~ (^|/)\.\.(/|$) ]]; then
      multi_host_usage_error "--fetch 读到「$fetch_path」：要写树里的相对路径，不许绝对路径与 .."
    fi
  done
  for send_pair in ${send_pairs[@]+"${send_pairs[@]}"}; do
    (( peer_mode )) || multi_host_usage_error "--send 只在 --peer 时有意义（本机模式命令就在仓根跑，没有第二棵树可送）"
    local send_source="${send_pair%%|*}" send_relative="${send_pair#*|}"
    [[ -d "$send_source" ]] || multi_host_usage_error "--send 读到「$send_source」：要是本机已有的目录"
    if [[ -z "$send_relative" || "$send_relative" == /* || "$send_relative" =~ (^|/)\.\.(/|$) ]]; then
      multi_host_usage_error "--send 的落点读到「$send_relative」：要写树里的相对路径，不许绝对路径与 .."
    fi
  done
  fetch_directory="${MULTI_HOST_RUN_FETCH_DIRECTORY:-${TMPDIR:-/tmp}/singlefs-multi-host-run/$run_label}"
  if (( ${#fetch_paths[@]} > 0 )) && [[ -e "$fetch_directory" ]] && [[ -n "$(ls -A -- "$fetch_directory" 2>/dev/null)" ]]; then
    multi_host_usage_error "取回目录 $fetch_directory 里已经有东西（换一个 --label，或删掉它、或把 MULTI_HOST_RUN_FETCH_DIRECTORY 指到空目录）"
  fi
  poll_seconds="${MULTI_HOST_RUN_POLL_SECONDS:-5}"
  [[ "$poll_seconds" =~ ^[1-9][0-9]*$ ]] || multi_host_usage_error "MULTI_HOST_RUN_POLL_SECONDS 读到「$poll_seconds」，要写正整数秒"
  tree_root="$(cd "$multi_host_script_directory/../.." && pwd)"
  local_command_process=""
  peer_command_started=0
  peer_run_pending=0
  multi_host_scratch="$(mktemp -d)"
  trap multi_host_on_exit EXIT
  trap 'multi_host_stop_and_exit TERM 143' TERM
  trap 'multi_host_stop_and_exit INT 130' INT
  trap 'multi_host_stop_and_exit HUP 129' HUP
  if (( peer_mode )); then
    multi_host_peer_main "$@"
    peer_mode=0   # 只有弄坏开关 switch-off-falls-back-to-local 走得到这里
  fi
  [[ -n "$memory_cap" ]] || memory_cap="$MULTI_HOST_LOCAL_DEFAULT_MEMORY_CAP"
  multi_host_local_main "$@"
}

# ── 自证 ──────────────────────────────────────────────────────────────────────
# 在临时目录里搭一棵小仓（本仓的 .claude/scripts、research/scripts、规范副本的符号链接、一个被 git 忽略的 .env），
# 「第二台」是本机上的另一个目录（SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1），配置写在场景目录里（SINGLEFS_MULTI_HOST_CONFIG）。
# 命令都经真的 run-with-memory-cap.sh（上限 256M）。逐格（弄坏开关见文件头，各让对应那一格判错）：
#   Ⓐ 默认本机跑：配置是坏的也照跑、退 0、在本机仓根跑、第二台一个目录都没建；经 run-with-memory-cap.sh（cgroup 在 singlefs-memory-cap- 的 scope 里）；
#      --fetch 取回的文件与仓根那一份逐字相同
#   Ⓑ --peer 跑成：退 0，在第二台 runs/<标签>/ 那棵副本里跑（副本里有规范副本、提交过一次），被 git 忽略的 .env 没拷过去；经副本里的 run-with-memory-cap.sh 跑；
#      命令退 7 ⇒ 驱动退 7；--memory-cap 比 PEER_MEMORY_CAP 大 ⇒ 拒（退 2）、命令没跑
#   Ⓒ 第二台的 rustc -Vv 不同 ⇒ 拒（退 255，说两台的工具链不同），命令没跑、第二台没建这一趟的目录
#   Ⓓ --fetch 一个文件、一个目录：取回的逐字相同（与命令在第二台上算的 sha256 对）
#   Ⓔ Ⓑ Ⓓ 那几趟跑完（含退 7 那一趟）第二台 runs/ 下什么都不剩
#   Ⓕ 第二台那条命令在跑时给驱动（只给它自己，kill "$!"）发 TERM ⇒ 退 143，第二台那条命令不在了，runs/ 下什么都不剩
#   Ⓖ 输出（驱动的标准输出与标准错误）里没有配置里的主机名与第二台的目录，而命令打的那两样换成了「第二台」「第二台的目录」
#   Ⓗ 配置里没写 ENABLE_ACROSS_MACHINES ⇒ --peer 拒（退 255，说在配置里写 ENABLE_ACROSS_MACHINES=1），命令哪台都没跑

multi_host_selftest() {
  local repository_root work template sop_source checked=0 failures=0
  repository_root="$(cd "$multi_host_script_directory/../.." && pwd)"
  work="$(mktemp -d)"
  multi_host_selftest_work="$work"
  trap 'echo "  · 自证的临时目录（$(du -sh "$multi_host_selftest_work" 2>/dev/null | cut -f1)）删掉"; rm -rf -- "${multi_host_selftest_work:?}"' EXIT
  expect() { # expect <格名> <判据的退出码：0 为过> <不过时的细节>
    checked=$((checked + 1))
    if [[ "$2" == 0 ]]; then
      echo "  ✓ $1"
    else
      echo "  ✗ $1：$3"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  }
  if ! sop_source="$(peer_host_sop_source "$repository_root" "$repository_root")"; then
    echo "  ✗ multi-host-run.sh 自证搭不起小仓：本仓与它的主仓里都没有规范副本 .claude/singlefs-ai-sop/"
    echo "     → 怎么办：在主仓里装好它（bash .claude/singlefs-ai-sop/install.sh）再跑"
    return 1
  fi

  # 小仓的模板
  template="$work/template"
  mkdir -p "$template/.claude" "$template/research" "$template/payload"
  rsync -a --exclude __pycache__ "$repository_root/.claude/scripts/" "$template/.claude/scripts/"
  rsync -a --exclude __pycache__ --exclude memory-peaks.tsv "$repository_root/research/scripts/" "$template/research/scripts/"
  ln -s "$sop_source" "$template/.claude/singlefs-ai-sop"
  printf '/.claude/singlefs-ai-sop\n__pycache__/\n/multi-host.env\n/.env\n/research/scripts/memory-peaks.tsv\ntarget/\n' > "$template/.gitignore"
  printf '多机脚本自证的输入\n' > "$template/payload/input.txt"
  printf 'SELFTEST_SECRET=被忽略的文件不许拷到第二台\n' > "$template/.env"

  prepare_scene() { # <场景名> [配置里 ENABLE_ACROSS_MACHINES 那一行，默认 ENABLE_ACROSS_MACHINES=1]：repo（git init 过）、peer-home、peer-bin、配置
    local scene="$work/$1"
    mkdir -p "$scene/peer-home" "$scene/peer-bin"
    rsync -a "$template/" "$scene/repo/"
    git -C "$scene/repo" init -q
    {
      echo "PEER_SSH_HOST=selftest-multi-host-peer-name"
      echo "PEER_REPOSITORY_DIRECTORY=$scene/peer-home"
      echo "PEER_CARGO_BIN_DIRECTORY=$scene/peer-bin"
      echo "PEER_MEMORY_CAP=1G"
      echo "${2-ENABLE_ACROSS_MACHINES=1}"
      echo "QUIESCE_STOP_COMMAND="
      echo "QUIESCE_STOPPED_CHECK_COMMAND="
      echo "QUIESCE_START_COMMAND="
      echo "QUIESCE_STARTED_CHECK_COMMAND="
    } > "$scene/multi-host.env"
  }
  scene_command_for() { # <场景名>：scene_command 设成在这个场景里起驱动的前缀（弄坏开关从调用方的环境里留着）
    local scene="$work/$1"
    scene_command=(env -u RUN_WITH_MEMORY_CAP_RESERVE -u RUN_WITH_MEMORY_CAP_KEY SINGLEFS_MULTI_HOST_CONFIG="$scene/multi-host.env" SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1
      MULTI_HOST_RUN_POLL_SECONDS=1 MULTI_HOST_RUN_FETCH_DIRECTORY="$scene/fetch" RUN_WITH_MEMORY_CAP_PEAKS="$scene/peaks.tsv"
      bash "$scene/repo/research/scripts/multi-host-run.sh")
  }
  run_in() { # <场景名> <这一趟的名字> <驱动的参数…>：stdout、stderr、退出码各写场景目录一个文件
    local scene="$work/$1" run_name="$2" exit_code
    shift 2
    scene_command_for "${scene##*/}"
    if "${scene_command[@]}" "$@" > "$scene/$run_name.out" 2> "$scene/$run_name.err" < /dev/null; then exit_code=0; else exit_code=$?; fi
    echo "$exit_code" > "$scene/$run_name.exit"
  }
  exit_of() { cat "$work/$1/$2.exit" 2>/dev/null; }
  tail_of() { tail -n 3 "$work/$1/$2.err" 2>/dev/null | tr '\n' '|'; }
  peer_runs_left() { # <场景名>：第二台 runs/ 下留着的条目数
    find "$work/$1/peer-home/runs" -mindepth 1 -maxdepth 1 2>/dev/null | wc -l
  }

  # Ⓐ 默认本机跑：配置是坏的（判法一读就拒），不带 --peer 照跑
  local scene
  prepare_scene local
  scene="$work/local"
  printf '这一行不是 KEY=值：默认本机跑读了它就会被拒\n' > "$scene/multi-host.env"
  run_in local run --memory-cap 256M --fetch out/local.txt -- bash -c "pwd > $(printf '%q' "$scene/where"); cat /proc/self/cgroup > $(printf '%q' "$scene/cgroup"); mkdir -p out && printf '本机产物\n' > out/local.txt"
  expect "Ⓐ 默认本机跑：配置是坏的也照跑、退 0、在本机仓根跑、第二台一个目录都没建" \
    "$([[ "$(exit_of local run)" == 0 && "$(cat "$scene/where" 2>/dev/null)" == "$scene/repo" && ! -e "$scene/peer-home/runs" ]]; echo $?)" \
    "退 $(exit_of local run)，在「$(cat "$scene/where" 2>/dev/null)」跑；stderr 尾部：$(tail_of local run)"
  expect "Ⓐ 默认本机跑经 run-with-memory-cap.sh（cgroup 在 singlefs-memory-cap- 的 scope 里）" \
    "$(grep -q 'singlefs-memory-cap-' "$scene/cgroup" 2>/dev/null; echo $?)" \
    "命令记下的 cgroup：$(tr '\n' ' ' < "$scene/cgroup" 2>/dev/null)"
  expect "Ⓐ 默认本机跑 --fetch：取回的与仓根那一份逐字相同" \
    "$(cmp -s "$scene/repo/out/local.txt" "$scene/fetch/out/local.txt"; echo $?)" \
    "$(cmp "$scene/repo/out/local.txt" "$scene/fetch/out/local.txt" 2>&1 | head -1)"

  # Ⓑ --peer 跑成、退出码带回、上限不许比 PEER_MEMORY_CAP 大
  prepare_scene peer
  scene="$work/peer"
  run_in peer run --peer --memory-cap 256M --label peer-ok -- bash -c "pwd > $(printf '%q' "$scene/where"); cat /proc/self/cgroup > $(printf '%q' "$scene/cgroup"); \
if test -e .env; then echo synced > $(printf '%q' "$scene/leaked"); fi; test -f .claude/singlefs-ai-sop/scripts/preflight.py && git rev-parse -q --verify HEAD > $(printf '%q' "$scene/head"); \
pwd; echo selftest-multi-host-peer-name"
  expect "Ⓑ --peer 跑成：退 0，在第二台 runs/<标签>/ 那棵副本里跑（有规范副本、提交过一次），被 git 忽略的 .env 没拷过去" \
    "$([[ "$(exit_of peer run)" == 0 && "$(cat "$scene/where" 2>/dev/null)" == "$scene/peer-home/runs/peer-ok" && -s "$scene/head" && ! -e "$scene/leaked" ]]; echo $?)" \
    "退 $(exit_of peer run)，在「$(cat "$scene/where" 2>/dev/null)」跑，提交 $(cat "$scene/head" 2>/dev/null || echo 没有)，.env $([[ -e "$scene/leaked" ]] && echo 拷过去了 || echo 没拷)；stderr 尾部：$(tail_of peer run)"
  expect "Ⓑ --peer 经副本里的 run-with-memory-cap.sh 跑（cgroup 在 singlefs-memory-cap- 的 scope 里）" \
    "$(grep -q 'singlefs-memory-cap-' "$scene/cgroup" 2>/dev/null; echo $?)" \
    "命令记下的 cgroup：$(tr '\n' ' ' < "$scene/cgroup" 2>/dev/null)"
  run_in peer seven --peer --memory-cap 256M --label peer-seven -- bash -c 'exit 7'
  expect "Ⓑ --peer 退出码原样带回：第二台那条命令退 7 ⇒ 驱动退 7" \
    "$([[ "$(exit_of peer seven)" == 7 ]]; echo $?)" "退 $(exit_of peer seven)；stderr 尾部：$(tail_of peer seven)"
  run_in peer too-big --peer --memory-cap 2G --label peer-too-big -- bash -c "touch $(printf '%q' "$scene/too-big-ran")"
  expect "Ⓑ --memory-cap 2G 比配置的 PEER_MEMORY_CAP 1G 大 ⇒ 拒（退 2），命令没跑" \
    "$([[ "$(exit_of peer too-big)" == 2 && ! -e "$scene/too-big-ran" ]] && grep -q 'PEER_MEMORY_CAP' "$scene/too-big.err"; echo $?)" \
    "退 $(exit_of peer too-big)，命令$([[ -e "$scene/too-big-ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of peer too-big)"

  # Ⓒ 工具链不同
  prepare_scene toolchain
  scene="$work/toolchain"
  printf '#!/usr/bin/env bash\nprintf "rustc 0.0.0-other\\nbinary: rustc\\ncommit-hash: 0000\\nhost: other-host\\n"\n' > "$scene/peer-bin/rustc"
  chmod +x "$scene/peer-bin/rustc"
  run_in toolchain run --peer --memory-cap 256M -- bash -c "touch $(printf '%q' "$scene/ran")"
  expect "Ⓒ 第二台的 rustc -Vv 不同 ⇒ 拒（退 255，说两台的工具链不同），命令没跑、第二台没建这一趟的目录" \
    "$([[ "$(exit_of toolchain run)" == 255 && ! -e "$scene/ran" && "$(peer_runs_left toolchain)" == 0 ]] && grep -q '两台的工具链不同' "$scene/run.err"; echo $?)" \
    "退 $(exit_of toolchain run)，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of toolchain run)"

  # Ⓓ --fetch 一个文件、一个目录
  prepare_scene fetch
  scene="$work/fetch"
  run_in fetch run --peer --memory-cap 256M --fetch out/random.bin --fetch out/tree -- bash -c "mkdir -p out/tree/deeper && head -c 200000 /dev/urandom > out/random.bin \
&& printf '取回的目录\n' > out/tree/deeper/a.txt && sha256sum out/random.bin out/tree/deeper/a.txt > $(printf '%q' "$scene/sums")"
  expect "Ⓓ --fetch 取回的一个文件、一个目录与第二台上算的 sha256 逐字相同" \
    "$([[ "$(exit_of fetch run)" == 0 && -s "$scene/sums" ]] && (cd "$scene/fetch" && sha256sum --quiet -c "$scene/sums") >/dev/null 2>&1; echo $?)" \
    "退 $(exit_of fetch run)；核对：$( (cd "$scene/fetch" 2>/dev/null && sha256sum -c "$scene/sums" 2>&1) | tr '\n' '|')；stderr 尾部：$(tail_of fetch run)"

  # Ⓔ 跑完删第二台这一趟的目录
  expect "Ⓔ Ⓑ Ⓓ 那几趟（含退 7 的那一趟）跑完，第二台 runs/ 下什么都不剩" \
    "$([[ "$(peer_runs_left peer)" == 0 && "$(peer_runs_left fetch)" == 0 && -d "$work/peer/peer-home/runs" ]]; echo $?)" \
    "Ⓑ 那边留着：$(ls -A "$work/peer/peer-home/runs" 2>/dev/null | tr '\n' ' ')；Ⓓ 那边留着：$(ls -A "$work/fetch/peer-home/runs" 2>/dev/null | tr '\n' ' ')"

  # Ⓕ 第二台那条命令在跑时收到 TERM
  prepare_scene signalled
  scene="$work/signalled"
  local driver_pid signalled_exit waited sleeper_pid="" sleeper_alive=0
  scene_command_for signalled
  "${scene_command[@]}" --peer --memory-cap 256M --label signalled -- bash -c "echo \$\$ > $(printf '%q' "$scene/sleeper.pid"); exec sleep 300" \
    > "$scene/run.out" 2> "$scene/run.err" < /dev/null &
  driver_pid=$!
  for (( waited = 0; waited < 600; waited++ )); do
    [[ -s "$scene/sleeper.pid" ]] && break
    sleep 0.1
  done
  sleeper_pid="$(cat "$scene/sleeper.pid" 2>/dev/null)"
  kill -TERM "$driver_pid"
  if wait "$driver_pid"; then signalled_exit=0; else signalled_exit=$?; fi
  if [[ -n "$sleeper_pid" ]] && peer_host_process_is_alive "$sleeper_pid"; then
    sleeper_alive=1
    kill -TERM "$sleeper_pid"  # 弄坏开关下没停掉的那一条：只停这一格自己起的那个进程号
  fi
  expect "Ⓕ 第二台那条命令在跑时收到 TERM ⇒ 退 143，第二台那条命令不在了，runs/ 下什么都不剩" \
    "$([[ "$signalled_exit" == 143 && -n "$sleeper_pid" && "$sleeper_alive" == 0 && "$(peer_runs_left signalled)" == 0 ]]; echo $?)" \
    "退 $signalled_exit，第二台那条命令的进程号「$sleeper_pid」$([[ "$sleeper_alive" == 1 ]] && echo 还活着 || echo 不在了)；runs/ 下留着 $(peer_runs_left signalled) 条；stderr 尾部：$(tail -n 4 "$scene/run.err" | tr '\n' '|')"

  # Ⓖ 输出里没有配置里的主机与目录
  local all_outputs=("$work/peer/run.out" "$work/peer/run.err" "$work/peer/seven.err" "$work/fetch/run.err" "$work/signalled/run.err")
  expect "Ⓖ 输出里没有配置里的主机名与第二台的目录，命令打的那两样换成了「第二台」「第二台的目录」" \
    "$(! grep -qE 'selftest-multi-host-peer-name|peer-home|peer-bin' "${all_outputs[@]}" && grep -qx '第二台' "$work/peer/run.out" && grep -q '^第二台的目录/runs/peer-ok$' "$work/peer/run.out"; echo $?)" \
    "有的：$(grep -lE 'selftest-multi-host-peer-name|peer-home|peer-bin' "${all_outputs[@]}" 2>/dev/null | sed "s|^$work/||" | tr '\n' ' ')；命令的输出：$(tr '\n' '|' < "$work/peer/run.out")"

  # Ⓗ 双机开关没开
  prepare_scene switched-off ""
  scene="$work/switched-off"
  run_in switched-off run --peer --memory-cap 256M -- bash -c "touch $(printf '%q' "$scene/ran")"
  expect "Ⓗ 配置里没写 ENABLE_ACROSS_MACHINES ⇒ --peer 拒（退 255，出路写 ENABLE_ACROSS_MACHINES=1），命令哪台都没跑、第二台没建目录" \
    "$([[ "$(exit_of switched-off run)" == 255 && ! -e "$scene/ran" && ! -e "$scene/peer-home/runs" ]] && grep -q 'ENABLE_ACROSS_MACHINES=1' "$scene/run.err"; echo $?)" \
    "退 $(exit_of switched-off run)，命令$([[ -e "$scene/ran" ]] && echo 跑了 || echo 没跑)；stderr 尾部：$(tail_of switched-off run)"

  if (( failures > 0 )); then
    echo "  ✗ multi-host-run.sh 自证有 $failures 格判错（共 $checked 格）"
    echo "     → 怎么办：设了 MULTI_HOST_RUN_BREAK 或 MULTI_HOST_CONFIGURATION_BREAK 的，这就是要的结果（那个开关把对应的一步弄坏了）；没设的，照上面那几格去看 research/scripts/multi-host-run.sh 与 peer-host-lib.sh"
    return 1
  fi
  echo "  ✓ multi-host-run.sh 自证 $checked 格都对（默认本机跑不读配置、--peer 跑成且退出码带回、上限不超过 PEER_MEMORY_CAP、工具链不同就拒、取回逐字相同、跑完删第二台的目录、收到 TERM 先停第二台、输出不露主机与目录、双机没开就拒）"
}

if [[ "${1:-}" == --selftest ]]; then
  multi_host_selftest
  exit $?
fi
multi_host_main "$@"
