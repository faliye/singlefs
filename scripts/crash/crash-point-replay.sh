#!/usr/bin/env bash
# task-server-shards: *
# task-server-cargo-family: -p singlefs-verification --release --features gpu
# admission: always 每次调都在这一刻的树上跑全部项或点名的项；测试函数那一段按测试函数单元键查门禁那一份判定库（文件头「按条复用」），崩溃点重放部分与放量部分由背后两套驱动各自的判定库按块定
# run-condition: command bash cargo python3
# run-condition: check source "$PREFLIGHT_SCRIPT_DIRECTORY/crash-amplification-lib.sh" && crash_amplification_run_condition_holds "$PREFLIGHT_SCRIPT_DIRECTORY/../.." :: 修好设备，或改配置：多机配置 multi-host.env 开了 GPU 而一份卡都没写、点名的卡与双机开关对不上；单跑 bash -c 'source scripts/crash/crash-amplification-lib.sh && crash_amplification_run_condition_holds' 看是哪一处
# 多机：这一份不自己分台、不自己连第二台，测试函数那一段、崩溃点重放部分与放量部分都在起它的那一台跑；机器、内存上限与分片都归 task-server
#   （records/2026-10-30-编译绕过内存总上限.md 收尾批；崩溃点重放部分两台分着判的，另经 task-server 按分片协议交）。
# 崩溃点重放的入口（判决 research/prompts/m3-crash-enumeration-merge-r3-main-verification.md 第三节 J2「一张表、一个入口、一道门禁」；
# 只有一种跑法：不带参数跑全部项，缩小范围用 --item，见 .claude/rules/verification.md）。登记表是合表 .claude/gate.d/crash-point-replay-items.tsv（写法在它的表头，
# 一项一条流），读法与核法在 scripts/crash/lib_crash_point_replay_items.py。背后照调两套驱动：崩溃点重放那一套（门禁 crash-point-replay：逐条用例照块判定库复用）
# 与放量分项驱动（scripts/crash/crash-amplification.sh）；引擎合并之后由引擎那一路换成一套。
#
#   crash-point-replay.sh                       全部项（连同整趟才跑的那几类 whole-run:<类>）：测试函数那一段照测试函数判定库复用、崩溃点重放部分照块判定库复用、放量部分照判定库复用
#   crash-point-replay.sh --item <项> […]       只跑点名的项（给几次取并集；整趟才跑的那几类也是项，写 whole-run:<类>）；缩小范围就用它（--list-items 列可点的项）
#   crash-point-replay.sh [--item <项>]… --crash-expand-up-to <n>
#                                               缩崩溃域（用户 2026-11-05 弹窗定：指定范围的参数允许缩域）：把崩溃放量与崩溃枚举用例的展开深度缩到不超过 n，
#                                               把 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=<n> 交给测试函数、崩溃点重放、放量三段（引擎只往小缩，大于引擎里放量的展开上限常量的不放宽）；
#                                               值不是非负整数、后面没有值都退 2、什么都不做；环境里已设了这个变量的当作给了同一个值，两样都给取参数；
#                                               环境里设了 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO（判器的展开深度）同样要是非负整数。缩过域的那一趟 ITEM、TOTAL 行末尾接
#                                               expand_up_to=<n>（只设了判器那个的接 judge_expand_up_to=<n>），成功句写缩过域、不算门禁通过
#   crash-point-replay.sh --list-items          逐行打「项名<TAB>测试函数 …（∪ 崩溃点重放 …）（∪ 放量 …）」，末尾是 whole-run:<类> 几行，只读合表，不编、不跑
#   crash-point-replay.sh --list-items --with-keys [--item <项>]…
#                                               同上，再按项各打一行 ITEM_PLAN item=<项名> to_judge=<要现判的件数> reusable=<能复用的件数>（件的算法见下「输出」），
#                                               紧接着按项各打一行 ITEM_DOMAIN item=<项名> states_to_judge=<这一项要现判的崩溃状态数，估不出写 unknown> source=<来源>
#                                               （态数的估法与来源的写法见 lib_crash_point_replay_items.py 文件头「指定范围的崩溃域上限」，每项另起读库程序 item 子命令读块账，
#                                               读不到的改按上一趟真跑的态数估；上限判的就是这个数），
#                                               末尾再打几行「键<TAB>…」：崩溃判定库里录下的块有几块键对不上（只在别的判法单元指纹下判过、
#                                               一种判器的结论都没有）、几块键相同，与两种判器这一份的判法单元指纹。可与 --item 合用，只列、只算点名的项
#                                               （不带 --with-keys 的 --list-items 不与 --item 合用，退 2）。起只读读库程序
#                                               （cargo run … --bin crash_library_reader -- keys：只读开库、不写），它的输出写法在
#                                               crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」那一段，自证 list-items-with-keys 格
#                                               把那一段原样喂进来钉读法；读库程序退非 0 或输出认不出照实报、退 1。
#                                               读库程序与真跑编同一棵树：真跑是 cargo test --release -p singlefs-verification [--features gpu]，验证包 [dev-dependencies]
#                                               里带特性的依赖（今天是 singlefs-core 的 lookup-work-counters）在 cargo test 的树里打开、cargo run 的树里没有，判法单元指纹跟着变，
#                                               所以读库程序那一趟 cargo run 带上真跑那一套（--release、gpu 开着加 gpu、再加这些依赖的 <依赖>/<特性>，keys_reader_features 现读清单）；
#                                               自证 keys-reader-same-build 格钉它
#   crash-point-replay.sh --selftest            假 cargo、假的两套驱动、假的准入模块走一遍各格，末尾一轮从下面「弄坏开关」那张表现读开关、逐个打开
#   别的参数一律当认不出的参数退 2（用法里只写正确用法）。
#   引擎配了 GPU 只由 GPU 判、没配只由 CPU 判，同一批状态一趟只判一次；GPU 判与 CPU 判的对拍是另立的测试，不在这里跑（用户 2026-10-11 定）。
#   不带 --item 的跑法（不指定范围）只在提交时或用户要求时由主 agent 交，崩溃状态数不设上限；带 --item 的（指定范围）主 agent 随时可以交（项目 settings 里注册的钩子判），
#   但没缩域时开跑之前先估这一趟点名的项要现判多少崩溃状态（只数键对不上、要现判的；估法与记录来源在 scripts/crash/lib_crash_point_replay_items.py 文件头「指定范围的崩溃域上限」），
#   超过 200000 态（常量 SCOPED_CRASH_STATES_LIMIT，只登记在那份库里，放量入口共用）或估不出就拒：不编、不跑，退 79，说明里给两条出路——带 --crash-expand-up-to <n> 缩域再交
#   （算得出就写出能压到上限以内的 n；缩过域的那一趟成功句写缩过域、不算门禁通过），或留给提交时不指定范围的那一趟。--crash-expand-up-to 的值不小于引擎展开上限常量的不算缩域，照判。
#   经 task-server 按件分片的跑法，这一判在计划那一步做，各片与合并不再判；放量部分起放量入口时交 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1，放量入口不再判一遍。
#   哪一种都经 task-server/task submit -- bash scripts/crash/crash-point-replay.sh … 交（内存上限、机器与显卡由服务定，命令里不写）。
#
# 三段（.claude/rules/verification.md）：
#   测试函数那一段 = 每项的测试目标里除登记成 crash-case: 的用例与放量分项之外的全部测试函数。项的测试目标不在合表里另登记：lib_crash_point_replay_items.py list 的第四段由这一项
#     点名的崩溃点重放部分与放量部分的测试目标现推；整趟才跑的那几类（WHOLE_RUN_GROUPS）的测试目标写在这份文件里。按测试函数单元键逐条复用（文件头「按条复用」），
#     登记的用例与放量分项经 test-plan 的 --exclude-registered 排掉、交下面两段；一个测试目标里的测试函数全登记了的，这一段对它没有可判的，不编不起也不记一笔。
#     按段登记里域装不进 u64 的段（驱动行 registered_segment_domains=<段号>:slice）引擎分片枚举：不指定范围那一趟判完全部片，SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES（没设取 2^32）只当一趟判几个态的分批：
#     三段都不设它，从调用方环境继承（片长进键；被打断之后重跑同一条命令，从 prune-server 里同一份判定库的续点接着判，库里还没有续点的头一趟从段内序号 0 起）；
#   崩溃点重放部分 = 第二列的 crash-case: 交门禁 crash-point-replay --item，键相同的块复用、对不上的现判；
#   放量部分 = 第三列的放量分项交 crash-amplification.sh，展开深度只有一处（引擎里放量的展开上限常量），这里不传；判定在 prune-server 里那一份判定库（crash-amplification-lib.sh 文件头
#     「判定库」；.claude/rules/verification.md 崩溃放量那一行），键相同、判过的块照库复用，被打断之后重跑同一条命令就是续跑
#     （test-script-discipline.md「判定可复用：按输入存，按前缀剪」）；ITEM 行与成功句照放量入口的 CRASH_AMPLIFICATION_ITEM 行报现判几态、复用几态。
#   计划（replay_plan）、复用、入库、判红在两种跑法里是同一段代码。项的取法：不带 --item 是全部项连同整趟才跑的那几类（WHOLE_RUN_GROUPS：小输入的 GPU 与 CPU 对拍、
#     剪枝自证、钉回来的红例），带 --item 只是点名的项。三段都照键复用：键相同、库里不红的不重跑，单元键对不上的（这一次改到的）与红过的才现判。
#   崩溃点重放部分里登记行第三列写了 red-blocks=judged-by-test 的用例，判法在那道门禁，这里只收它的退出码：按 cargo test 的退出码判，块那一行只核数，库里的红块不判。
#   三段各跑各的、各判各的：一段红了另两段照旧起（一项的结论是几段的合：有一段红就 red）。
# 按条复用（用户 2026-10-29 定：跑过的要入库、照键复用，红过的要复判）：测试函数那一段在两种跑法里是同一段：
#   每条测试函数各是一条判定，种类、请求、跳过名单与判绿名单只在 scripts/gate/admission.py 的 test-functions-request、test-plan、plan-skip-names、plan-green-names、
#   verdict-record（它文件头「按条复用的共用段」），与门禁 crash-point-replay 测试函数几格同一类判定库
#   （<git common-dir>/singlefs-unit-verdicts/crash-point-replay.sh:test-functions/）：这里写进的门禁照键复用，门禁写进的这里照键复用。
#   test-functions-request 在交给测试函数 cargo 的同一份环境里起（它照那一份环境给改判定域的变量打 variable: 那几项：按段登记文件的内容、片长这一类，
#   admission.py 文件头「判定请求」「单元键」；门禁的 cargo 继承门禁自己的环境、它在同一份里起，两处取法一致）。
#   开跑前 test-plan 算 验证包（这一趟的 --features 逐个交 --feature）每条测试函数的键、查库；每个测试目标
#   键相同的经 cargo test -- --exact --skip <名> 跳过（plan-skip-names --target-root tests/<目标>.rs），一条都不用现判就不编不起 cargo；跑完照
#   lib_libtest_result_lines.py recordable（开了 GPU 不带 --cpu-judging-recordable，带 --test-threads 1 的带 --serial）逐条认结局，判绿的经 plan-green-names
#   与 verdict-record 照键写回；这一趟真跑到而判红的（结果行 FAILED）经 verdict-record --red-names 照计划里的键记红、撤掉同一把键原有的绿
#   （记红、撤绿、下趟现判，admission.py 文件头「判定库」）：键相同而库里记着红的下一趟照现判，判绿之后再盖回绿。
#   判定库的名字是被判的根（这份脚本所在的仓，自证里是 CRASH_POINT_REPLAY_ROOT）的 git common-dir 下的种类目录（只当名字，键里不带哪棵仓，
#   命名空间只在临时服务底下给；admission.py 文件头「判定库」）；成功那一行点名库在哪。
#   逐条打一行「· <测试函数>：绿|红，现判|复用|强制现判（…）」（scripts/gate/lib_reuse_checks.py 的 ITEM_LINE），末了一行「· 按条复用（判定库 <种类>）：现判 N、复用 M、拒算 R……」。
#   被判的根不是一棵 git 工作树的顶（判定库没有名字）：测试函数一个测试目标都不编不起，整趟退 2、点名那棵树（弄坏开关 test-functions-without-library-tolerated 下照改前全部现判、不入库）。
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
#   分卡与清场归 task-server（records/2026-10-30-编译绕过内存总上限.md 收尾批：服务照多机配置的显卡先后挑一张、经 scripts/crash/gpu-quiesce-run.sh 清场再起这一整条，
#   挑的卡经环境 GPU_CARDS_BY_PRIORITY=<HOSTS 里的地址>:<卡号>:<额度> 交下来）：这里不读多机配置里的显卡先后、不自己清场，测试函数每一趟交的就是服务给的那一个值；
#   开了 GPU 而环境里没有 GPU_CARDS_BY_PRIORITY（不是经服务起的）退 78、一项都不起（自证 gpu-card-missing）；走 GPU 的计算类不送（引擎开了 GPU 就三类全走 GPU，
#   crash-amplification-lib.sh 文件头「配置」）。
#   开了 GPU 时单机路径的每一趟测试函数测试参数带 `-- --nocapture --test-threads 1`（GPU 对拍的测试各占卡的显存，不并行起）；没开的不加。
#   例外是 GPU_TEST_THREADS_RELEASED_TARGETS 里的测试目标（今天是 crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state；用户 2026-10-28 定案「放开并按显存排队」）：
#   它的测试按显存额度排队（排队在 crates 里），这里带 `--test-threads <登记的重活默认并行度>`（scripts/hosts/heavy-threads.sh 的 heavy_thread_default，SINGLEFS_THREAD_CAP 设了取它）；
#   自证 gpu-agreement-test-threads-released 一格钉它，弄坏开关 gpu-agreement-test-threads-one 下照改前带 1。
#   准入模块 admission.py 拼的 crash-case 命令里特性带 gpu 的也带。
#   还没带的：崩溃放量单机那几趟（crash-amplification.sh 经共用库 crash-amplification-lib.sh 拼的 cargo test 参数，里面没有这个参数）；改成带时同时改 .claude/rules/verification.md里 GPU 测试那一条。
#   这个参数只管同时起几个测试，不管一个测试里 CPU 判器自己开的线程：GPU 对拍的测试读 SINGLEFS_CRASH_JUDGE_CPU_THREADS（服务交下来的线程变量之一），
#   没设取本机可用并行度（available_parallelism）。
#   没开不带 --features，挂 gpu 特性的对拍编进来 0 条、报本次未跑。放量部分的 GPU 与清场由 crash-amplification.sh 自己照同一份配置办。
#   崩溃点重放部分里经引擎的 crash-case 行（.claude/gate.d/stage-inputs.tsv 里登记了 features= 的，今天是随机历史两项）由准入模块 admission.py 的 crash-case-command
#   带 --features 起、开 GPU 时同样加 gpu，不在这里拼（写法见 admission.py 文件头「崩溃枚举用例」）。
#
# 往引擎送的变量（测试函数、崩溃点重放部分与放量部分都从这一趟的环境继承）：
#   SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=<仓根>/.claude/gate.d/crash-point-replay-segments.tsv（按段登记文件的绝对路径，判决 J10、M4 第 15 条）：
#     每一趟都送；开跑前先经 scripts/crash/lib_crash_point_replay_items.py segments 核它（核不过退 2、什么都不起；写法与核法在那份库文件头「按段登记」）。
#   SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=<分批一趟判几个态>：片长只当分批，不指定范围那一趟判完全部片；三段都不设，从调用方环境继承（设了照用，空串也照原样交，引擎拒）。
#   SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=<n>：只在给了 --crash-expand-up-to 或环境里已设时有；三段都收到同一个值（测试函数那几趟 cargo 清掉调用方环境里别的 SINGLEFS_CRASH_AMPLIFICATION_*，
#     只把这一个原样带上；崩溃点重放那一道与放量入口从这一趟的环境继承）；它是改判定域的变量，进测试函数的单元键，所以缩过域的判定与全量的各存各的。
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
#     崩溃点重放部分每跑完一趟，在日志旁写 <日志>.scope（enumeration.<项>.log.scope，分片时 enumeration.<项>.<用例>.log.scope）：expand_up_to=<full|n>、judge_expand_up_to=<n>（判器展开深度缩过才有）、
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
#   （task-server/task_server_lib.py 文件头「分片」），哪台空出来哪台接下一片。带 --no-shard 的不分：计划写一片、那一片照不分片整趟跑、合并什么都不做。
#   不经服务起的没有这几个变量，照不分片跑。件是计划（replay_plan）里的每一条崩溃枚举用例与每一个放量分项，一件一片、超过上限时相邻的合片；
#   只有测试函数或本次未跑的项（整趟才跑的那几类）各算一件。这一份照环境里的 TASK_SERVER_SHARD_PHASE 只跑其中一步：
#   plan   往计划目录写 plan.tsv（整趟的计划）、shard-plan.tsv（片号加计划行，每片带上它的件所在项的测试函数那一段）与 plan.json，退 0；不起测试函数与两套驱动。
#   shard  照 TASK_SERVER_SHARD=<片号>/<片数> 只跑 shard-plan.tsv 里自己那一片的行，判法同不分片：结论写进产物目录（TASK_SERVER_SHARD_OUTPUT_DIRECTORY 下的 parts.tsv、
#          crash-flow-records.tsv，各份 .problems.log，末了 shard-finished）。判红（退 1）或本次未跑（退 77）这一片退 0：判红不影响后面的片（用户定），结论交合并那一步判、
#          整件的退出码在合并；退 78 与别的非 0 照原样退（服务照那一片的结局收尾）。
#   merge  不起测试函数与两套驱动，照各片存下的 parts.tsv 判，ITEM、TOTAL、上次红的清单与退出码同不分片的那一趟（同一个测试目标几片都跑了的测试函数行只算一次，有红取红）。
#          有一片的产物不全（产物表里没有那一片、或缺 shard-finished）退 1。
#   读完这几个变量就从环境里清掉（TASK_SERVER_SHARD_*），不传给下层的门禁与放量入口。TASK_SERVER_SHARD_PHASE 是别的值、片号对不上计划、计划不全退 2。
# 退出码：
#   0   跑到的项都不红、派出收回相等（有项本次未跑也是 0，成功句逐项点名）
#   1   有红、或派出收回对不上
#   2   用法错：参数认不得（删掉的选项一样当认不得，用法里只写正确用法）、--item 后面缺项名或项名认不得（列出可点名的项）、--list-items 或 --selftest 夹了别的参数、
#       合表核不过、按段登记文件核不过、平常跑时带了只供自证的变量（见「只供自证」）
#   75  prune-server 不可用：按条复用的 test-plan 或 verdict-record 退 75（判定库连不上；文件头「按条复用」），出路 python3 prune-server/control check、报给主 agent
#   77  派出的项全是本次未跑，也没有红
#   79  指定范围的崩溃域超上限：带了 --item、没缩域，点名的项这一趟要现判约多少崩溃状态超过 200000 态，或估不出来（一项都没编、没起）；出路：带 --crash-expand-up-to <n> 缩域，或留给提交时不指定范围的那一趟
#   78  运行条件不满足：preflight 判的（文件头两条 run-condition），或背后有一段退 78（显卡、清场用不上，给定状态表的判器借不到显存，
#       放量入口退 78 而这一项没打 ITEM 行）而这一趟没有红；或开了 GPU 而环境里没有 task-server 交的 GPU_CARDS_BY_PRIORITY（一项都没起）；
#       或起任何 cargo 之前 scripts/hosts/rocksdb-prebuilt.py entry 判红（这台没装预编库、此刻进程环境的取值缺了或对不上；一项都没起，出路是开新会话
#       或命令前 eval "$(python3 scripts/hosts/rocksdb-prebuilt.py env)"）。--list-items --with-keys 起读库程序之前同样判，判红退 1
# 编译目录（用户 2026-10-23 定案「自己用完自己删」，不跨趟复用）：建好这一趟的日志目录之后，调用方没设 CARGO_TARGET_DIR 时在里面建 target、export 给测试函数、
#   崩溃点重放那一道与放量入口（它们看到设了就不另建）；正常结束、红了、被 TERM / INT 停（等手里那一步退出）都在 EXIT 上删
#   （scripts/hosts/run-build-directory.sh；删之前核本机没有进程开着里面的文件，有就不删、报出进程、整趟退非 0）。调用方设了的照它、不删。
# 日志出两份（用户 2026-10-21：过万行的全量日志太长，一般只看错误警告）：测试函数、崩溃点重放部分、放量部分各自的日志都经 scripts/hosts/two-logs.sh 写，
#   <日志目录>/<段>.log 是全量，旁边同名的 <段>.problems.log 只收错误与警告（门禁的 ✗ 与 → 出路、⚠、本次未跑、汇总，cargo / rustc 的 error 与 warning:，libtest 的 FAILED 与 panicked，
#   Traceback，非 0 退出码），每行前带它在全量日志里的行号，末行写收了几行、全量几行；开跑时打日志目录，收尾逐份列出 .problems.log，红的原因里点名这一份。
# 弄坏开关 CRASH_POINT_REPLAY_BREAK=<项>（几个用逗号连；只在自证起的子进程里认）。写法同 scripts/crash/crash-amplification.sh 文件头「弄坏开关」一段：
#   一行「#   CRASH_POINT_REPLAY_BREAK=<开关名>  <弄坏哪一处>」，下一行起以「#       」开头写点名的格（、隔开；括号里是失败行的句首，开关轮不读）。
#   --selftest 末尾那一轮（开关轮，crash-amplification-lib.sh 的 crash_amplification_switch_wheel）从这张表现读，表里的开关与代码里 replay_break_is 认的要相等：
#   CRASH_POINT_REPLAY_BREAK=accept-unknown-item  认不得的项名悄悄丢掉、照跑
#       unknown-item（「项名写错应当退 2、列出可点名的项、什么都不起」）
#   CRASH_POINT_REPLAY_BREAK=list-items-builds  --list-items 先起一趟 cargo build 再列
#       list-items（「--list-items 应当退 0、逐项列出、不起 cargo 与驱动」）、list-items-with-keys（「--list-items --with-keys 应当照读库程序文件头「输出」那一段的写法报键对不上的块数、不起 cargo」）
#   CRASH_POINT_REPLAY_BREAK=test-functions-reuse-not-counted  测试函数每个测试目标的复用数写死 none（改前的样子）
#       test-functions-reuse-counted（「测试函数日志里有复用计数行时 ITEM 行应当报同一个数……」）
#   CRASH_POINT_REPLAY_BREAK=gpu-card-missing-unrefused  开了 GPU 而环境里没有 task-server 交的 GPU_CARDS_BY_PRIORITY 也照跑（不退 78）
#       gpu-card-missing（「开了 GPU 而环境里没有 task-server 交的卡，应当退 78、一项都不起」）
#   CRASH_POINT_REPLAY_BREAK=gpu-cards-not-sent  开了 GPU 时测试函数那几趟不交显卡先后
#       gpu-test-functions（「开了 GPU 测试函数应当带 gpu 特性、不自己清场…交 task-server 给的那张卡」）
#   CRASH_POINT_REPLAY_BREAK=gpu-test-threads-unset  开了 GPU 时测试函数那几趟测试参数不带 --test-threads 1（改前的样子）
#       gpu-test-functions（「开了 GPU 测试函数应当带 gpu 特性、不自己清场、测试参数带 --test-threads 1」：调用记录里 gpu 那几趟（三项的测试目标加 WHOLE_RUN_GROUPS 里的测试目标，个数自证现数）的 -- --nocapture 后面不是 --test-threads 1；没开 GPU 的 whole 那一格数的是 -- --nocapture 那几趟，这个开关不动它）
#   CRASH_POINT_REPLAY_BREAK=gpu-agreement-test-threads-one  开了 GPU 时 GPU_TEST_THREADS_RELEASED_TARGETS 里的测试目标也照改前带 --test-threads 1
#       gpu-test-functions（「开了 GPU 测试函数应当带 gpu 特性、不自己清场、测试参数带 --test-threads 1（GPU_TEST_THREADS_RELEASED_TARGETS 那几个除外）」：带 1 的那几趟多了放开的那一个）、
#       gpu-agreement-test-threads-released（「✗ 自证 gpu-agreement-test-threads-released：」：放开的测试目标带了 --test-threads 1）
#   CRASH_POINT_REPLAY_BREAK=red-as-green  测试函数 cargo 退非 0、有 FAILED 也记绿
#       test-functions-red（「测试函数一个测试目标红了，那一项应当红、整趟退 1」）、test-functions-red-continues（「测试函数那一段红了，别的两段照旧起……」）、
#       run-target-red（「跑完运行目录里不剩它建的编译目录（测试函数红退 1）」：红记成绿、整趟退 0）
#   CRASH_POINT_REPLAY_BREAK=not-run-as-green  测试函数跑到 0 条、有 NOT_RUN_NO_GPU_ADAPTER 也记绿
#       test-functions-not-run（「测试函数跑到 0 条或没卡没跑的，应当记本次未跑」）、all-not-run（「派出的项全是本次未跑，应当退 77」）
#   CRASH_POINT_REPLAY_BREAK=skip-collection-count  放量部分收不回的分项不记红、派出收回不核
#       collection-count（「放量那一项收不回，应当红、派出收回对不上」）、
#       pipeline-entry-1-still-red（「✗ 自证 pipeline-entry-1-still-red：」）、pipeline-entry-78-not-run（「✗ 自证 pipeline-entry-78-not-run：」）、pipeline-entry-78-red-first（「✗ 自证 pipeline-entry-78-red-first：」：收不回的项一笔都不记）
#   CRASH_POINT_REPLAY_BREAK=enumeration-exit-ignored  崩溃点重放那一道的退出码不看，一律记绿
#       enumeration-red（「崩溃点重放那一道红了，那一项应当红」）、shard-default-merge-red（「判红的片应当退 0」：合并判不出红）
#   CRASH_POINT_REPLAY_BREAK=pipeline-red-ignored  放量那一项的 red= 与 verdict=red 不看
#       pipeline-red（「放量那一项红了，那一项应当红」）、last-run-list（「上次红的清单不对」：放量红的那一项没记成红）、
#       pipeline-entry-78-red-first（「✗ 自证 pipeline-entry-78-red-first：」：放量红的 alpha 记成绿）
#   CRASH_POINT_REPLAY_BREAK=honor-test-variables-outside-selftest  平常跑时也认只供自证的变量，不拒；关守卫项，整值比，只在它是 CRASH_POINT_REPLAY_BREAK 唯一一项时认，与别的项同写平常跑照拒并点名
#       test-variables-outside-selftest（「平常跑时带了只供自证的变量应当退 2」）
#   CRASH_POINT_REPLAY_BREAK=segments-registry-not-sent  不往引擎送按段登记文件的路径
#       segments-registry-sent（「测试函数与放量每一趟都应当带按段登记文件的绝对路径」）
#       test-functions-domain-environment-keyed（「✗ 自证 test-functions-domain-environment-keyed：」连带：它判的复用或域环境跟着这一处变）
#   CRASH_POINT_REPLAY_BREAK=segments-registry-unchecked  开跑前不核按段登记文件
#       segments-registry-refused（「按段登记文件核不过应当退 2、什么都不起」）
#   CRASH_POINT_REPLAY_BREAK=given-states-run-condition-as-red  测试函数日志里有给定状态表借不到显存那一行也照红记（不退 78）
#       test-functions-run-condition-not-met（「测试函数日志里有给定状态表借不到显存那一行……整趟退 78」）
#   CRASH_POINT_REPLAY_BREAK=service-threads-dropped  测试函数的 cargo 起之前清掉服务交下来的 CARGO_BUILD_JOBS、RUST_TEST_THREADS，吃 cargo 自己的缺省
#       test-functions-threads-from-service（「测试函数每一趟 cargo 都应当原样收到服务交的线程数」）
#   CRASH_POINT_REPLAY_BREAK=logs-single  每一段的日志只写一份全量的（改前的样子：tee，不经 two-logs.sh，没有 .problems.log）
#       two-logs-per-part（「测试函数红的那一段应当出两份日志……」）
#   CRASH_POINT_REPLAY_BREAK=pipeline-78-as-red  放量入口退 78 而没打 ITEM 行，照旧记红（收不回，改前的样子）
#       pipeline-entry-78-not-run（「放量入口退 78 而没打 ITEM 行，两项应当记本次未跑……」）、pipeline-entry-78-red-first（「✗ 自证 pipeline-entry-78-red-first：」：gamma 记了红，不是本次未跑）
#   CRASH_POINT_REPLAY_BREAK=source-changed-reason-dropped  放量分项驱动报判段跑的过程中源码变了（退 77、verdict=not-run）时，本次未跑的原因不点名源码变了（只写 exit=77）
#       pipeline-source-changed-not-run（「放量分项驱动报跑的过程中源码变了，那一项应当记本次未跑、不记红，原因点名源码变了……」）
#   CRASH_POINT_REPLAY_BREAK=flow-coverage-table-skipped  跑完不往这一趟的日志目录落覆盖表与记录（改前的样子）
#       flow-coverage-table（「跑完应当在这一趟的日志目录里落覆盖表 crash-flow-coverage.md 与记录 crash-flow-records.tsv…」）
#   CRASH_POINT_REPLAY_BREAK=rocksdb-entry-unchecked  起 cargo 之前不调 scripts/hosts/rocksdb-prebuilt.py entry（改前的样子）
#       rocksdb-entry-refused（「验证包依赖 rocksdb 而进程环境里没有预编库的取值，应当在起 cargo 之前退 78…」：往下走起了假 cargo）
#   CRASH_POINT_REPLAY_BREAK=run-target-kept  跑完不删这一趟自己建的编译目录（改前的样子：测试函数在仓根照仓根的 target 编、跨趟留着）
#       run-target-green（「跑完运行目录里不剩它建的编译目录（正常退 0）」）、run-target-red（「跑完运行目录里不剩它建的编译目录（测试函数红退 1）」）、
#       run-target-term（「跑完运行目录里不剩它建的编译目录（被 TERM 停）」）
#   CRASH_POINT_REPLAY_BREAK=shard-phase-ignored  不认 TASK_SERVER_SHARD_PHASE 这几个变量，计划、一片、合并都照不分片跑（文件头「经 task-server 按件分片」）
#       shard-default-plan（「不带参数应当默认分片」）、shard-default-one-part（「默认分片的一片应当只跑自己的件」）、shard-default-merge-red（「判红的片应当退 0」）、shard-default-merge-green（「全绿的默认分片，合并应当退 0」）、
#       shard-default-many-parts（「件数超过片数上限应当合成」）、shard-position-refused（「片号对不上计划或阶段认不得应当退 2」）、shard-merge-refuses-incomplete（「有一片的产物不全」）、shard-no-shard（「带 --no-shard 应当计划一片」）、shard-plan-lines（「分片计划行每片每项一行、条数之和等于不分片那一趟」）
#   CRASH_POINT_REPLAY_BREAK=shard-red-exit-kept  一片判红照原样退 1（别的片被服务收掉、合并起不来）
#       shard-default-merge-red（「判红的片应当退 0」）
#   CRASH_POINT_REPLAY_BREAK=no-shard-ignored  不认 --no-shard，照分片
#       shard-no-shard（「带 --no-shard 应当计划一片」）
#   CRASH_POINT_REPLAY_BREAK=merge-incomplete-accepted  合并不核各片的产物全不全（缺的片当没判）
#       shard-merge-refuses-incomplete（「有一片的产物不全」：退了别的码）
#   CRASH_POINT_REPLAY_BREAK=plan-shards-not-merged  件数超过服务的片数上限也一件一片（服务记 shard-plan-refused）
#       shard-default-many-parts（「件数超过片数上限应当合成」）
#   CRASH_POINT_REPLAY_BREAK=test-functions-skip-ignored  测试函数照算计划，键相同的也不经 --skip 跳过、照编照跑（改前的样子：每个测试目标整趟现判）
#       reuse-three-questions（「✗ 自证 reuse-three-questions：」：测试函数那一半第二趟照样全现判）、
#       test-functions-reuse-shared-with-the-gate（「✗ 自证 test-functions-reuse-shared-with-the-gate：」：剪枝那一趟没复用逐条写进的）、
#       red-rejudged（「✗ 自证 red-rejudged：」：上一趟判绿的那一条没复用）、
#       test-functions-domain-environment-keyed（「✗ 自证 test-functions-domain-environment-keyed：」连带：它判的复用或域环境跟着这一处变）、red-revokes-green（「✗ 自证 red-revokes-green：」连带：它判的复用或域环境跟着这一处变）
#       registered-excluded（「按项跑时登记成 crash-case: 的用例与放量分项应当经 --skip 排掉」：参数里没有 --skip）
#   CRASH_POINT_REPLAY_BREAK=test-functions-not-recorded  测试函数现判出的判定不经 verdict-record 写回（改前的样子：测试函数跑了不入库）
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
#   CRASH_POINT_REPLAY_BREAK=test-functions-plan-failure-not-red  按条复用失灵（test-plan 退非 0）只全部现判、不判红
#       test-functions-plan-failure-red（「✗ 自证 test-functions-plan-failure-red：」）
#   CRASH_POINT_REPLAY_BREAK=test-functions-plan-unavailable-as-failure  测试函数 test-plan 退 75 照按条复用失灵办（全部现判、判红、整趟退 1），不退 75（改前的样子）
#       test-functions-plan-unavailable（「✗ 自证 test-functions-plan-unavailable：」）
#   CRASH_POINT_REPLAY_BREAK=last-run-whole-overwrite  上次红的清单照改前整份覆盖（不点名的跑法不交这一趟真判过的项）
#       last-run-list（「✗ 自证 last-run-list：」：本次未跑的那一项上一份的红丢了）
#   CRASH_POINT_REPLAY_BREAK=red-not-recorded  测试函数现判红的测试函数不交 verdict-record --red-names（改前的样子：只记绿，判红不撤同一把键的绿）
#       red-revokes-green（「✗ 自证 red-revokes-green：」：同键现判那一趟判红之后，下一趟那一条照复用）
#   CRASH_POINT_REPLAY_BREAK=test-functions-without-library-tolerated  被判的根不是一棵 git 工作树的顶时照改前测试函数全部现判、不入库、往下跑（不退 2）
#       test-functions-outside-git-exits-2（「✗ 自证 test-functions-outside-git-exits-2：」：不是 git 工作树的副本那一趟没退 2、照编照跑）
#   CRASH_POINT_REPLAY_BREAK=test-functions-domain-environment-unkeyed  test-functions-request 带 ADMISSION_BREAK=domain-environment-unkeyed 起（改前的样子：改判定域的环境变量不进键）
#       test-functions-domain-environment-keyed（「✗ 自证 test-functions-domain-environment-keyed：」：按段登记文件改了，闭包读它的那一条照复用）
#   CRASH_POINT_REPLAY_BREAK=registered-not-excluded  不排登记成 crash-case: 的用例与放量分项（test-plan 不带 --exclude-registered），它们也按条复用地跑
#       registered-excluded（「按项跑时登记成 crash-case: 的用例与放量分项应当经 --skip 排掉……」）
#   CRASH_POINT_REPLAY_BREAK=whole-run-item-unnamed  --item 点不到整趟才跑的那几类（whole-run:<类> 当不认得的项；--list-items 与不带参数的跑法照旧）
#       whole-run-item（「--item whole-run:<类> 应当只跑那一类的测试目标」：退 2）
#   CRASH_POINT_REPLAY_BREAK=expand-up-to-not-passed  给了 --crash-expand-up-to（或环境里已设缩域变量）不把变量交给测试函数、崩溃点重放、放量三段（输出照写缩过域）
#       expand-up-to-passed（「给了 --crash-expand-up-to 6 时三段都应当收到变量、ITEM 与 TOTAL 行带 expand_up_to=6、成功句写缩过域」）
#   CRASH_POINT_REPLAY_BREAK=expand-up-to-unchecked  --crash-expand-up-to 与环境里缩域变量的值不核是不是非负整数
#       expand-up-to-refused（「缩域的值不是非负整数应当退 2、什么都不起」）
#   CRASH_POINT_REPLAY_BREAK=item-plan-reusable-zero  --list-items --with-keys 的 ITEM_PLAN 里 reusable 一律 0（键相同的测试函数不数）
#       item-plan-lines（「--list-items --with-keys 应当每项一行 ITEM_PLAN、条数照 test-plan 的计划数」）
#   CRASH_POINT_REPLAY_BREAK=item-plan-selection-ignored  --list-items --with-keys 带 --item 时照样给全部项各打一行
#       item-plan-lines
#   CRASH_POINT_REPLAY_BREAK=item-done-reused-silent  ITEM_DONE 不为全部复用的项（现判 0）打行
#       item-done-lines（「真跑应当每项一行 ITEM_DONE、件数与退出码照各段的记录」）
#   CRASH_POINT_REPLAY_BREAK=item-done-seconds-whole-run  ITEM_DONE 的 seconds 写整趟各项秒数之和，不是这一项自己的
#       item-done-lines
#   CRASH_POINT_REPLAY_BREAK=shard-plan-lines-merged  分片计划行每项只打一行（不分片号、各片的条数相加）
#       shard-plan-lines
#   CRASH_POINT_REPLAY_BREAK=keys-reader-build-differs  --with-keys 的读库程序那一趟 cargo run 不带真跑那一套特性
#       keys-reader-same-build（「同一棵树 --with-keys 算的键与真跑记库的键相同：读库程序带的特性要与真跑的相同」）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-unbounded  带 --item、没缩域也不判指定范围的崩溃域上限（改前的样子）
#       scoped-domain-over-limit（「估数超过 200000 态应当退 79、不编不跑」）、scoped-domain-unknown-refused（「估不出应当退 79」）、
#       scoped-domain-shrunk-record-skipped（「只有缩过域的记录算不出全域态数，应当退 79」）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-narrowed-refused  缩了域（--crash-expand-up-to 小于引擎常量）也照没缩判：缩域值不交给判法
#       scoped-domain-narrowed-passes（「缩过域的放行，不读库、不判上限」）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-reusable-counted  库里录下的块键都对得上、全部可复用的项也按上一趟全域真跑的态数算（上一趟有记录才算）
#       scoped-domain-reusable-passes（「点名项全部可复用的放行，不论上一趟的态数多大」）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-unknown-passed  估不出现判多少态的项按 0 态放行
#       scoped-domain-unknown-refused（「估不出应当退 79」）、scoped-domain-shrunk-record-skipped（「只有缩过域的记录算不出全域态数，应当退 79」）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-suggestion-dropped  拒绝说明里不写算得出的 n
#       scoped-domain-over-limit（「说明里应当写能压到上限以内的 --crash-expand-up-to 6」）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-shrunk-records-used  估上一趟全域态数时不看展开深度，取最新的一份记录（缩过域的那趟也当全域）
#       scoped-domain-shrunk-record-skipped（「只有缩过域的记录算不出全域态数，应当退 79」）
#   CRASH_POINT_REPLAY_BREAK=enumeration-scope-not-recorded  崩溃点重放部分跑完不在日志旁写 .scope（展开深度与用例日志目录）
#       scoped-domain-enumeration-records（「跑完应当在日志旁写 .scope，下一趟估上限时读得到这一项的崩溃点重放态数」）、item-domain-line
#   CRASH_POINT_REPLAY_BREAK=item-domain-line-dropped  --list-items --with-keys 不打 ITEM_DOMAIN 行
#       item-domain-line（「每个列出的项一行 ITEM_DOMAIN、态数照估法、点名了只打点名的项」）、scoped-domain-enumeration-records（「跑完应当在日志旁写 .scope……」：ITEM_DOMAIN 行没有）
#   CRASH_POINT_REPLAY_BREAK=scoped-domain-handshake-dropped  起放量入口时不交 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1（放量入口会再判一遍）
#       scoped-domain-checked-handshake（「放量部分起放量入口应当交已判过的标记」）
# 只供自证（自证起的子进程里才认；平常跑时有一个非空就在参数解析之前退 2 拒跑，只有 --selftest 本身不拒，--list-items 与认不得的选项也先看这个）：CRASH_POINT_REPLAY_BREAK=<项>（上面那张表）；
#   CRASH_POINT_REPLAY_ROOT=<造出来的小仓>：合表、stage-inputs.tsv、门禁 crash-point-replay、crash-amplification.sh 都从它取
#   （门禁与放量入口那两份是自证写的假的：只记一行参数、照环境变量打结果）；按条复用调的是真的 scripts/gate/admission.py（在小仓上算键、查库、记库）。
#   按条复用那几格另用一份带小工作区、做成 git 仓的小仓（自证里的 unit-root，PATH 最前面另放只答版本的假 rustc）；别的格的小仓也是带小工作区、提交过一次的 git 仓
#   （测试函数照真的 test-plan 算计划；那份小仓的测试文件是空的，没有可复用、可记的测试函数，测试函数逐个测试目标现判）；
#   CRASH_POINT_REPLAY_KEYS_READER_OUTPUT=<文件>：--list-items --with-keys 不起读库程序，拿这份文件的内容当它的输出（list-items-with-keys 格用）。
#   CRASH_POINT_REPLAY_MOST_SHARDS=<片数>：按件分片的片数上限换成它（shard-default-many-parts 格用：小仓的件数够不到服务的上限）。
#   「自证起的子进程」的标记：CRASH_POINT_REPLAY_SELFTEST_MARK=<文件>，那份文件在、内容是 SELFTEST_MARK_CONTENT 那一行；只有 run_selftest 设它。
#   这几个变量读完就从环境里清掉，不传给背后的驱动。自证里的 cargo 是 PATH 最前面的假 cargo（起之前核 PATH 上解到的就是它，不是就不起）；
#   多机配置指到临时目录里只写了假清场命令的一份（放量入口是假的，这一份不清场），状态与日志目录都在临时目录里。
# 自证末尾的开关轮并行度 CRASH_AMPLIFICATION_SELFTEST_JOBS，没给取 SINGLEFS_THREAD_CAP，再没给取登记的重活默认并行度（同 crash-amplification-lib.sh 文件头「弄坏开关轮」）。
#
# gate-similar: crash-amplification.sh 它是放量分项的登记表与全部项的循环；这里是合表的入口，放量部分每趟调它一次，崩溃点重放部分调门禁 crash-point-replay
# gate-similar: crash-point-replay.sh 它跑崩溃点重放测试函数与登记的崩溃枚举用例；这里照项调它的 --item，测试函数那一段只跑合表点名的部分的测试目标
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../../.claude/scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}

