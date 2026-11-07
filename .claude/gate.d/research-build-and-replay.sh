#!/usr/bin/env bash
# admission: always 判的是此刻被判的仓（工作区或 --staged 的临时树），上一次的结论不替这一次作保
# run-condition: none 要 cargo：缺了就在单测那一格判红、复跑那一格由它调的 scripts/experiment-apparatus/replay.sh 编实验二进制时判红，不交给 gate.sh 预判；此外只要跑门禁本身就要的 bash、git、python3
# task-server-gpu-arguments: *   experiment-replay 格起 replay.sh，含点到 CHECKER_TIER_FEATURES 的行，由服务在最外层清场、分卡（task-server/task_server_lib.py 文件头「GPU」）
# gate-stage: research 工作区的构建与单测；入库的实验数今天还复现得出来吗
# gate-fixture: yes
# gate-category: checker-tier 类
# gate-similar: model-differential-and-scenarios.sh 它也跑 release 的 cargo test，但编的是仓根的 crates 工作区、判的是模型对拍那几段；这里编的是 research 工作区，两个工作区互相看不到对方，编译产物不共用，并进来反而要多编一份
# gate-similar: code-source-discipline.sh 它的实验源码纪律那一格静态判 research 的实验二进制与变异表（锚点、每个实验有变异表），不编译、不跑；这里真编、真跑单测与复跑，判的是编得过、跑得出同样的数；mutation-tables 格枚举的表与它同一个来源，逐条施加替换文、照跑测试，判的是每条替换文今天还编得过、还被测试抓到
# gate-similar: doc-experiments.sh 它判实验页与产物登记（索引行、产物写回、产物落在仓里），读的是 kb 与 research/results/ 的文字；这里把产物重跑一遍逐字节比，读的是实验二进制的输出
# gate-similar: code-tooling.sh 研究脚本的 --selftest 都在它的 research-script-selftests 格跑（不要 research 工作区先编好、经 selftest-process-guard.py 盯着）；这里只编 research 工作区、跑它的单测与复跑，一份研究脚本的自证都不跑
#
# gate-cell: research-unit-tests research 工作区编得过、单测全绿
# gate-cell: wallclock-asserts-registered 实验单测里按挂钟判的断言都有登记位（一行注释），没登记的判红
# gate-cell: experiment-replay 入库的纯模型实验重跑一遍，与 research/results/ 里的产物逐字节相同
# gate-cell: mutation-tables 研究侧每张变异表逐条施加替换文、照跑测试，没被测试抓到或编不过的一条就红
#   research-unit-tests  cargo build --release --all-features 编整个 research 工作区（带特性的 5 个 bin：e162 三个、e163、e174 也编，计划里也列它们的单测；每趟照编：「编得过」判的是整个目标，
#                        闭包够不到的死码在 deny 警告下也编不过，单元闭包罩不住；增量编，没改动时 cargo 判新鲜），再 cargo test --release --all-features --no-fail-fast（一个测试二进制红了，后面的照跑）。
#                        上游「构建与单测」只编 crates/，kb 里实测结论背后的实验二进制只有这一格编。
#                        单测读的别的实验的留存产物（scripts/experiment-apparatus/ 里按字面 concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/<名>") 写的）不在树里
#                        （按「每次提交删上一次的实验记录」归档进了版本库）的，开 test-plan 之前先从 git 历史取回、放回原处（git log --all --diff-filter=D 找最近一次删它的提交，
#                        git show <提交>^:research/results/<名>），记进草稿目录的 restored-products.list，收尾删；取回的那一份进单元键，照判、照记库。取不回判红（逐个点名），
#                        没有「本次未跑」的一档；单测输出里出现 NOT_RUN_<原因> test=… 行（读不到输入却照常返回）同样判红。
#                        按条复用：每条测试函数（lib、各 bin、tests/ 里带 #[test] 的）一条判定，键是它的单元键（scripts/gate/admission.py test-plan，
#                        闭包根 scripts/experiment-apparatus、release 档、工具链、解析出的依赖版本，判法 scripts/gate/lib_libtest_result_lines.py 的 result_line），
#                        键相同的经 cargo test -- --exact --skip <全名> 跳过，一条都不用现判就不起 cargo test；判绿的照键记进
#                        prune-server 那一份库（unit-verdicts 族，种类 singlefs-unit-verdicts/research-build-and-replay.sh:research-unit-tests，库名固定、库根不进键；
#                        scripts/gate/admission.py 文件头「判定库」）；样本档在临时 prune-server 底下起，判定另占临时服务里一份命名空间。拒算的照跑、不记；
#                        记红、撤绿、下趟现判：cargo test 退非 0 时，输出里 FAILED 的测试函数（结果行与末尾 failures: 名单）按（目标根文件，libtest 全名）
#                        对到计划里 judge 的那一行，research_unit_tests_record_red 逐包经 admission.py verdict-record --red-names 照计划里的键记红、撤掉同一把键原有的绿
#                        （打一行「· 判红记库（<包>）：…」），下一趟这几条现判、原因以「上次判红」起头；拒算的没有键，不记；
#                        记红没成（verdict-record 退非 0）带 --red 交 admission.py plan-failure：它判失灵、撤掉这几把键原有的绿（撤不掉说整道判红），下一趟不复用那几条旧绿；
#                        有红照记：同一趟里判绿的照键记绿（research_unit_tests_record_green_of_red_run，打一行「· 判绿记库（<包>）：…」），下一趟只现判红的那几条，整格照旧判红；
#                        结局认不出、结果行与汇总行对不上的那个二进制照旧一条都不记；
#                        成功句报「单测按条：现判 N、复用 M、拒算 R」，红句另列红的那几条这一趟为什么现判（变了的单元）。
#                        按条复用失灵（test-plan 退非 0、退 0 而没打 PLAN、它或 verdict-record 的复用哨兵报「复用失灵」「判定不稳」「强制现判没起作用」）：
#                        判法只在 scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」）；它判失灵，这一格照旧全部现判跑完、结论判红。
#                        样本（被判的根不是这道阶段所在的仓）另放了 .unit-verdict-sample/prior/（上一版的树）的，开跑前先拿上一版算键、照判绿记进样本自己的判定库。
#                        登记了的挂钟断言（C465：门禁自己的并发让按挂钟判的断言误红）：源码里测试函数上带 // wallclock-assert: 登记位（一行注释，装置不加任何代码），
#                        判法 scripts/gate/wallclock-assert-lint.py --list 逐行打「<文件>TAB<函数名>」。第一趟按登记名逐个给 cargo test 传 --skip（按条复用算得出时整条命令带 --exact，
#                        登记名换成计划里末段是它的那几条全名；算不出时照旧 cargo test --release --all-features -- --skip <名>…，按子串跳），不设环境变量、不加 --show-output；
#                        随后在同一格里另起一趟单线程跑这几条：cargo test --release --all-features --no-fail-fast -- [--exact] <名…> --test-threads=1，红判红；
#                        登记名在那一趟的输出里没有「test … ok」行（那一趟没起或没跑到）也判红。第一趟的输出里找 ^test .*<函数名> ... 行：登记的名字出现了判红（登记了却跑了：--skip 没生效）。
#                        这几条每趟都量，不记进判定库；成功句报「挂钟断言 K 条在单线程那一趟跑了」。登记位写坏时判法 --list 退 1，这一格不数、不传 --skip、不起那一趟，那一错归 wallclock-asserts-registered 格。
#   wallclock-asserts-registered  python3 scripts/gate/wallclock-assert-lint.py：scripts/experiment-apparatus/*.rs 的 #[test] 里按挂钟判的断言要带 // wallclock-assert: 登记位
#                        （或怕不着负载的 // wallclock-assert-load-safe:），缺了判红、点名文件:行号；带了而断言里没有挂钟量也判红；登记名是别的未登记测试名的子串
#                        （--skip 是子串匹配，会连那一条一起跳过）也判红。判法、登记位写法在判法脚本文件头；
#                        判法立之前就在仓里还没复核的在 .claude/gate.d/wallclock-assert-lag.tsv（只缩不涨），成功句另报条数，并往 $GATE_NOT_RUN_FILE 报一行。
#   experiment-replay    不整格问复用、不比树、不比 HEAD：派给 scripts/experiment-apparatus/replay.sh，它按行复用（下面「要跑时」那一段）。
#                        replay.sh 退 77（C350：整趟全是计时行的负载关，本次不判）不当红：这一格报「复跑整趟本次不判（负载高）」、往 $GATE_NOT_RUN_FILE 报一行、退 77；
#                        退 75（它的判定库在 prune-server 里，服务不可用）不当红也不当绿：这一格报「复跑没判：prune-server 不可用」、退 75（整道由 stage-cells 汇总成 75）；
#                        别的退出码照旧（非 0 判红）。
#                        不指定范围时派给 scripts/experiment-apparatus/replay.sh 的是表里的全部行（exact 与 timing 都派，timing 行照 replay.sh 自己的判法：负载关等写在它文件头）。
#                        对不上不一定是代码坏了，也可能是产物该更新，两种都要人判，所以判红。
#                        replay.sh 按行复用（它文件头「按行复用」：每一行的行键是装置入口的单元闭包、参数、GPU 开关与特性集、期望产物内容、工具链、
#                        解析出的依赖版本，算法在 scripts/gate/admission.py 文件头「复跑表每一行的行键」；键相同、判过的那一行不跑，全部行都复用的一趟不起编译、不清场）：
#                        这一格读它末尾那一行 REPLAY_TOTAL，报「复跑：现判 N 行、复用 M 行、拒算 R 行、负载高没判 K 行」；
#                        没有那一行（老的或假的 replay.sh）报复用数取不出，不写 0。留存产物不在树里的行 replay.sh 从 git 历史取回再比，取不回判红。
#   mutation-tables      研究侧变异表（scripts/mutation/tables/<名>.tsv）逐张、逐条施加替换文、照跑测试：每张调
#                        scripts/mutation/mutate.sh <bin> <名>.rs <表>（判法、cargo 跑什么、退出码都在它文件头；
#                        每个 cargo 带的 RUSTFLAGS=…--cap-lints warn 在它文件头「deny 警告」，这里不另加）；
#                        一条没被任何测试抓到（❌ 没红）、替换文编不过（⏭ 无效）、测试进程没跑完（💥）、抓名字的规则有盲区（⚠️）整格红，逐条点名表与条目、打没红的说明或第一行 error。
#                        枚举对象与 code-source-discipline 的 mutation-tables 格同一个来源：scripts/mutation/tables/ 下每张 .tsv 对 scripts/experiment-apparatus/ 下同名的 .rs；
#                        bin 名经 mutate.sh --binary-of <名>.rs 取（与它核对 bin 的读法同一份）。没有同名 .rs 的表（shell 探针、Python 驱动、crates 那一侧的）
#                        取不出（bin、源文件、表）三元组，逐张点名、往 $GATE_NOT_RUN_FILE 报本次未跑、不算绿；一张都判不了退 77。
#                        按条复用归 mutate.sh（它文件头「按条入库」：每条变异一条判定，键是原文、替换文、包住原文的单元闭包、cargo 参数与特性、工具链、依赖版本，
#                        库在 prune-server 那一份（mutation-units 族，种类 singlefs-mutate-library，库名固定，mutate.sh 文件头「按条入库」），
#                        不进 /tmp；判红的（没红、编不过、没跑完）照键记红、撤绿、下一趟现判，不当绿复用）：这一格不另建库、不整格问复用；一张表的条全部复用时 mutate.sh 一条都不编。
#                        这一格读每张表末尾「计数：」那一行，逐张报「现判 N 条、复用 M 条、拒算 R 条」，成功句报合计与「本次未跑 K 张表」；
#                        红的那一条另报 mutate.sh 写的「现判：[<名>] <原因>」（键对不上时是变了的单元）。
#                        编译目录一个、这一趟自己的：这一格的临时目录（mktemp -d）里的 target/，各表串行增量编（编译产物，不是判定库；每张表的 mutate.sh 自己再按它的工作进程数各编各的副本），收尾连同临时目录删
#                        （自己用完自己删，不跨轮复用）。
#                        挂了 required-features 的 bin 由 mutate.sh 带上 --features；E162 的 rocksdb 要的 BINDGEN_EXTRA_CLANG_ARGS 没设时取
#                        scripts/crash/crash-amplification-lib.sh 的 CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS（与 门禁 crash-point-replay 带特性那一趟同一处）。
# 格结构经 lib/stage-cells.sh。
#
# 读的环境变量：
#   内存上限不在这里定：这一道整条一个任务经 task-server/task submit 交，上限由服务按记录给、撞了放大重排（records/2026-10-30-编译绕过内存总上限.md 收尾批 M03），
#   cargo test 在这一道里面直接起、不另套内存包装。
#
#   bash .claude/gate.d/research-build-and-replay.sh [项目根]                                四格都跑
#   bash .claude/gate.d/research-build-and-replay.sh --list                                  逐行打格名与判什么，不跑格
#   bash .claude/gate.d/research-build-and-replay.sh --check research-unit-tests [项目根]     只跑单测那一格
#   bash .claude/gate.d/research-build-and-replay.sh --check wallclock-asserts-registered [项目根]  只跑挂钟断言登记位那一格
#   bash .claude/gate.d/research-build-and-replay.sh --list-items [项目根]                    逐行打 replay.sh 表里的实验号、exact 还是 timing，不编、不跑；
#                                                                                                         末尾另打一行复跑表行键对不上的行数（admission.py replay-row-inputs，只算键）
#   bash .claude/gate.d/research-build-and-replay.sh --check experiment-replay --item E142 [项目根]   只复跑点名的实验
#     --item 给几次取并集，timing 行也照跑；成功行写明只跑了点名的几个，不是整格结论；实验号不在表里退 2
#   bash .claude/gate.d/research-build-and-replay.sh --check mutation-tables [项目根]  只跑变异表那一格
#
# 样本：fixtures/research-build-and-replay.sh/ 下各份，各放 .gate-cells 只点名它判的那一格：
#   research-unit-tests-red    research 工作区里有一条单测红着的小仓，判红
#   research-unit-tests-green  一条单测全绿、scripts/ 下一份脚本都没有，判绿，成功行报一个测试批次（setup.sh git init：按条复用的判定库名取 git common-dir，
#                              不是 git 工作树的样本根算不出库名，这一格会判成按条复用失灵；plan-sentinel-red 同理）
#   green                      带假 replay.sh 的小仓，表里两行 exact、一行 timing：不指定范围，三行都派给 replay.sh（假 replay.sh 打收到的参数：一个都没有，表里 3 行）
#   red                        表一行都没有，判红
#   mutation-tables-red-uncompilable  最小 research 工作区（一个 bin、warnings = "deny"，bin 里带一条会被抓的单测），一张两条的表里一条替换文编不过（引了没有的名字）、另一条被单测抓到，判红、点名表与那一条
#   mutation-tables-red-not-caught    同形的工作区，一张两条的表里一条替换文编得过但没有任何测试抓到（加改成乘：2×2 与 2+2 一样），判红、点名那一条「没红」
#   mutation-tables-green             同形的工作区，两条替换文都编得过、都被单测抓到，另有一张没有同名 .rs 的表（逐张列进本次未跑）；
#                              样本根放 .mutation-tables-sample/run-twice：按条判定库放这一格自己的临时目录，接连判两趟，第二趟两条都复用、一条都不跑
#   mutation-tables-row-key           同形的工作区、run-twice，另放 .mutation-tables-sample/second-round/（第二趟之前盖上：表里第 2 条的替换文改一个字）：
#                              第二趟第 1 条复用、第 2 条现判（对照样本 T14 的 :2），判绿
#   mutation-tables-deny-green        同形的工作区（warnings = "deny"），一条替换文既改了被测行为（被抓到）、又让一个函数再没人调、留下死代码：deny 下编不过，
#                              mutate.sh 给每个 cargo 带的 --cap-lints warn 压成警告，判绿
#   wallclock-asserts-registered-red    scripts/experiment-apparatus/ 下一份 .rs 里有一个按挂钟判的断言（.elapsed() 比大小）的测试，没登记位，判红、点名文件:行号
#   wallclock-asserts-registered-green  同形的一份 .rs：一个带 // wallclock-assert: 的测试、一个带 // wallclock-assert-load-safe: 的测试，判绿并报登记 1 条、load-safe 1 条
#   replay-load-skipped        带假 replay.sh（退 77）的小仓：复跑那一格认成本次未跑，整道退 77、输出里有「复跑整趟本次不判（负载高）」
#   experiment-replay-reused   带假 replay.sh 的小仓，末行打一行 REPLAY_TOTAL（现判 1 行、复用 2 行、拒算 0 行、负载高没判 0 行）：这一格照它报复用数，判绿
#   research-unit-tests-unit-key  research 小工作区（一个 lib，测试模块里三条单测），setup.sh git init、照这一版拷一份上一版进 .unit-verdict-sample/prior/、
#                              在上一版里删掉第三条单测（对照样本 N06：测试模块加一条单测，别的不作废）：现判 1、复用 2，cargo test 带 --exact --skip 那两条，判绿
#   research-unit-tests-red-over-green  记红、撤绿、下趟现判：research 小工作区（一个 lib、三条单测，第三条读样本根下的开关文件 .red-switch，不进键），setup.sh git init、
#                              照这一版逐字拷一份上一版进 .unit-verdict-sample/prior/（三条照判绿记进库，同一把键），.unit-verdict-sample/red-over-green 在：这一格把计划里键相同的复用行改成现判行（样本根不抽复用哨兵，同键现判由这一格自己在样本档上做：research_unit_tests_plan 的 force_judge），
#                              三条照样现判，第三条判红、照键记红盖掉绿（「· 判红记库（tiny）：1 条记红…」），样本档接着照常再算一次计划：复用 2 条、现判第三条、
#                              原因以「上次判红」起头，退 1
#   research-unit-tests-red-record-failed  记红没成、下一趟不复用旧绿：同 research-unit-tests-red-over-green，另放 .unit-verdict-sample/red-record-failure
#                              （admission.py verdict-record 照它当判红没记成、退 1）：plan-failure 带 --red 判失灵、撤掉第三条那条绿（「同一把键原有的绿已撤掉 1 条」），
#                              样本档接着再算一次计划：复用 2 条、现判第三条、原因「库里没有这一条」，退 1
#   research-unit-tests-red-records-green  有红照记：research 小工作区（一个 lib 两条单测、tests/ 下一个集成测试二进制一条单测），setup.sh git init、没有上一版；
#                              lib 里一条单测总判红、另一条与集成测试那一条判绿（集成测试的二进制排在 lib 后面，不带 --no-fail-fast 它不跑）。
#                              .unit-verdict-sample/reread-after-red 在：判红那一条照键记红，判绿的两条照键记绿（「· 判绿记库（tiny）：RECORD recorded=2」），
#                              样本档接着再算一次计划：复用 2 条、只现判红的那一条（原因以「上次判红」起头），退 1
#   research-unit-tests-plan-exit-red  按条复用失灵：同 research-unit-tests-green 的小仓，样本根放 .unit-verdict-sample/plan-failure 写 exit-1（test-plan 退 1），
#                              照旧全部现判、单测都过，判红、点名 test-plan 退 1
#   research-unit-tests-plan-sentinel-red  同形，写 sentinel-reuse-failed（test-plan 的复用哨兵报复用失灵），判红、点名哨兵那一句
#   research-unit-tests-plan-unavailable  同形，写 exit-75（test-plan 装 prune-server 不可用、退 75）：plan-failure 判不可用，这一格不起 cargo test、退 75，整道 75
#   research-unit-tests-archived-retrieved  研究小仓（根上一个包，一条单测按字面 concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/sample-product.out") 读一份留存产物），
#                              setup.sh git init、把那份产物先入库再删（只在 git 历史里）：这一格从 git 历史取回、单测读得到，判绿，输出里有「从 git 历史取回」
#   research-unit-tests-archived-unrecoverable  同形，那份产物 git 历史里也没有：这一格判红、点名「取不回」，一个 cargo test 都不起
#   wallclock-single-thread-green  研究小仓（一个包、两条单测，其中一条在 scripts/experiment-apparatus/ 顶层的 .rs 里登记成挂钟断言）：第一趟跳过它，
#                              随后单线程那一趟跑到它、判绿，成功行报「挂钟断言 1 条在单线程那一趟跑了」
#   wallclock-single-thread-red    同形，那条登记了的单测在单线程那一趟里红（读样本根下的开关文件 .red-switch，第一趟跳过它所以看不见）：判红、点名它
#   experiment-replay-rocksdb-entry-refused  带假 replay.sh（同 experiment-replay-reused）的小仓，crates/singlefs-verification/Cargo.toml 依赖 rocksdb：
#                              起 replay.sh 之前 rocksdb-prebuilt.py entry 判红（样本根没有 Cargo.lock，进程环境有没有取值都红），这一格退 1、replay.sh 没起
#   experiment-replay-unavailable  带假 replay.sh 的小仓：打一句 prune-server 不可用后退 75，这一格报「复跑没判：prune-server 不可用」、整道退 75
#   样本不起真 cargo：样本根开跑时写 .research-fake-tools/cargo（lib/research-fake-cargo.py 的拷贝，登记表 samples:.research-fake-tools），PATH 前面放它；
#   它照小工作区的源码文本出 cargo 的输出（build 恒绿、test 照 assert_eq! 与开关文件算红绿并打 libtest 形态的行、[[bin]] 写了 path 的先照 missing_ 起头的名字与 deny 下没人调的函数判编不编得过，--bin 只跑那一个；写法在它的文件头）。
#   mutation-tables 的样本里按条判定库与编译目录都是这一格自己的临时目录，跑完删；run-twice 那两份第二趟读第一趟记的。mutation-tables 五份经 mutate.sh 调假 cargo test 一个小 bin；
#   research-unit-tests-unit-key 的判定在临时 prune-server 里这一份样本的命名空间（样本档在临时服务底下起，setup.sh git init 只给它一棵 git 树），临时服务收尾即删；
#   wallclock-asserts-registered 两份只读样本里的 .rs，不起 cargo。
# 弄坏开关（打开后样本判错；一条开关一行「变量=值」，下面逐行点名判错的样本，括号里是那一份出不来的那一句；
# 门禁 code-tooling 的 stage-break-switch-replay 格逐条设上、经 stage-selftest.sh 喂这一道的全部样本，判错的样本要与点名的相等）：
#   STAGE_CELLS_BREAK=red-swallowed  归共用库 lib/stage-cells.sh：红格按绿记、整道退 0（期望退 1 的样本都实得退 0）
#     experiment-replay-rocksdb-entry-refused、mutation-tables-red-not-caught、mutation-tables-red-uncompilable、red、research-unit-tests-archived-unrecoverable、
#     research-unit-tests-plan-exit-red、research-unit-tests-plan-sentinel-red、research-unit-tests-red、research-unit-tests-red-over-green、
#     research-unit-tests-red-record-failed、research-unit-tests-red-records-green、wallclock-asserts-registered-red、wallclock-single-thread-red
#   STAGE_CELLS_BREAK=marker-ignored  归共用库：不认样本根的 .gate-cells，每份样本把登记的四格都跑（汇总那句「跑了 1 格（样本根的 .gate-cells 点名）」与红句「1 格里 1 格红」找不到；
#     树里没有 replay.sh 表的样本，复跑那一格读不到一行实验而判红，期望退 0 或 75 的实得退 1）
#     experiment-replay-unavailable、green、mutation-tables-deny-green、mutation-tables-green、mutation-tables-red-not-caught、mutation-tables-red-uncompilable、
#     mutation-tables-row-key、red、replay-load-skipped、research-unit-tests-archived-retrieved、research-unit-tests-archived-unrecoverable、research-unit-tests-green、
#     research-unit-tests-plan-unavailable、research-unit-tests-red、research-unit-tests-red-over-green、research-unit-tests-red-record-failed、research-unit-tests-red-records-green、
#     research-unit-tests-unit-key、wallclock-asserts-registered-green、wallclock-asserts-registered-red、wallclock-single-thread-green、wallclock-single-thread-red
#   RESEARCH_BUILD_AND_REPLAY_BREAK=not-caught-ignored  mutation-tables 不看 mutate.sh 退 1（没红）与 ❌ 行，没被测试抓到的表也当绿（只编不跑）
#       mutation-tables-red-not-caught（期望退 1、输出里有「[两条之二_加改成乘] 一个测试都没红」，实得退 0）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=invalid-rows-ignored  mutation-tables 不看 ⏭ 行（替换文编不过），退 0 的表里有 ⏭ 也当绿
#       mutation-tables-red-uncompilable（期望退 1、输出里有「[两条之二_引了没有的名字] 变异导致编译失败」，实得退 0）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=no-cap-lints  mutation-tables 起 mutate.sh 时，PATH 前面放一个垫片 cargo（这一格自己的临时目录里）：它先 env -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS 再交给真 cargo，
#       mutate.sh 带的 --cap-lints warn 不管放在 RUSTFLAGS 还是 CARGO_ENCODED_RUSTFLAGS 里都到不了 rustc。
#       不靠设 CARGO_ENCODED_RUSTFLAGS：mutate.sh 看见它就把 cap 接进去，那一招顶不掉了。mutate.sh 自己的 MUTATE_BREAK 只在它的 --selftest 起的子进程里认，门禁里设了它 mutate.sh 退 2（要保留），所以样本上证红用这一个
#       mutation-tables-deny-green（期望退 0，实得退 1、输出里有「[两条之二_改加法并让加倍没人调] 变异导致编译失败」）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=skip-filter-dropped  research-unit-tests 起 cargo test 时不传 --skip：登记了的挂钟断言在门禁的负载下照跑，输出里出现它们的 test … ok / FAILED 行，这一格红「登记了却跑了：--skip 没生效」
#       wallclock-single-thread-green（期望退 0，实得退 1、输出里有「登记了却跑了」：第一趟没跳过小仓里登记了的那条挂钟断言，它在第一趟里照跑）、
#       wallclock-single-thread-red（输出里找不到「在单线程那一趟里红了」：那条断言在第一趟里先红，退 1、红句是「research 的构建或单测没过」）。
#       别的 research-unit-tests 样本的小仓没有登记位，登记数是 0、没有 --skip 可丢，判不出这一项；真仓真跑：
#       RESEARCH_BUILD_AND_REPLAY_BREAK=skip-filter-dropped bash .claude/gate.d/research-build-and-replay.sh --check research-unit-tests
#       退 1、输出里有「登记了却跑了」。wallclock-asserts-registered 格不起 cargo，不受影响。
#   RESEARCH_BUILD_AND_REPLAY_BREAK=archived-restore-skipped  research-unit-tests 不从 git 历史取回单测读的留存产物（读不到就记本次未跑）
#       research-unit-tests-archived-retrieved（期望退 0，实得退 1：单测读不到 sample-product.out）、research-unit-tests-archived-unrecoverable（退出码仍是 1：单测读不到文件红了，但没有「✗ 单测要读的 … 取不回」那一句）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=archived-unrecoverable-ignored  research-unit-tests 取不回的留存产物不打 ✗、不判红，往下照跑（只打一行 · 说取不回）
#       research-unit-tests-archived-unrecoverable（输出里找不到「✗ 单测要读的 research/results/sample-product.out 不在树里，git 历史里也取不回」；往下照跑的单测读不到文件，退出码仍是 1）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=wallclock-single-run-dropped  research-unit-tests 不另起单线程那一趟（记挂钟断言本次未跑）
#       wallclock-single-thread-red（期望退 1、输出里点名那条挂钟断言红在单线程那一趟，实得退 0）、
#       wallclock-single-thread-green（成功行里找不到「挂钟断言 1 条在单线程那一趟跑了」：单线程那一趟没起，这句不报）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=replay-reuse-unreported  experiment-replay 不读 replay.sh 的 REPLAY_TOTAL 行，复用数不报
#       experiment-replay-reused（输出里找不到「复跑：现判 1 行、复用 2 行、拒算 0 行、负载高没判 0 行」）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=row-key-with-product-name  experiment-replay 给 replay.sh 设 ADMISSION_BREAK=row-key-with-product-name：复跑表每一行的行键另加产物文件名（改名就作废）。
#       样本上证不了（要真的 replay.sh 与 research 工作区；点名的样本集合是空的）；同一个开关在 python3 scripts/gate/admission.py --selftest 的「复跑表行键」R03 那一格转红
#       （只改产物文件名、内容不变的那一行期望复用、实得现判）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=mutation-key-whole-table  mutation-tables 给 mutate.sh 的按条判定库按整张表的 sha256 分开放（表里改一条，整张表的条都作废）
#       mutation-tables-row-key（输出里找不到第二趟的「现判 1 条、复用 1 条」：第 1 条没改也被判成现判）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=unit-tests-without-library  research-unit-tests 的计划里键相同的复用行一律改成现判行（一条都不复用）
#       research-unit-tests-unit-key（输出里找不到「现判 1、复用 2」：只加了一条单测，另两条也现判）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=red-run-green-not-recorded  research-unit-tests 在 cargo test 退非 0 的那一趟不记判绿的（有一条红，判完的绿一条都不记，下一趟全部现判）
#       research-unit-tests-red-records-green（输出里找不到「· 判绿记库（tiny）：RECORD recorded=2」，再算一次计划答的是「复用 0 条」而不是「复用 2 条」）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=fail-fast  research-unit-tests 起 cargo test 不带 --no-fail-fast（lib 那个二进制红了，集成测试的二进制不跑、不记）
#       research-unit-tests-red-records-green（输出里找不到「· 判绿记库（tiny）：RECORD recorded=2」，实得 recorded=1；再算一次计划答「复用 1 条」）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=red-not-recorded  research-unit-tests 判红时不交 verdict-record --red-names（只记绿，判红的不记红、不撤同一把键原有的绿）
#       research-unit-tests-red-over-green（输出里找不到「· 判红记库（tiny）：1 条记红」，再算一次计划答的是「复用 3 条」而不是第三条「上次判红」）
#       research-unit-tests-red-records-green（输出里找不到「· 判红记库（tiny）：1 条记红」，再算一次计划里红的那一条现判的原因不是「上次判红」）
#       research-unit-tests-red-record-failed（不交红的就没有记红没成那一句、不撤绿：再算一次计划答的是「复用 3 条」）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=red-record-failure-kept-green  research-unit-tests 记红没成只打一行「!」、不交 plan-failure（同一把键原有的绿留着，下一趟照复用）
#       research-unit-tests-red-record-failed（输出里找不到「同一把键原有的绿已撤掉 1 条」，再算一次计划答的是「复用 3 条」而不是第三条「库里没有这一条」）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=rocksdb-entry-unchecked  experiment-replay 起 replay.sh 之前不调 scripts/hosts/rocksdb-prebuilt.py entry
#       experiment-replay-rocksdb-entry-refused（期望退 1、输出里有「rocksdb-prebuilt entry：」，实得退 0：假 replay.sh 照起了）
#   RESEARCH_BUILD_AND_REPLAY_BREAK=replay-unavailable-as-red  experiment-replay 把 replay.sh 退 75 照别的非 0 判红（服务不可用当成复跑红了）
#       experiment-replay-unavailable（期望退 75，实得退 1）
#   ADMISSION_BREAK=plan-unavailable-as-failure  归 scripts/gate/admission.py：计划退 75 照复用失灵判红
#       research-unit-tests-plan-unavailable（期望退 75，实得退 1）
#   ADMISSION_BREAK=plan-failure-not-red  归 scripts/gate/admission.py：按条复用失灵也不判红（退回全部现判）
#       research-unit-tests-plan-exit-red、research-unit-tests-plan-sentinel-red（期望退 1，实得退 0）
#   ADMISSION_BREAK=red-record-failure-swallowed  归 scripts/gate/admission.py：记红没成只打一行 !、照退 0，plan-failure 不撤绿
#       research-unit-tests-red-record-failed（输出里找不到「同一把键原有的绿已撤掉 1 条」，再算一次计划答的是「复用 3 条」）
set -uo pipefail
source "$(dirname "${BASH_SOURCE[0]}")/../scripts/preflight.sh"
preflight "${BASH_SOURCE[0]}" "$@"; set -- ${PREFLIGHT_ARGUMENTS[@]+"${PREFLIGHT_ARGUMENTS[@]}"}
if ! declare -F preflight >/dev/null; then
  echo "  ✗ research-build-and-replay.sh：source .claude/scripts/preflight.sh 没成（上面一行报了找不到），preflight 没有定义、参数已被清空，拒绝往下跑（不拦的话 --check 与 --list 会变成全跑）"
  echo "     → 怎么办：这棵树缺 .claude/scripts/preflight.sh：建副本时连 .claude/scripts 与 .claude/singlefs-ai-sop 一起拷（scripts/hosts/minimal-copy.py 逐条加 --with .claude/scripts=… 与 --with .claude/singlefs-ai-sop=…），或回主工作区跑"
  exit 2
