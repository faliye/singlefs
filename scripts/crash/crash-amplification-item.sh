#!/usr/bin/env bash
# task-server-compiles: yes
# admission: always 每次调都在这一刻的树与库上排派活计划、再跑；哪些块要重判由库里的块键与判法单元指纹定，不在这里
# run-condition: command bash cargo python3
# run-condition: check source "$PREFLIGHT_SCRIPT_DIRECTORY/crash-amplification-lib.sh" && crash_amplification_run_condition_holds "$PREFLIGHT_SCRIPT_DIRECTORY/../.." :: 修好设备，或改配置：多机配置 multi-host.env 开了 GPU 而一份卡都没写、点名的卡与双机开关对不上；单跑 bash -c 'source scripts/crash/crash-amplification-lib.sh && crash_amplification_run_condition_holds' 看是哪一处
# 崩溃放量一个分项的一趟：用不用 GPU 照配置，做不到就报错退出，不退回 CPU
# （配置中有 gpu 就一定要让 GPU 工作，如果不行就明确报错拒绝，不接受默认降级）。
# 在哪台机器跑、上限、线程由 task-server 统一管理；卡归 GPU 批处理验证服务（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：
#   这一份一律在起它的那一台上一趟跑完（SHARE 0/1），不自己连第二台、不调双机驱动、不领卡、不挑卡、不清场、不读多机配置里的显卡先后；两台分着判的是崩溃放量的分片驱动
#   scripts/crash/crash-amplification-two-hosts.sh，由 task-server 照分片协议起，不经这里。
#
#   crash-amplification-item.sh <测试目标> [--test-filter <用例函数名>] [--label <标签>] [--stage record|judge|flow-shape] [--judge-binary <路径>]
#       --stage record：只跑录入段，打录入行；开了 GPU 时接着编判段要的测试二进制（步骤 ②「先编后判」），打一行
#       CRASH_AMPLIFICATION_JUDGE_BINARY test=<测试目标> path=<路径>（编成了而取不到二进制的不打），退 0（不排计划、不判、不写上次红的清单）；
#       --stage judge：不跑录入段（调用方已录过），排计划、判。
#       --stage flow-shape：只想看流的段形（各段哪几段没展开、穷举标记）：录入段照 --stage record 起，另给引擎一份已经存在的叫停文件
#       （SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE，建在这一项的运行目录里，跑完删），引擎录完流、打完 CRASH_AMPLIFICATION_RECORDED 与 CRASH_FLOW_COVERAGE 行之后，
#       观察者的录入段在头一个要录的块之前就见到叫停、不录任何观察者块；不编判段二进制、不排计划、不判、不写上次红的清单，
#       打 CRASH_AMPLIFICATION_RECORDED 行与日志里的 CRASH_FLOW_COVERAGE 行（一行都没有的退 1），退 0。流照录入键写进判定库的那一份（与 --stage record 同一把键），观察者块与判过的块一块都不写；
#       引擎在两种行之后、见到叫停之前还做的几样（读回录入的流、数库里的块与复用数、建观察者要的枚举表、把观察者基底写进库）照做。
#       不给是两段先后跑（下面「步骤」）；分两次起（录完、判各一趟）由调用方定。值写别的退 2。
#       --judge-binary：只跟 --stage judge 一起给（别的写法退 2），是 --stage record 那一趟打的路径；开了 GPU 时判段直接起它、不再编（它不在或不可执行打一行「!」、照没给）。
#   crash-amplification-item.sh --selftest      拿造出来的派活计划行与日志走一遍判法、运行条件与失败分类、没编进 gpu 特性而排计划那一趟退 0 也退 78（第 19 格）、判段没跑那一行的转打、逐趟汇总行（GPU_PASS）的转打、往引擎送的按段登记文件、环境里交的卡原样交下去、不领卡、只录与只判、先编后判（第 46 格）、排计划那一趟观察者交回的盘取不到卡记本次未跑退 78（第 17 格）、认不出的选项退 2（不编、不跑、不上卡、不连第二台）；
#       只在临时 prune-server 上跑（scripts/prune-server/prune_server_client_selftest.py --with-temporary-service 起，crash-amplification-lib.sh 的 crash_amplification_selftest_service_problem 判），不是就退 2、一格都不跑
#
# 测试目标是 crates/singlefs-verification/tests/ 下一条走 crash_amplification::run_from_environment 的用例。
# 运行条件分两半：配置上判得了的（多机配置开了 GPU 却一份卡都没写、点名的卡与双机开关对不上；配置怎么找见 crash-amplification-lib.sh 文件头「配置」）由文件头的 run-condition: check 在开跑之前判，
#   带 --force 越过时在运行中照判、退 78；跑起来才知道的（点名的卡用不了、显存装不下）在运行中退 78，出路是「修好设备，或改配置」。
#   算法上判不了的（有状态 GPU 判不了、内核展开超预算、建不出表）不是运行条件，退 248。日志怎么分这几类见 crash-amplification-lib.sh 的 crash_amplification_failure_kind。
# 步骤：
#   ① 多机配置（${SINGLEFS_MULTI_HOST_CONFIG:-主工作树的根 multi-host.env}，按 git common-dir 找）没开 GPU：先在本机整份录入段，再本机 CPU 判（SHARE 0/1）。
#   ② 开了 GPU：先在本机整份录入段（在 CPU 上跑），录成了先编判段要的测试二进制（先编后判：共用库 crash_amplification_build_judge_binary，cargo test --no-run 带判段那一趟的特性；
#      给了 --judge-binary 的不编、用它；编不成照日志分类退：设备与这份二进制跑不了 78、算法 248、别的原样退编那一趟的退出码，判段一趟都不起；
#      取不到二进制的打一行「!」、判段照旧经 cargo），排计划与本机判两趟都直接起这份二进制、不经 cargo（编完之后树被改过也不在判段重编）。
#      这里不领卡：环境里有 GPU_CARDS_BY_PRIORITY 就原样交下去，没有照样往下跑；判段把要判的块写成待判行、等 GPU 批处理服务判完
#      （验证包 crash_amplification 的 judge_my_blocks_on_gpu）；服务读不了时判段一直等，卡住由任务服务判。
#      再在本机跑一趟只排派活计划（SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1，不判），读它打的 CRASH_AMPLIFICATION_PLACEMENT 行
#      （一条用例里几条流各一行，逐个字段加起来）：判器要判的状态数 pending_states、核对要核而库里还没有核对行的状态数 verifier_pending_states，
#      两个合计打进 DECISION 行。这一趟不取卡、不借显存、不试判，行里没有速度与估时，这里也不算速度。有计划行就跑：两个数都是 0 的一趟（全部复用）也跑，复用数与判红由判段那一趟报。
#      一行都没有就不跑，打出日志里 panicked 那几行，照日志分类退：设备用不上 78、算法上判不了 248、那一趟退了非 0 而认不出原因退它的退出码、退 0 却没打计划行 247；
#      没编进 gpu 特性的二进制遇到 ENABLE_GPU=1，那一趟日志里有 CRASH_AMPLIFICATION_NOT_RUN step=judge 与 NOT_RUN_NO_GPU_ADAPTER 两行（crates/singlefs-verification/src/crash_amplification.rs 的
#      not_run_lines_without_the_gpu_feature），原因写「这份二进制没有编进 gpu 特性」，照日志分类退 78（build 那一类），不记本次未跑。
#   ③ 有计划行：本机一台跑（SHARE 0/1）。那一趟退非 0，照 run.log 分类：设备用不上 78、算法上判不了 248，别的原样退它的退出码。
# 判定库与日志：崩溃判定只在 prune-server 里那一份库（crash-amplification-lib.sh 文件头「判定库」：引擎经客户端直接读写，这里不拼库目录、不送库的变量；重跑同一条命令就是续跑），
#   录入段每趟照起，引擎按录入键（录入函数单元指纹与录入输入摘要）查服务，命中就不再录（crates/singlefs-verification/src/crash_library.rs 的 two_stages），
#   日志在 <家目录>/runs/<标签>/（家目录 ${SINGLEFS_CRASH_AMPLIFICATION_HOME:-<工作根>/crash-amplification}，共用库 crash_amplification_home_directory，两台同一份）：
#   record.log（录入段）、placement.log（排计划那一趟）、run.log（本机一台跑的那一趟）。
# 按段登记（判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J10）：往引擎送 SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY。
#   调用方送了（崩溃放量入口与崩溃点重放入口都送 <被判仓根>/.claude/gate.d/crash-point-replay-segments.tsv，开跑前已核过）就照它；没送就送本树那一份的绝对路径
#   （<本树根>/.claude/gate.d/crash-point-replay-segments.tsv，这里不核，写坏了由引擎在录入段拒）。录入段、排计划、本机判都从环境继承。
# 输出：CRASH_AMPLIFICATION_PLACEMENT 行原样转打；一行 CRASH_AMPLIFICATION_DECISION where=<local|local-cpu> reason=<为什么> …（where=local 带 pending_states=、verifier_pending_states= 两个合计）；
#   那一趟的 CRASH_AMPLIFICATION、CRASH_AMPLIFICATION_GPU_CARD、CRASH_AMPLIFICATION_GPU_TOTAL、CRASH_AMPLIFICATION_GPU_PASS 行按它们在 run.log 里的次序原样转打
#   （GPU_PASS 是判器逐趟的汇总行，紧跟在 GPU_TOTAL 之后；没有这种行的判器打出的日志照常过，不算错）。
#   判段没跑（配置写了 ENABLE_GPU=1 而二进制没编进 gpu 特性：引擎每一趟判段打一行 CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=<流> reason=…
#   与一行 NOT_RUN_NO_GPU_ADAPTER、录入段照录、退 0）：本机一台跑的那一趟的 step=judge 行原样转打（两台分片跑时由双机驱动在合并那一步从各片的日志转打），
#   崩溃放量入口照它把这一项记 not-run（入口文件头「一项怎么判」）。
# 源码中途变了（共用主工作区上别的会话往树里打了补丁）：录入段跑成之后记下被测源码的摘要（crash-amplification-lib.sh 的 crash_amplification_source_digest，文件头「源码摘要」；
#   树根是本脚本所在的仓根），判段每一趟（排计划那一趟、本机一台跑的那一趟）起跑前与收尾后各再算一次，对不上（或那时算不出）：打一行
#   CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment reason=跑的过程中源码变了，判段没在录入段那份源码上判完（录入段之后 <摘要>，<哪一刻> <摘要>），
#   标准错误说这不是通过，退 77（本次未跑），不落红：那一趟的汇总行不转打，红了也不按红退；崩溃放量入口照这一行把这一项记 not-run，崩溃点重放入口照 ITEM 行的 verdict=not-run 报本次未跑并点名这条原因。
#   录入段之后算不出摘要的打一行说明、判段前后不核。弄坏开关 record-judge-source-unchecked：不核。
# 退出码（0、2、77、78 的含义照 .claude/rules/test-script-discipline.md「入口脚本：不带参数就是剪枝跑全部项」那张表）：
#   0        跑的那一趟退 0。有流本次未跑也是 0（判段没跑的流打了 CRASH_AMPLIFICATION_NOT_RUN step=judge 行、真跑那一趟被叫停的行里 stopped_early=true）：
#            退 0 不等于全判过了，看有没有 CRASH_AMPLIFICATION_NOT_RUN 行与 stopped_early=true
#   77       一项都没跑到：判段跑的过程中源码变了（上面「源码中途变了」，打了 CRASH_AMPLIFICATION_NOT_RUN step=judge 行）。本次未跑，不是通过，也不是失败
#   2        用法错（参数认不得、缺测试目标、带值的选项缺值：--test-filter、--label、--stage 后面没给值，或给的是空串、以 - 起头；--stage 的值不是 record、judge、flow-shape）
#   78       运行条件不满足：开了 GPU 却没在多机配置里写显卡先后、点名的卡与双机开关对不上（开跑前由 run-condition: check 拒；带 --force 越过时照跑到下面那几样）、
#            点名的卡用不了或显存装不下、结论的版本与树对不上（CRASH_POINT_REPLAY_NOT_WRITTEN_CONCLUSION_VERSION_DIFFERS_FROM_THE_BUILD 那一行，整趟不写库）、
#            多机配置开了 GPU 而二进制没编进 gpu 特性（这三样都照日志分类，crash-amplification-lib.sh 的 crash_amplification_failure_kind）。出路：修好设备，或改配置；
#            观察者交回的盘要在卡上判、取不到卡（CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS 那一行，同照日志分类）记本次未跑：任务不拿卡，
#            出路是等 GPU 批处理服务收这一族待判行（共用库 crash_amplification_observer_images_without_a_card_howto），不是红
#   247      排派活计划那一趟退 0，却一行 CRASH_AMPLIFICATION_PLACEMENT 都没打，日志里也认不出是设备还是算法
#   248      算法上判不了：有状态 GPU 判不了（表越过内核容量）、内核展开超预算、GPU 判器建不出表（照日志分类）
#   其余     跑的那一趟（cargo test，或起编好的判段测试二进制）的退出码，原样；编判段要的测试二进制那一趟（cargo test --no-run）退非 0 而认不出类的，原样退它的退出码
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」「测试目标」，.claude/hooks/run-scope-guard.sh 的拒绝说明读它）：
#   <数据根>/last-runs/scripts__crash__crash-amplification-item.sh.tsv（SINGLEFS_LAST_RUNS_DIRECTORY 盖过；数据根取 .claude/hooks/lib_work_root.py、两台同一份），红的测试名一行一个：
#   「<测试目标>」，带 --test-filter 的「<测试目标>::<用例函数>」；退 0 是绿、退 2 / 77 / 130 / 143 照留，别的退出码是红；整个目标跑的那一趟换掉这个目标的全部项，
#   点名用例的那一趟只换那一项。开跑之后每一种退法都写（EXIT 钩子 item_write_last_run）；只判不跑那一支不写；
#   测试目标的源文件（tests/<目标>.rs，目录式目标是 tests/<目标>/main.rs；认法在 lib_crash_point_replay_items.py 的 target-source-file）不在、又没显式设 SINGLEFS_LAST_RUNS_DIRECTORY 的
#   （别的自证拿假目标起这一份）不写。
# 编译目录（自己用完自己删，不跨趟复用）：建好这一项的运行目录（<家目录>/runs/<标签>/）之后，调用方没设 CARGO_TARGET_DIR 时
#   在里面建 target、export 给本机那几趟 cargo；正常结束、红了、被 TERM / INT 停（等手里那一趟退出）都在 EXIT 钩子里删
#   （scripts/hosts/run-build-directory.sh；删之前核本机没有进程开着里面的文件，有就不删、报出进程、退非 0）。调用方（崩溃放量入口、task-server）设了的照它、不删：
#   文件头的会编声明「# task-server-compiles: yes」让统一任务服务给这一次尝试一份 CARGO_TARGET_DIR，撞上限重来的尝试接着用同一份、不重编，那一份归服务删。
# 两台对称（task-server 把这一项派到哪一台都一样）：运行条件里显卡先后每项写 <HOSTS 里的地址>:<卡号>:<额度>，哪几项是这一台的照多机配置判法 --emit-addresses 认
#   （crash-amplification-lib.sh 文件头「这一台与另一台」），不读角色变量；地址不在 HOSTS 里的项当写坏拒（退 78）。
# 跑 验证包用例：谁起、什么时候起归统一任务服务与 Bash 检出钩子，这里不另判、不往 cargo 带放行的变量。
# 弄坏开关 CRASH_AMPLIFICATION_ITEM_BREAK=<项>（只给 --selftest 证红用；格号是 run_selftest 里的第 1–48 格，判错的那一格打「✗ 自证第 <号> 格」）：
#   accept-peer-cards-without-across   双机没开、显卡配置却点名了第二台的卡也照跑（「配置自相矛盾就拒绝」判错）   第 3 格
#   accept-missing-peer-cards          双机开着、显卡配置里却没有第二台的卡也照跑（「两台都要有 GPU 干活」判错）   第 4 格
#   accept-missing-placement           计划行一行都没有也照本机跑（「没有计划行就不跑」判错）                   第 6 格、第 8 格、第 9 格、第 17 格、第 19 格、第 23 格、第 24 格（第 17、19、23、24 格的日志也没有计划行：没编进 gpu 特性、结论版本对不上、观察者交回的盘取不到卡那几句本该退 78，开关一开先走「没有计划行照本机跑」，没退 78）
#   verifier-pending-states-not-counted  DECISION 行里核对要核的 verifier_pending_states 不取计划行的合计，恒写 0（「两个合计都打进 DECISION 行」判错）  第 20 格
#   device-unusable-as-run-failure     日志里卡用不上的原话不认（不退 78）                                    第 8 格、第 11 格
#   algorithm-refusal-as-run-failure   日志里算法判不了的原话不认（不退 248）                                 第 9 格
#   accept-missing-card-list           开了 GPU、多机配置里一份卡都没写也照跑（「没写卡就退 78」判错）           第 10 格、第 13 格
#   missing-value-exits-1              带值的选项缺值时退 1（不按用法错退 2）                                    第 12 格
#   （共用库 crash-amplification-lib.sh 里配置那几处的开关：configuration-from-tree-root（多机配置按树根找、不按 git common-dir 找）红在第 21 格；
#    home-under-home-directory（没设 SINGLEFS_CRASH_AMPLIFICATION_HOME 时运行目录取 $HOME 底下那一份、不取工作根）红在第 38 格；cards-read-from-configuration（显卡先后从多机配置读、不看环境里交的那一张）
#    红在第 15 格、第 42 格、第 44 格）
#   （共用库的开关 given-states-run-condition-as-run-failure（给定状态表的判器借不到显存那一行 CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 不认、不退 78）
#    打开时这里一格不红：这里的格都不造那一行；它由 scripts/crash/crash-amplification-two-hosts.sh --selftest 证红，红在 shard-given-states-run-condition 格）
#   （共用库的开关 library-marker-not-accepted（录入段只认 CRASH_AMPLIFICATION_RECORDED 行、不认崩溃枚举用例经判定库两段跑时打的 CRASH_LIBRARY_RECORDED 行）红在第 28 格：
#    照崩溃点重放用例的样子造的假用例录入段只打 CRASH_LIBRARY_RECORDED、退 1；scripts/crash/crash-amplification-two-hosts.sh --selftest 另红在 crash-case-shaped-records 格）
#   version-differs-as-run-failure     日志里结论版本对不上那一行不认（不退 78）                               第 23 格
#   gpu-feature-missing-as-run-failure 日志里「这份二进制没有编进 gpu 特性」不认（不退 78）                    第 19 格、第 24 格（第 19 格：排计划那一趟退 0、没有计划行、日志里只有没编进 gpu 特性那两行，本该退 78，开关一开认不出原因退 247）
#   observer-images-without-a-card-as-run-failure  排计划那一趟没有计划行时，观察者交回的盘取不到卡那一类不认（照认不出的原因退 247，不退 78、不打共用库那一类的出路）（「观察者交回的盘取不到卡记本次未跑退 78」判错）   第 17 格
#   record-failure-ignored             录入段没跑成也照样往下排计划、判（「录入段没跑成，判段一趟都不起」判错）   第 26 格
#   segments-registry-not-sent         调用方没送按段登记文件时也不送本树那一份（「每一趟都带按段登记文件的绝对路径」判错）   第 27 格
#   judge-not-run-dropped              本机一台跑的那一趟打的 CRASH_AMPLIFICATION_NOT_RUN step=judge 行不转打（「判段没跑记本次未跑」判错）   第 29 格
#   gpu-pass-lines-dropped             本机一台跑的那一趟打的 CRASH_AMPLIFICATION_GPU_PASS 行不转打（只转 CRASH_AMPLIFICATION、GPU_CARD、GPU_TOTAL 三种）（「逐趟汇总行转进任务日志」判错）   第 47 格
#   threads-default-cores              线程数取本机核数减一，不取服务交的 SINGLEFS_THREAD_CAP 与登记的重活默认并行度        第 30 格
#   unknown-option-accepted            认不出的选项也照收、当没给（「认不出的选项退 2」判错）   第 18 格
#   last-run-unwritten                 跑完不写上次红的清单                                        第 31 格
#   target-file-flat-only              上次红的清单的 --target-file 只给 tests/<目标>.rs，目录式目标的 tests/<目标>/main.rs 不认（那份文件算「不在」，清单悄悄不写）   第 43 格
#   run-target-kept                    跑完不删这一项自己建的编译目录（在仓根照仓根的 target 编、跨趟留着）（「跑完运行目录里不剩它建的编译目录」判错）   第 33 格、第 34 格、第 35 格
#   this-host-from-hosts-order         共用库不问多机配置判法 --emit-addresses，一律当 HOSTS 头一项是这一台（「哪一台是这一台照网卡上的地址认」判错）   第 37 格
#   card-address-outside-hosts-accepted  共用库照认显卡先后里地址不在 HOSTS 的项（「写坏的显卡先后退 78」判错）   第 40 格
#   test-variables-honored-outside-selftest  不在自证里设了只供测试的变量也照认（「只供测试的变量不在自证里设了退 2」判错）   第 39 格
#   record-judge-source-unchecked      录入段之后、判段起跑前与收尾后不核被测源码的摘要（源码中途变了照样往下判，判红按红退）（「跑的过程中源码变了记本次未跑、退 77、不落红」判错）   第 41 格
#   service-build-directory-ignored    环境里给了的 CARGO_TARGET_DIR（崩溃放量入口或服务给的）不认，照没给时自己建、用完删（「给了就照它编、不删」判错）   第 36 格、第 45 格
#   flow-shape-stop-file-unsent        --stage flow-shape 不给引擎叫停文件（观察者的录入段照录，跑到录入段结束）（「只录流、不录观察者」判错）   第 48 格
#   flow-shape-judge-binary-built      --stage flow-shape 开了 GPU 时也接着编判段要的测试二进制并打 CRASH_AMPLIFICATION_JUDGE_BINARY 行（「只录流、不编判段二进制」判错）   第 48 格
#   （多机配置判法 scripts/crash/crash-point-replay-shard-configuration-check.sh 的开关 MULTI_HOST_CONFIGURATION_BREAK=gpu-card-keys-unknown 红在第 22 格）
# 只供测试（只在 --selftest 里认：run_selftest 给它起的子进程设 CRASH_AMPLIFICATION_ITEM_SELFTEST=1；没有这个标记而设了下面任一个，参数解析之后退 2、点名设了的几个、一趟都不起）：
#   CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY=<造好的 placement.log> 时不编不跑，只拿那份日志判、打 CRASH_AMPLIFICATION_DECISION 行就退；
#   CRASH_AMPLIFICATION_ITEM_SOURCE_ROOT=<树根> 代替算被测源码摘要的树（缺省是本脚本所在的仓根；自证给一棵造出来的小树，不读真仓的 crates/——别的会话正改着它）；
#   CRASH_AMPLIFICATION_ITEM_SWITCHES=<gpu>,<双机> 代替读配置里的这两样（双机那一位只给运行条件判）；CRASH_AMPLIFICATION_ITEM_CARDS=<显卡配置的值> 代替运行条件读的显卡配置；
#   CRASH_AMPLIFICATION_ITEM_LOCAL_RUN=<脚本> 换掉本机那一趟（调法：<脚本> <share> <日志> [<附加环境 键=值>…]，自己写日志；环境照真的那一趟组（共用库
#   crash_amplification_environment_for_this_host），这一趟的 cargo test 参数——共用库拼的 cargo_test_arguments，空格连起来——放进它环境里的 CRASH_AMPLIFICATION_ITEM_LOCAL_TEST_ARGUMENTS，
#   判段要的测试二进制（共用库 crash_amplification_judge_binary，没有是空串）放进 CRASH_AMPLIFICATION_ITEM_LOCAL_JUDGE_BINARY）；编判段要的测试二进制那一趟
#   （共用库 crash_amplification_build_judge_binary）同样换成它：附加环境多一项 CRASH_AMPLIFICATION_ITEM_LOCAL_BUILD=1，CRASH_AMPLIFICATION_ITEM_LOCAL_TEST_ARGUMENTS 放
#   cargo_build_arguments，它往日志里写的 Executable 行照共用库 crash_amplification_test_binary_in_log 取成判段要的测试二进制。
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
# <测试目标>：上次红的清单的 --target-file（item_write_last_run）：测试目标的源文件，tests/<目标>.rs，目录式目标是 tests/<目标>/main.rs，两样都不在的照给 tests/<目标>.rs（拿它在不在认出自证造的假目标）。
#   认法在 lib_crash_point_replay_items.py 的 target-source-file，取不出也给 tests/<目标>.rs；弄坏开关 target-file-flat-only 下只给 tests/<目标>.rs
item_target_source_file() {
  if ! item_break_is target-file-flat-only; then
    python3 "$item_script_directory/lib_crash_point_replay_items.py" target-source-file "$repository_root" "$1" 2>/dev/null && return 0
  fi
  echo "$repository_root/crates/singlefs-verification/tests/$1.rs"
}
exit_code_is_explained() { [[ "$1" == 78 || "$1" == 248 ]]; }   # <要退的码>：78、248 由 crash_amplification_explain_exit_code 打出路

