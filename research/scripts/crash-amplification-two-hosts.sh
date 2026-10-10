#!/usr/bin/env bash
# admission: always 每次调都在这一刻的树上两台各跑一份、拷回、导入、对账，上一次的结论不替这一次作保
# run-condition: command cargo ssh rsync git
# 崩溃放量流水线的双机跑法（里程碑二「增补 4」第一、四、十一项；用户 2026-09-28 定四种跑法只由配置的 ENABLE_ACROSS_MACHINES、ENABLE_GPU 两个开关定）：
#
#   crash-amplification-two-hosts.sh <测试目标> [--test-filter <用例函数名>] [--library <本机库目录>] [--expand-up-to <N>] [--threads <N>] [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>]
#                                    [--host-weights <本机>,<第二台>] [--local-states-per-second <本机量到的速度>] [--quiesce] [--quiesced]
#   crash-amplification-two-hosts.sh --selftest      拿假的本机那一趟、假的第二台（换掉 multi-host-run.sh）与假的清场走一遍分份、快照、取回、导入、对账、
#                                                    先判着再按速度分、出错的几种与退出码的映射（不编、不起 cargo、不连第二台、不上卡；格名见下面「弄坏开关」）；
#                                                    末尾一轮（开关轮）从下面「弄坏开关」那张表现读开关与点名的格，逐个打开再跑一遍 --selftest，核红的格与点名的逐格相等
#
# 测试目标是 crates/singlefs-checker-tier/tests/ 下一条走 crash_amplification::run_from_environment 的用例（例 crash_amplification_fixed_script_through_rollback），
# 或经给定状态引擎判的随机历史用例（崩溃点重放入口 research/scripts/crash-point-replay.sh 的快档开了双机时照这三步起它们：引擎读同样的 SHARE、LIBRARY、IMPORT）。
# 步骤：① 本机 SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/2、库写到 --library（没给用 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-~/.local/share/singlefs/crash-amplification}/library-by-conclusion-kind，
#          crash-amplification-lib.sh 文件头「判定库」：按结论种类存之前的旧库 <家目录>/library 引擎开库就拒，不删、不读；
#          在 /tmp 之外：重启不清、重跑同一条命令就是续跑；用例里的 run_from_environment 拒绝落在 /tmp、/var/tmp、/dev/shm、/run 下的库），后台跑（先 6 路并行只编、再按线程数跑）；
#       ② 第二台 SHARE=1/2、库写到那棵树副本树根下的 crash-amplification-library/（相对路径由测试按仓根解析），经 research/scripts/multi-host-run.sh --peer --fetch 跑并取回
#          （树怎么拷、内存上限、工具链比对、脱离 ssh 会话、私有字换成「第二台」都归它）；树副本不带 multi-host.env，
#          两个开关 ENABLE_ACROSS_MACHINES / ENABLE_GPU 随命令的环境送过去（测试在配置不在时读同名环境变量）；
#       ③ 等本机那一份；④ 本机再跑一趟 SHARE=0/2 带 SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<取回的库>：导入第二台的块、对账、读违例——
#          用例在导入之后断言对账为空、违例为空，它退 0 这一趟才算全绿。② 失败（第二台被停、掉线、被杀）而库取回了一部分，也先导入再退：
#          第二台判到一半的块不丢，下一次同一条命令接着跑。--test-filter 只跑测试目标里点名的那一条用例（cargo test 的 --exact）。
# 跑法：配置 ENABLE_GPU=1 两台都带 gpu 特性编，各用多机配置里显卡先后（GPU_CARDS_BY_PRIORITY）点名给自己的卡、各在额度之内（用例里的 run_from_environment 做；
#       第二台的树副本不带配置，那个键与 SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer 随命令的环境送过去）；不是 1 只带 verdict-store。
#       走 GPU 的计算类：开了 GPU 引擎就三类全走 GPU，它不读 GPU_COMPUTATION_CLASSES（crash-amplification-lib.sh 文件头「配置」），
#       两台的每一趟（本机那几趟与第二台排计划、分份那两趟）都不送它，开不开 GPU 都一样；本机那几趟清掉调用方环境里漏进来的同名变量。
#       两台怎么分：--host-weights 给了照它；给了 --local-states-per-second（本机排派活计划时试判量到的速度）就先让第二台也排一趟计划、量它的速度，
#       按两个速度分；都没给就等分。第二台量速度的那段时间（它要拷树、从头编、编内核）本机不空等：本机先领整份判着，
#       第二台量完就叫它停（建停下文件，它判完手里那一块、写进库就收尾），再按速度分工——判过的块在库里，不重判。
#       这里默认不清场（卡在额度之内用）；全量由 crash-amplification.sh 开跑就清场、带 --quiesced 调到这里。单独要清场显式带 --quiesce：照配置里的清场命令先停，跑完不复原、退出时报一句
#       「清了场、没复原」（peer-host-lib.sh 的 peer_host_quiesce / peer_host_quiesce_left；要复原由用户手动跑私有复原脚本）。--quiesced：调用方已经清过场；这里只把显卡的先后与额度
#       换成清场之后那一份（多机配置的 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED）。
# 往引擎送的两个变量（crash-point-replay.sh 文件头「往引擎送的两个变量」）：SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER（本机设了才送、值原样）与
#   SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY（本机设了就把那份按段登记文件拷一份快照经 --send 送进第二台那棵树的 crash-amplification-segments-registry/，
#   变量值在第二台上是那一份的绝对路径）：第二台排计划、分份那两趟都送；本机那几趟从调用方环境继承。
# 线程（crash-amplification-lib.sh 文件头「现算线程」）：第二台排计划、分份那两趟起 capped.sh 的线程数与那两趟判的线程数（SINGLEFS_CRASH_AMPLIFICATION_THREADS），
#   各在起那一趟之前现算、两样取同一个数：CRASH_AMPLIFICATION_PEER_THREADS 给了就用它（正整数），没给取 research/scripts/host-resources.py --json
#   里第二台此刻的可用线程（至少 1）；--threads 只管本机跑的那一步（没给是本机核数减一），不带给第二台。
#   本机只编那一步同样现算（CRASH_AMPLIFICATION_BUILD_THREADS）。算不出退 78，那一趟与本机先判着、本机那一份都还没起。
# ENABLE_ACROSS_MACHINES 不是 1 由 multi-host-run.sh 拒（退 255），这里不偷偷退回单机。
# 输出：三趟各自的 CRASH_AMPLIFICATION 行原样打出（第二台那一趟经 multi-host-run.sh 已换掉私有的字；先判着那一趟行尾加 warm_up=true、导入那一趟行尾加 import_run=true，见下）；日志放 <库目录的上层>/runs/<标签>/（--selftest 的全在一个 mktemp 目录里，退出时删掉）。
# 退出码：④ 那一趟 cargo test 的退出码；① 或 ② 失败退它们的退出码（先失败的那一个），一步都没跑退 2；250–254 是内存包装自己的结局；
#         78 运行条件不满足（出路：修好设备，或改配置）：说是清过场却没在多机配置里写清场之后那一份显卡先后、
#            --quiesce 时多机配置判不过或清场没过、
#            第二台那一趟 multi-host-run.sh 自己没跑成（它退 255：配置、工具链、拷树、连不上、等不到结局）、第二台的库没取回、
#            第二台那两趟或本机只编那一步的线程数现算不出（文件头「线程」），
#            或哪一趟的日志里有卡用不了的原话、给定状态表的判器借不到显存那一行（crash-amplification-lib.sh 的 crash_amplification_failure_kind 判 device），
#            或这份二进制跑不了这一趟（结论版本与树对不上、开了 GPU 而没编进 gpu 特性；同一个函数判 build）；
#         248 算法上判不了：哪一趟的日志里有有状态 GPU 判不了、内核展开超预算、建不出表的原话（同一个函数判 algorithm）；
#         249 给了 --local-states-per-second 而第二台没量出速度、日志里认不出原因（打一行 CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED；本机先判着的块在库里，第二台一块都还没判）；
#         77  本次未跑：第二台排派活计划那一趟退 0、没打计划行，日志里有行首为 NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason= 的那一行
#             （第二台起过卡池、池里的卡等满了也借不到显存；行的写法同 crash-amplification-item.sh 文件头步骤 ②）。打一行
#             CRASH_AMPLIFICATION_NOT_RUN step=peer-placement test=crash_amplification::run_from_environment reason=<那一行 reason= 之后原样>，不分份、不改在本机一台跑；
#             本机先判着的块在库里。别的用例名的那一行、那一趟退了非 0 的，都不算这一种，照上面 249 那几种分类。
#         分类只在退非 0 的那一趟上做；这几处映射由 --selftest 的各格证红（下面「弄坏开关」），分类函数本身另由 crash-amplification-item.sh --selftest 的第 8、9、11 格证红。
# 本机先判着、第二台量完速度就叫停的那一趟（warm-local.log）：它的 CRASH_AMPLIFICATION 行带 stopped_early=true，是有意叫停、剩下的由两台分份判完，
#   转打时在行尾加 warm_up=true，崩溃放量入口据此不把这一项报成「本次未跑」，复用数也只取这一行的（入口文件头「复用数怎么取」）。
# 导入那一趟（import.log，含本机或第二台那一份红了之后先导入的那一趟）：它的 CRASH_AMPLIFICATION 行转打时在行尾加 import_run=true。
#   那一趟的 states_reused= 是导入之前本机那一份已判的，崩溃放量入口据此不把它算进复用数（flow= 行不带 share= 与 imported_blocks=，只认得这个标记）。
# 重型：跑 checker 档用例，只在用户要求时跑（SINGLEFS_HEAVY_TESTS=user-request 由这里带给三趟 cargo）；--selftest 不起 cargo，不算重型。
#
# 只供测试（--selftest 给它起的每一趟里层设这几个；与 crash-amplification-item.sh 的同类变量一样，平常跑时设了也照认）：
#   CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN=<脚本>      换掉本机那一趟（调法同 crash-amplification-lib.sh 的 crash_amplification_run_on_this_host：
#                                                       <脚本> <share> <库目录> <日志> [<附加环境 键=值>…]，自己写日志）；
#   CRASH_AMPLIFICATION_TWO_HOSTS_PEER_RUN=<脚本>       换掉 multi-host-run.sh（调法与它相同：--peer --label … [--fetch …] [--send …] [--memory-cap …] -- env <键=值…> <命令…>，
#                                                       取回的目录落 MULTI_HOST_RUN_FETCH_DIRECTORY）；
#   CRASH_AMPLIFICATION_TWO_HOSTS_PEER_HOST_LIB=<文件>  换掉 peer-host-lib.sh（--quiesce 用的 peer_host_load_configuration、peer_host_quiesce、peer_host_quiesce_left）。
#   CRASH_AMPLIFICATION_TWO_HOSTS_SWITCH_TABLE=<文件>   --selftest 末尾的开关轮从这份文件的文件头读开关表（不设读本脚本自己的），代码里认的开关照旧从本脚本取：
#                                                       拿一份改坏了表的拷贝（删一格、多一格、格名写错、漏一个开关）证开关轮会红。只在 --selftest 的开关轮里读。
#   CRASH_AMPLIFICATION_HOST_RESOURCES=<脚本>           换掉 research/scripts/host-resources.py（crash-amplification-lib.sh 文件头「现算线程」）：
#                                                       --selftest 给每一趟里层设成一份假的（python3 <脚本> --json 打造出来的两台，STUB_LOCAL_THREADS、STUB_PEER_THREADS
#                                                       是两台的 available_threads，STUB_HOST_RESOURCES=fails 退 1、=peer-unread 第二台记没读到）。
#   CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED=<文件>   第二台量完速度、叫停本机先判着那一趟的那一步（建停下文件，或不建）走过之后建这份文件：
#                                                       假的先判着那一趟等「停下文件在」或「这份文件在」，后者先到就是没被叫停、当场退 101，不等满 10 秒。
#                                                       --selftest 给每一格设成格目录下的一份；停下文件在它之前建，所以两份都在时照叫停了算。
#   换不掉的：真的那一趟由 crash-amplification-lib.sh 起 cargo、multi-host-run.sh 拷树连第二台，自证不碰它们。
# 弄坏开关 CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=<项>（几个用逗号连；只给 --selftest 证红用，自证起的每一趟里层都带着调用方设的值）。
# 逐项写开关名、弄坏哪一处、打开后自证哪几格红（格名是 run_selftest 里 run_cell 的第一个参数，或不起里层的 switch-wheel-judges；判错的那一格打「✗ 自证 <格名>：」）。
# 写法：一行「#   <变量>=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（、隔开；括号里是说明，开关轮不读，括号外的每一段都算一格）。
# --selftest 末尾的开关轮从这一段与下面「共用库」那一段现读（crash-amplification-lib.sh 的 crash_amplification_read_switch_table），判法是同一份库的 crash_amplification_switch_wheel
# （与崩溃放量入口共用）：表里的双机开关与代码里 two_hosts_break_is 认的要相等、表里的共用库开关要是共用库里认的，每个逐个打开、要退 1、红的格与点名的逐格相等；
# 共用库认而表里没列的开关也逐个打开，要退 0、一格不红。并行度 CRASH_AMPLIFICATION_SELFTEST_JOBS（没给取 SINGLEFS_THREAD_CAP，再没给按 nproc）。
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=same-share-on-both-hosts  第二台那一趟也领 0/2（两台判同一半）
#       peer-cross-check-skip-sent、peer-engine-variables-absent-when-unset、peer-segments-registry-sent（导入之后对账不为空、退 101）、
#       both-hosts-green、library-snapshot-sent、calibrated-weights（调用记录里没有第二台领 1/2 的那一趟）、
#       reconciliation-not-empty（没人判的是 3 块、不是漏判的那 1 块）、quiesce-not-restored（导入之后对账不为空、退 101）、
#       gpu-on-classes-not-sent、gpu-cards-from-multi-host-configuration、gpu-off-classes-not-sent（导入之后对账不为空、退 101，调用记录里也没有第二台领 1/2 的那一趟）、
#       peer-threads-from-host-resources、peer-threads-given、peer-threads-at-least-one、
#       peer-checking-threads-from-host-resources、peer-checking-threads-given、peer-checking-threads-at-least-one（导入之后对账不为空、退 101）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=skip-import               取回之后不跑导入那一趟、当它退 0
#       peer-cross-check-skip-sent、peer-segments-registry-sent（调用记录里没有带 IMPORT 的本机 0/2 那一趟）、
#       both-hosts-green（最后一趟不是带 IMPORT 的本机 0/2）、reconciliation-not-empty（退 0，不退 101）、
#       gpu-on-classes-not-sent、gpu-cards-from-multi-host-configuration、gpu-off-classes-not-sent（调用记录里没有带 IMPORT 的本机 0/2 那一趟）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=no-library-snapshot       本机库已有块也不照快照送给第二台
#       library-snapshot-sent（第二台那一趟没带 --send、库里那一块没复用）、library-snapshot-fails（不照快照也就拷不坏，退 0，不退 2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=ignore-snapshot-failure   快照拷不成也照跑
#       library-snapshot-fails（不退 2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=local-failure-ignored     本机那一份退非 0 也往下走（导入、退 0）
#       local-share-fails（退 0，不退 101）、local-share-card-unusable（退 0，不退 78）、local-share-given-states-run-condition（退 0，不退 78）、local-share-gpu-memory（退 0，不退 245）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=gpu-memory-unexplained    本机或第二台那一份退 245（内存包装认出的显存不够）时打通用出路、不说显存不够
#       local-share-gpu-memory、peer-share-gpu-memory
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=local-failure-skips-salvage  本机那一份红了就不导入第二台取回的库
#       local-share-fails（最后一趟不是带 IMPORT 的本机 0/2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-unreachable-as-run-failure  multi-host-run.sh 退 255 不映射成 78，照日志分类
#       peer-unreachable（退 255）、peer-placement-unreachable（退 249）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-failure-ignored      第二台那一份退非 0 也往下走（导入、退 0）
#       peer-share-card-unusable（退 0，不退 78）、peer-share-gpu-memory（退 0，不退 245）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-failure-skips-salvage  第二台那一份红了就不导入它取回的半份库
#       peer-share-card-unusable（最后一趟不是带 IMPORT 的本机 0/2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=accept-missing-fetched-library  第二台的库没取回也照导入
#       peer-library-not-fetched（导入那一趟找不到库、退 101，不退 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=import-failure-ignored    导入那一趟退非 0（对账不为空）也报绿
#       reconciliation-not-empty（退 0，不退 101）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=drop-host-weights         算出两台的分工也不带给两趟分份
#       calibrated-weights（分份那两趟的调用记录里 weights=none）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=no-warm-up-mark           先判着的那一趟转打时不加 warm_up=true
#       calibrated-weights（没有带 warm_up=true 的 CRASH_AMPLIFICATION 行）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=no-import-mark            导入那一趟转打时不加 import_run=true
#       both-hosts-green、local-share-fails、peer-share-card-unusable（导入那一行行尾没有 import_run=true）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=never-stop-warm-run       第二台量完速度不建停下文件
#       peer-cross-check-skip-sent、peer-engine-variables-absent-when-unset、peer-segments-registry-sent（同下退 101，调用记录里没有分份那一趟）、
#       calibrated-weights、peer-placement-unreachable、peer-placement-card-unusable、peer-placement-algorithm、peer-placement-unrecognized、
#       peer-placement-not-run、peer-placement-foreign-not-run-line、gpu-on-classes-not-sent、gpu-cards-from-multi-host-configuration、gpu-off-classes-not-sent（假的先判着那一趟看到叫停那一步走过了、停下文件却不在，退 101；各格退 101）、
#       peer-threads-from-host-resources、peer-threads-given、peer-threads-at-least-one（同上退 101，分份那一趟没起、调用记录里只有排计划那一笔线程数）、
#       peer-checking-threads-from-host-resources、peer-checking-threads-given、peer-checking-threads-at-least-one（同上退 101，第二台只记下排计划那一笔）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=warm-failure-ignored      本机先判着的那一趟退非 0 也往下走
#       warm-run-card-unusable（退 0，不退 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-placement-failure-ignored  第二台没量出速度也往下走（分工写成「本机速度,」）
#       peer-placement-unreachable、peer-placement-card-unusable、peer-placement-algorithm、peer-placement-unrecognized、peer-placement-foreign-not-run-line（都退 0）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-placement-not-run-as-failure  第二台排计划那一趟借不到显存的那一行不认，照没量出速度退 249（改前的写法）
#       peer-placement-not-run（退 249，不退 77）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-placement-accept-any-not-run-line  别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行也当第二台借不到显存
#       peer-placement-foreign-not-run-line（退 77，不退 249）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-placement-unclassified  第二台没量出速度时不照日志分类（一律 249）
#       peer-placement-card-unusable（不退 78）、peer-placement-algorithm（不退 248）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=accept-bad-host-weights   --host-weights 写错也照收
#       host-weights-malformed（不退 2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=accept-bad-local-speed    --local-states-per-second 写错也照收
#       local-speed-malformed（不退 2）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=usage-exits-1             用法错退 1（不退 2）
#       usage-missing-target、usage-unknown-option
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=prepare-failure-ignored   crash_amplification_prepare 交回非 0 也照跑（说是清过场却没在多机配置里写清场之后那一份显卡先后）
#       quiesced-without-card-list（不退 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=ignore-peer-configuration  --quiesce 时多机配置判不过也照跑
#       quiesce-configuration-fails（不退 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=ignore-quiesce-failure    --quiesce 时清场没过也照跑
#       quiesce-fails（不退 78）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=restore-after-run         --quiesce 清过场、退出时又照配置跑了复原命令
#       quiesce-not-restored（调用记录里有 restore）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=classes-sent-to-peer  开了 GPU 照改前把三类 GPU_COMPUTATION_CLASSES 送给第二台
#       gpu-on-classes-not-sent（第二台那几笔调用记录里 classes= 不是 [unset]）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=cross-check-skip-not-sent-to-peer  本机环境里设了 SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER 也不送给第二台那两趟（改前的样子）
#       peer-cross-check-skip-sent（第二台排计划、分份那两趟的 skip= 不是 [1]）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=segments-registry-not-sent-to-peer  本机环境里设了 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY 也不把那份文件送给第二台、不送变量（改前的样子）
#       peer-segments-registry-sent（第二台那两趟的 registry= 不是第二台树里那份快照的路径、或文件内容对不上）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-threads-fixed        第二台那两趟照改前写死 capped.sh 8，不现算、也不认 CRASH_AMPLIFICATION_PEER_THREADS
#       peer-threads-from-host-resources、peer-threads-given、peer-threads-at-least-one（第二台那两趟的线程数是 8）、peer-threads-unreadable（退 0，不退 78）、
#       peer-checking-threads-from-host-resources、peer-checking-threads-given、peer-checking-threads-at-least-one（第二台那两趟判的线程数也是 8）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=peer-checking-threads-from-this-host  第二台那两趟判的线程数 SINGLEFS_CRASH_AMPLIFICATION_THREADS 照改前沿用本机的（common_environment 里那一份）
#       peer-checking-threads-from-host-resources、peer-checking-threads-given、peer-checking-threads-at-least-one（第二台那两趟判的线程数是本机 --threads 的 9）
#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=wheel-ignores-cells       开关轮只比退出码、不比红的格（crash_amplification_switch_wheel 的第三个参数、crash_amplification_switch_wheel_verdict 的第五个参数给 1）
#       switch-wheel-judges（造出来的输出上删一格、多一格、格名写错、没列的开关红了都判成对上）
# 共用库 crash-amplification-lib.sh 的开关 CRASH_AMPLIFICATION_ITEM_BREAK（开关表在 crash-amplification-item.sh 文件头）打开时这里也红（共用库认而这里没列的，开关轮要它一格不红）：
#   CRASH_AMPLIFICATION_ITEM_BREAK=device-unusable-as-run-failure
#       local-share-card-unusable、peer-share-card-unusable、warm-run-card-unusable（退 101，不退 78）、peer-placement-card-unusable（退 249，不退 78）
#   CRASH_AMPLIFICATION_ITEM_BREAK=algorithm-refusal-as-run-failure
#       peer-placement-algorithm（退 249，不退 248）
#   CRASH_AMPLIFICATION_ITEM_BREAK=given-states-run-condition-as-run-failure  给定状态表的判器借不到显存那一行（CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET）不认
#       local-share-given-states-run-condition（退 101，不退 78）
#   CRASH_AMPLIFICATION_ITEM_BREAK=cards-read-from-cards-file  显卡先后照改前读退役的私有显卡配置，不读多机配置
#       gpu-cards-from-multi-host-configuration（调用记录里 cards= 是私有那一份的 local:1:3G peer:1:3G）、quiesced-without-card-list（私有那一份写了清场之后那一份，不退 78）
#   CRASH_AMPLIFICATION_ITEM_BREAK=inherited-classes-kept  本机那一趟不清掉调用方环境里漏进来的 GPU_COMPUTATION_CLASSES
#       gpu-on-classes-not-sent、gpu-off-classes-not-sent（本机那几笔调用记录里 classes=[crash_judge_gpu]）
#   CRASH_AMPLIFICATION_ITEM_BREAK=classes-sent-again  开了 GPU 照改前把三类 GPU_COMPUTATION_CLASSES 交给本机那一趟
#       gpu-on-classes-not-sent（本机那几笔调用记录里 classes= 是三类）
#   CRASH_AMPLIFICATION_ITEM_BREAK=build-threads-fixed  本机只编那一步照改前写死 capped.sh 6，不现算、也不认 CRASH_AMPLIFICATION_BUILD_THREADS
#       this-host-threads-from-host-resources、this-host-threads-given、this-host-threads-at-least-one（只编那一步的线程数是 6）、this-host-threads-unreadable（交回 0，不交回 78）
#
# gate-similar: layer0-shard-run.sh 它按 SINGLEFS_LAYER0_SHARD 切片跑登记的崩溃枚举用例、写 checker-tier-layer0-replay 全绿标记，账本与 merge 是层 0 那一套；这里跑的是判定存储那一套（各写各的库、导入并成一个）
# gate-similar: multi-host-run.sh 它只管把一条命令送到第二台跑并取回产物，不知道分片；这里调它送第二台那一份，本机那一份与导入那一趟自己跑
# gate-similar: mutation-shard-run.sh 它按用时把变异表分两堆、两台各跑门禁 checker-tier-crates-mutation-replay；分的是变异行，不是崩溃状态块
# gate-similar: crash-amplification-item.sh 它排派活计划、定一个分项在本机一台跑还是动用第二台；动用第二台时调这里
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

