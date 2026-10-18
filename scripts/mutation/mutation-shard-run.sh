#!/usr/bin/env bash
# 门禁 checker-tier-crates-mutation-replay（crates 变异表复跑）的双机分片驱动：本机库里没有键相同判定的行按用时表分成两堆，本机与第二台各跑一趟 checker-tier-crates-mutation-replay、
# 只判自己那一堆（GATE_MUTATION_ROW_SELECTION）；第二台判出的记录连同覆盖记录当场拷回，经 scripts/mutation/crates-mutation-rows.py import 按行键与判法单元指纹逐条核过，
# 并进本机那一份判定库，第二台这一趟的库随这一趟的树一起删（第二台不留长期库）。判定不在这里：
# 驱动退出之后 checker-tier-crates-mutation-replay 在本机整张判一遍（行键对得上的复用、对不上的现判），判定与单机逐字相同（复用的报数不同）。
#
#   mutation-shard-run.sh <树根>       门禁 checker-tier-crates-mutation-replay（.claude/gate.d/checker-tier-crates-mutation-replay.sh）在本地多机配置写了 ENABLE_ACROSS_MACHINES=1
#                                      （判法 crash-point-replay-shard-configuration-check.sh 退 0）时调（只由 checker-tier-crates-mutation-replay 调，输出进 checker-tier-crates-mutation-replay 的 stderr）；不认环境变量开。
#                                      退 0：两份都跑完、拷回的记录全收，或判法退 3（双机：关，报一行「双机：关」、不连第二台）；
#                                      退 1：有一步没成（没判的行留给 checker-tier-crates-mutation-replay 最后那一趟）；退 2：用法错，或在选行模式下被调，或弄坏开关 MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外
#   mutation-shard-run.sh --selftest   自证：「第二台」是本机上的另一个目录（只供测试的开关 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1），
#                                      假 cargo、真的内存包装，逐格见文件末尾 mutation_shard_selftest 的说明；格数由成功行现算
#
# 步骤：
# ① 配置照 scripts/crash/crash-point-replay-shard-configuration-check.sh --emit-assignments（只调，与崩溃点重放分片同一份）；判不过退 1，原因不在这里打；
#    判法退 3（配置没写 ENABLE_ACROSS_MACHINES=1）报一行「双机：关」、退 0，什么都不分（checker-tier-crates-mutation-replay 在本机整张判）。
# ② 两台工具链相同：rustc -Vv 前三行与 host 行、cargo -V。
#    第二台的 llvm 工具（记覆盖要的 llvm-profdata、llvm-cov，--version 的 LLVM 版本要以第二台 rustc -vV 的「LLVM version」起头；目录取 MUTATION_SHARD_PEER_LLVM_TOOLS，
#    没设取 GATE_MUTATION_LLVM_TOOLS，都没设取第二台 rustc --print sysroot 下 lib/rustlib/<host>/bin）分行之前先查：查不过报一行「第二台缺 llvm-tools」、
#    不分给它、退 0（没判的行交给 checker-tier-crates-mutation-replay 在本机判），第二台这一趟的目录不建。
# ③ 分行（mutation_shard_split_plan）：crates-mutation-rows.py row-plan 列出每一行的行键、这一趟会不会复用、上一次现跑的用时与锚点失效没有；要现判的行
#    （拒算的不算：第二台判出来也没有键可导，留给最后那一趟在本机判；第五列是 anchor-broken 的锚点失效行不算：这一趟不跑，最后那一趟在本机记锚点失效）
#    按用时从长到短，每条放进合计用时少的那一堆（相等放本机），没记过用时的按表里记过的中位数算（一条都没记过都按 1 秒）；本机那一堆与第二台那一堆各写一份选行文件。
#    要现判的行少于 2 行不分，退 0。SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1 时拒（checker-tier-crates-mutation-replay 最后那一趟不读库，分出去的白跑）。
# ④ 树拷到第二台 <PEER_REPOSITORY_DIRECTORY>/mutation-runs/<这一趟>/（新建目录，rsync 不带 --delete，排除 target、.git、规范副本与本地配置 multi-host.env），
#    规范副本另拷（树里有拷树里的，没有拷驱动所在仓的，再没有拷那份仓的主仓的），拷完回读；在那里 git init；
#    两台各算 crates-mutation-rows.py judging-units（判法单元指纹）与 toolchain（工具链行：rustc -vV、cargo -V、构建环境解析后的值），哪一样不同就拒
#    （行键里有这两样，不同的话第二台判出的记录一条都导不进来）；源码两台是同一棵树拷的，每一行的键由 ⑦ 的 import 照本机此刻的源码逐条核，这里不比源码。
# ⑤ 清场照配置做；跑完、跑红、收到信号都不复原（EXIT 上只报一句「清了场、没复原」，要复原由用户手动跑私有复原脚本）；清场的命令与输出都不打。
# ⑥ 两台同时跑：本机 GATE_MUTATION_ROW_SELECTION=<本机那一堆> bash <checker-tier-crates-mutation-replay> <树根>；第二台同一条只判它那一堆，脱离 ssh 会话起（setsid nohup），
#    进程号、退出码、输出各写第二台一个文件，驱动每 MUTATION_SHARD_RUN_POLL_SECONDS（默认 5）秒经 ssh 看一次；连续 60 次连不上、
#    进程没了又没留退出码、60 次都没起来，就不等了。第二台那一份整份经那棵树副本里的 run-with-memory-cap.sh 起
#    （上限 MUTATION_SHARD_PEER_MEMORY_CAP，默认 4G，键固定为「mutation-shard-run.sh 第二台那一份」：罩的是 checker-tier-crates-mutation-replay 自己在每条变异之外的进程；
#    不用配置的 PEER_MEMORY_CAP：那是崩溃点重放一片测试二进制的上限，拿它罩整份 checker-tier-crates-mutation-replay，它在第二台的账上就一直占着这么多、每条变异排在它后面），
#    每条变异在里面照 checker-tier-crates-mutation-replay 各自经包装，上限与限时与本机同一套（它们不进行键，只记在超时、撞顶那两档的记录里）。
#    第二台那一份的判定库不设 GATE_MUTATION_RECORDS_HOME：照默认放在这一趟的树的 git common-dir 里，每趟新建、⑦ 拷回、⑧ 随树删（第二台不留长期库）。
#    第二台跨趟保留的只有缓存，放 <PEER_REPOSITORY_DIRECTORY>/mutation-kept/：编译目录（GATE_MUTATION_TARGET_DIR=target）、峰值表（RUN_WITH_MEMORY_CAP_PEAKS=memory-peaks.tsv）、
#    变异复跑的崩溃判定库家目录（GATE_MUTATION_CRASH_LIBRARY_HOME=crash-library-home：变异版上走判定库的用例写这一份，不写进第二台真跑的长期库，
#    crates-mutation-rows.py 文件头「变异复跑的判定库」），都不在这一趟会删的树里。本机那一份不另设：共用模块照本机真跑家目录底下的 mutation-replay/。
#    跨趟复用的编译目录不接 peer-host-lib.sh 的 peer_host_shared_target_guard（不按 rsync -a 带来的旧 mtime 刷源码戳）：cargo 不在这棵树里起，起在每个工作进程自己的副本里，
#    副本由 crates-mutation-rows.py 的 prepare_shard 用 shutil.copy 拷、修改时刻取拷的那一刻（它的 --selftest 第十二格拿旧 mtime 的源文件证这一条），
#    几趟同时用同一个编译目录由 claim_worker_slot 的片号锁分开（-w<片号>，弄坏开关 GATE_MUTATION_BREAK=local-shard-numbering）。
#    带到第二台的环境：SINGLEFS_HEAVY_TESTS 原样；GATE_MUTATION_MEMORY_MAX、GATE_MUTATION_TIMEOUT、GATE_MUTATION_LONG_ROW_SECONDS、
#    GATE_MUTATION_BREAK、RUN_WITH_MEMORY_CAP_RESERVE、GATE_MUTATION_LLVM_TOOLS 设了就带（后者是 llvm 工具的目录，第二台要在同一个路径上有同一份；
#    设了 MUTATION_SHARD_PEER_LLVM_TOOLS 就给第二台换成它）；本机别的环境变量不带（真的第二台经 ssh 也拿不到）。
# ⑦ 第二台这一趟的判定库（那棵树的 git common-dir 底下，crates-mutation-rows.py library-directory 报的那一个；records/、baselines/、coverage/、compositions/、units/）
#    整个拷回本机，交 crates-mutation-rows.py import 逐条核文件名、判法单元指纹与行键（键是本机变异表此刻某一行照本机此刻的源码算得出的；跨机不比主机名），
#    符的并进本机那一份库，不符的由它列出。第二台那一份中途挂了、ssh 断了，已有的记录照样拷回导入，没判的行留给 checker-tier-crates-mutation-replay 最后那一趟。
# ⑧ 删第二台这一趟的树（连同它这一趟的判定库）与进程号、退出码、输出、选行文件（mutation-kept 下的缓存留着）；④ 之后在哪一步失败退出，这一步都挂在 EXIT 上照做。
# 收到 TERM / INT / HUP（退 143 / 130 / 129）：先停两份——本机那一份从它的进程号起，第二台那一份从它在第二台上记下的进程号起（核它的工作目录是这一趟的树），
#   都连同全部后代逐个按进程号冻住、从最底层起 TERM、放开，至多等 30 秒，还在的 KILL，回读一个不剩——再照 EXIT 上那一套删第二台这一趟的树，做完才退。
#   前台那几步（rsync、算判法单元指纹与工具链行）里收到的，等那一步跑完再收尾；KILL 接不住，那时第二台上这一趟的树留着，手动删。
# 两份的输出存进 <树根的 git common-dir>/singlefs-mutation-shard-logs/<这一趟>/（local.log、peer.log，只留最新 3 趟）。
# 驱动打的每一行、存下的输出都不写配置里的主机、目录与清场命令：第二台上的错误输出与 peer.log 经 scripts/hosts/peer-host-lib.sh 的 peer_host_redact 换成「第二台」。
# 第二台共用的那几段（跑命令、拷文件、脱离会话起与轮询、按进程号停、清场）在 scripts/hosts/peer-host-lib.sh。
# 重型：真跑是两台各跑一部分 crates/mutations.tsv，与 checker-tier-crates-mutation-replay 同一类（.claude/hooks/lib_heavy_tests.py 按名字认它）；--selftest 不算重型，在 .claude/hooks/selftest-registry.tsv 登记为 light，改了就跑（同一份判定）。
#
# 弄坏开关 MUTATION_SHARD_RUN_BREAK=<项>（只给证红用，逗号分隔可给几个；打开时自证退 1）。只在 --selftest 起的进程里生效：自证在它的临时目录里放一份标记文件，
#   路径经 MUTATION_SHARD_RUN_SELFTEST_MARK 带给它起的驱动与 checker-tier-crates-mutation-replay（驱动起的本机那一份也带着），
#   文件在、内容是 MUTATION_SHARD_SELFTEST_MARK_CONTENT 那一行才认开关；别处设了它，驱动拒、退 2，checker-tier-crates-mutation-replay 判红，什么都没判。
#   逐项写开关名、弄坏哪一处、打开后自证哪几格红
# （项名是自证 expect 的格名，一行一项；失败行是「✗ <格名>：<细节>」，括号里抄细节的句首，随跑变的路径、进程号、指纹写成 …）：
#   MUTATION_SHARD_RUN_BREAK=import-skipped  拷回的库不经 import、直接放进本机那一份库
#       ① 两台各判一份：起的变异不相交、合起来是全部 6 行、两边都不空（「本机「」里有一行判了两次：拒掉的那一条没拒，最后那一趟在本机补跑）
#       ① 最后那一趟全部复用（复用抓到 6 条、现跑变异至多 0 条）（「stderr 里的判定库记录行：  … 判定库记录：复用抓到 5 条」）
#       ② import 拒了行键对不上的那一条、逐条列出（「stderr 里导入那几行：」后面是空的）
#   MUTATION_SHARD_RUN_BREAK=no-final-pass  checker-tier-crates-mutation-replay 调完驱动不再整张判一遍（checker-tier-crates-mutation-replay 读这一项）
#       ① checker-tier-crates-mutation-replay 接驱动那一趟判绿，判定与单机那一趟逐字相同（「退 1；diff：」）
#       ① 最后那一趟全部复用（复用抓到 6 条、现跑变异至多 0 条）（「stderr 里的判定库记录行：」后面是空的）
#       ① 第二台判的行连同覆盖记录导入（「覆盖那一行：」后面是空的）
#       ③ 第二台那一份判完 1 条就停：第二台那一份退 1，checker-tier-crates-mutation-replay 最后那一趟复用 4 条、补跑缺的 2 行，判定仍与单机逐字相同（「退 0；判定库记录行：」）
#       ④ 驱动拒了之后 checker-tier-crates-mutation-replay 在本机整张判，判定与单机逐字相同（「退 1」）
#   MUTATION_SHARD_RUN_BREAK=environment-opens  checker-tier-crates-mutation-replay 见 GATE_MUTATION_ACROSS_MACHINES=1 照旧把双机打开、不看配置的开关（checker-tier-crates-mutation-replay 读这一项）
#       单机那一趟（配置没开双机、环境变量 GATE_MUTATION_ACROSS_MACHINES=1）判绿、报了双机：关、没调驱动（「退 0；stderr 尾部：」）
#   MUTATION_SHARD_RUN_BREAK=switch-off-ignored  判法退 3（双机：关）当成判不过、退 1
#       ⑧ 驱动直接起、配置没写 ENABLE_ACROSS_MACHINES=1：报一行双机：关、退 0、第二台这一趟的目录没建（「退 1；输出：  ✗ 双机分片的本地配置判不过」）
#   MUTATION_SHARD_RUN_BREAK=peer-without-memory-cap  第二台那一份不经 run-with-memory-cap.sh 起
#       ① 第二台那一份经 run-with-memory-cap.sh 起：跨趟保留的峰值表里有它的键（「跨趟保留的峰值表 8 行，没有键「mutation-shard-run.sh 第二台那一份」那一行」）
#   MUTATION_SHARD_RUN_BREAK=no-peer-cleanup-on-failure  ④ 之后失败退出（含收到信号）时不删第二台这一趟的树
#       ④ 两台的构建环境不同：驱动拒、两份都没起、第二台这一趟的树删了（「stderr：  ✗ 双机分片：两台的工具链行（含构建环境）不同」）
#       ⑥ 两份都在跑时收到 TERM：退 143，两份卡住的假 cargo 都不在了，没复原，第二台这一趟的树删了（「退 143；卡住的假 cargo 2 个（…），还活着的「」；第二台留着 1 个目录」）
#   MUTATION_SHARD_RUN_BREAK=peer-library-kept  第二台那一份的判定库放进跨趟保留的 mutation-kept/records（改前的样子：第二台留一份长期库），拷回的是这一趟的树里那一份（空的）
#       ① 两台各判一份（「本机「」里是全部 6 行」：第二台判的一条都没导进来，最后那一趟在本机补跑）
#       ① 最后那一趟全部复用（复用抓到 6 条、现跑变异至多 0 条）（「stderr 里的判定库记录行：  … 判定库记录：复用抓到 3 条」）
#       ① 第二台这一趟的树（连同它这一趟的判定库）删了，mutation-kept 下留着编译目录与峰值表、没有判定库（「mutation-kept 底下：」里有 records）
#       ① 第二台判的行连同覆盖记录导入……最后那一趟一份覆盖都没补记（「覆盖那一行：  … 覆盖：这一趟记了 3 份」）
#       ③ 第二台那一份判完 1 条就停……最后那一趟复用 4 条、补跑缺的 2 行（「判定库记录行：  … 复用抓到 3 条」）
#   MUTATION_SHARD_RUN_BREAK=peer-crash-library-home-unset  不给第二台那一份设 GATE_MUTATION_CRASH_LIBRARY_HOME（改前的样子：变异版的结论照第二台真跑家目录底下的 mutation-replay/ 写，不跨趟留在 mutation-kept 下）
#       ① 第二台那一份起 cargo 用 mutation-kept 下变异复跑自己的崩溃判定库家目录（「第二台那一份的输出里崩溃判定库家目录那一行：」）
#   MUTATION_SHARD_RUN_BREAK=break-honored-anywhere  弄坏开关在 --selftest 之外也认（驱动与 checker-tier-crates-mutation-replay 都读这一项）
#       ⑨ MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外（不带自证的标记）：驱动拒、退 2，说设在 --selftest 之外（「退 0；输出：」）
#       ⑩ MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外：checker-tier-crates-mutation-replay 判红（退 1）、说设在 --selftest 之外（「退 0；输出：」）
#   MUTATION_SHARD_RUN_BREAK=gpu-memory-unexplained  第二台那一份退 245 时不报显存不够，只报「第二台那一份 exit 245」
#       ⑪ 第二台那一份退 245 ⇒ 报显存不够、不算结果；退 252 ⇒ 报内存包装自己的结局；退 3 ⇒ 不报（「退 245 报的「第二台那一份 exit 245」；…」）
#   MUTATION_SHARD_RUN_BREAK=store-not-fetched  拷回时不带第二台的覆盖记录（coverage/）
#       ① 第二台判的行连同覆盖记录导入（「抓到的记录 6 条、覆盖记录 3 份」）
#   MUTATION_SHARD_RUN_BREAK=peer-llvm-tools-unchecked  分行之前不查第二台的 llvm 工具（改前的样子）
#       ⑬ 第二台缺 llvm-tools：分片前先查，报一行第二台缺 llvm-tools、退 0、不分给它，第二台这一趟的目录没建（「退 1；输出：」）
#   MUTATION_SHARD_RUN_BREAK=anchor-broken-dispatched  ③ 分行不看 row-plan 的第五列：锚点失效、而行键算得出的那一行照样分出去
#       ⑭ ③ 分行：锚点失效的行（第五列 anchor-broken，有键没键都算）不进两堆、单报几行（「本机「」第二台「」；汇总「」」里第 4 行进了一堆）
#   MUTATION_SHARD_RUN_BREAK=selftest-tmpdir-outside  自证不把 TMPDIR 换成自己临时目录下的 tmp/，场景照调用方的 TMPDIR（没设是 /tmp）建临时目录
#       ⑫ 跑完外面的临时目录里没多出 singlefs-mutation-replay-*（要连同 GATE_MUTATION_BREAK=shardrootkeptonsignal 一起开：
#          那样被 TERM 停的那一份不删它，「外面多出来的」后面不空；只开 crates-mutation-rows.py 那一个时「自证自己的 tmp/ 里剩下的」后面不空）
#   共用库的 PEER_HOST_LIB_BREAK=no-redact  peer_host_redact 原样转出、不换私有的字
#       ① 输出里没有配置的主机、第二台的目录与清场命令（「…/singlefs-mutation-shard-logs/…/peer.log 里有」）
# 只供测试的开关：SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1（照崩溃点重放分片）；MUTATION_SHARD_RUN_PEER_STOP_AFTER_ROWS=<k>
#   给第二台那一份设 GATE_MUTATION_STOP_AFTER_ROWS=k 与 GATE_MUTATION_WORKERS=1（造「第二台那一份中途停了」：一个工作进程，判完 k 条就不再领活）；
#   MUTATION_SHARD_RUN_SELFTEST_PLANT=<文件>（只在 --selftest 起的进程里认，同弄坏开关的标记）⑦ 拷回之后把这份文件照原名放进拷回来的 records/（造「另一台交来一条键对不上的记录」）。
# 配置的开关：MUTATION_SHARD_PEER_LLVM_TOOLS=<第二台上的目录> 第二台的 llvm 工具不在 GATE_MUTATION_LLVM_TOOLS 那个路径上时设它（② 查它、⑥ 给第二台那一份换成它）。
#
# admission: always 每次调都分这一刻这棵树上没有作数记录的行；复用判定在判定库记录里，不在这里
# run-condition: command python3 rsync ssh git cargo rustc
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
mutation_shard_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=peer-host-lib.sh
source "$mutation_shard_script_directory/../hosts/peer-host-lib.sh"