usage() { # [<哪里不对>]
  echo "  ✗ ${1:+$1；}用法：$0 <测试目标> [--test-filter <用例函数名>] [--label <标签>] [--stage record|judge|flow-shape] [--judge-binary <路径>（只跟 --stage judge）]，或 --selftest" >&2
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

test_target=""; label="crash-amplification-$(date -u +%Y%m%dT%H%M%SZ)-$$"; selftest=0; test_filter=""; item_stage=""; judge_binary_given=""
while (($# > 0)); do
  case "$1" in
    --test-filter) require_value "$1" $# "${2:-}"; test_filter="$2"; shift 2 ;;
    --judge-binary) require_value "$1" $# "${2:-}"; judge_binary_given="$2"; shift 2 ;;
    --label) require_value "$1" $# "${2:-}"; label="$2"; shift 2 ;;
    --stage)
      require_value "$1" $# "${2:-}"
      [[ "$2" == record || "$2" == judge || "$2" == flow-shape ]] \
        || usage "--stage 只认 record（只录）、judge（不录、排计划与判）与 flow-shape（只录流、打段形、不录观察者），收到「$2」"
      item_stage="$2"; shift 2 ;;
    --selftest) selftest=1; shift ;;
    -*)
      # 弄坏开关 unknown-option-accepted：认不出的选项也照收、当没给（不拒）
      if item_break_is unknown-option-accepted; then shift; continue; fi
      usage ;;
    *) if [[ -n "$test_target" ]]; then usage; fi; test_target="$1"; shift ;;
  esac