two_hosts_script_directory="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
two_hosts_script="$two_hosts_script_directory/$(basename "${BASH_SOURCE[0]}")"
repository_root="$(cd "$two_hosts_script_directory/../.." && pwd)"
source "$two_hosts_script_directory/crash-amplification-lib.sh"

two_hosts_break_is() { [[ ",${CRASH_AMPLIFICATION_TWO_HOSTS_BREAK:-}," == *",$1,"* ]]; }
two_hosts_exit_code_is_explained() { # <要退的码>：78、248 与 245（内存包装认出的显存不够）由 crash_amplification_explain_exit_code 打出路
  [[ "$1" == 78 || "$1" == 248 ]] || { [[ "$1" == 245 ]] && ! two_hosts_break_is gpu-memory-unexplained; }
}

# 只供测试（文件头「只供测试」）：换掉本机那一趟、第二台那一趟与清场。本机那一趟的环境照真的那一趟组（crash_amplification_environment_for_this_host），只把起 cargo 换成假的
if [[ -n "${CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN:-}" ]]; then
  crash_amplification_run_on_this_host() {
    crash_amplification_environment_for_this_host "$1" "$2" "${@:4}"
    env "${this_host_environment[@]}" bash "$CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN" "$@"
  }
fi
multi_host_runner="${CRASH_AMPLIFICATION_TWO_HOSTS_PEER_RUN:-$two_hosts_script_directory/multi-host-run.sh}"
peer_host_library="${CRASH_AMPLIFICATION_TWO_HOSTS_PEER_HOST_LIB:-$two_hosts_script_directory/peer-host-lib.sh}"

usage() {
  echo "  ✗ 用法：$0 <测试目标> [--test-filter <用例函数名>] [--expand-up-to <N>] [--threads <本机跑的那一步的核对线程数；第二台的取 CRASH_AMPLIFICATION_PEER_THREADS 或现算>] [--library <本机库目录，跨次复用>] [--memory-cap <本机上限>] [--peer-memory-cap <第二台上限>] [--label <标签>] [--host-weights <本机>,<第二台>] [--local-states-per-second <数>] [--quiesce]，或 --selftest" >&2
  echo "  → 怎么办：测试目标写 crates/singlefs-checker-tier/tests/ 下的文件名（不带 .rs），例 crash_amplification_fixed_script_through_rollback" >&2
  if two_hosts_break_is usage-exits-1; then exit 1; fi
  exit 2
}



