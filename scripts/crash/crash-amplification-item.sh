#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树与库上排派活计划、再跑；哪些块要重判由库里的块键与判法单元指纹定，不在这里
# run-condition: command bash cargo python3
# run-condition: check source "$PREFLIGHT_SCRIPT_DIRECTORY/crash-amplification-lib.sh" && crash_amplification_run_condition_holds "$PREFLIGHT_SCRIPT_DIRECTORY/../.." :: 修好设备，或改配置：多机配置 multi-host.env 开了 GPU 而一份卡都没写、点名的卡与双机开关对不上；单跑 bash -c 'source scripts/crash/crash-amplification-lib.sh && crash_amplification_run_condition_holds' 看是哪一处
# 崩溃放量一个分项的一趟：用不用 GPU 照配置，做不到就报错退出，不退回 CPU
# （配置中有 gpu 就一定要让 GPU 工作，如果不行就明确报错拒绝，不接受默认降级）。
# 在哪台机器跑、上限、线程、分卡与清场由 task-server 统一管理：
#   这一份一律在起它的那一台上一趟跑完（SHARE 0/1），不自己连第二台、不调双机驱动、不清场、不读多机配置里的显卡先后；两台分着判的是崩溃放量的分片驱动
#   scripts/crash/crash-amplification-two-hosts.sh，由 task-server 照分片协议起，不经这里。
#
#   crash-amplification-item.sh <测试目标> [--test-filter <用例函数名>] [--label <标签>]
#   crash-amplification-item.sh --selftest      拿造出来的派活计划行与日志走一遍判法、运行条件与失败分类、排计划那一趟借不到显存的本次未跑、判段没跑那一行的转打、往引擎送的按段登记文件、服务交的卡、认不出的选项退 2（不编、不跑、不上卡、不连第二台）；
#       只在临时 prune-server 上跑（scripts/prune-server/prune_server_client_selftest.py --with-temporary-service 起，crash-amplification-lib.sh 的 crash_amplification_selftest_service_problem 判），不是就退 2、一格都不跑
#
# 测试目标是 crates/singlefs-verification/tests/ 下一条走 crash_amplification::run_from_environment 的用例。
# 运行条件分两半：配置上判得了的（多机配置开了 GPU 却一份卡都没写、点名的卡与双机开关对不上；配置怎么找见 crash-amplification-lib.sh 文件头「配置」）由文件头的 run-condition: check 在开跑之前判，
#   带 --force 越过时在运行中照判、退 78；跑起来才知道的（环境里没有 task-server 交的卡、点名的卡用不了、显存装不下）在运行中退 78。出路都是「修好设备，或改配置」。
#   算法上判不了的（有状态 GPU 判不了、内核展开超预算、建不出表）不是运行条件，退 248。日志怎么分这几类见 crash-amplification-lib.sh 的 crash_amplification_failure_kind。
# 步骤：
#   ① 多机配置（${SINGLEFS_MULTI_HOST_CONFIG:-主工作树的根 multi-host.env}，按 git common-dir 找）没开 GPU：先在本机整份录入段，再本机 CPU 判（SHARE 0/1）。
#   ② 开了 GPU：卡是 task-server 挑好、清过场经环境 GPU_CARDS_BY_PRIORITY 交下来的那一张（共用库 crash_amplification_prepare 取；环境里没有退 78、一趟都不起）。
#      先在本机整份录入段，再在本机跑一趟只排派活计划（SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1，不判），读它打的 CRASH_AMPLIFICATION_PLACEMENT 行
#      （一条用例里几条流各一行，逐个字段加起来）：要判的状态数 pending_states、本机的卡试判量到的速度 states_per_second_on_this_host。
#      一行都没有就不跑，打出日志里 panicked 那几行，照日志分类退：设备用不上 78、算法上判不了 248、那一趟退了非 0 而认不出原因退它的退出码、退 0 却没打计划行 247。
#      只有一种例外是本次未跑：那一趟退 0、一行计划都没有，日志里有产品路径借不到显存的那一行
#      （NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=<为什么>：起过卡池的进程里池里的卡等满了也登记不上、借不到）。
#      这时不跑，打 CRASH_AMPLIFICATION_DECISION where=not-run 与一行 CRASH_AMPLIFICATION_NOT_RUN，退 77（见「退出码」77）。
#      别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行、那一趟退了非 0 的，都不算这一种，照上面分类。
#      有计划行、又有产品路径借不到显存的那一行（一条用例几条流，有的流借到了、有的没借到）：借到的照计划判；没借到的流逐条打一行
#      CRASH_AMPLIFICATION_NOT_RUN step=placement … flow=<第几条流>/<几条流> reason=…（第几条按排计划那一趟日志里计划行与那一行的先后数），
#      标准错误逐条说这不是通过；退出码照跑的那一趟，入口照这几行把这一项记 not-run、不算全量通过。
#   ③ 有计划行：本机一台跑（SHARE 0/1）。那一趟退非 0，照 run.log 分类：设备用不上 78、算法上判不了 248，别的原样退它的退出码。
# 判定库与日志：崩溃判定只在 prune-server 里那一份库（crash-amplification-lib.sh 文件头「判定库」：引擎经客户端直接读写，这里不拼库目录、不送库的变量；重跑同一条命令就是续跑），
#   录入段每趟照起，引擎按录入键（录入函数单元指纹与录入输入摘要）查服务，命中就不再录（crates/singlefs-verification/src/crash_library.rs 的 two_stages），
#   日志在 <家目录>/runs/<标签>/（家目录 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-<工作根>/crash-amplification}，共用库 crash_amplification_home_directory，两台同一份）：
#   record.log（录入段）、placement.log（排计划那一趟）、run.log（本机一台跑的那一趟）。
# 按段登记（判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J10）：往引擎送 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY。
#   调用方送了（崩溃放量入口与崩溃点重放入口都送 <被判仓根>/.claude/gate.d/crash-point-replay-segments.tsv，开跑前已核过）就照它；没送就送本树那一份的绝对路径
#   （<本树根>/.claude/gate.d/crash-point-replay-segments.tsv，这里不核，写坏了由引擎在录入段拒）。录入段、排计划、本机判都从环境继承。
# 输出：CRASH_AMPLIFICATION_PLACEMENT 行原样转打；一行 CRASH_AMPLIFICATION_DECISION where=<local|local-cpu|not-run> reason=<为什么> …；
#   那一趟的 CRASH_AMPLIFICATION、CRASH_AMPLIFICATION_GPU_CARD、CRASH_AMPLIFICATION_GPU_TOTAL 行原样转打。
#   排计划那一趟借不到显存而本次未跑（步骤 ②）：一行 CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment reason=<那一行 reason= 之后原样>，
#   不打 CRASH_AMPLIFICATION 行；崩溃放量入口照这一行把这一项记 not-run（入口文件头「一项怎么判」）。几条流里只有几条没借到的，那几条各一行（带 flow=），
#   跑的那一趟的 CRASH_AMPLIFICATION 行照转打。
#   判段没跑（配置写了 ENABLE_GPU=1 而二进制没编进 gpu 特性：引擎每一趟判段打一行 CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=<流> reason=…
#   与一行 NOT_RUN_NO_GPU_ADAPTER、录入段照录、退 0）：本机一台跑的那一趟的 step=judge 行原样转打（两台分片跑时由双机驱动在合并那一步从各片的日志转打），
#   崩溃放量入口照它把这一项记 not-run（入口文件头「一项怎么判」）。
# 源码中途变了（共用主工作区上别的会话往树里打了补丁）：录入段跑成之后记下被测源码的摘要（crash-amplification-lib.sh 的 crash_amplification_source_digest，文件头「源码摘要」；
#   树根是本脚本所在的仓根），判段每一趟（排计划那一趟、本机一台跑的那一趟）起跑前与收尾后各再算一次，对不上（或那时算不出）：打一行
#   CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment reason=跑的过程中源码变了，判段没在录入段那份源码上判完（录入段之后 <摘要>，<哪一刻> <摘要>），
#   标准错误说这不是通过，退 77（本次未跑），不落红：那一趟的汇总行不转打，红了也不按红退；崩溃放量入口照这一行把这一项记 not-run，崩溃点重放入口照 ITEM 行的 verdict=not-run 报本次未跑并点名这条原因。
#   录入段之后算不出摘要的打一行说明、判段前后不核。弄坏开关 record-judge-source-unchecked：不核。
# 退出码（0、2、77、78 的含义照 .claude/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」那张表）：
#   0        跑的那一趟退 0。有流本次未跑也是 0（几条流里没借到的那几条打了 CRASH_AMPLIFICATION_NOT_RUN 行、判段没跑的流打了 CRASH_AMPLIFICATION_NOT_RUN step=judge 行、真跑那一趟被叫停的行里 stopped_early=true）：
#            退 0 不等于全判过了，看有没有 CRASH_AMPLIFICATION_NOT_RUN 行与 stopped_early=true
#   77       一项都没跑到：排计划那一趟借不到显存、一块都没判（打了 CRASH_AMPLIFICATION_NOT_RUN 行）；或判段跑的过程中源码变了（上面「源码中途变了」，打了 CRASH_AMPLIFICATION_NOT_RUN step=judge 行）。本次未跑，不是通过，也不是失败
#   2        用法错（参数认不得、缺测试目标、带值的选项缺值：--test-filter、--label 后面没给值，或给的是空串、以 - 起头）
#   78       运行条件不满足：开了 GPU 却没在多机配置里写显卡先后、点名的卡与双机开关对不上（开跑前由 run-condition: check 拒；带 --force 越过时照跑到下面那几样）、
#            开了 GPU 而环境里没有 task-server 交的卡（共用库 crash_amplification_prepare 判，一趟都不起），
#            点名的卡用不了或显存装不下、结论的版本与树对不上（CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD 那一行，整趟不写库）、
#            多机配置开了 GPU 而二进制没编进 gpu 特性（这三样都照日志分类，crash-amplification-lib.sh 的 crash_amplification_failure_kind）。出路：修好设备，或改配置
#   247      排派活计划那一趟退 0，却一行 CRASH_AMPLIFICATION_PLACEMENT 都没打，日志里也认不出是设备还是算法，也没有产品路径借不到显存的那一行（那一种退 77，见上）
#   248      算法上判不了：有状态 GPU 判不了（表越过内核容量）、内核展开超预算、GPU 判器建不出表（照日志分类）
#   其余     跑的那一趟（cargo test）的退出码，原样
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」「测试目标」，.claude/hooks/run-scope-guard.sh 的拒绝说明读它）：
#   <数据根>/last-runs/scripts__crash__crash-amplification-item.sh.tsv（SINGLEFS_LAST_RUNS_DIRECTORY 盖过；数据根取 .claude/hooks/lib_work_root.py、两台同一份），红的测试名一行一个：
#   「<测试目标>」，带 --test-filter 的「<测试目标>::<用例函数>」；退 0 是绿、退 2 / 77 / 130 / 143 照留，别的退出码是红；整个目标跑的那一趟换掉这个目标的全部项，
#   点名用例的那一趟只换那一项。开跑之后每一种退法都写（EXIT 钩子 item_write_last_run）；只判不跑那一支不写；
#   测试目标的源文件不在、又没显式设 SINGLEFS_LAST_RUNS_DIRECTORY 的（别的自证拿假目标起这一份）不写。
# 编译目录（自己用完自己删，不跨趟复用）：建好这一项的运行目录（<家目录>/runs/<标签>/）之后，调用方没设 CARGO_TARGET_DIR 时
#   在里面建 target、export 给本机那几趟 cargo；正常结束、红了、被 TERM / INT 停（等手里那一趟退出）都在 EXIT 钩子里删
#   （scripts/hosts/run-build-directory.sh；删之前核本机没有进程开着里面的文件，有就不删、报出进程、退非 0）。调用方（崩溃放量入口、task-server）设了的照它、不删。
# 两台对称（task-server 把这一项派到哪一台都一样）：运行条件里显卡先后每项写 <HOSTS 里的地址>:<卡号>:<额度>，哪几项是这一台的照多机配置判法 --emit-addresses 认
#   （crash-amplification-lib.sh 文件头「这一台与另一台」），不读角色变量；地址不在 HOSTS 里的项当写坏拒（退 78）。
# 跑 验证包用例：谁起、什么时候起归统一任务服务与 Bash 检出钩子，这里不另判、不往 cargo 带放行的变量。
# 弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=<项>（只给 --selftest 证红用；格号是 run_selftest 里的第 1–41 格，判错的那一格打「✗ 自证第 <号> 格」）：
#   accept-peer-cards-without-across   双机没开、显卡配置却点名了第二台的卡也照跑（「配置自相矛盾就拒绝」判错）   第 3 格
#   accept-missing-peer-cards          双机开着、显卡配置里却没有第二台的卡也照跑（「两台都要有 GPU 干活」判错）   第 4 格
#   accept-missing-placement           计划行一行都没有也照本机跑（「没有计划行就不跑」判错）                   第 6 格、第 8 格、第 9 格、第 19 格、第 23 格、第 24 格（后两格的日志也没有计划行：结论版本对不上、没编进 gpu 特性那两句本该退 78，开关一开先走「没有计划行照本机跑」，没退 78）
#   placement-not-run-as-refusal       排计划那一趟借不到显存的那一行不认，照没有计划行退 247（「本次未跑不是拒绝」判错）  第 19 格
#   accept-any-not-run-line            别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行也当产品路径借不到显存（「只认产品路径那一行」判错）  第 19 格
#   accept-not-run-after-failed-placement  排计划那一趟退了非 0 也照那一行记本次未跑（「退了非 0 就照日志分类」判错）  第 19 格
#   not-run-exits-0                    一项都没跑到（排计划那一趟借不到显存）也退 0（「一项都没跑到退 77」判错）  第 19 格
#   partial-not-run-unreported         有计划行时那几条借不到显存的流不报（只照计划行往下跑）（「没借到的流逐条报本次未跑」判错）  第 20 格
#   device-unusable-as-run-failure     日志里卡用不上的原话不认（不退 78）                                    第 8 格、第 11 格
#   algorithm-refusal-as-run-failure   日志里算法判不了的原话不认（不退 248）                                 第 9 格
#   accept-missing-card-list           开了 GPU、多机配置里一份卡都没写也照跑（「没写卡就退 78」判错）           第 10 格、第 13 格
#   missing-value-exits-1              带值的选项缺值时退 1（不按用法错退 2）                                    第 12 格
#   （共用库 crash-amplification-lib.sh 里配置那几处的开关：configuration-from-tree-root（多机配置按树根找、不按 git common-dir 找）红在第 21 格；
#    home-under-home-directory（没设 SINGLEFS_CRASH_AMPLIFICATION_HOME 时运行目录取 $HOME 底下那一份、不取工作根）红在第 38 格；cards-read-from-configuration（显卡先后从多机配置读、不看环境里 task-server 交的那一张）
#    红在第 15 格、第 17 格；gpu-card-missing-unrefused（开了 GPU 而环境里没有服务交的卡也照跑）红在第 17 格）
#   （共用库的开关 given-states-run-condition-as-run-failure（给定状态表的判器借不到显存那一行 CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 不认、不退 78）
#    打开时这里一格不红：这里的格都不造那一行；它由 scripts/crash/crash-amplification-two-hosts.sh --selftest 证红，红在 shard-given-states-run-condition 格）
#   （共用库的开关 library-marker-not-accepted（录入段只认 CRASH_AMPLIFICATION_RECORDED 行、不认崩溃枚举用例经判定库两段跑时打的 CRASH_LIBRARY_RECORDED 行）红在第 28 格：
#    照崩溃点重放用例的样子造的假用例录入段只打 CRASH_LIBRARY_RECORDED、退 1；scripts/crash/crash-amplification-two-hosts.sh --selftest 另红在 crash-case-shaped-records 格）
#   version-differs-as-run-failure     日志里结论版本对不上那一行不认（不退 78）                               第 23 格
#   gpu-feature-missing-as-run-failure 日志里「这份二进制没有编进 gpu 特性」不认（不退 78）                    第 24 格
#   record-failure-ignored             录入段没跑成也照样往下排计划、判（「录入段没跑成，判段一趟都不起」判错）   第 26 格
#   segments-registry-not-sent         调用方没送按段登记文件时也不送本树那一份（「每一趟都带按段登记文件的绝对路径」判错）   第 27 格
#   judge-not-run-dropped              本机一台跑的那一趟打的 CRASH_AMPLIFICATION_NOT_RUN step=judge 行不转打（「判段没跑记本次未跑」判错）   第 29 格
#   threads-default-cores              线程数取本机核数减一，不取服务交的 SINGLEFS_THREAD_CAP 与登记的重活默认并行度        第 30 格
#   unknown-option-accepted            认不出的选项也照收、当没给（「认不出的选项退 2」判错）   第 18 格
#   last-run-unwritten                 跑完不写上次红的清单                                        第 31 格
#   run-target-kept                    跑完不删这一项自己建的编译目录（在仓根照仓根的 target 编、跨趟留着）（「跑完运行目录里不剩它建的编译目录」判错）   第 33 格、第 34 格、第 35 格
#   this-host-from-hosts-order         共用库不问多机配置判法 --emit-addresses，一律当 HOSTS 头一项是这一台（「哪一台是这一台照网卡上的地址认」判错）   第 37 格
#   card-address-outside-hosts-accepted  共用库照认显卡先后里地址不在 HOSTS 的项（「写坏的显卡先后退 78」判错）   第 40 格
#   test-variables-honored-outside-selftest  不在自证里设了只供测试的变量也照认（「只供测试的变量不在自证里设了退 2」判错）   第 39 格
#   record-judge-source-unchecked      录入段之后、判段起跑前与收尾后不核被测源码的摘要（源码中途变了照样往下判，判红按红退）（「跑的过程中源码变了记本次未跑、退 77、不落红」判错）   第 41 格
#   （多机配置判法 scripts/crash/crash-point-replay-shard-configuration-check.sh 的开关 MULTI_HOST_CONFIGURATION_BREAK=gpu-card-keys-unknown 红在第 22 格）
# 只供测试（只在 --selftest 里认：run_selftest 给它起的子进程设 CRASH_AMPLIFICATION_ITEM_SELFTEST=1；没有这个标记而设了下面任一个，参数解析之后退 2、点名设了的几个、一趟都不起）：
#   CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY=<造好的 placement.log> 时不编不跑，只拿那份日志判、打 CRASH_AMPLIFICATION_DECISION 行就退
#   （一块都没判那一种另打 CRASH_AMPLIFICATION_NOT_RUN 行、退 77；有流没借到的照样逐条打那一行）；
#   CRASH_AMPLIFICATION_ITEM_SOURCE_ROOT=<树根> 代替算被测源码摘要的树（缺省是本脚本所在的仓根；自证给一棵造出来的小树，不读真仓的 crates/——别的会话正改着它）；
#   CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机> 代替读配置里的这两样（双机那一位只给运行条件判）；CRASH_AMPLIFICATION_ITEM_CARDS=<显卡配置的值> 代替运行条件读的显卡配置；
#   CRASH_AMPLIFICATION_ITEM_LOCAL_RUN=<脚本> 换掉本机那一趟（调法：<脚本> <share> <日志> [<附加环境 键=值>…]，自己写日志；环境照真的那一趟组（共用库
#   crash_amplification_environment_for_this_host），这一趟的 cargo test 参数——共用库拼的 cargo_test_arguments，空格连起来——放进它环境里的 CRASH_AMPLIFICATION_ITEM_LOCAL_TEST_ARGUMENTS）。
#   换不掉的：真的那一趟由 crash-amplification-lib.sh 起 cargo，自证不碰它。
#
# gate-similar: crash-amplification-two-hosts.sh 它照 task-server 的分片协议把一条用例分两片判、合并时核两片的块在服务里读得到；这里一趟在起它的那一台上跑完，不调它
# gate-similar: crash-amplification.sh 它是分项登记表与全量的循环，每项调这里
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

