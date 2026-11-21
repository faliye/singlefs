#!/usr/bin/env bash
# admission: always 每一次调都判此刻的仓与这一次的参数，上一次的结论不替这一次作保
# run-condition: command cargo git
# task-server-compiles: yes
# 复跑已入库的实验，把今天的输出和 research/results/ 里那份逐字节比对。
#
#   bash scripts/experiment-apparatus/replay.sh [实验号...]      # 不给参数就全跑
#   bash scripts/experiment-apparatus/replay.sh --list-items     # 逐行打可点名的项：先是实验号（几行、二进制或驱动），再是每一行（<实验号>.<行序号>、1、二进制或驱动），不编、不跑
#   bash scripts/experiment-apparatus/replay.sh --item E171.5    # 只跑复跑表里 E171 的第 5 行（行序号是这个实验号的行在表里从 1 数的次序），给几次取并集
#
# 入口脚本（.claude/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」，C641）：可点名的项是复跑表的实验号与表里的每一行（<实验号>.<行序号>）。
#   --list-items 与「实验号写错」「行项写错」都在建本轮输出目录、truncate 镜像之前判（replay_table_experiments、replay_table_row_items 从本文件原文读复跑表）：
#   表里没有的实验号（选项只认 --admission-only、--selftest、--list-items、--item）一行都不跑、退 2；裸数字照旧由「没带 E」那一判点名。
#   --item 后面缺项名、项名不是表里的行项：列出那个实验号的行项（实验号也认不出就列实验号），退 2。
#   点到一行的行键、判法、判定库记录都与整表跑同一份：只是复跑表里那个实验号的其余行不进这一趟（--item 点到的实验号只留点名的行，没被 --item 点到的实验号照旧整个跑）。
#   本趟进度行里的记号（<实验号>.<数>）是这一趟里的先后，不是行序号；--item 点到的行开跑前在标准错误打一行对应。
#   --item 点到的实验号红了加进「上次红的清单」，绿了不从清单里划掉（只判了其中几行）。
#   卡不看参数：本脚本不领卡（文件头「GPU」），要卡的行在派行的最后一段一行一行跑，环境里有 GPU_CARDS_BY_PRIORITY 就原样交给它们。
#   每一行一条结果行（实验号、二进制、判定、说明），末尾 REPLAY_TOTAL 一行报派出、收回、现判、复用、拒算与各判定的条数。
#   弄坏开关 REPLAY_BREAK=unknown-experiment-accepted：表里没有的实验号照收（匹配 0 行、一行都不跑也退 0）。
#   弄坏开关 REPLAY_BREAK=item-unknown-accepted：表里没有的行项照收（点不到任何一行）；REPLAY_BREAK=item-row-ignored：--item 不剪行，点到的实验号整个跑。
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
#   不往固定的 scripts/experiment-apparatus/target/、<仓根>/target 编。调用方设了 CARGO_TARGET_DIR 的照它编、不删（相对路径按仓根算）：
#   文件头的会编声明「# task-server-compiles: yes」让统一任务服务给这一次尝试一份，撞内存上限由服务放大重排时给的还是同一份，下一次尝试接着增量编；
#   它归服务删（任务有了结局之后）。弄坏开关 REPLAY_BREAK=service-build-directory-ignored：不看环境里给的 CARGO_TARGET_DIR，照没给时自己建、跑完删。
#   驱动在本轮输出目录（$OUT_DIR）里拷 crates/ 去编的副本（E168 补过的那一份）编进副本自己的 target/，E171 逐行驱动的线道、E165 变异杀伤界的变体
#   也各在本轮输出目录里编；收尾时这些编译目录逐个删（replay_remove_copy_build_directories：认带签名的 CACHEDIR.TAG，不跟符号链接），产物、本轮输出目录与副本的源码照留。
#   正常结束、出错、被 TERM / INT 停都在 EXIT 钩子 replay_finish 上删：被停时先给在跑的每一行转 TERM（replay_stop → replay_one 再转给手里那一步）、等它们退；
#   删之前核本机没有进程开着里面的文件（scripts/hosts/run_build_directory.py），有就不删、报出进程、整趟退非 0。判定库、留存产物与本轮输出照留。
#   驱动里在前台一步一步起的命令（cargo run 这一类）转不到 TERM：被停时那一步还开着编译目录的，照上面不删、报出进程。
#   弄坏开关 REPLAY_BREAK=build-directory-kept、term-not-forwarded、copy-build-directories-kept、service-build-directory-ignored（自证 replay_build_directory_selftest 那六格）。
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
#     drift= broken= claims_bad= unchanged= load_skipped= not_run=（门禁 research-build-and-replay 的 experiment-replay 格读它）。
#   留存产物不在树里（按「每次提交删上一次的实验记录」归档进了版本库）的行：从 git 历史取回再逐字节比（replay_judge_fresh_product：git log --all --diff-filter=D 找最近一次删它的提交，
#   git show <提交>^:research/results/<名> 读回，放本轮输出目录；结果行写出处）；取回的照字节一致、仅计时不同、对不上判，取不回判「取不回」（跑不了，红）；留存产物不在就不比的一档不存在。
# E182 的片结果库（E182（被抛弃根读不出的三条岔路）两个段，driver_e182 与 driver_e182_units_kept）：装置读写的是目录、没有缺省的本地库（不设 E182_VERDICT_LIBRARY 退 2），
#   照两份长跑脚本（scripts/experiment-apparatus/e182-single-device-long-run.sh 文件头「片结果库」）开这一趟自己的中转目录 $OUT_DIR/e182-shards-from-prune-server/<种类>/：
#   这一趟点到这两个驱动的行时，开跑前经 admission.py library-files fetch 把 prune-server 里那两种（e182-single-device、e182-single-device-units-kept，
#   库名 <这棵树的 git common-dir>/singlefs-unit-verdicts/<种类>，与长跑脚本同一份）拉进来，两个驱动照 E182_VERDICT_LIBRARY 指到各自那一份，
#   各行收齐之后经 library-files store 并回（replay_store_e182_shards），收尾随 replay_finish 删。fetch 或 store 退 75 整趟退 75，别的非 0 整趟退 1。
#   拉与并回都在驱动外面做：驱动进行键，驱动里起 admission.py 会连上它的程序入口、这两行的键拒算。
#   弄坏开关 e182-transit-unset 下两个驱动不设 E182_VERDICT_LIBRARY。
# 输入缺时本次未跑（REPLAY_DEFERRED_ROW_ROOT）：登记了「要读 replay-frozen-roots 下哪个导出根」的行（复跑表前三列「实验号|二进制|参数」为键），导出根不在时不起驱动，
#   结果行写「本次未跑」，既不红也不绿，往 $GATE_NOT_RUN_FILE 报一行，REPLAY_TOTAL 的 not_run= 计数；导出根在了照常现跑。整趟判的全是本次未跑或本次不判（没有一行比过）退 77。
#   弄坏开关 REPLAY_BREAK=deferred-rows-ignored：不看这张表，登记的行照旧起驱动。
# 驱动要读的输入（别的实验的产物、拿来当期望值的留存产物、冻结根、导出根；research/ 可以整个删掉，所以产物缺了是常事）：驱动经 replay_inputs_present 开跑前核。
#   research/results/ 底下的缺了先从 git 历史取回、放回原处（本趟收尾删掉自己放回去的，replay_finish）；取不回，或冻结根、导出根不在，这一行记「跑不了」（红）：
#   stderr 里每缺一样一行「✗ 输入不在 test=<这一行> …」，出路在那一行里。冻结根与导出根在固定的位置：工作根下的 replay-frozen-roots/<名>（replay_frozen_root），不经环境变量指。
#   装置自己读不到输入退 77 并打 NOT_RUN_<原因> test=… 行的（研究一侧 scripts/experiment-apparatus/lib.rs 的 read_archived_products_or_exit_not_run）同样记「跑不了」，
#   那几行留在 <实验号>.err 里。整趟没有一行判得了（判的全是负载高本次不判）退 77。
# 开跑前的构建输出出两份日志（scripts/hosts/two-logs.sh）：$OUT_DIR/build.log 全量，旁边的 build.problems.log 只收 error、warning: 与位置行、非 0 退出码，行前带全量日志里的行号；
# 构建失败退 2，stderr 打第一处 error、日志末尾 20 行、日志路径、错误警告日志路径与单独重现的命令。
# GPU（test-script-discipline.md「用上现有的硬件加速」；resolve_checker_tier_features）：多机配置（scripts/crash/crash-amplification-lib.sh 文件头「配置」：
#   ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}）写了 ENABLE_GPU=1 时，起 实验包装置（用到给定状态引擎的 e156、e158、e161、e168、e170、e171 各装置）的
#   cargo run 一律带 --features gpu（数组 CHECKER_TIER_FEATURES）；开跑前先 cargo build --release -p singlefs-verification --features gpu --lib
#   试编一次，输出出两份日志落进 $OUT_DIR/gpu-build.log 与旁边的 gpu-build.problems.log。没开（文件不在、没写这个键、写 0）一个字都不变，不起这一次试编。
#   配置读不出（SINGLEFS_MULTI_HOST_CONFIG 点名的文件不在、文件不可读、读的时候出错、ENABLE_GPU 写了 0、1 之外的值），或开了 GPU 而 gpu 特性编不过：退 78，一条实验都没跑，
#   stderr 写原因与出路；不悄悄退回不带特性去跑（那样装置把判段当没编进 gpu 特性，报本次未跑或 panic）。
#   卡只归 GPU 批处理验证服务（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：本脚本不领卡、不挑卡、不清场，不另套清场包装。
#   环境里原有的 GPU_CARDS_BY_PRIORITY 开跑前先收起（不要卡的行看不到卡），派行的最后一段（要卡的行，文件头「派行」）原样交给这一段起的每一行，装置照它取卡；
#   环境里没有的不设，要卡的行照起：装置自己开不了卡时照它自己的判法报（本次未跑或跑不了），本脚本不另拦。没开 GPU 一个字都不变。
#   本脚本不交卡的行（登记在 REPLAY_DRIVERS_LEASING_THEIR_OWN_CARDS：E171 第 7、9、10、11 行整表那几趟）：照样算要卡的行（行键要带 GPU 开关：
#   它们核账时取的项与行在判定库里按开关分键，驱动闭包点到 CHECKER_TIER_FEATURES），派在要卡那一段的末尾，本脚本不交卡（不设 GPU_CARDS_BY_PRIORITY）；
#   这四行只核账（e171_take_shared_steps、e171_take_rows），不起装置。
#   GPU 开关与特性集进点到 CHECKER_TIER_FEATURES 的那几行的行键（文件头「复用」）：开关一换，只那几行现跑。开关先读（read_gpu_switch，不编），
#   试编（prepare_checker_tier_features）只在要现跑的行里有要卡的行时做：哪一行要卡照 admission.py replay-row-inputs 的第 4 列
#   （驱动闭包里点到 CHECKER_TIER_FEATURES 的行是 1，与行键里加 line:gpu 的同一判，本脚本不另判）。要现跑的行一行要卡的都没有：开了 GPU 也不试编，
#   CHECKER_TIER_FEATURES 是空的，stderr 写一句「都不点 CHECKER_TIER_FEATURES」。
# 派行（replay_row_memory_bound_fits、replay_row_is_exclusive）：同时开几行 = min(REPLAY_JOBS，这一台能给的内存 ÷ 每行留的内存)，线程不累计；只定哪几行一起起、谁先起，行键、判定与标准输出的结果行都不变（结果行照旧按表序回读）。
#   读数（scripts/experiment-apparatus/replay_row_costs.py read，开跑前判完、编译之前读一次）：这一次拿到的上限是统一任务服务交的 TASK_SERVER_MEMORY_CAP_BYTES、多机配置与 memory.max 取小（取法在 scripts/mutation/crates-mutation-rows.py 的 task_memory_cap_bytes）；
#     每一行量过的峰值与线程数在共享根下的运行记录里（它文件头「峰值记录」），只有装置退 0 的那一次才记。
#     放得下与否比的是这一台能给的内存（replay_settle_fit_memory：多机配置 SLICE_MEMORY_CAP_BY_HOST 里这一台那一项，scripts/hosts/task_memory.py 的 host_slice_cap_bytes，
#     由读数脚本 read 的 host-cap 那一行报来，本脚本不另起 python 取；取不到取这一次拿到的上限）：服务给的额度是这一次尝试的、从小处起，拿它压会把并行压到几行；
#     弄坏开关 memory-by-task-cap：照这一次拿到的上限比；host-cap-unreported：读数脚本不报 host-cap 那一行；host-cap-fallback-dropped：取不到时不退回这一次拿到的上限、不比内存。
#   独占（等别的行都收完才起，起了别的不进）：判据写 timing 的行、E181（长跑脚本自己按内存开并发，F11 带挂钟）。
#   别的行按表序并行，放得下才起：已起的每行留的内存之和（一行留 峰值 × 1.25 + 64 MiB，推的：比采样间隔还短的尖峰采不到）加底座 REPLAY_ROW_BASE_BYTES（256 MiB，推的：本脚本与监看进程）不超过这一台能给的内存；
#     没量过的行照峰值 0 留（留 64 MiB），放得下就起，撞了内存上限由统一任务服务放大重排；每行的线程不累计，每行都用满登记的每台默认重活线程数；
#     同时至多 REPLAY_JOBS 行（没设取登记的每台默认重活线程数，REPLAY_JOBS=1 就是一行一行跑）。这一台能给的内存与这一次拿到的上限都取不到的不比内存。一行都没在跑时，放不放得下都起。
#   要卡的行（admission.py replay-row-inputs 第 4 列为 1）不进上面两批，挪到最后一段：环境里有卡串就交给这一段（文件头「GPU」），按表序一行一行跑（每行独占），跑完再收起；
#     判据写 timing 的要卡的行、E181 要卡的话同样在这一段。弄坏开关 cards-shown-to-every-row（派行之前就交出卡串：不要卡的行也看得到卡）、
#     gpu-rows-not-serial（这一段几行一起起）。
#     要卡的行里本脚本不交卡的（文件头「GPU」，REPLAY_DRIVERS_LEASING_THEIR_OWN_CARDS）不进交卡的那一批：那一批跑完、卡串收起之后，按表序一行一行跑，看不到卡串。
#     弄坏开关 driver-leased-rows-held-by-replay（不看那张登记，这几行进交卡的那一批、看得到卡串）。
#   次序：可并行的先跑完，再按表序一行一行跑独占的，最后是要卡的那一段；可并行的跑完、独占的起之前等 1 分钟负载落到 REPLAY_TIMING_LOAD_RATIO 以下（计时行的负载关读它），至多等 REPLAY_TIMING_SETTLE_SECONDS 秒（默认 90）。
#   一行量过的留量比这一次拿到的上限还大（留量 + 底座 > 这一次拿到的上限）：开跑前判完就退 250（统一任务服务错误类型表的 memory-cap-hit，服务放大上限重排，task-server/task_server_lib.py 文件头「错误类型」），不编、不白跑前面的行。
#   读数脚本不在或退非 0 时，各行照没量过办、不比内存。
# 进度（标准错误；标准输出不动）：现跑的每一行起止各一行，「REPLAY_ROW <tag> 开跑」与「REPLAY_ROW <tag> 判 <结局> 用时 <秒> 秒 峰值 <字节或 没量到> 线程 <个数>｜<结果行原样>」；
#   在跑的每过 REPLAY_PROGRESS_SECONDS 秒（默认 300，.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的进度间隔）一行「REPLAY_ROW <tag> 在跑 …」，带这一行错误输出的末行。
#   被 TERM / INT 停时，已判完的行的结果行照表序打到标准输出，末行 REPLAY_STOPPED finished=<判完几行> dispatched=<这一趟几行>；判定是一行一行进判定库的，不受影响。
# 拆分（每一份去注释不超过 2000 行，上游 code-discipline.md「一个代码文件不超过 2000 行」）：本脚本留文件头、复跑表 TABLE 与准入键表 ADMISSION_ROW_KEYS（准入模块与门禁按原文读它们）、
#   读参数与选行（want、want_row）、驱动函数（复跑表第 2 列的 @<驱动函数>）与驱动开跑前编研究二进制的 build_research_driver_binary（准入模块按驱动闭包里留在本脚本里的单元原文找驱动点名的 bin、包与 GPU 特性）、
#   判一行的 replay_judge_fresh_product 与比对前删字段的几个函数、复跑一行的 replay_one；
#   其余按职责放在同目录被 source 的库里，各在本脚本里 source 它的那一行的位置载入（次序就是下面的次序）：
#   replay-inputs-and-build.sh：拷副本、驱动要读的输入、编研究二进制、GPU 开关与特性集、复跑表一行的准入键
#   replay-table-and-dispatch-limits.sh：计时行的滞后表、同时开几行的缺省与内存判法、复跑表的实验号与行项
#   replay-selftest.sh：--selftest 的全部自证（replay_selftest 与它调的各格）
#   replay-finish.sh：收尾（EXIT）、被 TERM / INT 停、监看进程的停法、被停时已判完的行
#   replay-claims.sh：结论区间断言（计时实验复跑不出同样的字节，靠这些把 kb 里的数钉住）
#   replay-admission.sh：准入那一段：向准入模块要每一行的准入键、--admission-only 只问准入（载入时执行语句）
#   replay-row-library.sh：按行复用的判定库：行键与记录的拉取、E182 片结果库、记判定、开跑前那一判（decide_row）（载入时执行语句）
#   replay-dispatch.sh：派行、进度、监看、收齐、判完的汇总与退出码（载入时执行语句）
#   弄坏开关的代码在哪份里（开关表仍是下面「弄坏开关」各条与上面各段，这里只说比较词写在哪一份；没列的写在本脚本里）：
#   replay-inputs-and-build.sh：archived-restore-off、build-log-single、build-output-dropped、copy-configuration-unset、copy-list-fixed、driver-leased-rows-held-by-replay、gpu-features-dropped、inputs-not-checked、row-key-by-experiment、thread-registry-not-copied
#   replay-table-and-dispatch-limits.sh：exclusive-row-shared、host-cap-fallback-dropped、jobs-from-nproc、memory-bound-ignored、memory-by-task-cap、row-count-unbound、threads-bound-kept、unmeasured-row-exclusive
#   replay-selftest.sh：broken-not-recorded、copy-lines-without-registry、gpu-for-every-run、guard-off、load-settle-builtin-name、mutation-driver-modules-unchecked、prune-server-failure-swallowed、red-not-recorded、row-library-failure-tolerated、run-line-quiesce-wrapped、run-lines-without-features
#   replay-finish.sh：build-directory-kept、copy-build-directories-kept、finished-rows-lost-when-stopped
#   replay-row-library.sh：deferred-rows-ignored、prune-server-failure-swallowed、reuse-ignored、reuse-key-by-row-only、row-library-failure-tolerated
#   replay-dispatch.sh：broken-not-recorded、build-everything、cards-shown-to-every-row、failed-row-peak-recorded、gpu-for-every-run、gpu-rows-not-serial、host-cap-unreported、oversized-row-not-refused、red-not-recorded、row-peak-unrecorded、row-progress-silent
#   自证（replay_selftest）读本脚本源码的几处（取驱动函数的定义、扫起装置的 cargo run 行）按 REPLAY_SOURCE_FILES（本脚本与各份库的字面路径）读；样本仓的 replay.sh 副本同目录带着这几份库。
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
REPLAY_LIBRARY_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"   # 被 source 的几份库与本脚本同目录；在下面的 cd 之前取成绝对路径
cd "$(dirname "$0")/../.." || { echo "  ✗ 进不去仓根 $(dirname "$0")/../.."; echo "     → 怎么办：从仓根照文件头的用法起：bash scripts/experiment-apparatus/replay.sh"; exit 2; }
REPLAY_REPOSITORY_ROOT="$PWD"   # replay_frozen_root 读工作根的库用绝对路径：自证在空目录里起驱动，当前目录不是仓根
REPLAY_ENTRY_SCRIPT="$REPLAY_LIBRARY_DIRECTORY/$(basename "${BASH_SOURCE[0]}")"   # 自证按字面读本脚本与它 source 的几份库：函数里的 BASH_SOURCE[0] 是函数定义所在的那一份，不是入口
REPLAY_SOURCE_FILES=("$REPLAY_ENTRY_SCRIPT" "$REPLAY_LIBRARY_DIRECTORY/replay-inputs-and-build.sh" "$REPLAY_LIBRARY_DIRECTORY/replay-table-and-dispatch-limits.sh" "$REPLAY_LIBRARY_DIRECTORY/replay-selftest.sh" "$REPLAY_LIBRARY_DIRECTORY/replay-finish.sh" "$REPLAY_LIBRARY_DIRECTORY/replay-claims.sh" "$REPLAY_LIBRARY_DIRECTORY/replay-admission.sh" "$REPLAY_LIBRARY_DIRECTORY/replay-row-library.sh" "$REPLAY_LIBRARY_DIRECTORY/replay-dispatch.sh")
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