# 自证：三份假的（本机那一趟、第二台那一趟的 multi-host-run.sh、清场用的 peer-host-lib.sh）加一份假的用例，每一格起一趟里层 bash <本脚本>，
# 看退出码、转打的行与调用记录。假的用例：一共 6 块，SHARE=i/n 判序号除以 n 余 i 的那几块；库是一个目录（CURRENT 加一份 blocks，一行一块的序号），
# 已在库里的块记复用、不重判；带 IMPORT 时把那个库的块并进来再对账：没人判的块数不是 0 就 panicked、退 101（真用例在导入之后断言对账为空）。
# 每一格各用自己的格目录（库、第二台的假树、调用记录、输出）与自己的标签（日志目录），不读别的格留下的东西。
run_selftest() {
  local work failures=0 cells=0 cell_directory="" cell_exit=0
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-amplification-two-hosts-selftest.XXXXXX")" || {
    echo "  ✗ --selftest：建不了临时目录（TMPDIR=${TMPDIR:-/tmp}）" >&2
    echo "  → 怎么办：看 TMPDIR 可不可写" >&2
    exit 2
  }
  trap 'rm -rf -- "${work:?}"' EXIT
  cat >"$work/case.sh" <<'STUB'
#!/usr/bin/env bash
# 假的用例（自证用）：6 块；SHARE=i/n 判序号除以 n 余 i 的块，库里已有的记复用；带 IMPORT 并进来再对账；带停下文件的那一趟判一块、等停下文件；每次调记一笔进 STUB_CALLS
# （classes=[…] 是这一趟环境里 GPU_COMPUTATION_CLASSES 的值，cards=[…] 是 GPU_CARDS_BY_PRIORITY 的值，没设记 [unset]）
set -uo pipefail
share="${SINGLEFS_CRASH_AMPLIFICATION_SHARE:?}" library="${SINGLEFS_CRASH_AMPLIFICATION_LIBRARY:?}"
import="${SINGLEFS_CRASH_AMPLIFICATION_IMPORT:-}" stop_file="${SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE:-}"
placement="${SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY:-0}" weights="${SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS:-none}" total=6
import_mark=no; [[ -n "$import" ]] && import_mark=yes
stop_mark=no; [[ -n "$stop_file" ]] && stop_mark=yes
echo "${STUB_HOST:?} share=$share import=$import_mark placement=$placement stop=$stop_mark sent=${STUB_SENT:-no} classes=[${GPU_COMPUTATION_CLASSES-unset}] cards=[${GPU_CARDS_BY_PRIORITY-unset}] weights=$weights" >>"${STUB_CALLS:?}"
# 这一趟判的线程数（SINGLEFS_CRASH_AMPLIFICATION_THREADS）另记进 <调用记录>.checking-threads（现算线程那几格读）
echo "$STUB_HOST placement=$placement checking_threads=${SINGLEFS_CRASH_AMPLIFICATION_THREADS-unset}" >>"$STUB_CALLS.checking-threads"
# 往引擎送的两个变量（SINGLEFS_CRASH_POINT_REPLAY_*）在这一趟环境里的值另记进 <调用记录>.engine-environment；registry_sum 是那份文件的 cksum（没设写 unset，指到的文件不在写 missing）
registry="${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" registry_sum=unset
if [[ "$registry" != unset ]]; then registry_sum=missing; [[ -f "$registry" ]] && registry_sum="$(cksum <"$registry" | cut -d' ' -f1)"; fi
echo "$STUB_HOST placement=$placement skip=[${SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER-unset}] registry=[$registry] registry_sum=[$registry_sum]" >>"$STUB_CALLS.engine-environment"
fail_if_told() { # STUB_FAILURE=plain|device：照点名失败，退 101；gpu-memory：演经内存包装跑的那一趟测试红了、包装认出显存不够，退 245
  case "${STUB_FAILURE:-}" in
    gpu-memory)
      echo "test result: FAILED"
      echo "run-with-memory-cap: 显存不够（假的用例）"
      exit 245
      ;;
    device) echo "GPU_CARD_NOT_USABLE index=0：配置点名了这张卡，这台机器却用不了它（假的用例）" ;;
    given-states) echo "CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET table=stub-table reason=配了 GPU，给定状态表的判器借不到显存（假的用例）" ;;
    plain) ;;
    *) return 0 ;;
  esac
  echo "thread 'the_case' panicked at crates/singlefs-checker-tier/src/crash_amplification.rs:1:1:"
  echo "假的用例照 STUB_FAILURE=$STUB_FAILURE 失败"
  echo "test result: FAILED"
  exit 101
}
if [[ "$placement" == 1 ]]; then
  case "${STUB_PLACEMENT:-ok}" in
    device) echo "GPU_CARD_NOT_USABLE index=2：配置点名了这张卡，这台机器却用不了它（假的用例）"; echo "test result: FAILED"; exit 101 ;;
    algorithm)
      echo "thread 'the_case' panicked at crates/singlefs-checker-tier/src/crash_amplification.rs:1:1:"
      echo "卡 2 判不了点 p 块内第 5 个状态（恢复内核第 1 字 7、checker 内核第 3 字 9：表越过了内核的容量）。配置开了 GPU，不退回 CPU 判器"
      echo "test result: FAILED"; exit 101 ;;
    unrecognized) echo "test result: FAILED"; exit 101 ;;
    not-run) # 第二台起过卡池、等满了也借不到显存：照 Rust 那一行打，不打计划行，退 0
      echo "running 1 test"
      echo "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=等了 1800 秒，池里没有一张卡空得出 2560 MiB（假的用例）"
      echo "test result: ok. 1 passed; 0 failed"; exit 0 ;;
    foreign-not-run) # 别的用例名的那一行（不是产品路径的），退 0、不打计划行
      echo "NOT_RUN_NO_GPU_ADAPTER test=some_gpu_parity_test reason=本机没有 NVIDIA 卡（假的用例）"
      echo "test result: ok. 1 passed; 0 failed"; exit 0 ;;
  esac
  rate="${STUB_RATE:-50}"
  echo "CRASH_AMPLIFICATION_PLACEMENT host=$STUB_HOST judge=gpu pending_blocks=$total pending_states=6000 states_per_second_on_this_host=$rate estimated_seconds_on_this_host=$(( 6000 / rate ))"
  exit 0
fi
{ mkdir -p -- "$library" && touch -- "$library/CURRENT" "$library/blocks"; } 2>/dev/null || { echo "假的用例建不了库 $library"; exit 3; }
if [[ -n "$stop_file" ]]; then
  fail_if_told
  grep -qx 0 "$library/blocks" || echo 0 >>"$library/blocks"
  # 等驱动叫停：停下文件在就是叫停了；驱动走过叫停那一步（STOP_DECIDED 那份文件在）而停下文件不在，就是没叫停、当场判红；两样都等不到，等满 10 秒判红
  waited=0 decided="${CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED:-}"
  while [[ ! -e "$stop_file" ]] && [[ -z "$decided" || ! -e "$decided" ]] && (( waited < 100 )); do sleep 0.1; waited=$((waited + 1)); done
  if [[ ! -e "$stop_file" && -n "$decided" && -e "$decided" ]]; then
    echo "thread 'the_case' panicked: 驱动走过了叫停那一步（$decided 在），停下文件 $stop_file 却没建"
    echo "test result: FAILED"
    exit 101
  fi
  if [[ ! -e "$stop_file" ]]; then
    echo "thread 'the_case' panicked: 等了 10 秒，停下文件 $stop_file 也没建出来"
    echo "test result: FAILED"
    exit 101
  fi
  echo "CRASH_AMPLIFICATION share=$share states_checked_here=1 stopped_early=true weights=$weights"
  exit 0
fi
index="${share%/*}" count="${share#*/}" judged=0 reused=0
for (( block = 0; block < total; block++ )); do
  (( block % count == index )) || continue
  [[ "$block" == "${STUB_DROP_BLOCK:-}" ]] && continue
  if grep -qx "$block" "$library/blocks"; then reused=$((reused + 1)); else echo "$block" >>"$library/blocks"; judged=$((judged + 1)); fi
done
fail_if_told
if [[ -z "$import" ]]; then
  echo "CRASH_AMPLIFICATION share=$share states_checked_here=$judged reused=$reused weights=$weights"
  exit 0
fi
if [[ ! -f "$import/CURRENT" ]]; then
  echo "thread 'the_case' panicked: 要导入的库 $import 不在"
  echo "test result: FAILED"
  exit 101
fi
imported=$(( $(sort -u "$library/blocks" "$import/blocks" | wc -l) - $(sort -u "$library/blocks" | wc -l) ))
sort -u -o "$library/blocks" "$library/blocks" "$import/blocks"
unjudged=$(( total - $(wc -l <"$library/blocks") ))
echo "CRASH_AMPLIFICATION share=$share states_checked_here=$judged reused=$reused imported=$imported unjudged_blocks=$unjudged red=0 weights=$weights"
if (( unjudged > 0 )); then
  echo "thread 'the_case' panicked: 导入之后对账不为空：$unjudged 块没人判"
  echo "test result: FAILED"
  exit 101
fi
exit 0
STUB
  cat >"$work/local-run.sh" <<'STUB'
#!/usr/bin/env bash
# 假的本机那一趟（自证用）：调法同 crash_amplification_run_on_this_host（<share> <库目录> <日志> [<附加环境 键=值>…]），跑假的用例、写日志。
# 照点名失败：带停下文件的那一趟看 STUB_WARM_FAILURE，带 IMPORT 的导入那一趟不失败，别的（分份那一趟）看 STUB_LOCAL_FAILURE
share="$1" library="$2" log="$3"
shift 3
case " $* " in
  *" SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE="*) failure="${STUB_WARM_FAILURE:-}" ;;
  *" SINGLEFS_CRASH_AMPLIFICATION_IMPORT="*) failure="" ;;
  *) failure="${STUB_LOCAL_FAILURE:-}" ;;
esac
env "$@" STUB_HOST=local STUB_SENT=no STUB_FAILURE="$failure" STUB_DROP_BLOCK= STUB_PLACEMENT=ok \
  SINGLEFS_CRASH_AMPLIFICATION_SHARE="$share" SINGLEFS_CRASH_AMPLIFICATION_LIBRARY="$library" bash "${STUB_CASE:?}" >"$log" 2>&1
STUB
  cat >"$work/multi-host-run.sh" <<'STUB'
#!/usr/bin/env bash
# 假的 multi-host-run.sh（自证用）：参数与它相同；不拷树、不连第二台，在 STUB_PEER_TREE 这棵假树根下跑假的用例，--send 的目录先拷进树，
# --fetch 点名的目录跑完拷进 MULTI_HOST_RUN_FETCH_DIRECTORY（跑红了也取回，与真的一样）。
# STUB_PEER_EXIT=255（分份那一趟）/ STUB_PEER_PLACEMENT=unreachable（排计划那一趟）：连不上，什么都不跑、不取回，退 255；
# STUB_PEER_FAILURE=plain|device：分份那一趟判完照点名失败；STUB_PEER_DROP_BLOCK=<序号>：漏判那一块；STUB_PEER_NO_FETCH=1：跑完不取回；
# STUB_PEER_PLACEMENT=device|algorithm|unrecognized：排计划那一趟照点名失败；STUB_PEER_RATE：排计划那一趟报的速度（默认 50）。
# 真的第二台那一趟是新起的 ssh 会话、不继承本机的环境：假的照此清掉 GPU_COMPUTATION_CLASSES 与往引擎送的两个变量，第二台只看得见命令里送过去的那一份。
# 命令最前面可以是 bash -c <脚本> <名> <变量> <树内相对路径>（双机驱动的 peer_tree_path_script）：假的在假树根里真跑那段脚本（拼 $PWD、文件不在退 2）取变量值，
# 脚本退非 0 就当那一趟命令退那个码、什么都不跑
fetch_name="" send_sources=() send_names=()
while (($# > 0)); do
  case "$1" in
    --peer) shift ;;
    --label|--memory-cap) shift 2 ;;
    --fetch) fetch_name="$2"; shift 2 ;;
    --send) send_sources+=("$2"); send_names+=("$3"); shift 3 ;;
    --) shift; break ;;
    *) echo "假的 multi-host-run.sh：认不得参数 $1"; exit 2 ;;
  esac
done
tree_path_variable="" tree_path_value=""
if [[ "${1:-}" == bash && "${2:-}" == -c ]]; then
  tree_path_variable="$5"
  tree_path_script="$3" tree_path_name="$4" tree_path_relative="$6"
  shift 6