fi
STAGE_DIR="$(cd "$(dirname "$0")" && pwd)"
STAGE_REPOSITORY="$(cd "$STAGE_DIR/../.." && pwd)"
STAGE_NAME="$(basename "$0")"
source "$STAGE_DIR/lib/stage-cells.sh"

stage_cell research-unit-tests cell_research_unit_tests "research 工作区编得过、单测全绿" \
  "先修好编不过或红着的单测（kb 里的实测结论全靠它们背书）；改完只重跑这一格：bash .claude/gate.d/research-build-and-replay.sh --check research-unit-tests"
stage_cell wallclock-asserts-registered cell_wallclock_asserts_registered "实验单测里按挂钟判的断言都有登记位（一行注释），没登记的判红" \
  "按判红点名的文件:行号，在 #[test] 与 fn 之间写一行 // wallclock-assert: <量的是什么>（只写注释，装置代码不动）；负载再高也不会红的写 // wallclock-assert-load-safe: <为什么>；登记名撞上别的测试名的，改名或把那一条也登记；改完只重跑这一格：bash .claude/gate.d/research-build-and-replay.sh --check wallclock-asserts-registered"
stage_cell experiment-replay cell_experiment_replay "入库的纯模型实验重跑一遍，与 research/results/ 里的产物逐字节相同" \
  "对不上先判是代码坏了还是产物该更新（两种都要人判）；只重跑红的那几个实验：bash .claude/gate.d/research-build-and-replay.sh --check experiment-replay --item <实验号>"
