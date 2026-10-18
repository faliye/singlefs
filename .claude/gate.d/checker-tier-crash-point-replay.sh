#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要的工具与设备登记在 stage-inputs.tsv 本阶段那一行第三列，由 scripts/gate/admission.py gate-preconditions 在阶段里判，没齐判红，不交给 gate.sh 预判
# gate-stage: 崩溃点重放与登记的崩溃枚举用例（快档跑 checker 档包不标 ignored 的用例、按条复用，核每条用例的崩溃状态块；--full 逐条按块的单元键跑全量）
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: code-source-discipline.sh 与 harness-model-differential-and-scenarios.sh 也调 scripts/crash/crash-case-check.py，但调的是 file-names、one-scenario 两样，分归代码类与 harness 类；这里开跑前只调 placement、modules 两样，判的是崩溃枚举用例与 checker 档测试文件，红了就不起 cargo
# gate-covers: 崩溃点重放
# gate-cell: crash-case-replay 快档跑 checker 档包不标 ignored 的用例（每条测试函数按单元键复用）、逐条核崩溃枚举用例的崩溃状态块；--full 逐条按块的单元键跑
# gate-cell: feature-gated-tests 快档带上 checker 档 tests/ 里 cfg 点到的特性另跑一趟（每条测试函数按单元键复用）：挂特性的测试文件逐个跑到或复用的用例不为 0，GPU 对拍没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑
#
# 复用：快档两趟里每条测试函数各是一条判定，键是它的单元键（测试函数的单元闭包、特性组合、release 档、工具链、解析出的依赖版本，判法 scripts/gate/lib_libtest_result_lines.py
#   的 result_line；算法在 scripts/gate/admission.py 文件头「单元键」），只跑键对不上的那几条（键相同的经 cargo test -- --exact --skip <名> 跳过），
#   结局经 lib_libtest_result_lines.py 的 binary_runs 逐条认、判绿的照键记进判定库（<git common-dir>/singlefs-unit-verdicts/<阶段文件名>:<格名>/）；
#   一个测试二进制的结果行条数与汇总行对不上（--nocapture 下测试自己打的字把结果行打散了）、或那一段里有 NOT_RUN_ / 判段判在 CPU 的行，那一段一条都不记、下一趟照跑。
#   一条都不用现跑就不起 cargo。拒算（单元闭包收不全）的那一条照跑、不复用、不入库，成功句报拒算条数与头几处形态。
#   按条复用失灵（test-plan 退非 0、退 0 而没打 PLAN、它或 verdict-record 的复用哨兵报「复用失灵」「判定不稳」「强制现判没起作用」）：判法只在
#   scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」），快档两格把那一趟的输出交给它；它判失灵，那一格照旧全部现判跑完、结论判红。
#   崩溃枚举用例的判定单位是崩溃状态块，块键与库写在 crates/singlefs-checker-tier/src/crash_identity.rs 文件头；用例跑完打一行
#   CRASH_CASE_BLOCKS blocks=<块数> reused=<复用> fresh=<现判> red=<红> refused=<拒算>（接口原样写在 scripts/gate/admission.py 文件头「崩溃枚举用例的复用不在这里」）。
#   快档逐条带 SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1 起用例（只算块键、查库，一块都不判），按那一行判「这条用例的全部块都有键相同的判定且无红」：
#   都有判定的算罩住，有红判红，别的报本次未跑（全量默认不在提交时跑，D13 已定项 15）；crates 侧还没接上块查询（用例源码里没有那个变量名）的，
#   不起用例、逐条报本次未跑。--full 跑登记的用例、块按键复用，--start-over 不读库、全部块现判、照键写回同一份库（SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1）。
#   登记行第三列带 red-blocks=judged-by-test 的用例（故障注入之后的红块由用例自己认下；admission.py crash-case-judged-by-test 答）快档不查库：
#   照全量起 crash-case-command 交的命令（库照键复用、不只写不读、不带块查询；features= 与 GPU 清场照原样），cargo test 退 0 算罩住、非 0 判红（出路看它的日志，
#   日志放 <common-dir>/singlefs-crash-point-replay-logs/quick-<开跑时刻>-<pid>/log.<用例名>，跑完不删），块那一行照 admission.py crash-case-blocks --ran
#   只核在不在、数对不对得上（缺行或对不上整条判红），红块不判；crates 侧还没接上块查询时它与别的用例一样报本次未跑、不起。
#   没有用例级复用标记，不比树、不比 HEAD，不问 stage-must-run.sh 与 change-touches-crates.sh；stage-inputs.tsv 本阶段那一行只登记前提与入口，不进任何键。
#
# 用法（参数顺序不限）：
#   bash .claude/gate.d/checker-tier-crash-point-replay.sh [项目根]                                     快档，两格都跑；gate.sh 只传项目根
#   bash <worktree>/.claude/gate.d/checker-tier-crash-point-replay.sh --full [--start-over] <worktree>   全量，只有 crash-case-replay 一格
#   --list                     打格名表（格名、制表符、判什么），不起 cargo、不起 python
#   --check <格名>[,<格名>…]   只跑点名的格
#   --list-items               逐行打被判那棵树 stage-inputs.tsv 里键是 crash-case: 的行与它们的 test=，不起 cargo、不起 python；
#                              另带 --with-keys 时末尾再打一行 crash-case-replay 那一格键对不上的测试函数条数（跑 admission.py test-plan，不起 cargo），
#                              再逐条打拒算的测试函数与理由「crash-case-replay<TAB>拒算<TAB><测试函数><TAB><理由>」（一条不落，不只列头几处）；
#                              只供证红：GATE_CRASH_POINT_REPLAY_LIST_PLAN_OUTPUT=<文件> 设了就不跑 test-plan，拿这份文件当它的标准输出
#   --item <用例名>            只看点名的崩溃枚举用例（crash-case: 可写可不写，给几次取并集）：快档照跑 checker 档包、只问这几条的块；--full 只跑这几条
#   --start-over               只跟 --full：只写不读，每条用例的全部块现判、照键写回同一份长期库（不另建库）
# 退出码：
#   0   绿；快档有用例的块没有判定、带特性那一格有本次未跑时也退 0，逐条报「本次未跑」
#   1   红；快档有一格红就退 1
#   2   用法错：参数认不出，格名或用例名写错或缺，--start-over 没跟 --full，--full 只点了 feature-gated-tests，项目根进不去，
#       样本档标记目录里的假 cargo 或假 rustc 不在或不可执行，PATH 上的 cargo 没解到标记目录里那一份
#   3   样本档流程判绿：样本档没跑过真的崩溃点重放，不与真跑的绿混在一起
#   77  本次未跑：没有 checker 档包
#   78  准入与运行条件没满足（preflight）
# 不用共用库 lib/stage-cells.sh：它的汇总把 0、77 之外的退出码并成 1（样本档要退 3）。
# 解析参数的判法与共用库同一套（写错退 2、列出格名）。
#
# 覆盖：两条流的崩溃点重放全量由登记的崩溃枚举用例跑，新池新建文件与 E142 产物逐字比对、「覆盖写、释放、回退与复用」固定脚本到 E 与闭式比对，都由用例自己断言；
#   只做过 mkfs 的池可写挂载再发第一个文件版本那条流与新池新建文件逐项相同，由 cargo test 里的快用例钉住，不另枚举。
#   用例平时标 ignored，--full 在 release 下带 --include-ignored --exact 逐条跑；用例的判别力在用例自己（计数断言与同文件的靶向阳性对照）。
#
# --full：在 HEAD + 暂存区的 worktree 里跑（主工作区读得到别的会话没暂存的改动）；建法与 gate.sh --staged 相同，命令在快档的出路句里。
#   逐条崩溃枚举用例，照登记表的次序，起 crash-case-command 交的命令（库目录照长期库，调用方环境里的 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY 清掉）；
#   日志判法在 admission.py crash-case-judge（test result 恰好 1 passed、登记的计数行、线程），块那一行在 admission.py crash-case-blocks（有红判红）。
#   全部判绿时在这一趟的日志目录里留一份 all-green（只记这一趟绿过，scripts/gate/apply-writer-patch.py 数「上次崩溃验证全绿以来打了几个补丁」用；不是复用的标记）。
#   线程数 SINGLEFS_CRASH_POINT_REPLAY_THREADS 没设取每台默认重活线程数（.claude/heavy-thread-default 的登记值，admission.py 的 crash_case_worker_threads 经 registered_heavy_threads 取）；
#   只起 1 个线程怎么判红，判法在 admission.py crash-case-judge。
#   双机分片默认关：本地配置 ${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}（模板 multi-host.env.example）经 scripts/crash/crash-point-replay-shard-configuration-check.sh
#   判得过才开；开着时登记了 shard=across-machines 的用例交 scripts/crash/crash-point-replay-shard-run.sh --merged-log（它经崩溃放量的双机驱动：本机 0/2、第二台 1/2 各写各的库，取回，本机带导入再跑一趟），
#   导入那一趟的日志照单机判。
#   日志：每趟一个新目录 <common-dir>/singlefs-crash-point-replay-logs/<开跑时刻>-<pid>/，跑完不删；log.<用例名> 是全量日志，同名加 .findings.tsv 是发现日志
#   （行格式以 crates/singlefs-checker-tier/src/crash_point_replay_progress.rs 为准）。发现日志判红：某节没有 summary、red_states 不是 0、定稿行数与 signatures= 对不上，
#   或发现日志空而全量日志里有 CRASH_POINT_REPLAY_FINDINGS 行。发现表超过 30 个签名只打前 30 行。
#
# 快档 feature-gated-tests 格（C604（快档不开特性，崩溃放量与 GPU 对拍编进来零条））：特性从 checker 档 tests/ 顶层每个 .rs 的 cfg 现读，不另抄名单；
#   挂特性的文件跑到加复用 0 条、每处 NOT_RUN_NO_GPU_ADAPTER（GPU 对拍没卡时打这一行后返回成功）都报「本次未跑」、不判红；一个挂特性的文件都没有，记无对象可判。
#   清掉调用方的 SINGLEFS_CRASH_AMPLIFICATION_* 与 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE：快档不写真的判定库、不按全量的规模跑。
#   特性里有 gpu 时这一趟经 scripts/crash/gpu-quiesce-run.sh 起：照多机配置清场并回读、跑，跑完不复原（包装打 step=restore result=not-restored；
#   要复原由用户手动跑私有复原脚本），它打的 GPU_QUIESCE 行逐行转打；清场回读没过（包装退 78、打 step=command result=not-started）判红、写出路，
#   卡上有不归清场白名单的计算进程时出路是等它跑完再只重跑这一格，这里不停它。样本档里清场配置只认样本根 .crash-point-replay-sample-tools/multi-host.env
#   （只写假清场命令），没有就判红、不起包装；包装的登记与本机日志放这一趟的临时目录，不落进真工作根。
#   特性里有 gpu 时这一趟的测试参数带 `-- --nocapture --test-threads 1`（GPU 对拍的测试各占卡的显存，不并行起）；特性里没有 gpu 的不加。
#   这个参数只管同时起几个测试，不管一个测试里 CPU 判器自己开的线程：GPU 对拍的测试读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取本机可用并行度
#   （available_parallelism，不照 SINGLEFS_THREAD_CAP 与登记值；改成取登记值归代码批）。样本里的假 cargo 把收到的参数打成一行 CRASH_POINT_REPLAY_PROGRESS sample-cargo-arguments: …，阶段转出来，quick-features-green 的 expect 核这一行带不带 --test-threads 1。
#   --test-threads 1 下 libtest 先打「test <名> ... 」、测试的输出接在后面，结果行常被打散：那一段对不上就整段不记（下一趟照跑），不算红。
#
# 多机配置开了 GPU（ENABLE_GPU=1）：快档两趟都在这份配置下跑，不靠仓根配置关 GPU。配置照 scripts/crash/crash-amplification-lib.sh 的
#   crash_amplification_multi_host_configuration_file 找（${SINGLEFS_MULTI_HOST_CONFIG:-<主工作树的根>/multi-host.env}，样本档只认样本根
#   .crash-point-replay-sample-tools/multi-host.env），找到的那一份在就经 SINGLEFS_MULTI_HOST_CONFIG=<它的绝对路径> 交给两趟 cargo，用例与这里判的是同一份。
#   ENABLE_GPU 的值另经 line:enable-gpu=<0|1> 进快档两趟每条测试函数的键（同一条测试在开不开 GPU 下判的不是同一件事）。
#   不带特性那一趟（crash-case-replay 格）：编不进 gpu 的用例遇到 ENABLE_GPU=1 打一行 NOT_RUN_<原因> test=<用例> reason=<一句> 报本次未跑、照常返回成功
#     （crates 那一侧的约定；GPU 对拍没卡时的 NOT_RUN_NO_GPU_ADAPTER 是同一种写法），这一格把日志里每一处这样的行逐条往 $GATE_NOT_RUN_FILE 报一行
#     （不判红、不算通过，逐格汇总里这一格记「不带特性那一趟有 N 处本次未跑」）；它要判的那一截归带特性那一趟。日志里有判段判在 CPU 的行
#     （崩溃枚举那一类的 CRASH_POINT_REPLAY_JUDGE route=cpu，或 CRASH_POINT_REPLAY_PARALLEL_START / _FINISHED 带 judge=cpu）判红：配了 GPU 不退回 CPU。
#   带特性那一趟（feature-gated-tests 格）：特性里有 gpu 时照旧经 scripts/crash/gpu-quiesce-run.sh 起；配置写了 ENABLE_GPU=1 而日志里有判段判在 CPU 的行
#     （同上两种）判红：配了 GPU、又编进了 gpu，判段必须在 GPU。
#   判得出的只有崩溃枚举那一类打的这两种行；给定状态与放量流水线的用例有的另写一份 ENABLE_GPU=0 的配置（GPU 对拍的 CPU 那一侧），它们的 mode= 不看。
#   配置没写 ENABLE_GPU=1（关着、没写、文件不在）两趟都不判判段在哪。
# 样本档：样本根上有 .crash-point-replay-sample-tools/ 就进样本档，把它放到 PATH 最前面（里面的假 cargo、假 rustc 照 cases/<测试目标>.log 打日志，
#   设了 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 时把 cases/<测试目标>.findings.tsv 追加进去），不管带没带 --full 都走 --full 那一路；判绿退 3，判红照常退 1。
#   里面另放一个 quick-tier 文件就留在快档：假 cargo 不带 --features 打 cases/default-pass.log，
#   带了打 cases/feature-pass.<--features 的值>.log、退出码照同名 .exit（没有那一份日志退 101）。
#   样本根另放 .unit-verdict-sample/prior/（上一版的源码树，setup.sh 照这一版拷一份再改）时，快档开跑前先拿那一版算键、照「判绿」记进这一版的判定库
#   （admission.py verdict-plan / verdict-record 的 --library-root：在上一版的树上算、记进样本自己的 git common-dir），再照常判：
#   expect 读出来的「现判 … 复用 …」逐条点名就是这一版相对上一版该作废与不该作废的测试函数。
#   样本由 .claude/singlefs-ai-sop/scripts/stage-selftest.sh 跑，setup.sh 在临时目录里 git init；进库的假工具 git 模式是 100755。
# 样本（fixtures/checker-tier-crash-point-replay.sh/）：
#   green                        一条用例的发现日志一节、0 个签名，一条用例不走发现日志
#   red                          两个签名 red_states=3、没有 summary、32 个签名三条用例
#   quick-gpu-configured-plain-pass-not-run      样本的多机配置写了 ENABLE_GPU=1，不带特性那一趟一条用例打了 NOT_RUN_GPU_FEATURE_NOT_COMPILED_IN：记本次未跑、不判红（退 3）
#   quick-gpu-configured-plain-pass-judged-on-cpu  同一份配置，不带特性那一趟的日志里有 CRASH_POINT_REPLAY_JUDGE route=cpu：退回 CPU，判红
#   quick-features-gpu-configured-green          同 quick-features-green，多机配置写了 ENABLE_GPU=1：不带特性那一趟报本次未跑、带特性那一趟判段在 GPU，判绿（退 3）
#   quick-features-gpu-configured-judged-on-cpu  同一份配置，带 gpu 特性那一趟的日志里有 CRASH_POINT_REPLAY_JUDGE route=cpu：判段落在 CPU，判红
#   static-check-red             一条标了 #[ignore]、调 enumerate_crash_states 却没登记 crash-case: 的 checker 档用例，一份第一行没声明模块的 checker 档测试文件：
#                                开跑前的静态判据判红、不起 cargo（red、green 的测试文件第一行都声明了「无」，静态判据判绿、往下走）
#   quick-features-green         三个挂特性的文件都跑到了用例
#   quick-features-gpu-not-run   两处 NOT_RUN_NO_GPU_ADAPTER（一处跟在 test 行后面、一处在库那一段）
#   quick-features-zero          一个文件只有 ignored、一个文件 cargo 输出里没有它那一段
#   quick-features-red           带特性那一趟退 101（不复原）
#   quick-features-quiesce-red   假清场配置的清场回读退 1：带特性那一趟不起、判红（不复原）
#   quick-features-interrupted   假 cargo 打完日志给自己发 TERM（退 143）：判红（不复原）
#   quick-features-no-quiesce-config  样本档里没放假的清场配置：不起包装、判红
#   quick-features-* 的 .crash-point-replay-sample-tools/multi-host.env 是只写假清场命令（true / false）的多机配置，quick-features-no-quiesce-config 没有它。
#   sample-cargo-not-executable  标记目录里只有假 cargo 与假 rustc，setup.sh 把假 cargo 的执行位去掉、给假 rustc 补上：退 2
#   unit-key-samples-green       快档按条复用（对照样本 S17、N19、F1、H2、H3）：tests/setting.rs 里 test_t 走观察者 observe、test_u 与 test_v 不走；
#                                tests/states.rs 三条 F1 测试各调一态的函数；tests/history.rs 里 h2_test、h3_test。setup.sh 照这一版拷一份上一版进
#                                .unit-verdict-sample/prior/、逐处改回去：observe（S17）、test_u 的一个字（N19）、中间那一态的 middle_state（F1）、h2_value（H2）；
#                                这一版连 HEAD 一起提交、refs/sop/staged-green 也挪过去，再做一次空提交（H3）。要现判的恰好是 test_t、test_u、f1_middle、h2_test
#                                （各报变了的那一个函数），复用的是 test_v、f1_first、f1_last、h3_test，假 cargo 照收到的 --skip 只打要现判的结果行，判绿（退 3）
#   unit-key-samples-red         同一份上一版与改动，假 cargo 让 S17 那一条（test_t）判 FAILED：判红，红句点名它与它变了的单元（observe），复用的几条照样 --skip 不跑
#   unit-key-plan-exit-red       按条复用失灵：.unit-verdict-sample/plan-failure 写 exit-1（admission.py 文件头「按条复用失灵」：test-plan 退 1），快档照旧全部现判
#                                （假 cargo 不带 --skip、三条都过），crash-case-replay 格判红、点名 test-plan 退 1
#   unit-key-plan-sentinel-red   同形，写 sentinel-reuse-failed（test-plan 的复用哨兵报复用失灵），判红、点名哨兵那一句
#   quick-judged-by-test-green   两条崩溃枚举用例，crates 侧接上了块查询（src/lib.rs 里有那个变量名）：plain-stream 没带标记，查询那一趟打 cases/default-pass.log 里
#                                全部块有判定、无红的那一行，块全罩住；fault-injection-given-states 登记 features=verdict-store red-blocks=judged-by-test，
#                                假 cargo 带 --features verdict-store 打 cases/feature-pass.verdict-store.log（test result: ok、CRASH_CASE_BLOCKS ... red=2），退 0：实跑绿，判绿（退 3）
#   quick-judged-by-test-red     同一份登记，cases/feature-pass.verdict-store.exit 写 101（日志里块那一行照样对得上）：实跑红，红句点名它、出路指到它的日志
#   quick-judged-by-test-missing-blocks-line  同一份登记，实跑退 0 而日志里没有 CRASH_CASE_BLOCKS 行：crash-case-blocks --ran 退 4，判红
#   四份 quick-features-* 的 tests/ 里挂特性的写法各有一处：整份 #![cfg(feature)]、折成几行的 #![cfg(all(…))]、逐条 #[cfg(feature)]，
#   与一个只在 // 注释里出现的特性名（读进来就找不到那组特性的日志）。
#   每份样本根的 .gate-cells 点名它算给哪一格：green、red、static-check-red、sample-cargo-not-executable、quick-gpu-configured-*、unit-key-samples-*、unit-key-plan-*、
#   quick-judged-by-test-* 算给 crash-case-replay，
#   quick-features-* 算给 feature-gated-tests。这一道不用共用库、自己不读 .gate-cells，它只给 code-tooling 的 fixture-claims 格逐格数红绿。
# 弄坏开关（只给证红用）。逐项写开关名、弄坏哪一处，下一行起写打开后判错的样本（fixtures/checker-tier-crash-point-replay.sh/ 下的目录名，
# 括号里是 stage-selftest.sh 报那份样本判错的第一行的句首）：
#   GATE_CRASH_POINT_REPLAY_BREAK=setting-file-whole-closure  快档每条测试函数的键按它所在的整份设点文件算（文件里每个单元都进键：改同文件里别的测试也作废）
#       unit-key-samples-green（输出里找不到「复用 crates/singlefs-checker-tier/tests/setting.rs::test_v」：N19 那一格，只改了 test_u 的一个字，同文件的 test_v 也被判成要现判）；
#       unit-key-samples-red（输出里找不到「CRASH_POINT_REPLAY_PROGRESS sample-cargo-arguments: …」：复用的那几条不再 --skip）
#   admission.py 的 ADMISSION_BREAK=plan-failure-not-red  按条复用失灵也不判红（改前快档两格退回全部现判、不判红的样子）
#       unit-key-plan-exit-red、unit-key-plan-sentinel-red（期望退出 1，实测 3：快档流程判绿）
#   GATE_CRASH_POINT_REPLAY_BREAK=no-findings-file  不设 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE
#       green（期望退出 3，实测 1）；red（输出里找不到「第 1 节…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-unfinished  没有 summary 的节当跑完了
#       red（输出里找不到「第 1 节…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-red-states  不看 red_states
#       red（输出里找不到「问题：第 1 节 red_states=3：有状态判红」）
#   GATE_CRASH_POINT_REPLAY_BREAK=no-truncation  签名全打不截
#       red（输出里找不到「其余 2 个见」）
#   GATE_CRASH_POINT_REPLAY_BREAK=static-check-skipped  跳过开跑前的静态判据
#       static-check-red（输出里找不到「── 开跑前的静态判据」）
#   GATE_CRASH_POINT_REPLAY_BREAK=no-feature-pass  不起带特性那一趟
#       quick-features-gpu-not-run（输出里找不到「本次未跑（这不是通过）：GPU 对拍没跑…」）；quick-features-green（输出里找不到「✓ feature-gated-tests：3 个挂特性的测…」）；
#       quick-features-interrupted、quick-features-quiesce-red、quick-features-red（期望退出 1，实测 3）；quick-features-zero（输出里找不到「! feature-gated-tests 不算全绿：3 个…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=features-not-from-source  特性不从源码读、写死 verdict-store,gpu（假 cargo 没有这组特性的日志、退 101）
#       quick-features-gpu-not-run、quick-features-green、quick-features-zero（期望退出 3，实测 1）；quick-features-interrupted（输出里找不到「GPU_QUIESCE step=command result=exit-143」）；
#       quick-features-red（输出里找不到「找不到 GPU 适配器…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-feature-pass-exit  带特性那一趟退非 0 当成跑过
#       quick-features-interrupted、quick-features-red（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-zero-feature-files  不判挂特性的文件跑到 0 条
#       quick-features-zero（输出里找不到「本次未跑（这不是通过）：checker 档 crates/s…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=ignore-gpu-not-run  不读 NOT_RUN_NO_GPU_ADAPTER
#       quick-features-gpu-not-run（输出里找不到「本次未跑（这不是通过）：GPU 对拍没跑…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=gpu-test-threads-unset  特性里有 gpu 也不加 --test-threads 1（照改前，测试线程取 RUST_TEST_THREADS 即登记值）
#       quick-features-green（输出里找不到「…-- --nocapture --test-threads 1」与「sample-cargo-arguments: … -- --nocapture --test-threads 1」两处）
#   GATE_CRASH_POINT_REPLAY_BREAK=gpu-pass-unquiesced  特性里有 gpu 也不经清场包装、直接起 cargo
#       quick-features-green、quick-features-red、quick-features-interrupted（输出里找不到「GPU_QUIESCE step=…」）；
#       quick-features-quiesce-red、quick-features-no-quiesce-config（期望退出 1，实测 3）
#   GPU_QUIESCE_BREAK=run-despite-failed-quiesce  清场包装（scripts/crash/gpu-quiesce-run.sh）清场回读没过也起命令
#       quick-features-quiesce-red（期望退出 1，实测 3）
#   GPU_QUIESCE_BREAK=restore-after-run  清场包装出去时又照多机配置复原
#       quick-features-green、quick-features-red、quick-features-interrupted、quick-features-quiesce-red（输出里找不到「GPU_QUIESCE step=restore result=not-restored holders=0」）
#   GATE_CRASH_POINT_REPLAY_BREAK=sample-tools-unchecked  不核标记目录里的假工具
#       sample-cargo-not-executable（期望退出 2，实测 1）
#   GATE_CRASH_POINT_REPLAY_BREAK=cpu-judging-accepted  配置写了 ENABLE_GPU=1 也不看判段判在哪
#       quick-gpu-configured-plain-pass-judged-on-cpu、quick-features-gpu-configured-judged-on-cpu（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=plain-pass-not-run-unreported  不带特性那一趟日志里的 NOT_RUN_<原因> 行不收、不报本次未跑
#       quick-gpu-configured-plain-pass-not-run、quick-features-gpu-configured-green（输出里找不到「本次未跑（这不是通过）：checker 档快档不带特性那一趟…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=list-items-refusals-counted-only  --list-items --with-keys 照改前只报拒算条数、不逐条列拒算的测试函数与理由
#       样本档喂不进 --list-items（stage-selftest.sh 只传根），证红的输入是一份照 test-plan 标准输出的写法造的文件（几条 refused、几条 judge 与 reuse、末行 PLAN）：
#       GATE_CRASH_POINT_REPLAY_LIST_PLAN_OUTPUT=<那份文件> bash .claude/gate.d/checker-tier-crash-point-replay.sh --list-items --with-keys <根>，
#       开关关着逐条打那几行「crash-case-replay<TAB>拒算<TAB>…」，打开时一行都没有
#   CRASH_CASE_CHECK_BREAK=rows-ignored  判法那一份（scripts/crash/crash-case-check.py）不报没登记的那一条
#       static-check-red（输出里找不到「forgot_to_register_full：标了 #[i…」）
#   GATE_CRASH_POINT_REPLAY_BREAK=judged-by-test-ignores-exit  快档实跑登记了 red-blocks=judged-by-test 的用例，cargo test 退非 0 也当绿
#       quick-judged-by-test-red（期望退出 1，实测 3）
#   GATE_CRASH_POINT_REPLAY_BREAK=judged-by-test-queried  不认 red-blocks=judged-by-test，带标记的用例照改前查库（有红判红）
#       quick-judged-by-test-green（期望退出 3，实测 1）；quick-judged-by-test-missing-blocks-line（期望退出 1，实测 3）；
#       quick-judged-by-test-red（输出里找不到「CRASH_POINT_REPLAY_PROGRESS sample-cargo-arguments: test --rel…」：带标记的那条不再实跑，只起块查询）
#   admission.py 的 ADMISSION_BREAK=ran-blocks-line-optional  crash-case-blocks --ran 没有那一行也退 0
#       quick-judged-by-test-missing-blocks-line（期望退出 1，实测 3）
crash_point_replay_tier="quick"
crash_point_replay_start_over=0
root_argument=""
crash_point_replay_list_cells=0
crash_point_replay_list_items=0
crash_point_replay_list_with_keys=0
crash_point_replay_named_cases=()
# 格名表：与文件头两行 `# gate-cell:` 逐字相同
CRASH_CASE_REPLAY_CELL_NAME="crash-case-replay"
CRASH_CASE_REPLAY_CELL_TITLE="快档跑 checker 档包不标 ignored 的用例（每条测试函数按单元键复用）、逐条核崩溃枚举用例的崩溃状态块；--full 逐条按块的单元键跑"
FEATURE_GATED_TESTS_CELL_NAME="feature-gated-tests"
FEATURE_GATED_TESTS_CELL_TITLE="快档带上 checker 档 tests/ 里 cfg 点到的特性另跑一趟（每条测试函数按单元键复用）：挂特性的测试文件逐个跑到或复用的用例不为 0，GPU 对拍没卡打的 NOT_RUN_NO_GPU_ADAPTER 逐条记本次未跑"
# --check 点名的格（空着就是两格都跑）
crash_point_replay_selected_cells=()
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
crash_point_replay_check_cells() { # <逗号隔开的格名>：认 crash-case-replay、feature-gated-tests，记进 crash_point_replay_selected_cells；写错、空着退 2 并列出可用的格名
  local requested_cell requested_cells=()
  IFS=, read -r -a requested_cells <<< "$1"
  ((${#requested_cells[@]})) || requested_cells=("")
  for requested_cell in "${requested_cells[@]}"; do
    if [[ "$requested_cell" != "$CRASH_CASE_REPLAY_CELL_NAME" && "$requested_cell" != "$FEATURE_GATED_TESTS_CELL_NAME" ]]; then
      echo "  ✗ checker-tier-crash-point-replay.sh：--check 里有没登记的格名：「$requested_cell」"
      echo "     → 怎么办：格名照下面写（也可以 bash .claude/gate.d/checker-tier-crash-point-replay.sh --list 看）；按崩溃枚举用例点名用 --item <用例名>："
      printf '       %s  %s\n' "$CRASH_CASE_REPLAY_CELL_NAME" "$CRASH_CASE_REPLAY_CELL_TITLE" "$FEATURE_GATED_TESTS_CELL_NAME" "$FEATURE_GATED_TESTS_CELL_TITLE"
      exit 2
    fi
    crash_point_replay_selected_cells+=("$requested_cell")
  done
}
crash_point_replay_runs_cell() { # <格名>：这一趟跑不跑这一格（没给 --check 两格都跑）
  local selected_cell
  ((${#crash_point_replay_selected_cells[@]})) || return 0
  for selected_cell in "${crash_point_replay_selected_cells[@]}"; do [[ "$selected_cell" == "$1" ]] && return 0; done
  return 1
}
while (($#)); do
  stage_argument="$1"
  case "$stage_argument" in
    --full) crash_point_replay_tier="full" ;;
    --start-over) crash_point_replay_start_over=1 ;;
    --list) crash_point_replay_list_cells=1 ;;
    --list-items) crash_point_replay_list_items=1 ;;
    --with-keys) crash_point_replay_list_with_keys=1 ;;
    --check|--item)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then
        echo "  ✗ $stage_argument 后面缺名字"
        echo "     → 怎么办：写成 --check $CRASH_CASE_REPLAY_CELL_NAME，或 --item <崩溃枚举用例名>（crash-case: 可写可不写，几条就给几次；可点的用 --list-items 看）。"
        exit 2
      fi
      if [[ "$stage_argument" == --check ]]; then crash_point_replay_check_cells "$2"; else crash_point_replay_named_cases+=("crash-case:${2#crash-case:}"); fi
      shift ;;
    --check=*) crash_point_replay_check_cells "${stage_argument#--check=}" ;;
    --item=*) crash_point_replay_named_value="${stage_argument#--item=}"; crash_point_replay_named_cases+=("crash-case:${crash_point_replay_named_value#crash-case:}") ;;
    -*)
      echo "  ✗ 认不出的参数：$stage_argument"
      echo "     → 怎么办：只认 --full、--start-over、--list、--check $CRASH_CASE_REPLAY_CELL_NAME、--list-items [--with-keys]、--item <用例名> 与项目根，顺序不限：bash .claude/gate.d/checker-tier-crash-point-replay.sh [--full [--start-over]] [--item <用例名>] [项目根]"
      exit 2 ;;
    *) root_argument="$stage_argument" ;;
  esac
  shift
