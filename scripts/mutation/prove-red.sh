#!/usr/bin/env bash
# admission: always 实现员每写一条新测试都要现证它会红，判的是这一刻副本里的源码与变异行；能不能复用由每一行的行键对判定库逐条判（文件头「判定库与复用」），不整次跳过
# run-condition: command python3 cargo rustc git
# selftest-serial: selftest 在一个工作目录里摘掉本进程环境里的 CARGO_TARGET_DIR（要共用编译目录的格自己设、自己换回）、把库根与假主工作区登记进模块级的 LIBRARY_ROOT_FOR_SELFTEST、CLOSURE_ROOT_FOR_SELFTEST，各格共用同一份假 cargo、同一个库根与同一份判定库，判定库那几格接着前一格留下的库判复用与记红撤绿，格与格之间有先后，次序本身是被测对象的一部分
# 编译族（task-server/task_server_lib.py 文件头「资源记录」的编译族）：编的包取第三个参数 <crate>（$3），变异名每条命令都不同、不进族；
#   这里起的是 cargo test <变异行的参数>，命中重测试目标（登记的崩溃枚举用例目标、gpu 特性）的行一律补 --release 起（见「判法与次序」的「重测试目标」一条）。
#   命中与否随点名的变异行变，声明只能写一份：声明 release 那一档、不带特性——重的那几行（编得多、撞上限的多是它们）在 release 档，按 release 族起步；
#   不命中的 dev 档那几行编得少，照 release 族起步只多给、不少给（带 --features gpu 的那几行编得多些，撞了由服务放大）。
#   算行键（文件头「判定库与复用」）时副本里有 verus! 块的文件要产品行为视图，scripts/gate/lib_function_units.py 的 built_unit_graph 照它的缓存取 unit-graph、
#   缓存里没有这一版才编一次（release 档的 singlefs-unit-graph，与门禁 crates-mutation-replay 共用那一份缓存）：那一次多编一个包，族照旧按 -p <crate> 起步，撞了由服务放大。
#   变异那几趟各起的临时 prune-server（文件头「判定库与复用」）：环境没给 PRUNE_SERVER_BINARY、又有要现判的行时，开跑多编一次 prune-server（release，进这一趟的临时目录，收尾删），族照旧，撞了由服务放大。
# 实现员的证红：在仓副本里逐条施加 crates/mutations.tsv 里点名的变异，跑那一行指定的测试目标，看点名的测试红没红，再还原。
# 不套内存包装、不收 --memory：这一整条经 task-server/task submit -- bash scripts/mutation/prove-red.sh … 交，上限由服务按记录给、撞了放大重排（收尾批 M03）。
#
#   prove-red.sh --copy <仓副本> <crate> <变异名…>
#   prove-red.sh --selftest            # 用假的 cargo 走一遍；PROVE_RED_BREAK=<项> 时必须判红。读写判定库，只在临时 prune-server 底下跑：
#                                      #   python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/mutation/prove-red.sh --selftest
#                                      #   （开跑先调客户端 selftest_service_problem，环境里没有临时服务的标记退 2、什么都不判）
#
# 判法只有一份，与门禁 crates-mutation-replay 同用（这里不另写）：
#   切表与成形照 scripts/mutation/crates-mutation-rows.py 的 read_table（malformed_reason）；一条跑完判哪一档照它的 verdict_of_run；
#   libtest 的结果行在那里经 scripts/gate/lib_libtest_result_lines.py 解析。点名的行一律照跑：谁起、什么时候起由项目代码闸与统一任务服务判，这里不判、不跳。
#
# 判定库与复用（与门禁 crates-mutation-replay 判的是同一件事：这一行改坏后点名的测试红不红。两边共用同一把行键、同一份判定，不另起种类；
#   行键的逐样、库的存放与复用判据在 scripts/mutation/crates-mutation-rows.py 文件头「行键」「判定库」「复用判据」，这里只调它）：
#   行键：点名的每一行照 crates-mutation-rows.py 的 KeyContext 在 --copy 那棵树上算（验证包只点名集成测试目标的行照补了 --no-default-features 之后的参数算，命中重测试目标的行照补了 --release 之后的参数算，开 GPU 补了 gpu 特性的行照补了之后的参数算），
#     表里没点名、而原文与替换文都与点名的某一行相同的行一起算，撞键照 refuse_colliding_row_keys 拒算（与门禁选行时同一取法）。
#   库：prune-server 里 mutation-units 族那一份 singlefs-crates-mutation-library（open_library）。库根取这份脚本所在的主工作区（它的 git common-dir）：
#     --copy 那棵副本是 minimal-copy.py 在副本根 git init 的工作树，库名固定、库根不进键，门禁在主工作区判的与这里判的落在同一份；自证换成自己 git init 的一棵临时仓（临时服务底下另占一份命名空间）。
#   复用：照 decide_rows（row_decision），库里这把行键记着「抓到」的复用：不起 cargo、不跑基线，那一行打「<名>\t抓到\t<测试> 红了（键相同（…））（复用）」。
#   现判：键对不上、上次判的不是抓到、复用哨兵抽中强制现判的照跑，判完照行键写 records/（字段与 crates-mutation-rows.py 的 judge_row 同形）：
#     抓到、没红、无效、本次未跑照 verdict_of_run 的档写；没红、无效盖掉同一把键原有的抓到（记红撤绿）；本次未跑不盖原有的算数结论
#     （uncounted_verdict_keeps_counted_record）；基线红的那几行照行键记 baseline-not-ok；「没重编」与施加那一刻原文命中不是一次的不写（判定不算数）。
#     组成清单照 KeyContext.persist 写、复用与现判出结局的行照 index_entry 写索引：门禁下一趟键对不上时照它报变了哪几个单元。
#   拒算（行键算不出、撞键）：照跑、不复用、不写库，另打一行「- 拒算 <名>：行键算不出（…）：<为什么>」。
#   起 cargo 的环境照门禁关调试信息（crates-mutation-rows.py 的 cargo_debug_overrides）：构建环境、构建档与线程数不进行键，这里只定起的 cargo 与门禁起的一样。
#   基线那几次例外：不带 CARGO_PROFILE_RELEASE_INCREMENTAL 这一项覆盖（baseline_environment，调用方自己设了的值照放回、没设就不放；与门禁的基线同用 crates-mutation-rows.py 的
#     baseline_cargo_environment，那份文件头「基线连调用方的判定库服务」写了为什么）：release 档的崩溃判定版本因此与门禁 crash-point-replay 的 cargo test --release 同一把。
#     基线不起临时 prune-server，连哪一处照调用方的环境（SINGLEFS_PRUNE_SERVER_SOCKET 等）与多机配置：基线跑的是没改坏的源码，判出的崩溃结论本来就该进那一份库。
#   变异那几趟例外：改坏之后起的每一次 cargo（judge_row 里的 run_cargo）各起一份自己的临时 prune-server（mutated_run_prune_server 调 crates-mutation-rows.py 的 mutation_run_prune_server，
#     与门禁的变异复跑同一份起法），那一次环境里连哪一处（SINGLEFS_PRUNE_SERVER_SOCKET、SINGLEFS_PRUNE_SERVER_TCP_ADDRESS）与临时服务的标记都指它，跑完停服务、删目录：
#     走判定库的 验证包用例在改坏的源码上判出的崩溃结论只落进那一份临时库，不读真库、不写真库。改坏的若是崩溃枚举的流水线文件（录写表、排计划、领块、存库、对账那几份，不进判器的键），
#     键与没改坏时相同，连真库会把改坏版的结论写在同一把键下。服务二进制取 PRUNE_SERVER_BINARY（绝对路径、可执行，没设就照这份脚本所在仓的 prune-server/ 编一份进这一趟的临时目录，
#     prepare_mutated_run_service），有要现判的行才取，取不到整次拒绝、退 2、测试的 cargo 一次都不起；复用的行不起服务。服务根（服务的目录与编出来的二进制）每一趟新建在这一台自己的临时目录下，
#     收尾删、被 TERM / HUP 停时也删；被 SIGKILL 杀掉时服务随起它的进程停（包装设的父进程死亡信号），服务根留在临时目录里，名字每趟是新的，归临时目录的属主收。
#   驱动（这份脚本与 crates-mutation-rows.py 各份）与算键库的代码不进行键：只改这里施加与起 cargo 的写法，库里的判定不作废。
#   复用哨兵：每一趟开跑前抽（scripts/gate/lib_reuse_checks.py 文件头「二」；候选是此刻行键在 records/ 里记着抓到的行，项名照门禁的 sentinel_item 是「<行号> <变异名>」）；
#     跑完照 observe_fresh_outcomes 交结局：哨兵被现判、强制现判那一条判得与库里不同，整次退 1；强制现判那一条判成本次未跑的不交哨兵。
#   每条判定那一行末尾标（现判：<栏>）（复用）（强制现判）（拒算），栏照门禁的 COLUMN_TITLES；成功句与失败句都报「现判 N 条、复用 M 条、拒算 R 条」。
#   算不出行键或库的位置，点名测试的闭包算不成：整次拒绝、退 2、cargo 一次都不起；服务连不上照客户端退 75，不当「库里没有」现判。
#   缩过域的一趟（环境里设了崩溃域的缩域变量；认法与门禁同一个函数 crates-mutation-rows.py 的 crash_domain_narrowing，那份文件头「缩过域的一趟」写了为什么这样办）：
#     现判出的判定、基线红一条都不写进判定库，索引也不写；复用照库里全域下判的抓到；不抽复用哨兵（打一行原因）；
#     现判的每一行在末尾的标之前接「；缩过域（<变量=值>）、不算门禁通过、没入库」，汇总句接「；缩过域（<变量=值>）、不算门禁通过、现判的没入库（复用的是库里全域下判的）」；退出码不变。
#
# 不领卡（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：卡与清场归 GPU 批处理验证服务，这一份不领卡、不挑卡、不清场、不藏卡。
#   起 cargo 的只有 run_cargo 一处：带不带 gpu 特性都照原样起，环境里有 GPU_CARDS_BY_PRIORITY 就原样交，没有不设；不另编。
#   走崩溃判段的测试由验证包写待判行、等 GPU 服务判；测试自己要开卡的（GPU 对拍、卡池测试）开不了卡时打 NOT_RUN_NO_GPU_ADAPTER 或 GPU_DEVICE_LOST，
#   照 verdict_of_run 的判法记本次未跑（不写判定库的算数结论、不算失败）。汇总只有本次未跑、一条都没抓到时退 77。
#   要卡的行（特性带 gpu）全在同一路（planned_lanes 的「要卡」组）、一行一行串行跑：同一个测试二进制之外不同时上卡。
#   被停（TERM、HUP）：处理函数（remove_trip_build_directories_when_stopped）删这一趟自己建的编译目录，再按默认处理重发同一个信号。
#
# 弄坏开关 PROVE_RED_BREAK（只给 --selftest 证明它会红用）。一项一行，下一行起写打开后判错的自证项：
#   any-target  放过不挑目标的参数
#       「不挑目标的变异行应当整次拒绝、退 2」
#   main-allowed  放过主工作区
#       「--copy 指主工作区应当以「主工作区」为由拒绝」（那一格的主工作区是临时目录里一棵假的、经 prove 的 main_root 顶替，
#       自证从哪棵树起都判得出；打开开关时假 cargo 在那棵假树上照跑，不再以主工作区为由退 2）
#   own-table  回到自己切表（六段就收、不看哪一段空着），不经 read_table
#       「点名的行不成形（必须红的测试名空着）：整次拒绝、写明哪一段空着」「切表与 read_table 逐行相同」
#   own-judge  回到自己判结局（任一行点名的 FAILED 就算抓到，不看退出码、不看对上几个用例、不认 NOT_RUN、不认 GPU_DEVICE_LOST），不经 verdict_of_run
#       「判法与 verdict_of_run 对同一组样本逐行相同」「并行判定与串行逐条相同」（p-notrun 那一行两边都判成没红，对不上写明的结局）
#   baseline-ignored  不看基线红没红
#       「基线（没改源码）就红：停下、退 2、一条变异都不记抓到」「并行基线」
#   never-caught  一条都不记抓到
#       「会红的变异应当抓到、退 0」等要抓到的那几格（编译目录那两格、副本锁那一格与跨机那一格过期接手的那一半也要抓到才退 0；
#       判定库那四格 reuse-three-questions、复用哨兵、记红撤绿、键相同的都复用同样要抓到；并行那几格里「并行判定与串行逐条相同」「并行单路红不吃掉」同样要抓到）
#   error-line-invalid  回到「有一行 error: 就算编不过」（cargo 测试红了也打 error: test failed，点名的没红会被判成无效）
#       「编过了、别的测试红而点名的没红……应当判没红、不判无效」
#   no-copy-lock  不在副本根上拿副本锁（见「判法与次序」⓪），锁被占着也照跑
#       「证红跑的时候副本根上拿着副本锁、跑完放掉；锁被占着时拒绝、退 2、cargo 一次都不起」
#       「别的机器上的证红占着副本时拒绝、过期的接手」（不拿锁就不读持有者记录，照跑、退 0，跑完记录还在）
#   shared-log-directory  日志不分每趟的目录，同一份副本上的几趟共用 <日志目录>/baseline.log 与 NNN.log
#       「同一份副本上先后起两趟，每趟读的是自己的基线日志」
#   no-rebuild-check  施加变异之后那一趟编译不核被改的包有没有 Compiling 行（见「判法与次序」⑤）
#       「编译目录里的产物比施加变异后的源码新（共用编译目录）：那一条判没重编、退 1，不判没红」（实际判成没红）；「并行判定与串行逐条相同」（p-stale 那一行）；
#       scripts/gate/lib_rebuilt_packages.py 的 REBUILT_PACKAGES_BREAK=any-compiling-line 不让这一格红（那一趟一行 Compiling 都没有）
#   path-included-package-ignored  被改文件经 #[path]（或 include_str!）编进别的包时，不认那个包的 Compiling 行，只认被改文件自己所在的包
#       「被改文件经 #[path] 编进点名的包（dep-info 列着它）：有点名的那个包的 Compiling 行就判重编过」
#   rocksdb-entry-unchecked  起 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry
#       「副本里的 验证包依赖 rocksdb 而进程环境里没有预编库的取值：应当在起 cargo 之前整次拒绝、退 2、假 cargo 一次都不起」
#       （entry 判的是 --copy 那棵树、--cargo 是要起的那一份 cargo；树里没有依赖 rocksdb 的清单、cargo 是带「# rocksdb-prebuilt-entry: selftest-fake-cargo」那一行的假 cargo 时不判）
#   no-restore-journal  施加变异前不落还原日志、起来也不看它（见「判法与次序」的「还原日志」一条）：被杀之后改坏的源码留在副本里
#       「被杀之后同一份副本上下一趟起来：先照还原日志把源码写回、报写回的那一行、基线照常跑、还原日志删掉」
#   old-text-checked-late  原文命中次数只在施加那一刻核（见「判法与次序」的「先核原文」一条）：留着替换文的副本先跑基线、报成基线红
#       「副本里留着一条替换文而没有还原日志：基线之前以原文命中 0 次整次拒绝、退 2、cargo 一次都不起」
#   copy-closure-unchecked  不按 lib_repository_copy.py 的闭包核副本缺不缺（见「判法与次序」①）：缺 litmus/ 的副本照跑，到 cargo 编不过、测试读不到才知道
#       「副本缺闭包里的条目（litmus/never.litmus）：整次拒绝、退 2、点名它、出路指到 minimal-copy.py、cargo 一次都不起」
#   should panic 那一段的解析在 lib_libtest_result_lines.py：LIBTEST_RESULT_LINES_BREAK=should-panic-in-name 时
#       「#[should_panic] 的用例红了……应当判抓到、退 0」判错
# 编译目录那几格的开关（开关一行，下一行起是打开后判错的自证项）：
#   PROVE_RED_BREAK=unbound-second-argument  开头拿 $2 拼 CARGO_TARGET_DIR 的默认值（set -u 下只给 --selftest 一个参数时当场退 1）；
#     只在自证起的那一层子进程（PROVE_RED_SELFTEST_PROLOGUE 设着）里生效，外层照常跑完才报得出是哪一格
#       「✗ 自检：不设 CARGO_TARGET_DIR、只给 --selftest：开头那几行」（子进程退 1，输出 $2: unbound variable）
#       连带红的格：「✗ 自检：不设 CARGO_BUILD_JOBS：开头那几行」、「✗ 自检：调用方设了 CARGO_BUILD_JOBS：开头那几行」
#   PROVE_RED_BREAK=caller-target-overridden  调用方设了的 CARGO_TARGET_DIR 也换成 <副本>/target
#       「✗ 自检：调用方设了 CARGO_TARGET_DIR」；连带「编译目录里的产物比施加变异后的源码新（共用编译目录）……」
#       （那一格的共用编译目录就是调用方设的 CARGO_TARGET_DIR，被换成副本自己的 target 之后照常重编、判抓到、退 0）
#   PROVE_RED_BREAK=build-jobs-six  没设 CARGO_BUILD_JOBS 时写死 6，不取登记的重活默认并行度；同样只在自证起的那层子进程里生效
#       「✗ 自检：不设 CARGO_BUILD_JOBS：开头那几行」（子进程打 CARGO_BUILD_JOBS=6）
#   PROVE_RED_BREAK=inherited-target-kept  自证各格照用调用方给的 CARGO_TARGET_DIR：第二台上服务给每趟一份编译目录，几份假副本的假产物落进同一个目录、
#     按修改时刻互相顶替，litmus 那两格之后 copy 那份副本的源码比顶上来的改坏产物旧、基线就红）；调用方没设时开关替它设一份共用的（演第二台），两台红的格相同
#       「✗ 自检：命中重测试目标的行（gpu 特性、crash-case 目标）起的每一次 cargo 都带 --release」
#       「✗ 自检：证红跑的时候副本根上拿着副本锁、跑完放掉」
#       「✗ 自检：别的机器上的证红占着副本时拒绝、过期的接手」
#       「✗ 自检：同一份副本上先后起两趟，每趟读的是自己的基线日志」
# 编译目录归这一趟那三格的开关（文件头「判法与次序」的「编译目录归这一趟」；开关一行，下一行起是打开后判错的自证项）：
#   PROVE_RED_BREAK=copy-target-reused  没给 CARGO_TARGET_DIR 时不另建，沿用 <副本>/target（副本里留着的 target 里 demo 的产物比源码新、被沿用，那一趟判没重编、退 1；沿用的 <副本>/target 也不删）
#       「✗ 自检：编译目录归这一趟·每一趟新建：」「✗ 自检：编译目录归这一趟·跑完删：」
#   PROVE_RED_BREAK=trip-target-kept  这一趟自己建的编译目录收尾不删
#       「✗ 自检：编译目录归这一趟·跑完删：」
#   PROVE_RED_BREAK=trip-target-kept-on-stop  被 TERM、HUP 停时不删这一趟自己建的编译目录
#       「✗ 自检：编译目录归这一趟·被打断删：」
# 基线那几次起 cargo 的环境那一格的开关（baseline_environment；开关一行，下一行起是打开后判错的自证项）：
#   PROVE_RED_BREAK=baseline-incremental-kept  基线也带门禁加的 CARGO_PROFILE_RELEASE_INCREMENTAL（崩溃判定的版本与 crash-point-replay 的不是同一把）
#       「✗ 自检：基线起的 cargo 不带门禁加的 release 增量覆盖：」
# 变异那几趟各起自己的临时 prune-server 那一格的开关（mutated_run_prune_server；开关一行，下一行起是打开后判错的自证项）：
#   PROVE_RED_BREAK=mutated-run-on-real-service  变异那几趟不起临时服务，照调用方的环境连那一份服务（改坏版判出的崩溃结论写进调用方的库）
#       「✗ 自检：变异那一趟连的不是调用方的服务：」
# 链接失败那一格的开关（verdict_of_run 的 linkfailed；开关一行，下一行起是打开后判错的自证项）：
#   PROVE_RED_BREAK=link-failure-listed-as-invalid  替换之后失败在链接的那一行也打成「无效」、不打「链接失败」与出路
#       「✗ 自检：链接失败：」
#   判法那一份 crates-mutation-rows.py 的开关 link-failure-as-invalid、link-failure-not-red 由它自己的自证证红。
# 重测试目标那一格的开关（开关一行，下一行起是打开后判错的自证项）：
#   PROVE_RED_BREAK=release-unforced  命中重测试目标而 cargo 参数没带 release 的行不补 --release：照变异行原样在 debug 档跑
#       「✗ 自检：命中重测试目标的行（gpu 特性、crash-case 目标）起的每一次 cargo 都带 --release、只带一个，不命中的不带，输出点名补了哪两行」
# 验证包的测试目标不编装置二进制那一格的开关（文件头「判法与次序」的「验证包的测试目标不编装置二进制」一条；开关一行，下一行起是打开后判错的自证项）：
#   PROVE_RED_BREAK=no-default-features-unadded  不调 default_features_off_row：验证包只点名集成测试目标的行照原样参数起（装置二进制照编）
#       「✗ 自检：验证包只点名集成测试目标的行：」
# 开 GPU 补 gpu 特性那一格的开关（文件头「判法与次序」的「开 GPU 补 gpu 特性」一条；开关一行，下一行起是打开后判错的自证项）：
#   PROVE_RED_BREAK=gpu-route-feature-unadded  不调 gpu_route_rows：开了 GPU 而行里没写特性的照原样参数起
#       「✗ 自检：开 GPU 补 gpu 特性：」
# 判定库那几格的开关（文件头「判定库与复用」；开关一行，下一行起是打开后判错的自证项）：
#   PROVE_RED_BREAK=library-unread  不读判定库，键算得出的行一律现判（复用哨兵照旧直接读记录抽，抽中的哨兵被现判）
#       「✗ 自检：reuse-three-questions：」「✗ 自检：复用哨兵：」「✗ 自检：记红撤绿：」「✗ 自检：键相同的都复用：」
#       连带红的格：「✗ 自检：缩过域·复用全域判定：」（那一格要两条都复用、假 cargo 一次都不起，不读库就全现判）
#   PROVE_RED_BREAK=sentinel-unplanned  不抽复用哨兵、不强制现判：一趟真跑里没有哨兵
#       「✗ 自检：复用哨兵：」「✗ 自检：记红撤绿：」
#       连带红的格：「✗ 自检：缩过域·复用全域判定：」（那一格要打一行写明缩过域的不抽哨兵原因，这个开关下那一行写的是弄坏开关 sentinel-unplanned 的原因）
#   PROVE_RED_BREAK=red-not-recorded  只记抓到（没红、无效、本次未跑、基线红都不写，同一把键原有的抓到留着照复用）
#       「✗ 自检：记红撤绿：」
#   PROVE_RED_BREAK=narrowed-recorded  缩过域的一趟照常写判定库与索引（文件头「判定库与复用」的「缩过域的一趟」）
#       「✗ 自检：缩过域·现判不入库：」
#   PROVE_RED_BREAK=narrowed-sentinel-drawn  缩过域的一趟照常抽复用哨兵、强制现判一条
#       「✗ 自检：缩过域·复用全域判定：」
# 并行那几格的开关（文件头「判法与次序」的「并行」一条；开关一行，下一行起是打开后判错的自证项）：
#   PROVE_RED_BREAK=parallel-one-at-a-time  各路一路跑完才起下一路（判定不变，只是不并行）
#       「✗ 自检：并行同时起：」
#   PROVE_RED_BREAK=parallel-lane-red-swallowed  头一路之外的没红、无效、没重编不算进退出码（那几行照打，整次照退 0）
#       「✗ 自检：并行判定与串行逐条相同：」「✗ 自检：并行单路红不吃掉：」
#   PROVE_RED_BREAK=parallel-count-unchecked  不核派出几路、收回几路
#       「✗ 自检：并行收回数核对：」
#   PROVE_RED_BREAK=parallel-output-finish-order  判定行按跑完的先后打，不按点名次序
#       「✗ 自检：并行判定与串行逐条相同：」「✗ 自检：并行输出按点名次序：」
#   PROVE_RED_BREAK=parallel-shared-target  各路共用一份编译目录
#       「✗ 自检：并行每路各一份工作副本与编译目录：」
#   PROVE_RED_BREAK=parallel-jobs-undivided  每路的 CARGO_BUILD_JOBS 照总数给，不除以路数
#       「✗ 自检：并行编译并行度：」
#   PROVE_RED_BREAK=parallel-lanes-kept  收尾不删各路的目录
#       「✗ 自检：并行收尾：」
#   PROVE_RED_BREAK=parallel-same-file-split  改同一个文件的行也分到不同的路
#       「✗ 自检：并行撞文件的行同一路：」
#   PROVE_RED_BREAK=parallel-gpu-rows-split  要卡的行不放进同一路（各路的对拍同时抢一张卡）
#       「✗ 自检：并行分路：」
#   PROVE_RED_BREAK=parallel-lanes-unchecked  PROVE_RED_LANES 写错不拒，当 1 路跑
#       「✗ 自检：并行路数写错：」
# 不领卡那一格的开关（文件头「不领卡」；自证用一个包着假 cargo 的假 cargo 记每次调用的参数与环境里的卡串）：
#   PROVE_RED_BREAK=gpu-rows-compiled-apart  带 gpu 特性的行先另起一条只编的 cargo test --no-run、再起真跑测试的那一条
#       「✗ 自检：不领卡：」
#       连带红的格（开关弄坏的是每一行都走的 run_cargo，别的格里带 gpu 特性的行起 cargo 的次数同样变了）：
#       「✗ 自检：命中重测试目标的行（gpu 特性、crash-case 目标）起的每一次 cargo 都带 --release」（那一格要命中的两行一共起 4 次 cargo，
#       带 gpu 特性那一行的基线、变异前各多一次 --no-run，成了 6 次）、
#       「✗ 自检：开 GPU 补 gpu 特性：」（那一格的假 cargo 不认 --no-run、照跑全部测试：变异那一趟只编的那一次就红了、真跑的那一次不再起，
#       route-41 点名测试的 cargo 只剩基线一次，那一格要基线、变异各一次）
#   PROVE_RED_BREAK=gpu-rows-cards-dropped  带 gpu 特性的行起 cargo 时去掉环境里的 GPU_CARDS_BY_PRIORITY
#       「✗ 自检：不领卡：」
#
# 判法与次序：
#   ① --copy 必须是一份副本：目录在、里面有 crates/mutations.tsv、不是这份脚本所在的主工作区（在主工作区上改坏源码会波及别的会话）；
#      还要拷全：按 scripts/hosts/lib_repository_copy.py 的同一份闭包（主工作区那棵树、带测试输入；entries_missing_in_copy）核副本根下缺不缺闭包里的条目，
#      缺的整次拒绝、退 2、逐条点名缺哪几条与它们为什么在闭包里，出路指到 minimal-copy.py（另建一份，不手补），一条都不跑；闭包算不成同样整次拒绝（copy_closure_refusal）；
#   ⓪ 过了 ① 就在副本根上拿副本锁 <副本>/.prove-red.lock（排他 flock、不等），一直拿到这一次返回；拿不到（别的证红在跑）整次拒绝、退 2、一条都不跑。
#      scripts/gate/apply-writer-patch.py --root <副本> 开头试拿同一把锁，拿不到就不打补丁（证红中的副本不许被改）；
#      锁的写法只在 apply-writer-patch.py 的 hold_copy_lock / release_copy_lock（这里按路径载入它）。flock 跨机不保证互斥，所以那里另在 <副本>/.prove-red.owner 上排他写持有者记录
#      （带统一任务服务给的任务号与尝试号 TASK_SERVER_TASK、TASK_SERVER_ATTEMPT；别的机器上的证红 300 秒内刷新过记录就拒绝，退 2、一条都不跑，拒绝那一句写出记录里的任务号与尝试号；
#      记录是这一趟所在任务更早的尝试写的（服务撞上限换台重排，后一次撞上前一次留下的记录），当场接手照跑，不等过期；
#      写法在 apply-writer-patch.py 文件头「副本锁」的跨机一段）；两趟即使撞上同一份副本，日志也各写各的目录，不共用同一个 baseline.log。正常收尾删掉锁文件；被打断时内核随进程放锁、文件留着不碍下一次拿；
#   ② 逐个找变异名对应的行（read_table 读成形的行；同名的取表里最后一行）；找不到、那一行不成形、参数里没有 -p <crate>、
#      没有 --lib / --test / --bin 之一的，整次拒绝（一条都不跑，退 2）——不挑目标的 cargo test 会把这个包的全部测试目标都编、都跑；
#   验证包的测试目标不编装置二进制：② 之后、重测试目标之前，点名的每一行照 scripts/mutation/crates-mutation-rows.py 的 default_features_off_row 换一遍（命中条件与补法只在那个函数的文档串与那份文件头
#      「验证包的测试目标不编装置二进制」，这里不另写）：命中的行这一次的 cargo 参数补 --no-default-features，基线与变异都照补了之后的参数起，行键照补了之后的参数算；
#      每补一行打一行「- <名>：cargo 参数补上 --no-default-features 起（验证包的测试目标不编装置二进制）」；变异表那一行不改。
#      次序同门禁 plan_rows：先补 --no-default-features、再补 --release（下一条）、再补 gpu 特性（开 GPU 补 gpu 特性那一条），补 --release 与补 gpu 特性都建在补过的参数上；表里没点名的同文行（key_peers）一样先换。
#   重测试目标：② 之后、先核原文之前，点名的每一行照 scripts/mutation/lib_release_only_tests.py 判（这里不另写判法；登记表读这份脚本所在主工作区那一份
#      .claude/gate.d/stage-inputs.tsv，副本里不一定有）：命中（cargo 参数选中登记的崩溃枚举用例目标，或特性里带 gpu）而没带 --release / --profile release 的，
#      这一次的 cargo 参数补 --release（插在第一个 -- 之前；基线与变异都照补了之后的参数），每补一行打一行「- <名>：cargo 参数补上 --release 起（<依据>）」；
#      变异表那一行不改。依据 .claude/rules/test-script-discipline.md「编译与算法都要快」第一句。登记表不在、读不了、crash-case: 行写法认不出，整次拒绝、退 2、一条都不跑；
#   开 GPU 补 gpu 特性：算行键那一步照门禁 crates-mutation-replay 的同一个函数 crates-mutation-rows.py 的 gpu_route_rows（那份文件头「开 GPU 时崩溃判段那几行补 gpu 特性」）：
#      多机配置（cargo_environment 交给 cargo 的那一份 SINGLEFS_MULTI_HOST_CONFIG）写了 ENABLE_GPU=1、点名测试的单元闭包够得着崩溃判段的选路函数、cargo 参数里没开 gpu 的行，
#      这一次的 cargo 参数补 --features gpu（补了就命中重测试目标，同时补 --release），基线与变异都照补了之后的参数起，键照补了之后的参数算；
#      每补一行打一行「- <名>：cargo 参数补上 --features <特性> 起（…）」，补了就不编的（cfg(not(feature = "gpu")) 的用例）照原样参数起、打一行为什么。变异表那一行不改。
#      补了 gpu 特性的行就是带 gpu 特性、真跑测试的 cargo test：照文件头「不领卡」原样起，要卡的行全进同一路（「并行」一条）；
#   还原日志：拿到副本锁之后、做 ② 之前先看副本根的还原日志 <副本>/.prove-red-pending（⑤ 施加变异之前落、写回原文之后删；记着变异名、文件、原文、替换文与施加前的整份原字节）：
#      在就是上一趟施加之后没走到写回就被杀了（撞内存上限、服务停任务；SIGKILL 下 finally 不跑），统一任务服务重排的下一次尝试落在同一份副本上。
#      那份文件此刻是施加之后的样子（或写到一半，是它的前缀）、或施加之前的样子（或它的前缀），就把原字节写回、删日志、报一行「上一趟没还原的变异已写回：…」再往下做；
#      两样都不是（被杀之后又被人改过）、日志读不了的，整次拒绝、退 2、一条都不跑；
#   先核原文：基线之前对点名的每一行核原文在副本源码里恰好命中一次；不是的整次拒绝（退 2、cargo 一次都不起）：命中 0 次多半是副本里留着别的变异（没有还原日志可照的那种），
#      不核就会先跑基线、报成「基线红」；
#   ③ 点名的行一律照跑（验证包 singlefs-verification 的行同样）：这一条命令能不能起由项目代码闸与统一任务服务定，脚本不看环境变量另判；
#   ④ 每组不同的参数先跑一次基线（不改源码），基线红就停（退 2）；
#   ⑤ 每条变异：原文在文件里恰好命中一次才改（「先核原文」已核过一遍，这里照核），先落还原日志再改，直接跑 cargo test <参数>（不限时、不套内存包装），
#      跑完照 verdict_of_run 判：抓到（退出码非 0、点名的测试恰好对上一个用例且有 FAILED，它自己那一段输出（libtest 的「---- <全名> stdout ----」）里、所有段之外都没有行首 GPU_DEVICE_LOST 的行；#[should_panic] 的结果行多一段「 - should panic」同样认）、
#      无效（点名的一行都没出现而有 error 行或 could not compile）、链接失败（点名的一行都没出现，输出里有「error: linking with」与 rust-lld、没有 error[E<码>]：
#      编过了、失败在链接，不是替换文编不过，单列一档、不记无效、不盖库里原有的算数结论、这一趟退 1；判法在 crates-mutation-rows.py 的 verdict_of_run，
#      那一行打「✗ <名>\t链接失败\t…」，出路是删掉这一趟用的编译目录重跑这一行）、没红（其余；点名省掉模块路径而对上两个用例也算）、
#      本次未跑（点名的测试 ok 又打了 NOT_RUN_NO_GPU_ADAPTER；或点名的测试 FAILED 而它自己那一段输出里、或所有段之外（归属不明）有行首 GPU_DEVICE_LOST 的行，红的是设备丢失、不是断言）；跑完把原文件写回并 touch，再删还原日志。
#      判之前先核这一趟真重编了被改的那个包（.claude/kb/milestone/03-third-txn.md「待优化：流程、门禁与装置上踩过的坑」第 1 行）：
#      变异行改的文件往上找到的包（scripts/gate/lib_rebuilt_packages.py 的 package_of）在这一趟的输出里要有「Compiling <包名> v…」行；
#      没有（编译目录里的产物比改过的源码新，cargo 认它已编、测试跑在旧产物上）记「✗ <名>\t没重编」、整次退 1，
#      出路是刷源码修改时刻或换干净编译目录（不设 CARGO_TARGET_DIR，这一趟自己建一份新的）重跑。被改文件经 #[path]（或 include_str!）编进别的包的，
#      -p 点名的包里编译目录的 dep-info 列着被改文件的（一份 dep-info 列的头一个源文件所在的包），那个包有 Compiling 行也算重编过。
#      改的不是包里的 .rs、或编完它不在编译目录的任何 dep-info 里（这一趟编译不读它，例如没开的 feature 下才编的模块）的不核（判法 packages_to_check）。
#   编译目录归这一趟（证红自己的做法：每一趟新建、跑完删，不跨趟、跨副本、跨脚本共用；.claude/rules/test-script-discipline.md「编译与算法都要快」只要不同的树不共用编译目录）：调用方给了 CARGO_TARGET_DIR
#      （统一任务服务给这一次尝试的那一份、手跑自己设的）照它的、不删，归调用方；没给的，拿到副本锁之后在这一台自己的临时目录下新建 singlefs-prove-red-target-<随机>，
#      这一趟的每条 cargo 都用它，成功、红、被 TERM / HUP 停都删（scripts/hosts/run_build_directory.py：正常收尾删之前核本机没有进程开着里面的文件，有就不删、报出进程、
#      整次退非 0；被停时进程马上按默认处理停，不核、直接删）。不沿用副本里的 target、也不建在副本里：别的命令（另一条 cargo build、另一趟证红）留下的 target 与这一趟的编译参数
#      对不上时，头一条变异链接不过（链接器报未定义的符号），被记成「替换之后编不过」。被 SIGKILL 杀掉的那一趟留下的目录在临时目录里，名字每趟都是新的、不会被下一趟沿用，
#      归临时目录的属主收。两路以上时各路用自己的编译目录（下面「并行」一条），这一份照样建、照样删，不起 cargo。
#   并行（.claude/rules/command-safety-by-review.md「一个脚本里的检测项，能并行就并行」「并行不许把失败吃掉」）：④⑤ 要现判的行按被改的文件分组（planned_lanes），互不撞文件的组分到几路同时跑。
#      改同一个文件的行留在同一路、按点名次序一行一行跑（同一份编译目录接着增量编）；要卡的行（特性带 gpu，判法在 lib_release_only_tests.py）全放进同一路：对拍不同时抢一张卡，判定才与串行相同。
#      路数取组数与上限的小者，上限是环境变量 PROVE_RED_LANES（正整数，写错整次拒绝、退 2、一条都不跑），没设就是这一趟的 CARGO_BUILD_JOBS（取法见开头 python3 之前那几行）；
#      每路的 CARGO_BUILD_JOBS 是总数除以路数、至少 1。不按内存收路数：上限由统一任务服务按记录给，撞了由服务放大重排，判完的行已在判定库里，重排的那一趟只补没判的。
#      只有一路（只有一组，或上限是 1）就在 --copy 那份副本上跑，还原日志落在它的根上，④⑤ 照上面一步不差。
#      两路以上：每路在 <调用方设的 CARGO_TARGET_DIR，没设就是临时目录>/singlefs-prove-red-lanes-XXXXXX/lane-<路号>/ 下各拷一份工作副本（tree/，拷 --copy 那棵树，不留修改时刻，target、prove-red-logs、副本锁与还原日志不拷）、各建一份编译目录（target/），
#      施加与还原都在这一路的 tree/ 里，还原日志也落在那里，--copy 那份副本的源码一个字节都不动。各路的目录收尾就删（删之前核没有进程开着，scripts/hosts/run_build_directory.py；删不成整次退 1、点名进程）；
#      被杀时留在起这一趟的临时目录里，归那个临时目录的属主收（经统一任务服务的是它的现场目录）。
#      分两段：先各路同时拷副本、跑基线（各路的冷编译落在这一段），收齐之后照串行的次序判（参数排好序，同一组参数按点名次序），有一路基线红整次退 2、一条变异都不施加；再各路同时施加变异。
#      一行判完当场写判定库（各路共用一把锁），被打断之后重跑同一条命令只补没判的。行键与判定都与串行同一份：并行只定哪几行一起起，不进键。
#      基线日志 lane-<路号>-baseline.log、变异日志 NNN.log（NNN 是这一行在要现判的行里的次序）都在这一趟的日志目录里；判定行收齐之后按点名次序打，与串行逐行相同（「- 并行」那一行与日志路径除外）。
#      派出几路要收回几路：对不上、或有一路抛了异常，逐路点名、整次红（基线那一段退 2，变异那一段退 1）；一路的红不吃掉，任何一路有没红、无效或没重编都退 1。
# 每条一行判定打到 stdout（名、结局、日志路径）；日志放 ${PROVE_RED_LOG_DIRECTORY:-<副本>/prove-red-logs}/run-<进程号>-<随机串>/（每一趟自己一个目录）。
# 退出码：0 跑了的都抓到、至少抓到一条（复用的抓到算在里面；本次未跑的照列，不算失败）；1 有没红、无效、链接失败或没重编，或复用哨兵判红，或变异那一段派出几路没收回几路、有一路抛了异常，或各路的目录删不成；
#   2 用法错、被拒、行键或判定库算不出、基线红、PROVE_RED_LANES 写错、各路的目录建不了、基线那一段派出几路没收回几路或有一路抛了异常；77 一条都没抓到、点名的都是本次未跑（点名的测试打了 NOT_RUN 或 GPU_DEVICE_LOST）；75 判定库服务连不上（客户端退 75）。
set -uo pipefail
# 拆成的几份（内联 python 只留一个载入器与 main；本体在同目录的 python 模块里，按这个次序往前 import、没有环；入口 import 之后名字一个不少地放在内联 python 的命名空间里）：
#   prove_red_core.py  证红的判法本体：读变异表与成形、重测试目标补 --release 与验证包补行、起 cargo 与编译目录、还原日志与断点还原、判定库的行键与复用、并行的路、prove 与 prove_on_restored_sources。名字在入口 prove-red.sh 的内联 python 里再导出；弄坏开关 PROVE_RED_BREAK 的取值在入口文件头的开关表里。
#   prove_red_selftest.py  自证：样本表与样本结局、假 cargo、各组格函数（并行、判定库、选路、卡租约、编译目录）与 selftest 本体。只被入口的 --selftest 起，import core。
# 弄坏开关的代码在哪一份（开关表仍在上面各节；shell 这一段读的开关写在这个文件里，python 部分读的名字出现在哪一份就在哪一份）：
#   prove_red_core.py：any-target、baseline-ignored、baseline-incremental-kept、caller-target-overridden、copy-closure-unchecked、copy-target-reused、error-line-invalid、gpu-route-feature-unadded、gpu-rows-cards-dropped、gpu-rows-compiled-apart、library-unread、link-failure-listed-as-invalid、main-allowed、mutated-run-on-real-service、narrowed-recorded、narrowed-sentinel-drawn、never-caught、no-copy-lock、no-default-features-unadded、no-rebuild-check、no-restore-journal、old-text-checked-late、own-judge、own-table、parallel-count-unchecked、parallel-gpu-rows-split、parallel-jobs-undivided、parallel-lane-red-swallowed、parallel-lanes-kept、parallel-lanes-unchecked、parallel-one-at-a-time、parallel-output-finish-order、parallel-same-file-split、parallel-shared-target、path-included-package-ignored、red-not-recorded、release-unforced、rocksdb-entry-unchecked、sentinel-unplanned、shared-log-directory、trip-target-kept、trip-target-kept-on-stop
#   prove_red_selftest.py：inherited-target-kept
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
# 弄坏开关只在 --selftest 里认（文件头「弄坏开关」）：命令行第一个参数不是 --selftest 而下面的变量有一个非空（不管值是什么），
# 在参数解析之前退 2、点名它只在自证里认，一步都没做
for break_variable in PROVE_RED_BREAK LIBTEST_RESULT_LINES_BREAK; do
  if [[ -n "${!break_variable:-}" && "${1:-}" != --selftest ]]; then
    echo "  ✗ $break_variable 只在 --selftest 里认（弄坏开关只给自证证红用），这一趟不是自证，一步都没做" >&2
    echo "     → 怎么办：去掉 $break_variable 再跑；要证红就在临时 prune-server 底下跑 $break_variable=<项> python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- bash scripts/mutation/prove-red.sh --selftest" >&2
    exit 2
  fi
