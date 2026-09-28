#!/usr/bin/env bash
# 层 0 崩溃重放按双机分片跑一条崩溃枚举用例（里程碑三第六项，.claude/kb/milestone/03-third-txn.md 第六节）：本机跑第 0/2 片、
# 第二台跑第 1/2 片（SINGLEFS_LAYER0_SHARD，切法与账本在 crates/singlefs-checker-tier/src/layer0_progress.rs），第二台的账本拷回本机，
# 本机 SINGLEFS_LAYER0_SHARD=merge/2 再跑同一条用例（只读两份账本、核齐、按切片序号并，不枚举）。
#
#   layer0-shard-run.sh <崩溃枚举用例的键> <树根>
#       两台各跑一片、merge，merge 那一趟的日志照门禁 54 号 --full 判（admission.py crash-case-judge），判绿写 54 号那一格全绿标记
#       （同一组标记、同一个输入指纹，crash-case-record）；判红删这批输入那一格。树根是 HEAD + 暂存区那棵树（建法同 54 号 --full）。
#   layer0-shard-run.sh --merged-log <崩溃枚举用例的键> <树根> <输入指纹> <日志文件>
#       门禁 54 号 --full 在分片开着时调：只做两片与 merge，merge 那一趟的整段输出写进日志文件，退出码是那一趟的；判与写标记归 54 号。
#   layer0-shard-run.sh --selftest
#       转给 research/scripts/layer0-shard-run-selftest.sh：不碰第二台，本机两个进程各跑一片走同一条路（含 merge、判与标记）。
#
# 步骤：① 两台各 rustc -Vv、cargo -V、nproc，rustc -Vv 前三行与 host 行、cargo -V 不同就拒；② 树 rsync 到第二台
# <PEER_REPOSITORY_DIRECTORY>/runs/<这一趟>/（新建的空目录，不带 --delete），在那里 git init（算输入指纹要 git 列文件）；规范副本
# .claude/singlefs-ai-sop/ 另拷进那棵树（它在 .gitignore 里，HEAD + 暂存区的 worktree 里没有；本机的准入模块经 git common-dir 回退到主仓找得到，
# 第二台那棵树 git init 过、没有主仓可回退）：树里有就拷树里的，没有就拷驱动所在主仓那一份（先驱动这棵树里的，再驱动这棵树的 git common-dir
# 所在的那份仓，与 .claude/scripts/preflight.sh 找它的次序相同），拷完回读 scripts/preflight.py 在不在；
# ③ 两台各算这条用例的输入指纹（admission.py crash-case-manifest，参数与 54 号相同），不同就拒；清场（配置里写了的话，回读验证，
# 复原挂在 EXIT 上，跑红了也复原）；④ 两片同时跑，各记 pid、各 wait 取退出码、各写自己的日志，LAYER0_PROGRESS 边跑边转出来；
# ⑤ 第二台的账本拷回本机进度目录；⑥ 本机 merge；⑦ 删掉第二台上这一趟的树与编译目录（两片的进度目录留着：被杀之后下一趟接着跑）。
# 第二台上这一趟的目录建好之后在哪一步失败退出，⑦ 都挂在 EXIT 上照做（删树与第二台上那一片的发现日志，进度目录留着）。
# 收到 TERM / INT / HUP（退 143 / 130 / 129）：先停两片——本机那一片从它的进程起、第二台那一片从它在第二台上记下的进程号起（第二台上
# runs/<这一趟>.shard-1-of-2.pid，核它的工作目录是这一趟的树），都按父子关系逐层冻住（STOP）、从最底层起 TERM、放开，等至多 30 秒，
# 还在的 KILL，回读一个不剩——再照 EXIT 上那一套复原、回读、删第二台这一趟的树，全部做完才退；收尾时不再理会这三个信号与 SIGPIPE
# （读输出的一方先没了，写不出去也照样往下做）。两片的进度目录留着，下一趟接着跑。前台那几步（rsync、算指纹、merge）里收到的，
# 等那一步跑完再照这一套收尾；KILL 接不住，那时第二台上这一趟的树留着，手动删。
# 进度目录：本机 <git common-dir>/singlefs-layer0-progress/<输入指纹>（与 54 号相同），第二台 <PEER_REPOSITORY_DIRECTORY>/progress/<输入指纹>。
# 线程：本机那一片取 SINGLEFS_LAYER0_THREADS（没设取本机 nproc），第二台那一片取第二台的 nproc；两台的 nproc 都清掉 OMP_NUM_THREADS、
# OMP_THREAD_LIMIT 再跑（nproc 认这两个变量，漏进来就把核数压成它们的值）。SINGLEFS_LAYER0_START_OVER=1 交给两片，
# merge 那一趟不带。只接登记了 shard=across-machines 的用例（它的枚举经 Layer0Resume::from_environment 认分片开关）。
# 内存：第二台那一片在第二台上经那棵树副本里的 research/scripts/run-with-memory-cap.sh 起，上限取配置的 PEER_MEMORY_CAP（第二台不在本机的进程树里，
# 本机包在外面的那一层罩不到它）；本机那一片与 merge 照旧，由起 54 号 / 这个驱动的那一层包装管。第二台那一片退 250–254 是包装自己的结局
# （含义见 run-with-memory-cap.sh 文件头「退出码」），那一片的输出不算结果，判红。
# 环境变量 SINGLEFS_LAYER0_PEER_MEMORY_CAP 设了就覆盖配置的 PEER_MEMORY_CAP（写法同它：正整数加 K / M / G / T），没设照配置。
# 本机环境里设了 RUN_WITH_MEMORY_CAP_RESERVE（包装的余量），同一个值带给第二台那一条 run-with-memory-cap.sh；没设就不带，第二台照包装的默认余量。
# 「第二台」是本机（只供测试）时，这两个变量不从本机环境漏给那一片（真的第二台经 ssh 也拿不到本机环境），只认驱动带过去的。
# 判与写标记：merge 那一趟的日志交 admission.py crash-case-judge 判，判绿交 crash-case-record 写；核数与线程数由它们自己现取，这里不转。
# 登记的崩溃枚举用例都标了 #[ignore]，三趟 cargo test 都带 --include-ignored --exact <用例函数>；只供测试的开关
# SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0 时不带 --include-ignored（--selftest 那条小流用例不标 ignore，不必把别的 ignore 用例放进过滤范围）。
#
# 配置（本地、git 忽略）：${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，键与写法见仓根 multi-host.env.example；
# 在不在、齐不齐、第二台连不连得上由 research/scripts/layer0-shard-configuration-check.sh 判（门禁 54 号拿同一份判走不走分片）。
# 这一份与配置判法按内容进登记了 shard=across-machines 的用例的输入指纹（research/scripts/admission.py 的 SHARD_DRIVER_FILES）：两片与 merge 的
# cargo 由这里起、不经 admission.py crash-case-command，改了这里那几条用例的旧全绿标记不再作数。
# 重型：跑的是标了 ignore 的崩溃枚举用例（--include-ignored），只在提交时（SINGLEFS_HEAVY_TESTS=commit）或用户要求时跑；--selftest 不算。
#
# 日志与发现日志（用户 2026-09-27 定全量与发现双份，records/2026-09-24-里程碑二收尾调度.md「层 0 放量的发现日志」那一行；行格式以 crates/singlefs-checker-tier/src/crash.rs 为准）：
# 三趟 cargo 各设 SINGLEFS_LAYER0_FINDINGS_FILE，发现日志一律是那一趟的日志同名加 .findings.tsv，开跑前先删掉旧的。--merged-log 时两片的日志放在
# 给的日志文件旁边（<日志文件>.shard-0-of-2.log、.shard-1-of-2.log），merge 那一趟的发现日志是 <日志文件>.findings.tsv——门禁 54 号读的就是这一份；
# 单独跑时三份日志放 <git common-dir>/singlefs-layer0-logs/<这一趟>/。第二台那一片的发现日志先写在第二台 runs/<这一趟>.shard-1-of-2.findings.tsv，
# 两片跑完拷回本机、放在那一片的日志旁边，再删第二台上那一份。三份路径都打进输出；发现表归 54 号读、归 54 号判，这里不判。
# 弄坏开关（只给证红用）LAYER0_SHARD_RUN_BREAK=<项>：no-peer-findings-copy 不拷回第二台那一片的发现日志，no-merge-findings 不给 merge 那一趟设发现日志；
# 各自打开时 --selftest 的 ⑦ 那一格判错。peer-without-memory-cap 第二台那一片不经 run-with-memory-cap.sh 起，打开时 --selftest 的 ⑧ 那一格判错。
# no-sop-copy 不往第二台那棵树拷规范副本，打开时 --selftest 的 ⑨ 那一格判错；no-peer-cleanup-on-failure 失败退出时不删第二台上这一趟的树，
# 打开时 --selftest 的 ⑩ 那一格判错。
#
# 配置在不在、判不判得过不写成运行条件（--selftest 不要配置）：开跑之后第一件事调配置判法，判不过照它的原因与出路退 1。
#
# admission: always 每次调都跑这一刻这棵树上的一条崩溃枚举用例；复用判定在门禁 54 号的全绿标记里，不在这里
# run-condition: command cargo rustc python3 rsync ssh nproc git
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
layer0_shard_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
[[ "${1:-}" != --selftest ]] || exec bash "$layer0_shard_script_directory/layer0-shard-run-selftest.sh"