item_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repository_root="$(cd "$item_script_directory/../.." && pwd)"
source "$item_script_directory/crash-amplification-lib.sh"

item_break_is() { crash_amplification_item_break_is "$1"; }
exit_code_is_explained() { [[ "$1" == 78 || "$1" == 248 ]]; }   # <要退的码>：78、248 由 crash_amplification_explain_exit_code 打出路

usage() { # [<哪里不对>]
  echo "  ✗ ${1:+$1；}用法：$0 <测试目标> [--test-filter <用例函数名>] [--label <标签>]，或 --selftest" >&2
  echo "  → 怎么办：测试目标写 crates/singlefs-verification/tests/ 下的文件名（不带 .rs），例 crash_amplification_fixed_script_through_rollback；带值的选项后面紧跟它的值（不以 - 起头）" >&2
  exit 2
}

# 带值的选项：后面没给值、给的是空串或以 - 起头，按用法错退 2（文件头「退出码」）
require_value() { # <选项> <余下的参数个数（含选项自己）> [<值>]
  if (( $2 >= 2 )) && [[ -n "${3:-}" && "${3:-}" != -* ]]; then return 0; fi
  if item_break_is missing-value-exits-1; then
    echo "  ✗ $1 后面缺值" >&2
    echo "  → 怎么办：在 $1 后面给值" >&2
    exit 1
  fi
  usage "$1 后面缺值"
}

test_target=""; label="crash-amplification-$(date -u +%Y%m%dT%H%M%SZ)-$$"; selftest=0; test_filter=""
while (($# > 0)); do
  case "$1" in
    --test-filter) require_value "$1" $# "${2:-}"; test_filter="$2"; shift 2 ;;
    --label) require_value "$1" $# "${2:-}"; label="$2"; shift 2 ;;
    --selftest) selftest=1; shift ;;
    -*)
      # 弄坏开关 unknown-option-accepted：认不出的选项也照收、当没给（不拒）
      if item_break_is unknown-option-accepted; then shift; continue; fi
      usage ;;
    *) if [[ -n "$test_target" ]]; then usage; fi; test_target="$1"; shift ;;
  esac
done

# 产品路径借不到显存的那一行（文件头步骤 ②）：起过卡池的进程等满了也借不到时 Rust 打的，记号与产品路径的名字照
#   crates/singlefs-verification/src/gpu_memory_ledger.rs 的 GPU_NOT_RUN_MARKER 与 crates/singlefs-verification/src/crash_amplification.rs 的 PRODUCT_PATH_NAME_IN_THE_NOT_RUN_LINE
PLACEMENT_NOT_RUN_PREFIX='NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason='
placement_not_run_line() { # <placement.log> <排计划那一趟的退出码>：那一趟退 0、日志里有那一行就打第一行，别的打空串
  local prefix="$PLACEMENT_NOT_RUN_PREFIX"
  item_break_is placement-not-run-as-refusal && return 0
  [[ "$2" == 0 ]] || item_break_is accept-not-run-after-failed-placement || return 0
  item_break_is accept-any-not-run-line && prefix='NOT_RUN_NO_GPU_ADAPTER test='
  [[ -f "$1" ]] || return 0
  awk -v prefix="$prefix" 'index($0, prefix) == 1 { print; exit }' "$1"
}
# 本次未跑（文件头步骤 ②、「输出」）：打 CRASH_AMPLIFICATION_NOT_RUN 行，标准错误说一句这不是通过
report_placement_not_run() { # <那一行>
  local reason="${1#*reason=}"
  echo "CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment reason=$reason"
  echo "  ! 本次未跑（这不是通过）：排派活计划那一趟借不到显存，一块都没判：$reason" >&2
  echo "  · 等占着卡的 GPU 用例或测试进程跑完，再重跑同一条命令（判过的块在库里，接着判）" >&2
}
# 有计划行、又有那一行（文件头步骤 ②：一条用例几条流，有的借到有的没借到）：没借到的流逐条打 CRASH_AMPLIFICATION_NOT_RUN 行，
#   第几条按 placement.log 里计划行与那一行的先后数；标准错误逐条说这不是通过。没有这种行时什么都不打
report_placement_partial_not_run() { # <placement.log>
  local partial_lines partial_line
  item_break_is partial-not-run-unreported && return 0
  [[ -f "$1" ]] || return 0
  partial_lines="$(awk -v prefix="$PLACEMENT_NOT_RUN_PREFIX" '
    index($0, "CRASH_AMPLIFICATION_PLACEMENT ") == 1 { flows++; next }
    index($0, prefix) == 1 { flows++; missing++; ordinal[missing] = flows; reason[missing] = substr($0, length(prefix) + 1) }
    END { for (i = 1; i <= missing; i++) printf "CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment flow=%d/%d reason=%s\n", ordinal[i], flows, reason[i] }' "$1")"
  [[ -n "$partial_lines" ]] || return 0
  printf '%s\n' "$partial_lines"
  while IFS= read -r partial_line; do
    partial_line="${partial_line#* flow=}"
    echo "  ! 本次未跑（这不是通过）：排派活计划那一趟第 ${partial_line%% *} 条流借不到显存，没排进计划、这一趟不判它：${partial_line#*reason=}" >&2
  done <<<"$partial_lines"
  echo "  · 借到的流照计划判；没借到的那几条，等占着卡的 GPU 用例或测试进程跑完再重跑同一条命令（判过的块在库里，接着判）" >&2
}

