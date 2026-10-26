#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上跑点名的档；哪些判定复用由背后两套驱动各自的判定库按块定，不在这里
# run-condition: command bash cargo python3
# run-condition: check source "$PREFLIGHT_SCRIPT_DIRECTORY/crash-amplification-lib.sh" && crash_amplification_run_condition_holds "$PREFLIGHT_SCRIPT_DIRECTORY/../.." :: 修好设备，或改配置：多机配置 multi-host.env 开了 GPU 而一份卡都没写、点名的卡与双机开关对不上；单跑 bash -c 'source scripts/crash/crash-amplification-lib.sh && crash_amplification_run_condition_holds' 看是哪一处
# 长活声明（判法 .claude/hooks/lib_long_run.py：scripts/hosts/detached-run.sh start --wall-clock over-2h 与 long-run-check 读这几行）：
# long-run-shards: 按合表 .claude/gate.d/crash-point-replay-items.tsv 的项逐项跑；每项的崩溃点重放部分交门禁 checker-tier-crash-point-replay --full（登记了 shard=across-machines 的用例按多机配置两台各判一份再导入），放量部分交 scripts/crash/crash-amplification.sh（双机开着时每一项的状态块按两台量到的速度分两份）
# long-run-resume: 重起同一条命令就是续跑：崩溃点重放部分与放量部分判过的块都照同一份判定库（${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/crash-state-verdicts）按块键复用（键只由内容与判法单元指纹定）
# long-run-unshardable: 多机：这一份不自己分台，台数与额度全由背后两套驱动照多机配置的双机开关取（至多本机与 PEER_SSH_HOST 两台，不读 crash-point-replay-shard-configuration-check.sh --emit-hosts 的清单、不调 host-resources.py 现算），快档那一段只在本机跑——经给定状态引擎的测试目标除外：配了双机时交双机驱动 crash-amplification-two-hosts.sh 两台各判一份（文件头「双机」）
# 崩溃点重放的入口（判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J2「一张表、一个入口、一道门禁」与 J9「两档」；
# 用户 2026-10-10 定的两档见 .claude/rules/verification.md「两档：轻型与重型」）。登记表是合表 .claude/gate.d/crash-point-replay-items.tsv（写法在它的表头，
# 一项一条流），读法与核法在 scripts/crash/lib_crash_point_replay_items.py。这一批背后照调今天的两套驱动：崩溃点重放那一套（门禁 checker-tier-crash-point-replay
# 的 --full：逐条用例照块判定库复用）与放量分项驱动（scripts/crash/crash-amplification.sh）；引擎合并之后由引擎那一路换成一套。
#
#   crash-point-replay.sh                       重型·剪枝：全部项，崩溃点重放部分照块判定库复用、放量部分照判定库复用，加上快档那一段与两档都跑的那几类
#   crash-point-replay.sh --item <项> […]       重型·逐条：只跑点名的项（给几次取并集），快档那一段只跑这几项的测试目标，两档都跑的那几类不跑
#   crash-point-replay.sh --full [--item …]     重型·全量（只写不读）：同上，崩溃点重放部分带 --start-over、放量部分带 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1
#                                               （两部分都是库不读、全部块现判、照键写回同一份库；不另建库）
#   crash-point-replay.sh --quick [--item …]    轻型：核心部分，每次都跑全部项（见「两个域的写法」）；点名的项已在核心部分里，--item 只核项名
#   crash-point-replay.sh --list-items          逐行打「项名<TAB>轻型：…；重型：…」，只读合表，不编、不跑
#   crash-point-replay.sh --list-items --with-keys
#                                               同上，末尾再打几行「键<TAB>…」：崩溃判定库里录下的块有几块键对不上（只在别的判法单元指纹下判过、
#                                               一种判器的结论都没有）、几块键相同，与两种判器这一份的判法单元指纹。起只读读库程序
#                                               （cargo run … --bin crash_library_reader -- keys：只读开库、不写），它的输出写法在
#                                               crates/singlefs-checker-tier/src/bin/crash_library_reader.rs 文件头「输出」那一段，自证 list-items-with-keys 格
#                                               把那一段原样喂进来钉读法；读库程序退非 0 或输出认不出照实报、退 1
#   crash-point-replay.sh --list-items --backlog --milestone <mN> [--backlog-states-at-least <态数>] [--states-at-least <态数>]
#                                               只读·积压（下面「积压跑法」）：同上，末尾再打几行「积压<TAB>…」：积压登记表在今天的判法键下的游标、这一里程碑新增
#                                               与判法键变了的先跑几件、到积压下限与到合计下限各要跑到哪一行。不改表，不起快档、门禁与放量入口；只编、只起表的小程序
#   crash-point-replay.sh --backlog --milestone <mN> [--backlog-states-at-least <态数>] [--states-at-least <态数>] [--local-only]
#                                               重型·积压（下面「积压跑法」）：照积压登记表取件起跑，先这一里程碑新增与判法键变了的，再从游标起照表的次序取到两条下限；
#                                               取到的件崩溃点重放部分交门禁 checker-tier-crash-point-replay --full --item <crash-case: 键>、放量部分交 crash-amplification.sh --item <分项>，
#                                               判过的块照块判定库复用（不带 --start-over、不设只写不读），每跑完一件经表的小程序 record 回写那一行
#   crash-point-replay.sh --local-only [--quick | --full] [--item …]
#                                               只用本机（与上面除 --list-items、--selftest 之外的任一种跑法合用；与那两个一起给退 2）：第二台一概不动、不连。开跑时把多机配置
#                                               派生一份临时的（ENABLE_ACROSS_MACHINES 写成 0，去掉 PEER_ 开头与 HOST_JUDGE_WEIGHTS 两类键、显卡先后里另一台的项；这一台的 GPU 与清场照旧；
#                                               哪几项是这一台的照 SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE 取：没设或 local 留 local: 去 peer:，peer（在第二台上跑）留 peer: 去 local:），
#                                               导出 SINGLEFS_MULTI_HOST_CONFIG 指到它，快档、门禁的崩溃点重放部分与放量入口都读这一份；原来那份不改，跑完删派生的。
#                                               文件头两条 run-condition 在参数解析之前判，读的仍是原来那份（只判配置对不对得上，不连第二台）；多机配置不在时两个开关本来就关，照原样跑
#   crash-point-replay.sh --selftest            假 cargo、假的两套驱动、假的准入模块走一遍各格，末尾一轮从下面「弄坏开关」那张表现读开关、逐个打开
#                                               （设了 CRASH_POINT_REPLAY_SELFTEST_ONLY=backlog 的：只走积压跑法那几格（backlog-* 与 list-items-backlog）就停，别的格与开关轮不走，
#                                               单看那几格用；这个变量只认 backlog 这一个值，别的值退 2）
#   引擎配了 GPU 只由 GPU 判、没配只由 CPU 判，同一批状态一趟只判一次；GPU 判与 CPU 判的对拍是另立的测试，不在这里跑（用户 2026-10-11 定）。
#   不带 --quick 的跑法是重型，只由带 SINGLEFS_HEAVY_TESTS 前缀的主 agent 跑（.claude/hooks/heavy-test-guard.sh 判）；轻型经
#   scripts/hosts/run-with-memory-cap.sh 起，谁都能跑。
#
# 两个域的写法（.claude/rules/verification.md「两档：轻型与重型」点名这里）：
#   「甲二」 = 每项第四列的测试目标不带 --ignored 跑到的那些用例（checker 档快档：按甲二展开的小域，crash::CrashPointReplaySegmentExpansion）；
#   「展开 6」 = 每项第三列的放量分项交 crash-amplification.sh --expand-up-to LIGHT_EXPAND_UP_TO（第一行代码之后那个赋值，今天 6），不带 --library：
#              用放量入口的默认长期库（<放量的家目录>/crash-state-verdicts，与重型的放量部分同一份；.claude/rules/verification.md「checker 档自己分快档与全量」
#              崩溃放量那一行），键相同、判过的块照库复用，被打断之后重跑同一条命令就是续跑（test-script-discipline.md「判定可复用：按输入存，按前缀剪」）；
#              ITEM 行与成功句照放量入口的 CRASH_AMPLIFICATION_ITEM 行报现判几态、复用几态。
#              按段登记里甲二装不进 u64 的段（驱动行 registered_segment_domains=<段号>:slice）引擎分片枚举、每趟判 SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES 个态
#              （没设取 2^32）：起放量入口那一趟，调用方没设这个变量就设成 LIGHT_REGISTERED_SEGMENT_SLICE_STATES（LIGHT_EXPAND_UP_TO 下一行那个赋值，今天 65536，引擎的一块），
#              设了照用（空串也照原样交，引擎拒）；长期库里有续点，所以轻型对这种段每趟从库里这一判法单元指纹的续点起判一块（65536 个态；库里还没有续点的从段内序号 0 起），
#              与重型的放量部分接着同一个续点往下判。不带 --quick 的跑法不设，从调用方环境继承。
#              快档那一段不分档（每种跑法里都是冒烟那一段）：每一趟 cargo test（连同开了双机时交双机驱动的那几趟）调用方没设同样设成 LIGHT_REGISTERED_SEGMENT_SLICE_STATES、
#              设了照用；快档的测试目标里不带 --ignored 的放量用例（例如 crash_enumeration_position_addressed_trees 的 crash_amplification_of_every_position_addressed_tree_stream_is_clean）
#              对分片的段同样只判第一块，全片归重型的放量部分。
#   全量的那一份库，这里只建上层目录，库目录本身留给引擎新建（引擎见 <库>/CURRENT 不在就建空库，目录已经在就拒 LibraryDirectoryAlreadyExists）。
# 轻型 = 全部项在这两个域上判，连同两档都跑的那几类（BOTH_TIERS_GROUPS：小输入的 GPU 与 CPU 对拍、剪枝自证、钉回来的红例）。
# 重型 = 全部项的整域：快档那一段照轻型跑一遍，崩溃点重放部分（第二列的 crash-case:）交门禁 checker-tier-crash-point-replay --full --item，
#   放量部分（第三列）交 crash-amplification.sh 照它的全量展开深度（不带 --expand-up-to）；不带参数时另跑两档都跑的那几类。
#   崩溃点重放部分里登记行第三列写了 red-blocks=judged-by-test 的用例，--full 与快档同一套判法（判法在那道门禁，这里只收它的退出码）：按 cargo test 的退出码判，块那一行只核数，库里的红块不判。
#   快档（上面第一件）先于这两部分，快档红了这两部分都不起（「一项怎么判」）。
# 轻型是重型的子集：轻型（全部项）的每一件在重型·剪枝（全部项）里都有——快档的测试目标照样在、展开 6 的放量分项在重型里按全量展开深度跑
#   （展开深度大的一趟罩住小的，一段不超过展开上限就展开全部真子集）；自证 light-within-heavy 一格拿真合表与小仓两份计划逐件核。
#   按段登记里分片的段：轻型展开 6 那一支每趟一块（65536 个态，见「展开 6」），重型放量部分每趟一片（没设取 2^32 个态），两边都从同一份长期库里的续点起
#   （库里还没有续点的头一趟从段内序号 0 起），轻型判过的块重型照键复用；自证 quick-slice-states-sent 一格核这个变量在 --quick 路径上交到放量入口、不带 --quick 时不设；
#   快档那一段两档都只判第一块，fast-tier-slice-states-sent 一格核它在轻型、剪枝、全量三种跑法里都交到快档每一趟（连同双机那几趟）。
#   崩溃点重放部分的全量与快档同一条流、同一个测试目标（合表第四列取自同一项的重型部分，lib_crash_point_replay_items.py 核）。
# 两档都跑的那几类（BOTH_TIERS_GROUPS，一行「类名<TAB>测试目标，空格隔开；没有写 -」）：没有测试目标的一类报本次未跑、写明为什么。
#
# GPU（.claude/rules/test-script-discipline.md「用上现有的硬件加速」）：多机配置（crash-amplification-lib.sh 文件头「配置」：
#   ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，按 git common-dir 找）写了 ENABLE_GPU=1 时快档那一段带 --features verdict-store,gpu 编，
#   每一趟 cargo test 经 scripts/crash/gpu-quiesce-run.sh 清场再起（跑完不复原），环境里交清场之后那一份显卡先后
#   （多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED，没写取 GPU_CARDS_BY_PRIORITY）；走 GPU 的计算类不送（引擎开了 GPU 就三类全走 GPU、不读
#   GPU_COMPUTATION_CLASSES，crash-amplification-lib.sh 文件头「配置」）。
#   开了 GPU 时单机路径的每一趟快档测试参数带 `-- --nocapture --test-threads 1`（GPU 对拍的测试各占卡的显存，不并行起）；没开的不加。
#   双机分支起的 crash-amplification-two-hosts.sh 在共用库 crash-amplification-lib.sh 拼好的 cargo test 参数后，开了 GPU 时另加 `--test-threads 1`；准入模块 admission.py 拼的 crash-case 命令里特性带 gpu 的也带。
#   还没带的：崩溃放量单机那几趟（crash-amplification.sh 经共用库 crash-amplification-lib.sh 拼的 cargo test 参数，里面没有这个参数）；改成带时同时改 .claude/rules/verification.md「两档：轻型与重型」里 GPU 测试那一条。
#   这个参数只管同时起几个测试，不管一个测试里 CPU 判器自己开的线程：GPU 对拍的测试读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取本机可用并行度（available_parallelism，
#   不经 capped.sh default 的登记值；改成取登记值归代码批）。
#   没开只带 --features verdict-store，挂 gpu 特性的对拍编进来 0 条、报本次未跑。放量部分的 GPU 与清场由 crash-amplification.sh 自己照同一份配置办。
#   崩溃点重放部分（重型）里经引擎的 crash-case 行（.claude/gate.d/stage-inputs.tsv 里登记了 features= 的，今天是随机历史两项）由准入模块 admission.py 的 crash-case-command
#   带 --features 起、开 GPU 时同样加 gpu 并整条经清场包装起，不在这里拼（写法见 admission.py 文件头「崩溃枚举用例」）。
#   快档那几趟 cargo 清掉调用方环境里的 SINGLEFS_CRASH_AMPLIFICATION_*、GPU_COMPUTATION_CLASSES、SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE、退役的 SINGLEFS_GPU_CARDS_CONFIG（同门禁
#   checker-tier-crash-point-replay 的 feature-gated-tests 格：快档不写真的判定库、不按全量的规模跑），多机配置在就交 SINGLEFS_MULTI_HOST_CONFIG=<它的绝对路径>。
#
# 双机（E4 报告第三节第 2 条：给定状态引擎读 SINGLEFS_CRASH_AMPLIFICATION_SHARE、…_HOST_WEIGHTS、…_LIBRARY、…_IMPORT，与放量流水线同一种读法）：
#   「经给定状态引擎的项」= 合表第二列点名的 crash-case 键在 .claude/gate.d/stage-inputs.tsv 里登记了 features= 的项（要经引擎、进判定库的才登记；今天是
#   crash-injection-fast-tier 与 read-faulted-states-never-hold-the-witness-criterion），它们第四列的测试目标在快档里这样跑：
#   多机配置写了 ENABLE_ACROSS_MACHINES=1 时先经 scripts/crash/crash-point-replay-shard-configuration-check.sh 判（双机开没开、键齐不齐、第二台连不连得上）：
#     退 0（双机开）：每个这样的测试目标一趟 scripts/crash/crash-amplification-two-hosts.sh <测试目标> --library <新建的空库> --label <标签> --given-states（本机与第二台都两段先后跑，驱动不另起录入段），照它的三步跑——
#       本机 SHARE=0/2、第二台 SHARE=1/2 经 multi-host-run.sh 跑完取回库、本机带 SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<取回的库> 再跑一趟；
#       库用那一份长期库 <放量的家目录>/crash-state-verdicts（CRASH_LIBRARY_DIRECTORY_NAME；单机那一趟用例照默认落的也是它，
#       crates/singlefs-checker-tier/src/crash_library.rs 的 CrashLibraryClass::GivenStates），跨趟留着、键相同的表照库复用，被打断之后重跑同一条命令就是续跑；
#       取回的那一份与送第二台的快照（<放量的家目录>/runs/<标签>/fetched、library-for-peer、library-for-peer-placement）双机驱动退出时自己删（它文件头「跑完删库的拷贝」），
#       这里跑完只再删一次 fetched 与 library-for-peer 兜底（两边都删是幂等的）；长期库不删，双机驱动的日志留在 <放量的家目录>/runs/<标签>/。
#       开了 GPU 整条经 gpu-quiesce-run.sh 清场再起、带 --quiesced。
#       判法：双机驱动退 0 照导入那一趟（import.log）的 libtest 结果判（同上「一项怎么判」）；退 78（第二台连不上、库没取回、卡用不上、给定状态表借不到显存）
#       与内存包装的结局 245、251、252、254 记 not-run、退 78；退 77 记 not-run；别的照导入那一趟的结果记红。
#     退 3（双机关）：照旧本机一趟；别的退出码（判不过，含第二台连不上）：这几个测试目标一趟都不起，记 not-run、退 78、原因写判法那一句。
#   没写 ENABLE_ACROSS_MACHINES=1 的（配置不在、没写、写 0）不调判法，照旧本机一趟。别的测试目标一律本机一趟。
# 内存包装：套在 scripts/hosts/run-with-memory-cap.sh 里跑时（轻型照规则这样起），包装给命令设了 RUN_WITH_MEMORY_CAP_OUTER_CAP=<外层的上限>
#   （run-with-memory-cap.sh 文件头「套在包装里」）；这一趟把它照原样给放量入口与双机驱动的 --memory-cap：里层上限比外层大，包装按用法错退 251。没设不给（各用各的默认）。
#
# 往引擎送的变量（快档、崩溃点重放部分与放量部分都从这一趟的环境继承；双机时第二台那几趟只看双机驱动送过去的环境，两个变量它都照本机的值送）：
#   SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=<仓根>/.claude/gate.d/crash-point-replay-segments.tsv（按段登记文件的绝对路径，判决 J10、M4 第 15 条）：
#     每一趟都送；开跑前先经 scripts/crash/lib_crash_point_replay_items.py segments 核它（核不过退 2、什么都不起；写法与核法在那份库文件头「按段登记」）。
#   SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=<分片的段一趟判几个态>：调用方没设时，快档那一段（不分档，连同交双机驱动的那几趟）与 --quick 起放量入口那一趟
#     设成 LIGHT_REGISTERED_SEGMENT_SLICE_STATES（文件头「展开 6」），设了照原值交（空串也照原样，引擎拒）；不带 --quick 的放量部分与崩溃点重放部分不设，从调用方环境继承。
#     快档组环境时 -u 掉的只有 SINGLEFS_CRASH_AMPLIFICATION_* 与几个 GPU 变量，前缀不同，清不到它。双机时第二台那几趟（快档交双机驱动的、放量部分动用第二台的）
#     由双机驱动照本机的值送进第二台排计划、分份那两趟（crash-amplification-two-hosts.sh 文件头「往引擎送的变量」），本机没设就不送、第二台照引擎的默认（2^32）判。
#
# 改轻型规模的变量（.claude/hooks/lib_heavy_tests.py 的 light_tier_break_variable 现读这张表，那边不抄名单）：调用方在命令前、env 里或前面的 export 里设了其中一个（值非空），
#   带 --quick 也不认轻型，照重型拒，--quick 的结果不作轻型证据。一行「#   LIGHT_SCALE_VARIABLE=<变量名>  <改什么>」，新增一个变量在这里加一行：
#   LIGHT_SCALE_VARIABLE=SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES  分片的段一趟判几个态（轻型调用方没设取 LIGHT_REGISTERED_SEGMENT_SLICE_STATES，引擎没设取 2^32）
#
# 输出：背后每一趟的输出边跑边转打，同时进这一趟的日志目录 <放量的家目录>/runs/crash-point-replay-<时戳>-<进程号>/（跑完不删）。每项（与两档都跑的每一类）
#   打一行 CRASH_POINT_REPLAY_ITEM item=<项> tier=<quick|pruned|named|full> exit=<0|1|77|78> verdict=<green|red|not-run>
#     fast_tier_tests=<快档跑到的用例数> fast_tier_red=<红的用例数> fast_tier_reused=<快档复用的块数，见「快档的复用数怎么取」；一个测试目标都取不出写 unknown>
#     fast_tier_reuse_unknown=<取不出复用数的测试目标个数>
#     pipeline_states=<放量现判的态数> pipeline_red=<红> pipeline_reused=<复用的态数，放量入口取不出写 unknown>
#     enumeration_cases=<崩溃点重放部分的用例数> enumeration_red=<红的用例数> enumeration_reused=<全部块都复用的用例数> seconds=<这一项各段的挂钟之和>
#     （这一项没有那一段的写 -）；末行 CRASH_POINT_REPLAY_TOTAL tier=… dispatched=<派出几项> collected=<收回几项> green= red= not_run=
#     fast_tier_tests= fast_tier_reused_known=<取得出的那几个测试目标复用的块数之和> fast_tier_reuse_unknown=<几个测试目标取不出>
#     pipeline_states= pipeline_reused_known= pipeline_reuse_unknown=<几项取不出> enumeration_cases= enumeration_reused= seconds=。
#   快档的复用数怎么取：走判定库的用例（崩溃枚举那一类，crates/singlefs-checker-tier/src/crash_state_enumeration.rs 文件头「行」）每判一趟打一行
#     CRASH_POINT_REPLAY_PARALLEL_FINISHED …，其中 resumed_slices= 是库里复用的块、freshly_run_slices= 是这一趟判的块（scripts/gate/admission.py 文件头
#     「崩溃枚举用例行」threads= 那一条按的也是这一行）；一个测试目标的复用数是它日志里这种行 resumed_slices= 之和（单位是块）。一行都没有、或有一行缺这个字段、
#     不是非负整数，这个测试目标记「取不出」（unknown），不并进总计、不写 0 也不写 none，成功句点名是哪几个测试目标（test-script-discipline.md「每项结束打一行」）。
#   一项怎么判：有一段红就 red；不红而有一段本次未跑（快档跑到 0 条、有 NOT_RUN_NO_GPU_ADAPTER、放量那一项 verdict=not-run、崩溃点重放那一道退 77 或 3，
#   某一段退 78）就 not-run，逐项写原因；别的 green。快档一个测试目标：cargo 退非 0 或有 FAILED 就红（编不过的整段红）；
#   日志里有行首（可带空白）是 CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 的那一行（给定状态表的判器起过卡池而借不到显存，引擎打这一行后报错、cargo 退 101；
#   行首取 crash-amplification-lib.sh 的 CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX）的，不记红：那一段记 not-run、退 78、原因写那一行，整趟没红就退 78。
#   放量部分点名的分项没打 CRASH_AMPLIFICATION_ITEM 行就红（收不回）；放量入口退 78（运行条件不满足）的除外：那一项记 not-run、退出码 78、原因写入口退 78 与日志路径，
#   整趟没有红就退 78（下面「退出码」），有红先判红、退 1；派出几项、收回几项对不上整趟红（command-safety.md「并行不许把失败吃掉」）。
#   重型三种跑法（剪枝、--item、--full）快档先跑：快档有一个测试目标红（编不过也算）就不起崩溃点重放部分与放量部分，计划里这两段的每一件各记一笔 not-run（退出码列 77，
#   原因「快档红」），标准错误先打「本次未跑：快档红」与出路，整趟退 1。轻型（--quick）没有崩溃点重放部分，放量部分是展开 6 那一趟，照旧往下跑，不受这一条管。
#   有红时出路给出只重跑红的那几项的命令（重型照档带 --item，项数就是红的项数；两档都跑的那几类不能点名；轻型每次跑全部项，给整趟重跑那一条）。
#   退 0 时打一句成功句：「崩溃点重放<档>：K 项都不红；本次未跑 J 项：<项（原因）…>；快档 N 条用例复用 M 块（另有 J 个测试目标取不出复用数：<测试目标…>）、
#   放量现判 N 态复用 M 态、崩溃枚举用例 N 条复用 M 条；派出 D 项、收回 D 项」，
#   成功句之后（红了也一样）把这一趟日志目录里的覆盖行与计数行落成两份产物，留在同一个目录（判法与写法在 scripts/crash/crash-flow-coverage.py 文件头）：
#     crash-flow-records.tsv  run-directory 转出的记录（快档那一段算 quick 域，放量部分照档，崩溃点重放部分读门禁 --full 打的 CRASH_FLOW_RECORDS_FILE 那一份）；
#                             填 .claude/gate.d/crash-ignored-nodes.tsv、invariant-zero-evaluations.tsv 的初值用 crash-flow-coverage.py baseline <ignored|zero-evaluations> 这一份
#     crash-flow-coverage.md  覆盖表（入口 × StepKind 成员，两个域各一张）、ignore 开着的节点、评估 0 次的（流，不变量）对
#   这两份只落产物、不判红、不改这一趟的退出码（判在门禁 checker-tier-crash-point-replay 的出口 4 那几格）；写不成报一行。
# 积压跑法（.claude/kb/milestone/04-fourth-txn.md 出口 10「崩溃全量：新增的跑完，另补积压」；用户 2026-10-26 定：在今天的单元键下有判定就算罩住，复用的块算跑了）：
#   积压登记表 crates/singlefs-checker-tier/crash-full-run-backlog.tsv（写法在它的表头）只经表的小程序 crash_full_run_backlog 读写：子命令与输出行在
#   crates/singlefs-checker-tier/src/bin/crash_full_run_backlog.rs 文件头，取件、游标与判法键的口径在 src/crash_full_run_backlog.rs，这里不另写一份读法。
#   小程序经 cargo run --release --quiet --offline -p singlefs-checker-tier --features verdict-store --bin crash_full_run_backlog -- <子命令> 起（经 capped.sh default；
#   判法键里有构建档与工具链：编法同门禁 checker-tier-crash-point-replay 起多数崩溃枚举用例的那一种，release、默认特性、不带 gpu），编译目录是这一趟的 CARGO_TARGET_DIR。
#   --backlog（重型）依次：
#   ① crash-amplification.sh --list-items 存成这一趟的 ITEMS 列表；append-missing 把合表加 ITEMS 展开里有、表里没有的件追加到表末尾（归属是 --milestone 那一个、态数未数）；
#     再 check 核表与展开逐件一致，核不过（小程序退非 0）退 2、不取件、什么都不起。
#   ② today-judging-key 取今天的判法键。
#   ③ plan 取件：--backlog-states-at-least、--states-at-least 给了才交，不给是小程序的默认（出口 10 的积压 50 亿、合计 100 亿）。先核 plan 的输出（完整性闸：游标那一行与末行
#     CRASH_FULL_RUN_BACKLOG_PLAN 各恰好一行、末行在最后、每件一行字段齐、序号不重、先跑的那一组（new-or-judging-key-changed）都排在积压（backlog）之前、两组各自照表的次序、
#     两组的态数之和与未数的件数等于末行报的），对不上退 1、一件都不跑；一件都没取到退 77。
#   ④ 计划：取到的件照取件的次序排（先跑的那一组在前）；它们所在的项的快档测试目标先跑一遍（快档红了两部分都不起，同重型别的跑法），再照取件的次序逐件跑：
#     crash-point-replay 那一部分交门禁 checker-tier-crash-point-replay --full --item <crash-case: 键> <根>（日志 enumeration.<项>.<用例名>.log），
#     crash-amplification 那一部分交 crash-amplification.sh --item <分项>（日志 pipeline.<分项>.log）。
#   ⑤ 每跑完一件经 record 回写那一行：判法键写今天的；最后一次写「<日期>；crash-point-replay.sh --backlog --milestone <mN> 那一趟」；绿的进度写「全绿」，
#     红的写「红：<哪一道退几>（<日期> 积压跑法；日志 <文件名>）」，本次未跑的不回写。态数：崩溃点重放部分取门禁打的那一行「全量日志：<用例日志>；发现日志：…」点名的用例日志里
#     CRASH_CASE_BLOCKS 行的 states= 之和（一行都没有就取 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行的 states= 之和），放量部分取 CRASH_AMPLIFICATION_ITEM 行的 states= 加 reused=
#     （复用的块算罩住，整件算）；数不出的不交 --states：表里那一行原来数过的照写进度，原来未数而判绿的不回写（全绿而未数是 check 判的一处问题），记 recorded=no、写明为什么。
#   每件一行 CRASH_POINT_REPLAY_BACKLOG_PART sequence=<序号> group=<new-or-judging-key-changed|backlog> item=<项> part=<部分> key=<键> verdict=<green|red|not-run>
#     recorded=<yes|no> states=<回写的态数，没交写 -> reason=<没回写的原因，回写了写 ->；之后一行 CRASH_POINT_REPLAY_BACKLOG_TOTAL milestone=<mN> taken=<取到几件>
#     new_or_judging_key_changed=<几件> backlog=<几件> recorded=<回写几件> not_recorded=<没回写几件> recorded_states=<回写的态数之和> record_failures=<record 没写成几件>；
#     record 没写成的、回写与没回写之和不等于取到的件数的，整趟退 1。
#   --list-items --backlog（只读）：today-judging-key，再三趟 plan：照交的两条下限一趟（积压跑法这一趟会取到哪些件）、只看积压下限一趟（加 --states-at-least 0）、
#     只看合计下限一趟（加 --backlog-states-at-least 0）；打「积压<TAB>…」几行：今天的判法键、游标（序号与项、部分、键，或没有）、先跑的那一组几件几态、
#     到积压下限要跑到哪一行、到合计下限要跑到哪一行、积压跑法一趟取几件。不起 append-missing、check、record，不起快档、门禁与放量入口；
#     编译目录是这一趟在 ${TMPDIR:-/tmp} 下建的（调用方设了 CARGO_TARGET_DIR 照它、不删），跑完删。起小程序之前同样先判 rocksdb-prebuilt.py entry，判红退 1。
# 退出码：
#   0   跑到的项都不红、派出收回相等（有项本次未跑也是 0，成功句逐项点名）
#   1   有红（含快档红：这时崩溃点重放部分与放量部分没起）、或派出收回对不上；积压跑法：plan 的输出对不上、取到的件不在合表里、record 没写成
#   2   用法错：参数认不得、--item 后面缺项名或项名认不得（列出可点名的项）、--quick 与 --full 一起给、--list-items 或 --selftest 夹了别的参数、
#       合表核不过、按段登记文件核不过、平常跑时带了只供自证的变量（见「只供自证」）；--backlog 没带 --milestone m<号>、与 --quick / --full / --item 一起给、
#       --milestone 或两条下限不跟 --backlog、下限不是十进制整数；积压登记表核不过（小程序 check 退非 0）
#   77  派出的项全是本次未跑，也没有红；积压跑法在今天的判法键下一件都没取到
#   78  运行条件不满足：preflight 判的（文件头两条 run-condition），或背后有一段退 78（显卡、第二台、清场用不上，给定状态表的判器借不到显存，
#       多机配置开了双机而判法判不过；放量入口退 78 而这一项没打 ITEM 行）而这一趟没有红；
#       或起任何 cargo 之前 scripts/hosts/rocksdb-prebuilt.py entry 判红（这台没装预编库、此刻进程环境的取值缺了或对不上；一项都没起，出路是开新会话
#       或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"）。--list-items --with-keys 起读库程序之前同样判，判红退 1
# 编译目录（用户 2026-10-23 定案「自己用完自己删」，不跨趟复用）：建好这一趟的日志目录之后，调用方没设 CARGO_TARGET_DIR 时在里面建 target、export 给快档、
#   崩溃点重放那一道、放量入口与双机驱动（它们看到设了就不另建）；正常结束、红了、被 TERM / INT 停（等手里那一步退出）都在 EXIT 上删
#   （scripts/hosts/run-build-directory.sh；删之前核本机没有进程开着里面的文件，有就不删、报出进程、整趟退非 0）。调用方设了的照它、不删。
# 日志出两份（用户 2026-10-21：过万行的全量日志太长，一般只看错误警告）：快档、崩溃点重放部分、放量部分、双机驱动各自的日志都经 scripts/hosts/two-logs.sh 写，
#   <日志目录>/<段>.log 是全量，旁边同名的 <段>.problems.log 只收错误与警告（门禁的 ✗ 与 → 出路、⚠、本次未跑、汇总，cargo / rustc 的 error 与 warning:，libtest 的 FAILED 与 panicked，
#   Traceback，非 0 退出码），每行前带它在全量日志里的行号，末行写收了几行、全量几行；开跑时打日志目录，收尾逐份列出 .problems.log，红的原因里点名这一份。
# 弄坏开关 CRASH_POINT_REPLAY_BREAK=<项>（几个用逗号连；只在自证起的子进程里认）。写法同 scripts/crash/crash-amplification.sh 文件头「弄坏开关」一段：
#   一行「#   CRASH_POINT_REPLAY_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（、隔开；括号里是失败行的句首，开关轮不读）。
#   --selftest 末尾那一轮（开关轮，crash-amplification-lib.sh 的 crash_amplification_switch_wheel）从这张表现读，表里的开关与代码里 replay_break_is 认的要相等：
#   CRASH_POINT_REPLAY_BREAK=accept-unknown-item  认不得的项名悄悄丢掉、照跑
#       unknown-item（「项名写错应当退 2、列出可点名的项、什么都不起」）
#   CRASH_POINT_REPLAY_BREAK=list-items-builds  --list-items 先起一趟 cargo build 再列
#       list-items（「--list-items 应当退 0、逐项列出、不起 cargo 与驱动」）、list-items-with-keys（「--list-items --with-keys 应当照读库程序文件头「输出」那一段的写法报键对不上的块数、不起 cargo」）、list-items-backlog（「--list-items --backlog 应当退 0、照小程序 plan 报游标与到两条下限各要跑到哪一行、只起表的小程序、不改表」）
#   CRASH_POINT_REPLAY_BREAK=quick-throwaway-library  轻型的放量部分带一份这一趟新建、跑完删掉的空判定库（改前的样子：不复用、打断之后没有续点）
#       quick-runs-the-light-domains（「轻型应当照甲二与展开 6 跑全部项」）、quick-reuses-the-long-term-library（「轻型那一趟第二次跑应当复用第一次判过的块」）、
#       reuse-three-questions（「复用三问没过」：每一趟一份空库，第二趟照样全现判）
#   CRASH_POINT_REPLAY_BREAK=fast-tier-reuse-not-counted  快档每个测试目标的复用数写死 none（改前的样子）
#       fast-tier-reuse-counted（「快档日志里有复用计数行时 ITEM 行应当报同一个数……」）
#   CRASH_POINT_REPLAY_BREAK=fast-tier-two-hosts-throwaway-library  双机开着时经给定状态引擎的测试目标交一份这一趟新建、跑完删掉的空库（改前的样子）
#       fast-tier-two-hosts（「开了双机，经给定状态引擎的测试目标应当照双机驱动的三步跑……」：交的库不是给定状态那一类的长期库、跑完没留着）
#   CRASH_POINT_REPLAY_BREAK=quick-without-both-tiers  轻型不跑两档都跑的那几类
#       quick-runs-the-light-domains（「轻型应当照甲二与展开 6 跑全部项」）、quick-gpu（「开了 GPU 快档应当带 gpu 特性、经清场包装起」：少了那几个测试目标）、
#       fast-tier-not-run（「快档跑到 0 条或没卡没跑的，应当记本次未跑」：钉表那一类没了，本次未跑少一项）、
#       local-only-keeps-gpu-drops-peer-cards（「✗ 自证 local-only-keeps-gpu-drops-peer-cards：」：--local-only 下的 GPU 那一格同样数快档的测试目标，少了两档都跑的那几类就数不对）、local-only-on-peer-keeps-own-cards（「✗ 自证 local-only-on-peer-keeps-own-cards：」：角色 peer 那一格同上）
#   CRASH_POINT_REPLAY_BREAK=heavy-without-fast-tier  重型的计划里没有快档那一段
#       light-within-heavy（「轻型的每一件在重型里都应当有」）、pruned-heavy（「不带参数的重型应当照剪枝跑全部项」）、named-heavy（「--item 应当只跑点名的项」）、
#       fast-tier-slice-states-sent（「✗ 自证 fast-tier-slice-states-sent：」：剪枝与 --full 两种跑法一趟快档都没跑）、
#       fast-tier-red-stops-heavy（「✗ 自证 fast-tier-red-stops-heavy：」：重型没有快档那一段，快档红不出来）、local-only（「✗ 自证 local-only：」：--item 那一趟找不到点名的快档测试目标）
#   CRASH_POINT_REPLAY_BREAK=gpu-without-quiesce  开了 GPU 时快档那几趟不经清场包装起
#       quick-gpu（「开了 GPU 快档应当带 gpu 特性、经清场包装起」）、
#       local-only-keeps-gpu-drops-peer-cards（「✗ 自证 local-only-keeps-gpu-drops-peer-cards：」：--local-only 下的 GPU 那一格同样要快档经清场包装起）、local-only-on-peer-keeps-own-cards（「✗ 自证 local-only-on-peer-keeps-own-cards：」：角色 peer 那一格同上）
#   CRASH_POINT_REPLAY_BREAK=gpu-cards-not-sent  开了 GPU 时快档那几趟不交显卡先后
#       quick-gpu（「开了 GPU 快档应当带 gpu 特性、经清场包装起」）、
#       local-only-keeps-gpu-drops-peer-cards（「✗ 自证 local-only-keeps-gpu-drops-peer-cards：」：--local-only 下的 GPU 那一格同样要交显卡先后）、local-only-on-peer-keeps-own-cards（「✗ 自证 local-only-on-peer-keeps-own-cards：」：角色 peer 那一格同上）
#   CRASH_POINT_REPLAY_BREAK=gpu-test-threads-unset  开了 GPU 时快档那几趟测试参数不带 --test-threads 1（改前的样子）
#       quick-gpu（「开了 GPU 快档应当带 gpu 特性、经清场包装起、测试参数带 --test-threads 1」：调用记录里 gpu 那几趟（三项的测试目标加 BOTH_TIERS_GROUPS 里的测试目标，个数自证现数）的 -- --nocapture 后面不是 --test-threads 1；没开 GPU 的 quick-runs-the-light-domains 那一格要求 -- --nocapture 后面直接是 cards=，这个开关不动它）、
#       local-only-keeps-gpu-drops-peer-cards（「✗ 自证 local-only-keeps-gpu-drops-peer-cards：」：--local-only 下的 GPU 那一格同样要带 --test-threads 1）、local-only-on-peer-keeps-own-cards（「✗ 自证 local-only-on-peer-keeps-own-cards：」：角色 peer 那一格同上）
#   CRASH_POINT_REPLAY_BREAK=red-as-green  快档 cargo 退非 0、有 FAILED 也记绿
#       fast-tier-red（「快档一个测试目标红了，那一项应当红、整趟退 1」）、
#       fast-tier-red-stops-heavy（「✗ 自证 fast-tier-red-stops-heavy：」：快档红记成绿，重型就不会停）、
#       run-target-red（「跑完运行目录里不剩它建的编译目录（快档红退 1）」：快档红记成绿、整趟退 0）
#   CRASH_POINT_REPLAY_BREAK=not-run-as-green  快档跑到 0 条、有 NOT_RUN_NO_GPU_ADAPTER 也记绿
#       fast-tier-not-run（「快档跑到 0 条或没卡没跑的，应当记本次未跑」）、all-not-run（「派出的项全是本次未跑，应当退 77」）
#   CRASH_POINT_REPLAY_BREAK=skip-collection-count  放量部分收不回的分项不记红、派出收回不核
#       collection-count（「放量那一项收不回，应当红、派出收回对不上」）、
#       pipeline-entry-1-still-red（「✗ 自证 pipeline-entry-1-still-red：」）、pipeline-entry-78-not-run（「✗ 自证 pipeline-entry-78-not-run：」）、pipeline-entry-78-red-first（「✗ 自证 pipeline-entry-78-red-first：」：收不回的项一笔都不记）
#   CRASH_POINT_REPLAY_BREAK=full-keeps-reuse  全量不带 --start-over、放量部分不设只写不读（SINGLEFS_CRASH_LIBRARY_WRITE_ONLY）
#       full-heavy（「全量应当只写不读」）、full-writes-back-into-the-one-library（「✗ 自证 full-writes-back-into-the-one-library：」：--full 那一支没设只写不读）
#   CRASH_POINT_REPLAY_BREAK=enumeration-exit-ignored  崩溃点重放那一道的退出码不看，一律记绿
#       enumeration-red（「崩溃点重放那一道红了，那一项应当红」）、backlog-red-part-recorded-red（「积压跑法里红的那一件应当回写成红、整趟退 1」：门禁退 1 当绿，那一件回写成全绿）
#   CRASH_POINT_REPLAY_BREAK=pipeline-red-ignored  放量那一项的 red= 与 verdict=red 不看
#       pipeline-red（「放量那一项红了，那一项应当红」）、last-run-list（「上次红的清单不对」：放量红的那一项没记成红）、
#       pipeline-entry-78-red-first（「✗ 自证 pipeline-entry-78-red-first：」：放量红的 alpha 记成绿）
#   CRASH_POINT_REPLAY_BREAK=honor-test-variables-outside-selftest  平常跑时也认只供自证的变量，不拒
#       test-variables-outside-selftest（「平常跑时带了只供自证的变量应当退 2」）
#   CRASH_POINT_REPLAY_BREAK=fast-tier-classes-sent  开了 GPU 时快档那几趟照改前送三类 GPU_COMPUTATION_CLASSES
#       quick-gpu（「开了 GPU 快档应当带 gpu 特性、经清场包装起」：调用记录里 classes= 不是 [unset]）、
#       local-only-keeps-gpu-drops-peer-cards（「✗ 自证 local-only-keeps-gpu-drops-peer-cards：」：--local-only 下的 GPU 那一格同样要求 classes=[unset]）、local-only-on-peer-keeps-own-cards（「✗ 自证 local-only-on-peer-keeps-own-cards：」：角色 peer 那一格同上）
#   CRASH_POINT_REPLAY_BREAK=segments-registry-not-sent  不往引擎送按段登记文件的路径
#       segments-registry-sent（「快档与放量每一趟都应当带按段登记文件的绝对路径」）
#   CRASH_POINT_REPLAY_BREAK=segments-registry-unchecked  开跑前不核按段登记文件
#       segments-registry-refused（「按段登记文件核不过应当退 2、什么都不起」）
#   CRASH_POINT_REPLAY_BREAK=given-states-run-condition-as-red  快档日志里有给定状态表借不到显存那一行也照红记（不退 78）
#       fast-tier-run-condition-not-met（「快档日志里有给定状态表借不到显存那一行……整趟退 78」）
#   CRASH_POINT_REPLAY_BREAK=two-hosts-not-used  多机配置开了双机，经给定状态引擎的测试目标也照本机一趟跑（改前的样子）
#       fast-tier-two-hosts（「开了双机，经给定状态引擎的测试目标应当照双机驱动的三步跑」）、
#       fast-tier-two-hosts-peer-unreachable（「第二台连不上…应当记本次未跑、写原因、整趟退 78」）、fast-tier-two-hosts-configuration-refused（「开了双机而判法判不过…」）、
#       fast-tier-slice-states-sent（「✗ 自证 fast-tier-slice-states-sent：」：双机开的那一趟没有双机驱动本机与第二台那几趟的调用记录）、
#       local-only-baseline-sees-peer（「✗ 自证 local-only-baseline-sees-peer：」：对照：不给 --local-only 时双机驱动该起）
#   CRASH_POINT_REPLAY_BREAK=two-hosts-when-off  双机关着也把经给定状态引擎的测试目标交双机驱动
#       fast-tier-two-hosts-off（「双机关着，经给定状态引擎的测试目标应当照旧本机一趟」）、
#       fast-tier-two-hosts-configuration-refused（「✗ 自证 fast-tier-two-hosts-configuration-refused：」：不调判法就交了双机驱动，判法判不过也照跑）、
#       local-only（「✗ 自证 local-only：」：派生的配置里双机关着也交了双机驱动）、local-only-keeps-gpu-drops-peer-cards（「✗ 自证 local-only-keeps-gpu-drops-peer-cards：」：同上）、local-only-on-peer-keeps-own-cards（「✗ 自证 local-only-on-peer-keeps-own-cards：」：角色 peer 那一格同上）
#   CRASH_POINT_REPLAY_BREAK=two-hosts-unreachable-as-red  双机驱动退 78（第二台连不上）照红记
#       fast-tier-two-hosts-peer-unreachable（「第二台连不上…应当记本次未跑、写原因、整趟退 78」）
#   CRASH_POINT_REPLAY_BREAK=two-hosts-check-failure-as-single  多机配置开了双机而判法判不过，照双机关、本机一趟跑
#       fast-tier-two-hosts-configuration-refused（「开了双机而判法判不过…」）
#   CRASH_POINT_REPLAY_BREAK=memory-cap-not-forwarded  套在内存包装里也不把外层的上限给放量入口与双机驱动（改前的样子）
#       outer-memory-cap-forwarded（「套在内存包装里，放量入口应当带外层的上限」）、fast-tier-two-hosts（「✗ 自证 fast-tier-two-hosts：」：双机驱动那一趟没带 --memory-cap 3G）
#   CRASH_POINT_REPLAY_BREAK=threads-not-defaulted  快档的 cargo 不经 capped.sh default 起，吃 cargo 自己的缺省（改前的样子）
#       fast-tier-threads-default（「快档每一趟 cargo 都应当带登记的重活默认并行度」）
#   CRASH_POINT_REPLAY_BREAK=full-in-a-library-of-its-own  --full 那一支照改前给放量部分另建一份空判定库（<放量的家目录>/crash-point-replay-full/…，跑完没人读）
#       full-heavy（「全量应当只写不读…」：另建库那一支不带 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1）、
#       full-writes-back-into-the-one-library（「--full 那一支应当照键写回同一份库、不另建库」）
#   CRASH_POINT_REPLAY_BREAK=quick-slice-states-not-set  轻型起放量入口时调用方没设也不设 SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES（改前的样子：分片的段照 2^32 判）
#       quick-slice-states-sent（「轻型起放量入口时调用方没设……应当设成 65536」）
#   CRASH_POINT_REPLAY_BREAK=quick-slice-states-override-caller  轻型起放量入口时调用方设了也改成 65536
#       quick-slice-states-sent（「调用方设了……放量入口应当照原值收到」）
#   CRASH_POINT_REPLAY_BREAK=heavy-slice-states-set  不带 --quick 的跑法起放量入口时也设成 65536
#       quick-slice-states-sent（「不带 --quick 的跑法……不设」）
#   CRASH_POINT_REPLAY_BREAK=fast-tier-slice-states-not-set  快档那一段调用方没设也不设 SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES（改前的样子：分片的段照 2^32 判）
#       fast-tier-slice-states-sent（「快档那一段调用方没设……每一趟应当收到 65536」）
#   CRASH_POINT_REPLAY_BREAK=fast-tier-slice-states-override-caller  快档那一段调用方设了也改成 65536
#       fast-tier-slice-states-sent（「调用方设了……快档每一趟应当照原值收到」）
#   CRASH_POINT_REPLAY_BREAK=fast-tier-red-continues  重型的快档红了也照旧往下起崩溃点重放部分与放量部分（改前的样子）
#       fast-tier-red-stops-heavy（「快档红了，重型的剪枝、逐条、全量都应当退 1、不起崩溃点重放那一道与放量入口……」）
#   CRASH_POINT_REPLAY_BREAK=logs-single  每一段的日志只写一份全量的（改前的样子：tee，不经 two-logs.sh，没有 .problems.log）
#       two-logs-per-part（「快档红的那一段应当出两份日志……」）
#   CRASH_POINT_REPLAY_BREAK=pipeline-78-as-red  放量入口退 78 而没打 ITEM 行，照旧记红（收不回，改前的样子）
#       pipeline-entry-78-not-run（「放量入口退 78 而没打 ITEM 行，两项应当记本次未跑……」）、pipeline-entry-78-red-first（「✗ 自证 pipeline-entry-78-red-first：」：gamma 记了红，不是本次未跑）
#   CRASH_POINT_REPLAY_BREAK=local-only-ignored  --local-only 认下却不派生多机配置，双机照开
#       local-only（「给了 --local-only，开了双机的多机配置也应当不起双机驱动……」）、local-only-keeps-gpu-drops-peer-cards（「✗ 自证 local-only-keeps-gpu-drops-peer-cards：」：显卡先后里第二台的项还在）、local-only-on-peer-keeps-own-cards（「✗ 自证 local-only-on-peer-keeps-own-cards：」：角色 peer 那一格同上）
#   CRASH_POINT_REPLAY_BREAK=local-only-role-ignored  --local-only 派生多机配置时不看角色，一律去 peer: 项（改前的样子：在第二台上去掉的是第二台自己的卡）
#       local-only-on-peer-keeps-own-cards（「✗ 自证 local-only-on-peer-keeps-own-cards：」：显卡先后只剩起活那一台的 local: 项）
#   CRASH_POINT_REPLAY_BREAK=local-only-accepted-with-list-items  --local-only 与 --list-items 一起给也不拒
#       local-only-usage（「--local-only 与 --list-items 一起给应当退 2、什么都不起」）
#   CRASH_POINT_REPLAY_BREAK=flow-coverage-table-skipped  跑完不往这一趟的日志目录落覆盖表与记录（改前的样子）
#       flow-coverage-table（「跑完应当在这一趟的日志目录里落覆盖表 crash-flow-coverage.md 与记录 crash-flow-records.tsv…」）
#   CRASH_POINT_REPLAY_BREAK=rocksdb-entry-unchecked  起 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry（改前的样子）
#       rocksdb-entry-refused（「checker 档包依赖 rocksdb 而进程环境里没有预编库的取值，应当在起 cargo 之前退 78…」：往下走起了假 cargo）
#   CRASH_POINT_REPLAY_BREAK=run-target-kept  跑完不删这一趟自己建的编译目录（改前的样子：快档在仓根照仓根的 target 编、跨趟留着）
#       run-target-green（「跑完运行目录里不剩它建的编译目录（正常退 0）」）、run-target-red（「跑完运行目录里不剩它建的编译目录（快档红退 1）」）、
#       run-target-term（「跑完运行目录里不剩它建的编译目录（被 TERM 停）」）
#   CRASH_POINT_REPLAY_BREAK=backlog-usage-unchecked  积压跑法的用法不核（--backlog 缺 --milestone、与 --quick / --full / --item 一起给、--milestone 不跟 --backlog 都照跑）
#       backlog-usage（「积压跑法的用法错应当退 2、什么都不起」）
#   CRASH_POINT_REPLAY_BREAK=backlog-table-unchecked  积压跑法开跑前不经小程序 append-missing 与 check 核表
#       backlog-table-refused（「积压登记表核不过（小程序 check 退 1）应当退 2、不取件、什么都不起」）
#   CRASH_POINT_REPLAY_BREAK=backlog-plan-unchecked  小程序 plan 的输出不核（完整性闸不判，照它报的件跑）
#       backlog-plan-integrity（「小程序 plan 的输出对不上（末行的积压态数比取件行加起来多 1）应当退 1、一件都不跑」）
#   CRASH_POINT_REPLAY_BREAK=backlog-bounds-dropped  两条下限不交给小程序 plan（照小程序的默认取件）
#       backlog-stops-at-the-bounds（「取件取到两条下限就停」：多跑了 gamma-amp）、list-items-backlog（「--list-items --backlog 应当……」：到积压下限那一行报到序号 4、不是 3）
#   CRASH_POINT_REPLAY_BREAK=backlog-table-order  取到的件照表的序号重排再跑（不照取件的次序）
#       backlog-new-and-key-changed-first（「这一里程碑新增与判法键变了的应当先跑」：alpha-amp 排到了最前）
#   CRASH_POINT_REPLAY_BREAK=backlog-not-recorded  跑完一件不经小程序 record 回写（照说回写了）
#       backlog-stops-at-the-bounds、backlog-below-the-bounds-takes-everything、backlog-red-part-recorded-red（三格都核 record 的调用）
#   CRASH_POINT_REPLAY_BREAK=backlog-red-as-green  红的那一件照「全绿」回写
#       backlog-red-part-recorded-red（「积压跑法里红的那一件应当回写成红、整趟退 1」）
#   CRASH_POINT_REPLAY_BREAK=backlog-not-run-recorded  本次未跑的那一件也照「全绿」回写
#       backlog-below-the-bounds-takes-everything（「本次未跑的那一件不回写」：gamma-amp 回写成了全绿）
#   CRASH_POINT_REPLAY_BREAK=backlog-listing-appends  只读用法照积压跑法那样先 append-missing（改了表）
#       list-items-backlog（「--list-items --backlog 应当……不改表」：调用记录里有 append-missing）
# 只供自证（自证起的子进程里才认；平常跑时有一个非空就退 2 拒跑，--list-items 与 --selftest 本身不拒）：CRASH_POINT_REPLAY_BREAK=<项>（上面那张表）；
#   CRASH_POINT_REPLAY_ROOT=<造出来的小仓>：合表、stage-inputs.tsv、门禁 checker-tier-crash-point-replay、crash-amplification.sh 与准入模块 admission.py 都从它取
#   （那三份是自证写的假的：只记一行参数、照环境变量打结果）；
#   CRASH_POINT_REPLAY_KEYS_READER_OUTPUT=<文件>：--list-items --with-keys 不起读库程序，拿这份文件的内容当它的输出（list-items-with-keys 格用）。
#   「自证起的子进程」的标记：CRASH_POINT_REPLAY_SELFTEST_MARK=<文件>，那份文件在、内容是 SELFTEST_MARK_CONTENT 那一行；只有 run_selftest 设它。
#   这三个变量读完就从环境里清掉，不传给背后的驱动。自证里的 cargo 是 PATH 最前面的假 cargo（起之前核 PATH 上解到的就是它，不是就不起）；
#   清场走真的 scripts/crash/gpu-quiesce-run.sh，多机配置指到临时目录里只写了假清场命令的一份，状态与日志目录都在临时目录里。
# 自证末尾的开关轮并行度 CRASH_AMPLIFICATION_SELFTEST_JOBS，没给取 SINGLEFS_THREAD_CAP，再没给取登记的重活默认并行度（同 crash-amplification-lib.sh 文件头「弄坏开关轮」）。
#
# gate-similar: crash-amplification.sh 它是放量分项的登记表与全量循环；这里是合表的入口，放量部分每趟调它一次，崩溃点重放部分调门禁 checker-tier-crash-point-replay
# gate-similar: checker-tier-crash-point-replay.sh 它跑崩溃点重放快档与登记的崩溃枚举用例；这里的重型照项调它的 --full --item，快档那一段只跑合表点名的测试目标
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

