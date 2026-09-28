#!/usr/bin/env bash
# 门禁 checker-tier-crates-mutation-replay（crates 变异表复跑）的双机分片驱动：本机没有作数记录的行按用时表分成两堆，本机与第二台各跑一趟 checker-tier-crates-mutation-replay、只判自己那一堆
# （GATE_MUTATION_ROW_SELECTION），第二台的按条记录拷回本机，经 research/scripts/crates-mutation-rows.py import 逐条核过再导入。判定不在这里：
# 驱动退出之后 checker-tier-crates-mutation-replay 在本机整张判一遍（命中按条记录的复用、缺的现跑），判定输出与单机逐字相同，全绿标记照旧由本机写。
#
#   mutation-shard-run.sh <树根>       门禁 checker-tier-crates-mutation-replay（.claude/gate.d/checker-tier-crates-mutation-replay.sh）在本地多机配置写了 ENABLE_ACROSS_MACHINES=1
#                                      （判法 layer0-shard-configuration-check.sh 退 0）时调（只由 checker-tier-crates-mutation-replay 调，输出进 checker-tier-crates-mutation-replay 的 stderr）；不认环境变量开。
#                                      退 0：两份都跑完、拷回的记录全收，或判法退 3（双机：关，报一行「双机：关」、不连第二台）；
#                                      退 1：有一步没成（没判的行留给 checker-tier-crates-mutation-replay 最后那一趟）；退 2：用法错，或在选行模式下被调
#   mutation-shard-run.sh --selftest   自证：「第二台」是本机上的另一个目录（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1），
#                                      假 cargo、真的内存包装，逐格见文件末尾 mutation_shard_selftest 的说明；格数由成功行现算
#
# 步骤：
# ① 配置照 research/scripts/layer0-shard-configuration-check.sh --emit-assignments（只调，与层 0 分片同一份）；判不过退 1，原因不在这里打；
#    判法退 3（配置没写 ENABLE_ACROSS_MACHINES=1）报一行「双机：关」、退 0，什么都不分（checker-tier-crates-mutation-replay 在本机整张判）。
# ② 两台工具链相同：rustc -Vv 前三行与 host 行、cargo -V。
# ③ 分行：crates-mutation-rows.py row-plan 列出每一行有没有作数的抓到记录与上一次现跑的用时；没有作数记录的行按用时从长到短，
#    每条放进合计用时少的那一堆（相等放本机），没记过用时的按表里记过的中位数算（一条都没记过都按 1 秒）；本机那一堆与第二台那一堆各写一份选行文件。
#    没有作数记录的行少于 2 行不分，退 0。SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1 时拒（checker-tier-crates-mutation-replay 最后那一趟不复用，分出去的白跑）。
# ④ 树拷到第二台 <PEER_REPOSITORY_DIRECTORY>/mutation-runs/<这一趟>/（新建目录，rsync 不带 --delete，排除 target、.git、规范副本与本地配置 multi-host.env），
#    规范副本另拷（树里有拷树里的，没有拷驱动所在仓的，再没有拷那份仓的主仓的），拷完回读；在那里 git init；
#    两台各算 crates-mutation-rows.py base-fingerprint，不同就拒。
# ⑤ 清场照配置做，复原挂在 EXIT 上（跑红了、收到信号也复原）；清场与复原的命令与输出都不打。
# ⑥ 两台同时跑：本机 GATE_MUTATION_ROW_SELECTION=<本机那一堆> bash <checker-tier-crates-mutation-replay> <树根>；第二台同一条只判它那一堆，脱离 ssh 会话起（setsid nohup），
#    进程号、退出码、输出各写第二台一个文件，驱动每 MUTATION_SHARD_RUN_POLL_SECONDS（默认 5）秒经 ssh 看一次；连续 60 次连不上、
#    进程没了又没留退出码、60 次都没起来，就不等了。第二台那一份整份经那棵树副本里的 run-with-memory-cap.sh 起
#    （上限 MUTATION_SHARD_PEER_MEMORY_CAP，默认 4G，键固定为「mutation-shard-run.sh 第二台那一份」：罩的是 checker-tier-crates-mutation-replay 自己在每条变异之外的进程；
#    不用配置的 PEER_MEMORY_CAP：那是层 0 一片测试二进制的上限，拿它罩整份 checker-tier-crates-mutation-replay，它在第二台的账上就一直占着这么多、每条变异排在它后面），
#    每条变异在里面照 checker-tier-crates-mutation-replay 各自经包装，上限与限时与本机同一套（行键里有它们）。
#    第二台跨趟保留的放 <PEER_REPOSITORY_DIRECTORY>/mutation-kept/：按条记录与用时表（GATE_MUTATION_RECORDS_HOME=records）、
#    编译目录（GATE_MUTATION_TARGET_DIR=target）、峰值表（RUN_WITH_MEMORY_CAP_PEAKS=memory-peaks.tsv），都不在这一趟会删的树里。
#    带到第二台的环境：SINGLEFS_HEAVY_TESTS 原样；GATE_MUTATION_MEMORY_MAX、GATE_MUTATION_TIMEOUT、GATE_MUTATION_LONG_ROW_SECONDS、SINGLEFS_REUSE_HOURS、
#    GATE_MUTATION_BREAK、RUN_WITH_MEMORY_CAP_RESERVE 设了就带；本机别的环境变量不带（真的第二台经 ssh 也拿不到）。
# ⑦ 第二台 mutation-kept/records 下这一格底座指纹的记录目录拷回本机，只留名字是本机变异表此刻某一行的记录（变异表改过之后留下的旧行的记录不导），
#    交 crates-mutation-rows.py import 逐条核底座指纹、行键与文件名，不符的由它列出。第二台那一份中途挂了、ssh 断了，已有的记录照样拷回导入，
#    没判的行留给 checker-tier-crates-mutation-replay 最后那一趟。
# ⑧ 删第二台这一趟的树与进程号、退出码、输出、选行文件（mutation-kept 下的留着）；④ 之后在哪一步失败退出，这一步都挂在 EXIT 上照做。
# 收到 TERM / INT / HUP（退 143 / 130 / 129）：先停两份——本机那一份从它的进程号起，第二台那一份从它在第二台上记下的进程号起（核它的工作目录是这一趟的树），
#   都连同全部后代逐个按进程号冻住、从最底层起 TERM、放开，至多等 30 秒，还在的 KILL，回读一个不剩——再照 EXIT 上那一套复原、删第二台这一趟的树，做完才退。
#   前台那几步（rsync、算底座指纹）里收到的，等那一步跑完再收尾；KILL 接不住，那时第二台上这一趟的树留着，手动删。
# 两份的输出存进 <树根的 git common-dir>/singlefs-mutation-shard-logs/<这一趟>/（local.log、peer.log，只留最新 3 趟）。
# 驱动打的每一行、存下的输出都不写配置里的主机、目录与清场命令：第二台上的错误输出与 peer.log 经 research/scripts/peer-host-lib.sh 的 peer_host_redact 换成「第二台」。
# 第二台共用的那几段（跑命令、拷文件、脱离会话起与轮询、按进程号停、清场复原）在 research/scripts/peer-host-lib.sh。
# 重型：真跑是两台各跑一部分 crates/mutations.tsv，与 checker-tier-crates-mutation-replay 同一类（.claude/hooks/lib_heavy_tests.py 按名字认它）；--selftest 不算重型，算「提交时才跑的检查」（同一份判定）。
#
# 弄坏开关 MUTATION_SHARD_RUN_BREAK=<项>（只给证红用，逗号分隔可给几个）：
#   import-skipped             拷回的记录不经 import、直接放进本机记录目录（自证「行键对不上的那一条被拒」两格判错）
#   no-final-pass              checker-tier-crates-mutation-replay 调完驱动不再整张判一遍（checker-tier-crates-mutation-replay 读这一项；自证「与单机逐字相同」「最后一趟全部复用」判错）
#   environment-opens          checker-tier-crates-mutation-replay 见 GATE_MUTATION_ACROSS_MACHINES=1 照旧把双机打开、不看配置的开关（checker-tier-crates-mutation-replay 读这一项；自证「配置没开双机、环境变量写 1：报双机：关、不调驱动」判错）
#   switch-off-ignored         判法退 3（双机：关）当成判不过、退 1（自证「驱动直接起、配置没开双机：报双机：关、退 0、不连第二台」判错）
#   peer-without-memory-cap    第二台那一份不经 run-with-memory-cap.sh 起（自证「第二台那一份经内存包装起」判错）
#   no-peer-cleanup-on-failure ④ 之后失败退出时不删第二台这一趟的树（自证「底座指纹不同：第二台这一趟的树删了」判错）
#   peer-kept-in-run-tree      第二台的记录、编译目录与峰值表放进这一趟的树里（自证「跨趟保留的留着」判错）
#   另有共用库的 PEER_HOST_LIB_BREAK=no-redact：第二台的输出不换私有的字（自证「输出里没有配置的主机、第二台的目录」判错）
# 只供测试的开关：SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1（照层 0 分片）；MUTATION_SHARD_RUN_PEER_STOP_AFTER_ROWS=<k>
#   给第二台那一份设 GATE_MUTATION_STOP_AFTER_ROWS=k 与 GATE_MUTATION_WORKERS=1（造「第二台那一份中途停了」：一个工作进程，判完 k 条就不再领活）。
#
# admission: always 每次调都分这一刻这棵树上没有作数记录的行；复用判定在按条记录里，不在这里
# run-condition: command python3 rsync ssh git cargo rustc
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
mutation_shard_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=peer-host-lib.sh
source "$mutation_shard_script_directory/peer-host-lib.sh"