layer0_shard_mode=standalone
if [[ "${1:-}" == --merged-log ]]; then layer0_shard_mode=merged-log; shift; fi
case_key="${1:-}"
tree_root="${2:-}"
given_fingerprint="${3:-}"
merged_log_file="${4:-}"
usage_ok=1
[[ "$case_key" == crash-case:* && -n "$tree_root" && -d "$tree_root" ]] || usage_ok=0
if [[ "$layer0_shard_mode" == merged-log && ( ! "$given_fingerprint" =~ ^[0-9a-f]{64}$ || -z "$merged_log_file" ) ]]; then usage_ok=0; fi
if (( ! usage_ok )); then
  echo "  ✗ 用法：layer0-shard-run.sh <crash-case:键> <树根>，或 --merged-log <crash-case:键> <树根> <输入指纹> <日志文件>，或 --selftest"
  echo "     → 怎么办：键照 .claude/gate.d/stage-inputs.tsv 里 crash-case: 那一行写；树根是 HEAD + 暂存区那棵树（建法见门禁 54 号 --full 的出路句）"
  exit 2
fi
tree_root="$(cd "$tree_root" && pwd)"
admission_module="$tree_root/research/scripts/admission.py"

fail() { # fail <原因> <出路>
  echo "  ✗ $case_key 双机分片：$1"
  echo "     → 怎么办：$2"
  exit 1
}
fail_after_the_run() { # fail_after_the_run <原因> <出路>：跑过之后判红——单独跑时删这批输入那一格全绿标记（先绿后红，前一趟那一格不再作数），再 fail
  local marker_path
  if [[ "$layer0_shard_mode" == standalone ]] \
      && marker_path="$(python3 "$admission_module" crash-case-marker-path "$tree_root" "$case_key" "$local_fingerprint")" && [[ -n "$marker_path" ]]; then
    rm -f -- "${marker_path:?}"
  fi
  fail "$1" "$2"
}

if ! configuration_assignments="$(bash "$layer0_shard_script_directory/layer0-shard-configuration-check.sh" --emit-assignments "$tree_root")"; then
  printf '%s\n' "$configuration_assignments"
  exit 1
fi
eval "$configuration_assignments"
peer_is_this_machine="${SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE:-0}"
# 第二台那一片的内存上限：SINGLEFS_LAYER0_PEER_MEMORY_CAP 设了就覆盖配置的 PEER_MEMORY_CAP
peer_memory_cap="$PEER_MEMORY_CAP"
if [[ -n "${SINGLEFS_LAYER0_PEER_MEMORY_CAP:-}" ]]; then
  [[ "$SINGLEFS_LAYER0_PEER_MEMORY_CAP" =~ ^[1-9][0-9]*[KMGT]$ ]] \
    || fail "SINGLEFS_LAYER0_PEER_MEMORY_CAP 要写第二台那一片的内存上限（正整数加 K / M / G / T，单位必写），读到「${SINGLEFS_LAYER0_PEER_MEMORY_CAP}」" \
      "写成例 24G；不设它就照配置的 PEER_MEMORY_CAP"
  peer_memory_cap="$SINGLEFS_LAYER0_PEER_MEMORY_CAP"
fi

# 这条用例登记的包、测试目标、用例函数（第三列的 test=）；要登记了 shard=across-machines
registration_row="$(awk -F'\t' -v key="$case_key" '$1 == key' "$tree_root/.claude/gate.d/stage-inputs.tsv")"
test_token="$(tr ' ' '\n' <<< "$(cut -f3 <<< "$registration_row")" | grep '^test=' | head -1)"
IFS=: read -r case_package case_target case_function <<< "${test_token#test=}"
[[ -n "$registration_row" && -n "$case_package" && -n "$case_target" && -n "$case_function" ]] \
  || fail "stage-inputs.tsv 里找不到这一行或它的 test=<包>:<测试目标>:<用例函数>" "照 research/scripts/admission.py 文件头「崩溃枚举用例行」登记"