fi
[[ "${1:-}" == env ]] && shift
assignments=()
while (($# > 0)) && [[ "$1" == *=* ]]; do assignments+=("$1"); shift; done
placement=0
case " ${assignments[*]} " in *" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1 "*) placement=1 ;; esac
# 送过去的命令是 bash research/scripts/capped.sh <线程数> cargo …：线程数记进 <调用记录>.threads（现算线程那几格读）
[[ "${1:-}" == bash && "${2:-}" == */capped.sh ]] && echo "placement=$placement threads=${3:-}" >>"${STUB_CALLS:?}.threads"
if { ((placement)) && [[ "${STUB_PEER_PLACEMENT:-ok}" == unreachable ]]; } || { ((!placement)) && [[ "${STUB_PEER_EXIT:-}" == 255 ]]; }; then
  echo "peer unreachable placement=$placement" >>"${STUB_CALLS:?}"
  echo "multi-host-run.sh（假的）：第二台连不上"
  exit 255
fi
tree="${STUB_PEER_TREE:?}"
mkdir -p -- "$tree" || exit 3
sent=no
for index in "${!send_sources[@]}"; do
  rm -rf -- "${tree:?}/${send_names[index]:?}"
  cp -a -- "${send_sources[index]}" "$tree/${send_names[index]}" || exit 3
  [[ "${send_names[index]}" == crash-amplification-library ]] && sent=yes
done
if [[ -n "$tree_path_variable" ]]; then
  tree_path_value="$(cd "$tree" && bash -c "$tree_path_script" "$tree_path_name" "$tree_path_variable" "$tree_path_relative" printenv "$tree_path_variable")" || exit 2
  assignments+=("$tree_path_variable=$tree_path_value")
fi
( cd "$tree" && env -u GPU_COMPUTATION_CLASSES -u SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER -u SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY "${assignments[@]}" STUB_HOST=peer STUB_SENT="$sent" STUB_FAILURE="${STUB_PEER_FAILURE:-}" STUB_DROP_BLOCK="${STUB_PEER_DROP_BLOCK:-}" \
    STUB_PLACEMENT="${STUB_PEER_PLACEMENT:-ok}" STUB_RATE="${STUB_PEER_RATE:-50}" bash "${STUB_CASE:?}" )
case_exit=$?
if [[ -n "$fetch_name" && "${STUB_PEER_NO_FETCH:-0}" != 1 && -e "$tree/$fetch_name" ]]; then
  mkdir -p -- "${MULTI_HOST_RUN_FETCH_DIRECTORY:?}"
  rm -rf -- "${MULTI_HOST_RUN_FETCH_DIRECTORY:?}/${fetch_name:?}"
  cp -a -- "$tree/$fetch_name" "$MULTI_HOST_RUN_FETCH_DIRECTORY/$fetch_name" || exit 3
fi
exit "$case_exit"
STUB
  cat >"$work/peer-host-lib.sh" <<'STUB'
# 假的 peer-host-lib.sh（自证用，被 source）：STUB_QUIESCE=configuration-fails|quiesce-fails 照点名失败；每次调记一笔进 STUB_CALLS
# 配置里的复原命令只记一笔 restore（这里不该调它，弄坏开关 restore-after-run 下才调）
peer_host_load_configuration() { echo "load-configuration" >>"${STUB_CALLS:?}"; QUIESCE_START_COMMAND='echo restore >>"$STUB_CALLS"'; [[ "${STUB_QUIESCE:-ok}" != configuration-fails ]]; }
peer_host_quiesce() { echo "quiesce" >>"${STUB_CALLS:?}"; [[ "${STUB_QUIESCE:-ok}" != quiesce-fails ]]; }
peer_host_quiesce_left() { echo "quiesce-left" >>"${STUB_CALLS:?}"; }
STUB
  cat >"$work/host-resources.py" <<'STUB'
# 假的 research/scripts/host-resources.py（自证用，python3 <它> --json 起）：打造出来的两台；STUB_LOCAL_THREADS、STUB_PEER_THREADS 是两台此刻的可用线程（默认 11、5），
# STUB_HOST_RESOURCES=fails 退 1、什么都不打；=peer-unread 第二台记没读到（read 是 false、不带 cpu）
import json, os, sys
mode = os.environ.get("STUB_HOST_RESOURCES") or "ok"
if mode == "fails":
    print("假的 host-resources.py：照 STUB_HOST_RESOURCES=fails 读不了", file=sys.stderr)
    sys.exit(1)
local = {"number": 1, "name": "本机", "is_local": True, "is_peer": False, "read": True, "cpu": {"available_threads": int(os.environ.get("STUB_LOCAL_THREADS") or 11)}}
peer = {"number": 2, "name": "第二台", "is_local": False, "is_peer": True, "read": mode != "peer-unread"}
if mode != "peer-unread":
    peer["cpu"] = {"available_threads": int(os.environ.get("STUB_PEER_THREADS") or 5)}
print(json.dumps({"hosts": [local, peer]}))
STUB
  printf '%s\n' 'GPU_CARDS_BY_PRIORITY="local:0:15G"' >"$work/multi-host-without-quiesced.env"
  # 退役的私有显卡配置（SINGLEFS_GPU_CARDS_CONFIG）指到一份写了别的卡与一类计算的假文件：读到它就是读错了（
  #   gpu-cards-from-multi-host-configuration、quiesced-without-card-list；弄坏开关下照它读，也不会去读这台机器真的私有配置）
  printf '%s\n' 'GPU_CARDS_BY_PRIORITY="local:1:3G peer:1:3G"' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="local:1:3G peer:1:3G"' "GPU_COMPUTATION_CLASSES='crash_judge_gpu'" >"$work/retired-gpu-cards.env"
  # 里层读的多机配置指到不在的文件（两个开关都关、一份卡都没写），库与日志都在临时目录下，本机、第二台与清场都换成上面那几份假的
  export SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host.env" SINGLEFS_GPU_CARDS_CONFIG="$work/retired-gpu-cards.env" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home"
  export CRASH_AMPLIFICATION_TWO_HOSTS_LOCAL_RUN="$work/local-run.sh" CRASH_AMPLIFICATION_TWO_HOSTS_PEER_RUN="$work/multi-host-run.sh" \
    CRASH_AMPLIFICATION_TWO_HOSTS_PEER_HOST_LIB="$work/peer-host-lib.sh" STUB_CASE="$work/case.sh" CRASH_AMPLIFICATION_HOST_RESOURCES="$work/host-resources.py"

  run_cell() { # <格名> [<环境 键=值>…] -- <参数…>：起一趟里层（标签是格名、库在格目录下），输出落 <格目录>/output、调用记录落 <格目录>/calls，退出码放 cell_exit
    local name="$1" environment=()
    shift
    while (($# > 0)) && [[ "$1" != -- ]]; do environment+=("$1"); shift; done
    shift
    cell_directory="$work/$name"
    mkdir -p "$cell_directory"
    : >"$cell_directory/calls"
    cells=$((cells + 1))
    env -u SINGLEFS_CRASH_AMPLIFICATION_IMPORT -u SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE -u SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY -u SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS -u GPU_COMPUTATION_CLASSES -u GPU_CARDS_BY_PRIORITY \
      -u SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER -u SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY \
      -u CRASH_AMPLIFICATION_PEER_THREADS -u CRASH_AMPLIFICATION_BUILD_THREADS STUB_HOST_RESOURCES= STUB_LOCAL_THREADS= STUB_PEER_THREADS= \
      STUB_LOCAL_FAILURE= STUB_WARM_FAILURE= STUB_PEER_EXIT= STUB_PEER_FAILURE= STUB_PEER_DROP_BLOCK= STUB_PEER_NO_FETCH= STUB_PEER_PLACEMENT= STUB_PEER_RATE= STUB_QUIESCE= \
      STUB_CALLS="$cell_directory/calls" STUB_PEER_TREE="$cell_directory/peer-tree" CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED="$cell_directory/stop-decided" \
      "${environment[@]+"${environment[@]}"}" \
      bash "$two_hosts_script" "$@" --library "$cell_directory/library" --label "$name" >"$cell_directory/output" 2>&1
    cell_exit=$?
  }
  cell_failed() { # <格名> <判错的一句> <出路>
    echo "  ✗ 自证 $1：$2（退 $cell_exit；调用记录「$(tr '\n' ';' <"$work/$1/calls")」）" >&2
    echo "     → 怎么办：$3" >&2
    tail -n 12 "$work/$1/output" | sed 's/^/       | /' >&2
    failures=$((failures + 1))
  }
  output_has() { grep -qE -- "$2" "$work/$1/output"; }   # <格名> <正则>
  calls_have() { grep -qE -- "$2" "$work/$1/calls"; }    # <格名> <正则>
  last_call_is() { [[ "$(tail -n 1 "$work/$1/calls")" =~ $2 ]]; }   # <格名> <正则>
  # <格名>：转打的 CRASH_AMPLIFICATION 行里恰好一行带 import_run=true，就是导入那一趟的那一行（假的用例导入时报 imported=）
  only_the_import_line_is_marked() {
    [[ "$(grep -c 'import_run=true' "$work/$1/output")" == 1 ]] && output_has "$1" '^CRASH_AMPLIFICATION share=0/2 .*imported=[0-9]+ .*import_run=true$'
  }

  # ── 分份、取回、导入、对账
  run_cell both-hosts-green -- some-target
  if [[ "$cell_exit" != 0 ]] || ! calls_have both-hosts-green '^local share=0/2 import=no ' || ! calls_have both-hosts-green '^peer share=1/2 import=no ' \
     || ! last_call_is both-hosts-green '^local share=0/2 import=yes ' || ! output_has both-hosts-green 'unjudged_blocks=0 red=0' \
     || ! only_the_import_line_is_marked both-hosts-green; then
    cell_failed both-hosts-green "本机领 0/2、第二台领 1/2、取回之后本机带 IMPORT 导入、对账为空，应当退 0；转打的行里只有导入那一行带 import_run=true" \
      "看主路径：第二台那一趟的 SINGLEFS_CRASH_AMPLIFICATION_SHARE 是 1/2，取回之后导入那一趟带 SINGLEFS_CRASH_AMPLIFICATION_IMPORT=<取回的库>，转打经 print_the_import_lines"
  fi
  mkdir -p "$work/library-snapshot-sent/library"
  : >"$work/library-snapshot-sent/library/CURRENT"; echo 1 >"$work/library-snapshot-sent/library/blocks"
  run_cell library-snapshot-sent -- some-target
  if [[ "$cell_exit" != 0 ]] || ! calls_have library-snapshot-sent '^peer share=1/2 import=no placement=0 stop=no sent=yes ' \
     || ! output_has library-snapshot-sent '^CRASH_AMPLIFICATION share=1/2 states_checked_here=2 reused=1 '; then
    cell_failed library-snapshot-sent "本机库里已有一块，应当照快照送给第二台（--send）、第二台复用那一块、退 0" \
      "看主路径 \$local_library/CURRENT 在时 cp -a 照快照、peer_arguments 加 --send 那一段"
  fi
  mkdir -p "$work/library-snapshot-fails/library" "$work/home/runs/library-snapshot-fails"
  : >"$work/library-snapshot-fails/library/CURRENT"; : >"$work/library-snapshot-fails/library/blocks"
  : >"$work/home/runs/library-snapshot-fails/library-for-peer"   # 占住快照的落点：一份同名的普通文件，cp -a 拷不过去
  run_cell library-snapshot-fails -- some-target
  [[ "$cell_exit" == 2 ]] \
    || cell_failed library-snapshot-fails "拷不了给第二台的库快照，应当退 2" "看主路径 cp -a 那一行：拷不成就退 2，不带着半份快照去跑"
  run_cell local-share-fails STUB_LOCAL_FAILURE=plain -- some-target
  if [[ "$cell_exit" != 101 ]] || ! last_call_is local-share-fails '^local share=0/2 import=yes ' || ! only_the_import_line_is_marked local-share-fails; then
    cell_failed local-share-fails "本机那一份退 101、认不出原因，应当先把第二台取回的库导进本机库（那一行带 import_run=true）、再原样退 101" \
      "看「本机那一份退」那一段：取回的库在就先导入、经 print_the_import_lines 转打，退出码经 crash_amplification_exit_code_of_a_failed_run 照 share-0.log 分类"
  fi
  run_cell local-share-card-unusable STUB_LOCAL_FAILURE=device -- some-target
  [[ "$cell_exit" == 78 ]] \
    || cell_failed local-share-card-unusable "本机那一份日志里有卡用不了的原话，应当退 78" "看「本机那一份退」那一段与 crash-amplification-lib.sh 的 crash_amplification_failure_kind 认不认 GPU_CARD_NOT_USABLE"
  # 给定状态表的判器借不到显存：引擎打 CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 那一行后报错（cargo 退 101），是运行条件不满足
  run_cell local-share-given-states-run-condition STUB_LOCAL_FAILURE=given-states -- some-target
  [[ "$cell_exit" == 78 ]] \
    || cell_failed local-share-given-states-run-condition "本机那一份日志里有给定状态表借不到显存那一行（CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET），应当退 78" \
      "看 crash-amplification-lib.sh 的 crash_amplification_failure_kind 的 device 那一组认不认 CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX 起头的那一行"
  run_cell local-share-gpu-memory STUB_LOCAL_FAILURE=gpu-memory -- some-target
  { [[ "$cell_exit" == 245 ]] && output_has local-share-gpu-memory '显存不够（那一趟退了非 0'; } \
    || cell_failed local-share-gpu-memory "本机那一份经内存包装退 245（显存不够），应当原样退 245、打显存不够的出路" \
      "看「本机那一份退」那一段与 two_hosts_exit_code_is_explained：245 走 crash-amplification-lib.sh 的 crash_amplification_explain_exit_code 那一支"
  run_cell peer-share-gpu-memory STUB_PEER_FAILURE=gpu-memory -- some-target
  { [[ "$cell_exit" == 245 ]] && output_has peer-share-gpu-memory '显存不够（那一趟退了非 0'; } \
    || cell_failed peer-share-gpu-memory "第二台那一份经那边的内存包装退 245（显存不够），应当原样退 245、打显存不够的出路" \
      "看「第二台那一份退」那一段与 two_hosts_exit_code_is_explained：245 不是 255，照 share-1.log 分类之后原样退、走显存不够那一支"
  run_cell peer-unreachable STUB_PEER_EXIT=255 -- some-target
  [[ "$cell_exit" == 78 ]] \
    || cell_failed peer-unreachable "multi-host-run.sh 退 255（第二台连不上），应当退 78" "看「第二台那一份退」那一段：255 是第二台用不上，映射成 78"
  run_cell peer-share-card-unusable STUB_PEER_FAILURE=device -- some-target
  if [[ "$cell_exit" != 78 ]] || ! last_call_is peer-share-card-unusable '^local share=0/2 import=yes ' || ! only_the_import_line_is_marked peer-share-card-unusable; then
    cell_failed peer-share-card-unusable "第二台那一份判完卡用不了（退 101，库取回了），应当先导入它判过的块（那一行带 import_run=true）、再退 78" \
      "看「第二台那一份退」那一段：取回的库在就先导入、经 print_the_import_lines 转打，退出码照 share-1.log 分类"
  fi
  run_cell peer-library-not-fetched STUB_PEER_NO_FETCH=1 -- some-target
  [[ "$cell_exit" == 78 ]] \
    || cell_failed peer-library-not-fetched "第二台退 0 而库没取回，应当退 78" "看「第二台的库没取回」那一段：\$fetched_library 不在就退 78，不拿空的去导入"
  run_cell reconciliation-not-empty STUB_PEER_DROP_BLOCK=3 -- some-target
  if [[ "$cell_exit" != 101 ]] || ! output_has reconciliation-not-empty 'unjudged_blocks=1 '; then
    cell_failed reconciliation-not-empty "第二台漏判一块、导入之后对账剩 1 块，应当原样退导入那一趟的 101" \
      "看末尾导入那一趟：它退非 0 就照 import.log 分类退出，不报绿"
  fi

  # ── 先判着、量第二台的速度、按速度分
  run_cell calibrated-weights -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! output_has calibrated-weights '^CRASH_AMPLIFICATION share=0/1 .*stopped_early=true.* warm_up=true$' \
     || ! output_has calibrated-weights '^CRASH_AMPLIFICATION_HOST_WEIGHTS local,peer=150,50$' \
     || ! calls_have calibrated-weights '^peer share=0/1 import=no placement=1 ' \
     || ! calls_have calibrated-weights '^local share=0/2 import=no .* weights=150,50$' || ! calls_have calibrated-weights '^peer share=1/2 import=no .* weights=150,50$'; then
    cell_failed calibrated-weights "本机先判着（转打带 warm_up=true）、第二台排计划量出每秒 50、两趟分份都带 SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=150,50，应当退 0" \
      "看「给了本机的速度」那一段：量完建停下文件、转打加 warm_up=true、host_weights 写进 weights_environment"
  fi
  run_cell warm-run-card-unusable STUB_WARM_FAILURE=device -- some-target --local-states-per-second 150
  [[ "$cell_exit" == 78 ]] \
    || cell_failed warm-run-card-unusable "本机先判着的那一趟卡用不了，应当退 78" "看「本机先判着的那一趟退」那一段：经 crash_amplification_exit_code_of_a_failed_run 照 warm-local.log 分类"
  run_cell peer-placement-unreachable STUB_PEER_PLACEMENT=unreachable -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 78 ]] || ! output_has peer-placement-unreachable '^CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED exit=255$'; then
    cell_failed peer-placement-unreachable "第二台排计划那一趟 multi-host-run.sh 退 255，应当打 CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED 并退 78" \
      "看「第二台排派活计划那一趟没量出速度」那一段：255 映射成 78"
  fi
  run_cell peer-placement-card-unusable STUB_PEER_PLACEMENT=device -- some-target --local-states-per-second 150
  [[ "$cell_exit" == 78 ]] \
    || cell_failed peer-placement-card-unusable "第二台排计划那一趟卡用不了，应当退 78" "看「第二台排派活计划那一趟没量出速度」那一段的 case：device 退 78"
  run_cell peer-placement-algorithm STUB_PEER_PLACEMENT=algorithm -- some-target --local-states-per-second 150
  [[ "$cell_exit" == 248 ]] \
    || cell_failed peer-placement-algorithm "第二台排计划那一趟算法上判不了，应当退 248" "看「第二台排派活计划那一趟没量出速度」那一段的 case：algorithm 退 248"
  run_cell peer-placement-unrecognized STUB_PEER_PLACEMENT=unrecognized -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 249 ]] || calls_have peer-placement-unrecognized '^peer share=1/2 '; then
    cell_failed peer-placement-unrecognized "第二台没量出速度、日志里认不出原因，应当退 249、不起分份那两趟" \
      "看「第二台排派活计划那一趟没量出速度」那一段：认不出的退 249，不往下分份"
  fi
  run_cell peer-placement-not-run STUB_PEER_PLACEMENT=not-run -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 77 ]] || calls_have peer-placement-not-run '^peer share=1/2 ' \
     || ! output_has peer-placement-not-run '^CRASH_AMPLIFICATION_NOT_RUN step=peer-placement test=crash_amplification::run_from_environment reason=等了 1800 秒，池里没有一张卡空得出 2560 MiB（假的用例）$' \
     || ! output_has peer-placement-not-run '本次未跑（这不是通过）：第二台排派活计划那一趟借不到显存' || output_has peer-placement-not-run '^CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED '; then
    cell_failed peer-placement-not-run "第二台排计划那一趟借不到显存（退 0、没打计划行、有产品路径那一行），应当打 CRASH_AMPLIFICATION_NOT_RUN step=peer-placement、报本次未跑、退 77、不起分份那两趟" \
      "看「第二台排派活计划那一趟没量出速度」之前认 peer_not_run_line 那一段：那一趟退 0、没有计划行、行首是产品路径那一行才认"
  fi
  run_cell peer-placement-foreign-not-run-line STUB_PEER_PLACEMENT=foreign-not-run -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 249 ]] || output_has peer-placement-foreign-not-run-line '^CRASH_AMPLIFICATION_NOT_RUN '; then
    cell_failed peer-placement-foreign-not-run-line "第二台排计划那一趟只有别的用例名的 NOT_RUN_NO_GPU_ADAPTER 行，应当照没量出速度、认不出原因退 249，不打 CRASH_AMPLIFICATION_NOT_RUN" \
      "看认 peer_not_run_line 那一段：只认行首是「NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=」的那一行"
  fi

  # ── 参数、配置与清场
  run_cell host-weights-malformed -- some-target --host-weights 3
  [[ "$cell_exit" == 2 ]] || cell_failed host-weights-malformed "--host-weights 3（缺第二台那一个数）应当退 2" "看参数之后 --host-weights 的格式判"
  run_cell local-speed-malformed -- some-target --local-states-per-second 0
  [[ "$cell_exit" == 2 ]] || cell_failed local-speed-malformed "--local-states-per-second 0 应当退 2" "看参数之后 --local-states-per-second 的格式判"
  run_cell usage-missing-target --
  [[ "$cell_exit" == 2 ]] || cell_failed usage-missing-target "没给测试目标应当退 2" "看 usage：用法错退 2"
  run_cell usage-unknown-option -- some-target --no-such-option
  [[ "$cell_exit" == 2 ]] || cell_failed usage-unknown-option "认不得的选项应当退 2" "看参数那一段的 -*) 分支与 usage"
  run_cell quiesced-without-card-list SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-without-quiesced.env" -- some-target --quiesced
  [[ "$cell_exit" == 78 ]] \
    || cell_failed quiesced-without-card-list "说是清过场、多机配置里却没写 GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED（退役的私有显卡配置里写了也不算），应当退 78" "看 crash_amplification_prepare 那一行：交回非 0 就退 78"
  run_cell quiesce-configuration-fails STUB_QUIESCE=configuration-fails -- some-target --quiesce
  [[ "$cell_exit" == 78 ]] \
    || cell_failed quiesce-configuration-fails "--quiesce 时多机配置判不过，应当退 78" "看 --quiesce 那一段 peer_host_load_configuration 之后的退出"
  run_cell quiesce-fails STUB_QUIESCE=quiesce-fails -- some-target --quiesce
  [[ "$cell_exit" == 78 ]] || cell_failed quiesce-fails "--quiesce 时清场没过，应当退 78" "看 --quiesce 那一段 peer_host_quiesce 之后的退出"
  run_cell quiesce-not-restored -- some-target --quiesce
  if [[ "$cell_exit" != 0 ]] || ! last_call_is quiesce-not-restored '^quiesce-left$' || grep -qx restore "$work/quiesce-not-restored/calls"; then
    cell_failed quiesce-not-restored "--quiesce 清过场、跑完退 0，退出时应当只报「清了场、没复原」、不跑复原命令（调用记录最后一笔是 quiesce-left、没有 restore）" "看 --quiesce 那一段 trap 'peer_host_quiesce_left' EXIT"
  fi

  # ── 走 GPU 的计算类与显卡先后（crash-amplification-lib.sh 文件头「配置」）：引擎不读 GPU_COMPUTATION_CLASSES，开不开 GPU 两台的每一趟都不送它
  #   （调用方环境里漏进来的那一份本机那几趟清掉；第二台那一趟是新起的 ssh 会话，只看得见命令里送过去的）；显卡先后读多机配置，两台的每一趟都交到。
  #   退役的私有显卡配置（上面 export 的 SINGLEFS_GPU_CARDS_CONFIG）写了别的卡与一类计算，读到它就红。
  #   各格带 --local-states-per-second，五种调用（本机先判着、第二台排计划、两台分份、本机导入）都走到；所以主路径上的开关（never-stop-warm-run、
  #   same-share-on-both-hosts、skip-import）打开时这几格也红：少了哪一种调用就核不到它的环境，不为脱开它们放掉退 0 与五种调用都在这两条
  printf '%s\n' ENABLE_GPU=1 ENABLE_ACROSS_MACHINES=1 'GPU_CARDS_BY_PRIORITY="local:0:15G peer:0:10G"' >"$work/multi-host-gpu-on.env"
  printf '%s\n' ENABLE_GPU=0 ENABLE_ACROSS_MACHINES=1 >"$work/multi-host-gpu-off.env"
  every_call_has_classes() { ! grep -qvF " classes=[$2] " "$work/$1/calls"; }   # <格名> <值>：每一笔调用记录都是这个 classes
  every_call_has_cards() { ! grep -qvF " cards=[$2] " "$work/$1/calls"; }       # <格名> <值>：每一笔调用记录都是这个 cards
  five_calls_seen() { # <格名>：先判着、第二台排计划、两台分份、本机导入五种调用都在，最后一趟是导入
    calls_have "$1" '^local share=0/1 import=no placement=0 stop=yes ' && calls_have "$1" '^peer share=0/1 import=no placement=1 ' \
      && calls_have "$1" '^local share=0/2 import=no ' && calls_have "$1" '^peer share=1/2 import=no ' && last_call_is "$1" '^local share=0/2 import=yes '
  }
  run_cell gpu-on-classes-not-sent SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" \
    GPU_COMPUTATION_CLASSES=crash_judge_gpu -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! every_call_has_classes gpu-on-classes-not-sent unset || ! five_calls_seen gpu-on-classes-not-sent; then
    cell_failed gpu-on-classes-not-sent "开了 GPU（调用方环境里漏进来一份 GPU_COMPUTATION_CLASSES）：本机三趟与第二台两趟的环境里都不该有 GPU_COMPUTATION_CLASSES（引擎不读它，经 cleared_then_set 设它的用例会 panic），退 0" \
      "本机那几趟看 crash-amplification-lib.sh 的 crash_amplification_environment_for_this_host：开头 -u GPU_COMPUTATION_CLASSES、不再加这个变量；第二台那两趟看 peer_environment：不加这个变量"
  fi
  run_cell gpu-cards-from-multi-host-configuration SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! every_call_has_cards gpu-cards-from-multi-host-configuration 'local:0:15G peer:0:10G' \
     || ! five_calls_seen gpu-cards-from-multi-host-configuration; then
    cell_failed gpu-cards-from-multi-host-configuration "开了 GPU、多机配置写了显卡先后「local:0:15G peer:0:10G」（退役的私有显卡配置里是 local:1:3G peer:1:3G）：两台每一趟的环境里 GPU_CARDS_BY_PRIORITY 都应当是多机配置那一份，退 0" \
      "看 crash-amplification-lib.sh 的 crash_amplification_prepare：显卡先后从 crash_amplification_cards_source_file 给的多机配置读"
  fi
  run_cell gpu-off-classes-not-sent SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" \
    GPU_COMPUTATION_CLASSES=crash_judge_gpu -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! every_call_has_classes gpu-off-classes-not-sent unset \
     || ! calls_have gpu-off-classes-not-sent '^local share=0/2 import=yes ' || ! calls_have gpu-off-classes-not-sent '^peer share=1/2 import=no '; then
    cell_failed gpu-off-classes-not-sent "没开 GPU（调用方环境里漏进来一份 GPU_COMPUTATION_CLASSES）：两台每一趟的环境里都不该有 GPU_COMPUTATION_CLASSES，退 0" \
      "看 crash_amplification_environment_for_this_host 开头的 -u GPU_COMPUTATION_CLASSES"
  fi

  # ── 往引擎送的两个变量（文件头「步骤」之后 peer_environment 那一段）：本机设了，第二台排计划、分份那两趟也要在环境里拿到；
  #   按段登记文件随快照送进第二台那棵树，变量值是第二台上那一份的绝对路径（假的 multi-host-run.sh 在假树根里真跑 peer_tree_path_script 拼出）。
  #   各格带 --local-states-per-second，排计划与分份两趟都走到；调用记录旁的 <调用记录>.engine-environment 逐趟记 skip=[…] registry=[…] registry_sum=[…]
  printf '%s\n' '# 自证用的按段登记文件' 'item-a	0	3	0000000000000000000000000000000000000000000000000000000000000000' >"$work/crash-point-replay-segments.tsv"
  registry_sum_of_local="$(cksum <"$work/crash-point-replay-segments.tsv" | cut -d' ' -f1)"
  peer_engine_lines() { grep '^peer ' "$work/$1/calls.engine-environment" 2>/dev/null; }   # <格名>
  peer_engine_lines_are() { # <格名> <每一趟都要含的子串>：第二台至少有排计划与分份两趟，每一趟都含它
    local lines
    lines="$(peer_engine_lines "$1")"
    [[ "$(grep -c . <<<"$lines")" -ge 2 ]] && ! grep -qvF -- "$2" <<<"$lines"
  }
  run_cell peer-cross-check-skip-sent SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER=1 -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! peer_engine_lines_are peer-cross-check-skip-sent ' skip=[1] ' || ! five_calls_seen peer-cross-check-skip-sent; then
    cell_failed peer-cross-check-skip-sent "本机环境里设了 SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER=1：第二台排计划、分份那两趟的环境里也要有它（实记「$(peer_engine_lines peer-cross-check-skip-sent | tr '\n' ';')」），退 0" \
      "看 peer_environment 后面那一段：本机环境里设了这个变量（值原样）就加进 peer_environment"
  fi
  run_cell peer-segments-registry-sent SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY="$work/crash-point-replay-segments.tsv" -- some-target --local-states-per-second 150
  peer_registry_path="$work/peer-segments-registry-sent/peer-tree/crash-amplification-segments-registry/crash-point-replay-segments.tsv"
  if [[ "$cell_exit" != 0 ]] || ! peer_engine_lines_are peer-segments-registry-sent " registry=[$peer_registry_path] registry_sum=[$registry_sum_of_local]" \
     || ! five_calls_seen peer-segments-registry-sent || ! grep -q "^local .* registry=\[$work/crash-point-replay-segments.tsv\] " "$work/peer-segments-registry-sent/calls.engine-environment"; then
    cell_failed peer-segments-registry-sent "本机环境里设了 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY：第二台排计划、分份那两趟的环境里要有它，值是第二台树里 crash-amplification-segments-registry/ 下那份快照的路径、内容与本机那份相同（实记「$(peer_engine_lines peer-segments-registry-sent | tr '\n' ';')」），本机那几趟照旧继承，退 0" \
      "看 peer_environment 后面那一段：快照拷进 <日志目录>/segments-registry-for-peer/、两趟 multi-host-run.sh 都带 --send、命令最前面加 peer_tree_path_prefix"
  fi
  run_cell peer-engine-variables-absent-when-unset -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! peer_engine_lines_are peer-engine-variables-absent-when-unset ' skip=[unset] registry=[unset] registry_sum=[unset]' \
     || [[ -e "$work/peer-engine-variables-absent-when-unset/peer-tree/crash-amplification-segments-registry" ]]; then
    cell_failed peer-engine-variables-absent-when-unset "本机环境里没设这两个变量：第二台那几趟也不该有、树里也不该多出登记文件快照（实记「$(peer_engine_lines peer-engine-variables-absent-when-unset | tr '\n' ';')」），退 0" \
      "看 peer_environment 后面那一段的条件：本机没设（空串也不送登记文件）就什么都不加"
  fi

  # ── 现算线程（crash-amplification-lib.sh 文件头「现算线程」）：喂假的 host-resources.py 输出
  # 第二台那两趟（排计划、分份）：送过去的 capped.sh 线程数由假的 multi-host-run.sh 记进 <调用记录>.threads
  peer_threads_are() { # <格名> <线程数>：排计划、分份那两趟各一笔、线程数都是它
    [[ "$(LC_ALL=C sort "$work/$1/calls.threads" 2>/dev/null | tr '\n' ';')" == "placement=0 threads=$2;placement=1 threads=$2;" ]]
  }
  run_cell peer-threads-from-host-resources STUB_PEER_THREADS=5 -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! peer_threads_are peer-threads-from-host-resources 5; then
    cell_failed peer-threads-from-host-resources "假的 host-resources.py 报第二台此刻可用 5 个线程：第二台排计划、分份那两趟都应当 capped.sh 5、退 0（实记「$(tr '\n' ';' <"$work/peer-threads-from-host-resources/calls.threads" 2>/dev/null)」）" \
      "看 peer_threads_now：没给 CRASH_AMPLIFICATION_PEER_THREADS 时取 crash_amplification_threads_now peer，两趟各在起之前现算"
  fi
  run_cell peer-threads-given CRASH_AMPLIFICATION_PEER_THREADS=3 STUB_PEER_THREADS=5 -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! peer_threads_are peer-threads-given 3; then
    cell_failed peer-threads-given "调用方给了 CRASH_AMPLIFICATION_PEER_THREADS=3：第二台那两趟都应当 capped.sh 3（不按现算的 5）、退 0（实记「$(tr '\n' ';' <"$work/peer-threads-given/calls.threads" 2>/dev/null)」）" \
      "看 peer_threads_now 把 CRASH_AMPLIFICATION_PEER_THREADS 交给 crash_amplification_threads_now 的第二个参数"
  fi
  run_cell peer-threads-at-least-one STUB_PEER_THREADS=0 -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! peer_threads_are peer-threads-at-least-one 1; then
    cell_failed peer-threads-at-least-one "假的 host-resources.py 报第二台此刻可用 0 个线程：第二台那两趟都应当 capped.sh 1（至少 1）、退 0（实记「$(tr '\n' ';' <"$work/peer-threads-at-least-one/calls.threads" 2>/dev/null)」）" \
      "看 crash_amplification_threads_now 里 max(1, available_threads)"
  fi
  run_cell peer-threads-unreadable STUB_HOST_RESOURCES=peer-unread -- some-target --local-states-per-second 150
  if [[ "$cell_exit" != 78 ]] || [[ -s "$work/peer-threads-unreadable/calls.threads" ]] || calls_have peer-threads-unreadable '^(local|peer) ' \
     || ! output_has peer-threads-unreadable '读不出第二台此刻的可用线程'; then
    cell_failed peer-threads-unreadable "假的 host-resources.py 读不到第二台：应当退 78、报读不出第二台此刻的可用线程，本机先判着与第二台那两趟都不起" \
      "看 peer_threads_now：现算不出时 exit 78，要在起本机先判着那一趟之前调"
  fi
  # 第二台那两趟判的线程数（SINGLEFS_CRASH_AMPLIFICATION_THREADS，假的用例记进 <调用记录>.checking-threads）：取第二台那一份，不沿用本机的。
  #   各格给本机 --threads 9，本机那几趟判 9；第二台的与它不同才分得出
  checking_threads_are() { # <格名> <第二台的线程数>：第二台排计划、分份两趟各一笔、都是它；本机那几趟至少一笔、都是 9
    local recorded="$work/$1/calls.checking-threads" local_lines
    local_lines="$(grep '^local ' "$recorded" 2>/dev/null)"
    [[ "$(grep '^peer ' "$recorded" 2>/dev/null | LC_ALL=C sort | tr '\n' ';')" == "peer placement=0 checking_threads=$2;peer placement=1 checking_threads=$2;" ]] \
      && [[ -n "$local_lines" ]] && ! grep -qv ' checking_threads=9$' <<<"$local_lines"
  }
  run_cell peer-checking-threads-from-host-resources STUB_PEER_THREADS=5 -- some-target --threads 9 --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! checking_threads_are peer-checking-threads-from-host-resources 5; then
    cell_failed peer-checking-threads-from-host-resources "假的 host-resources.py 报第二台此刻可用 5 个线程、本机 --threads 9：第二台排计划、分份那两趟判的线程数都应当是 5、本机那几趟是 9，退 0（实记「$(tr '\n' ';' <"$work/peer-checking-threads-from-host-resources/calls.checking-threads" 2>/dev/null)」）" \
      "看 peer_threads_now：peer_run_environment 换掉 common_environment 带来的本机那一份 SINGLEFS_CRASH_AMPLIFICATION_THREADS，两趟都用 peer_run_environment"
  fi
  run_cell peer-checking-threads-given CRASH_AMPLIFICATION_PEER_THREADS=3 STUB_PEER_THREADS=5 -- some-target --threads 9 --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! checking_threads_are peer-checking-threads-given 3; then
    cell_failed peer-checking-threads-given "调用方给了 CRASH_AMPLIFICATION_PEER_THREADS=3：第二台那两趟判的线程数都应当是 3（不按现算的 5、不沿用本机的 9）、退 0（实记「$(tr '\n' ';' <"$work/peer-checking-threads-given/calls.checking-threads" 2>/dev/null)」）" \
      "看 peer_threads_now：SINGLEFS_CRASH_AMPLIFICATION_THREADS 取 peer_threads，与 capped.sh 那一个数同源"
  fi
  run_cell peer-checking-threads-at-least-one STUB_PEER_THREADS=0 -- some-target --threads 9 --local-states-per-second 150
  if [[ "$cell_exit" != 0 ]] || ! checking_threads_are peer-checking-threads-at-least-one 1; then
    cell_failed peer-checking-threads-at-least-one "假的 host-resources.py 报第二台此刻可用 0 个线程：第二台那两趟判的线程数都应当是 1（至少 1）、退 0（实记「$(tr '\n' ';' <"$work/peer-checking-threads-at-least-one/calls.checking-threads" 2>/dev/null)」）" \
      "看 crash_amplification_threads_now 里 max(1, available_threads)，与 peer_threads_now 把它写进 peer_run_environment"
  fi
  # 本机只编那一步：另起一个 bash 只 source 共用库，调真的 crash_amplification_run_on_this_host；仓根换成格目录下的假树，
  # 里面的 run-with-memory-cap.sh 去掉上限照起、capped.sh 只把「线程数 只编还是跑」记进调用记录
  this_host_run() { # <格名> [<环境 键=值>…]：交回值放 cell_exit
    local name="$1"
    shift
    cell_directory="$work/$name"
    mkdir -p "$cell_directory/root/research/scripts"
    : >"$cell_directory/calls"
    cells=$((cells + 1))
    printf '%s\n' '#!/usr/bin/env bash' 'shift; exec "$@"' >"$cell_directory/root/research/scripts/run-with-memory-cap.sh"
    printf '%s\n' '#!/usr/bin/env bash' 'step=test; [[ " $* " == *" --no-run "* ]] && step=build' 'echo "threads=$1 step=$step" >>"${STUB_CALLS:?}"' >"$cell_directory/root/research/scripts/capped.sh"
    env -u CRASH_AMPLIFICATION_BUILD_THREADS STUB_HOST_RESOURCES= STUB_LOCAL_THREADS= STUB_PEER_THREADS= STUB_CALLS="$cell_directory/calls" "$@" \
      bash -c 'repository_root="$1"; source "$2" || exit 3; threads=7; memory_cap=1G; common_environment=(); gpu_cards_by_priority=""
               cargo_build_arguments=(test --no-run); cargo_test_arguments=(test)
               crash_amplification_run_on_this_host 0/1 "$3/library" "$3/run.log"' \
      this-host-run "$cell_directory/root" "$two_hosts_script_directory/crash-amplification-lib.sh" "$cell_directory" >"$cell_directory/output" 2>&1
    cell_exit=$?
  }
  this_host_run this-host-threads-from-host-resources STUB_LOCAL_THREADS=11
  if [[ "$cell_exit" != 0 || "$(tr '\n' ';' <"$work/this-host-threads-from-host-resources/calls")" != "threads=11 step=build;threads=7 step=test;" ]]; then
    cell_failed this-host-threads-from-host-resources "假的 host-resources.py 报本机此刻可用 11 个线程：只编那一步应当 capped.sh 11、跑的那一步照 threads（7）、交回 0" \
      "看 crash_amplification_run_on_this_host：只编那一步的线程数取 crash_amplification_threads_now local"
  fi
  this_host_run this-host-threads-given CRASH_AMPLIFICATION_BUILD_THREADS=3 STUB_LOCAL_THREADS=11
  if [[ "$cell_exit" != 0 || "$(tr '\n' ';' <"$work/this-host-threads-given/calls")" != "threads=3 step=build;threads=7 step=test;" ]]; then
    cell_failed this-host-threads-given "调用方给了 CRASH_AMPLIFICATION_BUILD_THREADS=3：只编那一步应当 capped.sh 3（不按现算的 11）、交回 0" \
      "看 crash_amplification_run_on_this_host 把 CRASH_AMPLIFICATION_BUILD_THREADS 交给 crash_amplification_threads_now 的第二个参数"
  fi
  this_host_run this-host-threads-at-least-one STUB_LOCAL_THREADS=0
  if [[ "$cell_exit" != 0 || "$(tr '\n' ';' <"$work/this-host-threads-at-least-one/calls")" != "threads=1 step=build;threads=7 step=test;" ]]; then
    cell_failed this-host-threads-at-least-one "假的 host-resources.py 报本机此刻可用 0 个线程：只编那一步应当 capped.sh 1（至少 1）、交回 0" \
      "看 crash_amplification_threads_now 里 max(1, available_threads)"
  fi
  this_host_run this-host-threads-unreadable STUB_HOST_RESOURCES=fails
  if [[ "$cell_exit" != 78 || -s "$work/this-host-threads-unreadable/calls" ]] || ! output_has this-host-threads-unreadable '现算本机线程数'; then
    cell_failed this-host-threads-unreadable "假的 host-resources.py 退 1：只编那一步应当交回 78、报现算本机线程数没成，一趟 cargo 都不起" \
      "看 crash_amplification_run_on_this_host：crash_amplification_threads_now 交回非 0 时 return 78，在起 capped.sh 之前"
  fi

  # ── 开关轮的判法（crash-amplification-lib.sh 的 crash_amplification_switch_wheel_verdict）：不起里层，拿一份造出来的输出（红在 cell-a、cell-b 两格）
  #   比几种点名：照实点名的、没列而一格不红的要判对上；删一格、多一格、格名写错、退出码不对、表里没写格、没列而红了的要判对不上
  cells=$((cells + 1)); cell_exit="（不起里层）"
  local wheel_sample="$work/switch-wheel-judges" exit_only=0 verdicts
  two_hosts_break_is wheel-ignores-cells && exit_only=1
  mkdir -p "$wheel_sample"; : >"$wheel_sample/calls"; : >"$wheel_sample/empty"
  printf '%s\n' '  ✗ 自证 cell-b：假的' '     → 怎么办：假的' '  ✗ 自证 cell-a：假的' '  ✗ 自证 cell-a：又一次' >"$wheel_sample/red"
  verdict_of() { # <点名的格> <退出码> <造出来的输出：red|empty>
    if crash_amplification_switch_wheel_verdict "$1" "$2" "$wheel_sample/$3" '✗ 自证 ([a-z0-9-]+)：' "$exit_only" >/dev/null; then echo 对上; else echo 对不上; fi
  }
  verdicts="照实点名=$(verdict_of 'cell-a cell-b' 1 red) 删一格=$(verdict_of cell-a 1 red) 多一格=$(verdict_of 'cell-a cell-b cell-c' 1 red)"
  verdicts+=" 格名写错=$(verdict_of 'cell-a cell-bb' 1 red) 退出码不对=$(verdict_of 'cell-a cell-b' 0 red) 没写格=$(verdict_of '' 1 red)"
  verdicts+=" 没列而一格不红=$(verdict_of - 0 empty) 没列而红了=$(verdict_of - 1 red)"
  printf '%s\n' "$verdicts" >"$wheel_sample/output"
  if [[ "$verdicts" != "照实点名=对上 删一格=对不上 多一格=对不上 格名写错=对不上 退出码不对=对不上 没写格=对不上 没列而一格不红=对上 没列而红了=对不上" ]]; then
    cell_failed switch-wheel-judges "开关轮的判法应当只认照实点名的（退 1、红的格逐格相等）与没列而一格不红的（退 0），别的都判对不上；实判「$verdicts」" \
      "看 crash-amplification-lib.sh 的 crash_amplification_switch_wheel_verdict：第五个参数是 0 时要逐格比，点名的格是空串一律对不上"
  fi

  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  if crash_amplification_switch_wheel_inner; then echo "  ✓ --selftest（开关轮里起的一份，不再起开关轮）：$cells 格都对"; exit 0; fi

  # 开关轮（文件头「弄坏开关」）：从文件头现读开关与点名的格、与代码里认的开关比齐，逐个打开再跑一遍 --selftest，红的格要与点名的逐格相等
  local table assignment named in_code in_library in_table_two="" in_table_item="" duplicated stray switch round_failures=0 listed_count unlisted_count=0
  switch_wheel_assignments=(); switch_wheel_expected=()
  local switch_table_source="${CRASH_AMPLIFICATION_TWO_HOSTS_SWITCH_TABLE:-$two_hosts_script}"
  [[ "$switch_table_source" == "$two_hosts_script" ]] || echo "  · 开关轮：开关表读自 $switch_table_source（CRASH_AMPLIFICATION_TWO_HOSTS_SWITCH_TABLE），代码里认的开关取自 $two_hosts_script"
  table="$(crash_amplification_read_switch_table "$switch_table_source" CRASH_AMPLIFICATION_TWO_HOSTS_BREAK CRASH_AMPLIFICATION_ITEM_BREAK)"
  while IFS=$'\t' read -r assignment named; do
    [[ -n "$assignment" ]] || continue
    switch_wheel_assignments+=("$assignment"); switch_wheel_expected+=("$named")
    case "$assignment" in
      CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=*) in_table_two+="${assignment#*=}"$'\n' ;;
      *) in_table_item+="${assignment#*=}"$'\n' ;;
    esac
  done <<<"$table"
  listed_count="${#switch_wheel_assignments[@]}"
  if (( listed_count == 0 )); then
    echo "  ✗ 开关轮：文件头「弄坏开关」那张表一个开关都没读到" >&2
    echo "     → 怎么办：表里一行「#   CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（文件头「弄坏开关」一段的写法）" >&2
    exit 1
  fi
  duplicated="$(printf '%s\n' "${switch_wheel_assignments[@]}" | LC_ALL=C sort | uniq -d | tr '\n' ' ')"
  if [[ -n "$duplicated" ]]; then
    echo "  ✗ 开关轮：文件头表里同一个开关列了不止一次：$duplicated" >&2
    echo "     → 怎么办：一个开关只写一行，点名的格都写在它下面" >&2
    round_failures=$((round_failures + 1))
  fi
  in_code="$(grep -oE 'two_hosts_break_is [a-z0-9-]+' "$two_hosts_script" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table_two="$(printf '%s' "$in_table_two" | LC_ALL=C sort -u)"
  if [[ "$in_code" != "$in_table_two" ]]; then
    echo "  ✗ 开关轮：代码里 two_hosts_break_is 认的开关与文件头表里列的双机开关不一样（只在代码里：$(LC_ALL=C comm -23 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table_two") | tr '\n' ' ')；只在表里：$(LC_ALL=C comm -13 <(printf '%s\n' "$in_code") <(printf '%s\n' "$in_table_two") | tr '\n' ' ')）" >&2
    echo "     → 怎么办：代码里每一处 two_hosts_break_is 认的开关都在表里写一行（连同点名的格），表里每一行都在代码里有对应的一处" >&2
    round_failures=$((round_failures + 1))
  fi
  in_library="$(grep -oE 'crash_amplification_item_break_is [a-z0-9-]+' "$two_hosts_script_directory/crash-amplification-lib.sh" | awk '{ print $2 }' | LC_ALL=C sort -u)"
  in_table_item="$(printf '%s' "$in_table_item" | LC_ALL=C sort -u)"
  stray="$(LC_ALL=C comm -13 <(printf '%s\n' "$in_library") <(printf '%s\n' "$in_table_item") | sed '/^$/d' | tr '\n' ' ')"
  if [[ -n "$stray" ]]; then
    echo "  ✗ 开关轮：文件头表里列的共用库开关，共用库里不认：$stray" >&2
    echo "     → 怎么办：照 crash-amplification-lib.sh 里 crash_amplification_item_break_is 认的名字写，不认的那一行删掉" >&2
    round_failures=$((round_failures + 1))
  fi
  # 共用库认而表里没列的：也逐个打开，要退 0、一格不红（点名写「-」）
  for switch in $(LC_ALL=C comm -23 <(printf '%s\n' "$in_library") <(printf '%s\n' "$in_table_item")); do
    switch_wheel_assignments+=("CRASH_AMPLIFICATION_ITEM_BREAK=$switch"); switch_wheel_expected+=(-); unlisted_count=$((unlisted_count + 1))
  done
  crash_amplification_switch_wheel "$work" '✗ 自证 ([a-z0-9-]+)：' "$exit_only" -- bash "$two_hosts_script" --selftest
  round_failures=$((round_failures + switch_wheel_failures))
  if (( round_failures > 0 )); then
    echo "  ✗ --selftest：$cells 格都对，开关轮有 $round_failures 处对不上" >&2
    echo "     → 怎么办：红的格与表里点名的不同，要么改那一处判法让开关红在点名的格，要么照实改文件头那一行；单看一个开关：CRASH_AMPLIFICATION_TWO_HOSTS_BREAK=<开关> $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：两台分份取回导入对账为空退 0、导入那一行（含红了之后先导入的）带 import_run=true、快照送第二台并复用、快照拷不成退 2、本机红先导入再原样退、本机卡用不了与给定状态表借不到显存退 78、本机与第二台那一份显存不够原样退 245、第二台连不上退 78、"
  echo "    第二台卡用不了先导入再退 78、库没取回退 78、对账不为空原样退、先判着按速度分、先判着那一趟卡用不了退 78、第二台量不出速度按日志退 78 / 78 / 248 / 249、第二台排计划借不到显存记本次未跑退 77（别的用例名的那一行照退 249）、"
  echo "    参数写错与用法错退 2、清过场没写那一份显卡配置退 78、清场判不过或没过退 78、清过场退出时不复原、"
  echo "    开不开 GPU 两台每一趟都不送 GPU_COMPUTATION_CLASSES（漏进来的也清掉）、显卡先后读多机配置（不读退役的私有显卡配置）、"
  echo "    第二台那两趟与本机只编那一步的线程数按 host-resources.py 现算（显式给的优先、至少 1、算不出退 78 不起；第二台那两趟判的线程数也取它、不沿用本机的）、开关轮的判法认得出删一格多一格与格名写错，$cells 格都对（本机、第二台与清场都是假的，不起 cargo）；"
  echo "    开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读：双机 $(grep -c . <<<"$in_table_two") 个、共用库 $(( listed_count - $(grep -c . <<<"$in_table_two") )) 个；共用库另有 $unlisted_count 个表里没列、要它一格不红）逐个打开，派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的格"
  exit 0
}