stage_cell mutation-tables cell_mutation_tables "研究侧每张变异表逐条施加替换文、照跑测试，没被测试抓到或编不过的一条就红" \
  "改那一行替换文（编得过、能被测试抓到）或补测试，或装置改回能编；表归实验执行员改。单张核：在 scripts/experiment-apparatus/ 里 bash ../mutation/mutate.sh <bin> <源文件> ../mutation/tables/<表>；改完只重跑这一格：bash .claude/gate.d/research-build-and-replay.sh --check mutation-tables"
stage_cells_parse "$@"; set -- ${STAGE_CELLS_REST[@]+"${STAGE_CELLS_REST[@]}"}

list_items=0; root_argument=""; named_experiments=()
while (($#)); do
  case "$1" in
    --list-items) list_items=1; shift ;;
    --item)
      if (($# < 2)) || [[ -z "$2" || "$2" == -* ]]; then
        echo "  ✗ --item 后面缺实验号"
        echo "     → 怎么办：写成 --item <实验号>（例 --item E142），几个实验就给几次；可点的号用 --list-items 看。"
        exit 2
      fi
      named_experiments+=("$2"); shift 2 ;;
    --item=*) named_experiments+=("${1#--item=}"); shift ;;
    -*)
      echo "  ✗ 认不出的选项 $1"
      echo "     → 怎么办：只认 --list、--check <格名>[,<格名>…]、--list-items、--item <实验号> 与一个项目根；格名用 --list 看。"
      exit 2 ;;
    *) root_argument="$1"; shift ;;
  esac
done
ROOT="${root_argument:-$STAGE_REPOSITORY}"
cd "$ROOT" || exit 2
# 样本（被判的根不是这道阶段所在的仓、也不是 --staged）不起真 cargo（文件头「样本」）：样本根里 .research-fake-tools/cargo 是 lib/research-fake-cargo.py 的拷贝
#   （登记表第二列 samples:.research-fake-tools 的假工具目录），PATH 前面放它；build、test 照样本小工作区的源码文本出 cargo 的输出，-V 只打一行版本
if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" ]]; then
  if mkdir -p "$ROOT/.research-fake-tools" && cp "$STAGE_DIR/lib/research-fake-cargo.py" "$ROOT/.research-fake-tools/cargo" && chmod +x "$ROOT/.research-fake-tools/cargo"; then
    export PATH="$ROOT/.research-fake-tools:$PATH"
  else
    echo "  ✗ 写不出样本的假 cargo $ROOT/.research-fake-tools/cargo（从 $STAGE_DIR/lib/research-fake-cargo.py 拷）"
    echo "     → 怎么办：样本根要可写，.claude/gate.d/lib/research-fake-cargo.py 要在。"
    exit 1
  fi
fi
R=scripts/experiment-apparatus
REPLAY_SCRIPT="scripts/experiment-apparatus/replay.sh"
ADMISSION="$STAGE_REPOSITORY/scripts/gate/admission.py"
# 复跑表按行判定库的库名（replay.sh 文件头「按行复用」）：只当 prune-server 里的名字，库名固定、本地不建（admission.py library-files 认最后两段）
REPLAY_ROW_LIBRARY_NAME=singlefs-unit-verdicts/replay-row

