#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上两台各跑一份、取回、本机带导入再跑一趟，上一次的结论不替这一次作保；块的复用在判定库里
# run-condition: command bash python3 rsync
# gate-similar: scripts/crash/crash-amplification-two-hosts.sh 不另起一套双机跑法：崩溃点重放的一条崩溃枚举用例也是崩溃放量的一条流（整条流一个崩溃点），这一份只把门禁 checker-tier-crash-point-replay 交的那一条用例交给它、再把导入那一趟的日志交回门禁
# gate-overlap:copy-kept scripts/crash/crash-amplification-item.sh 排计划取本机速度这一段（decide_where_to_run 里取 crash_amplification_placement_rate 那几行，与排计划之后带 --local-states-per-second 交驱动那一句）照那边写：item.sh 是顺着主线往下跑的脚本、不是可 source 的库，抽成共用要动 item.sh 与驱动，这一份不动它们；排计划那一趟与速度合计都经共用库 crash-amplification-lib.sh 的 crash_amplification_run_on_this_host、crash_amplification_placement_rate
# 崩溃点重放按双机跑一条崩溃枚举用例：交崩溃放量的双机驱动 scripts/crash/crash-amplification-two-hosts.sh（本机 SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2、
# 第二台 1/2 各写各的库，第二台的库取回，本机再带 SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<取回的库> 跑一趟 0/2：导入第二台判的块、从库重建整条流的计数、
# 用例自己断言）；导入那一趟的日志抄进门禁给的日志文件，门禁照单机的判法判（scripts/gate/admission.py crash-case-judge）、写同一格全绿标记。
# 交驱动之前先定两台怎么分活：多机配置写了 HOST_JUDGE_WEIGHTS（两台分活的权重）就带 --host-weights 交驱动，录入段与判段同一份权重、两台分着录（文件头「两台分活的权重」）；
# 没写才按本机卡的算力分：开了 GPU 就先在本机排一趟派活计划取速度，带 --local-states-per-second 交驱动（驱动让第二台也量一趟、按两个速度分，
# 文件头「本机速度」）；取不到速度就不带、两台等分，并打一行说明原因。
#
#   crash-point-replay-shard-run.sh --merged-log <崩溃枚举用例的键> <树根> <输入指纹> <日志文件> <测试目标> <用例函数>
#       门禁 checker-tier-crash-point-replay --full 在双机配置开着时调（scripts/crash/crash-point-replay-shard-configuration-check.sh 判得过）；键与指纹只进标签与输出，
#       测试目标与用例函数交驱动的 <测试目标> 与 --test-filter。
#   crash-point-replay-shard-run.sh --selftest
#       拿假的双机驱动（CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER）与假的排计划命令（CRASH_POINT_REPLAY_SHARD_RUN_PLACEMENT_COMMAND）走一遍：交的参数、库目录、--quiesce、两台分活权重、本机速度、日志抄回、退出码照驱动；
#       另直接起判法脚本核 HOST_JUDGE_WEIGHTS 这个键（写对、写坏、没写）；末尾开关轮逐个打开弄坏开关核点名的格红。
#       命令里设了 SINGLEFS_HEAVY_TESTS（非空）时，假的那几格与开关轮都对了再跑一格真跑 real-two-hosts-run（下面「真跑那一格」）；没设时这一格报本次未跑，不算绿。
#       退出码：0 全对，真跑那一格本次未跑（没设变量；或设了变量而双机没开、驱动退 78）也是 0，输出里写明、报进 GATE_NOT_RUN_FILE；1 有格判错。
#
# 真跑那一格（real-two-hosts-run）：经真驱动（本仓的 crash-amplification-two-hosts.sh，不换）跑 checker 档一条标 ignore 的崩溃枚举用例，
#   起法与门禁 checker-tier-crash-point-replay --full 双机那一路相同（本脚本 --merged-log）。用例是 crash-case:new-pool-file-creation-stream：登记了 shard=across-machines 的四条里态数最小的
#   （闭式 16777260 态；另三条 multi-record-publish-stream 1358954634、unlink-after-the-multi-record-publish-stream 1463812278、overwrite-fixed-script-stream 1678377204，
#   各取自测试文件里断言的 closed_form；没登记 shard 的那几条不走双机驱动的两段与导入）。测试目标与用例函数从 .claude/gate.d/stage-inputs.tsv 那一行的 test= 现读。
#   库的家目录 SINGLEFS_CRASH_AMPLIFICATION_HOME 指到自证的临时目录、带 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1（判定库拒绝落在 /tmp 下，这一份是一次性的）；
#   多机配置是这台机器那一份（crash-amplification-lib.sh 的 crash_amplification_multi_host_configuration_file）拷进临时目录、ENABLE_GPU 改成 0、去掉显卡先后那两个键
#   （不上卡、不交 --quiesce 清场）；双机没开（ENABLE_ACROSS_MACHINES 不是 1）这一格本次未跑。本机编译目录
#   CARGO_TARGET_DIR=${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/crash-point-replay-shard-run-selftest-target（跨轮复用，不删）；第二台的照驱动（树副本上一层的 crash-amplification-target）。
#   判：包装退 0；runs/<标签>/record.log 里有 CRASH_LIBRARY_RECORDED class=crash_state_enumeration … library=…/crash-state-verdicts（录入段真跑或录入键命中、那一行真打）；
#   抄回的日志（导入那一趟）里 CRASH_POINT_REPLAY_PARALLEL_START 带 imported_blocks=<正数> shard=0/2、有一行不带 blocks_left_to_other_hosts= 的 CRASH_POINT_REPLAY_PARALLEL_FINISHED（导入之后对账为空）、
#   CRASH_POINT_REPLAY 计数行 states 等于 closed_form、violations=0、exhaustive=true。驱动退 78（运行条件：第二台连不上这一类）记本次未跑。判错时 runs/<标签>/ 下的日志拷到
#   ${TMPDIR:-/tmp}/crash-point-replay-shard-run-selftest-logs-<进程号>/ 留着、打出路径。
#   必红的样本：CRASH_AMPLIFICATION_ITEM_BREAK=include-ignored-dropped（录入段一条用例都不跑，驱动退 1）、library-marker-not-accepted（录入行不认，驱动退 1），各红在这一格。
#   冻树（真跑那一格对着活的主工作区各编各跑的病：别的会话中途往 crates 里换文件，本机三趟编出不同的二进制，导入那一趟的观察者版本和录入段的不是一把，
#   报「录入段那一块不在库里」，是假红）：开跑时把树拷一份到自证的临时目录（git ls-files -co --exclude-standard 列得出的文件，与 multi-host-run.sh 拷给第二台的是同一批；
#   不是 git 仓的树按 find 列；都不带 .git、target、target-*、multi-host.env、.env、qemu.env；规范副本 .claude/singlefs-ai-sop 另拷；rsync --no-times，拷过去的 mtime 是此刻，
#   cargo 一定重编副本里的本地包，不会被旧 mtime 骗去用别的树编的旧产物），驱动、本机三趟 cargo test 与第二台的拷树都在这一份副本上，三趟共用一份源码，退出时随临时目录删。
#   crates 指纹：Cargo.toml、Cargo.lock、.cargo/、crates/ 下每个文件的 sha256 清单（find 排序后 sha256sum）。开跑前取一次、拷完取副本的一次、跑完取活的主工作区的一次：
#   副本和开跑前的对不上（拷的过程中源树被换了文件）不起驱动，跑完活的主工作区和开跑前的对不上（跑的中途源树被换了文件），这一格都报「本次作废」并列出变了的文件（至多 5 个）——
#   照「本次未跑」那一类处理（退 0，输出里写明、报进 GATE_NOT_RUN_FILE），不当红也不当绿：副本上跑出来的结论证不了现在这棵树。
#   必红的样本：CRASH_POINT_REPLAY_SHARD_RUN_BREAK=real-run-on-live-tree（不拷树）、crates-change-not-voided（不比指纹），在假的主工作区上演（自证 real-run-* 三格，不起 cargo、不连第二台）。
#   第二台编译之前刷源码戳、持锁（这一份没有自己的第二台编译：第二台那一份经驱动 → multi-host-run.sh 的 MULTI_HOST_RUN_SHARED_TARGET_DIRECTORY → peer-host-lib.sh 的
#   peer_host_shared_target_guard）：这一格另判 runs/<标签>/share-1.log（第二台分份那一趟）里「· 第二台：拿到共用编译目录 … 的锁」与 SOURCE_STAMP touched= 两行都在、
#   前者在后者之前，且都在 cargo 打的头一行 Compiling / Finished / Running 之前。必红的样本：CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-shared-target-unguarded（驱动不交那个变量，两行都没有）。
#   驱动本机那几趟经 run-with-memory-cap.sh 起、上限是驱动的默认 38G：整条套在 run-with-memory-cap.sh 里跑时外层上限要不小于它（里层比外层大，包装退 251），
#   所以套在外层里时按下面「内存包装」一段把外层的上限交给驱动。
#
# 库目录：SINGLEFS_CRASH_AMPLIFICATION_LIBRARY 设了交驱动的 --library；没设交 <库的家目录>/crash-state-verdicts（那一份长期库，几类测试同一份，
#   crates/singlefs-checker-tier/src/crash_library.rs 的 CRASH_LIBRARY_DIRECTORY_NAME；家目录照 SINGLEFS_CRASH_AMPLIFICATION_HOME，没设是
#   ~/.local/share/singlefs/crash-amplification）。门禁 --start-over 照旧交这一份、另设 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1（只写不读）。
#   第二台那一份库由驱动交相对路径、落在树副本里再取回（几类测试都认库目录变量）。
# 内存包装：套在 scripts/hosts/run-with-memory-cap.sh 里跑时（门禁 --full 整条套在 16G 里），包装给命令设了 RUN_WITH_MEMORY_CAP_OUTER_CAP=<外层的上限>
#   （run-with-memory-cap.sh 文件头「套在包装里」）；这一趟把它照原样交驱动的 --memory-cap（驱动本机那几趟与没给 --peer-memory-cap 时的第二台都用它）：
#   里层上限比外层大，包装按用法错退 251、命令没跑。没设、写法不认得的不交（驱动用自己的默认 38G）。
# 线程：SINGLEFS_CRASH_POINT_REPLAY_THREADS 设了交 --threads（本机那两趟的核对线程数；用例先读 SINGLEFS_CRASH_POINT_REPLAY_THREADS、再读 SINGLEFS_CRASH_AMPLIFICATION_THREADS）；第二台的由驱动现算。
# GPU：多机配置（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}）写了 ENABLE_GPU=1 交 --quiesce（驱动照配置清场、跑完不复原、两台带 gpu 特性编）。
# 两台分活的权重（用户 2026-10-17 定「gpu任务分配时候 按照gpu的核数、算力来配置」；键的定义只在 scripts/crash/crash-point-replay-shard-configuration-check.sh 文件头「两台分活的权重」）：
#   多机配置写了 HOST_JUDGE_WEIGHTS=<本机>,<第二台>（两个正整数）就带 --host-weights <值> 交驱动、不再排计划量速度、打一行「两台分活权重：<值>（多机配置 HOST_JUDGE_WEIGHTS）」；
#   驱动文件头：给了 --host-weights 录入段与判段同一份权重，观察者录入段两台分着录（挂了观察者的用例录入段几个钟头，不再由本机整份录）。
#   值经判法脚本的 --emit-host-weights 取（SINGLEFS_MULTI_HOST_CONFIG 指到这一趟读的那一份配置），这一份不自己解析配置：退 0 打的就是值，退 3 是没写（走下面的量速度），
#   退 1 是配置判不过（打判法的两行原话、这一份退 1、驱动没起）。
# 本机速度（用户 2026-10-17 定「gpu任务分配时候 按照gpu的核数、算力来配置」，只在配置没写 HOST_JUDGE_WEIGHTS 时走；驱动文件头「两台怎么分」：给了 --local-states-per-second 就让第二台也排一趟计划量速度、按两个速度分，
#   都没给就等分，两台卡不一样多时慢的那台拖着整条）：多机配置写了 ENABLE_GPU=1 时，交驱动之前先在本机跑一趟只排派活计划（SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1，不判，
#   起法同分项驱动 crash-amplification-item.sh 排计划那一趟：共用库 crash_amplification_run_on_this_host 0/1，库取交驱动的 --library 同一份，不带发现日志），经 scripts/crash/gpu-quiesce-run.sh
#   起、显卡先后取清场之后那一份（与驱动 --quiesce 之后一致；清了场不复原，驱动随后再清是回读）。排计划那一趟是这个脚本自己带 --placement-only 再起的一次（内部用，不给人直接调：
#   <树根> <库> <日志> <测试目标> <用例函数>），线程取 SINGLEFS_CRASH_POINT_REPLAY_THREADS，本机内存上限取外层包装给的（文件头「内存包装」）、没有取驱动默认的那一个。
#   日志放 <库的家目录>/runs/<标签>/placement.log（与驱动那几份同目录；包装的输出在同目录 placement-run.out）；CRASH_AMPLIFICATION_PLACEMENT 行原样转打。
#   速度取共用库 crash_amplification_placement_rate（几条流要判的状态合计 ÷ 估的用时合计，不是各行速度相加），取到正整数就带 --local-states-per-second <数> 交驱动。
#   取不到的五种都退回等分、不带这一项、输出里打一行「本机速度没量到，两台等分（…）：<原因>」：多机配置没开 GPU（排计划只在 GPU 判器上量，不起这一趟）、
#   排计划那一趟退非 0（打了计划行也不认）、退 0 而一行 CRASH_AMPLIFICATION_PLACEMENT 都没有、行里没有 states_per_second_on_this_host 字段、速度合计是 0。
#   这一趟在 GPU 路径上多一趟本机的编译与录入（录入段不设选段，排计划那一趟先录再排；驱动随后的录入段按录入键命中；SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1 下命中与否这一份没验过）。
# 发现日志：环境里设 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE=<日志文件>.findings.tsv 带给驱动的本机那两趟；只有导入那一趟判完整条流、写一节（0/2 那一趟只判自己领的块、不写）。
# 日志：驱动那几份在 <库的家目录>/runs/<标签>/（local.log、第二台那几份、import.log），标签 crash-point-replay-<用例名>-<开跑时刻>-<进程号>；跑完不删。
# 退出码：驱动的退出码（导入那一趟 cargo test 的退出码；77、78、248、249、250–254 的含义见驱动文件头）；2 用法错；
#   1 驱动退 0 而导入那一趟的日志不在（驱动没跑到那一步却退 0：不当绿）；多机配置判不过（HOST_JUDGE_WEIGHTS 写坏这一类，驱动没起）也是 1。
# 重型：驱动跑 checker 档用例，只在提交时或用户要求时跑（SINGLEFS_HEAVY_TESTS 由调用方带着）；--selftest 没设 SINGLEFS_HEAVY_TESTS 时不起 cargo，不算重型，
#   设了就起真跑那一格（.claude/hooks/selftest-registry.tsv 登记 heavy-if-set:SINGLEFS_HEAVY_TESTS）。
#
# 只供 --selftest（自证起的每一趟里层设）：
#   CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER=<脚本>   换掉双机驱动（调法相同：<测试目标> --test-filter … --label … [--library …] [--threads …] [--quiesce] [--local-states-per-second …]）
#   CRASH_POINT_REPLAY_SHARD_RUN_PLACEMENT_COMMAND=<脚本>   换掉本机排计划那一趟（调法：<库> <日志>，日志里写计划行、退码照那一趟；不换时经清场包装起本脚本的 --placement-only）
# 弄坏开关 CRASH_POINT_REPLAY_SHARD_RUN_BREAK=<项>（只给 --selftest 证红；几个用逗号连）。一项一行，下一行起写打开后判红的格（格名是自证里 selftest_cells 的名字）：
#   library-not-passed   库目录不交成 --library
#       library-default、library-from-environment
#   log-not-copied       导入那一趟的日志不抄进日志文件
#       log-copied
#   exit-swallowed       驱动退非 0 也退 0
#       driver-exit-kept
#   quiesce-not-passed   ENABLE_GPU=1 也不交 --quiesce
#       quiesce-when-gpu
#   ignore-outer-cap     套在外层内存包装里也不把外层的上限交成 --memory-cap（改前的样子：驱动用自己的默认 38G，大过外层 16G，退 251）
#       outer-memory-cap-to-driver
#   speed-not-passed     量到了本机速度也不交 --local-states-per-second（改前的样子：两台等分）
#       local-speed-passed
#   placement-failure-ignored  排计划那一趟退非 0、没有计划行、速度是 0 也当量到了，把量到的（没有就是 0）交成 --local-states-per-second
#       local-speed-unavailable-equal-split
#   host-weights-not-passed  配置写了 HOST_JUDGE_WEIGHTS 也不交 --host-weights（改前的样子：走排计划量速度那一支，录入段由本机整份录）
#       host-weights-passed
#   real-run-on-live-tree  真跑那一格不拷树，驱动与三趟 cargo test 照改前对着活的主工作区跑
#       real-run-uses-a-frozen-copy
#   crates-change-not-voided  真跑那一格跑完不比 crates 指纹，中途 crates 变了也照常判（红的当红）
#       real-run-voided-when-crates-change
# 判法脚本的弄坏开关 MULTI_HOST_CONFIGURATION_BREAK=host-weights-unknown（定义在判法脚本文件头）也在这一份自证里证红，开关轮单独打开它：
#   红的格逐字是 host-weights-emitted host-weights-bad-refused host-weights-unwritten host-weights-passed host-weights-bad-red（判法脚本把这个键当不认得的键，配置判不过）。
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
shard_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
source "$shard_script_directory/crash-amplification-lib.sh"