done
SCRIPT_DIRECTORY="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# 副本路径与主仓不同、或 bindgen 的环境变了，librocksdb-sys 的 C++ 会在基线那一次重编；cc 按 cargo 的并行度开编译器，
# 调用方另给 CARGO_BUILD_JOBS 就照它的，没给取登记的重活默认并行度（.claude/heavy-thread-default，scripts/hosts/heavy-threads.sh）。
# 32 路一起编 RocksDB 撞过 12G 与 16G 的上限：这一整条的上限由 task-server 按记录给，撞了由服务放大重排。
source "$SCRIPT_DIRECTORY/../hosts/heavy-threads.sh"
if [[ -z "${CARGO_BUILD_JOBS:-}" ]]; then
  if [[ "${PROVE_RED_BREAK:-}" == build-jobs-six && -n "${PROVE_RED_SELFTEST_PROLOGUE:-}" ]]; then
    CARGO_BUILD_JOBS=6   # 弄坏开关（文件头）
  else
    CARGO_BUILD_JOBS="$(heavy_thread_default)"
  fi
fi
export CARGO_BUILD_JOBS
# CARGO_TARGET_DIR 的默认值（这一趟自己建的编译目录，文件头「编译目录归这一趟」）在下面的 python 里解析完 --copy、拿到副本锁之后才建、只设给 cargo 那几次（prove）：
# 这里不从位置参数取——只给 --selftest 时没有 $2（set -u 下当场退 1），--copy 不在第一个时 $2 也不是副本。
if [[ "${PROVE_RED_BREAK:-}" == unbound-second-argument && -n "${PROVE_RED_SELFTEST_PROLOGUE:-}" ]]; then
  export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$2/target}"   # 弄坏开关（文件头）