MUTATION_SHARD_PEER_WRAPPER_KEY="mutation-shard-run.sh 第二台那一份"

mutation_shard_break_is() { [[ ",${MUTATION_SHARD_RUN_BREAK:-}," == *",$1,"* ]]; }

# 自证起的进程的标记（文件头「弄坏开关」那一段）；checker-tier-crates-mutation-replay 里抄着同一行，改的时候两处一起改
MUTATION_SHARD_SELFTEST_MARK_CONTENT="mutation-shard-run.sh --selftest 起的进程"
mutation_shard_inside_selftest() {
  [[ -n "${MUTATION_SHARD_RUN_SELFTEST_MARK:-}" && -f "$MUTATION_SHARD_RUN_SELFTEST_MARK" ]] \
    && [[ "$(head -n 1 -- "$MUTATION_SHARD_RUN_SELFTEST_MARK")" == "$MUTATION_SHARD_SELFTEST_MARK_CONTENT" ]]
}

mutation_shard_peer_problem() { # <第二台那一份的结局，「exit <码>」或别的>：那一份没成就打一句原因，成了（退 1 或 3）什么都不打
  case "$1" in
    "exit 1"|"exit 3") ;;
    "exit 245")
      if mutation_shard_break_is gpu-memory-unexplained; then
        echo "第二台那一份 $1"
      else
        echo "第二台那一份退 245：显存不够（那边的内存包装认出起止之间内核报了 NV_ERR_NO_MEMORY），那一份的输出不算结果；到第二台 nvidia-smi 看谁在用卡，等它跑完再重跑"
      fi
      ;;
    "exit 25"[0-4]) echo "第二台那一份${1#exit }：内存包装自己的结局（run-with-memory-cap.sh 文件头「退出码」），那一份的输出不算结果" ;;
    *) echo "第二台那一份 $1" ;;
  esac
}