shard_break_is() { [[ ",${CRASH_POINT_REPLAY_SHARD_RUN_BREAK:-}," == *",$1,"* ]]; }

usage_error() {
  echo "  ✗ 用法不对：$1" >&2
  echo "  → 怎么办：bash scripts/crash/crash-point-replay-shard-run.sh --merged-log <崩溃枚举用例的键> <树根> <输入指纹> <日志文件> <测试目标> <用例函数>，或 --selftest" >&2
  exit 2
}

# 驱动默认的本机内存上限：驱动文件里最先出现的 memory_cap="<数><单位>" 赋值（自证里的假驱动也这样取），读不到交回 1
shard_driver_default_memory_cap() { # <驱动文件>
  local driver_line pattern='memory_cap="([0-9]+[KMGT])"'
  while IFS= read -r driver_line; do
    if [[ "$driver_line" =~ $pattern ]]; then echo "${BASH_REMATCH[1]}"; return 0; fi
  done < "$1"
  return 1
}

# run_placement_only <树根> <库> <日志> <测试目标> <用例函数>：本机排一趟派活计划（SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1，不判），
# 日志写 <日志>；起法同分项驱动 crash-amplification-item.sh 排计划那一趟（共用库 crash_amplification_run_on_this_host 0/1）；已经清过场（文件头「本机速度」），
# 显卡先后取清场之后那一份，与驱动 --quiesce 之后的一致。退出码照 crash_amplification_run_on_this_host（线程数、内存上限算不出的 78 在里面）。
run_placement_only() {
  local library="$2" log="$3" outer_cap="${RUN_WITH_MEMORY_CAP_OUTER_CAP:-}"
  repository_root="$1" test_target="$4" test_filter="$5" expand_up_to="" library_is_temporary=0 quiesced=1
  threads="${SINGLEFS_CRASH_POINT_REPLAY_THREADS:-}"
  if [[ "$outer_cap" =~ ^[1-9][0-9]*[KMGT]$ ]]; then
    memory_cap="$outer_cap"
  else
    memory_cap="$(shard_driver_default_memory_cap "$repository_root/scripts/crash/crash-amplification-two-hosts.sh")" || {
      echo "  ✗ 读不出双机驱动默认的本机内存上限（$repository_root/scripts/crash/crash-amplification-two-hosts.sh 里没有 memory_cap=\"<数><单位>\"）" >&2
      echo "  → 怎么办：看驱动文件里 memory_cap 的默认值是不是改了写法；排计划取不到速度时这一趟照旧等分" >&2
      return 1
    }
  fi
  crash_amplification_prepare || return 78
  crash_amplification_run_on_this_host 0/1 "$library" "$log" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1
}

