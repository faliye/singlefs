#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上跑点名的档；哪些判定复用由背后两套驱动各自的全绿标记与判定库定，不在这里
# run-condition: command bash cargo python3
# run-condition: check source "$PREFLIGHT_SCRIPT_DIRECTORY/crash-amplification-lib.sh" && crash_amplification_run_condition_holds "$PREFLIGHT_SCRIPT_DIRECTORY/../.." :: 修好设备，或改配置：多机配置 multi-host.env 开了 GPU 而一份卡都没写、点名的卡与双机开关对不上；单跑 bash -c 'source scripts/crash/crash-amplification-lib.sh && crash_amplification_run_condition_holds' 看是哪一处
# 长活声明（判法 .claude/hooks/lib_long_run.py：scripts/hosts/detached-run.sh start --wall-clock over-2h 与 long-run-check 读这几行）：
# long-run-shards: 按合表 .claude/gate.d/crash-point-replay-items.tsv 的项逐项跑；每项的层 0 部分交门禁 checker-tier-layer0-replay --full（登记了 shard=across-machines 的用例按多机配置两台各跑一片），放量部分交 scripts/crash/crash-amplification.sh（双机开着时每一项的状态块按两台量到的速度分两份）
# long-run-resume: 重起同一条命令就是续跑：层 0 部分照全绿标记复用、进度文件接着跑（git common-dir 的 singlefs-layer0-progress/），放量部分照判定库（${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library-by-conclusion-kind）同一判法版本下判过的块复用
# long-run-unshardable: 多机：这一份不自己分台，台数与额度全由背后两套驱动照多机配置的双机开关取（至多本机与 PEER_SSH_HOST 两台，不读 layer0-shard-configuration-check.sh --emit-hosts 的清单、不调 host-resources.py 现算），快档那一段只在本机跑——经给定状态引擎的测试目标除外：配了双机时交双机驱动 crash-amplification-two-hosts.sh 两台各判一份（文件头「双机」）
# 崩溃点重放的入口（判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J2「一张表、一个入口、一道门禁」与 J9「两档」；
# 用户 2026-10-10 定的两档见 .claude/rules/verification.md「两档：轻型与重型」）。登记表是合表 .claude/gate.d/crash-point-replay-items.tsv（写法在它的表头，
# 一项一条流），读法与核法在 scripts/crash/lib_crash_point_replay_items.py。这一批背后照调今天的两套驱动：层 0 那一套（门禁 checker-tier-layer0-replay
# 的 --full：逐条照全绿标记复用）与放量分项驱动（scripts/crash/crash-amplification.sh）；引擎合并之后由引擎那一路换成一套。
#
#   crash-point-replay.sh                       重型·剪枝：全部项，层 0 部分照全绿标记复用、放量部分照判定库复用，加上快档那一段与两档都跑的那几类
#   crash-point-replay.sh --item <项> […]       重型·逐条：只跑点名的项（给几次取并集），快档那一段只跑这几项的测试目标，两档都跑的那几类不跑
#   crash-point-replay.sh --full [--item …]     重型·全量（清空复用）：同上，层 0 部分先删这几条用例这批输入的全绿标记、带 --start-over 丢掉进度文件，
#                                               放量部分换一份新的空判定库（留在 <放量的家目录>/crash-point-replay-full/<时戳>-<进程号>/library，跑完不删）
#   crash-point-replay.sh --quick [--item …]    轻型：核心部分，每次都跑全部项（见「两个域的写法」）；点名的项已在核心部分里，--item 只核项名
#   crash-point-replay.sh --list-items          逐行打「项名<TAB>轻型：…；重型：…」，只读合表，不编、不跑
#   crash-point-replay.sh --selftest            假 cargo、假的两套驱动、假的准入模块走一遍各格，末尾一轮从下面「弄坏开关」那张表现读开关、逐个打开
#   上面前四种跑法可再带 --cross-check-skipped-by-the-user：这一趟跳过 GPU 判与 CPU 判的对照（判决 J6、第三轮正文 H6）。只在用户当场授权时由主 agent 给；
#     给了才往引擎送 SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER=1、成功句写「对照：用户授权跳过」；没给就不送，调用方环境里带着这个变量也先清掉。
#   不带 --quick 的跑法是重型，只由带 SINGLEFS_HEAVY_TESTS 前缀的主 agent 跑（.claude/hooks/heavy-test-guard.sh 判）；轻型经
#   scripts/hosts/run-with-memory-cap.sh 起，谁都能跑。
#
# 两个域的写法（.claude/rules/verification.md「两档：轻型与重型」点名这里）：
#   「甲二」 = 每项第四列的测试目标不带 --ignored 跑到的那些用例（checker 档快档：按甲二展开的小域，crash::Layer0SegmentExpansion）；
#   「展开 6」 = 每项第三列的放量分项交 crash-amplification.sh --expand-up-to LIGHT_EXPAND_UP_TO（第一行代码之后那个赋值，今天 6），带一份这一趟新建、
#              跑完删掉的空判定库（不复用，轻型没有剪枝；库放 <放量的家目录>/crash-point-replay-quick/ 下，不放开机会清的目录）。
#   轻型与全量的那一份库，这里只建上层目录，库目录本身留给引擎新建（引擎见 <库>/CURRENT 不在就建空库，目录已经在就拒 LibraryDirectoryAlreadyExists）。
# 轻型 = 全部项在这两个域上判，连同两档都跑的那几类（BOTH_TIERS_GROUPS：小输入的 GPU 与 CPU 对拍、剪枝自证、钉回来的红例）。
# 重型 = 全部项的整域：快档那一段照轻型跑一遍，层 0 部分（第二列的 crash-case:）交门禁 checker-tier-layer0-replay --full --item，
#   放量部分（第三列）交 crash-amplification.sh 照它的全量展开深度（不带 --expand-up-to）；不带参数时另跑两档都跑的那几类。
# 轻型是重型的子集：轻型（全部项）的每一件在重型·剪枝（全部项）里都有——快档的测试目标照样在、展开 6 的放量分项在重型里按全量展开深度跑
#   （展开深度大的一趟罩住小的，一段不超过展开上限就展开全部真子集）；自证 light-within-heavy 一格拿真合表与小仓两份计划逐件核。
#   层 0 部分的全量与快档同一条流、同一个测试目标（合表第四列取自同一项的重型部分，lib_crash_point_replay_items.py 核）。
# 两档都跑的那几类（BOTH_TIERS_GROUPS，一行「类名<TAB>测试目标，空格隔开；没有写 -」）：没有测试目标的一类报本次未跑、写明为什么。
#
# GPU（.claude/singlefs-ai-sop/rules/test-script-discipline.md「用上现有的硬件加速」）：多机配置（crash-amplification-lib.sh 文件头「配置」：
#   ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，按 git common-dir 找）写了 ENABLE_GPU=1 时快档那一段带 --features verdict-store,gpu 编，
#   每一趟 cargo test 经 scripts/crash/gpu-quiesce-run.sh 清场再起（跑完不复原），环境里交清场之后那一份显卡先后
#   （多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED，没写取 GPU_CARDS_BY_PRIORITY）；走 GPU 的计算类不送（引擎开了 GPU 就三类全走 GPU、不读
#   GPU_COMPUTATION_CLASSES，crash-amplification-lib.sh 文件头「配置」）。
#   没开只带 --features verdict-store，挂 gpu 特性的对拍编进来 0 条、报本次未跑。放量部分的 GPU 与清场由 crash-amplification.sh 自己照同一份配置办。
#   层 0 部分（重型）里经引擎的 crash-case 行（.claude/gate.d/stage-inputs.tsv 里登记了 features= 的，今天是随机历史两项）由准入模块 admission.py 的 crash-case-command
#   带 --features 起、开 GPU 时同样加 gpu 并整条经清场包装起，不在这里拼（写法见 admission.py 文件头「崩溃枚举用例」）。
#   快档那几趟 cargo 清掉调用方环境里的 SINGLEFS_CRASH_AMPLIFICATION_*、GPU_COMPUTATION_CLASSES、SINGLEFS_LAYER0_FINDINGS_FILE、退役的 SINGLEFS_GPU_CARDS_CONFIG（同门禁
#   checker-tier-layer0-replay 的 feature-gated-tests 格：快档不写真的判定库、不按全量的规模跑），多机配置在就交 SINGLEFS_MULTI_HOST_CONFIG=<它的绝对路径>。
#
# 双机（E4 报告第三节第 2 条：给定状态引擎读 SINGLEFS_CRASH_AMPLIFICATION_SHARE、…_HOST_WEIGHTS、…_LIBRARY、…_IMPORT，与放量流水线同一种读法）：
#   「经给定状态引擎的项」= 合表第二列点名的 crash-case 键在 .claude/gate.d/stage-inputs.tsv 里登记了 features= 的项（要经引擎、进判定库的才登记；今天是
#   crash-injection-fast-tier 与 read-faulted-states-never-hold-the-witness-criterion），它们第四列的测试目标在快档里这样跑：
#   多机配置写了 ENABLE_ACROSS_MACHINES=1 时先经 scripts/crash/layer0-shard-configuration-check.sh 判（双机开没开、键齐不齐、第二台连不连得上）：
#     退 0（双机开）：每个这样的测试目标一趟 scripts/crash/crash-amplification-two-hosts.sh <测试目标> --library <新建的空库> --label <标签>，照它的三步跑——
#       本机 SHARE=0/2、第二台 SHARE=1/2 经 multi-host-run.sh 跑完取回库、本机带 SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<取回的库> 再跑一趟；
#       库放 <放量的家目录>/crash-point-replay-fast-tier/<时戳>-<进程号>/<测试目标>/library（不放开机会清的目录，引擎拒那里的库），跑完连同取回的那一份删掉、
#       双机驱动的日志留在 <放量的家目录>/runs/<标签>/。开了 GPU 整条经 gpu-quiesce-run.sh 清场再起、带 --quiesced。
#       判法：双机驱动退 0 照导入那一趟（import.log）的 libtest 结果判（同上「一项怎么判」）；退 78（第二台连不上、库没取回、卡用不上、给定状态表借不到显存）
#       与内存包装的结局 245、251、252、254 记 not-run、退 78；退 77 记 not-run；别的照导入那一趟的结果记红。
#     退 3（双机关）：照旧本机一趟；别的退出码（判不过，含第二台连不上）：这几个测试目标一趟都不起，记 not-run、退 78、原因写判法那一句。
#   没写 ENABLE_ACROSS_MACHINES=1 的（配置不在、没写、写 0）不调判法，照旧本机一趟。别的测试目标一律本机一趟。
# 内存包装：套在 scripts/hosts/run-with-memory-cap.sh 里跑时（轻型照规则这样起），包装给命令设了 RUN_WITH_MEMORY_CAP_OUTER_CAP=<外层的上限>
#   （run-with-memory-cap.sh 文件头「套在包装里」）；这一趟把它照原样给放量入口与双机驱动的 --memory-cap：里层上限比外层大，包装按用法错退 251。没设不给（各用各的默认）。
#
# 往引擎送的两个变量（快档、层 0 部分与放量部分都从这一趟的环境继承；放量部分双机时第二台那几趟只看双机驱动送过去的环境，这两个不在里面）：
#   SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=<仓根>/.claude/gate.d/crash-point-replay-segments.tsv（按段登记文件的绝对路径，判决 J10、M4 第 15 条）：
#     每一趟都送；开跑前先经 scripts/crash/lib_crash_point_replay_items.py segments 核它（核不过退 2、什么都不起；写法与核法在那份库文件头「按段登记」）。
#   SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER=1：只在给了 --cross-check-skipped-by-the-user 的那一趟送（见「用法」），没给先清掉。
#
# 输出：背后每一趟的输出边跑边转打，同时进这一趟的日志目录 <放量的家目录>/runs/crash-point-replay-<时戳>-<进程号>/（跑完不删）。每项（与两档都跑的每一类）
#   打一行 CRASH_POINT_REPLAY_ITEM item=<项> tier=<quick|pruned|named|full> exit=<0|1|77|78> verdict=<green|red|not-run>
#     fast_tier_tests=<快档跑到的用例数> fast_tier_red=<红的用例数> fast_tier_reused=none（快档没有判定复用这一层）
#     pipeline_states=<放量现判的态数> pipeline_red=<红> pipeline_reused=<复用的态数，放量入口取不出写 unknown>
#     enumeration_cases=<层 0 部分的用例数> enumeration_red=<红的用例数> enumeration_reused=<照全绿标记复用的用例数> seconds=<这一项各段的挂钟之和>
#     （这一项没有那一段的写 -）；末行 CRASH_POINT_REPLAY_TOTAL tier=… dispatched=<派出几项> collected=<收回几项> green= red= not_run=
#     fast_tier_tests= pipeline_states= pipeline_reused_known= pipeline_reuse_unknown=<几项取不出> enumeration_cases= enumeration_reused= seconds=。
#   一项怎么判：有一段红就 red；不红而有一段本次未跑（快档跑到 0 条、有 NOT_RUN_NO_GPU_ADAPTER、放量那一项 verdict=not-run、层 0 那一道退 77 或 3，
#   某一段退 78）就 not-run，逐项写原因；别的 green。快档一个测试目标：cargo 退非 0 或有 FAILED 就红（编不过的整段红）；
#   日志里有行首（可带空白）是 CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 的那一行（给定状态表的判器起过卡池而借不到显存，引擎打这一行后报错、cargo 退 101；
#   行首取 crash-amplification-lib.sh 的 CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX）的，不记红：那一段记 not-run、退 78、原因写那一行，整趟没红就退 78。
#   放量部分点名的分项没打 CRASH_AMPLIFICATION_ITEM 行就红（收不回）；派出几项、收回几项对不上整趟红（command-safety.md「并行不许把失败吃掉」）。
#   有红时出路给出只重跑红的那几项的命令（重型照档带 --item，项数就是红的项数；两档都跑的那几类不能点名；轻型每次跑全部项，给整趟重跑那一条）。
#   退 0 时打一句成功句：「崩溃点重放<档>：K 项都不红；本次未跑 J 项：<项（原因）…>；快档 N 条用例、放量现判 N 态复用 M 态、层 0 用例 N 条复用 M 条；派出 D 项、收回 D 项」，
#   给了 --cross-check-skipped-by-the-user 的那一趟句末另加「；对照：用户授权跳过」。
# 退出码：
#   0   跑到的项都不红、派出收回相等（有项本次未跑也是 0，成功句逐项点名）
#   1   有红、或派出收回对不上
#   2   用法错：参数认不得、--item 后面缺项名或项名认不得（列出可点名的项）、--quick 与 --full 一起给、--list-items 或 --selftest 夹了别的参数、
#       合表核不过、按段登记文件核不过、平常跑时带了只供自证的变量（见「只供自证」）
#   77  派出的项全是本次未跑，也没有红
#   78  运行条件不满足：preflight 判的（文件头两条 run-condition），或背后有一段退 78（显卡、第二台、清场用不上，给定状态表的判器借不到显存，
#       多机配置开了双机而判法判不过）而这一趟没有红
# 弄坏开关 CRASH_POINT_REPLAY_BREAK=<项>（几个用逗号连；只在自证起的子进程里认）。写法同 scripts/crash/crash-amplification.sh 文件头「弄坏开关」一段：
#   一行「#   CRASH_POINT_REPLAY_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（、隔开；括号里是失败行的句首，开关轮不读）。
#   --selftest 末尾那一轮（开关轮，crash-amplification-lib.sh 的 crash_amplification_switch_wheel）从这张表现读，表里的开关与代码里 replay_break_is 认的要相等：
#   CRASH_POINT_REPLAY_BREAK=accept-unknown-item  认不得的项名悄悄丢掉、照跑
#       unknown-item（「项名写错应当退 2、列出可点名的项、什么都不起」）
#   CRASH_POINT_REPLAY_BREAK=list-items-builds  --list-items 先起一趟 cargo build 再列
#       list-items（「--list-items 应当退 0、逐项列出、不起 cargo 与驱动」）
#   CRASH_POINT_REPLAY_BREAK=quick-reuses-library  轻型的放量部分不带新建的空判定库（照默认库复用，等于剪枝）
#       quick-runs-the-light-domains（「轻型应当照甲二与展开 6 跑全部项」）、quick-named-item（「--quick --item 应当照样跑核心部分」）、library-left-to-the-engine（轻型那一支没交库）
#   CRASH_POINT_REPLAY_BREAK=quick-without-both-tiers  轻型不跑两档都跑的那几类
#       quick-runs-the-light-domains（「轻型应当照甲二与展开 6 跑全部项」）、quick-gpu（「开了 GPU 快档应当带 gpu 特性、经清场包装起」：少了那几个测试目标）、
#       fast-tier-not-run（「快档跑到 0 条或没卡没跑的，应当记本次未跑」：钉表那一类没了，本次未跑少一项）
#   CRASH_POINT_REPLAY_BREAK=heavy-without-fast-tier  重型的计划里没有快档那一段
#       light-within-heavy（「轻型的每一件在重型里都应当有」）、pruned-heavy（「不带参数的重型应当照剪枝跑全部项」）、named-heavy（「--item 应当只跑点名的项」）
#   CRASH_POINT_REPLAY_BREAK=gpu-without-quiesce  开了 GPU 时快档那几趟不经清场包装起
#       quick-gpu（「开了 GPU 快档应当带 gpu 特性、经清场包装起」）
#   CRASH_POINT_REPLAY_BREAK=gpu-cards-not-sent  开了 GPU 时快档那几趟不交显卡先后
#       quick-gpu（「开了 GPU 快档应当带 gpu 特性、经清场包装起」）
#   CRASH_POINT_REPLAY_BREAK=red-as-green  快档 cargo 退非 0、有 FAILED 也记绿
#       fast-tier-red（「快档一个测试目标红了，那一项应当红、整趟退 1」）
#   CRASH_POINT_REPLAY_BREAK=not-run-as-green  快档跑到 0 条、有 NOT_RUN_NO_GPU_ADAPTER 也记绿
#       fast-tier-not-run（「快档跑到 0 条或没卡没跑的，应当记本次未跑」）、all-not-run（「派出的项全是本次未跑，应当退 77」）
#   CRASH_POINT_REPLAY_BREAK=skip-collection-count  放量部分收不回的分项不记红、派出收回不核
#       collection-count（「放量那一项收不回，应当红、派出收回对不上」）
#   CRASH_POINT_REPLAY_BREAK=full-keeps-reuse  全量不删全绿标记、不带 --start-over、不换新的空判定库
#       full-heavy（「全量应当清空复用」）、library-left-to-the-engine（--full 那一支没交库）
#   CRASH_POINT_REPLAY_BREAK=enumeration-exit-ignored  层 0 那一道的退出码不看，一律记绿
#       enumeration-red（「层 0 那一道红了，那一项应当红」）
#   CRASH_POINT_REPLAY_BREAK=pipeline-red-ignored  放量那一项的 red= 与 verdict=red 不看
#       pipeline-red（「放量那一项红了，那一项应当红」）
#   CRASH_POINT_REPLAY_BREAK=honor-test-variables-outside-selftest  平常跑时也认只供自证的变量，不拒
#       test-variables-outside-selftest（「平常跑时带了只供自证的变量应当退 2」）
#   CRASH_POINT_REPLAY_BREAK=fast-tier-classes-sent  开了 GPU 时快档那几趟照改前送三类 GPU_COMPUTATION_CLASSES
#       quick-gpu（「开了 GPU 快档应当带 gpu 特性、经清场包装起」：调用记录里 classes= 不是 [unset]）
#   CRASH_POINT_REPLAY_BREAK=cross-check-skip-not-sent  给了 --cross-check-skipped-by-the-user 也不送对照跳过的开关
#       cross-check-skip-sent（「给了 --cross-check-skipped-by-the-user，快档与放量每一趟都应当带」）
#   CRASH_POINT_REPLAY_BREAK=cross-check-skip-inherited  没给 --cross-check-skipped-by-the-user 时不清调用方环境里的对照跳过开关
#       cross-check-skip-cleared（「没给 --cross-check-skipped-by-the-user，调用方环境里带着那个变量也应当清掉」）
#   CRASH_POINT_REPLAY_BREAK=segments-registry-not-sent  不往引擎送按段登记文件的路径
#       segments-registry-sent（「快档与放量每一趟都应当带按段登记文件的绝对路径」）
#   CRASH_POINT_REPLAY_BREAK=segments-registry-unchecked  开跑前不核按段登记文件
#       segments-registry-refused（「按段登记文件核不过应当退 2、什么都不起」）
#   CRASH_POINT_REPLAY_BREAK=given-states-run-condition-as-red  快档日志里有给定状态表借不到显存那一行也照红记（不退 78）
#       fast-tier-run-condition-not-met（「快档日志里有给定状态表借不到显存那一行……整趟退 78」）
#   CRASH_POINT_REPLAY_BREAK=two-hosts-not-used  多机配置开了双机，经给定状态引擎的测试目标也照本机一趟跑（改前的样子）
#       fast-tier-two-hosts（「开了双机，经给定状态引擎的测试目标应当照双机驱动的三步跑」）、
#       fast-tier-two-hosts-peer-unreachable（「第二台连不上…应当记本次未跑、整趟退 78」）、fast-tier-two-hosts-configuration-refused（「开了双机而判法判不过…」）
#   CRASH_POINT_REPLAY_BREAK=two-hosts-when-off  双机关着也把经给定状态引擎的测试目标交双机驱动
#       fast-tier-two-hosts-off（「双机关着，经给定状态引擎的测试目标应当照旧本机一趟」）、
#       fast-tier-two-hosts-configuration-refused（不调判法就交了双机驱动，判法判不过也照跑）
#   CRASH_POINT_REPLAY_BREAK=two-hosts-unreachable-as-red  双机驱动退 78（第二台连不上）照红记
#       fast-tier-two-hosts-peer-unreachable（「第二台连不上…应当记本次未跑、整趟退 78」）
#   CRASH_POINT_REPLAY_BREAK=two-hosts-check-failure-as-single  多机配置开了双机而判法判不过，照双机关、本机一趟跑
#       fast-tier-two-hosts-configuration-refused（「开了双机而判法判不过…」）
#   CRASH_POINT_REPLAY_BREAK=memory-cap-not-forwarded  套在内存包装里也不把外层的上限给放量入口与双机驱动（改前的样子）
#       outer-memory-cap-forwarded（「套在内存包装里，放量入口应当带外层的上限」）、fast-tier-two-hosts（双机驱动那一趟没带 --memory-cap 3G）
#   CRASH_POINT_REPLAY_BREAK=library-directory-precreated  轻型与 --full 两支先把判定库目录本身建出来，再交给放量入口（改前的样子）
#       library-left-to-the-engine（「交给放量入口的判定库目录应当还不在……」）
# 只供自证（自证起的子进程里才认；平常跑时有一个非空就退 2 拒跑，--list-items 与 --selftest 本身不拒）：CRASH_POINT_REPLAY_BREAK=<项>（上面那张表）；
#   CRASH_POINT_REPLAY_ROOT=<造出来的小仓>：合表、stage-inputs.tsv、门禁 checker-tier-layer0-replay、crash-amplification.sh 与准入模块 admission.py 都从它取
#   （那三份是自证写的假的：只记一行参数、照环境变量打结果）。
#   「自证起的子进程」的标记：CRASH_POINT_REPLAY_SELFTEST_MARK=<文件>，那份文件在、内容是 SELFTEST_MARK_CONTENT 那一行；只有 run_selftest 设它。
#   这三个变量读完就从环境里清掉，不传给背后的驱动。自证里的 cargo 是 PATH 最前面的假 cargo（起之前核 PATH 上解到的就是它，不是就不起）；
#   清场走真的 scripts/crash/gpu-quiesce-run.sh，多机配置指到临时目录里只写了假清场命令的一份，状态与日志目录都在临时目录里。
# 自证末尾的开关轮并行度 CRASH_AMPLIFICATION_SELFTEST_JOBS，没给取 SINGLEFS_THREAD_CAP，再没给按 nproc（同 crash-amplification-lib.sh 文件头「弄坏开关轮」）。
#
# gate-similar: crash-amplification.sh 它是放量分项的登记表与全量循环；这里是合表的入口，放量部分每趟调它一次，层 0 部分调门禁 checker-tier-layer0-replay
# gate-similar: checker-tier-layer0-replay.sh 它跑层 0 快档与登记的崩溃枚举用例；这里的重型照项调它的 --full --item，快档那一段只跑合表点名的测试目标
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

