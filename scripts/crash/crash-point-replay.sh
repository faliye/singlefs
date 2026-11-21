#!/usr/bin/env bash
# task-server-shards: *
# task-server-compiles: yes
# admission: always 每次调都在这一刻的树上跑全部项或点名的项；测试函数那一段按测试函数单元键查门禁那一份判定库（文件头「按条复用」），崩溃点重放部分与放量部分由背后两套驱动各自的判定库按块定
# run-condition: command bash cargo python3
# run-condition: check source "$PREFLIGHT_SCRIPT_DIRECTORY/crash-amplification-lib.sh" && crash_amplification_run_condition_holds "$PREFLIGHT_SCRIPT_DIRECTORY/../.." :: 修好设备，或改配置：多机配置 multi-host.env 开了 GPU 而一份卡都没写、点名的卡与双机开关对不上；单跑 bash -c 'source scripts/crash/crash-amplification-lib.sh && crash_amplification_run_condition_holds' 看是哪一处
# 多机：这一份不自己分台、不自己连第二台。经 task-server 按件分片（文件头「经 task-server 按件分片」），计划、各片与合并派到哪一台由服务定，
#   一片里的测试函数那一段、崩溃点重放部分与放量部分都在这一片派到的那一台跑；机器、内存上限与分片归 task-server，卡归 GPU 批处理验证服务（文件头「GPU」）。
# 崩溃点重放的入口（判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J2「一张表、一个入口、一道门禁」；
# 只有一种跑法：不带参数跑全部项，缩小范围用 --item，见 .claude/rules/verification.md）。登记表是合表 .claude/gate.d/crash-point-replay-items.tsv（写法在它的表头，
# 一项一条流），读法与核法在 scripts/crash/lib_crash_point_replay_items.py。背后照调两套驱动：崩溃点重放那一套（门禁 crash-point-replay：逐条用例照块判定库复用）
# 与放量分项驱动（scripts/crash/crash-amplification.sh）；引擎合并之后由引擎那一路换成一套。
#
#   crash-point-replay.sh                       全部项（连同整趟才跑的那几类 whole-run:<类>）：测试函数那一段照测试函数判定库复用、崩溃点重放部分照块判定库复用、放量部分照判定库复用
#   crash-point-replay.sh --item <项> […]       只跑点名的项（给几次取并集；整趟才跑的那几类也是项，写 whole-run:<类>）；缩小范围就用它（--list-items 列可点的项）
#   crash-point-replay.sh [--item <项>]… --crash-expand-up-to <n>
#                                               缩崩溃域（指定范围的参数允许缩域）：把崩溃放量与崩溃枚举用例的展开深度缩到不超过 n，
#                                               把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=<n> 与 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=<n>（判器的展开深度，验证包里直接调判器的对拍读它）
#                                               两个都交给测试函数、崩溃点重放、放量三段（引擎只往小缩，大于引擎里放量的展开上限常量的不放宽）；
#                                               值不是非负整数、后面没有值都退 2、什么都不做；环境里已设了放量那个变量的当作给了同一个值，两样都给取参数；
#                                               判器那个：给了参数时设成参数的值（盖过环境里的）；没给参数而环境里已设的照用，没设的跟着放量那个的值走；
#                                               环境里只设了判器那个而没缩域参数的，只认它、放量那个不设。环境里设了的判器变量同样要是非负整数。缩过域的那一趟 ITEM、TOTAL 行末尾接
#                                               expand_up_to=<n>（判器那个是环境里设的接 judge_expand_up_to=<m>，跟着放量那个设的不接），成功句写缩过域、不算门禁通过
#   crash-point-replay.sh --list-items          逐行打「项名<TAB>测试函数 …（∪ 崩溃点重放 …）（∪ 放量 …）」，末尾是 whole-run:<类> 几行，只读合表，不编、不跑
#   crash-point-replay.sh --list-items --with-keys [--item <项>]…
#                                               同上，再按项各打一行 ITEM_PLAN item=<项名> to_judge=<要现判的件数> reusable=<能复用的件数>（件的算法见下「输出」），
#                                               紧接着按项各打一行 ITEM_DOMAIN item=<项名> states_to_judge=<这一项要现判的崩溃状态数，估不出写 unknown> source=<来源>
#                                               （态数的估法与来源的写法见 lib_crash_point_replay_items.py 文件头「指定范围的崩溃域上限」；列出的项一起交一趟判法，
#                                               起一次读库程序 items 子命令读全部项的块账（一次连库、一遍扫库，每项一行），读不到的改按上一趟真跑的态数估；上限判的就是这个数；行照项的次序打），
#                                               末尾再打几行「键<TAB>…」：崩溃判定库里录下的块有几块键对不上（只在别的判法单元指纹下判过、
#                                               一种判器的结论都没有）、几块键相同，与两种判器这一份的判法单元指纹；读库程序打了按态数的 CRASH_LIBRARY_STATES 行时
#                                               另打一句态数（录下的态、当前键下判过的、只在别的键下判过的、一种判器的结论都没有的；那一行认不出照实报、退 1），
#                                               没打那一行就只打块数、不判红（按态数的总账与完整性闸归 scripts/crash/crash-library-totals.sh）。可与 --item 合用，只列、只算点名的项
#                                               （不带 --with-keys 的 --list-items 不与 --item 合用，退 2）。起只读读库程序
#                                               （cargo run … --bin crash_library_reader -- keys：只读开库、不写），它的输出写法在
#                                               crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」那一段，自证 list-items-with-keys 格
#                                               把那一段原样喂进来钉读法；读库程序退非 0 或输出认不出照实报、退 1。
#                                               读库程序与真跑编同一棵树：真跑是 cargo test --release -p singlefs-verification [--features gpu]，验证包 [dev-dependencies]
#                                               里带特性的依赖（今天是 singlefs-core 的 lookup-work-counters）在 cargo test 的树里打开、cargo run 的树里没有，判法单元指纹跟着变，
#                                               所以读库程序那一趟 cargo run 带上真跑那一套（--release、gpu 开着加 gpu、再加这些依赖的 <依赖>/<特性>，keys_reader_features 现读清单）；
#                                               自证 keys-reader-same-build 格钉它
#   crash-point-replay.sh --selftest            假 cargo、假的两套驱动、假的准入模块走一遍各格，末尾一轮从下面「弄坏开关」那张表现读开关、逐个打开
#   CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT=<段号>/<段数>|base crash-point-replay.sh --selftest
#                                               开关轮分段点名单跑：只跑开关轮的那一段（写法与分法见下面「开关轮分段」），不跑基准趟各格；base 只跑基准趟各格、不跑开关轮
#   别的参数一律当认不出的参数退 2（用法里只写正确用法）。
#   引擎配了 GPU 只由 GPU 判、没配只由 CPU 判，同一批状态一趟只判一次；GPU 判与 CPU 判的对拍是另立的测试，不在这里跑。
#   不带 --item 的跑法（不指定范围）只在提交时或用户同意后由主 agent 交，崩溃状态数不设上限；带 --item 的（指定范围）主 agent 随时可以交（项目 settings 里注册的钩子判），
#   但没缩域时开跑之前先估这一趟点名的项要现判多少崩溃状态（只数键对不上、要现判的；估法与记录来源在 scripts/crash/lib_crash_point_replay_items.py 文件头「指定范围的崩溃域上限」），
#   超过 200000 态（常量 SCOPED_CRASH_STATES_LIMIT，只登记在那份库里，放量入口共用）或估不出就拒：不编、不跑，退 79，说明里给两条出路——带 --crash-expand-up-to <n> 缩域再交
#   （算得出就写出能压到上限以内的 n；缩过域的那一趟成功句写缩过域、不算门禁通过），或留给提交时不指定范围的那一趟。--crash-expand-up-to 的值不小于引擎展开上限常量的不算缩域，照判。
#   经 task-server 按件分片的跑法，这一判在计划那一步做，各片与合并不再判；放量部分起放量入口时交 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1，放量入口不再判一遍。
#   哪一种都经 task-server/task submit -- bash scripts/crash/crash-point-replay.sh … 交（内存上限与机器由服务定，卡归 GPU 批处理验证服务，命令里不写）。
#
# 三段（.claude/rules/verification.md）：
#   测试函数那一段 = 每项的测试目标里除登记成 crash-case: 的用例与放量分项之外的全部测试函数。项的测试目标不在合表里另登记：lib_crash_point_replay_items.py list 的第四段由这一项
#     点名的崩溃点重放部分与放量部分的测试目标现推；整趟才跑的那几类（WHOLE_RUN_GROUPS）的测试目标写在这份文件里。按测试函数单元键逐条复用（文件头「按条复用」），
#     登记的用例与放量分项经 test-plan 的 --exclude-registered 排掉、交下面两段；一个测试目标里的测试函数全登记了的，这一段对它没有可判的，不编不起也不记一笔。
#     按段登记里域装不进 u64 的段（驱动行 registered_segment_domains=<段号>:slice）引擎分片枚举：不指定范围那一趟判完全部片，SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES（没设取 2^32）只当一趟判几个态的分批：
#     三段都不设它，从调用方环境继承（片长进键；被打断之后重跑同一条命令，从 prune-server 里同一份判定库的续点接着判，库里还没有续点的头一趟从段内序号 0 起）；
#   崩溃点重放部分 = 第二列的 crash-case: 交门禁 crash-point-replay --item，键相同的块复用、对不上的现判；带 --item 的跑法起门禁时另带 --check crash-case-replay --crash-cases-only
#     （只走按块那一路，门禁不把整个验证包再按条判一遍：点名项的测试目标上面那一段已经判过）；不带 --item 的跑法门禁整道照旧；
#   放量部分 = 第三列的放量分项交 crash-amplification.sh，展开深度只有一处（引擎里放量的展开上限常量），这里不传；判定在 prune-server 里那一份判定库（crash-amplification-lib.sh 文件头
#     「判定库」；.claude/rules/verification.md 崩溃放量那一行），键相同、判过的块照库复用，被打断之后重跑同一条命令就是续跑
#     （test-script-discipline.md「判定可复用：按输入存，按前缀剪」）；ITEM 行与成功句照放量入口的 CRASH_AMPLIFICATION_ITEM 行报现判几态、复用几态。
#   计划（replay_plan）、复用、入库、判红在两种跑法里是同一段代码。项的取法：不带 --item 是全部项连同整趟才跑的那几类（WHOLE_RUN_GROUPS：小输入的 GPU 与 CPU 对拍、
#     剪枝自证、钉回来的红例），带 --item 只是点名的项。三段都照键复用：键相同、库里不红的不重跑，单元键对不上的（这一次改到的）与红过的才现判。
#   崩溃点重放部分里登记行第三列写了 red-blocks=judged-by-test 的用例，判法在那道门禁，这里只收它的退出码：按 cargo test 的退出码判，块那一行只核数，库里的红块不判。
#   三段各跑各的、各判各的：一段红了另两段照旧起（一项的结论是几段的合：有一段红就 red）。
# 按条复用（跑过的要入库、照键复用，红过的要复判）：测试函数那一段在两种跑法里是同一段：
#   每条测试函数各是一条判定，种类、请求、跳过名单与判绿名单只在 scripts/gate/admission.py 的 test-functions-request、test-plan、plan-skip-names、plan-green-names、
#   verdict-record（它文件头「按条复用的共用段」），与门禁 crash-point-replay 测试函数几格同一类判定库
#   （<git common-dir>/singlefs-unit-verdicts/crash-point-replay.sh:test-functions/）：这里写进的门禁照键复用，门禁写进的这里照键复用。
#   test-functions-request 在交给测试函数 cargo 的同一份环境里起（它照那一份环境给改判定域的变量打 variable: 那几项：按段登记文件的内容、片长这一类，
#   admission.py 文件头「判定请求」「单元键」；门禁的 cargo 继承门禁自己的环境、它在同一份里起，两处取法一致）。
#   开跑前 test-plan 算 验证包（这一趟的 --features 逐个交 --feature）每条测试函数的键、查库；每个测试目标
#   键相同的经 cargo test -- --exact --skip <名> 跳过（plan-skip-names --target-root tests/<目标>.rs，目录式目标写 tests/<目标>/main.rs），一条都不用现判就不编不起 cargo；跑完照
#   lib_libtest_result_lines.py recordable（开了 GPU 不带 --cpu-judging-recordable）逐条认结局，判绿的经 plan-green-names
#   与 verdict-record 照键写回；这一趟真跑到而判红的（结果行 FAILED）经 verdict-record --red-names 照计划里的键记红、撤掉同一把键原有的绿
#   （记红、撤绿、下趟现判，admission.py 文件头「判定库」）：键相同而库里记着红的下一趟照现判，判绿之后再盖回绿。
#   判定库的名字是被判的根（这份脚本所在的仓，自证里是 CRASH_POINT_REPLAY_ROOT）的 git common-dir 下的种类目录（只当名字，键里不带哪棵仓，
#   命名空间只在临时服务底下给；admission.py 文件头「判定库」）；成功那一行点名库在哪。
#   逐条打一行「· <测试函数>：绿|红，现判|复用|强制现判（…）」（scripts/gate/lib_reuse_checks.py 的 ITEM_LINE），末了一行「· 按条复用（判定库 <种类>）：现判 N、复用 M、拒算 R……」。
#   被判的根不是一棵 git 工作树的顶（判定库没有名字）：测试函数一个测试目标都不编不起，整趟退 2、点名那棵树（弄坏开关 test-functions-without-library-tolerated 下全部现判、不入库）。
#   放量入口那一趟编的特性与这里算键的特性同取这一趟交给驱动的那份多机配置（SINGLEFS_MULTI_HOST_CONFIG）的 ENABLE_GPU，所以判定照这里的键记。
#   按条复用失灵（test-plan 退非 0 或没打 PLAN、verdict-record 退非 0 而复用哨兵报红；判法只在 admission.py 的 plan-failure，它文件头「按条复用失灵」）：
#   照旧全部现判跑完，每个点到测试函数测试目标的项另记一笔红（不能复用就判红）。
#   test-plan 或记库的 verdict-record 退 75（prune-server 不可用）不是失灵：plan-failure 判不可用、退 75，这里打 ✗ 与出路、整趟退 75（计划那一步不可用时一个测试目标都不编不起）。
# 整趟才跑的那几类（WHOLE_RUN_GROUPS，一行「类名<TAB>测试目标，空格隔开；没有写 -」）：各是一项 whole-run:<类名>，--item 点得到、--list-items 列出，不带 --item 时一起跑；没有测试目标的一类报本次未跑、写明为什么。
#
#   测试函数那一段对登记的用例与放量分项的排法：test-plan 带 --exclude-registered，登记成 crash-case: 的用例（stage-inputs.tsv 的 test=）与放量分项（crash-amplification.sh 的 ITEMS）各记一行 registered，
#   plan-skip-names 把它们与键相同的一并列进 cargo test 的 --skip（门禁按条复用那一趟同样排）；弄坏开关 registered-not-excluded 下不排。
# GPU（.claude/rules/test-script-discipline.md「用上现有的硬件加速」）：多机配置（crash-amplification-lib.sh 文件头「配置」：
#   ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，按 git common-dir 找）写了 ENABLE_GPU=1 时测试函数那一段带 --features gpu 编。
#   卡只归 GPU 批处理验证服务（.claude/rules/verification.md「卡只归 GPU 服务，任务不拿卡」）：它照多机配置开卡、一直持着、自己清场。这一份不领卡、不挑卡、不清场，
#   不读多机配置里的显卡先后，文件头不写要卡的声明。测试函数那一段：环境里有 GPU_CARDS_BY_PRIORITY 就原样交给编那一趟与每一趟测试函数的 cargo（自证 gpu-test-functions），
#   没有就不设、照起，不退 78（自证 gpu-without-cards-runs）；要卡的测试读不到 GPU 服务的结论就红在测试里，照 cargo test 的退出码判红。
#   崩溃点重放部分与放量部分同样不领卡：门禁 crash-point-replay 照它文件头「显卡」办，放量入口照它自己的文件头办。
#   走 GPU 的计算类不送（引擎开了 GPU 就三类全走 GPU，crash-amplification-lib.sh 文件头「配置」）。
#   开了 GPU 时单机路径的每一趟测试函数测试参数带 `-- --nocapture`、不带 --test-threads：测试线程同没开 GPU 的，照服务交下来的 RUST_TEST_THREADS（登记的重活默认并行度），
#   同一个测试二进制里并行的 GPU 用例经卡池排队（crates/singlefs-verification 的 gpu_test_start，.claude/rules/verification.md 里 GPU 那一条）。
#   自证 gpu-test-functions（环境里有卡）与 gpu-without-cards-runs（环境里没有卡）两格钉它，弄坏开关 gpu-test-threads-one 下带 --test-threads 1。
#   准入模块 admission.py 拼的 crash-case 命令与崩溃放量（crash-amplification.sh 经共用库 crash-amplification-lib.sh 拼的 cargo test 参数）同样不带。
#   测试线程只管同时起几个测试，不管一个测试里 CPU 判器自己开的线程：GPU 对拍的测试读 SINGLEFS_THREAD_CAP（服务交下来的线程变量之一），
#   没设取登记的每台默认重活线程数（.claude/heavy-thread-default）。
#   没开不带 --features，挂 gpu 特性的对拍编进来 0 条、报本次未跑。放量部分的 GPU 由 crash-amplification.sh 自己照同一份配置办（它同样不领卡、不清场）。
#   崩溃点重放部分里经引擎的 crash-case 行（.claude/gate.d/stage-inputs.tsv 里登记了 features= 的，今天是随机历史两项）由准入模块 admission.py 的 crash-case-command
#   带 --features 起、开 GPU 时同样加 gpu，不在这里拼（写法见 admission.py 文件头「崩溃枚举用例」）。
#
# 往引擎送的变量（测试函数、崩溃点重放部分与放量部分都从这一趟的环境继承）：
#   SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=<仓根>/.claude/gate.d/crash-point-replay-segments.tsv（按段登记文件的绝对路径，判决 J10、M4 第 15 条）：
#     每一趟都送；开跑前先经 scripts/crash/lib_crash_point_replay_items.py segments 核它（核不过退 2、什么都不起；写法与核法在那份库文件头「按段登记」）。
#   SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=<分批一趟判几个态>：片长只当分批，不指定范围那一趟判完全部片；三段都不设，从调用方环境继承（设了照用，空串也照原样交，引擎拒）。
#   SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=<n>：只在给了 --crash-expand-up-to 或环境里已设时有；三段都收到同一个值（测试函数那几趟 cargo 清掉调用方环境里别的 SINGLEFS_CRASH_AMPLIFICATION_*，
#     只把这一个原样带上；崩溃点重放那一道与放量入口从这一趟的环境继承）；它是改判定域的变量，进测试函数的单元键，所以缩过域的判定与全量的各存各的。
#   SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=<n>：判器的展开深度，验证包里直接调判器的对拍读它（读哪一个由目标的源码定，这里不判）；缩域时与上一个设成同一个值（给了 --crash-expand-up-to 时盖过环境里的，
#     没给参数而环境里已设的照用），环境里只设了它的照旧只认它；它从这一趟的环境继承给三段，同样进单元键。
#   SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1：只交给放量部分起的放量入口（命令前缀），告诉它指定范围的崩溃域上限这一趟已判过、不再判一遍（放量入口文件头「指定范围的崩溃域上限」）；
#     这个变量不改判定域，不进任何键；弄坏开关 scoped-domain-handshake-dropped 下不交。
#   测试函数那几趟 cargo 清掉调用方环境里的 SINGLEFS_CRASH_AMPLIFICATION_*、SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE（同门禁 crash-point-replay 的 feature-gated-tests 格），
#   多机配置在就交 SINGLEFS_MULTI_HOST_CONFIG=<它的绝对路径>；这几趟 cargo 直接起、不套线程包装：编译与测试线程照服务交下来的 CARGO_BUILD_JOBS、RUST_TEST_THREADS、SINGLEFS_THREAD_CAP 原样继承；
#   判定都在 prune-server 里同一份库（引擎经客户端取连哪一处），不另交库的变量。
#
# 输出：背后每一趟的输出边跑边转打，同时进这一趟的日志目录 <放量的家目录>/runs/crash-point-replay-<时戳>-<进程号>/（跑完不删）。每项（与整趟才跑的每一类）
#   打一行 CRASH_POINT_REPLAY_ITEM item=<项> scope=<whole|items> exit=<0|1|77|78> verdict=<green|red|not-run>
#     test_functions_tests=<测试函数跑到的用例数> test_functions_red=<红的用例数> test_functions_reused=<测试函数复用的块数，见「测试函数的复用数怎么取」；一个测试目标都取不出写 unknown>
#     test_functions_reuse_unknown=<取不出复用数的测试目标个数>
#     pipeline_states=<放量现判的态数> pipeline_red=<红> pipeline_reused=<复用的态数，放量入口取不出写 unknown>
#     enumeration_cases=<崩溃点重放部分的用例数> enumeration_red=<红的用例数> enumeration_reused=<全部块都复用的用例数> seconds=<这一项各段的挂钟之和>
#     （这一项没有那一段的写 -）；末行 CRASH_POINT_REPLAY_TOTAL scope=… dispatched=<派出几项> collected=<收回几项> green= red= not_run=
#     test_functions_tests= test_functions_reused_known=<取得出的那几个测试目标复用的块数之和> test_functions_reuse_unknown=<几个测试目标取不出>
#     pipeline_states= pipeline_reused_known= pipeline_reuse_unknown=<几项取不出> enumeration_cases= enumeration_reused= seconds=；缩过域的那一趟两种行末尾再接 expand_up_to=<n>。
#   按项机读行（任务服务按项先算后排、估时用）。一项的件 = 测试函数那一段的一条测试函数（test-plan 判键对不上与拒算的算要现判、键相同的算复用）+ 一条崩溃枚举用例（crash-case）
#     + 一个放量分项；整趟才跑的那几类按它的测试目标算，没有测试目标的一类 0、0。崩溃枚举用例与放量分项的块键只有读库程序按整库数、没有按项的读法，所以它们一件算一条要现判、不算复用：
#     ITEM_PLAN item=<项名> to_judge=<件数> reusable=<件数>   --list-items --with-keys，stdout，列项之后、「键」几行之前每项一行；可与 --item 合用只算点名的项
#     ITEM_PLAN shard=<片号> item=<项名> to_judge=<件数> reusable=<件数>   按件分片的计划那一步，stdout，每片每项一行，条数按这一片算；一项的测试函数那一段在它所有的片里都跑，
#       条数记在它头一次出现的那一片，所以各片各项之和等于不分片那一趟的 ITEM_PLAN
#     ITEM_DOMAIN item=<项名> states_to_judge=<态数|unknown> source=<no-crash-domain|reusable|prior-run-scaled|prior-run|unknown>   --list-items --with-keys，stdout，每个列出的项一行，
#       在 ITEM_PLAN 之后、「键」几行之前：这一项没缩域时要现判的崩溃状态数（只数键对不上的），指定范围的上限 200000 态判的就是它（lib_crash_point_replay_items.py 文件头「指定范围的崩溃域上限」）
#     崩溃点重放部分每跑完一趟，在日志旁写 <日志>.scope（enumeration.<项>.log.scope，分片时 enumeration.<项>.<用例>.log.scope）：expand_up_to=<full|n>、judge_expand_up_to=<n>（只有环境里自己设了判器那个变量才有，跟着缩域参数设的不写）、
#       case_states=<用例名>:<态数> …（每条交门禁的用例一对：态数是门禁放的那条用例日志 log.<用例名> 里 CRASH_POINT_REPLAY_PARALLEL_FINISHED 行 states= 之和，
#       日志目录取自门禁打的「这一趟每条用例的日志与发现日志放」那一行，态数在跑的这一台上用 lib_crash_point_replay_items.py case-states 取，读不出的那条不写）；估上限时读它
#     ITEM_DONE item=<项名> seconds=<这一项各段的挂钟秒数之和> judged=<现判的件数> reused=<复用的件数> exit=<这一项的退出码 0|1|77|78>   真跑，stderr，每项一行，在 CRASH_POINT_REPLAY_ITEM 之前：
#       复用的件 = 键相同的测试函数、全部块都复用的崩溃枚举用例、判了 0 态而复用了态的放量分项；本次未跑的段不算件；分片合并时照各片存下的段记录算
#   测试函数的复用数怎么取：走判定库的用例（崩溃枚举那一类，crates/singlefs-verification/src/crash_state_enumeration.rs 文件头「行」）每判一趟打一行
#     CRASH_POINT_REPLAY_PARALLEL_FINISHED …，其中 resumed_slices= 是库里复用的块、freshly_run_slices= 是这一趟判的块（scripts/gate/admission.py 文件头
#     「崩溃枚举用例行」threads= 那一条按的也是这一行）；一个测试目标的复用数是它日志里这种行 resumed_slices= 之和（单位是块）。一行都没有、或有一行缺这个字段、
#     不是非负整数，这个测试目标记「取不出」（unknown），不并进总计、不写 0 也不写 none，成功句点名是哪几个测试目标（test-script-discipline.md「每项结束打一行」）。
#   一项怎么判：有一段红就 red；不红而有一段本次未跑（测试函数跑到 0 条、有 NOT_RUN_NO_GPU_ADAPTER、放量那一项 verdict=not-run、崩溃点重放那一道退 77 或 3，
#   某一段退 78）就 not-run，逐项写原因；别的 green。测试函数一个测试目标：cargo 退非 0 或有 FAILED 就红（编不过的整段红）；
#   日志里有行首（可带空白）是 CRASH_POINT_REPLAY_GIVEN_STATES_RUN_CONDITION_NOT_MET 的那一行（给定状态表的判器起过卡池而借不到显存，引擎打这一行后报错、cargo 退 101；
#   行首取 crash-amplification-lib.sh 的 CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX）的，不记红：那一段记 not-run、退 78、原因写那一行，整趟没红就退 78。
#   放量部分点名的分项没打 CRASH_AMPLIFICATION_ITEM 行就红（收不回）；放量入口退 78（运行条件不满足）的除外：那一项记 not-run、退出码 78、原因写入口退 78 与日志路径，
#   整趟没有红就退 78（下面「退出码」），有红先判红、退 1；派出几项、收回几项对不上整趟红（command-safety.md「并行不许把失败吃掉」）。
#   三段各跑各的：一段红了另两段照旧起，派出收回仍要相等。
#   有红时出路给出只重跑红的那几项的命令（项数就是红的项数；整趟才跑的那几类也是项，一样点名）。
#   退 0 时打一句成功句：「崩溃点重放（<范围>）：K 项都不红；本次未跑 J 项：<项（原因）…>；测试函数 N 条复用 M 块（另有 J 个测试目标取不出复用数：<测试目标…>）、
#   放量现判 N 态复用 M 态、崩溃枚举用例 N 条复用 M 条；派出 D 项、收回 D 项」，
#   成功句之后（红了也一样）把这一趟日志目录里的覆盖行与计数行落成两份产物，留在同一个目录（判法与写法在 scripts/crash/crash-flow-coverage.py 文件头）：
#     crash-flow-records.tsv  run-directory 转出的记录（崩溃点重放部分读门禁打的 CRASH_FLOW_RECORDS_FILE 那一份）；
#                             填 .claude/gate.d/crash-ignored-nodes.tsv、invariant-zero-evaluations.tsv 的初值用 crash-flow-coverage.py baseline <ignored|zero-evaluations> 这一份
#     crash-flow-coverage.md  覆盖表（入口 × StepKind 成员）、ignore 开着的节点、评估 0 次的（流，不变量）对
#   这两份只落产物、不判红、不改这一趟的退出码（判在门禁 crash-point-replay 的出口 4 那几格）；写不成报一行。
#   上次红的清单（<数据根>/last-runs/scripts__crash__crash-point-replay.sh.tsv，写法在 scripts/gate/lib_reuse_checks.py 文件头「三」）：点不点名
#   都只拿这一趟真判过的项（判绿或判红的）换掉它们在清单里的状态，本次未跑的项照留上一份的记录；几趟同时跑的不互盖（共用库在锁里读、合、换上）。
#   经 task-server 按件分片（用户：「默认应该就是分片的 不分片才要加参数」）：文件头「# task-server-shards: *」让服务把每一种跑法（不带参数、--item）拆成计划、各片、合并
#   （task-server/task_server_lib.py 文件头「分片」），哪台空出来哪台接下一片。不经服务起（环境里没有 TASK_SERVER_SHARD_PHASE）的跑法退 2、什么都不起，
#   出路是经 task-server/task submit 交；--list-items 与 --selftest 不分片、照直接跑。自证起的子进程不经服务时整份计划当一次判（假 cargo 与假驱动，只核判法；
#   CRASH_POINT_REPLAY_UNSHARDED_REFUSED=1 的子进程照平常跑拒，unsharded-run-refused 格用）。件是计划（replay_plan）里的每一条崩溃枚举用例与每一个放量分项，一件一片、超过上限时相邻的合片；
#   只有测试函数或本次未跑的项（整趟才跑的那几类）各算一件。这一份照环境里的 TASK_SERVER_SHARD_PHASE 只跑其中一步：
#   plan   往计划目录写 plan.tsv（整趟的计划）、shard-plan.tsv（片号加计划行，每片带上它的件所在项的测试函数那一段）与 plan.json，退 0；不起测试函数与两套驱动。
#   shard  照 TASK_SERVER_SHARD=<片号>/<片数> 只跑 shard-plan.tsv 里自己那一片的行，判法同不分片：结论写进产物目录（TASK_SERVER_SHARD_OUTPUT_DIRECTORY 下的 parts.tsv、
#          crash-flow-records.tsv，各份 .problems.log，末了 shard-finished）。判红（退 1）或本次未跑（退 77）这一片退 0：判红不影响后面的片，结论交合并那一步判、
#          整件的退出码在合并；退 78 与别的非 0 照原样退（服务照那一片的结局收尾）。
#   merge  不起测试函数与两套驱动，照各片存下的 parts.tsv 判，ITEM、TOTAL、上次红的清单与退出码同不分片的那一趟（同一个测试目标几片都跑了的测试函数行只算一次，有红取红）。
#          有一片的产物不全（产物表里没有那一片、或缺 shard-finished）退 1。
#   读完这几个变量就从环境里清掉（TASK_SERVER_SHARD_*），不传给下层的门禁与放量入口。TASK_SERVER_SHARD_PHASE 是别的值、片号对不上计划、计划不全退 2。
# 退出码：
#   0   跑到的项都不红、派出收回相等（有项本次未跑也是 0，成功句逐项点名）
#   1   有红、或派出收回对不上
#   2   用法错：CRASH_POINT_REPLAY_SELFTEST_CELLS 里的格名写错（文件头「选格」）、CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT 写错或与选格、弄坏开关同给（文件头「开关轮分段」）、参数认不得（删掉的选项一样当认不得，用法里只写正确用法）、--item 后面缺项名或项名认不得（列出可点名的项）、--list-items 或 --selftest 夹了别的参数、
#       合表核不过、按段登记文件核不过、平常跑时带了只供自证的变量（见「只供自证」）、不经服务起（环境里没有 TASK_SERVER_SHARD_PHASE，--list-items 与 --selftest 除外）
#   75  prune-server 不可用：按条复用的 test-plan 或 verdict-record 退 75（判定库连不上；文件头「按条复用」），出路 python3 prune-server/control check、报给主 agent
#   77  派出的项全是本次未跑，也没有红
#   79  指定范围的崩溃域超上限：带了 --item、没缩域，点名的项这一趟要现判约多少崩溃状态超过 200000 态，或估不出来（一项都没编、没起）；出路：带 --crash-expand-up-to <n> 缩域，或留给提交时不指定范围的那一趟
#   78  运行条件不满足：preflight 判的（文件头两条 run-condition），或背后有一段退 78（显卡用不上，给定状态表的判器借不到显存，
#       放量入口退 78 而这一项没打 ITEM 行）而这一趟没有红；
#       或起任何 cargo 之前 scripts/hosts/rocksdb-prebuilt.py entry 判红（这台没装预编库、此刻进程环境的取值缺了或对不上；一项都没起，出路是开新会话
#       或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"）。--list-items --with-keys 起读库程序之前同样判，判红退 1
# 编译目录（自己用完自己删，不跨趟复用）：建好这一趟的日志目录之后，调用方没设 CARGO_TARGET_DIR 时在里面建 target、export 给测试函数、
#   崩溃点重放那一道与放量入口（它们看到设了就不另建）；正常结束、红了、被 TERM / INT 停（等手里那一步退出）都在 EXIT 上删
#   （scripts/hosts/run-build-directory.sh；删之前核本机没有进程开着里面的文件，有就不删、报出进程、整趟退非 0）。调用方设了的照它、不删。
# 日志出两份（过万行的全量日志太长，一般只看错误警告）：测试函数、崩溃点重放部分、放量部分各自的日志都经 scripts/hosts/two-logs.sh 写，
#   <日志目录>/<段>.log 是全量，旁边同名的 <段>.problems.log 只收错误与警告（门禁的 ✗ 与 → 出路、⚠、本次未跑、汇总，cargo / rustc 的 error 与 warning:，libtest 的 FAILED 与 panicked，
#   Traceback，非 0 退出码），每行前带它在全量日志里的行号，末行写收了几行、全量几行；开跑时打日志目录，收尾逐份列出 .problems.log，红的原因里点名这一份。
# 弄坏开关 CRASH_POINT_REPLAY_BREAK=<项>（几个用逗号连；只在自证起的子进程里认）。写法同 scripts/crash/crash-amplification.sh 文件头「弄坏开关」一段：
#   一行「#   CRASH_POINT_REPLAY_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（、隔开）：<格名>（「✗ 自证 <格名>：…<失败行的句首>」…）；
#   开关轮读括号外的格名、不读括号里的；门禁 code-tooling 的 break-switch-replay 格读「」里的原文（照它判失败行、从「✗ 自证 <格名>：」取格名选格），自证 switch-table-names-cells 格核每一格都这样写了。
#   --selftest 末尾那一轮（开关轮，crash-amplification-lib.sh 的 crash_amplification_switch_wheel）从这张表现读，表里的开关与代码里 replay_break_is 认的要相等：
#   CRASH_POINT_REPLAY_BREAK=accept-unknown-item  认不得的项名悄悄丢掉、照跑
#       unknown-item（「✗ 自证 unknown-item：」）
#   CRASH_POINT_REPLAY_BREAK=list-items-builds  --list-items 先起一趟 cargo build 再列
#       list-items（「✗ 自证 list-items：」）、list-items-with-keys（「✗ 自证 list-items-with-keys：…--list-items --with-keys 应当照读库程序文件头「输出」那一段的写法报键对不上的块数、不起 cargo」、「✗ 自证 list-items-with-keys：…--list-items --with-keys 遇到读库程序按态数那一行，应当另打一句四个态数、退 0、不起 cargo」）
#   CRASH_POINT_REPLAY_BREAK=test-functions-reuse-not-counted  测试函数每个测试目标的复用数写死 none
#       test-functions-reuse-counted（「✗ 自证 test-functions-reuse-counted：…测试函数日志里有复用计数行时 ITEM 行应当报同一个数……」）
#   CRASH_POINT_REPLAY_BREAK=gpu-cards-absent-refused  开了 GPU 而环境里没有 GPU_CARDS_BY_PRIORITY 就一项都不起、退 78（当成任务要自己拿卡）
#       gpu-without-cards-runs（「✗ 自证 gpu-without-cards-runs：开了 GPU 而环境里没有卡也应当照起」：退 78，一趟测试函数也没起）
#   CRASH_POINT_REPLAY_BREAK=gpu-cards-not-sent  开了 GPU 时测试函数那几趟不交环境里的 GPU_CARDS_BY_PRIORITY
#       gpu-test-functions（「✗ 自证 gpu-test-functions：…开了 GPU 测试函数应当带 gpu 特性、不自己清场…原样交给 cargo」）
#   CRASH_POINT_REPLAY_BREAK=gpu-test-threads-one  开了 GPU 时测试函数那几趟测试参数带 --test-threads 1
#       gpu-test-functions（「✗ 自证 gpu-test-functions：…开了 GPU 测试函数应当带 gpu 特性、不自己清场、测试参数不带测试线程」：调用记录里 gpu 那几趟（三项的测试目标加 WHOLE_RUN_GROUPS 里的测试目标，个数自证现数）的 -- --nocapture 后面多了 --test-threads 1；没开 GPU 的 whole 那一格数的是 -- --nocapture 那几趟，这个开关不动它）、
#       gpu-without-cards-runs（「✗ 自证 gpu-without-cards-runs：开了 GPU 而环境里没有卡也应当照起」：那一格同样数 -- --nocapture 后面紧跟 cards= 的那几趟，多了 --test-threads 1 就对不上）
#   CRASH_POINT_REPLAY_BREAK=test-line-markers-unread  测试函数日志里的 NOT_RUN_NO_GPU_ADAPTER、借不到显存那一行与复用计数行只认行首，不认接在别的测试的「test <名> ... 」后面的
#       test-functions-marker-on-a-test-line（「✗ 自证 test-functions-marker-on-a-test-line：…接在别的测试结果行起头后面的 NOT_RUN_NO_GPU_ADAPTER 也算本次未跑」）
#   CRASH_POINT_REPLAY_BREAK=red-as-green  测试函数 cargo 退非 0、有 FAILED 也记绿
#       test-functions-red（「✗ 自证 test-functions-red：…测试函数一个测试目标红了，那一项应当红、整趟退 1」）、test-functions-red-continues（「✗ 自证 test-functions-red-continues：…测试函数那一段红了，别的两段照旧起……」）、
#       run-target-red（「✗ 自证 run-target-red：…跑完运行目录里不剩它建的编译目录（测试函数红退 1）」：红记成绿、整趟退 0）
#   CRASH_POINT_REPLAY_BREAK=not-run-as-green  测试函数跑到 0 条、有 NOT_RUN_NO_GPU_ADAPTER 也记绿
#       test-functions-marker-on-a-test-line（「✗ 自证 test-functions-marker-on-a-test-line：…」）、test-functions-not-run（「✗ 自证 test-functions-not-run：…测试函数跑到 0 条或没卡没跑的，应当记本次未跑」）、all-not-run（「✗ 自证 all-not-run：…派出的项全是本次未跑，应当退 77」）
#   CRASH_POINT_REPLAY_BREAK=skip-collection-count  放量部分收不回的分项不记红、派出收回不核
#       collection-count（「✗ 自证 collection-count：…放量那一项收不回，应当红、派出收回对不上」）、
#       pipeline-entry-1-still-red（「✗ 自证 pipeline-entry-1-still-red：」）、pipeline-entry-78-not-run（「✗ 自证 pipeline-entry-78-not-run：」）、pipeline-entry-78-red-first（「✗ 自证 pipeline-entry-78-red-first：」：收不回的项一笔都不记）
#   CRASH_POINT_REPLAY_BREAK=enumeration-exit-ignored  崩溃点重放那一道的退出码不看，一律记绿
#       enumeration-red（「✗ 自证 enumeration-red：…崩溃点重放那一道红了，那一项应当红」）、shard-default-merge-red（「✗ 自证 shard-default-merge-red：…判红的片应当退 0」：合并判不出红）
#   CRASH_POINT_REPLAY_BREAK=enumeration-gate-whole-package  带 --item 的跑法起门禁也不带 --check crash-case-replay --crash-cases-only，门禁照整道把验证包按条判一遍
#       enumeration-gate-cases-only（「✗ 自证 enumeration-gate-cases-only：…点名了项的跑法应当起门禁带 --check crash-case-replay --crash-cases-only」）
#   CRASH_POINT_REPLAY_BREAK=clip-cuts-in-character  说明文字只按字节截（clip_utf8 不丢切在汉字中间的半个字符），不成字符的字节带进记录与屏幕
#       record-text-clipped-in-character（「✗ 自证 record-text-clipped-in-character：」）
#   CRASH_POINT_REPLAY_BREAK=records-decoded-strictly  汇总读记录文件按严格 UTF-8 解码，记录里有不成字符的字节就抛 UnicodeDecodeError
#       record-text-with-bad-bytes（「✗ 自证 record-text-with-bad-bytes：」）
#   CRASH_POINT_REPLAY_BREAK=pipeline-red-ignored  放量那一项的 red= 与 verdict=red 不看
#       pipeline-red（「✗ 自证 pipeline-red：…放量那一项红了，那一项应当红」）、last-run-list（「✗ 自证 last-run-list：…上次红的清单不对」：放量红的那一项没记成红）、
#       pipeline-entry-78-red-first（「✗ 自证 pipeline-entry-78-red-first：」：放量红的 alpha 记成绿）
#   CRASH_POINT_REPLAY_BREAK=honor-test-variables-outside-selftest  平常跑时也认只供自证的变量，不拒；关守卫项，整值比，只在它是 CRASH_POINT_REPLAY_BREAK 唯一一项时认，与别的项同写平常跑照拒并点名
#       test-variables-outside-selftest（「✗ 自证 test-variables-outside-selftest：…平常跑时带了只供自证的变量应当退 2」）
#   CRASH_POINT_REPLAY_BREAK=segments-registry-not-sent  不往引擎送按段登记文件的路径
#       segments-registry-sent（「✗ 自证 segments-registry-sent：…测试函数与放量每一趟都应当带按段登记文件的绝对路径」）
#       test-functions-domain-environment-keyed（「✗ 自证 test-functions-domain-environment-keyed：」连带：它判的复用或域环境跟着这一处变）
#   CRASH_POINT_REPLAY_BREAK=segments-registry-unchecked  开跑前不核按段登记文件
#       segments-registry-refused（「✗ 自证 segments-registry-refused：…按段登记文件核不过应当退 2、什么都不起」）
#   CRASH_POINT_REPLAY_BREAK=given-states-run-condition-as-red  测试函数日志里有给定状态表借不到显存那一行也照红记（不退 78）
#       test-functions-run-condition-not-met（「✗ 自证 test-functions-run-condition-not-met：…测试函数日志里有给定状态表借不到显存那一行……整趟退 78」）
#   CRASH_POINT_REPLAY_BREAK=service-threads-dropped  测试函数的 cargo 起之前清掉服务交下来的 CARGO_BUILD_JOBS、RUST_TEST_THREADS，吃 cargo 自己的缺省
#       test-functions-threads-from-service（「✗ 自证 test-functions-threads-from-service：…测试函数每一趟 cargo 都应当原样收到服务交的线程数」）
#   CRASH_POINT_REPLAY_BREAK=logs-single  每一段的日志只写一份全量的（tee，不经 two-logs.sh，没有 .problems.log）
#       two-logs-per-part（「✗ 自证 two-logs-per-part：…测试函数红的那一段应当出两份日志……」）
#   CRASH_POINT_REPLAY_BREAK=pipeline-78-as-red  放量入口退 78 而没打 ITEM 行，照旧记红（收不回）
#       pipeline-entry-78-not-run（「✗ 自证 pipeline-entry-78-not-run：…放量入口退 78 而没打 ITEM 行，两项应当记本次未跑……」）、pipeline-entry-78-red-first（「✗ 自证 pipeline-entry-78-red-first：」：gamma 记了红，不是本次未跑）
#   CRASH_POINT_REPLAY_BREAK=source-changed-reason-dropped  放量分项驱动报判段跑的过程中源码变了（退 77、verdict=not-run）时，本次未跑的原因不点名源码变了（只写 exit=77）
#       pipeline-source-changed-not-run（「✗ 自证 pipeline-source-changed-not-run：」）、
#       record-text-with-bad-bytes（「✗ 自证 record-text-with-bad-bytes：」：这一格也要输出里有源码变了那一句，不点名就没有）
#   CRASH_POINT_REPLAY_BREAK=flow-coverage-table-skipped  跑完不往这一趟的日志目录落覆盖表与记录
#       flow-coverage-table（「✗ 自证 flow-coverage-table：…跑完应当在这一趟的日志目录里落覆盖表 crash-flow-coverage.md 与记录 crash-flow-records.tsv…」）
#   CRASH_POINT_REPLAY_BREAK=rocksdb-entry-unchecked  起 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry
#       rocksdb-entry-refused（「✗ 自证 rocksdb-entry-refused：…验证包依赖 rocksdb 而进程环境里没有预编库的取值，应当在起 cargo 之前退 78…」：往下走起了假 cargo）
#   CRASH_POINT_REPLAY_BREAK=run-target-kept  跑完不删这一趟自己建的编译目录（测试函数在仓根照仓根的 target 编、跨趟留着）
#       run-target-green（「✗ 自证 run-target-green：…跑完运行目录里不剩它建的编译目录（正常退 0）」）、run-target-red（「✗ 自证 run-target-red：…跑完运行目录里不剩它建的编译目录（测试函数红退 1）」）、
#       run-target-term（「✗ 自证 run-target-term：…跑完运行目录里不剩它建的编译目录（被 TERM 停）」）
#   CRASH_POINT_REPLAY_BREAK=shard-phase-ignored  不认 TASK_SERVER_SHARD_PHASE 这几个变量，计划、一片、合并都当没设（文件头「经 task-server 按件分片」）
#       shard-default-plan（「✗ 自证 shard-default-plan：…不带参数应当默认分片」）、shard-default-one-part（「✗ 自证 shard-default-one-part：…默认分片的一片应当只跑自己的件」）、shard-default-merge-red（「✗ 自证 shard-default-merge-red：…判红的片应当退 0」）、
#       shard-default-many-parts（「✗ 自证 shard-default-many-parts：…件数超过片数上限应当合成」）、shard-position-refused（「✗ 自证 shard-position-refused：…片号对不上计划或阶段认不得应当退 2」）、shard-merge-refuses-incomplete（「✗ 自证 shard-merge-refuses-incomplete：…有一片的产物不全」）、shard-plan-lines（「✗ 自证 shard-plan-lines：」）
#   CRASH_POINT_REPLAY_BREAK=shard-red-exit-kept  一片判红照原样退 1（别的片被服务收掉、合并起不来）
#       shard-default-merge-red（「✗ 自证 shard-default-merge-red：…判红的片应当退 0」）
#   CRASH_POINT_REPLAY_BREAK=unsharded-run-accepted  不经服务起（没有 TASK_SERVER_SHARD_PHASE）的平常跑法也照整份计划判，不退 2
#       unsharded-run-refused（「✗ 自证 unsharded-run-refused：…不经服务起应当退 2」）
#   CRASH_POINT_REPLAY_BREAK=merge-incomplete-accepted  合并不核各片的产物全不全（缺的片当没判）
#       shard-merge-refuses-incomplete（「✗ 自证 shard-merge-refuses-incomplete：…有一片的产物不全」：退了别的码）
#   CRASH_POINT_REPLAY_BREAK=plan-shards-not-merged  件数超过服务的片数上限也一件一片（服务记 shard-plan-refused）
#       shard-default-many-parts（「✗ 自证 shard-default-many-parts：…件数超过片数上限应当合成」）
#   CRASH_POINT_REPLAY_BREAK=test-functions-skip-ignored  测试函数照算计划，键相同的也不经 --skip 跳过、照编照跑（每个测试目标整趟现判）
#       reuse-three-questions（「✗ 自证 reuse-three-questions：」：测试函数那一半第二趟照样全现判）、
#       test-functions-reuse-shared-with-the-gate（「✗ 自证 test-functions-reuse-shared-with-the-gate：」：剪枝那一趟没复用逐条写进的）、
#       red-rejudged（「✗ 自证 red-rejudged：」：上一趟判绿的那一条没复用）、
#       test-functions-domain-environment-keyed（「✗ 自证 test-functions-domain-environment-keyed：」连带：它判的复用或域环境跟着这一处变）、red-revokes-green（「✗ 自证 red-revokes-green：」连带：它判的复用或域环境跟着这一处变）
#       registered-excluded（「✗ 自证 registered-excluded：…按项跑时登记成 crash-case: 的用例与放量分项应当经 --skip 排掉」：参数里没有 --skip）
#   CRASH_POINT_REPLAY_BREAK=test-functions-directory-root-unseen  按条复用认测试目标根文件只认平铺的 tests/<目标>.rs，目录式目标（自证小仓里的 beta：tests/beta/main.rs）的条数、--skip 名单、逐条结局与 ITEM_PLAN 都落空
#       reuse-three-questions（「✗ 自证 reuse-three-questions：」：头一趟的输出里没有 beta_stands_alone 那一条逐条结局）、test-functions-reuse-shared-with-the-gate（「✗ 自证 test-functions-reuse-shared-with-the-gate：」：库里少了 beta 那一条）、red-rejudged（「✗ 自证 red-rejudged：」：库里记着红的 beta_stands_alone 那一趟没打逐条结局）、item-plan-lines（「✗ 自证 item-plan-lines：」：to_judge 之和少了 beta 的那一条）
#   CRASH_POINT_REPLAY_BREAK=test-functions-not-recorded  测试函数现判出的判定不经 verdict-record 写回（测试函数跑了不入库）
#       reuse-three-questions（「✗ 自证 reuse-three-questions：」：测试函数那一半第二趟照样全现判）、
#       test-functions-reuse-shared-with-the-gate（「✗ 自证 test-functions-reuse-shared-with-the-gate：」：库里一条都没有）、
#       red-rejudged（「✗ 自证 red-rejudged：」：上一趟判绿的那一条没复用）、
#       test-functions-domain-environment-keyed（「✗ 自证 test-functions-domain-environment-keyed：」连带：它判的复用或域环境跟着这一处变）、red-revokes-green（「✗ 自证 red-revokes-green：」连带：它判的复用或域环境跟着这一处变）
#   CRASH_POINT_REPLAY_BREAK=test-functions-kind-not-gate  测试函数的判定种类不用门禁那一类（记进、读另一份库）
#       test-functions-reuse-shared-with-the-gate（「✗ 自证 test-functions-reuse-shared-with-the-gate：」：门禁那一类的库里没有逐条写进的）
#       red-revokes-green（「✗ 自证 red-revokes-green：」连带：它判的复用或域环境跟着这一处变）
#   CRASH_POINT_REPLAY_BREAK=red-reused-as-green  测试函数现判红的测试函数也照判绿写回，test-plan 读库时库里记着红的也当判定（admission.py 的 ADMISSION_BREAK=red-verdict-reused）
#       red-rejudged（「✗ 自证 red-rejudged：」：红过的那一条第二趟被跳过）
#       red-revokes-green（「✗ 自证 red-revokes-green：」连带：它判的复用或域环境跟着这一处变）
#   CRASH_POINT_REPLAY_BREAK=test-functions-plan-failure-not-red  按条复用失灵（test-functions-request 或 test-plan 退非 0）只全部现判、不判红
#       test-functions-plan-failure-red（「✗ 自证 test-functions-plan-failure-red：」）、
#       record-text-clipped-in-character（「✗ 自证 record-text-clipped-in-character：」：这一格靠 test-functions-request 没成判红，不判红就整趟退 0）
#   CRASH_POINT_REPLAY_BREAK=test-functions-plan-unavailable-as-failure  测试函数 test-plan 退 75 照按条复用失灵办（全部现判、判红、整趟退 1），不退 75
#       test-functions-plan-unavailable（「✗ 自证 test-functions-plan-unavailable：」）
#   CRASH_POINT_REPLAY_BREAK=last-run-whole-overwrite  上次红的清单整份覆盖（不点名的跑法不交这一趟真判过的项）
#       last-run-list（「✗ 自证 last-run-list：」：本次未跑的那一项上一份的红丢了）
#   CRASH_POINT_REPLAY_BREAK=red-not-recorded  测试函数现判红的测试函数不交 verdict-record --red-names（只记绿，判红不撤同一把键的绿）
#       red-revokes-green（「✗ 自证 red-revokes-green：」：同键现判那一趟判红之后，下一趟那一条照复用）
#   CRASH_POINT_REPLAY_BREAK=test-functions-without-library-tolerated  被判的根不是一棵 git 工作树的顶时测试函数全部现判、不入库、往下跑（不退 2）
#       test-functions-outside-git-exits-2（「✗ 自证 test-functions-outside-git-exits-2：」：不是 git 工作树的副本那一趟没退 2、照编照跑）
#   CRASH_POINT_REPLAY_BREAK=test-functions-domain-environment-unkeyed  test-functions-request 带 ADMISSION_BREAK=domain-environment-unkeyed 起（改判定域的环境变量不进键）
#       test-functions-domain-environment-keyed（「✗ 自证 test-functions-domain-environment-keyed：」：按段登记文件改了，闭包读它的那一条照复用）
#   CRASH_POINT_REPLAY_BREAK=registered-not-excluded  不排登记成 crash-case: 的用例与放量分项（test-plan 不带 --exclude-registered），它们也按条复用地跑
#       registered-excluded（「✗ 自证 registered-excluded：…按项跑时登记成 crash-case: 的用例与放量分项应当经 --skip 排掉……」）
#   CRASH_POINT_REPLAY_BREAK=whole-run-item-unnamed  --item 点不到整趟才跑的那几类（whole-run:<类> 当不认得的项；--list-items 与不带参数的跑法照旧）
#       whole-run-item（「✗ 自证 whole-run-item：…--item whole-run:<类> 应当只跑那一类的测试目标」：退 2）
#   CRASH_POINT_REPLAY_BREAK=expand-up-to-not-passed  给了 --crash-expand-up-to（或环境里已设缩域变量）不把变量交给测试函数、崩溃点重放、放量三段（输出照写缩过域）
#       expand-up-to-passed（「✗ 自证 expand-up-to-passed：」）
#   CRASH_POINT_REPLAY_BREAK=judge-expand-variable-not-passed  给了缩域值，判器那个变量（SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO）不跟着设（只交放量那一个）
#       expand-up-to-passed（「✗ 自证 expand-up-to-passed：」）
#   CRASH_POINT_REPLAY_BREAK=expand-up-to-unchecked  --crash-expand-up-to 与环境里缩域变量的值不核是不是非负整数
#       expand-up-to-refused（「✗ 自证 expand-up-to-refused：」）
#   CRASH_POINT_REPLAY_BREAK=item-plan-reusable-zero  --list-items --with-keys 的 ITEM_PLAN 里 reusable 一律 0（键相同的测试函数不数）
#       item-plan-lines（「✗ 自证 item-plan-lines：…--list-items --with-keys 应当每项一行 ITEM_PLAN、条数照 test-plan 的计划数」）
#   CRASH_POINT_REPLAY_BREAK=item-plan-selection-ignored  --list-items --with-keys 带 --item 时照样给全部项各打一行
#       item-plan-lines（「✗ 自证 item-plan-lines：」）
#   CRASH_POINT_REPLAY_BREAK=item-done-reused-silent  ITEM_DONE 不为全部复用的项（现判 0）打行
#       item-done-lines（「✗ 自证 item-done-lines：…真跑应当每项一行 ITEM_DONE、件数与退出码照各段的记录」）
#   CRASH_POINT_REPLAY_BREAK=item-done-seconds-whole-run  ITEM_DONE 的 seconds 写整趟各项秒数之和，不是这一项自己的
#       item-done-lines（「✗ 自证 item-done-lines：」）
#   CRASH_POINT_REPLAY_BREAK=shard-plan-lines-merged  分片计划行每项只打一行（不分片号、各片的条数相加）
#       shard-plan-lines（「✗ 自证 shard-plan-lines：」）
#   CRASH_POINT_REPLAY_BREAK=keys-reader-build-differs  --with-keys 的读库程序那一趟 cargo run 不带真跑那一套特性
#       keys-reader-same-build（「✗ 自证 keys-reader-same-build：」）
#   CRASH_POINT_REPLAY_BREAK=keys-states-line-unread  --with-keys 不认读库程序的 CRASH_LIBRARY_STATES 行（不打态数那一句）
#       list-items-with-keys（「✗ 自证 list-items-with-keys：」：带按态数那一行的样本里没有态数那一句）
#   CRASH_POINT_REPLAY_BREAK=keys-states-line-unchecked  --with-keys 不核 CRASH_LIBRARY_STATES 行的字段，认不出的也照打
#       list-items-with-keys（「✗ 自证 list-items-with-keys：」：字段写坏的那一份样本退 0）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-unbounded  带 --item、没缩域也不判指定范围的崩溃域上限
#       scoped-domain-over-limit（「✗ 自证 scoped-domain-over-limit：」）、scoped-domain-unknown-refused（「✗ 自证 scoped-domain-unknown-refused：」）、
#       scoped-domain-shrunk-record-skipped（「✗ 自证 scoped-domain-shrunk-record-skipped：」）、
#       scoped-domain-reusable-passes（「✗ 自证 scoped-domain-reusable-passes：」连带：守卫不跑，全部复用那一句不打）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-narrowed-refused  缩了域（--crash-expand-up-to 小于引擎常量）也照没缩判：缩域值不交给判法
#       scoped-domain-narrowed-passes（「✗ 自证 scoped-domain-narrowed-passes：」）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-reusable-counted  库里录下的块键都对得上、全部可复用的项也按上一趟全域真跑的态数算（上一趟有记录才算）
#       scoped-domain-reusable-passes（「✗ 自证 scoped-domain-reusable-passes：」）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-unknown-passed  估不出现判多少态的项按 0 态放行
#       scoped-domain-unknown-refused（「✗ 自证 scoped-domain-unknown-refused：」）、scoped-domain-shrunk-record-skipped（「✗ 自证 scoped-domain-shrunk-record-skipped：」）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-suggestion-dropped  拒绝说明里不写算得出的 n
#       scoped-domain-over-limit（「✗ 自证 scoped-domain-over-limit：」）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-shrunk-records-used  估上一趟全域态数时不看展开深度，取最新的一份记录（缩过域的那趟也当全域）
#       scoped-domain-shrunk-record-skipped（「✗ 自证 scoped-domain-shrunk-record-skipped：」）
#   CRASH_POINT_REPLAY_BREAK=enumeration-scope-not-recorded  崩溃点重放部分跑完不在日志旁写 .scope（展开深度与用例日志目录）
#       scoped-domain-enumeration-records（「✗ 自证 scoped-domain-enumeration-records：」）
#   CRASH_POINT_REPLAY_BREAK=item-domain-line-dropped  --list-items --with-keys 不打 ITEM_DOMAIN 行
#       item-domain-line（「✗ 自证 item-domain-line：」）、scoped-domain-enumeration-records（「✗ 自证 scoped-domain-enumeration-records：」：ITEM_DOMAIN 行没有）、item-domain-single-reader（「✗ 自证 item-domain-single-reader：」：一个读库程序都不起、一行都没有）
#   CRASH_POINT_REPLAY_BREAK=item-domain-reader-per-item  --list-items --with-keys 算 ITEM_DOMAIN 时每项各交一趟判法、各起一次读库程序（逐项起）
#       item-domain-single-reader（「✗ 自证 item-domain-single-reader：」：读库程序起了几次）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-handshake-dropped  起放量入口时不交 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1（放量入口会再判一遍）
#       scoped-domain-checked-handshake（「✗ 自证 scoped-domain-checked-handshake：」）
#   CRASH_POINT_REPLAY_BREAK=failure-text-cut-bytes  自证的失败行转打片段只按字节切、不丢不成字符的字节，半个字原样打到标准错误（selftest_clip_text、selftest_valid_text）
#       failure-text-clipped-in-character（「✗ 自证 failure-text-clipped-in-character：」）
#   CRASH_POINT_REPLAY_BREAK=cell-groups-by-name  整份自证把格分路时一个格名一组、不看一块点名了几个格名
#       selftest-worker-cells（「✗ 自证 selftest-worker-cells：整份自证应当把格按组分到几路」）
#   CRASH_POINT_REPLAY_BREAK=parallel-workers-serial  整份自证的各路一路一路地起（并行度当 1）
#       selftest-workers-run-together（「✗ 自证 selftest-workers-run-together：几路应当同时在跑」）
#   CRASH_POINT_REPLAY_BREAK=parallel-failure-dropped  回读各路时不看退出码与失败行，只数趟数
#       selftest-workers-run-together（「✗ 自证 selftest-workers-run-together：几路应当同时在跑」）
#   CRASH_POINT_REPLAY_BREAK=selftest-cells-run-extra  点名了格时 unknown-item 那一块也跑
#       selftest-cells-selected（「✗ 自证 selftest-cells-selected：」）
#   CRASH_POINT_REPLAY_BREAK=selftest-cells-unknown-accepted  CRASH_POINT_REPLAY_SELFTEST_CELLS 里写错的格名不退 2
#       selftest-cells-selected（「✗ 自证 selftest-cells-selected：」）
#   CRASH_POINT_REPLAY_BREAK=switch-table-check-blind  文件头开关表点名的格没写「✗ 自证 <格名>：」也认不出来
#       switch-table-names-cells（「✗ 自证 switch-table-names-cells：」）
#   CRASH_POINT_REPLAY_BREAK=switch-wheel-progress-silent  开关轮每收回一条开关不打进度行（selftest_switch_wheel_with_progress）
#       switch-wheel-progress-lines（「✗ 自证 switch-wheel-progress-lines：」）
#   CRASH_POINT_REPLAY_BREAK=switch-wheel-always-whole  开关轮不在整轮门禁里也每条开关跑整份自证，不只跑开关表给它点名的格（selftest_switch_wheel_with_progress）
#       switch-wheel-named-cells（「✗ 自证 switch-wheel-named-cells：」）
#   CRASH_POINT_REPLAY_BREAK=switch-wheel-segment-drops-one  开关轮分段时表里最后一条开关不落进任何一段（selftest_wheel_segment_indexes）
#       switch-wheel-segments-partition（「✗ 自证 switch-wheel-segments-partition：」）
# 只供自证（自证起的子进程里才认；平常跑时有一个非空就在参数解析之前退 2 拒跑，只有 --selftest 本身不拒，--list-items 与认不得的选项也先看这个）：CRASH_POINT_REPLAY_BREAK=<项>（上面那张表）；
#   CRASH_POINT_REPLAY_ROOT=<造出来的小仓>：合表、stage-inputs.tsv、门禁 crash-point-replay、crash-amplification.sh 都从它取
#   （门禁与放量入口那两份是自证写的假的：只记一行参数、照环境变量打结果）；按条复用调的是真的 scripts/gate/admission.py（在小仓上算键、查库、记库）。
#   按条复用那几格另用一份带小工作区、做成 git 仓的小仓（自证里的 unit-root，PATH 最前面另放只答版本的假 rustc）；别的格的小仓也是带小工作区、提交过一次的 git 仓
#   （测试函数照真的 test-plan 算计划；那份小仓的测试文件是空的，没有可复用、可记的测试函数，测试函数逐个测试目标现判）；
#   CRASH_POINT_REPLAY_KEYS_READER_OUTPUT=<文件>：--list-items --with-keys 不起读库程序，拿这份文件的内容当它的输出（list-items-with-keys 格用）。
#   CRASH_POINT_REPLAY_MOST_SHARDS=<片数>：按件分片的片数上限换成它（shard-default-many-parts 格用：小仓的件数够不到服务的上限）。
#   CRASH_POINT_REPLAY_SELFTEST_CELLS=<格名,…>：只跑点名的格（下面「选格」）。
#   CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT=<段号>/<段数>|base：开关轮分段点名单跑（下面「开关轮分段」）。
#   CRASH_POINT_REPLAY_UNSHARDED_REFUSED=1：自证起的子进程不经服务起时照平常跑拒（退 2；unsharded-run-refused 格用，文件头「经 task-server 按件分片」）。
#   「自证起的子进程」的标记：CRASH_POINT_REPLAY_SELFTEST_MARK=<文件>，那份文件在、内容是 SELFTEST_MARK_CONTENT 那一行；只有 run_selftest 设它。
#   这几个变量读完就从环境里清掉，不传给背后的驱动。自证里的 cargo 是 PATH 最前面的假 cargo（起之前核 PATH 上解到的就是它，不是就不起）；
#   多机配置指到临时目录里只写了假清场命令的一份（放量入口是假的，这一份不清场），状态与日志目录都在临时目录里。
# 选格：CRASH_POINT_REPLAY_SELFTEST_CELLS=<格名,…> bash scripts/crash/crash-point-replay.sh --selftest（经临时服务起，同不选格的那一趟）只跑点名的格、不跑末尾的开关轮，单跑一格与它在全部格里跑判定相同；
#   格名是失败行「✗ 自证 <格名>：」里那一段，写错退 2 并列出认得的格。门禁 code-tooling 的 break-switch-replay 格指定范围时经它每条开关只起点名的格。
#   准备的小仓、桩与函数一律先建好，每一块用 selftest_cell_wanted 判跑不跑。块与块之间读别人留下的只有两处，点名其中后一格时前一块照跑、前一格判错照报：
#   shard-plan-lines 比 item-plan-lines 那一趟的列项；test-functions-threads-from-service 判 segments-registry-sent 那一趟的调用记录。别的格各自建计划、运行记录与库。
#   不设这个变量（不指定范围那一趟）全部格按次序跑、再跑开关轮。
#   弄坏开关 selftest-cells-run-extra、selftest-cells-unknown-accepted 见下面开关表，这两条由 selftest-cells-selected 那一格证红。
# 开关轮分段：CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT=<段号>/<段数> bash scripts/crash/crash-point-replay.sh --selftest（经临时服务起，同不点名的那一趟）只跑开关轮里那一段的开关，
#   不跑基准趟各格；整轮开关一次跑下来超过一次运行的上限（.claude/rules/long-run-split.md「怎么做」：不超过 7200 秒），拆成几段各自一次运行。
#   写法：<段号>/<段数>，段数只能是 WHEEL_SEGMENT_COUNT，段号从 1 到段数；base 只跑基准趟各格（并行跑完、不跑开关轮）。写错退 2，列出可点的写法；与 CRASH_POINT_REPLAY_SELFTEST_CELLS
#   或 CRASH_POINT_REPLAY_BREAK 同给也退 2。不点名时整份照旧：基准趟各格并行跑完再跑整轮开关轮。
#   分法（selftest_wheel_segment_indexes）：开关表里第 i 条（从 0 数）落在第 (i 除以段数的余数 + 1) 段，每条开关恰好一段；段数相等、段与段互不相交、各段的并集是整张表。
#   段数的算法：每条开关单跑一份整份自证取至多 864 秒（负载下量到的单条上限），7200 ÷ 864 向下取整，每段至多 WHEEL_SEGMENT_SWITCHES_MOST = 8 条；
#   WHEEL_SEGMENT_COUNT = ⌈开关条数 ÷ 8⌉（开关表 75 条时取 10：每段 7 或 8 条）；开关条数涨到每段超过 8 条时 switch-wheel-segments-partition 那一格红，改 WHEEL_SEGMENT_COUNT。
#   这一段跑完打「✓ --selftest 只跑开关轮第 <段号>/<段数> 段：这一段跑了 <m> 条开关、开关表共 <n> 条……」，进度行里的「表里第 <序号> 条」是这条开关在整张表里的序号；
#   这一趟只核这一段的开关，基准趟各格与别的段没跑，不算门禁通过（各段与 base 全跑过才抵得上不点名的整份）。
#   自证 switch-wheel-segments-partition 格核：真开关表里各段的并集等于全部开关、互不相交、每段不超过每段至多条数，样本的分法、写法的认与拒、写错与同给时退 2；
#   弄坏开关 switch-wheel-segment-drops-one（下面开关表）。
# 自证末尾的开关轮并行度 CRASH_AMPLIFICATION_SELFTEST_JOBS，没给取 SINGLEFS_THREAD_CAP，再没给取登记的重活默认并行度（同 crash-amplification-lib.sh 文件头「弄坏开关轮」）。
#   开关轮由 selftest_switch_wheel_with_progress 跑（读表与判一条对不对得上用共用库的 crash_amplification_read_switch_table、crash_amplification_switch_wheel_verdict）：
#   整轮跑一个多小时，每收回一条开关当场打一行「· 开关轮进度：收回 <k>/<条数>（表里第 <序号> 条）<开关>，用了 <秒> 秒，红在点名的格「…」」（对不上的写「没红在点名的格：…」、
#   另往标准错误打一行 ✗），进度行只由开关轮这一层打、一行一次写，几路同时跑的行不交错（.claude/rules/implementation-workflow.md「测试与崩溃检测优先多线程」的进度间隔）。
#   自证 switch-wheel-progress-lines 格拿三条假开关核「每条一行、条数与开关表相同」。
# 自证并行（.claude/rules/test-script-discipline.md「编译与算法都要快」）：整份 --selftest（没点名格、没设弄坏开关、不是开关轮里起的一份）开关轮之前的各格也按这个并行度并行：
#   run_selftest_parallel 把格按组分到至多这么多路（selftest_worker_cells：一块点名几个格名、几块点名同一个格名的并成一组，同一组必须同一路，因为块与块之间读别人留下的只有「选格」那一段写的两处；
#   分量 = 块数加块里起入口的次数，大的先分、放到此刻最轻的一路），每路是这份脚本自己的 --selftest 点名它那几个格（CRASH_POINT_REPLAY_SELFTEST_CELLS），各建自己的临时目录、小仓与假工具，
#   输出各写各的文件；至多这么多路同时在跑（第 i 路等第 i-路数 路收完再起），退出码按序号收进数组；全部收回后按序号回读：标准错误原样转出，每路自己的成功句不转，趟数加起来，派出收回要相等。
#   有一路红整份退 1、不跑开关轮；都对了才跑开关轮（并行度同上，每个开关一份串行的自证：整轮门禁里（环境里 GATE_IN_STAGE=1）跑整份、核红的格恰好是点名的格，别处只跑开关表给它点名的那几格、核那几格都红，别的格跟没跟着红留给整轮门禁那一趟；表里没列的开关照旧跑整份），成功句报「自证并行 N 路」「开关轮并行 N 路」
#   （两句由本文件里的 selftest_report_parallel_cells、selftest_report_parallel_wheel 打：门禁 code-tooling 的 selftest-parallelism 格只读登记成自证的这一份文件里有没有报并行度的一行）。
#   没有留在一串里的格：每路各有自己的临时目录，同组的格在同一路里按脚本里的次序跑。设了弄坏开关的整份与开关轮里起的一份不并行（门禁 break-switch-replay 不指定范围时每条开关整份起，
#   并行度由它定，这里不叠一层）；点名了格的串行跑点名的格。自证 selftest-worker-cells（分路）、selftest-workers-run-together（同时在跑、按序收、失败行不吞）；
#   弄坏开关 cell-groups-by-name、parallel-workers-serial、parallel-failure-dropped（下面开关表）。
#
# 分在两份文件里：自证（run_selftest、自证并行、开关轮分段与开关轮）写在同目录 crash-point-replay-selftest.sh 里，本文件在 --selftest 那一支里 source 它
#   （路径照本文件自己所在的目录 replay_script_directory 找，存进 replay_selftest_library；第二台在同一个绝对路径上起本文件，同一个目录里就有它）；
#   开关表仍写在本文件，读它的 crash_amplification_read_switch_table 与 replay_switch_rows_without_cells 读本文件；
#   自证里「if selftest_cell_wanted <格名>」块与代码里 replay_break_is 认的开关，开关轮核表与代码相同时读本文件与 crash-point-replay-selftest.sh 两份。
#   弄坏开关 cell-groups-by-name、parallel-failure-dropped、parallel-workers-serial、selftest-cells-run-extra、selftest-cells-unknown-accepted、switch-wheel-always-whole、
#   switch-wheel-progress-silent、switch-wheel-segment-drops-one 的代码在 crash-point-replay-selftest.sh 里，别的开关的代码都在本文件。
#
# gate-similar: crash-amplification.sh 它是放量分项的登记表与全部项的循环；这里是合表的入口，放量部分每趟调它一次，崩溃点重放部分调门禁 crash-point-replay
# gate-similar: crash-point-replay.sh 它跑崩溃点重放测试函数与登记的崩溃枚举用例；这里照项调它的 --item，测试函数那一段只跑合表点名的部分的测试目标
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