mutation_shard_llvm_check_command() { # 打在第二台跑的那一段：llvm-profdata、llvm-cov 在、llvm-cov 的 LLVM 版本以第二台 rustc 的起头（文件头 ②）；查得过退 0 打一句版本，查不过退 3 打为什么
  printf 'directory=%q\n' "${MUTATION_SHARD_PEER_LLVM_TOOLS:-${GATE_MUTATION_LLVM_TOOLS:-}}"
  cat <<'LLVM_CHECK'
if [[ -z "$directory" ]]; then directory="$(rustc --print sysroot)/lib/rustlib/$(rustc -vV | sed -n 's/^host: //p')/bin"; fi
for tool in llvm-profdata llvm-cov; do [[ -x "$directory/$tool" ]] || { echo "$tool 不在或不能执行"; exit 3; }; done
wanted="$(rustc -vV | sed -n 's/^LLVM version: //p')"
[[ -n "$wanted" ]] || { echo "第二台 rustc -vV 读不出 LLVM 版本"; exit 3; }
found="$("$directory/llvm-cov" --version | sed -n 's/.*LLVM version \([^ ]*\).*/\1/p' | head -n 1)"
case "$found" in
  "$wanted"|"$wanted"-*) echo "llvm-cov 的 LLVM $found，rustc 的 $wanted" ;;
  *) echo "llvm-cov 的 LLVM 版本「${found:-读不出}」与第二台 rustc 的「${wanted:-读不出}」对不上"; exit 3 ;;
esac
LLVM_CHECK
}

mutation_shard_split_plan() { # <row-plan 的输出> <本机选行文件> <第二台选行文件> <键文件>：③ 分两堆、写选行文件与键文件，stdout 打一行汇总
  # 汇总：<表里的行数> <要现判的> <记过用时的> <没记过的按几秒> <本机行数> <本机秒数> <第二台行数> <第二台秒数> <拒算的> <锚点失效的>
  # 要现判 = 不复用、有键、第五列不是 anchor-broken（锚点失效的行这一趟不跑，crates-mutation-rows.py 文件头「锚点失效」）
  local anchor_dispatched=""
  mutation_shard_break_is anchor-broken-dispatched && anchor_dispatched=1
  MUTATION_SHARD_ANCHOR_DISPATCHED="$anchor_dispatched" python3 - "$@" <<'PY_SPLIT'
import os, statistics, sys
plan_path, local_path, peer_path, keys_path = sys.argv[1:]
anchor_dispatched = os.environ.get("MUTATION_SHARD_ANCHOR_DISPATCHED") == "1"   # 弄坏开关 anchor-broken-dispatched：不看第五列
with open(plan_path, encoding="utf-8") as handle:
    rows = [line.rstrip("\n").split("\t") for line in handle if line.strip()]
with open(keys_path, "w", encoding="utf-8") as handle:
    handle.write("".join(row[1] + "\n" for row in rows))
def anchor_broken(row):
    return not anchor_dispatched and len(row) > 4 and row[4] == "anchor-broken"
known = [float(row[3]) for row in rows if row[3] != "-"]
default = statistics.median(known) if known else 1.0
pending = sorted(((float(row[3]) if row[3] != "-" else default, int(row[0])) for row in rows if row[2] == "0" and row[1] != "-" and not anchor_broken(row)),
                 key=lambda item: (-item[0], item[1]))
heaps = [[0.0, []], [0.0, []]]
for seconds, line_number in pending:
    lighter = heaps[0] if heaps[0][0] <= heaps[1][0] else heaps[1]
    lighter[0] += seconds
    lighter[1].append(line_number)
for path, (_total, lines) in zip((local_path, peer_path), heaps):
    with open(path, "w", encoding="utf-8") as handle:
        handle.write("".join(f"{line_number}\n" for line_number in sorted(lines)))
broken = sum(1 for row in rows if anchor_broken(row))
refused = sum(1 for row in rows if row[1] == "-" and not anchor_broken(row))
print(len(rows), len(pending), len(known), f"{default:.1f}", len(heaps[0][1]), f"{heaps[0][0]:.0f}", len(heaps[1][1]), f"{heaps[1][0]:.0f}", refused, broken)
PY_SPLIT
}

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
  peer_host_quiesce_left
  if (( peer_run_pending )); then
    if mutation_shard_break_is no-peer-cleanup-on-failure; then
      echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=no-peer-cleanup-on-failure：失败退出，第二台这一趟的树不删"
    else
      mutation_shard_remove_peer_run
    fi
  fi
  if [[ -n "${mutation_shard_scratch:-}" ]]; then rm -rf -- "${mutation_shard_scratch:?}"; fi
}