source "$REPLAY_LIBRARY_DIRECTORY/replay-inputs-and-build.sh"
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
# 本脚本不交卡的行（文件头「GPU」「派行」）：复跑表第 2 列是 @<驱动名>、驱动名在这张登记里的行。开了 GPU 照样算要卡的行，派在要卡那一段的末尾，
# 本脚本不交卡；登记的 E171 那四行只核账、不起装置。
REPLAY_DRIVERS_LEASING_THEIR_OWN_CARDS=(driver_e171_s2_comparator driver_e171_s4_comparator driver_e171_s4_f4_comparator driver_e171_s4_f4_substitutes_comparator)
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

source "$REPLAY_LIBRARY_DIRECTORY/replay-table-and-dispatch-limits.sh"

source "$REPLAY_LIBRARY_DIRECTORY/replay-selftest.sh"
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
  replay_table_row_items
  exit 0
fi
replay_known_experiments="$(replay_table_experiments | cut -f1)"
replay_known_row_items="$(replay_table_row_items | cut -f1)"
replay_unknown_experiments=()
replay_unknown_items=()
replay_expecting_item=0
for replay_argument in "$@"; do
  if ((replay_expecting_item)); then   # --item 后面的词是项名，不当实验号
    replay_expecting_item=0
    grep -qxF -- "$replay_argument" <<<"$replay_known_row_items" || replay_unknown_items+=("$replay_argument")
    continue
  fi
  case "$replay_argument" in --admission-only) continue ;; --item) replay_expecting_item=1; continue ;; esac
  [[ "$replay_argument" =~ ^[0-9]+$ ]] && continue   # 裸数字由下面「没带 E」那一判点名
  grep -qxF -- "$replay_argument" <<<"$replay_known_experiments" || replay_unknown_experiments+=("$replay_argument")