LIGHT_EXPAND_UP_TO=6
SELFTEST_MARK_CONTENT="crash-point-replay.sh --selftest 起的子进程"
BOTH_TIERS_GROUPS=(
  $'gpu-and-cpu-agree-on-small-inputs\tcrash_judge_gpu_agrees_with_the_cpu_judge_on_every_state crash_facts_gpu_verifier_agrees_with_the_cpu_reference gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core'
  $'pruning-self-proof\tcrash_amplification_two_flows_share_their_prefix crash_amplification_pipeline_records_checks_and_reuses'
  $'pinned-red-examples\t-'
)
PINNED_RED_EXAMPLES_NOT_RUN_REASON="崩溃点重放还没有钉表：全量出的红要钉回快档（test-script-discipline.md「全量的红，钉回快档」），今天一条都没钉"

replay_script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
replay_script_directory="$(dirname "$replay_script")"
source "$replay_script_directory/crash-amplification-lib.sh"
repository_root="$(cd "$replay_script_directory/../.." && pwd)"

# 只供自证的三个变量（文件头「只供自证」）：读完就清掉
replay_in_selftest_child=0
if [[ -n "${CRASH_POINT_REPLAY_SELFTEST_MARK:-}" && -f "${CRASH_POINT_REPLAY_SELFTEST_MARK:-}" ]] \
   && [[ "$(head -n 1 -- "$CRASH_POINT_REPLAY_SELFTEST_MARK")" == "$SELFTEST_MARK_CONTENT" ]]; then
  replay_in_selftest_child=1
fi
replay_break="${CRASH_POINT_REPLAY_BREAK:-}"
replay_root_override="${CRASH_POINT_REPLAY_ROOT:-}"
replay_test_variables=()
[[ -n "$replay_break" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_BREAK)
[[ -n "$replay_root_override" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_ROOT)
unset CRASH_POINT_REPLAY_BREAK CRASH_POINT_REPLAY_ROOT CRASH_POINT_REPLAY_SELFTEST_MARK
replay_break_is() { [[ ",$replay_break," == *",$1,"* ]]; }

usage() {
  echo "  ✗ 用法不对：$1。用法：$0 [--quick | --full] [--item <项>]… [--cross-check-skipped-by-the-user] | --list-items | --selftest" >&2
  echo "  → 怎么办：不带参数是重型·剪枝；--quick 是轻型；项名照 --list-items 列的写；--quick 与 --full 不一起给" >&2
  exit 2
}

tier=""; quick=0; full=0; list_items=0; selftest=0; named_items=(); cross_check_skipped=0
while (($#)); do
  case "$1" in
    --quick) quick=1; shift ;;
    --cross-check-skipped-by-the-user) cross_check_skipped=1; shift ;;
    --full) full=1; shift ;;
    --list-items) list_items=1; shift ;;
    --selftest) selftest=1; shift ;;
    --item)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then usage "--item 后面缺项名"; fi
      named_items+=("$2"); shift 2 ;;
    --item=*) [[ -n "${1#--item=}" ]] || usage "--item= 后面缺项名"; named_items+=("${1#--item=}"); shift ;;
    *) usage "认不得的参数 $1" ;;
  esac
done