fi
python3 /dev/fd/3 "$SCRIPT_DIRECTORY" "$@" 3<<'PY'
import os, sys

sys.dont_write_bytecode = True   # 下面按同目录 import 的两份模块与按路径载入的两份模块不往 __pycache__ 里写
SCRIPT_DIRECTORY = sys.argv[1]
sys.path.insert(0, SCRIPT_DIRECTORY)   # prove_red_core.py、prove_red_selftest.py 与本脚本同目录
from prove_red_core import (SCRIPT_DIRECTORY, MAIN_ROOT, ROCKSDB_PREBUILT, BROKEN, TARGET_OPTIONS, EXIT_NOT_RUN, RESTORE_JOURNAL_NAME, load_module,
    ROWS, COPY_LOCK, REBUILT, REPOSITORY_COPY, RELEASE_ONLY_TESTS, RUN_BUILD_DIRECTORY, COPY_HOWTO, CLOSURE_ROOT_FOR_SELFTEST,
    LIBRARY_ROOT_FOR_SELFTEST, SCRIPT_KEY, SENTINEL_LABEL, ROW_STATE, HOW_TEXT, LANES_VARIABLE, LANES_ROOT_PREFIX, LANE_COPY_IGNORED, Lane,
    RowResult, DROP_LAST_LANE_FOR_SELFTEST, own_table_rows, table_rows, refusal_for, default_features_off_rows, release_forced_rows,
    source_path_problem, own_verdict, verdict_of, first_sentence, judged_line, named_packages, packages_reading_the_file, included_package_rebuilt,
    rebuild_problem, cargo_environment, baseline_environment, TRIP_BUILD_DIRECTORY_PREFIX, TRIP_BUILD_DIRECTORIES, new_trip_build_directory,
    remove_trip_build_directory, run_cargo_to_log, run_cargo, MUTATED_RUN_SERVICE_ROOT_PREFIX, MUTATED_RUN_SERVICE_ROOTS,
    prepare_mutated_run_service, remove_mutated_run_service_root, mutated_run_prune_server, stopped_by_signal,
    remove_trip_build_directories_when_stopped, normalized_text, mutated_bytes_of, write_restore_journal, drop_restore_journal,
    restore_unrestored_mutation, old_text_misses, copy_closure_refusal, LibraryPlanRefused, key_peers, gpu_route_notes, library_plan,
    planned_sentinel, state_text, record_verdict, record_index, build_jobs_total, lane_ceiling, row_needs_card, planned_lanes, prepared_lanes,
    run_in_lanes, lanes_collected_problem, remove_lanes_root, prove, prove_holding_the_lock, prove_on_restored_sources, write_text,
    prove_from_arguments)