SELFTEST_MARK_CONTENT="crash-point-replay.sh --selftest 起的子进程"
# 自证成功句里报并行度的两句（crash-point-replay-selftest.sh 的 run_selftest_parallel 调；文件头「自证并行」）：<各格分到的路数> <开关轮的路数>
selftest_report_parallel_cells() { printf '自证并行 %d 路（各格按组分路，每路是这份脚本自己的 --selftest 点名它那几个格，各用自己的临时目录与小仓，按序号回读）；开关轮并行 %d 路' "$1" "$2"; }
selftest_report_parallel_wheel() { printf '开关轮并行 %d 路' "$1"; }
WHOLE_RUN_GROUPS=(
  $'gpu-and-cpu-agree-on-small-inputs\tcrash_judge_gpu_agrees_with_the_cpu_judge_on_every_state crash_facts_gpu_verifier_agrees_with_the_cpu_reference gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core crash_judge_gpu_secondary_reads_after_the_source_segment_is_discarded'
  $'pruning-self-proof\tcrash_amplification_two_flows_share_their_prefix crash_amplification_pipeline_records_checks_and_reuses'
  $'pinned-red-examples\t-'
)
PINNED_RED_EXAMPLES_NOT_RUN_REASON="崩溃点重放还没有钉表：判出的红要钉成一条用例，今天一条都没钉"