LIGHT_EXPAND_UP_TO=6
LIGHT_REGISTERED_SEGMENT_SLICE_STATES=65536
CRASH_LIBRARY_DIRECTORY_NAME="crash-state-verdicts"   # 那一份长期判定库在放量家目录下的目录名（几类测试同一份；crates/singlefs-checker-tier/src/crash_library.rs 的 CRASH_LIBRARY_DIRECTORY_NAME，逐字相同）
SELFTEST_MARK_CONTENT="crash-point-replay.sh --selftest 起的子进程"
BOTH_TIERS_GROUPS=(
  $'gpu-and-cpu-agree-on-small-inputs\tcrash_judge_gpu_agrees_with_the_cpu_judge_on_every_state crash_facts_gpu_verifier_agrees_with_the_cpu_reference gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core crash_judge_gpu_secondary_reads_after_the_source_segment_is_discarded'
  $'pruning-self-proof\tcrash_amplification_two_flows_share_their_prefix crash_amplification_pipeline_records_checks_and_reuses'
  $'pinned-red-examples\t-'
)
PINNED_RED_EXAMPLES_NOT_RUN_REASON="崩溃点重放还没有钉表：全量出的红要钉回快档（test-script-discipline.md「全量的红，钉回快档」），今天一条都没钉"

replay_script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
replay_script_directory="$(dirname "$replay_script")"
source "$replay_script_directory/crash-amplification-lib.sh"
source "$replay_script_directory/../hosts/run-build-directory.sh"
repository_root="$(cd "$replay_script_directory/../.." && pwd)"