done
if ((crash_point_replay_list_cells)); then
  printf '%s\t%s\n' "$CRASH_CASE_REPLAY_CELL_NAME" "$CRASH_CASE_REPLAY_CELL_TITLE" "$FEATURE_GATED_TESTS_CELL_NAME" "$FEATURE_GATED_TESTS_CELL_TITLE"
  exit 0
fi
if ((crash_point_replay_list_items)); then
  crash_point_replay_list_table="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}/.claude/gate.d/stage-inputs.tsv"
  if [[ ! -f "$crash_point_replay_list_table" ]]; then
    echo "  ✗ 读不到 $crash_point_replay_list_table：列不出可点名的崩溃枚举用例"
    echo "     → 怎么办：在项目根跑，或把项目根作为参数传进来。"
    exit 1
  fi
  awk -F'\t' '$1 ~ /^crash-case:/ { test = ""; n = split($3, tokens, " "); for (i = 1; i <= n; i++) if (tokens[i] ~ /^test=/) test = substr(tokens[i], 6); printf "%s\t%s\n", $1, test }' "$crash_point_replay_list_table"
  # --with-keys：快档 crash-case-replay 那一格有几条测试函数的键对不上（admission.py test-plan，只算键、不起 cargo）。feature-gated-tests 的特性要从源码 cfg 现读、
  # 崩溃枚举用例的块要起用例才问得出，两样都不在这里报（跑那一格时成功句里有现判、复用）
  if ((crash_point_replay_list_with_keys)); then
    crash_point_replay_list_root="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"
    crash_point_replay_list_repository="$(cd "$(dirname "$0")/../.." && pwd)"
    crash_point_replay_list_scratch="$(mktemp -d)"
    trap 'rm -rf -- "${crash_point_replay_list_scratch:?}"' EXIT
    if [[ -n "${GATE_CRASH_POINT_REPLAY_LIST_PLAN_OUTPUT:-}" ]]; then
      # 只供证红（文件头 --list-items）：照 test-plan 标准输出写法造的文件当这一趟的输出
      crash_point_replay_list_plan_command=(cat -- "$GATE_CRASH_POINT_REPLAY_LIST_PLAN_OUTPUT")
    else
      crash_point_replay_list_plan_command=(python3 "$crash_point_replay_list_repository/scripts/gate/admission.py" test-plan "$crash_point_replay_list_root" "$(basename "$0"):$CRASH_CASE_REPLAY_CELL_NAME" singlefs-checker-tier
        "$crash_point_replay_list_scratch/plan" --profile release --token "judge:scripts/gate/lib_libtest_result_lines.py::result_line"
        --token "judge-root:$crash_point_replay_list_repository" --token "line:enable-gpu=0")
    fi
    if "${crash_point_replay_list_plan_command[@]}" >"$crash_point_replay_list_scratch/out" 2>&1 \
       && crash_point_replay_list_summary="$(grep '^PLAN ' "$crash_point_replay_list_scratch/out" | tail -n 1)" && [[ -n "$crash_point_replay_list_summary" ]]; then
      crash_point_replay_list_judge="$(sed -n 's/.* judge=\([0-9]*\).*/\1/p' <<<"$crash_point_replay_list_summary")"
      crash_point_replay_list_reuse="$(sed -n 's/.* reuse=\([0-9]*\).*/\1/p' <<<"$crash_point_replay_list_summary")"
      crash_point_replay_list_refused="$(sed -n 's/.* refused=\([0-9]*\).*/\1/p' <<<"$crash_point_replay_list_summary")"
      printf '%s\t键对不上的测试函数 %s 条（现判 %s、拒算 %s），键相同的 %s 条（照多机配置没开 GPU 算）\n' "$CRASH_CASE_REPLAY_CELL_NAME" \
        "$((crash_point_replay_list_judge + crash_point_replay_list_refused))" "$crash_point_replay_list_judge" "$crash_point_replay_list_refused" "$crash_point_replay_list_reuse"
      # 拒算的逐条列：测试函数与理由（test-plan 一行的第一列与第七列，理由去掉「拒算：」起头），一条不落；弄坏开关 list-items-refusals-counted-only 下照改前不列
      if [[ ",${GATE_CRASH_POINT_REPLAY_BREAK:-}," != *,list-items-refusals-counted-only,* ]]; then
        awk -F'\t' -v cell="$CRASH_CASE_REPLAY_CELL_NAME" 'NF >= 7 && $2 == "refused" { reason = $7; sub(/^拒算：/, "", reason); printf "%s\t拒算\t%s\t%s\n", cell, $1, reason }' \
          "$crash_point_replay_list_scratch/out"
      fi
    else
      printf '%s\t键对不上的测试函数：算不出（%s）\n' "$CRASH_CASE_REPLAY_CELL_NAME" "$(tail -n 1 "$crash_point_replay_list_scratch/out")"
    fi
    rm -rf -- "${crash_point_replay_list_scratch:?}"
  fi
  exit 0
