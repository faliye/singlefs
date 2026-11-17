#!/usr/bin/env bash
# admission: always 实现员每写一条新测试都要现证它会红，判的是这一刻副本里的源码与变异行；能不能复用由每一行的行键对判定库逐条判（文件头「判定库与复用」），不整次跳过
# run-condition: command python3 cargo rustc git
# 编译族（task-server/task_server_lib.py 文件头「资源记录」的编译族）：编的包取第三个参数 <crate>（$3），变异名每条命令都不同、不进族；
#   这里起的是 cargo test <变异行的参数>，命中重测试目标（登记的崩溃枚举用例目标、gpu 特性）的行一律补 --release 起（见「判法与次序」的「重测试目标」一条）。
#   命中与否随点名的变异行变，声明只能写一份：声明 release 那一档、不带特性——重的那几行（编得多、撞上限的多是它们）在 release 档，按 release 族起步；
#   不命中的 dev 档那几行编得少，照 release 族起步只多给、不少给（带 --features gpu 的那几行编得多些，撞了由服务放大）。
#   算行键（文件头「判定库与复用」）时副本里有 verus! 块的文件要产品行为视图，scripts/gate/lib_function_units.py 的 built_unit_graph 照它的缓存取 unit-graph、
#   缓存里没有这一版才编一次（release 档的 singlefs-unit-graph，与门禁 crates-mutation-replay 共用那一份缓存）：那一次多编一个包，族照旧按 -p <crate> 起步，撞了由服务放大。
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
#   起 cargo 的环境照门禁关调试信息（crates-mutation-rows.py 的 cargo_debug_overrides：取值进了行键的工具链行，不关就与键对不上）。
#   判法单元指纹（行键的一段）只收 crates-mutation-rows.py 那几个判法入口，这份脚本自己施加变异、起 cargo 那几段不在里面：只改这里施加与起 cargo 的写法，库里的判定不作废。
#   复用哨兵：每一趟开跑前抽（scripts/gate/lib_reuse_checks.py 文件头「二」；候选是此刻行键在 records/ 里记着抓到的行，项名照门禁的 sentinel_item 是「<行号> <变异名>」）；
#     跑完照 observe_fresh_outcomes 交结局：哨兵被现判、强制现判那一条判得与库里不同，整次退 1；强制现判那一条判成本次未跑的不交哨兵。
#   每条判定那一行末尾标（现判：<栏>）（复用）（强制现判）（拒算），栏照门禁的 COLUMN_TITLES；成功句与失败句都报「现判 N 条、复用 M 条、拒算 R 条」。
#   算不出判法单元指纹、工具链、依赖版本或库的位置，点名测试的闭包算不成：整次拒绝、退 2、cargo 一次都不起；服务连不上照客户端退 75，不当「库里没有」现判。
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
#   编译目录归这一趟（.claude/rules/test-script-discipline.md「编译与算法都要快」：每一趟新建、跑完删，不跨趟、跨副本、跨脚本共用）：调用方给了 CARGO_TARGET_DIR
#      （统一任务服务给这一次尝试的那一份、手跑自己设的）照它的、不删，归调用方；没给的，拿到副本锁之后在这一台自己的临时目录下新建 singlefs-prove-red-target-<随机>，
#      这一趟的每条 cargo 都用它，成功、红、被 TERM / HUP 停都删（scripts/hosts/run_build_directory.py：正常收尾删之前核本机没有进程开着里面的文件，有就不删、报出进程、
#      整次退非 0；被停时进程马上按默认处理停，不核、直接删）。不沿用副本里的 target、也不建在副本里：别的命令（另一条 cargo build、另一趟证红）留下的 target 与这一趟的编译参数
#      对不上时，头一条变异链接不过（链接器报未定义的符号），旧做法下被记成「替换之后编不过」。被 SIGKILL 杀掉的那一趟留下的目录在临时目录里，名字每趟都是新的、不会被下一趟沿用，
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
import base64, collections, hashlib, importlib.util, json, os, re, secrets, shlex, shutil, signal, subprocess, sys, tempfile, threading, time

sys.dont_write_bytecode = True   # 下面按路径载入的两份模块不往 __pycache__ 里写
SCRIPT_DIRECTORY = sys.argv[1]
MAIN_ROOT = os.path.realpath(os.path.join(SCRIPT_DIRECTORY, "..", ".."))
ROCKSDB_PREBUILT = os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "rocksdb-prebuilt.py")   # 起 cargo 之前判 RocksDB 预编库的取值（prove_holding_the_lock）
BROKEN = os.environ.get("PROVE_RED_BREAK", "")
TARGET_OPTIONS = {"--lib", "--test", "--bin"}
EXIT_NOT_RUN = 77
RESTORE_JOURNAL_NAME = ".prove-red-pending"   # 还原日志（文件头「判法与次序」的「还原日志」一条）：放在副本根


def load_module(path, module_name):
    specification = importlib.util.spec_from_file_location(module_name, path)
    module = importlib.util.module_from_spec(specification)
    specification.loader.exec_module(module)
    return module


# 判法（切表、成形、判结局）与门禁 crates-mutation-replay 同一份
ROWS = load_module(os.path.join(SCRIPT_DIRECTORY, "crates-mutation-rows.py"), "crates_mutation_rows_for_prove_red")
# 副本锁（文件头「判法与次序」⓪）与 apply-writer-patch.py 同一把、同一份写法
COPY_LOCK = load_module(os.path.join(SCRIPT_DIRECTORY, "..", "gate", "apply-writer-patch.py"), "apply_writer_patch_for_prove_red")
# 被改的包这一趟重编了没有（文件头「判法与次序」⑤）：与 merge-and-verify.sh 同一份判法
REBUILT = load_module(os.path.join(SCRIPT_DIRECTORY, "..", "gate", "lib_rebuilt_packages.py"), "lib_rebuilt_packages_for_prove_red")
# 副本里起 cargo 的环境（多机配置显式交过去）、核副本缺不缺（entries_missing_in_copy）与建副本的出路：拷贝范围的共用库
REPOSITORY_COPY = load_module(os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "lib_repository_copy.py"), "lib_repository_copy_for_prove_red")
# 重测试目标只许 release（文件头「判法与次序」的「重测试目标」一条）：判法只在这份共用库，与门禁 crates-mutation-replay 同用
RELEASE_ONLY_TESTS = load_module(os.path.join(SCRIPT_DIRECTORY, "lib_release_only_tests.py"), "lib_release_only_tests_for_prove_red")
# 各路目录的删法（文件头「判法与次序」的「并行」一条）：与起 cargo 的入口脚本删编译目录同一份
RUN_BUILD_DIRECTORY = load_module(os.path.join(SCRIPT_DIRECTORY, "..", "hosts", "run_build_directory.py"), "run_build_directory_for_prove_red")
COPY_HOWTO = ("python3 scripts/hosts/minimal-copy.py <仓根> <草稿目录>/copy --with-test-inputs 拷一份（拷贝范围取 scripts/hosts/lib_repository_copy.py 的闭包，"
              "连测试读的仓根文件），再给 --copy")
# 核副本缺不缺时闭包照哪棵树算：平常是 prove 的 main_root（主工作区）；自证开头放一棵临时的假主工作区进来，各格的假副本照它核（文件头「判法与次序」①）
CLOSURE_ROOT_FOR_SELFTEST = []
# 判定库的库根（文件头「判定库与复用」）：平常是这份脚本所在的主工作区 MAIN_ROOT；自证开头放一棵自己 git init 的临时仓进来
LIBRARY_ROOT_FOR_SELFTEST = []
SCRIPT_KEY = "scripts/mutation/prove-red.sh"   # 复用三问的规范输出与复用哨兵认的脚本名（.claude/gate.d/verdict-library-scripts.tsv 里这一行的路径）
SENTINEL_LABEL = "证红"
# 每条判定那一行末尾的标（文件头「判定库与复用」）：现判（带栏）、复用、强制现判、拒算
ROW_STATE = re.compile(r"（(现判|复用|强制现判|拒算)(?:：[^（）]*)?）$")
HOW_TEXT = "prove-red.sh 现判（直接起 cargo，不设上限与限时）"   # 判定库记录里的跑法：只是一样属性，不进键
# 并行（文件头「判法与次序」的「并行」一条）
LANES_VARIABLE = "PROVE_RED_LANES"
LANES_ROOT_PREFIX = "singlefs-prove-red-lanes-"
LANE_COPY_IGNORED = shutil.ignore_patterns("target", "prove-red-logs", COPY_LOCK.COPY_LOCK_NAME, COPY_LOCK.COPY_OWNER_NAME,
                                           RESTORE_JOURNAL_NAME, RESTORE_JOURNAL_NAME + ".tmp", LANES_ROOT_PREFIX + "*")
Lane = collections.namedtuple("Lane", "number rows copy environment log_prefix")
RowResult = collections.namedtuple("RowResult", "lines failed caught not_run fresh reported")   # 一行现判的结局（fresh 是交复用哨兵的那一份，没出结局的是 None）
DROP_LAST_LANE_FOR_SELFTEST = []   # 自证「并行收回数核对」那一格放一个值进来：变异那一段丢掉最后派出的那一路的结果


def own_table_rows(path):
    """弄坏开关 own-table：六段就收，哪一段空着都不管。"""
    rows = {}
    for line_number, line in enumerate(open(path, encoding="utf-8"), 1):
        if line.startswith("#") or not line.strip():
            continue
        fields = line.rstrip("\n").split("\t")
        if len(fields) == 6:
            rows[fields[0]] = ROWS.Row(line_number, fields[0], fields[1], ROWS.unescape(fields[2]), ROWS.unescape(fields[3]), fields[4], fields[5], tuple(fields))
    return rows, {}


def table_rows(copy):
    """({变异名: 成形的那一行}, {变异名: 为什么不成形})：照 crates-mutation-rows.py 的 read_table 读；同名的取表里最后一行。"""
    path = os.path.join(copy, "crates", "mutations.tsv")
    if BROKEN == "own-table":
        return own_table_rows(path)
    rows, malformed = ROWS.read_table(path)
    return {row.name: row for row in rows}, {name: reason for _line_number, name, reason in malformed}


def refusal_for(row, crate):
    arguments = row.arguments.split()
    before_separator = arguments[: arguments.index("--")] if "--" in arguments else arguments
    if BROKEN != "any-target" and not TARGET_OPTIONS & set(before_separator):
        return f"参数「{row.arguments}」没挑目标（--lib / --test / --bin 都没有），会把这个包的全部测试目标都编、都跑"
    if "-p" not in before_separator or before_separator[before_separator.index("-p") + 1:before_separator.index("-p") + 2] != [crate]:
        return f"参数「{row.arguments}」不是 -p {crate}"
    return None


def default_features_off_rows(chosen, environment):
    """(行, [要报的行])：验证包只点名集成测试目标的行换成补了 --no-default-features 的（文件头「判法与次序」的「验证包的测试目标不编装置二进制」一条），
    判法是 crates-mutation-rows.py 的 default_features_off_row（门禁 plan_rows 同一个函数，在补 --release 与补 gpu 特性之前）；别的原样。
    弄坏开关 no-default-features-unadded 下一律原样。"""
    if BROKEN == "no-default-features-unadded":
        return chosen, []
    off, notes = [], []
    for row in chosen:
        replaced = ROWS.default_features_off_row(row, environment)
        if replaced.arguments != row.arguments:
            notes.append(f"- {row.name}：cargo 参数补上 --no-default-features 起（验证包的测试目标不编装置二进制）")
        off.append(replaced)
    return off, notes


def release_forced_rows(chosen, registry_root):
    """(行, [要报的行])：命中重测试目标而 cargo 参数没带 release 的行换成补了 --release 的（文件头「判法与次序」的「重测试目标」一条），
    别的原样；登记表读 registry_root 那一份，读不了抛共用库的 RegistryUnreadable。弄坏开关 release-unforced 下一律原样。"""
    if BROKEN == "release-unforced":
        return chosen, []
    registered = RELEASE_ONLY_TESTS.crash_case_targets(registry_root)
    forced, notes = [], []
    for row in chosen:
        arguments, requirement = RELEASE_ONLY_TESTS.release_forced(row.arguments, registered=registered)
        if arguments != row.arguments:
            notes.append(f"- {row.name}：cargo 参数补上 --release 起（{'；'.join(requirement.basis)}）")
            row = row._replace(arguments=arguments)
        forced.append(row)
    return forced, notes


def source_path_problem(copy, path):
    root = os.path.realpath(copy)
    resolved = os.path.realpath(os.path.join(root, path))
    if os.path.isabs(path) or os.path.commonpath([root, resolved]) != root or resolved == root:
        return f"文件 {path} 不在副本内，不能施加变异"
    try:
        if not os.path.isfile(resolved):
            return f"文件 {path} 不是普通文件"
        if os.stat(resolved).st_nlink != 1:
            return f"文件 {path} 有硬链接，不能保证变异只改副本"
    except OSError as error:
        return f"文件 {path} 读不了：{error}"
    return None


def own_verdict(row, code, text):
    """弄坏开关 own-judge：prove-red 自己那一份判法（任一行点名的 FAILED 就算抓到，不看退出码、不看对上几个用例、不认 NOT_RUN）。"""
    if re.search(r"^test (?:\S*::)?" + re.escape(row.expected) + r"(?: - should panic)? \.\.\. FAILED", text, re.M):
        return "caught", ""
    if re.search(r"^error\[E\d+\]:", text, re.M) or "could not compile" in text:
        return "invalid", ""
    if re.search(r"^error:", text, re.M) and not re.search(r"^test result: ", text, re.M):
        return "invalid", ""
    return "failure", ""


def verdict_of(row, code, text):
    """(档, 那一句)：照 crates-mutation-rows.py 的 verdict_of_run（门禁同一份）。"""
    if BROKEN == "own-judge":
        return own_verdict(row, code, text)
    return ROWS.verdict_of_run(row, code, text, environment=os.environ)


def first_sentence(text):
    """verdict_of_run 那一句的头一行，去掉「crates/mutations.tsv:<行号> <名>：」。"""
    head = (text.splitlines() or [""])[0]
    return head.split("：", 1)[1] if "：" in head else head


def judged_line(row, code, text, log_path):
    """(caught / failed / not-run, 要打的那一行)。"""
    verdict, sentence = verdict_of(row, code, text)
    if verdict == "caught" and BROKEN != "never-caught":
        return "caught", f"{row.name}\t抓到\t{row.expected} 红了（日志 {log_path}）"
    if verdict == "linkfailed" and BROKEN != "link-failure-listed-as-invalid":
        return "failed", (f"✗ {row.name}\t链接失败\t替换之后编过了、失败在链接，不是替换文编不过：删掉这一趟用的编译目录（调用方给的 CARGO_TARGET_DIR；"
                          f"没给的这一趟自己建的已经删了），重跑这一行（日志 {log_path}）")
    if verdict in ("invalid", "linkfailed") or (verdict == "failure" and BROKEN == "error-line-invalid" and re.search(r"^error(?:\[E\d+\])?:", text, re.M)):
        return "failed", f"✗ {row.name}\t无效\t替换之后编不过（日志 {log_path}）"
    if verdict in ("failure", "caught"):
        detail = first_sentence(sentence) if verdict == "failure" and sentence else f"{row.expected} 没判红"
        return "failed", f"✗ {row.name}\t没红\t{detail}（日志 {log_path}）"
    if verdict == "notrun":
        return "not-run", f"- 本次未跑 {row.name}：{first_sentence(sentence)}（日志 {log_path}）"
    return "failed", f"✗ {row.name}：结局认不出（verdict_of_run 判成 {verdict}，退出码 {code}）（日志 {log_path}）"


def named_packages(row):
    """cargo test 参数里 -p / --package 点名的包（-- 之前）。"""
    words = row.arguments.split()
    words = words[:words.index("--")] if "--" in words else words
    packages = []
    for index, word in enumerate(words):
        if word in ("-p", "--package") and index + 1 < len(words):
            packages.append(words[index + 1])
        elif word.startswith("--package="):
            packages.append(word.split("=", 1)[1])
    return packages


def packages_reading_the_file(copy, relative, target_directory):
    """编译目录里 dep-info 列着被改文件的那几个包：一份 dep-info 列的头一个源文件是那个编译单元的根文件，它所在的包就是读这份文件的包。
    被改文件经 #[path]（或 include_str! 一类）编进别的包时，重编的是那个包，不是被改文件自己所在的包。"""
    wanted = {os.path.normpath(os.path.join(os.path.realpath(copy), relative)), os.path.normpath(os.path.join(os.path.abspath(copy), relative))}
    roots = {os.path.abspath(copy), os.path.realpath(copy), REBUILT.workspace_root_of(copy, relative)}   # dep-info 里的相对路径以工作区根解

    def resolved(word):
        return [os.path.normpath(word)] if os.path.isabs(word) else [os.path.normpath(os.path.join(root, word)) for root in roots]

    readers = set()
    for path in REBUILT.dep_info_files(target_directory):
        try:
            with open(path, encoding="utf-8", errors="replace") as handle:
                text = handle.read()
        except OSError:
            continue
        if os.path.basename(relative) not in text:
            continue
        words = [token.rstrip(":") for token in text.split()]
        if len(words) < 2 or not wanted & {candidate for word in words[1:] for candidate in resolved(word)}:
            continue
        for candidate in resolved(words[1]):
            inside = os.path.relpath(os.path.realpath(candidate), os.path.realpath(copy))
            package = None if inside.startswith("..") else REBUILT.package_of(copy, inside)
            if package is not None:
                readers.add(package)
                break
    return readers


def included_package_rebuilt(copy, row, text, target_directory):
    """被改文件经 #[path]（或 include_str!）编进别的包时的重编核对（文件头「判法与次序」⑤）：-p 点名的包里 dep-info 列着被改文件的，
    这一趟输出里有它的 Compiling 行，就算被改文件重编过。"""
    if not target_directory:
        return False
    readers = set(named_packages(row)) & packages_reading_the_file(copy, row.path, target_directory)
    return bool(readers & REBUILT.compiled_packages(text))


def rebuild_problem(copy, row, code, text, log_path, target_directory=None):
    """施加变异之后那一趟真重编了被改的包没有（文件头「判法与次序」⑤）：没有交要打的那一行，有（或这一趟不核）交 None。"""
    if BROKEN == "no-rebuild-check":
        return None
    if target_directory and not os.path.isabs(target_directory):   # 相对路径是 cargo 照它的当前目录（副本根）解的
        target_directory = os.path.join(copy, target_directory)
    packages, _note = REBUILT.packages_to_check(copy, [row.path], target_directory)
    if not packages:   # 改的不是包里的 .rs、或这一趟编译不读它（不在编译目录的任何 dep-info 里）：cargo 本来就不因它重编，不核
        return None
    package = packages[0]
    if REBUILT.missing_rebuilds([package], [text]):
        if BROKEN != "path-included-package-ignored" and included_package_rebuilt(copy, row, text, target_directory):
            return None
        return (f"✗ {row.name}\t没重编\t改了 {row.path}，这一趟的输出里却没有「Compiling {package} v…」行：编译目录里的产物比改过的源码新，"
                f"测试跑的是旧产物，判定不算数；刷源码修改时刻或换干净编译目录（不设 CARGO_TARGET_DIR，这一趟自己建一份新的）重跑（日志 {log_path}）")
    return None


def cargo_environment(copy, target_directory=None):
    """cargo 那几次的环境：调用方设了（非空）CARGO_TARGET_DIR 照它的（归调用方、这里不删）；没设就不放，prove 拿到副本锁之后给这一趟自己建一份（new_trip_build_directory，跑完删）。
    证红要这一趟自己专用的编译目录，不与别的编译共用、不沿用副本里别的命令留下的 target（.claude/rules/test-script-discipline.md「编译与算法都要快」）：
    每条变异改坏源码再还原，还原后的文件 mtime 可能比变异那次编出的产物旧，cargo 就把变异产物当成新的接着用——共用目录里另一份副本的测试就会跑在上一条变异的库上；
    别的命令编出的产物与这一趟的编译参数对不上时，头一条变异链接不过（链接器报未定义的符号）。
    弄坏开关 caller-target-overridden：调用方设了的也换成 <副本>/target（target_directory 给的取代它；调用方没设的不动、照旧自己建一份）；copy-target-reused：没设时用 <副本>/target、不另建。
    副本里没有多机配置（它是 minimal-copy.py git init 的一棵新仓，不带 multi-host.env）：多机配置照 lib_repository_copy.py 的 environment_for_commands_in_the_copy 显式交给 SINGLEFS_MULTI_HOST_CONFIG（取这份脚本所在的主工作区那一份）。
    构建档照门禁关调试信息（crates-mutation-rows.py 的 cargo_debug_overrides，文件头「判定库与复用」）：行键的工具链行照这一份环境算，取值进了键。"""
    environment = REPOSITORY_COPY.environment_for_commands_in_the_copy(MAIN_ROOT)
    if (BROKEN == "caller-target-overridden" and environment.get("CARGO_TARGET_DIR")) or (BROKEN == "copy-target-reused" and not environment.get("CARGO_TARGET_DIR")):
        environment["CARGO_TARGET_DIR"] = target_directory or os.path.join(copy, "target")
    environment.update(ROWS.cargo_debug_overrides(environment))
    return environment