mutation_shard_stop_and_exit() { # <信号名> <退出码>：先停两份，再 exit（EXIT 上删第二台这一趟的树，不复原）
  local stop_output
  trap '' TERM INT HUP PIPE
  echo "  ! 收到 $1：先停两份，再删第二台这一趟的树（不复原），做完再退"
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
  rows_module="$stage_repository/scripts/mutation/crates-mutation-rows.py"
  if [[ -z "${GATE_MUTATION_MEMORY_CAP_RUNNER:-}" ]]; then GATE_MUTATION_MEMORY_CAP_RUNNER="$stage_repository/scripts/hosts/run-with-memory-cap.sh"; fi
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
    mutation_shard_fail "SINGLEFS_GATE_FULL=1 或 GATE_MUTATION_START_OVER=1：checker-tier-crates-mutation-replay 最后那一趟不复用判定库记录，分出去判的行它还要再跑一遍" "这一趟单机跑（checker-tier-crates-mutation-replay 在这两个开关下不调驱动）"
  fi

  # ① 配置
  local configuration_exit=0
  peer_host_load_configuration "$mutation_shard_script_directory/../crash/crash-point-replay-shard-configuration-check.sh" "$tree_root" || configuration_exit=$?
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
    echo "  ! 第二台是本机上的另一个目录（只供测试的开关 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1）"
  fi
  mutation_shard_scratch="$(mktemp -d)"
  trap mutation_shard_on_exit EXIT
  trap 'mutation_shard_stop_and_exit TERM 143' TERM
  trap 'mutation_shard_stop_and_exit INT 130' INT
  trap 'mutation_shard_stop_and_exit HUP 129' HUP

  # ② 工具链
  peer_host_compare_toolchains || exit 1
  echo "  · ② 两台的工具链相同（${peer_host_toolchain_text}）"
  # ② 第二台的 llvm 工具（记覆盖要用）：分行之前先查，查不过不分给它（文件头 ②）
  local peer_llvm_report
  if mutation_shard_break_is peer-llvm-tools-unchecked; then
    echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=peer-llvm-tools-unchecked：不查第二台的 llvm 工具"
  elif peer_llvm_report="$(peer_host_run / "$(mutation_shard_llvm_check_command)" 2>&1)"; then
    echo "  · ② 第二台的 llvm 工具在（${peer_llvm_report##*$'\n'}）"
  else
    echo "  · 双机：第二台缺 llvm-tools（${peer_llvm_report##*$'\n'}）：不分给它，没判的行交给 checker-tier-crates-mutation-replay 在本机判；要分就在第二台装 rustup component add llvm-tools（与它的 rustc 同一份 LLVM），或设 MUTATION_SHARD_PEER_LLVM_TOOLS=<第二台上的目录>"
    exit 0
  fi

  # ③ 分行
  local plan_file="$mutation_shard_scratch/plan.tsv" split_summary
  python3 "$rows_module" row-plan "$tree_root" > "$plan_file" \
    || mutation_shard_fail "本机列不出每一行的记录与用时（crates-mutation-rows.py row-plan 退非 0：$(tail -2 "$plan_file" | tr '\n' ' ')）" "单跑 python3 scripts/mutation/crates-mutation-rows.py row-plan <树根> 看它报什么"
  split_summary="$(mutation_shard_split_plan "$plan_file" "$mutation_shard_scratch/local.selection" "$mutation_shard_scratch/peer.selection" "$mutation_shard_scratch/keys")" \
    || mutation_shard_fail "分行那一段 python 没跑成" "看上面 python 的报错"
  local table_rows pending_rows timed_rows default_seconds local_rows local_seconds peer_rows peer_seconds refused_rows anchor_broken_rows
  read -r table_rows pending_rows timed_rows default_seconds local_rows local_seconds peer_rows peer_seconds refused_rows anchor_broken_rows <<< "$split_summary"
  if (( pending_rows < 2 )); then
    echo "  · ③ 变异表 $table_rows 行里要现判的只有 $pending_rows 行（拒算的 $refused_rows 行、锚点失效的 $anchor_broken_rows 行不分）：不分，交给 checker-tier-crates-mutation-replay 在本机判"
    exit 0
  fi
  echo "  · ③ 变异表 $table_rows 行里要现判的 $pending_rows 行按用时两堆均分（记过用时的 $timed_rows 行，没记过的按中位数 $default_seconds 秒算；拒算的 $refused_rows 行、锚点失效的 $anchor_broken_rows 行不分，留给最后那一趟）：本机 $local_rows 行约 $local_seconds 秒，第二台 $peer_rows 行约 $peer_seconds 秒"

  # ④ 树拷到第二台、两台各算判法单元指纹与工具链行
  local peer_runs="$PEER_REPOSITORY_DIRECTORY/mutation-runs" peer_kept sop_source local_base peer_base
  peer_run_directory="$peer_runs/$run_label"
  peer_pid_file="$peer_runs/$run_label.pid"
  peer_exit_file="$peer_runs/$run_label.exit"
  peer_output_file="$peer_runs/$run_label.log"
  peer_selection_file="$peer_runs/$run_label.selection"
  peer_kept="$PEER_REPOSITORY_DIRECTORY/mutation-kept"
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
  local_base="$(python3 "$rows_module" judging-units "$tree_root")" \
    || mutation_shard_fail "本机算不出判法单元指纹：$local_base" "单跑 python3 scripts/mutation/crates-mutation-rows.py judging-units <树根> 看它报什么"
  peer_base="$(peer_host_run "$peer_run_directory" "python3 scripts/mutation/crates-mutation-rows.py judging-units .")" \
    || mutation_shard_fail "第二台算不出判法单元指纹" "在第二台那棵树里单跑同一条 judging-units 看它报什么"
  local_base="${local_base%% *}"
  peer_base="${peer_base%% *}"
  [[ "$local_base" == "$peer_base" ]] \
    || mutation_shard_fail "两台算出的判法单元指纹不同（本机 ${local_base:0:16}…，第二台 ${peer_base:0:16}…）：两棵树里判「抓没抓到」的那几个函数不一样" \
      "让第二台那棵树与本机被判的树同一份（树是这一趟现拷的，多半是本机被判的树与驱动所在的仓不同）"
  local local_toolchain peer_toolchain
  local_toolchain="$(python3 "$rows_module" toolchain "$tree_root")" \
    || mutation_shard_fail "本机算不出工具链行：$local_toolchain" "单跑 python3 scripts/mutation/crates-mutation-rows.py toolchain <树根> 看它报什么"
  peer_toolchain="$(peer_host_run "$peer_run_directory" "python3 scripts/mutation/crates-mutation-rows.py toolchain .")" \
    || mutation_shard_fail "第二台算不出工具链行" "在第二台那棵树里单跑同一条 toolchain 看它报什么"
  if [[ "$local_toolchain" != "$peer_toolchain" ]]; then
    mutation_shard_fail "两台的工具链行（含构建环境）不同：$(diff <(printf '%s\n' "$local_toolchain") <(printf '%s\n' "$peer_toolchain") | grep '^[<>]' | head -3 | tr '\n' ' ')；行键里有它，第二台判出的记录一条都导不进来" \
      "对一对两台的 cargo 配置（~/.cargo/config*、目录层级里的 .cargo/config*）与 RUSTFLAGS 这一类环境变量，改成一样再跑"
  fi
  echo "  · ④ 两台的判法单元指纹（${local_base:0:16}…）与工具链行都相同"

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
  for carried_variable in GATE_MUTATION_MEMORY_MAX GATE_MUTATION_TIMEOUT GATE_MUTATION_LONG_ROW_SECONDS GATE_MUTATION_BREAK RUN_WITH_MEMORY_CAP_RESERVE GATE_MUTATION_LLVM_TOOLS; do
    if [[ -n "${!carried_variable:-}" ]]; then peer_command+=" $carried_variable=$(printf '%q' "${!carried_variable}")"; fi
  done
  if [[ -n "${MUTATION_SHARD_PEER_LLVM_TOOLS:-}" ]]; then peer_command+=" GATE_MUTATION_LLVM_TOOLS=$(printf '%q' "$MUTATION_SHARD_PEER_LLVM_TOOLS")"; fi
  if [[ -n "${MUTATION_SHARD_RUN_PEER_STOP_AFTER_ROWS:-}" ]]; then
    peer_command+=" GATE_MUTATION_WORKERS=1 GATE_MUTATION_STOP_AFTER_ROWS=$(printf '%q' "$MUTATION_SHARD_RUN_PEER_STOP_AFTER_ROWS")"
  fi
  # 第二台那一份的判定库照默认放在这一趟的树的 git common-dir 里，⑦ 拷回、⑧ 随树删（文件头 ⑥）
  peer_command+=" GATE_MUTATION_ROW_SELECTION=$(printf '%q' "$peer_selection_file")"
  if mutation_shard_break_is peer-library-kept; then
    echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=peer-library-kept：第二台那一份的判定库放进跨趟保留的 mutation-kept/records（改前的样子）"
    peer_command+=" GATE_MUTATION_RECORDS_HOME=$(printf '%q' "$peer_kept/records")"
  fi
  peer_command+=" GATE_MUTATION_TARGET_DIR=$(printf '%q' "$peer_kept/target") RUN_WITH_MEMORY_CAP_PEAKS=$(printf '%q' "$peer_kept/memory-peaks.tsv")"
  # 变异复跑的崩溃判定库家目录（文件头 ⑥）：第二台那一份写 mutation-kept 下自己的一份，不写第二台真跑的长期库
  mutation_shard_break_is peer-crash-library-home-unset || peer_command+=" GATE_MUTATION_CRASH_LIBRARY_HOME=$(printf '%q' "$peer_kept/crash-library-home")"
  if mutation_shard_break_is peer-without-memory-cap; then
    echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=peer-without-memory-cap：第二台那一份不经 run-with-memory-cap.sh 起"
    peer_share_text="不经内存包装"
  else
    peer_command+=" RUN_WITH_MEMORY_CAP_KEY=$(printf '%q' "$MUTATION_SHARD_PEER_WRAPPER_KEY") bash scripts/hosts/run-with-memory-cap.sh $(printf '%q' "$peer_wrapper_cap")"
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
  local peer_problem
  peer_problem="$(mutation_shard_peer_problem "$peer_host_detached_outcome")"
  [[ -z "$peer_problem" ]] || problems+=("$peer_problem")
  if [[ "$local_exit" == 1 || "$peer_host_detached_outcome" != "exit 3" ]]; then
    echo "    本机那一份的尾部（全文 $local_log）："; tail -6 "$local_log" | sed 's/^/      /'
    if [[ -f "$peer_log" ]]; then echo "    第二台那一份的尾部（全文 $peer_log）："; tail -6 "$peer_log" | sed 's/^/      /'; fi
  fi

  # ⑦ 第二台这一趟的判定库拷回、核过再并进本机那一份（文件头 ⑦）；第二台那一份照默认把库放在这一趟的树的 git common-dir 里
  local peer_library fetched="$mutation_shard_scratch/fetched" record_count=0 baseline_count=0 coverage_count=0 import_exit=0 local_library store_directory
  mkdir -p -- "$fetched"
  if ! peer_library="$(peer_host_run "$peer_run_directory" "python3 scripts/mutation/crates-mutation-rows.py library-directory ." 2>/dev/null)" || [[ -z "$peer_library" ]]; then
    peer_library="$peer_run_directory/.git/singlefs-crates-mutation-library"
  fi
  if ! peer_host_copy "$(peer_host_path "$peer_library")/" "$fetched/" 2>/dev/null; then
    echo "  ! 第二台这一趟的判定库拷不回来（那一份一条都没判完，或 ssh 断了）"
  fi
  # 覆盖记录一并带上（基线复用按它判）；弄坏开关 store-not-fetched 下不带
  if mutation_shard_break_is store-not-fetched && [[ -d "$fetched/coverage" ]]; then
    echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=store-not-fetched：第二台的 coverage/ 不带过来"
    rm -rf -- "${fetched:?}/coverage"
  fi
  # 只供自证：往拷回来的 records/ 里放一条名字是本机某一行的键、内容的行键是别的的记录，看 import 拒它（标记文件在、内容对得上才认）
  if [[ -n "${MUTATION_SHARD_RUN_SELFTEST_PLANT:-}" ]] && mutation_shard_inside_selftest && [[ -f "$MUTATION_SHARD_RUN_SELFTEST_PLANT" ]]; then
    mkdir -p -- "$fetched/records" && cp -- "$MUTATION_SHARD_RUN_SELFTEST_PLANT" "$fetched/records/$(basename "$MUTATION_SHARD_RUN_SELFTEST_PLANT")"
  fi
  for store_directory in records baselines coverage; do
    if [[ -d "$fetched/$store_directory" ]]; then
      case "$store_directory" in
        records) record_count="$(find "$fetched/records" -maxdepth 1 -type f | wc -l)" ;;
        baselines) baseline_count="$(find "$fetched/baselines" -maxdepth 1 -type f | wc -l)" ;;
        coverage) coverage_count="$(find "$fetched/coverage" -maxdepth 1 -type f | wc -l)" ;;
      esac
    fi
  done
  echo "  · ⑦ 拷回第二台这一趟的判定库：记录 $record_count 条、基线 $baseline_count 条、覆盖 $coverage_count 份"
  if mutation_shard_break_is import-skipped; then
    echo "  ! 弄坏开关 MUTATION_SHARD_RUN_BREAK=import-skipped：拷回的库不经 import，直接放进本机那一份库"
    if local_library="$(python3 "$rows_module" library-directory "$tree_root")" && mkdir -p -- "$local_library"; then
      cp -r -- "$fetched"/. "$local_library/" 2>/dev/null
    fi
  elif (( record_count + baseline_count + coverage_count > 0 )); then
    if python3 "$rows_module" import "$tree_root" "$fetched"; then import_exit=0; else import_exit=$?; fi
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
# 在临时目录里搭一棵小仓（本仓的 .claude/scripts、checker-tier-crates-mutation-replay 与登记表、scripts 整个目录、规范副本的符号链接，加一个 crates/tiny 与六行变异表），
# 「第二台」是本机上的另一个目录（SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1），配置写在临时目录里（SINGLEFS_MULTI_HOST_CONFIG）。
# 假 cargo（本机一份、第二台 PATH 上一份，各往自己目录下的 invocations.log 记起了哪几条）照 lib.rs 里「// TEST <名> NEEDS <原文>」判 ok / FAILED；
# 每条变异照真的走 run-with-memory-cap.sh（上限 256M，排队与账都是真的）。逐格：
#   单机那一趟：配置没写 ENABLE_ACROSS_MACHINES=1、环境里 GATE_MUTATION_ACROSS_MACHINES=1（原来的开法）：checker-tier-crates-mutation-replay 判绿、报「双机：关」、不调驱动（它的 stdout 是下面比对的基准）
#   ① 主流程（checker-tier-crates-mutation-replay 接驱动）：退 0；判定与单机那一趟逐字相同；两台起的变异不相交、合起来是全部、都不空；
#      最后一趟全部复用；本机那一份（选行模式的 checker-tier-crates-mutation-replay）没再调驱动；清了场、跑完没复原；
#      第二台这一趟的树连同它这一趟的判定库删了，mutation-kept 下只留编译目录与峰值表（第二台不留长期库）；第二台判的记录连同覆盖记录并进本机那一份库；
#      第二台那一份经内存包装起（跨趟保留的峰值表里有它的键）、每条变异的峰值也记在那里；输出里没有配置的主机、第二台的目录与清场命令
#   ② import 拒行键对不上的：拷回来的 records/ 里另放一条名字是本机那一堆第一行的键、内容行键是别的的记录（只供自证的 MUTATION_SHARD_RUN_SELFTEST_PLANT）：
#      驱动列出它，本机库里那一行不是它
#   ⑬ 第二台缺 llvm-tools（MUTATION_SHARD_PEER_LLVM_TOOLS 指一个空目录）：驱动直接起，分行之前先查，报一行「第二台缺 llvm-tools」、退 0、不分给它，第二台这一趟的目录没建
#   ③ 第二台那一份判完 1 条就停（MUTATION_SHARD_RUN_PEER_STOP_AFTER_ROWS=1）：驱动退 1；拷回的记录照导；checker-tier-crates-mutation-replay 最后那一趟补跑缺的行，判定仍与单机逐字相同
#   ④ 两台的构建环境不同（第二台目录的上一层放 .cargo/config.toml，两台的工具链行就不同）：驱动拒、两份都没起、第二台这一趟的树删了；checker-tier-crates-mutation-replay 在本机整张判，判定与单机逐字相同
#   ⑤ 第二台的 rustc -Vv 不同：驱动拒，第二台这一趟的目录没建
#   ⑥ 两份都在跑的时候给驱动（只给它自己，kill "$!"）发 TERM：退 143，两份卡住的假 cargo 进程都不在了，没复原，第二台这一趟的树删了
#   ⑦ 选行模式下调驱动：退 2 拒
#   ⑭ ③ 分行（mutation_shard_split_plan）喂一份写好的 row-plan 输出：锚点失效的两行（一行没键、一行有键）与拒算、复用的不进两堆，汇总单报锚点失效 2 行
#   ⑧ 驱动直接起、配置没写 ENABLE_ACROSS_MACHINES=1（判法退 3）：报一行「双机：关」、退 0，第二台这一趟的目录没建
#   ⑫ 场景都跑在 TMPDIR=<自证的临时目录>/tmp 下（command-safety.md「测试镜像一律放临时目录」）：跑完外面的临时目录里没多出 singlefs-mutation-replay-*，
#      自证自己的 tmp/ 下也一个不剩（⑥ 被 TERM 停的那一份照样删了它的；删不删由 crates-mutation-rows.py 的 run_pool 管）
# 弄坏开关各让对应的格判错（见文件头「弄坏开关」）。