done
# --judge-binary 只跟 --stage judge 一起给（文件头用法）
if [[ -n "$judge_binary_given" && "$item_stage" != judge ]]; then usage "--judge-binary 只跟 --stage judge 一起给（它是 --stage record 那一趟编好的判段测试二进制）"; fi

# 跑不跑：读 placement.log，打 CRASH_AMPLIFICATION_DECISION 行；交回 0 本机一台跑（在哪台机器由 task-server 管）、
# 1 没有计划行（该退的码放进 refused_exit_code：设备用不上 78、算法上判不了 248、没编进 gpu 特性与结论版本对不上 78、那一趟退了非 0 而认不出原因就是它的退出码、退 0 却没打计划行 247）
decide_where_to_run() { # <placement.log> <排计划那一趟的退出码>
  local placement_log="$1" placement_exit="$2"
  local pending verifier_pending kind
  pending="$(crash_amplification_placement_total "$placement_log" pending_states)"
  if [[ -z "$pending" ]]; then
    if item_break_is accept-missing-placement; then
      echo "CRASH_AMPLIFICATION_DECISION where=local reason=no-placement-line"
      return 0
    fi
    echo "  ✗ 排派活计划那一趟（退 $placement_exit）一行 CRASH_AMPLIFICATION_PLACEMENT 都没打，不跑；日志 $placement_log" >&2
    grep -h -A 3 'panicked at' "$placement_log" 2>/dev/null | head -8 | sed 's/^/    /' >&2
    kind="$(crash_amplification_failure_kind "$placement_log")"
    # 观察者交回的盘要在卡上判、取不到卡那一类（共用库文件头「运行条件与失败分类」）记本次未跑、退 78，出路照共用库那一类打（crash_amplification_explain_exit_code）。
    #   弄坏开关 observer-images-without-a-card-as-run-failure 下不认这一类，照认不出的原因退
    [[ "$kind" == observer-images-without-a-card ]] && item_break_is observer-images-without-a-card-as-run-failure && kind=""
    case "$kind" in
      gpu-verification-server|observer-images-without-a-card|device|build) refused_exit_code=78 ;;
      algorithm) refused_exit_code=248 ;;
      *) if [[ "$placement_exit" != 0 ]]; then refused_exit_code="$placement_exit"; else refused_exit_code=247; fi ;;
    esac
    if exit_code_is_explained "$refused_exit_code"; then
      crash_amplification_explain_exit_code "$refused_exit_code" "$placement_log"
    else
      echo "  → 怎么办：照上面 panicked 那几行与日志全文改；日志里认不出是设备还是算法，单跑那一趟看全文（退 $refused_exit_code）" >&2
    fi
    return 1
  fi
  # 核对要核的状态数：弄坏开关 verifier-pending-states-not-counted 下不取，恒写 0
  verifier_pending="$(crash_amplification_placement_total "$placement_log" verifier_pending_states)"
  item_break_is verifier-pending-states-not-counted && verifier_pending=0
  echo "CRASH_AMPLIFICATION_DECISION where=local reason=placed-on-this-host pending_states=$pending verifier_pending_states=$verifier_pending placed_by=task-server"
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
  # 调用方交的卡与编译目录（CARGO_TARGET_DIR）不从调用方带进来（卡由第 15、44 格自己给，编译目录由第 36、45 格自己给）
  unset GPU_CARDS_BY_PRIORITY CARGO_TARGET_DIR
  # 假的本机那一趟的次序记录（第 46 格自己给）不从调用方带进来
  unset STUB_ORDER_LOG
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
  # 两份开了 GPU 的多机配置：一份卡都没写（第 13 格）；只点名本机的卡（第 14 格，第 15 格也读它：配置里的卡与环境里服务交的卡分得开）
  { cat "$work/multi-host-required.env"; printf '%s\n' ENABLE_GPU=1 QUIESCE_STOP_COMMAND= QUIESCE_STOPPED_CHECK_COMMAND= QUIESCE_START_COMMAND= QUIESCE_STARTED_CHECK_COMMAND=; } \
    >"$work/multi-host-gpu-on-without-cards.env"
  { cat "$work/multi-host-gpu-on-without-cards.env"; printf '%s\n' 'GPU_CARDS_BY_PRIORITY="127.0.0.1:0:15G"' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="127.0.0.1:0:24G"'; } \
    >"$work/multi-host-gpu-on-with-cards.env"
  selftest_cell_failed() { # <这一格判错的一句> <出路>
    echo "  ✗ 自证第 $cells 格：$1" >&2
    echo "     → 怎么办：$2" >&2
    failures=$((failures + 1))
  }
  placement_line() { # <判器要判的态数> <核对要核的态数>：引擎只排计划那一趟打的一行（crash_amplification.rs 的 placement_line_of_the_pending_blocks）
    echo "CRASH_AMPLIFICATION_PLACEMENT judge=gpu pending_blocks=4 pending_states=$1 states_reused=0 tables_mebibytes=10 verifier_pending_blocks=2 verifier_pending_states=$2"
  }
  decision_of() { # <placement.log> <gpu,双机> <显卡配置的值>：打 where= 的值与退出码
    local output exit_code
    output="$(CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$1" CRASH_AMPLIFICATION_ITEM_SWITCHES="$2" CRASH_AMPLIFICATION_ITEM_CARDS="$3" bash "$0" some-target 2>/dev/null)"; exit_code=$?
    printf '%s %s\n' "$(grep -o 'where=[a-z-]*' <<<"$output" | head -1)" "$exit_code"
  }
  local both="127.0.0.1:0:15G 2001:db8::65:2:10G" local_only="127.0.0.1:0:15G"
  placement_line 1000 300 >"$work/plan.log"
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
  # 第 5 格：一条用例几条流：判器要判的状态数加起来（两条流各 400 态 ⇒ 800 态；只取第一行是 400）
  cells=$((cells + 1)); { placement_line 400 100; placement_line 400 200; } >"$work/two-flows.log"
  local two_flows
  two_flows="$(CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/two-flows.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" bash "$0" some-target 2>/dev/null)"
  grep -q ' pending_states=800 ' <<<"$two_flows" \
    || selftest_cell_failed "两条流的要判状态没加起来：$two_flows" "看 crash-amplification-lib.sh 的 crash_amplification_placement_total 与 decide_where_to_run 里读 pending_states 的那一句"
  # 第 6 格：一行计划都没有、日志里认不出原因、那一趟退 0 ⇒ 不跑，退 247
  cells=$((cells + 1)); echo "test result: FAILED" >"$work/none.log"
  [[ "$(decision_of "$work/none.log" 1,1 "$both")" == " 247" ]] \
    || selftest_cell_failed "没有计划行、认不出原因却没退 247：$(decision_of "$work/none.log" 1,1 "$both")" "看 decide_where_to_run 开头 pending 为空那一支：分不出类、那一趟退 0 时 refused_exit_code=247"
  cat >"$work/local-run.sh" <<'STUB'