python3 "$admission_module" crash-case-shardable "$tree_root" "$case_key" >/dev/null \
  || fail "这条用例没登记 shard=across-machines（它的枚举不认分片开关，分了片也是每台各跑全量）" "单机跑它：bash .claude/gate.d/54-layer0-replay.sh --full <树根>；要分片先让用例经 Layer0Resume::from_environment 取续跑设置，再在登记行第三列加 shard=across-machines"

if ! git_common_directory="$(git -C "$tree_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$git_common_directory" ]]; then
  fail "$tree_root 不是 git 工作树（取不到 git common-dir）" "在 HEAD + 暂存区的 worktree 里跑，建法见门禁 54 号 --full 的出路句"
fi
run_label="${case_key#crash-case:}-$(date -u +%Y%m%dT%H%M%SZ)-$$"
peer_run_directory="$PEER_REPOSITORY_DIRECTORY/runs/$run_label"
peer_findings_file="$PEER_REPOSITORY_DIRECTORY/runs/$run_label.shard-1-of-2.findings.tsv"
peer_pid_file="$PEER_REPOSITORY_DIRECTORY/runs/$run_label.shard-1-of-2.pid"  # 第二台那一片的进程号（收到 TERM 时凭它停那一片）
peer_exit_file="$PEER_REPOSITORY_DIRECTORY/runs/$run_label.shard-1-of-2.exit"  # 第二台那一片跑完写它的退出码（驱动凭它判那一片跑完没有、怎么收的）
peer_output_file="$PEER_REPOSITORY_DIRECTORY/runs/$run_label.shard-1-of-2.log"  # 第二台那一片的输出：写在第二台的文件里，不经 ssh 的管道
layer0_shard_break="${LAYER0_SHARD_RUN_BREAK:-}"
# 三趟的日志放哪（跑完不删）：--merged-log 时在给的日志文件旁边，单独跑时在 common-dir 下这一趟的目录里
if [[ "$layer0_shard_mode" == merged-log ]]; then
  log_prefix="$merged_log_file"
  merge_findings_file="$merged_log_file.findings.tsv"
else
  log_prefix="$git_common_directory/singlefs-layer0-logs/$run_label/${case_key#crash-case:}"
  merge_findings_file="$log_prefix.merge.log.findings.tsv"
fi
mkdir -p -- "$(dirname "$log_prefix")" || fail "建不了日志目录 $(dirname "$log_prefix")" "看它可不可写、盘满没满"
scratch_directory="$(mktemp -d)"
quiesced=0
peer_run_directory_pending=0
local_shard_process=""
peer_shard_started=0
restore_on_exit() {
  trap '' TERM INT HUP PIPE
  if (( quiesced )); then
    echo "  · 复原：$QUIESCE_START_COMMAND"
    bash -c "$QUIESCE_START_COMMAND"
    if bash -c "$QUIESCE_STARTED_CHECK_COMMAND"; then
      echo "  · 复原之后回读过了（$QUIESCE_STARTED_CHECK_COMMAND 退 0）"
    else
      echo "  ✗ 复原之后回读没过：$QUIESCE_STARTED_CHECK_COMMAND 退非 0"
      echo "     → 怎么办：手动复原（$QUIESCE_START_COMMAND），回读到它退 0 为止"
    fi
  fi
  # 第二台上这一趟的目录建好之后失败退出：照 ⑦ 删掉（进度目录留着）；弄坏开关 no-peer-cleanup-on-failure 下不删
  if (( peer_run_directory_pending )); then
    if [[ "$layer0_shard_break" == no-peer-cleanup-on-failure ]]; then
      echo "  ! 弄坏开关 LAYER0_SHARD_RUN_BREAK=no-peer-cleanup-on-failure：失败退出，第二台上的 $peer_run_directory 不删"
    else
      remove_peer_run_directory
    fi
  fi
  rm -rf -- "${scratch_directory:?}"
}
trap restore_on_exit EXIT