done
if ((replay_expecting_item)); then replay_unknown_items+=("（--item 后面缺项名）"); fi
if ((${#replay_unknown_experiments[@]})) && [[ "$REPLAY_BROKEN" != unknown-experiment-accepted ]]; then
  echo "  ✗ 复跑表里没有：${replay_unknown_experiments[*]}（一行都没跑；表里 $(grep -c . <<<"$replay_known_experiments") 个实验号）" >&2
  echo "     → 怎么办：照 bash scripts/experiment-apparatus/replay.sh --list-items 列的实验号那一列写（带 E、区分大小写，例 E142、E16R5）；选项只认 --admission-only、--selftest、--list-items、--item" >&2
  exit 2
fi
if ((${#replay_unknown_items[@]})) && [[ "$REPLAY_BROKEN" != item-unknown-accepted ]]; then
  replay_item_experiment="${replay_unknown_items[0]%%.*}"
  echo "  ✗ 复跑表里没有这样的行项：${replay_unknown_items[*]}（一行都没跑）" >&2
  if grep -qxF -- "$replay_item_experiment" <<<"$replay_known_experiments"; then
    echo "     $replay_item_experiment 可点名的行项：$(replay_table_row_items | cut -f1 | grep -F -- "$replay_item_experiment." | tr '\n' ' ')" >&2   # gate-lint:detail
  else
    echo "     可点名的实验号：$(tr '\n' ' ' <<<"$replay_known_experiments")" >&2   # gate-lint:detail
  fi
  echo "     → 怎么办：--item 后面写 <实验号>.<行序号>，照 bash scripts/experiment-apparatus/replay.sh --list-items 列的第一列写（例 E171.5）" >&2
  exit 2
fi

if [[ -n "${REPLAY_OUT:-}" ]]; then
  OUT_DIR="$REPLAY_OUT"
else
  replay_temporary_root="${TMPDIR:-/tmp}"
  OUT_DIR="${replay_temporary_root:?}/singlefs-replay-$$"
fi
mkdir -p "$OUT_DIR"

# 编译目录（文件头「编译目录」）：调用方没设 CARGO_TARGET_DIR 时这一趟在 ${TMPDIR:-/tmp} 下建自己的运行目录，cargo 编进它的 target/；收尾的 EXIT 钩子
# replay_finish 先删驱动在本轮输出目录里的副本编出来的编译目录，再删这一份与运行目录。调用方设了的照它编、不删。研究二进制从 $REPLAY_RELEASE_DIRECTORY 起。
source scripts/hosts/run-build-directory.sh
REPLAY_BUILD_RUN_DIRECTORY=""
source "$REPLAY_LIBRARY_DIRECTORY/replay-finish.sh"
trap replay_finish EXIT
trap 'replay_stop 143' TERM
trap 'replay_stop 130' INT
# 弄坏开关 service-build-directory-ignored（文件头「编译目录」）：环境里给的 CARGO_TARGET_DIR 不看，照没给时自己在 TMPDIR 底下建、跑完删
[[ "$REPLAY_BROKEN" != service-build-directory-ignored ]] || unset CARGO_TARGET_DIR
if [[ -n "${CARGO_TARGET_DIR:-}" ]]; then
  # 调用方或统一任务服务给的编译目录（文件头「编译目录」）：照它编、不删，撞上限重排的下一次尝试接着用
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
E23|e23_journal_geom||e23-journal-geom-2026-11-08.out|exact
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
E83|e83-tombstone-grain||e83-tombstone-grain-2026-11-14.out|exact
E84|e84-tombstone-pinning||e84-tombstone-pinning-2026-09-03.out|exact
E85|e85-unit-header||e85-unit-header-2026-09-02.out|exact
E86|e86-scan-step||e86-scan-step-2026-09-02.out|exact
E87|e87-fixed-placement||e87-fixed-placement-2026-11-08.out|exact
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
E97|e97-entry-encoding-single-generation||e97-entry-encoding-single-generation-2026-11-14.out|exact
E98|e98-inode-record||e98-inode-record-2026-09-07.out|exact
E99|e99-writebuffer-sequence||e99-writebuffer-seq-2026-09-03.out|exact
E100|e100-system-configuration-slot||e100-system-configuration-slot-2026-09-03.out|exact
E101|e101-node-tag-reserve||e101-node-tag-reserve-2026-11-08.out|exact
E102|e102-unit-class-registry||e102-unit-class-registry-2026-11-14.out|exact
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
E139|e139_tightened_floor||e139-tightened-floor-2026-11-14.out|exact
E139|@driver_e139||e139-tightened-floor-2026-10-05.out|exact
E141|e141_switch_reserve_mount_admission||e141-switch-reserve-mount-admission-2026-09-14-row-writing.out|exact
E142|@driver_e142||e142-new-pool-file-creation-dry-run-r25-main-2026-10-13.out|exact
E142|@driver_e142_p8g_crates_dump||e142-r25-p8g-crates-dump-2026-10-13.out|exact
E143|e143-one-unit-per-txn-journal||e143-one-unit-per-txn-journal-2026-09-13.out|exact
E145|e145-self-describing-node-header||e145-self-describing-node-header-2026-09-16-tree-table-200.out|exact
E146|e146-livelist-entry-width||e146-livelist-entry-width-2026-09-16-tree-table-200.out|exact
E147|e147-system-configuration-recompute-from-layout||e147-system-configuration-recompute-from-layout-2026-09-13.out|exact
E148|e148-commit-fixpoint-two-record-trees||e148-commit-fixpoint-two-record-trees-2026-11-08.out|exact
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
E158|@driver_e158||e158-root-choice-repair-2026-11-09-segment1-rerun.out|exact
E158|@driver_e158_q3_1_g0||e158-root-choice-repair-2026-11-09-q3-1-g0.out|exact
E158|@driver_e158_q3_1_s16||e158-root-choice-repair-2026-11-09-q3-1-s16.out|exact
E158|@driver_e158_q3_1_small_ring||e158-root-choice-repair-2026-11-09-q3-1-small-ring.out|exact
E158|@driver_e158_q3_1_s4||e158-root-choice-repair-2026-11-09-q3-1-s4.out|exact
E158|@driver_e158_r3_seg1_compare||e158-root-choice-repair-2026-09-27-r3-seg1-compare.out|exact
E158|@driver_e158_r4_compare||e158-root-choice-repair-2026-09-27-r4-compare.out|exact
E181|@driver_e181||e181-landing-check-fixed-point-and-mount-lower-bound-landing.out|timing
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
E164|e164-crash-amplification-design-function-test|models|e164-crash-amplification-design-function-test-2026-09-28-models-2026-11-14.out|exact
E167|e167-multi-inflight-crash-cut|first|e167-multi-inflight-crash-cut-2026-11-14.out|exact
E167|e167-multi-inflight-crash-cut|second|e167-multi-inflight-crash-cut-second-2026-11-14.out|exact
E167|e167-multi-inflight-crash-cut|third|e167-multi-inflight-crash-cut-third-2026-11-14.out|exact
E168|@driver_e168_s1_history||e168-delete-ring-window-borrow-s1-history-2026-10-03.out|exact
E168|@driver_e168_positive_controls||e168-delete-ring-window-borrow-positive-controls-2026-10-03.out|exact
E173|e173-admission-ceiling-bad-ring-slot-dispositions|checks|e173-admission-ceiling-bad-ring-slot-dispositions-checks-2026-10-05.out|exact
E173|e173-admission-ceiling-bad-ring-slot-dispositions|first|e173-admission-ceiling-bad-ring-slot-dispositions-segment-one-2026-10-05.out|exact
E173|e173-admission-ceiling-bad-ring-slot-dispositions|second|e173-admission-ceiling-bad-ring-slot-dispositions-segment-two-2026-10-05.out|exact
E173|@driver_e173_checkpoint_cost||e173-checkpoint-cost-against-empty-publish-cost-2026-11-13-r2.out|exact
E173|e173-admission-ceiling-bad-ring-slot-dispositions|checkpoint-cost-reserve|e173-admission-ceiling-bad-ring-slot-dispositions-checkpoint-cost-reserve-2026-11-02.out|exact
E174|@driver_e174_checks||e174-lease-drop-wait-memory-return-checks-2026-10-08.out|exact
E174|@driver_e174_simulated_judge||e174-lease-drop-wait-memory-return-simulated-second-judge-2026-10-08.out|exact
E174|@driver_e174_segment_one_judge||e174-lease-drop-wait-memory-return-segment-one-judge-2026-10-08.out|exact
E174|@driver_e174_segment_two_judge||e174-lease-drop-wait-memory-return-segment-two-judge-2026-10-08.out|exact
E174|@driver_e174_run_segment_one||e174-lease-drop-wait-memory-return-segment-one-2026-10-08.out|timing
E174|@driver_e174_run_segment_two||e174-lease-drop-wait-memory-return-segment-two-2026-10-08.out|timing
E176|e176-inode-leaf-merge-capacity-and-rewrites|segment-one|e176-inode-leaf-merge-capacity-and-rewrites-segment-one-2026-11-14.out|exact
E176|e176-inode-leaf-merge-capacity-and-rewrites|segment-two|e176-inode-leaf-merge-capacity-and-rewrites-segment-two-2026-11-14.out|exact
E177|e177-per-tree-resurrection-without-tombstones|segment-one|e177-per-tree-resurrection-without-tombstones-segment-one-2026-11-14.out|exact
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
E166|e166-parallel-commit-shared-state-partition||e166-parallel-commit-shared-state-partition-r2-main-2026-11-14.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery|segment-one|e169-parallel-commit-append-and-prefix-recovery-2026-11-14.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery|segment-two|e169-parallel-commit-append-and-prefix-recovery-segment-two-2026-11-14.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery|segment-three|e169-parallel-commit-append-and-prefix-recovery-segment-three-2026-11-14.out|exact
E169|e169-parallel-commit-append-and-prefix-recovery|replay-core-inputs|e169-parallel-commit-append-and-prefix-recovery-replay-core-inputs-2026-11-14.out|exact
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
E171|@driver_e171_s2_comparator||e171-three-oracle-collusion-2026-10-03-s2-comparator-e171-r3.out|exact
E171|@driver_e171_s3_comparator_mutations||e171-three-oracle-collusion-2026-10-03-s3-comparator-mutations-2026-10-03-r7-e171-r3.out|exact
E171|@driver_e171_s4_comparator||e171-three-oracle-collusion-2026-10-03-s4-comparator-e171-r3.out|exact
E171|@driver_e171_s4_f4_comparator||e171-three-oracle-collusion-2026-10-03-s4-f4-comparator-e171-r3.out|exact
E171|@driver_e171_s4_f4_substitutes_comparator||e171-three-oracle-collusion-2026-10-03-s4-f4-substitutes-comparator-e171-r3.out|exact
E171|@driver_e171_r1_s1_revision_6_comparator||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-comparator-resume2-r6-r3-2026-10-07.out|exact
E171|@driver_e171_r1_s1_revision_6_cross_host_check||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-rows-cross-host-check-resume2-r6-r3-2026-10-07-r3.out|exact
E171|@driver_e171_r1_s1_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-05-s1-research-mutations-2026-10-07-r3.out|exact
E171|@driver_e171_r1_s2_revision_6_comparator||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-comparator-r6.out|exact
E171|@driver_e171_r1_s2_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-research-mutations-r6.out|exact
E171|@driver_e171_r1_s1_revision_8_tools_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s1-revision-8-tools-research-mutations-2026-11-02-r3.out|exact
E171|@driver_e171_r1_s2_revision_10_tools_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-s2-revision-10-tools-research-mutations.out|exact
E171|@driver_e171_r1_revision_11_tools_research_mutations||e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-08-revision-11-tools-research-mutations.out|exact
E171|@driver_e171_r1_revision_12_stream_constants||e171-r2-three-oracle-collusion-after-the-oracle-fixes-2026-11-01-stream-constants-r2-2026-11-08.out|exact
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
E183|@driver_e183||e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08.out|exact
E183|@driver_e183_anchor_check||e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08-anchor-check.out|exact
E184|@driver_e184||e184-regular-layer-cost-counts-selfcheck-2026-10-23.out|exact
E184|@driver_e184_segment_one||e184-regular-layer-cost-counts-segment-one-2026-10-24.out|timing
E183|@driver_e183_h5_anchor_check||e183-capacity-read-cache-reserve-and-sweep-costs-h5-2026-10-19-anchor-check.out|exact
E186|e186-unknown-tree-accounting-rederive||e186-unknown-tree-accounting-rederive.out.gz|exact
E186|e186-unknown-tree-accounting-rederive|--judgement dangling-u5|e186-unknown-tree-accounting-rederive-second-run-2026-10-20.out.gz|exact
E187|e187-quarantine-mark-cost|segment-one|e187-quarantine-mark-cost-segment-one-2026-11-08.out|exact
E187|e187-quarantine-mark-cost|segment-two|e187-quarantine-mark-cost-segment-two-2026-10-22.out|exact
E198|@driver_e198||e198-instance-table-pointer-birth-skip-premise-segment-one-2026-11-10.out|exact
E198|@driver_e198_segment_two||e198-instance-table-pointer-birth-skip-premise-segment-two-2026-11-10.out|exact
E199|e199-transaction-output-copy||e199-transaction-output-copy-2026-11-05-r2.out|exact
E201|@driver_e201_mutations||e201-three-oracle-collusion-full-table-on-the-current-tree-2026-11-18-r1-mutations-2026-11-18.out|exact
E203|@driver_e203_report||e203-admission-scan-cost-report-2026-11-13.out|exact
E203Y1|@driver_e203_many_unlinks||e203-admission-scan-cost-many-unlinks-completion.out|exact
E206|@driver_e206_report||e206-fixed-point-cost-report-2026-11-14.out|exact
E207|@driver_e207_report_first||e207-strict-scan-steps-report-first-2026-11-13-r4.out|exact
E207|@driver_e207_report_second||e207-strict-scan-steps-report-second-2026-11-13-r2.out|exact
E207|@driver_e207_report_third||e207-strict-scan-steps-report-third-2026-11-13-r2.out|exact
E207|@driver_e207_report_hundred_units_per_file_repeat||e207-strict-scan-steps-report-hundred-units-per-file-repeat-2026-11-13-r2.out|exact
E207|@driver_e207_report_reconciliation_split||e207-strict-scan-steps-report-reconciliation-split-2026-11-13-r2.out|exact
E207|@driver_e207_report_fourth||e207-strict-scan-steps-report-fourth-2026-11-14.out|exact
E209|@driver_e209_report_first||e209-spike-cause-report-first-2026-11-13-r3.out|exact
E209|@driver_e209_report_second||e209-spike-cause-report-second-2026-11-13-r2.out|exact
E209|@driver_e209_report_third||e209-spike-cause-report-third-2026-11-14.out|exact
E210|e210-capacity-grow-reader-rules|write-counts-and-root-level|e210-capacity-grow-reader-rules-write-counts-and-root-level-2026-11-18.out|exact
E210|e210-capacity-grow-reader-rules|reader-rules-on-histories|e210-capacity-grow-reader-rules-reader-rules-on-histories-2026-11-18.out|exact
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

source "$REPLAY_LIBRARY_DIRECTORY/replay-claims.sh"

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
  (cargo run -q -p singlefs-implementation-experiments --bin e142_new_pool_file_creation_write_dump) >"$impl_snapshot" || return 1
  "$REPLAY_RELEASE_DIRECTORY"/e142-new-pool-file-creation-dry-run "$impl_snapshot" "$arm_n15_reference" || return 1
  cat "$impl_snapshot"
}

# E142 第二十二次跑第二段（重跑登记 `research/prompts/e142-r22-prereg.md` 第五节 5.4 ⑧）：`crates/` 在两块 8 GiB 盘上的导出，
# 与 `driver_e142` 第一条命令相同，只多 `-- --device-bytes 8589934592`（导出 bin 的可选参数，不带时 4 GiB、输出不变）。
# 装置那一侧的 P8G 一段已在 `driver_e142` 的产物里（`name=p8g_` 打头的行），这一份是它要比的另一半，比对脚本 `p8g_region_compare.py`
# 在 `research/results/e142-r22-s2-comparisons-2026-10-05.out` 的附录里。
driver_e142_p8g_crates_dump() {
  (cargo run -q -p singlefs-implementation-experiments --bin e142_new_pool_file_creation_write_dump -- --device-bytes 8589934592) || return 1
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
# 整个装置就活在 crates/singlefs-implementation-experiments 里，没有 scripts/experiment-apparatus 侧的配对二进制，先例同 E142（第 371 行注释）——
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
  (set -o pipefail && E161_THREADS=31 cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e161_crash_state_dedup_and_time_split -- segment-one | grep '^E7RESULT ')
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
# `crates/singlefs-implementation-experiments/src/bin/e168_delete_ring_window_borrow.rs`。装置的行以 `E name=` 开头、收尾 `E name=done`，不用 E7RESULT 协议：
# 照 driver_e163_r1_merge 的先例在驱动里包一层（e168_wrap：行首 `E name=` 换成 `E7RESULT name=`，收尾行补 `emitted=`），入库产物就是包过的样子，
# 换回去与装置的原样输出逐字节相同（四份逐份核过）。
# s1-history 在主工作区上跑（不调删除入口）。positive-controls 与 segment1 要删除入口，主工作区那一份是桩、一调就打停机行退 3，
# positive-controls 在临时目录里拷一份 `crates/`、套删除路径补丁表 `scripts/mutation/tables/e168_delete_ring_window_borrow_delete_path.tsv`
# （每行旧串恰好命中一次，不中就整行跑不了：登记停机条款 S2）再编、跑（先例：driver_e165_segment2 在副本里改了再编）。segment1 一遍约 9 小时、36G，不进登记表，复跑照实验页「复跑」那一节手工步骤、由用户点名跑。
# 入库产物是第一段 r8 那一次在当时主工作区 `crates/` 的快照上编、跑的，快照逐文件的 sha256 在
# `e168-delete-ring-window-borrow-segment1-tree-2026-10-03.sha256`（不含补丁表）； `crates/singlefs-core/` 里的几份文件
# （extent_tree.rs、records.rs、recovery.rs 等）会随主工作区变，今天复出来对不上是新观测，先对着那份清单查。
# 计数与线程数无关（单测 segment_one_on_the_narrow_disk_with_four_slots_per_region_prints_the_same_lines_on_one_thread_and_on_eight_threads），线程数照那一次钉 4、P4 的片一次一片。
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
  (set -o pipefail && cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e168_delete_ring_window_borrow -- s1-history | e168_wrap)
}
driver_e168_positive_controls() {
  replay_inputs_present "${FUNCNAME[0]}" "留存产物：${REPLAY_ARCHIVED_REMEDY}" "research/results/e168-delete-ring-window-borrow-positive-controls-2026-10-03.out" || return $?
  local tree="$OUT_DIR/e168-positive-controls-tree" configuration
  e168_patched_tree "$tree" || return 1
  configuration="$(replay_configuration_for_the_copy)" || return 1
  (cd "$tree" && unset CARGO_TARGET_DIR && export SINGLEFS_MULTI_HOST_CONFIG="$configuration" && set -o pipefail && E168_LARGE_SLICE_THREADS=1 cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e168_delete_ring_window_borrow -- positive-controls | e168_wrap)
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
  cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin "$bin" -- "$@"
}
e170_r2_build_exports_e170() {
  local directory="$1"
  (set -o pipefail \
    && e170_r2_cargo_run e170_specification_executor_export dump --output-directory "$directory/e170-dump" >/dev/null \
    && e170_r2_cargo_run e170_specification_executor_export dump --output-directory "$directory/e170-dump-repeat" --compare-with "$directory/e170-dump" \
         | grep '^E7RESULT ' >"$directory/dump-compare.lines" \
    && cp -r "$directory/e170-dump" "$directory/e170-bad-states-second-run" \
    && e170_r2_cargo_run e170_specification_executor_export bad-states-second-run --dump-directory "$directory/e170-bad-states-second-run" --seed-hex 85441ae491419411 \
         | grep '^E7RESULT ' >"$directory/bad-states-second-run.lines")
}
e170_r2_build_exports_e171() {
  local directory="$1"
  (set -o pipefail \
    && e170_r2_cargo_run e171_three_oracle_collusion baseline --output-directory "$directory/e171-baseline-run1" >/dev/null \
    && cp -r "$directory/e171-baseline-run1" "$directory/e170-bad-states-copy-s2" \
    && e170_r2_cargo_run e170_specification_executor_export bad-states --dump-directory "$directory/e170-bad-states-copy-s2" --seed-hex 85441ae491419411 >/dev/null \
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
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates_journal_header_487)/run_replay_result_lines.py" unit-tests \
    --executor-directory "$(e170_r2_bin "$2")" --arm "$1" --scratch "$scratch/test-scratch" --output "$scratch/result.out" >&2 || return 1
  cat "$scratch/result.out"
}
driver_e170_r2_v2_487_unit_tests() { e170_r2_unit_tests v2-487 e170_o3_spec_executor_with_witness_predicates_journal_header_487; }
driver_e170_r2_v1_487_unit_tests() { e170_r2_unit_tests v1-487 e170_second_run_o3_spec_executor_journal_header_487; }
# 单测层变异：表整张跑。
e170_r2_unit_level_mutations() { # <臂名> <执行器目录名>
  local scratch="$OUT_DIR/e170-r2-unit-level-mutations-$1" table="e170_o3_spec_executor_with_witness_predicates_journal_header_487"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
  e170_r2_python "$(e170_r2_bin e170_o3_spec_executor_with_witness_predicates)/mutation_driver_unit_level.py" --executor-directory "$(e170_r2_bin "$2")" \
    --table "scripts/mutation/tables/$table.tsv" --scratch "$scratch/work" --output "$scratch/raw.out" >&2 || return 1
  e170_r2_wrap "$scratch/raw.out" "$scratch/result.out"
}
driver_e170_r2_v2_487_unit_level_mutations() { e170_r2_unit_level_mutations v2-487 e170_o3_spec_executor_with_witness_predicates_journal_header_487; }
driver_e170_r2_v1_487_unit_level_mutations() { e170_r2_unit_level_mutations v1-487 e170_second_run_o3_spec_executor_journal_header_487; }
driver_e170_r2_v2_487_z1() {
  local scratch="$OUT_DIR/e170-r2-v2-487-z1" exports
  exports="$(e170_r2_shared_exports e170)" || return 1
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
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
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
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
# O3 的执行器目录（对今天盘上格式的独立规范，journal 记录头 496 字节）与它的冻结行参照（该目录全量输出的产物，research/results/ 下、可以被归档删掉）：
# 第二段起读 O3 的几行驱动开跑前核参照，不在记跑不了（文件头「驱动要读的输入」；Python 一侧 scripts/experiment-apparatus/e171_three_oracle_collusion/frozen_o3.py
# 的 FROZEN_DIRECTORY_RELATIVE、FREEZE_REFERENCE_RELATIVE 与 FreezeReferenceMissing 同一句）。
# 冻结执行器在 E171 整表那几行里只经判定库取它的产物（共用前置 prerequisites 那一项，shared_steps.py），这里不起它；比对器读执行器产物按（流名、状态号）归位，行次序不影响结果行。
E171_FREEZE_REFERENCE="research/results/e170-o3-spec-executor-journal-header-496-2026-11-09-freeze-executor-e171-r3.out"
e171_device() {
  cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e171_three_oracle_collusion -- "$@"
}
e171_e170_exporter() {
  cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e170_specification_executor_export -- "$@"
}
driver_e171_s1_baseline() {
  local scratch="$OUT_DIR/e171-baseline"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
  (set -o pipefail && e171_device baseline --output-directory "$scratch/baseline-run1" | grep '^E7RESULT ')
}
driver_e171_s1_known_bad_images() {
  local scratch="$OUT_DIR/e171-known-bad"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
  (set -o pipefail \
    && e171_device baseline --output-directory "$scratch/baseline-run1" >/dev/null \
    && cp -r "$scratch/baseline-run1" "$scratch/e170-bad-states-copy" \
    && e171_e170_exporter bad-states --dump-directory "$scratch/e170-bad-states-copy" --seed-hex 0000000000000000 >/dev/null \
    && e171_device known-bad-images --dump-directory "$scratch/e170-bad-states-copy" --output-directory "$scratch/known-bad" --e170-bad-rows | grep '^E7RESULT ')
}
# E171 第二段（登记 `research/prompts/e171-preregistration.md` 5.4、5.5、第十二节修订）：O3 用 journal 记录头 496 字节的执行器
#（`scripts/experiment-apparatus/e170_second_run_o3_spec_executor_journal_header_496/`，冻结行在
# `e170-o3-spec-executor-journal-header-496-2026-11-09-freeze-executor-e171-r3.out`；盲测族种子 85441ae491419411 写死在各驱动里，不取自冻结行）。第一段的比对器与变异表产物（`e171-…-s1-comparator.out`、`…-s1-comparator-mutations.out`）
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
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" || return 1
  grep '^E7RESULT ' "$scratch/e170-bad-states.stdout"
}
driver_e171_s2_known_bad_images() {
  local scratch="$OUT_DIR/e171-s2-known-bad"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" && e171_s2_known_bad "$scratch" || return 1
  grep '^E7RESULT ' "$scratch/known-bad-images.stdout"
}
driver_e171_s2_images_o3() {
  local scratch="$OUT_DIR/e171-s2-images-o3"
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" && e171_s2_known_bad "$scratch" || return 1
  (e171_s2_python scripts/experiment-apparatus/e171_three_oracle_collusion/o3_bad_image_judge.py judge \
      --e170-dumps "$scratch/e170-bad-states-copy-s2" --known-bad-dir "$scratch/known-bad-s2" --output "$scratch/images-o3.out" >&2) || return 1
  grep '^E7RESULT ' "$scratch/images-o3.out"
}
driver_e171_s2_o3_control_baseline() {
  local scratch="$OUT_DIR/e171-s2-o3-control"
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
  e171_s2_prepare_bad_states "$scratch" || return 1
  (e171_s2_python scripts/experiment-apparatus/e171_three_oracle_collusion/o3_bad_image_judge.py control \
      --dumps "$scratch/baseline-run1" --e170-dumps "$scratch/e170-bad-states-copy-s2" --row-name P1-F --output "$scratch/o3-control.out" >&2) || return 1
  grep '^E7RESULT ' "$scratch/o3-control.out"
}
# 比对器那一份（第二段）：基线两趟、E170 导出装置的 dump、冻结的执行器在基线上（A3 的 O3 一份）、已知坏镜像与 E170 的 P、N、盲测各行（O2 与 O3）、关一条（O1、O2）、
# 阳性对照四条（基线与空变异各一趟）、逐行驱动全表（不给选行参数）、
# Q3-核的机制试跑（第一刀的第 8 行，关 O1、关 O2、驱动不跑执行器各一趟）；目录的名字（baseline-run1、rows-s2-trial、disable-O1……）进产物，改了名产物就变。
# E171 整表那几行（第 7、9、10、11 行用的 s2、s4、扩表、补位四趟）只核账：项与行都由 scripts/experiment-apparatus/e171-row-shard.sh 经服务分片判进判定库
# （共用前置一项一条、逐行驱动一行一条，prune-server 的 apparatus-items 族；shared_steps.py、row_driver.py 文件头「判定库」），这四个驱动一样都不现判：
# 共用前置与四条阳性对照五项经 e171_take_shared_steps 照键从判定库取（shared_steps.py 的 take：五项都在才逐项解进这个驱动的草稿目录、拷进来，名字与比对器读的相同），
# 整表与 Q3-核各趟的行经 e171_take_rows 照键取（shared_steps.py take-rows：row_driver.py 文件头「只取」，库里有键相同、判绿的才解进输出根）。
# 缺一项、缺一行，这一行记跑不了（驱动退 1），标准错误点名缺的、末尾一句出路是经服务交分片入口的整条命令（e171_library_remedy）。
# 这一条与 s4、F4、补位三条登记在 REPLAY_DRIVERS_LEASING_THEIR_OWN_CARDS（文件头「GPU」）：本脚本不替这几行持卡；它们只取、不起装置，一张卡都不领。
# 行键（scripts/gate/admission.py 的 replay_row_request 照驱动闭包里各函数的原文认 -p、--bin 与 CHECKER_TIER_FEATURES）：这几行的结局读判定库里两份装置判的项与行，
# 判它们的装置（e171_take_shared_steps、e171_take_rows 的函数体里各有一行「行键：」注释写着它的 cargo run 那一句）进行键，GPU 开关经 CHECKER_TIER_FEATURES
# 进行键（e171_take_shared_steps 读它定 --device-features）：装置或开关一变，这几行照现跑（核账），不复用上一趟的结局。
# e171_library_remedy：库里缺项缺行时这一行的出路句（✗ 与 →，带范围声明、能照抄的那一条分片入口命令），返回 1。
# 弄坏开关 REPLAY_BREAK=missing-row-remedy-silent：什么都不打、照返回 1（自证 e171-missing-row-refused 那一格转红）。
e171_library_remedy() {
  local work_root
  [[ "$REPLAY_BROKEN" == missing-row-remedy-silent ]] && return 1
  work_root="$(python3 "$REPLAY_REPOSITORY_ROOT/.claude/hooks/lib_work_root.py" 2>/dev/null)" || work_root="<工作根>"
  echo "  ✗ replay: E171 整表那几行只核账，判定库里缺上面点名的项或行，这一行记跑不了（这一趟一样都没现判）" >&2
  echo "  → 怎么办：经服务交分片入口把缺的判进库，再复跑这一行：SINGLEFS_RUN_SCOPE='种类=whole; 跑什么=E171 整表行的分片入口 e171-row-shard.sh（判库里缺的项与行、核账）; 范围=整轮全部项; 为什么=复跑表 E171 整表那几行只核账，库里缺的项与行经服务分片补判' task-server/task submit --cwd $PWD -- bash scripts/experiment-apparatus/e171-row-shard.sh --root ${work_root:-<工作根>}/e171-row-shards" >&2
  return 1
}
# e171_take_shared_steps <导出目录> <草稿目录> <E170 dump 导出在这个驱动里的名字>：共用前置与四条阳性对照五项照键从判定库取、解进 <草稿目录>/shared-steps，再拷进来
e171_take_shared_steps() {
  # 行键：判这五项的装置 cargo run -q --release -p singlefs-implementation-experiments --bin e171_three_oracle_collusion 与 --bin e170_specification_executor_export
  local features=none code
  (( ${#CHECKER_TIER_FEATURES[@]} == 0 )) || features=gpu
  if PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/shared_steps.py take --root "$2/shared-steps" --repo-root "$PWD" \
       --device-features "$features" --dumps "$1" --scratch "$2" --e170-dump-name "$3" >&2; then
    return 0
  else
    code=$?
  fi
  (( code != 4 )) || e171_library_remedy
  return 1
}
# e171_take_rows <工作根> <输出根> <row_driver.py 的选行与键的参数…>：逐行驱动的行照键从判定库取进输出根（只取，一行都不编、不起装置）；
# 标准输出是 `row …`、`table …` 那几行（比对器的 --driver-output 读 table 行），缺行退 1 并打出路。
# 弄坏开关 REPLAY_BREAK=missing-row-driver-passes：缺行（take-rows 退 4）打了出路照返回 0，驱动往下照起比对器（自证 e171-missing-row-refused 那一格转红）。
e171_take_rows() {
  # 行键：判这些行的装置 cargo run -q --release -p singlefs-implementation-experiments --bin e171_three_oracle_collusion
  local work="$1" output="$2" code
  shift 2
  if PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/shared_steps.py take-rows --repo-root "$PWD" \
       --work-root "$work" --output-root "$output" "$@"; then
    return 0
  else
    code=$?
  fi
  (( code != 4 )) || e171_library_remedy
  [[ "$REPLAY_BROKEN" == missing-row-driver-passes && "$code" == 4 ]] && return 0
  return 1
}
driver_e171_s2_comparator() {
  local scratch="$OUT_DIR/e171-s2-comparator" dumps name flag
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  dumps="$scratch/dumps"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$dumps" "$scratch/work" || return 1
  e171_take_shared_steps "$dumps" "$scratch" e170-dump || return 1
  e171_take_rows "$scratch/work" "$dumps/rows-s2-trial" --empty-mutation >"$scratch/driver-tables.log" || return 1
  for name in O1 O2 no-executor; do
    flag="--disable-oracle ${name}"
    [[ "$name" == no-executor ]] && flag="--no-executor"
    # shellcheck disable=SC2086
    (set -o pipefail && e171_take_rows "$scratch/work" "$dumps/rows-s2-q3core-$name" --rows 8 $flag | grep '^table ' >>"$scratch/driver-tables.log") || return 1
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
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion" && PYTHONDONTWRITEBYTECODE=1 python3 mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion.tsv" --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  grep '^E7RESULT ' "$scratch/mutations.out"
}
# E171 的 s4、F4、补位三行（driver_e171_s4_comparator、driver_e171_s4_f4_comparator、driver_e171_s4_f4_substitutes_comparator）共用 $OUT_DIR/e171-s4-comparator
# 这一份草稿目录与冻住的树，后一行的驱动第一步先做前一行那一段。每一段做完在 $1/.stage-$2-done 里写本趟复跑的进程号（$$，子 shell 里不变）：
# 同一趟里后一行的驱动见到前一段的记号就不重做，只读那一段留下的产物；别的趟留下的记号（进程号不同）不认，不跨趟复用。
# 三行都在同一台、同一个 $OUT_DIR 里依次跑（要卡的行互相串行），三段各做一次。
e171_s4_stage_done() { [[ "$(cat -- "$1/.stage-$2-done" 2>/dev/null)" == "$$" ]]; }   # <草稿目录> <段名>
e171_s4_stage_mark() { printf '%s\n' "$$" >"$1/.stage-$2-done"; }   # <草稿目录> <段名>
# E171 s4 整表（登记修订 5）：第二段比对器那一串前置（基线两趟、E170 导出装置 dump 与 bad-states、已知坏镜像、关一条两趟、冻结执行器在基线与坏镜像上、
# 阳性对照四条在基线与空变异各一趟），加逐行驱动全部 117 行与 Q3-核三趟，再出比对器。前置与行都只取（上面「E171 整表那几行」）：各趟都从同一份冻住的树
# 拍快照算键（--snapshot-from），三行核的是同一棵树。117 行的读数与 A4–A7 都读今天的 `crates/`：实现、O2、O1、装置或 `crates/mutations.tsv`
# 里那 117 行变了，产物就变，不是复跑坏了。目录的名字（baseline-run1、rows-s4、disable-O1、disable-O2）进产物，改了名产物就变。
driver_e171_s4_comparator() {
  local scratch="$OUT_DIR/e171-s4-comparator" dumps frozen rows name flag
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  if e171_s4_stage_done "$scratch" s4; then grep '^E7RESULT ' "$scratch/comparator.out"; return; fi
  dumps="$scratch/dumps"
  frozen="$scratch/frozen-tree"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$dumps" "$scratch/work" "$frozen" || return 1
  replay_copy_closure_into "$frozen" || return 1
  e171_take_shared_steps "$dumps" "$scratch" e170-pinned-dump || return 1
  e171_take_rows "$scratch/work" "$dumps/rows-s4" --snapshot-from "$frozen" >"$scratch/table-driver.out" || return 1
  rows=$(PYTHONDONTWRITEBYTECODE=1 python3 scripts/experiment-apparatus/e171_three_oracle_collusion/comparator.py --print-q3-core-rows \
           --rows-root "$dumps/rows-s4" --baseline-dir "$dumps/baseline-run1" | grep -o 'q3_core_rows=[0-9,]*' | cut -d= -f2)
  [[ -n "$rows" ]] || return 1
  for name in O1 O2 no-executor; do
    flag="--disable-oracle ${name}"
    [[ "$name" == no-executor ]] && flag="--no-executor"
    # shellcheck disable=SC2086
    e171_take_rows "$scratch/work" "$dumps/rows-s4-q3core-$name" --snapshot-from "$frozen" --rows "$rows" $flag >"$scratch/q3core-$name-driver.out" || return 1
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
  e171_s4_stage_mark "$scratch" s4 || return 1
  grep '^E7RESULT ' "$scratch/comparator.out"
}
# E171 F4 扩表（登记 5.2「扩表」、第十节 F4）：先整条跑 driver_e171_s4_comparator（它的草稿目录留着 s4 的导出与冻住的树），再从同一份冻住的树
# 取扩表的行（`bad-implementations-step-4-only.tsv`；E171 第 2 次重跑撤项之后 163 行），比对器把 rows-s4 与 rows-s4-f4 并成一张、
# 带 `--expansion-rows`、段名 s4-f4。读数读今天的 `crates/`，同 s4 那一条的口径。编不过的行补位那一趟见下面的 driver_e171_s4_f4_substitutes_comparator。
# 两张表的行是分片入口照编不过不补位（--keep-compile-failures-as-they-are）判进库的，这个开关不进行键（row_driver.py 文件头「判定库」）。
driver_e171_s4_f4_comparator() {
  local s4="$OUT_DIR/e171-s4-comparator" dumps
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  driver_e171_s4_comparator >/dev/null || return 1
  if e171_s4_stage_done "$s4" f4; then grep '^E7RESULT ' "$s4/comparator-f4.out"; return; fi
  dumps="$s4/dumps"
  e171_take_rows "$s4/work" "$dumps/rows-s4-f4" --snapshot-from "$s4/frozen-tree" \
    --selected-rows scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-step-4-only.tsv >"$s4/table-f4-driver.out" || return 1
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
  e171_s4_stage_mark "$s4" f4 || return 1
  grep '^E7RESULT ' "$s4/comparator-f4.out"
}
# E171 编不过的行补位（登记 5.2、第一节读法 5「替换之后编不过」）：先整条跑 driver_e171_s4_f4_comparator（它的草稿目录留着 s4 与 F4 的导出与冻住的树），
# 再从同一份冻住的树取步长 8 那张表的行进补位根（分片入口的 substitutes 那一项照补位判过：原行先编，编不过就换同一文件组的下一行、至多三次，补位跳过两张表的
# 全部名字；补位换了行的结局不入库，那几行库里没有，这一行就记跑不了），比对器把 rows-s4、rows-s4-f4 并上补位根（`--substitution-rows-root`）、段名 s4-f4-substitutes。
# 读今天的 `crates/`，同 s4 那一条的口径；补位挑中哪几行取决于 `crates/mutations.tsv` 与那几行今天编不编得过，表或实现变了换上的行就变、产物就变。
# 目录的名字（rows-s4-f4-substitutes）进产物的 config 行。
driver_e171_s4_f4_substitutes_comparator() {
  local s4="$OUT_DIR/e171-s4-comparator" dumps
  replay_inputs_present "${FUNCNAME[0]}" "冻结执行器的参照：${REPLAY_ARCHIVED_REMEDY}" "$E171_FREEZE_REFERENCE" || return $?
  driver_e171_s4_f4_comparator >/dev/null || return 1
  if e171_s4_stage_done "$s4" substitutes; then grep '^E7RESULT ' "$s4/comparator-f4-substitutes.out"; return; fi
  dumps="$s4/dumps"
  e171_take_rows "$s4/work" "$dumps/rows-s4-f4-substitutes" --snapshot-from "$s4/frozen-tree" \
    --selected-rows scripts/experiment-apparatus/e171_three_oracle_collusion/bad-implementations-step-8.tsv >"$s4/substitution-driver.out" || return 1
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
  e171_s4_stage_mark "$s4" substitutes || return 1
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
e171_r1_revision_6_python() { # <脚本的字面全路径> <参数…>：在当前目录（仓根或 e171_r1_view 搭的视图）起研究一侧的 Python；各调用处把脚本写成字面全路径，算键的闭包只收调用处点名的那几份
  (PYTHONDONTWRITEBYTECODE=1 nice -n 19 python3 -B "$@")
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
  rm -rf -- "${1:?}/view" && mkdir -p "$experiments/target" || return 1   # 与 E171_R1_VIEW 同一个目录：它就是 $1/view（相对的补上 $PWD）
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
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
  e171_r1_view "$scratch" || return 1
  (cd "$E171_R1_VIEW" && e171_r1_revision_6_python scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_one_comparator.py --baseline "$dumps/baseline-run1" --second-baseline "$dumps/baseline-run2" \
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
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
  e171_r1_view "$scratch" || return 1
  (cd "$E171_R1_VIEW" && e171_r1_revision_6_python scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/cross_host_o3_check.py compare --repo . --step-12-row-0 "$dumps/segment1-rows-resume2/row-0" \
    --cross-host-row-0 "$dumps/segment1-rows-resume2-cross-host/row-0" --step-12-product "$prefix-rows-driver-resume2-r6-r3.out" \
    --cross-host-product "$prefix-rows-cross-host-row-0-driver-resume2-r6-r3.out" --ledger "$dumps/segment1-rows-resume2-shards/row-hosts.tsv") \
    >"$scratch/compare.out" || return 1
  awk '{print} END{print "E7RESULT name=done emitted=" NR + 1}' "$scratch/compare.out"
}
driver_e171_r1_s1_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-s1-research-mutations" repository_root="$PWD"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
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
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
  e171_r1_view "$scratch" || return 1
  (cd "$E171_R1_VIEW" && e171_r1_revision_6_python scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes/segment_two_comparator.py --baseline "$dumps/revision-6-2026-10-05-r3/s2-device-baseline-streams-1-2" \
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
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_segment_two.tsv" --unit-test-module test_segment_two \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
driver_e171_r1_s1_revision_8_tools_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-s1-revision-8-tools-research-mutations" repository_root="$PWD"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_8.tsv" --unit-test-module test_revision_8 \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
# E201（三oracle合谋今树整表） 第 1 次重跑的变异那一条（重跑登记 research/prompts/e201-r1-prereg.md 第五节 5.6 步 5）：研究一侧的变异驱动对整张表在今天的树上跑，
# 六个单测模块读真仓（驱动经 E201_REPOSITORY_ROOT_FOR_A_COPY 把仓根交给副本里的单测），树动过的那一趟与留存产物不同是树的事。
# 六个单测模块逐个用 --unit-test-module 给，名字与次序同 mutation_driver.py 的 TEST_MODULES（产物 config 行的 unit_test_modules= 照这个次序拼，与不给时逐字节相同）：
# 静态核 mutation-driver-tables 从驱动正文取单测模块，TEST_MODULES 在那份驱动里跨两行写，按行取不全
driver_e201_mutations() {
  local unit="整张变异表五十七行" scratch="$OUT_DIR/e201-mutations" repository_root="$PWD"
  echo "replay: driver_e201_mutations 跑单元「$unit」：变异驱动对整张表跑一遍，与留存产物逐字节比" >&2
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e201_three_oracle_collusion_full_table_on_the_current_tree" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e201_three_oracle_collusion_full_table_on_the_current_tree.tsv" \
      --unit-test-module test_contrast_against_r1 --unit-test-module test_q2_misses --unit-test-module test_attribution \
      --unit-test-module test_whole_table_matrix --unit-test-module test_current_tree_row_driver --unit-test-module test_full_table_run \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
driver_e171_r1_s2_revision_10_tools_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-s2-revision-10-tools-research-mutations" repository_root="$PWD"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_revision_10.tsv" --unit-test-module test_revision_10 \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}

# E171 修订 11 的研究一侧变异表（这一个，与下面 driver_e171_second_rerun_current_state_library_root_research_mutations 那一张）带单测 test_revision_11 跑
# （两份登记的产物 config 行都是 unit_test_modules=test_revision_11）；那几条单测读写复用库，
# 复用库只在 prune-server 里、单测只在临时服务上跑（reuse_library.py 文件头「库」）：变异驱动整条套在临时服务底下起（客户端文件头「命令行」--with-temporary-service）
driver_e171_r1_revision_11_tools_research_mutations() {
  local scratch="$OUT_DIR/e171-r1-revision-11-tools-research-mutations" repository_root="$PWD"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
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
  (set -o pipefail && cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e171_three_oracle_collusion_after_the_oracle_fixes -- stream-constants --streams 1,2,3 | grep '^E7RESULT ')
}
# E171 第 2 次重跑（登记 research/prompts/e171-r2-prereg.md 第五节 5.3）：名单核对读工作区的 crates/mutations.tsv、改后的两张名单与开工那一刻的提交；
# 变异表或名单改了这一行会红（同 stream-constants 那一行，红了照登记读法 3 重分）。r1 逐行驱动产物从 git 对象库取（59ce5a865 里有它），文件名照原名，产物 config 行只写名字。
# 名单核对的 name=pinned_values 一行还读同目录 segment_two_comparator.py 与 count_reusable_units.py 的源文本（不载入它们），核整表行数那一行仍是照名单现算的写法：改成别的写法或读不了，那一格照实写进产物（not_derived、unreadable:…），产物随之不同。
# 四条登记行指登记修订 2 第 2.5 条 Z12 之后的重跑产物（名后接跑的那天）；作废那一趟的四份留在 research/results/，不登记复跑。
driver_e171_second_rerun_list_check() {
  local scratch="$OUT_DIR/e171-r2-list-check" repository_root="$PWD" driver_product=e171-r1-three-oracle-collusion-after-the-oracle-fixes-2026-10-07-s2-rows-driver-r6.out
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
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
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch" || return 1
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
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_list_removal.tsv" --unit-test-module test_list_check --unit-test-module test_removal_effect \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
# E171 第 2 次重跑修订 4（登记 research/prompts/e171-r2-prereg.md）：现状整表段前置那几处的两张研究一侧变异表——重写入库的 s2_compare.py 与逐行编排外层
# H16、冻结树两处运行条件（单测 test_revision_8），复用库根两台同一（单测 test_revision_11）；结果行不带耗时，exact 比。
driver_e171_second_rerun_current_state_preconditions_research_mutations() {
  local scratch="$OUT_DIR/e171-r2-current-state-preconditions-research-mutations" repository_root="$PWD"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_preconditions.tsv" --unit-test-module test_revision_8 \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
driver_e171_second_rerun_current_state_library_root_research_mutations() {
  local scratch="$OUT_DIR/e171-r2-current-state-library-root-research-mutations" repository_root="$PWD"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B \
      "$repository_root/scripts/prune-server/prune_server_client_selftest.py" --with-temporary-service -- python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_library_root.tsv" \
      --unit-test-module test_revision_11 --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
# E171 第 2 次重跑修订 5（登记 research/prompts/e171-r2-prereg.md 第 5.6 条）：第 0 步判法读宣布行的行首与调度记录（单测 test_revision_8）；exact 比。
driver_e171_second_rerun_current_state_announcement_research_mutations() {
  local scratch="$OUT_DIR/e171-r2-current-state-announcement-research-mutations" repository_root="$PWD"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_announcement.tsv" --unit-test-module test_revision_8 \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}
# E171 第 2 次重跑修订 6、修订 7、修订 8（登记 research/prompts/e171-r2-prereg.md）：线程额度在一条任务的进程里分的几处与任务里面不套线程包装（单测 test_thread_shares）、
# 第一段对照 ②③ 报造不回来与 ④ 没账时的台（单测 test_revision_8）；exact 比。
driver_e171_second_rerun_current_state_threads_and_contrast_research_mutations() {
  local scratch="$OUT_DIR/e171-r2-current-state-threads-and-contrast-research-mutations" repository_root="$PWD"
  replay_remove_directory_below "$OUT_DIR" "$scratch" && mkdir -p "$scratch/work" || return 1
  (cd "$repository_root/scripts/experiment-apparatus/e171_three_oracle_collusion_after_the_oracle_fixes" && PYTHONDONTWRITEBYTECODE=1 python3 -B mutation_driver.py \
      --table "$repository_root/scripts/mutation/tables/e171_three_oracle_collusion_after_the_oracle_fixes_current_state_threads_and_contrast.tsv" --unit-test-module test_thread_shares --unit-test-module test_revision_8 \
      --scratch "$scratch/work" --output "$scratch/mutations.out" >&2) || return 1
  cat "$scratch/mutations.out"
}

# E173 第 1 次重跑第一段（入库装置，重跑登记 research/prompts/e173-r1-prereg.md「装置写在哪」）：同 E156 的先例；
# `segment-one` 模式跑 9 个几何（S × 池大小）上的 κ 与一次空发布实际新占的固定点、两条路对拍、本地常量回比；
# 输出里只有一处与线程数相关：config 行的 `threads_of_the_geometries` = min(线程数, 21)，线程数取登记的每台默认重活线程数（31）时是 21。
# 期望产物是重跑登记 research/prompts/e173-r2-prereg.md 的 …-2026-11-13-r2.out（在这棵树的 crates/ 上现跑；与同目录 …-2026-11-02.out 那份只有 g_min_trial 两行的结局名不同，那份也留在 research/results/）。
driver_e173_checkpoint_cost() {
  (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e173_checkpoint_cost_against_empty_publish_cost -- segment-one)
}

driver_e156() {
  (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e156_allocation_basis_counts)
}

# E158（第一段，入库装置，跑前登记「装置写在哪」写死第 5 行）：同 E156 的先例，两个 cargo workspace
# 互相看不到对方，不能合并成一次调用。`all` 模式跑 5.7 常量回比、第七节锚点、S5 独立解码对拍、
# H3（岔路 3）四个几何点不注入全枚举、Q3-2/Q2-3 算术、PC3。
# 这一行承重 `e158-root-choice-repair-2026-11-09-segment1-rerun.out`（按如实账本重出：H3 四个几何点的 `peak_events_live` 与 N_w 是 0，
# PC3 判 `not_applicable`，`pc3` 一行不再打，这几行是空域，不是「打不中」；Q3-2、Q2-3 的算术行按 N_w=0 打）。`…-2026-09-25-segment1-rerun.out` 与更早的
# 原样留在 git 历史里，不是这一行比对的对象。
driver_e158() {
  (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- all)
}

# E158 第二段 Q3-1（岔路 3 候选 3 那一半）：`q3-1-g0` 模式只跑 G0 几何上的
# H3 × Φ3（|F|≤2）违例枚举，不跑第一段的 H3 全枚举（那部分归 `driver_e158`）。
# 登记的产物按如实账本（回退是向前发布，账上时间线不截、不记被抛弃的根）重出：G0/S16/小环三点都是 `nodes_visited=1371 pairs=153398`，
# S4 是 `nodes_visited=15956 pairs=1258176`，四点的 `pairs_with_any_violation` 都是 0；
# `cold_recover_with_fault_failed` 四点都是 0，`mount_writable_with_fault_failed` G0/S16/小环是 115、S4 是 1236（调用直接报错，不是「成功但违例」；两个数各等于同一份产物 `effective_root_is_not_the_tip` 的数，为什么相等没单查）。
# 新判据 `content_differs_from_its_effective_root` 与 `effective_root_off_the_timeline` 等六个量任一非 0，这一档打 `name=stop` 退 1；四点都是 0。
# 旧判据 `faulted_cold_content_differs_from_its_root` 只留着当读数（G0/S16/小环 10351，S4 93530），不判红。
# 登记的产物是 `e158-root-choice-repair-2026-11-09-{q3-1-g0,q3-1-s16,q3-1-small-ring,q3-1-s4}.out`，`…-09-25-…` 与更早的原样留在 git 历史里、不登记。
driver_e158_q3_1_g0() {
  (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-g0)
}
# 第八节敏感性（行 3）三个取样点：S16、小环（环长现算，产物里的 `small_ring_search` 那行同时钉住取到的环长）、
# S4（`sigma_length_limit=4`，比其余三点多穷举一层，代价数量级最大，real 约 18 分钟）。
driver_e158_q3_1_s16() {
  (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-s16)
}
driver_e158_q3_1_small_ring() {
  (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-small-ring)
}
driver_e158_q3_1_s4() {
  (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- q3-1-s4)
}
# E158 的 q2、r2、r3-今天 七行：它们的驱动在这里没有（依据见实验页里删这七行的那一节）。`r3-compare` 只读 `research/results/` 下九份产物、不碰 `crates/`，复跑逐字节可比；
# 产物头的 `E7INPUT name=crates_snapshot` 那一行是开工快照的汇总 sha256，比对前删掉。留存产物原样在 git 历史里，不删。
# r3-compare 与 r4-compare 读的臂产物在 research/results/ 下、可以被归档删掉（现查：r3 九臂只剩今天那一臂，r4 五段一份都不在）。装置那一侧缺一份时
# r3 打 verdict=stop 照退 0、r4 静默跳过一个字不报（crates/singlefs-implementation-experiments/src/bin/e158_root_choice_repair.rs 的 run_third_run_compare、
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
  (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- r3-compare research/results/e158-root-choice-repair-2026-09-27-r3-seg1)
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
  (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e158_root_choice_repair -- r4-compare research/results/e158-root-choice-repair-2026-09-27-r4-seg1 research/results/e158-root-choice-repair-2026-09-27-r4-seg2b research/results/e158-root-choice-repair-2026-09-27-r4-seg3 research/results/e158-root-choice-repair-2026-09-27-r4-seg4 research/results/e158-root-choice-repair-2026-09-27-r4-seg5)
}

# E181（落得下那一判与挂载下界）第一段（入库装置，跑前登记 5.5 节「第一段」）：全部 11 段走长跑脚本 scripts/experiment-apparatus/e181-landing-check-long-run.sh，十一段切成八片，
# 按它的内存预算开 F9 的试写拷贝与 F10 的走种子线程，判完的片在 prune-server 里（片键不含这个驱动），断了重起只补没判的。这一行 timing：F11 带计时，比结构。
# 片的输出与进度进这一行的 .err，标准输出只交它拼出的产物（落地那一份，每段各有收尾行）。编译归长跑脚本：有片要现判才编，全部复用就不编。
# echo 那一行不能删：行键照驱动正文里字面的 `-p <包>` 与 `--bin <名>` 收装置的 main（admission.py 文件头「复跑表每一行的行键」），长跑脚本里的变量认不出，少了它装置改了行键也不变。
# 这个装置没有 gpu 特性的用法，长跑脚本的片键也不带特性：不带 CHECKER_TIER_FEATURES。
driver_e181() {
  local products="$OUT_DIR/e181-long-run-products"
  echo "装置 cargo build --release -p singlefs-implementation-experiments --bin e181_landing_check_fixed_point_and_mount_lower_bound，有片要现判时由长跑脚本编" >&2
  rm -rf -- "${products:?}"
  E181_PRODUCT_DIRECTORY="$products" E181_LOG_DIRECTORY="$OUT_DIR/e181-long-run" bash scripts/experiment-apparatus/e181-landing-check-long-run.sh >&2 || return $?
  cat -- "$products/e181-landing-check-fixed-point-and-mount-lower-bound-landing.out"
}
# 弄坏开关 REPLAY_BREAK=e181-whole-process：载入时换掉驱动（不写进驱动正文，驱动正文进这一行的行键）
if [[ "$REPLAY_BROKEN" == e181-whole-process ]]; then
  eval 'driver_e181() { (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e181_landing_check_fixed_point_and_mount_lower_bound); }'
fi

# E182（被抛弃根读不出的三条岔路）第一段（入库装置，跑前登记 5.4「单盘择根段」）：`single-device` 模式打整份产物，没有计时字段。
# 片结果按复用键存在片结果库里（装置文件头「复用键」）：复跑时键相同的片直接取、产物逐字节相同；库不在就全部现判。
# 长跑的起法（分片、多机、产物另存的名字）在 scripts/experiment-apparatus/e182-single-device-long-run.sh 文件头。
#   装置读写的片结果库目录是这一趟的中转目录（文件头「E182 的片结果库」）：拉与并回在驱动外面做，这里只设 E182_VERDICT_LIBRARY。
driver_e182() {
  ([[ "$REPLAY_BROKEN" == e182-transit-unset ]] || export E182_VERDICT_LIBRARY="$REPLAY_E182_SHARDS/e182-single-device"
   cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e182_unreadable_abandoned_root_forks -- single-device)
}

# E182 第一段补臂「覆盖写保持 s 个单元」（登记修订 12.11）：`single-device-units-kept` 模式打整份产物，没有计时字段；复用与长跑起法同上，
# 长跑脚本是 scripts/experiment-apparatus/e182-single-device-units-kept-long-run.sh。
driver_e182_units_kept() {
  ([[ "$REPLAY_BROKEN" == e182-transit-unset ]] || export E182_VERDICT_LIBRARY="$REPLAY_E182_SHARDS/e182-single-device-units-kept"
   cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e182_unreadable_abandoned_root_forks -- single-device-units-kept)
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
# run-shard 与 merge 读写判定库，装置要代码键（--code-key-file）与转交判定库的管道：经 device-code-key.py run 起，参照、片、合并三个入口一起给
# （装置文件头「先录后判」那一条命令）；gen-input 不碰库，直接起。
e163_run_with_code_key() {
  python3 scripts/experiment-apparatus/device-code-key.py run . --binary e163-gpu-multicard-crc32c --feature e163-gpu \
    --entry e7_index_bench::reference_values_of_the_batch --entry e7_index_bench::gpu_values_of_the_shard --entry e7_index_bench::merge_lines_of \
    -- "$@"
}
driver_e163_r1_merge() {
  build_research_driver_binary e163-gpu e163-gpu-multicard-crc32c || return 1
  local bin="$REPLAY_RELEASE_DIRECTORY"/e163-gpu-multicard-crc32c
  local input="$OUT_DIR/e163-r1-merge-input.bin"
  local shard0="$OUT_DIR/e163-r1-merge-shard0.out"
  local shard1="$OUT_DIR/e163-r1-merge-shard1.out"
  "$bin" gen-input --output "$input" --units 2560 --seed 0xE163092700000001 >/dev/null || return 1
  e163_run_with_code_key run-shard --input "$input" --start 0 --count 1280 --adapter-index 0 --output "$shard0" >/dev/null || return 1
  e163_run_with_code_key run-shard --input "$input" --start 1280 --count 1280 --adapter-index 1 --output "$shard1" >/dev/null || return 1
  local merge_output line_count
  merge_output="$(e163_run_with_code_key merge --original-input "$input" --total-units 2560 --shard "$shard0" --shard "$shard1")" || return 1
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
  (cargo run -q -p singlefs-implementation-experiments --bin deletion_write_dump) >"$h1h2_snapshot" || return 1
  (cargo run -q -p singlefs-implementation-experiments --bin deletion_write_dump -- --history h3) >"$h3_snapshot" || return 1
  "$REPLAY_RELEASE_DIRECTORY"/e172-deletion-write-dry-run "$h1h2_snapshot" "$h3_snapshot" || return 1
}

# E175（产品路径上 inode 号水位退回与重发号）：装置是实验包的 bin e175_product_path_inode_watermark_regression（跑前登记
# research/prompts/e175-preregistration.md 第五节「装置写在哪」），直接在仓根 cargo run，不拷副本。跑 D 族 P 臂（8 段，D 族结果行不带挂钟），
# 结果文件由 E175_RESULT_FILE 点名、每趟先删（装置见到已有的结果文件就续写），驱动只留 E7RESULT 行并补收尾行（闸 2）。
driver_e175() {
  local result_file="$OUT_DIR/e175-d-P.tmp" worker_threads
  rm -f -- "$result_file" || return 1
  # 装置没给 E175_WORKER_THREADS 时取本机可用并行度，这里交每台默认重活线程数
  worker_threads="$(heavy_thread_default)" || return 1
  E175_ARM=P E175_FAMILY=D E175_RESULT_FILE="$result_file" E175_WORKER_THREADS="$worker_threads" \
    cargo run -q --release -p singlefs-implementation-experiments --bin e175_product_path_inode_watermark_regression >&2 || return 1
  awk '/^E7RESULT /{print; n++} END{print "E7RESULT name=done emitted=" n + 1}' "$result_file"
}

# E183（容量斜率、读缓存、小盘准入与清扫阈值的代价数）第一段（入库装置，跑前登记 research/prompts/e183-preregistration.md「装置写在哪」写死）：
# H4 的十项在一个进程里按序跑完（`all`），产物不带耗时（每项耗时与退出码打到标准错误），exact 比。先例同 E156、E158。
driver_e183() {
  (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e183_capacity_read_cache_reserve_and_sweep_costs -- all)
}

# E183 第一段 A31 的独立核对（纯 Python，不读 crates/）：锚点脚本现算每份读数的单元区容量与环窗口字节，与上面那份产物逐份比；
# 读的产物是 driver_e183 的登记产物（research/results/ 下，不是这一趟新跑出来的）。
driver_e183_anchor_check() {
  local directory="scripts/experiment-apparatus/e183_capacity_read_cache_reserve_and_sweep_costs"
  python3 -B -I "$directory/small_disk_anchor_check.py" "$directory/anchors_e183.py" research/results/e183-capacity-read-cache-reserve-and-sweep-costs-h4-2026-11-08.out
}

# E184（常规层与大负载层的代价计数）第一段（入库装置，跑前登记 research/prompts/e184-preregistration.md「装置写在哪」写死）：
# 只登记确定性的那一片：`selfcheck`（停机 T1 至 T4 与第七节锚点，产物里没有计时与线程数）。整份第一段产物（18 件）带 keys_seconds、seconds 等挂钟字段，
# 两次跑本来就不同，不登记逐字节比对；长跑的起法在 scripts/experiment-apparatus/e184-segment-one-long-run.sh 文件头，复跑命令写在实验页。
# 装置这一档的收尾行是 name=selfcheck_done，没有 name=done：照 E175 的先例在这里补 `name=done emitted=`，留存产物就是补过的样子。
# 锚 B1 对不上是登记预期（selfcheck 在这一格退 0）；停机检查或 A 开头的锚点没过装置退 3，管道带 pipefail，这一行判红。
driver_e184() {
  (cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e184_regular_layer_cost_counts -- selfcheck | awk '/^E7RESULT /{print; n++} END{print "E7RESULT name=done emitted=" n + 1}')
}

# E184 第一段整份（长跑脚本 e184-segment-one-long-run.sh 不带参数：本机把没做完的件按次序全做完、拼出产物）。工作目录与拼出的产物都放本轮输出目录里
# （E184_WORK_DIRECTORY、E184_PRODUCT_DIRECTORY：长跑脚本把拼出的产物以「e184-regular-layer-cost-counts-segment-one-<日期>.out」写进产物目录），
# 装置用这棵树里编的（--build-here），不跨趟留；产物带挂钟字段，timing 比结构。
driver_e184_segment_one() {
  local product_directory="$OUT_DIR/e184-segment-one-product"
  # echo 那一行不能删：行键照驱动正文里字面的 `-p <包>` 与 `--bin <名>` 收装置的 main（admission.py 文件头「复跑表每一行的行键」），长跑脚本里的变量认不出；
  # 这两个名字要与 e184-segment-one-long-run.sh 的 MAIN_BINARY_NAME、DUMP_BINARY_NAME 相同
  echo "装置 cargo build --release -p singlefs-implementation-experiments --bin e184_regular_layer_cost_counts --bin e184_regular_layer_cost_counts_write_dump，由长跑脚本 --build-here 编" >&2
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
  (set -o pipefail && cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e198_instance_table_pointer_birth_skip_premise -- segment-one | grep '^E7RESULT ')
}

driver_e198_segment_two() {
  (set -o pipefail && cargo run -q --release -p singlefs-implementation-experiments ${CHECKER_TIER_FEATURES[@]+"${CHECKER_TIER_FEATURES[@]}"} --bin e198_instance_table_pointer_birth_skip_premise -- segment-two | grep '^E7RESULT ')
}

# E203（准入之前扫类的步代价）合并报告（跑前登记 research/prompts/e203-preregistration.md 第十二节修订 7、8）：18 个点各两个构建的 36 份产物重算一遍。
# 输入：树里有的取树里的，树里没有的取提交 84b4ae23f 里那一份，再没有取最近一次删它的提交之前那一版，都拷进这一行自己的临时目录；都取不到这一行记跑不了。
# 模型的报告行以 name= 起头，这里加 E7RESULT 前缀与收尾行（完整性闸 2 要的形状），exact 比。
e203_report_input() { # <产物名> <放到哪个目录>
  local name="$1" target="$2/$1"
  if [[ -f "research/results/$name" ]]; then cp -- "research/results/$name" "$target"; return $?; fi
  if git show "84b4ae23f:research/results/$name" >"$target" 2>/dev/null && [[ -s "$target" ]]; then return 0; fi
  if replay_archived_product_copy "$name" "$target" >/dev/null; then return 0; fi
  printf '  ✗ 输入不在 test=driver_e203_report %s 不在（树里没有、提交 84b4ae23f 里没有、git 历史里也取不回），这一行记跑不了\n' "$name" >&2
  printf '     → 怎么办：%s\n' "$REPLAY_ARCHIVED_REMEDY" >&2
  return 1
}
driver_e203_report() {
  local inputs="$OUT_DIR/e203-report-inputs" build point name
  local -a counts=() timing=()
  rm -rf -- "$inputs" && mkdir -p "$inputs" || return 1
  for build in counts timing; do
    for point in base data_units_1 data_units_100 data_units_1000 data_units_10000 diagonal files_1 files_100 files_1000 files_10000 unit_area_2_pow_14 unit_area_2_pow_20 unit_area_2_pow_24 unlinks_1 unlinks_100 unlinks_1000 unlinks_10000; do
      name="e203-admission-scan-cost-$build-$point-2026-11-07.out"
      e203_report_input "$name" "$inputs" || return 1
      if [[ "$build" == counts ]]; then counts+=("$inputs/$name"); else timing+=("$inputs/$name"); fi
    done
    name="e203-admission-scan-cost-$build-user_scale_unit_area-2026-11-07-r2.out"
    e203_report_input "$name" "$inputs" || return 1
    if [[ "$build" == counts ]]; then counts+=("$inputs/$name"); else timing+=("$inputs/$name"); fi
  done
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e203-admission-scan-cost report "${counts[@]}" --timing "${timing[@]}" \
    | awk '{ print "E7RESULT " $0; emitted++ } END { print "E7RESULT name=done emitted=" (emitted + 1) }')
}

# E203 的 Y1（跑前登记第十二节修订 6）：表里单列实验号 E203Y1，点名 E203 不点到它，只有不带实验号的全量复跑与点名 E203Y1 才跑。两个构建依次起入口，判定照入口的单元键进判定库，之后照键复用；
# 源树是修订 7 那一趟的冻结副本（停机条款第 3 条）。入口的输出进 stderr；产物由入口写。这一行只比两个构建都做完（完成标记两行加收尾行），读数由实验页照原判据再判一次。
driver_e203_many_unlinks() {
  local frozen_source build
  frozen_source="$(replay_frozen_root e203-segment-two-source)" || return 1
  replay_inputs_present "${FUNCNAME[0]}" "冻结根：照实验页 E203（准入之前扫类的步代价）「复跑」用的那棵冻结源树拷一份到这个位置" "$frozen_source" || return $?
  for build in counts timing; do
    bash scripts/experiment-apparatus/e203-admission-scan-cost-run.sh --build "$build" --work "$OUT_DIR/e203-many-unlinks-$build" --source "$frozen_source" --item many_unlinks_unit_area_2_pow_16 >&2 || return $?
    printf 'E7RESULT name=item_done build=%s point=many_unlinks_unit_area_2_pow_16\n' "$build"
  done
  printf 'E7RESULT name=done emitted=3\n'
}

# E206（发布固定点轮数的代价）合并报告（跑前登记 research/prompts/e206-preregistration.md 第十二节修订 1）：第一段 40 份产物（16 个点的 p1、p2a，三个点的 single_step、closure_off，两个点的 plain）重算一遍。
# 输入：树里有的取树里的，树里没有的取最近一次删它的提交之前那一版，都拷进这一行自己的临时目录；都取不到这一行记跑不了。
# 模型的报告行以 name= 起头，这里加 E7RESULT 前缀与收尾行（完整性闸 2 要的形状），exact 比；留存产物（登记的那一份）是装置重算的整份（比 -r2 多 38 行 by_delta_pooled_fallback 与收尾行条数，别的行逐字不变）；-r2 是任务 11102 的合并报告（同目录不带 -r2 的那份）照同一个 awk 加了前缀与收尾行，留着。
e206_report_input() { # <产物名> <放到哪个目录>
  local name="$1" target="$2/$1"
  if [[ -f "research/results/$name" ]]; then cp -- "research/results/$name" "$target"; return $?; fi
  if replay_archived_product_copy "$name" "$target" >/dev/null; then return 0; fi
  printf '  ✗ 输入不在 test=driver_e206_report %s 不在（树里没有、git 历史里也取不回），这一行记跑不了\n' "$name" >&2
  printf '     → 怎么办：%s\n' "$REPLAY_ARCHIVED_REMEDY" >&2
  return 1
}
driver_e206_report() {
  local inputs="$OUT_DIR/e206-report-inputs" point arm name
  local -a names=() products=()
  rm -rf -- "$inputs" && mkdir -p "$inputs" || return 1
  for point in data_units_100 data_units_1000 data_units_10000 files_10000 files_1000_unit_area_2_pow_14 files_1000_unit_area_2_pow_20 files_1000_unit_area_2_pow_24 files_1000_units_100 files_1000_units_10 files_30000 unlinks_100 unlinks_1000 unlinks_10000; do
    for arm in p1 p2a; do names+=("e206-fixed-point-cost-$point-$arm-2026-11-09.out"); done
  done
  for point in base files_100 files_1000; do
    for arm in p1 p2a single_step closure_off; do names+=("e206-fixed-point-cost-$point-$arm-2026-11-09.out"); done
  done
  names+=("e206-fixed-point-cost-base-plain-2026-11-09.out" "e206-fixed-point-cost-files_1000-plain-2026-11-09.out")
  while IFS= read -r name; do
    e206_report_input "$name" "$inputs" || return 1
    products+=("$inputs/$name")
  done < <(printf '%s\n' "${names[@]}" | LC_ALL=C sort)
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e206-fixed-point-cost report "${products[@]}" \
    | awk '{ print "E7RESULT " $0; emitted++ } END { print "E7RESULT name=done emitted=" (emitted + 1) }')
}

# E207（严格判据下的扫类步数）第一段合并报告（跑前登记 research/prompts/e207-preregistration.md 第五节与执行员修订 4–6）：第一段 14 份产物（12 个点的 head、base 与 files_1000 的 plain）加 16 份 E206 p2a 产物（回扫）重算 `report --segment first`。
# 输入同 E206 那一行：树里有的取树里的，树里没有的取最近一次删它的提交之前那一版，都拷进这一行自己的临时目录；都取不到这一行记跑不了。
# 模型的报告行以 name= 起头，这里加 E7RESULT 前缀与收尾行（完整性闸 2 要的形状），exact 比；留存产物 -r4 是同目录 -r3 那份原始报告（档名写全后用新装置重出的）照同一个 awk 加了前缀与收尾行，行号不变。
driver_e207_report_first() {
  local inputs="$OUT_DIR/e207-report-inputs" point name
  local -a names=() retro_names=() products=() retro_products=()
  rm -rf -- "$inputs" && mkdir -p "$inputs" || return 1
  for point in base files_1000 files_10000 files_1000_units_100 files_100_units_100 files_10_units_1000 files_100_units_1000 files_10_units_10000 \
    files_then_big_1000 files_then_big_10000 files_10_units_1000_unit_area_2_pow_17 files_100_units_1000_unit_area_2_pow_20; do
    names+=("e207-strict-scan-steps-$point-head-2026-11-13.out")
  done
  names+=("e207-strict-scan-steps-base-plain-2026-11-13.out" "e207-strict-scan-steps-files_1000-plain-2026-11-13.out")
  for point in base data_units_100 data_units_1000 data_units_10000 files_100 files_1000 files_10000 files_1000_units_10 files_1000_units_100 files_30000 \
    files_1000_unit_area_2_pow_14 files_1000_unit_area_2_pow_20 files_1000_unit_area_2_pow_24 unlinks_100 unlinks_1000 unlinks_10000; do
    retro_names+=("e206-fixed-point-cost-$point-p2a-2026-11-09.out")
  done
  while IFS= read -r name; do
    e206_report_input "$name" "$inputs" || return 1
    products+=("$inputs/$name")
  done < <(printf '%s\n' "${names[@]}" | LC_ALL=C sort)
  while IFS= read -r name; do
    e206_report_input "$name" "$inputs" || return 1
    retro_products+=("$inputs/$name")
  done < <(printf '%s\n' "${retro_names[@]}" | LC_ALL=C sort)
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e207-strict-scan-steps report --segment first "${products[@]}" --retro "${retro_products[@]}" \
    | awk '{ print "E7RESULT " $0; emitted++ } END { print "E7RESULT name=done emitted=" (emitted + 1) }')
}

# E207 第二段合并报告（跑前登记第五节「第二段」与第十二节执行员修订 7）：第一段 `files_1000_units_100` 的 head（复现钉的 `standard` 探针）加第二段三份探针产物（cold_first、create_first、
# young_first）重算 `report --segment second`；输入的取法同 driver_e207_report_first，加前缀与收尾行的 awk 同它，留存产物 -r2 是同目录不带 -r2 那份原始报告照这个 awk 加了前缀与收尾行。
driver_e207_report_second() {
  local inputs="$OUT_DIR/e207-report-second-inputs" name
  local -a names=("e207-strict-scan-steps-files_1000_units_100-head-2026-11-13.out" "e207-strict-scan-steps-files_1000_units_100-head-cold_first-2026-11-13.out" \
    "e207-strict-scan-steps-files_1000_units_100-head-create_first-2026-11-13.out" "e207-strict-scan-steps-files_1000_units_100-head-young_first-2026-11-13.out") products=()
  rm -rf -- "$inputs" && mkdir -p "$inputs" || return 1
  for name in "${names[@]}"; do
    e206_report_input "$name" "$inputs" || return 1
    products+=("$inputs/$name")
  done
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e207-strict-scan-steps report --segment second "${products[@]}" \
    | awk '{ print "E7RESULT " $0; emitted++ } END { print "E7RESULT name=done emitted=" (emitted + 1) }')
}

# E207 第三段合并报告（跑前登记第五节「第三段」）：`files_30000` 的 head（带扩展窗口 W5–W7）一份产物重算 `report --segment third`；输入与前缀、收尾行同 driver_e207_report_second，
# 留存产物 -r2 是同目录不带 -r2 那份原始报告照这个 awk 加了前缀与收尾行。
driver_e207_report_third() {
  local inputs="$OUT_DIR/e207-report-third-inputs" name="e207-strict-scan-steps-files_30000-head-2026-11-13.out"
  rm -rf -- "$inputs" && mkdir -p "$inputs" || return 1
  e206_report_input "$name" "$inputs" || return 1
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e207-strict-scan-steps report --segment third "$inputs/$name" \
    | awk '{ print "E7RESULT " $0; emitted++ } END { print "E7RESULT name=done emitted=" (emitted + 1) }')
}

# E207 第四段两个小点的合并报告（任务 16080 那一条命令）：低端点 `files_10_units_1000_unit_area_2_pow_21` 的 head 原始产物与中点 `files_100_units_1000_unit_area_2_pow_21` 的 head 各一份，重算 `report --segment fourth`；
# 输入的取法与前缀、收尾行同 driver_e207_report_third；留存产物 -r2 是同目录不带 -r2 那份原始报告（760 行）照这个 awk 加了前缀与收尾行（761 行），行号不变。
driver_e207_report_fourth() {
  local inputs="$OUT_DIR/e207-report-fourth-inputs" name
  local -a names=("e207-strict-scan-steps-files_10_units_1000_unit_area_2_pow_21-head-2026-11-13.out" "e207-strict-scan-steps-files_100_units_1000_unit_area_2_pow_21-head-2026-11-13.out") products=()
  rm -rf -- "$inputs" && mkdir -p "$inputs" || return 1
  for name in "${names[@]}"; do
    e206_report_input "$name" "$inputs" || return 1
    products+=("$inputs/$name")
  done
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e207-strict-scan-steps report --segment fourth "${products[@]}" \
    | awk '{ print "E7RESULT " $0; emitted++ } END { print "E7RESULT name=done emitted=" (emitted + 1) }')
}

# E209（扫类尖峰成因）第一段合并报告（跑前登记 research/prompts/e209-preregistration.md 第五节「第一段」）：训练点 files_100_units_100 的 head 五份（first_mount_first、first_repeated、
# mount_then_first、mount_twice_first、walk_forward）、重出的孪生项（-r2，旧孪生那份缺 name=point 一行，不进输入）、plain 的 walk_forward，共 7 份产物重算 `report`。
# 输入都在树里（取不到的走 e206_report_input 的归档回取）；--crates-root 取仓根，报告里行表各行在 crates/ 里的函数行号现取（crates/ 里那几个函数挪了行，这一行会红，是该红的）。
# 模型的报告行以 name= 起头，这里加 E7RESULT 前缀与收尾行（完整性闸 2 要的形状），exact 比；留存产物 -r3 是模型原始报告照同一个 awk 加了前缀与收尾行的形态。
driver_e209_report_first() {
  local inputs="$OUT_DIR/e209-report-first-inputs" name
  local -a names=() products=()
  rm -rf -- "$inputs" && mkdir -p "$inputs" || return 1
  for name in first_mount_first first_repeated mount_then_first mount_twice_first walk_forward; do
    names+=("e209-spike-cause-files_100_units_100-head-$name-2026-11-13.out")
  done
  names+=("e209-spike-cause-files_100_units_100-head-walk_forward-twin-2026-11-13-r2.out" "e209-spike-cause-files_100_units_100-plain-walk_forward-2026-11-13.out")
  for name in "${names[@]}"; do
    e206_report_input "$name" "$inputs" || return 1
    products+=("$inputs/$name")
  done
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e209-spike-cause report --crates-root "$REPLAY_REPOSITORY_ROOT" "${products[@]}" \
    | awk '{ print "E7RESULT " $0; emitted++ } END { print "E7RESULT name=done emitted=" (emitted + 1) }')
}

# E209（扫类尖峰成因）第二段合并报告（跑前登记第五节「第二段」）：训练点 files_100_units_100 的第一段 7 份（head 五份、孪生项 -r2、plain 的 walk_forward）加第二段 20 份（head），共 27 份产物重算 `report`。
# 旧孪生那份（缺 name=point 一行）不进输入。其余同 driver_e209_report_first：输入走 e206_report_input，--crates-root 取仓根，加 E7RESULT 前缀与收尾行，exact 比；留存产物是 -r2。
driver_e209_report_second() {
  local inputs="$OUT_DIR/e209-report-second-inputs" name
  local -a names=() products=()
  rm -rf -- "$inputs" && mkdir -p "$inputs" || return 1
  for name in first_mount_first first_repeated mount_then_first mount_twice_first walk_forward; do
    names+=("e209-spike-cause-files_100_units_100-head-$name-2026-11-13.out")
  done
  names+=("e209-spike-cause-files_100_units_100-head-walk_forward-twin-2026-11-13-r2.out" "e209-spike-cause-files_100_units_100-plain-walk_forward-2026-11-13.out")
  for name in walk_backward walk_from_middle create_then_first filler_create_1 filler_create_2 filler_create_4 filler_create_8 filler_create_16 \
    filler_young_1 filler_young_4 filler_young_8 presweep_1 presweep_2 presweep_4 presweep_8 presweep_16 presweep_32 presweep_64 young_mount_first old_mount_old; do
    names+=("e209-spike-cause-files_100_units_100-head-$name-2026-11-13.out")
  done
  for name in "${names[@]}"; do
    e206_report_input "$name" "$inputs" || return 1
    products+=("$inputs/$name")
  done
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e209-spike-cause report --crates-root "$REPLAY_REPOSITORY_ROOT" "${products[@]}" \
    | awk '{ print "E7RESULT " $0; emitted++ } END { print "E7RESULT name=done emitted=" (emitted + 1) }')
}

# E209（扫类尖峰成因）第三段合并报告（跑前登记第五节「第三段」）：第二段的 27 份加第三段 48 份（五个检验点各自的 head 项：files_10_units_1000 八项，其余四点各十项），共 75 份产物重算 `report`；留存文件是 e209-spike-cause-report-third-2026-11-13.out（合并报告落盘之前这一行不要跑）。
driver_e209_report_third() {
  local inputs="$OUT_DIR/e209-report-third-inputs" name point script
  local -a names=() products=()
  rm -rf -- "$inputs" && mkdir -p "$inputs" || return 1
  for name in first_mount_first first_repeated mount_then_first mount_twice_first walk_forward; do
    names+=("e209-spike-cause-files_100_units_100-head-$name-2026-11-13.out")
  done
  names+=("e209-spike-cause-files_100_units_100-head-walk_forward-twin-2026-11-13-r2.out" "e209-spike-cause-files_100_units_100-plain-walk_forward-2026-11-13.out")
  for name in walk_backward walk_from_middle create_then_first filler_create_1 filler_create_2 filler_create_4 filler_create_8 filler_create_16 \
    filler_young_1 filler_young_4 filler_young_8 presweep_1 presweep_2 presweep_4 presweep_8 presweep_16 presweep_32 presweep_64 young_mount_first old_mount_old; do
    names+=("e209-spike-cause-files_100_units_100-head-$name-2026-11-13.out")
  done
  for point in files_75_units_100 files_200_units_100 files_100_units_100_unit_area_2_pow_17 files_100_units_100_unit_area_2_pow_20; do
    for script in filler_create_4 filler_young_4 first_mount_first first_repeated mount_then_first mount_twice_first presweep_32 presweep_8 walk_backward walk_forward; do
      names+=("e209-spike-cause-$point-head-$script-2026-11-13.out")
    done
  done
  for script in filler_create_4 filler_young_4 first_mount_first first_repeated mount_then_first mount_twice_first walk_backward walk_forward; do
    names+=("e209-spike-cause-files_10_units_1000-head-$script-2026-11-13.out")
  done
  for name in "${names[@]}"; do
    e206_report_input "$name" "$inputs" || return 1
    products+=("$inputs/$name")
  done
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e209-spike-cause report --crates-root "$REPLAY_REPOSITORY_ROOT" "${products[@]}" \
    | awk '{ print "E7RESULT " $0; emitted++ } END { print "E7RESULT name=done emitted=" (emitted + 1) }')
}

# E207 复量段合并报告（登记设计员修订 2）：复量段五项产物（四个新点加 files_100_units_100 再跑一次）加第一段 files_1000_units_100 的 head，钉对第一段 files_100_units_100 的 head；
# 复量段的产物同天同名已有的第一段那份，所以 files_100_units_100 这一份是 -r2（入口 new_product_name 的拼法）。登记行比的留存产物是加了 E7RESULT 前缀与收尾行的形态（同上）。
driver_e207_report_hundred_units_per_file_repeat() {
  local inputs="$OUT_DIR/e207-report-hundred-units-per-file-repeat-inputs" name
  local -a names=("e207-strict-scan-steps-files_56_units_100-head-2026-11-13.out" "e207-strict-scan-steps-files_100_units_100-head-2026-11-13-r2.out" \
    "e207-strict-scan-steps-files_178_units_100-head-2026-11-13.out" "e207-strict-scan-steps-files_316_units_100-head-2026-11-13.out" \
    "e207-strict-scan-steps-files_562_units_100-head-2026-11-13.out" "e207-strict-scan-steps-files_1000_units_100-head-2026-11-13.out" \
    "e207-strict-scan-steps-files_100_units_100-head-2026-11-13.out") products=()
  rm -rf -- "$inputs" && mkdir -p "$inputs" || return 1
  for name in "${names[@]}"; do
    e206_report_input "$name" "$inputs" || return 1
    products+=("$inputs/$name")
  done
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e207-strict-scan-steps report --segment hundred_units_per_file_repeat "${products[@]:0:6}" --pin-against "${products[6]}" \
    | awk '{ print "E7RESULT " $0; emitted++ } END { print "E7RESULT name=done emitted=" (emitted + 1) }')
}

# E207 对账分辨段合并报告（登记设计员修订 3）：四项新产物（base、files_1000 各 head 与 plain；同天同名已有第一段那份，所以都是 -r2）加第一段 base、files_1000、files_10000 的 head（--previous）
# 与 E206 的 base、files_1000、files_10000 三份 p2a 产物（--retro）。登记行比的留存产物是加了 E7RESULT 前缀与收尾行的形态。
driver_e207_report_reconciliation_split() {
  local inputs="$OUT_DIR/e207-report-reconciliation-split-inputs" name point
  local -a products=() previous=() retro=()
  rm -rf -- "$inputs" && mkdir -p "$inputs" || return 1
  for name in e207-strict-scan-steps-base-head-2026-11-13-r2.out e207-strict-scan-steps-base-plain-2026-11-13-r2.out \
    e207-strict-scan-steps-files_1000-head-2026-11-13-r2.out e207-strict-scan-steps-files_1000-plain-2026-11-13-r2.out; do
    e206_report_input "$name" "$inputs" || return 1
    products+=("$inputs/$name")
  done
  for point in base files_1000 files_10000; do
    name="e207-strict-scan-steps-$point-head-2026-11-13.out"
    e206_report_input "$name" "$inputs" || return 1
    previous+=("$inputs/$name")
    name="e206-fixed-point-cost-$point-p2a-2026-11-09.out"
    e206_report_input "$name" "$inputs" || return 1
    retro+=("$inputs/$name")
  done
  (set -o pipefail && "$REPLAY_RELEASE_DIRECTORY"/e207-strict-scan-steps report --segment reconciliation_split "${products[@]}" --previous "${previous[@]}" --retro "${retro[@]}" \
    | awk '{ print "E7RESULT " $0; emitted++ } END { print "E7RESULT name=done emitted=" (emitted + 1) }')
}
# --admission-only：只问准入（scripts/gate/admission.py，登记表 .claude/gate.d/stage-inputs.tsv），不编、不跑、不比。
ADMISSION_ONLY=0
ONLY=()
ITEM_ARGUMENTS=()
replay_expecting_item=0
for replay_argument in "$@"; do
  if ((replay_expecting_item)); then ITEM_ARGUMENTS+=("$replay_argument"); replay_expecting_item=0
  elif [[ "$replay_argument" == "--admission-only" ]]; then ADMISSION_ONLY=1
  elif [[ "$replay_argument" == "--item" ]]; then replay_expecting_item=1
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
# --item <实验号.行序号>（文件头「入口脚本」）：行序号是这个实验号的行在复跑表里从 1 数的次序。点到的实验号进 ONLY（want 认它），
# 并且只留点名的行（want_row）；没被 --item 点到的实验号照旧整个跑。行的身份是「实验号|二进制|参数」（复跑表前三列），与按行复用的行键用同一个身份。
declare -A ITEM_EXPERIMENT=() ITEM_ROW=() ROW_ORDINAL=() replay_ordinal_of_experiment=()
while IFS='|' read -r exp bin args _rest; do
  [[ "$exp" == E* ]] || continue
  replay_ordinal_of_experiment["$exp"]=$(( ${replay_ordinal_of_experiment["$exp"]:-0} + 1 ))
  ROW_ORDINAL["$exp|$bin|$args"]="${replay_ordinal_of_experiment["$exp"]}"
done <<<"$TABLE"
for replay_item in ${ITEM_ARGUMENTS[@]+"${ITEM_ARGUMENTS[@]}"}; do
  ITEM_EXPERIMENT["${replay_item%.*}"]=1
  [[ " ${ONLY[*]-} " == *" ${replay_item%.*} "* ]] || ONLY+=("${replay_item%.*}")
  for replay_identity in "${!ROW_ORDINAL[@]}"; do
    [[ "${replay_identity%%|*}" == "${replay_item%.*}" && "${ROW_ORDINAL["$replay_identity"]}" == "${replay_item##*.}" ]] && ITEM_ROW["$replay_identity"]=1
  done
done
want_row() { # <实验号> <二进制> <参数>：复跑表的这一行这一趟要不要判（弄坏开关 item-row-ignored：--item 点到的实验号整个判）
  want "$1" || return 1
  [[ -n "${ITEM_EXPERIMENT["$1"]+set}" ]] || return 0
  [[ "$REPLAY_BROKEN" == item-row-ignored ]] && return 0
  [[ -n "${ITEM_ROW["$1|$2|$3"]+set}" ]]
}

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
source "$REPLAY_LIBRARY_DIRECTORY/replay-admission.sh"
source "$REPLAY_LIBRARY_DIRECTORY/replay-row-library.sh"

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
    want_row "$exp" "$bin" "$args" || continue
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


pass=0; drift=0; timing_only=0; broken=0; claim_bad=0; unchanged=0; UNCHANGED_ROWS=()
reused=0; REUSED_ROWS=(); refused_rows=0   # 按行复用的行（文件头「按行复用」）；行键拒算的行
load_skipped=0; load_passed=0; load_lagged=0; LOAD_SKIPPED_ROWS=()   # 计时行的负载关：本次不判、放行（含滞后）、其中滞后；本次不判的逐个记名字与比值
not_run=0; NOT_RUN_ROWS=()   # 输入缺而登记了「本次未跑」的行（REPLAY_DEFERRED_ROW_ROOT）：既不红也不绿
# 输入缺时本次未跑的行：键是复跑表那一行的前三列「实验号|二进制|参数」，值是 replay-frozen-roots 下它要读的导出根的名字。
# E171 复跑表里读 r1 导出根的三行，导出根要等里程碑四收口做全量时重建（.claude/kb/milestone/04-fourth-txn.md 八·8）；根不在这一行记本次未跑，根在了照常现跑。
# 弄坏开关 REPLAY_BREAK=deferred-rows-ignored：不看这张表，三行照旧起驱动、缺输入退 1 记跑不了。
declare -A REPLAY_DEFERRED_ROW_ROOT=(
  ["E171|@driver_e171_r1_s1_revision_6_comparator|"]=e171-r1-dumps
  ["E171|@driver_e171_r1_s1_revision_6_cross_host_check|"]=e171-r1-dumps
  ["E171|@driver_e171_r1_s2_revision_6_comparator|"]=e171-r1-dumps
)
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
  rm -f "$fresh" "$OUT_DIR/$tag.load" "$OUT_DIR/$tag.skip" "${OUT_DIR:?}/${tag:?}.stored" "$OUT_DIR/$tag.line" "$OUT_DIR/$tag.verdict" "$OUT_DIR/$tag.claim"
  [[ -n "$reuse_key" && -n "$REPLAY_ROW_LIBRARY" ]] && reuse_record="$REPLAY_ROW_RECORDS/keys/$reuse_key"
  # 手里那一步放后台、wait 等它（文件头「编译目录」）：父进程 replay_stop 给这一行转 TERM 时 trap 当场跑，再转给它（`&` 起、`$!` 记下的那一个进程）、等它退，退 143。
  # 弄坏开关 REPLAY_BREAK=term-not-forwarded：不转（这一行自己退了，手里那一步照跑）
  local payload=""
  [[ "$REPLAY_BROKEN" == term-not-forwarded ]] || trap '[[ -z "$payload" ]] || { kill -TERM "$payload" 2>/dev/null; wait "$payload" 2>/dev/null; }; replay_row_tell_sampler "$tag" end "143 1"; exit 143' TERM
  local started_at="${EPOCHREALTIME/[.,]/}" row_threads wall_milliseconds
  replay_row_progress_line "REPLAY_ROW $tag 开跑"
  if [[ "$bin" == @* ]]; then
    "${bin#@}" >"$fresh" 2>"$OUT_DIR/$tag.err" &
  else
    # shellcheck disable=SC2086
    "$REPLAY_RELEASE_DIRECTORY/$bin" $args >"$fresh" 2>"$OUT_DIR/$tag.err" &
  fi
  payload=$!
  replay_row_tell_sampler "$tag" payload "$payload"$'\t'"$identity"
  wait "$payload"
  rc=$?
  payload=""
  times >"$OUT_DIR/$tag.times"   # 不进命令替换：子 shell 里读到的子进程用时是 0
  wall_milliseconds=$(( (${EPOCHREALTIME/[.,]/} - started_at) / 1000 ))
  row_threads="$(replay_row_threads_used "$OUT_DIR/$tag.times" "$wall_milliseconds")"
  replay_row_tell_sampler "$tag" end "$rc $row_threads"
  if [[ $rc -ne 0 && "$REPLAY_BROKEN" != exit-code-ignored ]]; then   # 闸 3（弄坏开关 REPLAY_BREAK=exit-code-ignored：退出码非 0 照往下判）
    printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "退出码 $rc，见 $OUT_DIR/$tag.err" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"
    remember_row_outcome "$tag" "$reuse_record" "$fresh" "$identity"
    replay_row_report_end "$tag" "$wall_milliseconds" "$row_threads"; return
  fi
  # 产物头由这里写的行（ADMISSION_ROW_KEYS 第三列 replay：跑的东西不接准入）：准入放行时给的 E7INPUT 行接在新产物最前面，与装置自己打的同一个样子
  if [[ -n "${ADMISSION_REGISTERED[$row_key]:-}" && "$header_writer" == replay ]]; then
    if ! { cat -- "$OUT_DIR/$tag.header" "$fresh" >"$fresh.stamped" && mv -- "$fresh.stamped" "$fresh"; }; then
      printf '%-5s %-24s %-10s %s\n' "$exp" "$bin" 跑不了 "准入键 $row_key 的产物头接不到新产物前面（$OUT_DIR/$tag.header）" >"$OUT_DIR/$tag.line"; echo broken >"$OUT_DIR/$tag.verdict"
      remember_row_outcome "$tag" "$reuse_record" "$fresh" "$identity"
      replay_row_report_end "$tag" "$wall_milliseconds" "$row_threads"; return
    fi
  fi
  replay_judge_fresh_product "$exp" "$bin" "$stored" "$kind" "$row_key" "$tag" "$fresh"
  remember_row_outcome "$tag" "$reuse_record" "$fresh" "$identity"
  replay_row_report_end "$tag" "$wall_milliseconds" "$row_threads"
}

source "$REPLAY_LIBRARY_DIRECTORY/replay-dispatch.sh"