# 自证起的子进程才认 CRASH_POINT_REPLAY_ROOT；平常跑时带了只供自证的变量就拒（--list-items 与 --selftest 本身不拒）
if (( ! selftest )); then
  if (( replay_in_selftest_child )); then
    [[ -n "$replay_root_override" ]] && repository_root="$replay_root_override"
  elif (( ${#replay_test_variables[@]} )) && (( ! list_items )) && ! replay_break_is honor-test-variables-outside-selftest; then
    echo "  ✗ 平常跑时带了只供自证的变量：${replay_test_variables[*]}（文件头「只供自证」）" >&2
    echo "  → 怎么办：去掉它们再跑（env -u <变量> …）；要证红就跑 $0 --selftest" >&2
    exit 2
  elif (( ! list_items )) && [[ -n "$replay_root_override" ]]; then
    repository_root="$replay_root_override"
  fi
fi

# ── 合表（lib_crash_point_replay_items.py list：核不过不打表、退 1）
declare -A item_cases=() item_pipeline_items=() item_light_targets=() pipeline_owner=()
item_names=()
load_items() {
  local listing listing_exit name cases pipeline_text light_targets word
  listing="$(python3 "$replay_script_directory/lib_crash_point_replay_items.py" list "$repository_root")"; listing_exit=$?
  if (( listing_exit != 0 )); then
    printf '%s\n' "$listing" >&2
    echo "  ✗ 崩溃点重放合表核不过（lib_crash_point_replay_items.py list 退 $listing_exit），什么都不跑" >&2
    echo "  → 怎么办：照上面逐处列的改 .claude/gate.d/crash-point-replay-items.tsv（写法在它的表头），再跑" >&2
    exit 2
  fi
  while IFS=$'\t' read -r name cases pipeline_text light_targets; do
    [[ -n "$name" ]] || continue
    item_names+=("$name")
    [[ "$cases" == - ]] && cases=""
    [[ "$pipeline_text" == - ]] && pipeline_text=""
    item_cases[$name]="$cases"; item_pipeline_items[$name]="$pipeline_text"; item_light_targets[$name]="$light_targets"
    for word in $pipeline_text; do pipeline_owner[$word]="$name"; done
  done <<<"$listing"
}

# 经给定状态引擎的项（文件头「双机」）：第二列点名的 crash-case 键在 stage-inputs.tsv 里登记了 features= 的，它们第四列的测试目标记进 given_states_targets
declare -A given_states_targets=()
collect_given_states_targets() {
  local name key word registration="$repository_root/.claude/gate.d/stage-inputs.tsv"
  for name in "${item_names[@]}"; do
    for key in ${item_cases[$name]}; do
      if awk -F'\t' -v key="$key" '$1 == key && $3 ~ /(^| )features=/ { found = 1 } END { exit !found }' "$registration" 2>/dev/null; then
        for word in ${item_light_targets[$name]}; do given_states_targets[$word]=1; done
        break
      fi
    done
  done
}

# 快档里经给定状态引擎的测试目标走不走两台（文件头「双机」）：设 fast_tier_two_hosts（0/1）与 fast_tier_two_hosts_refusal（开了双机而判法判不过时的原因，不空就不起）
fast_tier_two_hosts=0; fast_tier_two_hosts_refusal=""
decide_fast_tier_two_hosts() {
  local note check_exit
  fast_tier_two_hosts=0; fast_tier_two_hosts_refusal=""
  replay_break_is two-hosts-not-used && return 0
  if replay_break_is two-hosts-when-off; then fast_tier_two_hosts=1; return 0; fi
  [[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_ACROSS_MACHINES)" == 1 ]] || return 0
  if note="$(SINGLEFS_MULTI_HOST_CONFIG="$configuration_file" bash "$replay_script_directory/layer0-shard-configuration-check.sh" "$repository_root" 2>&1)"; then check_exit=0; else check_exit=$?; fi
  case "$check_exit" in
    0) fast_tier_two_hosts=1; echo "  · 双机：开（${note//$'\n'/；}）：快档里经给定状态引擎的测试目标交 crash-amplification-two-hosts.sh 两台各判一份" ;;
    3) echo "  · 双机：关（${note//$'\n'/；}）：快档全在本机跑" ;;
    *)
      if replay_break_is two-hosts-check-failure-as-single; then return 0; fi
      fast_tier_two_hosts_refusal="运行条件不满足：多机配置开了双机，判法 layer0-shard-configuration-check.sh 判不过（退 $check_exit：${note//$'\n'/；}），这个测试目标一趟都没起；修好第二台或多机配置再跑，不想用第二台就把 ENABLE_ACROSS_MACHINES 改成 0" ;;
  esac
}

# 快档一趟的 libtest 日志怎么判（文件头「一项怎么判」）：设 fast_state、fast_reason、fast_exit（记进记录的退出码）、fast_tests、fast_red
judge_fast_tier_log() { # <退出码> <日志>
  local run_exit="$1" log="$2" counts first_line passed failed not_run_lines condition_lines condition_line
  counts="$(python3 - "$log" "$replay_script_directory/../gate/lib_libtest_result_lines.py" "$CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX" <<'PYTHON'
import importlib.util, sys
specification = importlib.util.spec_from_file_location("lib_libtest_result_lines", sys.argv[2])
module = importlib.util.module_from_spec(specification)
specification.loader.exec_module(module)
try:
    text = open(sys.argv[1], encoding="utf-8", errors="replace").read()
except OSError:
    text = ""
passed = failed = 0
for summary in module.summary_lines(text):
    passed += summary.passed
    failed += summary.failed
failed = max(failed, len(module.failed_test_names(text)))
not_run = sum(1 for line in text.splitlines() if line.lstrip().startswith("NOT_RUN_NO_GPU_ADAPTER"))
prefix = sys.argv[3]
condition = [line.strip() for line in text.splitlines() if line.lstrip() == prefix or line.lstrip().startswith(prefix + " ")]
print(passed, failed, not_run, len(condition))
print(condition[0] if condition else "")
PYTHON
)"
  first_line="${counts%%$'\n'*}"
  condition_line=""
  [[ "$counts" == *$'\n'* ]] && condition_line="${counts#*$'\n'}"
  read -r passed failed not_run_lines condition_lines <<<"${first_line:-0 0 0 0}"
  [[ "$passed" =~ ^[0-9]+$ ]] || passed=0
  [[ "$failed" =~ ^[0-9]+$ ]] || failed=0
  [[ "$not_run_lines" =~ ^[0-9]+$ ]] || not_run_lines=0
  [[ "$condition_lines" =~ ^[0-9]+$ ]] || condition_lines=0
  fast_state=green; fast_reason=""; fast_exit="$run_exit"; fast_red=0
  if (( condition_lines > 0 )) && ! replay_break_is given-states-run-condition-as-red; then
    fast_state=not-run; fast_exit=78
    fast_reason="运行条件不满足：给定状态表的判器借不到显存（$condition_line；共 $condition_lines 处，日志 $log），这一趟不算数；等卡上占着显存的跑完再重跑"
  elif [[ "$run_exit" != 0 || "$failed" -gt 0 ]] && ! replay_break_is red-as-green; then
    fast_state=red; fast_reason="cargo test 退 $run_exit、红 $failed 条（日志 $log）"
    (( failed > 0 )) || failed=1
    fast_red=$failed
  elif (( passed + failed == 0 )) && ! replay_break_is not-run-as-green; then
    fast_state=not-run; fast_reason="快档跑到 0 条（这个测试目标只有标了 ignore 的全量，或挂的特性这一趟没编进来）"
  elif (( not_run_lines > 0 )) && ! replay_break_is not-run-as-green; then
    fast_state=not-run; fast_reason="有 $not_run_lines 处 NOT_RUN_NO_GPU_ADAPTER（这台机器找不到 GPU 适配器）"
  fi
  fast_tests=$((passed + failed))
}

# 经给定状态引擎的测试目标在双机开着时的一趟（文件头「双机」）：交 crash-amplification-two-hosts.sh 照三步跑，判完记给每一项。
# 调用方 run_fast_tier 的 quiesce_prefix 照用（开了 GPU 整条经清场包装起）
run_fast_tier_target_on_two_hosts() { # <测试目标> <记给哪几项>
  local target="$1" owners="$2" owner started label library_root two_hosts_runs two_hosts_log two_hosts_exit variable
  local two_hosts_environment=() two_hosts_arguments=()
  started=$SECONDS
  if [[ -n "$fast_tier_two_hosts_refusal" ]]; then
    for owner in $owners; do record_part "$owner" fast-tier "$target" 78 not-run 0 0 none 0 "$fast_tier_two_hosts_refusal"; done
    return 0
  fi
  label="crash-point-replay-$run_stamp-$target"
  library_root="$pipeline_home/crash-point-replay-fast-tier/$run_stamp"
  two_hosts_runs="$pipeline_home/runs/$label"
  if ! mkdir -p -- "$library_root/$target"; then
    for owner in $owners; do record_part "$owner" fast-tier "$target" 78 not-run 0 0 none 0 "运行条件不满足：建不了这一趟的空判定库的目录 $library_root/$target（看 $pipeline_home 可不可写、盘满没满）"; done
    return 0
  fi
  two_hosts_environment=(env -u SINGLEFS_LAYER0_FINDINGS_FILE -u SINGLEFS_GPU_CARDS_CONFIG -u GPU_COMPUTATION_CLASSES -u GPU_CARDS_BY_PRIORITY)
  for variable in $(compgen -e); do [[ "$variable" == SINGLEFS_CRASH_AMPLIFICATION_* ]] && two_hosts_environment+=(-u "$variable"); done
  two_hosts_environment+=("SINGLEFS_CRASH_AMPLIFICATION_HOME=$pipeline_home")
  [[ -f "$configuration_file" ]] && two_hosts_environment+=("SINGLEFS_MULTI_HOST_CONFIG=$(cd "$(dirname "$configuration_file")" && pwd)/$(basename "$configuration_file")")
  two_hosts_arguments=("$target" --library "$library_root/$target/library" --label "$label")
  [[ -n "$outer_memory_cap" ]] && two_hosts_arguments+=(--memory-cap "$outer_memory_cap")
  (( gpu_switch )) && two_hosts_arguments+=(--quiesced)
  two_hosts_log="$run_directory/fast-tier.$target.two-hosts.log"
  echo "── 快档（两台分片）：bash scripts/crash/crash-amplification-two-hosts.sh ${two_hosts_arguments[*]}（本机 0/2、第二台 1/2 取回库、本机带 IMPORT 再跑${quiesce_prefix[*]:+；经 scripts/crash/gpu-quiesce-run.sh 清场再起}）"
  run_logged "$two_hosts_log" ${quiesce_prefix[@]+"${quiesce_prefix[@]}"} "${two_hosts_environment[@]}" bash "$replay_script_directory/crash-amplification-two-hosts.sh" "${two_hosts_arguments[@]}"
  two_hosts_exit="$(cat "$two_hosts_log.exit" 2>/dev/null || echo 255)"
  # 这一趟的库不复用（快档没有判定复用这一层）：连同取回的那一份、送第二台的快照一起删，日志留着
  rm -rf -- "${library_root:?}/${target:?}" "${two_hosts_runs:?}/fetched" "${two_hosts_runs:?}/library-for-peer"
  rmdir -- "$library_root" "$(dirname "$library_root")" 2>/dev/null
  case "$two_hosts_exit" in
    0) judge_fast_tier_log 0 "$two_hosts_runs/import.log" ;;
    78)
      fast_tests=0; fast_red=0
      if replay_break_is two-hosts-unreachable-as-red; then
        fast_state=red; fast_exit=1; fast_red=1; fast_reason="双机驱动退 78（日志 $two_hosts_log、$two_hosts_runs/）"
      else
        fast_state=not-run; fast_exit=78
        fast_reason="运行条件不满足：双机驱动退 78（第二台连不上、第二台的库没取回、卡用不上或给定状态表的判器借不到显存，出路见它打的那一句；日志 $two_hosts_log、$two_hosts_runs/）"
      fi ;;
    245|251|252|254)
      fast_state=not-run; fast_exit=78; fast_tests=0; fast_red=0
      fast_reason="运行条件不满足：双机驱动退 $two_hosts_exit（内存包装自己的结局或显存不够，这一次的输出不算结果：scripts/hosts/run-with-memory-cap.sh 文件头「退出码」；日志 $two_hosts_log、$two_hosts_runs/）" ;;
    77)
      fast_state=not-run; fast_exit=77; fast_tests=0; fast_red=0
      fast_reason="双机驱动退 77（本次未跑；日志 $two_hosts_log、$two_hosts_runs/）" ;;
    *)
      judge_fast_tier_log "$two_hosts_exit" "$two_hosts_runs/import.log"
      fast_reason="双机驱动退 $two_hosts_exit；${fast_reason:-导入那一趟的结果判不出红}（日志 $two_hosts_log、$two_hosts_runs/）" ;;
  esac
  for owner in $owners; do record_part "$owner" fast-tier "$target" "$fast_exit" "$fast_state" "$fast_tests" "$fast_red" none "$((SECONDS - started))" "$fast_reason"; done
}

# 一档的计划（纯函数）：打这一档要做的每一件「项<TAB>种类<TAB>对象」。种类：fast-tier（快档的测试目标）、pipeline-expand-<N>（放量分项按展开 N）、
# pipeline-full（放量分项按全量展开深度）、enumeration-full（crash-case: 键交 checker-tier-layer0-replay --full）、not-run（两档都跑的一类没有测试目标，对象是原因）。
# 两档都跑的那几类记成项 both-tiers:<类名>。<档> 是 quick、pruned、named、full。
replay_plan() { # <档> <两档都跑的那几类跑不跑 0/1> <项…>
  local plan_tier="$1" with_groups="$2" item word group_line group_name group_targets
  shift 2
  for item in "$@"; do
    if [[ "$plan_tier" == quick ]] || ! replay_break_is heavy-without-fast-tier; then
      for word in ${item_light_targets[$item]}; do printf '%s\tfast-tier\t%s\n' "$item" "$word"; done
    fi
    if [[ "$plan_tier" == quick ]]; then
      for word in ${item_pipeline_items[$item]}; do printf '%s\tpipeline-expand-%s\t%s\n' "$item" "$LIGHT_EXPAND_UP_TO" "$word"; done
    else
      for word in ${item_cases[$item]}; do printf '%s\tenumeration-full\t%s\n' "$item" "$word"; done
      for word in ${item_pipeline_items[$item]}; do printf '%s\tpipeline-full\t%s\n' "$item" "$word"; done
    fi
  done
  if [[ "$with_groups" == 1 ]] && ! { [[ "$plan_tier" == quick ]] && replay_break_is quick-without-both-tiers; }; then
    for group_line in "${BOTH_TIERS_GROUPS[@]}"; do
      IFS=$'\t' read -r group_name group_targets <<<"$group_line"
      if [[ "$group_targets" == - ]]; then
        printf 'both-tiers:%s\tnot-run\t%s\n' "$group_name" "$PINNED_RED_EXAMPLES_NOT_RUN_REASON"
        continue
      fi
      if [[ "$plan_tier" == quick ]] || ! replay_break_is heavy-without-fast-tier; then
        for word in $group_targets; do printf 'both-tiers:%s\tfast-tier\t%s\n' "$group_name" "$word"; done
      fi
    done
  fi
}