# 跑不跑：读 placement.log，打 CRASH_AMPLIFICATION_DECISION 行；交回 0 本机一台跑（在哪台机器由 task-server 管）、
# 20 本次未跑（那一行放进 decided_not_run_line：排计划那一趟退 0、没有计划行、有产品路径借不到显存的那一行）、
# 1 没有计划行（该退的码放进 refused_exit_code：设备用不上 78、算法上判不了 248、那一趟退了非 0 就是它的退出码、退 0 却没打计划行 247）
decide_where_to_run() { # <placement.log> <排计划那一趟的退出码>
  local placement_log="$1" placement_exit="$2"
  local pending local_speed kind
  pending="$(crash_amplification_placement_total "$placement_log" pending_states)"
  if [[ -z "$pending" ]]; then
    decided_not_run_line="$(placement_not_run_line "$placement_log" "$placement_exit")"
    if [[ -n "$decided_not_run_line" ]]; then
      echo "CRASH_AMPLIFICATION_DECISION where=not-run reason=no-card-of-the-pool-had-room-in-time"
      return 20
    fi
    if item_break_is accept-missing-placement; then
      echo "CRASH_AMPLIFICATION_DECISION where=local reason=no-placement-line"
      return 0
    fi
    echo "  ✗ 排派活计划那一趟（退 $placement_exit）一行 CRASH_AMPLIFICATION_PLACEMENT 都没打，不跑；日志 $placement_log" >&2
    grep -h -A 3 'panicked at' "$placement_log" 2>/dev/null | head -8 | sed 's/^/    /' >&2
    kind="$(crash_amplification_failure_kind "$placement_log")"
    case "$kind" in
      device|build) refused_exit_code=78 ;;
      algorithm) refused_exit_code=248 ;;
      *) if [[ "$placement_exit" != 0 ]]; then refused_exit_code="$placement_exit"; else refused_exit_code=247; fi ;;
    esac
    if exit_code_is_explained "$refused_exit_code"; then
      crash_amplification_explain_exit_code "$refused_exit_code"
    else
      echo "  → 怎么办：照上面 panicked 那几行与日志全文改；日志里认不出是设备还是算法，单跑那一趟看全文（退 $refused_exit_code）" >&2
    fi
    return 1
  fi
  report_placement_partial_not_run "$placement_log"
  local_speed="$(crash_amplification_placement_rate "$placement_log")"
  echo "CRASH_AMPLIFICATION_DECISION where=local reason=placed-on-this-host pending_states=$pending states_per_second_on_this_host=$local_speed placed_by=task-server"
  return 0
}

run_selftest() {
  local work failures=0 cells=0 service_problem
  # 只在临时 prune-server 上跑（文件头「自证」），不是就退 2、一格都不跑
  service_problem="$(crash_amplification_selftest_service_problem)"
  if [[ -n "$service_problem" ]]; then
    echo "  ✗ crash-amplification-item.sh 自证没跑：$service_problem" >&2
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）" >&2
    exit 2
  fi
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-amplification-item-selftest.XXXXXX")"
  trap "rm -rf -- '$work'" EXIT   # 现在展开：退出时局部变量 work 不一定还在
  # 各格的运行目录（<家目录>/runs/<标签>/，文件头「日志」）都落在这里的临时目录，不写真的放量家目录（第 38 格自己去掉它）
  export SINGLEFS_CRASH_AMPLIFICATION_HOME="$work"
  # 只供测试的那几个变量（文件头「只供测试」）凭这个标记认；第 39 格自己去掉它
  export CRASH_AMPLIFICATION_ITEM_SELFTEST=1
  # 被测源码的摘要（文件头「源码中途变了」）各格都算这一棵造出来的小树，不读真仓的 crates/（别的会话正改着它）；第 41 格往里面的文件追加一行
  mkdir -p "$work/source-tree/crates" && printf '%s\n' '[workspace]' >"$work/source-tree/Cargo.toml" && printf '%s\n' 'fn main() {}' >"$work/source-tree/crates/changed.rs"
  export CRASH_AMPLIFICATION_ITEM_SOURCE_ROOT="$work/source-tree"
  # 服务交的卡不从调用方带进来（第 15、17 格自己给）
  unset GPU_CARDS_BY_PRIORITY
  # 自证里换掉这一台网卡上的地址的变量（多机配置判法只在自证里认它）不从调用方带进来：第 37 格自己给
  unset CONFIGURATION_SELFTEST_LOCAL_ADDRESSES
  # 子进程的运行条件只认这里给的：多机配置指到写全了必写键与假清场命令的一份（两个开关都关、一份卡都没写），开关与卡由各格的 CRASH_AMPLIFICATION_ITEM_* 给。
  export SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env"
  # 上次红的清单（文件头「上次红的清单」）：各格起的这一份都写进这里的临时目录，不写共用的那一份（第 31 格另给自己的目录）
  export SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs"
  # 各份多机配置共有的必写键（运行条件经多机配置判法 --emit-addresses 认这一台，整份要判得过）：主机数组写 127.0.0.1（每一台的网卡上都有）
  #   与文档用的 IPv6 地址段 2001:db8::65（不在任何一台上），两台判得一样；显卡先后的样例因此一项 IPv4、一项 IPv6（从右切两刀）
  #   按台的三个键两台各一项（判法文件头「按台的三个键」）
  printf '%s\n' "HOSTS=127.0.0.1 2001:db8::65" "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 2001:db8::65=/selftest/peer-cargo" \
    "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 2001:db8::65=/selftest/peer-builds" "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 2001:db8::65=1G" \
    ENABLE_ACROSS_MACHINES=0 "NFS_SHARED_ROOTS=$work" >"$work/multi-host-required.env"
  # 多机配置里只写假的清场命令：这里不再自己清场，清场与复原一笔都不该记（第 16 格看调用记录里没有 quiesce-stop）
  { cat "$work/multi-host-required.env"; printf '%s\n' ENABLE_GPU=0 \
    "QUIESCE_STOP_COMMAND='echo quiesce-stop >> \"\$STUB_CALLS\"'" \
    "QUIESCE_STOPPED_CHECK_COMMAND='[[ \"\${STUB_STOPPED_CHECK:-ok}\" == ok ]]'" \
    "QUIESCE_START_COMMAND='echo quiesce-start >> \"\$STUB_CALLS\"'" \
    "QUIESCE_STARTED_CHECK_COMMAND='[[ \"\${STUB_STARTED_CHECK:-ok}\" == ok ]]'"; } >"$work/multi-host.env"
  # 第 16 格指到这一份：写着假的清场命令与清场之后那一份显卡先后
  { cat "$work/multi-host.env"; printf '%s\n' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="127.0.0.1:0:15G"'; } >"$work/multi-host-quiesce-commands.env"
  # 两份开了 GPU 的多机配置：一份卡都没写（第 13 格）；只点名本机的卡（第 14 格，第 15、17 格也读它：配置里的卡与环境里服务交的卡分得开）
  { cat "$work/multi-host-required.env"; printf '%s\n' ENABLE_GPU=1 QUIESCE_STOP_COMMAND= QUIESCE_STOPPED_CHECK_COMMAND= QUIESCE_START_COMMAND= QUIESCE_STARTED_CHECK_COMMAND=; } \
    >"$work/multi-host-gpu-on-without-cards.env"
  { cat "$work/multi-host-gpu-on-without-cards.env"; printf '%s\n' 'GPU_CARDS_BY_PRIORITY="127.0.0.1:0:15G"' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="127.0.0.1:0:24G"'; } \
    >"$work/multi-host-gpu-on-with-cards.env"
  selftest_cell_failed() { # <这一格判错的一句> <出路>
    echo "  ✗ 自证第 $cells 格：$1" >&2
    echo "     → 怎么办：$2" >&2
    failures=$((failures + 1))
  }
  placement_line() { # <要判的> <本机速度>：估的用时 = 要判的 ÷ 速度
    echo "CRASH_AMPLIFICATION_PLACEMENT host=127.0.0.1 judge=gpu pending_blocks=4 pending_states=$1 states_reused=0 tables_mebibytes=10 cards=127.0.0.1:0:100:100,2001:db8::65:2:50:50 states_placed_on_this_host=100 states_placed_on_other_hosts=50 states_per_second_on_this_host=$2 estimated_seconds_on_this_host=$(( $1 / $2 ))"
  }
  decision_of() { # <placement.log> <gpu,双机> <显卡配置的值>：打 where= 的值与退出码
    local output exit_code
    output="$(CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$1" CRASH_AMPLIFICATION_ITEM_SWITCHES="$2" CRASH_AMPLIFICATION_ITEM_CARDS="$3" bash "$0" some-target 2>/dev/null)"; exit_code=$?
    printf '%s %s\n' "$(grep -o 'where=[a-z-]*' <<<"$output" | head -1)" "$exit_code"
  }
  local both="127.0.0.1:0:15G 2001:db8::65:2:10G" local_only="127.0.0.1:0:15G"
  placement_line 1000 10 >"$work/plan.log"
  # 第 1 格：双机开着、两台都有卡 ⇒ 照样本机一台跑（在哪台机器、分不分两台由 task-server 管，这里不分）
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,1 "$both")" == "where=local 0" ]] \
    || selftest_cell_failed "双机开着却没本机一台跑：$(decision_of "$work/plan.log" 1,1 "$both")" "看 decide_where_to_run：一律本机一台跑，不交双机驱动"
  # 第 2 格：双机没开、只点名本机的卡 ⇒ 本机一台跑
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,0 "$local_only")" == "where=local 0" ]] \
    || selftest_cell_failed "双机没开却没本机一台跑：$(decision_of "$work/plan.log" 1,0 "$local_only")" "看 decide_where_to_run 里 across 那一支"
  # 第 3 格：双机没开、显卡配置点名了第二台的卡 ⇒ 运行条件不满足，退 78（preflight 先拒；带 --force 时脚本里照判）
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,0 "$both")" == " 78" ]] \
    || selftest_cell_failed "双机没开、点名了第二台的卡却没退 78：$(decision_of "$work/plan.log" 1,0 "$both")" "看 crash-amplification-lib.sh 的 crash_amplification_cards_match_the_hosts 里 names_peer 那一支"
  # 第 4 格：双机开着、显卡配置里没有第二台的卡 ⇒ 退 78
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,1 "$local_only")" == " 78" ]] \
    || selftest_cell_failed "双机开着、第二台没有点名的卡却没退 78：$(decision_of "$work/plan.log" 1,1 "$local_only")" "看 crash_amplification_cards_match_the_hosts 里 across 那一支"
  # 第 5 格：一条用例几条流：要判的加起来，速度是合计的状态 ÷ 合计的用时（两条流 400 态每秒 10 态、400 态每秒 40 态 ⇒ 800 态、50 秒、每秒 16 态；
  #   把各行的速度直接相加是 50，只取第一行是 10）
  cells=$((cells + 1)); { placement_line 400 10; placement_line 400 40; } >"$work/two-flows.log"
  local two_flows
  two_flows="$(CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/two-flows.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" bash "$0" some-target 2>/dev/null)"
  grep -q 'pending_states=800 states_per_second_on_this_host=16 ' <<<"$two_flows" \
    || selftest_cell_failed "两条流的要判状态没加起来、或速度没按合计的状态 ÷ 合计的用时算：$two_flows" "看 crash_amplification_placement_total 与 crash_amplification_placement_rate"
  # 第 6 格：一行计划都没有、日志里认不出原因、那一趟退 0 ⇒ 不跑，退 247
  cells=$((cells + 1)); echo "test result: FAILED" >"$work/none.log"
  [[ "$(decision_of "$work/none.log" 1,1 "$both")" == " 247" ]] \
    || selftest_cell_failed "没有计划行、认不出原因却没退 247：$(decision_of "$work/none.log" 1,1 "$both")" "看 decide_where_to_run 开头 pending 为空那一支：分不出类、那一趟退 0 时 refused_exit_code=247"
  cat >"$work/local-run.sh" <<'STUB'
#!/usr/bin/env bash
# 假的本机那一趟：录入段那一次写一行 RECORDED（STUB_RECORD_EXIT 给了就照它退、不写那一行），排计划的那一次写一行计划，
# 真跑的那一次写一行汇总（STUB_RUN_DEVICE_FAILURE=1 时写一行卡用不了、退 101）；每次调记一笔，收到的卡另记一笔进 <调用记录>.cards
log="$2"
echo "cards=[${GPU_CARDS_BY_PRIORITY-unset}]" >> "${STUB_CALLS:?}.cards"
# STUB_CASE_SHAPE=crash-case（第 28 格）：照崩溃点重放双机那几条用例的样子——录入段只打 CRASH_LIBRARY_RECORDED、不打崩溃放量那一行，
#   判段打 CRASH_POINT_REPLAY_PARALLEL_FINISHED、不打 CRASH_AMPLIFICATION 行
if [[ "${STUB_CASE_SHAPE:-amplification}" == crash-case ]]; then
  if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_STAGE=record "* ]]; then
    echo "record" >> "${STUB_CALLS:?}"
    echo "CRASH_LIBRARY_RECORDED class=crash_state_enumeration item=stub::full manifest=0000000000000000" > "$log"
    exit 0
  fi
  echo "local-run" >> "${STUB_CALLS:?}"
  echo "CRASH_POINT_REPLAY_PARALLEL_FINISHED states=1000 slices=4 worker_threads=1 judge=cpu" > "$log"
  exit 0
fi
if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_STAGE=record "* ]]; then
  echo "record" >> "${STUB_CALLS:?}"
  if [[ -n "${STUB_RECORD_EXIT:-}" ]]; then echo "test result: FAILED. 0 passed; 1 failed" > "$log"; exit "$STUB_RECORD_EXIT"; fi
  echo "CRASH_AMPLIFICATION_RECORDED flow=stub recorded_flow=00 facts=00 packed_judge_tables=- blocks=4 exhaustive=true" > "$log"
  exit 0
