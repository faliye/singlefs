#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command cargo git
# task-server-gpu-arguments: *   不给实验号就全跑、含点到 CHECKER_TIER_FEATURES 的行，哪几行要卡开跑前才算得出，所以一律由服务在最外层清场、分卡（task-server/task_server_lib.py 文件头「GPU」）
# 复跑已入库的实验，把今天的输出和 research/results/ 里那份逐字节比对。
#
#   bash scripts/experiment-apparatus/replay.sh [实验号...]      # 不给参数就全跑
#   bash scripts/experiment-apparatus/replay.sh --list-items     # 逐行打可点名的实验号（几行、二进制或驱动），不编、不跑
#
# 入口脚本（.claude/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」，C641）：可点名的项是复跑表的实验号。
#   --list-items 与「实验号写错」都在建本轮输出目录、truncate 镜像之前判（replay_table_experiments 从本文件原文读复跑表）：
#   表里没有的实验号（选项只认 --admission-only、--selftest、--list-items）一行都不跑、退 2；裸数字照旧由「没带 E」那一判点名。
#   每一行一条结果行（实验号、二进制、判定、说明），末尾 REPLAY_TOTAL 一行报派出、收回、现判、复用、拒算与各判定的条数。
#   弄坏开关 REPLAY_BREAK=unknown-experiment-accepted：表里没有的实验号照收（匹配 0 行、一行都不跑也退 0）。
#
# 为什么要有它：kb-discipline.md「所有历史数据都只是参考」——每条实测都绑在当时那个
# 构建上，要拿它支撑新结论，先复跑一遍确认它今天还成立。手工比对会被抄第二遍，所以做成脚本。
#
# 三条完整性闸（command-safety.md「结果抓取要有完整性闸」）：
#   1. 跑前删本轮输出，不许跨轮复用；
#   2. 收尾行必须是 `name=done emitted=N`，且 N 必须等于本轮实际 E7RESULT 行数
#      （口径：emitted 把 config 行与 done 行自己都算进去）；
#   3. 退出码非 0 一律判红，不许当成「跑过了」。
#   三条各有弄坏开关与自证一格（replay_entry_selftest）：闸 1 的删旧记录在 replay_one 里无条件做，自证一格看树里的留存产物先于上一趟取回的那一份；闸 2 的弄坏开关 REPLAY_BREAK=emitted-unchecked、闸 3 的 exit-code-ignored 各配一格。
#
# 编译目录（自己用完自己删，不预留生成物）：调用方没设 CARGO_TARGET_DIR 时，这一趟在
#   ${TMPDIR:-/tmp} 下建自己的运行目录 singlefs-replay-build-<进程号>-XXXXXX（scripts/hosts/run-build-directory.sh 的 run_build_directory_begin），
#   研究工作区（scripts/experiment-apparatus/）与仓根工作区的 cargo 都编进它的 target/（两个工作区的二进制不重名），研究二进制一律从 $REPLAY_RELEASE_DIRECTORY 起；
#   不再往固定的 scripts/experiment-apparatus/target/、<仓根>/target 编。调用方设了的照它编、不删（相对路径按仓根算）。
#   驱动在本轮输出目录（$OUT_DIR）里拷 crates/ 去编的副本（E168 补过的那一份、E175 的仓副本）各编进副本自己的 target/，E171 逐行驱动的线道、E165 变异杀伤界的变体
#   也各在本轮输出目录里编；收尾时这些编译目录逐个删（replay_remove_copy_build_directories：认带签名的 CACHEDIR.TAG，不跟符号链接），产物、本轮输出目录与副本的源码照留。
#   正常结束、出错、被 TERM / INT 停都在 EXIT 钩子 replay_finish 上删：被停时先给在跑的每一行转 TERM（replay_stop → replay_one 再转给手里那一步）、等它们退；
#   删之前核本机没有进程开着里面的文件（scripts/hosts/run_build_directory.py），有就不删、报出进程、整趟退非 0。判定库、留存产物与本轮输出照留。
#   驱动里在前台一步一步起的命令（cargo run 这一类）转不到 TERM：被停时那一步还开着编译目录的，照上面不删、报出进程。
#   弄坏开关 REPLAY_BREAK=build-directory-kept、term-not-forwarded、copy-build-directories-kept（自证 replay_build_directory_selftest 那五格）。
#
# 计时行的负载关（C350，.claude/kb/checks-owed.md）：判据写 timing 的行，开比之前先读这一趟新产物每一条 `name=done` 收尾行的负载字段。
#   约定的收尾行形态：`E7RESULT name=done emitted=N loadavg1=<1 分钟负载> cpus=<在线 CPU 数>`（装置从 /proc/loadavg 与 available_parallelism 取）。
#   emitted 照旧只数 config 行与 done 行自己，两个字段不进 emitted；逐字节比对之前两边的这两个字段都删掉（strip_load_fields）。
#   loadavg1 / cpus 超过 $REPLAY_TIMING_LOAD_RATIO（默认 0.5）的那一行记「本次不判」：不比结构、不判区间断言，既不红也不绿，
#     往 $GATE_NOT_RUN_FILE 报一行；多段产物取各段最大的比值；整趟没有一行判得了（判的全是本次不判）退 77。
#   收尾行没有这两个字段（或值不是数）的判红，理由点名产物与「收尾行没有 loadavg1 / cpus」；
#     例外是 .claude/gate.d/timing-load-field-lag.tsv 登记的存量产物（一行两列、制表符分隔：复跑表第 4 列的留存产物名、理由）：照旧判，成功句报「滞后 K 份」。
#     滞后表只缩不涨：装置补了字段、留存产物换成带字段的新产物，就删那一行；不往里加新行。
#     登记的名字不是复跑表某条 timing 行的第 4 列、没写理由、重复登记、或留存产物自己已经有字段，开跑前判红退 2。
#
# 准入（门禁与实验共用 scripts/gate/admission.py，登记表 .claude/gate.d/stage-inputs.tsv）：登记了准入的实验，
# 每一行开跑之前先问一次（实验键 = 这一行的行键去掉期望产物，admission.py 文件头「复跑表每一行的行键」），实验键与留存产物头上记的相同就记「输入没变」、
# 不跑也不判红；产物头的 `E7INPUT` 行比对前删掉。
# 每一行按自己的准入键问、比自己的产物：键在准入那一段的 ADMISSION_ROW_KEYS 里按行指明，没指明的是实验号；同一个实验几行又登记了准入的，各行各指一个键。
#   bash scripts/experiment-apparatus/replay.sh --admission-only [实验号...]   只问准入，不编、不跑、不比
#   bash scripts/experiment-apparatus/replay.sh --selftest                    假 cargo 演构建的两种结局（replay_selftest），不编、不跑实验
# 复用（test-script-discipline.md「判定可复用：按输入存，按前缀剪」）：每一行复跑一条判定，行键是装置 main 起的单元闭包、驱动函数的 shell 单元闭包、
#   这一行的参数、GPU 开关与特性集、期望产物内容、工具链、解析出的依赖版本（scripts/gate/admission.py replay-row-inputs，算法在它文件头「复跑表每一行的行键」），
#   不带实验号与文件名。先算全部行的行键、判完准入，再定要现跑的几行：一行都不用现跑就不试编 gpu 特性、不编；要现跑的行里没有要卡的就不试编 gpu 特性（文件头「GPU」）；要现跑的只有 research 二进制的行时，
#   只编那几个 bin（cargo build --bin <名>…），有驱动函数的行才整编。判定库里这把键有一份判过的记录（字节一致、仅计时不同两种）就不跑、记「复用」，
#   结论区间断言照常对记录里留的那一份产物判；没有就现跑，判成这三种之一的连同新产物、组成清单、判定日期与跑法写进判定库。
#   键对不上而要现跑的那一行，开跑前在 stderr 报一行变了的单元（与这一行上一次判过的组成清单比）。行键拒算的那一行照跑、不复用、不入库，计进 refused=。
#   判定库在 prune-server 里（族 unit-verdicts、种类 replay-row，键空间在 scripts/prune-server/prune_server_client.py 文件头「键空间」）：
#   keys/<行键>/verdict、line、product、finished、how、composition 各一条，items/<前三列的 sha256> 一条记这一行最近一次的键；
#   库名 <这棵树的 git common-dir>/singlefs-unit-verdicts/replay-row 只当名字（键里只用种类名，命名空间只在临时服务底下给），本地不建、不读、不写。读写都经 scripts/gate/admission.py 的
#   library-files（它文件头「library-files」）：行键算好之后把这一趟要的那几把键的记录一次拉进本轮输出目录里的中转目录 row-records-from-prune-server/
#   （开跑前那一判与复用哨兵读它、复用来的那一行的产物从它判结论区间），一行判完把那一条记录写进中转目录再写进服务（verdict 最后写：它在就算整条在），
#   中转目录在这一趟收尾删掉（replay_finish）。服务连不上（replay-row-inputs、library-files 退 75）：整趟退 75，不退回本地、不当库里没有现跑
#   （弄坏开关 prune-server-failure-swallowed 下不按 75 办，改错的样子）。
#   不在 git 工作树里（判定库没有名字）退 2；行键整份算不出（replay-row-inputs 退 75 之外的非 0）、中转目录建不了、记录拉不下来（library-files fetch 退 75 之外的非 0）退 1；
#   都是开跑之前、一行都没跑，不照「不复用、不记」全跑（弄坏开关 row-library-failure-tolerated 下照「不复用、不记」全跑）。只记不删，键不按时限过期。
#   一行判完由 replay_one 在 replay_judge_fresh_product 外面记（remember_row_outcome：判绿的三种记绿，对不上与跑不了记红）；判法不碰库：存库起 admission.py，进了判法闭包每一行的键都拒算。
#   记红、撤绿、下趟现判（D13（验证路线） 已定项 17）：一行判成对不上（drift，含判据写错）的，照同一把行键记 verdict=drift，盖掉这把键原有的字节一致 / 仅计时不同 / 已归档；下一趟这一行现判、开跑前在 stderr 写「要现跑：上次判红」，不当绿复用，也不进复用哨兵的候选；
#   现判又判绿就盖回绿。这一行现跑了而跑不了（broken：装置或驱动退非 0、完整性闸不过、产物头接不上、缺负载字段这几种）照同一种写法记 verdict=broken、撤绿、下一趟现判
#   （开跑前同样写「要现跑：上次判红」；只在复用哨兵强制现判时碰得到：库里有绿的行别的时候不现跑）。本次不判（负载高）不是判定，不记，也不撤绿
#   （弄坏开关 red-not-recorded 下对不上的不记、broken-not-recorded 下跑不了的不记）。
#   每一趟末尾打一行 REPLAY_TOTAL dispatched=<派出> collected=<收回> judged=<现判> reused=<复用> refused=<行键拒算> pass= timing_only=
#     drift= broken= claims_bad= unchanged= load_skipped=（门禁 research-build-and-replay 的 experiment-replay 格读它）。
#   留存产物不在树里（按「每次提交删上一次的实验记录」归档进了版本库）的行：从 git 历史取回再逐字节比（replay_judge_fresh_product：git log --all --diff-filter=D 找最近一次删它的提交，
#   git show <提交>^:research/results/<名> 读回，放本轮输出目录；结果行写出处）；取回的照字节一致、仅计时不同、对不上判，取不回判「取不回」（跑不了，红）；留存产物不在就不比的一档不存在。
# E182 的片结果库（E182（被抛弃根读不出的三条岔路）两个段，driver_e182 与 driver_e182_units_kept）：装置读写的是目录、没有缺省的本地库（不设 E182_VERDICT_LIBRARY 退 2），
#   照两份长跑脚本（scripts/experiment-apparatus/e182-single-device-long-run.sh 文件头「片结果库」）开这一趟自己的中转目录 $OUT_DIR/e182-shards-from-prune-server/<种类>/：
#   这一趟点到这两个驱动的行时，开跑前经 admission.py library-files fetch 把 prune-server 里那两种（e182-single-device、e182-single-device-units-kept，
#   库名 <这棵树的 git common-dir>/singlefs-unit-verdicts/<种类>，与长跑脚本同一份）拉进来，两个驱动照 E182_VERDICT_LIBRARY 指到各自那一份，
#   各行收齐之后经 library-files store 并回（replay_store_e182_shards），收尾随 replay_finish 删。fetch 或 store 退 75 整趟退 75，别的非 0 整趟退 1。
#   拉与并回都在驱动外面做：驱动进行键，驱动里起 admission.py 会连上它的程序入口、这两行的键拒算。
#   弄坏开关 e182-transit-unset 下两个驱动不设 E182_VERDICT_LIBRARY。
# 驱动要读的输入（别的实验的产物、拿来当期望值的留存产物、冻结根、导出根；research/ 可以整个删掉，所以产物缺了是常事）：驱动经 replay_inputs_present 开跑前核。
#   research/results/ 底下的缺了先从 git 历史取回、放回原处（本趟收尾删掉自己放回去的，replay_finish）；取不回，或冻结根、导出根不在，这一行记「跑不了」（红）：
#   stderr 里每缺一样一行「✗ 输入不在 test=<这一行> …」，出路在那一行里。冻结根与导出根在固定的位置：工作根下的 replay-frozen-roots/<名>（replay_frozen_root），不经环境变量指。
#   装置自己读不到输入退 77 并打 NOT_RUN_<原因> test=… 行的（研究一侧 scripts/experiment-apparatus/lib.rs 的 read_archived_products_or_exit_not_run）同样记「跑不了」，
#   那几行留在 <实验号>.err 里。整趟没有一行判得了（判的全是负载高本次不判）退 77。
# 开跑前的构建输出出两份日志（scripts/hosts/two-logs.sh）：$OUT_DIR/build.log 全量，旁边的 build.problems.log 只收 error、warning: 与位置行、非 0 退出码，行前带全量日志里的行号；
# 构建失败退 2，stderr 打第一处 error、日志末尾 20 行、日志路径、错误警告日志路径与单独重现的命令。
# GPU（test-script-discipline.md「用上现有的硬件加速」；resolve_checker_tier_features）：多机配置（scripts/crash/crash-amplification-lib.sh 文件头「配置」：
#   ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}）写了 ENABLE_GPU=1 时，起 验证包装置（用到给定状态引擎的 e156、e158、e161、e168、e170、e171 各装置）的
#   cargo run 一律带 --features gpu（数组 CHECKER_TIER_FEATURES）；开跑前先 cargo build --release -p singlefs-verification --features gpu --lib
#   试编一次，输出出两份日志落进 $OUT_DIR/gpu-build.log 与旁边的 gpu-build.problems.log。没开（文件不在、没写这个键、写 0）一个字都不变，不起这一次试编。
#   配置读不出（SINGLEFS_MULTI_HOST_CONFIG 点名的文件不在、文件不可读、读的时候出错、ENABLE_GPU 写了 0、1 之外的值），或开了 GPU 而 gpu 特性编不过：退 78，一条实验都没跑，
#   stderr 写原因与出路；不悄悄退回不带特性去跑（那样装置把判段当没编进 gpu 特性，报本次未跑或 panic）。
#   清场与分卡（.claude/rules/verification.md「凡用 GPU 先清场，跑完不复原」）归统一任务服务：服务在最外层给整条复跑套一次清场，清场之后的卡经环境变量
#   GPU_CARDS_BY_PRIORITY 交下来，装置照它取卡；本脚本不另套清场包装（service_gpu_cards_present）。开了 GPU 而环境里没有 GPU_CARDS_BY_PRIORITY：退 78，
#   一条实验都没跑，stderr 写原因与出路。没开 GPU 一个字都不变。
#   GPU 开关与特性集进点到 CHECKER_TIER_FEATURES 的那几行的行键（文件头「复用」）：开关一换，只那几行现跑。开关先读（read_gpu_switch，不编），
#   试编与核卡（prepare_checker_tier_features）只在要现跑的行里有要卡的行时做：哪一行要卡照 admission.py replay-row-inputs 的第 4 列
#   （驱动闭包里点到 CHECKER_TIER_FEATURES 的行是 1，与行键里加 line:gpu 的同一判，本脚本不另判）。要现跑的行一行要卡的都没有：开了 GPU 也不试编、
#   不要 GPU_CARDS_BY_PRIORITY，CHECKER_TIER_FEATURES 是空的，stderr 写一句「都不点 CHECKER_TIER_FEATURES」。
set -uo pipefail
# 内存额度、线程与挑机器不在本脚本里定：整条命令交 task-server（task-server/task submit -- bash scripts/experiment-apparatus/replay.sh <实验号…>），
# 内存额度按记录由服务给、撞上限由服务放大重排；驱动里起 cargo run 与二进制都直接起，不套内存包装，不写死上限。
if [[ -n "${REPLAY_BREAK:-}" && "$REPLAY_BREAK" != guard-off && "${1:-}" != --selftest ]] \
    && [[ -z "${REPLAY_SELFTEST_MARK:-}" || "$(cat -- "$REPLAY_SELFTEST_MARK" 2>/dev/null)" != replay-selftest ]]; then
  echo "  ✗ REPLAY_BREAK 是弄坏开关，只在自证（--selftest）里认；平常跑设了就拒，这一次什么都没跑" >&2
  echo "     → 怎么办：去掉 REPLAY_BREAK 再跑；要证红就跑 REPLAY_BREAK=<项> bash scripts/experiment-apparatus/replay.sh --selftest" >&2
  exit 2
fi
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根 $(dirname "$0")/../.."; echo "     → 怎么办：从仓根照文件头的用法起：bash scripts/experiment-apparatus/replay.sh"; exit 2; }
REPLAY_REPOSITORY_ROOT="$PWD"   # replay_frozen_root 读工作根的库用绝对路径：自证在空目录里起驱动，当前目录不是仓根
source scripts/hosts/heavy-threads.sh
REPLAY_BROKEN="${REPLAY_BREAK:-}"   # 弄坏开关（只给 --selftest 证红用），见 replay_selftest 上面那一段
# 从这里往下载入期执行到的行找不到命令（一句注释丢了 #、写坏的命令名）时，bash 报一句就照样往下跑（脚本不带 set -e）：
# 记下找不到的命令名，--selftest 最后一格（load-commands-not-found）要它是空的。弄坏开关 load-errors-unchecked：不记。
# bash 在另一个执行环境里调 command_not_found_handle（改变量带不回来），所以写进一个描述符：开一个临时文件、拿到描述符就删掉文件（不留东西），
# 处理函数往描述符写，replay_commands_not_found 从 /dev/fd/<号> 从头读回
replay_not_found_record="$(mktemp "${TMPDIR:-/tmp}/replay-commands-not-found-XXXXXX")" || { echo "  ✗ 建不了临时文件（\$TMPDIR=${TMPDIR:-/tmp}）"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑"; exit 2; }
exec {REPLAY_NOT_FOUND_FD}>"$replay_not_found_record"
rm -f -- "${replay_not_found_record:?}"
command_not_found_handle() {
  [[ "$REPLAY_BROKEN" == load-errors-unchecked ]] || printf '%s\n' "$1" >&"$REPLAY_NOT_FOUND_FD"
  printf '%s: %s: command not found\n' "${BASH_SOURCE[1]:-replay.sh}" "$1" >&2
  return 127
}
replay_commands_not_found() { cat "/dev/fd/$REPLAY_NOT_FOUND_FD" 2>/dev/null; }

# 照写死的清单拷 crates/ 去编译的副本（只剩 replay_copy_closure_into 的弄坏开关那一支）都带上每台默认重活线程数的登记文件
# .claude/heavy-thread-default：harness 包经 include_str! 在编译时读它（crates/singlefs-verification/src/heavy_threads.rs），副本里没有就编不过。
# 拷 crates 的那一行在同一行里接着调它（replay_selftest 的 copy-lines-with-registry 那一格静态核）。在仓根调（脚本开头已 cd 到仓根）。
# 弄坏开关 REPLAY_BREAK=thread-registry-not-copied：什么都不拷。
copy_thread_registry_into() { # <副本目录>
  [[ "$REPLAY_BROKEN" == thread-registry-not-copied ]] && return 0
  mkdir -p "$1/.claude" && cp -- .claude/heavy-thread-default "$1/.claude/heavy-thread-default"
}

# 拷今天这棵树去编译的副本（e168_patched_tree 的副本、driver_e171_s4_comparator 的冻结树、driver_e175 的仓副本）：范围取 scripts/hosts/lib_repository_copy.py 的共用清单与闭包
# （path 依赖的 prune-server/、编译期读的 .claude/heavy-thread-default、运行期起的脚本都在里面），经 scripts/hosts/minimal-copy.py 拷，这里不写清单；
# 它打的两行进 stderr（驱动的 stdout 是产物）。在仓根调。副本里起命令的，多机配置照 replay_configuration_for_the_copy 交。
# 弄坏开关 REPLAY_BREAK=copy-list-fixed：照写死的那几样拷（共用清单里的四样加登记文件，没有 prune-server/）。
replay_copy_closure_into() { # <副本目录>
  if [[ "$REPLAY_BROKEN" == copy-list-fixed ]]; then
    rsync -a --exclude target Cargo.toml Cargo.lock .cargo crates "$1/" && copy_thread_registry_into "$1"
    return
  fi
  python3 scripts/hosts/minimal-copy.py . "$1" >&2
}

# 副本里起命令用的多机配置（scripts/hosts/lib_repository_copy.py configuration-path：调用方设了 SINGLEFS_MULTI_HOST_CONFIG 照它，没设取主工作区那一份）：
# 副本不在 git 里，判法照副本的位置找不到配置。打一行绝对路径。在仓根调。弄坏开关 REPLAY_BREAK=copy-configuration-unset：什么都不打（副本里不交）。
replay_configuration_for_the_copy() {
  [[ "$REPLAY_BROKEN" == copy-configuration-unset ]] && return 0
  python3 scripts/hosts/lib_repository_copy.py configuration-path .
}

# 取回归档的留存产物：research/results/ 底下的留存产物按「每次提交删上一次的实验记录」归档进了版本库，不在树里。git log --all --diff-filter=D 找最近一次删它的提交，
# git show <提交>^:research/results/<名> 读回删之前那一版。成功往标准输出打提交的前 12 位、内容写进 <取回到的文件>；找不到提交、读不出、读出来是空的返回 1。
# 弄坏开关 REPLAY_BREAK=archived-restore-off：一律取不回（改成不取的样子）。
replay_archived_product_copy() { # <research/results 底下的产物名> <取回到的文件>
  local name="$1" target="$2" commit
  [[ "$REPLAY_BROKEN" == archived-restore-off ]] && return 1
  commit="$(git log --all --diff-filter=D --format=%H -1 -- "research/results/$name" 2>/dev/null)" && [[ -n "$commit" ]] || return 1
  git show "$commit^:research/results/$name" >"$target" 2>/dev/null && [[ -s "$target" ]] || return 1
  printf '%s\n' "${commit:0:12}"
}
# 驱动开跑前核它要读的输入。research/results/ 底下缺的先从 git 历史取回、放回原处（记进 $OUT_DIR/restored-archived-products.list，replay_finish 收尾删）；
# 取不回、或不在 research/results/ 底下（冻结根、导出根）的缺了，每缺一样往 stderr 打一行「✗ 输入不在 test=<这一行> …」，返回 1，这一行记跑不了；都在返回 0。
# 驱动写成 replay_inputs_present <这一行> <缺了怎么办> <路径…> || return $?，放在编与跑之前。
# 弄坏开关 REPLAY_BREAK=inputs-not-checked：不核、照起（缺了在装置或比对器里跑不了，或静默少判）。
replay_inputs_present() { # <这一行> <缺了怎么办> <路径…>
  local row="$1" remedy="$2" path missing=0 commit temporary
  shift 2
  [[ "$REPLAY_BROKEN" == inputs-not-checked ]] && return 0
  for path in "$@"; do
    [[ -n "$path" && -e "$path" ]] && continue
    if [[ "$path" == research/results/* ]]; then
      temporary="$path.restoring.$BASHPID"
      if commit="$(replay_archived_product_copy "${path#research/results/}" "$temporary")" && mv -n -- "$temporary" "$path" && [[ ! -e "$temporary" ]]; then
        printf '%s\n' "$path" >>"${OUT_DIR:?}/restored-archived-products.list"
        printf '  · %s 要读的 %s 不在树里，从 git 历史取回（git show %s^:%s），本趟收尾删\n' "$row" "$path" "$commit" "$path" >&2
        continue
      fi
      rm -f -- "$temporary"
      [[ -e "$path" ]] && continue
    fi
    printf '  ✗ 输入不在 test=%s %s 不在（树里没有、git 历史里也取不回），这一行记跑不了；%s\n' "$row" "${path:-（没给路径）}" "$remedy" >&2
    missing=$((missing + 1))
  done
  (( missing == 0 ))
}
# 冻结根与导出根的固定位置：工作根（.claude/hooks/lib_work_root.py）下的 replay-frozen-roots/<名>。工作根在多机配置的共享根下，两台读同一个绝对路径；
# 根的重建办法在各驱动的出路里。取不到工作根返回 1。
replay_frozen_root() { # <名>
  local work_root
  work_root="$(python3 "$REPLAY_REPOSITORY_ROOT/.claude/hooks/lib_work_root.py" 2>/dev/null)" && [[ -n "$work_root" ]] || return 1
  printf '%s/replay-frozen-roots/%s\n' "$work_root" "$1"
}
# 取回那一句的出路（树里与 git 历史里都没有时给人看）
REPLAY_ARCHIVED_REMEDY='先从 git 历史取回：git log --all --diff-filter=D --name-only -- "*<文件名>" 找到删它的提交，git show <提交>^:<路径> 放回原处；历史里也没有就要重录这份产物'
# 装置 main 与单测按字面 concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/<名>") 读别的实验的留存产物（scripts/experiment-apparatus/lib.rs 的 ARCHIVED_PRODUCT_NOT_RUN_MARKER 那一段）：
# 有行要现跑时，把这些名字里不在树里的从 git 历史取回放回原处（replay_inputs_present，本趟收尾删）。取不回的这里不判：读它的装置自己退 77，那一行记跑不了。
replay_restore_referenced_products() {
  local name
  while IFS= read -r name; do
    [[ -n "$name" && ! -e "research/results/$name" ]] || continue
    replay_inputs_present "装置读的留存产物" "" "research/results/$name" 2>/dev/null || true
  done < <(grep -rhoE 'CARGO_MANIFEST_DIR"\), *"/\.\./\.\./research/results/[A-Za-z0-9_.-]+"' scripts/experiment-apparatus --include='*.rs' 2>/dev/null | sed -E 's#.*/research/results/##; s#"$##' | sort -u)
}

# 开跑之前编一次 scripts/experiment-apparatus/ 的装置（在 scripts/experiment-apparatus/ 里跑）：输出落进 $1（构建日志，以 .log 结尾；经 scripts/hosts/two-logs.sh 出两份：
# $1 是全量，同名的 .problems.log 只收 error、warning: 与它们的位置行、非 0 退出码，行前带全量日志里的行号）。失败时打第一处 error 那一行、日志末尾 20 行、
# 日志路径、错误警告日志路径与一条单独重现的命令，返回 2；成功什么都不打。弄坏开关 REPLAY_BREAK=build-log-single：只写一份全量日志。
build_research_binaries() { # <构建日志> [只编这几个 bin…]（不给就整编 research 工作区）
  local log="$1" build_exit bin_options=() name
  shift
  for name in "$@"; do bin_options+=(--bin "$name"); done
  if [[ "$REPLAY_BROKEN" == build-output-dropped ]]; then   # 弄坏开关：输出丢进 /dev/null、红了只说一句
    cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml >/dev/null 2>&1 || { echo "replay: 构建失败" >&2; return 2; }
    return 0
  fi
  if [[ "$REPLAY_BROKEN" == build-log-single ]]; then
    if cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml ${bin_options[@]+"${bin_options[@]}"} >"$log" 2>&1; then return 0; else build_exit=$?; fi
  elif bash scripts/hosts/two-logs.sh --quiet "${log%.log}" -- cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml ${bin_options[@]+"${bin_options[@]}"}; then
    return 0
  else
    build_exit=$?
  fi
  echo "replay: 构建失败（cargo build 退 $build_exit），一条实验都没跑" >&2
  grep -m1 -E '^error(\[|:)' -- "$log" | sed 's/^/  第一处错：/' >&2
  echo "  构建日志末尾 20 行：" >&2
  tail -n 20 -- "$log" | sed 's/^/    /' >&2
  echo "  整份构建日志：$log" >&2
  echo "  错误警告日志：${log%.log}.problems.log（只收 error、warning: 与位置行，行前带全量日志里的行号，先看它）" >&2
  echo "  → 怎么办：照日志里第一处 error 改；单独重现：(cd $PWD && cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml${bin_options[*]:+ ${bin_options[*]}})" >&2
  return 2
}

# 驱动函数开跑前编它要的那一个研究二进制。研究工作区是 scripts/experiment-apparatus/，仓根不是那个工作区（在仓根点那个包再带特性，cargo 拒），
# 照 build_research_binaries 经 --manifest-path 指过去；cargo 的输出一律进 stderr（驱动的 stderr 是这一行的 .err，stdout 是产物），
# 编不过另打一句重现命令、返回 1。行键照 --bin 那个名字解到 scripts/experiment-apparatus/ 里那份源码（admission.py 文件头「复跑表每一行的行键」）。
# 弄坏开关 REPLAY_BREAK=driver-build-output-dropped：不指工作区、输出丢进 /dev/null，.err 是空的。
build_research_driver_binary() { # <特性> <bin 名>
  local features="$1" binary="$2"
  if [[ "$REPLAY_BROKEN" == driver-build-output-dropped ]]; then
    cargo build -q --release --features "$features" --bin "$binary" >/dev/null 2>&1 || return 1
    return 0
  fi
  cargo build -q --release --manifest-path scripts/experiment-apparatus/Cargo.toml --features "$features" --bin "$binary" >&2 && return 0
  echo "replay: 驱动要的研究二进制 $binary 编不过（上面是 cargo 的输出）；单独重现：(cd $PWD && cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml --features $features --bin $binary)" >&2
  return 1
}

# 开了 GPU 时的卡（文件头「GPU」）：清场与分卡归统一任务服务，服务在最外层给整条复跑套一次清场、把清场之后的卡经环境 GPU_CARDS_BY_PRIORITY 交下来；
# 这里不另套清场，装置照环境里的 GPU_CARDS_BY_PRIORITY 取卡。环境里没有它（不是经服务起的，或服务没交卡）返回 1（打 ✗ 与 → 出路）：
# 装置没有它就拒，那样每一行都红在装置里，不如这里一次说清。弄坏开关 REPLAY_BREAK=service-cards-unchecked：不核（改错的样子：每一行到装置里才红）。
service_gpu_cards_present() {
  [[ "$REPLAY_BROKEN" == service-cards-unchecked ]] && return 0
  if [[ -z "${GPU_CARDS_BY_PRIORITY:-}" || -z "${GPU_CARDS_BY_PRIORITY//[[:space:]]/}" ]]; then
    echo "  ✗ replay: 多机配置开了 GPU（ENABLE_GPU=1），环境里却没有 GPU_CARDS_BY_PRIORITY（统一任务服务清场之后交的卡），一条实验都没跑" >&2
    echo "  → 怎么办：整条经 task-server/task submit -- bash scripts/experiment-apparatus/replay.sh … 交，清场与分卡由服务在最外层办，本脚本与命令里都不另套清场包装；确实不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
    return 1
  fi
  return 0
}

# 配置开了 GPU 时，起 验证包装置（cargo run … -p singlefs-verification --bin <装置>）带 --features gpu 编（文件头「GPU」）：
# 结果放进全局数组 CHECKER_TIER_FEATURES（空数组就是不带），不经命令替换带出来。多机配置怎么找、键怎么读只在
# scripts/crash/crash-amplification-lib.sh 一处（crash_amplification_multi_host_configuration_file、crash_amplification_configuration_value），这里 source 它不抄第二份。
# 返回 0：配置没开 GPU（文件不在、没写 ENABLE_GPU、写 0），或开了而 gpu 特性编得过（cargo build --release -p singlefs-verification --features gpu --lib，输出落进 $1）。
# 返回 78（打 ✗ 与 → 出路）：配置读不出（SINGLEFS_MULTI_HOST_CONFIG 点名的文件不在、文件不可读、读的时候出错、ENABLE_GPU 写了 0、1 之外的值），
# 或开了 GPU 而 gpu 特性编不过。不悄悄退回不带特性：那样装置会把判段当没编进 gpu 特性、报本次未跑或 panic，复跑结论不是配置要的那一份。
# 弄坏开关 REPLAY_BREAK=gpu-features-dropped：读配置、编也编，但开了 GPU 也不带特性。
# 开了 GPU、试编也过了之后接着核服务交的卡（service_gpu_cards_present）。
resolve_checker_tier_features() { # <gpu 编译日志>：先读开关、开着再试编与核卡（两步分开的写法见 read_gpu_switch、prepare_checker_tier_features）
  read_gpu_switch "$1" || return $?
  prepare_checker_tier_features "$1"
}

# 读多机配置的 ENABLE_GPU：结果放进全局 REPLAY_GPU_SWITCH（0 或 1）与 REPLAY_GPU_CONFIGURATION_FILE，不编。配置读不出返回 78（打 ✗ 与 → 出路）。
read_gpu_switch() { # <gpu 编译日志>（读配置出错时把那几行落在 <日志>.configuration）
  local log="$1" configuration_file switch="" reading_exit
  REPLAY_GPU_SWITCH=0
  REPLAY_GPU_CONFIGURATION_FILE=""
  CHECKER_TIER_FEATURES=()
  # 只拷了这一份脚本的树（自证造的临时仓）里没有那份库：那里也没有多机配置，ENABLE_GPU 算关，不读库
  if [[ ! -r scripts/crash/crash-amplification-lib.sh && -z "${SINGLEFS_MULTI_HOST_CONFIG:-}" && ! -e multi-host.env ]]; then
    return 0
  fi
  # shellcheck source=/dev/null
  if ! source scripts/crash/crash-amplification-lib.sh; then
    echo "  ✗ replay: 读不了 scripts/crash/crash-amplification-lib.sh（读多机配置的共用库），不知道配置开没开 GPU，一条实验都没跑" >&2
    echo "  → 怎么办：从仓根起（bash scripts/experiment-apparatus/replay.sh），或补回那份库" >&2
    return 78
  fi
  configuration_file="$(crash_amplification_multi_host_configuration_file "$PWD")"
  if [[ ! -e "$configuration_file" ]]; then
    if [[ -n "${SINGLEFS_MULTI_HOST_CONFIG:-}" ]]; then
      echo "  ✗ replay: SINGLEFS_MULTI_HOST_CONFIG 点名的多机配置 $configuration_file 不在，读不出 ENABLE_GPU，一条实验都没跑" >&2
      echo "  → 怎么办：把 SINGLEFS_MULTI_HOST_CONFIG 指到在的文件，或不设它（取主工作树根的 multi-host.env；那份不在就是两个开关都关）" >&2
      return 78
    fi
    return 0   # 没有多机配置：ENABLE_GPU 算关（crash-amplification-lib.sh 文件头「配置」）
  fi
  if [[ ! -r "$configuration_file" ]]; then
    echo "  ✗ replay: 多机配置 $configuration_file 不可读，读不出 ENABLE_GPU，一条实验都没跑" >&2
    echo "  → 怎么办：修好文件权限（ls -l $configuration_file），再跑" >&2
    return 78
  fi
  if switch="$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU 2>"$log.configuration")"; then reading_exit=0; else reading_exit=$?; fi
  if (( reading_exit != 0 )); then
    echo "  ✗ replay: 读多机配置 $configuration_file 的 ENABLE_GPU 出错（退 $reading_exit）：$(head -n 3 -- "$log.configuration" 2>/dev/null | tr "\n" " ")，一条实验都没跑" >&2
    echo "  → 怎么办：看那份配置是不是 UTF-8 文本；单跑 bash -c 'source scripts/crash/crash-amplification-lib.sh && crash_amplification_configuration_value \"\$1\" ENABLE_GPU' x $configuration_file" >&2
    return 78
  fi
  case "$switch" in
    ""|0) return 0 ;;
    1) REPLAY_GPU_SWITCH=1; REPLAY_GPU_CONFIGURATION_FILE="$configuration_file"; return 0 ;;
    *)
      echo "  ✗ replay: 多机配置 $configuration_file 的 ENABLE_GPU 写成了「$switch」，只认 0、1，读不出开没开 GPU，一条实验都没跑" >&2
      echo "  → 怎么办：改成 ENABLE_GPU=1 或 ENABLE_GPU=0（写法见仓根 multi-host.env.example）" >&2
      return 78 ;;
  esac
}

# 开了 GPU（read_gpu_switch 读出 1）时试编 gpu 特性、核服务交的卡，设 CHECKER_TIER_FEATURES；没开一个字都不变。编不过、环境里没有卡返回 78。
prepare_checker_tier_features() { # <gpu 编译日志>
  local log="$1" configuration_file="${REPLAY_GPU_CONFIGURATION_FILE:-}" build_exit
  CHECKER_TIER_FEATURES=()
  [[ "${REPLAY_GPU_SWITCH:-0}" == 1 ]] || return 0
  if [[ "$REPLAY_BROKEN" == build-log-single ]]; then   # 弄坏开关：只写一份全量日志
    if cargo build --release -p singlefs-verification --features gpu --lib >"$log" 2>&1; then build_exit=0; else build_exit=$?; fi
  elif bash scripts/hosts/two-logs.sh --quiet "${log%.log}" -- cargo build --release -p singlefs-verification --features gpu --lib; then
    build_exit=0
  else
    build_exit=$?
  fi
  if (( build_exit == 0 )); then
    service_gpu_cards_present || return 78
    [[ "$REPLAY_BROKEN" == gpu-features-dropped ]] || CHECKER_TIER_FEATURES=(--features gpu)
    return 0
  fi
  echo "  ✗ replay: 多机配置 $configuration_file 开了 GPU（ENABLE_GPU=1），验证包带 --features gpu 编不过（cargo build 退 $build_exit），一条实验都没跑；不退回不带 gpu 特性去跑" >&2
  grep -m1 -E '^error(\[|:)' -- "$log" | sed 's/^/  第一处错：/' >&2
  echo "  编译日志末尾 20 行：" >&2
  tail -n 20 -- "$log" | sed 's/^/    /' >&2
  echo "  整份编译日志：$log" >&2
  echo "  错误警告日志：${log%.log}.problems.log（只收 error、warning: 与位置行，行前带全量日志里的行号，先看它）" >&2
  echo "  → 怎么办：照日志里第一处 error 修好 gpu 特性的编译；单独重现：(cd $PWD && cargo build --release -p singlefs-verification --features gpu --lib)；确实不用 GPU 就把多机配置的 ENABLE_GPU 改成 0" >&2
  return 78
}

# 复跑表这一行问准入用哪个键：admission.py replay-row-keys 给的（按前三列 实验号|二进制|参数 查 ADMISSION_ROW_KEY），结果放进 ROW_ADMISSION_KEY
# （不经命令替换带出来）。弄坏开关 REPLAY_BREAK=row-key-by-experiment 下一律用实验号。
admission_key_of_row() {
  if [[ "$REPLAY_BROKEN" == row-key-by-experiment ]]; then
    ROW_ADMISSION_KEY="$1"
  else
    ROW_ADMISSION_KEY="${ADMISSION_ROW_KEY["$1|$2|$3"]-$1}"
  fi
}

# ── 计时行的负载关（约定见文件头「计时行的负载关」）──────────────────────────────────────
TIMING_LOAD_RATIO="${REPLAY_TIMING_LOAD_RATIO:-0.5}"
TIMING_LAG_FILE=".claude/gate.d/timing-load-field-lag.tsv"
declare -A TIMING_LAG=()   # 滞后表：留存产物名 → 理由

# 比对之前删掉 name=done 收尾行上的 loadavg1= 与 cpus=（管道过滤器）：每次复跑它们都不同，留存产物里也可能没有。别的行上同名的字段不动。
strip_load_fields() {
  sed -E '/^E7RESULT name=done /s/ (loadavg1|cpus)=[^ ]*//g'
}

# 读 $1（一份产物）里每一条 name=done 收尾行的 loadavg1= 与 cpus=，往 stdout 打一行：
#   missing <原因>                      没有收尾行，或有一条收尾行缺这两个字段、值不是数、cpus 不是正整数
#   ok|high <最大比值> <loadavg1> <cpus>  取各段里 loadavg1/cpus 最大的那一段；比 $TIMING_LOAD_RATIO 大是 high
timing_load_reading_raw() {
  awk -v limit="$TIMING_LOAD_RATIO" '
    /^E7RESULT name=done / {
      seen++; load = ""; cpus = ""
      for (i = 1; i <= NF; i++) { if ($i ~ /^loadavg1=/) load = substr($i, 10); else if ($i ~ /^cpus=/) cpus = substr($i, 6) }
      if (load !~ /^[0-9]+(\.[0-9]+)?$/ || cpus !~ /^[1-9][0-9]*$/) { malformed = 1; next }
      ratio = load / cpus
      if (!have || ratio > best) { have = 1; best = ratio; best_load = load; best_cpus = cpus }
    }
    END {
      if (!seen) print "missing 产物里没有 name=done 收尾行，收尾行没有 loadavg1 / cpus"
      else if (malformed) print "missing 收尾行没有 loadavg1 / cpus（或值不是数）"
      else { word = (best > limit + 0) ? "high" : "ok"; print word, best, best_load, best_cpus }
    }' "$1"
}

# 这一行（种类 $1，留存产物名 $2，本次新产物 $3）过不过负载关；结果放进全局，不经命令替换带出来：
#   TIMING_LOAD_DECISION  judge（照判）、skip（本次不判）、red（判红）；TIMING_LOAD_NOTE 是点名产物与比值的一句话；TIMING_LOAD_LAGGED 为 1 是「没字段、靠滞后表照判」
# 弄坏开关 REPLAY_BREAK=load-field-ignored：读字段，但 high 与「没字段又不在滞后表」都照判（不看字段的样子）。
timing_load_decide() {
  local kind="$1" stored="$2" fresh="$3" reading word ratio load cpus
  TIMING_LOAD_DECISION=judge; TIMING_LOAD_NOTE=""; TIMING_LOAD_LAGGED=0
  [[ "$kind" == timing ]] || return 0
  reading="$(timing_load_reading_raw "$fresh")"
  word="${reading%% *}"
  case "$word" in
    high|ok)
      read -r _ ratio load cpus <<<"$reading"
      TIMING_LOAD_NOTE="loadavg1=$load cpus=$cpus 比值 $ratio（阈值 $TIMING_LOAD_RATIO）"
      if [[ "$word" == high && "$REPLAY_BROKEN" != load-field-ignored ]]; then TIMING_LOAD_DECISION=skip; fi ;;
    *)
      if [[ -n "${TIMING_LAG["$stored"]+set}" ]]; then
        TIMING_LOAD_LAGGED=1; TIMING_LOAD_NOTE="${reading#* }；$stored 登记在滞后表 $TIMING_LAG_FILE，照旧判"
      else
        TIMING_LOAD_NOTE="产物 $stored（本次输出 $fresh）${reading#* }；$stored 也没登记在滞后表 $TIMING_LAG_FILE"
        if [[ "$REPLAY_BROKEN" != load-field-ignored ]]; then TIMING_LOAD_DECISION=red; fi
      fi ;;
  esac
  return 0
}

# 读滞后表 $1（一行 `<留存产物名><制表符><理由>`，# 开头的行与空行跳过）进 TIMING_LAG；$2 是复跑表原文，$3 是留存产物目录。
# 每个问题往 stdout 打一行，有问题返回 1：表不存在；没有制表符隔开的理由列；没写理由（至少 8 个字）；名字重复；名字不是 $2 里某条 timing 行的第 4 列；留存产物在树里且自己已经有字段（该删这一行）。
# 留存产物不在树里是对的（多数已归档进版本库，replay_judge_fresh_product 从 git 历史取回再比），所以这里只认复跑表，不认文件在不在。
load_timing_lag() {
  local file="$1" table_text="$2" results_directory="$3" line name reason line_number=0 problems=0 timing_products reading
  TIMING_LAG=()
  if [[ ! -f "$file" ]]; then echo "滞后表 $file 不存在"; return 1; fi
  timing_products="$(printf '%s\n' "$table_text" | awk -F'|' '$5 == "timing" {print $4}')"
  while IFS= read -r line || [[ -n "$line" ]]; do
    line_number=$((line_number + 1))
    [[ -z "${line//[[:space:]]/}" || "$line" =~ ^[[:space:]]*# ]] && continue
    if [[ "$line" != *$'\t'* ]]; then echo "$file:$line_number 没有用制表符隔开的理由（两列：留存产物名、理由）：$line"; problems=$((problems + 1)); continue; fi
    name="${line%%$'\t'*}"; reason="${line#*$'\t'}"
    name="${name//[[:space:]]/}"; reason="${reason#"${reason%%[![:space:]]*}"}"
    if [[ -z "$name" || ${#reason} -lt 8 ]]; then echo "$file:$line_number 名字空着或理由不足 8 个字：$line"; problems=$((problems + 1)); continue; fi
    if [[ -n "${TIMING_LAG["$name"]+set}" ]]; then echo "$file:$line_number 重复登记 $name"; problems=$((problems + 1)); continue; fi
    TIMING_LAG["$name"]="$reason"
    if ! grep -qxF -- "$name" <<<"$timing_products"; then
      echo "$file:$line_number $name 不是复跑表任何一条 timing 行的留存产物（第 4 列）"; problems=$((problems + 1)); continue
    fi
    if [[ -f "$results_directory/$name" ]]; then
      reading="$(timing_load_reading_raw "$results_directory/$name")"
      if [[ "${reading%% *}" != missing ]]; then echo "$file:$line_number $name 自己的收尾行已经有 loadavg1 / cpus，滞后表只缩不涨，删这一行"; problems=$((problems + 1)); fi
    fi
  done <"$file"
  ((problems == 0))
}

# 同时跑几条实验的缺省：登记的重活默认并行度（.claude/heavy-thread-default，经 scripts/hosts/heavy-threads.sh；SINGLEFS_THREAD_CAP 设了取它）。
# 弄坏开关 REPLAY_BREAK=jobs-from-nproc 下取核数的一半。
default_replay_jobs() {
  if [[ "$REPLAY_BROKEN" == jobs-from-nproc ]]; then
    echo $(( $(nproc 2>/dev/null || echo 4) / 2 ))
  else
    heavy_thread_default
  fi
}

# 复跑表里的实验号（文件头「入口脚本」）：从本文件原文读 TABLE 那一段（在仓根调，脚本开头已 cd 到仓根），每个实验号一行
# 「<实验号><TAB><几行><TAB><二进制或驱动，逗号连>」，按表里头一次出现的次序。--list-items 打它，「实验号写错退 2」拿实验号那一列比。
replay_table_experiments() {
  awk -F'|' '/^TABLE=\$\(cat <<.TSV.$/ { inside = 1; next }
             inside && /^TSV$/ { exit }
             inside && /^E[0-9A-Za-z]*\|/ { if (!($1 in rows)) order[++count] = $1; rows[$1]++; binaries[$1] = binaries[$1] (binaries[$1] == "" ? "" : ",") $2 }
             END { for (index_in_order = 1; index_in_order <= count; index_in_order++) { name = order[index_in_order]; printf "%s\t%d\t%s\n", name, rows[name], binaries[name] } }' \
    scripts/experiment-apparatus/replay.sh
}

# 自证（bash scripts/experiment-apparatus/replay.sh --selftest）：PATH 最前面放一个假 cargo，只演 build_research_binaries 的两种结局，不编、不跑任何实验。
#   build-red    假 cargo 打一行 error[E0425] 退 101：要返回 2，stderr 有第一处错、日志末尾、日志路径与重现命令，日志落了盘
#   build-green  假 cargo 退 0：要返回 0，stderr 一个字都没有
#   gpu-features-on / -off / -garbled / -no-such / -unreadable / -build-red  配置开了 GPU 的特性（resolve_checker_tier_features，假 cargo 记参数，配置文件放临时目录）：
#     写 ENABLE_GPU=1 → 试编一次、CHECKER_TIER_FEATURES 是 --features gpu；写 0 或没写 → 不带、不起 cargo；写 yes、点名的文件不在、文件不可读 → 78 点名原因；
#     开了而试编红 → 78、第一处错、日志路径、「不退回不带 gpu 特性去跑」
#   run-lines-with-features  起 验证包装置的 cargo run 行（本文件里 grep 出来）每一行都带数组
#   gpu-cards-from-service / gpu-service-cards-missing  开了 GPU 的卡由统一任务服务交（service_gpu_cards_present；多机配置里的清场命令是假的、只往临时目录的日志记一笔）：
#     gpu-cards-from-service  环境里有 GPU_CARDS_BY_PRIORITY → 返回 0，照装置那样直接起的命令看到的就是那一份，假清场命令一笔都没记（不另套清场）
#     gpu-service-cards-missing  环境里没有 GPU_CARDS_BY_PRIORITY → 78、点名这个变量，假清场命令一笔都没记
#   run-lines-without-quiesce-wrapper  起 验证包装置的 cargo run 行没有一行另套清场，本文件不是注释的行里也不起清场包装
#   row-key      一张两行的假键表（E1 第二行指明 E1/second）：admission_key_of_row 给第二行的键要是 E1/second，不是实验号
#   jobs-default 没设 SINGLEFS_THREAD_CAP 时 default_replay_jobs 要等于登记文件第一条非注释行的数；设成 7 时要是 7
#   整条按行认键（replay-row-keys 判红、按行问准入、产物头由这里写）在临时仓里跑的那几格在 python3 scripts/gate/admission.py --selftest 的「复跑表按行的准入键」
#   负载关（假产物放临时目录，阈值钉 0.5，timing_load_decide 判，不起任何实验）：
#     load-high    收尾行 loadavg1=20.50 cpus=32（比值 0.640625）→ skip，点名比值
#     load-low     loadavg1=2.50 cpus=32 → judge
#     load-none    收尾行没有字段、不在滞后表 → red，点名产物与「收尾行没有 loadavg1 / cpus」
#     load-lagged  收尾行没有字段、产物登记在滞后表 → judge 并记滞后
#     load-exact   判据写 exact 的行不看字段 → judge
#     lag-table-bad  滞后表登记复跑表里没有的名字、exact 行的产物、重复名字、理由不足 8 个字、用空格而不是制表符隔开理由 → 逐条点名（load_timing_lag）
#     lag-table-stale  滞后表登记的留存产物自己已经有字段 → 点名叫人删这一行
#     strip-load   strip_load_fields 只删 name=done 行上的 loadavg1= 与 cpus=
#   按行复用（replay_reuse_selftest：临时仓里两行的合成复跑表，E1 留存产物在、E2 的留存产物只在 git 历史里；这一份 replay.sh 的副本、假 cargo（记下每一次收到的参数）、假 rustc、
#   两个假 research 二进制，真仓的 .claude/scripts、.claude/singlefs-ai-sop、scripts/gate 链进来，判定库在临时 prune-server 里、库名取临时仓自己的 git common-dir
#   （键带临时目录的命名空间；库里有几条经 admission.py library-files fetch 拉进这一格的临时目录数），接连跑五趟：
#     reuse-first-run      头一趟两行都跑：E1 字节一致、E2 字节一致（留存产物从 git 历史取回），判定库记两份（各带组成清单），临时仓里不建 singlefs-unit-verdicts/
#     reuse-second-run     第二趟两行都复用、一个装置都没跑、cargo build 一次都不起（全复用就不编），REPLAY_TOTAL 报 reused=2 judged=0
#     reuse-device-source  改 E1 装置 main 的一行代码：只重跑 E1、只编 --bin e1-sample，开跑前报出变了 fn e1_sample.rs::main，E2 照旧复用
#     reuse-comment-only   只改 E2 装置源码的一行注释：两行都复用（行键取单元的去注释指纹，注释不算）
#     reuse-three-questions  复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：上面头一趟、第二趟（①）、只改注释（②）、改 E1 装置 main（③，affected E1）
#                          照共用库判、打规范输出「REUSE_SELFTEST script=scripts/experiment-apparatus/replay.sh …」；reuse-second-run、reuse-comment-only、reuse-device-source 三格在它里面点名
#     last-run-list        上次红的清单（文件头「三」那一份共用库）：全绿的几趟之后是空文件
#     reuse-sentinel       复用哨兵（共用库文件头「二」）：种子 5、哨兵 1 条那一趟绿、抽中一行强制现判；开着 REPLAY_BREAK=reuse-ignored 判红「复用失灵」
#                          （这两趟与它们前面清空开关垫的一趟，REPLAY_BREAK 自己定，不接调用方设的开关：别的开关打开时这一格不跟着红）
#     reuse-red-withdraws-green  记红、撤绿、下趟现判：清空开关垫一趟（E1 字节一致入库）；再跑一趟，复用哨兵（哨兵 1 条、种子 5：E2 当哨兵、E1 被抽成强制现判）让 E1 同键现判，E1 的假装置（REPLAY_SELFTEST_DRIFT_DEVICE=e1，装置在仓外不进键）演出另一份产物
#                          ⇒ E1 对不上、整趟退 1，库里这把行键记成 drift；下一趟：只起 e1、开跑前写「E1 要现跑：上次判红」、判回字节一致，E2 复用；再下一趟两行全复用
#     reuse-broken-withdraws-green  跑不了也记红、撤绿：清空开关垫一趟（E1 字节一致入库）；再跑一趟，复用哨兵（哨兵 1 条、种子 5）让 E1 同键现判，E1 的假装置退 3（REPLAY_SELFTEST_BROKEN_DEVICE=e1，装置在仓外不进键）
#                          ⇒ E1 跑不了、整趟退 1，库里这把行键记成 broken；下一趟：只起 e1、开跑前写「E1 要现跑：上次判红」、判回字节一致，E2 复用；再下一趟两行全复用
#     reuse-outside-git-exits-2  另搭一棵样本仓、删掉它的 .git，跑一趟：整趟退 2、点名不是 git 工作树、一个装置都没起、不打 REPLAY_TOTAL
#     reuse-unreachable-exits-75  服务连不上（socket 不在、TCP 端口刚放掉、重连总时长 2 秒）：整趟退 75、点名 prune-server 不可用、一个装置都没起、不打 REPLAY_TOTAL
#     reuse-gpu-only-for-feature-rows  另搭一棵样本仓、复跑表加一行点到 CHECKER_TIER_FEATURES 的驱动 E3，多机配置 ENABLE_GPU=1、环境里没有 GPU_CARDS_BY_PRIORITY：
#                          只跑 E1 E2 退 0、两行都起、不试编 gpu 特性、stderr 写「都不点 CHECKER_TIER_FEATURES」；只跑 E3 退 78、点名 GPU_CARDS_BY_PRIORITY、一个装置都没起
#   这几趟的哨兵数设 1000（键相同的全当哨兵、不抽强制现判），reuse-sentinel 那两趟除外；上次红的清单写临时目录（SINGLEFS_LAST_RUNS_DIRECTORY）
#   留存产物取回与装置读不到输入（replay_archived_selftest：同一份临时仓，E2 的留存产物只在 git 历史里）三格：
#     archived-retrieved-green    E2 字节一致、结果行写出处「取自 git show <提交>^:research/results/e2-sample.out」、整趟退 0
#     archived-unrecoverable-red  E2 的留存产物名改成历史里也没有的：判「取不回」、结果行点名这个产物、整趟退 1
#     device-not-run-red          E1 的假装置打一行 NOT_RUN_SAMPLE_INPUT 退 77：记跑不了（退出码 77）、整趟退 1，没有「本次未跑」的一档
#   留存产物只以 .out.gz 存在时的比对（replay_gzip_selftest：同一份临时仓，E1 的留存产物改成 e1-sample.out.gz）：
#     gz-stored-green      先解压再比：E1 字节一致、整趟退 0
#     gz-stored-red        换成内容不同的 gzip：E1 对不上、整趟退 1
#   驱动开跑前核输入（replay_inputs_present）：driver_e142、E170 两行判镜像、E171 第一段读冻结参照的七行、E171 第 1 次重跑三个比对器、
#     E174 三个 judge、E158 r3 / r4 比对、E165 七个变异杀伤界驱动，从本文件取出定义，在没有 research/、冻结根与导出根都没给的空目录里各起一次：
#     都要退 1、stderr 里点名自己的「✗ 输入不在 test=<驱动名>」行数恰好是缺的样数（E158 r4 六十六份臂产物逐份报），stdout 不打
#   编译目录（文件头「编译目录」；replay_build_directory_selftest：同一份临时仓，假 cargo 编的时候把假装置拷进 CARGO_TARGET_DIR 的 release/）五格：
#     跑绿、编不过、装置开跑之后被 TERM 三种结局之后这一格的 TMPDIR 底下都不剩这一趟的编译目录，调用方给了 CARGO_TARGET_DIR 照它编且不删，
#     装置在本轮输出目录里造的带 CACHEDIR.TAG 的副本编译目录跑完不在、本轮输出照留
# 弄坏开关：
#   REPLAY_BREAK=build-output-dropped  构建输出丢进 /dev/null、红了只说「replay: 构建失败」
#       build-red（「✗ 自检：构建失败时应当返回 2、打第一处错」）
#       连带红的格：「✗ 自检：按行复用 reuse-device-source：改了 E1 装置 main 的一行代码」、「✗ 自检：构建失败时」、「✗ 自检：构建成功时错误警告日志也要在、末行写收了 0 行（build-green.problems.log）；实际：cat: … No s…」
#   REPLAY_BREAK=build-log-single  构建日志（build.log、gpu-build.log）只写一份全量的（不经 two-logs.sh，没有 .problems.log）
#       build-two-logs（「✗ 自检：构建失败时应当出两份日志…」）、构建成功那一格的错误警告日志（「✗ 自检：构建成功时错误警告日志也要在、末行写收了 0 行」）
#   REPLAY_BREAK=gz-read-raw  留存产物 .gz 不解压、原样读（read_stored 的 *.gz 分支退成 cat）
#       gz-stored-green（「✗ 自检：留存产物 .gz gz-stored-green：…」）
#   REPLAY_BREAK=driver-build-output-dropped  驱动开跑前编研究二进制不指 scripts/experiment-apparatus 工作区、输出丢进 /dev/null
#       driver-build（「✗ 自检：驱动开跑前编研究二进制（build_research_driver_binary）要经 --manifest-path …」）
#       与 E162、E163 那三个驱动各一格（「✗ 自检：driver_e162_anchors 开跑前要经 build_research_driver_binary 编…」，driver_e162_power_cut_selftest、driver_e163_r1_merge 同）
#       连带红的格：「✗ 自检：driver_e162_power_cut_selftest 开跑前要经 build_research_driver_binary…」、「✗ 自检：driver_e163_r1_merge 开跑前要经 build_research_driver_binary 编（cargo 只…」
#   REPLAY_BREAK=row-key-by-experiment  每一行一律按实验号问准入
#       row-key（「✗ 自检：复跑表那一行指明了准入键 E1/second」）；admission.py --selftest 设着它跑，『复跑表按行的准入键』那几格转红
#   REPLAY_BREAK=load-field-ignored  不看负载字段：超阈值的与没字段又不在滞后表的，都照判
#       load-high（「✗ 自检：负载 20.5 / 32 核超过阈值 0.5，这一行应当记本次不判并点名比值」）、load-none（「✗ 自检：收尾行没有字段又不在滞后表，这一行应当判红…」）两格转红
#   REPLAY_BREAK=gpu-features-dropped  配置开了 GPU 也不带 gpu 特性
#       gpu-features-on（「✗ 自检：配置写了 ENABLE_GPU=1，应当先试编 gpu 特性…」）
#   REPLAY_BREAK=run-lines-without-features  自检里核「cargo run 行都带特性数组」时当没有数组（模拟新加的一行忘了带）
#       run-lines-with-features（「✗ 自检：起 验证包装置的 cargo run 行…应当每一行都带…」）
#   REPLAY_BREAK=service-cards-unchecked  开了 GPU 不核环境里有没有服务交的 GPU_CARDS_BY_PRIORITY（改错的样子）
#       gpu-service-cards-missing（「✗ 自检：开了 GPU 而环境里没有服务交的 GPU_CARDS_BY_PRIORITY」）
#   REPLAY_BREAK=run-line-quiesce-wrapped  自检里核「不另套清场」时多塞一行经清场包装起的 cargo run（模拟新加的一行又套了清场）
#       run-lines-without-quiesce-wrapper（「✗ 自检：起 验证包装置不另套清场」）
#   REPLAY_BREAK=reuse-ignored  按行复用的判定库只记不读（每一趟全跑）
#       reuse-second-run、reuse-device-source、reuse-comment-only、reuse-three-questions（『✗ 自检：按行复用 <格名>：』）
#       连带红的格：「✗ 自检：按行复用 reuse-second-run：第二趟行键没变」、「✗ 自检：按行复用 reuse-device-source：改了 E1 装置 main 的一行代码」、「✗ 自检：按行复用 reuse-comment-only：只改 E2 装置源码的一行注释」、「✗ 自检：按行复用 reuse-three-questions：reuse-three-questions：① 同一份输入跑两趟：第二趟现判…」
#   REUSE_CHECKS_BREAK=rejudged-sentinel-passes（共用库 scripts/gate/lib_reuse_checks.py 的开关）  哨兵被现判不红
#       reuse-sentinel（「✗ 自检：按行复用 reuse-sentinel：」）
#   REPLAY_BREAK=red-not-recorded  对不上的行不记红（只记绿，判红的不撤同一把行键原有的绿）
#       reuse-red-withdraws-green（「✗ 自检：按行复用 reuse-red-withdraws-green：」）
#   REPLAY_BREAK=broken-not-recorded  现跑了而跑不了的行不记红（不撤同一把行键原有的绿，下一趟照复用）
#       reuse-broken-withdraws-green（「✗ 自检：按行复用 reuse-broken-withdraws-green：」）
#   REPLAY_BREAK=row-library-failure-tolerated  按行复用的库没法用（不在 git 工作树里、行键整份算不出、中转目录建不了、记录拉不下来）时只打一句、往下全跑
#       reuse-outside-git-exits-2（「✗ 自检：按行复用 reuse-outside-git-exits-2：」）
#   REPLAY_BREAK=prune-server-failure-swallowed  判定库服务连不上（replay-row-inputs、library-files 退 75）不按 75 办（照别的失败退出或照记不下往下跑），不退 75
#       reuse-unreachable-exits-75（「✗ 自检：按行复用 reuse-unreachable-exits-75：」）
#   REPLAY_BREAK=reuse-key-by-row-only  按行复用的键只按复跑表那一行算，不看装置源码与工具链
#       reuse-first-run（「✗ 自检：按行复用 reuse-first-run：」：键不经准入模块算，判定库那两份不带组成清单）、
#       reuse-device-source（「✗ 自检：按行复用 reuse-device-source：」）
#       连带红的格：「✗ 自检：按行复用 reuse-three-questions：reuse-three-questions：③ 改闭包里一个函数单元：闭包含…」
#   REPLAY_BREAK=gpu-for-every-run  开了 GPU 时要现跑的行一律当要卡（不看 admission.py replay-row-inputs 第 4 列，有行要现跑就试编 gpu 特性、要服务交的卡）
#       reuse-gpu-only-for-feature-rows（「✗ 自检：按行复用 reuse-gpu-only-for-feature-rows：」）
#   REPLAY_BREAK=build-everything  有行要现跑时一律整编（不按要现跑的行只编那几个 bin）
#       reuse-device-source（「✗ 自检：按行复用 reuse-device-source：」）
#   REPLAY_BREAK=archived-restore-off  留存产物不在树里也不从 git 历史取回（replay_archived_product_copy 一律取不回）
#       archived-retrieved-green（「✗ 自检：留存产物取回 archived-retrieved-green：」）
#       连带红的格（E2 的留存产物只在 git 历史里，取不回就整趟退 1）：
#       「✗ 自检：按行复用 reuse-first-run：」、「✗ 自检：按行复用 reuse-second-run：」、「✗ 自检：按行复用 reuse-device-source：」、「✗ 自检：按行复用 reuse-comment-only：」、
#       「✗ 自检：按行复用 reuse-three-questions：」、「✗ 自检：按行复用 last-run-list：」、「✗ 自检：留存产物取回 device-not-run-red：」、「✗ 自检：入口与完整性闸 integrity-previous-output：」
#   REPLAY_BREAK=unrecoverable-as-green  取不回的留存产物当字节一致（改错的样子）
#       archived-unrecoverable-red（「✗ 自检：留存产物取回 archived-unrecoverable-red：」）
#   REPLAY_BREAK=inputs-not-checked  驱动开跑前不核输入、照起
#       驱动开跑前核输入那二十五格（「✗ 自检：driver_e142 在没有 research/、冻结根与导出根都没给的目录里」）
#       连带红的格：「✗ 自检：driver_e170_r2_v2_487_z3_images 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e170_r2_v2_487_z4_images 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s2_images_o3 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s2_o3_control_baseline 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s2_comparator 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s3_comparator_mutations 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s4_comparator 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s4_f4_comparator 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e171_s4_f4_substitutes_comparator 在没有 research/、冻结根与导出根都没给…」、「✗ 自检：driver_e171_r1_s1_revision_6_comparator 在没有 research/、冻结根与导出根都没给的…」、「✗ 自检：driver_e171_r1_s1_revision_6_cross_host_check 在没有 research/、冻结根与导…」、「✗ 自检：driver_e171_r1_s2_revision_6_comparator 在没有 research/、冻结根与导出根都没给的…」、「✗ 自检：driver_e174_simulated_judge 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e174_segment_one_judge 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e174_segment_two_judge 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e158_r3_seg1_compare 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e158_r4_compare 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e165_segment2 在没有 research/、冻结根与导出根都没给的目录里」、「✗ 自检：driver_e165_r3_segment3_root_choice_kill_bounds 在没有 research/、冻结根…」、「✗ 自检：driver_e165_r4_segment2_root_choice_faults_kill_bounds 在没有 resear…」、「✗ 自检：driver_e165_r5_segment2_r2_mode_kill_bounds 在没有 research/、冻结根与导出根…」、「✗ 自检：driver_e165_r5_segment2_r3_mode_kill_bounds 在没有 research/、冻结根与导出根…」、「✗ 自检：driver_e165_r5_segment2_r4_mode_kill_bounds 在没有 research/、冻结根与导出根…」、「✗ 自检：driver_e165_r5_segment5b_r4_gate_r2_mode_kill_bounds 在没有 research…」
#   REPLAY_BREAK=thread-registry-not-copied  拷 crates/ 去编译的副本不带 .claude/heavy-thread-default
#       thread-registry-copied（「✗ 自检：拷 crates/ 去编译的副本要带上每台默认重活线程数的登记文件…」）
#   REPLAY_BREAK=copy-lines-without-registry  自检里核「拷 crates 的行都接着拷登记文件」时当没有那一句（模拟新加的一行忘了带）
#       copy-lines-with-registry（「✗ 自检：拷 crates/ 去编译的行…应当每一行都接着 copy_thread_registry_into…」）
#   REPLAY_BREAK=copy-list-fixed  拷今天这棵树去编译的副本（e168_patched_tree、driver_e171_s4_comparator、driver_e175）照写死的四样加登记文件拷，不取共用库的闭包（没有 prune-server/）
#       copies-take-the-shared-closure（「✗ 自检：拷今天这棵树去编译的副本要照共用库的闭包拷全…」）
#   REPLAY_BREAK=copy-configuration-unset  副本里起命令不交多机配置（副本不在 git 里，照副本的位置找不到配置）
#       copy-configuration-from-the-main-tree（「✗ 自检：副本里起命令的多机配置要取主工作区那一份…」）
#   REPLAY_BREAK=build-directory-kept  收尾不删这一趟自己建的编译目录与运行目录（编进固定的 scripts/experiment-apparatus/target/，跨趟留着）
#       build-directory-removed-when-green、build-directory-removed-when-build-fails、build-directory-removed-when-stopped、
#       copy-build-directories-removed（这一格也核 TMPDIR 底下不剩）（『✗ 自检：编译目录 <格名>：』）
#       连带红的格：「✗ 自检：编译目录 build-directory-removed-when-green：跑绿」、「✗ 自检：编译目录 build-directory-removed-when-build-fails：编不过」、「✗ 自检：编译目录 build-directory-removed-when-stopped：装置开跑之后给副本发 TERM」、「✗ 自检：编译目录 copy-build-directories-removed：装置在本轮输出目录里造的副本编译目录（带 CACHEDIR…」
#   REPLAY_BREAK=term-not-forwarded  被停时 replay_one 不把 TERM 转给手里那一步（装置照跑、开着编译目录里的文件，编译目录删不掉）
#       build-directory-removed-when-stopped（「✗ 自检：编译目录 build-directory-removed-when-stopped：」）
#   REPLAY_BREAK=copy-build-directories-kept  收尾不删驱动在本轮输出目录里的副本编出来的编译目录
#       copy-build-directories-removed（「✗ 自检：编译目录 copy-build-directories-removed：」）
#   REPLAY_BREAK=guard-off  平常跑设了弄坏开关也不拒（认不得的选项先被「复跑表里没有」当用法错拒，不点名变量）
#       break-switch-refused-outside-selftest（「✗ 自检：平常跑设了弄坏开关应当退 2 并点名」）
# 弄坏开关 REPLAY_BREAK 只在 --selftest 起的那几趟里认（自证设 REPLAY_SELFTEST_MARK 指到它建的标记文件，子进程继承）：平常跑设了就在一切之前退 2、点名它只在自证里认。
# 按行复用的自证（文件头「按行复用」，replay_selftest 调）：<目录> 下建一个临时仓（git init），复跑表换成两行（E1 留存产物在、E2 的留存产物只在 git 历史里），
# 真仓的 .claude/scripts、.claude/singlefs-ai-sop、scripts/gate 链进来，PATH 前面放假 cargo、假 rustc；
# 拿这一份 replay.sh 的副本接连跑五趟，每一格往 stdout 打一行「<格名> ok」或「<格名> bad <一句>」。
# 按行复用与留存产物取回两份自证共用的临时仓（replay_reuse_selftest、replay_archived_selftest 调）：<目录>/root 下 git init，复跑表换成两行（E1 留存产物在、E2 的留存产物先入库再删掉、只在 git 历史里），
# 真仓的 .claude/scripts、.claude/singlefs-ai-sop、scripts/gate 链进来，PATH 前面放假 cargo、假 rustc（<目录>/bin）；
# 两个假 research 二进制往 $REPLAY_SELFTEST_RUN_LOG 记一笔，REPLAY_SELFTEST_NOT_RUN_DEVICE 点名它时打一行 NOT_RUN_SAMPLE_INPUT 退 77；
# REPLAY_SELFTEST_BROKEN_DEVICE 点名它时往 stderr 打一句、退 3（跑不了）。
# 设了 REPLAY_SELFTEST_DEVICE_HOLD 的，先建旁边那个 .fifo、再把自己的进程号写进它点名的文件，之后在 fifo 上 read -t 8 拿住 8 秒（开着编译目录里自己那份脚本）：
# 写下进程号之后不再起子进程：TERM 只停装置自己，它 fork 出来、还没 exec 的子进程照样开着同一份脚本，副本收尾核开着的文件时就不删编译目录（所以不一趟趟起 sleep 0.1）。
replay_selftest_sample_repository() { # <目录>
  local work="$1" real_root="$PWD" root name
  root="$work/root"
  mkdir -p "$root/.claude/gate.d" "$root/scripts/experiment-apparatus" "$work/devices" "$work/tmp" "$root/scripts/hosts" "$root/research/results" "$work/bin" || return 1
  git init -q "$root" || return 1
  ln -s "$real_root/.claude/scripts" "$root/.claude/scripts"
  ln -s "$real_root/.claude/singlefs-ai-sop" "$root/.claude/singlefs-ai-sop"
  ln -s "$real_root/scripts/gate" "$root/scripts/gate"
  ln -s "$real_root/scripts/crash" "$root/scripts/crash"   # resolve_checker_tier_features 读多机配置的共用库
  ln -s "$real_root/scripts/hosts/two-logs.sh" "$root/scripts/hosts/two-logs.sh"   # 构建日志出两份（build_research_binaries）
  ln -s "$real_root/scripts/hosts/two_logs.py" "$root/scripts/hosts/two_logs.py"
  ln -s "$real_root/scripts/hosts/run-build-directory.sh" "$root/scripts/hosts/run-build-directory.sh"   # 这一趟的编译目录（文件头「编译目录」）
  ln -s "$real_root/scripts/hosts/run_build_directory.py" "$root/scripts/hosts/run_build_directory.py"
  printf '%s\n' '.claude/scripts' '.claude/singlefs-ai-sop' 'scripts/gate' 'scripts/crash' 'scripts/hosts/two-logs.sh' 'scripts/hosts/two_logs.py' \
    'scripts/hosts/run-build-directory.sh' 'scripts/hosts/run_build_directory.py' >"$root/.gitignore"
  printf '%s\n' '# 自证的登记表：一行都不登记准入' >"$root/.claude/gate.d/stage-inputs.tsv"
  printf '%s\n' '# 自证的滞后表' >"$root/.claude/gate.d/timing-load-field-lag.tsv"
  printf '%s\n' '[package]' 'name = "e7-index-bench"' 'version = "0.1.0"' '' '[lib]' 'path = "lib.rs"' '' '[[bin]]' 'name = "e1-sample"' 'path = "e1_sample.rs"' \
    '' '[[bin]]' 'name = "e2-sample"' 'path = "e2_sample.rs"' >"$root/scripts/experiment-apparatus/Cargo.toml"
  printf '%s\n' '# 自证' >"$root/scripts/experiment-apparatus/Cargo.lock"
  printf '%s\n' 'pub fn shared() -> u32 { 1 }' >"$root/scripts/experiment-apparatus/lib.rs"
  printf '%s\n' 'fn main() { println!("one"); }' >"$root/scripts/experiment-apparatus/e1_sample.rs"
  printf '%s\n' 'fn main() { println!("two"); }' >"$root/scripts/experiment-apparatus/e2_sample.rs"
  for name in e1 e2; do
    printf '%s\n' '#!/usr/bin/env bash' "echo $name >>\"\${REPLAY_SELFTEST_RUN_LOG:?}\"" \
      "if [[ \"\${REPLAY_SELFTEST_NOT_RUN_DEVICE:-}\" == $name ]]; then echo 'NOT_RUN_SAMPLE_INPUT test=$name-sample reason=自证造的输入不在' >&2; exit 77; fi" \
      "if [[ \"\${REPLAY_SELFTEST_BROKEN_DEVICE:-}\" == $name ]]; then echo '自证造的跑不了' >&2; exit 3; fi" \
      '[[ -z "${REPLAY_SELFTEST_DEVICE_PATHS:-}" ]] || echo "$0" >>"$REPLAY_SELFTEST_DEVICE_PATHS"' \
      'if [[ -n "${REPLAY_SELFTEST_DEVICE_HOLD:-}" ]]; then [[ -p "$REPLAY_SELFTEST_DEVICE_HOLD.fifo" ]] || mkfifo -- "$REPLAY_SELFTEST_DEVICE_HOLD.fifo" || exit 1; echo "$$" >"$REPLAY_SELFTEST_DEVICE_HOLD"; read -r -t 8 _ <>"$REPLAY_SELFTEST_DEVICE_HOLD.fifo"; fi' \
      'if [[ -n "${REPLAY_SELFTEST_COPY_TARGET:-}" ]]; then copy="$(dirname "$REPLAY_DEV")/selftest-copy"; mkdir -p "$copy/target/release" "$copy/source" && printf "Signature: 8a477f597d28d172789f06886806bc55\n" >"$copy/target/CACHEDIR.TAG"; fi' \
      "if [[ \"\${REPLAY_SELFTEST_DRIFT_DEVICE:-}\" == $name ]]; then printf 'E7RESULT name=config arm=$name-drifted\\nE7RESULT name=done emitted=2\\n'; exit 0; fi" \
      "printf 'E7RESULT name=config arm=$name\\nE7RESULT name=done emitted=2\\n'" \
      >"$work/devices/$name-sample"
    chmod +x "$work/devices/$name-sample"
  done
  printf 'E7RESULT name=config arm=e1\nE7RESULT name=done emitted=2\n' >"$root/research/results/e1-sample.out"
  # E2 的留存产物只在 git 历史里：先入库、再删掉（按「每次提交删上一次的实验记录」归档的样子），复跑从 git 历史取回再比
  printf 'E7RESULT name=config arm=e2\nE7RESULT name=done emitted=2\n' >"$root/research/results/e2-sample.out"
  git -C "$root" add -f research/results/e2-sample.out \
    && git -C "$root" -c user.name=selftest -c user.email=selftest@example.invalid -c commit.gpgsign=false commit -q -m 'e2 sample product' \
    && git -C "$root" rm -q research/results/e2-sample.out \
    && git -C "$root" -c user.name=selftest -c user.email=selftest@example.invalid -c commit.gpgsign=false commit -q -m 'archive e2 sample product' || return 1
  # 假 cargo：记下收到的参数（与 CARGO_TARGET_DIR，另一份 .target）；编（build）的时候把 $work/devices/ 下的假装置拷进 CARGO_TARGET_DIR 的 release/（装置从那里起）；
  # REPLAY_SELFTEST_BUILD_FAILS=1 时编不过、退 101
  printf '%s\n' '#!/usr/bin/env bash' '[[ "${1:-}" == -V ]] && { echo "cargo 0.0.0-replay-reuse-selftest"; exit 0; }' 'echo "$*" >>"${REPLAY_SELFTEST_CARGO_LOG:?}"' \
    'echo "${CARGO_TARGET_DIR-}" >>"$REPLAY_SELFTEST_CARGO_LOG.target"' \
    '[[ "${REPLAY_SELFTEST_BUILD_FAILS:-}" == 1 ]] && { echo "error: 自证造的编译失败"; exit 101; }' \
    "if [[ \"\${1:-}\" == build ]]; then mkdir -p \"\${CARGO_TARGET_DIR:?}/release\" && cp -- \"$work/devices/\"* \"\$CARGO_TARGET_DIR/release/\" || exit 101; fi" 'exit 0' >"$work/bin/cargo"
  printf '%s\n' '#!/usr/bin/env bash' 'case "${1:-}" in -V) echo "rustc 0.0.0-replay-reuse-selftest";; -vV) printf "rustc 0.0.0-replay-reuse-selftest\nhost: x86_64-unknown-linux-gnu\n";; esac' 'exit 0' >"$work/bin/rustc"
  chmod +x "$work/bin/cargo" "$work/bin/rustc"
  python3 - "$real_root/scripts/experiment-apparatus/replay.sh" "$root/scripts/experiment-apparatus/replay.sh" <<'PY' || return 1
import sys
text = open(sys.argv[1], encoding="utf-8").read()
table_start = text.index("TABLE=$(cat <<'TSV'\n") + len("TABLE=$(cat <<'TSV'\n")
table_end = text.index("\nTSV\n)\n", table_start)
text = text[:table_start] + "E1|e1-sample||e1-sample.out|exact\nE2|e2-sample||e2-sample.out|exact" + text[table_end:]
keys_start = text.index("ADMISSION_ROW_KEYS=$(cat <<'TSV'\n") + len("ADMISSION_ROW_KEYS=$(cat <<'TSV'\n")
keys_end = text.index("TSV\n)\n", keys_start)
text = text[:keys_start] + text[keys_end:]
with open(sys.argv[2], "w", encoding="utf-8") as handle:
    handle.write(text)
PY
  chmod +x "$root/scripts/experiment-apparatus/replay.sh"
}
# 自证里临时仓的按行复用库名：与入口「库名」那一段同一种取法（<git common-dir>/singlefs-unit-verdicts/replay-row，文件头「按行复用」），
# 只当名字交给 admission.py library-files，本地不建；取不到 git common-dir 退 1、什么都不打
replay_selftest_row_library_name() { # <临时仓>
  local common_directory
  common_directory="$(git -C "$1" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$common_directory" ]] || return 1
  printf '%s/singlefs-unit-verdicts/replay-row\n' "$common_directory"
}
replay_reuse_selftest() {
  local work="$1" real_root="$PWD" root library calls cargo_calls out total run_number=0 reuse_exit=0 name
  root="$work/root"; calls="$work/devices-ran.log"; cargo_calls="$work/cargo-calls.log"
  replay_selftest_sample_repository "$work" || return 1
  library="$(replay_selftest_row_library_name "$root")" || { echo "reuse-first-run bad 临时仓 $root 取不到 git common-dir，库名拼不出"; return 1; }
  reuse_run() { # [参数…]：在临时仓里跑一趟副本；输出进 $out、这一趟起了哪几个装置进 $calls（每趟先清空）、末行 REPLAY_TOTAL 进 $total、退出码进 reuse_exit
    run_number=$((run_number + 1)); out="$work/out.$run_number"; : >"$calls"; : >"$cargo_calls"; : >"$work/not-run.$run_number"
    # 复用哨兵：键相同的全当哨兵、不抽强制现判（reuse_sentinel 设了的那几趟照它）；上次红的清单写临时目录
    if (cd "$root" && env -u SINGLEFS_MULTI_HOST_CONFIG -u SINGLEFS_REUSE_SENTINEL_SEED -u CARGO_TARGET_DIR TMPDIR="$work/tmp" PATH="$work/bin:$PATH" REPLAY_OUT="$work/replay-out.$run_number" REPLAY_JOBS=1 \
          REPLAY_SELFTEST_RUN_LOG="$calls" REPLAY_SELFTEST_CARGO_LOG="$cargo_calls" GATE_NOT_RUN_FILE="$work/not-run.$run_number" \
          SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" SINGLEFS_REUSE_SENTINEL_COUNT=1000 ${reuse_sentinel[@]+"${reuse_sentinel[@]}"} bash scripts/experiment-apparatus/replay.sh "$@") >"$out" 2>&1; then
      reuse_exit=0
    else
      reuse_exit=$?
    fi
    total="$(grep '^REPLAY_TOTAL ' "$out")"
    for name in E1 E2; do   # 复用三问那一格读的：这一趟每一行复用还是现判
      if grep -qE "^$name .*复用 " "$out"; then printf '%s\t复用\n' "$name"; else printf '%s\t现判\n' "$name"; fi
    done >"$work/states.$run_number"
  }
  local reuse_sentinel=()
  ran() { sort "$calls" | tr '\n' ' '; }
  reuse_run
  if [[ "$reuse_exit" == 0 && "$(ran)" == "e1 e2 " && "$total" == *" reused=0 "* ]] && grep -qE '^E1 .*字节一致' "$out" && grep -qE '^E2 .*字节一致.*取自 git show' "$out" \
     && [[ "$(replay_selftest_library_counts "$library" "$work/fetched")" == "2 2" && ! -e "${library%/*}" ]] \
     && [[ ! -s "$work/not-run.$run_number" ]]; then
    echo "reuse-first-run ok"
  else
    echo "reuse-first-run bad 头一趟应当两行都跑、E1 字节一致、E2 字节一致（留存产物从 git 历史取回）、判定库记两份（各带组成清单）、临时仓里不建 singlefs-unit-verdicts/、没有逐字节比对本次未跑的报告；退 $reuse_exit，起了 $(ran)，$total，判定库「判定 组成清单」$(replay_selftest_library_counts "$library" "$work/fetched") 份，临时仓里 singlefs-unit-verdicts/ $([[ -e "${library%/*}" ]] && echo 建了 || echo 没建)，本次未跑「$(tr '\n' ' ' <"$work/not-run.$run_number")」，输出尾部 $(tail -n 4 "$out" | tr '\n' ' ')"
  fi
  reuse_run
  if [[ "$reuse_exit" == 0 && -z "$(ran)" && "$total" == *" judged=0 reused=2 "* ]] && grep -qE '^E1 .*复用 +字节一致：行键相同' "$out" && grep -qE '^E2 .*复用 +字节一致：行键相同' "$out" \
     && ! grep -q '^build' "$cargo_calls" && grep -qF '要现跑的 0 行，不试编 gpu 特性、不编' "$out"; then
    echo "reuse-second-run ok"
  else
    echo "reuse-second-run bad 第二趟行键没变应当两行都复用、一个装置都不起、cargo build 一次都不起（不编）；退 $reuse_exit，起了「$(ran)」，cargo 收到「$(tr '\n' ';' <"$cargo_calls")」，$total"
  fi
  printf '%s\n' 'fn main() { println!("uno"); }' >"$root/scripts/experiment-apparatus/e1_sample.rs"
  reuse_run
  if [[ "$reuse_exit" == 0 && "$(ran)" == "e1 " && "$total" == *" reused=1 "* ]] && grep -qE '^E1 .*字节一致' "$out" && grep -qE '^E2 .*复用' "$out" \
     && grep -q -- '^build .*--bin e1-sample' "$cargo_calls" && ! grep -q -- '--bin e2-sample' "$cargo_calls" && grep -q 'E1 要现跑：键对不上.*变了 fn e1_sample.rs::main' "$out"; then
    echo "reuse-device-source ok"
  else
    echo "reuse-device-source bad 改了 E1 装置 main 的一行代码应当只重跑 E1（只编 --bin e1-sample、报出变了 fn e1_sample.rs::main）、E2 复用；退 $reuse_exit，起了「$(ran)」，cargo 收到「$(tr '\n' ';' <"$cargo_calls")」，$total，输出 $(grep '要现跑' "$out" | head -n 2 | tr '\n' ' ')"
  fi
  printf '%s\n' '// 只改一行注释：行键取单元的去注释指纹，注释不算' 'fn main() { println!("two"); }' >"$root/scripts/experiment-apparatus/e2_sample.rs"
  reuse_run
  if [[ "$reuse_exit" == 0 && -z "$(ran)" && "$total" == *" reused=2 "* ]]; then
    echo "reuse-comment-only ok"
  else
    echo "reuse-comment-only bad 只改 E2 装置源码的一行注释应当两行都复用；退 $reuse_exit，起了「$(ran)」，$total"
  fi
  # 复用三问（scripts/gate/lib_reuse_checks.py 文件头「一」）：上面四趟就是那四问——头一趟、第二趟（①）、只改注释（②，排在闭包内那一趟之后，库里是那一趟记下的）、
  # 改 E1 装置 main 的一行（③，affected 是 E1）；已有的 reuse-second-run、reuse-comment-only、reuse-device-source 三格在这一格里点名，判法照共用库
  if python3 "$real_root/scripts/gate/lib_reuse_checks.py" three-questions --script scripts/experiment-apparatus/replay.sh --first "$work/states.1" --second "$work/states.2" \
       --unrelated "$work/states.4" --unit-change "$work/states.3" --affected E1 >"$work/three.log" 2>&1; then
    echo "reuse-three-questions ok"
  else
    echo "reuse-three-questions bad $(tr '\n' ' ' <"$work/three.log")"
  fi
  grep '^REUSE_SELFTEST ' "$work/three.log"
  # 上次红的清单（lib_reuse_checks.py 文件头「三」）：这几趟都绿，清单是空文件
  if [[ -f "$work/last-runs/scripts__experiment-apparatus__replay.sh.tsv" && ! -s "$work/last-runs/scripts__experiment-apparatus__replay.sh.tsv" ]]; then
    echo "last-run-list ok"
  else
    echo "last-run-list bad 全绿的几趟之后 $work/last-runs/scripts__experiment-apparatus__replay.sh.tsv 应当在、是空文件：$(ls -l "$work/last-runs" 2>&1 | tr '\n' ' ')"
  fi
  # 复用哨兵（lib_reuse_checks.py 文件头「二」）：种子 5、哨兵 1 条：一行当哨兵复用、另一行强制现判、判定与库里相同，整趟绿；
  # 开着 REPLAY_BREAK=reuse-ignored（只记不读）再跑，哨兵判红「复用失灵」（共用库的开关 REUSE_CHECKS_BREAK=rejudged-sentinel-passes 下这一格判错）。
  # 这两趟的 REPLAY_BREAK 自己定、不接调用方设的开关：照常那一趟清空，只记不读那一趟只设 reuse-ignored（REPLAY_BROKEN 按整串比，接成「<别的>,reuse-ignored」就认不出），
  # 别的开关打开时这一格不跟着红（弄坏开关只红在文件头点名的格）。前面几趟是在调用方的开关下记的库（reuse-key-by-row-only 下键的算法都不同），
  # 所以先清空开关垫一趟（哨兵数 1000、不抽强制现判），库里是照常算的键，再起那两趟
  reuse_sentinel=(REPLAY_BREAK=)
  reuse_run
  local sentinel_primed_exit="$reuse_exit"
  reuse_sentinel=(SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5 REPLAY_BREAK=)
  reuse_run
  local sentinel_green_exit="$reuse_exit" sentinel_green_out="$out"
  reuse_sentinel=(SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5 REPLAY_BREAK=reuse-ignored)
  reuse_run
  reuse_sentinel=()
  if [[ "$sentinel_primed_exit" == 0 && "$sentinel_green_exit" == 0 ]] && grep -qF '复用哨兵（复跑表）：种子 5' "$sentinel_green_out" && grep -qF '复用哨兵抽中强制现判' "$sentinel_green_out" \
     && [[ "$reuse_exit" == 1 ]] && grep -qF '复用失灵：' "$out"; then
    echo "reuse-sentinel ok"
  else
    echo "reuse-sentinel bad 垫的那一趟与种子 5 那一趟应当绿、抽中一行强制现判，只记不读那一趟应当判红「复用失灵」；垫的那一趟退 $sentinel_primed_exit；照常那一趟退 $sentinel_green_exit：$(grep -E '复用哨兵|复用失灵' "$sentinel_green_out" | tr '\n' ' ')；只记不读那一趟退 $reuse_exit：$(grep -E '复用哨兵|复用失灵' "$out" | tr '\n' ' ')"
  fi
  # 记红、撤绿、下趟现判（文件头「按行复用」，reuse-red-withdraws-green）：清空开关垫一趟（E1 字节一致入库）；再跑一趟走复用哨兵强制现判（哨兵 1 条、种子 5：E2 当哨兵复用、E1 被抽成强制现判），E1 的假装置演出另一份产物
  # （REPLAY_SELFTEST_DRIFT_DEVICE=e1：装置在仓外，不进这一行的行键）⇒ E1 对不上、整趟退 1、库里这把行键记成 drift；下一趟：E1 现判（开跑前写「上次判红」）、判回字节一致，
  # E2 复用；再下一趟两行全复用（盖回绿）。弄坏开关 red-not-recorded（对不上的不记）让下一趟 E1 照旧复用；这一格只接这一个开关，别的开关打开时不跟着红
  local red_break=""
  [[ "$REPLAY_BROKEN" == red-not-recorded ]] && red_break=red-not-recorded
  reuse_sentinel=(REPLAY_BREAK=)
  reuse_run
  local red_primed_exit="$reuse_exit"
  reuse_sentinel=(REPLAY_BREAK="$red_break" REPLAY_SELFTEST_DRIFT_DEVICE=e1 SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5)
  reuse_run
  local red_drift_exit="$reuse_exit" red_drift_out="$out"
  reuse_sentinel=(REPLAY_BREAK=)
  reuse_run
  local red_rejudge_exit="$reuse_exit" red_rejudge_out="$out" red_rejudge_ran
  red_rejudge_ran="$(ran)"
  reuse_run
  if [[ "$red_primed_exit" == 0 && "$red_drift_exit" == 1 ]] && grep -qE '^E1 .*对不上' "$red_drift_out" \
     && [[ "$red_rejudge_exit" == 0 && "$red_rejudge_ran" == "e1 " ]] && grep -qF 'E1 要现跑：上次判红' "$red_rejudge_out" \
     && grep -qE '^E1 .*字节一致' "$red_rejudge_out" && grep -qE '^E2 .*复用' "$red_rejudge_out" \
     && [[ "$reuse_exit" == 0 && -z "$(ran)" && "$total" == *" judged=0 reused=2 "* ]]; then
    echo "reuse-red-withdraws-green ok"
  else
    echo "reuse-red-withdraws-green bad 同一把行键 E1 先字节一致入库、哨兵强制现判判成对不上，下一趟应当现判 E1（只起 e1、开跑前写上次判红、判回字节一致）、E2 复用，再下一趟两行全复用；垫的那一趟退 $red_primed_exit；强制现判那一趟退 $red_drift_exit；下一趟退 $red_rejudge_exit、起了「$red_rejudge_ran」、$(grep -F 'E1 要现跑' "$red_rejudge_out" | head -n 1 | cut -c1-200 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)；再下一趟退 $reuse_exit、起了「$(ran)」、$total"
  fi
  # 跑不了也记红、撤绿（文件头「按行复用」，reuse-broken-withdraws-green）：清空开关垫一趟（E1 字节一致入库）；再跑一趟走复用哨兵强制现判（哨兵 1 条、种子 5：E1 被抽成强制现判），E1 的假装置退 3
  # （REPLAY_SELFTEST_BROKEN_DEVICE=e1：装置在仓外，不进这一行的行键）⇒ E1 跑不了、整趟退 1、库里这把行键记成 broken；下一趟：E1 现判（开跑前写「上次判红」）、判回字节一致，
  # E2 复用；再下一趟两行全复用。弄坏开关 broken-not-recorded（跑不了的不记）让下一趟 E1 照旧复用；这一格只接这一个开关，别的开关打开时不跟着红
  local broken_break=""
  [[ "$REPLAY_BROKEN" == broken-not-recorded ]] && broken_break=broken-not-recorded
  reuse_sentinel=(REPLAY_BREAK=)
  reuse_run
  local broken_primed_exit="$reuse_exit"
  reuse_sentinel=(REPLAY_BREAK="$broken_break" REPLAY_SELFTEST_BROKEN_DEVICE=e1 SINGLEFS_REUSE_SENTINEL_COUNT=1 SINGLEFS_REUSE_SENTINEL_SEED=5)
  reuse_run
  local broken_run_exit="$reuse_exit" broken_run_out="$out"
  reuse_sentinel=(REPLAY_BREAK=)
  reuse_run
  local broken_rejudge_exit="$reuse_exit" broken_rejudge_out="$out" broken_rejudge_ran
  broken_rejudge_ran="$(ran)"
  reuse_run
  if [[ "$broken_primed_exit" == 0 && "$broken_run_exit" == 1 ]] && grep -qE '^E1 .*跑不了 +退出码 3' "$broken_run_out" \
     && [[ "$broken_rejudge_exit" == 0 && "$broken_rejudge_ran" == "e1 " ]] && grep -qF 'E1 要现跑：上次判红' "$broken_rejudge_out" \
     && grep -qE '^E1 .*字节一致' "$broken_rejudge_out" && grep -qE '^E2 .*复用' "$broken_rejudge_out" \
     && [[ "$reuse_exit" == 0 && -z "$(ran)" && "$total" == *" judged=0 reused=2 "* ]]; then
    echo "reuse-broken-withdraws-green ok"
  else
    echo "reuse-broken-withdraws-green bad 同一把行键 E1 先字节一致入库、哨兵强制现判跑成跑不了（退 3），下一趟应当现判 E1（只起 e1、开跑前写上次判红、判回字节一致）、E2 复用，再下一趟两行全复用；垫的那一趟退 $broken_primed_exit；强制现判那一趟退 $broken_run_exit；下一趟退 $broken_rejudge_exit、起了「$broken_rejudge_ran」、$(grep -F 'E1 要现跑' "$broken_rejudge_out" | head -n 1 | cut -c1-200 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)；再下一趟退 $reuse_exit、起了「$(ran)」、$total"
  fi
  # 不在 git 工作树里（文件头「按行复用」，reuse-outside-git-exits-2）：另搭一棵样本仓、删掉它的 .git（样本仓在临时目录里，往上找不到别的仓），跑一趟：
  # 整趟应当退 2、点名不是 git 工作树、一个装置都不起、不打 REPLAY_TOTAL。弄坏开关 row-library-failure-tolerated（不复用、不记、全跑）让它往下跑；这一格只接这一个开关
  local outside_break="" first_root="$root"
  [[ "$REPLAY_BROKEN" == row-library-failure-tolerated ]] && outside_break=row-library-failure-tolerated
  replay_selftest_sample_repository "$work/outside-git-cell" || return 1
  root="$work/outside-git-cell/root"
  rm -rf -- "${root:?}/.git"
  reuse_sentinel=(REPLAY_BREAK="$outside_break")
  reuse_run
  reuse_sentinel=()
  root="$first_root"
  if [[ "$reuse_exit" == 2 && -z "$(ran)" && -z "$total" ]] && grep -qF '不是 git 工作树' "$out"; then
    echo "reuse-outside-git-exits-2 ok"
  else
    echo "reuse-outside-git-exits-2 bad 不在 git 工作树里应当整趟退 2、点名不是 git 工作树、一个装置都不起、不打 REPLAY_TOTAL；退 $reuse_exit，起了「$(ran)」，$total，$(grep -E '✗|!' "$out" | head -n 2 | tr '\n' ' ' | cut -c1-300 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)"  # gate-lint:detail
  fi
  # 服务连不上（文件头「按行复用」，reuse-unreachable-exits-75）：socket 不在、TCP 端口刚放掉、重连总时长 2 秒；库里有判过的记录（前面几趟记的），
  # 整趟应当退 75、点名 prune-server 不可用、一个装置都不起、不打 REPLAY_TOTAL。弄坏开关 prune-server-failure-swallowed（照库里没有全跑）让它退 0；这一格只接这一个开关
  local unreachable_address unreachable_break=""
  [[ "$REPLAY_BROKEN" == prune-server-failure-swallowed ]] && unreachable_break=prune-server-failure-swallowed
  unreachable_address="$(python3 -B -c 'import socket; s = socket.socket(); s.bind(("127.0.0.1", 0)); print("127.0.0.1:%d" % s.getsockname()[1]); s.close()')"
  reuse_sentinel=(REPLAY_BREAK="$unreachable_break" SINGLEFS_PRUNE_SERVER_SOCKET="$work/no-such.sock" SINGLEFS_PRUNE_SERVER_TCP_ADDRESS="$unreachable_address"
    SINGLEFS_PRUNE_SERVER_RECONNECT_SECONDS=2)
  reuse_run
  reuse_sentinel=()
  if [[ "$reuse_exit" == 75 && -z "$(ran)" && -z "$total" ]] && grep -qF 'prune-server 不可用' "$out"; then
    echo "reuse-unreachable-exits-75 ok"
  else
    echo "reuse-unreachable-exits-75 bad 判定库服务连不上应当整趟退 75、点名 prune-server 不可用、一个装置都不起、不打 REPLAY_TOTAL；退 $reuse_exit，起了「$(ran)」，$total，$(grep -F 'prune-server' "$out" | head -n 2 | tr '\n' ' ' | cut -c1-300 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)"
  fi
  # 开了 GPU 只在要现跑的行里有要卡的行时才试编、要服务交的卡（文件头「GPU」，reuse-gpu-only-for-feature-rows）：另搭一棵样本仓，复跑表加一行
  # E3|@replay_selftest_gpu_driver（驱动闭包点到 CHECKER_TIER_FEATURES，admission.py replay-row-inputs 第 4 列给 1），多机配置写 ENABLE_GPU=1、环境里 GPU_CARDS_BY_PRIORITY 是空的：
  # 只跑 E1 E2（都不点）整趟退 0、两行都起、cargo 一次 --features gpu 都没收到、stderr 写「都不点 CHECKER_TIER_FEATURES」；只跑 E3 退 78、点名 GPU_CARDS_BY_PRIORITY、一个装置都不起。
  # 弄坏开关 gpu-for-every-run（要现跑的行一律当要卡）让只跑 E1 E2 那一趟退 78；这一格只接这一个开关
  local gpu_break="" gpu_plain_exit gpu_plain_out gpu_plain_ran gpu_plain_cargo
  [[ "$REPLAY_BROKEN" == gpu-for-every-run ]] && gpu_break=gpu-for-every-run
  replay_selftest_sample_repository "$work/gpu-cell" || return 1
  root="$work/gpu-cell/root"
  python3 - "$root/scripts/experiment-apparatus/replay.sh" <<'PY' || return 1
import sys
path = sys.argv[1]
text = open(path, encoding="utf-8").read()
table = "TABLE=$(cat <<'TSV'\n"
driver = ("replay_selftest_gpu_driver() {\n"
          "  echo \"features=${CHECKER_TIER_FEATURES[*]-}\" >&2\n"
          "  printf 'E7RESULT name=config arm=e3\\nE7RESULT name=done emitted=2\\n'\n"
          "}\n")
start = text.index(table)
row = "E2|e2-sample||e2-sample.out|exact"
row_end = text.index(row, start) + len(row)
text = text[:start] + driver + text[start:row_end] + "\nE3|@replay_selftest_gpu_driver||e3-sample.out|exact" + text[row_end:]
open(path, "w", encoding="utf-8").write(text)
PY
  printf '%s\n' '# 自证的多机配置' 'ENABLE_GPU=1' >"$work/gpu-cell/multi-host.env"
  reuse_sentinel=(REPLAY_BREAK="$gpu_break" SINGLEFS_MULTI_HOST_CONFIG="$work/gpu-cell/multi-host.env" GPU_CARDS_BY_PRIORITY=)
  reuse_run E1 E2
  gpu_plain_exit="$reuse_exit"; gpu_plain_out="$out"; gpu_plain_ran="$(ran)"; gpu_plain_cargo="$(tr '\n' ';' <"$cargo_calls")"
  reuse_run E3
  reuse_sentinel=()
  root="$first_root"
  if [[ "$gpu_plain_exit" == 0 && "$gpu_plain_ran" == "e1 e2 " && "$gpu_plain_cargo" != *"--features gpu"* ]] && grep -qF '都不点 CHECKER_TIER_FEATURES' "$gpu_plain_out" \
     && [[ "$reuse_exit" == 78 && -z "$(ran)" ]] && grep -qF '环境里却没有 GPU_CARDS_BY_PRIORITY' "$out"; then
    echo "reuse-gpu-only-for-feature-rows ok"
  else
    echo "reuse-gpu-only-for-feature-rows bad 开了 GPU、环境里没有卡：只跑不点 CHECKER_TIER_FEATURES 的 E1 E2 应当退 0、两行都起、不试编 gpu 特性，只跑点到的 E3 应当退 78、点名 GPU_CARDS_BY_PRIORITY；E1 E2 那一趟退 $gpu_plain_exit、起了「$gpu_plain_ran」、cargo 收到「$gpu_plain_cargo」、$(grep -E '✗|都不点' "$gpu_plain_out" | head -n 2 | tr '\n' ' ' | cut -c1-300 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)；E3 那一趟退 $reuse_exit、起了「$(ran)」、$(grep -F '✗' "$out" | head -n 2 | tr '\n' ' ' | cut -c1-300 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)"
  fi
}

# 自证读判定库（replay_reuse_selftest、replay_archived_selftest 调）：<库名> 底下的记录经 admission.py library-files 拉进 <落到哪>（先清空），
# 打「<判定份数> <组成清单份数>」；拉不成打「拉不成」。在真仓的工作目录里起（自证就在那里）
replay_selftest_library_counts() { # <库名> <落到哪>
  rm -rf -- "${2:?}"
  if python3 scripts/gate/admission.py library-files fetch "$1" "$2" keys/ >/dev/null 2>&1; then
    printf '%s %s\n' "$(find "$2/keys" -mindepth 2 -maxdepth 2 -name verdict 2>/dev/null | wc -l)" "$(find "$2/keys" -mindepth 2 -maxdepth 2 -name composition 2>/dev/null | wc -l)"
  else
    echo 拉不成
  fi
}

# 留存产物取回与装置读不到输入的自证（文件头「驱动要读的输入」，replay_selftest 调）：同一份临时仓（replay_selftest_sample_repository：E1 的留存产物在树里，
# E2 的留存产物只在 git 历史里——先入库、再删掉），接连跑三趟（每趟之前把两个假装置的源码各改一个字，行键各不相同，互不复用），每一格往 stdout 打一行「<格名> ok」或「<格名> bad <一句>」：
#   archived-retrieved-green    E2 的留存产物从 git 历史取回再比：E2 字节一致、结果行写出处「取自 git show <提交>^:research/results/e2-sample.out」、整趟退 0
#   device-not-run-red          E1 的假装置打一行 NOT_RUN_SAMPLE_INPUT 退 77：记跑不了（退出码 77）、整趟退 1，没有「本次未跑」的一档；E2 照旧字节一致
#   archived-unrecoverable-red  E2 的留存产物名改成历史里也没有的 e2-never-recorded.out：E2 判「取不回」、结果行点名这个产物、整趟退 1
replay_archived_selftest() {
  local work="$1" root calls out total exit_code=0
  root="$work/root"; calls="$work/devices-ran.log"
  replay_selftest_sample_repository "$work" || return 1
  archived_round() { # <趟号> [环境变量赋值…]：在临时仓里跑一趟副本；输出进 $out、起了哪几个装置进 $calls、末行 REPLAY_TOTAL 进 $total、退出码进 exit_code
    out="$work/out.$1"; : >"$calls"; : >"$work/not-run.$1"
    printf 'fn main() { println!("one-archived-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e1_sample.rs"
    printf 'fn main() { println!("two-archived-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e2_sample.rs"
    if (cd "$root" && env -u SINGLEFS_MULTI_HOST_CONFIG -u SINGLEFS_REUSE_SENTINEL_SEED -u CARGO_TARGET_DIR TMPDIR="$work/tmp" PATH="$work/bin:$PATH" REPLAY_OUT="$work/replay-out.$1" REPLAY_JOBS=1 \
          REPLAY_SELFTEST_RUN_LOG="$calls" REPLAY_SELFTEST_CARGO_LOG="$work/cargo-calls.log" GATE_NOT_RUN_FILE="$work/not-run.$1" \
          SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" SINGLEFS_REUSE_SENTINEL_COUNT=1000 "${@:2}" bash scripts/experiment-apparatus/replay.sh) >"$out" 2>&1; then
      exit_code=0
    else
      exit_code=$?
    fi
    total="$(grep '^REPLAY_TOTAL ' "$out")"
  }
  archived_round 1
  if [[ "$exit_code" == 0 ]] && grep -qE '^E1 +e1-sample +字节一致' "$out" \
     && grep -qE '^E2 +e2-sample +字节一致 +e2-sample\.out（取自 git show [0-9a-f]{12}\^:research/results/e2-sample\.out）' "$out"; then
    echo "archived-retrieved-green ok"
  else
    echo "archived-retrieved-green bad E2 的留存产物只在 git 历史里，应当取回再比、判字节一致并写出处、整趟退 0；退 $exit_code，$(grep -E '^E[12] ' "$out" | tr '\n' ';' | cut -c1-400)"
  fi
  archived_round 2 REPLAY_SELFTEST_NOT_RUN_DEVICE=e1
  if [[ "$exit_code" == 1 ]] && grep -qE '^E1 +e1-sample +跑不了 +退出码 77' "$out" && grep -qE '^E2 +e2-sample +字节一致' "$out" && ! grep -q '本次未跑 ' "$out"; then
    echo "device-not-run-red ok"
  else
    echo "device-not-run-red bad E1 的装置打了 NOT_RUN 行退 77，应当记跑不了（退出码 77）、整趟退 1、不再有本次未跑的一档；退 $exit_code，$(grep -E '^E[12] ' "$out" | tr '\n' ';' | cut -c1-400)"
  fi
  python3 - "$root/scripts/experiment-apparatus/replay.sh" <<'PY' || return 1
import sys
path = sys.argv[1]
lines = open(path, encoding="utf-8").read().split("\n")
rows = [index for index, line in enumerate(lines) if line == "E2|e2-sample||e2-sample.out|exact"]   # 整行相等：只有复跑表那一行，代码里的字面量不算
if len(rows) != 1:
    sys.exit("复跑表里找不到 E2 那一行（整行相等的有 %d 行）" % len(rows))
lines[rows[0]] = "E2|e2-sample||e2-never-recorded.out|exact"
with open(path, "w", encoding="utf-8") as handle:
    handle.write("\n".join(lines))
PY
  archived_round 3
  if [[ "$exit_code" == 1 ]] && grep -qE '^E2 +e2-sample +取不回 +留存产物 e2-never-recorded\.out' "$out"; then
    echo "archived-unrecoverable-red ok"
  else
    echo "archived-unrecoverable-red bad E2 的留存产物树里没有、git 历史里也取不回，应当判取不回、点名 e2-never-recorded.out、整趟退 1；退 $exit_code，$(grep -E '^E[12] ' "$out" | tr '\n' ';' | cut -c1-400)"
  fi
}

# 留存产物只以 .out.gz 存在时的比对（replay_one 里的 read_stored：*.gz 先 gzip -dc，E186 两份压成 .gz 入库）的自证，replay_selftest 调：同一份临时仓
# （replay_selftest_sample_repository），副本的复跑表里 E1 的留存产物改成 e1-sample.out.gz（临时仓里没有 .out，只有 .out.gz）；两格各一行收回来：
#   gz-stored-green  留存产物是装置输出的 gzip：整趟退 0，E1 判字节一致、产物名写 e1-sample.out.gz
#   gz-stored-red    留存产物换成内容不同的 gzip：整趟退 1，E1 判对不上、diff 命令点名 e1-sample.out.gz
replay_gzip_selftest() {
  local work="$1" root out exit_code=0
  root="$work/root"
  replay_selftest_sample_repository "$work" || return 1
  python3 - "$root/scripts/experiment-apparatus/replay.sh" <<'PY' || return 1
import sys
path = sys.argv[1]
lines = open(path, encoding="utf-8").read().split("\n")
rows = [index for index, line in enumerate(lines) if line == "E1|e1-sample||e1-sample.out|exact"]   # 整行相等：只有复跑表那一行，代码里的字面量不算
if len(rows) != 1:
    sys.exit("复跑表里找不到 E1 那一行（整行相等的有 %d 行）" % len(rows))
lines[rows[0]] = "E1|e1-sample||e1-sample.out.gz|exact"
with open(path, "w", encoding="utf-8") as handle:
    handle.write("\n".join(lines))
PY
  rm -f -- "$root/research/results/e1-sample.out"
  printf 'E7RESULT name=config arm=e1\nE7RESULT name=done emitted=2\n' | gzip -c >"$root/research/results/e1-sample.out.gz"
  gz_round() { # <趟号> [参数…]：在临时仓里只跑 E1；输出进 $out、退出码进 exit_code。先把 E1 的装置源码改一个字（行键变，不读上一趟记的判定）
    out="$work/out.$1"; : >"$work/devices-ran.log"; : >"$work/not-run.$1"
    printf 'fn main() { println!("one-gz-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e1_sample.rs"
    if (cd "$root" && env -u SINGLEFS_MULTI_HOST_CONFIG -u SINGLEFS_REUSE_SENTINEL_SEED -u CARGO_TARGET_DIR TMPDIR="$work/tmp" PATH="$work/bin:$PATH" REPLAY_OUT="$work/replay-out.$1" REPLAY_JOBS=1 \
          REPLAY_SELFTEST_RUN_LOG="$work/devices-ran.log" REPLAY_SELFTEST_CARGO_LOG="$work/cargo-calls.log" GATE_NOT_RUN_FILE="$work/not-run.$1" \
          SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" SINGLEFS_REUSE_SENTINEL_COUNT=1000 bash scripts/experiment-apparatus/replay.sh "${@:2}" E1) >"$out" 2>&1; then
      exit_code=0
    else
      exit_code=$?
    fi
  }
  gz_round 1
  if [[ "$exit_code" == 0 ]] && grep -qE '^E1 +e1-sample +字节一致 +e1-sample\.out\.gz' "$out"; then
    echo "gz-stored-green ok"
  else
    echo "gz-stored-green bad 留存产物只以 e1-sample.out.gz 存在（内容是装置输出的 gzip）时应当先解压再比、E1 判字节一致、整趟退 0；退 $exit_code，$(grep -E '^E1 ' "$out" | head -n 1)"
  fi
  printf 'E7RESULT name=config arm=other\nE7RESULT name=done emitted=2\n' | gzip -c >"$root/research/results/e1-sample.out.gz"
  gz_round 2
  if [[ "$exit_code" == 1 ]] && grep -qE '^E1 +e1-sample +对不上 +e1-sample\.out\.gz，[0-9]+ 行不同 → diff .* research/results/e1-sample\.out\.gz' "$out"; then
    echo "gz-stored-red ok"
  else
    echo "gz-stored-red bad 留存产物 e1-sample.out.gz 换成内容不同的 gzip 应当判对不上、整趟退 1；退 $exit_code，$(grep -E '^E1 ' "$out" | head -n 1)"
  fi
}

# 入口脚本格式与三条完整性闸（文件头「入口脚本」「三条完整性闸」）的自证，replay_selftest 调：同一份临时仓（replay_selftest_sample_repository），
# 每一格往 stdout 打一行「<格名> ok」或「<格名> bad <一句>」，每一趟之前先把 E1、E2 的装置源码各改一个字（entry_round：装置源码进行键，各格键不同，互不复用）：
#   list-items              --list-items 逐行打 E1、E2（几行、二进制），退 0，本轮输出目录（REPLAY_OUT）不建、一个装置都不起
#   unknown-experiment      点名表里没有的 E9：退 2、点名 E9，本轮输出目录不建、一个装置都不起
#   integrity-exit-code     闸 3：E1 的装置照常打两行却退 3，E1 记「跑不了 退出码 3」、整趟退 1（不当成跑过了）
#   integrity-emitted       闸 2：E2 的装置收尾行说 emitted=3、实收 2 条，E2 记「跑不了」、整趟退 1
#   integrity-previous-output  同一个 REPLAY_OUT 接连两趟：前一趟 E2 的留存产物从 git 历史取回（取回的那一份留在本轮输出目录里），后一趟树里补上了内容不同的留存产物：
#                           E2 判对不上、diff 点名树里的 research/results/e2-sample.out（读树里的，不读上一趟取回的那一份）
replay_entry_selftest() {
  local work="$1" root calls out total exit_code=0
  root="$work/root"; calls="$work/devices-ran.log"
  replay_selftest_sample_repository "$work" || return 1
  entry_round() { # <趟号> <REPLAY_OUT 的名字> [参数…]：在临时仓里跑一趟副本；输出进 $out、起了哪几个装置进 $calls、末行 REPLAY_TOTAL 进 $total、退出码进 exit_code。先把 E1、E2 的装置源码各改一个字（行键变，各趟互不复用）
    out="$work/out.$1"; : >"$calls"; : >"$work/not-run.$1"
    printf 'fn main() { println!("one-entry-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e1_sample.rs"
    printf 'fn main() { println!("two-entry-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e2_sample.rs"
    if (cd "$root" && env -u SINGLEFS_MULTI_HOST_CONFIG -u SINGLEFS_REUSE_SENTINEL_SEED -u CARGO_TARGET_DIR TMPDIR="$work/tmp" PATH="$work/bin:$PATH" REPLAY_OUT="$work/$2" REPLAY_JOBS=1 \
          REPLAY_SELFTEST_RUN_LOG="$calls" REPLAY_SELFTEST_CARGO_LOG="$work/cargo-calls.log" GATE_NOT_RUN_FILE="$work/not-run.$1" \
          SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" SINGLEFS_REUSE_SENTINEL_COUNT=1000 bash scripts/experiment-apparatus/replay.sh "${@:3}") >"$out" 2>&1; then
      exit_code=0
    else
      exit_code=$?
    fi
    total="$(grep '^REPLAY_TOTAL ' "$out")"
  }
  device() { # <装置名> <退出码> <收尾行的 emitted=>：把临时仓里那个假装置换成打 config 与 done 两行、退给定退出码的一份（假 cargo 编的时候拷进这一趟的编译目录）
    printf '%s\n' '#!/usr/bin/env bash' "echo $1 >>\"\${REPLAY_SELFTEST_RUN_LOG:?}\"" \
      "printf 'E7RESULT name=config arm=$1\\nE7RESULT name=done emitted=$3\\n'" "exit $2" >"$work/devices/$1-sample"
    chmod +x "$work/devices/$1-sample"
  }
  entry_round 1 list-out --list-items
  # 临时仓里没有 scripts/hosts/heavy-threads.sh，载入时 stderr 有一句找不到它（与这一格无关）：只比制表符隔开的那几行
  if [[ "$exit_code" == 0 && "$(grep -P '\t' "$out")" == $'E1\t1\te1-sample\nE2\t1\te2-sample' && ! -e "$work/list-out" && ! -s "$calls" ]]; then
    echo "list-items ok"
  else
    echo "list-items bad --list-items 应当逐行打 E1、E2（几行、二进制）、退 0、不建本轮输出目录、不起装置；退 $exit_code，输出「$(tr '\n' ';' <"$out" | cut -c1-300 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)」，输出目录 $([[ -e "$work/list-out" ]] && echo 建了 || echo 没建)"
  fi
  entry_round 2 unknown-out E9
  if [[ "$exit_code" == 2 ]] && grep -qF '复跑表里没有：E9' "$out" && [[ ! -e "$work/unknown-out" && ! -s "$calls" ]]; then
    echo "unknown-experiment ok"
  else
    echo "unknown-experiment bad 点名表里没有的 E9 应当退 2、点名 E9、不建本轮输出目录、不起装置；退 $exit_code，输出尾部「$(tail -n 3 "$out" | tr '\n' ' ' | cut -c1-300 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)」"
  fi
  device e1 3 2
  entry_round 3 exit-code-out E1
  if [[ "$exit_code" == 1 ]] && grep -qE '^E1 +e1-sample +跑不了 +退出码 3' "$out"; then
    echo "integrity-exit-code ok"
  else
    echo "integrity-exit-code bad E1 的装置照常打两行却退 3，应当记跑不了、整趟退 1；退 $exit_code，$(grep -E '^E1 ' "$out" | head -n 1)"
  fi
  device e1 0 2
  device e2 0 3
  entry_round 4 emitted-out E2
  if [[ "$exit_code" == 1 ]] && grep -qE '^E2 +e2-sample +跑不了 +第 1 段收尾行说 3 条，实收 2 条' "$out"; then
    echo "integrity-emitted ok"
  else
    echo "integrity-emitted bad E2 的收尾行说 emitted=3、实收 2 条，应当记跑不了、整趟退 1；退 $exit_code，$(grep -E '^E2 ' "$out" | head -n 1)"
  fi
  device e2 0 2
  entry_round 5 previous-out E2
  local earlier_round_exit="$exit_code" earlier_round_line
  earlier_round_line="$(grep -E '^E2 ' "$out" | head -n 1)"
  printf 'E7RESULT name=config arm=other\nE7RESULT name=done emitted=2\n' >"$root/research/results/e2-sample.out"
  entry_round 6 previous-out E2
  rm -f -- "$root/research/results/e2-sample.out"
  if [[ "$earlier_round_exit" == 0 && "$exit_code" == 1 ]] && grep -qE '^E2 +e2-sample +字节一致 +e2-sample\.out（取自 git show' <<<"$earlier_round_line" \
     && grep -qE '^E2 +e2-sample +对不上 +e2-sample\.out，[0-9]+ 行不同 → diff .* research/results/e2-sample\.out' "$out"; then
    echo "integrity-previous-output ok"
  else
    echo "integrity-previous-output bad 同一个 REPLAY_OUT 接连两趟，前一趟 E2 取回留存产物判字节一致，后一趟树里补上了内容不同的留存产物，应当判对不上、diff 点名树里的那一份；前一趟退 $earlier_round_exit（$earlier_round_line），后一趟退 $exit_code（$(grep -E '^E2 ' "$out" | head -n 1)）"
  fi
}

# 编译目录（文件头「编译目录」）的自证，replay_selftest 调：同一份临时仓（replay_selftest_sample_repository），假 cargo 编的时候把假装置拷进 CARGO_TARGET_DIR 的 release/；
# 每一格往 stdout 打一行「<格名> ok」或「<格名> bad <一句>」，每一趟之前先把 E1 的装置源码改一个字（行键变了，各格互不复用）、只跑 E1，各格一个自己的 TMPDIR（$work/tmp-<格名>，这一趟的编译目录建在它底下）
# 与自己的本轮输出目录（REPLAY_OUT=$work/out-<格名>）：
#   build-directory-removed-when-green        E1 字节一致、整趟退 0：假 cargo 拿到的 CARGO_TARGET_DIR 是 TMPDIR 底下的 singlefs-replay-build-*/target，
#                                             E1 的装置是从它的 release/ 起的，跑完 TMPDIR 底下不剩它
#   build-directory-removed-when-build-fails  假 cargo 编不过（退 101）：整趟退 2、装置没起，跑完 TMPDIR 底下不剩编译目录
#   build-directory-removed-when-stopped      E1 的装置开跑之后只给副本那一个进程（$!）发 TERM：5 秒内退 143，装置跟着停（它开着编译目录里自己那份文件），TMPDIR 底下不剩编译目录；
#                                             副本退了之后不在一个读的时刻上判：限时 5 秒，按装置的进程号等它真退出、轮询到 TMPDIR 底下不剩编译目录，两样都成了才不再等
#   caller-target-directory-kept              调用方设了 CARGO_TARGET_DIR：假 cargo 拿到的就是它、装置从它的 release/ 起，跑完它与里面的装置都在，TMPDIR 底下不另建
#   copy-build-directories-removed            E1 的装置在本轮输出目录里造一份副本（target/ 带 CACHEDIR.TAG，旁边一个 source/）：跑完 target/ 不在、source/ 与 E1 的产物照留
replay_build_directory_selftest() {
  local work="$1" root out exit_code=0 tmp leftover replay_pid waited settled stat_line device_pid device_state device_command
  root="$work/root"
  replay_selftest_sample_repository "$work" || return 1
  build_round() { # <格名> [环境变量赋值…]：在临时仓里只跑 E1（先把 E1 的装置源码改一个字：行键变，不读上一格记的判定），前台；输出进 $out、退出码进 exit_code、TMPDIR 底下剩的编译目录进 leftover
    tmp="$work/tmp-$1"; out="$work/out.$1"; mkdir -p "$tmp"; : >"$work/devices-ran.log"; : >"$work/cargo-calls.log"; : >"$work/cargo-calls.log.target"; : >"$work/device-paths.$1"
    printf 'fn main() { println!("one-build-%s"); }\n' "$1" >"$root/scripts/experiment-apparatus/e1_sample.rs"
    if env -C "$root" -u SINGLEFS_MULTI_HOST_CONFIG -u SINGLEFS_REUSE_SENTINEL_SEED -u CARGO_TARGET_DIR PATH="$work/bin:$PATH" TMPDIR="$tmp" \
         REPLAY_OUT="$work/out-$1" REPLAY_JOBS=1 REPLAY_SELFTEST_RUN_LOG="$work/devices-ran.log" REPLAY_SELFTEST_CARGO_LOG="$work/cargo-calls.log" \
         REPLAY_SELFTEST_DEVICE_PATHS="$work/device-paths.$1" GATE_NOT_RUN_FILE="$work/not-run.$1" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
         SINGLEFS_REUSE_SENTINEL_COUNT=1000 "${@:2}" bash scripts/experiment-apparatus/replay.sh E1 >"$out" 2>&1; then
      exit_code=0
    else
      exit_code=$?
    fi
    leftover="$(find "$tmp" -mindepth 1 -maxdepth 1 -name 'singlefs-replay-build-*' 2>/dev/null | tr '\n' ' ')"
  }
  build_round build-directory-removed-when-green
  if [[ "$exit_code" == 0 && -z "$leftover" ]] && grep -qE '^E1 +e1-sample +字节一致' "$out" \
     && grep -qxE "$tmp/singlefs-replay-build-[0-9]+-[A-Za-z0-9]+/target" "$work/cargo-calls.log.target" \
     && grep -qxE "$tmp/singlefs-replay-build-[0-9]+-[A-Za-z0-9]+/target/release/e1-sample" "$work/device-paths.build-directory-removed-when-green"; then
    echo "build-directory-removed-when-green ok"
  else
    echo "build-directory-removed-when-green bad 跑绿应当退 0、假 cargo 拿到的 CARGO_TARGET_DIR 与装置的路径都在 $tmp 下这一趟的编译目录里、跑完不剩它；退 $exit_code，cargo 拿到「$(sort -u "$work/cargo-calls.log.target" | tr '\n' ' ')」，装置在「$(tr '\n' ' ' <"$work/device-paths.build-directory-removed-when-green")」，剩「$leftover」，$(grep -E '^E1 ' "$out" | head -n 1)"
  fi
  build_round build-directory-removed-when-build-fails REPLAY_SELFTEST_BUILD_FAILS=1
  if [[ "$exit_code" == 2 && -z "$leftover" && ! -s "$work/devices-ran.log" ]]; then
    echo "build-directory-removed-when-build-fails ok"
  else
    echo "build-directory-removed-when-build-fails bad 编不过应当退 2、装置不起、跑完 $tmp 底下不剩这一趟的编译目录；退 $exit_code，起了「$(tr '\n' ' ' <"$work/devices-ran.log")」，剩「$leftover」，输出尾部 $(tail -n 4 "$out" | tr '\n' ' ')"
  fi
  # 被停：副本放后台起（env -C 进临时仓再 exec 成 bash，$! 就是副本那一个进程），等 E1 的装置写下它的进程号再发 TERM
  tmp="$work/tmp-build-directory-removed-when-stopped"; out="$work/out.build-directory-removed-when-stopped"; mkdir -p "$tmp"; : >"$work/devices-ran.log"
  printf 'fn main() { println!("one-build-stopped"); }\n' >"$root/scripts/experiment-apparatus/e1_sample.rs"
  env -C "$root" -u SINGLEFS_MULTI_HOST_CONFIG -u SINGLEFS_REUSE_SENTINEL_SEED -u CARGO_TARGET_DIR PATH="$work/bin:$PATH" TMPDIR="$tmp" \
    REPLAY_OUT="$work/out-build-directory-removed-when-stopped" REPLAY_JOBS=1 REPLAY_SELFTEST_RUN_LOG="$work/devices-ran.log" REPLAY_SELFTEST_CARGO_LOG="$work/cargo-calls.log" \
    REPLAY_SELFTEST_DEVICE_HOLD="$work/device-hold.pid" GATE_NOT_RUN_FILE="$work/not-run.stopped" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
    SINGLEFS_REUSE_SENTINEL_COUNT=1000 bash scripts/experiment-apparatus/replay.sh E1 >"$out" 2>&1 &
  replay_pid=$!
  for _ in $(seq 150); do
    [[ -s "$work/device-hold.pid" ]] && break
    sleep 0.1
  done
  kill -TERM "$replay_pid" 2>/dev/null
  waited=0
  # 副本退了之后在 wait 之前是僵尸，kill -0 照样成功：看 /proc 里的状态
  while ((waited < 50)) && IFS= read -r stat_line 2>/dev/null <"/proc/$replay_pid/stat" && [[ "${stat_line##*) }" != Z* ]]; do
    sleep 0.1
    waited=$((waited + 1))
  done
  wait "$replay_pid"
  exit_code=$?
  # 不在一个读的时刻上判：限时 5 秒，按进程号等装置真退出（/proc 里不在、或已是僵尸；进程号被别的进程复用了的不算装置）、轮询到 TMPDIR 底下不剩编译目录，两样都成了才不再等
  device_pid="$(cat "$work/device-hold.pid" 2>/dev/null)"
  settled=0
  while :; do
    leftover="$(find "$tmp" -mindepth 1 -maxdepth 1 -name 'singlefs-replay-build-*' 2>/dev/null | tr '\n' ' ')"
    device_state=gone
    if [[ -n "$device_pid" ]] && IFS= read -r stat_line 2>/dev/null <"/proc/$device_pid/stat" && [[ "${stat_line##*) }" != Z* ]] \
       && device_command="$(tr '\0' ' ' 2>/dev/null <"/proc/$device_pid/cmdline")" && [[ "$device_command" == */release/e1-sample* ]]; then device_state=running; fi
    [[ "$device_state" == gone && -z "$leftover" ]] && break
    ((settled < 50)) || break
    sleep 0.1
    settled=$((settled + 1))
  done
  if [[ -n "$device_pid" && "$exit_code" == 143 && "$waited" -lt 50 && -z "$leftover" && "$device_state" == gone ]]; then
    echo "build-directory-removed-when-stopped ok"
  else
    echo "build-directory-removed-when-stopped bad 装置开跑之后给副本发 TERM，应当 5 秒内退 143、装置跟着停、跑完 $tmp 底下不剩这一趟的编译目录；装置进程号「$device_pid」（$device_state），退 $exit_code、等了 $((waited / 10)) 秒多，副本退了之后又等了 $((settled / 10)) 秒多（限时 5 秒），剩「$leftover」，输出尾部 $(tail -n 6 "$out" | tr '\n' ' ')"
  fi
  mkdir -p "$work/caller-target"
  build_round caller-target-directory-kept CARGO_TARGET_DIR="$work/caller-target"
  if [[ "$exit_code" == 0 && -z "$leftover" && -x "$work/caller-target/release/e1-sample" ]] && grep -qxF "$work/caller-target" "$work/cargo-calls.log.target" \
     && grep -qxF "$work/caller-target/release/e1-sample" "$work/device-paths.caller-target-directory-kept"; then
    echo "caller-target-directory-kept ok"
  else
    echo "caller-target-directory-kept bad 调用方给了 CARGO_TARGET_DIR，应当照它编、装置从它的 release/ 起、跑完它还在、不另建编译目录；退 $exit_code，cargo 拿到「$(sort -u "$work/cargo-calls.log.target" | tr '\n' ' ')」，装置在「$(tr '\n' ' ' <"$work/device-paths.caller-target-directory-kept")」，剩「$leftover」"
  fi
  build_round copy-build-directories-removed REPLAY_SELFTEST_COPY_TARGET=1
  if [[ "$exit_code" == 0 && -z "$leftover" && ! -e "$work/out-copy-build-directories-removed/selftest-copy/target" && -d "$work/out-copy-build-directories-removed/selftest-copy/source" \
        && -s "$work/out-copy-build-directories-removed/E1.1.out" ]]; then
    echo "copy-build-directories-removed ok"
  else
    echo "copy-build-directories-removed bad 装置在本轮输出目录里造的副本编译目录（带 CACHEDIR.TAG）跑完应当不在、副本的 source/ 与 E1 的产物照留；退 $exit_code，本轮输出目录里有「$(find "$work/out-copy-build-directories-removed" -maxdepth 3 2>/dev/null | sed "s#$work/##" | tr '\n' ' ')」"
  fi
}

#   REPLAY_BREAK=jobs-from-nproc  同时跑几条实验的缺省取核数的一半
#       jobs-default（「✗ 自检：没设 REPLAY_JOBS 时同时跑几条实验要取登记的重活默认」）
#   REPLAY_BREAK=unknown-experiment-accepted  表里没有的实验号照收
#       unknown-experiment（「✗ 自检：入口与完整性闸 unknown-experiment：」）
#   REPLAY_BREAK=emitted-unchecked  闸 2 不核收尾行的 emitted= 与实收条数
#       integrity-emitted（「✗ 自检：入口与完整性闸 integrity-emitted：」）
#   REPLAY_BREAK=exit-code-ignored  闸 3 不看装置的退出码
#       integrity-exit-code（「✗ 自检：入口与完整性闸 integrity-exit-code：」）
#       连带红的格：「✗ 自检：留存产物取回 device-not-run-red：E1 的装置退 77 应当记跑不了、退出码 77」
#   REPLAY_BREAK=e182-transit-unset  driver_e182 与 driver_e182_units_kept 起装置时不设 E182_VERDICT_LIBRARY（装置删了缺省本地库之后退 2）
#       e182-transit 两个驱动各一格（「✗ 自检：driver_e182 起装置时要把 E182_VERDICT_LIBRARY 设成这一趟的中转目录」
#       「✗ 自检：driver_e182_units_kept 起装置时要把 E182_VERDICT_LIBRARY 设成这一趟的中转目录」）
#   REPLAY_BREAK=load-errors-unchecked  command_not_found_handle 不记找不到的命令（载入期一句注释丢了 # 只在 stderr 打一句、自证照判通过）
#       load-commands-not-found 的第二格（「✗ 自检：现调一个不存在的命令，command_not_found_handle 应当记下它的名字」）
replay_selftest() {
  local scratch failures=0 checked=0 status problems_last_line
  # 载入期找不到的命令（command_not_found_handle 记的），在自证别的步骤跑之前先存一份：load-commands-not-found 那一格判它
  local load_commands_not_found=()
  mapfile -t load_commands_not_found < <(replay_commands_not_found)
  # 按行复用、留存产物取回、.gz、编译目录那几组起的副本都读写判定库：只在临时 prune-server 上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」
  # --with-temporary-service），不是临时服务就一格都不跑、退 2，不写真服务（判据只在客户端的 selftest_service_problem 一处）
  local service_problem
  if ! service_problem="$(python3 -B -c 'import sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem())' "$PWD/scripts/prune-server" 2>&1)" \
     || [[ -n "$service_problem" ]]; then
    echo "  ✗ replay.sh 自检没跑：${service_problem:-载入 scripts/prune-server/prune_server_client.py 没成}"
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）"
    return 2
  fi
  scratch="$(mktemp -d "${TMPDIR:-/tmp}/replay-selftest-XXXXXX")"
  # 中途报错、被打断也删（成功、失败都删；local 变量到退出时已不在，路径在这里就展开）
  trap "rm -rf -- '$scratch'" EXIT
  mkdir -p "$scratch/bin"
  # 自证起的子进程的标记（文件头附近那一道拒）：自证里起的每一份 replay.sh 都继承它，平常跑设了 REPLAY_BREAK 的没有
  printf '%s\n' replay-selftest >"$scratch/mark"
  export REPLAY_SELFTEST_MARK="$scratch/mark"
  # 平常跑设了弄坏开关（门禁 code-tooling 的 break-switch-outside-selftest 格就这样探）：标记清掉，要退 2 并点名变量只在自证里认；guard-off 下这一道不拒，探针往下走到「复跑表里没有」退 2，不点名
  checked=$((checked + 1))
  local guard_probe_break=thread-registry-not-copied guard_probe_output guard_probe_status
  [[ "$REPLAY_BROKEN" != guard-off ]] || guard_probe_break=guard-off
  if guard_probe_output="$(env REPLAY_SELFTEST_MARK= REPLAY_BREAK="$guard_probe_break" bash scripts/experiment-apparatus/replay.sh --force --break-guard-probe-unknown-option 2>&1 </dev/null)"; then guard_probe_status=0; else guard_probe_status=$?; fi
  if [[ "$guard_probe_status" != 2 ]] || ! grep -qE 'REPLAY_BREAK.*(自证|selftest)' <<<"$guard_probe_output"; then
    echo "  ✗ 自检：平常跑设了弄坏开关应当退 2 并点名 REPLAY_BREAK 只在自证里认；实际退 $guard_probe_status：$(head -n 3 <<<"$guard_probe_output" | tr '\n' ' ')"
    echo "     → 怎么办：看文件头附近那一道拒（REPLAY_BREAK 非空而没有自证的标记就退 2）；REPLAY_BREAK=guard-off 设着的话这里本来就该红"
    failures=$((failures + 1))
  fi
  printf '%s\n' '#!/usr/bin/env bash' 'echo "   Compiling e7-index-bench v0.1.0"' \
    'if [[ "${FAKE_CARGO_OUTCOME:-}" == red ]]; then echo "error[E0425]: cannot find value \`missing\` in this scope"; echo "error: could not compile \`e7-index-bench\`"; exit 101; fi' \
    'echo "    Finished release"; exit 0' >"$scratch/bin/cargo"
  chmod +x "$scratch/bin/cargo"
  checked=$((checked + 1))
  if (PATH="$scratch/bin:$PATH" FAKE_CARGO_OUTCOME=red build_research_binaries "$scratch/build-red.log") 2>"$scratch/build-red.stderr"; then status=0; else status=$?; fi
  if [[ "$status" != 2 ]] || ! grep -qF "第一处错：error[E0425]: cannot find value" "$scratch/build-red.stderr" \
     || ! grep -qF "    error: could not compile" "$scratch/build-red.stderr" || ! grep -qF "整份构建日志：$scratch/build-red.log" "$scratch/build-red.stderr" \
     || ! grep -qF "单独重现：(cd $PWD && cargo build --release --manifest-path scripts/experiment-apparatus/Cargo.toml)" "$scratch/build-red.stderr" \
     || [[ ! -s "$scratch/build-red.log" ]]; then
    echo "  ✗ 自检：构建失败时应当返回 2、打第一处错、日志末尾、日志路径与重现命令，日志要落盘；实际返回 $status：$(cat "$scratch/build-red.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # build-two-logs：构建日志出两份（全量 build-red.log，旁边同名的 build-red.problems.log 带行号收 error 行、末行写退出码 101），stderr 点名错误警告日志的路径
  checked=$((checked + 1))
  problems_last_line="$(tail -n 1 "$scratch/build-red.problems.log" 2>/dev/null)"   # 管道收尾的 grep -q 在 pipefail 下会把命中读成没命中：先落到变量
  if [[ ! -s "$scratch/build-red.problems.log" ]] || ! grep -qE '^[0-9]+:error\[E0425\]: cannot find value' "$scratch/build-red.problems.log" \
     || ! grep -qE '^[0-9]+:error: could not compile' "$scratch/build-red.problems.log" || ! grep -qF '被包的命令退出码 101' <<<"$problems_last_line" \
     || ! grep -qF "错误警告日志：$scratch/build-red.problems.log" "$scratch/build-red.stderr"; then
    echo "  ✗ 自检：构建失败时应当出两份日志（build-red.log 全量；build-red.problems.log 带行号收两条 error、末行写退出码 101），并在 stderr 点名错误警告日志的路径；实际 $(ls "$scratch" | grep '^build-red' | tr '\n' ' ')：$(cat "$scratch/build-red.problems.log" 2>&1 | tr '\n' '|')"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  if (PATH="$scratch/bin:$PATH" FAKE_CARGO_OUTCOME=green build_research_binaries "$scratch/build-green.log") 2>"$scratch/build-green.stderr"; then status=0; else status=$?; fi
  if [[ "$status" != 0 || -s "$scratch/build-green.stderr" ]]; then
    echo "  ✗ 自检：构建成功时应当返回 0、一个字都不打；实际返回 $status：$(cat "$scratch/build-green.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  problems_last_line="$(tail -n 1 "$scratch/build-green.problems.log" 2>/dev/null)"
  if ! grep -qF '收了 0 行' <<<"$problems_last_line"; then
    echo "  ✗ 自检：构建成功时错误警告日志也要在、末行写收了 0 行（build-green.problems.log）；实际：$(cat "$scratch/build-green.problems.log" 2>&1 | tr '\n' '|')"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 驱动开跑前编研究二进制（build_research_driver_binary）：假 cargo 记收到的参数、照仓根编那种报错退 101；cargo 的输出进 stderr（这一行的 .err），stdout 一个字不打
  mkdir -p "$scratch/driver-bin"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "$*" >>"${FAKE_CARGO_LOG:?}"' 'echo "error: cannot specify features for packages outside of workspace"' 'exit 101' >"$scratch/driver-bin/cargo"
  chmod +x "$scratch/driver-bin/cargo"
  : >"$scratch/driver-cargo.log"
  checked=$((checked + 1))
  if (PATH="$scratch/driver-bin:$PATH" FAKE_CARGO_LOG="$scratch/driver-cargo.log" build_research_driver_binary e174-gpu e174-lease-drop-wait-memory-return) \
       >"$scratch/driver-build.stdout" 2>"$scratch/driver-build.err"; then status=0; else status=$?; fi
  if [[ "$status" != 1 || -s "$scratch/driver-build.stdout" ]] || ! grep -qF 'error: cannot specify features for packages outside of workspace' "$scratch/driver-build.err" \
     || ! grep -qxF 'build -q --release --manifest-path scripts/experiment-apparatus/Cargo.toml --features e174-gpu --bin e174-lease-drop-wait-memory-return' "$scratch/driver-cargo.log"; then
    echo "  ✗ 自检：驱动开跑前编研究二进制（build_research_driver_binary）要经 --manifest-path scripts/experiment-apparatus/Cargo.toml 编、编不过返回 1、cargo 的输出进 stderr（这一行的 .err 非空）、stdout 不打；实际返回 $status，cargo 收到「$(tr '\n' '|' <"$scratch/driver-cargo.log")」，stderr「$(head -c 300 "$scratch/driver-build.err" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)」，stdout「$(head -c 200 "$scratch/driver-build.stdout" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)」"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # E162、E163 那三个驱动同样经它编：起驱动本身，假 cargo 另记 BINDGEN_EXTRA_CLANG_ARGS（E162 两个要把它设进编那一步），编不过就停在编那一步。
  # 驱动定义在这个函数被调用之后才读到，这里从本文件里取出那三段定义、在子 shell 里 eval
  printf '%s\n' '#!/usr/bin/env bash' 'echo "$*|BINDGEN=${BINDGEN_EXTRA_CLANG_ARGS-没设}" >>"${FAKE_CARGO_LOG:?}"' 'echo "error: cannot specify features for packages outside of workspace"' 'exit 101' >"$scratch/driver-bin/cargo"
  local driver_case driver_name driver_expected driver_definitions
  driver_definitions="$(awk '/^(driver_e162_anchors|driver_e162_power_cut_selftest|driver_e163_r1_merge)\(\) \{/,/^}/' "${BASH_SOURCE[0]}")"
  checked=$((checked + 1))
  if [[ "$(grep -cE '^driver_e16[23]_[a-z0-9_]+\(\) \{' <<<"$driver_definitions")" != 3 ]]; then
    echo "  ✗ 自检：从 ${BASH_SOURCE[0]} 里取不全 driver_e162_anchors、driver_e162_power_cut_selftest、driver_e163_r1_merge 三段定义（取到的头几行：$(head -n 3 <<<"$driver_definitions" | tr '\n' '|')）"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  for driver_case in \
    "driver_e162_anchors|build -q --release --manifest-path scripts/experiment-apparatus/Cargo.toml --features e162-block-stores --bin e162-crash-verdict-block-store|BINDGEN=-I/usr/lib/gcc/x86_64-linux-gnu/13/include" \
    "driver_e162_power_cut_selftest|build -q --release --manifest-path scripts/experiment-apparatus/Cargo.toml --features e162-block-stores --bin e162-verdict-store-power-cut|BINDGEN=-I/usr/lib/gcc/x86_64-linux-gnu/13/include" \
    "driver_e163_r1_merge|build -q --release --manifest-path scripts/experiment-apparatus/Cargo.toml --features e163-gpu --bin e163-gpu-multicard-crc32c|BINDGEN=没设"; do
    driver_name="${driver_case%%|*}"; driver_expected="${driver_case#*|}"
    : >"$scratch/driver-cargo.log"
    checked=$((checked + 1))
    if (unset BINDGEN_EXTRA_CLANG_ARGS; OUT_DIR="$scratch"; eval "$driver_definitions"; PATH="$scratch/driver-bin:$PATH" FAKE_CARGO_LOG="$scratch/driver-cargo.log" "$driver_name") \
         >"$scratch/driver-build.stdout" 2>"$scratch/driver-build.err"; then status=0; else status=$?; fi
    if [[ "$status" != 1 || -s "$scratch/driver-build.stdout" ]] || ! grep -qF 'error: cannot specify features for packages outside of workspace' "$scratch/driver-build.err" \
       || [[ "$(cat "$scratch/driver-cargo.log")" != "$driver_expected" ]]; then
      echo "  ✗ 自检：$driver_name 开跑前要经 build_research_driver_binary 编（cargo 只起一次：$driver_expected）、编不过返回 1、cargo 的输出进 .err、stdout 不打；实际返回 $status，cargo 收到「$(tr '\n' '|' <"$scratch/driver-cargo.log")」，stderr「$(head -c 300 "$scratch/driver-build.err" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)」，stdout「$(head -c 200 "$scratch/driver-build.stdout" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)」"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done
  # E182 两个驱动（文件头「E182 的片结果库」）：起装置时 E182_VERDICT_LIBRARY 设成这一趟中转目录里各自那一种；从本文件取出两段定义、在子 shell 里 eval，假 cargo 记收到的值
  printf '%s\n' '#!/usr/bin/env bash' 'echo "$*|E182_VERDICT_LIBRARY=${E182_VERDICT_LIBRARY-没设}" >>"${FAKE_CARGO_LOG:?}"' 'exit 0' >"$scratch/driver-bin/cargo"
  local e182_definitions
  e182_definitions="$(awk '/^(driver_e182|driver_e182_units_kept)\(\) \{/,/^}/' "${BASH_SOURCE[0]}")"
  for driver_case in \
    "driver_e182|run -q --release -p singlefs-verification --bin e182_unreadable_abandoned_root_forks -- single-device|E182_VERDICT_LIBRARY=$scratch/e182-shards/e182-single-device" \
    "driver_e182_units_kept|run -q --release -p singlefs-verification --bin e182_unreadable_abandoned_root_forks -- single-device-units-kept|E182_VERDICT_LIBRARY=$scratch/e182-shards/e182-single-device-units-kept"; do
    driver_name="${driver_case%%|*}"; driver_expected="${driver_case#*|}"
    : >"$scratch/driver-cargo.log"
    checked=$((checked + 1))
    (unset E182_VERDICT_LIBRARY; CHECKER_TIER_FEATURES=(); REPLAY_E182_SHARDS="$scratch/e182-shards"; eval "$e182_definitions"
     PATH="$scratch/driver-bin:$PATH" FAKE_CARGO_LOG="$scratch/driver-cargo.log" "$driver_name") >/dev/null 2>&1
    if [[ "$(cat "$scratch/driver-cargo.log")" != "$driver_expected" ]]; then
      echo "  ✗ 自检：$driver_name 起装置时要把 E182_VERDICT_LIBRARY 设成这一趟的中转目录（cargo 只起一次：$driver_expected）；实际 cargo 收到「$(tr '\n' '|' <"$scratch/driver-cargo.log")」"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done
  checked=$((checked + 1))
  local -A ADMISSION_ROW_KEY=(["E1|@driver_e1|"]="E1" ["E1|@driver_e1_second|"]="E1/second")
  local ROW_ADMISSION_KEY=""
  admission_key_of_row E1 @driver_e1_second ""
  if [[ "$ROW_ADMISSION_KEY" != E1/second ]]; then
    echo "  ✗ 自检：复跑表那一行指明了准入键 E1/second，问准入就该用它；实际用的是「$ROW_ADMISSION_KEY」"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 配置开了 GPU 时 验证包装置带 gpu 特性编（resolve_checker_tier_features）：假 cargo 把收到的参数记进 gpu-calls.log，配置文件放临时目录
  mkdir -p "$scratch/gpu-bin"
  printf '%s\n' '#!/usr/bin/env bash' 'echo "cargo $*" >>"${FAKE_CARGO_LOG:?}"' \
    'if [[ "${FAKE_CARGO_OUTCOME:-}" == red ]]; then echo "error[E0432]: unresolved import wgpu"; exit 101; fi' 'exit 0' >"$scratch/gpu-bin/cargo"
  chmod +x "$scratch/gpu-bin/cargo"
  gpu_probe() { # <配置文件路径> <假 cargo 的结局>：子 shell 里跑一遍，打「状态 特性数组」一行；stderr 进 gpu-probe.stderr，cargo 的调用进 gpu-calls.log
    : >"$scratch/gpu-calls.log"
    ( export SINGLEFS_MULTI_HOST_CONFIG="$1" FAKE_CARGO_LOG="$scratch/gpu-calls.log" FAKE_CARGO_OUTCOME="$2" PATH="$scratch/gpu-bin:$PATH" \
        QUIESCE_CALLS="$scratch/quiesce-calls.log" GPU_CARDS_BY_PRIORITY=127.0.0.1:0:28G
      resolve_checker_tier_features "$scratch/gpu-probe.log" 2>"$scratch/gpu-probe.stderr"; probe_status=$?
      echo "$probe_status ${CHECKER_TIER_FEATURES[*]:-}" )
  }
  local gpu_reading
  # 开了 GPU 的多机配置：清场命令是假的（往 $QUIESCE_CALLS 记一笔）；本脚本不另套清场，这一笔不该有
  printf '%s\n' '# 自证的多机配置' 'ENABLE_GPU=1' \
    "QUIESCE_STOP_COMMAND='echo quiesce-stop >> \"\$QUIESCE_CALLS\"'" 'QUIESCE_STOPPED_CHECK_COMMAND=true' >"$scratch/gpu-on.env"
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-on.env" green)"
  if [[ "$gpu_reading" != "0 --features gpu" ]] \
     || ! grep -qxF "cargo build --release -p singlefs-verification --features gpu --lib" "$scratch/gpu-calls.log"; then
    echo "  ✗ 自检：配置写了 ENABLE_GPU=1，应当先试编 gpu 特性并把「--features gpu」放进 CHECKER_TIER_FEATURES；实际「$gpu_reading」，cargo 调用：$(tr '\n' '|' <"$scratch/gpu-calls.log")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  printf '%s\n' 'ENABLE_GPU=0' >"$scratch/gpu-off.env"
  printf '%s\n' 'ENABLE_ACROSS_MACHINES=1' >"$scratch/gpu-unset.env"
  checked=$((checked + 1))
  if [[ "$(gpu_probe "$scratch/gpu-off.env" green)" != "0 " || -s "$scratch/gpu-calls.log" || "$(gpu_probe "$scratch/gpu-unset.env" green)" != "0 " || -s "$scratch/gpu-calls.log" ]]; then
    echo "  ✗ 自检：配置写 ENABLE_GPU=0、或没写这个键，应当不带特性、一次 cargo 都不起；实际 $(gpu_probe "$scratch/gpu-off.env" green)，cargo 调用：$(tr '\n' '|' <"$scratch/gpu-calls.log")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  printf '%s\n' 'ENABLE_GPU=yes' >"$scratch/gpu-garbled.env"
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-garbled.env" green)"
  if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "ENABLE_GPU 写成了「yes」" "$scratch/gpu-probe.stderr" || [[ -s "$scratch/gpu-calls.log" ]]; then
    echo "  ✗ 自检：ENABLE_GPU 写了 0、1 之外的值，应当退 78、点名那个值、不起 cargo、不悄悄不带特性；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-no-such.env" green)"
  if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "点名的多机配置 $scratch/gpu-no-such.env 不在" "$scratch/gpu-probe.stderr"; then
    echo "  ✗ 自检：SINGLEFS_MULTI_HOST_CONFIG 点名的文件不在，应当退 78 并点名那个文件；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  if (( EUID != 0 )); then
    cp -- "$scratch/gpu-on.env" "$scratch/gpu-unreadable.env"; chmod 000 "$scratch/gpu-unreadable.env"
    checked=$((checked + 1))
    gpu_reading="$(gpu_probe "$scratch/gpu-unreadable.env" green)"
    if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "gpu-unreadable.env 不可读" "$scratch/gpu-probe.stderr" || [[ -s "$scratch/gpu-calls.log" ]]; then
      echo "  ✗ 自检：配置文件不可读，应当退 78、不起 cargo（不当成没开 GPU）；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  fi
  checked=$((checked + 1))
  gpu_reading="$(gpu_probe "$scratch/gpu-on.env" red)"
  if [[ "$gpu_reading" != "78 " ]] || ! grep -qF "第一处错：error[E0432]: unresolved import wgpu" "$scratch/gpu-probe.stderr" \
     || ! grep -qF "不退回不带 gpu 特性去跑" "$scratch/gpu-probe.stderr" || ! grep -qF "整份编译日志：$scratch/gpu-probe.log" "$scratch/gpu-probe.stderr"; then
    echo "  ✗ 自检：开了 GPU 而 gpu 特性编不过，应当退 78、打第一处错与日志路径、不带特性数组；实际「$gpu_reading」：$(cat "$scratch/gpu-probe.stderr")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 开了 GPU 的卡由服务交（service_gpu_cards_present）：cards_run 打「resolve=<返回值>」，过了再打照装置那样直接起的一条命令看到的「cards=<GPU_CARDS_BY_PRIORITY>」；
  #   多机配置里的清场命令是假的、往 quiesce-calls.log 记一笔（不另套清场，一笔都不该有）
  cards_run() { # <配置文件路径> <环境里的 GPU_CARDS_BY_PRIORITY，空串就是不设>
    : >"$scratch/gpu-calls.log"; : >"$scratch/quiesce-calls.log"
    ( export SINGLEFS_MULTI_HOST_CONFIG="$1" FAKE_CARGO_LOG="$scratch/gpu-calls.log" FAKE_CARGO_OUTCOME=green PATH="$scratch/gpu-bin:$PATH" QUIESCE_CALLS="$scratch/quiesce-calls.log"
      unset GPU_CARDS_BY_PRIORITY
      if [[ -n "$2" ]]; then export GPU_CARDS_BY_PRIORITY="$2"; fi
      resolve_checker_tier_features "$scratch/gpu-probe.log" 2>"$scratch/gpu-probe.stderr"; probe_status=$?
      echo "resolve=$probe_status"
      if (( probe_status == 0 )); then bash -c 'echo "cards=${GPU_CARDS_BY_PRIORITY-unset}"'; fi )
  }
  local cards_reading
  checked=$((checked + 1))
  cards_reading="$(cards_run "$scratch/gpu-on.env" 127.0.0.1:0:28G | tr '\n' ' ')"
  if [[ "$cards_reading" != "resolve=0 cards=127.0.0.1:0:28G " || -s "$scratch/quiesce-calls.log" ]]; then
    echo "  ✗ 自检：开了 GPU、环境里有服务交的 GPU_CARDS_BY_PRIORITY，应当返回 0、照装置那样起的命令看到的就是那一份 127.0.0.1:0:28G、一条清场命令都不跑；实际「$cards_reading」，假清场命令记了「$(tr '\n' '|' <"$scratch/quiesce-calls.log")」：$(tail -n 3 "$scratch/gpu-probe.stderr" | tr '\n' ' ')"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  cards_reading="$(cards_run "$scratch/gpu-on.env" '' | tr '\n' ' ')"
  if [[ "$cards_reading" != "resolve=78 " ]] || ! grep -qF "环境里却没有 GPU_CARDS_BY_PRIORITY" "$scratch/gpu-probe.stderr" || [[ -s "$scratch/quiesce-calls.log" ]]; then
    echo "  ✗ 自检：开了 GPU 而环境里没有服务交的 GPU_CARDS_BY_PRIORITY，应当退 78、点名 GPU_CARDS_BY_PRIORITY、一条清场命令都不跑；实际「$cards_reading」，假清场命令记了「$(tr '\n' '|' <"$scratch/quiesce-calls.log")」：$(tail -n 3 "$scratch/gpu-probe.stderr" | tr '\n' ' ')"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 起 验证包装置的 cargo run 行（用到给定状态引擎的那几个装置）每一行都带特性数组：新加一行忘了带，开了 GPU 时那一行拿不到判段结论
  local engine_lines unmarked_lines wrapper_name wrapped_lines
  engine_lines="$(grep -E 'cargo run .*-p singlefs-verification .*--bin (e156_|e158_|e161_|e168_|e170_|e171_|"\$bin")' -- "${BASH_SOURCE[0]}" | grep -vE '^[[:space:]]*#' || true)"
  if [[ "$REPLAY_BROKEN" == run-lines-without-features ]]; then engine_lines="${engine_lines//CHECKER_TIER_FEATURES/}"; fi
  unmarked_lines="$(grep -vF 'CHECKER_TIER_FEATURES' <<<"$engine_lines" || true)"
  checked=$((checked + 1))
  if [[ -z "$engine_lines" || -n "$unmarked_lines" ]]; then
    echo "  ✗ 自检：起 验证包装置的 cargo run 行（共 $(grep -c . <<<"$engine_lines") 行）应当每一行都带 \${CHECKER_TIER_FEATURES[@]…}；缺的：$(head -n 3 <<<"$unmarked_lines" | cut -c1-200 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 起 验证包装置不另套清场（清场与分卡归统一任务服务，任务里面再调清场包装由包装自己拒）：同一批行没有一行带清场包装或清场前缀，
  #   本文件不是注释的行里也不起清场包装（包装名拆开写，这一格自己的行不算命中）
  wrapper_name="gpu-quiesce-run"'.sh'
  if [[ "$REPLAY_BROKEN" == run-line-quiesce-wrapped ]]; then
    engine_lines+=$'\n'"bash scripts/crash/$wrapper_name -- cargo ""run -q --release -p singlefs-verification --bin e156_allocation_basis_counts"   # cargo 与 run 拆开写：不让上面那一格扫装置行时把这一行当成装置行
  fi
  wrapped_lines="$( { grep -vE '^[[:space:]]*#' -- "${BASH_SOURCE[0]}" | grep -F "$wrapper_name"; grep -E "QUIESCE|$wrapper_name" <<<"$engine_lines"; } || true)"
  checked=$((checked + 1))
  if [[ -z "$engine_lines" || -n "$wrapped_lines" ]]; then
    echo "  ✗ 自检：起 验证包装置不另套清场：cargo run 行（共 $(grep -c . <<<"$engine_lines") 行）不该带清场前缀，本文件不是注释的行里也不该起 $wrapper_name；命中的：$(head -n 3 <<<"$wrapped_lines" | cut -c1-200 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 拷 crates/ 去编译的副本带上每台默认重活线程数的登记文件（copy_thread_registry_into）：拷进去的与仓里那一份逐字节相同
  checked=$((checked + 1))
  copy_thread_registry_into "$scratch/registry-tree"
  if ! cmp -s -- .claude/heavy-thread-default "$scratch/registry-tree/.claude/heavy-thread-default"; then
    echo "  ✗ 自检：拷 crates/ 去编译的副本要带上每台默认重活线程数的登记文件 .claude/heavy-thread-default（harness 经 include_str! 读它），副本里实际有「$(ls -A "$scratch/registry-tree/.claude" 2>/dev/null | tr '\n' ' ')」"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 拷 crates 的每一行（「Cargo.lock .cargo crates」那几行）都在同一行里接着 copy_thread_registry_into：新加一行忘了带，那个副本编不过 harness
  local copy_lines unregistered_copy_lines
  copy_lines="$(grep -E 'Cargo\.lock \.cargo crates' -- "${BASH_SOURCE[0]}" | grep -vE '^[[:space:]]*#' || true)"
  if [[ "$REPLAY_BROKEN" == copy-lines-without-registry ]]; then copy_lines="${copy_lines//copy_thread_registry_into/}"; fi
  unregistered_copy_lines="$(grep -vF 'copy_thread_registry_into' <<<"$copy_lines" || true)"
  checked=$((checked + 1))
  if [[ -z "$copy_lines" || -n "$unregistered_copy_lines" ]]; then
    echo "  ✗ 自检：拷 crates/ 去编译的行（共 $(grep -c . <<<"$copy_lines") 行）应当每一行都接着 copy_thread_registry_into <副本>；缺的：$(head -n 3 <<<"$unregistered_copy_lines" | cut -c1-200 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 拷今天这棵树去编译的副本照共用库的闭包拷全（replay_copy_closure_into）：lib_repository_copy.py entries 列的每一条（path 依赖的 prune-server/、
  # 编译期读的 .claude/heavy-thread-default、运行期起的脚本）在副本里都在；e168_patched_tree、driver_e171_s4_comparator 与 driver_e175 都经它拷
  local closure_entries closure_entry closure_missing="" patched_tree_definition frozen_tree_definition positive_controls_definition repository_copy_definition
  # 驱动定义在这个函数被调用之后才读到：从本文件里取出那四段定义，落进变量再比（管道收尾的 grep -q 在 pipefail 下会把命中读成没命中）
  patched_tree_definition="$(awk '/^e168_patched_tree\(\) \{/,/^}/' "${BASH_SOURCE[0]}")"
  frozen_tree_definition="$(awk '/^driver_e171_s4_comparator\(\) \{/,/^}/' "${BASH_SOURCE[0]}")"
  positive_controls_definition="$(awk '/^driver_e168_positive_controls\(\) \{/,/^}/' "${BASH_SOURCE[0]}")"
  repository_copy_definition="$(awk '/^driver_e175\(\) \{/,/^}/' "${BASH_SOURCE[0]}")"
  checked=$((checked + 1))
  closure_entries="$(python3 scripts/hosts/lib_repository_copy.py entries . | awk -F'\t' '/^REPOSITORY_COPY entry /{ sub(/^REPOSITORY_COPY entry /, "", $1); print $1 }')"
  replay_copy_closure_into "$scratch/closure-tree" 2>"$scratch/closure-tree.err"
  while IFS= read -r closure_entry; do
    [[ -z "$closure_entry" || -e "$scratch/closure-tree/$closure_entry" ]] || closure_missing+=" $closure_entry"
  done <<<"$closure_entries"
  if [[ -z "$closure_entries" || -n "$closure_missing" || "$patched_tree_definition" != *replay_copy_closure_into* \
        || "$frozen_tree_definition" != *replay_copy_closure_into* || "$repository_copy_definition" != *replay_copy_closure_into* ]]; then
    echo "  ✗ 自检：拷今天这棵树去编译的副本要照共用库的闭包拷全（lib_repository_copy.py entries 列的 $(grep -c . <<<"$closure_entries") 条都在，e168_patched_tree、driver_e171_s4_comparator 与 driver_e175 都经 replay_copy_closure_into 拷）；副本里缺的：${closure_missing:- 没有}；$(tail -n 2 "$scratch/closure-tree.err" 2>/dev/null | tr '\n' ' ' | cut -c1-300)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  rm -rf -- "${scratch:?}/closure-tree"
  # 副本里起命令的多机配置取主工作区那一份（replay_configuration_for_the_copy）：没设 SINGLEFS_MULTI_HOST_CONFIG 时打 <主工作区>/multi-host.env，设了的照原样；
  # driver_e168_positive_controls 在副本里起 cargo 之前把它交给 SINGLEFS_MULTI_HOST_CONFIG
  local configuration_unset configuration_given main_tree
  checked=$((checked + 1))
  if main_tree="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"; then main_tree="$(cd "$(dirname "$main_tree")" && pwd -P)"; else main_tree="$(pwd -P)"; fi
  configuration_unset="$(unset SINGLEFS_MULTI_HOST_CONFIG; replay_configuration_for_the_copy)"
  configuration_given="$(export SINGLEFS_MULTI_HOST_CONFIG=/given/multi-host.env; replay_configuration_for_the_copy)"
  if [[ "$configuration_unset" != "$main_tree/multi-host.env" || "$configuration_given" != /given/multi-host.env \
        || "$positive_controls_definition" != *'export SINGLEFS_MULTI_HOST_CONFIG="$configuration"'* ]]; then
    echo "  ✗ 自检：副本里起命令的多机配置要取主工作区那一份（没设时应当是 $main_tree/multi-host.env、设了的照原样，driver_e168_positive_controls 起 cargo 前交给 SINGLEFS_MULTI_HOST_CONFIG）；实际没设得「$configuration_unset」、设了得「$configuration_given」"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 计时行的负载关：假产物放进 $scratch/results/，复跑表是下面这几行，滞后表登记 e-lagged.out
  mkdir -p "$scratch/results"
  local -A TIMING_LAG=()
  local TIMING_LOAD_RATIO=0.5 lag_table="$scratch/lag.tsv" lag_problems
  local selftest_table
  selftest_table="$(printf '%s\n' 'E1|e1-high||e-high.out|timing' 'E2|e2-low||e-low.out|timing' 'E3|e3-none||e-none.out|timing' 'E4|e4-lagged||e-lagged.out|timing' 'E5|e5-exact||e-exact.out|exact')"
  printf '%s\n' 'E7RESULT name=config threads=4' 'E7RESULT name=done emitted=2 loadavg1=20.50 cpus=32' >"$scratch/results/e-high.out"
  printf '%s\n' 'E7RESULT name=config threads=4' 'E7RESULT name=done emitted=2 loadavg1=2.50 cpus=32' >"$scratch/results/e-low.out"
  printf '%s\n' 'E7RESULT name=config threads=4' 'E7RESULT name=done emitted=2' >"$scratch/results/e-none.out"
  cp -- "$scratch/results/e-none.out" "$scratch/results/e-lagged.out"
  printf '%s\n' '# 样本滞后表' '' $'e-lagged.out\t2026-10-13 立 C350 之前的产物，装置还没加收尾字段' >"$lag_table"
  checked=$((checked + 1))
  if ! load_timing_lag "$lag_table" "$selftest_table" "$scratch/results" >"$scratch/lag.problems"; then   # 不经命令替换：TIMING_LAG 要留在这一层
    echo "  ✗ 自检：只登记了复跑表里 timing 行的留存产物、理由写全的滞后表应当读得过；实际报：$(cat "$scratch/lag.problems")"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-high.out "$scratch/results/e-high.out"
  if [[ "$TIMING_LOAD_DECISION" != skip || "$TIMING_LOAD_NOTE" != *"比值 0.640625"* ]]; then
    echo "  ✗ 自检：负载 20.5 / 32 核超过阈值 0.5，这一行应当记本次不判并点名比值；实际 $TIMING_LOAD_DECISION：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-low.out "$scratch/results/e-low.out"
  if [[ "$TIMING_LOAD_DECISION" != judge || "$TIMING_LOAD_LAGGED" != 0 ]]; then
    echo "  ✗ 自检：负载 2.5 / 32 核没超阈值，这一行应当照判；实际 $TIMING_LOAD_DECISION（滞后 $TIMING_LOAD_LAGGED）：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-none.out "$scratch/results/e-none.out"
  if [[ "$TIMING_LOAD_DECISION" != red || "$TIMING_LOAD_NOTE" != *"e-none.out"* || "$TIMING_LOAD_NOTE" != *"收尾行没有 loadavg1 / cpus"* ]]; then
    echo "  ✗ 自检：收尾行没有字段又不在滞后表，这一行应当判红并点名产物与「收尾行没有 loadavg1 / cpus」；实际 $TIMING_LOAD_DECISION：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide timing e-lagged.out "$scratch/results/e-lagged.out"
  if [[ "$TIMING_LOAD_DECISION" != judge || "$TIMING_LOAD_LAGGED" != 1 ]]; then
    echo "  ✗ 自检：收尾行没有字段但登记在滞后表里，这一行应当照判并记滞后；实际 $TIMING_LOAD_DECISION（滞后 $TIMING_LOAD_LAGGED）：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  timing_load_decide exact e-none.out "$scratch/results/e-none.out"
  if [[ "$TIMING_LOAD_DECISION" != judge ]]; then
    echo "  ✗ 自检：判据写 exact 的行不看负载字段，没字段也照判；实际 $TIMING_LOAD_DECISION：$TIMING_LOAD_NOTE"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  printf '%s\n' $'e-ghost.out\t2026-10-13 不在复跑表里的名字' $'e-exact.out\t2026-10-13 复跑表里是 exact 的行' $'e-lagged.out\t2026-10-13 重复登记' $'e-short.out\t短' 'e-spaced.out  # 用空格隔开理由的旧写法，不是两列' >>"$lag_table"
  if lag_problems="$(load_timing_lag "$lag_table" "$selftest_table" "$scratch/results")" \
     || [[ "$lag_problems" != *"e-ghost.out 不是复跑表任何一条 timing 行"* || "$lag_problems" != *"e-exact.out 不是复跑表任何一条 timing 行"* \
        || "$lag_problems" != *"重复登记 e-lagged.out"* || "$lag_problems" != *"理由不足 8 个字"* \
        || "$lag_problems" != *"没有用制表符隔开的理由"* ]]; then
    echo "  ✗ 自检：滞后表登记了复跑表里没有的名字、exact 行的产物、重复名字、没写够理由、用空格隔开理由，应当判红并逐条点名；实际：$lag_problems"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  printf '%s\n' '# 样本滞后表' $'e-high.out\t2026-10-13 这份已经有字段了却还登记着' >"$lag_table"
  if lag_problems="$(load_timing_lag "$lag_table" "$selftest_table" "$scratch/results")" || [[ "$lag_problems" != *"e-high.out 自己的收尾行已经有 loadavg1 / cpus"* ]]; then
    echo "  ✗ 自检：滞后表登记的留存产物自己已经有字段，应当判红叫人删这一行；实际：$lag_problems"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  if [[ "$(printf '%s\n' 'E7RESULT name=config cpus=4' 'E7RESULT name=done emitted=2 loadavg1=2.50 cpus=32' | strip_load_fields)" != $'E7RESULT name=config cpus=4\nE7RESULT name=done emitted=2' ]]; then
    echo "  ✗ 自检：比对前要删掉 name=done 行上的 loadavg1= 与 cpus=、别的行上的同名字段不动"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 按行复用（文件头「按行复用」）：十二格各一行收回来，收回的格数对不上整个算一处不对。
  #   收格的 grep 带 -a：哪一格的说明里混进了不合法的 UTF-8（按字节截的半个字），GNU grep 照二进制只报「binary file matches」、那一格的行不交出来（少收一格），
  #   -a 照文本收；各格说明里按字节截的那几处另经 iconv -c 去掉半个字（本次未跑与分组的收法同此）
  local reuse_cell reuse_verdict reuse_detail reuse_collected=0
  replay_reuse_selftest "$scratch/reuse" >"$scratch/reuse.report" 2>&1
  while read -r reuse_cell reuse_verdict reuse_detail; do
    checked=$((checked + 1)); reuse_collected=$((reuse_collected + 1))
    if [[ "$reuse_verdict" != ok ]]; then
      echo "  ✗ 自检：按行复用 $reuse_cell：$reuse_detail"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done < <(grep -aE '^(reuse-[a-z0-9-]+|last-run-list) (ok|bad)' "$scratch/reuse.report")
  grep '^REUSE_SELFTEST ' "$scratch/reuse.report"
  if (( reuse_collected != 12 )); then
    echo "  ✗ 自检：按行复用的自证只收回 $reuse_collected 格（要 12 格）：$(tail -n 6 "$scratch/reuse.report" | tr '\n' ' ' | cut -c1-800 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 留存产物取回与装置读不到输入（文件头「驱动要读的输入」）：三格各一行收回来，收回的格数对不上整个算一处不对
  local archived_cell archived_verdict archived_detail archived_collected=0
  replay_archived_selftest "$scratch/archived" >"$scratch/archived.report" 2>&1
  while read -r archived_cell archived_verdict archived_detail; do
    checked=$((checked + 1)); archived_collected=$((archived_collected + 1))
    if [[ "$archived_verdict" != ok ]]; then
      echo "  ✗ 自检：留存产物取回 $archived_cell：$archived_detail"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done < <(grep -aE '^(archived|device)-[a-z-]+ (ok|bad)' "$scratch/archived.report")
  if (( archived_collected != 3 )); then
    echo "  ✗ 自检：留存产物取回的自证只收回 $archived_collected 格（要 3 格）：$(tail -n 6 "$scratch/archived.report" | tr '\n' ' ' | cut -c1-800 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # 留存产物只以 .gz 存在时的比对（replay_gzip_selftest，两格）与入口脚本格式、三条完整性闸（replay_entry_selftest，五格）：每格一行收回来，
  # 收回的格数对不上整个算一处不对。一组一行「<函数>;<报告名>;<格名的式样>;<要几格>;<报的时候怎么称呼>」。
  # 两组共用这一段收法：这份文件里程序读的注释（gate-lint 的明细标记这一类）进复跑表每一行的行键（lib_function_units.py 的 kept-comments），多一处就让每一行都现跑
  local group_function group_report group_cells group_wanted group_title group_cell group_verdict group_detail group_collected
  while IFS=';' read -r group_function group_report group_cells group_wanted group_title; do
    group_collected=0
    "$group_function" "$scratch/$group_report" >"$scratch/$group_report.report" 2>&1
    while read -r group_cell group_verdict group_detail; do
      checked=$((checked + 1)); group_collected=$((group_collected + 1))
      if [[ "$group_verdict" != ok ]]; then
        echo "  ✗ 自检：$group_title $group_cell：$group_detail"  # gate-lint:detail
        failures=$((failures + 1))
      fi
    done < <(grep -aE "^($group_cells) (ok|bad)" "$scratch/$group_report.report")
    if (( group_collected != group_wanted )); then
      echo "  ✗ 自检：$group_title 的自证只收回 $group_collected 格（要 $group_wanted 格）：$(tail -n 6 "$scratch/$group_report.report" | tr '\n' ' ' | cut -c1-800 | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done <<'SELFTEST_GROUPS'
replay_gzip_selftest;gzip;gz-stored-[a-z]+;2;留存产物 .gz
replay_entry_selftest;entry;list-items|unknown-experiment|integrity-[a-z-]+;5;入口与完整性闸
replay_build_directory_selftest;build-directory;build-directory-[a-z-]+|caller-target-directory-kept|copy-build-directories-removed;5;编译目录
SELFTEST_GROUPS
  # 驱动开跑前核输入（replay_inputs_present）：从本文件取出这几个驱动与它们用的函数、全局量，在一个没有 research/ 的空目录里起，
  # 冻结根与导出根在固定的位置（工作根下 replay-frozen-roots/，这里没建）；每个都要退 1、stderr 里打「✗ 输入不在 test=<驱动名>」行恰好那么多行（缺几样报几样），stdout 不打。
  # 弄坏开关 REPLAY_BREAK=inputs-not-checked 下它们往下去起编不存在的脚本，退的不是 1，这几格转红。
  local input_driver_definitions input_case input_driver input_expected input_lines input_status
  input_driver_definitions="$(awk '
    /^(E171_FREEZE_REFERENCE|E171_R1_DUMPS_REMEDY|E158_[A-Z0-9_]*ARMS)=/ { print; next }
    /^(driver_e142|e170_r2_judge_images|driver_e170_r2_v2_487_z3_images|driver_e170_r2_v2_487_z4_images|driver_e171_s2_images_o3|driver_e171_s2_o3_control_baseline|driver_e171_s2_comparator|driver_e171_s3_comparator_mutations|driver_e171_s4_comparator|driver_e171_s4_f4_comparator|driver_e171_s4_f4_substitutes_comparator|e171_r1_revision_6_python|e171_r1_inputs_present|e171_r1_view|driver_e171_r1_s1_revision_6_comparator|driver_e171_r1_s1_revision_6_cross_host_check|driver_e171_r1_s2_revision_6_comparator|driver_e174_simulated_judge|driver_e174_segment_one_judge|driver_e174_segment_two_judge|driver_e158_r3_seg1_compare|driver_e158_r4_compare|driver_e165_segment2|driver_e165_r3_segment3_root_choice_kill_bounds|driver_e165_r4_segment2_root_choice_faults_kill_bounds|driver_e165_r5_segment2_r2_mode_kill_bounds|driver_e165_r5_segment2_r3_mode_kill_bounds|driver_e165_r5_segment2_r4_mode_kill_bounds|driver_e165_r5_segment5b_r4_gate_r2_mode_kill_bounds)\(\) \{/ {
      if ($0 ~ /; \}[[:space:]]*$/) { print; next }
      inside = 1
    }
    inside { print; if ($0 ~ /^\}/) inside = 0 }' "${BASH_SOURCE[0]}")"
  mkdir -p "$scratch/inputs-empty-root" "$scratch/inputs-out"
  for input_case in driver_e142:1 driver_e170_r2_v2_487_z3_images:1 driver_e170_r2_v2_487_z4_images:1 \
    driver_e171_s2_images_o3:1 driver_e171_s2_o3_control_baseline:1 driver_e171_s2_comparator:1 driver_e171_s3_comparator_mutations:1 \
    driver_e171_s4_comparator:1 driver_e171_s4_f4_comparator:1 driver_e171_s4_f4_substitutes_comparator:1 \
    driver_e171_r1_s1_revision_6_comparator:14 driver_e171_r1_s1_revision_6_cross_host_check:3 driver_e171_r1_s2_revision_6_comparator:3 \
    driver_e174_simulated_judge:1 driver_e174_segment_one_judge:1 driver_e174_segment_two_judge:1 driver_e158_r3_seg1_compare:9 driver_e158_r4_compare:66 \
    driver_e165_segment2:1 driver_e165_r3_segment3_root_choice_kill_bounds:1 driver_e165_r4_segment2_root_choice_faults_kill_bounds:1 \
    driver_e165_r5_segment2_r2_mode_kill_bounds:1 driver_e165_r5_segment2_r3_mode_kill_bounds:1 driver_e165_r5_segment2_r4_mode_kill_bounds:1 \
    driver_e165_r5_segment5b_r4_gate_r2_mode_kill_bounds:1; do
    input_driver="${input_case%%:*}"; input_expected="${input_case#*:}"
    checked=$((checked + 1))
    if (cd "$scratch/inputs-empty-root" && OUT_DIR="$scratch/inputs-out" && eval "$input_driver_definitions" && "$input_driver") \
         >"$scratch/inputs.stdout" 2>"$scratch/inputs.stderr"; then input_status=0; else input_status=$?; fi
    input_lines="$(grep -c "^  ✗ 输入不在 test=$input_driver " -- "$scratch/inputs.stderr")"
    if [[ "$input_status" != 1 || "$input_lines" != "$input_expected" || -s "$scratch/inputs.stdout" ]]; then
      echo "  ✗ 自检：$input_driver 在没有 research/、冻结根与导出根都没给的目录里应当退 1、打 $input_expected 行点名它的「✗ 输入不在」、stdout 不打；实际退 $input_status、$input_lines 行，stderr「$(head -c 400 "$scratch/inputs.stderr" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null)」"  # gate-lint:detail
      failures=$((failures + 1))
    fi
  done
  checked=$((checked + 1))
  local registered_default capped_jobs unset_jobs
  registered_default="$(grep -vE '^[[:space:]]*(#|$)' .claude/heavy-thread-default | head -n 1 | awk '{print $1}')"
  unset_jobs="$(env -u SINGLEFS_THREAD_CAP bash -c 'source scripts/hosts/heavy-threads.sh; REPLAY_BROKEN="$1"; '"$(declare -f default_replay_jobs)"'; default_replay_jobs' _ "$REPLAY_BROKEN")"
  capped_jobs="$(SINGLEFS_THREAD_CAP=7 bash -c 'source scripts/hosts/heavy-threads.sh; REPLAY_BROKEN="$1"; '"$(declare -f default_replay_jobs)"'; default_replay_jobs' _ "$REPLAY_BROKEN")"
  if [[ -z "$registered_default" || "$unset_jobs" != "$registered_default" || "$capped_jobs" != 7 ]]; then
    echo "  ✗ 自检：没设 REPLAY_JOBS 时同时跑几条实验要取登记的重活默认（登记「$registered_default」，没设上限得「$unset_jobs」，上限 7 得「$capped_jobs」）"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  # load-commands-not-found：这一趟载入 replay.sh 时执行到的行一个找不到的命令都没有；现调一个不存在的命令，要被记下（证明这一格认得出）
  checked=$((checked + 1))
  if ((${#load_commands_not_found[@]})); then
    echo "  ✗ 自检：载入 replay.sh 时有行找不到命令（多半是一句注释丢了 #）：${load_commands_not_found[*]}；照 bash 打的那几行「line N: … command not found」找到那一行补上 #"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  checked=$((checked + 1))
  replay_selftest_probe_command_that_does_not_exist 2>/dev/null
  local commands_not_found_now
  commands_not_found_now="$(replay_commands_not_found)"
  if ! grep -qx replay_selftest_probe_command_that_does_not_exist <<<"$commands_not_found_now"; then
    echo "  ✗ 自检：现调一个不存在的命令，command_not_found_handle 应当记下它的名字，实际没记：载入期找不到命令的行这一格认不出"  # gate-lint:detail
    failures=$((failures + 1))
  fi
  rm -rf -- "${scratch:?}"
  if ((failures)); then
    echo "  ✗ replay.sh 自检 $failures 处不对（查了 $checked 项）"
    echo "    → 怎么办：看 build_research_binaries、admission_key_of_row 与计时行的负载关（timing_load_decide、load_timing_lag）；「按行复用 <格名>」那几处看 replay_reuse_selftest 里那一格的条件与它打的那一句（不在 git 工作树里那一格看取库名那一段对 replay_row_library_failure 的调用）；REPLAY_BREAK 设着的话这里本来就该红"
    return 1
  fi
  echo "  ✓ replay.sh 自检通过（查了 $checked 项：构建失败返回 2 并打第一处错、日志末尾、日志路径与重现命令，构建成功一个字都不打，驱动开跑前编研究二进制经 --manifest-path 指到研究工作区、编不过的输出进 .err（build_research_driver_binary 与 E162、E163 那三个驱动），指明了准入键的行按它问准入，按行复用接连五趟（头一趟两行都跑并记进判定库、第二趟都复用、改一行装置源码只重跑那一行、只改注释照样复用），开了 GPU 而要现跑的行都不点特性数组时不试编、不要卡，点到的才退 78，计时行负载超阈值记本次不判、没超照判、没字段判红、没字段但在滞后表照判，滞后表的登记错误逐条点名，配置开了 GPU 时 验证包装置带 gpu 特性（开了先试编、没开不起 cargo、配置读不出或试编红退 78 点名原因、起装置的 cargo run 行都带特性数组，开了 GPU 不另套清场、卡读服务交的 GPU_CARDS_BY_PRIORITY（没有退 78），起装置的 cargo run 行都不带清场前缀、本文件不起清场包装），同时跑几条实验的缺省取登记的重活默认，拷 crates/ 去编译的副本都带上每台默认重活线程数的登记文件，留存产物只在 git 历史里的行取回再比（字节一致并写出处）、取不回判「取不回」，装置退 77 打了 NOT_RUN 行的记跑不了、没有本次未跑的一档，读别的实验产物与留存产物的二十五个驱动缺输入时退 1、缺几样报几样，--list-items 只列实验号、表里没有的实验号退 2，三条完整性闸各一格：退出码非 0、收尾行 emitted= 与实收对不上、同一输出目录里上一趟取回的留存产物不压过树里的，编译目录五格：跑绿、编不过、装置开跑之后被 TERM 三种结局之后都不剩这一趟的编译目录，调用方给了 CARGO_TARGET_DIR 照它编且不删，副本编出来的编译目录跑完删、产物照留）"
  return 0
}
if [[ "${1:-}" == "--selftest" ]]; then
  replay_selftest
  exit $?
fi

# 入口脚本的列项与写错（文件头「入口脚本」）：复跑表从本文件原文读（TABLE 那一段在下面），排在建本轮输出目录、truncate 镜像之前，不编、不跑、不留东西。
if [[ "${1:-}" == "--list-items" ]]; then
  if (($# != 1)); then
    echo "  ✗ --list-items 不带别的参数（收到：$*）" >&2
    echo "     → 怎么办：单独写 bash scripts/experiment-apparatus/replay.sh --list-items；要跑哪几个照它列的实验号写：bash scripts/experiment-apparatus/replay.sh <实验号…>" >&2
    exit 2
  fi
  replay_table_experiments
  exit 0
fi
replay_known_experiments="$(replay_table_experiments | cut -f1)"
replay_unknown_experiments=()
for replay_argument in "$@"; do
  case "$replay_argument" in --admission-only) continue ;; esac
  [[ "$replay_argument" =~ ^[0-9]+$ ]] && continue   # 裸数字由下面「没带 E」那一判点名
  grep -qxF -- "$replay_argument" <<<"$replay_known_experiments" || replay_unknown_experiments+=("$replay_argument")
done
if ((${#replay_unknown_experiments[@]})) && [[ "$REPLAY_BROKEN" != unknown-experiment-accepted ]]; then
  echo "  ✗ 复跑表里没有：${replay_unknown_experiments[*]}（一行都没跑；表里 $(grep -c . <<<"$replay_known_experiments") 个实验号）" >&2
  echo "     → 怎么办：照 bash scripts/experiment-apparatus/replay.sh --list-items 列的实验号那一列写（带 E、区分大小写，例 E142、E16R5）；选项只认 --admission-only、--selftest、--list-items" >&2
  exit 2
fi

OUT_DIR="${REPLAY_OUT:-${TMPDIR:-/tmp}/singlefs-replay-$$}"
mkdir -p "$OUT_DIR"

# 编译目录（文件头「编译目录」）：调用方没设 CARGO_TARGET_DIR 时这一趟在 ${TMPDIR:-/tmp} 下建自己的运行目录，cargo 编进它的 target/；收尾的 EXIT 钩子
# replay_finish 先删驱动在本轮输出目录里的副本编出来的编译目录，再删这一份与运行目录。调用方设了的照它编、不删。研究二进制从 $REPLAY_RELEASE_DIRECTORY 起。
source scripts/hosts/run-build-directory.sh
REPLAY_BUILD_RUN_DIRECTORY=""
# 驱动在本轮输出目录（$OUT_DIR）里拷 crates/ 去编的副本编出来的编译目录（文件头「编译目录」）：逐个找带签名对得上的 CACHEDIR.TAG（不跟符号链接、不出这个文件系统），
# 经 scripts/hosts/run_build_directory.py remove 删（删之前核本机没有进程开着里面的文件，有就不删、报出进程，交回 1）；调用方给的 CARGO_TARGET_DIR 落在里面的不碰。
# 弄坏开关 REPLAY_BREAK=copy-build-directories-kept：不删
replay_remove_copy_build_directories() {
  local marker directory output failed=0
  [[ -d "${OUT_DIR:-}" && "$REPLAY_BROKEN" != copy-build-directories-kept ]] || return 0
  while IFS= read -r -d '' marker; do
    directory="${marker%/CACHEDIR.TAG}"
    [[ -d "$directory" && "$(head -c 43 -- "$marker" 2>/dev/null)" == "Signature: 8a477f597d28d172789f06886806bc55" ]] || continue
    [[ -n "${CARGO_TARGET_DIR:-}" && ( "$directory" == "$CARGO_TARGET_DIR" || "$directory" == "$CARGO_TARGET_DIR"/* ) ]] && continue
    if ! output="$(python3 scripts/hosts/run_build_directory.py remove "$directory" 2>&1)"; then
      printf '%s\n' "$output" >&2
      failed=1
    fi
  done < <(find "$OUT_DIR" -xdev -type f -name CACHEDIR.TAG -print0 2>/dev/null)
  return "$failed"
}
# 收尾（EXIT）：正常结束、出错、被停都走到这里。删不成（有进程开着、删完回读还在、运行目录删完编译目录之后不是空的）的，本来要退 0 的改退 1。
# 弄坏开关 REPLAY_BREAK=build-directory-kept：这一趟自己的编译目录与运行目录不删
replay_finish() {
  local status=$? problems=0
  trap - EXIT TERM INT
  # 按行复用的中转目录（文件头「按行复用」）：判定库在 prune-server 里，这一份只给这一趟读写用，收尾删掉，不留给下一趟
  [[ -n "${REPLAY_ROW_RECORDS:-}" ]] && rm -rf -- "$REPLAY_ROW_RECORDS"
  [[ -n "${REPLAY_E182_SHARDS:-}" ]] && rm -rf -- "$REPLAY_E182_SHARDS"   # E182 的片结果库的中转目录（文件头「E182 的片结果库」），并回之后删
  # 驱动开跑前从 git 历史取回、放回 research/results/ 的留存产物（replay_inputs_present）：本趟放回去的，收尾删掉，不留给别的会话看见
  if [[ -n "${OUT_DIR:-}" && -f "$OUT_DIR/restored-archived-products.list" ]]; then
    while IFS= read -r restored_product; do [[ "$restored_product" == research/results/* ]] && rm -f -- "$restored_product"; done <"$OUT_DIR/restored-archived-products.list"
  fi
  replay_remove_copy_build_directories || problems=1
  if [[ -n "$REPLAY_BUILD_RUN_DIRECTORY" && "$REPLAY_BROKEN" == build-directory-kept ]]; then
    echo "  ! 这一趟的编译目录 $REPLAY_BUILD_RUN_DIRECTORY 不删（弄坏开关 build-directory-kept）" >&2
  elif [[ -n "$REPLAY_BUILD_RUN_DIRECTORY" ]]; then
    if ! run_build_directory_end; then
      problems=1
    elif [[ -d "$REPLAY_BUILD_RUN_DIRECTORY" ]] && ! rmdir -- "$REPLAY_BUILD_RUN_DIRECTORY" 2>/dev/null; then
      echo "  ✗ 这一趟的运行目录 $REPLAY_BUILD_RUN_DIRECTORY 删编译目录之后不是空的（或删不成），没删" >&2
      echo "     → 怎么办：ls -la $REPLAY_BUILD_RUN_DIRECTORY 看是谁写进去的；还开着里面文件的进程照上面那几行停掉再手删" >&2
      problems=1
    fi
  fi
  (( problems && status == 0 )) && status=1
  exit "$status"
}
# 被 TERM / INT 停：给自己起的每一行复跑（REPLAY_PIDS 里、/proc 里父进程还是这一趟的那几个）转一次 TERM（它们是 `&` 起的，INT 本来就忽略），
# 逐个等它们退，再退 143 / 130、走 EXIT 收尾。行里手里那一步由 replay_one 自己再转
replay_forward_term() { # <进程号>：只停点名的这一个，而且只在它的父进程还是这一趟时才发（进程号被别的进程复用了不误伤）
  local stat_line parent
  IFS= read -r stat_line 2>/dev/null <"/proc/$1/stat" || return 0
  read -r _ parent _ <<<"${stat_line##*) }"
  if [[ "$parent" == "$$" ]]; then kill -TERM "$1" 2>/dev/null; fi
  return 0
}
replay_stop() { # <退出码>
  local pid
  trap - TERM INT
  for pid in ${REPLAY_PIDS[@]+"${REPLAY_PIDS[@]}"}; do replay_forward_term "$pid"; done
  for pid in ${REPLAY_PIDS[@]+"${REPLAY_PIDS[@]}"}; do wait "$pid" 2>/dev/null; done
  exit "$1"
}
trap replay_finish EXIT
trap 'replay_stop 143' TERM
trap 'replay_stop 130' INT
if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
  [[ "$CARGO_TARGET_DIR" == /* ]] || CARGO_TARGET_DIR="$PWD/$CARGO_TARGET_DIR"
  export CARGO_TARGET_DIR
elif ! REPLAY_BUILD_RUN_DIRECTORY="$(mktemp -d "${TMPDIR:-/tmp}/singlefs-replay-build-$$-XXXXXX")"; then
  REPLAY_BUILD_RUN_DIRECTORY=""
  echo "  ✗ 建不了这一趟的运行目录（mktemp -d 在 ${TMPDIR:-/tmp} 下失败），一条实验都没跑" >&2
  echo "     → 怎么办：看 ${TMPDIR:-/tmp} 在不在、可不可写、盘满没满；或自己设 CARGO_TARGET_DIR 到仓外一个用完自己删的目录" >&2
  exit 2
elif ! run_build_directory_begin "$REPLAY_BUILD_RUN_DIRECTORY"; then
  exit 2
fi
REPLAY_RELEASE_DIRECTORY="$CARGO_TARGET_DIR/release"
export REPLAY_RELEASE_DIRECTORY   # driver_e9 在 bash -c 里起 e9-keylayout，要从环境里拿
# E9 要一个真设备/文件当后端（O_DIRECT）。镜像一律落临时目录，不许进仓
# （command-safety.md「测试镜像一律放临时目录」）。
REPLAY_DEV="${REPLAY_DEV:-$OUT_DIR/e9.img}"
# e9-keylayout 用 O_DIRECT 打开这个后端，自己不建文件。tmpfs 不支持 O_DIRECT，
# 所以 REPLAY_OUT 要落在真文件系统上（本机 /tmp 是 ext4，够用）。
[[ -e "$REPLAY_DEV" ]] || truncate -s 512M "$REPLAY_DEV"
export REPLAY_DEV
# E44 要一个 64 MiB 的 O_DIRECT 后端量本机 fsync 率
REPLAY_DEV45="${REPLAY_DEV45:-$OUT_DIR/e45.img}"
[[ -e "$REPLAY_DEV45" ]] || truncate -s 64M "$REPLAY_DEV45"
export REPLAY_DEV45
# ⚠️ **E58 的测试区不许预先 truncate 出来。** 稀疏洞读起来可能根本不碰设备，
# 而 e58 只看文件大小决定填不填 ⇒ 预创建一个空洞文件会让它去量「读洞要多久」。
# 交给二进制自己建、自己填。真读错了阳性对照会判红（读洞时 read_bytes 是 0）。
REPLAY_DEV58="${REPLAY_DEV58:-$OUT_DIR/e58.img}"
export REPLAY_DEV58
# E140 与 E58 同一套装置（8 GiB O_DIRECT 测试区，二进制自己建、自己填），同样不许预先 truncate。
REPLAY_DEV140="${REPLAY_DEV140:-$OUT_DIR/e140.img}"
export REPLAY_DEV140

# 实验号 | 二进制 | 参数 | 入库产物 | 判据（exact=应逐字节一致 / timing=含计时字段，只比结构）
TABLE=$(cat <<'TSV'
E14|e14-discrimination||e14-discrimination-2026-08-29.out|exact
E18|e18-branch||e18-branch-2026-08-28.out|exact
E19|e19-defer||e19-defer-2026-08-28.out|exact
E23|e23_journal_geom||e23-journal-geom-2026-08-29.out|exact
E24|e24_recovery||e24-recovery-2026-08-29.out|exact
E25|e25_journal_reserve||e25-journal-reserve-2026-08-29.out|exact
E26|e26_accounting||e26-accounting-2026-08-29.out|exact
E27|e27_snapshot_accounting_risk_paths||e27-d5-paths-2026-08-29.out|exact
E28|e28_map_rebuild||e28-map-rebuild-2026-08-29.out|exact
E29|e29_blast_radius||e29-blast-radius-2026-08-29.out|exact
E30|e30_range_rebuild||e30-range-rebuild-2026-08-29.out|exact
E31|e31-aad-snapshot||e31-aad-snapshot-2026-08-29.out|exact
E32|e32-journal-timeline||e32-journal-timeline-2026-08-29.out|exact
E33|e33-pin-rules||e33-pin-rules-2026-08-29.out|exact
E35|e35-head-forms||e35-head-forms-2026-08-29.out|exact
E36|e36-slot-mapping||e36-slot-mapping-2026-08-29.out|exact
E37|e37-log-epoch||e37-log-epoch-2026-08-29.out|exact
E38|e38_accounting_copy_on_write||e38-accounting-cow-2026-08-29.out|exact
E39|e39_back_chain||e39-back-chain-2026-08-29.out|exact
E42|e42_transaction_records||e42-txn-records-2026-08-29.out|exact
E44|e44_jsn_width|$REPLAY_DEV45|e44-jsn-width-2026-08-30.out|timing
E58|e58-csum-grain|$REPLAY_DEV58 1 none|e58-csum-grain-repro-2026-09-16.out|timing
E140|e140-header-alignment|$REPLAY_DEV140 1 none|e140-header-alignment-repro-2026-09-13.out|timing
E43|e43_extension_point_budget||e43-ext-budget-2026-10-04-h487.out|exact
E41|e41_root_ring_geom||e41-root-ring-geom-2026-08-30.out|exact
E71|e71-accounting-keys||e71-accounting-keys-2026-09-01.out|exact
E71|e71-accounting-keys-single-generation||e71-accounting-keys-single-generation-2026-10-20.out|exact
E75|e75-record-size||e75-record-size-2026-09-01.out|exact
E76|e76-payload-checksum||e76-payload-csum-2026-09-01.out|exact
E77|e77-publish-order||e77-publish-order-2026-09-02.out|exact
E78|e78-replay-start||e78-replay-start-2026-09-02.out|exact
E79|e79-root-record||e79-root-record-2026-09-06.out|exact
E112|e112-old-writer-unknown-tree||e112-old-writer-unknown-tree-2026-09-06.out|exact
E113|e113-unknown-tree-full-arms||e113-unknown-tree-full-arms-2026-09-06.out|exact
E114|e114-pack-ledger||e114-pack-ledger-2026-09-12.out|exact
E117|e117-reserved-header||e117-reserved-header-2026-09-12.out|exact
E118|e118-single-disk-recovery||e118-single-disk-recovery-2026-09-07.out|exact
E122|e122-directory-locality||e122-dir-locality-2026-09-07.out|exact
E116|e116-pack-settle||e116-pack-settle-2026-10-04-h487.out|exact
E119|e119-slot-tiers||e119-slot-tiers-2026-09-12.out|exact
E120|e120-tier-ratio||e120-tier-ratio-2026-09-12.out|exact
E121|e121-capacity-tiers||e121-cap-tiers-2026-09-12.out|exact
E115|e115-system-configuration-completeness||e115-system-configuration-completeness-2026-09-07.out|exact
E80|e80-partial-stripe||e80-partial-stripe-2026-09-02.out|exact
E81|e81-commit-fixpoint||e81-commit-fixpoint-2026-09-02.out|exact
E82|e82-admission-overlay||e82-admission-overlay-2026-09-02.out|exact
E83|e83-tombstone-grain||e83-tombstone-grain-2026-10-04-r2.out|exact
E84|e84-tombstone-pinning||e84-tombstone-pinning-2026-09-03.out|exact
E85|e85-unit-header||e85-unit-header-2026-09-02.out|exact
E86|e86-scan-step||e86-scan-step-2026-09-02.out|exact
E87|e87-fixed-placement||e87-fixed-placement-2026-09-02.out|exact
E88|e88-impostor-orphan||e88-impostor-orphan-2026-09-02.out|exact
E89|e89-interval-frontier||e89-interval-frontier-2026-09-03.out|exact
E90|e90-tree-aad||e90-tree-aad-2026-09-03.out|exact
E91|e91-ring-admission||e91-ring-admission-2026-09-03.out|exact
E92|e92-reuse-requirement||e92-reuse-requirement-2026-09-08.out|exact
E123|e123-reuse-window-versus-rollback-depth||e123-k-fork-cost-2026-09-09.out|exact
E124|e124-system-configuration-recompute||e124-system-configuration-recompute-2026-09-09.out|exact
E126|e126-system-configuration-slot-width||e126-system-configuration-slot-width-2026-09-09.out|exact
E127|e127-group-identity-under-split-merge||e127-group-identity-under-split-merge-2026-09-13-knobs.out|exact
E93|e93-aging-placement||e93-aging-placement-2026-09-03.out|exact
E95|e95-node-layout-arms||e95-node-layout-arms-2026-09-09.out|exact
E94|e94-move-touchset||e94-move-touchset-2026-09-03.out|exact
E96|e96-hybrid-consistency||e96-hybrid-consistency-2026-09-03.out|exact
E97|e97-entry-encoding||e97-entry-encoding-2026-09-07.out|exact
E97|e97-entry-encoding-single-generation||e97-entry-encoding-single-generation-2026-10-20.out|exact
E98|e98-inode-record||e98-inode-record-2026-09-07.out|exact
E99|e99-writebuffer-sequence||e99-writebuffer-seq-2026-09-03.out|exact
E100|e100-system-configuration-slot||e100-system-configuration-slot-2026-09-03.out|exact
E101|e101-node-tag-reserve||e101-node-tag-reserve-2026-09-03.out|exact
E102|e102-unit-class-registry||e102-unit-class-registry-2026-10-04.out|exact
E103|e103-inode-update-cost||e103-inode-update-cost-2026-09-14-round2.out|exact
E104|e104-current-version||e104-current-version-2026-09-05.out|exact
E105|e105-extent-leaf-packed||e105-extent-leaf-packed-2026-09-12.out|exact
E106|e106-stripe-member-table||e106-stripe-member-table-2026-09-12.out|exact
E108|e108-plaintext-layer-cost||e108-plaintext-layer-cost-2026-09-06.out|exact
E107|e107-stripe-table-wa||e107-stripe-table-wa-2026-09-06.out|timing
E109|e109-position-authority||e109-position-authority-2026-09-06.out|exact
E110|e110-stripe-table-steady||e110-stripe-table-steady-2026-09-06.out|exact
E111|e111-stripe-table-key||e111-stripe-table-key-2026-09-06.out|exact
E34|e34-ring-iomin||e34-ring-iomin-2026-09-01.out|exact
E73|e73-key-range||e73-key-range-2026-09-07.out|exact
E74|e74-allocation-records||e74-alloc-records-2026-09-01.out|exact
E40|e40_checksum_width||e40-csum-width-2026-08-30.out|exact
E46|e46_region_spacing||e46-region-spacing-2026-08-30.out|exact
E47|e47_ring_loss||e47-ring-loss-2026-08-30.out|exact
E48|e48_ring_placement||e48-ring-placement-2026-08-30.out|exact
E50|e50_ring_slots||e50-ring-slots-2026-08-30.out|exact
E49|e49_chain_width||e49-chain-width-2026-10-04-h487.out|exact
E51|e51_chain_chances||e51-chain-chances-2026-08-30.out|exact
E52|e52_head_mechanisms||e52-head-mechanisms-2026-08-30.out|exact
E54|e54_accounting_generations||e54-accounting-gen-2026-08-30.out|exact
E57|e57_field_authority||e57-field-authority-2026-08-31.out|exact
E59|e59_message_recompute||e59-msg-recompute-2026-08-31.out|exact
E61|e61-chain-hash||e61-chain-hash-2026-08-31.out|exact
E69|e69-backref-cost||e69-backref-cost-2026-08-31.out|exact
E60|e60-rebalance||e60-rebalance-2026-08-31.out|exact
E70|e70-ckpt-thresholds||e70-ckpt-thresholds-2026-08-31.out|exact
E62|e62-ring-home||e62-ring-home-2026-08-31.out|exact
E63|e63-width-rule||e63-width-rule-2026-08-31.out|exact
E67|e67-device-subset||e67-device-subset-2026-08-31.out|exact
E68|e68-inline-threshold||e68-inline-threshold-2026-08-31.out|exact
E8|e8-split||e8-split-2026-08-28.out|exact
E9|@driver_e9||e9-keylayout-2026-08-28.out|exact
E16|e16-journal||e16-journal-2026-08-31.out|exact
E16|e16-journal|bytes|e16-bytes-2026-09-03.out|exact
E16R5|e16-fifth-run-peak-under-today-widths|grid|e16-fifth-run-peak-under-today-widths-2026-11-02-h496.out|exact
E16R5|e16-fifth-run-peak-under-today-widths|pc3|e16-fifth-run-peak-under-today-widths-pc3-2026-11-02-h496.out|exact
E16R5|e16-fifth-run-peak-under-today-widths|second_segment|e16-fifth-run-peak-under-today-widths-second-segment-2026-11-02-h496.out|exact
E17|e17-merge||e17-merge-2026-08-29-repro.out|timing
E20|e20-fanout||e20-poscontrol-2026-08-29.out|timing
E21|e21-cpu|2048 5|e21-cpu-2026-08-28.out|timing
E128|e128-pointer-birth-cost|2000000|e128-pointer-birth-cost-2026-09-10.out|timing
E130|e130_livelist_bounded_destroy||e130-livelist-bounded-destroy-2026-09-10.out|exact
E131|e131_livelist_carrier||e131-livelist-carrier-2026-09-16.out|exact
E132|e132_livelist_carrier_recount||e132-livelist-carrier-recount-2026-09-16.out|exact
E133|e133_map_key_format_cost||e133-map-key-format-cost-2026-09-11.out|exact
E134|e134_map_key_slot_baselines||e134-map-key-slot-baselines-2026-09-11.out|exact
E136|e136_fork_cost_rows||e136-fork-cost-rows-2026-09-11.out|exact
E138|e138_per_disk_floor||e138-per-disk-floor-2026-09-11.out|exact
E139|e139_tightened_floor||e139-tightened-floor-2026-10-29.out|exact
E139|@driver_e139||e139-tightened-floor-2026-10-05.out|exact
E141|e141_switch_reserve_mount_admission||e141-switch-reserve-mount-admission-2026-09-14-row-writing.out|exact
E142|@driver_e142||e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out|exact
E142|@driver_e142_p8g_crates_dump||e142-r25-p8g-crates-dump-2026-10-13.out|exact
E143|e143-one-unit-per-txn-journal||e143-one-unit-per-txn-journal-2026-09-13.out|exact
E145|e145-self-describing-node-header||e145-self-describing-node-header-2026-09-16-tree-table-200.out|exact
E146|e146-livelist-entry-width||e146-livelist-entry-width-2026-09-16-tree-table-200.out|exact
E147|e147-system-configuration-recompute-from-layout||e147-system-configuration-recompute-from-layout-2026-09-13.out|exact
E148|e148-commit-fixpoint-two-record-trees||e148-commit-fixpoint-two-record-trees-2026-09-13.out|exact
E150|e150-rollback-reuse-of-abandoned-roots||e150-rollback-reuse-of-abandoned-roots-2026-09-13-admission.out|exact
E151|e151-arrival-and-container-arms||e151-arrival-and-container-arms-2026-09-13-region.out|exact
E149|e149-pack-container-repair-options||e149-pack-container-repair-options-2026-09-13.out|exact
E144|e144-header-checksum-cost||e144-header-checksum-cost-2026-09-13.out|timing
E135|e135_rollback_floor||e135-rollback-floor-2026-09-11.out|exact
E137|e137_map_key_performance||e137-map-key-performance-2026-09-11.out|exact
E154|e154-two-gates-serial-rejudge-and-reclaim-timing||e154-two-gates-serial-rejudge-and-reclaim-timing-2026-09-24-rename.out|exact
E153|e153-ledger-shape-and-ring-holes||e153-ledger-shape-and-ring-holes-2026-09-24-rename.out|exact
E155|e155-fsync-write-volume||e155-fsync-write-volume-2026-09-25-h311-replay.out|exact
E155R2|e155-second-run-fsync-write-volume||e155-second-run-fsync-write-volume-2026-09-25-h311-replay.out|exact
E155R3|e155-third-run-release-cascade||e155-third-run-release-cascade-2026-09-25-h311-replay.out|exact
E155R4|e155-fourth-run-group-commit-concurrency||e155-fourth-run-group-commit-concurrency-2026-09-25-h311-replay.out|exact
E156|@driver_e156||e156-alloc-basis-counts-2026-10-18.out|exact
E157|e157-parallel-line-one-clauses||e157-parallel-line-one-clauses-2026-10-04-h487.out|exact
E160|e160-random-small-read-share||e160-random-small-read-share-segment1-2026-09-24-realweight.out|exact
E158|@driver_e158||e158-root-choice-repair-2026-09-25-segment1-rerun.out|exact
E158|@driver_e158_q3_1_g0||e158-root-choice-repair-2026-09-25-q3-1-g0.out|exact
E158|@driver_e158_q3_1_s16||e158-root-choice-repair-2026-09-25-q3-1-s16.out|exact
E158|@driver_e158_q3_1_small_ring||e158-root-choice-repair-2026-09-25-q3-1-small-ring.out|exact
E158|@driver_e158_q3_1_s4||e158-root-choice-repair-2026-09-25-q3-1-s4.out|exact
E158|@driver_e158_q1_g0||e158-root-choice-repair-2026-09-25-q1-g0-today.out|exact
E158|@driver_e158_q2_1_g0||e158-root-choice-repair-2026-09-24-q2-1-g0-today.out|exact
E158|@driver_e158_q2_2a_g0||e158-root-choice-repair-2026-09-25-q2-2a-g0-today.out|exact
E158|@driver_e158_q2_1_pc2||e158-root-choice-repair-2026-09-24-pc2-today.out|exact
E158|@driver_e158_q2_1_g0_session_s5||e158-root-choice-repair-2026-09-24-q2-1-g0-today-session-s5.out|exact
E158|@driver_e158_q1_s16||e158-root-choice-repair-2026-09-25-q1-s16.out|exact
E158|@driver_e158_q1_s4||e158-root-choice-repair-2026-09-25-q1-s4.out|exact
E158|@driver_e158_q2_1_hc1||e158-root-choice-repair-2026-09-24-q2-1-hc1.out|exact
E158|@driver_e158_q2_1_hc1_lower_bound||e158-root-choice-repair-2026-09-24-q2-1-hc1-lower-bound.out|exact
E158|@driver_e158_r2_all||e158-root-choice-repair-2026-09-27-r2-all-today.out|exact
E158|@driver_e158_r3_seg1_today||e158-root-choice-repair-2026-09-27-r3-seg1-today.out|exact
E158|@driver_e158_r3_seg1_compare||e158-root-choice-repair-2026-09-27-r3-seg1-compare.out|exact
E158|@driver_e158_r4_compare||e158-root-choice-repair-2026-09-27-r4-compare.out|exact
E181|@driver_e181||e181-landing-check-fixed-point-and-mount-lower-bound-checks.out|timing
E182|@driver_e182||e182-unreadable-abandoned-root-forks-single-device-2026-10-19.out|exact
E182|@driver_e182_units_kept||e182-unreadable-abandoned-root-forks-single-device-units-kept.out|exact
E159|e159-fsync-wait-group-commit|anchors|e159-fsync-wait-group-commit-2026-09-25-h311-replay.out|exact
E162|@driver_e162_anchors||e162-crash-verdict-block-store-2026-09-27-anchors-2026-09-28.out|exact
E162|@driver_e162_power_cut_selftest||e162-verdict-store-power-cut-2026-09-27-selftest-r2.out|exact
E162|@driver_e162_s1_all_arms||e162-crash-verdict-block-store-s1-all-arms.out|timing
E162|@driver_e162_s2_all_arms||e162-crash-verdict-block-store-s2-all-arms.out|timing
E162|@driver_e162_s4_all_arms||e162-verdict-store-power-cut-s4-all-arms.out|timing
E161|@driver_e161_segment_one||e161-crash-state-dedup-and-time-split-segment-one.out|timing
E163|@driver_e163_r1_merge||e163-gpu-multicard-crc32c-2026-09-27-r1-merge.out|exact
E164|e164-crash-amplification-design-function-test|models|e164-crash-amplification-design-function-test-2026-09-28-models.out|exact
E167|e167-multi-inflight-crash-cut|first|e167-multi-inflight-crash-cut-2026-10-01-r2.out|exact
E167|e167-multi-inflight-crash-cut|second|e167-multi-inflight-crash-cut-second-2026-10-01-r3.out|exact
E167|e167-multi-inflight-crash-cut|third|e167-multi-inflight-crash-cut-third-2026-10-01.out|exact
E168|@driver_e168_s1_history||e168-delete-ring-window-borrow-s1-history-2026-10-03.out|exact
E168|@driver_e168_positive_controls||e168-delete-ring-window-borrow-positive-controls-2026-10-03.out|exact
E173|e173-admission-ceiling-bad-ring-slot-dispositions|checks|e173-admission-ceiling-bad-ring-slot-dispositions-checks-2026-10-05.out|exact
E173|e173-admission-ceiling-bad-ring-slot-dispositions|first|e173-admission-ceiling-bad-ring-slot-dispositions-segment-one-2026-10-05.out|exact
E173|e173-admission-ceiling-bad-ring-slot-dispositions|second|e173-admission-ceiling-bad-ring-slot-dispositions-segment-two-2026-10-05.out|exact
E173|@driver_e173_checkpoint_cost||e173-checkpoint-cost-against-empty-publish-cost-2026-11-02.out|exact
E173|e173-admission-ceiling-bad-ring-slot-dispositions|checkpoint-cost-reserve|e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out|exact
E174|@driver_e174_checks||e174-lease-drop-wait-memory-return-checks-2026-10-08.out|exact
E174|@driver_e174_simulated_judge||e174-lease-drop-wait-memory-return-simulated-second-judge-2026-10-08.out|exact
E174|@driver_e174_segment_one_judge||e174-lease-drop-wait-memory-return-segment-one-judge-2026-10-08.out|exact
E174|@driver_e174_segment_two_judge||e174-lease-drop-wait-memory-return-segment-two-judge-2026-10-08.out|exact
E174|@driver_e174_run_segment_one||e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out|timing
E174|@driver_e174_run_segment_two||e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out|timing
E176|e176-inode-leaf-merge-capacity-and-rewrites|segment-one|e176-inode-leaf-merge-capacity-and-rewrites-segment-one.out|exact
E176|e176-inode-leaf-merge-capacity-and-rewrites|segment-two|e176-inode-leaf-merge-capacity-and-rewrites-segment-two.out|exact
E177|e177-per-tree-resurrection-without-tombstones|segment-one|e177-per-tree-resurrection-without-tombstones-segment-one.out|exact
E178|e178-leaf-and-record-death-counts|segment-one|e178-leaf-and-record-death-counts-segment-one.out|exact
E178|e178-leaf-and-record-death-counts|segment-two|e178-leaf-and-record-death-counts-segment-two.out|exact
E178|e178-leaf-and-record-death-counts|segment-three|e178-leaf-and-record-death-counts-segment-three.out|exact
E178|@driver_e178_segment_four||e178-leaf-and-record-death-counts-segment-four.out|exact
E165|@driver_e165||e165-replay-bounded-enumeration-2026-10-01-segment1-legal-family.out|exact
E165|@driver_e165_adversarial||e165-replay-bounded-enumeration-2026-10-01-segment1-adversarial-family.out|exact
E165|@driver_e165_segment2||e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds.out|exact
E165|@driver_e165_r2_segment3_multi_disk||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-multi-disk.out|exact
E165|@driver_e165_r2_segment4_table_both_legal||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-legal-family.out|exact
E165|@driver_e165_r2_segment4_table_both_adversarial||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-adversarial-family.out|exact
E165|@driver_e165_r2_segment4_table_both_multi_disk||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-table-both-multi-disk.out|exact
E165|@driver_e165_r2_segment4_reversed_disk_order_legal||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-legal-family.out|exact
E165|@driver_e165_r2_segment4_reversed_disk_order_multi_disk||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r2-reversed-disk-order-multi-disk.out|exact
E165|@driver_e165_r3_segment1_adversarial||e165-replay-bounded-enumeration-2026-10-04-r5-segment1-r3-adversarial-family.out|exact
E165|@driver_e165_r6_r3_legal||e165-replay-bounded-enumeration-2026-10-09-r6-r3-legal-family.out|exact
E165|@driver_e165_r3_segment2_root_choice||e165-replay-bounded-enumeration-2026-10-23-r7-r3-root-choice.out|exact
E165|@driver_e165_r3_segment3_root_choice_kill_bounds||e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out|exact
E165|@driver_e165_r4_segment1_root_choice_faults||e165-replay-bounded-enumeration-2026-10-23-r7-r4-root-choice-faults.out|exact
E165|@driver_e165_r4_segment2_root_choice_faults_kill_bounds||e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out|exact
E165|@driver_e165_r5_segment2_r2_mode_kill_bounds||e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-kill-bounds.out|exact
E165|@driver_e165_r5_segment2_r3_mode_kill_bounds||e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r3-mode-kill-bounds.out|exact
E165|@driver_e165_r5_segment2_r4_mode_kill_bounds||e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r4-mode-kill-bounds.out|exact
E165|@driver_e165_r5_segment3_legal||e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-legal-family.out|exact
E165|@driver_e165_r5_segment3_adversarial||e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-adversarial-family.out|exact
E165|@driver_e165_r5_segment3_multi_disk||e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-multi-disk.out|exact
E165|@driver_e165_r5_segment3_odd_marker_legal||e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-legal-family.out|exact
E165|@driver_e165_r5_segment3_odd_marker_adversarial||e165-replay-bounded-enumeration-2026-10-04-r5-segment3-r5-odd-marker-adversarial-family.out|exact
E165|@driver_e165_r5_segment5_r2_gate_root_choice_faults||e165-replay-bounded-enumeration-2026-10-23-r7-r5-gate-r2-root-choice-faults.out|exact
E165|@driver_e165_r5_segment5b_r4_gate_r2_mode_kill_bounds||e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds.out|exact
E166|e166-parallel-commit-shared-state-partition||e166-parallel-commit-shared-state-partition-r2-main-2026-10-02.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery|segment-one|e169-parallel-commit-append-and-prefix-recovery-2026-10-01-r3.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery|segment-two|e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-10-01-r2.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery|segment-three|e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-10-01.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery|replay-core-inputs|e169-parallel-commit-append-and-prefix-recovery-replay-core-inputs-2026-10-01.out|exact
E170|@driver_e170_r2_dump_compare||e170-r2-o3-spec-executor-2026-10-05-s2b-dump-compare.out|exact
E170|@driver_e170_r2_bad_states_second_run_export||e170-r2-o3-spec-executor-2026-10-05-s2b-bad-states-second-run-export.out|exact
E170|@driver_e170_r2_v2_487_unit_tests||e170-r2-o3-spec-executor-2026-10-04-s1a-v2-487-unit-tests-2026-10-05.out|exact
E170|@driver_e170_r2_v1_487_unit_tests||e170-r2-o3-spec-executor-2026-10-04-s1a-v1-487-unit-tests-2026-10-05.out|exact
E170|@driver_e170_r2_v2_487_unit_level_mutations||e170-r2-o3-spec-executor-2026-10-04-s1a-v2-487-unit-level-mutations-2026-10-05.out|exact
E170|@driver_e170_r2_v1_487_unit_level_mutations||e170-r2-o3-spec-executor-2026-10-04-s1a-v1-487-unit-level-mutations-2026-10-05.out|exact
E170|@driver_e170_r2_v2_487_z1||e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z1-2026-10-05.out|exact
E170|@driver_e170_r2_v2_487_z3_images||e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z3-images-2026-10-05.out|exact
E170|@driver_e170_r2_v2_487_z4_images||e170-r2-o3-spec-executor-2026-10-04-s2-q5-v2-487-z4-images-2026-10-05.out|exact
E171|@driver_e171_s1_baseline||e171-three-oracle-collusion-2026-10-03-s1-baseline-run1.out|exact
E171|@driver_e171_s1_known_bad_images||e171-three-oracle-collusion-2026-10-03-s1-known-bad-images.out|exact
E171|@driver_e171_s2_bad_states_export||e171-three-oracle-collusion-2026-10-03-s2-bad-states-export.out|exact
E171|@driver_e171_s2_known_bad_images||e171-three-oracle-collusion-2026-10-03-s2-known-bad-images.out|exact
E171|@driver_e171_s2_images_o3||e171-three-oracle-collusion-2026-10-03-s2-images-o3.out|exact
E171|@driver_e171_s2_o3_control_baseline||e171-three-oracle-collusion-2026-10-03-s2-o3-control-baseline.out|exact
E171|@driver_e171_s2_comparator||e171-three-oracle-collusion-2026-10-03-s2-comparator.out|exact
E171|@driver_e171_s3_comparator_mutations||e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7.out|exact
E171|@driver_e171_s4_comparator||e171-three-oracle-collusion-2026-10-03-s4-comparator.out|exact
E171|@driver_e171_s4_f4_comparator||e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out|exact
E171|@driver_e171_s4_f4_substitutes_comparator||e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out|exact
E171|@driver_e171_r1_s1_revision_6_comparator||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out|exact
E171|@driver_e171_r1_s1_revision_6_cross_host_check||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out|exact
E171|@driver_e171_r1_s1_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r3.out|exact
E171|@driver_e171_r1_s2_revision_6_comparator||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out|exact
E171|@driver_e171_r1_s2_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-research-mutations-r6.out|exact
E171|@driver_e171_r1_s1_revision_8_tools_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s1-revision-8-tools-research-mutations-2026-11-02-r3.out|exact
E171|@driver_e171_r1_s2_revision_10_tools_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-s2-revision-10-tools-research-mutations.out|exact
E171|@driver_e171_r1_revision_11_tools_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-revision-11-tools-research-mutations.out|exact
E171|@driver_e171_r1_revision_12_stream_constants||e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants-r2.out|exact
E171|@driver_e171_second_rerun_list_check||e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-check-2026-11-01.out|exact
E171|@driver_e171_second_rerun_removal_effect_on_the_whole_table_and_expansion_table||e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-s4-f4-2026-11-01.out|exact
E171|@driver_e171_second_rerun_removal_effect_on_the_whole_table_and_expansion_table_substitutes||e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-s4-f4-substitutes-2026-11-01.out|exact
E171|@driver_e171_second_rerun_removal_effect_on_the_revision_6_segment_two||e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-removal-effect-on-the-revision-6-segment-two-2026-11-01.out|exact
E171|@driver_e171_second_rerun_list_removal_research_mutations||e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-removal-research-mutations.out|exact
E171|@driver_e171_second_rerun_current_state_preconditions_research_mutations||e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-preconditions-research-mutations.out|exact
E171|@driver_e171_second_rerun_current_state_library_root_research_mutations||e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-library-root-research-mutations-2026-11-02.out|exact
E171|@driver_e171_second_rerun_current_state_announcement_research_mutations||e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-announcement-research-mutations.out|exact
E171|@driver_e171_second_rerun_current_state_threads_and_contrast_research_mutations||e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-02-current-state-threads-and-contrast-research-mutations-2026-11-02.out|exact
E172|@driver_e172||e172-r3-deletion-write-dry-run-2026-11-02.out|exact
E175|@driver_e175||e175-product-path-inode-watermark-regression-2026-10-08-d-P-replay.out|exact
E183|@driver_e183||e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out|exact
E183|@driver_e183_anchor_check||e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30-anchor-check.out|exact
E184|@driver_e184||e184-regular-layer-cost-counts-selfcheck-2026-10-23.out|exact
E184|@driver_e184_segment_one||e184-regular-layer-cost-counts-segment-one-2026-10-24.out|timing
E183|@driver_e183_h5_anchor_check||e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19-anchor-check.out|exact
E186|e186-unknown-tree-accounting-rederive||e186-unknown-tree-accounting-rederive.out.gz|exact
E186|e186-unknown-tree-accounting-rederive|--judgement dangling-u5|e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz|exact
E187|e187-quarantine-mark-cost|segment-one|e187-quarantine-mark-cost-segment-one-2026-10-21.out|exact
E187|e187-quarantine-mark-cost|segment-two|e187-quarantine-mark-cost-segment-two-2026-10-22.out|exact
E198|@driver_e198||e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-02.out|exact
E198|@driver_e198_segment_two||e198-instance-table-pointer-birth-skip-premise-segment-two.out|exact
E199|e199-transaction-output-copy||e199-transaction-output-copy-2026-11-05-r2.out|exact
E205|@driver_e205||e205-c971-inverted-first-txg-arm-costs-six-cells.out|exact
E205|@driver_e205_torn_rotation||e205-c971-inverted-first-txg-arm-costs-torn-rotation.out|exact
E205|@driver_e205_first_txg_skips_s8||e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s8.out|exact
E205|@driver_e205_first_txg_skips_s4_s16||e205-c971-inverted-first-txg-arm-costs-first-txg-skips-s4-s16.out|exact
TSV
)

# 计时字段：换机器、换负载就会变，比对时抹掉。抹掉的是**值**不是**字段名**——
# 字段整个消失属于结构变化，仍然会被抓。
# ⚠️ **判决字段一律不抹**：E128 那几行里 `verdict=` 与 `rounds_jia_slower=` 是结论不是计时，
# 留着逐字比 ⇒ 结论翻向会被这一步当场抓住，不用等下面的区间断言。
# 准入写在产物最前面的几行（`E7INPUT` 开头：输入指纹、强制重跑的理由）只说「这一趟是按什么输入跑的」，不是结果：
# 每次输入一变它就变，老产物还没有它。逐字节比对之前两边都删掉；闸 2 只数 E7RESULT 行，本来就不数它们。
strip_admission_header() {
  sed '/^E7INPUT /d'
}

strip_timing() {
  sed -E 's/(per_sec_milli|median_per_sec_milli|spread_bp|min|max|ratio_bp|years_at_sync1|years_at_sync8|sync1_per_sec_milli|nosync_per_sec_milli|elapsed_ns|verify_ns|ns_per_op|ns_per_lookup|lookups_per_s|ns_small|ns_big|bing_median|jia_median|ratio_median|ratio_min|ratio_max|e128_median|deviation|ratio|one_shot_ns|two_phase_ns|two_phase_plus_5ms_ns|injected_recovered_ns|spread_one_shot|spread_two_phase|per_round_ns|best_ns|t1_ns|t16_ns|mibs|mib_per_s|entries_per_s|gbps|peak_gbps|speedup|threads16_speedup|dev|secs|copy_ns|r_rand_qd1|r_seq|random_ns_per_byte_padded|random_ns_per_byte_h133|seq_ns_per_byte_padded|seq_ns_per_byte_h133|crossover_random_share)=[^ ]*/\1=X/g'
}

# ── 结论区间断言 ──────────────────────────────────────────────────────────
# 计时实验复跑不出同样的字节，抹掉计时之后的「结构一致」又几乎什么都不证明——
# 三条臂一起漂到别处，结构照样一致（test-discipline.md「只让多条臂互相比，
# 测不出所有臂一起错」）。所以每个计时实验都要有一条把 kb 里那个数钉住的断言。
# 断言不中**不等于代码坏了**，也可能是 kb 里那个区间该改了——两种都要人来判，所以判红。
claim() { # claim <实验> <说的是什么> <实测值> <下界> <上界>
  local exp="$1" what="$2" got="$3" lo="$4" hi="$5"
  if [[ -z "$got" ]]; then
    printf '  ✗ %-5s %-46s 读不到这个值\n' "$exp" "$what"; return 1  # gate-lint:detail
  fi
  if awk -v g="$got" -v l="$lo" -v h="$hi" 'BEGIN{exit !(g>=l && g<=h)}'; then
    printf '  ✓ %-5s %-46s %s（kb 记的区间 %s–%s）\n' "$exp" "$what" "$got" "$lo" "$hi"; return 0
  fi
  printf '  ✗ %-5s %-46s %s 落在 kb 记的 %s–%s 之外\n' "$exp" "$what" "$got" "$lo" "$hi"; return 1  # gate-lint:detail
}
fld() { sed -n "s/.*$2=\([0-9.]*\).*/\1/p" "$1" | tail -1; }   # 取某行最后一个匹配字段

check_claims() {
  local exp="$1" f="$2" bad=0 v w x y
  case "$exp" in
  E17)
    # kb 记的是 29.9–31.8M 条目/秒（八轮独立运行）。
    # 留 1% 余量给机器状态波动，超出就是该改 kb 那个区间了。
    # 下界 29636000 按八轮读数的并集定（最低一轮 29.94M），不是为了让门禁变绿：
    # 同一批读数里并行加速与阳性对照一起偏低，
    # 而装置不记录跑时的机器负载，「漂移还是被别的负载挤」分不开，
    # 账在 C350（计时实验不记录跑时的机器负载）。
    v=$(grep 'arm=single' "$f" | sed -n 's/.*entries_per_s=\([0-9]*\).*/\1/p')
    claim E17 "单线程合并吞吐（条目/秒）" "$v" 29636000 32118000 || bad=1
    # 散射是并行度天花板：阳性对照 32 线程必须明显快过合并臂 32 线程，
    # 否则「散射吃掉 55%」这条读数没有判别力。
    w=$(grep 'arm=parallel threads=32' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    x=$(grep 'name=poscontrol threads=32' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    if awk -v a="$x" -v b="$w" 'BEGIN{exit !(a > b*1.5)}'; then
      printf '  ✓ %-5s %-46s 对照 %s× vs 合并臂 %s×\n' E17 "散射吃掉的那一半仍在（阳性对照更快）" "$x" "$w"
    else
      printf '  ✗ %-5s %-46s 对照 %s× 没比合并臂 %s× 快出 1.5 倍\n' E17 "阳性对照失去判别力" "$x" "$w"; bad=1  # gate-lint:detail
    fi ;;
  E20)
    # kb 的承重结论：超 L3 那一档 16 KiB 是唯一最小点，且 8 KiB 反常地差（五轮稳定，未解释）。
    for n in 2048 4096 8192 16384 32768 65536; do
      eval "v$n=\$(grep \"name=e20 node_bytes=$n keys=8388608 entry_bytes=40 \" '$f' | sed -n 's/.*ns_per_lookup=\\([0-9.]*\\).*/\\1/p')"
    done
    if awk -v a="$v16384" -v b="$v2048" -v c="$v4096" -v d="$v8192" -v e="$v32768" -v g="$v65536" \
         'BEGIN{exit !(a>0 && a<b && a<c && a<d && a<e && a<g)}'; then
      printf '  ✓ %-5s %-46s 16K=%s ns，其余 %s/%s/%s/%s/%s\n' E20 "超 L3 档 16 KiB 仍是唯一最小点" "$v16384" "$v2048" "$v4096" "$v8192" "$v32768" "$v65536"
    else
      printf '  ✗ %-5s %-46s 16K=%s，2K/4K/8K/32K/64K=%s/%s/%s/%s/%s\n' E20 "16 KiB 不再是最小点 ⇒ kb 那条要改" "$v16384" "$v2048" "$v4096" "$v8192" "$v32768" "$v65536"; bad=1  # gate-lint:detail
    fi
    if awk -v d="$v8192" -v c="$v4096" 'BEGIN{exit !(d>c)}'; then
      printf '  ✓ %-5s %-46s 8K=%s > 4K=%s\n' E20 "8 KiB 的未解释拐点又复现一次" "$v8192" "$v4096"
    else
      printf '  ✗ %-5s %-46s 8K=%s ≤ 4K=%s ⇒ kb 记的「五轮稳定」不再成立\n' E20 "8 KiB 拐点这次没出现" "$v8192" "$v4096"; bad=1  # gate-lint:detail
    fi ;;
  E144)
    # kb 记的是本机软件实现在 105 字节头上的纳秒数：crc32c 28、sha256 676（7 轮取最小）。留 ±30% 给机器状态波动，
    # 超出就是该改 kb 那个区间了；比值那一行由这两个数夹住，不另钉。
    v=$(grep 'name=cost arm=crc32c width=105 ' "$f" | sed -n 's/.*ns_per_op=\([0-9]*\).*/\1/p')
    claim E144 "CRC32C 算一个 105 字节头（ns）" "$v" 19 37 || bad=1
    w=$(grep 'name=cost arm=sha256 width=105 ' "$f" | sed -n 's/.*ns_per_op=\([0-9]*\).*/\1/p')
    claim E144 "SHA-256 算一个 105 字节头（ns）" "$w" 473 879 || bad=1
    # 判别力那一半是确定性的：阳性对照必须还漏得出双比特翻转，正式臂一个都不许漏。
    x=$(grep 'name=verdict' "$f" | sed -n 's/.*control_has_teeth=\([a-z]*\).*/\1/p')
    y=$(grep 'name=verdict' "$f" | sed -n 's/.*formal_double_bit_missed=\([0-9]*\).*/\1/p')
    if [[ "$x" == true && "$y" == 0 ]]; then
      printf '  ✓ %-5s %-46s 对照漏得出、正式臂零漏\n' E144 "判别力测试仍分得出差别"
    else
      printf '  ✗ %-5s %-46s control_has_teeth=%s formal_double_bit_missed=%s\n' E144 "判别力测试失去判别力" "$x" "$y"; bad=1  # gate-lint:detail
    fi ;;
  E128)
    # kb 的承重结论有两条，方向相反，所以两条都要钉——只钉一条会让「甲不慢」被读成
    # 「甲哪儿都不慢」，而记账那一对是慢的。
    ok67=$(grep 'name=xdev entry_bytes=67 ' "$f" | sed -n 's/.*ok=\([a-z]*\).*/\1/p')
    ok111=$(grep 'name=xdev entry_bytes=111 ' "$f" | sed -n 's/.*ok=\([a-z]*\).*/\1/p')
    if [[ "$ok67" == true && "$ok111" == true ]]; then
      printf '  ✓ %-5s %-46s 67 与 111 两档都落回 E20 的数\n' E128 "跨装置闸仍然成立"
    else
      printf '  ✗ %-5s %-46s ok67=%s ok111=%s\n' E128 "跨装置闸破了：这套装置与 E20 报不同的数" "$ok67" "$ok111"; bad=1  # gate-lint:detail
    fi
    ri=$(grep 'name=verdict pair=inode ' "$f" | sed -n 's/.*ratio_median=\([0-9.]*\).*/\1/p')
    rl=$(grep 'name=verdict pair=ledger ' "$f" | sed -n 's/.*ratio_median=\([0-9.]*\).*/\1/p')
    # ⚠️ **inode 那一对不钉方向。** 四整轮里三轮判 `0/5 jia_never_slower`、
    # 另一轮判 `5/5 jia_slower_every_round`，方向相反 ⇒ kb 记的是「不稳定」，没有方向可钉。
    # 钉任何一边都是把一条不稳定的观测写成断言。这里只把观测值打出来，不判绿也不判红。
    # 真正兜着它的是上面那一步**逐字节结构比对**：`strip_timing` 有意不抹 `verdict=` 与
    # `rounds_jia_slower=`（它们是结论不是计时）⇒ 判决再翻一次，那一步就判红。
    # ⚠️ **E128 因此会间歇性判红，这是有意的。** 判红时该做的不是调宽容差，
    # 是把新跑那一次的比值与判决添进 E128 正文「结论一」那张逐轮表，让它变成五轮、六轮。
    # 每一次红都是一次新观测——对一格记着「不稳定」的实验，这正是复跑该有的行为。
    printf '  ! %-5s %-46s 109/93 = %s（不判，kb 记「不稳定」）\n' E128 "inode 那一对四轮里翻过一次向" "$ri"
    if awk -v r="$rl" 'BEGIN{exit !(r>1.00 && r<1.10)}'; then
      printf '  ✓ %-5s %-46s 97/81 = %s\n' E128 "记账树上甲仍慢一点点，量级不变" "$rl"
    else
      printf '  ✗ %-5s %-46s 97/81 = %s 掉出 (1.00, 1.10)\n' E128 "记账树那一对的方向或量级变了" "$rl"; bad=1  # gate-lint:detail
    fi ;;
  E44)
    # 本机 fsync 率：换机器会变，但**量级**要稳住，否则寿命折算整个塌掉
    v=$(grep 'name=arm arm=Sync1' "$f" | sed -n 's/.*median_per_sec_milli=\([0-9]*\).*/\1/p')
    claim E44 "本机 fsync 率（每秒千分之一次）" "$v" 500000 20000000 || bad=1
    # 阳性对照：不 fsync 必须至少快一倍，否则 fdatasync 没到设备
    w=$(grep 'name=poscontrol' "$f" | sed -n 's/.*ok=\([a-z]*\).*/\1/p')
    if [[ "$w" == true ]]; then printf '  ✓ %-5s %-46s\n' E44 "阳性对照：fdatasync 确实到了设备"
    else printf '  ✗ %-5s %-46s ok=%s\n' E44 "阳性对照失败 ⇒ fdatasync 没到设备，整轮作废" "$w"; bad=1; fi
    # 48 位计数器在本机速率下的寿命：这是「48 位够不够」那条结论的落点
    x=$(grep 'name=lifetime bits=48' "$f" | sed -n 's/.*years_at_sync1=\([0-9]*\).*/\1/p')
    claim E44 "48 位计数器在本机撑多少年" "$x" 500 50000 || bad=1
    # 加宽 jsn 到 12 字节的代价：0..=100 项里一格都不该多占
    y=$(grep 'name=width unit=512 jsn_bytes=12 ' "$f" | sed -n 's/.*cost_unit_count_0_100=\([0-9]*\).*/\1/p')
    claim E44 "jsn 8→12 在 512 单元下多占几格" "$y" 0 0 || bad=1 ;;
  E58)
    # kb 的承重结论：32 KiB 在随机小读上比 16 KiB 贵约一成，而顺序侧只快 0.83%。
    # 只钉倍数不够（三条臂一起漂，倍数照样对），所以第三条钉的是绝对值。
    v=$(grep 'name=rand_g16384 ' "$f" | sed -n 's/.*ns_per_op=\([0-9.]*\).*/\1/p')
    w=$(grep 'name=rand_g32768 ' "$f" | sed -n 's/.*ns_per_op=\([0-9.]*\).*/\1/p')
    x=$(awk -v a="$w" -v b="$v" 'BEGIN{if(b>0) printf "%.4f", a/b}')
    claim E58 "随机小读 QD=1：32 KiB 是 16 KiB 的几倍" "$x" 1.03 1.25 || bad=1
    v=$(grep 'name=randq_g16384 ' "$f" | sed -n 's/.*user_mib_per_s=\([0-9.]*\).*/\1/p')
    w=$(grep 'name=randq_g32768 ' "$f" | sed -n 's/.*user_mib_per_s=\([0-9.]*\).*/\1/p')
    x=$(awk -v a="$v" -v b="$w" 'BEGIN{if(b>0) printf "%.4f", a/b}')
    claim E58 "QD=16 用户带宽：16 KiB 是 32 KiB 的几倍" "$x" 1.03 1.25 || bad=1
    # 绝对值：32 KiB 单元读 4 KiB 的放大恰为 8，由独立算术给出（32768/4096）
    x=$(grep 'name=meta_g32768 ' "$f" | sed -n 's/.*read_amp_4k=\([0-9.]*\).*/\1/p')
    claim E58 "32 KiB 单元的读放大（绝对值）" "$x" 8.0 8.0 || bad=1
    # 阳性对照：内核记的字节 ÷ ops×G，读到洞或读到缓存都会让它塌
    x=$(grep 'name=rand_g32768 ' "$f" | sed -n 's/.*pr_over_devbytes=\([0-9.]*\).*/\1/p')
    claim E58 "阳性对照：内核记的字节 ÷ (ops×G)" "$x" 0.98 1.02 || bad=1 ;;
  E140)
    # kb 的承重结论：含头随机页读比补齐慢 7.7%–9.1%，顺序读补齐少 7.6%–9.2% 带宽。复跑用的是种子 1、装置默认 ops 8192 的单轮，
    # 抽样比 kb 记的五个种子那一轮少，区间按那五个种子的极差再各放 5 个百分点。
    v=$(grep 'name=verdict' "$f" | sed -n 's/.*r_rand_qd1=\([0-9.]*\).*/\1/p')
    claim E140 "随机 4 KiB 页读：含头是补齐的几倍" "$v" 1.03 1.15 || bad=1
    w=$(grep 'name=verdict' "$f" | sed -n 's/.* r_seq=\([0-9.]*\).*/\1/p')
    claim E140 "顺序读：补齐带宽是含头的几倍" "$w" 0.85 0.97 || bad=1
    # 绝对值：跨单元页占比由闭式 (4096 − gcd) / 32635 独立算出，与计时无关
    x=$(grep 'name=model_h133 ' "$f" | sed -n 's/.*straddle_fraction=\([0-9.]*\).*/\1/p')
    claim E140 "头 133 时跨单元页占比（闭式）" "$x" 0.125479 0.125479 || bad=1
    # 阳性对照：内核记的字节 ÷ 程序记账，读到缓存就塌（阴性对照那份产物里同一格是 0.0000）
    y=$(grep 'name=rand_h133 ' "$f" | sed -n 's/.*pr_over_devbytes=\([0-9.]*\).*/\1/p')
    claim E140 "阳性对照：内核记的字节 ÷ 程序记账" "$y" 0.98 1.02 || bad=1 ;;
  E107)
    # kb 的承重结论有两条，一条是字节、一条是延迟，两条都要钉。
    # 字节那条是纯算术、逐次相同，但仍然钉住——只钉延迟会让一个把字节模型改错的变异照样绿。
    v=$(grep 'name=bytes leaves=8 f=0 ' "$f" | sed -n 's/.*ratio_a_over_c=\([0-9.]*\).*/\1/p')
    claim E107 "主负载 8 叶 f=0：甲臂 ÷ 丙臂" "$v" 0.9503 0.9503 || bad=1
    v=$(grep 'name=crossover' "$f" | sed -n 's/.*first_leaves_where_arm_a_cheaper=\([0-9-]*\).*/\1/p')
    claim E107 "甲臂第一次比丙臂便宜的叶数" "$v" 8 8 || bad=1
    # 延迟那条：durable 语义下三段写序比一次提交慢多少倍
    a=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*one_shot_ns=\([0-9]*\).*/\1/p')
    b=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*two_phase_ns=\([0-9]*\).*/\1/p')
    x=$(awk -v a="$a" -v b="$b" 'BEGIN{if(a>0) printf "%.4f", b/a}')
    claim E107 "durable：三段写序 ÷ 一次提交" "$x" 1.15 2.10 || bad=1
    # 阳性对照：注入的 5 ms 必须被量回来，落在 ±10% 内
    x=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*injected_recovered_ns=\([0-9-]*\).*/\1/p')
    claim E107 "阳性对照：注入 5 ms 回收到的纳秒" "$x" 4500000 5500000 || bad=1
    # 作废条款 3：不 O_DIRECT 不 fdatasync 那条必须快一个数量级，否则 I/O 没真落盘
    a=$(grep 'name=latency semantics=durable' "$f" | sed -n 's/.*one_shot_ns=\([0-9]*\).*/\1/p')
    b=$(grep 'name=nosync_control' "$f" | sed -n 's/.*per_round_ns=\([0-9]*\).*/\1/p')
    x=$(awk -v a="$a" -v b="$b" 'BEGIN{if(b>0) printf "%.2f", a/b}')
    claim E107 "作废条款 3：durable ÷ nosync" "$x" 10 500 || bad=1 ;;
  E21)
    # kb 的承重结论：CPU 扫描撞内存带宽墙（约 65 GB/s），16 线程几乎不加速 ⇒ GPU 传输地板已经更慢。
    v=$(grep 'name=scaling arm=bandwidth' "$f" | sed -n 's/.*peak_gbps=\([0-9.]*\).*/\1/p')
    claim E21 "CPU 扫描峰值带宽（GB/s）" "$v" 55 75 || bad=1
    w=$(grep 'name=scaling arm=bandwidth' "$f" | sed -n 's/.*threads16_speedup=\([0-9.]*\).*/\1/p')
    claim E21 "带宽受限：16 线程几乎不加速" "$w" 1.0 1.4 || bad=1
    # 阳性对照：同样 16 线程，计算受限的那条必须大幅加速，否则「不加速」分不清是带宽墙还是没跑起来
    x=$(grep 'name=poscontrol arm=compute' "$f" | sed -n 's/.*speedup=\([0-9.]*\).*/\1/p')
    claim E21 "阳性对照（计算受限）16 线程加速" "$x" 10 20 || bad=1 ;;
  esac
  return $bad
}

# E9 的入库产物是 25 次运行拼起来的（5 种子 × 5 改名档），而这个循环从没被写进 kb。
# 按产物里的 config 行反推出来，重建结果与入库产物**逐字节一致**。
# 25 次调用整条包一层（不是每次调用各包一层）：省掉 24 次多余的 systemd scope 建立。
driver_e9() {
  bash -c '
    device="$1"
    for r in 0 500 2000 5000 20000; do
      for s in 3 7 11 13 17; do
        "$REPLAY_RELEASE_DIRECTORY"/e9-keylayout "$device" "$s" interleave 8 "$r" || exit 1
      done
    done
  ' _ "$REPLAY_DEV" || return 1
}

# E139（按盘回退下界的收严形态） 第 3 次重跑（重跑登记 `research/prompts/e139-r3-prereg.md` 第六节第 2 行「旧行回归」、第十一节比对对象表 #1）：
# 比对对象是第 2 次产物 e139-tightened-floor-2026-10-05.out。今天的装置不带参数跑一遍，删掉第 3 次新加的 core_gate_*、verdict_core_gate_* 行与
# name=done 行，按删剩的行数补回 name=done，要与第 2 次产物逐字节相同（在第 3 次产物 e139-tightened-floor-2026-10-29.out 上照这个过滤量过：
# sha256 f4c079adc75abcd8c44765c160088c0a2650746a05619af8c07ba56662575135，与第 2 次产物相同）。第 3 次产物整份由复跑表里 E139 的装置行 exact 比。
# E139 没有单元表：单元名是这个函数里的字符串字面量，登记在 .claude/gate.d/replay-comparison-units.tsv；第 2 次产物归档出工作区时这一行从 git 历史取回再比。
driver_e139() {
  local unit="第 2 次重跑那一族与更早的全部行"
  echo "replay: driver_e139 跑单元「$unit」：装置不带参数跑一遍，删掉第 3 次新加的行之后与第 2 次产物逐字节比" >&2
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e139_tightened_floor \
    | awk '!/^E7RESULT name=(core_gate_|verdict_core_gate_|done )/ {print; kept++} END {print "E7RESULT name=done emitted=" kept + 1}')
}

# E142 第十五次跑步④（重跑登记 `research/prompts/e142-r15-prereg.md` 第六节）：装置↔crates/ 逐字节比对
# 要跨两个 cargo workspace（scripts/experiment-apparatus/ 与仓根的 crates/ workspace，仓根 Cargo.toml 显式 exclude =
# ["scripts/experiment-apparatus"]，两边互相看不到对方，不能合并成一次 cargo 调用）。旧的 `new_pool_file_creation_region_bytes`
# 读的是改位置寻址之前的旧布局（区域表写死八个单元，见该文件模块注释），已经比不出新写的五个分配记录树节点，
# 换成只读导出 `e142_new_pool_file_creation_write_dump`（不带任何布局知识，逐次写按 (设备, 偏移, 长度, sha256,
# 整段十六进制) 原样打出来）。装置的第一个命令行参数是这份导出的文件路径（做 Q142.1 真比对），第二个参数是
# arm O 的历史留存产物（`第十五次跑步④` 起 Q142.8 用它算「哪些区域从旧布局变到了新布局」，
# 交回报告 `research/prompts/e142-r15-step234-runner-report.md` 里写明这份参照为什么找不到能重新编译的
# 源码、只能用留存产物）。两段都各自有自己的 `name=done`，闸 2 逐段核过。
#
# E142 第十六次跑第一段（重跑登记 `research/prompts/e142-r16-prereg.md`；交回报告
# `/tmp/claude-1000/e142-r16-s1/report.md`）：α（内部/根节点一格）、β（根 largest_key）、ι（内部条目
# key 取什么）、γ（extent 上段叶 key 区间）三格 5.1 的变体开关整套删除，改成 D8（核心索引结构） 已定项 14
# 第 395/401 行、D18（块里携带什么信息） 已定项 2 射程写死的唯一写法（稀疏、整个 key 空间、按位置分片区间）；
# δ（盘上槽数）收口成 `slots_of_device_bytes` 一个 const fn。第二个命令行参数从「第十四次跑 arm O 参照」
# 改成「这一次步①现编现跑的臂 N15 参照」（`research/results/e142-new-pool-file-creation-dry-run-2026-09-25-r16-arm-n15.out`），
# 按 (设备, 偏移, 长度) 配对出 `name=old_new_region`（Q142.19），不再按名字配对出旧的 `name=old_new_region`
# 系列。新增 `name=g3_shape`（第八节 G3 五个几何点，不依赖 `crates/`）、`name=positive_control_p2`
# （四个点，锚点随稀疏改成第 0 条条目）、`name=g4_bytes_equal_summary`（第三个命令行参数给才跑，这一次
# `crates/` 一盘几何造不出（`make_filesystem.rs` 的 `make_filesystem` 开头断言两块盘，D2（RAID 条带策略） 已定项 9），不传，报
# `skipped=true`）。`code2_field_rows` 从「非空格与第一个空格」改成「每条都列 + 补齐区一行」。
# Q142.11（原 Q142.1）这一次判「全等」：29 个区域全部配上、29 个相等、0 个不等（三格上模型与 `crates/`
# 逐字节相同，只说明 kb 转写与 `crates/` 一致，不说明条款本身对，见第十二节修订与第 5.4 节）。
#
# E142 第十七次跑第一段（重跑登记 `research/prompts/e142-r17-prereg.md`；交回报告
# `/tmp/claude-1000/e142-r17-s1/report.md`）：问题单第 6、8 行——阳性对照的期望违例数三档改成从
# 写清单现算（R10），不再写死 `− 8`；加逐态预言器、结果类矩阵、G6/G7 两个几何敏感性取样点、PC2
# （主臂几何逐单元靶向对照）、角色名自检（Q142.27）、文本数字自检（Q142.28）、Q142.29 两处点名的
# 直接比较；`journal_record_name_for_offset` 的第三条记录角色名从写死 `"t9"` 改成写清单现算的
# 动态标签；G22 空白清单条目从字面改成绑定这份产物自己的记账数。这一段没有改动主臂写字节的路径
# （S7：改动前后 29 行 `name=device_region_bytes` 逐字节相同），`driver_e142` 本身不用改；
# 存盘文件名从 `-r16-combined.out` 改成 `-r17-main.out`（登记步③给的文件名）。
# 臂 N15 参照是第十六次跑步①的产物，放在 research/results/ 下、可以被归档删掉：不在时这一行记跑不了（文件头「驱动要读的输入」），不编、不跑。
# E142 的崩溃点重放 主臂枚举不设变量就跑（按键复用），复跑表里没有单独的第二段；准入键 E142/layer0 的同一趟前提（.claude/gate.d/stage-inputs.tsv 那一行的 same-run-field=）由准入模块自己判。
driver_e142() {
  local impl_snapshot="$OUT_DIR/e142-crates-write-dump.tmp"
  local arm_n15_reference="research/results/e142-new-pool-file-creation-dry-run-2026-09-25-r16-arm-n15.out"
  replay_inputs_present "${FUNCNAME[0]}" "臂 N15 参照：${REPLAY_ARCHIVED_REMEDY}" "$arm_n15_reference" || return $?
  (cargo run -q -p singlefs-verification --bin e142_new_pool_file_creation_write_dump) >"$impl_snapshot" || return 1
  "$REPLAY_RELEASE_DIRECTORY"/e142-new-pool-file-creation-dry-run "$impl_snapshot" "$arm_n15_reference" || return 1
  cat "$impl_snapshot"
}

# E142 第二十二次跑第二段（重跑登记 `research/prompts/e142-r22-prereg.md` 第五节 5.4 ⑧）：`crates/` 在两块 8 GiB 盘上的导出，
# 与 `driver_e142` 第一条命令相同，只多 `-- --device-bytes 8589934592`（导出 bin 的可选参数，不带时 4 GiB、输出不变）。
# 装置那一侧的 P8G 一段已在 `driver_e142` 的产物里（`name=p8g_` 打头的行），这一份是它要比的另一半，比对脚本 `p8g_region_compare.py`
# 在 `research/results/e142-r22-s2-comparisons-2026-10-05.out` 的附录里。
driver_e142_p8g_crates_dump() {
  (cargo run -q -p singlefs-verification --bin e142_new_pool_file_creation_write_dump -- --device-bytes 8589934592) || return 1
}

# E156（入库装置，跑前登记「一」读法写死第 1 行；重跑登记 `research/prompts/e156-r2-prereg.md`「五、5.7」
# 第一、二、三段都在同一个二进制里，产物是累计的：stage2.out 前 324 行是第一段（岔路 7），
# 之后是第二段（岔路 3，run_hf_single_cell）与第三段（岔路 1，run_hh_cell）新加的行。
# stage3.out（重跑登记「十二」修订这一段）在 stage2.out 的基础上追加：Q1 诊断溯源
# （`q1_delta_debug`／已回收未覆盖记录过滤，见修订）、岔路 1 的 S = 4 第二个几何取样点（`q1_hh`／
# `q1d_*` 的 `s=` 字段、`anchor_k8`、`q1_geometry_sensitivity_s`）、K9 前提现核（`k9_precondition`）；
# 在 stage2.out 原有的 329 行共有格式上逐字节相同（`e156-s4/report.md` 的对拍命令），Q3e（X8-A）仍
# `status=not_done`，未做。stage4.out（续派第二段）在 stage3.out 的基础上追加：岔路 1
# 步数对齐对照（`q1_step_matched_diff`，`run_hh_cell` 新增 `matched_to_holes` 参数）、岔路 3 的
# Q3e（X8-A/HY，`run_x8a_cell`，独立 128 槽小池）与 Q3d（`q3d_derived`）；在 stage3.out 原有的 323 行
# （K1/legal_state/s1d_step/Q7 全家/PC-检查三条/HK-HR-H0 构造）共有格式上逐字节相同。
# r3.out（第 3 次重跑登记 `research/prompts/e156-r3-prereg.md`：分配记录树按位置寻址之后
# 重算钉着旧布局的常量）在 stage4.out 的基础上结构性改动，不是逐字节追加：分配记录树不再是单节点，
# 装置里的三处常量与另外六处改成第七节 7.2 算出的闭式与绝对值（K1-1 第 1 项 13→17、β0 走读引用 12→16、
# S1(c) 隔离槽数 34→54，其余变成随「这次改动的记录落在几片叶」现算的闭式，不再是常数）；新增
# `anchor_a_d8`、`anchor_a_d8_root_level`、`s1ef_step`／`s1ef_summary`（Q3r.2 逐次闭式核对）、
# `r5_empty_publish_closed_form`、`r7_hy_cap`／`r7_hy_condition`（HY 的覆盖写次数改成搜出满足
# e ≥ max(8, f) 的那一档，这一轮搜到的是 0：搜索过程见交回报告，判定按登记走）行；H0／HR 不再撞
# `AllocationRecordsExceedOneNode` 那道墙，跑满登记要求的全部步数（`baseline_workload_completed_full_length`
# 取代 stage4.out 的 `baseline_workload_truncated_by_write_failure`）。Q3r.4（岔路 1、3、7 判定变不变）
# 的比对命令与判定表见交回报告；核心结论：岔路 1（Q1d 在 S = 4 上从「单调」变「不单调」）与岔路 7
# （`q7d2_min_item5` 从「非 0」变「= 0」，F16 从「触发」变「不触发」）判定变了，岔路 3（Q3c/Q3e/K9）
# 在已核的量上不变。
# fork7-selfproof.out（续派，E156 第 3 次重跑登记「十二」修订 4）在 r3.out 的基础上加
# 一个第 10 个基底 `beta_hr_rollback_row`：r3.out 的 `q7d2_min_item5` 第一次在可达状态上读到
# `min_item5=0 family=HR kind=rollback_row txg=76`（其余 7 个可达基底第 5 项最小是 1），这里把这一步
# 也捕成一个基底、并入既有的 `bases` 数组（9→10），让已有的 Q7a/Q7c①②/PC-检查循环再跑一遍，不改
# `q7c_self_test` 的公式、不新写判定逻辑。核心结论：Q7c① 在这个基底上第一次转色（`q7c1_not_subtracting_defer
# basis=beta_hr_rollback_row … flips_red_to_green=true`）——另外只有不可达的 β_syn 转过色；`basis_count`
# 9→10，`q7a_summary` 的 `all_red_count` 18→20（该基底 delta=1/8 各命中一次），`q7c1_flip_seen`/
# `q7c2_flip_seen` 字面不变（β_syn 已经让它们是 true）。在 r3.out 原有的 1076 行共有格式上逐字节相同，
# 只在旧的 `q7a_summary`/`integrity` 两行与新增 9 行（`basis_snapshot`/`q7a` ×3/`q7c1`/`q7c2`/
# `pc_check` ×3）上不同。
# 2026-09-26.out（第 4 次重跑登记 `research/prompts/e156-r4-prereg.md`，第十二节修订 2）整个换了装置：回退改成挂着时的一次向前发布、
# 挂载内回收、准入改式之后，岔路 3、7 与 H0/HR/HK/HF/HX/HY 不再跑，二进制只跑第一段（PQ1-前/后、PC-c2、PQ2、U11、U13 两半、G-adm）
# 与第二段（Hh(k, 位置, S, ρ) 38 条排得下的历史 × 「实」「每」两个回收时点、Q1c/Q1c-洞/Q1c-前/Q1c-后/Q1d、PC-多扣/洞/分配器/判定器）；
# 第一段任一停机条款触发就只出第一段（`name=stage_two_not_run`）。不是在 fork7-selfproof.out 上追加，两份没有共同格式。
# 整个装置就活在 crates/singlefs-verification 里，没有 scripts/experiment-apparatus 侧的配对二进制，先例同 E142（第 371 行注释）——
# 两个 cargo workspace 互相看不到对方，不能合并成一次调用。确定性：同一个二进制跑两遍逐字节一致（现查：
# 这一段的产物两次跑出 `cmp` 逐字节一致）。
# 2026-10-05.out（第 5 次重跑登记 `research/prompts/e156-r5-prereg.md`）：core 在分配器没装根环表而记了一条根时断言（C551），装置的
# PC-分配器 撤了、换成 PC-钉门槛（格表里 Hh(1, 后, S, ρ) 六格的两份滞后集合交 k = 0 的 A10 判定，打 `name=pc_pinned_floor` 七行）；
# `v2_pc_allocator_seen` 换名 `v2_pc_pinned_floor_seen`。与 2026-09-26-r2.out 在新 crates/ 上规范化之后有 294 行不同（登记 Q2b；
# `…-2026-10-05-attribution-without-31-1.out` 是副本里反打 31-1 那一块的归属跑，与本份逐字节相同，不进这里），r2 那一份留着、不再登记。
# 2026-10-05-r2.out（第 6 次重跑登记 `research/prompts/e156-r6-prereg.md`）：回收门槛照乙读（挂载时读回全零的根环槽
# 钉在它可能住过的最旧 txg，直到这个进程写掉那一槽），K10 守门、「每」列与锚点模型同改；新加 `name=all_zero_pins_at_mount`、
# `name=all_zero_pin_cell`、`name=all_zero_pin_summary` 三种行，已有行格式不变。产物用仓根 `target/release/` 的二进制跑（debug 一遍约 13 分钟，
# 子 agent 前台跑不完）；debug 二进制前 1803 行与之逐字节相同，整份 debug 复跑由这一行的 exact 比对。`…-2026-10-05-all-zero-pin-removed.out`
# 是副本里去掉 core 全零钉住（同目录 `.patch`）的判别力对照，不进这里；与第 5 次的判定行比较在 `…-2026-10-05-r6-vs-r5-compare.py` / `.txt`。
# 2026-10-18.out（第 7 次重跑登记 `research/prompts/e156-r7-prereg.md`，问题单 H1）：装置加了洞数档（含环满一圈）与跨圈的族（Hc、Hs）、
# `name=r7_tier_peaks`、`name=r7_verdict`、`name=pc_tier_sequence`、`name=pc_same_slot`、`name=anchor_r7_*` 等行，116 条历史 × 两个回收时点，
# 16669 行、挂钟约 241 秒；与 2026-10-05-r2.out 没有共同格式（不是追加），那一份留着、不再登记。判段（给定状态表）要有 gpu 特性：
# 多机配置 ENABLE_GPU=1 时不带 gpu 特性编的装置会在 `verdict` 函数里 panic（交给别的机器判的占位结论），所以本行经 CHECKER_TIER_FEATURES 带
# `--features gpu` 起（清场与分卡归统一任务服务；BINDGEN_EXTRA_CLANG_ARGS 由仓根 `.cargo/config.toml` 给）；
# 用 `--release`（产物用 release 二进制跑出来，debug 整份要几小时）。核对用的锚点输入与脚本在 `…-2026-10-18-support/`，不进这里。
# E161（崩溃放量的去重与分段耗时）登记档（登记 5.5 的 `segment-one`）：入库装置 `segment-one` 模式；可行性档 `feasibility` 是缩域（只验能跑能用，产物写 not_judged，不当正式产物），复跑表不跑它。
# crates 的崩溃点重放枚举（S3、S4 对拍时调的）往标准输出打 LAYER0_* 进度行（带墙钟秒数），产物只留 E7RESULT 行。
# 计时字段一律以 _elapsed_ns 或 _ratio 结尾，按 timing 比结构；PC3 的判定行随计时可能翻，翻了是新观测。线程数钉 31（第 3 次重跑登记 research/prompts/e161-r3-prereg.md J-8 与第十四节：登记值，产物 config 行记 threads=，timing 比法不去这个字段）；内存额度由 task-server 给，驱动里不套内存包装。
driver_e161_segment_one() {
  (set -o pipefail && E161_THREADS=31 cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e161_crash_state_dedup_and_time_split -- segment-one | grep '^E7RESULT ')
}

# E165（恢复重放有界穷举与变异杀伤界）第一段（登记 `research/prompts/e165-preregistration.md` 5.5）：入库装置，只跑合法历史一族
# （`--family legal`；对抗一族停在登记停机条款 H4，见登记第十二节修订 7，数没入库）。装置另打 `E7TIMING` 耗时行，产物只留 E7RESULT 行；
# 计数与线程数无关（10 线程与 7 线程两遍逐字节相同），线程数不钉、装置取每台默认重活线程数（.claude/heavy-thread-default）。
driver_e165() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal | grep '^E7RESULT ')
}
# 第一段对抗一族（登记修订第 9 条收窄 H4 之后入库；16 线程与 7 线程两遍逐字节相同）。
driver_e165_adversarial() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family adversarial | grep '^E7RESULT ')
}
# 第二段（第 2 次重跑 r2 登记 `research/prompts/e165-r2-prereg.md` 5.6、9.1–9.5，补跑 2c）：26 个变体各一份副本、各编一次，整段约 15 分钟（同机负载 12–30，--jobs 2 --threads 4；26 个变体都跑，估计 15 至 20 分钟，没整轮量过）。
# 驱动在副本里改 recovery.rs、自己经内存包装跑每个二进制，这里不再包一层。产物里每行的复跑命令写的是 research/results/ 下的路径，
# 这里的输出落在临时目录，比对前把两个文件名换回那个写法；红例账的 sha256 在产物的 name=verdict 行里，账一字不差才对得上。
# 驱动的 REGISTERED 钉的是第四段冻结根的表（`crates/mutations.tsv` 9d5fd9fc…）与 recovery.rs、lib.rs 的哈希，主工作区的 crates/ 变了它就停在 RH1，
# 所以复跑要 --root 指冻结根：没有默认、不在记跑不了（清单与建法见登记 r2 修订 4，第 819–820 行一带），那份没了就照修订 4 重建一份、
# 放到固定位置（工作根下 replay-frozen-roots/e165-r2-segment4）；根里要有 scripts、scripts/mutation/tables、第一段 r2 的 segment1b 两份产物与 .claude/scripts/preflight.sh。
# 驱动 --selftest 不带 --root；这一份基线是 1373 行（修订 4 挪了三项自检之后），同一棵根上复得出，别的树复不出来先对冻结根清单。
driver_e165_segment2() {
  local scratch="$OUT_DIR/e165-segment2"
  # 根在固定的位置（replay_frozen_root e165-r2-segment4，工作根下 replay-frozen-roots/e165-r2-segment4），不经环境变量指；根不在，这一行记跑不了（文件头「驱动要读的输入」），不报本次未跑
  local frozen_root
  frozen_root="$(replay_frozen_root e165-r2-segment4)" || return 1
  replay_inputs_present "${FUNCNAME[0]}" "冻结根：照登记 research/prompts/e165-r2-prereg.md 修订 4（第 819–820 行一带）的清单与建法重建一份（根里要有 scripts、scripts/mutation/tables、第一段 r2 的 segment1b 两份产物与 .claude/scripts/preflight.sh）、核过清单，放到固定位置（工作根下 replay-frozen-roots/e165-r2-segment4，replay_frozen_root 取）" "$frozen_root" || return $?
  mkdir -p "$scratch/out"
  (bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --root "$frozen_root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds.out" \
      --ledger "$scratch/out/e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-red-examples.tsv" >&2) || return 1
  sed -e 's#--output e165-#--output research/results/e165-#' -e 's#--ledger e165-#--ledger research/results/e165-#' \
    "$scratch/out/e165-replay-bounded-enumeration-2026-10-02-r2-segment2c-kill-bounds.out"
}
# 第 2 次重跑（登记 `research/prompts/e165-r2-prereg.md`）第三段与第四段的跑法：入库装置，`--profile r2`；第三段是多盘一族，第四段几份是 `--table both`（两种建表法各判一遍逐态比）与盘次序反过来。
# 登记行指的产物是第 5 次重跑第一段（登记 `research/prompts/e165-r5-prereg.md` 第十一节比对对象表第 8–13 行，主 agent 认定第 4 条）在第一段根
# （`/tmp/claude-1000/e165-r5-segment1/root`，清单 `root-manifest-at-start.sha256`，sha256 95237ecf…）上编出的二进制、7 线程跑的；第 2 次重跑的旧产物
# （`…-2026-10-02-r2-segment3-multi-disk-2026-10-02.out`、`…-2026-10-02-r2-segment4-*.out`）留着，今天的装置在 J7a 两格上复不出它们（根环那条线 7-2 改了施加规则，登记 r5 PF2）。
# 这里在主工作区上 `cargo run`：`crates/` 里的文件主工作区与第一段根可能不同（别的会话在改），那样复出来对不上是新观测，先对着第一段根的清单查。
# 计数与线程数无关（第 2 次重跑多盘一族 8 与 4 线程两遍逐字节相同），线程数不钉、装置取每台默认重活线程数（.claude/heavy-thread-default）。
driver_e165_r2_segment3_multi_disk() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family multi_disk --profile r2 | grep '^E7RESULT ')
}
driver_e165_r2_segment4_table_both_legal() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal --profile r2 --table both | grep '^E7RESULT ')
}
driver_e165_r2_segment4_table_both_adversarial() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family adversarial --profile r2 --table both | grep '^E7RESULT ')
}
driver_e165_r2_segment4_table_both_multi_disk() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family multi_disk --profile r2 --table both | grep '^E7RESULT ')
}
driver_e165_r2_segment4_reversed_disk_order_legal() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal --only R --profile r2 --table both --disk-order reversed | grep '^E7RESULT ')
}
driver_e165_r2_segment4_reversed_disk_order_multi_disk() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family multi_disk --only M,MC --profile r2 --table both --disk-order reversed | grep '^E7RESULT ')
}
# 第 3 次重跑（登记 `research/prompts/e165-r3-prereg.md`）：`--profile r3`，入库装置；第一段是对抗一族按「同一实例里 journal 记录的 txg 随计数器不减」拆行（5.6），
# 第二段是择根与重放合判一族 `--family root_choice`（5.3、5.5），计数与线程数无关（两段都是 8 与 4 线程两遍整份逐字节相同），线程数不钉、装置取每台默认重活线程数（.claude/heavy-thread-default）。
# 对抗一族那一行指的产物是第 5 次重跑第一段（登记 `research/prompts/e165-r5-prereg.md` 比对对象表第 14 行，主 agent 认定第 4 条）在第一段根
# （`/tmp/claude-1000/e165-r5-segment1/root`，清单 sha256 95237ecf…）上编出的二进制跑的；第 3 次重跑第一段的旧产物（`…-2026-10-02-r3-segment1-adversarial-family.out`）留着，
# 今天的装置在 J7a 分配记录树根那一格上复不出它。择根那一行由第 7 次重跑（KA → KL，登记 `research/prompts/e165-r7-prereg.md` 5.6 H9）改指
# `…-2026-10-23-r7-r3-root-choice.out`：对抗根环 KA 换成照落点公式摆根的 KL（27 态），KL 那几行之外与第 3 次第二段的旧产物
# （`…-2026-10-03-r3-segment2-root-choice.out`，留着）逐字节相同，汇总行状态数差 −98（登记 L2、B4）。产物是第 7 次重跑第一段执行员在主工作区当时
# `crates/` 的拷贝上打装置补丁编出的二进制跑的（打上之后装置 sha256 25a298d8…，二进制 ce552d8d…，31 线程；那份拷贝没留根清单）。
# 这里在主工作区上 `cargo run`，`crates/` 里的文件（别的会话在改）与两份根可能不同，复出来对不上是新观测，先对着根的清单查（对抗一族：第 5 次第一段
# `root-manifest-at-start.sha256`；择根：先核主工作区的装置是不是 25a298d8…，再查 `crates/` 别处的改动）。
driver_e165_r3_segment1_adversarial() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family adversarial --profile r3 | grep '^E7RESULT ')
}
driver_e165_r3_segment2_root_choice() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family root_choice --profile r3 | grep '^E7RESULT ')
}
# 第 6 次重跑（登记 `research/prompts/e165-r6-prereg.md`）：入库装置，`--profile r3 --family legal`（偶数卸载记号、直接建表、盘次序照常），r3 档合法一族的正式跑，
# J6Qn 与 J7 五行的阳性对照在这一档只在这一跑里出。产物是开工那一刻主工作区 Cargo.toml、Cargo.lock、.cargo/、crates/ 的拷贝
# （~/.local/share/singlefs/work/e165-r6-runner/root，清单 `root-manifest-at-start.sha256`，340 个文件，清单 sha256 56d52a1c…）上编出的二进制跑的；
# 计数与线程数无关（8 与 4 线程两遍整份逐字节相同，`…-r6-r3-legal-family-threads-second.out`），线程数不钉、装置取每台默认重活线程数（.claude/heavy-thread-default）。同一份根上的 r2 合法一族只当比较对象，不登记复跑。
# 这里在主工作区上 `cargo run`，`crates/` 里的文件（别的会话在改）与那份根可能不同，复出来对不上是新观测，先对着根的清单查。
driver_e165_r6_r3_legal() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal --profile r3 | grep '^E7RESULT ')
}
# 第 3 次重跑第三段（登记 `research/prompts/e165-r3-prereg.md` 5.8、9.1、9.3）：变体 1822–1827 各一份副本、各编一次，驱动带 --profile r3 调装置
# （--profile r3 --bounds segment1 --family root_choice），不落红例账。驱动在副本里改 recovery.rs 或 lib.rs、自己经内存包装跑每个二进制，这里不再包一层；
# 产物里每行的复跑命令写的 --output 是产物的文件名（--root 指冻结根时驱动只写 basename，不管产物落在哪），这里的输出落在临时目录，同样只写 basename，原样比，不做替换。
# 驱动的 REGISTERED_R3 钉的是第二段冻结根的 recovery.rs、lib.rs、crates/mutations.tsv 的哈希与两个函数的区间，主工作区的 crates/ 变了它就停在 RH1，
# 所以复跑要 --root 指冻结根：没有默认、不在记跑不了（清单 `segment2-root-after-overlay.sha256`，233 个文件），那份没了就照登记第十二节修订 2 的第 10 条重建一份、
# 放到固定位置（工作根下 replay-frozen-roots/e165-r3-segment2）。TV7 的比对对象是脚本所在的仓里的第二段产物（research/results/e165-replay-bounded-enumeration-2026-10-03-r3-segment2-root-choice.out）。
# 整段（基线两遍加变体六个，每个冷编一次再跑一遍 70 秒量级的装置）的用时跑之前没量过；驱动不给 --jobs、--threads（照驱动的缺省：1 份、每份取每台默认重活线程数），只在提交时由主 agent 跑。
driver_e165_r3_segment3_root_choice_kill_bounds() {
  local scratch="$OUT_DIR/e165-r3-segment3"
  # 根在固定的位置（replay_frozen_root e165-r3-segment2，工作根下 replay-frozen-roots/e165-r3-segment2），不经环境变量指；根不在，这一行记跑不了（文件头「驱动要读的输入」），不报本次未跑
  local frozen_root
  frozen_root="$(replay_frozen_root e165-r3-segment2)" || return 1
  replay_inputs_present "${FUNCNAME[0]}" "冻结根：照登记 research/prompts/e165-r3-prereg.md 第十二节修订 2 的第 10 条重建一份（清单 segment2-root-after-overlay.sha256，233 个文件）、核过清单，放到固定位置（工作根下 replay-frozen-roots/e165-r3-segment2，replay_frozen_root 取）" "$frozen_root" || return $?
  mkdir -p "$scratch/out"
  (bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --profile r3 --root "$frozen_root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out" >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-03-r3-segment3-root-choice-kill-bounds.out"
}
# 第 4 次重跑第一段（登记 `research/prompts/e165-r4-prereg.md` 5.3 至 5.6、第七节与修订 1、2）：入库装置，`--profile r4 --family root_choice_faults`，
# 带读故障的择根合判一族（KF、KF3、KF9、KFW、KG、KA；闸取实现的取甲）。计数与线程数无关（6 与 3 线程两遍整份逐字节相同，线程数不钉、取每台默认重活线程数）；整遍峰值 RSS 约 1.7 GB，
# 产物是第一段根（`/tmp/claude-1000/e165-r4-segment1b/root`，清单 `root-manifest-after-overlay.sha256`）上编出的二进制跑的；
# 这里在主工作区上 `cargo run`，`crates/` 里别的文件（别的会话在改，尤其 `mount.rs`、`transaction.rs`：条款改取 R2 之后实现一落地，这一族要重跑、产物会变）与那份根可能不同，
# 复出来对不上是新观测，先对着根清单查。
# 第 7 次重跑（KA → KL，登记 `research/prompts/e165-r7-prereg.md` 5.6 H9）起登记行改指 `…-2026-10-23-r7-r4-root-choice-faults.out`：这一族的 KA 换成照落点公式摆根的 KL
# （与 r3 档 KL 出自同一个生成函数），KL 那几行之外与第 4 次的旧产物（`…-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out`，留着）逐字节相同，
# 汇总行状态数差 −98（登记 L2、B4）。二进制同 r3 择根那一行的第 7 次产物（装置 25a298d8…，二进制 ce552d8d…，31 线程），复不出时先核主工作区的装置是不是 25a298d8…。
driver_e165_r4_segment1_root_choice_faults() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family root_choice_faults --profile r4 | grep '^E7RESULT ')
}
# 第 4 次重跑第二段（登记 `research/prompts/e165-r4-prereg.md` 9.1、9.3 与修订 3）：变体 v1822–v1827（在根里的 `crates/mutations.tsv` 按第一列的名字选行）各一份副本、各编一次，
# 驱动带 --profile r4 调装置（--profile r4 --bounds segment1 --family root_choice_faults），不落红例账。驱动在副本里改 recovery.rs 或 lib.rs、自己经内存包装跑每个二进制，这里不再包一层。
# 驱动的 R4_REGISTERED 钉的是第一段根的 recovery.rs、lib.rs、crates/mutations.tsv 的哈希与两个函数的区间，主工作区的 crates/ 变了它就停在开工核对，
# 所以复跑要 --root 指第一段根：没有默认、不在记跑不了（清单 `root-manifest-after-overlay-final.sha256`，246 个文件），那份没了就照登记第十二节修订 2 的「根」条重建一份、
# 放到固定位置（工作根下 replay-frozen-roots/e165-r4-segment1b）。TV7 的比对对象是脚本所在的仓里的第一段产物（research/results/e165-replay-bounded-enumeration-2026-10-03-r4-segment1-root-choice-faults-2026-10-03-r2.out）。
# 内存额度由统一任务服务给，这里不写上限（第一段整遍峰值约 1.66 GB；登记修订 3 记着第一趟限 4G、--jobs 2 时 tsv-1823 撞顶退 250，补跑限 8G 才得出结果）；那一次 --jobs 2 --threads 3、整轮约 10 分钟，这里不给这两项（驱动缺省 1 份、每份取每台默认重活线程数）
# （第二段那一次变体各编各跑 130 秒量级，开头收尾基线各 66 秒量级），只在提交时由主 agent 跑。
# 产物里每行的复跑命令写的 --output 是产物的文件名（--root 指第一段根时驱动只写 basename），这里的输出落在临时目录，同样只写 basename，原样比。
driver_e165_r4_segment2_root_choice_faults_kill_bounds() {
  local scratch="$OUT_DIR/e165-r4-segment2"
  # 根在固定的位置（replay_frozen_root e165-r4-segment1b，工作根下 replay-frozen-roots/e165-r4-segment1b），不经环境变量指；根不在，这一行记跑不了（文件头「驱动要读的输入」），不报本次未跑
  local frozen_root
  frozen_root="$(replay_frozen_root e165-r4-segment1b)" || return 1
  replay_inputs_present "${FUNCNAME[0]}" "第一段根：照登记 research/prompts/e165-r4-prereg.md 第十二节修订 2 的「根」条重建一份（清单 root-manifest-after-overlay-final.sha256，246 个文件）、核过清单，放到固定位置（工作根下 replay-frozen-roots/e165-r4-segment1b，replay_frozen_root 取）" "$frozen_root" || return $?
  mkdir -p "$scratch/out"
  (bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --profile r4 --root "$frozen_root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out" >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-03-r4-segment2-root-choice-faults-kill-bounds.out"
}
# 第 5 次重跑第二段（登记 `research/prompts/e165-r5-prereg.md` 5.6、9.1、9.3）：驱动三个模式加 `--pins r5`，变体同第 2–4 次（r2 模式 27 个变体、r3 / r4 模式各 7 个），
# r2 模式另落红例账。驱动的 R5_PINS 钉的是第 5 次第一段根的 recovery.rs、lib.rs、crates/mutations.tsv 的哈希与函数区间，主工作区的 crates/ 变了它就停在开工核对，
# 所以复跑要 --root 指那份根：没有默认、不在记跑不了（主工作区当时的 Cargo.toml、Cargo.lock、.cargo/、crates/ 的拷贝，清单
# `root-manifest-at-start.sha256`，291 个文件，清单 sha256 95237ecf…），那份没了就照登记第十一节「根、清单、在哪编」重建一份、核过清单，放到固定位置（工作根下 replay-frozen-roots/e165-r5-segment1）。
# 自造表、内存包装与 V7 / TV7 的比对对象（第 5 次第一段的重跑产物）取脚本所在的仓。产物里每行的复跑命令写的 --output / --ledger 是文件名，这里原样比，不做替换。
# 内存额度由统一任务服务给，这里不写上限（那一次限的是 r2、r4 模式 8G，r3 模式 1G）；那一次 --jobs 2 --threads 3，这里不给这两项（驱动缺省 1 份、每份取每台默认重活线程数）；用时那一次 r3 约 10 分钟、r4 约 15 分钟、r2 约 20 分钟量级（`-timing.out`），只在提交时由主 agent 跑。
driver_e165_r5_segment2_r2_mode_kill_bounds() {
  local scratch="$OUT_DIR/e165-r5-segment2-r2"
  # 根在固定的位置（replay_frozen_root e165-r5-segment1，工作根下 replay-frozen-roots/e165-r5-segment1），不经环境变量指；根不在，这一行记跑不了（文件头「驱动要读的输入」），不报本次未跑
  local root
  root="$(replay_frozen_root e165-r5-segment1)" || return 1
  replay_inputs_present "${FUNCNAME[0]}" "第一段根：照登记 research/prompts/e165-r5-prereg.md 第十一节「根、清单、在哪编」重建一份（主工作区当时的 Cargo.toml、Cargo.lock、.cargo/、crates/ 的拷贝，清单 root-manifest-at-start.sha256，291 个文件，清单 sha256 95237ecf…）、核过清单，放到固定位置（工作根下 replay-frozen-roots/e165-r5-segment1，replay_frozen_root 取）" "$root" || return $?
  mkdir -p "$scratch/out"
  (bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --pins r5 --root "$root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-kill-bounds.out" \
      --ledger "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-red-examples.tsv" >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r2-mode-kill-bounds.out"
}
driver_e165_r5_segment2_r3_mode_kill_bounds() {
  local scratch="$OUT_DIR/e165-r5-segment2-r3"
  # 根在固定的位置（replay_frozen_root e165-r5-segment1，工作根下 replay-frozen-roots/e165-r5-segment1），不经环境变量指；根不在，这一行记跑不了（文件头「驱动要读的输入」），不报本次未跑
  local root
  root="$(replay_frozen_root e165-r5-segment1)" || return 1
  replay_inputs_present "${FUNCNAME[0]}" "第一段根：照登记 research/prompts/e165-r5-prereg.md 第十一节「根、清单、在哪编」重建一份（主工作区当时的 Cargo.toml、Cargo.lock、.cargo/、crates/ 的拷贝，清单 root-manifest-at-start.sha256，291 个文件，清单 sha256 95237ecf…）、核过清单，放到固定位置（工作根下 replay-frozen-roots/e165-r5-segment1，replay_frozen_root 取）" "$root" || return $?
  mkdir -p "$scratch/out"
  (bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --profile r3 --pins r5 --root "$root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r3-mode-kill-bounds.out" >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r3-mode-kill-bounds.out"
}
driver_e165_r5_segment2_r4_mode_kill_bounds() {
  local scratch="$OUT_DIR/e165-r5-segment2-r4"
  # 根在固定的位置（replay_frozen_root e165-r5-segment1，工作根下 replay-frozen-roots/e165-r5-segment1），不经环境变量指；根不在，这一行记跑不了（文件头「驱动要读的输入」），不报本次未跑
  local root
  root="$(replay_frozen_root e165-r5-segment1)" || return 1
  replay_inputs_present "${FUNCNAME[0]}" "第一段根：照登记 research/prompts/e165-r5-prereg.md 第十一节「根、清单、在哪编」重建一份（主工作区当时的 Cargo.toml、Cargo.lock、.cargo/、crates/ 的拷贝，清单 root-manifest-at-start.sha256，291 个文件，清单 sha256 95237ecf…）、核过清单，放到固定位置（工作根下 replay-frozen-roots/e165-r5-segment1，replay_frozen_root 取）" "$root" || return $?
  mkdir -p "$scratch/out"
  (bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --profile r4 --pins r5 --root "$root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r4-mode-kill-bounds.out" >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment2-r4-mode-kill-bounds.out"
}
# 第 5 次重跑第三段（登记 5.3、5.4，`--profile r5`：记录后三项照 D23（journal 的角色与格式） 已定项 15 生成，J8 五行）与第五段 5-甲（修订 1，`--profile r4
# --family root_choice_faults --gate r2`：系统配置读错取 R2 的闸）：入库装置。产物是第一段根拷一份、盖上当时主工作区的装置与 crates/mutations.tsv 编出的二进制、7 线程跑的
# （第三段根 /tmp/claude-1000/e165-r5-segment3/root，装置 f72fc1a4…，清单 `root-manifest-after-overlay.sha256`；5-甲 的根 …/root5，装置 8006b251…，清单 sha256 47c6d349…）。
# 主工作区今天的装置是 8006b251…；5-甲 的二进制在 5-甲 的根上重跑第三段的合法历史那一份整份逐字节相同（`…-r5-segment5-profile-check.out` 的 pv13 行），另外四份没在它上面重跑过。
# 这里在主工作区上 `cargo run`，`crates/` 里别的文件（别的会话在改）与那两份根可能不同，复出来对不上是新观测，先对着根清单查。
# 计数与线程数无关（合法历史、对抗两族 7 与 3 线程逐字节相同；R2 一族 7 与 3 线程逐字节相同，`…-threads-second.out`），线程数不钉、取每台默认重活线程数；R2 一族整遍峰值没读到。
# 第 7 次重跑（KA → KL，登记 `research/prompts/e165-r7-prereg.md` 5.6 H9）起 5-甲 那一行改指 `…-2026-10-23-r7-r5-gate-r2-root-choice-faults.out`：KA 换成照落点公式摆根的 KL，
# KL 那几行之外与 5-甲 的旧产物（`…-2026-10-04-r5-segment5-r2-gate-root-choice-faults.out`，留着）逐字节相同，汇总行状态数差 −98（登记 L2、B4）；
# 二进制同 r3 择根那一行的第 7 次产物（装置 25a298d8…，二进制 ce552d8d…，31 线程，8G 跑完），复不出时先核主工作区的装置是不是 25a298d8…。
driver_e165_r5_segment3_legal() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal --profile r5 | grep '^E7RESULT ')
}
driver_e165_r5_segment3_adversarial() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family adversarial --profile r5 | grep '^E7RESULT ')
}
driver_e165_r5_segment3_multi_disk() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family multi_disk --profile r5 | grep '^E7RESULT ')
}
driver_e165_r5_segment3_odd_marker_legal() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family legal --profile r5 --marker-parity odd | grep '^E7RESULT ')
}
driver_e165_r5_segment3_odd_marker_adversarial() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family adversarial --profile r5 --marker-parity odd | grep '^E7RESULT ')
}
driver_e165_r5_segment5_r2_gate_root_choice_faults() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification --bin e165_replay_bounded_enumeration -- --bounds segment1 --family root_choice_faults --profile r4 --gate r2 | grep '^E7RESULT ')
}
# 第 5 次重跑第五段 5-乙（登记修订 1 修1.九乙）：驱动 r4 模式 `--pins r5 --gate r2`，变体 v1822–v1827 在 R2 那一族上各编各跑。驱动的 R5_GATE_R2_PINS 钉的是 5-甲 的根
# （/tmp/claude-1000/e165-r5-segment3/root5：第一段根盖上装置 8006b251… 与 crates/mutations.tsv c9a03eb4…，清单 `root5-manifest-after-overlay.sha256`，sha256 47c6d349…），
# 那份没了就照登记修1.十一「根、清单、在哪编」重建一份、核过清单，放到固定位置（工作根下 replay-frozen-roots/e165-r5-segment3-root5）；TV7 的比对对象是脚本所在的仓里的 `…-r5-segment5-r2-gate-root-choice-faults.out`。
# 那一次第一趟 8G、--jobs 2 时 tsv-1812 撞顶退 250，第二趟 16G 只补了它（登记修1.十一；两趟的驱动日志与计时各一份 `…-first-pass-*.out`、`…-second-pass-*.out`）；
# 这里不设内存上限（由统一任务服务定，撞顶由服务放大重排），驱动不给 --jobs、--threads（缺省 1 份、每份取每台默认重活线程数）一趟跑全部变体。
# 产物里每行的复跑命令写的 --output 是文件名、--root 写的是「<第一段根>」字样（驱动的写法，实际指的是 5-甲 的根），这里原样比。只在提交时由主 agent 跑。
driver_e165_r5_segment5b_r4_gate_r2_mode_kill_bounds() {
  local scratch="$OUT_DIR/e165-r5-segment5b"
  # 根在固定的位置（replay_frozen_root e165-r5-segment3-root5，工作根下 replay-frozen-roots/e165-r5-segment3-root5），不经环境变量指；根不在，这一行记跑不了（文件头「驱动要读的输入」），不报本次未跑
  local root
  root="$(replay_frozen_root e165-r5-segment3-root5)" || return 1
  replay_inputs_present "${FUNCNAME[0]}" "5-甲 的根：照登记 research/prompts/e165-r5-prereg.md 修1.十一「根、清单、在哪编」重建一份（清单 root5-manifest-after-overlay.sha256，sha256 47c6d349…）、核过清单，放到固定位置（工作根下 replay-frozen-roots/e165-r5-segment3-root5，replay_frozen_root 取）" "$root" || return $?
  mkdir -p "$scratch/out"
  (bash scripts/experiment-apparatus/e165-replay-mutation-kill-bounds.sh --profile r4 --pins r5 --gate r2 --root "$root" --scratch "$scratch" \
      --output "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds.out" >&2) || return 1
  cat "$scratch/out/e165-replay-bounded-enumeration-2026-10-04-r5-segment5b-r4-gate-r2-mode-kill-bounds.out"
}

# E168（删除借保留量的环窗口）第一段（登记 `research/prompts/e168-preregistration.md` 5.0、5.8、第十一节 S1、S2 与第十二节修订 1–17）：入库装置
# `crates/singlefs-verification/src/bin/e168_delete_ring_window_borrow.rs`。装置的行以 `E name=` 开头、收尾 `E name=done`，不用 E7RESULT 协议：
# 照 driver_e163_r1_merge 的先例在驱动里包一层（e168_wrap：行首 `E name=` 换成 `E7RESULT name=`，收尾行补 `emitted=`），入库产物就是包过的样子，
# 换回去与装置的原样输出逐字节相同（四份逐份核过）。
# s1-history 在主工作区上跑（不调删除入口）。positive-controls 与 segment1 要删除入口，主工作区那一份是桩、一调就打停机行退 3，
# positive-controls 在临时目录里拷一份 `crates/`、套删除路径补丁表 `scripts/mutation/tables/e168_delete_ring_window_borrow_delete_path.tsv`
# （每行旧串恰好命中一次，不中就整行跑不了：登记停机条款 S2）再编、跑（先例：driver_e165_segment2 在副本里改了再编）。segment1 一遍约 9 小时、36G，不进登记表，复跑照实验页「复跑」那一节手工步骤、由用户点名跑。
# 入库产物是第一段 r8 那一次在当时主工作区 `crates/` 的快照上编、跑的，快照逐文件的 sha256 在
# `e168-delete-ring-window-borrow-segment1-tree-2026-10-03.sha256`（不含补丁表）； `crates/singlefs-core/` 里的几份文件
# （extent_tree.rs、records.rs、recovery.rs 等）会随主工作区变，今天复出来对不上是新观测，先对着那份清单查。
# 计数与线程数无关（单测 segment_one_on_p2_s4_prints_the_same_lines_on_one_thread_and_on_eight_threads），线程数照那一次钉 4、P4 的片一次一片。
# positive-controls 峰值约 15 GB，s1-history 峰值约 2.3 GB，
# 只在提交时由主 agent 跑。
e168_wrap() {
  awk '/^E name=/{ sub(/^E name=/, "E7RESULT name="); n++; if ($0 == "E7RESULT name=done") { print $0 " emitted=" n; next } print }'
}
e168_patched_tree() { # e168_patched_tree <副本目录>：照共用库的闭包拷主工作区（replay_copy_closure_into），套删除路径补丁表
  local tree="$1"
  rm -rf "$tree" && mkdir -p "$tree" || return 1
  replay_copy_closure_into "$tree" || return 1
  (python3 -c 'import sys
root=sys.argv[2]; anchors_not_hit_once=0
for line in open(sys.argv[1],encoding="utf-8"):
    line=line.rstrip("\n")
    if not line or line.startswith("#"): continue
    _,path,old,new=line.split("\t"); old=old.replace("\\n","\n"); new=new.replace("\\n","\n")
    text=open(root+"/"+path,encoding="utf-8").read(); hits=text.count(old); print(path,hits); anchors_not_hit_once+=hits!=1
    if hits==1: open(root+"/"+path,"w",encoding="utf-8").write(text.replace(old,new,1))
sys.exit(1 if anchors_not_hit_once else 0)' scripts/mutation/tables/e168_delete_ring_window_borrow_delete_path.tsv "$tree") >&2
}
driver_e168_s1_history() {
  replay_inputs_present "${FUNCNAME[0]}" "留存产物：${REPLAY_ARCHIVED_REMEDY}" "research/results/e168-delete-ring-window-borrow-s1-history-2026-10-03.out" || return $?
  (set -o pipefail && cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e168_delete_ring_window_borrow -- s1-history | e168_wrap)
}
driver_e168_positive_controls() {
  replay_inputs_present "${FUNCNAME[0]}" "留存产物：${REPLAY_ARCHIVED_REMEDY}" "research/results/e168-delete-ring-window-borrow-positive-controls-2026-10-03.out" || return $?
  local tree="$OUT_DIR/e168-positive-controls-tree" configuration
  e168_patched_tree "$tree" || return 1
  configuration="$(replay_configuration_for_the_copy)" || return 1
  (cd "$tree" && unset CARGO_TARGET_DIR && export SINGLEFS_MULTI_HOST_CONFIG="$configuration" && set -o pipefail && E168_LARGE_SLICE_THREADS=1 cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e168_delete_ring_window_borrow -- positive-controls | e168_wrap)
}

# E170（O3规约执行器第一版）第 2 次重跑（登记 `research/prompts/e170-r2-prereg.md` 5.5「复跑行」、12.6、12.11 方案甲；取舍由主 agent 定，
# 调度记录 `records/2026-10-01-里程碑三形式化接入.md` 第 22 批）。今天的导出装置照 `crates/` 写 487 字节记录头，读 311 的第一代执行器与第 1 次重跑执行器（history-kept: 批号是调度记录里的登记位，只用来指出处）
# 那几行（第一代的坏状态导出、Q3 比对器、Q2 比对器、变异表，第 1 次重跑的 Q2 比对器）在今天的格式下没有对应物，不在这张表里。
# 九行：导出装置自己的两趟（`dump --compare-with`、`bad-states-second-run`），v2-487、v1-487 的单测与单测层变异（纯 Python），
# v2-487 判今天现导的 Z1（批 1–3）、Z3、Z4（造法照登记 12.6 与读法 13，种子 85441ae491419411 照抄）。v2（311）那几份不进复跑。
# 导出分两组、各造一次，同组的几行共用（flock 排着，复跑是并发起的；造好留完成标记，后来的直接用）：
#   e170 组：`dump` → e170-dump（Z1）；再 `dump --compare-with` → e170-dump-repeat；拷一份 e170-dump 造 `bad-states-second-run` → e170-bad-states-second-run（Z3）；
#   e171 组：E171 装置 `baseline` → e171-baseline-run1；拷一份造 `bad-states`（第一套表）→ e170-bad-states-copy-s2，再 E171 装置 `known-bad-images` → known-bad-s2（Z4）。
# 目录名照抄第 2 次重跑重导时的名字：判镜像那一份的 config 行写目录末一段名字。
# Python 一侧（执行器、`comparator_bad_image_judge.py`、`mutation_driver_unit_level.py`）原样跑，原始输出再经 v2-487 目录的
# `run_replay_result_lines.py` 改写成 E7RESULT 结果行：删去键以 _ns 结尾的字段（墙钟）与 worker=（进程号），绝对路径换成末一段名字，判定字段不动。
# 执行器用一个工作进程（--workers 1）：多进程时判定行按完成先后打出、行序每次不同。产物不带耗时，exact 比。
e170_r2_cargo_run() { # <bin> <参数…>：在仓根照今天的 crates/ 起入库装置
  local bin="$1"; shift
  cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin "$bin" -- "$@"
}
e170_r2_build_exports_e170() {
  local directory="$1"
  (set -o pipefail \
    && e170_r2_cargo_run e170_o3_spec_executor_export dump --output-directory "$directory/e170-dump" >/dev/null \
    && e170_r2_cargo_run e170_o3_spec_executor_export dump --output-directory "$directory/e170-dump-repeat" --compare-with "$directory/e170-dump" \
         | grep '^E7RESULT ' >"$directory/dump-compare.lines" \
    && cp -r "$directory/e170-dump" "$directory/e170-bad-states-second-run" \
    && e170_r2_cargo_run e170_o3_spec_executor_export bad-states-second-run --dump-directory "$directory/e170-bad-states-second-run" --seed-hex 85441ae491419411 \
         | grep '^E7RESULT ' >"$directory/bad-states-second-run.lines")
}
e170_r2_build_exports_e171() {
  local directory="$1"
  (set -o pipefail \
    && e170_r2_cargo_run e171_three_oracle_collusion baseline --output-directory "$directory/e171-baseline-run1" >/dev/null \
    && cp -r "$directory/e171-baseline-run1" "$directory/e170-bad-states-copy-s2" \
    && e170_r2_cargo_run e170_o3_spec_executor_export bad-states --dump-directory "$directory/e170-bad-states-copy-s2" --seed-hex 85441ae491419411 >/dev/null \
    && e170_r2_cargo_run e171_three_oracle_collusion known-bad-images --dump-directory "$directory/e170-bad-states-copy-s2" \
         --output-directory "$directory/known-bad-s2" --e170-bad-rows >/dev/null)
}
e170_r2_shared_exports() { # <组：e170 或 e171>：同一组只造一次，打出导出所在目录
  local directory="$OUT_DIR/e170-r2-exports-$1"
  (
    flock 9 || exit 1
    if [[ ! -f "$directory.done" ]]; then
      rm -rf "$directory" && mkdir -p "$directory" || exit 1
      "e170_r2_build_exports_$1" "$directory" || exit 1
      : >"$directory.done"
    fi
  ) 9>"$directory.lock" || return 1
  printf '%s\n' "$directory"
}
e170_r2_python() { # 在仓根下起 Python
  PYTHONDONTWRITEBYTECODE=1 python3 "$@"
}
e170_r2_bin() { printf '%s\n' "$PWD/scripts/experiment-apparatus/$1"; }
e170_r2_wrap() { # <原始输出> <产物>：改写成结果行再打出来
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates_journal_header_487)/run_replay_result_lines.py" wrap --input "$1" --output "$2" >&2 \
    && cat "$2"
}
driver_e170_r2_dump_compare() {
  local exports
  exports="$(e170_r2_shared_exports e170)" && cat "$exports/dump-compare.lines"
}
driver_e170_r2_bad_states_second_run_export() {
  local exports
  exports="$(e170_r2_shared_exports e170)" && cat "$exports/bad-states-second-run.lines"
}
e170_r2_unit_tests() { # <臂名> <执行器目录名>
  local scratch="$OUT_DIR/e170-r2-unit-tests-$1"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates_journal_header_487)/run_replay_result_lines.py" unit-tests \
    --executor-directory "$(e170_r2_bin "$2")" --arm "$1" --scratch "$scratch/test-scratch" --output "$scratch/result.out" >&2 || return 1
  cat "$scratch/result.out"
}
driver_e170_r2_v2_487_unit_tests() { e170_r2_unit_tests v2-487 e170_o3_spec_executor_with_witness_predicates_journal_header_487; }
driver_e170_r2_v1_487_unit_tests() { e170_r2_unit_tests v1-487 e170_second_run_o3_spec_executor_journal_header_487; }
# 单测层变异：表整张跑（XC 起头的行也在内，不排除）。
e170_r2_unit_level_mutations() { # <臂名> <执行器目录名>
  local scratch="$OUT_DIR/e170-r2-unit-level-mutations-$1" table="e170_o3_spec_executor_with_witness_predicates_journal_header_487"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates)/mutation_driver_unit_level.py" --executor-directory "$(e170_r2_bin "$2")" \
    --table "scripts/mutation/tables/$table.tsv" --scratch "$scratch/work" --output "$scratch/raw.out" >&2 || return 1
  e170_r2_wrap "$scratch/raw.out" "$scratch/result.out"
}
driver_e170_r2_v2_487_unit_level_mutations() { e170_r2_unit_level_mutations v2-487 e170_o3_spec_executor_with_witness_predicates_journal_header_487; }
driver_e170_r2_v1_487_unit_level_mutations() { e170_r2_unit_level_mutations v1-487 e170_second_run_o3_spec_executor_journal_header_487; }
driver_e170_r2_v2_487_z1() {
  local scratch="$OUT_DIR/e170-r2-v2-487-z1" exports
  exports="$(e170_r2_shared_exports e170)" || return 1
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates_journal_header_487)/executor_main.py" --dumps "$exports/e170-dump" \
    --workers 1 --output "$scratch/raw.out" >&2 || return 1
  e170_r2_wrap "$scratch/raw.out" "$scratch/result.out"
}
# 判镜像：冻结参照取第一段乙产物里不带 arm= 的那一组冻结行（同 run_segment_two_q5.sh），文件名照抄（config 行写它的名字）。
# 冻结参照那一份（第一段乙的产物）在 research/results/ 下、可以被归档删掉：不在时这两行记跑不了（文件头「驱动要读的输入」），不编、不导出。
e170_r2_judge_images() { # <输入名> <导出组> <含 bad-*.tsv 的导出目录名> [已知坏镜像目录名]
  local scratch="$OUT_DIR/e170-r2-v2-487-${1,,}-images" exports known=() freeze_product="research/results/e170-r2-o3-spec-executor-2026-10-04-s1b-executor.out"
  replay_inputs_present "${FUNCNAME[1]:-${FUNCNAME[0]}}" "冻结参照：${REPLAY_ARCHIVED_REMEDY}" "$freeze_product" || return $?
  exports="$(e170_r2_shared_exports "$2")" || return 1
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  grep -E '^name=freeze(_seed)? ' "$freeze_product" | grep -v ' arm=' >"$scratch/v2-487-freeze-reference.out" || return 1
  # 已知坏镜像：给了目录名就比（Z4）；没给的（Z3）这一份输入不适用，显式写 --known-bad-not-applicable（与 --known-bad-dir 互斥，结果行写 known_bad=not_applicable），不留空
  if [[ $# -ge 4 ]]; then known=(--known-bad-dir "$exports/$4"); else known=(--known-bad-not-applicable); fi
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates)/comparator_bad_image_judge.py" \
    --executor-directory "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates_journal_header_487)" --input-label "$1" \
    --dumps "$exports/$3" "${known[@]}" --freeze-reference "$scratch/v2-487-freeze-reference.out" --output "$scratch/raw.out" >&2 || return 1
  e170_r2_wrap "$scratch/raw.out" "$scratch/result.out"
}
driver_e170_r2_v2_487_z3_images() { e170_r2_judge_images Z3 e170 e170-bad-states-second-run; }
driver_e170_r2_v2_487_z4_images() { e170_r2_judge_images Z4 e171 e170-bad-states-copy-s2 known-bad-s2; }

# E171（三oracle合谋测试）第一段（登记 `research/prompts/e171-preregistration.md` 5.3、5.6、5.8「第一段」、第十一节 Z1 与 S2）：入库装置
# `e171_three_oracle_collusion` 与研究一侧（`scripts/experiment-apparatus/e171_three_oracle_collusion/`，Python，不是编译出来的代码，不经内存包装）。
# 每个驱动各带自己的临时目录（复跑是并发起的，不共用可写状态）。
# 产物只留 E7RESULT 行：不带耗时、不带路径（比对器的 config 行只写目录的名字），exact 比；耗时在 E7TIMING 行里，不入库。
# 第一段两份（baseline-run1、known-bad-images）不带 O3：O3 那几列要等 E170 第 1 次重跑冻结执行器，第二段才有。
# 比对器的 A4–A7 读今天的 `crates/mutations.tsv`（坏实现表 117 行按名字取、挑法脚本现算步长 8 与 4 的行数、改实现的行 844 行）：
# 表里别处加了核心实现的行、或改了那 117 行，这几行会变，不是装置坏了。
# E170 导出装置在 `bad-states` 模式、第一次的表（`bad-states`，不是 `bad-states-second-run`）上造 P 行与 N 行；第一段的 known-bad-images 用占位种子
# 0000000000000000（P、N 行与种子无关），第二段的盲测族用冻结执行器的种子（见下）。它在 E170 第 1 次重跑里加了第二套表，第一套的输出与加之前逐字节相同。
# 冻结执行器的冻结行参照（E170 第 1 次重跑的产物，research/results/ 下、可以被归档删掉）：第二段起读 O3 的几行驱动开跑前核它，不在记跑不了
# （文件头「驱动要读的输入」；Python 一侧 scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py 的 FreezeReferenceMissing 同一句）
E171_FREEZE_REFERENCE="research/results/e170-r1-o3-spec-executor-2026-10-03-s1-executor.out"
e171_device() {
  cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e171_three_oracle_collusion -- "$@"
}
e171_e170_exporter() {
  cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e170_o3_spec_executor_export -- "$@"
}
driver_e171_s1_baseline() {
  local scratch="$OUT_DIR/e171-baseline"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  (set -o pipefail && e171_device baseline --output-directory "$scratch/baseline-run1" | grep '^E7RESULT ')
}
driver_e171_s1_known_bad_images() {
  local scratch="$OUT_DIR/e171-known-bad"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  (set -o pipefail \
    && e171_device baseline --output-directory "$scratch/baseline-run1" >/dev/null \
    && cp -r "$scratch/baseline-run1" "$scratch/e170-bad-states-copy" \
    && e171_e170_exporter bad-states --dump-directory "$scratch/e170-bad-states-copy" --seed-hex 0000000000000000 >/dev/null \
    && e171_device known-bad-images --dump-directory "$scratch/e170-bad-states-copy" --output-directory "$scratch/known-bad" --e170-bad-rows | grep '^E7RESULT ')
}
# E171 第二段（登记 `research/prompts/e171-preregistration.md` 5.4、5.5、第十二节修订）：O3 用 E170 第 1 次重跑冻结的执行器
#（`scripts/experiment-apparatus/e170_second_run_o3_spec_executor/`，冻结行在 `e170-r1-o3-spec-executor-2026-10-03-s1-executor.out`；
# 盲测族种子取那份产物 `name=freeze_seed` 行的 85441ae491419411）。第一段的比对器与变异表产物（`e171-…-s1-comparator.out`、`…-s1-comparator-mutations.out`）
# 是第一段那一版比对器、九条变异跑出来的，留在 `research/results/` 里当历史；比对器、变异表在第二段改了，那两份不再复跑，由下面的 s2 两份取代。
# 坏镜像那三份（bad-states-export、known-bad-images、images-o3）共用「基线 → 拷一份 → E170 导出装置 bad-states（冻结的种子）」这三步，各带自己的临时目录。
# images-o3 的 config 行写两个目录名（e170-bad-states-copy-s2、known-bad-s2）与两个源文件的 sha256（本实验那份 `o3_bad_image_judge.py` 与它拷来的 E170
# `comparator_q3_bad_states.py`）：改了这两份源文件或目录名，这份产物就对不上，不是装置坏了。
e171_s2_prepare_bad_states() {
  local scratch="$1"
  (set -o pipefail \
    && e171_device baseline --output-directory "$scratch/baseline-run1" >"$scratch/baseline-run1.stdout" \
    && cp -r "$scratch/baseline-run1" "$scratch/e170-bad-states-copy-s2" \
    && e171_e170_exporter bad-states --dump-directory "$scratch/e170-bad-states-copy-s2" --seed-hex 85441ae491419411 >"$scratch/e170-bad-states.stdout")
}
e171_s2_known_bad() {
  local scratch="$1"
  (set -o pipefail \
    && e171_device known-bad-images --dump-directory "$scratch/e170-bad-states-copy-s2" --output-directory "$scratch/known-bad-s2" --e170-bad-rows >"$scratch/known-bad-images.stdout")
}
e171_s2_python() {
  PYTHONDONTWRITEBYTECODE=1 python3 "$@"
}
driver_e171_s2_bad_states_export() {
  local scratch="$OUT_DIR/e171-s2-bad-states"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" || return 1
  grep '^E7RESULT ' "$scratch/e170-bad-states.stdout"
}
driver_e171_s2_known_bad_images() {
  local scratch="$OUT_DIR/e171-s2-known-bad"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" && e171_s2_known_bad "$scratch" || return 1
  grep '^E7RESULT ' "$scratch/known-bad-images.stdout"
}
driver_e171_s2_images_o3() {
  local scratch="$OUT_DIR/e171-s2-images-o3"
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" && e171_s2_known_bad "$scratch" || return 1
  (e171_s2_python scripts/experiment-apparatus/e171_three_oracle_collusion/o3_bad_image_judge.py judge \
      --e170-dumps "$scratch/e170-bad-states-copy-s2" --known-bad-dir "$scratch/known-bad-s2" --output "$scratch/images-o3.out" >&2) || return 1
  grep '^E7RESULT ' "$scratch/images-o3.out"
}
driver_e171_s2_o3_control_baseline() {
  local scratch="$OUT_DIR/e171-s2-o3-control"
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" || return 1
  (e171_s2_python scripts/experiment-apparatus/e171_three_oracle_collusion/o3_bad_image_judge.py control \
      --dumps "$scratch/baseline-run1" --e170-dumps "$scratch/e170-bad-states-copy-s2" --row-name P1-F --output "$scratch/o3-control.out" >&2) || return 1
  grep '^E7RESULT ' "$scratch/o3-control.out"
}
# 比对器那一份（第二段）：基线两趟、E170 导出装置的 dump、冻结的执行器在基线上（A3 的 O3 一份）、已知坏镜像与 E170 的 P、N、盲测各行（O2 与 O3）、关一条（O1、O2）、
# 阳性对照四条（基线与空变异各一趟）、逐行驱动全表（不给选行参数；三条线道、每条线道两个线程）、
# Q3-核的机制试跑（第一刀的第 8 行，关 O1、关 O2、驱动不跑执行器各一趟）；目录的名字（baseline-run1、rows-s2-trial、disable-O1……）进产物，改了名产物就变。
driver_e171_s2_comparator() {
  local scratch="$OUT_DIR/e171-s2-comparator" dumps control name flag
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  dumps="$scratch/dumps"
  rm -rf "$scratch" && mkdir -p "$dumps" "$scratch/work" || return 1
  (set -o pipefail \
    && e171_device baseline --output-directory "$dumps/baseline-run1" >"$scratch/baseline-run1.stdout" \
    && e171_device baseline --output-directory "$dumps/baseline-run2" >/dev/null \
    && e171_e170_exporter dump --output-directory "$dumps/e170-dump" >/dev/null \
    && cp -r "$dumps/baseline-run1" "$dumps/e170-bad-states-copy-s2" \
    && e171_e170_exporter bad-states --dump-directory "$dumps/e170-bad-states-copy-s2" --seed-hex 85441ae491419411 >/dev/null \
    && e171_device known-bad-images --dump-directory "$dumps/e170-bad-states-copy-s2" --output-directory "$dumps/known-bad-s2" --e170-bad-rows >/dev/null \
    && e171_device baseline --output-directory "$dumps/disable-O1" --disable-oracle O1 >/dev/null \
    && e171_device baseline --output-directory "$dumps/disable-O2" --disable-oracle O2 >/dev/null \
    && e171_s2_python scripts/experiment-apparatus/e171_three_oracle_collusion/o3_bad_image_judge.py judge \
         --e170-dumps "$dumps/e170-bad-states-copy-s2" --known-bad-dir "$dumps/known-bad-s2" --output "$scratch/images-o3.out" >&2 \
    && e171_s2_python scripts/experiment-apparatus/e171_three_oracle_collusion/o3_bad_image_judge.py control \
         --dumps "$dumps/baseline-run1" --e170-dumps "$dumps/e170-bad-states-copy-s2" --row-name P1-F --output "$scratch/o3-control-baseline.out" >&2 \
    && (root="$PWD" && cd scripts/experiment-apparatus/e170_second_run_o3_spec_executor && PYTHONDONTWRITEBYTECODE=1 \
          python3 executor_main.py --dumps "$dumps/baseline-run1" \
          --workers 6 --output "$scratch/o3-baseline.out" >&2)) || return 1
  for control in O1-对拍 O1-崩溃判官 O2; do
    (e171_device baseline --output-directory "$dumps/control-baseline-$control" --positive-control "$control" >/dev/null \
      && PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
           --work-root "$scratch/work" --output-root "$dumps/rows-controls-$control" --empty-mutation --positive-control "$control" \
           >&2) || return 1
  done
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$scratch/work" --output-root "$dumps/rows-controls-O3" --empty-mutation --positive-control O3 \
      --o3-control-bad-states "$dumps/e170-bad-states-copy-s2" >&2 \
    && PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$scratch/work" --output-root "$dumps/rows-s2-trial" --empty-mutation --lanes 3 --threads 2 \
      >"$scratch/driver-tables.log") || return 1
  for name in O1 O2 no-executor; do
    flag="--disable-oracle ${name}"
    [[ "$name" == no-executor ]] && flag="--no-executor"
    # shellcheck disable=SC2086
    (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
        --work-root "$scratch/work" --output-root "$dumps/rows-s2-q3core-$name" --rows 8 $flag \
        | grep '^table ' >>"$scratch/driver-tables.log") || return 1
  done
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py --output "$scratch/comparator.out" \
      --mutations-table crates/mutations.tsv --baseline-dir "$dumps/baseline-run1" --baseline-device-output "$scratch/baseline-run1.stdout" \
      --baseline-repeat-dir "$dumps/baseline-run2" --e170-dump "$dumps/e170-dump" --o3-baseline "$scratch/o3-baseline.out" \
      --disabled O1:"$dumps/disable-O1" --disabled O2:"$dumps/disable-O2" \
      --rows-root "$dumps/rows-s2-trial" --driver-output "$scratch/driver-tables.log" \
      --known-bad-dir "$dumps/known-bad-s2" --e170-expected-dir "$dumps/e170-bad-states-copy-s2" --image-judgements "$scratch/images-o3.out" \
      --control "O1-对拍:baseline:$dumps/control-baseline-O1-对拍" --control "O1-对拍:empty_mutation:$dumps/rows-controls-O1-对拍/row-0/dump" \
      --control "O1-崩溃判官:baseline:$dumps/control-baseline-O1-崩溃判官" --control "O1-崩溃判官:empty_mutation:$dumps/rows-controls-O1-崩溃判官/row-0/dump" \
      --control "O2:baseline:$dumps/control-baseline-O2" --control "O2:empty_mutation:$dumps/rows-controls-O2/row-0/dump" \
      --control "O3:baseline:$scratch/o3-control-baseline.out" --control "O3:empty_mutation:$dumps/rows-controls-O3/row-0/o3-control.out" \
      --q3-core "O1:$dumps/rows-s2-trial:$dumps/rows-s2-q3core-O1" --q3-core "O2:$dumps/rows-s2-trial:$dumps/rows-s2-q3core-O2" \
      --q3-core "no_executor:$dumps/rows-s2-trial:$dumps/rows-s2-q3core-no-executor" >&2) || return 1
  grep '^E7RESULT ' "$scratch/comparator.out"
}
driver_e171_s3_comparator_mutations() {
  local scratch="$OUT_DIR/e171-s3-comparator-mutations" repository_root="$PWD"
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion" && PYTHONDONTWRITEBYTECODE=1 python3 mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion.tsv" --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  grep '^E7RESULT ' "$scratch/mutations.out"
}
# E171 s4 整表（登记修订 5）：第二段比对器那一串前置（基线两趟、E170 导出装置 dump 与 bad-states、已知坏镜像、关一条两趟、冻结执行器在基线与坏镜像上、
# 阳性对照四条在基线与空变异各一趟），加逐行驱动跑全部 117 行（两条线道、每条 2 线程，各趟都从同一份冻住的树拍快照）与 Q3-核三趟，再出比对器。
# 推的挂钟约两小时（s4 整表 5 569.6 秒、Q3-核三趟各约两分钟）。117 行的读数与 A4–A7 都读今天的 `crates/`：实现、O2、O1、装置或 `crates/mutations.tsv`
# 里那 117 行变了，产物就变，不是复跑坏了。目录的名字（baseline-run1、rows-s4、disable-O1、disable-O2）进产物，改了名产物就变。
driver_e171_s4_comparator() {
  local scratch="$OUT_DIR/e171-s4-comparator" dumps frozen control rows name flag
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  dumps="$scratch/dumps"
  frozen="$scratch/frozen-tree"
  rm -rf "$scratch" && mkdir -p "$dumps" "$scratch/work" "$frozen" || return 1
  replay_copy_closure_into "$frozen" || return 1
  (set -o pipefail \
    && e171_device baseline --output-directory "$dumps/baseline-run1" >"$scratch/baseline-run1.stdout" \
    && e171_device baseline --output-directory "$dumps/baseline-run2" >/dev/null \
    && e171_e170_exporter dump --output-directory "$dumps/e170-pinned-dump" >/dev/null \
    && cp -r "$dumps/baseline-run1" "$dumps/e170-bad-states-copy-s2" \
    && e171_e170_exporter bad-states --dump-directory "$dumps/e170-bad-states-copy-s2" --seed-hex 85441ae491419411 >/dev/null \
    && e171_device known-bad-images --dump-directory "$dumps/e170-bad-states-copy-s2" --output-directory "$dumps/known-bad-s2" --e170-bad-rows >/dev/null \
    && e171_device baseline --output-directory "$dumps/disable-O1" --disable-oracle O1 >/dev/null \
    && e171_device baseline --output-directory "$dumps/disable-O2" --disable-oracle O2 >/dev/null \
    && e171_s2_python scripts/experiment-apparatus/e171_three_oracle_collusion/o3_bad_image_judge.py judge \
         --e170-dumps "$dumps/e170-bad-states-copy-s2" --known-bad-dir "$dumps/known-bad-s2" --output "$scratch/images-o3.out" >&2 \
    && e171_s2_python scripts/experiment-apparatus/e171_three_oracle_collusion/o3_bad_image_judge.py control \
         --dumps "$dumps/baseline-run1" --e170-dumps "$dumps/e170-bad-states-copy-s2" --row-name P1-F --output "$scratch/o3-control-baseline.out" >&2 \
    && (root="$PWD" && cd scripts/experiment-apparatus/e170_second_run_o3_spec_executor && PYTHONDONTWRITEBYTECODE=1 \
          python3 executor_main.py --dumps "$dumps/baseline-run1" \
          --workers 6 --output "$scratch/o3-baseline.out" >&2)) || return 1
  for control in O1-对拍 O1-崩溃判官 O2; do
    (e171_device baseline --output-directory "$dumps/control-baseline-$control" --positive-control "$control" >/dev/null \
      && PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
           --work-root "$scratch/work" --snapshot-from "$frozen" --output-root "$dumps/rows-controls-$control" --empty-mutation \
           --positive-control "$control" >&2) || return 1
  done
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$scratch/work" --snapshot-from "$frozen" --output-root "$dumps/rows-controls-O3" --empty-mutation --positive-control O3 \
      --o3-control-bad-states "$dumps/e170-bad-states-copy-s2" >&2 \
    && PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$scratch/work" --snapshot-from "$frozen" --output-root "$dumps/rows-s4" --lanes 2 --threads 2 \
      --keep-compile-failures-as-they-are >"$scratch/table-driver.out") || return 1
  rows=$(PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py --print-q3-core-rows \
           --rows-root "$dumps/rows-s4" --baseline-dir "$dumps/baseline-run1" | grep -o 'q3_core_rows=[0-9,]*' | cut -d= -f2)
  [[ -n "$rows" ]] || return 1
  for name in O1 O2 no-executor; do
    flag="--disable-oracle ${name}"
    [[ "$name" == no-executor ]] && flag="--no-executor"
    # shellcheck disable=SC2086
    (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
        --work-root "$scratch/work" --snapshot-from "$frozen" --output-root "$dumps/rows-s4-q3core-$name" --rows "$rows" $flag --lanes 2 --threads 2 \
        >"$scratch/q3core-$name-driver.out") || return 1
  done
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py --output "$scratch/comparator.out" \
      --mutations-table crates/mutations.tsv --baseline-dir "$dumps/baseline-run1" --baseline-device-output "$scratch/baseline-run1.stdout" \
      --baseline-repeat-dir "$dumps/baseline-run2" --e170-dump "$dumps/e170-pinned-dump" --o3-baseline "$scratch/o3-baseline.out" \
      --disabled O1:"$dumps/disable-O1" --disabled O2:"$dumps/disable-O2" \
      --control "O1-对拍:baseline:$dumps/control-baseline-O1-对拍" --control "O1-对拍:empty_mutation:$dumps/rows-controls-O1-对拍/row-0/dump" \
      --control "O1-崩溃判官:baseline:$dumps/control-baseline-O1-崩溃判官" --control "O1-崩溃判官:empty_mutation:$dumps/rows-controls-O1-崩溃判官/row-0/dump" \
      --control "O2:baseline:$dumps/control-baseline-O2" --control "O2:empty_mutation:$dumps/rows-controls-O2/row-0/dump" \
      --control "O3:baseline:$scratch/o3-control-baseline.out" --control "O3:empty_mutation:$dumps/rows-controls-O3/row-0/o3-control.out" \
      --known-bad-dir "$dumps/known-bad-s2" --e170-expected-dir "$dumps/e170-bad-states-copy-s2" --image-judgements "$scratch/images-o3.out" \
      --rows-root "$dumps/rows-s4" --driver-output "$scratch/table-driver.out" \
      --q3-core "O1:$dumps/rows-s4:$dumps/rows-s4-q3core-O1" --q3-core "O2:$dumps/rows-s4:$dumps/rows-s4-q3core-O2" \
      --q3-core "no_executor:$dumps/rows-s4:$dumps/rows-s4-q3core-no-executor" >&2) || return 1
  grep '^E7RESULT ' "$scratch/comparator.out"
}
# E171 F4 扩表（登记 5.2「扩表」、第十节 F4）：先整条跑 driver_e171_s4_comparator（它的草稿目录留着 s4 的导出、冻住的树与编译目录），再在同一份冻住的树、
# 同一个工作根上跑扩表（`bad-implementations-step-4-only.tsv`，补位跳过步长 8 那张表 `bad-implementations-step-8.tsv` 的名字；E171 第 2 次重跑撤项之后 163 行），比对器把 rows-s4 与 rows-s4-f4 并成一张、
# 带 `--expansion-rows`、段名 s4-f4。推的挂钟约四小时（s4 那一串约两小时、扩表 164 行约两小时）。读数读今天的 `crates/`，同 s4 那一条的口径。
# 编不过的行补位那一趟（第八段起驱动在编不过之后照登记 5.2 补位）见下面的 driver_e171_s4_f4_substitutes_comparator。s4 与这一条的整表都带
# `--keep-compile-failures-as-they-are`：两份产物是编不过不补位的那一版驱动跑的，不带它，第 33、59、127、163、206、243 行会补位、产物就变。
driver_e171_s4_f4_comparator() {
  local s4="$OUT_DIR/e171-s4-comparator" dumps
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  driver_e171_s4_comparator >/dev/null || return 1
  dumps="$s4/dumps"
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$s4/work" --snapshot-from "$s4/frozen-tree" --output-root "$dumps/rows-s4-f4" \
      --selected-rows scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-step-4-only.tsv \
      --other-selected-rows scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-step-8.tsv --lanes 2 --threads 2 \
      --keep-compile-failures-as-they-are >"$s4/table-f4-driver.out") || return 1
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py --output "$s4/comparator-f4.out" \
      --mutations-table "$s4/frozen-tree/crates/mutations.tsv" --baseline-dir "$dumps/baseline-run1" --baseline-device-output "$s4/baseline-run1.stdout" \
      --baseline-repeat-dir "$dumps/baseline-run2" --e170-dump "$dumps/e170-pinned-dump" --o3-baseline "$s4/o3-baseline.out" \
      --disabled O1:"$dumps/disable-O1" --disabled O2:"$dumps/disable-O2" \
      --control "O1-对拍:baseline:$dumps/control-baseline-O1-对拍" --control "O1-对拍:empty_mutation:$dumps/rows-controls-O1-对拍/row-0/dump" \
      --control "O1-崩溃判官:baseline:$dumps/control-baseline-O1-崩溃判官" --control "O1-崩溃判官:empty_mutation:$dumps/rows-controls-O1-崩溃判官/row-0/dump" \
      --control "O2:baseline:$dumps/control-baseline-O2" --control "O2:empty_mutation:$dumps/rows-controls-O2/row-0/dump" \
      --control "O3:baseline:$s4/o3-control-baseline.out" --control "O3:empty_mutation:$dumps/rows-controls-O3/row-0/o3-control.out" \
      --known-bad-dir "$dumps/known-bad-s2" --e170-expected-dir "$dumps/e170-bad-states-copy-s2" --image-judgements "$s4/images-o3.out" \
      --segment s4-f4 --rows-root "$dumps/rows-s4" --rows-root "$dumps/rows-s4-f4" \
      --expansion-rows scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-step-4-only.tsv \
      --driver-output "$s4/table-driver.out" --driver-output "$s4/table-f4-driver.out" \
      --q3-core "O1:$dumps/rows-s4:$dumps/rows-s4-q3core-O1" --q3-core "O2:$dumps/rows-s4:$dumps/rows-s4-q3core-O2" \
      --q3-core "no_executor:$dumps/rows-s4:$dumps/rows-s4-q3core-no-executor" >&2) || return 1
  grep '^E7RESULT ' "$s4/comparator-f4.out"
}
# E171 编不过的 6 行补位（登记 5.2、第一节读法 5「替换之后编不过」）：先整条跑 driver_e171_s4_f4_comparator（它的草稿目录留着 s4 与 F4 的导出、冻住的树与编译目录；
# 两趟整表都带 `--keep-compile-failures-as-they-are`，所以第 33、59、127、163、206、243 行在两份输出根里的结局还是编不过），再在同一份冻住的树、同一个工作根上
# 跑补位那一趟（逐行驱动不再单列那六行：编不过的行由驱动按键复用时现判；原行先编，编不过就换同一文件组的下一行、至多三次，
# 补位跳过 117 行与扩表两张表的全部名字，补位那一趟不带 `--keep-compile-failures-as-they-are`），比对器把 rows-s4、rows-s4-f4 并上补位根（`--substitution-rows-root`）、段名 s4-f4-substitutes。
# 推的挂钟约四个半小时（F4 那一串约四小时、补位 6 行 1 277.9 秒）。读今天的 `crates/`，同 s4 那一条的口径；补位挑中哪几行取决于 `crates/mutations.tsv` 与那六行
# 今天编不编得过，表或实现变了换上的行就变、产物就变。目录的名字（rows-s4-f4-substitutes）进产物的 config 行。
driver_e171_s4_f4_substitutes_comparator() {
  local s4="$OUT_DIR/e171-s4-comparator" dumps
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  driver_e171_s4_f4_comparator >/dev/null || return 1
  dumps="$s4/dumps"
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/row_driver.py --repo-root "$PWD" \
      --work-root "$s4/work" --snapshot-from "$s4/frozen-tree" --output-root "$dumps/rows-s4-f4-substitutes" \
      --selected-rows scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-step-8.tsv \
      --other-selected-rows scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-step-4-only.tsv \
      --lanes 2 --threads 2 >"$s4/substitution-driver.out") || return 1
  (PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py --output "$s4/comparator-f4-substitutes.out" \
      --mutations-table "$s4/frozen-tree/crates/mutations.tsv" --baseline-dir "$dumps/baseline-run1" --baseline-device-output "$s4/baseline-run1.stdout" \
      --baseline-repeat-dir "$dumps/baseline-run2" --e170-dump "$dumps/e170-pinned-dump" --o3-baseline "$s4/o3-baseline.out" \
      --disabled O1:"$dumps/disable-O1" --disabled O2:"$dumps/disable-O2" \
      --control "O1-对拍:baseline:$dumps/control-baseline-O1-对拍" --control "O1-对拍:empty_mutation:$dumps/rows-controls-O1-对拍/row-0/dump" \
      --control "O1-崩溃判官:baseline:$dumps/control-baseline-O1-崩溃判官" --control "O1-崩溃判官:empty_mutation:$dumps/rows-controls-O1-崩溃判官/row-0/dump" \
      --control "O2:baseline:$dumps/control-baseline-O2" --control "O2:empty_mutation:$dumps/rows-controls-O2/row-0/dump" \
      --control "O3:baseline:$s4/o3-control-baseline.out" --control "O3:empty_mutation:$dumps/rows-controls-O3/row-0/o3-control.out" \
      --known-bad-dir "$dumps/known-bad-s2" --e170-expected-dir "$dumps/e170-bad-states-copy-s2" --image-judgements "$s4/images-o3.out" \
      --segment s4-f4-substitutes --rows-root "$dumps/rows-s4" --rows-root "$dumps/rows-s4-f4" --substitution-rows-root "$dumps/rows-s4-f4-substitutes" \
      --expansion-rows scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-step-4-only.tsv \
      --driver-output "$s4/table-driver.out" --driver-output "$s4/q3core-O1-driver.out" --driver-output "$s4/q3core-O2-driver.out" \
      --driver-output "$s4/q3core-no-executor-driver.out" --driver-output "$s4/table-f4-driver.out" --driver-output "$s4/substitution-driver.out" \
      --q3-core "O1:$dumps/rows-s4:$dumps/rows-s4-q3core-O1" --q3-core "O2:$dumps/rows-s4:$dumps/rows-s4-q3core-O2" \
      --q3-core "no_executor:$dumps/rows-s4:$dumps/rows-s4-q3core-no-executor" >&2) || return 1
  grep '^E7RESULT ' "$s4/comparator-f4-substitutes.out"
}
# E171 第 1 次重跑第一段·修订 6 那一趟（登记 `research/prompts/e171-r1-prereg.md` 修订 6–8 与其后各条「主 agent 认定」；第 0–10 步后缀 -r6-r3，
# 第 11–13 步 -resume2）：只在这一趟已有的导出与产物上重出，不起装置、不起 cargo、不碰别的台（研究一侧是 Python，不是编译出来的代码，不经内存包装）。
# 导出根是这一趟在本机的留存（不进仓）：编译树 scripts/experiment-apparatus/target/ 不当数据放处，没有默认，
# 重出到固定位置（工作根下 replay-frozen-roots/e171-r1-dumps）（下面 revision-6-2026-10-05-r3/、revision-6-segment-2/ 两个子目录照原样）。
# 比对器在这一趟自己的临时目录里搭的视图（e171_r1_view）里起，照产物里记的相对路径读：产物里写的路径逐字不变，仓里的 scripts/experiment-apparatus/target/ 一个字节不读。
# 导出根没给、它或要读的子目录与 research/results/ 下的产物缺了，这几行逐项记跑不了（文件头「驱动要读的输入」），不起比对器。
# 第 0–13 步本身（装置各趟、逐行驱动、O3 执行器、多机编排）要冻结树、真跑各行或第二台，推的挂钟几十小时，不登记。
E171_R1_DUMPS_REMEDY="导出根：照登记 research/prompts/e171-r1-prereg.md 修订 6–8 的第 0–13 步重出那一趟的导出，放到固定位置（工作根下 replay-frozen-roots/e171-r1-dumps，replay_frozen_root 取；不放 scripts/experiment-apparatus/target/）"
e171_r1_revision_6_python() { # 在当前目录（仓根或 e171_r1_view 搭的视图）起研究一侧的 Python
  (PYTHONDONTWRITEBYTECODE=1 nice -n 19 python3 -B "scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/$1" "${@:2}")
}
# e171_r1_inputs_present <这一行> <导出根下要读的子路径…> -- <research/results 下要读的产物…>：导出根没给只报它与产物，给了再逐个核子路径；缺的逐项报、返回 77
e171_r1_inputs_present() {
  local row="$1" dumps_root status=0 dump_paths=() product_paths=() seen_separator=0 path
  dumps_root="$(replay_frozen_root e171-r1-dumps)" || return 1
  shift
  for path in "$@"; do
    if [[ "$path" == -- ]]; then seen_separator=1; elif (( seen_separator )); then product_paths+=("$path"); else dump_paths+=("$dumps_root/$path"); fi
  done
  replay_inputs_present "$row" "$E171_R1_DUMPS_REMEDY" "$dumps_root" || status=$?
  if [[ -n "$dumps_root" && -d "$dumps_root" ]]; then
    replay_inputs_present "$row" "$E171_R1_DUMPS_REMEDY" ${dump_paths[@]+"${dump_paths[@]}"} || status=$?
  fi
  replay_inputs_present "$row" "留存产物：${REPLAY_ARCHIVED_REMEDY}" ${product_paths[@]+"${product_paths[@]}"} || status=$?
  return "$status"
}
# e171_r1_view <这一行的临时目录>：在里面搭视图 view/：research 与比对器目录连回仓里，scripts/experiment-apparatus/target/e171-r1-dumps 连到 E171_R1_REVISION_6_DUMPS，
# 固定位置（工作根下 replay-frozen-roots/e171-r1-frozen-tree-revision-6）在的话再把 scripts/experiment-apparatus/target/e171-r1-frozen-tree-revision-6 连过去；视图的路径写进全局 E171_R1_VIEW（不经命令替换带出来）。
e171_r1_view() {
  local experiments="$1/view/scripts/experiment-apparatus" dumps_root frozen_tree
  E171_R1_VIEW="$1/view"
  [[ "$E171_R1_VIEW" == /* ]] || E171_R1_VIEW="$PWD/$E171_R1_VIEW"
  dumps_root="$(replay_frozen_root e171-r1-dumps)" && dumps_root="$(realpath -- "$dumps_root")" || return 1
  rm -rf -- "${E171_R1_VIEW:?}" && mkdir -p "$experiments/target" || return 1
  ln -s "$PWD/research" "$1/view/research" && ln -s "$PWD/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" "$experiments/" \
    && ln -s "$dumps_root" "$experiments/target/e171-r1-dumps" || return 1
  frozen_tree="$(replay_frozen_root e171-r1-frozen-tree-revision-6)" || return 1
  if [[ -d "$frozen_tree" ]]; then
    frozen_tree="$(realpath -- "$frozen_tree")" && ln -s "$frozen_tree" "$experiments/target/e171-r1-frozen-tree-revision-6" || return 1
  fi
}
driver_e171_r1_s1_revision_6_comparator() {
  local scratch="$OUT_DIR/e171-r1-s1-revision-6-comparator" dumps="scripts/experiment-apparatus/target/e171-r1-dumps/revision-6-2026-10-05-r3"
  local prefix="research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1" control_name control_products=()
  for control_name in o1-comparison o1-comparison-unmount o1-crash-judge reference-filesystem o2 witness return-value memory-object panic o3-p1; do
    control_products+=("$prefix-positive-control-$control_name-r6-r3.out")
  done
  e171_r1_inputs_present "${FUNCNAME[0]}" revision-6-2026-10-05-r3/baseline-run1 revision-6-2026-10-05-r3/baseline-run2 revision-6-2026-10-05-r3/segment1-rows-resume2 \
    -- "$prefix-o3-baseline-run1-r6-r3.out" "$prefix-q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out" \
    "$prefix-q5-reverse-at-the-return-point-of-call-2-r6-r3.out" "${control_products[@]}" || return $?
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_r1_view "$scratch" || return 1
  (cd "$E171_R1_VIEW" && e171_r1_revision_6_python segment_one_comparator.py --baseline "$dumps/baseline-run1" --second-baseline "$dumps/baseline-run2" \
    --baseline-o3 "$prefix-o3-baseline-run1-r6-r3.out" --rows-root "$dumps/segment1-rows-resume2" \
    --q5-main "$prefix-q5-main-before-the-first-barrier-of-call-3-run-1-r6-r3.out" --q5-reverse "$prefix-q5-reverse-at-the-return-point-of-call-2-r6-r3.out" \
    --control "O1-对拍=$prefix-positive-control-o1-comparison-r6-r3.out" --control "O1-对拍-卸载=$prefix-positive-control-o1-comparison-unmount-r6-r3.out" \
    --control "O1-崩溃判官=$prefix-positive-control-o1-crash-judge-r6-r3.out" --control "RefFS=$prefix-positive-control-reference-filesystem-r6-r3.out" \
    --control "O2=$prefix-positive-control-o2-r6-r3.out" --control "见证=$prefix-positive-control-witness-r6-r3.out" \
    --control "交回值=$prefix-positive-control-return-value-r6-r3.out" --control "内存对象=$prefix-positive-control-memory-object-r6-r3.out" \
    --control "panic=$prefix-positive-control-panic-r6-r3.out" --control "O3=$prefix-positive-control-o3-p1-r6-r3.out" \
    --dumps-root "$dumps" --product-prefix e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1- --product-suffix=-r6-r3.out \
    --output "$E171_R1_VIEW/../comparator.out" >&2) || return $?
  cat "$scratch/comparator.out"
}
# 跨机一致对照（S17）的单独入口在已有的两份行目录上重出（「主 agent 认定：`-resume2` 跨机一致对照 S17 停机的处置」第 3、4 条）。入口只往标准输出写、
# 不写收尾行，照 e168_wrap 的先例在这里补 `name=done emitted=`：入库产物（…-2026-10-07-r3.out）就是补过的样子；它与 …-2026-10-07-r2.out 只差第 2 行的
# entry_sha256（入口改成读同目录里入库的编排器，04376c00… → afc9a9b0…），别的行逐字节相同。入口自己的 sha256 进产物，所以缺输入的那一判放在这里、不改入口。
driver_e171_r1_s1_revision_6_cross_host_check() {
  local scratch="$OUT_DIR/e171-r1-s1-revision-6-cross-host-check" dumps="scripts/experiment-apparatus/target/e171-r1-dumps/revision-6-2026-10-05-r3"
  local prefix="research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1"
  e171_r1_inputs_present "${FUNCNAME[0]}" revision-6-2026-10-05-r3/segment1-rows-resume2/row-0 revision-6-2026-10-05-r3/segment1-rows-resume2-cross-host/row-0 \
    revision-6-2026-10-05-r3/segment1-rows-resume2-shards/row-hosts.tsv \
    -- "$prefix-rows-driver-resume2-r6-r3.out" "$prefix-rows-cross-host-row-0-driver-resume2-r6-r3.out" || return $?
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_r1_view "$scratch" || return 1
  (cd "$E171_R1_VIEW" && e171_r1_revision_6_python cross_host_o3_check.py compare --repo . --step-12-row-0 "$dumps/segment1-rows-resume2/row-0" \
    --cross-host-row-0 "$dumps/segment1-rows-resume2-cross-host/row-0" --step-12-product "$prefix-rows-driver-resume2-r6-r3.out" \
    --cross-host-product "$prefix-rows-cross-host-row-0-driver-resume2-r6-r3.out" --ledger "$dumps/segment1-rows-resume2-shards/row-hosts.tsv") \
    >"$scratch/compare.out" || return 1
  awk '{print} END{print "E7RESULT name=done emitted=" NR + 1}' "$scratch/compare.out"
}
driver_e171_r1_s1_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-s1-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes.tsv" --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
# E171 第 1 次重跑第二段（整表）·修订 6 树：比对器只读已有的导出与合成产物重出（不起装置、不起 cargo、不碰别的台）。基线是修订 6 那一趟第 1 步
# S2 新装置基线，稳定性基线是同一趟第 7、9 步两次基线；行输出根 revision-6-segment-2/segment2-rows 是这一段在本机的留存（不进仓），
# 与第一段同在 固定位置（工作根下 replay-frozen-roots/e171-r1-dumps）的导出根下；冻结树的变异表在固定位置（工作根下 replay-frozen-roots/e171-r1-frozen-tree-revision-6）的冻结树里；
# 比对器同样在 e171_r1_view 搭的视图里照产物里记的相对路径读。
# 缺哪一样逐项记跑不了（文件头「驱动要读的输入」）。整表本身（281 行逐行改坏、编、跑装置，两台推约五个多小时）不登记。
driver_e171_r1_s2_revision_6_comparator() {
  local scratch="$OUT_DIR/e171-r1-s2-revision-6-comparator" dumps="scripts/experiment-apparatus/target/e171-r1-dumps" frozen_tree status=0
  frozen_tree="$(replay_frozen_root e171-r1-frozen-tree-revision-6)" || return 1
  local driver_product=research/results/e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6.out
  e171_r1_inputs_present "${FUNCNAME[0]}" revision-6-2026-10-05-r3/s2-device-baseline-streams-1-2 revision-6-2026-10-05-r3/baseline-run1 \
    revision-6-2026-10-05-r3/baseline-run2 revision-6-segment-2/segment2-rows -- "$driver_product" || status=$?
  replay_inputs_present "${FUNCNAME[0]}" "冻结树：照登记 research/prompts/e171-r1-prereg.md 修订 6 的冻结树清单重建一份，放到固定位置（工作根下 replay-frozen-roots/e171-r1-frozen-tree-revision-6，replay_frozen_root 取；不放 scripts/experiment-apparatus/target/）" \
    "${frozen_tree:+$frozen_tree/crates/mutations.tsv}" || status=$?
  (( status == 0 )) || return "$status"
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  e171_r1_view "$scratch" || return 1
  (cd "$E171_R1_VIEW" && e171_r1_revision_6_python segment_two_comparator.py --baseline "$dumps/revision-6-2026-10-05-r3/s2-device-baseline-streams-1-2" \
    --stability-baseline "$dumps/revision-6-2026-10-05-r3/baseline-run1" --stability-baseline "$dumps/revision-6-2026-10-05-r3/baseline-run2" \
    --rows-root "$dumps/revision-6-segment-2/segment2-rows" \
    --driver-product "$driver_product" \
    --mutations-table scripts/experiment-apparatus/target/e171-r1-frozen-tree-revision-6/crates/mutations.tsv \
    --dumps-root "$dumps/revision-6-2026-10-05-r3" --dumps-root "$dumps/revision-6-segment-2" \
    --product-prefix e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2- --product-suffix=-r6.out --table-rows 281 \
    --output "$E171_R1_VIEW/../comparator.out" >&2) || return $?
  cat "$scratch/comparator.out"
}
driver_e171_r1_s2_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-s2-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_segment_two.tsv" \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
driver_e171_r1_s1_revision_8_tools_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-s1-revision-8-tools-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_8.tsv" \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
driver_e171_r1_s2_revision_10_tools_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-s2-revision-10-tools-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_10.tsv" \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}

# E171 修订 11 的研究一侧变异表（这一个，与下面 driver_e171_second_rerun_current_state_library_root_research_mutations 那一张）带单测 test_revision_11 跑
# （两份登记的产物 config 行都是 unit_test_modules=test_revision_11）；那几条单测读写复用库，
# 复用库只在 prune-server 里、单测只在临时服务上跑（reuse_library.py 文件头「库」）：变异驱动整条套在临时服务底下起（客户端文件头「命令行」--with-temporary-service）
driver_e171_r1_revision_11_tools_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-revision-11-tools-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B \
      "$repository_root/scripts/prune-server/prune_server_client_selftest.py" --with-temporary-service -- python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_11.tsv" --unit-test-module test_revision_11 \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}

# E171 第 1 次重跑·修订 12（登记 `research/prompts/e171-r1-prereg.md` 修订 12）：入库装置 `stream-constants --streams 1,2,3` 在今天的 crates 上回比
# 三条流与 Q5 那一形的本地常量（S1）；只留 E7RESULT 行（这一模式不写导出、不带路径），exact 比。crates 改了写流这一行会红：照登记修订 8 第 8.2 节分类重取常量。
# 登记行指 E171 第 2 次重跑 S20 重钉之后那一份（research/prompts/e171-r2-prereg.md 修订 2 第 2.4 条、修订 4 第 4.6 条）；修订 12 那一份留在 git 里（59ce5a865^）。
driver_e171_r1_revision_12_stream_constants() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e171_three_oracle_collusion_after_the_oracle_fixes -- stream-constants --streams 1,2,3 | grep '^E7RESULT ')
}
# E171 第 2 次重跑（登记 research/prompts/e171-r2-prereg.md 第五节 5.3）：名单核对读工作区的 crates/mutations.tsv、改后的两张名单与开工那一刻的提交；
# 变异表或名单改了这一行会红（同 stream-constants 那一行，红了照登记读法 3 重分）。r1 逐行驱动产物从 git 对象库取（59ce5a865 里有它），文件名照原名，产物 config 行只写名字。
# 名单核对的 name=pinned_values 一行还载入同目录 count_reusable_units.py（它载入 reuse_library.py）：载入报错时那一格照实写进产物，产物随之不同。
# 四条登记行指登记修订 2 第 2.5 条 Z12 之后的重跑产物（名后接跑的那天）；作废那一趟的四份留在 research/results/，不登记复跑。
driver_e171_second_rerun_list_check() {
  local scratch="$OUT_DIR/e171-r2-list-check" repository_root="$PWD" driver_product=e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6.out
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  git show "59ce5a86550de2c2c9bc6b51b34b4f881313682c:research/results/$driver_product" >"$scratch/$driver_product" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B list_check.py \
      --repository "$repository_root" --reference-commit 59ce5a86550de2c2c9bc6b51b34b4f881313682c --r1-rows-driver-product "$scratch/$driver_product" \
      --output "$scratch/list-check.out" >&2) || return 1
  cat "$scratch/list-check.out"
}
# 撤项核算三份：读的比对器产物从 git 对象库取（原登记两份在 59ce5a865 的上一版、r1 第二段那份在 59ce5a865），名单核对产物读 research/results/。
e171_second_rerun_removal_effect() { # <调用它的驱动名> <kind> <取产物的提交> <产物名> <sha256> <行数>
  local row="$1" kind="$2" commit="$3" name="$4" digest="$5" count="$6" repository_root="$PWD"
  local scratch="$OUT_DIR/$row" list_check_product=research/results/e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-list-check-2026-11-01.out
  replay_inputs_present "$row" "名单核对产物：先跑 driver_e171_second_rerun_list_check 那一行" "$list_check_product" || return $?
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  git show "$commit:research/results/$name" >"$scratch/$name" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B removal_effect.py \
      --kind "$kind" --product "$scratch/$name" --expected-sha256 "$digest" --expected-lines "$count" \
      --list-check-product "$repository_root/$list_check_product" --output "$scratch/removal-effect.out" >&2) || return 1
  cat "$scratch/removal-effect.out"
}
driver_e171_second_rerun_removal_effect_on_the_whole_table_and_expansion_table() {
  e171_second_rerun_removal_effect "${FUNCNAME[0]}" original-registration 59ce5a86550de2c2c9bc6b51b34b4f881313682c^ \
    e171-three-oracle-collusion-2026-10-03-s4-f4-comparator.out e8660ccef12a13f76409a1b6f24ebd9a75444249ad65d5617551ac59c1e3c40e 4698
}
driver_e171_second_rerun_removal_effect_on_the_whole_table_and_expansion_table_substitutes() {
  e171_second_rerun_removal_effect "${FUNCNAME[0]}" original-registration 59ce5a86550de2c2c9bc6b51b34b4f881313682c^ \
    e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator.out 0f439cc2c86631a8d4c47a1f5da3205920b3f2bdb6e38684981a0f5b5cb5592a 4795
}
driver_e171_second_rerun_removal_effect_on_the_revision_6_segment_two() {
  e171_second_rerun_removal_effect "${FUNCNAME[0]}" segment-two 59ce5a86550de2c2c9bc6b51b34b4f881313682c \
    e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out d0bc79ca3f580774cfd4bad2fd672006b98c8a5d02063c48fb9915cb246a8549 451
}
driver_e171_second_rerun_list_removal_research_mutations() {
  local scratch="$OUT_DIR/e171-r2-list-removal-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_list_removal.tsv" \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
# E171 第 2 次重跑修订 4（登记 research/prompts/e171-r2-prereg.md）：现状整表段前置那几处的两张研究一侧变异表——重写入库的 s2_compare.py 与逐行编排外层
# H16、冻结树两处运行条件（单测 test_revision_8），复用库根两台同一（单测 test_revision_11）；结果行不带耗时，exact 比。
driver_e171_second_rerun_current_state_preconditions_research_mutations() {
  local scratch="$OUT_DIR/e171-r2-current-state-preconditions-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_preconditions.tsv" \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
driver_e171_second_rerun_current_state_library_root_research_mutations() {
  local scratch="$OUT_DIR/e171-r2-current-state-library-root-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B \
      "$repository_root/scripts/prune-server/prune_server_client_selftest.py" --with-temporary-service -- python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_library_root.tsv" \
      --unit-test-module test_revision_11 --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
# E171 第 2 次重跑修订 5（登记 research/prompts/e171-r2-prereg.md 第 5.6 条）：第 0 步判法读宣布行的行首与调度记录（单测 test_revision_8）；exact 比。
driver_e171_second_rerun_current_state_announcement_research_mutations() {
  local scratch="$OUT_DIR/e171-r2-current-state-announcement-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_announcement.tsv" \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
# E171 第 2 次重跑修订 6、修订 7、修订 8（登记 research/prompts/e171-r2-prereg.md）：线程额度在一条任务的进程里分的几处与任务里面不套线程包装（单测 test_thread_shares）、
# 第一段对照 ②③ 报造不回来与 ④ 没账时的台（单测 test_revision_8）；exact 比。
driver_e171_second_rerun_current_state_threads_and_contrast_research_mutations() {
  local scratch="$OUT_DIR/e171-r2-current-state-threads-and-contrast-research-mutations" repository_root="$PWD"
  rm -rf "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_threads_and_contrast.tsv" \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}

# E173 第 1 次重跑第一段（入库装置，重跑登记 research/prompts/e173-r1-prereg.md「装置写在哪」）：同 E156 的先例；
# `segment-one` 模式跑 9 个几何（S × 池大小）上的 κ 与一次空发布实际新占的固定点、两条路对拍、本地常量回比；
# 输出里只有一处与线程数相关：config 行的 `threads_of_the_geometries` = min(线程数, 21)，线程数取登记的每台默认重活线程数（31）时是 21。
driver_e173_checkpoint_cost() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e173_checkpoint_cost_against_empty_publish_cost -- segment-one)
}

driver_e156() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e156_allocation_basis_counts)
}

# E158（第一段，入库装置，跑前登记「装置写在哪」写死第 5 行）：同 E156 的先例，两个 cargo workspace
# 互相看不到对方，不能合并成一次调用。`all` 模式跑 5.7 常量回比、第七节锚点、S5 独立解码对拍、
# H3（岔路 3）四个几何点不注入全枚举、Q3-2/Q2-3 算术、PC3。
# 这一行承重 `e158-root-choice-repair-2026-09-25-segment1-rerun.out`。`…-2026-09-24-segment1-rerun.out` 与 `…-2026-09-23-segment1.out`（后者对应 C512（树表 0 条的一版上被换下的实例表记在哪没有条款） 落地之前的代码）
# 原样留着，不是这一行比对的对象。工作树 `crates/` 里 mount.rs、allocator.rs、allocation_record_tree.rs 一变，`pc3` 一行（`recover_verdict`、`recover_root`）会随之变，
# 对不上是新观测，先对着实验页与跑前登记「十二、修订」查。
driver_e158() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- all)
}

# E158 第二段 Q3-1（岔路 3 候选 3 那一半）：`q3-1-g0` 模式只跑 G0 几何上的
# H3 × Φ3（|F|≤2）违例枚举，不跑第一段的 H3 全枚举（那部分归 `driver_e158`）。
# 产物里 G0/S16/小环三点的 `pairs_with_any_violation` 是 0（`cold_recover_with_fault_failed`/`mount_writable_with_fault_failed` 也是 0：判定本身不再违例）；S4 那一点
# `pairs_with_any_violation` 是 0，`cold_recover_with_fault_failed`/`mount_writable_with_fault_failed` 是 239：同一批构造里有 239 个是「调用直接报错」，
# 与「成功但违例」是两种不同的失效形态。C332 正文钉的「2 个故障就能撤销回退」在这份 `crates/` 上测不出来，详见跑前登记「十二、修订」。
# 登记的产物是 `e158-root-choice-repair-2026-09-25-{q3-1-g0,q3-1-s16,q3-1-small-ring,q3-1-s4}.out`，`…-09-24-…` 原样留着、不登记。
driver_e158_q3_1_g0() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-g0)
}
# 第八节敏感性（行 3）三个取样点：S16、小环（环长现算，产物里的 `small_ring_search` 那行同时钉住取到的环长）、
# S4（`sigma_length_limit=4`，比其余三点多穷举一层，代价数量级最大，real 约 18 分钟）。
driver_e158_q3_1_s16() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-s16)
}
driver_e158_q3_1_small_ring() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-small-ring)
}
driver_e158_q3_1_s4() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-s4)
}
# E158 岔路单第 1 行（C393）：`q1-g0` 模式在今天的 `crates/`（候选 (c)）上跑 H1 家族 + Φ1 故障注入 +
# PC1-a/PC1-b，只跑 G0 几何（见实验页）。候选 (a)（A1 副本）的同一份数只存产物
# `e158-root-choice-repair-2026-09-24-q1-g0-a1-arm.out`，**不登记在这张表里**：它要在
# `scripts/mutation/tables/e158_arms.tsv` 描述的副本上重新编译才跑得出来，这张表假设「跑这一行就等于跑今天
# committed 的 crates/」，副本不满足这个假设；复跑它的步骤见实验页「复跑」一节。产物里
# 多了 171 格差集的分类诊断（`q1_2_subset_diff_pair*`，根因是 `RootRecord::instance_table` 没被走
# 全版本走到）与 op1 起三步挂载的持续/瞬时故障轨迹（`q1_1a_op1_trajectory`）。
# op1 有第三种变体（`MountWritableThenRaiseFloorTo`），对应 `q1_1a_raise_floor_trigger_summary`/`q1_1a_raise_floor_by_aspect_severity`（G0：
# `pairs=96 trigger_count=32 history_nodes_without_room=26`，both_copies 两个指称各 16/16 触发、
# disk0_only/disk1_only 各 0/16，与另外两种 op1 变体同一批构造上的模式一致），是真实数据、不是噪声。
# 同一份产物里还有随 `crates/`（`mount.rs`/`recovery.rs`）变的量：
# `q1_2_tree_table_crates_has_a_path`（`verify_named_units=false`）是「189+0」（不碰物理拷贝就能复算出指针的格数是 0）、`pc1_a` 的 `device_write_bytes` 是
# 353792（`AllocationRecordTreeNode` 写得更多，与 `q2_2a_g0` 那条同一根因），
# 这两条与 e158 装置本身无关，随 `crates/` 变。
# 登记的产物是 `e158-root-choice-repair-2026-09-25-{q1-g0-today,q1-s16,q1-s4}.out`，`…-09-24-…` 原样留着、不登记。
driver_e158_q1_g0() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q1-g0)
}
# E158 岔路单第 2 行 ①（C331 修法）：`q2-1-g0` 在今天的 `crates/`（丙 = 甲-jsn）上跑 H2 主族（op2=`mount_writable`，只 n1∈{0,1,2,3}、n2=1）的穷举
# 下界搜索。这条产物是在故障装配 bug 存在时跑出来的（`attempt_rootback_probe_and_advance` 一块盘只装得上一个故障目标，权重 ≥ 2 就可能漏装——见跑前登记「十二、修订」）：
# 修好之后同一个 n1 范围给出不同结果（甲-txg 臂是「k_min=4」，不是「0 命中」），这条产物不当「今天/丙 0 命中」的依据引用，只当 bug 在时的对照；
# 承重的是下面 `driver_e158_q2_1_g0_session_s5`。
driver_e158_q2_1_g0() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-1-g0)
}
# E158 岔路单第 2 行 ②（每条修法每次发布多写几字节）：`q2-2a-g0` 在今天的 `crates/` 上跑
# 固定脚本，按结构种类报每次发布写的字节。甲-txg 臂的同一份数只存产物，同上不登记在这张表里。
# `AllocationRecordTreeNode` 每次发布的 `write_calls`/
# `written_bytes` 较大（例如 `second_mount_row_publish` 是 `write_calls=10 written_bytes=163840`），
# 与 `q1_g0` 的 `pc1_a` 字节同一根因
# （`allocator.rs`/`allocation_record_tree.rs`，与 e158 装置本身无关）。
# 登记的产物是 `e158-root-choice-repair-2026-09-25-q2-2a-g0-today.out`，`…-09-24-…` 原样留着。
driver_e158_q2_2a_g0() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-2a-g0)
}
# E158 岔路单第 2 行 PC2 是阳性对照：复现判决 K2 的两个具体构造（甲-txg 4 个瞬时根槽
# 读失败、乙-只配置 0 个注入故障 + 1 个崩溃点），不靠穷举——见跑前登记「十二、修订」。
# 乙-只配置候选的同一份数只存产物（副本上的数，副本没有 `published_txg` 字段就编不过，不登记在这张
# 表里，复跑步骤见实验页与 `scripts/mutation/tables/e158_arms.tsv`）。
driver_e158_q2_1_pc2() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-1-pc2)
}
# E158 岔路单第 2 行 ①（C331 修法，承重）：`q2-1-g0` 在今天的 `crates/`（丙 = 甲-jsn）上跑 H2 主族，n1 范围是跑前登记 5.1 要求的
# {0,...,6}。甲-txg 臂的同一份数只存产物，同上不登记在这张表里（复跑步骤见实验页）。穷举按权重档边界停（`weight_ceiling`），
# 没有会在权重档中途停手的计数上限：n1=0..3 是完整穷举（`subsets_tried`==`full_space_subset_count`）；n1=4、
# 6 找到 `k_min=12`；n1=5 如实报
# `k_min=not_found_up_to_weight_ceiling`（完整空间 131072、只搜到权重 12 为止，未截断，见跑前登记
# 「十二、修订」）。与 `q2_1_pc2`/`q2_1_hc1`/`q2_1_hc1_lower_bound` 三行一样不受 `crates/` 里 `mount.rs`/`allocator.rs`/`allocation_record_tree.rs`（`q3-1`/`q1-g0`/`q2-2a-g0` 会读到的路径）变动的影响：
# H2 主族的穷举下界搜索走的是 `first_txg_of_new_instance`/`next_counter`/根环读取，不经过那几处，
# 行 2「够判」的结论不随这几处变。
driver_e158_q2_1_g0_session_s5() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-1-g0)
}
# 第八节几何敏感性（行 1）：岔路单第 1 行判决格 = Q1-1a 的 N_trig，S16（根环
# 大一倍）与 S4（根环小一半）两个方向相反的取样点，复用 H1 装置代码（`run_ledger_fault_family` 本身
# 就是几何参数化的）。两点都与 G0 逐字节等值（N_trig=798），判定不翻面，判别力自证「两点同值，自证
# 不适用」。
driver_e158_q1_s16() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q1-s16)
}
driver_e158_q1_s4() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q1-s4)
}
# H-C1 直接构造（岔路单第 2 行 ①，丙的具体历史）：判决 K2 说丙需要 12 个
# 故障（4 根槽 + 4 条记录各两块盘）才打得中；这里不靠 `run_rootback_tolerance_family` 的穷举，
# 直接按这个具体构造跑一次（hit=true, weight=12），
# 附一个只挡 4 条根槽、不挡记录的负对照（hit=false，证明「只挡根环挡不住丙」）。
driver_e158_q2_1_hc1() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-1-hc1)
}
# H-C1 下界探针（参数 weight_ceiling）：在同一个 n1=4 节点
# 上，穷举权重 0..11 的全部组合（`subsets_tried=22558`，`full_space_subset_count=32768`，没有撞
# 顶）都没有命中，权重 12 上第一个尝试的组合就命中。**这是「恰好 12」的严格证据**：不是构造上界，是
# 穷举下界真正走到了 12 且之下全空。不传第二个参数时 `weight_ceiling=None`——n1=4 这个节点的完整
# 空间只有 2^15=32768，在 `FEASIBLE_FULL_SEARCH_SUBSET_BUDGET`=100000 预算内，`None` 就等于穷举到
# 完整空间的顶，与手动调高到 60000 效果相同（60000 > 这段历史任何可能的权重值）。
driver_e158_q2_1_hc1_lower_bound() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q2-1-hc1-lower-bound)
}
# E158 第 2 次跑第一段（重跑登记 `research/prompts/e158-r2-prereg.md`）：`r2-all` 在工作树的 `crates/`（今天那一臂）上
# 跑第 2 次跑的开跑检查、H1d、Q1-0、PC1-a/b/c 与实七两段复现历史。另外七臂（A1、甲-txg、乙F-留环、乙 / 丁四臂）的产物
# `e158-root-choice-repair-2026-09-26-r2-all-<臂>.out` 要在 `scripts/mutation/tables/e158_arms.tsv` 套出来的副本上重编才跑得出来，
# 不登记在这张表里（同第一次跑 A1 产物的先例）；`…-r2-all-today-snapshot.out` 是同一装置在那几份副本共用的快照上跑的今天那一臂，
# 与这一行的产物只在实七崩溃注入那一段不同（快照之后 harness 的记录核对器改过，实验页写明）。
driver_e158_r2_all() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- r2-all)
}
# E158 第 3 次跑第一段（重跑登记 `research/prompts/e158-r3-prereg.md`）：`r2-all` 那一行的产物是在开工快照上重出的
# （工作树的 `crates/` 那一刻与快照逐字节相同，登记 S8）`…-2026-09-27-r2-all-today.out`，更早的产物原样留着。
# `r3-seg1` 在九份臂副本上各跑一遍，这里只登记今天那一臂（工作树的 `crates/` 就是它）；另外八臂的产物
# `e158-root-choice-repair-2026-09-27-r3-seg1-<臂>.out` 要在 `scripts/mutation/tables/e158_arms.tsv` 的 r3 行套出来的副本上重编才跑得出来，
# 不登记（同第 2 次跑的先例）。`r3-compare` 只读 `research/results/` 下九份产物、不碰 `crates/`，复跑逐字节可比。
# 产物头的 `E7INPUT name=crates_snapshot` 那一行是开工快照的汇总 sha256，比对前删掉。工作树的 `crates/` 与快照不同时，
# 这两行里读 `crates/` 的那一行（r3-seg1 今天、r2-all）会随之漂，不归这一次（登记第一节「上一次执行员交来的两件」）。
driver_e158_r3_seg1_today() {
  (SINGLEFS_E158_ARM=today cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- r3-seg1)
}
# r3-compare 与 r4-compare 读的臂产物在 research/results/ 下、可以被归档删掉（现查：r3 九臂只剩今天那一臂，r4 五段一份都不在）。装置那一侧缺一份时
# r3 打 verdict=stop 照退 0、r4 静默跳过一个字不报（crates/singlefs-verification/src/bin/e158_root_choice_repair.rs 的 run_third_run_compare、
# run_fourth_run_compare），这里开跑前照登记的臂逐份核，缺的逐项记跑不了（文件头「驱动要读的输入」），不起装置。
# 臂表照装置的 THIRD_RUN_ARMS 与留存产物 e158-root-choice-repair-2026-09-27-r4-compare.out 里 66 行 name=r4_compare_input 的 path= 抄：
# 五段各十二臂，seg5 另有六条 three-rounds 臂。
E158_THIRD_RUN_ARMS="today jia-cfg jia-cfg-carry jia-slot yi-cfg yi-cfg-carry yi-slot bing-cfg bing-cfg-carry"
E158_FOURTH_RUN_ARMS="today jia-cfg jia-cfg-carry jia-slot yi-cfg yi-cfg-carry yi-slot yi-narrow-cfg yi-narrow-cfg-carry yi-narrow-slot bing-cfg bing-cfg-carry"
E158_FOURTH_RUN_SEG5_EXTRA_ARMS="yi-cfg-three-rounds yi-cfg-carry-three-rounds yi-slot-three-rounds yi-narrow-cfg-three-rounds yi-narrow-cfg-carry-three-rounds yi-narrow-slot-three-rounds"
driver_e158_r3_seg1_compare() {
  local prefix=research/results/e158-root-choice-repair-2026-09-27-r3-seg1 arm inputs=()
  for arm in $E158_THIRD_RUN_ARMS; do inputs+=("$prefix-$arm.out"); done
  replay_inputs_present "${FUNCNAME[0]}" "臂产物：${REPLAY_ARCHIVED_REMEDY}；或照 scripts/mutation/tables/e158_arms.tsv 的 r3 行套出臂副本重跑 r3-seg1" "${inputs[@]}" || return $?
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- r3-compare research/results/e158-root-choice-repair-2026-09-27-r3-seg1)
}
# E158 第 4 次跑（重跑登记 `research/prompts/e158-r4-prereg.md`）：五段（seg1、seg2b、seg3、seg4、seg5）的臂产物都在
# `scripts/mutation/tables/e158_arms.tsv` 的 r4 行套出来的快照副本上编、跑，今天那一臂也是快照副本（工作树的 `crates/` 在快照之后
# 已合进 C554 乙，不再是「今天那一臂」），各段的臂产物都不登记（同第 2、3 次跑的先例）。这里只登记五段合并的 `r4-compare`：
# 它只读 `research/results/` 下那五组臂产物、不碰 `crates/`，复跑逐字节可比；第二段用 seg2b，
# 撞键之前那一份 seg2 不喂进来。产物头的 `E7INPUT name=crates_snapshot` 那一行比对前删掉。
driver_e158_r4_compare() {
  local prefix=research/results/e158-root-choice-repair-2026-09-27-r4 segment arm arms inputs=()
  for segment in seg1 seg2b seg3 seg4 seg5; do
    arms="$E158_FOURTH_RUN_ARMS"; [[ "$segment" == seg5 ]] && arms="$arms $E158_FOURTH_RUN_SEG5_EXTRA_ARMS"
    for arm in $arms; do inputs+=("$prefix-$segment-$arm.out"); done
  done
  replay_inputs_present "${FUNCNAME[0]}" "臂产物：${REPLAY_ARCHIVED_REMEDY}；或照 scripts/mutation/tables/e158_arms.tsv 的 r4 行套出快照副本重跑那一段" "${inputs[@]}" || return $?
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- r4-compare research/results/e158-root-choice-repair-2026-09-27-r4-seg1 research/results/e158-root-choice-repair-2026-09-27-r4-seg2b research/results/e158-root-choice-repair-2026-09-27-r4-seg3 research/results/e158-root-choice-repair-2026-09-27-r4-seg4 research/results/e158-root-choice-repair-2026-09-27-r4-seg5)
}

# E181（落得下那一判与挂载下界）第一段（入库装置，跑前登记 5.5 节「第一段」）：装置不带段名就跑全部 11 段——常量回比、第七节锚点、18 条自证、小几何的阳性对照，
# 以及长段（F9 的边界二分与窗口、F10 的走种子、F11 的挂钟，一趟要一两个小时）；F11 带计时，所以这一行 timing 比结构，段用时打到标准错误。
# 长跑的分片起法在 scripts/experiment-apparatus/e181-landing-check-long-run.sh 文件头，它拼出的产物就是这条登记比的产物。
driver_e181() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e181_landing_check_fixed_point_and_mount_lower_bound)
}

# E182（被抛弃根读不出的三条岔路）第一段（入库装置，跑前登记 5.4「单盘择根段」）：`single-device` 模式打整份产物，没有计时字段。
# 片结果按复用键存在片结果库里（装置文件头「复用键」）：复跑时键相同的片直接取、产物逐字节相同；库不在就全部现判。
# 长跑的起法（分片、多机、产物另存的名字）在 scripts/experiment-apparatus/e182-single-device-long-run.sh 文件头。
#   装置读写的片结果库目录是这一趟的中转目录（文件头「E182 的片结果库」）：拉与并回在驱动外面做，这里只设 E182_VERDICT_LIBRARY。
driver_e182() {
  ([[ "$REPLAY_BROKEN" == e182-transit-unset ]] || export E182_VERDICT_LIBRARY="$REPLAY_E182_SHARDS/e182-single-device"
   cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e182_unreadable_abandoned_root_forks -- single-device)
}

# E182 第一段补臂「覆盖写保持 s 个单元」（登记修订 12.11）：`single-device-units-kept` 模式打整份产物，没有计时字段；复用与长跑起法同上，
# 长跑脚本是 scripts/experiment-apparatus/e182-single-device-units-kept-long-run.sh。
driver_e182_units_kept() {
  ([[ "$REPLAY_BROKEN" == e182-transit-unset ]] || export E182_VERDICT_LIBRARY="$REPLAY_E182_SHARDS/e182-single-device-units-kept"
   cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e182_unreadable_abandoned_root_forks -- single-device-units-kept)
}

# E162（崩溃放量判定块存储选型）：只登记确定性的 anchors（跑前登记 7.2 的锚点）。S1、S2 与几何取样点的产物带杀点与计时，
# 两次跑本来就不同，不登记逐字节比对，复跑命令写在实验页。这个 bin 挂在不默认打开的特性 e162-block-stores 上
# （librocksdb-sys 要现编 C++），它的 bindgen 在本机要 BINDGEN_EXTRA_CLANG_ARGS 指到 gcc 的 stdbool.h 才编得过；开跑前经 build_research_driver_binary 编（编不过的输出进这一行的 .err）。
driver_e162_anchors() {
  BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--I/usr/lib/gcc/x86_64-linux-gnu/13/include}" \
    build_research_driver_binary e162-block-stores e162-crash-verdict-block-store || return 1
  "$REPLAY_RELEASE_DIRECTORY"/e162-crash-verdict-block-store anchors
}

# E162 第四段 S4 掉电装置：只登记确定性的 selftest（登记 7.2 A1–A6 与补 7 B 系列锚点）。s4-drive 起 QEMU 虚机，
# 不登记逐字节复跑；它的整条命令写在实验页。同一个特性、同一个 BINDGEN 变量，照上面 E162 的写法。
driver_e162_power_cut_selftest() {
  BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--I/usr/lib/gcc/x86_64-linux-gnu/13/include}" \
    build_research_driver_binary e162-block-stores e162-verdict-store-power-cut || return 1
  "$REPLAY_RELEASE_DIRECTORY"/e162-verdict-store-power-cut --role selftest
}

# E162 第一、二段全部臂（S1、S2：臂 R1、R0、K、F 按次序各起一次登记档，输出接成一份；`feasibility` 是缩域，复跑表不跑）。同一个特性、同一个 BINDGEN 变量，照上面 E162 的写法。
e162_arm_segments() { # <s1|s2>
  local arm
  BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--I/usr/lib/gcc/x86_64-linux-gnu/13/include}" \
    build_research_driver_binary e162-block-stores e162-crash-verdict-block-store || return 1
  for arm in R1 R0 K F; do
    "$REPLAY_RELEASE_DIRECTORY"/e162-crash-verdict-block-store "$1" "$arm" || return 1
  done
}
driver_e162_s1_all_arms() { e162_arm_segments s1; }
driver_e162_s2_all_arms() { e162_arm_segments s2; }
# E162 第四段 S4 掉电装置全部臂：`s4-drive` 不给 `--arms` 就是 R1、R0、K、F 四臂全跑（起 QEMU 虚机）。
driver_e162_s4_all_arms() {
  BINDGEN_EXTRA_CLANG_ARGS="${BINDGEN_EXTRA_CLANG_ARGS:--I/usr/lib/gcc/x86_64-linux-gnu/13/include}" \
    build_research_driver_binary e162-block-stores e162-verdict-store-power-cut || return 1
  "$REPLAY_RELEASE_DIRECTORY"/e162-verdict-store-power-cut s4-drive
}

# E163（GPU多卡算单元校验和）：只登记 R1（单机两张卡）的 merge 输出——它是唯一不含计时字段、
# 逐字节可比的一段（每片结果文件自己带 build_device_ms 等计时字段，两次跑不同，不登记）。
# 这个 bin 挂在不默认打开的特性 e163-gpu 上（wgpu、pollster，只有这个 bin 用），装置本身不用
# e7_index_bench::Emitter 的 E7RESULT 协议，这里在驱动函数里把 merge 的原始输出包一层，
# 满足下面「跑不了」判定要的 E7RESULT / name=done 收尾行；要机器上至少两张能起上下文的 NVIDIA 独显卡。
# 开跑前经 build_research_driver_binary 编（编不过的输出进这一行的 .err）。
driver_e163_r1_merge() {
  build_research_driver_binary e163-gpu e163-gpu-multicard-crc32c || return 1
  local bin="$REPLAY_RELEASE_DIRECTORY"/e163-gpu-multicard-crc32c
  local input="$OUT_DIR/e163-r1-merge-input.bin"
  local shard0="$OUT_DIR/e163-r1-merge-shard0.out"
  local shard1="$OUT_DIR/e163-r1-merge-shard1.out"
  "$bin" gen-input --output "$input" --units 2560 --seed 0xE163092700000001 >/dev/null || return 1
  "$bin" run-shard --input "$input" --start 0 --count 1280 --adapter-index 0 --output "$shard0" >/dev/null || return 1
  "$bin" run-shard --input "$input" --start 1280 --count 1280 --adapter-index 1 --output "$shard1" >/dev/null || return 1
  local merge_output line_count
  merge_output="$("$bin" merge --original-input "$input" --total-units 2560 --shard "$shard0" --shard "$shard1")" || return 1
  line_count=$(printf '%s\n' "$merge_output" | grep -c .)
  printf '%s\n' "$merge_output" | awk '{print "E7RESULT name=merge_line body=" $0}'
  echo "E7RESULT name=done emitted=$((line_count + 1))"
}

# E174（借据丢掉时等待与显存回还）：只登记不起卡、不含计时字段、逐字节可比的输出——`checks`（模型卡上的自检与清单检查）
# 和 `judge` 读产物的判决（模型卡上跑出来的第二段产物 `run --simulated`；真卡两段产物见 driver_e174_segment_one_judge）；
# 真卡上的产物本身含计时与显存读数，两次跑不同，不在这里复跑。
# 这个 bin 挂在不默认打开的特性 e174-gpu 上（wgpu、pollster），开跑前经 build_research_driver_binary 编（编不过的输出进这一行的 .err）；这两个子命令不碰卡。
driver_e174_checks() {
  build_research_driver_binary e174-gpu e174-lease-drop-wait-memory-return || return 1
  "$REPLAY_RELEASE_DIRECTORY"/e174-lease-drop-wait-memory-return checks
}

driver_e174_simulated_judge() {
  replay_inputs_present "${FUNCNAME[0]}" "判的那一份留存产物：${REPLAY_ARCHIVED_REMEDY}" research/results/e174-lease-drop-wait-memory-return-simulated-second-2026-10-08.out || return $?
  build_research_driver_binary e174-gpu e174-lease-drop-wait-memory-return || return 1
  "$REPLAY_RELEASE_DIRECTORY"/e174-lease-drop-wait-memory-return judge \
    --input research/results/e174-lease-drop-wait-memory-return-simulated-second-2026-10-08.out --scope second
}

# E174 两段真卡产物的判（`judge` 读产物、确定性、不碰卡）：真卡产物本身含计时与显存读数，两次跑不同，不在这里复跑，
# 登记的是判法对着入库产物的输出（第一段 `--scope first`，第二段 `--scope second`）。三个 judge 读的产物在 research/results/ 下、可以被归档删掉：
# 不在时那一行记跑不了（文件头「驱动要读的输入」），不编、不跑。
driver_e174_segment_one_judge() {
  replay_inputs_present "${FUNCNAME[0]}" "判的那一份留存产物：${REPLAY_ARCHIVED_REMEDY}" research/results/e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out || return $?
  build_research_driver_binary e174-gpu e174-lease-drop-wait-memory-return || return 1
  "$REPLAY_RELEASE_DIRECTORY"/e174-lease-drop-wait-memory-return judge \
    --input research/results/e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out --scope first
}

driver_e174_segment_two_judge() {
  replay_inputs_present "${FUNCNAME[0]}" "判的那一份留存产物：${REPLAY_ARCHIVED_REMEDY}" research/results/e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out || return $?
  build_research_driver_binary e174-gpu e174-lease-drop-wait-memory-return || return 1
  "$REPLAY_RELEASE_DIRECTORY"/e174-lease-drop-wait-memory-return judge \
    --input research/results/e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out --scope second
}

# E174 真卡两段整段跑（`run --segment first|second --output <文件>`）：产物含计时与显存读数，timing 比结构；写进 --output 的文件，这里原样打出来。
#   上面三个 judge 读的就是这两份产物（research/results/ 下，不在树里时 replay_inputs_present 从 git 历史取回）。
#   装置的 run 要代码键（--code-key-file）与取键脚本放进环境的装置路径，一律经 device-code-key.py run 起（装置文件头「子命令」那一段的写法）：它先取键、编（带 e174-gpu）、起装置、
#   装置读写轮库经它转给 prune-server；产物文件先删（已在而内容不同装置退 2）。
e174_run_segment() { # <first|second>
  local output="$OUT_DIR/e174-run-$1.out"
  rm -f -- "$output"
  python3 scripts/experiment-apparatus/device-code-key.py run . --binary e174-lease-drop-wait-memory-return --feature e174-gpu \
    --entry e7_index_bench::run_child_on_the_card --entry e7_index_bench::ChildJob::child_arguments \
    -- run --segment "$1" --output "$output" >&2 || return 1
  cat "$output"
}
driver_e174_run_segment_one() { e174_run_segment first; }
driver_e174_run_segment_two() { e174_run_segment second; }

# E178 第四段（登记修订 2 的两条新底座臂，997 个工作项读写项库）：装置要代码键（--code-key-file），一律经 device-code-key.py run 起（装置文件头「用法」那一行的写法），
# 项库经它转给 prune-server：库里键相同的项复用、不同的现判；产物按工作项次序输出，与读库还是现判无关，exact 比。
driver_e178_segment_four() {
  python3 scripts/experiment-apparatus/device-code-key.py run . --binary e178-leaf-and-record-death-counts \
    --entry e7_index_bench::segment_four_item_lines -- segment-four
}

# E172（删除写出的字节的干跑；第 3 次重跑登记 research/prompts/e172-r3-prereg.md 第五节 5.7 第 5 步）：形照 driver_e142——
# 先在仓根现跑 crates/ 侧只读导出 deletion_write_dump 两次进两个临时文件（不带参数的 H1、H2 一份，`-- --history h3` 一份），
# 再让研究侧独立解码器按「H1H2 在前、H3 在后」读它们。产物只是解码器的输出（入库的两份导出是
# research/results/e172-r3-deletion-write-dump-h1h2-2026-11-02.out 与 research/results/e172-r3-deletion-write-dump-h3-2026-11-02.out，不 cat 进产物）；
# 解码器产物先打准入的输入指纹行（`.claude/gate.d/stage-inputs.tsv` 的 E172 行），其后 `name=export_file file=h1h2 / h3` 各带一份导出的 sha256，哪一份导出一变那一行就对不上。
# 第 2 次产物 e172-r2-deletion-write-dry-run-2026-10-05.out 在 59ce5a865 归档删了、不再登记复跑：退役树加进 crates/ 之后导出变了，它复现不出来。
driver_e172() {
  local h1h2_snapshot="$OUT_DIR/e172-r3-deletion-write-dump-h1h2.tmp"
  local h3_snapshot="$OUT_DIR/e172-r3-deletion-write-dump-h3.tmp"
  (cargo run -q -p singlefs-verification --bin deletion_write_dump) >"$h1h2_snapshot" || return 1
  (cargo run -q -p singlefs-verification --bin deletion_write_dump -- --history h3) >"$h3_snapshot" || return 1
  "$REPLAY_RELEASE_DIRECTORY"/e172-deletion-write-dry-run "$h1h2_snapshot" "$h3_snapshot" || return 1
}

# E175（产品路径上 inode 号水位退回与重发号）：仓副本装置（跑前登记 research/prompts/e175-preregistration.md 第五节「装置写在哪」写死，
# 形式化线冻结期主工作区 crates/ 不动），装置源码在 scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs
# （反复要用的装置不放 research/，
# .claude/rules/directory-layout.md；sha256 cdb5d7be…）。驱动照共用库的闭包把今天的工作区拷进
# $OUT_DIR 下的副本（replay_copy_closure_into，不写清单）、放进那份源码、编一次，跑 D 族 P 臂（8 段，D 族结果行不带挂钟），只留 E7RESULT 行并补收尾行（闸 2）。
driver_e175() {
  local scratch="$OUT_DIR/e175-repository-copy" device_source="$PWD/scripts/experiment-apparatus/e175_product_path_inode_watermark_regression/device-source.rs" binary worker_threads
  rm -rf "$scratch" && mkdir -p "$scratch" || return 1
  replay_copy_closure_into "$scratch" || return 1
  cp "$device_source" "$scratch/crates/singlefs-verification/tests/e175_product_path_inode_watermark_regression.rs" || return 1
  (cd "$scratch" && unset CARGO_TARGET_DIR && nice -n 19 cargo test -q --release -p singlefs-verification --test e175_product_path_inode_watermark_regression --no-run >&2) || return 1
  binary="$(find "$scratch/target/release/deps" -maxdepth 1 -type f -executable -name 'e175_product_path_inode_watermark_regression-*' | head -n 1)"
  [[ -n "$binary" ]] || return 1
  # 装置没给 E175_WORKER_THREADS 时取本机可用并行度（装置源码冻结、不改），这里交每台默认重活线程数
  worker_threads="$(heavy_thread_default)" || return 1
  E175_ARM=P E175_FAMILY=D E175_RESULT_FILE="$scratch/d-P.out" E175_WORKER_THREADS="$worker_threads" \
    "$binary" --exact run_one_family_on_one_arm_into_the_result_file >&2 || return 1
  awk '/^E7RESULT /{print; n++} END{print "E7RESULT name=done emitted=" n + 1}' "$scratch/d-P.out"
}

# E183（容量斜率、读缓存、小盘准入与清扫阈值的代价数）第一段（入库装置，跑前登记 research/prompts/e183-preregistration.md「装置写在哪」写死）：
# H4 的十项在一个进程里按序跑完（`all`），产物不带耗时（每项耗时与退出码打到标准错误），exact 比。先例同 E156、E158。
driver_e183() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e183_capacity_read_cache_reserve_and_sweep_costs -- all)
}

# E183 第一段 A31 的独立核对（纯 Python，不读 crates/）：锚点脚本现算每份读数的单元区容量与环窗口字节，与上面那份产物逐份比；
# 读的产物是 driver_e183 的登记产物（research/results/ 下，不是这一趟新跑出来的）。
driver_e183_anchor_check() {
  local directory="scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs"
  python3 -B -I "$directory/small_disk_anchor_check.py" "$directory/anchors_e183.py" research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-10-30.out
}

# E184（常规层与大负载层的代价计数）第一段（入库装置，跑前登记 research/prompts/e184-preregistration.md「装置写在哪」写死）：
# 只登记确定性的那一片：`selfcheck`（停机 T1 至 T4 与第七节锚点，产物里没有计时与线程数）。整份第一段产物（18 件）带 keys_seconds、seconds 等挂钟字段，
# 两次跑本来就不同，不登记逐字节比对；长跑的起法在 scripts/experiment-apparatus/e184-segment-one-long-run.sh 文件头，复跑命令写在实验页。
# 装置这一档的收尾行是 name=selfcheck_done，没有 name=done：照 E175 的先例在这里补 `name=done emitted=`，留存产物就是补过的样子。
# 锚 B1 对不上是登记预期（selfcheck 在这一格退 0）；停机检查或 A 开头的锚点没过装置退 3，管道带 pipefail，这一行判红。
driver_e184() {
  (cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e184_regular_layer_cost_counts -- selfcheck | awk '/^E7RESULT /{print; n++} END{print "E7RESULT name=done emitted=" n + 1}')
}

# E184 第一段整份（长跑脚本 e184-segment-one-long-run.sh 不带参数：本机把没做完的件按次序全做完、拼出产物）。工作目录与拼出的产物都放本轮输出目录里
# （E184_WORK_DIRECTORY、E184_PRODUCT_DIRECTORY：长跑脚本把拼出的产物以「e184-regular-layer-cost-counts-segment-one-<日期>.out」写进产物目录），
# 装置用这棵树里编的（--build-here），不跨趟留；产物带挂钟字段，timing 比结构。
driver_e184_segment_one() {
  local product_directory="$OUT_DIR/e184-segment-one-product"
  rm -rf "$product_directory" && mkdir -p "$product_directory" || return 1
  E184_WORK_DIRECTORY="$OUT_DIR/e184-segment-one-work" E184_PRODUCT_DIRECTORY="$product_directory" \
    bash scripts/experiment-apparatus/e184-segment-one-long-run.sh --build-here >&2 || return 1
  cat "$product_directory"/e184-regular-layer-cost-counts-segment-one-*.out
}

# E183 第三段 H5 的独立核对（纯 Python，不读 crates/）：锚点脚本现算 24 格阈值历史的挂载次数、两次清扫、行数峰值、片数峰值与终了 txg，
# 与 H5 产物逐格比，加「交给后台」对照一格；读的产物是 research/results/ 下的 H5 产物（两台长跑的合并产物，含挂钟字段，所以产物本身不登记逐字节复跑）。
# 长跑的起法在 scripts/experiment-apparatus/e183-sweep-history-shards.sh 文件头。
driver_e183_h5_anchor_check() {
  local directory="scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs"
  local product="research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19.out"
  replay_inputs_present "${FUNCNAME[0]}" "H5 产物：${REPLAY_ARCHIVED_REMEDY}" "$product" || return $?
  python3 -B -I "$directory/sweep_history_anchor_check.py" "$directory/anchors_e183.py" "$product"
}

# E198（取号跳过认出生代号的前提）第一段与第二段（入库装置，跑前登记 research/prompts/e198-preregistration.md 第五节「装置写在哪」写死）：
# 一段一个进程，历史全部判完之后按历史名的次序一次打出，产物里没有计时（进度与段用时打到标准错误），exact 比。
# 配置行报线程数：线程数照登记值（跑前登记第十四节），装置没设 E198_THREADS 时取每台默认重活线程数（task-server 设的 SINGLEFS_THREAD_CAP，再没设取 .claude/heavy-thread-default），这里不写死；换了线程数产物照样逐行相同、只有配置行那两格不同。
driver_e198() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e198_instance_table_pointer_birth_skip_premise -- segment-one | grep '^E7RESULT ')
}

driver_e198_segment_two() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e198_instance_table_pointer_birth_skip_premise -- segment-two | grep '^E7RESULT ')
}

# E205（多处读坏倒挂三条改法的代价）三段（入库装置，跑前登记 research/prompts/e205-preregistration.md 第五节「装置写在哪」写死）：
# 一段一个进程，全部判完之后按件号的次序一次打出，产物里没有计时（进度打到标准错误），exact 比。
# 配置行报线程数：装置没设 E205_THREADS 时取每台默认重活线程数（task-server 设的 SINGLEFS_THREAD_CAP，再没设取 .claude/heavy-thread-default），这里不写死；换了线程数只有配置行那一格不同。
driver_e205() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e205_c971_inverted_first_txg_arm_costs -- six-cells | grep '^E7RESULT ')
}

driver_e205_torn_rotation() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e205_c971_inverted_first_txg_arm_costs -- torn-rotation | grep '^E7RESULT ')
}

driver_e205_first_txg_skips_s8() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e205_c971_inverted_first_txg_arm_costs -- first-txg-skips s8 | grep '^E7RESULT ')
}

driver_e205_first_txg_skips_s4_s16() {
  (set -o pipefail && cargo run -q --release -p singlefs-verification ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e205_c971_inverted_first_txg_arm_costs -- first-txg-skips s4-s16 | grep '^E7RESULT ')
}

# --admission-only：只问准入（scripts/gate/admission.py，登记表 .claude/gate.d/stage-inputs.tsv），不编、不跑、不比。
ADMISSION_ONLY=0
ONLY=()
for replay_argument in "$@"; do
  if [[ "$replay_argument" == "--admission-only" ]]; then ADMISSION_ONLY=1
  else ONLY+=("$replay_argument"); fi
done
# 替换表按 E103 这种带 E 的形态登记；裸数字会匹配 0 条并报全零。
for wanted in ${ONLY[@]+"${ONLY[@]}"}; do
  if [[ "$wanted" =~ ^[0-9]+$ ]]; then
    echo "  ✗ 实验号 $wanted 没带 E：这样匹配不到任何一行，跑出来全是零" >&2
    echo "     → 怎么办：写成 E$wanted，例：bash scripts/experiment-apparatus/replay.sh E$wanted" >&2
    exit 2
  fi
done
want() { [[ ${#ONLY[@]} -eq 0 ]] && return 0; local e; for e in "${ONLY[@]}"; do [[ "$e" == "$1" ]] && return 0; done; return 1; }

# 产物列是 research/results/ 底下的纯文件名，不是仓库根起的路径：写成路径时下面拼出 research/results/research/results/… 指不到文件，
# 而 diff 失败会被报成「对不上，N 行不同」——看着像产物变了，其实是登记表坏了。
bad_rows=$(printf "%s\n" "$TABLE" | awk -F"|" '/^E[0-9]+\|/ && $4 ~ /\// {print "      " $1 "：第 4 列 " $4}')
if [[ -n "$bad_rows" ]]; then
  echo "  ✗ 登记表第 4 列（留存产物）写成了带斜杠的路径，应当是 research/results/ 底下的纯文件名：" >&2
  printf "%s\n" "$bad_rows" >&2   # gate-lint:detail
  echo "     → 怎么办：把那几行第 4 列改回纯文件名（例 e100-system-configuration-slot-2026-09-03.out）；" >&2
  echo "               产物搬过家就同时改文件名本身，别把目录写进这一列。" >&2
  exit 2
fi

# 计时行的负载关：阈值要是数；滞后表要读得过（登记的名字都是上面表里 timing 行的留存产物、理由写全、留存产物自己还没有字段）。读不过一行都不开跑。
if [[ ! "$TIMING_LOAD_RATIO" =~ ^[0-9]+(\.[0-9]+)?$ ]]; then
  echo "  ✗ REPLAY_TIMING_LOAD_RATIO 是「$TIMING_LOAD_RATIO」，要是非负的数（默认 0.5）" >&2
  echo "     → 怎么办：写成 0.5 这样的小数，或不设" >&2
  exit 2
fi
if ! lag_problems_text="$(load_timing_lag "$TIMING_LAG_FILE" "$TABLE" research/results)"; then
  echo "  ✗ 计时行产物的负载字段滞后表 $TIMING_LAG_FILE 有 $(printf '%s\n' "$lag_problems_text" | wc -l) 处不对：" >&2
  printf '      %s\n' "$lag_problems_text" >&2   # gate-lint:detail
  echo "     → 怎么办：滞后表只缩不涨，一行一个留存产物名加「  # 理由」；名字要是上面复跑表里 timing 行的第 4 列，留存产物补了字段就删那一行，不往里加新行" >&2
  exit 2
fi
load_timing_lag "$TIMING_LAG_FILE" "$TABLE" research/results >/dev/null   # 上面在命令替换里读的，TIMING_LAG 要在这一层再读一遍

# ── 准入（门禁与实验共用的 scripts/gate/admission.py，登记表 .claude/gate.d/stage-inputs.tsv）──────────
# 登记了准入的实验，每一行开跑之前先问一次（driver_e142 这类驱动在装置之前还要编、要跑 crates 的导出，先问省掉这一截）；
# 装置入口自己还会再问一次，并把输入指纹打在产物最前面（`E7INPUT` 开头，比对前删掉，`name=done` 的条数不数它们）。
# 退 77（输入自上次产物以来没变）记「输入没变」，不跑、不判红，沿用留存产物的结论；别的非 0 记「跑不了」。
# 每一行问哪个准入键：下面 ADMISSION_ROW_KEYS 指明了的用指明的，没指明的用实验号（写法见 scripts/gate/admission.py 文件头「复跑表按行的准入键」）。
# 同一个实验在这张表里不止一行、又登记了准入的，每一行都要在这里指明一个键、在 .claude/gate.d/stage-inputs.tsv 各登记一行它读的输入，
# 每一行按自己的键问准入、比自己的产物；不然准入分不清哪一行对哪一份，admission.py replay-row-keys 判红、整份不开跑。
# 一行一条，制表符分隔：<准入键> <复跑表那一行的前三列，原样抄> <产物头谁写>。产物头谁写：device 是装置自己问准入、自己打 E7INPUT 行；
# replay 是那一行跑的东西不接准入（crates 的导出 bin 这一类），由 replay_one 把准入放行时给的 E7INPUT 行接在新产物前面。
# 这一段 bash 自己不读，准入模块从本文件原文读（replay-row-keys、experiment、product-field= 都按它认行）。
# shellcheck disable=SC2034
ADMISSION_ROW_KEYS=$(cat <<'TSV'
E142	E142|@driver_e142|	device
E142/p8g-crates-dump	E142|@driver_e142_p8g_crates_dump|	replay
E16R5	E16R5|e16-fifth-run-peak-under-today-widths|grid	device
E16R5/pc3	E16R5|e16-fifth-run-peak-under-today-widths|pc3	device
E16R5/second-segment	E16R5|e16-fifth-run-peak-under-today-widths|second_segment	device
TSV
)
ADMISSION_MODULE="scripts/gate/admission.py"
if ! admission_rows_text="$(python3 "$ADMISSION_MODULE" replay-row-keys .)"; then
  printf '%s\n' "$admission_rows_text" >&2
  echo "  ✗ 定不下复跑表每一行的准入键（python3 $ADMISSION_MODULE replay-row-keys .），一行都没跑" >&2
  echo "     → 怎么办：照上面准入模块打的那几条改 ADMISSION_ROW_KEYS 或 .claude/gate.d/stage-inputs.tsv；不知道每一行按哪个键问准入、比哪一份产物，就不开跑" >&2
  exit 2
fi
declare -A ADMISSION_REGISTERED=() ADMISSION_ROW_KEY=() ADMISSION_ROW_HEADER_WRITER=()
while IFS=$'\t' read -r row_identity row_key row_header_writer row_registered; do
  [[ -n "$row_identity" ]] || continue
  ADMISSION_ROW_KEY["$row_identity"]="$row_key"
  ADMISSION_ROW_HEADER_WRITER["$row_identity"]="$row_header_writer"
  [[ "$row_registered" == yes ]] && ADMISSION_REGISTERED["$row_key"]=1
done <<<"$admission_rows_text"
admission_rows_unkeyed=()
while IFS='|' read -r exp bin args _stored _kind; do
  [[ -z "$exp" ]] && continue
  [[ -n "${ADMISSION_ROW_KEY["$exp|$bin|$args"]+set}" ]] || admission_rows_unkeyed+=("$exp|$bin|$args")
done <<<"$TABLE"
if (( ${#admission_rows_unkeyed[@]} )); then
  echo "  ✗ 复跑表有 ${#admission_rows_unkeyed[@]} 行准入模块没给出准入键（它只认「编号|二进制|参数|产物|exact 或 timing」形状的行）：" >&2
  printf '      %s\n' "${admission_rows_unkeyed[@]}" >&2   # gate-lint:detail
  echo "     → 怎么办：把那几行改成这个形状（第 5 列只写 exact 或 timing），python3 scripts/gate/admission.py replay-row-keys . 列得出每一行再跑" >&2
  exit 2
fi

if (( ADMISSION_ONLY )); then
  admission_judged=0; admission_unregistered=0; admission_errors=0
  while IFS='|' read -r exp bin row_arguments stored _kind; do
    [[ -z "$exp" ]] && continue
    want "$exp" || continue
    admission_judged=$((admission_judged+1))
    admission_key_of_row "$exp" "$bin" "$row_arguments"
    if [[ -z "${ADMISSION_REGISTERED[$ROW_ADMISSION_KEY]:-}" ]]; then
      admission_unregistered=$((admission_unregistered+1))
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 没接准入 "准入登记表里没有 $ROW_ADMISSION_KEY，照跑（留存产物 $stored）"
      continue
    fi
    if admission_messages="$(python3 "$ADMISSION_MODULE" experiment . "$ROW_ADMISSION_KEY" 2>&1 >/dev/null)"; then admission_exit_code=0; else admission_exit_code=$?; fi
    case "$admission_exit_code" in
      0) admission_verdict=放行 ;; 77) admission_verdict=输入没变 ;; 3) admission_verdict=前提没齐 ;;
      *) admission_verdict=登记有错; admission_errors=$((admission_errors+1)) ;;
    esac
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" "$admission_verdict" "准入键 $ROW_ADMISSION_KEY，准入退 $admission_exit_code，留存产物 $stored"
    printf '%s\n' "$admission_messages" | sed 's/^/      /'   # gate-lint:detail
  done <<<"$TABLE"
  echo "只判准入、没编没跑：判了 $admission_judged 行（其中 $admission_unregistered 行没接准入，照跑），准入模块报登记有错 $admission_errors 行"
  (( admission_errors == 0 )) || exit 1
  exit 0
fi

# ── 按行复用（文件头「按行复用」）：先算全部行的行键、定哪几行要现跑，再决定编不编 ──────────────────────────────────
declare -A ROW_REUSE_KEY=() ROW_CHANGE_NOTE=() ROW_REFUSAL=() ROW_USES_GPU=()
REPLAY_ROW_LIBRARY=""
REPLAY_ROW_RECORDS=""
# 判定库服务不可用（文件头「按行复用」）：点名退了 75 的那一条，整趟退 75，不退回本地、不当库里没有现跑
replay_prune_server_unavailable() { # <退 75 的那一条命令>
  echo "  ✗ 按行复用：prune-server 不可用（$1 退 75）" >&2
  echo "     → 怎么办：python3 prune-server/control check 看服务，修好、重起服务之后重交这一趟（报给主 agent）；这一趟不退回本地文件、不当库里没有现跑" >&2
  exit 75
}
# 按行复用的库没法用（文件头「按行复用」）：开跑之前整趟退出（不在 git 工作树里退 2，别的退 1），不照「不复用、不记」全跑。
# 弄坏开关 row-library-failure-tolerated 下只打一句、往下全跑
replay_row_library_failure() { # <退出码> <一句为什么> <出路>
  if [[ "$REPLAY_BROKEN" == row-library-failure-tolerated ]]; then
    echo "  ! 按行复用：$2：这一趟一行都不复用、也不记" >&2
    REPLAY_ROW_LIBRARY=""; ROW_REUSE_KEY=()
    return 0
  fi
  echo "  ✗ 按行复用：$2，一行都没跑（退 $1）" >&2
  echo "     → 怎么办：$3" >&2
  exit "$1"
}
# 库名：这棵树的 git common-dir 下那一份，只当名字（键里只用种类名，命名空间只在临时服务底下给；文件头「按行复用」），本地不建
if replay_common_directory="$(git rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" && [[ -n "$replay_common_directory" ]]; then
  REPLAY_ROW_LIBRARY="$replay_common_directory/singlefs-unit-verdicts/replay-row"
  REPLAY_ROW_RECORDS="$OUT_DIR/row-records-from-prune-server"                      # 这一趟的中转目录：replay_finish 收尾删
else
  replay_row_library_failure 2 "$PWD 不是 git 工作树，判定库没有名字" "在 git 工作树里跑；rsync 拷的仓副本先在副本根 git init（键里不带哪棵仓，副本与主仓读写同一份库）"
fi
read_gpu_switch "$OUT_DIR/gpu-switch.log" || exit 78
mkdir -p "$OUT_DIR/compositions"
rm -f -- "$OUT_DIR/prune-server-unavailable"   # remember_row_verdict 记的服务不可用标记：同一个 REPLAY_OUT 里上一趟留下的不算这一趟的
row_inputs_exit=0
if [[ -z "$REPLAY_ROW_LIBRARY" ]]; then
  :   # 只在弄坏开关 row-library-failure-tolerated 下走到（改错的样子：不复用、不记）
elif row_inputs_text="$(python3 "$ADMISSION_MODULE" replay-row-inputs . --gpu "$REPLAY_GPU_SWITCH" --library "$REPLAY_ROW_LIBRARY" --compositions "$OUT_DIR/compositions" ${ONLY[@]+"${ONLY[@]}"})" \
     || { row_inputs_exit=$?; false; }; then
  while IFS=$'\t' read -r row_identity row_reuse_key _row_units row_uses_gpu row_note; do
    [[ -n "$row_identity" ]] || continue
    ROW_USES_GPU["$row_identity"]="$row_uses_gpu"   # 这一行要不要卡（点到 CHECKER_TIER_FEATURES），拒算的行也记（文件头「GPU」）
    if [[ "$REPLAY_BROKEN" == reuse-key-by-row-only ]]; then   # 弄坏开关：键只按复跑表那一行算，不看装置源码与工具链
      row_reuse_key="$(printf '%s' "$row_identity" | sha256sum | cut -c1-64)"; row_note=""
    fi
    if [[ "$row_reuse_key" == refused ]]; then
      ROW_REFUSAL["$row_identity"]="$row_note"
      continue
    fi
    ROW_REUSE_KEY["$row_identity"]="$row_reuse_key"
    ROW_CHANGE_NOTE["$row_identity"]="$row_note"
  done <<<"$row_inputs_text"
elif (( row_inputs_exit == 75 )) && [[ "$REPLAY_BROKEN" != prune-server-failure-swallowed ]]; then
  replay_prune_server_unavailable "python3 $ADMISSION_MODULE replay-row-inputs . --library $REPLAY_ROW_LIBRARY"
else
  printf '%s\n' "$row_inputs_text" >&2
  replay_row_library_failure 1 "每一行的行键算不出（python3 $ADMISSION_MODULE replay-row-inputs . 退 $row_inputs_exit，原因在上面）" \
    "单跑 python3 $ADMISSION_MODULE replay-row-inputs . 照它报的修（登记表、工具链、复跑表那几行）"
fi
# 这一趟要的那几把键的记录一次拉进中转目录（文件头「按行复用」）：开跑前那一判与复用哨兵读它
if (( ${#ROW_REUSE_KEY[@]} )); then
  rm -rf -- "${REPLAY_ROW_RECORDS:?}"
  if ! mkdir -p -- "$REPLAY_ROW_RECORDS/keys" "$REPLAY_ROW_RECORDS/items"; then
    replay_row_library_failure 1 "建不了中转目录 $REPLAY_ROW_RECORDS" "看 REPLAY_OUT 那个目录能不能写、盘满没满"
  else
    mapfile -t replay_fetch_prefixes < <(printf 'keys/%s/\n' "${ROW_REUSE_KEY[@]}" | sort -u)
    if replay_fetch_output="$(python3 "$ADMISSION_MODULE" library-files fetch "$REPLAY_ROW_LIBRARY" "$REPLAY_ROW_RECORDS" "${replay_fetch_prefixes[@]}" 2>&1)"; then
      replay_fetch_exit=0
    else
      replay_fetch_exit=$?
    fi
    if (( replay_fetch_exit == 75 )) && [[ "$REPLAY_BROKEN" != prune-server-failure-swallowed ]]; then
      printf '%s\n' "$replay_fetch_output" >&2
      replay_prune_server_unavailable "python3 $ADMISSION_MODULE library-files fetch $REPLAY_ROW_LIBRARY"
    elif (( replay_fetch_exit != 0 )); then
      printf '%s\n' "$replay_fetch_output" >&2
      replay_row_library_failure 1 "判定库里的记录拉不下来（python3 $ADMISSION_MODULE library-files fetch 退 $replay_fetch_exit，原因在上面）" \
        "照上面的报错修 admission.py library-files（文件头「library-files」）"
    fi
  fi
fi

# E182 的片结果库（文件头「E182 的片结果库」）：这一趟点到 driver_e182 或 driver_e182_units_kept 的行，开跑前把两种片拉进各自的中转目录，
#   各行收齐之后 replay_store_e182_shards 并回
REPLAY_E182_SHARD_KINDS=(e182-single-device e182-single-device-units-kept)
REPLAY_E182_SHARDS=""   # 拉过才设：$OUT_DIR/e182-shards-from-prune-server（底下 <种类>/ 一种一份）
replay_e182_wanted=0
while IFS='|' read -r exp bin _args _stored _kind; do
  [[ -n "$exp" ]] && want "$exp" && [[ "$bin" == @driver_e182 || "$bin" == @driver_e182_units_kept ]] && replay_e182_wanted=1
done <<<"$TABLE"
if (( replay_e182_wanted )) && [[ -n "${replay_common_directory:-}" ]]; then
  REPLAY_E182_SHARDS="$OUT_DIR/e182-shards-from-prune-server"
  rm -rf -- "${REPLAY_E182_SHARDS:?}"
  for replay_e182_kind in "${REPLAY_E182_SHARD_KINDS[@]}"; do
    if ! mkdir -p -- "$REPLAY_E182_SHARDS/$replay_e182_kind"; then
      replay_row_library_failure 1 "建不了 E182 片结果库的中转目录 $REPLAY_E182_SHARDS/$replay_e182_kind" "看 REPLAY_OUT 那个目录能不能写、盘满没满"
      continue
    fi
    replay_e182_exit=0
    replay_e182_output="$(python3 "$ADMISSION_MODULE" library-files fetch "$replay_common_directory/singlefs-unit-verdicts/$replay_e182_kind" "$REPLAY_E182_SHARDS/$replay_e182_kind" 2>&1)" \
      || replay_e182_exit=$?
    if (( replay_e182_exit == 75 )) && [[ "$REPLAY_BROKEN" != prune-server-failure-swallowed ]]; then
      printf '%s\n' "$replay_e182_output" >&2
      replay_prune_server_unavailable "python3 $ADMISSION_MODULE library-files fetch（E182 片结果库 $replay_e182_kind）"
    elif (( replay_e182_exit != 0 )); then
      printf '%s\n' "$replay_e182_output" >&2
      replay_row_library_failure 1 "E182 片结果库 $replay_e182_kind 拉不下来（python3 $ADMISSION_MODULE library-files fetch 退 $replay_e182_exit，原因在上面）" \
        "照上面的报错修 admission.py library-files（文件头「library-files」）"
    fi
  done
fi
# replay_store_e182_shards：各行收齐之后把两种片并回服务（逐字节相同的跳过）；退 75 整趟退 75，别的非 0 整趟退 1（文件头「E182 的片结果库」）
replay_store_e182_shards() {
  [[ -n "${REPLAY_E182_SHARDS:-}" ]] || return 0
  local kind store_exit store_output
  for kind in "${REPLAY_E182_SHARD_KINDS[@]}"; do
    store_exit=0
    store_output="$(python3 "$ADMISSION_MODULE" library-files store "$replay_common_directory/singlefs-unit-verdicts/$kind" "$REPLAY_E182_SHARDS/$kind" 2>&1)" || store_exit=$?
    if (( store_exit == 75 )) && [[ "$REPLAY_BROKEN" != prune-server-failure-swallowed ]]; then
      printf '%s\n' "$store_output" >&2
      replay_prune_server_unavailable "python3 $ADMISSION_MODULE library-files store（E182 片结果库 $kind）"
    elif (( store_exit != 0 )); then
      printf '%s\n' "$store_output" >&2
      echo "  ✗ E182 片结果库 $kind：判完的片并不回服务（library-files store 退 $store_exit，原因在上面），这一趟判的片没记下" >&2
      echo "     → 怎么办：照上面的报错修 admission.py library-files（文件头「library-files」），再重交这一趟（没记下的片下一趟照判）" >&2
      exit 1
    fi
  done
}

# 复用哨兵（scripts/gate/lib_reuse_checks.py 文件头「二」）：行键都算好之后、开跑前那一判之前抽；候选是这把行键在判定库里有判过的记录
# （keys/<行键>/verdict 是字节一致、仅计时不同之一），直接读中转目录里拉下来的记录、不经 decide_row。抽中强制现判的那一行 decide_row 不读库、照现跑
REPLAY_REUSE_CHECKS="${ADMISSION_MODULE%/admission.py}/lib_reuse_checks.py"
REPLAY_SENTINEL="$OUT_DIR/reuse-sentinel.json"
REPLAY_FORCED_TAG=""
: >"$REPLAY_SENTINEL.candidates"
if [[ ! -f "$REPLAY_REUSE_CHECKS" ]]; then
  # 拷进不带共用库的小树里跑的这一份（scripts/gate/admission.py 的自证那一类）：哨兵与上次红的清单都不做，说一句；真仓里这份库一直在
  echo "  ! 复用哨兵：找不到 $REPLAY_REUSE_CHECKS（这一份 replay.sh 跑在不带它的树里），这一趟不抽哨兵、不写上次红的清单" >&2
else
  candidate_sequence=0
  while IFS='|' read -r exp bin args stored kind; do
    [[ -z "$exp" ]] && continue
    want "$exp" || continue
    candidate_sequence=$((candidate_sequence + 1))
    candidate_key="${ROW_REUSE_KEY["$exp|$bin|$args"]-}"
    [[ -n "$candidate_key" && -f "$REPLAY_ROW_RECORDS/keys/$candidate_key/verdict" && -f "$REPLAY_ROW_RECORDS/keys/$candidate_key/product" ]] || continue
    candidate_verdict="$(cat -- "$REPLAY_ROW_RECORDS/keys/$candidate_key/verdict")"
    case "$candidate_verdict" in pass|timing_only) printf '%s\t%s\n' "$exp.$candidate_sequence" "$candidate_verdict" >>"$REPLAY_SENTINEL.candidates" ;; esac
  done <<<"$TABLE"
  python3 "$REPLAY_REUSE_CHECKS" sentinel-plan "$REPLAY_SENTINEL" --script scripts/experiment-apparatus/replay.sh --label 复跑表 --candidates "$REPLAY_SENTINEL.candidates" >&2 \
    || { echo "  ✗ 复用哨兵没抽成（$REPLAY_REUSE_CHECKS sentinel-plan）" >&2; echo "     → 怎么办：照它的报错修 scripts/gate/lib_reuse_checks.py；抽不成这一趟判不了复用" >&2; exit 1; }
fi
[[ -f "$REPLAY_SENTINEL" ]] && REPLAY_FORCED_TAG="$(python3 -c 'import json, sys; print(json.load(open(sys.argv[1], encoding="utf-8")).get("forced") or "")' "$REPLAY_SENTINEL")"

# 一行判完之后记进判定库（文件头「按行复用」）：先在中转目录里同目录临时目录写完再改名换上（连同这把键的组成清单与跑法），再经 library-files store
# 写进 prune-server：verdict 排在最后（它落库时同一条记录的别的文件都已落库）；items/<前三列的 sha256> 记这一行最近一次判过的键
# （下一趟键对不上时 admission.py 拿它比出变了的单元）。服务连不上（store 退 75）记一个标记，收齐各行之后整趟退 75；别的记不下只报一行，不判红
remember_row_verdict() { # <记录目录，空串不记> <判定> <结果行> <新产物> <前三列>
  local record="$1" temporary key identity_digest store_output store_exit=0 name
  local -a store_paths=()
  [[ -n "$record" ]] || return 0
  temporary="$record.tmp.$BASHPID"
  key="${record##*/}"
  rm -rf -- "${temporary:?}"
  if mkdir -p -- "$temporary" && cp -- "$4" "$temporary/product" && printf '%s\n' "$2" >"$temporary/verdict" \
     && printf '%s\n' "$3" >"$temporary/line" && TZ=Asia/Tokyo date +%F >"$temporary/finished" \
     && printf '%s\n' "bash scripts/experiment-apparatus/replay.sh（这一行的二进制与参数照复跑表；GPU 开关 ${REPLAY_GPU_SWITCH:-0}）" >"$temporary/how" \
     && { [[ ! -f "$OUT_DIR/compositions/$key.tsv" ]] || cp -- "$OUT_DIR/compositions/$key.tsv" "$temporary/composition"; }; then
    rm -rf -- "${record:?}"
    identity_digest="$(printf '%s' "$5" | sha256sum | cut -c1-64)"
    if mv -T -- "$temporary" "$record" 2>/dev/null && printf '%s\n' "$key" >"$REPLAY_ROW_RECORDS/items/$identity_digest"; then
      for name in product line finished how composition; do [[ -f "$record/$name" ]] && store_paths+=("keys/$key/$name"); done
      store_paths+=("items/$identity_digest" "keys/$key/verdict")
      store_output="$(python3 "$ADMISSION_MODULE" library-files store "$REPLAY_ROW_LIBRARY" "$REPLAY_ROW_RECORDS" "${store_paths[@]}" 2>&1)" || store_exit=$?
      (( store_exit == 0 )) && return 0
      printf '%s\n' "$store_output" >&2
      if (( store_exit == 75 )) && [[ "$REPLAY_BROKEN" != prune-server-failure-swallowed ]]; then
        : >"$OUT_DIR/prune-server-unavailable"
        echo "  ✗ 按行复用：prune-server 不可用（library-files store 退 75），$record 没记进判定库；收齐各行之后整趟退 75" >&2
        return 0
      fi
    fi
  fi
  rm -rf -- "${temporary:?}"
  echo "  ! 按行复用：记不下 $record（下一趟这一行照跑）" >&2
}

pass=0; drift=0; timing_only=0; broken=0; claim_bad=0; unchanged=0; UNCHANGED_ROWS=()
reused=0; REUSED_ROWS=(); refused_rows=0   # 按行复用的行（文件头「按行复用」）；行键拒算的行
load_skipped=0; load_passed=0; load_lagged=0; LOAD_SKIPPED_ROWS=()   # 计时行的负载关：本次不判、放行（含滞后）、其中滞后；本次不判的逐个记名字与比值
CLAIM_QUEUE=()

printf '%-5s %-24s %-10s %s\n' 实验 二进制 判定 说明
printf '%s\n' "-------------------------------------------------------------------------"
# 一行判跑出来的新产物（闸 2 逐段核、产物头、负载关、取回归档产物、逐字节比）：只写 $OUT_DIR/<tag>.line / .verdict / .claim 这几份；记进判定库由 replay_one 判完之后做
# （remember_row_outcome）：存库起 admission.py library-files store，不进判法的闭包（闭包里连上 admission.py 的程序入口，实验键与行键都拒算）。
# 这一段是复跑这一行的判法：行键里的 judge:scripts/experiment-apparatus/replay.sh::replay_judge_fresh_product（admission.py REPLAY_JUDGING_FUNCTIONS）。
replay_judge_fresh_product() { # <实验号> <二进制> <留存产物> <exact|timing> <准入键> <tag> <新产物>
  local exp="$1" bin="$2" stored="$3" kind="$4" row_key="$5" tag="$6" fresh="$7" gate2 n fresh_head result_line
  # 闸 2：**逐段**核。产物可能是多次运行拼起来的（E9 就是 25 段），
  # 只看最后一个 name=done 会让前 24 段的缺行全部漏过去。
  gate2=$(awk '/^E7RESULT/{n++}
               /^E7RESULT name=done emitted=/{
                 split($0,a,"emitted="); e=a[2]+0
                 segs++
                 if (e != n) { print "第 " segs " 段收尾行说 " e " 条，实收 " n " 条"; bad=1; exit }
                 n=0
               }
               END{ if (!bad) { if (segs==0) print "没有收尾行 name=done"; else if (n>0) print "最后一段没有收尾行，尾巴 " n " 条" } }' "$fresh")
  if [[ -n "$gate2" ]]; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "$gate2" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"; return
  fi
  # 登记了准入的实验，产物最前面要有实验键那一行；没有这一行，下一次准入永远判不了「输入没变」
  fresh_head="$(head -n 20 "$fresh")"
  if [[ -n "${ADMISSION_REGISTERED[$row_key]:-}" ]] && ! grep -q "^E7INPUT name=input_fingerprint key=$row_key " <<<"$fresh_head"; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "登记了准入，产物头却没有 key=$row_key 的实验键行：装置 main 没调 e7_index_bench::admit_experiment_run_or_exit（或传的键与 ADMISSION_ROW_KEYS 指明的不同）" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"; return
  fi
  # 计时行的负载关（文件头「计时行的负载关」）：负载越过阈值的记本次不判，不比结构、不判区间断言；没字段又不在滞后表的判红
  timing_load_decide "$kind" "$stored" "$fresh"
  case "$TIMING_LOAD_DECISION" in
    red)
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 缺负载字段 "$TIMING_LOAD_NOTE。→ 怎么办：装置的收尾行补 loadavg1= cpus=（约定见文件头）；留存产物还没字段的存量才登记进滞后表，只缩不涨" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"; return ;;
    skip)
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 本次不判 "$stored：$TIMING_LOAD_NOTE，机器正忙，不比结构、不判区间" >"$OUT_DIR/$tag.line"
      echo load_skipped >"$OUT_DIR/$tag.verdict"; printf '%s %s\n' "$exp" "$TIMING_LOAD_NOTE" >"$OUT_DIR/$tag.skip"; return ;;
  esac
  if [[ "$kind" == timing ]]; then
    if (( TIMING_LOAD_LAGGED )); then echo lagged >"$OUT_DIR/$tag.load"; else echo judged >"$OUT_DIR/$tag.load"; fi
  fi
  # 留存产物超过单文件推送上限的压成 .gz 入库（例 E186 两份），比对时先解压；别的照原样读。
  # 弄坏开关 REPLAY_BREAK=gz-read-raw：.gz 不解压直接读（replay_gzip_selftest 的 gz-stored-green 转红）
  # 留存产物按「每次提交删上一次的实验记录」归档进了版本库、不在树里的，从 git 历史取回到本轮输出目录再比（replay_archived_product_copy），结果行写出处；
  # 取不回判红（verdict broken，结果行判定列「取不回」）。弄坏开关 REPLAY_BREAK=unrecoverable-as-green：取不回当字节一致（改错的样子）
  local stored_file="research/results/$stored" stored_origin="" stored_commit
  if [[ ! -f "$stored_file" ]]; then
    stored_file="$OUT_DIR/$tag.stored"
    if stored_commit="$(replay_archived_product_copy "$stored" "$stored_file")"; then
      stored_origin="（取自 git show ${stored_commit}^:research/results/$stored）"
    elif [[ "$REPLAY_BROKEN" == unrecoverable-as-green ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 字节一致 "$stored（取不回，当字节一致）" >"$OUT_DIR/$tag.line"; echo pass >"$OUT_DIR/$tag.verdict"
      echo "$exp|$fresh" >"$OUT_DIR/$tag.claim"; return
    else
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 取不回 "留存产物 $stored 不在树里，git 历史里也取不回（git log --all --diff-filter=D -- research/results/$stored 找不到删它的提交，或那一版读不出），没有对照物。→ 怎么办：本趟新产物在 $fresh，照实验页的做法录成 research/results/$stored，或查这个产物名是不是改过" >"$OUT_DIR/$tag.line"
      echo broken >"$OUT_DIR/$tag.verdict"; return
    fi
  fi
  read_stored() { case "$stored" in *.gz) if [[ "$REPLAY_BROKEN" == gz-read-raw ]]; then cat -- "$stored_file"; else gzip -dc -- "$stored_file"; fi ;; *) cat -- "$stored_file" ;; esac; }
  if diff -q <(strip_admission_header <"$fresh") <(read_stored | strip_admission_header) >/dev/null 2>&1; then
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 字节一致 "$stored$stored_origin" >"$OUT_DIR/$tag.line"; echo pass >"$OUT_DIR/$tag.verdict"
    echo "$exp|$fresh" >"$OUT_DIR/$tag.claim"; return
  fi
  if diff -q <(strip_admission_header <"$fresh" | strip_load_fields | strip_timing) <(read_stored | strip_admission_header | strip_load_fields | strip_timing) >/dev/null 2>&1; then
    if [[ "$kind" == timing ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 仅计时不同 "$stored${stored_origin}（结构一致，符合声明）" >"$OUT_DIR/$tag.line"; echo timing_only >"$OUT_DIR/$tag.verdict"
      echo "$exp|$fresh" >"$OUT_DIR/$tag.claim"
    else
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 判据写错 "$stored$stored_origin 声明 exact 却只在抹掉计时后才一致" >"$OUT_DIR/$tag.line"; echo drift >"$OUT_DIR/$tag.verdict"
    fi
    return
  fi
  n=$(diff <(strip_admission_header <"$fresh" | strip_load_fields | strip_timing) <(read_stored | strip_admission_header | strip_load_fields | strip_timing) | grep -c '^[<>]')
  printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 对不上 "$stored$stored_origin，$n 行不同 → diff $fresh $stored_file" >"$OUT_DIR/$tag.line"
  echo drift >"$OUT_DIR/$tag.verdict"
}

# 闸 2 的弄坏开关 REPLAY_BREAK=emitted-unchecked：把上面那一判换成恒假（收尾行的 emitted= 与实收条数对不上也照往下判）。
# 换法写在函数外面、载入时办：replay_judge_fresh_product 的正文进复跑表每一行的行键（admission.py 的 REPLAY_JUDGING_FUNCTIONS），开关写进正文会让每一行的键都变。
# 换不上（那一句改了样子）就退 2，不悄悄当开关打开了。
if [[ "$REPLAY_BROKEN" == emitted-unchecked ]]; then
  replay_judge_definition="$(declare -f replay_judge_fresh_product)"
  replay_judge_broken="${replay_judge_definition/'if [[ -n "$gate2" ]]; then'/'if false; then'}"
  if [[ "$replay_judge_broken" == "$replay_judge_definition" ]]; then
    echo "  ✗ 弄坏开关 emitted-unchecked 换不上：replay_judge_fresh_product 里找不到闸 2 那一判（if [[ -n \"\$gate2\" ]]; then）" >&2
    echo "     → 怎么办：闸 2 那一句改了样子就同步改这里的换法，再跑 bash scripts/experiment-apparatus/replay.sh --selftest" >&2
    exit 2
  fi
  eval "$replay_judge_broken"
fi

# 开跑前的那一判（准入与按行复用）：照判定写 $OUT_DIR/<tag>.decision（reuse、unchanged、broken、run 之一），准入放行的产物头写 <tag>.header；
# 复用、输入没变、跑不了的那几行同时把结果行与结论写好，replay_one 读到不是 run 的就直接收尾。主循环先把每一行都判完，再定编不编（文件头「按行复用」）。
decide_row() { # <实验号> <tag> <二进制> <留存产物> <准入键> <行键，空串不复用>
  local exp="$1" tag="$2" bin="$3" stored="$4" row_key="$5" reuse_key="$6" reuse_record="" reused_verdict reused_title admission_exit_code=0
  rm -f "$OUT_DIR/$tag.decision" "$OUT_DIR/$tag.header" "$OUT_DIR/$tag.admission"
  if [[ -n "${ADMISSION_REGISTERED[$row_key]:-}" ]]; then
    if python3 "$ADMISSION_MODULE" experiment . "$row_key" >"$OUT_DIR/$tag.header" 2>"$OUT_DIR/$tag.admission"; then admission_exit_code=0; else admission_exit_code=$?; fi
    if [[ $admission_exit_code -eq 77 ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 输入没变 "实验键与留存产物头上记的相同，本次没跑，沿用 $stored 的结论（见 $OUT_DIR/$tag.admission）" >"$OUT_DIR/$tag.line"
      echo unchanged >"$OUT_DIR/$tag.verdict"; echo "$exp|results/$stored" >"$OUT_DIR/$tag.claim"; echo unchanged >"$OUT_DIR/$tag.decision"; return
    fi
    if [[ $admission_exit_code -ne 0 ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "准入没放行（退 $admission_exit_code），见 $OUT_DIR/$tag.admission" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"
      echo broken >"$OUT_DIR/$tag.decision"; return
    fi
  fi
  [[ -n "$reuse_key" && -n "$REPLAY_ROW_LIBRARY" ]] && reuse_record="$REPLAY_ROW_RECORDS/keys/$reuse_key"
  if [[ -n "$REPLAY_FORCED_TAG" && "$tag" == "$REPLAY_FORCED_TAG" ]]; then   # 复用哨兵抽中强制现判：不读库、照现跑
    printf '  · %s 要现跑：复用哨兵抽中强制现判（行键相同，这一趟不读库）\n' "$exp" >&2
    echo run >"$OUT_DIR/$tag.decision"; return
  fi
  if [[ -n "$reuse_record" && -f "$reuse_record/verdict" && -f "$reuse_record/product" ]] && [[ "$REPLAY_BROKEN" != reuse-ignored ]]; then
    reused_verdict="$(cat -- "$reuse_record/verdict")"
    case "$reused_verdict" in pass) reused_title=字节一致 ;; timing_only) reused_title=仅计时不同 ;; *) reused_title="" ;; esac
    if [[ -n "$reused_title" ]]; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 复用 "${reused_title}：行键相同（${reuse_key:0:16}…，$(cat -- "$reuse_record/finished" 2>/dev/null || echo 那一趟)判的；留存产物 $stored），这一趟没跑" >"$OUT_DIR/$tag.line"
      echo "reused:$reused_verdict" >"$OUT_DIR/$tag.verdict"; echo "$exp|$reuse_record/product" >"$OUT_DIR/$tag.claim"
      echo reuse >"$OUT_DIR/$tag.decision"; return
    fi
  fi
  # 库里这把行键最近一次判的是红（对不上，或现跑了而跑不了）：不当绿复用，现跑；写一行原因（文件头「按行复用」记红、撤绿、下趟现判）
  if [[ -n "$reuse_record" && -f "$reuse_record/verdict" ]]; then
    case "$(cat -- "$reuse_record/verdict")" in
      drift) printf '  · %s 要现跑：上次判红（%s 判的对不上，行键相同），不当绿复用\n' "$exp" "$(cat -- "$reuse_record/finished" 2>/dev/null || echo 那一趟)" >&2 ;;
      broken) printf '  · %s 要现跑：上次判红（%s 那一趟跑不了，行键相同），不当绿复用\n' "$exp" "$(cat -- "$reuse_record/finished" 2>/dev/null || echo 那一趟)" >&2 ;;
    esac
  fi
  echo run >"$OUT_DIR/$tag.decision"
}

# 一条实验的复跑：并发起的，所以计数、结果行与 claim 队列都落到自己的文件，
# 父进程按登记表的顺序回读（command-safety.md「并行不许把失败吃掉」「输出不许直接往 stdout 写」）。
replay_one() {
  # 第 2 个参数是这一条在登记表里的序号：同一个实验号可以有两行（E16 就是，一行比 e16-journal、
  # 一行带 args=bytes 比 e16-bytes），按实验号命名输出文件时两条会并发写同一个文件，
  # 后写完的那份整份留下、另一条的结果彻底消失，而「派多少收多少」那道闸只判文件存不存在、看不出来。
  # 第 7、8 个参数是这一行的准入键与产物头谁写（ADMISSION_ROW_KEYS，没指明的是实验号与 device）；
  # 第 9 个参数是这一行的行键（文件头「按行复用」，空串就是不复用也不记）；第 10 个是复跑表的前三列原样（判定库按它记这一行最近一次的键）
  local exp="$1" seq="$2" bin="$3" args="$4" stored="$5" kind="$6" row_key="$7" header_writer="$8" reuse_key="${9:-}" identity="${10:-}" tag fresh rc
  local reuse_record=""
  tag="$exp.$seq"
  fresh="$OUT_DIR/$tag.out"
  [[ "$(cat -- "$OUT_DIR/$tag.decision" 2>/dev/null)" == run ]] || return   # 复用、输入没变、跑不了：开跑前那一判已经写好了这一行
  # 闸 1：不许跨轮复用（负载关的记录、取回的留存产物同样）：跑前删上一趟留在同一个 REPLAY_OUT 里的这一行的记录
  rm -f "$fresh" "$OUT_DIR/$tag.load" "$OUT_DIR/$tag.skip" "${OUT_DIR:?}/${tag:?}.stored"
  [[ -n "$reuse_key" && -n "$REPLAY_ROW_LIBRARY" ]] && reuse_record="$REPLAY_ROW_RECORDS/keys/$reuse_key"
  # 手里那一步放后台、wait 等它（文件头「编译目录」）：父进程 replay_stop 给这一行转 TERM 时 trap 当场跑，再转给它（`&` 起、`$!` 记下的那一个进程）、等它退，退 143。
  # 弄坏开关 REPLAY_BREAK=term-not-forwarded：不转（这一行自己退了，手里那一步照跑）
  local payload=""
  [[ "$REPLAY_BROKEN" == term-not-forwarded ]] || trap '[[ -z "$payload" ]] || { kill -TERM "$payload" 2>/dev/null; wait "$payload" 2>/dev/null; }; exit 143' TERM
  if [[ "$bin" == @* ]]; then
    "${bin#@}" >"$fresh" 2>"$OUT_DIR/$tag.err" &
  else
    # shellcheck disable=SC2086
    "$REPLAY_RELEASE_DIRECTORY/$bin" $args >"$fresh" 2>"$OUT_DIR/$tag.err" &
  fi
  payload=$!
  wait "$payload"
  rc=$?
  payload=""
  if [[ $rc -ne 0 && "$REPLAY_BROKEN" != exit-code-ignored ]]; then   # 闸 3（弄坏开关 REPLAY_BREAK=exit-code-ignored：退出码非 0 照往下判）
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "退出码 $rc，见 $OUT_DIR/$tag.err" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"
    remember_row_outcome "$tag" "$reuse_record" "$fresh" "$identity"; return
  fi
  # 产物头由这里写的行（ADMISSION_ROW_KEYS 第三列 replay：跑的东西不接准入）：准入放行时给的 E7INPUT 行接在新产物最前面，与装置自己打的同一个样子
  if [[ -n "${ADMISSION_REGISTERED[$row_key]:-}" && "$header_writer" == replay ]]; then
    if ! { cat -- "$OUT_DIR/$tag.header" "$fresh" >"$fresh.stamped" && mv -- "$fresh.stamped" "$fresh"; }; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "准入键 $row_key 的产物头接不到新产物前面（$OUT_DIR/$tag.header）" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"
      remember_row_outcome "$tag" "$reuse_record" "$fresh" "$identity"; return
    fi
  fi
  replay_judge_fresh_product "$exp" "$bin" "$stored" "$kind" "$row_key" "$tag" "$fresh"
  remember_row_outcome "$tag" "$reuse_record" "$fresh" "$identity"
}

# 一行判完记进判定库（文件头「按行复用」）：照这一行的行键记下判法写的那个判定（$OUT_DIR/<tag>.verdict）。判绿的（字节一致、仅计时不同）记绿；
#   记红、撤绿：判成对不上（drift）或跑不了（broken）的照同一把键记下、盖掉这把键原有的绿，下一趟这一行现判。本次不判不碰库。
#   记在 replay_judge_fresh_product 外面：它的正文进每一行的行键，存库那一步（起 admission.py library-files store）进了它的闭包，每一行的键都拒算。
#   新产物不在（跑不了的那几种可能一个字都没写）记一份空的，不因为拷不了产物就留着原有的绿。
#   弄坏开关 red-not-recorded 下对不上的不记、broken-not-recorded 下跑不了的不记
remember_row_outcome() { # <tag> <记录目录，空串不记> <新产物> <前三列>
  local verdict product="$3"
  verdict="$(cat -- "$OUT_DIR/$1.verdict" 2>/dev/null)"
  case "$verdict" in
    pass|timing_only) ;;
    drift) [[ "$REPLAY_BROKEN" == red-not-recorded ]] && return 0 ;;
    broken) [[ "$REPLAY_BROKEN" == broken-not-recorded ]] && return 0 ;;
    *) return 0 ;;
  esac
  [[ -f "$product" ]] || product=/dev/null
  remember_row_verdict "$2" "$verdict" "$(cat -- "$OUT_DIR/$1.line")" "$product" "$4"
}


# 开跑前那一判：按表里的次序给每一行定 tag 与判定（decide_row），数出要现跑的几行；一行都不用现跑就不试编 gpu 特性、不编（文件头「按行复用」）；
# 要现跑的行里一行要卡的都没有（admission.py replay-row-inputs 第 4 列全是 0）就不试编 gpu 特性、不要服务交的卡（文件头「GPU」）。
# 弄坏开关 REPLAY_BREAK=gpu-for-every-run：要现跑的行一律当要卡
declare -a ROW_TEXTS=() REPLAY_ORDER=() TO_RUN_BINARIES=()
row_sequence=0; to_run=0; to_run_drivers=0; to_run_gpu=0
while IFS='|' read -r exp bin args stored kind; do
  [[ -z "$exp" ]] && continue
  want "$exp" || continue
  row_sequence=$((row_sequence+1))
  admission_key_of_row "$exp" "$bin" "$args"   # 按表里的原样查（$REPLAY_DEV 这类还没展开）
  row_identity="$exp|$bin|$args"
  ROW_TEXTS+=("$exp|$bin|$args|$stored|$kind")
  REPLAY_ORDER+=("$exp.$row_sequence")
  [[ -n "${ROW_REFUSAL["$row_identity"]+set}" ]] && refused_rows=$((refused_rows + 1))
  decide_row "$exp" "$exp.$row_sequence" "$bin" "$stored" "$ROW_ADMISSION_KEY" "${ROW_REUSE_KEY["$row_identity"]-}"
  if [[ "$(cat -- "$OUT_DIR/$exp.$row_sequence.decision" 2>/dev/null)" == run ]]; then
    to_run=$((to_run + 1))
    if [[ "$bin" == @* ]]; then to_run_drivers=$((to_run_drivers + 1)); else TO_RUN_BINARIES+=("$bin"); fi
    if [[ "${ROW_USES_GPU["$row_identity"]:-0}" == 1 || "$REPLAY_BROKEN" == gpu-for-every-run ]]; then to_run_gpu=$((to_run_gpu + 1)); fi
    if [[ -n "${ROW_CHANGE_NOTE["$row_identity"]:-}" ]]; then printf '  · %s 要现跑：%s\n' "$exp" "${ROW_CHANGE_NOTE["$row_identity"]}" >&2; fi
    if [[ -n "${ROW_REFUSAL["$row_identity"]+set}" ]]; then printf '  · %s 要现跑：行键%s\n' "$exp" "${ROW_REFUSAL["$row_identity"]}" >&2; fi
  fi
done <<<"$TABLE"
if (( to_run == 0 )); then
  echo "  · 开跑前判完：${#REPLAY_ORDER[@]} 行里要现跑的 0 行，不试编 gpu 特性、不编" >&2
else
  if (( to_run_gpu )); then
    prepare_checker_tier_features "$OUT_DIR/gpu-build.log" || exit 78
  elif [[ "$REPLAY_GPU_SWITCH" == 1 ]]; then
    echo "  · 开跑前判完：要现跑的 $to_run 行都不点 CHECKER_TIER_FEATURES，不试编 gpu 特性、不要服务交的卡" >&2
  fi
  if (( to_run_drivers )) || [[ "$REPLAY_BROKEN" == build-everything ]]; then
    build_research_binaries "$OUT_DIR/build.log" || exit 2
  else
    build_research_binaries "$OUT_DIR/build.log" $(printf '%s\n' "${TO_RUN_BINARIES[@]}" | sort -u) || exit 2
  fi
fi

if (( to_run )); then replay_restore_referenced_products; fi

# 并发度：这些实验多是跑 release 二进制的 CPU 活，缺省取登记的重活默认并行度（default_replay_jobs，不看忙闲）。
# REPLAY_JOBS 压过它。各条实验的产物、镜像（e9/e45/e58/e140.img）与结果文件各用各的，没有共用的可写状态。
REPLAY_JOBS="${REPLAY_JOBS:-$(default_replay_jobs)}"
[[ "$REPLAY_JOBS" -lt 1 ]] && REPLAY_JOBS=1
declare -a REPLAY_PIDS=()
running=0
row_sequence=0
for row_text in ${ROW_TEXTS[@]+"${ROW_TEXTS[@]}"}; do
  IFS='|' read -r exp bin args stored kind <<<"$row_text"
  admission_key_of_row "$exp" "$bin" "$args"
  row_identity="$exp|$bin|$args"
  row_header_writer="${ADMISSION_ROW_HEADER_WRITER["$row_identity"]-device}"
  row_reuse_key_of_this_row="${ROW_REUSE_KEY["$row_identity"]-}"   # 按行复用的键（文件头「按行复用」），按表里的原样查
  args="${args//\$REPLAY_DEV140/$REPLAY_DEV140}"  # 先换长的，否则前缀会被短的吃掉
  args="${args//\$REPLAY_DEV58/$REPLAY_DEV58}"
  args="${args//\$REPLAY_DEV45/$REPLAY_DEV45}"
  args="${args//\$REPLAY_DEV/$REPLAY_DEV}"   # 表里写字面量 $REPLAY_DEV，这里才展开
  row_sequence=$((row_sequence+1))
  replay_one "$exp" "$row_sequence" "$bin" "$args" "$stored" "$kind" "$ROW_ADMISSION_KEY" "$row_header_writer" "$row_reuse_key_of_this_row" "$row_identity" &
  REPLAY_PIDS+=($!)
  running=$((running+1))
  if (( running >= REPLAY_JOBS )); then wait -n 2>/dev/null || true; running=$((running-1)); fi
done
# 逐个 wait 写死的 pid，不写不带参数的 wait（它的退出码恒为 0，红了几个一个字都不说）
for pid in ${REPLAY_PIDS[@]+"${REPLAY_PIDS[@]}"}; do wait "$pid" 2>/dev/null || true; done

# 派出去多少条就要收回来多少条：对不上整道红，不许少跑一条还报绿
collected=0
for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
  [[ -f "$OUT_DIR/$tag.line" ]] && collected=$((collected+1))
done
if (( collected != ${#REPLAY_ORDER[@]} )); then
  echo "  ✗ 派出去 ${#REPLAY_ORDER[@]} 条复跑，只收回 $collected 条结果行"
  echo "     → 怎么办：这是并发收束自己的完整性闸红了，不是实验的问题；REPLAY_JOBS=1 再跑一遍看串行下全不全，"
  echo "       全的话去查 replay_one 与它的 pid 收束那一段。"
  exit 1
fi

# 按登记表的顺序回读：输出与并发度无关，REPLAY_JOBS=1 与 =16 逐字相同
for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
  cat "$OUT_DIR/$tag.line"
  if [[ -f "$OUT_DIR/$tag.verdict" ]]; then
    case "$(cat "$OUT_DIR/$tag.verdict")" in
      pass) pass=$((pass+1)) ;; timing_only) timing_only=$((timing_only+1)) ;;
      drift) drift=$((drift+1)) ;; broken) broken=$((broken+1)) ;;
      unchanged) unchanged=$((unchanged+1)); UNCHANGED_ROWS+=("${tag%.*}") ;;
      reused:*) reused=$((reused+1)); REUSED_ROWS+=("${tag%.*}") ;;
      load_skipped) load_skipped=$((load_skipped+1)); LOAD_SKIPPED_ROWS+=("$(cat "$OUT_DIR/$tag.skip")") ;;
    esac
  fi
  if [[ -f "$OUT_DIR/$tag.load" ]]; then   # 放行的计时行（judged 或 lagged）
    load_passed=$((load_passed+1)); if [[ "$(cat "$OUT_DIR/$tag.load")" == lagged ]]; then load_lagged=$((load_lagged+1)); fi
  fi
  [[ -f "$OUT_DIR/$tag.claim" ]] && CLAIM_QUEUE+=("$(cat "$OUT_DIR/$tag.claim")")
done
# E182 的片结果库并回服务（文件头「E182 的片结果库」）
replay_store_e182_shards
# 有一行判完记不进判定库是因为服务连不上（remember_row_verdict 记的标记，文件头「按行复用」）：整趟退 75，不判绿也不判红
if [[ -f "$OUT_DIR/prune-server-unavailable" ]]; then
  replay_prune_server_unavailable "python3 $ADMISSION_MODULE library-files store $REPLAY_ROW_LIBRARY（上面逐行点名了哪几条没记进去）"
fi

# 复用哨兵判：每一行的结局（复用来的、现判的、强制现判的）一次记好，哨兵被现判、强制现判那一行判定与库里不同，整道红
: >"$OUT_DIR/reuse-sentinel.outcomes"
for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
  row_verdict="$(cat -- "$OUT_DIR/$tag.verdict" 2>/dev/null)"
  case "$row_verdict" in
    reused:*) printf '%s\t复用\t%s\n' "$tag" "${row_verdict#reused:}" ;;
    "") ;;   # 没有结局的行不报结局：它没判，哨兵抽中它也照「没跑到」不判
    *) printf '%s\t%s\t%s\n' "$tag" "$([[ "$tag" == "$REPLAY_FORCED_TAG" ]] && echo 强制现判 || echo 现判)" "$row_verdict" ;;
  esac
done >>"$OUT_DIR/reuse-sentinel.outcomes"
sentinel_bad=0
if [[ -f "$REPLAY_SENTINEL" ]]; then
  python3 "$REPLAY_REUSE_CHECKS" sentinel-observe-many "$REPLAY_SENTINEL" "$OUT_DIR/reuse-sentinel.outcomes" || echo "  ! 复用哨兵没记下这一趟的结局（这一趟哨兵不判）" >&2
  python3 "$REPLAY_REUSE_CHECKS" sentinel-finish "$REPLAY_SENTINEL" || sentinel_bad=1
fi

printf '%s\n' "-------------------------------------------------------------------------"
echo "结论区间断言（计时实验复跑不出同样的字节，靠这些把 kb 里的数钉住）："
CLAIM_BAD_ROWS=()
for q in "${CLAIM_QUEUE[@]}"; do
  check_claims "${q%%|*}" "${q#*|}" || { claim_bad=$((claim_bad+1)); CLAIM_BAD_ROWS+=("${q%%|*}"); }
done
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」，.claude/hooks/run-scope-guard.sh 的拒绝说明读它）：一行一个红的实验号（对不上、跑不了、结论断言不中）；
# 点名了实验号的只换这几个，别的照留
{ for tag in ${REPLAY_ORDER[@]+"${REPLAY_ORDER[@]}"}; do
    case "$(cat -- "$OUT_DIR/$tag.verdict" 2>/dev/null)" in drift|broken) printf '%s\n' "${tag%.*}" ;; esac
  done
  printf '%s\n' ${CLAIM_BAD_ROWS[@]+"${CLAIM_BAD_ROWS[@]}"}; } | sort -u >"$OUT_DIR/last-run.red"
if [[ ! -f "$REPLAY_REUSE_CHECKS" ]]; then
  :   # 不带共用库的小树（上面复用哨兵那一段说过一句）
elif (( ${#ONLY[@]} )); then
  printf '%s\n' "${ONLY[@]}" >"$OUT_DIR/last-run.ran"
  python3 "$REPLAY_REUSE_CHECKS" last-run scripts/experiment-apparatus/replay.sh "$OUT_DIR/last-run.red" --ran "$OUT_DIR/last-run.ran" >&2 || echo "  ! 上次红的清单没写成" >&2
else
  python3 "$REPLAY_REUSE_CHECKS" last-run scripts/experiment-apparatus/replay.sh "$OUT_DIR/last-run.red" >&2 || echo "  ! 上次红的清单没写成" >&2
fi
printf '%s\n' "-------------------------------------------------------------------------"
echo "字节一致 $pass ／ 仅计时不同 $timing_only ／ 对不上 $drift ／ 跑不了 $broken ／ 结论断言不中 $claim_bad ／ 输入没变没跑 $unchanged ／ 负载高本次不判 $load_skipped ／ 按行复用 $reused"
echo "REPLAY_TOTAL dispatched=${#REPLAY_ORDER[@]} collected=$collected judged=$((pass + timing_only + drift)) reused=$reused refused=$refused_rows pass=$pass timing_only=$timing_only drift=$drift broken=$broken claims_bad=$claim_bad unchanged=$unchanged load_skipped=$load_skipped"
if (( reused )); then
  echo "  · 按行复用 $reused 行（行键相同，判定库在 prune-server 里、库名 ${REPLAY_ROW_LIBRARY:-没有}，结论区间断言照常对记录里留的那一份产物判）：${REUSED_ROWS[*]}"
fi
echo "本轮输出：$OUT_DIR"
for build_log_name in build gpu-build; do   # 全绿又用自动分配的临时目录时下面会整个删掉它，不列
  [[ -f "$OUT_DIR/$build_log_name.problems.log" ]] && { [[ -n "${REPLAY_OUT:-}" ]] || (( drift + broken + claim_bad + sentinel_bad )); } && echo "  · 编译日志两份（先看错误警告那份）：$OUT_DIR/$build_log_name.log 全量；$OUT_DIR/$build_log_name.problems.log 错误与警告"
done
if (( load_passed + load_skipped )); then
  echo "  计时行负载关：判了 $load_passed、本次不判 $load_skipped、滞后 $load_lagged 份（滞后表 $TIMING_LAG_FILE，登记 ${#TIMING_LAG[@]} 份）"
fi
if [[ $load_skipped -ne 0 ]]; then
  echo "  ! 「本次不判」$load_skipped 行：新产物收尾行的负载超过阈值 $TIMING_LOAD_RATIO（REPLAY_TIMING_LOAD_RATIO），机器正忙，计时数不可信；这些行既不判绿也不判红，结构与区间断言都没比："
  printf '      %s\n' "${LOAD_SKIPPED_ROWS[@]}"   # gate-lint:detail
  echo "     → 怎么办：等负载降下来，只重跑这几行：bash scripts/experiment-apparatus/replay.sh <实验号>"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    for skipped_row in "${LOAD_SKIPPED_ROWS[@]}"; do
      printf 'replay.sh %s，负载过高本次不判（阈值 %s）\n' "$skipped_row" "$TIMING_LOAD_RATIO" >>"$GATE_NOT_RUN_FILE"
    done
  fi
fi
if [[ $unchanged -ne 0 ]]; then
  echo "  ! 「输入没变」$unchanged 行没跑：${UNCHANGED_ROWS[*]}。实验键（这一行的行键去掉期望产物：装置入口单元闭包、驱动闭包、参数、GPU 开关、工具链）"
  echo "     与留存产物头上记的相同，准入（scripts/gate/admission.py）拒绝重跑，结论沿用留存产物；结论区间断言照常对留存产物判。"
  echo "     这不是判绿也不是判红。输入没变不重跑，结论沿用留存产物；要重跑就先改输入（上面实验键里的那几样），键变了准入才放行。"
fi
if (( sentinel_bad )); then
  echo "  ✗ 复用哨兵判红（上面 ✗ 那几行：键相同却现判了，或强制现判那一行判定与库里不同）"
  echo "     → 怎么办：照上面的出路查 decide_row 读库那一段与 remember_row_verdict；判法在 scripts/gate/lib_reuse_checks.py 文件头「二」"
fi
if [[ $drift -ne 0 || $broken -ne 0 || $claim_bad -ne 0 ]]; then
  echo "  → 怎么办：「跑不了」看上面那一行给的 $OUT_DIR/<实验号>.err；「对不上」按上面给的 diff 命令看差在哪，" \
       "结构性差异是代码改动带来的就更新入库产物，不是就说明代码退化了；" \
       "「结论断言不中」逐条去 check_claims() 里对应实验号那一段读注释——是该改 kb 里记的区间，还是真的退化了，" \
       "两种都要人判，不许为了让这里变绿就调宽容差（test-discipline.md）。"
fi
if [[ $drift -eq 0 && $broken -eq 0 && $claim_bad -eq 0 && $sentinel_bad -eq 0 ]]; then
  # 全绿且用的是自动分配的临时目录 ⇒ 收拾掉。有一条不绿就留着，上面的提示指着它。
  [[ -z "${REPLAY_OUT:-}" ]] && rm -rf "$OUT_DIR"
  if (( load_skipped && pass + timing_only + unchanged + reused == 0 )); then
    echo "  ! 判的全是本次不判（负载过高 $load_skipped 行），一行都没比：不算通过，退 77"
    exit 77
  fi
  echo "  ✓ 复跑现判 $((pass + timing_only)) 行都对得上（字节一致 $pass、仅计时不同 $timing_only）、复用 $reused 行、行键拒算照跑 $refused_rows 行、负载高本次不判 $load_skipped 行、输入没变没跑 $unchanged 行，结论断言全中；负载关：判了 $load_passed、滞后 $load_lagged 份"
  exit 0
fi
exit 1