fi
if [[ "$crash_point_replay_start_over" == 1 && "$crash_point_replay_tier" != full ]]; then
  echo "  ✗ --start-over 只跟 --full 一起用：快档不跑全量，没有库可换"
  echo "     → 怎么办：要换一份新的空库、从头跑全量，写成 bash .claude/gate.d/checker-tier-crash-point-replay.sh --full --start-over <根>。"
  exit 2
fi
if [[ "$crash_point_replay_tier" == full ]] && ! crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME"; then
  echo "  ✗ --full 只有 $CRASH_CASE_REPLAY_CELL_NAME 一格：$FEATURE_GATED_TESTS_CELL_NAME 只在快档跑"
  echo "     → 怎么办：带特性那一格不带 --full 跑：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME [项目根]；全量写成 --full（或 --full --check $CRASH_CASE_REPLAY_CELL_NAME）。"
  exit 2
fi
ROOT="${root_argument:-$(cd "$(dirname "$0")/../.." && pwd)}"
# 相对的 $0 在 cd 之后会指错，在 cd 之前取成绝对路径
crash_point_replay_stage_script_path="$(cd "$(dirname "$0")" && pwd)/$(basename "$0")"
# 准入模块：读登记表、算单元键、判前提、判日志、按条复用与崩溃状态块的判定都经它（子命令见它的文件头）
crash_point_replay_stage_repository="$(cd "$(dirname "$0")/../.." && pwd)"
crash_point_replay_admission_module="$crash_point_replay_stage_repository/scripts/gate/admission.py"
# 快档的两趟 cargo test 不吃 cargo 自己的缺省（核数）：经 capped.sh default 起，编译与测试线程取登记的重活默认并行度（.claude/heavy-thread-default，SINGLEFS_THREAD_CAP 设了取它）。
# 随仓走的 scripts/hosts/capped.sh 不在时（admission.py 自证建的临时仓、只放了准入模块与本阶段）不加前缀。
crash_point_replay_thread_prefix=()
[[ -f "$crash_point_replay_stage_repository/scripts/hosts/capped.sh" ]] && crash_point_replay_thread_prefix=(bash "$crash_point_replay_stage_repository/scripts/hosts/capped.sh" default)
cd "$ROOT" 2>/dev/null || exit 2
# 样本档（写法见文件头「样本档」）
crash_point_replay_sample_mode=0
if [[ -d "$PWD/.crash-point-replay-sample-tools" ]]; then
  crash_point_replay_sample_mode=1
  PATH="$PWD/.crash-point-replay-sample-tools:$PATH"
  export PATH
  if [[ -f "$PWD/.crash-point-replay-sample-tools/quick-tier" ]]; then
    echo "  ! 样本档（快档）：$PWD/.crash-point-replay-sample-tools 在、里面有 quick-tier，起 cargo 的是那里面的假 cargo，留在快档、按条复用照问样本自己的判定库；这一趟不算真跑过 checker 档"
  else
    crash_point_replay_tier="full"
    echo "  ! 样本档：$PWD/.crash-point-replay-sample-tools 在，起用例的是那里面的假 cargo，走 --full 那一路；这一趟不算真跑过崩溃点重放"
  fi
fi
# 弄坏开关（只给证红用，清单在文件头）
crash_point_replay_break="${GATE_CRASH_POINT_REPLAY_BREAK:-}"
# 多机配置与 GPU 开关（文件头「多机配置开了 GPU」）：样本档只认样本根的假配置，真跑照 crash-amplification-lib.sh 的找法；找到的那一份交给两趟 cargo
crash_point_replay_configuration_file=""
crash_point_replay_amplification_library="$crash_point_replay_stage_repository/scripts/crash/crash-amplification-lib.sh"
if [[ "$crash_point_replay_sample_mode" == 1 ]]; then
  if [[ -f "$PWD/.crash-point-replay-sample-tools/multi-host.env" ]]; then crash_point_replay_configuration_file="$PWD/.crash-point-replay-sample-tools/multi-host.env"; fi
elif [[ -f "$crash_point_replay_amplification_library" ]]; then
  crash_point_replay_configuration_file="$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_multi_host_configuration_file "$ROOT")"
fi
crash_point_replay_gpu_configured=0
crash_point_replay_configuration_setting=()
if [[ -n "$crash_point_replay_configuration_file" && -f "$crash_point_replay_configuration_file" ]]; then
  crash_point_replay_configuration_file="$(cd "$(dirname "$crash_point_replay_configuration_file")" && pwd)/$(basename "$crash_point_replay_configuration_file")"
  crash_point_replay_configuration_setting=("SINGLEFS_MULTI_HOST_CONFIG=$crash_point_replay_configuration_file")
  if [[ "$(source "$crash_point_replay_amplification_library" >/dev/null 2>&1; crash_amplification_configuration_value "$crash_point_replay_configuration_file" ENABLE_GPU)" == 1 ]]; then
    crash_point_replay_gpu_configured=1
  fi