# 本机上限默认按机器空着的量给（用户 2026-09-28：有 40G 就给 40G，不给 12G）——但要留在 slice 总上限（整机 − 余量 20G = 40.1G）之内：
# 给满 40G 时 slice 里残留的几百 MiB 就让它排不进队、等满 3600 秒退 252（2026-09-28 到 E 全域第一次起跑本机那一半就这样卡了一小时），所以给 38G
test_target=""; expand_up_to=""; memory_cap="38G"; peer_memory_cap=""; label="crash-amplification-$(date -u +%Y%m%dT%H%M%SZ)-$$"; selftest=0; threads=""; library_directory=""; test_filter=""; quiesce=0
host_weights=""; local_states_per_second=""; quiesced=0
while (($# > 0)); do
  case "$1" in
    --expand-up-to) expand_up_to="${2:?--expand-up-to 要一个数}"; shift 2 ;;
    --test-filter) test_filter="${2:?--test-filter 要一个用例函数名}"; shift 2 ;;
    --quiesce) quiesce=1; quiesced=1; shift ;;
    --quiesced) quiesced=1; shift ;;
    --host-weights) host_weights="${2:?--host-weights 要 <本机>,<第二台> 两个正整数}"; shift 2 ;;
    --local-states-per-second) local_states_per_second="${2:?--local-states-per-second 要一个正整数}"; shift 2 ;;
    --library) library_directory="${2:?--library 要一个目录}"; shift 2 ;;
    --threads) threads="${2:?--threads 要一个数}"; shift 2 ;;
    --memory-cap) memory_cap="${2:?--memory-cap 要一个上限}"; shift 2 ;;
    --peer-memory-cap) peer_memory_cap="${2:?--peer-memory-cap 要一个上限}"; shift 2 ;;
    --label) label="${2:?--label 要一个标签}"; shift 2 ;;
    --selftest) selftest=1; shift ;;
    -*) usage ;;
    *) if [[ -n "$test_target" ]]; then usage; fi; test_target="$1"; shift ;;
  esac