MUTATION_SHARD_PEER_WRAPPER_KEY="mutation-shard-run.sh 第二台那一份"

mutation_shard_break_is() { [[ ",${MUTATION_SHARD_RUN_BREAK:-}," == *",$1,"* ]]; }

mutation_shard_fail() { # <原因> <出路>
  echo "  ✗ 双机分片：$1"
  echo "     → 怎么办：$2；没判的行由 checker-tier-crates-mutation-replay 最后那一趟在本机补跑"
  exit 1
}

mutation_shard_remove_peer_run() { # ⑧ 删第二台这一趟的树与这一趟的几个文件（mutation-kept 下的留着）
  local peer_run_size
  peer_run_pending=0
  peer_run_size="$(peer_host_run / "du -sh $(printf '%q' "$peer_run_directory") 2>/dev/null | cut -f1")"
  if peer_host_run / "rm -rf -- $(printf '%q' "${peer_run_directory:?}") $(printf '%q' "${peer_pid_file:?}") $(printf '%q' "${peer_exit_file:?}") $(printf '%q' "${peer_output_file:?}") $(printf '%q' "${peer_selection_file:?}")"; then
    echo "  · ⑧ 删了第二台这一趟的树（${peer_run_size:-大小没读到}）；第二台跨趟保留的记录、编译目录与峰值表留着"
  else
    echo "  ✗ 第二台这一趟的树没删掉"
    echo "     → 怎么办：下一趟另起一个目录，不碍事；到第二台的 mutation-runs/ 底下手动删 $run_label"
  fi
}

mutation_shard_on_exit() {
  trap '' TERM INT HUP PIPE
  peer_host_restore
  if (( peer_run_pending )); then
    if mutation_shard_break_is no-peer-cleanup-on-failure; then
      echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=no-peer-cleanup-on-failure：失败退出，第二台这一趟的树不删"
    else
      mutation_shard_remove_peer_run
    fi
  fi
  if [[ -n "${mutation_shard_scratch:-}" ]]; then rm -rf -- "${mutation_shard_scratch:?}"; fi
}

mutation_shard_stop_and_exit() { # <信号名> <退出码>：先停两份，再 exit（EXIT 上复原、删第二台这一趟的树）
  local stop_output
  trap '' TERM INT HUP PIPE
  echo "  ! 收到 $1：先停两份，再复原、删第二台这一趟的树，做完再退"
  if [[ -n "$local_share_process" ]]; then
    if stop_output="$(peer_host_stop_process_tree "$local_share_process")"; then
      echo "  · 本机那一份：$stop_output"
    else
      echo "  ✗ 本机那一份没停下：$stop_output"
      echo "     → 怎么办：照上面列的进程号逐个 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>"
    fi
  fi
  if (( peer_share_started )); then
    if stop_output="$(peer_host_stop_detached "$peer_run_directory" "$peer_pid_file")"; then
      echo "  · 第二台那一份：$stop_output"
    else
      echo "  ✗ 第二台那一份没停下：$stop_output"
      echo "     → 怎么办：到第二台 mutation-runs/ 底下 $run_label.pid 里的进程号，连同后代逐个 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>"
    fi
  fi
  exit "$2"
}

mutation_shard_prune_logs() { # <日志总目录>：只留最新 3 趟
  local old_directory
  while IFS= read -r old_directory; do
    [[ -n "$old_directory" ]] && rm -rf -- "${old_directory:?}"
  done < <(ls -1dt "$1"/mutation-* 2>/dev/null | tail -n +4)
}