SELFTEST_MARK_CONTENT="crash-point-replay.sh --selftest 起的子进程"
WHOLE_RUN_GROUPS=(
  $'gpu-and-cpu-agree-on-small-inputs\tcrash_judge_gpu_agrees_with_the_cpu_judge_on_every_state crash_facts_gpu_verifier_agrees_with_the_cpu_reference gpu_unit_checks_agree_with_the_cpu_and_catch_a_flipped_byte gpu_judge_root_choice_tie_takes_the_earlier_root_like_the_cpu_checker gpu_judge_tree_identifier_watermark_of_the_newest_root_like_the_cpu_checker gpu_recovery_root_choice_refuses_root_record_pointer_head_fields_like_the_core crash_judge_gpu_secondary_reads_after_the_source_segment_is_discarded'
  $'pruning-self-proof\tcrash_amplification_two_flows_share_their_prefix crash_amplification_pipeline_records_checks_and_reuses'
  $'pinned-red-examples\t-'
)
# GPU 测试里放开 --test-threads 1 的测试目标（文件头「GPU」；.claude/rules/verification.mdGPU 那一条的例外，用户 2026-10-28 定案「放开并按显存排队」）：
#   它的测试按显存额度排队（排队在 crates 里），开了 GPU 时测试线程取登记的重活默认并行度；门禁 crash-point-replay 的 GPU_TEST_THREADS_RELEASED_TARGETS 与
#   scripts/gate/admission.py 的 GPU_TEST_THREADS_RELEASED_TARGETS 是同一份名单，三处一起改
GPU_TEST_THREADS_RELEASED_TARGETS=(crash_judge_gpu_agrees_with_the_cpu_judge_on_every_state)
PINNED_RED_EXAMPLES_NOT_RUN_REASON="崩溃点重放还没有钉表：判出的红要钉成一条用例，今天一条都没钉"