from prove_red_selftest import (SELFTEST_TABLE, SELFTEST_VERDICT_SAMPLES, path_included_package_failure, process_gone, selftest_copy, reuse_states,
    REUSE_SOURCE, REUSE_TABLE, REUSE_FAKE_CARGO, PARALLEL_SOURCES, PARALLEL_TABLE, PARALLEL_EXPECTED, PARALLEL_FAKE_CARGO, parallel_cells,
    library_cells, NO_CARD_LEASE_TABLE, GPU_ROUTE_TABLE, gpu_route_cells, no_card_lease_cells, trip_build_directory_cells, SERVICE_PROBE_PROGRAM,
    selftest)

def main():
    arguments = sys.argv[2:]
    if arguments == ["--selftest"]:
        if os.environ.get("PROVE_RED_SELFTEST_PROLOGUE"):
            # 自证起的那一层子进程：只证开头那几行走得过去，打它此刻看到的 CARGO_TARGET_DIR 就退（编译目录那一格核这一行）
            print(f"PROVE_RED_PROLOGUE CARGO_TARGET_DIR={os.environ.get('CARGO_TARGET_DIR', '<未设>')}")
            print(f"PROVE_RED_PROLOGUE CARGO_BUILD_JOBS={os.environ.get('CARGO_BUILD_JOBS', '<未设>')}")
            return 0
        return selftest()
    remove_trip_build_directories_when_stopped()
    code, lines = prove_from_arguments(arguments)
    for line in lines:
        print(line)
    return code


sys.exit(main())
PY