# shard_measure_local_speed <树根> <库> <本趟日志目录> <测试目标> <用例函数>：在本机排一趟派活计划，取本机这一趟的速度（态/秒）。
# 置 shard_local_speed（取到的正整数；取不到是空串）与 shard_speed_reason（取不到的原因，取到是空串）；直接调，不在 $(…) 里。
# 取速度照 crash-amplification-item.sh 的 decide_where_to_run：几条流的速度合计用共用库 crash_amplification_placement_rate（要判的状态合计 ÷ 估的用时合计）。
shard_measure_local_speed() {
  local root="$1" library="$2" run_directory="$3" target="$4" function="$5"
  local placement_log="$run_directory/placement.log" placement_exit speed lines_total lines_with_speed
  local -a placement_command
  shard_local_speed="" shard_speed_reason=""
  if ! mkdir -p -- "$run_directory"; then shard_speed_reason="建不了日志目录 $run_directory"; return 0; fi
  : > "$placement_log"
  if [[ -n "${CRASH_POINT_REPLAY_SHARD_RUN_PLACEMENT_COMMAND:-}" ]]; then
    placement_command=(bash "$CRASH_POINT_REPLAY_SHARD_RUN_PLACEMENT_COMMAND" "$library" "$placement_log")
  else
    # 经清场包装起：排计划量的是清场之后这台机器上的卡，与驱动 --quiesce 之后判的一致（清了场不复原，驱动随后再清一遍是回读）
    placement_command=(bash "$root/scripts/crash/gpu-quiesce-run.sh" -- bash "$0" --placement-only "$root" "$library" "$placement_log" "$target" "$function")
  fi
  echo "  · 本机先排一趟派活计划取速度（只排计划、不判）：日志 $placement_log"
  if "${placement_command[@]}" >"$run_directory/placement-run.out" 2>&1; then placement_exit=0; else placement_exit=$?; fi
  grep -h '^CRASH_AMPLIFICATION_PLACEMENT ' "$placement_log" 2>/dev/null
  speed="$(crash_amplification_placement_rate "$placement_log")"
  # 弄坏开关 placement-failure-ignored：排计划退非 0、没有计划行、速度是 0 也当量到了，把量到的（没有就是 0）交驱动
  if shard_break_is placement-failure-ignored; then shard_local_speed="${speed:-0}"; return 0; fi
  lines_total="$(grep -c '^CRASH_AMPLIFICATION_PLACEMENT ' "$placement_log" 2>/dev/null)"
  lines_with_speed="$(grep -c '^CRASH_AMPLIFICATION_PLACEMENT .*states_per_second_on_this_host=[0-9]' "$placement_log" 2>/dev/null)"
  if [[ "$placement_exit" != 0 ]]; then
    shard_speed_reason="排计划那一趟退 $placement_exit（看 $placement_log 与 $run_directory/placement-run.out）"
  elif [[ "${lines_total:-0}" == 0 ]]; then
    shard_speed_reason="排计划那一趟退 0，日志 $placement_log 里却一行 CRASH_AMPLIFICATION_PLACEMENT 都没有"
  elif [[ "${lines_with_speed:-0}" == 0 ]]; then
    shard_speed_reason="日志 $placement_log 的 CRASH_AMPLIFICATION_PLACEMENT 行里没有 states_per_second_on_this_host 字段"
  elif [[ ! "$speed" =~ ^[1-9][0-9]*$ ]]; then
    shard_speed_reason="日志 $placement_log 里本机的速度合计是 ${speed:-空}（要判的状态合计 ÷ 估的用时合计取不出正整数）"
  else
    shard_local_speed="$speed"
  fi
  return 0
}

# shard_read_host_weights <配置文件> <树根>：经判法脚本的 --emit-host-weights 取配置里的两台分活权重（文件头「两台分活的权重」），不自己解析配置。
# 置 shard_host_weights（写了是 <本机>,<第二台>；没写是空串）；配置判不过（退 1）或打出的值不是两个正整数，打判法的原话、返回 1；直接调，不在 $(…) 里。
shard_host_weights=""
shard_read_host_weights() {
  local weights_output weights_exit
  shard_host_weights=""
  if weights_output="$(env SINGLEFS_MULTI_HOST_CONFIG="$1" bash "$shard_script_directory/crash-point-replay-shard-configuration-check.sh" --emit-host-weights "$2" 2>&1)"; then
    weights_exit=0
  else
    weights_exit=$?
  fi
  case "$weights_exit" in
    0)
      if [[ "$weights_output" =~ ^[1-9][0-9]*,[1-9][0-9]*$ ]]; then shard_host_weights="$weights_output"; return 0; fi
      echo "  ✗ 判法脚本 --emit-host-weights 打的不是 <本机>,<第二台> 两个正整数：「$weights_output」"
      echo "     → 怎么办：单跑 SINGLEFS_MULTI_HOST_CONFIG=$1 bash $shard_script_directory/crash-point-replay-shard-configuration-check.sh --emit-host-weights 看它打了什么"
      return 1 ;;
    3) return 0 ;;
    *)
      echo "$weights_output"
      echo "  ✗ 多机配置 $1 判不过（取两台分活权重那一步，判法退 $weights_exit），双机驱动没起"
      echo "     → 怎么办：照上面判法写的原因改配置里点名的键（HOST_JUDGE_WEIGHTS 要写 <本机>,<第二台> 两个正整数，例 1,2），改好重跑 --full"
      return 1 ;;
  esac
}

# run_merged_log <键> <树根> <指纹> <日志> <测试目标> <用例函数>：交双机驱动，导入那一趟的日志抄进 <日志>；退出码照文件头「退出码」。
run_merged_log() {
  local key="$1" root="$2" fingerprint="$3" log="$4" target="$5" function="$6"
  [[ -d "$root" ]] || usage_error "树根 $root 不在"
  [[ "$fingerprint" =~ ^[0-9a-f]{64}$ ]] || usage_error "输入指纹要 64 位十六进制：$fingerprint"
  [[ "$key" == crash-case:* && -n "$target" && -n "$function" ]] || usage_error "键要以 crash-case: 起头，测试目标与用例函数不许空"
  local configuration_file gpu library_home library label driver import_log driver_exit
  configuration_file="$(crash_amplification_multi_host_configuration_file "$root")"
  gpu="$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU)"
  library_home="${SINGLEFS_CRASH_AMPLIFICATION_HOME:-$HOME/.local/share/singlefs/crash-amplification}"
  library="${SINGLEFS_CRASH_AMPLIFICATION_LIBRARY:-$library_home/crash-state-verdicts}"
  label="crash-point-replay-${key#crash-case:}-$(date -u +%Y%m%dT%H%M%SZ)-$$"
  driver="${CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER:-$root/scripts/crash/crash-amplification-two-hosts.sh}"
  local -a driver_arguments=("$target" --test-filter "$function" --label "$label")
  shard_break_is library-not-passed || driver_arguments+=(--library "$library")
  if [[ -n "${SINGLEFS_CRASH_POINT_REPLAY_THREADS:-}" ]]; then driver_arguments+=(--threads "$SINGLEFS_CRASH_POINT_REPLAY_THREADS"); fi
  if [[ "$gpu" == 1 ]] && ! shard_break_is quiesce-not-passed; then driver_arguments+=(--quiesce); fi
  # 外层内存包装给的上限（文件头「内存包装」）：写法不认得的当没给
  local outer_cap="${RUN_WITH_MEMORY_CAP_OUTER_CAP:-}"
  [[ "$outer_cap" =~ ^[1-9][0-9]*[KMGT]$ ]] || outer_cap=""
  shard_break_is ignore-outer-cap && outer_cap=""
  if [[ -n "$outer_cap" ]]; then driver_arguments+=(--memory-cap "$outer_cap"); fi
  # 两台分活的权重（文件头「两台分活的权重」）：配置写了 HOST_JUDGE_WEIGHTS 就带 --host-weights 交驱动、不再量速度；弄坏开关 host-weights-not-passed：写了也不带，当没写走量速度那一支
  shard_read_host_weights "$configuration_file" "$root" || return 1
  if shard_break_is host-weights-not-passed; then shard_host_weights=""; fi
  shard_local_speed="" shard_speed_reason=""
  if [[ -n "$shard_host_weights" ]]; then
    echo "  · 两台分活权重：$shard_host_weights（多机配置 HOST_JUDGE_WEIGHTS）：交驱动 --host-weights，录入段与判段同一份权重、两台分着录；不排计划量速度"
    driver_arguments+=(--host-weights "$shard_host_weights")
  else
    # 本机的速度（文件头「本机速度」）：开了 GPU 先在本机排一趟派活计划、取速度，带 --local-states-per-second 交驱动，驱动让第二台也量一趟、按两个速度分；取不到就等分、说明为什么
    if [[ "$gpu" != 1 ]]; then
      shard_speed_reason="多机配置没开 GPU（ENABLE_GPU 不是 1）：排派活计划只在 GPU 判器上量速度，CPU 判的两台没有按卡算的速度可量"
    else
      shard_measure_local_speed "$root" "$library" "$library_home/runs/$label" "$target" "$function"
    fi
    if [[ -n "$shard_local_speed" ]]; then
      echo "  · 本机速度：排派活计划量到本机每秒 $shard_local_speed 个态，交驱动 --local-states-per-second，驱动让第二台也量一趟、按两个速度分；配置没写 HOST_JUDGE_WEIGHTS，观察者录入段由本机整份录（要两台分着录，在配置里写 HOST_JUDGE_WEIGHTS=<本机>,<第二台>）"
      shard_break_is speed-not-passed || driver_arguments+=(--local-states-per-second "$shard_local_speed")
    else
      echo "  · 本机速度没量到，两台等分（驱动不带 --local-states-per-second）：$shard_speed_reason"
    fi
  fi
  echo "  · 崩溃点重放双机：$key（输入指纹 ${fingerprint:0:16}…）交崩溃放量的双机驱动：标签 $label，库 $library，ENABLE_GPU=${gpu:-0}；本机 0/2、第二台 1/2 各写各的库，取回，本机带导入再跑一趟"
  ( cd "$root" && env SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE="$log.findings.tsv" bash "$driver" "${driver_arguments[@]}" ); driver_exit=$?
  import_log="$library_home/runs/$label/import.log"
  if [[ -f "$import_log" ]]; then
    if ! shard_break_is log-not-copied; then
      cp -- "$import_log" "$log"
      echo "  · 导入那一趟的日志抄到 $log（驱动那几份在 $library_home/runs/$label/）"
    fi
  elif [[ "$driver_exit" == 0 ]]; then
    echo "  ✗ 双机驱动退 0，导入那一趟的日志 $import_log 却不在：没跑到导入那一步，不当绿"
    echo "     → 怎么办：看 $library_home/runs/$label/ 里有哪几份日志、驱动的输出停在哪一步；修好之后重跑 --full。"
    return 1
  else
    : > "$log"
    echo "  · 双机驱动退 $driver_exit，导入那一趟的日志不在（$import_log）：日志文件留空，门禁按驱动的退出码判红"
  fi
  if shard_break_is exit-swallowed; then return 0; fi
  return "$driver_exit"
}