process_is_alive() { # process_is_alive <进程号>：还在、且不是僵尸（僵尸算停了）
  [[ -d "/proc/$1" ]] && [[ "$(awk '{print $3}' "/proc/$1/stat" 2>/dev/null)" != Z ]]
}
stop_one_process_by_pid() { # stop_one_process_by_pid <进程号>：只停这一个点名的进程——一条 TERM，至多等 30 秒（等的时候只看 /proc，不发信号），
  # 还在就一条 KILL；回读停了退 0。每一片只有测试二进制这一个进程（先编后跑、exec），停它一个就够。本机与第二台都用这一份（declare -f 带过去）
  local waited
  kill -TERM "$1" 2>/dev/null || { echo "进程 $1 已经不在"; return 0; }
  for ((waited = 0; waited < 300; waited++)); do process_is_alive "$1" || break; sleep 0.1; done
  if process_is_alive "$1"; then
    kill -KILL "$1" 2>/dev/null
    sleep 0.5
  fi
  if process_is_alive "$1"; then echo "进程 $1 发了 TERM、KILL 还在"; return 1; fi
  echo "进程 $1 停了"
}
stop_shards_and_exit() { # stop_shards_and_exit <信号名> <退出码>：先停两片，再 exit（EXIT 上复原、回读、删第二台这一趟的树）
  local stop_output
  trap '' TERM INT HUP PIPE
  echo "  ! 收到 $1：先停两片，再复原、回读、删第二台这一趟的树（两片的进度目录留着），做完再退"
  if [[ -n "$local_shard_process" ]]; then
    if stop_output="$(stop_one_process_by_pid "$local_shard_process")"; then
      echo "  · 本机那一片（测试二进制）：$stop_output"
    else
      echo "  ✗ 本机那一片没停下：$stop_output"
      echo "     → 怎么办：python3 .claude/singlefs-ai-sop/scripts/proc.py stop $local_shard_process"
    fi
  fi
  if (( peer_shard_started )); then
    # 进程号文件要等第二台那一片起来才有：至多等 10 秒；核它的工作目录是这一趟的包目录（进程号没被别的进程重用）
    if stop_output="$(run_on_peer "$peer_run_directory" "$(declare -f process_is_alive stop_one_process_by_pid)
for ((pid_rounds = 0; pid_rounds < 100; pid_rounds++)); do [[ -s $(printf '%q' "$peer_pid_file") ]] && break; sleep 0.1; done
peer_shard_pid=\"\$(cat $(printf '%q' "$peer_pid_file") 2>/dev/null)\"
if [[ -z \"\$peer_shard_pid\" ]]; then echo '没有进程号文件：那一片没起来'
elif [[ \"\$(readlink /proc/\$peer_shard_pid/cwd 2>/dev/null)\" != \"\$(pwd -P)\"* ]]; then echo \"进程号 \$peer_shard_pid 已经不在（或不是这一趟的）\"
else stop_one_process_by_pid \"\$peer_shard_pid\"; fi")"; then
      echo "  · 第二台那一片（测试二进制）：$stop_output"
    else
      echo "  ✗ 第二台那一片没停下：$stop_output"
      echo "     → 怎么办：ssh 到第二台，照 $peer_pid_file 里的进程号 python3 .claude/singlefs-ai-sop/scripts/proc.py stop <进程号>"
    fi
  fi
  exit "$2"
}
trap 'stop_shards_and_exit TERM 143' TERM
trap 'stop_shards_and_exit INT 130' INT
trap 'stop_shards_and_exit HUP 129' HUP

# run_on_peer <目录> <命令文本>：在第二台的这个目录里 bash -c 跑（PATH 前面加 PEER_CARGO_BIN_DIRECTORY）；第二台是本机（只供测试）时本机跑
run_on_peer() {
  if [[ "$peer_is_this_machine" == 1 ]]; then
    # 和 ssh 一样等标准输出关上才返回（经 cat 转出）：留在后台、还握着输出的进程会让它一直等，自证才分得出这种写法
    (cd "$1" && env -u RUN_WITH_MEMORY_CAP_RESERVE -u SINGLEFS_LAYER0_PEER_MEMORY_CAP PATH="$PEER_CARGO_BIN_DIRECTORY:$PATH" bash -c "$2") | cat
    return "${PIPESTATUS[0]}"
  else
    ssh -o BatchMode=yes "$PEER_SSH_HOST" "cd $(printf '%q' "$1") && PATH=$(printf '%q' "$PEER_CARGO_BIN_DIRECTORY"):\$PATH bash -c $(printf '%q' "$2")" </dev/null
  fi
}
# copy_to_peer / copy_from_peer：rsync -a，不带 --delete
copy_to_peer() {
  if [[ "$peer_is_this_machine" == 1 ]]; then rsync -a "$@"; else rsync -a -e "ssh -o BatchMode=yes" "$@"; fi
}
peer_path() { # peer_path <第二台上的路径>：rsync 的参数写法
  if [[ "$peer_is_this_machine" == 1 ]]; then printf '%s' "$1"; else printf '%s:%s' "$PEER_SSH_HOST" "$1"; fi
}
remove_peer_run_directory() { # ⑦ 删掉第二台上这一趟的树与编译目录，连同第二台上那一片的发现日志（已拷回）；两片跑完、或失败退出时（EXIT 上）调
  local peer_run_size
  peer_run_directory_pending=0
  peer_run_size="$(run_on_peer / "du -sh $(printf '%q' "$peer_run_directory") | cut -f1")"
  if run_on_peer / "rm -rf -- $(printf '%q' "${peer_run_directory:?}") $(printf '%q' "${peer_findings_file:?}") $(printf '%q' "${peer_pid_file:?}") $(printf '%q' "${peer_exit_file:?}") $(printf '%q' "${peer_output_file:?}")"; then
    echo "  · ⑦ 删掉第二台上这一趟的树与编译目录 $peer_run_directory（${peer_run_size:-大小没读到}）；两片的进度目录留着"
  else
    echo "  ! 第二台上的 $peer_run_directory 没删掉：下一趟另起一个目录，不碍事；手动删它"
  fi
}
if [[ "$peer_is_this_machine" == 1 ]]; then
  echo "  ! 第二台是本机上的 $PEER_REPOSITORY_DIRECTORY（只供测试的开关 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1）"
fi

# ① 工具链：rustc -Vv 前三行与 host 行、cargo -V 两台相同；线程数各取各的核数
toolchain_description_command='rustc -Vv | head -3 && rustc -Vv | grep "^host: " && cargo -V'
local_toolchain="$(bash -c "$toolchain_description_command")" || fail "本机 rustc / cargo 报不出版本" "看 PATH 里有没有 rustc 与 cargo"
peer_toolchain="$(run_on_peer / "$toolchain_description_command")" || fail "第二台 rustc / cargo 报不出版本" "看 $PEER_CARGO_BIN_DIRECTORY 下有没有 rustc 与 cargo"
[[ "$local_toolchain" == "$peer_toolchain" ]] \
  || fail "两台的工具链不同：本机「${local_toolchain//$'\n'/；}」，第二台「${peer_toolchain//$'\n'/；}」" "两台装同一个 rustup 工具链（rustc -Vv 的 commit 相同）再跑"
local_cores="$(env -u OMP_NUM_THREADS -u OMP_THREAD_LIMIT nproc)"
peer_cores="$(run_on_peer / "env -u OMP_NUM_THREADS -u OMP_THREAD_LIMIT nproc")" || fail "第二台 nproc 跑不起来" "看 ssh $PEER_SSH_HOST 能不能跑命令"
if [[ -n "${SINGLEFS_LAYER0_THREADS:-}" ]]; then
  local_threads="$SINGLEFS_LAYER0_THREADS"; local_threads_text="SINGLEFS_LAYER0_THREADS=${local_threads}（显式设的）"
else
  local_threads="$local_cores"; local_threads_text="SINGLEFS_LAYER0_THREADS=${local_threads}（没设，取本机核数）"
fi
echo "  · ① 工具链两台相同（${local_toolchain%%$'\n'*}）；本机 ${local_cores} 核跑 0/2（${local_threads_text}），第二台 ${peer_cores} 核跑 1/2（取第二台核数）"

# ② 要一起拷过去的规范副本：树里有就拷树里的，没有就取驱动所在主仓那一份（先驱动这棵树里的，再驱动这棵树的 git common-dir 所在的那份仓）
if [[ -f "$tree_root/.claude/singlefs-ai-sop/scripts/preflight.py" ]]; then
  sop_copy_source="$tree_root/.claude/singlefs-ai-sop"; sop_copy_origin="树里的"
else
  driver_tree_root="$(cd "$layer0_shard_script_directory/../.." && pwd)"
  sop_copy_candidates=("$driver_tree_root/.claude/singlefs-ai-sop")
  if driver_common_directory="$(git -C "$driver_tree_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$driver_common_directory" ]]; then
    sop_copy_candidates+=("${driver_common_directory%/*}/.claude/singlefs-ai-sop")
  fi
  sop_copy_source=""
  if [[ -f "${sop_copy_candidates[0]}/scripts/preflight.py" ]]; then
    sop_copy_source="${sop_copy_candidates[0]}"
  elif (( ${#sop_copy_candidates[@]} > 1 )) && [[ -f "${sop_copy_candidates[1]}/scripts/preflight.py" ]]; then
    sop_copy_source="${sop_copy_candidates[1]}"
  fi
  [[ -n "$sop_copy_source" ]] || fail "树里与驱动所在主仓都没有规范副本（找过 $tree_root/.claude/singlefs-ai-sop ${sop_copy_candidates[*]}，要有 scripts/preflight.py）：第二台那棵树的准入模块一步都判不了" \
    "在主仓里装好它（bash .claude/singlefs-ai-sop/install.sh），再在主仓的 worktree 里跑"
  sop_copy_origin="树里没有，取驱动所在主仓那一份"
fi

# ② 树拷到第二台这一趟的专用目录（新建的空目录，不带 --delete），在那里 git init；规范副本另拷（树的那一趟不带它：树里那一份可能是指回本机的符号链接）
run_on_peer / "mkdir -p $(printf '%q' "$peer_run_directory") $(printf '%q' "$PEER_REPOSITORY_DIRECTORY/progress")" || fail "第二台建不了 $peer_run_directory" "看 PEER_REPOSITORY_DIRECTORY 可不可写"
peer_run_directory_pending=1
copy_to_peer --exclude target --exclude .git --exclude /.claude/singlefs-ai-sop "$tree_root/" "$(peer_path "$peer_run_directory")/" || fail "树 rsync 到第二台失败" "看第二台盘满没满、$PEER_REPOSITORY_DIRECTORY 可不可写"
if [[ "$layer0_shard_break" == no-sop-copy ]]; then
  echo "  ! 弄坏开关 LAYER0_SHARD_RUN_BREAK=no-sop-copy：规范副本不拷到第二台"
  sop_copy_text="规范副本没拷（弄坏开关 no-sop-copy）"
else
  sop_copy_text="规范副本 $sop_copy_source 拷到第二台（${sop_copy_origin}）"
  copy_to_peer "$sop_copy_source/" "$(peer_path "$peer_run_directory/.claude/singlefs-ai-sop")/" \
    || fail "规范副本 $sop_copy_source rsync 到第二台失败" "看第二台盘满没满、$PEER_REPOSITORY_DIRECTORY 可不可写"
  run_on_peer "$peer_run_directory" "test -f .claude/singlefs-ai-sop/scripts/preflight.py" \
    || fail "规范副本拷过去之后回读，第二台 $peer_run_directory/.claude/singlefs-ai-sop/scripts/preflight.py 不在" "看 $sop_copy_source 里有没有 scripts/preflight.py、第二台那棵树可不可写"
fi
run_on_peer "$peer_run_directory" "git init -q ." || fail "第二台上 git init 失败" "看第二台装没装 git"
echo "  · ② 树拷到第二台 $peer_run_directory；$sop_copy_text"

# ③ 两台各算这条用例的输入指纹（参数与门禁 54 号相同：准入模块的判法摘要、工具链与构建环境；登记了 shard=across-machines 的，
# 这一份驱动脚本与配置判法由准入模块按内容加进清单）
local_manifest="$scratch_directory/manifest.local"
local_summary="$(python3 "$admission_module" crash-case-manifest "$tree_root" "$case_key" "$local_manifest" \
  --judging-digest --toolchain --build-environment)" || fail "本机算不出输入指纹：$local_summary" "单跑 python3 research/scripts/admission.py crash-case-manifest <树根> $case_key <清单文件> --judging-digest --toolchain --build-environment 看它报什么"
read -r local_fingerprint local_file_count local_excluded_count <<< "$local_summary"
peer_summary="$(run_on_peer "$peer_run_directory" "python3 research/scripts/admission.py crash-case-manifest . $(printf '%q' "$case_key") .layer0-shard-manifest \
  --judging-digest --toolchain --build-environment")" || fail "第二台算不出输入指纹：$peer_summary" "在第二台 $peer_run_directory 里单跑同一条 crash-case-manifest 看它报什么"
peer_fingerprint="${peer_summary%% *}"
[[ "$local_fingerprint" == "$peer_fingerprint" ]] \
  || fail "两台算出的输入指纹不同（本机 ${local_fingerprint:0:16}…，第二台 ${peer_fingerprint:0:16}…）：构建环境（~/.cargo/config、RUSTFLAGS 这一类）或树不一样" "对一对两台的 ~/.cargo/config*、环境变量 RUSTFLAGS / CARGO_*，改成一样再跑"
if [[ "$layer0_shard_mode" == merged-log && "$local_fingerprint" != "$given_fingerprint" ]]; then
  fail "本机算出的输入指纹 ${local_fingerprint:0:16}… 与 54 号给的 ${given_fingerprint:0:16}… 不同" "别在有人改这些路径的树里跑；在 HEAD + 暂存区的 worktree 里重跑 54 号 --full"
fi
echo "  · ③ 两台的输入指纹相同：${local_fingerprint:0:16}…（${local_file_count} 个文件，减去用例读不到的 ${local_excluded_count} 个）"

local_progress_directory="$git_common_directory/singlefs-layer0-progress/$local_fingerprint"
peer_progress_directory="$PEER_REPOSITORY_DIRECTORY/progress/$local_fingerprint"
mkdir -p "$local_progress_directory"
run_on_peer / "mkdir -p $(printf '%q' "$peer_progress_directory")" || fail "第二台建不了进度目录 $peer_progress_directory" "看 PEER_REPOSITORY_DIRECTORY 可不可写"

# 清场（配置里写了才做）：跑完回读，复原挂在 EXIT 上
if [[ -n "$QUIESCE_STOP_COMMAND" ]]; then
  echo "  · 清场：$QUIESCE_STOP_COMMAND"
  quiesced=1
  bash -c "$QUIESCE_STOP_COMMAND"
  bash -c "$QUIESCE_STOPPED_CHECK_COMMAND" || fail "清场之后回读没过：$QUIESCE_STOPPED_CHECK_COMMAND 退非 0" "手动清场，回读到它退 0 为止；复原命令在退出时照跑"
  echo "  · 清场之后回读过了（$QUIESCE_STOPPED_CHECK_COMMAND 退 0）"
fi

# ④ 两片同时跑
start_over_settings=(-u SINGLEFS_LAYER0_START_OVER)
start_over_text=""
if [[ "${SINGLEFS_LAYER0_START_OVER:-0}" == 1 ]]; then start_over_settings=(SINGLEFS_LAYER0_START_OVER=1); start_over_text="SINGLEFS_LAYER0_START_OVER=1 "; fi
libtest_arguments=(--exact "$case_function" --nocapture)
# 第二台那一片经那棵树副本里的内存包装起（上限 PEER_MEMORY_CAP）；弄坏开关 peer-without-memory-cap 下不经它
peer_memory_reserve_text=""
if [[ -n "${RUN_WITH_MEMORY_CAP_RESERVE:-}" ]]; then peer_memory_reserve_text="RUN_WITH_MEMORY_CAP_RESERVE=$(printf '%q' "$RUN_WITH_MEMORY_CAP_RESERVE") "; fi
peer_memory_cap_text="${peer_memory_reserve_text}bash research/scripts/run-with-memory-cap.sh $(printf '%q' "$peer_memory_cap") "
if [[ "$layer0_shard_break" == peer-without-memory-cap ]]; then
  peer_memory_cap_text=""
  echo "  ! 弄坏开关 LAYER0_SHARD_RUN_BREAK=peer-without-memory-cap：第二台那一片不经 run-with-memory-cap.sh 起"
fi
case "${SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED:-1}" in
  1) libtest_arguments=(--include-ignored "${libtest_arguments[@]}") ;;
  0) echo "  ! 不带 --include-ignored（只供测试的开关 SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED=0；标了 ignore 的用例这样跑一条都跑不到，会判红）" ;;
  *) fail "SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED 只许 0 或 1，读到「${SINGLEFS_LAYER0_SHARD_INCLUDE_IGNORED}」" "不设它（默认 1）；只有 --selftest 设成 0" ;;
esac
cargo_test_text="cargo test --release -p $case_package --test $case_target -- ${libtest_arguments[*]}"
local_log="$log_prefix.shard-0-of-2.log"
peer_log="$log_prefix.shard-1-of-2.log"
local_findings_file="$local_log.findings.tsv"
fetched_peer_findings_file="$peer_log.findings.tsv"
rm -f -- "$scratch_directory"/exit.* "$local_log" "$peer_log" "$local_findings_file" "$fetched_peer_findings_file" "$merge_findings_file"
run_on_peer / "rm -f -- $(printf '%q' "$peer_findings_file")" || fail "第二台删不了旧的发现日志 $peer_findings_file" "看 PEER_REPOSITORY_DIRECTORY 可不可写"
echo "  · 发现日志：本机 0/2 $local_findings_file；第二台 1/2 写在第二台 $peer_findings_file、跑完拷回 $fetched_peer_findings_file；merge $merge_findings_file"
# 每一片先编出测试二进制（cargo test --no-run），再直接执行它：一片只有测试二进制这一个进程，收到 TERM 时凭进程号停它一个。
# 输出直接写进文件、不经管道：读输出的一方（显示用的 tail、第二台的 ssh）断了，测试进程照样写文件，不会因写断掉的管道 panic——
# panic 展开时进度文件的守卫会把进度文件删掉，续跑就读回 0 片（2026-09-28 预演：第二台那一片 ssh 一断就这样丢了 1920 片）。
# 第二台那一片在第二台上脱离 ssh 会话起（setsid nohup），进程号、退出码、输出各写一个文件，驱动每 5 秒经 ssh 看一次。
# 每一片起跑前在自己的进度目录拿一把 flock（锁跟着 exec 留在测试进程手里）：上一趟那一片还活着就拿不到，这一片当场退 75。
# 直接执行测试二进制时补上 cargo test 会给的两样：工作目录是包的目录；环境变量 CARGO（账本头记工具链要它）取工具链里真正的 cargo。
test_binary_from_cargo_messages() { # 读 cargo --message-format=json 的标准输出，打两行：<测试目标> 那一个测试可执行文件、它的包目录
  python3 -c '
import json, os, sys
target = sys.argv[1]; found = []
for line in sys.stdin:
    try:
        message = json.loads(line)
    except ValueError:
        continue
    artifact_target = message.get("target") or {}
    if message.get("reason") == "compiler-artifact" and artifact_target.get("name") == target and "test" in artifact_target.get("kind", []) and message.get("executable"):
        found.append((message["executable"], os.path.dirname(message.get("manifest_path", ""))))
if len(found) != 1:
    sys.exit(f"cargo 交出的测试目标 {target} 的可执行文件不是恰好一个：{found}")
print(found[0][0]); print(found[0][1])' "$1"
}
cargo_executable_text='$(rustup which cargo 2>/dev/null || command -v cargo)'
local_build_messages="$scratch_directory/build.local.json"
( cd "$tree_root" && cargo test --release -p "$case_package" --test "$case_target" --no-run --message-format=json ) > "$local_build_messages" 2> "$local_log.build" \
  || fail "本机编测试二进制失败（cargo test --no-run）" "看 $local_log.build"
mapfile -t local_test_binary_lines < <(test_binary_from_cargo_messages "$case_target" < "$local_build_messages")
(( ${#local_test_binary_lines[@]} == 2 )) || fail "本机从 cargo 的输出里取不到测试可执行文件（$local_build_messages）" "看 $local_log.build 与 $local_build_messages"
local_test_binary="${local_test_binary_lines[0]}"; local_package_directory="${local_test_binary_lines[1]}"
local_cargo_executable="$(rustup which cargo 2>/dev/null || command -v cargo)"
peer_build_messages="$(run_on_peer "$peer_run_directory" "${peer_memory_cap_text}$(printf '%q ' cargo test --release -p "$case_package" --test "$case_target" --no-run --message-format=json) 2> .layer0-shard-build.log")" \
  || fail "第二台编测试二进制失败（cargo test --no-run）" "ssh 到第二台看 $peer_run_directory/.layer0-shard-build.log"
mapfile -t peer_test_binary_lines < <(test_binary_from_cargo_messages "$case_target" <<< "$peer_build_messages")
(( ${#peer_test_binary_lines[@]} == 2 )) || fail "第二台从 cargo 的输出里取不到测试可执行文件" "ssh 到第二台看 $peer_run_directory/.layer0-shard-build.log"
peer_test_binary="${peer_test_binary_lines[0]}"; peer_package_directory="${peer_test_binary_lines[1]}"
echo "  · ④ 两台各编好测试二进制：本机 $local_test_binary，第二台 $peer_test_binary"
mkdir -p -- "$local_progress_directory" || fail "建不了本机进度目录 $local_progress_directory" "看它可不可写"
( cd "$local_package_directory" && exec 9>"$local_progress_directory/.layer0-shard-run.lock" \
    && { flock -n 9 || { echo "✗ 本机进度目录 $local_progress_directory 的锁被占着：上一趟本机那一片还在跑"; echo "     → 怎么办：等上一趟本机那一片跑完再起（进度留着，这一趟接着续跑）；它该停，就按进程号停它，别删锁文件"; exit 75; }; } \
    && exec env "${start_over_settings[@]}" CARGO="$local_cargo_executable" SINGLEFS_LAYER0_SHARD=0/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY="$local_progress_directory" \
      SINGLEFS_LAYER0_INPUT_FINGERPRINT="$local_fingerprint" SINGLEFS_LAYER0_THREADS="$local_threads" SINGLEFS_LAYER0_FINDINGS_FILE="$local_findings_file" \
      "$local_test_binary" "${libtest_arguments[@]}" ) > "$local_log" 2>&1 &
local_shard_process=$!
tail -n +1 -F --pid="$local_shard_process" "$local_log" 2>/dev/null | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } | sed -u 's/^/    [本机 0\/2] /' &
peer_shard_command="exec 9>$(printf '%q' "$peer_progress_directory/.layer0-shard-run.lock") && { flock -n 9 || { echo '✗ 第二台进度目录的锁被占着：上一趟第二台那一片还在跑'; echo '     → 怎么办：等第二台上一趟那一片跑完再起（进度留着，这一趟接着续跑）；它该停，就按进程号停它，别删锁文件'; exit 75; }; } && echo \$\$ > $(printf '%q' "$peer_pid_file") && cd $(printf '%q' "$peer_package_directory") && exec env SINGLEFS_HEAVY_TESTS=${SINGLEFS_HEAVY_TESTS:-} ${start_over_text}CARGO=\"$cargo_executable_text\" SINGLEFS_LAYER0_SHARD=1/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=$(printf '%q' "$peer_progress_directory") SINGLEFS_LAYER0_INPUT_FINGERPRINT=$local_fingerprint SINGLEFS_LAYER0_THREADS=$peer_cores SINGLEFS_LAYER0_FINDINGS_FILE=$(printf '%q' "$peer_findings_file") $(printf '%q ' "$peer_test_binary" "${libtest_arguments[@]}")"
# 只让 setsid nohup 那一条进后台（它的输入输出都重定向了）：写成 `A && B && setsid … &` 的话，`&` 把整串放进后台子进程，
# 那个子进程握着 ssh 的输出通道、等第二台那一片跑完才退，ssh 就一直不返回（2026-09-28 预演第二条流卡在这里）
run_on_peer "$peer_run_directory" "mkdir -p $(printf '%q' "$peer_progress_directory") && rm -f -- $(printf '%q' "$peer_pid_file") $(printf '%q' "$peer_exit_file") $(printf '%q' "$peer_output_file") || exit 1; setsid nohup bash -c $(printf '%q' "${peer_memory_cap_text}bash -c $(printf '%q' "$peer_shard_command") > $(printf '%q' "$peer_output_file") 2>&1; echo \$? > $(printf '%q' "$peer_exit_file")") < /dev/null > /dev/null 2>&1 &" \
  || fail "第二台那一片起不来" "ssh 到第二台看 $peer_output_file"
peer_shard_started=1
{
  for ((pid_file_rounds = 0; pid_file_rounds < 30; pid_file_rounds++)); do
    run_on_peer / "test -s $(printf '%q' "$peer_pid_file") || test -s $(printf '%q' "$peer_exit_file")" && break
    sleep 1
  done
  run_on_peer / "peer_shard_pid=\"\$(cat $(printf '%q' "$peer_pid_file") 2>/dev/null)\" && tail -n +1 -F --pid=\"\$peer_shard_pid\" $(printf '%q' "$peer_output_file") 2>/dev/null" 2>/dev/null \
    | { grep --line-buffered '^LAYER0_PROGRESS ' || true; } | sed -u 's/^/    [第二台 1\/2] /'
} &
echo "  · ④ 两片开跑：本机 ${start_over_text}SINGLEFS_LAYER0_SHARD=0/2 直接执行测试二进制（pid $local_shard_process），第二台 SINGLEFS_LAYER0_SHARD=1/2 同一条、经 ${peer_memory_cap_text:-（不经内存包装）}起（进程号在第二台 $peer_pid_file）"
if wait "$local_shard_process"; then local_exit=0; else local_exit=$?; fi
peer_exit=""; peer_unreachable_rounds=0; peer_vanished_rounds=0
while [[ -z "$peer_exit" ]]; do
  if peer_poll="$(run_on_peer / "if [[ -s $(printf '%q' "$peer_exit_file") ]]; then echo \"exit \$(cat $(printf '%q' "$peer_exit_file"))\"; elif [[ -s $(printf '%q' "$peer_pid_file") && -d /proc/\$(cat $(printf '%q' "$peer_pid_file")) ]]; then echo running; elif [[ -s $(printf '%q' "$peer_pid_file") ]]; then echo vanished; else echo starting; fi" 2>/dev/null)"; then
    peer_unreachable_rounds=0
  else
    peer_poll=unreachable; peer_unreachable_rounds=$((peer_unreachable_rounds + 1))
  fi
  case "$peer_poll" in
    "exit "*) peer_exit="${peer_poll#exit }" ;;
    vanished) peer_vanished_rounds=$((peer_vanished_rounds + 1)); (( peer_vanished_rounds < 3 )) || peer_exit="vanished（进程没了、没留退出码）" ;;
    unreachable) (( peer_unreachable_rounds < 60 )) || peer_exit="unreachable（连续 5 分钟 ssh 不通）" ;;
    *) peer_vanished_rounds=0 ;;
  esac
  [[ -n "$peer_exit" ]] || sleep 5
done
copy_to_peer "$(peer_path "$peer_output_file")" "$peer_log" 2>/dev/null || echo "  ! 第二台那一片的输出 $peer_output_file 没拷回来"
shard_problems=()
case "$local_exit" in
  0) ;;
  75) shard_problems+=("本机那一片退 75：本机进度目录的锁被占着，上一趟本机那一片还在跑（先停它）") ;;
  *) shard_problems+=("本机那一片测试二进制退 $local_exit") ;;