# 只供自证的三个变量（文件头「只供自证」）：读完就清掉
replay_in_selftest_child=0
if [[ -n "${CRASH_POINT_REPLAY_SELFTEST_MARK:-}" && -f "${CRASH_POINT_REPLAY_SELFTEST_MARK:-}" ]] \
   && [[ "$(head -n 1 -- "$CRASH_POINT_REPLAY_SELFTEST_MARK")" == "$SELFTEST_MARK_CONTENT" ]]; then
  replay_in_selftest_child=1
fi
replay_break="${CRASH_POINT_REPLAY_BREAK:-}"
replay_root_override="${CRASH_POINT_REPLAY_ROOT:-}"
replay_keys_reader_output="${CRASH_POINT_REPLAY_KEYS_READER_OUTPUT:-}"
replay_test_variables=()
[[ -n "$replay_break" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_BREAK)
[[ -n "$replay_root_override" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_ROOT)
[[ -n "$replay_keys_reader_output" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_KEYS_READER_OUTPUT)
unset CRASH_POINT_REPLAY_BREAK CRASH_POINT_REPLAY_ROOT CRASH_POINT_REPLAY_SELFTEST_MARK CRASH_POINT_REPLAY_KEYS_READER_OUTPUT
replay_break_is() { [[ ",$replay_break," == *",$1,"* ]]; }
# 起任何 cargo 之前判这台的 RocksDB 预编库取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」：这台装了、此刻进程环境的取值对得上）。判红交回 1，
# entry 自己打 ✗ 与出路；树里没有依赖 rocksdb 的清单、PATH 上是带标记的假 cargo，entry 报不判、交回 0。弄坏开关 rocksdb-entry-unchecked 下不判
replay_rocksdb_entry_holds() {
  replay_break_is rocksdb-entry-unchecked && return 0
  python3 "$replay_script_directory/../hosts/rocksdb-prebuilt.py" entry --root "$repository_root" >&2
}

usage() {
  echo "  ✗ 用法不对：$1。用法：$0 [--quick | --full] [--local-only] [--item <项>]… | --backlog --milestone <mN> [--backlog-states-at-least <态数>] [--states-at-least <态数>] [--local-only] | --list-items [--with-keys] [--backlog --milestone <mN> …] | --selftest" >&2
  echo "  → 怎么办：不带参数是重型·剪枝；--quick 是轻型；--local-only 只用本机、不动第二台；项名照 --list-items 列的写；--quick 与 --full 不一起给；--backlog 照积压登记表取件（文件头「积压跑法」），要带 --milestone" >&2
  exit 2
}

tier=""; quick=0; full=0; list_items=0; with_keys=0; selftest=0; local_only=0; named_items=()
backlog=0; backlog_milestone=""; backlog_backlog_bound=""; backlog_total_bound=""   # 积压跑法（文件头「积压跑法」）的开关、里程碑与两条下限
while (($#)); do
  case "$1" in
    --quick) quick=1; shift ;;
    --full) full=1; shift ;;
    --list-items) list_items=1; shift ;;
    --with-keys) with_keys=1; shift ;;
    --selftest) selftest=1; shift ;;
    --local-only) local_only=1; shift ;;
    --backlog) backlog=1; shift ;;
    --milestone|--backlog-states-at-least|--states-at-least)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then usage "$1 后面缺值"; fi
      case "$1" in
        --milestone) backlog_milestone="$2" ;;
        --backlog-states-at-least) [[ "$2" =~ ^[0-9]+$ ]] || usage "--backlog-states-at-least 要十进制整数（态数），给的是 $2"; backlog_backlog_bound="$2" ;;
        *) [[ "$2" =~ ^[0-9]+$ ]] || usage "--states-at-least 要十进制整数（态数），给的是 $2"; backlog_total_bound="$2" ;;
      esac
      shift 2 ;;
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
  if note="$(SINGLEFS_MULTI_HOST_CONFIG="$configuration_file" bash "$replay_script_directory/crash-point-replay-shard-configuration-check.sh" "$repository_root" 2>&1)"; then check_exit=0; else check_exit=$?; fi
  case "$check_exit" in
    0) fast_tier_two_hosts=1; echo "  · 双机：开（${note//$'\n'/；}）：快档里经给定状态引擎的测试目标交 crash-amplification-two-hosts.sh 两台各判一份" ;;
    3) echo "  · 双机：关（${note//$'\n'/；}）：快档全在本机跑" ;;
    *)
      if replay_break_is two-hosts-check-failure-as-single; then return 0; fi
      fast_tier_two_hosts_refusal="运行条件不满足：多机配置开了双机，判法 crash-point-replay-shard-configuration-check.sh 判不过（退 $check_exit：${note//$'\n'/；}），这个测试目标一趟都没起；修好第二台或多机配置再跑，不想用第二台就把 ENABLE_ACROSS_MACHINES 改成 0" ;;
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
# 复用计数行（文件头「快档的复用数怎么取」）：CRASH_POINT_REPLAY_PARALLEL_FINISHED 的 resumed_slices= 之和；一行都没有、有一行缺或写坏，取不出
finished = [line.strip() for line in text.splitlines() if line.lstrip().startswith("CRASH_POINT_REPLAY_PARALLEL_FINISHED ")]
reused = 0 if finished else None
for line in finished:
    value = dict(token.split("=", 1) for token in line.split()[1:] if "=" in token).get("resumed_slices", "")
    if not value.isdigit():
        reused = None
        break
    reused += int(value)
print(passed, failed, not_run, len(condition), "unknown" if reused is None else reused)
print(condition[0] if condition else "")
PYTHON
)"
  first_line="${counts%%$'\n'*}"
  condition_line=""
  [[ "$counts" == *$'\n'* ]] && condition_line="${counts#*$'\n'}"
  read -r passed failed not_run_lines condition_lines fast_reused <<<"${first_line:-0 0 0 0 unknown}"
  [[ "$fast_reused" =~ ^[0-9]+$ ]] || fast_reused=unknown
  replay_break_is fast-tier-reuse-not-counted && fast_reused=none   # 弄坏开关：改前写死 none
  [[ "$passed" =~ ^[0-9]+$ ]] || passed=0
  [[ "$failed" =~ ^[0-9]+$ ]] || failed=0
  [[ "$not_run_lines" =~ ^[0-9]+$ ]] || not_run_lines=0
  [[ "$condition_lines" =~ ^[0-9]+$ ]] || condition_lines=0
  fast_state=green; fast_reason=""; fast_exit="$run_exit"; fast_red=0
  if (( condition_lines > 0 )) && ! replay_break_is given-states-run-condition-as-red; then
    fast_state=not-run; fast_exit=78
    fast_reason="运行条件不满足：给定状态表的判器借不到显存（$condition_line；共 $condition_lines 处，日志 $log、错误警告 ${log%.log}.problems.log），这一趟不算数；等卡上占着显存的跑完再重跑"
  elif [[ "$run_exit" != 0 || "$failed" -gt 0 ]] && ! replay_break_is red-as-green; then
    fast_state=red; fast_reason="cargo test 退 $run_exit、红 $failed 条（日志 $log、错误警告 ${log%.log}.problems.log）"
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
# 调用方 run_fast_tier 的 quiesce_prefix（开了 GPU 整条经清场包装起）与 fast_tier_slice_states（分片的段一趟判几个态）照用
run_fast_tier_target_on_two_hosts() { # <测试目标> <记给哪几项>
  local target="$1" owners="$2" owner started label library_root="" library library_parent two_hosts_runs two_hosts_log two_hosts_exit variable
  local two_hosts_environment=() two_hosts_arguments=()
  started=$SECONDS
  if [[ -n "$fast_tier_two_hosts_refusal" ]]; then
    for owner in $owners; do record_part "$owner" fast-tier "$target" 78 not-run 0 0 unknown 0 "$fast_tier_two_hosts_refusal"; done
    return 0
  fi
  label="crash-point-replay-$run_stamp-$target"
  # 那一份长期库（文件头「双机」）：跨趟留着、键相同的表复用；库目录本身留给引擎建（见 <库>/CURRENT 不在就建空库）
  library="$pipeline_home/$CRASH_LIBRARY_DIRECTORY_NAME"; library_parent="$pipeline_home"
  if replay_break_is fast-tier-two-hosts-throwaway-library; then   # 弄坏开关：改前每趟一份新建、跑完删掉的空库
    library_root="$pipeline_home/crash-point-replay-fast-tier/$run_stamp"; library="$library_root/$target/library"; library_parent="$library_root/$target"
  fi
  two_hosts_runs="$pipeline_home/runs/$label"
  if ! mkdir -p -- "$library_parent"; then
    for owner in $owners; do record_part "$owner" fast-tier "$target" 78 not-run 0 0 unknown 0 "运行条件不满足：建不了判定库的上层目录 $library_parent（看 $pipeline_home 可不可写、盘满没满）"; done
    return 0
  fi
  two_hosts_environment=(env -u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE -u SINGLEFS_GPU_CARDS_CONFIG -u GPU_COMPUTATION_CLASSES -u GPU_CARDS_BY_PRIORITY)
  for variable in $(compgen -e); do [[ "$variable" == SINGLEFS_CRASH_AMPLIFICATION_* ]] && two_hosts_environment+=(-u "$variable"); done
  two_hosts_environment+=("SINGLEFS_CRASH_AMPLIFICATION_HOME=$pipeline_home" ${fast_tier_slice_states[@]+"${fast_tier_slice_states[@]}"})
  [[ -f "$configuration_file" ]] && two_hosts_environment+=("SINGLEFS_MULTI_HOST_CONFIG=$(cd "$(dirname "$configuration_file")" && pwd)/$(basename "$configuration_file")")
  two_hosts_arguments=("$target" --library "$library" --label "$label" --given-states)
  [[ -n "$outer_memory_cap" ]] && two_hosts_arguments+=(--memory-cap "$outer_memory_cap")
  (( gpu_switch )) && two_hosts_arguments+=(--quiesced)
  two_hosts_log="$run_directory/fast-tier.$target.two-hosts.log"
  echo "── 快档（两台分片）：bash scripts/crash/crash-amplification-two-hosts.sh ${two_hosts_arguments[*]}（本机 0/2、第二台 1/2 取回库、本机带 IMPORT 再跑${quiesce_prefix[*]:+；经 scripts/crash/gpu-quiesce-run.sh 清场再起}）"
  run_logged "$two_hosts_log" ${quiesce_prefix[@]+"${quiesce_prefix[@]}"} "${two_hosts_environment[@]}" bash "$replay_script_directory/crash-amplification-two-hosts.sh" "${two_hosts_arguments[@]}"
  two_hosts_exit="$(cat "$two_hosts_log.exit" 2>/dev/null || echo 255)"
  # 长期库留着（下一趟照键复用、打断之后续跑）；取回的那一份与送第二台的快照是这一趟的拷贝，双机驱动退出时已经自己删了（它文件头「跑完删库的拷贝」），这里再兜一次；日志留着
  rm -rf -- "${two_hosts_runs:?}/fetched" "${two_hosts_runs:?}/library-for-peer"
  if [[ -n "$library_root" ]]; then
    rm -rf -- "${library_root:?}/${target:?}"
    rmdir -- "$library_root" "$(dirname "$library_root")" 2>/dev/null
  fi
  fast_reused=unknown
  case "$two_hosts_exit" in
    0) judge_fast_tier_log 0 "$two_hosts_runs/import.log" ;;
    78)
      fast_tests=0; fast_red=0
      if replay_break_is two-hosts-unreachable-as-red; then
        fast_state=red; fast_exit=1; fast_red=1; fast_reason="双机驱动退 78（日志 $two_hosts_log、错误警告 ${two_hosts_log%.log}.problems.log、$two_hosts_runs/）"
      else
        fast_state=not-run; fast_exit=78
        fast_reason="运行条件不满足：双机驱动退 78（第二台连不上、第二台的库没取回、卡用不上或给定状态表的判器借不到显存，出路见它打的那一句；日志 $two_hosts_log、错误警告 ${two_hosts_log%.log}.problems.log、$two_hosts_runs/）"
      fi ;;
    245|251|252|254)
      fast_state=not-run; fast_exit=78; fast_tests=0; fast_red=0
      fast_reason="运行条件不满足：双机驱动退 $two_hosts_exit（内存包装自己的结局或显存不够，这一次的输出不算结果：scripts/hosts/run-with-memory-cap.sh 文件头「退出码」；日志 $two_hosts_log、错误警告 ${two_hosts_log%.log}.problems.log、$two_hosts_runs/）" ;;
    77)
      fast_state=not-run; fast_exit=77; fast_tests=0; fast_red=0
      fast_reason="双机驱动退 77（本次未跑；日志 $two_hosts_log、错误警告 ${two_hosts_log%.log}.problems.log、$two_hosts_runs/）" ;;
    *)
      judge_fast_tier_log "$two_hosts_exit" "$two_hosts_runs/import.log"
      fast_reason="双机驱动退 $two_hosts_exit；${fast_reason:-导入那一趟的结果判不出红}（日志 $two_hosts_log、错误警告 ${two_hosts_log%.log}.problems.log、$two_hosts_runs/）" ;;
  esac
  for owner in $owners; do record_part "$owner" fast-tier "$target" "$fast_exit" "$fast_state" "$fast_tests" "$fast_red" "$fast_reused" "$((SECONDS - started))" "$fast_reason"; done
}

# 一档的计划（纯函数）：打这一档要做的每一件「项<TAB>种类<TAB>对象」。种类：fast-tier（快档的测试目标）、pipeline-expand-<N>（放量分项按展开 N）、
# pipeline-full（放量分项按全量展开深度）、enumeration-full（crash-case: 键交 checker-tier-crash-point-replay --full）、not-run（两档都跑的一类没有测试目标，对象是原因）。
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

# 跑一条命令：输出边跑边转打，同时出两份日志（scripts/hosts/two-logs.sh：<日志> 是全量，同名的 .problems.log 只收错误与警告、行前带全量日志里的行号，一般只看它）；
# 退出码写进 <日志>.exit。弄坏开关 logs-single：照改前只写一份全量日志（tee）
run_logged() { # <日志，以 .log 结尾> <命令…>
  local log="$1" exit_code
  shift
  if replay_break_is logs-single; then
    { "$@" 2>&1; echo "$?" >"$log.exit"; } | tee "$log"
    return 0
  fi
  if bash "$replay_script_directory/../hosts/two-logs.sh" "${log%.log}" -- "$@" 2>&1; then exit_code=0; else exit_code=$?; fi
  echo "$exit_code" >"$log.exit"
}

run_fast_tier() { # <计划文件>：快档那一段（fast-tier 的件），一个测试目标一趟；同一个测试目标几项都点名时跑一次、记给每一项
  local plan_file="$1" targets=() target target_arguments=() environment=() quiesce_prefix=() build_log build_exit run_log run_exit
  local variable started owners owner any_given_states=0 fast_tier_slice_states=() thread_prefix=() test_threads_arguments=()
  local fast_state fast_reason fast_exit fast_tests fast_red fast_reused
  while IFS=$'\t' read -r _plan_item plan_kind plan_object; do
    [[ "$plan_kind" == fast-tier ]] || continue
    [[ " ${targets[*]-} " == *" $plan_object "* ]] || targets+=("$plan_object")
  done <"$plan_file"
  (( ${#targets[@]} )) || return 0
  for target in "${targets[@]}"; do target_arguments+=(--test "$target"); [[ -n "${given_states_targets[$target]+set}" ]] && any_given_states=1; done
  # 经给定状态引擎的测试目标走不走两台（文件头「双机」）：只在计划里有这样的测试目标时判
  (( any_given_states )) && decide_fast_tier_two_hosts
  environment=(env -u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE -u SINGLEFS_GPU_CARDS_CONFIG -u GPU_COMPUTATION_CLASSES -u GPU_CARDS_BY_PRIORITY)
  for variable in $(compgen -e); do [[ "$variable" == SINGLEFS_CRASH_AMPLIFICATION_* ]] && environment+=(-u "$variable"); done
  environment+=("BINDGEN_EXTRA_CLANG_ARGS=$CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS")
  [[ -f "$configuration_file" ]] && environment+=("SINGLEFS_MULTI_HOST_CONFIG=$(cd "$(dirname "$configuration_file")" && pwd)/$(basename "$configuration_file")")
  # 分片的段一趟判几个态（文件头「往引擎送的变量」）：不分档，调用方没设就设成一块，设了照原值交（空串也照原样，引擎拒）；
  # 上面 -u 的前缀是 SINGLEFS_CRASH_AMPLIFICATION_，清不到它。交双机驱动的那几趟（run_fast_tier_target_on_two_hosts）同样带 fast_tier_slice_states
  if replay_break_is fast-tier-slice-states-override-caller \
     || { [[ -z "${SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES+set}" ]] && ! replay_break_is fast-tier-slice-states-not-set; }; then
    fast_tier_slice_states=("SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=$LIGHT_REGISTERED_SEGMENT_SLICE_STATES")
  fi
  environment+=(${fast_tier_slice_states[@]+"${fast_tier_slice_states[@]}"})
  # 快档的 cargo 不吃 cargo 自己的缺省（核数）：经 capped.sh default 起，编译与测试线程、各装置的线程变量都取登记的重活默认并行度（SINGLEFS_THREAD_CAP 设了取它）
  replay_break_is threads-not-defaulted || thread_prefix=(bash "$replay_script_directory/../hosts/capped.sh" default)
  if (( gpu_switch )); then
    replay_break_is gpu-cards-not-sent || environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_for_fast_tier")
    replay_break_is fast-tier-classes-sent && environment+=("GPU_COMPUTATION_CLASSES=gpu_unit_checks crash_verify_gpu crash_judge_gpu")   # 弄坏开关：改前送的三类
    replay_break_is gpu-without-quiesce || quiesce_prefix=(bash "$replay_script_directory/gpu-quiesce-run.sh" --)
    # 单机路径带 gpu 特性的每一趟测试线程只开 1 个（GPU 对拍的测试各占卡的显存，并行起会互相抢）；双机分支（下面的 run_fast_tier_target_on_two_hosts）起的双机驱动还不接这个参数，文件头写明了；
    # 一个测试里 CPU 判器自己的线程不受这个参数管（读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取本机可用并行度）
    replay_break_is gpu-test-threads-unset || test_threads_arguments=(--test-threads 1)
  fi
  echo "── 快档那一段：${#targets[@]} 个测试目标，cargo test --release -p singlefs-checker-tier --features $features（不带 --ignored${quiesce_prefix[*]:+；每一趟经 scripts/crash/gpu-quiesce-run.sh 清场再起}）"
  build_log="$run_directory/fast-tier-build.log"
  started=$SECONDS
  run_logged "$build_log" "${environment[@]}" ${thread_prefix[@]+"${thread_prefix[@]}"} cargo test --release -p singlefs-checker-tier --features "$features" --no-run "${target_arguments[@]}"
  build_exit="$(cat "$build_log.exit" 2>/dev/null || echo 255)"
  for target in "${targets[@]}"; do
    owners="$(awk -F'\t' -v wanted="$target" '$2 == "fast-tier" && $3 == wanted { print $1 }' "$plan_file")"
    if [[ "$build_exit" != 0 ]]; then
      for owner in $owners; do record_part "$owner" fast-tier "$target" "$build_exit" red 0 1 unknown "$((SECONDS - started))" "快档编不过（cargo test --no-run 退 $build_exit，日志 $build_log、错误警告 ${build_log%.log}.problems.log）"; done
      continue
    fi
    # 经给定状态引擎的测试目标，双机开着（或开着而判法判不过）：交双机驱动（文件头「双机」）
    if [[ -n "${given_states_targets[$target]+set}" ]] && { (( fast_tier_two_hosts )) || [[ -n "$fast_tier_two_hosts_refusal" ]]; }; then
      run_fast_tier_target_on_two_hosts "$target" "$owners"
      continue
    fi
    run_log="$run_directory/fast-tier.$target.log"
    started=$SECONDS
    run_logged "$run_log" ${quiesce_prefix[@]+"${quiesce_prefix[@]}"} "${environment[@]}" ${thread_prefix[@]+"${thread_prefix[@]}"} cargo test --release -p singlefs-checker-tier --features "$features" --test "$target" -- --nocapture ${test_threads_arguments[@]+"${test_threads_arguments[@]}"}
    run_exit="$(cat "$run_log.exit" 2>/dev/null || echo 255)"
    judge_fast_tier_log "$run_exit" "$run_log"
    for owner in $owners; do record_part "$owner" fast-tier "$target" "$fast_exit" "$fast_state" "$fast_tests" "$fast_red" "$fast_reused" "$((SECONDS - started))" "$fast_reason"; done
  done
}

run_enumeration_part() { # <项> <crash-case: 键…>：交门禁 checker-tier-crash-point-replay --full --item（全量带 --start-over：只写不读、全部块现判、写回同一份库）
  local item="$1" key arguments=(--full) log exit_code state reason reused started red_cases=0
  shift
  started=$SECONDS
  if [[ "$tier" == full ]] && ! replay_break_is full-keeps-reuse; then
    arguments+=(--start-over)
  fi
  for key in "$@"; do arguments+=(--item "$key"); done
  log="$run_directory/enumeration.$item.log"
  [[ "$tier" == backlog ]] && log="$run_directory/enumeration.$item.${1#crash-case:}.log"   # 积压跑法一趟只交一条用例，同一项的几条各一份日志
  echo "── 崩溃点重放部分（$item）：bash .claude/gate.d/checker-tier-crash-point-replay.sh ${arguments[*]} <根>"
  run_logged "$log" bash "$repository_root/.claude/gate.d/checker-tier-crash-point-replay.sh" "${arguments[@]}" "$repository_root"
  exit_code="$(cat "$log.exit" 2>/dev/null || echo 255)"
  reused="$(grep -cE '^  · crash-case:[a-z0-9-]+ 复用：' "$log")" || reused=0
  replay_break_is enumeration-exit-ignored && exit_code=0
  case "$exit_code" in
    0) state=green; reason="" ;;
    77) state=not-run; reason="崩溃点重放那一道退 77（本次未跑）" ;;
    3) state=not-run; reason="崩溃点重放那一道退 3（样本档，没跑真的崩溃点重放）" ;;
    78) state=not-run; reason="崩溃点重放那一道退 78（运行条件不满足：修好设备，或改配置）" ;;
    *) state=red; reason="崩溃点重放那一道退 $exit_code（日志 $log、错误警告 ${log%.log}.problems.log）"; red_cases=$# ;;
  esac
  record_part "$item" enumeration "$*" "$exit_code" "$state" "$#" "$red_cases" "$reused" "$((SECONDS - started))" "$reason"
}

# 只用本机（文件头参数表 --local-only）：多机配置派生一份临时的，ENABLE_ACROSS_MACHINES 写成 0，去掉第二台的键与显卡先后里另一台的项（照角色取，见下；这一台的 GPU 与清场照旧），
# 改 configuration_file 指到它并导出 SINGLEFS_MULTI_HOST_CONFIG（快档、门禁、放量入口都经这个变量找多机配置）；多机配置不在时两个开关本来就关，什么都不用做。
# 临时的一份放在 local_only_directory 里，进程退出时删。
local_only_directory=""
# 收尾（EXIT trap）：删 --local-only 派生多机配置的临时目录与这一趟自己建的编译目录（文件头「编译目录」）；编译目录没删成而本来要退 0 的改退 1。
#   弄坏开关 run-target-kept：编译目录不删
replay_finish() {
  local finish_exit=$?
  trap - EXIT
  if [[ -n "${local_only_directory:-}" ]]; then rm -rf -- "${local_only_directory:?}"; fi
  if ! run_build_directory_end $(replay_break_is run-target-kept && echo keep); then
    (( finish_exit != 0 )) || finish_exit=1
  fi
  exit "$finish_exit"
}

make_local_only_configuration() {
  local source_file="$configuration_file" derived_file
  if [[ ! -f "$source_file" ]]; then
    echo "  · 只用本机：多机配置 $source_file 不在，双机与 GPU 两个开关本来就关，照原样跑"
    return 0
  fi
  if ! local_only_directory="$(mktemp -d "${TMPDIR:-/tmp}/crash-point-replay-local-only.XXXXXX")"; then
    echo "  ✗ 只用本机：建不了放派生多机配置的临时目录（${TMPDIR:-/tmp}）" >&2
    echo "  → 怎么办：看 ${TMPDIR:-/tmp} 可不可写、盘满没满；不想为这个建目录就改多机配置的 ENABLE_ACROSS_MACHINES 为 0 再不带 --local-only 跑" >&2
    exit 78
  fi
  trap replay_finish EXIT   # 收尾连同这一趟的编译目录一起删（replay_finish）
  derived_file="$local_only_directory/multi-host.env"
  # 显卡先后里哪几项是这一台的照角色取（scripts/hosts/peer-host-lib.sh 文件头「两台用同一份多机配置」）：角色 local（没设也是）留 local: 项、去 peer: 项；
  #   角色 peer（这一份在第二台上跑，配置是起活那一台带过来的同一份）留 peer: 项、去 local: 项。弄坏开关 local-only-role-ignored 下照改前一律去 peer: 项
  local own_card_prefix="local:" other_card_prefix="peer:"
  if [[ "${SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE:-local}" == peer ]] && ! replay_break_is local-only-role-ignored; then
    own_card_prefix="peer:" other_card_prefix="local:"
  fi
  if ! python3 - "$source_file" "$derived_file" "$other_card_prefix" <<'PYTHON'
import sys
source, target, other_card_prefix = sys.argv[1], sys.argv[2], sys.argv[3]
second_host_keys = ("PEER_SSH_HOST", "PEER_REPOSITORY_DIRECTORY", "PEER_CARGO_BIN_DIRECTORY", "PEER_MEMORY_CAP", "HOST_JUDGE_WEIGHTS")
card_keys = ("GPU_CARDS_BY_PRIORITY", "GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED")
kept = ["# crash-point-replay.sh --local-only 从多机配置派生的一份：只用这一台（ENABLE_ACROSS_MACHINES 写成 0，去掉第二台的键与显卡先后里另一台的项，哪几项是这一台的照角色取）"]
for raw in open(source, encoding="utf-8"):
    line = raw.rstrip("\n")
    key, separator, value = line.strip().partition("=")
    if not separator or line.strip().startswith("#"):
        kept.append(line)
        continue
    if key == "ENABLE_ACROSS_MACHINES" or key in second_host_keys:
        continue
    if key in card_keys:
        text = value.strip()
        if len(text) >= 2 and text[0] == text[-1] and text[0] in "'\"":
            text = text[1:-1]
        kept.append(f"{key}='{' '.join(entry for entry in text.split() if not entry.startswith(other_card_prefix))}'")
        continue
    kept.append(line)
kept.append("ENABLE_ACROSS_MACHINES=0")
with open(target, "x", encoding="utf-8") as handle:
    handle.write("\n".join(kept) + "\n")
PYTHON
  then
    echo "  ✗ 只用本机：派生不出关了双机的多机配置（从 $source_file 到 $derived_file）" >&2
    echo "  → 怎么办：看上面 python3 报的错；多机配置读不了就修好它，或把 ENABLE_ACROSS_MACHINES 改成 0 再不带 --local-only 跑" >&2
    exit 78
  fi
  configuration_file="$derived_file"
  export SINGLEFS_MULTI_HOST_CONFIG="$derived_file"
  echo "  · 只用本机：多机配置照 $source_file 派生了一份关了双机的（去掉第二台的键与显卡先后里 $other_card_prefix 起头的项、留 $own_card_prefix 起头的这一台的卡），快档、门禁与放量入口都读这一份，另一台不动"
}