fi
if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1 "* ]]; then
  echo "placement" >> "${STUB_CALLS:?}"
  # STUB_MUTATE_SOURCE（第 41 格）：给的文件追加一行，演判段跑的过程中别的会话往树里打了补丁
  if [[ -n "${STUB_MUTATE_SOURCE:-}" && "${STUB_MUTATE_IN:-}" == placement ]]; then echo "changed" >> "$STUB_MUTATE_SOURCE"; fi
  if [[ "${STUB_PLACEMENT_NOT_RUN:-0}" == 1 ]]; then
    # 第 19 格：演起过卡池的产品路径借不到显存——照 Rust 那一行打（不打计划行），退 STUB_PLACEMENT_EXIT（没给退 0）
    printf '%s\n' "running 1 test" \
      "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=等了 1800 秒，池里没有一张卡空得出 2560 MiB（每张卡的账 [15360]；这台机器上别的 GPU 用例或测试进程一直占着）" \
      "test result: ok. 1 passed; 0 failed" > "$log"
    exit "${STUB_PLACEMENT_EXIT:-0}"
  fi
  if [[ "${STUB_PLACEMENT_PARTIAL:-0}" == 1 ]]; then
    # 第 20 格：一条用例三条流，第 2 条借不到显存——两行计划夹着 Rust 那一行；STUB_PLACEMENT_PARTIAL_TEST 换掉那一行的用例名（别的用例的那一行）
    printf '%s\n' "running 1 test" \
      "CRASH_AMPLIFICATION_PLACEMENT host=127.0.0.1 judge=gpu pending_blocks=4 pending_states=1000 states_reused=0 tables_mebibytes=10 cards=127.0.0.1:0:100:100 states_placed_on_this_host=1000 states_placed_on_other_hosts=0 states_per_second_on_this_host=1 estimated_seconds_on_this_host=1000" \
      "NOT_RUN_NO_GPU_ADAPTER test=${STUB_PLACEMENT_PARTIAL_TEST:-crash_amplification::run_from_environment} reason=等了 1800 秒，池里没有一张卡空得出 5120 MiB（第 2 条流）" \
      "CRASH_AMPLIFICATION_PLACEMENT host=127.0.0.1 judge=gpu pending_blocks=4 pending_states=1000 states_reused=0 tables_mebibytes=10 cards=127.0.0.1:0:100:100 states_placed_on_this_host=1000 states_placed_on_other_hosts=0 states_per_second_on_this_host=1 estimated_seconds_on_this_host=1000" \
      "test result: ok. 1 passed; 0 failed" > "$log"
    exit 0
  fi
  # STUB_PLACEMENT_PENDING（第 39 格）：计划行里要现判的态数（没给 1000）；0 是这一项全部复用
  echo "CRASH_AMPLIFICATION_PLACEMENT host=127.0.0.1 judge=gpu pending_blocks=4 pending_states=${STUB_PLACEMENT_PENDING:-1000} states_reused=0 tables_mebibytes=10 cards=127.0.0.1:0:100:100,2001:db8::65:2:50:50 states_placed_on_this_host=100 states_placed_on_other_hosts=50 states_per_second_on_this_host=1 estimated_seconds_on_this_host=1000" > "$log"
else
  echo "local-run" >> "${STUB_CALLS:?}"
  if [[ -n "${STUB_MUTATE_SOURCE:-}" && "${STUB_MUTATE_IN:-}" == run ]]; then echo "changed" >> "$STUB_MUTATE_SOURCE"; fi
  if [[ "${STUB_RUN_SELF_TERM:-0}" == 1 ]]; then kill -s TERM "$$"; fi
  if [[ "${STUB_RUN_RED:-0}" == 1 ]]; then
    printf '%s\n' "CRASH_AMPLIFICATION mode=single_host_cpu states_checked_here=1000 red=2" "test result: FAILED. 0 passed; 1 failed" > "$log"
    exit 101
  fi
  if [[ "${STUB_RUN_DEVICE_FAILURE:-0}" == 1 ]]; then
    echo "GPU_CARD_NOT_USABLE index=0：配置点名了这张卡，本机却用不了它" > "$log"
    exit 101
  fi
  echo "CRASH_AMPLIFICATION mode=single_host_gpu states_checked_here=1000 red=0" > "$log"