replay_script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
replay_script_directory="$(dirname "$replay_script")"
replay_selftest_library="$replay_script_directory/crash-point-replay-selftest.sh"   # 自证（run_selftest 与并行、开关轮那几段）；--selftest 那一支 source 它，自证读「if selftest_cell_wanted」块与 replay_break_is 的地方读它
source "$replay_script_directory/crash-amplification-lib.sh"
source "$replay_script_directory/../hosts/run-build-directory.sh"
repository_root="$(cd "$replay_script_directory/../.." && pwd)"
# 按条复用（文件头「按条复用」）：准入模块与判法根都取这份脚本所在的仓（自证换了 repository_root 也照旧：在小仓上算键，判法照真的这一份）；
# 判定种类按门禁的阶段文件名起（admission.py test-functions-request 拼出 <它>:test-functions）
replay_admission_module="$replay_script_directory/../gate/admission.py"
replay_judge_root="$repository_root"
TEST_FUNCTIONS_GATE_FILE_NAME="crash-point-replay.sh"

# 只供自证的几个变量（文件头「只供自证」）：读完就清掉
replay_in_selftest_child=0
if [[ -n "${CRASH_POINT_REPLAY_SELFTEST_MARK:-}" && -f "${CRASH_POINT_REPLAY_SELFTEST_MARK:-}" ]] \
   && [[ "$(head -n 1 -- "$CRASH_POINT_REPLAY_SELFTEST_MARK")" == "$SELFTEST_MARK_CONTENT" ]]; then
  replay_in_selftest_child=1