done

if [[ -n "$host_weights" ]] && ! [[ "$host_weights" =~ ^[1-9][0-9]*,[1-9][0-9]*$ ]] && ! two_hosts_break_is accept-bad-host-weights; then
  echo "  ✗ --host-weights 要写 <本机>,<第二台> 两个正整数，读到「$host_weights」" >&2
  echo "  → 怎么办：例 --host-weights 3,1（本机领四分之三）" >&2
  exit 2
fi
if [[ -n "$local_states_per_second" ]] && ! [[ "$local_states_per_second" =~ ^[1-9][0-9]*$ ]] && ! two_hosts_break_is accept-bad-local-speed; then
  echo "  ✗ --local-states-per-second 要一个正整数，读到「$local_states_per_second」" >&2
  echo "  → 怎么办：给本机排派活计划那一趟 CRASH_AMPLIFICATION_PLACEMENT 行里 states_per_second_on_this_host 的合计" >&2
  exit 2
fi
if [[ "$selftest" == 1 ]]; then run_selftest; fi
library_is_temporary=0
crash_amplification_prepare || two_hosts_break_is prepare-failure-ignored || exit 78

# 库与日志的落点：/tmp 开机会清空（tmpfiles 的 D /tmp），2026-09-29 到 E 全域的库与树就丢在那里、续不上；正式跑一律放家目录下的固定位置，
# 重跑同一条命令就是续跑（同一判法版本下判过的块全部复用）。
crash_amplification_home="${SINGLEFS_CRASH_AMPLIFICATION_HOME:-$HOME/.local/share/singlefs/crash-amplification}"
run_directory="$crash_amplification_home/runs/$label"
local_library="$(crash_amplification_default_library "$crash_amplification_home")"
mkdir -p "$run_directory" || { echo "  ✗ 建不了 $run_directory" >&2; echo "  → 怎么办：看 ${crash_amplification_home} 可不可写" >&2; exit 2; }
# --library：本机的库跨次复用（已核过的块下一次不再核；第二台开跑前拿到它的一份快照，同样复用）
[[ -n "$library_directory" ]] && local_library="$library_directory"