# 快档红：重型不起崩溃点重放部分与放量部分（文件头「一项怎么判」）。计划里这两段的每一件各记一笔本次未跑（退出码列 77、原因写快档红与红的测试目标）
skip_parts_after_red_fast_tier() { # <计划文件>
  local plan_file="$1" red_targets reason skipped_item skipped_kind skipped_object skipped_items=()
  local -A skipped_keys=()
  red_targets="$(awk -F'\t' '$2 == "fast-tier" && $5 == "red" && !seen[$3]++ { printf "%s%s", (n++ ? "、" : ""), $3 }' "$part_records")"
  reason="快档红（测试目标 ${red_targets}），先过快档才起这一段"
  while IFS=$'\t' read -r skipped_item skipped_kind skipped_object; do
    case "$skipped_kind" in
      enumeration-full)
        [[ -n "${skipped_keys[$skipped_item]+set}" ]] || skipped_items+=("$skipped_item")
        skipped_keys[$skipped_item]+="${skipped_keys[$skipped_item]:+ }$skipped_object" ;;
      pipeline-*) record_part "$skipped_item" pipeline "$skipped_object" 77 not-run 0 0 0 0 "$reason" ;;
    esac
  done <"$plan_file"
  for skipped_item in ${skipped_items[@]+"${skipped_items[@]}"}; do
    record_part "$skipped_item" enumeration "${skipped_keys[$skipped_item]}" 77 not-run 0 0 0 0 "$reason"
  done
  echo "  ✗ 本次未跑：快档红，崩溃点重放部分与放量部分这一趟没起（红的快档测试目标：${red_targets}）" >&2
  echo "  → 怎么办：先照下面每一项的原因与日志修好快档红的那几个测试目标，再重跑同一条命令（快档先过，才值得起枚举与放量）" >&2
}

# 判定库只建上层目录，库目录本身留给引擎新建（文件头「两个域的写法」：目录已经在，引擎拒 LibraryDirectoryAlreadyExists）
make_library_parent() { # <库目录>
  mkdir -p -- "$(dirname -- "$1")"
}