# ============================== 自证 ==============================

FAKE_TWO_HOSTS_DRIVER='#!/usr/bin/env bash
printf "%s\n" "$*" > "${FAKE_DRIVER_CONTROL:?}/arguments"
printf "%s\n" "${SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE-unset}" > "$FAKE_DRIVER_CONTROL/findings-file"
label=""
local_cap=""
while (($#)); do
  if [[ "$1" == --label ]]; then label="$2"; shift 2
  elif [[ "$1" == --memory-cap ]]; then local_cap="$2"; shift 2
  else shift; fi
done
# 本机那几趟的上限：没给 --memory-cap 就取真驱动文件里 memory_cap="<数><单位>" 的第一处（驱动的默认），照真驱动的规矩
if [[ -z "$local_cap" ]]; then
  default_pattern="memory_cap=\"([0-9]+[KMGT])\""
  while IFS= read -r driver_line; do
    if [[ "$driver_line" =~ $default_pattern ]]; then local_cap="${BASH_REMATCH[1]}"; break; fi
  done < "${FAKE_DRIVER_REAL_DRIVER:?}"
fi
printf "%s\n" "$local_cap" > "$FAKE_DRIVER_CONTROL/local-memory-cap"
size_in_bytes() {
  local number="${1%[KMGT]}" unit="${1: -1}"
  case "$unit" in K) echo $((number * 1024)) ;; M) echo $((number * 1024 ** 2)) ;; G) echo $((number * 1024 ** 3)) ;; T) echo $((number * 1024 ** 4)) ;; esac
}
# 内存包装的规矩（scripts/hosts/run-with-memory-cap.sh 文件头「套在包装里」）：套在外层里而里层上限比外层大，按用法错退 251、命令没跑
echo "fake-driver: local memory cap=$local_cap outer=${RUN_WITH_MEMORY_CAP_OUTER_CAP-unset}"
if [[ "${RUN_WITH_MEMORY_CAP_OUTER_CAP:-}" =~ ^[1-9][0-9]*[KMGT]$ ]] && (( $(size_in_bytes "$local_cap") > $(size_in_bytes "$RUN_WITH_MEMORY_CAP_OUTER_CAP") )); then
  echo "run-with-memory-cap: 用法错：里层上限 $local_cap 比外层 $RUN_WITH_MEMORY_CAP_OUTER_CAP 大"
  exit 251
fi
if [[ ! -f "$FAKE_DRIVER_CONTROL/no-import-log" ]]; then
  mkdir -p "${SINGLEFS_CRASH_AMPLIFICATION_HOME:?}/runs/$label"
  printf "CRASH_POINT_REPLAY_PARALLEL_FINISHED states=29 slices=1 worker_threads=2\nCRASH_POINT_REPLAY states=29 exhaustive=true\n" > "$SINGLEFS_CRASH_AMPLIFICATION_HOME/runs/$label/import.log"
fi
exit "$(cat "$FAKE_DRIVER_CONTROL/exit" 2>/dev/null || echo 0)"
'
# 假的排计划命令（CRASH_POINT_REPLAY_SHARD_RUN_PLACEMENT_COMMAND，调法：<库> <日志>）：记下收到的库与被调了一次，按 $FAKE_DRIVER_CONTROL/placement-mode 写计划行、退码：
#   speed:<数>  打一行带 states_per_second_on_this_host=<数> 的 CRASH_AMPLIFICATION_PLACEMENT 行（要判的状态 = 数 × 10、估的用时 10 秒，合计出来仍是这个数），退 0
#   exit:<码>   一行都不打，退 <码>；exit-with-line:<码>:<数>  打一行好行，退 <码>
#   no-line     一行都不打，退 0；no-field  打一行缺 states_per_second_on_this_host 字段、也缺估的用时的行，退 0；zero  打一行速度与估的用时都是 0 的行，退 0
#   没写模式文件按 exit:1
FAKE_PLACEMENT_COMMAND='#!/usr/bin/env bash
library="$1"; log="$2"
printf "%s\n" "$library" > "${FAKE_DRIVER_CONTROL:?}/placement-library"
echo called >> "$FAKE_DRIVER_CONTROL/placement-called"
mode="$(cat "$FAKE_DRIVER_CONTROL/placement-mode" 2>/dev/null || echo exit:1)"
good_line() { printf "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=4 pending_states=%s tables_mebibytes=10 cards=local:0:100:100,peer:2:50:50 states_placed_on_local=100 states_placed_on_peer=50 states_per_second_on_this_host=%s estimated_seconds_on_this_host=10\n" "$(( $1 * 10 ))" "$1"; }
case "$mode" in
  speed:*) good_line "${mode#speed:}" > "$log"; exit 0 ;;
  exit-with-line:*) rest="${mode#exit-with-line:}"; good_line "${rest#*:}" > "$log"; exit "${rest%%:*}" ;;
  exit:*) exit "${mode#exit:}" ;;
  no-line) exit 0 ;;
  no-field) printf "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=4 pending_states=1000 tables_mebibytes=10\n" > "$log"; exit 0 ;;
  zero) printf "CRASH_AMPLIFICATION_PLACEMENT host=local judge=gpu pending_blocks=4 pending_states=1000 tables_mebibytes=10 states_per_second_on_this_host=0 estimated_seconds_on_this_host=0\n" > "$log"; exit 0 ;;
esac
exit 99
'
selftest_cells=(library-default library-from-environment log-copied driver-exit-kept quiesce-when-gpu no-import-log-red findings-file threads-passed outer-memory-cap-to-driver local-speed-passed local-speed-unavailable-equal-split
  host-weights-emitted host-weights-bad-refused host-weights-unwritten host-weights-passed host-weights-bad-red
  real-run-uses-a-frozen-copy real-run-voided-when-crates-change real-run-red-when-crates-unchanged)
selftest_host_weights_cells="host-weights-emitted host-weights-bad-refused host-weights-unwritten host-weights-passed host-weights-bad-red"  # MULTI_HOST_CONFIGURATION_BREAK=host-weights-unknown 该红的五格
selftest_zero_fingerprint="0000000000000000000000000000000000000000000000000000000000000000"