# 轻型（全部项）的每一件在重型·剪枝（全部项）里都有（文件头「轻型是重型的子集」）：打缺的那几件；全有时只打一行 within
replay_light_within_heavy() {
  local light heavy missing
  light="$(replay_plan quick 1 "${item_names[@]}")"
  heavy="$(replay_plan pruned 1 "${item_names[@]}")"
  missing="$(while IFS=$'\t' read -r plan_item plan_kind plan_object; do
      [[ -n "$plan_item" ]] || continue
      case "$plan_kind" in
        pipeline-expand-*) wanted_kind=pipeline-full ;;
        *) wanted_kind="$plan_kind" ;;
      esac
      grep -qxF -- "$plan_item"$'\t'"$wanted_kind"$'\t'"$plan_object" <<<"$heavy" || printf '%s %s %s；' "$plan_item" "$plan_kind" "$plan_object"
    done <<<"$light")"
  if [[ -z "$missing" && -n "$light" ]]; then echo within; else printf '%s\n' "${missing:-轻型的计划是空的}"; fi
}

# ── 背后的几段。每段往记录文件 $part_records 追加一行：
#   项<TAB>段（fast-tier|pipeline|enumeration|not-run）<TAB>对象<TAB>退出码<TAB>结论（green|red|not-run）<TAB>判了多少<TAB>红了多少<TAB>复用（数、unknown 或 none）<TAB>秒<TAB>原因
record_part() { printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "$@" >>"$part_records"; }

# 跑一条命令：输出边跑边转打、同时进日志；退出码写进 <日志>.exit
run_logged() { # <日志> <命令…>
  local log="$1"
  shift
  { "$@" 2>&1; echo "$?" >"$log.exit"; } | tee "$log"
}

run_fast_tier() { # <计划文件>：快档那一段（fast-tier 的件），一个测试目标一趟；同一个测试目标几项都点名时跑一次、记给每一项
  local plan_file="$1" targets=() target target_arguments=() environment=() quiesce_prefix=() build_log build_exit run_log run_exit
  local variable started owners owner any_given_states=0
  local fast_state fast_reason fast_exit fast_tests fast_red
  while IFS=$'\t' read -r _plan_item plan_kind plan_object; do
    [[ "$plan_kind" == fast-tier ]] || continue
    [[ " ${targets[*]-} " == *" $plan_object "* ]] || targets+=("$plan_object")
  done <"$plan_file"
  (( ${#targets[@]} )) || return 0
  for target in "${targets[@]}"; do target_arguments+=(--test "$target"); [[ -n "${given_states_targets[$target]+set}" ]] && any_given_states=1; done
  # 经给定状态引擎的测试目标走不走两台（文件头「双机」）：只在计划里有这样的测试目标时判
  (( any_given_states )) && decide_fast_tier_two_hosts
  environment=(env -u SINGLEFS_LAYER0_FINDINGS_FILE -u SINGLEFS_GPU_CARDS_CONFIG -u GPU_COMPUTATION_CLASSES -u GPU_CARDS_BY_PRIORITY)
  for variable in $(compgen -e); do [[ "$variable" == SINGLEFS_CRASH_AMPLIFICATION_* ]] && environment+=(-u "$variable"); done
  environment+=("BINDGEN_EXTRA_CLANG_ARGS=$CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS")
  [[ -f "$configuration_file" ]] && environment+=("SINGLEFS_MULTI_HOST_CONFIG=$(cd "$(dirname "$configuration_file")" && pwd)/$(basename "$configuration_file")")
  if (( gpu_switch )); then
    replay_break_is gpu-cards-not-sent || environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_for_fast_tier")
    replay_break_is fast-tier-classes-sent && environment+=("GPU_COMPUTATION_CLASSES=gpu_unit_checks crash_verify_gpu crash_judge_gpu")   # 弄坏开关：改前送的三类
    replay_break_is gpu-without-quiesce || quiesce_prefix=(bash "$replay_script_directory/gpu-quiesce-run.sh" --)
  fi
  echo "── 快档那一段：${#targets[@]} 个测试目标，cargo test --release -p singlefs-checker-tier --features $features（不带 --ignored${quiesce_prefix[*]:+；每一趟经 scripts/crash/gpu-quiesce-run.sh 清场再起}）"
  build_log="$run_directory/fast-tier-build.log"
  started=$SECONDS
  run_logged "$build_log" "${environment[@]}" cargo test --release -p singlefs-checker-tier --features "$features" --no-run "${target_arguments[@]}"
  build_exit="$(cat "$build_log.exit" 2>/dev/null || echo 255)"
  for target in "${targets[@]}"; do
    owners="$(awk -F'\t' -v wanted="$target" '$2 == "fast-tier" && $3 == wanted { print $1 }' "$plan_file")"
    if [[ "$build_exit" != 0 ]]; then
      for owner in $owners; do record_part "$owner" fast-tier "$target" "$build_exit" red 0 1 none "$((SECONDS - started))" "快档编不过（cargo test --no-run 退 $build_exit，日志 $build_log）"; done
      continue
    fi
    # 经给定状态引擎的测试目标，双机开着（或开着而判法判不过）：交双机驱动（文件头「双机」）
    if [[ -n "${given_states_targets[$target]+set}" ]] && { (( fast_tier_two_hosts )) || [[ -n "$fast_tier_two_hosts_refusal" ]]; }; then
      run_fast_tier_target_on_two_hosts "$target" "$owners"
      continue
    fi
    run_log="$run_directory/fast-tier.$target.log"
    started=$SECONDS
    run_logged "$run_log" ${quiesce_prefix[@]+"${quiesce_prefix[@]}"} "${environment[@]}" cargo test --release -p singlefs-checker-tier --features "$features" --test "$target" -- --nocapture
    run_exit="$(cat "$run_log.exit" 2>/dev/null || echo 255)"
    judge_fast_tier_log "$run_exit" "$run_log"
    for owner in $owners; do record_part "$owner" fast-tier "$target" "$fast_exit" "$fast_state" "$fast_tests" "$fast_red" none "$((SECONDS - started))" "$fast_reason"; done
  done
}

run_enumeration_part() { # <项> <crash-case: 键…>：交门禁 checker-tier-layer0-replay --full --item（全量先删全绿标记、带 --start-over）
  local item="$1" key arguments=(--full) log exit_code state reason reused manifest summary fingerprint marker started red_cases=0
  shift
  started=$SECONDS
  if [[ "$tier" == full ]] && ! replay_break_is full-keeps-reuse; then
    arguments+=(--start-over)
    for key in "$@"; do
      manifest="$run_directory/manifest.${key#crash-case:}"
      if summary="$(python3 "$repository_root/scripts/gate/admission.py" crash-case-manifest "$repository_root" "$key" "$manifest" --judging-digest --toolchain --build-environment)" \
         && read -r fingerprint _ <<<"$summary" && [[ "$fingerprint" =~ ^[0-9a-f]{64}$ ]] \
         && marker="$(python3 "$repository_root/scripts/gate/admission.py" crash-case-marker-path "$repository_root" "$key" "$fingerprint")"; then
        if [[ -n "$marker" && -e "$marker" ]]; then rm -f -- "${marker:?}"; echo "  · 全量（清空复用）：删掉 $key 这批输入的全绿标记 $marker"; fi
      else
        echo "  ! 全量（清空复用）：算不出 $key 这批输入的指纹或标记路径（admission.py crash-case-manifest / crash-case-marker-path 没成），这一条的全绿标记没删，门禁照它复用" >&2
      fi
    done
  fi
  for key in "$@"; do arguments+=(--item "$key"); done
  log="$run_directory/enumeration.$item.log"
  echo "── 层 0 部分（$item）：bash .claude/gate.d/checker-tier-layer0-replay.sh ${arguments[*]} <根>"
  run_logged "$log" bash "$repository_root/.claude/gate.d/checker-tier-layer0-replay.sh" "${arguments[@]}" "$repository_root"
  exit_code="$(cat "$log.exit" 2>/dev/null || echo 255)"
  reused="$(grep -cE '^  · crash-case:[a-z0-9-]+ 复用：' "$log")" || reused=0
  replay_break_is enumeration-exit-ignored && exit_code=0
  case "$exit_code" in
    0) state=green; reason="" ;;
    77) state=not-run; reason="层 0 那一道退 77（本次未跑）" ;;
    3) state=not-run; reason="层 0 那一道退 3（样本档，没跑真的层 0）" ;;
    78) state=not-run; reason="层 0 那一道退 78（运行条件不满足：修好设备，或改配置）" ;;
    *) state=red; reason="层 0 那一道退 $exit_code（日志 $log）"; red_cases=$# ;;
  esac
  record_part "$item" enumeration "$*" "$exit_code" "$state" "$#" "$red_cases" "$reused" "$((SECONDS - started))" "$reason"
}

# 判定库只建上层目录，库目录本身留给引擎新建（文件头「两个域的写法」：目录已经在，引擎拒 LibraryDirectoryAlreadyExists）
make_library_parent() { # <库目录>
  if replay_break_is library-directory-precreated; then mkdir -p -- "$1"; else mkdir -p -- "$(dirname -- "$1")"; fi
}