# 本机的一趟（crash-amplification-lib.sh 的 crash_amplification_run_on_this_host）。<share> <库目录> <日志> [IMPORT=<库…>]；两台的分工随 weights_environment 带
weights_environment=()
run_local_share() {
  local share="$1" library="$2" log="$3" import="${4:-}"
  local extra=("${weights_environment[@]+"${weights_environment[@]}"}")
  [[ -n "$import" ]] && extra+=("SINGLEFS_CRASH_AMPLIFICATION_IMPORT=$import")
  crash_amplification_run_on_this_host "$share" "$library" "$log" "${extra[@]+"${extra[@]}"}"
}

# 转打导入那一趟的 CRASH_AMPLIFICATION 行，行尾加 import_run=true（文件头「导入那一趟」那一段）；三处导入都经这里
print_the_import_lines() {
  local import_mark=" import_run=true"
  two_hosts_break_is no-import-mark && import_mark=""
  grep -h '^CRASH_AMPLIFICATION ' "$run_directory/import.log" 2>/dev/null | sed "s/\$/$import_mark/"
}

[[ -n "$test_target" ]] || usage

# --quiesce：照配置里的清场命令先停，跑完不复原、退出时报一句「清了场、没复原」（做法同 layer0-shard-run.sh；命令与输出不打，里面有服务名）。不带就不清场：卡在额度之内用。
if [[ "$quiesce" == 1 ]]; then
  source "$peer_host_library"
  peer_host_load_configuration "$two_hosts_script_directory/layer0-shard-configuration-check.sh" "$repository_root" || two_hosts_break_is ignore-peer-configuration || { echo "  ✗ 多机配置判不过或没开双机（上面是判法的原因）" >&2; echo "  → 怎么办：修好设备，或改配置：照判法打出的出路改 multi-host.env" >&2; exit 78; }
  if ! peer_host_quiesce && ! two_hosts_break_is ignore-quiesce-failure; then
    echo "  ✗ 第二台清场没过（回读没通过）" >&2
    echo "  → 怎么办：修好设备，或改配置：看配置里 QUIESCE_STOP_COMMAND 与 QUIESCE_STOPPED_CHECK_COMMAND 在第二台上跑不跑得通" >&2
    exit 78
  fi
  if two_hosts_break_is restore-after-run; then
    trap 'peer_host_quiesce_left; bash -c "${QUIESCE_START_COMMAND:-}" >/dev/null 2>&1' EXIT
  else
    trap 'peer_host_quiesce_left' EXIT
  fi
fi

# 第二台那一趟的环境：两个开关、它是第二台、显卡配置里的优先级（树副本不带配置）；三个内核一个一个编（驱动编大内核吃内存，一起编在第二台上撞过内存上限）；
# 编译目录共用树副本上一层的 crash-amplification-target：每一趟的树副本是新拷的，不共用就每趟从头编 rocksdb 等依赖（约 3 分钟）
peer_environment=("${common_environment[@]}" ENABLE_ACROSS_MACHINES=1 "ENABLE_GPU=$gpu_switch" SINGLEFS_CRASH_AMPLIFICATION_HOST_ROLE=peer SINGLEFS_GPU_KERNELS_ONE_AT_A_TIME=1 CARGO_TARGET_DIR=../crash-amplification-target)
[[ -n "$gpu_cards_by_priority" ]] && peer_environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_by_priority")
# 走 GPU 的计算类不送（文件头「跑法」：引擎不读 GPU_COMPUTATION_CLASSES）；弄坏开关 classes-sent-to-peer 下照改前送三类
if (( gpu_switch )) && two_hosts_break_is classes-sent-to-peer; then
  peer_environment+=("GPU_COMPUTATION_CLASSES=gpu_unit_checks crash_verify_gpu crash_judge_gpu")
fi

# 往引擎送的两个变量（crash-point-replay.sh 文件头「往引擎送的两个变量」）：本机那几趟从调用方环境继承（env 不清它们）；第二台那一趟是新起的 ssh 会话，
# 只看得见命令里送过去的，所以两趟（排计划、分份）都照送：
#   对照跳过 SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER：本机环境里设了（值原样）才送，没设不送。
#   按段登记 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY：本机设了就把那份文件拷一份快照到 <日志目录>/segments-registry-for-peer/，随库快照一样经 --send 送进第二台那棵树的
#     crash-amplification-segments-registry/（树副本每趟新拷、跑完删，所以排计划、分份两趟各送一次）；变量值换成第二台上那一份的绝对路径——那个路径只有第二台自己的 shell 知道，
#     由命令最前面的 bash -c（peer_tree_path_script）用第二台的 $PWD 拼、文件不在就退 2 不起 cargo。
# 弄坏开关 cross-check-skip-not-sent-to-peer、segments-registry-not-sent-to-peer 下各自不送。
peer_tree_path_script='registry="$PWD/$2"; [[ -f "$registry" ]] || { echo "  ✗ 第二台那棵树里没有 $2（本机送过去的按段登记文件快照）" >&2; echo "  → 怎么办：看 multi-host-run.sh 的 --send 那几行有没有送到" >&2; exit 2; }; export "$1=$registry"; shift 2; exec "$@"'
peer_tree_path_prefix=(); peer_registry_send=()
if [[ -n "${SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER+set}" ]] && ! two_hosts_break_is cross-check-skip-not-sent-to-peer; then
  peer_environment+=("SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER=$SINGLEFS_CRASH_POINT_REPLAY_CROSS_CHECK_SKIPPED_BY_THE_USER")
fi
if [[ -n "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY:-}" ]] && ! two_hosts_break_is segments-registry-not-sent-to-peer; then
  segments_registry_for_peer="$run_directory/segments-registry-for-peer"
  segments_registry_file_name="$(basename -- "$SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY")"
  if ! { mkdir -p -- "$segments_registry_for_peer" && cp -- "$SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY" "$segments_registry_for_peer/$segments_registry_file_name"; }; then
    echo "  ✗ 拷不了按段登记文件 $SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY 的快照给第二台（落点 $segments_registry_for_peer）" >&2
    echo "  → 怎么办：看 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY 指的文件在不在、$run_directory 可不可写" >&2
    exit 2
  fi
  peer_registry_send=(--send "$segments_registry_for_peer" crash-amplification-segments-registry)
  peer_tree_path_prefix=(bash -c "$peer_tree_path_script" peer-tree-path SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY "crash-amplification-segments-registry/$segments_registry_file_name")
fi