esac
case "$peer_exit" in
  0) ;;
  25[0-4]) shard_problems+=("第二台那一片退 $peer_exit：内存包装 run-with-memory-cap.sh 自己的结局（250 撞了 PEER_MEMORY_CAP、251 起不了带上限的 scope、252 排不上、253 超时、254 被总上限挤掉），那一片的输出不算结果") ;;
  75) shard_problems+=("第二台那一片退 75：第二台进度目录的锁被占着，上一趟第二台那一片还在跑（先停它）") ;;
  *) shard_problems+=("第二台那一片退 $peer_exit") ;;
esac
grep -q 'LAYER0_SHARD mode=run shard=0/2 ' "$local_log" || shard_problems+=("本机的日志里没有 LAYER0_SHARD mode=run shard=0/2 那一行（这条用例没按分片跑）")
grep -q 'LAYER0_SHARD mode=run shard=1/2 ' "$peer_log" || shard_problems+=("第二台的日志里没有 LAYER0_SHARD mode=run shard=1/2 那一行（这条用例没按分片跑）")
# 第二台那一片的发现日志拷回本机、放在那一片的日志旁边（两片跑红了也拷：死之前找到的签名在里面）；第二台上那一份随 ⑦ 删掉
if [[ "$layer0_shard_break" == no-peer-findings-copy ]]; then
  echo "  ! 弄坏开关 LAYER0_SHARD_RUN_BREAK=no-peer-findings-copy：第二台那一片的发现日志不拷回"