fi
replay_break="${CRASH_POINT_REPLAY_BREAK:-}"
replay_root_override="${CRASH_POINT_REPLAY_ROOT:-}"
replay_keys_reader_output="${CRASH_POINT_REPLAY_KEYS_READER_OUTPUT:-}"
replay_most_shards_override="${CRASH_POINT_REPLAY_MOST_SHARDS:-}"
replay_selftest_cells="${CRASH_POINT_REPLAY_SELFTEST_CELLS:-}"
replay_wheel_segment="${CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT:-}"
replay_unsharded_refused="${CRASH_POINT_REPLAY_UNSHARDED_REFUSED:-}"
replay_test_variables=()
[[ -n "$replay_break" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_BREAK)
[[ -n "$replay_root_override" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_ROOT)
[[ -n "$replay_keys_reader_output" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_KEYS_READER_OUTPUT)
[[ -n "$replay_most_shards_override" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_MOST_SHARDS)
[[ -n "$replay_selftest_cells" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_SELFTEST_CELLS)
[[ -n "$replay_wheel_segment" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT)
[[ -n "$replay_unsharded_refused" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_UNSHARDED_REFUSED)
unset CRASH_POINT_REPLAY_BREAK CRASH_POINT_REPLAY_ROOT CRASH_POINT_REPLAY_SELFTEST_MARK CRASH_POINT_REPLAY_KEYS_READER_OUTPUT CRASH_POINT_REPLAY_MOST_SHARDS CRASH_POINT_REPLAY_SELFTEST_CELLS \
  CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT CRASH_POINT_REPLAY_UNSHARDED_REFUSED
replay_break_is() { [[ ",$replay_break," == *",$1,"* ]]; }
# clip_utf8 <字节数>：标准输入每行取前 <字节数> 个字节，再丢掉被切在中间的半个字符。GNU 的 cut -c 与 cut -b 一样按字节切，三个字节的汉字会被切断；
#   切断的字节写进记录文件、打到屏幕与日志里，汇总和别的按 UTF-8 解码读它们的程序就崩。切完用 iconv -c 去掉不成字符的尾巴，输出总是合法的 UTF-8。
#   弄坏开关 clip-cuts-in-character：只按字节切
clip_utf8() {
  if replay_break_is clip-cuts-in-character; then
    cut -b "1-$1"
  else
    cut -b "1-$1" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null
  fi
}
# 自证自己打的失败行（run_selftest 的 cell_failed 与它引的片段）用这两个，不用 clip_utf8：开关 clip-cuts-in-character 弄坏的是被测的 clip_utf8，它红的那一格要把坏字节留在 $work/out 里，
#   失败行转打那一段时不能跟着把半个字打进自证的标准错误。selftest_clip_text <字节数> 同 clip_utf8 的切法，selftest_valid_text 丢掉整段输出里不成字符的字节。
#   弄坏开关 failure-text-cut-bytes：两个都不过 iconv，半个字原样打到标准错误（failure-text-clipped-in-character 红）
selftest_clip_text() {
  if replay_break_is failure-text-cut-bytes; then
    cut -b "1-$1"
  else
    cut -b "1-$1" | iconv -c -f UTF-8 -t UTF-8 2>/dev/null
  fi
}
selftest_valid_text() {
  if replay_break_is failure-text-cut-bytes; then
    cat
  else
    iconv -c -f UTF-8 -t UTF-8 2>/dev/null
  fi
}
# replay_switch_rows_without_cells <脚本文件>：文件头开关表里点名的格（括号外的格名）没有写成「✗ 自证 <格名>：…」的，一条打一行「<开关名> <格名>」，全写了什么都不打。
#   门禁 code-tooling 的 break-switch-replay 格指定范围时从「」里的这一段读格名，读不出的那条开关整份起（连带基准趟整份起、带开关轮）。
#   弄坏开关 switch-table-check-blind：什么都不打（switch-table-names-cells 红）
replay_switch_rows_without_cells() {
  replay_break_is switch-table-check-blind && return 0
  python3 - "$1" <<'PY_SWITCH_ROWS'
import re, sys
start = re.compile(r"^#   CRASH_POINT_REPLAY_BREAK=([a-z0-9-]+)(\s|$)")
rows, current = [], None
with open(sys.argv[1], encoding="utf-8") as handle:
    for line in handle:
        line = line.rstrip("\n")
        if not line.startswith("#"):
            break
        found = start.match(line)
        if found:
            current = [found.group(1), ""]
            rows.append(current)
        elif current is not None and line.startswith("#       "):
            current[1] += " " + line[1:]
        else:
            current = None
for name, text in rows:
    outside, depth = "", 0
    for character in text:
        if character in "（(":
            depth += 1
        elif character in "）)":
            depth = max(0, depth - 1)
        elif depth == 0:
            outside += character
    for cell in re.findall(r"[a-z0-9]+(?:-[a-z0-9]+)*", outside):
        if f"自证 {cell}：" not in text:
            print(name, cell)
PY_SWITCH_ROWS
}
# 关守卫项 honor-test-variables-outside-selftest 整值比：只在它是 CRASH_POINT_REPLAY_BREAK 唯一一项时认（与别的项同写，平常跑照拒；开关轮按「replay_break_is <开关>」数代码里认的开关，这一句要写出它的名字）
replay_guard_off_honored() { [[ "$replay_break" == honor-test-variables-outside-selftest ]] && replay_break_is honor-test-variables-outside-selftest; }
# 起任何 cargo 之前判这台的 RocksDB 预编库取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」：这台装了、此刻进程环境的取值对得上）。判红交回 1，
# entry 自己打 ✗ 与出路；树里没有依赖 rocksdb 的清单、PATH 上是带标记的假 cargo，entry 报不判、交回 0。弄坏开关 rocksdb-entry-unchecked 下不判
replay_rocksdb_entry_holds() {
  replay_break_is rocksdb-entry-unchecked && return 0
  python3 "$replay_script_directory/../hosts/rocksdb-prebuilt.py" entry --root "$repository_root" >&2
}

# 交给测试函数 cargo 的那一份环境（env -u … 变量=值 … 那一串，放进 functions_environment）：run_test_functions 起测试函数、plan_test_functions 算键、
#   --list-items --with-keys 编读库程序都用它。调用方环境里的 SINGLEFS_CRASH_AMPLIFICATION_* 一律清掉，缩过域的那一趟只把缩域变量原样带上（文件头「往引擎送的变量」）；
#   编译与测试线程照服务交下来的 CARGO_BUILD_JOBS、RUST_TEST_THREADS 原样继承（弄坏开关 service-threads-dropped 下清掉这两个，吃 cargo 自己的缺省）
test_functions_environment() {
  local variable
  functions_environment=(env -u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE -u GPU_CARDS_BY_PRIORITY)
  for variable in $(compgen -e); do [[ "$variable" == SINGLEFS_CRASH_AMPLIFICATION_* ]] && functions_environment+=(-u "$variable"); done
  # -u 一律写在「变量=值」之前：env 见到头一个「变量=值」之后就不再认选项，后面的 -u 会被当成要起的命令
  replay_break_is service-threads-dropped && functions_environment+=(-u CARGO_BUILD_JOBS -u RUST_TEST_THREADS)
  if [[ -n "$crash_expand_up_to_origin" ]] && ! replay_break_is expand-up-to-not-passed; then
    functions_environment+=("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=$crash_expand_up_to")
  fi
  functions_environment+=("BINDGEN_EXTRA_CLANG_ARGS=$CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS")
  [[ -f "$configuration_file" ]] && functions_environment+=("SINGLEFS_MULTI_HOST_CONFIG=$(cd "$(dirname "$configuration_file")" && pwd)/$(basename "$configuration_file")")
}

# 读库程序要与真跑编同一套特性（文件头「--list-items --with-keys」）：打逗号隔开的特性串 = <第二个参数里给的特性（gpu 开着是 gpu）> 加上 <验证包清单> 的 [dev-dependencies]
#   里每个带特性的依赖的 <依赖>/<特性>；清单读不了交回 1
keys_reader_features() { # <验证包的 Cargo.toml> <真跑带的特性，逗号隔开，可空>：算法只在 lib_crash_point_replay_items.py 的 reader-features，指定范围的态数上限起读库程序也取它
  python3 "$replay_script_directory/lib_crash_point_replay_items.py" reader-features "$1" "$2"
}

# 指定范围的崩溃域上限（文件头「不带 --item 的跑法…」）借判法 lib_crash_point_replay_items.py 的 --defect 开关，每个弄坏开关一个值，没开打空
scoped_domain_defect() {
  if replay_break_is scoped-domain-reusable-counted; then printf '%s' count-reusable
  elif replay_break_is scoped-domain-unknown-passed; then printf '%s' unknown-as-zero
  elif replay_break_is scoped-domain-suggestion-dropped; then printf '%s' suggestion-dropped
  elif replay_break_is scoped-domain-shrunk-records-used; then printf '%s' shrunk-records-used
  fi
}

# 指定范围的崩溃域上限：带 --item 才调（没缩域才判；估法、上限 200000 态与退出码 79 都在 lib_crash_point_replay_items.py 的 scoped-domain guard）。
#   交回 0 放行，79 超上限或估不出（说明已打在标准错误）。起读库程序用真跑那一份环境（test_functions_environment）与这一趟的编译目录。
#   弄坏开关 scoped-domain-unbounded 下不判，scoped-domain-narrowed-refused 下缩了域也照没缩判（缩域值不交给判法）
scoped_domain_guard() { # <点名的项…>
  local defect narrowed_arguments=() defect_arguments=() item_arguments=() name
  replay_break_is scoped-domain-unbounded && return 0
  if [[ -n "$crash_expand_up_to_origin" ]] && ! replay_break_is scoped-domain-narrowed-refused; then narrowed_arguments=(--expand-up-to "$crash_expand_up_to"); fi
  defect="$(scoped_domain_defect)"
  if [[ -n "$defect" ]]; then defect_arguments=(--defect "$defect"); fi
  for name in "$@"; do item_arguments+=(--item "$name"); done
  test_functions_environment
  "${functions_environment[@]}" python3 "$replay_script_directory/lib_crash_point_replay_items.py" scoped-domain guard --root "$repository_root" --runs "$pipeline_home/runs" \
    --gpu "$gpu_switch" ${narrowed_arguments[@]+"${narrowed_arguments[@]}"} ${defect_arguments[@]+"${defect_arguments[@]}"} "${item_arguments[@]}"
}

# --list-items --with-keys 的 ITEM_DOMAIN（文件头「输出」）：<项…> 每项一行，估法同 scoped_domain_guard（scoped-domain report）。弄坏开关 item-domain-line-dropped 下不打
#   列出的项一起交一趟判法：判法起一次读库程序 items 子命令读全部项的块账（一次连库、一遍扫库），行照项的次序打；判法退非 0 照原样交回。
#   弄坏开关 item-domain-reader-per-item 下每项各交一趟判法、各起一次读库程序
report_the_item_domain() {
  local name item_arguments=() item_exit=0
  replay_break_is item-domain-line-dropped && return 0
  (( $# )) || return 0
  test_functions_environment
  if replay_break_is item-domain-reader-per-item; then
    for name in "$@"; do
      "${functions_environment[@]}" python3 "$replay_script_directory/lib_crash_point_replay_items.py" scoped-domain report --root "$repository_root" --runs "$pipeline_home/runs" \
        --gpu "$gpu_switch" --item "$name" || item_exit=$?
    done
    return "$item_exit"
  fi
  for name in "$@"; do item_arguments+=(--item "$name"); done
  "${functions_environment[@]}" python3 "$replay_script_directory/lib_crash_point_replay_items.py" scoped-domain report --root "$repository_root" --runs "$pipeline_home/runs" \
    --gpu "$gpu_switch" "${item_arguments[@]}"
}

# 崩溃点重放部分每跑完一趟，在日志旁写 <日志>.scope（文件头「输出」）：估指定范围的态数上限时读（lib_crash_point_replay_items.py 的 enumeration_totals）。弄坏开关 enumeration-scope-not-recorded 下不写
record_enumeration_scope() { # <日志> <crash-case: 键…>
  local log="$1" key case_directory scope_text states pairs=""
  shift
  replay_break_is enumeration-scope-not-recorded && return 0
  case_directory="$(sed -nE 's/^  · 这一趟每条用例的日志与发现日志放 (.*)\/（log\.<用例名> .*$/\1/p' "$log" 2>/dev/null | head -n 1)"
  scope_text="expand_up_to=${crash_expand_up_to:-full}"$'\n'
  ((! crash_judge_from_caller)) || scope_text+="judge_expand_up_to=$crash_judge_expand_up_to"$'\n'
  # 态数在跑的这一台上从门禁放的用例日志里取出来（那份日志在这一台的 git common-dir 下，别的机器读不到），读不出的那条用例不写，下一趟估上限时这一项算不出
  if [[ -n "$case_directory" ]]; then
    for key in "$@"; do
      states="$(python3 "$replay_script_directory/lib_crash_point_replay_items.py" case-states "$case_directory/log.${key#crash-case:}" 2>/dev/null)" && pairs+="${key#crash-case:}:$states "
    done
  fi
  scope_text+="case_states=${pairs% }"$'\n'
  printf '%s' "$scope_text" >"$log.scope" || echo "  ! 写不成 $log.scope：这一趟的展开深度与各条用例的态数估上限时读不到" >&2
}

# 按项机读行 ITEM_PLAN（文件头「输出」）：<list|shard> <计划文件> <分片计划文件或 -> <test-plan 的输出或 ->，stdout。
#   list 一项一行；shard 每片每项一行，一项的测试函数那一段记在它头一次出现的那一片。
#   弄坏开关 item-plan-reusable-zero 下 reusable 一律 0，shard-plan-lines-merged 下分片计划行不分片、各片相加，test-functions-directory-root-unseen 下目录式目标（目标根文件 tests/<目标>/main.rs）不认
emit_item_plan_lines() {
  python3 - "$1" "$2" "$3" "$4" "$(replay_break_is item-plan-reusable-zero && echo 1 || echo 0)" "$(replay_break_is shard-plan-lines-merged && echo 1 || echo 0)" "$(replay_break_is test-functions-directory-root-unseen && echo 1 || echo 0)" <<'PYTHON'
import collections, re, sys
mode, plan_file, shard_file, plan_output, reusable_zero, lines_merged, directory_root_unseen = sys.argv[1:8]
counts = collections.defaultdict(lambda: [0, 0])   # 测试目标 -> [要现判的（键对不上与拒算）, 复用的]
if plan_output != "-":
    for line in open(plan_output, encoding="utf-8", errors="replace"):
        columns = line.rstrip("\n").split("\t")
        if len(columns) >= 7 and (root := re.fullmatch(r"tests/([^/]+)(?:\.rs|/main\.rs)", columns[4])) and not (directory_root_unseen == "1" and root.group(0).endswith("/main.rs")):
            target = root.group(1)
            if columns[1] in ("judge", "refused"):
                counts[target][0] += 1
            elif columns[1] == "reuse":
                counts[target][1] += 1

def units(kind, obj):
    if kind == "test-functions":
        return tuple(counts[obj])
    if kind in ("enumeration", "pipeline"):
        return (1, 0)
    return (0, 0)

def show(label, to_judge, reusable):
    return f"ITEM_PLAN {label}to_judge={to_judge} reusable={0 if reusable_zero == '1' else reusable}"

if mode == "list":
    totals = collections.OrderedDict()
    for line in open(plan_file, encoding="utf-8"):
        columns = line.rstrip("\n").split("\t")
        if len(columns) < 3:
            continue
        item, kind, obj = columns[:3]
        total = totals.setdefault(item, [0, 0])
        judge, reuse = units(kind, obj)
        total[0] += judge
        total[1] += reuse
    for item, (judge, reuse) in totals.items():
        print(show(f"item={item} ", judge, reuse))
else:
    per_shard = collections.OrderedDict()
    first_shard = {}
    for line in open(shard_file, encoding="utf-8"):
        columns = line.rstrip("\n").split("\t")
        if len(columns) < 4:
            continue
        shard, item, kind, obj = int(columns[0]), columns[1], columns[2], columns[3]
        first_shard.setdefault(item, shard)
        entry = per_shard.setdefault(shard, collections.OrderedDict()).setdefault(item, [0, 0])
        judge, reuse = units(kind, obj)
        if kind == "test-functions" and first_shard[item] != shard:
            judge = reuse = 0
        entry[0] += judge
        entry[1] += reuse
    if lines_merged == "1":
        merged = collections.OrderedDict()
        for items in per_shard.values():
            for item, (judge, reuse) in items.items():
                total = merged.setdefault(item, [0, 0])
                total[0] += judge
                total[1] += reuse
        for item, (judge, reuse) in merged.items():
            print(show(f"item={item} ", judge, reuse))
    else:
        for shard, items in per_shard.items():
            for item, (judge, reuse) in items.items():
                print(show(f"shard={shard} item={item} ", judge, reuse))
PYTHON
}

usage() {
  echo "  ✗ 用法不对：$1。用法：$0 [--item <项>]… [--crash-expand-up-to <n>] | --list-items [--with-keys [--item <项>]…] | --selftest" >&2
  echo "  → 怎么办：不带参数跑全部项；缩小范围用 --item <项>，项名照 --list-items 列的写（含整趟才跑的那几类 whole-run:<类>）；经 task-server/task submit 交，服务照件分片" >&2
  exit 2
}

# 平常跑时带了只供自证的变量就拒，排在一切参数解析之前（认不得的选项、--list-items 也先看这个）；自证起的子进程与命令行第一个参数就是 --selftest 的除外（文件头「只供自证」）。
# 关守卫项 honor-test-variables-outside-selftest 整值比：只在它是 CRASH_POINT_REPLAY_BREAK 唯一一项时认；与别的项同写，平常跑照拒并点名 CRASH_POINT_REPLAY_BREAK。
if (( ! replay_in_selftest_child )) && [[ "${1:-}" != --selftest ]] && (( ${#replay_test_variables[@]} )) && ! replay_guard_off_honored; then
  echo "  ✗ 平常跑时带了只供自证的变量：${replay_test_variables[*]}；这几个变量只在自证里认（文件头「只供自证」）" >&2
  echo "  → 怎么办：去掉它们再跑（env -u <变量> …）；要证红就跑 $0 --selftest" >&2
  exit 2
fi

list_items=0; with_keys=0; selftest=0; named_items=()
crash_expand_given=0; crash_expand_argument=""; crash_expand_up_to=""; crash_expand_up_to_origin=""; crash_judge_expand_up_to=""; narrowed_labels=""
while (($#)); do
  case "$1" in
    --list-items) list_items=1; shift ;;
    --with-keys) with_keys=1; shift ;;
    --selftest) selftest=1; shift ;;
    --item)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then usage "--item 后面缺项名"; fi
      named_items+=("$2"); shift 2 ;;
    --item=*) [[ -n "${1#--item=}" ]] || usage "--item= 后面缺项名"; named_items+=("${1#--item=}"); shift ;;
    --crash-expand-up-to)
      (($# >= 2)) || usage "--crash-expand-up-to 后面要跟一个非负整数（展开深度上限，只往小缩）"
      crash_expand_given=1; crash_expand_argument="$2"; shift 2 ;;
    --crash-expand-up-to=*) crash_expand_given=1; crash_expand_argument="${1#--crash-expand-up-to=}"; shift ;;
    *) usage "认不得的参数 $1" ;;
  esac
done

# 缩崩溃域（文件头「往引擎送的变量」）：--crash-expand-up-to 与环境里已设的 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 是同一个值（两样都给取参数）；
#   判器的展开深度 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO（文件头「往引擎送的变量」）：给了 --crash-expand-up-to 设成同一个值；没给参数时环境里已设的照用（crash_judge_from_caller=1，标签与 .scope 里另写
#   judge_expand_up_to=），没设而放量那个有值的跟着设成它的值；环境里只设了判器那个的只认它。值不是非负整数退 2、什么都不做。
#   弄坏开关 expand-up-to-unchecked 下不核，expand-up-to-not-passed 下不交放量那个变量，judge-expand-variable-not-passed 下判器那个不跟着设
crash_judge_from_caller=0
if ((crash_expand_given)); then
  crash_expand_up_to="$crash_expand_argument"; crash_expand_up_to_origin="--crash-expand-up-to"
elif [[ -n "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO+x}" ]]; then
  crash_expand_up_to="$SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO"; crash_expand_up_to_origin="环境变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO"
fi
[[ -z "${SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO+x}" ]] || { crash_judge_expand_up_to="$SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO"; crash_judge_from_caller=1; }
if ! replay_break_is expand-up-to-unchecked; then
  if [[ -n "$crash_expand_up_to_origin" && ! "$crash_expand_up_to" =~ ^[0-9]+$ ]]; then
    usage "$crash_expand_up_to_origin 的值「$crash_expand_up_to」不是非负整数（展开深度上限，只往小缩）"
  fi
  if [[ -n "${SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO+x}" && ! "$crash_judge_expand_up_to" =~ ^[0-9]+$ ]]; then
    usage "环境变量 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO 的值「$crash_judge_expand_up_to」不是非负整数（判器的展开深度上限）"
  fi
fi
if [[ -n "$crash_expand_up_to_origin" ]]; then
  if replay_break_is expand-up-to-not-passed; then unset SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO
  else export SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO="$crash_expand_up_to"; fi
  narrowed_labels="expand_up_to=$crash_expand_up_to"
  # 判器那个：给了参数盖过环境里的；没给参数时环境里已设的照用、没设的跟着放量那个
  if ! replay_break_is judge-expand-variable-not-passed && { ((crash_expand_given)) || ((! crash_judge_from_caller)); }; then
    crash_judge_expand_up_to="$crash_expand_up_to"; crash_judge_from_caller=0
    export SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO="$crash_judge_expand_up_to"
  fi
fi
((! crash_judge_from_caller)) || narrowed_labels+="${narrowed_labels:+ }judge_expand_up_to=$crash_judge_expand_up_to"

# 自证起的子进程才认 CRASH_POINT_REPLAY_ROOT（平常跑时带了只供自证的变量，上面参数解析之前已经拒了；走到这里的平常跑只有开着关守卫项的那一种）
if (( ! selftest )); then
  if (( replay_in_selftest_child )); then
    [[ -n "$replay_root_override" ]] && repository_root="$replay_root_override"
  elif (( ! list_items )) && [[ -n "$replay_root_override" ]]; then
    repository_root="$replay_root_override"
  fi
fi

# ── 合表（lib_crash_point_replay_items.py list：核不过不打表、退 1）
declare -A item_cases=() item_pipeline_items=() item_test_targets=() item_not_run_reason=() pipeline_owner=()
item_names=(); whole_run_names=()
load_items() {
  local listing listing_exit name cases pipeline_text test_targets word
  listing="$(python3 "$replay_script_directory/lib_crash_point_replay_items.py" list "$repository_root")"; listing_exit=$?
  if (( listing_exit != 0 )); then
    printf '%s\n' "$listing" >&2
    echo "  ✗ 崩溃点重放合表核不过（lib_crash_point_replay_items.py list 退 $listing_exit），什么都不跑" >&2
    echo "  → 怎么办：照上面逐处列的改 .claude/gate.d/crash-point-replay-items.tsv（写法在它的表头），再跑" >&2
    exit 2
  fi
  while IFS=$'\t' read -r name cases pipeline_text test_targets; do
    [[ -n "$name" ]] || continue
    item_names+=("$name")
    [[ "$cases" == - ]] && cases=""
    [[ "$pipeline_text" == - ]] && pipeline_text=""
    item_cases[$name]="$cases"; item_pipeline_items[$name]="$pipeline_text"; item_test_targets[$name]="$test_targets"
    for word in $pipeline_text; do pipeline_owner[$word]="$name"; done
  done <<<"$listing"
}

# 整趟才跑的那几类（WHOLE_RUN_GROUPS）各是一项 whole-run:<类名>：--item 点得到、--list-items 列出；没有测试目标的一类（钉回来的红例）点到也记本次未跑、写原因
load_whole_run_items() {
  local group_line group_name group_targets name
  for group_line in "${WHOLE_RUN_GROUPS[@]}"; do
    IFS=$'\t' read -r group_name group_targets <<<"$group_line"
    name="whole-run:$group_name"
    whole_run_names+=("$name")
    item_cases[$name]=""; item_pipeline_items[$name]=""
    if [[ "$group_targets" == - ]]; then
      item_test_targets[$name]=""; item_not_run_reason[$name]="$PINNED_RED_EXAMPLES_NOT_RUN_REASON"
    else
      item_test_targets[$name]="$group_targets"
    fi
  done
}

# 测试函数一趟的 libtest 日志怎么判（文件头「一项怎么判」）：设 functions_state、functions_reason、functions_exit（记进记录的退出码）、functions_tests、functions_red
judge_test_functions_log() { # <退出码> <日志>
  local run_exit="$1" log="$2" counts first_line passed failed not_run_lines condition_lines condition_line
  counts="$(python3 - "$log" "$replay_script_directory/../gate/lib_libtest_result_lines.py" "$CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX" "$(replay_break_is test-line-markers-unread && echo line-start-only)" <<'PYTHON'
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
# 测试自己打的那一截：几条同时跑时它可能接在别的测试的「test <名> ... 」后面（lib_libtest_result_lines.py 文件头「按条记库」）。弄坏开关 test-line-markers-unread 下只认行首
def printed_by_the_test(line):
    stripped = line.strip()
    started = None if sys.argv[4] or stripped.startswith("test result: ") else module.SCATTERED_START_FORM.match(stripped)
    return started.group("rest").strip() if started else stripped
printed = [printed_by_the_test(line) for line in text.splitlines()]
not_run = sum(1 for line in printed if line.startswith("NOT_RUN_NO_GPU_ADAPTER"))
prefix = sys.argv[3]
condition = [line for line in printed if line == prefix or line.startswith(prefix + " ")]
# 复用计数行（文件头「测试函数的复用数怎么取」）：CRASH_POINT_REPLAY_PARALLEL_FINISHED 的 resumed_slices= 之和；一行都没有、有一行缺或写坏，取不出
finished = [line for line in printed if line.startswith("CRASH_POINT_REPLAY_PARALLEL_FINISHED ")]
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
  read -r passed failed not_run_lines condition_lines functions_reused <<<"${first_line:-0 0 0 0 unknown}"
  [[ "$functions_reused" =~ ^[0-9]+$ ]] || functions_reused=unknown
  replay_break_is test-functions-reuse-not-counted && functions_reused=none   # 弄坏开关：写死 none
  [[ "$passed" =~ ^[0-9]+$ ]] || passed=0
  [[ "$failed" =~ ^[0-9]+$ ]] || failed=0
  [[ "$not_run_lines" =~ ^[0-9]+$ ]] || not_run_lines=0
  [[ "$condition_lines" =~ ^[0-9]+$ ]] || condition_lines=0
  functions_state=green; functions_reason=""; functions_exit="$run_exit"; functions_red=0
  if (( condition_lines > 0 )) && ! replay_break_is given-states-run-condition-as-red; then
    functions_state=not-run; functions_exit=78
    functions_reason="运行条件不满足：给定状态表的判器借不到显存（$condition_line；共 $condition_lines 处，日志 $log、错误警告 ${log%.log}.problems.log），这一趟不算数；等卡上占着显存的跑完再重跑"
  elif [[ "$run_exit" != 0 || "$failed" -gt 0 ]] && ! replay_break_is red-as-green; then
    functions_state=red; functions_reason="cargo test 退 $run_exit、红 $failed 条（日志 $log、错误警告 ${log%.log}.problems.log）"
    (( failed > 0 )) || failed=1
    functions_red=$failed
  elif (( passed + failed == 0 )) && ! replay_break_is not-run-as-green; then
    functions_state=not-run; functions_reason="测试函数这一段跑到 0 条（这个测试目标里没有不登记的测试函数，或挂的特性这一趟没编进来）"
  elif (( not_run_lines > 0 )) && ! replay_break_is not-run-as-green; then
    functions_state=not-run; functions_reason="有 $not_run_lines 处 NOT_RUN_NO_GPU_ADAPTER（这台机器找不到 GPU 适配器）"
  fi
  functions_tests=$((passed + failed))
}

# 计划（纯函数）：打这一趟要做的每一件「项<TAB>种类<TAB>对象」。种类：test-functions（按条复用的测试目标）、
# pipeline（放量分项）、enumeration（crash-case: 键交 门禁 crash-point-replay）、not-run（整趟才跑的一类没有测试目标，对象是原因）。
# 整趟才跑的那几类是项 whole-run:<类名>，同别的项一样点名或不点名；两种跑法（全部项、点名）只差项取哪些。
replay_plan() { # <项…>
  local item word
  for item in "$@"; do
    if [[ -n "${item_not_run_reason[$item]+set}" ]]; then
      printf '%s\tnot-run\t%s\n' "$item" "${item_not_run_reason[$item]}"
      continue
    fi
    for word in ${item_test_targets[$item]}; do printf '%s\ttest-functions\t%s\n' "$item" "$word"; done
    for word in ${item_cases[$item]}; do printf '%s\tenumeration\t%s\n' "$item" "$word"; done
    for word in ${item_pipeline_items[$item]}; do printf '%s\tpipeline\t%s\n' "$item" "$word"; done
  done
}

# ── 背后的几段。每段往记录文件 $part_records 追加一行：
#   项<TAB>段（test-functions|pipeline|enumeration|not-run）<TAB>对象<TAB>退出码<TAB>结论（green|red|not-run）<TAB>判了多少<TAB>红了多少<TAB>复用（数、unknown 或 none）<TAB>秒<TAB>原因<TAB>件数（现判:复用）
record_part() { # 十个字段；第十一个是这一段的件数「现判:复用」（没给写 0:0，文件头「输出」ITEM_DONE）
  local units="${11:-0:0}"
  printf '%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\t%s\n' "${@:1:10}" "$units" >>"$part_records"
}

# 跑一条命令：输出边跑边转打，同时出两份日志（scripts/hosts/two-logs.sh：<日志> 是全量，同名的 .problems.log 只收错误与警告、行前带全量日志里的行号，一般只看它）；
# 退出码写进 <日志>.exit。弄坏开关 logs-single：只写一份全量日志（tee）
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

# ── 按条复用（文件头「按条复用」）：种类、请求、跳过与记库的名单只在 scripts/gate/admission.py 的 test-functions-request、test-plan、plan-skip-names、
#    plan-green-names、verdict-record、plan-failure，与门禁 crash-point-replay 测试函数几格同一类判定库；这里只接它们的输入输出
functions_plan_state=""; functions_plan_note=""; functions_plan_kind=""; functions_plan_file=""; functions_plan_output=""   # state：planned 算成了、off 不读库全部现判、nolibrary 判定库没有名字、failed 按条复用失灵、unavailable 服务不可用
functions_plan_library=""                                         # 判定库的名字：被判的根的 git common-dir（库在 prune-server 里）
functions_recorded=0                                                                   # 这一趟判绿写回的条数（verdict-record 的 RECORD recorded= 之和）
functions_red_recorded=0                                                               # 这一趟判红记下、撤掉同一把键原有的绿的条数（RECORD red= 之和）
functions_target_judge=0; functions_target_reuse=0; functions_target_refused=0; functions_target_registered=0; functions_target_skip_arguments=(); functions_target_record_red=""

# plan_test_functions：算这一趟测试函数的计划（test-plan 落进 functions_plan_output，计划文件 functions_plan_file），设 functions_plan_state 与 functions_plan_note。
#   被判的根不是 git 工作树的顶（判定库没有名字）：nolibrary，run_test_functions 照它整趟退 2（弄坏开关 test-functions-without-library-tolerated 下 off：全部现判、不入库）；
#   test-plan 退非 0 或没打 PLAN：交 admission.py plan-failure 判，失灵是 failed（弄坏开关
#   test-functions-plan-failure-not-red 下照 off 办：全部现判、不判红）。弄坏开关 test-functions-kind-not-gate 下种类换成另一类，red-reused-as-green 下读库带 ADMISSION_BREAK=red-verdict-reused
#   参数是交给测试函数 cargo 的那一份环境（env -u … 变量=值 … 那一串）：test-functions-request 在它里面起（文件头「按条复用」：改判定域的环境变量照它进键）。
#   判定库的名字取被判的根的 git common-dir。弄坏开关 test-functions-domain-environment-unkeyed 下 test-functions-request 带 ADMISSION_BREAK=domain-environment-unkeyed
plan_test_functions() { # <交给测试函数 cargo 的那一份环境…>
  local request_output request_tag request_value plan_exit toplevel physical_root package="" feature
  local -a request_arguments=() feature_options=() exclude_option=(--exclude-registered) admission_break=() request_environment=("$@") request_break=()
  functions_plan_state=off; functions_plan_note=""; functions_plan_kind=""; functions_plan_library=""
  functions_plan_file="$run_directory/test-functions.plan"; functions_plan_output="$run_directory/test-functions.plan.out"
  toplevel="$(git -C "$repository_root" rev-parse --show-toplevel 2>/dev/null)"
  physical_root="$(cd "$repository_root" && pwd -P)"
  functions_plan_library="$(git -C "$repository_root" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)"
  if [[ -z "$toplevel" || "$toplevel" != "$physical_root" || -z "$functions_plan_library" ]]; then
    functions_plan_state=nolibrary
    functions_plan_note="被判的根 $repository_root 不是一棵 git 工作树的顶，判定库没有名字（scripts/gate/admission.py 文件头「判定库」）"
    if replay_break_is test-functions-without-library-tolerated; then
      functions_plan_state=off
      functions_plan_note+="：测试函数全部现判、不入库"
    fi
    return 0
  fi
  replay_break_is test-functions-domain-environment-unkeyed && request_break=("ADMISSION_BREAK=domain-environment-unkeyed")
  if ! request_output="$(env ${request_break[@]+"${request_break[@]}"} ${request_environment[@]+"${request_environment[@]}"} \
         python3 "$replay_admission_module" test-functions-request "$TEST_FUNCTIONS_GATE_FILE_NAME" "$replay_judge_root" 2>&1)"; then
    functions_plan_state=failed
    functions_plan_note="按条复用失灵：scripts/gate/admission.py test-functions-request 没成（$(tr '\n' ' ' <<<"$request_output" | clip_utf8 300)）"
    replay_break_is test-functions-plan-failure-not-red && functions_plan_state=off
    return 0
  fi
  while IFS=$'\t' read -r request_tag request_value; do
    case "$request_tag" in
      kind) functions_plan_kind="$request_value" ;;
      package) package="$request_value" ;;
      argument) request_arguments+=("$request_value") ;;
    esac
  done <<<"$request_output"
  # 弄坏开关：种类与门禁不同（记进、读另一份库）。在门禁那一类前面加一段，不写死另一个名字：入口与门禁同名（crash-point-replay.sh）之后，写死的名字就是门禁那一类，开关什么都不弄坏
  replay_break_is test-functions-kind-not-gate && functions_plan_kind="entry-only-$functions_plan_kind"
  for feature in ${features//,/ }; do feature_options+=(--feature "$package/$feature"); done
  replay_break_is registered-not-excluded && exclude_option=()   # 弄坏开关：不排登记的用例与放量分项，它们也按条复用地跑
  replay_break_is red-reused-as-green && admission_break=("ADMISSION_BREAK=red-verdict-reused")
  if env ${admission_break[@]+"${admission_break[@]}"} python3 "$replay_admission_module" test-plan "$repository_root" "$functions_plan_kind" "$package" "$functions_plan_file" \
       ${request_arguments[@]+"${request_arguments[@]}"} ${feature_options[@]+"${feature_options[@]}"} ${exclude_option[@]+"${exclude_option[@]}"} >"$functions_plan_output" 2>"$functions_plan_output.err"; then
    plan_exit=0
  else
    plan_exit=$?
  fi
  if (( plan_exit == 0 )) && grep -q '^PLAN ' "$functions_plan_output"; then
    functions_plan_state=planned
    return 0
  fi
  local failure_exit=0
  python3 "$replay_admission_module" plan-failure "crash-point-replay.sh 测试函数那一段" test-plan "$plan_exit" "$functions_plan_output" "$functions_plan_output.err" >"$functions_plan_output.failure" 2>&1 \
    || failure_exit=$?
  # prune-server 不可用（plan-failure 退 75）不是失灵：run_test_functions 照它整趟退 75（文件头「按条复用」）；弄坏开关 test-functions-plan-unavailable-as-failure 下照失灵办
  if (( failure_exit == 75 )) && ! replay_break_is test-functions-plan-unavailable-as-failure; then
    cat -- "$functions_plan_output.failure"
    functions_plan_state=unavailable
    functions_plan_note="prune-server 不可用（test-plan 退 75；日志 $functions_plan_output.failure）"
    return 0
  fi
  (( failure_exit == 75 )) && failure_exit=1
  if (( failure_exit == 0 )) || replay_break_is test-functions-plan-failure-not-red; then
    functions_plan_note="test-plan 退 $plan_exit（$(tail -n 2 "$functions_plan_output" "$functions_plan_output.err" 2>/dev/null | tr '\n' ' ' | clip_utf8 300)）：测试函数全部现判、不入库（本次未跑复用）"
    return 0
  fi
  cat -- "$functions_plan_output.failure"
  functions_plan_state=failed
  functions_plan_note="$(grep -m 1 '✗' "$functions_plan_output.failure" | sed 's/^ *✗ *//' | clip_utf8 400)（日志 $functions_plan_output.failure）"
}

# test_functions_target_roots <测试目标>：test-plan 第五列（目标根文件，包目录起）里这个测试目标的写法，空格隔开：tests/<目标>.rs、目录式的 tests/<目标>/main.rs（test_functions_target_counts、test_functions_target_skips、record_test_functions_target 与 emit_item_plan_lines 都按它认；弄坏开关 test-functions-directory-root-unseen 下没有后一个）
test_functions_target_roots() { echo "tests/$1.rs $(replay_break_is test-functions-directory-root-unseen || echo "tests/$1/main.rs")"; }
# test_functions_target_counts <测试目标>：这个测试目标在计划里要现判、复用、拒算、排给别的两段的条数放进 functions_target_judge、functions_target_reuse、functions_target_refused、functions_target_registered（计划没算成都是 0）
test_functions_target_counts() {
  local counts
  functions_target_judge=0; functions_target_reuse=0; functions_target_refused=0; functions_target_registered=0
  [[ "$functions_plan_state" == planned ]] || return 0
  counts="$(awk -F'\t' -v roots="$(test_functions_target_roots "$1")" 'NF >= 7 && index(" " roots " ", " " $5 " ") { count[$2]++ } END { printf "%d %d %d %d\n", count["judge"], count["reuse"], count["refused"], count["registered"] }' "$functions_plan_output")"
  read -r functions_target_judge functions_target_reuse functions_target_refused functions_target_registered <<<"$counts"
}

# test_functions_target_all_reused <测试目标>：这个测试目标编得进的测试函数全有键相同、不红的判定（一条都不用现判，不编不起 cargo）。弄坏开关 test-functions-skip-ignored 下一律不算
test_functions_target_all_reused() {
  test_functions_target_counts "$1"
  ! replay_break_is test-functions-skip-ignored && (( functions_target_reuse > 0 && functions_target_judge + functions_target_refused == 0 ))
}

# test_functions_target_skips <测试目标>：这个测试目标要 --skip 的放进 functions_target_skip_arguments（--exact 打头；名单取 admission.py plan-skip-names）。
#   取不出名单就一条不跳（全部现判，宁多跑）、打一行。弄坏开关 test-functions-skip-ignored 下一条不跳
test_functions_target_skips() {
  local names name; local -a roots=(); functions_target_skip_arguments=()
  [[ "$functions_plan_state" == planned ]] || return 0
  replay_break_is test-functions-skip-ignored && return 0
  read -ra roots <<<"$(test_functions_target_roots "$1")"
  if ! names="$(python3 "$replay_admission_module" plan-skip-names "$functions_plan_output" --target-root "${roots[0]}" ${roots[1]:+--target-root "${roots[1]}"} 2>&1)"; then
    echo "  ! 按条复用：admission.py plan-skip-names 没成（$(tr '\n' ' ' <<<"$names" | clip_utf8 300)），$1 一条都不跳、全部现判"
    return 0
  fi
  while IFS= read -r name; do [[ -n "$name" ]] && functions_target_skip_arguments+=(--skip "$name"); done <<<"$names"
  (( ${#functions_target_skip_arguments[@]} )) && functions_target_skip_arguments=(--exact "${functions_target_skip_arguments[@]}")
  return 0
}

# record_test_functions_target <测试目标> <日志或 ->：逐条打这个测试目标里计划到的测试函数的结局
#   （「· <测试函数>：绿|红，现判|复用|强制现判（…）」），现判而判绿的经 plan-green-names 与 verdict-record 照键写回（日志写 - 是整目标复用、没起 cargo）。
#   记库那一趟 verdict-record 退非 0 而 plan-failure 判失灵的，原因放进 functions_target_record_red。计划没算成什么都不做。
#   弄坏开关 test-functions-not-recorded 下不写回，red-reused-as-green 下现判红的也照判绿写回
record_test_functions_target() {
  local target="$1" log="$2" prefix record_exit recorded red_recorded
  local -a cpu_option=() red_option=()
  functions_target_record_red=""
  [[ "$functions_plan_state" == planned ]] || return 0
  test_functions_target_counts "$target"
  (( functions_target_judge + functions_target_reuse + functions_target_refused > 0 )) || return 0   # 计划里没有这个测试目标的测试函数（test-plan 没列到它）：照跑过的算，什么都不记
  prefix="$run_directory/test-functions.$target"
  : >"$prefix.accounting"; : >"$prefix.green-names"; : >"$prefix.green-names.withheld"
  if [[ "$log" != - ]]; then
    (( gpu_switch )) || cpu_option=(--cpu-judging-recordable)   # 没开 GPU 时判段判在 CPU 的绿照记（同门禁，D13（验证路线）已定项 17）
    if python3 "$replay_script_directory/../gate/lib_libtest_result_lines.py" recordable "$log" ${cpu_option[@]+"${cpu_option[@]}"} \
         >"$prefix.accounting" 2>"$prefix.accounting.err" && grep -q '^RECORDABLE ' "$prefix.accounting"; then
      if ! python3 "$replay_admission_module" plan-green-names "$functions_plan_output" "$prefix.accounting" "$prefix.green-names.withheld" >"$prefix.green-names" 2>&1; then
        echo "  ! 按条复用：admission.py plan-green-names 没成（$(tr '\n' ' ' <"$prefix.green-names" | clip_utf8 300)），$target 一条都不记"
        : >"$prefix.green-names"
      fi
    else
      echo "  ! 按条复用：lib_libtest_result_lines.py recordable 没成（$(tail -n 2 "$prefix.accounting" "$prefix.accounting.err" 2>/dev/null | tr '\n' ' ' | clip_utf8 300)），$target 一条都不记"
      : >"$prefix.accounting"
    fi
  fi
  # 逐条打结局（结局取这一趟 libtest 的结果行，判法 lib_libtest_result_lines.py 的 result_lines）；现判红的另落一份名单（弄坏开关 red-reused-as-green 用）；计数追加进 test-functions.tsv
  python3 - "$functions_plan_output" "$(test_functions_target_roots "$target")" "$log" "$prefix.green-names" "$prefix.green-names.withheld" "$prefix.red-names" "$run_directory/test-functions.tsv" "$replay_script_directory/../gate/lib_libtest_result_lines.py" "$target" <<'PYTHON'
import importlib.util, sys
plan_output, target_roots, log, green_file, withheld_file, red_file, counts_file, library_path, target = sys.argv[1:10]
specification = importlib.util.spec_from_file_location("lib_libtest_result_lines", library_path)
module = importlib.util.module_from_spec(specification)
specification.loader.exec_module(module)
def lines_of(path):
    try:
        return [line.rstrip("\n") for line in open(path, encoding="utf-8", errors="surrogateescape")]
    except OSError:
        return []
rows = [line.split("\t") for line in lines_of(plan_output)]
rows = [columns for columns in rows if len(columns) >= 7 and columns[4] in target_roots.split() and columns[1] in ("judge", "reuse", "refused")]
green = set(name for name in lines_of(green_file) if name)
withheld = dict(line.split("\t", 1) for line in lines_of(withheld_file) if "\t" in line)
outcomes = {}
if log != "-":
    try:
        text = open(log, encoding="utf-8", errors="replace").read()
    except OSError:
        text = ""
    for result in module.result_lines(text):
        outcomes[result.full_name] = result.outcome
judged = reused = refused = 0
red = []
for name, decision, _key, _target_id, _root, libtest_name, reason in (columns[:7] for columns in rows):
    if decision == "reuse" and libtest_name not in outcomes:
        reused += 1
        print(f"  · {name}：绿，复用（{reason}）")
        continue
    state = "强制现判" if reason.startswith("强制现判") else "现判"
    if decision == "refused":
        refused += 1
    else:
        judged += 1
    outcome = outcomes.get(libtest_name)
    if outcome == "ok":
        if decision == "reuse":
            note = "键相同、库里判绿，这一趟没跳过"
        elif name in green:
            note = "判绿写回"
        else:
            note = "不入库：" + (withheld[name] if name in withheld else ("拒算" if decision == "refused" else "按条记库没认这一条"))
        print(f"  · {name}：绿，{state}（{note}）")
    elif outcome == "FAILED":
        red.append(name)
        print(f"  · {name}：红，{state}（记红、撤掉同一把键的绿，下一趟照现判）")
    else:
        print(f"  · {name}：本次未跑（{state}，这一趟日志里没有它的结果行{'' if outcome is None else '：' + outcome}）")
with open(red_file, "w", encoding="utf-8", errors="surrogateescape") as handle:
    handle.write("".join(name + "\n" for name in red))
with open(counts_file, "a", encoding="utf-8") as handle:
    handle.write(f"{target}\t{judged}\t{reused}\t{refused}\t{'reused' if log == '-' else 'ran'}\n")
PYTHON
  if replay_break_is red-reused-as-green; then cat -- "$prefix.red-names" >>"$prefix.green-names"; fi   # 弄坏开关：红的也照判绿写回
  # 判红的记红、撤掉同一把键原有的绿（文件头「按条复用」）：现判红的名单交 --red-names。弄坏开关 red-not-recorded 下不交，red-reused-as-green 下已并进判绿的名单
  red_option=()
  if [[ -s "$prefix.red-names" ]] && ! replay_break_is red-reused-as-green && ! replay_break_is red-not-recorded; then red_option=(--red-names "$prefix.red-names"); fi
  [[ -s "$prefix.green-names" ]] || (( ${#red_option[@]} )) || return 0
  replay_break_is test-functions-not-recorded && return 0   # 弄坏开关：不写回
  if python3 "$replay_admission_module" verdict-record "$repository_root" "$functions_plan_kind" "$functions_plan_file" "$prefix.green-names" ${red_option[@]+"${red_option[@]}"} \
       --how "crash-point-replay.sh 测试函数那一段（$scope）：cargo test --release -p singlefs-verification ${features:+--features $features }--test $target" >"$prefix.record" 2>&1; then
    record_exit=0
  else
    record_exit=$?
  fi
  recorded="$(sed -n 's/^RECORD recorded=\([0-9]*\) .*/\1/p' "$prefix.record" | tail -n 1)"
  functions_recorded=$((functions_recorded + ${recorded:-0}))
  red_recorded="$(sed -n 's/^RECORD .* red=\([0-9]*\) .*/\1/p' "$prefix.record" | tail -n 1)"
  functions_red_recorded=$((functions_red_recorded + ${red_recorded:-0}))
  (( record_exit == 0 )) && return 0
  echo "  ! 按条复用：$target 判绿写回 admission.py verdict-record 退 $record_exit（$(tail -n 2 "$prefix.record" | tr '\n' ' ' | clip_utf8 300)），下一趟这几条照判"
  local record_failure_exit=0
  local -a red_withdrawal=()
  # 记红没成（admission.py 文件头「按条复用失灵」）：交了红的带 --red，它判失灵、撤掉那几把键原有的绿
  (( ${#red_option[@]} )) && red_withdrawal=(--red "$repository_root" "$functions_plan_kind" "$functions_plan_file" "$prefix.red-names")
  python3 "$replay_admission_module" plan-failure "crash-point-replay.sh 测试函数那一段（$target）" verdict-record "$record_exit" ${red_withdrawal[@]+"${red_withdrawal[@]}"} \
    "$prefix.record" >"$prefix.record.failure" 2>&1 || record_failure_exit=$?
  if (( record_failure_exit == 75 )) && ! replay_break_is test-functions-plan-unavailable-as-failure; then
    # 记库时 prune-server 不可用（文件头「按条复用」）：这一趟判出的判定记不下，整趟退 75
    cat -- "$prefix.record.failure"
    echo "  ✗ 测试函数那一段：$target 判完记库时 prune-server 不可用，这一趟的判定没记下；这一趟退 75"
    echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好之后重交同一条命令"
    exit 75
  fi
  if (( record_failure_exit != 0 )) && ! replay_break_is test-functions-plan-failure-not-red; then
    cat -- "$prefix.record.failure"
    functions_target_record_red="$(grep -m 1 '✗' "$prefix.record.failure" | sed 's/^ *✗ *//' | clip_utf8 400)（日志 $prefix.record.failure）"
  fi
  return 0
}

run_test_functions() { # <计划文件>：测试函数那一段（test-functions 的件），一个测试目标一趟；同一个测试目标几项都点名时跑一次、记给每一项
  local plan_file="$1" targets=() active_targets=() target target_arguments=() environment=() build_log build_exit run_log run_exit
  local variable started owners owner test_threads_arguments=() target_libtest_arguments=() target_units
  local functions_state functions_reason functions_exit functions_tests functions_red functions_reused
  while IFS=$'\t' read -r _plan_item plan_kind plan_object; do
    [[ "$plan_kind" == test-functions ]] || continue
    [[ " ${targets[*]-} " == *" $plan_object "* ]] || targets+=("$plan_object")
  done <"$plan_file"
  (( ${#targets[@]} )) || return 0
  # 交给测试函数 cargo 的那一份环境（test_functions_environment）：这几趟 cargo 直接起、不套线程包装（文件头 GPU 一段之后）
  test_functions_environment
  environment=("${functions_environment[@]}")
  # 分片的段一趟判几个态（SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES）：同另两段，从调用方环境继承，这里不设
  if (( gpu_switch )); then
    # 卡归 GPU 批处理验证服务（文件头「GPU」）：环境里有 GPU_CARDS_BY_PRIORITY 就原样交给每一趟，没有不设、照起。弄坏开关 gpu-cards-not-sent 下有也不交
    if [[ -n "$gpu_cards_for_test_functions" ]] && ! replay_break_is gpu-cards-not-sent; then
      environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_for_test_functions")
    fi
    # 测试线程不在这里控制（文件头「GPU」）；这里不领卡、不另套清场包装。弄坏开关 gpu-test-threads-one 下带 --test-threads 1
    if replay_break_is gpu-test-threads-one; then test_threads_arguments=(--test-threads 1); fi
  fi
  local card_note=""
  if (( gpu_switch )) && [[ -n "$gpu_cards_for_test_functions" ]]; then
    card_note="；环境里有 GPU_CARDS_BY_PRIORITY=$gpu_cards_for_test_functions，原样交给 cargo（这一段不领卡、不挑卡、不另套清场包装）"
  elif (( gpu_switch )); then
    card_note="；环境里没有 GPU_CARDS_BY_PRIORITY，照起（卡归 GPU 批处理验证服务，这一段不领卡）"
  fi
  echo "── 测试函数那一段：${#targets[@]} 个测试目标，cargo test --release -p singlefs-verification${features:+ --features $features}（登记成崩溃枚举用例与放量分项的测试函数经 --skip 排掉，交下面两段$card_note）"
  # 按条复用（文件头「按条复用」）：先算计划；编得进的测试函数全有键相同、不红的判定的测试目标不编不跑，只编要现判的那几个
  local functions_build_arguments=() functions_owners_all="" functions_plan_total
  plan_test_functions "${environment[@]}"
  if [[ "$functions_plan_state" == unavailable ]]; then
    echo "  ✗ 测试函数那一段没判：$functions_plan_note，${#targets[@]} 个测试目标一个都没编、没起；这一趟退 75（不是红，也不算通过）"
    echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好之后重交同一条命令"
    exit 75
  fi
  if [[ "$functions_plan_state" == nolibrary ]]; then
    echo "  ✗ 测试函数那一段没判：$functions_plan_note，${#targets[@]} 个测试目标一个都没编、没起；这一趟退 2"
    echo "     → 怎么办：在 git 工作树的顶上跑；rsync 拷的仓副本先在副本根 git init（键里不带哪棵仓，副本与主仓读写同一份库）"
    exit 2
  fi
  # 按条复用失灵（文件头「按条复用」）：不能复用就判红，而且没有计划就排不掉登记的用例（它们是整个崩溃枚举，不能当普通测试跑），所以一个测试目标都不起，
  #   每个点到测试目标的项记一笔红
  if [[ "$functions_plan_state" == failed ]]; then
    echo "  ✗ 测试函数那一段没判：$functions_plan_note，${#targets[@]} 个测试目标一个都没编、没起"
    for target in "${targets[@]}"; do
      for owner in $(awk -F'\t' -v wanted="$target" '$2 == "test-functions" && $3 == wanted { print $1 }' "$plan_file"); do functions_owners_all+=" $owner"; done
    done
    for owner in $(tr ' ' '\n' <<<"$functions_owners_all" | awk 'NF && !seen[$0]++'); do
      record_part "$owner" test-functions "按条复用" 1 red 0 1 0 0 "$functions_plan_note"
    done
    return 0
  fi
  : >"$run_directory/test-functions.tsv"
  for target in "${targets[@]}"; do
    test_functions_target_counts "$target"
    # 这个测试目标里的测试函数全登记成了崩溃枚举用例或放量分项（计划里只有 registered 行）：这一段没有可判的，不编不起，也不记一笔
    if (( functions_target_registered > 0 && functions_target_judge + functions_target_reuse + functions_target_refused == 0 )); then
      echo "  · 测试函数那一段：$target 里的测试函数全登记成了崩溃枚举用例或放量分项，不在这一段判"
      continue
    fi
    active_targets+=("$target")
  done
  (( ${#active_targets[@]} )) || return 0
  for target in "${active_targets[@]}"; do test_functions_target_all_reused "$target" || functions_build_arguments+=(--test "$target"); done
  build_log="$run_directory/test-functions-build.log"
  started=$SECONDS
  if (( ${#functions_build_arguments[@]} )); then
    run_logged "$build_log" "${environment[@]}" cargo test --release -p singlefs-verification ${features:+--features "$features"} --no-run "${functions_build_arguments[@]}"
    build_exit="$(cat "$build_log.exit" 2>/dev/null || echo 255)"
  else
    build_exit=0
    echo "  · 按条复用：${#active_targets[@]} 个测试目标编得进的测试函数全有键相同、不红的判定，这一趟不编不起 cargo"
  fi
  for target in "${active_targets[@]}"; do
    owners="$(awk -F'\t' -v wanted="$target" '$2 == "test-functions" && $3 == wanted { print $1 }' "$plan_file")"
    functions_owners_all+=" $owners"
    test_functions_target_counts "$target"
    target_units="$((functions_target_judge + functions_target_refused)):$functions_target_reuse"
    # 键全相同、库里不红：照复用记绿，不起 cargo（文件头「按条复用」）
    if test_functions_target_all_reused "$target"; then
      record_test_functions_target "$target" - 0
      for owner in $owners; do record_part "$owner" test-functions "$target" 0 green 0 0 0 0 "" "$target_units"; done
      continue
    fi
    if [[ "$build_exit" != 0 ]]; then
      for owner in $owners; do record_part "$owner" test-functions "$target" "$build_exit" red 0 1 unknown "$((SECONDS - started))" "编不过（cargo test --no-run 退 $build_exit，日志 $build_log、错误警告 ${build_log%.log}.problems.log）" "$target_units"; done
      continue
    fi
    # 键相同的跳过名单（文件头「按条复用」）：复用的、登记给另两段的都在里面
    test_functions_target_skips "$target"
    run_log="$run_directory/test-functions.$target.log"
    started=$SECONDS
    # 键相同的经 --exact --skip 跳过（文件头「按条复用」）
    target_libtest_arguments=(${test_threads_arguments[@]+"${test_threads_arguments[@]}"} ${functions_target_skip_arguments[@]+"${functions_target_skip_arguments[@]}"})
    run_logged "$run_log" "${environment[@]}" cargo test --release -p singlefs-verification ${features:+--features "$features"} --test "$target" -- --nocapture ${target_libtest_arguments[@]+"${target_libtest_arguments[@]}"}
    run_exit="$(cat "$run_log.exit" 2>/dev/null || echo 255)"
    judge_test_functions_log "$run_exit" "$run_log"
    record_test_functions_target "$target" "$run_log"
    if [[ -n "$functions_target_record_red" ]]; then
      functions_state=red; functions_exit=1; (( functions_red > 0 )) || functions_red=1
      functions_reason="${functions_reason:+$functions_reason；}$functions_target_record_red"
    fi
    for owner in $owners; do record_part "$owner" test-functions "$target" "$functions_exit" "$functions_state" "$functions_tests" "$functions_red" "$functions_reused" "$((SECONDS - started))" "$functions_reason" "$target_units"; done
  done
  functions_plan_total="$(awk -F'\t' '{ judged += $2; reused += $3; refused += $4; if ($5 == "reused") whole++ } END { printf "现判 %d、复用 %d、拒算 %d；全部复用、不起 cargo 的测试目标 %d 个", judged, reused, refused, whole }' "$run_directory/test-functions.tsv")"
  echo "  · 按条复用（判定库 $functions_plan_library/singlefs-unit-verdicts/$functions_plan_kind，与门禁 crash-point-replay 同一份）：测试函数 $functions_plan_total；判绿写回 $functions_recorded 条、判红记下 $functions_red_recorded 条（撤掉同一把键原有的绿）"
}

run_enumeration_part() { # <项> <crash-case: 键…>：交门禁 crash-point-replay --item（键相同的块复用、对不上的现判，写回同一份库）
  local item="$1" key arguments=() log exit_code state reason reused started red_cases=0
  shift
  started=$SECONDS
  # 点名了项的跑法（scope=items）只走按块那一路：测试函数那一段已经按点名项的测试目标判过，门禁不再把整个验证包按条判一遍
  #   （门禁文件头「--crash-cases-only」；scripts/gate/merge-and-verify.sh 的验证包那一步同样这样起）。不带 --item 的跑法照旧整道，验证包里不属于任何项的测试目标由它那一趟罩住。
  #   弄坏开关 enumeration-gate-whole-package：点名了项的跑法也不带这三个参数
  if [[ "$scope" == items ]] && ! replay_break_is enumeration-gate-whole-package; then
    arguments+=(--check crash-case-replay --crash-cases-only)
  fi
  for key in "$@"; do arguments+=(--item "$key"); done
  log="$run_directory/enumeration.$item.log"
  [[ "$shard_phase" == shard ]] && log="$run_directory/enumeration.$item.${1#crash-case:}.log"   # 一片一趟只交一条用例，同一项的几条各一份日志
  echo "── 崩溃点重放部分（$item）：bash .claude/gate.d/crash-point-replay.sh ${arguments[*]} <根>"
  run_logged "$log" bash "$repository_root/.claude/gate.d/crash-point-replay.sh" "${arguments[@]}" "$repository_root"
  exit_code="$(cat "$log.exit" 2>/dev/null || echo 255)"
  record_enumeration_scope "$log" "$@"
  reused="$(grep -cE '^  · crash-case:[a-z0-9-]+ 复用：' "$log")" || reused=0
  replay_break_is enumeration-exit-ignored && exit_code=0
  case "$exit_code" in
    0) state=green; reason="" ;;
    77) state=not-run; reason="崩溃点重放那一道退 77（本次未跑）" ;;
    3) state=not-run; reason="崩溃点重放那一道退 3（样本档，没跑真的崩溃点重放）" ;;
    78) state=not-run; reason="崩溃点重放那一道退 78（运行条件不满足：修好设备，或改配置）" ;;
    *) state=red; reason="崩溃点重放那一道退 $exit_code（日志 $log、错误警告 ${log%.log}.problems.log）"; red_cases=$# ;;
  esac
  local judged_cases=$(( $# > reused ? $# - reused : 0 )) units
  units="$judged_cases:$reused"
  [[ "$state" == not-run ]] && units=0:0
  record_part "$item" enumeration "$*" "$exit_code" "$state" "$#" "$red_cases" "$reused" "$((SECONDS - started))" "$reason" "$units"
}

# 收尾（EXIT trap）：删这一趟自己建的编译目录（文件头「编译目录」）；没删成而本来要退 0 的改退 1。
#   弄坏开关 run-target-kept：编译目录不删
replay_finish() {
  local finish_exit=$?
  trap - EXIT
  if ! run_build_directory_end $(replay_break_is run-target-kept && echo keep); then
    (( finish_exit != 0 )) || finish_exit=1
  fi
  exit "$finish_exit"
}

# 放量入口日志里这一项的分项驱动打的 CRASH_AMPLIFICATION_NOT_RUN step=judge … reason=跑的过程中源码变了…（crash-amplification-item.sh 文件头「源码中途变了」：
#   录入段之后被测源码的摘要在判段前后对不上，驱动退 77、入口记这一项 verdict=not-run）：取 reason= 之后原样；这一项没有这一行打空串。
#   分项的区段从入口打的「== 分项 <名>（」起到下一个「== 分项 」止
pipeline_source_changed_reason() { # <放量入口的日志> <分项>
  awk -v name="$2" '
    index($0, "== 分项 " name "（") == 1 { inside = 1; next }
    index($0, "== 分项 ") == 1 { inside = 0 }
    inside && index($0, "CRASH_AMPLIFICATION_NOT_RUN step=judge ") == 1 && (at = index($0, " reason=跑的过程中源码变了")) { print substr($0, at + 8); exit }' "$1"
}

run_pipeline_part() { # <计划文件>：放量部分（pipeline 的件）一趟交 crash-amplification.sh
  local plan_file="$1" requested=() arguments=() log exit_code line name started fields state reason all_named=1 word
  local item_exit item_states item_red item_reused item_verdict source_changed_reason units
  while IFS=$'\t' read -r _plan_item plan_kind plan_object; do
    [[ "$plan_kind" == pipeline ]] || continue
    requested+=("$plan_object")
  done <"$plan_file"
  (( ${#requested[@]} )) || return 0
  for word in "${!pipeline_owner[@]}"; do [[ " ${requested[*]} " == *" $word "* ]] || all_named=0; done
  # 全部项都点了就不带 --item（放量入口照全部项判，scope=whole）；判定在 prune-server 里那一份库：判过的块照服务复用，被打断之后重跑同一条命令就是续跑；
  #   SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES 不设，从调用方环境继承
  if [[ "$scope" == items ]] || (( ! all_named )); then
    for word in "${requested[@]}"; do arguments+=(--item "$word"); done
  fi
  # 套在内存包装里：外层的上限给放量入口（文件头「内存包装」）
  log="$run_directory/pipeline.log"
  echo "── 放量部分：bash scripts/crash/crash-amplification.sh ${arguments[*]}"
  started=$SECONDS
  local checked_environment=(SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1)   # 这一趟在计划那一步（或不分片的开头）已判过指定范围的上限，放量入口不再判一遍；弄坏开关 scoped-domain-handshake-dropped 下不交
  replay_break_is scoped-domain-handshake-dropped && checked_environment=()
  run_logged "$log" env ${checked_environment[@]+"${checked_environment[@]}"} bash "$repository_root/scripts/crash/crash-amplification.sh" ${arguments[@]+"${arguments[@]}"}
  exit_code="$(cat "$log.exit" 2>/dev/null || echo 255)"
  for name in "${requested[@]}"; do
    line="$(grep -E "^CRASH_AMPLIFICATION_ITEM item=$name " "$log" | tail -n 1)"
    if [[ -z "$line" ]]; then
      replay_break_is skip-collection-count && continue
      # 放量入口退 78（运行条件不满足：卡用不上、第二台连不上……）：这一项一行都没判，记本次未跑，不记红（文件头「一项怎么判」）
      if [[ "$exit_code" == 78 ]] && ! replay_break_is pipeline-78-as-red; then
        record_part "${pipeline_owner[$name]}" pipeline "$name" 78 not-run 0 0 0 "$((SECONDS - started))" "运行条件不满足：放量入口退 78、没打 $name 的 CRASH_AMPLIFICATION_ITEM 行，这一项本次未跑（出路见入口打的那一句，日志 $log）"
        continue
      fi
      record_part "${pipeline_owner[$name]}" pipeline "$name" "$exit_code" red 0 1 unknown "$((SECONDS - started))" "放量入口退 $exit_code、没打 $name 的 CRASH_AMPLIFICATION_ITEM 行（收不回，日志 $log、错误警告 ${log%.log}.problems.log）" "1:0"
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
      # 判段跑的过程中源码变了（共用主工作区上别的会话往树里打了补丁）：不是红，原因点名出来，免得当成借不到显存；弄坏开关 source-changed-reason-dropped 下不点名
      source_changed_reason="$(pipeline_source_changed_reason "$log" "$name")"
      if [[ -n "$source_changed_reason" ]] && ! replay_break_is source-changed-reason-dropped; then
        reason="${reason}：${source_changed_reason}；这不是红也不是通过，等树里的改动停下再重跑同一条命令"
      fi
    elif [[ "$item_verdict" != green ]]; then
      state=red; reason="放量 $name 的 verdict 认不得（$line）"
    fi
    [[ "$item_red" =~ ^[0-9]+$ ]] || item_red=1
    [[ "$item_states" =~ ^[0-9]+$ ]] || item_states=0
    units="1:0"
    if [[ "$state" == not-run ]]; then units=0:0
    elif (( item_states == 0 )) && [[ "$item_reused" =~ ^[1-9][0-9]*$ ]]; then units=0:1; fi
    record_part "${pipeline_owner[$name]}" pipeline "$name" "$item_exit" "$state" "$item_states" "$item_red" "$item_reused" "$((SECONDS - started))" "$reason" "$units"
  done
}

# 汇总：按计划的项次序逐项打 CRASH_POINT_REPLAY_ITEM、末行 TOTAL、成功句；退出码写进 $run_directory/exit
summarize() { # <计划文件> <范围的中文名>
  python3 - "$1" "$part_records" "$scope" "$2" "$run_directory/exit" "$(replay_break_is skip-collection-count && echo 1 || echo 0)" \
    "$narrowed_labels" "$(replay_break_is item-done-reused-silent && echo 1 || echo 0)" "$(replay_break_is item-done-seconds-whole-run && echo 1 || echo 0)" \
    "$(replay_break_is records-decoded-strictly && echo strict || echo replace)" <<'PYTHON'
import collections, sys
plan_file, records_file, scope, scope_title, exit_file, skip_count, narrowed, reused_silent, seconds_whole_run, records_decoding = sys.argv[1:11]
order = []
for line in open(plan_file, encoding="utf-8"):
    item = line.rstrip("\n").split("\t")[0]
    if item and item not in order:
        order.append(item)
parts = collections.defaultdict(list)
for line in open(records_file, encoding="utf-8", errors=records_decoding):   # 记录里的说明可能带不成字符的字节（别的程序的日志行原样进来）：坏字节换成 U+FFFD，不让汇总崩；弄坏开关 records-decoded-strictly 下严格解码
    fields = line.rstrip("\n").split("\t")
    if len(fields) == 11:
        parts[fields[0]].append(fields)
totals = collections.Counter()
suffix = " " + narrowed if narrowed else ""   # 缩过域的那一趟 ITEM、TOTAL 行末尾接 expand_up_to=<n>
narrowed_text = f"；缩过域（{narrowed}，只往小缩，大于引擎里放量的展开上限常量的不放宽），不算门禁通过" if narrowed else ""
whole_seconds = sum(int(row[8]) for rows_of_an_item in parts.values() for row in rows_of_an_item if row[8].isdigit())
reuse_unknown = 0
unknown_functions_targets = set()   # 测试函数那一段取不出复用数的测试目标（文件头「测试函数的复用数怎么取」），几项都点名的只算一次
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
    functions_tests, functions_red = total("test-functions", 5), total("test-functions", 6)
    # 测试函数的复用（文件头「测试函数的复用数怎么取」）：取得出的测试目标相加；取不出的不并进来、逐个记名；全是写死的 none（弄坏开关）照原样打
    functions_known = [int(row[7]) for row in by_part["test-functions"] if row[7].isdigit()]
    functions_unknown_targets = sorted({row[2] for row in by_part["test-functions"] if not row[7].isdigit()})
    if not by_part["test-functions"]:
        functions_reused, functions_reuse_unknown = "-", "-"
    elif all(row[7] == "none" for row in by_part["test-functions"]):
        functions_reused, functions_reuse_unknown = "none", "-"
    else:
        functions_reused = sum(functions_known) if functions_known or not functions_unknown_targets else "unknown"
        functions_reuse_unknown = len(functions_unknown_targets)
        totals["test_functions_reused_known"] += sum(functions_known)
        unknown_functions_targets.update(functions_unknown_targets)
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
    for name, value in (("test_functions_tests", functions_tests), ("pipeline_states", amp_states), ("enumeration_cases", enumeration_cases), ("enumeration_reused", enumeration_reused)):
        if isinstance(value, int):
            totals[name] += value
    totals["seconds"] += seconds
    verdicts[item] = (verdict, [row[9] for row in rows if row[4] == verdict and row[9]])
    # ITEM_DONE（文件头「输出」）：件数取各段记录的第十一列「现判:复用」，本次未跑的段不算件；seconds 是这一项各段的挂钟秒数之和
    done_judged = done_reused = 0
    for row in rows:
        if row[4] == "not-run":
            continue
        judged_part, _, reused_part = row[10].partition(":")
        done_judged += int(judged_part) if judged_part.isdigit() else 0
        done_reused += int(reused_part) if reused_part.isdigit() else 0
    done_seconds = whole_seconds if seconds_whole_run == "1" else seconds
    if not (reused_silent == "1" and done_judged == 0 and done_reused > 0):
        print(f"ITEM_DONE item={item} seconds={done_seconds} judged={done_judged} reused={done_reused} exit={exit_code}", file=sys.stderr)
    print(f"CRASH_POINT_REPLAY_ITEM item={item} scope={scope} exit={exit_code} verdict={verdict} test_functions_tests={functions_tests} test_functions_red={functions_red} "
          f"test_functions_reused={functions_reused} test_functions_reuse_unknown={functions_reuse_unknown} pipeline_states={amp_states} pipeline_red={amp_red} pipeline_reused={amp_reused} "
          f"enumeration_cases={enumeration_cases} enumeration_red={enumeration_red} enumeration_reused={enumeration_reused} seconds={seconds}{suffix}")
dispatched, collected = len(order), len(verdicts)
red = [item for item, (verdict, _) in verdicts.items() if verdict == "red"]
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」）由外面的 shell 写，红的项交给它
with open(exit_file + ".red-items", "w", encoding="utf-8") as handle:
    handle.write("".join(item + "\n" for item in red))
# 这一趟真判过的项（绿或红；本次未跑的不算）：上次红的清单只换它们的状态（lib_reuse_checks.py 文件头「三」）
with open(exit_file + ".judged-items", "w", encoding="utf-8") as handle:
    handle.write("".join(item + "\n" for item, (verdict, _) in verdicts.items() if verdict in ("red", "green")))
not_run = [item for item, (verdict, _) in verdicts.items() if verdict == "not-run"]
green = [item for item, (verdict, _) in verdicts.items() if verdict == "green"]
print(f"CRASH_POINT_REPLAY_TOTAL scope={scope} dispatched={dispatched} collected={collected} green={len(green)} red={len(red)} not_run={len(not_run)} "
      f"test_functions_tests={totals['test_functions_tests']} test_functions_reused_known={totals['test_functions_reused_known']} test_functions_reuse_unknown={len(unknown_functions_targets)} "
      f"pipeline_states={totals['pipeline_states']} pipeline_reused_known={totals['pipeline_reused_known']} "
      f"pipeline_reuse_unknown={reuse_unknown} enumeration_cases={totals['enumeration_cases']} enumeration_reused={totals['enumeration_reused']} seconds={totals['seconds']}{suffix}")
for item in red:
    for reason in verdicts[item][1]:
        print(f"  ✗ {item}：{reason}", file=sys.stderr)  # gate-lint:detail
for item in not_run:
    for reason in verdicts[item][1]:
        print(f"  · 本次未跑 {item}：{reason}", file=sys.stderr)
for item in red:   # 红的项里另有本次未跑的段（例如崩溃点重放那一道退 77），一并点名
    for reason in dict.fromkeys(row[9] for row in parts[item] if row[4] == "not-run" and row[9]):
        print(f"  · 本次未跑 {item}：{reason}", file=sys.stderr)
if narrowed:
    print(f"  ! 缩过域（{narrowed}）：这一趟的展开深度被缩过，只往小缩；不算门禁通过，结果行里的数不当全量", file=sys.stderr)
count_mismatch = dispatched != collected and skip_count != "1"
if red or count_mismatch:
    if count_mismatch:
        missing = [item for item in order if item not in verdicts]
        print(f"  ✗ 派出 {dispatched} 项、收回 {collected} 项：{'、'.join(missing)} 一段都没记下", file=sys.stderr)  # gate-lint:detail
    print(f"  ✗ 崩溃点重放{scope_title}：{len(red)} 项红（{'、'.join(red) or '无'}）", file=sys.stderr)  # gate-lint:summary
    rerun = "只重跑红的那 %d 项：bash scripts/crash/crash-point-replay.sh %s" % (len(red), " ".join(f"--item {item}" for item in red))
    print("  → 怎么办：照上面每一项的原因与日志查；红了必须修好或查清原因、红着不提交（.claude/rules/verification.md）；"
          + rerun, file=sys.stderr)
    code = 1
elif any_78:
    print(f"  ✗ 崩溃点重放{scope_title}：有一段运行条件不满足（退 78），这一趟没跑全", file=sys.stderr)
    print("  → 怎么办：修好设备，或改配置（多机配置 multi-host.env）；看上面退 78 的那一段打的出路", file=sys.stderr)
    code = 78
elif not green:
    print(f"  ! 崩溃点重放{scope_title}：派出的 {dispatched} 项全是本次未跑（这不是通过）", file=sys.stderr)
    code = 77
else:
    not_run_text = "；本次未跑 %d 项：%s" % (len(not_run), "、".join(f"{item}（{'；'.join(verdicts[item][1])}）" for item in not_run)) if not_run else ""
    fast_reuse_text = (f"复用 {totals['test_functions_reused_known']} 块" + (f"（另有 {len(unknown_functions_targets)} 个测试目标取不出复用数：{'、'.join(sorted(unknown_functions_targets))}）"
                                                                       if unknown_functions_targets else ""))
    print(f"  ✓ 崩溃点重放{scope_title}：{len(green) + len(not_run)} 项都不红{not_run_text}；测试函数 {totals['test_functions_tests']} 条{fast_reuse_text}、"
          f"放量现判 {totals['pipeline_states']} 态复用 {totals['pipeline_reused_known']} 态"
          f"{'（另有 %d 项取不出复用数）' % reuse_unknown if reuse_unknown else ''}、崩溃枚举用例 {totals['enumeration_cases']} 条复用 {totals['enumeration_reused']} 条；"
          f"派出 {dispatched} 项、收回 {collected} 项{narrowed_text}")
    code = 0
open(exit_file, "w").write(f"{code}\n")
PYTHON
}

# ── 经 task-server 按件分片（文件头「经 task-server 按件分片」）
MOST_SHARDS="${replay_most_shards_override:-64}"   # 同 task-server/task_server_lib.py 的 MAXIMUM_SHARDS；CRASH_POINT_REPLAY_MOST_SHARDS 只供自证
shard_refuse() { # <卡在哪> <下一步> <退出码>
  echo "  ✗ 按件分片的 $shard_phase 那一步：$1" >&2
  echo "  → 怎么办：$2" >&2
  exit "$3"
}

# 计划那一步：每条崩溃枚举用例、每个放量分项一件，只有测试函数或本次未跑的项各算一件；一件一片，超过上限时相邻的件合成一片
shard_write_the_plan() {
  [[ -n "$shard_plan_directory" ]] || shard_refuse "环境里没有 TASK_SERVER_SHARD_PLAN_DIRECTORY" "这一步只由 task-server 照分片协议起（task-server/task_server_lib.py 文件头「分片」）" 2
  mkdir -p -- "$shard_plan_directory" || shard_refuse "建不了计划目录 $shard_plan_directory" "看共享工作根可不可写" 78
  python3 - "$plan_file" "$shard_plan_directory" "$MOST_SHARDS" "$(replay_break_is plan-shards-not-merged && echo 1 || echo 0)" <<'PYTHON' || shard_refuse "写不成计划" "看计划目录 $shard_plan_directory" 1
import json, os, sys
plan_file, directory, most, not_merged = sys.argv[1:5]
rows = [line.rstrip("\n").split("\t") for line in open(plan_file, encoding="utf-8") if line.strip()]
parts = [row for row in rows if row[1] in ("enumeration", "pipeline")]
items_with_parts = {row[0] for row in parts}
units = [(row[2], [row]) for row in parts]
alone = {}
for row in rows:
    if row[0] not in items_with_parts:
        alone.setdefault(row[0], []).append(row)
units += [(item, item_rows) for item, item_rows in alone.items()]
count = len(units) if not_merged == "1" else min(len(units), int(most))
shards = [[] for _ in range(count)]
for index, unit in enumerate(units):
    shards[index * count // len(units)].append(unit)
with open(os.path.join(directory, "shard-plan.tsv"), "w", encoding="utf-8") as handle:
    for number, shard in enumerate(shards):
        lines = [row for row in rows if row[1] == "test-functions" and row[0] in {unit_rows[0][0] for _, unit_rows in shard}]
        lines += [row for _, unit_rows in shard for row in unit_rows if row not in lines]
        handle.writelines("\t".join([str(number), *row]) + "\n" for row in lines)
with open(os.path.join(directory, "plan.tsv"), "w", encoding="utf-8") as handle:
    handle.writelines("\t".join(row) + "\n" for row in rows)
with open(os.path.join(directory, "plan.json"), "w", encoding="utf-8") as handle:
    json.dump({"shards": [{"label": "、".join(name for name, _ in shard)} for shard in shards], "note": f"崩溃点重放按件分片：{len(units)} 件、{count} 片"}, handle, ensure_ascii=False)
print(f"  ✓ 按件分片的计划：{len(units)} 件、{count} 片，写进 {directory}（plan.tsv、shard-plan.tsv、plan.json）")
PYTHON
}

# 计划那一步另打 ITEM_PLAN shard=<片号> item=<项名> to_judge=<n> reusable=<n>（文件头「输出」，stdout）：测试函数那一段的键要算（admission.py test-plan）；
#   算不成只打一行说明、不影响已经写好的计划
shard_report_the_item_plan() {
  test_functions_environment
  plan_test_functions "${functions_environment[@]}"
  if [[ "$functions_plan_state" == planned ]]; then
    emit_item_plan_lines shard "$plan_file" "$shard_plan_directory/shard-plan.tsv" "$functions_plan_output"
  else
    echo "  ! 按项机读行 ITEM_PLAN 没打：测试函数那一段的计划算不成（${functions_plan_note:-$functions_plan_state}）" >&2
  fi
}

# 一片与合并：读计划。一片只读自己的行（<片号>），合并读整趟。设 shard_count、selected_items
shard_read_the_plan() { # [<片号>]
  local plan_item
  [[ -f "$shard_plan_directory/plan.tsv" && -f "$shard_plan_directory/shard-plan.tsv" ]] \
    || shard_refuse "计划目录里没有 plan.tsv 或 shard-plan.tsv（${shard_plan_directory:-没给计划目录}）" "这一步只由 task-server 照分片协议在计划那一步之后起" 2
  shard_count="$(awk -F'\t' 'END { print $1 + 1 }' "$shard_plan_directory/shard-plan.tsv")"
  if [[ -n "${1:-}" ]]; then
    awk -F'\t' -v number="$1" 'BEGIN { OFS = "\t" } $1 == number { print $2, $3, $4 }' "$shard_plan_directory/shard-plan.tsv" >"$plan_file"
    [[ -s "$plan_file" ]] || shard_refuse "片号对不上计划：第 $1 片在 shard-plan.tsv 里一行都没有" "照计划那一步写的 plan.json 的片数起" 2
  else
    cp -- "$shard_plan_directory/plan.tsv" "$plan_file" || shard_refuse "读不了计划 $shard_plan_directory/plan.tsv" "看共享工作根可不可写" 78
  fi
  selected_items=()
  while IFS=$'\t' read -r plan_item _; do
    [[ " ${selected_items[*]-} " == *" $plan_item "* ]] || selected_items+=("$plan_item")
  done <"$plan_file"
}

# 一片的收尾：把判的结论存进产物目录；shard-finished 最后写，合并认它才算这一片收齐
shard_save_the_output() {
  local log
  mkdir -p -- "$shard_output_directory" \
    && cp -- "$part_records" "$shard_output_directory/" \
    || shard_refuse "存不进产物目录 $shard_output_directory" "看共享工作根可不可写、盘满没满" 78
  if [[ -f "$run_directory/crash-flow-records.tsv" ]]; then
    cp -- "$run_directory/crash-flow-records.tsv" "$shard_output_directory/" || shard_refuse "存不进产物目录 $shard_output_directory" "看共享工作根可不可写、盘满没满" 78
  fi
  for log in "$run_directory"/*.problems.log; do
    [[ -f "$log" ]] && { cp -- "$log" "$shard_output_directory/" || shard_refuse "存不进产物目录 $shard_output_directory" "看共享工作根可不可写、盘满没满" 78; }
  done
  : >"$shard_output_directory/shard-finished" || shard_refuse "存不进产物目录 $shard_output_directory" "看共享工作根可不可写、盘满没满" 78
}

# 合并：核各片的产物都收齐（不全就一件都不回写），把它们并进这一趟的 parts.tsv、结论表与覆盖记录
shard_collect_the_outputs() {
  local shard_index shard_host shard_directory directories=() ordered=()
  [[ -f "$shard_outputs" ]] || shard_refuse "环境里的 TASK_SERVER_SHARD_OUTPUTS 不是一份文件（${shard_outputs:-没给}）" "这一步只由 task-server 在各片都成了之后起" 2
  while IFS=$'\t' read -r shard_index shard_host shard_directory; do
    [[ "$shard_index" =~ ^[0-9]+$ ]] && directories[shard_index]="$shard_directory"
  done <"$shard_outputs"
  for ((shard_index = 0; shard_index < shard_count; shard_index++)); do
    shard_directory="${directories[shard_index]:-}"
    if [[ -z "$shard_directory" || ! -f "$shard_directory/shard-finished" ]]; then
      replay_break_is merge-incomplete-accepted && continue
      shard_refuse "第 $shard_index 片的产物不全：产物表 $shard_outputs 里$([[ -n "$shard_directory" ]] && echo "指到 $shard_directory，缺 shard-finished" || echo "没有这一片")，一件都没回写" \
        "看那一片的日志（task-server/task status <号> 列各片）；重交整件，判过的块照单元键复用" 1
    fi
    ordered+=("$shard_directory")
  done
  : >"$run_directory/shards-parts.tsv"; : >"$run_directory/crash-flow-records.tsv"
  for shard_directory in ${ordered[@]+"${ordered[@]}"}; do
    cat -- "$shard_directory/parts.tsv" >>"$run_directory/shards-parts.tsv"
    [[ ! -f "$shard_directory/crash-flow-records.tsv" ]] || cat -- "$shard_directory/crash-flow-records.tsv" >>"$run_directory/crash-flow-records.tsv"
  done
  # 同一个测试目标几片都跑了的测试函数行只算一次，有红取红；别的行原样
  awk -F'\t' '
    $2 != "test-functions" { rows[++count] = $0; next }
    { key = $1 FS $3
      if (!(key in position)) { rows[++count] = $0; position[key] = count; state[key] = $5 }
      else if ($5 == "red" && state[key] != "red") { rows[position[key]] = $0; state[key] = "red" } }
    END { for (row = 1; row <= count; row++) print rows[row] }' "$run_directory/shards-parts.tsv" >"$part_records"
  echo "  · 按件分片的合并：${#ordered[@]} 片的产物（各片的日志与错误警告在产物表 $shard_outputs 点名的目录）"
}


# 经 task-server 按件分片（文件头「经 task-server 按件分片」）：读完就从环境里清掉，不传给下层
shard_phase="${TASK_SERVER_SHARD_PHASE:-}"; shard_position="${TASK_SERVER_SHARD:-}"; shard_plan_directory="${TASK_SERVER_SHARD_PLAN_DIRECTORY:-}"
shard_output_directory="${TASK_SERVER_SHARD_OUTPUT_DIRECTORY:-}"; shard_outputs="${TASK_SERVER_SHARD_OUTPUTS:-}"
unset TASK_SERVER_SHARD_PHASE TASK_SERVER_SHARD TASK_SERVER_SHARD_PLAN_DIRECTORY TASK_SERVER_SHARD_OUTPUT_DIRECTORY TASK_SERVER_SHARD_OUTPUTS TASK_SERVER_SHARD_HOST
replay_break_is shard-phase-ignored && shard_phase=""
(( ! list_items && ! selftest )) || shard_phase=""
case "$shard_phase" in
  ""|plan|shard|merge) ;;
  *) shard_refuse "TASK_SERVER_SHARD_PHASE=$shard_phase 认不得（只认 plan、shard、merge）" "照 task-server/task_server_lib.py 文件头「分片」的协议起" 2 ;;
esac
# 不经服务起（文件头「经 task-server 按件分片」）：没有 TASK_SERVER_SHARD_PHASE 的平常跑法退 2、什么都不起；--list-items、--selftest 与自证起的子进程不拒
#   （子进程设了 CRASH_POINT_REPLAY_UNSHARDED_REFUSED=1 的照平常跑拒）。弄坏开关 unsharded-run-accepted 下不拒
if [[ -z "$shard_phase" ]] && (( ! list_items && ! selftest )) && { (( ! replay_in_selftest_child )) || [[ "$replay_unsharded_refused" == 1 ]]; } \
   && ! replay_break_is unsharded-run-accepted; then
  echo "  ✗ 不经服务起：环境里没有 TASK_SERVER_SHARD_PHASE，这一份只经统一任务服务按件分片跑，一项都没起" >&2
  echo "  → 怎么办：照原参数经 task-server/task submit -- bash scripts/crash/crash-point-replay.sh [--item <项>]… 交（范围声明写在 submit 前面），服务拆成计划、各片、合并" >&2
  exit 2
fi

if (( selftest )); then
  source "$replay_script_directory/crash-point-replay-selftest.sh"   # 自证的函数只在这一支用，别的调用不载入（路径同 replay_selftest_library）
  (( list_items || with_keys || ${#named_items[@]} )) && usage "--selftest 不与别的参数一起给"
  # 开关轮分段（文件头「开关轮分段」）：写错、与选格或弄坏开关同给都退 2，排在查临时服务之前
  if [[ -n "$replay_wheel_segment" ]]; then
    selftest_wheel_segment_choose "$replay_wheel_segment" || exit 2
    if [[ -n "$replay_selftest_cells" || -n "$replay_break" ]]; then
      echo "  ✗ --selftest：CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT 只配整份自证，不与 CRASH_POINT_REPLAY_SELFTEST_CELLS、CRASH_POINT_REPLAY_BREAK 一起给" >&2
      echo "  → 怎么办：去掉其中一边：点名格用 CRASH_POINT_REPLAY_SELFTEST_CELLS（不跑开关轮），点名开关轮的一段用 CRASH_POINT_REPLAY_SELFTEST_WHEEL_SEGMENT" >&2
      exit 2
    fi
  fi
  if [[ -z "$replay_selftest_cells" && -z "$replay_break" ]] && ! crash_amplification_switch_wheel_inner; then
    run_selftest_parallel
  else
    run_selftest
  fi
  exit $?
fi
(( with_keys && ! list_items )) && usage "--with-keys 只跟 --list-items 一起给"

# --list-items --with-keys 的 ITEM_PLAN（文件头「输出」）：<项…> 每项一行，件数取 test-plan 对测试函数那一段的计划（按条复用的键对不上与拒算的现判、键相同的复用）加崩溃枚举用例与放量分项各一件。
#   计划算不成交回 1（点名为什么），prune-server 不可用交回 75。弄坏开关 item-plan-selection-ignored 下点名了项也给全部项各打一行
report_the_item_plan() {
  local plan_items=("$@") work_directory
  work_directory="$(mktemp -d "${TMPDIR:-/tmp}/crash-point-replay-plan.XXXXXX")" || { echo "  ✗ 建不了算 ITEM_PLAN 的临时目录" >&2; echo "     → 怎么办：看 ${TMPDIR:-/tmp} 满没满、可不可写，腾出地方再跑同一条命令" >&2; return 1; }
  run_directory="$work_directory"; plan_file="$work_directory/plan.tsv"
  replay_break_is item-plan-selection-ignored && plan_items=("${item_names[@]}" ${whole_run_names[@]+"${whole_run_names[@]}"})
  replay_plan ${plan_items[@]+"${plan_items[@]}"} >"$plan_file"
  test_functions_environment
  plan_test_functions "${functions_environment[@]}"
  case "$functions_plan_state" in
    planned) emit_item_plan_lines list "$plan_file" - "$functions_plan_output" ;;
    off) emit_item_plan_lines list "$plan_file" - - ;;
    unavailable)
      echo "  ✗ ITEM_PLAN 算不出：$functions_plan_note" >&2
      echo "  → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好之后重跑" >&2
      rm -rf -- "$work_directory"; return 75 ;;
    *)
      echo "  ✗ ITEM_PLAN 算不出：$functions_plan_note" >&2
      echo "  → 怎么办：照上面那一句修（被判的根要是一棵 git 工作树的顶，test-plan 要算得成），再跑" >&2
      rm -rf -- "$work_directory"; return 1 ;;
  esac
  rm -rf -- "$work_directory"
}

# --list-items --with-keys：崩溃判定库里键对不上的块（只读读库程序 crash_library_reader keys；它的输出写法在
# crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」那一段）。读库程序退非 0、输出认不出照实报，交回 1。
#   读库程序打了按态数的 CRASH_LIBRARY_STATES 行就多打一句态数；没打那一行只打块数、不判红（文件头「--list-items --with-keys」）。
report_the_keys_of_the_library() {
  local output reader_exit tag first second third fourth fifth seen=0 reader_features library_name=""
  if (( replay_in_selftest_child )) && [[ -n "$replay_keys_reader_output" ]]; then
    output="$(cat -- "$replay_keys_reader_output")"; reader_exit=$?
  else
    # 与真跑编同一套特性（文件头「--list-items --with-keys」）：gpu 开着带 gpu，再加 验证包 [dev-dependencies] 里带特性的依赖的 <依赖>/<特性>；弄坏开关 keys-reader-build-differs 下不带
    if ! reader_features="$(keys_reader_features "$repository_root/crates/singlefs-verification/Cargo.toml" "$features")"; then
      echo "  ✗ 读库程序要带的特性算不出：读不了 $repository_root/crates/singlefs-verification/Cargo.toml 的 [dev-dependencies]" >&2
      echo "  → 怎么办：看那份清单可不可读、写法对不对；读库程序与真跑要编同一套特性，键才对得上" >&2
      return 1
    fi
    replay_break_is keys-reader-build-differs && reader_features=""
    test_functions_environment
    output="$(cd "$repository_root" && "${functions_environment[@]}" cargo run --release --quiet --offline -p singlefs-verification ${reader_features:+--features "$reader_features"} --bin crash_library_reader -- keys 2>&1)"; reader_exit=$?
  fi
  if (( reader_exit != 0 )); then
    printf '%s\n' "$output" | tail -n 5 >&2
    echo "  ✗ 读库程序 crash_library_reader keys 退 $reader_exit：键对不上的块数报不出" >&2
    echo "  → 怎么办：照上面那几行修（库不是这一版的布局就换一个新目录建库）；单跑 cargo run --release -p singlefs-verification --bin crash_library_reader -- keys" >&2
    return 1
  fi
  while IFS=$'\t' read -r tag first second third fourth fifth; do
    case "$tag" in
      CRASH_LIBRARY_KEYS)
        seen=1
        if [[ "$first" != library=* || "$second" != recorded_blocks=* || "$third" != judged_under_the_current_key=* \
              || "$fourth" != judged_only_under_another_key=* || "$fifth" != never_judged=* ]]; then
          echo "  ✗ 读库程序那一行认不出：$tag $first $second $third $fourth $fifth（写法与 crash_library_reader.rs 文件头「输出」那一段对不上）" >&2
          echo "  → 怎么办：让 report_the_keys_of_the_library 与 crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」那一段的写法对上，再跑 --list-items --with-keys" >&2
          return 1
        fi
        library_name="${first#library=}"
        printf '键\t崩溃判定库 %s：录下的块 %s，键对不上的块 %s（只在别的判法单元指纹下判过 %s、一种判器的结论都没有 %s），键相同的 %s\n' \
          "${first#library=}" "${second#recorded_blocks=}" "$(( ${fourth#judged_only_under_another_key=} + ${fifth#never_judged=} ))" \
          "${fourth#judged_only_under_another_key=}" "${fifth#never_judged=}" "${third#judged_under_the_current_key=}" ;;
      CRASH_LIBRARY_KEYS_JUDGE)
        printf '键\t%s 这一份的判法单元指纹 %s\n' "$first" "${second#current_key=}" ;;
      CRASH_LIBRARY_STATES)
        # 弄坏开关 keys-states-line-unread 下不认这一行；keys-states-line-unchecked 下不核字段
        replay_break_is keys-states-line-unread && continue
        if ! replay_break_is keys-states-line-unchecked \
           && [[ ! "$first" =~ ^recorded_states=[0-9]+$ || ! "$second" =~ ^states_judged_under_the_current_key=[0-9]+$ \
                 || ! "$third" =~ ^states_judged_only_under_another_key=[0-9]+$ || ! "$fourth" =~ ^states_never_judged=[0-9]+$ || -n "$fifth" ]]; then
          echo "  ✗ 读库程序按态数那一行认不出：$tag $first $second $third $fourth $fifth（写法与 crash_library_reader.rs 文件头「输出」那一段对不上）" >&2
          echo "  → 怎么办：让 report_the_keys_of_the_library 与 crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」那一段的写法对上，再跑 --list-items --with-keys" >&2
          return 1
        fi
        printf '键\t崩溃判定库 %s 按态数：录下的态 %s，当前键下判过的 %s、只在别的键下判过的 %s、一种判器的结论都没有的 %s\n' \
          "$library_name" "${first#recorded_states=}" "${second#states_judged_under_the_current_key=}" \
          "${third#states_judged_only_under_another_key=}" "${fourth#states_never_judged=}" ;;
      *) ;;
    esac
  done <<<"$output"
  if (( ! seen )); then
    echo "  ✗ 读库程序的输出里没有 CRASH_LIBRARY_KEYS 那一行（写法与 crash_library_reader.rs 文件头「输出」那一段对不上）" >&2
    echo "  → 怎么办：看读库程序的输出是不是编不过或换了写法，照 crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」那一段对上，再跑 --list-items --with-keys" >&2
    return 1
  fi
}

load_items
load_whole_run_items
# 核点名的项（合表的项与整趟才跑的那几类 whole-run:<类>）
unknown_items=()
for name in ${named_items[@]+"${named_items[@]}"}; do
  [[ -n "${item_cases[$name]+set}" ]] || unknown_items+=("$name")
  # 弄坏开关 whole-run-item-unnamed：整趟才跑的那几类 --item 点不到（--list-items 照列、不带参数照跑）
  [[ "$name" == whole-run:* ]] && replay_break_is whole-run-item-unnamed && unknown_items+=("$name")
done
if (( ${#unknown_items[@]} )) && ! replay_break_is accept-unknown-item; then
  echo "  ✗ 合表里没有这些项：${unknown_items[*]}" >&2
  echo "  → 怎么办：项名照下面这张表写（--list-items 打的同一份），给几次 --item 取并集：" >&2
  printf '       %s\n' "${item_names[@]}" ${whole_run_names[@]+"${whole_run_names[@]}"} >&2
  exit 2
fi
selected_items=()
for name in ${named_items[@]+"${named_items[@]}"}; do
  [[ -n "${item_cases[$name]+set}" && " ${selected_items[*]-} " != *" $name "* ]] && selected_items+=("$name")
done
if (( list_items )); then
  (( ${#named_items[@]} && ! with_keys )) && usage "--list-items 不带 --with-keys 时不与 --item 一起给"
  replay_break_is list-items-builds && cargo build --release -p singlefs-verification >/dev/null 2>&1
  # 列哪些项：不点名是全部项连同整趟才跑的那几类，点名了（只许和 --with-keys 一起）只是点名的项
  listed_items=("${item_names[@]}" ${whole_run_names[@]+"${whole_run_names[@]}"})
  (( ${#named_items[@]} )) && listed_items=(${selected_items[@]+"${selected_items[@]}"})
  for name in ${listed_items[@]+"${listed_items[@]}"}; do
    if [[ -n "${item_not_run_reason[$name]+set}" ]]; then printf '%s\t本次未跑（%s）\n' "$name" "${item_not_run_reason[$name]}"
    elif [[ "$name" == whole-run:* ]]; then printf '%s\t测试函数 %s\n' "$name" "${item_test_targets[$name]}"
    else
      parts_text="测试函数 ${item_test_targets[$name]}"
      [[ -n "${item_cases[$name]}" ]] && parts_text+=" ∪ 崩溃点重放 ${item_cases[$name]}"
      [[ -n "${item_pipeline_items[$name]}" ]] && parts_text+=" ∪ 放量 ${item_pipeline_items[$name]}"
      printf '%s\t%s\n' "$name" "$parts_text"
    fi
  done
  if (( with_keys )); then
    # 读库程序是 cargo run 起的（验证包，编到 librocksdb-sys）：起之前判取值，判红退 1；自证喂读库输出的那一趟不起 cargo、不判
    if ! (( replay_in_selftest_child )) || [[ -z "$replay_keys_reader_output" ]]; then replay_rocksdb_entry_holds || exit 1; fi
    # 算 ITEM_PLAN 的键与编读库程序都要真跑那一套配置与特性（文件头「--list-items --with-keys」）
    configuration_file="$(crash_amplification_multi_host_configuration_file "$repository_root")"
    gpu_switch=0
    [[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU)" == 1 ]] && gpu_switch=1
    features=""; (( gpu_switch )) && features="gpu"
    report_the_item_plan ${listed_items[@]+"${listed_items[@]}"} || exit $?
    pipeline_home="$(crash_amplification_home_directory)" || pipeline_home=/nonexistent   # 取不到工作根（函数已打原因）：运行记录读不到，ITEM_DOMAIN 里要靠记录估的写 unknown
    report_the_item_domain ${listed_items[@]+"${listed_items[@]}"} || exit 1
    report_the_keys_of_the_library || exit 1
  fi
  exit 0
fi

# 范围：不带 --item 是全部项（连同整趟才跑的那几类），带 --item 只是点名的项
if (( ${#named_items[@]} )); then
  scope=items; scope_title="（点名的 ${#selected_items[@]} 项）"
else
  scope=whole; scope_title="（全部项）"; selected_items=("${item_names[@]}" ${whole_run_names[@]+"${whole_run_names[@]}"})
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
# 起任何 cargo 之前（测试函数、崩溃点重放那一道、放量入口都在这之后起）判 RocksDB 预编库的取值：判红退 78、什么都不起
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
gpu_switch=0
[[ "$(crash_amplification_configuration_value "$configuration_file" ENABLE_GPU)" == 1 ]] && gpu_switch=1
features=""; (( gpu_switch )) && features="gpu"   # 计划那一步算 ITEM_PLAN 的键也要这一套特性
# 卡归 GPU 批处理验证服务（文件头「GPU」）：这里不领卡、不挑卡、不读多机配置里的显卡先后；环境里有 GPU_CARDS_BY_PRIORITY 就原样交给测试函数那几趟 cargo，没有照起。
#   弄坏开关 gpu-cards-absent-refused 下开了 GPU 而环境里没有卡就一项都不起、退 78
gpu_cards_for_test_functions="${GPU_CARDS_BY_PRIORITY:-}"
if (( gpu_switch )) && [[ -z "$gpu_cards_for_test_functions" && "$shard_phase" != plan && "$shard_phase" != merge ]] && replay_break_is gpu-cards-absent-refused; then
  echo "  ✗ 多机配置开了 GPU（ENABLE_GPU=1），环境里没有 GPU_CARDS_BY_PRIORITY：这一趟一项都没起，退 78" >&2
  echo "  → 怎么办：这是弄坏开关 gpu-cards-absent-refused 演的拒法，只在自证里出现；平常跑时卡归 GPU 批处理验证服务，环境里没有卡照起" >&2
  exit 78
fi
pipeline_home="$(crash_amplification_home_directory)" || exit 2
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
# 指定范围的崩溃域上限（文件头「不带 --item 的跑法…」）：带了 --item 才判；经服务分片的在计划那一步判，各片与合并不再判。超上限或估不出退 79，一个测试函数、崩溃点重放、放量都不起
if (( ${#named_items[@]} && ${#selected_items[@]} )) && [[ "$shard_phase" != shard && "$shard_phase" != merge ]]; then
  scoped_domain_guard "${selected_items[@]}" || exit $?
fi
if [[ "$shard_phase" == plan ]]; then
  replay_plan ${selected_items[@]+"${selected_items[@]}"} >"$plan_file"
  shard_write_the_plan
  shard_report_the_item_plan
  exit 0
elif [[ "$shard_phase" == shard ]]; then
  [[ "$shard_position" =~ ^([0-9]+)/([0-9]+)$ ]] || shard_refuse "片号对不上计划：TASK_SERVER_SHARD=$shard_position 不是 <片号>/<片数>" "照计划那一步写的 plan.json 的片数起" 2
  shard_index="${BASH_REMATCH[1]}"; shard_total="${BASH_REMATCH[2]}"
  [[ -n "$shard_output_directory" ]] || shard_refuse "环境里没有 TASK_SERVER_SHARD_OUTPUT_DIRECTORY" "这一步只由 task-server 照分片协议起（task-server/task_server_lib.py 文件头「分片」）" 2
  shard_read_the_plan "$shard_index"
  (( shard_total == shard_count )) || shard_refuse "片号对不上计划：TASK_SERVER_SHARD=$shard_position，计划是 $shard_count 片" "照计划那一步写的 plan.json 的片数起" 2
  echo "  · 按件分片：第 $shard_position 片，只跑计划里自己那几行（$(awk -F'\t' '$2 != "test-functions" { printf "%s%s", (n++ ? "、" : ""), $3 }' "$plan_file")）；结论交合并那一步"
elif [[ "$shard_phase" == merge ]]; then
  shard_read_the_plan
else
  replay_plan ${selected_items[@]+"${selected_items[@]}"} >"$plan_file"
fi
echo "── 崩溃点重放$scope_title：${#selected_items[@]} 项（多机配置 $configuration_file：GPU $( (( gpu_switch )) && echo 开 || echo 关)），日志 $run_directory/（每份 .log 旁边有同名的 .problems.log：只收错误与警告，行前带全量日志里的行号，一般只看它）"

# 计划里直接记本次未跑的（整趟才跑的那几类里没有测试目标的）
while IFS=$'\t' read -r plan_item plan_kind plan_object; do
  [[ "$plan_kind" == not-run ]] && record_part "$plan_item" not-run - - not-run 0 0 none 0 "$plan_object"
done <"$plan_file"
if [[ "$shard_phase" == merge ]]; then
  shard_collect_the_outputs
else
  run_test_functions "$plan_file"
  if [[ "$shard_phase" == shard ]]; then
    while IFS=$'\t' read -r plan_item plan_kind plan_object; do
      [[ "$plan_kind" != enumeration ]] || run_enumeration_part "$plan_item" "$plan_object"
    done <"$plan_file"
  else
    for name in ${selected_items[@]+"${selected_items[@]}"}; do
      [[ -n "${item_cases[$name]}" ]] || continue
      # shellcheck disable=SC2086  # 键是空格隔开的一串，逐个当参数
      run_enumeration_part "$name" ${item_cases[$name]}
    done
  fi
  run_pipeline_part "$plan_file"
fi
summarize "$plan_file" "$scope_title"
echo "  · 错误警告日志（每份全量日志旁边同名的 .problems.log，一般只看它）：$(find "$run_directory" -maxdepth 1 -name '*.problems.log' | LC_ALL=C sort | tr '\n' ' ')"
# 覆盖行与计数行落成产物（文件头「输出」末段）：只落产物、不判红、不改退出码；弄坏开关 flow-coverage-table-skipped 下不落
if ! replay_break_is flow-coverage-table-skipped; then
  flow_coverage_judge="$replay_script_directory/crash-flow-coverage.py"
  # 合并那一步的记录是各片存下的（shard_collect_the_outputs 并好了），这一趟的目录里没有日志可读
  if { [[ "$shard_phase" == merge ]] || python3 "$flow_coverage_judge" run-directory "$run_directory" >"$run_directory/crash-flow-records.tsv" 2>"$run_directory/crash-flow-coverage.err"; } \
     && python3 "$flow_coverage_judge" table "$repository_root" "$run_directory/crash-flow-records.tsv" >"$run_directory/crash-flow-coverage.md" 2>>"$run_directory/crash-flow-coverage.err"; then
    echo "  · 覆盖表（入口 × StepKind、ignore 开着的节点、评估 0 次的对）：$run_directory/crash-flow-coverage.md；记录 $(grep -c . "$run_directory/crash-flow-records.tsv") 条：$run_directory/crash-flow-records.tsv"
  else
    echo "  ! 覆盖表没落成（$(tail -n 1 "$run_directory/crash-flow-coverage.err")；判法 $flow_coverage_judge），这一趟的判定不受影响" >&2
  fi
fi
# 上次红的清单（scripts/gate/lib_reuse_checks.py 文件头「三」，.claude/hooks/run-scope-guard.sh 的拒绝说明读它）：红的项一行一个；
# 点不点名都只换这一趟真判过的项（summarize 写的 exit.judged-items：判绿或判红的），本次未跑、没判的项照留上一份的记录；读、合、换上在共用库的锁里做，
# 几趟同时跑不互盖。弄坏开关 last-run-whole-overwrite 下：不点名的跑法整份写、点名的交点名的项。分片时由合并那一步写，一片不写
if [[ -f "$run_directory/exit.red-items" && "$shard_phase" != shard ]]; then
  if replay_break_is last-run-whole-overwrite; then
    if (( ${#named_items[@]} > 0 )); then
      printf '%s\n' "${named_items[@]}" >"$run_directory/last-run.ran"
      python3 "$replay_script_directory/../gate/lib_reuse_checks.py" last-run scripts/crash/crash-point-replay.sh "$run_directory/exit.red-items" --ran "$run_directory/last-run.ran" >/dev/null
    else
      python3 "$replay_script_directory/../gate/lib_reuse_checks.py" last-run scripts/crash/crash-point-replay.sh "$run_directory/exit.red-items" >/dev/null
    fi
  else
    python3 "$replay_script_directory/../gate/lib_reuse_checks.py" last-run scripts/crash/crash-point-replay.sh "$run_directory/exit.red-items" --ran "$run_directory/exit.judged-items" >/dev/null
  fi || echo "  ! 上次红的清单没写成（scripts/gate/lib_reuse_checks.py last-run），这一趟的判定不受影响" >&2
fi
# 一片：结论存进产物目录；判红（1）或本次未跑（77）退 0，结论交合并那一步判（文件头「经 task-server 按件分片」）；没收出退出码的不存、退 1
if [[ "$shard_phase" == shard && -f "$run_directory/exit" ]]; then
  shard_save_the_output
  shard_exit="$(<"$run_directory/exit")"
  if (( shard_exit == 1 || shard_exit == 77 )) && ! replay_break_is shard-red-exit-kept; then shard_exit=0; fi
  exit "$shard_exit"
fi
exit "$(cat "$run_directory/exit" 2>/dev/null || echo 1)"