mutation_shard_main() {
  if [[ -n "${GATE_MUTATION_ROW_SELECTION:-}" ]]; then
    echo "  ✗ 双机分片的驱动在选行模式下被调了（GATE_MUTATION_ROW_SELECTION 设着）：驱动起的那两份 checker-tier-crates-mutation-replay 不许再调驱动"
    echo "     → 怎么办：只由整道的 checker-tier-crates-mutation-replay 调驱动；选行模式下 checker-tier-crates-mutation-replay 自己不调，这里被调说明调用方写错了"
    exit 2
  fi
  tree_root="${1:-}"
  if [[ -z "$tree_root" || ! -d "$tree_root" ]]; then
    echo "  ✗ 用法：mutation-shard-run.sh <树根>，或 --selftest"
    echo "     → 怎么办：树根是 checker-tier-crates-mutation-replay 被判的那棵树（工作区或 --staged 的临时树）；只由 checker-tier-crates-mutation-replay 调"
    exit 2
  fi
  tree_root="$(cd "$tree_root" && pwd)"
  stage_repository="$(cd "$mutation_shard_script_directory/../.." && pwd)"
  stage_script="$stage_repository/.claude/gate.d/checker-tier-crates-mutation-replay.sh"
  rows_module="$stage_repository/research/scripts/crates-mutation-rows.py"
  if [[ -z "${GATE_MUTATION_MEMORY_CAP_RUNNER:-}" ]]; then GATE_MUTATION_MEMORY_CAP_RUNNER="$stage_repository/research/scripts/run-with-memory-cap.sh"; fi
  export GATE_MUTATION_MEMORY_CAP_RUNNER
  poll_seconds="${MUTATION_SHARD_RUN_POLL_SECONDS:-5}"
  peer_wrapper_cap="${MUTATION_SHARD_PEER_MEMORY_CAP:-4G}"
  [[ "$peer_wrapper_cap" =~ ^[1-9][0-9]*[KMGT]$ ]] \
    || mutation_shard_fail "MUTATION_SHARD_PEER_MEMORY_CAP 要写正整数加 K / M / G / T（单位必写），读到「$peer_wrapper_cap」" "写成例 4G，或者不设它"
  local_share_process=""
  peer_share_started=0
  peer_run_pending=0
  run_label="mutation-$$-$RANDOM$RANDOM"
  if [[ "${SINGLEFS_GATE_FULL:-}" == 1 || "${GATE_MUTATION_START_OVER:-}" == 1 ]]; then
    mutation_shard_fail "SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1：checker-tier-crates-mutation-replay 最后那一趟不复用按条记录，分出去判的行它还要再跑一遍" "这一趟单机跑（checker-tier-crates-mutation-replay 在这两个开关下不调驱动）"
  fi

  # ① 配置
  local configuration_exit=0
  peer_host_load_configuration "$mutation_shard_script_directory/layer0-shard-configuration-check.sh" "$tree_root" || configuration_exit=$?
  if (( configuration_exit == 3 )); then
    if mutation_shard_break_is switch-off-ignored; then
      echo "  ✗ 双机分片的本地配置判不过（弄坏开关 MUTATION_SHARD_RUN_BREAK=switch-off-ignored：判法退 3 当成判不过）"
      echo "     → 怎么办：这是证红用的开关，拿掉它再跑"
      exit 1
    fi
    echo "  · 双机：关（本地多机配置没写 ENABLE_ACROSS_MACHINES=1，不连第二台）；要开就在配置里写上，模板是仓根 multi-host.env.example；没判的行交给 checker-tier-crates-mutation-replay 在本机判"
    exit 0
  fi
  (( configuration_exit == 0 )) || exit 1
  if [[ "$peer_host_is_this_machine" == 1 ]]; then
    echo "  ! 第二台是本机上的另一个目录（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1）"
  fi
  mutation_shard_scratch="$(mktemp -d)"
  trap mutation_shard_on_exit EXIT
  trap 'mutation_shard_stop_and_exit TERM 143' TERM
  trap 'mutation_shard_stop_and_exit INT 130' INT
  trap 'mutation_shard_stop_and_exit HUP 129' HUP

  # ② 工具链
  peer_host_compare_toolchains || exit 1
  echo "  · ② 两台的工具链相同（${peer_host_toolchain_text}）"

  # ③ 分行
  local plan_file="$mutation_shard_scratch/plan.tsv" split_summary
  python3 "$rows_module" row-plan "$tree_root" > "$plan_file" \
    || mutation_shard_fail "本机列不出每一行的记录与用时（crates-mutation-rows.py row-plan 退非 0：$(tail -2 "$plan_file" | tr '\n' ' ')）" "单跑 python3 research/scripts/crates-mutation-rows.py row-plan <树根> 看它报什么"
  split_summary="$(python3 - "$plan_file" "$mutation_shard_scratch/local.selection" "$mutation_shard_scratch/peer.selection" "$mutation_shard_scratch/keys" <<'PY_SPLIT'
import statistics, sys
plan_path, local_path, peer_path, keys_path = sys.argv[1:]
with open(plan_path, encoding="utf-8") as handle:
    rows = [line.rstrip("\n").split("\t") for line in handle if line.strip()]
with open(keys_path, "w", encoding="utf-8") as handle:
    handle.write("".join(row[1] + "\n" for row in rows))
known = [float(row[3]) for row in rows if row[3] != "-"]
default = statistics.median(known) if known else 1.0
pending = sorted(((float(row[3]) if row[3] != "-" else default, int(row[0])) for row in rows if row[2] == "0"), key=lambda item: (-item[0], item[1]))
heaps = [[0.0, []], [0.0, []]]
for seconds, line_number in pending:
    lighter = heaps[0] if heaps[0][0] <= heaps[1][0] else heaps[1]
    lighter[0] += seconds
    lighter[1].append(line_number)
for path, (_total, lines) in zip((local_path, peer_path), heaps):
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("".join(f"{line_number}\n" for line_number in sorted(lines)))
print(len(rows), len(pending), len(known), f"{default:.1f}", len(heaps[0][1]), f"{heaps[0][0]:.0f}", len(heaps[1][1]), f"{heaps[1][0]:.0f}")
PY_SPLIT
)" || mutation_shard_fail "分行那一段 python 没跑成" "看上面 python 的报错"
  local table_rows pending_rows timed_rows default_seconds local_rows local_seconds peer_rows peer_seconds
  read -r table_rows pending_rows timed_rows default_seconds local_rows local_seconds peer_rows peer_seconds <<< "$split_summary"
  if (( pending_rows < 2 )); then
    echo "  · ③ 变异表 $table_rows 行里没有作数记录的只有 $pending_rows 行：不分，交给 checker-tier-crates-mutation-replay 在本机判"
    exit 0
  fi
  echo "  · ③ 变异表 $table_rows 行里没有作数记录的 $pending_rows 行按用时两堆均分（记过用时的 $timed_rows 行，没记过的按中位数 $default_seconds 秒算）：本机 $local_rows 行约 $local_seconds 秒，第二台 $peer_rows 行约 $peer_seconds 秒"

  # ④ 树拷到第二台、两台各算底座指纹
  local peer_runs="$PEER_REPOSITORY_DIRECTORY/mutation-runs" peer_kept sop_source local_base peer_base
  peer_run_directory="$peer_runs/$run_label"
  peer_pid_file="$peer_runs/$run_label.pid"
  peer_exit_file="$peer_runs/$run_label.exit"
  peer_output_file="$peer_runs/$run_label.log"
  peer_selection_file="$peer_runs/$run_label.selection"
  peer_kept="$PEER_REPOSITORY_DIRECTORY/mutation-kept"
  if mutation_shard_break_is peer-kept-in-run-tree; then
    peer_kept="$peer_run_directory/.mutation-kept"
    echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=peer-kept-in-run-tree：第二台的记录、编译目录与峰值表放进这一趟的树里"
  fi
  sop_source="$(peer_host_sop_source "$tree_root" "$stage_repository")" \
    || mutation_shard_fail "树里、驱动所在的仓与它的主仓里都没有规范副本 .claude/singlefs-ai-sop/（要有 scripts/preflight.py）：第二台那棵树的 checker-tier-crates-mutation-replay 一步都判不了" "在主仓里装好它（bash .claude/singlefs-ai-sop/install.sh）"
  peer_host_run / "mkdir -p $(printf '%q' "$peer_run_directory") $(printf '%q' "$peer_kept")" \
    || mutation_shard_fail "第二台建不了这一趟的目录" "看配置的 PEER_REPOSITORY_DIRECTORY 可不可写"
  peer_run_pending=1
  peer_host_copy --exclude target --exclude .git --exclude /.claude/singlefs-ai-sop --exclude /multi-host.env "$tree_root/" "$(peer_host_path "$peer_run_directory")/" \
    || mutation_shard_fail "树 rsync 到第二台失败" "看第二台盘满没满、配置的 PEER_REPOSITORY_DIRECTORY 可不可写"
  peer_host_copy "$sop_source/" "$(peer_host_path "$peer_run_directory/.claude/singlefs-ai-sop")/" \
    || mutation_shard_fail "规范副本 rsync 到第二台失败" "看第二台盘满没满"
  peer_host_run "$peer_run_directory" "test -f .claude/singlefs-ai-sop/scripts/preflight.py && git init -q ." \
    || mutation_shard_fail "规范副本拷过去之后回读不到 scripts/preflight.py，或第二台上 git init 失败" "看第二台那棵树可不可写、装没装 git"
  peer_host_copy "$mutation_shard_scratch/peer.selection" "$(peer_host_path "$peer_selection_file")" \
    || mutation_shard_fail "第二台那一堆的选行文件拷不过去" "看第二台盘满没满"
  echo "  · ④ 树拷到第二台这一趟的目录、在那里 git init（规范副本另拷）"
  local_base="$(python3 "$rows_module" base-fingerprint "$tree_root")" \
    || mutation_shard_fail "本机算不出底座指纹：$local_base" "单跑 python3 research/scripts/crates-mutation-rows.py base-fingerprint <树根> 看它报什么"
  peer_base="$(peer_host_run "$peer_run_directory" "python3 research/scripts/crates-mutation-rows.py base-fingerprint .")" \
    || mutation_shard_fail "第二台算不出底座指纹" "在第二台那棵树里单跑同一条 base-fingerprint 看它报什么"
  local_base="${local_base%% *}"
  peer_base="${peer_base%% *}"
  [[ "$local_base" == "$peer_base" ]] \
    || mutation_shard_fail "两台算出的底座指纹不同（本机 ${local_base:0:16}…，第二台 ${peer_base:0:16}…）：构建环境（~/.cargo/config*、目录层级里的 .cargo/config*、RUSTFLAGS 这一类）或树不一样" \
      "对一对两台的 cargo 配置与环境变量，改成一样再跑"
  echo "  · ④ 两台的底座指纹相同：${local_base:0:16}…"

  # ⑤ 清场
  peer_host_quiesce || exit 1

  # ⑥ 两份同时跑
  local git_common_directory logs_root log_directory local_log peer_log local_exit peer_command peer_share_text carried_variable
  git_common_directory="$(git -C "$tree_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" \
    || mutation_shard_fail "$tree_root 不是 git 工作树（取不到 git common-dir）" "checker-tier-crates-mutation-replay 被判的树都是 git 工作树；单跑时给一棵"
  logs_root="$git_common_directory/singlefs-mutation-shard-logs"
  log_directory="$logs_root/$run_label"
  mkdir -p -- "$log_directory" || mutation_shard_fail "建不了日志目录 $log_directory" "看它可不可写、盘满没满"
  mutation_shard_prune_logs "$logs_root"
  local_log="$log_directory/local.log"
  peer_log="$log_directory/peer.log"
  ( cd "$tree_root" && exec env GATE_MUTATION_ROW_SELECTION="$mutation_shard_scratch/local.selection" bash "$stage_script" "$tree_root" ) > "$local_log" 2>&1 &
  local_share_process=$!
  peer_command="env"
  if [[ -n "${SINGLEFS_HEAVY_TESTS+set}" ]]; then peer_command+=" SINGLEFS_HEAVY_TESTS=$(printf '%q' "$SINGLEFS_HEAVY_TESTS")"; fi
  for carried_variable in GATE_MUTATION_MEMORY_MAX GATE_MUTATION_TIMEOUT GATE_MUTATION_LONG_ROW_SECONDS SINGLEFS_REUSE_HOURS GATE_MUTATION_BREAK RUN_WITH_MEMORY_CAP_RESERVE; do
    if [[ -n "${!carried_variable:-}" ]]; then peer_command+=" $carried_variable=$(printf '%q' "${!carried_variable}")"; fi
  done
  if [[ -n "${MUTATION_SHARD_RUN_PEER_STOP_AFTER_ROWS:-}" ]]; then
    peer_command+=" GATE_MUTATION_WORKERS=1 GATE_MUTATION_STOP_AFTER_ROWS=$(printf '%q' "$MUTATION_SHARD_RUN_PEER_STOP_AFTER_ROWS")"
  fi
  peer_command+=" GATE_MUTATION_ROW_SELECTION=$(printf '%q' "$peer_selection_file") GATE_MUTATION_RECORDS_HOME=$(printf '%q' "$peer_kept/records")"
  peer_command+=" GATE_MUTATION_TARGET_DIR=$(printf '%q' "$peer_kept/target") RUN_WITH_MEMORY_CAP_PEAKS=$(printf '%q' "$peer_kept/memory-peaks.tsv")"
  if mutation_shard_break_is peer-without-memory-cap; then
    echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=peer-without-memory-cap：第二台那一份不经 run-with-memory-cap.sh 起"
    peer_share_text="不经内存包装"
  else
    peer_command+=" RUN_WITH_MEMORY_CAP_KEY=$(printf '%q' "$MUTATION_SHARD_PEER_WRAPPER_KEY") bash research/scripts/run-with-memory-cap.sh $(printf '%q' "$peer_wrapper_cap")"
    peer_share_text="整份经 run-with-memory-cap.sh $peer_wrapper_cap 起"
  fi
  peer_command+=" bash .claude/gate.d/checker-tier-crates-mutation-replay.sh $(printf '%q' "$peer_run_directory")"
  peer_share_started=1   # 先置上：起的那一刻收到信号，也按第二台上记下的进程号去停
  peer_host_start_detached "$peer_run_directory" "$peer_pid_file" "$peer_exit_file" "$peer_output_file" "$peer_command" \
    || mutation_shard_fail "第二台那一份起不来" "看 ssh 连不连得上、第二台那棵树在不在"
  echo "  · ⑥ 两份开跑：本机 $local_rows 行（pid $local_share_process），第二台 $peer_rows 行（脱离 ssh 会话起，${peer_share_text}，每条变异在里面各自经包装）；输出在 $log_directory/"
  if wait "$local_share_process"; then local_exit=0; else local_exit=$?; fi
  local_share_process=""
  echo "  · 本机那一份退 $local_exit（3 是选中的都抓到、1 是有红或没判完；判定以 checker-tier-crates-mutation-replay 最后那一趟为准）"
  peer_host_wait_detached "$peer_pid_file" "$peer_exit_file" "$poll_seconds"
  peer_share_started=0
  echo "  · 第二台那一份：$peer_host_detached_outcome"
  if peer_host_copy "$(peer_host_path "$peer_output_file")" "$mutation_shard_scratch/peer.raw.log" 2>/dev/null; then
    peer_host_redact < "$mutation_shard_scratch/peer.raw.log" > "$peer_log"
  else
    echo "  ! 第二台那一份的输出没拷回来"
  fi
  local problems=()
  case "$local_exit" in
    1|3) ;;
    *) problems+=("本机那一份退 $local_exit") ;;
  esac
  case "$peer_host_detached_outcome" in
    "exit 1"|"exit 3") ;;
    "exit 25"[0-4]) problems+=("第二台那一份${peer_host_detached_outcome#exit }：内存包装自己的结局（run-with-memory-cap.sh 文件头「退出码」），那一份的输出不算结果") ;;
    *) problems+=("第二台那一份 $peer_host_detached_outcome") ;;
  esac
  if [[ "$local_exit" == 1 || "$peer_host_detached_outcome" != "exit 3" ]]; then
    echo "    本机那一份的尾部（全文 $local_log）："; tail -6 "$local_log" | sed 's/^/      /'
    if [[ -f "$peer_log" ]]; then echo "    第二台那一份的尾部（全文 $peer_log）："; tail -6 "$peer_log" | sed 's/^/      /'; fi
  fi

  # ⑦ 第二台的记录拷回、核过再导入
  local peer_records="$peer_kept/records/singlefs-mutation-rows/$local_base" fetched="$mutation_shard_scratch/fetched" importable="$mutation_shard_scratch/importable"
  local fetched_count=0 importable_count=0 stale_count=0 record_file record_name import_exit=0 local_records
  mkdir -p -- "$fetched" "$importable"
  if ! peer_host_copy "$(peer_host_path "$peer_records")/" "$fetched/" 2>/dev/null; then
    echo "  ! 第二台这一格底座指纹的记录目录拷不回来（那一份一条都没判完，或 ssh 断了）"
  fi
  for record_file in "$fetched"/*; do
    [[ -f "$record_file" ]] || continue
    fetched_count=$((fetched_count + 1))
    record_name="${record_file##*/}"
    if [[ "$record_name" =~ ^(mutation|baseline)\.([0-9a-f]{64})$ ]] && grep -qxF "${BASH_REMATCH[2]}" "$mutation_shard_scratch/keys"; then
      cp -- "$record_file" "$importable/" && importable_count=$((importable_count + 1))
    else
      stale_count=$((stale_count + 1))
    fi
  done
  echo "  · ⑦ 拷回第二台的记录 $fetched_count 条：本机变异表此刻的行 $importable_count 条，别的 $stale_count 条（变异表改过之后留下的旧行，不导）"
  if mutation_shard_break_is import-skipped; then
    echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=import-skipped：拷回的记录不经 import，直接放进本机记录目录"
    if local_records="$(python3 "$rows_module" records-directory "$tree_root")" && mkdir -p -- "$local_records"; then
      cp -- "$importable"/* "$local_records/" 2>/dev/null
    fi
  elif (( importable_count > 0 )); then
    if python3 "$rows_module" import "$tree_root" "$importable"; then import_exit=0; else import_exit=$?; fi
    (( import_exit == 0 )) || problems+=("import 退 $import_exit（拒掉的逐条列在上面，那几行 checker-tier-crates-mutation-replay 最后那一趟照跑）")
  fi

  # ⑧ 删第二台这一趟的树
  mutation_shard_remove_peer_run
  if (( ${#problems[@]} > 0 )); then
    mutation_shard_fail "$(IFS='；'; echo "${problems[*]}")" "看上面两份的尾部与 $log_directory/ 里的全文"
  fi
  echo "  ✓ 双机分片：本机 $local_rows 行、第二台 $peer_rows 行都判完，第二台的记录导入 $importable_count 条；checker-tier-crates-mutation-replay 最后那一趟整张判"
}

# ── 自证 ──────────────────────────────────────────────────────────────────────
# 在临时目录里搭一棵小仓（本仓的 .claude/scripts、checker-tier-crates-mutation-replay 与登记表、research/scripts 整个目录、规范副本的符号链接，加一个 crates/tiny 与六行变异表），
# 「第二台」是本机上的另一个目录（SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1），配置写在临时目录里（SINGLEFS_MULTI_HOST_CONFIG）。
# 假 cargo（本机一份、第二台 PATH 上一份，各往自己目录下的 invocations.log 记起了哪几条）照 lib.rs 里「// TEST <名> NEEDS <原文>」判 ok / FAILED；
# 每条变异照真的走 run-with-memory-cap.sh（上限 256M，排队与账都是真的）。逐格：
#   单机那一趟：配置没写 ENABLE_ACROSS_MACHINES=1、环境里 GATE_MUTATION_ACROSS_MACHINES=1（原来的开法）：checker-tier-crates-mutation-replay 判绿、报「双机：关」、不调驱动（它的 stdout 是下面比对的基准）
#   ① 主流程（checker-tier-crates-mutation-replay 接驱动）：退 0；stdout 与单机那一趟逐字相同；两台起的变异不相交、合起来是全部、都不空；
#      最后一趟全部复用；本机那一份（选行模式的 checker-tier-crates-mutation-replay）没再调驱动；清场复原了；第二台这一趟的树删了、跨趟保留的记录与峰值表留着；
#      第二台那一份经内存包装起（跨趟保留的峰值表里有它的键）、每条变异的峰值也记在那里；输出里没有配置的主机、第二台的目录与清场命令
#   ② import 拒行键对不上的：第二台跨趟保留的记录目录里预先放一条名字是本机那一堆某行、内容行键是别的的记录：驱动列出它，本机记录目录里那一行不是它
#   ③ 第二台那一份判完 1 条就停（MUTATION_SHARD_RUN_PEER_STOP_AFTER_ROWS=1）：驱动退 1；拷回的记录照导；checker-tier-crates-mutation-replay 最后那一趟补跑缺的行，stdout 仍与单机逐字相同
#   ④ 两台底座指纹不同（第二台目录的上一层放 .cargo/config.toml）：驱动拒、两份都没起、第二台这一趟的树删了；checker-tier-crates-mutation-replay 在本机整张判，stdout 与单机逐字相同
#   ⑤ 第二台的 rustc -Vv 不同：驱动拒，第二台这一趟的目录没建
#   ⑥ 两份都在跑的时候给驱动（只给它自己，kill "$!"）发 TERM：退 143，两份卡住的假 cargo 进程都不在了，复原了，第二台这一趟的树删了
#   ⑦ 选行模式下调驱动：退 2 拒
#   ⑧ 驱动直接起、配置没写 ENABLE_ACROSS_MACHINES=1（判法退 3）：报一行「双机：关」、退 0，第二台这一趟的目录没建
# 弄坏开关各让对应的格判错（见文件头「弄坏开关」）。

mutation_shard_selftest() {
  local repository_root work template checked=0 failures=0
  repository_root="$(cd "$mutation_shard_script_directory/../.." && pwd)"
  work="$(mktemp -d)"
  mutation_shard_selftest_work="$work"
  trap 'echo "  · 自证的临时目录（$(du -sh "$mutation_shard_selftest_work" 2>/dev/null | cut -f1)）删掉"; rm -rf -- "${mutation_shard_selftest_work:?}"' EXIT
  expect() { # expect <格名> <判据的退出码：0 为过> <不过时的细节>
    checked=$((checked + 1))
    if [[ "$2" == 0 ]]; then
      echo "  ✓ $1"
    else
      echo "  ✗ $1：$3"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  }

  # 小仓的模板
  template="$work/template"
  mkdir -p "$template/.claude/gate.d" "$template/crates/tiny/src" "$template/research"
  rsync -a --exclude __pycache__ "$repository_root/.claude/scripts/" "$template/.claude/scripts/"
  cp -- "$repository_root/.claude/gate.d/checker-tier-crates-mutation-replay.sh" "$repository_root/.claude/gate.d/stage-inputs.tsv" "$template/.claude/gate.d/"
  rsync -a --exclude __pycache__ "$repository_root/.claude/gate.d/lib/" "$template/.claude/gate.d/lib/"   # checker-tier-crates-mutation-replay source 的格共用库
  rsync -a --exclude __pycache__ --exclude memory-peaks.tsv "$repository_root/research/scripts/" "$template/research/scripts/"
  ln -s "$repository_root/.claude/singlefs-ai-sop" "$template/.claude/singlefs-ai-sop"
  printf 'target/\n__pycache__/\n/multi-host.env\n/research/scripts/memory-peaks.tsv\n/.claude/singlefs-ai-sop\n' > "$template/.gitignore"
  printf '[workspace]\nmembers = ["crates/tiny"]\n' > "$template/Cargo.toml"
  printf '[package]\nname = "tiny"\nversion = "0.1.0"\n' > "$template/crates/tiny/Cargo.toml"
  cat > "$template/crates/tiny/src/lib.rs" <<'SAMPLE_SOURCE'
// 双机分片驱动自证的样本 crate（假 cargo 按下面几行判）
// TEST adds NEEDS left + right
// TEST doubles NEEDS value * 2
// TEST negates NEEDS 0 - value
// TEST halves NEEDS value / 2
// TEST triples NEEDS value * 3
// TEST squares NEEDS value * value
pub fn add(left: u32, right: u32) -> u32 { left + right }
pub fn double(value: u32) -> u32 { value * 2 }
pub fn negate(value: i32) -> i32 { 0 - value }
pub fn halve(value: u32) -> u32 { value / 2 }
pub fn triple(value: u32) -> u32 { value * 3 }
pub fn square(value: u32) -> u32 { value * value }
SAMPLE_SOURCE
  {
    printf '# 每条变异的内存上限：256M\n# 每条变异的超时秒数：120\n# 变异名\t文件\t原文\t替换文\tcargo test 参数\t必须红的测试名\n'
    printf 'add-becomes-sub\tcrates/tiny/src/lib.rs\t{ left + right }\t{ left - right }\t-p tiny --lib -- adds\tadds\n'
    printf 'double-becomes-quintuple\tcrates/tiny/src/lib.rs\t{ value * 2 }\t{ value * 5 }\t-p tiny --lib -- doubles\tdoubles\n'
    printf 'negate-becomes-identity\tcrates/tiny/src/lib.rs\t{ 0 - value }\t{ value }\t-p tiny --lib -- negates\tnegates\n'
    printf 'halve-becomes-third\tcrates/tiny/src/lib.rs\t{ value / 2 }\t{ value / 3 }\t-p tiny --lib -- halves\thalves\n'
    printf 'triple-becomes-add\tcrates/tiny/src/lib.rs\t{ value * 3 }\t{ value + 3 }\t-p tiny --lib -- triples\ttriples\n'
    printf 'square-becomes-double\tcrates/tiny/src/lib.rs\t{ value * value }\t{ value + value }\t-p tiny --lib -- squares\tsquares\n'
  } > "$template/crates/mutations.tsv"
  mkdir -p "$work/fake"
  cat > "$work/fake/cargo" <<'FAKE_CARGO'
#!/usr/bin/env python3
# 双机分片驱动自证的假 cargo：-V 打版本；test 照 crates/tiny/src/lib.rs 里「// TEST <名> NEEDS <原文>」判，原文在就 ok、不在就 FAILED；
# 每起一次往自己目录下的 invocations.log 记「<过滤词>\t<lib.rs 的 sha256 前 12 位>」。自己目录下有 hang 文件时，把进程号写进 hanging.<进程号> 再睡 300 秒
import hashlib, os, sys, time
here = os.path.dirname(os.path.abspath(__file__))
arguments = sys.argv[1:]
if arguments[:1] in (["-V"], ["--version"]):
    print("cargo 0.0.0 (mutation-shard-run selftest)")
    sys.exit(0)
if arguments[:1] != ["test"]:
    sys.exit(2)
if os.path.exists(os.path.join(here, "hang")):
    with open(os.path.join(here, f"hanging.{os.getpid()}"), "w") as handle:
        handle.write(str(os.getpid()))
    time.sleep(300)
    sys.exit(101)
source = open("crates/tiny/src/lib.rs", encoding="utf-8").read()
after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
filters = [word for word in after if not word.startswith("-")]
with open(os.path.join(here, "invocations.log"), "a", encoding="utf-8") as log:
    log.write(" ".join(filters) + "\t" + hashlib.sha256(source.encode()).hexdigest()[:12] + "\n")
failed = False
for line in source.splitlines():
    words = line.split(" ", 4)
    if len(words) < 5 or words[:2] != ["//", "TEST"]:
        continue
    name, what = words[2], words[4]
    if filters and not any(word in "tests::" + name for word in filters):
        continue
    ok = what in source.replace(line, "")
    failed = failed or not ok
    print(f"test tests::{name} ... {'ok' if ok else 'FAILED'}")
sys.exit(101 if failed else 0)
FAKE_CARGO
  chmod +x "$work/fake/cargo"
  local pristine_hash
  pristine_hash="$(sha256sum "$template/crates/tiny/src/lib.rs" | cut -c1-12)"

  # 每一格一个场景目录：repo（被判的树，git init）、local-bin 与 peer-bin（两台 PATH 上的假 cargo）、peer-home（第二台的 PEER_REPOSITORY_DIRECTORY）、配置
  prepare_scene() { # <场景名> [off]：off 时配置里写 ENABLE_ACROSS_MACHINES=0（判法退 3，双机：关）
    local scene="$work/$1" across_machines_switch=1
    [[ "${2:-}" == off ]] && across_machines_switch=0
    mkdir -p "$scene/local-bin" "$scene/peer-bin" "$scene/peer-home"
    rsync -a "$template/" "$scene/repo/"
    git -C "$scene/repo" init -q
    cp -- "$work/fake/cargo" "$scene/local-bin/cargo"
    cp -- "$work/fake/cargo" "$scene/peer-bin/cargo"
    {
      echo "PEER_SSH_HOST=selftest-peer-host-name"
      echo "PEER_REPOSITORY_DIRECTORY=$scene/peer-home"
      echo "PEER_CARGO_BIN_DIRECTORY=$scene/peer-bin"
      echo "PEER_MEMORY_CAP=1G"
      echo "ENABLE_ACROSS_MACHINES=$across_machines_switch"
      echo "QUIESCE_STOP_COMMAND=touch $scene/quiesce-marker-selftest"
      echo "QUIESCE_STOPPED_CHECK_COMMAND=test -e $scene/quiesce-marker-selftest"
      echo "QUIESCE_START_COMMAND=rm -f $scene/quiesce-marker-selftest"
      echo "QUIESCE_STARTED_CHECK_COMMAND=test ! -e $scene/quiesce-marker-selftest"
    } > "$scene/multi-host.env"
  }
  scene_command_for() { # <场景名>：scene_command 设成在这个场景的环境里起命令的 env 前缀（本机 PATH 先找假 cargo；checker-tier-crates-mutation-replay 读的变量从调用方的环境里清掉，弄坏开关留着）
    local scene="$work/$1"
    scene_command=(env -u GATE_MUTATION_ROW_SELECTION -u GATE_MUTATION_START_OVER -u SINGLEFS_GATE_FULL -u SINGLEFS_REUSE_HOURS -u GATE_MUTATION_WORKERS
      -u GATE_MUTATION_MEMORY_MAX -u GATE_MUTATION_TIMEOUT -u GATE_MUTATION_RECORDS_HOME -u GATE_MUTATION_ACROSS_MACHINES -u GATE_MUTATION_MEMORY_CAP_RUNNER
      -u MUTATION_SHARD_RUN_PEER_STOP_AFTER_ROWS -u SINGLEFS_HEAVY_TESTS -u RUN_WITH_MEMORY_CAP_RESERVE
      PATH="$scene/local-bin:$PATH" SINGLEFS_MULTI_HOST_CONFIG="$scene/multi-host.env" SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1
      MUTATION_SHARD_RUN_POLL_SECONDS=1 GATE_MUTATION_TARGET_DIR="$scene/local-target" RUN_WITH_MEMORY_CAP_PEAKS="$scene/local-peaks.tsv")
  }
  scene_environment() { # <场景名> <命令…>：在这个场景的环境里跑命令
    scene_command_for "$1"
    shift
    "${scene_command[@]}" "$@"
  }
  run_stage_in() { # <场景名> [变量=值…]：在这个场景的 repo 上跑 checker-tier-crates-mutation-replay，stdout、stderr、退出码各写场景目录一个文件
    local scene="$work/$1" exit_code
    shift
    if scene_environment "${scene##*/}" env "$@" bash "$scene/repo/.claude/gate.d/checker-tier-crates-mutation-replay.sh" "$scene/repo" > "$scene/stdout" 2> "$scene/stderr"; then
      exit_code=0
    else
      exit_code=$?
    fi
    echo "$exit_code" > "$scene/exit"
  }
  mutated_filters() { # <假 cargo 的记录>：变异那几次（lib.rs 不是原样的）的过滤词，一行一个、排好
    [[ -f "$1" ]] || return 0
    awk -F'\t' -v pristine="$pristine_hash" '$2 != pristine { print $1 }' "$1" | sort
  }
  peer_run_trees_left() { # <场景名>：第二台 mutation-runs/ 底下这一趟留下的目录数
    find "$work/$1/peer-home/mutation-runs" -mindepth 1 -maxdepth 1 -type d 2>/dev/null | wc -l
  }

  # 单机那一趟：配置没开双机（ENABLE_ACROSS_MACHINES=0），环境里照原来的开法写 GATE_MUTATION_ACROSS_MACHINES=1，作为逐字比对的基准
  prepare_scene single off
  run_stage_in single GATE_MUTATION_ACROSS_MACHINES=1
  expect "单机那一趟（配置没开双机、环境变量 GATE_MUTATION_ACROSS_MACHINES=1）判绿、报了双机：关、没调驱动" \
    "$([[ "$(cat "$work/single/exit")" == 0 ]] && grep -q '双机分片：没分（双机：关' "$work/single/stderr" && ! grep -q '双机分片：驱动退' "$work/single/stderr"; echo $?)" \
    "退 $(cat "$work/single/exit")；stderr 尾部：$(tail -3 "$work/single/stderr" | tr '\n' ' ')"

  # ① 主流程，连同 ② import 拒行键对不上的
  prepare_scene main
  local plan_line bogus_key base
  plan_line="$(scene_environment main python3 "$work/main/repo/research/scripts/crates-mutation-rows.py" row-plan "$work/main/repo" | head -1)"
  base="$(scene_environment main env GATE_MUTATION_MEMORY_CAP_RUNNER="$work/main/repo/research/scripts/run-with-memory-cap.sh" \
    python3 "$work/main/repo/research/scripts/crates-mutation-rows.py" base-fingerprint "$work/main/repo" | cut -d' ' -f1)"
  bogus_key="$(printf 'bogus' | sha256sum | cut -c1-64)"
  local planted_directory="$work/main/peer-home/mutation-kept/records/singlefs-mutation-rows/$base" planted_key
  planted_key="$(cut -f2 <<< "$plan_line")"
  mkdir -p "$planted_directory"
  printf '# 自证预先放的记录：名字是本机那一堆第一行，内容的行键是别的\nkind=mutation\nrow_key=%s\nbase_fingerprint=%s\nverdict=caught\nfinished_utc=%s\n' \
    "$bogus_key" "$base" "$(date -u +%Y-%m-%dT%H:%M:%SZ)" > "$planted_directory/mutation.$planted_key"
  run_stage_in main
  local main_stderr="$work/main/stderr" local_filters peer_filters all_filters
  local_filters="$(mutated_filters "$work/main/local-bin/invocations.log" | tr '\n' ' ')"
  peer_filters="$(mutated_filters "$work/main/peer-bin/invocations.log" | tr '\n' ' ')"
  all_filters="$(printf '%s\n' $local_filters $peer_filters | sort | tr '\n' ' ')"
  expect "① checker-tier-crates-mutation-replay 接驱动那一趟判绿，stdout 与单机那一趟逐字相同" \
    "$([[ "$(cat "$work/main/exit")" == 0 ]] && cmp -s "$work/single/stdout" "$work/main/stdout"; echo $?)" \
    "退 $(cat "$work/main/exit")；diff：$(diff "$work/single/stdout" "$work/main/stdout" | head -4 | tr '\n' ' ')；stderr 尾部：$(tail -4 "$main_stderr" | tr '\n' ' ')"
  expect "① 两台各判一份：起的变异不相交、合起来是全部 6 行、两边都不空" \
    "$([[ -n "$local_filters" && -n "$peer_filters" && "$all_filters" == "adds doubles halves negates squares triples " ]]; echo $?)" \
    "本机「$local_filters」，第二台「$peer_filters」"
  expect "① 最后那一趟全部复用（复用抓到 6 条、现跑变异至多 0 条）" \
    "$(grep -q '按条记录：复用抓到 6 条.*现跑变异至多 0 条' "$main_stderr"; echo $?)" \
    "stderr 里的按条记录行：$(grep '按条记录：' "$main_stderr" | tail -1)"
  local main_logs
  main_logs="$(find "$work/main/repo/.git/singlefs-mutation-shard-logs" -name local.log 2>/dev/null | head -1)"
  expect "① 本机那一份（选行模式的 checker-tier-crates-mutation-replay）没再调驱动" \
    "$([[ -n "$main_logs" ]] && grep -q '只判了选中的' "$main_logs" && ! grep -q '双机分片' "$main_logs"; echo $?)" \
    "本机那一份的输出：${main_logs:-没找到}"
  expect "① 照配置清了场、复原了（标记文件不在了）" \
    "$(grep -q '照配置清了场' "$main_stderr" && grep -q '照配置复原了' "$main_stderr" && [[ ! -e "$work/main/quiesce-marker-selftest" ]]; echo $?)" \
    "stderr：$(grep -E '清了场|复原' "$main_stderr" | tr '\n' ' ')"
  expect "① 第二台这一趟的树删了，跨趟保留的记录与峰值表在 mutation-kept 下留着" \
    "$([[ "$(peer_run_trees_left main)" == 0 && -s "$work/main/peer-home/mutation-kept/memory-peaks.tsv" ]] \
       && ls "$planted_directory"/mutation.* >/dev/null 2>&1 && [[ "$(ls "$planted_directory" | wc -l)" -ge 3 ]]; echo $?)" \
    "mutation-runs 底下留着 $(peer_run_trees_left main) 个目录；mutation-kept 底下：$(ls -R "$work/main/peer-home/mutation-kept" 2>/dev/null | head -8 | tr '\n' ' ')"
  expect "① 第二台那一份经 run-with-memory-cap.sh 起：跨趟保留的峰值表里有它的键" \
    "$(grep -qF "$MUTATION_SHARD_PEER_WRAPPER_KEY" "$work/main/peer-home/mutation-kept/memory-peaks.tsv" 2>/dev/null; echo $?)" \
    "跨趟保留的峰值表 $(grep -vc '^#' "$work/main/peer-home/mutation-kept/memory-peaks.tsv" 2>/dev/null || true) 行，没有键「$MUTATION_SHARD_PEER_WRAPPER_KEY」那一行"
  expect "① 第二台每条变异的峰值记在跨趟保留的峰值表里" \
    "$(grep -q 'cargo test --offline' "$work/main/peer-home/mutation-kept/memory-peaks.tsv" 2>/dev/null; echo $?)" \
    "峰值表里没有 cargo test --offline 的行"
  expect "① 输出里没有配置的主机、第二台的目录与清场命令" \
    "$(! grep -qE 'selftest-peer-host-name|peer-home|peer-bin|quiesce-marker-selftest' "$work/main/stdout" "$main_stderr" "$(dirname "$main_logs")/peer.log" 2>/dev/null; echo $?)" \
    "$(grep -lE 'selftest-peer-host-name|peer-home|peer-bin|quiesce-marker-selftest' "$work/main/stdout" "$main_stderr" "$(dirname "$main_logs")/peer.log" 2>/dev/null | sed "s|^$work/||" | tr '\n' ' ')里有"
  local local_records_directory
  local_records_directory="$(scene_environment main env GATE_MUTATION_MEMORY_CAP_RUNNER="$work/main/repo/research/scripts/run-with-memory-cap.sh" \
    python3 "$work/main/repo/research/scripts/crates-mutation-rows.py" records-directory "$work/main/repo")"
  expect "② import 拒了行键对不上的那一条、逐条列出" \
    "$(grep -q "拒了 1 条另一台的按条记录" "$main_stderr" && grep -q "mutation.$planted_key：文件名与记录里的类别、行键对不上" "$main_stderr"; echo $?)" \
    "stderr 里导入那几行：$(grep -A2 '导入：' "$main_stderr" | head -3 | tr '\n' ' ')"
  expect "② 本机记录目录里那一行的记录不是拒掉的那一条" \
    "$(! grep -q "row_key=$bogus_key" "$local_records_directory/mutation.$planted_key" 2>/dev/null; echo $?)" \
    "$local_records_directory/mutation.$planted_key 里是那条预先放的"

  # ③ 第二台那一份判完 1 条就停
  prepare_scene stopped
  run_stage_in stopped MUTATION_SHARD_RUN_PEER_STOP_AFTER_ROWS=1
  expect "③ 第二台那一份判完 1 条就停：第二台那一份退 1，checker-tier-crates-mutation-replay 最后那一趟复用 4 条、补跑缺的 2 行，stdout 仍与单机逐字相同" \
    "$([[ "$(cat "$work/stopped/exit")" == 0 ]] && cmp -s "$work/single/stdout" "$work/stopped/stdout" && grep -q '第二台那一份：exit 1' "$work/stopped/stderr" \
       && grep -q '复用抓到 4 条.*现跑变异至多 2 条' "$work/stopped/stderr"; echo $?)" \
    "退 $(cat "$work/stopped/exit")；按条记录行：$(grep '按条记录：' "$work/stopped/stderr" | tail -1)；第二台：$(grep '第二台那一份：' "$work/stopped/stderr" | tail -1)"

  # ④ 底座指纹不同
  prepare_scene diverged
  mkdir -p "$work/diverged/peer-home/.cargo"
  printf '[build]\nrustflags = ["-Cdebug-assertions"]\n' > "$work/diverged/peer-home/.cargo/config.toml"
  run_stage_in diverged
  expect "④ 两台底座指纹不同：驱动拒、两份都没起、第二台这一趟的树删了" \
    "$(grep -q '两台算出的底座指纹不同' "$work/diverged/stderr" && [[ ! -s "$work/diverged/peer-bin/invocations.log" && "$(peer_run_trees_left diverged)" == 0 ]]; echo $?)" \
    "stderr：$(grep -E '✗|删' "$work/diverged/stderr" | head -3 | tr '\n' ' ')；第二台留着 $(peer_run_trees_left diverged) 个目录"
  expect "④ 驱动拒了之后 checker-tier-crates-mutation-replay 在本机整张判，stdout 与单机逐字相同" \
    "$([[ "$(cat "$work/diverged/exit")" == 0 ]] && cmp -s "$work/single/stdout" "$work/diverged/stdout"; echo $?)" \
    "退 $(cat "$work/diverged/exit")"

  # ⑤ 工具链不同：第二台 PATH 上的 rustc 报另一个版本
  prepare_scene toolchain
  printf '#!/usr/bin/env bash\nprintf "rustc 0.0.0-other\\nbinary: rustc\\ncommit-hash: 0000\\nhost: other-host\\n"\n' > "$work/toolchain/peer-bin/rustc"
  chmod +x "$work/toolchain/peer-bin/rustc"
  local toolchain_exit
  if scene_environment toolchain bash "$work/toolchain/repo/research/scripts/mutation-shard-run.sh" "$work/toolchain/repo" > "$work/toolchain/driver.out" 2>&1; then toolchain_exit=0; else toolchain_exit=$?; fi
  expect "⑤ 两台的工具链不同：驱动拒，第二台这一趟的目录没建" \
    "$([[ "$toolchain_exit" == 1 ]] && grep -q '两台的工具链不同' "$work/toolchain/driver.out" && [[ ! -d "$work/toolchain/peer-home/mutation-runs" ]]; echo $?)" \
    "退 $toolchain_exit；输出：$(tail -3 "$work/toolchain/driver.out" | tr '\n' ' ')"

  # ⑥ 两份都在跑的时候收到 TERM
  prepare_scene signalled
  touch "$work/signalled/local-bin/hang" "$work/signalled/peer-bin/hang"
  local driver_pid signalled_exit waited hanging_pids=() hanging_file survivors=""
  # env 起的就是驱动本身（env exec 过去）：$! 是驱动的进程号，TERM 只发给它
  scene_command_for signalled
  "${scene_command[@]}" bash "$work/signalled/repo/research/scripts/mutation-shard-run.sh" "$work/signalled/repo" > "$work/signalled/driver.out" 2>&1 &
  driver_pid=$!
  for (( waited = 0; waited < 1200; waited++ )); do
    compgen -G "$work/signalled/local-bin/hanging.*" >/dev/null && compgen -G "$work/signalled/peer-bin/hanging.*" >/dev/null && break
    sleep 0.1
  done
  for hanging_file in "$work/signalled/local-bin"/hanging.* "$work/signalled/peer-bin"/hanging.*; do
    [[ -f "$hanging_file" ]] && hanging_pids+=("$(cat "$hanging_file")")
  done
  kill -TERM "$driver_pid"
  if wait "$driver_pid"; then signalled_exit=0; else signalled_exit=$?; fi
  local hanging_pid
  for hanging_pid in ${hanging_pids[@]+"${hanging_pids[@]}"}; do
    peer_host_process_is_alive "$hanging_pid" && survivors+=" $hanging_pid"
  done
  expect "⑥ 两份都在跑时收到 TERM：退 143，两份卡住的假 cargo 都不在了，复原了，第二台这一趟的树删了" \
    "$([[ "$signalled_exit" == 143 && "${#hanging_pids[@]}" -ge 2 && -z "$survivors" && ! -e "$work/signalled/quiesce-marker-selftest" && "$(peer_run_trees_left signalled)" == 0 ]]; echo $?)" \
    "退 $signalled_exit；卡住的假 cargo ${#hanging_pids[@]} 个（${hanging_pids[*]:-没有}），还活着的「${survivors}」；第二台留着 $(peer_run_trees_left signalled) 个目录；输出尾部：$(tail -4 "$work/signalled/driver.out" | tr '\n' ' ')"

  # ⑦ 选行模式下调驱动
  local selection_exit
  if scene_environment single env GATE_MUTATION_ROW_SELECTION="$work/single/stdout" bash "$work/single/repo/research/scripts/mutation-shard-run.sh" "$work/single/repo" > "$work/single/driver.out" 2>&1; then
    selection_exit=0
  else
    selection_exit=$?
  fi
  expect "⑦ 选行模式下调驱动：退 2 拒" \
    "$([[ "$selection_exit" == 2 ]] && grep -q '选行模式下被调了' "$work/single/driver.out"; echo $?)" "退 $selection_exit"

  # ⑧ 驱动直接起、配置没开双机（单机那一趟的场景）：判法退 3
  local switched_off_exit
  if scene_environment single bash "$work/single/repo/research/scripts/mutation-shard-run.sh" "$work/single/repo" > "$work/single/switched-off.out" 2>&1; then
    switched_off_exit=0
  else
    switched_off_exit=$?
  fi
  expect "⑧ 驱动直接起、配置没写 ENABLE_ACROSS_MACHINES=1：报一行双机：关、退 0、第二台这一趟的目录没建" \
    "$([[ "$switched_off_exit" == 0 ]] && [[ "$(grep -c '双机：关' "$work/single/switched-off.out")" == 1 ]] && [[ ! -d "$work/single/peer-home/mutation-runs" ]]; echo $?)" \
    "退 $switched_off_exit；输出：$(tail -3 "$work/single/switched-off.out" | tr '\n' ' ')"

  if (( failures > 0 )); then
    echo "  ✗ mutation-shard-run.sh 自证有 $failures 格判错（共 $checked 格）"
    echo "     → 怎么办：设了 MUTATION_SHARD_RUN_BREAK 的，这就是要的结果（那个开关把对应的一步弄坏了）；没设的，照上面那几格去看驱动与 research/scripts/peer-host-lib.sh"
    return 1
  fi
  echo "  ✓ mutation-shard-run.sh 自证 $checked 格都对（两台各判一份、拷回核过再导入、最后一趟全部复用且与单机逐字相同、第二台中途停了补跑、行键对不上被拒、底座指纹与工具链不同就拒、收到 TERM 先停两份、配置没开双机就报双机：关不分、环境变量开不了双机）"
}

if [[ "${1:-}" == --selftest ]]; then
  mutation_shard_selftest
  exit $?
fi
mutation_shard_main "$@"