elif copy_to_peer "$(peer_path "$peer_findings_file")" "$fetched_peer_findings_file" 2>/dev/null; then
  echo "  · 第二台那一片的发现日志拷回 $fetched_peer_findings_file"
else
  echo "  ! 第二台那一片的发现日志没拷回来（第二台上 $peer_findings_file 不在：那一片的枚举一趟都没走到，或它不走读 SINGLEFS_LAYER0_FINDINGS_FILE 的入口）"
fi
if (( ${#shard_problems[@]} > 0 )); then
  echo "    本机那一片的日志尾部（全文 $local_log）："; tail -20 "$local_log" | sed 's/^/      /'
  echo "    第二台那一片的日志尾部（全文 $peer_log）："; tail -20 "$peer_log" | sed 's/^/      /'
  remove_peer_run_directory
  fail_after_the_run "$(IFS='；'; echo "${shard_problems[*]}")" "单跑那一片看细节（本机：SINGLEFS_LAYER0_SHARD=0/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY=… SINGLEFS_LAYER0_INPUT_FINGERPRINT=… $cargo_test_text）；断言消息里是第一处对不上的计数或违例"
fi
echo "  · ④ 两片都跑完"

# ⑤ 第二台的账本拷回本机进度目录
copy_to_peer --include 'layer0-shard-*-shard-1-of-2.tally' --exclude '*' "$(peer_path "$peer_progress_directory")/" "$local_progress_directory/" \
  || fail_after_the_run "第二台的账本拷不回来" "看 $peer_progress_directory 里有没有 layer0-shard-*-shard-1-of-2.tally"
remove_peer_run_directory
fetched_ledgers=("$local_progress_directory"/layer0-shard-*-shard-1-of-2.tally)
own_ledgers=("$local_progress_directory"/layer0-shard-*-shard-0-of-2.tally)
[[ -f "${fetched_ledgers[0]}" && ${#fetched_ledgers[@]} == 1 && -f "${own_ledgers[0]}" && ${#own_ledgers[@]} == 1 ]] \
  || fail_after_the_run "本机进度目录里两片的账本不是各恰好一份（第 0 片 ${#own_ledgers[@]} 份、第 1 片 ${#fetched_ledgers[@]} 份，在 $local_progress_directory）" "看两片的日志里 LAYER0_SHARD mode=run 那一行的 ledger= 在哪；删掉多出来的旧账本再跑"
echo "  · ⑤ 第二台的账本拷回 ${fetched_ledgers[0]}"

# ⑥ 本机 merge：只读两份账本、核齐、按切片序号并，照旧走用例钉死的计数断言
merge_log="$scratch_directory/merge.log"
if [[ "$layer0_shard_mode" == standalone ]]; then merge_log="$log_prefix.merge.log"; fi
merge_findings_setting=(SINGLEFS_LAYER0_FINDINGS_FILE="$merge_findings_file")
if [[ "$layer0_shard_break" == no-merge-findings ]]; then merge_findings_setting=(-u SINGLEFS_LAYER0_FINDINGS_FILE); fi
merge_exit=0
( cd "$tree_root" && env -u SINGLEFS_LAYER0_START_OVER "${merge_findings_setting[@]}" SINGLEFS_LAYER0_SHARD=merge/2 SINGLEFS_LAYER0_PROGRESS_DIRECTORY="$local_progress_directory" \
  SINGLEFS_LAYER0_INPUT_FINGERPRINT="$local_fingerprint" SINGLEFS_LAYER0_THREADS="$local_threads" \
  cargo test --release -p "$case_package" --test "$case_target" -- "${libtest_arguments[@]}" ) > "$merge_log" 2>&1 || merge_exit=$?
if [[ "$merge_exit" == 0 ]] && ! grep -q 'LAYER0_SHARD mode=merge shards=2 ' "$merge_log"; then merge_exit=merge-line-missing; fi
echo "  · ⑥ 三份发现日志：本机 0/2 $local_findings_file；第二台 1/2 $fetched_peer_findings_file；merge $merge_findings_file"
if [[ "$layer0_shard_mode" == merged-log ]]; then
  cp -- "$merge_log" "$merged_log_file"
  echo "  · ⑥ merge 那一趟退 $merge_exit，整段输出交给 54 号判（$merged_log_file），发现日志交给 54 号读（$merge_findings_file）"
  [[ "$merge_exit" == 0 ]] && exit 0
  exit 1
fi
if [[ "$merge_exit" != 0 ]]; then
  echo "    merge 那一趟的日志尾部（全文 $merge_log）："
  tail -40 "$merge_log" | sed 's/^/      /'
  fail_after_the_run "merge 那一趟退 $merge_exit（上面是它的尾部；merge-line-missing 是日志里没有 LAYER0_SHARD mode=merge shards=2 那一行）" "账本核不齐时 panic 那一句说清缺哪一片、哪一处不同；计数断言红了与单机全量红了是同一回事，照 54 号 --full 的出路查"
fi
judged_lines_file="$scratch_directory/judged"
case_started_utc="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
if ! judge_output="$(python3 "$admission_module" crash-case-judge "$tree_root" "$case_key" "$merge_log" "$judged_lines_file")"; then
  printf '%s\n' "$judge_output" | sed 's/^/       /'
  fail_after_the_run "merge 那一趟跑过了，日志却判不绿（上面逐条列出）" "计数行、test result、exhaustive=true、每一片的工作线程，照 54 号 --full 同一句出路查"
fi
finish_manifest="$scratch_directory/manifest.finish"
finish_summary="$(python3 "$admission_module" crash-case-manifest "$tree_root" "$case_key" "$finish_manifest" \
  --judging-digest --toolchain --build-environment)" || fail "跑完之后算不出输入指纹：$finish_summary；不写全绿标记" "等改动停下，在 HEAD + 暂存区的 worktree 里重跑"
[[ "${finish_summary%% *}" == "$local_fingerprint" ]] \
  || fail "跑的过程中这条用例的输入变了（开跑 ${local_fingerprint:0:16}…，跑完 ${finish_summary:0:16}…）：不写全绿标记" "别在有人改这些路径的树里跑；在 HEAD + 暂存区的 worktree 里重跑"
threads_text="${judge_output//$'\n'/；}${judge_output:+；}双机分片：本机 ${local_threads_text}、本机 ${local_cores} 核，第二台 ${peer_cores} 核"
record_output="$(python3 "$admission_module" crash-case-record "$tree_root" "$case_key" "$local_fingerprint" "$local_manifest" "$judged_lines_file" \
  --files "$local_file_count" --excluded "$local_excluded_count" --started "$case_started_utc" --judged-root "$tree_root")" \
  || fail "判绿，全绿标记却没写成：$record_output" "看 $git_common_directory 可不可写、盘满没满"
echo "  ✓ $case_key 双机分片判绿（${threads_text}；三趟的日志与发现日志在 $(dirname "$log_prefix")/）：全绿标记写进 ${record_output}，记下的 $(grep -c . "$judged_lines_file") 行原样："
sed 's/^/      /' "$judged_lines_file"