# selftest_write_configuration <ENABLE_GPU 的值> [<HOST_JUDGE_WEIGHTS 的值>]：写自证用的多机配置（判法脚本要的八个必写键齐、双机开；给了第二个参数（空串也算）就写 HOST_JUDGE_WEIGHTS 一行）
selftest_write_configuration() {
  {
    printf '%s\n' PEER_SSH_HOST=selftest-peer PEER_REPOSITORY_DIRECTORY=/selftest/peer/repository PEER_CARGO_BIN_DIRECTORY=/selftest/peer/cargo/bin PEER_MEMORY_CAP=24G \
      QUIESCE_STOP_COMMAND= QUIESCE_STOPPED_CHECK_COMMAND= QUIESCE_START_COMMAND= QUIESCE_STARTED_CHECK_COMMAND= ENABLE_ACROSS_MACHINES=1 "ENABLE_GPU=$1"
    if (( $# >= 2 )); then printf 'HOST_JUDGE_WEIGHTS=%s\n' "$2"; fi
  } > "$selftest_work/root/multi-host.env"
}

selftest_reset() {
  rm -rf -- "${selftest_work:?}/control" "${selftest_work:?}/home"
  rm -f -- "${selftest_work:?}/log" "${selftest_work:?}/log.findings.tsv" "${selftest_work:?}/output"
  mkdir -p -- "$selftest_work/control" "$selftest_work/home" "$selftest_work/root"
  selftest_write_configuration 0
}

# selftest_run_wrapper <日志> [环境 键=值…]：起这一份 --merged-log，输出写 <自证目录>/output，退出码放进 selftest_exit（直接调，不在 $(…) 里）
selftest_exit=""
selftest_run_wrapper() {
  local log="$1"
  shift
  local -a wrapper_environment=(-u SINGLEFS_CRASH_AMPLIFICATION_LIBRARY -u SINGLEFS_CRASH_POINT_REPLAY_THREADS -u SINGLEFS_MULTI_HOST_CONFIG
    -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP
    "SINGLEFS_CRASH_AMPLIFICATION_HOME=$selftest_work/home" "FAKE_DRIVER_CONTROL=$selftest_work/control"
    "FAKE_DRIVER_REAL_DRIVER=$shard_script_directory/crash-amplification-two-hosts.sh"
    "CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER=$selftest_work/fake-driver.sh"
    "CRASH_POINT_REPLAY_SHARD_RUN_PLACEMENT_COMMAND=$selftest_work/fake-placement.sh" "$@")
  if env "${wrapper_environment[@]}" bash "$0" --merged-log crash-case:demo "$selftest_work/root" "$selftest_zero_fingerprint" "$log" demo_target demo_function >"$selftest_work/output" 2>&1; then
    selftest_exit=0
  else
    selftest_exit=$?
  fi
}

selftest_arguments() { cat "$selftest_work/control/arguments" 2>/dev/null; }
selftest_last_output() { cat "$selftest_work/output" 2>/dev/null; }

cell_library_default() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" == "demo_target --test-filter demo_function --label crash-point-replay-demo-"*"--library $selftest_work/home/crash-state-verdicts"* ]]
}
cell_library_from_environment() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log" SINGLEFS_CRASH_AMPLIFICATION_LIBRARY="$selftest_work/start-over-library"
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" == *"--library $selftest_work/start-over-library"* ]]
}
cell_log_copied() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && -f "$selftest_work/log" && "$(cat "$selftest_work/log")" == *"CRASH_POINT_REPLAY states=29 exhaustive=true"* ]]
}
cell_driver_exit_kept() {
  selftest_reset
  echo 1 > "$selftest_work/control/exit"
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 1 ]]
}
cell_quiesce_when_gpu() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" != *"--quiesce"* ]] || return 1
  selftest_write_configuration 1
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" == *"--quiesce"* ]]
}
cell_no_import_log_red() {
  selftest_reset
  : > "$selftest_work/control/no-import-log"
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 1 && "$(selftest_last_output)" == *"不当绿"* ]]
}
cell_findings_file() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 && "$(cat "$selftest_work/control/findings-file")" == "$selftest_work/log.findings.tsv" ]]
}
cell_threads_passed() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log" SINGLEFS_CRASH_POINT_REPLAY_THREADS=5
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" == *"--threads 5"* ]]
}
# 外层内存包装给了 16G（RUN_WITH_MEMORY_CAP_OUTER_CAP=16G，门禁 --full 整条套在 16G 里）：驱动收到的本机上限不大于 16G、包装不退 251；
# 没套在外层里不交 --memory-cap（驱动用自己的默认）；外层上限写法不认得的当没给
cell_outer_memory_cap_to_driver() {
  selftest_reset
  selftest_run_wrapper "$selftest_work/log" RUN_WITH_MEMORY_CAP_OUTER_CAP=16G
  [[ "$selftest_exit" == 0 && "$(selftest_arguments)" == *"--memory-cap 16G"* && "$(cat "$selftest_work/control/local-memory-cap")" == 16G ]] || return 1
  selftest_reset
  selftest_run_wrapper "$selftest_work/log"
  [[ "$(selftest_arguments)" != *"--memory-cap"* ]] || return 1
  selftest_reset
  selftest_run_wrapper "$selftest_work/log" RUN_WITH_MEMORY_CAP_OUTER_CAP=sixteen
  [[ "$(selftest_arguments)" != *"--memory-cap"* ]]
}