# 第二台那一趟的线程数（文件头「线程」）：起 capped.sh 的放进 peer_threads；算不出退 78（这时这一趟还什么都没起）。
# 那一趟判的线程数 SINGLEFS_CRASH_AMPLIFICATION_THREADS 也取 peer_threads：peer_environment 里随 common_environment 带着的那一份是本机的
# （--threads 或本机核数减一），换掉它，组成这一趟的环境 peer_run_environment；弄坏开关 peer-checking-threads-from-this-host 下照改前留着本机那一份
peer_threads_now() {
  local entry
  if two_hosts_break_is peer-threads-fixed; then
    peer_threads=8
  elif ! peer_threads="$(crash_amplification_threads_now peer "${CRASH_AMPLIFICATION_PEER_THREADS:-}")"; then
    echo "  ✗ 第二台那一趟没起：它的线程数现算不出（上面一句是原因）" >&2
    echo "  → 怎么办：修好设备，或改配置：第二台连不上就修好它或改多机配置；要绕开现算就设 CRASH_AMPLIFICATION_PEER_THREADS=<正整数>（退 78：运行条件不满足）" >&2
    exit 78
  fi
  peer_run_environment=()
  if two_hosts_break_is peer-checking-threads-from-this-host; then
    peer_run_environment=("${peer_environment[@]}")
    return 0
  fi
  for entry in "${peer_environment[@]}"; do
    [[ "$entry" == SINGLEFS_CRASH_AMPLIFICATION_THREADS=* ]] || peer_run_environment+=("$entry")
  done
  peer_run_environment+=("SINGLEFS_CRASH_AMPLIFICATION_THREADS=$peer_threads")
}

# 给了本机的速度：第二台先排一趟派活计划、量它的速度，两台按速度分（四舍五入到整数，至少 1）
if [[ -z "$host_weights" && -n "$local_states_per_second" ]]; then
  peer_threads_now
  # 第二台量速度的那段时间本机先判着（整份），量完叫它停
  warm_stop_file="$run_directory/stop-the-run-on-this-host"
  rm -f -- "$warm_stop_file"
  crash_amplification_run_on_this_host 0/1 "$local_library" "$run_directory/warm-local.log" "SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE=$warm_stop_file" &
  warm_job=$!
  placement_arguments=(--peer --label "$label-placement" "${peer_registry_send[@]+"${peer_registry_send[@]}"}")
  [[ -n "$peer_memory_cap" ]] && placement_arguments+=(--memory-cap "$peer_memory_cap")
  bash "$multi_host_runner" "${placement_arguments[@]}" -- \
    "${peer_tree_path_prefix[@]+"${peer_tree_path_prefix[@]}"}" env "${peer_run_environment[@]}" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1 SINGLEFS_CRASH_AMPLIFICATION_SHARE=0/1 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=crash-amplification-library \
    bash research/scripts/capped.sh "$peer_threads" cargo "${cargo_test_arguments[@]}" >"$run_directory/peer-placement.log" 2>&1
  peer_placement_exit=$?
  two_hosts_break_is never-stop-warm-run || : > "$warm_stop_file"
  # 只供测试（文件头「只供测试」CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED）：叫停这一步走过了，在停下文件之后建
  [[ -n "${CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED:-}" ]] && : > "$CRASH_AMPLIFICATION_TWO_HOSTS_STOP_DECIDED"
  wait "$warm_job"; warm_exit=$?
  rm -f -- "$warm_stop_file"
  # 有意叫停的那一趟：CRASH_AMPLIFICATION 行尾加 warm_up=true（文件头「本机先判着」那一段）
  warm_up_mark=" warm_up=true"
  two_hosts_break_is no-warm-up-mark && warm_up_mark=""
  grep -hE '^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL|_STOPPED)? ' "$run_directory/warm-local.log" 2>/dev/null | sed "/^CRASH_AMPLIFICATION /s/\$/$warm_up_mark/"
  if [[ "$warm_exit" != 0 ]] && ! two_hosts_break_is warm-failure-ignored; then
    warm_exit_code="$(crash_amplification_exit_code_of_a_failed_run "$warm_exit" "$run_directory/warm-local.log")"
    echo "  ✗ 本机先判着的那一趟退 $warm_exit；日志 $run_directory/warm-local.log" >&2
    if two_hosts_exit_code_is_explained "$warm_exit_code"; then crash_amplification_explain_exit_code "$warm_exit_code"; else
      echo "  → 怎么办：看日志里 test result、panicked 与 run-with-memory-cap 那几行；判过的块已经在库里，重跑同一条命令接着判" >&2
    fi
    exit "$warm_exit_code"
  fi
  grep -h '^CRASH_AMPLIFICATION_PLACEMENT ' "$run_directory/peer-placement.log" 2>/dev/null
  peer_states_per_second="$(crash_amplification_placement_rate "$run_directory/peer-placement.log")"
  # 第二台排计划那一趟借不到显存（文件头「退出码」77）：那一趟退 0、没打计划行、行首是产品路径借不到显存的那一行。
  #   弄坏开关 peer-placement-not-run-as-failure 下不认（照没量出速度退 249），peer-placement-accept-any-not-run-line 下别的用例名的也认
  peer_not_run_prefix='NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason='
  two_hosts_break_is peer-placement-accept-any-not-run-line && peer_not_run_prefix='NOT_RUN_NO_GPU_ADAPTER test='
  peer_not_run_line=""
  if [[ "$peer_placement_exit" == 0 && -z "$(grep -m 1 '^CRASH_AMPLIFICATION_PLACEMENT ' "$run_directory/peer-placement.log" 2>/dev/null)" ]] \
     && ! two_hosts_break_is peer-placement-not-run-as-failure; then
    peer_not_run_line="$(awk -v prefix="$peer_not_run_prefix" 'index($0, prefix) == 1 { print; exit }' "$run_directory/peer-placement.log" 2>/dev/null)"
  fi
  if [[ -n "$peer_not_run_line" ]]; then
    echo "CRASH_AMPLIFICATION_NOT_RUN step=peer-placement test=crash_amplification::run_from_environment reason=${peer_not_run_line#*reason=}"
    echo "  ! 本次未跑（这不是通过）：第二台排派活计划那一趟借不到显存，两台都没分份判：${peer_not_run_line#*reason=}；日志 $run_directory/peer-placement.log" >&2
    echo "  · 等第二台上占着卡的 GPU 用例或测试进程跑完，再重跑同一条命令（本机先判着的块在库里，接着判）；配置了两台就要两台都跑，不改在本机一台跑" >&2
    exit 77
  fi
  if [[ "$peer_placement_exit" != 0 || -z "$peer_states_per_second" || "$peer_states_per_second" == 0 ]] && ! two_hosts_break_is peer-placement-failure-ignored; then
    echo "CRASH_AMPLIFICATION_PEER_PLACEMENT_FAILED exit=$peer_placement_exit"
    echo "  ✗ 第二台排派活计划那一趟没量出速度（退 $peer_placement_exit）；日志 $run_directory/peer-placement.log" >&2
    # 照日志分类：multi-host-run.sh 自己没跑成（255）或卡用不了 78，算法上判不了 248，认不出 249
    peer_placement_exit_code=249
    if [[ "$peer_placement_exit" == 255 ]] && ! two_hosts_break_is peer-unreachable-as-run-failure; then peer_placement_exit_code=78
    elif [[ "$peer_placement_exit" == 245 ]] && ! two_hosts_break_is gpu-memory-unexplained; then peer_placement_exit_code=245   # 那边的内存包装认出显存不够
    elif ! two_hosts_break_is peer-placement-unclassified; then
      case "$(crash_amplification_failure_kind "$run_directory/peer-placement.log")" in
        device|build) peer_placement_exit_code=78 ;;
        algorithm) peer_placement_exit_code=248 ;;
      esac
    fi
    if two_hosts_exit_code_is_explained "$peer_placement_exit_code"; then crash_amplification_explain_exit_code "$peer_placement_exit_code"; else
      echo "  → 怎么办：看日志里 GPU_CARD_NOT_USABLE、panicked、run-with-memory-cap 与 multi-host-run.sh 那几行（250 是第二台那一趟撞了内存上限）；配置了两台就要两台都跑成，修好第二台再重跑同一条命令" >&2
    fi
    exit "$peer_placement_exit_code"
  fi
  host_weights="$local_states_per_second,$peer_states_per_second"
fi
if [[ -n "$host_weights" ]]; then
  two_hosts_break_is drop-host-weights || weights_environment=("SINGLEFS_CRASH_AMPLIFICATION_HOST_WEIGHTS=$host_weights")
  echo "CRASH_AMPLIFICATION_HOST_WEIGHTS local,peer=$host_weights"
fi

peer_arguments=(--peer --label "$label" --fetch crash-amplification-library "${peer_registry_send[@]+"${peer_registry_send[@]}"}")
if [[ -f "$local_library/CURRENT" ]] && ! two_hosts_break_is no-library-snapshot; then
  # 库开着的时候不能拷（RocksDB 的文件在变）：本机那一份开跑之前先照一份快照，送给第二台当它的起点
  library_for_peer="$run_directory/library-for-peer"
  cp -a -- "$local_library" "$library_for_peer" || two_hosts_break_is ignore-snapshot-failure || { echo "  ✗ 拷不了库的快照给第二台" >&2; echo "  → 怎么办：看 $run_directory 可不可写" >&2; exit 2; }
  peer_arguments+=(--send "$library_for_peer" crash-amplification-library)
fi
peer_threads_now
run_local_share 0/2 "$local_library" "$run_directory/share-0.log" &
local_pid=$!
[[ -n "$peer_memory_cap" ]] && peer_arguments+=(--memory-cap "$peer_memory_cap")
fetch_directory="$run_directory/fetched"
peer_share=1/2
two_hosts_break_is same-share-on-both-hosts && peer_share=0/2
MULTI_HOST_RUN_FETCH_DIRECTORY="$fetch_directory" bash "$multi_host_runner" "${peer_arguments[@]}" -- \
  "${peer_tree_path_prefix[@]+"${peer_tree_path_prefix[@]}"}" env "${peer_run_environment[@]}" "${weights_environment[@]+"${weights_environment[@]}"}" "SINGLEFS_CRASH_AMPLIFICATION_SHARE=$peer_share" SINGLEFS_CRASH_AMPLIFICATION_LIBRARY=crash-amplification-library \
  bash research/scripts/capped.sh "$peer_threads" cargo "${cargo_test_arguments[@]}" >"$run_directory/share-1.log" 2>&1
peer_exit=$?
wait "$local_pid"; local_exit=$?
grep -hE '^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL)? ' "$run_directory/share-0.log" "$run_directory/share-1.log" 2>/dev/null
fetched_library="$fetch_directory/crash-amplification-library"
if [[ "$local_exit" != 0 ]] && ! two_hosts_break_is local-failure-ignored; then
  echo "  ✗ 本机那一份退 $local_exit；日志 $run_directory/share-0.log" >&2
  if [[ -f "$fetched_library/CURRENT" ]] && ! two_hosts_break_is local-failure-skips-salvage; then
    # 第二台那一份判过的块已经取回来了：先导进本机库再退，不因为本机这边红了就把第二台判的丢掉；下一次同一条命令接着跑
    run_local_share 0/2 "$local_library" "$run_directory/import.log" "$fetched_library"; salvage_exit=$?
    print_the_import_lines
    echo "  · 第二台取回的库已导入本机库（导入那一趟退 $salvage_exit）" >&2
  fi
  local_exit_code="$(crash_amplification_exit_code_of_a_failed_run "$local_exit" "$run_directory/share-0.log")"
  if two_hosts_exit_code_is_explained "$local_exit_code"; then crash_amplification_explain_exit_code "$local_exit_code"; else
    echo "  → 怎么办：看日志里 test result、panicked 与 run-with-memory-cap 那几行（250–254 是内存包装自己的结局）" >&2
  fi
  exit "$local_exit_code"
fi
if [[ "$peer_exit" != 0 ]] && ! two_hosts_break_is peer-failure-ignored; then
  echo "  ✗ 第二台那一份退 $peer_exit；日志 $run_directory/share-1.log" >&2
  if [[ -f "$fetched_library/CURRENT" ]] && ! two_hosts_break_is peer-failure-skips-salvage; then
    # 第二台判到一半的库取回来了：先导进本机库再退，它判过的块不丢；下一次同一条命令接着跑
    run_local_share 0/2 "$local_library" "$run_directory/import.log" "$fetched_library"; salvage_exit=$?
    print_the_import_lines
    echo "  · 第二台判到一半的库已导入本机库（导入那一趟退 $salvage_exit）：重跑同一条命令接着判" >&2
  fi
  # multi-host-run.sh 自己没跑成（255：配置、工具链、拷树、连不上、等不到结局）是第二台用不上，退 78；别的照 share-1.log 分类
  if [[ "$peer_exit" == 255 ]] && ! two_hosts_break_is peer-unreachable-as-run-failure; then peer_exit_code=78; else
    peer_exit_code="$(crash_amplification_exit_code_of_a_failed_run "$peer_exit" "$run_directory/share-1.log")"
  fi
  if two_hosts_exit_code_is_explained "$peer_exit_code"; then crash_amplification_explain_exit_code "$peer_exit_code"; else
    echo "  → 怎么办：看日志里第二台的 test result 与 panicked（250–254 是内存包装自己的结局）" >&2
  fi
  exit "$peer_exit_code"
fi
if [[ ! -d "$fetched_library" ]] && ! two_hosts_break_is accept-missing-fetched-library; then
  echo "  ✗ 第二台的库没取回：$fetched_library 不在" >&2
  echo "  → 怎么办：修好设备，或改配置：看 share-1.log 里 multi-host-run.sh --fetch 那几行，第二台那一趟的库落在哪、取回时连没连上（退 78）" >&2
  exit 78
fi
if two_hosts_break_is skip-import; then
  : > "$run_directory/import.log"; import_exit=0
else
  run_local_share 0/2 "$local_library" "$run_directory/import.log" "$fetched_library"; import_exit=$?
fi
print_the_import_lines
if [[ "$import_exit" != 0 ]] && ! two_hosts_break_is import-failure-ignored; then
  echo "  ✗ 导入第二台的库之后那一趟退 $import_exit（对账不为空或有红）；日志 $run_directory/import.log" >&2
  import_exit_code="$(crash_amplification_exit_code_of_a_failed_run "$import_exit" "$run_directory/import.log")"
  if two_hosts_exit_code_is_explained "$import_exit_code"; then crash_amplification_explain_exit_code "$import_exit_code"; else
    echo "  → 怎么办：看 import.log 里 CRASH_AMPLIFICATION 那一行的 unjudged_blocks 与 red，与 panicked 那几行" >&2
  fi
  exit "$import_exit_code"
fi
echo "  ✓ 双机各录各核、第二台的库拷回导入、对账为空、0 红（3 趟日志在 $run_directory）"