# replay.sh 表里的行：编号|二进制|参数|产物|exact 或 timing。门禁靠这个形状取行，改表格式要同时改这里。
replay_rows_of_kind() { sed -n "s/^\(E[0-9]*\)|[^|]*|[^|]*|[^|]*|$1\$/\1/p" "$REPLAY_SCRIPT" 2>/dev/null; }

if ((list_items)); then
  if [[ ! -f "$REPLAY_SCRIPT" ]]; then
    echo "  ✗ 没有 $REPLAY_SCRIPT：列不出可点名的实验"
    echo "     → 怎么办：在项目根跑，或把项目根作为参数传进来。"
    exit 1
  fi
  listed=0
  while IFS= read -r experiment; do
    [[ -n "$experiment" ]] || continue
    kinds="$(sed -n "s/^${experiment}|[^|]*|[^|]*|[^|]*|\(exact\|timing\)\$/\1/p" "$REPLAY_SCRIPT" | sort -u | paste -sd, -)"
    printf '%s\t%s\n' "$experiment" "$kinds"
    listed=$((listed + 1))
  done < <({ replay_rows_of_kind exact; replay_rows_of_kind timing; } | awk '!seen[$0]++')
  ((listed)) || { echo "  ✗ 从 $REPLAY_SCRIPT 里一行都没读到：表的格式变了？"; echo "     → 怎么办：这里靠 '编号|二进制|参数|产物|exact 或 timing' 这个形状取行，改表格式要同时改这里。"; exit 1; }
  # 末尾一行：复跑表每一行的行键对不对得上按行判定库（admission.py replay-row-inputs 只算键，不编不跑；GPU 开关照没开算，开了的行键另算）。
  # 判定库在 prune-server 里（replay.sh 文件头「按行复用」）：哪几把键有判过的记录经 admission.py library-files has 问服务；服务不可用报连不上，不当键对不上
  list_library="$REPLAY_ROW_LIBRARY_NAME"
  if list_keys="$(python3 "$ADMISSION" replay-row-inputs . --gpu 0 2>/dev/null)"; then
    list_same=0; list_mismatch=0; list_refused=0
    list_verdicts=()
    while IFS=$'\t' read -r _list_identity list_key _list_units _list_note; do
      [[ -n "${list_key:-}" && "$list_key" != refused ]] && list_verdicts+=("keys/$list_key/verdict")
    done <<<"$list_keys"
    if ((${#list_verdicts[@]})) && ! list_present="$(python3 "$ADMISSION" library-files has "$list_library" "${list_verdicts[@]}" 2>/dev/null)"; then
      printf '复跑表行键\t键相同几行问不出（admission.py library-files has 没成：prune-server 不可用或库名不对；库名 %s）\n' "$list_library"
      exit 0
    fi
    while IFS=$'\t' read -r _list_identity list_key _list_units _list_note; do
      [[ -n "${list_key:-}" ]] || continue
      if [[ "$list_key" == refused ]]; then list_refused=$((list_refused + 1))
      elif grep -qxF "keys/$list_key/verdict" <<<"${list_present:-}"; then list_same=$((list_same + 1))
      else list_mismatch=$((list_mismatch + 1)); fi
    done <<<"$list_keys"
    printf '复跑表行键\t键对不上 %s 行、键相同 %s 行、拒算 %s 行（照 GPU 开关没开算；判定库在 prune-server 里，库名 %s）\n' "$list_mismatch" "$list_same" "$list_refused" "$list_library"
  else
    printf '复跑表行键\t算不出（admission.py replay-row-inputs 没成或不在 git 工作树里）\n'
  fi
  exit 0
fi

if ((${#named_experiments[@]})); then
  mapfile -t known_experiments < <({ replay_rows_of_kind exact; replay_rows_of_kind timing; } | awk '!seen[$0]++')
  unknown_experiments=()
  for experiment in "${named_experiments[@]}"; do
    found=0
    for known in ${known_experiments[@]+"${known_experiments[@]}"}; do [[ "$known" == "$experiment" ]] && found=1; done
    ((found)) || unknown_experiments+=("$experiment")
  done
  if ((${#unknown_experiments[@]})); then
    echo "  ✗ --item 点名的实验不在 $REPLAY_SCRIPT 的表里：${unknown_experiments[*]}"
    echo "     → 怎么办：实验号照 bash .claude/gate.d/${STAGE_NAME} --list-items 列的写（表里有 ${#known_experiments[@]} 个）。"
    exit 2
  fi
fi

# ── research-unit-tests ──
# research_test_packages：research 工作区里带测试的包名（scripts/experiment-apparatus/Cargo.toml 的 [package]，与 [workspace] members 各自 Cargo.toml 的 [package]），一行一个
research_test_packages() {
  python3 - "$R" <<'PY'
import glob, os, sys, tomllib
root = sys.argv[1]
def manifest(path):
    try:
        with open(path, "rb") as handle:
            return tomllib.load(handle)
    except (OSError, tomllib.TOMLDecodeError):
        return {}
top = manifest(os.path.join(root, "Cargo.toml"))
names = [top["package"]["name"]] if isinstance(top.get("package"), dict) and top["package"].get("name") else []
for member in (top.get("workspace") or {}).get("members") or []:
    for directory in sorted(glob.glob(os.path.join(root, member))):
        package = manifest(os.path.join(directory, "Cargo.toml")).get("package") or {}
        if package.get("name"):
            names.append(package["name"])
print("\n".join(dict.fromkeys(names)))
PY
}

# 带特性的 bin（Cargo.toml 的 [features]：e162-block-stores、e163-gpu、e174-gpu）也编也测：cargo 一律带 --all-features，test-plan 的单元键同样带这些特性（--feature <包>/<特性>）。
mapfile -t research_features < <(python3 - "$R/Cargo.toml" <<'PY'
import sys, tomllib
try:
    with open(sys.argv[1], "rb") as handle:
        manifest = tomllib.load(handle)
except (OSError, tomllib.TOMLDecodeError):
    sys.exit(0)
package = (manifest.get("package") or {}).get("name")
for feature in (manifest.get("features") or {}):
    if package and feature != "default":
        print(f"{package}/{feature}")
PY
)
research_feature_arguments=()
for research_feature in ${research_features[@]+"${research_features[@]}"}; do research_feature_arguments+=(--feature "$research_feature"); done

# test-plan 的请求参数（算键与记红后再算一次计划共用）：闭包根、编译档、判法与它所在的仓、带特性的 bin 的特性
research_plan_tokens=(--root "$R" --profile release --token "judge:scripts/gate/lib_libtest_result_lines.py::result_line" --token "judge-root:$STAGE_REPOSITORY" ${research_feature_arguments[@]+"${research_feature_arguments[@]}"})

# 单测按字面 concat!(env!("CARGO_MANIFEST_DIR"), "/../../research/results/<名>") 读别的实验的留存产物（scripts/experiment-apparatus/lib.rs 那一段）。不在树里
# （按「每次提交删上一次的实验记录」归档进了版本库）的，开 test-plan 之前先从 git 历史取回、放回原处：git log --all --diff-filter=D 找最近一次删它的提交，
# git show <提交>^:research/results/<名>；放回的名字记进 <草稿目录>/restored-products.list，收尾由 research_remove_restored_products 删掉。
# 取不回的逐个打 ✗ 点名，整格判红（单测读不到输入不算过，没有「本次未跑」的一档）。
# 弄坏开关 archived-restore-skipped：不取回（读不到就记本次未跑）；archived-unrecoverable-ignored：取不回也不判红、往下照跑。
research_restore_archived_products() { # <草稿目录>
  local output rc ignore_mode=fail
  research_build_broken archived-restore-skipped && return 0
  research_build_broken archived-unrecoverable-ignored && ignore_mode=ignore
  output="$(python3 - "$R" "$1/restored-products.list" "$ignore_mode" <<'PY' 2>&1
import os, re, subprocess, sys
root, record, mode = sys.argv[1:4]
pattern = re.compile(r'CARGO_MANIFEST_DIR"\),\s*"/\.\./\.\./research/results/([A-Za-z0-9_.-]+)"')
names = set()
for base, _directories, files in os.walk(root):
    if os.sep + "target" in base + os.sep:
        continue
    for file in files:
        if file.endswith(".rs"):
            names.update(pattern.findall(open(os.path.join(base, file), encoding="utf-8", errors="replace").read()))
unrecoverable = []
for name in sorted(names):
    path = os.path.join("research", "results", name)
    if os.path.exists(path):
        continue
    commit = subprocess.run(["git", "log", "--all", "--diff-filter=D", "--format=%H", "-1", "--", path], capture_output=True, text=True).stdout.strip()
    shown = subprocess.run(["git", "show", f"{commit}^:{path}"], capture_output=True) if commit else None
    if not commit or shown.returncode != 0 or not shown.stdout:
        unrecoverable.append(name)
        continue
    os.makedirs(os.path.dirname(path), exist_ok=True)
    temporary = f"{path}.restoring.{os.getpid()}"
    with open(temporary, "wb") as handle:
        handle.write(shown.stdout)
    os.rename(temporary, path)
    with open(record, "a", encoding="utf-8") as handle:
        handle.write(name + "\n")
    print(f"  · 单测要读的 research/results/{name} 不在树里，从 git 历史取回（git show {commit[:12]}^:{path}），收尾删")
for name in unrecoverable:
    if mode != "fail":
        print(f"  · 单测要读的 research/results/{name} 取不回，开关 archived-unrecoverable-ignored 下不判红")
        continue
    print(f"  ✗ 单测要读的 research/results/{name} 不在树里，git 历史里也取不回（git log --all --diff-filter=D -- research/results/{name} 找不到删它的提交，或那一版读不出）")  # gate-lint:detail
sys.exit(1 if unrecoverable and mode == "fail" else 0)
PY
)"; rc=$?
  [[ -z "$output" ]] || printf '%s\n' "$output"
  if ((rc != 0)); then
    echo "     → 怎么办：这些留存产物要在版本库里有（删它的那次提交的上一版），或重录进 research/results/；单测读不到输入不算过"
    return 1
  fi
  return 0
}
# 收尾：删掉 research_restore_archived_products 放回去的留存产物（不留给别的会话看见）
research_remove_restored_products() { # <草稿目录>
  local restored
  [[ -f "$1/restored-products.list" ]] || return 0
  while IFS= read -r restored; do [[ -n "$restored" ]] && rm -f -- "research/results/$restored"; done <"$1/restored-products.list"
  return 0
}

# research_unit_tests_plan <草稿目录>：每个包跑一次 admission.py test-plan（文件头 research-unit-tests 那一段「按条复用」），计划输出并进 <草稿目录>/plan.out、
# 各包的计划文件放 <草稿目录>/plan.<包>；四个数放进 unit_plan_judge、unit_plan_reuse、unit_plan_refused；算不出返回 1、原因放进 unit_plan_problem。
# 样本（被判的根不是这道阶段所在的仓）另放了 .unit-verdict-sample/prior/ 的，先拿上一版算键、照判绿记进这一版样本的判定库。
# 弄坏开关 unit-tests-without-library 下计划里键相同的复用行一律改成现判行（一条都不复用）。
research_unit_tests_plan() {
  local scratch="$1" package summary seeded plan_exit
  local -a tokens=("${research_plan_tokens[@]}")
  local force_judge=0
  research_build_broken unit-tests-without-library && force_judge=1
  # 样本档带 .unit-verdict-sample/red-over-green：库里已有上一版照判绿记进来的同一把键，这一格要同键现判（文件头「样本」）：样本根不抽复用哨兵
  #   （admission.py 文件头「复用哨兵」：被判的根下有 .unit-verdict-sample/ 不抽），所以这一格自己把计划里键相同的复用行改成现判行
  [[ -f .unit-verdict-sample/red-over-green ]] && force_judge=1
  unit_plan_judge=0; unit_plan_reuse=0; unit_plan_refused=0; unit_plan_problem=""; unit_plan_failure=()
  : >"$scratch/plan.out"
  mapfile -t unit_plan_packages < <(research_test_packages)
  if ((${#unit_plan_packages[@]} == 0)); then unit_plan_problem="$R/Cargo.toml 里读不出包名"; return 1; fi
  for package in "${unit_plan_packages[@]}"; do
    if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" && -d .unit-verdict-sample/prior ]]; then
      if python3 "$ADMISSION" test-plan "$ROOT/.unit-verdict-sample/prior" "$STAGE_NAME:research-unit-tests" "$package" "$scratch/seed.$package" "${tokens[@]}" \
           >"$scratch/seed.$package.out" 2>&1; then plan_exit=0; else plan_exit=$?; fi
      if ((plan_exit != 0)); then
        unit_plan_problem="样本：上一版的树上算不出键（$(tail -n 2 "$scratch/seed.$package.out" | tr '\n' ' ' | cut -c1-300)）"
        unit_plan_failure=(test-plan "$plan_exit" "$scratch/seed.$package.out"); return 1
      fi
      awk -F'\t' 'NF >= 7 && ($2 == "judge" || $2 == "reuse") { print $1 }' "$scratch/seed.$package.out" >"$scratch/seed.$package.names"
      seeded="$(python3 "$ADMISSION" verdict-record "$ROOT/.unit-verdict-sample/prior" "$STAGE_NAME:research-unit-tests" "$scratch/seed.$package" "$scratch/seed.$package.names" \
        --how "样本：上一版" 2>&1 | tail -n 1)"
      echo "  · 样本：上一版（.unit-verdict-sample/prior）$package 的 $(grep -c . "$scratch/seed.$package.names") 条测试函数照判绿记进这一版的判定库（$seeded）"
    fi
    if python3 "$ADMISSION" test-plan "$ROOT" "$STAGE_NAME:research-unit-tests" "$package" "$scratch/plan.$package" "${tokens[@]}" \
         >"$scratch/plan.$package.out" 2>&1; then plan_exit=0; else plan_exit=$?; fi
    if ((plan_exit != 0)); then
      unit_plan_problem="admission.py test-plan $package 没成（$(tail -n 2 "$scratch/plan.$package.out" | tr '\n' ' ' | cut -c1-300)）"
      unit_plan_failure=(test-plan "$plan_exit" "$scratch/plan.$package.out"); return 1
    fi
    if ((force_judge)); then
      # 键相同的复用行改成现判行，PLAN 汇总里 judge 加上 reuse、reuse 记 0；原因列写同键现判
      awk -F'\t' -v OFS='\t' '
        /^PLAN / { match($0, / judge=[0-9]+/); judged = substr($0, RSTART + 7, RLENGTH - 7); match($0, / reuse=[0-9]+/); reused = substr($0, RSTART + 7, RLENGTH - 7)
                   sub(/ judge=[0-9]+/, " judge=" (judged + reused)); sub(/ reuse=[0-9]+/, " reuse=0"); print; next }
        NF >= 7 && $2 == "reuse" { $2 = "judge"; $7 = "同键现判（样本档与弄坏开关：不读判定库）" }
        { print }' "$scratch/plan.$package.out" >"$scratch/plan.$package.forced" && mv "$scratch/plan.$package.forced" "$scratch/plan.$package.out"
    fi
    summary="$(grep '^PLAN ' "$scratch/plan.$package.out" | tail -n 1)"
    [[ -n "$summary" ]] || { unit_plan_problem="admission.py test-plan $package 没打 PLAN 那一行"; unit_plan_failure=(test-plan 0 "$scratch/plan.$package.out"); return 1; }
    unit_plan_judge=$((unit_plan_judge + $(sed -n 's/.* judge=\([0-9]*\).*/\1/p' <<<"$summary")))
    unit_plan_reuse=$((unit_plan_reuse + $(sed -n 's/.* reuse=\([0-9]*\).*/\1/p' <<<"$summary")))
    unit_plan_refused=$((unit_plan_refused + $(sed -n 's/.* refused=\([0-9]*\).*/\1/p' <<<"$summary")))
    awk -F'\t' -v package="$package" 'NF >= 7 { print package "\t" $0 }' "$scratch/plan.$package.out" >>"$scratch/plan.out"
  done
  return 0
}

# research_unit_tests_record <草稿目录> <cargo test 的输出文件> <不记的名字文件>：现判而判绿的测试函数照键记进库（结局经 lib_libtest_result_lines.py 的 binary_runs 逐条认；
# 结果行条数与汇总行对不上的那个测试二进制整段不记）。逐包打一行「· 判绿记库：…」；记不下只报、不判红；复用哨兵报红（强制现判那一条判定不稳）的经
# research_unit_plan_failure_verdict 判失灵、记 unit_plan_red=1
research_unit_tests_record() {
  local scratch="$1" output_file="$2" excluded="$3" package record_exit
  for package in "${unit_plan_packages[@]}"; do
    python3 - "$scratch/plan.out" "$package" "$output_file" "$excluded" "$STAGE_REPOSITORY/scripts/gate/lib_libtest_result_lines.py" >"$scratch/green.$package" <<'PY' || { echo "  ! 判绿记库：$package 取不出判绿的名单，这一趟这个包一条都不记"; continue; }
import importlib.util, sys
plan_output, package, log, excluded_file, library_path = sys.argv[1:6]
specification = importlib.util.spec_from_file_location("libtest_result_lines", library_path)
library = importlib.util.module_from_spec(specification)
specification.loader.exec_module(library)
by_binary = {}
for line in open(plan_output, encoding="utf-8", errors="replace"):
    columns = line.rstrip("\n").split("\t")
    if len(columns) >= 8 and columns[0] == package and columns[2] == "judge":
        by_binary.setdefault(columns[5], {})[columns[6]] = columns[1]
excluded = {line.strip() for line in open(excluded_file, encoding="utf-8", errors="replace") if line.strip()}
for run in library.binary_runs(open(log, encoding="utf-8", errors="replace").read()):
    if not run.consistent:
        continue
    for result in run.results:
        name = by_binary.get(run.source, {}).get(result.full_name)
        if name and result.outcome == "ok" and name not in excluded:
            print(name)
PY
    [[ -s "$scratch/green.$package" ]] || continue
    if python3 "$ADMISSION" verdict-record "$ROOT" "$STAGE_NAME:research-unit-tests" "$scratch/plan.$package" "$scratch/green.$package" \
         --how "cargo test --release（research 工作区，$package）" >"$scratch/record.$package.out" 2>&1; then record_exit=0; else record_exit=$?; fi
    if ((record_exit == 0)); then
      echo "  · 判绿记库（$package）：$(tail -n 1 "$scratch/record.$package.out")"
    else
      echo "  ! 判绿记库（$package）：admission.py verdict-record 没成（$(tail -n 2 "$scratch/record.$package.out" | tr '\n' ' ')），下一趟这几条照跑"
      research_unit_plan_failure_verdict verdict-record "$record_exit" "$scratch/record.$package.out"
    fi
  done
}

# research_unit_plan_failure_verdict <子命令> <退出码> <输出文件>：按条复用失灵没有，判法只在 scripts/gate/admission.py 的 plan-failure（文件头「按条复用失灵」）；
# 失灵它打 ✗ 与出路，这里记 unit_plan_red=1，这一格照旧跑完、结论判红；它退 75 是 prune-server 不可用（不是失灵），记 unit_service_unavailable=1，这一格退 75
research_unit_plan_failure_verdict() {
  local verdict_exit
  if python3 "$ADMISSION" plan-failure "$STAGE_NAME 的 research-unit-tests 格" "$@"; then return 0; else verdict_exit=$?; fi
  if ((verdict_exit == 75)); then unit_service_unavailable=1; else unit_plan_red=1; fi
}

# research_unit_tests_record_red <草稿目录> <cargo test 的输出文件>：cargo test 退非 0 的那一趟（文件头 research-unit-tests「记红、撤绿、下趟现判」）：输出里 FAILED 的测试函数
# （结果行与末尾 failures: 名单）按（目标根文件，libtest 全名）对到计划里 judge 的那一行，逐包经 admission.py verdict-record --red-names 照计划里的键记红、
# 撤掉同一把键原有的绿。拒算的没有键，不记；记不下的带 --red 交 research_unit_plan_failure_verdict（这一格已经判红）：记红没成由 admission.py plan-failure
# 判失灵、撤掉这几把键原有的绿（撤不掉说整道判红，它文件头「按条复用失灵」），复用哨兵报的失灵照打出路。
# 弄坏开关 red-not-recorded 下不交（只记绿，判红的不撤同一把键原有的绿）；red-record-failure-kept-green 下记红没成只打一行「!」、不交 plan-failure
# （同一把键原有的绿留着，下一趟照复用）
research_unit_tests_record_red() {
  local scratch="$1" output_file="$2" package record_exit red_count
  research_build_broken red-not-recorded && return 0
  for package in "${unit_plan_packages[@]}"; do
    python3 - "$scratch/plan.out" "$package" "$output_file" "$STAGE_REPOSITORY/scripts/gate/lib_libtest_result_lines.py" >"$scratch/red.$package" <<'PY' || { echo "  ! 判红记库：$package 取不出判红的名单，这个包一条都不记"; continue; }
import importlib.util, re, sys
plan_output, package, log, library_path = sys.argv[1:5]
specification = importlib.util.spec_from_file_location("libtest_result_lines", library_path)
library = importlib.util.module_from_spec(specification)
specification.loader.exec_module(library)
by_binary = {}
for line in open(plan_output, encoding="utf-8", errors="replace"):
    columns = line.rstrip("\n").split("\t")
    if len(columns) >= 8 and columns[0] == package and columns[2] == "judge":
        by_binary.setdefault(columns[5], {})[columns[6]] = columns[1]
failed, source, in_failure_list = [], None, False
for line in open(log, encoding="utf-8", errors="replace").read().split("\n"):
    running, doctests = library.RUNNING_LINE_FORM.match(line), library.DOCTESTS_LINE_FORM.match(line)
    if running or doctests:
        source, in_failure_list = (running.group("source") if running else f"doctest:{doctests.group('package')}"), False
        continue
    parsed = library.result_line(line)
    if parsed is not None:
        if parsed.outcome == "FAILED":
            failed.append((source, parsed.full_name))
        continue
    if line.rstrip() == "failures:":
        in_failure_list = True
    elif in_failure_list:
        entry = re.match(r"^    (\S+)\s*$", line)
        if entry is not None:
            failed.append((source, entry.group(1)))
        elif line.strip():
            in_failure_list = False
for name in dict.fromkeys(by_binary.get(source, {}).get(full_name) for source, full_name in failed):
    if name:
        print(name)
PY
    [[ -s "$scratch/red.$package" ]] || continue
    red_count="$(grep -c . "$scratch/red.$package")"
    : >"$scratch/red.$package.green"
    if python3 "$ADMISSION" verdict-record "$ROOT" "$STAGE_NAME:research-unit-tests" "$scratch/plan.$package" "$scratch/red.$package.green" --red-names "$scratch/red.$package" \
         --how "cargo test --release（research 工作区，$package）" >"$scratch/red-record.$package.out" 2>&1; then record_exit=0; else record_exit=$?; fi
    if ((record_exit == 0)); then
      echo "  · 判红记库（$package）：${red_count} 条记红、撤掉同一把键原有的绿，下一趟这几条现判（$(tail -n 1 "$scratch/red-record.$package.out")）"
    elif research_build_broken red-record-failure-kept-green; then
      # 弄坏开关：记红没成只打一行「!」、不交 plan-failure，同一把键原有的绿留着、下一趟照复用
      echo "  ! 判红记库（$package）：admission.py verdict-record 没成（$(tail -n 2 "$scratch/red-record.$package.out" | tr '\n' ' ')），红的没记下、同一把键原有的绿还在"
    else
      # 记红没成（admission.py 文件头「按条复用失灵」）：带 --red 交 plan-failure，它判失灵、撤掉这几把键原有的绿（撤不掉说整道判红）；这一格已经判红
      echo "  · 判红记库（$package）：admission.py verdict-record 退 $record_exit（$(tail -n 2 "$scratch/red-record.$package.out" | tr '\n' ' ' | cut -c1-300)），记红没成，交 plan-failure 判、撤同一把键原有的绿"
      research_unit_plan_failure_verdict verdict-record "$record_exit" --red "$ROOT" "$STAGE_NAME:research-unit-tests" "$scratch/plan.$package" "$scratch/red.$package" \
        "$scratch/red-record.$package.out" || :
    fi
  done
}

# research_unit_tests_record_green_of_red_run <草稿目录> <cargo test 的输出文件>：cargo test 退非 0 的那一趟（文件头 research-unit-tests「有红照记」），
# 同一趟里判绿的测试函数照键记绿，与全绿那一趟同一个 research_unit_tests_record（结局认不出、结果行与汇总行对不上的那个二进制整段不记）。
# 不记的另有：登记了的挂钟断言（<草稿目录>/wallclock.names 里的）。弄坏开关 red-run-green-not-recorded 下不记（有红就一条绿都不记）
research_unit_tests_record_green_of_red_run() {
  local scratch="$1" output_file="$2"
  research_build_broken red-run-green-not-recorded && return 0
  research_unit_tests_record "$scratch" "$output_file" "$scratch/wallclock.names"
}

# research_unit_tests_reread_after_red <草稿目录>：样本档带 .unit-verdict-sample/red-over-green 或 reread-after-red 时（文件头「样本」），判红记库之后再算一次每个包的计划，
# 打复用几条、现判哪几条与原因（记红盖掉了绿：红的那一条答「上次判红」；没盖掉：它答复用）
research_unit_tests_reread_after_red() {
  local package
  [[ -f .unit-verdict-sample/red-over-green || -f .unit-verdict-sample/reread-after-red ]] || return 0
  for package in "${unit_plan_packages[@]}"; do
    if python3 "$ADMISSION" test-plan "$ROOT" "$STAGE_NAME:research-unit-tests" "$package" "$1/reread.$package" "${research_plan_tokens[@]}" >"$1/reread.$package.out" 2>&1; then
      echo "  · 样本档：判红记库之后再算一次计划（$package）：复用 $(awk -F'\t' 'NF >= 7 && $2 == "reuse" { count++ } END { print count + 0 }' "$1/reread.$package.out") 条"
      awk -F'\t' 'NF >= 7 && $2 == "judge" { printf "    现判 %s（%s）\n", $1, substr($7, 1, 300) }' "$1/reread.$package.out"
    else
      echo "  ✗ 样本档：判红记库之后再算 $package 的计划没成（$(tail -n 2 "$1/reread.$package.out" | tr '\n' ' ')）"
      echo "     → 怎么办：看括号里 admission.py test-plan 报的错，修好样本或判定库，再单跑这一份样本"
    fi
  done
}

cell_research_unit_tests() {
  local out rc b n lint listing listing_rc name joined scratch plan_ok=1 output_file wallclock_rows first_pass_needed=1
  local registered_wallclock=() ran_wallclock=() skip_arguments=() wallclock_exact=() single_names=() single_exact=() single_output single_rc unit_counts missing_single=()
  [[ -f "$R/Cargo.toml" ]] || { echo "  ! 没有 $R/Cargo.toml，这一格无对象可判"; exit 77; }
  command -v cargo >/dev/null || {
    echo "  ✗ 没有 cargo，装不了就没法验 research 的证据"
    echo "     → 怎么办：装 Rust 工具链（curl https://sh.rustup.rs -sSf | sh），"
    echo "               或把已装的 cargo 放进 PATH。跳过这一步等于 research/ 的数字没人验过。"
    exit 1; }
  scratch="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  # shellcheck disable=SC2064  # 路径在这里就展开：退出时 local 变量已经不在
  trap "research_remove_restored_products $(printf '%q' "$scratch"); rm -rf -- $(printf '%q' "$scratch")" EXIT
  # 单测读的别的实验的留存产物：不在树里的先从 git 历史取回、放回原处（文件头 research-unit-tests 那一段），取不回判红
  research_restore_archived_products "$scratch" || exit 1
  # 先编整个 research 工作区的 release 产物（带特性的 5 个 bin 也编）：复跑那一格的 replay.sh 编的就是这一份，跑到那一格时 cargo 判它新鲜、不再编。
  # 「编得过」判的是整个目标（闭包够不到的死码在 deny 警告下也编不过），单元闭包罩不住，这一步每趟照编（增量编，没改动时 cargo 判新鲜）。
  # cargo 直接起：编译与测试线程是服务给这一条套的 capped.sh 设好的 CARGO_BUILD_JOBS、RUST_TEST_THREADS（capped.sh 只认服务套的那一层）
  out="$(cd "$R" && cargo build --release --all-features 2>&1)"
  rc=$?
  if (( rc != 0 )); then
    grep -E '^error' <<<"$out" | head -8 | sed 's/^/     /'
    echo "  ✗ research 编不过（cargo build --release --all-features 退出码 $rc）"
    echo "     → 怎么办：先修好——kb 里的实测结论全靠这些实验二进制背书，编不过就等于那些数字今天没有来源。"
    exit 1
  fi
  # C465：登记了哪几条按挂钟判的断言以判法的 --list（一行「<文件>TAB<函数名>」）为准；登记位写坏了（判法退 1）时数不出，不在这里查，那一错归 wallclock-asserts-registered 格
  lint="$STAGE_REPOSITORY/scripts/gate/wallclock-assert-lint.py"
  if [[ -f "$lint" ]] && command -v python3 >/dev/null; then
    listing="$(python3 "$lint" --list . 2>/dev/null)"; listing_rc=$?
    case "$listing_rc" in
      0) mapfile -t registered_wallclock < <(cut -f2 <<<"$listing" | sed '/^$/d' | sort -u) ;;
      1|77) ;;
      *) echo "  ✗ python3 scripts/gate/wallclock-assert-lint.py --list 退 $listing_rc（0 列出来了、1 登记位有红、77 无对象）：数不出登记了哪几条按挂钟判的断言"
         echo "     → 怎么办：单独跑 python3 scripts/gate/wallclock-assert-lint.py --list . 看它的输出，修好再重跑这一格。"
         exit 1 ;;
    esac
  fi
  # 按条复用（文件头 research-unit-tests 那一段）：每条测试函数一条判定，键相同的经 -- --exact --skip <全名> 跳过
  unit_plan_red=0; unit_service_unavailable=0
  if research_unit_tests_plan "$scratch"; then
    echo "  · 单测按条复用：$((unit_plan_judge + unit_plan_reuse + unit_plan_refused)) 条编得进的测试函数，现判 ${unit_plan_judge}、复用 ${unit_plan_reuse}、拒算 ${unit_plan_refused}"
    awk -F'\t' 'NF >= 8 && $3 == "judge" { if (judged++ < 20) printf "    现判 %s（%s）\n", $2, substr($8, 1, 300) }
                NF >= 8 && $3 == "refused" { if (refused++ < 20) printf "    拒算 %s（%s）\n", $2, substr($8, 1, 300) }
                END { if (judged > 20) printf "    …现判另有 %d 条没列\n", judged - 20; if (refused > 20) printf "    …拒算另有 %d 条没列\n", refused - 20 }' "$scratch/plan.out"
  else
    plan_ok=0
    echo "  ! 单测按条复用算不出（$unit_plan_problem）：这一趟全部现判、一条都不记库"
    ((${#unit_plan_failure[@]})) && research_unit_plan_failure_verdict "${unit_plan_failure[@]}"
    if ((unit_service_unavailable)); then
      echo "  ✗ research-unit-tests 这一格没判：prune-server 不可用（上面那一句），没起 cargo test，这一格退 75"
      echo "     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再只跑这一格：bash .claude/gate.d/research-build-and-replay.sh --check research-unit-tests"
      exit 75
    fi
  fi
  # 登记了的挂钟断言第一趟按名跳过；弄坏开关 skip-filter-dropped（文件头）：不传，它们照跑。
  # 按条复用算得出时 libtest 带 --exact（整名相同才跳），登记名换成计划里末段是它的那几条全名；算不出照旧按子串跳
  : >"$scratch/wallclock.names"
  if ((${#registered_wallclock[@]} > 0)) && ! research_build_broken skip-filter-dropped; then
    if ((plan_ok)); then
      for name in "${registered_wallclock[@]}"; do
        wallclock_rows="$(awk -F'\t' -v name="$name" 'NF >= 8 && ($7 == name || substr($7, length($7) - length(name) - 1) == "::" name) { print $7 "\t" $2 }' "$scratch/plan.out")"
        if [[ -n "$wallclock_rows" ]]; then
          while IFS=$'\t' read -r full_name plan_name; do wallclock_exact+=("$full_name"); printf '%s\n' "$plan_name" >>"$scratch/wallclock.names"; done <<<"$wallclock_rows"
        else
          wallclock_exact+=("$name")
        fi
      done
    else
      skip_arguments=(--)
      for name in "${registered_wallclock[@]}"; do skip_arguments+=(--skip "$name"); done
    fi
  fi
  if ((plan_ok)); then
    skip_arguments=(-- --exact)
    while IFS= read -r name; do [[ -n "$name" ]] && skip_arguments+=(--skip "$name"); done < <(
      awk -F'\t' 'NF >= 8 && $3 == "reuse" { skip[$7] = 1 } NF >= 8 && ($3 == "judge" || $3 == "refused") { keep[$7] = 1 }
                  END { for (name in skip) if (!(name in keep)) print name }' "$scratch/plan.out" | LC_ALL=C sort)
    for name in ${wallclock_exact[@]+"${wallclock_exact[@]}"}; do skip_arguments+=(--skip "$name"); done
    # 要现判的（现判加拒算）全是登记了的挂钟断言，或一条都没有：第一趟没有可跑的，不起 cargo test（挂钟断言另在下面单线程那一趟跑）
    if (( unit_plan_judge + unit_plan_refused == 0 )) || (( unit_plan_judge + unit_plan_refused == $(grep -c . "$scratch/wallclock.names") )); then
      first_pass_needed=0
    fi
  fi
  output_file="$scratch/cargo-test.out"
  if ((first_pass_needed)); then
    # --no-fail-fast：一个测试二进制红了，后面的二进制照跑，判完的照键记（文件头「有红照记」）；弄坏开关 fail-fast 下不带
    local -a fail_fast_option=(--no-fail-fast)
    research_build_broken fail-fast && fail_fast_option=()
    (cd "$R" && cargo test --release --all-features ${fail_fast_option[@]+"${fail_fast_option[@]}"} ${skip_arguments[@]+"${skip_arguments[@]}"}) >"$output_file" 2>&1
    rc=$?
    out="$(cat "$output_file")"
    if (( rc != 0 )); then
      echo "  ✗ research 的构建或单测没过（cargo test 退出码 $rc）"
      grep -E '^error|FAILED|panicked at' <<<"$out" | head -8 | sed 's/^/     /'
      if ((plan_ok)); then
        while IFS= read -r name; do
          awk -F'\t' -v name="$name" 'NF >= 8 && $7 == name && ($3 == "judge" || $3 == "refused") { printf "     红的 %s：%s\n", $2, substr($8, 1, 400) }' "$scratch/plan.out"
        done < <(sed -n 's/^test \(.*\) \.\.\. FAILED$/\1/p' <<<"$out" | LC_ALL=C sort -u)
        research_unit_tests_record_red "$scratch" "$output_file"
        research_unit_tests_record_green_of_red_run "$scratch" "$output_file"
        research_unit_tests_reread_after_red "$scratch"
      fi
      echo "     → 怎么办：先修好——kb 里的实测结论全靠它们背书，编不过就等于那些数字今天没有来源（「红的」那几行是各自这一趟为什么现判：变了的单元）。"
      exit 1
    fi
    # 条数只用一个 awk 数（「test result: ok. N passed; …」的第 4 段）：串几道工具的话哪一环失败，条数就静默变成「?」
    b="$(grep -c '^test result: ok' <<<"$out")"
    n="$(awk '/^test result: ok\./ { passed += $4 } END { print passed + 0 }' <<<"$out")"
    [[ "$n" =~ ^[0-9]+$ ]] || n="没数出来（awk 输出「$n」）"
    # 一个测试批次都没有：不记通过，退 77
    if (( b == 0 )); then
      echo "  ! 本次无对象可判：cargo test 退 0，但输出里一个「test result: ok」批次都没有，research 的单测没判"
      exit 77
    fi
    # 登记了的按挂钟判的断言：--skip 之后第一趟的输出里不该再有它们的 test … ok / FAILED 行（被 --skip 滤掉的测试不打任何一行）
    for name in ${registered_wallclock[@]+"${registered_wallclock[@]}"}; do
      if grep -Eq -- "^test (.*::)?${name} \\.\\.\\. " <<<"$out"; then ran_wallclock+=("$name"); fi
    done
    if ((${#ran_wallclock[@]} > 0)); then
      echo "  ✗ 登记了的挂钟断言 ${#ran_wallclock[@]} 条（${ran_wallclock[*]}）在第一趟 cargo test 的输出里以 test … 行出现：登记了却跑了，--skip 没生效，它们在门禁的负载下照跑了"
      echo "     → 怎么办：看起 cargo test 的命令里有没有 -- --skip <函数名>（判法 python3 scripts/gate/wallclock-assert-lint.py --list . 列登记名）、"
      echo "               登记名是不是被别的写法拼错；--skip 是子串匹配，函数名要和登记的一字不差。"
      exit 1
    fi
    # 单测打了 NOT_RUN_<原因> test=… 行（读不到输入却照常返回）：这一格没有本次未跑的一档，判红
    if grep -Eq 'NOT_RUN_[A-Z_]+ test=' "$output_file"; then
      echo "  ✗ 单测的输出里有 NOT_RUN_<原因> test=… 行（读不到输入却照常返回）："
      grep -E 'NOT_RUN_[A-Z_]+ test=' "$output_file" | head -8 | sed 's/^/     /'   # gate-lint:detail
      echo "     → 怎么办：它读的留存产物要在版本库里有（删它的那次提交的上一版），或重录进 research/results/；上面开 test-plan 之前的取回只认 concat!(env!(\"CARGO_MANIFEST_DIR\"), \"/../../research/results/<名>\") 写的字面路径"
      exit 1
    fi
  else
    : >"$output_file"; b=0; n=0
  fi
  # 登记了的挂钟断言另起一趟单线程跑（文件头 research-unit-tests 那一段）：红判红；登记名在那一趟的输出里没有 test … ok 行（那一趟没起或没跑到）也判红。
  # 它们每趟都量，不记进判定库。弄坏开关 wallclock-single-run-dropped：这一趟不起（记本次未跑）。
  if ((${#registered_wallclock[@]} > 0)) && ! research_build_broken wallclock-single-run-dropped; then
    single_output="$scratch/wallclock-single-thread.out"
    if ((plan_ok)); then single_names=("${wallclock_exact[@]}"); single_exact=(--exact); else single_names=("${registered_wallclock[@]}"); fi
    (cd "$R" && cargo test --release --all-features --no-fail-fast -- ${single_exact[@]+"${single_exact[@]}"} "${single_names[@]}" --test-threads=1) >"$single_output" 2>&1
    single_rc=$?
    if (( single_rc != 0 )); then
      echo "  ✗ 登记了的挂钟断言 ${#registered_wallclock[@]} 条（${registered_wallclock[*]}）在单线程那一趟里红了（cargo test 退出码 $single_rc）"
      grep -E '^error|FAILED|panicked at' "$single_output" | head -8 | sed 's/^/     /'   # gate-lint:detail
      echo "     → 怎么办：这几条按挂钟判，单线程下仍然红就是真的慢了或断言写错；改完单独核：cd $R && cargo test --release --all-features -- --test-threads=1 <函数名>"
      exit 1
    fi
    for name in "${registered_wallclock[@]}"; do
      grep -Eq -- "^test (.*::)?${name} \\.\\.\\. ok" "$single_output" || missing_single+=("$name")
    done
    if ((${#missing_single[@]} > 0)); then
      echo "  ✗ 登记了的挂钟断言 ${#missing_single[@]} 条（${missing_single[*]}）在单线程那一趟的输出里没有「test … ok」行：那一趟没起或没跑到它们"
      echo "     → 怎么办：看登记名与测试函数名是不是一字不差（判法 python3 scripts/gate/wallclock-assert-lint.py --list . 列登记名），登记位挂在没有这个测试的包里也会这样"
      exit 1
    fi
  fi
  ((plan_ok && first_pass_needed)) && research_unit_tests_record "$scratch" "$output_file" "$scratch/wallclock.names"
  if ((unit_service_unavailable && ! unit_plan_red)); then
    echo "  ✗ research-unit-tests 这一格：$b 个测试批次共 $n 个单测都过了，判绿记库时 prune-server 不可用（上面那一句），判定没记下，这一格退 75"
    echo "     → 怎么办：python3 prune-server/control check 看服务，起不来报给主 agent；修好再只跑这一格：bash .claude/gate.d/research-build-and-replay.sh --check research-unit-tests"
    exit 75
  fi
  if ((unit_plan_red)); then
    echo "  ✗ research-unit-tests 这一格判红：单测按条复用失灵（上面「按条复用失灵」那一句点名了哪一条哨兵、或 admission.py 退了几）；这一趟照旧全部现判跑完、$b 个测试批次共 $n 个单测都过了"
    echo "     → 怎么办：照那一句的出路修 scripts/gate/admission.py 点名的那一段，再只跑这一格：bash .claude/gate.d/research-build-and-replay.sh --check research-unit-tests；不能复用就判红，不退回全部现判当绿"
    exit 1
  fi
  unit_counts="单测按条：现判 ${unit_plan_judge}、复用 ${unit_plan_reuse}、拒算 ${unit_plan_refused}"
  ((plan_ok)) || unit_counts="单测按条复用算不出，全部现判"
  ((${#registered_wallclock[@]} > 0)) && ! research_build_broken wallclock-single-run-dropped && unit_counts+="；挂钟断言 ${#registered_wallclock[@]} 条在单线程那一趟跑了（${registered_wallclock[*]}），不记库"
  if ((first_pass_needed)); then
    echo "  ✓ research 构建与单测全绿：$b 个测试批次、共 $n 个单测（${unit_counts}）"
  else
    echo "  ✓ research 构建与单测全绿：单测 ${unit_plan_reuse} 条都有键相同的判定，全部复用，第一趟没起 cargo test（${unit_counts}）"
  fi
}

# ── wallclock-asserts-registered ──
cell_wallclock_asserts_registered() {
  local lint="$STAGE_REPOSITORY/scripts/gate/wallclock-assert-lint.py" out rc lagging
  command -v python3 >/dev/null || { echo "  ✗ 没有 python3，跑不了挂钟断言登记位的判法"; echo "     → 怎么办：装 python3 或把它放进 PATH（门禁本身也要它）。"; exit 1; }
  [[ -f "$lint" ]] || { echo "  ✗ 没有 $lint：这一格的判法在它里面"; echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一格按红记。"; exit 1; }
  out="$(python3 "$lint" . 2>&1)"; rc=$?
  printf '%s\n' "$out"
  case "$rc" in
    0)
      lagging="$(grep -o '另有 [0-9]* 条在滞后表' <<<"$out" | grep -o '[0-9]*')"
      if [[ -n "$lagging" && -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 wallclock-asserts-registered 格：滞后表里 $lagging 条挂钟断言还没复核登记" >> "$GATE_NOT_RUN_FILE"; fi
      exit 0 ;;
    1|77) exit "$rc" ;;
    *)
      echo "  ✗ python3 scripts/gate/wallclock-assert-lint.py 退 $rc（0 全对、1 有红、2 用法错、77 无对象）：判不出结果"
      echo "     → 怎么办：照上面它的输出查；单独跑 python3 scripts/gate/wallclock-assert-lint.py . 看。"
      exit 1 ;;
  esac
}

# ── experiment-replay ──
# replay.sh 退 0 往下走；退 77（整趟全是本次不判的计时行：负载过高，它自己的汇总里有逐行说明）认成本次未跑，这一格退 77；别的退出码判红
run_replay() {
  local replay_rc replay_log replay_total judged reused refused load_skipped
  local -a break_setting=()
  # 弄坏开关 row-key-with-product-name（文件头）：交给 replay.sh 起的 admission.py（ADMISSION_BREAK），行键另加产物文件名
  research_build_broken row-key-with-product-name && break_setting=(ADMISSION_BREAK=row-key-with-product-name)
  # RocksDB 预编库的取值（scripts/hosts/rocksdb-prebuilt.py 文件头「entry」）：replay.sh 编 验证包的装置与 E162 那几个 bin（都编到 librocksdb-sys），
  #   起它之前判一次这台装了、此刻进程环境的取值对得上，判红这一格退 1、replay.sh 不起（一个 cargo 都不起）。树里没有依赖 rocksdb 的清单（别的样本）、
  #   PATH 上是带标记的假 cargo，entry 报不判。弄坏开关 rocksdb-entry-unchecked 下不判（样本 experiment-replay-rocksdb-entry-refused 判错）
  #   这一份被拷到不带 scripts/hosts/ 的树里时报一行没判、往下走
  if ! research_build_broken rocksdb-entry-unchecked && [[ ! -f "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" ]]; then
    echo "  ! RocksDB 预编库的取值没判：$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py 不在（这一份阶段被拷到了不带 scripts/hosts/ 的树里）"
  elif ! research_build_broken rocksdb-entry-unchecked && ! python3 "$STAGE_REPOSITORY/scripts/hosts/rocksdb-prebuilt.py" entry --root "$ROOT"; then
    echo "  ✗ 复跑：起 replay.sh（它编到 librocksdb-sys）之前 RocksDB 预编库的取值没判过（上面 rocksdb-prebuilt.py entry 的红句），一个实验都没跑"
    echo "    → 怎么办：照上面的出路修（开新会话，或命令前 eval \"\$(python3 scripts/hosts/rocksdb-prebuilt.py env)\"），再只跑这一格：bash .claude/gate.d/research-build-and-replay.sh --check experiment-replay"
    exit 1
  fi
  replay_log="$(mktemp)"
  env ${break_setting[@]+"${break_setting[@]}"} bash "$REPLAY_SCRIPT" "$@" | tee "$replay_log"
  replay_rc="${PIPESTATUS[0]}"
  # 按行复用（replay.sh 文件头「按行复用」）：照它末尾那一行 REPLAY_TOTAL 报现判、复用与负载高没判的行数；没有那一行报取不出，不写 0
  replay_total="$(grep '^REPLAY_TOTAL ' "$replay_log" | tail -n 1)"
  rm -f -- "${replay_log:?}"
  if [[ -n "$replay_total" ]] && ! research_build_broken replay-reuse-unreported; then
    judged="$(sed -n 's/.* judged=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    reused="$(sed -n 's/.* reused=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    refused="$(sed -n 's/.* refused=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    load_skipped="$(sed -n 's/.* load_skipped=\([0-9]*\).*/\1/p' <<<"$replay_total")"
    echo "  · 复跑：现判 ${judged:-取不出} 行、复用 ${reused:-取不出} 行、拒算 ${refused:-取不出} 行、负载高没判 ${load_skipped:-取不出} 行（replay.sh 按行复用：键相同、判过的那一行不跑，见它文件头「按行复用」；留存产物不在树里的行它从 git 历史取回再比，取不回判红）"
  elif ! research_build_broken replay-reuse-unreported; then
    echo "  ! 复跑：复用数取不出（replay.sh 没打 REPLAY_TOTAL 行），这一趟现判几行、复用几行这一格报不出"
  fi
  case "$replay_rc" in
    0) return 0 ;;
    77)
      # replay.sh 退 77：整趟没有一行判得了，判的全是负载高本次不判的行（它文件头「驱动要读的输入」末句）
      local why_not_judged="负载高"
      echo "  ! 复跑整趟本次不判（${why_not_judged}）：replay.sh 退 77，一行都没比；不算通过也不算红"
      if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 experiment-replay 格：复跑整趟本次不判（${why_not_judged}）" >> "$GATE_NOT_RUN_FILE"; fi
      exit 77 ;;
    75)
      # replay.sh 退 75：它按行复用的判定库在 prune-server 里，服务不可用（它文件头「按行复用」）；不判红也不判绿。弄坏开关 replay-unavailable-as-red 下照红
      if research_build_broken replay-unavailable-as-red; then exit 1; fi
      echo "  ✗ 复跑没判：prune-server 不可用（replay.sh 退 75，原因在上面），这一格退 75"
      echo "     → 怎么办：python3 prune-server/control check 看服务，修好、重起之后只重跑这一格：bash .claude/gate.d/${STAGE_NAME} --check experiment-replay"
      exit 75 ;;
    *) exit 1 ;;
  esac
}

cell_experiment_replay() {
  local ALL_EXACT=()
  if ((${#named_experiments[@]})); then
    # 点名的实验：replay.sh 照样按行复用（键相同、判过的那一行不跑）
    run_replay "${named_experiments[@]}"
    echo "  ✓ 只复跑了点名的 ${#named_experiments[@]} 个实验：${named_experiments[*]}（其中准入判「输入没变」而没跑的，见上面 replay.sh 汇总里那一行）；这不是整格结论"
    exit 0
  fi
  # 这一格不整格问复用：replay.sh 按行复用（每一行的行键，它文件头「按行复用」），全部行都复用的一趟不起编译、不清场
  # 跑哪些、没跑哪些都从 replay.sh 的表里现算，不写死清单：写死的清单漏掉新加的实验，「没跑」那句就成了假话
  # （.claude/singlefs-ai-sop/rules/show-me-test.md「门禁不许假装通过」）。
  mapfile -t ALL_EXACT < <(replay_rows_of_kind exact)
  if ((${#ALL_EXACT[@]} == 0)); then
    echo "  ✗ 从 scripts/experiment-apparatus/replay.sh 里一行都没读到——表的格式变了？"
    echo "    → 本阶段靠 '编号|二进制|参数|产物|exact' 这个形状取行，改表格式要同时改这里。"
    exit 1
  fi
  # 不指定范围：表里全部行（exact 与 timing）都交给 replay.sh，一个都不剔；timing 行照 replay.sh 自己的判法（它文件头「本次未跑」：负载过高的计时行记本次不判）
  run_replay
}

# ── mutation-tables ──
research_build_broken() { [[ ",${RESEARCH_BUILD_AND_REPLAY_BREAK:-}," == *",$1,"* ]]; }

# mutation_counts_of_log <mutate.sh 的输出>：它末尾「计数：」那一行里的现判、复用、拒算条数，打成「<现判> <复用> <拒算>」；取不出打空串
mutation_counts_of_log() {
  sed -n 's/^计数：.*、现判 \([0-9]*\) 条、复用 \([0-9]*\) 条、拒算 \([0-9]*\) 条$/\1 \2 \3/p' "$1" | tail -n 1
}

cell_mutation_tables() {
  local tables_directory=scripts/mutation/tables mutate_script="$STAGE_REPOSITORY/scripts/mutation/mutate.sh"
  local sample_mode=0 rounds=1 round library="" target_directory scratch cleanup_paths="" library_root_for_break=""
  local table stem binary rc log rows line counts fresh reused refused table_library first_lines
  local fresh_total reused_total refused_total tables_judged tables_all_reused caught_rows red_total
  local -a table_library=()
  local -a not_run=() red_lines=() red_tables=() red_rows=() broken_environment=() library_setting=()
  [[ -d "$R" && -d "$tables_directory" ]] || { echo "  ! 没有 $R 或 $tables_directory，这一格无对象可判"; exit 77; }
  if ! command -v cargo >/dev/null; then
    echo "  ✗ 没有 cargo：变异表的替换文编不编得过、测试抓不抓得到判不了"
    echo "     → 怎么办：装 Rust 工具链或把已装的 cargo 放进 PATH，再只重跑这一格：bash .claude/gate.d/${STAGE_NAME} --check mutation-tables"
    exit 1
  fi
  if [[ ! -f "$mutate_script" ]]; then
    echo "  ✗ 没有 $mutate_script：这一格的判法（逐条施加替换文、跑测试、--binary-of）与按条判定库都在它里面"
    echo "     → 怎么办：它随仓走，被删了或挪了就从 git 找回来；找回之前这一格按红记。"
    exit 1
  fi
  scratch="$(mktemp -d)" || { echo "  ✗ 建不了临时目录"; echo "     → 怎么办：看 \$TMPDIR 满没满、有没有写权限，修好再跑。"; exit 1; }
  cleanup_paths="$(printf '%q' "$scratch")"
  # 编译目录一律用这一格自己的临时目录，跑完删；样本（被判的根不是这道阶段所在的仓、也不是 --staged）另把按条判定库也放这里；
  # 只有 run-twice 那份接连判两趟（同一份库，第二趟照条复用），另放了 second-round/ 的在第二趟之前把它下面的文件盖到样本根上（改一条替换文）
  if [[ "$(pwd -P)" != "$(cd "$STAGE_REPOSITORY" && pwd -P)" && -z "${GATE_STAGED_FROM:-}" ]]; then sample_mode=1; fi
  target_directory="$scratch/target"   # 这一格自己的编译目录（文件头 mutation-tables），收尾随 $scratch 删
  if ((sample_mode != 0)); then
    library="$scratch/library"
    library_setting=("MUTATE_LIBRARY_DIRECTORY=$library")
    [[ -f .mutation-tables-sample/run-twice ]] && rounds=2
  fi
  # shellcheck disable=SC2064  # 路径在这里就展开（%q 过）：退出时 local 变量已经不在
  trap "rm -rf -- $cleanup_paths" EXIT
  # 弄坏开关 no-cap-lints（文件头）：PATH 前面放垫片 cargo，剥掉 RUSTFLAGS 与 CARGO_ENCODED_RUSTFLAGS 再交给真 cargo，mutate.sh 带的 cap 一条都到不了 rustc
  if research_build_broken no-cap-lints; then
    local real_cargo
    real_cargo="$(command -v cargo)"
    mkdir "$scratch/no-cap-bin" && printf '#!/usr/bin/env bash\nexec env -u RUSTFLAGS -u CARGO_ENCODED_RUSTFLAGS %q "$@"\n' "$real_cargo" >"$scratch/no-cap-bin/cargo" \
      && chmod +x "$scratch/no-cap-bin/cargo" || { echo "  ✗ 弄坏开关 no-cap-lints 建不了垫片 cargo"; echo "     → 怎么办：看自证临时目录能不能写、PATH 上有没有 cargo，修好再跑自证"; exit 1; }
    broken_environment=("PATH=$scratch/no-cap-bin:$PATH")
  fi
  # 弄坏开关 mutation-key-whole-table（文件头）：按条判定库按整张表的 sha256 分开放（表里改一条，整张表的条都作废）
  if research_build_broken mutation-key-whole-table; then
    library_root_for_break="$scratch/singlefs-mutate-library-by-table"   # 弄坏开关只在样本根上认（共用库 lib/stage-cells.sh），样本的库都在这一格的临时目录里"
  fi
  # E162 的 rocksdb 要 BINDGEN_EXTRA_CLANG_ARGS（mutate.sh 文件头 required-features 那一段：由调用方设）；没设就取放量共用库里那一份
  if [[ -z "${BINDGEN_EXTRA_CLANG_ARGS:-}" && -f "$STAGE_REPOSITORY/scripts/crash/crash-amplification-lib.sh" ]]; then
    source "$STAGE_REPOSITORY/scripts/crash/crash-amplification-lib.sh"
    BINDGEN_EXTRA_CLANG_ARGS="${CRASH_AMPLIFICATION_BINDGEN_INCLUDE_ARGUMENTS:-}"
    export BINDGEN_EXTRA_CLANG_ARGS
  fi
  for ((round = 1; round <= rounds; round++)); do
    if ((rounds > 1)); then
      if ((round == 2)) && [[ -d .mutation-tables-sample/second-round ]]; then
        cp -dR --preserve=timestamps,links .mutation-tables-sample/second-round/. . || { echo "  ✗ 样本：第二趟之前盖不上 .mutation-tables-sample/second-round/"; echo "     → 怎么办：核样本目录里的 second-round/ 在不在、能不能读，补齐样本再跑"; exit 1; }
        echo "  ── 第 2 趟之前把 .mutation-tables-sample/second-round/ 下的文件盖到样本根上（改了表里一条）"
      fi
      echo "  ── 第 $round 趟（样本 .mutation-tables-sample/run-twice：按条判定库 $library，第二趟没改的条应当复用）"
    fi
    not_run=(); red_lines=(); red_tables=(); fresh_total=0; reused_total=0; refused_total=0; tables_judged=0; tables_all_reused=0; caught_rows=0; red_total=0
    for table in "$tables_directory"/*.tsv; do
      [[ -f "$table" ]] || continue
      stem="$(basename "$table" .tsv)"
      if [[ ! -f "$R/$stem.rs" ]]; then
        not_run+=("$table（$R/ 下没有同名的 $stem.rs，取不出 bin、源文件、表三元组）")
        continue
      fi
      if ! binary="$(cd "$R" && bash "$mutate_script" --binary-of "$stem.rs" 2>&1)"; then
        not_run+=("$table（mutate.sh --binary-of $stem.rs 没给出 bin 名：$binary）")
        continue
      fi
      table_library=(${library_setting[@]+"${library_setting[@]}"})
      if [[ -n "$library_root_for_break" ]]; then table_library=("MUTATE_LIBRARY_DIRECTORY=$library_root_for_break/$(sha256sum "$table" | cut -c1-64)"); fi
      log="$scratch/$stem.log"
      if (cd "$R" && env ${broken_environment[@]+"${broken_environment[@]}"} ${table_library[@]+"${table_library[@]}"} MUTATE_TARGET_DIR="$target_directory" \
            bash "$mutate_script" "$binary" "$stem.rs" "../mutation/tables/$stem.tsv") >"$log" 2>&1; then rc=0; else rc=$?; fi
      tables_judged=$((tables_judged + 1))
      # 弄坏开关 not-caught-ignored：退 1（没红、没跑完、盲区）也当绿，❌ 💥 ⚠️ 行不收
      research_build_broken not-caught-ignored && ((rc == 1)) && rc=0
      counts="$(mutation_counts_of_log "$log")"
      if [[ -n "$counts" ]]; then
        read -r fresh reused refused <<<"$counts"
        fresh_total=$((fresh_total + fresh)); reused_total=$((reused_total + reused)); refused_total=$((refused_total + refused))
        ((fresh + refused == 0)) && tables_all_reused=$((tables_all_reused + 1))
        counts="现判 $fresh 条、复用 $reused 条、拒算 $refused 条"
      else
        counts="按条的现判、复用数取不出（mutate.sh 的「计数：」那一行没有那三样）"
      fi
      case "$rc" in
        0|1)
          # ❌ 没红（没有任何测试抓到）、💥 没跑完、⚠️ 抓名字有盲区、⏭ 替换文编不过（平常跑里就是一条无效变异）：逐条点名，整格红
          red_rows=()
          while IFS= read -r line; do
            case "$line" in
              "⏭"*) research_build_broken invalid-rows-ignored && continue ;;
              "❌"*|"💥"*|"⚠"*) research_build_broken not-caught-ignored && continue ;;
              *) continue ;;
            esac
            red_rows+=("$line")
          done < <(grep -E '^(❌|💥|⚠|⏭)' "$log")
          if ((${#red_rows[@]})); then
            for line in "${red_rows[@]}"; do
              red_lines+=("$table：$line")
              # 这一条这一趟为什么现判（mutate.sh 在 stderr 逐条写「现判：[<名>] <原因>」，键对不上时原因里是变了的单元）
              first_lines="$(grep -F "mutate: 现判：[$(sed -n 's/^[^[]*\[\([^]]*\)\].*/\1/p' <<<"$line")]" "$log" | head -n 1 | sed 's/^mutate: 现判：//')"
              [[ -n "$first_lines" ]] && red_lines+=("    这一条为什么现判：${first_lines}")
            done
            red_total=$((red_total + ${#red_rows[@]}))
            echo "  · $table：${#red_rows[@]} 条没被测试抓到或编不过（$counts）"
          elif ((rc == 1)); then
            red_tables+=("$table：mutate.sh $binary $stem.rs 退 1 而一行 ❌ 💥 ⚠️ ⏭ 都没有；输出末尾：$(tail -n 3 "$log" | tr '\n' ' ')")
          else
            rows="$(grep -c '^✅ ' "$log")"
            caught_rows=$((caught_rows + rows))
            echo "  · $table：$rows 条全被测试抓到（$counts）"
          fi
          ;;
        *)
          red_tables+=("$table：mutate.sh $binary $stem.rs 退 $rc（2 基线红或用法错、3 锚点没命中、4 还原之后没回到全绿、5 跑的时候源文件被改、6 bin 与源文件对不上、8 收回的判定对不上、75 判定库服务不可用）；输出末尾：$(tail -n 4 "$log" | tr '\n' ' ')")
          ;;
      esac
    done
    if ((${#red_lines[@]} + ${#red_tables[@]})); then
      for line in ${red_lines[@]+"${red_lines[@]}"}; do
        echo "  ✗ $line"   # gate-lint:detail
      done
      for line in ${red_tables[@]+"${red_tables[@]}"}; do
        echo "  ✗ $line"   # gate-lint:detail
      done
      echo "  ✗ 研究侧变异表有 $red_total 条没被测试抓到或编不过、${#red_tables[@]} 张表没判完（按条：现判 $fresh_total 条、复用 $reused_total 条、拒算 $refused_total 条）"
      echo "     → 怎么办：没被抓到的补一条能抓到它的测试，或改那一行替换文（编得过、仍改被测行为）；编不过的改替换文或把装置改回能编；没判完的照括号里 mutate.sh 的退出码办。"
      echo "               单张核：(cd scripts/experiment-apparatus && bash ../mutation/mutate.sh <bin> <源文件> ../mutation/tables/<表>)"
      exit 1
    fi
  done
  for line in ${not_run[@]+"${not_run[@]}"}; do
    echo "  ! 本次未跑：$line"
    if [[ -n "${GATE_NOT_RUN_FILE:-}" ]]; then printf '%s\n' "$STAGE_NAME 的 mutation-tables 格没判 $line" >> "$GATE_NOT_RUN_FILE"; fi
  done
  if ((tables_judged == 0)); then
    echo "  ! 本次无对象可判：$tables_directory 下没有一张表对得上 $R/ 下同名的 .rs（没跑 ${#not_run[@]} 张，上面逐张列名）"
    exit 77
  fi
  echo "  ✓ 研究侧变异表逐条照跑测试全被抓到：判了 $tables_judged 张表（整张复用、一条都没跑的 $tables_all_reused 张），按条：现判 $fresh_total、复用 $reused_total、拒算 $refused_total、本次未跑 ${#not_run[@]} 张表（上面「本次未跑」逐张列名）"
}

stage_cells_run "$ROOT"