run_pipeline_part() { # <计划文件>：放量部分（pipeline-* 的件）一趟交 crash-amplification.sh
  local plan_file="$1" requested=() kind="" arguments=() log exit_code line name started fields state reason library="" all_named=1 word
  local item_exit item_states item_red item_reused item_verdict
  while IFS=$'\t' read -r _plan_item plan_kind plan_object; do
    [[ "$plan_kind" == pipeline-* ]] || continue
    kind="$plan_kind"; requested+=("$plan_object")
  done <"$plan_file"
  (( ${#requested[@]} )) || return 0
  for word in "${!pipeline_owner[@]}"; do [[ " ${requested[*]} " == *" $word "* ]] || all_named=0; done
  if [[ "$kind" == pipeline-expand-* ]]; then
    arguments+=(--expand-up-to "$LIGHT_EXPAND_UP_TO")
    for word in "${requested[@]}"; do arguments+=(--item "$word"); done
    if ! replay_break_is quick-reuses-library; then
      library="$pipeline_home/crash-point-replay-quick/$run_stamp/library"
      make_library_parent "$library" || { echo "  ✗ 建不了这一趟的空判定库的上层目录 $(dirname -- "$library")" >&2; echo "  → 怎么办：看 $pipeline_home 可不可写、盘满没满" >&2; exit 78; }
      arguments+=(--library "$library")
    fi
  else
    # 重型：全部项都点了就不带 --item（放量入口照全量判，scope=full）；全量换一份新的空判定库
    if [[ "$tier" == named ]] || (( ! all_named )); then
      for word in "${requested[@]}"; do arguments+=(--item "$word"); done
    fi
    if [[ "$tier" == full ]] && ! replay_break_is full-keeps-reuse; then
      library="$pipeline_home/crash-point-replay-full/$run_stamp/library"
      make_library_parent "$library" || { echo "  ✗ 建不了全量那一份空判定库的上层目录 $(dirname -- "$library")" >&2; echo "  → 怎么办：看 $pipeline_home 可不可写、盘满没满" >&2; exit 78; }
      arguments+=(--library "$library")
      echo "  · 全量（清空复用）：放量部分用一份新的空判定库 $library（跑完不删）"
    fi
  fi
  # 套在内存包装里：外层的上限给放量入口（文件头「内存包装」）
  [[ -n "$outer_memory_cap" ]] && arguments+=(--memory-cap "$outer_memory_cap")
  log="$run_directory/pipeline.log"
  echo "── 放量部分：bash scripts/crash/crash-amplification.sh ${arguments[*]}"
  started=$SECONDS
  run_logged "$log" bash "$repository_root/scripts/crash/crash-amplification.sh" ${arguments[@]+"${arguments[@]}"}
  exit_code="$(cat "$log.exit" 2>/dev/null || echo 255)"
  if [[ "$kind" == pipeline-expand-* && -n "$library" ]]; then
    rm -rf -- "${library:?}"
    rmdir -- "$(dirname "$library")" 2>/dev/null
  fi
  for name in "${requested[@]}"; do
    line="$(grep -E "^CRASH_AMPLIFICATION_ITEM item=$name " "$log" | tail -n 1)"
    if [[ -z "$line" ]]; then
      replay_break_is skip-collection-count && continue
      record_part "${pipeline_owner[$name]}" pipeline "$name" "$exit_code" red 0 1 unknown "$((SECONDS - started))" "放量入口退 $exit_code、没打 $name 的 CRASH_AMPLIFICATION_ITEM 行（收不回，日志 $log）"
      continue
    fi
    fields="$(python3 -c '
import sys
fields = dict(part.split("=", 1) for part in sys.argv[1].split()[1:] if "=" in part)
print(fields.get("exit", "?"), fields.get("states", "0"), fields.get("red", "?"), fields.get("reused", "unknown"), fields.get("verdict", "?"))' "$line")"
    read -r item_exit item_states item_red item_reused item_verdict <<<"$fields"
    state=green; reason=""
    if replay_break_is pipeline-red-ignored; then item_red=0; [[ "$item_verdict" == red ]] && item_verdict=green; fi
    if [[ "$item_verdict" == red || ! "$item_red" =~ ^[0-9]+$ ]] || (( item_red > 0 )); then
      state=red; reason="放量 $name 红（$line）"
    elif [[ "$item_verdict" == not-run ]]; then
      state=not-run; reason="放量 $name 本次未跑（exit=$item_exit）"
    elif [[ "$item_verdict" != green ]]; then
      state=red; reason="放量 $name 的 verdict 认不得（$line）"
    fi
    [[ "$item_red" =~ ^[0-9]+$ ]] || item_red=1
    [[ "$item_states" =~ ^[0-9]+$ ]] || item_states=0
    record_part "${pipeline_owner[$name]}" pipeline "$name" "$item_exit" "$state" "$item_states" "$item_red" "$item_reused" "$((SECONDS - started))" "$reason"
  done
}

# 汇总：按计划的项次序逐项打 CRASH_POINT_REPLAY_ITEM、末行 TOTAL、成功句；退出码写进 $run_directory/exit
summarize() { # <计划文件> <档的中文名>
  python3 - "$1" "$part_records" "$tier" "$2" "$run_directory/exit" "$(replay_break_is skip-collection-count && echo 1 || echo 0)" "$cross_check_skipped" <<'PYTHON'
import collections, sys
plan_file, records_file, tier, tier_title, exit_file, skip_count, cross_check_skipped = sys.argv[1:8]
order = []
for line in open(plan_file, encoding="utf-8"):
    item = line.rstrip("\n").split("\t")[0]
    if item and item not in order:
        order.append(item)
parts = collections.defaultdict(list)
for line in open(records_file, encoding="utf-8"):
    fields = line.rstrip("\n").split("\t")
    if len(fields) == 10:
        parts[fields[0]].append(fields)
totals = collections.Counter()
reuse_unknown = 0
verdicts = {}
any_78 = False
for item in order:
    if item not in parts:
        continue
    rows = parts[item]
    states = [row[4] for row in rows]
    verdict = "red" if "red" in states else ("not-run" if "not-run" in states else "green")
    has_78 = any(row[3] == "78" for row in rows)
    any_78 = any_78 or (has_78 and verdict != "red")
    exit_code = {"red": 1, "green": 0}.get(verdict, 78 if has_78 else 77)
    by_part = collections.defaultdict(list)
    for row in rows:
        by_part[row[1]].append(row)

    def total(part, column):
        return sum(int(row[column]) for row in by_part[part] if row[column].isdigit()) if by_part[part] else "-"
    fast_tests, fast_red = total("fast-tier", 5), total("fast-tier", 6)
    amp_states, amp_red = total("pipeline", 5), total("pipeline", 6)
    if by_part["pipeline"]:
        if all(row[7].isdigit() for row in by_part["pipeline"]):
            amp_reused = sum(int(row[7]) for row in by_part["pipeline"])
            totals["pipeline_reused_known"] += amp_reused
        else:
            amp_reused = "unknown"
            reuse_unknown += 1
    else:
        amp_reused = "-"
    enumeration_cases, enumeration_red, enumeration_reused = total("enumeration", 5), total("enumeration", 6), total("enumeration", 7)
    seconds = sum(int(row[8]) for row in rows if row[8].isdigit())
    for name, value in (("fast_tier_tests", fast_tests), ("pipeline_states", amp_states), ("enumeration_cases", enumeration_cases), ("enumeration_reused", enumeration_reused)):
        if isinstance(value, int):
            totals[name] += value
    totals["seconds"] += seconds
    verdicts[item] = (verdict, [row[9] for row in rows if row[4] == verdict and row[9]])
    print(f"CRASH_POINT_REPLAY_ITEM item={item} tier={tier} exit={exit_code} verdict={verdict} fast_tier_tests={fast_tests} fast_tier_red={fast_red} "
          f"fast_tier_reused={'none' if by_part['fast-tier'] else '-'} pipeline_states={amp_states} pipeline_red={amp_red} pipeline_reused={amp_reused} "
          f"enumeration_cases={enumeration_cases} enumeration_red={enumeration_red} enumeration_reused={enumeration_reused} seconds={seconds}")
dispatched, collected = len(order), len(verdicts)
red = [item for item, (verdict, _) in verdicts.items() if verdict == "red"]
not_run = [item for item, (verdict, _) in verdicts.items() if verdict == "not-run"]
green = [item for item, (verdict, _) in verdicts.items() if verdict == "green"]
print(f"CRASH_POINT_REPLAY_TOTAL tier={tier} dispatched={dispatched} collected={collected} green={len(green)} red={len(red)} not_run={len(not_run)} "
      f"fast_tier_tests={totals['fast_tier_tests']} pipeline_states={totals['pipeline_states']} pipeline_reused_known={totals['pipeline_reused_known']} "
      f"pipeline_reuse_unknown={reuse_unknown} enumeration_cases={totals['enumeration_cases']} enumeration_reused={totals['enumeration_reused']} seconds={totals['seconds']}")
for item in red:
    for reason in verdicts[item][1]:
        print(f"  ✗ {item}：{reason}", file=sys.stderr)  # gate-lint:detail
for item in not_run:
    for reason in verdicts[item][1]:
        print(f"  · 本次未跑 {item}：{reason}", file=sys.stderr)
count_mismatch = dispatched != collected and skip_count != "1"
if red or count_mismatch:
    if count_mismatch:
        missing = [item for item in order if item not in verdicts]
        print(f"  ✗ 派出 {dispatched} 项、收回 {collected} 项：{'、'.join(missing)} 一段都没记下", file=sys.stderr)  # gate-lint:detail
    print(f"  ✗ 崩溃点重放{tier_title}：{len(red)} 项红（{'、'.join(red) or '无'}）", file=sys.stderr)  # gate-lint:summary
    nameable = [item for item in red if not item.startswith("both-tiers:")]
    if tier == "quick":
        rerun = "轻型每次跑全部项，改完整趟重跑：bash scripts/hosts/run-with-memory-cap.sh <上限> bash scripts/crash/crash-point-replay.sh --quick"
    elif nameable:
        rerun = ("只重跑红的那 %d 项：SINGLEFS_HEAVY_TESTS=<commit|user-request> bash scripts/crash/crash-point-replay.sh %s%s"
                 % (len(nameable), "--full " if tier == "full" else "", " ".join(f"--item {item}" for item in nameable)))
        if len(nameable) != len(red):
            rerun += "（两档都跑的那几类不能点名，只在不带 --item 的跑法里跑）"
    else:
        rerun = "红的只有两档都跑的那几类（不能点名），整趟重跑同一条命令"
    print("  → 怎么办：照上面每一项的原因与日志查；重型红了必须修好或查清原因、红着不提交（.claude/rules/verification.md「两档：轻型与重型」）；"
          + rerun, file=sys.stderr)
    code = 1
elif any_78:
    print(f"  ✗ 崩溃点重放{tier_title}：有一段运行条件不满足（退 78），这一趟没跑全", file=sys.stderr)
    print("  → 怎么办：修好设备，或改配置（多机配置 multi-host.env）；看上面退 78 的那一段打的出路", file=sys.stderr)
    code = 78
elif not green:
    print(f"  ! 崩溃点重放{tier_title}：派出的 {dispatched} 项全是本次未跑（这不是通过）", file=sys.stderr)
    code = 77
else:
    not_run_text = "；本次未跑 %d 项：%s" % (len(not_run), "、".join(f"{item}（{'；'.join(verdicts[item][1])}）" for item in not_run)) if not_run else ""
    print(f"  ✓ 崩溃点重放{tier_title}：{len(green) + len(not_run)} 项都不红{not_run_text}；快档 {totals['fast_tier_tests']} 条用例、"
          f"放量现判 {totals['pipeline_states']} 态复用 {totals['pipeline_reused_known']} 态"
          f"{'（另有 %d 项取不出复用数）' % reuse_unknown if reuse_unknown else ''}、层 0 用例 {totals['enumeration_cases']} 条复用 {totals['enumeration_reused']} 条；"
          f"派出 {dispatched} 项、收回 {collected} 项{'；对照：用户授权跳过' if cross_check_skipped == '1' else ''}")
    code = 0
open(exit_file, "w").write(f"{code}\n")
PYTHON
}

run_selftest() {
  local work root calls cells=0 failures=0 exit_code
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-point-replay-selftest.XXXXXX")"
  trap 'rm -rf -- "${work:?}"' EXIT
  root="$work/root"; calls="$work/calls"
  mkdir -p "$root/.claude/gate.d" "$root/scripts" "$root/crates/singlefs-checker-tier/tests" "$work/bin" "$work/home" "$work/markers"
  printf '%s\n' "$SELFTEST_MARK_CONTENT" >"$work/mark"
  # 造出来的小仓：三项（两段都有、只有层 0、只有放量），合表核法照真的那一份判
  printf '%s\n' "# 自证的合表" $'alpha\tcrash-case:alpha\talpha-amp\talpha\t# 两段都有' $'beta\tcrash-case:beta\t-\tbeta\t# 只有层 0' \
    $'gamma\t-\tgamma-amp\tgamma_amp\t# 只有放量' >"$root/.claude/gate.d/crash-point-replay-items.tsv"
  printf '%s\n' "# 自证的登记表" $'crash-case:alpha\tcrates/\ttest=singlefs-checker-tier:alpha:alpha_full\t# 自证' \
    $'crash-case:beta\tcrates/\ttest=singlefs-checker-tier:beta:beta_full\t# 自证' >"$root/.claude/gate.d/stage-inputs.tsv"
  # 双机那几格（文件头「双机」）临时换成 crash-case:beta 带 features= 的一份（beta 那一项算经给定状态引擎的），跑完换回
  cp -- "$root/.claude/gate.d/stage-inputs.tsv" "$work/stage-inputs-plain.tsv"
  sed 's/^\(crash-case:beta\t.*test=singlefs-checker-tier:beta:beta_full\)\t/\1 features=verdict-store\t/' "$work/stage-inputs-plain.tsv" >"$work/stage-inputs-given-states.tsv"
  : >"$root/crates/singlefs-checker-tier/tests/alpha.rs"; : >"$root/crates/singlefs-checker-tier/tests/beta.rs"; : >"$root/crates/singlefs-checker-tier/tests/gamma_amp.rs"
  # 按段登记文件：一行登记（alpha 第 0 段），segments-registry-refused 那一格临时换成核不过的一份、跑完换回
  printf '%s\n' "# 自证的按段登记" "alpha	0	7	$(printf '0123456789abcdef%.0s' 1 2 3 4)" >"$root/.claude/gate.d/crash-point-replay-segments.tsv"
  cat >"$root/scripts/crash/crash-amplification.sh" <<'STUB'
#!/usr/bin/env bash
# 假的崩溃放量入口（自证用）：ITEMS 给合表核法读；起它时记一行参数，按 --item（没给就 ITEMS 全部）逐项打 CRASH_AMPLIFICATION_ITEM 行
ITEMS=(
  $'alpha-amp\talpha\talpha_amplified\t全量'
  $'gamma-amp\tgamma_amp\tgamma_amplified\t全量'
)
printf 'pipeline-entry %s\n' "$*" >>"${STUB_CALLS:?}"
printf 'pipeline-env skip=[%s] segments=[%s]\n' "${SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER-unset}" "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" >>"$STUB_CALLS"
if [[ -n "${STUB_AMP_ENGINE_LIBRARY:-}" ]]; then   # 照引擎建库：<库>/CURRENT 在就开；不在而目录已经在就拒（LibraryDirectoryAlreadyExists）、一项都不打；都不在就建出来
  library=""; previous=""
  for argument in "$@"; do [[ "$previous" == --library ]] && library="$argument"; previous="$argument"; done
  if [[ -n "$library" && ! -e "$library/CURRENT" ]]; then
    if [[ -e "$library" ]]; then
      echo "pipeline-library refused $library" >>"$STUB_CALLS"
      echo "建不了库 $library：LibraryDirectoryAlreadyExists（假的放量入口，自证）"
      exit 1
    fi
    mkdir -p -- "$library" && : >"$library/CURRENT"
    echo "pipeline-library created $library" >>"$STUB_CALLS"
  fi
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
  echo "CRASH_AMPLIFICATION_ITEM item=$item exit=$item_exit states=10 reused=4 red=$red lines=1 expand_up_to=6 exhaustive=true verdict=$verdict seconds=0 ignored_points=0 states_skipped_by_ignore=0"
done
exit "${STUB_AMP_EXIT:-$exit_code}"
STUB
  cat >"$root/.claude/gate.d/checker-tier-layer0-replay.sh" <<'STUB'
#!/usr/bin/env bash
# 假的门禁 checker-tier-layer0-replay（自证用）：记一行参数；STUB_LAYER0_REUSED 设了就给每条点名的用例打一行复用；退 STUB_LAYER0_EXIT
printf 'enumeration-gate %s\n' "$*" >>"${STUB_CALLS:?}"
previous=""
for argument in "$@"; do
  [[ "$previous" == --item && -n "${STUB_LAYER0_REUSED:-}" ]] && echo "  · $argument 复用：这批输入那一格全绿标记作数（自证）"
  previous="$argument"
done
exit "${STUB_LAYER0_EXIT:-0}"
STUB
  cat >"$root/scripts/gate/admission.py" <<'STUB'
# 假的准入模块（自证用）：记一行参数；crash-case-manifest 打一份假指纹，crash-case-marker-path 打 STUB_MARKERS 下以键为名的标记
import os, sys
open(os.environ["STUB_CALLS"], "a").write("admission " + " ".join(sys.argv[1:]) + "\n")
if sys.argv[1] == "crash-case-manifest":
    print("a" * 64, 1, 0)
elif sys.argv[1] == "crash-case-marker-path":
    print(os.path.join(os.environ["STUB_MARKERS"], sys.argv[3]))
STUB
  cat >"$work/bin/cargo" <<'STUB'
#!/usr/bin/env bash
# 假 cargo（自证用）：每次调记一行（参数、这一趟的显卡先后、计算类、是不是在清场包装之下、对照跳过的开关、按段登记文件的路径）；按 --test 点名的目标打 libtest 的样子
printf 'cargo %s cards=[%s] classes=[%s] quiesced=[%s] skip=[%s] segments=[%s]\n' "$*" "${GPU_CARDS_BY_PRIORITY-unset}" "${GPU_COMPUTATION_CLASSES-unset}" "${GPU_QUIESCE_HOLDER:+yes}" \
  "${SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER-unset}" "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" >>"${STUB_CALLS:?}"
[[ " $* " == *" --no-run "* ]] && exit "${STUB_BUILD_EXIT:-0}"
target=""; previous=""
for argument in "$@"; do [[ "$previous" == --test ]] && target="$argument"; previous="$argument"; done
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
printf 'running 2 tests\ntest first ... ok\ntest second ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'
STUB
  chmod +x "$work/bin/cargo"
  if [[ "$(PATH="$work/bin:$PATH" type -P cargo)" != "$work/bin/cargo" ]]; then
    echo "  ✗ 自证 setup：PATH 最前面放了假 cargo，解到的却不是它（$(PATH="$work/bin:$PATH" type -P cargo)），不起子进程（起了就是真 cargo）" >&2
    echo "  → 怎么办：看 $work/bin/cargo 可不可执行、TMPDIR 是不是挂成了 noexec" >&2
    exit 1
  fi
  # 多机配置两份：没开 GPU（只写假清场命令）、开了 GPU（加显卡先后两份）；清场往调用记录记一笔
  printf '%s\n' "QUIESCE_STOP_COMMAND='echo quiesce-stop >> \"\$STUB_CALLS\"'" "QUIESCE_STOPPED_CHECK_COMMAND=true" >"$work/multi-host-gpu-off.env"
  { cat "$work/multi-host-gpu-off.env"; printf '%s\n' ENABLE_GPU=1 'GPU_CARDS_BY_PRIORITY="local:0:8G"' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="local:0:24G"'; } >"$work/multi-host-gpu-on.env"
  # 双机那几格（文件头「双机」）的多机配置：开了双机、键齐（判法带 SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1，不连第二台）；开了双机而缺键（判法判不过）
  printf '%s\n' PEER_SSH_HOST=peer-selftest PEER_REPOSITORY_DIRECTORY=/nonexistent/peer-tree PEER_CARGO_BIN_DIRECTORY=/nonexistent/bin PEER_MEMORY_CAP=4G \
    QUIESCE_STOP_COMMAND= QUIESCE_STOPPED_CHECK_COMMAND= QUIESCE_START_COMMAND= QUIESCE_STARTED_CHECK_COMMAND= ENABLE_ACROSS_MACHINES=1 >"$work/multi-host-two-hosts.env"
  printf '%s\n' ENABLE_ACROSS_MACHINES=1 PEER_SSH_HOST=peer-selftest >"$work/multi-host-two-hosts-broken.env"
  # 双机驱动是真的 scripts/crash/crash-amplification-two-hosts.sh，只换掉它本机那一趟与第二台那一趟（它文件头「只供测试」的两个变量，entry 每一格都设上）
  cat >"$work/two-hosts-local.sh" <<'STUB'
#!/usr/bin/env bash
# 假的双机驱动本机那一趟（自证用，CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN）：调法 <share> <库目录> <日志> [<附加环境 键=值>…]，环境由双机驱动照真的那一趟组。
# 记一行进 STUB_CALLS（份、IMPORT 指到哪、那里有没有取回的库）；带 IMPORT 的那一趟照 libtest 的样子打两条都过，没带的打一行本次未跑（表归第二台判）
share="$1" library="$2" log="$3"
import="${SINGLEFS_CRASH_AMPLIFICATION_IMPORT-unset}" fetched=no
[[ "$import" != unset && -f "$import/CURRENT" ]] && fetched=yes
printf 'two-hosts-local share=%s import=%s fetched=%s library=%s\n' "$share" "$import" "$fetched" "$library" >>"${STUB_CALLS:?}"
mkdir -p -- "$library" && : >"$library/CURRENT"
if [[ "$import" == unset ]]; then
  printf 'running 1 test\nNOT_RUN_NO_GPU_ADAPTER test=given reason=两台分片：这一份是 %s，有表归第二台判（自证）\ntest given ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n' "$share" >"$log"
else
  printf 'running 2 tests\ntest first ... ok\ntest second ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n' >"$log"
fi
STUB
  cat >"$work/two-hosts-peer.sh" <<'STUB'
#!/usr/bin/env bash
# 假的 multi-host-run.sh（自证用，CRASH_AMPLIFICATION_TWO_HOSTS_PEER_RUN）：参数与它相同；不连第二台，记一行进 STUB_CALLS（份、取回哪个目录），
# --fetch 点名的目录建在 MULTI_HOST_RUN_FETCH_DIRECTORY 下（带 CURRENT，当成第二台判完的库）。STUB_PEER_EXIT=255：连不上，什么都不跑、不取回，退 255
fetch_name=""
while (($# > 0)); do
  case "$1" in
    --peer) shift ;;
    --label|--memory-cap) shift 2 ;;
    --fetch) fetch_name="$2"; shift 2 ;;
    --send) shift 3 ;;
    --) shift; break ;;
    *) echo "假的 multi-host-run.sh：认不得参数 $1"; exit 2 ;;
  esac
done
share=""
for word in "$@"; do [[ "$word" == SINGLEFS_CRASH_AMPLIFICATION_SHARE=* ]] && share="${word#*=}"; done
if [[ "${STUB_PEER_EXIT:-}" == 255 ]]; then
  echo "two-hosts-peer unreachable share=$share" >>"${STUB_CALLS:?}"
  echo "multi-host-run.sh（假的）：第二台连不上"
  exit 255
fi
echo "two-hosts-peer share=$share fetch=$fetch_name" >>"${STUB_CALLS:?}"
if [[ -n "$fetch_name" ]]; then mkdir -p -- "${MULTI_HOST_RUN_FETCH_DIRECTORY:?}/$fetch_name" && : >"$MULTI_HOST_RUN_FETCH_DIRECTORY/$fetch_name/CURRENT"; fi
printf 'running 1 test\ntest given ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'
STUB
  export GPU_QUIESCE_STATE_DIRECTORY="$work/gpu-quiesce-state" GPU_QUIESCE_LOG_DIRECTORY="$work/gpu-quiesce-logs"
  unset GATE_NOT_RUN_FILE STUB_RUN_CONDITION_TARGETS STUB_PEER_EXIT STUB_RED_TARGETS STUB_ZERO_TARGETS STUB_NOT_RUN_TARGETS STUB_AMP_DROP STUB_AMP_RED STUB_AMP_NOT_RUN STUB_AMP_EXIT STUB_AMP_ENGINE_LIBRARY STUB_LAYER0_EXIT STUB_LAYER0_REUSED STUB_BUILD_EXIT

  cell_failed() { # <格名> <判错的一句> <出路>
    echo "  ✗ 自证 $1：$2（退 $exit_code；调用记录：$(tr '\n' '|' <"$calls" | cut -c1-600)）" >&2
    echo "     → 怎么办：$3" >&2
    failures=$((failures + 1))
  }
  entry() { # [<变量=值>…] -- <参数…>：在小仓上起这一份（子进程认自证标记），输出进 $work/out、退出码进 exit_code
    local settings=()
    while (($#)) && [[ "$1" != -- ]]; do settings+=("$1"); shift; done
    shift
    : >"$calls"
    cells=$((cells + 1))
    env -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP -u SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE \
      PATH="$work/bin:$PATH" STUB_CALLS="$calls" STUB_MARKERS="$work/markers" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" \
      SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" CRASH_POINT_REPLAY_SELFTEST_MARK="$work/mark" CRASH_POINT_REPLAY_ROOT="$root" \
      CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN="$work/two-hosts-local.sh" CRASH_AMPLIFICATION_TWO_HOSTS_PEER_RUN="$work/two-hosts-peer.sh" CRASH_AMPLIFICATION_PEER_THREADS=2 \
      CRASH_POINT_REPLAY_BREAK="$replay_break" ${settings[@]+"${settings[@]}"} bash "$replay_script" "$@" >"$work/out" 2>&1
    exit_code=$?
  }
  calls_have() { grep -qE -- "$1" "$calls"; }
  calls_count() { grep -cE -- "$1" "$calls"; }
  out_has() { grep -qF -- "$1" "$work/out"; }
  item_line() { grep -E "^CRASH_POINT_REPLAY_ITEM item=$1 " "$work/out"; }
  item_has() { grep -qE "^CRASH_POINT_REPLAY_ITEM item=$1 .*$2" "$work/out"; }   # <项> <那一行里要有的一段>（不接管道：pipefail 下 grep -q 收尾的管道会把命中读成没命中）

  local extras_count=8
  # list-items：退 0、三行、不起 cargo 与驱动
  entry -- --list-items
  if [[ "$exit_code" != 0 || "$(grep -cE $'^(alpha|beta|gamma)\t轻型：' "$work/out")" != 3 ]] || [[ -s "$calls" ]]; then
    cell_failed list-items "--list-items 应当退 0、逐项列出、不起 cargo 与驱动" "看 --list-items 那一段：只读合表、打表就退"
  fi
  # unknown-item / missing-item-value / quick-and-full：用法错退 2、什么都不起
  entry -- --item no-such-item
  if [[ "$exit_code" != 2 ]] || ! out_has "alpha" || ! out_has "gamma" || [[ -s "$calls" ]]; then
    cell_failed unknown-item "项名写错应当退 2、列出可点名的项、什么都不起" "看核项名那一段"
  fi
  entry -- --item
  [[ "$exit_code" == 2 && ! -s "$calls" ]] || cell_failed missing-item-value "--item 后面缺项名应当退 2" "看参数那一段的 --item"
  entry -- --quick --full
  [[ "$exit_code" == 2 && ! -s "$calls" ]] || cell_failed quick-and-full "--quick 与 --full 一起给应当退 2" "看定档那一段"
  # quick-runs-the-light-domains：全部项的甲二（三个测试目标，加两档都跑的那几类）与展开 6（两个放量分项、一份新建跑完就删的空库），不碰层 0 那一道
  entry -- --quick
  if [[ "$exit_code" != 0 ]] || ! calls_have '^cargo test --release -p singlefs-checker-tier --features verdict-store --no-run --test alpha --test beta --test gamma_amp --test crash_judge_gpu_agrees' \
     || [[ "$(calls_count '^cargo test --release -p singlefs-checker-tier --features verdict-store --test [a-z0-9_]+ -- --nocapture ')" != $((3 + extras_count)) ]] \
     || ! calls_have "^pipeline-entry --expand-up-to $LIGHT_EXPAND_UP_TO --item alpha-amp --item gamma-amp --library $work/home/crash-point-replay-quick/[^ ]+/library\$" \
     || calls_have '^enumeration-gate ' || [[ -n "$(ls -A "$work/home/crash-point-replay-quick" 2>/dev/null)" ]] \
     || ! item_has alpha 'verdict=green' || ! item_has 'both-tiers:pinned-red-examples' 'verdict=not-run' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL tier=quick dispatched=6 collected=6 green=5 red=0 not_run=1 ' "$work/out"; then
    cell_failed quick-runs-the-light-domains "轻型应当照甲二与展开 6 跑全部项（快档 $((3 + extras_count)) 个测试目标、放量两项带新建的空库跑完删掉、不起层 0），三项与两档都跑的三类都收回、钉表那一类报本次未跑" \
      "看 replay_plan 的 quick 那一支、run_pipeline_part 的 --library 与 BOTH_TIERS_GROUPS"
  fi
  # quick-named-item：--quick --item beta 照样跑核心部分（全部项）
  entry -- --quick --item beta
  if [[ "$exit_code" != 0 ]] || ! calls_have '^pipeline-entry --expand-up-to [0-9]+ --item alpha-amp --item gamma-amp ' || ! out_has "点名的项已在核心部分里"; then
    cell_failed quick-named-item "--quick --item 应当照样跑核心部分（全部项）、说明点名的项已在里面" "看定档那一段 quick 的 selected_items"
  fi
  # quick-gpu：开了 GPU，快档带 gpu 特性、每一趟经清场包装起、交清场之后那一份显卡先后、不送走 GPU 的计算类（漏进来的也清掉）；编那一趟不清场
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" GPU_COMPUTATION_CLASSES=crash_judge_gpu -- --quick
  if [[ "$exit_code" != 0 ]] || ! calls_have '^quiesce-stop$' \
     || [[ "$(calls_count "^cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --test [a-z0-9_]+ -- --nocapture cards=\[local:0:24G\] classes=\[unset\] quiesced=\[yes\]")" != $((3 + extras_count)) ]] \
     || ! calls_have '^cargo test .*--features verdict-store,gpu --no-run .*quiesced=\[\]'; then
    cell_failed quick-gpu "开了 GPU 快档应当带 gpu 特性、经清场包装起、交清场之后那一份显卡先后（local:0:24G）、不送 GPU_COMPUTATION_CLASSES（调用方环境里漏进来的也清掉）" \
      "看 run_fast_tier 里 gpu_switch 那一段与 environment 开头的 -u GPU_COMPUTATION_CLASSES"
  fi
  # fast-tier-red：beta 那个测试目标红 ⇒ beta 红、退 1，别的照绿
  entry STUB_RED_TARGETS=beta -- --quick
  if [[ "$exit_code" != 1 ]] || ! item_has beta 'verdict=red' || ! item_has alpha 'verdict=green' || ! out_has "✗ beta："; then
    cell_failed fast-tier-red "快档一个测试目标红了，那一项应当红、整趟退 1" "看 run_fast_tier 判 state 那一段"
  fi
  # fast-tier-not-run：beta 跑到 0 条、alpha 有 NOT_RUN_NO_GPU_ADAPTER ⇒ 两项本次未跑、退 0，成功句点名
  entry STUB_ZERO_TARGETS=beta STUB_NOT_RUN_TARGETS=alpha -- --quick
  if [[ "$exit_code" != 0 ]] || ! item_has beta 'verdict=not-run' || ! item_has alpha 'verdict=not-run' \
     || ! grep -q '本次未跑 3 项：alpha（有 1 处 NOT_RUN_NO_GPU_ADAPTER' "$work/out"; then
    cell_failed fast-tier-not-run "快档跑到 0 条或没卡没跑的，应当记本次未跑、成功句点名" "看 run_fast_tier 判 state 那一段与 summarize 的成功句"
  fi
  # all-not-run：快档全是 0 条、放量全是 not-run ⇒ 退 77
  entry STUB_ZERO_TARGETS=all STUB_AMP_NOT_RUN="alpha-amp gamma-amp" -- --quick
  [[ "$exit_code" == 77 ]] || cell_failed all-not-run "派出的项全是本次未跑，应当退 77" "看 summarize 里 not green 那一支"
  # pruned-heavy：不带参数 ⇒ 快档（连同两档都跑的那几类）、层 0 每项一趟 --full --item（不带 --start-over）、放量不带 --item 与 --library；复用数照实报
  entry STUB_LAYER0_REUSED=1 --
  if [[ "$exit_code" != 0 ]] || [[ "$(calls_count '^cargo test --release -p singlefs-checker-tier --features verdict-store --test [a-z0-9_]+ -- --nocapture ')" != $((3 + extras_count)) ]] \
     || ! calls_have "^enumeration-gate --full --item crash-case:alpha $root\$" || ! calls_have "^enumeration-gate --full --item crash-case:beta $root\$" \
     || ! calls_have '^pipeline-entry $' || calls_have 'start-over' || calls_have '^admission ' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL tier=pruned dispatched=6 collected=6 .* pipeline_reused_known=8 .* enumeration_cases=2 enumeration_reused=2 ' "$work/out"; then
    cell_failed pruned-heavy "不带参数的重型应当照剪枝跑全部项（快档照跑、层 0 每项 --full --item、放量照全量、复用数照实报）" "看 replay_plan 的重型那一支与 run_enumeration_part"
  fi
  # named-heavy：--item gamma ⇒ 只跑 gamma：快档只 gamma_amp、不跑两档都跑的那几类、不起层 0、放量 --item gamma-amp
  entry -- --item gamma
  if [[ "$exit_code" != 0 ]] || [[ "$(calls_count ' --test [a-z0-9_]+ -- --nocapture ')" != 1 ]] || ! calls_have ' --test gamma_amp -- --nocapture ' \
     || calls_have '^enumeration-gate ' || ! calls_have '^pipeline-entry --item gamma-amp$' || item_line 'both-tiers:pruning-self-proof' >/dev/null; then
    cell_failed named-heavy "--item 应当只跑点名的项（逐条）" "看定档那一段 named 与 run_pipeline_part 的 --item"
  fi
  # full-heavy：--full ⇒ 先删每条用例的全绿标记、层 0 带 --start-over、放量带一份新的空库（留着）
  : >"$work/markers/crash-case:alpha"; : >"$work/markers/crash-case:beta"
  entry -- --full
  if [[ "$exit_code" != 0 ]] || [[ -e "$work/markers/crash-case:alpha" || -e "$work/markers/crash-case:beta" ]] \
     || ! calls_have "^enumeration-gate --full --start-over --item crash-case:alpha $root\$" \
     || ! calls_have "^pipeline-entry --library $work/home/crash-point-replay-full/[^ ]+/library\$" || ! calls_have '^admission crash-case-manifest '; then
    cell_failed full-heavy "全量应当清空复用（删全绿标记、--start-over、新的空判定库）" "看 run_enumeration_part 与 run_pipeline_part 里 tier=full 那两段"
  fi
  # library-left-to-the-engine：假的放量入口照引擎建库（库目录已经在而没有 CURRENT 就拒、退 1）⇒ 轻型与 --full 两支交下去的库目录都还不在、由入口建出来；轻型跑完连同上层删掉
  entry STUB_AMP_ENGINE_LIBRARY=1 -- --quick
  if [[ "$exit_code" != 0 ]] || ! calls_have "^pipeline-library created $work/home/crash-point-replay-quick/[^ ]+/library\$" || calls_have '^pipeline-library refused ' \
     || [[ -n "$(ls -A "$work/home/crash-point-replay-quick" 2>/dev/null)" ]]; then
    cell_failed library-left-to-the-engine "轻型那一支交给放量入口的判定库目录应当还不在（只建上层，库目录本身留给引擎新建：已经在的目录引擎拒 LibraryDirectoryAlreadyExists）、跑完连同上层删掉" \
      "看 make_library_parent 与 run_pipeline_part 里 pipeline-expand 那一支"
  fi
  entry STUB_AMP_ENGINE_LIBRARY=1 -- --full
  if [[ "$exit_code" != 0 ]] || ! calls_have "^pipeline-library created $work/home/crash-point-replay-full/[^ ]+/library\$" || calls_have '^pipeline-library refused '; then
    cell_failed library-left-to-the-engine "--full 那一支交给放量入口的判定库目录应当还不在（只建上层，库目录本身留给引擎新建：已经在的目录引擎拒 LibraryDirectoryAlreadyExists）" \
      "看 make_library_parent 与 run_pipeline_part 里 tier=full 那一段"
  fi
  # enumeration-red：层 0 那一道退 1 ⇒ alpha、beta 红，退 1
  entry STUB_LAYER0_EXIT=1 -- --item alpha --item beta
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'verdict=red' || ! item_has beta 'enumeration_red=1' \
     || ! out_has "只重跑红的那 2 项：SINGLEFS_HEAVY_TESTS=<commit|user-request> bash scripts/crash/crash-point-replay.sh --item alpha --item beta"; then
    cell_failed enumeration-red "层 0 那一道红了，那一项应当红、出路给出只重跑那两项的命令" "看 run_enumeration_part 判 exit_code 那一段与 summarize 的出路"
  fi
  # pipeline-red：放量 alpha-amp red=2 ⇒ alpha 红
  entry STUB_AMP_RED=alpha-amp -- --quick
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'pipeline_red=2' || ! item_has alpha 'verdict=red' || ! item_has gamma 'verdict=green'; then
    cell_failed pipeline-red "放量那一项红了，那一项应当红" "看 run_pipeline_part 判 state 那一段"
  fi
  # collection-count：放量入口没打 gamma-amp 那一行 ⇒ gamma 红（收不回）
  entry STUB_AMP_DROP=gamma-amp -- --quick
  if [[ "$exit_code" != 1 ]] || ! item_has gamma 'verdict=red' || ! out_has "收不回"; then
    cell_failed collection-count "放量那一项收不回，应当红、派出收回对不上" "看 run_pipeline_part 里 line 为空那一支"
  fi
  # test-variables-outside-selftest：不带自证标记、带了 CRASH_POINT_REPLAY_ROOT ⇒ 退 2、什么都不起
  : >"$calls"; cells=$((cells + 1))
  env PATH="$work/bin:$PATH" STUB_CALLS="$calls" STUB_MARKERS="$work/markers" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" \
    SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" CRASH_POINT_REPLAY_ROOT="$root" \
    CRASH_POINT_REPLAY_BREAK="$replay_break" bash "$replay_script" --quick >"$work/out" 2>&1
  exit_code=$?
  [[ "$exit_code" == 2 && ! -s "$calls" ]] || cell_failed test-variables-outside-selftest "平常跑时带了只供自证的变量应当退 2" "看开头拒只供自证变量那一段"
  # 往引擎送的两个变量（文件头「往引擎送的两个变量」）：快档每一趟 cargo test 与放量入口那一趟都要带到
  local fast_runs_pattern='^cargo test --release -p singlefs-checker-tier --features verdict-store --test [a-z0-9_]+ -- --nocapture '
  local registry="$root/.claude/gate.d/crash-point-replay-segments.tsv"
  every_fast_run_has() { # <那一行里要有的一段（正则）>：快档至少跑了一趟，且每一趟 cargo test 的调用记录都有它
    local all
    all="$(calls_count "$fast_runs_pattern")"
    (( all > 0 )) && [[ "$(calls_count "${fast_runs_pattern}.*$1")" == "$all" ]]
  }
  # cross-check-skip-sent：给了 --cross-check-skipped-by-the-user ⇒ 每一趟都带 =1、成功句写「对照：用户授权跳过」
  entry -- --quick --cross-check-skipped-by-the-user
  if [[ "$exit_code" != 0 ]] || ! every_fast_run_has ' skip=\[1\] ' \
     || ! calls_have '^pipeline-env skip=\[1\] ' || ! grep -q '✓ 崩溃点重放.*；对照：用户授权跳过$' "$work/out"; then
    cell_failed cross-check-skip-sent "给了 --cross-check-skipped-by-the-user，快档与放量每一趟都应当带 SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER=1、成功句写「对照：用户授权跳过」" \
      "看「往引擎送的两个变量」那一段的 cross_check_skipped 那一支与 summarize 的成功句"
  fi
  # cross-check-skip-cleared：没给、调用方环境里带着 =1 ⇒ 每一趟都不带、成功句不写跳过
  entry SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER=1 -- --quick
  if [[ "$exit_code" != 0 ]] || ! every_fast_run_has ' skip=\[unset\] ' \
     || ! calls_have '^pipeline-env skip=\[unset\] ' || out_has "用户授权跳过"; then
    cell_failed cross-check-skip-cleared "没给 --cross-check-skipped-by-the-user，调用方环境里带着那个变量也应当清掉：快档与放量每一趟都不带、成功句不写跳过" \
      "看「往引擎送的两个变量」那一段没给选项时的 unset"
  fi
  # segments-registry-sent：每一趟都带按段登记文件的绝对路径
  entry -- --quick
  if [[ "$exit_code" != 0 ]] || ! every_fast_run_has " segments=\[$registry\]\$" \
     || ! calls_have "^pipeline-env .* segments=\[$registry\]\$"; then
    cell_failed segments-registry-sent "快档与放量每一趟都应当带按段登记文件的绝对路径（SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=$registry）" \
      "看「往引擎送的两个变量」那一段的 export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY"
  fi
  # segments-registry-refused：按段登记文件核不过（项名不在合表里）⇒ 退 2、什么都不起
  cp -- "$registry" "$work/segments-saved.tsv"
  printf '%s\n' "# 自证：核不过的按段登记" "ghost	0	7	$(printf '0123456789abcdef%.0s' 1 2 3 4)" >"$registry"
  entry -- --quick
  cp -- "$work/segments-saved.tsv" "$registry"
  if [[ "$exit_code" != 2 || -s "$calls" ]] || ! out_has "项名 ghost 不是合表"; then
    cell_failed segments-registry-refused "按段登记文件核不过应当退 2、什么都不起、逐处列出问题" \
      "看「往引擎送的两个变量」那一段开头的 lib_crash_point_replay_items.py segments"
  fi
  # fast-tier-run-condition-not-met：alpha 那个测试目标的日志里有给定状态表借不到显存那一行（cargo 退 101）⇒ alpha 记本次未跑、写原因、退 78，别的照绿
  entry STUB_RUN_CONDITION_TARGETS=alpha -- --quick
  if [[ "$exit_code" != 78 ]] || ! item_has alpha 'exit=78 verdict=not-run ' || ! item_has beta 'verdict=green' \
     || ! out_has "本次未跑 alpha：运行条件不满足：给定状态表的判器借不到显存（$CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX table=stub-table"; then
    cell_failed fast-tier-run-condition-not-met "快档日志里有给定状态表借不到显存那一行（$CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX），那一项应当记本次未跑、写原因，整趟退 78" \
      "看 judge_fast_tier_log 里 condition_lines 那一支与 summarize 的 any_78"
  fi
  # outer-memory-cap-forwarded：套在内存包装里（包装给命令设的 RUN_WITH_MEMORY_CAP_OUTER_CAP=3G）⇒ 放量入口带 --memory-cap 3G
  entry RUN_WITH_MEMORY_CAP_OUTER_CAP=3G -- --quick
  if [[ "$exit_code" != 0 ]] || ! calls_have "^pipeline-entry .* --memory-cap 3G\$"; then
    cell_failed outer-memory-cap-forwarded "套在内存包装里，放量入口应当带外层的上限（RUN_WITH_MEMORY_CAP_OUTER_CAP=3G ⇒ --memory-cap 3G）" "看文件头「内存包装」与 run_pipeline_part 里 outer_memory_cap 那一行"
  fi
  # 双机那几格（文件头「双机」）：beta 那一项临时登记成经给定状态引擎的；双机驱动是真的，本机那一趟与第二台那一趟是 entry 设的两份假的
  cp -- "$work/stage-inputs-given-states.tsv" "$root/.claude/gate.d/stage-inputs.tsv"
  local two_hosts_settings=(SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-two-hosts.env" SINGLEFS_LAYER0_SHARD_PEER_IS_THIS_MACHINE=1) last_two_hosts_call import_wanted
  import_wanted="^two-hosts-local share=0/2 import=$work/home/runs/crash-point-replay-[^ ]+-beta/fetched/crash-amplification-library fetched=yes "
  # fast-tier-two-hosts：开了双机 ⇒ beta 交双机驱动：本机 0/2、第二台 1/2 取回库、最后本机带 IMPORT（指到取回的库）再跑一趟；照导入那一趟判（2 条都过）；
  # alpha 照旧本机一趟；外层的上限给双机驱动；那一份空库跑完删掉
  entry "${two_hosts_settings[@]}" RUN_WITH_MEMORY_CAP_OUTER_CAP=3G -- --quick
  last_two_hosts_call="$(grep -E '^two-hosts-' "$calls" | tail -n 1)"
  if [[ "$exit_code" != 0 ]] || ! grep -q 'features=verdict-store' "$root/.claude/gate.d/stage-inputs.tsv" || [[ "$(calls_count '^two-hosts-')" != 3 ]] \
     || ! calls_have '^two-hosts-local share=0/2 import=unset ' || ! calls_have '^two-hosts-peer share=1/2 fetch=crash-amplification-library$' \
     || ! [[ "$last_two_hosts_call" =~ $import_wanted ]] || calls_have ' --test beta -- --nocapture ' || ! calls_have ' --test alpha -- --nocapture ' \
     || ! item_has beta 'exit=0 verdict=green fast_tier_tests=2 ' || ! out_has 'crash-amplification-two-hosts.sh beta --library ' || ! out_has ' --memory-cap 3G' \
     || [[ -n "$(ls -A "$work/home/crash-point-replay-fast-tier" 2>/dev/null)" ]]; then
    cell_failed fast-tier-two-hosts "开了双机，经给定状态引擎的测试目标应当照双机驱动的三步跑（本机 0/2、第二台 1/2 取回库、本机带 IMPORT 指到取回的库再跑，照导入那一趟判），别的测试目标照旧本机一趟，外层的上限给双机驱动，空库跑完删掉（最后一笔双机调用：${last_two_hosts_call:-没有}）" \
      "看 run_fast_tier 里 given_states_targets 那一支、decide_fast_tier_two_hosts 与 run_fast_tier_target_on_two_hosts"
  fi
  # fast-tier-two-hosts-off：双机关着（没写 ENABLE_ACROSS_MACHINES=1）⇒ beta 照旧本机一趟，不起双机驱动
  entry -- --quick
  if [[ "$exit_code" != 0 ]] || calls_have '^two-hosts-' || ! calls_have ' --test beta -- --nocapture ' || ! item_has beta 'exit=0 verdict=green'; then
    cell_failed fast-tier-two-hosts-off "双机关着，经给定状态引擎的测试目标应当照旧本机一趟（不起双机驱动）" "看 decide_fast_tier_two_hosts：没写 ENABLE_ACROSS_MACHINES=1 就不调判法、不交双机驱动"
  fi
  # fast-tier-two-hosts-peer-unreachable：第二台连不上（假的 multi-host-run.sh 退 255、双机驱动退 78）⇒ beta 记本次未跑、写原因、整趟退 78
  entry "${two_hosts_settings[@]}" STUB_PEER_EXIT=255 -- --quick
  if [[ "$exit_code" != 78 ]] || ! calls_have '^two-hosts-peer unreachable share=1/2$' || ! item_has beta 'exit=78 verdict=not-run ' || ! out_has '本次未跑 beta：运行条件不满足：双机驱动退 78'; then
    cell_failed fast-tier-two-hosts-peer-unreachable "第二台连不上（双机驱动退 78），那一项应当记本次未跑、写原因、整趟退 78" "看 run_fast_tier_target_on_two_hosts 里双机驱动退 78 那一支"
  fi
  # fast-tier-two-hosts-configuration-refused：开了双机而多机配置缺键（判法退 1）⇒ beta 一趟都不起、记本次未跑、整趟退 78
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-two-hosts-broken.env" -- --quick
  if [[ "$exit_code" != 78 ]] || calls_have '^two-hosts-' || calls_have ' --test beta -- --nocapture ' || ! item_has beta 'exit=78 verdict=not-run ' \
     || ! out_has '判法 layer0-shard-configuration-check.sh 判不过'; then
    cell_failed fast-tier-two-hosts-configuration-refused "开了双机而判法判不过（多机配置缺键），经给定状态引擎的测试目标应当一趟都不起、记本次未跑、整趟退 78" \
      "看 decide_fast_tier_two_hosts 里判法退别的码那一支"
  fi
  cp -- "$work/stage-inputs-plain.tsv" "$root/.claude/gate.d/stage-inputs.tsv"
  # light-within-heavy：真合表与小仓两份上，轻型（全部项）的每一件在重型·剪枝（全部项）里都有
  cells=$((cells + 1))
  local within_real within_sample
  within_real="$(repository_root="$(cd "$replay_script_directory/../.." && pwd)"; load_items >/dev/null; replay_light_within_heavy)"
  within_sample="$(repository_root="$root"; load_items >/dev/null; replay_light_within_heavy)"
  exit_code=-
  if [[ "$within_real" != within || "$within_sample" != within ]]; then
    : >"$calls"
    cell_failed light-within-heavy "轻型的每一件在重型里都应当有（真合表：$within_real；小仓：$within_sample）" "看 replay_plan：重型要罩住轻型的每一件"
  fi

  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  if crash_amplification_switch_wheel_inner; then echo "  ✓ --selftest（开关轮里起的一份，不再起开关轮）：$cells 格都对"; exit 0; fi

  # 开关轮（文件头「弄坏开关」）：从文件头现读开关与点名的格，与代码里 replay_break_is 认的开关比齐，逐个打开再跑一遍 --selftest
  local table assignment named in_code in_table="" round_failures=0
  switch_wheel_assignments=(); switch_wheel_expected=()
  table="$(crash_amplification_read_switch_table "$replay_script" CRASH_POINT_REPLAY_BREAK)"
  while IFS=$'\t' read -r assignment named; do
    [[ -n "$assignment" ]] || continue
    switch_wheel_assignments+=("$assignment"); switch_wheel_expected+=("$named"); in_table+="${assignment#*=}"$'\n'
  done <<<"$table"
  in_code="$(grep -oE 'replay_break_is [a-z0-9-]+' "$replay_script" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table="$(printf '%s' "$in_table" | LC_ALL=C sort -u)"
  if (( ${#switch_wheel_assignments[@]} == 0 )) || [[ "$in_code" != "$in_table" ]]; then
    echo "  ✗ 开关轮：文件头表里的开关与代码里 replay_break_is 认的不一样（只在代码里：$(LC_ALL=C comm -23 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table") | tr '\n' ' ')；只在表里：$(LC_ALL=C comm -13 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table") | tr '\n' ' ')）" >&2
    echo "     → 怎么办：代码里每一处 replay_break_is 认的开关都在表里写一行（连同点名的格），表里每一行都在代码里有对应的一处" >&2
    round_failures=$((round_failures + 1))
  fi
  crash_amplification_switch_wheel "$work" '✗ 自证 ([a-z0-9-]+)：' 0 -- bash "$replay_script" --selftest
  round_failures=$((round_failures + switch_wheel_failures))
  if (( round_failures > 0 )); then
    echo "  ✗ --selftest：$cells 格都对，开关轮有 $round_failures 处对不上" >&2
    echo "     → 怎么办：红的格与表里点名的不同，要么改那一处判法让开关红在点名的格，要么照实改文件头那一行；单看一个开关：CRASH_POINT_REPLAY_BREAK=<开关> $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：列项不起编译、项名写错与缺项名与 --quick --full 退 2、轻型照甲二与展开 6 跑全部项（放量带新建的空库、库目录本身留给放量入口建、跑完删）、--quick --item 照跑核心部分、"
  echo "    开了 GPU 快档带 gpu 特性经清场包装起并交清场之后的显卡先后（不送走 GPU 的计算类）、给了 --cross-check-skipped-by-the-user 每一趟带对照跳过的开关而没给的清掉、"
  echo "    每一趟带按段登记文件的绝对路径而它核不过退 2 不起、快档红退 1、跑到 0 条与没卡记本次未跑、全是本次未跑退 77、重型剪枝每项交层 0 --full --item 与放量全量、"
  echo "    逐条只跑点名的项、全量删全绿标记带 --start-over 换新库（库目录同样留给放量入口建）、层 0 红与放量红各红在那一项、放量收不回判红、平常跑带只供自证的变量退 2、轻型每一件在重型里都有（真合表与小仓），
    快档日志里有给定状态表借不到显存那一行记本次未跑退 78、套在内存包装里把外层的上限给放量入口与双机驱动、开了双机时经给定状态引擎的测试目标照双机驱动三步跑
    （本机 0/2、第二台 1/2 取回库、本机带 IMPORT 指到取回的库再跑，双机驱动是真的、两趟是假的）而双机关着照旧本机一趟、第二台连不上与判法判不过记本次未跑退 78，"
  echo "    $cells 格都对（cargo、两套驱动与准入模块都是假的，不编不跑）；开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读）逐个打开，派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的格"
  exit 0
}

if (( selftest )); then
  (( quick || full || list_items || cross_check_skipped || ${#named_items[@]} )) && usage "--selftest 不与别的参数一起给"
  run_selftest
fi

load_items
if (( list_items )); then
  (( quick || full || cross_check_skipped || ${#named_items[@]} )) && usage "--list-items 不与别的参数一起给"
  replay_break_is list-items-builds && cargo build --release -p singlefs-checker-tier >/dev/null 2>&1
  for name in "${item_names[@]}"; do
    light_text="甲二 ${item_light_targets[$name]}"
    [[ -n "${item_pipeline_items[$name]}" ]] && light_text+=" ∪ 展开 $LIGHT_EXPAND_UP_TO ${item_pipeline_items[$name]}"
    heavy_text="${item_cases[$name]:+层 0 ${item_cases[$name]}}"
    [[ -n "${item_pipeline_items[$name]}" ]] && heavy_text+="${heavy_text:+ ∪ }放量 ${item_pipeline_items[$name]}"
    printf '%s\t轻型：%s；重型：快档 %s ∪ %s\n' "$name" "$light_text" "${item_light_targets[$name]}" "$heavy_text"
  done
  exit 0
fi

# 核点名的项
unknown_items=()
for name in ${named_items[@]+"${named_items[@]}"}; do
  [[ -n "${item_cases[$name]+set}" ]] || unknown_items+=("$name")
done
if (( ${#unknown_items[@]} )) && ! replay_break_is accept-unknown-item; then
  echo "  ✗ 合表里没有这些项：${unknown_items[*]}" >&2
  echo "  → 怎么办：项名照下面这张表写（--list-items 打的同一份），给几次 --item 取并集：" >&2
  printf '       %s\n' "${item_names[@]}" >&2
  exit 2
fi
selected_items=()
for name in ${named_items[@]+"${named_items[@]}"}; do
  [[ -n "${item_cases[$name]+set}" && " ${selected_items[*]-} " != *" $name "* ]] && selected_items+=("$name")
done

# 定档
(( quick && full )) && usage "--quick 与 --full 不一起给"
with_groups=0
if (( quick )); then
  tier=quick; tier_title="轻型（核心部分：全部项在甲二 ∪ 展开 $LIGHT_EXPAND_UP_TO 上，连同两档都跑的那几类）"; with_groups=1
  (( ${#named_items[@]} )) && echo "  · 点名的项已在核心部分里（核心部分每次都跑全部项）：${named_items[*]}"
  selected_items=("${item_names[@]}")
elif (( full )); then
  tier=full; tier_title="重型·全量（清空复用）"
  if (( ${#selected_items[@]} )); then tier_title+="点名的 ${#selected_items[@]} 项"; else selected_items=("${item_names[@]}"); with_groups=1; fi
elif (( ${#named_items[@]} )); then
  tier=named; tier_title="重型·逐条（点名的 ${#selected_items[@]} 项）"
else
  tier=pruned; tier_title="重型·剪枝"; selected_items=("${item_names[@]}"); with_groups=1
fi

# 往引擎送的两个变量（文件头「往引擎送的两个变量」）：按段登记文件先核、再送绝对路径；对照跳过的开关只在给了选项的那一趟送，没给先清掉
segments_registry="$repository_root/.claude/gate.d/crash-point-replay-segments.tsv"   # 路径同 lib_crash_point_replay_items.py 的 SEGMENTS_TABLE
if ! replay_break_is segments-registry-unchecked; then
  if ! segments_listing="$(python3 "$replay_script_directory/lib_crash_point_replay_items.py" segments "$repository_root")"; then
    printf '%s\n' "$segments_listing" >&2
    echo "  ✗ 按段登记文件 $segments_registry 核不过（lib_crash_point_replay_items.py segments），什么都不跑" >&2
    echo "  → 怎么办：照上面逐处列的改 .claude/gate.d/crash-point-replay-segments.tsv（写法在它的表头），再跑" >&2
    exit 2
  fi
fi
if replay_break_is segments-registry-not-sent; then
  unset SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY
else
  export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY="$segments_registry"
fi
if (( cross_check_skipped )); then
  if replay_break_is cross-check-skip-not-sent; then unset SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER
  else export SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER=1; fi
  echo "  · 对照：用户授权跳过（--cross-check-skipped-by-the-user，这一趟往引擎送 SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER=1）"
elif ! replay_break_is cross-check-skip-inherited; then
  unset SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER
fi

# GPU 与配置（文件头「GPU」）
configuration_file="$(crash_amplification_multi_host_configuration_file "$repository_root")"
gpu_switch=0
[[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU)" == 1 ]] && gpu_switch=1
features="verdict-store"
gpu_cards_for_fast_tier=""
if (( gpu_switch )); then
  features="verdict-store,gpu"
  gpu_cards_for_fast_tier="$(crash_amplification_configuration_value "$configuration_file" GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED)"
  [[ -n "$gpu_cards_for_fast_tier" ]] || gpu_cards_for_fast_tier="$(crash_amplification_configuration_value "$configuration_file" GPU_CARDS_BY_PRIORITY)"
fi
# 外层内存包装给的上限（文件头「内存包装」）：写法不认得的当没给
outer_memory_cap="${RUN_WITH_MEMORY_CAP_OUTER_CAP:-}"
[[ "$outer_memory_cap" =~ ^[1-9][0-9]*[KMGT]$ ]] || outer_memory_cap=""
replay_break_is memory-cap-not-forwarded && outer_memory_cap=""
collect_given_states_targets
pipeline_home="${SINGLEFS_CRASH_AMPLIFICATION_HOME:-$HOME/.local/share/singlefs/crash-amplification}"
run_stamp="$(date -u +%Y%m%dT%H%M%SZ)-$$"
run_directory="$pipeline_home/runs/crash-point-replay-$run_stamp"
if ! mkdir -p -- "$run_directory"; then
  echo "  ✗ 建不了这一趟的日志目录 $run_directory" >&2
  echo "  → 怎么办：看 $pipeline_home 可不可写、盘满没满，修好再跑" >&2
  exit 78
fi
part_records="$run_directory/parts.tsv"; : >"$part_records"
plan_file="$run_directory/plan.tsv"
replay_plan "$tier" "$with_groups" ${selected_items[@]+"${selected_items[@]}"} >"$plan_file"
echo "── 崩溃点重放$tier_title：${#selected_items[@]} 项（多机配置 $configuration_file：GPU $( (( gpu_switch )) && echo 开 || echo 关)），日志 $run_directory/"

# 计划里直接记本次未跑的（两档都跑的那几类里没有测试目标的）
while IFS=$'\t' read -r plan_item plan_kind plan_object; do
  [[ "$plan_kind" == not-run ]] && record_part "$plan_item" not-run - - not-run 0 0 none 0 "$plan_object"
done <"$plan_file"
run_fast_tier "$plan_file"
if [[ "$tier" != quick ]]; then
  for name in ${selected_items[@]+"${selected_items[@]}"}; do
    [[ -n "${item_cases[$name]}" ]] || continue
    # shellcheck disable=SC2086  # 键是空格隔开的一串，逐个当参数
    run_enumeration_part "$name" ${item_cases[$name]}
  done
fi
run_pipeline_part "$plan_file"
summarize "$plan_file" "$tier_title"
exit "$(cat "$run_directory/exit" 2>/dev/null || echo 1)"