# 本机的速度（文件头「本机速度」）：开了 GPU，本机排计划那一趟打了带 states_per_second_on_this_host=<数> 的计划行，驱动收到 --local-states-per-second <同一个数>；
# 排计划那一趟拿到的是交驱动的同一份库，日志放在与驱动几份日志同目录的 placement.log 里
cell_local_speed_passed() {
  selftest_reset
  selftest_write_configuration 1
  echo speed:1234 > "$selftest_work/control/placement-mode"
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 0 ]] || return 1
  local arguments label_arguments
  arguments="$(selftest_arguments)"
  [[ "$arguments" == *" --local-states-per-second 1234" || "$arguments" == *" --local-states-per-second 1234 "* ]] || return 1
  [[ "$(cat "$selftest_work/control/placement-library")" == "$selftest_work/home/crash-state-verdicts" ]] || return 1
  [[ "$(selftest_last_output)" == *"每秒 1234 个态"*"配置没写 HOST_JUDGE_WEIGHTS，观察者录入段由本机整份录"* ]] || return 1
  local -a placement_logs=("$selftest_work/home/runs/crash-point-replay-demo-"*/placement.log)
  [[ ${#placement_logs[@]} == 1 && -f "${placement_logs[0]}" ]] || return 1
  label_arguments="--label $(basename "$(dirname "${placement_logs[0]}")")"
  [[ "$arguments" == *"$label_arguments"* ]] || return 1
  [[ "$(cat "${placement_logs[0]}")" == *"states_per_second_on_this_host=1234 "* ]]
}
# 速度取不到：排计划那一趟退非 0（连同它打了好行的）、退 0 而一行计划行都没有、行里没有速度字段、速度合计是 0，驱动都不收到 --local-states-per-second，
# 输出里有一行说明退回等分和原因；没开 GPU 时排计划那一趟根本不起，同样等分、说明
cell_local_speed_unavailable_equal_split() {
  local case_row mode reason arguments
  for case_row in "exit:3|排计划那一趟退 3" "exit-with-line:3:1234|排计划那一趟退 3" "no-line|一行 CRASH_AMPLIFICATION_PLACEMENT 都没有" \
    "no-field|没有 states_per_second_on_this_host 字段" "zero|本机的速度合计是 0"; do
    mode="${case_row%%|*}" reason="${case_row#*|}"
    selftest_reset
    selftest_write_configuration 1
    echo "$mode" > "$selftest_work/control/placement-mode"
    selftest_run_wrapper "$selftest_work/log"
    arguments="$(selftest_arguments)"
    [[ "$selftest_exit" == 0 && -f "$selftest_work/control/placement-called" && "$arguments" == "demo_target --test-filter demo_function --label "* \
      && "$arguments" != *"--local-states-per-second"* && "$(selftest_last_output)" == *"本机速度没量到，两台等分"*"$reason"* ]] || return 1
  done
  selftest_reset
  echo speed:1234 > "$selftest_work/control/placement-mode"
  selftest_run_wrapper "$selftest_work/log"
  arguments="$(selftest_arguments)"
  [[ "$selftest_exit" == 0 && ! -e "$selftest_work/control/placement-called" && "$arguments" != *"--local-states-per-second"* \
    && "$(selftest_last_output)" == *"本机速度没量到，两台等分"*"没开 GPU"* ]]
}

# 判法脚本直接起（文件头「两台分活的权重」）：selftest_check_script [<方式>]，配置是 selftest_write_configuration 写的那一份，不连第二台；
# 输出放进 selftest_check_output，退出码放进 selftest_check_exit（直接调，不在 $(…) 里）
selftest_check_output="" selftest_check_exit=""
selftest_check_script() {
  local -a check_mode=()
  if [[ -n "${1:-}" ]]; then check_mode=("$1"); fi
  if selftest_check_output="$(env SINGLEFS_MULTI_HOST_CONFIG="$selftest_work/root/multi-host.env" SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 \
      bash "$shard_script_directory/crash-point-replay-shard-configuration-check.sh" ${check_mode[@]+"${check_mode[@]}"} "$selftest_work/root" 2>&1)"; then
    selftest_check_exit=0
  else
    selftest_check_exit=$?
  fi
}

# 键写对（1,2、10,3）：判得过（退 0），--emit-host-weights 退 0、只打写的那个值
cell_host_weights_emitted() {
  selftest_reset
  local weights
  for weights in 1,2 10,3; do
    selftest_write_configuration 0 "$weights"
    selftest_check_script
    [[ "$selftest_check_exit" == 0 ]] || return 1
    selftest_check_script --emit-host-weights
    [[ "$selftest_check_exit" == 0 && "$selftest_check_output" == "$weights" ]] || return 1
  done
}
# 键写坏（1,0、a,b、1、1,2,3、前导 0、带空格、0,1）：四种方式都判不过（退 1），点名 HOST_JUDGE_WEIGHTS 并说清要写什么，附出路
cell_host_weights_bad_refused() {
  selftest_reset
  local bad mode
  for bad in 1,0 a,b 1 1,2,3 01,2 "1, 2" 0,1; do
    selftest_write_configuration 0 "$bad"
    for mode in "" --emit-host-weights --emit-hosts --emit-assignments; do
      selftest_check_script "$mode"
      [[ "$selftest_check_exit" == 1 && "$selftest_check_output" == *"HOST_JUDGE_WEIGHTS 要写两台分活的权重"*"读到「$bad」"* && "$selftest_check_output" == *"怎么办"* ]] || return 1
    done
  done
}
# 键没写（整行没有、写空串）：--emit-host-weights 退 3、什么都不打；--emit-hosts、--emit-assignments 的输出在写 1,2 与没写时逐字相同、都不打这个键
cell_host_weights_unwritten() {
  selftest_reset
  local baseline_hosts baseline_assignments
  selftest_write_configuration 0
  selftest_check_script --emit-host-weights
  [[ "$selftest_check_exit" == 3 && -z "$selftest_check_output" ]] || return 1
  selftest_check_script --emit-hosts
  [[ "$selftest_check_exit" == 0 ]] || return 1
  baseline_hosts="$selftest_check_output"
  selftest_check_script --emit-assignments
  [[ "$selftest_check_exit" == 0 ]] || return 1
  baseline_assignments="$selftest_check_output"
  selftest_write_configuration 0 ""
  selftest_check_script --emit-host-weights
  [[ "$selftest_check_exit" == 3 && -z "$selftest_check_output" ]] || return 1
  selftest_write_configuration 0 1,2
  selftest_check_script --emit-hosts
  [[ "$selftest_check_exit" == 0 && "$selftest_check_output" == "$baseline_hosts" && "$selftest_check_output" != *HOST_JUDGE_WEIGHTS* ]] || return 1
  selftest_check_script --emit-assignments
  [[ "$selftest_check_exit" == 0 && "$selftest_check_output" == "$baseline_assignments" && "$selftest_check_output" != *HOST_JUDGE_WEIGHTS* ]]
}
# 配置写了键：驱动收到 --host-weights <值>（GPU 开着、关着都一样）、不收到 --local-states-per-second，排计划命令没被调（排计划模式文件写了速度也不用），输出打那一行
cell_host_weights_passed() {
  local arguments
  selftest_reset
  selftest_write_configuration 1 1,2
  echo speed:1234 > "$selftest_work/control/placement-mode"
  selftest_run_wrapper "$selftest_work/log"
  arguments="$(selftest_arguments)"
  [[ "$selftest_exit" == 0 && ( "$arguments" == *" --host-weights 1,2" || "$arguments" == *" --host-weights 1,2 "* ) && "$arguments" != *"--local-states-per-second"* \
    && ! -e "$selftest_work/control/placement-called" && "$(selftest_last_output)" == *"两台分活权重：1,2（多机配置 HOST_JUDGE_WEIGHTS）"* ]] || return 1
  selftest_reset
  selftest_write_configuration 0 3,1
  selftest_run_wrapper "$selftest_work/log"
  arguments="$(selftest_arguments)"
  [[ "$selftest_exit" == 0 && ( "$arguments" == *" --host-weights 3,1" || "$arguments" == *" --host-weights 3,1 "* ) && "$arguments" != *"--local-states-per-second"* \
    && ! -e "$selftest_work/control/placement-called" ]]
}
# 配置里键写坏：包装退 1、驱动没起（没有记下参数的文件）、输出点名 HOST_JUDGE_WEIGHTS 并说清要写什么
cell_host_weights_bad_red() {
  selftest_reset
  selftest_write_configuration 1 1,0
  selftest_run_wrapper "$selftest_work/log"
  [[ "$selftest_exit" == 1 && ! -e "$selftest_work/control/arguments" && "$(selftest_last_output)" == *"HOST_JUDGE_WEIGHTS 要写两台分活的权重"* ]]
}

# 真跑那一格的冻树与 crates 指纹（文件头「真跑那一格」冻树一段），在假的主工作区上演，不起 cargo、不连第二台：假的真跑包装记下收到的树根、前后读副本里 crates/lib.rs 两次，
# 模式 change-crates-during 时在两次之间把活的主工作区里同一份改成 version two（等同别的会话中途往 crates 里换文件），退 3（等同真跑红）。
# selftest_real_scene <场景名> [<包装模式>]：造假的主工作区与假包装，调 real_two_hosts_run；结果在 real_cell_outcome、real_cell_detail，场景目录放进 real_scene_directory
real_scene_directory=""
selftest_real_scene() {
  local name="$1" mode="${2:-}" had_configuration=0 saved_configuration="${SINGLEFS_MULTI_HOST_CONFIG-}"
  if [[ -v SINGLEFS_MULTI_HOST_CONFIG ]]; then had_configuration=1; fi
  real_scene_directory="$selftest_work/real-scene-$name"
  rm -rf -- "${real_scene_directory:?}"
  mkdir -p -- "$real_scene_directory/source/crates" "$real_scene_directory/source/.claude/gate.d" "$real_scene_directory/source/.cargo"
  printf 'version one\n' >"$real_scene_directory/source/crates/lib.rs"
  printf '[workspace]\n' >"$real_scene_directory/source/Cargo.toml"
  printf '# lock\n' >"$real_scene_directory/source/Cargo.lock"
  printf 'crash-case:new-pool-file-creation-stream\t-\ttest=pkg:demo_target:demo_function shard=across-machines\n' >"$real_scene_directory/source/.claude/gate.d/stage-inputs.tsv"
  printf 'ENABLE_ACROSS_MACHINES=1\nENABLE_GPU=1\n' >"$real_scene_directory/source/multi-host.env"
  cat >"$real_scene_directory/fake-wrapper.sh" <<'FAKE_REAL_WRAPPER'
#!/usr/bin/env bash
# 假的真跑包装（自证用）：调法 <树根> <这一格的目录>
tree="$1" real="$2"
printf '%s\n' "$tree" >"$real/wrapper-saw-tree"
cat "$tree/crates/lib.rs" >"$real/wrapper-saw-first"
if [[ "${FAKE_REAL_WRAPPER_MODE:-}" == change-crates-during ]]; then printf 'version two\n' >"${FAKE_REAL_WRAPPER_LIVE_ROOT:?}/crates/lib.rs"; fi
cat "$tree/crates/lib.rs" >"$real/wrapper-saw-last"
exit 3
FAKE_REAL_WRAPPER
  real_cell_source_root="$real_scene_directory/source" real_cell_work="$real_scene_directory/work" real_cell_wrapper="$real_scene_directory/fake-wrapper.sh"
  export SINGLEFS_MULTI_HOST_CONFIG="$real_scene_directory/source/multi-host.env" FAKE_REAL_WRAPPER_MODE="$mode" FAKE_REAL_WRAPPER_LIVE_ROOT="$real_scene_directory/source"
  real_two_hosts_run >"$real_scene_directory/run.out" 2>&1   # 它的「· 真跑那一格」说明不进自证格的输出（run_selftest_cells 的 stdout 就是红格名单）
  real_cell_source_root="" real_cell_work="" real_cell_wrapper=""
  unset FAKE_REAL_WRAPPER_MODE FAKE_REAL_WRAPPER_LIVE_ROOT
  if (( had_configuration )); then export SINGLEFS_MULTI_HOST_CONFIG="$saved_configuration"; else unset SINGLEFS_MULTI_HOST_CONFIG; fi
}
# 驱动与三趟 cargo test 跑在开跑时拷出来的树上：包装收到的树根是副本（不是活的主工作区），活的主工作区中途被换了文件，副本里前后读到的还是原来的；
# 副本里有 stage-inputs.tsv 这类整棵树的文件、没有 multi-host.env（私有配置不拷）
cell_real_run_uses_a_frozen_copy() {
  selftest_real_scene frozen change-crates-during
  local work="$real_scene_directory/work" saw_tree
  [[ -f "$work/wrapper-saw-tree" && -f "$work/wrapper-saw-last" ]] || return 1
  saw_tree="$(cat "$work/wrapper-saw-tree")"
  [[ "$saw_tree" == "$work/tree" && "$saw_tree" != "$real_scene_directory/source" && "$(cat "$work/wrapper-saw-first")" == "version one" \
     && "$(cat "$work/wrapper-saw-last")" == "version one" && "$(cat "$real_scene_directory/source/crates/lib.rs")" == "version two" \
     && -f "$saw_tree/.claude/gate.d/stage-inputs.tsv" && ! -e "$saw_tree/multi-host.env" ]]
}
# 跑的中途活的主工作区的 crates 变了：这一格报「本次作废」（outcome 是 not-run，不是 red），列出变了的文件
cell_real_run_voided_when_crates_change() {
  selftest_real_scene voided change-crates-during
  [[ "$real_cell_outcome" == not-run && "$real_cell_detail" == "本次作废"*"crates/lib.rs"* ]]
}
# 对照：crates 没变、包装退 3：照常判红（作废只在 crates 变了的时候）
cell_real_run_red_when_crates_unchanged() {
  selftest_real_scene unchanged
  [[ "$real_cell_outcome" == red && "$real_cell_detail" == "包装退 3"* ]]
}

# 真跑那一格（文件头「真跑那一格」）：real_cell_outcome 置成 green、red 或 not-run，real_cell_detail 是一句说明
REAL_CELL_CASE_KEY="crash-case:new-pool-file-creation-stream"
real_cell_outcome="" real_cell_detail=""
# 只给自证里的假场景设（real-run-* 三格）：假的主工作区、这一趟的工作目录、假的真跑包装；平常三个都空，取真的主工作区、$selftest_work/real、真的 --merged-log
real_cell_source_root="" real_cell_work="" real_cell_wrapper=""
# 冻树要用的两个函数（文件头「真跑那一格」冻树一段）。
# shard_crates_manifest <树根> <输出文件>：Cargo.toml、Cargo.lock、.cargo/、crates/ 下每个文件的「sha256  路径」清单（路径相对树根、排序），任何一个文件读不了就退 1
shard_crates_manifest() {
  local root="$1" output="$2" entry
  local -a entries=()
  for entry in Cargo.toml Cargo.lock .cargo crates; do
    if [[ -e "$root/$entry" ]]; then entries+=("$entry"); fi
  done
  (( ${#entries[@]} > 0 )) || return 1
  ( cd "$root" && find "${entries[@]}" \( -type d \( -name target -o -name 'target-*' -o -name __pycache__ \) -prune \) -o -type f -print0 \
      | LC_ALL=C sort -z | xargs -0 -r sha256sum ) >"$output" 2>"$output.err"
}
# shard_manifest_changes <清单甲> <清单乙>：两份清单里不一样的文件路径，至多 5 个，逗号连起来
shard_manifest_changes() {
  diff -- "$1" "$2" | sed -n -E 's/^[<>] [0-9a-f]{64}  //p' | LC_ALL=C sort -u | head -n 5 | paste -sd, -
}
# shard_freeze_tree <源树根> <副本>：把源树拷一份到副本（文件头「真跑那一格」冻树一段）；任何一步失败退 1
shard_freeze_tree() {
  local source="$1" destination="$2" list="$2.list"
  mkdir -p -- "$destination" || return 1
  if git -C "$source" rev-parse --is-inside-work-tree >/dev/null 2>&1; then
    git -C "$source" ls-files -co --exclude-standard -z --deduplicate >"$list" || return 1
  else
    ( cd "$source" && find . -mindepth 1 \( -name .git -o -name target -o -name 'target-*' -o -name multi-host.env -o -name .env -o -name qemu.env -o -path ./.claude/singlefs-ai-sop \) -prune \
        -o \( -type f -o -type l \) -printf '%P\0' ) >"$list" || return 1
  fi
  rsync -a --no-times --ignore-missing-args --from0 --files-from="$list" "$source/" "$destination/" || return 1
  if [[ -d "$source/.claude/singlefs-ai-sop" ]]; then
    mkdir -p -- "$destination/.claude" && rsync -a --no-times --exclude __pycache__ "$source/.claude/singlefs-ai-sop/" "$destination/.claude/singlefs-ai-sop/" || return 1
  fi
  rm -f -- "$list"
}

real_two_hosts_run() {
  local repository_root real configuration_source across row test_field target function label_directories runs import_copy record_log wrapper_exit
  local frozen_tree cargo_target voided="" crates_before crates_after
  local -a real_wrapper
  repository_root="${real_cell_source_root:-$(cd "$shard_script_directory/../.." && pwd)}"
  real="${real_cell_work:-$selftest_work/real}"
  cargo_target="${GATE_CROSS_RUN_TMPDIR:-${TMPDIR:-/tmp}}/crash-point-replay-shard-run-selftest-target"
  mkdir -p -- "$real/home"
  configuration_source="$(crash_amplification_multi_host_configuration_file "$repository_root")"
  across="$(crash_amplification_configuration_value "$configuration_source" ENABLE_ACROSS_MACHINES)"
  if [[ ! -f "$configuration_source" || "$across" != 1 ]]; then
    real_cell_outcome=not-run real_cell_detail="多机配置 $configuration_source 不在、或没写 ENABLE_ACROSS_MACHINES=1：双机没开，真驱动跑不了双机"
    return 0
  fi
  # 拷一份：ENABLE_GPU 改成 0、去掉显卡先后那两个键（不上卡、不清场）；拷贝在自证的临时目录里，退出时删
  { grep -vE '^[[:space:]]*(export[[:space:]]+)?(ENABLE_GPU|GPU_CARDS_BY_PRIORITY|GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED)[[:space:]]*=' "$configuration_source"; echo ENABLE_GPU=0; } >"$real/multi-host.env"
  row="$(awk -F'\t' -v key="$REAL_CELL_CASE_KEY" '$1 == key { print $3; exit }' "$repository_root/.claude/gate.d/stage-inputs.tsv")"
  test_field="$(tr ' ' '\n' <<<"$row" | grep -m 1 '^test=')"
  target="$(cut -d: -f2 <<<"${test_field#test=}")" function="$(cut -d: -f3 <<<"${test_field#test=}")"
  if [[ -z "$target" || -z "$function" || " $row " != *" shard=across-machines "* ]]; then
    real_cell_outcome=red real_cell_detail=".claude/gate.d/stage-inputs.tsv 里 $REAL_CELL_CASE_KEY 那一行读不出 test=<包>:<测试目标>:<用例函数>、或没登记 shard=across-machines（读到「$row」）"
    return 0
  fi
  # 冻树（文件头「真跑那一格」冻树一段）：开跑前取 crates 指纹，把树拷一份，副本的指纹要和开跑前的一样；驱动与三趟 cargo test 都在副本上跑
  crates_before="$real/crates-before.tsv" crates_after="$real/crates-after.tsv"
  if ! shard_crates_manifest "$repository_root" "$crates_before"; then
    real_cell_outcome=not-run real_cell_detail="本次作废：开跑前读不出 $repository_root 的 crates 指纹（Cargo.toml、Cargo.lock、.cargo/、crates/ 下有文件读不了：$(head -c 300 "$crates_before.err" 2>/dev/null)）"
    return 0
  fi
  frozen_tree="$real/tree"
  if shard_break_is real-run-on-live-tree; then   # 弄坏开关 real-run-on-live-tree：不拷树，对着活的主工作区跑（改前的样子）
    frozen_tree="$repository_root"
  elif ! shard_freeze_tree "$repository_root" "$frozen_tree" >"$real/freeze.out" 2>&1; then
    real_cell_outcome=red real_cell_detail="开跑前把 $repository_root 拷到 $frozen_tree 失败（$(tail -n 3 "$real/freeze.out" | tr '\n' '|')）：看临时目录盘满没满、rsync 在不在"
    return 0
  elif ! shard_crates_manifest "$frozen_tree" "$real/crates-copy.tsv" || ! cmp -s -- "$crates_before" "$real/crates-copy.tsv"; then
    real_cell_outcome=not-run real_cell_detail="本次作废：拷树的过程中 $repository_root 的 crates 被换了文件，副本和开跑前的对不上（变了：$(shard_manifest_changes "$crates_before" "$real/crates-copy.tsv")），没起驱动"
    return 0
  fi
  echo "  · 真跑那一格：$REAL_CELL_CASE_KEY（$target::$function）在开跑时拷出来的树 $frozen_tree 上经真驱动双机跑（三趟共用这一份源码），库的家目录 $real/home，编译目录 $cargo_target"
  if [[ -n "$real_cell_wrapper" ]]; then
    real_wrapper=(bash "$real_cell_wrapper" "$frozen_tree" "$real")
  else
    real_wrapper=(bash "$0" --merged-log "$REAL_CELL_CASE_KEY" "$frozen_tree" "$selftest_zero_fingerprint" "$real/merged.log" "$target" "$function")
  fi
  if env -u SINGLEFS_CRASH_AMPLIFICATION_LIBRARY -u CRASH_POINT_REPLAY_SHARD_RUN_TWO_HOSTS_DRIVER -u CRASH_POINT_REPLAY_SHARD_RUN_BREAK \
      SINGLEFS_CRASH_AMPLIFICATION_HOME="$real/home" SINGLEFS_CRASH_AMPLIFICATION_LIBRARY_IS_TEMPORARY=1 SINGLEFS_MULTI_HOST_CONFIG="$real/multi-host.env" \
      CARGO_TARGET_DIR="$cargo_target" "${real_wrapper[@]}" >"$real/output" 2>&1; then
    wrapper_exit=0
  else
    wrapper_exit=$?
  fi
  # 跑完再取活的主工作区的 crates 指纹；弄坏开关 crates-change-not-voided：不比（中途变了也照常判）
  if ! shard_break_is crates-change-not-voided; then
    if ! shard_crates_manifest "$repository_root" "$crates_after"; then
      voided="本次作废：跑完读不出 $repository_root 的 crates 指纹（$(head -c 300 "$crates_after.err" 2>/dev/null)），副本上的结论证不了现在这棵树"
    elif ! cmp -s -- "$crates_before" "$crates_after"; then
      voided="本次作废：跑的过程中 $repository_root 的 crates 被换了文件（变了：$(shard_manifest_changes "$crates_before" "$crates_after")），这一趟是在开跑时拷出来的树上跑的，证不了现在这棵树；不当红也不当绿，等换文件的会话停手再跑"
    fi
  fi
  label_directories=("$real/home/runs/crash-point-replay-${REAL_CELL_CASE_KEY#crash-case:}-"*)
  runs="${label_directories[0]}"
  record_log="$runs/record.log" import_copy="$real/merged.log"
  # 第二台分份那一趟编译之前刷源码戳、持锁（文件头「真跑那一格」）：ok，或「lock=<行号> stamp=<行号> cargo=<行号>」（0 是没有）
  local peer_guard_order=""
  if [[ -n "$voided" ]]; then
    real_cell_outcome=not-run real_cell_detail="$voided"
  elif [[ "$wrapper_exit" == 78 ]]; then
    real_cell_outcome=not-run real_cell_detail="驱动退 78（运行条件不满足：第二台连不上、库没取回、线程数现算不出这一类；输出尾部：$(tail -n 4 "$real/output" | tr '\n' '|')）"
  elif [[ "$wrapper_exit" != 0 ]]; then
    real_cell_outcome=red real_cell_detail="包装退 $wrapper_exit（输出尾部：$(tail -n 6 "$real/output" | tr '\n' '|')）"
  elif (( ${#label_directories[@]} != 1 )) || [[ ! -d "$runs" ]]; then
    real_cell_outcome=red real_cell_detail="$real/home/runs/ 下这一趟的日志目录不是恰好一个（${label_directories[*]}）"
  elif ! grep -qE '^CRASH_LIBRARY_RECORDED class=crash_state_enumeration item=[^ ]+ manifest=[0-9a-f]+ library=.*/crash-state-verdicts$' "$record_log" 2>/dev/null; then
    real_cell_outcome=red real_cell_detail="录入段那一趟的日志 $record_log 里没有 CRASH_LIBRARY_RECORDED class=crash_state_enumeration … library=…/crash-state-verdicts 那一行（录入段没真跑，或录进了别的库）"
  elif peer_guard_order="$(awk '/^  · 第二台：拿到共用编译目录 / && !lock { lock = NR } /^SOURCE_STAMP touched=/ && !stamp { stamp = NR }
                                /^ +(Compiling|Finished|Running) / && !cargo { cargo = NR }
                                END { if (lock && stamp && lock < stamp && (!cargo || stamp < cargo)) print "ok"; else print "lock=" lock + 0 " stamp=" stamp + 0 " cargo=" cargo + 0 }' "$runs/share-1.log" 2>/dev/null)"; \
       [[ "$peer_guard_order" != ok ]]; then
    real_cell_outcome=red real_cell_detail="第二台分份那一趟的日志 $runs/share-1.log 里编译之前没走共用编译目录的守卫：要「· 第二台：拿到共用编译目录 … 的锁」在 SOURCE_STAMP touched= 之前、两行都在 cargo 打的头一行 Compiling / Finished / Running 之前，实际行号 ${peer_guard_order:-（日志读不了）}（0 是没有）"
  elif ! grep -qE '^CRASH_POINT_REPLAY_PARALLEL_START .* imported_blocks=[1-9][0-9]* shard=0/2 ' "$import_copy"; then
    real_cell_outcome=red real_cell_detail="抄回的导入那一趟日志里 CRASH_POINT_REPLAY_PARALLEL_START 不带 imported_blocks=<正数> shard=0/2（第二台判的块没导进来）"
  elif [[ -z "$(grep '^CRASH_POINT_REPLAY_PARALLEL_FINISHED ' "$import_copy" | grep -v 'blocks_left_to_other_hosts=')" ]]; then
    real_cell_outcome=red real_cell_detail="抄回的导入那一趟日志里没有不带 blocks_left_to_other_hosts= 的 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行（导入之后对账不为空）"
  elif ! grep -qE '^CRASH_POINT_REPLAY states=([0-9]+) closed_form=\1 violations=0 .*exhaustive=true' "$import_copy"; then
    real_cell_outcome=red real_cell_detail="抄回的导入那一趟日志里 CRASH_POINT_REPLAY 计数行不是 states 等于 closed_form、violations=0、exhaustive=true（$(grep -m 1 '^CRASH_POINT_REPLAY ' "$import_copy" | cut -c1-200)）"
  else
    real_cell_outcome=green
    real_cell_detail="$(grep -m 1 '^CRASH_POINT_REPLAY states=' "$import_copy" | cut -d' ' -f1-4)；导入 $(grep -m 1 -oE 'imported_blocks=[0-9]+' "$import_copy")"
  fi
  if [[ "$real_cell_outcome" == red && -d "${runs:-}" ]]; then
    local kept="${TMPDIR:-/tmp}/crash-point-replay-shard-run-selftest-logs-$$"
    mkdir -p -- "$kept" && cp -- "$runs"/*.log "$real/output" "$kept/" 2>/dev/null
    real_cell_detail+="；日志拷到 $kept/ 留着"
  fi
  return 0
}

# run_selftest_cells：跑全部格，红的格名打在 stdout（空格隔开）
run_selftest_cells() {
  local cell red=()
  for cell in "${selftest_cells[@]}"; do
    "cell_${cell//-/_}" || red+=("$cell")
  done
  echo "${red[*]+"${red[*]}"}"
}

if [[ "${1:-}" == --selftest ]]; then
  [[ $# -eq 1 ]] || usage_error "--selftest 不带别的参数"
  selftest_work="$(mktemp -d)"
  trap 'rm -rf -- "${selftest_work:?}"' EXIT
  mkdir -p -- "$selftest_work/root"
  printf '%s' "$FAKE_TWO_HOSTS_DRIVER" > "$selftest_work/fake-driver.sh"
  chmod +x "$selftest_work/fake-driver.sh"
  printf '%s' "$FAKE_PLACEMENT_COMMAND" > "$selftest_work/fake-placement.sh"
  chmod +x "$selftest_work/fake-placement.sh"
  selftest_failed=0
  red_cells="$(CRASH_POINT_REPLAY_SHARD_RUN_BREAK= run_selftest_cells)"
  if [[ -n "$red_cells" ]]; then
    echo "  ✗ crash-point-replay-shard-run 自证：这几格红：$red_cells（最后一趟的输出：$(selftest_last_output))"
    echo "  → 怎么办：对着文件头「用法」与这几格的判法看交给驱动的参数、日志抄回、退出码哪一样变了。"
    selftest_failed=1
  fi
  declare -A switch_cells=([library-not-passed]="library-default library-from-environment" [log-not-copied]="log-copied" [exit-swallowed]="driver-exit-kept" [quiesce-not-passed]="quiesce-when-gpu" [ignore-outer-cap]="outer-memory-cap-to-driver"
    [speed-not-passed]="local-speed-passed" [placement-failure-ignored]="local-speed-unavailable-equal-split" [host-weights-not-passed]="host-weights-passed"
    [real-run-on-live-tree]="real-run-uses-a-frozen-copy" [crates-change-not-voided]="real-run-voided-when-crates-change")
  switch_count=0
  for switch in library-not-passed log-not-copied exit-swallowed quiesce-not-passed ignore-outer-cap speed-not-passed placement-failure-ignored host-weights-not-passed \
      real-run-on-live-tree crates-change-not-voided; do
    switch_count=$((switch_count + 1))
    red_cells="$(CRASH_POINT_REPLAY_SHARD_RUN_BREAK="$switch" run_selftest_cells)"
    echo "  · 开关轮：CRASH_POINT_REPLAY_SHARD_RUN_BREAK=$switch 红的格：${red_cells:-（无）}"
    if [[ "$red_cells" != "${switch_cells[$switch]}" ]]; then
      echo "  ✗ 开关轮：打开 $switch 该红的是「${switch_cells[$switch]}」，实际红的是「$red_cells」"
      echo "  → 怎么办：文件头「弄坏开关」那一张表与 run_merged_log 里 shard_break_is 的分支对一遍。"
      selftest_failed=1
    fi
  done
  # 判法脚本的弄坏开关：把 HOST_JUDGE_WEIGHTS 当不认得的键，红的恰是五格
  switch_count=$((switch_count + 1))
  red_cells="$(CRASH_POINT_REPLAY_SHARD_RUN_BREAK= MULTI_HOST_CONFIGURATION_BREAK=host-weights-unknown run_selftest_cells)"
  echo "  · 开关轮：MULTI_HOST_CONFIGURATION_BREAK=host-weights-unknown 红的格：${red_cells:-（无）}"
  if [[ "$red_cells" != "$selftest_host_weights_cells" ]]; then
    echo "  ✗ 开关轮：打开 MULTI_HOST_CONFIGURATION_BREAK=host-weights-unknown 该红的是「$selftest_host_weights_cells」，实际红的是「$red_cells」"
    echo "  → 怎么办：判法脚本文件头「弄坏开关」那一张表与它里面 host-weights-unknown 的分支对一遍。"
    selftest_failed=1
  fi
  if (( selftest_failed )); then exit 1; fi
  light_summary="${#selftest_cells[@]} 格绿（假的双机驱动：参数、库目录、--quiesce、日志抄回、退出码、发现日志、线程、外层内存上限；假的排计划命令：本机速度交驱动、取不到速度等分；多机配置的 HOST_JUDGE_WEIGHTS：写对、写坏、没写、交驱动 --host-weights；真跑那一格的冻树与 crates 指纹在假的主工作区上演：包装收到副本、中途 crates 变了报本次作废、没变照常判红），开关轮 ${switch_count} 个开关各红在点名的格"
  if [[ -z "${SINGLEFS_HEAVY_TESTS:-}" ]]; then
    echo "  ✓ crash-point-replay-shard-run 自证（假的那几格）通过：$light_summary"
    echo "  ! 真跑那一格 real-two-hosts-run 本次未跑（这不是通过）：命令里没设 SINGLEFS_HEAVY_TESTS；要跑照文件头「真跑那一格」带 SINGLEFS_HEAVY_TESTS=user-request 起 --selftest"
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then echo "crash-point-replay-shard-run.sh --selftest real-two-hosts-run：没设 SINGLEFS_HEAVY_TESTS" >>"$GATE_NOT_RUN_FILE"; fi
    exit 0
  fi
  real_two_hosts_run
  case "$real_cell_outcome" in
    green)
      echo "  ✓ crash-point-replay-shard-run 自证通过：$light_summary；真跑那一格 real-two-hosts-run 绿（$REAL_CELL_CASE_KEY 经真驱动双机跑：录入段真跑、CRASH_LIBRARY_RECORDED 真打、导入之后对账为空；$real_cell_detail），共 $(( ${#selftest_cells[@]} + 1 )) 格"
      exit 0 ;;
    not-run)
      echo "  ! 真跑那一格 real-two-hosts-run 本次未跑（这不是通过）：$real_cell_detail；假的那几格：$light_summary"
      if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then echo "crash-point-replay-shard-run.sh --selftest real-two-hosts-run：$real_cell_detail" >>"$GATE_NOT_RUN_FILE"; fi
      exit 0 ;;
    *)
      echo "  ✗ crash-point-replay-shard-run 自证 real-two-hosts-run：$real_cell_detail"
      echo "  → 怎么办：对着文件头「真跑那一格」的判法看那几份日志：录入段（record.log）、第二台那一份（share-1.log）、导入那一趟（import.log）哪一步没做到；假的那几格：$light_summary"
      exit 1 ;;
  esac
fi

if [[ "${1:-}" == --placement-only ]]; then   # 内部用：shard_measure_local_speed 经清场包装起的排计划那一趟，不给人直接调
  [[ $# -eq 6 ]] || usage_error "--placement-only 后面要五个参数：树根、库、日志、测试目标、用例函数（给了 $(( $# - 1 )) 个）"
  run_placement_only "$2" "$3" "$4" "$5" "$6"; exit $?
fi
[[ "${1:-}" == --merged-log ]] || usage_error "第一个参数要是 --merged-log 或 --selftest（给的是「${1:-}」）"
[[ $# -eq 7 ]] || usage_error "--merged-log 后面要六个参数：键、树根、输入指纹、日志文件、测试目标、用例函数（给了 $(( $# - 1 )) 个）"
run_merged_log "$2" "$3" "$4" "$5" "$6" "$7"