#!/usr/bin/env bash
# 假的本机那一趟：录入段那一次写一行 RECORDED（STUB_RECORD_EXIT 给了就照它退、不写那一行），排计划的那一次写一行计划，
# 真跑的那一次写一行汇总（STUB_RUN_DEVICE_FAILURE=1 时写一行卡用不了、退 101）；每次调记一笔，收到的卡另记一笔进 <调用记录>.cards
log="$2"
# 编判段要的测试二进制那一趟（附加环境里有 CRASH_AMPLIFICATION_ITEM_LOCAL_BUILD=1）：不记进 STUB_CALLS；给了 STUB_ORDER_LOG 的记「build <带 --no-run 是 no-run，否则 run> features=<--features 的值，没有 none>」，
#   在日志所在目录写一份可执行的假二进制 judge-binary、日志里打 cargo 那种 Executable 行指到它，退 0
if [[ " $* " == *" CRASH_AMPLIFICATION_ITEM_LOCAL_BUILD=1 "* ]]; then
  build_words=" ${CRASH_AMPLIFICATION_ITEM_LOCAL_TEST_ARGUMENTS:-} "
  build_target="${build_words#* --test }"; build_target="${build_target%% *}"
  build_features=none
  if [[ "$build_words" == *" --features "* ]]; then build_features="${build_words#* --features }"; build_features="${build_features%% *}"; fi
  build_mode=run; [[ "$build_words" == *" --no-run "* ]] && build_mode=no-run
  [[ -z "${STUB_ORDER_LOG:-}" ]] || echo "build $build_mode features=$build_features" >> "$STUB_ORDER_LOG"
  printf '%s\n' '#!/usr/bin/env bash' 'exit 0' > "$(dirname "$log")/judge-binary"; chmod +x "$(dirname "$log")/judge-binary"
  echo "     Executable tests/$build_target.rs ($(dirname "$log")/judge-binary)" > "$log"
  exit 0