replay_script="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)/$(basename "${BASH_SOURCE[0]}")"
replay_script_directory="$(dirname "$replay_script")"
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
replay_test_variables=()
[[ -n "$replay_break" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_BREAK)
[[ -n "$replay_root_override" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_ROOT)
[[ -n "$replay_keys_reader_output" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_KEYS_READER_OUTPUT)
[[ -n "$replay_most_shards_override" ]] && replay_test_variables+=(CRASH_POINT_REPLAY_MOST_SHARDS)
unset CRASH_POINT_REPLAY_BREAK CRASH_POINT_REPLAY_ROOT CRASH_POINT_REPLAY_SELFTEST_MARK CRASH_POINT_REPLAY_KEYS_READER_OUTPUT CRASH_POINT_REPLAY_MOST_SHARDS
replay_break_is() { [[ ",$replay_break," == *",$1,"* ]]; }
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
  if [[ -n "$crash_expand_up_to_origin" ]] && ! replay_break_is expand-up-to-not-passed; then
    functions_environment+=("SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=$crash_expand_up_to")
  fi
  replay_break_is service-threads-dropped && functions_environment+=(-u CARGO_BUILD_JOBS -u RUST_TEST_THREADS)
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
report_the_item_domain() {
  local item_arguments=() name
  replay_break_is item-domain-line-dropped && return 0
  for name in "$@"; do item_arguments+=(--item "$name"); done
  (( ${#item_arguments[@]} )) || return 0
  test_functions_environment
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
  [[ -z "$crash_judge_expand_up_to" ]] || scope_text+="judge_expand_up_to=$crash_judge_expand_up_to"$'\n'
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
#   弄坏开关 item-plan-reusable-zero 下 reusable 一律 0，shard-plan-lines-merged 下分片计划行不分片、各片相加
emit_item_plan_lines() {
  python3 - "$1" "$2" "$3" "$4" "$(replay_break_is item-plan-reusable-zero && echo 1 || echo 0)" "$(replay_break_is shard-plan-lines-merged && echo 1 || echo 0)" <<'PYTHON'
import collections, sys
mode, plan_file, shard_file, plan_output, reusable_zero, lines_merged = sys.argv[1:7]
counts = collections.defaultdict(lambda: [0, 0])   # 测试目标 -> [要现判的（键对不上与拒算）, 复用的]
if plan_output != "-":
    for line in open(plan_output, encoding="utf-8", errors="replace"):
        columns = line.rstrip("\n").split("\t")
        if len(columns) >= 7 and columns[4].startswith("tests/") and columns[4].endswith(".rs"):
            target = columns[4][len("tests/"):-len(".rs")]
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
  echo "  ✗ 用法不对：$1。用法：$0 [--no-shard] [--item <项>]… [--crash-expand-up-to <n>] | --list-items [--with-keys [--item <项>]…] | --selftest" >&2
  echo "  → 怎么办：不带参数跑全部项；缩小范围用 --item <项>，项名照 --list-items 列的写（含整趟才跑的那几类 whole-run:<类>）；经服务交的默认分片，--no-shard 才不分" >&2
  exit 2
}

# 平常跑时带了只供自证的变量就拒，排在一切参数解析之前（认不得的选项、--list-items 也先看这个）；自证起的子进程与命令行第一个参数就是 --selftest 的除外（文件头「只供自证」）。
# 关守卫项 honor-test-variables-outside-selftest 整值比：只在它是 CRASH_POINT_REPLAY_BREAK 唯一一项时认；与别的项同写，平常跑照拒并点名 CRASH_POINT_REPLAY_BREAK。
if (( ! replay_in_selftest_child )) && [[ "${1:-}" != --selftest ]] && (( ${#replay_test_variables[@]} )) && ! replay_guard_off_honored; then
  echo "  ✗ 平常跑时带了只供自证的变量：${replay_test_variables[*]}；这几个变量只在自证里认（文件头「只供自证」）" >&2
  echo "  → 怎么办：去掉它们再跑（env -u <变量> …）；要证红就跑 $0 --selftest" >&2
  exit 2
fi

list_items=0; with_keys=0; selftest=0; named_items=(); no_shard=0
crash_expand_given=0; crash_expand_argument=""; crash_expand_up_to=""; crash_expand_up_to_origin=""; crash_judge_expand_up_to=""; narrowed_labels=""
while (($#)); do
  case "$1" in
    --no-shard) no_shard=1; shift ;;
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
#   判器的展开深度只认环境里的 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO。值不是非负整数退 2、什么都不做。弄坏开关 expand-up-to-unchecked 下不核，expand-up-to-not-passed 下不交变量
if ((crash_expand_given)); then
  crash_expand_up_to="$crash_expand_argument"; crash_expand_up_to_origin="--crash-expand-up-to"
elif [[ -n "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO+x}" ]]; then
  crash_expand_up_to="$SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO"; crash_expand_up_to_origin="环境变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO"
fi
[[ -z "${SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO+x}" ]] || crash_judge_expand_up_to="$SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO"
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
fi
[[ -z "$crash_judge_expand_up_to" ]] || narrowed_labels+="${narrowed_labels:+ }judge_expand_up_to=$crash_judge_expand_up_to"

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
# 复用计数行（文件头「测试函数的复用数怎么取」）：CRASH_POINT_REPLAY_PARALLEL_FINISHED 的 resumed_slices= 之和；一行都没有、有一行缺或写坏，取不出
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
  read -r passed failed not_run_lines condition_lines functions_reused <<<"${first_line:-0 0 0 0 unknown}"
  [[ "$functions_reused" =~ ^[0-9]+$ ]] || functions_reused=unknown
  replay_break_is test-functions-reuse-not-counted && functions_reused=none   # 弄坏开关：改前写死 none
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

# ── 按条复用（文件头「按条复用」）：种类、请求、跳过与记库的名单只在 scripts/gate/admission.py 的 test-functions-request、test-plan、plan-skip-names、
#    plan-green-names、verdict-record、plan-failure，与门禁 crash-point-replay 测试函数几格同一类判定库；这里只接它们的输入输出
functions_plan_state=""; functions_plan_note=""; functions_plan_kind=""; functions_plan_file=""; functions_plan_output=""   # state：planned 算成了、off 不读库全部现判、nolibrary 判定库没有名字、failed 按条复用失灵、unavailable 服务不可用
functions_plan_library=""                                         # 判定库的名字：被判的根的 git common-dir（库在 prune-server 里）
functions_recorded=0                                                                   # 这一趟判绿写回的条数（verdict-record 的 RECORD recorded= 之和）
functions_red_recorded=0                                                               # 这一趟判红记下、撤掉同一把键原有的绿的条数（RECORD red= 之和）
functions_target_judge=0; functions_target_reuse=0; functions_target_refused=0; functions_target_registered=0; functions_target_skip_arguments=(); functions_target_record_red=""

# plan_test_functions：算这一趟测试函数的计划（test-plan 落进 functions_plan_output，计划文件 functions_plan_file），设 functions_plan_state 与 functions_plan_note。
#   被判的根不是 git 工作树的顶（判定库没有名字）：nolibrary，run_test_functions 照它整趟退 2（弄坏开关 test-functions-without-library-tolerated 下照改前 off：全部现判、不入库）；
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
    functions_plan_note="按条复用失灵：scripts/gate/admission.py test-functions-request 没成（$(tr '\n' ' ' <<<"$request_output" | cut -c1-300)）"
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
  replay_break_is test-functions-kind-not-gate && functions_plan_kind="crash-point-replay.sh:test-functions"   # 弄坏开关：种类与门禁不同（记进、读另一份库）
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
    functions_plan_note="test-plan 退 $plan_exit（$(tail -n 2 "$functions_plan_output" "$functions_plan_output.err" 2>/dev/null | tr '\n' ' ' | cut -c1-300)）：测试函数全部现判、不入库（本次未跑复用）"
    return 0
  fi
  cat -- "$functions_plan_output.failure"
  functions_plan_state=failed
  functions_plan_note="$(grep -m 1 '✗' "$functions_plan_output.failure" | sed 's/^ *✗ *//' | cut -c1-400)（日志 $functions_plan_output.failure）"
}

# test_functions_target_counts <测试目标>：这个测试目标在计划里要现判、复用、拒算、排给别的两段的条数放进 functions_target_judge、functions_target_reuse、functions_target_refused、functions_target_registered（计划没算成都是 0）
test_functions_target_counts() {
  local counts
  functions_target_judge=0; functions_target_reuse=0; functions_target_refused=0; functions_target_registered=0
  [[ "$functions_plan_state" == planned ]] || return 0
  counts="$(awk -F'\t' -v root="tests/$1.rs" 'NF >= 7 && $5 == root { count[$2]++ } END { printf "%d %d %d %d\n", count["judge"], count["reuse"], count["refused"], count["registered"] }' "$functions_plan_output")"
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
  local names name
  functions_target_skip_arguments=()
  [[ "$functions_plan_state" == planned ]] || return 0
  replay_break_is test-functions-skip-ignored && return 0
  if ! names="$(python3 "$replay_admission_module" plan-skip-names "$functions_plan_output" --target-root "tests/$1.rs" 2>&1)"; then
    echo "  ! 按条复用：admission.py plan-skip-names 没成（$(tr '\n' ' ' <<<"$names" | cut -c1-300)），$1 一条都不跳、全部现判"
    return 0
  fi
  while IFS= read -r name; do [[ -n "$name" ]] && functions_target_skip_arguments+=(--skip "$name"); done <<<"$names"
  (( ${#functions_target_skip_arguments[@]} )) && functions_target_skip_arguments=(--exact "${functions_target_skip_arguments[@]}")
  return 0
}

# record_test_functions_target <测试目标> <日志或 -> <这一趟带没带 --test-threads 1：1|0>：逐条打这个测试目标里计划到的测试函数的结局
#   （「· <测试函数>：绿|红，现判|复用|强制现判（…）」），现判而判绿的经 plan-green-names 与 verdict-record 照键写回（日志写 - 是整目标复用、没起 cargo）。
#   记库那一趟 verdict-record 退非 0 而 plan-failure 判失灵的，原因放进 functions_target_record_red。计划没算成什么都不做。
#   弄坏开关 test-functions-not-recorded 下不写回，red-reused-as-green 下现判红的也照判绿写回
record_test_functions_target() {
  local target="$1" log="$2" serial="$3" prefix record_exit recorded red_recorded
  local -a serial_option=() cpu_option=() red_option=()
  functions_target_record_red=""
  [[ "$functions_plan_state" == planned ]] || return 0
  test_functions_target_counts "$target"
  (( functions_target_judge + functions_target_reuse + functions_target_refused > 0 )) || return 0   # 计划里没有这个测试目标的测试函数（test-plan 没列到它）：照跑过的算，什么都不记
  prefix="$run_directory/test-functions.$target"
  : >"$prefix.accounting"; : >"$prefix.green-names"; : >"$prefix.green-names.withheld"
  if [[ "$log" != - ]]; then
    [[ "$serial" == 1 ]] && serial_option=(--serial)
    (( gpu_switch )) || cpu_option=(--cpu-judging-recordable)   # 没开 GPU 时判段判在 CPU 的绿照记（同门禁，D13（验证路线）已定项 17）
    if python3 "$replay_script_directory/../gate/lib_libtest_result_lines.py" recordable "$log" ${serial_option[@]+"${serial_option[@]}"} ${cpu_option[@]+"${cpu_option[@]}"} \
         >"$prefix.accounting" 2>"$prefix.accounting.err" && grep -q '^RECORDABLE ' "$prefix.accounting"; then
      if ! python3 "$replay_admission_module" plan-green-names "$functions_plan_output" "$prefix.accounting" "$prefix.green-names.withheld" >"$prefix.green-names" 2>&1; then
        echo "  ! 按条复用：admission.py plan-green-names 没成（$(tr '\n' ' ' <"$prefix.green-names" | cut -c1-300)），$target 一条都不记"
        : >"$prefix.green-names"
      fi
    else
      echo "  ! 按条复用：lib_libtest_result_lines.py recordable 没成（$(tail -n 2 "$prefix.accounting" "$prefix.accounting.err" 2>/dev/null | tr '\n' ' ' | cut -c1-300)），$target 一条都不记"
      : >"$prefix.accounting"
    fi
  fi
  # 逐条打结局（结局取这一趟 libtest 的结果行，判法 lib_libtest_result_lines.py 的 result_lines）；现判红的另落一份名单（弄坏开关 red-reused-as-green 用）；计数追加进 test-functions.tsv
  python3 - "$functions_plan_output" "tests/$target.rs" "$log" "$prefix.green-names" "$prefix.green-names.withheld" "$prefix.red-names" "$run_directory/test-functions.tsv" \
    "$replay_script_directory/../gate/lib_libtest_result_lines.py" "$target" <<'PYTHON'
import importlib.util, sys
plan_output, target_root, log, green_file, withheld_file, red_file, counts_file, library_path, target = sys.argv[1:10]
specification = importlib.util.spec_from_file_location("lib_libtest_result_lines", library_path)
module = importlib.util.module_from_spec(specification)
specification.loader.exec_module(module)
def lines_of(path):
    try:
        return [line.rstrip("\n") for line in open(path, encoding="utf-8", errors="surrogateescape")]
    except OSError:
        return []
rows = [line.split("\t") for line in lines_of(plan_output)]
rows = [columns for columns in rows if len(columns) >= 7 and columns[4] == target_root and columns[1] in ("judge", "reuse", "refused")]
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
  # 判红的记红、撤掉同一把键原有的绿（文件头「按条复用」）：现判红的名单交 --red-names。弄坏开关 red-not-recorded 下不交（改前的样子），red-reused-as-green 下已并进判绿的名单
  red_option=()
  if [[ -s "$prefix.red-names" ]] && ! replay_break_is red-reused-as-green && ! replay_break_is red-not-recorded; then red_option=(--red-names "$prefix.red-names"); fi
  [[ -s "$prefix.green-names" ]] || (( ${#red_option[@]} )) || return 0
  replay_break_is test-functions-not-recorded && return 0   # 弄坏开关：不写回（改前的样子）
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
  echo "  ! 按条复用：$target 判绿写回 admission.py verdict-record 退 $record_exit（$(tail -n 2 "$prefix.record" | tr '\n' ' ' | cut -c1-300)），下一趟这几条照判"
  local record_failure_exit=0
  python3 "$replay_admission_module" plan-failure "crash-point-replay.sh 测试函数那一段（$target）" verdict-record "$record_exit" "$prefix.record" >"$prefix.record.failure" 2>&1 \
    || record_failure_exit=$?
  if (( record_failure_exit == 75 )) && ! replay_break_is test-functions-plan-unavailable-as-failure; then
    # 记库时 prune-server 不可用（文件头「按条复用」）：这一趟判出的判定记不下，整趟退 75
    cat -- "$prefix.record.failure"
    echo "  ✗ 测试函数那一段：$target 判完记库时 prune-server 不可用，这一趟的判定没记下；这一趟退 75"
    echo "     → 怎么办：python3 prune-server/control check 看服务在不在、起没起来，起不来报给主 agent；修好之后重交同一条命令"
    exit 75
  fi
  if (( record_failure_exit != 0 )) && ! replay_break_is test-functions-plan-failure-not-red; then
    cat -- "$prefix.record.failure"
    functions_target_record_red="$(grep -m 1 '✗' "$prefix.record.failure" | sed 's/^ *✗ *//' | cut -c1-400)（日志 $prefix.record.failure）"
  fi
  return 0
}

run_test_functions() { # <计划文件>：测试函数那一段（test-functions 的件），一个测试目标一趟；同一个测试目标几项都点名时跑一次、记给每一项
  local plan_file="$1" targets=() active_targets=() target target_arguments=() environment=() build_log build_exit run_log run_exit
  local variable started owners owner test_threads_arguments=() target_test_threads=() released_test_threads=1 target_units
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
    replay_break_is gpu-cards-not-sent || environment+=("GPU_CARDS_BY_PRIORITY=$gpu_cards_for_test_functions")
    # 带 gpu 特性的每一趟测试线程只开 1 个（GPU 对拍的测试各占卡的显存，并行起会互相抢）；清场归 task-server（文件头「GPU」），这里不另套清场包装；
    # 一个测试里 CPU 判器自己的线程不受这个参数管（读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取本机可用并行度）
    replay_break_is gpu-test-threads-unset || test_threads_arguments=(--test-threads 1)
    # GPU_TEST_THREADS_RELEASED_TARGETS 里的测试目标不带 --test-threads 1，测试线程取登记的重活默认并行度（它的测试按显存额度排队）；
    #   取不出登记值就照旧 1 个并打一句。弄坏开关 gpu-agreement-test-threads-one 下照改前带 --test-threads 1
    if ! released_test_threads="$(heavy_thread_default)"; then
      echo "  ! 取不出登记的重活默认并行度（scripts/hosts/heavy-threads.sh 的 heavy_thread_default），${GPU_TEST_THREADS_RELEASED_TARGETS[*]} 照旧带 --test-threads 1" >&2
      released_test_threads=1
    fi
  fi
  echo "── 测试函数那一段：${#targets[@]} 个测试目标，cargo test --release -p singlefs-verification${features:+ --features $features}（登记成崩溃枚举用例与放量分项的测试函数经 --skip 排掉，交下面两段$( (( gpu_switch )) && echo "；显卡是 task-server 交的 $gpu_cards_for_test_functions")）"
  # 按条复用（文件头「按条复用」）：先算计划；编得进的测试函数全有键相同、不红的判定的测试目标不编不跑，只编要现判的那几个
  local functions_build_arguments=() functions_owners_all="" functions_serial functions_plan_total
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
    target_test_threads=(${test_threads_arguments[@]+"${test_threads_arguments[@]}"})
    if (( gpu_switch )) && [[ " ${GPU_TEST_THREADS_RELEASED_TARGETS[*]} " == *" $target "* ]] && ! replay_break_is gpu-agreement-test-threads-one; then
      target_test_threads=(--test-threads "$released_test_threads")
    fi
    # 键相同的经 --exact --skip 跳过（文件头「按条复用」），接在测试线程参数后面
    functions_serial=0; [[ "${target_test_threads[*]-}" == "--test-threads 1" ]] && functions_serial=1
    target_test_threads+=(${functions_target_skip_arguments[@]+"${functions_target_skip_arguments[@]}"})
    run_logged "$run_log" "${environment[@]}" cargo test --release -p singlefs-verification ${features:+--features "$features"} --test "$target" -- --nocapture ${target_test_threads[@]+"${target_test_threads[@]}"}
    run_exit="$(cat "$run_log.exit" 2>/dev/null || echo 255)"
    judge_test_functions_log "$run_exit" "$run_log"
    record_test_functions_target "$target" "$run_log" "$functions_serial"
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
      # 放量入口退 78（运行条件不满足：清场回读没过、卡用不上、第二台连不上……）：这一项一行都没判，记本次未跑，不记红（文件头「一项怎么判」）
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
    "$narrowed_labels" "$(replay_break_is item-done-reused-silent && echo 1 || echo 0)" "$(replay_break_is item-done-seconds-whole-run && echo 1 || echo 0)" <<'PYTHON'
import collections, sys
plan_file, records_file, scope, scope_title, exit_file, skip_count, narrowed, reused_silent, seconds_whole_run = sys.argv[1:10]
order = []
for line in open(plan_file, encoding="utf-8"):
    item = line.rstrip("\n").split("\t")[0]
    if item and item not in order:
        order.append(item)
parts = collections.defaultdict(list)
for line in open(records_file, encoding="utf-8"):
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

run_selftest() {
  local work root calls cells=0 failures=0 exit_code service_problem
  # 按条复用那几格经 admission.py 写 prune-server：只在临时服务上跑（scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service），不是就退 2、一格都不跑
  if ! service_problem="$(python3 -c 'import sys; sys.path.insert(0, sys.argv[1]); import prune_server_client; print(prune_server_client.selftest_service_problem())' "$replay_script_directory/../prune-server" 2>&1)" || [[ -n "$service_problem" ]]; then
    echo "  ✗ crash-point-replay.sh 自证没跑：${service_problem:-读不了 scripts/prune-server/prune_server_client.py}"
    echo "     → 怎么办：照 scripts/prune-server/prune_server_client.py 文件头「命令行」--with-temporary-service 那一条，在临时服务底下起这一份自证（带 --server-binary 就不编）"
    return 2
  fi
  work="$(mktemp -d "${TMPDIR:-/tmp}/crash-point-replay-selftest.XXXXXX")"
  trap 'rm -rf -- "${work:?}"' EXIT
  root="$work/root"; calls="$work/calls"
  mkdir -p "$root/.claude/gate.d" "$root/scripts" "$root/scripts/crash" "$root/scripts/gate" "$root/crates/singlefs-verification/tests" "$work/bin" "$work/home"
  printf '%s\n' "$SELFTEST_MARK_CONTENT" >"$work/mark"
  # 造出来的小仓：三项（两段都有、只有崩溃点重放、只有放量），合表核法照真的那一份判
  printf '%s\n' "# 自证的合表" $'alpha\tcrash-case:alpha\talpha-amp\t# 两段都有' $'beta\tcrash-case:beta\t-\t# 只有崩溃点重放' \
    $'gamma\t-\tgamma-amp\t# 只有放量' >"$root/.claude/gate.d/crash-point-replay-items.tsv"
  printf '%s\n' "# 自证的登记表" $'crash-case:alpha\tcrates/\ttest=singlefs-verification:alpha:alpha_full\t# 自证' \
    $'crash-case:beta\tcrates/\ttest=singlefs-verification:beta:beta_full\t# 自证' >"$root/.claude/gate.d/stage-inputs.tsv"
  : >"$root/crates/singlefs-verification/tests/alpha.rs"; : >"$root/crates/singlefs-verification/tests/beta.rs"; : >"$root/crates/singlefs-verification/tests/gamma_amp.rs"
  # 按段登记文件：一行登记（alpha 那一项的流 alpha_flow 第 0 段，五列），segments-registry-refused 那一格临时换成核不过的一份、跑完换回
  printf '%s\n' "# 自证的按段登记" "alpha	alpha_flow	0	7	$(printf '0123456789abcdef%.0s' 1 2 3 4)" >"$root/.claude/gate.d/crash-point-replay-segments.tsv"
  cat >"$root/scripts/crash/crash-amplification.sh" <<'STUB'
#!/usr/bin/env bash
# 假的崩溃放量入口（自证用）：ITEMS 给合表核法读；起它时记一行参数，按 --item（没给就 ITEMS 全部）逐项打 CRASH_AMPLIFICATION_ITEM 行
ITEMS=(
  $'alpha-amp\talpha\talpha_amplified'
  $'gamma-amp\tgamma_amp\tgamma_amplified'
)
config_summary() { local file="${SINGLEFS_MULTI_HOST_CONFIG:-}"; if [[ -f "$file" ]]; then printf 'across=%s peer=%s cards=%s' "$(grep -c '^ENABLE_ACROSS_MACHINES=1' "$file")" "$(grep -c 'peer-selftest' "$file")" "$(grep -c '^GPU_CARDS_BY_PRIORITY' "$file")"; else printf 'none'; fi; }
printf 'pipeline-entry %s\n' "$*" >>"${STUB_CALLS:?}"
printf 'pipeline-env config=[%s]\n' "$(config_summary)" >>"$STUB_CALLS"
printf 'pipeline-env segments=[%s]\n' "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" >>"$STUB_CALLS"
printf 'pipeline-env slice-states=[%s]\n' "${SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES-unset}" >>"$STUB_CALLS"
printf 'pipeline-env expand=[%s]\n' "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO-unset}" >>"$STUB_CALLS"
printf 'pipeline-env scoped-checked=[%s]\n' "${SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED-unset}" >>"$STUB_CALLS"
if [[ -n "${STUB_AMP_PERSISTENT:-}" ]]; then   # 照引擎按键复用：判过的分项在 STUB_AMP_SERVICE 指的目录（演 prune-server 里那一份判定库）里留一份文件，再判就报复用、现判 0 态
  persistent_service="${STUB_AMP_SERVICE:?}"
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
  # STUB_AMP_SOURCE_CHANGED（pipeline-source-changed-not-run 格）：照真入口的样子，分项驱动判段跑的过程中源码变了——入口打「== 分项」那一行、驱动打 CRASH_AMPLIFICATION_NOT_RUN step=judge 那一行、退 77、ITEM 行 verdict=not-run
  if [[ " ${STUB_AMP_SOURCE_CHANGED:-} " == *" $item "* ]]; then
    echo "== 分项 $item（stub::run，展开 6）自证"
    echo "CRASH_AMPLIFICATION_NOT_RUN step=judge test=crash_amplification::run_from_environment reason=跑的过程中源码变了，判段没在录入段那份源码上判完（录入段之后 aaaa，本机那一趟收尾后 bbbb）"
    verdict=not-run; item_exit=77
  fi
  states=10; reused=4
  if [[ -n "${STUB_AMP_PERSISTENT:-}" ]]; then
    unit_key=""   # STUB_AMP_UNITS 设了：键另带这一项的单元文件（<STUB_AMP_UNITS>/<分项>）的内容摘要（复用三问那一格用）
    [[ -n "${STUB_AMP_UNITS:-}" ]] && unit_key=".$(sha256sum "$STUB_AMP_UNITS/$item" | cut -c1-64)"
    if [[ -e "$persistent_service/$item$unit_key.judged" ]]; then states=0; reused=10; else states=10; reused=0; mkdir -p -- "$persistent_service" && : >"$persistent_service/$item$unit_key.judged"; fi
  fi
  echo "CRASH_AMPLIFICATION_ITEM item=$item exit=$item_exit states=$states reused=$reused red=$red lines=1 expand_up_to=6 exhaustive=true verdict=$verdict seconds=0 ignored_points=0 states_skipped_by_ignore=0"
done
exit "${STUB_AMP_EXIT:-$exit_code}"
STUB
  cat >"$root/.claude/gate.d/crash-point-replay.sh" <<'STUB'
#!/usr/bin/env bash
# 假的门禁 crash-point-replay（自证用）：记一行参数；STUB_CRASH_POINT_REPLAY_REUSED 设了就给每条点名的用例打一行复用；退 STUB_CRASH_POINT_REPLAY_EXIT
config_summary() { local file="${SINGLEFS_MULTI_HOST_CONFIG:-}"; if [[ -f "$file" ]]; then printf 'across=%s peer=%s cards=%s' "$(grep -c '^ENABLE_ACROSS_MACHINES=1' "$file")" "$(grep -c 'peer-selftest' "$file")" "$(grep -c '^GPU_CARDS_BY_PRIORITY' "$file")"; else printf 'none'; fi; }
printf 'enumeration-gate %s\n' "$*" >>"${STUB_CALLS:?}"
printf 'enumeration-env config=[%s]\n' "$(config_summary)" >>"$STUB_CALLS"
printf 'enumeration-env expand=[%s]\n' "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO-unset}" >>"$STUB_CALLS"
if [[ -n "${STUB_CASE_LOG_DIRECTORY:-}" ]]; then   # 照真门禁：打每条用例的日志放哪个目录，各放一份带 CRASH_POINT_REPLAY_PARALLEL_FINISHED 的日志（scoped-domain-enumeration-records 格）
  mkdir -p -- "$STUB_CASE_LOG_DIRECTORY"
  echo "  · 这一趟每条用例的日志与发现日志放 ${STUB_CASE_LOG_DIRECTORY}/（log.<用例名> 与 log.<用例名>.findings.tsv，跑完不删）"
  previous=""
  for argument in "$@"; do
    [[ "$previous" == --item ]] && printf 'CRASH_POINT_REPLAY_PARALLEL_FINISHED states=%s slices=3 worker_threads=2 resumed_slices=0 freshly_run_slices=3\n' "${STUB_CASE_STATES:-1234}" >"$STUB_CASE_LOG_DIRECTORY/log.${argument#crash-case:}"
    previous="$argument"
  done
fi
previous=""
for argument in "$@"; do
  [[ "$previous" == --item && -n "${STUB_CRASH_POINT_REPLAY_REUSED:-}" ]] && echo "  · $argument 复用：全部块都有键相同的判定（自证）"
  previous="$argument"
done
[[ -n "${STUB_GATE_BETA_SLEEP:-}" && " $* " == *" crash-case:beta "* ]] && sleep "$STUB_GATE_BETA_SLEEP"
exit "${STUB_CRASH_POINT_REPLAY_EXIT:-0}"
STUB
  cat >"$root/scripts/gate/admission.py" <<'STUB'
# 假的准入模块（自证用）：记一行参数；这一份不该再调它（用例一级没有全绿标记可删），调了就在 whole 那一格判红
import os, sys
open(os.environ["STUB_CALLS"], "a").write("admission " + " ".join(sys.argv[1:]) + "\n")
STUB
  cat >"$work/bin/cargo" <<'STUB'
#!/usr/bin/env bash
# 假 cargo（自证用）：每次调记一行（参数、这一趟的显卡先后、是不是在清场包装之下、分片的段一趟判几个态、编译并行度、按段登记文件的路径）；按 --test 点名的目标打 libtest 的样子
# 只问版本的（scripts/gate/admission.py 算单元键时起 cargo -V）只答版本、不记调用
[[ "${1:-}" == -V || "${1:-}" == --version ]] && { echo "cargo 0.0.0-selftest"; exit 0; }
# 指定范围的崩溃域上限起的读库程序 item 子命令（scoped-domain-* 与 item-domain-line 那几格）：不记进调用记录（别的格数 cargo 调用不受影响），
#   按 STUB_ITEM_RECORDED、STUB_ITEM_TO_JUDGE、STUB_ITEM_REUSABLE（没给是 1、0、1：录下的块都对得上）答一行 CRASH_LIBRARY_ITEM，设点文件记进 STUB_ITEM_CALLS
if [[ " $* " == *" --bin crash_library_reader -- item "* ]]; then
  stub_item_name=""; previous=""
  for argument in "$@"; do [[ "$previous" == item ]] && { stub_item_name="$argument"; break; }; previous="$argument"; done
  [[ -z "${STUB_ITEM_CALLS:-}" ]] || printf 'item %s\n' "${*#* -- item }" >>"$STUB_ITEM_CALLS"
  printf 'CRASH_LIBRARY_ITEM\titem=%s\trecorded_blocks=%s\tto_judge=%s\treusable=%s\n' "$stub_item_name" "${STUB_ITEM_RECORDED:-1}" "${STUB_ITEM_TO_JUDGE:-0}" "${STUB_ITEM_REUSABLE:-1}"
  exit 0
fi
printf 'cargo %s cards=[%s] quiesced=[%s] slice-states=[%s] jobs=[%s] segments=[%s]\n' "$*" "${GPU_CARDS_BY_PRIORITY-unset}" "${GPU_QUIESCE_HOLDER:+yes}" \
  "${SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES-unset}" "${CARGO_BUILD_JOBS-unset}" "${SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY-unset}" >>"${STUB_CALLS:?}"
config_summary() { local file="${SINGLEFS_MULTI_HOST_CONFIG:-}"; if [[ -f "$file" ]]; then printf 'across=%s peer=%s cards=%s' "$(grep -c '^ENABLE_ACROSS_MACHINES=1' "$file")" "$(grep -c 'peer-selftest' "$file")" "$(grep -c '^GPU_CARDS_BY_PRIORITY' "$file")"; else printf 'none'; fi; }
printf 'cargo-env config=[%s]\n' "$(config_summary)" >>"$STUB_CALLS"
printf 'cargo-env expand=[%s]\n' "${SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO-unset}" >>"$STUB_CALLS"
# --list-items --with-keys 的读库程序（keys-reader-same-build 格）：问到 crash_library_reader 就把 STUB_KEYS_READER 那份文件当它的输出
if [[ " $* " == *" --bin crash_library_reader "* ]]; then cat -- "${STUB_KEYS_READER:?}"; exit 0; fi
# 编译目录那几格（run-target-*）：记下收到的 CARGO_TARGET_DIR、往里写一份产物；给了 STUB_SLEEP 每一趟睡这么多秒（被 TERM 停那一格）
if [[ -n "${STUB_TARGET_LOG:-}" ]]; then
  printf '%s\n' "${CARGO_TARGET_DIR-unset}" >>"$STUB_TARGET_LOG"
  [[ -z "${CARGO_TARGET_DIR:-}" ]] || { mkdir -p "$CARGO_TARGET_DIR/release" && echo built >"$CARGO_TARGET_DIR/release/artifact"; }
fi
[[ -z "${STUB_SLEEP:-}" ]] || sleep "$STUB_SLEEP"
[[ " $* " == *" --no-run "* ]] && exit "${STUB_BUILD_EXIT:-0}"
target=""; previous=""
for argument in "$@"; do [[ "$previous" == --test ]] && target="$argument"; previous="$argument"; done
# 按条复用那几格（STUB_UNIT_ROOT 设了、那份小仓里有 tests/<目标>.rs）：照那份测试文件里 #[test] 下一行的 fn 名打 libtest 的样子（先打 cargo 的 Running 行），
#   照 -- 之后的 --skip 跳过、照 STUB_RED_TESTS 判红，汇总行的数照实
unit_test_file="${STUB_UNIT_ROOT:-/nonexistent}/crates/singlefs-verification/tests/$target.rs"
if [[ -n "${STUB_UNIT_ROOT:-}" && -f "$unit_test_file" ]]; then
  skipped=(); ran=(); filtered=0; failed=0; previous=""
  for argument in "$@"; do [[ "$previous" == --skip ]] && skipped+=("$argument"); previous="$argument"; done
  while IFS= read -r name; do
    [[ -n "$name" ]] || continue
    if [[ " ${skipped[*]-} " == *" $name "* ]]; then filtered=$((filtered + 1)); else ran+=("$name"); fi
  done < <(awk 'previous ~ /#\[test\]/ && match($0, /fn [a-z0-9_]+/) { print substr($0, RSTART + 3, RLENGTH - 3) } { previous = $0 }' "$unit_test_file")
  printf '     Running tests/%s.rs (target/release/deps/%s-0000000000000000)\n\nrunning %d tests\n' "$target" "$target" "${#ran[@]}"
  for name in ${ran[@]+"${ran[@]}"}; do
    if [[ " ${STUB_RED_TESTS:-} " == *" $name "* ]]; then printf 'test %s ... FAILED\n' "$name"; failed=$((failed + 1)); else printf 'test %s ... ok\n' "$name"; fi
  done
  if (( failed )); then
    printf '\ntest result: FAILED. %d passed; %d failed; 0 ignored; 0 measured; %d filtered out\n' "$((${#ran[@]} - failed))" "$failed" "$filtered"
    exit 101
  fi
  printf '\ntest result: ok. %d passed; 0 failed; 0 ignored; 0 measured; %d filtered out\n' "${#ran[@]}" "$filtered"
  exit 0
fi
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
[[ -n "${STUB_FLOW_LINE:-}" && "$target" == alpha ]] && echo 'CRASH_FLOW_COVERAGE entry=selftest_entry step=UnitWrite segment=0 expansion=expanded'   # flow-coverage-table 那一格
printf 'running 2 tests\ntest first ... ok\ntest second ... ok\n\ntest result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out\n'
STUB
  chmod +x "$work/bin/cargo"
  if [[ "$(PATH="$work/bin:$PATH" type -P cargo)" != "$work/bin/cargo" ]]; then
    echo "  ✗ 自证 setup：PATH 最前面放了假 cargo，解到的却不是它（$(PATH="$work/bin:$PATH" type -P cargo)），不起子进程（起了就是真 cargo）" >&2
    echo "  → 怎么办：看 $work/bin/cargo 可不可执行、TMPDIR 是不是挂成了 noexec" >&2
    exit 1
  fi
  # 别的格的小仓 $root 也做成 git 工作树的顶（文件头「只供自证」）：测试函数要被判的根是 git 工作树的顶才有判定库的名字，不是就整趟退 2（plan_test_functions 的 nolibrary）。
  #   加一个只有 singlefs-verification 一个包的小工作区（真的 admission.py test-plan 读它；测试文件是空的，计划里一条测试函数都没有），git init 提交一次；
  #   包清单留一份底稿，rocksdb-entry-refused 那一格临时换掉、跑完照底稿换回
  printf '%s\n' '[workspace]' 'members = ["crates/singlefs-verification"]' 'resolver = "2"' >"$root/Cargo.toml"
  printf '%s\n' 'version = 4' '' '[[package]]' 'name = "singlefs-verification"' 'version = "0.0.0"' >"$root/Cargo.lock"
  printf '%s\n' '[package]' 'name = "singlefs-verification"' 'version = "0.0.0"' 'edition = "2021"' '' '[features]' 'gpu = []' >"$root/crates/singlefs-verification/Cargo.toml"
  cp -- "$root/crates/singlefs-verification/Cargo.toml" "$work/root-crate-manifest.toml"
  mkdir -p "$root/crates/singlefs-verification/src"
  printf '%s\n' 'pub fn root_helper() -> u32 {' '    1' '}' >"$root/crates/singlefs-verification/src/lib.rs"
  # 引擎放量展开上限常量（指定范围的上限判法读它认哪份记录是全域：lib_crash_point_replay_items.py 的 engine_expand_constant）；不是 lib.rs 的模块，不进任何测试函数的键
  printf '%s\n' 'pub const EXPAND_UP_TO_WRITES_PER_SEGMENT_OF_THE_AMPLIFICATION: usize = 30;' >"$root/crates/singlefs-verification/src/crash_amplification.rs"
  if ! { git -C "$root" init -q -b master && git -C "$root" add -A && git -C "$root" -c core.hooksPath=/dev/null -c user.name=selftest -c user.email=selftest@invalid commit -qm root; } >"$work/root-git.log" 2>&1; then
    echo "  ✗ 自证 setup：小仓 $root 做不成 git 仓（$(tr '\n' ' ' <"$work/root-git.log" | cut -c1-300)）" >&2
    echo "  → 怎么办：看 git 在不在 PATH 上、TMPDIR 可不可写" >&2
    exit 1
  fi
  # 按条复用那几格（文件头「按条复用」：reuse-three-questions 的测试函数那一半、test-functions-reuse-shared-with-the-gate、red-rejudged、test-functions-plan-failure-red）的小仓 unit-root：
  #   照 $root 拷一份（不带它的 .git：unit-root 自己另做一份 git 仓，判定库的名字与 $root 的不同），加一个只有 singlefs-verification 一个包的小工作区（三个测试目标：alpha 两条、beta 一条、gamma_amp 一条，alpha_reads_the_helper 引库里的 alpha_helper），
  #   git init 提交一次（判定库在它的 git common-dir 下），放一个空的 .unit-verdict-sample/（admission.py 文件头「复用哨兵」：样本根不抽哨兵，现判与复用的条数钉死）；
  #   真的 scripts/gate/admission.py 在它上面算键、查库、记库。unit-bin 里放只答版本的假 rustc（算键时起 rustc -vV），这几格的 PATH 把它放最前面
  local unit_root="$work/unit-root" unit_crate
  mkdir -p "$work/unit-bin"
  printf '%s\n' '#!/usr/bin/env bash' '# 假 rustc（自证用）：只答版本' \
    'case "${1:-}" in -V) echo "rustc 0.0.0-selftest" ;; -vV) printf "rustc 0.0.0-selftest\nbinary: rustc\nhost: x86_64-unknown-linux-gnu\nrelease: 0.0.0-selftest\n" ;; esac' 'exit 0' >"$work/unit-bin/rustc"
  chmod +x "$work/unit-bin/rustc"
  cp -dR --preserve=timestamps,links -- "$root" "$unit_root"
  rm -rf -- "${unit_root:?}/.git"
  unit_crate="$unit_root/crates/singlefs-verification"
  mkdir -p "$unit_crate/src" "$unit_root/.unit-verdict-sample"
  printf '%s\n' '[workspace]' 'members = ["crates/singlefs-verification"]' 'resolver = "2"' >"$unit_root/Cargo.toml"
  printf '%s\n' 'version = 4' '' '[[package]]' 'name = "singlefs-verification"' 'version = "0.0.0"' >"$unit_root/Cargo.lock"
  printf '%s\n' '[package]' 'name = "singlefs-verification"' 'version = "0.0.0"' 'edition = "2021"' '' '[features]' 'gpu = []' >"$unit_crate/Cargo.toml"
  printf '%s\n' 'pub fn alpha_helper() -> u32 {' '    1' '}' '' 'pub fn not_read_by_any_test() -> u32 {' '    7' '}' >"$unit_crate/src/lib.rs"
  cp -- "$unit_crate/src/lib.rs" "$work/unit-lib.rs"
  printf '%s\n' 'use singlefs_verification::alpha_helper;' '' '#[test]' 'fn alpha_reads_the_helper() {' '    assert_eq!(alpha_helper(), 1);' '}' '' '#[test]' 'fn alpha_stands_alone() {' '    assert_eq!(2, 2);' '}' \
    >"$unit_crate/tests/alpha.rs"
  printf '%s\n' '#[test]' 'fn beta_stands_alone() {' '    assert_eq!(3, 3);' '}' >"$unit_crate/tests/beta.rs"
  printf '%s\n' '#[test]' 'fn gamma_stands_alone() {' '    assert_eq!(4, 4);' '}' >"$unit_crate/tests/gamma_amp.rs"
  if ! { git -C "$unit_root" init -q -b master && git -C "$unit_root" add -A && git -C "$unit_root" -c core.hooksPath=/dev/null -c user.name=selftest -c user.email=selftest@invalid commit -qm unit-root; } >"$work/unit-root-git.log" 2>&1; then
    echo "  ✗ 自证 setup：按条复用那几格的小仓 $unit_root 做不成 git 仓（$(tr '\n' ' ' <"$work/unit-root-git.log" | cut -c1-300)）" >&2
    echo "  → 怎么办：看 git 在不在 PATH 上、TMPDIR 可不可写" >&2
    exit 1
  fi
  # 多机配置两份：没开 GPU（只写假清场命令）、开了 GPU（加显卡先后两份）；清场往调用记录记一笔
  printf '%s\n' "QUIESCE_STOP_COMMAND='echo quiesce-stop >> \"\$STUB_CALLS\"'" "QUIESCE_STOPPED_CHECK_COMMAND=true" >"$work/multi-host-gpu-off.env"
  #   开了 GPU 的那一份要过入口文件头 run-condition 的配置判法（crash_amplification_run_condition_holds 经 crash-point-replay-shard-configuration-check.sh --emit-addresses 判整份），
  #   键照那份判法的自证样本写全：127.0.0.1 在每一台的网卡上（lo）、是这一台，192.0.2.1 是文档地址段、是另一台；显卡先后的地址取 HOSTS 里的 127.0.0.1
  { cat "$work/multi-host-gpu-off.env"
    printf '%s\n' 'QUIESCE_START_COMMAND=' 'QUIESCE_STARTED_CHECK_COMMAND=' 'HOSTS=127.0.0.1 192.0.2.1' \
      'CARGO_BIN_DIRECTORY_BY_HOST=127.0.0.1=/selftest/this-cargo 192.0.2.1=/selftest/peer-cargo' \
      'BUILD_ROOT_BY_HOST=127.0.0.1=/selftest/this-builds 192.0.2.1=/selftest/peer-builds' 'SLICE_MEMORY_CAP_BY_HOST=127.0.0.1=2G 192.0.2.1=1G' \
      "NFS_SHARED_ROOTS=$work" ENABLE_ACROSS_MACHINES=0 ENABLE_GPU=1 'GPU_CARDS_BY_PRIORITY="127.0.0.1:0:8G"' 'GPU_CARDS_BY_PRIORITY_WHEN_QUIESCED="127.0.0.1:0:24G"'
  } >"$work/multi-host-gpu-on.env"
  export GPU_QUIESCE_STATE_DIRECTORY="$work/gpu-quiesce-state" GPU_QUIESCE_LOG_DIRECTORY="$work/gpu-quiesce-logs"
  unset STUB_AMP_PERSISTENT STUB_AMP_SERVICE STUB_REUSE_TARGETS
  unset GATE_NOT_RUN_FILE STUB_RUN_CONDITION_TARGETS STUB_PEER_EXIT STUB_RED_TARGETS STUB_ZERO_TARGETS STUB_NOT_RUN_TARGETS STUB_AMP_DROP STUB_AMP_RED STUB_AMP_NOT_RUN STUB_AMP_EXIT STUB_CRASH_POINT_REPLAY_EXIT STUB_CRASH_POINT_REPLAY_REUSED STUB_BUILD_EXIT

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
    # 服务交下来的 CARGO_BUILD_JOBS 不带进子进程：test-functions-threads-from-service 那一格自己设一个认得出的值，别的格桩 cargo 记 unset
    env -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP -u SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE -u SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES \
      -u GPU_QUIESCE_ORIGIN_READ_BACK -u GPU_CARDS_BY_PRIORITY -u SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO -u SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO \
      -u CARGO_BUILD_JOBS \
      PATH="$work/bin:$PATH" STUB_CALLS="$calls" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
      SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" CRASH_POINT_REPLAY_SELFTEST_MARK="$work/mark" CRASH_POINT_REPLAY_ROOT="$root" \
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

  # 整趟才跑的那几类的测试目标个数，从 WHOLE_RUN_GROUPS 现数（期望与名单同源：名单加减一个名字，自证不用跟着改）
  local extras_count=0 whole_run_line whole_run_name whole_run_targets whole_run_words
  for whole_run_line in "${WHOLE_RUN_GROUPS[@]}"; do
    IFS=$'\t' read -r whole_run_name whole_run_targets <<<"$whole_run_line"
    [[ "$whole_run_targets" == - ]] && continue
    read -r -a whole_run_words <<<"$whole_run_targets"
    extras_count=$((extras_count + ${#whole_run_words[@]}))
  done
  # 其中放开 --test-threads 1 的那几个（GPU_TEST_THREADS_RELEASED_TARGETS，同源现数）：开了 GPU 时它们带 --test-threads <登记的重活默认并行度>，别的照旧带 --test-threads 1
  local released_extras=0 released_target released_test_threads_expected released_wrong
  for released_target in "${GPU_TEST_THREADS_RELEASED_TARGETS[@]}"; do
    for whole_run_line in "${WHOLE_RUN_GROUPS[@]}"; do
      IFS=$'\t' read -r whole_run_name whole_run_targets <<<"$whole_run_line"
      [[ " $whole_run_targets " == *" $released_target "* ]] && released_extras=$((released_extras + 1))
    done
  done
  released_test_threads_expected="$(heavy_thread_default)"
  # list-items：退 0、三行、不起 cargo 与驱动
  entry -- --list-items
  if [[ "$exit_code" != 0 || "$(grep -cE $'^(alpha|beta|gamma)\t测试函数 ' "$work/out")" != 3 ]] || [[ -s "$calls" ]] \
     || [[ "$(grep -c '^whole-run:' "$work/out")" != "${#WHOLE_RUN_GROUPS[@]}" ]] || ! out_has $'alpha\t测试函数 alpha ∪ 崩溃点重放 crash-case:alpha ∪ 放量 alpha-amp'; then
    cell_failed list-items "--list-items 应当退 0、逐项列出（含整趟才跑的几类 whole-run:<类>）、不起 cargo 与驱动" "看 --list-items 那一段：只读合表、打表就退"
  fi
  # list-items-with-keys：读库程序的输出取它文件头「输出」那一段原样喂进来（写法与读法钉在同一段字上）⇒ 退 0、三项照列，
  # 报录下的块 6、键对不上的块 4（只在别的判法单元指纹下判过 3、一种判器的结论都没有 1）、键相同的 2，与镜像判器（image_judgement）那一把指纹；不起 cargo。
  #   库的名字照那一段 CRASH_LIBRARY_KEYS 行的 library= 现取（判定库在 prune-server 里，读库程序报哪一处由它定，这里不抄死）
  local keys_reader_source="$replay_script_directory/../../crates/singlefs-verification/src/bin/crash_library_reader.rs"
  sed -n '/^\/\/! keys-format-begin$/,/^\/\/! keys-format-end$/p' "$keys_reader_source" | sed '1d;$d' | sed 's|^//! ||' >"$work/keys-reader-output"
  local keys_library_name
  keys_library_name="$(awk -F'\t' '$1 == "CRASH_LIBRARY_KEYS" { sub(/^library=/, "", $2); print $2; exit }' "$work/keys-reader-output")"
  entry CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys
  if [[ "$exit_code" != 0 || "$(grep -cE $'^(alpha|beta|gamma)\t测试函数 ' "$work/out")" != 3 ]] || [[ -s "$calls" || -z "$keys_library_name" ]] \
     || ! out_has $'键\t崩溃判定库 '"$keys_library_name"$'：录下的块 6，键对不上的块 4（只在别的判法单元指纹下判过 3、一种判器的结论都没有 1），键相同的 2' \
     || ! out_has $'键\timage_judgement 这一份的判法单元指纹 0505050505050505'; then
    cell_failed list-items-with-keys "--list-items --with-keys 应当照读库程序文件头「输出」那一段的写法报键对不上的块数、不起 cargo" \
      "看 report_the_keys_of_the_library 与 crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」那一段"
  fi
  # keys-reader-same-build：--list-items --with-keys 的读库程序与真跑编同一套特性（文件头「--list-items --with-keys」）：① 特性串 = 真跑带的特性加上 验证包清单
  #   [dev-dependencies] 里每个带特性的依赖的 <依赖>/<特性>（cargo test 的树里打开、cargo run 的树里没有的那几个），没有带特性的依赖就只剩真跑带的；
  #   ② 开了 GPU 的小仓里，真跑测试函数那一趟带的 --features 与读库程序那一趟 cargo run 带的相同
  local keys_wrong="" keys_manifest="$work/keys-manifest.toml" keys_test_features keys_reader_features_seen
  printf '%s\n' '[package]' 'name = "singlefs-verification"' '[dev-dependencies]' \
    'singlefs-core = { path = "../singlefs-core", features = ["lookup-work-counters", "second-sample"] }' 'plain = "1"' >"$keys_manifest"
  [[ "$(keys_reader_features "$keys_manifest" gpu)" == "gpu,singlefs-core/lookup-work-counters,singlefs-core/second-sample" ]] \
    || keys_wrong+="特性串没把 dev-dependencies 里带特性的依赖算进去（得到「$(keys_reader_features "$keys_manifest" gpu)」）；"
  [[ -z "$(keys_reader_features "$root/crates/singlefs-verification/Cargo.toml" "")" ]] || keys_wrong+="没有 dev-dependencies、没开 GPU 的清单应当得到空串；"
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" GPU_CARDS_BY_PRIORITY=127.0.0.1:1:20G --
  keys_test_features="$(sed -nE 's/^cargo test --release -p singlefs-verification --features ([^ ]+) .*/\1/p' "$calls" | head -n 1)"
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" STUB_KEYS_READER="$work/keys-reader-output" -- --list-items --with-keys
  keys_reader_features_seen="$(sed -nE 's/^cargo run --release --quiet --offline -p singlefs-verification --features ([^ ]+) --bin crash_library_reader .*/\1/p' "$calls" | head -n 1)"
  [[ "$exit_code" == 0 && -n "$keys_test_features" && "$keys_reader_features_seen" == "$keys_test_features" ]] \
    || keys_wrong+="真跑测试函数带的特性「$keys_test_features」与读库程序带的「$keys_reader_features_seen」不同（--list-items --with-keys 退 $exit_code）；"
  [[ -z "$keys_wrong" ]] || cell_failed keys-reader-same-build "读库程序应当与真跑编同一套特性：$keys_wrong" "看 report_the_keys_of_the_library 里 keys_reader_features 那一段与文件头「--list-items --with-keys」"
  # 按件分片的计划目录（默认分片那几格用）
  local shard_root="$work/shard"
  shard_plan_settings() { printf '%s\n' TASK_SERVER_SHARD_PLAN_DIRECTORY="$shard_root/plan"; }
  # 默认分片那几格（文件头「经 task-server 按件分片」）：小仓合表的件是 crash-case:alpha、alpha-amp、crash-case:beta、gamma-amp，加整趟才跑的那几类
  default_shard_plan() { # [<变量=值>…] -- <参数…>：新建计划
    local settings=()
    while (($#)) && [[ "$1" != -- ]]; do settings+=("$1"); shift; done
    shift
    rm -rf -- "${shard_root:?}"; mkdir -p -- "$shard_root"
    entry TASK_SERVER_SHARD_PHASE=plan "$(shard_plan_settings)" ${settings[@]+"${settings[@]}"} -- "$@"
  }
  default_shard_count() { grep -o '"label"' "$shard_root/plan/plan.json" 2>/dev/null | wc -l; }
  default_shard_of() { awk -F'\t' -v object="$1" '$4 == object && $3 != "test-functions" { print $1; exit }' "$shard_root/plan/shard-plan.tsv" 2>/dev/null; }   # <件>：它在第几片
  default_shard_run() { # <片号> <片数> [<变量=值>…] -- <参数…>
    local number="$1" total="$2" settings=()
    shift 2
    while (($#)) && [[ "$1" != -- ]]; do settings+=("$1"); shift; done
    shift
    entry TASK_SERVER_SHARD_PHASE=shard "$(shard_plan_settings)" TASK_SERVER_SHARD="$number/$total" TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$shard_root/output-$number" ${settings[@]+"${settings[@]}"} -- "$@"
  }
  default_shard_merge() { # <片数> <参数…>
    local number total="$1"
    shift
    for ((number = 0; number < total; number++)); do printf '%s\t192.0.2.1\t%s\n' "$number" "$shard_root/output-$number"; done >"$shard_root/outputs.tsv"
    entry TASK_SERVER_SHARD_PHASE=merge "$(shard_plan_settings)" TASK_SERVER_SHARD_OUTPUTS="$shard_root/outputs.tsv" -- "$@"
  }
  local default_count default_number default_wrong
  # shard-default-plan：不带参数（不带参数）也分：每件恰好落在一片，每片带上它的项的测试函数；不起测试函数与两套驱动
  default_shard_plan --
  default_count="$(default_shard_count)"
  if [[ "$exit_code" != 0 || "$default_count" -lt 4 ]] || calls_have '^cargo test ' || calls_have '^(pipeline-entry|enumeration-gate)' \
     || [[ "$(awk -F'\t' '$3 != "test-functions" && ($4 == "crash-case:alpha" || $4 == "alpha-amp" || $4 == "crash-case:beta" || $4 == "gamma-amp") { print $4 }' "$shard_root/plan/shard-plan.tsv" 2>/dev/null | LC_ALL=C sort | tr '\n' ' ')" != "alpha-amp crash-case:alpha crash-case:beta gamma-amp " ]] \
     || ! awk -F'\t' -v shard="$(default_shard_of crash-case:beta)" '$1 == shard && $2 == "beta" && $3 == "test-functions" { found = 1 } END { exit !found }' "$shard_root/plan/shard-plan.tsv" 2>/dev/null; then
    cell_failed shard-default-plan "不带参数应当默认分片：每件恰好落在一片、每片带上它的项的测试函数，计划那一步不起测试函数与两套驱动（退 $exit_code，$default_count 片）" "看 shard_write_the_plan 与文件头第 2 行的声明"
  fi
  # shard-default-one-part：一片只跑自己的件（crash-case:beta 那一片：只起它与 beta 的测试函数）
  default_shard_run "$(default_shard_of crash-case:beta)" "$default_count" --
  if [[ "$exit_code" != 0 ]] || ! calls_have '^enumeration-gate --item crash-case:beta ' || calls_have 'crash-case:alpha' || calls_have '^pipeline-entry ' \
     || [[ ! -f "$shard_root/output-$(default_shard_of crash-case:beta)/shard-finished" ]] || ! calls_have '^cargo test --release -p singlefs-verification --no-run --test beta cards='; then
    cell_failed shard-default-one-part "默认分片的一片应当只跑自己的件与它的项的测试函数、存下 shard-finished（退 $exit_code）" "看主流程 shard_phase == shard 那两支与 shard_read_the_plan"
  fi
  # shard-default-merge-red：崩溃点重放那一道全红：每片照退 0，别的片照跑；合并不起测试函数与两套驱动，判出红、退 1；ITEM 与 TOTAL 同不分片
  default_shard_plan --
  default_wrong=""
  for ((default_number = 0; default_number < default_count; default_number++)); do
    default_shard_run "$default_number" "$default_count" STUB_CRASH_POINT_REPLAY_EXIT=1 --
    [[ "$exit_code" == 0 ]] || default_wrong+="第 $default_number 片退 $exit_code；"
  done
  default_shard_merge "$default_count"
  if [[ -n "$default_wrong" || "$exit_code" != 1 ]] || calls_have '^cargo test ' || calls_have '^(pipeline-entry|enumeration-gate)' \
     || ! item_has beta 'verdict=red' || ! item_has gamma 'verdict=green' || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL scope=whole .* red=2 ' "$work/out"; then
    cell_failed shard-default-merge-red "判红的片应当退 0、别的片照跑，合并判出红退 1、不起测试函数与两套驱动（${default_wrong}合并退 $exit_code）" "看 shard_collect_the_outputs 与收尾把一片的 1 改成 0 那一段"
  fi
  # shard-default-merge-green：全绿：合并退 0、TOTAL 报 0 红
  default_shard_plan --
  for ((default_number = 0; default_number < default_count; default_number++)); do default_shard_run "$default_number" "$default_count" --; done
  default_shard_merge "$default_count"
  if [[ "$exit_code" != 0 ]] || ! item_has alpha 'verdict=green' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL scope=whole .* red=0 ' "$work/out"; then
    cell_failed shard-default-merge-green "全绿的默认分片，合并应当退 0、TOTAL 报 0 红（退 $exit_code）" "看 shard_collect_the_outputs 与 summarize"
  fi
  # shard-default-many-parts：上限压到 2：件都合进 2 片、一件不丢，合成的片跑它的几件，合并照判
  default_shard_plan CRASH_POINT_REPLAY_MOST_SHARDS=2 --
  default_count="$(default_shard_count)"
  if [[ "$exit_code" != 0 || "$default_count" != 2 ]] \
     || [[ "$(awk -F'\t' '$3 != "test-functions" && ($4 == "crash-case:alpha" || $4 == "alpha-amp" || $4 == "crash-case:beta" || $4 == "gamma-amp") { print $4 }' "$shard_root/plan/shard-plan.tsv" 2>/dev/null | LC_ALL=C sort | tr '\n' ' ')" != "alpha-amp crash-case:alpha crash-case:beta gamma-amp " ]]; then
    cell_failed shard-default-many-parts "件数超过片数上限应当合成上限那么多片、一件不丢（退 $exit_code，$default_count 片）" "看 shard_write_the_plan 定片数那一段"
  else
    default_shard_run 0 2 --
    default_wrong="第 0 片退 $exit_code、起了 $(calls_count '^(pipeline-entry|enumeration-gate) ') 件"
    if [[ "$exit_code" != 0 ]] || [[ "$(calls_count '^(pipeline-entry|enumeration-gate) ')" -lt 2 ]]; then
      cell_failed shard-default-many-parts "合成的那一片应当跑它的几件（$default_wrong）" "看主流程一片逐行起崩溃点重放那一支与 run_pipeline_part"
    fi
  fi
  # shard-position-refused：片号不在计划里、片数对不上、阶段认不得 ⇒ 都退 2、一件都不跑
  default_shard_plan --
  default_count="$(default_shard_count)"
  local shard_position_wrong=""
  default_shard_run 99 "$default_count" --; [[ "$exit_code" == 2 ]] && ! calls_have '^(pipeline-entry|enumeration-gate)' || shard_position_wrong+="片号 99 退 $exit_code；"
  default_shard_run 0 999 --; [[ "$exit_code" == 2 ]] && ! calls_have '^(pipeline-entry|enumeration-gate)' || shard_position_wrong+="片数 999 退 $exit_code；"
  entry TASK_SERVER_SHARD_PHASE=no-such-phase "$(shard_plan_settings)" --
  [[ "$exit_code" == 2 ]] && ! calls_have '^(pipeline-entry|enumeration-gate)' || shard_position_wrong+="认不得的阶段退 $exit_code；"
  [[ -z "$shard_position_wrong" ]] || cell_failed shard-position-refused "片号对不上计划或阶段认不得应当退 2、一件都不跑（$shard_position_wrong）" "看主流程 shard_phase == shard 那一支与参数之后核阶段那一段"
  # shard-merge-refuses-incomplete：第 0 片缺 shard-finished ⇒ 合并退 1、不判
  default_shard_plan --
  default_count="$(default_shard_count)"
  for ((default_number = 0; default_number < default_count; default_number++)); do default_shard_run "$default_number" "$default_count" --; done
  rm -f -- "$shard_root/output-0/shard-finished"
  default_shard_merge "$default_count"
  if [[ "$exit_code" != 1 ]] || ! out_has "第 0 片的产物不全"; then
    cell_failed shard-merge-refuses-incomplete "有一片的产物不全，合并应当退 1、点名那一片（退 $exit_code）" "看 shard_collect_the_outputs 核 shard-finished 那一段"
  fi
  # shard-no-shard：带 --no-shard 不分：计划写一片、什么都不起；那一片整趟跑；合并什么都不做
  rm -rf -- "${shard_root:?}"; mkdir -p -- "$shard_root"
  entry TASK_SERVER_SHARD_PHASE=plan "$(shard_plan_settings)" -- --no-shard
  default_wrong=""
  [[ "$exit_code" == 0 && "$(default_shard_count)" == 1 && ! -s "$calls" ]] || default_wrong+="计划退 $exit_code、$(default_shard_count) 片；"
  entry TASK_SERVER_SHARD_PHASE=shard "$(shard_plan_settings)" TASK_SERVER_SHARD=0/1 TASK_SERVER_SHARD_OUTPUT_DIRECTORY="$shard_root/output-0" -- --no-shard
  [[ "$exit_code" == 0 && "$(calls_count '^enumeration-gate ')" == 2 && -d "$shard_root/output-0" ]] || default_wrong+="那一片退 $exit_code、起了 $(calls_count '^enumeration-gate ') 条用例；"
  default_shard_merge 1 --no-shard
  [[ "$exit_code" == 0 && ! -s "$calls" ]] || default_wrong+="合并退 $exit_code；"
  [[ -z "$default_wrong" ]] || cell_failed shard-no-shard "带 --no-shard 应当计划一片、那一片整趟跑、合并什么都不做（$default_wrong）" "看参数之后 no_shard 那一段"
  # unknown-item / missing-item-value / refused-options：用法错退 2、什么都不起
  entry -- --item no-such-item
  if [[ "$exit_code" != 2 ]] || ! out_has "alpha" || ! out_has "gamma" || ! out_has "whole-run:pruning-self-proof" || [[ -s "$calls" ]]; then
    cell_failed unknown-item "项名写错应当退 2、列出可点名的项（含 whole-run:<类>）、什么都不起" "看核项名那一段"
  fi
  entry -- --item
  [[ "$exit_code" == 2 && ! -s "$calls" ]] || cell_failed missing-item-value "--item 后面缺项名应当退 2" "看参数那一段的 --item"
  # refused-options：删掉的选项（整趟现判、清空复用、快速那一档、积压跑法、跟在 --item 后面的）一律当认不得的参数退 2、什么都不起、出路只写正确用法
  #   选项名用两段拼，不在这份文件里写出整串
  local refused_wrong="" refused_arguments refused_option
  for refused_option in "--""full" "--""start-over" "--""quick" "--""back""log" "--""light"; do
    for refused_arguments in "$refused_option" "--item beta $refused_option"; do
      # shellcheck disable=SC2086  # 参数是空格隔开的几个词，逐个交
      entry -- $refused_arguments
      { [[ "$exit_code" == 2 && ! -s "$calls" ]] && out_has "认不得的参数 $refused_option" && ! out_has "已删"; } || refused_wrong+="「$refused_arguments」退 $exit_code；"
    done
  done
  [[ -z "$refused_wrong" ]] || cell_failed refused-options "删掉的选项应当照认不得的参数退 2、什么都不起、不说已删（$refused_wrong）" "看参数那一段：只认 --no-shard、--list-items、--with-keys、--selftest、--item"
  # expand-up-to-refused：--crash-expand-up-to 的值不是非负整数（abc、-1、后面是别的选项）、后面没有值，环境里的缩域变量写坏 ⇒ 都退 2、什么都不起
  local expand_wrong="" expand_case
  for expand_case in "--crash-expand-up-to abc" "--crash-expand-up-to -1" "--crash-expand-up-to --item" "--crash-expand-up-to"; do
    # shellcheck disable=SC2086  # 参数是空格隔开的几个词，逐个交
    entry -- $expand_case
    { [[ "$exit_code" == 2 && ! -s "$calls" ]] && out_has "用法不对"; } || expand_wrong+="「$expand_case」退 $exit_code；"
  done
  entry SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=abc --
  { [[ "$exit_code" == 2 && ! -s "$calls" ]]; } || expand_wrong+="环境变量 SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO 写 abc 退 $exit_code；"
  entry SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO=abc --
  { [[ "$exit_code" == 2 && ! -s "$calls" ]]; } || expand_wrong+="环境变量 SINGLEFS_CRASH_JUDGE_EXPAND_UP_TO 写 abc 退 $exit_code；"
  [[ -z "$expand_wrong" ]] || cell_failed expand-up-to-refused "缩域的值不是非负整数、后面没有值都应当退 2、什么都不起、点名用法（$expand_wrong）" "看参数之后核缩域值的那一段（crash_expand_up_to_origin）"
  # expand-up-to-passed：给了 --crash-expand-up-to 6（或环境里已设变量）⇒ 测试函数、崩溃点重放、放量三段都收到同一个值，ITEM 与 TOTAL 行末尾接 expand_up_to=6，
  #   成功句写缩过域、不算门禁通过；参数与环境变量都给取参数；不缩的那一趟三段看到的是 unset，行里与句里都没有缩域的说法
  local expand_passed_wrong=""
  expand_check() { # <要的值> <说明>：上一趟 entry 的结果对不对
    local value="$1" cargo_runs
    cargo_runs="$(calls_count '^cargo-env expand=')"
    { [[ "$exit_code" == 0 ]] && (( cargo_runs > 0 )) && [[ "$(calls_count "^cargo-env expand=\[$value\]\$")" == "$cargo_runs" ]] \
      && calls_have "^enumeration-env expand=\[$value\]\$" && calls_have "^pipeline-env expand=\[$value\]\$" \
      && grep -qE "^CRASH_POINT_REPLAY_TOTAL .* expand_up_to=$value\$" "$work/out" && grep -qE "^CRASH_POINT_REPLAY_ITEM item=alpha .* expand_up_to=$value\$" "$work/out" \
      && out_has "缩过域" && out_has "不算门禁通过"; } || expand_passed_wrong+="$2（退 $exit_code）；"
  }
  entry -- --crash-expand-up-to 6; expand_check 6 "参数 6"
  entry SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=4 --; expand_check 4 "只设环境变量 4"
  entry SINGLEFS_CRASH_AMPLIFICATION_EXPAND_UP_TO=4 -- --crash-expand-up-to 6; expand_check 6 "参数 6 与环境变量 4 都给"
  entry --
  { [[ "$exit_code" == 0 ]] && [[ "$(calls_count '^cargo-env expand=\[unset\]$')" == "$(calls_count '^cargo-env expand=')" ]] \
    && calls_have '^enumeration-env expand=\[unset\]$' && calls_have '^pipeline-env expand=\[unset\]$' \
    && ! grep -q 'expand_up_to=' "$work/out" && ! out_has "缩过域"; } || expand_passed_wrong+="不缩的那一趟三段应当看到 unset、行里与句里没有缩域的说法（退 $exit_code）；"
  [[ -z "$expand_passed_wrong" ]] || cell_failed expand-up-to-passed "给了缩域值，三段都应当收到变量、ITEM 与 TOTAL 行带 expand_up_to、成功句写缩过域（$expand_passed_wrong）" \
    "看参数之后「缩崩溃域」那一段与 test_functions_environment"
  # item-done-lines：真跑每项跑完一行 ITEM_DONE（stderr），行数等于 ITEM 行数；alpha（崩溃枚举用例全复用、放量判了 10 态）judged=1 reused=1，beta（用例全复用）judged=0 reused=1，
  #   gamma（放量判了）judged=1 reused=0，都退 0；beta 的崩溃点重放那一道睡 3 秒：beta 的 seconds 不小于 3、gamma 的比它小（各记各的，不是整趟的）
  local done_wrong="" done_beta_seconds done_gamma_seconds
  entry STUB_CRASH_POINT_REPLAY_REUSED=1 STUB_GATE_BETA_SLEEP=3 --
  [[ "$exit_code" == 0 && "$(grep -c '^ITEM_DONE ' "$work/out")" == "$(grep -c '^CRASH_POINT_REPLAY_ITEM ' "$work/out")" ]] \
    || done_wrong+="ITEM_DONE 的行数应当等于 ITEM 行数（退 $exit_code，$(grep -c '^ITEM_DONE ' "$work/out") 对 $(grep -c '^CRASH_POINT_REPLAY_ITEM ' "$work/out")）；"
  item_done_has() { grep -qE "^ITEM_DONE item=$1 seconds=[0-9]+ judged=$2 reused=$3 exit=0\$" "$work/out"; }   # <项> <现判> <复用>
  { item_done_has alpha 1 1 && item_done_has beta 0 1 && item_done_has gamma 1 0; } || done_wrong+="alpha 应当 judged=1 reused=1、beta judged=0 reused=1、gamma judged=1 reused=0，都退 0；"
  done_beta_seconds="$(sed -nE 's/^ITEM_DONE item=beta seconds=([0-9]+) .*/\1/p' "$work/out")"
  done_gamma_seconds="$(sed -nE 's/^ITEM_DONE item=gamma seconds=([0-9]+) .*/\1/p' "$work/out")"
  { [[ "$done_beta_seconds" =~ ^[0-9]+$ && "$done_gamma_seconds" =~ ^[0-9]+$ ]] && (( done_beta_seconds >= 3 && done_gamma_seconds < done_beta_seconds )); } \
    || done_wrong+="seconds 应当各记各的：beta「$done_beta_seconds」不小于 3、gamma「$done_gamma_seconds」比 beta 小；"
  [[ -z "$done_wrong" ]] || cell_failed item-done-lines "真跑应当每项一行 ITEM_DONE、件数与退出码照各段的记录、挂钟各记各的：$done_wrong" "看 summarize 里 ITEM_DONE 那一段与 record_part 第十一列"
  # gpu-test-functions：开了 GPU，测试函数带 gpu 特性、每一趟交 task-server 给的那张卡（环境里的 GPU_CARDS_BY_PRIORITY，多机配置里写的显卡先后不读）、不自己清场
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" GPU_CARDS_BY_PRIORITY=127.0.0.1:1:20G --
  if [[ "$exit_code" != 0 ]] || calls_have '^quiesce-stop$' \
     || [[ "$(calls_count "^cargo test --release -p singlefs-verification --features gpu --test [a-z0-9_]+ -- --nocapture --test-threads 1 cards=\[127.0.0.1:1:20G\] quiesced=\[\]")" != $((3 + extras_count - released_extras)) ]] \
     || ! calls_have '^cargo test .*--features gpu --no-run .*quiesced=\[\]'; then
    cell_failed gpu-test-functions "开了 GPU 测试函数应当带 gpu 特性、不自己清场、测试参数带 --test-threads 1（GPU_TEST_THREADS_RELEASED_TARGETS 那几个除外）、交 task-server 给的那张卡（127.0.0.1:1:20G，不是多机配置里的显卡先后）" \
      "看 run_test_functions 里 gpu_switch 那一段"
  fi
  # gpu-agreement-test-threads-released：开了 GPU，GPU_TEST_THREADS_RELEASED_TARGETS 里的测试目标（整趟才跑的那几类里有）不带 --test-threads 1，
  #   带 --test-threads <登记的重活默认并行度>（它的测试按显存额度排队）；别的 GPU 目标照旧带 1，由上面 gpu-test-functions 那一格数
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" GPU_CARDS_BY_PRIORITY=127.0.0.1:1:20G --
  released_wrong=""
  (( released_extras > 0 )) || released_wrong+="整趟才跑的那几类里一个放开的测试目标都没有（WHOLE_RUN_GROUPS 与 GPU_TEST_THREADS_RELEASED_TARGETS 对不上）；"
  for released_target in "${GPU_TEST_THREADS_RELEASED_TARGETS[@]}"; do
    calls_have "^cargo test --release -p singlefs-verification --features gpu --test $released_target -- --nocapture --test-threads $released_test_threads_expected cards=" \
      || released_wrong+="$released_target 没带 --test-threads $released_test_threads_expected；"
    calls_have "^cargo test .*--test $released_target -- --nocapture --test-threads 1 " && released_wrong+="$released_target 带了 --test-threads 1；"
  done
  if [[ -n "$released_wrong" ]]; then
    cell_failed gpu-agreement-test-threads-released "开了 GPU，放开的测试目标应当带 --test-threads $released_test_threads_expected（登记的重活默认并行度）、不带 --test-threads 1：$released_wrong" \
      "看 run_test_functions 里 target_test_threads 那一段与文件头 GPU_TEST_THREADS_RELEASED_TARGETS"
  fi
  # gpu-card-missing：开了 GPU 而环境里没有 task-server 交的卡（不是经服务起的）⇒ 退 78、测试函数一趟都不起、写明经 task-server 交
  entry SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-on.env" --
  if [[ "$exit_code" != 78 ]] || calls_have '^cargo test' || ! out_has '环境里却没有 task-server 交的显卡（GPU_CARDS_BY_PRIORITY）'; then
    cell_failed gpu-card-missing "开了 GPU 而环境里没有 task-server 交的卡，应当退 78、一项都不起" "看主体「GPU 与配置」那一段 gpu_cards_for_test_functions 为空那一支"
  fi
  # test-functions-red：beta 那个测试目标红 ⇒ beta 红、退 1，别的测试目标照绿（alpha 的测试函数两条都过、不红；它的放量部分因测试函数红没起，记本次未跑，见 test-functions-red-stops-light）
  entry STUB_RED_TARGETS=beta --
  if [[ "$exit_code" != 1 ]] || ! item_has beta 'verdict=red' || item_has alpha 'verdict=red' || ! item_has alpha 'test_functions_tests=2 test_functions_red=0 ' || ! out_has "✗ beta："; then
    cell_failed test-functions-red "测试函数一个测试目标红了，那一项应当红、整趟退 1" "看 run_test_functions 判 state 那一段"
  fi
  # test-functions-not-run：beta 跑到 0 条、alpha 有 NOT_RUN_NO_GPU_ADAPTER ⇒ 两项本次未跑、退 0，成功句点名
  entry STUB_ZERO_TARGETS=beta STUB_NOT_RUN_TARGETS=alpha --
  if [[ "$exit_code" != 0 ]] || ! item_has beta 'verdict=not-run' || ! item_has alpha 'verdict=not-run' \
     || ! grep -q '本次未跑 3 项：alpha（有 1 处 NOT_RUN_NO_GPU_ADAPTER' "$work/out"; then
    cell_failed test-functions-not-run "测试函数跑到 0 条或没卡没跑的，应当记本次未跑、成功句点名" "看 run_test_functions 判 state 那一段与 summarize 的成功句"
  fi
  # all-not-run：测试函数全是 0 条、放量全是 not-run ⇒ 退 77
  entry STUB_ZERO_TARGETS=all STUB_AMP_NOT_RUN="alpha-amp gamma-amp" --
  [[ "$exit_code" == 77 ]] || cell_failed all-not-run "派出的项全是本次未跑，应当退 77" "看 summarize 里 not green 那一支"
  # whole：不带参数 ⇒ 测试函数那一段（连同整趟才跑的那几类）、崩溃点重放每项一趟 --item、放量不带 --item；复用数照实报
  entry STUB_CRASH_POINT_REPLAY_REUSED=1 --
  if [[ "$exit_code" != 0 ]] || [[ "$(calls_count '^cargo test --release -p singlefs-verification --test [a-z0-9_]+ -- --nocapture ')" != $((3 + extras_count)) ]] \
     || ! calls_have "^enumeration-gate --item crash-case:alpha $root\$" || ! calls_have "^enumeration-gate --item crash-case:beta $root\$" \
     || ! calls_have '^pipeline-entry $' || calls_have '^admission ' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL scope=whole dispatched=6 collected=6 .* pipeline_reused_known=8 .* enumeration_cases=2 enumeration_reused=2 ' "$work/out"; then
    cell_failed whole "不带参数应当跑全部项（测试函数那一段照跑、崩溃点重放每项 --item、放量照全部项、复用数照实报）" "看 replay_plan 与 run_enumeration_part"
  fi
  # items：--item gamma ⇒ 只跑 gamma：测试函数那一段只 gamma_amp、不跑整趟才跑的那几类、不起崩溃点重放、放量 --item gamma-amp
  entry -- --item gamma
  if [[ "$exit_code" != 0 ]] || [[ "$(calls_count ' --test [a-z0-9_]+ -- --nocapture ')" != 1 ]] || ! calls_have ' --test gamma_amp -- --nocapture ' \
     || calls_have '^enumeration-gate ' || ! calls_have '^pipeline-entry --item gamma-amp$' || item_line 'whole-run:pruning-self-proof' >/dev/null \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL scope=items dispatched=1 ' "$work/out"; then
    cell_failed items "--item 应当只跑点名的项" "看 scope 那一段 items 与 run_pipeline_part 的 --item"
  fi
  # whole-run-item：--item whole-run:pruning-self-proof ⇒ 只跑那一类的两个测试目标，不起崩溃点重放与放量，ITEM 行点名那一项
  entry -- --item whole-run:pruning-self-proof
  if [[ "$exit_code" != 0 ]] || [[ "$(calls_count ' --test [a-z0-9_]+ -- --nocapture ')" != 2 ]] \
     || ! calls_have ' --test crash_amplification_two_flows_share_their_prefix -- --nocapture ' || ! calls_have ' --test crash_amplification_pipeline_records_checks_and_reuses -- --nocapture ' \
     || calls_have '^enumeration-gate ' || calls_have '^pipeline-entry ' || ! item_has 'whole-run:pruning-self-proof' 'scope=items'; then
    cell_failed whole-run-item "--item whole-run:<类> 应当只跑那一类的测试目标（退 $exit_code）" "看 load_whole_run_items 与 replay_plan 对 whole-run 项的处理"
  fi
  # 登记的用例不进测试函数那一段（--exclude-registered）：登记给 crash-case:alpha 的 alpha_full 与放量分项 alpha-amp 的 alpha_amplified 经 --skip 排掉，不判、不记；
  #   alpha_stands_alone 照判。unit-root 的 alpha.rs 临时加两条登记的用例，跑完换回
  # （这一格在下面 unit-root 那一批里做，需要真的 admission.py）
  # enumeration-red：崩溃点重放那一道退 1 ⇒ alpha、beta 红，退 1
  entry STUB_CRASH_POINT_REPLAY_EXIT=1 -- --item alpha --item beta
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'verdict=red' || ! item_has beta 'enumeration_red=1' \
     || ! out_has "只重跑红的那 2 项：bash scripts/crash/crash-point-replay.sh --item alpha --item beta"; then
    cell_failed enumeration-red "崩溃点重放那一道红了，那一项应当红、出路给出只重跑那两项的命令" "看 run_enumeration_part 判 exit_code 那一段与 summarize 的出路"
  fi
  # pipeline-red：放量 alpha-amp red=2 ⇒ alpha 红
  entry STUB_AMP_RED=alpha-amp --
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'pipeline_red=2' || ! item_has alpha 'verdict=red' || ! item_has gamma 'verdict=green'; then
    cell_failed pipeline-red "放量那一项红了，那一项应当红" "看 run_pipeline_part 判 state 那一段"
  fi
  # collection-count：放量入口没打 gamma-amp 那一行 ⇒ gamma 红（收不回）
  entry STUB_AMP_DROP=gamma-amp --
  if [[ "$exit_code" != 1 ]] || ! item_has gamma 'verdict=red' || ! out_has "收不回"; then
    cell_failed collection-count "放量那一项收不回，应当红、派出收回对不上" "看 run_pipeline_part 里 line 为空那一支"
  fi
  # test-functions-red-continues：beta 那个测试目标红 ⇒ 整趟退 1，崩溃点重放那一道与放量入口照旧起（两种跑法一样），派出收回相等
  local continue_wrong=() continue_case
  for continue_case in "" "--item alpha --item beta"; do
    # shellcheck disable=SC2086  # 参数是空格隔开的一串，逐个当参数
    entry STUB_RED_TARGETS=beta -- $continue_case
    if [[ "$exit_code" != 1 ]] || ! calls_have '^enumeration-gate --item crash-case:beta ' || ! calls_have '^pipeline-entry ' || ! item_has beta 'verdict=red' \
       || ! grep -qE 'CRASH_POINT_REPLAY_TOTAL .*dispatched=([0-9]+) collected=\1 ' "$work/out"; then
      continue_wrong+=("「crash-point-replay.sh $continue_case」退 $exit_code、调用记录 $(tr '\n' '|' <"$calls" | cut -c1-200)")
    fi
  done
  if (( ${#continue_wrong[@]} )); then
    exit_code="${#continue_wrong[@]} 处"
    cell_failed test-functions-red-continues "测试函数那一段红了，别的两段照旧起（崩溃点重放那一道与放量入口）、整趟退 1、派出收回相等：${continue_wrong[*]}" \
      "看主流程：测试函数、崩溃点重放、放量三段各自跑，不看别的段红不红"
  fi
  # run-target-green / run-target-red / run-target-term：跑完运行目录里不剩它建的编译目录（文件头「编译目录」）：调用方没设 CARGO_TARGET_DIR，不带参数（不带参数）；
  #   假 cargo 记下每一趟收到的编译目录、往里写一份产物。正常退 0、测试函数红退 1、被 TERM 停（假 cargo 每一趟睡 3 秒，头一趟起了之后只给入口发 TERM，
  #   入口等手里那一步退出后走 EXIT，退 143）：每一趟收到的都是 $work/home/runs/crash-point-replay-*/target 那同一份，跑完不在了
  run_target_check() { # <格名> <要的退出码> <一句结局>
    local seen
    seen="$(sort -u "$work/targets.log" 2>/dev/null)"
    if [[ "$exit_code" != "$2" || "$(grep -c . <<<"$seen")" != 1 || "$seen" != "$work"/home/runs/crash-point-replay-*/target || -e "$seen" ]]; then
      cell_failed "$1" "跑完运行目录里不剩它建的编译目录（$3）：假 cargo 收到「$(tr '\n' ' ' <<<"$seen")」，要的退出码 $2，跑完$([[ -n "$seen" && -e "$seen" ]] && echo 还在 || echo 不在)" \
        "看建日志目录之后的 run_build_directory_begin 与收尾的 replay_finish（run_build_directory_end），TERM / INT 的 trap 退 143 / 130、等手里那一步退出再走 EXIT"
    fi
  }
  : >"$work/targets.log"; entry STUB_TARGET_LOG="$work/targets.log" --; run_target_check run-target-green 0 "正常退 0"
  : >"$work/targets.log"; entry STUB_TARGET_LOG="$work/targets.log" STUB_RED_TARGETS=beta --; run_target_check run-target-red 1 "测试函数红退 1"
  local term_entry term_attempt
  : >"$work/targets.log"; : >"$calls"; cells=$((cells + 1))
  env -u RUN_WITH_MEMORY_CAP_OUTER_UNIT -u RUN_WITH_MEMORY_CAP_OUTER_CAP -u SINGLEFS_CRASH_POINT_REPLAY_SHARD_PEER_IS_THIS_MACHINE -u SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES \
    -u GPU_QUIESCE_ORIGIN_READ_BACK \
    PATH="$work/bin:$PATH" STUB_CALLS="$calls" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
    SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" CRASH_POINT_REPLAY_SELFTEST_MARK="$work/mark" CRASH_POINT_REPLAY_ROOT="$root" \
    CRASH_POINT_REPLAY_BREAK="$replay_break" STUB_TARGET_LOG="$work/targets.log" STUB_SLEEP=3 bash "$replay_script" >"$work/out" 2>&1 &
  term_entry=$!
  for (( term_attempt = 0; term_attempt < 600; term_attempt++ )); do [[ -s "$work/targets.log" ]] && break; sleep 0.1; done
  kill -TERM "$term_entry" 2>/dev/null
  if wait "$term_entry"; then exit_code=0; else exit_code=$?; fi
  run_target_check run-target-term 143 "被 TERM 停"
  # run-target-caller：调用方设了 CARGO_TARGET_DIR ⇒ 每一趟照收到它、跑完不删
  mkdir -p "$work/caller-target"; : >"$work/targets.log"
  entry STUB_TARGET_LOG="$work/targets.log" CARGO_TARGET_DIR="$work/caller-target" --
  if [[ "$exit_code" != 0 || "$(sort -u "$work/targets.log")" != "$work/caller-target" || ! -f "$work/caller-target/release/artifact" ]]; then
    cell_failed run-target-caller "调用方设了 CARGO_TARGET_DIR 时每一趟应当照收到它、跑完不删（收到「$(sort -u "$work/targets.log" | tr '\n' ' ')」）" \
      "看 run_build_directory_begin：设了 CARGO_TARGET_DIR 就不建、不删"
  fi
  # two-logs-per-part：每一段的日志出两份（全量 <段>.log，旁边同名的 <段>.problems.log 只收错误与警告：行前带全量日志里的行号，末行写收了几行全量几行）；
  #   测试函数红的那个测试目标的 FAILED 行在错误警告日志里，输出里点名这份日志的路径
  local problems_run problems_file problems_last_line
  entry STUB_RED_TARGETS=beta --
  problems_run="$(ls -td "$work/home/runs"/crash-point-replay-* 2>/dev/null | head -n 1)"
  problems_file="$problems_run/test-functions.beta.problems.log"
  problems_last_line="$(tail -n 1 "$problems_file" 2>/dev/null)"   # 管道收尾的 grep -q 在 pipefail 下会把命中读成没命中：先落到变量
  if [[ ! -f "$problems_run/test-functions.beta.log" || ! -f "$problems_file" ]] || ! grep -qE '^[0-9]+:test broken \.\.\. FAILED$' "$problems_file" \
     || ! grep -qE '收了 [0-9]+ 行，全量日志共 [0-9]+ 行' <<<"$problems_last_line" || ! out_has "错误警告 $problems_file"; then
    cell_failed two-logs-per-part "测试函数红的那一段应当出两份日志（test-functions.beta.log 全量、旁边的 test-functions.beta.problems.log 带行号收那条 FAILED、末行写收了几行全量几行），输出里点名错误警告日志的路径 $problems_file" \
      "看 run_logged 经 scripts/hosts/two-logs.sh 起命令，与红的原因里的 \${log%.log}.problems.log"
  fi
  # pipeline-source-changed-not-run：放量分项驱动判段跑的过程中源码变了（驱动退 77、打 CRASH_AMPLIFICATION_NOT_RUN step=judge …跑的过程中源码变了，入口记 verdict=not-run）⇒
  #   alpha 这一项本次未跑、不红，原因点名源码变了（「放量 alpha-amp 本次未跑」之后接那一行的 reason=）
  entry STUB_AMP_SOURCE_CHANGED=alpha-amp --
  if [[ "$exit_code" != 0 ]] || ! item_has alpha 'verdict=not-run' || item_has alpha 'verdict=red' \
     || ! out_has '本次未跑 alpha：放量 alpha-amp 本次未跑（exit=77）：跑的过程中源码变了，判段没在录入段那份源码上判完'; then
    cell_failed pipeline-source-changed-not-run "放量分项驱动报跑的过程中源码变了，那一项应当记本次未跑（verdict=not-run）、不记红，原因点名「跑的过程中源码变了」、整趟退 0" "看 run_pipeline_part 里 item_verdict 为 not-run 那一支与 pipeline_source_changed_reason"
  fi
  # pipeline-entry-78-not-run：放量入口退 78（运行条件不满足）而没打 CRASH_AMPLIFICATION_ITEM 行 ⇒ 两项都记本次未跑、整趟退 78，不说「都不红」；
  #   红先判：alpha-amp 红、gamma-amp 收不回而入口退 78 ⇒ 退 1；入口退 1 而收不回照旧红
  entry STUB_AMP_EXIT=78 STUB_AMP_DROP="alpha-amp gamma-amp" --
  if [[ "$exit_code" != 78 ]] || ! item_has alpha 'exit=78 verdict=not-run' || ! item_has gamma 'exit=78 verdict=not-run' || item_has alpha 'verdict=red' \
     || ! out_has '本次未跑 alpha：运行条件不满足：放量入口退 78' || out_has '项都不红' || ! out_has '有一段运行条件不满足（退 78）'; then
    cell_failed pipeline-entry-78-not-run "放量入口退 78 而没打 ITEM 行，两项应当记本次未跑（exit=78 verdict=not-run）、整趟退 78、不说都不红" "看 run_pipeline_part 里 line 为空那一支对退出码 78 的分支"
  fi
  entry STUB_AMP_RED=alpha-amp STUB_AMP_DROP=gamma-amp STUB_AMP_EXIT=78 --
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'verdict=red' || ! item_has gamma 'exit=78 verdict=not-run'; then
    cell_failed pipeline-entry-78-red-first "放量入口退 78 而 alpha-amp 红时，整趟应当退 1、alpha 红、gamma 本次未跑" "看 run_pipeline_part 与 summarize：红先判，退 78 的只在没有红时才让整趟退 78"
  fi
  entry STUB_AMP_EXIT=1 STUB_AMP_DROP=gamma-amp --
  if [[ "$exit_code" != 1 ]] || ! item_has gamma 'verdict=red' || ! out_has '收不回'; then
    cell_failed pipeline-entry-1-still-red "放量入口退 1（不是 78）而没打 ITEM 行，仍应当红（收不回）" "看 run_pipeline_part 里 line 为空那一支：只有退 78 才记本次未跑"
  fi
  # pipeline-reuses-the-long-term-library：假的放量入口照键复用（判过的分项在假服务 STUB_AMP_SERVICE 里留一份，演 prune-server 里那一份判定库）⇒ 剪枝接连两趟：
  #   头一趟现判 20 态复用 0 态，第二趟现判 0 态复用 20 态（第二次跑复用了第一次判过的块），成功句照实报，服务里判过的留着
  local reuse_first_ok=0 reuse_first_total reuse_service="$work/stub-service"
  rm -rf -- "${reuse_service:?}"
  entry STUB_AMP_PERSISTENT=1 STUB_AMP_SERVICE="$reuse_service" --
  reuse_first_total="$(grep -E '^CRASH_POINT_REPLAY_TOTAL ' "$work/out")"
  if [[ "$exit_code" == 0 && "$reuse_first_total" == *" pipeline_states=20 pipeline_reused_known=0 "* ]] && out_has "放量现判 20 态复用 0 态"; then reuse_first_ok=1; fi
  entry STUB_AMP_PERSISTENT=1 STUB_AMP_SERVICE="$reuse_service" --
  if (( ! reuse_first_ok )) || [[ "$exit_code" != 0 ]] \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL .* pipeline_states=0 pipeline_reused_known=20 ' "$work/out" || ! out_has "放量现判 0 态复用 20 态" \
     || [[ ! -e "$reuse_service/alpha-amp.judged" || ! -e "$reuse_service/gamma-amp.judged" ]]; then
    cell_failed pipeline-reuses-the-long-term-library "剪枝那一趟第二次跑应当复用第一次判过的块：头一趟现判 20 态复用 0 态、第二趟现判 0 态复用 20 态，服务里判过的留着（头一趟：${reuse_first_total:-没有 TOTAL 行}）" \
      "看 run_pipeline_part 收 CRASH_AMPLIFICATION_ITEM 行那一段与文件头「三段」放量部分"
  fi
  rm -rf -- "${reuse_service:?}"
  # reuse-three-questions（scripts/gate/lib_reuse_checks.py 文件头「一」）：假的放量入口照键复用、键另带每个分项的单元文件（STUB_AMP_UNITS）⇒ 剪枝接连四趟：
  #   头一趟、同一份输入、加一份哪个分项都不读的单元文件（闭包外）、改 alpha-amp 的单元文件（闭包内）；从 ITEM 行取有放量那一段的项（alpha、gamma）复用还是现判
  #   （pipeline_states=0 而 pipeline_reused 是正整数是复用），照三问判、打规范输出。这一份入口不判复用（判定在引擎里）：核的是每一趟都不碰判定库（判过的照服务复用）、
  #   不因闭包外的改动重判，并照实把放量入口报的现判与复用写进 ITEM 行；pipeline-reuses-the-long-term-library 那一格是这里的 ①
  local three_units="$work/amp-units" three_exits=() three_round_name
  mkdir -p -- "$three_units"; printf 'alpha-amp 的单元\n' >"$three_units/alpha-amp"; printf 'gamma-amp 的单元\n' >"$three_units/gamma-amp"
  three_round() { # <名>
    entry STUB_AMP_PERSISTENT=1 STUB_AMP_SERVICE="$work/stub-service" STUB_AMP_UNITS="$three_units" --
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
      "照规范输出那一行 red 的那一问查：run_pipeline_part 有没有碰判定库，与 ITEM 行的 pipeline_states=、pipeline_reused= 取法"
  fi
  grep '^REUSE_SELFTEST ' "$work/three.log"
  rm -rf -- "${work:?}/stub-service"
  # ── 按条复用那几格（文件头「按条复用」）：小仓 unit-root 上真的 admission.py 算键、查库、记库；结局取每趟逐条打的「· <测试函数>：绿|红，现判|复用」那几行
  local unit_settings=(PATH="$work/unit-bin:$work/bin:$PATH" CRASH_POINT_REPLAY_ROOT="$unit_root" STUB_UNIT_ROOT="$unit_root")
  # 判定库在 prune-server 里（被判那棵树的 git common-dir 只定临时服务底下的命名空间），删不掉：要「库先清空」的格各给 unit-root 换一处新的 git 目录
  #   （unit_fresh_library：把它此刻的 git 目录整个拷到一处新地方，unit-root/.git 换成指过去的 gitdir 文件；内容、提交与索引照旧，common-dir 换了，
  #   这一份的库在临时服务里就是新的一个命名空间）
  local unit_library_root="" unit_library_count=0 unit_current_git
  unit_fresh_library() {
    unit_library_count=$((unit_library_count + 1))
    unit_library_root="$work/unit-library-$unit_library_count"
    unit_current_git="$(git -C "$unit_root" rev-parse --path-format=absolute --git-common-dir)" || return 1
    cp -R -- "$unit_current_git" "$unit_library_root" && rm -rf -- "${unit_root:?}/.git" && printf 'gitdir: %s\n' "$unit_library_root" >"$unit_root/.git" || return 1
    unit_settings=(PATH="$work/unit-bin:$work/bin:$PATH" CRASH_POINT_REPLAY_ROOT="$unit_root" STUB_UNIT_ROOT="$unit_root")
  }
  # unit_library_query count|red|plant-red <键> <测试函数>：门禁那一类在 prune-server 里的那份库（unit_library_root 这一处 git 目录的命名空间）按键前缀读：
  #   count 打 keys/ 下几条、red 打其中 verdict 是 red 的几条；plant-red 照 admission.py 判定库的写法（VerdictLibrary.record_red）造一条红
  unit_library_query() {
    python3 - "$replay_admission_module" "$unit_library_root" "$TEST_FUNCTIONS_GATE_FILE_NAME:test-functions" "$@" <<'PY_UNIT_LIBRARY'
import importlib.util, json, sys
specification = importlib.util.spec_from_file_location("admission_for_crash_point_replay_selftest", sys.argv[1])
admission = importlib.util.module_from_spec(specification)
specification.loader.exec_module(admission)
library = admission.VerdictLibrary(sys.argv[2], sys.argv[3])
action = sys.argv[4]
if action == "count":
    print(len(library.stored_keys()))
elif action == "red":
    print(sum(1 for _relative, value in library.store.scan("keys/") if json.loads(value).get("verdict") == "red"))
else:
    print(library.record_red("", sys.argv[6], sys.argv[5], "自证：造的一条红") or "recorded")
PY_UNIT_LIBRARY
  }
  local unit_alpha_reads_the_helper="crates/singlefs-verification/tests/alpha.rs::alpha_reads_the_helper" unit_alpha_stands_alone="crates/singlefs-verification/tests/alpha.rs::alpha_stands_alone"
  local unit_beta_stands_alone="crates/singlefs-verification/tests/beta.rs::beta_stands_alone" unit_exits=() unit_round_name unit_wrong unit_key unit_last_run
  local gate_request gate_kind gate_arguments=() request_tag request_value
  unit_states() { grep -E '^\s*· [^：]+：(绿|红)，' "$work/out" >"$work/unit-$1.out"; }   # <趟名>：这一趟逐条打的测试函数结局落进 unit-<趟名>.out
  unit_state_is() { grep -qF -- "· $2：$3" "$work/unit-$1.out"; }                        # <趟名> <测试函数> <绿，现判|绿，复用|红，现判>
  unit_count() { grep -cF -- "$2" "$work/unit-$1.out"; }                                   # <趟名> <一段字>：几条测试函数的结局里有它
  # reuse-three-questions 的测试函数那一半（scripts/gate/lib_reuse_checks.py 文件头「一」）：unit-root 上剪枝接连四趟：库是空的那一趟、同一份输入、
  #   改一个哪条测试都不引的函数 not_read_by_any_test（闭包外）、改 alpha_reads_the_helper 引的 alpha_helper（闭包内）；照三问判（--from-output）、打规范输出
  unit_fresh_library
  for unit_round_name in fresh same unrelated unit; do
    case "$unit_round_name" in
      unrelated) printf '%s\n' 'pub fn alpha_helper() -> u32 {' '    1' '}' '' 'pub fn not_read_by_any_test() -> u32 {' '    8' '}' >"$unit_crate/src/lib.rs" ;;
      unit) printf '%s\n' 'pub fn alpha_helper() -> u32 {' '    1 + 0' '}' '' 'pub fn not_read_by_any_test() -> u32 {' '    8' '}' >"$unit_crate/src/lib.rs" ;;
    esac
    entry "${unit_settings[@]}" --
    unit_exits+=("$exit_code")
    unit_states "three-$unit_round_name"
  done
  cp -- "$work/unit-lib.rs" "$unit_crate/src/lib.rs"
  if ! python3 "$replay_script_directory/../gate/lib_reuse_checks.py" three-questions --script scripts/crash/crash-point-replay.sh --first "$work/unit-three-fresh.out" \
         --second "$work/unit-three-same.out" --unrelated "$work/unit-three-unrelated.out" --unit-change "$work/unit-three-unit.out" --affected "$unit_alpha_reads_the_helper" --from-output \
         >"$work/unit-three.log" 2>&1 || [[ "${unit_exits[*]}" != "0 0 0 0" ]]; then
    cell_failed reuse-three-questions "按条复用的复用三问没过（四趟退 ${unit_exits[*]}）：$(tr '\n' ' ' <"$work/unit-three.log" | cut -c1-600)" \
      "照规范输出那一行 red 的那一问查：run_test_functions 里 plan_test_functions（test-plan）、test_functions_target_skips（--skip）与 record_test_functions_target（verdict-record）"
  fi
  grep '^REUSE_SELFTEST ' "$work/unit-three.log"
  # test-functions-reuse-shared-with-the-gate：unit-root 上（库先清空）逐条点名三项的一趟把四条测试函数判绿写回门禁那一类判定库（keys/ 下恰好四份）；同一份输入不带参数的那一趟
  #   四条都复用、alpha、beta、gamma_amp 都不编不起；门禁不带特性那一趟的计划（真门禁取种类与请求的 test-functions-request、admission.py test-plan、不带 --feature）四条都是 reuse；
  unit_wrong=""
  unit_fresh_library
  entry "${unit_settings[@]}" -- --item alpha --item beta --item gamma
  unit_states shared-named
  [[ "$exit_code" == 0 && "$(unit_count shared-named '，现判')" == 4 && "$(unit_library_query count)" == 4 ]] \
    || unit_wrong+="逐条那一趟退 $exit_code、现判 $(unit_count shared-named '，现判') 条、门禁那一类的库（prune-server 里 $unit_library_root 的命名空间）keys/ 下 $(unit_library_query count) 条（要退 0、4 条、4 条）；"
  entry "${unit_settings[@]}" --
  unit_states shared-pruned
  [[ "$(unit_count shared-pruned '：绿，复用')" == 4 ]] && ! calls_have '^cargo test .*--test (alpha|beta|gamma_amp)( |$)' \
    || unit_wrong+="剪枝那一趟复用 $(unit_count shared-pruned '：绿，复用') 条（要 4 条，alpha、beta、gamma_amp 都不编不起）；"
  # 直接起的两条 admission.py 与 entry 同一份多机环境：起活那一方的回读握手不带进来，多机配置指 entry 那一份（第二台那一趟由 multi-host-run.sh 带上本台的两样）
  gate_request="$(env -u GPU_QUIESCE_ORIGIN_READ_BACK SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" \
    python3 "$replay_admission_module" test-functions-request "$TEST_FUNCTIONS_GATE_FILE_NAME" "$replay_judge_root" 2>&1)"
  gate_kind=""; gate_arguments=()
  while IFS=$'\t' read -r request_tag request_value; do
    case "$request_tag" in
      kind) gate_kind="$request_value" ;;
      argument) gate_arguments+=("$request_value") ;;
    esac
  done <<<"$gate_request"
  grep -q 'scripts/gate/admission.py" test-functions-request' "$replay_judge_root/.claude/gate.d/crash-point-replay.sh" \
    || unit_wrong+="门禁 crash-point-replay 的种类与请求不取 admission.py test-functions-request；"
  env -u GPU_QUIESCE_ORIGIN_READ_BACK SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" \
    PATH="$work/unit-bin:$work/bin:$PATH" python3 "$replay_admission_module" test-plan "$unit_root" "$gate_kind" singlefs-verification "$work/unit-gate.plan" \
    ${gate_arguments[@]+"${gate_arguments[@]}"} >"$work/unit-gate.plan.out" 2>&1
  [[ "$(awk -F'\t' 'NF >= 7 && $2 == "reuse"' "$work/unit-gate.plan.out" | grep -c .)" == 4 ]] \
    || unit_wrong+="门禁的计划（种类 $gate_kind）没把四条都复用：$(grep '^PLAN ' "$work/unit-gate.plan.out")；"
  [[ -z "$unit_wrong" ]] || cell_failed test-functions-reuse-shared-with-the-gate "逐条写进的判定，同一份输入不带参数的与门禁的计划应当照键复用（同一类、同一份库）：$unit_wrong" \
    "看 plan_test_functions 取的种类（admission.py test-functions-request）、record_test_functions_target 的 verdict-record 与 test_functions_target_all_reused"
  # item-plan-lines：--list-items --with-keys 每项一行 ITEM_PLAN（文件头「输出」）。unit-root 上库先清空、逐条记库一趟，再一趟（拿这一趟 test-plan 的计划当真值：几条复用、几条现判），
  #   再 --list-items --with-keys：alpha、beta、gamma 三项的 reusable 之和等于那一趟计划里复用的条数，to_judge 之和等于 4 件（崩溃枚举用例 2 + 放量分项 2）加那一趟计划里要现判的条数；
  #   整趟才跑的三类各一行；--item beta 只打 beta 一行；--list-items 带 --item 不带 --with-keys 退 2
  local plan_wrong="" plan_run_directory plan_reuse_truth plan_judge_truth plan_reusable_sum plan_to_judge_sum plan_list_lines plan_item_lines
  unit_fresh_library
  entry "${unit_settings[@]}" --
  entry "${unit_settings[@]}" --
  plan_run_directory="$(ls -td "$work/home/runs"/crash-point-replay-* 2>/dev/null | head -n 1)"
  plan_reuse_truth="$(awk -F'\t' 'NF >= 7 && $2 == "reuse"' "$plan_run_directory/test-functions.plan.out" 2>/dev/null | grep -c .)"
  plan_judge_truth="$(awk -F'\t' 'NF >= 7 && ($2 == "judge" || $2 == "refused")' "$plan_run_directory/test-functions.plan.out" 2>/dev/null | grep -c .)"
  entry "${unit_settings[@]}" CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys
  [[ "$exit_code" == 0 ]] || plan_wrong+="--list-items --with-keys 退 $exit_code；"
  plan_list_lines="$(grep '^ITEM_PLAN ' "$work/out")"
  plan_reusable_sum="$(awk '/^ITEM_PLAN item=(alpha|beta|gamma) / { for (i = 1; i <= NF; i++) if ($i ~ /^reusable=/) { sub(/^reusable=/, "", $i); sum += $i } } END { print sum + 0 }' <<<"$plan_list_lines")"
  plan_to_judge_sum="$(awk '/^ITEM_PLAN item=(alpha|beta|gamma) / { for (i = 1; i <= NF; i++) if ($i ~ /^to_judge=/) { sub(/^to_judge=/, "", $i); sum += $i } } END { print sum + 0 }' <<<"$plan_list_lines")"
  [[ "$(grep -c . <<<"$plan_list_lines")" == 6 ]] && grep -qE '^ITEM_PLAN item=whole-run:pinned-red-examples to_judge=0 reusable=0$' <<<"$plan_list_lines" \
    || plan_wrong+="应当 6 项各一行（alpha、beta、gamma 与整趟才跑的三类），钉回来的红例 0、0；"
  (( plan_reusable_sum == plan_reuse_truth )) || plan_wrong+="reusable 之和 $plan_reusable_sum 与计划里复用的 $plan_reuse_truth 条不同；"
  (( plan_to_judge_sum == 4 + plan_judge_truth )) || plan_wrong+="to_judge 之和 $plan_to_judge_sum 应当是 4 件加计划里现判的 $plan_judge_truth 条；"
  entry "${unit_settings[@]}" CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys --item beta
  plan_item_lines="$(grep '^ITEM_PLAN ' "$work/out")"
  { [[ "$exit_code" == 0 && "$(grep -c . <<<"$plan_item_lines")" == 1 ]] && grep -q '^ITEM_PLAN item=beta ' <<<"$plan_item_lines"; } \
    || plan_wrong+="--item beta 应当只打 beta 一行（退 $exit_code，打了 $(grep -c . <<<"$plan_item_lines") 行）；"
  entry "${unit_settings[@]}" -- --list-items --item beta
  [[ "$exit_code" == 2 ]] || plan_wrong+="--list-items --item beta（不带 --with-keys）应当退 2（退 $exit_code）；"
  [[ -z "$plan_wrong" ]] || cell_failed item-plan-lines "--list-items --with-keys 应当每项一行 ITEM_PLAN、条数照 test-plan 的计划数、点名了只算点名的项：$plan_wrong" \
    "看 report_the_item_plan 与 emit_item_plan_lines，与 test-plan 对测试函数那一段的计划"
  # shard-plan-lines：按件分片的计划那一步另打 ITEM_PLAN shard=<片号> item=…（文件头「输出」）：每片每项一行（同一片同一项不出两行）、alpha 的两件落在不同的片上，
  #   各片各项之和等于上面不分片那一趟 --list-items --with-keys 的 ITEM_PLAN（一项的测试函数那一段只记在它头一次出现的那一片）
  local shard_lines_wrong="" shard_lines_sum list_lines_sum shard_lines_awk
  shard_lines_awk='{ item = ""; judge = 0; reuse = 0; for (i = 2; i <= NF; i++) { split($i, pair, "="); if (pair[1] == "item") item = pair[2]; if (pair[1] == "to_judge") judge = pair[2]; if (pair[1] == "reusable") reuse = pair[2] } j[item] += judge; r[item] += reuse } END { for (item in j) print item, j[item], r[item] }'
  default_shard_plan "${unit_settings[@]}" --
  [[ "$exit_code" == 0 ]] || shard_lines_wrong+="计划那一步退 $exit_code；"
  [[ "$(awk '/^ITEM_PLAN shard=/ { print $2, $3 }' "$work/out" | LC_ALL=C sort | uniq -d | grep -c .)" == 0 ]] || shard_lines_wrong+="同一片同一项出了两行；"
  [[ "$(awk '/^ITEM_PLAN shard=[0-9]+ item=alpha / { print $2 }' "$work/out" | LC_ALL=C sort -u | grep -c .)" -ge 2 ]] || shard_lines_wrong+="alpha 的两件应当落在两片、各一行；"
  shard_lines_sum="$(awk "/^ITEM_PLAN shard=/ $shard_lines_awk" "$work/out" | LC_ALL=C sort)"
  list_lines_sum="$(awk "/^ITEM_PLAN item=/ $shard_lines_awk" <<<"$plan_list_lines" | LC_ALL=C sort)"
  { [[ -n "$shard_lines_sum" ]] && [[ "$shard_lines_sum" == "$list_lines_sum" ]]; } \
    || shard_lines_wrong+="各片各项之和「$(tr '\n' '|' <<<"$shard_lines_sum")」与不分片那一趟「$(tr '\n' '|' <<<"$list_lines_sum")」不同；"
  [[ -z "$shard_lines_wrong" ]] || cell_failed shard-plan-lines "分片计划行应当每片每项一行、条数之和等于不分片那一趟：$shard_lines_wrong" "看 shard_report_the_item_plan 与 emit_item_plan_lines 的 shard 模式"
  # red-rejudged：红过的不当绿复用（库先清空）：① 逐条一趟 alpha_reads_the_helper 判红（STUB_RED_TESTS）、alpha_stands_alone 判绿 ⇒ 再一趟（不再红）alpha_reads_the_helper 照现判、alpha_stands_alone 复用；
  #   ② beta_stands_alone 的键在门禁那一类的库里记着一条红（照 admission.py 判定库的写法造一份 verdict=red 的 keys/<键>.json）⇒ 下一趟 beta_stands_alone 照现判
  unit_wrong=""
  unit_fresh_library
  entry "${unit_settings[@]}" STUB_RED_TESTS=alpha_reads_the_helper -- --item alpha
  unit_states red-failing
  unit_state_is red-failing "$unit_alpha_reads_the_helper" '红，现判' || unit_wrong+="判红的那一趟 alpha_reads_the_helper 没判红；"
  unit_key="$(awk -F'\t' -v name="$unit_beta_stands_alone" '$1 == name { print $3 }' "$(ls -td "$work/home/runs"/crash-point-replay-* 2>/dev/null | head -n 1)/test-functions.plan.out" 2>/dev/null)"
  entry "${unit_settings[@]}" -- --item alpha
  unit_states red-again
  unit_state_is red-again "$unit_alpha_reads_the_helper" '绿，现判' || unit_wrong+="再一趟 alpha_reads_the_helper 没照现判（$(grep -F "$unit_alpha_reads_the_helper" "$work/unit-red-again.out")）；"
  unit_state_is red-again "$unit_alpha_stands_alone" '绿，复用' || unit_wrong+="再一趟 alpha_stands_alone 没复用（$(grep -F "$unit_alpha_stands_alone" "$work/unit-red-again.out")）；"
  if [[ "$unit_key" =~ ^[0-9a-f]{64}$ ]]; then
    unit_library_query plant-red "$unit_key" "$unit_beta_stands_alone" >/dev/null || unit_wrong+="造那一条红没写进临时服务里的库；"
    entry "${unit_settings[@]}" -- --item beta
    unit_states red-planted
    unit_state_is red-planted "$unit_beta_stands_alone" '绿，现判' || unit_wrong+="库里记着红的 beta_stands_alone 没照现判（$(grep -F "$unit_beta_stands_alone" "$work/unit-red-planted.out")）；"
  else
    unit_wrong+="判红那一趟的计划里取不出 beta_stands_alone 的键（「$unit_key」）；"
  fi
  [[ -z "$unit_wrong" ]] || cell_failed red-rejudged "红过的不当绿复用、下一趟照现判：$unit_wrong" \
    "看 record_test_functions_target（判绿的写回、判红的记红）与 scripts/gate/admission.py 的 VerdictLibrary.read（只认判绿的记录）、lookup 里认红的那一句"
  # red-revokes-green：记红、撤绿、下趟现判（文件头「按条复用」；库先清空）：逐条一趟 alpha 两条判绿入库 ⇒ 同键现判那一趟（准入模块的弄坏开关 verdict-lookup-skipped 不读库，
  #   造「同一把键又判了一遍」）alpha_reads_the_helper 判红（STUB_RED_TESTS）、照计划里的键记红、撤掉那把键的绿（判定库那一类里有一份 verdict 是 red 的）⇒
  #   下一趟逐条 alpha_reads_the_helper 照现判、alpha_stands_alone 复用
  unit_wrong=""
  unit_fresh_library
  entry "${unit_settings[@]}" -- --item alpha
  entry "${unit_settings[@]}" ADMISSION_BREAK=verdict-lookup-skipped STUB_RED_TESTS=alpha_reads_the_helper -- --item alpha
  unit_states red-rejudged-same-key
  unit_state_is red-rejudged-same-key "$unit_alpha_reads_the_helper" '红，现判' || unit_wrong+="同键现判那一趟 alpha_reads_the_helper 没判红；"
  (( $(unit_library_query red || echo 0) > 0 )) || unit_wrong+="判定库（prune-server 里 $unit_library_root 的命名空间）keys/ 下没有记红的那一条；"
  entry "${unit_settings[@]}" -- --item alpha
  unit_states red-after-rejudge
  unit_state_is red-after-rejudge "$unit_alpha_reads_the_helper" '绿，现判' || unit_wrong+="判红之后的那一趟 alpha_reads_the_helper 没照现判（$(grep -F "$unit_alpha_reads_the_helper" "$work/unit-red-after-rejudge.out")）；"
  unit_state_is red-after-rejudge "$unit_alpha_stands_alone" '绿，复用' || unit_wrong+="判红之后的那一趟 alpha_stands_alone 没复用（$(grep -F "$unit_alpha_stands_alone" "$work/unit-red-after-rejudge.out")）；"
  [[ -z "$unit_wrong" ]] || cell_failed red-revokes-green "判红要记红、撤掉同一把键原有的绿，下一趟照现判：$unit_wrong" \
    "看 record_test_functions_target 交 verdict-record 的 --red-names 与 scripts/gate/admission.py 的 VerdictLibrary.record_red、lookup 里认红的那一句"
  # test-functions-outside-git-exits-2（文件头「按条复用」）：照 unit-root 拷一份不带 .git 的副本（临时目录里，往上找不到别的仓），在副本上跑剪枝
  #   ⇒ 整趟退 2、点名不是 git 工作树的顶、测试函数一个测试目标都不编不起（弄坏开关 test-functions-without-library-tolerated 下照改前全部现判、往下跑）
  local unit_copy="$work/unit-copy"
  rm -rf -- "$unit_copy"
  cp -dR --preserve=timestamps,links -- "$unit_root" "$unit_copy" && rm -rf -- "${unit_copy:?}/.git"
  entry PATH="$work/unit-bin:$work/bin:$PATH" CRASH_POINT_REPLAY_ROOT="$unit_copy" STUB_UNIT_ROOT="$unit_copy" --
  if [[ "$exit_code" != 2 ]] || ! out_has "不是一棵 git 工作树的顶" || calls_have '^cargo test '; then
    cell_failed test-functions-outside-git-exits-2 "被判的根不是 git 工作树的顶时测试函数应当整趟退 2、点名它、一个测试目标都不编不起（退 $exit_code）" \
      "看 plan_test_functions 里 nolibrary 那一支与 run_test_functions 里照它退 2 那一段"
  fi
  rm -rf -- "${unit_copy:?}"
  # test-functions-domain-environment-keyed（文件头「按条复用」test-functions-request 在测试函数 cargo 的环境里起）：alpha_helper 改成读按段登记文件与片长两个变量（原文里有两个名字的字面串），
  #   库先清空：逐条一趟 ⇒ 同一份输入再一趟 alpha 两条都复用 ⇒ 按段登记那一行的摘要换一个 ⇒ alpha_reads_the_helper 现判、alpha_stands_alone 复用 ⇒
  #   调用方设片长 131072 ⇒ alpha_reads_the_helper 又现判、alpha_stands_alone 复用
  unit_wrong=""
  printf '%s\n' 'pub fn alpha_helper() -> u32 {' \
    '    let _ = (std::env::var_os("SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY"), std::env::var_os("SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES"));' \
    '    1' '}' '' 'pub fn not_read_by_any_test() -> u32 {' '    7' '}' >"$unit_crate/src/lib.rs"
  cp -- "$unit_root/.claude/gate.d/crash-point-replay-segments.tsv" "$work/unit-segments-plain.tsv"
  unit_fresh_library
  entry "${unit_settings[@]}" -- --item alpha
  entry "${unit_settings[@]}" -- --item alpha
  unit_states domain-same
  [[ "$(unit_count domain-same '：绿，复用')" == 2 ]] || unit_wrong+="同一份输入再一趟 alpha 复用 $(unit_count domain-same '：绿，复用') 条（要 2 条）；"
  printf '%s\n' "# 自证的按段登记" "alpha	alpha_flow	0	7	$(printf 'fedcba9876543210%.0s' 1 2 3 4)" >"$unit_root/.claude/gate.d/crash-point-replay-segments.tsv"
  entry "${unit_settings[@]}" -- --item alpha
  unit_states domain-registry
  unit_state_is domain-registry "$unit_alpha_reads_the_helper" '绿，现判' || unit_wrong+="按段登记文件改了，读它的 alpha_reads_the_helper 没现判（$(grep -F "$unit_alpha_reads_the_helper" "$work/unit-domain-registry.out")）；"
  unit_state_is domain-registry "$unit_alpha_stands_alone" '绿，复用' || unit_wrong+="按段登记文件改了，不读它的 alpha_stands_alone 没复用；"
  entry "${unit_settings[@]}" SINGLEFS_CRASH_REGISTERED_SEGMENT_SLICE_STATES=131072 -- --item alpha
  unit_states domain-slice
  unit_state_is domain-slice "$unit_alpha_reads_the_helper" '绿，现判' || unit_wrong+="片长没设换成 131072，读它的 alpha_reads_the_helper 没现判（$(grep -F "$unit_alpha_reads_the_helper" "$work/unit-domain-slice.out")）；"
  unit_state_is domain-slice "$unit_alpha_stands_alone" '绿，复用' || unit_wrong+="片长换了，不读它的 alpha_stands_alone 没复用；"
  cp -- "$work/unit-lib.rs" "$unit_crate/src/lib.rs"
  cp -- "$work/unit-segments-plain.tsv" "$unit_root/.claude/gate.d/crash-point-replay-segments.tsv"
  [[ -z "$unit_wrong" ]] || cell_failed test-functions-domain-environment-keyed "按段登记文件的内容、片长要进闭包读它们的那几条的键：$unit_wrong" \
    "看 plan_test_functions 起 test-functions-request 的那一份环境（要是交给测试函数 cargo 的同一份）与 scripts/gate/admission.py 的 domain_variable_tokens、domain_variable_lines"
  # test-functions-plan-failure-red：unit-root 放 .unit-verdict-sample/plan-failure 写 exit-1（admission.py 文件头「按条复用失灵」：test-plan 不算键、退 1）⇒ 没有计划就排不掉登记的用例，
  #   测试函数那一段一个测试目标都不起，点到测试目标的项（alpha、gamma 都在里面）都红、整趟退 1、原因写按条复用失灵
  printf 'exit-1\n' >"$unit_root/.unit-verdict-sample/plan-failure"
  entry "${unit_settings[@]}" --
  rm -f -- "$unit_root/.unit-verdict-sample/plan-failure"
  if [[ "$exit_code" != 1 ]] || ! item_has alpha 'verdict=red' || ! item_has gamma 'verdict=red' || ! out_has '按条复用失灵' \
     || calls_have '^cargo test .*--test alpha -- --nocapture'; then
    cell_failed test-functions-plan-failure-red "按条复用失灵（test-plan 退 1）应当不起测试目标（没有计划排不掉登记的用例）、点到测试目标的项都红、整趟退 1、原因写按条复用失灵" \
      "看 plan_test_functions 里交 admission.py plan-failure 那一段与 run_test_functions 末尾 functions_plan_state=failed 那一支"
  fi
  # test-functions-plan-unavailable：unit-root 放 .unit-verdict-sample/plan-failure 写 exit-75（admission.py 文件头「按条复用失灵」：test-plan 装 prune-server 不可用、退 75）
  #   ⇒ 整趟退 75、说 prune-server 不可用、不说按条复用失灵，测试函数一个测试目标都不编不起（弄坏开关 test-functions-plan-unavailable-as-failure 下照失灵办、退 1）
  printf 'exit-75\n' >"$unit_root/.unit-verdict-sample/plan-failure"
  entry "${unit_settings[@]}" --
  rm -f -- "$unit_root/.unit-verdict-sample/plan-failure"
  if [[ "$exit_code" != 75 ]] || ! out_has 'prune-server 不可用' || out_has '按条复用失灵' || calls_have '^cargo test '; then
    cell_failed test-functions-plan-unavailable "测试函数 test-plan 退 75（prune-server 不可用）应当整趟退 75、说不可用、不判按条复用失灵、一个测试目标都不编（实测退 $exit_code）" \
      "看 plan_test_functions 里 plan-failure 退 75 那一支（functions_plan_state=unavailable）与 run_test_functions 里照它退 75 那一段"
  fi
  # registered-excluded：登记成 crash-case: 的用例（stage-inputs.tsv 的 crash-case:alpha 登记了 alpha_full）与放量分项（ITEMS 的 alpha-amp 登记了 alpha_amplified）不进测试函数那一段：
  #   unit-root 的 alpha.rs 临时加这两条，逐条点名 alpha ⇒ 它们经 --skip 排掉（cargo 参数里有 --skip alpha_full 与 --skip alpha_amplified）、不判不记（逐条结局里没有它们）、
  #   alpha_stands_alone 照判（弄坏开关 registered-not-excluded 下它们也被判，参数里没有 --skip）
  cp -- "$unit_crate/tests/alpha.rs" "$work/unit-alpha.rs"
  printf '\n#[test]\nfn alpha_full() {\n    assert_eq!(5, 5);\n}\n\n#[test]\nfn alpha_amplified() {\n    assert_eq!(6, 6);\n}\n' >>"$unit_crate/tests/alpha.rs"
  unit_fresh_library
  entry "${unit_settings[@]}" -- --item alpha
  unit_states registered
  if [[ "$exit_code" != 0 ]] || ! calls_have '^cargo test .*--test alpha .*--skip alpha_amplified --skip alpha_full' \
     || grep -qE 'alpha_(full|amplified)' "$work/unit-registered.out" || ! unit_state_is registered "$unit_alpha_stands_alone" '绿，现判'; then
    cell_failed registered-excluded "按项跑时登记成 crash-case: 的用例与放量分项应当经 --skip 排掉、不进测试函数那一段的判定与逐条结局（退 $exit_code；调用 $(grep '^cargo test .*--test alpha ' "$calls" | cut -c1-200)）" \
      "看 plan_test_functions 交给 test-plan 的 --exclude-registered 与 test_functions_target_skips 取的 plan-skip-names（admission.py 把 registered 的行也列进去）"
  fi
  cp -- "$work/unit-alpha.rs" "$unit_crate/tests/alpha.rs"
  # last-run-list（scripts/gate/lib_reuse_checks.py 文件头「三」）：剪枝一趟 gamma 那一项放量红，上次红的清单是 gamma；再一趟 gamma 的放量本次未跑（gamma 那一项没判），
  #   清单照留 gamma；再一趟全绿，清单成空文件
  local last_run_file="$work/last-runs/scripts__crash__crash-point-replay.sh.tsv" last_red
  rm -f -- "$last_run_file"
  entry STUB_AMP_RED=gamma-amp --
  last_red="$(cat "$last_run_file" 2>/dev/null)"
  entry STUB_AMP_NOT_RUN=gamma-amp --
  unit_last_run="$(cat "$last_run_file" 2>/dev/null)"
  entry --
  if [[ "$last_red" != gamma || "$unit_last_run" != gamma || ! -f "$last_run_file" || -s "$last_run_file" ]]; then
    cell_failed last-run-list "上次红的清单不对：gamma 红的那一趟之后是「$last_red」，gamma 本次未跑的那一趟之后是「$unit_last_run」（要照留 gamma），全绿那一趟之后$([[ -f "$last_run_file" ]] && echo "是「$(cat "$last_run_file")」" || echo 没有这份文件)" \
      "看收尾写 last-run 那一段：summarize 交出的红的项（exit.red-items）与这一趟真判过的项（exit.judged-items），经 --ran 只换判过的项"
  fi
  # test-functions-reuse-counted：测试函数 alpha 那个测试目标的日志里有复用计数行（resumed_slices=3）⇒ alpha 那一行 test_functions_reused=3；beta 没有那一行 ⇒ unknown、计进取不出的、成功句点名
  entry STUB_REUSE_TARGETS=alpha --
  if [[ "$exit_code" != 0 ]] || ! item_has alpha 'test_functions_reused=3 test_functions_reuse_unknown=0 ' || ! item_has beta 'test_functions_reused=unknown test_functions_reuse_unknown=1 ' \
     || ! grep -qE '^CRASH_POINT_REPLAY_TOTAL .* test_functions_reused_known=3 test_functions_reuse_unknown=[1-9][0-9]* ' "$work/out" \
     || ! out_has "测试函数 " || ! grep -qE '复用 3 块（另有 [0-9]+ 个测试目标取不出复用数：[^）]*beta' "$work/out"; then
    cell_failed test-functions-reuse-counted "测试函数日志里有复用计数行时 ITEM 行应当报同一个数（alpha：test_functions_reused=3），没有时报取不出（beta：test_functions_reused=unknown、计进 test_functions_reuse_unknown）、成功句点名取不出的测试目标" \
      "看 judge_test_functions_log 里数 resumed_slices= 那一段与 summarize 的 functions_reused"
  fi
  # test-variables-outside-selftest：不带自证标记、带了 CRASH_POINT_REPLAY_ROOT ⇒ 退 2、什么都不起
  : >"$calls"; cells=$((cells + 1))
  env PATH="$work/bin:$PATH" STUB_CALLS="$calls" SINGLEFS_CRASH_AMPLIFICATION_HOME="$work/home" SINGLEFS_LAST_RUNS_DIRECTORY="$work/last-runs" \
    SINGLEFS_MULTI_HOST_CONFIG="$work/multi-host-gpu-off.env" CRASH_POINT_REPLAY_ROOT="$root" \
    CRASH_POINT_REPLAY_BREAK="$replay_break" bash "$replay_script" >"$work/out" 2>&1
  exit_code=$?
  [[ "$exit_code" == 2 && ! -s "$calls" ]] || cell_failed test-variables-outside-selftest "平常跑时带了只供自证的变量应当退 2" "看开头拒只供自证变量那一段"
  # 往引擎送的变量（文件头「往引擎送的变量」）：测试函数每一趟 cargo test 与放量入口那一趟都要带到
  local test_functions_runs_pattern='^cargo test --release -p singlefs-verification --test [a-z0-9_]+ -- --nocapture '
  local registry="$root/.claude/gate.d/crash-point-replay-segments.tsv"
  every_test_functions_run_has() { # <那一行里要有的一段（正则）>：测试函数至少跑了一趟，且每一趟 cargo test 的调用记录都有它
    local all
    all="$(calls_count "$test_functions_runs_pattern")"
    (( all > 0 )) && [[ "$(calls_count "${test_functions_runs_pattern}.*$1")" == "$all" ]]
  }
  # segments-registry-sent：每一趟都带按段登记文件的绝对路径；这一趟另设 CARGO_BUILD_JOBS=3 当服务交的线程数，下面 test-functions-threads-from-service 判同一趟
  entry CARGO_BUILD_JOBS=3 --
  if [[ "$exit_code" != 0 ]] || ! every_test_functions_run_has " segments=\[$registry\]\$" \
     || ! calls_have "^pipeline-env segments=\[$registry\]\$"; then
    cell_failed segments-registry-sent "测试函数与放量每一趟都应当带按段登记文件的绝对路径（SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY=$registry）" \
      "看「往引擎送的变量」那一段的 export SINGLEFS_CRASH_POINT_REPLAY_SEGMENTS_REGISTRY"
  fi
  # test-functions-threads-from-service：测试函数每一趟 cargo 直接起、不套线程包装，调用方环境里服务交的 CARGO_BUILD_JOBS（上一趟设的 3）原样带到
  if [[ "$exit_code" != 0 ]] || ! every_test_functions_run_has " jobs=\[3\] segments="; then
    cell_failed test-functions-threads-from-service "测试函数每一趟 cargo 都应当原样收到服务交的线程数（调用方环境 CARGO_BUILD_JOBS=3）" \
      "看 run_test_functions 拼的 env：不清掉、不改写 CARGO_BUILD_JOBS 与 RUST_TEST_THREADS，cargo 前面不套线程包装"
  fi
  # segments-registry-refused：按段登记文件核不过（项名不在合表里）⇒ 退 2、什么都不起
  cp -- "$registry" "$work/segments-saved.tsv"
  printf '%s\n' "# 自证：核不过的按段登记" "ghost	ghost_flow	0	7	$(printf '0123456789abcdef%.0s' 1 2 3 4)" >"$registry"
  entry --
  cp -- "$work/segments-saved.tsv" "$registry"
  if [[ "$exit_code" != 2 || -s "$calls" ]] || ! out_has "项名 ghost 不是合表"; then
    cell_failed segments-registry-refused "按段登记文件核不过应当退 2、什么都不起、逐处列出问题" \
      "看「往引擎送的变量」那一段开头的 lib_crash_point_replay_items.py segments"
  fi
  # test-functions-run-condition-not-met：alpha 那个测试目标的日志里有给定状态表借不到显存那一行（cargo 退 101）⇒ alpha 记本次未跑、写原因、退 78，别的照绿
  entry STUB_RUN_CONDITION_TARGETS=alpha --
  if [[ "$exit_code" != 78 ]] || ! item_has alpha 'exit=78 verdict=not-run ' || ! item_has beta 'verdict=green' \
     || ! out_has "本次未跑 alpha：运行条件不满足：给定状态表的判器借不到显存（$CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX table=stub-table"; then
    cell_failed test-functions-run-condition-not-met "测试函数日志里有给定状态表借不到显存那一行（$CRASH_AMPLIFICATION_GIVEN_STATES_RUN_CONDITION_LINE_PREFIX），那一项应当记本次未跑、写原因，整趟退 78" \
      "看 judge_test_functions_log 里 condition_lines 那一支与 summarize 的 any_78"
  fi
  # flow-coverage-table：剪枝跑完，这一趟的日志目录里落了覆盖表与记录（scripts/crash/crash-flow-coverage.py 的 run-directory、table），测试函数打的覆盖行在里面
  entry STUB_FLOW_LINE=1 --
  local flow_run_directory
  flow_run_directory="$(ls -td "$work/home/runs"/crash-point-replay-* 2>/dev/null | head -n 1)"
  if [[ "$exit_code" != 0 || -z "$flow_run_directory" ]] || ! grep -q '| selftest_entry' "$flow_run_directory/crash-flow-coverage.md" 2>/dev/null \
     || ! grep -q $'^CRASH_FLOW_RECORD\tcoverage\talpha\tselftest_entry\t' "$flow_run_directory/crash-flow-records.tsv" 2>/dev/null; then
    cell_failed flow-coverage-table "跑完应当在这一趟的日志目录里落覆盖表 crash-flow-coverage.md 与记录 crash-flow-records.tsv，测试函数打的覆盖行在里面（日志目录 ${flow_run_directory:-没找到}）" \
      "看 summarize 之后落覆盖表那一段与 scripts/crash/crash-flow-coverage.py 的 run-directory、table"
  fi
  # rocksdb-entry-refused：小仓的 验证包依赖 rocksdb、进程环境里四个取值是空的，PATH 最前面的假 cargo 不带 rocksdb-prebuilt.py 认的标记
  #   ⇒ 起任何 cargo 之前 rocksdb-prebuilt.py entry 判红、退 78，假 cargo 与两套驱动一次都没起（别的格小仓里这份清单没有 rocksdb 依赖，entry 报不判；跑完照底稿换回）
  printf '%s\n' '[package]' 'name = "singlefs-verification"' 'version = "0.0.0"' '[dependencies]' 'rocksdb = { version = "=0.25.0", optional = true }' \
    >"$root/crates/singlefs-verification/Cargo.toml"
  entry ROCKSDB_LIB_DIR= ROCKSDB_STATIC= SNAPPY_LIB_DIR= SNAPPY_STATIC= --
  if [[ "$exit_code" != 78 ]] || [[ -s "$calls" ]] || ! out_has "rocksdb-prebuilt entry：" || ! out_has "这一趟不起 cargo"; then
    cell_failed rocksdb-entry-refused "验证包依赖 rocksdb 而进程环境里没有预编库的取值，应当在起 cargo 之前退 78、一个 cargo 与驱动都不起、输出里有 rocksdb-prebuilt entry 的红句" \
      "看定档之后 replay_rocksdb_entry_holds 那一段（scripts/hosts/rocksdb-prebuilt.py entry）"
  fi
  cp -- "$work/root-crate-manifest.toml" "$root/crates/singlefs-verification/Cargo.toml"

  # ── 指定范围的崩溃域上限（文件头「不带 --item 的跑法…」；估法与记录来源在 lib_crash_point_replay_items.py 文件头「指定范围的崩溃域上限」）
  #   小仓里 gamma 只有放量分项 gamma-amp，beta 只有崩溃点重放 crash-case:beta，alpha 两样都有。假 cargo 的读库程序 item 子命令按 STUB_ITEM_RECORDED、STUB_ITEM_TO_JUDGE、
  #   STUB_ITEM_REUSABLE 答块账（不给是录下的块全部可复用）；放量分项的运行记录放 $work/home/runs/entry-scoped-*/<分项>.log（共享根里同一处），每格要的记录建在格里，这一段末了删
  local scoped_runs="$work/home/runs" scoped_wrong scoped_run_directory scoped_scope_file domain_lines domain_order
  scoped_record() { # <目录名> <分项> <展开深度> <态数> <几天前>：放一份放量分项的运行记录（一行 CRASH_AMPLIFICATION，修改时刻照几天前定，新旧次序靠它）
    mkdir -p -- "$scoped_runs/$1"
    printf 'CRASH_AMPLIFICATION flow=%s mode=single_host_cpu states_checked_here=0 blocks_checked_here=0 unjudged_blocks=0 red=0 states_reused=%s exhaustive=true expand_up_to=%s\n' \
      "$2" "$4" "$3" >"$scoped_runs/$1/$2.log"
    touch -d "$5 days ago" -- "$scoped_runs/$1/$2.log"
  }
  # scoped-domain-over-limit：gamma 上一趟不缩域真跑 300000 态（展开深度 30，较新）、更早缩到 6 的那一趟 225 态，库里 4 块键都对不上 ⇒ 估 300000 态，超上限退 79，
  #   说明写能压到上限以内的 n（6）与两条出路，不起放量入口、门禁与 cargo test
  scoped_wrong=""
  scoped_record entry-scoped-old gamma-amp 6 225 2
  scoped_record entry-scoped-new gamma-amp 30 300000 1
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=4 STUB_ITEM_REUSABLE=0 -- --item gamma
  [[ "$exit_code" == 79 ]] || scoped_wrong+="估 300000 态超上限应当退 79（退 $exit_code）；"
  out_has "300000" || scoped_wrong+="说明里应当写估的态数；"
  out_has "--crash-expand-up-to 6" || scoped_wrong+="说明里应当写能压到上限以内的 --crash-expand-up-to 6；"
  out_has "留给提交时不指定范围的那一趟" || scoped_wrong+="说明里应当写第二条出路（留给提交时不指定范围的那一趟）；"
  if calls_have '^(pipeline-entry|enumeration-gate|cargo test) '; then scoped_wrong+="拒了之后不该起放量入口、门禁与 cargo test；"; fi
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-over-limit "带 --item、没缩域，估数超过 200000 态应当退 79、说明里写 --crash-expand-up-to 6 与留给提交时那一趟、什么都不起：$scoped_wrong" \
    "看 scoped_domain_guard 与 lib_crash_point_replay_items.py 的 run_scoped_domain、refusal_text"
  # scoped-domain-reusable-passes：同样的记录，库里 4 块键都对得上 ⇒ 现判 0 态，放行，放量入口照起
  scoped_wrong=""
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=0 STUB_ITEM_REUSABLE=4 -- --item gamma
  [[ "$exit_code" == 0 ]] || scoped_wrong+="点名项全部可复用应当放行（退 $exit_code）；"
  out_has "全部复用，现判 0 态" || scoped_wrong+="应当写出这一项全部复用、现判 0 态；"
  calls_have '^pipeline-entry ' || scoped_wrong+="放行之后放量入口应当照起；"
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-reusable-passes "库里录下的块键都对得上，不论上一趟的态数多大都应当放行、现判 0 态：$scoped_wrong" \
    "看 lib_crash_point_replay_items.py 的 estimate_subject：录下的块大于 0 且键对不上的块是 0 就是现判 0 态"
  # scoped-domain-narrowed-passes：给了 --crash-expand-up-to 6（小于引擎常量 30）⇒ 缩过域，放行，不起读库程序
  scoped_wrong=""
  : >"$work/item-calls"
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=4 STUB_ITEM_REUSABLE=0 STUB_ITEM_CALLS="$work/item-calls" -- --item gamma --crash-expand-up-to 6
  [[ "$exit_code" == 0 ]] || scoped_wrong+="缩过域应当放行（退 $exit_code）；"
  out_has "缩过域" || scoped_wrong+="应当写缩过域；"
  [[ ! -s "$work/item-calls" ]] || scoped_wrong+="缩过域不判上限，不该起读库程序（起了：$(tr '\n' '|' <"$work/item-calls" | cut -c1-200)）；"
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-narrowed-passes "带了 --crash-expand-up-to 6 应当放行、不读库、不判上限：$scoped_wrong" \
    "看 scoped_domain_guard 交 --expand-up-to 与 run_scoped_domain 开头缩域那一支"
  # scoped-domain-unknown-refused：alpha 没有任何运行记录、库里一块都没录 ⇒ 估不出，不放行，退 79
  scoped_wrong=""
  entry STUB_ITEM_RECORDED=0 STUB_ITEM_TO_JUDGE=0 STUB_ITEM_REUSABLE=0 -- --item alpha
  [[ "$exit_code" == 79 ]] || scoped_wrong+="估不出应当退 79（退 $exit_code）；"
  out_has "算不出" || scoped_wrong+="说明里应当写算不出；"
  out_has "--crash-expand-up-to" || scoped_wrong+="说明里应当写 --crash-expand-up-to 这条出路；"
  if calls_have '^(pipeline-entry|enumeration-gate|cargo test) '; then scoped_wrong+="拒了之后不该起放量入口、门禁与 cargo test；"; fi
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-unknown-refused "库里没录这一项的块、也没有上一趟的运行记录，应当退 79、说明写算不出与出路、什么都不起：$scoped_wrong" \
    "看 lib_crash_point_replay_items.py 的 estimate_subject 算不出那一支与 refusal_text"
  # scoped-domain-shrunk-record-skipped：只剩缩到 6 的那一趟记录（225 态）、库里 4 块键都对不上 ⇒ 缩过域的记录不当全域，估不出，退 79
  scoped_wrong=""
  rm -rf -- "${scoped_runs:?}/entry-scoped-new"
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=4 STUB_ITEM_REUSABLE=0 -- --item gamma
  [[ "$exit_code" == 79 ]] || scoped_wrong+="只有缩过域的记录应当退 79（退 $exit_code）；"
  out_has "没有不缩域那一趟的真跑记录" || scoped_wrong+="说明里应当写没有不缩域那一趟的真跑记录；"
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-shrunk-record-skipped "缩过域那一趟的态数少，拿来当全域会估小，只有它时应当算不出、退 79：$scoped_wrong" \
    "看 lib_crash_point_replay_items.py 的 full_prior：只认展开深度等于引擎常量的记录"
  # scoped-domain-enumeration-records：beta 只有崩溃点重放部分。头一趟 --item beta（库里全部可复用，放行）假门禁打每条用例日志放哪个目录、放一份 states=1234 的日志 ⇒
  #   日志旁写 .scope（expand_up_to=full、case_states=beta:1234）；再 --list-items --with-keys --item beta（库里 3 块都要判）⇒ ITEM_DOMAIN 取那份日志的 1234 态
  scoped_wrong=""
  rm -rf -- "${work:?}/case-logs"
  entry STUB_CASE_LOG_DIRECTORY="$work/case-logs" -- --item beta
  [[ "$exit_code" == 0 ]] || scoped_wrong+="--item beta（库里全部可复用）应当放行（退 $exit_code）；"
  scoped_run_directory="$(ls -td "$scoped_runs"/crash-point-replay-* 2>/dev/null | head -n 1)"
  scoped_scope_file="$scoped_run_directory/enumeration.beta.log.scope"
  { grep -qxF 'expand_up_to=full' "$scoped_scope_file" && grep -qxF 'case_states=beta:1234' "$scoped_scope_file"; } 2>/dev/null \
    || scoped_wrong+="崩溃点重放部分跑完应当在日志旁写 .scope（expand_up_to=full、case_states=beta:1234；文件 ${scoped_scope_file:-没找到}：$(tr '\n' '|' <"$scoped_scope_file" 2>/dev/null)）；"
  entry STUB_ITEM_RECORDED=3 STUB_ITEM_TO_JUDGE=3 STUB_ITEM_REUSABLE=0 CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys --item beta
  grep -qxF 'ITEM_DOMAIN item=beta states_to_judge=1234 source=prior-run' "$work/out" \
    || scoped_wrong+="下一趟估上限应当读到那份日志的 1234 态（ITEM_DOMAIN 行：$(grep '^ITEM_DOMAIN ' "$work/out" | tr '\n' '|')）；"
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-enumeration-records "崩溃点重放部分每跑完一趟应当在日志旁写 .scope，下一趟估上限时读那一趟的用例日志里的态数：$scoped_wrong" \
    "看 record_enumeration_scope 与 lib_crash_point_replay_items.py 的 enumeration_totals"
  # item-domain-line：--list-items --with-keys 每个列出的项一行 ITEM_DOMAIN（ITEM_PLAN 之后、「键」几行之前）；gamma 上一趟全域 300000 态、库里 4 块里 1 块键对不上 ⇒ 75000 态（按块数比例），
  #   alpha 没有记录 ⇒ unknown，整趟才跑的类 0 态；点名了只打点名的项
  scoped_wrong=""
  scoped_record entry-scoped-new gamma-amp 30 300000 0
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=1 STUB_ITEM_REUSABLE=3 CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys
  domain_lines="$(grep '^ITEM_DOMAIN ' "$work/out")"
  [[ "$exit_code" == 0 && "$(grep -c . <<<"$domain_lines")" == 6 ]] || scoped_wrong+="应当 6 项各一行（退 $exit_code，打了 $(grep -c . <<<"$domain_lines") 行）；"
  grep -qxF 'ITEM_DOMAIN item=gamma states_to_judge=75000 source=prior-run-scaled' <<<"$domain_lines" || scoped_wrong+="gamma 应当是 75000 态（按块数比例）；"
  grep -qxF 'ITEM_DOMAIN item=alpha states_to_judge=unknown source=unknown' <<<"$domain_lines" || scoped_wrong+="alpha 没有记录应当写 unknown；"
  grep -qxF 'ITEM_DOMAIN item=whole-run:pinned-red-examples states_to_judge=0 source=no-crash-domain' <<<"$domain_lines" || scoped_wrong+="整趟才跑的类没有崩溃域应当写 0；"
  domain_order="$(awk '/^ITEM_PLAN /{ plan = NR } /^ITEM_DOMAIN /{ if (!first) first = NR; last = NR } /^键\t/{ if (!key) key = NR } END { print (plan < first && last < key) ? "ok" : "bad" }' "$work/out")"
  [[ "$domain_order" == ok ]] || scoped_wrong+="ITEM_DOMAIN 应当在 ITEM_PLAN 之后、「键」几行之前；"
  entry STUB_ITEM_RECORDED=4 STUB_ITEM_TO_JUDGE=1 STUB_ITEM_REUSABLE=3 CRASH_POINT_REPLAY_KEYS_READER_OUTPUT="$work/keys-reader-output" -- --list-items --with-keys --item gamma
  [[ "$exit_code" == 0 && "$(grep -c '^ITEM_DOMAIN ' "$work/out")" == 1 ]] && grep -q '^ITEM_DOMAIN item=gamma ' "$work/out" || scoped_wrong+="--item gamma 应当只打 gamma 一行（退 $exit_code）；"
  [[ -z "$scoped_wrong" ]] || cell_failed item-domain-line "--list-items --with-keys 应当每项一行 ITEM_DOMAIN、态数照估法、点名了只打点名的项：$scoped_wrong" \
    "看 report_the_item_domain 与 lib_crash_point_replay_items.py 的 run_scoped_domain report 那一支"
  # scoped-domain-checked-handshake：放量部分起放量入口时交 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1（放量入口不再判一遍）；运行记录先删，这一格只看交没交
  scoped_wrong=""
  rm -rf -- "${scoped_runs:?}"/entry-scoped-*
  entry -- --item gamma
  calls_have '^pipeline-env scoped-checked=\[1\]$' || scoped_wrong+="放量入口应当收到 SINGLEFS_CRASH_SCOPED_DOMAIN_CHECKED=1（退 $exit_code）；"
  [[ -z "$scoped_wrong" ]] || cell_failed scoped-domain-checked-handshake "放量部分起放量入口应当交已判过上限的标记：$scoped_wrong" "看 run_pipeline_part 里 checked_environment"
  rm -rf -- "${scoped_runs:?}"/entry-scoped-* "${work:?}/case-logs" "${work:?}/item-calls"

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
  echo "  ✓ --selftest $cells 格都对：列项（含整趟才跑的几类）不起编译、项名写错与缺项名退 2、删掉的选项一律当认不得的参数退 2、放量的判定在 prune-server 里（接连两趟第二趟复用头一趟判过的块）、"
  echo "    按条复用（真的 admission.py 在做成 git 仓的小仓上算键、查库、记库）：复用三问过、逐条写进的判定剪枝与门禁的计划照键复用、红过的与库里记着红的照现判、"
  echo "    同键又判红记红并撤掉那把键的绿（下一趟照现判）、登记成崩溃枚举用例与放量分项的测试函数经 --skip 排掉、被判的根不是 git 工作树的顶整趟退 2、按段登记文件的内容与片长进闭包读它们的那几条的键、"
  echo "    按条复用失灵判红且不起测试目标、上次红的清单只换真判过的项（本次未跑的照留）、"
  echo "    开了 GPU 测试函数那一段带 gpu 特性、交 task-server 给的那张卡、不自己清场（不送走 GPU 的计算类）而服务没给卡退 78 不起、"
  echo "    每一趟带按段登记文件的绝对路径而它核不过退 2 不起、测试函数红退 1 而另两段照旧起、跑到 0 条与没卡记本次未跑、全是本次未跑退 77、不带参数每项交崩溃点重放 --item 与放量全部项、"
  echo "    --item 只跑点名的项（含 whole-run:<类>）、测试函数的复用数照复用计数行报（没有那一行报取不出、成功句点名）、崩溃点重放红与放量红各红在那一项、放量收不回判红、平常跑带只供自证的变量退 2、开了 GPU 时放开的测试目标带登记的重活默认并行度而别的带 --test-threads 1，"
  echo "    测试函数日志里有给定状态表借不到显存那一行记本次未跑退 78、放量入口退 78 记本次未跑（红先判）而退 1 照旧红、"
  echo "    跑完运行目录里不剩它建的编译目录（正常、测试函数红、被 TERM 停）而调用方设的 CARGO_TARGET_DIR 不删，"
  echo "    按件分片（计划一件一片、一片只跑自己的件、判红的片退 0、合并判出红、件数超上限合片、片号对不上与产物不全退错、--no-shard 不分），"
  echo "    缩崩溃域（--crash-expand-up-to 与环境里的缩域变量：三段都收到、行里与句里写缩过域、值不是非负整数退 2）、--list-items --with-keys 的 ITEM_PLAN（每项一行、可与 --item 合用）与读库程序同一套特性、"
  echo "    真跑每项一行 ITEM_DONE、按件分片的计划另打每片每项一行 ITEM_PLAN，"
  echo "    （cargo、两套驱动与准入模块都是假的，不编不跑）；开关轮 ${#switch_wheel_assignments[@]} 个开关（文件头现读）逐个打开，派出 ${#switch_wheel_assignments[@]} 个、收回 $switch_wheel_collected 个，各红在点名的格"
  exit 0
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
# --no-shard 不分片：计划写一片，那一片照不分片整趟跑，合并什么都不做。弄坏开关 no-shard-ignored 下照分片
if [[ -n "$shard_phase" ]] && (( no_shard )) && ! replay_break_is no-shard-ignored; then
  case "$shard_phase" in
    plan)
      [[ -n "$shard_plan_directory" ]] || shard_refuse "环境里没有 TASK_SERVER_SHARD_PLAN_DIRECTORY" "这一步只由 task-server 照分片协议起（task-server/task_server_lib.py 文件头「分片」）" 2
      mkdir -p -- "$shard_plan_directory" || shard_refuse "建不了计划目录 $shard_plan_directory" "看共享工作根可不可写" 78
      printf '{"shards": [{"label": "整趟"}], "note": "不分片：--no-shard，整趟一片"}\n' >"$shard_plan_directory/plan.json"
      echo "  ✓ 不分片（--no-shard）：计划写 $(grep -o '"label"' "$shard_plan_directory/plan.json" | wc -l) 片，那一片整趟跑"
      exit 0 ;;
    shard)
      mkdir -p -- "$shard_output_directory" || shard_refuse "建不了产物目录 ${shard_output_directory:-（没给）}" "看共享工作根可不可写" 78
      shard_phase="" ;;
    merge)
      echo "  · 不分片（--no-shard）：结论在那一片里，合并什么都不做"
      exit 0 ;;
  esac
fi

if (( selftest )); then
  (( no_shard || list_items || with_keys || ${#named_items[@]} )) && usage "--selftest 不与别的参数一起给"
  run_selftest
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
report_the_keys_of_the_library() {
  local output reader_exit tag first second third fourth fifth seen=0 reader_features
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
        if [[ "$second" == missing ]]; then
          printf '键\t崩溃判定库 %s 不在：一块都没判过\n' "${first#library=}"
          continue
        fi
        if [[ "$first" != library=* || "$second" != recorded_blocks=* || "$third" != judged_under_the_current_key=* \
              || "$fourth" != judged_only_under_another_key=* || "$fifth" != never_judged=* ]]; then
          echo "  ✗ 读库程序那一行认不出：$tag $first $second $third $fourth $fifth（写法与 crash_library_reader.rs 文件头「输出」那一段对不上）" >&2
          echo "  → 怎么办：让 report_the_keys_of_the_library 与 crates/singlefs-verification/src/bin/crash_library_reader.rs 文件头「输出」那一段的写法对上，再跑 --list-items --with-keys" >&2
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
# 卡是 task-server 挑好交下来的（文件头「GPU」）：这里不读多机配置里的显卡先后
gpu_cards_for_test_functions="${GPU_CARDS_BY_PRIORITY:-}"
if (( gpu_switch )) && [[ "$shard_phase" != plan && "$shard_phase" != merge ]]; then
  # 开了 GPU 而服务没交卡：一项都不起、退 78。弄坏开关 gpu-card-missing-unrefused 下照跑
  if [[ -z "$gpu_cards_for_test_functions" ]] && ! replay_break_is gpu-card-missing-unrefused; then
    echo "  ✗ 多机配置开了 GPU（ENABLE_GPU=1），环境里却没有 task-server 交的显卡（GPU_CARDS_BY_PRIORITY）：这一趟一项都没起，退 78" >&2
    echo "  → 怎么办：经 task-server/task submit -- bash scripts/crash/crash-point-replay.sh … 交，服务挑卡、清场再起；分卡与清场不在这里做" >&2
    exit 78
  fi
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
# 覆盖行与计数行落成产物（文件头「输出」末段）：只落产物、不判红、不改退出码；弄坏开关 flow-coverage-table-skipped 下照改前不落
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
# 几趟同时跑不互盖。弄坏开关 last-run-whole-overwrite 下照改前：不点名的跑法整份写、点名的交点名的项。分片时由合并那一步写，一片不写
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