fi
STUB
  # 第 7 格：双机开着、开了 GPU、计划行有要判的态 ⇒ 照样本机一台跑：录入段、排计划、跑各一趟，退 0、DECISION where=local，不交双机驱动
  cells=$((cells + 1))
  : > "$work/calls"
  local across_output across_exit
  across_output="$(STUB_CALLS="$work/calls" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" GPU_CARDS_BY_PRIORITY="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target 2>/dev/null)"; across_exit=$?
  if [[ "$across_exit" != 0 ]] || [[ "$(tr '\n' ' ' <"$work/calls")" != "record placement local-run " ]] || ! grep -q '^CRASH_AMPLIFICATION_DECISION where=local ' <<<"$across_output"; then
    selftest_cell_failed "双机开着、开了 GPU 时应当本机一台跑（录入段、排计划、跑各一趟，退 0；实退 $across_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" "看主流程末尾：一律本机一台跑，不交双机驱动"
  fi
  # 第 8 格：排计划那一趟没打计划行，日志里有卡用不了的原话 ⇒ 运行条件不满足，退 78
  cells=$((cells + 1))
  { echo "GPU_CARD_NOT_USABLE index=3：配置点名了这张卡，本机却用不了它（没有这个序号、空闲显存低于 1024 MiB、或上下文起不来）"
    echo "thread 'the_case' panicked at crates/singlefs-verification/src/crash_amplification.rs:1:1:"
    echo "配置点名给 local 这台机器的卡里，序号 [3] 用不了（上面 GPU_CARD_NOT_USABLE 那几行）"
    echo "test result: FAILED"; } >"$work/device.log"
  [[ "$(decision_of "$work/device.log" 1,0 "$local_only")" == " 78" ]] \
    || selftest_cell_failed "卡用不了却没退 78：$(decision_of "$work/device.log" 1,0 "$local_only")" "看 crash-amplification-lib.sh 的 crash_amplification_failure_kind 认不认 GPU_CARD_NOT_USABLE，与 decide_where_to_run 里 device 那一支"
  # 第 9 格：排计划那一趟没打计划行，日志里是算法判不了的原话（表越过内核容量）⇒ 退 248，不算运行条件
  cells=$((cells + 1))
  { echo "thread 'the_case' panicked at crates/singlefs-verification/src/crash_amplification.rs:1:1:"
    echo "卡 0 判不了点 p 块内第 5 个状态（恢复内核第 1 字 7、checker 内核第 3 字 9：表越过了内核的容量）。配置开了 GPU，不退回 CPU 判器"
    echo "test result: FAILED"; } >"$work/algorithm.log"
  [[ "$(decision_of "$work/algorithm.log" 1,0 "$local_only")" == " 248" ]] \
    || selftest_cell_failed "算法判不了却没退 248：$(decision_of "$work/algorithm.log" 1,0 "$local_only")" "看 crash_amplification_failure_kind 认不认「表越过了内核的容量」，与 decide_where_to_run 里 algorithm 那一支"
  # 第 10 格：开了 GPU、显卡配置一份都没写 ⇒ 运行条件不满足，退 78
  cells=$((cells + 1))
  [[ "$(decision_of "$work/plan.log" 1,0 "")" == " 78" ]] \
    || selftest_cell_failed "开了 GPU、一份卡都没写却没退 78：$(decision_of "$work/plan.log" 1,0 "")" "看 crash-amplification-lib.sh 的 crash_amplification_run_condition_holds 里 lists 为空那一支"
  # 第 11 格：本机一台跑的那一趟退 101、日志里有卡用不了的原话 ⇒ 退 78，不原样退 101
  cells=$((cells + 1))
  : > "$work/calls"
  local device_run_exit
  STUB_CALLS="$work/calls" STUB_RUN_DEVICE_FAILURE=1 CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" GPU_CARDS_BY_PRIORITY="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    bash "$0" some-target >/dev/null 2>&1; device_run_exit=$?
  if [[ "$device_run_exit" != 78 ]] || [[ "$(tr '\n' ' ' <"$work/calls")" != "record placement local-run " ]]; then
    selftest_cell_failed "本机那一趟卡用不了却没退 78、或没照「录入段、排计划、跑」走（退 $device_run_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" "看 run_on_this_host_alone：退非 0 时经 crash_amplification_exit_code_of_a_failed_run 照 run.log 分类"
  fi
  # 第 12 格：带值的选项缺值（放在末尾、或后面紧跟另一个选项）：退 2（用法错）；各个带值的选项各试两次。
  #   给了只判不跑的日志与开关：万一参数照收了，走的也是只判不跑那一支，不编不跑
  cells=$((cells + 1))
  local option missing_value_exit missing_value_wrong=""
  for option in --test-filter --label; do
    CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/plan.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 bash "$0" some-target "$option" >/dev/null 2>&1; missing_value_exit=$?
    [[ "$missing_value_exit" == 2 ]] || missing_value_wrong+="$option 放末尾退 $missing_value_exit；"
    CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/plan.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 bash "$0" some-target "$option" --label >/dev/null 2>&1; missing_value_exit=$?
    [[ "$missing_value_exit" == 2 ]] || missing_value_wrong+="$option 后面紧跟另一个选项退 $missing_value_exit；"
  done
  [[ -z "$missing_value_wrong" ]] \
    || selftest_cell_failed "带值的选项缺值没按用法错退 2：$missing_value_wrong" "看参数那一段的 require_value：缺值、空串、以 - 起头都交 usage（退 2）"
  # 第 13、14 格：运行条件（文件头 run-condition: check 调的 crash_amplification_run_condition_holds，不经 preflight 直接调，看它自己的 ✗）读的是多机配置里的显卡先后：
  #   13 开了 GPU、多机配置里一份卡都没写 ⇒ 交回 1、打「一份都没写」；
  #   14 开了 GPU、多机配置只点名本机的卡、双机没开 ⇒ 交回 0
  local cards_cell cards_configuration cards_exit cards_want
  for cards_cell in 13 14; do
    cells=$((cells + 1))
    if [[ "$cards_cell" == 13 ]]; then cards_configuration="$work/multi-host-gpu-on-without-cards.env"; cards_want=1; else cards_configuration="$work/multi-host-gpu-on-with-cards.env"; cards_want=0; fi
    env -u CRASH_AMPLIFICATION_ITEM_SWITCHES -u CRASH_AMPLIFICATION_ITEM_CARDS SINGLEFS_MULTI_HOST_CONFIG="$cards_configuration" \
      bash -c 'source "$1" && crash_amplification_run_condition_holds "$2"' run-condition "$item_script_directory/crash-amplification-lib.sh" "$work" \
      >"$work/cards-$cards_cell.out" 2>&1; cards_exit=$?
    if [[ "$cards_exit" != "$cards_want" ]] || { [[ "$cards_cell" == 13 ]] && ! grep -q 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 一份都没写' "$work/cards-$cards_cell.out"; }; then
      selftest_cell_failed "开了 GPU、多机配置$( [[ "$cards_cell" == 13 ]] && echo 一份卡都没写 || echo 只点名本机的卡 )：运行条件应当交回 $cards_want$( [[ "$cards_cell" == 13 ]] && echo '、打「一份都没写」')，实际交回 $cards_exit（$(tr '\n' ' ' <"$work/cards-$cards_cell.out" | cut -c1-200)）" \
        "看 crash-amplification-lib.sh 的 crash_amplification_run_condition_holds：显卡先后从多机配置读"
    fi
  done
  # 第 15 格：多机配置开了 GPU、显卡先后写着 127.0.0.1:0:15G，环境里是 task-server 交的 127.0.0.1:1:20G ⇒ 录入段、排计划、跑三趟收到的卡都是服务交的那一张，退 0
  cells=$((cells + 1))
  : > "$work/calls"; : > "$work/calls.cards"
  local card_exit
  STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on-with-cards.env" GPU_CARDS_BY_PRIORITY=127.0.0.1:1:20G \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target >"$work/service-card.out" 2>&1; card_exit=$?
  if [[ "$card_exit" != 0 || "$(tr '\n' ' ' <"$work/calls")" != "record placement local-run " || "$(tr '\n' ' ' <"$work/calls.cards")" != "cards=[127.0.0.1:1:20G] cards=[127.0.0.1:1:20G] cards=[127.0.0.1:1:20G] " ]]; then
    selftest_cell_failed "开了 GPU：三趟收到的卡应当都是环境里服务交的 127.0.0.1:1:20G、不读多机配置里的显卡先后（退 $card_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」，收到的卡「$(tr '\n' ' ' <"$work/calls.cards")」）" \
      "看 crash-amplification-lib.sh 的 crash_amplification_prepare 取卡那一段与 crash_amplification_environment_for_this_host"
  fi
  # 第 16 格：开了 GPU、多机配置里写着假的清场命令 ⇒ 不再自己清场：调用次序「录入段、排计划、跑」、没有 quiesce-stop，输出里没有经清场包装重起的那一句，退 0
  cells=$((cells + 1))
  : > "$work/calls"
  local quiesce_exit
  STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesce-commands.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" GPU_CARDS_BY_PRIORITY="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target >"$work/no-quiesce.out" 2>&1; quiesce_exit=$?
  if [[ "$quiesce_exit" != 0 || "$(tr '\n' ' ' <"$work/calls")" != "record placement local-run " ]] || grep -q 'gpu-quiesce-run.sh' "$work/no-quiesce.out"; then
    selftest_cell_failed "开了 GPU 时不该再自己清场（退 $quiesce_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" "分卡与清场由 task-server 管：这里不经 scripts/crash/gpu-quiesce-run.sh 重起自己"
  fi
  # 第 17 格：多机配置开了 GPU、环境里没有 task-server 交的卡 ⇒ 退 78、一趟都没起、说是 task-server 交的显卡
  cells=$((cells + 1))
  : > "$work/calls"
  STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on-with-cards.env" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target >"$work/card-missing.out" 2>&1; card_exit=$?
  if [[ "$card_exit" != 78 || -s "$work/calls" ]] || ! grep -q 'task-server 交的显卡' "$work/card-missing.out"; then
    selftest_cell_failed "开了 GPU 而环境里没有服务交的卡，应当退 78、一趟都不起（退 $card_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" \
      "看 crash-amplification-lib.sh 的 crash_amplification_prepare 里 gpu_cards_by_priority 为空那一支"
  fi
  # 第 18 格：认不出的选项（用一个从没用过的名字）⇒ 退 2、一趟都不起（调用记录是空的）、一行判定都不打
  cells=$((cells + 1))
  : > "$work/calls"
  local unknown_option_output unknown_option_exit
  unknown_option_output="$(STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesce-commands.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" GPU_CARDS_BY_PRIORITY="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target --never-used-option 2>"$work/unknown-option.err")"; unknown_option_exit=$?
  if [[ "$unknown_option_exit" != 2 || -s "$work/calls" || -n "$unknown_option_output" ]]; then
    selftest_cell_failed "认不出的选项应当退 2、一趟都不起、一行判定都不打（退 $unknown_option_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」，标准输出「$(tr '\n' ' ' <<<"$unknown_option_output" | cut -c1-160)」）" "看参数解析那一段：-* 的分支交 usage（退 2）"
  fi
  # 第 19 格：排计划那一趟借不到显存（起过卡池的产品路径打 NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment 那一行、退 0、没有计划行）⇒
  #   一项都没跑到：打 DECISION where=not-run 与 CRASH_AMPLIFICATION_NOT_RUN 行、退 77，不跑（双机开着也一样），不退 247、不退 0。
  #   两样不算这一种：别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行照没有计划行退 247；那一趟退了非 0（101）照日志分类原样退 101、也不跑。
  cells=$((cells + 1))
  local not_run_exit not_run_calls not_run_wrong=""
  : > "$work/calls"
  STUB_CALLS="$work/calls" STUB_PLACEMENT_NOT_RUN=1 CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    bash "$0" some-target >"$work/not-run.out" 2>&1; not_run_exit=$?
  not_run_calls="$(tr '\n' ' ' <"$work/calls")"
  if [[ "$not_run_exit" != 77 || "$not_run_calls" != "record placement " ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_DECISION where=not-run ' "$work/not-run.out" \
     || ! grep -q '^CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment reason=等了 1800 秒，池里没有一张卡空得出 2560 MiB' "$work/not-run.out" \
     || ! grep -q '本次未跑（这不是通过）：排派活计划那一趟借不到显存' "$work/not-run.out" || grep -q '^CRASH_AMPLIFICATION ' "$work/not-run.out"; then
    not_run_wrong+="产品路径借不到显存那一趟退 $not_run_exit、调用次序「$not_run_calls」、输出尾部 $(tail -n 3 "$work/not-run.out" | tr '\n' '|')；"
  fi
  printf '%s\n' "NOT_RUN_NO_GPU_ADAPTER test=some_gpu_parity_test reason=本机没有 NVIDIA 卡" "test result: ok. 1 passed; 0 failed" >"$work/other-not-run.log"
  [[ "$(decision_of "$work/other-not-run.log" 1,0 "$local_only")" == " 247" ]] \
    || not_run_wrong+="别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行没照没有计划行退 247（$(decision_of "$work/other-not-run.log" 1,0 "$local_only")）；"
  : > "$work/calls"
  STUB_CALLS="$work/calls" STUB_PLACEMENT_NOT_RUN=1 STUB_PLACEMENT_EXIT=101 CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    bash "$0" some-target >"$work/not-run-failed.out" 2>&1; not_run_exit=$?
  not_run_calls="$(tr '\n' ' ' <"$work/calls")"
  if [[ "$not_run_exit" != 101 || "$not_run_calls" != "record placement " ]] || grep -q '^CRASH_AMPLIFICATION_NOT_RUN ' "$work/not-run-failed.out"; then
    not_run_wrong+="排计划那一趟退 101 而日志里有那一行，退 $not_run_exit、调用次序「$not_run_calls」（要原样退 101、不跑、不打 CRASH_AMPLIFICATION_NOT_RUN）；"
  fi
  [[ -z "$not_run_wrong" ]] \
    || selftest_cell_failed "排计划那一趟借不到显存没按一项都没跑到退 77、或不该算的也算了：$not_run_wrong" \
      "看 decide_where_to_run 里 pending 为空那一支开头的 placement_not_run_line（只认那一趟退 0、行首是「$PLACEMENT_NOT_RUN_PREFIX」的那一行）与末尾 case 的 20 那一支"
  # 第 20 格：一条用例三条流、第 2 条借不到显存（两行计划夹着产品路径那一行，那一趟退 0）⇒ 借到的照判：本机一台跑（DECISION pending_states=2000、跑了 local-run、退 0），
  #   没借到的那一条打一行 CRASH_AMPLIFICATION_NOT_RUN step=placement … flow=2/3 reason=…、标准错误说这不是通过；只判不跑、双机开着也照打那一行。
  #   别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行夹在计划行之间不算，不打 CRASH_AMPLIFICATION_NOT_RUN
  cells=$((cells + 1))
  local partial_exit partial_calls partial_wrong="" partial_decision
  : > "$work/calls"
  STUB_CALLS="$work/calls" STUB_PLACEMENT_PARTIAL=1 CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    bash "$0" some-target >"$work/partial.out" 2>&1; partial_exit=$?
  partial_calls="$(tr '\n' ' ' <"$work/calls")"
  if [[ "$partial_exit" != 0 || "$partial_calls" != "record placement local-run " ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_DECISION where=local .*pending_states=2000 ' "$work/partial.out" \
     || [[ "$(grep -c '^CRASH_AMPLIFICATION_NOT_RUN ' "$work/partial.out")" != 1 ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment flow=2/3 reason=等了 1800 秒，池里没有一张卡空得出 5120 MiB（第 2 条流）$' "$work/partial.out" \
     || ! grep -q '本次未跑（这不是通过）：排派活计划那一趟第 2/3 条流借不到显存' "$work/partial.out" || ! grep -q '^CRASH_AMPLIFICATION mode=' "$work/partial.out"; then
    partial_wrong+="三条流第 2 条借不到显存那一趟退 $partial_exit、调用次序「$partial_calls」、CRASH_AMPLIFICATION_NOT_RUN $(grep -c '^CRASH_AMPLIFICATION_NOT_RUN ' "$work/partial.out") 行、输出尾部 $(tail -n 4 "$work/partial.out" | tr '\n' '|')；"
  fi
  printf '%s\n' "running 1 test" "$(placement_line 1000 10)" \
    "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=第 2 条流借不到" "$(placement_line 1000 10)" >"$work/partial-plan.log"
  partial_decision="$(CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/partial-plan.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" bash "$0" some-target 2>/dev/null)"
  grep -q '^CRASH_AMPLIFICATION_NOT_RUN step=placement test=crash_amplification::run_from_environment flow=2/3 reason=第 2 条流借不到$' <<<"$partial_decision" \
    || partial_wrong+="只判不跑、双机开着时没打 flow=2/3 那一行：$(tr '\n' '|' <<<"$partial_decision")；"
  printf '%s\n' "$(placement_line 1000 10)" "NOT_RUN_NO_GPU_ADAPTER test=some_gpu_parity_test reason=本机没有 NVIDIA 卡" "$(placement_line 1000 10)" >"$work/partial-other.log"
  partial_decision="$(CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/partial-other.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" bash "$0" some-target 2>/dev/null)"
  if grep -q '^CRASH_AMPLIFICATION_NOT_RUN ' <<<"$partial_decision"; then partial_wrong+="别的用例名的那一行夹在计划行之间也打了 CRASH_AMPLIFICATION_NOT_RUN：$(tr '\n' '|' <<<"$partial_decision")；"; fi
  [[ -z "$partial_wrong" ]] \
    || selftest_cell_failed "几条流里有的借不到显存，没借到的没逐条报本次未跑、或借到的没照判：$partial_wrong" \
      "看 decide_where_to_run 里 pending 不空那一支开头的 report_placement_partial_not_run（只认行首是「$PLACEMENT_NOT_RUN_PREFIX」的那一行，第几条按计划行与那一行的先后数）"
  # 第 21 格：没设 SINGLEFS_MULTI_HOST_CONFIG 时多机配置按 git common-dir 找主工作树那一份（在 worktree 里跑也读主工作树的 multi-host.env，判决 r3 第二节 M1-3）：
  #   临时目录里 git init 一个主仓、加一个 worktree，从 worktree 的树根找，要找到主仓根下的那一份；不是 git 工作树的目录退回它自己的树根
  cells=$((cells + 1))
  local common_found plain_found
  mkdir -p "$work/git-main" "$work/not-a-tree"
  if git -C "$work/git-main" init -q 2>/dev/null && git -C "$work/git-main" -c user.name=selftest -c user.email=selftest@invalid commit -q --allow-empty -m selftest 2>/dev/null \
     && git -C "$work/git-main" worktree add -q --detach "$work/git-worktree" 2>/dev/null; then
    common_found="$(env -u SINGLEFS_MULTI_HOST_CONFIG bash -c 'source "$1" && crash_amplification_multi_host_configuration_file "$2"' find "$item_script_directory/crash-amplification-lib.sh" "$work/git-worktree" 2>&1)"
    plain_found="$(env -u SINGLEFS_MULTI_HOST_CONFIG GIT_CEILING_DIRECTORIES="$work" bash -c 'source "$1" && crash_amplification_multi_host_configuration_file "$2"' find "$item_script_directory/crash-amplification-lib.sh" "$work/not-a-tree" 2>&1)"
    if [[ "$common_found" != "$(cd "$work/git-main" && pwd -P)/multi-host.env" && "$common_found" != "$work/git-main/multi-host.env" ]] || [[ "$plain_found" != "$work/not-a-tree/multi-host.env" ]]; then
      selftest_cell_failed "多机配置没按 git common-dir 找：从 worktree 找到「$common_found」（应当是 $work/git-main/multi-host.env），从不是 git 工作树的目录找到「$plain_found」（应当是 $work/not-a-tree/multi-host.env）" \
        "看 crash-amplification-lib.sh 的 crash_amplification_multi_host_configuration_file：取 git -C <树根> rev-parse --git-common-dir 的上一层"
    fi
  else
    selftest_cell_failed "临时目录里建不出主仓与 worktree（git init、commit、worktree add 有一步没成）" "看 git 在不在 PATH 上、TMPDIR 可不可写"
  fi
  # 第 22 格：多机配置判法（scripts/crash/crash-point-replay-shard-configuration-check.sh）认显卡的两个键：
  #   必写键（主机数组写 127.0.0.1 与文档用的地址段：127.0.0.1 在每一台的网卡上，两台判得一样）加 GPU_CARDS_BY_PRIORITY、GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED、双机没开 ⇒ 退 3（判得过、双机：关，不连第二台）
  cells=$((cells + 1))
  local shard_check_exit
  printf '%s\n' "HOSTS=127.0.0.1 192.0.2.65" "CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.65=/selftest/peer-cargo" \
    "BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.65=/selftest/peer-builds" "SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 192.0.2.65=1G" \
    QUIESCE_STOP_COMMAND= QUIESCE_STOPPED_CHECK_COMMAND= QUIESCE_START_COMMAND= QUIESCE_STARTED_CHECK_COMMAND= ENABLE_ACROSS_MACHINES=0 ENABLE_GPU=1 NFS_SHARED_ROOTS="$work" \
    'GPU_CARDS_BY_PRIORITY="127.0.0.1:0:15G"' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="127.0.0.1:0:24G"' >"$work/multi-host-merged.env"
  SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-merged.env" bash "$item_script_directory/crash-point-replay-shard-configuration-check.sh" "$work" >"$work/shard-check.out" 2>&1; shard_check_exit=$?
  if [[ "$shard_check_exit" != 3 ]]; then
    selftest_cell_failed "多机配置写了显卡的两个键却判不过（退 $shard_check_exit，应当 3：$(tr '\n' ' ' <"$work/shard-check.out" | cut -c1-160)）" \
      "看 scripts/crash/crash-point-replay-shard-configuration-check.sh 的 MULTI_HOST_GPU_CARD_KEYS 与认键那一段"
  fi
  # 第 23 格：排计划那一趟没打计划行，日志里是引擎打的结论版本对不上那一行（整趟不写库，引擎 E 报告第九节第 ① 条）⇒ 运行条件不满足，退 78
  cells=$((cells + 1))
  { echo "running 1 test"
    echo "CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD reason=table_differs entries=crash.rs（整趟不写库：重编再跑，或在提交那一趟的 worktree 里跑；退 78）"
    echo "thread 'the_case' panicked at crates/singlefs-verification/src/crash_amplification.rs:1:1:"
    echo "test result: FAILED. 0 passed; 1 failed"; } >"$work/version-differs.log"
  [[ "$(decision_of "$work/version-differs.log" 1,0 "$local_only")" == " 78" ]] \
    || selftest_cell_failed "结论版本对不上那一行却没退 78：$(decision_of "$work/version-differs.log" 1,0 "$local_only")" "看 crash-amplification-lib.sh 的 crash_amplification_failure_kind 认不认行首 CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD（build 那一类），与 decide_where_to_run 里 device|build 那一支"
  # 第 24 格：排计划那一趟没打计划行，日志里是「这份二进制没有编进 gpu 特性」（多机配置开了 GPU 而编的不带 gpu 特性）⇒ 运行条件不满足，退 78
  cells=$((cells + 1))
  { echo "running 1 test"
    echo "thread 'the_case' panicked at crates/singlefs-verification/src/crash_amplification.rs:1:1:"
    echo "配置写了 ENABLE_GPU=1，判器、核对与单元校验和都要走 GPU，这份二进制没有编进 gpu 特性（cargo … --features gpu）"
    echo "test result: FAILED. 0 passed; 1 failed"; } >"$work/gpu-feature-missing.log"
  [[ "$(decision_of "$work/gpu-feature-missing.log" 1,0 "$local_only")" == " 78" ]] \
    || selftest_cell_failed "开了 GPU 而二进制没编进 gpu 特性却没退 78：$(decision_of "$work/gpu-feature-missing.log" 1,0 "$local_only")" "看 crash-amplification-lib.sh 的 crash_amplification_failure_kind 认不认「这份二进制没有编进 gpu 特性」（build 那一类）"
  # 第 25 格：自证只在临时 prune-server 上跑（文件头「自证」）：摘掉临时服务的标记再起一份自证 ⇒ 退 2、说「自证没跑」，一格都不跑（输出里没有「✗ 自证第」）
  cells=$((cells + 1))
  local bare_exit
  env -u PRUNE_SERVER_TEMPORARY_SERVICE_SOCKET bash "$0" --selftest >"$work/bare-selftest.out" 2>&1; bare_exit=$?
  if [[ "$bare_exit" != 2 ]] || ! grep -q 'crash-amplification-item.sh 自证没跑：' "$work/bare-selftest.out" || grep -q '✗ 自证第' "$work/bare-selftest.out"; then
    selftest_cell_failed "不在临时 prune-server 底下起的自证应当退 2、说自证没跑、一格都不跑（退 $bare_exit，输出 $(head -n 2 "$work/bare-selftest.out" | tr '\n' '|')）" \
      "看 run_selftest 开头调 crash-amplification-lib.sh 的 crash_amplification_selftest_service_problem 那一段"
  fi
  # 第 26 格：录入段那一趟退 101 ⇒ 这一项原样退 101，只调了录入段那一次，排计划与判段一趟都没起（两段：录入段没跑成，判段不起）
  cells=$((cells + 1))
  : > "$work/calls"
  local record_failed_exit
  STUB_CALLS="$work/calls" STUB_RECORD_EXIT=101 CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    bash "$0" some-target >"$work/record-failed.out" 2>&1; record_failed_exit=$?
  if [[ "$record_failed_exit" != 101 || "$(tr '\n' ' ' <"$work/calls")" != "record " ]]; then
    selftest_cell_failed "录入段退 101 却没原样退 101、或之后还起了别的趟（退 $record_failed_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" \
      "看主流程里 crash_amplification_record_on_this_host 那一句：录入段没跑成就 exit，判段一趟都不起"
  fi
  # 第 27 格：按段登记（文件头「按段登记」）：假的本机那一趟记下环境里的 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY。调用方没送 ⇒ 录入段与判段两趟
  #   都收到本树那一份的绝对路径；调用方送了 <别的文件> ⇒ 两趟收到的都是它（不换成本树那一份）
  cells=$((cells + 1))
  printf '%s\n' '#!/usr/bin/env bash' 'echo "segments=[${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}]" >> "${STUB_CALLS:?}"' \
    'if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_STAGE=record "* ]]; then echo "CRASH_AMPLIFICATION_RECORDED flow=stub" > "$2"; exit 0; fi' \
    'echo "CRASH_AMPLIFICATION mode=single_host_cpu states_checked_here=1 red=0" > "$2"' >"$work/segments-run.sh"
  local segments_exit segments_given_exit segments_calls segments_given_calls own_registry="$repository_root/.claude/gate.d/crash-point-replay-segments.tsv"
  : > "$work/calls"
  env -u SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY STUB_CALLS="$work/calls" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/segments-run.sh" \
    bash "$0" some-target >"$work/segments.out" 2>&1; segments_exit=$?
  segments_calls="$(tr '\n' ' ' <"$work/calls")"
  : > "$work/calls"
  SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY="$work/given-segments.tsv" STUB_CALLS="$work/calls" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/segments-run.sh" \
    bash "$0" some-target >"$work/segments-given.out" 2>&1; segments_given_exit=$?
  segments_given_calls="$(tr '\n' ' ' <"$work/calls")"
  if [[ "$segments_exit" != 0 || "$segments_given_exit" != 0 || "$segments_calls" != "segments=[$own_registry] segments=[$own_registry] " \
        || "$segments_given_calls" != "segments=[$work/given-segments.tsv] segments=[$work/given-segments.tsv] " ]]; then
    selftest_cell_failed "每一趟都带按段登记文件的绝对路径判错：调用方没送时两趟收到「$segments_calls」（应当都是 $own_registry，退 $segments_exit），送了 $work/given-segments.tsv 时收到「$segments_given_calls」（退 $segments_given_exit）" \
      "看只判不跑那一段之后「按段登记」那一段：SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY 为空时 export <本树根>/.claude/gate.d/crash-point-replay-segments.tsv"
  fi
  # 第 28 格：照崩溃点重放双机那几条用例的样子（假的本机那一趟 STUB_CASE_SHAPE=crash-case：录入段只打 CRASH_LIBRARY_RECORDED）、点名用例、本机一台 CPU 跑 ⇒
  #   录入段真跑、CRASH_LIBRARY_RECORDED 那一行算录入成了并转打出来、
  #   之后照判段跑、退 0（调用次序「record local-run」）
  cells=$((cells + 1))
  : > "$work/calls"
  local crash_case_shape_exit crash_case_shape_calls
  STUB_CALLS="$work/calls" STUB_CASE_SHAPE=crash-case CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    bash "$0" some-target --test-filter the_full_case >"$work/crash-case-shape.out" 2>&1; crash_case_shape_exit=$?
  crash_case_shape_calls="$(tr '\n' ' ' <"$work/calls")"
  if [[ "$crash_case_shape_exit" != 0 || "$crash_case_shape_calls" != "record local-run " ]] \
     || ! grep -qxF "CRASH_LIBRARY_RECORDED class=crash_state_enumeration item=stub::full manifest=0000000000000000" "$work/crash-case-shape.out" \
     || grep -q '^CRASH_AMPLIFICATION_RECORDED ' "$work/crash-case-shape.out"; then
    selftest_cell_failed "照崩溃点重放用例的样子造的用例（录入段只打 CRASH_LIBRARY_RECORDED）点名了用例：录入段应当真跑、那一行算录入成了并转打出来、之后照判段跑、退 0（退 $crash_case_shape_exit，调用次序「$crash_case_shape_calls」，输出尾部 $(tail -n 3 "$work/crash-case-shape.out" | tr '\n' '|')）" \
      "看 crash-amplification-lib.sh 的 crash_amplification_prepare（test_filter 非空时 cargo test 参数带 --exact）与 crash_amplification_record_on_this_host 认的录入行（CRASH_LIBRARY_RECORDED 也算）"
  fi
  # 第 29 格：本机一台跑的那一趟判段没跑（没编进 gpu 特性的二进制遇到 ENABLE_GPU=1：打一行 CRASH_AMPLIFICATION_NOT_RUN step=judge 与一行 NOT_RUN_NO_GPU_ADAPTER、录入段照录、退 0）⇒
  #   那一行原样转打、恰好一行，这一项退 0（入口照它记本次未跑）；NOT_RUN_NO_GPU_ADAPTER 那一行不转打
  cells=$((cells + 1))
  printf '%s\n' '#!/usr/bin/env bash' \
    'if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_STAGE=record "* ]]; then echo "CRASH_AMPLIFICATION_RECORDED flow=stub" > "$2"; exit 0; fi' \
    'printf "%s\n" "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=stub reason=配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（假的用例）" "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（假的用例）" "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=0 red=0" > "$2"' >"$work/judge-not-run.sh"
  local judge_not_run_exit
  CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/judge-not-run.sh" \
    bash "$0" some-target >"$work/judge-not-run.out" 2>&1; judge_not_run_exit=$?
  if [[ "$judge_not_run_exit" != 0 ]] \
     || [[ "$(grep -c '^CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=stub reason=' "$work/judge-not-run.out")" != 1 ]] \
     || grep -q '^NOT_RUN_NO_GPU_ADAPTER ' "$work/judge-not-run.out"; then
    selftest_cell_failed "本机一台跑的那一趟判段没跑（打了 CRASH_AMPLIFICATION_NOT_RUN step=judge 行、退 0）：那一行应当原样转打恰好一行、NOT_RUN_NO_GPU_ADAPTER 那一行不转、退 0（退 $judge_not_run_exit，输出 $(tr '\n' '|' <"$work/judge-not-run.out")）" \
      "看 run_on_this_host_alone：run.log 里行首 CRASH_AMPLIFICATION_NOT_RUN step=judge 的行原样转打"
  fi
  # 第 30 格：线程数只读服务交的（共用库文件头「线程数的缺省」）：服务交了 SINGLEFS_THREAD_CAP=7 取 7；没交取 .claude/heavy-thread-default 第一条非注释行的数
  cells=$((cells + 1))
  local registered_threads default_threads_seen
  registered_threads="$(grep -vE '^[[:space:]]*(#|$)' "$repository_root/.claude/heavy-thread-default" | head -n 1 | awk '{print $1}')"
  default_threads_seen="$(env -u SINGLEFS_THREAD_CAP bash -c 'source "$1"; SINGLEFS_THREAD_CAP=7 crash_amplification_default_threads; crash_amplification_default_threads' _ "$item_script_directory/crash-amplification-lib.sh" | tr '\n' ' ')"
  [[ -n "$registered_threads" && "$default_threads_seen" == "7 $registered_threads " ]] \
    || selftest_cell_failed "线程数应当取服务交的 SINGLEFS_THREAD_CAP（交了 7 取 7），没交取登记的重活默认并行度 $registered_threads（实际「$default_threads_seen」）" \
      "看 crash-amplification-lib.sh 的 crash_amplification_default_threads"
  # 第 31 格：上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」「测试目标」）：本机一台 CPU 跑、清单写这一格自己的目录；
  #   alpha 整个红 ⇒ 清单 alpha；beta 点名用例 one_case 红 ⇒ 添 beta::one_case；beta 点名 one_case 绿 ⇒ 那一项换掉；alpha 整个绿 ⇒ alpha 换掉（清单空）
  cells=$((cells + 1))
  printf '%s\n' '#!/usr/bin/env bash' \
    'if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_STAGE=record "* ]]; then echo "CRASH_AMPLIFICATION_RECORDED flow=stub" > "$2"; exit 0; fi' \
    'if [[ "${STUB_LAST_RUN_RED:-0}" == 1 ]]; then echo "test result: FAILED. 0 passed; 1 failed" > "$2"; exit 101; fi' \
    'echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=1 red=0" > "$2"' >"$work/last-run-local.sh"
  local last_run_list="$work/last-runs-cell/scripts__crash__crash-amplification-item.sh.tsv" last_run_seen="" last_run_step
  for last_run_step in "alpha 1" "beta 1 one_case" "beta 0 one_case" "alpha 0"; do
    read -r last_run_target last_run_red last_run_filter <<<"$last_run_step"
    SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs-cell" STUB_LAST_RUN_RED="$last_run_red" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 \
      CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/last-run-local.sh" \
      bash "$0" "$last_run_target" ${last_run_filter:+--test-filter "$last_run_filter"} >/dev/null 2>&1
    last_run_seen+="[$(tr '\n' ',' <"$last_run_list" 2>/dev/null || echo 没有清单)]"
  done
  [[ "$last_run_seen" == "[alpha,][alpha,beta::one_case,][alpha,][]" ]] \
    || selftest_cell_failed "上次红的清单：四趟之后应当依次是「alpha」「alpha、beta::one_case」「alpha」「空」，实际 $last_run_seen" \
      "看主流程开头装的 EXIT 钩子 item_write_last_run（经 scripts/gate/lib_reuse_checks.py last-run-target；红的测试目标一行一个，点名用例的只换那一项）"
  # 第 32 格：没开 GPU、开了双机 ⇒ 照样本机 CPU 一台跑：先录入段、再跑，DECISION where=local-cpu，退 0（不交双机驱动）
  cells=$((cells + 1))
  : > "$work/calls"
  local cpu_exit
  STUB_CALLS="$work/calls" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,1 \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target >"$work/cpu-across.out" 2>&1; cpu_exit=$?
  if [[ "$cpu_exit" != 0 || "$(tr '\n' ' ' <"$work/calls")" != "record local-run " ]] || ! grep -q '^CRASH_AMPLIFICATION_DECISION where=local-cpu ' "$work/cpu-across.out"; then
    selftest_cell_failed "没开 GPU、开了双机：应当本机 CPU 一台跑（先录入段、再跑）、打 where=local-cpu、退 0（退 $cpu_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" \
      "看主流程「没开 GPU」那一支"
  fi
  # 第 33–35 格：跑完运行目录里不剩它建的编译目录（文件头「编译目录」）：调用方没设 CARGO_TARGET_DIR，本机一台 CPU 跑；假的本机那一趟记下收到的编译目录、
  #   往里写一份产物。第 33 格正常退 0、第 34 格本机那一趟退 101（这一项照它退 101）、第 35 格被 TERM 停（假的那一趟睡 3 秒时只给这一项发 TERM，等它退出后走 EXIT，退 143）：
  #   收到的编译目录在 <家目录>/runs/<标签>/target（自证把家目录设成自己的临时目录），跑完不在了
  printf '%s\n' '#!/usr/bin/env bash' \
    'printf "%s\n" "${CARGO_TARGET_DIR-unset}" >> "$STUB_TARGET_LOG"' \
    '[[ -z "${CARGO_TARGET_DIR:-}" ]] || { mkdir -p "$CARGO_TARGET_DIR/release" && echo built > "$CARGO_TARGET_DIR/release/artifact"; }' \
    '[[ -z "${STUB_SLEEP:-}" ]] || sleep "$STUB_SLEEP"' \
    'if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_STAGE=record "* ]]; then echo "CRASH_AMPLIFICATION_RECORDED flow=stub" > "$2"; exit 0; fi' \
    'if [[ "${STUB_TARGET_RED:-0}" == 1 ]]; then echo "test result: FAILED. 0 passed; 1 failed" > "$2"; exit 101; fi' \
    'echo "CRASH_AMPLIFICATION flow=stub mode=single_host_cpu states_checked_here=1 red=0" > "$2"' >"$work/target-local.sh"
  local target_exit
  # 逐格点名：被 TERM 停的那一格只给自己起的那一个后台进程发一次，三格逐条调用、不在循环里发信号
  selftest_target_case() { # <格号:要的退出码>
    local target_case="$1" target_seen target_item target_exit target_attempt
    cells=$((cells + 1)); : > "$work/targets.log"
    if [[ "${target_case%%:*}" == 35 ]]; then
      env -u CARGO_TARGET_DIR STUB_TARGET_LOG="$work/targets.log" STUB_SLEEP=3 CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/target-local.sh" \
        bash "$0" some-target >/dev/null 2>&1 &
      target_item=$!
      for (( target_attempt = 0; target_attempt < 600; target_attempt++ )); do [[ -s "$work/targets.log" ]] && break; sleep 0.1; done
      kill -TERM "$target_item" 2>/dev/null
      if wait "$target_item"; then target_exit=0; else target_exit=$?; fi
    else
      env -u CARGO_TARGET_DIR STUB_TARGET_LOG="$work/targets.log" STUB_TARGET_RED="$([[ "${target_case%%:*}" == 34 ]] && echo 1 || echo 0)" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 \
        CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/target-local.sh" \
        bash "$0" some-target >/dev/null 2>&1; target_exit=$?
    fi
    target_seen="$(head -n 1 "$work/targets.log" 2>/dev/null)"
    if [[ "$target_exit" != "${target_case#*:}" || "$target_seen" != "$work"/runs/*/target || -e "$target_seen" ]]; then
      selftest_cell_failed "跑完运行目录里不剩它建的编译目录（$([[ "${target_case%%:*}" == 33 ]] && echo 正常退 0 || { [[ "${target_case%%:*}" == 34 ]] && echo 本机那一趟红 || echo 被 TERM 停; })）：本机那一趟收到「${target_seen:-没收到}」，退 $target_exit（要 ${target_case#*:}），跑完$([[ -n "$target_seen" && -e "$target_seen" ]] && echo 还在 || echo 不在)" \
        "看 run_directory 建好之后的 run_build_directory_begin 与 EXIT 钩子 item_finish（run_build_directory_end），TERM / INT 的 trap 退 143 / 130、等手里那一步退出再走 EXIT"
    fi
  }
  selftest_target_case 33:0
  selftest_target_case 34:101
  selftest_target_case 35:143
  # 第 36 格：调用方（崩溃放量入口）设了 CARGO_TARGET_DIR：本机那一趟照收到它、跑完不删
  cells=$((cells + 1)); : > "$work/targets.log"; mkdir -p "$work/caller-target"
  CARGO_TARGET_DIR="$work/caller-target" STUB_TARGET_LOG="$work/targets.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/target-local.sh" \
    bash "$0" some-target >/dev/null 2>&1; target_exit=$?
  if [[ "$target_exit" != 0 || "$(head -n 1 "$work/targets.log" 2>/dev/null)" != "$work/caller-target" || ! -f "$work/caller-target/release/artifact" ]]; then
    selftest_cell_failed "调用方设了 CARGO_TARGET_DIR 时本机那一趟应当照收到它、跑完不删（退 $target_exit，收到「$(head -n 1 "$work/targets.log" 2>/dev/null)」）" \
      "看 run_build_directory_begin：设了 CARGO_TARGET_DIR 就不建、不删"
  fi
  # 第 37 格：两台对称——哪几项是这一台的照多机配置判法 --emit-addresses 认（HOSTS 与这一台网卡上的地址求交），不看 HOSTS 次序、不读角色变量。
  #   同一份配置（双机没开）、显卡先后只点名 2001:db8::65 的卡：这一台网卡上的地址换成 2001:db8::65（多机配置判法只在自证里认 CONFIGURATION_SELFTEST_LOCAL_ADDRESSES）⇒
  #   那是这一台的卡，运行条件照过、本机一台跑；这一台是 127.0.0.1（照真网卡）⇒ 点名的是另一台的卡，退 78
  cells=$((cells + 1))
  local own_cards_decision other_cards_decision
  own_cards_decision="$(CONFIGURATION_SELFTEST_LOCAL_ADDRESSES=2001:db8::65 decision_of "$work/plan.log" 1,0 "2001:db8::65:2:10G")"
  other_cards_decision="$(decision_of "$work/plan.log" 1,0 "2001:db8::65:2:10G")"
  [[ "$own_cards_decision" == "where=local 0" && "$other_cards_decision" == " 78" ]] \
    || selftest_cell_failed "双机没开、显卡先后只点名 2001:db8::65：这一台是 2001:db8::65 时应当照过、本机一台跑（实际「$own_cards_decision」），这一台是 127.0.0.1 时应当退 78（实际「$other_cards_decision」）" \
      "看 crash-amplification-lib.sh 的 crash_amplification_host_addresses（照 --emit-addresses 认这一台，不照 HOSTS 次序）与 crash_amplification_cards_match_the_hosts"
  # 第 38 格：没设 SINGLEFS_CRASH_AMPLIFICATION_HOME 时运行目录是工作根下的 crash-amplification（共用库 crash_amplification_home_directory，文件头「日志」），不经 $HOME
  cells=$((cells + 1))
  local home_seen
  home_seen="$(env -u SINGLEFS_CRASH_AMPLIFICATION_HOME SINGLEFS_WORK_ROOT="$work/work-root" bash -c 'source "$1" && crash_amplification_home_directory' home-directory \
    "$item_script_directory/crash-amplification-lib.sh" 2>&1)"
  [[ "$home_seen" == "$work/work-root/crash-amplification" ]] \
    || selftest_cell_failed "没设 SINGLEFS_CRASH_AMPLIFICATION_HOME 时运行目录应当是工作根下的 crash-amplification（$work/work-root/crash-amplification），实际「$home_seen」" \
      "看 crash-amplification-lib.sh 的 crash_amplification_home_directory：工作根照 .claude/hooks/lib_work_root.py 取，不经 \$HOME"
  # 第 39 格：只供测试的变量（文件头「只供测试」）不在自证里设了（去掉 CRASH_AMPLIFICATION_ITEM_SELFTEST）⇒ 退 2、点名它、一趟都不起
  cells=$((cells + 1))
  local outside_exit
  : > "$work/calls"
  env -u CRASH_AMPLIFICATION_ITEM_SELFTEST STUB_CALLS="$work/calls" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    bash "$0" some-target >"$work/test-variables-outside.out" 2>&1; outside_exit=$?
  if [[ "$outside_exit" != 2 || -s "$work/calls" ]] || ! grep -qF 'CRASH_AMPLIFICATION_ITEM_SWITCHES' "$work/test-variables-outside.out" \
     || ! grep -qF 'CRASH_AMPLIFICATION_ITEM_LOCAL_RUN' "$work/test-variables-outside.out"; then
    selftest_cell_failed "只供测试的变量不在自证里设了应当退 2、点名设了的几个、一趟都不起（退 $outside_exit，调用记录「$(tr '\n' ' ' <"$work/calls")」：$(tr '\n' ' ' <"$work/test-variables-outside.out" | cut -c1-200)）" \
      "看参数解析之后那一段：没有 CRASH_AMPLIFICATION_ITEM_SELFTEST=1 而设了只供测试的变量就退 2"
  fi
  # 第 40 格：显卡先后写坏了（地址 192.0.2.99 不在 HOSTS 里），运行条件退 78（经 preflight），不经 preflight 直接调 crash_amplification_run_condition_holds 交回 1、打原因
  local malformed_cards malformed_text malformed_decision malformed_exit
  cells=$((cells + 1))
  malformed_cards="192.0.2.99:0:15G"; malformed_text="的地址「192.0.2.99」不在 HOSTS"
  malformed_decision="$(decision_of "$work/plan.log" 1,0 "$malformed_cards")"
  CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$malformed_cards" \
    bash -c 'source "$1" && crash_amplification_run_condition_holds "$2"' run-condition "$item_script_directory/crash-amplification-lib.sh" "$work" \
    >"$work/malformed-cards.out" 2>&1; malformed_exit=$?
  if [[ "$malformed_decision" != " 78" || "$malformed_exit" != 1 ]] || ! grep -qF "$malformed_text" "$work/malformed-cards.out"; then
    selftest_cell_failed "显卡先后「$malformed_cards」写坏了：运行条件应当退 78（实际「$malformed_decision」）、直接调交回 1 并打「$malformed_text」（实际交回 $malformed_exit：$(tr '\n' ' ' <"$work/malformed-cards.out" | cut -c1-200)）" \
      "看 crash-amplification-lib.sh 的 crash_amplification_card_list_problem：从右切两刀、地址要是 HOSTS 里的一项"
  fi
  # 第 41 格：判段跑的过程中源码变了（文件头「源码中途变了」）⇒ 本次未跑，退 77，不落红：假的本机那一趟（STUB_MUTATE_SOURCE）往造出来的小树里一个文件追加一行。
  #   没开 GPU、本机那一趟里变、那一趟还退 101（红）：退 77，打 CRASH_AMPLIFICATION_NOT_RUN step=judge 那一行，不转打那一趟的汇总行（不退 101）；
  #   开了 GPU、排计划那一趟里变：退 77，同样打那一行，不起本机那一趟（调用次序只有 record placement）
  cells=$((cells + 1))
  local changed_cpu_output changed_cpu_exit changed_cpu_calls changed_gpu_output changed_gpu_exit changed_gpu_calls
  : > "$work/calls"
  changed_cpu_output="$(STUB_CALLS="$work/calls" STUB_MUTATE_SOURCE="$work/source-tree/crates/changed.rs" STUB_MUTATE_IN=run STUB_RUN_RED=1 CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target 2>/dev/null)"; changed_cpu_exit=$?
  changed_cpu_calls="$(tr '\n' ' ' <"$work/calls")"
  : > "$work/calls"
  changed_gpu_output="$(STUB_CALLS="$work/calls" STUB_MUTATE_SOURCE="$work/source-tree/crates/changed.rs" STUB_MUTATE_IN=placement CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 \
    CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" GPU_CARDS_BY_PRIORITY="$local_only" CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target 2>/dev/null)"; changed_gpu_exit=$?
  changed_gpu_calls="$(tr '\n' ' ' <"$work/calls")"
  if [[ "$changed_cpu_exit" != 77 || "$changed_cpu_calls" != "record local-run " ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment reason=跑的过程中源码变了' <<<"$changed_cpu_output" \
     || grep -q '^CRASH_AMPLIFICATION ' <<<"$changed_cpu_output" \
     || [[ "$changed_gpu_exit" != 77 || "$changed_gpu_calls" != "record placement " ]] \
     || ! grep -q '^CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment reason=跑的过程中源码变了' <<<"$changed_gpu_output"; then
    selftest_cell_failed "判段跑的过程中源码变了应当退 77、打 CRASH_AMPLIFICATION_NOT_RUN step=judge …跑的过程中源码变了、不转打那一趟的汇总行、不落红（没开 GPU、本机那一趟红：退 $changed_cpu_exit，调用次序「$changed_cpu_calls」，输出「$(tr '\n' ' ' <<<"$changed_cpu_output" | cut -c1-160)」；开了 GPU、排计划那一趟里变：退 $changed_gpu_exit，调用次序「$changed_gpu_calls」，输出「$(tr '\n' ' ' <<<"$changed_gpu_output" | cut -c1-160)」）" \
      "看 item_note_source_after_record 与 item_exit_if_source_changed：录入段跑成后记摘要，排计划与本机那一趟起跑前、收尾后各核一次，对不上打本次未跑、退 77，核完再转打汇总行"
  fi
  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：双机开着与没开都本机一台跑（不交双机驱动）、没开双机却点名第二台的卡退 78、开了双机第二台没卡退 78、几条流累加、没有计划行认不出原因退 247、卡用不了退 78、算法判不了退 248、开了 GPU 没写卡退 78、本机那一趟卡用不了退 78、带值的选项缺值退 2、运行条件读多机配置里的显卡先后、三趟收到的卡是 task-server 交的那一张（不读多机配置里的显卡先后）、不再自己清场、环境里没有服务交的卡退 78、认不出的选项退 2、排计划那一趟借不到显存一项都没跑到退 77（别的用例名的那一行退 247、那一趟退了非 0 原样退）、几条流里没借到的逐条报本次未跑而借到的照判、没设 SINGLEFS_MULTI_HOST_CONFIG 时多机配置按 git common-dir 找、多机配置判法认显卡的两个键、结论版本对不上与没编进 gpu 特性都退 78、不在临时 prune-server 底下起的自证退 2、录入段没跑成就停（判段一趟都不起）、调用方没送按段登记文件时两趟都送本树那一份的绝对路径（送了照它）、照崩溃点重放用例的样子造的用例（录入段只打 CRASH_LIBRARY_RECORDED）点名了也照跑录入段并算录入成了、本机那一趟判段没跑时转打 CRASH_AMPLIFICATION_NOT_RUN step=judge 那一行、线程数取服务交的 SINGLEFS_THREAD_CAP（没交取登记的重活默认并行度）、跑完写上次红的清单（点名用例的只换那一项）、没开 GPU 开了双机时本机 CPU 一台跑、跑完运行目录里不剩它建的编译目录（正常、红、被 TERM 停）而调用方设的 CARGO_TARGET_DIR 不删、哪几项是这一台的卡照 --emit-addresses 认（这一台是 HOSTS 里的 IPv6 那一项时它的卡照过、同一份配置在 127.0.0.1 上退 78）、没设放量家目录时运行目录在工作根下、只供测试的变量不在自证里设了退 2、显卡先后里地址不在 HOSTS 里退 78、判段跑的过程中源码变了记本次未跑退 77 不落红，$cells 格都对"
  exit 0
}
if (( selftest )); then run_selftest; fi

[[ -n "$test_target" ]] || usage

# 只供测试的变量（文件头「只供测试」）只在自证里认：没有 CRASH_AMPLIFICATION_ITEM_SELFTEST=1 而设了，退 2、点名、一趟都不起。
#   弄坏开关 test-variables-honored-outside-selftest 下照认
if [[ "${CRASH_AMPLIFICATION_ITEM_SELFTEST:-}" != 1 ]] && ! item_break_is test-variables-honored-outside-selftest; then
  test_only_set=()
  for test_only_variable in CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY CRASH_AMPLIFICATION_ITEM_SWITCHES CRASH_AMPLIFICATION_ITEM_CARDS CRASH_AMPLIFICATION_ITEM_LOCAL_RUN CRASH_AMPLIFICATION_ITEM_SOURCE_ROOT; do
    [[ -n "${!test_only_variable:-}" ]] && test_only_set+=("$test_only_variable")
  done
  if (( ${#test_only_set[@]} )); then
    echo "  ✗ 设了只供测试的 ${test_only_set[*]}，这一趟不是自证：一趟都没起" >&2
    echo "  → 怎么办：unset ${test_only_set[*]} 再跑（只给 $0 --selftest 用）" >&2
    exit 2
  fi
fi

# 只判不跑（自证用）：拿造好的日志判跑不跑
if [[ -n "${CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY:-}" ]]; then
  refused_exit_code=""; decided_not_run_line=""
  decide_where_to_run "$CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY" 0; decision=$?
  case "$decision" in
    0) exit 0 ;;
    20) report_placement_not_run "$decided_not_run_line"; item_break_is not-run-exits-0 && exit 0; exit 77 ;;
    *) exit "${refused_exit_code:-247}" ;;
  esac
fi

# 按段登记（文件头「按段登记」）：调用方没送就送本树那一份的绝对路径；弄坏开关 segments-registry-not-sent 下不送
if [[ -z "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY:-}" ]] && ! item_break_is segments-registry-not-sent; then
  export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY="$repository_root/.claude/gate.d/crash-point-replay-segments.tsv"
fi

# 上次红的清单（文件头「上次红的清单」）：从这里起每一种退法都经 EXIT 钩子写一次；只判不跑的那一支不写
item_write_last_run() { # <这一趟的退出码>
  item_break_is last-run-unwritten && return 0   # 弄坏开关：跑完不写
  python3 "$repository_root/scripts/gate/lib_reuse_checks.py" last-run-target scripts/crash/crash-amplification-item.sh "$test_target" "$1" \
    ${test_filter:+--test-filter "$test_filter"} --target-file "$repository_root/crates/singlefs-verification/tests/$test_target.rs" >/dev/null 2>&1 \
    || echo "  ! 上次红的清单没写成（scripts/gate/lib_reuse_checks.py last-run-target），这一趟的判定不受影响" >&2
}
# EXIT 钩子：写上次红的清单，再删这一项自己建的编译目录（文件头「编译目录」；没建的不动，删不成而本来要退 0 的改退 1）。
#   被 TERM / INT 停时经 exit 143 / 130 走到这里（共用库本机那一趟收到 TERM 先转给它起的 cargo、等它退出，再放回这里的处理）
item_finish() {
  local finish_exit=$?
  trap - EXIT
  item_write_last_run "$finish_exit"
  if ! run_build_directory_end $(item_break_is run-target-kept && echo keep); then
    (( finish_exit != 0 )) || finish_exit=1
  fi
  exit "$finish_exit"
}
source "$item_script_directory/../hosts/run-build-directory.sh"
trap item_finish EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
# 开了 GPU 而环境里没有 task-server 交的卡：共用库交回 2，这里退 78、一趟都不起（文件头「退出码」78）
crash_amplification_prepare || exit 78
if [[ -n "${CRASH_AMPLIFICATION_ITEM_SWITCHES:-}" ]]; then
  IFS=, read -r gpu_switch _ <<<"$CRASH_AMPLIFICATION_ITEM_SWITCHES"
fi
# 只供测试（文件头「只供测试」）：换掉本机那一趟。环境照真的那一趟组（crash_amplification_environment_for_this_host），只把起 cargo 换成假的
if [[ -n "${CRASH_AMPLIFICATION_ITEM_LOCAL_RUN:-}" ]]; then
  crash_amplification_run_on_this_host() {
    crash_amplification_environment_for_this_host "$1" "${@:3}"
    env "${this_host_environment[@]}" CRASH_AMPLIFICATION_ITEM_LOCAL_TEST_ARGUMENTS="${cargo_test_arguments[*]}" bash "$CRASH_AMPLIFICATION_ITEM_LOCAL_RUN" "$@"
  }
fi
crash_amplification_home="$(crash_amplification_home_directory)" || exit 2
run_directory="$crash_amplification_home/runs/$label"
mkdir -p "$run_directory" || { echo "  ✗ 建不了 $run_directory" >&2; echo "  → 怎么办：看放量家目录 $crash_amplification_home 可不可写，或把 SINGLEFS_CRASH_AMPLIFICATION_HOME 指到可写的目录" >&2; exit 2; }
run_build_directory_begin "$run_directory" || exit 78   # 这一项的编译目录（文件头「编译目录」），EXIT 钩子 item_finish 删

# 两段：先在本机跑录入段，把这一项的流程写进判定库（prune-server）；之后排计划、本机判每一趟都只跑判段（从服务里读）
crash_amplification_record_on_this_host "$run_directory/record.log"; record_exit=$?
if (( record_exit != 0 )) && ! item_break_is record-failure-ignored; then exit "$record_exit"; fi
export SINGLEFS_CRASH_AMPLIFICATION_STAGE=judge

# 源码中途变了（文件头「源码中途变了」）：录入段跑成后记下被测源码的摘要，判段每一趟起跑前与收尾后再算，对不上记本次未跑、退 77，不落红。
#   摘要算哪棵树：本脚本所在的仓根；只供测试的 CRASH_AMPLIFICATION_ITEM_SOURCE_ROOT 换成别的树（文件头「只供测试」）。弄坏开关 record-judge-source-unchecked：不核
item_source_root="${CRASH_AMPLIFICATION_ITEM_SOURCE_ROOT:-$repository_root}"
source_digest_recorded=""
item_note_source_after_record() {
  local digest
  if digest="$(crash_amplification_source_digest "$item_source_root" 2>&1)"; then
    source_digest_recorded="$digest"
    echo "  · 录入段之后记下被测源码摘要 ${digest%% *}（${digest#* }，$item_source_root）"
  else
    echo "  ! 录入段之后算不出被测源码摘要，判段前后不核源码变没变：$digest" >&2
  fi
}
item_exit_if_source_changed() { # <哪一刻，写进原因>：此刻的摘要和录入段之后记下的对不上（或算不出）就打本次未跑那一行、退 77；对得上、没记下、开关打开都往下
  local digest_now shown_now
  [[ -n "$source_digest_recorded" ]] || return 0
  item_break_is record-judge-source-unchecked && return 0
  if digest_now="$(crash_amplification_source_digest "$item_source_root" 2>&1)"; then
    [[ "${digest_now%% *}" != "${source_digest_recorded%% *}" ]] || return 0
    shown_now="${digest_now%% *}"
  else
    shown_now="算不出（$digest_now）"
  fi
  echo "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment reason=跑的过程中源码变了，判段没在录入段那份源码上判完（录入段之后 ${source_digest_recorded%% *}，$1 $shown_now）"
  echo "  ! 本次未跑（这不是通过）：跑的过程中源码变了，判段没在录入段那份源码上判完：录入段之后被测源码摘要 ${source_digest_recorded}，$1 $shown_now（$item_source_root 下 Cargo.toml、Cargo.lock、.cargo/、crates/）；这一趟的结果不记红也不记绿" >&2
  echo "  → 怎么办：等树里的改动停下（别的会话的补丁打完）再重跑同一条命令，录入段与判过的块键没变的照服务里的复用" >&2
  exit 77
}
item_note_source_after_record

run_on_this_host_alone() { # 本机一台跑整份，打它的汇总行；退 0，或照 run.log 分类之后该退的码
  item_exit_if_source_changed "本机那一趟起跑前"
  crash_amplification_run_on_this_host 0/1 "$run_directory/run.log"; local run_exit=$?
  item_exit_if_source_changed "本机那一趟收尾后"
  grep -hE '^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL)? ' "$run_directory/run.log" 2>/dev/null
  # 判段没跑（文件头「输出」：没编进 gpu 特性的二进制遇到 ENABLE_GPU=1）那一趟打的本次未跑行原样转打；弄坏开关 judge-not-run-dropped 下不转
  item_break_is judge-not-run-dropped || grep -h '^CRASH_AMPLIFICATION_NOT_RUN step=judge ' "$run_directory/run.log" 2>/dev/null
  if [[ "$run_exit" == 0 ]]; then return 0; fi
  local exit_code
  exit_code="$(crash_amplification_exit_code_of_a_failed_run "$run_exit" "$run_directory/run.log")"
  echo "  ✗ 本机那一趟退 $run_exit；日志 $run_directory/run.log" >&2
  grep -h -A 3 'panicked at' "$run_directory/run.log" 2>/dev/null | head -8 | sed 's/^/    /' >&2
  if exit_code_is_explained "$exit_code"; then
    crash_amplification_explain_exit_code "$exit_code"
  else
    echo "  → 怎么办：看日志里 test result 与 panicked 那几行；判过的块已经在库里，重跑同一条命令接着判" >&2
  fi
  return "$exit_code"
}

# ① 没开 GPU：本机 CPU 一台跑
if (( ! gpu_switch )); then
  echo "CRASH_AMPLIFICATION_DECISION where=local-cpu reason=gpu-is-off placed_by=task-server"
  run_on_this_host_alone
  exit $?
fi

# ② 开了 GPU：在本机排派活计划（卡是服务交的那一张，共用库已取好）
item_exit_if_source_changed "排计划那一趟起跑前"
crash_amplification_run_on_this_host 0/1 "$run_directory/placement.log" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1; placement_exit=$?
item_exit_if_source_changed "排计划那一趟收尾后"
grep -h '^CRASH_AMPLIFICATION_PLACEMENT ' "$run_directory/placement.log" 2>/dev/null
refused_exit_code=""; decided_not_run_line=""
decide_where_to_run "$run_directory/placement.log" "$placement_exit"; decision=$?
case "$decision" in
  0) run_on_this_host_alone; exit $? ;;
  20) # 一项都没跑到（文件头步骤 ②、退出码 77）：不跑；弄坏开关 not-run-exits-0 下退 0
    report_placement_not_run "$decided_not_run_line"
    item_break_is not-run-exits-0 && exit 0
    exit 77 ;;
  *) exit "${refused_exit_code:-247}" ;;
esac