fi
# 次序记录（第 46 格）：给了 STUB_ORDER_LOG 的录入段那一趟记「record」，排计划与本机判两趟记「placement|local-run binary=[<收到的判段测试二进制，没有 unset>]」
if [[ -n "${STUB_ORDER_LOG:-}" ]]; then
  if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_STAGE=record "* ]]; then echo record
  elif [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1 "* ]]; then echo "placement binary=[${CRASH_AMPLIFICATION_ITEM_LOCAL_JUDGE_BINARY:-unset}]"
  else echo "local-run binary=[${CRASH_AMPLIFICATION_ITEM_LOCAL_JUDGE_BINARY:-unset}]"; fi >> "$STUB_ORDER_LOG"
fi
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
  # 覆盖行（第 48 格读它）：STUB_NO_COVERAGE=1 时不打（演引擎没打覆盖行）；STUB_STOP_FILE_LOG 给了的记一笔收到的叫停文件在不在（stop-file=present、missing、none）
  if [[ "${STUB_NO_COVERAGE:-0}" != 1 ]]; then
    echo "CRASH_FLOW_COVERAGE entry=stub step=UnitWrite segment=0 expansion=expanded" >> "$log"
    echo "CRASH_FLOW_COVERAGE entry=stub step=RootRecordFua segment=1 expansion=not_expanded" >> "$log"
  fi
  if [[ -n "${STUB_STOP_FILE_LOG:-}" ]]; then
    stub_stop_state=none
    for stub_word in "$@"; do
      if [[ "$stub_word" == SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE=* ]]; then
        if [[ -f "${stub_word#*=}" ]]; then stub_stop_state=present; else stub_stop_state=missing; fi
      fi
    done
    echo "stop-file=$stub_stop_state" >> "$STUB_STOP_FILE_LOG"
  fi
  exit 0
fi
if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_PLACEMENT_ONLY=1 "* ]]; then
  echo "placement" >> "${STUB_CALLS:?}"
  # STUB_MUTATE_SOURCE（第 41 格）：给的文件追加一行，演判段跑的过程中别的会话往树里打了补丁
  if [[ -n "${STUB_MUTATE_SOURCE:-}" && "${STUB_MUTATE_IN:-}" == placement ]]; then echo "changed" >> "$STUB_MUTATE_SOURCE"; fi
  if [[ "${STUB_PLACEMENT_WITHOUT_THE_GPU_FEATURE:-0}" == 1 ]]; then
    # 第 19 格：演没编进 gpu 特性的二进制遇到 ENABLE_GPU=1——照 Rust 的 not_run_lines_without_the_gpu_feature 打那两行（不打计划行），退 0
    printf '%s\n' "running 1 test" \
      "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment flow=stub reason=配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（假的用例）" \
      "NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment reason=配置写了 ENABLE_GPU=1，这份二进制没有编进 gpu 特性（假的用例）" \
      "test result: ok. 1 passed; 0 failed" > "$log"
    exit 0
  fi
  echo "CRASH_AMPLIFICATION_PLACEMENT judge=gpu pending_blocks=4 pending_states=1000 states_reused=0 tables_mebibytes=10 verifier_pending_blocks=2 verifier_pending_states=300" > "$log"
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
      selftest_cell_failed "开了 GPU、多机配置$( [[ "$cards_cell" == 13 ]] && echo 一份卡都没写 || echo 只点名本机的卡 )：运行条件应当交回 $cards_want$( [[ "$cards_cell" == 13 ]] && echo '、打「一份都没写」')，实际交回 $cards_exit（$(tr '\n' ' ' <"$work/cards-$cards_cell.out" | crash_amplification_clip_text 200)）" \
        "看 crash-amplification-lib.sh 的 crash_amplification_run_condition_holds：显卡先后从多机配置读"
    fi
  done
  # 第 15 格：多机配置开了 GPU、显卡先后写着 127.0.0.1:0:15G，环境里是调用方交的 127.0.0.1:1:20G ⇒ 录入段、排计划、跑三趟收到的卡都是环境里那一张，退 0
  cells=$((cells + 1))
  : > "$work/calls"; : > "$work/calls.cards"
  local card_exit
  STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on-with-cards.env" GPU_CARDS_BY_PRIORITY=127.0.0.1:1:20G \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target >"$work/service-card.out" 2>&1; card_exit=$?
  if [[ "$card_exit" != 0 || "$(tr '\n' ' ' <"$work/calls")" != "record placement local-run " || "$(tr '\n' ' ' <"$work/calls.cards")" != "cards=[127.0.0.1:1:20G] cards=[127.0.0.1:1:20G] cards=[127.0.0.1:1:20G] " ]]; then
    selftest_cell_failed "开了 GPU：三趟收到的卡应当都是环境里交的 127.0.0.1:1:20G、不读多机配置里的显卡先后（退 $card_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」，收到的卡「$(tr '\n' ' ' <"$work/calls.cards")」）" \
      "看 crash-amplification-lib.sh 的 crash_amplification_prepare 取卡那一段与 crash_amplification_environment_for_this_host"
  fi
  # 第 16 格：开了 GPU、多机配置里写着假的清场命令 ⇒ 不再自己清场：调用次序「录入段、排计划、跑」、没有 quiesce-stop，输出里没有经清场包装重起的那一句，退 0
  cells=$((cells + 1))
  : > "$work/calls"
  local quiesce_exit
  STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesce-commands.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" GPU_CARDS_BY_PRIORITY="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target >"$work/no-quiesce.out" 2>&1; quiesce_exit=$?
  if [[ "$quiesce_exit" != 0 || "$(tr '\n' ' ' <"$work/calls")" != "record placement local-run " ]] || grep -q 'gpu-quiesce-run.sh' "$work/no-quiesce.out"; then
    selftest_cell_failed "开了 GPU 时不该再自己清场（退 $quiesce_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」）" "卡与清场归 GPU 批处理验证服务：这里不经 scripts/crash/gpu-quiesce-run.sh 重起自己"
  fi
  # 第 17 格：排计划那一趟没打计划行，日志里是观察者交回的盘要在卡上判、取不到卡那一行（行首 CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS，
  #   接在「test <名> ... 」后面，共用库 crash_amplification_failure_kind 认成 observer-images-without-a-card）⇒ 记本次未跑、退 78，
  #   标准错误打共用库那一类的出路（crash_amplification_observer_images_without_a_card_howto）。弄坏开关 observer-images-without-a-card-as-run-failure 下照认不出原因退 247
  cells=$((cells + 1))
  local observer_exit
  { echo "running 1 test"
    echo "test crash_amplification::run_from_environment ... ${CRASH_AMPLIFICATION_NO_CARD_FOR_THE_IMAGES_AFTER_THE_OBSERVERS_LINE_PREFIX} reason=取卡等满了也没领到"
    echo "thread 'the_case' panicked at crates/singlefs-verification/src/crash_amplification.rs:1:1:"
    echo "test result: FAILED. 0 passed; 1 failed"; } >"$work/observer-images.log"
  CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY="$work/observer-images.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" \
    bash "$0" some-target >"$work/observer-images.out" 2>"$work/observer-images.err"; observer_exit=$?
  [[ "$observer_exit" == 78 ]] && grep -qF '这一条记本次未跑（退 78），不是红' "$work/observer-images.err" \
    || selftest_cell_failed "观察者交回的盘取不到卡那一行却没记本次未跑、退 78、打共用库那一类的出路（退 $observer_exit，标准错误「$(tr '\n' ' ' <"$work/observer-images.err" | cut -c1-240)」）" \
      "看 decide_where_to_run 里 pending 为空那一支的 case：observer-images-without-a-card 与 device、build 同退 78，出路由 crash_amplification_explain_exit_code 照日志分类打"
  # 第 18 格：认不出的选项（用一个从没用过的名字）⇒ 退 2、一趟都不起（调用记录是空的）、一行判定都不打
  cells=$((cells + 1))
  : > "$work/calls"
  local unknown_option_output unknown_option_exit
  unknown_option_output="$(STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-quiesce-commands.env" CRASH_AMPLIFICATION_ITEM_SWITCHES=1,0 CRASH_AMPLIFICATION_ITEM_CARDS="$local_only" GPU_CARDS_BY_PRIORITY="$local_only" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target --never-used-option 2>"$work/unknown-option.err")"; unknown_option_exit=$?
  if [[ "$unknown_option_exit" != 2 || -s "$work/calls" || -n "$unknown_option_output" ]]; then
    selftest_cell_failed "认不出的选项应当退 2、一趟都不起、一行判定都不打（退 $unknown_option_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」，标准输出「$(tr '\n' ' ' <<<"$unknown_option_output" | cut -c1-160)」）" "看参数解析那一段：-* 的分支交 usage（退 2）"
  fi
  # 第 19 格：没编进 gpu 特性的二进制遇到 ENABLE_GPU=1，排计划那一趟退 0、没有计划行、日志里只有 Rust 打的那两行（CRASH_AMPLIFICATION_NOT_RUN step=judge 与
  #   NOT_RUN_NO_GPU_ADAPTER test=crash_amplification::run_from_environment，原因写「这份二进制没有编进 gpu 特性」）⇒ 照日志分类退 78（build 那一类）：
  #   调用次序只有「record placement」（不跑本机那一趟）、不退 77。
  #   弄坏开关 accept-missing-placement 下没有计划行照本机跑（起了本机那一趟、退 0），gpu-feature-missing-as-run-failure 下这一类不认、退 247
  cells=$((cells + 1))
  local feature_missing_exit feature_missing_calls feature_missing_wrong=""
  : > "$work/calls"
  STUB_CALLS="$work/calls" STUB_PLACEMENT_WITHOUT_THE_GPU_FEATURE=1 CRASH_AMPLIFICATION_ITEM_SWITCHES=1,1 CRASH_AMPLIFICATION_ITEM_CARDS="$both" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    bash "$0" some-target >"$work/feature-missing.out" 2>&1; feature_missing_exit=$?
  feature_missing_calls="$(tr '\n' ' ' <"$work/calls")"
  if [[ "$feature_missing_exit" != 78 || "$feature_missing_calls" != "record placement " ]]; then
    feature_missing_wrong+="没编进 gpu 特性、排计划那一趟退 0 而没有计划行：退 $feature_missing_exit（要 78）、调用次序「$feature_missing_calls」（要「record placement 」）、输出尾部 $(tail -n 3 "$work/feature-missing.out" | tr '\n' '|')；"
  fi
  [[ -z "$feature_missing_wrong" ]] \
    || selftest_cell_failed "排计划那一趟退 0、没有计划行时分类不对：$feature_missing_wrong" \
      "看 decide_where_to_run 里 pending 为空那一支：没有计划行一律照 crash_amplification_failure_kind 分类（没编进 gpu 特性是 build 那一类、退 78），认不出原因才退 247"
  # 第 20 格：计划行里核对要核的状态数 verifier_pending_states：两条流各一行（核对要核 100、200）⇒ DECISION 行写 verifier_pending_states=300（各行加起来；只取第一行是 100）；
  #   判器要判与核对要核都是 0 的一趟（全部复用）也照本机跑：DECISION where=local、退 0（复用数与判红由判段那一趟报）。
  #   弄坏开关 verifier-pending-states-not-counted 下 DECISION 行恒写 verifier_pending_states=0
  cells=$((cells + 1))
  placement_line 0 0 >"$work/all-reused.log"
  grep -q ' verifier_pending_states=300 ' <<<"$two_flows" && [[ "$(decision_of "$work/all-reused.log" 1,1 "$both")" == "where=local 0" ]] \
    || selftest_cell_failed "核对要核的状态数没加起来写进 DECISION 行、或两个数都是 0 的一趟没照本机跑：$(tr '\n' '|' <<<"$two_flows" | cut -c1-200)；全部复用 $(decision_of "$work/all-reused.log" 1,1 "$both")" \
      "看 decide_where_to_run 里 verifier_pending 那两句：取 crash_amplification_placement_total 的 verifier_pending_states 合计，有计划行就本机跑"
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
  # 第 42 格：开了 GPU（显卡先后写着 127.0.0.1:0:15G）、环境里没有卡 ⇒ 不领卡：录入段、排计划、本机判三趟收到的卡都是 unset（不从多机配置挑），退 0。
  #   弄坏开关 cards-read-from-configuration 下三趟收到配置里那一张
  cells=$((cells + 1))
  : > "$work/calls"; : > "$work/calls.cards"
  local no_card_exit
  STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on-with-cards.env" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target >"$work/no-card.out" 2>&1; no_card_exit=$?
  if [[ "$no_card_exit" != 0 || "$(tr '\n' ' ' <"$work/calls")" != "record placement local-run " \
        || "$(tr '\n' ' ' <"$work/calls.cards")" != "cards=[unset] cards=[unset] cards=[unset] " ]]; then
    selftest_cell_failed "开了 GPU、环境里没有卡：三趟都不该带卡（不领、不从多机配置挑），退 0（退 $no_card_exit，调用次序「$(tr '\n' ' ' <"$work/calls")」，收到的卡「$(tr '\n' ' ' <"$work/calls.cards")」）" \
      "看 crash-amplification-lib.sh 的 crash_amplification_prepare 取卡那一段与 crash_amplification_environment_for_this_host"
  fi
  # 第 43 格：测试目标的源文件（上次红的清单的 --target-file，文件头「上次红的清单」）：平铺的取 tests/<目标>.rs，目录式的取 tests/<目标>/main.rs，两样都不在的照给 tests/<目标>.rs。
  #   弄坏开关 target-file-flat-only 下目录式目标取 tests/<目标>.rs（那份文件不在，清单悄悄不写）
  cells=$((cells + 1))
  local target_file_tests="$work/target-file-root/crates/singlefs-verification/tests" target_files_seen target_file_name
  mkdir -p "$target_file_tests/dir_target" && : >"$target_file_tests/flat_target.rs" && : >"$target_file_tests/dir_target/main.rs"
  target_files_seen="$(repository_root="$work/target-file-root"; for target_file_name in flat_target dir_target absent_target; do item_target_source_file "$target_file_name"; done | tr '\n' ' ')"
  [[ "$target_files_seen" == "$target_file_tests/flat_target.rs $target_file_tests/dir_target/main.rs $target_file_tests/absent_target.rs " ]] \
    || selftest_cell_failed "测试目标的源文件应当平铺的取 tests/<目标>.rs、目录式的取 tests/<目标>/main.rs、两样都不在的照给 tests/<目标>.rs（实际「$target_files_seen」）" \
      "看 item_target_source_file：经 lib_crash_point_replay_items.py 的 target-source-file 取；目录式目标取成 tests/<目标>.rs，那份文件不在，上次红的清单悄悄不写"
  # 第 44 格：分两次起（文件头用法 --stage）：--stage record 只跑录入段、退 0；--stage judge 带着调用方交的卡（GPU_CARDS_BY_PRIORITY）不跑录入段，
  #   排计划与本机判都用那一张；--stage 写别的值退 2、一趟都不起
  cells=$((cells + 1))
  local stage_wrong="" stage_exit
  : > "$work/calls"
  STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on-with-cards.env" \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target --stage record >"$work/stage-record.out" 2>&1; stage_exit=$?
  [[ "$stage_exit" == 0 && "$(tr '\n' ' ' <"$work/calls")" == "record " ]] && grep -q '只录（--stage record）：录入段跑完，录入 4 块' "$work/stage-record.out" \
    || stage_wrong+="--stage record 退 $stage_exit、调用次序「$(tr '\n' ' ' <"$work/calls")」；"
  : > "$work/calls"; : > "$work/calls.cards"
  STUB_CALLS="$work/calls" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on-with-cards.env" GPU_CARDS_BY_PRIORITY=127.0.0.1:1:20G \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target --stage judge >"$work/stage-judge.out" 2>&1; stage_exit=$?
  [[ "$stage_exit" == 0 && "$(tr '\n' ' ' <"$work/calls")" == "placement local-run " && "$(tr '\n' ' ' <"$work/calls.cards")" == "cards=[127.0.0.1:1:20G] cards=[127.0.0.1:1:20G] " ]] \
    || stage_wrong+="--stage judge 退 $stage_exit、调用次序「$(tr '\n' ' ' <"$work/calls")」、收到的卡「$(tr '\n' ' ' <"$work/calls.cards")」；"
  : > "$work/calls"
  STUB_CALLS="$work/calls" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target --stage both >/dev/null 2>&1; stage_exit=$?
  [[ "$stage_exit" == 2 && ! -s "$work/calls" ]] || stage_wrong+="--stage both 退 $stage_exit（要 2）、调用次序「$(tr '\n' ' ' <"$work/calls")」；"
  [[ -z "$stage_wrong" ]] || selftest_cell_failed "分两次起（--stage）不对：$stage_wrong" \
    "看参数解析的 --stage 与主流程里 item_stage 那两支：record 录完就退、judge 不录（judge 带着交下来的卡）"
  # 第 45 格：服务照会编声明给了这一次尝试的 CARGO_TARGET_DIR（文件头「编译目录」），里面有撞上限之前那一次尝试编出的产物 ⇒
  #   本机那一趟收到的编译目录就是它、那份产物照留（接着用、不重建）、跑完不删
  cells=$((cells + 1)); : > "$work/targets.log"; mkdir -p "$work/service-target/release"; echo earlier > "$work/service-target/release/earlier-attempt"
  CARGO_TARGET_DIR="$work/service-target" STUB_TARGET_LOG="$work/targets.log" CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 \
    CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/target-local.sh" bash "$0" some-target >/dev/null 2>&1; target_exit=$?
  if [[ "$target_exit" != 0 || "$(head -n 1 "$work/targets.log" 2>/dev/null)" != "$work/service-target" || ! -f "$work/service-target/release/artifact" \
        || ! -f "$work/service-target/release/earlier-attempt" ]]; then
    selftest_cell_failed "服务给了 CARGO_TARGET_DIR 时本机那一趟应当照收到它、上一次尝试的产物照留、跑完不删（退 $target_exit，收到「$(head -n 1 "$work/targets.log" 2>/dev/null)」）" \
      "看 run_build_directory_begin（设了 CARGO_TARGET_DIR 就不建、不删）与主流程里弄坏开关 service-build-directory-ignored 那一句"
  fi
  # 第 46 格：先编后判（文件头步骤 ②）：开了 GPU，判段要的测试二进制（--no-run，带 gpu 特性）在判段之前编好，排计划与本机判两趟都直接起它。
  #   假的本机那一趟记 STUB_ORDER_LOG，次序就是这一项录、编、判的次序。五种起法：
  #   两段先后跑（环境里没有卡）：record、build no-run features=gpu、placement 与 local-run（binary 是编出来的那一份）；
  #   --stage judge（环境里没有卡）：build、placement、local-run；
  #   --stage record：record、build，标准输出打一行 CRASH_AMPLIFICATION_JUDGE_BINARY 指到编出来的那一份；
  #   --stage judge --judge-binary <上一种打的路径>（环境里交了卡）：不编，placement 与 local-run 都用那一份；
  #   --judge-binary 不跟 --stage judge 给：退 2、一趟都不起
  cells=$((cells + 1))
  local order_wrong="" order_exit order_want order_binary order_record_binary
  order_run() { # <标签> [<参数>…]：开了 GPU 的多机配置，起这一份（--label judge-order-<标签>），次序落 $work/order.log，输出落 $work/order-<标签>.out，退出码放 order_exit
    local order_label="$1"
    shift
    : > "$work/order.log"; : > "$work/calls"
    STUB_CALLS="$work/calls" STUB_ORDER_LOG="$work/order.log" SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on-with-cards.env" \
      CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" bash "$0" some-target --label "judge-order-$order_label" "$@" >"$work/order-$order_label.out" 2>&1; order_exit=$?
  }
  order_run whole
  order_binary="$work/runs/judge-order-whole/judge-binary"
  order_want="record|build no-run features=gpu|placement binary=[$order_binary]|local-run binary=[$order_binary]|"
  [[ "$order_exit" == 0 && "$(tr '\n' '|' <"$work/order.log")" == "$order_want" ]] \
    || order_wrong+="两段先后跑退 $order_exit、次序「$(tr '\n' '|' <"$work/order.log")」（要「$order_want」）；"
  order_run judge --stage judge
  order_binary="$work/runs/judge-order-judge/judge-binary"
  order_want="build no-run features=gpu|placement binary=[$order_binary]|local-run binary=[$order_binary]|"
  [[ "$order_exit" == 0 && "$(tr '\n' '|' <"$work/order.log")" == "$order_want" ]] \
    || order_wrong+="--stage judge（环境里没有卡）退 $order_exit、次序「$(tr '\n' '|' <"$work/order.log")」（要「$order_want」）；"
  order_run record --stage record
  order_record_binary="$work/runs/judge-order-record/judge-binary"
  [[ "$order_exit" == 0 && "$(tr '\n' '|' <"$work/order.log")" == "record|build no-run features=gpu|" ]] \
    && grep -qxF "CRASH_AMPLIFICATION_JUDGE_BINARY test=some-target path=$order_record_binary" "$work/order-record.out" \
    || order_wrong+="--stage record 退 $order_exit、次序「$(tr '\n' '|' <"$work/order.log")」（要「record|build no-run features=gpu|」、标准输出有 CRASH_AMPLIFICATION_JUDGE_BINARY 行指到 $order_record_binary）；"
  GPU_CARDS_BY_PRIORITY=127.0.0.1:1:20G order_run given --stage judge --judge-binary "$order_record_binary"
  order_want="placement binary=[$order_record_binary]|local-run binary=[$order_record_binary]|"
  [[ "$order_exit" == 0 && "$(tr '\n' '|' <"$work/order.log")" == "$order_want" ]] \
    || order_wrong+="--stage judge --judge-binary（环境里交了卡）退 $order_exit、次序「$(tr '\n' '|' <"$work/order.log")」（要「$order_want」）；"
  order_run misplaced --judge-binary "$order_record_binary"
  [[ "$order_exit" == 2 && ! -s "$work/order.log" && ! -s "$work/calls" ]] \
    || order_wrong+="--judge-binary 不跟 --stage judge 给退 $order_exit（要 2）、次序「$(tr '\n' '|' <"$work/order.log")」（要空）；"
  [[ -z "$order_wrong" ]] || selftest_cell_failed "先编后判不对：$order_wrong" \
    "看 item_prepare_judge_binary（共用库 crash_amplification_build_judge_binary）在排计划之前、--stage record 录完接着编并打 CRASH_AMPLIFICATION_JUDGE_BINARY 行、--judge-binary 给了就不编"
  # 第 47 格：本机一台跑的那一趟日志里的 CRASH_AMPLIFICATION、GPU_CARD、GPU_TOTAL、GPU_PASS 四种行（两条流各一组，GPU_PASS 紧跟在 GPU_TOTAL 之后）⇒
  #   按 run.log 里的次序逐行原样转打；行首不在第一列的 GPU_PASS 与别的行首词不转；日志里没有 GPU_PASS 行（判器没有这种行）照常转打其余三种、退 0
  cells=$((cells + 1))
  local pass_forward_wrong="" pass_forward_exit
  printf '%s\n' \
    'CRASH_AMPLIFICATION_GPU_CARD card="card at index one" index=1 memory_budget_mebibytes=20480 blocks=3 states=30 judge_seconds=2.0 states_per_second=15' \
    'CRASH_AMPLIFICATION_GPU_TOTAL cards_used=1 blocks=3 states=30 compile_seconds=0.5 judge_seconds=2.0 states_per_second=15 seconds_storing=0.1 expand_up_to=2 exhaustive=true' \
    'CRASH_AMPLIFICATION_GPU_PASS flow=flow-alpha card="card at index one" index=1 pass=recovery submissions=2 seconds_waiting_for_the_card=1.000 seconds_of_the_longest_submission=0.600 median_states_per_submission=4 share_of_the_waits_of_the_flow=0.5000' \
    'CRASH_AMPLIFICATION_GPU_PASS flow=flow-alpha card="card at index one" index=1 pass=readback submissions=1 seconds_waiting_for_the_card=1.000 seconds_of_the_longest_submission=1.000 median_states_per_submission=8 share_of_the_waits_of_the_flow=0.5000' \
    'CRASH_AMPLIFICATION_GPU_CARD card="card at index two" index=2 memory_budget_mebibytes=8192 blocks=1 states=10 judge_seconds=1.0 states_per_second=10' \
    'CRASH_AMPLIFICATION_GPU_TOTAL cards_used=1 blocks=1 states=10 compile_seconds=0.2 judge_seconds=1.0 states_per_second=10 seconds_storing=0.1 expand_up_to=2 exhaustive=true' \
    'CRASH_AMPLIFICATION_GPU_PASS flow=flow-beta card=none index=none pass=none submissions=0 seconds_waiting_for_the_card=0.000 seconds_of_the_longest_submission=0.000 median_states_per_submission=0 share_of_the_waits_of_the_flow=0.0000' \
    'CRASH_AMPLIFICATION share=0/1 states_checked_here=40 blocks_checked_here=4 reused=0 unjudged_blocks=0 red=0' >"$work/pass-forward-expected.txt"
  # 日志 = 要转的八行，夹着不转的：测试库自己的行、行首有空格的 GPU_PASS、行首词多一个字的行
  { sed -n 1,2p "$work/pass-forward-expected.txt"; echo 'running 1 test'; sed -n 3,4p "$work/pass-forward-expected.txt"
    echo "  $(sed -n 3p "$work/pass-forward-expected.txt")"; echo 'CRASH_AMPLIFICATION_GPU_PASSES flow=not-a-pass-line'
    sed -n 5,8p "$work/pass-forward-expected.txt"; echo 'test result: ok. 1 passed'; } >"$work/pass-forward-log.txt"
  printf '%s\n' '#!/usr/bin/env bash' \
    'if [[ " $* " == *" SINGLEFS_CRASH_AMPLIFICATION_STAGE=record "* ]]; then echo "CRASH_AMPLIFICATION_RECORDED flow=stub" > "$2"; exit 0; fi' \
    "cat -- '$work/pass-forward-log.txt' > \"\$2\"" >"$work/pass-forward.sh"
  CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/pass-forward.sh" \
    bash "$0" some-target >"$work/pass-forward.out" 2>&1; pass_forward_exit=$?
  grep -E '^CRASH_AMPLIFICATION(_GPU_[A-Z]+)? ' "$work/pass-forward.out" >"$work/pass-forward.forwarded" 2>/dev/null
  [[ "$pass_forward_exit" == 0 ]] && cmp -s "$work/pass-forward.forwarded" "$work/pass-forward-expected.txt" \
    || pass_forward_wrong+="带 GPU_PASS 行的日志：应当退 0、转打的行与日志里的八行逐行相同、次序不变（退 $pass_forward_exit，转打了 $(wc -l <"$work/pass-forward.forwarded") 行，与期望差「$(diff "$work/pass-forward.forwarded" "$work/pass-forward-expected.txt" | head -n 4 | tr '\n' '|' | cut -c1-300)」）；"
  grep -v '^CRASH_AMPLIFICATION_GPU_PASS ' "$work/pass-forward-expected.txt" >"$work/pass-forward-old-expected.txt"
  { sed -n 1,2p "$work/pass-forward-old-expected.txt"; echo 'running 1 test'; sed -n 3,5p "$work/pass-forward-old-expected.txt"; } >"$work/pass-forward-log.txt"
  CRASH_AMPLIFICATION_ITEM_SWITCHES=0,0 CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/pass-forward.sh" \
    bash "$0" some-target >"$work/pass-forward-old.out" 2>&1; pass_forward_exit=$?
  grep -E '^CRASH_AMPLIFICATION(_GPU_[A-Z]+)? ' "$work/pass-forward-old.out" >"$work/pass-forward-old.forwarded" 2>/dev/null
  [[ "$pass_forward_exit" == 0 ]] && cmp -s "$work/pass-forward-old.forwarded" "$work/pass-forward-old-expected.txt" \
    || pass_forward_wrong+="没有 GPU_PASS 行的日志：应当退 0、其余五行照转（退 $pass_forward_exit，转打了 $(wc -l <"$work/pass-forward-old.forwarded") 行）；"
  [[ -z "$pass_forward_wrong" ]] || selftest_cell_failed "本机一台跑的那一趟逐趟汇总行没转进任务日志：$pass_forward_wrong" \
    "看 run_on_this_host_alone：run.log 里行首 CRASH_AMPLIFICATION、_GPU_CARD、_GPU_TOTAL、_GPU_PASS 加一个空格的行按次序原样转打"
  # 第 48 格：--stage flow-shape（文件头用法）只录流、打段形、不录观察者。开了 GPU 的多机配置、环境里没有卡，假的本机那一趟记收到的叫停文件在不在：
  #   ① 退 0；只起录入段那一趟（次序记录里只有 record，没有 build、placement、local-run）；那一趟收到了已经存在的叫停文件（stop-file=present）；
  #   标准输出有 CRASH_AMPLIFICATION_RECORDED 行与两行 CRASH_FLOW_COVERAGE 行（含 expansion=not_expanded），没有 CRASH_AMPLIFICATION_JUDGE_BINARY 行；跑完叫停文件删了；
  #   ② 假的本机那一趟不打覆盖行 ⇒ 退 1、标准错误说一行 CRASH_FLOW_COVERAGE 都没有；
  #   ③ --stage record 那一趟不给叫停文件（stop-file=none）、不打覆盖行（只有 flow-shape 转打它）。
  #   弄坏开关 flow-shape-stop-file-unsent 下①的叫停文件那一项判错、③不变；flow-shape-judge-binary-built 下①的次序与 CRASH_AMPLIFICATION_JUDGE_BINARY 那两项判错
  cells=$((cells + 1))
  local shape_wrong="" shape_exit shape_stop_file="$work/runs/flow-shape-whole/flow-shape.stop"
  : > "$work/calls"; : > "$work/order.log"; : > "$work/shape-stop.log"
  STUB_CALLS="$work/calls" STUB_ORDER_LOG="$work/order.log" STUB_STOP_FILE_LOG="$work/shape-stop.log" \
    SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on-with-cards.env" CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    bash "$0" some-target --label flow-shape-whole --stage flow-shape >"$work/shape.out" 2>"$work/shape.err"; shape_exit=$?
  [[ "$shape_exit" == 0 && "$(tr '\n' '|' <"$work/order.log")" == "record|" ]] \
    || shape_wrong+="flow-shape 退 $shape_exit、次序「$(tr '\n' '|' <"$work/order.log")」（要退 0、次序只有「record|」：不编判段二进制、不排计划、不判）；"
  [[ "$(tr '\n' ' ' <"$work/shape-stop.log")" == "stop-file=present " ]] \
    || shape_wrong+="录入段那一趟收到的叫停文件「$(tr '\n' ' ' <"$work/shape-stop.log")」（要 stop-file=present：给引擎一份已经存在的叫停文件，观察者的录入段才不录）；"
  [[ "$(grep -c '^CRASH_AMPLIFICATION_RECORDED ' "$work/shape.out")" == 1 && "$(grep -c '^CRASH_FLOW_COVERAGE ' "$work/shape.out")" == 2 ]] \
    && grep -q '^CRASH_FLOW_COVERAGE .* expansion=not_expanded$' "$work/shape.out" && ! grep -q '^CRASH_AMPLIFICATION_JUDGE_BINARY ' "$work/shape.out" \
    || shape_wrong+="标准输出要有 1 行 CRASH_AMPLIFICATION_RECORDED、2 行 CRASH_FLOW_COVERAGE（含 expansion=not_expanded）、没有 CRASH_AMPLIFICATION_JUDGE_BINARY（实际 $(tr '\n' '|' <"$work/shape.out" | cut -c1-240)）；"
  [[ ! -e "$shape_stop_file" ]] || shape_wrong+="跑完叫停文件 $shape_stop_file 还在；"
  : > "$work/calls"; : > "$work/order.log"; : > "$work/shape-stop.log"
  STUB_CALLS="$work/calls" STUB_ORDER_LOG="$work/order.log" STUB_STOP_FILE_LOG="$work/shape-stop.log" STUB_NO_COVERAGE=1 \
    SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on-with-cards.env" CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    bash "$0" some-target --label flow-shape-no-coverage --stage flow-shape >"$work/shape-no-coverage.out" 2>"$work/shape-no-coverage.err"; shape_exit=$?
  [[ "$shape_exit" == 1 ]] && grep -q '一行 CRASH_FLOW_COVERAGE 都没有' "$work/shape-no-coverage.err" \
    || shape_wrong+="假的本机那一趟不打覆盖行：要退 1、标准错误说一行 CRASH_FLOW_COVERAGE 都没有（退 $shape_exit）；"
  : > "$work/calls"; : > "$work/order.log"; : > "$work/shape-stop.log"
  STUB_CALLS="$work/calls" STUB_ORDER_LOG="$work/order.log" STUB_STOP_FILE_LOG="$work/shape-stop.log" \
    SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on-with-cards.env" CRASH_AMPLIFICATION_ITEM_LOCAL_RUN="$work/local-run.sh" \
    bash "$0" some-target --label flow-shape-record-only --stage record >"$work/shape-record.out" 2>&1; shape_exit=$?
  [[ "$shape_exit" == 0 && "$(tr '\n' ' ' <"$work/shape-stop.log")" == "stop-file=none " ]] && ! grep -q '^CRASH_FLOW_COVERAGE ' "$work/shape-record.out" \
    || shape_wrong+="--stage record 那一趟：不给叫停文件、不转打覆盖行（退 $shape_exit，叫停文件「$(tr '\n' ' ' <"$work/shape-stop.log")」）；"
  [[ -z "$shape_wrong" ]] || selftest_cell_failed "--stage flow-shape 只录流、打段形、不录观察者不对：$shape_wrong" \
    "看主流程「两段」那一段里 flow_shape_environment 与 flow-shape 那一支：给录入段一份已经存在的叫停文件、转打 CRASH_FLOW_COVERAGE 行、录完就退（不编判段二进制）"
  if (( failures > 0 )); then
    echo "  ✗ --selftest：$cells 格里 $failures 格判错" >&2
    echo "     → 怎么办：照上面每一格给的出路改，再跑 $0 --selftest" >&2
    exit 1
  fi
  echo "  ✓ --selftest：双机开着与没开都本机一台跑（不交双机驱动）、没开双机却点名第二台的卡退 78、开了双机第二台没卡退 78、几条流累加（判器要判的与核对要核的）、没有计划行认不出原因退 247、卡用不了退 78、算法判不了退 248、开了 GPU 没写卡退 78、本机那一趟卡用不了退 78、带值的选项缺值退 2、运行条件读多机配置里的显卡先后、三趟收到的卡是环境里交的那一张（不读多机配置里的显卡先后）、不再自己清场、认不出的选项退 2、没编进 gpu 特性而排计划那一趟退 0、没有计划行时退 78、核对要核的状态数合计写进 DECISION 行而两个数都是 0 的一趟也本机跑、没设 SINGLEFS_MULTI_HOST_CONFIG 时多机配置按 git common-dir 找、多机配置判法认显卡的两个键、结论版本对不上与没编进 gpu 特性都退 78、不在临时 prune-server 底下起的自证退 2、录入段没跑成就停（判段一趟都不起）、调用方没送按段登记文件时两趟都送本树那一份的绝对路径（送了照它）、照崩溃点重放用例的样子造的用例（录入段只打 CRASH_LIBRARY_RECORDED）点名了也照跑录入段并算录入成了、本机那一趟判段没跑时转打 CRASH_AMPLIFICATION_NOT_RUN step=judge 那一行、本机那一趟日志里的 CRASH_AMPLIFICATION、GPU_CARD、GPU_TOTAL、GPU_PASS 行按次序原样转打（没有 GPU_PASS 行的日志照常过）、线程数取服务交的 SINGLEFS_THREAD_CAP（没交取登记的重活默认并行度）、跑完写上次红的清单（点名用例的只换那一项）、没开 GPU 开了双机时本机 CPU 一台跑、跑完运行目录里不剩它建的编译目录（正常、红、被 TERM 停）而调用方设的 CARGO_TARGET_DIR 不删、哪几项是这一台的卡照 --emit-addresses 认（这一台是 HOSTS 里的 IPv6 那一项时它的卡照过、同一份配置在 127.0.0.1 上退 78）、没设放量家目录时运行目录在工作根下、只供测试的变量不在自证里设了退 2、显卡先后里地址不在 HOSTS 里退 78、判段跑的过程中源码变了记本次未跑退 77 不落红、不领卡（开了 GPU、环境里没有卡时三趟都不带卡）、--stage 只录与只判、服务给的 CARGO_TARGET_DIR 照用不删（上一次尝试的产物照留）、先编后判（判段要的测试二进制判段之前编好、两趟直接起它，--stage record 编好打路径，--stage judge --judge-binary 不再编）、--stage flow-shape 只录流（给引擎已经存在的叫停文件、转打 CRASH_FLOW_COVERAGE 行、不编判段二进制）、排计划那一趟观察者交回的盘取不到卡记本次未跑退 78 并打共用库那一类的出路，$cells 格都对"
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
  refused_exit_code=""
  decide_where_to_run "$CRASH_AMPLIFICATION_ITEM_DECIDE_ONLY" 0; decision=$?
  case "$decision" in
    0) exit 0 ;;
    *) exit "${refused_exit_code:-247}" ;;
  esac