TRIP_BUILD_DIRECTORY_PREFIX = "singlefs-prove-red-target-"
TRIP_BUILD_DIRECTORIES = []   # 这一趟自己建、还没删的编译目录：被 TERM、HUP 停时 stopped_by_signal 照它删


def new_trip_build_directory():
    """调用方没给 CARGO_TARGET_DIR 时这一趟自己的编译目录（文件头「判法与次序」的「编译目录归这一趟」）：建在这一台自己的临时目录下
    （副本在工作根的共享盘上，编译目录不放过去，也不进副本的 target），名字随机，每一趟一份新的。建不了抛 OSError。"""
    directory = os.path.realpath(tempfile.mkdtemp(prefix=TRIP_BUILD_DIRECTORY_PREFIX))
    TRIP_BUILD_DIRECTORIES.append(directory)
    return directory


def remove_trip_build_directory(directory):
    """收尾删这一趟自己建的编译目录（成功、红、被打断都走）：→ (删成了没有, [要打的行])。directory 是 None（调用方给了编译目录，归调用方删）原样交 (True, [])；
    删成了不打字（输出与调用方给了编译目录时一样多）；有进程开着里面的文件、删不成交 (False, 原因与出路)。弄坏开关 trip-target-kept：不删。"""
    if directory is None or BROKEN == "trip-target-kept":
        return True, []
    removed, lines = RUN_BUILD_DIRECTORY.remove_run_build_directory(directory)
    if removed:
        TRIP_BUILD_DIRECTORIES.remove(directory)
    return removed, [] if removed else lines


def run_cargo_to_log(copy, arguments, log_path, cargo, environment, mode):
    """cargo test <参数> 起一次，输出写进日志（mode：w 覆盖、a 接着写）：→ (退出码, 日志全文)。"""
    with open(log_path, mode, encoding="utf-8") as log:
        result = subprocess.run([cargo, "test", *arguments], cwd=copy, stdout=log, stderr=subprocess.STDOUT,
                                env=environment)
    return result.returncode, open(log_path, encoding="utf-8", errors="replace").read()


def run_cargo(copy, arguments, log_path, cargo, environment):
    """cargo 是调用方交的：平常是 PATH 上的 cargo，自证给它写出的假 cargo 桩。这一处登在 scripts/gate/lib_function_units.py 的 UNKEYED_PROGRAM_WORDS：
    真 cargo 编的 crates 由每条变异行的行键管，假 cargo 的正文在自证里，单元闭包不往上认这个参数。
    不领卡（文件头「不领卡」）：带不带 gpu 特性都照原样起、不另编，环境里的 GPU_CARDS_BY_PRIORITY 有就原样交。
    弄坏开关 gpu-rows-compiled-apart：带 gpu 特性的先另起一条只编的（--no-run，输出写进同一份日志），编过了再起这一条；
    gpu-rows-cards-dropped：带 gpu 特性的起之前去掉环境里的 GPU_CARDS_BY_PRIORITY。"""
    gpu_row = BROKEN in ("gpu-rows-compiled-apart", "gpu-rows-cards-dropped") and ROWS.gpu_test_run(["cargo", "test", *arguments], environment)
    if gpu_row and BROKEN == "gpu-rows-cards-dropped":
        environment = {name: value for name, value in environment.items() if name != "GPU_CARDS_BY_PRIORITY"}
    mode = "w"
    if gpu_row and BROKEN == "gpu-rows-compiled-apart":
        separator = arguments.index("--") if "--" in arguments else len(arguments)
        code, text = run_cargo_to_log(copy, [*arguments[:separator], "--no-run"], log_path, cargo, environment, mode)
        if code != 0:
            return code, text
        mode = "a"
    return run_cargo_to_log(copy, arguments, log_path, cargo, environment, mode)


def stopped_by_signal(signal_number, _frame):
    """被停（文件头「编译目录归这一趟」）：删这一趟自己建的编译目录（进程马上按默认处理停，起 cargo 的子进程开着的文件不再等，不核开着的进程），
    再按默认处理重发同一个信号。弄坏开关 trip-target-kept-on-stop：不删。"""
    for directory in list(TRIP_BUILD_DIRECTORIES) if BROKEN != "trip-target-kept-on-stop" else []:
        removed, removal_lines = RUN_BUILD_DIRECTORY.remove_run_build_directory(directory, check_open=False)
        if not removed:
            print("\n".join(removal_lines), file=sys.stderr)
    signal.signal(signal_number, signal.SIG_DFL)
    os.kill(os.getpid(), signal_number)


def remove_trip_build_directories_when_stopped():
    """TERM、HUP 都走 stopped_by_signal；只在主线程里设得了。"""
    for signal_number in (signal.SIGTERM, signal.SIGHUP):
        signal.signal(signal_number, stopped_by_signal)


def normalized_text(original_bytes):
    """锚点按文本模式匹配（通用换行）；写回用原始字节，保留 CRLF。"""
    return original_bytes.decode("utf-8").replace("\r\n", "\n").replace("\r", "\n")


def mutated_bytes_of(original_bytes, old, new):
    """施加变异之后那份文件的字节（⑤ 施加与还原日志认「施加之后的样子」同用这一份）。"""
    return normalized_text(original_bytes).replace(old, new, 1).encode("utf-8")


def write_restore_journal(copy, row, original_bytes):
    """施加之前在副本根落还原日志（文件头「判法与次序」的「还原日志」一条）：先写临时文件、落盘再换上，被杀在中途时要么没有、要么是整份。"""
    if BROKEN == "no-restore-journal":
        return
    record = {"name": row.name, "path": row.path, "old": row.old, "new": row.new,
              "original_sha256": hashlib.sha256(original_bytes).hexdigest(),
              "original_base64": base64.b64encode(original_bytes).decode("ascii")}
    journal = os.path.join(copy, RESTORE_JOURNAL_NAME)
    with open(journal + ".tmp", "w", encoding="utf-8") as handle:
        json.dump(record, handle, ensure_ascii=False)
        handle.flush()
        os.fsync(handle.fileno())
    os.replace(journal + ".tmp", journal)


def drop_restore_journal(copy):
    for leftover in (RESTORE_JOURNAL_NAME, RESTORE_JOURNAL_NAME + ".tmp"):
        try:
            os.unlink(os.path.join(copy, leftover))
        except FileNotFoundError:
            pass


def restore_unrestored_mutation(copy):
    """拿到副本锁之后、做 ② 之前照还原日志把上一趟没还原的变异写回（文件头「判法与次序」的「还原日志」一条）。
    → (None, [要报的行])：没有日志，或写回了 / 本来就是原文；(2, [拒绝行])：日志读不了、文件两样都不是。"""
    journal = os.path.join(copy, RESTORE_JOURNAL_NAME)
    if BROKEN == "no-restore-journal" or not os.path.exists(journal):
        return None, []
    clean_copy_way = "→ 怎么办：照还原日志里的文件与原文核对那份文件（diff -rq 对一份干净副本），盖回原文、删掉日志再证；或换一份干净副本；一条都没跑"
    try:
        with open(journal, encoding="utf-8") as handle:
            record = json.load(handle)
        name, relative, old, new = record["name"], record["path"], record["old"], record["new"]
        if not all(isinstance(value, str) for value in (name, relative, old, new)):
            raise TypeError("日志里的变异名、文件、原文、替换文不全是字符串")
        original_bytes = base64.b64decode(record["original_base64"], validate=True)
        if hashlib.sha256(original_bytes).hexdigest() != record["original_sha256"]:
            raise ValueError("日志里的原字节与它记的 sha256 对不上")
    except (OSError, ValueError, KeyError, TypeError) as error:
        return 2, [f"✗ 副本里有上一趟证红留下的还原日志 {journal}，读不了：{error}", clean_copy_way]
    problem = source_path_problem(copy, relative)
    if problem:
        return 2, [f"✗ 还原日志 {journal} 点名的{problem}", clean_copy_way]
    path = os.path.join(copy, relative)
    with open(path, "rb") as handle:
        current = handle.read()
    if current == original_bytes:   # 写回之后、删日志之前被打断
        drop_restore_journal(copy)
        return None, [f"- 上一趟的还原日志还在、{relative} 已是施加 {name} 之前的原文：删了日志"]
    mutated = mutated_bytes_of(original_bytes, old, new)
    if not (mutated.startswith(current) or original_bytes.startswith(current)):   # 前缀：写到一半被杀
        return 2, [f"✗ 副本里 {relative} 既不是上一趟证红施加 {name} 之前、也不是施加之后的样子（还原日志 {journal}）：被打断之后又被改过", clean_copy_way]
    with open(path, "wb") as handle:
        handle.write(original_bytes)
    os.utime(path)
    drop_restore_journal(copy)
    return None, [f"- 上一趟没还原的变异已写回：{name}（{relative}，还原日志 {journal}）"]


def old_text_misses(copy, chosen):
    """基线之前核点名的每一行原文在副本源码里恰好命中一次（文件头「判法与次序」的「先核原文」一条）：→ [拒绝行]。"""
    if BROKEN == "old-text-checked-late":
        return []
    misses = []
    for row in chosen:
        with open(os.path.join(copy, row.path), "rb") as handle:
            hits = normalized_text(handle.read()).count(row.old)
        if hits == 0:
            misses.append(f"✗ {row.name}：原文在 {row.path} 里命中 0 次，要恰好 1 次——副本里可能留着别的变异，照还原日志或换一份干净副本"
                          f"（diff -rq 对一份干净副本看哪份文件不同）；副本是干净的，就是这一行的原文与源码对不上，照源码改原文")
        elif hits != 1:
            misses.append(f"✗ {row.name}：原文在 {row.path} 里命中 {hits} 次，要恰好 1 次——把 crates/mutations.tsv 里这一行的原文写长到只命中一处")
    return misses


def copy_closure_refusal(copy, closure_root):
    """①（文件头「判法与次序」）的后半：按 lib_repository_copy.py 的同一份闭包（closure_root 那棵树、带测试输入）核副本根下缺不缺闭包里的条目。
    缺的、闭包算不成的交 (2, [输出行])；不缺交 None。弄坏开关 copy-closure-unchecked 下不核。"""
    if BROKEN == "copy-closure-unchecked":
        return None
    try:
        missing, checked = REPOSITORY_COPY.entries_missing_in_copy(closure_root, copy, test_inputs=True)
    except (ValueError, OSError) as error:   # ClosureRefused 是 ValueError 的子类
        return 2, [f"✗ 算不出 {closure_root} 的副本闭包，判不了副本 {copy} 缺不缺：{error}",
                   f"→ 怎么办：python3 scripts/hosts/lib_repository_copy.py entries {closure_root} --test-inputs 看它报什么、照它的出路修；一条都没跑"]
    if not missing:
        return None
    return 2, [f"✗ 副本 {copy} 缺 {len(missing)} 条闭包里的仓根条目（闭包共 {checked} 条，照 {closure_root} 带测试输入算）："
               + "；".join(f"{entry}（{'；'.join(reasons)}）" for entry, reasons in missing),
               f"→ 怎么办：{REPOSITORY_COPY.copy_rebuild_howto(closure_root)}，再给 --copy；一条都没跑"]


class LibraryPlanRefused(Exception):
    """行键或判定库算不出（文件头「判定库与复用」末句）：lines 是整次拒绝要打的那几行。"""

    def __init__(self, lines):
        super().__init__(lines[0])
        self.lines = lines


def key_peers(copy, chosen, registry_root, environment):
    """表里没点名、而原文与替换文都与点名的某一行相同的行（行键撞得上的只有这几行；crates-mutation-rows.py 的 plan_rows 同一取法），
    照点名的行一样先补 --no-default-features（验证包只点名集成测试目标的）、再补 --release（命中重测试目标的）。"""
    planned = {row.line_number for row in chosen}
    texts = {(row.old, row.new) for row in chosen}
    rows, _malformed = ROWS.read_table(os.path.join(copy, "crates", "mutations.tsv"))
    peers = default_features_off_rows([row for row in rows if row.line_number not in planned and (row.old, row.new) in texts], environment)[0]
    return release_forced_rows(peers, registry_root)[0]


def gpu_route_notes(key_context, before, after):
    """补了 gpu 特性的行与补了之后不编的行各打一行（文件头「判法与次序」的「开 GPU 补 gpu 特性」一条）：before、after 是补之前与之后的点名的行（点名次序）。"""
    planned = {row.line_number: (row, after_row) for row, after_row in zip(before, after)}
    notes = []
    for line_number, feature in key_context.gpu_route_lines:
        if line_number in planned:
            row, featured = planned[line_number]
            released = "--release" in featured.arguments.split() and "--release" not in row.arguments.split()
            notes.append(f"- {row.name}：cargo 参数补上 --features {feature} 起（多机配置 {key_context.gpu_route_configuration} 开了 GPU（ENABLE_GPU=1），"
                         f"点名测试的单元闭包够得着崩溃判段的选路函数；同门禁 crates-mutation-replay 的 gpu_route_rows）"
                         + ("，补了 gpu 特性就命中重测试目标、另补 --release" if released else ""))
    for line_number, why in key_context.gpu_route_unbuilt:
        if line_number in planned:
            notes.append(f"- {planned[line_number][0].name}：点名的测试走崩溃判段，补 gpu 特性之后不编（{why}），照原样参数起")
    return notes


def library_plan(copy, chosen, registry_root, environment):
    """(KeyContext, 这一趟的点名的行, {行号: Components}, 判定库, {行号: (Decision, 记录)}, [补 gpu 特性要报的行])：照 crates-mutation-rows.py 的 KeyContext、
    gpu_route_rows、refuse_colliding_row_keys、open_library 与 decide_rows（文件头「判定库与复用」）。库根平常是 MAIN_ROOT，自证换成 LIBRARY_ROOT_FOR_SELFTEST 里那一棵。
    交回的行：多机配置开了 GPU 时点名测试走崩溃判段、cargo 参数没开 gpu 的那几行换成补了 gpu 特性的（门禁 plan_rows 同一个函数），别的原样；键照换了之后的参数算。
    弄坏开关 library-unread 下键算得出的行一律判现判、不读 records/ 与索引；gpu-route-feature-unadded 下不补 gpu 特性。算不出抛 LibraryPlanRefused。"""
    library_root = LIBRARY_ROOT_FOR_SELFTEST[0] if LIBRARY_ROOT_FOR_SELFTEST else MAIN_ROOT
    function_units = ROWS.mutation_units.function_units()
    route_notes = []
    try:
        peers = key_peers(copy, chosen, registry_root, environment)
        key_context = ROWS.KeyContext(copy, environment)
        key_context.release_registered = RELEASE_ONLY_TESTS.crash_case_targets(registry_root)   # 补了 gpu 特性的行再照判一次重测试目标（gpu_route_rows 里）
        key_context.prepare(list(chosen) + peers)
        components = {row.line_number: key_context.components(row) for row in list(chosen) + peers}
        if BROKEN != "gpu-route-feature-unadded":
            every_row, components = ROWS.gpu_route_rows(key_context, list(chosen) + peers, components, environment)
            routed = {row.line_number: row for row in every_row}
            before, chosen = list(chosen), [routed[row.line_number] for row in chosen]
            route_notes = gpu_route_notes(key_context, before, chosen)
        key_context.collisions = ROWS.refuse_colliding_row_keys(components, environment)
        library = ROWS.open_library(library_root, environment)
    except ROWS.mutation_units.ClosureRefused as error:
        raise LibraryPlanRefused([f"✗ 点名的测试在 cargo 参数选的这一组特性下认不出或不编（{len(error.failures)} 处），行键算不出来：一条都没跑"]
                                 + [f"   {line}" for line in error.detail_lines()]
                                 + [f"→ 怎么办：{''.join(ROWS.CLOSURE_REFUSED_HOWTO)}"]) from error
    except (ROWS.JudgingUnitsRefused, ROWS.mutation_units.MutationUnitsError, ROWS.admission_module().InputManifestError, ROWS.HeavyThreadDefaultError,
            function_units.FunctionUnitsError, function_units.UsageError, RELEASE_ONLY_TESTS.RegistryUnreadable, OSError, UnicodeDecodeError) as error:
        raise LibraryPlanRefused([f"✗ 算不出判法单元指纹、工具链、依赖版本或判定库的位置（{type(error).__name__}：{error}），判定库没处对：一条都没跑",
                                  f"→ 怎么办：库根 {library_root} 要是 git 工作树、rustc -vV 与 cargo -V 跑得通、副本的 Cargo.lock 读得懂；"
                                  f"python3 scripts/mutation/crates-mutation-rows.py judging-units {MAIN_ROOT} 看判法单元报什么"]) from error
    chosen_components = {row.line_number: components[row.line_number] for row in chosen}
    if BROKEN == "library-unread":
        decided = {line_number: (ROWS.Decision(False, "refused", comp.refusal) if comp.row_key is None
                                 else ROWS.Decision(False, "unexplained", "弄坏开关 library-unread：没读库"), None)
                   for line_number, comp in chosen_components.items()}
    else:
        decided = ROWS.decide_rows(library, chosen, chosen_components, key_context, environment)
    return key_context, chosen, chosen_components, library, decided, route_notes


def planned_sentinel(library, chosen, components, environment):
    """复用哨兵（文件头「判定库与复用」）：→ (哨兵, 要打的行)。候选是此刻行键在 records/ 里记着抓到的行（直接读记录，与 decide_rows 分开），
    项名照门禁的 sentinel_item。弄坏开关 sentinel-unplanned 下不抽。缩过域的一趟不抽（文件头「缩过域的一趟」；弄坏开关 narrowed-sentinel-drawn 下照抽）。"""
    sentinel = ROWS.reuse_checks.ReuseSentinel(SCRIPT_KEY, SENTINEL_LABEL, environment=os.environ)
    lines = []
    if BROKEN == "sentinel-unplanned":
        sentinel.plan({}, reuse=False, off_reason="弄坏开关 sentinel-unplanned", printer=lines.append)
        return sentinel, lines
    narrowing = ROWS.crash_domain_narrowing(environment)
    if narrowing and BROKEN != "narrowed-sentinel-drawn":
        sentinel.plan({}, reuse=False, off_reason=ROWS.narrowed_sentinel_off_reason(narrowing), printer=lines.append)
        return sentinel, lines
    candidates = {}
    for row in chosen:
        key = components[row.line_number].row_key
        record = library.read("records", key) if key else None
        if record is not None and record.get("row_key") == key and record.get("verdict") == ROWS.REUSABLE_VERDICT["mutation"]:
            candidates[ROWS.sentinel_item(row)] = "抓到"
    sentinel.plan(candidates, printer=lines.append)
    return sentinel, lines


def state_text(decision):
    """每条判定那一行末尾标的那几个字（文件头「判定库与复用」）：复用、强制现判、拒算，别的是「现判：<栏>」。"""
    if decision.reuse:
        return "复用"
    if decision.column == "sentinel":
        return "强制现判"
    if decision.column == "refused":
        return "拒算"
    return f"现判：{ROWS.COLUMN_TITLES.get(decision.column, decision.column)}"


def record_verdict(library, row, comp, verdict, text, code, seconds, test_full_name, environment):
    """照行键写一条判定（文件头「判定库与复用」；字段与 crates-mutation-rows.py 的 judge_row 同形，这里没有每条自己的 TMPDIR，删 TMPDIR 那几个数写 0）：
    → 写不进去时要打的那一行，写了或不写交 None。拒算的不写；本次未跑不盖同一把键原有的算数结论；弄坏开关 red-not-recorded 下只写抓到。
    缩过域的一趟一条都不写（文件头「缩过域的一趟」；弄坏开关 narrowed-recorded 下照写）。"""
    if not comp.row_key:
        return None
    if ROWS.crash_domain_narrowing(environment) and BROKEN != "narrowed-recorded":
        return None
    if verdict != ROWS.REUSABLE_VERDICT["mutation"] and BROKEN == "red-not-recorded":
        return None
    if verdict in ROWS.VERDICTS_UNCOUNTED and ROWS.uncounted_verdict_keeps_counted_record(
            library.read("records", comp.row_key), verdict, comp.row_key, environment):
        return None
    light = ROWS.light_components(comp)
    fields = collections.OrderedDict(
        kind="mutation", row_key=comp.row_key, judging=comp.judging, verdict=verdict, text=text,
        removal_files=0, removal_allocated_bytes=0, removal_apparent_bytes=0, removal_left=0,
        exit_code=code, judged_date=ROWS.mutation_units.today_text(), how=HOW_TEXT, test_full_name=test_full_name,
        test_closure=light["test_closure"], mutated=light["mutated"], toolchain=light["toolchain"], dependency=light["dependency"],
        seconds_running=f"{seconds:.3f}")
    try:
        library.write("records", comp.row_key, fields)
    except OSError as error:
        return f"! {row.name}：判定没写进判定库（{library.path('records', comp.row_key)}：{error}），下一趟这一条照现判"
    return None