mutation_shard_selftest() {
  local repository_root work template checked=0 failures=0
  repository_root="$(cd "$mutation_shard_script_directory/../.." && pwd)"
  work="$(mktemp -d)"
  mutation_shard_selftest_work="$work"
  # 自证起的驱动与 checker-tier-crates-mutation-replay 认弄坏开关的标记：场景里起的都带着它
  printf '%s\n' "$MUTATION_SHARD_SELFTEST_MARK_CONTENT" > "$work/selftest-mark"
  export MUTATION_SHARD_RUN_SELFTEST_MARK="$work/selftest-mark"
  # 复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：场景里 checker-tier-crates-mutation-replay 每一趟都抽；这里比的是复用与现跑的条数，
  #   键相同的全当哨兵、不抽强制现判（哨兵数比行数多），哨兵与强制现判本身在 crates-mutation-rows.py 的自证 reuse-sentinel 那一格判
  export SINGLEFS_REUSE_SENTINEL_COUNT=1000
  unset SINGLEFS_REUSE_SENTINEL_SEED
  trap 'echo "  · 自证的临时目录（$(du -sh "$mutation_shard_selftest_work" 2>/dev/null | cut -f1)）删掉"; rm -rf -- "${mutation_shard_selftest_work:?}"' EXIT
  # 场景起的 checker-tier-crates-mutation-replay 与它的工作进程都照 TMPDIR 建临时目录：换成自证自己目录下的 tmp/，跑完随自证的临时目录一起删；
  # 开跑前先记下外面那个临时目录里已有的 singlefs-mutation-replay-*，⑫ 比的是自证跑完多出来的
  local outside_temporary="${TMPDIR:-/tmp}" outside_before
  outside_before="$(compgen -G "$outside_temporary/singlefs-mutation-replay-*" | sort || true)"
  mkdir -p "$work/tmp"
  if ! mutation_shard_break_is selftest-tmpdir-outside; then
    export TMPDIR="$work/tmp"
  fi
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
  cp -- "$repository_root/.claude/heavy-thread-default" "$template/.claude/"   # 算闭包开几个子进程照每台默认重活线程数的登记（scripts/hosts/heavy_threads.py）
  rsync -a --exclude __pycache__ "$repository_root/.claude/gate.d/lib/" "$template/.claude/gate.d/lib/"   # checker-tier-crates-mutation-replay source 的格共用库
  rsync -a --exclude __pycache__ --exclude /experiments/target/ "$repository_root/scripts/" "$template/scripts/"   # 装置的编译目录几十 G，自证用不到
  ln -s "$repository_root/.claude/singlefs-ai-sop" "$template/.claude/singlefs-ai-sop"
  printf 'target/\n__pycache__/\n/multi-host.env\n/.claude/singlefs-ai-sop\n' > "$template/.gitignore"
  printf '[workspace]\nmembers = ["crates/tiny"]\n' > "$template/Cargo.toml"
  printf '[package]\nname = "tiny"\nversion = "0.1.0"\n' > "$template/crates/tiny/Cargo.toml"
  cat > "$template/crates/tiny/src/lib.rs" <<'SAMPLE_SOURCE'
// 双机分片驱动自证的样本 crate（假 cargo 按下面几行判；下面的 #[test] 函数只给行键认点名的测试是哪一个单元）
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

#[cfg(test)]
mod tests {
    #[test]
    fn adds() { assert_eq!(super::add(2, 3), 5); }
    #[test]
    fn doubles() { assert_eq!(super::double(3), 6); }
    #[test]
    fn negates() { assert_eq!(super::negate(3), -3); }
    #[test]
    fn halves() { assert_eq!(super::halve(6), 3); }
    #[test]
    fn triples() { assert_eq!(super::triple(2), 6); }
    #[test]
    fn squares() { assert_eq!(super::square(3), 9); }
}
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
# 每起一次往自己目录下的 invocations.log 记「<过滤词>\t<lib.rs 的 sha256 前 12 位>」。自己目录下有 hang 文件时，把进程号写进 hanging.<进程号> 再睡 300 秒。
# 记覆盖那两次（照 crates-mutation-rows.py 自证的假 cargo）：开头的 --config <值> 跳过；test --no-run 在 $CARGO_TARGET_DIR/debug/deps/ 下写假的可执行文件
# （JSON：副本的根与 crates/tiny/src 下的 .rs）与它的 dep-info、打一行 compiler-artifact；设了 LLVM_PROFILE_FILE 的把跑到的用例执行的函数
# （定义行里有那段原文的）写进那份 .profraw；假 llvm 工具是 crates-mutation-rows.py 的 FAKE_LLVM_TOOL
import hashlib, json, os, re, sys, time
here = os.path.dirname(os.path.abspath(__file__))
arguments = sys.argv[1:]
while arguments[:1] == ["--config"]:
    arguments = arguments[2:]
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
if "--no-run" in arguments:
    deps = os.path.join(os.environ["CARGO_TARGET_DIR"], "debug", "deps")
    os.makedirs(deps, exist_ok=True)
    executable = os.path.join(deps, "tiny-0000000000000000")
    with open(executable, "w", encoding="utf-8") as handle:
        json.dump({"root": os.getcwd(), "sources": ["crates/tiny/src/lib.rs"]}, handle)
    with open(executable + ".d", "w", encoding="utf-8") as handle:
        handle.write(f"{executable}: crates/tiny/src/lib.rs\n\n")
    print(json.dumps({"reason": "compiler-artifact", "manifest_path": os.path.join(os.getcwd(), "crates", "tiny", "Cargo.toml"),
                      "target": {"kind": ["lib"], "name": "tiny"}, "profile": {"test": True}, "filenames": [executable], "executable": executable}))
    sys.exit(0)
after = arguments[arguments.index("--") + 1:] if "--" in arguments else []
filters = [word for word in after if not word.startswith("-")]
with open(os.path.join(here, "invocations.log"), "a", encoding="utf-8") as log:
    log.write(" ".join(filters) + "\t" + hashlib.sha256(source.encode()).hexdigest()[:12] + "\n")
failed, executed = False, set()
definitions = re.findall(r"(?m)^pub fn (\w+).*$", source)
lines = source.splitlines()
for line in lines:
    words = line.split(" ", 4)
    if len(words) < 5 or words[:2] != ["//", "TEST"]:
        continue
    name, what = words[2], words[4]
    if filters and not any(word in "tests::" + name for word in filters):
        continue
    executed |= {match.group(1) for definition in lines for match in [re.match(r"^pub fn (\w+)", definition)] if match and what in definition}
    ok = what in source.replace(line, "")
    failed = failed or not ok
    print(f"test tests::{name} ... {'ok' if ok else 'FAILED'}")
if os.environ.get("LLVM_PROFILE_FILE"):
    with open(os.environ["LLVM_PROFILE_FILE"].replace("%p", str(os.getpid())).replace("%m", "0"), "w", encoding="utf-8") as handle:
        json.dump({"executed": sorted(executed)}, handle)
sys.exit(101 if failed else 0)
FAKE_CARGO
  chmod +x "$work/fake/cargo"
  # 假 llvm 工具：取 crates-mutation-rows.py 里那一份（FAKE_LLVM_TOOL），两个名字各一份
  mkdir -p "$work/llvm"
  python3 - "$repository_root/scripts/mutation/crates-mutation-rows.py" "$work/llvm" <<'FAKE_LLVM' || { echo "  ✗ 写不出假 llvm 工具"; echo "     → 怎么办：单跑 python3 scripts/mutation/crates-mutation-rows.py --selftest 看共用模块读不读得进来"; return 1; }
import importlib.util, os, sys
specification = importlib.util.spec_from_file_location("crates_mutation_rows_for_shard_selftest", sys.argv[1])
module = importlib.util.module_from_spec(specification)
specification.loader.exec_module(module)
for tool in module.LLVM_TOOLS:
    path = os.path.join(sys.argv[2], tool)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(module.FAKE_LLVM_TOOL)
    os.chmod(path, 0o755)
FAKE_LLVM
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
      PATH="$scene/local-bin:$PATH" SINGLEFS_MULTI_HOST_CONFIG="$scene/multi-host.env" SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1
      MUTATION_SHARD_RUN_POLL_SECONDS=1 GATE_MUTATION_TARGET_DIR="$scene/local-target" RUN_WITH_MEMORY_CAP_PEAKS="$scene/local-peaks.tsv"
      GATE_MUTATION_LLVM_TOOLS="$work/llvm")
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
  verdict_view() { # <stdout 文件>：去掉随记录变的几处（「复用：」「覆盖：」两行、抓到那一行末尾的栏、成功句末尾「；这一趟现判 …」），剩下的是判定
    sed -E 's/（现判：[^）]*）$|（复用）$|（拒算）$//; s/；这一趟现判 .*$//' "$1" | grep -v -e '^  复用：' -e '^  覆盖：'
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

  # ① 主流程，连同 ② import 拒行键对不上的：拷回来的 records/ 里另放一条名字是本机那一堆第一行的键、内容的行键是别的的记录（只供自证的 MUTATION_SHARD_RUN_SELFTEST_PLANT）
  prepare_scene main
  local plan_line bogus_key base planted_key planted_file="$work/main/plant"
  plan_line="$(scene_environment main python3 "$work/main/repo/scripts/mutation/crates-mutation-rows.py" row-plan "$work/main/repo" | head -1)"
  base="$(scene_environment main python3 "$work/main/repo/scripts/mutation/crates-mutation-rows.py" judging-units "$work/main/repo" | cut -d' ' -f1)"
  bogus_key="$(printf 'bogus' | sha256sum | cut -c1-64)"
  planted_key="$(cut -f2 <<< "$plan_line")"
  mkdir -p "$planted_file"
  planted_file="$planted_file/$planted_key"
  printf '# 自证另放的记录：名字是本机那一堆第一行的键，内容的行键是别的\nkind=mutation\nrow_key=%s\njudging=%s\nverdict=caught\njudged_date=%s\n' \
    "$bogus_key" "$base" "$(date -u +%Y-%m-%d)" > "$planted_file"
  run_stage_in main MUTATION_SHARD_RUN_SELFTEST_PLANT="$planted_file"
  local main_stderr="$work/main/stderr" local_filters peer_filters all_filters
  local_filters="$(mutated_filters "$work/main/local-bin/invocations.log" | tr '\n' ' ')"
  peer_filters="$(mutated_filters "$work/main/peer-bin/invocations.log" | tr '\n' ' ')"
  all_filters="$(printf '%s\n' $local_filters $peer_filters | sort | tr '\n' ' ')"
  expect "① checker-tier-crates-mutation-replay 接驱动那一趟判绿，判定与单机那一趟逐字相同" \
    "$([[ "$(cat "$work/main/exit")" == 0 ]] && cmp -s <(verdict_view "$work/single/stdout") <(verdict_view "$work/main/stdout"); echo $?)" \
    "退 $(cat "$work/main/exit")；diff：$(diff <(verdict_view "$work/single/stdout") <(verdict_view "$work/main/stdout") | head -4 | tr '\n' ' ')；stderr 尾部：$(tail -4 "$main_stderr" | tr '\n' ' ')"
  expect "① 两台各判一份：起的变异不相交、合起来是全部 6 行、两边都不空" \
    "$([[ -n "$local_filters" && -n "$peer_filters" && "$all_filters" == "adds doubles halves negates squares triples " ]]; echo $?)" \
    "本机「$local_filters」，第二台「$peer_filters」"
  expect "① 最后那一趟全部复用（复用抓到 6 条、现跑变异至多 0 条）" \
    "$(grep -q '判定库记录：复用抓到 6 条.*现跑变异至多 0 条' "$main_stderr"; echo $?)" \
    "stderr 里的判定库记录行：$(grep '判定库记录：' "$main_stderr" | tail -1)"
  local main_logs
  main_logs="$(find "$work/main/repo/.git/singlefs-mutation-shard-logs" -name local.log 2>/dev/null | head -1)"
  expect "① 本机那一份（选行模式的 checker-tier-crates-mutation-replay）没再调驱动" \
    "$([[ -n "$main_logs" ]] && grep -q '只判了选中的' "$main_logs" && ! grep -q '双机分片' "$main_logs"; echo $?)" \
    "本机那一份的输出：${main_logs:-没找到}"
  expect "① 照配置清了场、跑完没复原（清场标记还在，报了「清了场、没复原」）" \
    "$(grep -q '照配置清了场' "$main_stderr" && grep -q '清了场、没复原' "$main_stderr" && [[ -e "$work/main/quiesce-marker-selftest" ]]; echo $?)" \
    "stderr：$(grep -E '清了场|复原' "$main_stderr" | tr '\n' ' ')"
  expect "① 第二台这一趟的树（连同它这一趟的判定库）删了，mutation-kept 下留着编译目录与峰值表、没有判定库（第二台不留长期库）" \
    "$([[ "$(peer_run_trees_left main)" == 0 && -s "$work/main/peer-home/mutation-kept/memory-peaks.tsv" && ! -e "$work/main/peer-home/mutation-kept/records" ]]; echo $?)" \
    "mutation-runs 底下留着 $(peer_run_trees_left main) 个目录；mutation-kept 底下：$(ls "$work/main/peer-home/mutation-kept" 2>/dev/null | tr '\n' ' ')"
  expect "① 第二台那一份经 run-with-memory-cap.sh 起：跨趟保留的峰值表里有它的键" \
    "$(grep -qF "$MUTATION_SHARD_PEER_WRAPPER_KEY" "$work/main/peer-home/mutation-kept/memory-peaks.tsv" 2>/dev/null; echo $?)" \
    "跨趟保留的峰值表 $(grep -vc '^#' "$work/main/peer-home/mutation-kept/memory-peaks.tsv" 2>/dev/null || true) 行，没有键「$MUTATION_SHARD_PEER_WRAPPER_KEY」那一行"
  expect "① 第二台每条变异的峰值记在跨趟保留的峰值表里" \
    "$(grep -q 'cargo test --offline' "$work/main/peer-home/mutation-kept/memory-peaks.tsv" 2>/dev/null; echo $?)" \
    "峰值表里没有 cargo test --offline 的行"
  expect "① 输出里没有配置的主机、第二台的目录与清场命令" \
    "$(! grep -qE 'selftest-peer-host-name|peer-home|peer-bin|quiesce-marker-selftest' "$work/main/stdout" "$main_stderr" "$(dirname "$main_logs")/peer.log" 2>/dev/null; echo $?)" \
    "$(grep -lE 'selftest-peer-host-name|peer-home|peer-bin|quiesce-marker-selftest' "$work/main/stdout" "$main_stderr" "$(dirname "$main_logs")/peer.log" 2>/dev/null | sed "s|^$work/||" | tr '\n' ' ')里有"
  expect "① 第二台那一份起 cargo 用 mutation-kept 下变异复跑自己的崩溃判定库家目录" \
    "$(grep -qF '起 cargo 时的崩溃判定库家目录：GATE_MUTATION_CRASH_LIBRARY_HOME 设的那一份' "$(dirname "$main_logs")/peer.log" 2>/dev/null; echo $?)" \
    "第二台那一份的输出里崩溃判定库家目录那一行：$(grep -h '崩溃判定库家目录' "$(dirname "$main_logs")/peer.log" 2>/dev/null | head -1)"
  local local_library
  local_library="$(scene_environment main python3 "$work/main/repo/scripts/mutation/crates-mutation-rows.py" library-directory "$work/main/repo")"
  expect "② import 拒了行键对不上的那一条、逐条列出" \
    "$(grep -q "拒了 1 条另一台的判定库记录" "$main_stderr" && grep -q "records/$planted_key：文件名与记录里的键对不上" "$main_stderr"; echo $?)" \
    "stderr 里导入那几行：$(grep -A2 '导入：' "$main_stderr" | head -3 | tr '\n' ' ')"
  local caught_records coverage_records
  caught_records="$(grep -lx 'verdict=caught' "$local_library"/records/* 2>/dev/null | wc -l)"
  coverage_records="$(find "$local_library/coverage" -maxdepth 1 -name '*.json.gz' 2>/dev/null | wc -l)"
  expect "① 第二台判的行连同覆盖记录导入：本机库里 6 条抓到的记录、6 份覆盖记录（6 条点名测试各一份），最后那一趟一份覆盖都没补记" \
    "$([[ "$caught_records" == 6 && "$coverage_records" == 6 ]] && grep -q '覆盖：这一趟记了 0 份' "$main_stderr"; echo $?)" \
    "抓到的记录 $caught_records 条、覆盖记录 $coverage_records 份；覆盖那一行：$(grep '覆盖：这一趟' "$main_stderr" | tail -1 | cut -c1-120)"
  expect "② 本机库里那一行的记录不是拒掉的那一条" \
    "$(! grep -q "row_key=$bogus_key" "$local_library/records/$planted_key" 2>/dev/null; echo $?)" \
    "$local_library/records/$planted_key 里是那条另放的"

  # ③ 第二台那一份判完 1 条就停
  prepare_scene stopped
  run_stage_in stopped MUTATION_SHARD_RUN_PEER_STOP_AFTER_ROWS=1
  expect "③ 第二台那一份判完 1 条就停：第二台那一份退 1，checker-tier-crates-mutation-replay 最后那一趟复用 4 条、补跑缺的 2 行，判定仍与单机逐字相同" \
    "$([[ "$(cat "$work/stopped/exit")" == 0 ]] && cmp -s <(verdict_view "$work/single/stdout") <(verdict_view "$work/stopped/stdout") && grep -q '第二台那一份：exit 1' "$work/stopped/stderr" \
       && grep -q '复用抓到 4 条.*现跑变异至多 2 条' "$work/stopped/stderr"; echo $?)" \
    "退 $(cat "$work/stopped/exit")；判定库记录行：$(grep '判定库记录：' "$work/stopped/stderr" | tail -1)；第二台：$(grep '第二台那一份：' "$work/stopped/stderr" | tail -1)"

  # ④ 构建环境不同（第二台目录的上一层放 .cargo/config.toml）：两台的工具链行不同
  prepare_scene diverged
  mkdir -p "$work/diverged/peer-home/.cargo"
  printf '[build]\nrustflags = ["-Cdebug-assertions"]\n' > "$work/diverged/peer-home/.cargo/config.toml"
  run_stage_in diverged
  expect "④ 两台的构建环境不同：驱动拒、两份都没起、第二台这一趟的树删了" \
    "$(grep -q '两台的工具链行（含构建环境）不同' "$work/diverged/stderr" && [[ ! -s "$work/diverged/peer-bin/invocations.log" && "$(peer_run_trees_left diverged)" == 0 ]]; echo $?)" \
    "stderr：$(grep -E '✗|删' "$work/diverged/stderr" | head -3 | tr '\n' ' ')；第二台留着 $(peer_run_trees_left diverged) 个目录"
  expect "④ 驱动拒了之后 checker-tier-crates-mutation-replay 在本机整张判，判定与单机逐字相同" \
    "$([[ "$(cat "$work/diverged/exit")" == 0 ]] && cmp -s <(verdict_view "$work/single/stdout") <(verdict_view "$work/diverged/stdout"); echo $?)" \
    "退 $(cat "$work/diverged/exit")"

  # ⑤ 工具链不同：第二台 PATH 上的 rustc 报另一个版本
  prepare_scene toolchain
  printf '#!/usr/bin/env bash\nprintf "rustc 0.0.0-other\\nbinary: rustc\\ncommit-hash: 0000\\nhost: other-host\\n"\n' > "$work/toolchain/peer-bin/rustc"
  chmod +x "$work/toolchain/peer-bin/rustc"
  local toolchain_exit
  if scene_environment toolchain bash "$work/toolchain/repo/scripts/mutation/mutation-shard-run.sh" "$work/toolchain/repo" > "$work/toolchain/driver.out" 2>&1; then toolchain_exit=0; else toolchain_exit=$?; fi
  expect "⑤ 两台的工具链不同：驱动拒，第二台这一趟的目录没建" \
    "$([[ "$toolchain_exit" == 1 ]] && grep -q '两台的工具链不同' "$work/toolchain/driver.out" && [[ ! -d "$work/toolchain/peer-home/mutation-runs" ]]; echo $?)" \
    "退 $toolchain_exit；输出：$(tail -3 "$work/toolchain/driver.out" | tr '\n' ' ')"

  # ⑥ 两份都在跑的时候收到 TERM
  prepare_scene signalled
  touch "$work/signalled/local-bin/hang" "$work/signalled/peer-bin/hang"
  local driver_pid signalled_exit waited hanging_pids=() hanging_file survivors=""
  # env 起的就是驱动本身（env exec 过去）：$! 是驱动的进程号，TERM 只发给它
  scene_command_for signalled
  "${scene_command[@]}" bash "$work/signalled/repo/scripts/mutation/mutation-shard-run.sh" "$work/signalled/repo" > "$work/signalled/driver.out" 2>&1 &
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
  expect "⑥ 两份都在跑时收到 TERM：退 143，两份卡住的假 cargo 都不在了，没复原，第二台这一趟的树删了" \
    "$([[ "$signalled_exit" == 143 && "${#hanging_pids[@]}" -ge 2 && -z "$survivors" && -e "$work/signalled/quiesce-marker-selftest" && "$(peer_run_trees_left signalled)" == 0 ]]; echo $?)" \
    "退 $signalled_exit；卡住的假 cargo ${#hanging_pids[@]} 个（${hanging_pids[*]:-没有}），还活着的「${survivors}」；第二台留着 $(peer_run_trees_left signalled) 个目录；输出尾部：$(tail -4 "$work/signalled/driver.out" | tr '\n' ' ')"

  # ⑦ 选行模式下调驱动
  local selection_exit
  if scene_environment single env GATE_MUTATION_ROW_SELECTION="$work/single/stdout" bash "$work/single/repo/scripts/mutation/mutation-shard-run.sh" "$work/single/repo" > "$work/single/driver.out" 2>&1; then
    selection_exit=0
  else
    selection_exit=$?
  fi
  expect "⑦ 选行模式下调驱动：退 2 拒" \
    "$([[ "$selection_exit" == 2 ]] && grep -q '选行模式下被调了' "$work/single/driver.out"; echo $?)" "退 $selection_exit"

  # ⑧ 驱动直接起、配置没开双机（单机那一趟的场景）：判法退 3
  local switched_off_exit
  if scene_environment single bash "$work/single/repo/scripts/mutation/mutation-shard-run.sh" "$work/single/repo" > "$work/single/switched-off.out" 2>&1; then
    switched_off_exit=0
  else
    switched_off_exit=$?
  fi
  expect "⑧ 驱动直接起、配置没写 ENABLE_ACROSS_MACHINES=1：报一行双机：关、退 0、第二台这一趟的目录没建" \
    "$([[ "$switched_off_exit" == 0 ]] && [[ "$(grep -c '双机：关' "$work/single/switched-off.out")" == 1 ]] && [[ ! -d "$work/single/peer-home/mutation-runs" ]]; echo $?)" \
    "退 $switched_off_exit；输出：$(tail -3 "$work/single/switched-off.out" | tr '\n' ' ')"

  # ⑨ ⑩ 弄坏开关设在自证之外（起时去掉自证的标记，弄坏开关照调用方的再各加一项）：驱动拒、退 2；checker-tier-crates-mutation-replay 判红、退 1
  local outside_driver_exit outside_stage_exit
  if scene_environment single env -u MUTATION_SHARD_RUN_SELFTEST_MARK MUTATION_SHARD_RUN_BREAK="${MUTATION_SHARD_RUN_BREAK:+$MUTATION_SHARD_RUN_BREAK,}import-skipped" \
       bash "$work/single/repo/scripts/mutation/mutation-shard-run.sh" "$work/single/repo" > "$work/single/outside-driver.out" 2>&1; then
    outside_driver_exit=0
  else
    outside_driver_exit=$?
  fi
  expect "⑨ MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外（不带自证的标记）：驱动拒、退 2，说设在 --selftest 之外" \
    "$([[ "$outside_driver_exit" == 2 ]] && grep -q '设在 --selftest 之外' "$work/single/outside-driver.out"; echo $?)" \
    "退 $outside_driver_exit；输出：$(tail -3 "$work/single/outside-driver.out" | tr '\n' ' ')"
  if scene_environment single env -u MUTATION_SHARD_RUN_SELFTEST_MARK MUTATION_SHARD_RUN_BREAK="${MUTATION_SHARD_RUN_BREAK:+$MUTATION_SHARD_RUN_BREAK,}no-final-pass" \
       bash "$work/single/repo/.claude/gate.d/checker-tier-crates-mutation-replay.sh" "$work/single/repo" > "$work/single/outside-stage.out" 2>&1; then
    outside_stage_exit=0
  else
    outside_stage_exit=$?
  fi
  expect "⑩ MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外：checker-tier-crates-mutation-replay 判红（退 1）、说设在 --selftest 之外" \
    "$([[ "$outside_stage_exit" == 1 ]] && grep -q '设在 scripts/mutation/mutation-shard-run.sh --selftest 之外' "$work/single/outside-stage.out"; echo $?)" \
    "退 $outside_stage_exit；输出：$(tail -3 "$work/single/outside-stage.out" | tr '\n' ' ')"

  # ⑪ 第二台那一份的结局逐个判（mutation_shard_peer_problem，⑥ 收两份时调的那一个）：那边的内存包装退 245 报显存不够、退 252 报包装自己的结局、退 3 不报
  local gpu_problem slice_problem done_problem
  gpu_problem="$(mutation_shard_peer_problem 'exit 245')"
  slice_problem="$(mutation_shard_peer_problem 'exit 252')"
  done_problem="$(mutation_shard_peer_problem 'exit 3')"
  expect "⑪ 第二台那一份退 245 ⇒ 报显存不够、不算结果；退 252 ⇒ 报内存包装自己的结局；退 3 ⇒ 不报" \
    "$([[ "$gpu_problem" == *'显存不够'* && "$gpu_problem" == *'不算结果'* && "$slice_problem" == *'内存包装自己的结局'* && -z "$done_problem" ]]; echo $?)" \
    "退 245 报的「$gpu_problem」；退 252 报的「$slice_problem」；退 3 报的「$done_problem」"

  # ⑭ ③ 分行：喂一份 row-plan 的输出——两行要现判、第 3 行锚点失效没键、第 4 行锚点失效而键算得出（数据文件那一类）、第 5 行拒算、第 6 行复用：
  #    两堆合起来只有第 1、2 行，汇总报拒算 1 行、锚点失效 2 行（弄坏开关 anchor-broken-dispatched 下第 4 行进了一堆）
  mkdir -p "$work/split"
  printf '1\t%s\t0\t5.000\t-\n2\t%s\t0\t4.000\t-\n3\t-\t0\t-\tanchor-broken\n4\t%s\t0\t3.000\tanchor-broken\n5\t-\t0\t-\t-\n6\t%s\t1\t2.000\t-\n' \
    "$(printf 'row-one' | sha256sum | cut -c1-64)" "$(printf 'row-two' | sha256sum | cut -c1-64)" "$(printf 'row-four' | sha256sum | cut -c1-64)" \
    "$(printf 'row-six' | sha256sum | cut -c1-64)" > "$work/split/plan.tsv"
  local split_line split_local split_peer
  split_line="$(mutation_shard_split_plan "$work/split/plan.tsv" "$work/split/local.selection" "$work/split/peer.selection" "$work/split/keys")"
  split_local="$(tr '\n' ' ' < "$work/split/local.selection" 2>/dev/null)"
  split_peer="$(tr '\n' ' ' < "$work/split/peer.selection" 2>/dev/null)"
  expect "⑭ ③ 分行：锚点失效的行（第五列 anchor-broken，有键没键都算）不进两堆、单报几行" \
    "$([[ "$(sort -n "$work/split/local.selection" "$work/split/peer.selection" | tr '\n' ' ')" == "1 2 " && "$split_line" == "6 2 4 3.5 1 5 1 4 1 2" ]]; echo $?)" \
    "本机「$split_local」第二台「$split_peer」；汇总「$split_line」"

  # ⑬ 第二台缺 llvm-tools（MUTATION_SHARD_PEER_LLVM_TOOLS 指一个空目录）：驱动直接起，分行之前先查，报一行、退 0、不分给它，第二台这一趟的目录没建
  prepare_scene nollvm
  mkdir -p "$work/nollvm/empty-llvm"
  local nollvm_exit
  if scene_environment nollvm env MUTATION_SHARD_PEER_LLVM_TOOLS="$work/nollvm/empty-llvm" bash "$work/nollvm/repo/scripts/mutation/mutation-shard-run.sh" "$work/nollvm/repo" \
       > "$work/nollvm/driver.out" 2>&1; then
    nollvm_exit=0
  else
    nollvm_exit=$?
  fi
  expect "⑬ 第二台缺 llvm-tools：分片前先查，报一行第二台缺 llvm-tools、退 0、不分给它，第二台这一趟的目录没建" \
    "$([[ "$nollvm_exit" == 0 && "$(grep -c '第二台缺 llvm-tools' "$work/nollvm/driver.out")" == 1 && ! -d "$work/nollvm/peer-home/mutation-runs" && ! -s "$work/nollvm/peer-bin/invocations.log" ]]; echo $?)" \
    "退 $nollvm_exit；输出：$(tail -3 "$work/nollvm/driver.out" | tr '\n' ' ')"

  # ⑫ 跑完不留：外面的临时目录里没多出 singlefs-mutation-replay-*，自证自己的 tmp/ 下也一个不剩
  local outside_after outside_new inside_left
  outside_after="$(compgen -G "$outside_temporary/singlefs-mutation-replay-*" | sort || true)"
  outside_new="$(comm -13 <(printf '%s\n' "$outside_before" | sed '/^$/d') <(printf '%s\n' "$outside_after" | sed '/^$/d'))"
  inside_left="$(compgen -G "$work/tmp/singlefs-mutation-replay-*" || true)"
  expect "⑫ 场景都跑在自证自己的 tmp/ 下：跑完外面的临时目录（$outside_temporary）里没多出 singlefs-mutation-replay-*，自己的 tmp/ 下也一个不剩" \
    "$([[ -z "$outside_new" && -z "$inside_left" ]]; echo $?)" \
    "外面多出来的「$(tr '\n' ' ' <<< "$outside_new")」；自证自己的 tmp/ 里剩下的「$(tr '\n' ' ' <<< "$inside_left")」"
  # 弄坏开关下外面多出来的，是这一趟自证自己建的：照名字逐个删，不留给别人
  local outside_entry
  while IFS= read -r outside_entry; do
    [[ -n "$outside_entry" && "$outside_entry" == "$outside_temporary"/singlefs-mutation-replay-* ]] && rm -rf -- "${outside_entry:?}"
  done <<< "$outside_new"

  if (( failures > 0 )); then
    echo "  ✗ mutation-shard-run.sh 自证有 $failures 格判错（共 $checked 格）"
    echo "     → 怎么办：设了 MUTATION_SHARD_RUN_BREAK 的，这就是要的结果（那个开关把对应的一步弄坏了）；没设的，照上面那几格去看驱动与 scripts/hosts/peer-host-lib.sh"
    return 1
  fi
  echo "  ✓ mutation-shard-run.sh 自证 $checked 格都对（两台各判一份、第二台判出的记录连同覆盖记录当场拷回核过再并进本机那一份库、第二台不留长期库、最后一趟全部复用且判定与单机逐字相同、第二台中途停了补跑、行键对不上被拒、判法单元指纹与工具链（含构建环境）不同就拒、第二台缺 llvm-tools 不分片、收到 TERM 先停两份、场景跑完不留 singlefs-mutation-replay-*、配置没开双机就报双机：关不分、环境变量开不了双机、弄坏开关设在自证之外驱动与门禁都拒、第二台那一份退 245 报显存不够、锚点失效的行不分给第二台）"
}

if [[ "${1:-}" == --selftest ]]; then
  mutation_shard_selftest
  exit $?
fi
if [[ -n "${MUTATION_SHARD_RUN_BREAK:-}" ]] && ! mutation_shard_break_is break-honored-anywhere && ! mutation_shard_inside_selftest; then
  echo "  ✗ MUTATION_SHARD_RUN_BREAK=$MUTATION_SHARD_RUN_BREAK 设在 --selftest 之外：弄坏开关只在自证起的进程里生效，这一次什么都没分（退 2）"
  echo "     → 怎么办：去掉它再跑（env -u MUTATION_SHARD_RUN_BREAK …）；要证红就跑 MUTATION_SHARD_RUN_BREAK=<项> bash scripts/mutation/mutation-shard-run.sh --selftest"
  exit 2
fi
mutation_shard_main "$@"