run_pipeline_part() { # <计划文件>：放量部分（pipeline-* 的件）一趟交 crash-amplification.sh
  local plan_file="$1" requested=() kind="" arguments=() log exit_code line name started fields state reason library="" all_named=1 word
  local item_exit item_states item_red item_reused item_verdict pipeline_environment=()
  while IFS=$'\t' read -r _plan_item plan_kind plan_object; do
    [[ "$plan_kind" == pipeline-* ]] || continue
    kind="$plan_kind"; requested+=("$plan_object")
  done <"$plan_file"
  (( ${#requested[@]} )) || return 0
  for word in "${!pipeline_owner[@]}"; do [[ " ${requested[*]} " == *" $word "* ]] || all_named=0; done
  if [[ "$kind" == pipeline-expand-* ]]; then
    arguments+=(--expand-up-to "$LIGHT_EXPAND_UP_TO")
    for word in "${requested[@]}"; do arguments+=(--item "$word"); done
    # 轻型用放量入口的默认长期库（文件头「展开 6」）：不交 --library，判过的块照库复用，被打断之后重跑同一条命令就是续跑
    if replay_break_is quick-throwaway-library; then   # 弄坏开关：改前每趟一份新建、跑完删掉的空库
      library="$pipeline_home/crash-point-replay-quick/$run_stamp/library"
      make_library_parent "$library" || { echo "  ✗ 建不了这一趟的空判定库的上层目录 $(dirname -- "$library")" >&2; echo "  → 怎么办：看 $pipeline_home 可不可写、盘满没满" >&2; exit 78; }
      arguments+=(--library "$library")
    fi
    # 分片的段只判第一块（文件头「展开 6」）：调用方没设就设成一块，设了照用
    if replay_break_is quick-slice-states-override-caller \
       || { [[ -z "${SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES+set}" ]] && ! replay_break_is quick-slice-states-not-set; }; then
      pipeline_environment+=("SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=$LIGHT_REGISTERED_SEGMENT_SLICE_STATES")
    fi
  else
    replay_break_is heavy-slice-states-set && pipeline_environment+=("SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=$LIGHT_REGISTERED_SEGMENT_SLICE_STATES")
    # 重型：全部项都点了就不带 --item（放量入口照全量判，scope=full）；全量只写不读：库不读、全部块现判、照键写回同一份库（不另建库）
    if [[ "$tier" == named || "$tier" == backlog ]] || (( ! all_named )); then
      for word in "${requested[@]}"; do arguments+=(--item "$word"); done
    fi
    if [[ "$tier" == full ]] && replay_break_is full-in-a-library-of-its-own; then   # 弄坏开关：改前另建一份跑完没人读的空库
      library="$pipeline_home/crash-point-replay-full/$run_stamp/library"
      make_library_parent "$library" || { echo "  ✗ 建不了全量那一份空判定库的上层目录 $(dirname -- "$library")" >&2; echo "  → 怎么办：看 $pipeline_home 可不可写、盘满没满" >&2; exit 78; }
      arguments+=(--library "$library")
    elif [[ "$tier" == full ]] && ! replay_break_is full-keeps-reuse; then
      pipeline_environment+=("SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1")
      echo "  · 全量（只写不读）：放量部分带 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1，库不读、全部块现判、照键写回同一份库"
    fi
  fi
  # 套在内存包装里：外层的上限给放量入口（文件头「内存包装」）
  [[ -n "$outer_memory_cap" ]] && arguments+=(--memory-cap "$outer_memory_cap")
  log="$run_directory/pipeline.log"
  [[ "$tier" == backlog ]] && log="$run_directory/pipeline.${requested[0]}.log"   # 积压跑法一趟只交一个分项
  echo "── 放量部分：${pipeline_environment[*]+${pipeline_environment[*]} }bash scripts/crash/crash-amplification.sh ${arguments[*]}"
  started=$SECONDS
  run_logged "$log" env ${pipeline_environment[@]+"${pipeline_environment[@]}"} bash "$repository_root/scripts/crash/crash-amplification.sh" ${arguments[@]+"${arguments[@]}"}
  exit_code="$(cat "$log.exit" 2>/dev/null || echo 255)"
  if [[ "$kind" == pipeline-expand-* && -n "$library" ]]; then
    rm -rf -- "${library:?}"
    rmdir -- "$(dirname "$library")" 2>/dev/null
  fi
  for name in "${requested[@]}"; do
    line="$(grep -E "^CRASH_AMPLIFICATION_ITEM item=$name " "$log" | tail -n 1)"
    if [[ -z "$line" ]]; then
      replay_break_is skip-collection-count && continue
      # 放量入口退 78（运行条件不满足：清场回读没过、卡用不上、第二台连不上……）：这一项一行都没判，记本次未跑，不记红（文件头「一项怎么判」）
      if [[ "$exit_code" == 78 ]] && ! replay_break_is pipeline-78-as-red; then
        record_part "${pipeline_owner[$name]}" pipeline "$name" 78 not-run 0 0 0 "$((SECONDS - started))" "运行条件不满足：放量入口退 78、没打 $name 的 CRASH_AMPLIFICATION_ITEM 行，这一项本次未跑（出路见入口打的那一句，日志 $log）"
        continue
      fi
      record_part "${pipeline_owner[$name]}" pipeline "$name" "$exit_code" red 0 1 unknown "$((SECONDS - started))" "放量入口退 $exit_code、没打 $name 的 CRASH_AMPLIFICATION_ITEM 行（收不回，日志 $log、错误警告 ${log%.log}.problems.log）"
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
  python3 - "$1" "$part_records" "$tier" "$2" "$run_directory/exit" "$(replay_break_is skip-collection-count && echo 1 || echo 0)" <<'PYTHON'
import collections, sys
plan_file, records_file, tier, tier_title, exit_file, skip_count = sys.argv[1:7]
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
unknown_fast_targets = set()   # 快档取不出复用数的测试目标（文件头「快档的复用数怎么取」），几项都点名的只算一次
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
    # 快档复用（文件头「快档的复用数怎么取」）：取得出的测试目标相加；取不出的不并进来、逐个记名；全是写死的 none（弄坏开关）照原样打
    fast_known = [int(row[7]) for row in by_part["fast-tier"] if row[7].isdigit()]
    fast_unknown_targets = sorted({row[2] for row in by_part["fast-tier"] if not row[7].isdigit()})
    if not by_part["fast-tier"]:
        fast_reused, fast_reuse_unknown = "-", "-"
    elif all(row[7] == "none" for row in by_part["fast-tier"]):
        fast_reused, fast_reuse_unknown = "none", "-"
    else:
        fast_reused = sum(fast_known) if fast_known or not fast_unknown_targets else "unknown"
        fast_reuse_unknown = len(fast_unknown_targets)
        totals["fast_tier_reused_known"] += sum(fast_known)
        unknown_fast_targets.update(fast_unknown_targets)
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
          f"fast_tier_reused={fast_reused} fast_tier_reuse_unknown={fast_reuse_unknown} pipeline_states={amp_states} pipeline_red={amp_red} pipeline_reused={amp_reused} "
          f"enumeration_cases={enumeration_cases} enumeration_red={enumeration_red} enumeration_reused={enumeration_reused} seconds={seconds}")
dispatched, collected = len(order), len(verdicts)
red = [item for item, (verdict, _) in verdicts.items() if verdict == "red"]
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」）由外面的 shell 写，红的项交给它
with open(exit_file + ".red-items", "w", encoding="utf-8") as handle:
    handle.write("".join(item + "\n" for item in red))
not_run = [item for item, (verdict, _) in verdicts.items() if verdict == "not-run"]
green = [item for item, (verdict, _) in verdicts.items() if verdict == "green"]
print(f"CRASH_POINT_REPLAY_TOTAL tier={tier} dispatched={dispatched} collected={collected} green={len(green)} red={len(red)} not_run={len(not_run)} "
      f"fast_tier_tests={totals['fast_tier_tests']} fast_tier_reused_known={totals['fast_tier_reused_known']} fast_tier_reuse_unknown={len(unknown_fast_targets)} "
      f"pipeline_states={totals['pipeline_states']} pipeline_reused_known={totals['pipeline_reused_known']} "
      f"pipeline_reuse_unknown={reuse_unknown} enumeration_cases={totals['enumeration_cases']} enumeration_reused={totals['enumeration_reused']} seconds={totals['seconds']}")
for item in red:
    for reason in verdicts[item][1]:
        print(f"  ✗ {item}：{reason}", file=sys.stderr)  # gate-lint:detail
for item in not_run:
    for reason in verdicts[item][1]:
        print(f"  · 本次未跑 {item}：{reason}", file=sys.stderr)
for item in red:   # 红的项里另有本次未跑的段（快档红之后没起的两段），一并点名
    for reason in dict.fromkeys(row[9] for row in parts[item] if row[4] == "not-run" and row[9]):
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
    fast_reuse_text = (f"复用 {totals['fast_tier_reused_known']} 块" + (f"（另有 {len(unknown_fast_targets)} 个测试目标取不出复用数：{'、'.join(sorted(unknown_fast_targets))}）"
                                                                       if unknown_fast_targets else ""))
    print(f"  ✓ 崩溃点重放{tier_title}：{len(green) + len(not_run)} 项都不红{not_run_text}；快档 {totals['fast_tier_tests']} 条用例{fast_reuse_text}、"
          f"放量现判 {totals['pipeline_states']} 态复用 {totals['pipeline_reused_known']} 态"
          f"{'（另有 %d 项取不出复用数）' % reuse_unknown if reuse_unknown else ''}、崩溃枚举用例 {totals['enumeration_cases']} 条复用 {totals['enumeration_reused']} 条；"
          f"派出 {dispatched} 项、收回 {collected} 项")
    code = 0
open(exit_file, "w").write(f"{code}\n")
PYTHON
}

# ── 积压跑法（文件头「积压跑法」）：积压登记表只经表的小程序读写，这里只起它、核它的输出、照它取到的件跑
BACKLOG_TABLE_PATH_IN_REPOSITORY="crates/singlefs-checker-tier/crash-full-run-backlog.tsv"   # 同 crates/singlefs-checker-tier/src/crash_full_run_backlog.rs 的 CRASH_FULL_RUN_BACKLOG_TABLE_PATH_IN_REPOSITORY，逐字相同
backlog_table=""; backlog_directory=""; backlog_today_key=""
backlog_take_sequences=(); backlog_take_groups=(); backlog_take_items=(); backlog_take_parts=(); backlog_take_keys=(); backlog_take_states=()
backlog_recorded=0; backlog_not_recorded=0; backlog_recorded_states=0; backlog_record_failures=0; backlog_parts_reported=0

# 交给 plan 的两条下限：给了才交（不给是小程序的默认）。弄坏开关 backlog-bounds-dropped 下一条都不交
backlog_backlog_bound_options() { [[ -n "$backlog_backlog_bound" ]] && ! replay_break_is backlog-bounds-dropped && printf '%s\n' --backlog-states-at-least "$backlog_backlog_bound"; return 0; }
backlog_total_bound_options() { [[ -n "$backlog_total_bound" ]] && ! replay_break_is backlog-bounds-dropped && printf '%s\n' --states-at-least "$backlog_total_bound"; return 0; }

backlog_program() { # <子命令> <选项…>：起表的小程序（编法见文件头「积压跑法」），标准输出原样交回、退出码照它
  ( cd "$repository_root" && bash "$replay_script_directory/../hosts/capped.sh" default \
      cargo run --release --quiet --offline -p singlefs-checker-tier --features verdict-store --bin crash_full_run_backlog -- "$@" )
}

backlog_read_today_key() { # 设 backlog_today_key；取不到交回 1（打 ✗ 与出路）
  local output
  if ! output="$(backlog_program today-judging-key)"; then
    printf '%s\n' "$output" | tail -n 5 >&2
    echo "  ✗ 积压登记表的小程序 today-judging-key 没取到今天的判法键（编不过或拒算，上面是它的输出尾部）" >&2
    echo "  → 怎么办：单跑 cargo run --release -p singlefs-checker-tier --features verdict-store --bin crash_full_run_backlog -- today-judging-key 看报错，修好再跑" >&2
    return 1
  fi
  backlog_today_key="$(sed -nE 's/^CRASH_FULL_RUN_BACKLOG_TODAYS_JUDGING_KEY\tkey=([0-9a-f]{64})$/\1/p' <<<"$output" | tail -n 1)"
  if [[ -z "$backlog_today_key" ]]; then
    echo "  ✗ 小程序 today-judging-key 的输出里没有「CRASH_FULL_RUN_BACKLOG_TODAYS_JUDGING_KEY<TAB>key=<64 位小写十六进制>」那一行" >&2
    echo "  → 怎么办：照 crates/singlefs-checker-tier/src/bin/crash_full_run_backlog.rs 文件头的写法对上 backlog_read_today_key，再跑" >&2
    return 1
  fi
}

# 起一趟 plan 并核它的输出（文件头「积压跑法」③的完整性闸）：核过了才把取到的件写进 <取件文件>（一行「序号<TAB>组<TAB>项<TAB>部分<TAB>键<TAB>态数」，照取件的次序），
# 另写 <取件文件>.summary 一行「游标<TAB>先跑那一组的态数<TAB>积压的态数<TAB>未数的件数<TAB>lower_bounds」；对不上交回 1、<取件文件> 空着。弄坏开关 backlog-plan-unchecked 下不核
backlog_plan() { # <取件文件> [<plan 的下限选项>…]
  local takes_file="$1" plan_exit
  shift
  : >"$takes_file"; rm -f -- "$takes_file.summary"
  if backlog_program plan --table "$backlog_table" --today-judging-key "$backlog_today_key" --current-milestone "$backlog_milestone" "$@" >"$takes_file.raw"; then plan_exit=0; else plan_exit=$?; fi
  python3 - "$takes_file.raw" "$takes_file" "$plan_exit" "$(replay_break_is backlog-plan-unchecked && echo 1 || echo 0)" <<'PYTHON'
import sys
raw_file, takes_file, plan_exit, unchecked = sys.argv[1:5]
GROUPS = ("new-or-judging-key-changed", "backlog")
PARTS = ("crash-point-replay", "crash-amplification")
lines = [line for line in open(raw_file, encoding="utf-8", errors="replace").read().split("\n") if line]
problems, takes, cursors, plans = [], [], [], []
for number, line in enumerate(lines, 1):
    fields = line.split("\t")
    values = dict(field.split("=", 1) for field in fields[1:] if "=" in field)
    if fields[0] == "CRASH_FULL_RUN_BACKLOG_CURSOR":
        cursors.append(values.get("sequence", ""))
    elif fields[0] == "CRASH_FULL_RUN_BACKLOG_TAKE":
        take = [values.get(name, "") for name in ("sequence", "group", "item", "part", "key", "states")]
        if (not take[0].isdigit() or take[1] not in GROUPS or not take[2] or take[3] not in PARTS or not take[4]
                or not (take[5].isdigit() or take[5] == "not-counted") or any(" " in value for value in take)):
            problems.append(f"第 {number} 行取件行字段不齐或认不出：{line[:200]}")
        takes.append(take)
    elif fields[0] == "CRASH_FULL_RUN_BACKLOG_PLAN":
        plans.append((number, values))
    else:
        problems.append(f"第 {number} 行认不出：{line[:200]}")
if plan_exit != "0":
    problems.append(f"小程序 plan 退 {plan_exit}")
if len(cursors) != 1 or not (cursors[0] == "none" or cursors[0].isdigit()):
    problems.append(f"游标那一行要恰好一行、序号是数或 none（实有 {len(cursors)} 行：{'、'.join(cursors) or '无'}）")
if len(plans) != 1 or plans[-1][0] != len(lines):
    problems.append(f"末行要恰好是一行 CRASH_FULL_RUN_BACKLOG_PLAN（实有 {len(plans)} 行）")
sequences = [take[0] for take in takes]
if len(set(sequences)) != len(sequences):
    problems.append(f"取件的序号有重复：{' '.join(sequences)}")
groups = [take[1] for take in takes]
first_backlog = next((index for index, group in enumerate(groups) if group == GROUPS[1]), len(groups))
if GROUPS[0] in groups[first_backlog:]:
    problems.append(f"先跑的那一组（{GROUPS[0]}）有件排在积压之后")
for group in GROUPS:
    numbers = [int(take[0]) for take in takes if take[1] == group and take[0].isdigit()]
    if numbers != sorted(numbers):
        problems.append(f"{group} 那一组没照表的次序：{' '.join(map(str, numbers))}")
summary = plans[-1][1] if plans else {}
if plans:
    def total(group):
        return sum(int(take[5]) for take in takes if take[1] == group and take[5].isdigit())
    expected = {"new_or_judging_key_changed_states": total(GROUPS[0]), "backlog_states": total(GROUPS[1]),
                "taken_without_a_count": sum(1 for take in takes if take[5] == "not-counted")}
    for name, value in expected.items():
        if summary.get(name) != str(value):
            problems.append(f"末行 {name}={summary.get(name, '（没有）')}，取件行加起来是 {value}")
    if summary.get("lower_bounds") not in ("both", "every-uncovered-row-below-the-bounds"):
        problems.append(f"末行 lower_bounds={summary.get('lower_bounds', '（没有）')} 认不出")
if problems and unchecked != "1":
    for problem in problems:
        print(f"  ✗ 积压登记表的小程序 plan 的输出：{problem}", file=sys.stderr)  # gate-lint:detail
    print(f"  ✗ 积压登记表的小程序 plan 的输出对不上（{len(problems)} 处），一件都不跑", file=sys.stderr)
    print(f"  → 怎么办：照上面逐处对 crates/singlefs-checker-tier/src/bin/crash_full_run_backlog.rs 的 plan 与它文件头的写法；那一趟的原样输出在 {raw_file}", file=sys.stderr)
    sys.exit(1)
with open(takes_file, "w", encoding="utf-8") as handle:
    handle.write("".join("\t".join(take) + "\n" for take in takes))
with open(takes_file + ".summary", "w", encoding="utf-8") as handle:
    handle.write("\t".join([cursors[0] if cursors else "none", summary.get("new_or_judging_key_changed_states", "0"), summary.get("backlog_states", "0"),
                            summary.get("taken_without_a_count", "0"), summary.get("lower_bounds", "?")]) + "\n")
PYTHON
}

# 只读用法的收尾（EXIT trap）：删这一趟的编译目录与临时目录
backlog_listing_finish() {
  local finish_exit=$?
  trap - EXIT
  if ! run_build_directory_end; then (( finish_exit != 0 )) || finish_exit=1; fi
  [[ -z "$backlog_directory" ]] || rm -rf -- "${backlog_directory:?}"
  exit "$finish_exit"
}

# --list-items --backlog（文件头「积压跑法」末段）：today-judging-key、三趟 plan，打「积压<TAB>…」几行；没列成交回 1
backlog_listing() {
  local plan_bounds=() backlog_bound=() total_bound=() word
  backlog_table="$repository_root/$BACKLOG_TABLE_PATH_IN_REPOSITORY"
  if ! backlog_directory="$(mktemp -d "${TMPDIR:-/tmp}/crash-point-replay-backlog-list.XXXXXX")"; then
    echo "  ✗ 建不了只读积压用法的临时目录（${TMPDIR:-/tmp}）" >&2
    echo "  → 怎么办：看 ${TMPDIR:-/tmp} 可不可写、盘满没满，修好再跑" >&2
    return 1
  fi
  trap backlog_listing_finish EXIT
  run_build_directory_begin "$backlog_directory" || return 1
  while IFS= read -r word; do backlog_bound+=("$word"); done < <(backlog_backlog_bound_options)
  while IFS= read -r word; do total_bound+=("$word"); done < <(backlog_total_bound_options)
  plan_bounds=(${backlog_bound[@]+"${backlog_bound[@]}"} ${total_bound[@]+"${total_bound[@]}"})
  if replay_break_is backlog-listing-appends; then   # 弄坏开关：只读用法照积压跑法那样先追加少登的件（改了表）
    bash "$repository_root/scripts/crash/crash-amplification.sh" --list-items >"$backlog_directory/amplification-items.txt" 2>/dev/null
    backlog_program append-missing --table "$backlog_table" --merged-items "$repository_root/.claude/gate.d/crash-point-replay-items.tsv" \
      --amplification-items "$backlog_directory/amplification-items.txt" --registered-in "$backlog_milestone" --provenance "只读用法追加" >/dev/null
  fi
  backlog_read_today_key || return 1
  backlog_plan "$backlog_directory/takes-run.tsv" ${plan_bounds[@]+"${plan_bounds[@]}"} || return 1
  backlog_plan "$backlog_directory/takes-backlog-bound.tsv" ${backlog_bound[@]+"${backlog_bound[@]}"} --states-at-least 0 || return 1
  backlog_plan "$backlog_directory/takes-total-bound.tsv" --backlog-states-at-least 0 ${total_bound[@]+"${total_bound[@]}"} || return 1
  python3 - "$backlog_today_key" "$backlog_milestone" "${backlog_bound[1]:-}" "${total_bound[1]:-}" \
    "$backlog_directory/takes-run.tsv" "$backlog_directory/takes-backlog-bound.tsv" "$backlog_directory/takes-total-bound.tsv" <<'PYTHON'
import sys
key, milestone, backlog_bound, total_bound, run_file, backlog_file, total_file = sys.argv[1:8]


def read(path):
    takes = [line.rstrip("\n").split("\t") for line in open(path, encoding="utf-8") if line.strip()]
    cursor, new_states, backlog_states, not_counted, bounds = open(path + ".summary", encoding="utf-8").read().rstrip("\n").split("\t")
    return {"takes": takes, "cursor": cursor, "new_states": int(new_states), "backlog_states": int(backlog_states), "not_counted": int(not_counted), "bounds": bounds}


def group(plan, name):
    return [take for take in plan["takes"] if take[1] == name]


def not_counted(takes):
    return sum(1 for take in takes if take[5] == "not-counted")


run, backlog_only, total_only = read(run_file), read(backlog_file), read(total_file)
backlog_text = f"{backlog_bound} 态" if backlog_bound else "小程序的默认（出口 10 的 50 亿态）"
total_text = f"{total_bound} 态" if total_bound else "小程序的默认（出口 10 的 100 亿态）"
print(f"积压\t今天的判法键 {key}（镜像判定那一组判法单元在这份小程序里的指纹）")
cursor_take = next((take for plan in (run, backlog_only, total_only) for take in plan["takes"] if take[0] == run["cursor"]), None)
if run["cursor"] == "none":
    print("积压\t游标：没有（今天的判法键下每一行都全绿）")
elif cursor_take:
    print(f"积压\t游标：序号 {run['cursor']}（{cursor_take[2]} {cursor_take[3]} {cursor_take[4]}）")
else:
    print(f"积压\t游标：序号 {run['cursor']}")
new = group(run, "new-or-judging-key-changed")
print(f"积压\t先跑这一里程碑（{milestone}）新增与判法键变了的：{len(new)} 件、{run['new_states']} 态（未数 {not_counted(new)} 件，照取、算 0）"
      + (f"：序号 {'、'.join(take[0] for take in new)}" if new else ""))
for label, bound_text, plan, total in (("到积压下限", backlog_text, backlog_only, False), ("到合计下限", total_text, total_only, True)):
    backlog = group(plan, "backlog")
    reached = "够了" if plan["bounds"] == "both" else "不够：没罩住的积压全取也到不了"
    states = plan["new_states"] + plan["backlog_states"] if total else plan["backlog_states"]
    if backlog:
        where = ("从游标起" if not total else "") + f"跑到序号 {max(int(take[0]) for take in backlog)}"
    else:
        where = "积压一件都不用跑"
    print(f"积压\t{label} {bound_text}：{where}（积压 {len(backlog)} 件；{'新增与积压合计' if total else '积压'} {states} 态，{reached}；未数 {not_counted(plan['takes'])} 件，照取、算 0）")
backlog = group(run, "backlog")
print(f"积压\t积压跑法一趟（两条下限都够才停：积压 {backlog_text}、合计 {total_text}）：取 {len(run['takes'])} 件（先跑 {len(new)} 件、积压 {len(backlog)} 件）"
      + (f"，积压跑到序号 {max(int(take[0]) for take in backlog)}" if backlog else "") + f"；lower_bounds={run['bounds']}（只读：没核表与合表、ITEMS 对不对得上，那一核在积压跑法开跑时与门禁 code-tooling 的 crash-full-run-backlog 格）")
PYTHON
}

# 重型·积压的计划（文件头「积压跑法」①到④）：核表、取件、写 $plan_file，填取到的件、selected_items 与 named_items；退不了的直接退（收尾由 EXIT trap 管）
backlog_prepare_plan() {
  local items_list="$run_directory/amplification-items.txt" output check_exit takes_file="$run_directory/backlog-takes.tsv"
  local sequence group item part key states index word bound_options=()
  backlog_table="$repository_root/$BACKLOG_TABLE_PATH_IN_REPOSITORY"
  backlog_directory="$run_directory"
  if ! bash "$repository_root/scripts/crash/crash-amplification.sh" --list-items >"$items_list" 2>"$items_list.err"; then
    echo "  ✗ 崩溃放量入口 --list-items 没列成（$(tail -n 2 "$items_list.err" | tr '\n' ' ')），积压登记表核不了，什么都不跑" >&2
    echo "  → 怎么办：单跑 bash scripts/crash/crash-amplification.sh --list-items 看报错，修好再跑" >&2
    exit 2
  fi
  if ! replay_break_is backlog-table-unchecked; then
    if ! output="$(backlog_program append-missing --table "$backlog_table" --merged-items "$repository_root/.claude/gate.d/crash-point-replay-items.tsv" \
         --amplification-items "$items_list" --registered-in "$backlog_milestone" \
         --provenance "积压跑法开跑时追加（$(TZ=Asia/Tokyo date +%F)）：合表加 ITEMS 展开里有、表里没有；等这一件跑完那一趟的计数行再填")"; then
      printf '%s\n' "$output" | tail -n 5 >&2
      echo "  ✗ 积压登记表的小程序 append-missing 没做成（上面是它的输出尾部），什么都不跑" >&2
      echo "  → 怎么办：照它报的改合表、ITEMS 或表（表的写法在它的表头），再跑" >&2
      exit 2
    fi
    echo "  · 积压登记表：$(sed -nE 's/^CRASH_FULL_RUN_BACKLOG_APPENDED\tparts=([0-9]+)$/合表加 ITEMS 展开里有、表里没有的追加了 \1 件（归属 '"$backlog_milestone"'，态数未数）/p' <<<"$output")"
    if output="$(backlog_program check --table "$backlog_table" --merged-items "$repository_root/.claude/gate.d/crash-point-replay-items.tsv" --amplification-items "$items_list")"; then
      check_exit=0
    else
      check_exit=$?
    fi
    if (( check_exit != 0 )); then
      printf '%s\n' "$output" >&2
      echo "  ✗ 积压登记表 $BACKLOG_TABLE_PATH_IN_REPOSITORY 核不过（小程序 check 退 $check_exit，上面逐处列了），不取件、什么都不跑" >&2
      echo "  → 怎么办：照上面每一处改表（多登、次序不对、全绿而态数没数、跑过而判法键没记），或改合表与 ITEMS；表的写法在它的表头，再跑" >&2
      exit 2
    fi
  fi
  backlog_read_today_key || exit 1
  while IFS= read -r word; do bound_options+=("$word"); done < <(backlog_backlog_bound_options; backlog_total_bound_options)
  backlog_plan "$takes_file" ${bound_options[@]+"${bound_options[@]}"} || exit 1
  # 取到的件照取件的次序（弄坏开关 backlog-table-order 下照表的序号重排）
  while IFS=$'\t' read -r sequence group item part key states; do
    [[ -n "$sequence" ]] || continue
    backlog_take_sequences+=("$sequence"); backlog_take_groups+=("$group"); backlog_take_items+=("$item")
    backlog_take_parts+=("$part"); backlog_take_keys+=("$key"); backlog_take_states+=("$states")
  done < <(if replay_break_is backlog-table-order; then LC_ALL=C sort -t $'\t' -k1,1n -- "$takes_file"; else cat -- "$takes_file"; fi)
  if (( ${#backlog_take_sequences[@]} == 0 )); then
    echo "  ! 积压登记表在今天的判法键下没有要跑的件（游标 $(cut -f1 "$takes_file.summary")），这一趟一件都没跑：本次未跑，不是通过" >&2
    exit 77
  fi
  for index in "${!backlog_take_sequences[@]}"; do
    item="${backlog_take_items[index]}"; part="${backlog_take_parts[index]}"; key="${backlog_take_keys[index]}"
    if [[ -z "${item_cases[$item]+set}" ]] \
       || { [[ "$part" == crash-point-replay ]] && [[ " ${item_cases[$item]} " != *" $key "* ]]; } \
       || { [[ "$part" == crash-amplification ]] && [[ " ${item_pipeline_items[$item]} " != *" $key "* ]]; }; then
      echo "  ✗ 积压登记表取到的序号 ${backlog_take_sequences[index]}（$item $part $key）不在合表那一项里，什么都不跑" >&2
      echo "  → 怎么办：表与合表对不上，先跑一趟 check 看哪几处（门禁 code-tooling 的 crash-full-run-backlog 格同一判法），照它改表或合表再跑" >&2
      exit 1
    fi
  done
  # 计划：取到的件所在的项的快档测试目标在前，再照取件的次序每件一行
  : >"$plan_file"
  for item in "${backlog_take_items[@]}"; do
    [[ " ${selected_items[*]-} " == *" $item "* ]] && continue
    selected_items+=("$item")
    for word in ${item_light_targets[$item]}; do printf '%s\tfast-tier\t%s\n' "$item" "$word" >>"$plan_file"; done
  done
  for index in "${!backlog_take_sequences[@]}"; do
    if [[ "${backlog_take_parts[index]}" == crash-point-replay ]]; then
      printf '%s\tenumeration-full\t%s\n' "${backlog_take_items[index]}" "${backlog_take_keys[index]}" >>"$plan_file"
    else
      printf '%s\tpipeline-full\t%s\n' "${backlog_take_items[index]}" "${backlog_take_keys[index]}" >>"$plan_file"
    fi
  done
  named_items=("${selected_items[@]}")   # 上次红的清单只换这一趟跑到的项（同 --item）
  echo "  · 积压登记表取件：$(wc -l <"$takes_file") 件（游标 $(cut -f1 "$takes_file.summary")；今天的判法键 $backlog_today_key），照取件的次序跑：$(awk -F'\t' '{ printf "%s%s %s %s", (NR > 1 ? "、" : ""), $1, $2, $5 }' "$takes_file")"
}

# 用例日志里 <行首> 那几行的 states= 之和：打「<和> <行数>」（日志不在打「0 0」）
backlog_states_in_the_case_log() { # <用例日志> <行首>
  [[ -n "$1" && -f "$1" ]] || { echo "0 0"; return 0; }
  awk -v prefix="$2" '{ line = $0; sub(/^[ \t]+/, "", line); count = split(line, words, /[ \t]+/); if (words[1] != prefix) next
                         for (index_of_word = 2; index_of_word <= count; index_of_word++) if (words[index_of_word] ~ /^states=[0-9]+$/) { total += substr(words[index_of_word], 8); lines++ } }
                       END { printf "%.0f %d\n", total, lines }' "$1"
}

# 跑完一件之后回写那一行（文件头「积压跑法」⑤），打 CRASH_POINT_REPLAY_BACKLOG_PART
backlog_record_the_part() { # <下标> <这一件的记录行（parts.tsv 的最后一行）> <这一件的日志>
  local index="$1" record_line="$2" log="$3" sequence group item part key planned_states wanted_kind today case_log
  local record_item record_kind record_object record_exit state record_states record_reused
  local verdict=not-run recorded=no states="-" reason="-" progress="" states_text="" counted_states="" counted_lines=0 counted_source="" record_output
  local record_arguments=()
  sequence="${backlog_take_sequences[index]}"; group="${backlog_take_groups[index]}"; item="${backlog_take_items[index]}"
  part="${backlog_take_parts[index]}"; key="${backlog_take_keys[index]}"; planned_states="${backlog_take_states[index]}"
  today="$(TZ=Asia/Tokyo date +%F)"
  backlog_parts_reported=$((backlog_parts_reported + 1))
  wanted_kind=pipeline; [[ "$part" == crash-point-replay ]] && wanted_kind=enumeration
  IFS=$'\t' read -r record_item record_kind record_object record_exit state record_states _ record_reused _ <<<"$record_line"
  if [[ "$record_item" != "$item" || "$record_kind" != "$wanted_kind" || "$record_object" != "$key" ]]; then
    state=lost; reason="这一件没记下结果（收不回），不回写"
  fi
  if [[ "$state" == green || "$state" == red ]]; then
    verdict="$state"
    if [[ "$part" == crash-point-replay ]]; then
      case_log="$(sed -nE 's/^    全量日志：(.*)；发现日志：.*$/\1/p' -- "$log" 2>/dev/null | tail -n 1)"
      counted_source=CRASH_CASE_BLOCKS
      read -r counted_states counted_lines <<<"$(backlog_states_in_the_case_log "$case_log" CRASH_CASE_BLOCKS)"
      if [[ "$counted_lines" == 0 ]]; then
        counted_source=CRASH_POINT_REPLAY_PARALLEL_FINISHED
        read -r counted_states counted_lines <<<"$(backlog_states_in_the_case_log "$case_log" CRASH_POINT_REPLAY_PARALLEL_FINISHED)"
      fi
      [[ "$counted_lines" =~ ^[1-9][0-9]*$ && "$counted_states" =~ ^[0-9]+$ ]] \
        && states_text="$counted_states；$today crash-point-replay.sh --backlog 那一趟用例日志里 $counted_source 行的 states= 之和（$counted_lines 行）"
      progress="红：门禁 checker-tier-crash-point-replay --full --item $key 退 $record_exit（$today 积压跑法；日志 $(basename -- "$log")）"
    else
      if [[ "$record_states" =~ ^[0-9]+$ && "$record_reused" =~ ^[0-9]+$ ]]; then
        counted_states=$((record_states + record_reused))
        states_text="$counted_states；$today crash-point-replay.sh --backlog 那一趟 CRASH_AMPLIFICATION_ITEM 行的 states= 加 reused="
      fi
      progress="红：崩溃放量 $key 判红（CRASH_AMPLIFICATION_ITEM 退 $record_exit；$today 积压跑法；日志 $(basename -- "$log")）"
    fi
    if [[ "$verdict" == green ]] || replay_break_is backlog-red-as-green; then progress="全绿"; fi
  elif [[ "$state" == not-run ]] && replay_break_is backlog-not-run-recorded; then
    progress="全绿"   # 弄坏开关：本次未跑的也照全绿回写
  fi
  [[ "$state" == not-run && "$reason" == - ]] && reason="本次未跑，不回写"
  if [[ -z "$progress" ]]; then
    :
  elif [[ -z "$states_text" && "$planned_states" == not-counted && "$progress" == 全绿 ]]; then
    reason="判绿而态数数不出、表里这一行原来也未数（全绿而未数是小程序check判的一处问题），不回写"
  else
    record_arguments=(record --table "$backlog_table" --sequence "$sequence" --judging-key "$backlog_today_key" --progress "$progress"
                      --last-run "$today；crash-point-replay.sh --backlog --milestone $backlog_milestone 那一趟")
    [[ -n "$states_text" ]] && record_arguments+=(--states "$states_text")
    if replay_break_is backlog-not-recorded; then
      recorded=yes   # 弄坏开关：照说回写了，其实没起 record
    elif record_output="$(backlog_program "${record_arguments[@]}")" && grep -qxF -- "CRASH_FULL_RUN_BACKLOG_RECORDED"$'\t'"sequence=$sequence" <<<"$record_output"; then
      recorded=yes
    else
      reason="小程序record没写成"; backlog_record_failures=$((backlog_record_failures + 1))
      echo "  ✗ 积压登记表：序号 $sequence（$item $part $key）的 record 没写成（输出：${record_output//$'\n'/｜}）" >&2
      echo "  → 怎么办：单跑 cargo run --release -p singlefs-checker-tier --features verdict-store --bin crash_full_run_backlog -- ${record_arguments[*]} 看报错；表里这一行还是跑之前的样子" >&2
    fi
  fi
  if [[ "$recorded" == yes ]]; then
    backlog_recorded=$((backlog_recorded + 1)); reason="-"
    if [[ -n "$states_text" ]]; then states="$counted_states"; backlog_recorded_states=$((backlog_recorded_states + counted_states)); fi
  else
    backlog_not_recorded=$((backlog_not_recorded + 1))
  fi
  echo "CRASH_POINT_REPLAY_BACKLOG_PART sequence=$sequence group=$group item=$item part=$part key=$key verdict=$verdict recorded=$recorded states=$states reason=$reason"
}

run_backlog_parts() { # 照取件的次序逐件跑、逐件回写（文件头「积压跑法」④⑤）
  local index item part key log one_plan
  for index in "${!backlog_take_sequences[@]}"; do
    item="${backlog_take_items[index]}"; part="${backlog_take_parts[index]}"; key="${backlog_take_keys[index]}"
    if [[ "$part" == crash-point-replay ]]; then
      run_enumeration_part "$item" "$key"
      log="$run_directory/enumeration.$item.${key#crash-case:}.log"
    else
      one_plan="$run_directory/backlog-part.${backlog_take_sequences[index]}.tsv"
      printf '%s\tpipeline-full\t%s\n' "$item" "$key" >"$one_plan"
      run_pipeline_part "$one_plan"
      log="$run_directory/pipeline.$key.log"
    fi
    backlog_record_the_part "$index" "$(tail -n 1 -- "$part_records")" "$log"
  done
}

# 末行 CRASH_POINT_REPLAY_BACKLOG_TOTAL（文件头「积压跑法」）：快档红没起的件逐件补一行；record 没写成、件数对不上的改退 1
backlog_summarize() {
  local index taken="${#backlog_take_sequences[@]}" new_count=0 group
  for ((index = backlog_parts_reported; index < taken; index++)); do
    echo "CRASH_POINT_REPLAY_BACKLOG_PART sequence=${backlog_take_sequences[index]} group=${backlog_take_groups[index]} item=${backlog_take_items[index]} part=${backlog_take_parts[index]} key=${backlog_take_keys[index]} verdict=not-run recorded=no states=- reason=快档红，这一件没起"
    backlog_not_recorded=$((backlog_not_recorded + 1))
  done
  for group in ${backlog_take_groups[@]+"${backlog_take_groups[@]}"}; do [[ "$group" == new-or-judging-key-changed ]] && new_count=$((new_count + 1)); done
  echo "CRASH_POINT_REPLAY_BACKLOG_TOTAL milestone=$backlog_milestone taken=$taken new_or_judging_key_changed=$new_count backlog=$((taken - new_count)) recorded=$backlog_recorded not_recorded=$backlog_not_recorded recorded_states=$backlog_recorded_states record_failures=$backlog_record_failures"
  if (( backlog_record_failures > 0 || backlog_recorded + backlog_not_recorded != taken )); then
    echo "  ✗ 积压登记表回写：取到 $taken 件，回写 $backlog_recorded 件、没回写 $backlog_not_recorded 件，其中 record 没写成 $backlog_record_failures 件" >&2
    echo "  → 怎么办：看上面 record 没写成的那几行；表里那几行还是跑之前的样子，修好重跑同一条命令（判过的块照库复用）" >&2
    echo 1 >"$run_directory/exit"
  else
    echo "  · 积压登记表回写：取到 $taken 件（先跑 $new_count 件、积压 $((taken - new_count)) 件），回写 $backlog_recorded 件、共 $backlog_recorded_states 态；没回写 $backlog_not_recorded 件（原因见上面 recorded=no 那几行）"
  fi
}

run_selftest() {
  local work root calls cells=0 failures=0 exit_code
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-point-replay-selftest.XXXXXX")"
  trap 'rm -rf -- "${work:?}"' EXIT
  root="$work/root"; calls="$work/calls"
  mkdir -p "$root/.claude/gate.d" "$root/scripts" "$root/scripts/crash" "$root/scripts/gate" "$root/crates/singlefs-checker-tier/tests" "$work/bin" "$work/home"
  printf '%s\n' "$SELFTEST_MARK_CONTENT" >"$work/mark"
  # 造出来的小仓：三项（两段都有、只有崩溃点重放、只有放量），合表核法照真的那一份判
  printf '%s\n' "# 自证的合表" $'alpha\tcrash-case:alpha\talpha-amp\talpha\t# 两段都有' $'beta\tcrash-case:beta\t-\tbeta\t# 只有崩溃点重放' \
    $'gamma\t-\tgamma-amp\tgamma_amp\t# 只有放量' >"$root/.claude/gate.d/crash-point-replay-items.tsv"
  printf '%s\n' "# 自证的登记表" $'crash-case:alpha\tcrates/\ttest=singlefs-checker-tier:alpha:alpha_full\t# 自证' \
    $'crash-case:beta\tcrates/\ttest=singlefs-checker-tier:beta:beta_full\t# 自证' >"$root/.claude/gate.d/stage-inputs.tsv"
  # 双机那几格（文件头「双机」）临时换成 crash-case:beta 带 features= 的一份（beta 那一项算经给定状态引擎的），跑完换回
  cp -- "$root/.claude/gate.d/stage-inputs.tsv" "$work/stage-inputs-plain.tsv"
  sed 's/^\(crash-case:beta\t.*test=singlefs-checker-tier:beta:beta_full\)\t/\1 features=verdict-store\t/' "$work/stage-inputs-plain.tsv" >"$work/stage-inputs-given-states.tsv"
  : >"$root/crates/singlefs-checker-tier/tests/alpha.rs"; : >"$root/crates/singlefs-checker-tier/tests/beta.rs"; : >"$root/crates/singlefs-checker-tier/tests/gamma_amp.rs"
  # 按段登记文件：一行登记（alpha 那一项的流 alpha_flow 第 0 段，五列），segments-registry-refused 那一格临时换成核不过的一份、跑完换回
  printf '%s\n' "# 自证的按段登记" "alpha	alpha_flow	0	7	$(printf '0123456789abcdef%.0s' 1 2 3 4)" >"$root/.claude/gate.d/crash-point-replay-segments.tsv"
  cat >"$root/scripts/crash/crash-amplification.sh" <<'STUB'
#!/usr/bin/env bash
# 假的崩溃放量入口（自证用）：ITEMS 给合表核法读；起它时记一行参数，按 --item（没给就 ITEMS 全部）逐项打 CRASH_AMPLIFICATION_ITEM 行
ITEMS=(
  $'alpha-amp\talpha\talpha_amplified\t全量'
  $'gamma-amp\tgamma_amp\tgamma_amplified\t全量'
)
config_summary() { local file="${SINGLEFS_MULTI_HOST_CONFIG:-}"; if [[ -f "$file" ]]; then printf 'across=%s peer=%s cards=%s' "$(grep -c '^ENABLE_ACROSS_MACHINES=1' "$file")" "$(grep -c 'peer-selftest' "$file")" "$(grep -c 'peer:' "$file")"; else printf 'none'; fi; }
if [[ "${1:-}" == --list-items ]]; then   # 积压跑法那几格：照真的入口逐行打 ITEMS（分项名在第一列），记一行、什么都不判
  echo 'pipeline-list-items' >>"${STUB_CALLS:?}"
  printf '%s\n' "${ITEMS[@]}"
  exit 0
fi
printf 'pipeline-entry %s\n' "$*" >>"${STUB_CALLS:?}"
printf 'pipeline-env config=[%s]\n' "$(config_summary)" >>"$STUB_CALLS"
printf 'pipeline-env segments=[%s]\n' "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" >>"$STUB_CALLS"
printf 'pipeline-env slice-states=[%s]\n' "${SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES-unset}" >>"$STUB_CALLS"
printf 'pipeline-env write-only=[%s]\n' "${SINGLEFS_CRASH_LIBRARY_WRITE_ONLY-unset}" >>"$STUB_CALLS"
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
if [[ -n "${STUB_AMP_PERSISTENT:-}" ]]; then   # 照引擎按键复用：库是 --library 给的，没给是 <家目录>/crash-state-verdicts；判过的分项在库里留一份文件，再判就报复用、现判 0 态
  persistent_library="${SINGLEFS_CRASH_AMPLIFICATION_HOME:?}/crash-state-verdicts"; previous=""
  for argument in "$@"; do [[ "$previous" == --library ]] && persistent_library="$argument"; previous="$argument"; done
  echo "pipeline-persistent-library $persistent_library" >>"$STUB_CALLS"
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
  states=10; reused=4
  if [[ -n "${STUB_AMP_PERSISTENT:-}" ]]; then
    unit_key=""   # STUB_AMP_UNITS 设了：键另带这一项的单元文件（<STUB_AMP_UNITS>/<分项>）的内容摘要（复用三问那一格用）
    [[ -n "${STUB_AMP_UNITS:-}" ]] && unit_key=".$(sha256sum "$STUB_AMP_UNITS/$item" | cut -c1-64)"
    if [[ -e "$persistent_library/$item$unit_key.judged" ]]; then states=0; reused=10; else states=10; reused=0; mkdir -p -- "$persistent_library" && : >"$persistent_library/$item$unit_key.judged"; fi
  fi
  echo "CRASH_AMPLIFICATION_ITEM item=$item exit=$item_exit states=$states reused=$reused red=$red lines=1 expand_up_to=6 exhaustive=true verdict=$verdict seconds=0 ignored_points=0 states_skipped_by_ignore=0"
done
exit "${STUB_AMP_EXIT:-$exit_code}"
STUB
  cat >"$root/.claude/gate.d/checker-tier-crash-point-replay.sh" <<'STUB'
#!/usr/bin/env bash
# 假的门禁 checker-tier-crash-point-replay（自证用）：记一行参数；STUB_CRASH_POINT_REPLAY_REUSED 设了就给每条点名的用例打一行复用；退 STUB_CRASH_POINT_REPLAY_EXIT
config_summary() { local file="${SINGLEFS_MULTI_HOST_CONFIG:-}"; if [[ -f "$file" ]]; then printf 'across=%s peer=%s cards=%s' "$(grep -c '^ENABLE_ACROSS_MACHINES=1' "$file")" "$(grep -c 'peer-selftest' "$file")" "$(grep -c 'peer:' "$file")"; else printf 'none'; fi; }
printf 'enumeration-gate %s\n' "$*" >>"${STUB_CALLS:?}"
printf 'enumeration-env config=[%s]\n' "$(config_summary)" >>"$STUB_CALLS"
# 积压跑法那几格（STUB_CASE_LOG_DIRECTORY 设了）：给每条点名的用例照真的那一道写一份用例日志（CRASH_CASE_BLOCKS 行带 states=37），打「开跑」与「全量日志：」那两行
if [[ -n "${STUB_CASE_LOG_DIRECTORY:-}" ]]; then
  previous_word=""
  for word in "$@"; do
    if [[ "$previous_word" == --item ]]; then
      echo "CRASH_CASE_BLOCKS blocks=4 reused=1 fresh=3 red=0 refused=0 states=37" >"$STUB_CASE_LOG_DIRECTORY/${word#crash-case:}.log"
      echo "  · $word 开跑（自证）"
      echo "    全量日志：$STUB_CASE_LOG_DIRECTORY/${word#crash-case:}.log；发现日志：$STUB_CASE_LOG_DIRECTORY/${word#crash-case:}.findings"
    fi
    previous_word="$word"
  done
fi
previous=""
for argument in "$@"; do
  [[ "$previous" == --item && -n "${STUB_CRASH_POINT_REPLAY_REUSED:-}" ]] && echo "  · $argument 复用：全部块都有键相同的判定（自证）"
  previous="$argument"
done
exit "${STUB_CRASH_POINT_REPLAY_EXIT:-0}"
STUB
  cat >"$root/scripts/gate/admission.py" <<'STUB'
# 假的准入模块（自证用）：记一行参数；这一份不该再调它（用例一级没有全绿标记可删），调了就在 full-heavy 那一格判红
import os, sys
open(os.environ["STUB_CALLS"], "a").write("admission " + " ".join(sys.argv[1:]) + "\n")
STUB
  cat >"$work/bin/cargo" <<'STUB'
#!/usr/bin/env bash
# 假 cargo（自证用）：每次调记一行（参数、这一趟的显卡先后、计算类、是不是在清场包装之下、分片的段一趟判几个态、编译并行度、按段登记文件的路径）；按 --test 点名的目标打 libtest 的样子
printf 'cargo %s cards=[%s] classes=[%s] quiesced=[%s] slice-states=[%s] jobs=[%s] segments=[%s]\n' "$*" "${GPU_CARDS_BY_PRIORITY-unset}" "${GPU_COMPUTATION_CLASSES-unset}" "${GPU_QUIESCE_HOLDER:+yes}" \
  "${SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES-unset}" "${CARGO_BUILD_JOBS-unset}" "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" >>"${STUB_CALLS:?}"
config_summary() { local file="${SINGLEFS_MULTI_HOST_CONFIG:-}"; if [[ -f "$file" ]]; then printf 'across=%s peer=%s cards=%s' "$(grep -c '^ENABLE_ACROSS_MACHINES=1' "$file")" "$(grep -c 'peer-selftest' "$file")" "$(grep -c 'peer:' "$file")"; else printf 'none'; fi; }
printf 'cargo-env config=[%s]\n' "$(config_summary)" >>"$STUB_CALLS"
# 编译目录那几格（run-target-*）：记下收到的 CARGO_TARGET_DIR、往里写一份产物；给了 STUB_SLEEP 每一趟睡这么多秒（被 TERM 停那一格）
if [[ -n "${STUB_TARGET_LOG:-}" ]]; then
  printf '%s\n' "${CARGO_TARGET_DIR-unset}" >>"$STUB_TARGET_LOG"
  [[ -z "${CARGO_TARGET_DIR:-}" ]] || { mkdir -p "$CARGO_TARGET_DIR/release" && echo built >"$CARGO_TARGET_DIR/release/artifact"; }
fi
[[ -z "${STUB_SLEEP:-}" ]] || sleep "$STUB_SLEEP"
# 积压跑法那几格：cargo run … --bin crash_full_run_backlog -- <子命令…> 交给假的小程序（STUB_BACKLOG_PROGRAM），退它的退出码
if [[ " $* " == *" --bin crash_full_run_backlog "* ]]; then
  backlog_arguments=(); after_separator=0
  for argument in "$@"; do
    if (( after_separator )); then backlog_arguments+=("$argument"); elif [[ "$argument" == -- ]]; then after_separator=1; fi
  done
  exec python3 "${STUB_BACKLOG_PROGRAM:?}" "${backlog_arguments[@]}"
fi
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
if [[ " ${STUB_REUSE_TARGETS:-} " == *" $target "* ]]; then   # 走判定库的用例：跑完那一行带库里复用的块（resumed_slices=3）与这一趟判的块
  printf 'running 2 tests\nCRASH_POINT_REPLAY_PARALLEL_FINISHED states=80 slices=8 worker_threads=2 configured_worker_threads=2 worker_threads_source=environment_variable resumed_slices=3 freshly_run_slices=5 progress_file_after_completion=none elapsed_seconds=0.1 judge=cpu\ntest first ... ok\ntest second ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'; exit 0
fi
[[ -n "${STUB_FLOW_LINE:-}" && "$target" == alpha ]] && echo 'CRASH_FLOW_COVERAGE entry=selftest_entry step=UnitWrite segment=0 expansion=expanded domain=quick'   # flow-coverage-table 那一格
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
  # 双机那几格（文件头「双机」）的多机配置：开了双机、键齐（判法带 SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1，不连第二台）；开了双机而缺键（判法判不过）
  printf '%s\n' PEER_SSH_HOST=peer-selftest PEER_REPOSITORY_DIRECTORY=/nonexistent/peer-tree PEER_CARGO_BIN_DIRECTORY=/nonexistent/bin PEER_MEMORY_CAP=4G \
    QUIESCE_STOP_COMMAND= QUIESCE_STOPPED_CHECK_COMMAND= QUIESCE_START_COMMAND= QUIESCE_STARTED_CHECK_COMMAND= ENABLE_ACROSS_MACHINES=1 >"$work/multi-host-two-hosts.env"
  printf '%s\n' ENABLE_ACROSS_MACHINES=1 PEER_SSH_HOST=peer-selftest >"$work/multi-host-two-hosts-broken.env"
  # 只用本机那几格（--local-only）的多机配置：双机与 GPU 都开，显卡先后里第二台也点了名（派生的那一份要去掉第二台的键与它的卡、留着本机的卡与清场）
  { cat "$work/multi-host-gpu-off.env"; printf '%s\n' PEER_SSH_HOST=peer-selftest PEER_REPOSITORY_DIRECTORY=/nonexistent/peer-tree PEER_CARGO_BIN_DIRECTORY=/nonexistent/bin PEER_MEMORY_CAP=4G \
    ENABLE_ACROSS_MACHINES=1 ENABLE_GPU=1 'GPU_CARDS_BY_PRIORITY="local:0:8G peer:0:8G"' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="local:0:24G peer:0:16G"'; } >"$work/multi-host-two-hosts-gpu.env"
  # 双机驱动是真的 scripts/crash/crash-amplification-two-hosts.sh，只换掉它本机那一趟与第二台那一趟（它文件头「只供测试」的两个变量，entry 每一格都设上）；
  #   它算被测源码摘要的树也换成这里的小仓（同一段的 CRASH_AMPLIFICATION_TWO_HOSTS_SOURCE_ROOT：真仓的 crates/ 别的会话正改着，双机那几格不因此退 77）
  cat >"$work/two-hosts-local.sh" <<'STUB'
#!/usr/bin/env bash
# 假的双机驱动本机那一趟（自证用，CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN）：调法 <share> <库目录> <日志> [<附加环境 键=值>…]，环境由双机驱动照真的那一趟组。
# 记一行进 STUB_CALLS（份、IMPORT 指到哪、那里有没有取回的库、分片的段一趟判几个态）；带 IMPORT 的那一趟照 libtest 的样子打两条都过，没带的打一行本次未跑（表归第二台判）
share="$1" library="$2" log="$3"
import="${SINGLEFS_CRASH_AMPLIFICATION_IMPORT-unset}" fetched=no
[[ "$import" != unset && -f "$import/CURRENT" ]] && fetched=yes
printf 'two-hosts-local share=%s import=%s fetched=%s library=%s slice-states=[%s]\n' "$share" "$import" "$fetched" "$library" "${SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES-unset}" >>"${STUB_CALLS:?}"
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
# --fetch 点名的目录建在 MULTI_HOST_RUN_FETCH_DIRECTORY 下（带 CURRENT，当成第二台判完的库）。STUB_PEER_EXIT=255：连不上，什么都不跑、不取回，退 255。
# 第二台只看得见命令里送过去的：命令里 SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=<值> 那一词另记一行 peer-env slice-states=[<值>]（没送记 unset）
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
share="" slice_states=unset
for word in "$@"; do
  [[ "$word" == SINGLEFS_CRASH_AMPLIFICATION_SHARE=* ]] && share="${word#*=}"
  [[ "$word" == SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=* ]] && slice_states="${word#*=}"
done
echo "peer-env slice-states=[$slice_states]" >>"${STUB_CALLS:?}"
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
  unset STUB_AMP_PERSISTENT STUB_REUSE_TARGETS
  unset GATE_NOT_RUN_FILE STUB_RUN_CONDITION_TARGETS STUB_PEER_EXIT STUB_RED_TARGETS STUB_ZERO_TARGETS STUB_NOT_RUN_TARGETS STUB_AMP_DROP STUB_AMP_RED STUB_AMP_NOT_RUN STUB_AMP_EXIT STUB_AMP_ENGINE_LIBRARY STUB_CRASH_POINT_REPLAY_EXIT STUB_CRASH_POINT_REPLAY_REUSED STUB_BUILD_EXIT

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
    env -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP -u SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE -u SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES \
      -u SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE -u GPU_QUIESCE_ORIGIN_READ_BACK \
      PATH="$work/bin:$PATH" STUB_CALLS="$calls" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
      SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" CRASH_POINT_REPLAY_SELFTEST_MARK="$work/mark" CRASH_POINT_REPLAY_ROOT="$root" \
      CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN="$work/two-hosts-local.sh" CRASH_AMPLIFICATION_TWO_HOSTS_PEER_RUN="$work/two-hosts-peer.sh" CRASH_AMPLIFICATION_PEER_THREADS=2 \
      CRASH_AMPLIFICATION_TWO_HOSTS_SOURCE_ROOT="$root" \
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

  # 两档都跑的那几类的测试目标个数，从 BOTH_TIERS_GROUPS 现数（期望与名单同源：名单加减一个名字，自证不用跟着改）
  local extras_count=0 both_tiers_line both_tiers_name both_tiers_targets both_tiers_words
  for both_tiers_line in "${BOTH_TIERS_GROUPS[@]}"; do
    IFS=$'\t' read -r both_tiers_name both_tiers_targets <<<"$both_tiers_line"
    [[ "$both_tiers_targets" == - ]] && continue
    read -r -a both_tiers_words <<<"$both_tiers_targets"
    extras_count=$((extras_count + ${#both_tiers_words[@]}))
  done
  # list-items：退 0、三行、不起 cargo 与驱动
  entry -- --list-items
  if [[ "$exit_code" != 0 || "$(grep -cE $'^(alpha|beta|gamma)\t轻型：' "$work/out")" != 3 ]] || [[ -s "$calls" ]]; then
    cell_failed list-items "--list-items 应当退 0、逐项列出、不起 cargo 与驱动" "看 --list-items 那一段：只读合表、打表就退"
  fi
  # list-items-with-keys：读库程序的输出取它文件头「输出」那一段原样喂进来（写法与读法钉在同一段字上）⇒ 退 0、三项照列，
  # 报录下的块 6、键对不上的块 4（只在别的判法单元指纹下判过 3、一种判器的结论都没有 1）、键相同的 2，与镜像判器（image_judgement）那一把指纹；不起 cargo
  local keys_reader_source="$replay_script_directory/../../crates/singlefs-checker-tier/src/bin/crash_library_reader.rs"
  sed -n '/^\/\/! keys-format-begin$/,/^\/\/! keys-format-end$/p' "$keys_reader_source" | sed '1d;$d' | sed 's|^//! ||' >"$work/keys-reader-output"
  entry CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys
  if [[ "$exit_code" != 0 || "$(grep -cE $'^(alpha|beta|gamma)\t轻型：' "$work/out")" != 3 ]] || [[ -s "$calls" ]] \
     || ! out_has $'键\t崩溃判定库 /library/crash-state-verdicts：录下的块 6，键对不上的块 4（只在别的判法单元指纹下判过 3、一种判器的结论都没有 1），键相同的 2' \
     || ! out_has $'键\timage_judgement 这一份的判法单元指纹 0505050505050505'; then
    cell_failed list-items-with-keys "--list-items --with-keys 应当照读库程序文件头「输出」那一段的写法报键对不上的块数、不起 cargo" \
      "看 report_the_keys_of_the_library 与 crates/singlefs-checker-tier/src/bin/crash_library_reader.rs 文件头「输出」那一段"
  fi
  # ── 积压跑法那几格（文件头「积压跑法」）：假 cargo 把 cargo run … --bin crash_full_run_backlog -- <子命令…> 交给假的小程序 $work/backlog-program.py，
  #   假的门禁给点名的用例写一份用例日志（CRASH_CASE_BLOCKS 行带 states=37）、打「全量日志：」那一行，假的放量入口认 --list-items。
  #   小仓的表四行（照小仓合表加 ITEMS 的展开）：1 alpha 的 crash-case:alpha、2 alpha 的 alpha-amp、3 beta 的 crash-case:beta、4 gamma 的 gamma-amp
  cat >"$work/backlog-program.py" <<'STUB'
# 假的积压登记表小程序（自证用）：照 crates/singlefs-checker-tier/src/bin/crash_full_run_backlog.rs 文件头的写法打行；每次调记一行「backlog-program <参数…>」。
# plan 照小程序的口径取件：今天的判法键下全绿的算罩住；没罩住的里先全取这一里程碑登记的与判法键变了的（跑过、记下的键不是今天的），
#   再从游标起照表的次序取积压，取到积压够积压下限、两组合计够合计下限为止，不到就全取；未数的照取、算 0。
# STUB_BACKLOG_CHECK_PROBLEMS=<n>：check 报 n 处问题、退 1；STUB_BACKLOG_PLAN_TAMPER=1：plan 末行的积压态数比取件行加起来多 1；STUB_BACKLOG_RECORD_EXIT=<n>：record 退 n
import os, sys
TODAY = "a" * 64
arguments = sys.argv[1:]
with open(os.environ["STUB_CALLS"], "a", encoding="utf-8") as handle:
    handle.write("backlog-program " + " ".join(arguments) + "\n")
command, options = arguments[0], dict(zip(arguments[1::2], arguments[2::2]))
if command == "today-judging-key":
    print(f"CRASH_FULL_RUN_BACKLOG_TODAYS_JUDGING_KEY\tkey={TODAY}")
    sys.exit(0)
if command == "append-missing":
    print("CRASH_FULL_RUN_BACKLOG_APPENDED\tparts=0")
    sys.exit(0)
if command == "check":
    problems = int(os.environ.get("STUB_BACKLOG_CHECK_PROBLEMS", "0"))
    for number in range(problems):
        print(f"CRASH_FULL_RUN_BACKLOG_PROBLEM\t自证造的第 {number + 1} 处问题")
    print(f"CRASH_FULL_RUN_BACKLOG_CHECK\trows=4\texpansion_parts=4\tproblems={problems}")
    sys.exit(1 if problems else 0)
if command == "record":
    print(f"CRASH_FULL_RUN_BACKLOG_RECORDED\tsequence={options['--sequence']}")
    sys.exit(int(os.environ.get("STUB_BACKLOG_RECORD_EXIT", "0")))
if command != "plan":
    sys.exit(2)
rows = [line.rstrip("\n").split("\t") for line in open(options["--table"], encoding="utf-8")]
rows = [row for row in rows if len(row) == 9 and row[0].isdigit()]
today, milestone = options["--today-judging-key"], options["--current-milestone"]
backlog_bound = int(options.get("--backlog-states-at-least", "5000000000"))
total_bound = int(options.get("--states-at-least", "10000000000"))


def is_today(key):
    return key != "未记" and today.startswith(key)


def states(row):
    head = row[5].split("；")[0]
    return int(head) if head.isdigit() else None


uncovered = [row for row in rows if not (row[7] == "全绿" and is_today(row[6]))]
new = [row for row in uncovered if row[4] == milestone or (row[7] != "未跑" and not is_today(row[6]))]
taken = [("new-or-judging-key-changed", row) for row in new]
new_states = sum(states(row) or 0 for row in new)
backlog_states = 0
for row in uncovered:
    if row in new:
        continue
    if backlog_states >= backlog_bound and new_states + backlog_states >= total_bound:
        break
    taken.append(("backlog", row))
    backlog_states += states(row) or 0
reached = backlog_states >= backlog_bound and new_states + backlog_states >= total_bound
print(f"CRASH_FULL_RUN_BACKLOG_CURSOR\tsequence={uncovered[0][0] if uncovered else 'none'}")
for group, row in taken:
    count = states(row)
    print(f"CRASH_FULL_RUN_BACKLOG_TAKE\tgroup={group}\tsequence={row[0]}\titem={row[1]}\tpart={row[2]}\tkey={row[3]}\tstates={'not-counted' if count is None else count}")
reported_backlog = backlog_states + (1 if os.environ.get("STUB_BACKLOG_PLAN_TAMPER") else 0)
print(f"CRASH_FULL_RUN_BACKLOG_PLAN\tnew_or_judging_key_changed_states={new_states}\tbacklog_states={reported_backlog}"
      f"\ttaken_without_a_count={sum(1 for _, row in taken if states(row) is None)}\tlower_bounds={'both' if reached else 'every-uncovered-row-below-the-bounds'}")
STUB
  local backlog_table_file="$root/crates/singlefs-checker-tier/crash-full-run-backlog.tsv" backlog_key_today backlog_table_digest arguments backlog_usage_wrong
  local backlog_settings=(STUB_BACKLOG_PROGRAM="$work/backlog-program.py" STUB_CASE_LOG_DIRECTORY="$work/case-logs")
  local backlog_row_alpha_green backlog_row_alpha_amp backlog_row_beta backlog_row_gamma_amp backlog_record_head
  mkdir -p "$work/case-logs"
  backlog_key_today="$(printf 'a%.0s' {1..64})"
  write_backlog_table() { # <行…>：表头三行照真表的样子，之后每个参数一行
    { printf '%s\n' '# 自证的积压登记表' $'照展开次序登记的行数\t4' $'序号\t项名\t部分\t键\t归属\t态数\t判法键\t进度\t最后一次'; printf '%s\n' "$@"; } >"$backlog_table_file"
  }
  backlog_row() { # <序号> <项> <部分> <键> <归属> <态数> <判法键> <进度>：一行九列
    local last_run='2026-10-25；自证'
    [[ "$8" == 未跑 ]] && last_run=无
    printf '%s\t%s\t%s\t%s\t%s\t%s；自证\t%s\t%s\t%s' "$1" "$2" "$3" "$4" "$5" "$6" "$7" "$8" "$last_run"
  }
  call_line() { grep -nE -- "$1" "$calls" | head -n 1 | cut -d: -f1; }   # <模式>：调用记录里头一处命中的行号，没有打空
  backlog_row_alpha_green="$(backlog_row 1 alpha crash-point-replay crash-case:alpha m3-and-earlier 100 "$backlog_key_today" 全绿)"
  backlog_row_alpha_amp="$(backlog_row 2 alpha crash-amplification alpha-amp m3-and-earlier 100 未记 未跑)"
  backlog_row_beta="$(backlog_row 3 beta crash-point-replay crash-case:beta m3-and-earlier 100 未记 未跑)"
  backlog_row_gamma_amp="$(backlog_row 4 gamma crash-amplification gamma-amp m3-and-earlier 100 未记 未跑)"
  backlog_record_head="^backlog-program record --table $backlog_table_file"
  # backlog-usage：--backlog 缺 --milestone、与 --quick / --full / --item 一起给、--milestone 不跟 --backlog ⇒ 都退 2、什么都不起
  write_backlog_table "$backlog_row_alpha_green" "$backlog_row_alpha_amp" "$backlog_row_beta" "$backlog_row_gamma_amp"
  backlog_usage_wrong=""
  for arguments in "--backlog" "--backlog --milestone m4 --quick" "--backlog --milestone m4 --full" "--backlog --milestone m4 --item alpha" "--milestone m4" "--states-at-least 5"; do
    # shellcheck disable=SC2086  # 参数是空格隔开的几个词，逐个交
    entry "${backlog_settings[@]}" -- $arguments
    [[ "$exit_code" == 2 && ! -s "$calls" ]] || backlog_usage_wrong+="「$arguments」退 $exit_code；"
  done
  [[ -z "$backlog_usage_wrong" ]] || cell_failed backlog-usage "积压跑法的用法错应当退 2、什么都不起（$backlog_usage_wrong）" "看参数那一段之后积压跑法的用法核"
  # backlog-stops-at-the-bounds：两条下限都是 150、序号 1 在今天的键下全绿 ⇒ 游标在序号 2，取 2（alpha-amp，100 态）、3（crash-case:beta，100 态）就够了、不取 4；
  #   快档只跑 alpha 与 beta 两项的测试目标；两件各回写一次全绿、今天的键，态数 14（放量 states=10 加 reused=4）与 37（用例日志里 CRASH_CASE_BLOCKS 行的 states=）
  entry "${backlog_settings[@]}" -- --backlog --milestone m4 --backlog-states-at-least 150 --states-at-least 150
  if [[ "$exit_code" != 0 ]] || ! calls_have '^backlog-program plan .* --current-milestone m4 --backlog-states-at-least 150 --states-at-least 150$' \
     || ! calls_have '^pipeline-entry --item alpha-amp$' || ! calls_have "^enumeration-gate --full --item crash-case:beta " \
     || calls_have 'gamma-amp$' || calls_have '^enumeration-gate .*crash-case:alpha' \
     || ! calls_have '^cargo test --release -p singlefs-checker-tier --features verdict-store --no-run --test alpha --test beta cards=' \
     || ! calls_have "$backlog_record_head --sequence 2 --judging-key $backlog_key_today --progress 全绿 --last-run [0-9]{4}-[0-9]{2}-[0-9]{2}；crash-point-replay.sh --backlog --milestone m4 那一趟 --states 14；" \
     || ! calls_have "$backlog_record_head --sequence 3 --judging-key $backlog_key_today --progress 全绿 .* --states 37；" \
     || [[ "$(calls_count '^backlog-program record ')" != 2 ]] \
     || ! grep -qE '^CRASH_POINT_REPLAY_BACKLOG_TOTAL milestone=m4 taken=2 new_or_judging_key_changed=0 backlog=2 recorded=2 not_recorded=0 recorded_states=51 record_failures=0$' "$work/out"; then
    cell_failed backlog-stops-at-the-bounds "取件取到两条下限就停：两条下限 150 应当只跑序号 2（alpha-amp）与 3（crash-case:beta）、不跑 4（gamma-amp），两件各回写全绿与态数 14、37" \
      "看 backlog_prepare_plan 交给 plan 的两条下限、run_backlog_parts 照取件跑与 backlog_record_the_part 的回写"
  fi
  # backlog-below-the-bounds-takes-everything：两条下限 100000、没罩住的只有三件（300 态）⇒ 三件全取全跑；gamma-amp 那一项放量报本次未跑 ⇒ 它不回写，另两件回写
  entry "${backlog_settings[@]}" STUB_AMP_NOT_RUN=gamma-amp -- --backlog --milestone m4 --backlog-states-at-least 100000 --states-at-least 100000
  if [[ "$exit_code" != 0 ]] || ! calls_have '^pipeline-entry --item alpha-amp$' || ! calls_have '^pipeline-entry --item gamma-amp$' \
     || ! calls_have "^enumeration-gate --full --item crash-case:beta " || calls_have "$backlog_record_head --sequence 4 " \
     || [[ "$(calls_count '^backlog-program record ')" != 2 ]] \
     || ! grep -qE '^CRASH_POINT_REPLAY_BACKLOG_PART sequence=4 group=backlog item=gamma part=crash-amplification key=gamma-amp verdict=not-run recorded=no states=- reason=' "$work/out" \
     || ! grep -qE '^CRASH_POINT_REPLAY_BACKLOG_TOTAL milestone=m4 taken=3 new_or_judging_key_changed=0 backlog=3 recorded=2 not_recorded=1 ' "$work/out"; then
    cell_failed backlog-below-the-bounds-takes-everything "两条下限都到不了时没罩住的三件应当全取全跑；本次未跑的那一件（gamma-amp）不回写，另两件回写" \
      "看 backlog_plan 取到的件与 backlog_record_the_part 里本次未跑那一支"
  fi
  # backlog-new-and-key-changed-first：序号 3 归属 m4（这一里程碑新增）、序号 4 在别的键下全绿（判法键变了），两条下限 50
  #   ⇒ 先跑 3、4（照表的次序），再从游标起跑积压序号 2；调用记录里 crash-case:beta、gamma-amp、alpha-amp 依次
  write_backlog_table "$backlog_row_alpha_green" "$backlog_row_alpha_amp" "$(backlog_row 3 beta crash-point-replay crash-case:beta m4 100 未记 未跑)" \
    "$(backlog_row 4 gamma crash-amplification gamma-amp m3-and-earlier 100 bbbbbbbbbbbbbbbb 全绿)"
  entry "${backlog_settings[@]}" -- --backlog --milestone m4 --backlog-states-at-least 50 --states-at-least 50
  local backlog_beta_line backlog_gamma_line backlog_alpha_line
  backlog_beta_line="$(call_line '^enumeration-gate --full --item crash-case:beta ')"; backlog_gamma_line="$(call_line '^pipeline-entry --item gamma-amp$')"
  backlog_alpha_line="$(call_line '^pipeline-entry --item alpha-amp$')"
  if [[ "$exit_code" != 0 || -z "$backlog_beta_line" || -z "$backlog_gamma_line" || -z "$backlog_alpha_line" ]] \
     || (( backlog_beta_line > backlog_gamma_line || backlog_gamma_line > backlog_alpha_line )) \
     || ! grep -qE '^CRASH_POINT_REPLAY_BACKLOG_TOTAL milestone=m4 taken=3 new_or_judging_key_changed=2 backlog=1 ' "$work/out"; then
    cell_failed backlog-new-and-key-changed-first "这一里程碑新增与判法键变了的应当先跑：crash-case:beta（m4）、gamma-amp（键变了）在前，积压 alpha-amp 在后（调用记录行号 ${backlog_beta_line:-无}、${backlog_gamma_line:-无}、${backlog_alpha_line:-无}）" \
      "看 backlog_prepare_plan 读取件文件的次序（照取件的次序，不照表的序号）"
  fi
  # list-items-backlog：只读用法，两条下限 150 与 250 ⇒ 游标序号 2；只看积压下限跑到序号 3，只看合计下限跑到序号 4；三项照列；
  #   只起表的小程序（today-judging-key 一次、plan 三次），不起 record、append-missing、check、快档、门禁与放量入口，表一个字节不动
  write_backlog_table "$backlog_row_alpha_green" "$backlog_row_alpha_amp" "$backlog_row_beta" "$backlog_row_gamma_amp"
  backlog_table_digest="$(sha256sum <"$backlog_table_file")"
  entry "${backlog_settings[@]}" -- --list-items --backlog --milestone m4 --backlog-states-at-least 150 --states-at-least 250
  if [[ "$exit_code" != 0 || "$(grep -cE $'^(alpha|beta|gamma)\t轻型：' "$work/out")" != 3 ]] \
     || ! out_has $'积压\t游标：序号 2（alpha crash-amplification alpha-amp）' || ! out_has $'积压\t到积压下限 150 态：从游标起跑到序号 3（' \
     || ! out_has $'积压\t到合计下限 250 态：跑到序号 4（' \
     || [[ "$(calls_count '^cargo ')" != 4 || "$(calls_count '^cargo run --release --quiet --offline -p singlefs-checker-tier --features verdict-store --bin crash_full_run_backlog -- (today-judging-key|plan) ')" != 4 ]] \
     || [[ "$(calls_count '^backlog-program plan ')" != 3 ]] || calls_have '^backlog-program (record|append-missing|check) ' \
     || calls_have '^(pipeline-entry|pipeline-list-items|enumeration-gate)' || [[ "$(sha256sum <"$backlog_table_file")" != "$backlog_table_digest" ]]; then
    cell_failed list-items-backlog "--list-items --backlog 应当退 0、照小程序 plan 报游标与到两条下限各要跑到哪一行、只起表的小程序（today-judging-key 一次、plan 三次）、不改表、不起快档与两套驱动" \
      "看 backlog_listing 与 --list-items 那一段"
  fi
  # backlog-plan-integrity：plan 末行的积压态数比取件行加起来多 1 ⇒ 退 1、一件都不跑、不回写
  entry "${backlog_settings[@]}" STUB_BACKLOG_PLAN_TAMPER=1 -- --backlog --milestone m4 --backlog-states-at-least 150 --states-at-least 150
  if [[ "$exit_code" != 1 ]] || ! out_has "积压登记表的小程序 plan 的输出对不上" || calls_have '^(pipeline-entry|enumeration-gate) ' \
     || calls_have '^cargo test ' || calls_have '^backlog-program record '; then
    cell_failed backlog-plan-integrity "小程序 plan 的输出对不上（末行的积压态数比取件行加起来多 1）应当退 1、一件都不跑" "看 backlog_plan 里核 plan 输出的那一段"
  fi
  # backlog-table-refused：先 append-missing、再 check；check 报一处问题、退 1 ⇒ 退 2、不取件（不起 plan）、什么都不起
  entry "${backlog_settings[@]}" STUB_BACKLOG_CHECK_PROBLEMS=1 -- --backlog --milestone m4
  if [[ "$exit_code" != 2 ]] || ! out_has "积压登记表 $BACKLOG_TABLE_PATH_IN_REPOSITORY 核不过" || calls_have '^backlog-program plan ' \
     || ! calls_have '^backlog-program append-missing ' || ! calls_have '^backlog-program check ' \
     || calls_have '^(pipeline-entry|enumeration-gate) ' || calls_have '^cargo test '; then
    cell_failed backlog-table-refused "积压登记表核不过（小程序 check 退 1）应当退 2、不取件、什么都不起" "看 backlog_prepare_plan 里 append-missing 与 check 那一段"
  fi
  # backlog-red-part-recorded-red：门禁对 crash-case:beta 退 1 ⇒ 整趟退 1；序号 3 回写成红（进度「红：门禁 … 退 1…」），序号 2 照回写全绿
  entry "${backlog_settings[@]}" STUB_CRASH_POINT_REPLAY_EXIT=1 -- --backlog --milestone m4 --backlog-states-at-least 150 --states-at-least 150
  if [[ "$exit_code" != 1 ]] || ! calls_have "$backlog_record_head --sequence 3 --judging-key $backlog_key_today --progress 红：门禁 checker-tier-crash-point-replay --full --item crash-case:beta 退 1（" \
     || ! calls_have "$backlog_record_head --sequence 2 --judging-key $backlog_key_today --progress 全绿 " \
     || ! grep -qE '^CRASH_POINT_REPLAY_BACKLOG_PART sequence=3 group=backlog item=beta part=crash-point-replay key=crash-case:beta verdict=red recorded=yes ' "$work/out"; then
    cell_failed backlog-red-part-recorded-red "积压跑法里红的那一件应当回写成红、整趟退 1（序号 3 的进度写「红：门禁 … 退 1」，序号 2 照写全绿）" \
      "看 backlog_record_the_part 里定进度那一段与 run_enumeration_part 的退出码"
  fi
  if [[ -n "${CRASH_POINT_REPLAY_SELFTEST_ONLY:-}" ]]; then
    if [[ "$CRASH_POINT_REPLAY_SELFTEST_ONLY" != backlog ]]; then
      echo "  ✗ CRASH_POINT_REPLAY_SELFTEST_ONLY=$CRASH_POINT_REPLAY_SELFTEST_ONLY 认不得（只认 backlog）" >&2
      echo "     → 怎么办：去掉这个变量跑全部格与开关轮，或写成 CRASH_POINT_REPLAY_SELFTEST_ONLY=backlog 只看积压跑法那几格" >&2
      exit 2
    fi
    if (( failures > 0 )); then
      echo "  ✗ --selftest（只走积压跑法那几格）：$cells 趟里 $failures 格判错" >&2
      echo "     → 怎么办：照上面每一格给的出路改，再跑 CRASH_POINT_REPLAY_SELFTEST_ONLY=backlog $0 --selftest；改完去掉这个变量跑全部格与开关轮" >&2
      exit 1
    fi
    echo "  ✓ --selftest（只走积压跑法那几格，别的格与开关轮没走）：$cells 趟都对"
    exit 0
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
  # quick-runs-the-light-domains：全部项的甲二（三个测试目标，加两档都跑的那几类）与展开 6（两个放量分项、不交 --library：照放量入口的默认长期库），不碰崩溃点重放那一道
  entry -- --quick
  if [[ "$exit_code" != 0 ]] || ! calls_have '^cargo test --release -p singlefs-checker-tier --features verdict-store --no-run --test alpha --test beta --test gamma_amp --test crash_judge_gpu_agrees' \
     || [[ "$(calls_count '^cargo test --release -p singlefs-checker-tier --features verdict-store --test [a-z0-9_]+ -- --nocapture ')" != $((3 + extras_count)) ]] \
     || ! calls_have "^pipeline-entry --expand-up-to $LIGHT_EXPAND_UP_TO --item alpha-amp --item gamma-amp\$" \
     || calls_have '^enumeration-gate ' || [[ -e "$work/home/crash-point-replay-quick" ]] \
     || ! item_has alpha 'verdict=green' || ! item_has 'both-tiers:pinned-red-examples' 'verdict=not-run' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL tier=quick dispatched=6 collected=6 green=5 red=0 not_run=1 ' "$work/out"; then
    cell_failed quick-runs-the-light-domains "轻型应当照甲二与展开 6 跑全部项（快档 $((3 + extras_count)) 个测试目标、放量两项不交 --library 照默认长期库、不建 crash-point-replay-quick/、不起崩溃点重放），三项与两档都跑的三类都收回、钉表那一类报本次未跑" \
      "看 replay_plan 的 quick 那一支、run_pipeline_part 的 --library 与 BOTH_TIERS_GROUPS"
  fi
  # quick-named-item：--quick --item beta 照样跑核心部分（全部项）
  entry -- --quick --item beta
  if [[ "$exit_code" != 0 ]] || ! calls_have '^pipeline-entry --expand-up-to [0-9]+ --item alpha-amp --item gamma-amp( |$)' || ! out_has "点名的项已在核心部分里"; then
    cell_failed quick-named-item "--quick --item 应当照样跑核心部分（全部项）、说明点名的项已在里面" "看定档那一段 quick 的 selected_items"
  fi
  # quick-gpu：开了 GPU，快档带 gpu 特性、每一趟经清场包装起、交清场之后那一份显卡先后、不送走 GPU 的计算类（漏进来的也清掉）；编那一趟不清场
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" GPU_COMPUTATION_CLASSES=crash_judge_gpu -- --quick
  if [[ "$exit_code" != 0 ]] || ! calls_have '^quiesce-stop$' \
     || [[ "$(calls_count "^cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --test [a-z0-9_]+ -- --nocapture --test-threads 1 cards=\[local:0:24G\] classes=\[unset\] quiesced=\[yes\]")" != $((3 + extras_count)) ]] \
     || ! calls_have '^cargo test .*--features verdict-store,gpu --no-run .*quiesced=\[\]'; then
    cell_failed quick-gpu "开了 GPU 快档应当带 gpu 特性、经清场包装起、测试参数带 --test-threads 1、交清场之后那一份显卡先后（local:0:24G）、不送 GPU_COMPUTATION_CLASSES（调用方环境里漏进来的也清掉）" \
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
  # pruned-heavy：不带参数 ⇒ 快档（连同两档都跑的那几类）、崩溃点重放每项一趟 --full --item（不带 --start-over）、放量不带 --item 与 --library；复用数照实报
  entry STUB_CRASH_POINT_REPLAY_REUSED=1 --
  if [[ "$exit_code" != 0 ]] || [[ "$(calls_count '^cargo test --release -p singlefs-checker-tier --features verdict-store --test [a-z0-9_]+ -- --nocapture ')" != $((3 + extras_count)) ]] \
     || ! calls_have "^enumeration-gate --full --item crash-case:alpha $root\$" || ! calls_have "^enumeration-gate --full --item crash-case:beta $root\$" \
     || ! calls_have '^pipeline-entry $' || calls_have 'start-over' || calls_have '^admission ' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL tier=pruned dispatched=6 collected=6 .* pipeline_reused_known=8 .* enumeration_cases=2 enumeration_reused=2 ' "$work/out"; then
    cell_failed pruned-heavy "不带参数的重型应当照剪枝跑全部项（快档照跑、崩溃点重放每项 --full --item、放量照全量、复用数照实报）" "看 replay_plan 的重型那一支与 run_enumeration_part"
  fi
  # named-heavy：--item gamma ⇒ 只跑 gamma：快档只 gamma_amp、不跑两档都跑的那几类、不起崩溃点重放、放量 --item gamma-amp
  entry -- --item gamma
  if [[ "$exit_code" != 0 ]] || [[ "$(calls_count ' --test [a-z0-9_]+ -- --nocapture ')" != 1 ]] || ! calls_have ' --test gamma_amp -- --nocapture ' \
     || calls_have '^enumeration-gate ' || ! calls_have '^pipeline-entry --item gamma-amp$' || item_line 'both-tiers:pruning-self-proof' >/dev/null; then
    cell_failed named-heavy "--item 应当只跑点名的项（逐条）" "看定档那一段 named 与 run_pipeline_part 的 --item"
  fi
  # full-heavy：--full ⇒ 崩溃点重放带 --start-over（只写不读）、放量带 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1（只写不读）；准入模块一次都不调（用例一级没有全绿标记可删）
  entry -- --full
  if [[ "$exit_code" != 0 ]] || ! calls_have "^enumeration-gate --full --start-over --item crash-case:alpha $root\$" \
     || ! calls_have '^pipeline-env write-only=\[1\]$' || calls_have '^admission '; then
    cell_failed full-heavy "全量应当只写不读（崩溃点重放 --start-over、放量带 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1，不调准入模块）" "看 run_enumeration_part 与 run_pipeline_part 里 tier=full 那两段"
  fi
  # full-writes-back-into-the-one-library：假的放量入口照引擎建库 ⇒ --full 那一支不交 --library、不另建库（不建 crash-point-replay-full/），放量那一趟只写不读
  entry STUB_AMP_ENGINE_LIBRARY=1 -- --full
  if [[ "$exit_code" != 0 ]] || calls_have '^pipeline-entry .*--library' || calls_have '^pipeline-library created ' \
     || [[ -e "$work/home/crash-point-replay-full" ]] || ! calls_have '^pipeline-env write-only=\[1\]$'; then
    cell_failed full-writes-back-into-the-one-library "--full 那一支应当照键写回同一份库、不另建库（不交 --library、不建 crash-point-replay-full/、放量带 SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1）" \
      "看 run_pipeline_part 里 tier=full 那一段"
  fi
  # enumeration-red：崩溃点重放那一道退 1 ⇒ alpha、beta 红，退 1
  entry STUB_CRASH_POINT_REPLAY_EXIT=1 -- --item alpha --item beta
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'verdict=red' || ! item_has beta 'enumeration_red=1' \
     || ! out_has "只重跑红的那 2 项：SINGLEFS_HEAVY_TESTS=<commit|user-request> bash scripts/crash/crash-point-replay.sh --item alpha --item beta"; then
    cell_failed enumeration-red "崩溃点重放那一道红了，那一项应当红、出路给出只重跑那两项的命令" "看 run_enumeration_part 判 exit_code 那一段与 summarize 的出路"
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
  # fast-tier-red-stops-heavy：beta 那个测试目标红 ⇒ 重型的三种跑法（剪枝、逐条、全量）退 1，不起崩溃点重放那一道与放量入口，两段记「本次未跑：快档红」、派出收回相等；
  #   轻型（--quick）照旧往下跑放量部分
  local stop_wrong=() stop_case stop_settings
  for stop_case in "" "--item alpha --item beta" "--full"; do
    # shellcheck disable=SC2086  # 参数是空格隔开的一串，逐个当参数
    entry STUB_RED_TARGETS=beta -- $stop_case
    if [[ "$exit_code" != 1 ]] || calls_have '^enumeration-gate ' || calls_have '^pipeline-entry ' || ! item_has beta 'verdict=red' \
       || ! out_has '本次未跑：快档红' || ! out_has '本次未跑 alpha：快档红' || ! grep -qE 'CRASH_POINT_REPLAY_TOTAL .*dispatched=([0-9]+) collected=\1 ' "$work/out"; then
      stop_wrong+=("「crash-point-replay.sh $stop_case」退 $exit_code、调用记录 $(tr '\n' '|' <"$calls" | cut -c1-200)")
    fi
  done
  entry STUB_RED_TARGETS=beta -- --quick
  if [[ "$exit_code" != 1 ]] || ! calls_have '^pipeline-entry ' || out_has '本次未跑：快档红'; then
    stop_wrong+=("「crash-point-replay.sh --quick」退 $exit_code（轻型快档红照旧往下跑放量部分），调用记录 $(tr '\n' '|' <"$calls" | cut -c1-200)")
  fi
  if (( ${#stop_wrong[@]} )); then
    exit_code="${#stop_wrong[@]} 处"
    cell_failed fast-tier-red-stops-heavy "快档红了，重型的剪枝、逐条、全量都应当退 1、不起崩溃点重放那一道与放量入口、两段记本次未跑：快档红、派出收回相等；轻型照旧往下跑放量部分：${stop_wrong[*]}" \
      "看收尾那一段主流程里 skip_parts_after_red_fast_tier 的判据与 summarize 里红的项连带打的本次未跑"
  fi
  # run-target-green / run-target-red / run-target-term：跑完运行目录里不剩它建的编译目录（文件头「编译目录」）：调用方没设 CARGO_TARGET_DIR，轻型；
  #   假 cargo 记下每一趟收到的编译目录、往里写一份产物。正常退 0、快档红退 1、被 TERM 停（假 cargo 每一趟睡 3 秒，头一趟起了之后只给入口发 TERM，
  #   入口等手里那一步退出后走 EXIT，退 143）：每一趟收到的都是 $work/home/runs/crash-point-replay-*/target 那同一份，跑完不在了
  run_target_check() { # <格名> <要的退出码> <一句结局>
    local seen
    seen="$(sort -u "$work/targets.log" 2>/dev/null)"
    if [[ "$exit_code" != "$2" || "$(grep -c . <<<"$seen")" != 1 || "$seen" != "$work"/home/runs/crash-point-replay-*/target || -e "$seen" ]]; then
      cell_failed "$1" "跑完运行目录里不剩它建的编译目录（$3）：假 cargo 收到「$(tr '\n' ' ' <<<"$seen")」，要的退出码 $2，跑完$([[ -n "$seen" && -e "$seen" ]] && echo 还在 || echo 不在)" \
        "看建日志目录之后的 run_build_directory_begin 与收尾的 replay_finish（run_build_directory_end），TERM / INT 的 trap 退 143 / 130、等手里那一步退出再走 EXIT"
    fi
  }
  : >"$work/targets.log"; entry STUB_TARGET_LOG="$work/targets.log" -- --quick; run_target_check run-target-green 0 "正常退 0"
  : >"$work/targets.log"; entry STUB_TARGET_LOG="$work/targets.log" STUB_RED_TARGETS=beta -- --quick; run_target_check run-target-red 1 "快档红退 1"
  local term_entry term_attempt
  : >"$work/targets.log"; : >"$calls"; cells=$((cells + 1))
  env -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP -u SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE -u SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES \
    PATH="$work/bin:$PATH" STUB_CALLS="$calls" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
    SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" CRASH_POINT_REPLAY_SELFTEST_MARK="$work/mark" CRASH_POINT_REPLAY_ROOT="$root" \
    CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN="$work/two-hosts-local.sh" CRASH_AMPLIFICATION_TWO_HOSTS_PEER_RUN="$work/two-hosts-peer.sh" CRASH_AMPLIFICATION_PEER_THREADS=2 \
    CRASH_POINT_REPLAY_BREAK="$replay_break" STUB_TARGET_LOG="$work/targets.log" STUB_SLEEP=3 bash "$replay_script" --quick >"$work/out" 2>&1 &
  term_entry=$!
  for (( term_attempt = 0; term_attempt < 600; term_attempt++ )); do [[ -s "$work/targets.log" ]] && break; sleep 0.1; done
  kill -TERM "$term_entry" 2>/dev/null
  if wait "$term_entry"; then exit_code=0; else exit_code=$?; fi
  run_target_check run-target-term 143 "被 TERM 停"
  # run-target-caller：调用方设了 CARGO_TARGET_DIR ⇒ 每一趟照收到它、跑完不删
  mkdir -p "$work/caller-target"; : >"$work/targets.log"
  entry STUB_TARGET_LOG="$work/targets.log" CARGO_TARGET_DIR="$work/caller-target" -- --quick
  if [[ "$exit_code" != 0 || "$(sort -u "$work/targets.log")" != "$work/caller-target" || ! -f "$work/caller-target/release/artifact" ]]; then
    cell_failed run-target-caller "调用方设了 CARGO_TARGET_DIR 时每一趟应当照收到它、跑完不删（收到「$(sort -u "$work/targets.log" | tr '\n' ' ')」）" \
      "看 run_build_directory_begin：设了 CARGO_TARGET_DIR 就不建、不删"
  fi
  # two-logs-per-part：每一段的日志出两份（全量 <段>.log，旁边同名的 <段>.problems.log 只收错误与警告：行前带全量日志里的行号，末行写收了几行全量几行）；
  #   快档红的那个测试目标的 FAILED 行在错误警告日志里，输出里点名这份日志的路径
  local problems_run problems_file problems_last_line
  entry STUB_RED_TARGETS=beta -- --quick
  problems_run="$(ls -td "$work/home/runs"/crash-point-replay-* 2>/dev/null | head -n 1)"
  problems_file="$problems_run/fast-tier.beta.problems.log"
  problems_last_line="$(tail -n 1 "$problems_file" 2>/dev/null)"   # 管道收尾的 grep -q 在 pipefail 下会把命中读成没命中：先落到变量
  if [[ ! -f "$problems_run/fast-tier.beta.log" || ! -f "$problems_file" ]] || ! grep -qE '^[0-9]+:test broken \.\.\. FAILED$' "$problems_file" \
     || ! grep -qE '收了 [0-9]+ 行，全量日志共 [0-9]+ 行' <<<"$problems_last_line" || ! out_has "错误警告 $problems_file"; then
    cell_failed two-logs-per-part "快档红的那一段应当出两份日志（fast-tier.beta.log 全量、旁边的 fast-tier.beta.problems.log 带行号收那条 FAILED、末行写收了几行全量几行），输出里点名错误警告日志的路径 $problems_file" \
      "看 run_logged 经 scripts/hosts/two-logs.sh 起命令，与红的原因里的 \${log%.log}.problems.log"
  fi
  # pipeline-entry-78-not-run：放量入口退 78（运行条件不满足）而没打 CRASH_AMPLIFICATION_ITEM 行 ⇒ 两项都记本次未跑、整趟退 78，不说「都不红」；
  #   红先判：alpha-amp 红、gamma-amp 收不回而入口退 78 ⇒ 退 1；入口退 1 而收不回照旧红
  entry STUB_AMP_EXIT=78 STUB_AMP_DROP="alpha-amp gamma-amp" -- --quick
  if [[ "$exit_code" != 78 ]] || ! item_has alpha 'exit=78 verdict=not-run' || ! item_has gamma 'exit=78 verdict=not-run' || item_has alpha 'verdict=red' \
     || ! out_has '本次未跑 alpha：运行条件不满足：放量入口退 78' || out_has '项都不红' || ! out_has '有一段运行条件不满足（退 78）'; then
    cell_failed pipeline-entry-78-not-run "放量入口退 78 而没打 ITEM 行，两项应当记本次未跑（exit=78 verdict=not-run）、整趟退 78、不说都不红" "看 run_pipeline_part 里 line 为空那一支对退出码 78 的分支"
  fi
  entry STUB_AMP_RED=alpha-amp STUB_AMP_DROP=gamma-amp STUB_AMP_EXIT=78 -- --quick
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'verdict=red' || ! item_has gamma 'exit=78 verdict=not-run'; then
    cell_failed pipeline-entry-78-red-first "放量入口退 78 而 alpha-amp 红时，整趟应当退 1、alpha 红、gamma 本次未跑" "看 run_pipeline_part 与 summarize：红先判，退 78 的只在没有红时才让整趟退 78"
  fi
  entry STUB_AMP_EXIT=1 STUB_AMP_DROP=gamma-amp -- --quick
  if [[ "$exit_code" != 1 ]] || ! item_has gamma 'verdict=red' || ! out_has '收不回'; then
    cell_failed pipeline-entry-1-still-red "放量入口退 1（不是 78）而没打 ITEM 行，仍应当红（收不回）" "看 run_pipeline_part 里 line 为空那一支：只有退 78 才记本次未跑"
  fi
  # quick-reuses-the-long-term-library：假的放量入口照键复用（判过的分项在库里留一份）⇒ 轻型接连两趟：都不交 --library、用 <家目录>/crash-state-verdicts，
  #   头一趟现判 20 态复用 0 态，第二趟现判 0 态复用 20 态（第二次跑复用了第一次判过的块），成功句照实报，库跑完留着、不建 crash-point-replay-quick/
  local reuse_first_ok=0 reuse_first_total reuse_library="$work/home/crash-state-verdicts"
  rm -rf -- "${reuse_library:?}"
  entry STUB_AMP_PERSISTENT=1 -- --quick
  reuse_first_total="$(grep -E '^CRASH_POINT_REPLAY_TOTAL ' "$work/out")"
  if [[ "$exit_code" == 0 ]] && calls_have "^pipeline-persistent-library $reuse_library\$" && ! calls_have '^pipeline-entry .*--library' \
     && [[ "$reuse_first_total" == *" pipeline_states=20 pipeline_reused_known=0 "* ]] && out_has "放量现判 20 态复用 0 态"; then reuse_first_ok=1; fi
  entry STUB_AMP_PERSISTENT=1 -- --quick
  if (( ! reuse_first_ok )) || [[ "$exit_code" != 0 ]] || ! calls_have "^pipeline-persistent-library $reuse_library\$" || calls_have '^pipeline-entry .*--library' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL .* pipeline_states=0 pipeline_reused_known=20 ' "$work/out" || ! out_has "放量现判 0 态复用 20 态" \
     || [[ ! -e "$reuse_library/alpha-amp.judged" || ! -e "$reuse_library/gamma-amp.judged" ]] || [[ -e "$work/home/crash-point-replay-quick" ]]; then
    cell_failed quick-reuses-the-long-term-library "轻型那一趟第二次跑应当复用第一次判过的块：两趟都不交 --library、用 $reuse_library，头一趟现判 20 态复用 0 态、第二趟现判 0 态复用 20 态，库留着（头一趟：${reuse_first_total:-没有 TOTAL 行}）" \
      "看 run_pipeline_part 里 pipeline-expand 那一支（不交 --library）与文件头「展开 6」"
  fi
  rm -rf -- "${reuse_library:?}"
  # reuse-three-questions（scripts/gate/lib_reuse_checks.py 文件头「一」）：假的放量入口照键复用、键另带每个分项的单元文件（STUB_AMP_UNITS）⇒ 轻型接连四趟：
  #   头一趟、同一份输入、加一份哪个分项都不读的单元文件（闭包外）、改 alpha-amp 的单元文件（闭包内）；从 ITEM 行取有放量那一段的项（alpha、gamma）复用还是现判
  #   （pipeline_states=0 而 pipeline_reused 是正整数是复用），照三问判、打规范输出。这一份入口不判复用（判定在引擎里）：核的是每一趟把同一份长期库交下去、
  #   不因闭包外的改动换库，并照实把放量入口报的现判与复用写进 ITEM 行；quick-reuses-the-long-term-library 那一格是这里的 ①
  local three_units="$work/amp-units" three_exits=() three_round_name
  mkdir -p -- "$three_units"; printf 'alpha-amp 的单元\n' >"$three_units/alpha-amp"; printf 'gamma-amp 的单元\n' >"$three_units/gamma-amp"
  three_round() { # <名>
    entry STUB_AMP_PERSISTENT=1 STUB_AMP_UNITS="$three_units" -- --quick
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
      "照规范输出那一行 red 的那一问查：run_pipeline_part 交给放量入口的库（不交 --library、用长期库）与 ITEM 行的 pipeline_states=、pipeline_reused= 取法"
  fi
  grep '^REUSE_SELFTEST ' "$work/three.log"
  rm -rf -- "${reuse_library:?}"
  # last-run-list（scripts/gate/lib_reuse_checks.py 文件头「三」）：轻型一趟 gamma 那一项放量红，上次红的清单是 gamma；再一趟全绿，清单成空文件
  local last_run_file="$work/last-runs/scripts__crash__crash-point-replay.sh.tsv" last_red
  rm -f -- "$last_run_file"
  entry STUB_AMP_RED=gamma-amp -- --quick
  last_red="$(cat "$last_run_file" 2>/dev/null)"
  entry -- --quick
  if [[ "$last_red" != gamma || ! -f "$last_run_file" || -s "$last_run_file" ]]; then
    cell_failed last-run-list "上次红的清单不对：gamma 红的那一趟之后是「$last_red」，全绿那一趟之后$([[ -f "$last_run_file" ]] && echo "是「$(cat "$last_run_file")」" || echo 没有这份文件)" \
      "看收尾写 last-run 那一段：summarize 交出的红的项一行一个（exit.red-items），轻型整份写"
  fi
  # fast-tier-reuse-counted：快档 alpha 那个测试目标的日志里有复用计数行（resumed_slices=3）⇒ alpha 那一行 fast_tier_reused=3；beta 没有那一行 ⇒ unknown、计进取不出的、成功句点名
  entry STUB_REUSE_TARGETS=alpha -- --quick
  if [[ "$exit_code" != 0 ]] || ! item_has alpha 'fast_tier_reused=3 fast_tier_reuse_unknown=0 ' || ! item_has beta 'fast_tier_reused=unknown fast_tier_reuse_unknown=1 ' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL .* fast_tier_reused_known=3 fast_tier_reuse_unknown=[1-9][0-9]* ' "$work/out" \
     || ! out_has "快档 " || ! grep -qE '复用 3 块（另有 [0-9]+ 个测试目标取不出复用数：[^）]*beta' "$work/out"; then
    cell_failed fast-tier-reuse-counted "快档日志里有复用计数行时 ITEM 行应当报同一个数（alpha：fast_tier_reused=3），没有时报取不出（beta：fast_tier_reused=unknown、计进 fast_tier_reuse_unknown）、成功句点名取不出的测试目标" \
      "看 judge_fast_tier_log 里数 resumed_slices= 那一段与 summarize 的 fast_reused"
  fi
  # test-variables-outside-selftest：不带自证标记、带了 CRASH_POINT_REPLAY_ROOT ⇒ 退 2、什么都不起
  : >"$calls"; cells=$((cells + 1))
  env PATH="$work/bin:$PATH" STUB_CALLS="$calls" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" \
    SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" CRASH_POINT_REPLAY_ROOT="$root" \
    CRASH_POINT_REPLAY_BREAK="$replay_break" bash "$replay_script" --quick >"$work/out" 2>&1
  exit_code=$?
  [[ "$exit_code" == 2 && ! -s "$calls" ]] || cell_failed test-variables-outside-selftest "平常跑时带了只供自证的变量应当退 2" "看开头拒只供自证变量那一段"
  # 往引擎送的变量（文件头「往引擎送的变量」）：快档每一趟 cargo test 与放量入口那一趟都要带到
  local fast_runs_pattern='^cargo test --release -p singlefs-checker-tier --features verdict-store --test [a-z0-9_]+ -- --nocapture '
  local registry="$root/.claude/gate.d/crash-point-replay-segments.tsv"
  every_fast_run_has() { # <那一行里要有的一段（正则）>：快档至少跑了一趟，且每一趟 cargo test 的调用记录都有它
    local all
    all="$(calls_count "$fast_runs_pattern")"
    (( all > 0 )) && [[ "$(calls_count "${fast_runs_pattern}.*$1")" == "$all" ]]
  }
  # segments-registry-sent：每一趟都带按段登记文件的绝对路径
  entry -- --quick
  if [[ "$exit_code" != 0 ]] || ! every_fast_run_has " segments=\[$registry\]\$" \
     || ! calls_have "^pipeline-env segments=\[$registry\]\$"; then
    cell_failed segments-registry-sent "快档与放量每一趟都应当带按段登记文件的绝对路径（SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=$registry）" \
      "看「往引擎送的变量」那一段的 export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY"
  fi
  # fast-tier-threads-default：快档每一趟 cargo 都经 capped.sh default 起，CARGO_BUILD_JOBS 是登记的重活默认并行度（SINGLEFS_THREAD_CAP 设了取它）
  local expected_jobs
  expected_jobs="${SINGLEFS_THREAD_CAP:-$(grep -vE '^[[:space:]]*(#|$)' "$replay_script_directory/../../.claude/heavy-thread-default" | head -n 1 | awk '{print $1}')}"
  if [[ "$exit_code" != 0 ]] || [[ -z "$expected_jobs" ]] || ! every_fast_run_has " jobs=\[$expected_jobs\] segments="; then
    cell_failed fast-tier-threads-default "快档每一趟 cargo 都应当带登记的重活默认并行度（CARGO_BUILD_JOBS=$expected_jobs）" \
      "看 run_fast_tier 里的 thread_prefix（bash scripts/hosts/capped.sh default）"
  fi
  # segments-registry-refused：按段登记文件核不过（项名不在合表里）⇒ 退 2、什么都不起
  cp -- "$registry" "$work/segments-saved.tsv"
  printf '%s\n' "# 自证：核不过的按段登记" "ghost	ghost_flow	0	7	$(printf '0123456789abcdef%.0s' 1 2 3 4)" >"$registry"
  entry -- --quick
  cp -- "$work/segments-saved.tsv" "$registry"
  if [[ "$exit_code" != 2 || -s "$calls" ]] || ! out_has "项名 ghost 不是合表"; then
    cell_failed segments-registry-refused "按段登记文件核不过应当退 2、什么都不起、逐处列出问题" \
      "看「往引擎送的变量」那一段开头的 lib_crash_point_replay_items.py segments"
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
  # quick-slice-states-sent：--quick（不点名与 --item 两种）起放量入口时调用方没设 ⇒ 入口收到 65536；调用方设了 131072 ⇒ 收到 131072；不带 --quick（剪枝、逐条、全量）⇒ 入口收到 unset
  local slice_wrong=() slice_case slice_want slice_setting slice_arguments slice_settings
  for slice_case in "65536|-|--quick" "65536|-|--quick --item alpha" "131072|SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=131072|--quick" \
                    "unset|-|" "unset|-|--item gamma" "unset|-|--full"; do
    IFS='|' read -r slice_want slice_setting slice_arguments <<<"$slice_case"
    slice_settings=()
    [[ "$slice_setting" == - ]] || slice_settings=("$slice_setting")
    # shellcheck disable=SC2086  # 参数是空格隔开的一串，逐个当参数
    entry ${slice_settings[@]+"${slice_settings[@]}"} -- $slice_arguments
    if [[ "$exit_code" != 0 ]] || [[ "$(calls_count '^pipeline-env slice-states=')" != 1 ]] || ! calls_have "^pipeline-env slice-states=\[$slice_want\]\$"; then
      slice_wrong+=("「${slice_settings[*]+${slice_settings[*]} }crash-point-replay.sh $slice_arguments」退 $exit_code、入口收到 $(grep -E '^pipeline-env slice-states=' "$calls" | tr '\n' ' ')（应当 [$slice_want]）")
    fi
  done
  if (( ${#slice_wrong[@]} )); then
    exit_code="${#slice_wrong[@]} 处"
    cell_failed quick-slice-states-sent "轻型起放量入口时调用方没设 SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES 应当设成 $LIGHT_REGISTERED_SEGMENT_SLICE_STATES、调用方设了（131072）放量入口应当照原值收到、不带 --quick 的跑法应当不设：${slice_wrong[*]}" \
      "看 run_pipeline_part 里 pipeline-expand 那一支的 pipeline_environment 与文件头「展开 6」"
  fi
  # 双机那几格（文件头「双机」）：beta 那一项临时登记成经给定状态引擎的；双机驱动是真的，本机那一趟与第二台那一趟是 entry 设的两份假的
  cp -- "$work/stage-inputs-given-states.tsv" "$root/.claude/gate.d/stage-inputs.tsv"
  local two_hosts_settings=(SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-two-hosts.env" SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1) last_two_hosts_call import_wanted
  import_wanted="^two-hosts-local share=0/2 import=$work/home/runs/crash-point-replay-[^ ]+-beta/fetched/crash-amplification-library fetched=yes "
  # fast-tier-two-hosts：开了双机 ⇒ beta 交双机驱动：本机 0/2、第二台 1/2 取回库、最后本机带 IMPORT（指到取回的库）再跑一趟；照导入那一趟判（2 条都过）；
  # alpha 照旧本机一趟；外层的上限给双机驱动；库是给定状态那一类的长期库、跑完留着，取回的那一份删掉
  entry "${two_hosts_settings[@]}" RUN_WITH_MEMORY_CAP_OUTER_CAP=3G -- --quick
  last_two_hosts_call="$(grep -E '^two-hosts-' "$calls" | tail -n 1)"
  if [[ "$exit_code" != 0 ]] || ! grep -q 'features=verdict-store' "$root/.claude/gate.d/stage-inputs.tsv" || [[ "$(calls_count '^two-hosts-')" != 3 ]] \
     || ! calls_have '^two-hosts-local share=0/2 import=unset ' || ! calls_have '^two-hosts-peer share=1/2 fetch=crash-amplification-library$' \
     || ! [[ "$last_two_hosts_call" =~ $import_wanted ]] || calls_have ' --test beta -- --nocapture ' || ! calls_have ' --test alpha -- --nocapture ' \
     || ! item_has beta 'exit=0 verdict=green fast_tier_tests=2 ' || ! out_has 'crash-amplification-two-hosts.sh beta --library ' || ! out_has ' --memory-cap 3G' \
     || [[ -e "$work/home/crash-point-replay-fast-tier" ]] || ! calls_have "^two-hosts-local share=0/2 import=unset fetched=no library=$work/home/$CRASH_LIBRARY_DIRECTORY_NAME " \
     || [[ ! -e "$work/home/$CRASH_LIBRARY_DIRECTORY_NAME/CURRENT" ]] || compgen -G "$work/home/runs/crash-point-replay-*-beta/fetched" >/dev/null; then
    cell_failed fast-tier-two-hosts "开了双机，经给定状态引擎的测试目标应当照双机驱动的三步跑（本机 0/2、第二台 1/2 取回库、本机带 IMPORT 指到取回的库再跑，照导入那一趟判），别的测试目标照旧本机一趟，外层的上限给双机驱动，库用那一份长期库 $work/home/$CRASH_LIBRARY_DIRECTORY_NAME、跑完留着、取回的那一份删掉（最后一笔双机调用：${last_two_hosts_call:-没有}）" \
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
     || ! out_has '判法 crash-point-replay-shard-configuration-check.sh 判不过'; then
    cell_failed fast-tier-two-hosts-configuration-refused "开了双机而判法判不过（多机配置缺键），经给定状态引擎的测试目标应当一趟都不起、记本次未跑、整趟退 78" \
      "看 decide_fast_tier_two_hosts 里判法退别的码那一支"
  fi
  # fast-tier-slice-states-sent：快档那一段不分档，调用方没设 ⇒ 每一趟 cargo test 收到 65536（--quick、剪枝、--full 三种跑法）；调用方设了 131072 ⇒ 照原值收到（--quick、剪枝）；
  #   开了双机、调用方没设 ⇒ 交双机驱动的 beta 那几趟里本机那几趟（假的本机那一趟）与第二台那一趟（假的 multi-host-run.sh 收到的命令）都收到 65536，本机一趟的 alpha 同样收到
  local fast_slice_wrong=() fast_slice_case fast_slice_want fast_slice_setting fast_slice_arguments fast_slice_settings fast_slice_local_runs
  for fast_slice_case in "65536|-|--quick" "65536|-|" "65536|-|--full" \
                         "131072|SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=131072|--quick" "131072|SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=131072|"; do
    IFS='|' read -r fast_slice_want fast_slice_setting fast_slice_arguments <<<"$fast_slice_case"
    fast_slice_settings=()
    [[ "$fast_slice_setting" == - ]] || fast_slice_settings=("$fast_slice_setting")
    # shellcheck disable=SC2086  # 参数是空格隔开的一串，逐个当参数
    entry ${fast_slice_settings[@]+"${fast_slice_settings[@]}"} -- $fast_slice_arguments
    if [[ "$exit_code" != 0 ]] || ! every_fast_run_has " slice-states=\[$fast_slice_want\] "; then
      fast_slice_wrong+=("「${fast_slice_settings[*]+${fast_slice_settings[*]} }crash-point-replay.sh $fast_slice_arguments」退 $exit_code、快档那几趟收到 $(grep -oE "^cargo test .* -- --nocapture .* slice-states=\[[^]]*\]" "$calls" | grep -oE 'slice-states=\[[^]]*\]' | sort | uniq -c | tr -s ' \n' ' ')（应当每一趟 [$fast_slice_want]）")
    fi
  done
  entry "${two_hosts_settings[@]}" -- --quick
  fast_slice_local_runs="$(calls_count '^two-hosts-local ')"
  if [[ "$exit_code" != 0 ]] || (( fast_slice_local_runs == 0 )) || [[ "$(calls_count '^two-hosts-local .* slice-states=\[65536\]$')" != "$fast_slice_local_runs" ]] \
     || [[ "$(calls_count '^peer-env slice-states=')" != 1 ]] || ! calls_have '^peer-env slice-states=\[65536\]$' || ! every_fast_run_has " slice-states=\[65536\] "; then
    fast_slice_wrong+=("「双机开、调用方没设，crash-point-replay.sh --quick」退 $exit_code、双机驱动本机那几趟与第二台那一趟收到 $(grep -oE '^(two-hosts-local|peer-env) .*slice-states=\[[^]]*\]' "$calls" | sed -E 's/^(two-hosts-local|peer-env) .*(slice-states=\[[^]]*\])$/\1 \2/' | tr '\n' ' ')（应当都是 [65536]）")
  fi
  if (( ${#fast_slice_wrong[@]} )); then
    exit_code="${#fast_slice_wrong[@]} 处"
    cell_failed fast-tier-slice-states-sent "快档那一段调用方没设 SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES 时（不分 --quick 与否，连同交双机驱动的那几趟）每一趟应当收到 $LIGHT_REGISTERED_SEGMENT_SLICE_STATES、调用方设了（131072）快档每一趟应当照原值收到：${fast_slice_wrong[*]}" \
      "看 run_fast_tier 里 fast_tier_slice_states 那一段、run_fast_tier_target_on_two_hosts 的 two_hosts_environment 与文件头「往引擎送的变量」"
  fi
  # local-only：多机配置开了双机，--local-only ⇒ beta 不交双机驱动、本机一趟；假放量入口、假门禁、假 cargo 收到的多机配置里双机关着、没有第二台的名字、显卡先后里没有第二台的项；
  #   对照（不给参数）同一份配置 ⇒ 双机驱动照起、假放量入口收到的多机配置里双机开着、有第二台的名字（假驱动确实看得到第二台，上面的「看不到」才有意义）
  entry "${two_hosts_settings[@]}" -- --quick
  if [[ "$exit_code" != 0 ]] || ! calls_have '^two-hosts-local ' || ! calls_have '^pipeline-env config=\[across=1 peer=1 ' || ! calls_have '^cargo-env config=\[across=1 peer=1 '; then
    cell_failed local-only-baseline-sees-peer "对照：不给 --local-only 时双机驱动应当照起、假放量入口与假 cargo 收到的多机配置里双机开着、有第二台的名字" "看双机那几格的现场：two_hosts_settings 与假驱动的配置摘要"
  fi
  local local_only_wrong=() local_only_case
  for local_only_case in "--quick" "--item alpha --item beta"; do
    # shellcheck disable=SC2086  # 参数是空格隔开的一串，逐个当参数
    entry "${two_hosts_settings[@]}" -- $local_only_case --local-only
    if [[ "$exit_code" != 0 ]] || calls_have '^two-hosts-' || ! calls_have ' --test beta -- --nocapture ' || ! item_has beta 'exit=0 verdict=green' \
       || ! calls_have '^cargo-env config=\[across=0 peer=0 ' || ! calls_have '^pipeline-env config=\[across=0 peer=0 ' \
       || calls_have '^(pipeline|cargo|enumeration)-env config=\[across=[1-9]' || calls_have '^(pipeline|cargo|enumeration)-env config=\[across=0 peer=[1-9]' \
       || { [[ "$local_only_case" != --quick ]] && ! calls_have '^enumeration-env config=\[across=0 peer=0 '; }; then
      local_only_wrong+=("「crash-point-replay.sh $local_only_case --local-only」退 $exit_code、调用记录 $(tr '\n' '|' <"$calls" | cut -c1-300)")
    fi
  done
  if (( ${#local_only_wrong[@]} )); then
    exit_code="${#local_only_wrong[@]} 处"
    cell_failed local-only "给了 --local-only，开了双机的多机配置也应当不起双机驱动、经给定状态引擎的测试目标本机一趟，快档（假 cargo）、门禁（假）与放量入口（假）收到的多机配置里双机关着、没有第二台的名字：${local_only_wrong[*]}" \
      "看「GPU 与配置」那一段的 make_local_only_configuration（派生一份关了双机的多机配置，导出 SINGLEFS_MULTI_HOST_CONFIG）与文件头参数表"
  fi
  # local-only-keeps-gpu-drops-peer-cards：双机与 GPU 都开、显卡先后里第二台点了名 ⇒ --local-only 下快档仍带 gpu 特性、经清场包装起，交清场之后那一份里本机的卡（local:0:24G），第二台的卡去掉
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-two-hosts-gpu.env" SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 -- --quick --local-only
  if [[ "$exit_code" != 0 ]] || calls_have '^two-hosts-' || calls_have 'cards=\[[^]]*peer:' || ! calls_have '^cargo-env config=\[across=0 peer=0 cards=0\]' \
     || [[ "$(calls_count "^cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --test [a-z0-9_]+ -- --nocapture --test-threads 1 cards=\[local:0:24G\] classes=\[unset\] quiesced=\[yes\]")" != $((3 + extras_count)) ]]; then
    cell_failed local-only-keeps-gpu-drops-peer-cards "--local-only 下 GPU 照旧开（verdict-store,gpu、清场包装、测试参数带 --test-threads 1）、显卡先后只剩本机的卡 local:0:24G，第二台的卡不进快档" \
      "看 make_local_only_configuration 对 GPU_CARDS_BY_PRIORITY 与 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED 去掉 peer: 项的那一段"
  fi
  # local-only-on-peer-keeps-own-cards：同一份配置在第二台上跑（角色 peer；起活那一台照配置清过场、回读过了，清场包装照 GPU_QUIESCE_ORIGIN_READ_BACK=1 起命令）
  #   ⇒ --local-only 下快档交的显卡先后只剩这一台的卡 peer:0:16G（清场之后那一份里 peer: 那几项），起活那一台的 local: 项去掉；双机驱动不起
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-two-hosts-gpu.env" SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE=1 \
    SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer GPU_QUIESCE_ORIGIN_READ_BACK=1 -- --quick --local-only
  if [[ "$exit_code" != 0 ]] || calls_have '^two-hosts-' || calls_have 'cards=\[[^]]*local:' || ! calls_have '^cargo-env config=\[across=0 peer=0 cards=2\]' \
     || [[ "$(calls_count "^cargo test --release -p singlefs-checker-tier --features verdict-store,gpu --test [a-z0-9_]+ -- --nocapture --test-threads 1 cards=\[peer:0:16G\] classes=\[unset\] quiesced=\[yes\]")" != $((3 + extras_count)) ]]; then
    cell_failed local-only-on-peer-keeps-own-cards "角色 peer（在第二台上跑）时 --local-only 下快档交的显卡先后应当只剩这一台的卡 peer:0:16G、去掉起活那一台的 local: 项（GPU、清场包装、--test-threads 1 照旧）" \
      "看 make_local_only_configuration 按 SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE 换读的那一段：角色 peer 时留 peer: 项、去 local: 项"
  fi
  # local-only-usage：--local-only 与 --list-items 一起给 ⇒ 退 2、什么都不起
  entry -- --list-items --local-only
  [[ "$exit_code" == 2 && ! -s "$calls" ]] || cell_failed local-only-usage "--local-only 与 --list-items 一起给应当退 2、什么都不起" "看参数那一段 --list-items 的互斥核"
  cp -- "$work/stage-inputs-plain.tsv" "$root/.claude/gate.d/stage-inputs.tsv"
  # flow-coverage-table：轻型跑完，这一趟的日志目录里落了覆盖表与记录（scripts/crash/crash-flow-coverage.py 的 run-directory、table），快档打的覆盖行在里面
  entry STUB_FLOW_LINE=1 -- --quick
  local flow_run_directory
  flow_run_directory="$(ls -td "$work/home/runs"/crash-point-replay-* 2>/dev/null | head -n 1)"
  if [[ "$exit_code" != 0 || -z "$flow_run_directory" ]] || ! grep -q '| selftest_entry' "$flow_run_directory/crash-flow-coverage.md" 2>/dev/null \
     || ! grep -q $'^CRASH_FLOW_RECORD\tcoverage\talpha\tselftest_entry\t' "$flow_run_directory/crash-flow-records.tsv" 2>/dev/null; then
    cell_failed flow-coverage-table "跑完应当在这一趟的日志目录里落覆盖表 crash-flow-coverage.md 与记录 crash-flow-records.tsv，快档打的覆盖行在里面（日志目录 ${flow_run_directory:-没找到}）" \
      "看 summarize 之后落覆盖表那一段与 scripts/crash/crash-flow-coverage.py 的 run-directory、table"
  fi
  # rocksdb-entry-refused：小仓的 checker 档包依赖 rocksdb、进程环境里四个取值是空的，PATH 最前面的假 cargo 不带 rocksdb-prebuilt.py 认的标记
  #   ⇒ 起任何 cargo 之前 rocksdb-prebuilt.py entry 判红、退 78，假 cargo 与两套驱动一次都没起（别的格小仓里没有这份清单，entry 报不判）
  printf '%s\n' '[package]' 'name = "singlefs-checker-tier"' 'version = "0.0.0"' '[dependencies]' 'rocksdb = { version = "=0.25.0", optional = true }' \
    >"$root/crates/singlefs-checker-tier/Cargo.toml"
  entry ROCKSDB_LIB_DIR= ROCKSDB_STATIC= SNAPPY_LIB_DIR= SNAPPY_STATIC= -- --quick
  if [[ "$exit_code" != 78 ]] || [[ -s "$calls" ]] || ! out_has "rocksdb-prebuilt entry：" || ! out_has "这一趟不起 cargo"; then
    cell_failed rocksdb-entry-refused "checker 档包依赖 rocksdb 而进程环境里没有预编库的取值，应当在起 cargo 之前退 78、一个 cargo 与驱动都不起、输出里有 rocksdb-prebuilt entry 的红句" \
      "看定档之后 replay_rocksdb_entry_holds 那一段（scripts/hosts/rocksdb-prebuilt.py entry）"
  fi
  rm -f -- "$root/crates/singlefs-checker-tier/Cargo.toml"
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
  echo "  ✓ --selftest $cells 格都对：列项不起编译、项名写错与缺项名与 --quick --full 退 2、轻型照甲二与展开 6 跑全部项（放量不交 --library、照默认长期库，接连两趟第二趟复用头一趟判过的块）、--quick --item 照跑核心部分、"
  echo "    开了 GPU 快档带 gpu 特性经清场包装起并交清场之后的显卡先后（不送走 GPU 的计算类）、"
  echo "    每一趟带按段登记文件的绝对路径而它核不过退 2 不起、快档红退 1、跑到 0 条与没卡记本次未跑、全是本次未跑退 77、重型剪枝每项交崩溃点重放 --full --item 与放量全量、"
  echo "    逐条只跑点名的项、全量崩溃点重放带 --start-over 只写不读、放量换新库（库目录留给放量入口建）、快档的复用数照复用计数行报（没有那一行报取不出、成功句点名）、双机时给定状态引擎的测试目标用给定状态那一类的长期库、崩溃点重放红与放量红各红在那一项、放量收不回判红、平常跑带只供自证的变量退 2、轻型每一件在重型里都有（真合表与小仓），
    快档日志里有给定状态表借不到显存那一行记本次未跑退 78、套在内存包装里把外层的上限给放量入口与双机驱动、
    轻型起放量入口时调用方没设就把分片的段一趟判的态数设成 $LIGHT_REGISTERED_SEGMENT_SLICE_STATES（设了照用、不带 --quick 不设）、快档那一段不分档同样设（设了照用，连同交双机驱动的本机与第二台那几趟）、开了双机时经给定状态引擎的测试目标照双机驱动三步跑
    （本机 0/2、第二台 1/2 取回库、本机带 IMPORT 指到取回的库再跑，双机驱动是真的、两趟是假的）而双机关着照旧本机一趟、第二台连不上与判法判不过记本次未跑退 78、
    快档红了重型不起崩溃点重放部分与放量部分（两段记本次未跑：快档红）而轻型照旧往下跑、放量入口退 78 记本次未跑（红先判）而退 1 照旧红、
    --local-only 下假驱动看不到第二台（多机配置派生一份关了双机的，显卡先后只剩本机的卡，GPU 照开；角色 peer 时只剩第二台自己的卡），与 --list-items 一起给退 2，
    跑完运行目录里不剩它建的编译目录（正常、快档红、被 TERM 停）而调用方设的 CARGO_TARGET_DIR 不删，
    积压跑法照积压登记表取件（两条下限交给小程序 plan、取到两条下限就停、不足全取、这一里程碑新增与判法键变了的先跑、每件回写全绿与态数、红的回写成红、本次未跑的不回写、
    用法错与表核不过退 2 不起、plan 的输出对不上退 1 不跑）而 --list-items --backlog 只起表的小程序（游标、到两条下限各跑到哪一行）、不改表，"
  echo "    （cargo、两套驱动与准入模块都是假的，不编不跑）；开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读）逐个打开，派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的格"
  exit 0
}

if (( selftest )); then
  (( quick || full || list_items || with_keys || local_only || backlog || ${#named_items[@]} || ${#backlog_milestone} || ${#backlog_backlog_bound} || ${#backlog_total_bound} )) && usage "--selftest 不与别的参数一起给"
  run_selftest
fi
(( with_keys && ! list_items )) && usage "--with-keys 只跟 --list-items 一起给"
# 积压跑法的用法（文件头「积压跑法」）：--milestone 写成 m<号>、跟 --backlog 一起给；不与 --quick、--full、--item 一起给；两条下限只跟 --backlog。弄坏开关 backlog-usage-unchecked 下不核
if ! replay_break_is backlog-usage-unchecked; then
  if (( backlog )); then
    [[ "$backlog_milestone" =~ ^m[0-9]+$ ]] || usage "--backlog 要带 --milestone m<号>（这一里程碑，例 m4），给的是「$backlog_milestone」"
    (( quick || full || ${#named_items[@]} )) && usage "--backlog 不与 --quick、--full、--item 一起给（取哪几件由积压登记表定）"
  elif [[ -n "$backlog_milestone$backlog_backlog_bound$backlog_total_bound" ]]; then
    usage "--milestone、--backlog-states-at-least、--states-at-least 只跟 --backlog 一起给"
  fi
fi

# --list-items --with-keys：崩溃判定库里键对不上的块（只读读库程序 crash_library_reader keys；它的输出写法在
# crates/singlefs-checker-tier/src/bin/crash_library_reader.rs 文件头「输出」那一段）。读库程序退非 0、输出认不出照实报，交回 1。
report_the_keys_of_the_library() {
  local output reader_exit tag first second third fourth fifth seen=0
  if (( replay_in_selftest_child )) && [[ -n "$replay_keys_reader_output" ]]; then
    output="$(cat -- "$replay_keys_reader_output")"; reader_exit=$?
  else
    output="$(cd "$repository_root" && cargo run --release --quiet --offline -p singlefs-checker-tier --bin crash_library_reader -- keys 2>&1)"; reader_exit=$?
  fi
  if (( reader_exit != 0 )); then
    printf '%s\n' "$output" | tail -n 5 >&2
    echo "  ✗ 读库程序 crash_library_reader keys 退 $reader_exit：键对不上的块数报不出" >&2
    echo "  → 怎么办：照上面那几行修（库是这一套之前的布局就换一个新目录建库）；单跑 cargo run --release -p singlefs-checker-tier --bin crash_library_reader -- keys" >&2
    return 1
  fi
  while IFS=$'\t' read -r tag first second third fourth fifth; do
    case "$tag" in
      CRASH_LIBRARY_KEYS)
        seen=1
        if [[ "$second" == missing ]]; then
          printf '键\t崩溃判定库 %s 不在：一块都没判过\n' "${first#library=}"
          continue
        fi
        if [[ "$first" != library=* || "$second" != recorded_blocks=* || "$third" != judged_under_the_current_key=* \
              || "$fourth" != judged_only_under_another_key=* || "$fifth" != never_judged=* ]]; then
          echo "  ✗ 读库程序那一行认不出：$tag $first $second $third $fourth $fifth（写法与 crash_library_reader.rs 文件头「输出」那一段对不上）" >&2
          echo "  → 怎么办：让 report_the_keys_of_the_library 与 crates/singlefs-checker-tier/src/bin/crash_library_reader.rs 文件头「输出」那一段的写法对上，再跑 --list-items --with-keys" >&2
          return 1
        fi
        printf '键\t崩溃判定库 %s：录下的块 %s，键对不上的块 %s（只在别的判法单元指纹下判过 %s、一种判器的结论都没有 %s），键相同的 %s\n' \
          "${first#library=}" "${second#recorded_blocks=}" "$(( ${fourth#judged_only_under_another_key=} + ${fifth#never_judged=} ))" \
          "${fourth#judged_only_under_another_key=}" "${fifth#never_judged=}" "${third#judged_under_the_current_key=}" ;;
      CRASH_LIBRARY_KEYS_JUDGE)
        printf '键\t%s 这一份的判法单元指纹 %s\n' "$first" "${second#current_key=}" ;;
      *) ;;
    esac
  done <<<"$output"
  if (( ! seen )); then
    echo "  ✗ 读库程序的输出里没有 CRASH_LIBRARY_KEYS 那一行（写法与 crash_library_reader.rs 文件头「输出」那一段对不上）" >&2
    echo "  → 怎么办：看读库程序的输出是不是编不过或换了写法，照 crates/singlefs-checker-tier/src/bin/crash_library_reader.rs 文件头「输出」那一段对上，再跑 --list-items --with-keys" >&2
    return 1
  fi
}

load_items
if (( list_items )); then
  (( quick || full || ${#named_items[@]} )) && usage "--list-items 不与别的参数一起给"
  (( local_only )) && ! replay_break_is local-only-accepted-with-list-items && usage "--local-only 不与 --list-items 一起给（--list-items 只读合表，不碰任何一台机器）"
  replay_break_is list-items-builds && cargo build --release -p singlefs-checker-tier >/dev/null 2>&1
  for name in "${item_names[@]}"; do
    light_text="甲二 ${item_light_targets[$name]}"
    [[ -n "${item_pipeline_items[$name]}" ]] && light_text+=" ∪ 展开 $LIGHT_EXPAND_UP_TO ${item_pipeline_items[$name]}"
    heavy_text="${item_cases[$name]:+崩溃点重放 ${item_cases[$name]}}"
    [[ -n "${item_pipeline_items[$name]}" ]] && heavy_text+="${heavy_text:+ ∪ }放量 ${item_pipeline_items[$name]}"
    printf '%s\t轻型：%s；重型：快档 %s ∪ %s\n' "$name" "$light_text" "${item_light_targets[$name]}" "$heavy_text"
  done
  if (( with_keys )); then
    # 读库程序是 cargo run 起的（checker 档包，编到 librocksdb-sys）：起之前判取值，判红退 1；自证喂读库输出的那一趟不起 cargo、不判
    if ! (( replay_in_selftest_child )) || [[ -z "$replay_keys_reader_output" ]]; then replay_rocksdb_entry_holds || exit 1; fi
    report_the_keys_of_the_library || exit 1
  fi
  if (( backlog )); then
    # 只读·积压（文件头「积压跑法」末段）：表的小程序编到 librocksdb-sys，起它之前判 RocksDB 预编库的取值，判红退 1
    replay_rocksdb_entry_holds || exit 1
    backlog_listing || exit 1
  fi
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
if (( backlog )); then
  tier=backlog; tier_title="重型·积压（里程碑 $backlog_milestone，照积压登记表取件）"; selected_items=()   # 项由 backlog_prepare_plan 照取到的件填
elif (( quick )); then
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

# 往引擎送的变量（文件头「往引擎送的变量」）：按段登记文件先核、再送绝对路径
segments_registry="$repository_root/.claude/gate.d/crash-point-replay-segments.tsv"   # 路径同 lib_crash_point_replay_items.py 的 SEGMENTS_TABLE
if ! replay_break_is segments-registry-unchecked; then
  if ! segments_listing="$(python3 "$replay_script_directory/lib_crash_point_replay_items.py" segments "$repository_root")"; then
    printf '%s\n' "$segments_listing" >&2
    echo "  ✗ 按段登记文件 $segments_registry 核不过（lib_crash_point_replay_items.py segments），什么都不跑" >&2
    echo "  → 怎么办：照上面逐处列的改 .claude/gate.d/crash-point-replay-segments.tsv（写法在它的表头），再跑" >&2
    exit 2
  fi
fi
# 起任何 cargo 之前（快档、崩溃点重放那一道、放量入口、双机驱动都在这之后起）判 RocksDB 预编库的取值：判红退 78、什么都不起
if ! replay_rocksdb_entry_holds; then
  echo "  ✗ 起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），这一趟一项都没起，退 78" >&2
  echo "  → 怎么办：照上面那几行的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再跑同一条命令" >&2
  exit 78
fi
if replay_break_is segments-registry-not-sent; then
  unset SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY
else
  export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY="$segments_registry"
fi

# GPU 与配置（文件头「GPU」）
configuration_file="$(crash_amplification_multi_host_configuration_file "$repository_root")"
if (( local_only )) && ! replay_break_is local-only-ignored; then make_local_only_configuration; fi
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
# 这一趟的编译目录（文件头「编译目录」）：收尾的 EXIT trap（replay_finish）删
run_build_directory_begin "$run_directory" || exit 78
trap replay_finish EXIT
trap 'exit 143' TERM
trap 'exit 130' INT
part_records="$run_directory/parts.tsv"; : >"$part_records"
plan_file="$run_directory/plan.tsv"
if [[ "$tier" == backlog ]]; then
  backlog_prepare_plan   # 文件头「积压跑法」①到④：核表、取件、写计划；退不了的在里面直接退（收尾由 EXIT trap 管）
else
  replay_plan "$tier" "$with_groups" ${selected_items[@]+"${selected_items[@]}"} >"$plan_file"
fi
echo "── 崩溃点重放$tier_title：${#selected_items[@]} 项（多机配置 $configuration_file：GPU $( (( gpu_switch )) && echo 开 || echo 关)$( (( local_only )) && echo '，只用本机、第二台不动')），日志 $run_directory/（每份 .log 旁边有同名的 .problems.log：只收错误与警告，行前带全量日志里的行号，一般只看它）"

# 计划里直接记本次未跑的（两档都跑的那几类里没有测试目标的）
while IFS=$'\t' read -r plan_item plan_kind plan_object; do
  [[ "$plan_kind" == not-run ]] && record_part "$plan_item" not-run - - not-run 0 0 none 0 "$plan_object"
done <"$plan_file"
run_fast_tier "$plan_file"
# 重型：快档先过才起崩溃点重放部分与放量部分，快档红了两段都不起（文件头「一项怎么判」）；轻型没有崩溃点重放部分，放量部分照旧跑
if [[ "$tier" != quick ]] && ! replay_break_is fast-tier-red-continues \
   && awk -F'\t' '$2 == "fast-tier" && $5 == "red" { found = 1 } END { exit !found }' "$part_records"; then
  skip_parts_after_red_fast_tier "$plan_file"
else
  if [[ "$tier" == backlog ]]; then
    run_backlog_parts   # 照取件的次序逐件跑、逐件回写（文件头「积压跑法」④⑤）
  elif [[ "$tier" != quick ]]; then
    for name in ${selected_items[@]+"${selected_items[@]}"}; do
      [[ -n "${item_cases[$name]}" ]] || continue
      # shellcheck disable=SC2086  # 键是空格隔开的一串，逐个当参数
      run_enumeration_part "$name" ${item_cases[$name]}
    done
  fi
  [[ "$tier" == backlog ]] || run_pipeline_part "$plan_file"
fi
summarize "$plan_file" "$tier_title"
[[ "$tier" != backlog ]] || backlog_summarize
echo "  · 错误警告日志（每份全量日志旁边同名的 .problems.log，一般只看它）：$(find "$run_directory" -maxdepth 1 -name '*.problems.log' | LC_ALL=C sort | tr '\n' ' ')"
# 覆盖行与计数行落成产物（文件头「输出」末段）：只落产物、不判红、不改退出码；弄坏开关 flow-coverage-table-skipped 下照改前不落
if ! replay_break_is flow-coverage-table-skipped; then
  flow_coverage_judge="$replay_script_directory/crash-flow-coverage.py"
  # 积压跑法照点名的重型记（逐件点名、整域；crash-flow-coverage.py 的 --tier 只认 quick、pruned、named、full 四档）
  if python3 "$flow_coverage_judge" run-directory "$run_directory" --tier "${tier/#backlog/named}" >"$run_directory/crash-flow-records.tsv" 2>"$run_directory/crash-flow-coverage.err" \
     && python3 "$flow_coverage_judge" table "$repository_root" "$run_directory/crash-flow-records.tsv" >"$run_directory/crash-flow-coverage.md" 2>>"$run_directory/crash-flow-coverage.err"; then
    echo "  · 覆盖表（入口 × StepKind、ignore 开着的节点、评估 0 次的对）：$run_directory/crash-flow-coverage.md；记录 $(grep -c . "$run_directory/crash-flow-records.tsv") 条：$run_directory/crash-flow-records.tsv"
  else
    echo "  ! 覆盖表没落成（$(tail -n 1 "$run_directory/crash-flow-coverage.err")；判法 $flow_coverage_judge），这一趟的判定不受影响" >&2
  fi
fi
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」，.claude/hooks/run-scope-guard.sh 的拒绝说明读它）：红的项一行一个；
# 重型点名了项的（--item，不带 --quick）只换这几项，轻型与不点名的重型照这一趟整份写
if [[ -f "$run_directory/exit.red-items" ]]; then
  if [[ "$tier" != quick ]] && (( ${#named_items[@]} > 0 )); then
    printf '%s\n' "${named_items[@]}" >"$run_directory/last-run.ran"
    python3 "$replay_script_directory/../gate/lib_reuse_checks.py" last-run scripts/crash/crash-point-replay.sh "$run_directory/exit.red-items" --ran "$run_directory/last-run.ran" >/dev/null
  else
    python3 "$replay_script_directory/../gate/lib_reuse_checks.py" last-run scripts/crash/crash-point-replay.sh "$run_directory/exit.red-items" >/dev/null
  fi || echo "  ! 上次红的清单没写成（scripts/gate/lib_reuse_checks.py last-run），这一趟的判定不受影响" >&2
fi
exit "$(cat "$run_directory/exit" 2>/dev/null || echo 1)"