def record_index(library, rows, components):
    """复用与现判出结局的行照 index_entry 写索引（只用来让下一趟说清现判的原因）：→ 写不进去时要打的那几行。"""
    problems = []
    for row in rows:
        comp = components[row.line_number]
        if not comp.row_key:
            continue
        try:
            library.write("index", ROWS.row_identity(row), ROWS.index_entry(comp, row))
        except OSError as error:
            problems.append(f"! {row.name}：索引没写进判定库（{error}），下一趟键对不上时说不清变了什么")
    return problems


def build_jobs_total(environment):
    """这一趟的编译并行度总数（开头 python3 之前那几行定的 CARGO_BUILD_JOBS）：不是正整数抛 ValueError。"""
    jobs = environment.get("CARGO_BUILD_JOBS", "").strip()
    if not (jobs.isascii() and jobs.isdigit() and int(jobs) >= 1):
        raise ValueError(f"CARGO_BUILD_JOBS={jobs} 不是正整数")
    return int(jobs)


def lane_ceiling(environment):
    """同时至多开几路（文件头「判法与次序」的「并行」一条）：PROVE_RED_LANES 设了照它，没设是编译并行度总数；写错抛 ValueError。"""
    given = os.environ.get(LANES_VARIABLE, "").strip()
    total = build_jobs_total(environment)
    if not given:
        return total
    if given.isascii() and given.isdigit() and int(given) >= 1:
        return int(given)
    if BROKEN == "parallel-lanes-unchecked":
        return 1
    raise ValueError(f"{LANES_VARIABLE}={given} 不是正整数")


def row_needs_card(row):
    """这一行的 cargo 参数带 gpu 特性：判法只在 lib_release_only_tests.py（不给 crash-case 目标，依据就只剩 gpu 特性那一样）。"""
    return RELEASE_ONLY_TESTS.release_requirement(row.arguments, registered=()).required


def planned_lanes(rows, ceiling):
    """rows（点名次序）分成几路：→ [每路的行，点名次序]，几路按头一行的点名次序排（文件头「判法与次序」的「并行」一条）。
    改同一个文件的行、要卡的行并进同一组；组数比上限多时把大的组先分、每次放进此刻行数最少的那一路。弄坏开关 parallel-same-file-split、parallel-gpu-rows-split。"""
    leaders = list(range(len(rows)))

    def leader_of(index):
        while leaders[index] != index:
            index = leaders[index]
        return index

    group_holder = {}
    for index, row in enumerate(rows):
        keys = [(os.path.normpath(row.path), index if BROKEN == "parallel-same-file-split" else None)]
        if BROKEN != "parallel-gpu-rows-split" and row_needs_card(row):
            keys.append("要卡")
        for key in keys:
            leaders[leader_of(index)] = leader_of(group_holder.setdefault(key, index))
    groups = {}
    for index in range(len(rows)):
        groups.setdefault(leader_of(index), []).append(index)
    lanes = [[] for _lane in range(min(len(groups), ceiling))]
    for members in sorted(groups.values(), key=lambda members: (-len(members), members[0])):
        min(lanes, key=len).extend(members)
    return [[rows[index] for index in sorted(lane)] for lane in sorted(lanes, key=min)]