fi

# 按段登记（文件头「按段登记」）：调用方没送就送本树那一份的绝对路径；弄坏开关 segments-registry-not-sent 下不送
if [[ -z "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY:-}" ]] && ! item_break_is segments-registry-not-sent; then
  export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY="$repository_root/.claude/gate.d/crash-point-replay-segments.tsv"
fi

# 上次红的清单（文件头「上次红的清单」）：从这里起每一种退法都经 EXIT 钩子写一次；只判不跑的那一支与只录的两趟（--stage record、--stage flow-shape）不写
item_write_last_run() { # <这一趟的退出码>
  item_break_is last-run-unwritten && return 0   # 弄坏开关：跑完不写
  [[ "$item_stage" == record || "$item_stage" == flow-shape ]] && return 0
  python3 "$repository_root/scripts/gate/lib_reuse_checks.py" last-run-target scripts/crash/crash-amplification-item.sh "$test_target" "$1" \
    ${test_filter:+--test-filter "$test_filter"} --target-file "$(item_target_source_file "$test_target")" >/dev/null 2>&1 \
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
# 准备不要卡：环境里有 GPU_CARDS_BY_PRIORITY 就原样交下去，没有照样往下跑（文件头步骤 ②）
crash_amplification_prepare || exit 78
if [[ -n "${CRASH_AMPLIFICATION_ITEM_SWITCHES:-}" ]]; then
  IFS=, read -r gpu_switch _ <<<"$CRASH_AMPLIFICATION_ITEM_SWITCHES"
fi
# 只供测试（文件头「只供测试」）：换掉本机那一趟。环境照真的那一趟组（crash_amplification_environment_for_this_host），只把起 cargo 换成假的
if [[ -n "${CRASH_AMPLIFICATION_ITEM_LOCAL_RUN:-}" ]]; then
  crash_amplification_run_on_this_host() {
    # 编判段要的测试二进制那一趟（共用库 crash_amplification_build_judge_binary 设 crash_amplification_this_host_build_only=1）：参数换成 cargo_build_arguments、附加环境多一项 CRASH_AMPLIFICATION_ITEM_LOCAL_BUILD=1
    local stub_cargo_arguments="${cargo_test_arguments[*]}"
    if [[ "${crash_amplification_this_host_build_only:-0}" == 1 ]]; then stub_cargo_arguments="${cargo_build_arguments[*]}"; set -- "$@" CRASH_AMPLIFICATION_ITEM_LOCAL_BUILD=1; fi
    crash_amplification_environment_for_this_host "$1" "${@:3}"
    env "${this_host_environment[@]}" CRASH_AMPLIFICATION_ITEM_LOCAL_TEST_ARGUMENTS="$stub_cargo_arguments" CRASH_AMPLIFICATION_ITEM_LOCAL_JUDGE_BINARY="${crash_amplification_judge_binary:-}" \
      bash "$CRASH_AMPLIFICATION_ITEM_LOCAL_RUN" "$@"
  }
fi
crash_amplification_home="$(crash_amplification_home_directory)" || exit 2
run_directory="$crash_amplification_home/runs/$label"
mkdir -p "$run_directory" || { echo "  ✗ 建不了 $run_directory" >&2; echo "  → 怎么办：看放量家目录 $crash_amplification_home 可不可写，或把 SINGLEFS_CRASH_AMPLIFICATION_HOME 指到可写的目录" >&2; exit 2; }
# 弄坏开关 service-build-directory-ignored：环境里给了的 CARGO_TARGET_DIR 不认，照没给时自己建、用完删（文件头「编译目录」）
if item_break_is service-build-directory-ignored; then unset CARGO_TARGET_DIR; fi
run_build_directory_begin "$run_directory" || exit 78   # 这一项的编译目录（文件头「编译目录」），EXIT 钩子 item_finish 删

# 判段要的测试二进制（文件头步骤 ②「先编后判」）：给了 --judge-binary 而它在、可执行就用它；否则在本机编一趟（只编的那一步），
#   编不成照日志分类、打 ✗ 与出路，交回该退的码（判段一趟都不起）
item_prepare_judge_binary() {
  local build_exit exit_code
  if [[ -n "$judge_binary_given" ]]; then
    if [[ -f "$judge_binary_given" && -x "$judge_binary_given" ]]; then
      crash_amplification_judge_binary="$judge_binary_given"
      echo "  · 判段直接起调用方编好的测试二进制 $judge_binary_given（--judge-binary），不再编"
      return 0
    fi
    echo "  ! --judge-binary 给的 $judge_binary_given 不在或不可执行：照没给，在本机编一趟" >&2
  fi
  if crash_amplification_build_judge_binary "$run_directory/build.log"; then return 0; else build_exit=$?; fi
  exit_code="$(crash_amplification_exit_code_of_a_failed_run "$build_exit" "$run_directory/build.log")"
  echo "  ✗ 编判段要的测试二进制（cargo ${cargo_build_arguments[*]}）退 $build_exit；日志 $run_directory/build.log；判段一趟都不起" >&2
  if exit_code_is_explained "$exit_code"; then
    crash_amplification_explain_exit_code "$exit_code"
  else
    echo "  → 怎么办：看日志里 error 那几行，修好再重跑同一条命令；录下的照录入键复用" >&2
  fi
  return "$exit_code"
}

# 两段：先在本机跑录入段，把这一项的流程写进判定库（prune-server）；之后排计划、本机判每一趟都只跑判段（从服务里读）。
#   --stage judge（调用方已录过）不跑录入段；--stage record、--stage flow-shape 录完就退（文件头用法）
if [[ "$item_stage" != judge ]]; then
  # --stage flow-shape：给引擎一份已经存在的叫停文件，引擎录完流、打完录入与覆盖两种行之后，观察者的录入段在头一个要录的块之前就见到叫停（文件头用法）；
  #   叫停文件跑完就删。弄坏开关 flow-shape-stop-file-unsent：不给
  flow_shape_environment=()
  flow_shape_stop_file="$run_directory/flow-shape.stop"
  if [[ "$item_stage" == flow-shape ]] && ! item_break_is flow-shape-stop-file-unsent; then
    : > "$flow_shape_stop_file" || { echo "  ✗ 建不了叫停文件 $flow_shape_stop_file" >&2; echo "  → 怎么办：看放量家目录 $crash_amplification_home 可不可写" >&2; exit 2; }
    flow_shape_environment=("SINGLEFS_CRASH_AMPLIFICATION_STOP_FILE=$flow_shape_stop_file")
  fi
  crash_amplification_record_on_this_host "$run_directory/record.log" 0/1 ${flow_shape_environment[@]+"${flow_shape_environment[@]}"}; record_exit=$?
  [[ "$item_stage" != flow-shape ]] || rm -f -- "$flow_shape_stop_file"
  if (( record_exit != 0 )) && ! item_break_is record-failure-ignored; then exit "$record_exit"; fi
  if [[ "$item_stage" == flow-shape ]]; then
    # 要的是段形：录入行（crash_amplification_record_on_this_host 已转打）加日志里的 CRASH_FLOW_COVERAGE 行；覆盖行一行都没有就没有段形可看，退 1
    if ! grep -E '^CRASH_FLOW_COVERAGE ' < <(crash_amplification_lines_without_test_prefix "$run_directory/record.log"); then
      echo "  ✗ 录入段退 0，日志 $run_directory/record.log 里却一行 CRASH_FLOW_COVERAGE 都没有（引擎没跑到打覆盖行那一步，或用例没经 run_from_environment 起）" >&2
      echo "  → 怎么办：看那份日志跑的是不是这一项的用例；要整条流的段形而这条用例不打覆盖行的，用 --stage record" >&2
      exit 1
    fi
    echo "  · 只录流（--stage flow-shape）：流录进判定库，录入 $(crash_amplification_recorded_blocks "$run_directory/record.log") 块；观察者没录，不编判段二进制、不排计划、不判"
    # 弄坏开关 flow-shape-judge-binary-built：开了 GPU 时也接着编判段要的测试二进制，打出路径
    if (( gpu_switch )) && item_break_is flow-shape-judge-binary-built; then
      item_prepare_judge_binary || exit $?
      [[ -z "$crash_amplification_judge_binary" ]] || echo "CRASH_AMPLIFICATION_JUDGE_BINARY test=$test_target path=$crash_amplification_judge_binary"
    fi
    exit "$record_exit"
  fi
  if [[ "$item_stage" == record ]]; then
    echo "  · 只录（--stage record）：录入段跑完，录入 $(crash_amplification_recorded_blocks "$run_directory/record.log") 块；排计划与判段留给 --stage judge 那一趟"
    # 开了 GPU：接着编判段要的测试二进制（文件头步骤 ②「先编后判」），打出路径交给调用方（调用方经 --judge-binary 交回给 --stage judge 那一趟）
    if (( gpu_switch )); then
      item_prepare_judge_binary || exit $?
      [[ -z "$crash_amplification_judge_binary" ]] || echo "CRASH_AMPLIFICATION_JUDGE_BINARY test=$test_target path=$crash_amplification_judge_binary"
    fi
    exit "$record_exit"
  fi
fi
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

# 先编后判（文件头步骤 ②）：开了 GPU，判段要的测试二进制在判段之前编好，排计划与本机判两趟直接起它；不领卡（判段写待判行、等 GPU 批处理服务判）
if (( gpu_switch )); then item_prepare_judge_binary || exit $?; fi

run_on_this_host_alone() { # 本机一台跑整份，打它的汇总行；退 0，或照 run.log 分类之后该退的码
  item_exit_if_source_changed "本机那一趟起跑前"
  crash_amplification_run_on_this_host 0/1 "$run_directory/run.log"; local run_exit=$?
  item_exit_if_source_changed "本机那一趟收尾后"
  # 转打的四种行（文件头「输出」）；弄坏开关 gpu-pass-lines-dropped 下少 CRASH_AMPLIFICATION_GPU_PASS 那一种
  local forwarded_lines='^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL|_GPU_PASS)? '
  item_break_is gpu-pass-lines-dropped && forwarded_lines='^CRASH_AMPLIFICATION(_GPU_CARD|_GPU_TOTAL)? '
  grep -hE "$forwarded_lines" "$run_directory/run.log" 2>/dev/null
  # 判段没跑（文件头「输出」：没编进 gpu 特性的二进制遇到 ENABLE_GPU=1）那一趟打的本次未跑行原样转打；弄坏开关 judge-not-run-dropped 下不转
  item_break_is judge-not-run-dropped || grep -h '^CRASH_AMPLIFICATION_NOT_RUN step=judge ' "$run_directory/run.log" 2>/dev/null
  if [[ "$run_exit" == 0 ]]; then return 0; fi
  local exit_code
  exit_code="$(crash_amplification_exit_code_of_a_failed_run "$run_exit" "$run_directory/run.log")"
  echo "  ✗ 本机那一趟退 $run_exit；日志 $run_directory/run.log" >&2
  grep -h -A 3 'panicked at' "$run_directory/run.log" 2>/dev/null | head -8 | sed 's/^/    /' >&2
  if exit_code_is_explained "$exit_code"; then
    crash_amplification_explain_exit_code "$exit_code" "$run_directory/run.log"
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
refused_exit_code=""
decide_where_to_run "$run_directory/placement.log" "$placement_exit"; decision=$?
case "$decision" in
  0) run_on_this_host_alone; exit $? ;;
  *) exit "${refused_exit_code:-247}" ;;
esac