fi
# 样本档先核假工具：不可执行时 PATH 解到真 cargo，就在样本目录里起了真编译、判的也不再是样本；有一样不对退 2
if [[ "$crash_point_replay_sample_mode" == 1 && "$crash_point_replay_break" != sample-tools-unchecked ]]; then
  crash_point_replay_sample_tools_directory="$PWD/.crash-point-replay-sample-tools"
  crash_point_replay_sample_tool_problems=()
  for crash_point_replay_sample_tool in cargo rustc; do
    crash_point_replay_sample_tool_path="$crash_point_replay_sample_tools_directory/$crash_point_replay_sample_tool"
    if [[ ! -e "$crash_point_replay_sample_tool_path" ]]; then
      crash_point_replay_sample_tool_problems+=("假 $crash_point_replay_sample_tool 不在：$crash_point_replay_sample_tool_path")
    elif [[ ! -f "$crash_point_replay_sample_tool_path" || ! -x "$crash_point_replay_sample_tool_path" ]]; then
      crash_point_replay_sample_tool_mode="$(stat -c '%A' "$crash_point_replay_sample_tool_path")"
      crash_point_replay_sample_tool_problems+=("假 $crash_point_replay_sample_tool 不可执行（$crash_point_replay_sample_tool_mode）：$crash_point_replay_sample_tool_path")
    fi
  done
  if crash_point_replay_resolved_cargo="$(type -P cargo)"; then :; else crash_point_replay_resolved_cargo=""; fi
  if [[ "$crash_point_replay_resolved_cargo" != "$crash_point_replay_sample_tools_directory/cargo" ]]; then
    crash_point_replay_sample_tool_problems+=("PATH 最前面放了标记目录，cargo 却解到 ${crash_point_replay_resolved_cargo:-（哪里都解不到）}")
  fi
  if (( ${#crash_point_replay_sample_tool_problems[@]} > 0 )); then
    echo "  ✗ 样本档：$crash_point_replay_sample_tools_directory 里的假工具有 ${#crash_point_replay_sample_tool_problems[@]} 处不对，不往下走（往下走 cargo 就是真的那一个，在样本目录里起真编译）："
    printf '      · %s\n' "${crash_point_replay_sample_tool_problems[@]}"
    echo "     → 怎么办：给标记目录里的假 cargo、假 rustc 补上执行位（chmod 755；进库的样本 git 模式写 100755：git update-index --chmod=+x <文件>），"
    echo "                缺的照 .claude/gate.d/fixtures/checker-tier-crash-point-replay.sh/green/.crash-point-replay-sample-tools 补上，再跑；不是有意跑样本档的，把 $crash_point_replay_sample_tools_directory 挪走再跑。"
    exit 2
  fi
fi

# 开跑前的静态判据（placement、modules；判法在 scripts/crash/crash-case-check.py）：快档、--full、样本档都在复用判定之前判，红了不起 cargo。
# 判法脚本不在（admission.py --selftest 拷这一份去的临时仓没有它）报一行本次未跑、往下走，不判绿也不判红。
crash_point_replay_static_judge="$crash_point_replay_stage_repository/scripts/crash/crash-case-check.py"
if [[ -d crates && "$crash_point_replay_break" != static-check-skipped && ! -f "$crash_point_replay_static_judge" ]]; then
  echo "  ! 开跑前的静态判据没跑：判法脚本 $crash_point_replay_static_judge 不在（这一份 checker-tier-crash-point-replay 被拷到了不带 scripts/ 的树里），往 GATE_NOT_RUN_FILE 报一行"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
    printf '%s\n' "checker-tier-crash-point-replay.sh 开跑前的静态判据（placement、modules）没跑：$crash_point_replay_static_judge 不在" >> "$GATE_NOT_RUN_FILE"
  fi
elif [[ -d crates && "$crash_point_replay_break" != static-check-skipped ]]; then
  echo "── 开跑前的静态判据：崩溃枚举用例住哪、登记没有，checker 档测试文件声明模块"
  if python3 "$crash_point_replay_static_judge" --only placement,modules "$ROOT"; then crash_point_replay_static_exit=0; else crash_point_replay_static_exit=$?; fi
  if [[ "$crash_point_replay_static_exit" != 0 && "$crash_point_replay_static_exit" != 77 ]]; then
    echo "  ✗ 开跑前的静态判据：crash-case-check --only placement,modules 退 $crash_point_replay_static_exit（逐处列在上面），这一趟不起 cargo"
    echo "     → 怎么办：照上面那一句出路改（崩溃枚举用例挪进 crates/singlefs-checker-tier/tests/、全量那条登记进 .claude/gate.d/stage-inputs.tsv 的 crash-case: 行；"
    echo "                checker 档测试文件第一行写「//! checker 档模块：…」）；改完先单跑 python3 scripts/crash/crash-case-check.py --only placement,modules 到它退 0，再跑这一道。"
    exit 1
  fi
fi

crash_point_replay_stage_file_name="$(basename "$0")"
[[ -f Cargo.toml && -d crates/singlefs-checker-tier ]] || { echo "  ! 没有 crates/singlefs-checker-tier（checker 档包），本阶段跳过"; exit 77; }
python3 "$crash_point_replay_admission_module" gate-preconditions "$crash_point_replay_stage_repository" "$crash_point_replay_stage_file_name" || exit 1

if ! git_common_directory="$(git -C "$ROOT" rev-parse --path-format=absolute --git-common-dir 2>/dev/null)" || [[ -z "$git_common_directory" ]]; then
  echo "  ✗ $ROOT 不是 git 工作树（取不到 git common-dir）：快档按条复用的判定库与 --full 的日志没处放、也没处读"
  echo "     → 怎么办：在这个项目的 git 仓里跑，或把仓库根作为参数传进来；判定库在 \$(git rev-parse --git-common-dir)/singlefs-unit-verdicts/。"
  exit 1
fi
crash_point_replay_scratch_directory="$(mktemp -d)"
# 快档实跑登记了 red-blocks=judged-by-test 的用例时的日志目录（文件头「复用」）：头一次用到才建，跑完不删
crash_point_replay_quick_log_directory=""
trap 'rm -rf -- "${crash_point_replay_scratch_directory:?}"' EXIT

# crash-cases 一条一行：键、包、测试目标、用例函数（制表符分隔），照登记表的次序；登记有错退非 0
if ! crash_case_listing="$(python3 "$crash_point_replay_admission_module" crash-cases "$ROOT")"; then
  printf '%s\n' "$crash_case_listing" | sed 's/^/       /'
  echo "  ✗ .claude/gate.d/stage-inputs.tsv 里 crash-case: 那几行登记有错（上面逐条列出）：判不出要跑、要核哪几条崩溃枚举用例"
  echo "     → 怎么办：照 scripts/gate/admission.py 文件头「崩溃枚举用例行」改那几行（test=<包>:<测试目标>:<用例函数> 恰好一条，用例函数在测试目标里找得到）；"
  echo "                单跑 python3 scripts/gate/admission.py crash-cases <项目根>，改到它退 0 为止。"
  exit 1
fi
crash_case_rows=()
if [[ -n "$crash_case_listing" ]]; then mapfile -t crash_case_rows <<< "$crash_case_listing"; fi
if (( ${#crash_case_rows[@]} == 0 )); then
  echo "  ✗ .claude/gate.d/stage-inputs.tsv 里一条崩溃枚举用例（键是 crash-case: 的行）都没登记：崩溃点重放全量没有东西可跑、可核"
  echo "     → 怎么办：两条流的崩溃点重放全量至少各登记一行，写法见 scripts/gate/admission.py 文件头「崩溃枚举用例行」。"
  exit 1
fi
if (( ${#crash_point_replay_named_cases[@]} > 0 )); then
  crash_point_replay_kept_rows=()
  crash_point_replay_unknown_cases=()
  for crash_point_replay_named_case in "${crash_point_replay_named_cases[@]}"; do
    crash_point_replay_found=0
    for crash_case_row in "${crash_case_rows[@]}"; do [[ "${crash_case_row%%$'\t'*}" == "$crash_point_replay_named_case" ]] && crash_point_replay_found=1; done
    (( crash_point_replay_found )) || crash_point_replay_unknown_cases+=("$crash_point_replay_named_case")
  done
  if (( ${#crash_point_replay_unknown_cases[@]} > 0 )); then
    echo "  ✗ --item 点名的崩溃枚举用例没登记：${crash_point_replay_unknown_cases[*]}"
    echo "     → 怎么办：用例名照 bash .claude/gate.d/checker-tier-crash-point-replay.sh --list-items 列的写（登记表里有 ${#crash_case_rows[@]} 条）。"
    exit 2
  fi
  for crash_case_row in "${crash_case_rows[@]}"; do
    for crash_point_replay_named_case in "${crash_point_replay_named_cases[@]}"; do
      if [[ "${crash_case_row%%$'\t'*}" == "$crash_point_replay_named_case" ]]; then crash_point_replay_kept_rows+=("$crash_case_row"); break; fi
    done
  done
  echo "  · 只看 --item 点名的 ${#crash_point_replay_kept_rows[@]} 条崩溃枚举用例（登记表里共 ${#crash_case_rows[@]} 条，别的这一趟不跑、不核）"
  crash_case_rows=("${crash_point_replay_kept_rows[@]}")
fi


# ── 快档按条复用（文件头「复用」）：每条测试函数一条判定 ─────────
# plan_tests <种类> <计划文件> <计划输出> [--feature <包>/<特性>…]：admission.py test-plan 算 checker 档包里每条测试函数的键、查库，输出一条一行
#   （<名字>、<reuse|judge|refused|not-compiled>、<键>、<目标号>、<目标根文件>、<libtest 全名>、<原因>）落进 <计划输出>，末行 PLAN 那一句的数放进
#   plan_judge、plan_reuse、plan_refused、plan_not_compiled；算不出（工作区读不出、模块出错）返回 1、原因放进 plan_problem、那一趟的子命令与退出码与输出放进 plan_failure，
#   调用方全部现判、什么都不记，再经 plan_failure_verdict 判按条复用失灵（失灵那一格结论判红）。
#   样本档放了 .unit-verdict-sample/prior/ 的，先拿上一版的树算一遍、照判绿记进这一版的判定库（文件头「样本档」）。
#   弄坏开关 GATE_CRASH_POINT_REPLAY_BREAK=setting-file-whole-closure 下每条测试函数的键另加它所在的整份设点文件（admission.py 的 ADMISSION_BREAK=test-key-with-whole-file）。
plan_tests() {
  local kind="$1" plan_file="$2" plan_output="$3" summary plan_exit
  shift 3
  local -a feature_options=("$@") break_setting=()
  local -a tokens=(--profile release --token "judge:scripts/gate/lib_libtest_result_lines.py::result_line" --token "judge-root:$crash_point_replay_stage_repository"
                   --token "line:enable-gpu=$crash_point_replay_gpu_configured")
  [[ "$crash_point_replay_break" == setting-file-whole-closure ]] && break_setting=("ADMISSION_BREAK=test-key-with-whole-file")
  plan_judge=0; plan_reuse=0; plan_refused=0; plan_not_compiled=0; plan_problem=""; plan_failure=()
  if [[ "$crash_point_replay_sample_mode" == 1 && -d "$PWD/.unit-verdict-sample/prior" ]]; then
    seed_prior_verdicts "$kind" ${feature_options[@]+"${feature_options[@]}"} || return 1
  fi
  if env ${break_setting[@]+"${break_setting[@]}"} python3 "$crash_point_replay_admission_module" test-plan "$ROOT" "$kind" singlefs-checker-tier "$plan_file" \
       "${tokens[@]}" ${feature_options[@]+"${feature_options[@]}"} >"$plan_output" 2>"$plan_output.err"; then plan_exit=0; else plan_exit=$?; fi
  if ((plan_exit != 0)); then
    plan_problem="admission.py test-plan 没成：$(tail -n 3 "$plan_output" "$plan_output.err" 2>/dev/null | tr '\n' ' ' | cut -c1-400)"
    plan_failure=(test-plan "$plan_exit" "$plan_output" "$plan_output.err")
    return 1
  fi
  summary="$(grep '^PLAN ' "$plan_output" | tail -n 1)"
  [[ -n "$summary" ]] || { plan_problem="admission.py test-plan 没打 PLAN 那一行"; plan_failure=(test-plan 0 "$plan_output" "$plan_output.err"); return 1; }
  plan_judge="$(sed -n 's/.* judge=\([0-9]*\).*/\1/p' <<<"$summary")"
  plan_reuse="$(sed -n 's/.* reuse=\([0-9]*\).*/\1/p' <<<"$summary")"
  plan_refused="$(sed -n 's/.* refused=\([0-9]*\).*/\1/p' <<<"$summary")"
  plan_not_compiled="$(sed -n 's/.* not-compiled=\([0-9]*\).*/\1/p' <<<"$summary")"
  return 0
}

# seed_prior_verdicts <种类> [--feature …]：样本档（文件头「样本档」）：拿 .unit-verdict-sample/prior/ 那一版算每条测试函数的键、全部照判绿记进这一版样本的判定库
seed_prior_verdicts() {
  local kind="$1" prior="$PWD/.unit-verdict-sample/prior" seed="$crash_point_replay_scratch_directory/seed" seed_exit
  shift
  local -a break_setting=()
  [[ "$crash_point_replay_break" == setting-file-whole-closure ]] && break_setting=("ADMISSION_BREAK=test-key-with-whole-file")
  if env ${break_setting[@]+"${break_setting[@]}"} python3 "$crash_point_replay_admission_module" test-plan "$prior" "$kind" singlefs-checker-tier "$seed.plan" \
       --profile release --token "judge:scripts/gate/lib_libtest_result_lines.py::result_line" --token "judge-root:$crash_point_replay_stage_repository" \
       --token "line:enable-gpu=$crash_point_replay_gpu_configured" --library-root "$ROOT" "$@" >"$seed.out" 2>&1; then seed_exit=0; else seed_exit=$?; fi
  if ((seed_exit != 0)); then
    plan_problem="样本档：上一版的树上算不出键：$(tail -n 3 "$seed.out" | tr '\n' ' ' | cut -c1-400)"
    plan_failure=(test-plan "$seed_exit" "$seed.out")
    return 1
  fi
  awk -F'\t' '$2 == "judge" || $2 == "reuse" { print $1 }' "$seed.out" >"$seed.names"
  if python3 "$crash_point_replay_admission_module" verdict-record "$prior" "$kind" "$seed.plan" "$seed.names" --how "样本档：上一版" --library-root "$ROOT" >"$seed.record" 2>&1; then
    seed_exit=0; else seed_exit=$?; fi
  if ((seed_exit != 0)); then
    plan_problem="样本档：上一版那一趟的判定没记进库：$(tail -n 2 "$seed.record" | tr '\n' ' ')"; plan_failure=(verdict-record "$seed_exit" "$seed.record"); return 1; fi
  echo "  · 样本档：上一版（.unit-verdict-sample/prior）的 $(wc -l <"$seed.names") 条测试函数照判绿记进这一版的判定库（$(tail -n 1 "$seed.record")）"
}

# skip_arguments_of_plan <计划输出>：键相同（reuse）、这一组特性下不编（not-compiled）的测试函数逐条 --skip（加 --exact）；要现判的同名就不跳（几个二进制里同名的测试都跑，宁多跑）。
# 结果放进全局数组 plan_skip_arguments（cargo test 的 -- 之后接）
skip_arguments_of_plan() {
  local name
  plan_skip_arguments=()
  while IFS= read -r name; do
    [[ -n "$name" ]] && plan_skip_arguments+=(--skip "$name")
  done < <(awk -F'\t' 'NF >= 7 && ($2 == "reuse" || $2 == "not-compiled") { skip[$6] = 1 } NF >= 7 && ($2 == "judge" || $2 == "refused") { keep[$6] = 1 }
                         END { for (name in skip) if (!(name in keep)) print name }' "$1" | LC_ALL=C sort)
  ((${#plan_skip_arguments[@]})) && plan_skip_arguments=(--exact "${plan_skip_arguments[@]}")
  return 0
}

# plan_failure_verdict <格名> <子命令> <退出码> <输出文件>…：按条复用失灵没有（判法只在 scripts/gate/admission.py 的 plan-failure，文件头「按条复用失灵」）；
# 失灵它打 ✗ 与出路、这里返回 1，那一格照旧跑完、结论判红
plan_failure_verdict() {
  local cell="$1"
  shift
  python3 "$crash_point_replay_admission_module" plan-failure "$crash_point_replay_stage_file_name 的 $cell 格" "$@"
}

# record_green_tests <种类> <计划文件> <计划输出> <日志> <要排除的段的清单文件> <格名>：这一趟判绿的测试函数照键记进库（文件头「复用」：结果行与汇总行对不上的段、
# 点到 <排除清单> 里那几段（有 NOT_RUN_ / 判段判在 CPU 的行）的整段不记）。打一行「· 判绿记库：…」；记不下只报、不判红；
# 复用哨兵报红（强制现判那一条判定不稳）的经 plan_failure_verdict 判失灵、返回 1
record_green_tests() {
  local kind="$1" plan_file="$2" plan_output="$3" log="$4" excluded="$5" cell="$6" names="$crash_point_replay_scratch_directory/green-names.$kind" record_exit
  python3 - "$plan_output" "$log" "$excluded" "$crash_point_replay_stage_repository/scripts/gate/lib_libtest_result_lines.py" >"$names" <<'PY' || { echo "  ! 判绿记库：取不出判绿的名单，这一趟一条都不记"; return 0; }
import importlib.util, sys
plan_output, log, excluded_file, library_path = sys.argv[1:5]
specification = importlib.util.spec_from_file_location("libtest_result_lines", library_path)
library = importlib.util.module_from_spec(specification)
specification.loader.exec_module(library)
by_binary = {}
for line in open(plan_output, encoding="utf-8", errors="replace"):
    columns = line.rstrip("\n").split("\t")
    if len(columns) >= 7 and columns[1] in ("judge",):
        by_binary.setdefault(columns[4], {})[columns[5]] = columns[0]
excluded = {line.strip() for line in open(excluded_file, encoding="utf-8", errors="replace") if line.strip()}
for run in library.binary_runs(open(log, encoding="utf-8", errors="replace").read()):
    if not run.consistent or run.source in excluded or ("库与别的二进制" in excluded and not run.source.startswith("tests/")):
        continue
    for result in run.results:
        name = by_binary.get(run.source, {}).get(result.full_name)
        if name and result.outcome == "ok":
            print(name)
PY
  if [[ ! -s "$names" ]]; then echo "  · 判绿记库：这一趟没有可记的（现判的都不在对得上的段里，或一条都没现判）"; return 0; fi
  if python3 "$crash_point_replay_admission_module" verdict-record "$ROOT" "$kind" "$plan_file" "$names" --how "cargo test --release -p singlefs-checker-tier（$kind）" \
       >"$names.record" 2>&1; then record_exit=0; else record_exit=$?; fi
  if ((record_exit == 0)); then
    echo "  · 判绿记库：$(tail -n 1 "$names.record")"
  else
    echo "  ! 判绿记库：admission.py verdict-record 没成（$(tail -n 2 "$names.record" | tr '\n' ' ')），下一趟这几条照跑"
    plan_failure_verdict "$cell" verdict-record "$record_exit" "$names.record" || return 1
  fi
  return 0
}

# excluded_binaries_of_log <日志> <清单文件>：有 NOT_RUN_ 行或判段判在 CPU 的那几段（tests/<名>.rs，或「库与别的二进制」）写进清单文件：那几段的判绿不记库
excluded_binaries_of_log() {
  { test_not_run_lines "$1" | cut -f1; cpu_judging_lines "$1" | sed 's/：.*//'; gpu_adapter_not_run_lines "$1" | cut -f1; } | LC_ALL=C sort -u >"$2"
}

# plan_report <计划输出> [<本次未跑>]：成功句里那一截「现判 N、复用 M、拒算 R」（给了第二个参数另接「、本次未跑 K」），拒算另带头三处形态
plan_report() {
  local forms
  forms="$(awk -F'\t' 'NF >= 7 && $2 == "refused" { sub(/^拒算：/, "", $7); split($7, parts, "；"); count[parts[1]]++ } END { for (form in count) print count[form] "\t" form }' "$1" \
    | sort -rn | head -n 3 | awk -F'\t' '{ printf "%s%s 条：%s", (NR > 1 ? "；" : ""), $1, substr($2, 1, 120) }')"
  printf '现判 %s、复用 %s、拒算 %s%s%s' "$plan_judge" "$plan_reuse" "$plan_refused" "${2:+、本次未跑 $2}" "${forms:+（拒算的头几处：$forms）}"
}

# plan_listing <计划输出>：逐条打现判（连同变了的单元）与拒算（连同头一处形态），各至多 20 条；复用的只在样本档或总共不过 40 条时逐条打
plan_listing() {
  local show_reuse=0
  if [[ "$crash_point_replay_sample_mode" == 1 ]] || (( plan_judge + plan_reuse + plan_refused <= 40 )); then show_reuse=1; fi
  awk -F'\t' -v show_reuse="$show_reuse" '
    NF >= 7 && $2 == "judge" { if (judged++ < 20) printf "    现判 %s（%s）\n", $1, substr($7, 1, 300) }
    NF >= 7 && $2 == "refused" { if (refused++ < 20) printf "    拒算 %s（%s）\n", $1, substr($7, 1, 300) }
    NF >= 7 && $2 == "reuse" && show_reuse { printf "    复用 %s\n", $1 }
    END {
      if (judged > 20) printf "    …现判另有 %d 条没列\n", judged - 20
      if (refused > 20) printf "    …拒算另有 %d 条没列\n", refused - 20
    }' "$1"
}

# failed_tests_changed_units <计划输出> <日志>：日志里判 FAILED 的那几条测试，各自这一趟为什么要现判（键对不上时是变了的单元，计划里那一列原样）
failed_tests_changed_units() {
  local name
  while IFS= read -r name; do
    awk -F'\t' -v name="$name" 'NF >= 7 && $6 == name && ($2 == "judge" || $2 == "refused") { printf "    红的 %s：%s\n", $1, substr($7, 1, 400) }' "$1"
  done < <(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' "$2" | LC_ALL=C sort -u)
}

# run_checker_package_tests <日志> [<libtest 参数…>]：只跑不标 ignored 的用例（--tests 连 src/bin/ 的内联单测一起带上），键相同的那几条经 -- --exact --skip 跳过
# （参数由调用方从计划里取，skip_arguments_of_plan）；cargo 的整段输出进日志；快档不写发现日志，清掉调用方的 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE。
run_checker_package_tests() {
  local log="$1"
  shift
  env -u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
    ${crash_point_replay_thread_prefix[@]+"${crash_point_replay_thread_prefix[@]}"} cargo test --release --no-fail-fast -p singlefs-checker-tier --lib --tests -- --nocapture "$@" 2>&1 \
    | tee "$log" \
    | { grep --line-buffered '^CRASH_POINT_REPLAY_PROGRESS ' || true; } \
    | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}

# read_crash_case_command <键> [--blocks-query]：准入模块 crash-case-command 交出的这一条的命令与环境（以 NUL 分隔）读进 case_machine_cores、
# case_threads、case_threads_origin（这三个只打进开跑那一行）与数组 case_command；--full --start-over 时带 --start-over（只写不读）。
# 交不出、写法不对返回 1，原因放进 case_command_problem。
read_crash_case_command() {
  local command_file="$crash_point_replay_scratch_directory/command.${1#crash-case:}" command_output
  local -a options=()
  if [[ "$crash_point_replay_start_over" == 1 ]]; then options+=(--start-over); fi
  if [[ "${2:-}" == --blocks-query ]]; then options+=(--blocks-query); fi
  if ! python3 "$crash_point_replay_admission_module" crash-case-command "$ROOT" "$1" ${options[@]+"${options[@]}"} > "$command_file"; then
    case_command_problem="$(tr '\0' ' ' < "$command_file")"
    return 1
  fi
  local -a command_words=()
  mapfile -d '' -t command_words < "$command_file"
  if (( ${#command_words[@]} < 4 )) || [[ ! "${command_words[0]}" =~ ^[1-9][0-9]*$ || ! "${command_words[1]}" =~ ^[1-9][0-9]*$ \
      || ! "${command_words[2]}" =~ ^(explicit|default)$ ]]; then
    command_output="$(tr '\0' ' ' < "$command_file")"
    case_command_problem="准入模块交的不是「核数、线程数、explicit|default、命令…」：${command_output}"
    return 1
  fi
  case_machine_cores="${command_words[0]}"
  case_threads="${command_words[1]}"
  case_threads_origin="${command_words[2]}"
  case_command=("${command_words[@]:3}")
  return 0
}

# run_crash_case <日志> <发现日志> <命令的词…>：环境里设 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE=<发现日志>；整段输出进日志，CRASH_POINT_REPLAY_PROGRESS 行边跑边转出来。
run_crash_case() {
  local log_file="$1"
  local -a findings_setting=(SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE="$2")
  if [[ "$crash_point_replay_break" == no-findings-file ]]; then findings_setting=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE); fi
  shift 2
  env "${findings_setting[@]}" "$@" 2>&1 \
    | tee "$log_file" \
    | { grep --line-buffered '^CRASH_POINT_REPLAY_PROGRESS ' || true; } \
    | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}

# run_crash_case_in_two_shards <键> <日志> <测试目标> <用例函数>：双机跑一条（crash-point-replay-shard-run.sh --merged-log，它经崩溃放量的双机驱动），
# 导入那一趟的日志由驱动抄进 <日志>；库照长期库（清掉调用方的 SINGLEFS_CRASH_AMPLIFICATION_LIBRARY），--start-over 时另设只写不读，线程数照 case_threads。
run_crash_case_in_two_shards() {
  local -a library_setting=(-u SINGLEFS_CRASH_AMPLIFICATION_LIBRARY)
  local label
  if [[ "$crash_point_replay_start_over" == 1 ]]; then library_setting+=("SINGLEFS_CRASH_LIBRARY_WRITE_ONLY=1"); fi
  # 驱动那一格「输入指纹」只进标签与输出（它的文件头）：用例一级没有输入指纹了，交键名的 sha256 当标签
  label="$(printf '%s' "$1" | sha256sum | cut -c1-64)"
  env "${library_setting[@]}" "SINGLEFS_CRASH_POINT_REPLAY_THREADS=$case_threads" bash "$crash_point_replay_shard_driver" --merged-log "$1" "$ROOT" "$label" "$2" "$3" "$4" 2>&1 | sed -u 's/^/    /'
  return "${PIPESTATUS[0]}"
}
# report_crash_case_findings <发现日志> <全量日志>：逐节打发现表（行格式与节的写法以 crates/singlefs-checker-tier/src/crash.rs 的发现日志为准：
# 字段制表符分，第一个字段是行的种类，其余 key=value）。每一处不对各打一行「问题：…」，有问题返回 1；发现日志不在又不该有时返回 0。
report_crash_case_findings() {
  local findings_file="$1" log_file="$2" stdout_findings_lines
  stdout_findings_lines="$(grep -c '^CRASH_POINT_REPLAY_FINDINGS ' "$log_file" 2>/dev/null)"
  stdout_findings_lines="${stdout_findings_lines:-0}"
  if [[ ! -s "$findings_file" ]]; then
    if [[ "$stdout_findings_lines" == 0 ]]; then
      echo "    发现日志：没有（$findings_file 不在或是空的，全量日志里也没有 CRASH_POINT_REPLAY_FINDINGS 行：这条用例没走读 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 的枚举入口）"
      return 0
    fi
    echo "    发现日志：$findings_file 不在或是空的"
    echo "      问题：全量日志里有 ${stdout_findings_lines} 行 CRASH_POINT_REPLAY_FINDINGS（枚举跑完了 ${stdout_findings_lines} 趟），发现日志却一节都没写：SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 没传进用例"
    return 1
  fi
  echo "    发现日志：$findings_file（全量日志里 CRASH_POINT_REPLAY_FINDINGS ${stdout_findings_lines} 行）"
  awk -F'\t' -v shown_limit=30 -v findings_path="$findings_file" -v break_item="$crash_point_replay_break" '
    function field(name,   position, equals) {
      for (position = 2; position <= NF; position++) {
        equals = index($position, "=")
        if (equals > 0 && substr($position, 1, equals - 1) == name) return substr($position, equals + 1)
      }
      return ""
    }
    function signature_text(tail_name) {
      return "#" field("finding") " pass=" field("pass") " violated=" field("violated") " segment=" field("segment") " publish=" field("publish") " " tail_name
    }
    $1 == "crash_point_replay_findings_begin" {
      sections++
      section_name[sections] = "stream=" field("stream") (field("shard") != "" ? " shard=" field("shard") : "") " states=" field("states")
      next
    }
    sections == 0 { lines_before_first_section++; next }
    $1 == "crash_point_replay_finding" { final_rows[sections, ++final_count[sections]] = signature_text("states=" field("states") " sample_states=" field("sample_states")); next }
    $1 == "crash_point_replay_finding_new" { new_rows[sections, ++new_count[sections]] = signature_text("first_state=" field("first_state")); next }
    $1 == "crash_point_replay_finding_threshold" { next }
    $1 == "crash_point_replay_findings_summary" {
      summary_count[sections]++
      summary_signatures[sections] = field("signatures"); summary_red_states[sections] = field("red_states"); summary_states[sections] = field("states")
      next
    }
    { unknown_kinds[$1]++ }
    function print_rows(kind, section, count,   row, limit) {
      limit = (break_item == "no-truncation") ? count : shown_limit
      for (row = 1; row <= count && row <= limit; row++) print "        " (kind == "final" ? final_rows[section, row] : new_rows[section, row])
      if (count > limit) print "        其余 " (count - limit) " 个见 " findings_path
    }
    END {
      problems = 0
      if (sections == 0) { print "      问题：一行 crash_point_replay_findings_begin 都没有，认不出节"; problems++ }
      if (lines_before_first_section > 0) { print "      问题：第一行 crash_point_replay_findings_begin 之前有 " lines_before_first_section " 行"; problems++ }
      for (kind in unknown_kinds) print "      （认不出的行种类 " kind "：" unknown_kinds[kind] " 行，没判）"
      for (section = 1; section <= sections; section++) {
        if (summary_count[section] == 0) {
          if (break_item == "ignore-unfinished") continue
          print "      第 " section " 节（" section_name[section] "）：这一趟没跑完（没有 crash_point_replay_findings_summary 行）；死之前找到 " (new_count[section] + 0) " 个签名："
          print_rows("new", section, new_count[section] + 0)
          print "      问题：第 " section " 节这一趟没跑完（没有 crash_point_replay_findings_summary 行：还在跑、被杀或 panic 判红）"
          problems++
          continue
        }
        print "      第 " section " 节（" section_name[section] "）：signatures=" summary_signatures[section] " red_states=" summary_red_states[section] " states=" summary_states[section]
        if (final_count[section] + 0 == 0) print "        没有签名"
        print_rows("final", section, final_count[section] + 0)
        if (summary_count[section] > 1) { print "      问题：第 " section " 节有 " summary_count[section] " 行 crash_point_replay_findings_summary"; problems++ }
        if (summary_signatures[section] != ((final_count[section] + 0) "")) {
          print "      问题：第 " section " 节定稿 " (final_count[section] + 0) " 行 crash_point_replay_finding，summary 却说 signatures=" summary_signatures[section]; problems++
        }
        if (break_item != "ignore-red-states" && summary_red_states[section] != "0") {
          print "      问题：第 " section " 节 red_states=" summary_red_states[section] "：有状态判红（不是 0 都判红；读不出数也判红）"; problems++
        }
      }
      exit (problems > 0 ? 1 : 0)
    }
  ' "$findings_file"
}

# print_staged_worktree_full_commands：出路里建「HEAD + 暂存区」worktree 并在里面跑 --full 的命令；建法与共享 gate.sh --staged 相同。
print_staged_worktree_full_commands() {
  echo '                在项目根、暂存之后，把下面三行命令放进同一次 Bash 调用（三行共用 crash_point_replay_full_base 这个变量，分开跑它就是空的；这次调用的退出码是经内存包装的那条 --full 命令的，250–254 是包装自己的结局）：'
  echo '                crash_point_replay_tree_ready=; crash_point_replay_full_base="$(mktemp -d)"; git diff --cached --binary > "$crash_point_replay_full_base/staged.patch"'
  echo '                git worktree add --detach "$crash_point_replay_full_base/tree" HEAD && { [ ! -s "$crash_point_replay_full_base/staged.patch" ] || git -C "$crash_point_replay_full_base/tree" apply --index "$crash_point_replay_full_base/staged.patch"; } && crash_point_replay_tree_ready=1'
  echo '                if [ "$crash_point_replay_tree_ready" = 1 ]; then SINGLEFS_HEAVY_TESTS=commit bash scripts/hosts/run-with-memory-cap.sh 16G bash "${crash_point_replay_full_base:?}/tree/.claude/gate.d/checker-tier-crash-point-replay.sh" --full "${crash_point_replay_full_base:?}/tree"; crash_point_replay_full_rc=$?; else echo "worktree 没建好或暂存区的 diff 套不上，--full 没跑"; crash_point_replay_full_rc=1; fi; git worktree remove --force "${crash_point_replay_full_base:?}/tree" 2>/dev/null; rm -rf "${crash_point_replay_full_base:?}"; ( exit "$crash_point_replay_full_rc" )'
}


# run_checker_quick_tier：快档跑 checker 档包不标 ignored 的用例，按条复用（文件头「复用」）。判红打出路、返回 1；判绿把现判、复用、拒算与每个测试二进制
# 那一行 test result 的 passed / ignored 加总进 quick_tier_report。一条都不用现判就不起 cargo（quick_tier_ran=0）。
run_checker_quick_tier() {
  local quick_log result_lines passed_count ignored_count passed_total=0 ignored_total=0 binary_count=0 plan_ok=1
  local kind="$crash_point_replay_stage_file_name:$CRASH_CASE_REPLAY_CELL_NAME"
  local plan_file="$crash_point_replay_scratch_directory/quick.plan" plan_output="$crash_point_replay_scratch_directory/quick.plan.out"
  quick_log="$crash_point_replay_scratch_directory/quick-singlefs-checker-tier.log"
  quick_tier_ran=1
  : >"$quick_log"
  plan_skip_arguments=()
  if plan_tests "$kind" "$plan_file" "$plan_output"; then
    skip_arguments_of_plan "$plan_output"
    echo "  · 快档按条复用：checker 档包 $((plan_judge + plan_reuse + plan_refused)) 条编得进的测试函数，$(plan_report "$plan_output")；这一组特性下不编 $plan_not_compiled 条"
    plan_listing "$plan_output"
  else
    plan_ok=0
    echo "  ! 快档按条复用算不出（$plan_problem）：这一趟全部现判、一条都不记库"
    ((${#plan_failure[@]})) && { plan_failure_verdict "$CRASH_CASE_REPLAY_CELL_NAME" "${plan_failure[@]}" || quick_tier_plan_red=1; }
  fi
  if (( plan_ok && plan_judge + plan_refused == 0 )); then
    quick_tier_ran=0
    quick_tier_report="checker 档包 $plan_reuse 条测试函数都有键相同的判定，全部复用，这一趟没起 cargo（现判 0、复用 $plan_reuse、拒算 0、本次未跑 0）"
  elif ! run_checker_package_tests "$quick_log" ${plan_skip_arguments[@]+"${plan_skip_arguments[@]}"}; then
    tail -40 "$quick_log"
    # --no-fail-fast：一个测试二进制红了其余照跑，红的用例与测试目标在这里一次列全
    grep -E '^test .* FAILED$|^error: test failed, to rerun pass' "$quick_log" | sed 's/^/    /' || true
    (( plan_ok )) && failed_tests_changed_units "$plan_output" "$quick_log"
    echo "  ✗ checker 档快档判红（上面是 cargo test 的尾部、全部红的用例与测试目标，「红的」那几行是各自变了的单元；键相同而复用的那几条这一趟没跑）"
    echo "     → 怎么办：单跑看细节：cargo test --release -p singlefs-checker-tier -- --nocapture（只看一个测试二进制加 --test <名>）；"
    echo "                断言消息里是第一处对不上的计数或违例；改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
    return 1
  fi
  if (( quick_tier_ran )); then
    result_lines="$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed; \([0-9]*\) ignored;.*/\1 \2/p' "$quick_log")"
    while read -r passed_count ignored_count; do
      [[ -n "${passed_count:-}" ]] || continue
      passed_total=$((passed_total + passed_count)); ignored_total=$((ignored_total + ignored_count)); binary_count=$((binary_count + 1))
    done <<< "$result_lines"
    if (( passed_total == 0 )); then
      echo "  ✗ checker 档快档跑过了，却读不到 cargo 的 test result 行，或要现判的用例一条都没通过：扫到 0 条不是通过"
      echo "     → 怎么办：cargo test --release -p singlefs-checker-tier -- --list 看这个包里还剩几条不标 ignored 的用例；"
      echo "                一条都没有，就是快用例被整批标了 ignored 或删掉了，补回来。"
      return 1
    fi
  fi
  # 配置开了 GPU（文件头「多机配置开了 GPU」）：判段判在 CPU 的判红；用例报的本次未跑逐条报，不判红、不算通过
  local cpu_judging=() plain_where plain_line
  if (( crash_point_replay_gpu_configured )) && [[ "$crash_point_replay_break" != cpu-judging-accepted ]]; then
    mapfile -t cpu_judging < <(cpu_judging_lines "$quick_log")
    if (( ${#cpu_judging[@]} > 0 )); then
      printf '      %s\n' "${cpu_judging[@]:0:20}"
      echo "  ✗ checker 档快档不带 gpu 特性那一趟：多机配置 $crash_point_replay_configuration_file 写了 ENABLE_GPU=1，日志里有 ${#cpu_judging[@]} 处判段判在 CPU（上面列了前 20 处），配了 GPU 不退回 CPU"
      echo "     → 怎么办：不带 gpu 特性那一趟遇到 ENABLE_GPU=1 打一行 NOT_RUN_<原因> test=<用例> reason=<一句> 报本次未跑、不判（crates 那一侧改），判归带特性那一趟（feature-gated-tests 格）；改完只重跑这一格：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME"
      return 1
    fi
  fi
  crash_point_replay_plain_not_run_total=0
  if [[ "$crash_point_replay_break" != plain-pass-not-run-unreported ]]; then
    while IFS=$'\t' read -r plain_where plain_line; do
      [[ -n "$plain_line" ]] || continue
      crash_point_replay_plain_not_run_total=$((crash_point_replay_plain_not_run_total + 1))
      crash_point_replay_report_not_run "checker 档快档不带特性那一趟（${plain_where}）报了本次未跑：${plain_line}；这一截归带特性那一趟（$FEATURE_GATED_TESTS_CELL_NAME 格）判，这一趟不算跑过"
    done < <(test_not_run_lines "$quick_log")
  fi
  if (( quick_tier_ran && plan_ok )); then
    quick_tier_report="checker 档包 $(plan_report "$plan_output" "$crash_point_replay_plain_not_run_total")；跑了的 ${binary_count} 个测试二进制 ${passed_total} 条通过、${ignored_total} 条 ignored（全量那几条留给 --full）"
  elif (( quick_tier_ran )); then
    quick_tier_report="checker 档包 ${binary_count} 个测试二进制，${passed_total} 条通过、${ignored_total} 条 ignored、本次未跑 ${crash_point_replay_plain_not_run_total} 处（按条复用算不出，全部现判；全量那几条留给 --full）"
  fi
  if (( plan_ok && quick_tier_ran )); then
    excluded_binaries_of_log "$quick_log" "$crash_point_replay_scratch_directory/quick.excluded"
    record_green_tests "$kind" "$plan_file" "$plan_output" "$quick_log" "$crash_point_replay_scratch_directory/quick.excluded" "$CRASH_CASE_REPLAY_CELL_NAME" \
      || quick_tier_plan_red=1
  fi
  return 0
}

# quick_tier_reuse_held：快档那一趟按条复用失灵了（quick_tier_plan_red，plan_failure_verdict 判的）就判红、返回 1；块判定那一半照旧先跑完
quick_tier_reuse_held() {
  (( ! quick_tier_plan_red )) && return 0
  echo "  ✗ $CRASH_CASE_REPLAY_CELL_NAME 这一格判红：快档按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；这一趟照旧全部现判跑完、用例都过了"
  echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再只跑这一格：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME；不能复用就判红，不退回全部现判当绿"
  return 1
}

# crash_case_judged_by_test <键>：这条用例登记了 red-blocks=judged-by-test 没有（admission.py crash-case-judged-by-test）：退 0 登记了、退 1 没登记；
# 判不出（它退别的）退 2、原因放进 judged_by_test_problem。弄坏开关 judged-by-test-queried 下一律答没登记（照改前查库）。
crash_case_judged_by_test() {
  local answer answer_exit
  [[ "$crash_point_replay_break" == judged-by-test-queried ]] && return 1
  if answer="$(python3 "$crash_point_replay_admission_module" crash-case-judged-by-test "$ROOT" "$1")"; then answer_exit=0; else answer_exit=$?; fi
  if (( answer_exit == 0 || answer_exit == 1 )); then return "$answer_exit"; fi
  judged_by_test_problem="admission.py crash-case-judged-by-test 退 ${answer_exit}：${answer}"
  return 2
}

# run_judged_by_test_case <键>：快档实跑一条登记了 red-blocks=judged-by-test 的用例（文件头「复用」）：起 crash-case-command 交的命令（不带 --blocks-query，
# 库照键复用、不只写不读；features= 与 GPU 清场照它原样），cargo test 退 0 且 crash-case-blocks --ran 对得上返回 0、块那一句放进 judged_by_test_note；
# 否则打红句与出路、返回 1。日志放 crash_point_replay_quick_log_directory，跑完不删。弄坏开关 judged-by-test-ignores-exit 下 cargo test 退非 0 也当退 0。
run_judged_by_test_case() {
  local case_key="$1" ran_log ran_exit blocks_output blocks_exit
  if [[ -z "$crash_point_replay_quick_log_directory" ]]; then
    crash_point_replay_quick_log_directory="$git_common_directory/singlefs-crash-point-replay-logs/quick-$(date -u +%Y%m%dT%H%M%SZ)-$$"
    if ! mkdir -p -- "$crash_point_replay_quick_log_directory"; then
      echo "  ✗ $case_key：建不了快档实跑的日志目录 $crash_point_replay_quick_log_directory，没起用例"
      echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后只重跑这一条：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME --item $case_key"
      crash_point_replay_quick_log_directory=""
      return 1
    fi
  fi
  ran_log="$crash_point_replay_quick_log_directory/log.${case_key#crash-case:}"
  if ! read_crash_case_command "$case_key"; then
    echo "  ✗ $case_key：登记了 red-blocks=judged-by-test，准入模块交不出起用例的命令：$case_command_problem"
    echo "     → 怎么办：单跑 python3 scripts/gate/admission.py crash-case-command <根> $case_key 看它报什么，修好之后只重跑这一条：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME --item $case_key"
    return 1
  fi
  echo "    $case_key：登记了 red-blocks=judged-by-test，照全量起用例、按 cargo test 的退出码判（${case_command[*]}）；日志：$ran_log"
  if run_crash_case "$ran_log" "$ran_log.findings.tsv" "${case_command[@]}"; then ran_exit=0; else ran_exit=$?; fi
  if [[ "$crash_point_replay_break" == judged-by-test-ignores-exit ]]; then ran_exit=0; fi
  if (( ran_exit != 0 )); then
    [[ -f "$ran_log" ]] && tail -n 40 "$ran_log" | sed 's/^/      /'
    echo "  ✗ $case_key：实跑红（cargo test 退 ${ran_exit}，上面是日志尾部）"
    echo "     → 怎么办：看它的日志 $ran_log：断言消息里是用例认下的两张清单之外的失败，或对不上的计数；改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
    echo "                改完只重跑这一条：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME --item $case_key"
    return 1
  fi
  if blocks_output="$(python3 "$crash_point_replay_admission_module" crash-case-blocks "$ROOT" "$ran_log" --ran)"; then blocks_exit=0; else blocks_exit=$?; fi
  if (( blocks_exit != 0 )); then
    echo "  ✗ $case_key：实跑退 0，块那一行不对（crash-case-blocks --ran 退 ${blocks_exit}：${blocks_output}）"
    echo "     → 怎么办：看它的日志 $ran_log：用例跑完每条流要打一行 CRASH_CASE_BLOCKS blocks=… reused=… fresh=… red=… refused=…，blocks = reused + fresh + refused"
    echo "                （接口在 scripts/gate/admission.py 文件头「崩溃枚举用例的复用不在这里」）；没有这一行多半是 --exact 过滤之后一条用例都没跑到，对一对 stage-inputs.tsv 里 $case_key 那一行的 test=。"
    return 1
  fi
  judged_by_test_note="$blocks_output"
  return 0
}

# check_crash_case_blocks：crash-case-replay 格快档的后一半（文件头「复用」）：逐条带 SINGLEFS_CRASH_CASE_BLOCKS_QUERY=1 起用例，按它打的 CRASH_CASE_BLOCKS
# 判「全部块都有键相同的判定且无红」；有红判红（返回 1），没罩住的报本次未跑、不判红；crates 侧还没接上块查询的一条都不起、逐条报本次未跑。
# 登记了 red-blocks=judged-by-test 的不查库、经 run_judged_by_test_case 实跑，红了判红（返回 1）。结论放进 crash_point_replay_cell_state。
check_crash_case_blocks() {
  local covered=0 uncovered_cases=() red_cases_here=() query_log blocks_output blocks_exit supported=1 case_row probe_exit
  local ran_green=0 ran_red_cases=() judged_exit queried_count
  # crates 侧接没接上块查询（admission.py crash-case-blocks 退 5 是没接上；拿空日志问一次，只看这一样）
  if python3 "$crash_point_replay_admission_module" crash-case-blocks "$ROOT" /dev/null --query >"$crash_point_replay_scratch_directory/blocks-supported" 2>&1; then probe_exit=0; else probe_exit=$?; fi
  (( probe_exit == 5 )) && supported=0
  echo "  ✓ checker 档快档跑完（release，只跑不标 ignored 的用例，全量留给 --full）：${quick_tier_report}"
  if (( ! supported )); then
    for case_row in "${crash_case_rows[@]}"; do uncovered_cases+=("${case_row%%$'\t'*}"); done
    echo "  ! ${#uncovered_cases[@]} 条崩溃枚举用例的块判定没问：$(head -n 1 "$crash_point_replay_scratch_directory/blocks-supported")；记「本次未跑」、不判红（全量默认不在提交时跑，D13 已定项 15）：${uncovered_cases[*]}"
  else
    for case_row in "${crash_case_rows[@]}"; do
      IFS=$'\t' read -r case_key _case_package _case_target _case_function <<< "$case_row"
      if crash_case_judged_by_test "$case_key"; then judged_exit=0; else judged_exit=$?; fi
      if (( judged_exit == 2 )); then
        ran_red_cases+=("$case_key")
        echo "  ✗ $case_key：判不出它带没带 red-blocks=judged-by-test（${judged_by_test_problem}），没起"
        echo "     → 怎么办：单跑 python3 scripts/gate/admission.py crash-case-judged-by-test <根> $case_key，照它报的改 .claude/gate.d/stage-inputs.tsv 里那一行"
        continue
      fi
      if (( judged_exit == 0 )); then
        if run_judged_by_test_case "$case_key"; then
          ran_green=$((ran_green + 1))
          echo "    $case_key：实跑绿（${judged_by_test_note}）"
        else
          ran_red_cases+=("$case_key")
        fi
        continue
      fi
      query_log="$crash_point_replay_scratch_directory/blocks-query.${case_key#crash-case:}.log"
      if ! read_crash_case_command "$case_key" --blocks-query; then
        uncovered_cases+=("$case_key")
        echo "       $case_key：交不出块查询的命令：$case_command_problem"
        continue
      fi
      run_crash_case "$query_log" "$crash_point_replay_scratch_directory/blocks-query.findings" "${case_command[@]}" >/dev/null
      if blocks_output="$(python3 "$crash_point_replay_admission_module" crash-case-blocks "$ROOT" "$query_log" --query)"; then blocks_exit=0; else blocks_exit=$?; fi
      case "$blocks_exit" in
        0) covered=$((covered + 1)); echo "    $case_key：全部块都有键相同的判定、没有红（$blocks_output）" ;;
        1) red_cases_here+=("$case_key"); echo "    $case_key：库里有判红的块（$blocks_output）" ;;
        *) uncovered_cases+=("$case_key"); echo "       $case_key：块没罩住（退 $blocks_exit：$blocks_output）" ;;
      esac
    done
  fi
  if (( ${#red_cases_here[@]} > 0 )); then
    echo "  ✗ ${#red_cases_here[@]} 条崩溃枚举用例在崩溃判定库里有键相同而判红的块：${red_cases_here[*]}"
    echo "     → 怎么办：跑那几条的全量看哪一块红：暂存之后在 HEAD + 暂存区的 worktree 里跑 --full --item <用例名>（建法见下）；块判红是被判的代码有问题，不是库坏了："
    print_staged_worktree_full_commands
  fi
  if (( ${#ran_red_cases[@]} > 0 )); then
    echo "  ✗ ${#ran_red_cases[@]} 条登记了 red-blocks=judged-by-test 的崩溃枚举用例快档实跑红：${ran_red_cases[*]}"
    echo "     → 怎么办：逐条看上面它那一句点名的日志；改完只重跑这几条：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $CRASH_CASE_REPLAY_CELL_NAME --item <用例名>"
  fi
  if (( ${#red_cases_here[@]} + ${#ran_red_cases[@]} > 0 )); then return 1; fi
  if (( ${#uncovered_cases[@]} > 0 )); then
    echo "  ! ${#uncovered_cases[@]} 条崩溃枚举用例的块没全罩住或没问，全量这一轮没跑，记「本次未跑」、不判红：${uncovered_cases[*]}"
    echo "     → 要跑全量（用户要求时）：暂存之后，在 HEAD + 暂存区的 worktree 里用那棵树里的 checker-tier-crash-point-replay 跑 --full <它的根>"
    echo "                （与 gate.sh --staged 同一建法；块按键复用，只判键对不上的块）："
    print_staged_worktree_full_commands
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
      for case_row in "${uncovered_cases[@]}"; do
        echo "崩溃枚举用例 ${case_row} 的崩溃状态块没全有键相同的判定（或 crates 侧还没接块查询），全量没跑（要跑：checker-tier-crash-point-replay --full）" >> "$GATE_NOT_RUN_FILE"
      done
    fi
  fi
  queried_count=$(( ${#crash_case_rows[@]} - ran_green ))
  echo "  ✓ 登记 ${#crash_case_rows[@]} 条崩溃枚举用例：实跑 ${ran_green} 条（登记了 red-blocks=judged-by-test，按退出码判，都绿）；查库 ${queried_count} 条：块全罩住 ${covered} 条、有红 ${#red_cases_here[@]} 条、本次未跑 ${#uncovered_cases[@]} 条"
  crash_point_replay_cell_state="绿"
  if (( ${#uncovered_cases[@]} > 0 )); then crash_point_replay_cell_state="有 ${#uncovered_cases[@]} 条崩溃枚举用例全量本次未跑"; fi
  return 0
}
# cpu_judging_lines <日志>：崩溃枚举那一类判段判在 CPU 的行（CRASH_POINT_REPLAY_JUDGE route=cpu，或 CRASH_POINT_REPLAY_PARALLEL_START / _FINISHED 带 judge=cpu；
# 行首可带空白），一处一行「<它所在的 tests/<名>.rs，或「库与别的二进制」>：<那一行>」
cpu_judging_lines() {
  awk '
    /^[[:space:]]*Running / {
      current = "库与别的二进制"
      if (match($0, /tests\/[^\/ ]+\.rs \(/)) current = "tests/" substr($0, RSTART + 6, RLENGTH - 8)
      next
    }
    {
      line = $0
      sub(/^[[:space:]]+/, "", line)
      if (line ~ /^CRASH_POINT_REPLAY_JUDGE route=cpu( |$)/ || (line ~ /^CRASH_POINT_REPLAY_PARALLEL_(START|FINISHED) / && line ~ / judge=cpu( |$)/))
        print ((current == "") ? "库与别的二进制" : current) "：" line
    }
  ' "$1"
}

# test_not_run_lines <日志>：用例报的本次未跑行（NOT_RUN_<原因> test=<用例> …，跟在「test <名> ... 」后面的也认，按子串找），
# 一处一行「<它所在的 tests/<名>.rs，或「库与别的二进制」><TAB><从 NOT_RUN_ 起到下一处 NOT_RUN_ 之前>」
test_not_run_lines() {
  awk '
    /^[[:space:]]*Running / {
      current = "库与别的二进制"
      if (match($0, /tests\/[^\/ ]+\.rs \(/)) current = "tests/" substr($0, RSTART + 6, RLENGTH - 8)
      next
    }
    {
      rest = $0
      while (match(rest, /NOT_RUN_[A-Z][A-Z0-9_]* test=/) > 0) {
        rest = substr(rest, RSTART)
        tail = substr(rest, 2)
        if (match(tail, /NOT_RUN_[A-Z][A-Z0-9_]* test=/) > 0) { piece = substr(rest, 1, RSTART); rest = substr(tail, RSTART) } else { piece = rest; rest = "" }
        sub(/[[:space:]]+$/, "", piece)
        print ((current == "") ? "库与别的二进制" : current) "\t" piece
      }
    }
  ' "$1"
}

# crash_point_replay_report_not_run <哪一部分没跑：为什么、怎么补>：照共享 lib.sh 的 report_not_run（这一份不 source lib.sh）。
crash_point_replay_report_not_run() {
  echo "  ! 本次未跑（这不是通过）：$1"
  if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$1" >> "$GATE_NOT_RUN_FILE"; fi
}

# list_feature_gated_test_files <tests 目录>：目录顶层每个 .rs（一个文件一个测试二进制）里 #[cfg(…)] / #![cfg(…)] 点到的特性，
# 一个文件一行「文件名<TAB>whole-file|items<TAB>特性,特性」（有 #![cfg(…)] 点到特性的记 whole-file），照文件名排序；挂特性的一个都没有时不打。
# 括号按层数配对（rustfmt 把长的 cfg 折成几行也认）；// 起头的行（注释与文档注释）不读。
list_feature_gated_test_files() {
  python3 - "$1" <<'PYTHON'
import pathlib
import re
import sys

attribute_start = re.compile(r"#(!?)\[\s*cfg\s*\(")
feature_name = re.compile(r"\bfeature\s*=\s*\"([^\"]+)\"")
for path in sorted(pathlib.Path(sys.argv[1]).glob("*.rs")):
    text = "\n".join(line for line in path.read_text(encoding="utf-8").splitlines() if not line.lstrip().startswith("//"))
    features, whole_file = set(), False
    for match in attribute_start.finditer(text):
        depth, position = 1, match.end()
        while position < len(text) and depth > 0:
            depth += {"(": 1, ")": -1}.get(text[position], 0)
            position += 1
        named = feature_name.findall(text[match.end():position])
        features.update(named)
        if named and match.group(1) == "!":
            whole_file = True
    if features:
        print(f"{path.name}\t{'whole-file' if whole_file else 'items'}\t{','.join(sorted(features))}")
PYTHON
}

# count_tests_run_by_file <日志…>：cargo test 输出里逐个集成测试文件跑到的用例数，几份日志相加；一行「文件名<TAB>条数」。
# 一个测试二进制从 cargo 的「Running …tests/<名>.rs (…)」那一行起、到下一行 Running 为止，条数是它 test result 行的 passed 加 failed（ignored 不算跑到）；
# 库（Running unittests …）与别的二进制不打。
count_tests_run_by_file() {
  awk '
    FNR == 1 { current = "" }
    /^[[:space:]]*Running / {
      current = ""
      if (match($0, /tests\/[^\/ ]+\.rs \(/)) { current = substr($0, RSTART + 6, RLENGTH - 8); seen[current] = 1 }
      next
    }
    current != "" && /^test result: / {
      if (match($0, /[0-9]+ passed/)) ran[current] += substr($0, RSTART, RLENGTH) + 0
      if (match($0, /[0-9]+ failed/)) ran[current] += substr($0, RSTART, RLENGTH) + 0
    }
    END { for (name in seen) printf "%s\t%d\n", name, ran[name] + 0 }
  ' "$@"
}

# gpu_adapter_not_run_lines <日志>：日志里每一处 NOT_RUN_NO_GPU_ADAPTER（GPU 对拍找不到 GPU 适配器时打的那一行：test=<用例名> reason=<一句>），
# 一处一行「<它所在的 tests/<名>.rs，或「库与别的二进制」><TAB>NOT_RUN_NO_GPU_ADAPTER …」；--nocapture 下它可能跟在「test <名> ... 」后面，按子串找。
gpu_adapter_not_run_lines() {
  awk '
    /^[[:space:]]*Running / {
      current = "库与别的二进制"
      if (match($0, /tests\/[^\/ ]+\.rs \(/)) current = "tests/" substr($0, RSTART + 6, RLENGTH - 8)
      next
    }
    {
      rest = $0
      while ((at = index(rest, "NOT_RUN_NO_GPU_ADAPTER ")) > 0) {
        rest = substr(rest, at)
        following = index(substr(rest, 2), "NOT_RUN_NO_GPU_ADAPTER ")
        piece = (following > 0) ? substr(rest, 1, following) : rest
        sub(/[[:space:]]+$/, "", piece)
        print ((current == "") ? "库与别的二进制" : current) "\t" piece
        rest = (following > 0) ? substr(rest, following + 1) : ""
      }
    }
  ' "$1"
}

# run_feature_gated_tests_cell <crash-case-replay 格那一趟的日志，没跑传空串>：判红打出路、返回 1；不红返回 0，结论放进 feature_cell_state。
# 按条复用（文件头「复用」）：这一组特性下每条测试函数各一条判定（种类 <阶段文件名>:feature-gated-tests），键相同的经 --skip 跳过，一条都不用现判就不起 cargo；
# 挂特性的文件「跑到 0 条」按跑到的加复用的算。
run_feature_gated_tests_cell() {
  local tests_directory="crates/singlefs-checker-tier/tests" listing feature_list feature_log feature_exit=0 bindgen_arguments=""
  local amplification_library="$crash_point_replay_stage_repository/scripts/crash/crash-amplification-lib.sh"
  local counts_file="$crash_point_replay_scratch_directory/feature-gated-counts"
  local row file_name file_scope file_features scope_text count ran_total=0 environment_name gpu_where gpu_line not_run_total
  local -a gated_rows=() features=() target_arguments=() bindgen_setting=() cleared_environment=(-u SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE) zero_files=() gpu_not_run=() gpu_test_arguments=()
  if ! listing="$(list_feature_gated_test_files "$tests_directory")"; then
    echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：读不出 $tests_directory 下哪几个测试文件挂特性 cfg（上面是 python 的报错）"
    echo "     → 怎么办：看报错里点名的文件（多半不是 UTF-8 或读不了），修好之后只重跑这一格：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
    return 1
  fi
  if [[ -z "$listing" ]]; then
    feature_cell_state="无对象可判"
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：$tests_directory 下 0 个测试文件挂特性 cfg，无对象可判，带特性那一趟不起（往 GATE_NOT_RUN_FILE 记一行）"
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then
      printf '%s\n' "checker-tier-crash-point-replay.sh 的 $FEATURE_GATED_TESTS_CELL_NAME 格无对象可判：$tests_directory 下没有挂特性 cfg 的测试文件" >> "$GATE_NOT_RUN_FILE"
    fi
    return 0
  fi
  mapfile -t gated_rows <<< "$listing"
  mapfile -t features < <(cut -f3 <<< "$listing" | tr ',' '\n' | LC_ALL=C sort -u)
  feature_list="$(IFS=,; printf '%s' "${features[*]}")"
  if [[ "$crash_point_replay_break" == features-not-from-source ]]; then feature_list="verdict-store,gpu"; fi
  for row in "${gated_rows[@]}"; do target_arguments+=(--test "${row%%.rs$'\t'*}"); done
  # 特性里有 gpu：测试线程只开 1 个（GPU 对拍的几个测试各占一张卡的显存，并行起会互相抢；一个测试里 CPU 判器自己的线程不受这个参数管：读 SINGLEFS_CRASH_JUDGE_CPU_THREADS，没设取本机可用并行度）
  if [[ ",$feature_list," == *,gpu,* && "$crash_point_replay_break" != gpu-test-threads-unset ]]; then gpu_test_arguments=(--test-threads 1); fi
  # 特性里有 gpu：这一趟经清场包装起（文件头「快档 feature-gated-tests 格」）；样本档只认样本里只写假清场命令的那份多机配置
  local -a quiesce_prefix=() quiesce_environment=()
  if [[ ",$feature_list," == *,gpu,* && "$crash_point_replay_break" != gpu-pass-unquiesced ]]; then
    quiesce_prefix=(bash "$crash_point_replay_stage_repository/scripts/crash/gpu-quiesce-run.sh" --)
    if [[ "$crash_point_replay_sample_mode" == 1 ]]; then
      if [[ ! -f "$PWD/.crash-point-replay-sample-tools/multi-host.env" ]]; then
        echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：样本档里没有假的清场配置 $PWD/.crash-point-replay-sample-tools/multi-host.env，带 gpu 特性那一趟不起（起了就是照真的多机配置清场）"
        echo "     → 怎么办：照 .claude/gate.d/fixtures/checker-tier-crash-point-replay.sh/quick-features-green/.crash-point-replay-sample-tools/multi-host.env 放一份只写假清场命令的配置，再跑"
        return 1
      fi
      quiesce_environment=("SINGLEFS_MULTI_HOST_CONFIG=$PWD/.crash-point-replay-sample-tools/multi-host.env" "GPU_QUIESCE_STATE_DIRECTORY=$crash_point_replay_scratch_directory/gpu-quiesce-state"
        "GPU_QUIESCE_LOG_DIRECTORY=$crash_point_replay_scratch_directory/gpu-quiesce-logs")
    fi
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：特性里有 gpu，这一趟经 scripts/crash/gpu-quiesce-run.sh 照多机配置清场再起，跑完不复原（要复原手动跑私有复原脚本，多机配置的 QUIESCE_START_COMMAND）"
  fi
# librocksdb-sys 的 bindgen 要的头文件目录：与崩溃放量取同一个值，编译产物才共用。在子 shell 里 source 那份库、只读这一个变量。
  if [[ -f "$amplification_library" ]]; then
    bindgen_arguments="$(source "$amplification_library" >/dev/null 2>&1; printf '%s' "${CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS:-}")"
  fi
  if [[ -n "$bindgen_arguments" ]]; then
    bindgen_setting=("BINDGEN_EXTRA_CLANG_ARGS=$bindgen_arguments")
  else
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：从 $amplification_library 读不到 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS，BINDGEN_EXTRA_CLANG_ARGS 照调用方环境（${BINDGEN_EXTRA_CLANG_ARGS:-没设}）；librocksdb-sys 编不过时这一格判红"
  fi
  while IFS= read -r environment_name; do
    if [[ "$environment_name" == SINGLEFS_CRASH_AMPLIFICATION_* ]]; then cleared_environment+=(-u "$environment_name"); fi
  done < <(compgen -e)
  feature_log="$crash_point_replay_scratch_directory/quick-singlefs-checker-tier-features.log"
  : > "$feature_log"
  local feature_kind="$crash_point_replay_stage_file_name:$FEATURE_GATED_TESTS_CELL_NAME" feature_plan_ok=1 feature_ran=1 feature_name feature_plan_red=0
  local feature_plan_file="$crash_point_replay_scratch_directory/features.plan" feature_plan_output="$crash_point_replay_scratch_directory/features.plan.out"
  local -a feature_plan_options=() feature_roots=(src/lib.rs) feature_names=()
  for row in "${gated_rows[@]}"; do feature_roots+=("tests/${row%%$'\t'*}"); done
  IFS=, read -r -a feature_names <<< "$feature_list"
  for feature_name in "${feature_names[@]}"; do feature_plan_options+=(--feature "singlefs-checker-tier/$feature_name"); done
  plan_skip_arguments=()
  if plan_tests "$feature_kind" "$feature_plan_file" "$feature_plan_output" "${feature_plan_options[@]}"; then
    # 这一趟只跑 --lib 与挂特性的那几个测试目标：计数只数这几个目标里的
    awk -F'\t' -v roots="$(IFS=' '; printf '%s' "${feature_roots[*]}")" 'BEGIN { n = split(roots, list, " "); for (i = 1; i <= n; i++) keep[list[i]] = 1 }
      NF >= 7 && ($5 in keep) { print }' "$feature_plan_output" > "$feature_plan_output.kept"
    plan_judge="$(awk -F'\t' '$2 == "judge"' "$feature_plan_output.kept" | grep -c .)"; plan_reuse="$(awk -F'\t' '$2 == "reuse"' "$feature_plan_output.kept" | grep -c .)"
    plan_refused="$(awk -F'\t' '$2 == "refused"' "$feature_plan_output.kept" | grep -c .)"
    skip_arguments_of_plan "$feature_plan_output"
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME 按条复用：--lib 与挂特性的 ${#gated_rows[@]} 个测试目标里 $((plan_judge + plan_reuse + plan_refused)) 条编得进的测试函数，$(plan_report "$feature_plan_output.kept")"
    plan_listing "$feature_plan_output.kept"
    (( plan_judge + plan_refused == 0 )) && feature_ran=0
  else
    feature_plan_ok=0
    : > "$feature_plan_output.kept"
    echo "  ! $FEATURE_GATED_TESTS_CELL_NAME 按条复用算不出（$plan_problem）：这一趟全部现判、一条都不记库"
    ((${#plan_failure[@]})) && { plan_failure_verdict "$FEATURE_GATED_TESTS_CELL_NAME" "${plan_failure[@]}" || feature_plan_red=1; }
  fi
  echo "── $FEATURE_GATED_TESTS_CELL_NAME：带特性那一趟（特性从 $tests_directory 下 ${#gated_rows[@]} 个挂特性的测试文件现读）：cargo test --release -p singlefs-checker-tier --features $feature_list --lib ${target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}"
  if [[ "$crash_point_replay_break" == no-feature-pass ]]; then
    : > "$feature_log"
  elif (( ! feature_ran )); then
    echo "  · $FEATURE_GATED_TESTS_CELL_NAME：这一组特性下要跑的测试函数都有键相同的判定，全部复用，这一趟没起 cargo"
  else
    env "${cleared_environment[@]}" ${crash_point_replay_configuration_setting[@]+"${crash_point_replay_configuration_setting[@]}"} \
      ${quiesce_environment[@]+"${quiesce_environment[@]}"} ${bindgen_setting[@]+"${bindgen_setting[@]}"} \
      ${quiesce_prefix[@]+"${quiesce_prefix[@]}"} ${crash_point_replay_thread_prefix[@]+"${crash_point_replay_thread_prefix[@]}"} cargo test --release --no-fail-fast -p singlefs-checker-tier --features "$feature_list" --lib "${target_arguments[@]}" -- --nocapture ${gpu_test_arguments[@]+"${gpu_test_arguments[@]}"} ${plan_skip_arguments[@]+"${plan_skip_arguments[@]}"} 2>&1 \
      | tee "$feature_log" \
      | { grep --line-buffered '^CRASH_POINT_REPLAY_PROGRESS ' || true; } \
      | sed -u 's/^/    /'
    feature_exit="${PIPESTATUS[0]}"
  fi
  if (( ${#quiesce_prefix[@]} > 0 && feature_ran )); then
    grep '^GPU_QUIESCE ' "$feature_log" | sed 's/^/    /'
    if [[ "$feature_exit" == 78 ]] && grep -q '^GPU_QUIESCE step=command result=not-started' "$feature_log"; then
      echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：带 gpu 特性那一趟没起：清场回读没过或多机配置没写全 QUIESCE_*（清场那两个键；清场包装的原话在 $feature_log）"
      echo "     → 怎么办：卡上有不归清场白名单的计算进程（别的会话的 GPU 活）时清场回读本来就不过：等那个进程跑完再只重跑这一格，不停它：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME；多机配置缺键的照仓根 multi-host.env.example 写全"
      return 1
    fi
  fi
  if [[ "$feature_exit" != 0 && "$crash_point_replay_break" != ignore-feature-pass-exit ]]; then
    tail -40 "$feature_log"
    grep -E '^test .* FAILED$|^error: test failed, to rerun pass' "$feature_log" | sed 's/^/    /' || true
    (( feature_plan_ok )) && failed_tests_changed_units "$feature_plan_output.kept" "$feature_log"
    echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：带特性那一趟 cargo test 退 $feature_exit（上面是它的尾部与全部红的用例、测试目标）"
    echo "     → 怎么办：单跑看细节：BINDGEN_EXTRA_CLANG_ARGS=\"$bindgen_arguments\" cargo test --release -p singlefs-checker-tier --features $feature_list --lib ${target_arguments[*]} -- --nocapture${gpu_test_arguments[*]:+ ${gpu_test_arguments[*]}}；"
    echo "                编不过多半是 librocksdb-sys 现编 C++ 缺 libclang、C++ 编译器或这个变量；用例红照断言消息改；没卡的机器上 GPU 对拍该打一行 NOT_RUN_NO_GPU_ADAPTER 之后返回成功，panic 就是那条用例没接这个约定。"
    return 1
  fi
  # 配置开了 GPU、这一趟编进了 gpu（文件头「多机配置开了 GPU」）：判段必须在 GPU，日志里有判在 CPU 的行判红
  if [[ ",$feature_list," == *,gpu,* ]] && (( crash_point_replay_gpu_configured )) && [[ "$crash_point_replay_break" != cpu-judging-accepted ]]; then
    local feature_cpu_judging=()
    mapfile -t feature_cpu_judging < <(cpu_judging_lines "$feature_log")
    if (( ${#feature_cpu_judging[@]} > 0 )); then
      printf '      %s\n' "${feature_cpu_judging[@]:0:20}"
      echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME：带 gpu 特性那一趟判在 CPU：多机配置 $crash_point_replay_configuration_file 写了 ENABLE_GPU=1、这一趟编进了 gpu，日志里有 ${#feature_cpu_judging[@]} 处判段判在 CPU（上面列了前 20 处）"
      echo "     → 怎么办：看那几行的 why_not_the_gpu=（选路在 crates/singlefs-checker-tier/src/crash_state_enumeration.rs 的 crash_point_replay_judge_route）：用例读的配置与这里判的不是同一份，就对齐 SINGLEFS_MULTI_HOST_CONFIG；改完只重跑这一格：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
      return 1
    fi
  fi
  count_tests_run_by_file "$feature_log" ${1:+"$1"} > "$counts_file"
  # 键相同而复用的也算跑到（这一组特性下的计划，与不带特性那一趟的计划）：一条一行「文件名<TAB>条数」
  awk -F'\t' 'NF >= 7 && $2 == "reuse" && $5 ~ /^tests\// { sub(/^tests\//, "", $5); count[$5]++ } END { for (name in count) printf "%s\t%d\n", name, count[name] }' \
    "$feature_plan_output.kept" "$crash_point_replay_scratch_directory/quick.plan.out" 2>/dev/null > "$counts_file.reused"
  for row in "${gated_rows[@]}"; do
    IFS=$'\t' read -r file_name file_scope file_features <<< "$row"
    count="$(awk -F'\t' -v name="$file_name" '$1 == name { print $2 }' "$counts_file")"
    count="${count:-0}"
    count=$((count + $(awk -F'\t' -v name="$file_name" '$1 == name { total += $2 } END { print total + 0 }' "$counts_file.reused")))
    ran_total=$((ran_total + count))
    if [[ "$count" == 0 && "$crash_point_replay_break" != ignore-zero-feature-files ]]; then
      zero_files+=("tests/$file_name")
      scope_text="逐条挂"; if [[ "$file_scope" == whole-file ]]; then scope_text="整份挂"; fi
      crash_point_replay_report_not_run "checker 档 $tests_directory/$file_name ${scope_text}特性 $file_features，快档跑到它的用例 0 条（cargo 输出里没有它那一段，或只有 ignored）：给它补一条不标 ignored 的快用例，或查 cfg 里的特性名与 Cargo.toml 的 [features] 对不对得上"
    fi
  done
  if [[ "$crash_point_replay_break" != ignore-gpu-not-run ]]; then
    while IFS=$'\t' read -r gpu_where gpu_line; do
      [[ -n "$gpu_line" ]] || continue
      gpu_not_run+=("$gpu_line")
      crash_point_replay_report_not_run "GPU 对拍没跑（${gpu_where}；这台机器找不到 GPU 适配器）：${gpu_line}；在有卡的机器上跑这一格补上"
    done < <(gpu_adapter_not_run_lines "$feature_log")
  fi
  if (( feature_plan_ok && feature_ran )) && [[ "$crash_point_replay_break" != no-feature-pass ]]; then
    excluded_binaries_of_log "$feature_log" "$crash_point_replay_scratch_directory/features.excluded"
    record_green_tests "$feature_kind" "$feature_plan_file" "$feature_plan_output" "$feature_log" "$crash_point_replay_scratch_directory/features.excluded" \
      "$FEATURE_GATED_TESTS_CELL_NAME" || feature_plan_red=1
  fi
  if (( feature_plan_red )); then
    echo "  ✗ $FEATURE_GATED_TESTS_CELL_NAME 这一格判红：按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；带特性那一趟照旧全部现判跑完了"
    echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再只跑这一格：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME；不能复用就判红，不退回全部现判当绿"
    return 1
  fi
  not_run_total=$(( ${#zero_files[@]} + ${#gpu_not_run[@]} ))
  if (( not_run_total > 0 )); then
    feature_cell_state="有 ${not_run_total} 处本次未跑"
    echo "  ! $FEATURE_GATED_TESTS_CELL_NAME 不算全绿：${#gated_rows[@]} 个挂特性的测试文件（特性 $feature_list）跑到 ${ran_total} 条，本次未跑 ${not_run_total} 处：编进来 0 条的文件 ${#zero_files[@]} 个（${zero_files[*]:-无}），没卡没跑的 GPU 对拍 ${#gpu_not_run[@]} 条"
    echo "     → 编进来 0 条的给它补不标 ignored 的快用例；GPU 对拍在有卡的机器上补跑：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $FEATURE_GATED_TESTS_CELL_NAME"
  else
    feature_cell_state="绿"
    echo "  ✓ $FEATURE_GATED_TESTS_CELL_NAME：${#gated_rows[@]} 个挂特性的测试文件（特性 $feature_list）每个都编进来、跑到了用例，共 ${ran_total} 条（含复用；$( (( feature_plan_ok )) && printf '按条：%s' "$(plan_report "$feature_plan_output.kept" 0)" || printf '按条复用算不出，全部现判')）"
  fi
  return 0
}

# ── 快档：crash-case-replay 格与 feature-gated-tests 格 ─────────
if [[ "$crash_point_replay_tier" == quick ]]; then
  quick_red_cells=()
  quick_cell_states=()
  quick_tier_log=""
  if crash_point_replay_runs_cell "$CRASH_CASE_REPLAY_CELL_NAME"; then
    quick_tier_report=""
    quick_tier_plan_red=0
    if run_checker_quick_tier && check_crash_case_blocks && quick_tier_reuse_held; then
      (( quick_tier_ran )) && quick_tier_log="$crash_point_replay_scratch_directory/quick-singlefs-checker-tier.log"
      # 不带特性那一趟用例报的本次未跑（文件头「多机配置开了 GPU」）：逐格汇总里点出来，不算全绿
      if (( crash_point_replay_plain_not_run_total > 0 )); then
        if [[ "$crash_point_replay_cell_state" == 绿 ]]; then crash_point_replay_cell_state="不带特性那一趟有 ${crash_point_replay_plain_not_run_total} 处本次未跑"
        else crash_point_replay_cell_state+="；不带特性那一趟有 ${crash_point_replay_plain_not_run_total} 处本次未跑"; fi
      fi
      quick_cell_states+=("$CRASH_CASE_REPLAY_CELL_NAME：$crash_point_replay_cell_state")
    else
      quick_red_cells+=("$CRASH_CASE_REPLAY_CELL_NAME")
      quick_cell_states+=("$CRASH_CASE_REPLAY_CELL_NAME：红")
    fi
  fi
  if crash_point_replay_runs_cell "$FEATURE_GATED_TESTS_CELL_NAME"; then
    if run_feature_gated_tests_cell "$quick_tier_log"; then
      quick_cell_states+=("$FEATURE_GATED_TESTS_CELL_NAME：$feature_cell_state")
    else
      quick_red_cells+=("$FEATURE_GATED_TESTS_CELL_NAME")
      quick_cell_states+=("$FEATURE_GATED_TESTS_CELL_NAME：红")
    fi
  fi
  quick_cell_summary=""
  for quick_cell_state in "${quick_cell_states[@]}"; do quick_cell_summary+="${quick_cell_summary:+；}${quick_cell_state}"; done
  echo "  · 逐格：${quick_cell_summary}"
  if (( ${#quick_red_cells[@]} > 0 )); then
    echo "  ✗ 快档有 ${#quick_red_cells[@]} 格判红：${quick_red_cells[*]}（各自的出路在上面）"
    echo "     → 怎么办：照上面那几格各自的「→ 怎么办」改完，只重跑红的格：bash .claude/gate.d/checker-tier-crash-point-replay.sh --check $(IFS=,; printf '%s' "${quick_red_cells[*]}") [项目根]"
    exit 1
  fi
  if [[ "$crash_point_replay_sample_mode" == 1 ]]; then
    echo "  ! 样本档（.crash-point-replay-sample-tools/quick-tier，假 cargo）：快档流程判绿，退 3 不退 0——这一趟没跑过真的 checker 档"
    echo "     → 真跑要在不带 .crash-point-replay-sample-tools 的树里跑快档。"
    exit 3
  fi
  exit 0
fi

# ── --full：逐条崩溃枚举用例跑全量，块按单元键复用 ─────────
full_started_date="$(TZ=Asia/Tokyo date +%F)"
start_over_note="不带 --start-over：块按单元键照长期库复用、只判键对不上的块"
if [[ "$crash_point_replay_start_over" == 1 ]]; then start_over_note="带 --start-over：只写不读，全部块现判、照键写回同一份长期库"; fi
echo "  · --full 开跑（${full_started_date}）：${#crash_case_rows[@]} 条崩溃枚举用例逐条跑；${start_over_note}"
# 每趟一个新目录，不拿上一趟的输出顶上
crash_point_replay_log_directory="$git_common_directory/singlefs-crash-point-replay-logs/$(date -u +%Y%m%dT%H%M%SZ)-$$"
if ! mkdir -p -- "$crash_point_replay_log_directory"; then
  echo "  ✗ 建不了这一趟的日志目录 $crash_point_replay_log_directory：全量日志与发现日志没处放"
  echo "     → 怎么办：看 $git_common_directory 可不可写、盘满没满，修好之后重跑 --full。"
  exit 1
fi
echo "  · 这一趟的全量日志与发现日志放 ${crash_point_replay_log_directory}/（log.<用例名> 与 log.<用例名>.findings.tsv，跑完不删）"
crash_point_replay_shard_driver="$ROOT/scripts/crash/crash-point-replay-shard-run.sh"
if crash_point_replay_shard_configuration_note="$(bash "$ROOT/scripts/crash/crash-point-replay-shard-configuration-check.sh" "$ROOT" 2>&1)"; then
  crash_point_replay_sharded=1
  echo "  · 双机分片：开（${crash_point_replay_shard_configuration_note//$'\n'/；}）；登记了 shard=across-machines 的用例两台各跑一片、本机 merge，别的单机跑"
else
  crash_point_replay_sharded=0
  echo "  · 双机分片：关（${crash_point_replay_shard_configuration_note//$'\n'/；}）；每条用例单机跑"
fi
blocks_supported=1
if python3 "$crash_point_replay_admission_module" crash-case-blocks "$ROOT" /dev/null >/dev/null 2>&1; then blocks_probe_exit=0; else blocks_probe_exit=$?; fi
if (( blocks_probe_exit == 5 )); then
  blocks_supported=0
  echo "  · 块判定那一行（CRASH_CASE_BLOCKS）：crates 侧还没接上（用例源码里没有 SINGLEFS_CRASH_CASE_BLOCKS_QUERY），这一趟不要求那一行、复用数取不出"
fi
red_cases=()
green_cases=()
reused_cases=()
blocks_reused_total=0
blocks_fresh_total=0
blocks_refused_total=0
for crash_case_row in "${crash_case_rows[@]}"; do
  IFS=$'\t' read -r case_key case_package case_target case_function <<< "$crash_case_row"
  case_label="${case_key#crash-case:}"
  if ! read_crash_case_command "$case_key"; then
    echo "  ✗ $case_key：准入模块交不出起用例的命令：$case_command_problem"
    echo "     → 怎么办：单跑 python3 scripts/gate/admission.py crash-case-command <根> $case_key 看它报什么（取不到 git common-dir、"
    echo "                本机核数读不到、SINGLEFS_CRASH_POINT_REPLAY_THREADS 不是正整数、每台默认重活线程数的登记取不了都在这里报）；修好之后重跑 --full。"
    red_cases+=("$case_key")
    continue
  fi
  case_log="$crash_point_replay_log_directory/log.$case_label"
  case_findings_file="$case_log.findings.tsv"
  rm -f -- "$case_log" "$case_findings_file"
  case_threads_origin_text="没设，取每台默认重活线程数的登记值"
  if [[ "$case_threads_origin" == explicit ]]; then case_threads_origin_text="显式设的"; fi
  case_threads_note="SINGLEFS_CRASH_POINT_REPLAY_THREADS=${case_threads}（${case_threads_origin_text}），本机 ${case_machine_cores} 核"
  case_sharded=0
  if [[ "$crash_point_replay_sharded" == 1 ]] && python3 "$crash_point_replay_admission_module" crash-case-shardable "$ROOT" "$case_key" >/dev/null; then case_sharded=1; fi
  case_way="单机跑：${case_command[*]}"
  if [[ "$case_sharded" == 1 ]]; then
    case_way="双机跑（本机 0/2、第二台 1/2 各写各的库，取回，本机带导入再跑一趟；bash scripts/crash/crash-point-replay-shard-run.sh --merged-log $case_key <树根> <标签> <日志> $case_target $case_function）"
    case_threads_note="本机那两趟 ${case_threads_note}，第二台那一趟的线程由双机驱动取登记的重活默认并行度（CRASH_AMPLIFICATION_PEER_THREADS 设了照它）"
  fi
  echo "  · $case_key 开跑（${case_way}；${case_threads_note}）"
  echo "    全量日志：${case_log}；发现日志：${case_findings_file}"
  if [[ "$case_sharded" == 1 ]]; then
    run_crash_case_in_two_shards "$case_key" "$case_log" "$case_target" "$case_function"
  else
    run_crash_case "$case_log" "$case_findings_file" "${case_command[@]}"
  fi
  case_run_exit=$?
# 发现表跑完就打（判红的那一趟最要看它），判定留到下面
  if report_crash_case_findings "$case_findings_file" "$case_log"; then case_findings_exit=0; else case_findings_exit=$?; fi
  if [[ "$case_run_exit" != 0 ]]; then
    [[ -f "$case_log" ]] && tail -40 "$case_log"
    if [[ "$case_sharded" == 1 ]]; then
      echo "  ✗ $case_key 判红：双机那一趟退 $case_run_exit（上面是驱动脚本的输出与导入那一趟日志的尾部）"
      echo "     → 怎么办：驱动脚本输出里判红的那一句说清卡在哪一步（工具链、某一片、账本、merge）；要单机复核，挪开本地配置 multi-host.env 再跑 --full"
      red_cases+=("$case_key")
      continue
    fi
    echo "  ✗ $case_key 判红：cargo test 退非 0（上面是它的尾部）"
    echo "     → 怎么办：单跑看细节：cargo test --release -p $case_package --test $case_target -- --include-ignored --exact $case_function --nocapture"
    echo "                断言消息里是第一处对不上的计数或违例。改代码还是改断言先想清楚是哪一种——直接改断言等于把测试关掉。"
    red_cases+=("$case_key")
    continue
  fi
  judged_lines_file="$crash_point_replay_scratch_directory/judged.$case_label"
  if ! judge_output="$(python3 "$crash_point_replay_admission_module" crash-case-judge "$ROOT" "$case_key" "$case_log" "$judged_lines_file")"; then
    printf '%s\n' "$judge_output" | sed 's/^/       /'
    echo "  ✗ $case_key 的用例跑过了，日志却判不绿（上面逐条列出）"
    echo "     → 怎么办：计数行不是恰好一行、过滤之后没跑到恰好一条用例，对一对 stage-inputs.tsv 里 $case_key 那一行第三列与用例打印的行；"
    echo "                不是全量去 crates/singlefs-checker-tier/src/crash_state_enumeration.rs 的 enumerate_crash_states 看；只起了 1 个线程看 crash_point_replay_worker_threads 读没读到 SINGLEFS_CRASH_POINT_REPLAY_THREADS，"
    echo "                真要单线程跑（比对单进程读数），显式写 SINGLEFS_CRASH_POINT_REPLAY_THREADS=1 bash .claude/gate.d/checker-tier-crash-point-replay.sh --full <根>。"
    red_cases+=("$case_key")
    continue
  fi
  if [[ "$case_findings_exit" != 0 ]]; then
    echo "  ✗ $case_key 的用例跑过了、日志也判得绿，发现日志却判红（上面发现表里的「问题：」逐条列出）"
    echo "     → 怎么办：「这一趟没跑完」看全量日志 $case_log 的尾部是 panic 还是被杀；red_states 不是 0 是有状态判红而用例没红，"
    echo "                对一对用例的计数断言与 crates/singlefs-checker-tier/src/crash.rs 的发现表；发现日志不在而全量日志里有 CRASH_POINT_REPLAY_FINDINGS 行，"
    echo "                是 SINGLEFS_CRASH_POINT_REPLAY_FINDINGS_FILE 没传进用例（单机看本阶段 run_crash_case，双机看 scripts/crash/crash-point-replay-shard-run.sh 给导入那一趟的设置）。"
    red_cases+=("$case_key")
    continue
  fi
  blocks_text="块判定那一行没要（crates 侧还没接上）"
  if (( blocks_supported )); then
    if blocks_output="$(python3 "$crash_point_replay_admission_module" crash-case-blocks "$ROOT" "$case_log")"; then blocks_exit=0; else blocks_exit=$?; fi
    if (( blocks_exit != 0 )); then
      echo "  ✗ $case_key 的块判定那一行不对（crash-case-blocks 退 $blocks_exit：$blocks_output）"
      echo "     → 怎么办：退 1 是有块判红：看全量日志里那几块的发现；退 4 是用例没打恰好一行 CRASH_CASE_BLOCKS、或 blocks 不等于 reused + fresh（接口在 scripts/gate/admission.py 文件头）。"
      red_cases+=("$case_key")
      continue
    fi
    blocks_text="$blocks_output"
    if [[ "$blocks_output" =~ 复用\ ([0-9]+)、现判\ ([0-9]+)、红\ [0-9]+、拒算\ ([0-9]+) ]]; then
      blocks_reused_total=$((blocks_reused_total + BASH_REMATCH[1])); blocks_fresh_total=$((blocks_fresh_total + BASH_REMATCH[2]))
      blocks_refused_total=$((blocks_refused_total + BASH_REMATCH[3]))
    fi
    if grep -qE '现判 0、' <<<"$blocks_output"; then
      echo "  · $case_key 复用：全部块都有键相同的判定（$blocks_output）"
      reused_cases+=("$case_key")
    fi
  fi
  threads_text="${judge_output//$'\n'/；}${judge_output:+；}${case_threads_note}"
  echo "  ✓ $case_key 判绿（${threads_text}；${blocks_text}），判日志认出的行原样："
  sed 's/^/      /' "$judged_lines_file"
  green_cases+=("$case_key")
done
full_finished_date="$(TZ=Asia/Tokyo date +%F)"
if (( ${#red_cases[@]} > 0 )); then
  echo "  ✗ --full 有 ${#red_cases[@]} 条崩溃枚举用例判红：${red_cases[*]}（这一趟判绿 ${#green_cases[@]} 条，其中全部块复用 ${#reused_cases[@]} 条；开跑 ${full_started_date}，跑完 ${full_finished_date}）"
  echo "     → 怎么办：逐条照它自己那一句「→ 怎么办」改；改完暂存，再在 HEAD + 暂存区的 worktree 里跑 --full（判过的块照键复用，只判改到的块）。"
  exit 1
fi
# 全部判绿：日志目录里留一份 all-green（只记这一趟绿过，apply-writer-patch.py 数「上次崩溃验证全绿以来打了几个补丁」用；不是复用的标记）
printf '%s\n' "${full_finished_date} ${#crash_case_rows[@]} 条崩溃枚举用例全部判绿" > "$crash_point_replay_log_directory/all-green" \
  || echo "  ! 没写成 $crash_point_replay_log_directory/all-green（apply-writer-patch.py 的提示照上一趟全绿算）"
if (( blocks_supported )); then
  echo "  ✓ 崩溃状态块：现判 ${blocks_fresh_total}、复用 ${blocks_reused_total}、拒算 ${blocks_refused_total}、本次未跑 0"
else
  echo "  ✓ 用例：现判 ${#green_cases[@]}、复用 0、拒算 0、本次未跑 0（块一级的数取不出：crates 侧还没接上 CRASH_CASE_BLOCKS，整条用例照跑）"
fi
echo "  ✓ --full 跑完（开跑 ${full_started_date}，跑完 ${full_finished_date}）：${#crash_case_rows[@]} 条崩溃枚举用例这一趟都判绿（${green_cases[*]:-无}），其中全部块复用 ${#reused_cases[@]} 条（${reused_cases[*]:-无}）；日志在 ${crash_point_replay_log_directory}/"
if [[ "$crash_point_replay_sample_mode" == 1 ]]; then
  echo "  ! 样本档（.crash-point-replay-sample-tools，假 cargo）：流程判绿，退 3 不退 0——这一趟没跑过真的崩溃点重放"
  echo "     → 真跑要在不带 .crash-point-replay-sample-tools 的树里跑 --full（建法见快档的出路句）。"
  exit 3
fi