def prepared_lanes(copy, lane_rows, environment):
    """→ ([Lane…], 各路目录的根或 None, [要打的行])。只有一路就在 --copy 那份副本上跑，不建目录；两路以上建好各路的目录与环境，工作副本由这一路在基线那一段自己拷。
    弄坏开关 parallel-shared-target、parallel-jobs-undivided。建不了目录抛 OSError。"""
    if len(lane_rows) <= 1:
        return [Lane(1, rows, copy, environment, "") for rows in lane_rows], None, []
    base = os.path.join(copy, os.environ["CARGO_TARGET_DIR"]) if os.environ.get("CARGO_TARGET_DIR") else tempfile.gettempdir()
    os.makedirs(base, exist_ok=True)
    lanes_root = os.path.realpath(tempfile.mkdtemp(prefix=LANES_ROOT_PREFIX, dir=base))
    total = build_jobs_total(environment)
    jobs = total if BROKEN == "parallel-jobs-undivided" else max(1, total // len(lane_rows))
    lanes = []
    for number, rows in enumerate(lane_rows, 1):
        lane_environment = dict(environment)
        lane_environment["CARGO_TARGET_DIR"] = os.path.join(lanes_root, "target" if BROKEN == "parallel-shared-target" else f"lane-{number:02d}/target")
        lane_environment["CARGO_BUILD_JOBS"] = str(jobs)
        lanes.append(Lane(number, rows, os.path.join(lanes_root, f"lane-{number:02d}", "tree"), lane_environment, f"lane-{number:02d}-"))
    return lanes, lanes_root, [f"- 并行：开 {len(lanes)} 路（撞文件的行与要卡的行留在同一路），每路编译并行度 {jobs}；各路的工作副本与编译目录在 {lanes_root}"]


def run_in_lanes(lanes, work):
    """每路一条线程跑 work(lane)，全部收齐才回：→ ([每路的结果，按派活的次序；抛了异常的是 None], [每路抛的异常，一路一行])。
    只有一路就在本线程跑，异常照常往外抛。弄坏开关 parallel-one-at-a-time。"""
    if len(lanes) <= 1:
        return [work(lane) for lane in lanes], []
    results, problems = [None] * len(lanes), [None] * len(lanes)

    def run_lane(position, lane):
        try:
            results[position] = work(lane)
        except Exception as error:   # 收进这一路的结局，由收束那一步点名、整次判红；这里不往外抛，线程里抛了没人接
            problems[position] = f"✗ 第 {lane.number} 路抛了 {type(error).__name__}：{error}"

    threads = [threading.Thread(target=run_lane, args=(position, lane)) for position, lane in enumerate(lanes)]
    for thread in threads:
        thread.start()
        if BROKEN == "parallel-one-at-a-time":
            thread.join()
    for thread in threads:
        thread.join()
    return results, [problem for problem in problems if problem]


def lanes_collected_problem(lanes, results):
    """派出几路要收回几路（文件头「判法与次序」的「并行」一条）：对不上交要打的那一行，对得上交 None。弄坏开关 parallel-count-unchecked。"""
    collected = sum(1 for result in results if result is not None)
    if BROKEN == "parallel-count-unchecked" or collected == len(lanes):
        return None
    return f"✗ 派出 {len(lanes)} 路、只收回 {collected} 路，没收回的那几路的行没判定：整次不算数"


def remove_lanes_root(lanes_root):
    """收尾删各路的目录：→ (删成了没有, [要打的行])。删成了不打字（输出与串行一样多）。弄坏开关 parallel-lanes-kept。"""
    if lanes_root is None or BROKEN == "parallel-lanes-kept":
        return True, []
    removed, lines = RUN_BUILD_DIRECTORY.remove_run_build_directory(lanes_root)
    return removed, [] if removed else lines


def prove(copy, crate, names, cargo="cargo", target_directory=None, main_root=None, closure_root=None, registry_root=None):
    """→ (退出码, [输出行])。main_root 是判「--copy 指的是不是主工作区」时的主工作区，不给就是这份脚本所在的仓根 MAIN_ROOT；
    自证给一棵临时的假主工作区，那一格判得出来不靠自证从哪棵树起（从符号链接拼的草稿树起时 MAIN_ROOT 里没有 crates/mutations.tsv）。
    closure_root 是核副本缺不缺时闭包照哪棵树算：不给时取自证放进 CLOSURE_ROOT_FOR_SELFTEST 的那一棵，没有就是 main_root。
    registry_root 是重测试目标的登记表读哪一份仓根的（文件头「判法与次序」的「重测试目标」一条）：不给就是 MAIN_ROOT，自证那一格给自己造的一份。"""
    copy = os.path.realpath(copy)
    main_root = MAIN_ROOT if main_root is None else os.path.realpath(main_root)
    environment = cargo_environment(copy, target_directory)
    if not os.path.isfile(os.path.join(copy, "crates", "mutations.tsv")):
        return 2, [f"✗ {copy} 里没有 crates/mutations.tsv，不像仓副本", f"→ 怎么办：{COPY_HOWTO}"]
    if copy == main_root and BROKEN != "main-allowed":
        return 2, [f"✗ --copy 指的是主工作区 {copy}", f"→ 怎么办：在草稿目录里拷一份仓副本：{COPY_HOWTO}；证红只在副本上改坏源码"]
    refused = copy_closure_refusal(copy, closure_root or (CLOSURE_ROOT_FOR_SELFTEST[0] if CLOSURE_ROOT_FOR_SELFTEST else main_root))
    if refused is not None:
        return refused
    lock = None
    if BROKEN != "no-copy-lock" and copy != main_root:   # 主工作区只在弄坏开关 main-allowed 下走到这里，不在它根上建锁文件
        try:
            lock = COPY_LOCK.hold_copy_lock(copy)
        except OSError as error:
            return 2, [f"✗ 在副本 {copy} 上建不了副本锁 {COPY_LOCK.COPY_LOCK_NAME}：{error}", "→ 怎么办：看副本根可不可写；一条都没跑"]
        if lock is None:
            return 2, [f"✗ 副本 {copy} 上已有一趟证红在跑（{COPY_LOCK.COPY_LOCK_NAME} 被进程 {COPY_LOCK.copy_lock_holder(copy)} 占着），这一次一条都没跑",
                       "→ 怎么办：副本上有证红在跑，等它跑完或换一份副本"]
    try:
        # 编译目录归这一趟（文件头「判法与次序」的「编译目录归这一趟」）：调用方给了 CARGO_TARGET_DIR 的照它的、不删；没给的这一趟自己建一份新的，成功、红、被打断都在这里删
        trip_directory = None
        if not environment.get("CARGO_TARGET_DIR"):
            try:
                trip_directory = new_trip_build_directory()
            except OSError as error:
                return 2, [f"✗ 建不了这一趟自己的编译目录（{tempfile.gettempdir()} 下）：{error}", "→ 怎么办：看临时目录可不可写、盘满没满；一条都没跑"]
            environment["CARGO_TARGET_DIR"] = trip_directory
        try:
            code, lines = prove_holding_the_lock(copy, crate, names, cargo, environment, registry_root)
        finally:
            removed, removal_lines = remove_trip_build_directory(trip_directory)
        return (code, lines) if removed else (code or 1, lines + removal_lines)
    finally:
        if lock is not None:
            COPY_LOCK.release_copy_lock(copy, lock)


def prove_holding_the_lock(copy, crate, names, cargo, environment, registry_root=None):
    """拿着副本锁（文件头「判法与次序」⓪）先照还原日志写回上一趟没还原的变异，再往下做 ②–⑤：→ (退出码, [输出行])，写回的那一行打在最前面。"""
    refused, restored = restore_unrestored_mutation(copy)
    if refused is not None:
        return refused, restored
    code, lines = prove_on_restored_sources(copy, crate, names, cargo, environment, registry_root)
    return code, restored + lines


def prove_on_restored_sources(copy, crate, names, cargo, environment, registry_root=None):
    """②–⑤（副本源码里没有上一趟没还原的变异之后）：→ (退出码, [输出行])。registry_root 见 prove。"""
    rows, malformed = table_rows(copy)
    refusals, chosen = [], []
    for name in names:
        row = rows.get(name)
        if row is None:
            refusals.append(f"{name}：crates/mutations.tsv 里这一行不成形，{malformed[name]}" if name in malformed
                            else f"{name}：crates/mutations.tsv 里没有这一行")
            continue
        why = refusal_for(row, crate) or source_path_problem(copy, row.path)
        if why:
            refusals.append(f"{name}：{why}")
        else:
            chosen.append(row)
    if refusals:
        return 2, [f"✗ {line}" for line in refusals] + ["→ 怎么办：照 crates/mutations.tsv 表头把那一行写成六段（替换文之外不许空），cargo test 参数加上 -p <crate> 与 --lib / --test <目标> / --bin <名> 之一再证；一条都没跑"]
    chosen, default_features_notes = default_features_off_rows(chosen, environment)
    try:
        chosen, release_notes = release_forced_rows(chosen, registry_root or MAIN_ROOT)
        release_notes = default_features_notes + release_notes
    except RELEASE_ONLY_TESTS.RegistryUnreadable as error:
        return 2, [f"✗ 判不了点名的行里哪几行是重测试目标：{error}", f"→ 怎么办：{RELEASE_ONLY_TESTS.REGISTRY_HOWTO}；一条都没跑"]
    try:
        ceiling = lane_ceiling(environment)
    except ValueError as error:
        return 2, [f"✗ 算不出同时开几路：{error}", f"→ 怎么办：{LANES_VARIABLE} 写正整数或不设（不设时取 CARGO_BUILD_JOBS）；CARGO_BUILD_JOBS 写正整数或不设（不设时取登记的重活默认并行度）；一条都没跑"]
    misses = old_text_misses(copy, chosen)
    if misses:
        return 2, misses + ["→ 怎么办：照上面每一行的出路办，基线与变异一条都没跑"]
    # 判定库（文件头「判定库与复用」）：算行键、读库、抽复用哨兵；键相同、库里记着抓到的复用（不起 cargo），别的照跑
    try:
        key_context, chosen, components, library, decided, route_notes = library_plan(copy, chosen, registry_root or MAIN_ROOT, environment)
    except LibraryPlanRefused as error:
        return 2, list(release_notes) + error.lines
    sentinel, sentinel_lines = planned_sentinel(library, chosen, components, environment)
    decisions, reused = {}, []
    for row in chosen:
        decision = decided[row.line_number][0]
        if decision.reuse and sentinel.forced(ROWS.sentinel_item(row)):
            decision = ROWS.Decision(False, "sentinel", "复用哨兵抽中：键相同，这一趟不读库")
        decisions[row.line_number] = decision
        if decision.reuse:
            reused.append(row)
    to_judge = [row for row in chosen if not decisions[row.line_number].reuse]
    refused_count = sum(1 for row in chosen if components[row.line_number].row_key is None)
    counts_text = f"现判 {len(to_judge) - refused_count} 条、复用 {len(reused)} 条、拒算 {refused_count} 条"
    # 缩过域的一趟（文件头「缩过域的一趟」）：判法与门禁同一个函数 crash_domain_narrowing；现判的每一行末尾的标之前接一句缩过域，不写判定库与索引
    narrowing = ROWS.crash_domain_narrowing(environment)
    narrowed_unrecorded = bool(narrowing) and BROKEN != "narrowed-recorded"
    narrowed_note = f"；缩过域（{narrowing}）、不算门禁通过、没入库" if narrowing else ""
    # 组成清单先进库（判法单元、点名测试的闭包、工具链行、依赖版本行）：这一趟中途停了，门禁下一趟照样报得出变了哪几个单元
    key_context.persist(library, components)
    output = list(release_notes) + route_notes + sentinel_lines
    for row in chosen:
        if components[row.line_number].row_key is None:
            output.append(f"- 拒算 {row.name}：行键算不出（照跑、不复用、不写库）：{components[row.line_number].refusal}")
    for row in reused:
        output.append(f"{row.name}\t抓到\t{row.expected} 红了（{decisions[row.line_number].detail}）（复用）")
    # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：有要跑的行就在起任何 cargo 之前判一次（判的是副本那棵树，起的是 cargo 那一份），
    #   判红整次拒绝、退 2、一条都不跑。树里没有依赖 rocksdb 的清单（自证的副本）、cargo 是带标记的假 cargo，entry 报不判。弄坏开关 rocksdb-entry-unchecked 下不判
    if to_judge and BROKEN != "rocksdb-entry-unchecked":
        entry = subprocess.run([sys.executable, ROCKSDB_PREBUILT, "entry", "--root", copy, "--cargo", cargo], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        if entry.returncode != 0:
            return 2, [line for line in (entry.stdout + entry.stderr).splitlines() if line.strip()] + [
                "✗ 起 cargo 之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一条都没跑",
                '→ 怎么办：照上面的出路修（开新会话，或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"），再证这一批']
    logs = os.environ.get("PROVE_RED_LOG_DIRECTORY") or os.path.join(copy, "prove-red-logs")
    if BROKEN != "shared-log-directory":   # 每一趟写自己的日志目录：同一份副本上别的机器上的证红不会把基线日志换成它的（文件头「判法与次序」的日志一条）
        logs = os.path.join(logs, f"run-{os.getpid()}-{secrets.token_hex(4)}")
    if to_judge:
        os.makedirs(logs, exist_ok=True)

    def baseline_red(rows, code, problem_of):
        """基线红的那几行照行键记 baseline-not-ok（文件头「判定库与复用」记红撤绿）：→ 写不进去时要打的那几行。"""
        problems = []
        for row in rows:
            text = f"crates/mutations.tsv:{row.line_number} {row.name}：基线不绿——点名的测试 {row.expected} 在没改坏的副本上{problem_of(row)}"
            problem = record_verdict(library, row, components[row.line_number], "baseline-not-ok", text, code, 0.0, "", environment)
            if problem:
                problems.append(problem)
        return problems

    def lane_baselines(lane):
        """这一路（两路以上时先拷工作副本）跑自己那几组参数的基线：→ {参数: (退出码, 输出, 日志)}；一组红了或没证实点名的测试就停，这一路后面几组不跑（收齐之后的判法在那一组上就退了）。"""
        if lane.copy != copy:
            shutil.copytree(copy, lane.copy, ignore=LANE_COPY_IGNORED, copy_function=shutil.copy, symlinks=True)
        runs = {}
        for position, arguments in enumerate(sorted({row.arguments for row in lane.rows})):
            group_suffix = f"-{position + 1}" if position else ""
            log_path = os.path.join(logs, f"{lane.log_prefix}baseline{group_suffix}.log")
            baseline_arguments = ROWS.with_shown_output(arguments.split())
            code, text = run_cargo(lane.copy, baseline_arguments, log_path, cargo, lane.environment)
            runs[arguments] = (code, text, log_path)
            if BROKEN != "baseline-ignored" and (code != 0 or any(ROWS.baseline_problem(row, code, text) or ROWS.not_run_reason(row.expected, text)
                                                                  for row in lane.rows if row.arguments == arguments)):
                break
        return runs

    def judge_row(lane, row, index, library_lock):
        """在这一路的工作副本上施加、跑、还原、判一行（文件头「判法与次序」⑤）：→ RowResult。写判定库拿着 library_lock：各路共用一个判定库客户端。"""
        path = os.path.join(lane.copy, row.path)
        log_path = os.path.join(logs, f"{index:03d}.log")
        with open(path, "rb") as handle:
            original_bytes = handle.read()
        hits = normalized_text(original_bytes).count(row.old)
        if hits != 1:
            return RowResult([f"✗ {row.name}：原文在 {row.path} 里命中 {hits} 次，要恰好 1 次"], True, 0, 0, None, False)
        try:
            write_restore_journal(lane.copy, row, original_bytes)
        except OSError as error:
            return RowResult([f"✗ {row.name}：施加之前落不了还原日志 {os.path.join(lane.copy, RESTORE_JOURNAL_NAME)}（{error}），没施加——看副本根可不可写"], True, 0, 0, None, False)
        state = state_text(decisions[row.line_number])
        started = time.monotonic()
        try:
            with open(path, "wb") as handle:
                handle.write(mutated_bytes_of(original_bytes, row.old, row.new))
            code, text = run_cargo(lane.copy, row.arguments.split(), log_path, cargo, lane.environment)
        finally:
            with open(path, "wb") as handle:
                handle.write(original_bytes)
            os.utime(path)
            drop_restore_journal(lane.copy)
        seconds = time.monotonic() - started
        rebuilt_problem = rebuild_problem(lane.copy, row, code, text, log_path, lane.environment.get("CARGO_TARGET_DIR"))
        if rebuilt_problem:   # 判定不算数：不写库、不交哨兵（文件头「判定库与复用」）
            return RowResult([f"{rebuilt_problem}（{state}）"], True, 0, 0, None, False)
        outcome, line = judged_line(row, code, text, log_path)
        # 写进判定库的档：抓到、本次未跑照 judged_line 的结局，别的照 verdict_of_run 的档（没红、无效；弄坏开关改出来的「没红」记没红）
        verdict, sentence = verdict_of(row, code, text)
        verdict = {"caught": "caught", "not-run": "notrun"}.get(outcome) or (verdict if verdict in ("failure", "invalid", "linkfailed") else "failure")
        with library_lock:
            record_problem = record_verdict(library, row, components[row.line_number], verdict, sentence or line, code, seconds,
                                            ROWS.single_full_name(row.expected, text) or "", environment)
        lines = [f"{line}{narrowed_note}（{state}）"] + ([record_problem] if record_problem else [])
        return RowResult(lines, outcome == "failed", int(outcome == "caught"), int(outcome == "not-run"), (row.line_number, verdict, sentence, None), True)

    def judged_on_lanes(lanes):
        """④⑤ 按路跑（文件头「判法与次序」的「并行」一条）：先各路的基线、收齐照串行的次序判，再各路施加变异、收齐按点名次序打：→ (退出码, [输出行])。"""
        baseline_results, baseline_exceptions = run_in_lanes(lanes, lane_baselines)
        collected_problem = lanes_collected_problem(lanes, baseline_results)
        if baseline_exceptions or collected_problem:
            return 2, output + baseline_exceptions + ([collected_problem] if collected_problem else []) + [
                f"→ 怎么办：基线那一段有一路没走完，一条变异都没施加；看 {logs} 下那一路的基线日志与各路目录所在的盘还有多少，修好再证这一批；要退回串行就设 {LANES_VARIABLE}=1"]
        lane_of = {row.line_number: lane.number for lane in lanes for row in lane.rows}
        run_of = {(lane.number, arguments): run for lane, runs in zip(lanes, baseline_results) for arguments, run in runs.items()}
        for arguments in sorted({row.arguments for row in to_judge}):
            group = [row for row in to_judge if row.arguments == arguments]
            red_rows = [row for row in group if run_of[lane_of[row.line_number], arguments][0] != 0]
            if red_rows and BROKEN != "baseline-ignored":
                written = []
                for row in red_rows:
                    code, text, _log_path = run_of[lane_of[row.line_number], arguments]
                    written += baseline_red([row], code, lambda row: ROWS.baseline_problem(row, code, text) or f"退 {code}（cargo test {arguments}）")
                red_log_path = run_of[lane_of[red_rows[0].line_number], arguments][2]
                return 2, output + written + [f"✗ 基线（没改源码）就红：cargo test {arguments}（日志 {red_log_path}）；{counts_text}",
                                              "→ 怎么办：先让基线绿；基线里本来就红的测试不能拿来证红"]
            if BROKEN != "baseline-ignored":
                for row in group:
                    code, text, log_path = run_of[lane_of[row.line_number], arguments]
                    problem = ROWS.baseline_problem(row, code, text)
                    written = baseline_red([row], code, lambda _row: problem) if problem else []
                    problem = problem or ROWS.not_run_reason(row.expected, text)
                    if problem:
                        return 2, output + written + [f"✗ 基线没有证实 {row.name} 的点名测试通过：{problem}（日志 {log_path}）；{counts_text}",
                                                      "→ 怎么办：确认基线实际执行了点名测试并通过，再证红"]
        positions = {row.line_number: position for position, row in enumerate(to_judge, 1)}
        library_lock = threading.Lock()
        finish_order = []

        def lane_rows(lane):
            results = {}
            for row in lane.rows:
                results[row.line_number] = judge_row(lane, row, positions[row.line_number], library_lock)
                finish_order.append(row.line_number)
            return results

        lane_results, lane_exceptions = run_in_lanes(lanes, lane_rows)
        if DROP_LAST_LANE_FOR_SELFTEST and len(lanes) > 1:
            lane_results[-1] = None
        collected_problem = lanes_collected_problem(lanes, lane_results)
        results_of = {line_number: result for results in lane_results if results for line_number, result in results.items()}
        lines = output + lane_exceptions + ([collected_problem] if collected_problem else [])
        failed, caught, not_run = bool(lane_exceptions or collected_problem), len(reused), 0
        fresh_outcomes, reported = [], list(reused)   # 交复用哨兵的现判结局（行号, 档, 那一句, 删临时目录）；写索引的行
        # 收齐才按点名次序回读；弄坏开关 parallel-output-finish-order 下按跑完的先后，parallel-lane-red-swallowed 下头一路之外的红不算
        in_finish_order = sorted(to_judge, key=lambda row: finish_order.index(row.line_number) if row.line_number in finish_order else len(finish_order))
        for row in in_finish_order if BROKEN == "parallel-output-finish-order" else to_judge:
            result = results_of.get(row.line_number)
            if result is None:
                continue
            lines += result.lines
            swallowed = BROKEN == "parallel-lane-red-swallowed" and len(lanes) > 1 and lane_of[row.line_number] != lanes[0].number
            failed = failed or (result.failed and not swallowed)
            caught += result.caught
            not_run += result.not_run
            if result.fresh is not None:
                fresh_outcomes.append(result.fresh)
            if result.reported:
                reported.append(row)
        return finished_summary(lines, failed, caught, not_run, fresh_outcomes, reported)

    def finished_summary(lines, failed, caught, not_run, fresh_outcomes, reported):
        """记索引、交复用哨兵、打汇总句：→ (退出码, [输出行])。缩过域的一趟索引也不写，汇总句写缩过域（文件头「缩过域的一趟」）。"""
        lines = lines + ([] if narrowed_unrecorded else record_index(library, reported, components))
        # 复用哨兵（文件头「判定库与复用」）：复用的交「复用」，现判出结局的照门禁的 observe_fresh_outcomes 交
        for row in reused:
            sentinel.observe(ROWS.sentinel_item(row), ROWS.reuse_checks.REUSED, "抓到")
        ROWS.observe_fresh_outcomes(sentinel, fresh_outcomes, {row.line_number: row for row in chosen}, environment)
        sentinel_problems = sentinel.finish(printer=lines.append)
        for message, howto in sentinel_problems:
            lines += [f"✗ {message}", f"→ 怎么办：{howto}"]
        summary = (f"点名 {len(names)} 条：{counts_text}；跑了 {len(to_judge)} 条，抓到 {caught} 条（其中复用 {len(reused)} 条），本次未跑 {not_run} 条"
                   + (f"；缩过域（{narrowing}）、不算门禁通过、现判的没入库（复用的是库里全域下判的）" if narrowing else ""))
        if failed:
            return 1, lines + [f"✗ {summary}，有没红、无效、链接失败或没重编的，或有一路没收回（逐条列在上面）",
                               "→ 怎么办：没红的换一条真会改行为的变异或补断言；无效的改替换文；照 .claude/rules/mutation-sampling.md 分类；"
                               "链接失败的删掉调用方给的编译目录重跑那一行（没给的这一趟自己建的编译目录跑完已删，再重跑一次同一条命令）；没重编的按那一行刷修改时刻或换编译目录重跑；"
                               f"有一路没收回的照那一行重跑这一批（判完的已在判定库里），要退回串行就设 {LANES_VARIABLE}=1"]
        if sentinel_problems:
            return 1, lines + [f"✗ {summary}，复用哨兵判红（逐条列在上面）",
                               "→ 怎么办：照上面复用哨兵那几行的出路办"]
        if caught == 0:
            return EXIT_NOT_RUN, lines + [f"- {summary}：一条都没抓到，点名的都是本次未跑（逐条列在上面）",
                                          "→ 怎么办：打了 NOT_RUN、GPU_DEVICE_LOST 的是测试自己要开卡，任务不拿卡（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）；"
                                          "交 GPU 服务代跑的接口还没定，这几条照记本次未跑，不算证过红"]
        return 0, lines + [f"✓ {summary}，跑了的都抓到了"]

    try:
        lanes, lanes_root, lane_plan = prepared_lanes(copy, planned_lanes(to_judge, ceiling) if to_judge else [], environment)
    except OSError as error:
        return 2, output + [f"✗ 建不了各路的目录：{error}",
                            f"→ 怎么办：看调用方设的 CARGO_TARGET_DIR 或临时目录可不可写、盘满没满；要退回串行就设 {LANES_VARIABLE}=1；一条都没跑"]
    output += lane_plan
    try:
        code, lines = judged_on_lanes(lanes)
    except BaseException:
        remove_lanes_root(lanes_root)
        raise
    removed, removal_lines = remove_lanes_root(lanes_root)
    return (code, lines) if removed else (code or 1, lines + removal_lines)


def write_text(path, text, executable=False):
    os.makedirs(os.path.dirname(path), exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        handle.write(text)
    if executable:
        os.chmod(path, 0o755)


# 自证的样本表：checker 那一行是 验证包的（--lib），verification-test 那一行是验证包只点名集成测试目标的，noexpected 那一行必须红的测试名空着（不成形）
SELFTEST_TABLE = ("# 样本\n"
                  "caught\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "untargeted\tcrates/demo/src/lib.rs\t{ 42 }\t{ 43 }\t-p demo -- answer\tanswer_is_42\n"
                  "layer\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --test demo_crash_point_replay -- answer\tanswer_is_42\n"
                  "panics\tcrates/demo/src/lib.rs\t{ 42 }\t{ 45 }\t-p demo --lib -- answer\tanswer_panics\n"
                  "broken\tcrates/demo/src/lib.rs\t{ 42 }\t{ 46 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "elsewhere\tcrates/demo/src/lib.rs\t{ 42 }\t{ 47 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "linkfail\tcrates/demo/src/lib.rs\t{ 42 }\t{ 52 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "gpu\tcrates/demo/src/lib.rs\t{ 42 }\t{ 48 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "checker\tcrates/singlefs-verification/src/lib.rs\t{ 42 }\t{ 41 }\t-p singlefs-verification --lib -- answer\tanswer_is_42\n"
                  "verification-test\tcrates/singlefs-verification/src/lib.rs\t{ 42 }\t{ 40 }\t-p singlefs-verification --test verification_target -- answer\tanswer_is_42\n"
                  "redbase\tcrates/redbase/src/lib.rs\t{ 50 }\t{ 51 }\t-p redbase --lib -- answer\tanswer_is_42\n"
                  "release-gpu\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --features gpu --lib -- answer\tanswer_is_42\n"
                  "release-crash\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --test demo_crash_stream -- answer\tanswer_is_42\n"
                  "noexpected\tcrates/demo/src/lib.rs\t{ 42 }\t{ 49 }\t-p demo --lib -- answer\t\n"
                  "five-fields\tcrates/demo/src/lib.rs\t{ 42 }\t{ 49 }\t-p demo --lib -- answer\n")

# 判法逐行相同那一格的样本：(说明, 退出码, 输出, 点名)；prove-red 自己的判法与 verdict_of_run 判得不一样的几种都在里面
SELFTEST_VERDICT_SAMPLES = (
    ("点名的红了、退 101", 101, "test tests::answer_is_42 ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed\n", "answer_is_42"),
    ("should panic 的红了", 101, "test tests::answer_panics - should panic ... FAILED\ntest result: FAILED. 0 passed; 1 failed\n", "answer_panics"),
    ("点名省掉模块路径、两个模块各红一个", 101, "test a::answer_is_42 ... FAILED\ntest b::answer_is_42 ... FAILED\ntest result: FAILED. 0 passed; 2 failed\n", "answer_is_42"),
    ("退出码 0 而输出里有 FAILED", 0, "test tests::answer_is_42 ... FAILED\n", "answer_is_42"),
    ("编不过", 101, "error[E0308]: mismatched types\nerror: could not compile `demo`\n", "answer_is_42"),
    ("编过了、失败在链接", 101, "error: linking with `cc` failed: exit status: 1\n  = note: rust-lld: error: undefined hidden symbol: anon.0.llvm.1\n"
                               "error: could not compile `demo` (lib test) due to 1 previous error\n", "answer_is_42"),
    ("点名的绿、别的红", 101, "test tests::answer_is_42 ... ok\ntest tests::other ... FAILED\ntest result: FAILED. 1 passed; 1 failed\nerror: test failed\n", "answer_is_42"),
    ("点名的绿、打了 NOT_RUN", 0, "test tests::answer_is_42 ... ok\n---- tests::answer_is_42 stdout ----\nNOT_RUN_NO_GPU_ADAPTER test=tests::answer_is_42 reason=没卡\n"
                                   "test result: ok. 1 passed; 0 failed\n", "answer_is_42"),
    ("点名的红了、它那一段里有 GPU_DEVICE_LOST 行（test=main）", 101, "test tests::answer_is_42 ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                                  "GPU_DEVICE_LOST test=main card=NVIDIA GeForce RTX 5090 reason=Xid 31\n\nfailures:\n    tests::answer_is_42\n\n"
                                                                  "test result: FAILED. 0 passed; 1 failed\n", "answer_is_42"),
    ("点名的红了、它那一段里有 GPU_DEVICE_LOST 行（test=unnamed-thread）", 101, "test tests::answer_is_42 ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                                            "GPU_DEVICE_LOST test=unnamed-thread card=NVIDIA GeForce RTX 5090 reason=Xid 31\n\nfailures:\n    tests::answer_is_42\n\n"
                                                                            "test result: FAILED. 0 passed; 1 failed\n", "answer_is_42"),
    ("点名的红了、它那一段里有 GPU_DEVICE_LOST 行（test=点名的全名）", 101, "test tests::answer_is_42 ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                                       "GPU_DEVICE_LOST test=tests::answer_is_42 card=NVIDIA GeForce RTX 5090 reason=Xid 31\n\nfailures:\n    tests::answer_is_42\n\n"
                                                                       "test result: FAILED. 0 passed; 1 failed\n", "answer_is_42"),
    ("点名的红了、GPU_DEVICE_LOST 行在别的用例的那一段里", 101, "test tests::answer_is_42 ... FAILED\ntest tests::other ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                           "assertion failed\n\n---- tests::other stdout ----\n"
                                                           "GPU_DEVICE_LOST test=tests::answer_is_42 card=NVIDIA GeForce RTX 5090 reason=Xid 31\n\nfailures:\n    tests::answer_is_42\n    tests::other\n\n"
                                                           "test result: FAILED. 0 passed; 2 failed\n", "answer_is_42"),
    ("点名的红了、GPU_DEVICE_LOST 行在所有段之外（实时打印的部分）", 101, "GPU_DEVICE_LOST test=wgpu-callback card=NVIDIA GeForce RTX 5090 reason=Xid 31\n"
                                                                 "test tests::answer_is_42 ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                                 "assertion failed\n\nfailures:\n    tests::answer_is_42\n\ntest result: FAILED. 0 passed; 1 failed\n", "answer_is_42"),
    ("点名的红了、GPU_DEVICE_LOST 行在所有段之外，别的用例的那一段里另有一行", 101, "GPU_DEVICE_LOST test=wgpu-callback card=NVIDIA GeForce RTX 5090 reason=Xid 31\n"
                                                                        "test tests::answer_is_42 ... FAILED\ntest tests::other ... FAILED\n\nfailures:\n\n---- tests::answer_is_42 stdout ----\n"
                                                                        "assertion failed\n\n---- tests::other stdout ----\n"
                                                                        "GPU_DEVICE_LOST test=tests::other card=NVIDIA GeForce RTX 5090 reason=Xid 31\n\nfailures:\n    tests::answer_is_42\n    tests::other\n\n"
                                                                        "test result: FAILED. 0 passed; 2 failed\n", "answer_is_42"),
)


def path_included_package_failure(work):
    """自证一格（文件头「判法与次序」⑤，弄坏开关 path-included-package-ignored）：被改文件在 crates/owner/build.rs，经 #[path] 编进 crates/includer
    （includer 的测试目标 dep-info 列着它，相对工作区根、带 ..，真 cargo 的写法）；点名的是 -p includer，这一趟输出里只有「Compiling includer」、
    没有「Compiling owner」：判重编过；includer 也没有 Compiling 行的（只有别的包）仍判没重编。→ 失败说明，过了交 None。"""
    path_copy = os.path.join(work, "path-copy")
    write_text(os.path.join(path_copy, "Cargo.toml"), '[workspace]\nmembers = ["crates/*"]\n')
    for package in ("owner", "includer"):
        write_text(os.path.join(path_copy, "crates", package, "Cargo.toml"), f'[package]\nname = "{package}"\n')
        write_text(os.path.join(path_copy, "crates", package, "src", "lib.rs"), "fn answer() -> u32 { 42 }\n")
    write_text(os.path.join(path_copy, "crates", "owner", "build.rs"), "fn main() {}\n")
    path_target = os.path.join(work, "path-target")
    write_text(os.path.join(path_target, "debug", "deps", "includer-0123.d"),
               "target/debug/deps/includer-0123: crates/includer/src/lib.rs crates/includer/src/../../owner/build.rs\n\n"
               "crates/includer/src/lib.rs:\ncrates/includer/src/../../owner/build.rs:\n")
    write_text(os.path.join(path_target, "debug", "deps", "build_script_build-4567.d"),
               "target/debug/build/owner-4567/build_script_build-4567: crates/owner/build.rs\n\ncrates/owner/build.rs:\n")
    path_row = ROWS.Row(1, "included", "crates/owner/build.rs", "main", "mine", "-p includer --lib -- answer", "answer_is_42", ())
    included_green = rebuild_problem(path_copy, path_row, 101, "   Compiling includer v0.1.0 (/x/crates/includer)\ntest tests::answer_is_42 ... FAILED\n", "log", path_target)
    included_red = rebuild_problem(path_copy, path_row, 101, "   Compiling unrelated v0.1.0 (/x/crates/unrelated)\ntest tests::answer_is_42 ... FAILED\n", "log", path_target)
    if included_green is not None or included_red is None or "没重编" not in included_red:
        return (f"被改文件经 #[path] 编进点名的包（dep-info 列着它）：有点名的那个包的 Compiling 行就判重编过、没有就仍判没重编，"
                f"实际有 Compiling 行的判 {included_green!r}、没有的判 {included_red!r}")
    return None


def process_gone(pid, seconds=5):
    """自证自己起的进程号 pid 在 seconds 秒内没了（或只剩僵尸）就交 True。"""
    deadline = time.monotonic() + seconds
    while time.monotonic() < deadline:
        try:
            with open(f"/proc/{pid}/stat", encoding="utf-8") as handle:
                if handle.read().rsplit(")", 1)[1].split()[0] == "Z":
                    return True
        except (FileNotFoundError, ProcessLookupError):
            return True
        time.sleep(0.05)
    return False


def selftest_copy(work, name="copy"):
    copy = os.path.join(work, name)
    write_text(os.path.join(copy, "Cargo.toml"), '[workspace]\nmembers = ["crates/*"]\n')
    for package, answer in (("demo", 42), ("singlefs-verification", 42), ("redbase", 50)):
        write_text(os.path.join(copy, "crates", package, "Cargo.toml"), f'[package]\nname = "{package}"\n')
        write_text(os.path.join(copy, "crates", package, "src", "lib.rs"), f"fn answer() -> u32 {{ {answer} }}\n")
    write_text(os.path.join(copy, "crates", "mutations.tsv"), SELFTEST_TABLE)
    return copy


def reuse_states(lines):
    """prove 交的输出行 → {变异名: 现判 / 复用 / 强制现判 / 拒算}：认每条判定那一行末尾的标（文件头「判定库与复用」）。"""
    states = {}
    for line in lines:
        name, tab, _rest = (line[2:] if line.startswith("✗ ") else line).partition("\t")
        found = ROW_STATE.search(line)
        if tab and found:
            states[name] = found.group(1)
    return states


# 判定库那几格的样本（文件头「判定库与复用」）：demo 一个包两条测试、两条变异；假 cargo 照源码里那两段原文在不在判两条测试的红绿
REUSE_SOURCE = ("pub fn answer() -> u32 { 42 }\n"
                "pub fn other() -> u32 {\n"
                "    let base = 7;\n"
                "    base\n"
                "}\n"
                "\n"
                "#[cfg(test)]\n"
                "mod tests {\n"
                "    #[test]\n"
                "    fn answer_is_42() { assert_eq!(super::answer(), 42); }\n"
                "    #[test]\n"
                "    fn other_is_7() { assert_eq!(super::other(), 7); }\n"
                "}\n")
REUSE_TABLE = ("# 判定库那几格的样本\n"
               "answer-41\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer_is_42\tanswer_is_42\n"
               "other-8\tcrates/demo/src/lib.rs\tlet base = 7;\tlet base = 8;\t-p demo --lib -- other_is_7\tother_is_7\n")
# 判定库那几格的假 cargo：每起一次往 @LOG@ 记一行参数；`--` 之后头一个不以 - 起头的词是过滤词（照 libtest 按全名子串选）；
# answer_is_42 在源码里有「{ 42 }」时 ok（PROVE_RED_SELFTEST_TEST_WEAKENED 设了一律 ok：演点名的测试被改弱、同一把键判出没红），other_is_7 在有「let base = 7;」时 ok
REUSE_FAKE_CARGO = r'''#!/usr/bin/env bash
printf '%s\n' "$*" >> @LOG@
filter=''; separated=''
for argument in "$@"; do
  if [[ -n "$separated" && -z "$filter" && "$argument" != -* ]]; then filter="$argument"; fi
  if [[ "$argument" == -- ]]; then separated=1; fi
done
source_file=crates/demo/src/lib.rs
echo "   Compiling demo v0.1.0 ($PWD/crates/demo)"
passed=0; failed=0; results=''
for name in answer_is_42 other_is_7; do
  if [[ -n "$filter" && "tests::$name" != *"$filter"* ]]; then continue; fi
  status=FAILED
  if [[ "$name" == answer_is_42 ]] && { grep -qF '{ 42 }' "$source_file" || [[ -n "${PROVE_RED_SELFTEST_TEST_WEAKENED:-}" ]]; }; then status=ok; fi
  if [[ "$name" == other_is_7 ]] && grep -qF 'let base = 7;' "$source_file"; then status=ok; fi
  results+="test tests::$name ... $status"$'\n'
  if [[ "$status" == ok ]]; then passed=$((passed + 1)); else failed=$((failed + 1)); fi
done
printf 'running %s tests\n%s\n' "$((passed + failed))" "$results"
if (( failed )); then printf 'test result: FAILED. %s passed; %s failed; 0 ignored\n' "$passed" "$failed"; exit 101; fi
printf 'test result: ok. %s passed; 0 failed; 0 ignored\n' "$passed"
'''


# 并行那几格的样本（文件头「判法与次序」的「并行」一条）：demo 一个包五个源文件；假 cargo 照全部源文件里的数判，不点名具体的文件
PARALLEL_SOURCES = {"lib.rs": "fn answer() -> u32 { 42 }\nfn other() -> u32 { 7 }\n", "beta.rs": "fn beta() -> u32 { 42 }\n", "gamma.rs": "fn gamma() -> u32 { 42 }\n",
                    "delta.rs": "fn delta() -> u32 { 42 }\n", "epsilon.rs": "fn epsilon() -> u32 { 42 }\n"}
PARALLEL_TABLE = ("# 并行那几格的样本\n"
                  "p-elsewhere\tcrates/demo/src/beta.rs\t{ 42 }\t{ 47 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "p-caught\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer_caught\tanswer_is_42\n"
                  "p-same-file\tcrates/demo/src/lib.rs\t{ 7 }\t{ 8 }\t-p demo --lib -- answer_same\tanswer_is_42\n"
                  "p-broken\tcrates/demo/src/gamma.rs\t{ 42 }\t{ 46 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "p-notrun\tcrates/demo/src/delta.rs\t{ 42 }\t{ 52 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "p-stale\tcrates/demo/src/epsilon.rs\t{ 42 }\t{ 53 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "p-beta-caught\tcrates/demo/src/beta.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer\tanswer_is_42\n"
                  "p-base-red\tcrates/demo/src/beta.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- baseline_red_filter\tanswer_is_42\n")
# 点名次序；每一行要判成的那个开头（「并行判定与串行逐条相同」那一格另核串行、并行两边都是它）
PARALLEL_EXPECTED = (("p-caught", "p-caught\t抓到\t"), ("p-elsewhere", "✗ p-elsewhere\t没红\t"), ("p-same-file", "p-same-file\t抓到\t"), ("p-broken", "✗ p-broken\t无效\t"),
                     ("p-notrun", "- 本次未跑 p-notrun："), ("p-stale", "✗ p-stale\t没重编\t"))
# 并行那几格的假 cargo：每起一次往 @LOG@ 记一行「工作目录、编译目录、编译并行度、参数」。改坏的那一趟（源文件里有替换文）设了 PROVE_RED_SELFTEST_RENDEZVOUS 就在那里等齐
# PROVE_RED_SELFTEST_RENDEZVOUS_COUNT 路才往下走，等齐记 met、等不齐（30 秒）记 gave-up，之后的不再等；等齐之后读源文件定结局。41（p-caught）多睡 1 秒，让点名在前的跑完得晚：
# 8、41 点名的红；47 点名的绿、别的红；46 编不过；52 点名的绿、打 NOT_RUN；53 全绿而不打 Compiling（没重编）。参数里带 baseline_red_filter 的基线红。
# dep-info 文件名带工作目录的上一层，几路共用编译目录时各写各的（共用编译目录那条开关只红「每路各一份」那一格）
PARALLEL_FAKE_CARGO = r'''#!/usr/bin/env bash
printf '%s\t%s\t%s\t%s\n' "$PWD" "$CARGO_TARGET_DIR" "$CARGO_BUILD_JOBS" "$*" >> @LOG@
sources="$(cat crates/demo/src/*.rs)"
case "$sources" in
  *'{ 41 }'*|*'{ 8 }'*|*'{ 46 }'*|*'{ 47 }'*|*'{ 52 }'*|*'{ 53 }'*)
    rendezvous="${PROVE_RED_SELFTEST_RENDEZVOUS:-}"
    if [[ -n "$rendezvous" ]]; then
      : > "$rendezvous/arrived-$$"
      if [[ ! -e "$rendezvous/gave-up" ]]; then
        for _ in {1..600}; do
          arrived=("$rendezvous"/arrived-*)
          if (( ${#arrived[@]} >= PROVE_RED_SELFTEST_RENDEZVOUS_COUNT )); then : > "$rendezvous/met"; break; fi
          sleep 0.05
        done
        [[ -e "$rendezvous/met" ]] || : > "$rendezvous/gave-up"
      fi
      sources="$(cat crates/demo/src/*.rs)"
    fi
    ;;
esac
green='running 1 test\ntest tests::answer_is_42 ... ok\n\ntest result: ok. 1 passed; 0 failed; 0 ignored\n'
case "$sources" in *'{ 53 }'*) ;; *) echo "   Compiling demo v0.1.0 ($PWD/crates/demo)" ;; esac
mkdir -p "$CARGO_TARGET_DIR/debug/deps"
printf '%s: %s\n' "$CARGO_TARGET_DIR/debug/deps/demo" "$(printf '%s ' "$PWD"/crates/demo/src/*.rs)" > "$CARGO_TARGET_DIR/debug/deps/demo-$(basename "$(dirname "$PWD")").d"
if [[ "$*" == *baseline_red_filter* ]]; then printf 'test tests::answer_is_42 ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored\n'; exit 101; fi
case "$sources" in
  *'{ 46 }'*) printf 'error[E0308]: mismatched types\nerror: could not compile `demo` (lib test) due to 1 previous error\n'; exit 101 ;;
  *'{ 41 }'*) sleep 1; printf 'test tests::answer_is_42 ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored\n'; exit 101 ;;
  *'{ 8 }'*) printf 'test tests::answer_is_42 ... FAILED\n\ntest result: FAILED. 0 passed; 1 failed; 0 ignored\n'; exit 101 ;;
  *'{ 47 }'*) printf 'running 2 tests\ntest tests::answer_is_42 ... ok\ntest tests::other ... FAILED\n\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\n'; exit 101 ;;
  *'{ 52 }'*) printf 'test tests::answer_is_42 ... ok\n---- tests::answer_is_42 stdout ----\nNOT_RUN_NO_GPU_ADAPTER test=tests::answer_is_42 reason=没卡\ntest result: ok. 1 passed; 0 failed\n'; exit 0 ;;
esac
printf "$green"
'''


def parallel_cells(work):
    """并行那十二格（文件头「判法与次序」的「并行」一条）：→ (格数, [失败说明])。同一份副本、同一份假 cargo，点名六行（五个文件）串行（PROVE_RED_LANES=1，就在副本上）与并行（等齐五路）各跑一趟，
    比判定、看各路的工作目录与编译目录；另跑丢掉一路的、一路基线红的、路数写错的，分路判法直接调。样本里没有点名的 #[test]，行键算不出：每趟都现判，不进判定库。"""
    copy = selftest_copy(work, "parallel")
    sources = {name: os.path.join(copy, "crates", "demo", "src", name) for name in PARALLEL_SOURCES}
    for name, text in PARALLEL_SOURCES.items():
        write_text(sources[name], text)
    write_text(os.path.join(copy, "crates", "mutations.tsv"), PARALLEL_TABLE)
    log = os.path.join(work, "parallel-cargo-calls.txt")
    cargo = os.path.join(work, "bin", "parallel-cargo")
    write_text(cargo, PARALLEL_FAKE_CARGO.replace("@LOG@", shlex.quote(log)), executable=True)
    parallel_jobs = "16"
    lanes_base = os.path.join(work, "parallel-lanes-base")
    names = [name for name, _start in PARALLEL_EXPECTED]
    failures = []

    def trial_in_lanes(chosen, lanes, rendezvous=None):
        """一趟证红：(退出码, 输出行, 假 cargo 每次的 [工作目录, 编译目录, 编译并行度, 参数])。"""
        settings = {LANES_VARIABLE: lanes, "CARGO_BUILD_JOBS": parallel_jobs, "CARGO_TARGET_DIR": lanes_base, "PROVE_RED_SELFTEST_RENDEZVOUS": rendezvous,
                    "PROVE_RED_SELFTEST_RENDEZVOUS_COUNT": str(len(PARALLEL_SOURCES)) if rendezvous else None}
        saved = {name: os.environ.get(name) for name in settings}
        for name, value in settings.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        if os.path.exists(log):
            os.unlink(log)
        try:
            code, lines = prove(copy, "demo", chosen, cargo=cargo)
        finally:
            for name, value in saved.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value
        calls = [line.split("\t") for line in open(log, encoding="utf-8").read().splitlines()] if os.path.exists(log) else []
        return code, lines, calls

    def judged_lines(lines):
        """比串行与并行时不看的：「- 并行」那一行、随机种子的复用哨兵行与日志路径。"""
        return [re.sub(r"（日志 [^）]*）", "（日志）", line) for line in lines if not line.startswith("- 并行") and "复用哨兵" not in line]

    rendezvous = os.path.join(work, "parallel-rendezvous")
    os.makedirs(rendezvous)
    serial_code, serial_lines, _serial_calls = trial_in_lanes(names, "1")
    code, lines, calls = trial_in_lanes(names, "8", rendezvous)
    expected_missing = [start for _name, start in PARALLEL_EXPECTED if not any(line.startswith(start) for line in lines)
                        or not any(line.startswith(start) for line in serial_lines)]
    if code != serial_code or code != 1 or judged_lines(lines) != judged_lines(serial_lines) or expected_missing:
        failures.append(f"并行判定与串行逐条相同：两边都退 1、逐行相同、六行各判成写明的结局，实际串行退 {serial_code}、并行退 {code}、缺 {expected_missing}；串行 {judged_lines(serial_lines)}；并行 {judged_lines(lines)}")
    if not os.path.exists(os.path.join(rendezvous, "met")):
        failures.append(f"并行同时起：互不撞文件的五路应当同时在施加变异（假 cargo 等齐五路），实际没等齐，进程起了 {len(calls)} 次，输出 {lines}")
    plan = next((line for line in lines if line.startswith("- 并行：开 ")), "")
    planned = re.match(r"- 并行：开 (\d+) 路.*各路的工作副本与编译目录在 (.+)$", plan)
    lane_count = int(planned.group(1)) if planned else 0
    lanes_root = planned.group(2) if planned else ""
    pairs = {(call[0], call[1]) for call in calls}
    if (lane_count != len({call[0] for call in calls}) or lane_count != len({call[1] for call in calls}) or lane_count != len(pairs) or lane_count < 2
            or any(call[0] == os.path.realpath(copy) or not call[0].startswith(lanes_root) or not call[1].startswith(lanes_root) for call in calls)):
        failures.append(f"并行每路各一份工作副本与编译目录：报 {lane_count} 路，工作目录与编译目录一一对应、都在各路的目录里、不是 --copy 那一份，实际 {sorted(pairs)}，那一行 {plan!r}")
    wanted_jobs = str(max(1, int(parallel_jobs) // max(1, lane_count)))
    if {call[2] for call in calls} != {wanted_jobs}:
        failures.append(f"并行编译并行度：总数 {parallel_jobs}、{lane_count} 路，每路 {wanted_jobs}，实际 {sorted({call[2] for call in calls})}")
    unchanged = all(open(sources[name], encoding="utf-8").read() == text for name, text in PARALLEL_SOURCES.items())
    if not unchanged or not lanes_root or os.path.exists(lanes_root):
        failures.append(f"并行收尾：--copy 那份副本的源码不变、各路的目录删掉，实际源码{'没变' if unchanged else '变了'}、目录 {lanes_root!r} {'还在' if lanes_root and os.path.exists(lanes_root) else '不在'}")
    same_file_directories = {call[0] for call in calls if "answer_caught" in call[3] or "answer_same" in call[3]}
    if len(same_file_directories) != 1:
        failures.append(f"并行撞文件的行同一路：p-caught 与 p-same-file 改同一个文件，应当在同一份工作副本里跑，实际 {sorted(same_file_directories)}")
    if (code != 1 or not any(line.startswith("p-caught\t抓到\t") for line in lines) or not any(line.startswith("✗ p-elsewhere\t没红\t") for line in lines)
            or not any(line.startswith("✗ 点名 6 条") for line in lines)):
        failures.append(f"并行单路红不吃掉：头一路都抓到、别的路有没红，整次退 1、汇总句报红、逐行照列，实际退 {code} {lines}")
    verdict_order = [name for line in lines for name, start in PARALLEL_EXPECTED if line.startswith(start)]
    if verdict_order != names:
        failures.append(f"并行输出按点名次序：判定行的次序应当是 {names}（p-caught 跑完得最晚也排头一个），实际 {verdict_order}")
    # 收回数：自证放一个值进 DROP_LAST_LANE_FOR_SELFTEST，变异那一段丢掉最后派出的那一路
    DROP_LAST_LANE_FOR_SELFTEST.append(True)
    try:
        dropped_code, dropped_lines, _calls = trial_in_lanes(["p-caught", "p-beta-caught"], "8")
    finally:
        DROP_LAST_LANE_FOR_SELFTEST.clear()
    if dropped_code != 1 or not any("派出 2 路、只收回 1 路" in line for line in dropped_lines):
        failures.append(f"并行收回数核对：派出 2 路、收回 1 路，整次退 1、点名两个数，实际退 {dropped_code} {dropped_lines}")
    fake_lanes = [ROWS.Row(number, f"row-{number}", f"crates/demo/src/{path}", "a", "b", arguments, "answer_is_42", ())
                  for number, (path, arguments) in enumerate((("one.rs", "-p demo --features gpu --lib"), ("two.rs", "-p demo --lib"), ("three.rs", "-p demo --features gpu --lib"), ("four.rs", "-p demo --lib")), 1)]
    card_lanes = [[row.name for row in lane] for lane in planned_lanes(fake_lanes, 8)]
    ceiling_lanes = planned_lanes(fake_lanes, 2)
    if card_lanes != [["row-1", "row-3"], ["row-2"], ["row-4"]] or len(ceiling_lanes) != 2 or sum(len(lane) for lane in ceiling_lanes) != 4:
        failures.append(f"并行分路：要卡的两行在同一路、不要卡的各一路，上限 2 时开 2 路、四行都在，实际 {card_lanes}、上限 2 时 {[[row.name for row in lane] for lane in ceiling_lanes]}")
    refused_code, refused_lines, refused_calls = trial_in_lanes(["p-caught", "p-beta-caught"], "abc")
    if refused_code != 2 or refused_calls or not any(LANES_VARIABLE in line for line in refused_lines):
        failures.append(f"并行路数写错：{LANES_VARIABLE}=abc 整次拒绝、退 2、点名这个变量、cargo 一次都不起，实际退 {refused_code}、cargo 起了 {len(refused_calls)} 次 {refused_lines}")
    # 一路基线红（p-base-red 那一路的参数带 baseline_red_filter，别的路基线绿）：两边都退 2、一条变异都不施加（假 cargo 每一次都带 --show-output）、输出逐行相同
    baseline_names = ["p-caught", "p-base-red"]
    serial_baseline = trial_in_lanes(baseline_names, "1")
    parallel_baseline = trial_in_lanes(baseline_names, "8")
    applied = [call for each in (serial_baseline, parallel_baseline) for call in each[2] if "--show-output" not in call[3]]
    if (serial_baseline[0] != 2 or parallel_baseline[0] != 2 or applied or judged_lines(serial_baseline[1]) != judged_lines(parallel_baseline[1])
            or not any("基线（没改源码）就红" in line for line in parallel_baseline[1])):
        failures.append(f"并行基线：有一路基线红，串行、并行都退 2、一条变异都不施加、输出逐行相同，实际退 {serial_baseline[0]}、{parallel_baseline[0]}，施加了 {applied}；串行 {serial_baseline[1]}；并行 {parallel_baseline[1]}")
    return 12, failures


def library_cells(work):
    """判定库那六格（文件头「判定库与复用」，弄坏开关 library-unread、sentinel-unplanned、red-not-recorded、narrowed-recorded、narrowed-sentinel-drawn）：→ (格数, [失败说明])。
    一份副本（与别的格同一份闭包，demo 的源码与变异表换成 REUSE_SOURCE、REUSE_TABLE）、一份假 cargo、库根是自证开头 git init 的那一棵临时仓：
    头一趟（库是空的）、同一份输入第二趟、加一个没人调的函数之后一趟、改 other 函数体之后一趟，照 lib_reuse_checks.py 判复用三问、打规范输出；
    第二趟另核复用哨兵抽一条、强制现判一条；加函数那一趟哨兵取全部（不强制现判）另核 cargo 一次都不起；
    再挑一个种子让强制现判落在 answer-41、假 cargo 演点名的测试被改弱：那一趟判没红、报判定不稳、退 1、照行键记下没红，下一趟 answer-41 现判（上次没判成抓到）。
    缩过域那两格：改两条变异各自被改坏的函数之后缩域跑一趟（两条现判、不入库、判定与汇总句写缩过域），不缩域再跑一趟（两条仍现判），
    再缩域跑一趟（库里有了全域下判的抓到：两条都复用、不抽哨兵、假 cargo 一次都不起）。"""
    copy = selftest_copy(work, "reuse")
    source = os.path.join(copy, "crates", "demo", "src", "lib.rs")
    write_text(source, REUSE_SOURCE)
    write_text(os.path.join(copy, "crates", "mutations.tsv"), REUSE_TABLE)
    log = os.path.join(work, "reuse-cargo-arguments.txt")
    cargo = os.path.join(work, "bin", "reuse-cargo")
    write_text(cargo, REUSE_FAKE_CARGO.replace("@LOG@", shlex.quote(log)), executable=True)
    names = ["answer-41", "other-8"]
    items = {row.name: ROWS.sentinel_item(row) for row in ROWS.read_table(os.path.join(copy, "crates", "mutations.tsv"))[0]}
    count_variable, seed_variable = ROWS.reuse_checks.SENTINEL_COUNT_VARIABLE, ROWS.reuse_checks.SENTINEL_SEED_VARIABLE
    narrowing_variables = tuple(ROWS.admission_module().CRASH_CASE_NARROWING_VARIABLES)
    failures = []

    def trial_with_sentinels(count, seed=None, weakened=False, narrowing=None):
        """一趟证红：(退出码, 输出行, {变异名: 结局}, 假 cargo 起了几次)。count、seed 是这一趟复用哨兵的条数与种子（seed 不给就不设）；
        narrowing 给了是这一趟缩域：{缩域变量: 值}（文件头「判定库与复用」的「缩过域的一趟」），不给两个缩域变量都不设。"""
        saved = {name: os.environ.get(name) for name in (count_variable, seed_variable, "PROVE_RED_SELFTEST_TEST_WEAKENED") + narrowing_variables}
        os.environ[count_variable] = str(count)
        for name in (seed_variable, "PROVE_RED_SELFTEST_TEST_WEAKENED") + narrowing_variables:
            os.environ.pop(name, None)
        if seed is not None:
            os.environ[seed_variable] = str(seed)
        if weakened:
            os.environ["PROVE_RED_SELFTEST_TEST_WEAKENED"] = "1"
        os.environ.update(narrowing or {})
        if os.path.exists(log):
            os.unlink(log)
        try:
            code, lines = prove(copy, "demo", names, cargo=cargo)
        finally:
            for name, value in saved.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value
        calls = open(log, encoding="utf-8").read().splitlines() if os.path.exists(log) else []
        return code, lines, reuse_states(lines), len(calls)

    def replace_once(old, new):
        with open(source, encoding="utf-8") as handle:
            text = handle.read()
        write_text(source, text.replace(old, new, 1))

    first = trial_with_sentinels(1, 5)
    second = trial_with_sentinels(1, 5)
    replace_once("\n#[cfg(test)]", "pub fn spare() -> u32 { 5 }\n\n#[cfg(test)]")   # 闭包外：没人调的函数
    unrelated = trial_with_sentinels(1000)
    replace_once("    base\n}", "    base + 0\n}")   # 闭包内：other-8 改坏的那个函数（other_is_7 的闭包里有它），answer-41 那一条的闭包里没有
    unit = trial_with_sentinels(1, 5)
    runs = (first, second, unrelated, unit)
    passed, problems = ROWS.reuse_checks.report_three_questions(SCRIPT_KEY, first[2], second[2], unrelated[2], unit[2], ["other-8"])
    if not passed or any(each[0] != 0 for each in runs):
        failures.append(f"reuse-three-questions：四趟退 {[each[0] for each in runs]}、结局 {[each[2] for each in runs]}；{'；'.join(problems)}；末一趟输出 {unit[1]}")
    if (second[0] != 0 or sorted(second[2].values()) != ["复用", "强制现判"]
            or not any(line.strip().startswith("· 复用哨兵（证红）：种子 5") for line in second[1])):
        failures.append(f"复用哨兵：同一份输入的第二趟从键相同的两条里抽一条哨兵、强制现判另一条（种子 5、哨兵 1 条），哨兵复用、强制现判与库里同为抓到、退 0，"
                        f"实际退 {second[0]}、结局 {second[2]}、输出 {second[1]}")
    if (unrelated[0] != 0 or unrelated[3] != 0 or unrelated[2] != {"answer-41": "复用", "other-8": "复用"}
            or not any("现判 0 条、复用 2 条、拒算 0 条" in line for line in unrelated[1])
            or not any("现判 2 条、复用 0 条、拒算 0 条" in line for line in first[1])):
        failures.append(f"键相同的都复用：加一个没人调的函数之后两条都复用、假 cargo 一次都不起、成功句报现判 0 条、复用 2 条、拒算 0 条（头一趟报现判 2 条、复用 0 条），"
                        f"实际退 {unrelated[0]}、假 cargo 起了 {unrelated[3]} 次、结局 {unrelated[2]}、输出 {unrelated[1]}；头一趟输出 {first[1]}")

    def forced_under(seed):
        planned = ROWS.reuse_checks.ReuseSentinel(SCRIPT_KEY, SENTINEL_LABEL, count=1, seed=seed)
        planned.plan({items[name]: "抓到" for name in names}, printer=lambda _line: None)
        return planned.forced_item

    seed = next((candidate for candidate in range(256) if forced_under(candidate) == items["answer-41"]), None)
    weakened = trial_with_sentinels(1, seed, weakened=True) if seed is not None else (None, [], {}, 0)
    after = trial_with_sentinels(1000)
    rejudged = [line for line in after[1] if line.startswith("answer-41\t")]
    if (seed is None or weakened[0] != 1 or weakened[2].get("answer-41") != "强制现判"
            or not any(line.startswith("✗ answer-41\t没红\t") for line in weakened[1]) or not any("判定不稳：" in line for line in weakened[1])
            or after[0] != 0 or after[2] != {"answer-41": "现判", "other-8": "复用"} or not any("上次没判成抓到" in line for line in rejudged)):
        failures.append(f"记红撤绿：强制现判落在 answer-41、点名的测试被改弱的那一趟判没红、报判定不稳、退 1，没红照行键记进库，下一趟 answer-41 现判（上次没判成抓到）、other-8 复用、退 0，"
                        f"实际种子 {seed}、那一趟退 {weakened[0]}、结局 {weakened[2]}、输出 {weakened[1]}；下一趟退 {after[0]}、结局 {after[2]}、输出 {after[1]}")

    # 缩过域的一趟（文件头「判定库与复用」的「缩过域的一趟」）：先改两条变异各自被改坏的函数（行键都变、库里没有），缩域跑一趟、再不缩域跑一趟，
    # 再缩域跑一趟（库里有了全域下判的抓到，复用哨兵取 1 条、强制现判 1 条）
    replace_once("pub fn answer() -> u32 { 42 }", "pub fn answer() -> u32 { let _unused = 0; { 42 } }")
    replace_once("    base + 0\n}", "    base + 0 + 0\n}")
    narrowed = trial_with_sentinels(1000, narrowing={"SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO": "3"})
    plain = trial_with_sentinels(1000)
    narrowed_reuse = trial_with_sentinels(1, 5, narrowing={"SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO": "5"})
    narrowed_rows = [line for line in narrowed[1] if line.startswith(("answer-41\t", "other-8\t"))]
    if (narrowed[0] != 0 or narrowed[2] != {"answer-41": "现判", "other-8": "现判"} or len(narrowed_rows) != 2
            or not all("；缩过域（SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3）、不算门禁通过、没入库（现判：" in line for line in narrowed_rows)
            or not any(line.startswith("✓ 点名 2 条：") and "；缩过域（SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3）、不算门禁通过、现判的没入库" in line for line in narrowed[1])
            or plain[0] != 0 or plain[2] != {"answer-41": "现判", "other-8": "现判"} or any("缩过域" in line for line in plain[1])):
        failures.append(f"缩过域·现判不入库：设了 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=3 的一趟两条都现判、都抓到、退 0，两行判定与汇总句写缩过域（变量=值）、不算门禁通过、没入库；"
                        f"同一份输入不设变量再跑一趟两条仍现判、不复用，实际缩域那一趟退 {narrowed[0]}、结局 {narrowed[2]}、输出 {narrowed[1]}；"
                        f"不设变量那一趟退 {plain[0]}、结局 {plain[2]}、输出 {plain[1]}")
    if (narrowed_reuse[0] != 0 or narrowed_reuse[2] != {"answer-41": "复用", "other-8": "复用"} or narrowed_reuse[3] != 0
            or not any("缩过域（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5）：强制现判出的是缩域下的结论" in line and "不抽哨兵" in line for line in narrowed_reuse[1])
            or not any(line.startswith("✓ 点名 2 条：") and "；缩过域（SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5）、不算门禁通过" in line for line in narrowed_reuse[1])):
        failures.append(f"缩过域·复用全域判定：库里有了全域下判的抓到之后设 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=5（复用哨兵取 1 条、强制现判 1 条）再跑：两条都复用、"
                        f"假 cargo 一次都不起、打一行不抽哨兵与缩过域的原因、汇总句写缩过域、退 0，实际退 {narrowed_reuse[0]}、假 cargo 起了 {narrowed_reuse[3]} 次、"
                        f"结局 {narrowed_reuse[2]}、输出 {narrowed_reuse[1]}")
    return 6, failures


# 不领卡那一格的样本表（文件头「不领卡」）：cards-plain 不带 gpu 特性，cards-gpu 带
NO_CARD_LEASE_TABLE = ("# 样本\n"
                       "cards-plain\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer\tanswer_is_42\n"
                       "cards-gpu\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --features gpu --lib -- answer\tanswer_is_42\n")


GPU_ROUTE_TABLE = ("# 开 GPU 补 gpu 特性那一格的样本：两行都不写特性，route-41 点名的测试调到 answer（这一格把它当崩溃判段的选路函数），plain-8 点名的测试调不到\n"
                   "route-41\tcrates/demo/src/lib.rs\t{ 42 }\t{ 41 }\t-p demo --lib -- answer_is_42\tanswer_is_42\n"
                   "plain-8\tcrates/demo/src/lib.rs\tlet base = 7;\tlet base = 8;\t-p demo --lib -- other_is_7\tother_is_7\n")


def gpu_route_cells(work):
    """开 GPU 补 gpu 特性那一格（文件头「判法与次序」的「开 GPU 补 gpu 特性」一条，弄坏开关 gpu-route-feature-unadded）：→ (格数, [失败说明])。
    副本的 demo 换成 REUSE_SOURCE、表换成 GPU_ROUTE_TABLE，假 cargo 同判定库那几格（每起一次记一行参数）；crates-mutation-rows.py 的 GPU_ROUTE_UNITS 临时换成 demo 的 answer。
    多机配置 ENABLE_GPU=1（SINGLEFS_MULTI_HOST_CONFIG 指这一格自己写的一份）：route-41 起的每一次 cargo（基线与变异）`--` 之前带 --features gpu 与 --release，
    plain-8 一次都不带，输出点名补了 route-41、不点 plain-8，两行都抓到、退 0；配置换成 ENABLE_GPU=0 再证一趟：route-41 起的 cargo 不带 --features。"""
    copy = selftest_copy(work, "gpu-route")
    write_text(os.path.join(copy, "crates", "demo", "src", "lib.rs"), REUSE_SOURCE)
    write_text(os.path.join(copy, "crates", "mutations.tsv"), GPU_ROUTE_TABLE)
    log = os.path.join(work, "gpu-route-cargo-arguments.txt")
    cargo = os.path.join(work, "bin", "gpu-route-cargo")
    write_text(cargo, REUSE_FAKE_CARGO.replace("@LOG@", shlex.quote(log)), executable=True)
    configurations = {}
    for state, value in (("on", "1"), ("off", "0")):
        configurations[state] = os.path.join(work, f"gpu-route-{state}", "multi-host.env")
        write_text(configurations[state], f"# 自证用的多机配置：GPU {'开' if value == '1' else '关'}\nENABLE_GPU={value}\n")
    saved_units, saved_configuration = ROWS.GPU_ROUTE_UNITS[0], os.environ.get("SINGLEFS_MULTI_HOST_CONFIG")
    trials = {}
    try:
        ROWS.GPU_ROUTE_UNITS[0] = (("demo", "crates/demo/src/lib.rs", "answer"),)
        for state in ("on", "off"):
            os.environ["SINGLEFS_MULTI_HOST_CONFIG"] = configurations[state]
            # 两条点名测试的函数体每一趟各带一句这一格、这一趟独有的断言说明：行键跟着变，判定库里别的格（与上一趟）判过的同键判定复用不到，
            # 这一趟两行都现判、都起 cargo（复用到了就一次 cargo 都不起，这一格核不到参数）
            write_text(os.path.join(copy, "crates", "demo", "src", "lib.rs"),
                       REUSE_SOURCE.replace("assert_eq!(super::answer(), 42);", f'assert_eq!(super::answer(), 42, "gpu route cell {state}");')
                                   .replace("assert_eq!(super::other(), 7);", f'assert_eq!(super::other(), 7, "gpu route cell {state}");'))
            if os.path.exists(log):
                os.unlink(log)
            code, lines = prove(copy, "demo", ["route-41", "plain-8"], cargo=cargo)
            calls = open(log, encoding="utf-8").read().splitlines() if os.path.exists(log) else []
            trials[state] = (code, lines, [call.split(" -- ")[0].split() for call in calls if "answer_is_42" in call],
                             [call.split(" -- ")[0].split() for call in calls if "other_is_7" in call])
    finally:
        ROWS.GPU_ROUTE_UNITS[0] = saved_units
        if saved_configuration is None:
            os.environ.pop("SINGLEFS_MULTI_HOST_CONFIG", None)
        else:
            os.environ["SINGLEFS_MULTI_HOST_CONFIG"] = saved_configuration
    code, lines, route_calls, plain_calls = trials["on"]
    off_code, off_lines, off_route_calls, _off_plain_calls = trials["off"]
    failures = []
    if (code != 0 or len(route_calls) < 2 or not plain_calls
            or any(words[words.index("--features") + 1:words.index("--features") + 2] != ["gpu"] if "--features" in words else True for words in route_calls)
            or any(words.count("--release") != 1 for words in route_calls)
            or any("--features" in words or "--release" in words for words in plain_calls)
            or not any(line.startswith("- route-41：cargo 参数补上 --features gpu 起（多机配置 ") for line in lines)
            or any(line.startswith("- plain-8：") for line in lines)
            or not any(line.startswith("route-41\t抓到\t") for line in lines) or not any(line.startswith("plain-8\t抓到\t") for line in lines)
            or off_code != 0 or not off_route_calls or any("--features" in words for words in off_route_calls)
            or any(line.startswith("- route-41：cargo 参数补上 --features") for line in off_lines)):
        failures.append(f"开 GPU 补 gpu 特性：行不写特性、配置 ENABLE_GPU=1、点名测试够得着选路函数，起的每一次 cargo 带 --features gpu 与 --release、输出点名补了那一行，"
                        f"够不着的那一行不带；ENABLE_GPU=0 时不带。实际 开：退 {code}、route-41 的 cargo {route_calls}、plain-8 的 cargo {plain_calls}、输出 {lines}；"
                        f"关：退 {off_code}、route-41 的 cargo {off_route_calls}、输出 {off_lines}")
    return 1, failures


def no_card_lease_cells(work, fake_cargo):
    """不领卡那一格（文件头「不领卡」）：→ (格数, [失败说明])。包着 fake_cargo 的假 cargo 把每次起 cargo 的参数、环境里的卡串与 VK_DRIVER_FILES 记进一份事件日志；
    环境里设 GPU_CARDS_BY_PRIORITY=FAKE-CARD-0。cards-plain 与 cards-gpu 各证一趟：每趟起两条 cargo（基线、变异），都不带 --no-run（不另编）、
    收到的卡串都是 FAKE-CARD-0（原样交）、都没有藏卡的 VK_DRIVER_FILES，照常抓到、退 0。弄坏开关 gpu-rows-compiled-apart、gpu-rows-cards-dropped 下 cards-gpu 那一趟判错。"""
    copy = selftest_copy(work, "no-card-lease")
    write_text(os.path.join(copy, "crates", "mutations.tsv"), NO_CARD_LEASE_TABLE)
    events = os.path.join(work, "no-card-lease-events.txt")
    logging_cargo = os.path.join(work, "bin", "no-card-lease-cargo")
    write_text(logging_cargo,
               "#!/usr/bin/env bash\n"
               f"printf 'cargo %s cards=%s vk=%s\\n' \"$*\" \"${{GPU_CARDS_BY_PRIORITY-none}}\" \"${{VK_DRIVER_FILES-unset}}\" >> {shlex.quote(events)}\n"
               f"exec {shlex.quote(fake_cargo)} \"$@\"\n", executable=True)
    failures = []

    def trial(chosen):
        """→ (退出码, 输出行, 事件日志的行)。GPU_CARDS_BY_PRIORITY 只在这一趟里设。"""
        saved = os.environ.get("GPU_CARDS_BY_PRIORITY")
        os.environ["GPU_CARDS_BY_PRIORITY"] = "FAKE-CARD-0"
        if os.path.exists(events):
            os.unlink(events)
        try:
            code, lines = prove(copy, "demo", chosen, cargo=logging_cargo)
        finally:
            if saved is None:
                os.environ.pop("GPU_CARDS_BY_PRIORITY", None)
            else:
                os.environ["GPU_CARDS_BY_PRIORITY"] = saved
        return code, lines, [line for line in open(events).read().splitlines() if line.startswith("cargo")] if os.path.exists(events) else []

    for name in ("cards-plain", "cards-gpu"):
        code, lines, calls = trial([name])
        if (code != 0 or len(calls) != 2 or any(" --no-run" in call for call in calls)
                or any("cards=FAKE-CARD-0 " not in call or not call.endswith("vk=unset") for call in calls)
                or not any(line.startswith(f"{name}\t抓到\t") for line in lines)):
            failures.append(f"不领卡：{name} 起两条 cargo（基线、变异），都不带 --no-run、收到的卡串都是环境里给的 FAKE-CARD-0、都没有藏卡的 VK_DRIVER_FILES，照常抓到、退 0，"
                            f"实际退 {code}、cargo {calls}、输出 {lines}")
    return 1, failures


def trip_build_directory_cells(work, fake_cargo, target_log):
    """编译目录归这一趟三格（文件头「判法与次序」的「编译目录归这一趟」）：→ (格数, [失败说明])。调用方没给 CARGO_TARGET_DIR（这里取下来）。
    ① 每一趟新建：副本里留着别的命令编出的 target（里面 demo 的产物比源码新，沿用就判没重编），连起两趟证红：每一趟的 cargo 看到的编译目录恰好一个、都不是 <副本>/target、
       两趟互不相同，照常抓到、退 0，副本里那份旧 target 原样留着（不是这一趟建的，不用、不删）；
    ② 跑完删：上面两趟与基线红（退 2）那一趟的编译目录，跑完一个都不在了；
    ③ 被打断删：子进程里建一份、装好处理函数、睡着，点名发它 TERM，那份编译目录删了、进程按默认处理停（被 TERM 杀死）。
    弄坏开关 copy-target-reused 下 ① ② 判错，trip-target-kept 下 ② 判错，trip-target-kept-on-stop 下 ③ 判错。"""
    failures = []
    copy = selftest_copy(work, "trip-copy")
    old_target = os.path.join(copy, "target")
    old_artifact = os.path.join(old_target, "fake-build", "demo.rs")
    write_text(old_artifact, "fn answer() -> u32 { 42 }\n")
    write_text(os.path.join(old_target, "debug", "deps", "demo.d"), f"{old_artifact}: {os.path.realpath(copy)}/crates/demo/src/lib.rs\n")
    tomorrow = time.time() + 86400
    os.utime(old_artifact, (tomorrow, tomorrow))
    saved_target = os.environ.pop("CARGO_TARGET_DIR", None)
    try:
        trips = []
        for chosen, crate in ((["caught"], "demo"), (["caught"], "demo"), (["redbase"], "redbase")):
            if os.path.exists(target_log):
                os.unlink(target_log)
            code, lines = prove(copy, crate, chosen, cargo=fake_cargo)
            trips.append((code, lines, set(open(target_log).read().splitlines()) if os.path.exists(target_log) else set()))
        first, second, red = trips
        if (first[0] != 0 or second[0] != 0 or not any("\t抓到\t" in line for line in first[1] + second[1])
                or len(first[2]) != 1 or len(second[2]) != 1 or first[2] == second[2] or old_target in first[2] | second[2] or not os.path.exists(old_artifact)
                or not all(directory.startswith(os.path.join(os.path.realpath(tempfile.gettempdir()), TRIP_BUILD_DIRECTORY_PREFIX)) for directory in first[2] | second[2])):
            failures.append(f"编译目录归这一趟·每一趟新建：副本里留着别的命令编出的 target（产物比源码新）时，没给 CARGO_TARGET_DIR 的每一趟都自己建一份新的，不沿用 <副本>/target、"
                            f"两趟互不相同，照常抓到、退 0，旧 target 原样留着；实际两趟退 {first[0]}、{second[0]}，cargo 看到的编译目录 {sorted(first[2])}、{sorted(second[2])}，"
                            f"旧产物还在 {os.path.exists(old_artifact)}，输出 {first[1] + second[1]}")
        left = sorted(directory for directory in first[2] | second[2] | red[2] if os.path.exists(directory))
        if red[0] != 2 or not red[2] or left:
            failures.append(f"编译目录归这一趟·跑完删：成功两趟与基线红那一趟（退 2）跑完，这一趟自己建的编译目录一个都不留；实际基线红那一趟退 {red[0]}，"
                            f"cargo 看到过 {sorted(first[2] | second[2] | red[2])}，跑完还在的 {left}")
    finally:
        for directory in list(TRIP_BUILD_DIRECTORIES):
            shutil.rmtree(directory, ignore_errors=True)
            TRIP_BUILD_DIRECTORIES.remove(directory)
        if saved_target is not None:
            os.environ["CARGO_TARGET_DIR"] = saved_target
    ready = os.path.join(work, "trip-stop-ready")
    child = os.fork()
    if child == 0:
        try:
            remove_trip_build_directories_when_stopped()
            held_directory = new_trip_build_directory()
            write_text(os.path.join(held_directory, "artifact"), "x")
            write_text(ready, held_directory)
            time.sleep(60)
        finally:
            os._exit(0)
    deadline = time.monotonic() + 30
    while not os.path.exists(ready) and time.monotonic() < deadline:
        time.sleep(0.05)
    os.kill(child, signal.SIGTERM if os.path.exists(ready) else signal.SIGKILL)
    _pid, status = os.waitpid(child, 0)
    held = open(ready).read() if os.path.exists(ready) else ""
    if not held or os.path.exists(held) or not os.WIFSIGNALED(status) or os.WTERMSIG(status) != signal.SIGTERM:
        failures.append(f"编译目录归这一趟·被打断删：这一趟自己建了编译目录的进程收到 TERM，处理函数删了它再按默认处理停（被 TERM 杀死）；"
                        f"实际 就绪 {bool(held)}、编译目录 {held} 还在 {bool(held) and os.path.exists(held)}、退出状态 {status}")
        if held:
            shutil.rmtree(held, ignore_errors=True)
    return 3, failures


def selftest():
    # 读写判定库：只在临时 prune-server 底下跑（文件头用法一段；scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service）
    client = ROWS.mutation_units.Library.client()
    service_problem = client.selftest_service_problem()
    if service_problem:
        print(f"  ✗ prove-red 自证没跑：{service_problem}")  # gate-lint:detail
        print(f"  → 怎么办：{client.SELFTEST_SERVICE_REMEDY}：python3 scripts/prune-server/prune_server_client_selftest.py --with-temporary-service -- "
              "bash scripts/mutation/prove-red.sh --selftest")
        return 2
    work = tempfile.mkdtemp(prefix="prove-red-selftest-")
    # 判法单元指纹与 Rust 词法单元的磁盘缓存放这一趟自己的临时目录，不碰真仓那一份（crates-mutation-rows.py 文件头「判法单元」末段）
    cache_variables = {ROWS.JUDGING_CACHE_DIRECTORY_VARIABLE: os.path.join(work, "judging-units-cache"),
                       "FUNCTION_UNITS_LEX_CACHE_DIRECTORY": os.path.join(work, "rust-lex-cache")}
    saved_caches = {name: os.environ.get(name) for name in cache_variables}
    os.environ.update(cache_variables)
    # 调用方环境里的卡串不进各格：不领卡那一格自己设
    saved_card_variables = {name: os.environ.pop(name, None) for name in ("GPU_CARDS_BY_PRIORITY",)}
    # 调用方给的 CARGO_TARGET_DIR 不进各格（多机可用：服务在第二台给每趟一份编译目录、本机不给）：假 cargo 照「编译目录 + 包名」放假产物、按修改时刻判新不新鲜，
    #   几份假副本共用一个编译目录时，后建的副本编出的改坏产物比 copy 那份的源码新、顶掉它，之后 copy 上的基线就红，结局随派到哪台变。
    #   各格照不设算（每一趟自己建一份新的编译目录、跑完删）；要共用编译目录的那几格自己设、自己换回。弄坏开关 inherited-target-kept 照用调用方的，没给就替它设一份共用的
    inherited_target = os.environ.pop("CARGO_TARGET_DIR", None)
    if BROKEN == "inherited-target-kept":
        os.environ["CARGO_TARGET_DIR"] = inherited_target or os.path.join(work, "inherited-target")
    try:
        copy = selftest_copy(work)
        CLOSURE_ROOT_FOR_SELFTEST.append(selftest_copy(work, "closure-main"))   # 各格的假副本照这棵假主工作区核缺不缺（闭包只有 Cargo.toml 与 crates）
        # 判定库的库根（文件头「判定库与复用」）：一棵自己 git init 的临时仓，库在临时服务里另占一份命名空间、开跑时是空的
        library_root = os.path.join(work, "library-root")
        os.makedirs(library_root)
        initialized = subprocess.run(["git", "init", "-q", library_root], capture_output=True, text=True, stdin=subprocess.DEVNULL)
        if initialized.returncode != 0:
            print(f"  ✗ 自检：判定库的库根 {library_root} git init 退 {initialized.returncode}：{(initialized.stdout + initialized.stderr).strip()}")  # gate-lint:detail
            print("  → 看 git 在不在 PATH 上、临时目录可不可写；判定库那几格要一棵 git 工作树当库根")
            return 1
        LIBRARY_ROOT_FOR_SELFTEST.append(library_root)
        fake_cargo = os.path.join(work, "bin", "cargo")
        target_log = os.path.join(work, "target-directories.txt")
        cargo_log = os.path.join(work, "cargo-arguments.txt")
        lock_log = os.path.join(work, "copy-lock.txt")
        # 假 cargo 照源码里那个数打 libtest / cargo 的原样形态：42 基线全绿；45 是 #[should_panic] 的用例没 panic
        # （结果行在名字与「 ... 」之间多一段「 - should panic」，cargo 收尾还打一行 error: test failed）；46 替换之后编不过；
        # 47 点名的测试绿、别的测试红（同样带 error: test failed 与 test result: 行）；别的数照旧只打点名的那一行 FAILED。
        # 读哪份源码看 -p：-p <包> 读 crates/<包>/src/lib.rs（redbase 那个包的源码是 50，基线就红）。
        # 每起一次先往 target_log 记一行它看到的 CARGO_TARGET_DIR（编译目录那两格核它），往 cargo_log 记一行参数（验证包那两格核它起没起）
        write_text(fake_cargo,
                   "#!/usr/bin/env bash\n"
                   f"printf '%s\\n' \"${{CARGO_TARGET_DIR-<未设>}}\" >> {shlex.quote(target_log)}\n"
                   f"printf '%s\\n' \"$*\" >> {shlex.quote(cargo_log)}\n"
                   # 副本锁那一格：起的时候副本根（当前目录）上的锁被人拿着记 held，没有锁文件或拿得到记 free
                   f"if [[ -e {shlex.quote(COPY_LOCK.COPY_LOCK_NAME)} ]] && ! flock -n {shlex.quote(COPY_LOCK.COPY_LOCK_NAME)} true 2>/dev/null; then echo held; else echo free; fi >> {shlex.quote(lock_log)}\n"
                   "echo \"stamp=${PROVE_RED_SELFTEST_STAMP-none}\"\n"   # 日志目录那一格靠它认出每个基线日志是哪一趟写的
                   "package=demo; previous=''\n"
                   "for argument in \"$@\"; do [[ \"$previous\" == -p ]] && package=\"$argument\"; previous=\"$argument\"; done\n"
                   "source_file=\"crates/$package/src/lib.rs\"\n"
                   # 照 cargo 按 mtime 判新不新鲜：编译目录里这个包的产物不在、或源码比它新，才重编（拷一份当产物、照真 cargo 打 Compiling 行）；
                   # 测试跑的是产物，不是此刻的源码（编译目录里留着比源码新的旧产物时，测试跑在旧产物上）
                   "build_directory=\"${CARGO_TARGET_DIR:-target}/fake-build\"; mkdir -p \"$build_directory\"; artifact=\"$build_directory/$package.rs\"\n"
                   "if [[ ! -e \"$artifact\" || \"$source_file\" -nt \"$artifact\" ]]; then cp \"$source_file\" \"$artifact\"; echo \"   Compiling $package v0.1.0 ($PWD/crates/$package)\"\n"
                   # 照 rustc 写 dep-info（<编译目录>/debug/deps/<包>.d 列出读过的源文件）：没重编那一格靠它认出这个文件是这一趟编译读的
                   "  mkdir -p \"${CARGO_TARGET_DIR:-target}/debug/deps\"; echo \"$artifact: $PWD/$source_file\" > \"${CARGO_TARGET_DIR:-target}/debug/deps/$package.d\"; fi\n"
                   # --no-run（弄坏开关 gpu-rows-compiled-apart 下那一条只编不跑的）：编过了只打 Executable 行退 0，替换之后编不过（46）照打编译错误退 101
                   "if [[ \" $* \" == *' --no-run '* ]]; then\n"
                   "  case \"$(cat \"$artifact\")\" in\n"
                   "  *'{ 46 }'*) printf 'error[E0308]: mismatched types\\n --> crates/demo/src/lib.rs:1:22\\n\\n"
                   "error: could not compile \\x60demo\\x60 (lib test) due to 1 previous error\\n'; exit 101 ;;\n"
                   "  esac\n"
                   "  echo '  Executable unittests src/lib.rs'; exit 0\n"
                   "fi\n"
                   "case \"$(cat \"$artifact\")\" in\n"
                   "*'{ 42 }'*) printf 'running 2 tests\\ntest tests::answer_is_42 ... ok\\ntest tests::answer_panics - should panic ... ok\\n\\n"
                   "test result: ok. 2 passed; 0 failed; 0 ignored\\n'; exit 0 ;;\n"
                   "*'{ 45 }'*) printf 'running 2 tests\\ntest tests::answer_is_42 ... ok\\ntest tests::answer_panics - should panic ... FAILED\\n\\n"
                   "failures:\\n\\n---- tests::answer_panics stdout ----\\nnote: test did not panic as expected at crates/demo/src/lib.rs:9:5\\n\\n"
                   "failures:\\n    tests::answer_panics\\n\\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\\n\\n"
                   "error: test failed, to rerun pass \\x60-p demo --lib\\x60\\n'; exit 101 ;;\n"
                   "*'{ 46 }'*) printf 'error[E0308]: mismatched types\\n --> crates/demo/src/lib.rs:1:22\\n\\n"
                   "error: could not compile \\x60demo\\x60 (lib test) due to 1 previous error\\n'; exit 101 ;;\n"
                   "*'{ 47 }'*) printf 'running 2 tests\\ntest tests::answer_is_42 ... ok\\ntest tests::answer_panics - should panic ... FAILED\\n\\n"
                   "failures:\\n    tests::answer_panics\\n\\ntest result: FAILED. 1 passed; 1 failed; 0 ignored\\n\\n"
                   "error: test failed, to rerun pass \\x60-p demo --lib\\x60\\n'; exit 101 ;;\n"
                   # 52：编过了、失败在链接（头一条变异链接不过的样子：没有 error[E<码>]，有 error: linking with 与 rust-lld 的报错）
                   "*'{ 52 }'*) printf 'error: linking with \\x60cc\\x60 failed: exit status: 1\\n  = note: rust-lld: error: undefined hidden symbol: anon.0123456789abcdef.0.llvm.1\\n\\n"
                   "error: could not compile \\x60demo\\x60 (lib test) due to 1 previous error\\n'; exit 101 ;;\n"
                   "*) echo 'test tests::answer_is_42 ... FAILED'; exit 101 ;;\n"
                   "esac\n", executable=True)
        os.environ["PROVE_RED_LOG_DIRECTORY"] = os.path.join(work, "logs")
        failures = []
        checked_cases = 0

        def cargo_calls():
            calls = open(cargo_log).read().splitlines() if os.path.exists(cargo_log) else []
            if os.path.exists(cargo_log):
                os.unlink(cargo_log)
            return calls

        code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        checked_cases += 1
        if code != 0 or not any("\t抓到\t" in line for line in lines):
            failures.append(f"会红的变异应当抓到、退 0，实际 {code} {lines}")
        # 拒算（文件头「判定库与复用」）：样本里没有点名的那条 #[test]，行键算不出：照跑、打一行为什么、那一行标（拒算）、成功句报拒算 1 条
        checked_cases += 1
        if (not any(line.startswith("- 拒算 caught：行键算不出") for line in lines) or reuse_states(lines).get("caught") != "拒算"
                or not any("现判 0 条、复用 0 条、拒算 1 条" in line for line in lines)):
            failures.append(f"拒算：行键算不出的行照跑、打一行为什么、那一行标（拒算）、成功句报现判 0 条、复用 0 条、拒算 1 条，实际 {lines}")
        checked_cases += 1
        if open(os.path.join(copy, "crates", "demo", "src", "lib.rs")).read() != "fn answer() -> u32 { 42 }\n":
            failures.append("跑完没把源码写回原样")
        code, lines = prove(copy, "demo", ["caught", "untargeted"], cargo=fake_cargo)
        checked_cases += 1
        if code != 2 or not any("没挑目标" in line for line in lines):
            failures.append(f"不挑目标的变异行应当整次拒绝、退 2，实际 {code} {lines}")
        code, lines = prove(copy, "demo", ["layer"], cargo=fake_cargo)
        checked_cases += 1
        if code != 0 or not any(line.startswith("layer\t抓到\t") for line in lines):
            failures.append(f"--test 目标名带 crash_point_replay 而包不是 验证包：照跑、判抓到、退 0（只按包判），实际 {code} {lines}")
        code, lines = prove(copy, "demo", ["panics"], cargo=fake_cargo)
        checked_cases += 1
        if code != 0 or not any(line.startswith("panics\t抓到\t") for line in lines):
            failures.append(f"#[should_panic] 的用例红了（结果行带「 - should panic」、收尾有 error: test failed）应当判抓到、退 0，实际 {code} {lines}")
        code, lines = prove(copy, "demo", ["broken"], cargo=fake_cargo)
        checked_cases += 1
        if code != 1 or not any(line.startswith("✗ broken\t无效\t") for line in lines):
            failures.append(f"替换之后编不过（error[E…] 与 could not compile）应当判无效、退 1，实际 {code} {lines}")
        code, lines = prove(copy, "demo", ["elsewhere"], cargo=fake_cargo)
        checked_cases += 1
        if code != 1 or not any(line.startswith("✗ elsewhere\t没红\t") for line in lines):
            failures.append(f"编过了、别的测试红而点名的没红（有 error: test failed 也有 test result:）应当判没红、不判无效，实际 {code} {lines}")
        # 链接失败（文件头「判法与次序」⑤）：替换之后编过了、失败在链接（有 error: linking with 与 rust-lld、没有 error[E<码>]）：单列一档「链接失败」、退 1、写出路（删编译目录重跑），不判无效
        code, lines = prove(copy, "demo", ["linkfail"], cargo=fake_cargo)
        checked_cases += 1
        if (code != 1 or not any(line.startswith("✗ linkfail\t链接失败\t") and "删掉这一趟用的编译目录" in line for line in lines)
                or any("\t无效\t" in line for line in lines)):
            failures.append(f"链接失败：替换之后编过了、失败在链接（有 error: linking with 与 rust-lld、没有 error[E<码>]）应当单列「链接失败」、写出路、退 1，不判无效，实际 {code} {lines}")
        code, lines = prove(copy, "redbase", ["redbase"], cargo=fake_cargo)
        checked_cases += 1
        if code != 2 or not any("基线（没改源码）就红" in line for line in lines) or any(line.startswith("redbase\t抓到\t") for line in lines):
            failures.append(f"基线（没改源码）就红：停下、退 2、一条变异都不记抓到，实际 {code} {lines}")
        # 起 cargo 之前判 RocksDB 预编库的取值：另一份副本里放一份依赖 rocksdb 的 验证包清单、进程环境里四个取值是空的，假 cargo 不带 entry 认的标记
        #   ⇒ 整次拒绝、退 2、假 cargo 一次都没起（别的格副本里没有这份清单，entry 报不判）
        rocksdb_copy = selftest_copy(work, "rocksdb")
        write_text(os.path.join(rocksdb_copy, "crates", "singlefs-verification", "Cargo.toml"),
                   '[package]\nname = "singlefs-verification"\nversion = "0.0.0"\n[dependencies]\nrocksdb = { version = "=0.25.0", optional = true }\n')
        saved_values = {name: os.environ.get(name) for name in ("ROCKSDB_LIB_DIR", "ROCKSDB_STATIC", "SNAPPY_LIB_DIR", "SNAPPY_STATIC")}
        os.environ.update({name: "" for name in saved_values})
        try:
            cargo_calls()
            code, lines = prove(rocksdb_copy, "demo", ["caught"], cargo=fake_cargo)
            calls = cargo_calls()
        finally:
            for name, value in saved_values.items():
                if value is None:
                    os.environ.pop(name, None)
                else:
                    os.environ[name] = value
        checked_cases += 1
        if code != 2 or calls or not any("rocksdb-prebuilt entry：" in line for line in lines):
            failures.append(f"副本里的 验证包依赖 rocksdb 而进程环境里没有预编库的取值：应当在起 cargo 之前整次拒绝、退 2、假 cargo 一次都不起，实际退 {code}、假 cargo 起了 {len(calls)} 次 {lines}")
        # 主工作区拿临时目录里一棵同样形态的树（有 crates/mutations.tsv）顶替、经 main_root 交给 prove：自证从哪棵树起（仓里、仓副本、符号链接拼的草稿树）都判得出这一格
        fake_main = selftest_copy(work, "main")
        code, lines = prove(fake_main, "demo", ["caught"], cargo=fake_cargo, main_root=fake_main)
        checked_cases += 1
        if code != 2 or not any("主工作区" in line for line in lines):
            failures.append(f"--copy 指主工作区应当以「主工作区」为由拒绝，实际 {code} {lines}")
        # 副本缺不缺那两格（文件头「判法与次序」①，弄坏开关 copy-closure-unchecked）：闭包照另一棵假主工作区算（它的测试代码 include_str! 读仓根的 litmus/never.litmus），
        #   缺 litmus/ 的副本整次拒绝、退 2、点名 litmus/never.litmus、出路指到 minimal-copy.py、cargo 一次都不起；带上 litmus/ 的副本照跑、判抓到、退 0
        litmus_main = selftest_copy(work, "litmus-main")
        write_text(os.path.join(litmus_main, "crates", "demo", "tests", "litmus_files.rs"), 'const NEVER: &str = include_str!("../../../litmus/never.litmus");\n')
        write_text(os.path.join(litmus_main, "litmus", "never.litmus"), "C never\n")
        short_copy = selftest_copy(work, "litmus-short")
        full_copy = selftest_copy(work, "litmus-full")
        write_text(os.path.join(full_copy, "litmus", "never.litmus"), "C never\n")
        cargo_calls()
        code, lines = prove(short_copy, "demo", ["caught"], cargo=fake_cargo, closure_root=litmus_main)
        calls = cargo_calls()
        checked_cases += 1
        if (code != 2 or calls or not any("litmus/never.litmus" in line for line in lines)
                or not any(line.startswith("→") and "minimal-copy.py" in line for line in lines)):
            failures.append(f"副本缺闭包里的条目（litmus/never.litmus）：整次拒绝、退 2、点名它、出路指到 minimal-copy.py、cargo 一次都不起，"
                            f"实际退 {code}、cargo 起了 {len(calls)} 次 {lines}")
        code, lines = prove(full_copy, "demo", ["caught"], cargo=fake_cargo, closure_root=litmus_main)
        cargo_calls()
        checked_cases += 1
        if code != 0 or not any("\t抓到\t" in line for line in lines):
            failures.append(f"副本带全闭包里的条目（连 litmus/never.litmus）：照跑、判抓到、退 0，实际 {code} {lines}")
        # 验证包的行照跑（能不能起由项目代码闸与统一任务服务判，脚本不看环境变量另判）
        cargo_calls()
        code, lines = prove(copy, "singlefs-verification", ["checker"], cargo=fake_cargo)
        calls = cargo_calls()
        checked_cases += 1
        if code != 0 or not any(line.startswith("checker\t抓到\t") for line in lines) or not any("-p singlefs-verification" in call for call in calls):
            failures.append(f"验证包的行照跑、判抓到、退 0，实际退 {code}、cargo 的参数 {calls} {lines}")
        # 验证包只点名集成测试目标的行那一格（文件头「判法与次序」的「验证包的测试目标不编装置二进制」一条，弄坏开关 no-default-features-unadded）：
        #   点名 --test 的 verification-test 与点名 --lib 的 checker 两行：verification-test 起的每一次 cargo（基线与变异）`--` 之前带 --no-default-features、只带一个，
        #   checker 一次都不带；输出点名补了 verification-test、不点 checker；两行都抓到、退 0
        cargo_calls()
        code, lines = prove(copy, "singlefs-verification", ["verification-test", "checker"], cargo=fake_cargo)
        verification_sides = [words[:words.index("--")] if "--" in words else words for words in (call.split() for call in cargo_calls())]
        target_sides = [words for words in verification_sides if "verification_target" in words]
        library_sides = [words for words in verification_sides if "--lib" in words]
        checked_cases += 1
        if (code != 0 or len(target_sides) < 2 or any(words.count("--no-default-features") != 1 for words in target_sides)
                or len(library_sides) < 2 or any("--no-default-features" in words for words in library_sides)
                or not any(line.startswith("- verification-test：cargo 参数补上 --no-default-features 起") for line in lines)
                or any(line.startswith("- checker：") for line in lines)
                or not any(line.startswith("verification-test\t抓到\t") for line in lines) or not any(line.startswith("checker\t抓到\t") for line in lines)):
            failures.append(f"验证包只点名集成测试目标的行：起的每一次 cargo（基线与变异）`--` 之前带一个 --no-default-features，点名 --lib 的行一次都不带，"
                            f"输出点名补了 verification-test、不点 checker，两行都抓到、退 0，实际退 {code}、点名 --test 的 {target_sides}、点名 --lib 的 {library_sides} {lines}")
        # 重测试目标那一格（文件头「判法与次序」的「重测试目标」一条，弄坏开关 release-unforced）：登记表换成这一格自己造的一份（只登记 demo 的集成测试目标
        #   demo_crash_stream），点名带 gpu 特性的、点名登记目标的与两样都不中的 caught 三行：前两行起的每一次 cargo（基线与变异）`--` 之前带 --release、只带一个，
        #   caught 那两次不带；输出逐行点名补了哪两行、不点 caught；三行都抓到、退 0
        release_registry = os.path.join(work, "release-registry")
        write_text(os.path.join(release_registry, ".claude", "gate.d", "stage-inputs.tsv"),
                   "crash-case:demo-stream\t-\ttest=demo:demo_crash_stream:answer_is_42 count-line=DEMO\t# 自证样本\n")
        cargo_calls()
        code, lines = prove(copy, "demo", ["caught", "release-gpu", "release-crash"], cargo=fake_cargo, registry_root=release_registry)
        release_calls = cargo_calls()
        cargo_sides = [words[:words.index("--")] if "--" in words else words for words in (call.split() for call in release_calls)]
        hit_sides = [words for words in cargo_sides if "gpu" in words or "demo_crash_stream" in words]
        plain_sides = [words for words in cargo_sides if "gpu" not in words and "demo_crash_stream" not in words]
        checked_cases += 1
        if (code != 0 or len(hit_sides) != 4 or any(words.count("--release") != 1 for words in hit_sides)
                or len(plain_sides) != 2 or any("--release" in words for words in plain_sides)
                or not any(line.startswith("- release-gpu：cargo 参数补上 --release 起（gpu 特性 gpu）") for line in lines)
                or not any(line.startswith("- release-crash：cargo 参数补上 --release 起（crash-case 目标 demo:demo_crash_stream") for line in lines)
                or any(line.startswith("- caught：") for line in lines)):
            failures.append(f"命中重测试目标的行（gpu 特性、crash-case 目标）起的每一次 cargo 都带 --release、只带一个，不命中的不带，输出点名补了哪两行，"
                            f"实际退 {code}、cargo 的参数 {release_calls} {lines}")
        # 切表两格：点名的行不成形整次拒绝；切出来的行与 read_table 逐行相同
        code, lines = prove(copy, "demo", ["noexpected"], cargo=fake_cargo)
        checked_cases += 1
        if code != 2 or not any("不成形" in line and "必须红的测试名" in line for line in lines):
            failures.append(f"点名的行不成形（必须红的测试名空着）：整次拒绝、写明哪一段空着、退 2，实际 {code} {lines}")
        ours = sorted(table_rows(copy)[0])
        theirs = sorted({row.name for row in ROWS.read_table(os.path.join(copy, "crates", "mutations.tsv"))[0]})
        checked_cases += 1
        if ours != theirs:
            failures.append(f"切表与 read_table 逐行相同：这里切出 {ours}，read_table 切出 {theirs}")
        # 判法那一格：同一组样本，这里判的档与 verdict_of_run 判的逐行相同
        differing = []
        for description, sample_code, sample_output, expected in SELFTEST_VERDICT_SAMPLES:
            row = ROWS.Row(1, "sample", "crates/demo/src/lib.rs", "a", "b", "-p demo --lib", expected, ())
            ours = verdict_of(row, sample_code, sample_output)[0]
            theirs = ROWS.verdict_of_run(row, sample_code, sample_output, environment=os.environ)[0]
            if ours != theirs:
                differing.append(f"{description}：这里判 {ours}，verdict_of_run 判 {theirs}")
        checked_cases += 1
        if differing:
            failures.append(f"判法与 verdict_of_run 对同一组样本逐行相同（{len(SELFTEST_VERDICT_SAMPLES)} 条），对不上的：{'；'.join(differing)}")
        # 副本锁那一格（文件头「判法与次序」⓪）：证红跑的时候副本根上拿着锁（假 cargo 每次起都试拿一次，记 held / free）、跑完放掉、锁文件不留；
        #   锁被别人占着（这里自己先拿着，演另一趟证红）时整次拒绝、退 2、cargo 一次都不起
        if os.path.exists(lock_log):
            os.unlink(lock_log)
        lock_wrong = []
        code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        seen = open(lock_log).read().split() if os.path.exists(lock_log) else []
        if code != 0 or not seen or set(seen) != {"held"}:
            lock_wrong.append(f"跑的时候假 cargo 每次都应当看到锁被拿着（held），实际 {seen}、退 {code}")
        if os.path.exists(os.path.join(copy, COPY_LOCK.COPY_LOCK_NAME)):
            lock_wrong.append("跑完锁文件还在")
        if os.path.exists(os.path.join(copy, COPY_LOCK.COPY_OWNER_NAME)):
            lock_wrong.append("跑完持有者记录还在")
        cargo_calls()
        held = COPY_LOCK.hold_copy_lock(copy)
        try:
            code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
            calls = cargo_calls()
        finally:
            if held is not None:
                COPY_LOCK.release_copy_lock(copy, held)
        if held is None or code != 2 or calls or not any("副本上有证红在跑" in line for line in lines):
            lock_wrong.append(f"锁被占着时应当拒绝、退 2、cargo 一次都不起，实际退 {code}、cargo 起了 {len(calls)} 次 {lines}（自证{'拿到了' if held is not None else '没拿到'}锁）")
        checked_cases += 1
        if lock_wrong:
            failures.append("证红跑的时候副本根上拿着副本锁、跑完放掉；锁被占着时拒绝、退 2、cargo 一次都不起：" + "；".join(lock_wrong))
        # 跨机那一格（apply-writer-patch.py 文件头「副本锁」的跨机一段，弄坏开关 APPLY_WRITER_PATCH_BREAK=copy-owner-ignored）：副本根上有别的机器的持有者记录、
        #   刚刷新过（flock 跨机不一定互斥，这里自己不拿 flock 演那种情形）：整次拒绝、退 2、cargo 一次都不起；记录早过过期线就接手照跑，跑完记录不留
        owner_path = os.path.join(copy, COPY_LOCK.COPY_OWNER_NAME)
        remote_wrong = []
        write_text(owner_path, "other-machine-selftest 4242 selftest-token\n")
        cargo_calls()
        code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        calls = cargo_calls()
        if code != 2 or calls or not any("上已有一趟证红在跑" in line for line in lines):   # 拒绝句是「副本 <路径> 上已有一趟证红在跑」，路径夹在中间
            remote_wrong.append(f"别的机器上的证红占着（持有者记录刚刷新过）时应当拒绝、退 2、cargo 一次都不起，实际退 {code}、cargo 起了 {len(calls)} 次 {lines}")
        stale_at = time.time() - 3 * COPY_LOCK.COPY_OWNER_STALE_SECONDS
        os.utime(owner_path, (stale_at, stale_at))
        code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        cargo_calls()
        if code != 0 or os.path.exists(owner_path):
            remote_wrong.append(f"别的机器上的持有者记录早过过期线时应当接手照跑、退 0、跑完记录不留，实际退 {code}、记录{'还在' if os.path.exists(owner_path) else '已删'} {lines}")
        if os.path.exists(owner_path):
            os.unlink(owner_path)
        checked_cases += 1
        if remote_wrong:
            failures.append("别的机器上的证红占着副本时拒绝、过期的接手：" + "；".join(remote_wrong))
        # 日志目录那一格（文件头「判法与次序」，弄坏开关 shared-log-directory）：同一份副本上先后起两趟（假 cargo 在输出里打 stamp=<这一趟的标记>），
        #   每趟的基线日志写在自己的目录里：两份基线日志各带自己那趟的标记，后一趟没有把前一趟的盖掉
        log_wrong = []
        logs_root = os.environ["PROVE_RED_LOG_DIRECTORY"]
        shutil.rmtree(logs_root, ignore_errors=True)
        run_stamps = ("stamp-sparrow", "stamp-heron")
        for run_stamp in run_stamps:
            os.environ["PROVE_RED_SELFTEST_STAMP"] = run_stamp
            code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
            if code != 0:
                log_wrong.append(f"标记 {run_stamp} 那一趟应当抓到、退 0，实际 {code} {lines}")
        os.environ.pop("PROVE_RED_SELFTEST_STAMP", None)
        cargo_calls()
        baseline_stamps = sorted(re.search(r"stamp=(\S+)", open(os.path.join(directory, "baseline.log"), encoding="utf-8").read()).group(1)
                                 for directory, _subdirectories, names in os.walk(logs_root) if "baseline.log" in names)
        checked_cases += 1
        if baseline_stamps != sorted(run_stamps):
            log_wrong.append(f"两趟各写自己的基线日志，日志目录下读回的标记应当是 {sorted(run_stamps)}，实际 {baseline_stamps}")
        if log_wrong:
            failures.append("同一份副本上先后起两趟，每趟读的是自己的基线日志：" + "；".join(log_wrong))
        # 被杀那一格（文件头「判法与次序」的「还原日志」一条，弄坏开关 no-restore-journal）：另一份副本上，假 cargo 在源码是改坏的样子时记下自己的进程号、睡着不退；
        #   这一格 fork 出的子进程在那份副本上跑 prove，看到记号就对这个子进程发 SIGKILL（finally 不跑，源码留在改坏的样子，与撞内存上限被杀同形），
        #   再停掉睡着的那个假 cargo（两个都是这一格自己起的进程号）。同一份副本上再起一趟：源码要被写回、报写回的那一行、基线照常跑
        #   （假 cargo 起了带 --show-output 的那一次）、不以命中 0 次退 2、还原日志删掉
        killed_copy = selftest_copy(work, "killed")
        killed_source = os.path.join(killed_copy, "crates", "demo", "src", "lib.rs")
        killed_marker = os.path.join(work, "killed-cargo-pid")
        killed_cargo = os.path.join(work, "bin", "killed-cargo")
        write_text(killed_cargo,
                   "#!/usr/bin/env bash\n"
                   f"if grep -q '{{ 42 }}' crates/demo/src/lib.rs; then exec {shlex.quote(fake_cargo)} \"$@\"; fi\n"
                   f"echo \"$$\" > {shlex.quote(killed_marker + '.tmp')} && mv {shlex.quote(killed_marker + '.tmp')} {shlex.quote(killed_marker)}\n"
                   "exec sleep 120\n", executable=True)
        killed_wrong = []
        child = os.fork()
        if child == 0:
            try:
                prove(killed_copy, "demo", ["caught"], cargo=killed_cargo)
            finally:
                os._exit(0)
        deadline = time.monotonic() + 60
        finished_alone = False
        while not os.path.exists(killed_marker) and time.monotonic() < deadline:
            if os.waitpid(child, os.WNOHANG)[0]:
                finished_alone = True
                break
            time.sleep(0.05)
        if not finished_alone:
            os.kill(child, signal.SIGKILL)
            os.waitpid(child, 0)
        sleeper = open(killed_marker).read().strip() if os.path.exists(killed_marker) else ""
        if sleeper.isdigit():
            try:
                os.kill(int(sleeper), signal.SIGKILL)
            except ProcessLookupError:
                pass
            if not process_gone(int(sleeper)):
                killed_wrong.append(f"睡着的假 cargo（进程 {sleeper}）停不掉")
        if finished_alone or not sleeper.isdigit() or open(killed_source).read() == "fn answer() -> u32 { 42 }\n":
            killed_wrong.append(f"没造出被杀的样子（子进程自己跑完了：{finished_alone}，假 cargo 记的进程号 {sleeper!r}，源码此刻 {open(killed_source).read()!r}）")
        else:
            cargo_calls()
            code, lines = prove(killed_copy, "demo", ["caught"], cargo=fake_cargo)
            calls = cargo_calls()
            if open(killed_source).read() != "fn answer() -> u32 { 42 }\n":
                killed_wrong.append(f"源码没写回，此刻 {open(killed_source).read()!r}")
            if not any(line.startswith("- 上一趟没还原的变异已写回：caught") for line in lines):
                killed_wrong.append("没报写回的那一行")
            if code == 2 or not any("--show-output" in call for call in calls):
                killed_wrong.append(f"基线没照常跑（退 {code}、cargo 的参数 {calls}）")
            if os.path.exists(os.path.join(killed_copy, RESTORE_JOURNAL_NAME)):
                killed_wrong.append("跑完还原日志还在")
            if killed_wrong:
                killed_wrong.append(f"输出 {lines}")
        checked_cases += 1
        if killed_wrong:
            failures.append("被杀之后同一份副本上下一趟起来：先照还原日志把源码写回、报写回的那一行、基线照常跑、还原日志删掉：" + "；".join(killed_wrong))
        # 留着替换文那一格（文件头「判法与次序」的「先核原文」一条，弄坏开关 old-text-checked-late）：副本里 demo 的源码已是 caught 那一行的替换文（41），
        #   没有还原日志可照：基线之前以原文命中 0 次整次拒绝、出路点到还原日志与干净副本、退 2、假 cargo 一次都不起
        leftover_copy = selftest_copy(work, "leftover")
        write_text(os.path.join(leftover_copy, "crates", "demo", "src", "lib.rs"), "fn answer() -> u32 { 41 }\n")
        cargo_calls()
        code, lines = prove(leftover_copy, "demo", ["caught"], cargo=fake_cargo)
        calls = cargo_calls()
        checked_cases += 1
        if code != 2 or calls or not any("命中 0 次" in line and "还原日志" in line and "干净副本" in line for line in lines):
            failures.append(f"副本里留着一条替换文而没有还原日志：基线之前以原文命中 0 次整次拒绝、退 2、cargo 一次都不起，实际退 {code}、cargo 起了 {len(calls)} 次 {lines}")
        # 没重编那一格（文件头「判法与次序」⑤）：调用方设的 CARGO_TARGET_DIR 是一份共用编译目录，里面 demo 的产物（没改过的源码）比施加变异后的源码新
        #   （另一份副本后编的那种，修改时刻取明天）：假 cargo 不重编、测试跑在旧产物上全绿；要判没重编、退 1，不判没红
        shared_target = os.path.join(work, "shared-target")
        stale_artifact = os.path.join(shared_target, "fake-build", "demo.rs")
        write_text(stale_artifact, "fn answer() -> u32 { 42 }\n")
        # 上一次在这份编译目录里编这个副本时留下的 dep-info：lib.rs 是编译读过的文件
        write_text(os.path.join(shared_target, "debug", "deps", "demo.d"), f"{stale_artifact}: {os.path.realpath(copy)}/crates/demo/src/lib.rs\n")
        tomorrow = time.time() + 86400
        os.utime(stale_artifact, (tomorrow, tomorrow))
        saved_target = os.environ.get("CARGO_TARGET_DIR")
        os.environ["CARGO_TARGET_DIR"] = shared_target
        try:
            code, lines = prove(copy, "demo", ["caught"], cargo=fake_cargo)
        finally:
            os.environ.pop("CARGO_TARGET_DIR", None)
            if saved_target is not None:
                os.environ["CARGO_TARGET_DIR"] = saved_target
        checked_cases += 1
        if code != 1 or not any(line.startswith("✗ caught\t没重编\t") and "Compiling demo" in line for line in lines):
            failures.append(f"编译目录里的产物比施加变异后的源码新（共用编译目录）：那一条判没重编、退 1，不判没红，实际退 {code} {lines}")
        # 编译目录归这一趟三格（文件头「判法与次序」的「编译目录归这一趟」）：每一趟新建、跑完删、被打断删
        trip_count, trip_failures = trip_build_directory_cells(work, fake_cargo, target_log)
        checked_cases += trip_count
        failures += trip_failures
        checked_cases += 1
        path_included_failure = path_included_package_failure(work)
        if path_included_failure:
            failures.append(path_included_failure)
        # 判定库那四格（文件头「判定库与复用」）：复用三问、复用哨兵、键相同的都复用、记红撤绿
        library_count, library_failures = library_cells(work)
        checked_cases += library_count
        failures += library_failures
        # 开 GPU 补 gpu 特性那一格（文件头「判法与次序」的「开 GPU 补 gpu 特性」一条）
        route_count, route_failures = gpu_route_cells(work)
        checked_cases += route_count
        failures += route_failures
        # 并行那十二格（文件头「判法与次序」的「并行」一条）
        parallel_count, parallel_failures = parallel_cells(work)
        checked_cases += parallel_count
        failures += parallel_failures
        # 不领卡那一格（文件头「不领卡」）
        no_card_lease_count, no_card_lease_failures = no_card_lease_cells(work, fake_cargo)
        checked_cases += no_card_lease_count
        failures += no_card_lease_failures
        # 编译目录三格：开头那几行不碰位置参数（只给 --selftest 也走得过去）；默认值在解析完 --copy 之后定，次序不限；调用方设了的不覆盖
        caller_target = os.environ.pop("CARGO_TARGET_DIR", None)
        try:
            probe = subprocess.run(["bash", os.path.join(SCRIPT_DIRECTORY, "prove-red.sh"), "--selftest"], capture_output=True, text=True,
                                   env=dict(os.environ, PROVE_RED_SELFTEST_PROLOGUE="1"))
            checked_cases += 1
            if probe.returncode != 0 or "PROVE_RED_PROLOGUE CARGO_TARGET_DIR=<未设>" not in probe.stdout.splitlines():
                failures.append(f"不设 CARGO_TARGET_DIR、只给 --selftest：开头那几行应当走得过去、不替调用方设编译目录（子进程打 CARGO_TARGET_DIR=<未设>、退 0），"
                                f"实际退 {probe.returncode}，输出 {(probe.stdout + probe.stderr).strip()!r}")
            # 编译并行度：没设 CARGO_BUILD_JOBS 取登记文件第一条非注释行的数（SINGLEFS_THREAD_CAP 也没设），设了的照它
            with open(os.path.join(MAIN_ROOT, ".claude", "heavy-thread-default"), encoding="utf-8") as registry:
                registered_jobs = next((line.split()[0] for line in registry if line.strip() and not line.lstrip().startswith("#")), "<登记里没有数>")
            for preset, want in ((None, registered_jobs), ("3", "3")):
                environment = {name: value for name, value in os.environ.items() if name not in ("CARGO_BUILD_JOBS", "SINGLEFS_THREAD_CAP")}
                environment["PROVE_RED_SELFTEST_PROLOGUE"] = "1"
                if preset is not None:
                    environment["CARGO_BUILD_JOBS"] = preset
                jobs_probe = subprocess.run(["bash", os.path.join(SCRIPT_DIRECTORY, "prove-red.sh"), "--selftest"], capture_output=True, text=True, env=environment)
                checked_cases += 1
                if jobs_probe.returncode != 0 or f"PROVE_RED_PROLOGUE CARGO_BUILD_JOBS={want}" not in jobs_probe.stdout.splitlines():
                    failures.append(f"{'不设' if preset is None else '调用方设了'} CARGO_BUILD_JOBS：开头那几行应当{'取登记的重活默认并行度 ' if preset is None else '照它的 '}{want}（子进程打 CARGO_BUILD_JOBS={want}），"
                                    f"实际退 {jobs_probe.returncode}，输出 {(jobs_probe.stdout + jobs_probe.stderr).strip()!r}")
            if os.path.exists(target_log):
                os.unlink(target_log)
            os.environ["CARGO_TARGET_DIR"] = os.path.join(work, "caller-target")
            code, lines = prove_from_arguments(["--copy", copy, "demo", "caught"], cargo=fake_cargo)
            seen = open(target_log).read().splitlines() if os.path.exists(target_log) else []
            checked_cases += 1
            if code != 0 or not seen or set(seen) != {os.path.join(work, "caller-target")}:
                failures.append(f"调用方设了 CARGO_TARGET_DIR：应当照它的、不覆盖，实际退 {code}，cargo 看到的 {seen}")
        finally:
            os.environ.pop("CARGO_TARGET_DIR", None)
            if caller_target is not None:
                os.environ["CARGO_TARGET_DIR"] = caller_target
        for failure in failures:
            print(f"  ✗ 自检：{failure}")  # gate-lint:detail
        if failures:
            print("  → 看 prove() / table_rows() / refusal_for() / verdict_of() / judged_line() 的判法（判结局在 crates-mutation-rows.py 的 verdict_of_run，"
                  "还原日志与先核原文看 restore_unrestored_mutation() / write_restore_journal() / old_text_misses()，"
                  "补 --no-default-features 看 default_features_off_rows()（判法在 crates-mutation-rows.py 的 default_features_off_row），补 --release 看 release_forced_rows()（判法在 lib_release_only_tests.py），补 gpu 特性看 library_plan() 调 gpu_route_rows 那一段（判法在 crates-mutation-rows.py），"
                  "结果行在 lib_libtest_result_lines.py），编译目录那三格看 prove_from_arguments() / cargo_environment() 与开头 python3 之前那几行，编译目录归这一趟那三格看 prove() 里 new_trip_build_directory() / remove_trip_build_directory() 与 stopped_by_signal()；链接失败看 judged_line()（判法在 crates-mutation-rows.py 的 verdict_of_run）；"
                  "判定库那几格看 library_plan() / planned_sentinel() / record_verdict() 与 prove_on_restored_sources() 里复用、交哨兵那一段（行键与复用判据在 crates-mutation-rows.py）；"
                  "并行那几格看 planned_lanes() / prepared_lanes() / run_in_lanes() / lanes_collected_problem() 与 prove_on_restored_sources() 里 judged_on_lanes()；"
                  "不领卡那一格看 run_cargo()；"
                  "PROVE_RED_BREAK 或 LIBTEST_RESULT_LINES_BREAK 设着的话这里本来就该红")
            return 1
        print(f"  ✓ prove-red 自检通过（判法样本 {len(SELFTEST_VERDICT_SAMPLES)} 条）：会红的抓到并写回源码，should_panic 红了抓到，编不过判无效，别的测试红判没红，基线红停下退 2，"
              f"不挑目标的整次拒绝，目标名带 crash_point_replay 而包不是 验证包的照跑，主工作区拒绝，缺闭包里条目的副本整次拒绝、拷全的照跑，"
              f"验证包的行照跑，验证包只点名集成测试目标的行补 --no-default-features 起而点名 --lib 的不补，命中重测试目标的行补 --release 起而不命中的不补，开 GPU 时点名测试走崩溃判段的行补 gpu 特性起，不成形的行整次拒绝，切表与 read_table 相同，"
              f"判法与 verdict_of_run 对 {len(SELFTEST_VERDICT_SAMPLES)} 条样本逐行相同，跑的时候副本根上拿着副本锁、跑完放掉、锁被占着时拒绝，"
              f"施加之后被杀的下一趟先照还原日志写回源码再跑基线，留着替换文而没有还原日志的基线之前就拒绝，"
              f"每一趟自己建编译目录、不沿用副本里别的命令留下的 target、跑完删（成功、红、被 TERM 停都删），替换之后失败在链接的单列链接失败、不判无效，"
              f"共用编译目录里的产物比施加变异后的源码新时判没重编，被改文件经 #[path] 编进点名的包时认那个包的 Compiling 行，"
              f"不设 CARGO_TARGET_DIR 只给 --selftest 走得过开头，调用方设了的不覆盖，"
              f"行键算不出的照跑、报拒算，判定按行键进判定库：复用三问都过、复用哨兵抽一条强制现判一条、键相同的都复用时 cargo 一次都不起、强制现判判成没红的记红撤绿，"
              f"互不撞文件的行并行：判定与串行逐条相同、输出按点名次序、每路各一份工作副本与编译目录、单路红不吃掉、收回数核对、一路基线红一条变异都不施加，"
              f"不领卡：带不带 gpu 特性的行都照原样起、不另编、环境里给的卡串原样交、不藏卡（查了 {checked_cases} 种）")
        return 0
    finally:
        CLOSURE_ROOT_FOR_SELFTEST.clear()
        LIBRARY_ROOT_FOR_SELFTEST.clear()
        for name, value in {**saved_caches, **saved_card_variables}.items():
            if value is None:
                os.environ.pop(name, None)
            else:
                os.environ[name] = value
        shutil.rmtree(work, ignore_errors=True)
        os.environ.pop("CARGO_TARGET_DIR", None)
        if inherited_target is not None:
            os.environ["CARGO_TARGET_DIR"] = inherited_target


def prove_from_arguments(arguments, cargo="cargo"):
    """→ (退出码, [输出行])：先解析 --copy，再证红；编译目录的默认值由 prove 照解析出的副本定（上限由 task-server 按记录给，不收 --memory 这类参数）。"""
    if len(arguments) < 4 or arguments[0] != "--copy":
        return 2, ["✗ 用法不对", "→ 怎么办：prove-red.sh --copy <仓副本> <crate> <变异名…>"]
    return prove(arguments[1], arguments[2], arguments[3:], cargo=cargo)


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
